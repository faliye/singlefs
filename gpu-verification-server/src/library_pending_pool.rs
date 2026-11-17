//! 待判池的判定库那一份：从判定库里取待判行、认领、交给服务主循环判，判完在一帧里写回结论、删掉待判行（强制现判的另写凭据行）。
//! 待判行的键与值、强制现判凭据的键与值、在库上现算判完、读回装判器的材料、把结论字编成结论帧，一律照验证包那一套
//! （`singlefs_verification::verdict_store` 的表 `1D`、`1E` 与 `crash_amplification` 交出的三个函数），服务不另写一份；
//! 验证包那一侧经接口 [`VerificationPackageSide`]（真的那一份在 `crate::verification_package_side`，不上卡的测试换成替身）。
//!
//! 进主循环的块照旧走内存里的待判池（[`crate::pending_pool::InMemoryPendingPool`]，批次调度只看它）；这一份在主循环外面当它的客户端：
//! 取活分两步，先挑（[`LibraryPendingPool::candidates`]：扫库、对版本、现算判完），再认领（[`LibraryPendingPool::claim_the_candidates`]）。
//!
//! - 版本对账：起服务时算出编进这个二进制的 GPU 判器组键与结论版本键（[`VersionKeysOfThisService::compiled_into_this_service`]）。
//!   认不认领只核 GPU 判器组键（服务真在跑的只有 GPU 判器那一组）：待判行里期待的 GPU 判器组键不同的不认领、留给编出对的键的那一份服务，
//!   每一行只点名一次（流键、块号、两边的键，[`VERSION_MISMATCH_LINE_PREFIX`]；这个二进制算得出的结论版本键只打进那一行给人看）。
//!   结论版本键不核：它跟着编那一份二进制时依赖包开的特性走（判段的测试二进制与服务的 GPU 判器组键相同、结论版本键可以不同），
//!   结论照待判行里写的那一把结论版本键存（验证包 `conclusion_frame_of_a_gpu_pending_block` 照行里的键编帧）。
//! - 判完：不是强制现判的块，挑之前在库上现算（验证包 `judging_of_a_gpu_pending_block_in_the_library`）：没判完的、判完而红的照取；
//!   判完而可复用的不取，带守卫删掉那一行待判行（判段等的是「行不在且续点口径判完」，行留着判段就一直等）。强制现判的块一律现判，不问库。
//! - 认领：键是判定库服务登记的认领键（`prune_server::gpu_judging_rows::gpu_claim_key`，只记认领、不当数据行写）；
//!   别的持有者持着、租期没到的不取；认领到的进「在判」，判着的时候每过三分之一租期续一次（[`LibraryPendingPool::renew_the_claims`]）。
//! - 写回（[`LibraryPendingPool::write_back`]）：一帧判定库服务的写结论（`prune_server::protocol` 文件头「写结论」）：核认领、核守卫那一行
//!   （结论块那一行，期待是编帧之前读到的那一份）、核删键（待判行的期待是这个服务读到的那一份：判段读完之后把它升成强制现判的不删）。
//!   守卫不过照种类办：认领被别人持着不写；守卫那一行变了就重编（验证包照判段存块的规矩重读、重比：两份都绿而不同交回错、报红，
//!   一份红整份换上）再写；待判行变了就重读那一行：还是同一块的照重读到的那一份再写（升成强制现判的带上凭据行），不在了的不再删它。
//!   这一帧没写成（被别人持着、编不成、写库失败、守卫不过 [`CONCLUSION_WRITE_ATTEMPTS`] 次）不丢结论：结论字留在 [`LibraryPendingPool::refused_writes`]，
//!   打一行 [`CONCLUSION_WRITE_REFUSED_LINE_PREFIX`] 报红，待判行留在库里。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use prune_server::gpu_judging_rows::gpu_claim_key;
use prune_server::protocol::{
    ClaimOutcome, GuardRefusal, GuardRowState, GuardedConclusionWrite, GuardedDeletion, Table,
};
use prune_server::reliable_client::{ClientFailure, GuardedConclusionWriteOutcome, ReliableClient};
use prune_server::segmented_values::SEGMENT_BYTES;
use singlefs_verification::crash_amplification::{
    CardThatJudgedAPendingBlock, GpuPendingBlockInTheLibrary,
};
use singlefs_verification::crash_identity::{
    image_judgement_units_key, judgement_units_key_with_the_gpu_judge, judging_units_key,
    ConclusionVersion, JudgingUnits, JudgingUnitsKeyRefused,
};
use singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    key_bytes_of_the_gpu_pending_row_in_the_one_library,
    key_bytes_of_the_gpu_rejudged_row_in_the_one_library, ForcedRejudging, GpuPendingRow,
    GpuPendingStreamKey, GpuRejudgedRow, PruneServerFailure, VerdictStoreError,
};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::pending_pool::StateVerdict;
use crate::stream_identity::{StreamFacts, StreamKey};

