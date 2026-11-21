//! 每次 API 调用的结局、返回值摘要与理想模型的判，停机核对、阳性对照与关掉 oracle 的开关。

use super::*;

// ───────────────────────────── 驱动方记的调用（calls.tsv、call-outcomes.tsv） ─────────────────────────────

/// 一次调用的结局：成功；实现交回一个错误（成员名取 `Debug` 文本的第一个标识符）；实现 panic（实现自崩，登记第一节读法 5）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CallOutcome {
    Succeeded,
    ReturnedError(String),
    Panicked(String),
    /// Q5 那一形（读法 7）里截断处还没返回的那次调用：实现在装置里照常跑完了，那一形的写流截在它发出的第一道屏障之前，它在途。
    InFlightAtTheTruncation,
}

impl CallOutcome {
    /// `calls.tsv` 最后一列：E170（O3规约执行器第一版） 登记 5.3 写的 `ok` / `err`；panic 那次调用也写 `err`
    /// （第 1 次重跑登记第一节读法 5：panic 与报错同样截断照判，那份格式只有这两个词）。
    pub(super) fn calls_file_word(&self) -> &'static str {
        match self {
            Self::Succeeded => "ok",
            Self::ReturnedError(_) | Self::Panicked(_) | Self::InFlightAtTheTruncation => "err",
        }
    }
    /// `call-outcomes.tsv` 里的结局列：成功是 `ok`，错误是 `err:<成员>`，panic 是 `panic`（消息另写结果行）。
    pub(super) fn outcome_word(&self) -> String {
        match self {
            Self::Succeeded => "ok".to_string(),
            Self::ReturnedError(member) => format!("err:{member}"),
            Self::Panicked(_) => "panic".to_string(),
            Self::InFlightAtTheTruncation => "in_flight".to_string(),
        }
    }
}

/// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ModelVerdictOfCall {
    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）。
    NotModelled,
    /// 对得上，模型往前走。
    Agrees,
    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
    Disagrees {
        aspect: &'static str,
        model_answer: String,
        implementation_answer: String,
    },
    /// 前面有一次对不上（或前面有一次调用出了错或 panic），这次不比。
    SkippedAfterTheModelStopped,
    /// 这次调用 panic：实现自崩、不比，不红不绿（读法 5）；模型停在这次调用。
    NotComparedBecauseTheImplementationCrashed,
    /// 调用成功了，而胶水造不出交给模型的观测（`model_comparison::model_unit_role` 里模型还不罩的几种单元在这一版里出现，胶水 panic）：
    /// 不比、模型停在这次调用。读法 6 的三处来源之外的一处，停机 S5 记它（第一段流三跨一片叶那一次撞上过；理想模型那件修理之后
    /// 胶水按片记 inode 叶容器的角色、不再断言，登记修订 2 第 2.2 节：合入后的树上再出现就是 S5）。
    ModelCannotTakeTheObservation { what: String },
}

impl ModelVerdictOfCall {
    pub(super) fn word(&self) -> String {
        match self {
            Self::NotModelled => "not_modelled".to_string(),
            Self::Agrees => "agrees".to_string(),
            Self::Disagrees { aspect, .. } => format!("disagrees:{aspect}"),
            Self::SkippedAfterTheModelStopped => "skipped_after_the_model_stopped".to_string(),
            Self::NotComparedBecauseTheImplementationCrashed => {
                "implementation_crashed_not_compared".to_string()
            }
            Self::ModelCannotTakeTheObservation { .. } => {
                "model_cannot_take_the_observation".to_string()
            }
        }
    }
    /// O1-对拍判过这次调用（读法 5「可判的调用」：`agrees` 或 `disagrees`；`not_modelled` 不算）。
    pub(super) fn was_judged_by_the_comparison(&self) -> bool {
        match self {
            Self::Agrees | Self::Disagrees { .. } => true,
            Self::NotModelled
            | Self::SkippedAfterTheModelStopped
            | Self::NotComparedBecauseTheImplementationCrashed
            | Self::ModelCannotTakeTheObservation { .. } => false,
        }
    }
}

