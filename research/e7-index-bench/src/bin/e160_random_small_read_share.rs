//! E160：随机小读在负载里占多少 —— 三个来源（甲：权重、乙：公开参考负载、丙：E152 作业混合）
//! 与两条重开判据（D4（校验和位置） 已定项 1／8 的判据 1、已定项 5 的交叉点）的合成。
//!
//! 只做第一段（纯算术，不碰盘）：判据、臂、失败与作废条款写在
//! `research/prompts/e160-preregistration.md`，装置写之前。第二段（计时）不在这个文件里。
//!
//! ## 口径
//!
//! - R 类：来源判为随机、且单次请求小于 32768 字节的读；Q 类：其余所有读。
//! - p_B（用户字节口径，对已定项 5）= R 类用户字节 ÷ 全部读的用户字节；
//!   p_N（I/O 次数口径）= R 类请求数 ÷ 全部读请求数，只报数，不直接进判定。
//! - 判据 1 只取读法 A（登记第一节第 6 条，主 agent 2026-09-24 定）：逐次倍数，
//!   来源的 R 类有多种尺寸时按请求数取「均值之比」。
//! - 判据 5 的带 [1.0%, 1.5%]（两端都含，登记第一节第 7 条）；判据 1 的阈值 1.136（登记 A1）。
//!
//! ## 停机条款核对（跑产物之前）
//!
//! - S1：本文件的页到单元算术只在净荷 32634 上核对（`net_payload_bytes` 常量与 `spans_two_units_given_payload`），
//!   与 `crates/singlefs-core/src/mounted_read.rs` 的
//!   `about_one_in_eight_four_kibibyte_pages_spans_two_data_units` 单测 2026-09-24 各自跑过、结论一致
//!   （一个周期 16317 页、2047 页跨单元）；见报告，这个文件不依赖 `crates`（`.claude/rules/implementation-first.md` 第 4 条）。
//! - S3：`HEADER_REGISTERED_BYTES`（105）、`NONCE_MAC_RESERVED_BYTES`（29）、`UNIT_BYTES_32`（32768）
//!   2026-09-24 与 `crates/singlefs-format/src/lib.rs` 的
//!   `DATA_UNIT_HEADER_BYTES`、`NONCE_MAC_ALGORITHM_RESERVED_BYTES`、`DATA_UNIT_BYTES` 现值比对过，相同。
//! - S2 管第二段的 L32h 计时臂，这一轮不适用（第二段不跑）。

use e7_index_bench::Emitter;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

// ============================== 有理数 ==============================
// 判据阈值附近的比较用精确算术，不用浮点（登记第一节第 7 条）。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RationalNumber {
    numerator: i128,
    denominator: i128,
}

fn greatest_common_divisor(first: i128, second: i128) -> i128 {
    let (mut larger, mut smaller) = (first.abs(), second.abs());
    while smaller != 0 {
        let remainder = larger % smaller;
        larger = smaller;
        smaller = remainder;
    }
    larger
}

impl RationalNumber {
    fn new(numerator: i128, denominator: i128) -> Self {
        assert!(denominator != 0, "分母不能是 0");
        let sign: i128 = if denominator < 0 { -1 } else { 1 };
        let numerator = numerator * sign;
        let denominator = denominator * sign;
        let divisor = greatest_common_divisor(numerator, denominator).max(1);
        Self { numerator: numerator / divisor, denominator: denominator / divisor }
    }
    fn zero() -> Self {
        Self { numerator: 0, denominator: 1 }
    }
    fn one() -> Self {
        Self { numerator: 1, denominator: 1 }
    }
    fn from_integer(value: i128) -> Self {
        Self { numerator: value, denominator: 1 }
    }
    fn to_approximate_f64(self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
    #[allow(dead_code, reason = "只在单测 rational_number_arithmetic_is_reduced_and_ordered 与 M8 的替换文里用到，主流程用不到取最大值")]
    fn maximum(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }
    fn as_percent_text(self) -> String {
        format!("{:.6}%", self.to_approximate_f64() * 100.0)
    }
    fn as_plain_text(self) -> String {
        format!("{:.6}", self.to_approximate_f64())
    }
}

impl Add for RationalNumber {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        RationalNumber::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}
impl Sub for RationalNumber {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        RationalNumber::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}
impl Mul for RationalNumber {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        RationalNumber::new(self.numerator * other.numerator, self.denominator * other.denominator)
    }
}
impl Div for RationalNumber {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        assert!(other.numerator != 0, "除以 0");
        RationalNumber::new(self.numerator * other.denominator, self.denominator * other.numerator)
    }
}
impl PartialOrd for RationalNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.numerator * other.denominator).cmp(&(other.numerator * self.denominator)))
    }
}
impl Ord for RationalNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("RationalNumber 全序，比较恒有结果")
    }
}

fn option_percent_text(value: Option<RationalNumber>) -> String {
    value.map(RationalNumber::as_percent_text).unwrap_or_else(|| "读不到".to_string())
}
fn option_plain_text(value: Option<RationalNumber>) -> String {
    value.map(RationalNumber::as_plain_text).unwrap_or_else(|| "读不到".to_string())
}
fn option_bool_text(value: Option<bool>) -> String {
    match value {
        Some(true) => "重开".to_string(),
        Some(false) => "不重开".to_string(),
        None => "不可判".to_string(),
    }
}

// ============================== 常量 ==============================

/// D4（校验和位置） 已定项 1：随机小读也是 4 KiB。
const PAGE_BYTES: u64 = 4096;
/// D8（核心索引结构） 已定项 2：索引节点单元；D4 已定项 8 的判据 1 用它当「16 KiB 单元」的对照。
const UNIT_BYTES_16: u64 = 16384;
/// D4（校验和位置） 已定项 5：数据单元恒 32768 字节，含头。
const UNIT_BYTES_32: u64 = 32768;
const UNIT_PAGES_16: u64 = UNIT_BYTES_16 / PAGE_BYTES;
const UNIT_PAGES_32: u64 = UNIT_BYTES_32 / PAGE_BYTES;
/// D18（块里携带什么信息） 已定项 7；与 `crates/singlefs-format/src/lib.rs` 的
/// `DATA_UNIT_HEADER_BYTES` 现值一致（S3，2026-09-24 现查）。
const HEADER_REGISTERED_BYTES: u64 = 105;
/// D18（块里携带什么信息） 已定项 14；与 `crates/singlefs-format/src/lib.rs` 的
/// `NONCE_MAC_ALGORITHM_RESERVED_BYTES` 现值一致（S3，2026-09-24 现查）。
const NONCE_MAC_RESERVED_BYTES: u64 = 29;
/// 含预留位之后的码 1 头（登记第五节 5.3：「头 134、净荷 32634」）。
const HEADER_WITH_RESERVE_BYTES: u64 = HEADER_REGISTERED_BYTES + NONCE_MAC_RESERVED_BYTES;
/// D4（校验和位置） 已定项 5 的含头净荷：32768 − 105 − 29。
const NET_PAYLOAD_BYTES: u64 = UNIT_BYTES_32 - HEADER_WITH_RESERVE_BYTES;
/// 补齐形态：净荷补到 7 个整页，头与补齐占一个页（D4（校验和位置） 已定项 5 射程；E140（单元含头逼出的跨单元页与读侧拷贝） 的替代形态）。
const PADDED_PAYLOAD_BYTES: u64 = 7 * PAGE_BYTES;

/// E58（校验和粒度的端到端代价） 实测的随机小读设备侧倍数（读法 A 的锚点）。
const ANCHOR_MULTIPLIER_NUMERATOR: i128 = 1086;
const ANCHOR_MULTIPLIER_DENOMINATOR: i128 = 1000;
fn anchor_multiplier() -> RationalNumber {
    RationalNumber::new(ANCHOR_MULTIPLIER_NUMERATOR, ANCHOR_MULTIPLIER_DENOMINATOR)
}
/// D4（校验和位置） 已定项 8 定案：「随机小读设备侧倍数 ≥ 1.136（E58 实测 1.086 再高 5 个百分点）」。
fn judgement_one_threshold() -> RationalNumber {
    RationalNumber::new(ANCHOR_MULTIPLIER_NUMERATOR, ANCHOR_MULTIPLIER_DENOMINATOR) + RationalNumber::new(5, 100)
}
/// D4（校验和位置） 已定项 5 的交叉点带下沿；C357 那一行与岔路单第 8 行。
fn band_lower_bound() -> RationalNumber {
    RationalNumber::new(1, 100)
}
/// 带上沿。
fn band_upper_bound() -> RationalNumber {
    RationalNumber::new(3, 200)
}
/// 「相加」规则的阈值主值（登记第一节第 9 条：「本登记在被测条款里找到的随机读代价容忍度只有判据 1 的 13.6%」）。
fn additive_threshold_primary() -> RationalNumber {
    RationalNumber::new(17, 125)
}
fn additive_threshold_alternate_low() -> RationalNumber {
    RationalNumber::new(1, 20)
}
fn additive_threshold_alternate_zero() -> RationalNumber {
    RationalNumber::zero()
}
/// 两类模型：顺序请求（默认 1 MiB）相对随机请求（4 KiB）的字节比。
const DEFAULT_SEQUENTIAL_TO_RANDOM_SIZE_RATIO: i128 = 256;
/// 只报数、不进判定的敏感性变体（顺序请求改成 128 KiB；登记第八节 G3：已删，只报数不进判定）。
#[allow(dead_code, reason = "只在单测 two_class_conversion_matches_the_cited_values 里报数，登记第八节 G3 明说不进判定")]
const ALTERNATE_SEQUENTIAL_REQUEST_SIZE_RATIO_128_KIBIBYTES: i128 = 32;

