//! C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算（用户 2026-09-28 定，被攻过零轮；
//! `research/prompts/unreadable-at-mount-r2-main-verification.md` L3）：崩溃恢复抛弃根 C (2, 8) 之后再重开可写挂载（实例 4），
//! C 的分配记录树根节点或树表两份都读不出时，影子账不再跳过 C，从中央映射现算 C 引用的落点照样隔离（与读得出那一臂同是每盘 14 槽），
//! 计进 `abandoned_roots_unreadable` 往上报；之后覆盖写到 txg 31（txg 32 的根盖掉 C 的根槽）C 的单元一个都不复用、I-7.4（近 K 代块未被复用） 成立。
//! 改之前读不出只计数、跳过，隔离 0，重开之后第一次覆盖写（txg 14）就把 C 的数据单元（盘 0 槽 50184）改写、池级 checker 红 I-7.4
//! （调查员报告 `research/prompts/closeout-recheck-2026-09-28/c393-investigator-report.md`；这份用例从那份调查的副本用例改来）。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
    publish_overwrite_in_process, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::transaction::{TransactionOutput, TransactionUnit};
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::MemoryPool;
use std::cell::Cell;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Account {
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
enum Unreadability {
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
fn pool_after_a_recovery_abandoned_c(tag: &str) -> (BuiltPool, TransactionOutput) {
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
    let mounted = singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("取号 2");
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

struct Remount {
    pool: BuiltPool,
    abandoned: TransactionOutput,
    account_slot: u64,
    result: Result<(), String>,
    isolated: Vec<(DeviceIdentity, u64)>,
    abandoned_roots_unreadable: u64,
    rereads: String,
    space_admission: String,
    reads_of_each_copy: (u64, u64),
    failed_reads: u64,
    hook_calls: u32,
    current_txg: Option<CheckpointTxg>,
    /// C 那次发布写出的单元里，重开之后在盘 0 的分配器里是空闲的那几个（起点槽）。
    free_units_of_c_after_remount: Vec<u64>,
}

fn remount(tag: &str, account: Account, unreadability: Unreadability) -> Remount {
    let (mut pool, abandoned) = pool_after_a_recovery_abandoned_c(tag);
    let unit = abandoned.unit(account.unit());
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
            offset_in_bytes: unit.slot.0 * SLOT_BYTES,
            length_in_bytes: u64::try_from(unit.bytes.len()).expect("长度"),
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
        account_slot: unit.slot.0,
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
fn overwritten_units_of(image: &MemoryPool, abandoned: &TransactionOutput) -> Vec<(u64, String)> {
    let device = image.devices.get(&DeviceIdentity(0)).expect("盘 0");
    abandoned
        .units
        .iter()
        .filter(|unit| {
            device.read(DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES), unit.bytes.len()) != unit.bytes
        })
        .map(|unit| (unit.slot.0, unit.identity.tag()))
        .collect()
}

fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(name, _)| *name == invariant)
        .expect("清单里有")
        .1
}

fn violated(image: &MemoryPool) -> Vec<&'static str> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| matches!(verdict, InvariantVerdict::Violated(_)).then_some(name))
        .collect()
}

/// 实例 4 里接着覆盖写，直到 C 的某个单元在盘上被改写或 txg 到 31（txg 32 的根盖掉 C 的根槽）。
/// 交回 (第一次改写时的 txg, 被改写的单元, 那一刻 I-7.4 的判定, 那一刻红的不变量, C 的账两份抹零之后 I-7.4 的判定)。
fn continue_until_a_unit_of_c_is_reused(remount: &mut Remount) -> String {
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
            let verdict = verdict_of(&image, "I-7.4");
            let red = violated(&image);
            let mut damaged = image.clone();
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
            let damaged_verdict = verdict_of(&damaged, "I-7.4");
            report.push_str(&format!(
                "reuse_txg={} overwritten={overwritten:?} I-7.4={verdict:?} red={red:?} I-7.4_after_zeroing_the_account={damaged_verdict:?}",
                txg.0
            ));
            return report;
        }
    }
    let image = remount.pool.memory_pool();
    report.push_str(&format!(
        "no_reuse_through_txg={} I-7.4={:?} red={:?}",
        remount.pool.output.root.checkpoint_txg.0,
        verdict_of(&image, "I-7.4"),
        violated(&image)
    ));
    report
}

