//! 第 5 次跑的枚举域、工作单位、分片与变体结果的文本。

use super::*;

// ============================================================================
// 十五、第 5 次跑（`research/prompts/e158-r5-prereg.md`）：被抛弃根的账读不出，枚举型覆盖。
//      模式 `r5-constants`、`r5-enum-g0`、`r5-enum-g0-one-thread`、`r5-enum-s16`、`r5-enum-small-ring`、`r5-enum-s4`；
//      臂由 `SINGLEFS_E158_ARM` 选（today / skip）。前几次跑的模式只调用、不改语义，这一节只加。
//      一档推不进一条命令的挂钟时分片跑：枚举模式后面给 `--shard-index <n> --shard-count <m>`（n < m）只跑单位列表里归第 n 片的单位，
//      判定写进库、产物只有参数行与摘要行；各片都跑完之后给 `--reuse-only-from-the-library` 再起同一个模式，判定全部从库里取
//      （至多现判转交哨兵让回「不在」的那一条），库里再缺哪一个当场停；这一趟的结果行（`r5_run_parameters` 以外）与整趟一次跑的逐字相同。
//      节点沿用 `enumerate` 的深度优先序（H3），造法是登记 5.1：每个节点的镜像先写成「系统配置轮换没落盘」，
//      再做抛弃步，成立的对在抛弃步之后的镜像上做一个孪生、十二个账读不出变体与两个根槽读不出变体。
// ============================================================================

/// 一趟枚举的域：几何、步序列上限 L、起点 n1 的取值（登记第一节「读法写死」表「节点」）。
#[derive(Clone)]
pub(super) struct FifthRunDomain {
    pub(super) geometry: Geometry,
    pub(super) step_limit: u64,
    pub(super) starting_overwrites: Vec<u64>,
}

/// 起点 n1 的取值（登记第一节「读法写死」表「节点」）。
pub(super) const FIFTH_RUN_STARTING_OVERWRITES: [u64; 3] = [1, 2, 3];
/// 单位数至少是线程数的几倍（登记 5.7 第 2 条）。
const FIFTH_RUN_UNITS_PER_THREAD_AT_LEAST: usize = 8;

/// 步序列的文本：键与产物行里用，不含空白；起点（没有任何步）写 `start`。
pub(super) fn step_sequence_text(steps: &[HistoryStep]) -> String {
    if steps.is_empty() {
        return "start".to_string();
    }
    steps
        .iter()
        .map(|step| match step {
            HistoryStep::Overwrite => "W".to_string(),
            HistoryStep::MountWritable => "M".to_string(),
            HistoryStep::RollbackTo(instance, txg) => format!("R({instance}.{txg})"),
        })
        .collect::<Vec<String>>()
        .join(",")
}

/// 一个节点最后一步的种类（Q0 按它分）。
pub(super) fn last_step_kind(steps: &[HistoryStep]) -> &'static str {
    match steps.last() {
        None => "start",
        Some(HistoryStep::Overwrite) => "W",
        Some(HistoryStep::MountWritable) => "M",
        Some(HistoryStep::RollbackTo(..)) => "R",
    }
}

/// 节点的步序列去掉起点 `bootstrap` 自己做的 n1 次覆盖写之后剩下的那一段。
pub(super) fn steps_after_the_starting_overwrites(
    node: &SimNode,
    starting_overwrites: u64,
) -> Vec<HistoryStep> {
    let skipped = usize::try_from(starting_overwrites).expect("n1 装得进 usize");
    node.path[skipped..].to_vec()
}

/// 从起点按步序列重放出一个节点（工人自己重建节点，不跨线程传 `SimNode`，登记 5.7 第 2、6 条）。
pub(super) fn replay_steps(
    geometry: &Geometry,
    starting_overwrites: u64,
    steps: &[HistoryStep],
) -> Result<SimNode, String> {
    let parameters = parameters_for(geometry);
    let mut node = bootstrap(geometry, starting_overwrites)?;
    for step in steps {
        node = match step {
            HistoryStep::Overwrite => apply_overwrite(&node, &parameters)?,
            HistoryStep::MountWritable => apply_mount_writable(&node, &parameters)?,
            HistoryStep::RollbackTo(instance, txg) => apply_mount_rollback(
                &node,
                RollbackTarget {
                    instance: InstanceGeneration(*instance),
                    checkpoint_txg: CheckpointTxg(*txg),
                },
                &parameters,
            )?,
        };
    }
    Ok(node)
}

