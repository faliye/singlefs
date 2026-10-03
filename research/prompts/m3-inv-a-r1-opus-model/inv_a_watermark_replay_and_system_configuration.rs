//! m3-inv-a-r1 攻方腿的原型用例（只在攻方副本上跑，不入 crates/）。全部用内存稀疏盘，不在磁盘上建镜像文件。
//! 每个用例只打印观测（`INV-A …` 开头的行），断言只钉「造出来的历史真是想造的那一段」，不钉结论。

mod common;

use common::{crash_state_devices, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{check_pool_image, inv_a_take_last_watermark_diagnostics};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, replay_journal, scan_journal, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_sequential_write, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn fresh_devices(stream: &SharedStream) -> Devices {
    let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    crash_state_devices(&base, &[], &[], stream)
}

fn devices_of(image: &MemoryPool, stream: &SharedStream) -> Devices {
    crash_state_devices(image, &[], &[], stream)
}

fn classifier(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn empty_pool() -> MemoryPool {
    MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES)
}

/// 一段录制流里只留单元写与 journal 记录（崩在记录落盘之后、根槽 FUA 之前：根槽与系统配置轮换都没落）。
fn units_and_records_only(parameters: &MakeFilesystemParameters, operations: &[RetainedOperation]) -> Vec<RetainedOperation> {
    let classifier = classifier(parameters);
    operations
        .iter()
        .filter(|retained| {
            matches!(classifier.classify(&retained.operation), StepKind::UnitWrite | StepKind::JournalRecord)
        })
        .cloned()
        .collect()
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts.iter().filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_))).map(|(name, _)| *name).collect()
}

