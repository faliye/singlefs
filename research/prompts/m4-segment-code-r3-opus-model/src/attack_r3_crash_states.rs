//! 攻方腿原型（m4-segment-code-r3，Q1 新攻击面）：整理的各次发布、回退那次发布写到一半崩溃。
//! 前两轮攻方都只在批与批之间整步崩（执行器自己的 `CrashesAfterBatchesThenMountsWritable`），一次没调崩溃枚举。
//! 这里自己造流：先跑前缀（录制流保留内容），再跑前缀 + 被攻的那一步，后一条流比前一条多出的那一截就是被攻那一步的写；
//! 按屏障与 FUA 切段（`writes_and_segments`），每一段展开整写子集（原地覆写另取撕裂第三态，认法照崩溃点重放的三条），
//! 每个崩溃状态：冷读恢复（根要是模型认过的一版、读出的内容与模型那一版相同）→ 可写挂载 → 池级 checker →
//! 挂载之后还有活意图的接着整理推到停 → 再过池级 checker、再冷读一次内容不变。
//! 不依赖 checker 档：枚举、撕裂镜像都在这里自己写。
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::compaction_plan::{CompactionBatchSize, CompactionBatchUnits, CompactionSwitches};
use singlefs_core::mount::mount_writable_with_space_admission;
use singlefs_core::mounted_session::{CompactionRunOutcome, MountedSession};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::history::{
    execute_history_with, CompactionChoice, CompactionRunEnding, ContentChoice, ContentLength,
    GeneratedHistory, HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation,
    HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, SnapshotChoice,
};
use singlefs_harness::memory_pool::{
    writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice, WrittenContents,
};
use singlefs_harness::model::{ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RetainedOperation, SharedStream};

const SECTOR: u64 = 512;

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn spanning(units: u64, selector: u64, fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::SpanningDataUnitsThroughTheSequentialWrite {
            data_units: units,
            selector,
        },
        fill_seed,
    }
}

fn compaction(batch_units: u64, region_selector: u64) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending: CompactionRunEnding::RunsUntilItStops,
    })
}

fn compaction_crashing(batch_units: u64, region_selector: u64, batches: u64) -> HistoryOperation {
    HistoryOperation::CompactWhileMounted(CompactionChoice {
        batch_units,
        region_selector,
        ending: CompactionRunEnding::CrashesAfterBatchesThenMountsWritable { batches },
    })
}

