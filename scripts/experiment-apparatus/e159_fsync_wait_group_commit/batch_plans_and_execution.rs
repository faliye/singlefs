//! 单元种类、区域、批计划与三条臂的批计划构造、单元缓冲与区域游标、写任务的展开与批的执行。

use e7_index_bench::{truncate_f64_to_u64, usize_from_u64};
use super::*;

// ============================================================
// 单元种类、随机取整、批计划
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(super) enum UnitKind {
    Data,
    ExtentNode,
    InodeContainer,
    InodeRootNode,
    AllocationNode,
    AccountingNode,
    MappingNode,
    TreeTableNode,
    Record,
    RootSlot,
    SystemConfigurationSlot,
}
impl UnitKind {
    pub(super) fn width_bytes(self) -> u64 {
        match self {
            UnitKind::Data | UnitKind::InodeContainer => UNIT_BYTES,
            UnitKind::ExtentNode | UnitKind::InodeRootNode | UnitKind::AllocationNode | UnitKind::AccountingNode | UnitKind::MappingNode | UnitKind::TreeTableNode => NODE_BYTES,
            UnitKind::Record => RECORD_BYTES,
            UnitKind::RootSlot => PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES,
            UnitKind::SystemConfigurationSlot => SYSTEM_CONFIGURATION_SLOT_BYTES,
        }
    }
    /// 落在哪个区域（5.3）：单元区还是 journal 环。根槽与系统配置槽走各自专用的槽位，不走游标。
    fn region(self) -> Region {
        match self {
            UnitKind::Record => Region::Journal,
            UnitKind::RootSlot | UnitKind::SystemConfigurationSlot => Region::Configuration,
            UnitKind::Data
            | UnitKind::ExtentNode
            | UnitKind::InodeContainer
            | UnitKind::InodeRootNode
            | UnitKind::AllocationNode
            | UnitKind::AccountingNode
            | UnitKind::MappingNode
            | UnitKind::TreeTableNode => Region::Unit,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Region {
    Unit,
    Journal,
    Configuration,
}

/// 期望个数 = 这一种的字节（两盘合计）÷ REPLAY_DEVICE_COUNT ÷ 单元宽（第五节 5.3）。
fn expected_count_from_both_device_bytes(both_device_bytes: u64, unit_width_bytes: u64) -> f64 {
    let single_device_bytes = both_device_bytes / DEVICE_COUNT; // 整除：模型自己乘过 DEVICE_COUNT
    (single_device_bytes as f64) / (unit_width_bytes as f64) * (REPLAY_DEVICE_COUNT as f64)
}

/// 随机取整：整数部分 + 以小数部分为概率再加 1（第五节 5.3；M7 的变异目标）。
pub(super) fn round_stochastic(expected: f64, random_state: &mut u64) -> u64 {
    assert!(expected >= 0.0, "期望个数不该是负的：{expected}");
    let floor_value = expected.floor();
    let fractional_part = expected - floor_value;
    let draw = uniform_unit_interval(random_state);
    truncate_f64_to_u64(floor_value) + u64::from(draw < fractional_part)
}


/// 一批要写的东西：每种单元的实际个数（已随机取整）+ 记录数 + 写不写根槽 / 系统配置槽。
#[derive(Clone, Debug, Default)]
pub(super) struct BatchPlan {
    unit_counts: Vec<(UnitKind, u64)>,
    pub(super) record_count: u64,
    writes_root_slot: bool,
    writes_system_configuration_slot: bool,
}
impl BatchPlan {
    /// W：这一批实际写出、且被记录点名的单元总数（不含记录、根槽、系统配置槽自己）。
    pub(super) fn named_unit_total(&self) -> u64 {
        self.unit_counts.iter().map(|&(_, count)| count).sum()
    }
    pub(super) fn total_write_calls(&self) -> u64 {
        self.unit_counts.iter().map(|&(_, count)| count).sum::<u64>() + self.record_count + u64::from(self.writes_root_slot) + u64::from(self.writes_system_configuration_slot)
    }
}

/// 三条候选臂（第二节 2.3、第五节 5.2）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    Jia,
    WriteAheadLogFull,
    WriteAheadLogLeaf,
}
impl Arm {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Arm::Jia => "jia",
            Arm::WriteAheadLogFull => "wal_full-K10",
            Arm::WriteAheadLogLeaf => "yi-M-K10",
        }
    }
    pub(super) fn write_ahead_log_arm(self) -> Option<WriteAheadLogArm> {
        match self {
            Arm::Jia => None,
            Arm::WriteAheadLogFull => Some(WriteAheadLogArm::WriteAheadLogFull),
            Arm::WriteAheadLogLeaf => Some(WriteAheadLogArm::WriteAheadLogLeaf),
        }
    }
}

