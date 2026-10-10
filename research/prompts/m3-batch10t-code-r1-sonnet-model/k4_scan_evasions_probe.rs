//! 正推腿对 K4-2 的探针：把冻结副本里的扫源码测试文件（checker 档那份，不跑它）拷成 `scan_copy.rs`
//! （只把顶格的 `fn` 改成 `pub fn`，其余一字不动），在 harness 的测试二进制里拿它的词法剥离与「测试代码行」两个函数喂样本。
//! 放进 `crates/singlefs-harness/tests/`、与 `scan_copy.rs` 同目录；只跑名字带 `probe_` 的：
//! `cargo test -p singlefs-harness --test k4_scan_evasions_probe -- probe_ --nocapture`。
#![allow(dead_code)]
mod scan_copy;
use scan_copy::{code_lines, lines_mentioning, test_code_lines_of_a_library_source};

const BYPASS: [&str; 2] = ["usable_gpu_cards", "gpu_cards_by_priority"];

/// 对一份「库源码」样本：返回扫得出的行号（先剥离再取测试代码行再找名字）。
fn library_source_hits(sample: &str) -> Vec<usize> {
    let (lines, _items) = test_code_lines_of_a_library_source(&code_lines(sample));
    lines_mentioning("src/sample.rs", &lines, &BYPASS)
        .iter()
        .map(|hit| hit.line_number)
        .collect()
}

/// 对一份「tests/ 下整份」样本：返回扫得出的行号。
fn test_file_hits(sample: &str) -> Vec<usize> {
    lines_mentioning("tests/sample.rs", &code_lines(sample), &BYPASS)
        .iter()
        .map(|hit| hit.line_number)
        .collect()
}

#[test]
fn probe_control_the_plain_forms_are_caught() {
    assert_eq!(test_file_hits("fn f() {\n    let _ = usable_gpu_cards();\n}\n"), vec![2]);
    assert_eq!(
        library_source_hits("#[cfg(test)]\nmod t {\n    fn f() {\n        let _ = usable_gpu_cards();\n    }\n}\n"),
        vec![4]
    );
}

/// 绕法 1：C 原始字符串 `cr#"…"#`，里面有一个 `"`：扫源码的词法只认 `r#"`、`br#"`，`cr#` 的 `"` 当成普通字符串起头，
/// 在内部的 `"` 处提前收尾，后面的代码被当成字符串内容吃掉。
#[test]
fn probe_evasion_1_a_c_raw_string_swallows_the_call_after_it() {
    let sample = "fn f() {\n    let _a = cr#\"x\" \"#; let _c = usable_gpu_cards(); let _b = cr#\"y\"#;\n}\n";
    let hits = test_file_hits(sample);
    println!("PROBE_EVASION_1 hits={hits:?} (真实 Rust 里第 2 行调了 usable_gpu_cards)");
    assert_eq!(hits, Vec::<usize>::new(), "扫不出");
}

/// 绕法 2：`#[cfg(test)]` 下的函数有两个泛型参数：条目的收尾在最外层遇到 `,` 就停，`<A, B>` 里的逗号让条目在签名里就收了，函数体没进「测试代码行」。
#[test]
fn probe_evasion_2_a_comma_in_the_generics_ends_the_item_before_its_body() {
    let sample = "#[cfg(test)]\nfn helper<A, B>() {\n    let _ = usable_gpu_cards();\n}\n";
    let hits = library_source_hits(sample);
    println!("PROBE_EVASION_2 hits={hits:?}");
    assert_eq!(hits, Vec::<usize>::new(), "扫不出");
    let where_clause = "#[cfg(test)]\nfn helper<T>() where T: Copy, {\n    let _ = usable_gpu_cards();\n}\n";
    let hits = library_source_hits(where_clause);
    println!("PROBE_EVASION_2b hits={hits:?}");
    assert_eq!(hits, Vec::<usize>::new(), "where 子句里的逗号同样让条目提前收尾");
}

/// 绕法 3：库源码里不带 `#[cfg(test)]` 的 `#[test]` 函数（只在测试编译里存在）不被收成测试代码。
#[test]
fn probe_evasion_3_a_bare_test_attribute_outside_a_cfg_test_item_is_not_scanned() {
    let sample = "#[test]\nfn takes_cards() {\n    let _ = usable_gpu_cards();\n}\n";
    let hits = library_source_hits(sample);
    println!("PROBE_EVASION_3 hits={hits:?}");
    assert_eq!(hits, Vec::<usize>::new());
}

/// 绕法 4：`#![cfg(test)]` 文件级内部属性、`#[cfg(test)] mod x;`（模块在自己的文件里、文件里没有 cfg）、
/// 与别的属性写在同一行的 `#[cfg(test)]`、`#[cfg (test)]` 带空格：都不是「顶格以 `#[cfg(` 起头的一行」。
#[test]
fn probe_evasion_4_the_ways_of_writing_cfg_test_that_do_not_start_a_line_with_the_attribute_open() {
    let inner = "#![cfg(test)]\nfn f() {\n    let _ = usable_gpu_cards();\n}\n";
    let own_file = "pub fn f() {\n    let _ = usable_gpu_cards();\n}\n";
    let same_line = "#[allow(dead_code)] #[cfg(test)] mod m {\n    fn f() {\n        let _ = usable_gpu_cards();\n    }\n}\n";
    let spaced = "#[cfg (test)]\nmod m {\n    fn f() {\n        let _ = usable_gpu_cards();\n    }\n}\n";
    for (name, sample) in [("inner", inner), ("own_file", own_file), ("same_line", same_line), ("spaced", spaced)] {
        let hits = library_source_hits(sample);
        println!("PROBE_EVASION_4 form={name} hits={hits:?}");
        assert_eq!(hits, Vec::<usize>::new(), "{name}");
    }
}

/// 对照：不用那两个名字、直接用 wgpu 起设备，扫的名字表里没有 `request_device`、`wgpu::Instance`。
#[test]
fn probe_evasion_5_starting_a_device_straight_from_wgpu_names_none_of_the_scanned_functions() {
    let sample = "fn f() {\n    let instance = wgpu::Instance::default();\n    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();\n    let _ = pollster::block_on(adapter.request_device(&Default::default()));\n}\n";
    assert_eq!(test_file_hits(sample), Vec::<usize>::new());
    println!("PROBE_EVASION_5 hits=[]");
}
