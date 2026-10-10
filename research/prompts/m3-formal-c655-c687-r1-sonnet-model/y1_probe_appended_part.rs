// ===================== Y1 probe（正推腿副本，不入库）=====================
use singlefs_core::records::TreeTableEntry as ProbeTreeTableEntry;
use singlefs_core::recovery::{verified_system_configuration_slots as probe_slots, PoolReader as ProbePoolReader};
use singlefs_core::root_record::RootRecord as ProbeRootRecord;
use singlefs_core::unit::parse_index_node as probe_parse_index_node;

fn probe_tree_table_entries(pool: &MemoryPool, root: &ProbeRootRecord) -> Vec<ProbeTreeTableEntry> {
    let loc = root.tree_table.locations[0];
    let bytes = pool
        .read(loc.device, loc.slot.to_device_offset(), 16384)
        .expect("读树表");
    let node = probe_parse_index_node(&bytes).expect("解树表");
    node.entries
        .iter()
        .map(|entry| ProbeTreeTableEntry::parse(entry).expect("解条目"))
        .collect()
}

fn probe_describe(label: &str, pool: &MemoryPool, compared: ComparedVersion) -> (Option<ProbeRootRecord>, Vec<ProbeTreeTableEntry>, Vec<(u32, u64, u64, u32, u64)>) {
    let fields = read_recovery_fields(pool, compared);
    let params = SMALL.parameters();
    let spacing = u64::from(params.geometry.fixed_structure_slot_spacing);
    let mut slots = Vec::new();
    for device in pool.device_identities() {
        for s in probe_slots(pool, device, spacing, &params.filesystem_identifier) {
            slots.push((device.0, s.quantities.slot_generation, s.quantities.journal_tail, s.quantities.journal_instance.0, s.quantities.rollback_floor.0));
        }
    }
    let version = fields.compared_version.as_ref();
    let root = version.map(|v| v.root);
    let entries = root.as_ref().map(|r| probe_tree_table_entries(pool, r)).unwrap_or_default();
    println!("[{label}] root={:?}", root.as_ref().map(|r| (r.instance.0, r.checkpoint_txg.0, r.tree_identifier_watermark, r.rollback_floor.0, r.unmount_marker)));
    println!("[{label}] mapping_root_zero={} alloc_root_zero={}", root.as_ref().map(|r| r.mapping_root == singlefs_core::pointer::NodePointer::empty_root()).unwrap_or(false), root.as_ref().map(|r| r.allocation_record_tree_root == singlefs_core::pointer::NodePointer::empty_root()).unwrap_or(false));
    println!("[{label}] sysconf(dev,gen,tail,inst,floor)={slots:?}");
    println!("[{label}] tree_table_entry_count={}", entries.len());
    (root, entries, slots)
}

fn probe_compare(name: &str, before: &MemoryPool, after: &MemoryPool, after_compared: ComparedVersion) {
    println!("===== {name} =====");
    let (r1, e1, s1) = probe_describe("R1", before, ComparedVersion::TheRecoveredVersion);
    let (r2, e2, s2) = probe_describe("R2", after, after_compared);
    if let (Some(a), Some(b)) = (&r1, &r2) {
        println!("[cmp] mapping_root_equal={} alloc_root_equal={} tree_table_ptr_equal={} instance_table_ptr_equal={} fsid_equal={}", a.mapping_root == b.mapping_root, a.allocation_record_tree_root == b.allocation_record_tree_root, a.tree_table == b.tree_table, a.instance_table == b.instance_table, a.filesystem_identifier == b.filesystem_identifier);
    }
    for (x, y) in e1.iter().zip(e2.iter()) {
        println!("[cmp] entry kind={} tree={} root_equal={} root_zero_before={} root_zero_after={} birth_txg {}->{} head {}->{}", x.kind, x.tree.0, x.root == y.root, x.root == singlefs_core::pointer::NodePointer::empty_root(), y.root == singlefs_core::pointer::NodePointer::empty_root(), x.birth_txg.0, y.birth_txg.0, x.head_identifier, y.head_identifier);
    }
    println!("[cmp] entries before={} after={}", e1.len(), e2.len());
    println!("[cmp] sysconf before={s1:?}");
    println!("[cmp] sysconf after ={s2:?}");
}

