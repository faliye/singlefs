//! 命令行的解析的单测。

use super::*;

#[test]
fn the_sampling_threads_argument_is_parsed_and_zero_is_refused() {
    let arguments_with = |threads: &str| {
        [
            "baseline",
            "--output-directory",
            "目录",
            "--sampling-threads",
            threads,
        ]
        .map(str::to_string)
    };
    let Some(ParsedCommand::Dump(command_line)) = parse_command_line(&arguments_with("1")) else {
        panic!("baseline 带 --sampling-threads 1 要认");
    };
    assert_eq!(
        command_line.switches.sampling_threads,
        Some(NonZeroUsize::MIN)
    );
    assert!(
        parse_command_line(&arguments_with("0")).is_none(),
        "0 条线程不认"
    );
}
