//! 根环上有洞之后的回收门槛（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：2026-10-02 起取一圈窗口，
//! 用户 2026-10-04 定 RGall 退回整个环（`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）。文件名留着窗口那一轮的叫法，
//! 盘面就是当初定窗口用的那一形。
//!
//! 盘面（与崩溃注入种子 0 第 135 段那一形同形，`/tmp/claude-1000/inject6-findings/report.md` 第三节）：实例 1 在 24 槽的根环上发到 txg 30，
//! txg 31 那次发布崩在「记录已持久、根槽没持久」（根槽与之后的系统配置轮换都没落：那一槽里留着上一圈的 txg 7 的根）。
//! 可写挂载（实例 2）恢复由记录施加出 txg 31 那一版，写行 txg 32、暖机 txg 33、34 盖掉 txg 8、9、10 的槽；之后覆盖写 txg 35、36、37。
//! txg 7 的根一直留在 txg 31 那一槽里，是环里最旧的有效根，而 txg 8–11 已经离开根环。
//!
//! - 分配器（进程内每记一条根）：门槛卡在 txg 7，txg 7 那一版引用的单元一个都不回收。照一圈窗口取时 txg 35 起 txg 7 在窗口外，
//!   门槛跟着窗口里最旧的有效根往前走，txg 7 引用、已释放的单元当场回收。
//! - 重新挂载（重建分配器）与抬 F（抬 F 那一遍的回收）同一个门槛：txg 7 的单元仍不回收。
//! - 池级 checker：txg 7 是候选、照走；只被 txg 8–11 那几版引用的单元由「由记录施加出来、根槽从没落盘的那一版」那一路走到
//!   （它们的记录还在 journal 环里），记账与遍历对得上。
//! - 洞里 txg 7 的根所在那一槽再挂那一刻读不出（txg 7 比 N − 3S 还旧）：读坏的槽在回收门槛里钉在它可能住过的最旧 txg 7
//!   （用户 2026-10-04 定，条文还没写回，定案在 `records/2026-09-30-里程碑三开工第一轮.md`），挂载时、进程内与抬 F 那一遍都不越过它。
mod common;

use common::{
    build_pool, crash_state_devices, memory_pool_of_sparse_devices,
    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
    system_configuration_slots_of_every_device, unreadable_root_slot_of, with_unreadable_ranges,
    write_back_system_configuration_slots, DeviceWithUnreadableRanges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{PoolAllocator, TxgsARingSlotBadAtMountCanHold};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::mount::{mount_writable, raise_rollback_floor, ShadowLedger};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_system_configuration, readable_roots_with_ring_slots,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
/// 洞那一槽里留着的上一圈的根。
const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
/// 挂载之后覆盖写到的那一版（txg 35、36、37）。
const LAST_TXG_OF_INSTANCE_TWO: u64 = 37;
/// 一次覆盖写的内容长度：一个数据单元装得下。
const OVERWRITE_BYTES: usize = 3000;
/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
const INVARIANTS_THE_RING_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];

type RecordedDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
}

/// 挂载之后覆盖写到 txg 37 那一刻：实例 2 的盘、分配器、现行那一版。
struct PoolAfterTheHole {
    devices: RecordedDevices,
    allocator: PoolAllocator,
    current: TransactionOutput,
}

