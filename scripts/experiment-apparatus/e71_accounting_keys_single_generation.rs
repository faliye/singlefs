//! E71（记账 key 的条目数与宽度）第 1 次重跑（r1）：判据 2 按「每条根只装一代、只写不删」重算。
//!
//! 跑前登记：`research/prompts/e71-r1-prereg.md`（问题单 `research/prompts/c693-forks.md` 的 Q3，E71 那一半）。
//! 原装置 `scripts/experiment-apparatus/e71_accounting_keys.rs` 一个字不动；这份是独立的新装置，不 `use` 原装置，也不 `use` `crates/` 下任何一个包。
//!
//! ## 这一次唯一改的口径（登记「读法」第 1 条）
//!
//! - 改前（旧口径）：一次发布的记账写字节 = `2 × 一代条目数 × 条目宽`（原装置 `checkpoint_bytes`：一写一删）。
//! - 改后（新口径）：`1 × 一代条目数 × 条目宽`。D5（快照 / 空间记账机制） 已定项 2 射程逐字：
//!   「每次发布把这一代的行全写一遍，写的行数是实付值，不是上界」；定案逐字「每条根的记账树只装这条根那一代的行，不留多代」（K = 1）。
//!
//! 两个口径各是 [`WriteAccounting`] 的一个成员，同一个函数 [`publish_write_bytes`] 按成员取写次数系数；
//! 旧口径那一列由同一套式子现算，拿去与原产物 `e71-accounting-keys-2026-09-01.out` 的 `name=ckpt_budget` 行逐字比（登记 V1）。
//!
//! ## 照原装置不动的（登记「读法」第 3 条）
//!
//! 统计量表九项（2026-09-01 那天的 D5 已定项 4：带设备维第 1、2、3、5 共 4 项；窄读只有第 8、9 两项按树分；宽读九项都按树分）；
//! 三档宽度 narrow（key 14 + value 8）、middle（22 + 8，原产物写 `mid`）、wide（34 + 16）；树数 1 / 8 / 64 / 1024 / 8192、设备数 1 / 8 / 64；
//! 门槛 `T_dirty` 的 1% = 21 474 836 字节（D16（发布语义） 已定项 5「`T_dirty` = 2 GiB」），过界 = 严格大于门槛。
//! 这张九项表不是今天的已定项 4（今天十二项），结论只能写成「按 2026-09-01 那张九项表、新行数口径」。
//!
//! ## 加的（登记第五节、第八节）
//!
//! - 宽读臂补进判据 2，标附带（原判据 2 只扫上界臂与窄读臂）。
//! - 几何敏感性点 S2：条目宽 34 = key 22 + value 8 + seq 4，抄自 `crates/singlefs-format/src/lib.rs` 的
//!   `ACCOUNTING_ENTRY_BYTES`（与 `ACCOUNTING_KEY_BYTES` 两个常量）；这里写本地常量，不 `use`。
//!
//! 登记的附带 2（撑爆点）、附带 3（今天实现的行集 3 + 6d 行 × 34 字节）标「够判后不跑」，这份装置不算也不打；
//! 过界的是哪几格由 `name=over_budget_count` 行逐格列出。
//!
//! ## 它答不了的
//!
//! 纯算术：没有记账树、没有 btree、没有 write buffer，文件操作 0 处；不建模 inode 号水位的台阶行（随根留着、不随每次发布重写）；
//! 不答今天十二项统计量表下的数；数的是条目字节，不数脏节点（那是 E97（记账与分配记录的条目编码） 判据 10）。
//! 单线程跑完：每一格是同一组参数上的纯函数，整趟不到一秒，不值得切片（登记第十四节）。

use e7_index_bench::Emitter;

/// 2026-09-01 那天的 D5 已定项 4：统计量取九个。
const STATISTIC_COUNT: usize = 9;
/// D16 已定项 5：`T_dirty` = 2 GiB。
const DIRTY_BYTES_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;
/// 原判据 2 的预算线：`T_dirty` 的 1%（整除）。
const ACCOUNTING_WRITE_BUDGET_BYTES: u64 = DIRTY_BYTES_THRESHOLD / 100;
/// 今天实现的记账条目里 seq 占的字节（`crates/singlefs-format/src/lib.rs` 的 `ACCOUNTING_ENTRY_BYTES` = key 22 + value 8 + seq 4）。
const CURRENT_IMPLEMENTATION_SEQUENCE_BYTES: u64 = 4;
/// 原装置判据 2 扫的树数与设备数（`e71_accounting_keys.rs` 判据 2 那一段的两层循环）。
const TREE_COUNTS: [u64; 5] = [1, 8, 64, 1024, 8192];
const DEVICE_COUNTS: [u64; 3] = [1, 8, 64];
/// 判决格（登记 Q-E71-1）：上界臂、middle 宽度、树数 1024、设备数 64。
const JUDGEMENT_CELL_TREE_COUNT: u64 = 1024;
const JUDGEMENT_CELL_DEVICE_COUNT: u64 = 64;

fn statistic_count() -> u64 {
    u64::try_from(STATISTIC_COUNT).expect("九个统计量装得进 u64")
}

/// 九个统计量各自用不用得上两个维度。次序与 2026-09-01 那天 D5 已定项 4 的表逐行对应（同原装置）。
struct Statistic {
    name: &'static str,
    uses_device_dimension: bool,
    /// 窄读：只有「每树独占 / 每树共享」按树分。
    uses_tree_dimension_in_narrow_reading: bool,
}

