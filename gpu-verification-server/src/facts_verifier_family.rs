//! 核对待判行那一族（验证包表 `1F`：配了 GPU 的判段第 ② 段把要核的块写成核对待判行）在服务里自己的几样：块的名字与认领键、
//! 上卡核要的材料、这一族的行首。扫表、认领、重读、续租、放认领、写回与镜像判定那一族走同一套（`crate::library_pending_pool`
//! 文件头「核对待判行那一族」），上卡核走服务主循环借显存（`crate::service_loop` 文件头「核对」）。
//!
//! - 块的名字 [`FactsVerifierBlockName`]：流键（验证包 `FactsVerifierPendingStreamKey`）与块号；待判行的键照验证包
//!   `key_bytes_of_the_facts_verifier_pending_row_in`（带库的键空间）。
//! - 认领键 `gpu-claim/facts-verifier/<流键的小写十六进制>/<块号的 16 位小写十六进制>`（[`FactsVerifierBlockName::claim_key`]）：
//!   以判定库服务的 GPU 认领键前缀起头（放认领帧只收以它起头的键，`prune_server::protocol` 文件头「放认领」）；前缀之后那一段
//!   `facts-verifier/` 不是十六进制，镜像判定那一族的认领键 `gpu-claim/<流键的十六进制>/<块号>` 在那个位置上是十六进制，两族的键不会相同。
//!   只经认领帧认领、经写回那一帧或放认领帧放，不当数据行写。
//! - 上卡核的材料 [`FactsVerifierMaterial`]：事实表与每段的拆法，真的那一份是验证包 `material_of_a_facts_verifier_pending_block` 读回的；
//!   一组（同一份录下的流程、同一份事实表，[`MaterialGroupOfARow`]）的块共用一个核对器。

use std::fmt;

use prune_server::gpu_judging_rows::GPU_CLAIM_FAMILY_PREFIX;
use singlefs_verification::crash_amplification::FactsVerifierPendingBlockMaterial;
use singlefs_verification::crash_facts::{FlowFactsReadBack, SegmentExpansionFact};
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    key_bytes_of_the_facts_verifier_pending_row_in, FactsVerifierPendingRow,
    FactsVerifierPendingStreamKey, LibraryKeySpace,
};

/// 认领键里 GPU 认领键前缀之后、流键之前那一段（文件头「认领键」）。
const FACTS_VERIFIER_CLAIM_KEY_SEGMENT: &[u8] = b"facts-verifier/";
/// 认领键里流键与块号之间的分隔。
const CLAIM_KEY_FIELD_SEPARATOR: u8 = b'/';

/// 认领到一块核对待判行（重读过、读回了材料）的那一行的开头。
pub const FACTS_VERIFIER_BLOCK_CLAIMED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_FACTS_VERIFIER_BLOCK_CLAIMED";
/// 别的持有者持着、这一轮不取的那一行的开头（同一块同一个持有者连着几轮持着只打头一轮）。
pub const FACTS_VERIFIER_CLAIM_HELD_BY_OTHER_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_FACTS_VERIFIER_CLAIM_HELD_BY_OTHER";
/// 认领到之后重读核对待判行，不在了（或指的是另一块、重读不了）而放掉刚认领到的那一行的开头。
pub const FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_FACTS_VERIFIER_CLAIM_RELEASED_AFTER_THE_REREAD";
/// 每核完一块那一行的开头（字段与 `crate::serving_from_the_library::BLOCK_JUDGED_LINE_PREFIX` 那一行同名）。
pub const FACTS_BLOCK_VERIFIED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_FACTS_BLOCK_VERIFIED";
/// 核对行那一帧写成了的那一行的开头。
pub const FACTS_VERIFIER_WRITTEN_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_FACTS_VERIFIER_WRITTEN";
/// 核对行那一帧没写成、结论位留在服务里报红的那一行的开头。
pub const FACTS_VERIFIER_WRITE_REFUSED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_FACTS_VERIFIER_WRITE_REFUSED";
/// 已核过（核对行在）的块，核对待判行带守卫删掉了的那一行的开头。
pub const VERIFIED_FACTS_BLOCK_ROW_DELETED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_VERIFIED_FACTS_BLOCK_ROW_DELETED";

