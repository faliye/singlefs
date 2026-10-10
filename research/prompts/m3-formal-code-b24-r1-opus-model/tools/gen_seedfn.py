#!/usr/bin/env python3
"""从 crash_injection.rs 原样抽出二次崩溃种子那一段与 F3-c、F3-d 四条用例，写成探针包的一个模块（m3-formal-code-b24-r1 opus）。
用法：gen_seedfn.py <crash_injection.rs> <输出 .rs> [<变异表> <行号>]
给了变异表与行号：先把那一行（六段制表符分隔，原文 / 替换文里的 \\n 是换行，与 crates/mutations.tsv 同一写法）施加到源文本上
（原文要恰好命中一次，否则退 3），再抽。抽的几段按起止锚点取，锚点不是恰好命中一次退 4。用例去掉 #[test]、改成 pub fn，函数体一字不改。"""
import hashlib, json, sys
src_path, out_path = sys.argv[1], sys.argv[2]
src = open(src_path, encoding="utf-8").read()
label = "pristine"
if len(sys.argv) > 3:
    rows = open(sys.argv[3], encoding="utf-8").read().split("\n")
    row = rows[int(sys.argv[4]) - 1].split("\t")
    old, new = row[2].replace("\\n", "\n"), row[3].replace("\\n", "\n")
    if src.count(old) != 1:
        print(f"变异原文命中 {src.count(old)} 次：{row[0]}", file=sys.stderr); sys.exit(3)
    src = src.replace(old, new)
    label = f"{sys.argv[3].rsplit('/', 1)[-1]}:{sys.argv[4]} {row[0]}"

def between(start, end):
    if src.count(start) != 1 or src.count(end) != 1:
        print(f"锚点命中不是一次：{start[:40]!r} {src.count(start)} / {end[:40]!r} {src.count(end)}", file=sys.stderr); sys.exit(4)
    i = src.index(start); j = src.index(end, i)
    return src[i:j]

consts = between("/// 摆挂载途中的二次崩溃的随机源", "/// 每个工作线程摊到的片数")
seedfn = between("/// 第一次崩溃那个段内子集的指纹：段内第 k 个写", "/// 挂载的每一段各摆一个二次崩溃")
tests = between("    /// 二次崩溃的种子把段内子集整个折进去", "    /// 实例表清扫两段")
test_lines = []
for line in tests.split("\n"):
    if line.strip() == "#[test]":
        continue
    line = line[4:] if line.startswith("    ") else line
    if line.startswith("fn "):
        line = "pub " + line
    test_lines.append(line)
names = [l.split("(")[0][len("pub fn "):] for l in test_lines if l.startswith("pub fn ") and l.split("(")[0][len("pub fn "):].startswith(("all_first", "first_crash"))]
digest = hashlib.sha256((consts + seedfn + tests).encode()).hexdigest()
out = f"""// 生成的（tools/gen_seedfn.py），别手改。来源 {src_path.rsplit('crates/', 1)[-1] if 'crates/' in src_path else src_path}；施加的变异：{label}
// 抽出的三段（常量、种子函数、四条用例）合起来的 sha256（施加变异之后）：{digest}
#![allow(dead_code, clippy::all, rustdoc::broken_intra_doc_links)]
use singlefs_checker_tier::crash_injection::CrashPoint;
use singlefs_harness::history::{{HistoryOperationKind, HistorySeed, StepPosition}};
use std::collections::BTreeSet;

pub const LABEL: &str = {json.dumps(label, ensure_ascii=False)};
pub const EXTRACT_SHA256: &str = "{digest}";

{consts}
{seedfn}
/// 探针用：(历史种子, 段号, 段内子集) ↦ 种子。
pub fn seed_of(seed: u64, segment_index: usize, persisted: &[bool]) -> u64 {{
    seed_of_the_second_crash_points(
        HistorySeed(seed),
        &CrashPoint {{
            segment_index,
            persisted_within_the_segment: persisted.to_vec(),
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        }},
    )
    .0
}}

pub mod extracted_tests {{
use super::*;
{chr(10).join(test_lines)}
}}

pub const TESTS: &[(&str, fn())] = &[
{chr(10).join(f'    ("{n}", extracted_tests::{n}),' for n in names)}
];
"""
open(out_path + ".new", "w", encoding="utf-8").write(out)
import os
os.replace(out_path + ".new", out_path)
print(f"{label} tests={len(names)} sha256={digest}")
