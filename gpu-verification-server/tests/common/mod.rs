//! 几份测试共用的：新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、新建文件，录 mkfs 之后那一段）的判表，
//! 不上卡的调度测试造流、造卡、模拟判完一波的写法，与判定库那一份待判池的测试要的临时判定库服务、替身验证包一侧、造待判行、核对待判行与判表待判行的写法。

#![allow(dead_code, reason = "每份测试文件只用其中几样")]

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpu_verification_server::batch_scheduling::{
    BatchSchedulingConfiguration, PendingBlock, ResidentJudge, WavePlan,
};
use gpu_verification_server::break_switches::BreakSwitches;
use gpu_verification_server::card_memory_ledger::{
    bytes_of_one_state_on_the_card, states_fitting_in, CardIndex, CardOfTheService,
    BYTES_PER_MEBIBYTE,
};
use gpu_verification_server::facts_verifier_family::{
    FactsVerifierBlockName, FactsVerifierMaterial, FactsVerifierPendingRowOfTheLibrary,
};
use gpu_verification_server::library_pending_pool::{
    ConclusionFrame, PendingBlockName, PendingRowOfTheLibrary, StreamOfTheRow,
    VerificationPackageSide,
};
use gpu_verification_server::packed_tables_family::{
    PackedTablesFrame, PackedTablesName, PackedTablesPendingRowOfTheLibrary,
};
use gpu_verification_server::pending_pool::{
    CardWorkOnAPiece, InMemoryPendingPool, PendingPool, StateVerdict,
};
use gpu_verification_server::stream_identity::{StreamFacts, StreamKey};
use prune_server::client::RouteAddress;
use prune_server::configuration::socket_path_problem;
use prune_server::protocol::Table;
use prune_server::reliable_client::{ReliableClient, ReliableClientSettings};
use prune_server::server::{start_in_this_process, ServiceInThisProcess};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_verification::crash::CrashPointReplaySegmentExpansion;
use singlefs_verification::crash_amplification::{
    container_capacity_knobs_digest, facts_verifier_pending_block_is_verified_in_the_library,
    facts_verifier_pending_stream_key, facts_verifier_row_key,
    packed_tables_verdict_words_are_in_the_library, CardThatJudgedAPendingBlock,
    GpuPendingBlockInTheLibrary,
};
use singlefs_verification::crash_facts::{
    extract_flow_facts, FlowFacts, FlowFactsReadBack, SegmentExpansionFact,
};
use singlefs_verification::crash_identity::ConclusionVersion;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::crash_judge_tables::{build_judge_tables, JudgeFlow, JudgeTables};
use singlefs_verification::crash_library::ExclusiveTemporaryDirectory;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
    SparseBlockDevice,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::verdict_store::{
    BlockStartStateIndex, ContainerCapacityKnobs, EnumerationPlanHash, FactsVerifierPendingRow,
    FactsVerifierRowsForOneFrame, ForcedRejudging, GpuPendingRow, GpuPendingRowPut,
    GpuPendingStreamKey, JudgingDevice, LibraryKeySpace, NodeInputDigest, PackedTablesPendingRow,
    PruneServerFailure, PruneServerFailureHandling, PruneServerStorage, VerdictBlockKey,
    VerdictStore, VerdictStoreError, VerifierBlockRow,
};
use singlefs_verification::{RecordingBlockDevice, SharedStream};

/// 两块内存盘各多大。
const DEVICE_BYTES: u64 = 4 << 30;

/// 新池新建文件那条流的判表：段长不超过 `expand_up_to` 的段逐个真子集展开，更长的段只以整段持久进入后面的状态。
/// 展开上限不同，表与态数不同，就是两条流。
///
/// # Panics
/// 新池新建文件那条路做不成、建不出表。
pub fn new_pool_file_creation_judge_tables(expand_up_to: usize) -> JudgeTables {
    new_pool_file_creation_judge_tables_expanding(expand_up_to, None)
}

/// 同 [`new_pool_file_creation_judge_tables`]，只是序号为 `segment_left_whole` 的那一段不展开（段长够得上也只以整段持久）：
/// 同一个展开上限下少一段的展开，态数少一截、表不同，是另一条流。
///
/// # Panics
/// 新池新建文件那条路做不成、建不出表。
pub fn new_pool_file_creation_judge_tables_leaving_one_segment_whole(
    expand_up_to: usize,
    segment_left_whole: usize,
) -> JudgeTables {
    new_pool_file_creation_judge_tables_expanding(expand_up_to, Some(segment_left_whole))
}

/// 新池新建文件那条流录下的：mkfs 之后的基线、mkfs 之后的写与段、判的那一条根槽写、发布的版本。
pub struct NewPoolFileCreationRecording {
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub segments: Vec<Vec<usize>>,
    pub judged_root_index: usize,
    pub versions: Vec<PublishedVersion>,
}

/// 段长不超过 `expand_up_to`、且不是 `segment_left_whole` 那一段的逐个真子集展开，别的只以整段持久。
fn segment_expansion_up_to(
    expand_up_to: usize,
    segment_left_whole: Option<usize>,
) -> impl Fn(usize, &[usize]) -> CrashPointReplaySegmentExpansion {
    move |segment_index: usize, segment: &[usize]| {
        if segment.len() <= expand_up_to && segment_left_whole != Some(segment_index) {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    }
}

/// 新池新建文件那条流的事实表（第 ② 段核对读的那一份）与每段的拆法，段的展开同 [`new_pool_file_creation_judge_tables`]。
///
/// # Panics
/// 新池新建文件那条路做不成。
pub fn new_pool_file_creation_facts(expand_up_to: usize) -> (FlowFacts, Vec<SegmentExpansionFact>) {
    let recording = record_new_pool_file_creation();
    let expansion = segment_expansion_up_to(expand_up_to, None);
    let facts = extract_flow_facts(
        &recording.base,
        &recording.writes,
        &recording.segments,
        &expansion,
    );
    let segment_expansions = recording
        .segments
        .iter()
        .enumerate()
        .map(|(segment_index, segment)| {
            SegmentExpansionFact::of(
                expansion(segment_index, segment),
                &recording.writes,
                segment,
            )
        })
        .collect();
    (facts, segment_expansions)
}

/// 录新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、新建文件，录 mkfs 之后那一段）。
///
/// # Panics
/// 新池新建文件那条路做不成；录下的写里没有根槽写。
pub fn record_new_pool_file_creation() -> NewPoolFileCreationRecording {
    let stream = SharedStream::retaining_contents();
    let mut devices = [0, 1].map(|device| {
        (
            DeviceIdentity(device),
            RecordingBlockDevice::with_shared_stream(
                DeviceIdentity(device),
                SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                stream.clone(),
            ),
        )
    });
    let parameters = singlefs_verification::known_bad_images::parameters_of_the_common_test_pool();
    let run = singlefs_verification::scenario::run_new_pool_file_creation(
        &parameters,
        &mut devices,
        &stream,
        |_, _| {},
    )
    .expect("新池新建文件那条路做得成");
    let operations = stream.retained_operations();
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], DEVICE_BYTES);
    base.apply(&operations[..run.mkfs_operation_count]);
    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[run.mkfs_operation_count..],
        &FixedGeometry {
            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
        },
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("新建文件那次发布有一条根槽写");
    let versions = vec![PublishedVersion {
        instance: run.output.root.instance,
        checkpoint_txg: run.output.root.checkpoint_txg,
        content: singlefs_verification::scenario::first_file_content(),
    }];
    NewPoolFileCreationRecording {
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    }
}