fn run_arm(account: Account, unreadability: Unreadability) -> (Remount, String) {
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
        continue_until_a_unit_of_c_is_reused(&mut remount)
    } else {
        "mount_refused".to_string()
    };
    (remount, format!("{line} {reuse}"))
}

const ISOLATED_14: [(DeviceIdentity, u64); 2] = [(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)];

/// 读不出的那几臂共同的结局：照常可写挂载（实例 4，txg 到 13），不重读（现算就成了，钩子 0 次），计数 1（显式报告），
/// 从映射现算照样隔离每盘 14 槽，之后覆盖写到 txg 31 C 的单元一个都不复用、I-7.4 成立。
fn assert_recomputed_from_the_mapping_isolated_and_not_reused(account: Account, unreadability: Unreadability) {
    let (remount, line) = run_arm(account, unreadability);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()), "照常可写挂载");
    assert_eq!(remount.current_txg, Some(CheckpointTxg(13)));
    assert_eq!(remount.hook_calls, 0, "从映射现算就成了，不走「重读一次」");
    assert_eq!(remount.abandoned_roots_unreadable, 1, "读不出的账照样计数往上报");
    assert_eq!(remount.isolated, ISOLATED_14.to_vec(), "现算出的落点与读得出那一臂隔离的一样多");
    assert_eq!(remount.free_units_of_c_after_remount, Vec::<u64>::new(), "C 的单元一个都没回到空闲");
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

fn readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(account: Account) {
    let (remount, line) = run_arm(account, Unreadability::Readable);
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(remount.reads_of_each_copy.0 >= 1, "影子账那一读真的读到了这一段");
    assert_eq!(remount.free_units_of_c_after_remount, Vec::<u64>::new());
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

#[test]
fn c393_readable_allocation_record_tree_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(Account::AllocationRecordTreeRoot);
}

#[test]
fn c393_readable_tree_table_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31() {
    readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31(Account::TreeTable);
}

#[test]
fn c393_allocation_record_tree_unreadable_on_every_read() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::AllocationRecordTreeRoot, Unreadability::EveryRead);
}

#[test]
fn c393_allocation_record_tree_unreadable_until_the_reread_hook() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(
        Account::AllocationRecordTreeRoot,
        Unreadability::EveryReadUntilTheRereadHook,
    );
}

/// 分配记录树根节点是进映射的节点：提示两份读不出时经中央映射回退再读一次（`recovery::read_mapped_tree_node_via_hint_then_central_mapping`），
/// 映射落点与提示同槽，第二次读读得出——设备一层只坏第一次的瞬时错在这里被吸收。
#[test]
fn c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy() {
    let (remount, line) = run_arm(
        Account::AllocationRecordTreeRoot,
        Unreadability::OnlyTheFirstReadOfEachCopy,
    );
    eprintln!("{line}");
    assert_eq!(remount.result, Ok(()));
    assert_eq!(remount.hook_calls, 0);
    assert_eq!(remount.reads_of_each_copy, (2, 1), "盘 0 提示一读、盘 1 提示一读、映射回退盘 0 再一读");
    assert_eq!(remount.failed_reads, 2);
    assert_eq!(remount.abandoned_roots_unreadable, 0);
    assert_eq!(remount.isolated, ISOLATED_14.to_vec());
    assert!(line.contains("no_reuse_through_txg=31 I-7.4=Holds red=[]"), "{line}");
}

