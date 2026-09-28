//! unreadable-at-mount-r1 云端攻方腿的原型（只在仓副本里跑）：C393 的候选（今天、(a)、(d)）在「被抛弃根的账读不出」的历史上，
//! 可写挂载成不成、隔离几个、之后复用不复用被抛弃根还引用的单元、池级 checker 的 I-7.4 红不红。每段历史打一行 `name=c393grid …`，不做断言。
//!
//! 起点状态每个只建一次池（文件镜像建完立刻转成内存稀疏盘），重开挂载与之后的覆盖写都在内存盘上。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
    crash_state_devices, parameters, publish_overwrite_in_process, with_unreadable_ranges,
    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread, BeforeTheOneReread,
    ShadowLedger,
};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolVersion, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;
use std::cell::Cell;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 起点：盘上的样子与被抛弃的那几条根那次发布的输出（按抛弃的先后）。
struct Start {
    name: &'static str,
    image: MemoryPool,
    abandoned: Vec<TransactionOutput>,
}

/// 调查员那段历史：A、B（实例 1）、重开取号 2、C (2, 8)；崩溃恢复抛弃 C（落到 (2, 7)，实例 3 写行 9、暖机 10）、C 写回。
/// `second_abandonment` 时再接：实例 3 发 D (3, 11)，第二次崩溃恢复抛弃 D（落到 (3, 10)，实例 4 写行、暖机）、D 写回——两条被抛弃根。
fn start(name: &'static str, second_abandonment: bool) -> Start {
    let mut pool = build_pool(name);
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
    let c = publish_overwrite_in_process(
        &mut pool,
        &before,
        &content_of(2500, 11),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(2),
    )
    .expect("C");
    let abandoning = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &c);
    let newest = *abandoning.current.root();
    eprintln!(
        "start {name}: C=({},{}) abandoned, newest now ({},{})",
        c.root.instance.0, c.root.checkpoint_txg.0, newest.instance.0, newest.checkpoint_txg.0
    );
    let mut abandoned = vec![c];
    if second_abandonment {
        // 抛弃 C 的那次挂载看不见 C（C 那时读不出），它交回的分配器没隔离 C；先干净地重开一次（C 读得出、影子账隔离它），
        // 在这次挂载的分配器上发 D，D 才不会复用 C 的单元（否则起点里 C 已被 D 盖掉，那是 C554 一族，不是这一格）。
        let mut devices = pool.reopen_recorded();
        let remounted =
            singlefs_core::mount::mount_writable(&parameters(), &mut devices).expect("干净重开");
        eprintln!(
            "start {name}: clean remount instance {} isolated {:?}",
            remounted.output.instance.0, remounted.output.isolated_slots_per_device
        );
        pool.devices = Some(devices);
        pool.allocator = remounted.allocator;
        pool.output = remounted.current.into_file_version().expect("带文件");
        let previous = pool.output.clone();
        let d = publish_overwrite_in_process(
            &mut pool,
            &previous,
            &content_of(2900, 13),
            FIXED_WRITE_TIME_SECONDS + 120,
            remounted.output.instance,
        )
        .expect("D");
        let second = abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &d);
        let newest = *second.current.root();
        eprintln!(
            "start {name}: D=({},{}) abandoned, newest now ({},{})",
            d.root.instance.0, d.root.checkpoint_txg.0, newest.instance.0, newest.checkpoint_txg.0
        );
        abandoned.push(d);
    }
    let image = pool.memory_pool();
    for (index, output) in abandoned.iter().enumerate() {
        eprintln!(
            "start {name}: abandoned[{index}]=({},{}) units already overwritten at start: {:?}",
            output.root.instance.0,
            output.root.checkpoint_txg.0,
            overwritten_units_of(&image, output)
        );
    }
    Start {
        name,
        image,
        abandoned,
    }
}

struct StartCell(Start);
static SINGLE: std::sync::OnceLock<StartCell> = std::sync::OnceLock::new();
static DOUBLE: std::sync::OnceLock<StartCell> = std::sync::OnceLock::new();

