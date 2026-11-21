//! 待判池的判定库那一份：从判定库里取待判行、认领、交给服务主循环判，判完在一帧里写回结论、删掉待判行（强制现判的另写凭据行）。
//! 待判行的键与值、强制现判凭据的键与值、在库上现算判完、读回装判器的材料、把结论字编成结论帧，一律照验证包那一套
//! （`singlefs_verification::verdict_store` 的表 `1D`、`1E` 与 `crash_amplification` 交出的三个函数），服务不另写一份；
//! 验证包那一侧经接口 [`VerificationPackageSide`]（真的那一份在 `crate::verification_package_side`，不上卡的测试换成替身）。
//!
//! 进主循环的块照旧走内存里的待判池（[`crate::pending_pool::InMemoryPendingPool`]，批次调度只看它）；这一份在主循环外面当它的客户端：
//! 取活分两步，先挑（[`LibraryPendingPool::candidates`]：扫库、现算判完），再认领（[`LibraryPendingPool::claim_the_candidates`]）。
//!
//! - 不按键挑：待判行不带期待的判器键，三族都只照行的数据算，不认判器是哪一版；结论照待判行里写的那一把结论版本键存
//!   （验证包 `conclusion_frame_of_a_gpu_pending_block` 照行里的键编帧）。
//! - 判完：不是强制现判的块，挑之前在库上现算（验证包 `judging_of_a_gpu_pending_block_in_the_library`）：没判完的、判完而红的照取；
//!   判完而可复用的不取，带守卫删掉那一行待判行（判段等的是「行不在且续点口径判完」，行留着判段就一直等）。强制现判的块一律现判，不问库。
//! - 认领：键是判定库服务登记的认领键（`prune_server::gpu_judging_rows::gpu_claim_key`，只记认领、不当数据行写）；
//!   别的持有者持着、租期没到的不取；认领到的进「在判」，判着的时候每过三分之一租期续一次（[`LibraryPendingPool::renew_the_claims`]）。
//!   认领只看认领键：扫库之后、认领之前别的持有者写回了这一块（那一帧删了待判行、放了认领键），认领照样回「认领到」。所以认领到之后
//!   重读这一块的待判行：不在了、或指的是另一块，不取，并以自己的持有者放掉刚认领到的（判定库服务的放认领帧，`prune_server::protocol` 文件头「放认领」）；
//!   还是同一块的照重读到的那一行取（判段这时可能已把它升成强制现判）。认领回「认领到」是判定库服务提交、同步之后才回，
//!   排在它前面的那一帧写回已落盘，之后的点读读得到行已删。
//! - 放认领：持有者是认领与写结论那个客户端的身份，每个服务进程起来时新取；上一个进程留下的认领这一个进程认不回，起来时
//!   照状态目录里上一个进程的「在跑」那一行判它还在不在（`crate::service_process` 文件头），没了就以它的持有者放掉库里各待判块上它的认领
//!   （[`LibraryPendingPool::release_the_claims_left_by_the_previous_process`]），还活着、判不了的不放；停的时候以自己的持有者放掉还持着的
//!   （[`LibraryPendingPool::release_the_claims_of_this_process`]）。放认领帧只放点名的持有者名下的，别人持着的不动。
//!   借卡（`crate::card_lease` 文件头）：服务以这同一个持有者持这一台的卡键（[`LibraryPendingPool::lend_the_cards_of_this_host`] 之后），
//!   起来时放上一个进程的、停的时候放自己的，卡键跟三族的认领键一起放。
//!   这一台判不了的块（主循环拒的理由是在腾空的卡上也显存不够、或判法自己 panic，`crate::serving_from_the_library` 文件头）当场以自己的持有者
//!   放掉它的认领（[`LibraryPendingPool::release_the_claim_left_to_another_service`] 与另两族同名的那两个），另一台的服务随即取得到。
//! - 日志：认领到、别的持有者持着（同一块同一个持有者连着几轮持着只打头一轮）、写回成了、放认领，各打一行可数的（行首常量在下面）；
//!   都经 [`LibraryPendingPool::print_a_service_line`]（测试打开 [`LibraryPendingPool::keep_the_service_lines`] 时另收进内存）。
//! - 写回（[`LibraryPendingPool::write_back`]）：一帧判定库服务的写结论（`prune_server::protocol` 文件头「写结论」）：核认领、核守卫那一行
//!   （结论块那一行，期待是编帧之前读到的那一份）、核删键（待判行的期待是这个服务读到的那一份：判段读完之后把它升成强制现判的不删）。
//!   守卫不过照种类办：认领被别人持着不写；守卫那一行变了就重编（验证包照判段存块的规矩重读、重比：两份都绿而不同交回错、报红，
//!   一份红整份换上）再写；待判行变了就重读那一行：还是同一块的照重读到的那一份再写（升成强制现判的带上凭据行），不在了的不再删它。
//!   这一帧没写成（被别人持着、编不成、写库失败、守卫不过 [`CONCLUSION_WRITE_ATTEMPTS`] 次）不丢结论：结论字留在 [`LibraryPendingPool::refused_writes`]，
//!   打一行 [`CONCLUSION_WRITE_REFUSED_LINE_PREFIX`] 报红，待判行留在库里。
//!
//! 核对待判行那一族（验证包表 `1F`；名字、认领键、材料与行首在 `crate::facts_verifier_family`）走同一套，各步取这一族自己的那一样：
//! - 扫表：`VerdictStore::every_facts_verifier_pending_row`（[`LibraryPendingPool::facts_verifier_candidates`]）；
//! - 核过没有：在库上现算（验证包 `facts_verifier_pending_block_is_verified_in_the_library`），核对行已在的不取、带守卫只删核对待判行；
//! - 认领、重读、续租、放认领：认领键是这一族自己的（`FactsVerifierBlockName::claim_key`），认领到之后重读核对待判行，不在了或指的是另一块
//!   （入池序号之外有一项不同）的放掉；起来时放上一个进程的、停的时候放自己的，三族的认领键一起放；
//! - 材料：认领到的块按（录下的流程、事实表）分组，每组照组里键最小的那一块读回一份材料（验证包 `material_of_a_facts_verifier_pending_block`），
//!   一组交服务主循环在一张卡上核（[`FactsVerifierJobOfTheLibrary`]）；读不回的点名一次，这个进程里不再取它，认领到租期过了自己失效；
//! - 写回（[`LibraryPendingPool::write_back_the_facts_verifier_block`]）：验证包照这一块编一帧（`facts_verifier_frame_of_a_pending_block`：
//!   要写的核对行、核对行守卫、要删的核对待判行与它的守卫），服务把它放进判定库服务的写结论帧：认领键与持有者是服务的
//!   （验证包那一帧里不带，写结论帧要），守卫键是核对行、期待是编帧之前读到的那一份，删键是核对待判行、期待是服务取这一行时读到的那一份；
//!   核认领、核两个守卫、写、删、放认领一帧原子地落。守卫不过照种类办：认领被别人持着不写；核对行变了重编（验证包照此刻那一份比，相同不写、
//!   只删）；核对待判行变了重读，还是同一块的照重读到的那一份再写，不在了的不再删它；至多 [`CONCLUSION_WRITE_ATTEMPTS`] 次。
//!   没写成的结论位留在 [`LibraryPendingPool::refused_facts_verifier_writes`]，打一行 `FACTS_VERIFIER_WRITE_REFUSED_LINE_PREFIX` 报红，
//!   核对待判行留在库里。
//!
//! 判表待判行那一族（验证包表 `20` 判表待判行、表 `21` 判表结论字；名字、认领键、帧与行首在 `crate::packed_tables_family`）同样走这一套：
//! - 扫表：`VerdictStore::every_packed_tables_pending_row`（[`LibraryPendingPool::packed_tables_candidates`]）；
//! - 结论字在不在：在库上现算（验证包 `packed_tables_verdict_words_are_in_the_library`：表 `21` 有这一把键、不强制现判的算在；
//!   强制现判的照验证包那一侧的凭据判），在的不取、带守卫只删判表待判行（守卫取这一族的认领键，同镜像判定那一族删可复用块的行）；
//! - 认领、重读、续租、放认领：认领键是这一族自己的（`PackedTablesName::claim_key`），认领到之后重读判表待判行，不在了或指的是另一张
//!   （旋钮栏或态数不同）的放掉；起来时放上一个进程的、停的时候放自己的，三族的认领键一起放；
//! - 流：认领到的每张表照行读回判表、照旋钮栏取内核源码（验证包那一侧，`VerificationPackageSide::packed_tables_stream`；行里旋钮的摘要要与名字里的相同），
//!   读回的判表的摘要要与名字里的相同、判表的态数要与行里的相同，不同的点名一次、这个进程里不再取它，认领到租期过了自己失效；配上的交服务主循环判 `0..态数` 一整块；
//! - 写回（[`LibraryPendingPool::write_back_the_packed_tables`]）：验证包照名字、行、结论字与设备属性编一帧（表 `21` 那一行、它的守卫、要删的判表待判行与它的守卫），
//!   服务放进判定库服务的写结论帧：认领键与持有者是服务的，守卫键是表 `21` 那一行、期待是编帧之前读到的那一份，删键是判表待判行、期待是服务取这一行时读到的那一份；
//!   核认领、核两个守卫、写、删、放认领一帧原子地落。守卫不过照种类办：认领被别人持着不写；表 `21` 那一行变了重编；判表待判行变了重读，还是同一张的照重读到的那一份再写，
//!   不在了的不再删它；至多 [`CONCLUSION_WRITE_ATTEMPTS`] 次。值长过一段（`prune_server::segmented_values::SEGMENT_BYTES`）的不写（写结论那一帧不分段）。
//!   没写成的结论字留在 [`LibraryPendingPool::refused_packed_tables_writes`]，打一行 `PACKED_TABLES_VERDICT_WORDS_WRITE_REFUSED_LINE_PREFIX` 报红，
//!   判表待判行留在库里。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use prune_server::gpu_judging_rows::gpu_claim_key;
use prune_server::protocol::{
    ClaimOutcome, ClaimReleaseOutcome, GuardRefusal, GuardRowState, GuardedConclusionWrite,
    GuardedDeletion, ListedClaim, Table,
};
use prune_server::reliable_client::{ClientFailure, GuardedConclusionWriteOutcome, ReliableClient};
use prune_server::segmented_values::SEGMENT_BYTES;
use singlefs_verification::crash_amplification::{
    CardThatJudgedAPendingBlock, GpuPendingBlockInTheLibrary,
};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    key_bytes_of_the_gpu_pending_row_in, key_bytes_of_the_gpu_rejudged_row_in,
    FactsVerifierPendingRow, FactsVerifierRowsForOneFrame, ForcedRejudging, GpuPendingRow,
    GpuPendingStreamKey, GpuRejudgedRow, JudgingDevice, LibraryKeySpace, PackedTablesPendingRow,
    PruneServerFailure, VerdictStoreError,
};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::card_lease::{claim_a_card_lease_key_as, CardLeaseKeyAnswer, CardLeaseKeys};
use crate::facts_verifier_family::{
    is_the_same_facts_verifier_block, material_group_of, FactsVerifierBlockName,
    FactsVerifierMaterial, FactsVerifierPendingRowOfTheLibrary, MaterialGroupOfARow,
    FACTS_VERIFIER_BLOCK_CLAIMED_LINE_PREFIX, FACTS_VERIFIER_CLAIM_HELD_BY_OTHER_LINE_PREFIX,
    FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX,
    FACTS_VERIFIER_WRITE_REFUSED_LINE_PREFIX, FACTS_VERIFIER_WRITTEN_LINE_PREFIX,
    VERIFIED_FACTS_BLOCK_ROW_DELETED_LINE_PREFIX,
};
use crate::packed_tables_family::{
    is_the_same_packed_tables_row, judging_device_of_the_card, PackedTablesFrame, PackedTablesName,
    PackedTablesPendingRowOfTheLibrary, PACKED_TABLES_CLAIMED_LINE_PREFIX,
    PACKED_TABLES_CLAIM_HELD_BY_OTHER_LINE_PREFIX,
    PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX,
    PACKED_TABLES_PENDING_ROW_DELETED_LINE_PREFIX,
    PACKED_TABLES_VERDICT_WORDS_WRITE_REFUSED_LINE_PREFIX,
    PACKED_TABLES_VERDICT_WORDS_WRITTEN_LINE_PREFIX,
};
use crate::pending_pool::StateVerdict;
use crate::service_process::{PreviousServiceProcess, ProcessLiveness};
use crate::stream_identity::{StreamFacts, StreamKey};

