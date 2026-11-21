//! 每次 API 调用的结局与理想模型的判、停机核对、阳性对照与关掉 oracle 的开关。

use super::*;

// ───────────────────────────── 驱动方记的调用（calls.tsv、call-outcomes.tsv） ─────────────────────────────

/// 一次调用的结局：成功；实现交回一个错误（成员名取 `Debug` 文本的第一个标识符）；实现 panic（实现自崩，登记第一节读法 5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CallOutcome {
    Succeeded,
    ReturnedError(String),
    Panicked(String),
}

impl CallOutcome {
    /// `calls.tsv` 最后一列：E170（O3规约执行器第一版） 登记 5.3 写的 `ok` / `err`；panic 的流不导出，这里不会写到 panic。
    pub(super) fn calls_file_word(&self) -> &'static str {
        match self {
            Self::Succeeded => "ok",
            Self::ReturnedError(_) | Self::Panicked(_) => "err",
        }
    }
    /// `call-outcomes.tsv` 里的结局列：成功是 `ok`，错误是 `err:<成员>`，panic 是 `panic`（消息另写结果行）。
    pub(super) fn outcome_word(&self) -> String {
        match self {
            Self::Succeeded => "ok".to_string(),
            Self::ReturnedError(member) => format!("err:{member}"),
            Self::Panicked(_) => "panic".to_string(),
        }
    }
}

/// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ModelVerdictOfCall {
    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`crates/singlefs-verification/src/model.rs` 里已有 `answer_unmount`，本装置还没改调它，卸载这一步照旧不问模型；改调归 E171 第 1 次重跑）。
    NotModelled,
    /// 对得上，模型往前走。
    Agrees,
    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
    Disagrees {
        aspect: String,
        model_answer: String,
        implementation_answer: String,
    },
    /// 前面有一次对不上（或前面有一次调用出了错），这次不比。
    SkippedAfterTheModelStopped,
}

impl ModelVerdictOfCall {
    pub(super) fn word(&self) -> String {
        match self {
            Self::NotModelled => "not_modelled".to_string(),
            Self::Agrees => "agrees".to_string(),
            Self::Disagrees { aspect, .. } => format!("disagrees:{aspect}"),
            Self::SkippedAfterTheModelStopped => "skipped_after_the_model_stopped".to_string(),
        }
    }
}

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度，都从 mkfs 之后数）、结局、模型的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecordedCall {
    pub(super) label: String,
    pub(super) first_operation: usize,
    pub(super) return_point: usize,
    pub(super) outcome: CallOutcome,
    pub(super) model: ModelVerdictOfCall,
}

/// 一条停机条款 S1 的回比：本地值与 crates 那一侧的值，两边按文本比。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StopCheck {
    pub(super) name: String,
    pub(super) local: String,
    pub(super) crates: String,
}

impl StopCheck {
    pub(super) fn new(
        name: &str,
        local: impl std::fmt::Debug,
        crates: impl std::fmt::Debug,
    ) -> Self {
        Self {
            name: name.to_string(),
            local: format!("{local:?}"),
            crates: format!("{crates:?}"),
        }
    }
    pub(super) fn holds(&self) -> bool {
        self.local == self.crates
    }
}

/// 阳性对照要改的那一处观测（登记 5.6「坏实现」行的阳性一栏）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PositiveControl {
    /// 流二第一次 `publish` 的观测里那一版的文件内容末字节翻一位，再交 `judge_and_advance`。
    ComparisonOfTheCalls,
    /// 批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位，再交 `crash_recovery_disagreement`。
    CrashJudge,
    /// 流一全持久态叠已知坏镜像族第一条，再交 `check_pool_image`。
    PoolChecker,
}

impl PositiveControl {
    pub(super) fn parse(word: &str) -> Option<Self> {
        match word {
            "O1-对拍" => Some(Self::ComparisonOfTheCalls),
            "O1-崩溃判官" => Some(Self::CrashJudge),
            "O2" => Some(Self::PoolChecker),
            _ => None,
        }
    }
}

/// 关掉的那条 oracle（Q3-核，登记第一节读法 9）：关掉的列不算、写成 `off`，别的两列照常。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DisabledOracle {
    ComparisonAndCrashJudge,
    PoolChecker,
}

/// 一次跑的开关：阳性对照、关一条、判取样态与坏镜像的线程数（不进产物，`None` 取每台默认重活线程数）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunSwitches {
    pub(super) positive_control: Option<PositiveControl>,
    pub(super) disabled_oracle: Option<DisabledOracle>,
    pub(super) sampling_threads: Option<NonZeroUsize>,
}

/// 实现 panic 的消息：`String` 或 `&str`，别的写 `non_text_panic`。
pub(super) fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "non_text_panic".to_string())
}

/// 错误值 `Debug` 文本的第一个标识符（`Variant(..)`、`Variant { .. }`、`Variant`）：成员名。
pub(super) fn error_member_of(error: &dyn std::fmt::Debug) -> String {
    let text = format!("{error:?}");
    text.chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect()
}

#[cfg(test)]
mod tests;