fn verdict_of(verdicts: &[(&'static str, InvariantVerdict)], name: &str) -> InvariantVerdict {
    verdicts.iter().find(|(candidate, _)| *candidate == name).map(|(_, verdict)| verdict.clone()).expect("在清单里")
}

/// 跑池级 checker，打印违例清单、I-7.8 的判定与 I-7.8 那一处的观测；交回违例清单与观测。
fn report(label: &str, image: &MemoryPool) -> (Vec<&'static str>, singlefs_checker::walk::InvAWatermarkDiagnostics) {
    let verdicts = check_pool_image(image);
    let diagnostics = inv_a_take_last_watermark_diagnostics().expect("走到了 I-7.8 那一处");
    let violated = violated(&verdicts);
    let with_records = diagnostics.roots_max.max(diagnostics.records_max.unwrap_or(0)) > diagnostics.highest_seen;
    println!(
        "INV-A {label}: violated={violated:?} I-7.8={:?} roots_max={} records_max={:?} highest_seen={} with_records_I-7.8={} I-7.8b={}",
        verdict_of(&verdicts, "I-7.8"),
        diagnostics.roots_max,
        diagnostics.records_max,
        diagnostics.highest_seen,
        if with_records { "Holds" } else { "Violated" },
        diagnostics.b78_first_violation.as_deref().unwrap_or("Holds"),
    );
    (violated, diagnostics)
}

fn outcome_text(image: &MemoryPool, policy: JournalPolicy) -> String {
    let recovered = recover(image, policy);
    let outcome = match &recovered.outcome {
        RecoveryOutcome::NoFile { root } => format!("NoFile {root:?}"),
        RecoveryOutcome::FileRead { root, content } => format!("FileRead {root:?} {} 字节", content.len()),
        RecoveryOutcome::Failed { root, failure } => format!("Failed {root:?} {failure:?}"),
    };
    format!("{outcome} effective={:?} prefix_applied={} verification_failed={}", recovered.effective_root, recovered.journal.prefix_applied, recovered.journal.verification_failed)
}

/// mkfs → 取号 1 → 暖机 txg 1、2 → 新池新建文件 txg 3（与 `common::build_pool` 同一段，内存盘）。
/// 交回流、mkfs 之后流里几步、暖机之后几步、设备、分配器与新池新建文件的输出。
struct MkfsStream {
    stream: SharedStream,
    warm_up_operation_count: usize,
    devices: Devices,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

fn mkfs_stream() -> MkfsStream {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = fresh_devices(&stream);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(
        Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
        Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
    );
    let content = common::file_content();
    let warm_up_operation_count;
    let output = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let warmed = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        warm_up_operation_count = stream.operations().len();
        publish_first_file(
            &mut pool,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    MkfsStream { stream, warm_up_operation_count, devices, allocator, output }
}

/// A5 W1–W3：新池新建文件（发树 11..18、水位 11 → 19）崩在记录落盘之后、根槽之前——「根槽读不出而记录在」的崩溃镜像；
/// 可写挂载一次（施加那条记录、写行、暖机）；再把那次挂载的录制流逐前缀截断（每个前缀是一个崩溃状态，只取前缀、不取整写子集），
/// 每个前缀镜像上判一次，再在它上面可写挂载一次、判一次。
#[test]
fn a5_record_without_root_then_writable_mount_prefixes() {
    let parameters = parameters();
    let built = mkfs_stream();
    let operations = built.stream.retained_operations();
    let mut image_w1 = empty_pool();
    image_w1.apply(&operations[..built.warm_up_operation_count]);
    image_w1.apply(&units_and_records_only(&parameters, &operations[built.warm_up_operation_count..]));
    assert_eq!(built.output.root.tree_identifier_watermark, 19, "新池新建文件之后水位 19");
    let (w1_violated, w1) = report("W1 新池新建文件记录在、根槽没落", &image_w1);
    println!("INV-A W1 recover(Consult): {}", outcome_text(&image_w1, JournalPolicy::Consult));
    let mount_stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&image_w1, &mount_stream);
    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
    println!(
        "INV-A W2 mount: instance={:?} chosen={:?} effective={:?} prefix_applied={} rows={:?} row_publish_watermark={}",
        mounted.output.instance,
        (mounted.output.chosen_root.instance, mounted.output.chosen_root.checkpoint_txg),
        (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg),
        mounted.output.journal.prefix_applied,
        mounted.output.rows_written,
        mounted.output.row_publish.root().tree_identifier_watermark
    );
    let image_w2 = memory_pool_of_sparse_devices(&devices);
    let (w2_violated, _) = report("W2 可写挂载之后", &image_w2);
    let mount_operations = mount_stream.retained_operations();
    let mut prefix_states = 0;
    let mut prefix_red = Vec::new();
    let mut remount_red = Vec::new();
    let mut remount_refused = Vec::new();
    for cut in 0..=mount_operations.len() {
        let mut image = image_w1.clone();
        image.apply(&mount_operations[..cut]);
        prefix_states += 1;
        let (violated_here, _) = report(&format!("W3 前缀 {cut}/{}", mount_operations.len()), &image);
        if !violated_here.is_empty() {
            prefix_red.push((cut, violated_here));
        }
        let remount_stream = SharedStream::retaining_contents();
        let mut remount_devices = devices_of(&image, &remount_stream);
        match mount_writable(&parameters, &mut remount_devices) {
            Ok(remounted) => {
                let after = memory_pool_of_sparse_devices(&remount_devices);
                let (violated_after, _) = report(
                    &format!("W3 前缀 {cut} 再可写挂载（实例 {:?}，行 {:?}）", remounted.output.instance, remounted.output.rows_written),
                    &after,
                );
                if !violated_after.is_empty() {
                    remount_red.push((cut, violated_after));
                }
            }
            Err(error) => remount_refused.push((cut, format!("{error:?}").chars().take(120).collect::<String>())),
        }
    }
    println!(
        "INV-A W1-W3 汇总: W1 违例 {w1_violated:?}（W1 records_max {:?}），W2 违例 {w2_violated:?}，前缀状态 {prefix_states} 个，前缀上有违例的 {prefix_red:?}，再挂载后有违例的 {remount_red:?}，再挂载被拒的 {} 个：{remount_refused:?}",
        w1.records_max,
        remount_refused.len()
    );
}

fn root_slot_of(txg: CheckpointTxg) -> (u32, u64) {
    let target = singlefs_core::root_ring::target_for_publish(txg, parameters().geometry.root_ring_slots_per_region);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = singlefs_core::root_ring::slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
    (device.0, offset.0)
}

fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
    PoolReader::read(pool, DeviceIdentity(device), DeviceOffsetInBytes(offset), length).expect("读")
}

fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    pool.devices.get_mut(&DeviceIdentity(device)).expect("盘").write(DeviceOffsetInBytes(offset), bytes);
}

/// 根环全部槽（区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096），与 checker_known_bad_images.rs 同一份。
fn root_slots() -> Vec<(u32, u64)> {
    (0..3u64)
        .flat_map(|region| {
            (0..8u64).map(move |slot| ([0u32, 1, 0][usize::try_from(region).expect("区域")], 64 * 16384 + region * 3 * (1 << 20) + slot * 4096))
        })
        .collect()
}

/// 根环里每一条根的树 ID 水位改成 `watermark`、重封（同 checker_known_bad_images.rs 的 `set_the_tree_identifier_watermark_of_every_root`）。
fn set_the_tree_identifier_watermark_of_every_root(image: &mut MemoryPool, watermark: u64) -> usize {
    let mut rewritten = 0;
    for (device, offset) in root_slots() {
        let mut bytes = read(image, device, offset, 512);
        if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
            continue;
        }
        bytes[122..130].copy_from_slice(&watermark.to_le_bytes());
        let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&bytes, 512, 138);
        bytes[138..170].copy_from_slice(&digest);
        write(image, device, offset, &bytes);
        rewritten += 1;
    }
    rewritten
}

