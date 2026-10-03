//! m3-inv-a-r1 攻方腿原型：在自己造的小流上，把每个崩溃状态（层 0 的枚举域：屏障与 FUA 切段、段内整写子集、原地覆写第三态）
//! 交给观察者——判一次池级 checker（只取根的 I-7.8、并进记录的读法、候选 I-7.8b），再可写挂载一次、判一次，
//! 挂出来是树表 0 条的一版就再新建一次文件、判一次。全部用内存稀疏盘；流都在这里自己造。
//! 断言只钉「造出来的历史真是想造的那一段」，不钉结论；观测打成 `INV-A-E …` 开头的行。

#[path = "../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;

use common::{crash_state_devices, geometry, memory_pool_of_sparse_devices, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::{
    check_pool_image, inv_a_global_counts, inv_a_skip_global_counts_on_this_thread, inv_a_take_last_watermark_diagnostics,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_selecting_versions_observing_each_state, full_expansion,
    layer0_state_count_with_torn_in_place_overwrites,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

fn devices_of(image: &MemoryPool, stream: &SharedStream) -> Devices {
    crash_state_devices(image, &[], &[], stream)
}

fn file_content(seed: usize) -> Vec<u8> {
    (0..3000usize).map(|index| u8::try_from((index * 7 + seed) % 251).expect("小于 256")).collect()
}

/// 一次池级 checker 的粗键：只取根的 I-7.8、并进记录的读法、候选 I-7.8b、违例清单。交回 (键, I-7.8b 的第一处)。
fn checker_key(image: &MemoryPool) -> (String, Option<String>) {
    let verdicts = check_pool_image(image);
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    match inv_a_take_last_watermark_diagnostics() {
        Some(diagnostics) => {
            let stock = diagnostics.roots_max > diagnostics.highest_seen;
            let with_records = diagnostics.roots_max.max(diagnostics.records_max.unwrap_or(0)) > diagnostics.highest_seen;
            (
                format!(
                    "只取根={} 并进记录={} I-7.8b={} 违例={violated:?}",
                    if stock { "H" } else { "V" },
                    if with_records { "H" } else { "V" },
                    if diagnostics.b78_first_violation.is_none() { "H" } else { "V" },
                ),
                diagnostics.b78_first_violation,
            )
        }
        None => (format!("没走到 I-7.8 违例={violated:?}"), None),
    }
}

/// 每块盘两槽里自证过的系统配置各几份。
fn verified_slots(image: &MemoryPool) -> String {
    let parameters = parameters();
    [0u32, 1]
        .iter()
        .map(|device| {
            singlefs_core::recovery::verified_system_configuration_slots(
                image,
                DeviceIdentity(*device),
                u64::from(parameters.geometry.fixed_structure_slot_spacing),
                &parameters.filesystem_identifier,
            )
            .len()
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// 观察者的累计：粗键 → 状态数；再记几样第一处。
#[derive(Default)]
struct Observed {
    keys: BTreeMap<String, u64>,
    first_b78: Option<String>,
    first_refusal: BTreeMap<String, String>,
    states: u64,
}

impl Observed {
    /// 一个崩溃状态：判 → 可写挂载 → 判 →（树表 0 条时）新建文件 → 判。
    fn observe(&mut self, image: &MemoryPool) {
        let parameters = parameters();
        self.states += 1;
        let (crash_key, crash_b78) = checker_key(image);
        if self.first_b78.is_none() {
            self.first_b78 = crash_b78.map(|text| format!("崩溃态：{text}"));
        }
        let mut key = format!("崩溃态[自证槽 {}] {crash_key}", verified_slots(image));
        let stream = SharedStream::retaining_contents();
        let mut devices = devices_of(image, &stream);
        match mount_writable(&parameters, &mut devices) {
            Err(error) => {
                let text = format!("{error:?}");
                let kind: String = text.chars().take_while(|character| character.is_alphanumeric() || *character == '_').collect();
                let kind = if text.contains("SomeSystemConfigurationSlotUnreadOrUnverified") {
                    format!("{kind}/SomeSystemConfigurationSlotUnreadOrUnverified")
                } else {
                    kind
                };
                self.first_refusal.entry(kind.clone()).or_insert_with(|| text.chars().take(500).collect());
                key.push_str(&format!(" → 可写挂载 Err {kind}"));
            }
            Ok(mounted) => {
                let after_mount = memory_pool_of_sparse_devices(&devices);
                let (mount_key, mount_b78) = checker_key(&after_mount);
                if self.first_b78.is_none() {
                    self.first_b78 = mount_b78.map(|text| format!("可写挂载之后：{text}"));
                }
                let applied = mounted.output.journal.prefix_applied;
                match &mounted.current {
                    PoolVersion::WithFile(_) => {
                        key.push_str(&format!(" → 可写挂载 Ok（施加 {applied} 条，带文件）{mount_key}"));
                    }
                    PoolVersion::WithoutFile(version) => {
                        key.push_str(&format!(" → 可写挂载 Ok（施加 {applied} 条，树表 0 条，写行水位 {}）{mount_key}", mounted.output.row_publish.root().tree_identifier_watermark));
                        let mut allocator = mounted.allocator.clone();
                        let published = {
                            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                            publish_first_file(
                                &mut writer,
                                &mut allocator,
                                &version.root,
                                FirstFile { content: &file_content(29), write_time_seconds: FIXED_WRITE_TIME_SECONDS + 600 },
                                mounted.output.instance,
                                &version.record_bytes,
                            )
                        };
                        match published {
                            Ok(output) => {
                                let after_file = memory_pool_of_sparse_devices(&devices);
                                let (file_key, file_b78) = checker_key(&after_file);
                                if self.first_b78.is_none() {
                                    self.first_b78 = file_b78.map(|text| format!("新建文件之后：{text}"));
                                }
                                key.push_str(&format!(" → 新建文件 Ok（extent 树 {}）{file_key}", output.tree_identifiers.extent.0));
                            }
                            Err(error) => {
                                let text: String = format!("{error:?}").chars().take(80).collect();
                                key.push_str(&format!(" → 新建文件 Err {text}"));
                            }
                        }
                    }
                }
            }
        }
        *self.keys.entry(key).or_insert(0) += 1;
    }

    fn print(&self, label: &str) {
        println!("INV-A-E {label} 观察者看过 {} 个状态，粗键 {} 种：", self.states, self.keys.len());
        for (key, count) in &self.keys {
            println!("INV-A-E {label} {count:>7} × {key}");
        }
        println!("INV-A-E {label} I-7.8b 第一处：{:?}", self.first_b78);
        for (kind, text) in &self.first_refusal {
            println!("INV-A-E {label} 可写挂载被拒 {kind} 的第一处：{text}");
        }
    }
}

fn pool_of(image: &CrashImage<'_>) -> MemoryPool {
    let stream = SharedStream::retaining_contents();
    memory_pool_of_sparse_devices(&crash_state_devices(image.base, image.writes, &image.persisted, &stream))
}

/// 起点历史：mkfs → 可写挂载（实例 1，树表 0 条）→ 新池新建文件（发树 11..18，水位 11 → 19）。只跑一次。
struct History {
    /// 新建文件之前的池。
    before_the_publish: MemoryPool,
    /// 新建文件那次发布的写表与段。
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    operations: Vec<RetainedOperation>,
    content: Vec<u8>,
    first_file: singlefs_core::transaction::TransactionOutput,
}

fn history() -> History {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
    let mut devices = devices_of(&base, &stream);
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mounted = mount_writable(&parameters, &mut devices).expect("实例 1");
    let PoolVersion::WithoutFile(version) = &mounted.current else { panic!("树表 0 条") };
    let before_the_publish = memory_pool_of_sparse_devices(&devices);
    let before = stream.retained_operations().len();
    let content = file_content(3);
    let mut allocator = mounted.allocator.clone();
    let first_file = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            &version.root,
            FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            mounted.output.instance,
            &version.record_bytes,
        )
        .expect("新池新建文件")
    };
    assert_eq!(first_file.root.tree_identifier_watermark, 19, "新建文件之后水位 19");
    let operations: Vec<RetainedOperation> = stream.retained_operations()[before..].to_vec();
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    History { before_the_publish, writes, segments, operations, content, first_file }
}

/// 新建文件那次发布里某几种写落了的镜像。
fn image_with(history: &History, kinds: &[StepKind]) -> MemoryPool {
    let persisted: Vec<bool> = history.writes.iter().map(|write| kinds.contains(&write.kind)).collect();
    let stream = SharedStream::retaining_contents();
    memory_pool_of_sparse_devices(&crash_state_devices(&history.before_the_publish, &history.writes, &persisted, &stream))
}

/// 在 `base` 上可写挂载一次、录下它的流，把这条流的每个崩溃状态（每一段都展开）交给观察者。
fn enumerate_the_mount_stream_over(label: &str, base: &MemoryPool, versions_content: Option<&[u8]>) {
    let one_in: u64 = std::env::var("INV_A_OBSERVER_ONE_IN").ok().and_then(|text| text.parse().ok()).unwrap_or(1);
    enumerate_the_mount_stream_over_sampling_the_observer(label, base, versions_content, one_in);
}

/// 同上；观察者那一串（可写挂载、新建文件，各判一次）只在序号的散列 mod `one_in` = 0 的状态上做（`one_in` = 1 时每个状态都做）。
/// 每个状态的池级 checker 判定（只取根、并进记录、候选 I-7.8b）由枚举器的工作线程照常判，从全进程累计里读，不受取样影响。
fn enumerate_the_mount_stream_over_sampling_the_observer(label: &str, base: &MemoryPool, versions_content: Option<&[u8]>, one_in: u64) {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = devices_of(base, &stream);
    let mounted = mount_writable(&parameters, &mut devices).expect("起点上可写挂载");
    let operations = stream.retained_operations();
    let (writes, segments) = writes_and_segments(&operations, &geometry());
    let closed_form = closed_form_state_count(&segments);
    let with_torn = layer0_state_count_with_torn_in_place_overwrites(base, &writes, &segments, &full_expansion);
    println!(
        "INV-A-E {label} 挂载流 {} 步、写 {} 次、段 {:?}；closed_form_state_count={closed_form}，带第三态的全量 {with_torn}；这次挂载施加 {} 条、实例 {:?}、行 {:?}",
        operations.len(),
        writes.len(),
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        mounted.output.journal.prefix_applied,
        mounted.output.instance,
        mounted.output.rows_written
    );
    assert!(closed_form <= 1_000_000, "一段历史的全量不超过约 10⁶");
    let judged_root_index = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua).expect("挂载流里有根槽写");
    let mut versions: Vec<PublishedVersion> = Vec::new();
    if let Some(content) = versions_content {
        let mut roots = vec![mounted.output.effective_root.clone()];
        roots.push(mounted.output.row_publish.root().clone());
        roots.extend(mounted.output.warm_up_publishes.iter().map(|publish| publish.root().clone()));
        for root in roots {
            versions.push(PublishedVersion { instance: root.instance, checkpoint_txg: root.checkpoint_txg, content: content.to_vec() });
        }
    }
    let mut observed = Observed::default();
    let started = std::time::Instant::now();
    let counts_before = inv_a_global_counts();
    inv_a_skip_global_counts_on_this_thread(true);
    let mut ordinal = 0u64;
    let tally = enumerate_layer0_selecting_versions_observing_each_state(
        base,
        &writes,
        &segments,
        judged_root_index,
        &versions,
        &|_segment_index, _segment| true,
        &mut |image: &CrashImage<'_>, _report: &RecoveryReport| {
            let mut mixed = ordinal.wrapping_add(0x9E37_79B9_7F4A_7C15);
            mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            mixed ^= mixed >> 31;
            ordinal += 1;
            if mixed % one_in == 0 {
                observed.observe(&pool_of(image));
            }
        },
    );
    inv_a_skip_global_counts_on_this_thread(false);
    let counts_after = inv_a_global_counts();
    let counts: Vec<u64> = counts_after.iter().zip(counts_before).map(|(after, before)| after - before).collect();
    println!(
        "INV-A-E {label} 工作线程上每个状态的池级 checker：走到 I-7.8 的 {} 次，只取根判违例 {}，并进记录判违例 {}，候选 I-7.8b 判违例 {}，记录里的水位高于根环里的 {}；观察者取样 1/{one_in}",
        counts[0], counts[1], counts[2], counts[3], counts[4]
    );
    println!(
        "INV-A-E {label} 枚举完：states={}（观察者 {}），耗时 {:.1} 秒；枚举器自己的判定：oracle 违例 {}、不看 journal 那一遍违例 {}、checker 判违例的 {:?}、恢复失败的状态 {}、点名验证跑过 {} / 不过 {}",
        tally.states,
        tally.observed_states,
        started.elapsed().as_secs_f64(),
        tally.violations,
        tally.ignored_violations,
        tally.checker_violated_states,
        tally.failed_states,
        tally.verification_ran_states,
        tally.verification_failed_states
    );
    if let Some(first) = &tally.first_violation {
        println!("INV-A-E {label} oracle 第一处：{}", first.chars().take(400).collect::<String>());
    }
    assert_eq!(tally.states, with_torn, "枚举出来的状态数 = 带第三态的全量");
    assert_eq!(tally.observed_states, tally.states, "枚举器把每个状态都交给过观察者");
    observed.print(label);
}

/// E2：新建文件的单元与记录落了、根槽与系统配置轮换没落（「根槽读不出而记录在」的崩溃镜像），其上可写挂载那条流的全部崩溃状态。
#[test]
fn e2_mount_stream_over_a_record_without_its_root() {
    let history = history();
    let base = image_with(&history, &[StepKind::UnitWrite, StepKind::JournalRecord]);
    let one_in: u64 = std::env::var("INV_A_OBSERVER_ONE_IN").ok().and_then(|text| text.parse().ok()).unwrap_or(1);
    enumerate_the_mount_stream_over_sampling_the_observer("E2 记录在、根没落", &base, Some(&history.content), one_in);
}

/// E3：新建文件只有单元落了（记录、根都没落），其上可写挂载那条流的全部崩溃状态。
#[test]
fn e3_mount_stream_over_units_without_their_record() {
    let history = history();
    let base = image_with(&history, &[StepKind::UnitWrite]);
    let one_in: u64 = std::env::var("INV_A_OBSERVER_ONE_IN").ok().and_then(|text| text.parse().ok()).unwrap_or(1);
    enumerate_the_mount_stream_over_sampling_the_observer("E3 只有单元落了", &base, None, one_in);
}

/// E4：同 E2 的起点，再把那次发布的数据单元两份清零（介质故障，不是崩溃态）——记录在、点名验证不过、不施加；其上可写挂载那条流的全部崩溃状态。
#[test]
fn e4_mount_stream_over_a_record_whose_named_data_unit_is_gone() {
    let history = history();
    let mut base = image_with(&history, &[StepKind::UnitWrite, StepKind::JournalRecord]);
    for pointer in &history.first_file.data_pointers {
        for location in pointer.locations {
            base.devices
                .get_mut(&location.device)
                .expect("盘")
                .write(DeviceOffsetInBytes(location.slot.to_device_offset().0), &vec![0u8; 32768]);
        }
    }
    enumerate_the_mount_stream_over("E4 记录在、数据单元清零", &base, None);
}

/// E5：新建文件那次发布自己的崩溃状态。全量（单元段 24 写）超过约 10⁶，不跑全量；这里只取两样，不是层 0 的枚举域：
/// ① 单元段只取「一个都没落 / 整段落」，其余各段整写子集（不含第三态）；② 单元段的随机子集 `INV_A_RANDOM_STATES` 个（记录、根、轮换都没落）。
#[test]
fn e5_sampled_states_of_the_first_file_publish() {
    let history = history();
    println!(
        "INV-A-E E5 新建文件那次发布：写 {} 次、段 {:?}、各段种类 {:?}；closed_form_state_count={}（超过约 10⁶，全量不跑）",
        history.writes.len(),
        history.segments.iter().map(Vec::len).collect::<Vec<_>>(),
        history.segments.iter().map(|segment| format!("{:?}", history.writes[segment[0]].kind)).collect::<Vec<_>>(),
        closed_form_state_count(&history.segments)
    );
    assert_eq!(history.operations.len() >= history.writes.len(), true);
    let mut observed = Observed::default();
    let started = std::time::Instant::now();
    // ① 前面的段全落、这一段取每个真子集（单元段只取空集），再加全部落了那一个。
    let mut persisted_before: Vec<bool> = vec![false; history.writes.len()];
    let mut partial_states = 0u64;
    for segment in &history.segments {
        let subsets: Vec<u64> = if segment.len() > 10 { vec![0] } else { (0..(1u64 << segment.len()) - 1).collect() };
        for subset in subsets {
            let mut persisted = persisted_before.clone();
            for (position, write) in segment.iter().enumerate() {
                persisted[*write] = subset >> position & 1 == 1;
            }
            observed.observe(&pool_of(&CrashImage { base: &history.before_the_publish, writes: &history.writes, persisted }));
            partial_states += 1;
        }
        for write in segment {
            persisted_before[*write] = true;
        }
    }
    observed.observe(&pool_of(&CrashImage { base: &history.before_the_publish, writes: &history.writes, persisted: persisted_before }));
    partial_states += 1;
    println!("INV-A-E E5 ① {partial_states} 个状态，耗时 {:.1} 秒", started.elapsed().as_secs_f64());
    observed.print("E5 ① 单元段整段");
    // ② 单元段的随机子集。
    let count: u64 = std::env::var("INV_A_RANDOM_STATES").ok().and_then(|text| text.parse().ok()).unwrap_or(16);
    let seed: u64 = std::env::var("INV_A_RANDOM_SEED").ok().and_then(|text| text.parse().ok()).unwrap_or(1);
    let unit_segment = history.segments.iter().find(|segment| segment.len() > 10).expect("单元段");
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut observed = Observed::default();
    let started = std::time::Instant::now();
    for _ in 0..count {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let mut persisted = vec![false; history.writes.len()];
        for (position, write) in unit_segment.iter().enumerate() {
            persisted[*write] = state >> position & 1 == 1;
        }
        observed.observe(&pool_of(&CrashImage { base: &history.before_the_publish, writes: &history.writes, persisted }));
    }
    println!("INV-A-E E5 ② 种子 {seed}、{count} 个随机单元子集，耗时 {:.1} 秒", started.elapsed().as_secs_f64());
    observed.print(&format!("E5 ② 随机单元子集（种子 {seed}）"));
}