/// 造盘面：实例 1 发到 txg 30、txg 31 崩在根槽之前，可写挂载（实例 2）之后覆盖写到 txg `last`；每次覆盖写之后调 `after_each_overwrite`。
fn pool_after_the_hole_overwritten_through(
    last: u64,
    after_each_overwrite: &mut dyn FnMut(u64, &RecordedDevices, &PoolAllocator),
) -> PoolAfterTheHole {
    let mut pool = build_pool("one-revolution-window-after-a-hole");
    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
    assert_eq!(
        3 * slots_per_region.count(),
        TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST - TXG_OF_THE_STALE_ROOT_IN_THE_HOLE,
        "根环 24 槽：txg 31 与 txg 7 同槽"
    );
    // 迭代上界：txg 4 到 30，27 次覆盖写。
    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
        let output = overwrite_in_process_as_instance_one_and_advance_the_pool(
            &mut pool,
            &content_of_txg(checkpoint_txg),
        );
        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
    }
    // txg 31 那次发布之前：它要写的那个根槽里是 txg 7 的根，每块盘两槽系统配置是 txg 30 那次轮换之后的样子。
    let target = target_for_publish(
        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
        slots_per_region,
    );
    let device_of_the_slot =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset_of_the_slot =
        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
    let mut stale_root_slot =
        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")];
    {
        let devices = pool.devices.as_ref().expect("镜像开着");
        let (_, device) = devices
            .iter()
            .find(|(identity, _)| *identity == device_of_the_slot)
            .expect("池里有这块盘");
        device
            .read_at(offset_of_the_slot, &mut stale_root_slot)
            .expect("读那个根槽");
    }
    let system_configuration_before = system_configuration_slots_of_every_device(&pool);
    overwrite_in_process_as_instance_one_and_advance_the_pool(
        &mut pool,
        &content_of_txg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
    );
    // 崩溃状态：txg 31 的单元与记录都落了，根槽与之后的系统配置轮换没落（D16（发布语义） 已定项 7 的持久顺序）。
    let mut crash_image = pool.memory_pool();
    crash_image
        .devices
        .get_mut(&device_of_the_slot)
        .expect("有这块盘")
        .write(offset_of_the_slot, &stale_root_slot);
    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);

    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&crash_image, &[], &[], &stream);
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.instance,
            mounted.output.effective_root.checkpoint_txg,
            mounted.current.root().checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(31), CheckpointTxg(34)),
        "恢复由记录施加出 txg 31；写行 txg 32、暖机 txg 33、34"
    );
    let mut allocator = mounted.allocator;
    let PoolVersion::WithFile(mut current) = mounted.current else {
        panic!("txg 31 那一版带文件");
    };
    let publish_parameters = parameters();
    // 迭代上界：覆盖写到 txg `last`；跨轮携带的是现行那一版。
    while current.root.checkpoint_txg.0 < last {
        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
        current = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut allocator,
                &current,
                FirstFile {
                    content: &content_of_txg(checkpoint_txg),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
                },
                InstanceGeneration(2),
            )
            .expect("覆盖写")
        };
        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
        after_each_overwrite(checkpoint_txg, &devices, &allocator);
    }
    PoolAfterTheHole {
        devices,
        allocator,
        current,
    }
}

fn violations_of(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter(|(invariant, _)| INVARIANTS_THE_RING_KEEPS_GREEN.contains(invariant))
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 洞那一槽里留着的 txg 7 的根（读得出的那一条）。
fn stale_root_in_the_hole(image: &MemoryPool) -> RootRecord {
    let system_configuration = choose_system_configuration(image).expect("系统配置");
    let immutable = &system_configuration.immutable;
    readable_roots_with_ring_slots(
        image,
        &immutable.region_devices,
        &immutable.sizes,
        &immutable.filesystem_identifier,
    )
    .into_iter()
    .map(|(_, root)| root)
    .find(|root| {
        (root.instance, root.checkpoint_txg)
            == (
                InstanceGeneration(1),
                CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
            )
    })
    .expect("txg 7 的根还在环里、读得出")
}

/// txg 7 那一版的账里仍分配、而分配器上空着（可再分配）的槽：(盘, 槽)。
fn slots_of_the_stale_root_left_free(
    image: &MemoryPool,
    allocator: &PoolAllocator,
) -> Vec<(u32, u64)> {
    let stale_root = stale_root_in_the_hole(image);
    let records =
        allocation_records_under_root(image, &stale_root).expect("txg 7 那一版的账读得出");
    let mut left_free = Vec::new();
    for record in records.iter().filter(|record| !record.is_released) {
        let device_map = allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == record.device)
            .expect("账里的盘在分配器里");
        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
            if device_map.is_free(SlotNumber(slot)) {
                left_free.push((record.device.0, slot));
            }
        }
    }
    left_free
}