/// 版本对不上、不认领的那一行的开头。
pub const VERSION_MISMATCH_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_VERSION_MISMATCH";
/// 结论那一帧没写成、结论留在服务里报红的那一行的开头。
pub const CONCLUSION_WRITE_REFUSED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_CONCLUSION_WRITE_REFUSED";
/// 待判行扫不了、或验证包一侧答不了它的那一行的开头（每行只打一次）。
pub const PENDING_ROW_SKIPPED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_PENDING_ROW_SKIPPED";
/// 判完而可复用的块，待判行带守卫删掉了的那一行的开头。
pub const REUSABLE_BLOCK_ROW_DELETED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_REUSABLE_BLOCK_ROW_DELETED";
/// 写结论那一帧守卫不过（守卫那一行或删键那一行读完之后变了）之后重编、重读再写，一块至多写几次。
/// 取 4 是推的：每一次守卫不过都是这一块在编帧与写帧之间被别人改过，连着四次被改是另有一方在反复写它，报红交人看。
pub const CONCLUSION_WRITE_ATTEMPTS: u32 = 4;

/// 编进这个二进制的两组键：GPU 判器组键（认不认领只核它），与这个二进制算得出的结论版本键（判崩溃镜像那一把、判给定状态那一把；只打进版本对不上那一行）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionKeysOfThisService {
    pub gpu_judge: [u8; DIGEST_BYTES],
    pub conclusion_versions: Vec<[u8; DIGEST_BYTES]>,
}

impl VersionKeysOfThisService {
    /// 照编进二进制的判器键表算（`singlefs_verification::crash_identity::judging_units_key`：读编译时算好的那份表，不读盘上的树）。
    ///
    /// # Errors
    /// GPU 判器那一组、判崩溃镜像或判给定状态那一组在编译时拒算或没编进来：这一份服务认不出自己判的是哪一版，不起。
    pub fn compiled_into_this_service() -> Result<Self, String> {
        // judging_units_key 这一版不读工具链（工具链在编译时已进键，那个函数的文档），空的这一份只为凑签名
        let toolchain = CrashPointReplayToolchainIdentity {
            rustc_version_lines: String::new(),
            cargo_version: String::new(),
            target_triple: String::new(),
        };
        let refused = |refusal: JudgingUnitsKeyRefused| {
            format!(
                "编进这个二进制的判器键表里算不出这一把：{}",
                refusal.sentence()
            )
        };
        let gpu_judge = judging_units_key(&JudgingUnits::GpuJudge, &toolchain).map_err(refused)?;
        let crash_images = image_judgement_units_key(&toolchain).map_err(refused)?;
        let given_states_definition =
            judging_units_key(&JudgingUnits::JudgementOfTheGivenStates, &toolchain)
                .map_err(refused)?;
        let given_states = judgement_units_key_with_the_gpu_judge(
            &JudgingUnits::JudgementOfTheGivenStates,
            &given_states_definition,
            &gpu_judge,
        );
        Ok(Self {
            gpu_judge: *gpu_judge.fingerprint(),
            conclusion_versions: vec![*crash_images.fingerprint(), *given_states.fingerprint()],
        })
    }

    /// 这一行待判行这份服务认不认领：期待的 GPU 判器组键与编进来的相同就认（文件头「版本对账」：结论版本键不核）。
    #[must_use]
    pub fn accepts(&self, row: &GpuPendingRow) -> bool {
        row.expected_gpu_judge_units.0 == self.gpu_judge
    }

    /// 期待的结论版本键是不是这个二进制算得出的那几把之一（只给弄坏开关 `conclusion-version-checked` 用：认领不看它）。
    #[must_use]
    pub fn computes_the_conclusion_version_of(&self, row: &GpuPendingRow) -> bool {
        self.conclusion_versions.contains(&row.conclusion_version.0)
    }
}

/// 一块待判的名字：哪条流（验证包的流键）的第几块。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingBlockName {
    pub stream: GpuPendingStreamKey,
    pub block_number: u64,
}

impl PendingBlockName {
    /// 这一块的待判行在全仓那一份判定库里的键（验证包的表 `1D`）。
    #[must_use]
    pub fn pending_key(&self) -> Vec<u8> {
        key_bytes_of_the_gpu_pending_row_in_the_one_library(&self.stream, self.block_number)
    }

    /// 这一块的认领键（判定库服务登记的认领键：只记认领，不当数据行写）。
    #[must_use]
    pub fn claim_key(&self) -> Vec<u8> {
        gpu_claim_key(&self.stream.0, self.block_number)
    }

