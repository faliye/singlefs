//! 攻方模型（m3-close-rootring-r3，云端攻方腿，草稿目录里的仓副本，攻补丁前的 `crates/`）：C751 的收窄形态
//! 「所选根的实例表读不出时，只在见证到的那一版与所选根同实例时，才把同实例的根当作不被抛弃」。
//! 盘面两组：
//! - 第二轮攻方那三形（384 槽小盘、S = 8，造法照抄 `research/prompts/m3-close-rootring-r2-opus-model/tests/rootring_r2_attack_same_instance_rule.rs`）：
//!   `cross`（见证到的那一版属更旧实例）、`registered`（C751 登记那一形）、`pinned656`（入库用例 `:656` 那一形）。
//! - `sweep`（这一轮新造，环洞池、S = 4）：实例 2 覆盖写到 txg 19 → 实例 3 可写挂载（写行、暖机、末尾清扫发布重写实例表）→ 实例 3 接着覆盖写 n 次；
//!   崩在这串里第 j 条根的根槽之后、它那次轮换之前；读的那一刻所选根的实例表读回全零（`fault=table`），或再加每块盘较新的系统配置槽读回全零
//!   （`fault=table+cfg`，与 C751 登记那一形同一个故障预算：一个单元两份、每块盘较新的系统配置槽、一次断电）。
//! 每格打今天三处的真结局、core 的 F2 交回哪一条、照抄判法三种规则各交回哪一条，与三处在每种规则下的结局（触发照今天，每处看自己那一样读者）。
mod common;
mod common_images_for_the_checker_tier;
mod common_root_ring_holes;
mod common_unwitnessed_publish;
mod rootring_r3_common;
#[path = "common_images_for_the_checker_tier/instance_table_sweep_at_the_end_of_a_writable_mount.rs"]
mod scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount;

use common_root_ring_holes::{
    content_of_the_version, fixed_geometry_of_the_ring_hole_pools, parameters_of_the_ring_hole_pools,
};
use common_unwitnessed_publish::{
    devices_reading_back_all_zero_on, fixed_geometry_of_the_384_slot_small_devices,
    record_one_publish_after_overwrites, recorded_devices_reading_back_all_zero_on,
    PublishRecordedAsR, RangesReadingBackAllZero, RecordedPublish,
};
use rootring_r3_common::{
    core_fall_back_to, describe_three_under, fall_back_to, key_of,
    newer_system_configuration_slot_of_each_device, witnessed_publish_of, RuleWhenTheTableIsUnreadable,
    RULES,
};
use scenarios_of_instance_table_sweep_at_the_end_of_a_writable_mount::{
    pool_with_a_dead_instance, publish_an_overwrite, sweeping_at_the_end_of_the_mount_on_any_row,
    sweeping_mount_on,
};
use singlefs_core::address::InstanceGeneration;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::instance_table_sweep::{InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable, mount_writable_with_test_only_switches, ShadowLedger};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::recovery::{choose_root, choose_system_configuration, recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn root_write_indexes(writes: &[RetainedWrite]) -> Vec<usize> {
    writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect()
}

fn instance_table_ranges_of(root: &RootRecord) -> Vec<(singlefs_core::address::DeviceIdentity, u64, u64)> {
    root.instance_table
        .locations
        .iter()
        .map(|location| (location.device, location.slot.to_device_offset().0, 32768))
        .collect()
}

fn describe_key(key: Option<(u32, u64)>) -> String {
    key.map_or_else(|| "None".to_string(), |(instance, txg)| format!("({instance},{txg})"))
}

/// 一格打一行 `R3NARROW`。交回：照抄判法取「今天」时与 core 是否相同，三种规则下恢复各是 Ok 还是 Fail。
fn judge(
    label: &str,
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> (bool, [bool; 3]) {
    let devices = devices_reading_back_all_zero_on(image, ranges);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let chosen = choose_root(&devices, &configuration).expect("择得到根");
    let table_readable =
        singlefs_core::recovery::instance_table_chain_of_root(&devices, &chosen).is_ok();
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    };
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => format!("Ok({},{})", mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0),
        Err(_) => "Fail".to_string(),
    };
    let mut devices_of_the_mount = devices_reading_back_all_zero_on(image, ranges);
    let writable = match mount_writable(parameters, &mut devices_of_the_mount) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    let core = core_fall_back_to(&devices, &chosen);
    let witnessed = witnessed_publish_of(&devices)
        .map_or_else(|| "None".to_string(), |(txg, instance)| format!("({instance},{txg})"));
    let mut line = format!(
        "R3NARROW {label} chosen={} table_readable={table_readable} witnessed={witnessed} today rec={recovery} ro={read_only} rw={writable} | f2_core={}",
        describe_key(Some(key_of(&chosen))),
        describe_key(core.as_ref().map(key_of)),
    );
    let mut replica_matches_core = true;
    let mut recovery_ok = [false; 3];
    for (index, (rule, name)) in RULES.iter().enumerate() {
        let target = fall_back_to(&devices, &chosen, *rule);
        if *rule == RuleWhenTheTableIsUnreadable::ExactlyTheWitnessedVersion {
            replica_matches_core = target.as_ref().map(key_of) == core.as_ref().map(key_of);
        }
        let shares = target.as_ref().map_or("-".to_string(), |root| {
            (root.instance_table.locations == chosen.instance_table.locations).to_string()
        });
        let three = describe_three_under(&devices, &chosen, *rule);
        recovery_ok[index] = three.starts_with("rec=Ok");
        line.push_str(&format!(
            " | {name}: f2={} shares_chosen_table={shares} {three}",
            describe_key(target.as_ref().map(key_of))
        ));
    }
    line.push_str(&format!(" | replica_exact_matches_core={replica_matches_core}"));
    println!("{line}");
    (replica_matches_core, recovery_ok)
}