/// `enumerate` 对一个节点展开的结果，次序相同：覆盖写（挂着时）、可写挂载、对每条可读根的回退。
pub(super) struct NodeExpansion {
    children: Vec<SimNode>,
    /// 前提不满足的分支数，与 `enumerate` 记进 `summary.errors` 的同数。
    pub(super) transition_failures: u64,
}

pub(super) fn expansion_of_node(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
) -> NodeExpansion {
    let mut expansion = NodeExpansion {
        children: Vec::new(),
        transition_failures: 0,
    };
    let mut note = |attempt: Result<SimNode, _>| match attempt {
        Ok(next) => expansion.children.push(next),
        Err(_) => expansion.transition_failures += 1,
    };
    if node.session.is_some() {
        note(apply_overwrite(node, parameters));
    }
    note(apply_mount_writable(node, parameters));
    for root in &readable_roots_independent(&node.pool, geometry_view) {
        let target = RollbackTarget {
            instance: InstanceGeneration(root.0),
            checkpoint_txg: CheckpointTxg(root.1),
        };
        note(apply_mount_rollback(node, target, parameters));
    }
    expansion
}

/// 工人的一个单位（登记 5.7 第 2 条）：起点 n1 加步序列前缀；`whole_subtree` 为真时带着这个节点整棵子树（到步序列上限），
/// 为假时只处理这个节点自己。
#[derive(Clone, Debug)]
pub(super) struct EnumerationUnit {
    pub(super) starting_overwrites: u64,
    pub(super) prefix: Vec<HistoryStep>,
    pub(super) whole_subtree: bool,
}

/// 串行走一遍树到切深 `cut_depth`，按深度优先序列出单位。
struct UnitListing<'listing> {
    parameters: &'listing MakeFilesystemParameters,
    geometry_view: &'listing PoolGeometry,
    starting_overwrites: u64,
    cut_depth: u64,
    units: Vec<EnumerationUnit>,
}

impl UnitListing<'_> {
    pub(super) fn collect(&mut self, node: &SimNode, depth: u64) {
        let at_the_cut = depth == self.cut_depth;
        self.units.push(EnumerationUnit {
            starting_overwrites: self.starting_overwrites,
            prefix: steps_after_the_starting_overwrites(node, self.starting_overwrites),
            whole_subtree: at_the_cut,
        });
        if at_the_cut {
            return;
        }
        let expansion = expansion_of_node(node, self.parameters, self.geometry_view);
        for child in &expansion.children {
            self.collect(child, depth + 1);
        }
    }
}

pub(super) fn list_enumeration_units(
    domain: &FifthRunDomain,
    cut_depth: u64,
) -> Result<Vec<EnumerationUnit>, String> {
    let parameters = parameters_for(&domain.geometry);
    let mut units = Vec::new();
    for &starting_overwrites in &domain.starting_overwrites {
        let root = bootstrap(&domain.geometry, starting_overwrites)?;
        let geometry_view = independent_geometry(&root.pool)?;
        let mut listing = UnitListing {
            parameters: &parameters,
            geometry_view: &geometry_view,
            starting_overwrites,
            cut_depth,
            units: Vec::new(),
        };
        listing.collect(&root, 0);
        units.extend(listing.units);
    }
    Ok(units)
}