/// 丙臂：随机作业 `--time_based --runtime=20`（`e152_file_system_benchmark.rs:651-663`）。
const RANDOM_JOB_RUNTIME_SECONDS: i128 = 20;
/// 丙臂：顺序与随机两个作业都 `--size=4G`（同上 `:648-663`）。
const DECLARED_SEQUENTIAL_FILE_BYTES: i128 = 4 * 1024 * 1024 * 1024;
/// 丙臂：顺序作业 `--bs=1M`。
const DECLARED_SEQUENTIAL_REQUEST_BYTES: i128 = 1024 * 1024;
/// M1 的错误来源：`write_sequential_large` 同样声明 `size=4G`（不该算进读的分母）。
#[allow(dead_code, reason = "只被 research/mutations/e160_random_small_read_share.tsv 的 M1 替换文引用，基线的正确公式不用它")]
const WRITE_SEQUENTIAL_JOB_BYTES: i128 = 4 * 1024 * 1024 * 1024;
/// M10 的错误来源：两个 `control_buffered_read_*` 对照各 256 MiB（E152 实验页第 274 行；不该算进读的分母）。
#[allow(dead_code, reason = "只被 research/mutations/e160_random_small_read_share.tsv 的 M10 替换文引用，基线的正确公式不用它")]
const CONTROL_BUFFERED_READ_BYTES: i128 = 2 * 256 * 1024 * 1024;

// ============================== 判据一侧：分档 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThresholdSide {
    BelowLowerBound,
    WithinBand,
    AboveUpperBound,
}
impl ThresholdSide {
    fn as_text(self) -> &'static str {
        match self {
            ThresholdSide::BelowLowerBound => "未越过",
            ThresholdSide::WithinBand => "带内",
            ThresholdSide::AboveUpperBound => "越过",
        }
    }
}

/// 登记第一节第 7 条：< 1.0% 未越过，[1.0%, 1.5%] 带内（两端都含），> 1.5% 越过。
fn classify_against_band(value: RationalNumber, lower_bound: RationalNumber, upper_bound: RationalNumber) -> ThresholdSide {
    if value > upper_bound {
        ThresholdSide::AboveUpperBound
    } else if value < lower_bound {
        ThresholdSide::BelowLowerBound
    } else {
        ThresholdSide::WithinBand
    }
}

/// G1：把下沿也算「越过」（带内也算越过）。
fn classify_against_band_treating_lower_bound_as_crossed(value: RationalNumber, lower_bound: RationalNumber) -> bool {
    value >= lower_bound
}

// ============================== 判据本身 ==============================

/// 判据 1：`>=` 是条款字面。
fn judgement_one_triggered(value: RationalNumber) -> bool {
    value >= judgement_one_threshold() // M4 目标：>= 改成 >
}
/// 判据 5「越过」才算重开，「带内」不算。
fn judgement_five_reopens(side: ThresholdSide) -> bool {
    side == ThresholdSide::AboveUpperBound // M5 目标：带内按越过进规则
}
fn first_to_trigger_reopens(judgement_one: bool, judgement_five_side: ThresholdSide) -> bool {
    judgement_one || judgement_five_reopens(judgement_five_side) // M9 目标：先到先触发写成"两条都越过"
}
fn only_judgement_one_reopens(judgement_one: bool) -> bool {
    judgement_one
}
fn only_judgement_five_reopens(judgement_five_side: ThresholdSide) -> bool {
    judgement_five_reopens(judgement_five_side)
}
fn additive_reopens(load_level_cost_c1: RationalNumber, header_cost_c5: RationalNumber, threshold_sum: RationalNumber) -> bool {
    load_level_cost_c1 + header_cost_c5 >= threshold_sum // M8 目标：相加用 max 代替 +
}

/// 三种情形都可能缺信息（乙的部分来源），短路：判据 5 已确定越过就不必等判据 1。
fn first_to_trigger_reopens_optional(judgement_one: Option<bool>, judgement_five_side: Option<ThresholdSide>) -> Option<bool> {
    if judgement_five_side.map(judgement_five_reopens) == Some(true) {
        return Some(true);
    }
    if judgement_one == Some(true) {
        return Some(true);
    }
    if judgement_one == Some(false) && judgement_five_side.map(judgement_five_reopens) == Some(false) {
        return Some(false);
    }
    None
}

/// 登记第四节丙类「带下沿以下 Σ 有上界」：p_B < 1.0% 时 C5 < 0（交叉点定义），
/// C1 ≤ m_r 锚点 − 1 = 0.086（m_q = 1，D4 已定项 1 依据「顺序侧……设备侧为零」）。
fn additive_load_level_cost_upper_bound() -> RationalNumber {
    anchor_multiplier() - RationalNumber::one()
}
/// 只有上界 < 阈值时才能担保「不重开」；上界 ≥ 阈值时凭上界判不出（需要第二段实测）。
fn additive_conclusion_from_upper_bound(threshold_sum: RationalNumber) -> Option<bool> {
    if additive_load_level_cost_upper_bound() < threshold_sum {
        Some(false)
    } else {
        None
    }
}

// ============================== 两类模型换算 ==============================

/// 甲臂：p_N = ratio·p_B ÷ (ratio·p_B + 1 − p_B)。
fn request_share_from_byte_share(byte_share: RationalNumber, sequential_to_random_size_ratio: RationalNumber) -> RationalNumber {
    let scaled = sequential_to_random_size_ratio * byte_share;
    scaled / (scaled + (RationalNumber::one() - byte_share))
}
/// 反方向：p_B = p_N ÷ (p_N + (1 − p_N)·ratio)。用户若按请求数给权重 v 时用这条（登记第五节甲臂）；
/// 今天的真实基线是按字节给权重（D25（目标负载优先级） 已定项 6：项目今天认的来源是人给的权重，没有按请求数给的数），
/// 主报告不调用，正确性由 two_class_conversion_round_trips_in_both_directions 钉住。
#[allow(dead_code, reason = "两种换算都实现（登记第五节甲臂），今天没有按请求数给权重的真实输入，主报告只用字节方向")]
fn byte_share_from_request_share(request_share: RationalNumber, sequential_to_random_size_ratio: RationalNumber) -> RationalNumber {
    request_share / (request_share + (RationalNumber::one() - request_share) * sequential_to_random_size_ratio)
}

// ============================== 页到单元算术（S1、B1、B2、B8） ==============================

/// 第 `page_index` 页（0 起）是否跨在两个单元里，净荷 `payload_bytes`。
fn spans_two_units_given_payload(payload_bytes: u64, page_index: u64) -> bool {
    let first_byte = page_index * PAGE_BYTES;
    let last_byte = first_byte + PAGE_BYTES - 1;
    first_byte / payload_bytes != last_byte / payload_bytes
}
/// D4（校验和位置） 已定项 5 的含头形态：按净荷除法定单元（不按整单元除）。
fn spans_two_data_units_at_page(page_index: u64) -> bool {
    spans_two_units_given_payload(NET_PAYLOAD_BYTES, page_index) // M14 目标：按 32768 除
}
fn count_spanning_pages_given_payload(payload_bytes: u64, page_count: u64) -> u64 {
    (0..page_count).filter(|&page_index| spans_two_units_given_payload(payload_bytes, page_index)).count() as u64
}

/// 无头臂（L16 / L32 / L32p）：净荷恒是单元的整数倍，每次读满一个单元、从不跨。
fn device_bytes_no_header_arm(unit_bytes: u64, page_count: u64) -> u64 {
    unit_bytes * page_count
}
/// 含头臂（L32h）：不跨读一个单元，跨读两个单元。
fn device_bytes_header_arm(payload_bytes: u64, unit_bytes: u64, page_count: u64) -> u64 {
    let spanning_pages = count_spanning_pages_given_payload(payload_bytes, page_count);
    unit_bytes * (page_count + spanning_pages)
}

// ============================== 读法 A（判据 1） ==============================

