//! 崩溃判定库的只读读库程序（`records/2026-10-13-复用键与全仓改名.md`「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」crates 红批第 4 条；
//! 4-2a 报告第四节第 10 条：键对不上的条数要读 RocksDB，python 读不了它）。子命令只有 `keys`：只读地开那一份长期库
//! （`VerdictStore::open_existing_for_reading_only`：不拿锁、库目录里一个文件都不动，RocksDB 的运行日志写到一份一次性目录），按编进这份二进制的
//! 判器键表（`crash_identity::compiled_judging_units_keys`）数录入表里的块：几块在这一把判法单元指纹下判过、几块只在别的指纹下判过
//! （键对不上，下一趟要现判）、几块哪一种判器的结论都没有。不写库。
//!
//! 用法：`crash_library_reader keys [--library <绝对路径>]`。没给 `--library` 照 `crash_library::crash_library_directory`
//! （`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`，没设是家目录下那一份长期库）；给的不是绝对路径就拒（仓根不是库的输入）。
//!
//! 输出：一行一项，制表符隔开，`<名>=<值>` 的字段次序固定。`scripts/crash/crash-point-replay.sh --list-items --with-keys` 照这一段读，
//! 它的自证 `list-items-with-keys` 格把下面两行标记之间的那一段（去掉行首的 `//! `）原样当这个程序的输出喂给它；本文件的单测钉格式化出来的行与
//! 这一段逐字相同——两边的写法与读法钉在同一段字上：
//! keys-format-begin
//! CRASH_LIBRARY_KEYS	library=/library/crash-state-verdicts	recorded_blocks=6	judged_under_the_current_key=2	judged_only_under_another_key=3	never_judged=1
//! CRASH_LIBRARY_KEYS_JUDGE	image_judgement	current_key=0505050505050505
//! keys-format-end
//! 库不在只打一行 `CRASH_LIBRARY_KEYS	library=<目录>	missing`（一块都没判过），退 0。
//!
//! 一块「在这一把指纹下判过」：CPU 判器或 GPU 判器那一种结论在编进这份二进制的那一把判法单元指纹（`current_key=` 那十六进制的头 8 字节）
//! 下存着；GPU 那一把要编进 `gpu` 特性才有（没编进是 `not_computed_at_the_build`，编译时闭包拒算是 `refused_at_the_build`）。
//! 「只在别的指纹下判过」：有哪一种判器的结论，可没有一把是这一份的。块按录入表数（录入段录下的块，键只由内容定，几条流共有的块只数一次）。
//!
//! 退出码：0 读完（库不在也是 0）；2 用法错；1 库在而读不了（只读开不了、布局是这一套之前的、读失败）。
#![allow(
    clippy::tabs_in_doc_comments,
    reason = "文件头「输出」那一段就是这个程序打的行，字段之间是制表符；脚本的自证照原样取那一段，换成空格两边就对不上"
)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use singlefs_checker_tier::crash_identity::{
    judging_units_key, ConclusionKind, ConclusionVersion, JudgingUnits, JudgingUnitsKeyRefused,
};
use singlefs_checker_tier::crash_library::{
    absolute_library_directory_of_the_variable_value, crash_library_directory,
    toolchain_of_this_process, DisposableLibraryDirectory,
};
use singlefs_checker_tier::verdict_store::{VerdictStore, VerdictStoreError};
use singlefs_harness::hexadecimal::hexadecimal_text;

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