/// 场景：挂载之后覆盖写 txg 35、36、37（这个进程每记一条根就按环里最旧有效根回收）。
/// 预期：每次覆盖写之后 txg 7 那一版引用的单元在分配器上一个都不空闲，盘上那几条不变量不红。
#[test]
fn reclaiming_after_a_hole_in_the_root_ring_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated(
) {
    pool_after_the_hole_overwritten_through(
        LAST_TXG_OF_INSTANCE_TWO,
        &mut |checkpoint_txg, devices, allocator| {
            let image = memory_pool_of_sparse_devices(devices);
            assert_eq!(
                slots_of_the_stale_root_left_free(&image, allocator),
                Vec::<(u32, u64)>::new(),
                "txg {checkpoint_txg} 之后：txg 7 是环里最旧的有效根，它引用的单元不回收"
            );
            assert_eq!(
                violations_of(&image),
                Vec::<String>::new(),
                "txg {checkpoint_txg} 之后"
            );
        },
    );
}

/// 场景：覆盖写到 txg 37 之后进程退出，再可写挂载（重建分配器：挂载时的门槛锚在环里读得出的最新有效根 37，txg 7 落在它的一圈窗口外）。
/// 预期：重建出来的分配器上 txg 7 那一版引用的单元一个都不空闲，挂载之后盘上那几条不变量不红。
#[test]
fn a_remount_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated() {
    let pool = pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
    let image = memory_pool_of_sparse_devices(&pool.devices);
    let stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&image, &[], &[], &stream);
    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
    let after = memory_pool_of_sparse_devices(&devices);
    assert_eq!(
        slots_of_the_stale_root_left_free(&after, &remounted.allocator),
        Vec::<(u32, u64)>::new(),
        "再挂之后：txg 7 仍是环里最旧的有效根，重建分配器不回收它引用的单元"
    );
    assert_eq!(violations_of(&after), Vec::<String>::new(), "再挂之后");
}

/// 场景：覆盖写到 txg 37 之后在同一个进程里把 F 抬到 7（不高于上限）。
/// 预期：抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根 7) = 7，txg 7 那一版引用的单元一个都不空闲，抬完之后盘上那几条不变量不红。
/// 照一圈窗口取时那一遍的门槛是窗口里最旧的有效根 14，txg 7 引用的单元扣到生效之后放开。
#[test]
fn raising_the_floor_after_the_hole_keeps_every_unit_of_the_stale_root_left_in_the_hole_allocated()
{
    let mut pool =
        pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
    raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.current,
        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        ShadowLedger::On,
    )
    .expect("抬 F 到 7");
    let image = memory_pool_of_sparse_devices(&pool.devices);
    assert_eq!(
        slots_of_the_stale_root_left_free(&image, &pool.allocator),
        Vec::<(u32, u64)>::new(),
        "抬 F 到 7 之后：txg 7 引用的单元不回收"
    );
    assert_eq!(violations_of(&image), Vec::<String>::new(), "抬 F 之后");
}

/// 覆盖写到 txg 37 之后进程退出，再可写挂载（实例 3）：洞里 txg 7 的根所在的那一槽（txg 31 的槽）从这一次挂载起每次读都坏，
/// 按 `read_back` 报设备错或报成功、字节全 0。
/// 交回盘（带读故障）、它的读故障、起点盘面、录下的流与挂载交回的分配器、现行那一版。
struct RemountReadingTheSlotOfTheStaleRootBad {
    devices: Vec<(
        DeviceIdentity,
        DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
    )>,
    unreadable_slot: SharedUnreadableRanges,
    image_before_the_remount: MemoryPool,
    stream: SharedStream,
    allocator: PoolAllocator,
    current: TransactionOutput,
}

impl RemountReadingTheSlotOfTheStaleRootBad {
    fn image_now(&self) -> MemoryPool {
        let mut image = self.image_before_the_remount.clone();
        image.apply(&self.stream.retained_operations());
        image
    }
}

