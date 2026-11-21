//! 结论那一截：观察者的名字、结论种类（镜像判定、观察者）与结论的版本（结论种类加判器引到的被检测产品代码）。

use super::*;

// ============================== 三种结论 ==============================

/// 观察者的名字：非空，不超过 `u16::MAX` 字节。只当索引：拼进它那一组判法单元的组名（`observer_code:<文件>:<名字>`）与行里报的名字；
/// 结论种类里没有它，是哪一个观察者由它的判定项标识担（组名取 SHA-256，进它的版本，`crate::crash_amplification::observer_identity_digest`）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObserverName(String);

/// 给的观察者名字是空的，或超过 `u16::MAX` 字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverNameRejected {
    pub text: String,
}

impl ObserverName {
    /// # Errors
    /// 空，或超过 `u16::MAX` 字节。
    pub fn new(text: &str) -> Result<Self, ObserverNameRejected> {
        if text.is_empty() || u16::try_from(text.len()).is_err() {
            return Err(ObserverNameRejected {
                text: text.to_string(),
            });
        }
        Ok(Self(text.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 一块存的两种结论：判镜像的结论、一个观察者的结果。判定库按（块键，结论种类，版本）存与记账，哪一种没有就只排那一种。
/// 镜像判定不分在哪判的（数据不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备）：GPU 判器与 CPU 判器判出的
/// 存在同一种类、同一版本下（[`ConclusionVersion::of_the_crash_image_judgement`]），判器自己的代码不进版本，判器引到的被检测产品代码进；
/// 在哪判的、驱动版本与卡型都是块的属性（判定库的块属性表），不进种类、不进版本。
/// 观察者不带名字：是哪一个观察者由版本担（判定项标识进版本，同一份文件里两个观察者两把版本）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConclusionKind {
    ImageJudgement,
    Observer,
}

impl ConclusionKind {
    /// 种类写成文字：`image_judgement`、`observer`。
    #[must_use]
    pub fn version_text(&self) -> String {
        match self {
            Self::ImageJudgement => "image_judgement".to_string(),
            Self::Observer => "observer".to_string(),
        }
    }
}

/// 判器判出的三种结论：崩溃镜像（崩溃放量与崩溃枚举用例按块判的）、给定状态（给定状态表引擎判的）、观察者交回的盘上跑池级 checker。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgedConclusion {
    CrashImageJudgement,
    GivenStatesJudgement,
    PoolCheckerOnTheImagesAfterTheObservers,
}

impl JudgedConclusion {
    /// 三种都在这里，次序同成员的次序。
    pub const EVERY_JUDGED_CONCLUSION: [Self; 3] = [
        Self::CrashImageJudgement,
        Self::GivenStatesJudgement,
        Self::PoolCheckerOnTheImagesAfterTheObservers,
    ];

    /// 进版本文字的那一截种类名。
    fn kind_text(self) -> &'static str {
        match self {
            Self::CrashImageJudgement => "crash image judgement",
            Self::GivenStatesJudgement => "given states judgement",
            Self::PoolCheckerOnTheImagesAfterTheObservers => {
                "pool checker on the images after the observers"
            }
        }
    }
}

/// 判器引到的被检测产品代码那一组（[`judged_product_code_key`]：CPU 参照判器、池级 checker、记录核对器的入口收的闭包只留产品包的单元）
/// 在这个二进制里取到的样子：判器判出的三种结论的版本与对拍结论的键都拼它。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JudgedProductCodeOfThisBinary {
    Keyed {
        fingerprint: [u8; DIGEST_BYTES],
    },
    /// 编译时那一组拒算或没编进来：版本拼这个进程的占位（`crate::crash_amplification::placeholder_judging_units_key_of_the_refused`：
    /// 几个进程各一把，撞不上算得出键的那一把），照它存的行照判、不入库（[`conclusion_versions_kept_out_of_the_library_in_this_process`]）。
    RefusedInThisProcess {
        placeholder: [u8; DIGEST_BYTES],
        refusal: JudgingUnitsKeyRefused,
    },
}

impl JudgedProductCodeOfThisBinary {
    /// 从 [`judged_product_code_key`] 交回的那一把取：算得出的照键，拒算的取这个进程的占位。
    #[must_use]
    pub fn of_the_key(key: Result<JudgingUnitsKey, JudgingUnitsKeyRefused>) -> Self {
        match key {
            Ok(key) => Self::Keyed {
                fingerprint: *key.fingerprint(),
            },
            Err(refusal) => Self::RefusedInThisProcess {
                placeholder:
                    *crate::crash_amplification::placeholder_judging_units_key_of_the_refused(
                        &JudgingUnits::JudgedProductCode,
                    )
                    .fingerprint(),
                refusal,
            },
        }
    }