/// 输出的那几行（文件头「输出」那一段的写法）。
fn keys_lines(
    library: &Path,
    counts: &BlockCounts,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Vec<String> {
    let mut lines = vec![format!(
        "CRASH_LIBRARY_KEYS\tlibrary={}\trecorded_blocks={}\tjudged_under_the_current_key={}\tjudged_only_under_another_key={}\tnever_judged={}",
        library.display(),
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

/// 库不在时那一行。
fn missing_library_line(library: &Path) -> String {
    format!("CRASH_LIBRARY_KEYS\tlibrary={}\tmissing", library.display())
}

/// 镜像判定在这份二进制里的那一把判法单元指纹：判定定义那一组（CPU 参照判器的闭包）；GPU 判器那一组只是块属性里的出处，
/// 不是版本，这里不列（用户 2026-10-17 定：数据不与设备绑定）。
fn current_keys_of_this_binary() -> Vec<(ConclusionKind, CurrentKeyOfAJudge)> {
    [(
        ConclusionKind::ImageJudgement,
        JudgingUnits::JudgementOfTheCrashImages,
    )]
    .into_iter()
    .map(
        |(kind, units)| match judging_units_key(&units, toolchain_of_this_process()) {
            Ok(key) => (
                kind,
                CurrentKeyOfAJudge::Keyed(ConclusionVersion(*key.fingerprint())),
            ),
            Err(refusal) => (
                kind,
                CurrentKeyOfAJudge::NotComputed(match refusal {
                    JudgingUnitsKeyRefused::NotComputedAtTheBuild { .. } => {
                        "not_computed_at_the_build"
                    }
                    JudgingUnitsKeyRefused::RefusedAtTheBuild { .. } => "refused_at_the_build",
                }),
            ),
        },
    )
    .collect()
}

/// 数库里录下的每一块。
///
/// # Errors
/// 读库失败。
fn counts_of_the_library(
    store: &VerdictStore,
    current_keys: &[(ConclusionKind, CurrentKeyOfAJudge)],
) -> Result<BlockCounts, VerdictStoreError> {
    let mut counts = BlockCounts::default();
    for (block_key, _state_count) in store.recorded_blocks()? {
        let stored_versions_of_each_judge = current_keys
            .iter()
            .map(|(kind, _current)| store.conclusion_versions_stored_for(&block_key, kind))
            .collect::<Result<Vec<Vec<ConclusionVersion>>, VerdictStoreError>>()?;
        counts.count(standing_of_a_block(
            &stored_versions_of_each_judge,
            current_keys,
        ));
    }
    Ok(counts)
}

/// 命令行里给的库目录（`--library` 只收绝对路径），没给照 `crash_library_directory`。
fn library_of_the_arguments(arguments: &[String]) -> Result<PathBuf, String> {
    match arguments {
        [subcommand] if subcommand == "keys" => crash_library_directory(),
        [subcommand, option, directory] if subcommand == "keys" && option == "--library" => {
            absolute_library_directory_of_the_variable_value("--library", directory)
        }
        _ => Err(format!(
            "认不得的参数 {arguments:?}：用法 crash_library_reader keys [--library <绝对路径>]"
        )),
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let library = match library_of_the_arguments(&arguments) {
        Ok(library) => library,
        Err(refusal) => {
            eprintln!("  ✗ {refusal}");
            eprintln!("  → 怎么办：写成 crash_library_reader keys，或 crash_library_reader keys --library <库目录的绝对路径>");
            return ExitCode::from(2);
        }
    };
    if !library.join("CURRENT").exists() {
        println!("{}", missing_library_line(&library));
        return ExitCode::SUCCESS;
    }
    let log_directory = match DisposableLibraryDirectory::new("library-keys-reader") {
        Ok(log_directory) => log_directory,
        Err(refusal) => {
            eprintln!("  ✗ 建不了放 RocksDB 运行日志的一次性目录：{refusal}");
            return ExitCode::from(1);
        }
    };
    let store = match VerdictStore::open_existing_for_reading_only(
        &library,
        log_directory.directory(),
    ) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("  ✗ 只读开不了库 {}：{error:?}", library.display());
            eprintln!("  → 怎么办：库是这一套之前的布局就换一个新目录建库（不迁移）；别的错照 RocksDB 报的修");
            return ExitCode::from(1);
        }
    };
    let current_keys = current_keys_of_this_binary();
    match counts_of_the_library(&store, &current_keys) {
        Ok(counts) => {
            for line in keys_lines(&library, &counts, &current_keys) {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("  ✗ 读库 {} 失败：{error:?}", library.display());
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 文件头「输出」两行标记之间的那一段（去掉行首的 `//! `）：脚本的自证照同一段喂给 `crash-point-replay.sh --list-items --with-keys`。
    fn format_lines_of_the_file_header() -> Vec<String> {
        let source = include_str!("crash_library_reader.rs");
        source
            .lines()
            .skip_while(|line| *line != "//! keys-format-begin")
            .skip(1)
            .take_while(|line| *line != "//! keys-format-end")
            .map(|line| line.trim_start_matches("//! ").to_string())
            .collect()
    }

    /// 写法与读法钉在同一段字上：照文件头那一段的账与两把指纹格式化出来的行，与文件头那一段逐字相同。
    /// 判别力（`crates/mutations.tsv`）：字段次序或字段名改了而文件头没跟着改（脚本照文件头读），这条红。
    #[test]
    fn the_keys_lines_are_the_lines_the_file_header_shows_the_script() {
        let lines = keys_lines(
            Path::new("/library/crash-state-verdicts"),
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
        assert_eq!(lines, format_lines_of_the_file_header());
        assert_eq!(
            missing_library_line(Path::new("/library/crash-state-verdicts")),
            "CRASH_LIBRARY_KEYS\tlibrary=/library/crash-state-verdicts\tmissing"
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

    /// `--library` 只收绝对路径；子命令写错是用法错。
    #[test]
    fn the_library_argument_takes_only_an_absolute_path() {
        let arguments = |words: &[&str]| -> Vec<String> {
            words.iter().map(|word| (*word).to_string()).collect()
        };
        assert_eq!(
            library_of_the_arguments(&arguments(&["keys", "--library", "/library/x"])),
            Ok(PathBuf::from("/library/x"))
        );
        assert!(
            library_of_the_arguments(&arguments(&["keys", "--library", "relative/x"])).is_err()
        );
        assert!(library_of_the_arguments(&arguments(&["blocks"])).is_err());
    }
}