fn new_pool_file_creation_judge_tables_expanding(
    expand_up_to: usize,
    segment_left_whole: Option<usize>,
) -> JudgeTables {
    let recording = record_new_pool_file_creation();
    let expansion = segment_expansion_up_to(expand_up_to, segment_left_whole);
    build_judge_tables(&JudgeFlow {
        base: &recording.base,
        writes: &recording.writes,
        segments: &recording.segments,
        judged_root_index: recording.judged_root_index,
        versions: &recording.versions,
        expansion: &expansion,
    })
    .expect("新池新建文件那条流建得出判表")
}

/// 不上卡的测试里的一条流：表摘要的每个字节都是 `tag`（`tag` 越大键越大）。
pub fn stream_tagged(tag: u8) -> StreamKey {
    StreamKey {
        tables_digest: [tag; 32],
        kernel_sources_digest: [0; 32],
    }
}

/// `states` 个态的草稿区要几 MiB（向上取整）。
pub fn mebibytes_of_states(states: u64) -> u64 {
    let mebibytes = (states * bytes_of_one_state_on_the_card()).div_ceil(BYTES_PER_MEBIBYTE);
    assert_eq!(
        states_fitting_in(mebibytes),
        states,
        "一个态比 1 MiB 大：向上取整的那一点装不下多一个态"
    );
    mebibytes
}

/// 一张给判器留 `mebibytes_for_the_judges` 的卡（额度另加驱动那一份）。
pub fn card_leaving(index: u32, mebibytes_for_the_judges: u64) -> CardOfTheService {
    CardOfTheService {
        card: CardIndex(index),
        service_quota_mebibytes: mebibytes_for_the_judges + MEBIBYTES_LEFT_FOR_THE_DRIVER,
    }
}

/// 限时 `limit_milliseconds`，弄坏开关照这个进程的环境（`GPU_VERIFICATION_SERVER_BREAK`）。
pub fn configuration_with_the_limit(limit_milliseconds: u64) -> BatchSchedulingConfiguration {
    BatchSchedulingConfiguration {
        oldest_pending_wait_limit: Duration::from_millis(limit_milliseconds),
        break_switches: BreakSwitches::of_the_environment(),
    }
}

pub fn milliseconds(count: u64) -> Duration {
    Duration::from_millis(count)
}

/// 一份流表。
pub fn streams_of(entries: &[(StreamKey, u64, u64)]) -> BTreeMap<StreamKey, StreamFacts> {
    entries
        .iter()
        .map(|(stream, table_mebibytes, state_count)| {
            (
                *stream,
                StreamFacts {
                    table_mebibytes: *table_mebibytes,
                    state_count: *state_count,
                },
            )
        })
        .collect()
}

/// 一个内存池，按次序入池 `(流, 序号, 入池时刻)`。
pub fn pool_with(blocks: &[(StreamKey, Range<u64>, Duration)]) -> InMemoryPendingPool {
    let mut pool = InMemoryPendingPool::empty(BreakSwitches::of_the_environment());
    for (stream, ordinals, entered_at) in blocks {
        pool.add_block(*stream, ordinals.clone(), *entered_at)
            .expect("非空块入得了池");
    }
    pool
}

/// 照计划把派出去的截在池里记成在判，交回这一波之后各卡驻留的判器（没进计划的卡上的照留）。
pub fn apply_the_wave(
    pool: &mut InMemoryPendingPool,
    plan: &WavePlan,
    residents: &[ResidentJudge],
) -> Vec<ResidentJudge> {
    let mut pieces: Vec<_> = plan.pieces().collect();
    pieces.sort_by_key(|piece| (piece.block, piece.ordinals.start));
    for piece in pieces {
        pool.mark_dispatched(piece).expect("池收得下调度派出去的截");
    }
    let planned: Vec<CardIndex> = plan.cards.iter().map(|card| card.card).collect();
    let mut after: Vec<ResidentJudge> = residents
        .iter()
        .filter(|resident| !planned.contains(&resident.card))
        .copied()
        .collect();
    for card in &plan.cards {
        for judge in card.resident_after_the_wave().judges() {
            after.push(ResidentJudge {
                card: card.card,
                stream: judge.stream,
                states_per_dispatch: judge.states_per_dispatch,
            });
        }
    }
    after
}

/// 照计划把每一截都记上结论，记在计划派它的那张卡上（结论随便填：调度测试只看派法）。
pub fn judge_the_wave(pool: &mut InMemoryPendingPool, plan: &WavePlan) {
    for card in &plan.cards {
        for piece in card.batches.iter().flat_map(|batch| batch.pieces.iter()) {
            let states =
                usize::try_from(piece.ordinals.end - piece.ordinals.start).expect("装得进 usize");
            pool.record_verdicts(
                piece,
                card.card,
                vec![[0; VERDICT_WORDS]; states],
                CardWorkOnAPiece {
                    dispatches: 1,
                    card_nanoseconds: 0,
                },
            )
            .expect("派出去的截记得上结论");
        }
    }
}

/// 池里此刻没派的块。
pub fn undispatched(pool: &InMemoryPendingPool) -> Vec<PendingBlock> {
    pool.undispatched_blocks()
}

/// socket 放不进临时服务自己的目录（`TMPDIR` 嵌得深）时另开短目录的地方，与验证包 `crash_library` 的临时服务退到同一处。
const SHORT_SOCKET_DIRECTORY_ROOT: &str = "/tmp";
/// 那份短目录只有本用户进得去（`/tmp` 人人可写，socket 不放进别人进得去的目录）。
const SHORT_SOCKET_DIRECTORY_MODE: u32 = 0o700;
/// 测试里的临时判定库服务的 TCP 绑哪里：测试只经 socket 连，TCP 绑本机回环、端口由内核挑。
const NETWORK_ADDRESS_OF_A_TEST_SERVICE: &str = "127.0.0.1:0";
/// 这个进程里开过几份短目录、几份追加日志（名字里带序号，同一个进程里几份互不相撞）。
static OPENED_IN_THIS_PROCESS: AtomicU64 = AtomicU64::new(0);

/// 测试进程里起的一份临时判定库服务（`prune_server::server::start_in_this_process`，不写真服务）：状态目录与库在它独占的一次性临时目录里，
/// socket 也放那里，放不下时另开 `/tmp` 下的短目录。丢掉时停服、删目录。
pub struct TemporaryLibraryService {
    service: Option<ServiceInThisProcess>,
    socket: PathBuf,
    directory: ExclusiveTemporaryDirectory,
    short_socket_directory: Option<PathBuf>,
}