fn rollback(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

fn width_of(name: &str) -> HistoryDeviceWidth {
    match name {
        "384" => HistoryDeviceWidth::UnitAreaOf384Slots,
        "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
        "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
        _ => HistoryDeviceWidth::FourGibibytes,
    }
}

/// 一个形状：盘宽、前缀、被攻的那一步、续推整理时的批大小。
struct Shape {
    width: HistoryDeviceWidth,
    prefix: Vec<HistoryOperation>,
    target: Vec<HistoryOperation>,
    batch_units: u64,
}

fn shape_of(name: &str) -> Shape {
    let w = std::env::var("ATTACK_WIDTH").unwrap_or_else(|_| "384".to_string());
    let width = width_of(&w);
    let k = env_u64("ATTACK_K", 1);
    let region = env_u64("ATTACK_REGION", 0);
    let units = env_u64("ATTACK_UNITS", 40);
    let rollback_index = env_u64("ATTACK_ROLLBACK", 2);
    let write = || HistoryOperation::PublishOverwrite(spanning(units, 3, 17));
    let write2 = || HistoryOperation::PublishOverwrite(spanning(units / 2 + 1, 5, 23));
    match name {
        // 整理一趟推到停，整趟每次发布都在枚举里（签发、各批、最后一批删意图）。
        "compaction-run" => Shape { width, prefix: vec![write(), write2()], target: vec![compaction(k, region)], batch_units: k },
        // 先签发并推一批后崩、挂载（意图活着），被攻的是续推的那一趟。
        "compaction-resume" => Shape { width, prefix: vec![write(), write2(), compaction_crashing(k, region, 1)], target: vec![compaction(k, region)], batch_units: k },
        // 整理推到停之后回退，被攻的是回退那次发布（F1：映射照现行版 COW、位置变了的路径重写）。
        "rollback-after-compaction" => Shape { width, prefix: vec![write(), write2(), compaction(k, region)], target: vec![rollback(rollback_index)], batch_units: k },
        // 整理推一批后崩、挂载（意图活着），回退到签发之前，被攻的是回退那次发布（目标版没有意图、现行版有）。
        "rollback-with-live-intent" => Shape { width, prefix: vec![write(), write2(), compaction_crashing(k, region, 1)], target: vec![rollback(rollback_index)], batch_units: k },
        // 快照之下整理：建快照、覆盖写，再整理推到停（搬钉给快照的单元，KeptForTheSnapshots）。
        "compaction-under-snapshot" => Shape { width, prefix: vec![write(), HistoryOperation::CreateSnapshot, write2()], target: vec![compaction(k, region)], batch_units: k },
        // 快照之下整理推到停之后删快照，被攻的是删快照那几次发布（放的是搬过之后的目的地）。
        // 不带整理：小文件上覆盖写一次，被攻的就是这次覆盖写（对照：日志重建出来的一版之后接若干次覆盖写）。
        "overwrite-crash" => Shape { width, prefix: vec![HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 1 }, fill_seed: 5 })], target: vec![HistoryOperation::PublishOverwrite(ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 2 }, fill_seed: 6 })], batch_units: k },
        "destroy-snapshot-after-compaction" => Shape { width, prefix: vec![write(), HistoryOperation::CreateSnapshot, write2(), compaction(k, region)], target: vec![HistoryOperation::DestroySnapshot(SnapshotChoice { index_among_the_snapshots: 0 })], batch_units: k },
        other => panic!("没有这个形状：{other}"),
    }
}

struct Recorded {
    operations: Vec<RetainedOperation>,
    committed: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    newest_txg_at_the_end: u64,
    ending: String,
}

fn record(width: HistoryDeviceWidth, operations: Vec<HistoryOperation>) -> Recorded {
    let stream = SharedStream::retaining_contents();
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations,
    };
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let run = execute_history_with(
        &history,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: width,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        &stream,
        &mut |observation| {
            for (key, content) in observation.model.committed_versions() {
                committed.insert(key, content.map(|c| c.to_vec()));
            }
        },
    );
    for (key, content) in &run.committed_versions_at_the_end {
        committed.insert(*key, content.clone());
    }
    let newest_txg_at_the_end = run
        .committed_versions_at_the_end
        .iter()
        .map(|(key, _)| key.checkpoint_txg.0)
        .max()
        .unwrap_or(0);
    let ending = match &run.ending {
        HistoryEnding::Completed => "Completed".to_string(),
        other => format!("{other:?}").chars().take(600).collect(),
    };
    eprintln!(
        "OUTCOMES {:?}",
        run.outcomes
            .iter()
            .map(|o| format!("{o:?}").chars().take(220).collect::<String>())
            .collect::<Vec<_>>()
    );
    Recorded {
        operations: stream.retained_operations(),
        committed,
        newest_txg_at_the_end,
        ending,
    }
}

fn pool_of(width: HistoryDeviceWidth, operations: &[RetainedOperation]) -> MemoryPool {
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
    pool.apply(operations);
    pool
}

/// 原地覆写的认法三条（与崩溃点重放同一组）：不是单元写、长于一个扇区、罩住的范围里原来有东西（基镜像非零或更早一次带字节的写重叠）。
fn tearable(base: &MemoryPool, writes: &[RetainedWrite], index: usize) -> bool {
    let write = &writes[index];
    if write.kind == StepKind::UnitWrite || write.length_in_bytes() <= SECTOR {
        return false;
    }
    let end = write.offset.0 + write.length_in_bytes();
    let base_nonzero = base.devices.get(&write.device).is_some_and(|device| {
        device
            .written_sectors()
            .range(write.offset.0 / SECTOR..end.div_ceil(SECTOR))
            .any(|(_, bytes)| bytes.iter().any(|b| *b != 0))
    });
    let earlier = writes[..index].iter().any(|e| {
        matches!(e.contents, WrittenContents::Bytes(_))
            && e.device == write.device
            && e.offset.0 < end
            && write.offset.0 < e.offset.0 + e.length_in_bytes()
    });
    base_nonzero || earlier
}