/// A5 W4：checker_known_bad_images.rs 的 `published_nodes_behind_an_intermediate_row_still_count_against_the_tree_identifier_watermark`
/// 那段历史照搬到内存盘上（实例 2 发 11..18 之后根槽全清零、记录还在，实例 3 在实例 1 那一版上写行 (1, T, 0)、(2, 0, 0)、暖机，
/// 水位照可写挂载取环里的记录 19）。干净镜像判一次；再把根环水位全压到 11（= 实例 3 只取根环、不并记录时写下的水位）判一次：
/// 只取根的 I-7.8 红不红、「并进记录」那一读法红不红。
#[test]
fn a5_intermediate_row_image_under_both_watermark_readings() {
    use singlefs_core::journal::back_chain_of;
    use singlefs_core::mount::InstanceRow;
    use singlefs_core::recovery::instance_table_chain_of_root;
    use singlefs_core::transaction::{
        publish_instance_table_on_version_without_file, publish_without_units, InstanceTableOnlyPublishPlan,
        InstanceTableRewrite, ZeroUnitPublishPlan,
    };
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let first_instance = mount_writable(&parameters, &mut devices).expect("只做过 mkfs 的池可写挂载");
    let PoolVersion::WithoutFile(version_of_the_first_instance) = &first_instance.current else { panic!("树表 0 条") };
    let newest_root_of_the_first_instance = version_of_the_first_instance.root;
    drop(first_instance);
    let second_instance = mount_writable(&parameters, &mut devices).expect("再挂一次");
    let PoolVersion::WithoutFile(version) = &second_instance.current else { panic!("树表 0 条") };
    let mut allocator = second_instance.allocator.clone();
    let content: Vec<u8> = (0..3300usize).map(|index| u8::try_from((index * 7 + 17) % 253).expect("小于 256")).collect();
    let first_file_of_the_second_instance = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120 },
            second_instance.output.instance,
            &version.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    let mut roots_of_the_second_instance: Vec<CheckpointTxg> = vec![second_instance.output.row_publish.root().checkpoint_txg];
    roots_of_the_second_instance.extend(second_instance.output.warm_up_publishes.iter().map(|warm_up| warm_up.root().checkpoint_txg));
    roots_of_the_second_instance.push(first_file_of_the_second_instance.root.checkpoint_txg);
    for txg in &roots_of_the_second_instance {
        let (device, offset) = root_slot_of(*txg);
        let (_, recorded) = devices.iter_mut().find(|(identity, _)| identity.0 == device).expect("根槽所在的盘");
        recorded.write_at(DeviceOffsetInBytes(offset), &[0u8; 512], WriteDurability::Plain).expect("清零实例 2 的根槽");
    }
    let rows_of_the_third_instance = vec![
        InstanceRow { instance: InstanceGeneration(1), selected_root_txg: newest_root_of_the_first_instance.checkpoint_txg, applied_transaction_high_water: 0 },
        InstanceRow { instance: InstanceGeneration(2), selected_root_txg: CheckpointTxg(0), applied_transaction_high_water: 0 },
    ];
    let newest_root_of_the_third_instance = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let third_instance = acquire_instance(&mut writer).expect("取号 3");
        let mut allocator_rebuilt = PoolAllocator::new(
            [0u32, 1].iter().map(|device| DeviceFreeMap::new(DeviceIdentity(*device), IMAGE_BYTES)).collect(),
        );
        allocator_rebuilt.mark_format_time_units(
            Placement { slot: INSTANCE_TABLE_SLOT, span: 2 },
            Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 },
        );
        let rewrite = InstanceTableRewrite { rows: rows_of_the_third_instance, replaced_chain: vec![newest_root_of_the_first_instance.instance_table] };
        let row_publish = publish_instance_table_on_version_without_file(
            &mut writer,
            &mut allocator_rebuilt,
            &newest_root_of_the_first_instance,
            InstanceTableOnlyPublishPlan {
                txg: CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1),
                counter: first_file_of_the_second_instance.record.counter + 1,
                instance: third_instance,
                back_chain: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: &rewrite,
                tree_identifier_watermark: first_file_of_the_second_instance.root.tree_identifier_watermark,
            },
        )
        .expect("实例 3 写行");
        let mut devices_holding_a_root = std::collections::BTreeSet::from([root_slot_of(row_publish.root.checkpoint_txg).0]);
        let mut newest = row_publish;
        for _ in 0..3 {
            if [0u32, 1].iter().all(|device| devices_holding_a_root.contains(device)) {
                break;
            }
            let warm = publish_without_units(
                &mut writer,
                &newest.root,
                ZeroUnitPublishPlan {
                    txg: CheckpointTxg(newest.root.checkpoint_txg.0 + 1),
                    counter: newest.record.counter + 1,
                    instance: third_instance,
                    back_chain: back_chain_of(&newest.record_bytes),
                    rollback_floor: CheckpointTxg(0),
                    tree_identifier_watermark: newest.root.tree_identifier_watermark,
                },
            )
            .expect("实例 3 暖机");
            devices_holding_a_root.insert(root_slot_of(warm.root.checkpoint_txg).0);
            newest = warm;
        }
        newest.root
    };
    let clean = memory_pool_of_sparse_devices(&devices);
    let rows = instance_table_chain_of_root(&clean, &newest_root_of_the_third_instance).expect("实例表").records.rows;
    println!("INV-A W4 实例 3 最新根的行 {rows:?}；实例 2 发的树 {:?}", first_file_of_the_second_instance.tree_identifiers);
    report("W4 干净镜像（实例 3 水位取记录 19）", &clean);
    let mut lowered = clean.clone();
    let rewritten = set_the_tree_identifier_watermark_of_every_root(&mut lowered, 11);
    report(&format!("W4 根环水位全压到 11（改了 {rewritten} 条根）"), &lowered);
}

