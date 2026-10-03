#!/usr/bin/env python3
"""在副本工作区里逐行套用 crates/mutations.tsv 的指定行（只管 harness 档的行），跑该行点名的 cargo 参数，判点名的测试红没红，再还原。

用法：run_harness_mutation_rows.py <副本根> <行号> [<行号> ...]
副本根里要有 Cargo.toml、Cargo.lock、crates/（冻结副本拷出来的）；只改副本里的源文件，改前存原文、跑完还原并逐字节比对。
每行：锚点（原文）在源文件里恰好命中一次才套，替换后跑 `cargo test --offline <第 5 列>`，
输出里 `test <第 6 列名> ... FAILED` 算红；编译失败单独标「编不过」，不算红。
命令经 capped.sh 与 run-with-memory-cap.sh 起（线程 8、内存 8G）。
"""
import os
import re
import subprocess
import sys

ROOT_OF_REPO = "/home/fy5090/code/singlefs"
CAPPED = ROOT_OF_REPO + "/research/scripts/capped.sh"
MEMCAP = ROOT_OF_REPO + "/research/scripts/run-with-memory-cap.sh"


def main() -> int:
    ws = sys.argv[1]
    rows = [int(x) for x in sys.argv[2:]]
    lines = open(os.path.join(ws, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
    env = dict(os.environ)
    status = 0
    for number in rows:
        columns = lines[number - 1].split("\t")
        label, path, old, new, cargo_args, test_name = columns[:6]
        old = old.replace("\\n", "\n")
        new = new.replace("\\n", "\n")
        target = os.path.join(ws, path)
        original = open(target, encoding="utf-8").read()
        count = original.count(old)
        print(f"=== 行 {number}: {label}")
        print(f"    文件 {path}；锚点命中 {count} 次；点名测试 {test_name}")
        if count != 1:
            print("    锚点不是恰好命中一次，不套")
            status = 1
            continue
        mutated = original.replace(old, new)
        open(target, "w", encoding="utf-8").write(mutated)
        try:
            command = (
                ["nice", "-n", "19", "bash", CAPPED, "8", "bash", MEMCAP, "8G", "cargo", "test", "--offline"]
                + cargo_args.split()
            )
            result = subprocess.run(
                command, cwd=ws, env=env, capture_output=True, text=True, timeout=230
            )
            output = result.stdout + result.stderr
            compile_failed = "could not compile" in output
            red_names = re.findall(r"^test (\S+) \.\.\. FAILED$", output, re.M)
            ok_names = re.findall(r"^test (\S+) \.\.\. ok$", output, re.M)
            named_red = any(name.endswith(test_name) for name in red_names)
            named_ok = any(name.endswith(test_name) for name in ok_names)
            print(f"    退出码 {result.returncode}；编不过 {compile_failed}")
            print(f"    红的测试 {red_names}")
            print(f"    绿的测试数 {len(ok_names)}；点名测试红 {named_red}；点名测试绿 {named_ok}")
            summary = [line for line in output.splitlines() if line.startswith("test result")]
            if not summary or result.returncode not in (0, 101):
                print("    输出末 12 行：")
                for line in output.splitlines()[-12:]:
                    print("      | " + line)
            print(f"    汇总行 {summary}")
        finally:
            open(target, "w", encoding="utf-8").write(original)
            assert open(target, encoding="utf-8").read() == original
    return status


if __name__ == "__main__":
    sys.exit(main())
