//! 命令行的解析。

use super::*;

pub(super) struct CommandLine {
    pub(super) run_kind: RunKind,
    pub(super) output_directory: PathBuf,
    pub(super) switches: RunSwitches,
    pub(super) streams: Vec<StreamName>,
}

pub(super) struct ConfirmedLossCommandLine {
    pub(super) output_directory: PathBuf,
    pub(super) truncate_at_the_return_point: bool,
}

pub(super) enum ParsedCommand {
    Dump(CommandLine),
    StreamConstants(Vec<StreamName>),
    ConfirmedLossForm(ConfirmedLossCommandLine),
}

pub(super) fn parse_command_line(arguments: &[String]) -> Option<ParsedCommand> {
    let mut words = arguments.iter();
    let mode = words.next()?.as_str();
    let mut output_directory = None;
    let mut switches = RunSwitches::default();
    let mut streams = STREAM_NAMES.to_vec();
    let mut truncate_at_the_return_point = false;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--streams" => streams = stream_names_of_the_numbers(words.next()?)?,
            "--writable-mount-probe" => switches.writable_mount_probe = true,
            "--truncate-at-the-return-point" => truncate_at_the_return_point = true,
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
            streams,
        })),
        "stream-constants" => Some(ParsedCommand::StreamConstants(streams)),
        "confirmed-loss-form" => Some(ParsedCommand::ConfirmedLossForm(ConfirmedLossCommandLine {
            output_directory: output_directory?,
            truncate_at_the_return_point,
        })),
        _ => None,
    }
}