/// E[u_G(k)] = 1 + (k − 1) ÷ (G / 4096)：k 页覆盖的请求，在单元粒度 unit_pages 下期望碰到的单元数。
fn expected_units_touched(pages_covered: u64, unit_pages: u64) -> RationalNumber {
    RationalNumber::one() + RationalNumber::from_integer((pages_covered - 1) as i128) / RationalNumber::from_integer(unit_pages as i128) // M6 目标：漏掉 -1
}
/// 单一尺寸（k 页）的读法 A 倍数。
fn read_form_a_multiplier_single_size(pages_covered: u64) -> RationalNumber {
    anchor_multiplier() * expected_units_touched(pages_covered, UNIT_PAGES_32) / expected_units_touched(pages_covered, UNIT_PAGES_16)
}
/// 多尺寸混合：均值之比（正确做法），不是比值的均值。`size_distribution` 是
/// (请求覆盖的 4 KiB 页数, 这类请求占 R 类请求数的份额)，份额之和应为 1。
fn read_form_a_multiplier_over_distribution(size_distribution: &[(u64, RationalNumber)]) -> RationalNumber {
    let mut weighted_units_32 = RationalNumber::zero();
    let mut weighted_units_16 = RationalNumber::zero();
    for &(pages_covered, share) in size_distribution {
        weighted_units_32 = weighted_units_32 + share * expected_units_touched(pages_covered, UNIT_PAGES_32);
        weighted_units_16 = weighted_units_16 + share * expected_units_touched(pages_covered, UNIT_PAGES_16);
    }
    anchor_multiplier() * weighted_units_32 / weighted_units_16 // M7 目标：均值之比改成比值的均值
}
/// R 类恒为 4 KiB 请求（甲、丙两类模型的定义）。
fn pure_four_kibibyte_distribution() -> Vec<(u64, RationalNumber)> {
    vec![(1, RationalNumber::one())]
}
/// 给 Q13 用：判据 1 的值是不是与随机小读占比无关，只取决于 R 类内部尺寸混合。
/// 正确实现完全不看 `byte_share`；M21 把占比混进来验证 Q13 抓得住。
fn instance_judgement_one_value(byte_share: RationalNumber, size_distribution: &[(u64, RationalNumber)]) -> RationalNumber {
    let _ = byte_share;
    read_form_a_multiplier_over_distribution(size_distribution) // M21 目标
}

// ============================== 甲臂：由人给权重 ==============================

/// 五档顺序：seq、metaheavy、rand、multistream、smallfile（D25（目标负载优先级） 已定项 1 表的次序）。
type FiveTierShares = [RationalNumber; 5];

fn weighted_byte_share(weights: FiveTierShares, random_shares_within_tier: FiveTierShares) -> RationalNumber {
    (0..5).fold(RationalNumber::zero(), |accumulator, tier_index| {
        accumulator + weights[tier_index] * random_shares_within_tier[tier_index] // M18 目标：不按权重、直接取 b 的平均
    })
}

fn epsilon_weights(epsilon: RationalNumber) -> FiveTierShares {
    let other_tier_weight = epsilon / RationalNumber::from_integer(4);
    [RationalNumber::one() - epsilon, other_tier_weight, other_tier_weight, other_tier_weight, other_tier_weight]
}

/// M1：「只有 rand 档有随机小读」(0, 0, 1, 0, 0)。
fn random_share_mapping_only_random_tier() -> FiveTierShares {
    [RationalNumber::zero(), RationalNumber::zero(), RationalNumber::one(), RationalNumber::zero(), RationalNumber::zero()]
}
/// M2：「小操作档都是随机小读」(0, 1, 1, 0, 1)。
fn random_share_mapping_small_operation_tiers() -> FiveTierShares {
    [RationalNumber::zero(), RationalNumber::one(), RationalNumber::one(), RationalNumber::zero(), RationalNumber::one()]
}

const EPSILON_SAMPLE_POINTS: [(&str, i128, i128); 7] =
    [("0", 0, 1), ("1%", 1, 100), ("2%", 2, 100), ("5%", 5, 100), ("20%", 20, 100), ("50%", 50, 100), ("80%", 80, 100)];
const OMEGA_SAMPLE_POINTS: [(&str, i128, i128); 12] = [
    ("0", 0, 1),
    ("0.25%", 25, 10000),
    ("0.5%", 5, 1000),
    ("1.0%", 1, 100),
    ("1.25%", 125, 10000),
    ("1.5%", 15, 1000),
    ("2%", 2, 100),
    ("5%", 5, 100),
    ("10%", 10, 100),
    ("25%", 25, 100),
    ("50%", 50, 100),
    ("100%", 100, 100),
];

// ============================== 丙臂：E152 作业混合 ==============================

/// 丙-规格：随机作业读的字节数 R = X·4096·20。
fn declared_random_read_bytes(random_reads_per_second: RationalNumber) -> RationalNumber {
    random_reads_per_second * RationalNumber::from_integer(PAGE_BYTES as i128) * RationalNumber::from_integer(RANDOM_JOB_RUNTIME_SECONDS) // M17 目标
}
/// p_B(X) = R ÷ (R + 4 GiB)。
fn declared_byte_share(random_reads_per_second: RationalNumber) -> RationalNumber {
    let random_read_bytes = declared_random_read_bytes(random_reads_per_second);
    let sequential_read_bytes = RationalNumber::from_integer(DECLARED_SEQUENTIAL_FILE_BYTES);
    random_read_bytes / (random_read_bytes + sequential_read_bytes) // M1、M10 目标
}
/// p_N(X) = 20X ÷ (20X + 4096)（顺序作业 4 GiB / 1 MiB = 4096 次）。
fn declared_request_share(random_reads_per_second: RationalNumber) -> RationalNumber {
    let random_read_requests = random_reads_per_second * RationalNumber::from_integer(RANDOM_JOB_RUNTIME_SECONDS);
    let sequential_read_requests =
        RationalNumber::from_integer(DECLARED_SEQUENTIAL_FILE_BYTES / DECLARED_SEQUENTIAL_REQUEST_BYTES);
    random_read_requests / (random_read_requests + sequential_read_requests)
}
/// 反解 X*：给定目标 p_B，求要多少 IOPS。
fn declared_reads_per_second_for_byte_share(target_byte_share: RationalNumber) -> RationalNumber {
    let sequential_read_bytes = RationalNumber::from_integer(DECLARED_SEQUENTIAL_FILE_BYTES);
    (target_byte_share * sequential_read_bytes)
        / ((RationalNumber::one() - target_byte_share) * RationalNumber::from_integer(RANDOM_JOB_RUNTIME_SECONDS * PAGE_BYTES as i128))
}

/// 丙-实测：解析 `research/results/` 下归档的 E152 原始输出一行，只取需要的字段。
fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    let prefix = format!("{key}=");
    line.split_whitespace().find_map(|token| token.strip_prefix(prefix.as_str()))
}

#[derive(Debug, Clone)]
struct MeasuredE152Row {
    configuration: String,
    job: String,
    read_kibibytes: i128,
}

fn parse_e152_rows(raw_output: &str) -> Vec<MeasuredE152Row> {
    raw_output
        .lines()
        .filter(|line| line.contains("verdict=counted") && line.contains("job="))
        .filter_map(|line| {
            let configuration = field_value(line, "configuration")?.to_string();
            let job = field_value(line, "job")?.to_string();
            let read_kibibytes: i128 = field_value(line, "read_kibibytes")?.parse().ok()?;
            Some(MeasuredE152Row { configuration, job, read_kibibytes })
        })
        .collect()
}

/// 每配置 5 轮（E152（按里程碑对比六家文件系统的文件性能） 实验页第 272 行），取中位。
fn median_read_kibibytes(rows: &[MeasuredE152Row], configuration: &str, job: &str) -> i128 {
    let mut values: Vec<i128> =
        rows.iter().filter(|row| row.configuration == configuration && row.job == job).map(|row| row.read_kibibytes).collect();
    values.sort_unstable();
    assert_eq!(values.len(), 5, "{configuration}/{job} 应有 5 轮，实得 {}", values.len());
    values[2]
}

/// X = read_kibibytes × 1024 ÷ 4096 ÷ 20（登记第五节丙-实测）。
fn measured_random_reads_per_second(median_random_read_kibibytes: i128) -> RationalNumber {
    RationalNumber::new(median_random_read_kibibytes * 1024, (PAGE_BYTES as i128) * RANDOM_JOB_RUNTIME_SECONDS)
}
/// 顺序作业实测字节应与丙-规格假设的 4 GiB 一致；不一致只报差异，不作废（登记没有把它列进 V 条款）。
fn measured_sequential_matches_declared_assumption(median_sequential_read_kibibytes: i128) -> bool {
    median_sequential_read_kibibytes * 1024 == DECLARED_SEQUENTIAL_FILE_BYTES
}

fn distinct_configurations(rows: &[MeasuredE152Row], job: &str) -> Vec<String> {
    let mut configurations: Vec<String> =
        rows.iter().filter(|row| row.job == job).map(|row| row.configuration.clone()).collect();
    configurations.sort();
    configurations.dedup();
    configurations
}

// ============================== 乙臂：公开参考负载 ==============================

/// 一份读负载的随机/顺序拆分；字段可能缺失（读不到），不许当 0 处理。
#[derive(Debug, Clone)]
struct RandomReadMixSource {
    label: &'static str,
    citation: &'static str,
    random_total_bytes: Option<RationalNumber>,
    random_total_requests: Option<RationalNumber>,
    /// (请求覆盖的 4 KiB 页数, 这一档占 R 类请求数的份额；份额之和 = random_total_requests 对应的份额，不必是 1)
    random_size_distribution: Vec<(u64, RationalNumber)>,
    sequential_total_bytes: Option<RationalNumber>,
    sequential_total_requests: Option<RationalNumber>,
}

/// 缺请求数就是「读不到」，不许填 0（M12 目标）。
fn share_from_two_totals(part: Option<RationalNumber>, other_part: Option<RationalNumber>) -> Option<RationalNumber> {
    let part_value = part?; // M12 目标
    let other_value = other_part?;
    Some(part_value / (part_value + other_value))
}

