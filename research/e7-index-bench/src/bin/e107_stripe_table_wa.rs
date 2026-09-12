//! E107：条带表对主负载的写放大——`w ≥ 3` + 条带成员表 与 `w` 恒 2 全镜像，
//! 在 D25 主负载上每次发布各写多少字节，以及三段写序多几次串行等待。
//!
//! **它答的是** D2 已定项 12（parity 列的位置与发现机制）的存废那一格：条带成员表买到的是容量效率，
//! 代价是每次发布都要维护它；而 `w` 恒 2 全镜像根本不需要这张表。
//! 两者的**写字节**放在同一把尺子上是多少，全仓此前没有数。
//!
//! ## 三条臂
//!
//! | 臂 | 做什么 |
//! |---|---|
//! | 甲（字节模型） | `w = clamp(桶内格数 + 1, 2, 4)`（D2 已定项 6），按格宽分桶；每个格一条 56 字节成员记录（含 parity 格）；记录进码 3 容器，容器与容器索引整条链恒走 `w = 2` 镜像 |
//! | 丙（对照：`w` 恒 2 全镜像） | 数据与索引节点各写两份，**一条条带记录都不写** |
//! | 乙（真机延迟） | O_DIRECT，比「一次提交全部、等一次完成」与「先等数据+parity 完成、再提交记录容器」的端到端延迟 |
//!
//! ## 口径（跑前写死，改它就是新实验）
//!
//! 1. **一次发布** = D25 主负载：`N` 个数据叶（单元 32768）+ 一条共享脊柱上 4 个索引节点（16384）
//!    + 1 条 journal 记录（4096）+ 1 个根槽（512）。
//! 2. **journal 记录与根槽不进条带**：它们是 D22 已定项 8 的固定结构，各自有自己的冗余形态
//!    （根环 R = 3），**两条臂都按写一份计**——同加同减，不影响比较，但绝对值因此不是「盘上真实字节」。
//! 3. **parity 恒 1 格每条带**（D2 已定项 6 的 ⚠️ 逐字：上界 4 那套算术「只在 `parity = 1` 下成立」）。
//! 4. **成员记录一格一条**，因为记录里有「自己是第几列」这一段（E106 已钉的 56 字节拆分）。
//! 5. **`N = 0` 表示这一次没有发布**，两条臂的全部字节恒 0（阴性对照，作废条款 1）。
//! 6. 甲臂报两个总数：`no_redundancy`（基线 + 条带表，用来复核第三轮那份手算的 1.493×）与
//!    `with_redundancy`（再加 parity）。**只有后者与丙臂同尺**。
//!
//! ## 它不答什么（跑前写死）
//!
//! 只覆盖 D25 主负载这一个点，不覆盖 metaheavy / rand / multistream；不答加密开启后
//! 「锁盘期不修复」那条分支的代价；**不裁决三段写序里「写成功」是 ack 还是 durable**——两种各跑一遍；
//! 不答镜像两份落在不同盘对乙臂延迟的干扰（只测单盘）；空间效率那一维**不进判据**
//! （`.claude/rules/fs-design.md`「比谁省不构成任何判据」）。

use e7_index_bench::{Emitter, Sample};
use std::io::Write;
use std::time::Instant;

// ── 格式常量（每一个都指得到一条已定条款）────────────────────────────────
/// 数据单元恒 32768 含头（D4 已定项 1 / 已定项 5）。
const DATA_UNIT_SIZE_BYTES: u64 = 32768;
/// 索引节点 16384（D8 已定项 2）。
const NODE_SIZE_BYTES: u64 = 16384;
/// journal 记录 4 KiB（D23 已定项 12）。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 根槽 512（D22 已定项 2 的槽宽）。
const ROOT_SLOT_BYTES: u64 = 512;
/// D25 主负载：一次 fsync 带 8 叶、落在 1 条共享脊柱上。
const MAIN_LEAVES: u64 = 8;
/// 那条共享脊柱上被 COW 的索引节点数。
const SPINE_NODES: u64 = 4;
/// 码 3 打包记录容器 32768、头 107（D18 已定项 11）。
const PACKED_CONTAINER_HEADER_BYTES: u64 = 107;
/// 条带成员记录定长 56（E106 已钉：格宽标志 1 + w 1 + 自己是第几列 1 + 成员表 4×10 + 条带诞生代 8 + 补齐 5）。
const STRIPE_RECORD_BYTES: u64 = 56;
/// 容器索引节点头 76、条目 85（E106 / D18 已定项 11 口径）。
const CONTAINER_INDEX_HEADER_BYTES: u64 = 76;
const CONTAINER_INDEX_ENTRY_BYTES: u64 = 85;
/// 全池容器总数，用来钉容器索引的高（E106 口径：16 TiB / 90% 填充）。
const TOTAL_CONTAINERS: u64 = 828_789;
/// `w` 的上界，超级块声明的常量（D2 已定项 6）。
const STRIPE_WIDTH_MAXIMUM: u64 = 4;
/// `w` 的硬下界（D2 已定项 6：零冗余的条带不许发出）。
const STRIPE_WIDTH_MINIMUM: u64 = 2;