/// 新旧不同的那一截前一半取新、后一半取旧。
fn torn(old: &[u8], new: &[u8]) -> Vec<u8> {
    let Some(first) = (0..new.len()).find(|i| old[*i] != new[*i]) else {
        return new.to_vec();
    };
    let last = (0..new.len()).rev().find(|i| old[*i] != new[*i]).expect("有第一个就有最后一个");
    let first_old = first + (last - first + 1) / 2;
    let mut out = new.to_vec();
    out[first_old..].copy_from_slice(&old[first_old..]);
    out
}

/// 在 `image` 上按次序叠一组写，每个写取一态：0 没落、1 撕裂（新旧不同的那一截前一半新、后一半旧）、2 落了。
fn apply_landing(image: &mut MemoryPool, writes: &[RetainedWrite], landing: &[(usize, u8)]) {
    for (index, how) in landing {
        let write = &writes[*index];
        match how {
            0 => {}
            2 => image.apply_writes(std::slice::from_ref(write)),
            _ => {
                let length = usize::try_from(write.length_in_bytes()).expect("长度");
                let old = image.devices.get(&write.device).expect("有这块盘").read(write.offset, length);
                let mut new = vec![0u8; length];
                write.contents.copy_range_into(0, &mut new);
                let device = image.devices.get_mut(&write.device).expect("有这块盘");
                device.write(write.offset, &torn(&old, &new));
            }
        }
    }
}

/// 一个崩溃状态按（段号、段内编码）寻址：前几段全落，第 `segment` 段里每次写各取一态（编码按各写的态数做混合进制，
/// 不含全落那一组；全落归下一段的空子集）；`segment == 段数` 是整条流全落那一个。
#[derive(Clone, Copy)]
struct CrashState {
    segment: usize,
    code: u64,
}

struct Enumeration {
    radices: Vec<Vec<u8>>,
    /// 前 k 段全落的镜像（k = 0..=段数）。
    prefix_images: Vec<MemoryPool>,
    states: Vec<(usize, u64)>,
}

impl Enumeration {
    fn of(base: &MemoryPool, writes: &[RetainedWrite], segments: &[Vec<usize>]) -> Self {
        let radices: Vec<Vec<u8>> = segments
            .iter()
            .map(|segment| segment.iter().map(|i| if tearable(base, writes, *i) { 3 } else { 2 }).collect())
            .collect();
        let mut prefix_images = vec![base.clone()];
        for segment in segments {
            let mut image = prefix_images.last().expect("至少一张").clone();
            let landing: Vec<(usize, u8)> = segment.iter().map(|i| (*i, 2u8)).collect();
            apply_landing(&mut image, writes, &landing);
            prefix_images.push(image);
        }
        // 每段（段号、这一段的状态数 = 各写态数之积 − 1），最后整条流全落那一个。
        let mut states: Vec<(usize, u64)> = radices
            .iter()
            .enumerate()
            .map(|(k, r)| (k, r.iter().map(|x| u64::from(*x)).product::<u64>() - 1))
            .collect();
        states.push((segments.len(), 1));
        Self { radices, prefix_images, states }
    }

    fn count(&self) -> u64 {
        self.states.iter().map(|(_, n)| *n).sum()
    }

    /// 全体状态按（段、编码）排的第 `ordinal` 个。
    fn nth(&self, mut ordinal: u64) -> CrashState {
        for (segment, n) in &self.states {
            if ordinal < *n {
                return CrashState { segment: *segment, code: ordinal };
            }
            ordinal -= n;
        }
        unreachable!("序号小于状态总数")
    }

