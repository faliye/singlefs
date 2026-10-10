#!/usr/bin/env python3
"""变异表 GPU 那几行（checker-tier，只读静态核）：锚点在 wgsl 里命中几次、点名的测试在对拍文件里是不是 #[test] fn、--exact 名对不对得上。用法：gpu_named_tests_exist.py <树根> <首行> <末行>"""
import re, sys
from pathlib import Path
tree, first, last = Path(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
rows = (tree / "crates" / "mutations.tsv").read_text(encoding="utf-8").split("\n")
test_file = tree / "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs"
tests = re.findall(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([A-Za-z0-9_]+)", test_file.read_text(encoding="utf-8"))
for line in range(first, last + 1):
    name, file, original, replacement, args, named = rows[line - 1].split("\t")
    text = (tree / file).read_text(encoding="utf-8")
    hits = text.count(original.replace("\\n", "\n"))
    exact = args.split("--exact")[-1].strip().split()[0] if "--exact" in args else "-"
    print(f"GPU-ANCHOR line={line} anchor_hits={hits} named_is_a_test={'yes' if named in tests else 'no'} exact_arg_equals_named={'yes' if exact == named else 'no'} test_count_with_that_name={tests.count(named)}")