impl TemporaryLibraryService {
    /// # Panics
    /// 建不了目录、服务起不来。
    pub fn start(label: &str) -> Self {
        use std::os::unix::fs::DirBuilderExt;
        let directory = ExclusiveTemporaryDirectory::create(&format!("library-{label}"))
            .unwrap_or_else(|problem| panic!("{problem}"));
        let socket_in_the_directory = directory.join("socket");
        let short_socket_directory = socket_path_problem(&socket_in_the_directory).map(|_| {
            let short = Path::new(SHORT_SOCKET_DIRECTORY_ROOT).join(format!(
                "gpu-service-socket-{}-{}",
                std::process::id(),
                OPENED_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::DirBuilder::new()
                .mode(SHORT_SOCKET_DIRECTORY_MODE)
                .create(&short)
                .unwrap_or_else(|error| panic!("建不了 {}：{error}", short.display()));
            short
        });
        let socket = short_socket_directory
            .as_ref()
            .map_or(socket_in_the_directory, |short| short.join("socket"));
        let service = start_in_this_process(
            &directory.join("state"),
            &socket,
            NETWORK_ADDRESS_OF_A_TEST_SERVICE,
        )
        .unwrap_or_else(|failure| {
            let (problem, next_step) = failure.describe();
            panic!(
                "临时判定库服务起不来（{}）：{problem}；{next_step}",
                socket.display()
            )
        });
        Self {
            service: Some(service),
            socket,
            directory,
            short_socket_directory,
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// 这份服务的连法（可以交给别的线程开客户端与验证包一侧）。
    pub fn handle(&self) -> TemporaryLibraryHandle {
        TemporaryLibraryHandle {
            socket: self.socket.clone(),
            directory: self.directory.path().to_path_buf(),
        }
    }

    pub fn route(&self) -> RouteAddress {
        self.handle().route()
    }

    /// 连上这份服务的一个新容错客户端：追加日志是新建的一份，客户端身份（认领的持有者）与别的客户端都不同。
    ///
    /// # Panics
    /// 连不上。
    pub fn open_client(&self) -> ReliableClient {
        self.handle().open_client()
    }

    /// 这份服务里全仓那一份库的键空间开一份验证包的 `VerdictStore`（临时服务里那一份就是它的全仓那一份）。
    ///
    /// # Panics
    /// 连不上、开不了库。
    pub fn open_store(&self) -> VerdictStore {
        self.open_store_in(LibraryKeySpace::TheOneLibrary)
    }

    /// 同 [`Self::open_store`]，开 `key_space` 那一段键空间（登记的临时库是一段名字空间）。
    ///
    /// # Panics
    /// 连不上、开不了库。
    pub fn open_store_in(&self, key_space: LibraryKeySpace) -> VerdictStore {
        self.handle().open_store_in(key_space)
    }

    /// 起一份新的追加日志路径（测试自己开客户端、自己连服务要用）。
    pub fn journal_path_for_a_new_client(&self) -> PathBuf {
        self.handle().journal_path_for_a_new_client()
    }

    /// 这份服务此刻就停（之后连它的都连不上）；丢掉时不再停第二回。
    ///
    /// # Panics
    /// 服务没停干净。
    pub fn stop_the_service(&mut self) {
        if let Some(service) = self.service.take() {
            service
                .stop()
                .unwrap_or_else(|problem| panic!("临时判定库服务没停干净：{problem}"));
        }
    }
}

/// 一份临时判定库服务的连法：socket 与它独占的目录。只是路径，可以复制、交给别的线程；服务本身还在 [`TemporaryLibraryService`] 手里。
#[derive(Clone, Debug)]
pub struct TemporaryLibraryHandle {
    socket: PathBuf,
    directory: PathBuf,
}

impl TemporaryLibraryHandle {
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    pub fn route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.socket.clone())
    }

    /// 起一份新的追加日志路径（名字里带进程内序号，几份互不相撞）。
    pub fn journal_path_for_a_new_client(&self) -> PathBuf {
        self.directory.join(format!(
            "journal-{}",
            OPENED_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst)
        ))
    }

    /// # Panics
    /// 连不上。
    pub fn open_client(&self) -> ReliableClient {
        ReliableClient::open(ReliableClientSettings::new(
            self.route(),
            self.journal_path_for_a_new_client(),
        ))
        .unwrap_or_else(|failure| panic!("连不上临时判定库服务：{failure}"))
    }

    /// # Panics
    /// 连不上、开不了库。
    pub fn open_store_in(&self, key_space: LibraryKeySpace) -> VerdictStore {
        let storage = PruneServerStorage::connect(
            self.route(),
            self.journal_path_for_a_new_client(),
            PruneServerFailureHandling::ReturnTheError,
            key_space,
        )
        .unwrap_or_else(|error| panic!("连不上临时判定库服务：{error:?}"));
        VerdictStore::open_the_prune_server(storage)
            .unwrap_or_else(|error| panic!("开不了临时判定库服务里的库：{error:?}"))
    }
}

impl Drop for TemporaryLibraryService {
    fn drop(&mut self) {
        if let Some(service) = self.service.take() {
            if let Err(problem) = service.stop() {
                eprintln!("  ✗ 临时判定库服务没停干净：{problem}");
            }
        }
        if let Some(short) = &self.short_socket_directory {
            let _ = std::fs::remove_dir_all(short);
        }
    }
}

/// 替身结论行的键头：替身把结论字原样存成一行（验证包的结论行写法不在这里演，由验证包那一侧的测试管）。
pub const STAND_IN_CONCLUSION_PREFIX: &str = "test-conclusion/";

/// 替身结论行的键：`test-conclusion/<结论版本键的十六进制>/<块键的十六进制>`（同验证包：结论落在待判行写的那一把结论版本键下；
/// 一块一种结论一行，按结论版本键与块键找）。
pub fn stand_in_conclusion_key(row: &GpuPendingRow) -> Vec<u8> {
    stand_in_conclusion_key_in(&[], row)
}

/// 同 [`stand_in_conclusion_key`]，在 `key_prefix` 那一段键空间里（登记的临时库：前面接一段名字空间）。
pub fn stand_in_conclusion_key_in(key_prefix: &[u8], row: &GpuPendingRow) -> Vec<u8> {
    let mut key = key_prefix.to_vec();
    key.extend_from_slice(
        format!(
            "{STAND_IN_CONCLUSION_PREFIX}{}/{}",
            hexadecimal_text(&row.conclusion_version.0),
            hexadecimal_text(&row.recorded_block.to_block_key_bytes())
        )
        .as_bytes(),
    );
    key
}

/// 替身结论里记红的那个字：一个态的结论这个字不是 0 就是红（替身判器只填前两个字，判出来的都不红）。
pub const STAND_IN_RED_WORD: usize = 2;

/// 这一份替身结论红不红。
pub fn is_stand_in_conclusion_red(verdicts: &[StateVerdict]) -> bool {
    verdicts
        .iter()
        .any(|verdict| verdict[STAND_IN_RED_WORD] != 0)
}

/// 结论字的字节（每字 4 字节小端，按态的次序）。
pub fn bytes_of_the_verdicts(verdicts: &[StateVerdict]) -> Vec<u8> {
    verdicts
        .iter()
        .flat_map(|verdict| verdict.iter().flat_map(|word| word.to_le_bytes()))
        .collect()
}

/// 从替身结论行的字节读回结论字。
///
/// # Panics
/// 字节数不是整数个结论。
pub fn verdicts_of_the_bytes(bytes: &[u8]) -> Vec<StateVerdict> {
    let verdict_bytes = VERDICT_WORDS * 4;
    assert_eq!(
        bytes.len() % verdict_bytes,
        0,
        "替身结论行的字节数是整数个结论"
    );
    bytes
        .chunks(verdict_bytes)
        .map(|chunk| {
            let mut verdict = [0u32; VERDICT_WORDS];
            for (word, word_bytes) in verdict.iter_mut().zip(chunk.chunks(4)) {
                *word = u32::from_le_bytes(word_bytes.try_into().expect("切的正好是 4 字节"));
            }
            verdict
        })
        .collect()
}

/// 核对行那一帧编好之后、服务写之前插进来的别的写方（替身验证包那一侧照它演，`StandInVerificationPackageSide::facts_verifier_frame_interference`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactsVerifierFrameInterference {
    /// 不插进来。
    NoInterference,
    /// 每编好一帧，别的写方在这一块的核对键下写一份不同的核对行（每一次都不同）。
    AnotherVerifierRowEveryTime,
    /// 编好一帧之后，别的写方写下逐字节相同的核对行（别的服务核同一块核出一样的）；只插这一回。
    TheSameVerifierRowOnce,
    /// 每编好一帧，判段照同一块补写核对待判行（入池序号换一个）。
    PendingRowRewrittenEveryTime,
    /// 编好一帧之后，判段照同一块补写一次核对待判行；只插这一回。
    PendingRowRewrittenOnce,
}