    fn landing_of(&self, segments: &[Vec<usize>], state: CrashState) -> Vec<(usize, u8)> {
        if state.segment == segments.len() {
            return Vec::new();
        }
        let mut rest = state.code;
        segments[state.segment]
            .iter()
            .zip(&self.radices[state.segment])
            .map(|(write_index, radix)| {
                let r = u64::from(*radix);
                let digit = u8::try_from(rest % r).expect("小");
                rest /= r;
                (*write_index, if r == 2 { digit * 2 } else { digit })
            })
            .collect()
    }

    fn image_of(&self, writes: &[RetainedWrite], segments: &[Vec<usize>], state: CrashState) -> MemoryPool {
        let mut image = self.prefix_images[state.segment].clone();
        apply_landing(&mut image, writes, &self.landing_of(segments, state));
        image
    }
}

/// I-3.1 红在记账多的一边、环里自证过的根槽少于环长（有一槽是从没落盘的那一版留下的洞）：
/// `crates/singlefs-checker/invariants/05-空间记账.md` 第 8 行「钉最旧 txg 的代价（用户认的）」那一形，单列、不算打中。
fn is_the_known_pinned_threshold_form(name: &str, why: &str) -> bool {
    if name != "I-3.1" {
        return false;
    }
    let number_after = |marker: &str| -> Option<u64> {
        let rest = &why[why.find(marker)? + marker.len()..];
        rest.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()
    };
    let accounting = number_after("记账的已分配 Some(");
    let walked = number_after("遍历全部有效根得到 ");
    let ring = number_after("根环槽数 ");
    let readable = number_after("环里自证过的根槽 ");
    matches!((accounting, walked, ring, readable), (Some(a), Some(w), Some(r), Some(v)) if a > w && v < r)
}

fn reds_of(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(name, verdict)| match verdict {
            InvariantVerdict::Violated(why) => Some(if is_the_known_pinned_threshold_form(name, &why) {
                format!("已知形（钉最旧 txg，记账多）{name}: {}", why.chars().take(200).collect::<String>())
            } else {
                format!("{name}: {}", why.chars().take(300).collect::<String>())
            }),
            _ => None,
        })
        .collect()
}

fn devices_of(image: &MemoryPool) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(image.device_size_in_bytes, PhysicalBlockSizeInBytes(512));
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], size: u64) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(), device_size_in_bytes: size }
}

