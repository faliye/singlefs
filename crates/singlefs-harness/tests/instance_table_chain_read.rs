//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 38 行（实例表第二片）的读路径那一半：一张多于一片的实例表，读者沿链读、
//! 准入沿链核、checker 沿链判（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录
//! `kind 1 | 有无下一片 1 | 位置指针 86` 指着第 k + 1 片，身份四元组 (0, 4, 片序号, 0)）。
//!
//! 这里的两片链是用例手搭的（真写者写出的多片链在 `instance_table_second_page_write.rs`）：
//! mkfs 之后连着可写挂载几次、不写文件（树表 0 条的那一版上每次写行），把最新那一版的实例表拆成两片——第 0 片原地重写、
//! 第二片写进一个远离开放段的空槽，指着第 0 片的每一条根补上它的新整单元校验和。第二片的出生序号随手取第 0 片的加一
//! （读者不看它；实例表单元按 I-1.2（块头写序已发布） 那一行的例外只免谓词那一半，checker 只判链上每一片的头与指着它的指针记的相等），
//! 它也不进那一版的分配记录——树表 0 条的那一版没有记账树，I-3.1（已分配统计对得上） 不适用，手搭的第二片不进分配记录不会让它红；
//! 可写挂载要把整条链换下，这一片不在账里，取号之前的准入就拒（`writable_mount_follows_the_chain_…`）。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/instance_table_chain_read.rs"]
mod scenarios_of_instance_table_chain_read;

use common::{disk_snapshot, memory_pool_of_sparse_devices, parameters, DiskSnapshot};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_instance_table_chain_read::{
    chain_record_to, newest_root, pool_with_a_valid_two_page_chain,
    pool_without_file_after_writable_mounts, read_unit, rows_of_the_newest_table,
    write_two_page_chain, write_unit, Devices, DISKS, SECOND_PAGE_SLOT,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::instance_table::{InstanceRow, InstanceTablePageIndex, InstanceTableRecords};
use singlefs_core::mount::{mount_writable, MountError, Mounted};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_chain_of_root, instance_table_of_root,
    readable_roots, recover, root_is_abandoned_by_the_instance_table,
    verified_system_configuration_slots, JournalPolicy, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::transaction::{PublishError, TransactionUnit};
use singlefs_harness::SharedStream;

/// 读者沿链读：整张表是两片的行按链上的次序接起来，指针两条（第 0 片是根记录里那一条、第 1 片是第 0 片链指针记录里那一条）；
/// 只拿第 0 片的字节解整张表要交 `None`（交回前半张会被当成整张表判）。
#[test]
fn the_reader_follows_the_chain_into_the_second_page_and_a_single_page_parse_refuses_a_chained_first_page(
) {
    let (devices, _stream, rows, chain) = pool_with_a_valid_two_page_chain();
    let newest = newest_root(&devices);
    let read = instance_table_chain_of_root(&devices, &newest).expect("两片都读得出");
    assert_eq!(read.records.rows, rows, "三行都在，按链上的次序");
    assert_eq!(
        read.page_pointers,
        vec![newest.instance_table, chain.second_page_pointer],
        "第 0 片是根记录里那一条，第 1 片是第 0 片链指针记录里那一条"
    );
    assert_eq!(
        instance_table_of_root(&devices, &newest)
            .expect("整张表读得出")
            .rows,
        rows
    );
    assert_eq!(
        InstanceTableRecords::parse(&read_unit(&devices, chain.first_page_slot)),
        None,
        "第 0 片的链指针说还有下一片：只拿这一片的字节解不出整张表"
    );
}

/// 冷启动（只读那一路，`recovery::recover`）沿链读：两片都读得出时照常走完（树表 0 条 ⇒ 没有文件）；
/// 第二片两份都坏了，整张表就不可读，走读停在第二片（与第 0 片读不出同一个结局）。
#[test]
fn cold_start_walks_every_page_of_the_instance_table_and_an_unreadable_second_page_stops_it() {
    let (mut devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
    let newest = newest_root(&devices);
    let report_with_both_pages_readable = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report_with_both_pages_readable.outcome,
        RecoveryOutcome::NoFile {
            root: (newest.instance, newest.checkpoint_txg)
        },
        "两片都读得出：照常走完"
    );

    let mut second_page = read_unit(&devices, SECOND_PAGE_SLOT);
    second_page[20_000] ^= 0xff;
    write_unit(&mut devices, SECOND_PAGE_SLOT, &second_page);
    let report_with_the_second_page_unreadable = recover(&devices, JournalPolicy::Consult);
    assert_eq!(
        report_with_the_second_page_unreadable.outcome,
        RecoveryOutcome::Failed {
            root: Some((newest.instance, newest.checkpoint_txg)),
            failure: RecoveryFailure::UnitUnreadable {
                slot: SECOND_PAGE_SLOT
            },
        },
        "第二片两份的校验和都对不上：表不可读"
    );
}

/// 两块盘四个系统配置槽里自证过的那些槽写着的实例代号，按盘排。
fn system_configuration_instances(
    devices: &Devices,
) -> Vec<(DeviceIdentity, Vec<InstanceGeneration>)> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    DISKS
        .iter()
        .map(|device| {
            let mut instances: Vec<InstanceGeneration> = verified_system_configuration_slots(
                devices,
                *device,
                spacing,
                &parameters().filesystem_identifier,
            )
            .iter()
            .map(|system_configuration| system_configuration.quantities.journal_instance)
            .collect();
            instances.sort();
            (*device, instances)
        })
        .collect()
}