const TABLE: [Statistic; STATISTIC_COUNT] = [
    Statistic { name: "allocated", uses_device_dimension: true, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "free", uses_device_dimension: true, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "unreclaimable", uses_device_dimension: true, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "pending_delete", uses_device_dimension: false, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "defer_pending", uses_device_dimension: true, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "reserved", uses_device_dimension: false, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "extension_quota_used", uses_device_dimension: false, uses_tree_dimension_in_narrow_reading: false },
    Statistic { name: "per_tree_exclusive", uses_device_dimension: false, uses_tree_dimension_in_narrow_reading: true },
    Statistic { name: "per_tree_shared", uses_device_dimension: false, uses_tree_dimension_in_narrow_reading: true },
];

/// 三条臂（定义同原装置）。没有 `_ =>` 通配臂：加第四条读法时这里编译不过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 维度用满：九项一律乘树数与设备数。
    UpperBound,
    /// 窄读：只有第 8、9 项按树分；设备维按那张表。
    PerStatisticNarrow,
    /// 宽读：九项都按树分；设备维同上。判据 2 里是附带（登记 Q-E71-附带 1）。
    PerStatisticWide,
}

const ARMS: [Arm; 3] = [Arm::UpperBound, Arm::PerStatisticNarrow, Arm::PerStatisticWide];

impl Arm {
    fn tag(self) -> &'static str {
        match self {
            Arm::UpperBound => "upper_bound",
            Arm::PerStatisticNarrow => "per_statistic_narrow",
            Arm::PerStatisticWide => "per_statistic_wide",
        }
    }

    /// 原产物 `name=ckpt_budget` 行里这条臂的写法：旧口径那一列要与原产物逐字相同，只能照它的写法打。
    fn original_product_tag(self) -> &'static str {
        match self {
            Arm::UpperBound => "upper_bound",
            Arm::PerStatisticNarrow => "per_stat_narrow",
            Arm::PerStatisticWide => "per_stat_wide",
        }
    }

    /// 这条臂在判据 2 里的身份：上界与窄读是原判据 2 的两条臂，宽读是这一次补的附带臂。
    fn criterion_two_role(self) -> &'static str {
        match self {
            Arm::UpperBound | Arm::PerStatisticNarrow => "registered",
            Arm::PerStatisticWide => "incidental_wide_reading",
        }
    }
}

/// 设备维进不进式子：阳性对照（登记第五节丙）把它去掉。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DeviceDimension {
    Kept,
    Dropped,
}

/// 一个统计量在给定设备数下占几行（只看设备维）。
fn device_factor(statistic_uses_device_dimension: bool, device_dimension: DeviceDimension, device_count: u64) -> u64 {
    match (device_dimension, statistic_uses_device_dimension) {
        (DeviceDimension::Kept, true) => device_count,
        (DeviceDimension::Kept, false) | (DeviceDimension::Dropped, _) => 1,
    }
}

/// 一代里的条目数（key 组合数）。
fn entries_per_generation(arm: Arm, tree_count: u64, device_count: u64, device_dimension: DeviceDimension) -> u64 {
    match arm {
        Arm::UpperBound => statistic_count() * tree_count * device_factor(true, device_dimension, device_count),
        Arm::PerStatisticNarrow => TABLE
            .iter()
            .map(|statistic| {
                let tree_factor = if statistic.uses_tree_dimension_in_narrow_reading { tree_count } else { 1 };
                tree_factor * device_factor(statistic.uses_device_dimension, device_dimension, device_count)
            })
            .sum(),
        Arm::PerStatisticWide => TABLE
            .iter()
            .map(|statistic| tree_count * device_factor(statistic.uses_device_dimension, device_dimension, device_count))
            .sum(),
    }
}

/// 一次发布写记账行的口径。没有 `_ =>` 通配臂：加第三种口径时这里编译不过。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteAccounting {
    /// 旧口径（原装置判据 2）：记账树留 K 代，一次发布把一代的每一行写一次、删一次。
    OneWriteOneDeleteAcrossKeptGenerations,
    /// 新口径（D5 已定项 2 现行定案）：每条根的记账树只装一代（K = 1），一次发布把这一代的每一行写一次、不删。
    SingleGenerationWriteOnly,
}

impl WriteAccounting {
    fn write_count_factor(self) -> u64 {
        match self {
            WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations => 2,
            WriteAccounting::SingleGenerationWriteOnly => 1,
        }
    }

    /// 一条根的记账树里留几代。旧口径的 K 只进总条目数（判据 1、3，这一次不重跑），不进判据 2 的写字节，所以这里不给旧口径的值。
    fn kept_generations_in_one_root(self) -> Option<u64> {
        match self {
            WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations => None,
            WriteAccounting::SingleGenerationWriteOnly => Some(1),
        }
    }
}

/// 一条记账条目各段的宽（字节）。原三档没有 seq 段（2026-09-01 那天 seq 还没进 value）；S2 点带 seq 4。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EntryWidth {
    tag: &'static str,
    /// 原产物 `name=ckpt_budget` 行里这一档的写法；原产物里没有的那一档（S2）不打旧格式行。
    original_product_tag: Option<&'static str>,
    key_bytes: u64,
    value_bytes: u64,
    sequence_bytes: u64,
}

impl EntryWidth {
    fn entry_bytes(self) -> u64 {
        self.key_bytes + self.value_bytes + self.sequence_bytes
    }

    /// 这一档在判据 2 里的身份：原三档是原登记的取样点，S2 是这一次加的几何敏感性点。
    fn criterion_two_role(self) -> &'static str {
        match self.original_product_tag {
            Some(_) => "registered_width",
            None => "geometry_sensitivity_width",
        }
    }
}

