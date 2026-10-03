#!/usr/bin/env python3
"""照门禁 checker-tier-formal-verification 的 verus-must-red-worlds 格（research/scripts/verus-proof-check.py 的 check_must_red_worlds）判一行：
读 Verus 的 stdout（--output-json）与 stderr，拼「verified, M errors」，数 rlimit 用尽与证明错，看期望片段在不在。
用法：verus-judge-row.py <行号> <rlimit> <stdout 文件> <stderr 文件> <期望片段（攻方自己的行写 -）>；打印一行制表符分隔的判定。"""
import json
import sys

RLIMIT_EXHAUSTED = "Resource limit (rlimit) exceeded"
row, rlimit, stdout_path, stderr_path, expected = sys.argv[1:6]
stdout = open(stdout_path, encoding="utf-8", errors="replace").read()
stderr = open(stderr_path, encoding="utf-8", errors="replace").read()
diagnostics = stderr + "\n" + stdout
try:
    results = json.loads(stdout)["verification-results"] if stdout.strip().startswith("{") else None
except (ValueError, KeyError):
    results = None
if results is None:
    print(f"{row}\trlimit={rlimit}\tNO-JSON\t-\t-\t-\tverdict=NOT_JUDGED")
    sys.exit(0)
verified, errors = results.get("verified"), results.get("errors")
summary_line = f"verification results:: {verified} verified, {errors} errors"
rlimit_exhausted = RLIMIT_EXHAUSTED in diagnostics
proof_errors = []
for line in diagnostics.split("\n"):
    if not line.startswith("error"):
        continue
    message = line.split(":", 1)[1].strip() if ":" in line else line
    if RLIMIT_EXHAUSTED in message or message.startswith("aborting due to"):
        continue
    proof_errors.append(message)
rlimit_lines = sum(1 for line in diagnostics.split("\n") if line.startswith("error") and RLIMIT_EXHAUSTED in line)
if expected == "-":
    verdict = "VERIFIED" if errors == 0 else "RED"
elif rlimit_exhausted and not proof_errors:
    verdict = "GATE_RED_rlimit_only"
elif errors < 1:
    verdict = "GATE_RED_survived"
elif expected not in diagnostics + "\n" + summary_line:
    verdict = "GATE_RED_fragment_missing"
else:
    verdict = "GATE_OK"
print(f"{row}\trlimit={rlimit}\tverified={verified}\terrors={errors}\trlimit_error_lines={rlimit_lines}\tproof_errors={len(proof_errors)}\texpected={expected}\tverdict={verdict}")