/// 主几何（第五节 5.1）：P=10⁴、F8A、共享脊柱、seq、L均、φ=1、K3、g=24、pbs=512。
pub(super) fn main_shape(file_count: u64, family: Family) -> PoolShape {
    PoolShape { file_count, family, placement: Placement::Sequential, policy: PositionPolicy::Balanced, geometry: Geometry::main() }
}

/// 甲一批的计划（第五节 5.2「怎么做」+ 5.3「取整」）。
pub(super) fn plan_jia_batch(shape: PoolShape, concurrent_fsync_count: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_jia_batch_publish(shape, concurrent_fsync_count, shared);
    let fields: [(UnitKind, u64); 8] = [
        (UnitKind::Data, outcome.data_bytes),
        (UnitKind::ExtentNode, outcome.extent_bytes),
        (UnitKind::InodeContainer, outcome.inode_container_bytes),
        (UnitKind::InodeRootNode, outcome.inode_root_bytes),
        (UnitKind::AllocationNode, outcome.allocation_bytes),
        (UnitKind::AccountingNode, outcome.accounting_bytes),
        (UnitKind::MappingNode, outcome.mapping_bytes),
        (UnitKind::TreeTableNode, outcome.tree_table_bytes),
    ];
    let mut unit_counts = Vec::with_capacity(8);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
    let record_count = named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    BatchPlan { unit_counts, record_count, writes_root_slot: true, writes_system_configuration_slot: true }
}

/// WAL 两臂一批 fsync 的计划（不发根、不写固定点；第五节 5.2）。
pub(super) fn plan_write_ahead_log_fsync_batch(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_write_ahead_log_batch_fsync(shape, arm, concurrent_fsync_count, shared);
    let fields: [(UnitKind, u64); 3] = [(UnitKind::Data, outcome.data_bytes), (UnitKind::ExtentNode, outcome.extent_bytes), (UnitKind::InodeContainer, outcome.inode_container_bytes)];
    let mut unit_counts = Vec::with_capacity(4);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let record_count = match arm {
        // wal_full 恒点名两棵新树根，不随 k 变（第五节 5.3；M13 之外另一条不变量，未在变异表登记，
        // 因为它是臂定义本身，见第五节 5.2「wal_full-K10」）。
        WriteAheadLogArm::WriteAheadLogFull => {
            let inode_root_expected = expected_count_from_both_device_bytes(outcome.inode_root_bytes, UnitKind::InodeRootNode.width_bytes());
            let inode_root_actual = round_stochastic(inode_root_expected, random_state);
            if inode_root_actual > 0 {
                unit_counts.push((UnitKind::InodeRootNode, inode_root_actual));
            }
            1
        }
        WriteAheadLogArm::WriteAheadLogLeaf => {
            let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
            named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1)
        }
    };
    BatchPlan { unit_counts, record_count, writes_root_slot: false, writes_system_configuration_slot: false }
}

/// WAL 两臂一次 checkpoint 的计划（第五节 5.2；发根、写四样固定点）。
#[allow(clippy::too_many_arguments, reason = "一次 checkpoint 的计划要这些输入（形状、臂、并发数、共享与否、间隔批数、策略……），拆结构体不会少传哪一项")]
pub(super) fn plan_write_ahead_log_checkpoint(shape: PoolShape, arm: WriteAheadLogArm, concurrent_fsync_count: u64, shared: bool, batches_in_interval: u64, policy: IntermediateVersionPolicy, random_state: &mut u64) -> BatchPlan {
    let outcome = solve_write_ahead_log_batch_checkpoint(shape, arm, concurrent_fsync_count, shared, batches_in_interval, policy);
    let fields: [(UnitKind, u64); 6] = [
        (UnitKind::ExtentNode, outcome.extent_catch_up_bytes),
        (UnitKind::InodeRootNode, outcome.inode_catch_up_bytes),
        (UnitKind::AllocationNode, outcome.allocation_bytes),
        (UnitKind::AccountingNode, outcome.accounting_bytes),
        (UnitKind::MappingNode, outcome.mapping_bytes),
        (UnitKind::TreeTableNode, outcome.tree_table_bytes),
    ];
    let mut unit_counts = Vec::with_capacity(6);
    for (kind, both_device_bytes) in fields {
        let expected = expected_count_from_both_device_bytes(both_device_bytes, kind.width_bytes());
        unit_counts.push((kind, round_stochastic(expected, random_state)));
    }
    let named_unit_total: u64 = unit_counts.iter().map(|&(_, count)| count).sum();
    let record_count = named_unit_total.max(1).div_ceil(NAMED_ITEMS_PER_RECORD_MAIN).max(1);
    BatchPlan { unit_counts, record_count, writes_root_slot: true, writes_system_configuration_slot: true }
}