fn probe_mount_alone(name: &str, width: HistoryDeviceWidth, crash_image: &MemoryPool) -> MemoryPool {
    let parameters = width.parameters();
    let stream = SharedStream::new();
    let mut devices = devices_of(crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    drop(mounted);
    let after = pool_of(devices, crash_image.device_size_in_bytes);
    probe_compare(&format!("{name} / mount alone"), crash_image, &after, ComparedVersion::TheRecoveredVersion);
    after
}

#[test]
fn y1_probe_after_make_filesystem() {
    let p = pool_after_make_filesystem();
    probe_mount_alone("after_mkfs", SMALL, &p);
}

#[test]
fn y1_probe_after_first_file() {
    let p = pool_after_the_first_file();
    probe_mount_alone("after_first_file", SMALL, &p);
    let j = mount_and_one_publish_on(SMALL, &p);
    let _ = j;
}

#[test]
fn y1_probe_left_by_unmount() {
    let p = pool_left_by_an_unmount();
    probe_mount_alone("left_by_unmount", SMALL, &p);
}

#[test]
fn y1_probe_after_mkfs_second_mount() {
    // 两次可写挂载：第二次挂载写的行非空（树表 0 条那一版 + 要写的行非空）
    let p = pool_after_make_filesystem();
    let after1 = probe_mount_alone("after_mkfs_mount1", SMALL, &p);
    probe_mount_alone("after_mkfs_mount2", SMALL, &after1);
}

#[test]
fn y1_probe_unlink_continuation() {
    let p = pool_crashed_right_after_the_unlink_intent();
    probe_mount_alone("unlink_continuation", SMALL, &p);
}

#[test]
fn y1_probe_raise_floor() {
    let p = pool_where_the_mount_raises_the_floor();
    probe_mount_alone("raise_floor", HistoryDeviceWidth::UnitAreaOf256Slots, &p);
}

// ---- 第二批探针：世代号与 tail 的确定关系 ----
use singlefs_core::recovery::{choose_system_configuration as probe_choose_sc, scan_journal as probe_scan_journal};

fn probe_sysconf_relation(name: &str, width: HistoryDeviceWidth, crash_image: &MemoryPool, with_publish: bool) {
    let parameters = width.parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(crash_image, &parameters, &stream);
    let mounted = mount(&parameters, &mut devices);
    if with_publish {
        publish_once_after_the_mount(&parameters, &mut devices, mounted);
    } else {
        drop(mounted);
    }
    let operations = stream.retained_operations();
    let (writes, _segments, _idx) = writes_and_segments_with_stream_indexes(&operations, &width.fixed_geometry());
    let after = pool_of(devices, crash_image.device_size_in_bytes);
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    for device in after.device_identities() {
        let max_gen = |pool: &MemoryPool| probe_slots(pool, device, spacing, &parameters.filesystem_identifier).iter().map(|s| s.quantities.slot_generation).max().unwrap();
        let max_tail = |pool: &MemoryPool| probe_slots(pool, device, spacing, &parameters.filesystem_identifier).iter().map(|s| s.quantities.journal_tail).max().unwrap();
        let n_sys = writes.iter().filter(|w| w.device == device && w.kind == StepKind::SystemConfigurationSlot).count() as u64;
        let sc = probe_choose_sc(&after).expect("择系统配置");
        let records = probe_scan_journal(&after, &sc);
        let inst = sc.quantities.journal_instance;
        let max_counter_of_instance = records.keys().filter(|(i, _)| *i == inst).map(|(_, c)| *c).max();
        let g1 = max_gen(crash_image); let g2 = max_gen(&after);
        println!("[{name}{}] dev={} gen_R1={} gen_R2={} sysconf_writes_to_dev={} => gen_R2==gen_R1+n: {} | tail_R1={} tail_R2={} max_counter_of_instance({})={:?} => tail_R2==that: {}",
            if with_publish {"+publish"} else {""}, device.0, g1, g2, n_sys, g2 == g1 + n_sys,
            max_tail(crash_image), max_tail(&after), inst.0, max_counter_of_instance, Some(max_tail(&after)) == max_counter_of_instance);
    }
}

#[test]
fn y1_probe_sysconf_relations() {
    let p = pool_after_make_filesystem();
    probe_sysconf_relation("after_mkfs", SMALL, &p, false);
    probe_sysconf_relation("after_mkfs", SMALL, &p, true);
    let p = pool_after_the_first_file();
    probe_sysconf_relation("after_first_file", SMALL, &p, false);
    probe_sysconf_relation("after_first_file", SMALL, &p, true);
    let p = pool_left_by_an_unmount();
    probe_sysconf_relation("left_by_unmount", SMALL, &p, false);
    let p = pool_rebuilt_from_a_record_of_an_overwrite();
    probe_sysconf_relation("rebuilt_from_record", SMALL, &p, false);
    let p = pool_where_the_mount_raises_the_floor();
    probe_sysconf_relation("raise_floor", HistoryDeviceWidth::UnitAreaOf256Slots, &p, false);
    let p = pool_crashed_right_after_the_unlink_intent();
    probe_sysconf_relation("unlink_continuation", SMALL, &p, false);
    let p = pool_after_make_filesystem();
    let stream = SharedStream::new();
    let mut devices = devices_of(&p, &SMALL.parameters(), &stream);
    let _ = mount(&SMALL.parameters(), &mut devices);
    let p2 = pool_of(devices, p.device_size_in_bytes);
    probe_sysconf_relation("after_mkfs_mount2", SMALL, &p2, false);
}

#[test]
fn y1_probe_with_trees_and_no_file() {
    // 第一个文件之后 unlink 走完（意图 + 最后一批），再挂载：R1 是有树而文件集合为空的一版
    let p = pool_after_the_history(
        SMALL,
        HistoryStartingPoint::AfterFirstFile,
        vec![HistoryOperation::Unlink],
        SpaceAdmission::JudgedByTheFormula,
    );
    probe_mount_alone("with_trees_and_no_file", SMALL, &p);
    probe_sysconf_relation("with_trees_and_no_file", SMALL, &p, false);
}

// ---- Y3 f 格：开放段回落够不够得着（扫单元区宽度 × 只暖机挂载次数，再发第一个文件）----
use singlefs_checker::image::InvariantVerdict as ProbeVerdict;
use singlefs_checker::walk::check_pool_image as probe_check_pool_image;

fn probe_i55(pool: &MemoryPool) -> (String, Vec<&'static str>) {
    let verdicts = probe_check_pool_image(pool);
    let i55 = verdicts.iter().find(|(id, _)| *id == "I-5.5").map(|(_, v)| match v {
        ProbeVerdict::Holds => "Holds".to_string(),
        ProbeVerdict::Violated(d) => format!("Violated({d})"),
        ProbeVerdict::NotApplicable(r) => format!("NotApplicable({r})"),
        ProbeVerdict::PartiallyJudged(r) => format!("PartiallyJudged({r})"),
    }).unwrap();
    let red: Vec<&'static str> = verdicts.iter().filter(|(_, v)| matches!(v, ProbeVerdict::Violated(_))).map(|(id, _)| *id).collect();
    (i55, red)
}

#[test]
fn y3_probe_open_segment_scan() {
    for width in [HistoryDeviceWidth::UnitAreaOf240Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf384Slots] {
        for k in [1usize, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24, 30] {
            let mut operations = vec![HistoryOperation::CloseAndMountWritable; k];
            operations.push(HistoryOperation::PublishFirstFile(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed: 11 }));
            let stream = SharedStream::retaining_contents();
            let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterMakeFilesystem, operations: operations.clone() };
            let run = execute_history_with(
                &history,
                HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::JudgedByTheFormula },
                &stream,
                &mut |_o| {},
            );
            let done = run.outcomes.len();
            if done != operations.len() {
                println!("[scan] width={width:?} mounts={k}: 历史只跑了 {done}/{} 步，ending={:?}", operations.len(), run.ending);
                continue;
            }
            let mut pool = MemoryPool::with_devices(&DEVICES, width.device_bytes());
            pool.apply(&stream.retained_operations());
            let (i55, red) = probe_i55(&pool);
            println!("[scan] width={width:?} mounts={k}: I-5.5={i55} 红的={red:?}");
        }
    }
}