const NARROW_WIDTH: EntryWidth = EntryWidth { tag: "narrow", original_product_tag: Some("narrow"), key_bytes: 14, value_bytes: 8, sequence_bytes: 0 };
const MIDDLE_WIDTH: EntryWidth = EntryWidth { tag: "middle", original_product_tag: Some("mid"), key_bytes: 22, value_bytes: 8, sequence_bytes: 0 };
const WIDE_WIDTH: EntryWidth = EntryWidth { tag: "wide", original_product_tag: Some("wide"), key_bytes: 34, value_bytes: 16, sequence_bytes: 0 };
/// S2：今天实现的条目宽（key 22 + value 8 + seq 4 = 34）。
const CURRENT_IMPLEMENTATION_WIDTH: EntryWidth = EntryWidth {
    tag: "current_implementation",
    original_product_tag: None,
    key_bytes: 22,
    value_bytes: 8,
    sequence_bytes: CURRENT_IMPLEMENTATION_SEQUENCE_BYTES,
};
/// 原登记的三档（原装置判据 2 的次序）。
const REGISTERED_WIDTHS: [EntryWidth; 3] = [NARROW_WIDTH, MIDDLE_WIDTH, WIDE_WIDTH];
/// 产物里每格新旧并列报的全部宽度：原三档加 S2。
const ALL_WIDTHS: [EntryWidth; 4] = [NARROW_WIDTH, MIDDLE_WIDTH, WIDE_WIDTH, CURRENT_IMPLEMENTATION_WIDTH];

/// 一次发布的记账写字节 = 写次数系数 × 一代条目数 × 条目宽。
fn publish_write_bytes(write_accounting: WriteAccounting, arm: Arm, tree_count: u64, device_count: u64, width: EntryWidth, device_dimension: DeviceDimension) -> u64 {
    write_accounting.write_count_factor() * entries_per_generation(arm, tree_count, device_count, device_dimension) * width.entry_bytes()
}

/// 原判据 2：严格大于门槛判「过界」。门槛当参数传进来，判别力自证（登记第八节）挪门槛用。
fn is_over_budget(publish_write_bytes: u64, budget_bytes: u64) -> bool {
    publish_write_bytes > budget_bytes
}

/// 一格：一档宽度 × 一条臂 × 一组 (树数, 设备数)，旧、新两口径并列。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PublishBudgetCell {
    width: EntryWidth,
    arm: Arm,
    tree_count: u64,
    device_count: u64,
    entries_per_generation: u64,
    old_bytes: u64,
    new_bytes: u64,
}

impl PublishBudgetCell {
    fn at(width: EntryWidth, arm: Arm, tree_count: u64, device_count: u64) -> PublishBudgetCell {
        PublishBudgetCell {
            width,
            arm,
            tree_count,
            device_count,
            entries_per_generation: entries_per_generation(arm, tree_count, device_count, DeviceDimension::Kept),
            old_bytes: publish_write_bytes(WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations, arm, tree_count, device_count, width, DeviceDimension::Kept),
            new_bytes: publish_write_bytes(WriteAccounting::SingleGenerationWriteOnly, arm, tree_count, device_count, width, DeviceDimension::Kept),
        }
    }

    fn is_old_over_budget(self) -> bool {
        is_over_budget(self.old_bytes, ACCOUNTING_WRITE_BUDGET_BYTES)
    }

    fn is_new_over_budget(self) -> bool {
        is_over_budget(self.new_bytes, ACCOUNTING_WRITE_BUDGET_BYTES)
    }

    /// 新口径字节占门槛的千分比（整除，登记 Q-E71-4）。
    fn new_bytes_permille_of_budget(self) -> u64 {
        self.new_bytes * 1000 / ACCOUNTING_WRITE_BUDGET_BYTES
    }
}

/// 全部格：宽度（原三档加 S2）× 树数 × 设备数 × 三条臂，次序同原装置判据 2（宽度 → 树数 → 设备数 → 臂）。
fn all_publish_budget_cells() -> Vec<PublishBudgetCell> {
    let mut cells = Vec::with_capacity(ALL_WIDTHS.len() * TREE_COUNTS.len() * DEVICE_COUNTS.len() * ARMS.len());
    for width in ALL_WIDTHS {
        for tree_count in TREE_COUNTS {
            for device_count in DEVICE_COUNTS {
                for arm in ARMS {
                    cells.push(PublishBudgetCell::at(width, arm, tree_count, device_count));
                }
            }
        }
    }
    cells
}

/// 旧口径那一列按原产物的格式打出来的一行（不带 `E7RESULT ` 前缀）。字段名 `key_w`、`val_w` 与臂、宽度的写法照原产物，
/// 为的是能拿 `diff` 与原产物的 `name=ckpt_budget` 行逐字比（登记 V1），不是这一次自己起的名字。
fn original_format_checkpoint_budget_line(cell: PublishBudgetCell) -> Option<String> {
    let original_width_tag = cell.width.original_product_tag?;
    match cell.arm {
        Arm::UpperBound | Arm::PerStatisticNarrow => Some(format!(
            "name=ckpt_budget width={original_width_tag} key_w={} val_w={} arm={} t={} d={} bytes={} budget={ACCOUNTING_WRITE_BUDGET_BYTES} over={}",
            cell.width.key_bytes,
            cell.width.value_bytes,
            cell.arm.original_product_tag(),
            cell.tree_count,
            cell.device_count,
            cell.old_bytes,
            u8::from(cell.is_old_over_budget())
        )),
        // 原判据 2 没有宽读臂，原产物里没有这一格。
        Arm::PerStatisticWide => None,
    }
}

/// 每格新旧并列的一行（登记 Q-E71-4、Q-E71-附带 1、第八节 S2）。
fn publish_budget_line(cell: PublishBudgetCell) -> String {
    format!(
        "name=publish_budget width={} width_role={} key_bytes={} value_bytes={} sequence_bytes={} entry_bytes={} arm={} arm_role={} \
         trees={} devices={} entries_per_generation={} old_bytes={} old_over={} new_bytes={} new_over={} new_permille_of_budget={} budget={ACCOUNTING_WRITE_BUDGET_BYTES}",
        cell.width.tag,
        cell.width.criterion_two_role(),
        cell.width.key_bytes,
        cell.width.value_bytes,
        cell.width.sequence_bytes,
        cell.width.entry_bytes(),
        cell.arm.tag(),
        cell.arm.criterion_two_role(),
        cell.tree_count,
        cell.device_count,
        cell.entries_per_generation,
        cell.old_bytes,
        u8::from(cell.is_old_over_budget()),
        cell.new_bytes,
        u8::from(cell.is_new_over_budget()),
        cell.new_bytes_permille_of_budget()
    )
}