/// 判一个崩溃状态，交回问题（空就是没问题）与一行摘要。
fn judge(shape: &Shape, recorded: &Recorded, prefix_newest_txg: u64, image: &MemoryPool) -> (Vec<String>, String) {
    let mut problems = Vec::new();
    let report = recover(image, JournalPolicy::Consult);
    let (root, content) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (*root, Some(content.clone())),
        RecoveryOutcome::NoFile { root } => (*root, None),
        RecoveryOutcome::Failed { root, failure } => {
            problems.push(format!("冷读恢复失败 root={root:?} {failure:?}"));
            return (problems, "recover-failed".into());
        }
    };
    let effective = report.effective_root.unwrap_or(root);
    let key = ModelRootKey { checkpoint_txg: ModelCheckpointTxg(effective.1 .0), instance: ModelInstanceGeneration(u64::from(effective.0 .0)) };
    match recorded.committed.get(&key) {
        None => problems.push(format!("冷读走到模型没认过的根 {effective:?}")),
        Some(expected) => {
            if expected.as_deref() != content.as_deref() {
                problems.push(format!("冷读内容与模型那一版不同 {effective:?}"));
            }
        }
    }
    if effective.1 .0 < prefix_newest_txg {
        problems.push(format!("冷读走到被攻那一步之前最新根之前 {effective:?} < {prefix_newest_txg}"));
    }
    let parameters = shape.width.parameters();
    let mut devices = devices_of(image);
    let mounted = match mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(error) => {
            problems.push(format!("可写挂载拒 {error:?}"));
            return (problems, format!("root={effective:?} mount-refused"));
        }
    };
    let after_mount = image_of(&devices, image.device_size_in_bytes);
    for red in reds_of(&after_mount) {
        problems.push(format!("挂载之后 checker 红 {red}"));
    }
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let live = session.allocator.compaction().live_intent();
    let mut summary = format!("root={}:{} live={}", effective.0 .0, effective.1 .0, live.is_some());
    if let Some(live) = live.filter(|_| env_u64("ATTACK_RESUME", 1) == 1) {
        let mut switches = CompactionSwitches::PRODUCT;
        switches.batch_size = CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(usize::try_from(shape.batch_units).expect("小")).expect("至少 1"));
        switches.forced_region = Some(live.intent.region);
        session.allocator.set_compaction_switches(switches);
        match session.run_compaction(&mut devices) {
            Ok(run) => {
                let stop = match &run.outcome {
                    CompactionRunOutcome::Stopped(stop) => format!("{stop:?}"),
                    CompactionRunOutcome::Failed(failure) => {
                        problems.push(format!("续推失败 {failure:?}"));
                        "failed".into()
                    }
                };
                summary.push_str(&format!(" resumed batches={} stop={}", run.batches_published, stop.chars().take(80).collect::<String>()));
                if session.allocator.compaction().live_intent().is_some() {
                    summary.push_str(" still-live");
                }
            }
            Err(refused) => problems.push(format!("续推拒 {refused:?}")),
        }
        let after_resume = image_of(&devices, image.device_size_in_bytes);
        for red in reds_of(&after_resume) {
            problems.push(format!("续推之后 checker 红 {red}"));
        }
        let again = recover(&after_resume, JournalPolicy::Consult);
        let content_again = match again.outcome {
            RecoveryOutcome::FileRead { content, .. } => Some(content),
            _ => None,
        };
        if content_again != content {
            problems.push("续推之后冷读内容变了".into());
        }
    }
    (problems, summary)
}