/// A5 W5：实例 1（只做过 mkfs 的池可写挂载，树表 0 条）发新池新建文件，崩在记录落盘之后、根槽之前；下一次可写挂载时那次发布的
/// 数据单元读不出（两份清零，不写回），记录点名验证不过、不施加：择根落在实例 1 的暖机根，写行 (1, T, W)、T > 0。
/// 实例 2 在树表 0 条那一版上再发新池新建文件。环境变量 INV_A_CORE_WATERMARK_ROOTS_ONLY 开着时 core 取水位只看根环（C342 退回去），
/// 实例 2 重发 11..18。每一步判一次：只取根的 I-7.8、并进记录的读法、候选 I-7.8b。
#[test]
fn a5_unapplied_first_file_record_behind_a_positive_row() {
    let parameters = parameters();
    let mutant = std::env::var_os("INV_A_CORE_WATERMARK_ROOTS_ONLY").is_some();
    let stream = SharedStream::retaining_contents();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let first_instance = mount_writable(&parameters, &mut devices).expect("实例 1");
    let PoolVersion::WithoutFile(version) = &first_instance.current else { panic!("树表 0 条") };
    let mut allocator = first_instance.allocator.clone();
    let before_the_publish = stream.retained_operations().len();
    let first_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &common::file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            first_instance.output.instance,
            &version.record_bytes,
        )
        .expect("实例 1 新池新建文件")
    };
    let operations = stream.retained_operations();
    let mut image = empty_pool();
    image.apply(&operations[..before_the_publish]);
    image.apply(&units_and_records_only(&parameters, &operations[before_the_publish..]));
    report("W5a 实例 1 新池新建文件记录在、根槽没落", &image);
    for pointer in &first_file.data_pointers {
        for location in pointer.locations {
            write(&mut image, location.device.0, location.slot.to_device_offset().0, &vec![0u8; 32768]);
        }
    }
    let mount_stream = SharedStream::retaining_contents();
    let mut devices = devices_of(&image, &mount_stream);
    let second_instance = mount_writable(&parameters, &mut devices).expect("实例 2 可写挂载");
    println!(
        "INV-A W5 mount(mutant={mutant}): instance={:?} chosen={:?} effective={:?} prefix_applied={} verification_failed={} rows={:?} row_publish_watermark={}",
        second_instance.output.instance,
        (second_instance.output.chosen_root.instance, second_instance.output.chosen_root.checkpoint_txg),
        (second_instance.output.effective_root.instance, second_instance.output.effective_root.checkpoint_txg),
        second_instance.output.journal.prefix_applied,
        second_instance.output.journal.verification_failed,
        second_instance.output.rows_written,
        second_instance.output.row_publish.root().tree_identifier_watermark
    );
    report(&format!("W5b 实例 2 写行与暖机之后（mutant={mutant}）"), &memory_pool_of_sparse_devices(&devices));
    let PoolVersion::WithoutFile(version2) = &second_instance.current else { panic!("择到树表 0 条的那一版") };
    let mut allocator2 = second_instance.allocator.clone();
    let second_first_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator2,
            &version2.root,
            FirstFile { content: &common::file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 },
            second_instance.output.instance,
            &version2.record_bytes,
        )
        .expect("实例 2 新池新建文件")
    };
    println!(
        "INV-A W5 实例 1 那次发的树 {:?}；实例 2 发的树 {:?}（mutant={mutant}）",
        first_file.tree_identifiers, second_first_file.tree_identifiers
    );
    report(&format!("W5c 实例 2 新池新建文件之后（mutant={mutant}）"), &memory_pool_of_sparse_devices(&devices));
}

/// A1 X1：环长 1 MiB（在飞上限 1048576 ÷ 4096 ÷ 3 = 85）的池上两次各 44 个数据单元的顺序写（每单元一条记录）B、C：
/// C 崩在记录落盘之后、根槽之前，B 的根槽读不出（改坏一个字节）。按 core 的 `replay_journal` 从所选根施加一次，再从施加出来的那一版施加一次。
#[test]
fn a1_replay_twice_across_the_in_flight_cap() {
    let mut parameters = parameters();
    parameters.geometry.journal_ring_bytes = 1 << 20;
    let stream = SharedStream::retaining_contents();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters, &mut devices).expect("mkfs（环 1 MiB）");
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 1");
    let PoolVersion::WithoutFile(version) = &mounted.current else { panic!("树表 0 条") };
    let mut allocator = mounted.allocator.clone();
    let instance = mounted.output.instance;
    let units = 44usize;
    let content = |seed: usize| -> Vec<u8> { (0..units * 32768).map(|index| u8::try_from((index * 7 + seed) % 251).expect("小于 256")).collect() };
    let (first, second) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let first = publish_first_file(&mut writer, &mut allocator, &version.root, FirstFile { content: &[1u8; 100], write_time_seconds: FIXED_WRITE_TIME_SECONDS }, instance, &version.record_bytes).expect("A");
        let second = publish_sequential_write(&mut writer, &mut allocator, &first, FirstFile { content: &content(3), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1 }, instance).expect("B");
        (first, second)
    };
    let before_c = stream.retained_operations().len();
    let third = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_sequential_write(&mut writer, &mut allocator, &second, FirstFile { content: &content(5), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 2 }, instance).expect("C")
    };
    let operations = stream.retained_operations();
    let mut image = empty_pool();
    image.apply(&operations[..before_c]);
    image.apply(&units_and_records_only(&parameters, &operations[before_c..]));
    println!(
        "INV-A X1 A=({:?},{:?}) B=({:?},{:?}) 记录 {} 条 C=({:?},{:?}) 记录 {} 条",
        first.root.instance, first.root.checkpoint_txg, second.root.instance, second.root.checkpoint_txg,
        second.earlier_records_of_this_publish.len() + 1, third.root.instance, third.root.checkpoint_txg,
        third.earlier_records_of_this_publish.len() + 1
    );
    println!("INV-A X1 B 的根读得出时 recover(Consult): {}", outcome_text(&image, JournalPolicy::Consult));
    let (device, offset) = root_slot_of(second.root.checkpoint_txg);
    let mut bytes = read(&image, device, offset, 512);
    bytes[100] ^= 0xff;
    write(&mut image, device, offset, &bytes);
    println!("INV-A X1 B 的根读不出时 recover(Consult): {}", outcome_text(&image, JournalPolicy::Consult));
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    let chosen = choose_root(&image, &system_configuration).expect("根");
    let records = scan_journal(&image, &system_configuration);
    let ring_bytes = system_configuration.immutable.sizes.journal_ring_bytes;
    let (once_report, once) = replay_journal(&image, &chosen, ring_bytes, &records, true).expect("施加一次");
    let (twice_report, twice) = replay_journal(&image, &once, ring_bytes, &records, true).expect("从施加出来的那一版再施加一次");
    let (again_report, again) = replay_journal(&image, &chosen, ring_bytes, &records, true).expect("从所选根再施加一次");
    println!(
        "INV-A X1 所选根 ({:?},{:?})；施加一次 → ({:?},{:?}) prefix_applied={} above_water={}；从那一版再施加 → ({:?},{:?}) prefix_applied={}；从所选根重做 → ({:?},{:?}) prefix_applied={}；一次与两次相同={}",
        chosen.instance, chosen.checkpoint_txg, once.instance, once.checkpoint_txg, once_report.prefix_applied, once_report.above_water,
        twice.instance, twice.checkpoint_txg, twice_report.prefix_applied, again.instance, again.checkpoint_txg, again_report.prefix_applied,
        once == twice
    );
    report("X1 B 根读不出、C 只有记录", &image);
}