fn byte_share_of_source(source: &RandomReadMixSource) -> Option<RationalNumber> {
    share_from_two_totals(source.random_total_bytes, source.sequential_total_bytes)
}
fn request_share_of_source(source: &RandomReadMixSource) -> Option<RationalNumber> {
    share_from_two_totals(source.random_total_requests, source.sequential_total_requests)
}
fn judgement_one_value_of_source(source: &RandomReadMixSource) -> Option<RationalNumber> {
    if source.random_size_distribution.is_empty() {
        return None;
    }
    let total_share =
        source.random_size_distribution.iter().fold(RationalNumber::zero(), |accumulator, &(_, share)| accumulator + share);
    if total_share == RationalNumber::zero() {
        return None;
    }
    let normalized: Vec<(u64, RationalNumber)> =
        source.random_size_distribution.iter().map(|&(pages, share)| (pages, share / total_share)).collect();
    Some(read_form_a_multiplier_over_distribution(&normalized))
}

/// 事实 1（`research/prompts/e160-prior-art.md:26-27,41-53`）：fio 上游
/// `examples/iometer-file-access-server.fio`，`bssplit=512/10:1k/5:2k/5:4k/60:8k/2:16k/4:32k/4:64k/10`
/// （第 41 行），`percentage_random` 未设，HOWTO 默认 100%（第一行事实 1 引的定义），全部随机；
/// 读、写共用同一份 `bssplit`，读子集与全体同分布（事实 2 的说明）。(size_bytes, 请求数占比的百分之几)。
const FIO_IOMETER_BSSPLIT: [(u64, i128); 8] =
    [(512, 10), (1024, 5), (2048, 5), (4096, 60), (8192, 2), (16384, 4), (32768, 4), (65536, 10)];

/// `random_size_cutoff_bytes` 是 R 类的上界（不含）：主值传 32768（「小于 32768 字节」），
/// R4 敏感性（G2）传 4097（「≤ 4096 字节」）。
fn fio_iometer_source(random_size_cutoff_bytes: u64, label: &'static str) -> RandomReadMixSource {
    let mut random_total_requests = RationalNumber::zero();
    let mut random_total_bytes = RationalNumber::zero();
    let mut sequential_total_requests = RationalNumber::zero();
    let mut sequential_total_bytes = RationalNumber::zero();
    let mut random_size_distribution = Vec::new();
    for &(size_bytes, percent) in FIO_IOMETER_BSSPLIT.iter() {
        let share = RationalNumber::new(percent, 100);
        let bytes_share = share * RationalNumber::from_integer(size_bytes as i128);
        if size_bytes < random_size_cutoff_bytes {
            random_total_requests = random_total_requests + share;
            random_total_bytes = random_total_bytes + bytes_share;
            random_size_distribution.push((size_bytes.div_ceil(PAGE_BYTES), share));
        } else {
            sequential_total_requests = sequential_total_requests + share;
            sequential_total_bytes = sequential_total_bytes + bytes_share;
        }
    }
    RandomReadMixSource {
        label,
        citation: "research/prompts/e160-prior-art.md:26-27,41-53",
        random_total_bytes: Some(random_total_bytes),
        random_total_requests: Some(random_total_requests),
        random_size_distribution,
        sequential_total_bytes: Some(sequential_total_bytes),
        sequential_total_requests: Some(sequential_total_requests),
    }
}

/// 事实 3／4（`research/prompts/e160-prior-art.md:28-29`）：Kavalanekar 等 IISWC 2008，
/// 块层、按请求数的边际分布（平均请求大小、众数、「顺序发起占比」），
/// 没有「随机且 < 32 KiB」的联合统计，也没有字节口径 —— p_B、p_N 都读不到。
fn kavalanekar_source(label: &'static str) -> RandomReadMixSource {
    RandomReadMixSource {
        label,
        citation: "research/prompts/e160-prior-art.md:28-29",
        random_total_bytes: None,
        random_total_requests: None,
        random_size_distribution: Vec::new(),
        sequential_total_bytes: None,
        sequential_total_requests: None,
    }
}

/// 事实 5／6（`research/prompts/e160-prior-art.md:30-31`）：三方比较论文只给大小分位数（A.2）
/// 与用 128 KiB 阈值定义的随机度（B.9），两者都只是边际分布，论文自己没有把两者交叉成一个数，
/// 本报告据此也不替它凑（事实 6 的说明）—— p_B、p_N 都读不到。
fn three_way_comparison_source(label: &'static str) -> RandomReadMixSource {
    RandomReadMixSource {
        label,
        citation: "research/prompts/e160-prior-art.md:30-31",
        random_total_bytes: None,
        random_total_requests: None,
        random_size_distribution: Vec::new(),
        sequential_total_bytes: None,
        sequential_total_requests: None,
    }
}

fn prior_art_sources() -> Vec<RandomReadMixSource> {
    vec![
        fio_iometer_source(32768, "乙1_fio_iometer_文件服务器定义值_R32切点"),
        fio_iometer_source(4097, "乙1a_fio_iometer_文件服务器定义值_R4切点_G2敏感性"),
        kavalanekar_source("乙2_kavalanekar2008_tpc_c"),
        kavalanekar_source("乙3_kavalanekar2008_tpc_e"),
        kavalanekar_source("乙4_kavalanekar2008_build_server"),
        kavalanekar_source("乙5_kavalanekar2008_dtrs"),
        three_way_comparison_source("乙6_三方比较_alicloud"),
        three_way_comparison_source("乙7_三方比较_tencentcloud"),
        three_way_comparison_source("乙8_三方比较_msrc"),
    ]
}

// ============================== main ==============================