mod claims_across_the_families;
mod facts_verifier_pending_rows;
mod image_judgement_pending_rows;
mod packed_tables_pending_rows;

/// 结论那一帧没写成、结论留在服务里报红的那一行的开头。
pub const CONCLUSION_WRITE_REFUSED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_CONCLUSION_WRITE_REFUSED";
/// 待判行扫不了、或验证包一侧答不了它的那一行的开头（每行只打一次）。
pub const PENDING_ROW_SKIPPED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_PENDING_ROW_SKIPPED";
/// 判完而可复用的块，待判行带守卫删掉了的那一行的开头。
pub const REUSABLE_BLOCK_ROW_DELETED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_REUSABLE_BLOCK_ROW_DELETED";
/// 认领到一块（认领到之后重读过待判行、配上了流）的那一行的开头。
pub const BLOCK_CLAIMED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_BLOCK_CLAIMED";
/// 别的持有者持着、这一轮不取的那一行的开头（同一块同一个持有者连着几轮持着只打头一轮）。
pub const CLAIM_HELD_BY_OTHER_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CLAIM_HELD_BY_OTHER";
/// 结论那一帧写成了的那一行的开头。
pub const CONCLUSION_WRITTEN_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CONCLUSION_WRITTEN";
/// 认领到之后重读待判行，不在了（或指的是另一块、重读不了）而放掉刚认领到的那一行的开头。
pub const CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_CLAIM_RELEASED_AFTER_THE_REREAD";
/// 起来时以上一个进程的持有者、停的时候以自己的持有者放认领的那一行的开头。
pub const CLAIMS_RELEASED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_CLAIMS_RELEASED";
/// 起来时上一个进程的认领不放（它还活着，或那一行认不出、判不了活没活）的那一行的开头。
pub const PREVIOUS_PROCESS_CLAIMS_KEPT_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PREVIOUS_PROCESS_CLAIMS_KEPT";
/// 放认领一帧至多带几把键：一把约百来字节，一帧约几百 KB，远在帧长上限之下。
pub const CLAIM_RELEASE_KEYS_PER_FRAME: usize = 4096;
/// 一个待判池同时认领着、在判的块数水位上限：够把这一台的全部卡填满好几波，
/// 又不会在一轮里把库里几万块全抢过来囤着、导致别的台空耗或排在后面的块等过租期。
pub const CLAIMED_BLOCKS_WATERMARK: usize = 64;
/// 库里待判行超过一轮认领上限时，一轮至多挑出、认领这么多块（挑之前先按入池序号升序排、跳过别的持有者正持着的认领）。
pub const CANDIDATES_PER_ROUND_LIMIT: usize = 32;
/// 库里待判行很多时，一轮顺手删掉的可复用待判行（或已核过的核对待判行）上限，余下的留给后续轮次，不堵住这一轮派卡。
pub const REUSABLE_DELETIONS_PER_ROUND_LIMIT: usize = 64;
/// 写结论那一帧守卫不过（守卫那一行或删键那一行读完之后变了）之后重编、重读再写，一块至多写几次。
/// 取 4 是推的：每一次守卫不过都是这一块在编帧与写帧之间被别人改过，连着四次被改是另有一方在反复写它，报红交人看。
pub const CONCLUSION_WRITE_ATTEMPTS: u32 = 4;