    /// 进版本文字与对拍结论键的那一行。
    #[must_use]
    pub fn key_text(&self) -> String {
        match self {
            Self::Keyed { fingerprint } => {
                format!("judged_product_code={}\n", hexadecimal_text(fingerprint))
            }
            Self::RefusedInThisProcess { placeholder, .. } => format!(
                "judged_product_code_refused_in_this_process={}\n",
                hexadecimal_text(placeholder)
            ),
        }
    }
}

/// 这个二进制里判器引到的被检测产品代码那一把（[`JudgedProductCodeOfThisBinary::of_the_key`]，读编进二进制的那份表，取一次）。
#[must_use]
pub fn judged_product_code_of_this_binary() -> &'static JudgedProductCodeOfThisBinary {
    static JUDGED_PRODUCT_CODE: std::sync::OnceLock<JudgedProductCodeOfThisBinary> =
        std::sync::OnceLock::new();
    JUDGED_PRODUCT_CODE
        .get_or_init(|| JudgedProductCodeOfThisBinary::of_the_key(judged_product_code_key()))
}

/// 这个进程里照判不入库的结论版本：判器引到的被检测产品代码那一组拒算时是判器判出的三种结论各一把（都从占位拼的），算得出键时一把都没有。
/// 判定库开库时登记（`crate::verdict_store::VerdictStore` 装起来的那一下）。
#[must_use]
pub fn conclusion_versions_kept_out_of_the_library_in_this_process() -> Vec<[u8; DIGEST_BYTES]> {
    conclusion_versions_kept_out_of_the_library_under(judged_product_code_of_this_binary())
}

/// 同 [`conclusion_versions_kept_out_of_the_library_in_this_process`]，判器引到的被检测产品代码那一把由调用方给（单测造拒算用）。
#[must_use]
pub fn conclusion_versions_kept_out_of_the_library_under(
    judged_product_code: &JudgedProductCodeOfThisBinary,
) -> Vec<[u8; DIGEST_BYTES]> {
    match judged_product_code {
        JudgedProductCodeOfThisBinary::Keyed { .. } => Vec::new(),
        JudgedProductCodeOfThisBinary::RefusedInThisProcess { .. } => {
            JudgedConclusion::EVERY_JUDGED_CONCLUSION
                .iter()
                .map(|conclusion| {
                    ConclusionVersion::of_the_judged_conclusion(*conclusion, judged_product_code).0
                })
                .collect()
        }
    }
}

/// 一种结论的版本（SHA-256）。判器判出的那几种（崩溃镜像、给定状态、观察者交回的盘上的池级 checker）由结论种类与判器引到的被检测产品代码
/// （[`JudgedProductCodeOfThisBinary`]）定：判器自己的代码（CPU 判器、GPU 判器、池级 checker、记录核对器）不进，改判器不作废已判的块，
/// 判法改了对不对靠对拍与复用哨兵抓；判器调到的产品恢复、挂载与格式代码进，改了它们已判的块现判。观察者那一种是它的判定项标识、它引到的
/// 被检测产品代码与另读的输入拼的（`crate::crash_amplification::ObserverVersions`）。同一块在几把版本下的结论并存，复用只认同一版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConclusionVersion(pub [u8; DIGEST_BYTES]);

impl ConclusionVersion {
    /// 判崩溃镜像那一种结论（崩溃放量与崩溃枚举用例按块判的）的版本：这一种结论的名字拼这个二进制里判器引到的被检测产品代码。
    #[must_use]
    pub fn of_the_crash_image_judgement() -> Self {
        Self::of_the_judged_conclusion(
            JudgedConclusion::CrashImageJudgement,
            judged_product_code_of_this_binary(),
        )
    }

    /// 判给定状态那一种结论（给定状态表引擎判的）的版本：这一种结论的名字拼这个二进制里判器引到的被检测产品代码。
    #[must_use]
    pub fn of_the_given_states_judgement() -> Self {
        Self::of_the_judged_conclusion(
            JudgedConclusion::GivenStatesJudgement,
            judged_product_code_of_this_binary(),
        )
    }

    /// 观察者交回的盘上跑池级 checker 那一种结论的版本：这一种结论的名字拼这个二进制里判器引到的被检测产品代码。
    #[must_use]
    pub fn of_the_pool_checker_on_the_images_after_the_observers() -> Self {
        Self::of_the_judged_conclusion(
            JudgedConclusion::PoolCheckerOnTheImagesAfterTheObservers,
            judged_product_code_of_this_binary(),
        )
    }

