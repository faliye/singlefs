//! harness 测试文件 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use std::cell::Cell;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Account {
    AllocationRecordTreeRoot,
    TreeTable,
}

impl Account {
    fn unit(self) -> TransactionUnit {
        match self {
            Account::AllocationRecordTreeRoot => TransactionUnit::AllocationTree,
            Account::TreeTable => TransactionUnit::TreeTable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unreadability {
    /// 阳性对照：包装在，一次都不坏。
    Readable,
    /// 两份每次读都坏，钩子不撤（产品路径 RereadImmediately）。
    EveryRead,
    /// 两份每次读都坏，直到产品的「重读一次」钩子被调用时撤掉（若产品重读，重读读得出）。
    EveryReadUntilTheRereadHook,
    /// 两份各只坏第一次读，之后照读（设备一层的瞬时错）。
    OnlyTheFirstReadOfEachCopy,
}

/// A、B（实例 1）、重开取号 2、C (2, 8)；崩溃恢复抛弃 C（落到 (2, 7)，实例 3 写行 9、暖机 10）、C 写回。
pub fn pool_after_a_recovery_abandoned_c(tag: &str) -> (BuiltPool, TransactionOutput) {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    pool.output = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(4100, 3),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("B");
    let mut devices = pool.reopen_recorded();
    let mounted =
        singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("取号 2");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current.into_file_version().expect("带文件");
    let before = pool.output.clone();
    let abandoned = publish_overwrite_in_process(
        &mut pool,
        &before,
        &content_of(2500, 11),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("C");
    let abandoning =
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &abandoned);
    let newest = *abandoning.current.root();
    assert_eq!(
        (newest.instance, newest.checkpoint_txg),
        (InstanceGeneration(3), CheckpointTxg(10))
    );
    (pool, abandoned)
}

pub struct Remount {
    pub pool: BuiltPool,
    pub abandoned: TransactionOutput,
    pub account_slot: u64,
    pub result: Result<(), String>,
    pub isolated: Vec<(DeviceIdentity, u64)>,
    pub abandoned_roots_unreadable: u64,
    pub rereads: String,
    pub space_admission: String,
    pub reads_of_each_copy: (u64, u64),
    pub failed_reads: u64,
    pub hook_calls: u32,
    pub current_txg: Option<CheckpointTxg>,
    /// C 那次发布写出的单元里，重开之后在盘 0 的分配器里是空闲的那几个（起点槽）。
    pub free_units_of_c_after_remount: Vec<u64>,
}

pub fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount {
    let (mut pool, abandoned) = pool_after_a_recovery_abandoned_c(tag);
    let account_unit = abandoned.unit(account.unit());
    let failing = match unreadability {
        Unreadability::Readable => FailingReadsOfARange::Never,
        Unreadability::EveryRead | Unreadability::EveryReadUntilTheRereadHook => {
            FailingReadsOfARange::Every
        }
        Unreadability::OnlyTheFirstReadOfEachCopy => FailingReadsOfARange::OnlyTheFirst,
    };
    let ranges: Vec<UnreadableRange> = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|device| UnreadableRange {
            device,
            offset_in_bytes: account_unit.slot.0 * SLOT_BYTES,
            length_in_bytes: u64::try_from(account_unit.bytes.len()).expect("长度"),
            failing_reads: failing,
        })
        .collect();
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        if unreadability == Unreadability::EveryReadUntilTheRereadHook {
            lifting.lift();
        }
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
    );
    let reads_of_each_copy = (unreadable.reads_of_range(0), unreadable.reads_of_range(1));
    let failed_reads = unreadable.failed_reads();
    pool.devices = Some(without_unreadable_ranges(devices));
    let mut remount = Remount {
        pool,
        abandoned: abandoned.clone(),
        account_slot: account_unit.slot.0,
        result: Ok(()),
        isolated: Vec::new(),
        abandoned_roots_unreadable: 0,
        rereads: String::new(),
        space_admission: String::new(),
        reads_of_each_copy,
        failed_reads,
        hook_calls: hook_calls.get(),
        current_txg: None,
        free_units_of_c_after_remount: Vec::new(),
    };
    match mounted {
        Ok(mounted) => {
            assert_eq!(mounted.output.instance, InstanceGeneration(4));
            remount.isolated = mounted.output.isolated_slots_per_device.clone();
            remount.abandoned_roots_unreadable = mounted.output.abandoned_roots_unreadable;
            remount.rereads = format!("{:?}", mounted.output.rereads);
            remount.space_admission = format!("{:?}", mounted.output.space_admission);
            remount.free_units_of_c_after_remount = abandoned
                .units
                .iter()
                .filter(|unit| mounted.allocator.devices[0].is_free(unit.slot))
                .map(|unit| unit.slot.0)
                .collect();
            remount.pool.allocator = mounted.allocator;
            remount.pool.output = mounted.current.into_file_version().expect("带文件");
            remount.current_txg = Some(remount.pool.output.root.checkpoint_txg);
        }
        Err(error) => remount.result = Err(format!("{error:?}")),
    }
    remount
}

/// C 那次发布写出的单元里，此刻盘 0 上逐字节已不是 C 写的那几个。
pub fn overwritten_units_of(
    image: &MemoryPool,
    abandoned: &TransactionOutput,
) -> Vec<(u64, String)> {
    let device = image.devices.get(&DeviceIdentity(0)).expect("盘 0");
    abandoned
        .units
        .iter()
        .filter(|unit| {
            device.read(
                DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES),
                unit.bytes.len(),
            ) != unit.bytes
        })
        .map(|unit| (unit.slot.0, unit.identity.tag()))
        .collect()
}