/// 一块待判的名字：哪条流（验证包的流键）的第几块。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingBlockName {
    pub stream: GpuPendingStreamKey,
    pub block_number: u64,
}

impl PendingBlockName {
    /// 这一块的待判行在 `key_space` 那一份判定库里的整把键（验证包的表 `1D`，键空间的接法归验证包）。
    #[must_use]
    pub fn pending_key(&self, key_space: &LibraryKeySpace) -> Vec<u8> {
        key_bytes_of_the_gpu_pending_row_in(key_space, &self.stream, self.block_number)
    }

    /// 这一块的认领键（判定库服务登记的认领键：只记认领，不当数据行写）。
    #[must_use]
    pub fn claim_key(&self) -> Vec<u8> {
        gpu_claim_key(&self.stream.0, self.block_number)
    }

    /// 这一块的强制现判凭据在 `key_space` 那一份判定库里的整把键（验证包的表 `1E`）。
    #[must_use]
    pub fn rejudged_key(&self, key_space: &LibraryKeySpace, pool_entry_number: u64) -> Vec<u8> {
        key_bytes_of_the_gpu_rejudged_row_in(
            key_space,
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
/// 一份红整份换上时要删的旧行、守卫那一行的键与编帧之前读到的那一份、结论摘要（强制现判凭据里的那一个）。键都是这一份库里的整把键（登记的临时库里接着那一段键空间前缀）。
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

    /// 这一侧连的那一份库的键空间：全仓那一份是 [`LibraryKeySpace::TheOneLibrary`]；登记的临时库是登记里的键空间前缀那一段名字空间
    /// （头一个字节是一次性库名字空间的头一个字节）。待判池自己编的键（删待判行、强制现判凭据）调验证包带这份键空间的那一把；
    /// 验证包交回的帧里的键已经接过。
    fn key_space(&self) -> LibraryKeySpace;

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

    /// 库里全部核对待判行（验证包 `VerdictStore::every_facts_verifier_pending_row`）。
    ///
    /// # Errors
    /// 读库失败；库里有一行不合布局。
    fn facts_verifier_pending_rows(
        &mut self,
    ) -> Result<Vec<FactsVerifierPendingRowOfTheLibrary>, VerdictStoreError>;

    /// 重读一行核对待判行（不在是 `None`）。
    ///
    /// # Errors
    /// 读库失败；那一行不合布局。
    fn facts_verifier_pending_row(
        &mut self,
        name: &FactsVerifierBlockName,
    ) -> Result<Option<FactsVerifierPendingRow>, VerdictStoreError>;

    /// 这一块在行的核对键下已有核对行（验证包 `facts_verifier_pending_block_is_verified_in_the_library`）。
    ///
    /// # Errors
    /// 读库失败（不当成没核过）。
    fn facts_verifier_block_is_verified(
        &mut self,
        row: &FactsVerifierPendingRow,
    ) -> Result<bool, VerdictStoreError>;

    /// 这一行指的那一块上卡核要的材料（验证包 `material_of_a_facts_verifier_pending_block`）：同一组（录下的流程、事实表）的块共用它建核对器。
    ///
    /// # Errors
    /// 读不回、摘要对不上、这一行的区间不是计划里块键相同的那一块：交回为什么。
    fn facts_verifier_material(
        &mut self,
        row: &FactsVerifierPendingRow,
    ) -> Result<Arc<dyn FactsVerifierMaterial>, String>;

    /// 把这一块的结论位（按态序号次序，一个状态一个）编成写回的那一帧（验证包 `facts_verifier_frame_of_a_pending_block`；不写库）。
    ///
    /// # Errors
    /// 编不成（材料读不回、对不上这一行，结论位条数不对，读库失败）：交回为什么。
    fn facts_verifier_frame(
        &mut self,
        name: &FactsVerifierBlockName,
        row: &FactsVerifierPendingRow,
        bits_of_each_state: &[u32],
    ) -> Result<FactsVerifierRowsForOneFrame, String>;

    /// 库里全部判表待判行（验证包 `VerdictStore::every_packed_tables_pending_row`）。
    ///
    /// # Errors
    /// 读库失败；库里有一行不合布局。
    fn packed_tables_pending_rows(
        &mut self,
    ) -> Result<Vec<PackedTablesPendingRowOfTheLibrary>, VerdictStoreError>;

    /// 照键重读一行判表待判行（不在是 `None`；验证包 `VerdictStore::read_packed_tables_pending_row`）。
    ///
    /// # Errors
    /// 读库失败；那一行不合布局。
    fn packed_tables_pending_row(
        &mut self,
        name: &PackedTablesName,
    ) -> Result<Option<PackedTablesPendingRow>, VerdictStoreError>;

    /// 这张判表的结论字已在表 `21`（验证包 `packed_tables_verdict_words_are_in_the_library`：不强制现判的有这一把键就算在，强制现判的照凭据判）。
    ///
    /// # Errors
    /// 读库失败（不当成不在）。
    fn packed_tables_verdict_words_are_in_the_library(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
    ) -> Result<bool, VerdictStoreError>;

    /// 这张判表的流：照判表摘要读回判表（验证包 `read_packed_judge_tables_for_a_pending_row`）、照旋钮栏取内核源码
    /// （`crate::packed_tables_family` 文件头「内核源码」）。判表摘要与态数对不对得上名字与行，由调用方核。
    ///
    /// # Errors
    /// 读不回、摘要对不上、行里旋钮的摘要与名字里的不同：交回为什么。
    fn packed_tables_stream(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
    ) -> Result<StreamOfTheRow<Self::Materials>, String>;

    /// 把这张判表全部状态的结论字（按态序号次序）连同设备属性编成写回的那一帧（验证包
    /// `VerdictStore::packed_tables_verdict_words_rows_for_one_frame`；不写库）。
    ///
    /// # Errors
    /// 编不成（结论字份数不是态数、读库失败）：交回为什么。
    fn packed_tables_frame(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
        verdicts: &[StateVerdict],
        device: &JudgingDevice,
    ) -> Result<PackedTablesFrame, String>;
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

/// 一回放认领的账：放掉了几把、几把没人持着、几把别人持着（没放）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClaimsReleased {
    pub released: u64,
    pub not_claimed: u64,
    pub held_by_others: u64,
}

/// 起来时上一个服务进程留下的认领怎么办了（文件头「放认领」）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviousProcessClaims {
    /// 状态目录里没有「在跑」那一行：头一次起，或上一个进程没走到写那一行（没走到就没认领过）。
    NoPreviousProcess,
    /// 上一个进程还活着：它的认领不放。
    KeptWhileItRuns,
    /// 那一行认不出、或判不了上一个进程活没活：不放，认领到租期过了自己失效。
    KeptUndetermined { reason: String },
    /// 上一个进程没了：以它的持有者放了库里各待判块上它的认领。
    Released(ClaimsReleased),
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
    claim_a_key(client, name.claim_key(), lease_milliseconds)
}

/// 以 `client` 的身份认领（或续上）这一块核对待判行（认领键是这一族自己的，`crate::facts_verifier_family` 文件头「认领键」）。
///
/// # Errors
/// 读写失败；回的结局条数不是 1。
pub fn claim_a_facts_verifier_pending_block(
    client: &mut ReliableClient,
    name: &FactsVerifierBlockName,
    lease_milliseconds: u32,
) -> Result<ClaimAnswer, ClientFailure> {
    claim_a_key(client, name.claim_key(), lease_milliseconds)
}

/// 以 `client` 的身份认领（或续上）这一张判表（认领键是这一族自己的，`crate::packed_tables_family` 文件头「认领键」）。
///
/// # Errors
/// 读写失败；回的结局条数不是 1。
pub fn claim_a_packed_tables_pending_row(
    client: &mut ReliableClient,
    name: &PackedTablesName,
    lease_milliseconds: u32,
) -> Result<ClaimAnswer, ClientFailure> {
    claim_a_key(client, name.claim_key(), lease_milliseconds)
}

/// 以 `client` 的身份认领（或续上）一把认领键（三族的认领都走它）。
fn claim_a_key(
    client: &mut ReliableClient,
    claim_key: Vec<u8>,
    lease_milliseconds: u32,
) -> Result<ClaimAnswer, ClientFailure> {
    let mut outcomes = client.claim(&[(Table::Conclusions, claim_key)], lease_milliseconds)?;
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
        && reread.conclusion_version == judged.conclusion_version
}

/// 认领到、在判的一块。
struct ClaimedBlock {
    row: GpuPendingRow,
    renewed_at: Instant,
    claim_lost_to: Option<Vec<u8>>,
}

/// 认领到、在核的一块核对待判行。
struct ClaimedFactsVerifierBlock {
    row: FactsVerifierPendingRow,
    renewed_at: Instant,
    claim_lost_to: Option<Vec<u8>>,
}

/// 认领到、交服务主循环在一张卡上核的一组核对块（文件头「核对待判行那一族」：同一份录下的流程、同一份事实表），按键排；
/// 材料照组里键最小的那一块读回，几块共用。
pub struct FactsVerifierJobOfTheLibrary {
    pub blocks: Vec<FactsVerifierPendingRowOfTheLibrary>,
    pub material: Arc<dyn FactsVerifierMaterial>,
}

/// 核对行那一帧没写成的一块：结论位留在这里，不丢。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedFactsVerifierWrite {
    pub name: FactsVerifierBlockName,
    pub bits_of_each_state: Vec<u32>,
    pub reason: String,
}