/// 一块核对待判行的名字：哪条流（验证包的流键）的第几块。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactsVerifierBlockName {
    pub stream: FactsVerifierPendingStreamKey,
    pub block_number: u64,
}

impl FactsVerifierBlockName {
    /// 这一块的核对待判行在 `key_space` 那一份判定库里的整把键（验证包的表 `1F`，键空间的接法归验证包）。
    #[must_use]
    pub fn pending_key(&self, key_space: &LibraryKeySpace) -> Vec<u8> {
        key_bytes_of_the_facts_verifier_pending_row_in(key_space, &self.stream, self.block_number)
    }

    /// 这一块的认领键（文件头「认领键」）。
    #[must_use]
    pub fn claim_key(&self) -> Vec<u8> {
        let mut key = GPU_CLAIM_FAMILY_PREFIX.to_vec();
        key.extend_from_slice(FACTS_VERIFIER_CLAIM_KEY_SEGMENT);
        key.extend_from_slice(hexadecimal_text(&self.stream.0).as_bytes());
        key.push(CLAIM_KEY_FIELD_SEPARATOR);
        key.extend_from_slice(format!("{:016x}", self.block_number).as_bytes());
        key
    }
}

impl fmt::Display for FactsVerifierBlockName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "stream={} block={}",
            hexadecimal_text(&self.stream.0),
            self.block_number
        )
    }
}

/// 库里的一行核对待判行：名字与验证包认回的行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactsVerifierPendingRowOfTheLibrary {
    pub name: FactsVerifierBlockName,
    pub row: FactsVerifierPendingRow,
}

/// 共用一个核对器的一组块的键：录下的流程的摘要、事实表摘要（两样相同，事实表与段的拆法就相同）。
pub type MaterialGroupOfARow = ([u8; DIGEST_BYTES], [u8; DIGEST_BYTES]);

/// 这一行属于哪一组（[`MaterialGroupOfARow`]）。
#[must_use]
pub fn material_group_of(row: &FactsVerifierPendingRow) -> MaterialGroupOfARow {
    (row.recorded_flow_digest, row.facts_digest)
}

/// 重读到的核对待判行指的还是不是同一块（入池序号之外逐项相同）：判段补写同一块时只换入池序号。
#[must_use]
pub fn is_the_same_facts_verifier_block(
    reread: &FactsVerifierPendingRow,
    taken: &FactsVerifierPendingRow,
) -> bool {
    reread.recorded_block == taken.recorded_block
        && reread.recorded_flow_digest == taken.recorded_flow_digest
        && reread.facts_digest == taken.facts_digest
        && reread.state_ordinals == taken.state_ordinals
}

/// 在一张卡上建核对器要的：事实表与每段的拆法（验证包 `crash_verify_gpu::GpuFactsVerifier::new_by_the_segment_expansions` 收的两样）。
pub trait FactsVerifierMaterial: Send + Sync {
    fn facts(&self) -> &FlowFactsReadBack;

    fn segment_expansions(&self) -> &[SegmentExpansionFact];
}

/// 验证包读回的那一份（`material_of_a_facts_verifier_pending_block`）。
impl FactsVerifierMaterial for FactsVerifierPendingBlockMaterial {
    fn facts(&self) -> &FlowFactsReadBack {
        &self.facts
    }

    fn segment_expansions(&self) -> &[SegmentExpansionFact] {
        &self.segment_expansions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 认领键逐字的样子，且与镜像判定那一族同一条流键、同一个块号的认领键不同（文件头「认领键」）。
    #[test]
    fn the_claim_key_is_written_word_for_word_and_differs_from_the_image_judgement_claim_key() {
        let name = FactsVerifierBlockName {
            stream: FactsVerifierPendingStreamKey([0xab; DIGEST_BYTES]),
            block_number: 0x1f,
        };
        let expected = format!(
            "gpu-claim/facts-verifier/{}/000000000000001f",
            "ab".repeat(32)
        );
        assert_eq!(name.claim_key(), expected.into_bytes());
        assert_ne!(
            name.claim_key(),
            prune_server::gpu_judging_rows::gpu_claim_key(&name.stream.0, name.block_number)
        );
    }
}