/// 两份镜像逐扇区比：交回不同的扇区数（按盘）。
fn differing_sectors(left: &MemoryPool, right: &MemoryPool) -> Vec<(u32, usize)> {
    let mut out = Vec::new();
    for (identity, left_device) in &left.devices {
        let right_device = right.devices.get(identity).expect("同一组盘");
        let mut sectors: std::collections::BTreeSet<u64> = left_device.written_sectors().keys().copied().collect();
        sectors.extend(right_device.written_sectors().keys().copied());
        let differing = sectors
            .iter()
            .filter(|sector| left_device.written_sectors().get(sector) != right_device.written_sectors().get(sector))
            .count();
        out.push((identity.0, differing));
    }
    out
}

/// A1 X2（R-换 的字面）：新池新建文件做完的镜像上 ① 可写挂载一次；② 可写挂载崩在取号写之后（取号那两写与其后的屏障落了，写行一步没落），
/// 再可写挂载一次。比两份镜像：实例、写的行、恢复读出的内容、逐扇区。
#[test]
fn a1_recovery_and_row_write_once_versus_twice() {
    let parameters = parameters();
    let built = mkfs_stream();
    let image_a = memory_pool_of_sparse_devices(&built.devices);
    let once_stream = SharedStream::retaining_contents();
    let mut once_devices = devices_of(&image_a, &once_stream);
    let once = mount_writable(&parameters, &mut once_devices).expect("挂一次");
    let image_once = memory_pool_of_sparse_devices(&once_devices);
    let classifier = classifier(&parameters);
    let operations = once_stream.retained_operations();
    let kinds: Vec<StepKind> = operations.iter().map(|retained| classifier.classify(&retained.operation)).collect();
    let last_acquisition_write = kinds.iter().position(|kind| *kind == StepKind::SystemConfigurationSlot).expect("取号写") + 1;
    let cut = last_acquisition_write + 1 + kinds[last_acquisition_write + 1..].iter().take_while(|kind| **kind == StepKind::Barrier).count();
    println!("INV-A X2 挂载流前 {} 步的种类 {:?}，截在第 {cut} 步", cut + 2, &kinds[..(cut + 2).min(kinds.len())]);
    let mut image_crashed = image_a.clone();
    image_crashed.apply(&operations[..cut]);
    let twice_stream = SharedStream::retaining_contents();
    let mut twice_devices = devices_of(&image_crashed, &twice_stream);
    let twice = mount_writable(&parameters, &mut twice_devices).expect("崩过一次之后再挂");
    let image_twice = memory_pool_of_sparse_devices(&twice_devices);
    println!(
        "INV-A X2 挂一次: 实例 {:?} 行 {:?} 写行 txg {:?}；崩过再挂: 实例 {:?} 行 {:?} 写行 txg {:?}",
        once.output.instance, once.output.rows_written, once.output.row_publish.root().checkpoint_txg,
        twice.output.instance, twice.output.rows_written, twice.output.row_publish.root().checkpoint_txg
    );
    println!("INV-A X2 恢复读出: 挂一次 {}；崩过再挂 {}", outcome_text(&image_once, JournalPolicy::Consult), outcome_text(&image_twice, JournalPolicy::Consult));
    println!("INV-A X2 逐扇区不同（盘, 扇区数）: {:?}", differing_sectors(&image_once, &image_twice));
    report("X2 挂一次", &image_once);
    report("X2 崩过再挂", &image_twice);
}

/// 一块盘两槽里自证过的系统配置：(槽号, 值)。
fn system_configuration_slots(image: &MemoryPool, device: u32) -> Vec<(u64, singlefs_core::system_configuration::SystemConfiguration)> {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    singlefs_core::recovery::verified_system_configuration_slots(image, DeviceIdentity(device), spacing, &parameters().filesystem_identifier)
        .into_iter()
        .map(|slot| (slot.quantities.slot_generation % 2, slot))
        .collect()
}

fn write_system_configuration_slot(image: &mut MemoryPool, device: u32, slot: u64, value: &singlefs_core::system_configuration::SystemConfiguration) {
    let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
    write(image, device, slot * spacing, &value.to_slot());
}