/// 核对行那一帧写成了的一块：写没写核对行（库里已有逐字节相同的那一行时只删核对待判行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactsVerifierWrittenBack {
    pub verifier_row_written: bool,
}

/// 认领到、在判的一张判表。
struct ClaimedPackedTables {
    row: PackedTablesPendingRow,
    renewed_at: Instant,
    claim_lost_to: Option<Vec<u8>>,
}

/// 认领到、交服务主循环判的一张判表：名字、重读到的那一行与配上的流（判 `0..态数` 一整块）。
pub struct TakenPackedTables<Materials> {
    pub pending: PackedTablesPendingRowOfTheLibrary,
    pub stream: StreamOfTheRow<Materials>,
}

/// 结论字那一帧没写成的一张判表：结论字留在这里，不丢。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefusedPackedTablesWrite {
    pub name: PackedTablesName,
    pub verdicts: Vec<StateVerdict>,
    pub reason: String,
}

/// 结论字那一帧写成了的一张判表：那一帧写了几行（库里已有逐字节相同的表 `21` 那一行、验证包那一侧不重写时是 0，只删判表待判行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedTablesWrittenBack {
    pub rows_written: usize,
}

/// 待判池的判定库那一份。
pub struct LibraryPendingPool<Side> {
    side: Side,
    /// 认领与写结论那一帧走的容错客户端：它的身份就是认领的持有者（一个服务进程一份）。
    claims: ReliableClient,
    claim_lease_milliseconds: u32,
    break_switches: BreakSwitches,
    claimed: BTreeMap<PendingBlockName, ClaimedBlock>,
    /// 点名过的（扫不了、验证包一侧答不了、可复用块的行没删成）：每样只打一次。
    named: BTreeSet<Vec<u8>>,
    named_lines: Vec<String>,
    /// 认领到了却配不出流的块：这个进程里不再挑它（不然每一圈都为它认领一回），重起服务再试。
    streams_not_configured: BTreeSet<PendingBlockName>,
    refused_writes: Vec<RefusedConclusionWrite>,
    reusable_rows_deleted: u64,
    /// 上一轮认领时别的持有者持着的（块、持有者）：这一轮又见到的不再打行。
    held_by_others_last_round: BTreeSet<(PendingBlockName, Vec<u8>)>,
    /// 打开时（测试）服务打的那几种行另收一份在这里；服务进程不开，不随运行时长涨。
    kept_service_lines: Option<Vec<String>>,
    facts_claimed: BTreeMap<FactsVerifierBlockName, ClaimedFactsVerifierBlock>,
    /// 认领到了却读不回材料的核对块：这个进程里不再挑它，重起服务再试。
    facts_materials_not_read: BTreeSet<FactsVerifierBlockName>,
    refused_facts_verifier_writes: Vec<RefusedFactsVerifierWrite>,
    verified_facts_rows_deleted: u64,
    facts_held_by_others_last_round: BTreeSet<(FactsVerifierBlockName, Vec<u8>)>,
    packed_tables_claimed: BTreeMap<PackedTablesName, ClaimedPackedTables>,
    /// 认领到了却配不出流（读不回、流键或态数对不上）的判表：这个进程里不再挑它，重起服务再试。
    packed_tables_not_configured: BTreeSet<PackedTablesName>,
    refused_packed_tables_writes: Vec<RefusedPackedTablesWrite>,
    packed_tables_rows_deleted: u64,
    packed_tables_held_by_others_last_round: BTreeSet<(PackedTablesName, Vec<u8>)>,
    /// 这一台的借卡键（服务进程起来时给；没给的不借卡，取活循环不探请求键）。
    card_lease: Option<CardLeaseKeys>,
    /// 这一份库的键空间（[`VerificationPackageSide::key_space`]，起来时取一次）。
    key_space: LibraryKeySpace,
}

