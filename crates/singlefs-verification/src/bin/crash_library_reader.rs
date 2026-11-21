//! 崩溃判定库的读库程序：键对不上的块数、一项要现判几块都要读判定库，python 不读它的布局。子命令 `keys` 读 prune-server 里那一份崩溃判定库
//! （`crash_library::LibraryOpenInThisProcess::open_or_share`：连哪一处照 `crash_library::prune_server_route_of_the_configuration`；
//! 没写服务的地址、连不上、读写不成照 prune-server 客户端的退出码退出，码见本文件头「退出码」那一段），一行都不写：
//! 按这份二进制的镜像判定版本（`crash_identity::ConclusionVersion::of_the_crash_image_judgement`，由结论种类与判器引到的被检测产品代码定）数录入表里的块：几块在这一把版本下判过、
//! 几块只在别的指纹下判过（键对不上，下一趟要现判）、几块哪一种判器的结论都没有；同一遍里把录入表记的每块状态数按同样三类加起来，
//! 再按块属性表里存着的每一种结论每一把版本各数一行。
//!
//! 读法（不逐块往返）：录入表整表逐帧扫一遍（值是 8 字节的状态数），块属性表整表逐帧扫一遍收每块存着的结论种类与版本
//! （`VerdictStore::visit_stored_conclusion_keys`；块属性与结论块同一批写、键的后缀相同、值只有几十字节，不把整块结论读回来），
//! 两份在内存里按块键对上。
//!
//! 用法：`crash_library_reader keys`；`crash_library_reader item <合表项名> <设点文件>…`；
//! `crash_library_reader items --item <合表项名> <设点文件>… [--item <合表项名> <设点文件>…]…`。
//!
//! 子命令 `item`（按项读法）：给崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项与这一项的设点文件（相对仓根的路径，
//! 至少一份；崩溃枚举用例与放量分项的测试目标文件 `crates/singlefs-verification/tests/<目标>.rs`，由入口照合表点名的部分现给），报这一项
//! 要现判几块、能复用几块：路径索引（`VerdictStore::paths_with_prefix`，录入段每个点登一行「路径 → <设点文件>::<点名>::<复用键十六进制>」）里
//! 设点文件是给的那几份的点，它们的复用键就是块键的头 32 字节；录入表里块键头 32 字节落在这几把复用键上的块，在这一把判法单元指纹下判过的
//! 能复用，别的（只在别的指纹下判过、哪一种结论都没有）要现判。同一个点在几项里都登过（设点文件是共用模块）就各项都数；
//! 路径索引同一路径只留最后登的身份，所以这是给任务服务估时的数，不是判定。几项的设点文件互不相交、合起来罩住录入表里每一块时，
//! 各项的能复用之和等于 `keys` 的 `judged_under_the_current_key`，要现判之和等于另两格之和。
//!
//! 子命令 `items`（一次报几项）：连一次库，路径索引、录入表、块属性表各整表扫一遍，几项共用这一遍；按命令行上 `--item` 的次序每项打一行，
//! 每行的写法与 `item` 那一行相同。同一份库上，它的输出与逐项各起一次 `item` 打的行依次拼起来逐字相同（本文件的单测钉）。
//! 项名与设点文件都不以 `-` 起头；每个 `--item` 后面是项名与至少一份设点文件。
//!
//! 输出：一行一项，制表符隔开，`<名>=<值>` 的字段次序固定。`scripts/crash/crash-point-replay.sh --list-items --with-keys` 照这一段读，
//! 它的自证 `list-items-with-keys` 格把下面两行标记之间的那一段（去掉行首的 `//! `）原样当这个程序的输出喂给它；本文件的单测钉格式化出来的行与
//! 这一段逐字相同——两边的写法与读法钉在同一段字上：
//! keys-format-begin
//! CRASH_LIBRARY_KEYS	library=prune-server unix:/run/singlefs-prune-server/server.sock	recorded_blocks=6	judged_under_the_current_key=2	judged_only_under_another_key=3	never_judged=1
//! CRASH_LIBRARY_KEYS_JUDGE	image_judgement	current_key=0505050505050505
//! CRASH_LIBRARY_STATES	recorded_states=20497	states_judged_under_the_current_key=4113	states_judged_only_under_another_key=12288	states_never_judged=4096
//! CRASH_LIBRARY_STORED_VERSION	image_judgement	version=0505050505050505	against_this_binary=current	blocks=2	blocks_recorded=2	states_of_the_blocks_recorded=4113
//! CRASH_LIBRARY_STORED_VERSION	image_judgement	version=0a0a0a0a0a0a0a0a	against_this_binary=another	blocks=4	blocks_recorded=3	states_of_the_blocks_recorded=12288
//! CRASH_LIBRARY_STORED_VERSION	observer	version=0c0c0c0c0c0c0c0c	against_this_binary=not_compared	blocks=1	blocks_recorded=1	states_of_the_blocks_recorded=4096
//! CRASH_LIBRARY_STORED_KIND	image_judgement	versions=2	block_version_pairs=6	pairs_recorded=5	states_of_the_pairs_recorded=16401	pairs_not_recorded=1
//! CRASH_LIBRARY_STORED_KIND	observer	versions=1	block_version_pairs=1	pairs_recorded=1	states_of_the_pairs_recorded=4096	pairs_not_recorded=0
//! keys-format-end
//!
//! `CRASH_LIBRARY_STATES` 是录入表记的状态数按所在那一块的去向加起来（同 `CRASH_LIBRARY_KEYS` 的三类）：三格之和等于 `recorded_states`。
//! `CRASH_LIBRARY_STORED_VERSION` 是块属性表里存着的每一种结论每一把版本一行，按种类、再按版本字节排：`version=` 是那一把的头 8 字节；
//! `against_this_binary=` 是 `current`（就是这份二进制里那一种判器的那一把）、`another`（这份二进制算得出那一种的那一把，而存着的不是它）、
//! `not_compared`（这份二进制没有那一种的那一把可比：`keys` 不数的种类，例观察者）；`blocks=` 是存着这一把结论的块数，
//! `blocks_recorded=` 是其中录入表里有的块数，`states_of_the_blocks_recorded=` 是这几块的状态数之和（录入表里没有的块不知道几态，不加）。
//! 一块在几把版本下都存着就在几行里都数，所以各行态数之和等于录入表每块状态数乘它存着的版本数再加起来，不等于判过的态数。
//! `CRASH_LIBRARY_STORED_KIND` 是每一种结论一行合计，接在各把版本那几行之后、按种类排，种类的写法与 `CRASH_LIBRARY_STORED_VERSION` 那一格相同：
//! `versions=` 是这一种存着几把版本（那一种的版本行数）；`block_version_pairs=` 是块×版本对数（那几行 `blocks=` 之和）；
//! `pairs_recorded=` 是其中块在录入表里的对数（那几行 `blocks_recorded=` 之和）；`states_of_the_pairs_recorded=` 是这些对的态数之和
//! （那几行 `states_of_the_blocks_recorded=` 之和）；`pairs_not_recorded=` 是块不在录入表里、读不出态数的对数（按块属性表里的块查录入表另数，
//! 不是拿另两格相减），所以 `pairs_recorded` 与 `pairs_not_recorded` 之和等于 `block_version_pairs` 是一条对得上的账。
//! 红块数不报：一块一种结论红不红是违例表里有没有这一块的行，判定库只有逐块问的读法（`VerdictStore::conclusion_block_has_violations`），
//! 没有整表扫违例表的读法，逐块问就是每把版本一次往返。
//! `scripts/crash/crash-point-replay.sh --list-items --with-keys` 只认 `CRASH_LIBRARY_KEYS` 与 `CRASH_LIBRARY_KEYS_JUDGE` 两种行，别的行跳过。
//!
//! `item` 的输出一行（同样钉在文件头这一段上；`items` 每项一行，同一个写法）：
//! item-format-begin
//! CRASH_LIBRARY_ITEM	item=acquisition-barrier	recorded_blocks=5	to_judge=3	reusable=2
//! item-format-end
//!
//! 一块「在这一把指纹下判过」：CPU 判器或 GPU 判器判出的那一种结论在这份二进制的那一把版本（`current_key=` 那十六进制的头 8 字节；
//! 由结论种类与判器引到的被检测产品代码定，判器的代码不进，两条路线同一把）下存着。
//! 「只在别的指纹下判过」：有哪一种判器的结论，可没有一把是这一份的。块按录入表数（录入段录下的块，键只由内容定，几条流共有的块只数一次）。
//!
//! 退出码：0 读完；1 读不了（库不是这一版的布局、读回的行不合布局）；2 用法错（子命令认不得、参数多了）；3 平常跑设了改连接地址的变量、
//! socket 不在而 TCP 地址两处都没有（`crash_library::PruneServerRouteRefusal`）、服务拒了一帧这类别的错；75 连不上 prune-server、读到一半服务停了
//! （照 prune-server 客户端的退出码表）。
#![allow(
    clippy::tabs_in_doc_comments,
    reason = "文件头 keys 输出那一段就是这个程序打的行，字段之间是制表符；脚本照原样取那一段，换成空格两边就对不上"
)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::ExitCode;

