use super::*;
use crate::helpers_shared_by_the_tests::scratch_directory;

/// `library` 子命令不收选库的参数：带 `--library <路径>` 是用法错，开库之前就拒。
#[test]
fn the_library_command_refuses_a_library_argument_before_touching_anything() {
    let keys = scratch_directory("library-keys");
    let arguments: Vec<String> = [
        "--keys",
        keys.to_str().expect("路径"),
        "--library",
        "/library/x",
    ]
    .iter()
    .map(|word| (*word).to_string())
    .collect();
    assert_eq!(run_library_command(&arguments), EXIT_CODE_USAGE);
}
