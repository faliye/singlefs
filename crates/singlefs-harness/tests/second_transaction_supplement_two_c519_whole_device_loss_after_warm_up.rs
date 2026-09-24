//! 里程碑「第二个事务」增补 2 收口表第 61 行（C519（丢一整块盘时恢复丢掉刚确认的那一版））：入库装置上的复现，只坐实、不修。
//!
//! 历史（`research/prompts/m2-presumed-clauses-r1-main-verification.md` 第四节）：mkfs → 第一个文件（txg 3）→ 进程退出 →
//! 重开可写挂载（取号 2、写行 txg 4 落盘 1、暖机 txg 5 落盘 0，本实例的根覆盖两块盘，D16（发布语义） 已定项 8）→
//! 发一版数据 txg 6（发布返回 = fsync 已返回，它的根落盘 0）→ 盘 0 整块掉了 → 在幸存的盘 1 上冷恢复。零崩溃、一个故障。
//!
//! 今天的结局：择到 (2, 4)，水位之上 jsn 5（暖机 txg 5）与 jsn 6（txg 6）两条都在；施加 jsn 5 之前逐项验点名单元，
//! `crates/singlefs-core/src/recovery.rs` 的 `replay_journal` 要一个单元的两条位置条目都读得出、校验和都对，
//! 其中一条在掉了的盘 0 上 ⇒ 验证失败、前缀停在第一条，读回的是 (2, 4) 那一版（第一个文件的内容），txg 6 那一版丢了。
//! 没有哪个错误成员报出来：结局是 `RecoveryOutcome::FileRead`，只在扫描报告里记一次 `verification_failed`。
//!
//! 「盘整块掉了」两种形态各做一遍：盘 0 每一次读都报块设备错（故障注入，降级读走的那条读路径），
//! 与盘 0 换成一块从没写过的盘（三方攻方腿的模型：每一处读回全 0、自证全不过）。两种形态结局逐项相同。
//!
//! D16（发布语义） 已定项 8 的暖机买的正是「根所在的盘掉了之后退到本实例的另一条根、靠重放追上」；
//! D23（journal 的角色与格式） 已定项 14「施加前逐项验证点名单元」字面没说一个单元的两份要都过还是任一份过。
//! 那一条期望「那一版还在」的用例标 `#[ignore]` 留在这里，等用户定。

mod common;

use common::{build_pool, file_content, parameters, BuiltPool};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseDevice};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};

/// 掉了的那块盘：txg 6 的根落在区域 0，区域 0 归盘 0（D2（RAID 条带策略） 已定项 7 的 0 / 1 / 0）。
const LOST_DEVICE: DeviceIdentity = DeviceIdentity(0);
const SURVIVING_DEVICE: DeviceIdentity = DeviceIdentity(1);

