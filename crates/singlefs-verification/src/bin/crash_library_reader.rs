//! 崩溃判定库的读库程序（`records/2026-10-13-复用键与全仓改名.md` crates 红批第 4 条；
//! 4-2a 报告第四节第 10 条：键对不上的条数要读判定库，python 不读它的布局）。子命令 `keys` 读 prune-server 里那一份崩溃判定库
//! （`crash_library::LibraryOpenInThisProcess::open_or_share`：连哪一处照 `crash_library::prune_server_route_of_the_configuration`；
//! 没写服务的地址、连不上、读写不成照 prune-server 客户端的退出码退出，码见本文件头「退出码」那一段），一行都不写：
//! 按编进这份二进制的判器键表（`crash_identity::compiled_judging_units_keys`）数录入表里的块：几块在这一把判法单元指纹下判过、
//! 几块只在别的指纹下判过（键对不上，下一趟要现判）、几块哪一种判器的结论都没有。
//!
//! 读法（不逐块往返）：录入表整表逐帧扫一遍（值是 8 字节的状态数），块属性表整表逐帧扫一遍收每块存着的结论版本
//! （`VerdictStore::visit_stored_conclusion_keys`；块属性与结论块同一批写、键的后缀相同、值只有几十字节，不把整块结论读回来），
//! 两份在内存里按块键对上。
//!
//! 用法：`crash_library_reader keys`；`crash_library_reader item <合表项名> <设点文件>…`。
//!
//! 子命令 `item`（按项读法）：给崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv` 的一项与这一项的设点文件（相对仓根的路径，
//! 至少一份；崩溃枚举用例与放量分项的测试目标文件 `crates/singlefs-verification/tests/<目标>.rs`，由入口照合表点名的部分现给），报这一项
//! 要现判几块、能复用几块：路径索引（`VerdictStore::paths_with_prefix`，录入段每个点登一行「路径 → <设点文件>::<点名>::<复用键十六进制>」）里
//! 设点文件是给的那几份的点，它们的复用键就是块键的头 32 字节；录入表里块键头 32 字节落在这几把复用键上的块，在这一把判法单元指纹下判过的
//! 能复用，别的（只在别的指纹下判过、哪一种结论都没有）要现判。同一个点在几项里都登过（设点文件是共用模块）就各项都数；
//! 路径索引同一路径只留最后登的身份，所以这是给任务服务估时的数，不是判定。几项的设点文件互不相交、合起来罩住录入表里每一块时，
//! 各项的能复用之和等于 `keys` 的 `judged_under_the_current_key`，要现判之和等于另两格之和。
//!
//! 输出：一行一项，制表符隔开，`<名>=<值>` 的字段次序固定。`scripts/crash/crash-point-replay.sh --list-items --with-keys` 照这一段读，
//! 它的自证 `list-items-with-keys` 格把下面两行标记之间的那一段（去掉行首的 `//! `）原样当这个程序的输出喂给它；本文件的单测钉格式化出来的行与
//! 这一段逐字相同——两边的写法与读法钉在同一段字上：
//! keys-format-begin
//! CRASH_LIBRARY_KEYS	library=prune-server unix:/run/singlefs-prune-server/server.sock	recorded_blocks=6	judged_under_the_current_key=2	judged_only_under_another_key=3	never_judged=1
//! CRASH_LIBRARY_KEYS_JUDGE	image_judgement	current_key=0505050505050505
//! keys-format-end
//!
//! `item` 的输出一行（同样钉在文件头这一段上）：
//! item-format-begin
//! CRASH_LIBRARY_ITEM	item=acquisition-barrier	recorded_blocks=5	to_judge=3	reusable=2
//! item-format-end
//!
//! 一块「在这一把指纹下判过」：CPU 判器或 GPU 判器那一种结论在编进这份二进制的那一把判法单元指纹（`current_key=` 那十六进制的头 8 字节）
//! 下存着；GPU 那一把要编进 `gpu` 特性才有（没编进是 `not_computed_at_the_build`，编译时闭包拒算是 `refused_at_the_build`）。
//! 「只在别的指纹下判过」：有哪一种判器的结论，可没有一把是这一份的。块按录入表数（录入段录下的块，键只由内容定，几条流共有的块只数一次）。
//!
//! 退出码：0 读完；1 读不了（库不是这一版的布局、读回的行不合布局）；2 用法错（子命令认不得、参数多了）；3 平常跑设了改连接地址的变量、
//! socket 不在而 TCP 地址两处都没有（`crash_library::PruneServerRouteRefusal`）、服务拒了一帧这类别的错；75 连不上 prune-server、读到一半服务停了
//! （照 prune-server 客户端的退出码表）。
#![allow(
    clippy::tabs_in_doc_comments,
    reason = "文件头 keys 输出那一段就是这个程序打的行，字段之间是制表符；脚本照原样取那一段，换成空格两边就对不上"
)]

use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

use singlefs_verification::crash_identity::{
    image_judgement_units_key, ConclusionKind, ConclusionVersion, JudgingUnitsKeyRefused,
};
use singlefs_verification::crash_library::{toolchain_of_this_process, LibraryOpenInThisProcess};
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

