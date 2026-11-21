//! 第三段：`bad-states` 模式的导出、报告与命令行的单测。

use super::*;

#[test]
fn the_bad_states_command_line_takes_a_dump_directory_and_a_sixteen_digit_seed() {
    let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
    let parsed = parse_bad_states_command_line(&words(
        "bad-states --dump-directory d --seed-hex 8546b1258e427c34",
    ))
    .expect("合法的命令行");
    assert_eq!(parsed.dump_directory, PathBuf::from("d"));
    assert_eq!(parsed.seed_hex, "8546b1258e427c34");
    assert!(
        parse_bad_states_command_line(&words("bad-states --dump-directory d --seed-hex 8546"))
            .is_none()
    );
    assert!(
        parse_bad_states_command_line(&words("bad-states --seed-hex 8546b1258e427c34")).is_none()
    );
    assert!(parse_bad_states_command_line(&words(
        "dump --dump-directory d --seed-hex 8546b1258e427c34"
    ))
    .is_none());
}

#[test]
fn the_second_run_bad_states_mode_is_its_own_word_and_the_first_run_word_still_means_the_first_table(
) {
    let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
    let first_run = parse_bad_states_command_line(&words(
        "bad-states --dump-directory d --seed-hex 85441ae491419411",
    ))
    .expect("合法的命令行");
    assert_eq!(first_run.table_version, BadTableVersion::FirstRun);
    let second_run = parse_bad_states_command_line(&words(
        "bad-states-second-run --dump-directory d --seed-hex 85441ae491419411",
    ))
    .expect("合法的命令行");
    assert_eq!(second_run.table_version, BadTableVersion::SecondRun);
    assert_eq!(second_run.seed_hex, "85441ae491419411");
    assert!(parse_bad_states_command_line(&words(
        "bad-states-second-run-extra --dump-directory d --seed-hex 85441ae491419411"
    ))
    .is_none());
    assert_eq!(BadTableVersion::FirstRun.mode_word(), "bad-states");
    assert_eq!(
        BadTableVersion::SecondRun.mode_word(),
        "bad-states-second-run"
    );
}