    /// 这一块的强制现判凭据在全仓那一份判定库里的键（验证包的表 `1E`）。
    #[must_use]
    pub fn rejudged_key(&self, pool_entry_number: u64) -> Vec<u8> {
        key_bytes_of_the_gpu_rejudged_row_in_the_one_library(
            pool_entry_number,
            &self.stream,
            self.block_number,
        )
    }
}

impl fmt::Display for PendingBlockName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "stream={} block={}",
            hexadecimal_text(&self.stream.0),
            self.block_number
        )
    }
}

/// 库里的一行待判行：名字与验证包认回的行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingRowOfTheLibrary {
    pub name: PendingBlockName,
    pub row: GpuPendingRow,
}

/// 一块的流：服务主循环登记它要的键、事实与装判器的材料。
pub struct StreamOfTheRow<Materials> {
    pub stream: StreamKey,
    pub facts: StreamFacts,
    pub materials: Arc<Materials>,
}

/// 一块的结论编成的一帧（验证包 `conclusion_frame_of_a_gpu_pending_block` 编的那一帧里写结论要的几样）：要写的行（守卫那一行在的话排最后）、
/// 一份红整份换上时要删的旧行、守卫那一行的键与编帧之前读到的那一份、结论摘要（强制现判凭据里的那一个）。键都是全仓那一份库里的整把键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConclusionFrame {
    pub rows_to_put: Vec<(Vec<u8>, Vec<u8>)>,
    pub keys_to_delete: Vec<Vec<u8>>,
    pub guard_key: Vec<u8>,
    pub guard_row_as_read: Option<Vec<u8>>,
    pub conclusion_digest: [u8; DIGEST_BYTES],
}

/// 验证包那一侧：扫与重读待判行、读回装判器的材料、在库上现算判到了哪一步、把结论字编成结论帧。全照验证包那一套，服务不另写一份。
pub trait VerificationPackageSide {
    type Materials: Send + Sync + 'static;

    /// 库里全部待判行（验证包 `VerdictStore::every_gpu_pending_row`）。
    ///
    /// # Errors
    /// 读库失败；库里有一行不合布局。
    fn pending_rows(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, VerdictStoreError>;

    /// 重读一行待判行（不在是 `None`）。
    ///
    /// # Errors
    /// 读库失败；那一行不合布局。
    fn pending_row(
        &mut self,
        name: &PendingBlockName,
    ) -> Result<Option<GpuPendingRow>, VerdictStoreError>;

    /// 这一行的流：读回判表、配上内核源码。
    ///
    /// # Errors
    /// 读不回、摘要对不上：交回为什么。
    fn stream_of_the_row(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<StreamOfTheRow<Self::Materials>, String>;

    /// 这一块在库里判到了哪一步。
    ///
    /// # Errors
    /// 读库失败（不当成没判完）。
    fn judging_in_the_library(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<GpuPendingBlockInTheLibrary, VerdictStoreError>;

    /// 把这一块的结论字（按态序号次序）连同判它的卡编成一帧（不写库：照库里此刻那一份比，结论相同不写结论行、一份红整份换上）。
    ///
    /// # Errors
    /// 编不成（有判不了的态、材料对不上、库里同一把键下存着不同而两份都不红的结论、读库失败）：交回为什么。
    fn conclusion_frame(
        &mut self,
        row: &GpuPendingRow,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> Result<ConclusionFrame, String>;
}

/// 认领到、交给主循环判的一块。
pub struct TakenBlock<Materials> {
    pub pending: PendingRowOfTheLibrary,
    pub stream: StreamOfTheRow<Materials>,
}

/// 一轮认领的结局。
pub struct ClaimRound<Materials> {
    pub taken: Vec<TakenBlock<Materials>>,
    /// 别的持有者持着、这一轮不取的（名字、持有者身份）。
    pub held_by_others: Vec<(PendingBlockName, Vec<u8>)>,
}

/// 结论那一帧没写成的一块：结论字留在这里，不丢。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedConclusionWrite {
    pub name: PendingBlockName,
    pub verdicts: Vec<StateVerdict>,
    pub reason: String,
}

/// 写回成了的一块。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WrittenBack {
    pub conclusion_digest: [u8; DIGEST_BYTES],
    pub wrote_the_rejudged_row: bool,
}

/// 读写判定库失败（连不上、忙到上限、服务停了这一类）：主循环照它退出，退出码照判定库客户端的表（75 服务不可用、3 别的错）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryFailure {
    pub exit_code: i32,
    pub description: String,
}

impl From<ClientFailure> for LibraryFailure {
    fn from(failure: ClientFailure) -> Self {
        Self {
            exit_code: failure.exit_code(),
            description: failure.to_string(),
        }
    }
}

impl From<PruneServerFailure> for LibraryFailure {
    fn from(failure: PruneServerFailure) -> Self {
        Self {
            exit_code: failure.exit_code,
            description: failure.description,
        }
    }
}

impl fmt::Display for LibraryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}（退出码 {}）",
            self.description, self.exit_code
        )
    }
}

