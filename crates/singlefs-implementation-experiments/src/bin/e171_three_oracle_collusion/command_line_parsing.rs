//! 命令行的解析。

use super::*;

pub(super) struct CommandLine {
    pub(super) run_kind: RunKind,
    pub(super) output_directory: PathBuf,
    pub(super) switches: RunSwitches,
}

pub(super) struct KnownBadImagesCommandLine {
    pub(super) dump_directory: PathBuf,
    pub(super) output_directory: PathBuf,
    pub(super) with_e170_bad_rows: bool,
    pub(super) switches: RunSwitches,
}

pub(super) enum ParsedCommand {
    Dump(CommandLine),
    KnownBadImages(KnownBadImagesCommandLine),
}

pub(super) fn parse_command_line(arguments: &[String]) -> Option<ParsedCommand> {
    let mut words = arguments.iter();
    let mode = words.next()?.as_str();
    let mut output_directory = None;
    let mut dump_directory = None;
    let mut with_e170_bad_rows = false;
    let mut switches = RunSwitches::default();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--dump-directory" => dump_directory = Some(PathBuf::from(words.next()?)),
            "--e170-bad-rows" => with_e170_bad_rows = true,
            "--positive-control" => {
                switches.positive_control = Some(PositiveControl::parse(words.next()?)?);
            }
            "--disable-oracle" => {
                switches.disabled_oracle = Some(match words.next()?.as_str() {
                    "O1" => DisabledOracle::ComparisonAndCrashJudge,
                    "O2" => DisabledOracle::PoolChecker,
                    _ => return None,
                });
            }
            "--sampling-threads" => {
                switches.sampling_threads = Some(words.next()?.parse::<NonZeroUsize>().ok()?);
            }
            _ => return None,
        }
    }
    match mode {
        "baseline" | "row" => Some(ParsedCommand::Dump(CommandLine {
            run_kind: if mode == "baseline" {
                RunKind::Baseline
            } else {
                RunKind::BadImplementationRow
            },
            output_directory: output_directory?,
            switches,
        })),
        "known-bad-images" => Some(ParsedCommand::KnownBadImages(KnownBadImagesCommandLine {
            dump_directory: dump_directory?,
            output_directory: output_directory?,
            with_e170_bad_rows,
            switches,
        })),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