fn single() -> &'static Start {
    &SINGLE.get_or_init(|| StartCell(start("c393grid-single", false))).0
}
fn double() -> &'static Start {
    &DOUBLE.get_or_init(|| StartCell(start("c393grid-double", true))).0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Account {
    TreeTable,
    AllocationRecordTreeRoot,
    /// 分配记录树根之下的一个节点（多层树）。
    AllocationRecordTreeNodeBelowTheRoot,
    /// 树表与分配记录树根两样都读不出。
    TreeTableAndAllocationRecordTreeRoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unreadability {
    Readable,
    EveryRead,
    EveryReadUntilTheRereadHook,
    OnlyTheFirstReadOfEachCopy,
}

/// 连带读不出的 journal 记录（候选 (d) 从记录取「写过的槽」，看它缺了记录时怎样）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordsAlsoUnreadable {
    No,
    /// 被抛弃根自己那次发布的记录（两份）。
    TheAbandonedPublish,
    /// 被抛弃根所在实例的全部记录（两份）。
    EveryRecordOfItsInstance,
}

fn account_units(output: &TransactionOutput, account: Account) -> Vec<(u64, u64)> {
    let pick = |matches: &dyn Fn(&TransactionUnit) -> bool| -> Vec<(u64, u64)> {
        output
            .units
            .iter()
            .filter(|unit| matches(&unit.identity))
            .take(1)
            .map(|unit| (unit.slot.0, u64::try_from(unit.bytes.len()).expect("长度")))
            .collect()
    };
    match account {
        Account::TreeTable => pick(&|identity| *identity == TransactionUnit::TreeTable),
        Account::AllocationRecordTreeRoot => {
            pick(&|identity| *identity == TransactionUnit::AllocationTree)
        }
        Account::AllocationRecordTreeNodeBelowTheRoot => pick(&|identity| {
            matches!(identity, TransactionUnit::AllocationTreeNodeBelowTheRoot(_))
        }),
        Account::TreeTableAndAllocationRecordTreeRoot => {
            let mut units = pick(&|identity| *identity == TransactionUnit::TreeTable);
            units.extend(pick(&|identity| *identity == TransactionUnit::AllocationTree));
            units
        }
    }
}

fn record_counters_of(image: &MemoryPool, instance: InstanceGeneration) -> Vec<u64> {
    let system_configuration =
        singlefs_core::recovery::choose_system_configuration(image).expect("系统配置");
    singlefs_core::recovery::scan_journal(image, &system_configuration)
        .values()
        .filter(|record| record.instance == instance)
        .map(|record| record.counter)
        .collect()
}