/// A4 Y1–Y5：新池新建文件做完的镜像上改系统配置，逐个看 core 的只读恢复、可写挂载与池级 checker。
#[test]
fn a4_system_configuration_redundancy_images() {
    let parameters = parameters();
    let built = mkfs_stream();
    let clean = memory_pool_of_sparse_devices(&built.devices);
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let judge = |label: &str, image: &MemoryPool| {
        let verdicts = check_pool_image(image);
        let _ = inv_a_take_last_watermark_diagnostics();
        let not_applicable = verdicts.iter().filter(|(_, verdict)| matches!(verdict, InvariantVerdict::NotApplicable(_))).count();
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of(image, &stream);
        let writable = match mount_writable(&parameters, &mut devices) {
            Ok(mounted) => format!("Ok 实例 {:?}", mounted.output.instance),
            Err(error) => format!("Err {}", format!("{error:?}").chars().take(1400).collect::<String>()),
        };
        println!(
            "INV-A {label}: recover={} | mount_writable={writable} | checker 违例 {:?}，不适用 {not_applicable} 条（共 {} 条）",
            outcome_text(image, JournalPolicy::Consult),
            violated(&verdicts),
            verdicts.len()
        );
    };
    judge("Y0 干净", &clean);
    let slots_on_device_one = system_configuration_slots(&clean, 1);
    println!("INV-A Y 盘 1 两槽（槽号, 世代号, device_count）: {:?}", slots_on_device_one.iter().map(|(slot, value)| (*slot, value.quantities.slot_generation, value.immutable.device_count)).collect::<Vec<_>>());
    let mut one_slot_zeroed = clean.clone();
    let newest_slot = slots_on_device_one.iter().max_by_key(|(_, value)| value.quantities.slot_generation).expect("两槽").0;
    write(&mut one_slot_zeroed, 1, newest_slot * spacing, &[0u8; 4096]);
    judge("Y2 盘 1 世代号大的那一槽清零", &one_slot_zeroed);
    let mut both_zeroed = clean.clone();
    write(&mut both_zeroed, 1, 0, &[0u8; 4096]);
    write(&mut both_zeroed, 1, spacing, &[0u8; 4096]);
    judge("Y1 盘 1 两槽都清零", &both_zeroed);
    let mut both_zeroed_on_device_zero = clean.clone();
    write(&mut both_zeroed_on_device_zero, 0, 0, &[0u8; 4096]);
    write(&mut both_zeroed_on_device_zero, 0, spacing, &[0u8; 4096]);
    judge("Y1b 盘 0 两槽都清零", &both_zeroed_on_device_zero);
    let mut older_slot_zeroed = clean.clone();
    let older = slots_on_device_one.iter().min_by_key(|(_, value)| value.quantities.slot_generation).expect("两槽").0;
    write(&mut older_slot_zeroed, 1, older * spacing, &[0u8; 4096]);
    judge("Y2b 盘 1 世代号小的那一槽清零", &older_slot_zeroed);
    for device in [0u32, 1] {
        println!(
            "INV-A Y 盘 {device} 两槽（槽号, 世代号, tail）: {:?}",
            system_configuration_slots(&clean, device).iter().map(|(slot, value)| (*slot, value.quantities.slot_generation, format!("{:?}", value.quantities).chars().take(400).collect::<String>())).collect::<Vec<_>>()
        );
    }
    let mut device_count_both = clean.clone();
    for (slot, value) in &slots_on_device_one {
        let mut changed = value.clone();
        changed.immutable.device_count = 3;
        write_system_configuration_slot(&mut device_count_both, 1, *slot, &changed);
    }
    judge("Y3 盘 1 两槽 device_count 都改成 3（重封）", &device_count_both);
    let mut device_count_everywhere = clean.clone();
    for device in [0u32, 1] {
        for (slot, value) in &system_configuration_slots(&clean, device) {
            let mut changed = value.clone();
            changed.immutable.device_count = 3;
            write_system_configuration_slot(&mut device_count_everywhere, device, *slot, &changed);
        }
    }
    judge("Y6 两块盘四个槽 device_count 都改成 3（重封，池里只有两块盘）", &device_count_everywhere);
    let mut device_count_newest = clean.clone();
    let (newest_slot_number, newest_value) = slots_on_device_one.iter().max_by_key(|(_, value)| value.quantities.slot_generation).expect("两槽");
    let mut changed_newest = newest_value.clone();
    changed_newest.immutable.device_count = 3;
    write_system_configuration_slot(&mut device_count_newest, 1, *newest_slot_number, &changed_newest);
    judge("Y7 盘 1 世代号大的那一槽 device_count 改成 3（重封）", &device_count_newest);
    let mut device_count_older = clean.clone();
    let (older_slot, older_value) = slots_on_device_one.iter().min_by_key(|(_, value)| value.quantities.slot_generation).expect("两槽");
    let mut changed = older_value.clone();
    changed.immutable.device_count = 3;
    write_system_configuration_slot(&mut device_count_older, 1, *older_slot, &changed);
    judge("Y4 盘 1 世代号小的那一槽 device_count 改成 3（重封）", &device_count_older);
    let mut only_device_zero = clean.clone();
    only_device_zero.devices.remove(&DeviceIdentity(1));
    let verdicts = check_pool_image(&only_device_zero);
    let _ = inv_a_take_last_watermark_diagnostics();
    println!(
        "INV-A Y5 只剩盘 0: recover={} | checker 违例 {:?}",
        outcome_text(&only_device_zero, JournalPolicy::Consult),
        violated(&verdicts)
    );
}

