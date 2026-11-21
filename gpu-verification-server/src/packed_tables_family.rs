//! 判表待判行那一族（`packed-tables`；验证包表 `20` 判表待判行、表 `21` 判表结论字）在服务里自己的几样：一张判表的名字与认领键、
//! 写回那一帧要的几样、写进表 `21` 的设备属性、这一族的行首。给定状态表的引擎与观察者交回的盘的判段在任务里取不到卡时，把要判的那张打包判表
//! （已照判段的写法写进表 `11`）写成一行判表待判行、等服务写回这张表全部状态的结论字；服务只上卡判、交回结论字，不解字、不编结论块。
//! 扫表、认领、重读、续租、放认领、写回与镜像判定那一族走同一套（`crate::library_pending_pool` 文件头「判表待判行那一族」），
//! 上卡走服务主循环的判器路（`register_stream` / `submit_block`，同一个 `GpuCrashJudge`，显存照判器记账）：一张判表是一条流里从 0 到态数的一整块。
//!
//! - 名字 [`PackedTablesName`]：键的两段（判表摘要、容器容量旋钮摘要），表 `20` 与表 `21` 同一把；待判行的键照验证包
//!   `key_bytes_of_the_packed_tables_pending_row_in`（带库的键空间）。键只由内容定：两个判段要判同一张判表、照同一组旋钮，就是同一行。
//! - 认领键 `gpu-claim/packed-tables/<判表摘要>/<旋钮摘要>`（两段都是小写十六进制，[`PackedTablesName::claim_key`]）：
//!   以判定库服务的 GPU 认领键前缀起头（放认领帧只收以它起头的键，`prune_server::protocol` 文件头「放认领」）；前缀之后那一段
//!   `packed-tables/` 不是十六进制开头，镜像判定那一族的认领键 `gpu-claim/<流键的十六进制>/<块号>` 在那个位置上是十六进制，
//!   核对待判行那一族是 `facts-verifier/`，三族的键不会相同。只经认领帧认领、经写回那一帧或放认领帧放，不当数据行写。
//! - 内核源码：照行里的容器容量旋钮栏取（验证包 `kernel_sources_of_a_packed_tables_pending_row`：先核行里旋钮的摘要与名字里的旋钮摘要相同，再取
//!   `JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`）；旋钮栏是 `ContainerCapacityKnobs::PRODUCT` 时取到的就是
//!   `JudgeKernelSources::of_the_judge()`（它照 PRODUCT 调同一个函数），所以观察者那一族（上卡照 `of_the_judge()`）在行里写 PRODUCT。
//!   读回的判表的摘要要与名字里的判表摘要逐字节相同，判表的态数要与行里的态数相同，不同的不判、点名一次（`crate::library_pending_pool` 的
//!   `claim_the_packed_tables_candidates`）。内核源码与判器的代码不进键。
//! - 设备属性（[`judging_device_of_the_card`]）：判这张表的那张卡的卡型与驱动版本；块属性里不记 GPU 判器是哪一版。

use std::fmt;

use prune_server::gpu_judging_rows::GPU_CLAIM_FAMILY_PREFIX;
use singlefs_verification::crash_amplification::CardThatJudgedAPendingBlock;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    key_bytes_of_the_packed_tables_pending_row_in, JudgingDevice, LibraryKeySpace, PackedTablesKey,
    PackedTablesPendingRow,
};

/// 认领键里 GPU 认领键前缀之后、判表摘要之前那一段（文件头「认领键」）。
const PACKED_TABLES_CLAIM_KEY_SEGMENT: &[u8] = b"packed-tables/";
/// 认领键里两段之间的分隔。
const CLAIM_KEY_FIELD_SEPARATOR: u8 = b'/';

/// 认领到一张判表（重读过判表待判行、配上了流）的那一行的开头。
pub const PACKED_TABLES_CLAIMED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_PACKED_TABLES_CLAIMED";
/// 别的持有者持着、这一轮不取的那一行的开头（同一张表同一个持有者连着几轮持着只打头一轮）。
pub const PACKED_TABLES_CLAIM_HELD_BY_OTHER_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PACKED_TABLES_CLAIM_HELD_BY_OTHER";
/// 认领到之后重读判表待判行，不在了（或指的是另一张、重读不了）而放掉刚认领到的那一行的开头。
pub const PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PACKED_TABLES_CLAIM_RELEASED_AFTER_THE_REREAD";
/// 每判完一张判表那一行的开头（字段除名字那一项外与 `crate::serving_from_the_library::BLOCK_JUDGED_LINE_PREFIX` 那一行同名）。
pub const PACKED_TABLES_JUDGED_LINE_PREFIX: &str = "GPU_VERIFICATION_SERVER_PACKED_TABLES_JUDGED";
/// 结论字那一帧写成了的那一行的开头。
pub const PACKED_TABLES_VERDICT_WORDS_WRITTEN_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PACKED_TABLES_VERDICT_WORDS_WRITTEN";
/// 结论字那一帧没写成、结论字留在服务里报红的那一行的开头。
pub const PACKED_TABLES_VERDICT_WORDS_WRITE_REFUSED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PACKED_TABLES_VERDICT_WORDS_WRITE_REFUSED";
/// 表 `21` 已有这张判表的结论字（不强制现判），判表待判行带守卫删掉了的那一行的开头。
pub const PACKED_TABLES_PENDING_ROW_DELETED_LINE_PREFIX: &str =
    "GPU_VERIFICATION_SERVER_PACKED_TABLES_PENDING_ROW_DELETED";