/// 一次成功调用的交回值（读法 4）：`{:?}` 文本的 sha256，与顶层字段逐个的 (字段名, sha256)（字段名照 `{:?}` 文本切出，
/// 元组成员与没有名字的写 `#<序>`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReturnValueDigest {
    pub(super) whole: String,
    pub(super) fields: Vec<(String, String)>,
}

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度，都从 mkfs 之后数）、结局、模型的判定、
/// 成功时的交回值摘要、这次调用之后装置手里由实现交出的对象各自的 (名字, `{:?}` 的 sha256)（读法 4，只记、不进触达）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RecordedCall {
    pub(super) label: &'static str,
    pub(super) first_operation: usize,
    pub(super) return_point: usize,
    pub(super) outcome: CallOutcome,
    pub(super) model: ModelVerdictOfCall,
    pub(super) return_value: Option<ReturnValueDigest>,
    pub(super) memory_objects: Vec<(&'static str, String)>,
    /// 发布调用交给实现的内容（RefFS 子列要它：建第一个文件或覆盖成哪几个字节，读法 1）；别的调用没有。
    pub(super) published_content: Option<PublishedContent>,
    /// 胶水交给模型的观测 `{:?}` 的 sha256（几份按次序用 `+` 接起来）；这次调用没交观测（取号、暖机、panic）是 `None`。
    pub(super) observation_sha256: Option<String>,
}

/// 发布调用改了什么内容：第一个文件建成什么、覆盖成什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum PublishedContent {
    FirstFile(Vec<u8>),
    Overwrite(Vec<u8>),
}

/// 切 `{:?}` 文本的顶层字段：第一个开括号（`{`、`(`、`[`）之内、深度 1 上按逗号切，引号里的不算；每一段有 `名字: ` 起头的取名字，
/// 没有的写 `#<序>`。没有括号的（单元成员、数）只有一段 `#0`，就是整份文本。
fn top_level_fields_of_debug_text(text: &str) -> Vec<(String, String)> {
    let Some(open) = text.find(['{', '(', '[']) else {
        return vec![("#0".to_string(), text.to_string())];
    };
    let mut pieces: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut inside_quotes = false;
    let mut escaped = false;
    for character in text[open..].chars() {
        if inside_quotes {
            current.push(character);
            match (escaped, character) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => inside_quotes = false,
                (false, _) => {}
            }
            continue;
        }
        match character {
            '{' | '(' | '[' => {
                depth += 1;
                if depth > 1 {
                    current.push(character);
                }
            }
            '}' | ')' | ']' => {
                depth = depth.saturating_sub(1);
                if depth >= 1 {
                    current.push(character);
                } else {
                    break;
                }
            }
            ',' if depth == 1 => pieces.push(std::mem::take(&mut current)),
            '"' => {
                inside_quotes = true;
                current.push(character);
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        pieces.push(current);
    }
    pieces
        .iter()
        .enumerate()
        .map(|(position, piece)| {
            let piece = piece.trim();
            let named = piece.split_once(": ").filter(|(name, _)| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|character| character.is_alphanumeric() || character == '_')
            });
            match named {
                Some((name, value)) => (name.to_string(), value.to_string()),
                None => (format!("#{position}"), piece.to_string()),
            }
        })
        .collect()
}

/// 交回值的摘要：整份 `{:?}` 文本与顶层字段逐个取 sha256。
pub(super) fn return_value_digest_of_debug_text(text: &str) -> ReturnValueDigest {
    ReturnValueDigest {
        whole: sha256_hexadecimal(text.as_bytes()),
        fields: top_level_fields_of_debug_text(text)
            .into_iter()
            .map(|(name, value)| (name, sha256_hexadecimal(value.as_bytes())))
            .collect(),
    }
}

/// 交回值的 `{:?}` 文本：`Debug` 有的照打。
pub(super) fn debug_text<Value: std::fmt::Debug>(value: &Value) -> String {
    format!("{value:?}")
}