// ---- Y3 f 格第二批：数回落（FallBackToLowestFreeSlot）一共走到过几次 ----
fn probe_fallback_run(width: HistoryDeviceWidth, admission: SpaceAdmission, mounts: usize, then_first_file: bool) -> (usize, usize, u64, u64) {
    use std::sync::atomic::Ordering;
    singlefs_core::allocator::FALLBACK_COUNT.store(0, Ordering::SeqCst);
    let mut operations = vec![HistoryOperation::CloseAndMountWritable; mounts];
    if then_first_file {
        operations.push(HistoryOperation::PublishFirstFile(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed: 11 }));
    }
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterMakeFilesystem, operations: operations.clone() };
    let run = execute_history_with(
        &history,
        HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: admission },
        &stream,
        &mut |_o| {},
    );
    (run.outcomes.len(), operations.len(), singlefs_core::allocator::FALLBACK_COUNT.load(Ordering::SeqCst), singlefs_core::allocator::FALLBACK_LAST_SLOT.load(Ordering::SeqCst))
}

#[test]
fn y3_probe_fallback_count_with_the_admission() {
    for width in [HistoryDeviceWidth::UnitAreaOf240Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf384Slots] {
        for mounts in [1usize, 2, 3, 4, 6, 8, 10, 12, 16, 20, 24, 30, 40, 60] {
            let (done, total, count, last) = probe_fallback_run(width, SpaceAdmission::JudgedByTheFormula, mounts, true);
            println!("[fb] 准入开 width={width:?} mounts={mounts}: 跑完 {done}/{total} 步，回落 {count} 次，最后一次回落槽 {last}");
        }
    }
}

#[test]
fn y3_probe_fallback_count_without_the_admission() {
    for width in [HistoryDeviceWidth::UnitAreaOf240Slots, HistoryDeviceWidth::UnitAreaOf256Slots, HistoryDeviceWidth::UnitAreaOf384Slots] {
        for mounts in [1usize, 4, 8, 12, 24, 60] {
            let (done, total, count, last) = probe_fallback_run(width, SpaceAdmission::SkippedByTheTestOnlySwitch, mounts, true);
            println!("[fb] 准入关 width={width:?} mounts={mounts}: 跑完 {done}/{total} 步，回落 {count} 次，最后一次回落槽 {last}");
        }
    }
}