/// 挂载之后发的那一版数据的内容：与第一个文件不同，读回时分得出是哪一版。
fn acknowledged_version_content() -> Vec<u8> {
    (0..3700usize)
        .map(|index| u8::try_from((index * 13 + 5) % 251).expect("小于 256"))
        .collect()
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 盘 0 整块掉了的两种形态。
#[derive(Clone, Copy, Debug)]
enum WholeDeviceLoss {
    /// 盘 0 还在池里，每一次读都报块设备错。
    EveryReadOfTheLostDeviceFails,
    /// 盘 0 换成一块从没写过的盘：每一处读回全 0。
    LostDeviceReplacedByANeverWrittenDevice,
}

const BOTH_LOSS_FORMS: [WholeDeviceLoss; 2] = [
    WholeDeviceLoss::EveryReadOfTheLostDeviceFails,
    WholeDeviceLoss::LostDeviceReplacedByANeverWrittenDevice,
];

/// 暖机覆盖两块盘之后又确认了一版数据的池：写行那次与暖机那次各自的记录（点名单元要看）、确认的那一版的 txg 与 jsn。
struct AcknowledgedVersionAfterWarmUp {
    pool: BuiltPool,
    warm_up_record: singlefs_core::journal::JournalRecord,
    acknowledged_txg: CheckpointTxg,
    acknowledged_counter: u64,
}

fn build_acknowledged_version_after_warm_up(tag: &str) -> AcknowledgedVersionAfterWarmUp {
    let mut pool = build_pool(tag);
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(3));
    let mut devices = pool.reopen_recorded();
    let mut mounted = mount_writable(&parameters(), &mut devices).expect("重开可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg,
        CheckpointTxg(4)
    );
    assert_eq!(region_device(4), SURVIVING_DEVICE, "写行 txg 4 落盘 1");
    let warm_up_txgs: Vec<CheckpointTxg> = mounted
        .output
        .warm_up_publishes
        .iter()
        .map(|publish| publish.root().checkpoint_txg)
        .collect();
    assert_eq!(
        warm_up_txgs,
        vec![CheckpointTxg(5)],
        "暖机一次、txg 5 落盘 0，本实例的根覆盖两块盘"
    );
    assert_eq!(region_device(5), LOST_DEVICE);
    let warm_up_record = mounted.output.warm_up_publishes[0].record().clone();
    let current = mounted
        .current
        .file_version()
        .expect("第一个文件之后重开，现行那一版带文件")
        .clone();
    let acknowledged = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut mounted.allocator,
            &current,
            FirstFile {
                content: &acknowledged_version_content(),
                write_time_seconds: common::FIXED_WRITE_TIME_SECONDS + 600,
            },
            InstanceGeneration(2),
        )
        .expect("发一版数据：发布返回就是 fsync 已返回")
    };
    assert_eq!(
        region_device(acknowledged.root.checkpoint_txg.0),
        LOST_DEVICE,
        "确认的那一版的根落在要掉的那块盘上"
    );
    pool.devices = Some(devices);
    AcknowledgedVersionAfterWarmUp {
        pool,
        warm_up_record,
        acknowledged_txg: acknowledged.root.checkpoint_txg,
        acknowledged_counter: acknowledged.record.counter,
    }
}

/// 盘 0 整块掉了之后在幸存盘上冷恢复。
fn recover_on_the_surviving_device(
    history: &mut AcknowledgedVersionAfterWarmUp,
    loss: WholeDeviceLoss,
    policy: JournalPolicy,
) -> RecoveryReport {
    match loss {
        WholeDeviceLoss::EveryReadOfTheLostDeviceFails => {
            let plan = SharedFaultPlan::armed(
                common::geometry(),
                FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(LOST_DEVICE),
                    placement: FaultPlacement::AnyOffset,
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
                },
            );
            let devices: Vec<_> = history
                .pool
                .reopen_cold()
                .into_iter()
                .map(|(identity, device)| {
                    (
                        identity,
                        FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                    )
                })
                .collect();
            let report = recover(&devices, policy);
            assert!(plan.fired_count() > 0, "盘 0 的读真被拒过");
            report
        }
        WholeDeviceLoss::LostDeviceReplacedByANeverWrittenDevice => {
            let mut survivor = history.pool.memory_pool();
            survivor
                .devices
                .insert(LOST_DEVICE, SparseDevice::default());
            recover(&survivor, policy)
        }
    }
}

fn unit_bytes_of_class(unit_class: u8) -> usize {
    let bytes = match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
        other => panic!("点名项的单元类不认识：{other}"),
    };
    usize::try_from(bytes).expect("单元宽装得进 usize")
}