/// 验证包交回的错里，读写判定库不成的那一种是 [`LibraryFailure`]（主循环照它退出），别的（库里的行不合布局这类）交回说明，由调用方点名、跳过。
fn library_failure_or_the_reason(error: VerdictStoreError) -> Result<String, LibraryFailure> {
    match error {
        VerdictStoreError::PruneServer(failure) => Err(LibraryFailure::from(failure)),
        other @ (VerdictStoreError::StoredBytesDoNotMatchTheLayout { .. }
        | VerdictStoreError::EveryU32VerdictVectorNumberIsAlreadyRegistered
        | VerdictStoreError::LibraryLayoutIsNotThisVersion { .. }
        | VerdictStoreError::DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion { .. }
        | VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion { .. }
        | VerdictStoreError::ObserverWitnessBatchRefusedByTheTestSwitch) => Ok(format!("{other:?}")),
    }
}

/// 认领一块的结局。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimAnswer {
    /// 这个持有者持着（新认领到，或自己持着、续上了）。
    Claimed,
    /// 别的持有者持着、租期没到。
    HeldByOther {
        holder: Vec<u8>,
        remaining_lease_milliseconds: u64,
    },
    /// 库里有认领那把键（认领键不当数据行写，出现就是别处写坏了）。
    ClaimKeyWritten,
}

/// 以 `client` 的身份认领（或续上）这一块。
///
/// # Errors
/// 读写失败；回的结局条数不是 1。
pub fn claim_a_pending_block(
    client: &mut ReliableClient,
    name: &PendingBlockName,
    lease_milliseconds: u32,
) -> Result<ClaimAnswer, ClientFailure> {
    let mut outcomes = client.claim(
        &[(Table::Conclusions, name.claim_key())],
        lease_milliseconds,
    )?;
    if outcomes.len() != 1 {
        return Err(ClientFailure::Protocol(format!(
            "认领一把键回了 {} 个结局",
            outcomes.len()
        )));
    }
    Ok(match outcomes.remove(0) {
        ClaimOutcome::Claimed => ClaimAnswer::Claimed,
        ClaimOutcome::HeldByOther {
            holder_identity,
            remaining_lease_milliseconds,
        } => ClaimAnswer::HeldByOther {
            holder: holder_identity,
            remaining_lease_milliseconds,
        },
        ClaimOutcome::AlreadyConcluded => ClaimAnswer::ClaimKeyWritten,
    })
}

/// 重读到的待判行指的还是不是同一块（强制现判与入池序号之外逐项相同）：判段把不强制的行升成强制现判时只换这两样。
fn is_the_same_block(reread: &GpuPendingRow, judged: &GpuPendingRow) -> bool {
    reread.recorded_block == judged.recorded_block
        && reread.recorded_flow_digest == judged.recorded_flow_digest
        && reread.packed_judge_tables_digest == judged.packed_judge_tables_digest
        && reread.state_ordinals == judged.state_ordinals
        && reread.expected_gpu_judge_units == judged.expected_gpu_judge_units
        && reread.conclusion_version == judged.conclusion_version
}

/// 认领到、在判的一块。
struct ClaimedBlock {
    row: GpuPendingRow,
    renewed_at: Instant,
}

/// 待判池的判定库那一份。
pub struct LibraryPendingPool<Side> {
    side: Side,
    /// 认领与写结论那一帧走的容错客户端：它的身份就是认领的持有者（一个服务进程一份）。
    claims: ReliableClient,
    version_keys: VersionKeysOfThisService,
    claim_lease_milliseconds: u32,
    break_switches: BreakSwitches,
    claimed: BTreeMap<PendingBlockName, ClaimedBlock>,
    /// 点名过的（版本对不上、扫不了、验证包一侧答不了、可复用块的行没删成）：每样只打一次。
    named: BTreeSet<Vec<u8>>,
    named_lines: Vec<String>,
    /// 认领到了却配不出流的块：这个进程里不再挑它（不然每一圈都为它认领一回），重起服务再试。
    streams_not_configured: BTreeSet<PendingBlockName>,
    refused_writes: Vec<RefusedConclusionWrite>,
    reusable_rows_deleted: u64,
}