/// 名字空间之外的一把点名键：扫不了待判行那一样只打一次。
const SCAN_PROBLEM_NAMED_KEY: &[u8] = b"pending-rows-scan";
/// 同上，扫不了核对待判行那一样。
const FACTS_VERIFIER_SCAN_PROBLEM_NAMED_KEY: &[u8] = b"facts-verifier-pending-rows-scan";
/// 同上，扫不了判表待判行那一样。
const PACKED_TABLES_SCAN_PROBLEM_NAMED_KEY: &[u8] = b"packed-tables-pending-rows-scan";

impl<Side: VerificationPackageSide> LibraryPendingPool<Side> {
    #[must_use]
    pub fn new(
        side: Side,
        claims: ReliableClient,
        claim_lease_milliseconds: u32,
        break_switches: BreakSwitches,
    ) -> Self {
        let key_space = side.key_space();
        Self {
            side,
            claims,
            claim_lease_milliseconds,
            break_switches,
            claimed: BTreeMap::new(),
            named: BTreeSet::new(),
            named_lines: Vec::new(),
            streams_not_configured: BTreeSet::new(),
            refused_writes: Vec::new(),
            reusable_rows_deleted: 0,
            held_by_others_last_round: BTreeSet::new(),
            kept_service_lines: None,
            facts_claimed: BTreeMap::new(),
            facts_materials_not_read: BTreeSet::new(),
            refused_facts_verifier_writes: Vec::new(),
            verified_facts_rows_deleted: 0,
            facts_held_by_others_last_round: BTreeSet::new(),
            packed_tables_claimed: BTreeMap::new(),
            packed_tables_not_configured: BTreeSet::new(),
            refused_packed_tables_writes: Vec::new(),
            packed_tables_rows_deleted: 0,
            packed_tables_held_by_others_last_round: BTreeSet::new(),
            card_lease: None,
            key_space,
        }
    }