use singlefs_verification::crash_identity::{ConclusionKind, ConclusionVersion};
use singlefs_verification::crash_library::LibraryOpenInThisProcess;
use singlefs_verification::hexadecimal::hexadecimal_text;
use singlefs_verification::verdict_store::{VerdictBlockKey, VerdictStore, VerdictStoreError};

/// 一块在库里的去向。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StandingOfABlock {
    JudgedUnderTheCurrentKey,
    JudgedOnlyUnderAnotherKey,
    NeverJudged,
}

/// 库里录下的块按去向数出来的账。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BlockCounts {
    recorded_blocks: u64,
    judged_under_the_current_key: u64,
    judged_only_under_another_key: u64,
    never_judged: u64,
}

impl BlockCounts {
    fn count(&mut self, standing: StandingOfABlock) {
        self.recorded_blocks += 1;
        match standing {
            StandingOfABlock::JudgedUnderTheCurrentKey => self.judged_under_the_current_key += 1,
            StandingOfABlock::JudgedOnlyUnderAnotherKey => self.judged_only_under_another_key += 1,
            StandingOfABlock::NeverJudged => self.never_judged += 1,
        }
    }
}

/// 录入表记的状态数按所在那一块的去向加起来的账（`CRASH_LIBRARY_STATES` 那一行）。每块的状态数是 u64，几块加起来用 u128 不会溢出。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StateCounts {
    recorded_states: u128,
    states_judged_under_the_current_key: u128,
    states_judged_only_under_another_key: u128,
    states_never_judged: u128,
}

impl StateCounts {
    fn count(&mut self, standing: StandingOfABlock, state_count_of_the_block: u64) {
        let states = u128::from(state_count_of_the_block);
        self.recorded_states += states;
        match standing {
            StandingOfABlock::JudgedUnderTheCurrentKey => {
                self.states_judged_under_the_current_key += states;
            }
            StandingOfABlock::JudgedOnlyUnderAnotherKey => {
                self.states_judged_only_under_another_key += states;
            }
            StandingOfABlock::NeverJudged => {
                self.states_never_judged += states;
            }
        }
    }
}

/// 块属性表里存着的一种结论一把版本的账（`CRASH_LIBRARY_STORED_VERSION` 那一行）：存着它的块数、其中录入表里有的块数与这几块的状态数之和，
/// 与录入表里没有的块数（不打在那一行上，只进 `CRASH_LIBRARY_STORED_KIND` 的 `pairs_not_recorded=`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StoredVersionCounts {
    blocks: u64,
    blocks_recorded: u64,
    states_of_the_blocks_recorded: u128,
    blocks_not_recorded: u64,
}

/// 一种结论各把版本的账加起来（`CRASH_LIBRARY_STORED_KIND` 那一行）：几把版本、块×版本对数、其中块在录入表里的对数与这些对的态数、
/// 块不在录入表里的对数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StoredKindTotals {
    versions: u64,
    block_version_pairs: u64,
    pairs_recorded: u64,
    states_of_the_pairs_recorded: u128,
    pairs_not_recorded: u64,
}

/// 块属性表里存着的一把结论版本，对着这份二进制里那一种判器的那一把是什么（文件头 `against_this_binary=` 那一格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoredVersionAgainstThisBinary {
    /// 就是这份二进制里那一种判器的那一把。
    Current,
    /// 这份二进制算得出那一种判器的那一把，而存着的不是它。
    Another,
    /// 这份二进制没有那一种的那一把可比：`keys` 不数的种类（观察者）。
    NotCompared,
}

impl StoredVersionAgainstThisBinary {
    fn text(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Another => "another",
            Self::NotCompared => "not_compared",
        }
    }
}

/// `keys` 报的账：录入表里的块与态按去向数，块属性表里每一种结论每一把版本各一份账（按种类、再按版本字节排）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct LibraryTally {
    blocks: BlockCounts,
    states: StateCounts,
    stored_versions: BTreeMap<(ConclusionKind, ConclusionVersion), StoredVersionCounts>,
}

/// 块属性表里每块存着的结论种类与版本（[`stored_conclusions_of_each_block`] 扫出来的）；没有的块是一种结论都没有。
type StoredConclusionsOfEachBlock =
    HashMap<VerdictBlockKey, Vec<(ConclusionKind, ConclusionVersion)>>;

/// 版本在输出里只打头几个字节（`current_key=`、`version=` 两格）。
const HEXADECIMAL_HEAD_BYTES_OF_A_VERSION: usize = 8;

/// 一种结论在这份二进制里的那一把版本（由结论种类与判器引到的被检测产品代码定，判器的代码不进；那一组拒算时取这个进程的占位，照样取得出）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CurrentKeyOfAJudge {
    Keyed(ConclusionVersion),
}