/// 一组格里新口径过界的那几格，写成 `宽度:树数:设备数` 逗号分隔；一格都没有写 `none`。
fn over_budget_cell_list(cells: &[PublishBudgetCell]) -> String {
    let listed: Vec<String> = cells
        .iter()
        .filter(|cell| cell.is_new_over_budget())
        .map(|cell| format!("{}:{}:{}", cell.width.tag, cell.tree_count, cell.device_count))
        .collect();
    if listed.is_empty() { "none".to_string() } else { listed.join(",") }
}

/// 阳性对照丙（登记第五节）：树数 64、设备数 8、middle 宽度、新口径，去掉设备维前后的写字节。
struct DropDeviceControl {
    with_device_bytes: u64,
    without_device_bytes: u64,
}

impl DropDeviceControl {
    const TREE_COUNT: u64 = 64;
    const DEVICE_COUNT: u64 = 8;

    fn of(arm: Arm) -> DropDeviceControl {
        let bytes_with = |device_dimension| {
            publish_write_bytes(WriteAccounting::SingleGenerationWriteOnly, arm, Self::TREE_COUNT, Self::DEVICE_COUNT, MIDDLE_WIDTH, device_dimension)
        };
        DropDeviceControl { with_device_bytes: bytes_with(DeviceDimension::Kept), without_device_bytes: bytes_with(DeviceDimension::Dropped) }
    }

    fn drops_exactly_by_device_count(&self) -> bool {
        self.with_device_bytes == Self::DEVICE_COUNT * self.without_device_bytes
    }

    /// 上界臂必须恰好降设备数倍；窄读、宽读两臂严格下降且降得少于设备数倍（九项里只有 4 项带设备维）。
    fn holds_for(&self, arm: Arm) -> bool {
        match arm {
            Arm::UpperBound => self.drops_exactly_by_device_count(),
            Arm::PerStatisticNarrow | Arm::PerStatisticWide => {
                self.without_device_bytes < self.with_device_bytes && self.with_device_bytes < Self::DEVICE_COUNT * self.without_device_bytes
            }
        }
    }
}