#[test]
fn c393_tree_table_unreadable_on_every_read() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::EveryRead);
}

#[test]
fn c393_tree_table_unreadable_until_the_reread_hook() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::EveryReadUntilTheRereadHook);
}

/// 树表豁免映射，只按位置条目读一遍（两份各一次），没有第二次读：只坏第一次与一直坏结局相同。
#[test]
fn c393_tree_table_unreadable_only_on_the_first_read_of_each_copy() {
    assert_recomputed_from_the_mapping_isolated_and_not_reused(Account::TreeTable, Unreadability::OnlyTheFirstReadOfEachCopy);
}

/// 钩子的阳性对照：同一段历史，最新那条根 (3, 10) 的实例表那一片在影子账那一读起坏、钩子里撤——产品的重读走到钩子（1 次），
/// 重读读得出，照隔离 14。证明上面各臂的「钩子 0 次」不是钩子接错了。
#[test]
fn c393_positive_control_the_hook_is_called_when_the_newest_instance_table_is_unreadable() {
    let (mut pool, _) = pool_after_a_recovery_abandoned_c("c393-hook-positive-control");
    let mut devices_for_the_page = pool.reopen_recorded();
    let newest = singlefs_core::recovery::choose_root(
        &devices_for_the_page,
        &singlefs_core::recovery::choose_system_configuration(&devices_for_the_page).expect("择"),
    )
    .expect("最新根");
    let _ = &mut devices_for_the_page;
    pool.devices = Some(devices_for_the_page);
    let page = newest.instance_table;
    let ranges: Vec<UnreadableRange> = page
        .locations
        .iter()
        .map(|location| UnreadableRange {
            device: location.device,
            offset_in_bytes: location.slot.to_device_offset().0,
            length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
            failing_reads: if location.device == DeviceIdentity(0) {
                FailingReadsOfARange::FromTheNthOnward(3)
            } else {
                FailingReadsOfARange::Every
            },
        })
        .collect();
    let unreadable = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &unreadable);
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        lifting.lift();
    };
    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
        &parameters(),
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        BeforeTheOneReread::CallTheTestOnlyHookFirst(&mut hook),
    )
    .expect("重读读得出");
    pool.devices = Some(without_unreadable_ranges(devices));
    eprintln!(
        "name=c393-hook-positive-control hook_calls={} isolated={:?} abandoned_roots_unreadable={} rereads={:?}",
        hook_calls.get(),
        mounted.output.isolated_slots_per_device,
        mounted.output.abandoned_roots_unreadable,
        mounted.output.rereads
    );
    assert_eq!(hook_calls.get(), 1);
    assert_eq!(mounted.output.isolated_slots_per_device, ISOLATED_14.to_vec());
}

/// 账在盘上真坏（两份抹零）而没有复用时，池级 checker 对 I-7.4 判什么：拿读得出那一臂重开之后、还没接着写的镜像抹 C 的账。
fn checker_on_a_persistently_damaged_account_without_reuse(account: Account) {
    let tag = format!("c393-damaged-{account:?}");
    let remount = remount(&tag, account, Unreadability::Readable);
    let image = remount.pool.memory_pool();
    let before = verdict_of(&image, "I-7.4");
    let mut damaged = image.clone();
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
    let after = verdict_of(&damaged, "I-7.4");
    let red = violated(&damaged);
    eprintln!("name=c393-damaged account={account:?} I-7.4_before={before:?} I-7.4_after={after:?} red_after={red:?}");
    assert_eq!(before, InvariantVerdict::Holds);
}

#[test]
fn c393_checker_on_a_persistently_damaged_allocation_record_tree_without_reuse() {
    checker_on_a_persistently_damaged_account_without_reuse(Account::AllocationRecordTreeRoot);
}

#[test]
fn c393_checker_on_a_persistently_damaged_tree_table_without_reuse() {
    checker_on_a_persistently_damaged_account_without_reuse(Account::TreeTable);
}
