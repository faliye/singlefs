//! 崩溃全量的积压登记表（`.claude/kb/milestone/04-fourth-txn.md` 出口 10「这一里程碑立积压登记表……表放在 checker 档 crate 里」）：
//! 表是本包根上的 `crash-full-run-backlog.tsv`，一行一件：崩溃点重放合表（`.claude/gate.d/crash-point-replay-items.tsv`）一项里的一条
//! `crash-case:`，或一个崩溃放量分项（`scripts/crash/crash-amplification.sh` 的 ITEMS）。这里只有表的读、写、核、取件与合判法键，
//! 不读盘、不跑任何崩溃枚举；读文件、写回、从编进二进制的判器键表取今天的判法键都在小程序 `src/bin/crash_full_run_backlog.rs`。
//!
//! 展开的次序（主 agent 2026-10-26 定，记在 `records/2026-10-13-里程碑四预开发.md`「用户弹窗四条」那一行）：照合表的项序逐项走，
//! 每项先列第二列的 `crash-case:`（照第二列的次序），再列第三列的放量分项（照 ITEMS 的次序）。表头一行写「照展开次序登记的行数」：
//! 开头那么多行照展开的次序排，之后的是建表以后追加的；新一版与上一版比（[`problems_against_the_previous_table`]），上一版的件次序不变、
//! 新件都在它们后面（出口 10「新增跑不完的那一截进积压登记表末尾」）。
//!
//! 判法键：判法单元入口登记表（`crate::judging_units_entries`）登记的全部判器组（镜像判定、给定状态、每一个观察者，连同 GPU 判器、
//! 两种核对、录入、观察者结论拼块与两种对拍的比法）各自在这一份二进制里的单元键，按组名排好合成的一个指纹（[`combined_judging_key_of_the_groups`]；
//! 主 agent 2026-10-26 定取合指纹，记在 `records/2026-10-13-里程碑四预开发.md`「出口 10 积压登记表实现员交回」那一行）：任何一组的键变了，
//! 这一把就变。只取镜像判定那一把时，观察者的键变了、给定状态那几条用例的判定版本变了，行都照旧读，罩不住。
//! 一行的判法键与今天的对不上，这一行的进度当「未跑」读（出口 10 ①「单元键变了的块先跑」）。
//! 罩住（用户 2026-10-26 定「算：在今天的单元键下有判定就算罩住」）：键相同、判过的块复用了也算判了，所以取件时一行的态数整行算，
//! 不扣复用的块；在今天的判法键下进度是「全绿」的行不再取。

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use singlefs_harness::hexadecimal::hexadecimal_text;
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
use singlefs_unit_graph::entry_groups::{ReadBackEntryGroupTable, ReadBackEntryGroupVerdict};

use crate::judging_units_entries::{
    observer_group_name, JUDGING_UNITS_ENTRY_GROUPS, OBSERVER_ENTRIES,
};

/// 表在仓里的位置（与本包清单同一层，出口 10「表放在 checker 档 crate 里」）。
pub const CRASH_FULL_RUN_BACKLOG_TABLE_PATH_IN_REPOSITORY: &str =
    "crates/singlefs-checker-tier/crash-full-run-backlog.tsv";

/// 崩溃点重放合表在仓里的位置（展开的次序照它）。
pub const CRASH_POINT_REPLAY_ITEMS_TABLE_PATH_IN_REPOSITORY: &str =
    ".claude/gate.d/crash-point-replay-items.tsv";

/// 出口 10「② 再从积压登记表里补跑，至少 50 亿态」。
pub const BACKLOG_STATES_AT_LEAST_PER_MILESTONE: u64 = 5_000_000_000;

/// 出口 10「这一里程碑至少跑 100 亿态」（新增与积压合起来）。
pub const STATES_AT_LEAST_PER_MILESTONE: u64 = 10_000_000_000;

/// 列头那一行（九列，制表符隔开）。
pub const BACKLOG_TABLE_COLUMN_HEADER: &str =
    "序号\t项名\t部分\t键\t归属\t态数\t判法键\t进度\t最后一次";

/// 列头之前那一行的头：开头几行照展开的次序登记（之后的是追加的）。
pub const ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX: &str = "照展开次序登记的行数\t";

/// 一格里正文与出处之间的分隔（制表符是列分隔，格里不许有）。
const CELL_PART_SEPARATOR: char = '；';
const NOT_COUNTED_TEXT: &str = "未数";
const NOT_RECORDED_TEXT: &str = "未记";
const TO_BE_REJUDGED_UNDER_THE_COMBINED_KEY_TEXT: &str = "待重判";
const NOT_RUN_TEXT: &str = "未跑";
const ALL_GREEN_TEXT: &str = "全绿";
const RED_PREFIX: &str = "红：";
const PARTIAL_PREFIX: &str = "部分：已判块 ";
const PARTIAL_MIDDLE: &str = " / 共 ";
const PARTIAL_SUFFIX: &str = " 块";
const NEVER_RUN_TEXT: &str = "无";
const MILESTONE_THREE_AND_EARLIER_TEXT: &str = "m3-and-earlier";
const MILESTONE_PREFIX: char = 'm';
const CRASH_CASE_KEY_PREFIX: &str = "crash-case:";
/// 合表第二、三列「没有」的写法。
const EMPTY_COLUMN_OF_THE_MERGED_ITEMS: &str = "-";
/// 里程碑号从 4 起才单记（三及以前合成一档 `m3-and-earlier`：积压登记表是里程碑四立的）。
const SMALLEST_SEPARATELY_REGISTERED_MILESTONE: u32 = 4;

/// 一件属于合表一项的哪一部分。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartOfAnItem {
    /// 合表第二列：`.claude/gate.d/stage-inputs.tsv` 的一条 `crash-case:`，由门禁 checker-tier-crash-point-replay 跑。
    CrashPointReplay,
    /// 合表第三列：`crash-amplification.sh` ITEMS 的一个分项，由 `crash-amplification.sh --item` 跑。
    CrashAmplification,
}

impl PartOfAnItem {
    /// 表里「部分」那一列的写法。
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::CrashPointReplay => "crash-point-replay",
            Self::CrashAmplification => "crash-amplification",
        }
    }

    fn of_the_text(text: &str) -> Option<Self> {
        [Self::CrashPointReplay, Self::CrashAmplification]
            .into_iter()
            .find(|part| part.text() == text)
    }
}

/// 一件：合表的项名、哪一部分、键（`crash-case:<名>` 或放量分项名）。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrashFullRunPart {
    pub item_name: String,
    pub part: PartOfAnItem,
    pub key: String,
}

impl CrashFullRunPart {
    /// 报问题时点名这一件的写法。
    #[must_use]
    pub fn text(&self) -> String {
        format!("{} {} {}", self.item_name, self.part.text(), self.key)
    }
}

/// 表里的序号（从 1 起逐行加一）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowSequence(pub u64);

/// 里程碑号，从 4 起（构造时核）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeparatelyRegisteredMilestone(u32);

impl SeparatelyRegisteredMilestone {
    /// 小于 4 的交 `None`（三及以前合成 [`RegisteredInMilestone::MilestoneThreeAndEarlier`]）。
    #[must_use]
    pub fn new(milestone_number: u32) -> Option<Self> {
        (milestone_number >= SMALLEST_SEPARATELY_REGISTERED_MILESTONE)
            .then_some(Self(milestone_number))
    }

    #[must_use]
    pub fn milestone_number(self) -> u32 {
        self.0
    }

    /// `m<号>`。
    #[must_use]
    pub fn text(self) -> String {
        format!("{MILESTONE_PREFIX}{}", self.0)
    }

    /// `m<号>` 的逆：号要能原样写回（不认前导 0、正号），且不小于 4。
    #[must_use]
    pub fn of_the_text(text: &str) -> Option<Self> {
        let number_text = text.strip_prefix(MILESTONE_PREFIX)?;
        let milestone_number = number_text.parse::<u32>().ok()?;
        if milestone_number.to_string() != number_text {
            return None;
        }
        Self::new(milestone_number)
    }
}

/// 一件第一次登记属于哪个里程碑（「归属」那一列；只用来分「这一里程碑新增」还是「积压」，登了之后不改）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegisteredInMilestone {
    MilestoneThreeAndEarlier,
    Milestone(SeparatelyRegisteredMilestone),
}

impl RegisteredInMilestone {
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::MilestoneThreeAndEarlier => MILESTONE_THREE_AND_EARLIER_TEXT.to_string(),
            Self::Milestone(milestone) => milestone.text(),
        }
    }

    #[must_use]
    pub fn of_the_text(text: &str) -> Option<Self> {
        if text == MILESTONE_THREE_AND_EARLIER_TEXT {
            return Some(Self::MilestoneThreeAndEarlier);
        }
        SeparatelyRegisteredMilestone::of_the_text(text).map(Self::Milestone)
    }
}

/// 「态数」那一列：整域的态数现数（带出处），或还没数（写明为什么、约数在哪）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatesOfThePart {
    Counted { states: u64, provenance: String },
    NotCounted { provenance: String },
}

impl StatesOfThePart {
    /// 数过的态数；没数交 `None`。
    #[must_use]
    pub fn counted_states(&self) -> Option<u64> {
        match self {
            Self::Counted { states, .. } => Some(*states),
            Self::NotCounted { .. } => None,
        }
    }

    /// `<态数>；<出处>` 或 `未数；<出处>`。
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::Counted { states, provenance } => {
                format!("{states}{CELL_PART_SEPARATOR}{provenance}")
            }
            Self::NotCounted { provenance } => {
                format!("{NOT_COUNTED_TEXT}{CELL_PART_SEPARATOR}{provenance}")
            }
        }
    }

    /// 表里的写法的逆（小程序写回时也照它读命令行给的态数）。
    ///
    /// # Errors
    /// 不是「<态数>；<出处>」或「未数；<出处>」；出处空着。
    pub fn of_the_text(text: &str) -> Result<Self, String> {
        let Some((head, provenance)) = text.split_once(CELL_PART_SEPARATOR) else {
            return Err(format!(
                "态数「{text}」不是「<态数>{CELL_PART_SEPARATOR}<出处>」或「{NOT_COUNTED_TEXT}{CELL_PART_SEPARATOR}<出处>」"
            ));
        };
        if provenance.is_empty() {
            return Err(format!("态数「{text}」没写出处"));
        }
        if head == NOT_COUNTED_TEXT {
            return Ok(Self::NotCounted {
                provenance: provenance.to_string(),
            });
        }
        let states = decimal_number_of_the_text(head)
            .ok_or_else(|| format!("态数「{head}」不是十进制整数、也不是「{NOT_COUNTED_TEXT}」"))?;
        Ok(Self::Counted {
            states,
            provenance: provenance.to_string(),
        })
    }
}

/// 「判法键」那一列：判完那一趟的合判法键（[`combined_judging_key_of_the_groups`]，64 位小写十六进制），或没记，或待重判。
/// 待重判是改取合判法键之前判的行：那一趟只记了镜像判定那一把的头 8 字节，那一趟的合判法键取不回来，这一行在任何一把合判法键下都不算今天的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JudgingKeyOfThePart {
    NotRecorded,
    ToBeRejudgedUnderTheCombinedKey,
    Fingerprint([u8; DIGEST_BYTES]),
}