    /// 从此把这一台的卡借给判段之外要卡的进程（`crate::card_lease` 文件头）：取活循环探请求键、持卡键，放认领时卡键一起放。
    pub fn lend_the_cards_of_this_host(&mut self, keys: CardLeaseKeys) {
        self.card_lease = Some(keys);
    }

    /// 这一台的借卡键（没给是 `None`）。
    #[must_use]
    pub fn card_lease_keys(&self) -> Option<&CardLeaseKeys> {
        self.card_lease.as_ref()
    }

    /// 以这个池的持有者认领（或续上）一把借卡的键。
    ///
    /// # Errors
    /// 读写判定库失败；库里有这把键（认领键不当数据行写）。
    pub fn claim_a_card_lease_key(
        &mut self,
        key: Vec<u8>,
        lease_milliseconds: u32,
    ) -> Result<CardLeaseKeyAnswer, LibraryFailure> {
        let holder = self.holder();
        claim_a_card_lease_key_as(&mut self.claims, &holder, key, lease_milliseconds)
            .map_err(LibraryFailure::from)
    }

    /// 以这个池的持有者放掉卡键（借出去；没给借卡键的不放、交空账）。
    ///
    /// # Errors
    /// 读写判定库失败。
    pub fn release_the_card_key(&mut self) -> Result<ClaimsReleased, LibraryFailure> {
        let Some(card_key) = self.card_lease.as_ref().map(CardLeaseKeys::card_key) else {
            return Ok(ClaimsReleased::default());
        };
        let holder = self.holder();
        self.release_the_claim_keys(&holder, &[card_key])
    }