#[test]
fn crash_states_of_one_shape() {
    let name = std::env::var("ATTACK_SHAPE").unwrap_or_else(|_| "compaction-run".into());
    let shape = shape_of(&name);
    let prefix = record(shape.width, shape.prefix.clone());
    let mut all = shape.prefix.clone();
    all.extend(shape.target.iter().copied());
    let full = record(shape.width, all);
    eprintln!("SHAPE {name} prefix_ending={} full_ending={}", prefix.ending, full.ending);
    assert!(full.operations.len() >= prefix.operations.len());
    assert_eq!(full.operations[..prefix.operations.len()], prefix.operations[..], "同一前缀跑出同一段录制流");
    let base = pool_of(shape.width, &prefix.operations);
    let target_ops = &full.operations[prefix.operations.len()..];
    let (writes, segments) = writes_and_segments(target_ops, &shape.width.fixed_geometry());
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    let tearables: usize = (0..writes.len()).filter(|i| tearable(&base, &writes, *i)).count();
    let enumeration = Enumeration::of(&base, &writes, &segments);
    let total = enumeration.count();
    let kinds: BTreeMap<String, usize> = writes.iter().fold(BTreeMap::new(), |mut m, w| { *m.entry(format!("{:?}", w.kind)).or_default() += 1; m });
    eprintln!("STREAM writes={} segments={} sizes={sizes:?} tearable={tearables} kinds={kinds:?} states={total}", writes.len(), segments.len());
    let limit = env_u64("ATTACK_STATE_LIMIT", 1_000_000);
    if env_u64("ATTACK_COUNT_ONLY", 0) == 1 || total > limit {
        eprintln!("COUNT_ONLY states={total} limit={limit}");
        return;
    }
    // 只为估时：ATTACK_ESTIMATE_EVERY=n 时每 n 个状态判一个（正式跑不设）。
    let every = env_u64("ATTACK_ESTIMATE_EVERY", 1).max(1);
    // 分片：ATTACK_ORDINAL_FROM..ATTACK_ORDINAL_TO（全量按序号切几片分开跑，合起来就是全体状态）。
    let from = env_u64("ATTACK_ORDINAL_FROM", 0);
    let to = env_u64("ATTACK_ORDINAL_TO", total).min(total);
    // ATTACK_ONLY_SEGMENTS_WITHOUT_UNIT_WRITES=1：只判不含单元写的段（记录、根、系统配置那几段）与全落那一个；单元段那一大片交整片分片跑。
    let only_small = env_u64("ATTACK_ONLY_SEGMENTS_WITHOUT_UNIT_WRITES", 0) == 1;
    let ordinals: Vec<u64> = (from..to)
        .step_by(usize::try_from(every).expect("小"))
        .filter(|o| {
            !only_small || {
                let st = enumeration.nth(*o);
                st.segment == segments.len() || segments[st.segment].iter().all(|i| writes[*i].kind != StepKind::UnitWrite)
            }
        })
        .collect();
    eprintln!("ORDINALS from={from} to={to} judged_here={}", ordinals.len());
    let started = std::time::Instant::now();
    let threads = usize::try_from(env_u64("ATTACK_THREADS", 31)).expect("小");
    let next = AtomicUsize::new(0);
    let findings: Mutex<Vec<(u64, CrashState, Vec<String>, String)>> = Mutex::new(Vec::new());
    let summaries: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());
    let judged = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let position = next.fetch_add(1, Ordering::Relaxed);
                if position >= ordinals.len() {
                    break;
                }
                let ordinal = ordinals[position];
                let state = enumeration.nth(ordinal);
                let image = enumeration.image_of(&writes, &segments, state);
                let (problems, summary) = judge(&shape, &full, prefix.newest_txg_at_the_end, &image);
                judged.fetch_add(1, Ordering::Relaxed);
                let key = format!("seg{} {}", state.segment, summary);
                *summaries.lock().unwrap().entry(key).or_default() += 1;
                if !problems.is_empty() {
                    findings.lock().unwrap().push((ordinal, state, problems, summary));
                }
            });
        }
    });
    let summaries = summaries.into_inner().unwrap();
    for (key, count) in &summaries {
        eprintln!("SUMMARY {key} count={count}");
    }
    let mut findings = findings.into_inner().unwrap();
    findings.sort_by_key(|f| f.0);
    let mut kinds_of_problems: BTreeMap<String, usize> = BTreeMap::new();
    for (ordinal, state, problems, summary) in &findings {
        for p in problems {
            *kinds_of_problems.entry(p.chars().take(90).collect()).or_default() += 1;
        }
        if env_u64("ATTACK_VERBOSE", 0) == 1 {
            let landing = enumeration.landing_of(&segments, *state);
            let kinds_landing: Vec<String> = landing.iter().map(|(i, h)| format!("{}:{:?}@{}:{}={h}", i, writes[*i].kind, writes[*i].device.0, writes[*i].offset.0)).collect();
            eprintln!("FINDING ordinal={ordinal} seg={} landing={kinds_landing:?} {summary} {problems:?}", state.segment);
        }
    }
    for (kind, count) in &kinds_of_problems {
        eprintln!("PROBLEM {kind} count={count}");
    }
    let first_few: Vec<u64> = findings.iter().take(10).map(|f| f.0).collect();
    eprintln!("ATTACK_TOTAL shape={name} states={total} judged={} findings={} first={first_few:?} every={every} seconds={}", judged.load(Ordering::Relaxed), findings.len(), started.elapsed().as_secs());
    }