impl JudgingKeyOfThePart {
    /// 这一行的判法键是不是今天这一把；没记的、待重判的都不是。
    #[must_use]
    pub fn is_todays(&self, today: &TodaysJudgingKey) -> bool {
        match self {
            Self::NotRecorded | Self::ToBeRejudgedUnderTheCombinedKey => false,
            Self::Fingerprint(bytes) => *bytes == today.0,
        }
    }

    /// 小写十六进制，或 `未记`、`待重判`。
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::NotRecorded => NOT_RECORDED_TEXT.to_string(),
            Self::ToBeRejudgedUnderTheCombinedKey => {
                TO_BE_REJUDGED_UNDER_THE_COMBINED_KEY_TEXT.to_string()
            }
            Self::Fingerprint(bytes) => hexadecimal_text(bytes),
        }
    }

    /// 表里的写法的逆（小程序写回时也照它读命令行给的判法键）。
    ///
    /// # Errors
    /// 不是「未记」「待重判」，也不是 32 字节的小写十六进制（改取合判法键之前那种只记头 8 字节的写法不再认）。
    pub fn of_the_text(text: &str) -> Result<Self, String> {
        if text == NOT_RECORDED_TEXT {
            return Ok(Self::NotRecorded);
        }
        if text == TO_BE_REJUDGED_UNDER_THE_COMBINED_KEY_TEXT {
            return Ok(Self::ToBeRejudgedUnderTheCombinedKey);
        }
        let bytes = bytes_of_the_lowercase_hexadecimal_text(text).ok_or_else(|| {
            format!(
                "判法键「{text}」不是小写十六进制、也不是「{NOT_RECORDED_TEXT}」「{TO_BE_REJUDGED_UNDER_THE_COMBINED_KEY_TEXT}」"
            )
        })?;
        <[u8; DIGEST_BYTES]>::try_from(bytes.as_slice())
            .map(Self::Fingerprint)
            .map_err(|_length_mismatch| {
                format!(
                    "判法键「{text}」是 {} 字节：合判法键只认 {DIGEST_BYTES} 字节的全长（改取合判法键之前判的行写「{TO_BE_REJUDGED_UNDER_THE_COMBINED_KEY_TEXT}」）",
                    bytes.len()
                )
            })
    }
}

/// 今天的判法键：这一份二进制里的合判法键（小程序 `today-judging-key` 经 [`todays_judging_key_of_the_compiled_table`] 取）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TodaysJudgingKey(pub [u8; DIGEST_BYTES]);

/// 一组判法单元在这一份二进制里的键，合判法键的一份输入。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JudgingUnitsGroupKeyAtTheBuild {
    Keyed([u8; DIGEST_BYTES]),
    /// 这一组的入口在这一趟开的特性下一个都不编进来（例：不开 `gpu` 编时的 GPU 判器与 GPU 核对）：这一份二进制里的判定用不到它的代码，
    /// 按「不编进来」这个状态进合判法键，开没开那个特性编的两份二进制合判法键不同。
    NotCompiledInThisBuild,
}

/// 合判法键的一份输入：组名（登记表里的写法，只当索引与排序）与它的键。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JudgingUnitsGroupForTheCombinedKey {
    pub group_name: String,
    pub key: JudgingUnitsGroupKeyAtTheBuild,
}

/// 合判法键（主 agent 2026-10-26 定）：几组判法单元各自的键按组名排好（同名再按键排），逐组写进摘要的输入，取 SHA-256。
/// 输入给的次序不影响结果；任何一组的键、键的状态或组名变了，结果就变。逐组的写法：组名字节数（u64 小端）、组名、状态字节
/// （键算得出 1、不编进来 0）、键算得出时再接 32 字节的键。组名带着长度写，两组之间的边界不会挪。
#[must_use]
pub fn combined_judging_key_of_the_groups(
    groups: &[JudgingUnitsGroupForTheCombinedKey],
) -> TodaysJudgingKey {
    let mut sorted: Vec<&JudgingUnitsGroupForTheCombinedKey> = groups.iter().collect();
    sorted.sort();
    let mut message = Vec::new();
    for group in sorted {
        let name_length = u64::try_from(group.group_name.len()).expect("组名的字节数装得进 u64");
        message.extend_from_slice(&name_length.to_le_bytes());
        message.extend_from_slice(group.group_name.as_bytes());
        match &group.key {
            JudgingUnitsGroupKeyAtTheBuild::Keyed(key) => {
                message.push(1);
                message.extend_from_slice(key);
            }
            JudgingUnitsGroupKeyAtTheBuild::NotCompiledInThisBuild => message.push(0),
        }
    }
    TodaysJudgingKey(sha256_digest(&message))
}

/// 合判法键算不出：登记的一组在编进二进制的判器键表里没有，或编译时那一组的闭包里有认不出的形态。小程序 `today-judging-key` 退 1，
/// 不拿别的几组凑一把（那样这一组的代码变了，行照旧读成罩住）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombinedJudgingKeyRefused {
    GroupMissingFromTheCompiledTable {
        group_name: String,
    },
    GroupRefusedAtTheBuild {
        group_name: String,
        refusal_lines: Vec<String>,
    },
}

impl CombinedJudgingKeyRefused {
    #[must_use]
    pub fn sentence(&self) -> String {
        match self {
            Self::GroupMissingFromTheCompiledTable { group_name } => format!(
                "判法单元入口登记表里的组 {group_name} 在编进二进制的判器键表里没有（构建脚本与登记表对不上），合判法键算不出"
            ),
            Self::GroupRefusedAtTheBuild {
                group_name,
                refusal_lines,
            } => format!(
                "组 {group_name} 编译时拒算（闭包里有认不出的形态：{}），合判法键算不出",
                refusal_lines.join(" | ").replace('\t', " ")
            ),
        }
    }
}

/// 判法单元入口登记表（`crate::judging_units_entries`）登记的全部组名：判器、核对、录入、拼块、对拍比法那几组照登记的次序，
/// 再接每一个观察者（`observer_code:<观察者代码文件>:<观察者名>`），与构建脚本逐组算键时的组名同一种拼法。
#[must_use]
pub fn registered_judging_units_group_names() -> Vec<String> {
    JUDGING_UNITS_ENTRY_GROUPS
        .iter()
        .map(|group| group.group_name.to_string())
        .chain(OBSERVER_ENTRIES.iter().map(|observer| {
            observer_group_name(observer.observer_code_file, observer.observer_name)
        }))
        .collect()
}

/// 照登记的组名逐组从判器键表里取合判法键的输入。不经 `crate::crash_identity::judging_units_key`：它把「表里没有这一组」与
/// 「这一趟不编进来」都交成同一种拒算，这里要把后一种当一个状态并进合判法键、前一种拒算。
///
/// # Errors
/// 登记的一组在表里没有、或编译时拒算。
pub fn groups_for_the_combined_key_of_the_table(
    registered_group_names: &[String],
    table: &ReadBackEntryGroupTable,
) -> Result<Vec<JudgingUnitsGroupForTheCombinedKey>, CombinedJudgingKeyRefused> {
    registered_group_names
        .iter()
        .map(|group_name| {
            let group = table
                .groups
                .iter()
                .find(|group| group.group_name == *group_name)
                .ok_or_else(
                    || CombinedJudgingKeyRefused::GroupMissingFromTheCompiledTable {
                        group_name: group_name.clone(),
                    },
                )?;
            let key = match &group.verdict {
                ReadBackEntryGroupVerdict::Keyed { key } => {
                    JudgingUnitsGroupKeyAtTheBuild::Keyed(*key)
                }
                ReadBackEntryGroupVerdict::NotCompiled { .. } => {
                    JudgingUnitsGroupKeyAtTheBuild::NotCompiledInThisBuild
                }
                ReadBackEntryGroupVerdict::Refused { refusal_lines } => {
                    return Err(CombinedJudgingKeyRefused::GroupRefusedAtTheBuild {
                        group_name: group_name.clone(),
                        refusal_lines: refusal_lines.clone(),
                    })
                }
            };
            Ok(JudgingUnitsGroupForTheCombinedKey {
                group_name: group_name.clone(),
                key,
            })
        })
        .collect()
}

/// 今天的判法键：登记表里全部组在这一张判器键表（小程序传编进二进制的那一张）里的合判法键，连同并进去的组数。
///
/// # Errors
/// 同 [`groups_for_the_combined_key_of_the_table`]。
pub fn todays_judging_key_of_the_compiled_table(
    table: &ReadBackEntryGroupTable,
) -> Result<(TodaysJudgingKey, Vec<JudgingUnitsGroupForTheCombinedKey>), CombinedJudgingKeyRefused>
{
    let groups =
        groups_for_the_combined_key_of_the_table(&registered_judging_units_group_names(), table)?;
    Ok((combined_judging_key_of_the_groups(&groups), groups))
}

impl TodaysJudgingKey {
    /// 64 位小写十六进制的逆；长短不对、不是小写十六进制交 `None`。
    #[must_use]
    pub fn of_the_hexadecimal_text(text: &str) -> Option<Self> {
        let bytes = bytes_of_the_lowercase_hexadecimal_text(text)?;
        <[u8; DIGEST_BYTES]>::try_from(bytes.as_slice())
            .ok()
            .map(Self)
    }
}

/// 「进度」那一列。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressOfThePart {
    NotRun,
    /// 判段中途停下：判了几块、一共几块（判了的块下一趟由判定库复用，表里只记块数）。
    Partial {
        judged_blocks: u64,
        total_blocks: u64,
    },
    AllGreen,
    /// 判红：发现日志里的签名。
    Red {
        finding_signature: String,
    },
}

impl ProgressOfThePart {
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::NotRun => NOT_RUN_TEXT.to_string(),
            Self::Partial {
                judged_blocks,
                total_blocks,
            } => format!(
                "{PARTIAL_PREFIX}{judged_blocks}{PARTIAL_MIDDLE}{total_blocks}{PARTIAL_SUFFIX}"
            ),
            Self::AllGreen => ALL_GREEN_TEXT.to_string(),
            Self::Red { finding_signature } => format!("{RED_PREFIX}{finding_signature}"),
        }
    }

    /// 表里的写法的逆（小程序写回时也照它读命令行给的进度）。
    ///
    /// # Errors
    /// 认不出的写法；「部分」里判了的块不少于总块数（那是全绿或红，不是部分）。
    pub fn of_the_text(text: &str) -> Result<Self, String> {
        if text == NOT_RUN_TEXT {
            return Ok(Self::NotRun);
        }
        if text == ALL_GREEN_TEXT {
            return Ok(Self::AllGreen);
        }
        if let Some(finding_signature) = text.strip_prefix(RED_PREFIX) {
            if finding_signature.is_empty() {
                return Err(format!("进度「{text}」没写签名"));
            }
            return Ok(Self::Red {
                finding_signature: finding_signature.to_string(),
            });
        }
        let blocks = text
            .strip_prefix(PARTIAL_PREFIX)
            .and_then(|rest| rest.strip_suffix(PARTIAL_SUFFIX))
            .and_then(|rest| rest.split_once(PARTIAL_MIDDLE))
            .and_then(|(judged_text, total_text)| {
                Some((
                    decimal_number_of_the_text(judged_text)?,
                    decimal_number_of_the_text(total_text)?,
                ))
            });
        let Some((judged_blocks, total_blocks)) = blocks else {
            return Err(format!(
                "进度「{text}」认不出：只认「{NOT_RUN_TEXT}」「{PARTIAL_PREFIX}<块数>{PARTIAL_MIDDLE}<块数>{PARTIAL_SUFFIX}」「{ALL_GREEN_TEXT}」「{RED_PREFIX}<签名>」"
            ));
        };
        if judged_blocks >= total_blocks {
            return Err(format!(
                "进度「{text}」判了的块不少于总块数：那是「{ALL_GREEN_TEXT}」或「{RED_PREFIX}<签名>」"
            ));
        }
        Ok(Self::Partial {
            judged_blocks,
            total_blocks,
        })
    }
}

