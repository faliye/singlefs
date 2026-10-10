#!/usr/bin/env python3
"""K4：把冻结副本 checker 档两份源文件测试模块里的扫源码函数与几条扫源码用例的函数体原样抄进一个独立小 crate（不是 checker 档包，
不跑它的测试）：build-scan-proto.py <冻结副本根> <输出目录>。用例 `#[test] fn 名() { let source = include_str!(…);` 改成收 `source` 的函数，
别的一字不改；每一段的头一行先核原文，对不上退 3。编由调用方起（cargo build，经内存包装）。"""
import re
import sys
from pathlib import Path

frozen, out = Path(sys.argv[1]), Path(sys.argv[2])
gpu = (frozen / "crates/singlefs-checker-tier/src/gpu_unit_checks.rs").read_text().split("\n")
crash = (frozen / "crates/singlefs-checker-tier/src/crash_amplification.rs").read_text().split("\n")


def take(lines, first, last, must_start):
    block = lines[first - 1:last]
    if must_start not in block[0]:
        print(f"第 {first} 行不是 {must_start!r}：{block[0]!r}", file=sys.stderr)
        sys.exit(3)
    return "\n".join(block)


def as_function(text, file_name):
    text = text.replace("    #[test]\n", "", 1)
    pattern = r"fn (\w+)\(\)\s*\{\s*let source = include_str!\(\"" + re.escape(file_name) + r"\"\);"
    text, count = re.subn(pattern, r"fn \1(source: &str) {", text, count=1)
    if count != 1:
        text, count = re.subn(r"fn (\w+)\(\) \{", r"fn \1(source: &str) {", text, count=1)
        text = text.replace('include_str!(\n                "' + file_name + '"\n            )', "source")
    assert count == 1, text[:200]
    return text


gpu_parts = [
    take(gpu, 1920, 1922, "const TESTS_MODULE_OPENING"),
    take(gpu, 2458, 2983, "/// `text` 以"),
]
gpu_tests = [
    (2219, 2249, "the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool"),
    (2339, 2395, "the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first"),
    (2419, 2456, "the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking"),
    (3020, 3031, "the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call"),
]
for first, last, name in gpu_tests:
    gpu_parts.append(as_function(take(gpu, first, last, "#[test]"), "gpu_unit_checks.rs"))
crash_parts = [
    take(crash, 3581, 3583, "const TESTS_MODULE_OPENING"),
    take(crash, 4643, 4702, "/// `source`"),
    take(crash, 4864, 5262, "/// `_stop_is_requested`"),
    take(crash, 5379, 5595, "/// `source`"),
    as_function(take(crash, 5368, 5378, "#[test]"), "crash_amplification.rs"),
]
main_template = """
fn main() {
    // 用法：<二进制> <源文件>…：每份对每条抄来的用例跑一次（catch_unwind），打印绿 / 红与红的那一句的头 200 字。
    std::panic::set_hook(Box::new(|_| {}));
    for path in std::env::args().skip(1) {
        let source: &'static str = Box::leak(std::fs::read_to_string(&path).expect("读源文件").into_boxed_str());
        for (name, check) in CHECKS {
            let outcome = std::panic::catch_unwind(|| check(source));
            match outcome {
                Ok(()) => println!("ATK {TAG} {path} {name} green"),
                Err(payload) => {
                    let text = payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    println!("ATK {TAG} {path} {name} RED {}", text.replace('\\n', " ").chars().take(200).collect::<String>());
                }
            }
        }
    }
}
"""
gpu_checks = "const TAG: &str = \"gpu\";\nconst CHECKS: [(&str, fn(&'static str)); 4] = [\n" + "".join(f"    (\"{n}\", {n}),\n" for _, _, n in gpu_tests) + "];\n"
crash_checks = "const TAG: &str = \"stop\";\nconst CHECKS: [(&str, fn(&'static str)); 1] = [\n    (\"every_layer_handed_the_stop_request_hands_it_on_or_asks_it\", every_layer_handed_the_stop_request_hands_it_on_or_asks_it),\n];\n"
(out / "src/bin").mkdir(parents=True, exist_ok=True)
(out / "Cargo.toml").write_text('[package]\nname = "atk-scan"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n')
header = "#![allow(dead_code, clippy::all, unused)]\n// 抄自冻结副本 crates/singlefs-checker-tier/src/{} 测试模块，原样（用例改成收 source 的函数）；CHECKS 与 main 是攻方加的。\n"
(out / "src/bin/atk-gpu-scan.rs").write_text(header.format("gpu_unit_checks.rs") + "\n\n".join(gpu_parts) + "\n" + gpu_checks + main_template)
(out / "src/bin/atk-stop-scan.rs").write_text(header.format("crash_amplification.rs") + "\n\n".join(crash_parts) + "\n" + crash_checks + main_template)
(out / "src/lib.rs").write_text("")
print("写了", out)