// ============================================================
// 预生成缓冲、区域游标、批的执行（单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → [放行] → 系统配置槽）
// ============================================================

/// 每种单元宽度各预生成一块固定伪随机内容的缓冲，写的时候共享只读，不逐次现生成
/// （第五节 5.3「内容是固定种子生成的伪随机字节」；内容不需要逐次唯一）。
pub(super) struct UnitBuffers {
    data: AlignedBuffer,
    node: AlignedBuffer,
    record: AlignedBuffer,
    root_slot: AlignedBuffer,
    system_configuration_slot: AlignedBuffer,
}
impl UnitBuffers {
    pub(super) fn new(seed: u64) -> UnitBuffers {
        UnitBuffers {
            data: AlignedBuffer::new_with_seed(usize_from_u64(UNIT_BYTES), seed ^ 1),
            node: AlignedBuffer::new_with_seed(usize_from_u64(NODE_BYTES), seed ^ 2),
            record: AlignedBuffer::new_with_seed(usize_from_u64(RECORD_BYTES), seed ^ 3),
            root_slot: AlignedBuffer::new_with_seed(usize_from_u64(PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES), seed ^ 4),
            system_configuration_slot: AlignedBuffer::new_with_seed(usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), seed ^ 5),
        }
    }
    fn buffer_for(&self, kind: UnitKind) -> &[u8] {
        match kind {
            UnitKind::Data | UnitKind::InodeContainer => self.data.as_ref(),
            UnitKind::ExtentNode | UnitKind::InodeRootNode | UnitKind::AllocationNode | UnitKind::AccountingNode | UnitKind::MappingNode | UnitKind::TreeTableNode => self.node.as_ref(),
            UnitKind::Record => self.record.as_ref(),
            UnitKind::RootSlot => self.root_slot.as_ref(),
            UnitKind::SystemConfigurationSlot => self.system_configuration_slot.as_ref(),
        }
    }
}

/// 单元区、journal 环各一个顺序游标，写满回绕（第五节 5.3）；根槽按发布序号轮着用 24 个位置；
/// 系统配置槽在两个物理槽之间交替。
pub(super) struct RegionCursors {
    unit_cursor_bytes: AtomicU64,
    journal_cursor_bytes: AtomicU64,
    publish_counter: AtomicU64,
    system_configuration_next_slot: AtomicU64,
}
impl RegionCursors {
    pub(super) fn new() -> RegionCursors {
        RegionCursors { unit_cursor_bytes: AtomicU64::new(0), journal_cursor_bytes: AtomicU64::new(0), publish_counter: AtomicU64::new(0), system_configuration_next_slot: AtomicU64::new(0) }
    }
    fn claim_unit_offset(&self, width_bytes: u64) -> u64 {
        let raw = self.unit_cursor_bytes.fetch_add(width_bytes, Ordering::Relaxed);
        UNIT_REGION_OFFSET_BYTES + raw % UNIT_REGION_BYTES
    }
    fn claim_journal_offset(&self, width_bytes: u64) -> u64 {
        let raw = self.journal_cursor_bytes.fetch_add(width_bytes, Ordering::Relaxed);
        JOURNAL_REGION_OFFSET_BYTES + raw % JOURNAL_REGION_BYTES
    }
    fn claim_root_slot_offset(&self) -> u64 {
        let index = self.publish_counter.fetch_add(1, Ordering::Relaxed) % ROOT_SLOT_COUNT;
        ROOT_SLOT_BASE_OFFSET_BYTES + index * ROOT_SLOT_STRIDE_BYTES
    }
    /// 系统配置槽：先读两个槽（生成号语义不需要，只为读次数与次序对拍），再写「下一个」那个。
    fn system_configuration_offsets(&self) -> (u64, u64, u64) {
        let next = self.system_configuration_next_slot.fetch_add(1, Ordering::Relaxed) % 2;
        let write_offset = CONFIGURATION_REGION_OFFSET_BYTES + if next == 0 { SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES } else { SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES };
        (CONFIGURATION_REGION_OFFSET_BYTES + SYSTEM_CONFIGURATION_SLOT_ZERO_OFFSET_BYTES, CONFIGURATION_REGION_OFFSET_BYTES + SYSTEM_CONFIGURATION_SLOT_ONE_OFFSET_BYTES, write_offset)
    }
}