/// 一个崩溃状态逐步看（ATTACK_ORDINAL 点名）：挂载之后每推一次发布（整理一次一批，或 ATTACK_CONTINUE_WITH=overwrite 时覆盖写一次）
/// 就过一遍池级 checker，打印每一步的 I-3.1 判定；ATTACK_ORDINAL 不给时取整条流全落那一个。
#[test]
fn one_crash_state_step_by_step() {
    let Ok(name) = std::env::var("ATTACK_STEP_SHAPE") else {
        return;
    };
    let shape = shape_of(&name);
    let prefix = record(shape.width, shape.prefix.clone());
    let mut all = shape.prefix.clone();
    all.extend(shape.target.iter().copied());
    let full = record(shape.width, all);
    let base = pool_of(shape.width, &prefix.operations);
    let target_ops = &full.operations[prefix.operations.len()..];
    let (writes, segments) = writes_and_segments(target_ops, &shape.width.fixed_geometry());
    let enumeration = Enumeration::of(&base, &writes, &segments);
    let ordinal = env_u64("ATTACK_ORDINAL", enumeration.count() - 1);
    let state = match std::env::var("ATTACK_STEP_SEGMENT").ok().and_then(|v| v.parse::<usize>().ok()) {
        Some(segment) => CrashState { segment, code: env_u64("ATTACK_STEP_CODE", 0) },
        None => enumeration.nth(ordinal),
    };
    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
    eprintln!("SEGMENTS sizes={sizes:?} kinds={:?}", segments.iter().map(|s| s.iter().map(|i| format!("{:?}", writes[*i].kind).chars().take(4).collect::<String>()).collect::<Vec<_>>().join(",")).collect::<Vec<_>>());
    let image = enumeration.image_of(&writes, &segments, state);
    let report = recover(&image, JournalPolicy::Consult);
    eprintln!("STATE ordinal={ordinal} seg={} effective={:?} journal={:?}", state.segment, report.effective_root, report.journal);
    let parameters = shape.width.parameters();
    let mut devices = devices_of(&image);
    let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("挂得上");
    eprintln!("MOUNT chosen={:?} effective={:?} isolated={:?} abandoned_unreadable={}", (mounted.output.chosen_root.instance, mounted.output.chosen_root.checkpoint_txg), (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg), mounted.output.isolated_slots_per_device, mounted.output.abandoned_roots_unreadable);
    let size = image.device_size_in_bytes;
    let print_reds = |label: &str, devices: &[(DeviceIdentity, SparseBlockDevice)]| {
        let reds = reds_of(&image_of(devices, size));
        eprintln!("CHECK {label} reds={}", reds.len());
        for red in reds {
            eprintln!("  RED {}", red.chars().take(400).collect::<String>());
        }
    };
    print_reds("after-mount", &devices);
    let (_output, mut session) = MountedSession::of_the_mount(mounted);
    let steps = env_u64("ATTACK_STEPS", 24);
    let with = std::env::var("ATTACK_CONTINUE_WITH").unwrap_or_else(|_| "compaction".into());
    let content: Vec<u8> = (0..3000u32).map(|i| u8::try_from(i % 251).expect("小")).collect();
    for step in 0..steps {
        let live = session.allocator.compaction().live_intent();
        if with == "overwrite" {
            let result = session.publish_user_change(&mut devices, singlefs_core::mounted_session::UserChange::Overwrite(singlefs_core::transaction::FirstFile { content: &content, write_time_seconds: 7 }));
            eprintln!("STEP {step} overwrite ok={} txg={:?}", result.is_ok(), session.current.version_of_trees().map(|v| v.root.checkpoint_txg));
        } else {
            let Some(live) = live else {
                eprintln!("STEP {step} no live intent");
                break;
            };
            let mut switches = CompactionSwitches::PRODUCT;
            switches.batch_size = CompactionBatchSize::ForcedForTests(CompactionBatchUnits::new(usize::try_from(shape.batch_units).expect("小")).expect("至少 1"));
            switches.forced_region = Some(live.intent.region);
            session.allocator.set_compaction_switches(switches);
            session.compaction.publishes_since_the_last_user_change = 19;
            let run = session.run_compaction(&mut devices).expect("推得动");
            let stop = match &run.outcome {
                CompactionRunOutcome::Stopped(stop) => format!("{stop:?}"),
                CompactionRunOutcome::Failed(failure) => format!("failed {failure:?}"),
            };
            eprintln!("STEP {step} batches={} versions={} txg={:?} stop={}", run.batches_published, run.versions_published.len(), run.versions_published.last().map(|v| v.root.checkpoint_txg), stop.chars().take(120).collect::<String>());
        }
        print_reds(&format!("after-step-{step}"), &devices);
    }
}