/// C 那次发布写出的单元里，此刻两块盘上逐字节已不是它写的那几个（任一块盘）。
fn overwritten_units_of(image: &MemoryPool, abandoned: &TransactionOutput) -> Vec<(u64, String)> {
    abandoned
        .units
        .iter()
        .filter(|unit| {
            [DeviceIdentity(0), DeviceIdentity(1)].iter().any(|device| {
                image.devices[device].read(DeviceOffsetInBytes(unit.slot.0 * SLOT_BYTES), unit.bytes.len())
                    != unit.bytes
            })
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

type FaultyDevices = Vec<(
    DeviceIdentity,
    common::DeviceWithUnreadableRanges<
        singlefs_harness::RecordingBlockDevice<singlefs_harness::memory_pool::SparseBlockDevice>,
    >,
)>;

fn snapshot_of(devices: &FaultyDevices) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner_device().wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: common::IMAGE_BYTES,
    }
}

#[derive(Clone, Copy, Debug)]
struct Arm {
    /// 哪几条被抛弃根的账读不出（下标按抛弃的先后）。
    which_abandoned_roots: &'static [usize],
    account: Account,
    unreadability: Unreadability,
    read_back: UnreadableRangeReadBack,
    records: RecordsAlsoUnreadable,
}

fn run_arm(start: &Start, arm: &Arm) -> String {
    let failing = match arm.unreadability {
        Unreadability::Readable => FailingReadsOfARange::Never,
        Unreadability::EveryRead | Unreadability::EveryReadUntilTheRereadHook => {
            FailingReadsOfARange::Every
        }
        Unreadability::OnlyTheFirstReadOfEachCopy => FailingReadsOfARange::OnlyTheFirst,
    };
    let mut ranges: Vec<UnreadableRange> = Vec::new();
    for index in arm.which_abandoned_roots {
        let abandoned = &start.abandoned[*index];
        for (slot, length) in account_units(abandoned, arm.account) {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                ranges.push(UnreadableRange {
                    device,
                    offset_in_bytes: slot * SLOT_BYTES,
                    length_in_bytes: length,
                    failing_reads: failing,
                });
            }
        }
        let counters: Vec<u64> = match arm.records {
            RecordsAlsoUnreadable::No => Vec::new(),
            RecordsAlsoUnreadable::TheAbandonedPublish => vec![abandoned.record.counter],
            RecordsAlsoUnreadable::EveryRecordOfItsInstance => {
                record_counters_of(&start.image, abandoned.root.instance)
            }
        };
        for counter in counters {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                ranges.push(UnreadableRange {
                    device,
                    offset_in_bytes: record_offset(counter, parameters().geometry.journal_ring_bytes).0,
                    length_in_bytes: JOURNAL_RECORD_BYTES,
                    failing_reads: failing,
                });
            }
        }
    }
    let account_ranges = ranges.len();
    let unreadable = SharedUnreadableRanges::new(ranges, arm.read_back);
    let stream = SharedStream::new();
    let mut devices: FaultyDevices = with_unreadable_ranges(
        crash_state_devices(&start.image, &[], &[], &stream),
        &unreadable,
    );
    let hook_calls = Cell::new(0u32);
    let lifting = unreadable.clone();
    let lift_in_hook = arm.unreadability == Unreadability::EveryReadUntilTheRereadHook;
    let mut hook = || {
        hook_calls.set(hook_calls.get() + 1);
        if lift_in_hook {
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
    let key = format!(
        "start={} abandoned_roots_unreadable_account={:?} account={:?} unreadability={:?} read_back={:?} records_also_unreadable={:?} fault_ranges={account_ranges}",
        start.name, arm.which_abandoned_roots, arm.account, arm.unreadability, arm.read_back, arm.records
    );
    let failed_reads = unreadable.failed_reads();
    let mut mounted = match mounted {
        Err(error) => {
            let text: String = format!("{error:?}").chars().take(160).collect();
            return format!(
                "name=c393grid {key} writable=refused hook_calls={} failed_reads={failed_reads} error={text}",
                hook_calls.get()
            );
        }
        Ok(mounted) => mounted,
    };
    unreadable.lift();
    let header = format!(
        "name=c393grid {key} writable=ok instance={} isolated={:?} abandoned_roots_unreadable={} hook_calls={} failed_reads={failed_reads}",
        mounted.output.instance.0,
        mounted.output.isolated_slots_per_device,
        mounted.output.abandoned_roots_unreadable,
        hook_calls.get()
    );
    let params = parameters();
    loop {
        let previous = mounted.current.file_version().expect("带文件").clone();
        if previous.root.checkpoint_txg >= CheckpointTxg(31) {
            let image = snapshot_of(&devices);
            return format!(
                "{header} reuse=none_through_txg={} I-7.4={:?} red={:?}",
                previous.root.checkpoint_txg.0,
                verdict_of(&image, "I-7.4"),
                violated(&image)
            );
        }
        let steps = usize::try_from(previous.root.checkpoint_txg.0).expect("txg");
        let published = {
            let mut writer = PoolWriter::new(&params, devices.as_mut_slice());
            publish_overwrite(
                &mut writer,
                &mut mounted.allocator,
                &previous,
                FirstFile {
                    content: &content_of(2000 + steps, steps + 41),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600,
                },
                mounted.output.instance,
            )
        };
        let published = match published {
            Ok(output) => output,
            Err(error) => return format!("{header} publish_err={error:?}"),
        };
        mounted.current = PoolVersion::WithFile(published);
        let image = snapshot_of(&devices);
        let overwritten: Vec<(usize, Vec<(u64, String)>)> = start
            .abandoned
            .iter()
            .enumerate()
            .map(|(index, abandoned)| (index, overwritten_units_of(&image, abandoned)))
            .filter(|(_, units)| !units.is_empty())
            .collect();
        if !overwritten.is_empty() {
            let txg = mounted.current.root().checkpoint_txg.0;
            let verdict = verdict_of(&image, "I-7.4");
            let text: String = format!("{verdict:?}").chars().take(120).collect();
            return format!(
                "{header} reuse_txg={txg} overwritten={overwritten:?} I-7.4={text} red={:?}",
                violated(&image)
            );
        }
    }
}

const UNREADABILITIES: [Unreadability; 4] = [
    Unreadability::Readable,
    Unreadability::EveryRead,
    Unreadability::EveryReadUntilTheRereadHook,
    Unreadability::OnlyTheFirstReadOfEachCopy,
];

fn run_many(start: &Start, which: &'static [usize], accounts: &[Account], records: RecordsAlsoUnreadable) -> usize {
    let mut lines = 0;
    for account in accounts {
        for unreadability in UNREADABILITIES {
            for read_back in [UnreadableRangeReadBack::DeviceError, UnreadableRangeReadBack::Zeros] {
                if unreadability == Unreadability::Readable
                    && read_back == UnreadableRangeReadBack::Zeros
                {
                    continue;
                }
                let arm = Arm {
                    which_abandoned_roots: which,
                    account: *account,
                    unreadability,
                    read_back,
                    records,
                };
                eprintln!("{}", run_arm(start, &arm));
                lines += 1;
            }
        }
    }
    lines
}

macro_rules! c393_tests {
    ($($test_name:ident => ($start:expr, $which:expr, $account:expr, $records:expr);)*) => {
        $(
            #[test]
            fn $test_name() {
                let lines = run_many($start, $which, &[$account], $records);
                eprintln!("name=c393grid-count test={} scenarios={lines}", stringify!($test_name));
            }
        )*
    };
}

c393_tests! {
    single_tree_table => (single(), &[0], Account::TreeTable, RecordsAlsoUnreadable::No);
    single_allocation_record_tree_root => (single(), &[0], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::No);
    single_allocation_record_tree_node_below_the_root => (single(), &[0], Account::AllocationRecordTreeNodeBelowTheRoot, RecordsAlsoUnreadable::No);
    single_tree_table_and_allocation_record_tree_root => (single(), &[0], Account::TreeTableAndAllocationRecordTreeRoot, RecordsAlsoUnreadable::No);
    records_of_the_abandoned_publish_tree_table => (single(), &[0], Account::TreeTable, RecordsAlsoUnreadable::TheAbandonedPublish);
    records_of_the_abandoned_publish_allocation_record_tree_root => (single(), &[0], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::TheAbandonedPublish);
    records_of_its_instance_tree_table => (single(), &[0], Account::TreeTable, RecordsAlsoUnreadable::EveryRecordOfItsInstance);
    records_of_its_instance_allocation_record_tree_root => (single(), &[0], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::EveryRecordOfItsInstance);
    double_first_tree_table => (double(), &[0], Account::TreeTable, RecordsAlsoUnreadable::No);
    double_first_allocation_record_tree_root => (double(), &[0], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::No);
    double_second_tree_table => (double(), &[1], Account::TreeTable, RecordsAlsoUnreadable::No);
    double_second_allocation_record_tree_root => (double(), &[1], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::No);
    double_both_tree_table => (double(), &[0, 1], Account::TreeTable, RecordsAlsoUnreadable::No);
    double_both_allocation_record_tree_root => (double(), &[0, 1], Account::AllocationRecordTreeRoot, RecordsAlsoUnreadable::No);
}