/// 「最后一次」那一列的日期（`YYYY-MM-DD`，只写日期）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunDate(String);

impl RunDate {
    /// 不是 `四位-两位-两位` 的数字交 `None`。
    #[must_use]
    pub fn of_the_text(text: &str) -> Option<Self> {
        let digit_groups: Vec<&str> = text.split('-').collect();
        let is_a_date = matches!(digit_groups.as_slice(), [year, month, day]
            if year.len() == 4 && month.len() == 2 && day.len() == 2
                && [year, month, day].iter().all(|group| group.bytes().all(|byte| byte.is_ascii_digit())));
        is_a_date.then(|| Self(text.to_string()))
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.0
    }
}

/// 「最后一次」那一列：没跑过，或日期与那一趟的记录。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LastRunOfThePart {
    Never,
    On { date: RunDate, record: String },
}

impl LastRunOfThePart {
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::Never => NEVER_RUN_TEXT.to_string(),
            Self::On { date, record } => format!("{}{CELL_PART_SEPARATOR}{record}", date.text()),
        }
    }

    /// 表里的写法的逆。
    ///
    /// # Errors
    /// 不是「无」也不是「<日期>；<记录>」。
    pub fn of_the_text(text: &str) -> Result<Self, String> {
        if text == NEVER_RUN_TEXT {
            return Ok(Self::Never);
        }
        let Some((date_text, record)) = text.split_once(CELL_PART_SEPARATOR) else {
            return Err(format!(
                "最后一次「{text}」不是「{NEVER_RUN_TEXT}」或「<日期>{CELL_PART_SEPARATOR}<记录>」"
            ));
        };
        let date = RunDate::of_the_text(date_text)
            .ok_or_else(|| format!("最后一次的日期「{date_text}」不是 YYYY-MM-DD"))?;
        if record.is_empty() {
            return Err(format!("最后一次「{text}」没写记录"));
        }
        Ok(Self::On {
            date,
            record: record.to_string(),
        })
    }
}

/// 表里的一行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BacklogRow {
    pub sequence: RowSequence,
    pub part: CrashFullRunPart,
    pub registered_in: RegisteredInMilestone,
    pub states: StatesOfThePart,
    pub judging_key: JudgingKeyOfThePart,
    pub progress: ProgressOfThePart,
    pub last_run: LastRunOfThePart,
}

impl BacklogRow {
    /// 表里这一行的写法（九列，制表符隔开，不带换行）。
    #[must_use]
    pub fn line(&self) -> String {
        [
            self.sequence.0.to_string(),
            self.part.item_name.clone(),
            self.part.part.text().to_string(),
            self.part.key.clone(),
            self.registered_in.text(),
            self.states.text(),
            self.judging_key.text(),
            self.progress.text(),
            self.last_run.text(),
        ]
        .join("\t")
    }
}

/// 整张表：开头的注释行（原样留着、原样写回）、照展开次序登记的行数、各行。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrashFullRunBacklog {
    pub comment_lines: Vec<String>,
    pub rows_in_the_expansion_order: usize,
    pub rows: Vec<BacklogRow>,
}

/// 表读不成：第几行（从 1 起）、为什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BacklogTableUnreadable {
    pub line_number: usize,
    pub reason: String,
}

impl BacklogTableUnreadable {
    #[must_use]
    pub fn sentence(&self) -> String {
        format!(
            "积压登记表第 {} 行读不成：{}",
            self.line_number, self.reason
        )
    }
}

/// 读表：开头连着的 `#` 行是注释，接着一行 [`ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX`]、一行 [`BACKLOG_TABLE_COLUMN_HEADER`]，之后一行一件、九列。
///
/// # Errors
/// 缺那两行、列头不对、某一行不是九列或某一格认不出（交那一行的行号与原因）。
pub fn backlog_of_the_table_text(
    text: &str,
) -> Result<CrashFullRunBacklog, BacklogTableUnreadable> {
    let mut numbered_lines = text
        .lines()
        .enumerate()
        .map(|(line_index, line)| (line_index + 1, line))
        .peekable();
    let mut comment_lines = Vec::new();
    while let Some((_line_number, line)) =
        numbered_lines.next_if(|(_line_number, line)| line.starts_with('#'))
    {
        comment_lines.push(line.to_string());
    }
    let Some((count_line_number, count_line)) = numbered_lines.next() else {
        return Err(BacklogTableUnreadable {
            line_number: comment_lines.len() + 1,
            reason: format!(
                "注释之后没有「{ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX}<行数>」那一行"
            ),
        });
    };
    let rows_in_the_expansion_order = count_line
        .strip_prefix(ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX)
        .and_then(decimal_number_of_the_text)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(|| BacklogTableUnreadable {
            line_number: count_line_number,
            reason: format!(
                "「{count_line}」不是「{ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX}<十进制行数>」"
            ),
        })?;
    let Some((header_line_number, header_line)) = numbered_lines.next() else {
        return Err(BacklogTableUnreadable {
            line_number: count_line_number + 1,
            reason: "没有列头那一行".to_string(),
        });
    };
    if header_line != BACKLOG_TABLE_COLUMN_HEADER {
        return Err(BacklogTableUnreadable {
            line_number: header_line_number,
            reason: format!("列头「{header_line}」不是「{BACKLOG_TABLE_COLUMN_HEADER}」"),
        });
    }
    let mut rows = Vec::new();
    for (line_number, line) in numbered_lines {
        let cells: Vec<&str> = line.split('\t').collect();
        let row = backlog_row_of_the_cells(&cells).map_err(|reason| BacklogTableUnreadable {
            line_number,
            reason,
        })?;
        rows.push(row);
    }
    Ok(CrashFullRunBacklog {
        comment_lines,
        rows_in_the_expansion_order,
        rows,
    })
}

fn backlog_row_of_the_cells(cells: &[&str]) -> Result<BacklogRow, String> {
    let &[sequence_text, item_name, part_text, key, registered_text, states_text, judging_key_text, progress_text, last_run_text] =
        cells
    else {
        return Err(format!(
            "这一行是 {} 列，要九列（{BACKLOG_TABLE_COLUMN_HEADER}）",
            cells.len()
        ));
    };
    let sequence = decimal_number_of_the_text(sequence_text)
        .map(RowSequence)
        .ok_or_else(|| format!("序号「{sequence_text}」不是十进制整数"))?;
    let part = PartOfAnItem::of_the_text(part_text).ok_or_else(|| {
        format!(
            "部分「{part_text}」不是 {} 或 {}",
            PartOfAnItem::CrashPointReplay.text(),
            PartOfAnItem::CrashAmplification.text()
        )
    })?;
    if item_name.is_empty() || item_name.contains(' ') {
        return Err(format!("项名「{item_name}」是空的或带空格"));
    }
    if key.is_empty() || key.contains(' ') {
        return Err(format!("键「{key}」是空的或带空格"));
    }
    let key_is_a_crash_case = key.starts_with(CRASH_CASE_KEY_PREFIX);
    let key_fits_the_part = match part {
        PartOfAnItem::CrashPointReplay => key_is_a_crash_case,
        PartOfAnItem::CrashAmplification => !key_is_a_crash_case,
    };
    if !key_fits_the_part {
        return Err(format!(
            "键「{key}」与部分 {part_text} 对不上：崩溃点重放那一部分的键以 {CRASH_CASE_KEY_PREFIX} 起头，放量分项的不"
        ));
    }
    let registered_in = RegisteredInMilestone::of_the_text(registered_text).ok_or_else(|| {
        format!(
            "归属「{registered_text}」不是 {MILESTONE_THREE_AND_EARLIER_TEXT} 或 m<不小于 4 的号>"
        )
    })?;
    Ok(BacklogRow {
        sequence,
        part: CrashFullRunPart {
            item_name: item_name.to_string(),
            part,
            key: key.to_string(),
        },
        registered_in,
        states: StatesOfThePart::of_the_text(states_text)?,
        judging_key: JudgingKeyOfThePart::of_the_text(judging_key_text)?,
        progress: ProgressOfThePart::of_the_text(progress_text)?,
        last_run: LastRunOfThePart::of_the_text(last_run_text)?,
    })
}

/// [`backlog_of_the_table_text`] 的逆（每行带换行）。
#[must_use]
pub fn backlog_table_text(backlog: &CrashFullRunBacklog) -> String {
    let mut text = String::new();
    for comment_line in &backlog.comment_lines {
        text.push_str(comment_line);
        text.push('\n');
    }
    text.push_str(ROWS_IN_THE_EXPANSION_ORDER_LINE_PREFIX);
    text.push_str(&backlog.rows_in_the_expansion_order.to_string());
    text.push('\n');
    text.push_str(BACKLOG_TABLE_COLUMN_HEADER);
    text.push('\n');
    for row in &backlog.rows {
        text.push_str(&row.line());
        text.push('\n');
    }
    text
}

/// 合表读不成：第几行（从 1 起）、为什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MergedItemsUnreadable {
    pub line_number: usize,
    pub reason: String,
}

impl MergedItemsUnreadable {
    #[must_use]
    pub fn sentence(&self) -> String {
        format!(
            "崩溃点重放合表第 {} 行展不开：{}",
            self.line_number, self.reason
        )
    }
}

/// `crash-amplification.sh --list-items` 的输出（一行一个分项，制表符隔开的头一段是分项名）里的分项名，照 ITEMS 的次序。
#[must_use]
pub fn amplification_item_names_of_the_list(list_items_output: &str) -> Vec<String> {
    list_items_output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| line.split('\t').next())
        .map(str::to_string)
        .collect()
}

/// 合表一格（第二或第三列）里点名的几个，`-` 是一个都没有。
fn names_of_the_merged_items_column(cell: &str) -> Vec<&str> {
    if cell == EMPTY_COLUMN_OF_THE_MERGED_ITEMS {
        return Vec::new();
    }
    cell.split(' ').filter(|name| !name.is_empty()).collect()
}