    /// 判器判出的一种结论在给定的判器引到的被检测产品代码下的版本：`singlefs conclusion version`、种类名与那一把那一行拼起来取 SHA-256。
    #[must_use]
    pub fn of_the_judged_conclusion(
        conclusion: JudgedConclusion,
        judged_product_code: &JudgedProductCodeOfThisBinary,
    ) -> Self {
        Self(sha256_digest(
            format!(
                "singlefs conclusion version\nkind={}\n{}",
                conclusion.kind_text(),
                judged_product_code.key_text()
            )
            .as_bytes(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 场景：判器引到的被检测产品代码取两把不同的指纹与一把拒算，三种结论各取版本。
    /// 预期：同一把下三种结论两两不同；同一种结论换一把指纹版本就变（改产品恢复，已判的块现判）；版本就是种类名接那一把那一行的摘要；
    /// 拒算的那一把下的版本与算得出的都不同，三把都登成照判不入库，算得出的一把都不登。
    /// 判别力（`crates/mutations.tsv`）：给定状态那一把取成崩溃镜像那一把，这条红在「两两不同」那一句；版本文字不拼判器引到的产品代码，
    /// 这条红在「换一把指纹版本就变」那一句。
    #[test]
    fn the_judged_conclusion_versions_take_the_judged_product_code_and_differ_by_kind() {
        let keyed = |byte: u8| JudgedProductCodeOfThisBinary::Keyed {
            fingerprint: [byte; DIGEST_BYTES],
        };
        let versions_under = |judged_product_code: &JudgedProductCodeOfThisBinary| {
            JudgedConclusion::EVERY_JUDGED_CONCLUSION.map(|conclusion| {
                ConclusionVersion::of_the_judged_conclusion(conclusion, judged_product_code)
            })
        };
        let under_one = versions_under(&keyed(1));
        assert_ne!(under_one[0], under_one[1], "崩溃镜像与给定状态两把不同");
        assert_ne!(
            under_one[0], under_one[2],
            "崩溃镜像与观察者盘上的池级 checker 两把不同"
        );
        assert_ne!(
            under_one[1], under_one[2],
            "给定状态与观察者盘上的池级 checker 两把不同"
        );
        let under_two = versions_under(&keyed(2));
        for (before, after) in under_one.iter().zip(&under_two) {
            assert_ne!(
                before, after,
                "判器引到的被检测产品代码换一把指纹，版本就变"
            );
        }
        assert_eq!(
            under_one[0].0,
            sha256_digest(
                format!(
                    "singlefs conclusion version\nkind=crash image judgement\njudged_product_code={}\n",
                    hexadecimal_text(&[1; DIGEST_BYTES])
                )
                .as_bytes()
            ),
            "崩溃镜像那一把就是种类名接判器引到的产品代码那一行的摘要"
        );
        let refused = JudgedProductCodeOfThisBinary::of_the_key(Err(
            JudgingUnitsKeyRefused::RefusedAtTheBuild {
                units: JudgingUnits::JudgedProductCode,
                refusal_lines: vec![
                    "refused\tcurrent-exe\tcrates/a/src/lib.rs:3\tcurrent_exe".to_string()
                ],
            },
        ));
        let under_the_refused = versions_under(&refused);
        for version in &under_the_refused {
            assert!(
                !under_one.contains(version) && !under_two.contains(version),
                "拒算的那一把下的版本撞不上算得出的"
            );
        }
        assert_eq!(
            conclusion_versions_kept_out_of_the_library_under(&refused),
            under_the_refused.map(|version| version.0).to_vec(),
            "拒算时三种结论的版本都照判不入库"
        );
        assert!(
            conclusion_versions_kept_out_of_the_library_under(&keyed(1)).is_empty(),
            "算得出键时一把都不登"
        );
    }

    #[test]
    fn observer_name_takes_one_to_u16_maximum_bytes() {
        assert_eq!(
            ObserverName::new(""),
            Err(ObserverNameRejected {
                text: String::new()
            })
        );
        let longest_name = "n".repeat(usize::from(u16::MAX));
        assert!(
            ObserverName::new(&longest_name).is_ok(),
            "u16::MAX 字节的名字收"
        );
        assert!(
            ObserverName::new(&format!("{longest_name}n")).is_err(),
            "超过 u16::MAX 字节的名字不收"
        );
    }
}
