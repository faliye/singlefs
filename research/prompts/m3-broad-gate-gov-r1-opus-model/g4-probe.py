#!/usr/bin/env python3
"""攻方腿模型（m3-broad-gate-gov-r1，G4）：载入冻结副本拷贝里的 .claude/hooks/lib_heavy_tests.py，逐条判命令文本，看驱动与门禁的各种用法判成哪一类；
（入口 heavy_tests_in_command_text，与 heavy-test-guard.sh 判命令文本同一个）；再照 heavy-test-guard.sh 的 refusal_reason（子 agent 一律拒；主 agent 不带 =commit / =user-request 拒）写出谁会被拒。只判命令文本，不执行它们。
用法：g4-probe.py <tree>"""
import importlib.util, os, sys
tree = sys.argv[1]
specification = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(tree, ".claude/hooks/lib_heavy_tests.py"))
library = importlib.util.module_from_spec(specification)
sys.path.insert(0, os.path.join(tree, ".claude/hooks"))
specification.loader.exec_module(library)
driver = "research/scripts/broad-mutation.py"
gate = ".claude/gate.d/checker-tier-broad-mutation.sh"
cases = [
    ("驱动 --summary（门禁 gate-verdict 格起的那一条）", ["python3", driver, "--summary"]),
    ("驱动 --list-items --changed-since HEAD（文件头说只读）", ["python3", driver, "--list-items", "--changed-since", "HEAD"]),
    ("驱动 --check-ignored（文件头：不编不跑、不要前缀）", ["python3", driver, "--check-ignored"]),
    ("驱动 --selftest（登记 light）", ["python3", driver, "--selftest"]),
    ("第 2b 步 ② --sync-tree --commit", ["python3", driver, "--sync-tree", "--commit", "0" * 40]),
    ("第 2b 步 ③ --changed-since HEAD", ["python3", driver, "--changed-since", "HEAD", "--threads", "8", "--memory-cap", "16G"]),
    ("里程碑末全量 --milestone-full（几天）", ["python3", driver, "--milestone-full"]),
    ("--import-results（往结果表里导入别处的行）", ["python3", driver, "--import-results", "/tmp/claude-1000/peer"]),
    ("--item 1 --rerun（已有定论也重跑）", ["python3", driver, "--item", "1", "--rerun"]),
    ("门禁整道（第 2b 步 ④）", ["bash", gate]),
    ("门禁 --check gate-verdict（只读库目录、起 --summary）", ["bash", gate, "--check", "gate-verdict"]),
    ("门禁 --list", ["bash", gate, "--list"]),
    ("门禁 --tool-versions", ["bash", gate, "--tool-versions"]),
    ("python3 读 stdin 喂的代码（文件头自认罩不住）", ["python3", "-"]),
    ("python3 跑一份 import 驱动、调它函数的脚本（文件头自认罩不住）", ["python3", "/tmp/claude-1000/x/driver-model.py"]),
]
for label, words in cases:
    found = library.heavy_tests_in_command_text(" ".join(words), tree)
    kind = found[0].kind if found else None
    category = library.KIND_CATEGORY.get(kind, "—") if kind else "—"
    if kind is None:
        who = "谁都放行（不算重型）"
    elif kind in ("commit-check-stage", "commit-check-lint", "commit-check-selftest", "commit-check-selftest-unregistered", "commit-check-structure"):
        who = "提交时才跑的检查：子 agent 拒；主 agent 不带前缀拒"
    else:
        who = "子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行"
    print(f"PROBE {label}\tkind={kind}\t类={category}\t{who}")