fn print_and_emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();

    print_and_emit(
        &mut emitter,
        &format!(
            "name=configuration unit_bytes_32={UNIT_BYTES_32} unit_bytes_16={UNIT_BYTES_16} header_registered_bytes={HEADER_REGISTERED_BYTES} nonce_mac_reserved_bytes={NONCE_MAC_RESERVED_BYTES} net_payload_bytes={NET_PAYLOAD_BYTES} padded_payload_bytes={PADDED_PAYLOAD_BYTES} anchor_multiplier={} judgement_one_threshold={} band_lower={} band_upper={} additive_threshold_primary={}",
            anchor_multiplier().as_plain_text(),
            judgement_one_threshold().as_plain_text(),
            band_lower_bound().as_percent_text(),
            band_upper_bound().as_percent_text(),
            additive_threshold_primary().as_plain_text()
        ),
    );

    // ---------- S1/B1/B2/B8：页到单元算术与读放大参照（第一段的静态几何，不含计时） ----------
    let period_pages = 16317u64;
    print_and_emit(
        &mut emitter,
        &format!(
            "name=unit_geometry period_pages={period_pages} spanning_pages={} spans_page_at_period_boundary={} device_amplification_l16={} device_amplification_l32={} device_amplification_l32_padded={} device_amplification_l32h={}",
            count_spanning_pages_given_payload(NET_PAYLOAD_BYTES, period_pages),
            spans_two_data_units_at_page(period_pages),
            device_bytes_no_header_arm(UNIT_BYTES_16, period_pages) / (period_pages * PAGE_BYTES),
            device_bytes_no_header_arm(UNIT_BYTES_32, period_pages) / (period_pages * PAGE_BYTES),
            device_bytes_no_header_arm(UNIT_BYTES_32, period_pages) / (period_pages * PAGE_BYTES),
            RationalNumber::new(
                device_bytes_header_arm(NET_PAYLOAD_BYTES, UNIT_BYTES_32, period_pages) as i128,
                (period_pages * PAGE_BYTES) as i128
            )
            .as_plain_text()
        ),
    );
    for &pages_covered in &[1u64, 2, 3, 4, 7] {
        print_and_emit(
            &mut emitter,
            &format!(
                "name=read_form_a_single_size pages_covered={pages_covered} multiplier={}",
                read_form_a_multiplier_single_size(pages_covered).as_plain_text()
            ),
        );
    }
    // ---------- 5.4 正对照 f 行：相加规则的代价示例（真实 C1、C5 要第二段实测，这里只演示合成规则的算术） ----------
    for &(threshold_name, threshold) in
        &[("0.136", additive_threshold_primary()), ("0.05", additive_threshold_alternate_low()), ("0", additive_threshold_alternate_zero())]
    {
        print_and_emit(
            &mut emitter,
            &format!(
                "name=additive_rule_demo load_level_cost=0.08 header_cost=0.07 threshold={threshold_name} reopens={}",
                additive_reopens(RationalNumber::new(8, 100), RationalNumber::new(7, 100), threshold)
            ),
        );
    }

    // ---------- 甲：ε 取样（两组映射 × 7 个 ε） ----------
    for &(mapping_name, mapping) in
        &[("only_random_tier", random_share_mapping_only_random_tier()), ("small_operation_tiers", random_share_mapping_small_operation_tiers())]
    {
        for &(epsilon_text, epsilon_numerator, epsilon_denominator) in EPSILON_SAMPLE_POINTS.iter() {
            let epsilon = RationalNumber::new(epsilon_numerator, epsilon_denominator);
            let weights = epsilon_weights(epsilon);
            let byte_share = weighted_byte_share(weights, mapping);
            emit_weight_instance(&mut emitter, "epsilon", mapping_name, epsilon_text, byte_share);
        }
    }
    // ---------- 甲：ω 取样（直接给定占比，画函数的样子） ----------
    for &(omega_text, omega_numerator, omega_denominator) in OMEGA_SAMPLE_POINTS.iter() {
        let omega = RationalNumber::new(omega_numerator, omega_denominator);
        emit_weight_instance(&mut emitter, "omega", "direct", omega_text, omega);
    }

    // ---------- 乙：公开参考负载 ----------
    for source in prior_art_sources() {
        emit_prior_art_instance(&mut emitter, &source);
    }

    // ---------- 丙-规格 ----------
    let declared_sample_points: Vec<(&str, RationalNumber)> = vec![
        ("X*/2_at_0.5025%", declared_reads_per_second_for_byte_share(band_lower_bound()) / RationalNumber::from_integer(2)),
        ("X*_at_1.0%", declared_reads_per_second_for_byte_share(band_lower_bound())),
        ("X*_at_1.5%", declared_reads_per_second_for_byte_share(band_upper_bound())),
        ("2X*_at_1.5%", declared_reads_per_second_for_byte_share(band_upper_bound()) * RationalNumber::from_integer(2)),
        ("10X*_at_1.5%", declared_reads_per_second_for_byte_share(band_upper_bound()) * RationalNumber::from_integer(10)),
        ("positive_control_1000", RationalNumber::from_integer(1000)),
        ("positive_control_850", RationalNumber::from_integer(850)),
    ];
    for (point_label, reads_per_second) in declared_sample_points {
        emit_declared_instance(&mut emitter, point_label, reads_per_second);
    }

    // ---------- 丙-实测：E152 归档的原始输出 ----------
    let single_disk_raw = include_str!("../../../results/e160-e152-input-single-2026-09-15.out");
    let mirror_raw = include_str!("../../../results/e160-e152-input-mirror-2026-09-15.out");
    let single_disk_rows = parse_e152_rows(single_disk_raw);
    let mirror_rows = parse_e152_rows(mirror_raw);
    print_and_emit(
        &mut emitter,
        &format!(
            "name=e152_input_lines single_lines={} single_last_line={:?} mirror_lines={} mirror_last_line={:?}",
            single_disk_raw.lines().count(),
            single_disk_raw.lines().last().unwrap_or(""),
            mirror_raw.lines().count(),
            mirror_raw.lines().last().unwrap_or("")
        ),
    );
    let mut measured_byte_shares = Vec::new();
    for (source_label, rows) in [("single", &single_disk_rows), ("mirror", &mirror_rows)] {
        for configuration in distinct_configurations(rows, "read_random_4k") {
            let random_median = median_read_kibibytes(rows, &configuration, "read_random_4k");
            let sequential_median = median_read_kibibytes(rows, &configuration, "read_sequential_large");
            let reads_per_second = measured_random_reads_per_second(random_median);
            let byte_share = declared_byte_share(reads_per_second);
            let request_share = declared_request_share(reads_per_second);
            let side = classify_against_band(byte_share, band_lower_bound(), band_upper_bound());
            let judgement_one_value = instance_judgement_one_value(byte_share, &pure_four_kibibyte_distribution());
            let judgement_one = judgement_one_triggered(judgement_one_value);
            measured_byte_shares.push(byte_share);
            print_and_emit(
                &mut emitter,
                &format!(
                    "name=measured_c_arm source={source_label} configuration={configuration} random_median_read_kibibytes={random_median} sequential_median_read_kibibytes={sequential_median} sequential_matches_declared_assumption={} byte_share_percent={} request_share_percent={} judgement_one_value={} judgement_one_triggered={judgement_one} judgement_five_side={} rule_first_to_trigger={} rule_only_one={} rule_only_five={}",
                    measured_sequential_matches_declared_assumption(sequential_median),
                    byte_share.as_percent_text(),
                    request_share.as_percent_text(),
                    judgement_one_value.as_plain_text(),
                    side.as_text(),
                    if first_to_trigger_reopens(judgement_one, side) { "重开" } else { "不重开" },
                    if only_judgement_one_reopens(judgement_one) { "重开" } else { "不重开" },
                    if only_judgement_five_reopens(side) { "重开" } else { "不重开" }
                ),
            );
        }
    }
    measured_byte_shares.sort();
    let minimum = *measured_byte_shares.first().expect("14 个配置至少 1 个");
    let maximum = *measured_byte_shares.last().expect("14 个配置至少 1 个");
    let median = measured_byte_shares[measured_byte_shares.len() / 2];
    let (below, within, above) = measured_byte_shares.iter().fold((0u32, 0u32, 0u32), |(below, within, above), &value| {
        match classify_against_band(value, band_lower_bound(), band_upper_bound()) {
            ThresholdSide::BelowLowerBound => (below + 1, within, above),
            ThresholdSide::WithinBand => (below, within + 1, above),
            ThresholdSide::AboveUpperBound => (below, within, above + 1),
        }
    });
    print_and_emit(
        &mut emitter,
        &format!(
            "name=measured_c_arm_summary count={} minimum_byte_share_percent={} median_byte_share_percent={} maximum_byte_share_percent={} count_below_lower={below} count_within_band={within} count_above_upper={above}",
            measured_byte_shares.len(),
            minimum.as_percent_text(),
            median.as_percent_text(),
            maximum.as_percent_text()
        ),
    );

    // ---------- Q13：判据 1 的值是不是与随机小读占比无关 ----------
    let weight_arm_values: Vec<RationalNumber> = EPSILON_SAMPLE_POINTS
        .iter()
        .map(|&(_, numerator, denominator)| {
            let epsilon = RationalNumber::new(numerator, denominator);
            let weights = epsilon_weights(epsilon);
            let byte_share = weighted_byte_share(weights, random_share_mapping_only_random_tier());
            instance_judgement_one_value(byte_share, &pure_four_kibibyte_distribution())
        })
        .collect();
    let declared_arm_values: Vec<RationalNumber> = [1000i128, 850, 1, 100000]
        .iter()
        .map(|&reads_per_second| {
            let byte_share = declared_byte_share(RationalNumber::from_integer(reads_per_second));
            instance_judgement_one_value(byte_share, &pure_four_kibibyte_distribution())
        })
        .collect();
    let weight_arm_all_equal = weight_arm_values.windows(2).all(|pair| pair[0] == pair[1]);
    let declared_arm_all_equal = declared_arm_values.windows(2).all(|pair| pair[0] == pair[1]);
    print_and_emit(
        &mut emitter,
        &format!(
            "name=q13 weight_arm_all_equal={weight_arm_all_equal} weight_arm_reference={} declared_arm_all_equal={declared_arm_all_equal} declared_arm_reference={} reading={}",
            weight_arm_values[0].as_plain_text(),
            declared_arm_values[0].as_plain_text(),
            if weight_arm_all_equal && declared_arm_all_equal { "无关" } else { "有关" }
        ),
    );

    println!("{}", emitter.finish());
}

/// 判据 5 要按字节口径的占比判，不是按请求次数口径（M2 抓的正是把这两者用反）。
fn weight_instance_judgement_five_side(weight_byte_share: RationalNumber, request_share: RationalNumber) -> ThresholdSide {
    let _ = request_share;
    classify_against_band(weight_byte_share, band_lower_bound(), band_upper_bound())
}

fn emit_weight_instance(emitter: &mut Emitter, kind: &str, mapping_name: &str, parameter_text: &str, weight_byte_share: RationalNumber) {
    let request_share = request_share_from_byte_share(weight_byte_share, RationalNumber::from_integer(DEFAULT_SEQUENTIAL_TO_RANDOM_SIZE_RATIO));
    let side = weight_instance_judgement_five_side(weight_byte_share, request_share);
    let judgement_one_value = instance_judgement_one_value(weight_byte_share, &pure_four_kibibyte_distribution());
    let judgement_one = judgement_one_triggered(judgement_one_value);
    let additive_at_primary = additive_conclusion_from_upper_bound(additive_threshold_primary());
    let additive_at_low = additive_conclusion_from_upper_bound(additive_threshold_alternate_low());
    let additive_at_zero = additive_conclusion_from_upper_bound(additive_threshold_alternate_zero());
    let lower_bound_as_crossed_side = classify_against_band_treating_lower_bound_as_crossed(weight_byte_share, band_lower_bound());
    print_and_emit(
        emitter,
        &format!(
            "name=weight_sample kind={kind} mapping={mapping_name} parameter={parameter_text} byte_share_percent={} request_share_percent={} judgement_one_value={} judgement_one_triggered={judgement_one} judgement_five_side={} g1_side_lower_bound_as_crossed={lower_bound_as_crossed_side} rule_first_to_trigger={} rule_only_one={} rule_only_five={} additive_upper_bound_at_0.136={} additive_upper_bound_at_0.05={} additive_upper_bound_at_0={}",
            weight_byte_share.as_percent_text(),
            request_share.as_percent_text(),
            judgement_one_value.as_plain_text(),
            side.as_text(),
            if first_to_trigger_reopens(judgement_one, side) { "重开" } else { "不重开" },
            if only_judgement_one_reopens(judgement_one) { "重开" } else { "不重开" },
            if only_judgement_five_reopens(side) { "重开" } else { "不重开" },
            option_bool_text(additive_at_primary),
            option_bool_text(additive_at_low),
            option_bool_text(additive_at_zero)
        ),
    );
}