/// 照合表展开成件（模块文档「展开的次序」）：`amplification_item_names` 是 ITEMS 的分项名，照 ITEMS 的次序。
///
/// # Errors
/// 合表的一行不到四列；第二列有不以 `crash-case:` 起头的键；第三列的分项不在 ITEMS 里；同一件被两项点名。
pub fn parts_in_the_expansion_order(
    merged_items_table_text: &str,
    amplification_item_names: &[String],
) -> Result<Vec<CrashFullRunPart>, MergedItemsUnreadable> {
    let order_in_the_amplification_items: BTreeMap<&str, usize> = amplification_item_names
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect();
    let mut parts = Vec::new();
    let mut seen_parts: BTreeSet<(PartOfAnItem, String)> = BTreeSet::new();
    for (line_index, line) in merged_items_table_text.lines().enumerate() {
        let line_number = line_index + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        let &[item_name, crash_point_replay_column, amplification_column, _light_tier_column, ..] =
            cells.as_slice()
        else {
            return Err(MergedItemsUnreadable {
                line_number,
                reason: format!("这一行是 {} 列，合表一行至少四列", cells.len()),
            });
        };
        let mut parts_of_the_item: Vec<CrashFullRunPart> = Vec::new();
        for key in names_of_the_merged_items_column(crash_point_replay_column) {
            if !key.starts_with(CRASH_CASE_KEY_PREFIX) {
                return Err(MergedItemsUnreadable {
                    line_number,
                    reason: format!("第二列的「{key}」不以 {CRASH_CASE_KEY_PREFIX} 起头"),
                });
            }
            parts_of_the_item.push(CrashFullRunPart {
                item_name: item_name.to_string(),
                part: PartOfAnItem::CrashPointReplay,
                key: key.to_string(),
            });
        }
        let mut amplification: Vec<(usize, &str)> = Vec::new();
        for name in names_of_the_merged_items_column(amplification_column) {
            let Some(position) = order_in_the_amplification_items.get(name) else {
                return Err(MergedItemsUnreadable {
                    line_number,
                    reason: format!("第三列的分项「{name}」不在崩溃放量的 ITEMS 里"),
                });
            };
            amplification.push((*position, name));
        }
        amplification.sort_unstable();
        for (_position, name) in amplification {
            parts_of_the_item.push(CrashFullRunPart {
                item_name: item_name.to_string(),
                part: PartOfAnItem::CrashAmplification,
                key: name.to_string(),
            });
        }
        for part in parts_of_the_item {
            if !seen_parts.insert((part.part, part.key.clone())) {
                return Err(MergedItemsUnreadable {
                    line_number,
                    reason: format!("{} 已经被前面的一项点名过", part.text()),
                });
            }
            parts.push(part);
        }
    }
    Ok(parts)
}

/// 表核出来的一处不对。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BacklogProblem {
    /// 少登：合表与 ITEMS 展开出来有、表里没有。
    PartMissingFromTheTable { part: CrashFullRunPart },
    /// 多登：表里有、展开出来没有。
    PartNotInTheExpansion {
        sequence: RowSequence,
        part: CrashFullRunPart,
    },
    /// 同一件登了两行。
    PartRegisteredTwice {
        earlier_row: RowSequence,
        later_row: RowSequence,
        part: CrashFullRunPart,
    },
    /// 照展开次序登记的那几行里，这一行在展开里排在前面登过的一行之前。
    OutOfTheExpansionOrder {
        sequence: RowSequence,
        part: CrashFullRunPart,
        registered_above_it: RowSequence,
    },
    /// 序号不是从 1 起逐行加一（跳号或重复）：这一行是第几行、写的序号。
    SequenceNotConsecutive {
        row_position: usize,
        sequence: RowSequence,
    },
    /// 照展开次序登记的行数比表里的行多。
    RowsInTheExpansionOrderBeyondTheTable { claimed_rows: usize, rows: usize },
    /// 进度是全绿而态数没数（全绿必须说清罩住了多少态）。
    AllGreenWithoutCountedStates { sequence: RowSequence },
    /// 进度不是未跑而判法键没记（跑过的进度要带那一趟的判法键，不然对不上今天的键）。
    ProgressWithoutAJudgingKey { sequence: RowSequence },
    /// 与上一版比：上一版的件次序变了，这一件上一版排在 `previously_ahead_of` 之前，这一版挪到了它后面。
    PreviousPartMovedBehind {
        sequence: RowSequence,
        part: CrashFullRunPart,
        previously_ahead_of: CrashFullRunPart,
    },
    /// 与上一版比：上一版没有的这一件没追加在末尾，插在上一版的 `previous_part_below_it` 之前。
    NewPartInsertedAmongThePreviousParts {
        sequence: RowSequence,
        part: CrashFullRunPart,
        previous_part_below_it: CrashFullRunPart,
    },
}

impl BacklogProblem {
    #[must_use]
    pub fn sentence(&self) -> String {
        match self {
            Self::PartMissingFromTheTable { part } => {
                format!("少登：{} 在合表与 ITEMS 展开里有、表里没有", part.text())
            }
            Self::PartNotInTheExpansion { sequence, part } => {
                format!(
                    "多登：第 {} 行 {} 在合表与 ITEMS 展开里没有",
                    sequence.0,
                    part.text()
                )
            }
            Self::PartRegisteredTwice {
                earlier_row,
                later_row,
                part,
            } => format!(
                "{} 登了两行：第 {} 行与第 {} 行",
                part.text(),
                earlier_row.0,
                later_row.0
            ),
            Self::OutOfTheExpansionOrder {
                sequence,
                part,
                registered_above_it,
            } => format!(
                "次序：第 {} 行 {} 在展开里排在第 {} 行之前",
                sequence.0,
                part.text(),
                registered_above_it.0
            ),
            Self::SequenceNotConsecutive {
                row_position,
                sequence,
            } => format!(
                "序号：第 {row_position} 行写的是 {}（跳号或重复）",
                sequence.0
            ),
            Self::RowsInTheExpansionOrderBeyondTheTable { claimed_rows, rows } => {
                format!("照展开次序登记的行数写 {claimed_rows}，表里只有 {rows} 行")
            }
            Self::AllGreenWithoutCountedStates { sequence } => {
                format!("第 {} 行进度全绿而态数没数", sequence.0)
            }
            Self::ProgressWithoutAJudgingKey { sequence } => {
                format!("第 {} 行进度不是未跑而判法键没记", sequence.0)
            }
            Self::PreviousPartMovedBehind {
                sequence,
                part,
                previously_ahead_of,
            } => format!(
                "与上一版比：第 {} 行 {} 上一版排在 {} 之前，这一版挪到了它后面",
                sequence.0,
                part.text(),
                previously_ahead_of.text()
            ),
            Self::NewPartInsertedAmongThePreviousParts {
                sequence,
                part,
                previous_part_below_it,
            } => format!(
                "与上一版比：新件 {}（第 {} 行）没追加在末尾，插在上一版的 {} 之前",
                part.text(),
                sequence.0,
                previous_part_below_it.text()
            ),
        }
    }
}

/// 表对着合表与 ITEMS 展开出来的件核：少登、多登、重登、照展开次序登记的那几行的次序、序号、全绿要有态数、跑过要有判法键。
#[must_use]
pub fn problems_against_the_expansion(
    backlog: &CrashFullRunBacklog,
    expansion: &[CrashFullRunPart],
) -> Vec<BacklogProblem> {
    let position_in_the_expansion: BTreeMap<&CrashFullRunPart, usize> = expansion
        .iter()
        .enumerate()
        .map(|(position, part)| (part, position))
        .collect();
    let mut problems = Vec::new();
    let mut row_of_each_part: BTreeMap<&CrashFullRunPart, RowSequence> = BTreeMap::new();
    for (row_index, row) in backlog.rows.iter().enumerate() {
        let expected_sequence = u64::try_from(row_index + 1).expect("表的行数装得进 u64");
        if row.sequence.0 != expected_sequence {
            problems.push(BacklogProblem::SequenceNotConsecutive {
                row_position: row_index + 1,
                sequence: row.sequence,
            });
        }
        match row_of_each_part.entry(&row.part) {
            Entry::Occupied(earlier) => problems.push(BacklogProblem::PartRegisteredTwice {
                earlier_row: *earlier.get(),
                later_row: row.sequence,
                part: row.part.clone(),
            }),
            Entry::Vacant(vacant) => {
                vacant.insert(row.sequence);
            }
        }
        if !position_in_the_expansion.contains_key(&row.part) {
            problems.push(BacklogProblem::PartNotInTheExpansion {
                sequence: row.sequence,
                part: row.part.clone(),
            });
        }
        if row.progress == ProgressOfThePart::AllGreen && row.states.counted_states().is_none() {
            problems.push(BacklogProblem::AllGreenWithoutCountedStates {
                sequence: row.sequence,
            });
        }
        if row.progress != ProgressOfThePart::NotRun
            && row.judging_key == JudgingKeyOfThePart::NotRecorded
        {
            problems.push(BacklogProblem::ProgressWithoutAJudgingKey {
                sequence: row.sequence,
            });
        }
    }
    for part in expansion {
        if !row_of_each_part.contains_key(part) {
            problems.push(BacklogProblem::PartMissingFromTheTable { part: part.clone() });
        }
    }
    if backlog.rows_in_the_expansion_order > backlog.rows.len() {
        problems.push(BacklogProblem::RowsInTheExpansionOrderBeyondTheTable {
            claimed_rows: backlog.rows_in_the_expansion_order,
            rows: backlog.rows.len(),
        });
    }
    // 照展开次序登记的那几行：展开里的位置逐行变大（重登的那一行位置相同，上面已报）
    let mut furthest_registered: Option<(usize, RowSequence)> = None;
    for row in backlog
        .rows
        .iter()
        .take(backlog.rows_in_the_expansion_order)
    {
        let Some(position) = position_in_the_expansion.get(&row.part) else {
            continue;
        };
        if let Some((furthest_position, furthest_sequence)) = furthest_registered {
            if *position < furthest_position {
                problems.push(BacklogProblem::OutOfTheExpansionOrder {
                    sequence: row.sequence,
                    part: row.part.clone(),
                    registered_above_it: furthest_sequence,
                });
                continue;
            }
        }
        furthest_registered = Some((*position, row.sequence));
    }
    problems
}

/// 新一版对着上一版核（出口 10「新增跑不完的那一截进积压登记表末尾」）：上一版还在的件次序不变；上一版没有的件都在上一版的件后面。
/// 上一版有、这一版删了的（合表删了那一项）不报：删了的件由 [`problems_against_the_expansion`] 管多登少登。
#[must_use]
pub fn problems_against_the_previous_table(
    previous: &CrashFullRunBacklog,
    current: &CrashFullRunBacklog,
) -> Vec<BacklogProblem> {
    let position_in_the_previous_table: BTreeMap<&CrashFullRunPart, usize> = previous
        .rows
        .iter()
        .enumerate()
        .map(|(position, row)| (&row.part, position))
        .collect();
    let mut problems = Vec::new();
    let mut new_rows_not_yet_followed: Vec<&BacklogRow> = Vec::new();
    let mut furthest_previous: Option<(usize, &CrashFullRunPart)> = None;
    for row in &current.rows {
        let Some(previous_position) = position_in_the_previous_table.get(&row.part) else {
            new_rows_not_yet_followed.push(row);
            continue;
        };
        for new_row in new_rows_not_yet_followed.drain(..) {
            problems.push(BacklogProblem::NewPartInsertedAmongThePreviousParts {
                sequence: new_row.sequence,
                part: new_row.part.clone(),
                previous_part_below_it: row.part.clone(),
            });
        }
        if let Some((furthest_previous_position, furthest_previous_part)) = furthest_previous {
            if *previous_position < furthest_previous_position {
                problems.push(BacklogProblem::PreviousPartMovedBehind {
                    sequence: row.sequence,
                    part: row.part.clone(),
                    previously_ahead_of: furthest_previous_part.clone(),
                });
                continue;
            }
        }
        furthest_previous = Some((*previous_position, &row.part));
    }
    problems
}

