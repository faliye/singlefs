use super::*;

#[test]
fn result_lines_count_every_line_they_print() {
    let mut output = ResultLines::new();
    output.line("name=test first".to_string());
    output.line("name=test second".to_string());
    output.line("name=test third".to_string());
    assert_eq!(output.emitted, 3, "V4：收尾计数行数的是真打出去的行");
}

/// `preflight` 交给 `preflight.py check` 的路径要读得到，读出的就是编进这个二进制的根文件，文件头带着准入与运行条件的声明。
/// 要读运行时那条路径上的文件：断言的就是装置跑起来时 `preflight.py` 拿到的那一份在不在、是不是这一份。
#[test]
fn preflight_source_path_reads_the_root_file_compiled_into_this_binary_with_its_declarations() {
    let text = std::fs::read_to_string(SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS)
        .unwrap_or_else(|error| {
            panic!(
                "preflight 要判的源文件 {SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS} 读不到：{error}"
            )
        });
    assert_eq!(
        text,
        include_str!("main.rs"),
        "{SOURCE_PATH_CARRYING_THE_PREFLIGHT_DECLARATIONS} 就是编进这个二进制的根文件"
    );
    assert!(
        text.contains("// admission:") && text.contains("// run-condition:"),
        "根文件的文件头要有准入与运行条件"
    );
}