/// 一批展开成的写任务：offset 已经从游标领到，kind 决定用哪块缓冲、算不算「持久点」的一部分。
struct WriteJob {
    offset: u64,
    kind: UnitKind,
}

fn expand_jobs(plan: &BatchPlan, cursors: &RegionCursors) -> Vec<WriteJob> {
    let mut jobs = Vec::with_capacity(usize_from_u64(plan.unit_counts.iter().map(|&(_, count)| count).sum::<u64>()));
    for &(kind, count) in &plan.unit_counts {
        for _ in 0..count {
            jobs.push(WriteJob { offset: cursors.claim_unit_offset(kind.width_bytes()), kind });
        }
    }
    jobs
}

fn expand_record_jobs(plan: &BatchPlan, cursors: &RegionCursors) -> Vec<WriteJob> {
    (0..plan.record_count).map(|_| WriteJob { offset: cursors.claim_journal_offset(RECORD_BYTES), kind: UnitKind::Record }).collect()
}

/// 把一批写任务并发交给最多 `worker_count` 个线程，全部完成才返回（第五节 5.3「同一阶段的写并发交出去」）。
fn execute_phase_parallel(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, jobs: &[WriteJob], worker_count: usize) {
    if jobs.is_empty() {
        return;
    }
    let chunk_size = jobs.len().div_ceil(worker_count.max(1));
    std::thread::scope(|scope| {
        for chunk in jobs.chunks(chunk_size) {
            scope.spawn(move || {
                for job in chunk {
                    device.write_at(job.offset, buffers.buffer_for(job.kind), job.kind);
                }
            });
        }
    });
}

/// 持久点之前的全部步骤：单元 → 屏障 → 记录 → 屏障 → （甲/checkpoint）根槽写 + FUA。
/// 返回 made_durable_at。WAL 一批 fsync（`writes_root_slot=false`）在第二道屏障之后就是 made_durable_at。
pub(super) fn execute_batch_persist(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, plan: &BatchPlan, worker_count: usize) -> Instant {
    let unit_jobs = expand_jobs(plan, cursors);
    execute_phase_parallel(device, buffers, &unit_jobs, worker_count);
    device.barrier();
    // M2 的变异目标：把返回值换成这个早期时刻，就是「在第一道屏障之后就放行，不等根槽」。
    let after_first_barrier = Instant::now();
    let record_jobs = expand_record_jobs(plan, cursors);
    execute_phase_parallel(device, buffers, &record_jobs, worker_count);
    device.barrier();
    if plan.writes_root_slot {
        let root_offset = cursors.claim_root_slot_offset();
        device.write_at(root_offset, buffers.buffer_for(UnitKind::RootSlot), UnitKind::RootSlot);
        device.barrier(); // FUA = 写完立刻 sync_data（`block_device.rs:519-520`，第三节抄）
    }
    let made_durable_at = Instant::now();
    debug_assert!(made_durable_at >= after_first_barrier);
    made_durable_at
}

/// 持久点、放行调用方之后才做的一步：系统配置槽先读两个槽、再写一个（第五节 5.2）。
pub(super) fn execute_batch_finalize_system_configuration(device: &(dyn BlockDevice + Sync), buffers: &UnitBuffers, cursors: &RegionCursors, plan: &BatchPlan) {
    if !plan.writes_system_configuration_slot {
        return;
    }
    let (slot_zero, slot_one, write_offset) = cursors.system_configuration_offsets();
    let _ = device.read_at(slot_zero, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), UnitKind::SystemConfigurationSlot);
    let _ = device.read_at(slot_one, usize_from_u64(SYSTEM_CONFIGURATION_SLOT_BYTES), UnitKind::SystemConfigurationSlot);
    device.write_at(write_offset, buffers.buffer_for(UnitKind::SystemConfigurationSlot), UnitKind::SystemConfigurationSlot);
}


impl BatchPlan {
    /// 这一批实际会往回放盘上写的字节（单盘；供 S4 写量预算与 Q9 的 journal 字节计数用）。
    pub(super) fn approximate_bytes_single_device(&self) -> u64 {
        let unit_bytes: u64 = self.unit_counts.iter().map(|&(kind, count)| kind.width_bytes() * count).sum();
        let record_bytes = self.record_count * RECORD_BYTES;
        let root_bytes = if self.writes_root_slot { PRIMARY_PHYSICAL_BLOCK_SIZE_BYTES } else { 0 };
        let system_configuration_bytes = if self.writes_system_configuration_slot { SYSTEM_CONFIGURATION_SLOT_BYTES } else { 0 };
        unit_bytes + record_bytes + root_bytes + system_configuration_bytes
    }
}
