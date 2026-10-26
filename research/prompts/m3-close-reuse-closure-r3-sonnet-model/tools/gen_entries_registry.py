#!/usr/bin/env python3
"""gen_entries_registry.py <仓根或冻结副本>：从源码里的登记表现取判法入口，一行一个，和 tools/entries-registry.txt 同一份：
广谱变异 JUDGING_ENTRIES 与 GPU_JUDGING_ENTRIES、crates 变异 JUDGING_ENTRIES、admission.py 的 REPLAY_JUDGING_FUNCTIONS（replay.sh 里）、e165 的 replay、
verus-proof-check.py 的 JUDGING_ENTRIES，再加三个 judge: 令牌入口（lib_libtest_result_lines.py 的 result_line 与 recordable_results、harness-model-differential-and-scenarios.sh 的 model_section_verdict）。"""
import ast, os, re, sys
root = os.path.abspath(sys.argv[1]) + "/"
def literal(path, name):
    for node in ast.parse(open(root + path, encoding="utf-8").read()).body:
        if isinstance(node, ast.Assign) and any(getattr(t, "id", None) == name for t in node.targets):
            return ast.literal_eval(node.value)
out = []
out += [f"{p}::{n}" for p, n in literal("scripts/mutation/broad-mutation.py", "JUDGING_ENTRIES")]
out += [f"{p}::{n}" for p, n in literal("scripts/mutation/broad-mutation.py", "GPU_JUDGING_ENTRIES")]
out += list(literal("scripts/mutation/crates-mutation-rows.py", "JUDGING_ENTRIES"))
admission = open(root + "scripts/gate/admission.py", encoding="utf-8").read()
out += [f"scripts/experiment-apparatus/replay.sh::{f}" for f in ast.literal_eval(re.search(r"^REPLAY_JUDGING_FUNCTIONS = (\(.*\))$", admission, re.M).group(1))]
out.append("scripts/experiment-apparatus/e165-pin-red-examples.py::replay")
out += [f"scripts/gate/verus-proof-check.py::{v}" for v in literal("scripts/gate/verus-proof-check.py", "JUDGING_ENTRIES").values()]
out += ["scripts/gate/lib_libtest_result_lines.py::result_line", "scripts/gate/lib_libtest_result_lines.py::recordable_results",
        ".claude/gate.d/harness-model-differential-and-scenarios.sh::model_section_verdict"]
print("\n".join(out))