fn remount_reading_the_slot_of_the_stale_root_bad(
    read_back: UnreadableRangeReadBack,
) -> RemountReadingTheSlotOfTheStaleRootBad {
    let pool = pool_after_the_hole_overwritten_through(LAST_TXG_OF_INSTANCE_TWO, &mut |_, _, _| {});
    let image_before_the_remount = memory_pool_of_sparse_devices(&pool.devices);
    let stream = SharedStream::retaining_contents();
    let unreadable_slot = SharedUnreadableRanges::new(
        vec![unreadable_root_slot_of(
            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
            FailingReadsOfARange::Every,
        )],
        read_back,
    );
    let mut devices = with_unreadable_ranges(
        crash_state_devices(&image_before_the_remount, &[], &[], &stream),
        &unreadable_slot,
    );
    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载");
    assert!(
        unreadable_slot.failed_reads() > 0,
        "再挂那一次读洞那一槽读错了"
    );
    let PoolVersion::WithFile(current) = remounted.current else {
        panic!("txg 37 那一版带文件");
    };
    RemountReadingTheSlotOfTheStaleRootBad {
        devices,
        unreadable_slot,
        image_before_the_remount,
        stream,
        allocator: remounted.allocator,
        current,
    }
}

/// 场景：覆盖写到 txg 37 之后再可写挂载，洞里 txg 7 的根所在那一槽再挂那一刻读不出（之后读得出），再覆盖写两次。
/// txg 7 比 N − 3S = 37 − 24 = 13 还旧：照一圈窗口给读坏的槽划 txg 区间时那一槽只能住 txg 31，门槛取环里读得出的最旧有效根 14，
/// txg 7 引用、释放代不高于 14 的单元当场回收（实现员报告第 10 批 core 簇第八节第 2 条）。
/// 预期（用户 2026-10-04 定：读坏的槽钉在它可能住过的最旧 txg）：根环表把那一槽钉在 7（能住的最大 txg 31）；再挂之后与每一次覆盖写之后
/// txg 7 那一版引用的单元一个都不空闲，盘上那几条不变量不红；挂载之后第一次覆盖写重读那一槽、读出 txg 7 的根。
#[test]
fn a_remount_reading_the_slot_of_the_stale_root_left_in_the_hole_bad_keeps_every_unit_of_it_allocated(
) {
    let mut remounted =
        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::DeviceError);
    let slot_of_the_stale_root = target_for_publish(
        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        parameters().geometry.root_ring_slots_per_region,
    );
    assert_eq!(
        remounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表")
            .ring_slots_bad_at_mount()
            .get(&slot_of_the_stale_root),
        Some(&TxgsARingSlotBadAtMountCanHold::new(
            CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
            CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST)
        )),
        "根环表把那一槽钉在可能住过的最旧 txg 7，能住的最大 txg 31"
    );
    remounted.unreadable_slot.lift();
    let after_the_remount = remounted.image_now();
    assert_eq!(
        (
            slots_of_the_stale_root_left_free(&after_the_remount, &remounted.allocator),
            violations_of(&after_the_remount)
        ),
        (Vec::new(), Vec::<String>::new()),
        "再挂之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
    );
    let publish_parameters = parameters();
    let last_txg = remounted.current.root.checkpoint_txg.0 + 2;
    // 迭代上界：两次覆盖写；跨轮携带的是现行那一版。
    while remounted.current.root.checkpoint_txg.0 < last_txg {
        let checkpoint_txg = remounted.current.root.checkpoint_txg.0 + 1;
        remounted.current = {
            let mut writer = PoolWriter::new(&publish_parameters, remounted.devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut remounted.allocator,
                &remounted.current,
                FirstFile {
                    content: &content_of_txg(checkpoint_txg),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
                },
                InstanceGeneration(3),
            )
            .expect("覆盖写")
        };
        let occupancy = remounted
            .allocator
            .root_ring_occupancy()
            .expect("可写挂载装了根环表");
        assert_eq!(
            (
                occupancy
                    .ring_slots_bad_at_mount()
                    .get(&slot_of_the_stale_root)
                    .copied(),
                occupancy
                    .ring_slots_known_to_hold_a_root()
                    .contains(&slot_of_the_stale_root)
            ),
            (None, true),
            "txg {checkpoint_txg}：重读读出 txg 7 的根，那一槽算这个进程知道住着根的槽"
        );
        let image = remounted.image_now();
        assert_eq!(
            (
                slots_of_the_stale_root_left_free(&image, &remounted.allocator),
                violations_of(&image)
            ),
            (Vec::new(), Vec::<String>::new()),
            "txg {checkpoint_txg} 之后"
        );
    }
}