/// 在 `image` 上做一次可写挂载（`ranges` 那几段读回全零），录下这次挂载的全部写（照抄第二轮攻方模型）。
fn writes_of_a_writable_mount(
    recorded: &RecordedPublish,
    image: &MemoryPool,
    ranges: &RangesReadingBackAllZero,
) -> Vec<RetainedWrite> {
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices_reading_back_all_zero_on(image, ranges, &stream);
    mount_writable_with_test_only_switches(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
    )
    .expect("这一次可写挂载做得成");
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    writes
}

/// `:656` 那一形（照抄第二轮攻方模型的造法）。
fn pinned_656_image(recorded: &RecordedPublish) -> (MemoryPool, RangesReadingBackAllZero) {
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let at_the_mount = RangesReadingBackAllZero::default()
        .with_the_writes(&[recorded.root_slot_write()])
        .with_the_writes(&recorded.unit_writes());
    let writes = writes_of_a_writable_mount(recorded, &crash_image, &at_the_mount);
    let row_root = root_write_indexes(&writes)[0];
    let mut image = crash_image;
    image.apply_writes(&writes[..=row_root]);
    let row_units: Vec<&RetainedWrite> = writes[..row_root]
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    (image, RangesReadingBackAllZero::default().with_the_writes(&row_units))
}

/// 一份崩溃镜像上两种故障各判一格。
fn judge_both_faults(
    label: &str,
    parameters: &MakeFilesystemParameters,
    image: &MemoryPool,
    root_of_the_crash: &RootRecord,
    tally: &mut Tally,
) {
    let table_only = RangesReadingBackAllZero(instance_table_ranges_of(root_of_the_crash));
    let mut with_configuration = table_only.0.clone();
    with_configuration.extend(newer_system_configuration_slot_of_each_device(image));
    for (fault, ranges) in [
        ("table", table_only),
        ("table+cfg", RangesReadingBackAllZero(with_configuration)),
    ] {
        tally.record(judge(&format!("{label} fault={fault}"), parameters, image, &ranges));
    }
}

#[derive(Default)]
struct Tally {
    cells: u32,
    replica_mismatches: u32,
    /// 恢复今天（照抄判法取「今天」）读得出、在收窄形态下读不出的格数。
    recovery_ok_today_fails_narrowed: u32,
    /// 恢复今天读不出、在收窄形态下读得出的格数。
    recovery_fails_today_ok_narrowed: u32,
}

impl Tally {
    fn record(&mut self, (matches, recovery_ok): (bool, [bool; 3])) {
        self.cells += 1;
        self.replica_mismatches += u32::from(!matches);
        self.recovery_ok_today_fails_narrowed += u32::from(recovery_ok[0] && !recovery_ok[2]);
        self.recovery_fails_today_ok_narrowed += u32::from(!recovery_ok[0] && recovery_ok[2]);
    }
}