/// 一行在今天的判法键下的进度：判法键是今天这一把，照记下的读；对不上（或没记）当未跑读。
#[must_use]
pub fn progress_under_todays_judging_key(
    row: &BacklogRow,
    today: &TodaysJudgingKey,
) -> ProgressOfThePart {
    if row.judging_key.is_todays(today) {
        row.progress.clone()
    } else {
        ProgressOfThePart::NotRun
    }
}

/// 游标（出口 10「按登记表的次序接着上一次停下的地方」）：在今天的判法键下，第一行进度不是全绿的那一行；全都绿交 `None`。
#[must_use]
pub fn cursor_under_todays_judging_key(
    backlog: &CrashFullRunBacklog,
    today: &TodaysJudgingKey,
) -> Option<RowSequence> {
    backlog
        .rows
        .iter()
        .find(|row| progress_under_todays_judging_key(row, today) != ProgressOfThePart::AllGreen)
        .map(|row| row.sequence)
}

/// 取件的两条下限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateLowerBounds {
    /// 积压那一组至少取到这么多态。
    pub backlog_states_at_least: u64,
    /// 新增与积压两组合起来至少取到这么多态。
    pub states_at_least: u64,
}

impl StateLowerBounds {
    /// 出口 10 定的两条：积压至少 50 亿、合计至少 100 亿。
    pub const OF_THE_MILESTONE_EXIT: Self = Self {
        backlog_states_at_least: BACKLOG_STATES_AT_LEAST_PER_MILESTONE,
        states_at_least: STATES_AT_LEAST_PER_MILESTONE,
    };

    /// 两条都够了：积压不少于积压的下限，两组合起来不少于合计的下限。
    #[must_use]
    pub fn are_reached_by(
        &self,
        new_or_judging_key_changed_states: u64,
        backlog_states: u64,
    ) -> bool {
        backlog_states >= self.backlog_states_at_least
            && new_or_judging_key_changed_states.saturating_add(backlog_states)
                >= self.states_at_least
    }
}

/// 取到的件够不够两条下限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowerBoundsReached {
    BothBounds,
    /// 在今天的判法键下没罩住的行全取了，还不到下限（出口 10「合起来不足 100 亿时全部跑完」）。
    EveryUncoveredRowTakenBelowTheBounds,
}

/// 这一趟要跑的件（序号，照表的次序）：先跑新增与判法键变了的那一组，再跑从游标起的积压。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BacklogSelection {
    pub cursor: Option<RowSequence>,
    pub new_or_judging_key_changed: Vec<RowSequence>,
    pub backlog_from_the_cursor: Vec<RowSequence>,
    /// 两组各取到的态数（没数的行算 0，逐行列在 `taken_without_a_count`）。
    pub counted_states_of_the_new_or_judging_key_changed: u64,
    pub counted_states_of_the_backlog: u64,
    pub taken_without_a_count: Vec<RowSequence>,
    pub lower_bounds: LowerBoundsReached,
}

/// 取件（出口 10 ①②）：在今天的判法键下没罩住（进度不是全绿）的行里，先取这一里程碑登记的与判法键变了的（跑过、而记下的判法键不是
/// 今天这一把），照表的次序全取；再从游标起照表的次序取积压，取到积压不少于 `bounds.backlog_states_at_least`、两组合起来不少于
/// `bounds.states_at_least` 为止，不到就全取。一行的态数整行算（复用的块算罩住）；没数的行照取、算 0，逐行交回，好让调用方先数。
#[must_use]
pub fn parts_to_run_under_todays_judging_key(
    backlog: &CrashFullRunBacklog,
    today: &TodaysJudgingKey,
    current_milestone: SeparatelyRegisteredMilestone,
    bounds: StateLowerBounds,
) -> BacklogSelection {
    let is_new_or_judging_key_changed = |row: &BacklogRow| {
        let is_registered_in_this_milestone =
            row.registered_in == RegisteredInMilestone::Milestone(current_milestone);
        let was_run_under_another_judging_key =
            row.progress != ProgressOfThePart::NotRun && !row.judging_key.is_todays(today);
        is_registered_in_this_milestone || was_run_under_another_judging_key
    };
    let uncovered_rows: Vec<&BacklogRow> = backlog
        .rows
        .iter()
        .filter(|row| progress_under_todays_judging_key(row, today) != ProgressOfThePart::AllGreen)
        .collect();
    let mut selection = BacklogSelection {
        cursor: cursor_under_todays_judging_key(backlog, today),
        new_or_judging_key_changed: Vec::new(),
        backlog_from_the_cursor: Vec::new(),
        counted_states_of_the_new_or_judging_key_changed: 0,
        counted_states_of_the_backlog: 0,
        taken_without_a_count: Vec::new(),
        lower_bounds: LowerBoundsReached::EveryUncoveredRowTakenBelowTheBounds,
    };
    for row in uncovered_rows
        .iter()
        .filter(|row| is_new_or_judging_key_changed(row))
    {
        selection.new_or_judging_key_changed.push(row.sequence);
        match row.states.counted_states() {
            Some(states) => {
                selection.counted_states_of_the_new_or_judging_key_changed = selection
                    .counted_states_of_the_new_or_judging_key_changed
                    .checked_add(states)
                    .expect("全域约 3.8×10¹⁰ 态，几张表加起来装得进 u64");
            }
            None => selection.taken_without_a_count.push(row.sequence),
        }
    }
    for row in uncovered_rows
        .iter()
        .filter(|row| !is_new_or_judging_key_changed(row))
    {
        if bounds.are_reached_by(
            selection.counted_states_of_the_new_or_judging_key_changed,
            selection.counted_states_of_the_backlog,
        ) {
            break;
        }
        selection.backlog_from_the_cursor.push(row.sequence);
        match row.states.counted_states() {
            Some(states) => {
                selection.counted_states_of_the_backlog = selection
                    .counted_states_of_the_backlog
                    .checked_add(states)
                    .expect("全域约 3.8×10¹⁰ 态，几张表加起来装得进 u64");
            }
            None => selection.taken_without_a_count.push(row.sequence),
        }
    }
    selection.lower_bounds = if bounds.are_reached_by(
        selection.counted_states_of_the_new_or_judging_key_changed,
        selection.counted_states_of_the_backlog,
    ) {
        LowerBoundsReached::BothBounds
    } else {
        LowerBoundsReached::EveryUncoveredRowTakenBelowTheBounds
    };
    selection
}

/// 跑完一件写回的四列（项名、部分、键、归属不改）；态数给 `None` 照旧。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowAfterARun {
    pub states: Option<StatesOfThePart>,
    pub judging_key: JudgingKeyOfThePart,
    pub progress: ProgressOfThePart,
    pub last_run: LastRunOfThePart,
}

/// 把跑完一件的结果写进表里那一行（改的是传进来的表）。
///
/// # Errors
/// 表里没有这个序号的行。
pub fn write_the_run_into_the_row(
    backlog: &mut CrashFullRunBacklog,
    sequence: RowSequence,
    run: RowAfterARun,
) -> Result<(), String> {
    let row = backlog
        .rows
        .iter_mut()
        .find(|row| row.sequence == sequence)
        .ok_or_else(|| format!("积压登记表里没有序号 {} 的行", sequence.0))?;
    if let Some(states) = run.states {
        row.states = states;
    }
    row.judging_key = run.judging_key;
    row.progress = run.progress;
    row.last_run = run.last_run;
    Ok(())
}

/// 合表与 ITEMS 展开里有、表里还没有的件追加到末尾（出口 10「新增……进积压登记表末尾」）：照展开的次序，序号接着往下编，
/// 归属写 `registered_in`，态数「未数」（出处写 `provenance`）、判法键「未记」、进度「未跑」、最后一次「无」。交回追加了几件。
pub fn append_the_parts_missing_from_the_table(
    backlog: &mut CrashFullRunBacklog,
    expansion: &[CrashFullRunPart],
    registered_in: RegisteredInMilestone,
    provenance: &str,
) -> usize {
    let registered_parts: BTreeSet<CrashFullRunPart> =
        backlog.rows.iter().map(|row| row.part.clone()).collect();
    let mut appended = 0usize;
    for part in expansion {
        if registered_parts.contains(part) {
            continue;
        }
        let sequence = u64::try_from(backlog.rows.len() + 1).expect("表的行数装得进 u64");
        backlog.rows.push(BacklogRow {
            sequence: RowSequence(sequence),
            part: part.clone(),
            registered_in,
            states: StatesOfThePart::NotCounted {
                provenance: provenance.to_string(),
            },
            judging_key: JudgingKeyOfThePart::NotRecorded,
            progress: ProgressOfThePart::NotRun,
            last_run: LastRunOfThePart::Never,
        });
        appended += 1;
    }
    appended
}

/// 十进制整数，要能原样写回（不认前导 0、正号、空白）。
fn decimal_number_of_the_text(text: &str) -> Option<u64> {
    let number = text.parse::<u64>().ok()?;
    (number.to_string() == text).then_some(number)
}

