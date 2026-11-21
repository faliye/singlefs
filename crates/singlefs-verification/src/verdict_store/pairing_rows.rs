//! 对拍那两张表（`19`、`1C`）：对拍结论与对拍输入。

use super::*;

/// 对拍结论的键：输入的摘要（调用方按对拍的输入数据算，`crate::crash_library::pairing_conclusion_key_of_the_inputs` 再拼判器引到的
/// 被检测产品代码）。两边判器（GPU 判器与 CPU 判器、GPU 核对与 CPU 参照这类）与比法自己的代码都不进键：改哪一边都不作废已判的对拍结论，
/// 判法改了对不对靠对拍自己与复用哨兵抓；判器调到的产品恢复、格式代码改了，对拍重拍。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PairingConclusionKey {
    pub inputs_digest: [u8; SHA256_DIGEST_BYTES],
}

/// 一条对拍结论两边比下来一致还是不一致：不一致就是对拍判红（照同一把键记红、撤掉这把键原有的绿，键记着红的下一趟现判、不当绿复用：
/// `.claude/rules/verification.md`「一个门禁：两种跑法」那一条「记红、撤绿、下趟现判」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingAgreement {
    BothSidesAgree,
    SidesDisagree,
}

impl PairingAgreement {
    /// 对拍结论那一行里的码。
    fn code(self) -> u8 {
        match self {
            Self::BothSidesAgree => 1,
            Self::SidesDisagree => 2,
        }
    }

    /// 码换回一致不一致；这一版不认的码交回 None（读回的那一行照不合布局报错，不当没判）。
    fn of_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::BothSidesAgree),
            2 => Some(Self::SidesDisagree),
            _ => None,
        }
    }
}

/// 存着的一条对拍结论：调用方给的结论字节（写法由调用方定）、两边比下来一致还是不一致（[`PairingAgreement`]，调用方判了交来），
/// 连同判定日期与跑法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredPairingConclusion {
    pub conclusion: Vec<u8>,
    pub agreement: PairingAgreement,
    pub circumstances: JudgementCircumstances,
}

impl StoredPairingConclusion {
    pub(super) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(
            &u32::try_from(self.conclusion.len())
                .expect("对拍结论装得进 u32 字节")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(&self.conclusion);
        bytes.push(self.agreement.code());
        push_length_prefixed_text(&mut bytes, &self.circumstances.judged_date);
        bytes.push(self.circumstances.run_kind.code());
        bytes
    }

    /// [`Self::to_bytes`] 的逆；不合布局交回 None（一致不一致的码、跑法码不是这一版认得的也是不合布局）。
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut cursor = ByteCursor { bytes, position: 0 };
        let conclusion_length = usize::try_from(cursor.big_endian_u32()?).ok()?;
        let conclusion = cursor.take(conclusion_length)?.to_vec();
        let agreement = PairingAgreement::of_code(*cursor.take(1)?.first()?)?;
        let judged_date = cursor.length_prefixed_text()?;
        let run_kind = RunKindOfTheJudgement::of_code(*cursor.take(1)?.first()?)?;
        cursor.is_at_the_end().then_some(Self {
            conclusion,
            agreement,
            circumstances: JudgementCircumstances {
                judged_date,
                run_kind,
            },
        })
    }
}

impl VerdictStore {
    /// 存一条对拍结论（[`StoredPairingConclusion`] 的字节同步落盘）。同一键再存照写（同一份输入换上这一次判出的结论）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_pairing_conclusion(
        &mut self,
        key: &PairingConclusionKey,
        conclusion: &StoredPairingConclusion,
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(
            &VerdictStoreKey::PairingConclusion(key),
            &conclusion.to_bytes(),
        )
    }

    /// 读一条对拍结论；没存过交回 None。
    ///
    /// # Errors
    /// 服务读不成；值不合 [`StoredPairingConclusion`] 的布局（跑法码不是这一版认得的也是）。
    pub fn read_pairing_conclusion(
        &self,
        key: &PairingConclusionKey,
    ) -> Result<Option<StoredPairingConclusion>, VerdictStoreError> {
        let key_bytes = VerdictStoreKey::PairingConclusion(key).to_key_bytes();
        let Some(stored) = self.read_value_kept_in_or_out(&key_bytes.0)? else {
            return Ok(None);
        };
        StoredPairingConclusion::from_bytes(&stored)
            .map(Some)
            .ok_or_else(|| {
                stored_bytes_mismatch(
                    VerdictStoreTable::PairingConclusions,
                    &key_bytes.0,
                    "对拍结论的值不合布局",
                )
            })
    }

    /// 写下一份对拍输入（键是它的摘要）。
    ///
    /// # Errors
    /// 服务写不成。
    pub fn store_pairing_input(
        &mut self,
        digest: &[u8; 32],
        bytes: &[u8],
    ) -> Result<(), VerdictStoreError> {
        self.put_one_synced(&VerdictStoreKey::PairingInput(digest), bytes)
    }

    /// 读一份对拍输入的字节；没写过交回 None。
    ///
    /// # Errors
    /// 服务读不成。
    pub fn read_pairing_input(
        &self,
        digest: &[u8; 32],
    ) -> Result<Option<Vec<u8>>, VerdictStoreError> {
        self.read_one_stored_value(&VerdictStoreKey::PairingInput(digest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 对拍结论按输入摘要存、读回逐字节相同；换了输入摘要读不到。
    /// 判别力（`crates/mutations.tsv`）：对拍结论的键不带输入摘要（只剩表标签），换了输入摘要照样读回那一条，这条红。
    #[test]
    fn a_pairing_conclusion_reads_back_under_its_inputs_digest_only() {
        let library = TestLibrary::new("pairing-conclusion");
        let mut store = library.open();
        let key = PairingConclusionKey {
            inputs_digest: [1; SHA256_DIGEST_BYTES],
        };
        let conclusion = StoredPairingConclusion {
            conclusion: b"agree states=12".to_vec(),
            agreement: PairingAgreement::SidesDisagree,
            circumstances: JudgementCircumstances {
                judged_date: "2026-10-15".to_string(),
                run_kind: RunKindOfTheJudgement::Pruned,
            },
        };
        assert_eq!(store.read_pairing_conclusion(&key).expect("读"), None);
        store
            .store_pairing_conclusion(&key, &conclusion)
            .expect("存");
        assert_eq!(
            store.read_pairing_conclusion(&key).expect("读"),
            Some(conclusion)
        );
        let other = PairingConclusionKey {
            inputs_digest: [9; SHA256_DIGEST_BYTES],
        };
        assert_eq!(store.read_pairing_conclusion(&other).expect("读"), None);
    }
}