/// 一个容器装多少条成员记录。
fn records_per_container() -> u64 {
    (DATA_UNIT_SIZE_BYTES - PACKED_CONTAINER_HEADER_BYTES) / STRIPE_RECORD_BYTES
}

/// 容器索引一个节点装多少条目。
fn container_index_fanout() -> u64 {
    (NODE_SIZE_BYTES - CONTAINER_INDEX_HEADER_BYTES) / CONTAINER_INDEX_ENTRY_BYTES
}

/// 装得下 `container_count` 个容器要几层（`container_count <= 1` 时 1 层）。
fn container_index_height(container_count: u64) -> u32 {
    let fanout = container_index_fanout();
    let mut height_levels = 1u32;
    let mut containers_covered = fanout;
    while containers_covered < container_count {
        containers_covered = containers_covered.saturating_mul(fanout);
        height_levels += 1;
    }
    height_levels
}

/// D2 已定项 6：`w = clamp(桶内格数 + 1, 2, 4)`，再夹当时可写的设备数。
fn stripe_width_for(cells: u64, devices: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    let stripe_width = (cells + 1).clamp(STRIPE_WIDTH_MINIMUM, STRIPE_WIDTH_MAXIMUM);
    stripe_width.min(devices).max(STRIPE_WIDTH_MINIMUM)
}

/// 一个桶里 `cells` 个数据格、宽度 `stripe_width` ⇒ 几条条带。数据列数是 `stripe_width - 1`。
fn stripe_count_for(cells: u64, stripe_width: u64) -> u64 {
    if cells == 0 {
        return 0;
    }
    let data_columns = stripe_width - 1;
    cells.div_ceil(data_columns)
}

/// 一次发布的形状。
#[derive(Clone, Copy, Debug)]
struct Publish {
    leaf_count: u64,
    spine_node_count: u64,
}

impl Publish {
    fn main_workload(leaf_count: u64) -> Self {
        Publish {
            leaf_count,
            spine_node_count: if leaf_count == 0 { 0 } else { SPINE_NODES },
        }
    }
    /// 基线：不含任何冗余、不含条带表。
    fn baseline_bytes(&self) -> u64 {
        if self.leaf_count == 0 && self.spine_node_count == 0 {
            return 0;
        }
        self.leaf_count * DATA_UNIT_SIZE_BYTES + self.spine_node_count * NODE_SIZE_BYTES + JOURNAL_RECORD_BYTES + ROOT_SLOT_BYTES
    }
}

/// 甲臂一次发布的分解。
#[derive(Clone, Copy, Debug, PartialEq)]
struct StripeTableArmBreakdown {
    baseline_bytes: u64,
    parity_bytes: u64,
    /// 条带表那一份（容器 + 容器索引路径），**未镜像**。
    table_component_bytes: u64,
    /// 条带表整条链恒走 w=2 镜像之后的字节。
    table_mirrored_bytes: u64,
    /// 这次发布产生多少条成员记录（含 parity 格）。
    member_record_count: u64,
    /// 碰到几个容器。
    containers_touched: u64,
    data_stripe_width: u64,
    spine_stripe_width: u64,
}

impl StripeTableArmBreakdown {
    fn no_redundancy(&self) -> u64 {
        self.baseline_bytes + self.table_mirrored_bytes
    }
    fn with_redundancy(&self) -> u64 {
        self.baseline_bytes + self.parity_bytes + self.table_mirrored_bytes
    }
}