#[test]
fn the_narrowed_rule_on_the_three_shapes_of_the_second_round() {
    let mut tally = Tally::default();
    for (kind_label, kind) in [
        ("ow", PublishRecordedAsR::Overwrite),
        ("sw3", PublishRecordedAsR::SequentialWriteOfThreeDataUnits),
    ] {
        for overwrites in [3u64, 5] {
            let recorded = record_one_publish_after_overwrites(overwrites, kind);
            let parameters = recorded.pool.parameters.clone();
            let fsid = parameters.filesystem_identifier;
            let confirmed = recorded.image_with_the_whole_publish_persisted();
            let writes = writes_of_a_writable_mount(&recorded, &confirmed, &RangesReadingBackAllZero::default());
            for (ordinal, root_index) in root_write_indexes(&writes).into_iter().enumerate() {
                let mut image = confirmed.clone();
                image.apply_writes(&writes[..=root_index]);
                let root_of_the_crash =
                    RootRecord::parse_slot(writes[root_index].bytes().expect("根槽写带字节"), &fsid)
                        .expect("这次挂载写的根自证");
                judge_both_faults(
                    &format!("shape=cross kind={kind_label} k={overwrites} crash_after_mount_root#{}", ordinal + 1),
                    &parameters,
                    &image,
                    &root_of_the_crash,
                    &mut tally,
                );
            }
            let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let mut registered = newer_system_configuration_slot_of_each_device(&crash_image);
            registered.extend(instance_table_ranges_of(&recorded.root()));
            let registered = RangesReadingBackAllZero(registered).with_the_writes(&recorded.unit_writes());
            tally.record(judge(
                &format!("shape=registered kind={kind_label} k={overwrites}"),
                &parameters,
                &crash_image,
                &registered,
            ));
            if kind == PublishRecordedAsR::Overwrite {
                let (image, ranges) = pinned_656_image(&recorded);
                tally.record(judge(
                    &format!("shape=pinned656 kind={kind_label} k={overwrites}"),
                    &parameters,
                    &image,
                    &ranges,
                ));
            }
        }
    }
    println!(
        "R3NARROW_SUMMARY group=second_round_shapes cells={} replica_mismatches={} recovery_ok_today_fails_narrowed={} recovery_fails_today_ok_narrowed={}",
        tally.cells, tally.replica_mismatches, tally.recovery_ok_today_fails_narrowed, tally.recovery_fails_today_ok_narrowed
    );
    assert_eq!(tally.replica_mismatches, 0, "照抄的 F2 判法取「今天」时要与 core 逐格相同");
}

/// `sweep` 那一组：实例 3 的可写挂载末尾清扫发布重写了实例表，之后再覆盖写 `OVERWRITES_AFTER_THE_SWEEP` 次；崩在这串里每一条根的根槽之后。
const OVERWRITES_AFTER_THE_SWEEP: u64 = 3;

#[test]
fn the_narrowed_rule_when_a_sweep_publish_rewrote_the_instance_table_inside_the_chosen_instance() {
    let parameters = parameters_of_the_ring_hole_pools();
    let fsid = parameters.filesystem_identifier;
    let pool = pool_with_a_dead_instance(None);
    let (mounted, mut devices, stream) =
        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
    let swept = matches!(
        mounted.output.instance_table_sweep,
        InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted { .. })
    );
    println!("R3NARROW_SWEEP_PRECONDITION instance={} rows_deleted={swept}", mounted.output.instance.0);
    assert!(swept, "前提：实例 3 那次挂载末尾的清扫发布删了行（重写了实例表）");
    let instance = mounted.output.instance;
    assert_eq!(instance, InstanceGeneration(3));
    let mut allocator = mounted.allocator;
    let mut current = mounted.current.into_file_version().expect("清扫之后那一版带文件");
    for _ in 0..OVERWRITES_AFTER_THE_SWEEP {
        let txg = current.root.checkpoint_txg.0 + 1;
        current = publish_an_overwrite(&mut devices, &mut allocator, &current, &content_of_the_version(txg), instance);
    }
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &fixed_geometry_of_the_ring_hole_pools(),
    );
    let mut tally = Tally::default();
    let mut previous_table = None;
    for root_index in root_write_indexes(&writes) {
        let mut image = pool.image.clone();
        image.apply_writes(&writes[..=root_index]);
        let root_of_the_crash =
            RootRecord::parse_slot(writes[root_index].bytes().expect("根槽写带字节"), &fsid)
                .expect("这串里写的根自证");
        let table = root_of_the_crash.instance_table.locations.clone();
        let rewrote_the_table = previous_table.as_ref() != Some(&table);
        previous_table = Some(table);
        judge_both_faults(
            &format!(
                "shape=sweep crash_after_root=({},{}) this_publish_rewrote_the_instance_table={rewrote_the_table}",
                root_of_the_crash.instance.0, root_of_the_crash.checkpoint_txg.0
            ),
            &parameters,
            &image,
            &root_of_the_crash,
            &mut tally,
        );
    }
    println!(
        "R3NARROW_SUMMARY group=sweep cells={} replica_mismatches={} recovery_ok_today_fails_narrowed={} recovery_fails_today_ok_narrowed={}",
        tally.cells, tally.replica_mismatches, tally.recovery_ok_today_fails_narrowed, tally.recovery_fails_today_ok_narrowed
    );
    assert_eq!(tally.replica_mismatches, 0, "照抄的 F2 判法取「今天」时要与 core 逐格相同");
}