/// A3 Z：内存盘（物理块 4096）上 mkfs 收不收 io_min 3072 / 12288，收了的话每个根槽偏移对 4096 与 io_min 取模。
#[test]
fn a3_mkfs_root_slot_alignment() {
    for (device_block, physical_block_size, io_min) in [(4096u32, 4096u32, 3072u32), (4096, 4096, 12288), (4096, 4096, 8192), (512, 512, 4096), (512, 4096, 3072), (512, 512, 3072), (512, 4096, 12288)] {
        let mut parameters = parameters();
        parameters.geometry.physical_block_size = physical_block_size;
        parameters.geometry.minimum_input_output_bytes = io_min;
        parameters.geometry.fixed_structure_slot_spacing = singlefs_core::system_configuration::SystemImmutableSizes::slot_spacing_for(io_min);
        let stream = SharedStream::retaining_contents();
        let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
        let mut devices: Devices = base
            .devices
            .iter()
            .map(|(identity, sparse)| {
                let mut device = SparseBlockDevice::new(IMAGE_BYTES, singlefs_core::block_device::PhysicalBlockSizeInBytes(device_block));
                device.image = sparse.clone();
                (*identity, RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()))
            })
            .collect();
        let result = make_filesystem(&parameters, &mut devices);
        println!("INV-A Z 设备块={device_block} pbs={physical_block_size} io_min={io_min} mkfs 交回时录制流里已有 {} 步", stream.operations().len());
        let offsets: Vec<(u64, u64, u64)> = (0..3u64)
            .flat_map(|region| (0..8u64).map(move |slot| (region, slot)))
            .map(|(region, slot)| {
                let offset = singlefs_core::root_ring::slot_offset(
                    singlefs_core::root_ring::RootRingSlot { region, slot },
                    parameters.geometry.fixed_structure_slot_spacing,
                )
                .0;
                (offset, offset % u64::from(physical_block_size), offset % u64::from(io_min))
            })
            .collect();
        let misaligned_to_the_block = offsets.iter().filter(|(_, by_block, _)| *by_block != 0).count();
        let misaligned_to_io_min = offsets.iter().filter(|(_, _, by_io_min)| *by_io_min != 0).count();
        println!(
            "INV-A Z 设备块={device_block} pbs={physical_block_size} io_min={io_min} 槽距={} mkfs={} 24 个根槽里偏移不是 pbs 整数倍的 {misaligned_to_the_block} 个、不是 io_min 整数倍的 {misaligned_to_io_min} 个；前 4 个 {:?}",
            parameters.geometry.fixed_structure_slot_spacing,
            match &result { Ok(_) => "Ok".to_string(), Err(error) => format!("Err {error:?}") },
            &offsets[..4]
        );
        if result.is_ok() {
            let image = MemoryPool {
                devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(),
                device_size_in_bytes: IMAGE_BYTES,
            };
            let verdicts = check_pool_image(&image);
            println!("INV-A Z 设备块={device_block} pbs={physical_block_size} io_min={io_min} mkfs 之后 checker 违例 {:?}", violated(&verdicts));
        }
    }
}

/// A5 R（「根轮出环」）：新池新建文件（txg 3，水位 11 → 19）之后连发 `k` 次零单元发布。两条臂：计划里的水位照抄 19（今天的实现）；
/// 计划里的水位写 11（不累计的实现）。每个 k 判一次：根环里还有没有 txg 3 那条根、只取根的 I-7.8、并进记录的读法、候选 I-7.8b。
#[test]
fn a5_roots_rotating_out_of_the_ring() {
    use singlefs_core::journal::back_chain_of;
    use singlefs_core::transaction::{publish_without_units, ZeroUnitPublishPlan};
    let parameters = parameters();
    for carried in [19u64, 11] {
        let mut built = mkfs_stream();
        let mut root = built.output.root;
        let mut counter = built.output.record.counter;
        let mut record_bytes = built.output.record_bytes.clone();
        let mut first_red: Option<usize> = None;
        for publish in 1..=60usize {
            {
                let mut writer = PoolWriter::new(&parameters, built.devices.as_mut_slice());
                let output = publish_without_units(
                    &mut writer,
                    &root,
                    ZeroUnitPublishPlan {
                        txg: CheckpointTxg(root.checkpoint_txg.0 + 1),
                        counter: counter + 1,
                        instance: root.instance,
                        back_chain: back_chain_of(&record_bytes),
                        rollback_floor: root.rollback_floor,
                        tree_identifier_watermark: carried,
                    },
                )
                .expect("零单元发布");
                root = output.root;
                counter = output.record.counter;
                record_bytes = output.record_bytes;
            }
            if ![1usize, 2, 22, 23, 24, 25, 30, 59, 60].contains(&publish) {
                continue;
            }
            let image = memory_pool_of_sparse_devices(&built.devices);
            let (slot_device, slot_offset) = root_slot_of(CheckpointTxg(3));
            let slot_bytes = read(&image, slot_device, slot_offset, 512);
            let txg_in_the_slot_of_the_first_file = u64::from_le_bytes(slot_bytes[28..36].try_into().expect("8 字节"));
            let (violated_here, _) = report(&format!("R 水位照抄 {carried}，第 {publish} 次零单元发布之后（txg {}）", root.checkpoint_txg.0), &image);
            println!(
                "INV-A R 水位照抄 {carried} 第 {publish} 次之后：txg 3 那个根槽里现在的 txg（偏移 28）= {txg_in_the_slot_of_the_first_file}；recover={}",
                outcome_text(&image, JournalPolicy::Consult)
            );
            if !violated_here.is_empty() && first_red.is_none() {
                first_red = Some(publish);
            }
        }
        println!("INV-A R 汇总：水位照抄 {carried}，第一次有违例是第 {first_red:?} 次零单元发布之后");
    }
}