/// 甲臂：`w ≥ 3` + 条带成员表。
/// `occupied_slots_in_open_container` 是当前打开容器已占用的槽位数 ∈ [0, records_per_container()-1]。
fn publish_on_stripe_table_arm(publish: Publish, devices: u64, occupied_slots_in_open_container: u64, is_stripe_table_mirrored: bool, container_index_layers: u32) -> StripeTableArmBreakdown {
    if publish.leaf_count == 0 && publish.spine_node_count == 0 {
        return StripeTableArmBreakdown {
            baseline_bytes: 0,
            parity_bytes: 0,
            table_component_bytes: 0,
            table_mirrored_bytes: 0,
            member_record_count: 0,
            containers_touched: 0,
            data_stripe_width: 0,
            spine_stripe_width: 0,
        };
    }
    // 两个格宽桶各算各的 w（D2 已定项 6 逐字：按格宽分桶）。
    let data_stripe_width = stripe_width_for(publish.leaf_count, devices);
    let spine_stripe_width = stripe_width_for(publish.spine_node_count, devices);
    let data_stripe_count = stripe_count_for(publish.leaf_count, data_stripe_width);
    let spine_stripe_count = stripe_count_for(publish.spine_node_count, spine_stripe_width);
    let parity_bytes = data_stripe_count * DATA_UNIT_SIZE_BYTES + spine_stripe_count * NODE_SIZE_BYTES;
    // 成员记录：每个格一条，含 parity 格。
    let member_record_count = publish.leaf_count + data_stripe_count + publish.spine_node_count + spine_stripe_count;
    let container_capacity_records = records_per_container();
    // 从已占 occupied_slots_in_open_container 槽的那个容器开始摆，摆满就开新容器。
    let containers_touched = (occupied_slots_in_open_container + member_record_count).div_ceil(container_capacity_records).max(1);
    let table_component_bytes = containers_touched * DATA_UNIT_SIZE_BYTES + u64::from(container_index_layers) * NODE_SIZE_BYTES;
    let table_mirrored_bytes = if is_stripe_table_mirrored {
        table_component_bytes * 2
    } else {
        table_component_bytes
    };
    StripeTableArmBreakdown {
        baseline_bytes: publish.baseline_bytes(),
        parity_bytes,
        table_component_bytes,
        table_mirrored_bytes,
        member_record_count,
        containers_touched,
        data_stripe_width,
        spine_stripe_width,
    }
}

/// 丙臂：`w` 恒 2 全镜像，一条条带记录都不写。
/// journal 与根槽按口径 2 各写一份（与甲臂同加同减）。
fn publish_on_full_mirror_arm(publish: Publish) -> u64 {
    if publish.leaf_count == 0 && publish.spine_node_count == 0 {
        return 0;
    }
    (publish.leaf_count * DATA_UNIT_SIZE_BYTES + publish.spine_node_count * NODE_SIZE_BYTES) * 2 + JOURNAL_RECORD_BYTES + ROOT_SLOT_BYTES
}

/// 阳性对照用的第二个模型：**记录直接住索引叶**，没有码 3 容器这一层。
/// 只有当容器几何与索引节点几何完全相同（尺寸与头都相同）时，
/// 两个模型的「打包侧那一层」才必须逐字节相等——不等说明两条臂不在同一把尺子上。
fn packed_layer_bytes(unit_size_bytes: u64, header_bytes: u64, member_record_count: u64, occupied_slots_in_open_container: u64) -> u64 {
    if member_record_count == 0 {
        return 0;
    }
    let records_per_unit = (unit_size_bytes - header_bytes) / STRIPE_RECORD_BYTES;
    (occupied_slots_in_open_container + member_record_count).div_ceil(records_per_unit).max(1) * unit_size_bytes
}

