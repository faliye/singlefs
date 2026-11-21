//! 根落盘之前崩溃的两种造法、Q7c 的判别力自证与分配代的影子账。

use super::*;

/// 发完再改回的两份对照镜像（第 4 次重跑登记 W6：只留作对照）。
struct RestoredAfterPublishing {
    /// 只把这次的根槽（512 字节，单设备不镜像）改回发布之前的旧内容：装置原有的造法。它自称与「在根槽那一次写之前截断」
    /// 逐字节相同，前提是根槽是那次发布最后一笔写；第 4 次重跑第一段量到这个前提不成立——实现在根槽 FUA 之后还轮换每块盘的
    /// 系统配置槽（`transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`，D16（发布语义） 已定项 7 的持久顺序）。
    root_slot_only: MemoryPool,
    /// 根槽与每块盘的系统配置两槽都改回发布之前（登记第十二节修订 2）：按实现的持久顺序根槽之后只有系统配置槽轮换，
    /// W6 录写切段镜像与它逐字节比（停机 S1(i)）——比的是「根槽之前的写，录写切段与真发一字不差，根槽之后除系统配置轮换外没有别的写」。
    root_slot_and_system_configuration: MemoryPool,
}

/// 系统配置每块盘几槽（`.claude/kb/decisions/22-单元原子性怎么合成.md`「系统配置每盘 2 槽按世代号 mod 2 轮换」）、一槽几字节
/// （同一文件「`| **系统配置槽宽** | **格式常量 4096**`」那一行）：本地常量，[`emit_static_anchors`] 与 `singlefs_format` 同名常量回比。
pub(super) const E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: u64 = 2;
pub(super) const E156_SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;

/// 真发一次，再按 [`RestoredAfterPublishing`] 的两种办法改回。
#[allow(
    clippy::too_many_arguments,
    reason = "c2 崩溃要凑齐设备、分配器、上一版、发布种类、内容、实例、几何七类各自独立的参数"
)]
fn crash_before_root_persists(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, SparseBlockDevice)>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    kind: WorkloadPublishKind,
    content_salt: u64,
    instance: InstanceGeneration,
    slots_per_region: RootRingSlotsPerRegion,
) -> Result<RestoredAfterPublishing, PublishError> {
    let before = memory_pool_snapshot(devices);
    let published = publish_one_workload(
        parameters,
        devices.as_mut_slice(),
        allocator,
        previous,
        instance,
        kind,
        content_salt,
    )?;
    let (root_device, root_offset) = root_slot_location(
        published.root.checkpoint_txg,
        &parameters.region_devices,
        parameters.geometry.fixed_structure_slot_spacing,
        slots_per_region,
    );
    let old_root_bytes = mp_read(&before, root_device, root_offset, 512);
    let mut root_slot_only = memory_pool_snapshot(devices);
    mp_write(
        &mut root_slot_only,
        root_device,
        root_offset,
        &old_root_bytes,
    );
    let mut root_slot_and_system_configuration = root_slot_only.clone();
    let slot_bytes = usize::try_from(E156_SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    for device in before
        .devices
        .keys()
        .map(|identity| identity.0)
        .collect::<Vec<u32>>()
    {
        for slot_index in 0..E156_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            let offset = slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing);
            let old_bytes = mp_read(&before, device, offset, slot_bytes);
            mp_write(
                &mut root_slot_and_system_configuration,
                device,
                offset,
                &old_bytes,
            );
        }
    }
    Ok(RestoredAfterPublishing {
        root_slot_only,
        root_slot_and_system_configuration,
    })
}

/// W6 切段点：根槽那一次写之前（登记写死），或 PC-c2 把根槽那一次写也留下。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum CrashCutPoint {
    BeforeTheRootSlotWrite,
    AfterTheRootSlotWritePositiveControl,
}

/// 一次 c2 崩溃的两份镜像与这次发布的输出。
pub(super) struct CrashedPublish {
    pub(super) cut_image: MemoryPool,
    /// W6 的录写切段镜像与「发完把根槽与每块盘的系统配置两槽改回」的镜像读回来逐字节相同（停机 S1(i) 的对拍项，登记第十二节修订 2；
    /// PC-c2 那一格按构造不同）。
    pub(super) cut_matches_the_restored_image: bool,
    /// 录写切段镜像与装置原有的「只改回根槽」镜像读回来不同的 (盘, 扇区起点字节偏移)：S1(i) 两边查时量出来的那一格，照报。
    pub(super) sectors_differing_from_the_root_slot_only_image: Vec<(u32, u64)>,
    /// 写日志里根槽那一次写之后还有的步（`StepKind::name`）：「发完改回根槽」那一造法的前提是它为空。
    pub(super) steps_after_the_root_slot_write: Vec<&'static str>,
    pub(super) root_slot_writes_in_the_publish: usize,
    pub(super) crashed_output: TransactionOutput,
    pub(super) allocator_after_the_crashed_publish: PoolAllocator,
}