/// 结论字那一帧编好之后、服务写之前插进来的别的写方（替身验证包那一侧照它演，`StandInVerificationPackageSide::packed_tables_frame_interference`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackedTablesFrameInterference {
    /// 不插进来。
    NoInterference,
    /// 每编好一帧，别的写方照验证包的写法在这张判表的键下写一份不同的结论字（每一次都不同）。
    AnotherVerdictWordsRowEveryTime,
    /// 编好一帧之后，别的写方写下逐字节相同的表 `21` 那一行（别的服务判同一张表判出一样的）；只插这一回。
    TheSameVerdictWordsRowOnce,
    /// 每编好一帧，判段照同一张判表补写判表待判行（入池序号换一个）。
    PendingRowRewrittenEveryTime,
    /// 编好一帧之后，判段照同一张判表补写一次判表待判行；只插这一回。
    PendingRowRewrittenOnce,
}

/// 替身验证包那一侧：待判行照验证包的那一份读（`VerdictStore` 的 `every_gpu_pending_row`、`read_gpu_pending_rows`，写法就是表 `1D`），
/// 别的三样是替身：按判表摘要认流（测试事先登记）；判到了哪一步看替身结论行（不在没判完、在而红是判完而红、在而不红是可复用）；
/// 编结论帧照验证包存块那一处的规矩比（相同不写、一份红整份换上、两份都不红而不同交回错），结论行是结论字原样、守卫就是那一行。
/// 数「判到了哪一步」问了几次。`written_by_another_between_the_frame_and_the_write` 放进一份结论时，下一次编帧在读完库、编好那一帧之后
/// 把它写进库（演别的写方在编帧与写帧之间插进来写同一块），只演一次。
/// 核对待判行那一族：扫、重读、核过没有、编那一帧都照验证包那一份（`every_facts_verifier_pending_row`、`read_facts_verifier_pending_rows`、
/// `facts_verifier_pending_block_is_verified_in_the_library`、`VerdictStore::facts_verifier_rows_for_one_frame`，核对键照
/// `facts_verifier_row_key` 拼），只有材料是替身（登记过一份就交它，`register_facts_material`；没登记交 [`StandInFactsVerifierMaterial`]，
/// 替身核对器不读事实表）；数读了几回材料。
/// `facts_verifier_frame_interference` 照 [`FactsVerifierFrameInterference`] 在编好核对行那一帧之后插进来写。
/// 判表待判行那一族：扫、重读、结论字在不在、编结论字那一帧都照验证包那一份（`every_packed_tables_pending_row`、`read_packed_tables_pending_row`、
/// `packed_tables_verdict_words_are_in_the_library`、`VerdictStore::packed_tables_verdict_words_rows_for_one_frame`），只有流是替身
/// （照判表摘要认登记过的那一条，同待判行那一族）；数配了几回流。`packed_tables_frame_interference` 照 [`PackedTablesFrameInterference`]
/// 在编好结论字那一帧之后插进来写。
pub struct StandInVerificationPackageSide<Materials> {
    store: VerdictStore,
    raw: ReliableClient,
    key_space: LibraryKeySpace,
    key_prefix: Vec<u8>,
    streams: BTreeMap<[u8; 32], (StreamKey, StreamFacts, Arc<Materials>)>,
    pub judging_questions: Arc<AtomicU64>,
    /// 设上之后扫待判行读不了（演这一份库读写失败：服务不可用，退出码 75）。
    pub reads_fail: Arc<AtomicBool>,
    pub written_by_another_between_the_frame_and_the_write: Arc<Mutex<Option<Vec<StateVerdict>>>>,
    pub facts_material_questions: Arc<AtomicU64>,
    pub facts_verifier_frame_interference: Arc<Mutex<FactsVerifierFrameInterference>>,
    facts_verifier_interferences: u64,
    facts_material: Option<Arc<dyn FactsVerifierMaterial>>,
    pub packed_tables_stream_questions: Arc<AtomicU64>,
    pub packed_tables_frame_interference: Arc<Mutex<PackedTablesFrameInterference>>,
    packed_tables_interferences: u64,
}

impl<Materials> StandInVerificationPackageSide<Materials> {
    pub fn on(library: &TemporaryLibraryService) -> Self {
        Self::on_key_space(&library.handle(), Vec::new())
    }

    /// 连这份服务里 `key_prefix` 那一段键空间（空的是全仓那一份；不空的是登记的临时库里的一段名字空间）：
    /// 验证包的读写经带键空间的 `VerdictStore`，替身结论行的键前面接同一段前缀。
    pub fn on_key_space(library: &TemporaryLibraryHandle, key_prefix: Vec<u8>) -> Self {
        let key_space = if key_prefix.is_empty() {
            LibraryKeySpace::TheOneLibrary
        } else {
            LibraryKeySpace::DisposableLibrary(key_prefix.clone())
        };
        Self {
            store: library.open_store_in(key_space.clone()),
            raw: library.open_client(),
            key_space,
            key_prefix,
            streams: BTreeMap::new(),
            judging_questions: Arc::new(AtomicU64::new(0)),
            reads_fail: Arc::new(AtomicBool::new(false)),
            written_by_another_between_the_frame_and_the_write: Arc::new(Mutex::new(None)),
            facts_material_questions: Arc::new(AtomicU64::new(0)),
            facts_verifier_frame_interference: Arc::new(Mutex::new(
                FactsVerifierFrameInterference::NoInterference,
            )),
            facts_verifier_interferences: 0,
            facts_material: None,
            packed_tables_stream_questions: Arc::new(AtomicU64::new(0)),
            packed_tables_frame_interference: Arc::new(Mutex::new(
                PackedTablesFrameInterference::NoInterference,
            )),
            packed_tables_interferences: 0,
        }
    }