// ── 乙臂：真机延迟 ───────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod device {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    pub const ALIGNMENT_BYTES: usize = 4096;

    /// 4096 对齐的缓冲，O_DIRECT 要求。
    pub struct AlignedBuffer {
        pointer: *mut u8,
        length_bytes: usize,
    }
    impl AlignedBuffer {
        pub fn new(length_bytes: usize) -> Self {
            let layout = std::alloc::Layout::from_size_align(length_bytes, ALIGNMENT_BYTES).unwrap();
            let pointer = unsafe { std::alloc::alloc_zeroed(layout) };
            assert!(!pointer.is_null(), "alloc_zeroed 失败");
            AlignedBuffer { pointer, length_bytes }
        }
        pub fn as_slice(&self) -> &[u8] {
            unsafe { std::slice::from_raw_parts(self.pointer, self.length_bytes) }
        }
    }
    impl Drop for AlignedBuffer {
        fn drop(&mut self) {
            let layout = std::alloc::Layout::from_size_align(self.length_bytes, ALIGNMENT_BYTES).unwrap();
            unsafe { std::alloc::dealloc(self.pointer, layout) };
        }
    }

    pub fn open_direct(path: &str) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .custom_flags(libc_open_direct_flag())
            .open(path)
    }

    fn libc_open_direct_flag() -> i32 {
        // O_DIRECT 在 x86_64/aarch64 linux 上是 0o40000。
        0o40000
    }

    pub fn pwrite_all(file: &File, buffer: &[u8], offset_bytes: u64) -> io::Result<()> {
        let mut bytes_written = 0usize;
        while bytes_written < buffer.len() {
            let bytes_written_this_call = unsafe {
                pwrite_raw(
                    file.as_raw_fd(),
                    buffer.as_ptr().add(bytes_written) as *const core::ffi::c_void,
                    buffer.len() - bytes_written,
                    (offset_bytes + bytes_written as u64) as i64,
                )
            };
            if bytes_written_this_call < 0 {
                return Err(io::Error::last_os_error());
            }
            if bytes_written_this_call == 0 {
                return Err(io::Error::other("pwrite 返回 0"));
            }
            bytes_written += bytes_written_this_call as usize;
        }
        Ok(())
    }

    extern "C" {
        #[link_name = "pwrite"]
        fn pwrite_raw(
            fd: i32,
            buf: *const core::ffi::c_void,
            count: usize,
            offset: i64,
        ) -> isize;
        #[link_name = "fdatasync"]
        fn fdatasync_raw(fd: i32) -> i32;
    }

    pub fn fdatasync(file: &File) -> io::Result<()> {
        let return_code = unsafe { fdatasync_raw(file.as_raw_fd()) };
        if return_code != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

fn main() {
    let mut emitter = Emitter::new();
    let mut output_text = String::new();

    // ── 阴性对照（作废条款 1）：N = 0 ⇒ 两条臂恒 0 ───────────────────────
    let empty_publish = Publish::main_workload(0);
    let stripe_table_arm_for_empty_publish = publish_on_stripe_table_arm(empty_publish, 4, 0, true, container_index_height(TOTAL_CONTAINERS));
    let full_mirror_arm_bytes_for_empty_publish = publish_on_full_mirror_arm(empty_publish);
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=negative_control leaves=0 arm_a_with_redundancy={} arm_c={} verdict={}",
        stripe_table_arm_for_empty_publish.with_redundancy(),
        full_mirror_arm_bytes_for_empty_publish,
        if stripe_table_arm_for_empty_publish.with_redundancy() == 0 && full_mirror_arm_bytes_for_empty_publish == 0 {
            "pass"
        } else {
            "VOID"
        }
    )));
    output_text.push('\n');

    let container_index_layers = container_index_height(TOTAL_CONTAINERS);
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=geometry recs_per_container={} cidx_fanout={} cidx_height={} total_containers={}",
        records_per_container(),
        container_index_fanout(),
        container_index_layers,
        TOTAL_CONTAINERS
    )));
    output_text.push('\n');

    // ── 甲臂 / 丙臂：扫 N 与 occupied_slots_in_open_container ────────────────────────────────────────────
    let container_capacity_records = records_per_container();
    for &leaf_count in &[1u64, 8, 12, 64] {
        let publish = Publish::main_workload(leaf_count);
        // occupied_slots_in_open_container 扫全部取值太多行，取三个端点 + 一个会溢出的点。
        for &occupied_slots_in_open_container in &[0u64, container_capacity_records / 2, container_capacity_records - 1] {
            let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, occupied_slots_in_open_container, true, container_index_layers);
            let stripe_table_arm_without_mirror = publish_on_stripe_table_arm(publish, 4, occupied_slots_in_open_container, false, container_index_layers);
            let full_mirror_arm_bytes = publish_on_full_mirror_arm(publish);
            output_text.push_str(&emitter.emit_raw(&format!(
                "name=bytes leaves={leaf_count} f={occupied_slots_in_open_container} w_data={} w_meta={} records={} containers={} \
                 baseline={} parity={} table_component={} table_mirrored={} \
                 arm_a_no_redundancy={} arm_a_with_redundancy={} arm_a_nomirror_with_redundancy={} \
                 arm_c_mirror={} ratio_a_over_c={:.4} wa_no_redundancy={:.4}",
                stripe_table_arm.data_stripe_width,
                stripe_table_arm.spine_stripe_width,
                stripe_table_arm.member_record_count,
                stripe_table_arm.containers_touched,
                stripe_table_arm.baseline_bytes,
                stripe_table_arm.parity_bytes,
                stripe_table_arm.table_component_bytes,
                stripe_table_arm.table_mirrored_bytes,
                stripe_table_arm.no_redundancy(),
                stripe_table_arm.with_redundancy(),
                stripe_table_arm_without_mirror.with_redundancy(),
                full_mirror_arm_bytes,
                stripe_table_arm.with_redundancy() as f64 / full_mirror_arm_bytes as f64,
                stripe_table_arm.no_redundancy() as f64 / stripe_table_arm.baseline_bytes as f64,
            )));
            output_text.push('\n');
        }
    }

    // ── 交叉点：甲臂从第几个叶开始比丙臂省 ────────────────────────────────
    let mut crossover_leaf_count = None;
    for leaf_count in 1u64..=4096 {
        let publish = Publish::main_workload(leaf_count);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, container_index_layers);
        if stripe_table_arm.with_redundancy() < publish_on_full_mirror_arm(publish) {
            crossover_leaf_count = Some(leaf_count);
            break;
        }
    }
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=crossover first_leaves_where_arm_a_cheaper={} main_workload_leaves={}",
        crossover_leaf_count.map(|crossover_leaves| crossover_leaves as i64).unwrap_or(-1),
        MAIN_LEAVES
    )));
    output_text.push('\n');

    // ── 饱和对照（作废条款 2）：记录数远大于一个容器的容量 ────────────────
    let saturating_publish = Publish::main_workload(20_000);
    let saturating_arm = publish_on_stripe_table_arm(saturating_publish, 4, 0, true, container_index_layers);
    let expected_containers = saturating_arm.member_record_count.div_ceil(container_capacity_records);
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=saturation_control records={} containers={} expect_ceil={} verdict={}",
        saturating_arm.member_record_count,
        saturating_arm.containers_touched,
        expected_containers,
        if saturating_arm.containers_touched == expected_containers {
            "pass"
        } else {
            "VOID"
        }
    )));
    output_text.push('\n');

    // ── 阳性对照：同几何时两个模型必须逐字节相等 ──────────────────────────
    let member_record_count = 17u64;
    let same_geometry_packed_bytes = packed_layer_bytes(NODE_SIZE_BYTES, CONTAINER_INDEX_HEADER_BYTES, member_record_count, 0);
    let same_geometry_leaf_bytes = packed_layer_bytes(NODE_SIZE_BYTES, CONTAINER_INDEX_HEADER_BYTES, member_record_count, 0);
    let real_geometry_packed_bytes = packed_layer_bytes(DATA_UNIT_SIZE_BYTES, PACKED_CONTAINER_HEADER_BYTES, member_record_count, 0);
    output_text.push_str(&emitter.emit_raw(&format!(
        "name=positive_control same_geometry_packed={} same_geometry_leaf={} real_geometry_packed={} verdict={}",
        same_geometry_packed_bytes,
        same_geometry_leaf_bytes,
        real_geometry_packed_bytes,
        if same_geometry_packed_bytes == same_geometry_leaf_bytes { "pass" } else { "VOID" }
    )));
    output_text.push('\n');

    // ── 乙臂：真机延迟 ────────────────────────────────────────────────────
    #[cfg(target_os = "linux")]
    {
        let temporary_directory = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
        let path = format!("{temporary_directory}/e107-latency.img");
        match device::open_direct(&path) {
            Ok(latency_file) => {
                let data_buffer = device::AlignedBuffer::new((MAIN_LEAVES * DATA_UNIT_SIZE_BYTES) as usize);
                let parity_buffer = device::AlignedBuffer::new((3 * DATA_UNIT_SIZE_BYTES) as usize);
                let container_buffer = device::AlignedBuffer::new(DATA_UNIT_SIZE_BYTES as usize);
                let rounds = 5;

                // 预热：文件是新建的，第一次写要走首触分配，那一轮的耗时不是被测量。
                // 不预热时实测 one_shot 的五轮离散到 214%，直接触发作废条款 4。
                for _ in 0..8 {
                    device::pwrite_all(&latency_file, data_buffer.as_slice(), 0).unwrap();
                    device::pwrite_all(&latency_file, parity_buffer.as_slice(), 1 << 20).unwrap();
                    device::pwrite_all(&latency_file, container_buffer.as_slice(), 2 << 20).unwrap();
                    device::fdatasync(&latency_file).unwrap();
                }

                // 三种走法 × 两种「写成功」语义。
                for &is_durable_semantics in &[false, true] {
                    let mut one_shot = Vec::new();
                    let mut two_phase = Vec::new();
                    let mut two_phase_sleep = Vec::new();
                    for _ in 0..rounds {
                        // 走法一：一次提交全部，等一次完成。
                        let start_instant = Instant::now();
                        device::pwrite_all(&latency_file, data_buffer.as_slice(), 0).unwrap();
                        device::pwrite_all(&latency_file, parity_buffer.as_slice(), 1 << 20).unwrap();
                        device::pwrite_all(&latency_file, container_buffer.as_slice(), 2 << 20).unwrap();
                        if is_durable_semantics {
                            device::fdatasync(&latency_file).unwrap();
                        }
                        one_shot.push(start_instant.elapsed().as_nanos() as u64);

                        // 走法二：先数据 + parity，等完成，再记录容器，再等完成。
                        let start_instant = Instant::now();
                        device::pwrite_all(&latency_file, data_buffer.as_slice(), 0).unwrap();
                        device::pwrite_all(&latency_file, parity_buffer.as_slice(), 1 << 20).unwrap();
                        if is_durable_semantics {
                            device::fdatasync(&latency_file).unwrap();
                        }
                        device::pwrite_all(&latency_file, container_buffer.as_slice(), 2 << 20).unwrap();
                        if is_durable_semantics {
                            device::fdatasync(&latency_file).unwrap();
                        }
                        two_phase.push(start_instant.elapsed().as_nanos() as u64);

                        // 阳性对照：同一条路径里插一个已知 5 ms。
                        let start_instant = Instant::now();
                        device::pwrite_all(&latency_file, data_buffer.as_slice(), 0).unwrap();
                        device::pwrite_all(&latency_file, parity_buffer.as_slice(), 1 << 20).unwrap();
                        if is_durable_semantics {
                            device::fdatasync(&latency_file).unwrap();
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        device::pwrite_all(&latency_file, container_buffer.as_slice(), 2 << 20).unwrap();
                        if is_durable_semantics {
                            device::fdatasync(&latency_file).unwrap();
                        }
                        two_phase_sleep.push(start_instant.elapsed().as_nanos() as u64);
                    }
                    let median_nanoseconds = |samples: &mut Vec<u64>| {
                        samples.sort_unstable();
                        samples[samples.len() / 2]
                    };
                    let relative_spread = |samples: &[u64]| {
                        let fastest = *samples.iter().min().unwrap() as f64;
                        let slowest = *samples.iter().max().unwrap() as f64;
                        (slowest - fastest) / fastest
                    };
                    let spread_one_shot = relative_spread(&one_shot);
                    let spread_two_phase = relative_spread(&two_phase);
                    let median_one_shot_nanoseconds = median_nanoseconds(&mut one_shot);
                    let median_two_phase_nanoseconds = median_nanoseconds(&mut two_phase);
                    let median_two_phase_with_sleep_nanoseconds = median_nanoseconds(&mut two_phase_sleep);
                    let injected_recovered_nanoseconds = median_two_phase_with_sleep_nanoseconds as i64 - median_two_phase_nanoseconds as i64;
                    let semantics_label = if is_durable_semantics { "durable" } else { "ack" };
                    output_text.push_str(&emitter.emit_raw(&format!(
                        "name=latency semantics={semantics_label} rounds={rounds} one_shot_ns={median_one_shot_nanoseconds} two_phase_ns={median_two_phase_nanoseconds} \
                         two_phase_plus_5ms_ns={median_two_phase_with_sleep_nanoseconds} injected_recovered_ns={injected_recovered_nanoseconds} \
                         spread_one_shot={spread_one_shot:.3} spread_two_phase={spread_two_phase:.3}"
                    )));
                    output_text.push('\n');
                }
                let _ = std::fs::remove_file(&path);
                // 对照：不带 O_DIRECT、不 fdatasync 的那条路必须快一个数量级。
                let plain_path = format!("{temporary_directory}/e107-nosync.img");
                if let Ok(mut plain_file) = std::fs::File::create(&plain_path) {
                    let plain_buffer = vec![0u8; (MAIN_LEAVES * DATA_UNIT_SIZE_BYTES) as usize];
                    let start_instant = Instant::now();
                    for _ in 0..5 {
                        plain_file.write_all(&plain_buffer).unwrap();
                    }
                    let per_round_nanoseconds = start_instant.elapsed().as_nanos() as u64 / 5;
                    output_text.push_str(&emitter.emit_raw(&format!("name=nosync_control per_round_ns={per_round_nanoseconds}")));
                    output_text.push('\n');
                    let _ = std::fs::remove_file(&plain_path);
                }
            }
            Err(open_error) => {
                output_text.push_str(&emitter.emit_raw(&format!(
                    "name=latency status=unavailable reason={open_error} howto=在支持 O_DIRECT 的块设备上跑，或设 TMPDIR 指向 ext4/xfs"
                )));
                output_text.push('\n');
            }
        }
    }

    // 保留一个 Sample 形态的出口，供 vm-bench.sh 的通用抓取路径复用。
    let publish = Publish::main_workload(MAIN_LEAVES);
    let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, container_index_layers);
    output_text.push_str(&emitter.emit(
        "main_workload_arm_a",
        &Sample {
            operation_count: 1,
            bytes_per_operation: stripe_table_arm.with_redundancy(),
            elapsed_nanoseconds: 1,
        },
    ));
    output_text.push('\n');
    output_text.push_str(&emitter.finish());
    println!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 钉绝对值的断言（跑前写死）──────────────────────────────────────────
    // 写成加法不写减法：常量被变异改大时若做减法会编译期溢出，
    // 那条本该变红的断言会被记成「无效变异」（test-discipline 逐字）。

    /// E106 已钉的同一个数，直接抄过来做交叉校验。
    #[test]
    fn records_per_container_matches_e106() {
        assert_eq!(PACKED_CONTAINER_HEADER_BYTES + STRIPE_RECORD_BYTES * 583 + 13, 32768);
        assert_eq!(records_per_container(), 583);
    }

    #[test]
    fn container_index_height_is_pinned() {
        assert_eq!(container_index_fanout(), 191);
        assert_eq!(container_index_height(TOTAL_CONTAINERS), 3);
    }

    #[test]
    fn mirror_doubles_component_bytes_exactly() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, 3);
        assert_eq!(stripe_table_arm.table_component_bytes, 81920);
        assert_eq!(stripe_table_arm.table_mirrored_bytes, 81920 * 2);
        assert_eq!(stripe_table_arm.table_mirrored_bytes, 163840);
    }

    #[test]
    fn baseline_total_bytes_is_pinned() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        assert_eq!(publish.baseline_bytes(), 8 * 32768 + 4 * 16384 + 4096 + 512);
        assert_eq!(publish.baseline_bytes(), 332288);
    }

    #[test]
    fn mirror_arm_bytes_are_pinned() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        assert_eq!(publish_on_full_mirror_arm(publish), (8 * 32768 + 4 * 16384) * 2 + 4096 + 512);
        assert_eq!(publish_on_full_mirror_arm(publish), 659968);
    }

    /// 主负载那一点上甲臂的三个分量都钉死，防「三条臂一起错」。
    #[test]
    fn main_workload_stripe_table_arm_components_are_pinned() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, 3);
        // w = clamp(格数 + 1, 2, 4)：数据桶 8 格 ⇒ 4；元数据桶 4 格 ⇒ 4。
        assert_eq!(stripe_table_arm.data_stripe_width, 4);
        assert_eq!(stripe_table_arm.spine_stripe_width, 4);
        // 数据条带 ceil(8/3) = 3，元数据条带 ceil(4/3) = 2。
        assert_eq!(stripe_table_arm.parity_bytes, 3 * 32768 + 2 * 16384);
        assert_eq!(stripe_table_arm.parity_bytes, 131072);
        // 记录：8 数据格 + 3 parity + 4 元数据格 + 2 parity = 17。
        assert_eq!(stripe_table_arm.member_record_count, 17);
        assert_eq!(stripe_table_arm.containers_touched, 1);
        assert_eq!(stripe_table_arm.with_redundancy(), 332288 + 131072 + 163840);
        assert_eq!(stripe_table_arm.with_redundancy(), 627200);
        assert_eq!(stripe_table_arm.no_redundancy(), 332288 + 163840);
        assert_eq!(stripe_table_arm.no_redundancy(), 496128);
    }

    /// 第三轮那份手算的 1.493× 是「不含 parity」这个口径下的数——把它钉住，
    /// 免得后来的人拿它当「与丙臂同尺」的比值用。
    #[test]
    fn third_round_hand_arithmetic_is_the_no_redundancy_ratio() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, 3);
        let ratio = stripe_table_arm.no_redundancy() as f64 / stripe_table_arm.baseline_bytes as f64;
        assert!((ratio - 1.4930).abs() < 5e-4, "算出来 {ratio}");
        let ratio_without_mirror =
            publish_on_stripe_table_arm(publish, 4, 0, false, 3).no_redundancy() as f64 / stripe_table_arm.baseline_bytes as f64;
        assert!((ratio_without_mirror - 1.2465).abs() < 5e-4, "算出来 {ratio_without_mirror}");
    }

    /// 单叶那一点：条带表的固定开销把甲臂推到丙臂的 1.65 倍。
    #[test]
    fn single_leaf_stripe_table_arm_is_more_expensive_and_pinned() {
        let publish = Publish::main_workload(1);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, 3);
        assert_eq!(stripe_table_arm.data_stripe_width, 2);
        assert_eq!(stripe_table_arm.spine_stripe_width, 4);
        assert_eq!(stripe_table_arm.parity_bytes, 1 * 32768 + 2 * 16384);
        assert_eq!(stripe_table_arm.with_redundancy(), 102912 + 65536 + 163840);
        assert_eq!(stripe_table_arm.with_redundancy(), 332288);
        assert_eq!(publish_on_full_mirror_arm(publish), 201216);
    }

    /// 交叉点：甲臂从第几个叶起比丙臂便宜。**这是这个实验要回答的那个数。**
    ///
    /// 手算（不看模型输出，逐格列出来）：
    /// `甲 − 丙 = parity + 163840 − (数据字节 + 索引字节)`。
    /// 6 叶：parity `2×32768 + 2×16384 = 98304`，数据+索引 `196608 + 65536 = 262144`，
    /// 而 `98304 + 163840 = 262144` ⇒ **差恰为 0**。
    /// 7 叶：数据条带从 2 涨到 3 ⇒ 数据与 parity 同时 +32768 ⇒ **差仍恰为 0**。
    /// 8 叶：数据 +32768 而 `ceil(8/3)` 仍是 3 ⇒ parity 不涨 ⇒ **差 −32768**。
    /// ⇒ 严格「更便宜」第一次出现在 **8 叶**，正好是 D25 主负载那一点。
    #[test]
    fn crossover_leaf_count_is_pinned() {
        let container_index_layers = container_index_height(TOTAL_CONTAINERS);
        let mut first_cheaper_leaf_count = None;
        for leaf_count in 1u64..=4096 {
            let publish = Publish::main_workload(leaf_count);
            if publish_on_stripe_table_arm(publish, 4, 0, true, container_index_layers).with_redundancy() < publish_on_full_mirror_arm(publish) {
                first_cheaper_leaf_count = Some(leaf_count);
                break;
            }
        }
        assert_eq!(first_cheaper_leaf_count, Some(8));
        assert_eq!(first_cheaper_leaf_count, Some(MAIN_LEAVES));
    }

    /// 6 叶与 7 叶两条臂**逐字节相等**——比交叉点本身更硬的一条：
    /// 它把「甲臂在主负载附近没有优势」钉成等式，而不是一个不等号。
    #[test]
    fn six_and_seven_leaves_are_byte_identical() {
        for leaf_count in [6u64, 7] {
            let publish = Publish::main_workload(leaf_count);
            let stripe_table_arm_bytes = publish_on_stripe_table_arm(publish, 4, 0, true, 3).with_redundancy();
            let full_mirror_arm_bytes = publish_on_full_mirror_arm(publish);
            assert_eq!(stripe_table_arm_bytes, full_mirror_arm_bytes, "{leaf_count} 叶：甲 {stripe_table_arm_bytes} 丙 {full_mirror_arm_bytes}");
        }
        assert_eq!(publish_on_full_mirror_arm(Publish::main_workload(6)), 528896);
        assert_eq!(publish_on_full_mirror_arm(Publish::main_workload(7)), 594432);
    }

    /// 主负载那一点上甲臂只便宜 32768 字节。**便宜多少要钉住**——
    /// 只钉「甲更便宜」会让一个把差额算大十倍的变异照样判绿。
    #[test]
    fn main_workload_margin_is_one_data_unit() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        let stripe_table_arm_bytes = publish_on_stripe_table_arm(publish, 4, 0, true, 3).with_redundancy();
        assert_eq!(stripe_table_arm_bytes + DATA_UNIT_SIZE_BYTES, publish_on_full_mirror_arm(publish));
        assert_eq!(stripe_table_arm_bytes + 32768, 659968);
    }

    /// 阴性对照（作废条款 1）。
    #[test]
    fn empty_publish_is_zero_on_both_arms() {
        let publish = Publish::main_workload(0);
        assert_eq!(publish_on_stripe_table_arm(publish, 4, 0, true, 3).with_redundancy(), 0);
        assert_eq!(publish_on_full_mirror_arm(publish), 0);
    }

    /// 饱和对照（作废条款 2）：容器数必须恰等于 ⌈记录数 / 583⌉。
    #[test]
    fn saturation_container_count_is_exact() {
        let publish = Publish::main_workload(20_000);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 0, true, 3);
        assert_eq!(stripe_table_arm.containers_touched, stripe_table_arm.member_record_count.div_ceil(583));
        assert!(stripe_table_arm.member_record_count > 583 * 10, "记录数 {} 不够饱和", stripe_table_arm.member_record_count);
    }

    /// 阳性对照：几何相同时两个模型必须逐字节相等；几何不同时必须不等
    /// （后半句证明这条对照分得出差别，不是恒真）。
    #[test]
    fn positive_control_same_geometry_is_equal_and_discriminating() {
        assert_eq!(
            packed_layer_bytes(NODE_SIZE_BYTES, CONTAINER_INDEX_HEADER_BYTES, 17, 0),
            packed_layer_bytes(NODE_SIZE_BYTES, CONTAINER_INDEX_HEADER_BYTES, 17, 0)
        );
        assert_ne!(
            packed_layer_bytes(NODE_SIZE_BYTES, CONTAINER_INDEX_HEADER_BYTES, 17, 0),
            packed_layer_bytes(DATA_UNIT_SIZE_BYTES, PACKED_CONTAINER_HEADER_BYTES, 17, 0)
        );
    }

    /// `w` 的判据与两条边界（D2 已定项 6）。
    #[test]
    fn width_rule_matches_stripe_strategy_decision_item_6() {
        assert_eq!(stripe_width_for(1, 8), 2);
        assert_eq!(stripe_width_for(2, 8), 3);
        assert_eq!(stripe_width_for(3, 8), 4);
        assert_eq!(stripe_width_for(99, 8), 4); // 上界 4 与设备数无关
        assert_eq!(stripe_width_for(99, 2), 2); // 再夹当时可写的设备数
        assert_eq!(stripe_width_for(0, 8), 0);
    }

    /// 条带数：数据列是 w−1，不是 w。
    #[test]
    fn stripe_count_uses_data_columns_not_width() {
        assert_eq!(stripe_count_for(8, 4), 3);
        assert_eq!(stripe_count_for(8, 2), 8);
        assert_eq!(stripe_count_for(0, 4), 0);
    }

    /// 容器溢出：occupied_slots_in_open_container 接近满时同一次发布要碰两个容器。
    #[test]
    fn container_overflow_touches_two() {
        let publish = Publish::main_workload(MAIN_LEAVES);
        let stripe_table_arm = publish_on_stripe_table_arm(publish, 4, 583 - 1, true, 3);
        assert_eq!(stripe_table_arm.containers_touched, 2);
        assert_eq!(stripe_table_arm.table_component_bytes, 2 * 32768 + 3 * 16384);
    }
}