/// W6：录下那次发布的设备写，切在根槽那一次写之前（`crash.rs` 的写日志，`FixedGeometry::classify` 认根槽写），
/// 切出来的镜像就是「记录已持久、根槽没持久」的 c2 崩溃镜像。同一次发布另按 [`crash_before_root_persists`] 造一份对照镜像。
#[allow(
    clippy::too_many_arguments,
    reason = "c2 崩溃要凑齐设备、分配器、上一版、发布种类、内容、实例、切段点七类各自独立的参数"
)]
pub(super) fn crash_publish_before_its_root_persists(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    allocator: &PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
    kind: WorkloadPublishKind,
    content_salt: u64,
    cut_point: CrashCutPoint,
) -> Result<CrashedPublish, PublishError> {
    let before = memory_pool_snapshot(devices);
    let stream = SharedStream::retaining_contents();
    let mut recording_devices = recording_copies_of(&before, &stream);
    let mut recording_allocator = allocator.clone();
    let crashed_output = publish_one_workload(
        parameters,
        recording_devices.as_mut_slice(),
        &mut recording_allocator,
        previous,
        instance,
        kind,
        content_salt,
    )?;
    let operations = stream.retained_operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let root_slot_write_indexes: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| {
            matches!(
                geometry.classify(&retained.operation),
                StepKind::RootRecordFua
            )
        })
        .map(|(index, _)| index)
        .collect();
    let root_slot_writes_in_the_publish = root_slot_write_indexes.len();
    let root_slot_write_index = *root_slot_write_indexes
        .first()
        .expect("一次发布恰有一次根槽写（单设备不镜像）");
    let kept_operations = match cut_point {
        CrashCutPoint::BeforeTheRootSlotWrite => root_slot_write_index,
        CrashCutPoint::AfterTheRootSlotWritePositiveControl => root_slot_write_index + 1,
    };
    let mut restore_devices = devices_from_memory_pool(&before);
    let mut cut_image = before;
    cut_image.apply(&operations[..kept_operations]);

    let mut restore_allocator = allocator.clone();
    let restored_image = crash_before_root_persists(
        parameters,
        &mut restore_devices,
        &mut restore_allocator,
        previous,
        kind,
        content_salt,
        instance,
        parameters.geometry.root_ring_slots_per_region,
    )?;
    let steps_after_the_root_slot_write: Vec<&'static str> = operations
        [root_slot_write_index + 1..]
        .iter()
        .map(|retained| geometry.classify(&retained.operation).name())
        .collect();
    Ok(CrashedPublish {
        cut_matches_the_restored_image: memory_pools_read_back_identically(
            &cut_image,
            &restored_image.root_slot_and_system_configuration,
        ),
        sectors_differing_from_the_root_slot_only_image: sectors_read_back_differently(
            &cut_image,
            &restored_image.root_slot_only,
        ),
        steps_after_the_root_slot_write,
        cut_image,
        root_slot_writes_in_the_publish,
        crashed_output,
        allocator_after_the_crashed_publish: recording_allocator,
    })
}

/// Q7c①②：判别力自证。②在 `deferred == 0` 的基底上（β_syn）无从执行（Bd(−1) 会把 item5 减成负数），记 `not_applicable=true`。
pub(super) fn q7c_self_test(
    emitter: &mut Emitter,
    basis: &str,
    base_pool: &MemoryPool,
    accounting_slot: u64,
    referenced: u64,
) -> (bool, bool) {
    let (allocated, _free, deferred) = mirror_accounting_row_slots(base_pool, accounting_slot);
    // ① 不减第 5 项：Bd(+1)。
    let corrupted_deferred_plus1 = deferred + 1;
    let real_check_red_plus1 = allocated != corrupted_deferred_plus1 + referenced;
    let without_subtracting_defer_is_red = allocated != referenced;
    let flip1 = real_check_red_plus1 && !without_subtracting_defer_is_red;
    emitter.emit(&format!(
        "name=q7c1_not_subtracting_defer basis={basis} k=1 real_check_is_red={real_check_red_plus1} without_subtracting_defer_is_red={without_subtracting_defer_is_red} flips_red_to_green={flip1}"
    ));
    // ② `==` → `>=`：Bd(−1)。
    let flip2 = if deferred == 0 {
        emitter.emit(&format!(
            "name=q7c2_threshold_at_least basis={basis} k=-1 not_applicable=true reason=deferred_is_zero_cannot_subtract_one"
        ));
        false
    } else {
        let corrupted_deferred_minus1 = deferred - 1;
        let real_check_red_minus1 = allocated != corrupted_deferred_minus1 + referenced;
        let at_least_holds = allocated >= corrupted_deferred_minus1 + referenced;
        let flip = real_check_red_minus1 && at_least_holds;
        emitter.emit(&format!(
            "name=q7c2_threshold_at_least basis={basis} k=-1 real_check_is_red={real_check_red_minus1} at_least_holds={at_least_holds} flips_red_to_green={flip}"
        ));
        flip
    };
    (flip1, flip2)
}

/// D0 上「仍分配」记录的 (槽 → 分配代)：只给岔路 1 的分配代账本用（G12 需要一个真实记录里没有的分配代字段，
/// 登记「三」N2；装置自己维护一份影子表，见登记 M6）。
pub(super) fn snapshot_allocated_generations(allocator: &PoolAllocator) -> HashMap<u64, u64> {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && !record.is_released)
        .map(|record| (record.slot.0, record.generation.0))
        .collect()
}

/// 把这次发布新产生的释放记进分配代账本：`before` 是这次发布之前 D0 全部「仍分配」记录的 (槽 → 分配代)，
/// `allocator` 是这次发布之后的状态。真实记录的 `generation` 字段在释放那一刻被改写成释放代（登记「三」I1），
/// 分配代因此只能从 `before` 里找；账本里查不到的按 0（genesis）记，这是保守默认，不是量出来的数（产物逐格报查不到的条数）。
pub(super) fn record_release_generations(
    ledger: &mut HashMap<u64, u64>,
    before: &HashMap<u64, u64>,
    allocator: &PoolAllocator,
) {
    for record in allocator
        .records()
        .iter()
        .filter(|record| record.device == DeviceIdentity(0) && record.is_released)
    {
        if let Some(&allocation_generation) = before.get(&record.slot.0) {
            ledger.insert(record.slot.0, allocation_generation);
        }
    }
}

#[cfg(test)]
mod tests;