/// 实例 4 里接着覆盖写，直到 C 的某个单元在盘上被改写或 txg 到 31（txg 32 的根盖掉 C 的根槽）。
/// 交回一行：第一次改写时的 txg 与被改写的单元；没有改写就是写到了哪个 txg，那一刻的镜像交给池级 checker（I-7.4 判成立、一条不红）。
pub fn continue_until_a_unit_of_c_is_reused(
    remount: &mut Remount,
    images: &mut PoolCheckerImages,
) -> String {
    let mut report = String::new();
    while remount.pool.output.root.checkpoint_txg < CheckpointTxg(31) {
        let steps = usize::try_from(remount.pool.output.root.checkpoint_txg.0).expect("txg");
        let previous = remount.pool.output.clone();
        remount.pool.output = publish_overwrite_in_process(
            &mut remount.pool,
            &previous,
            &content_of(2000 + steps, steps + 41),
            FIXED_WRITE_TIME_SECONDS + 600,
            InstanceGeneration(4),
        )
        .expect("实例 4 覆盖写");
        let image = remount.pool.memory_pool();
        let overwritten = overwritten_units_of(&image, &remount.abandoned);
        if !overwritten.is_empty() {
            let txg = remount.pool.output.root.checkpoint_txg;
            report.push_str(&format!("reuse_txg={} overwritten={overwritten:?}", txg.0));
            return report;
        }
    }
    report.push_str(&format!(
        "no_reuse_through_txg={}",
        remount.pool.output.root.checkpoint_txg.0
    ));
    images.hand_out(
        &format!(
            "写到 txg {} 之后",
            remount.pool.output.root.checkpoint_txg.0
        ),
        || remount.pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated().and_holds("I-7.4"),
    );
    report
}

pub fn run_arm(
    account: Account,
    unreadability: Unreadability,
    images: &mut PoolCheckerImages,
) -> (Remount, String) {
    let tag = format!("c393-{account:?}-{unreadability:?}");
    let mut remount = remount(&tag, account, unreadability);
    let line = format!(
        "name=c393 account={account:?} unreadability={unreadability:?} account_slot={} result={:?} current_txg={:?} free_units_of_c_after_remount={:?} isolated={:?} abandoned_roots_unreadable={} reads_of_each_copy={:?} failed_reads={} hook_calls={} rereads={} space_admission={}",
        remount.account_slot,
        remount.result,
        remount.current_txg.map(|txg| txg.0),
        remount.free_units_of_c_after_remount,
        remount.isolated,
        remount.abandoned_roots_unreadable,
        remount.reads_of_each_copy,
        remount.failed_reads,
        remount.hook_calls,
        remount.rereads,
        remount.space_admission,
    );
    let reuse = if remount.result.is_ok() {
        continue_until_a_unit_of_c_is_reused(&mut remount, images)
    } else {
        "mount_refused".to_string()
    };
    (remount, format!("{line} {reuse}"))
}

pub const ISOLATED_14: [(DeviceIdentity, u64); 2] =
    [(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)];

/// 读不出的那几臂共同的结局：照常可写挂载（实例 4，txg 到 13），不重读（现算就成了，钩子 0 次），计数 1（显式报告），
/// 从映射现算照样隔离每盘 14 槽，之后覆盖写到 txg 31 C 的单元一个都不复用、I-7.4 成立。
pub fn assert_recomputed_from_the_mapping_isolated_and_not_reused(
    account: Account,
    unreadability: Unreadability,
    images: &mut PoolCheckerImages,
) {
    let (remount, line) = run_arm(account, unreadability, images);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()), "照常可写挂载");
    assert_eq!(remount.current_txg, Some(CheckpointTxg(13)));
    assert_eq!(remount.hook_calls, 0, "从映射现算就成了，不走「重读一次」");
    assert_eq!(
        remount.abandoned_roots_unreadable, 1,
        "读不出的账照样计数往上报"
    );
    assert_eq!(
        remount.isolated,
        ISOLATED_14.to_vec(),
        "现算出的落点与读得出那一臂隔离的一样多"
    );
    assert_eq!(
        remount.free_units_of_c_after_remount,
        Vec::<u64>::new(),
        "C 的单元一个都没回到空闲"
    );
    assert!(line.contains("no_reuse_through_txg=31"), "{line}");
}