/// 一块的去向：两种判器各存着的版本（`stored_versions_of_each_judge`，与 `current_keys` 同序）里有一把是这一份的，就是在这一把指纹下判过；
/// 都不是而有哪一种存着，就是只在别的指纹下判过；一种都没有就是没判过。
fn standing_of_a_block(
    stored_versions_of_each_judge: &[Vec<ConclusionVersion>],
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> StandingOfABlock {
    let judged_under_the_current_key =
        stored_versions_of_each_judge
            .iter()
            .zip(current_keys)
            .any(|(stored, (_kind, current))| match current {
                CurrentKeyOfAJudge::Keyed(version) => stored.contains(version),
            });
    if judged_under_the_current_key {
        StandingOfABlock::JudgedUnderTheCurrentKey
    } else if stored_versions_of_each_judge
        .iter()
        .any(|stored| !stored.is_empty())
    {
        StandingOfABlock::JudgedOnlyUnderAnotherKey
    } else {
        StandingOfABlock::NeverJudged
    }
}

/// 存着的一种结论一把版本对着 `current_keys` 里那一种结论的那一把：相同是这一把，不同是别的；`current_keys` 里没有这一种，没得比。
fn stored_version_against_this_binary(
    kind: &ConclusionKind,
    version: &ConclusionVersion,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> StoredVersionAgainstThisBinary {
    match current_keys
        .iter()
        .find(|(current_kind, _current)| current_kind == kind)
    {
        Some((_kind, CurrentKeyOfAJudge::Keyed(current_version))) => {
            if current_version == version {
                StoredVersionAgainstThisBinary::Current
            } else {
                StoredVersionAgainstThisBinary::Another
            }
        }
        None => StoredVersionAgainstThisBinary::NotCompared,
    }
}

/// 输出的那几行（文件头「输出」那一段的写法）：`library_name` 是连的是 prune-server 哪一处、哪一个键空间。
fn keys_lines(
    library_name: &str,
    tally: &LibraryTally,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Vec<String> {
    let counts = &tally.blocks;
    let mut lines = vec![format!(
        "CRASH_LIBRARY_KEYS\tlibrary={library_name}\trecorded_blocks={}\tjudged_under_the_current_key={}\tjudged_only_under_another_key={}\tnever_judged={}",
        counts.recorded_blocks,
        counts.judged_under_the_current_key,
        counts.judged_only_under_another_key,
        counts.never_judged
    )];
    for (kind, current) in current_keys {
        let current_text = match current {
            CurrentKeyOfAJudge::Keyed(version) => {
                hexadecimal_text(&version.0[..HEXADECIMAL_HEAD_BYTES_OF_A_VERSION])
            }
        };
        lines.push(format!(
            "CRASH_LIBRARY_KEYS_JUDGE\t{}\tcurrent_key={current_text}",
            kind.version_text()
        ));
    }
    let StateCounts {
        recorded_states,
        states_judged_under_the_current_key,
        states_judged_only_under_another_key,
        states_never_judged,
    } = tally.states;
    lines.push(format!(
        "CRASH_LIBRARY_STATES\trecorded_states={recorded_states}\tstates_judged_under_the_current_key={states_judged_under_the_current_key}\tstates_judged_only_under_another_key={states_judged_only_under_another_key}\tstates_never_judged={states_never_judged}"
    ));
    for ((kind, version), stored_version_counts) in &tally.stored_versions {
        lines.push(format!(
            "CRASH_LIBRARY_STORED_VERSION\t{}\tversion={}\tagainst_this_binary={}\tblocks={}\tblocks_recorded={}\tstates_of_the_blocks_recorded={}",
            kind.version_text(),
            hexadecimal_text(&version.0[..HEXADECIMAL_HEAD_BYTES_OF_A_VERSION]),
            stored_version_against_this_binary(kind, version, current_keys).text(),
            stored_version_counts.blocks,
            stored_version_counts.blocks_recorded,
            stored_version_counts.states_of_the_blocks_recorded
        ));
    }
    for (kind, totals) in totals_of_each_stored_kind(&tally.stored_versions) {
        lines.push(format!(
            "CRASH_LIBRARY_STORED_KIND\t{}\tversions={}\tblock_version_pairs={}\tpairs_recorded={}\tstates_of_the_pairs_recorded={}\tpairs_not_recorded={}",
            kind.version_text(),
            totals.versions,
            totals.block_version_pairs,
            totals.pairs_recorded,
            totals.states_of_the_pairs_recorded,
            totals.pairs_not_recorded
        ));
    }
    lines
}

/// 每一种结论各把版本的账加起来（按种类排）：存着的版本数一把记一，另几格是那一种各把版本对应那一格之和。只算，不读库。
fn totals_of_each_stored_kind(
    stored_versions: &BTreeMap<(ConclusionKind, ConclusionVersion), StoredVersionCounts>,
) -> BTreeMap<ConclusionKind, StoredKindTotals> {
    let mut totals_of_each_kind: BTreeMap<ConclusionKind, StoredKindTotals> = BTreeMap::new();
    // 迭代上界：存着的版本数（每种结论每把版本一份账）
    for ((kind, _version), stored_version_counts) in stored_versions {
        let totals_of_the_kind = totals_of_each_kind.entry(kind.clone()).or_default();
        totals_of_the_kind.versions += 1;
        totals_of_the_kind.block_version_pairs += stored_version_counts.blocks;
        totals_of_the_kind.pairs_recorded += stored_version_counts.blocks_recorded;
        totals_of_the_kind.states_of_the_pairs_recorded +=
            stored_version_counts.states_of_the_blocks_recorded;
        totals_of_the_kind.pairs_not_recorded += stored_version_counts.blocks_not_recorded;
    }
    totals_of_each_kind
}

/// 镜像判定在这份二进制里的那一把版本：判崩溃镜像那一种结论的版本（`crash_identity::ConclusionVersion::of_the_crash_image_judgement`），
/// 由结论种类与判器引到的被检测产品代码定，判器的代码不进键，CPU 判与 GPU 判同一把，不与设备绑定。
fn current_image_judgement_key_of_this_binary() -> CurrentKeyOfAJudge {
    CurrentKeyOfAJudge::Keyed(ConclusionVersion::of_the_crash_image_judgement())
}

/// `keys` 数的每一种判器与它在这份二进制里的那一把指纹（只有镜像判定一种，见 [`current_image_judgement_key_of_this_binary`]）。
fn current_keys_of_this_binary() -> Vec<(ConclusionKind, CurrentKeyOfAJudge)> {
    vec![(
        ConclusionKind::ImageJudgement,
        current_image_judgement_key_of_this_binary(),
    )]
}

/// `keys` 的账：录入表整表扫一遍、块属性表整表扫一遍（[`stored_conclusions_of_each_block`]），在内存里按块键对上（[`tally_of_the_library`]）。
///
/// # Errors
/// 读库失败。
fn tally_of_the_library_read_from_the_store(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Result<LibraryTally, VerdictStoreError> {
    let stored_conclusions_of_each_block = stored_conclusions_of_each_block(store)?;
    Ok(tally_of_the_library(
        &store.recorded_blocks()?,
        &stored_conclusions_of_each_block,
        current_keys,
    ))
}

/// 录入表里每一块按去向（[`standing_of_a_block`]）数块、加它的状态数；块属性表里每一种结论每一把版本数存着它的块，录入表里有的另数块、加状态数，
/// 录入表里没有的另数块（按块键查录入表，与录了的那一份分开数）。
/// `stored_conclusions_of_each_block` 是块属性表里每块存着的结论种类与版本（没有的块是一种结论都没有）。只算，不读库。
fn tally_of_the_library(
    recorded_blocks: &[(VerdictBlockKey, u64)],
    stored_conclusions_of_each_block: &StoredConclusionsOfEachBlock,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> LibraryTally {
    let recorded_block_keys: HashSet<&VerdictBlockKey> = recorded_blocks
        .iter()
        .map(|(block_key, _state_count)| block_key)
        .collect();
    let mut stored_versions: BTreeMap<(ConclusionKind, ConclusionVersion), StoredVersionCounts> =
        BTreeMap::new();
    // 迭代上界：块属性表的行数（每块几种结论几把版本就几行）
    for (block_key, stored_conclusions) in stored_conclusions_of_each_block {
        let block_is_recorded = recorded_block_keys.contains(block_key);
        for (kind, version) in stored_conclusions {
            let stored_version_counts =
                stored_versions.entry((kind.clone(), *version)).or_default();
            stored_version_counts.blocks += 1;
            if !block_is_recorded {
                stored_version_counts.blocks_not_recorded += 1;
            }
        }
    }
    let mut blocks = BlockCounts::default();
    let mut states = StateCounts::default();
    // 迭代上界：录入表的块数，每块里再是它存着的结论行数
    for (block_key, state_count_of_the_block) in recorded_blocks {
        let stored_conclusions = stored_conclusions_of_each_block
            .get(block_key)
            .map_or(&[][..], Vec::as_slice);
        let standing = standing_of_a_block(
            &versions_of_each_judge(stored_conclusions, current_keys),
            current_keys,
        );
        blocks.count(standing);
        states.count(standing, *state_count_of_the_block);
        for (kind, version) in stored_conclusions {
            let stored_version_counts = stored_versions
                .get_mut(&(kind.clone(), *version))
                .expect("这一块存着的每一把版本在数块属性表那一圈里都建了一份账");
            stored_version_counts.blocks_recorded += 1;
            stored_version_counts.states_of_the_blocks_recorded +=
                u128::from(*state_count_of_the_block);
        }
    }
    LibraryTally {
        blocks,
        states,
        stored_versions,
    }
}

/// 一块存着的结论（种类与版本）按 `current_keys` 的次序分到各判器名下；不是 `current_keys` 里哪一种判器的结论不收。
fn versions_of_each_judge(
    stored_conclusions: &[(ConclusionKind, ConclusionVersion)],
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Vec<Vec<ConclusionVersion>> {
    let mut versions_of_each_judge = vec![Vec::new(); current_keys.len()];
    for (kind, version) in stored_conclusions {
        if let Some(judge_position) = current_keys
            .iter()
            .position(|(current_kind, _current)| current_kind == kind)
        {
            versions_of_each_judge[judge_position].push(*version);
        }
    }
    versions_of_each_judge
}

/// 块属性表整表扫一遍（[`VerdictStore::visit_stored_conclusion_keys`]），收每块存着的结论种类与版本（每种判器都收）。
///
/// # Errors
/// 读库失败。
fn stored_conclusions_of_each_block(
    store: &VerdictStore,
) -> Result<StoredConclusionsOfEachBlock, VerdictStoreError> {
    let mut stored_conclusions_of_each_block = StoredConclusionsOfEachBlock::new();
    store.visit_stored_conclusion_keys(&mut |(block_key, kind, version)| {
        stored_conclusions_of_each_block
            .entry(block_key)
            .or_default()
            .push((kind, version));
        Ok(())
    })?;
    Ok(stored_conclusions_of_each_block)
}

/// 每块各判器存着的版本（与 `current_keys` 同序，[`versions_of_each_judge`]）：块属性表整表扫一遍（[`stored_conclusions_of_each_block`]）。
///
/// # Errors
/// 读库失败。
fn stored_versions_of_each_block(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Result<HashMap<VerdictBlockKey, Vec<Vec<ConclusionVersion>>>, VerdictStoreError> {
    Ok(stored_conclusions_of_each_block(store)?
        .into_iter()
        .map(|(block_key, stored_conclusions)| {
            (
                block_key,
                versions_of_each_judge(&stored_conclusions, current_keys),
            )
        })
        .collect())
}

/// 录入表里 `is_chosen` 挑中的块按去向数（[`standing_of_a_block`]）：`stored_versions_of_each_block` 是块属性表里每块各判器存着的版本
/// （与 `current_keys` 同序），没有的块是一种结论都没有。只算，不读库。
fn counts_of_the_recorded_blocks(
    recorded_blocks: &[(VerdictBlockKey, u64)],
    stored_versions_of_each_block: &HashMap<VerdictBlockKey, Vec<Vec<ConclusionVersion>>>,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
    is_chosen: &dyn Fn(&VerdictBlockKey) -> bool,
) -> BlockCounts {
    let nothing_stored = vec![Vec::new(); current_keys.len()];
    let mut counts = BlockCounts::default();
    for (block_key, _state_count) in recorded_blocks {
        if !is_chosen(block_key) {
            continue;
        }
        let stored_versions_of_each_judge = stored_versions_of_each_block
            .get(block_key)
            .unwrap_or(&nothing_stored);
        counts.count(standing_of_a_block(
            stored_versions_of_each_judge,
            current_keys,
        ));
    }
    counts
}

/// 路径索引里设点文件是 `setting_files` 之一的那些点的复用键（十六进制，与块键头 32 字节的十六进制同一个写法）：身份的写法是
/// `<设点文件>::<点名>::<复用键十六进制>`（`crash_identity` 的 `NodeLabel::identity_text`；点名里可以有 `::`，设点文件取第一个 `::` 之前、
/// 复用键取最后一个 `::` 之后）。认不出这个写法的身份不算哪一项的。
fn reuse_keys_of_the_points_set_in(
    path_entries: &[(String, String)],
    setting_files: &[String],
) -> HashSet<String> {
    path_entries
        .iter()
        .filter_map(|(_path, identity)| {
            let (setting_file, _rest) = identity.split_once("::")?;
            let (_point_name, reuse_key_hexadecimal) = identity.rsplit_once("::")?;
            setting_files
                .iter()
                .any(|wanted| wanted == setting_file)
                .then(|| reuse_key_hexadecimal.to_string())
        })
        .collect()
}

/// 一项的块账（`item` 的输出，文件头 `item-format` 那一段的写法）：在这一把判法单元指纹下判过的能复用，别的要现判。
fn item_line(item_name: &str, counts: &BlockCounts) -> String {
    format!(
        "CRASH_LIBRARY_ITEM\titem={item_name}\trecorded_blocks={}\tto_judge={}\treusable={}",
        counts.recorded_blocks,
        counts.judged_only_under_another_key + counts.never_judged,
        counts.judged_under_the_current_key
    )
}

/// 按项数块要从库里读的三样，各整表扫一遍读进来：路径索引、录入表、块属性表里每块各判器存着的版本（与 `current_keys` 同序）。
struct ScannedLibrary {
    path_entries: Vec<(String, String)>,
    recorded_blocks: Vec<(VerdictBlockKey, u64)>,
    stored_versions_of_each_block: HashMap<VerdictBlockKey, Vec<Vec<ConclusionVersion>>>,
}

/// 把按项数块要的三样各整表扫一遍（`item` 一项扫一遍，`items` 几项共用一遍）。
///
/// # Errors
/// 读库失败。
fn scanned_library(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Result<ScannedLibrary, VerdictStoreError> {
    Ok(ScannedLibrary {
        path_entries: store.paths_with_prefix("")?,
        stored_versions_of_each_block: stored_versions_of_each_block(store, current_keys)?,
        recorded_blocks: store.recorded_blocks()?,
    })
}

/// 一项的块账，只算不读库：路径索引里这一项的设点文件的点（[`reuse_keys_of_the_points_set_in`]），录入表里落在它们复用键上的块按去向数。
fn counts_of_the_item(
    scanned: &ScannedLibrary,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
    setting_files: &[String],
) -> BlockCounts {
    let reuse_keys = reuse_keys_of_the_points_set_in(&scanned.path_entries, setting_files);
    counts_of_the_recorded_blocks(
        &scanned.recorded_blocks,
        &scanned.stored_versions_of_each_block,
        current_keys,
        &|block_key| block_is_under_one_of_the_reuse_keys(block_key, &reuse_keys),
    )
}

/// `items` 的输出：按 `items` 的次序每项一行 [`item_line`]，各项的块账都从同一遍扫库（`scanned`）里数。
fn item_lines_of_the_scanned_library(
    scanned: &ScannedLibrary,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
    items: &[ItemAndItsSettingFiles],
) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            item_line(
                &item.item_name,
                &counts_of_the_item(scanned, current_keys, &item.setting_files),
            )
        })
        .collect()
}

/// 块键的头 32 字节（节点输入摘要，就是那个点的复用键）是不是 `reuse_keys` 里的一把。
fn block_is_under_one_of_the_reuse_keys(
    block_key: &VerdictBlockKey,
    reuse_keys: &HashSet<String>,
) -> bool {
    reuse_keys.contains(&hexadecimal_text(&block_key.node_input_digest.0))
}

/// `items` 点名的一项：合表项名与这一项的设点文件。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ItemAndItsSettingFiles {
    item_name: String,
    setting_files: Vec<String>,
}

/// 命令行点名的子命令（文件头「用法」）；每个子命令都是数块，成员名写它数的是哪些块。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Subcommand {
    /// `keys`：数库里录下的块。
    RecordedBlocksByKey,
    /// `item <合表项名> <设点文件>…`：数这一项的块。
    BlocksOfAnItem {
        item_name: String,
        setting_files: Vec<String>,
    },
    /// `items --item <合表项名> <设点文件>…`…：一遍扫库数几项的块。
    BlocksOfEachItem { items: Vec<ItemAndItsSettingFiles> },
}

/// `items` 之后的参数：一个 `--item` 起一项，后面是项名与至少一份设点文件，至少一项；项名与设点文件都不以 `-` 起头。
/// 认不出交回 `None`。
fn items_of_the_arguments(arguments: &[String]) -> Option<Vec<ItemAndItsSettingFiles>> {
    let mut items = Vec::new();
    let mut words = arguments.iter().peekable();
    while let Some(marker) = words.next() {
        if marker != "--item" {
            return None;
        }
        let item_name = words.next().filter(|name| !name.starts_with('-'))?;
        let mut setting_files = Vec::new();
        while let Some(setting_file) = words.next_if(|word| !word.starts_with('-')) {
            setting_files.push(setting_file.clone());
        }
        if setting_files.is_empty() {
            return None;
        }
        items.push(ItemAndItsSettingFiles {
            item_name: item_name.clone(),
            setting_files,
        });
    }
    (!items.is_empty()).then_some(items)
}

/// 命令行里点名的子命令。
///
/// # Errors
/// 子命令认不得、参数多了少了：都按用法错报。
fn subcommand_of_the_arguments(arguments: &[String]) -> Result<Subcommand, String> {
    let refusal = || {
        format!(
            "认不得的参数 {arguments:?}：用法 crash_library_reader keys，或 crash_library_reader item <合表项名> <设点文件>…，或 crash_library_reader items --item <合表项名> <设点文件>… [--item …]…"
        )
    };
    match arguments {
        [name] if name == "keys" => Ok(Subcommand::RecordedBlocksByKey),
        [name, item_name, setting_files @ ..]
            if name == "item" && !item_name.starts_with('-') && !setting_files.is_empty() =>
        {
            Ok(Subcommand::BlocksOfAnItem {
                item_name: item_name.clone(),
                setting_files: setting_files.to_vec(),
            })
        }
        [name, item_arguments @ ..] if name == "items" => items_of_the_arguments(item_arguments)
            .map(|items| Subcommand::BlocksOfEachItem { items })
            .ok_or_else(refusal),
        _ => Err(refusal()),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = match subcommand_of_the_arguments(&arguments) {
        Ok(subcommand) => subcommand,
        Err(refusal) => {
            eprintln!("  ✗ {refusal}");
            eprintln!("  → 怎么办：写成 crash_library_reader keys、crash_library_reader item <合表项名> <设点文件>…，或 crash_library_reader items --item <合表项名> <设点文件>… [--item …]…（库是 prune-server 里那一份，不收库目录）");
            return ExitCode::from(2);
        }
    };
    match subcommand {
        Subcommand::RecordedBlocksByKey => print_the_lines_read_from_the_library(|store| {
            let current_keys = current_keys_of_this_binary();
            let tally = tally_of_the_library_read_from_the_store(store, &current_keys)?;
            Ok(keys_lines(store.library_name(), &tally, &current_keys))
        }),
        Subcommand::BlocksOfAnItem {
            item_name,
            setting_files,
        } => print_the_lines_read_from_the_library(|store| {
            let current_keys = current_keys_of_this_binary();
            let scanned = scanned_library(store, &current_keys)?;
            let counts = counts_of_the_item(&scanned, &current_keys, &setting_files);
            Ok(vec![item_line(&item_name, &counts)])
        }),
        Subcommand::BlocksOfEachItem { items } => print_the_lines_read_from_the_library(|store| {
            let current_keys = current_keys_of_this_binary();
            let scanned = scanned_library(store, &current_keys)?;
            Ok(item_lines_of_the_scanned_library(
                &scanned,
                &current_keys,
                &items,
            ))
        }),
    }
}

/// 开 prune-server 里那一份、照 `lines_of_the_library` 读出要打的行、打到标准输出；读库失败一行都不打。
fn print_the_lines_read_from_the_library(
    lines_of_the_library: impl FnOnce(&VerdictStore) -> Result<Vec<String>, VerdictStoreError>,
) -> ExitCode {
    let library = match LibraryOpenInThisProcess::open_or_share() {
        Ok(library) => library,
        Err(refusal) => {
            eprintln!("  ✗ {refusal}");
            eprintln!("  → 怎么办：库不是这一版的布局的，查 prune-server 里那一份是怎么写进去的（不迁移）；别的错照报的修");
            return ExitCode::from(1);
        }
    };
    let store = library.lock();
    match lines_of_the_library(&store) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("  ✗ 读库 {} 失败：{error:?}", store.library_name());
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 文件头 `<段名>-begin`、`<段名>-end` 两行标记之间的那一段（去掉行首的 `//! `）：`keys-format` 那一段脚本的自证照样喂给
    /// `crash-point-replay.sh --list-items --with-keys`。
    fn format_lines_of_the_file_header(section_name: &str) -> Vec<String> {
        let source = include_str!("crash_library_reader.rs");
        let begin_marker = format!("//! {section_name}-begin");
        let end_marker = format!("//! {section_name}-end");
        source
            .lines()
            .skip_while(|line| *line != begin_marker)
            .skip(1)
            .take_while(|line| *line != end_marker)
            .map(|line| line.trim_start_matches("//! ").to_string())
            .collect()
    }

    /// 写法与读法钉在同一段字上：照文件头那一段的账与几把指纹格式化出来的行，与文件头那一段逐字相同。
    /// 判别力（`crates/mutations.tsv`）：块账那一行、态账那一行、存着的版本那几行或每种结论的合计那几行的字段次序或字段名改了而文件头没跟着改
    /// （脚本照文件头读），这条红。
    #[test]
    fn the_keys_lines_are_the_lines_the_file_header_shows_the_script() {
        let lines = keys_lines(
            "prune-server unix:/run/singlefs-prune-server/server.sock",
            &LibraryTally {
                blocks: BlockCounts {
                    recorded_blocks: 6,
                    judged_under_the_current_key: 2,
                    judged_only_under_another_key: 3,
                    never_judged: 1,
                },
                states: StateCounts {
                    recorded_states: 20497,
                    states_judged_under_the_current_key: 4113,
                    states_judged_only_under_another_key: 12288,
                    states_never_judged: 4096,
                },
                stored_versions: [
                    (
                        (ConclusionKind::ImageJudgement, ConclusionVersion([5; 32])),
                        StoredVersionCounts {
                            blocks: 2,
                            blocks_recorded: 2,
                            states_of_the_blocks_recorded: 4113,
                            blocks_not_recorded: 0,
                        },
                    ),
                    (
                        (
                            ConclusionKind::ImageJudgement,
                            ConclusionVersion([0x0a; 32]),
                        ),
                        StoredVersionCounts {
                            blocks: 4,
                            blocks_recorded: 3,
                            states_of_the_blocks_recorded: 12288,
                            blocks_not_recorded: 1,
                        },
                    ),
                    (
                        (ConclusionKind::Observer, ConclusionVersion([0x0c; 32])),
                        StoredVersionCounts {
                            blocks: 1,
                            blocks_recorded: 1,
                            states_of_the_blocks_recorded: 4096,
                            blocks_not_recorded: 0,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
            },
            &[(
                ConclusionKind::ImageJudgement,
                CurrentKeyOfAJudge::Keyed(ConclusionVersion([5; 32])),
            )],
        );
        assert_eq!(lines, format_lines_of_the_file_header("keys-format"));
    }

    /// 造的小库，三样就是 [`tally_of_the_library`] 的三个参数。
    struct SmallLibrary {
        recorded_blocks: Vec<(VerdictBlockKey, u64)>,
        stored_conclusions_of_each_block: StoredConclusionsOfEachBlock,
        current_keys: [(ConclusionKind, CurrentKeyOfAJudge); 1],
    }

    /// 造的小库：这一把下判过的两块（一块另存着别的一把）、只在别的一把下判过的一块、只有观察者结论的一块、什么都没存的一块，
    /// 再加一块录入表里没有、只在块属性表里存着别的一把的。
    fn small_library_with_every_standing() -> SmallLibrary {
        let current = ConclusionVersion([1; 32]);
        let another = ConclusionVersion([2; 32]);
        let observer = ConclusionVersion([3; 32]);
        let block_under = |reuse_key_byte: u8, block_start: u64| VerdictBlockKey {
            node_input_digest: singlefs_verification::verdict_store::NodeInputDigest(
                [reuse_key_byte; 32],
            ),
            enumeration_plan_hash: singlefs_verification::verdict_store::EnumerationPlanHash(
                [9; 32],
            ),
            block_start: singlefs_verification::verdict_store::BlockStartStateIndex(block_start),
        };
        let recorded_blocks = vec![
            (block_under(0xaa, 0), 4096),
            (block_under(0xaa, 4096), 4096),
            (block_under(0xaa, 8192), 17),
            (block_under(0xbb, 0), 300),
            (block_under(0xbb, 300), 3),
        ];
        let stored_conclusions_of_each_block = [
            (
                block_under(0xaa, 0),
                vec![(ConclusionKind::ImageJudgement, current)],
            ),
            (
                block_under(0xaa, 4096),
                vec![
                    (ConclusionKind::ImageJudgement, another),
                    (ConclusionKind::ImageJudgement, current),
                ],
            ),
            (
                block_under(0xaa, 8192),
                vec![(ConclusionKind::ImageJudgement, another)],
            ),
            (
                block_under(0xbb, 0),
                vec![(ConclusionKind::Observer, observer)],
            ),
            (
                block_under(0xcc, 0),
                vec![(ConclusionKind::ImageJudgement, another)],
            ),
        ]
        .into_iter()
        .collect();
        SmallLibrary {
            recorded_blocks,
            stored_conclusions_of_each_block,
            current_keys: [(
                ConclusionKind::ImageJudgement,
                CurrentKeyOfAJudge::Keyed(current),
            )],
        }
    }

    /// 录下的态按所在那一块的去向分三类：这一把下判过的两块 4096 + 4096、只在别的一把下判过的一块 17、没有镜像判定的两块 300 + 3
    /// （只有观察者结论的那一块也算没判过，同块账的口径）；三类之和等于录下的态，块账照旧。
    /// 判别力（`crates/mutations.tsv`）：一类的态记进别的类、录下的态不加、观察者的结论也算成镜像判定的，这条红。
    #[test]
    fn recorded_states_split_into_the_three_standings_by_the_state_count_of_each_block() {
        let SmallLibrary {
            recorded_blocks,
            stored_conclusions_of_each_block,
            current_keys,
        } = small_library_with_every_standing();
        let tally = tally_of_the_library(
            &recorded_blocks,
            &stored_conclusions_of_each_block,
            &current_keys,
        );
        assert_eq!(
            tally.states,
            StateCounts {
                recorded_states: 4096 + 4096 + 17 + 300 + 3,
                states_judged_under_the_current_key: 4096 + 4096,
                states_judged_only_under_another_key: 17,
                states_never_judged: 300 + 3,
            }
        );
        assert_eq!(
            tally.states.recorded_states,
            tally.states.states_judged_under_the_current_key
                + tally.states.states_judged_only_under_another_key
                + tally.states.states_never_judged,
            "三类的态之和等于录下的态"
        );
        assert_eq!(
            tally.blocks,
            BlockCounts {
                recorded_blocks: 5,
                judged_under_the_current_key: 2,
                judged_only_under_another_key: 1,
                never_judged: 2,
            }
        );
    }

    /// 存着的每一种结论每一把版本各一份账：这一把存在两块上（都录了，4096 + 4096 态）；别的一把存在三块上，录了的两块 4096 + 17 态，
    /// 录入表里没有的那一块数进存着它的块与录入表里没有的块；观察者那一把一块 300 态。各份态数之和等于录入表每块状态数乘它存着的版本数再加起来（在测试里另算），
    /// 这一把那一份的态数等于这一把下判过的态（只有一种判器时）。
    /// 判别力（`crates/mutations.tsv`）：录入表里没有的块也算录了、录了的块不数、态数加成块数、存着它的块不数、
    /// 录入表里没有的块不数或数成录了的，这条红。
    #[test]
    fn each_stored_version_counts_its_blocks_and_the_states_of_its_recorded_blocks() {
        let SmallLibrary {
            recorded_blocks,
            stored_conclusions_of_each_block,
            current_keys,
        } = small_library_with_every_standing();
        let tally = tally_of_the_library(
            &recorded_blocks,
            &stored_conclusions_of_each_block,
            &current_keys,
        );
        let expected: BTreeMap<(ConclusionKind, ConclusionVersion), StoredVersionCounts> = [
            (
                (ConclusionKind::ImageJudgement, ConclusionVersion([1; 32])),
                StoredVersionCounts {
                    blocks: 2,
                    blocks_recorded: 2,
                    states_of_the_blocks_recorded: 4096 + 4096,
                    blocks_not_recorded: 0,
                },
            ),
            (
                (ConclusionKind::ImageJudgement, ConclusionVersion([2; 32])),
                StoredVersionCounts {
                    blocks: 3,
                    blocks_recorded: 2,
                    states_of_the_blocks_recorded: 4096 + 17,
                    blocks_not_recorded: 1,
                },
            ),
            (
                (ConclusionKind::Observer, ConclusionVersion([3; 32])),
                StoredVersionCounts {
                    blocks: 1,
                    blocks_recorded: 1,
                    states_of_the_blocks_recorded: 300,
                    blocks_not_recorded: 0,
                },
            ),
        ]
        .into_iter()
        .collect();
        assert_eq!(tally.stored_versions, expected);
        let states_times_the_versions_stored: u128 = recorded_blocks
            .iter()
            .map(|(block_key, state_count)| {
                let versions_stored = stored_conclusions_of_each_block
                    .get(block_key)
                    .map_or(0, Vec::len);
                u128::from(*state_count)
                    * u128::try_from(versions_stored).expect("几把版本放得进 u128")
            })
            .sum();
        assert_eq!(
            tally
                .stored_versions
                .values()
                .map(|counts| counts.states_of_the_blocks_recorded)
                .sum::<u128>(),
            states_times_the_versions_stored,
            "各份态数之和等于录入表每块状态数乘它存着的版本数再加起来"
        );
        assert_eq!(
            tally.stored_versions[&(ConclusionKind::ImageJudgement, ConclusionVersion([1; 32]))]
                .states_of_the_blocks_recorded,
            tally.states.states_judged_under_the_current_key,
            "这一把那一份的态数等于这一把下判过的态"
        );
    }

    /// 存着的一把版本对着这份二进制：镜像判定存着这一份的那一把是 current，别的一把是 another；观察者（`keys` 不数的种类）存着的是 not_compared。
    /// 判别力（`crates/mutations.tsv`）：相同与不同判反、`keys` 不数的种类算成 another，这条红。
    #[test]
    fn a_stored_version_is_current_another_or_not_compared_against_this_binary() {
        let current = ConclusionVersion([1; 32]);
        let another = ConclusionVersion([2; 32]);
        let current_keys = [(
            ConclusionKind::ImageJudgement,
            CurrentKeyOfAJudge::Keyed(current),
        )];
        assert_eq!(
            stored_version_against_this_binary(
                &ConclusionKind::ImageJudgement,
                &current,
                &current_keys
            ),
            StoredVersionAgainstThisBinary::Current
        );
        assert_eq!(
            stored_version_against_this_binary(
                &ConclusionKind::ImageJudgement,
                &another,
                &current_keys
            ),
            StoredVersionAgainstThisBinary::Another
        );
        assert_eq!(
            stored_version_against_this_binary(&ConclusionKind::Observer, &current, &current_keys),
            StoredVersionAgainstThisBinary::NotCompared,
            "keys 不数的种类没得比"
        );
    }

    /// 每一种结论一份合计，各格等于那一种各把版本那几份账对应那一格之和（在测试里逐份另加）：镜像判定两把版本、5 个块×版本对，
    /// 块在录入表里的 4 对（这一把两块 4096 + 4096 态，别的一把两块 4096 + 17 态），块不在录入表里的 1 对（只存着别的一把、录入表里没有的那一块）；
    /// 观察者一把版本、1 对、在录入表里、300 态。块在与不在录入表里的对数之和等于块×版本对数。
    /// 判别力（`crates/mutations.tsv`）：版本不数、对数不加、录了的对数不加、态数不加、不在录入表里的对数不加，这条红。
    #[test]
    fn each_stored_kind_totals_the_counts_of_its_stored_versions() {
        let SmallLibrary {
            recorded_blocks,
            stored_conclusions_of_each_block,
            current_keys,
        } = small_library_with_every_standing();
        let tally = tally_of_the_library(
            &recorded_blocks,
            &stored_conclusions_of_each_block,
            &current_keys,
        );
        let totals = totals_of_each_stored_kind(&tally.stored_versions);
        let expected: BTreeMap<ConclusionKind, StoredKindTotals> = [
            (
                ConclusionKind::ImageJudgement,
                StoredKindTotals {
                    versions: 2,
                    block_version_pairs: 2 + 3,
                    pairs_recorded: 2 + 2,
                    states_of_the_pairs_recorded: 4096 + 4096 + 4096 + 17,
                    pairs_not_recorded: 1,
                },
            ),
            (
                ConclusionKind::Observer,
                StoredKindTotals {
                    versions: 1,
                    block_version_pairs: 1,
                    pairs_recorded: 1,
                    states_of_the_pairs_recorded: 300,
                    pairs_not_recorded: 0,
                },
            ),
        ]
        .into_iter()
        .collect();
        assert_eq!(totals, expected);
        for (kind, totals_of_the_kind) in &totals {
            let stored_versions_of_the_kind: Vec<&StoredVersionCounts> = tally
                .stored_versions
                .iter()
                .filter(|((stored_kind, _version), _counts)| stored_kind == kind)
                .map(|(_kind_and_version, counts)| counts)
                .collect();
            assert_eq!(
                totals_of_the_kind.versions,
                u64::try_from(stored_versions_of_the_kind.len()).expect("几把版本放得进 u64"),
                "{kind:?} 存着几把版本是它的版本账份数"
            );
            assert_eq!(
                totals_of_the_kind.block_version_pairs,
                stored_versions_of_the_kind
                    .iter()
                    .map(|counts| counts.blocks)
                    .sum::<u64>(),
                "{kind:?} 块×版本对数是各把版本存着它的块数之和"
            );
            assert_eq!(
                totals_of_the_kind.pairs_recorded,
                stored_versions_of_the_kind
                    .iter()
                    .map(|counts| counts.blocks_recorded)
                    .sum::<u64>(),
                "{kind:?} 块在录入表里的对数是各把版本录了的块数之和"
            );
            assert_eq!(
                totals_of_the_kind.states_of_the_pairs_recorded,
                stored_versions_of_the_kind
                    .iter()
                    .map(|counts| counts.states_of_the_blocks_recorded)
                    .sum::<u128>(),
                "{kind:?} 这些对的态数是各把版本录了的块的态数之和"
            );
            assert_eq!(
                totals_of_the_kind.block_version_pairs,
                totals_of_the_kind.pairs_recorded + totals_of_the_kind.pairs_not_recorded,
                "{kind:?} 块在与不在录入表里的对数之和等于块×版本对数"
            );
        }
    }

    /// `item` 那一行与文件头 `item-format` 那一段逐字相同（入口照文件头那一段读）：要现判是另两格之和、能复用是这一把下判过的。
    /// 判别力（`crates/mutations.tsv`）：要现判与能复用两格写反，这条红。
    #[test]
    fn the_item_line_is_the_line_the_file_header_shows_the_script() {
        let line = item_line(
            "acquisition-barrier",
            &BlockCounts {
                recorded_blocks: 5,
                judged_under_the_current_key: 2,
                judged_only_under_another_key: 2,
                never_judged: 1,
            },
        );
        assert_eq!(vec![line], format_lines_of_the_file_header("item-format"));
    }

    /// 按项读法：两项的设点文件互不相交、合起来罩住录入表里每一块时，各项能复用之和等于整库在这一把下判过的，要现判之和等于整库另两格之和；
    /// 一项只数它那几把复用键上的块（点名里带 `::` 的身份照样认得出设点文件与复用键）。
    /// 判别力（`crates/mutations.tsv`）：按项挑块不看复用键（整库都算这一项的），这条红。
    #[test]
    fn the_blocks_of_each_item_add_up_to_the_counts_of_the_whole_library() {
        let current = ConclusionVersion([1; 32]);
        let another = ConclusionVersion([2; 32]);
        let current_keys = [(
            ConclusionKind::ImageJudgement,
            CurrentKeyOfAJudge::Keyed(current),
        )];
        let block_under = |reuse_key_byte: u8, block_start: u64| VerdictBlockKey {
            node_input_digest: singlefs_verification::verdict_store::NodeInputDigest(
                [reuse_key_byte; 32],
            ),
            enumeration_plan_hash: singlefs_verification::verdict_store::EnumerationPlanHash(
                [9; 32],
            ),
            block_start: singlefs_verification::verdict_store::BlockStartStateIndex(block_start),
        };
        let barrier_file =
            "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs";
        let spill_over_file =
            "crates/singlefs-verification/tests/crash_enumeration_record_spill_over_stream.rs";
        let path_entries = vec![
            (
                "acquisition_barrier::remount_after_the_first_file".to_string(),
                format!(
                    "{barrier_file}::acquisition_barrier::remount_after_the_first_file::{}",
                    hexadecimal_text(&[0xaa; 32])
                ),
            ),
            (
                "record_spill_over::two_records_whole_or_not_at_all".to_string(),
                format!(
                    "{spill_over_file}::record_spill_over::two_records_whole_or_not_at_all::{}",
                    hexadecimal_text(&[0xbb; 32])
                ),
            ),
        ];
        // 取号屏障那一项三块：一块这一把下判过、一块只在别的指纹下判过、一块没判过；记录跨记录那一项两块：一块判过、一块没判过
        let recorded_blocks = vec![
            (block_under(0xaa, 0), 4096),
            (block_under(0xaa, 4096), 4096),
            (block_under(0xaa, 8192), 17),
            (block_under(0xbb, 0), 4096),
            (block_under(0xbb, 4096), 3),
        ];
        let stored_versions_of_each_block: HashMap<VerdictBlockKey, Vec<Vec<ConclusionVersion>>> =
            [
                (block_under(0xaa, 0), vec![vec![current]]),
                (block_under(0xaa, 4096), vec![vec![another]]),
                (block_under(0xbb, 0), vec![vec![another, current]]),
            ]
            .into_iter()
            .collect();
        let counts_of_the_files = |setting_files: &[&str]| {
            let reuse_keys = reuse_keys_of_the_points_set_in(
                &path_entries,
                &setting_files
                    .iter()
                    .map(|file| (*file).to_string())
                    .collect::<Vec<String>>(),
            );
            counts_of_the_recorded_blocks(
                &recorded_blocks,
                &stored_versions_of_each_block,
                &current_keys,
                &|block_key| block_is_under_one_of_the_reuse_keys(block_key, &reuse_keys),
            )
        };
        let whole_library = counts_of_the_recorded_blocks(
            &recorded_blocks,
            &stored_versions_of_each_block,
            &current_keys,
            &|_block_key| true,
        );
        let barrier = counts_of_the_files(&[barrier_file]);
        let spill_over = counts_of_the_files(&[spill_over_file]);
        assert_eq!(
            item_line("acquisition-barrier", &barrier),
            "CRASH_LIBRARY_ITEM\titem=acquisition-barrier\trecorded_blocks=3\tto_judge=2\treusable=1",
            "取号屏障那一项只数它那一把复用键上的三块"
        );
        assert_eq!(
            item_line("record-spill-over", &spill_over),
            "CRASH_LIBRARY_ITEM\titem=record-spill-over\trecorded_blocks=2\tto_judge=1\treusable=1"
        );
        assert_eq!(
            barrier.judged_under_the_current_key + spill_over.judged_under_the_current_key,
            whole_library.judged_under_the_current_key,
            "各项能复用之和等于整库在这一把下判过的"
        );
        assert_eq!(
            barrier.judged_only_under_another_key
                + barrier.never_judged
                + spill_over.judged_only_under_another_key
                + spill_over.never_judged,
            whole_library.judged_only_under_another_key + whole_library.never_judged,
            "各项要现判之和等于整库另两格之和"
        );
        assert_eq!(
            counts_of_the_files(&["crates/singlefs-verification/tests/not_a_setting_file.rs"]),
            BlockCounts::default(),
            "设点文件对不上任何一个点：一块都不算这一项的"
        );
    }

    /// `items` 一遍扫库报几项：输出与逐项各起一次 `item`（每次各扫一遍同一份库、照 `item` 那一支数一项）打的行依次拼起来逐字相同。
    /// 几项里有设点文件共用的（两项都数）、有一项几份设点文件的、有设点文件对不上任何点的（零块）。
    /// 判别力（`crates/mutations.tsv`）：几项都按头一项的设点文件数，这条红。
    #[test]
    fn the_items_lines_are_the_item_lines_of_each_item_one_after_another() {
        let current = ConclusionVersion([1; 32]);
        let another = ConclusionVersion([2; 32]);
        let current_keys = [(
            ConclusionKind::ImageJudgement,
            CurrentKeyOfAJudge::Keyed(current),
        )];
        let block_under = |reuse_key_byte: u8, block_start: u64| VerdictBlockKey {
            node_input_digest: singlefs_verification::verdict_store::NodeInputDigest(
                [reuse_key_byte; 32],
            ),
            enumeration_plan_hash: singlefs_verification::verdict_store::EnumerationPlanHash(
                [9; 32],
            ),
            block_start: singlefs_verification::verdict_store::BlockStartStateIndex(block_start),
        };
        let barrier_file =
            "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs";
        let spill_over_file =
            "crates/singlefs-verification/tests/crash_enumeration_record_spill_over_stream.rs";
        let scanned = ScannedLibrary {
            path_entries: vec![
                (
                    "acquisition_barrier::remount_after_the_first_file".to_string(),
                    format!(
                        "{barrier_file}::acquisition_barrier::remount_after_the_first_file::{}",
                        hexadecimal_text(&[0xaa; 32])
                    ),
                ),
                (
                    "record_spill_over::two_records_whole_or_not_at_all".to_string(),
                    format!(
                        "{spill_over_file}::record_spill_over::two_records_whole_or_not_at_all::{}",
                        hexadecimal_text(&[0xbb; 32])
                    ),
                ),
            ],
            recorded_blocks: vec![
                (block_under(0xaa, 0), 4096),
                (block_under(0xaa, 4096), 4096),
                (block_under(0xaa, 8192), 17),
                (block_under(0xbb, 0), 4096),
                (block_under(0xbb, 4096), 3),
                (block_under(0xcc, 0), 4096),
            ],
            stored_versions_of_each_block: [
                (block_under(0xaa, 0), vec![vec![current]]),
                (block_under(0xaa, 4096), vec![vec![another]]),
                (block_under(0xbb, 0), vec![vec![another, current]]),
                (block_under(0xcc, 0), vec![vec![current]]),
            ]
            .into_iter()
            .collect(),
        };
        let item = |item_name: &str, setting_files: &[&str]| ItemAndItsSettingFiles {
            item_name: item_name.to_string(),
            setting_files: setting_files
                .iter()
                .map(|file| (*file).to_string())
                .collect(),
        };
        let items = vec![
            item("acquisition-barrier", &[barrier_file]),
            item("record-spill-over", &[spill_over_file]),
            item("both-setting-files", &[barrier_file, spill_over_file]),
            item(
                "not-a-setting-file",
                &["crates/singlefs-verification/tests/not_a_setting_file.rs"],
            ),
            item("acquisition-barrier-again", &[barrier_file]),
        ];
        let item_by_item: Vec<String> = items
            .iter()
            .map(|each| {
                item_line(
                    &each.item_name,
                    &counts_of_the_item(&scanned, &current_keys, &each.setting_files),
                )
            })
            .collect();
        assert_eq!(
            item_lines_of_the_scanned_library(&scanned, &current_keys, &items),
            item_by_item,
            "一遍扫库报几项，与逐项各起一次 item 打的行依次拼起来逐字相同"
        );
        assert_eq!(
            item_by_item,
            vec![
                "CRASH_LIBRARY_ITEM\titem=acquisition-barrier\trecorded_blocks=3\tto_judge=2\treusable=1",
                "CRASH_LIBRARY_ITEM\titem=record-spill-over\trecorded_blocks=2\tto_judge=1\treusable=1",
                "CRASH_LIBRARY_ITEM\titem=both-setting-files\trecorded_blocks=5\tto_judge=3\treusable=2",
                "CRASH_LIBRARY_ITEM\titem=not-a-setting-file\trecorded_blocks=0\tto_judge=0\treusable=0",
                "CRASH_LIBRARY_ITEM\titem=acquisition-barrier-again\trecorded_blocks=3\tto_judge=2\treusable=1",
            ],
            "逐项各自只数自己那几把复用键上的块（复用键对不上任何一项的那一块哪一项都不算）"
        );
    }

    /// `items` 收一组或几组 `--item <项名> <设点文件>…`，次序照命令行；不以 `--item` 起头、项名写成选项、一组里没有设点文件、
    /// 一组都没有、组之间夹了别的选项，都是用法错。
    /// 判别力：一组里没有设点文件也收下，这条红。
    #[test]
    fn the_items_subcommand_takes_groups_of_an_item_name_and_its_setting_files() {
        let arguments = |words: &[&str]| -> Vec<String> {
            words.iter().map(|word| (*word).to_string()).collect()
        };
        let barrier_file =
            "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs";
        let spill_over_file =
            "crates/singlefs-verification/tests/crash_enumeration_record_spill_over_stream.rs";
        assert_eq!(
            subcommand_of_the_arguments(&arguments(&[
                "items",
                "--item",
                "acquisition-barrier",
                barrier_file,
                "--item",
                "both-setting-files",
                barrier_file,
                spill_over_file,
            ])),
            Ok(Subcommand::BlocksOfEachItem {
                items: vec![
                    ItemAndItsSettingFiles {
                        item_name: "acquisition-barrier".to_string(),
                        setting_files: vec![barrier_file.to_string()],
                    },
                    ItemAndItsSettingFiles {
                        item_name: "both-setting-files".to_string(),
                        setting_files: vec![barrier_file.to_string(), spill_over_file.to_string()],
                    },
                ],
            })
        );
        for refused in [
            arguments(&["items"]),
            arguments(&["items", "acquisition-barrier", barrier_file]),
            arguments(&["items", "--item", "acquisition-barrier"]),
            arguments(&[
                "items",
                "--item",
                "acquisition-barrier",
                "--item",
                "record-spill-over",
                spill_over_file,
            ]),
            arguments(&["items", "--item", "--library", barrier_file]),
            arguments(&[
                "items",
                "--item",
                "acquisition-barrier",
                barrier_file,
                "--library",
                "/library/x",
            ]),
        ] {
            assert!(
                subcommand_of_the_arguments(&refused).is_err(),
                "{refused:?} 是用法错"
            );
        }
    }

    /// 一块的去向：镜像判定存着这一份的那一把版本就是在这一把下判过（不分 CPU 还是 GPU 判的）；只存着别的版本是键对不上；
    /// 什么都没有是没判过。
    #[test]
    fn block_is_judged_under_the_current_key_only_when_the_judgement_stored_this_binarys_fingerprint(
    ) {
        let current = ConclusionVersion([1; 32]);
        let another = ConclusionVersion([2; 32]);
        let current_keys = [(
            ConclusionKind::ImageJudgement,
            CurrentKeyOfAJudge::Keyed(current),
        )];
        assert_eq!(
            standing_of_a_block(&[vec![another, current]], &current_keys),
            StandingOfABlock::JudgedUnderTheCurrentKey
        );
        assert_eq!(
            standing_of_a_block(&[vec![another]], &current_keys),
            StandingOfABlock::JudgedOnlyUnderAnotherKey
        );
        assert_eq!(
            standing_of_a_block(&[Vec::new()], &current_keys),
            StandingOfABlock::NeverJudged
        );
    }

    /// 只认三个子命令：`keys` 不带参数；`item` 带项名与至少一份设点文件；`items` 带至少一组 `--item <项名> <设点文件>…`。
    /// 别的子命令、`keys` 多给参数（`--library <路径>` 这类选库的参数也算）、`item` 不给设点文件或项名写成选项、不给参数都是用法错。
    /// 判别力：`--library <路径>` 照旧收下（选库的参数），这条红。
    #[test]
    fn only_the_keys_and_item_subcommands_with_their_arguments_are_taken() {
        let arguments = |words: &[&str]| -> Vec<String> {
            words.iter().map(|word| (*word).to_string()).collect()
        };
        assert_eq!(
            subcommand_of_the_arguments(&arguments(&["keys"])),
            Ok(Subcommand::RecordedBlocksByKey)
        );
        assert_eq!(
            subcommand_of_the_arguments(&arguments(&[
                "item",
                "acquisition-barrier",
                "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs",
            ])),
            Ok(Subcommand::BlocksOfAnItem {
                item_name: "acquisition-barrier".to_string(),
                setting_files: vec![
                    "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs"
                        .to_string()
                ],
            })
        );
        for refused in [
            arguments(&["keys", "--library", "/library/x"]),
            arguments(&["keys", "extra"]),
            arguments(&["item", "acquisition-barrier"]),
            arguments(&["item", "--library", "/library/x"]),
            arguments(&["item"]),
            arguments(&["adopt"]),
            arguments(&["import-into-prune-server"]),
            arguments(&["blocks"]),
            Vec::new(),
        ] {
            assert!(
                subcommand_of_the_arguments(&refused).is_err(),
                "{refused:?} 是用法错"
            );
        }
    }
}