/// 小写十六进制（两个字符一个字节，与 `singlefs_harness::hexadecimal::hexadecimal_text` 互逆）；奇数个字符、有别的字符交 `None`。
fn bytes_of_the_lowercase_hexadecimal_text(text: &str) -> Option<Vec<u8>> {
    if text.is_empty()
        || !text.len().is_multiple_of(2)
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&text[offset..offset + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODAY_BYTE: u8 = 0x5a;
    const ANOTHER_DAY_BYTE: u8 = 0x3c;

    fn todays_key() -> TodaysJudgingKey {
        TodaysJudgingKey([TODAY_BYTE; DIGEST_BYTES])
    }

    fn milestone_four() -> SeparatelyRegisteredMilestone {
        SeparatelyRegisteredMilestone::new(4).expect("4 单记")
    }

    /// 两项的小合表：前一项两条 `crash-case:`、两个放量分项（第三列写成与 ITEMS 相反的次序），后一项只有一条 `crash-case:`。
    const MERGED_ITEMS_TEXT: &str = "# 小合表（单测造的）\n\
        fixed-flow\tcrash-case:fixed-flow-stream crash-case:fixed-flow-other-stream\tfixed-flow-later fixed-flow-earlier\tcrash_enumeration_fixed_flow\t# 两条用例、两个分项\n\
        new-flow\tcrash-case:new-flow-stream\t-\tcrash_enumeration_new_flow\t# 只有崩溃点重放那一部分\n";

    fn amplification_item_names() -> Vec<String> {
        amplification_item_names_of_the_list(
            "fixed-flow-earlier\tcrash_enumeration_fixed_flow\tearlier_is_clean\t全量（不在快档）\n\
             fixed-flow-later\tcrash_enumeration_fixed_flow\tlater_is_clean\t全量（不在快档）\n",
        )
    }

    fn expansion() -> Vec<CrashFullRunPart> {
        parts_in_the_expansion_order(MERGED_ITEMS_TEXT, &amplification_item_names())
            .expect("小合表展得开")
    }

    fn row_of(
        sequence: u64,
        part: &CrashFullRunPart,
        registered_in: RegisteredInMilestone,
        counted_states: Option<u64>,
        judging_key: JudgingKeyOfThePart,
        progress: ProgressOfThePart,
    ) -> BacklogRow {
        BacklogRow {
            sequence: RowSequence(sequence),
            part: part.clone(),
            registered_in,
            states: match counted_states {
                Some(states) => StatesOfThePart::Counted {
                    states,
                    provenance: "单测造的".to_string(),
                },
                None => StatesOfThePart::NotCounted {
                    provenance: "单测造的".to_string(),
                },
            },
            judging_key,
            progress,
            last_run: LastRunOfThePart::Never,
        }
    }

    /// 照展开逐件一行、都没跑过、每件 10 态的表。
    fn table_of_the_expansion() -> CrashFullRunBacklog {
        let rows: Vec<BacklogRow> = expansion()
            .iter()
            .enumerate()
            .map(|(row_index, part)| {
                row_of(
                    u64::try_from(row_index + 1).expect("行数装得进 u64"),
                    part,
                    RegisteredInMilestone::MilestoneThreeAndEarlier,
                    Some(10),
                    JudgingKeyOfThePart::NotRecorded,
                    ProgressOfThePart::NotRun,
                )
            })
            .collect();
        CrashFullRunBacklog {
            comment_lines: vec!["# 单测造的表".to_string()],
            rows_in_the_expansion_order: rows.len(),
            rows,
        }
    }

    fn renumbered(mut backlog: CrashFullRunBacklog) -> CrashFullRunBacklog {
        for (row_index, row) in backlog.rows.iter_mut().enumerate() {
            row.sequence = RowSequence(u64::try_from(row_index + 1).expect("行数装得进 u64"));
        }
        backlog
    }

    /// 主 agent 2026-10-26 定的展开次序：每项先第二列的 `crash-case:`（照第二列的次序）、再第三列的放量分项（照 ITEMS 的次序，不照第三列写的次序）。
    /// 判别力（`crates/mutations.tsv`）：放量分项不照 ITEMS 排，这条红。
    #[test]
    fn the_expansion_lists_the_crash_cases_then_the_amplification_items_in_the_items_order() {
        let keys: Vec<(String, PartOfAnItem)> = expansion()
            .into_iter()
            .map(|part| (part.key, part.part))
            .collect();
        assert_eq!(
            keys,
            vec![
                (
                    "crash-case:fixed-flow-stream".to_string(),
                    PartOfAnItem::CrashPointReplay
                ),
                (
                    "crash-case:fixed-flow-other-stream".to_string(),
                    PartOfAnItem::CrashPointReplay
                ),
                (
                    "fixed-flow-earlier".to_string(),
                    PartOfAnItem::CrashAmplification
                ),
                (
                    "fixed-flow-later".to_string(),
                    PartOfAnItem::CrashAmplification
                ),
                (
                    "crash-case:new-flow-stream".to_string(),
                    PartOfAnItem::CrashPointReplay
                ),
            ],
            "先第二列、再第三列；第三列照 ITEMS 的次序"
        );
    }

    /// 照展开逐件一行的表没有问题；删掉一行就报少登，点名删掉的那一件。判别力（`crates/mutations.tsv`）：不核少登，这条红。
    #[test]
    fn a_part_of_the_expansion_missing_from_the_table_is_reported_as_missing() {
        let complete = table_of_the_expansion();
        assert_eq!(
            problems_against_the_expansion(&complete, &expansion()),
            Vec::new(),
            "逐件一行的表没有问题"
        );
        let mut missing_one = complete.clone();
        let removed = missing_one.rows.remove(2);
        missing_one.rows_in_the_expansion_order -= 1;
        let problems = problems_against_the_expansion(&renumbered(missing_one), &expansion());
        assert_eq!(
            problems,
            vec![BacklogProblem::PartMissingFromTheTable { part: removed.part }],
            "少登那一件被点名，别的不报"
        );
    }

    /// 表里多一件展开里没有的（追加在末尾）就报多登。判别力（`crates/mutations.tsv`）：不核多登，这条红。
    #[test]
    fn a_part_not_in_the_expansion_is_reported_as_registered_too_much() {
        let mut with_an_extra_part = table_of_the_expansion();
        let extra = CrashFullRunPart {
            item_name: "removed-flow".to_string(),
            part: PartOfAnItem::CrashPointReplay,
            key: "crash-case:removed-flow-stream".to_string(),
        };
        let sequence = u64::try_from(with_an_extra_part.rows.len() + 1).expect("行数装得进 u64");
        with_an_extra_part.rows.push(row_of(
            sequence,
            &extra,
            RegisteredInMilestone::MilestoneThreeAndEarlier,
            Some(10),
            JudgingKeyOfThePart::NotRecorded,
            ProgressOfThePart::NotRun,
        ));
        assert_eq!(
            problems_against_the_expansion(&with_an_extra_part, &expansion()),
            vec![BacklogProblem::PartNotInTheExpansion {
                sequence: RowSequence(sequence),
                part: extra
            }]
        );
    }

    /// 照展开次序登记的那几行里两行对调（序号照新次序重编）就报次序：点名排到后面去的那一行。判别力（`crates/mutations.tsv`）：不核次序，这条红。
    #[test]
    fn rows_registered_out_of_the_expansion_order_are_reported() {
        let mut swapped = table_of_the_expansion();
        swapped.rows.swap(1, 3);
        let swapped = renumbered(swapped);
        let problems = problems_against_the_expansion(&swapped, &expansion());
        assert_eq!(
            problems,
            vec![
                BacklogProblem::OutOfTheExpansionOrder {
                    sequence: RowSequence(3),
                    part: swapped.rows[2].part.clone(),
                    registered_above_it: RowSequence(2),
                },
                BacklogProblem::OutOfTheExpansionOrder {
                    sequence: RowSequence(4),
                    part: swapped.rows[3].part.clone(),
                    registered_above_it: RowSequence(2),
                },
            ],
            "第 2 行登的是展开里的第 4 件，它下面第 3、4 行（展开里的第 3、2 件）都排错了"
        );
        let mut swapped_among_the_appended_rows = table_of_the_expansion();
        swapped_among_the_appended_rows.rows_in_the_expansion_order = 1;
        swapped_among_the_appended_rows.rows.swap(1, 3);
        assert_eq!(
            problems_against_the_expansion(
                &renumbered(swapped_among_the_appended_rows),
                &expansion()
            ),
            Vec::new(),
            "照展开次序登记的只有开头一行时，之后的是追加的、不按展开排"
        );
    }

    /// 序号跳号与重复各报一处。判别力（`crates/mutations.tsv`）：不核序号，这条红。
    #[test]
    fn skipped_or_repeated_sequence_numbers_are_reported() {
        let mut skipped = table_of_the_expansion();
        skipped.rows[2].sequence = RowSequence(4);
        let problems_of_the_skipped = problems_against_the_expansion(&skipped, &expansion());
        assert_eq!(
            problems_of_the_skipped,
            vec![BacklogProblem::SequenceNotConsecutive {
                row_position: 3,
                sequence: RowSequence(4)
            }],
            "第 3 行写成 4：跳号"
        );
        let mut repeated = table_of_the_expansion();
        repeated.rows[2].sequence = RowSequence(2);
        let problems_of_the_repeated = problems_against_the_expansion(&repeated, &expansion());
        assert_eq!(
            problems_of_the_repeated,
            vec![BacklogProblem::SequenceNotConsecutive {
                row_position: 3,
                sequence: RowSequence(2)
            }],
            "第 3 行写成 2：重复"
        );
    }

    /// 进度全绿而态数没数报一处；数了就不报。判别力（`crates/mutations.tsv`）：不核全绿的态数，这条红。
    #[test]
    fn an_all_green_row_without_counted_states_is_reported() {
        let mut green_without_states = table_of_the_expansion();
        green_without_states.rows[0].states = StatesOfThePart::NotCounted {
            provenance: "单测造的".to_string(),
        };
        green_without_states.rows[0].progress = ProgressOfThePart::AllGreen;
        green_without_states.rows[0].judging_key = JudgingKeyOfThePart::Fingerprint(todays_key().0);
        assert_eq!(
            problems_against_the_expansion(&green_without_states, &expansion()),
            vec![BacklogProblem::AllGreenWithoutCountedStates {
                sequence: RowSequence(1)
            }]
        );
        let mut green_with_states = green_without_states;
        green_with_states.rows[0].states = StatesOfThePart::Counted {
            states: 10,
            provenance: "单测造的".to_string(),
        };
        assert_eq!(
            problems_against_the_expansion(&green_with_states, &expansion()),
            Vec::new()
        );
    }

    /// 进度不是未跑（跑过、判红、跑到一半）而判法键没记报一处：对不上今天的键，下一趟分不出它是在哪一把键下判的。未跑的不要判法键。
    /// 判别力（`crates/mutations.tsv`）：不核跑过的行有没有判法键，这条红。
    #[test]
    fn a_run_row_without_a_judging_key_is_reported() {
        let mut red_without_a_key = table_of_the_expansion();
        red_without_a_key.rows[1].progress = ProgressOfThePart::Red {
            finding_signature: "单测造的签名".to_string(),
        };
        assert_eq!(
            problems_against_the_expansion(&red_without_a_key, &expansion()),
            vec![BacklogProblem::ProgressWithoutAJudgingKey {
                sequence: RowSequence(2)
            }]
        );
        assert_eq!(
            problems_against_the_expansion(&table_of_the_expansion(), &expansion()),
            Vec::new(),
            "未跑的行判法键未记不报"
        );
    }

    /// 一行全绿，可判法键是别的一把、或是待重判（改取合判法键之前判的）：在今天的键下当未跑读，游标停在这一行；键是今天这一把才照全绿读、
    /// 游标越过它。判别力（`crates/mutations.tsv`）：判法键对不上也照记下的进度读，这条红。
    #[test]
    fn a_green_row_judged_under_another_judging_key_reads_as_not_run_and_holds_the_cursor() {
        let mut backlog = table_of_the_expansion();
        for row in &mut backlog.rows {
            row.progress = ProgressOfThePart::AllGreen;
            row.judging_key = JudgingKeyOfThePart::Fingerprint(todays_key().0);
        }
        backlog.rows[2].judging_key =
            JudgingKeyOfThePart::Fingerprint([ANOTHER_DAY_BYTE; DIGEST_BYTES]);
        backlog.rows[3].judging_key = JudgingKeyOfThePart::ToBeRejudgedUnderTheCombinedKey;
        let today = todays_key();
        assert_eq!(
            progress_under_todays_judging_key(&backlog.rows[2], &today),
            ProgressOfThePart::NotRun
        );
        assert_eq!(
            progress_under_todays_judging_key(&backlog.rows[3], &today),
            ProgressOfThePart::NotRun
        );
        assert_eq!(
            cursor_under_todays_judging_key(&backlog, &today),
            Some(RowSequence(3)),
            "游标停在键对不上的那一行"
        );
        backlog.rows[2].judging_key = JudgingKeyOfThePart::Fingerprint(today.0);
        backlog.rows[3].judging_key = JudgingKeyOfThePart::Fingerprint(today.0);
        assert_eq!(
            progress_under_todays_judging_key(&backlog.rows[3], &today),
            ProgressOfThePart::AllGreen,
            "判法键是今天这一把照全绿读"
        );
        assert_eq!(
            cursor_under_todays_judging_key(&backlog, &today),
            None,
            "全都在今天的键下全绿"
        );
    }

    /// 积压够了下限而合计还不够时接着取：五件积压各 10 态，积压下限 15、合计下限 35（新增那一组 0 态），要取到第四件（积压 40）才停，
    /// 第五件不取。判别力（`crates/mutations.tsv`）：只看积压的下限就停，这条红。
    #[test]
    fn the_backlog_is_taken_until_both_lower_bounds_are_reached() {
        let backlog = table_of_the_expansion();
        let selection = parts_to_run_under_todays_judging_key(
            &backlog,
            &todays_key(),
            milestone_four(),
            StateLowerBounds {
                backlog_states_at_least: 15,
                states_at_least: 35,
            },
        );
        assert_eq!(selection.cursor, Some(RowSequence(1)));
        assert_eq!(selection.new_or_judging_key_changed, Vec::new());
        assert_eq!(
            selection.backlog_from_the_cursor,
            vec![
                RowSequence(1),
                RowSequence(2),
                RowSequence(3),
                RowSequence(4)
            ],
            "积压到 20 就够了积压的下限，合计要到 40 才够 35"
        );
        assert_eq!(selection.counted_states_of_the_backlog, 40);
        assert_eq!(selection.lower_bounds, LowerBoundsReached::BothBounds);
    }

    /// 新增那一组自己就够了合计的下限，积压照样要取到积压的下限：新增一件 100 态，四件积压各 10 态，积压下限 15、合计下限 35，
    /// 积压取两件（20 态）就停。判别力（`crates/mutations.tsv`）：只看合计的下限，这条红。
    #[test]
    fn the_backlog_is_taken_up_to_its_own_lower_bound_even_when_the_new_rows_reach_the_total() {
        let mut backlog = table_of_the_expansion();
        backlog.rows[0].registered_in = RegisteredInMilestone::Milestone(milestone_four());
        backlog.rows[0].states = StatesOfThePart::Counted {
            states: 100,
            provenance: "单测造的".to_string(),
        };
        let selection = parts_to_run_under_todays_judging_key(
            &backlog,
            &todays_key(),
            milestone_four(),
            StateLowerBounds {
                backlog_states_at_least: 15,
                states_at_least: 35,
            },
        );
        assert_eq!(selection.new_or_judging_key_changed, vec![RowSequence(1)]);
        assert_eq!(
            selection.backlog_from_the_cursor,
            vec![RowSequence(2), RowSequence(3)],
            "积压取到 20 态够了 15 才停"
        );
        assert_eq!(selection.lower_bounds, LowerBoundsReached::BothBounds);
    }

    /// 这一里程碑登记的与判法键变了的先取、全取，积压从游标起取、越过它们；合计不到下限时没罩住的行全取。全绿（今天的键）的那一行不取。
    /// 判别力（`crates/mutations.tsv`）：不认这一里程碑登记的，这条红。
    #[test]
    fn new_and_key_changed_rows_are_taken_ahead_and_every_uncovered_row_is_taken_below_the_bounds()
    {
        let mut backlog = table_of_the_expansion();
        backlog.rows[0].progress = ProgressOfThePart::AllGreen;
        backlog.rows[0].judging_key = JudgingKeyOfThePart::Fingerprint(todays_key().0);
        backlog.rows[1].progress = ProgressOfThePart::AllGreen;
        backlog.rows[1].judging_key =
            JudgingKeyOfThePart::Fingerprint([ANOTHER_DAY_BYTE; DIGEST_BYTES]);
        backlog.rows[3].registered_in = RegisteredInMilestone::Milestone(milestone_four());
        backlog.rows[4].states = StatesOfThePart::NotCounted {
            provenance: "单测造的".to_string(),
        };
        let selection = parts_to_run_under_todays_judging_key(
            &backlog,
            &todays_key(),
            milestone_four(),
            StateLowerBounds::OF_THE_MILESTONE_EXIT,
        );
        assert_eq!(
            selection.cursor,
            Some(RowSequence(2)),
            "第 1 行今天的键下全绿，第 2 行键变了"
        );
        assert_eq!(
            selection.new_or_judging_key_changed,
            vec![RowSequence(2), RowSequence(4)],
            "键变了的第 2 行、这一里程碑登记的第 4 行先取"
        );
        assert_eq!(
            selection.backlog_from_the_cursor,
            vec![RowSequence(3), RowSequence(5)]
        );
        assert_eq!(
            selection.counted_states_of_the_new_or_judging_key_changed,
            20
        );
        assert_eq!(selection.counted_states_of_the_backlog, 10);
        assert_eq!(selection.taken_without_a_count, vec![RowSequence(5)]);
        assert_eq!(
            selection.lower_bounds,
            LowerBoundsReached::EveryUncoveredRowTakenBelowTheBounds,
            "40 态到不了 50 亿：全取"
        );
    }

    /// 新一版在上一版的件中间插一件（序号重编、照展开次序登记的行数也加一）就报插在中间；追加在末尾不报。
    /// 判别力（`crates/mutations.tsv`）：不报插在中间的新件，这条红。
    #[test]
    fn a_new_part_inserted_among_the_previous_parts_is_reported() {
        let previous = table_of_the_expansion();
        let new_part = CrashFullRunPart {
            item_name: "later-flow".to_string(),
            part: PartOfAnItem::CrashPointReplay,
            key: "crash-case:later-flow-stream".to_string(),
        };
        let new_row = row_of(
            0,
            &new_part,
            RegisteredInMilestone::Milestone(milestone_four()),
            None,
            JudgingKeyOfThePart::NotRecorded,
            ProgressOfThePart::NotRun,
        );
        let mut inserted = previous.clone();
        inserted.rows.insert(2, new_row.clone());
        inserted.rows_in_the_expansion_order += 1;
        let inserted = renumbered(inserted);
        assert_eq!(
            problems_against_the_previous_table(&previous, &inserted),
            vec![BacklogProblem::NewPartInsertedAmongThePreviousParts {
                sequence: RowSequence(3),
                part: new_part,
                previous_part_below_it: previous.rows[2].part.clone(),
            }]
        );
        let mut appended = previous.clone();
        appended.rows.push(new_row);
        assert_eq!(
            problems_against_the_previous_table(&previous, &renumbered(appended)),
            Vec::new(),
            "追加在末尾不报"
        );
    }

    /// 表的写法读回来逐格相同（各种进度、判法键的全长与待重判、最后一次、注释行）；跑完写回一行、追加缺的件之后照样读得回、核得过。
    /// 判别力（`crates/mutations.tsv`）：「部分」那一格把两个块数写反，这条红。
    #[test]
    fn the_table_text_reads_back_to_the_same_backlog_after_a_run_is_written_and_missing_parts_are_appended(
    ) {
        let mut backlog = table_of_the_expansion();
        backlog.rows[0].progress = ProgressOfThePart::Partial {
            judged_blocks: 3,
            total_blocks: 7,
        };
        backlog.rows[0].judging_key = JudgingKeyOfThePart::ToBeRejudgedUnderTheCombinedKey;
        backlog.rows[1].progress = ProgressOfThePart::Red {
            finding_signature: "I-1.2 违例".to_string(),
        };
        backlog.rows[1].judging_key = JudgingKeyOfThePart::Fingerprint(todays_key().0);
        backlog.rows[1].last_run = LastRunOfThePart::On {
            date: RunDate::of_the_text("2026-10-26").expect("是日期"),
            record: "records/单测造的.md".to_string(),
        };
        let text = backlog_table_text(&backlog);
        assert_eq!(
            backlog_of_the_table_text(&text),
            Ok(backlog.clone()),
            "读回来逐格相同"
        );
        write_the_run_into_the_row(
            &mut backlog,
            RowSequence(1),
            RowAfterARun {
                states: None,
                judging_key: JudgingKeyOfThePart::Fingerprint(todays_key().0),
                progress: ProgressOfThePart::AllGreen,
                last_run: LastRunOfThePart::Never,
            },
        )
        .expect("有第 1 行");
        assert_eq!(
            backlog.rows[0].states.counted_states(),
            Some(10),
            "态数给 None 照旧"
        );
        assert!(write_the_run_into_the_row(
            &mut backlog,
            RowSequence(99),
            RowAfterARun {
                states: None,
                judging_key: JudgingKeyOfThePart::NotRecorded,
                progress: ProgressOfThePart::NotRun,
                last_run: LastRunOfThePart::Never,
            },
        )
        .is_err());
        let removed = backlog.rows.pop().expect("表不空");
        let appended = append_the_parts_missing_from_the_table(
            &mut backlog,
            &expansion(),
            RegisteredInMilestone::Milestone(milestone_four()),
            "追加时还没数",
        );
        assert_eq!(appended, 1);
        assert_eq!(
            backlog.rows.last().map(|row| &row.part),
            Some(&removed.part)
        );
        assert_eq!(
            problems_against_the_expansion(&backlog, &expansion()),
            Vec::new()
        );
        assert_eq!(
            backlog_of_the_table_text(&backlog_table_text(&backlog)),
            Ok(backlog)
        );
    }

    /// 进本包的那一份表读得成、照展开次序登记的就是全部行、自己成形（序号、全绿有态数、跑过有判法键；多登少登要合表，归门禁那一格）。
    /// 判别力（`crates/mutations.tsv`）：把表里一行全绿的态数改成未数，这条红。
    #[test]
    fn the_registered_table_reads_and_is_well_formed_on_its_own() {
        let backlog = backlog_of_the_table_text(include_str!("../crash-full-run-backlog.tsv"))
            .unwrap_or_else(|unreadable| panic!("{}", unreadable.sentence()));
        assert_eq!(
            backlog.rows_in_the_expansion_order,
            backlog.rows.len(),
            "建表那一版没有追加的行"
        );
        let expansion_of_the_table: Vec<CrashFullRunPart> =
            backlog.rows.iter().map(|row| row.part.clone()).collect();
        let problems = problems_against_the_expansion(&backlog, &expansion_of_the_table);
        assert_eq!(
            problems,
            Vec::new(),
            "{}",
            problems
                .iter()
                .map(BacklogProblem::sentence)
                .collect::<Vec<_>>()
                .join("；")
        );
        assert!(
            backlog
                .rows
                .iter()
                .any(|row| row.progress == ProgressOfThePart::AllGreen),
            "2026-10-25 那一趟判绿的几行记成全绿"
        );
    }

    const IMAGE_JUDGEMENT_GROUP_NAME: &str = "judgement_of_the_crash_images";
    const GIVEN_STATES_GROUP_NAME: &str = "judgement_of_the_given_states";
    const OBSERVER_GROUP_NAME: &str =
        "observer_code:crates/singlefs-checker-tier/tests/单测造的观察者.rs:landed_version";
    const IMAGE_JUDGEMENT_KEY_BYTE: u8 = 0x11;
    const GIVEN_STATES_KEY_BYTE: u8 = 0x22;
    const OBSERVER_KEY_BYTE_BEFORE: u8 = 0x33;
    const OBSERVER_KEY_BYTE_AFTER: u8 = 0x44;

    /// 单测造的三组假单元键：镜像判定、给定状态、一个观察者，照这个次序。
    fn three_groups(
        given_states_key_byte: u8,
        observer_key_byte: u8,
    ) -> Vec<JudgingUnitsGroupForTheCombinedKey> {
        vec![
            JudgingUnitsGroupForTheCombinedKey {
                group_name: IMAGE_JUDGEMENT_GROUP_NAME.to_string(),
                key: JudgingUnitsGroupKeyAtTheBuild::Keyed(
                    [IMAGE_JUDGEMENT_KEY_BYTE; DIGEST_BYTES],
                ),
            },
            JudgingUnitsGroupForTheCombinedKey {
                group_name: GIVEN_STATES_GROUP_NAME.to_string(),
                key: JudgingUnitsGroupKeyAtTheBuild::Keyed([given_states_key_byte; DIGEST_BYTES]),
            },
            JudgingUnitsGroupForTheCombinedKey {
                group_name: OBSERVER_GROUP_NAME.to_string(),
                key: JudgingUnitsGroupKeyAtTheBuild::Keyed([observer_key_byte; DIGEST_BYTES]),
            },
        ]
    }

    /// 主 agent 2026-10-26 定的合指纹：只改观察者那一组的键，合判法键变，在旧键下全绿的一行在新键下当未跑读、归判法键变了的那一组先取；
    /// 只改给定状态那一组的键同样变。判别力（`crates/mutations.tsv`）：合判法键只取镜像判定那一组，这条红。
    #[test]
    fn a_change_in_the_observer_group_alone_changes_the_combined_key_and_the_row_reads_as_not_run()
    {
        let before = combined_judging_key_of_the_groups(&three_groups(
            GIVEN_STATES_KEY_BYTE,
            OBSERVER_KEY_BYTE_BEFORE,
        ));
        let after = combined_judging_key_of_the_groups(&three_groups(
            GIVEN_STATES_KEY_BYTE,
            OBSERVER_KEY_BYTE_AFTER,
        ));
        assert_ne!(before, after, "只改观察者那一组的键，合判法键要变");
        assert_ne!(
            before,
            combined_judging_key_of_the_groups(&three_groups(
                GIVEN_STATES_KEY_BYTE.wrapping_add(1),
                OBSERVER_KEY_BYTE_BEFORE,
            )),
            "只改给定状态那一组的键，合判法键要变"
        );
        let mut backlog = table_of_the_expansion();
        backlog.rows[0].progress = ProgressOfThePart::AllGreen;
        backlog.rows[0].judging_key = JudgingKeyOfThePart::Fingerprint(before.0);
        assert_eq!(
            progress_under_todays_judging_key(&backlog.rows[0], &before),
            ProgressOfThePart::AllGreen,
            "在判它的那一把合判法键下照全绿读"
        );
        assert_eq!(
            progress_under_todays_judging_key(&backlog.rows[0], &after),
            ProgressOfThePart::NotRun,
            "观察者那一组的键变了，当未跑读"
        );
        let selection = parts_to_run_under_todays_judging_key(
            &backlog,
            &after,
            milestone_four(),
            StateLowerBounds::OF_THE_MILESTONE_EXIT,
        );
        assert_eq!(selection.cursor, Some(RowSequence(1)));
        assert_eq!(
            selection.new_or_judging_key_changed,
            vec![RowSequence(1)],
            "判法键变了的那一行先取"
        );
    }

    /// 合判法键只看有哪些组、各是什么键，不看给的次序：倒过来给、轮转着给，合判法键都相同。
    /// 判别力（`crates/mutations.tsv`）：合之前不按组名排，这条红。
    #[test]
    fn the_combined_key_does_not_depend_on_the_order_the_groups_are_given_in() {
        let groups = three_groups(GIVEN_STATES_KEY_BYTE, OBSERVER_KEY_BYTE_BEFORE);
        let combined = combined_judging_key_of_the_groups(&groups);
        let mut reversed = groups.clone();
        reversed.reverse();
        assert_eq!(
            combined_judging_key_of_the_groups(&reversed),
            combined,
            "倒过来给"
        );
        let mut rotated = groups;
        rotated.rotate_left(1);
        assert_eq!(
            combined_judging_key_of_the_groups(&rotated),
            combined,
            "轮转着给"
        );
    }

    fn read_back_group(
        group_name: &str,
        verdict: ReadBackEntryGroupVerdict,
    ) -> singlefs_unit_graph::entry_groups::ReadBackEntryGroup {
        singlefs_unit_graph::entry_groups::ReadBackEntryGroup {
            group_name: group_name.to_string(),
            verdict,
            entries_not_compiled: Vec::new(),
            composition: Vec::new(),
        }
    }

    /// 单测造的判器键表：镜像判定、观察者两组算得出键，给定状态那一组由参数给。
    fn compiled_table_with_the_given_states_verdict(
        given_states_verdict: ReadBackEntryGroupVerdict,
    ) -> ReadBackEntryGroupTable {
        ReadBackEntryGroupTable {
            toolchain_lines: Vec::new(),
            groups: vec![
                read_back_group(
                    IMAGE_JUDGEMENT_GROUP_NAME,
                    ReadBackEntryGroupVerdict::Keyed {
                        key: [IMAGE_JUDGEMENT_KEY_BYTE; DIGEST_BYTES],
                    },
                ),
                read_back_group(GIVEN_STATES_GROUP_NAME, given_states_verdict),
                read_back_group(
                    OBSERVER_GROUP_NAME,
                    ReadBackEntryGroupVerdict::Keyed {
                        key: [OBSERVER_KEY_BYTE_BEFORE; DIGEST_BYTES],
                    },
                ),
            ],
            watched_files: Vec::new(),
            features_enabled_by_the_build: singlefs_unit_graph::FeaturesEnabledByTheBuild {
                features_of_each_package: BTreeMap::new(),
            },
        }
    }

    fn three_registered_group_names() -> Vec<String> {
        [
            IMAGE_JUDGEMENT_GROUP_NAME,
            GIVEN_STATES_GROUP_NAME,
            OBSERVER_GROUP_NAME,
        ]
        .iter()
        .map(|name| (*name).to_string())
        .collect()
    }

    /// 从判器键表取合判法键的输入：算得出键的照键进；这一趟不编进来的按「不编进来」这个状态进（合判法键与算得出键时不同）；
    /// 编译时拒算的、登记了而表里没有的，整把拒算，不拿别的几组凑。判别力（`crates/mutations.tsv`）：拒算的组当不编进来并进去，这条红。
    #[test]
    fn a_group_refused_at_the_build_or_missing_from_the_table_refuses_the_combined_key() {
        let names = three_registered_group_names();
        let keyed = groups_for_the_combined_key_of_the_table(
            &names,
            &compiled_table_with_the_given_states_verdict(ReadBackEntryGroupVerdict::Keyed {
                key: [GIVEN_STATES_KEY_BYTE; DIGEST_BYTES],
            }),
        )
        .expect("三组都算得出键");
        assert_eq!(
            keyed,
            three_groups(GIVEN_STATES_KEY_BYTE, OBSERVER_KEY_BYTE_BEFORE)
        );
        let not_compiled = groups_for_the_combined_key_of_the_table(
            &names,
            &compiled_table_with_the_given_states_verdict(ReadBackEntryGroupVerdict::NotCompiled {
                reason: "单测造的：这一趟没开那个特性".to_string(),
            }),
        )
        .expect("不编进来的组按状态进，不拒算");
        assert_eq!(
            not_compiled[1].key,
            JudgingUnitsGroupKeyAtTheBuild::NotCompiledInThisBuild
        );
        assert_ne!(
            combined_judging_key_of_the_groups(&not_compiled),
            combined_judging_key_of_the_groups(&keyed),
            "不编进来与算得出键的合判法键不同"
        );
        let refusal_lines = vec!["单测造的拒算行".to_string()];
        assert_eq!(
            groups_for_the_combined_key_of_the_table(
                &names,
                &compiled_table_with_the_given_states_verdict(ReadBackEntryGroupVerdict::Refused {
                    refusal_lines: refusal_lines.clone(),
                }),
            ),
            Err(CombinedJudgingKeyRefused::GroupRefusedAtTheBuild {
                group_name: GIVEN_STATES_GROUP_NAME.to_string(),
                refusal_lines,
            })
        );
        let mut names_with_one_more = names;
        names_with_one_more.push("单测造的没进表的组".to_string());
        assert_eq!(
            groups_for_the_combined_key_of_the_table(
                &names_with_one_more,
                &compiled_table_with_the_given_states_verdict(ReadBackEntryGroupVerdict::Keyed {
                    key: [GIVEN_STATES_KEY_BYTE; DIGEST_BYTES],
                }),
            ),
            Err(
                CombinedJudgingKeyRefused::GroupMissingFromTheCompiledTable {
                    group_name: "单测造的没进表的组".to_string(),
                }
            )
        );
    }

    /// 合判法键并进的组就是入口登记表登记的全部：判器、核对、录入、拼块、对拍比法那几组一个不少，连同每一个观察者。
    /// 判别力（`crates/mutations.tsv`）：登记的组名里漏了观察者，这条红。
    #[test]
    fn the_registered_group_names_are_every_entry_group_and_every_observer() {
        let names = registered_judging_units_group_names();
        assert_eq!(
            names.len(),
            JUDGING_UNITS_ENTRY_GROUPS.len() + OBSERVER_ENTRIES.len()
        );
        for group in JUDGING_UNITS_ENTRY_GROUPS {
            assert!(
                names.iter().any(|name| name == group.group_name),
                "少了组 {}",
                group.group_name
            );
        }
        for observer in OBSERVER_ENTRIES {
            let observer_name =
                observer_group_name(observer.observer_code_file, observer.observer_name);
            assert!(names.contains(&observer_name), "少了观察者 {observer_name}");
        }
        assert!(names.iter().any(|name| name == IMAGE_JUDGEMENT_GROUP_NAME));
        assert!(names.iter().any(|name| name == GIVEN_STATES_GROUP_NAME));
    }

    /// 待重判（改取合判法键之前判的）的全绿行：在任何一把合判法键下都当未跑读，归判法键变了的那一组先取。
    /// 判别力（`crates/mutations.tsv`）：待重判当成今天这一把，这条红。
    #[test]
    fn a_green_row_to_be_rejudged_under_the_combined_key_is_taken_with_the_key_changed_rows() {
        let mut backlog = table_of_the_expansion();
        for row in &mut backlog.rows {
            row.progress = ProgressOfThePart::AllGreen;
            row.judging_key = JudgingKeyOfThePart::Fingerprint(todays_key().0);
        }
        backlog.rows[1].judging_key = JudgingKeyOfThePart::ToBeRejudgedUnderTheCombinedKey;
        let selection = parts_to_run_under_todays_judging_key(
            &backlog,
            &todays_key(),
            milestone_four(),
            StateLowerBounds::OF_THE_MILESTONE_EXIT,
        );
        assert_eq!(selection.cursor, Some(RowSequence(2)));
        assert_eq!(selection.new_or_judging_key_changed, vec![RowSequence(2)]);
        assert_eq!(selection.backlog_from_the_cursor, Vec::new());
        assert_eq!(
            JudgingKeyOfThePart::of_the_text("待重判"),
            Ok(JudgingKeyOfThePart::ToBeRejudgedUnderTheCombinedKey)
        );
        assert!(
            JudgingKeyOfThePart::of_the_text("329cd43c35141f61").is_err(),
            "改取合判法键之前那种只记头 8 字节的写法不再认"
        );
    }
}