fn version_mismatch_line(
    pending: &PendingRowOfTheLibrary,
    version_keys: &VersionKeysOfThisService,
) -> String {
    let this_conclusion_versions: Vec<String> = version_keys
        .conclusion_versions
        .iter()
        .map(|key| hexadecimal_text(&key[..]))
        .collect();
    format!(
        "{VERSION_MISMATCH_LINE_PREFIX} {} expected_gpu_judge={} this_gpu_judge={} expected_conclusion_version={} this_conclusion_versions={}",
        pending.name,
        hexadecimal_text(&pending.row.expected_gpu_judge_units.0),
        hexadecimal_text(&version_keys.gpu_judge),
        hexadecimal_text(&pending.row.conclusion_version.0),
        this_conclusion_versions.join(",")
    )
}

/// 名字空间之外的一把点名键：扫不了待判行那一样只打一次。
const SCAN_PROBLEM_NAMED_KEY: &[u8] = b"pending-rows-scan";

impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    #[must_use]
    pub fn new(
        side: Side,
        claims: ReliableClient,
        version_keys: VersionKeysOfThisService,
        claim_lease_milliseconds: u32,
        break_switches: BreakSwitches,
    ) -> Self {
        Self {
            side,
            claims,
            version_keys,
            claim_lease_milliseconds,
            break_switches,
            claimed: BTreeMap::new(),
            named: BTreeSet::new(),
            named_lines: Vec::new(),
            streams_not_configured: BTreeSet::new(),
            refused_writes: Vec::new(),
            reusable_rows_deleted: 0,
        }
    }

    /// 认领的持有者身份（认领与写结论那一个客户端的身份）。
    #[must_use]
    pub fn holder(&self) -> Vec<u8> {
        self.claims.client_identity().to_vec()
    }

    /// 结论那一帧没写成、留在服务里的块。
    #[must_use]
    pub fn refused_writes(&self) -> &[RefusedConclusionWrite] {
        &self.refused_writes
    }

    /// 认领着、在判的块数。
    #[must_use]
    pub fn claimed_count(&self) -> usize {
        self.claimed.len()
    }

    /// 判完而可复用、待判行带守卫删掉了的块数。
    #[must_use]
    pub fn reusable_rows_deleted(&self) -> u64 {
        self.reusable_rows_deleted
    }

    /// 点名过的那几行，按打出的先后。
    #[must_use]
    pub fn named_lines(&self) -> &[String] {
        &self.named_lines
    }

    fn name_once(&mut self, key: Vec<u8>, line: String) {
        if self.named.insert(key) {
            eprintln!("{line}");
            self.named_lines.push(line);
        }
    }

    /// 扫库挑能取的块（不认领）：版本对不上的、已在判的不挑；不是强制现判的在库上现算判到了哪一步，判完而可复用的不挑、带守卫删掉它的待判行。
    /// 按键排。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn candidates(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, LibraryFailure> {
        let scanned = match self.side.pending_rows() {
            Ok(scanned) => scanned,
            Err(error) => {
                let reason = library_failure_or_the_reason(error)?;
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} 扫不了待判行（库里有一行不合验证包的布局）：{reason}"
                );
                self.name_once(SCAN_PROBLEM_NAMED_KEY.to_vec(), line);
                return Ok(Vec::new());
            }
        };
        let mut candidates = Vec::new();
        for pending in scanned {
            if self.claimed.contains_key(&pending.name)
                || self.streams_not_configured.contains(&pending.name)
            {
                continue;
            }
            let accepted = self.version_keys.accepts(&pending.row)
                && !(self
                    .break_switches
                    .is_on(BreakSwitch::ConclusionVersionChecked)
                    && !self
                        .version_keys
                        .computes_the_conclusion_version_of(&pending.row));
            if !accepted
                && !self
                    .break_switches
                    .is_on(BreakSwitch::VersionMismatchIgnored)
            {
                let line = version_mismatch_line(&pending, &self.version_keys);
                self.name_once(pending.name.pending_key(), line);
                continue;
            }
            let reads_the_library = pending.row.forced == ForcedRejudging::NotForced
                || self
                    .break_switches
                    .is_on(BreakSwitch::ForcedBlockReadsTheLibrary);
            if reads_the_library {
                match self.side.judging_in_the_library(&pending.row) {
                    Ok(GpuPendingBlockInTheLibrary::JudgedAndReusable) => {
                        if !self
                            .break_switches
                            .is_on(BreakSwitch::ReusableBlockPendingRowKept)
                        {
                            self.delete_the_pending_row_of_a_reusable_block(&pending)?;
                        }
                        continue;
                    }
                    Ok(
                        GpuPendingBlockInTheLibrary::NotJudged
                        | GpuPendingBlockInTheLibrary::JudgedRed,
                    ) => {}
                    Err(error) => {
                        let reason = library_failure_or_the_reason(error)?;
                        let line = format!(
                            "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 判到了哪一步现算不了：{reason}",
                            pending.name
                        );
                        self.name_once(pending.name.pending_key(), line);
                        continue;
                    }
                }
            }
            candidates.push(pending);
        }
        Ok(candidates)
    }

    /// 判完而可复用的块：一帧写结论只删它的待判行，删键的期待是这里读到的那一份（判段读完之后升成强制现判的不删，下一轮照强制现判取）；
    /// 别的持有者持着的不删（归持有者判）。这一帧没有结论行：守卫取认领键（认领键只记认领、从不当数据行写，样子恒为「不在」），
    /// 核的是删键与认领。
    fn delete_the_pending_row_of_a_reusable_block(
        &mut self,
        pending: &PendingRowOfTheLibrary,
    ) -> Result<(), LibraryFailure> {
        let claim_key = pending.name.claim_key();
        let request = GuardedConclusionWrite {
            holder_identity: self.holder(),
            claim_key: claim_key.clone(),
            guard_key: claim_key,
            guard_expectation: GuardRowState::Absent,
            entries: Vec::new(),
            deleted_keys: vec![GuardedDeletion {
                key: pending.name.pending_key(),
                expectation: GuardRowState::of_stored_value(Some(&pending.row.to_value_bytes())),
            }],
        };
        match self.claims.write_guarded_conclusion(&request)? {
            GuardedConclusionWriteOutcome::Written(_) => {
                self.reusable_rows_deleted += 1;
                eprintln!("{REUSABLE_BLOCK_ROW_DELETED_LINE_PREFIX} {}", pending.name);
            }
            GuardedConclusionWriteOutcome::Refused { refusal, message } => {
                let line = format!(
                    "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 判完而可复用、待判行这一回没删（{refusal:?}：{message}）：下一轮照它此刻的样子再定",
                    pending.name
                );
                self.name_once(pending.name.claim_key(), line);
            }
        }
        Ok(())
    }

    /// 认领挑出来的块：认领到的配上流交回；别的持有者持着的不取；配不出流的不取（点名一次，认领留到租期过了，这个进程里不再挑它）。
    ///
    /// # Errors
    /// 认领时读写判定库失败。
    pub fn claim_the_candidates(
        &mut self,
        candidates: Vec<PendingRowOfTheLibrary>,
    ) -> Result<ClaimRound<Side::Materials>, LibraryFailure> {
        let mut round = ClaimRound {
            taken: Vec::new(),
            held_by_others: Vec::new(),
        };
        for pending in candidates {
            if self.claimed.contains_key(&pending.name) {
                continue;
            }
            match claim_a_pending_block(
                &mut self.claims,
                &pending.name,
                self.claim_lease_milliseconds,
            )? {
                ClaimAnswer::Claimed => {}
                ClaimAnswer::HeldByOther { holder, .. }
                    if !self
                        .break_switches
                        .is_on(BreakSwitch::ClaimHeldByOthersIgnored) =>
                {
                    round.held_by_others.push((pending.name, holder));
                    continue;
                }
                ClaimAnswer::HeldByOther { .. } => {}
                ClaimAnswer::ClaimKeyWritten => {
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 库里有认领那把键（认领键不当数据行写）：不取",
                        pending.name
                    );
                    self.name_once(pending.name.claim_key(), line);
                    continue;
                }
            }
            let stream = match self.side.stream_of_the_row(&pending.row) {
                Ok(stream) => stream,
                Err(reason) => {
                    let line = format!(
                        "{PENDING_ROW_SKIPPED_LINE_PREFIX} {} 配不出流（这个进程里不再取它，认领到租期过了自己失效）：{reason}",
                        pending.name
                    );
                    self.name_once(pending.name.pending_key(), line);
                    self.streams_not_configured.insert(pending.name);
                    continue;
                }
            };
            self.claimed.insert(
                pending.name,
                ClaimedBlock {
                    row: pending.row.clone(),
                    renewed_at: Instant::now(),
                },
            );
            round.taken.push(TakenBlock { pending, stream });
        }
        Ok(round)
    }

    /// 在判的块里上一次续认领过了三分之一租期的，续一次；续不上（别的持有者持着）的交回名字，块照判，写回时那一帧核认领、不写。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn renew_the_claims(&mut self) -> Result<Vec<PendingBlockName>, LibraryFailure> {
        let renew_after = Duration::from_millis(u64::from(self.claim_lease_milliseconds) / 3);
        let due: Vec<PendingBlockName> = self
            .claimed
            .iter()
            .filter(|(_, claimed)| claimed.renewed_at.elapsed() >= renew_after)
            .map(|(name, _)| *name)
            .collect();
        let mut lost = Vec::new();
        for name in due {
            let answer =
                claim_a_pending_block(&mut self.claims, &name, self.claim_lease_milliseconds)?;
            if answer == ClaimAnswer::Claimed {
                if let Some(claimed) = self.claimed.get_mut(&name) {
                    claimed.renewed_at = Instant::now();
                }
            } else {
                lost.push(name);
            }
        }
        Ok(lost)
    }

    /// 主循环把这一块撤了（判它的卡坏了）：不再当在判，认领留到租期过了，下一轮扫到再取。
    pub fn withdraw(&mut self, name: &PendingBlockName) {
        self.claimed.remove(name);
    }

    fn refuse(
        &mut self,
        name: PendingBlockName,
        verdicts: Vec<StateVerdict>,
        reason: String,
    ) -> RefusedConclusionWrite {
        let refused = RefusedConclusionWrite {
            name,
            verdicts,
            reason,
        };
        eprintln!(
            "{CONCLUSION_WRITE_REFUSED_LINE_PREFIX} {} states={} reason={}",
            refused.name,
            refused.verdicts.len(),
            refused.reason
        );
        if !self
            .break_switches
            .is_on(BreakSwitch::RefusedConclusionsDropped)
        {
            self.refused_writes.push(refused.clone());
        }
        refused
    }

    /// 一帧写结论要带的：认领键、守卫那一行与读到的那一份、条目（强制现判的凭据插在守卫那一行之前）、删键（待判行的期待是 `row_as_read`，
    /// 不在了是「不在」；一份红整份换上时旧的那几行由守卫那一行担着，期待取写之前此刻的样子）。
    ///
    /// # Errors
    /// 有一行的值长过一段（写结论那一帧不分段，一帧原子写不下）；读旧那几行此刻的样子失败。
    fn guarded_write_of_the_frame(
        &mut self,
        name: &PendingBlockName,
        row_as_read: Option<&GpuPendingRow>,
        frame: &ConclusionFrame,
    ) -> Result<(GuardedConclusionWrite, bool), String> {
        if let Some((key, value)) = frame
            .rows_to_put
            .iter()
            .find(|(_, value)| value.len() > SEGMENT_BYTES)
        {
            return Err(format!(
                "键 {} 的值 {} 字节，长过一段（{SEGMENT_BYTES} 字节）：写结论那一帧不分段，原子写不下",
                hexadecimal_text(&key[..key.len().min(16)]),
                value.len()
            ));
        }
        let mut entries: Vec<(Table, Vec<u8>, Vec<u8>)> = frame
            .rows_to_put
            .iter()
            .map(|(key, value)| (Table::Conclusions, key.clone(), value.clone()))
            .collect();
        let forced_row = row_as_read.filter(|row| {
            row.forced == ForcedRejudging::Forced
                && !self
                    .break_switches
                    .is_on(BreakSwitch::RejudgedRowNotWritten)
        });
        if let Some(row) = forced_row {
            let credential = (
                Table::Conclusions,
                name.rejudged_key(row.pool_entry_number),
                GpuRejudgedRow {
                    holder_identity: self.holder(),
                    conclusion_digest: frame.conclusion_digest,
                }
                .to_value_bytes(),
            );
            let is_the_guard_row_last = entries
                .last()
                .is_some_and(|(_, key, _)| *key == frame.guard_key);
            let position = if is_the_guard_row_last {
                entries.len() - 1
            } else {
                entries.len()
            };
            entries.insert(position, credential);
        }
        let replaced_rows: Vec<(Table, Vec<u8>)> = frame
            .keys_to_delete
            .iter()
            .map(|key| (Table::Conclusions, key.clone()))
            .collect();
        let replaced_rows_now = self
            .claims
            .read(&replaced_rows)
            .map_err(|failure| format!("读一份红整份换上时旧的那几行失败：{failure}"))?;
        let mut deleted_keys: Vec<GuardedDeletion> = frame
            .keys_to_delete
            .iter()
            .zip(replaced_rows_now)
            .map(|(key, value)| GuardedDeletion {
                key: key.clone(),
                expectation: GuardRowState::of_stored_value(value.as_deref()),
            })
            .collect();
        deleted_keys.push(GuardedDeletion {
            key: name.pending_key(),
            expectation: GuardRowState::of_stored_value(
                row_as_read.map(GpuPendingRow::to_value_bytes).as_deref(),
            ),
        });
        Ok((
            GuardedConclusionWrite {
                holder_identity: self.holder(),
                claim_key: name.claim_key(),
                guard_key: frame.guard_key.clone(),
                guard_expectation: GuardRowState::of_stored_value(
                    frame.guard_row_as_read.as_deref(),
                ),
                entries,
                deleted_keys,
            },
            forced_row.is_some(),
        ))
    }

    /// 一块判完：编结论帧，一帧写下结论、删待判行（强制现判的加凭据行），守卫不过照种类重编、重读再写（文件头「写回」）。
    ///
    /// # Errors
    /// 那一帧没写成（结论留在 [`Self::refused_writes`]、打了红行）。
    ///
    /// # Panics
    /// 这一块不在认领着、在判的块里（主循环交回的块与认领对不上）。
    pub fn write_back(
        &mut self,
        name: &PendingBlockName,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> Result<WrittenBack, RefusedConclusionWrite> {
        let ClaimedBlock { row, .. } = self.claimed.remove(name).unwrap_or_else(|| {
            panic!("{name} 不在这个服务认领着的块里：主循环交回的块与认领对不上")
        });
        let mut row_as_read = Some(row.clone());
        // 结论照待判行里写的结论版本键编帧（文件头「版本对账」）；弄坏开关 conclusion-written-under-this-build-key 下换成这个二进制算的头一把
        let row_of_the_frame = match self.version_keys.conclusion_versions.first() {
            Some(this_build_key)
                if self
                    .break_switches
                    .is_on(BreakSwitch::ConclusionWrittenUnderThisBuildKey) =>
            {
                GpuPendingRow {
                    conclusion_version: ConclusionVersion(*this_build_key),
                    ..row.clone()
                }
            }
            Some(_) | None => row.clone(),
        };
        let mut last_refusal = String::new();
        // 迭代上界：CONCLUSION_WRITE_ATTEMPTS；每一圈要么写成、要么拒、要么重编重读之后再写一次
        for _ in 0..CONCLUSION_WRITE_ATTEMPTS {
            let frame = match self
                .side
                .conclusion_frame(&row_of_the_frame, verdicts.clone(), card)
            {
                Ok(frame) => frame,
                Err(reason) => {
                    return Err(self.refuse(*name, verdicts, format!("编不成结论帧：{reason}")));
                }
            };
            let (request, wrote_the_rejudged_row) =
                match self.guarded_write_of_the_frame(name, row_as_read.as_ref(), &frame) {
                    Ok(request) => request,
                    Err(reason) => return Err(self.refuse(*name, verdicts, reason)),
                };
            let refusal = match self.claims.write_guarded_conclusion(&request) {
                Ok(GuardedConclusionWriteOutcome::Written(_)) => {
                    return Ok(WrittenBack {
                        conclusion_digest: frame.conclusion_digest,
                        wrote_the_rejudged_row,
                    });
                }
                Ok(GuardedConclusionWriteOutcome::Refused { refusal, .. }) => refusal,
                Err(failure) => {
                    let reason = format!("写结论那一帧没写成：{failure}");
                    return Err(self.refuse(*name, verdicts, reason));
                }
            };
            match refusal {
                GuardRefusal::ClaimHeldByOther {
                    holder_identity, ..
                } => {
                    let reason = format!(
                        "认领被别的持有者 {} 持着，这一帧不写",
                        hexadecimal_text(&holder_identity)
                    );
                    return Err(self.refuse(*name, verdicts, reason));
                }
                GuardRefusal::GuardRowChanged { current } => {
                    last_refusal = format!("结论块那一行编帧之后变了（此刻 {current:?}）");
                    if self
                        .break_switches
                        .is_on(BreakSwitch::GuardRowChangeNotReread)
                    {
                        return Err(self.refuse(*name, verdicts, last_refusal));
                    }
                }
                GuardRefusal::DeletedRowChanged { key, current } => {
                    last_refusal = format!(
                        "删键 {} 那一行读完之后变了（此刻 {current:?}）",
                        hexadecimal_text(&key[..key.len().min(16)])
                    );
                    if key == name.pending_key() {
                        if self
                            .break_switches
                            .is_on(BreakSwitch::PendingRowChangeNotReread)
                        {
                            return Err(self.refuse(*name, verdicts, last_refusal));
                        }
                        match self.side.pending_row(name) {
                            Ok(None) => row_as_read = None,
                            Ok(Some(reread)) if is_the_same_block(&reread, &row) => {
                                row_as_read = Some(reread);
                            }
                            Ok(Some(_)) => {
                                let reason = format!(
                                    "{last_refusal}；重读到的待判行指的是另一块（块键、流程、判表、区间或期待的键不同）"
                                );
                                return Err(self.refuse(*name, verdicts, reason));
                            }
                            Err(error) => {
                                let reason = format!("{last_refusal}；重读待判行失败：{error:?}");
                                return Err(self.refuse(*name, verdicts, reason));
                            }
                        }
                    }
                }
            }
        }
        let reason = format!(
            "写结论那一帧守卫不过 {CONCLUSION_WRITE_ATTEMPTS} 次，最后一次：{last_refusal}"
        );
        Err(self.refuse(*name, verdicts, reason))
    }
}