/// 一种判器在这份二进制里的那一把判法单元指纹：取得出，或为什么取不出（拒算的形态名）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CurrentKeyOfAJudge {
    Keyed(ConclusionVersion),
    NotComputed(&'static str),
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
                CurrentKeyOfAJudge::NotComputed(_form) => false,
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

/// 输出的那几行（文件头「输出」那一段的写法）：`library_name` 是连的是 prune-server 哪一处、哪一个键空间。
fn keys_lines(
    library_name: &str,
    counts: &BlockCounts,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Vec<String> {
    let mut lines = vec![format!(
        "CRASH_LIBRARY_KEYS\tlibrary={library_name}\trecorded_blocks={}\tjudged_under_the_current_key={}\tjudged_only_under_another_key={}\tnever_judged={}",
        counts.recorded_blocks,
        counts.judged_under_the_current_key,
        counts.judged_only_under_another_key,
        counts.never_judged
    )];
    for (kind, current) in current_keys {
        let current_text = match current {
            CurrentKeyOfAJudge::Keyed(version) => hexadecimal_text(&version.0[..8]),
            CurrentKeyOfAJudge::NotComputed(form) => (*form).to_string(),
        };
        lines.push(format!(
            "CRASH_LIBRARY_KEYS_JUDGE\t{}\tcurrent_key={current_text}",
            kind.version_text()
        ));
    }
    lines
}

/// 镜像判定在这份二进制里的那一把判法单元指纹：判定定义那一组（CPU 参照判器的闭包）与 GPU 判器那一组并成的那一把
/// （`crash_identity::image_judgement_units_key`「GPU 判器代码进键」，CPU 判与 GPU 判同一把，不与设备绑定）。
fn current_image_judgement_key_of_this_binary() -> CurrentKeyOfAJudge {
    match image_judgement_units_key(toolchain_of_this_process()) {
        Ok(key) => CurrentKeyOfAJudge::Keyed(ConclusionVersion(*key.fingerprint())),
        Err(refusal) => CurrentKeyOfAJudge::NotComputed(match refusal {
            JudgingUnitsKeyRefused::NotComputedAtTheBuild { .. } => "not_computed_at_the_build",
            JudgingUnitsKeyRefused::RefusedAtTheBuild { .. } => "refused_at_the_build",
        }),
    }
}

/// `keys` 数的每一种判器与它在这份二进制里的那一把指纹（只有镜像判定一种，见 [`current_image_judgement_key_of_this_binary`]）。
fn current_keys_of_this_binary() -> Vec<(ConclusionKind, CurrentKeyOfAJudge)> {
    vec![(
        ConclusionKind::ImageJudgement,
        current_image_judgement_key_of_this_binary(),
    )]
}

/// 数库里录下的每一块：录入表整表扫一遍、块属性表整表扫一遍（[`VerdictStore::visit_stored_conclusion_keys`]），按块键对上。
///
/// # Errors
/// 读库失败。
fn counts_of_the_library(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Result<BlockCounts, VerdictStoreError> {
    counts_of_the_blocks_chosen_by(store, current_keys, &|_block_key| true)
}

/// 同 [`counts_of_the_library`]，只数 `is_chosen` 挑中的块（`item` 按这一项那几把复用键挑）。
///
/// # Errors
/// 读库失败。
fn counts_of_the_blocks_chosen_by(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
    is_chosen: &dyn Fn(&VerdictBlockKey) -> bool,
) -> Result<BlockCounts, VerdictStoreError> {
    let mut stored_versions_of_each_block: HashMap<VerdictBlockKey, Vec<Vec<ConclusionVersion>>> =
        HashMap::new();
    store.visit_stored_conclusion_keys(&mut |(block_key, kind, version)| {
        if let Some(judge_position) = current_keys
            .iter()
            .position(|(current_kind, _current)| *current_kind == kind)
        {
            stored_versions_of_each_block
                .entry(block_key)
                .or_insert_with(|| vec![Vec::new(); current_keys.len()])[judge_position]
                .push(version);
        }
        Ok(())
    })?;
    Ok(counts_of_the_recorded_blocks(
        &store.recorded_blocks()?,
        &stored_versions_of_each_block,
        current_keys,
        is_chosen,
    ))
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

/// 一项的块账：路径索引里这一项的设点文件的点（[`reuse_keys_of_the_points_set_in`]），录入表里落在它们复用键上的块按去向数。
///
/// # Errors
/// 读库失败。
fn counts_of_the_item(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
    setting_files: &[String],
) -> Result<BlockCounts, VerdictStoreError> {
    let reuse_keys = reuse_keys_of_the_points_set_in(&store.paths_with_prefix("")?, setting_files);
    counts_of_the_blocks_chosen_by(store, current_keys, &|block_key| {
        block_is_under_one_of_the_reuse_keys(block_key, &reuse_keys)
    })
}

/// 块键的头 32 字节（节点输入摘要，就是那个点的复用键）是不是 `reuse_keys` 里的一把。
fn block_is_under_one_of_the_reuse_keys(
    block_key: &VerdictBlockKey,
    reuse_keys: &HashSet<String>,
) -> bool {
    reuse_keys.contains(&hexadecimal_text(&block_key.node_input_digest.0))
}

/// 命令行点名的子命令（文件头「用法」）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum Subcommand {
    /// `keys`：数库里录下的块。
    CountTheRecordedBlocksByKey,
    /// `item <合表项名> <设点文件>…`：数这一项的块。
    CountTheBlocksOfAnItem {
        item_name: String,
        setting_files: Vec<String>,
    },
}

/// 命令行里点名的子命令。
///
/// # Errors
/// 子命令认不得、参数多了少了：都按用法错报。
fn subcommand_of_the_arguments(arguments: &[String]) -> Result<Subcommand, String> {
    match arguments {
        [name] if name == "keys" => Ok(Subcommand::CountTheRecordedBlocksByKey),
        [name, item_name, setting_files @ ..]
            if name == "item" && !item_name.starts_with('-') && !setting_files.is_empty() =>
        {
            Ok(Subcommand::CountTheBlocksOfAnItem {
                item_name: item_name.clone(),
                setting_files: setting_files.to_vec(),
            })
        }
        _ => Err(format!(
            "认不得的参数 {arguments:?}：用法 crash_library_reader keys，或 crash_library_reader item <合表项名> <设点文件>…"
        )),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = match subcommand_of_the_arguments(&arguments) {
        Ok(subcommand) => subcommand,
        Err(refusal) => {
            eprintln!("  ✗ {refusal}");
            eprintln!("  → 怎么办：写成 crash_library_reader keys，或 crash_library_reader item <合表项名> <设点文件>…（库是 prune-server 里那一份，不收库目录）");
            return ExitCode::from(2);
        }
    };
    match subcommand {
        Subcommand::CountTheRecordedBlocksByKey => print_the_lines_read_from_the_library(|store| {
            let current_keys = current_keys_of_this_binary();
            let counts = counts_of_the_library(store, &current_keys)?;
            Ok(keys_lines(store.library_name(), &counts, &current_keys))
        }),
        Subcommand::CountTheBlocksOfAnItem {
            item_name,
            setting_files,
        } => print_the_lines_read_from_the_library(|store| {
            let current_keys = current_keys_of_this_binary();
            let counts = counts_of_the_item(store, &current_keys, &setting_files)?;
            Ok(vec![item_line(&item_name, &counts)])
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

    /// 写法与读法钉在同一段字上：照文件头那一段的账与两把指纹格式化出来的行，与文件头那一段逐字相同。
    /// 判别力（`crates/mutations.tsv`）：字段次序或字段名改了而文件头没跟着改（脚本照文件头读），这条红。
    #[test]
    fn the_keys_lines_are_the_lines_the_file_header_shows_the_script() {
        let lines = keys_lines(
            "prune-server unix:/run/singlefs-prune-server/server.sock",
            &BlockCounts {
                recorded_blocks: 6,
                judged_under_the_current_key: 2,
                judged_only_under_another_key: 3,
                never_judged: 1,
            },
            &[(
                ConclusionKind::ImageJudgement,
                CurrentKeyOfAJudge::Keyed(ConclusionVersion([5; 32])),
            )],
        );
        assert_eq!(lines, format_lines_of_the_file_header("keys-format"));
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

    /// 一块的去向：镜像判定存着这一份定义的指纹就是在这一把下判过（不分 CPU 还是 GPU 判的）；只存着别的指纹是键对不上；
    /// 什么都没有是没判过；定义那一组拒算时存着的都不算这一份的。
    #[test]
    fn a_block_is_judged_under_the_current_key_only_when_the_judgement_stored_this_binarys_fingerprint(
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
        let refused_keys = [(
            ConclusionKind::ImageJudgement,
            CurrentKeyOfAJudge::NotComputed("not_computed_at_the_build"),
        )];
        assert_eq!(
            standing_of_a_block(&[vec![current]], &refused_keys),
            StandingOfABlock::JudgedOnlyUnderAnotherKey,
            "定义那一组拒算：存着的哪一把都不算这一份的"
        );
    }

    /// 只认两个子命令：`keys` 不带参数；`item` 带项名与至少一份设点文件。别的子命令、`keys` 多给参数（连同早先的 `--library`）、
    /// `item` 不给设点文件或项名写成选项、不给参数都是用法错。
    /// 判别力：`--library <路径>` 照旧收下（选库的参数），这条红。
    #[test]
    fn only_the_keys_and_item_subcommands_with_their_arguments_are_taken() {
        let arguments = |words: &[&str]| -> Vec<String> {
            words.iter().map(|word| (*word).to_string()).collect()
        };
        assert_eq!(
            subcommand_of_the_arguments(&arguments(&["keys"])),
            Ok(Subcommand::CountTheRecordedBlocksByKey)
        );
        assert_eq!(
            subcommand_of_the_arguments(&arguments(&[
                "item",
                "acquisition-barrier",
                "crates/singlefs-verification/tests/crash_enumeration_acquisition_barrier.rs",
            ])),
            Ok(Subcommand::CountTheBlocksOfAnItem {
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