/// 场景：同上再挂，洞里 txg 7 的根所在那一槽一直读不出，在这一次挂载里把 F 抬到 7（不高于上限）。
/// 预期：抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根)，那一槽重读仍读不出、钉在 7，门槛是 7：txg 7 那一版引用的单元一个都不空闲，
/// 抬完之后盘上那几条不变量不红。照一圈窗口划区间时那一槽按 31 并、门槛是读得出的最旧有效根 14，txg 7 引用的单元扣到生效之后放开。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_stays_unreadable_keeps_every_unit_of_it_allocated(
) {
    let mut remounted =
        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::DeviceError);
    raise_rollback_floor(
        &parameters(),
        &mut remounted.devices,
        &mut remounted.allocator,
        &mut remounted.current,
        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        ShadowLedger::On,
    )
    .expect("抬 F 到 7");
    let image = remounted.image_now();
    assert_eq!(
        (
            slots_of_the_stale_root_left_free(&image, &remounted.allocator),
            violations_of(&image)
        ),
        (Vec::new(), Vec::<String>::new()),
        "抬 F 到 7 之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
    );
}

/// 场景：同上再挂，洞里 txg 7 的根所在那一槽从再挂起每次读都报成功、字节全 0（盘上那一槽里是 txg 7 的根），在这一次挂载里把 F 抬到 7。
/// 预期：那一槽在根环表里记成挂载时读回全零、钉在它可能住过的最旧 txg 7（用户 2026-10-05 定乙：回收门槛里照「自证不过」那一类钉住），
/// 不进准入抬 F 的上限要并的槽；抬 F 那一遍的回收门槛 = max(7, 环里最旧有效根)，重读读回全零照旧钉在 7，门槛是 7：txg 7 那一版引用的单元
/// 一个都不空闲，抬完之后盘上那几条不变量不红。抬 F 那一遍不并读回全零的槽时门槛是读得出的最旧有效根 14，txg 7 引用的单元放开。
#[test]
fn raising_the_floor_while_the_slot_of_the_stale_root_left_in_the_hole_reads_back_all_zero_keeps_every_unit_of_it_allocated(
) {
    let mut remounted =
        remount_reading_the_slot_of_the_stale_root_bad(UnreadableRangeReadBack::Zeros);
    let slot_of_the_stale_root = target_for_publish(
        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        parameters().geometry.root_ring_slots_per_region,
    );
    let occupancy = remounted
        .allocator
        .root_ring_occupancy()
        .expect("可写挂载装了根环表");
    assert_eq!(
        (
            occupancy
                .ring_slots_reading_back_all_zero_at_mount_pinned_at_the_oldest_txg_they_can_have_held()
                .get(&slot_of_the_stale_root)
                .copied(),
            occupancy
                .ring_slots_bad_at_mount()
                .contains_key(&slot_of_the_stale_root)
        ),
        (Some(CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE)), false),
        "根环表把那一槽记成挂载时读回全零、钉在 7，不进上限要并的槽"
    );
    let after_the_remount = remounted.image_now();
    assert_eq!(
        (
            slots_of_the_stale_root_left_free(&after_the_remount, &remounted.allocator),
            violations_of(&after_the_remount)
        ),
        (Vec::new(), Vec::<String>::new()),
        "再挂之后：挂载时的回收门槛钉在 7，txg 7 引用的单元一个都不空闲，那几条不变量不红"
    );
    raise_rollback_floor(
        &parameters(),
        &mut remounted.devices,
        &mut remounted.allocator,
        &mut remounted.current,
        CheckpointTxg(TXG_OF_THE_STALE_ROOT_IN_THE_HOLE),
        ShadowLedger::On,
    )
    .expect("抬 F 到 7");
    let image = remounted.image_now();
    assert_eq!(
        (
            slots_of_the_stale_root_left_free(&image, &remounted.allocator),
            violations_of(&image)
        ),
        (Vec::new(), Vec::<String>::new()),
        "抬 F 到 7 之后：txg 7 引用的单元一个都不空闲，那几条不变量不红"
    );
}