    /// 从此把服务打的行（[`Self::print_a_service_line`]）另收一份进内存，测试照它数。
    pub fn keep_the_service_lines(&mut self) {
        self.kept_service_lines = Some(Vec::new());
    }

    /// 打开 [`Self::keep_the_service_lines`] 之后收下的行，按打出的先后；没打开是空的。
    #[must_use]
    pub fn kept_service_lines(&self) -> &[String] {
        self.kept_service_lines.as_deref().unwrap_or(&[])
    }

    /// 打一行服务日志（标准错误，经服务的日志过滤，`crate::service_log`），打开了收行的另收一份。
    pub fn print_a_service_line(&mut self, line: String) {
        eprintln!("{line}");
        if let Some(kept) = &mut self.kept_service_lines {
            kept.push(line);
        }
    }

    /// 认领的持有者身份（认领与写结论那一个客户端的身份）。
    #[must_use]
    pub fn holder(&self) -> Vec<u8> {
        self.claims.client_identity().to_vec()
    }

    /// 列出这一份库的判定库服务里 `prefix` 下此刻有效（租期没到）的认领，按键升序（登记的临时库就是这样列出来的，
    /// `crate::registered_libraries` 文件头）。
    ///
    /// # Errors
    /// 读写判定库失败；前缀不是 GPU 认领键的族前缀。
    pub fn list_the_claims_with_prefix(
        &mut self,
        prefix: &[u8],
    ) -> Result<Vec<ListedClaim>, LibraryFailure> {
        self.claims
            .list_claims(prefix)
            .map_err(LibraryFailure::from)
    }

    /// 结论那一帧没写成、留在服务里的块。
    #[must_use]
    pub fn refused_writes(&self) -> &[RefusedConclusionWrite] {
        &self.refused_writes
    }

    /// 这一块此刻认领着、在判。
    #[must_use]
    pub fn is_claimed(&self, name: &PendingBlockName) -> bool {
        self.claimed.contains_key(name)
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
}