    /// 每一行核对待判行都配这一份核对材料（上卡的对拍照真的事实表建核对器）。
    pub fn register_facts_material(&mut self, material: Arc<dyn FactsVerifierMaterial>) {
        self.facts_material = Some(material);
    }

    /// 判表摘要是 `tables_digest` 的行配这条流。
    pub fn register(
        &mut self,
        tables_digest: [u8; 32],
        stream: StreamKey,
        facts: StreamFacts,
        materials: Arc<Materials>,
    ) {
        self.streams
            .insert(tables_digest, (stream, facts, materials));
    }
}

/// 替身核对材料：一份空的事实表、没有段（替身核对器不读它们，照序号出结论位，[`stand_in_facts_bits`]）。
pub struct StandInFactsVerifierMaterial {
    facts: FlowFactsReadBack,
}

impl StandInFactsVerifierMaterial {
    /// # Panics
    /// 空的事实表编了读不回（验证包的事实表编码坏了）。
    pub fn empty() -> Self {
        let facts = FlowFacts {
            recorded_write_count: 0,
            enumerated_write_count: 0,
            is_tearable: Vec::new(),
            torn_images: Vec::new(),
            segments: Vec::new(),
            state_count: 0,
            units: Vec::new(),
            roots: Vec::new(),
            newest_base_root: None,
        };
        Self {
            facts: FlowFactsReadBack::from_the_library_bytes(&facts.to_bytes(), &facts.digest())
                .unwrap_or_else(|problem| panic!("空的事实表读不回：{problem}")),
        }
    }
}

impl FactsVerifierMaterial for StandInFactsVerifierMaterial {
    fn facts(&self) -> &FlowFactsReadBack {
        &self.facts
    }

    fn segment_expansions(&self) -> &[SegmentExpansionFact] {
        &[]
    }
}

/// 替身核对器核出的结论位：序号除以 7 的余数（几块互不相同、有 0 有非 0）。
pub fn stand_in_facts_bits(ordinals: Range<u64>) -> Vec<u32> {
    ordinals
        .map(|ordinal| u32::try_from(ordinal % 7).expect("余数装得进 u32"))
        .collect()
}

/// 往库里原样写一行（测试演别的写方）。
///
/// # Panics
/// 写不进。
pub fn write_a_raw_row(client: &mut ReliableClient, key: Vec<u8>, value: Vec<u8>) {
    client
        .submit_write_entries(&[(Table::Conclusions, key, value)])
        .and_then(|_| client.wait_all_acknowledged())
        .unwrap_or_else(|failure| panic!("那一行写不进：{failure}"));
}

/// 一行核对待判行的核对键（验证包 `facts_verifier_row_key`：只由事实表摘要定）。
pub fn facts_verifier_row_key_of(row: &FactsVerifierPendingRow) -> [u8; 32] {
    facts_verifier_row_key(&row.facts_digest)
}

/// 第 `tag` 份事实表上的一块核对待判行：名字（流键照验证包 `facts_verifier_pending_stream_key` 算）与行；块键由事实表与区间起点造，
/// 录下的流程随便填（替身不读），同一个 `tag` 的几块是同一组（同一份流程、同一份事实表）。
pub fn facts_verifier_block(
    tag: u8,
    block_number: u64,
    ordinals: Range<u64>,
    pool_entry_number: u64,
) -> (FactsVerifierBlockName, FactsVerifierPendingRow) {
    let row = FactsVerifierPendingRow {
        recorded_block: VerdictBlockKey {
            node_input_digest: NodeInputDigest([tag; 32]),
            enumeration_plan_hash: EnumerationPlanHash([0x5a; 32]),
            block_start: BlockStartStateIndex(ordinals.start),
        },
        recorded_flow_digest: [0x6b; 32],
        facts_digest: [tag; 32],
        state_ordinals: ordinals,
        pool_entry_number,
    };
    let stream = facts_verifier_pending_stream_key(&row.recorded_flow_digest, &row.facts_digest);
    (
        FactsVerifierBlockName {
            stream,
            block_number,
        },
        row,
    )
}

/// 照判段的写法（验证包 `put_the_facts_verifier_pending_row_unless_present`）写这几行核对待判行。
///
/// # Panics
/// 写不进。
pub fn put_facts_verifier_pending_rows(
    store: &VerdictStore,
    pending: &[(FactsVerifierBlockName, FactsVerifierPendingRow)],
) {
    for (name, row) in pending {
        store
            .put_the_facts_verifier_pending_row_unless_present(&name.stream, name.block_number, row)
            .unwrap_or_else(|error| panic!("核对待判行写不进临时判定库：{error:?}"));
    }
}

/// 库里此刻还在的核对待判行的名字。
///
/// # Panics
/// 扫不了。
pub fn facts_verifier_pending_names(store: &VerdictStore) -> Vec<FactsVerifierBlockName> {
    store
        .every_facts_verifier_pending_row()
        .unwrap_or_else(|error| panic!("扫不了核对待判行：{error:?}"))
        .into_iter()
        .map(|(stream, block_number, _)| FactsVerifierBlockName {
            stream,
            block_number,
        })
        .collect()
}

/// 库里此刻那一行核对待判行（不在是 `None`）。
///
/// # Panics
/// 读不了。
pub fn facts_verifier_pending_row_in_the_library(
    store: &VerdictStore,
    name: &FactsVerifierBlockName,
) -> Option<FactsVerifierPendingRow> {
    store
        .read_facts_verifier_pending_rows(&[(name.stream, name.block_number)])
        .unwrap_or_else(|error| panic!("读不了核对待判行：{error:?}"))
        .pop()
        .flatten()
}

/// 这一块在这一行的核对键下的核对行（不在是 `None`）。
///
/// # Panics
/// 读不了。
pub fn verifier_row_in_the_library(
    store: &VerdictStore,
    row: &FactsVerifierPendingRow,
) -> Option<VerifierBlockRow> {
    store
        .read_verifier_block(&row.recorded_block, &facts_verifier_row_key_of(row))
        .unwrap_or_else(|error| panic!("读不了核对行：{error:?}"))
}

/// 读回一份替身结论（不在的是 `None`）。
///
/// # Panics
/// 读不了。
pub fn stand_in_verdicts(
    client: &mut ReliableClient,
    row: &GpuPendingRow,
) -> Option<Vec<StateVerdict>> {
    stand_in_verdicts_in(client, &[], row)
}

/// 同 [`stand_in_verdicts`]，在 `key_prefix` 那一段键空间里。
///
/// # Panics
/// 读不了。
pub fn stand_in_verdicts_in(
    client: &mut ReliableClient,
    key_prefix: &[u8],
    row: &GpuPendingRow,
) -> Option<Vec<StateVerdict>> {
    client
        .read(&[(
            Table::Conclusions,
            stand_in_conclusion_key_in(key_prefix, row),
        )])
        .unwrap_or_else(|failure| panic!("读不了替身结论行：{failure}"))
        .remove(0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| verdicts_of_the_bytes(&bytes))
}