/// C519 的已知丢失（今天的结局）：暖机覆盖两块盘之后确认的那一版（txg 6），在它的根所在的盘整块掉了之后丢了。
/// 断言：择到 (2, 4)、水位之上两条（jsn 5、6）；施加第一条（jsn 5，暖机那条）时点名单元验证失败一次、一条都不施加，
/// 读回 (2, 4) 那一版（第一个文件的内容）；没有错误成员。同一个幸存盘面上关掉点名验证就施加两条、读回 txg 6——
/// 挡住它的只有「一个单元两条位置条目都要验过」这一步。jsn 5 的每个点名单元恰有一条位置条目在掉了的盘上，
/// 幸存盘上那一份读得出、校验和对得上：「任一份验过」会放它过去。
/// 判别力自证：把 `replay_journal` 里位置条目的 `all` 改成 `any`，这条用例红（`crates/mutations.tsv`）。
#[test]
fn c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version(
) {
    let mut history = build_acknowledged_version_after_warm_up("c519-known-loss");
    assert_eq!(
        (history.acknowledged_txg, history.acknowledged_counter),
        (CheckpointTxg(6), 6)
    );
    let full_image: MemoryPool = history.pool.memory_pool();
    assert!(
        !history.warm_up_record.named.is_empty(),
        "暖机那条记录点名了重写的固定点单元"
    );
    for named in &history.warm_up_record.named {
        let devices_of_locations: Vec<DeviceIdentity> = named
            .locations
            .iter()
            .map(|location| location.device)
            .collect();
        assert_eq!(
            devices_of_locations,
            vec![LOST_DEVICE, SURVIVING_DEVICE],
            "每个点名单元两条位置条目，各在一块盘上"
        );
        let surviving_location = named
            .locations
            .iter()
            .find(|location| location.device == SURVIVING_DEVICE)
            .expect("幸存盘上那一条");
        let surviving_copy = full_image
            .read(
                SURVIVING_DEVICE,
                surviving_location.slot.to_device_offset(),
                unit_bytes_of_class(named.unit_class),
            )
            .expect("幸存盘上读得到");
        assert_eq!(
            crc32_castagnoli(&surviving_copy),
            surviving_location.unit_checksum,
            "幸存盘上那一份自己验得过"
        );
    }

    for loss in BOTH_LOSS_FORMS {
        let report = recover_on_the_surviving_device(&mut history, loss, JournalPolicy::Consult);
        assert_eq!(
            report.outcome,
            RecoveryOutcome::FileRead {
                root: (InstanceGeneration(2), CheckpointTxg(4)),
                content: file_content()
            },
            "{loss:?}：今天读回的是 (2, 4) 那一版（写行那次照抄的第一个文件），不是确认过的 txg 6"
        );
        assert_eq!(
            report.effective_root,
            Some((InstanceGeneration(2), CheckpointTxg(4))),
            "{loss:?}：一条记录都没施加"
        );
        assert_eq!(
            (
                report.journal.valid_records,
                report.journal.above_water,
                report.journal.verification_passed,
                report.journal.verification_failed,
                report.journal.prefix_applied
            ),
            (6, 2, 0, 1, 0),
            "{loss:?}：jsn 1–6 两份镜像在幸存盘上都在；水位之上 jsn 5、6；施加 jsn 5 之前验点名单元失败一次、前缀停在那里"
        );

        let without_named_verification = recover_on_the_surviving_device(
            &mut history,
            loss,
            JournalPolicy::ConsultWithoutNamedVerification,
        );
        assert_eq!(
            (
                without_named_verification.outcome,
                without_named_verification.effective_root,
                without_named_verification.journal.prefix_applied
            ),
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(4)),
                    content: acknowledged_version_content()
                },
                Some((InstanceGeneration(2), CheckpointTxg(6))),
                2
            ),
            "{loss:?}：同一个盘面上关掉点名验证就读回 txg 6——挡住它的只有点名单元那一步"
        );
    }
}

/// D16（发布语义） 已定项 8 期望的结局：暖机覆盖两块盘之后确认的那一版，在一块盘整块掉了之后还在。
/// 今天红（见上一条）；「点名单元两份都验」还是「任一份验过」由用户定，定成后者之后这一条转绿、上一条转红。
#[test]
#[ignore = "C519：点名单元两份都验还是任一份验过，待用户定"]
fn c519_after_the_warm_up_covers_both_devices_the_acknowledged_version_survives_losing_the_device_holding_its_root(
) {
    let mut history = build_acknowledged_version_after_warm_up("c519-expected");
    for loss in BOTH_LOSS_FORMS {
        let report = recover_on_the_surviving_device(&mut history, loss, JournalPolicy::Consult);
        assert_eq!(
            (report.outcome, report.effective_root),
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(4)),
                    content: acknowledged_version_content()
                },
                Some((InstanceGeneration(2), CheckpointTxg(6)))
            ),
            "{loss:?}：确认过的 txg 6 那一版要读得回来"
        );
    }
}
