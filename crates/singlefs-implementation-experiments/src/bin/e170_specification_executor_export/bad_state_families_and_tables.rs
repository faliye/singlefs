//! 第三段：坏状态（登记 5.5、5.8「第三段」）的常量、坏族、表的版本与行的类型。

use super::*;

// ───────────────────────────── 第三段：坏状态（登记 5.5、5.8「第三段」） ─────────────────────────────
//
// `bad-states --dump-directory <第一段的导出目录> --seed-hex <第二段冻结的种子>`：重驱动两条流（与 `dump` 同一份驱动），先核重驱动的流
// 与导出目录里的 base-sectors.bin、ops.tsv、ops-payload.bin、calls.tsv 逐字节相同（manifest 记的 sha256），再按登记 5.5 造 P 行、N 行与
// 盲测族：底态（F、Mk、Mk⁺、Mk⁻、盲测的 Mk 型调用边界）+ 字节补丁。补丁用 `crates/` 的格式知识造（根记录、journal 记录、系统配置槽的
// 编码与重封都调 core 的写者），补丁不算写（A(c) 按底态算，登记第一节读法 3）。每行同时过一遍实现的判定（`judge_crash_image`，补丁
// 接在写表末尾当持久的写，种类记成系统配置槽写：`publishes_of_one_recording` 不把它算进任何一次发布）。
// 每条流写进 `<导出目录>/<流名>/`：`bad-states.tsv` 与 `bad-patches.bin`（执行器那一侧读：行号、底态落地向量、两个摘要、补丁），
// `bad-expected.tsv`（期望，只给比对器读）、`bad-impl-verdicts.tsv`（实现判定）、`bad-seed.tsv`（种子）。已存在就拒绝，不覆盖。

/// 登记 5.5 写死的改法数值（本地常量，值抄自登记 5.5 的 P 表、N 表与盲测族一段）。
pub(super) const BAD_ROOT_RING_BASE_SLOT_PATCHED: u64 = 65;
pub(super) const BAD_ROOT_RING_BASE_SLOT_FIRST_VERSION: u64 = 64;
pub(super) const BAD_PSEUDO_ROOT_TXG_STEP: u64 = 3;
pub(super) const BAD_WATERMARK_STEP: u64 = 1000;
pub(super) const BAD_LOWERED_TXG_STEP: u64 = 2;
pub(super) const BAD_ROOT_TREE_IDENTIFIER_WATERMARK_FIRST_BYTE: usize = 122;
pub(super) const BAD_UNIT_FLIP_OFFSET: usize = 200;
pub(super) const BAD_FLIPPED_BIT: u8 = 0x01;
pub(super) const BLIND_ROWS_PER_STREAM: usize = 20;
/// 第 1 次重跑多出来的那一行（P10）与它要对照底态的那一行（P9）的行名；S5b 按这两个名字找行。
pub(super) const LOWER_BOUND_ROW_NAME: &str = "P10-Mk+";
pub(super) const FIRST_RUN_MK_PLUS_ROW_NAME: &str = "P9-Mk+";
pub(super) const BLIND_PSEUDO_ROOT_STEPS: u64 = 7;
pub(super) const BLIND_LOWEST_WATERMARK_STEP: u64 = 1000;
pub(super) const BLIND_HIGHEST_WATERMARK_STEP: u64 = 1_000_000;
/// 记录的点名项区起点 = 记录头字节数（D23（journal 的角色与格式） 已定项 4：记录头 496 字节）；新根段起点 = 本次发布内序号 4 + 反向链 4 + 载荷校验和 4
/// 之后。两个数写成本地常量，`bad-states` 开头与 core 的公开常量回比（S1 同形，回比不过照打、退 3）。
pub(super) const BAD_RECORD_NAMED_ITEM_AREA_START: usize = 496;
pub(super) const BAD_RECORD_NEW_ROOT_SEGMENT_START: usize = 99;
/// 系统配置槽里「根环起点」那 8 字节的偏移（`layout/01-first-txn.md` 一）：core 里是私有常量，补丁之后由 `SystemConfiguration::parse_slot`
/// 读回 65 来核这个偏移（`bad_s1` 行 `p1_root_ring_base_reads_back`）。
pub(super) const BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET: usize = 371;

