#!/usr/bin/env python3
"""基线与变异逐行复跑（只 CPU 与 harness 档，GPU 两行不跑）。
用法：drive.py <工作目录 W>；环境变量：SINGLEFS_REPO（被判的树）、SINGLEFS_WRAPPER（含 scripts/hosts/run-with-memory-cap.sh 与 capped.sh 的仓）。
产出：W/out/baseline.txt（探针行与每条用例的 ok / FAILED，排序后）、W/out/mutations.txt（每条变异：名、点名的测试、RED / NOT_RED、红的用例）。"""
import os, re, shutil, subprocess, sys, time

START = time.time()
BUDGET = float(os.environ.get("DRIVE_BUDGET_SECONDS", "150"))  # 每次调用最多再开始新的一步的挂钟；到了就退 3，再调一次接着做

W = sys.argv[1]
SRC = os.environ["SINGLEFS_REPO"]
WRAP = os.environ["SINGLEFS_WRAPPER"]
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = W + "/repo"
OUT = W + "/out"
os.makedirs(OUT, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=W + "/target")


def sh(cap, args, log):
    command = ["nice", "-n", "19", "bash", WRAP + "/scripts/hosts/run-with-memory-cap.sh", cap,
               "bash", WRAP + "/scripts/hosts/capped.sh", "7", "cargo", "test", "--offline", *args]
    with open(log, "w") as handle:
        return subprocess.run(command, cwd=REPO, env=env, stdout=handle, stderr=subprocess.STDOUT).returncode


def copy_tree():
    if os.path.exists(REPO):
        shutil.rmtree(REPO)
    os.makedirs(REPO)
    for name in ("Cargo.toml", "Cargo.lock"):
        shutil.copy(SRC + "/" + name, REPO + "/" + name)
    shutil.copytree(SRC + "/crates", REPO + "/crates")


def results(log):
    text = open(log, encoding="utf-8", errors="replace").read()
    return re.findall(r"^test (\S+) \.\.\. (ok|FAILED)$", text, re.M), text


# 基线：新增的与 H-V 的用例，带探针
if not os.path.exists(OUT + "/baseline.txt"):
    copy_tree()
    subprocess.run([sys.executable, HERE + "/probe.py", REPO], check=True)
    test_args = ["-p", "singlefs-harness", "--test", "checker_known_bad_images"]
    filters = ["beside_a_copy_verifying_as_a_node", "child_pointer_birth_tree_is_one_off_over_a_leaf",
               "whose_second_copy_verifies_as_a_node", "copy_verifying_as_a_node"]
    assert sh("6G", test_args + ["--no-run"], OUT + "/build.log") == 0, "编译失败"
    rc = sh("4G", test_args + ["--", "--nocapture", *filters], OUT + "/baseline.log")
    cases, text = results(OUT + "/baseline.log")
    probes = sorted(set(line for line in text.splitlines() if line.startswith("PROBE-")))
    with open(OUT + "/baseline.tmp", "w") as handle:
        handle.write(f"rc={rc}\n")
        for line in sorted(f"{name} {state}" for name, state in cases):
            handle.write(line + "\n")
        for line in probes:
            handle.write(line + "\n")
    os.rename(OUT + "/baseline.tmp", OUT + "/baseline.txt")

# 变异：第 30 批的 CPU 行与第 2919 行那一行（行从 SRC 的变异表里按名字取）
rows = [line.split("\t") for line in open(SRC + "/crates/mutations.tsv", encoding="utf-8").read().split("\n") if line and not line.startswith("#")]
new_row_prefixes = ("第30批 F-W", "第30批 X2", "第30批 X13", "第30批 X14")  # 第 30 批新增的 CPU 行；同样以「第30批」起头的别的行（如 30-3）不选
selected = [r for r in rows if r[0].startswith(new_row_prefixes) or r[0].startswith("第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得")]
extra_path = HERE + "/extra-rows.tsv"
extras = [line.split("\t") for line in open(extra_path, encoding="utf-8").read().split("\n") if line and not line.startswith("#")]
selected = selected + extras
print("selected rows:", len(selected))
with open(OUT + "/mutations.txt", "a") as report:
    for number, row in enumerate(selected):
        name, file, original, replacement, cargo_args, named_test = row
        if os.path.exists(f"{OUT}/mutation-{number}.done"):
            continue
        if time.time() - START > BUDGET:
            print("预算用完，再调一次接着做")
            sys.exit(3)
        copy_tree()
        path = REPO + "/" + file
        source = open(path, encoding="utf-8").read()
        old = original.replace("\\n", "\n")
        assert source.count(old) == 1, f"锚点命中 {source.count(old)} 次：{name}"
        open(path, "w", encoding="utf-8").write(source.replace(old, replacement.replace("\\n", "\n")))
        words = cargo_args.split()
        assert words[:2] == ["-p", "singlefs-harness"], name
        separator = words.index("--")
        build = words[:separator]
        run_filters = words[separator + 1:]
        log = f"{OUT}/mutation-{number}.log"
        assert sh("6G", build + ["--no-run"], log + ".build") == 0, "变异后编译失败：" + name
        rc = sh("4G", build + ["--", *run_filters], log)
        cases, _ = results(log)
        red = sorted(case for case, state in cases if state == "FAILED")
        status = ("RED" if red else "SURVIVES") if named_test == "-" else ("RED" if named_test in red else "NOT_RED")
        report.write(f"{name}\t{named_test}\t{status}\t{len(cases)} 条跑过\t红 {len(red)}：{' '.join(red)}\n")
        report.flush()
        open(f"{OUT}/mutation-{number}.done", "w").close()
print("done")