fn snapshot(devices: &Devices, stream: &SharedStream) -> DiskSnapshot {
    disk_snapshot(&memory_pool_of_sparse_devices(devices), stream)
}

/// 可写挂载沿链读整张表、要把整条链换下：写行那次发布逐片释放旧链，取号之前的准入先按释放判定路径逐片核
/// （`transaction::instance_table_chain_to_release`）。手搭的第二片不在这一版的分配记录里 ⇒ 在取号之前拒、点名第二片
/// （只读第 0 片的写法看不到它，取号写完才在发布路径里撞上，号就烧了），盘上逐字节不变、系统配置里的实例代号不变。
#[test]
fn writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition(
) {
    let (mut devices, stream, _rows, chain) = pool_with_a_valid_two_page_chain();
    let before = snapshot(&devices, &stream);
    let instances_before = system_configuration_instances(&devices);
    let refused: Result<Mounted, MountError> = mount_writable(&parameters(), &mut devices);
    match refused {
        Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
            instance_to_acquire,
            cause: PublishError::ReleaseTargetNotAllocated { unit, device, slot },
        }) => {
            assert_eq!(
                (instance_to_acquire, unit, device, slot),
                (
                    InstanceGeneration(5),
                    TransactionUnit::InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)),
                    DeviceIdentity(0),
                    chain.second_page_pointer.locations[0].slot
                ),
                "要取的号；沿链走到的第二片不在这一版的分配记录里"
            );
        }
        other => panic!(
            "第二片不在账里：要在取号之前拒绝：{:?}",
            other.map(|_| "挂上了")
        ),
    }
    assert_eq!(
        snapshot(&devices, &stream),
        before,
        "盘上逐字节不变：系统配置槽、根环里的根、录制流步数（一个写、一道屏障都没发）"
    );
    assert_eq!(
        system_configuration_instances(&devices),
        instances_before,
        "系统配置里的实例代号不变：号没烧"
    );
}

/// 回退候选集与影子账按实例表判「被抛弃」读的是整张表：实例 3 那一行只在第二片上、它的 T 比实例 3 最后那条根的 txg 小一，
/// 那条根就在被抛弃的时间线上（只读第 0 片就看不到这一行，它照样有效）。管理员回退改成挂着时的向前发布之后，回退入口要一版带文件的
/// 现行版本，这张树表 0 条的池上调不到它；判定照同一条路走（`recovery::instance_table_of_root` 读整条链，
/// `recovery::root_is_abandoned_by_the_instance_table` 按行判），回退入口那一侧的排除由步 4 的候选集用例钉。
#[test]
fn abandonment_by_the_instance_table_reads_the_row_that_lives_on_the_second_page() {
    let (mut devices, _stream) = pool_without_file_after_writable_mounts();
    let rows = rows_of_the_newest_table(&devices);
    let system_configuration = choose_system_configuration(&devices).expect("系统配置");
    let last_root_of_instance_three = readable_roots(
        &devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.instance == InstanceGeneration(3))
    .max_by_key(|root| root.checkpoint_txg)
    .expect("实例 3 发布过根");
    assert_eq!(
        rows[2].selected_root_txg, last_root_of_instance_three.checkpoint_txg,
        "写行时实例 3 那一行的 T 就是它最后那条根的 txg"
    );
    let table_before =
        instance_table_of_root(&devices, &newest_root(&devices)).expect("一片的表读得出");
    assert!(
        !root_is_abandoned_by_the_instance_table(&last_root_of_instance_three, &table_before),
        "拆片之前：实例 3 那一行的 T 就是它，不被抛弃"
    );
    let row_that_abandons_the_last_root = InstanceRow {
        instance: InstanceGeneration(3),
        selected_root_txg: CheckpointTxg(last_root_of_instance_three.checkpoint_txg.0 - 1),
        applied_transaction_high_water: 0,
    };
    write_two_page_chain(
        &mut devices,
        &rows[..2],
        chain_record_to,
        &[row_that_abandons_the_last_root],
    );
    let table =
        instance_table_of_root(&devices, &newest_root(&devices)).expect("两片的表沿链读得出");
    assert_eq!(
        table.rows.last(),
        Some(&row_that_abandons_the_last_root),
        "第二片上那一行读进来了"
    );
    assert!(
        root_is_abandoned_by_the_instance_table(&last_root_of_instance_three, &table),
        "第二片上那一行把实例 3 最后那条根判进被抛弃的时间线"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_chain_read::the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds() {
    scenarios_of_instance_table_chain_read::the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_chain_read::chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant() {
    scenarios_of_instance_table_chain_read::chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_chain_read::chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant() {
    scenarios_of_instance_table_chain_read::chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_chain_read::the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant() {
    scenarios_of_instance_table_chain_read::the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_instance_table_chain_read::contradictory_chain_record_reddens_only_the_instance_table_invariant`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn contradictory_chain_record_reddens_only_the_instance_table_invariant() {
    scenarios_of_instance_table_chain_read::contradictory_chain_record_reddens_only_the_instance_table_invariant(&mut PoolCheckerImages::left_to_the_checker_tier());
}