pub fn readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
    account: Account,
    images: &mut PoolCheckerImages,
) {
    let (remount, line) = run_arm(account, Unreadability::Readable, images);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(
        remount.reads_of_each_copy.0 >= 1,
        "影子账那一读真的读到了这一段"
    );
    assert_eq!(remount.free_units_of_c_after_remount, Vec::<u64>::new());
    assert!(line.contains("no_reuse_through_txg=31"), "{line}");
}

pub fn c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
    images: &mut PoolCheckerImages,
) {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
        Account::AllocationRecordTreeRoot,
        images,
    );
}

pub fn c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
    images: &mut PoolCheckerImages,
) {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(
        Account::TreeTable,
        images,
    );
}

pub fn c393_allocation_record_tree_unreadable_on_every_read(images: &mut PoolCheckerImages) {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::AllocationRecordTreeRoot,
        Unreadability::EveryRead,
        images,
    );
}

pub fn c393_allocation_record_tree_unreadable_until_the_reread_hook(
    images: &mut PoolCheckerImages,
) {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::AllocationRecordTreeRoot,
        Unreadability::EveryReadUntilTheRereadHook,
        images,
    );
}

/// 分配记录树根节点是进映射的节点：提示两份读不出时经中央映射回退再读一次（`recovery::read_mapped_tree_node_via_hint_then_central_mapping`），
/// 映射落点与提示同槽，第二次读读得出——设备一层只坏第一次的瞬时错在这里被吸收。
pub fn c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy(
    images: &mut PoolCheckerImages,
) {
    let (remount, line) = run_arm(
        Account::AllocationRecordTreeRoot,
        Unreadability::OnlyTheFirstReadOfEachCopy,
        images,
    );
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(
        remount.reads_of_each_copy,
        (2, 1),
        "盘 0 提示一读、盘 1 提示一读、映射回退盘 0 再一读"
    );
    assert_eq!(remount.failed_reads, 2);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(line.contains("no_reuse_through_txg=31"), "{line}");
}

pub fn c393_tree_table_unreadable_on_every_read(images: &mut PoolCheckerImages) {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::TreeTable,
        Unreadability::EveryRead,
        images,
    );
}

pub fn c393_tree_table_unreadable_until_the_reread_hook(images: &mut PoolCheckerImages) {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::TreeTable,
        Unreadability::EveryReadUntilTheRereadHook,
        images,
    );
}

/// 树表豁免映射，只按位置条目读一遍（两份各一次），没有第二次读：只坏第一次与一直坏结局相同。
pub fn c393_tree_table_unreadable_only_on_the_first_read_of_each_copy(
    images: &mut PoolCheckerImages,
) {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::TreeTable,
        Unreadability::OnlyTheFirstReadOfEachCopy,
        images,
    );
}

/// 账在盘上真坏（两份抹零）而没有复用时，池级 checker 对 I-7.4 判什么：拿读得出那一臂重开之后、还没接着写的镜像抹 C 的账。
/// 抹之前那一张 I-7.4 判成立；抹之后那一张交出去只看判成什么（不定判据，checker 档那一侧打出它的结论）。
pub fn checker_on_a_persistently_damaged_account_without_reuse(
    account: Account,
    images: &mut PoolCheckerImages,
) {
    let tag = format!("c393-damaged-{account:?}");
    let remount = remount(&tag, account, Unreadability::Readable);
    images.hand_out(
        "重开之后、抹 C 的账之前",
        || remount.pool.memory_pool(),
        PoolCheckerExpectation::only_the_named_invariants().and_holds("I-7.4"),
    );
    images.hand_out(
        "C 的账两份抹零之后",
        || {
            let mut damaged = remount.pool.memory_pool();
            for device in [0u32, 1] {
                damaged
                    .devices
                    .get_mut(&DeviceIdentity(device))
                    .expect("盘")
                    .write(
                        DeviceOffsetInBytes(remount.account_slot * SLOT_BYTES),
                        &vec![0u8; usize::try_from(SLOT_BYTES).expect("槽宽")],
                    );
            }
            damaged
        },
        PoolCheckerExpectation::only_the_named_invariants(),
    );
}

pub fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse(
    images: &mut PoolCheckerImages,
) {
    checker_on_a_persistently_damaged_account_without_reuse(
        Account::AllocationRecordTreeRoot,
        images,
    );
}

pub fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse(
    images: &mut PoolCheckerImages,
) {
    checker_on_a_persistently_damaged_account_without_reuse(Account::TreeTable, images);
}
