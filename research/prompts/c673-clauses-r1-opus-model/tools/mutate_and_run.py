#!/usr/bin/env python3
"""c673-clauses-r1 云端攻方：在仓副本上逐条改坏、跑点名的 singlefs-harness 测试目标、还原。

用法：mutate_and_run.py <仓副本根> <变异表> <输出目录> [--only 变异名]... [--baseline]
  --baseline 不改任何文件，把变异表里出现过的测试目标各跑一遍（基线）。
只改仓副本，不碰仓库本身。每个目标一行 `name=mutation_result`，原样输出另存 <输出目录>/<变异名>-<目标>.log。
cargo 由 research/scripts/capped.sh 8 起；整条脚本由调用方经 research/scripts/run-with-memory-cap.sh 包着跑。
"""
import os
import subprocess
import sys

REPO_SCRIPTS = os.environ.get("SINGLEFS_REPO", "/home/fy5090/code/singlefs") + "/research/scripts"


def unescape(text):
    return text.replace("\\n", "\n").replace("\\t", "\t")


def load(table):
    rows = []
    for line in open(table, encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        name, path, old, new, targets = line.rstrip("\n").split("\t")
        rows.append((name, path, unescape(old), unescape(new), targets.split()))
    return rows


def run_target(repo, target_spec, log_path):
    target, _, filter_word = target_spec.partition(":")
    command = ["bash", f"{REPO_SCRIPTS}/capped.sh", "8", "cargo", "test", "-p", "singlefs-harness", "--test", target]
    if filter_word:
        command += ["--", "--nocapture", "--test-threads=1", filter_word]
    completed = subprocess.run(command, cwd=repo, capture_output=True, text=True)
    output = completed.stdout + completed.stderr
    with open(log_path, "w", encoding="utf-8") as log:
        log.write(output)
    results = [line.strip() for line in output.splitlines() if line.startswith("test result:")]
    failed = [line.split()[1] for line in output.splitlines() if line.startswith("test ") and line.endswith("FAILED")]
    probe_lines = [line for line in output.splitlines() if "name=" in line]
    return completed.returncode, results, failed, probe_lines


def main():
    repo, table, out = sys.argv[1], sys.argv[2], sys.argv[3]
    only = [sys.argv[i + 1] for i, arg in enumerate(sys.argv) if arg == "--only"]
    baseline = "--baseline" in sys.argv
    os.makedirs(out, exist_ok=True)
    rows = load(table)
    if baseline:
        targets = sorted({spec for row in rows for spec in row[4]})
        rows = [("baseline", None, None, None, targets)]
    for name, path, old, new, targets in rows:
        if only and name not in only:
            continue
        original = None
        if path is not None:
            full = os.path.join(repo, path)
            original = open(full, encoding="utf-8").read()
            assert original.count(old) == 1, f"{name}: 原文在 {path} 里命中 {original.count(old)} 次"
            open(full, "w", encoding="utf-8").write(original.replace(old, new))
        try:
            for spec in targets:
                log_path = os.path.join(out, f"{name}-{spec.replace(':', '-')}.log")
                rc, results, failed, probe_lines = run_target(repo, spec, log_path)
                print(f"name=mutation_result mutation={name} target={spec} rc={rc} failed={failed} results={results}", flush=True)
                for line in probe_lines:
                    print(f"  {name} {line[line.index('name='):][:600]}", flush=True)
        finally:
            if original is not None:
                open(full, "w", encoding="utf-8").write(original)


if __name__ == "__main__":
    main()