/// A6：新池新建文件做完的镜像上，把每条有效根里实例表指针（根记录偏移 170）头部的 extent 偏移（指针偏移 32，2 字节）改成 1、重封；
/// 对照：同一条指针的 MAC 第一个字节改成 1、重封。看 core 的只读恢复、可写挂载与池级 checker。
#[test]
fn a6_nonzero_extent_offset_in_a_pointer_head() {
    let parameters = parameters();
    let built = mkfs_stream();
    let clean = memory_pool_of_sparse_devices(&built.devices);
    for (label, pointer_offset) in [("extent 偏移（指针偏移 32）", 32usize), ("MAC 首字节（指针偏移 0）", 0)] {
        let mut image = clean.clone();
        let mut rewritten = 0;
        for (device, offset) in root_slots() {
            let mut bytes = read(&image, device, offset, 512);
            if bytes[..4] != singlefs_core::root_record::ROOT_MAGIC {
                continue;
            }
            bytes[170 + pointer_offset] = 1;
            let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&bytes, 512, 138);
            bytes[138..170].copy_from_slice(&digest);
            write(&mut image, device, offset, &bytes);
            rewritten += 1;
        }
        let verdicts = check_pool_image(&image);
        let _ = inv_a_take_last_watermark_diagnostics();
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of(&image, &stream);
        let writable = match mount_writable(&parameters, &mut devices) {
            Ok(mounted) => format!("Ok 实例 {:?}", mounted.output.instance),
            Err(error) => format!("Err {}", format!("{error:?}").chars().take(300).collect::<String>()),
        };
        println!(
            "INV-A A6 {label} 改成 1（改了 {rewritten} 条根）: recover={} | mount_writable={writable} | checker 违例 {:?}",
            outcome_text(&image, JournalPolicy::Consult),
            verdicts.iter().filter_map(|(name, verdict)| match verdict { InvariantVerdict::Violated(detail) => Some(format!("{name}: {}", detail.chars().take(120).collect::<String>())), _ => None }).collect::<Vec<_>>()
        );
    }
}

/// A5 R2（「根轮出环」，走会话的发布入口）：mkfs → 可写挂载 → 新池新建文件（水位 11 → 19）→ 经 `MountedSession::publish_user_change`
/// 连发 40 次覆盖写。几个取样点上判一次（合法镜像：发树的那条根轮出环之后只取根的 I-7.8 还成不成立）；最后把根环里每条根的水位压到 11
/// （不累计的实现写下的样子）再判一次。
#[test]
fn a5_roots_rotating_out_through_the_session() {
    use singlefs_core::mounted_session::{MountedSession, UserChange};
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = fresh_devices(&stream);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 1");
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let PoolVersion::WithoutFile(version) = &session.current else { panic!("树表 0 条") };
    let (root, record_bytes) = (version.root, version.record_bytes.clone());
    let first_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut session.allocator,
            &root,
            FirstFile { content: &common::file_content(), write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            session.instance,
            &record_bytes,
        )
        .expect("新池新建文件")
    };
    let first_file_txg = first_file.root.checkpoint_txg;
    println!("INV-A R2 新建文件 txg {:?}，水位 {}", first_file_txg, first_file.root.tree_identifier_watermark);
    session.current = PoolVersion::WithFile(first_file);
    let mut last_image = memory_pool_of_sparse_devices(&devices);
    for publish in 1..=40u64 {
        let content = vec![u8::try_from(publish).expect("小于 256"); 100];
        let outcome = session.publish_user_change(
            &mut devices,
            UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + publish }),
        );
        let published = match outcome {
            Ok(published) => published,
            Err(refused) => {
                println!("INV-A R2 第 {publish} 次覆盖写被拒：{}", format!("{refused:?}").chars().take(300).collect::<String>());
                break;
            }
        };
        let PoolVersion::WithFile(current) = &session.current else { panic!("带文件") };
        last_image = memory_pool_of_sparse_devices(&devices);
        if [1u64, 12, 20, 21, 22, 23, 24, 25, 26, 30, 40].contains(&publish) {
            let (slot_device, slot_offset) = root_slot_of(first_file_txg);
            let slot_bytes = read(&last_image, slot_device, slot_offset, 512);
            let txg_in_the_slot = u64::from_le_bytes(slot_bytes[28..36].try_into().expect("8 字节"));
            report(
                &format!(
                    "R2 第 {publish} 次覆盖写之后（现行 txg {}，水位 {}，这一次推了 {} 串抬 F；新建文件那个根槽里现在的 txg {txg_in_the_slot}）",
                    current.root.checkpoint_txg.0,
                    current.root.tree_identifier_watermark,
                    published.floor_raises.len()
                ),
                &last_image,
            );
        }
    }
    println!("INV-A R2 最后一份镜像 recover={}", outcome_text(&last_image, JournalPolicy::Consult));
    let mut lowered = last_image.clone();
    let rewritten = set_the_tree_identifier_watermark_of_every_root(&mut lowered, 11);
    let (violated_lowered, _) = report(&format!("R2 最后一份镜像根环水位全压到 11（改了 {rewritten} 条根）"), &lowered);
    println!("INV-A R2 压低之后违例 {violated_lowered:?}；recover={}", outcome_text(&lowered, JournalPolicy::Consult));
    let mut newest_only = last_image.clone();
    let PoolVersion::WithFile(current) = &session.current else { panic!("带文件") };
    let (device, offset) = root_slot_of(current.root.checkpoint_txg);
    let mut bytes = read(&newest_only, device, offset, 512);
    bytes[122..130].copy_from_slice(&11u64.to_le_bytes());
    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&bytes, 512, 138);
    bytes[138..170].copy_from_slice(&digest);
    write(&mut newest_only, device, offset, &bytes);
    let (violated_newest, _) = report("R2 最后一份镜像只把最新那条根的水位压到 11", &newest_only);
    println!("INV-A R2 只压最新根之后违例 {violated_newest:?}；recover={}", outcome_text(&newest_only, JournalPolicy::Consult));
}