fn emit_prior_art_instance(emitter: &mut Emitter, source: &RandomReadMixSource) {
    let byte_share = byte_share_of_source(source);
    let request_share = request_share_of_source(source);
    let judgement_one_value = judgement_one_value_of_source(source);
    let judgement_one = judgement_one_value.map(judgement_one_triggered);
    let side = byte_share.map(|value| classify_against_band(value, band_lower_bound(), band_upper_bound()));
    print_and_emit(
        emitter,
        &format!(
            "name=prior_art_source label={} citation={} byte_share_percent={} request_share_percent={} judgement_one_value={} judgement_one_triggered={} judgement_five_side={} rule_first_to_trigger={} rule_only_one={} rule_only_five={}",
            source.label,
            source.citation,
            option_percent_text(byte_share),
            option_percent_text(request_share),
            option_plain_text(judgement_one_value),
            judgement_one.map(|value| value.to_string()).unwrap_or_else(|| "读不到".to_string()),
            side.map(ThresholdSide::as_text).unwrap_or("读不到"),
            option_bool_text(first_to_trigger_reopens_optional(judgement_one, side)),
            option_bool_text(judgement_one),
            option_bool_text(side.map(judgement_five_reopens))
        ),
    );
}

fn emit_declared_instance(emitter: &mut Emitter, point_label: &str, reads_per_second: RationalNumber) {
    let byte_share = declared_byte_share(reads_per_second);
    let request_share = declared_request_share(reads_per_second);
    let side = classify_against_band(byte_share, band_lower_bound(), band_upper_bound());
    let judgement_one_value = instance_judgement_one_value(byte_share, &pure_four_kibibyte_distribution());
    let judgement_one = judgement_one_triggered(judgement_one_value);
    print_and_emit(
        emitter,
        &format!(
            "name=declared_c_arm point={point_label} reads_per_second={} byte_share_percent={} request_share_percent={} judgement_one_value={} judgement_one_triggered={judgement_one} judgement_five_side={} rule_first_to_trigger={} rule_only_one={} rule_only_five={}",
            reads_per_second.as_plain_text(),
            byte_share.as_percent_text(),
            request_share.as_percent_text(),
            judgement_one_value.as_plain_text(),
            side.as_text(),
            if first_to_trigger_reopens(judgement_one, side) { "重开" } else { "不重开" },
            if only_judgement_one_reopens(judgement_one) { "重开" } else { "不重开" },
            if only_judgement_five_reopens(side) { "重开" } else { "不重开" }
        ),
    );
}

// ============================== 单测 ==============================

#[cfg(test)]
mod tests {
    use super::*;

    /// 第一类：出自被测条款本身（A1、A2、A3、A4、A5、A6，登记第七节）。
    #[test]
    fn constants_match_the_cited_clauses() {
        assert_eq!(judgement_one_threshold(), RationalNumber::new(142, 125), "A1、B9：1086/1000 + 5/100 = 142/125");
        assert_eq!(band_lower_bound(), RationalNumber::new(1, 100), "A2");
        assert_eq!(band_upper_bound(), RationalNumber::new(3, 200), "A2");
        assert_eq!(UNIT_BYTES_32, 32768, "A3");
        let spanning = count_spanning_pages_given_payload(NET_PAYLOAD_BYTES, 16317);
        assert!((10 * 16317..20 * 16317).contains(&(spanning * 100)), "A4：一成多跨在两个单元上");
        assert_eq!(spanning, 2047);
        assert_eq!(anchor_multiplier() - RationalNumber::one(), RationalNumber::new(43, 500), "A5：约一成 = 8.6%");
        assert_eq!(RationalNumber::one(), RationalNumber::new(1, 1), "A6：m_q = 1（顺序侧设备侧为零）");
        // M11 目标：两类模型默认比是 256（1 MiB ÷ 4 KiB），不是 128。
        assert_eq!(DEFAULT_SEQUENTIAL_TO_RANDOM_SIZE_RATIO, 256);
    }

    /// 第二类 B1：净荷、gcd、跨单元页数。
    #[test]
    fn page_to_unit_arithmetic_matches_the_cited_clause() {
        assert_eq!(NET_PAYLOAD_BYTES, 32634);
        assert_eq!(greatest_common_divisor(PAGE_BYTES as i128, NET_PAYLOAD_BYTES as i128), 2);
        assert_eq!(count_spanning_pages_given_payload(NET_PAYLOAD_BYTES, 16317), 2047);
        assert_eq!(count_spanning_pages_given_payload(32634, 32633), 4094);
        assert_eq!(count_spanning_pages_given_payload(32635, 32633), 4095);
        assert_eq!(count_spanning_pages_given_payload(32633, 32633), 4095);
        assert!(!spans_two_units_given_payload(32634, 16317));
        assert!(spans_two_units_given_payload(32635, 16317));
        assert!(!spans_two_data_units_at_page(16317), "S1：与 crates 的单测结论一致（2026-09-24 现查，见报告）");
        // M14 目标：spans_two_data_units_at_page 内部必须按净荷除，不按整单元除
        // （按整单元除时 32768 是 4096 的整数倍，永远不会跨，下面这条会从 2047 变成 0）。
        let spanning_via_wrapper = (0..16317).filter(|&page_index| spans_two_data_units_at_page(page_index)).count() as u64;
        assert_eq!(spanning_via_wrapper, 2047);
    }

    /// 第二类 B2：补齐形态，7×4096，从不跨，多花 1/7。
    #[test]
    fn padded_form_never_spans_and_costs_one_seventh_more() {
        assert_eq!(PADDED_PAYLOAD_BYTES, 7 * PAGE_BYTES);
        assert_eq!(count_spanning_pages_given_payload(PADDED_PAYLOAD_BYTES, 7000), 0);
        assert_eq!(
            RationalNumber::from_integer(UNIT_BYTES_32 as i128) / RationalNumber::from_integer(PADDED_PAYLOAD_BYTES as i128)
                - RationalNumber::one(),
            RationalNumber::new(1, 7)
        );
    }