/// 切深：使单位数 ≥ 8 × 线程数的最小深度，到步序列上限为止（登记 5.7 第 2 条）。
fn choose_cut_depth(
    domain: &FifthRunDomain,
    threads: usize,
) -> Result<(u64, Vec<EnumerationUnit>), String> {
    let wanted = threads * FIFTH_RUN_UNITS_PER_THREAD_AT_LEAST;
    for cut_depth in 0..=domain.step_limit {
        let units = list_enumeration_units(domain, cut_depth)?;
        if units.len() >= wanted || cut_depth == domain.step_limit {
            return Ok((cut_depth, units));
        }
    }
    Err("切深：步序列上限没有取值".to_string())
}

/// 这一趟的单位列表与切深：给了切深（单测用）就用它，没给按线程数选。
pub(super) fn units_of_the_run(
    domain: &FifthRunDomain,
    threads: usize,
    cut_depth: Option<u64>,
) -> Result<(u64, Vec<EnumerationUnit>), String> {
    match cut_depth {
        Some(depth) => {
            let depth = depth.min(domain.step_limit);
            Ok((depth, list_enumeration_units(domain, depth)?))
        }
        None => choose_cut_depth(domain, threads),
    }
}

/// 一趟枚举怎么拼出产物：整趟一次跑；只跑单位的一片（判定写进库，不汇总）；或判定全部从库里取
/// （片都跑完之后的合并，产物与整趟一次跑的结果行逐字相同）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FifthRunAssembly {
    Whole,
    Shard(UnitShard),
    ReuseOnlyFromTheLibrary,
}

impl FifthRunAssembly {
    pub(super) fn name(self) -> &'static str {
        match self {
            FifthRunAssembly::Whole => "whole",
            FifthRunAssembly::Shard(_) => "shard",
            FifthRunAssembly::ReuseOnlyFromTheLibrary => "reuse_only_from_the_library",
        }
    }
}

/// 单位列表（深度优先序）的一片：下标为 `i` 的单位归 `holds(i)` 为真的那一片。
/// 各片两两不交、合起来是整张单位列表；列表只由域、切深决定，与哪一片无关。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UnitShard {
    pub(super) index: u64,
    pub(super) count: u64,
}

/// 斐波那契散列的乘数（2^64 除以黄金比例取整）：相邻下标的散列值相差约 0.618 圈，各片分到的单位数几乎相等，不随深度优先序的周期结构偏。
const UNIT_SHARD_HASH_MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;

impl UnitShard {
    pub(super) fn holds(&self, unit_index: usize) -> bool {
        let position = u64::try_from(unit_index).expect("下标装得进 u64");
        let spread = position.wrapping_mul(UNIT_SHARD_HASH_MULTIPLIER) >> 32;
        (spread * self.count) >> 32 == self.index
    }
}

/// 片数上限：`holds` 里散列值（32 位）乘片数不溢出 u64。
const UNIT_SHARD_COUNT_LIMIT: u64 = 65_536;

/// 命令行 `--shard-index <n>`：没给交 None，给了没跟数或不是数报错。
fn unsigned_flag_of(arguments: &[String], flag: &str) -> Result<Option<u64>, String> {
    let mut value = None;
    let mut remaining = arguments.iter();
    while let Some(argument) = remaining.next() {
        if argument == flag {
            let text = remaining.next().ok_or_else(|| format!("{flag} 后面缺数"))?;
            value = Some(
                text.parse::<u64>()
                    .map_err(|_| format!("{flag} 后面的 {text} 不是非负整数"))?,
            );
        }
    }
    Ok(value)
}

/// 命令行 `--reuse-only-from-the-library`。
pub(super) const REUSE_ONLY_FLAG: &str = "--reuse-only-from-the-library";