/// 一张判表的名字：键的两段（文件头「名字」）。服务自己的一份（按字节排、能当集合的键），与验证包的键逐段相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackedTablesName {
    pub packed_judge_tables_digest: [u8; DIGEST_BYTES],
    pub container_capacity_knobs_digest: [u8; DIGEST_BYTES],
}

impl PackedTablesName {
    /// 验证包那一把键的名字。
    #[must_use]
    pub fn of_the_key(key: &PackedTablesKey) -> Self {
        Self {
            packed_judge_tables_digest: key.packed_judge_tables_digest,
            container_capacity_knobs_digest: key.container_capacity_knobs_digest,
        }
    }

    /// 这个名字的验证包那一把键（表 `20`、`21` 同一把）。
    #[must_use]
    pub fn library_key(&self) -> PackedTablesKey {
        PackedTablesKey {
            packed_judge_tables_digest: self.packed_judge_tables_digest,
            container_capacity_knobs_digest: self.container_capacity_knobs_digest,
        }
    }

    /// 这张判表的判表待判行在 `key_space` 那一份判定库里的整把键（验证包的表 `20`，键空间的接法归验证包）。
    #[must_use]
    pub fn pending_key(&self, key_space: &LibraryKeySpace) -> Vec<u8> {
        key_bytes_of_the_packed_tables_pending_row_in(key_space, &self.library_key())
    }

    /// 这张判表的认领键（文件头「认领键」）。
    #[must_use]
    pub fn claim_key(&self) -> Vec<u8> {
        let mut key = GPU_CLAIM_FAMILY_PREFIX.to_vec();
        key.extend_from_slice(PACKED_TABLES_CLAIM_KEY_SEGMENT);
        key.extend_from_slice(hexadecimal_text(&self.packed_judge_tables_digest).as_bytes());
        key.push(CLAIM_KEY_FIELD_SEPARATOR);
        key.extend_from_slice(hexadecimal_text(&self.container_capacity_knobs_digest).as_bytes());
        key
    }
}

impl fmt::Display for PackedTablesName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "tables={} container_capacity_knobs={}",
            hexadecimal_text(&self.packed_judge_tables_digest),
            hexadecimal_text(&self.container_capacity_knobs_digest)
        )
    }
}

/// 库里的一行判表待判行：名字与验证包认回的行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedTablesPendingRowOfTheLibrary {
    pub name: PackedTablesName,
    pub row: PackedTablesPendingRow,
}

/// 重读到的判表待判行指的还是不是同一张（强制现判与入池序号之外逐项相同）：判段补写、升成强制现判时只换这两样。
#[must_use]
pub fn is_the_same_packed_tables_row(
    reread: &PackedTablesPendingRow,
    taken: &PackedTablesPendingRow,
) -> bool {
    reread.container_capacity_knobs == taken.container_capacity_knobs
        && reread.state_count == taken.state_count
}

/// 一张判表的结论字编成的一帧（验证包 `VerdictStore::packed_tables_verdict_words_rows_for_one_frame` 编的那一帧里写结论要的几样；不写库）：
/// 要写的行（表 `21` 那一行，强制现判的凭据在验证包那一侧定了要写时一并在这里）、表 `21` 那一行的键与编帧之前读到的那一份（守卫）、
/// 要删的判表待判行的键与服务取这一行时读到的那一份。键都是全仓那一份库里的整把键。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedTablesFrame {
    pub rows_to_put: Vec<(Vec<u8>, Vec<u8>)>,
    pub verdict_words_row_key: Vec<u8>,
    pub verdict_words_row_as_read: Option<Vec<u8>>,
    pub pending_row_key_to_delete: Vec<u8>,
    pub pending_row_as_read: Vec<u8>,
}

/// 写进表 `21` 的设备属性（文件头「设备属性」）：判它的那张卡的卡型与驱动版本。
#[must_use]
pub fn judging_device_of_the_card(card: &CardThatJudgedAPendingBlock) -> JudgingDevice {
    JudgingDevice::Gpu {
        card_type: card.card_name.clone(),
        driver_version: card.card_driver_version.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 认领键逐字的样子，且与镜像判定那一族、核对待判行那一族的认领键不同（文件头「认领键」）。
    #[test]
    fn the_claim_key_is_written_word_for_word_and_differs_from_the_other_families_claim_keys() {
        let name = PackedTablesName {
            packed_judge_tables_digest: [0xab; DIGEST_BYTES],
            container_capacity_knobs_digest: [0xcd; DIGEST_BYTES],
        };
        let expected = format!(
            "gpu-claim/packed-tables/{}/{}",
            "ab".repeat(32),
            "cd".repeat(32)
        );
        assert_eq!(name.claim_key(), expected.into_bytes());
        assert_ne!(
            name.claim_key(),
            prune_server::gpu_judging_rows::gpu_claim_key(&name.packed_judge_tables_digest, 0)
        );
        let facts_verifier = crate::facts_verifier_family::FactsVerifierBlockName {
            stream: singlefs_verification::verdict_store::FactsVerifierPendingStreamKey(
                name.packed_judge_tables_digest,
            ),
            block_number: 0,
        };
        assert_ne!(name.claim_key(), facts_verifier.claim_key());
    }

    /// 名字与验证包的键来回换逐段相同。
    #[test]
    fn the_name_and_the_library_key_convert_both_ways_segment_for_segment() {
        let name = PackedTablesName {
            packed_judge_tables_digest: [0x01; DIGEST_BYTES],
            container_capacity_knobs_digest: [0x02; DIGEST_BYTES],
        };
        assert_eq!(PackedTablesName::of_the_key(&name.library_key()), name);
    }
}