/// 把一份替身结论原样写进库（测试造「库里已有的那一份」）。
///
/// # Panics
/// 写不进。
pub fn store_stand_in_verdicts(
    client: &mut ReliableClient,
    row: &GpuPendingRow,
    verdicts: &[StateVerdict],
) {
    store_stand_in_verdicts_in(client, &[], row, verdicts);
}

/// 同 [`store_stand_in_verdicts`]，在 `key_prefix` 那一段键空间里。
///
/// # Panics
/// 写不进。
pub fn store_stand_in_verdicts_in(
    client: &mut ReliableClient,
    key_prefix: &[u8],
    row: &GpuPendingRow,
    verdicts: &[StateVerdict],
) {
    client
        .submit_write_entries(&[(
            Table::Conclusions,
            stand_in_conclusion_key_in(key_prefix, row),
            bytes_of_the_verdicts(verdicts),
        )])
        .and_then(|_| client.wait_all_acknowledged())
        .unwrap_or_else(|failure| panic!("替身结论行写不进：{failure}"));
}

impl<Materials: Send + Sync + 'static> VerificationPackageSide
    for StandInVerificationPackageSide<Materials>
{
    type Materials = Materials;

    fn key_space(&self) -> LibraryKeySpace {
        self.key_space.clone()
    }

    fn pending_rows(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, VerdictStoreError> {
        if self.reads_fail.load(Ordering::SeqCst) {
            return Err(VerdictStoreError::PruneServer(PruneServerFailure {
                exit_code: 75,
                description: "替身：这一份库读不了".to_string(),
            }));
        }
        Ok(self
            .store
            .every_gpu_pending_row()?
            .into_iter()
            .map(|(stream, block_number, row)| PendingRowOfTheLibrary {
                name: PendingBlockName {
                    stream,
                    block_number,
                },
                row,
            })
            .collect())
    }

    fn pending_row(
        &mut self,
        name: &PendingBlockName,
    ) -> Result<Option<GpuPendingRow>, VerdictStoreError> {
        Ok(self
            .store
            .read_gpu_pending_rows(&[(name.stream, name.block_number)])?
            .pop()
            .flatten())
    }

    fn stream_of_the_row(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<StreamOfTheRow<Materials>, String> {
        let (stream, facts, materials) = self
            .streams
            .get(&row.packed_judge_tables_digest)
            .ok_or_else(|| "这一行的判表摘要没登记".to_string())?;
        Ok(StreamOfTheRow {
            stream: *stream,
            facts: *facts,
            materials: Arc::clone(materials),
        })
    }

    fn judging_in_the_library(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<GpuPendingBlockInTheLibrary, VerdictStoreError> {
        self.judging_questions.fetch_add(1, Ordering::SeqCst);
        Ok(
            match stand_in_verdicts_in(&mut self.raw, &self.key_prefix, row) {
                None => GpuPendingBlockInTheLibrary::NotJudged,
                Some(stored) if is_stand_in_conclusion_red(&stored) => {
                    GpuPendingBlockInTheLibrary::JudgedRed
                }
                Some(_) => GpuPendingBlockInTheLibrary::JudgedAndReusable,
            },
        )
    }

    fn conclusion_frame(
        &mut self,
        row: &GpuPendingRow,
        verdicts: Vec<StateVerdict>,
        _card: &CardThatJudgedAPendingBlock,
    ) -> Result<ConclusionFrame, String> {
        let key = stand_in_conclusion_key_in(&self.key_prefix, row);
        let stored = self
            .raw
            .read(&[(Table::Conclusions, key.clone())])
            .map_err(|failure| failure.to_string())?
            .remove(0);
        let bytes = bytes_of_the_verdicts(&verdicts);
        let rows_to_put = match &stored {
            None => vec![(key.clone(), bytes.clone())],
            Some(same) if *same == bytes => Vec::new(),
            Some(other)
                if is_stand_in_conclusion_red(&verdicts_of_the_bytes(other))
                    || is_stand_in_conclusion_red(&verdicts) =>
            {
                vec![(key.clone(), bytes.clone())]
            }
            Some(_) => {
                return Err(
                    "库里同一块存着不同而两份都不红的结论（替身照验证包存块那一处拒）".to_string(),
                )
            }
        };
        let written_by_another = self
            .written_by_another_between_the_frame_and_the_write
            .lock()
            .expect("没有别的线程 panic 着持锁")
            .take();
        if let Some(verdicts_of_another) = written_by_another {
            store_stand_in_verdicts_in(&mut self.raw, &self.key_prefix, row, &verdicts_of_another);
        }
        Ok(ConclusionFrame {
            rows_to_put,
            keys_to_delete: Vec::new(),
            guard_key: key,
            guard_row_as_read: stored,
            conclusion_digest: sha256_digest(&bytes),
        })
    }

    fn facts_verifier_pending_rows(
        &mut self,
    ) -> Result<Vec<FactsVerifierPendingRowOfTheLibrary>, VerdictStoreError> {
        Ok(self
            .store
            .every_facts_verifier_pending_row()?
            .into_iter()
            .map(
                |(stream, block_number, row)| FactsVerifierPendingRowOfTheLibrary {
                    name: FactsVerifierBlockName {
                        stream,
                        block_number,
                    },
                    row,
                },
            )
            .collect())
    }

    fn facts_verifier_pending_row(
        &mut self,
        name: &FactsVerifierBlockName,
    ) -> Result<Option<FactsVerifierPendingRow>, VerdictStoreError> {
        Ok(self
            .store
            .read_facts_verifier_pending_rows(&[(name.stream, name.block_number)])?
            .pop()
            .flatten())
    }

    fn facts_verifier_block_is_verified(
        &mut self,
        row: &FactsVerifierPendingRow,
    ) -> Result<bool, VerdictStoreError> {
        facts_verifier_pending_block_is_verified_in_the_library(&self.store, row)
    }

    fn facts_verifier_material(
        &mut self,
        _row: &FactsVerifierPendingRow,
    ) -> Result<Arc<dyn FactsVerifierMaterial>, String> {
        self.facts_material_questions.fetch_add(1, Ordering::SeqCst);
        let material: Arc<dyn FactsVerifierMaterial> = match &self.facts_material {
            Some(registered) => Arc::clone(registered),
            None => Arc::new(StandInFactsVerifierMaterial::empty()),
        };
        Ok(material)
    }

    fn facts_verifier_frame(
        &mut self,
        name: &FactsVerifierBlockName,
        row: &FactsVerifierPendingRow,
        bits_of_each_state: &[u32],
    ) -> Result<FactsVerifierRowsForOneFrame, String> {
        let frame = self
            .store
            .facts_verifier_rows_for_one_frame(
                &name.stream,
                name.block_number,
                row,
                &facts_verifier_row_key_of(row),
                bits_of_each_state,
            )
            .map_err(|error| format!("{error:?}"))?;
        self.facts_verifier_interferences += 1;
        let interference_count = self.facts_verifier_interferences;
        let mut interference = self
            .facts_verifier_frame_interference
            .lock()
            .expect("没有别的线程 panic 着持锁");
        let rewritten_pending_row = FactsVerifierPendingRow {
            pool_entry_number: row.pool_entry_number + interference_count,
            ..row.clone()
        };
        match *interference {
            FactsVerifierFrameInterference::NoInterference => {}
            FactsVerifierFrameInterference::AnotherVerifierRowEveryTime => {
                let mut another = row.facts_digest.to_vec();
                another.extend_from_slice(
                    &u32::try_from(1000 + interference_count)
                        .expect("插进来的次数装得进 u32")
                        .to_le_bytes(),
                );
                write_a_raw_row(&mut self.raw, frame.verifier_row_key.clone(), another);
            }
            FactsVerifierFrameInterference::TheSameVerifierRowOnce => {
                if let Some((key, value)) = &frame.verifier_row_to_put {
                    write_a_raw_row(&mut self.raw, key.clone(), value.clone());
                }
                *interference = FactsVerifierFrameInterference::NoInterference;
            }
            FactsVerifierFrameInterference::PendingRowRewrittenEveryTime => {
                write_a_raw_row(
                    &mut self.raw,
                    frame.pending_row_key_to_delete.clone(),
                    rewritten_pending_row.to_value_bytes(),
                );
            }
            FactsVerifierFrameInterference::PendingRowRewrittenOnce => {
                write_a_raw_row(
                    &mut self.raw,
                    frame.pending_row_key_to_delete.clone(),
                    rewritten_pending_row.to_value_bytes(),
                );
                *interference = FactsVerifierFrameInterference::NoInterference;
            }
        }
        Ok(frame)
    }

    fn packed_tables_pending_rows(
        &mut self,
    ) -> Result<Vec<PackedTablesPendingRowOfTheLibrary>, VerdictStoreError> {
        Ok(self
            .store
            .every_packed_tables_pending_row()?
            .into_iter()
            .map(|(key, row)| PackedTablesPendingRowOfTheLibrary {
                name: PackedTablesName::of_the_key(&key),
                row,
            })
            .collect())
    }

    fn packed_tables_pending_row(
        &mut self,
        name: &PackedTablesName,
    ) -> Result<Option<PackedTablesPendingRow>, VerdictStoreError> {
        self.store
            .read_packed_tables_pending_row(&name.library_key())
    }

    fn packed_tables_verdict_words_are_in_the_library(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
    ) -> Result<bool, VerdictStoreError> {
        packed_tables_verdict_words_are_in_the_library(&self.store, &name.library_key(), row)
    }

    fn packed_tables_stream(
        &mut self,
        name: &PackedTablesName,
        _row: &PackedTablesPendingRow,
    ) -> Result<StreamOfTheRow<Materials>, String> {
        self.packed_tables_stream_questions
            .fetch_add(1, Ordering::SeqCst);
        let (stream, facts, materials) = self
            .streams
            .get(&name.packed_judge_tables_digest)
            .ok_or_else(|| "这张判表的判表摘要没登记".to_string())?;
        Ok(StreamOfTheRow {
            stream: *stream,
            facts: *facts,
            materials: Arc::clone(materials),
        })
    }

    fn packed_tables_frame(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
        verdicts: &[StateVerdict],
        device: &JudgingDevice,
    ) -> Result<PackedTablesFrame, String> {
        let key = name.library_key();
        let frame = self
            .store
            .packed_tables_verdict_words_rows_for_one_frame(&key, row, verdicts, device)
            .map_err(|error| format!("{error:?}"))?;
        let frame = PackedTablesFrame {
            rows_to_put: frame.rows_to_put,
            verdict_words_row_key: frame.verdict_words_row_key,
            verdict_words_row_as_read: frame.verdict_words_row_as_read,
            pending_row_key_to_delete: frame.pending_row_key_to_delete,
            pending_row_as_read: frame.pending_row_as_read,
        };
        self.packed_tables_interferences += 1;
        let interference_count = self.packed_tables_interferences;
        let mut interference = self
            .packed_tables_frame_interference
            .lock()
            .expect("没有别的线程 panic 着持锁");
        let rewritten_pending_row = PackedTablesPendingRow {
            pool_entry_number: row.pool_entry_number + interference_count,
            ..*row
        };
        let verdict_words_row_of = |rows: &[(Vec<u8>, Vec<u8>)]| {
            rows.iter()
                .find(|(row_key, _)| *row_key == frame.verdict_words_row_key)
                .cloned()
        };
        match *interference {
            PackedTablesFrameInterference::NoInterference => {}
            PackedTablesFrameInterference::AnotherVerdictWordsRowEveryTime => {
                let another: Vec<StateVerdict> = verdicts
                    .iter()
                    .map(|verdict| {
                        let mut changed = *verdict;
                        changed[0] = changed[0].wrapping_add(
                            u32::try_from(1000 + interference_count)
                                .expect("插进来的次数装得进 u32"),
                        );
                        changed
                    })
                    .collect();
                let another_frame = self
                    .store
                    .packed_tables_verdict_words_rows_for_one_frame(&key, row, &another, device)
                    .map_err(|error| format!("{error:?}"))?;
                let (key_of_another, value_of_another) =
                    verdict_words_row_of(&another_frame.rows_to_put)
                        .ok_or_else(|| "别的写方那一份结论字编不出表 21 那一行".to_string())?;
                write_a_raw_row(&mut self.raw, key_of_another, value_of_another);
            }
            PackedTablesFrameInterference::TheSameVerdictWordsRowOnce => {
                if let Some((same_key, same_value)) = verdict_words_row_of(&frame.rows_to_put) {
                    write_a_raw_row(&mut self.raw, same_key, same_value);
                }
                *interference = PackedTablesFrameInterference::NoInterference;
            }
            PackedTablesFrameInterference::PendingRowRewrittenEveryTime => {
                write_a_raw_row(
                    &mut self.raw,
                    frame.pending_row_key_to_delete.clone(),
                    rewritten_pending_row.to_value_bytes(),
                );
            }
            PackedTablesFrameInterference::PendingRowRewrittenOnce => {
                write_a_raw_row(
                    &mut self.raw,
                    frame.pending_row_key_to_delete.clone(),
                    rewritten_pending_row.to_value_bytes(),
                );
                *interference = PackedTablesFrameInterference::NoInterference;
            }
        }
        drop(interference);
        Ok(frame)
    }
}

/// 判表摘要每个字节都是 `tag`（与替身流键 `StreamKey { tables_digest: [tag; 32], kernel_sources_digest: [0; 32] }` 的判表摘要相同）、
/// 一张判表：名字里的旋钮摘要与一行旋钮栏都是产品那一份、态数 `state_count` 的判表待判行。
pub fn packed_tables_pending(
    tag: u8,
    state_count: u64,
    forced: ForcedRejudging,
    pool_entry_number: u64,
) -> (PackedTablesName, PackedTablesPendingRow) {
    packed_tables_pending_on_the_stream(
        StreamKey {
            tables_digest: [tag; 32],
            kernel_sources_digest: [0; 32],
        },
        state_count,
        forced,
        pool_entry_number,
    )
}

/// 同 [`packed_tables_pending`]，判表摘要取这条流的键里的判表摘要（上卡的对拍照真判表的那一条）。
pub fn packed_tables_pending_on_the_stream(
    stream: StreamKey,
    state_count: u64,
    forced: ForcedRejudging,
    pool_entry_number: u64,
) -> (PackedTablesName, PackedTablesPendingRow) {
    (
        PackedTablesName {
            packed_judge_tables_digest: stream.tables_digest,
            container_capacity_knobs_digest: container_capacity_knobs_digest(
                ContainerCapacityKnobs::PRODUCT,
            ),
        },
        PackedTablesPendingRow {
            container_capacity_knobs: ContainerCapacityKnobs::PRODUCT,
            state_count,
            forced,
            pool_entry_number,
        },
    )
}

/// 照判段的写法（验证包 `put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced`）写这几行判表待判行。
///
/// # Panics
/// 写不进。
pub fn put_packed_tables_pending_rows(
    store: &VerdictStore,
    pending: &[(PackedTablesName, PackedTablesPendingRow)],
) {
    for (name, row) in pending {
        store
            .put_the_packed_tables_pending_row_unless_present_or_raise_it_to_forced(
                &name.library_key(),
                row,
            )
            .unwrap_or_else(|error| panic!("判表待判行写不进临时判定库：{error:?}"));
    }
}

/// 库里此刻还在的判表待判行的名字。
///
/// # Panics
/// 扫不了。
pub fn packed_tables_pending_names(store: &VerdictStore) -> Vec<PackedTablesName> {
    store
        .every_packed_tables_pending_row()
        .unwrap_or_else(|error| panic!("扫不了判表待判行：{error:?}"))
        .iter()
        .map(|(key, _)| PackedTablesName::of_the_key(key))
        .collect()
}

/// 库里此刻那一行判表待判行（不在是 `None`）。
///
/// # Panics
/// 读不了。
pub fn packed_tables_pending_row_in_the_library(
    store: &VerdictStore,
    name: &PackedTablesName,
) -> Option<PackedTablesPendingRow> {
    store
        .read_packed_tables_pending_row(&name.library_key())
        .unwrap_or_else(|error| panic!("读不了判表待判行：{error:?}"))
}

/// 这张判表在表 `21` 里的结论字（不在是 `None`；验证包 `VerdictStore::read_packed_tables_verdict_words`）。
///
/// # Panics
/// 读不了。
pub fn packed_tables_verdict_words_in_the_library(
    store: &VerdictStore,
    name: &PackedTablesName,
) -> Option<Vec<StateVerdict>> {
    store
        .read_packed_tables_verdict_words(&name.library_key())
        .unwrap_or_else(|error| panic!("读不了表 21：{error:?}"))
        .map(|stored| stored.verdict_words)
}

/// 照验证包的写法把这张判表的结论字直接写进表 `21`（测试造「库里已有的那一份」）：编一帧、只写表 `21` 那一行，不删判表待判行。
///
/// # Panics
/// 编不出、写不进。
pub fn store_packed_tables_verdict_words(
    library: &TemporaryLibraryService,
    name: &PackedTablesName,
    row: &PackedTablesPendingRow,
    verdicts: &[StateVerdict],
) {
    let frame = library
        .open_store()
        .packed_tables_verdict_words_rows_for_one_frame(
            &name.library_key(),
            row,
            verdicts,
            &stand_in_judging_device(),
        )
        .unwrap_or_else(|error| panic!("编不出结论字那一帧：{error:?}"));
    let (key, value) = frame
        .rows_to_put
        .into_iter()
        .find(|(key, _)| *key == frame.verdict_words_row_key)
        .expect("库里还没有这张判表的结论字：那一帧要写表 21 那一行");
    write_a_raw_row(&mut library.open_client(), key, value);
}

/// 替身判这张判表的那张卡写进表 `21` 的设备属性（同 `packed_tables_family::judging_device_of_the_card`：卡型与驱动版本）。
pub fn stand_in_judging_device() -> JudgingDevice {
    JudgingDevice::Gpu {
        card_type: "stand-in card".to_string(),
        driver_version: "stand-in driver".to_string(),
    }
}

/// 第 `tag` 条流（验证包的流键每个字节都是 `tag`）的第 `block_number` 块。
pub fn pending_block_name(tag: u8, block_number: u64) -> PendingBlockName {
    PendingBlockName {
        stream: GpuPendingStreamKey([tag; 32]),
        block_number,
    }
}

/// 一行待判行（验证包的 `GpuPendingRow`）：块键由判表摘要与区间起点造（替身按块键找结论行，几块互不相撞），录下的流程随便填（替身不读）。
pub fn pending_row(
    tables_digest: [u8; 32],
    ordinals: Range<u64>,
    conclusion_version: [u8; 32],
    forced: ForcedRejudging,
    pool_entry_number: u64,
) -> GpuPendingRow {
    GpuPendingRow {
        recorded_block: VerdictBlockKey {
            node_input_digest: NodeInputDigest(tables_digest),
            enumeration_plan_hash: EnumerationPlanHash([0x5a; 32]),
            block_start: BlockStartStateIndex(ordinals.start),
        },
        recorded_flow_digest: [0x6b; 32],
        packed_judge_tables_digest: tables_digest,
        state_ordinals: ordinals,
        conclusion_version: ConclusionVersion(conclusion_version),
        forced,
        pool_entry_number,
    }
}

/// 验证包给判崩溃镜像那一种结论的版本键（只由结论种类定）：待判行里写它，结论就落在直判那条路也写的那一把键下。服务只照行里写的存，
/// 不核它是不是这一把（任意一把都照存）。
pub fn conclusion_version_of_the_crash_image_judgement() -> [u8; 32] {
    ConclusionVersion::of_the_crash_image_judgement().0
}

/// 照判段的写法（验证包 `put_the_gpu_pending_row_unless_present_or_raise_it_to_forced`）写这几行，交回每一行的结局。
///
/// # Panics
/// 写不进。
pub fn put_pending_rows(
    store: &VerdictStore,
    pending: &[(PendingBlockName, GpuPendingRow)],
) -> Vec<GpuPendingRowPut> {
    pending
        .iter()
        .map(|(name, row)| {
            store
                .put_the_gpu_pending_row_unless_present_or_raise_it_to_forced(
                    &name.stream,
                    name.block_number,
                    row,
                )
                .unwrap_or_else(|error| panic!("待判行写不进临时判定库：{error:?}"))
        })
        .collect()
}

/// 库里此刻还在的待判行的名字。
///
/// # Panics
/// 扫不了。
pub fn pending_names(store: &VerdictStore) -> Vec<PendingBlockName> {
    store
        .every_gpu_pending_row()
        .unwrap_or_else(|error| panic!("扫不了待判行：{error:?}"))
        .into_iter()
        .map(|(stream, block_number, _)| PendingBlockName {
            stream,
            block_number,
        })
        .collect()
}

/// 库里此刻那一行待判行（不在是 `None`）。
///
/// # Panics
/// 读不了。
pub fn pending_row_in_the_library(
    store: &VerdictStore,
    name: &PendingBlockName,
) -> Option<GpuPendingRow> {
    store
        .read_gpu_pending_rows(&[(name.stream, name.block_number)])
        .unwrap_or_else(|error| panic!("读不了待判行：{error:?}"))
        .pop()
        .flatten()
}