/// 模式后面的片与合并参数：`--shard-index <n> --shard-count <m>` 成对给（n < m）、或 `--reuse-only-from-the-library`、或都不给；
/// 两样不同时给；单线程对拍趟不连库，只许都不给。
pub(super) fn fifth_run_assembly_of(
    mode: &str,
    arguments: &[String],
) -> Result<FifthRunAssembly, String> {
    let index = unsigned_flag_of(arguments, "--shard-index")?;
    let count = unsigned_flag_of(arguments, "--shard-count")?;
    let reuse_only = arguments.iter().any(|argument| argument == REUSE_ONLY_FLAG);
    let assembly = match (index, count, reuse_only) {
        (None, None, false) => FifthRunAssembly::Whole,
        (None, None, true) => FifthRunAssembly::ReuseOnlyFromTheLibrary,
        (Some(index), Some(count), false) => {
            if count == 0 || count > UNIT_SHARD_COUNT_LIMIT || index >= count {
                return Err(format!(
                    "--shard-index {index} 要小于 --shard-count {count}，片数在 1 到 {UNIT_SHARD_COUNT_LIMIT} 之间"
                ));
            }
            FifthRunAssembly::Shard(UnitShard { index, count })
        }
        (Some(_), Some(_), true) => {
            return Err(format!(
                "{REUSE_ONLY_FLAG} 与 --shard-index / --shard-count 不能同给"
            ));
        }
        _ => return Err("--shard-index 与 --shard-count 要成对给".to_string()),
    };
    if mode == "r5-enum-g0-one-thread" && assembly != FifthRunAssembly::Whole {
        return Err("单线程对拍趟不连库，不分片也不只取库".to_string());
    }
    Ok(assembly)
}

/// 一个变体（一次 `mount_writable`，加续写）的结局。判定单位的值：文本形态由 [`VariantResult::to_text`] 写、
/// [`VariantResult::from_text`] 读，库里复用来的与现判的走同一条解析，汇总不分来路。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VariantResult {
    /// 这个变体在这个对上做得成（盘上没有那一份副本的单份变体不适用）。
    pub(super) applicable: bool,
    pub(super) class: String,
    pub(super) error: String,
    /// `abandoned_roots_unreadable`；挂载报错时没有。
    pub(super) count: Option<u64>,
    /// 装置自己数的 through(u)（两份坏的变体）；单份坏与孪生、根槽变体写 0。
    pub(super) expected_count: u64,
    /// 分盘的隔离槽数 (盘号, 槽数)。
    pub(super) isolated_by_device: Vec<(u32, u64)>,
    /// 点名的落点这次调用里全被拦到。
    pub(super) intercepted_all: bool,
    pub(super) writes: u64,
    /// 变体坏的那个单元被其余可读根共享。
    pub(super) shared: bool,
    /// 被抛弃根独占的落点按槽展开的槽数 |U_a|。
    pub(super) exclusive_slots: u64,
    /// 同上，分盘。
    pub(super) exclusive_by_device: Vec<(u32, u64)>,
    /// 续写的终点 txg；没续写（挂载报错，或这个变体不续写）没有。
    pub(super) continued_to_txg: Option<u64>,
    /// 续写每发布一次之后的累计复用数。
    pub(super) reuse_trajectory: Vec<u64>,
}

pub(super) fn dash_when_empty(text: String) -> String {
    if text.is_empty() {
        "-".to_string()
    } else {
        text
    }
}

/// 分盘的计数文本：`盘号:数` 用 `;` 连起来，空写 `-`。
fn device_counts_text(counts: &[(u32, u64)]) -> String {
    dash_when_empty(
        counts
            .iter()
            .map(|(device, count)| format!("{device}:{count}"))
            .collect::<Vec<String>>()
            .join(";"),
    )
}

fn parse_device_counts(text: &str) -> Result<Vec<(u32, u64)>, String> {
    if text == "-" {
        return Ok(Vec::new());
    }
    text.split(';')
        .map(|part| -> Result<(u32, u64), String> {
            let (device, count) = part
                .split_once(':')
                .ok_or_else(|| format!("分盘计数认不出：{part}"))?;
            Ok((
                device
                    .parse::<u32>()
                    .map_err(|_| format!("盘号认不出：{part}"))?,
                count
                    .parse::<u64>()
                    .map_err(|_| format!("计数认不出：{part}"))?,
            ))
        })
        .collect()
}

pub(super) fn optional_number_text(value: Option<u64>) -> String {
    value.map_or("-".to_string(), |number| number.to_string())
}