/// 坏状态表里一行属于哪一族（登记 5.5）：P 行与盲测族期望「不在」、决定 Q3；N 行期望「在」、附带。
/// 第 1 次重跑多一族 P10（`research/prompts/e170-r1-prereg.md` 5.5）：考 A(c) 下界的一行，附带 Q3-g、不进 Q3-a，所以不与 P 行共用标签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BadFamily {
    Positive,
    Negative,
    Blind,
    AcknowledgementLowerBound,
}

impl BadFamily {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Positive => "P",
            Self::Negative => "N",
            Self::Blind => "B",
            Self::AcknowledgementLowerBound => "P10",
        }
    }
}

/// 坏状态表的哪一版：第一次跑的表（`bad-states`，P8 期望「不在」、没有 P10），或第 1 次重跑的表（`bad-states-second-run`，P8 期望「在」、
/// 每条流末尾多一行 P10；`research/prompts/e170-r1-prereg.md` 5.3、5.5）。两版的补丁、底态、盲测族抽样逐行相同（第十一节 S5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BadTableVersion {
    FirstRun,
    SecondRun,
}

impl BadTableVersion {
    pub(super) fn mode_word(self) -> &'static str {
        match self {
            Self::FirstRun => "bad-states",
            Self::SecondRun => "bad-states-second-run",
        }
    }

    /// `config` 结果行里 `mode=` 写的名字（第一次的模式照旧写 `bad_states`）。
    pub(super) fn configuration_mode_text(self) -> &'static str {
        match self {
            Self::FirstRun => "bad_states",
            Self::SecondRun => "bad_states_second_run",
        }
    }

    /// P8 的期望判定。第一次的登记写「不在」；第 1 次重跑的登记 5.5 修订按读法 3 改成「在」（落点 v_{n−1} 仍在 A(c) 里）。
    pub(super) fn previous_version_landing_expected_verdict(self) -> &'static str {
        match self {
            Self::FirstRun => "not_in",
            Self::SecondRun => "in",
        }
    }
}

/// 一个底态：名字（F、Mk、Mk+、Mk-、Mk@<调用序号>）、它对应的调用（F 没有）、按写序号的持久向量（只有持久与没持久两值）。
#[derive(Clone, Debug)]
pub(super) struct BaseState {
    pub(super) label: String,
    pub(super) call: Option<usize>,
    pub(super) persisted: Vec<bool>,
    pub(super) notes: String,
}

/// 一处字节补丁：在底态镜像上把 (盘, 偏移) 起的这几个字节换掉。
#[derive(Clone, Debug)]
pub(super) struct BytePatch {
    pub(super) device: DeviceIdentity,
    pub(super) offset: u64,
    pub(super) bytes: Vec<u8>,
}

/// 期望落到哪里（登记 5.5「期望与依据」一列；Q3-d、Q3-e 的比对用）。
#[derive(Clone, Debug)]
pub(super) enum ExpectedLanding {
    Nothing,
    Version(String),
    OutsideTheVersionTable,
}

impl ExpectedLanding {
    pub(super) fn text(&self) -> String {
        match self {
            Self::Nothing => "none".to_string(),
            Self::Version(projection) => projection.clone(),
            Self::OutsideTheVersionTable => "outside_version_table".to_string(),
        }
    }
}

/// 坏状态表的一行。
#[derive(Clone, Debug)]
pub(super) struct BadRow {
    pub(super) name: String,
    pub(super) family: BadFamily,
    pub(super) base: BaseState,
    pub(super) patches: Vec<BytePatch>,
    pub(super) expected_verdict: &'static str,
    pub(super) expected_landing: ExpectedLanding,
}

/// splitmix64（盲测族的抽样，登记 5.5「当 u64 喂 splitmix64」）：状态先加黄金比常数，再两轮异或移位乘。
pub(super) struct SplitMix64 {
    pub(super) state: u64,
}

impl SplitMix64 {
    fn next_value(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// [0, choices) 里均匀取一个：取模，偏差至多 choices / 2⁶⁴（choices 不过一百万，可以不计）。
    pub(super) fn below(&mut self, choices: u64) -> u64 {
        assert!(choices > 0, "至少一个可选");
        self.next_value() % choices
    }
}

/// 一条流在坏状态这一段要的几样：每次写的流序下标、按发布分好的写、发过根槽写的调用（调用序号升序）。
pub(super) struct BadStreamFacts<'stream> {
    pub(super) stream: &'stream PreparedStream,
    pub(super) operation_index_of_write: Vec<usize>,
    pub(super) publishes: Vec<PublishWrites>,
    pub(super) root_issuing_calls: Vec<usize>,
}

#[cfg(test)]
mod tests;