fn optional_number(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

fn boolean_word(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn main() {
    let mut emitter = Emitter::new();
    let mut emit = |body: String| println!("{}", emitter.emit_raw(&body));
    emit(format!(
        "name=config statistics={STATISTIC_COUNT} t_dirty={DIRTY_BYTES_THRESHOLD} budget={ACCOUNTING_WRITE_BUDGET_BYTES} \
         old_write_count_factor={} new_write_count_factor={} new_kept_generations_in_one_root={} model=arithmetic file_operations=0 \
         threads_sweep=1 threads_controls=1",
        WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations.write_count_factor(),
        WriteAccounting::SingleGenerationWriteOnly.write_count_factor(),
        optional_number(WriteAccounting::SingleGenerationWriteOnly.kept_generations_in_one_root())
    ));
    for statistic in TABLE.iter() {
        emit(format!(
            "name=statistic_dimensions statistic={} uses_device_dimension={} uses_tree_dimension_in_narrow_reading={}",
            statistic.name,
            u8::from(statistic.uses_device_dimension),
            u8::from(statistic.uses_tree_dimension_in_narrow_reading)
        ));
    }

    let cells = all_publish_budget_cells();

    // ── 每格新旧并列（Q-E71-4、Q-E71-附带 1、S2）──────────────────────────
    for cell in &cells {
        emit(publish_budget_line(*cell));
    }

    // ── 旧口径那一列照原产物格式重打，拿去与原产物逐字比（V1）──────────────
    for cell in &cells {
        if let Some(line) = original_format_checkpoint_budget_line(*cell) {
            emit(line);
        }
    }

    // ── 阳性对照甲：每条臂、每档宽度、每一格，旧口径字节恰好等于新口径字节 × 2 ──
    let mut write_factor_control_holds = true;
    for arm in ARMS {
        let cells_of_this_arm: Vec<&PublishBudgetCell> = cells.iter().filter(|cell| cell.arm == arm).collect();
        let old_equals_twice_new = cells_of_this_arm.iter().filter(|cell| cell.old_bytes == 2 * cell.new_bytes).count();
        write_factor_control_holds &= old_equals_twice_new == cells_of_this_arm.len();
        emit(format!(
            "name=positive_control_write_factor arm={} cells={} old_equals_twice_new={old_equals_twice_new}",
            arm.tag(),
            cells_of_this_arm.len()
        ));
    }

    // ── 阳性对照丙：去掉设备维 ─────────────────────────────────────────
    let mut drop_device_control_holds = true;
    for arm in ARMS {
        let control = DropDeviceControl::of(arm);
        drop_device_control_holds &= control.holds_for(arm);
        emit(format!(
            "name=positive_control_drop_device arm={} trees={} devices={} width={} with_device_new_bytes={} without_device_new_bytes={} \
             ratio_x1000={} exactly_by_device_count={} holds={}",
            arm.tag(),
            DropDeviceControl::TREE_COUNT,
            DropDeviceControl::DEVICE_COUNT,
            MIDDLE_WIDTH.tag,
            control.with_device_bytes,
            control.without_device_bytes,
            control.with_device_bytes * 1000 / control.without_device_bytes,
            u8::from(control.drops_exactly_by_device_count()),
            u8::from(control.holds_for(arm))
        ));
    }

    // ── 三条臂的次序：每一组 (宽度, 树数, 设备数) 上界 ≥ 宽读 ≥ 窄读 ──────────────
    let groups: Vec<&[PublishBudgetCell]> = cells.chunks(ARMS.len()).collect();
    let ordered_groups = groups
        .iter()
        .filter(|group| {
            let new_bytes_of = |arm: Arm| group.iter().find(|cell| cell.arm == arm).expect("每组三条臂各一格").new_bytes;
            new_bytes_of(Arm::UpperBound) >= new_bytes_of(Arm::PerStatisticWide) && new_bytes_of(Arm::PerStatisticWide) >= new_bytes_of(Arm::PerStatisticNarrow)
        })
        .count();
    emit(format!("name=arm_ordering groups={} upper_at_least_wide_at_least_narrow={ordered_groups}", groups.len()));

    // ── Q-E71-1 判决格 ────────────────────────────────────────────────
    let judgement_cell = PublishBudgetCell::at(MIDDLE_WIDTH, Arm::UpperBound, JUDGEMENT_CELL_TREE_COUNT, JUDGEMENT_CELL_DEVICE_COUNT);
    emit(format!(
        "name=judgement_cell arm={} width={} trees={} devices={} old_bytes={} old_over={} new_bytes={} new_over={} budget={ACCOUNTING_WRITE_BUDGET_BYTES}",
        judgement_cell.arm.tag(),
        judgement_cell.width.tag,
        judgement_cell.tree_count,
        judgement_cell.device_count,
        judgement_cell.old_bytes,
        u8::from(judgement_cell.is_old_over_budget()),
        judgement_cell.new_bytes,
        u8::from(judgement_cell.is_new_over_budget())
    ));

    // ── Q-E71-2、Q-E71-3、Q-E71-附带 1：每条臂过界格数，原三档 45 格与 S2 15 格分开数 ──
    for arm in ARMS {
        for (width_group, widths) in [("registered_widths", &REGISTERED_WIDTHS[..]), ("current_implementation_width", &[CURRENT_IMPLEMENTATION_WIDTH][..])] {
            let group_cells: Vec<PublishBudgetCell> = cells.iter().copied().filter(|cell| cell.arm == arm && widths.contains(&cell.width)).collect();
            emit(format!(
                "name=over_budget_count arm={} widths={width_group} cells={} old_over={} new_over={} new_over_cells={}",
                arm.tag(),
                group_cells.len(),
                group_cells.iter().filter(|cell| cell.is_old_over_budget()).count(),
                group_cells.iter().filter(|cell| cell.is_new_over_budget()).count(),
                over_budget_cell_list(&group_cells)
            ));
        }
    }

    // ── 第八节几何敏感性：判决格在四档宽度上各判一次，另加两个 树数 × 设备数 邻格 ──
    for (point, registered_label, width) in [
        ("narrower_entry_width", "S1", NARROW_WIDTH),
        ("main_entry_width", "main", MIDDLE_WIDTH),
        ("current_implementation_entry_width", "S2", CURRENT_IMPLEMENTATION_WIDTH),
        ("wider_entry_width", "S3", WIDE_WIDTH),
    ] {
        let cell = PublishBudgetCell::at(width, Arm::UpperBound, JUDGEMENT_CELL_TREE_COUNT, JUDGEMENT_CELL_DEVICE_COUNT);
        emit(format!(
            "name=geometry_sensitivity point={point} registered_label={registered_label} arm={} width={} entry_bytes={} trees={} devices={} \
             new_bytes={} new_over={} same_new_verdict_as_main_point={}",
            cell.arm.tag(),
            width.tag,
            width.entry_bytes(),
            cell.tree_count,
            cell.device_count,
            cell.new_bytes,
            u8::from(cell.is_new_over_budget()),
            u8::from(cell.is_new_over_budget() == judgement_cell.is_new_over_budget())
        ));
    }
    for (point, tree_count, device_count) in [("neighbor_fewer_devices", 1024u64, 8u64), ("neighbor_more_trees", 8192, 64)] {
        let cell = PublishBudgetCell::at(MIDDLE_WIDTH, Arm::UpperBound, tree_count, device_count);
        emit(format!(
            "name=geometry_sensitivity point={point} registered_label=trees_times_devices_neighbor arm={} width={} entry_bytes={} trees={tree_count} devices={device_count} \
             new_bytes={} new_over={} same_new_verdict_as_main_point={}",
            cell.arm.tag(),
            MIDDLE_WIDTH.tag,
            MIDDLE_WIDTH.entry_bytes(),
            cell.new_bytes,
            u8::from(cell.is_new_over_budget()),
            u8::from(cell.is_new_over_budget() == judgement_cell.is_new_over_budget())
        ));
    }

    // ── 这一趟测量自己站不站得住（阳性对照甲、丙与三臂次序），各报各的 ──────────
    emit(format!(
        "name=verdict cells={} positive_control_write_factor_holds={} positive_control_drop_device_holds={} arm_ordering_holds={}",
        cells.len(),
        boolean_word(write_factor_control_holds),
        boolean_word(drop_device_control_holds),
        boolean_word(ordered_groups == groups.len())
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 登记甲 1：门槛是 `T_dirty` 的 1%，`T_dirty` = 2 GiB。
    #[test]
    fn budget_line_is_one_percent_of_two_gibibytes() {
        assert_eq!(DIRTY_BYTES_THRESHOLD, 2_147_483_648);
        assert_eq!(ACCOUNTING_WRITE_BUDGET_BYTES, 21_474_836);
        assert_eq!(100 * ACCOUNTING_WRITE_BUDGET_BYTES + 48, DIRTY_BYTES_THRESHOLD, "整除丢掉的余数是 48");
    }

    /// 登记甲 2：新口径写次数系数 1、一条根只装一代；旧口径系数 2。
    #[test]
    fn single_generation_writes_each_row_once_and_the_old_reading_writes_twice() {
        assert_eq!(WriteAccounting::SingleGenerationWriteOnly.write_count_factor(), 1);
        assert_eq!(WriteAccounting::SingleGenerationWriteOnly.kept_generations_in_one_root(), Some(1));
        assert_eq!(WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations.write_count_factor(), 2);
    }

    /// 九项表：4 项带设备维、窄读 2 项按树分（同原装置）。
    #[test]
    fn statistic_table_has_nine_rows_four_with_devices_two_per_tree_in_narrow_reading() {
        assert_eq!(statistic_count(), 9);
        assert_eq!(TABLE.iter().filter(|statistic| statistic.uses_device_dimension).count(), 4);
        assert_eq!(TABLE.iter().filter(|statistic| statistic.uses_tree_dimension_in_narrow_reading).count(), 2);
    }

    /// 一代条目数，三条臂逐格手算（登记第七节乙类后那两条单元算术，另加原装置单测的 树数 64 / 设备数 8 一格）。
    #[test]
    fn entries_per_generation_match_hand_arithmetic_on_three_arms() {
        assert_eq!(entries_per_generation(Arm::UpperBound, 1024, 64, DeviceDimension::Kept), 9 * 65_536);
        assert_eq!(entries_per_generation(Arm::UpperBound, 1024, 64, DeviceDimension::Kept), 589_824);
        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 1024, 64, DeviceDimension::Kept), 4 * 64 + 2 * 1024 + 3);
        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 1024, 64, DeviceDimension::Kept), 2307);
        assert_eq!(entries_per_generation(Arm::PerStatisticWide, 1024, 64, DeviceDimension::Kept), 4 * 65_536 + 5 * 1024);
        assert_eq!(entries_per_generation(Arm::PerStatisticWide, 1024, 64, DeviceDimension::Kept), 267_264);
        assert_eq!(entries_per_generation(Arm::UpperBound, 64, 8, DeviceDimension::Kept), 4608);
        assert_eq!(entries_per_generation(Arm::PerStatisticNarrow, 64, 8, DeviceDimension::Kept), 163);
        assert_eq!(entries_per_generation(Arm::PerStatisticWide, 64, 8, DeviceDimension::Kept), 2368);
    }

    /// 旧口径路径的第三条独立出处：原装置单测 `criterion2_upper_bound_blows_the_budget` 钉的两格。
    #[test]
    fn old_reading_reproduces_the_original_unit_test_numbers() {
        let old = |arm| publish_write_bytes(WriteAccounting::OneWriteOneDeleteAcrossKeptGenerations, arm, 1024, 64, MIDDLE_WIDTH, DeviceDimension::Kept);
        assert_eq!(old(Arm::UpperBound), 35_389_440);
        assert_eq!(old(Arm::PerStatisticNarrow), 138_420);
    }

    /// Q-E71-1 判决格：上界臂 middle 树数 1024 设备数 64，新口径 9 × 1024 × 64 × 30。
    #[test]
    fn judgement_cell_new_reading_is_seventeen_million_and_within_budget() {
        let cell = PublishBudgetCell::at(MIDDLE_WIDTH, Arm::UpperBound, JUDGEMENT_CELL_TREE_COUNT, JUDGEMENT_CELL_DEVICE_COUNT);
        assert_eq!(cell.new_bytes, 9 * 1024 * 64 * 30);
        assert_eq!(cell.new_bytes, 17_694_720);
        assert_eq!(cell.old_bytes, 35_389_440);
        assert!(!cell.is_new_over_budget());
        assert!(cell.is_old_over_budget());
        assert_eq!(cell.new_bytes_permille_of_budget(), 823);
    }

    /// 过界是严格大于：恰好等于门槛判不过界（登记 EV4：原取样点里没有一格恰好等于门槛，这一条直接调判定函数）。
    #[test]
    fn exactly_the_budget_is_within_and_one_byte_more_is_over() {
        assert!(!is_over_budget(21_474_836, ACCOUNTING_WRITE_BUDGET_BYTES));
        assert!(is_over_budget(21_474_837, ACCOUNTING_WRITE_BUDGET_BYTES));
    }

    /// 判别力自证（登记第八节）：门槛挪到判决格新口径字节上判不过界，挪低一字节判过界。
    #[test]
    fn moving_the_budget_across_the_judgement_cell_flips_its_verdict() {
        let judgement_cell_new_bytes = PublishBudgetCell::at(MIDDLE_WIDTH, Arm::UpperBound, 1024, 64).new_bytes;
        assert!(!is_over_budget(judgement_cell_new_bytes, 17_694_720));
        assert!(is_over_budget(judgement_cell_new_bytes, 17_694_719));
        assert!(!is_over_budget(judgement_cell_new_bytes, ACCOUNTING_WRITE_BUDGET_BYTES));
    }

    /// 阳性对照甲：每条臂、每档宽度、每一格，旧口径字节恰好等于新口径字节 × 2（加法形不写减法）。
    #[test]
    fn old_reading_is_exactly_twice_the_new_reading_on_every_cell_of_every_arm() {
        let cells = all_publish_budget_cells();
        for arm in ARMS {
            let cells_of_this_arm: Vec<&PublishBudgetCell> = cells.iter().filter(|cell| cell.arm == arm).collect();
            assert_eq!(cells_of_this_arm.len(), 60, "{arm:?}：四档宽度 × 5 个树数 × 3 个设备数");
            for cell in cells_of_this_arm {
                assert_eq!(cell.old_bytes, 2 * cell.new_bytes, "{cell:?}");
            }
        }
    }

    /// 阳性对照丙：树数 64 设备数 8、middle、新口径，去掉设备维。上界臂恰好降 8 倍，另两臂严格下降且不到 8 倍。
    #[test]
    fn dropping_the_device_dimension_divides_the_upper_bound_by_eight_and_the_others_by_less() {
        let upper_bound = DropDeviceControl::of(Arm::UpperBound);
        assert_eq!(upper_bound.with_device_bytes, 4608 * 30);
        assert_eq!(upper_bound.without_device_bytes, 576 * 30);
        assert!(upper_bound.holds_for(Arm::UpperBound));
        let narrow = DropDeviceControl::of(Arm::PerStatisticNarrow);
        assert_eq!(narrow.with_device_bytes, 163 * 30);
        assert_eq!(narrow.without_device_bytes, 135 * 30);
        assert!(narrow.holds_for(Arm::PerStatisticNarrow));
        assert!(!narrow.drops_exactly_by_device_count());
        let wide = DropDeviceControl::of(Arm::PerStatisticWide);
        assert_eq!(wide.with_device_bytes, 2368 * 30);
        assert_eq!(wide.without_device_bytes, 576 * 30);
        assert!(wide.holds_for(Arm::PerStatisticWide));
        assert!(!wide.drops_exactly_by_device_count());
    }

    /// 登记第七节乙类：独立锚点（`anchors.py` 只按登记读法第 2、3 条的式子现算，sha256 a3d0bd98… 的 `anchors.out`）逐行钉住。
    /// 宽度 22 / 30 / 34 / 50 依次是 narrow、middle、S2、wide；每行是 (宽度, 臂, 树数, 设备数, 新口径字节, 新口径过界, 旧口径字节, 旧口径过界)。
    #[test]
    fn independent_anchor_rows_match_on_every_width_arm_and_cell() {
        let anchor_rows: [(EntryWidth, Arm, u64, u64, u64, bool, u64, bool); 48] = [
        (NARROW_WIDTH, Arm::UpperBound, 1024, 64, 12976128, false, 25952256, true),
        (NARROW_WIDTH, Arm::PerStatisticNarrow, 1024, 64, 50754, false, 101508, false),
        (NARROW_WIDTH, Arm::PerStatisticWide, 1024, 64, 5879808, false, 11759616, false),
        (NARROW_WIDTH, Arm::UpperBound, 8192, 8, 12976128, false, 25952256, true),
        (NARROW_WIDTH, Arm::PerStatisticNarrow, 8192, 8, 361218, false, 722436, false),
        (NARROW_WIDTH, Arm::PerStatisticWide, 8192, 8, 6668288, false, 13336576, false),
        (NARROW_WIDTH, Arm::UpperBound, 8192, 64, 103809024, true, 207618048, true),
        (NARROW_WIDTH, Arm::PerStatisticNarrow, 8192, 64, 366146, false, 732292, false),
        (NARROW_WIDTH, Arm::PerStatisticWide, 8192, 64, 47038464, true, 94076928, true),
        (NARROW_WIDTH, Arm::UpperBound, 1024, 8, 1622016, false, 3244032, false),
        (NARROW_WIDTH, Arm::PerStatisticNarrow, 1024, 8, 45826, false, 91652, false),
        (NARROW_WIDTH, Arm::PerStatisticWide, 1024, 8, 833536, false, 1667072, false),
        (MIDDLE_WIDTH, Arm::UpperBound, 1024, 64, 17694720, false, 35389440, true),
        (MIDDLE_WIDTH, Arm::PerStatisticNarrow, 1024, 64, 69210, false, 138420, false),
        (MIDDLE_WIDTH, Arm::PerStatisticWide, 1024, 64, 8017920, false, 16035840, false),
        (MIDDLE_WIDTH, Arm::UpperBound, 8192, 8, 17694720, false, 35389440, true),
        (MIDDLE_WIDTH, Arm::PerStatisticNarrow, 8192, 8, 492570, false, 985140, false),
        (MIDDLE_WIDTH, Arm::PerStatisticWide, 8192, 8, 9093120, false, 18186240, false),
        (MIDDLE_WIDTH, Arm::UpperBound, 8192, 64, 141557760, true, 283115520, true),
        (MIDDLE_WIDTH, Arm::PerStatisticNarrow, 8192, 64, 499290, false, 998580, false),
        (MIDDLE_WIDTH, Arm::PerStatisticWide, 8192, 64, 64143360, true, 128286720, true),
        (MIDDLE_WIDTH, Arm::UpperBound, 1024, 8, 2211840, false, 4423680, false),
        (MIDDLE_WIDTH, Arm::PerStatisticNarrow, 1024, 8, 62490, false, 124980, false),
        (MIDDLE_WIDTH, Arm::PerStatisticWide, 1024, 8, 1136640, false, 2273280, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::UpperBound, 1024, 64, 20054016, false, 40108032, true),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticNarrow, 1024, 64, 78438, false, 156876, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticWide, 1024, 64, 9086976, false, 18173952, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::UpperBound, 8192, 8, 20054016, false, 40108032, true),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticNarrow, 8192, 8, 558246, false, 1116492, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticWide, 8192, 8, 10305536, false, 20611072, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::UpperBound, 8192, 64, 160432128, true, 320864256, true),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticNarrow, 8192, 64, 565862, false, 1131724, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticWide, 8192, 64, 72695808, true, 145391616, true),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::UpperBound, 1024, 8, 2506752, false, 5013504, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticNarrow, 1024, 8, 70822, false, 141644, false),
        (CURRENT_IMPLEMENTATION_WIDTH, Arm::PerStatisticWide, 1024, 8, 1288192, false, 2576384, false),
        (WIDE_WIDTH, Arm::UpperBound, 1024, 64, 29491200, true, 58982400, true),
        (WIDE_WIDTH, Arm::PerStatisticNarrow, 1024, 64, 115350, false, 230700, false),
        (WIDE_WIDTH, Arm::PerStatisticWide, 1024, 64, 13363200, false, 26726400, true),
        (WIDE_WIDTH, Arm::UpperBound, 8192, 8, 29491200, true, 58982400, true),
        (WIDE_WIDTH, Arm::PerStatisticNarrow, 8192, 8, 820950, false, 1641900, false),
        (WIDE_WIDTH, Arm::PerStatisticWide, 8192, 8, 15155200, false, 30310400, true),
        (WIDE_WIDTH, Arm::UpperBound, 8192, 64, 235929600, true, 471859200, true),
        (WIDE_WIDTH, Arm::PerStatisticNarrow, 8192, 64, 832150, false, 1664300, false),
        (WIDE_WIDTH, Arm::PerStatisticWide, 8192, 64, 106905600, true, 213811200, true),
        (WIDE_WIDTH, Arm::UpperBound, 1024, 8, 3686400, false, 7372800, false),
        (WIDE_WIDTH, Arm::PerStatisticNarrow, 1024, 8, 104150, false, 208300, false),
        (WIDE_WIDTH, Arm::PerStatisticWide, 1024, 8, 1894400, false, 3788800, false),
        ];
        for (width, arm, tree_count, device_count, new_bytes, new_over, old_bytes, old_over) in anchor_rows {
            let cell = PublishBudgetCell::at(width, arm, tree_count, device_count);
            assert_eq!((cell.new_bytes, cell.is_new_over_budget()), (new_bytes, new_over), "新口径 {cell:?}");
            assert_eq!((cell.old_bytes, cell.is_old_over_budget()), (old_bytes, old_over), "旧口径 {cell:?}");
        }
    }

    /// 旧格式行与原产物逐字相同（原产物第 112、164、165 行，去掉 `E7RESULT ` 前缀）。
    #[test]
    fn original_format_lines_match_the_original_product_text() {
        let line_of = |width, arm, tree_count, device_count| {
            original_format_checkpoint_budget_line(PublishBudgetCell::at(width, arm, tree_count, device_count)).expect("原三档、原两臂都有旧格式行")
        };
        assert_eq!(
            line_of(NARROW_WIDTH, Arm::UpperBound, 1, 1),
            "name=ckpt_budget width=narrow key_w=14 val_w=8 arm=upper_bound t=1 d=1 bytes=396 budget=21474836 over=0"
        );
        assert_eq!(
            line_of(MIDDLE_WIDTH, Arm::UpperBound, 1024, 64),
            "name=ckpt_budget width=mid key_w=22 val_w=8 arm=upper_bound t=1024 d=64 bytes=35389440 budget=21474836 over=1"
        );
        assert_eq!(
            line_of(MIDDLE_WIDTH, Arm::PerStatisticNarrow, 1024, 64),
            "name=ckpt_budget width=mid key_w=22 val_w=8 arm=per_stat_narrow t=1024 d=64 bytes=138420 budget=21474836 over=0"
        );
        assert_eq!(original_format_checkpoint_budget_line(PublishBudgetCell::at(MIDDLE_WIDTH, Arm::PerStatisticWide, 1024, 64)), None);
        assert_eq!(original_format_checkpoint_budget_line(PublishBudgetCell::at(CURRENT_IMPLEMENTATION_WIDTH, Arm::UpperBound, 1024, 64)), None);
    }

    /// 取样点覆盖装置会跑到的全部取值（mutation-sampling.md 第四类）：main 打的每一格这里都算一遍、打一遍，条数钉死。
    #[test]
    fn every_cell_the_device_prints_is_computed_without_overflow_and_counted() {
        let cells = all_publish_budget_cells();
        assert_eq!(cells.len(), 4 * 5 * 3 * 3);
        assert_eq!(cells.iter().map(|cell| publish_budget_line(*cell)).count(), 180);
        assert_eq!(cells.iter().filter_map(|cell| original_format_checkpoint_budget_line(*cell)).count(), 90);
        assert_eq!(cells.iter().map(|cell| cell.old_bytes).max(), Some(2 * 9 * 8192 * 64 * 50));
        for chunk in cells.chunks(ARMS.len()) {
            let new_bytes_of = |arm: Arm| chunk.iter().find(|cell| cell.arm == arm).expect("每组三条臂各一格").new_bytes;
            assert!(new_bytes_of(Arm::UpperBound) >= new_bytes_of(Arm::PerStatisticWide), "{chunk:?}");
            assert!(new_bytes_of(Arm::PerStatisticWide) >= new_bytes_of(Arm::PerStatisticNarrow), "{chunk:?}");
        }
    }

    /// S2 的条目宽：key 22 + value 8 + seq 4（`crates/singlefs-format/src/lib.rs` 的 `ACCOUNTING_ENTRY_BYTES`）。
    #[test]
    fn current_implementation_entry_is_thirty_four_bytes() {
        assert_eq!(CURRENT_IMPLEMENTATION_WIDTH.entry_bytes(), 22 + 8 + 4);
        assert_eq!(CURRENT_IMPLEMENTATION_WIDTH.entry_bytes(), 34);
        assert_eq!(MIDDLE_WIDTH.entry_bytes(), 30);
    }

    /// Q-E71-2、Q-E71-3、Q-E71-附带 1 那几行列出的过界格：新口径下 middle 档上界臂只有 8192 × 64 一格，窄读臂一格都没有。
    #[test]
    fn over_budget_cell_lists_on_the_middle_width() {
        let middle_cells_of = |arm: Arm| -> Vec<PublishBudgetCell> {
            all_publish_budget_cells().into_iter().filter(|cell| cell.arm == arm && cell.width == MIDDLE_WIDTH).collect()
        };
        assert_eq!(over_budget_cell_list(&middle_cells_of(Arm::UpperBound)), "middle:8192:64");
        assert_eq!(over_budget_cell_list(&middle_cells_of(Arm::PerStatisticWide)), "middle:8192:64");
        assert_eq!(over_budget_cell_list(&middle_cells_of(Arm::PerStatisticNarrow)), "none");
    }
}