/// 文本里 `名=值` 形的字段（空格隔开）的值。
pub(super) fn text_field_named<'text>(text: &'text str, name: &str) -> Result<&'text str, String> {
    text.split(' ')
        .find_map(|part| {
            part.strip_prefix(name)
                .and_then(|rest| rest.strip_prefix('='))
        })
        .ok_or_else(|| format!("判定值里没有字段 {name}：{text}"))
}

fn parse_optional_number(text: &str) -> Option<u64> {
    if text == "-" {
        None
    } else {
        text.parse().ok()
    }
}

impl VariantResult {
    pub(super) fn not_applicable() -> Self {
        VariantResult {
            applicable: false,
            class: "none".to_string(),
            error: "none".to_string(),
            count: None,
            expected_count: 0,
            isolated_by_device: Vec::new(),
            intercepted_all: true,
            writes: 0,
            shared: false,
            exclusive_slots: 0,
            exclusive_by_device: Vec::new(),
            continued_to_txg: None,
            reuse_trajectory: Vec::new(),
        }
    }

    pub(super) fn isolated_total(&self) -> u64 {
        self.isolated_by_device
            .iter()
            .map(|(_, slots)| *slots)
            .sum()
    }

    pub(super) fn reuse_final(&self) -> u64 {
        self.reuse_trajectory.last().copied().unwrap_or(0)
    }

    pub(super) fn reuse_peak(&self) -> u64 {
        self.reuse_trajectory.iter().copied().max().unwrap_or(0)
    }

    /// 轨迹里第一次为正的步序（从 1 数）。
    pub(super) fn first_positive_reuse_step(&self) -> Option<u64> {
        self.reuse_trajectory
            .iter()
            .position(|reused| *reused > 0)
            .map(|index| u64::try_from(index + 1).expect("步序装得进 u64"))
    }

    pub(super) fn mounted_ok(&self) -> bool {
        self.applicable && self.error == "none"
    }

    pub(super) fn to_text(&self) -> String {
        let isolated = device_counts_text(&self.isolated_by_device);
        let exclusive_by_device = device_counts_text(&self.exclusive_by_device);
        let trajectory = dash_when_empty(
            self.reuse_trajectory
                .iter()
                .map(u64::to_string)
                .collect::<Vec<String>>()
                .join(";"),
        );
        format!(
            "applicable={} class={} error={} count={} expected={} isolated={isolated} intercepted={} writes={} shared={} exclusive_slots={} exclusive_by_device={exclusive_by_device} continued_to={} trajectory={trajectory}",
            u8::from(self.applicable),
            self.class,
            self.error,
            optional_number_text(self.count),
            self.expected_count,
            u8::from(self.intercepted_all),
            self.writes,
            u8::from(self.shared),
            self.exclusive_slots,
            optional_number_text(self.continued_to_txg),
        )
    }

    pub(super) fn from_text(text: &str) -> Result<Self, String> {
        let field = |name: &str| text_field_named(text, name);
        let number = |name: &str| -> Result<u64, String> {
            field(name)?
                .parse()
                .map_err(|_| format!("判定值字段 {name} 不是数：{text}"))
        };
        let isolated_by_device = parse_device_counts(field("isolated")?)?;
        let exclusive_by_device = parse_device_counts(field("exclusive_by_device")?)?;
        let trajectory_text = field("trajectory")?;
        let mut reuse_trajectory = Vec::new();
        if trajectory_text != "-" {
            for part in trajectory_text.split(';') {
                reuse_trajectory.push(part.parse().map_err(|_| format!("轨迹认不出：{text}"))?);
            }
        }
        Ok(VariantResult {
            applicable: number("applicable")? == 1,
            class: field("class")?.to_string(),
            error: field("error")?.to_string(),
            count: parse_optional_number(field("count")?),
            expected_count: number("expected")?,
            isolated_by_device,
            intercepted_all: number("intercepted")? == 1,
            writes: number("writes")?,
            shared: number("shared")? == 1,
            exclusive_slots: number("exclusive_slots")?,
            exclusive_by_device,
            continued_to_txg: parse_optional_number(field("continued_to")?),
            reuse_trajectory,
        })
    }
}
