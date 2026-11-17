//! 几份测试共用的：新池新建文件那条流（两块稀疏内存盘上 mkfs、取号、暖机、新建文件，录 mkfs 之后那一段）的判表，
//! 不上卡的调度测试造流、造卡、模拟判完一波的写法，与判定库那一份待判池的测试要的临时判定库服务、替身验证包一侧、造待判行的写法。

#![allow(dead_code, reason = "每份测试文件只用其中几样")]

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
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
use gpu_verification_server::library_pending_pool::{
    ConclusionFrame, PendingBlockName, PendingRowOfTheLibrary, StreamOfTheRow,
    VerificationPackageSide, VersionKeysOfThisService,
};
use gpu_verification_server::pending_pool::{InMemoryPendingPool, PendingPool, StateVerdict};
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
    CardThatJudgedAPendingBlock, GpuPendingBlockInTheLibrary,
};
use singlefs_verification::crash_identity::ConclusionVersion;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::VERDICT_WORDS;
use singlefs_verification::crash_judge_tables::{build_judge_tables, JudgeFlow, JudgeTables};
use singlefs_verification::crash_library::ExclusiveTemporaryDirectory;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, SparseBlockDevice,
};
use singlefs_verification::segments::{FixedGeometry, StepKind};
use singlefs_verification::sha256::sha256_digest;
use singlefs_verification::verdict_store::{
    BlockStartStateIndex, EnumerationPlanHash, ForcedRejudging, GpuPendingRow, GpuPendingRowPut,
    GpuPendingStreamKey, JudgingUnitsFingerprint, LibraryKeySpace, NodeInputDigest,
    PruneServerFailureHandling, PruneServerStorage, VerdictBlockKey, VerdictStore,
    VerdictStoreError,
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

fn new_pool_file_creation_judge_tables_expanding(
    expand_up_to: usize,
    segment_left_whole: Option<usize>,
) -> JudgeTables {
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
    let expansion = |segment_index: usize, segment: &[usize]| {
        if segment.len() <= expand_up_to && segment_left_whole != Some(segment_index) {
            CrashPointReplaySegmentExpansion::EveryProperSubset
        } else {
            CrashPointReplaySegmentExpansion::NotExpanded
        }
    };
    build_judge_tables(&JudgeFlow {
        base: &base,
        writes: &writes,
        segments: &segments,
        judged_root_index,
        versions: &versions,
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
            pool.record_verdicts(piece, card.card, vec![[0; VERDICT_WORDS]; states])
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
        let service =
            start_in_this_process(&directory.join("state"), &socket).unwrap_or_else(|failure| {
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

    pub fn route(&self) -> RouteAddress {
        RouteAddress::UnixSocket(self.socket.clone())
    }

    fn new_journal_path(&self) -> PathBuf {
        self.directory.join(format!(
            "journal-{}",
            OPENED_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst)
        ))
    }

    /// 连上这份服务的一个新容错客户端：追加日志是新建的一份，客户端身份（认领的持有者）与别的客户端都不同。
    ///
    /// # Panics
    /// 连不上。
    pub fn open_client(&self) -> ReliableClient {
        ReliableClient::open(ReliableClientSettings::new(
            self.route(),
            self.new_journal_path(),
        ))
        .unwrap_or_else(|failure| panic!("连不上临时判定库服务：{failure}"))
    }

    /// 这份服务里全仓那一份库的键空间开一份验证包的 `VerdictStore`（服务只连全仓那一份，临时服务里那一份就是它的全仓那一份）。
    ///
    /// # Panics
    /// 连不上、开不了库。
    pub fn open_store(&self) -> VerdictStore {
        let storage = PruneServerStorage::connect(
            self.route(),
            self.new_journal_path(),
            PruneServerFailureHandling::ReturnTheError,
            LibraryKeySpace::TheOneLibrary,
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
    format!(
        "{STAND_IN_CONCLUSION_PREFIX}{}/{}",
        hexadecimal_text(&row.conclusion_version.0),
        hexadecimal_text(&row.recorded_block.to_block_key_bytes())
    )
    .into_bytes()
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

/// 替身验证包那一侧：待判行照验证包的那一份读（`VerdictStore` 的 `every_gpu_pending_row`、`read_gpu_pending_rows`，写法就是表 `1D`），
/// 别的三样是替身：按判表摘要认流（测试事先登记）；判到了哪一步看替身结论行（不在没判完、在而红是判完而红、在而不红是可复用）；
/// 编结论帧照验证包存块那一处的规矩比（相同不写、一份红整份换上、两份都不红而不同交回错），结论行是结论字原样、守卫就是那一行。
/// 数「判到了哪一步」问了几次。`written_by_another_between_the_frame_and_the_write` 放进一份结论时，下一次编帧在读完库、编好那一帧之后
/// 把它写进库（演别的写方在编帧与写帧之间插进来写同一块），只演一次。
pub struct StandInVerificationPackageSide<Materials> {
    store: VerdictStore,
    raw: ReliableClient,
    streams: BTreeMap<[u8; 32], (StreamKey, StreamFacts, Arc<Materials>)>,
    pub judging_questions: Arc<AtomicU64>,
    pub written_by_another_between_the_frame_and_the_write: Arc<Mutex<Option<Vec<StateVerdict>>>>,
}

impl<Materials> StandInVerificationPackageSide<Materials> {
    pub fn on(library: &TemporaryLibraryService) -> Self {
        Self {
            store: library.open_store(),
            raw: library.open_client(),
            streams: BTreeMap::new(),
            judging_questions: Arc::new(AtomicU64::new(0)),
            written_by_another_between_the_frame_and_the_write: Arc::new(Mutex::new(None)),
        }
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

/// 读回一份替身结论（不在的是 `None`）。
///
/// # Panics
/// 读不了。
pub fn stand_in_verdicts(
    client: &mut ReliableClient,
    row: &GpuPendingRow,
) -> Option<Vec<StateVerdict>> {
    client
        .read(&[(Table::Conclusions, stand_in_conclusion_key(row))])
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
    client
        .submit_write_entries(&[(
            Table::Conclusions,
            stand_in_conclusion_key(row),
            bytes_of_the_verdicts(verdicts),
        )])
        .and_then(|_| client.wait_all_acknowledged())
        .unwrap_or_else(|failure| panic!("替身结论行写不进：{failure}"));
}

impl<Materials: Send + Sync + 'static> VerificationPackageSide
    for StandInVerificationPackageSide<Materials>
{
    type Materials = Materials;

    fn pending_rows(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, VerdictStoreError> {
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
        Ok(match stand_in_verdicts(&mut self.raw, row) {
            None => GpuPendingBlockInTheLibrary::NotJudged,
            Some(stored) if is_stand_in_conclusion_red(&stored) => {
                GpuPendingBlockInTheLibrary::JudgedRed
            }
            Some(_) => GpuPendingBlockInTheLibrary::JudgedAndReusable,
        })
    }

    fn conclusion_frame(
        &mut self,
        row: &GpuPendingRow,
        verdicts: Vec<StateVerdict>,
        _card: &CardThatJudgedAPendingBlock,
    ) -> Result<ConclusionFrame, String> {
        let key = stand_in_conclusion_key(row);
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
            store_stand_in_verdicts(&mut self.raw, row, &verdicts_of_another);
        }
        Ok(ConclusionFrame {
            rows_to_put,
            keys_to_delete: Vec::new(),
            guard_key: key,
            guard_row_as_read: stored,
            conclusion_digest: sha256_digest(&bytes),
        })
    }
}

/// 一行待判行期待的两把键：GPU 判器组键与结论版本键。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpectedVersionKeys {
    pub gpu_judge: [u8; 32],
    pub conclusion_version: [u8; 32],
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
    expected: ExpectedVersionKeys,
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
        expected_gpu_judge_units: JudgingUnitsFingerprint(expected.gpu_judge),
        conclusion_version: ConclusionVersion(expected.conclusion_version),
        forced,
        pool_entry_number,
    }
}

/// 编进这个测试二进制的两把键（期待的键照它填就对得上）。
///
/// # Panics
/// 判器键表里算不出。
pub fn version_keys_of_this_build() -> VersionKeysOfThisService {
    VersionKeysOfThisService::compiled_into_this_service()
        .unwrap_or_else(|problem| panic!("{problem}"))
}

/// 对得上这个二进制的期待的键（结论版本取判崩溃镜像那一把）。
pub fn matching_expected_keys(version_keys: &VersionKeysOfThisService) -> ExpectedVersionKeys {
    ExpectedVersionKeys {
        gpu_judge: version_keys.gpu_judge,
        conclusion_version: version_keys.conclusion_versions[0],
    }
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