/// `mount::Unmounted` 与它里面的 `UnmountRaisedTheFloor` 不带 `Debug`：照 `#[derive(Debug)]` 会打出的样子逐个字段拼（字段名与次序照
/// `crates/singlefs-core/src/mount.rs` 里的声明），每个字段的值用它自己的 `Debug`。
pub(super) fn unmounted_debug_text(unmounted: &Unmounted) -> String {
    match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => format!(
            "FloorRaisedToTheCurrentVersion(UnmountRaisedTheFloor {{ rollback_floor: {:?}, system_configuration_writes_before_the_first_publish: {:?}, publishes: {:?}, reclaimed: {:?}, abandoned_roots_unreadable: {:?} }})",
            raised.rollback_floor,
            raised.system_configuration_writes_before_the_first_publish,
            raised.publishes,
            raised.reclaimed,
            raised.abandoned_roots_unreadable
        ),
        Unmounted::NothingWrittenOnAVersionWithoutFile { current } => {
            format!("NothingWrittenOnAVersionWithoutFile {{ current: {current:?} }}")
        }
    }
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

/// 阳性对照要改的那一处（登记 5.2 那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PositiveControl {
    /// 流二第一次 `publish` 的观测里那一版的文件内容末字节翻一位，再交 `judge_and_advance`。
    ComparisonOfTheCalls,
    /// 流二卸载那一次交给模型的观测里，第一条根的 F 加一（16-2 的观测 `observed_unmount` 是那几次带新 F 的空发布的根，
    /// `ObservedRoot::rollback_floor` 就是卸载要比的新 F；第 1 次重跑登记第十二节修订写明）。
    ComparisonOfTheUnmount,
    /// 每条流批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位，再交 `crash_recovery_disagreement`。
    CrashJudge,
    /// 同一个态（每条流批 1 里第一个读回了文件的态），交给 RefFS 的读回内容末字节翻一位。
    ReferenceFileSystem,
    /// 流一全持久态叠已知坏镜像族第一条，再交 `check_pool_image`。
    PoolChecker,
    /// 流二第 2 次调用（从 0 数）的交回值在记之前复制一份、把那一版根的 checkpoint_txg 加一（只改记下的那份，不改交给胶水的）。
    ReturnValue,
    /// 流二第 2 次调用之后记分配器摘要之前，在副本上多分配一槽（只改记下的那份）。
    MemoryObject,
    /// 流二第 3 次调用（从 0 数）的闭包里先 panic（装置这一侧注入，不改实现）。
    PanicInTheCall,
    /// 基线流一全持久态：每块盘两槽系统配置的 journal_tail 改成所选那一版末条记录的计数器加一、重编、自证过，再起可写挂载探针。
    Witness,
}

impl PositiveControl {
    pub(super) fn parse(word: &str) -> Option<Self> {
        match word {
            "O1-对拍" => Some(Self::ComparisonOfTheCalls),
            "O1-对拍-卸载" => Some(Self::ComparisonOfTheUnmount),
            "O1-崩溃判官" => Some(Self::CrashJudge),
            "RefFS" => Some(Self::ReferenceFileSystem),
            "O2" => Some(Self::PoolChecker),
            "交回值" => Some(Self::ReturnValue),
            "内存对象" => Some(Self::MemoryObject),
            "panic" => Some(Self::PanicInTheCall),
            "见证" => Some(Self::Witness),
            _ => None,
        }
    }
}

/// 阳性对照「交回值」「内存对象」改的那一次调用、「panic」注入的那一次调用（流二，从 0 数）。
pub(super) const RETURN_VALUE_CONTROL_CALL: usize = 2;
pub(super) const PANIC_CONTROL_CALL: usize = 3;

/// 关掉的那条 oracle（Q3-核，登记第一节读法 9）：关掉的列不算、写成 `off`，别的两列照常。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DisabledOracle {
    ComparisonAndCrashJudge,
    PoolChecker,
}

/// 一次跑的开关：阳性对照、关一条、可写挂载探针（Q4，读法 8）。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunSwitches {
    pub(super) positive_control: Option<PositiveControl>,
    pub(super) disabled_oracle: Option<DisabledOracle>,
    pub(super) writable_mount_probe: bool,
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