    /// 第二类 B3：丙-规格的 X*、正对照。
    #[test]
    fn declared_workload_arm_positive_controls() {
        let reads_per_second_at_one_percent = declared_reads_per_second_for_byte_share(RationalNumber::new(1, 100));
        let reads_per_second_at_one_point_five_percent = declared_reads_per_second_for_byte_share(RationalNumber::new(3, 200));
        assert!((reads_per_second_at_one_percent.to_approximate_f64() - 529.5838).abs() < 1e-3);
        assert!((declared_request_share(reads_per_second_at_one_percent).to_approximate_f64() - 0.7211267605633803).abs() < 1e-9);
        assert!((reads_per_second_at_one_point_five_percent.to_approximate_f64() - 798.4081218274112).abs() < 1e-3);
        assert!((declared_request_share(reads_per_second_at_one_point_five_percent).to_approximate_f64() - 0.7958549222797927).abs() < 1e-9);
        let at_1000 = declared_byte_share(RationalNumber::from_integer(1000));
        assert!((at_1000.to_approximate_f64() - 0.018716497469529542).abs() < 1e-12);
        assert!((declared_request_share(RationalNumber::from_integer(1000)).to_approximate_f64() - 0.8300132802124834).abs() < 1e-9);
        let at_850 = declared_byte_share(RationalNumber::from_integer(850));
        assert!((at_850.to_approximate_f64() - 0.015953812773560967).abs() < 1e-12);
        assert_eq!(classify_against_band(at_1000, band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound);
        assert_eq!(classify_against_band(at_850, band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound);
    }

    /// 第二类 B4：两类模型换算，比取 256（默认）与 32（只报数）。
    #[test]
    fn two_class_conversion_matches_the_cited_values() {
        let one_percent = RationalNumber::new(1, 100);
        let one_point_five_percent = RationalNumber::new(3, 200);
        assert!((request_share_from_byte_share(one_percent, RationalNumber::from_integer(256)).to_approximate_f64() - 0.7211267605633803).abs() < 1e-9);
        assert!((request_share_from_byte_share(one_point_five_percent, RationalNumber::from_integer(256)).to_approximate_f64() - 0.7958549222797927).abs() < 1e-9);
        assert!(
            (request_share_from_byte_share(one_percent, RationalNumber::from_integer(ALTERNATE_SEQUENTIAL_REQUEST_SIZE_RATIO_128_KIBIBYTES))
                .to_approximate_f64()
                - 0.24427480916030533)
                .abs()
                < 1e-9
        );
        assert!(
            (request_share_from_byte_share(one_point_five_percent, RationalNumber::from_integer(ALTERNATE_SEQUENTIAL_REQUEST_SIZE_RATIO_128_KIBIBYTES))
                .to_approximate_f64()
                - 0.32764505119453924)
                .abs()
                < 1e-9
        );
    }

    /// 甲臂另一个换算方向（若用户按请求数给权重 v）：p_B = byte_share_from_request_share(p_N)
    /// 与 p_N = request_share_from_byte_share(p_B) 互为反函数（登记第五节甲臂）。
    #[test]
    fn two_class_conversion_round_trips_in_both_directions() {
        let ratio = RationalNumber::from_integer(DEFAULT_SEQUENTIAL_TO_RANDOM_SIZE_RATIO);
        for &byte_share in &[RationalNumber::new(1, 100), RationalNumber::new(3, 200), RationalNumber::new(3, 5)] {
            let request_share = request_share_from_byte_share(byte_share, ratio);
            assert_eq!(byte_share_from_request_share(request_share, ratio), byte_share);
        }
    }

    /// 第二类 B5：锚点下 m_r(k)。
    #[test]
    fn read_form_a_multiplier_single_size_matches_the_anchor_series() {
        let expected = [1.086, 0.9774, 0.905, 0.8532857142857143, 0.7602];
        for (index, &pages_covered) in [1u64, 2, 3, 4, 7].iter().enumerate() {
            let value = read_form_a_multiplier_single_size(pages_covered).to_approximate_f64();
            assert!((value - expected[index]).abs() < 1e-9, "k={pages_covered}: got {value}, want {}", expected[index]);
        }
    }

    /// 第二类 B6：合成乙条目一、二、三（5.4 正对照）。
    #[test]
    fn synthetic_prior_art_entries_match_the_cited_values() {
        // 条目一：随机 4 KiB 1000 次 + 顺序 1 MiB 1000 次。
        let entry_one = RandomReadMixSource {
            label: "条目一",
            citation: "synthetic",
            random_total_bytes: Some(RationalNumber::from_integer(1000 * 4096)),
            random_total_requests: Some(RationalNumber::from_integer(1000)),
            random_size_distribution: vec![(1, RationalNumber::one())],
            sequential_total_bytes: Some(RationalNumber::from_integer(1000 * 1024 * 1024)),
            sequential_total_requests: Some(RationalNumber::from_integer(1000)),
        };
        assert!((byte_share_of_source(&entry_one).unwrap().to_approximate_f64() - 0.0038910505836575876).abs() < 1e-12);
        assert_eq!(request_share_of_source(&entry_one), Some(RationalNumber::new(1, 2)));

        // 条目二：随机 4 KiB 500 次 + 随机 16 KiB 500 次（只测读法 A 倍数）。
        let entry_two_distribution = vec![(1u64, RationalNumber::new(1, 2)), (4u64, RationalNumber::new(1, 2))];
        let multiplier = read_form_a_multiplier_over_distribution(&entry_two_distribution);
        assert!((multiplier.to_approximate_f64() - 0.9379090909090909).abs() < 1e-12);

        // 条目三：给了字节、缺随机请求数（顺序请求数已知）⇒ p_B 能算，p_N 读不到。
        let entry_three = RandomReadMixSource {
            label: "条目三",
            citation: "synthetic",
            random_total_bytes: Some(RationalNumber::new(1, 100)),
            random_total_requests: None,
            random_size_distribution: Vec::new(),
            sequential_total_bytes: Some(RationalNumber::new(99, 100)),
            sequential_total_requests: Some(RationalNumber::from_integer(1000)),
        };
        assert!(byte_share_of_source(&entry_three).is_some());
        assert_eq!(request_share_of_source(&entry_three), None, "M12 抓的正是这一格：缺请求数要读不到，不是 0");
    }

    /// 第二类 B7：甲的正对照。
    #[test]
    fn weight_arm_positive_controls() {
        let only_random = random_share_mapping_only_random_tier();
        let small_operation = random_share_mapping_small_operation_tiers();
        let equal_weights = [RationalNumber::new(1, 5); 5];
        assert_eq!(weighted_byte_share(equal_weights, only_random), RationalNumber::new(1, 5));
        let byte_share_equal_small_operation = weighted_byte_share(equal_weights, small_operation);
        assert_eq!(byte_share_equal_small_operation, RationalNumber::new(3, 5));
        assert!((request_share_from_byte_share(byte_share_equal_small_operation, RationalNumber::from_integer(256)).to_approximate_f64() - 0.9974025974025974).abs() < 1e-9);
        assert_eq!(weighted_byte_share(epsilon_weights(RationalNumber::new(5, 100)), only_random), RationalNumber::new(1, 80));
        assert_eq!(weighted_byte_share(epsilon_weights(RationalNumber::new(2, 100)), small_operation), RationalNumber::new(3, 200));
        assert_eq!(weighted_byte_share(epsilon_weights(RationalNumber::new(1, 100)), small_operation), RationalNumber::new(3, 400));
    }

    /// 第二类 B8：读放大。
    #[test]
    fn device_amplification_matches_the_cited_values() {
        assert_eq!(device_bytes_no_header_arm(UNIT_BYTES_16, 16317) / (16317 * PAGE_BYTES), 4);
        assert_eq!(device_bytes_no_header_arm(UNIT_BYTES_32, 16317) / (16317 * PAGE_BYTES), 8);
        assert_eq!(device_bytes_no_header_arm(UNIT_BYTES_32, 16317) / (16317 * PAGE_BYTES), 8, "L32p 同 L32：无头、整数倍单元");
        let l32h = RationalNumber::new(
            device_bytes_header_arm(NET_PAYLOAD_BYTES, UNIT_BYTES_32, 16317) as i128,
            (16317 * PAGE_BYTES) as i128,
        );
        assert_eq!(l32h, RationalNumber::new(146912, 16317));
        assert!((l32h.to_approximate_f64() - 9.003615860758718).abs() < 1e-9);
    }

    /// 第二类 B9。
    #[test]
    fn threshold_equals_the_registered_fraction() {
        assert_eq!(RationalNumber::new(1086, 1000) + RationalNumber::new(5, 100), RationalNumber::new(142, 125));
    }

    /// 5.4 判侧正对照。
    #[test]
    fn judgement_side_positive_controls() {
        assert_eq!(classify_against_band(RationalNumber::new(1, 200), band_lower_bound(), band_upper_bound()), ThresholdSide::BelowLowerBound);
        assert_eq!(classify_against_band(RationalNumber::new(1, 100), band_lower_bound(), band_upper_bound()), ThresholdSide::WithinBand);
        assert_eq!(classify_against_band(RationalNumber::new(5, 400), band_lower_bound(), band_upper_bound()), ThresholdSide::WithinBand);
        assert_eq!(classify_against_band(RationalNumber::new(3, 200), band_lower_bound(), band_upper_bound()), ThresholdSide::WithinBand);
        assert_eq!(classify_against_band(RationalNumber::new(15001, 1000000), band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound);
        assert_eq!(classify_against_band(RationalNumber::new(1, 50), band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound);
        assert!(!judgement_one_triggered(RationalNumber::new(1086, 1000)));
        assert!(!judgement_one_triggered(RationalNumber::new(11359, 10000)));
        assert!(judgement_one_triggered(RationalNumber::new(1136, 1000)));
        assert!(judgement_one_triggered(RationalNumber::new(120, 100)));
    }

    /// 5.4 规则真值表 a-f。
    #[test]
    fn combine_rule_truth_table() {
        struct Row {
            judgement_one: bool,
            judgement_five_side: ThresholdSide,
            sigma: RationalNumber,
            first_to_trigger: bool,
            additive: bool,
            only_one: bool,
            only_five: bool,
        }
        let threshold_sum = additive_threshold_primary();
        let rows = [
            Row { judgement_one: true, judgement_five_side: ThresholdSide::BelowLowerBound, sigma: RationalNumber::new(20, 100), first_to_trigger: true, additive: true, only_one: true, only_five: false },
            Row { judgement_one: false, judgement_five_side: ThresholdSide::AboveUpperBound, sigma: RationalNumber::new(10, 100), first_to_trigger: true, additive: false, only_one: false, only_five: true },
            Row { judgement_one: false, judgement_five_side: ThresholdSide::BelowLowerBound, sigma: RationalNumber::new(15, 100), first_to_trigger: false, additive: true, only_one: false, only_five: false },
            Row { judgement_one: false, judgement_five_side: ThresholdSide::WithinBand, sigma: RationalNumber::new(5, 100), first_to_trigger: false, additive: false, only_one: false, only_five: false },
            Row { judgement_one: true, judgement_five_side: ThresholdSide::AboveUpperBound, sigma: RationalNumber::new(30, 100), first_to_trigger: true, additive: true, only_one: true, only_five: true },
        ];
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(first_to_trigger_reopens(row.judgement_one, row.judgement_five_side), row.first_to_trigger, "行 {index}");
            assert_eq!(additive_reopens(row.sigma, RationalNumber::zero(), threshold_sum), row.additive, "行 {index}");
            assert_eq!(only_judgement_one_reopens(row.judgement_one), row.only_one, "行 {index}");
            assert_eq!(only_judgement_five_reopens(row.judgement_five_side), row.only_five, "行 {index}");
        }
        // f 行：C1 = 0.08、C5 = 0.07 ⇒ Σ = 0.15 ⇒ 相加重开。
        assert!(additive_reopens(RationalNumber::new(8, 100), RationalNumber::new(7, 100), threshold_sum));
        assert!(additive_reopens(RationalNumber::new(8, 100), RationalNumber::new(7, 100), threshold_sum));
    }

    /// Q13：读法 A 的值与随机小读占比无关（甲、丙各自在多个占比取样点上逐点相同）。
    #[test]
    fn judgement_one_value_is_independent_of_the_random_share() {
        let distribution = pure_four_kibibyte_distribution();
        let weight_values: Vec<RationalNumber> = [0i128, 1, 5, 20, 50, 80]
            .iter()
            .map(|&epsilon_percent| {
                let epsilon = RationalNumber::new(epsilon_percent, 100);
                let byte_share = weighted_byte_share(epsilon_weights(epsilon), random_share_mapping_only_random_tier());
                instance_judgement_one_value(byte_share, &distribution)
            })
            .collect();
        assert!(weight_values.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(weight_values[0], anchor_multiplier());

        let declared_values: Vec<RationalNumber> = [1i128, 1000, 850, 1_000_000]
            .iter()
            .map(|&reads_per_second| {
                let byte_share = declared_byte_share(RationalNumber::from_integer(reads_per_second));
                instance_judgement_one_value(byte_share, &distribution)
            })
            .collect();
        assert!(declared_values.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(declared_values[0], anchor_multiplier());
    }

    /// 乙 1：fio iometer 定义值，R32 与 R4 两种切点（事实 1／2 的手算，G2 敏感性）。
    #[test]
    fn fio_iometer_source_matches_hand_computation() {
        let source_under_thirty_two_kibibyte_cutoff = fio_iometer_source(32768, "test-thirty-two-kibibyte-cutoff");
        assert_eq!(byte_share_of_source(&source_under_thirty_two_kibibyte_cutoff), Some(RationalNumber::new(85, 277)));
        assert_eq!(request_share_of_source(&source_under_thirty_two_kibibyte_cutoff), Some(RationalNumber::new(43, 50)));
        let source_under_four_kibibyte_cutoff = fio_iometer_source(4097, "test-four-kibibyte-cutoff");
        assert_eq!(byte_share_of_source(&source_under_four_kibibyte_cutoff), Some(RationalNumber::new(65, 277)));
        assert_eq!(request_share_of_source(&source_under_four_kibibyte_cutoff), Some(RationalNumber::new(4, 5)));
        // G2：两种切点都在同一侧（越过），不触发 F7。
        assert_eq!(
            classify_against_band(byte_share_of_source(&source_under_thirty_two_kibibyte_cutoff).unwrap(), band_lower_bound(), band_upper_bound()),
            classify_against_band(byte_share_of_source(&source_under_four_kibibyte_cutoff).unwrap(), band_lower_bound(), band_upper_bound())
        );
    }

    /// V5：E152 归档产物的行数与末行完成标记。
    #[test]
    fn e152_input_files_match_the_archived_shape() {
        let single_disk_raw = include_str!("../../../results/e160-e152-input-single-2026-09-15.out");
        let mirror_raw = include_str!("../../../results/e160-e152-input-mirror-2026-09-15.out");
        assert_eq!(single_disk_raw.lines().count(), 890);
        assert_eq!(mirror_raw.lines().count(), 941);
        assert_eq!(single_disk_raw.lines().last(), Some("E7RESULT name=summary_done emitted=174"));
        assert_eq!(mirror_raw.lines().last(), Some("E7RESULT name=summary_done emitted=174"));
        let single_rows = parse_e152_rows(single_disk_raw);
        let mirror_rows = parse_e152_rows(mirror_raw);
        assert_eq!(distinct_configurations(&single_rows, "read_random_4k").len(), 7);
        assert_eq!(distinct_configurations(&mirror_rows, "read_random_4k").len(), 7);
    }

    /// 丙-实测：14 个配置全部落在「越过」这一侧（钉两个极值）。
    #[test]
    fn measured_declared_workload_arm_all_configurations_cross_the_upper_band() {
        let single_disk_raw = include_str!("../../../results/e160-e152-input-single-2026-09-15.out");
        let mirror_raw = include_str!("../../../results/e160-e152-input-mirror-2026-09-15.out");
        let single_rows = parse_e152_rows(single_disk_raw);
        let mirror_rows = parse_e152_rows(mirror_raw);
        let mut count = 0;
        let mut byte_shares = Vec::new();
        for rows in [&single_rows, &mirror_rows] {
            for configuration in distinct_configurations(rows, "read_random_4k") {
                let random_median = median_read_kibibytes(rows, &configuration, "read_random_4k");
                let sequential_median = median_read_kibibytes(rows, &configuration, "read_sequential_large");
                assert!(measured_sequential_matches_declared_assumption(sequential_median), "{configuration}");
                let byte_share = declared_byte_share(measured_random_reads_per_second(random_median));
                assert_eq!(classify_against_band(byte_share, band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound, "{configuration}");
                byte_shares.push(byte_share);
                count += 1;
            }
        }
        assert_eq!(count, 14);
        byte_shares.sort();
        assert!((byte_shares.first().unwrap().to_approximate_f64() - 0.05620505823022246).abs() < 1e-6, "zfs-mirror 最小");
        assert!((byte_shares.last().unwrap().to_approximate_f64() - 0.5754249390717199).abs() < 1e-6, "raw 单盘最大");
    }

    /// 第八节判别力自证：把带上沿从 1.5% 挪到 1.7%，只翻一格；把 θ1 从 1.136 挪到 1.08，只翻一格。
    #[test]
    fn geometry_sensitivity_self_proof() {
        let alternate_upper_bound = RationalNumber::new(17, 1000);
        let byte_share_at_850 = declared_byte_share(RationalNumber::from_integer(850));
        let byte_share_at_1000 = declared_byte_share(RationalNumber::from_integer(1000));
        assert_eq!(classify_against_band(byte_share_at_850, band_lower_bound(), band_upper_bound()), ThresholdSide::AboveUpperBound);
        assert_eq!(classify_against_band(byte_share_at_850, band_lower_bound(), alternate_upper_bound), ThresholdSide::WithinBand);
        assert_eq!(classify_against_band(byte_share_at_1000, band_lower_bound(), alternate_upper_bound), ThresholdSide::AboveUpperBound);

        let alternate_threshold = RationalNumber::new(108, 100);
        assert!(!judgement_one_triggered(read_form_a_multiplier_single_size(1)));
        assert!(read_form_a_multiplier_single_size(1) >= alternate_threshold);
        assert!(read_form_a_multiplier_single_size(2) < alternate_threshold);
    }

    /// M2：判据 5 要按字节口径判，不许把请求次数口径用反（甲 ω=1.0% 时 p_B 带内、p_N 越过）。
    #[test]
    fn weight_instance_judgement_five_side_uses_byte_share_not_request_share() {
        let byte_share = RationalNumber::new(1, 100);
        let request_share = request_share_from_byte_share(byte_share, RationalNumber::from_integer(DEFAULT_SEQUENTIAL_TO_RANDOM_SIZE_RATIO));
        assert!(request_share > band_upper_bound(), "p_N 在这一点上确实越过，才能把用反的错误暴露出来");
        assert_eq!(weight_instance_judgement_five_side(byte_share, request_share), ThresholdSide::WithinBand);
    }

    /// G1：下沿也算越过。
    #[test]
    fn lower_bound_as_crossed_variant() {
        assert!(!classify_against_band_treating_lower_bound_as_crossed(RationalNumber::new(1, 200), band_lower_bound()));
        assert!(classify_against_band_treating_lower_bound_as_crossed(RationalNumber::new(1, 100), band_lower_bound()));
        assert!(classify_against_band_treating_lower_bound_as_crossed(RationalNumber::new(3, 200), band_lower_bound()));
    }

    /// 阳性对照跑遍每一条来源臂：甲、乙、丙都至少产出一个可判的实例。
    #[test]
    fn positive_control_runs_on_every_source_arm() {
        let weight_byte_share = weighted_byte_share([RationalNumber::new(1, 5); 5], random_share_mapping_small_operation_tiers());
        assert!(weight_byte_share > RationalNumber::zero());
        for source in prior_art_sources() {
            // 至少一条(fio 定义值)要算得出来，不能全部读不到（F4 的判据）。
            let _ = byte_share_of_source(&source);
        }
        assert!(prior_art_sources().iter().any(|source| byte_share_of_source(source).is_some()), "F4：不能整列读不到");
        let declared_byte_share_value = declared_byte_share(RationalNumber::from_integer(1000));
        assert!(declared_byte_share_value > RationalNumber::zero());
    }

    /// RationalNumber 基本算术与比较。
    #[test]
    fn rational_number_arithmetic_is_reduced_and_ordered() {
        assert_eq!(RationalNumber::new(2, 4), RationalNumber::new(1, 2));
        assert_eq!(RationalNumber::new(1, -2), RationalNumber::new(-1, 2));
        assert!(RationalNumber::new(1, 2) < RationalNumber::new(2, 3));
        assert_eq!(RationalNumber::new(1, 3) + RationalNumber::new(1, 6), RationalNumber::new(1, 2));
        assert_eq!(RationalNumber::from_integer(3).maximum(RationalNumber::from_integer(5)), RationalNumber::from_integer(5));
    }
}
