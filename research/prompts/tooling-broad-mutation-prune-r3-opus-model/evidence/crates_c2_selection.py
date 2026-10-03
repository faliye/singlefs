#!/usr/bin/env python3
"""真 crates 副本上的 C2：第一段（harness 档四个包、不含 #[ignore]）每条测试单独一个进程插桩跑一遍，得到「执行到的函数」；
再对库目录这一版清单的每条变异挑「执行到被改函数」的测试，与库里的结局比：
  - 库里判成 caught 的，抓它的那条测试在不在 C2 挑出来的集合里（不在就是挑漏的候选，要在副本上真跑核）；
  - 每条变异 C2 要跑的测试按库里基线日志的耗时加起来，与第一段整遍比（U3）。

用法：
  crates_c2_selection.py cover  <副本根> <插桩编译目录> <llvm bin 目录> <输出目录> <并行数>
  crates_c2_selection.py select <副本根> <库目录这一版> <输出目录>
cover 可重跑：输出目录里已有结果的测试跳过。测试的线程变量一律设成 1（并行数 × 1 不超过派发的线程上限）。
"""
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import time

PACKAGES = ("singlefs-harness", "singlefs-checker", "singlefs-core", "singlefs-format")
THREAD_VARIABLES = ("RUST_TEST_THREADS", "SINGLEFS_LAYER0_THREADS", "SINGLEFS_CRASH_INJECTION_THREADS",
                    "SINGLEFS_FAULT_INJECTION_WORKER_THREADS", "SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS",
                    "SINGLEFS_RANDOM_HISTORY_THREADS", "SINGLEFS_THREAD_CAP")


def binaries(root, target):
    env = dict(os.environ, CARGO_TARGET_DIR=target, RUSTFLAGS="-C instrument-coverage",
               LLVM_PROFILE_FILE=os.path.join(target, "build-%p.profraw"))
    command = ["cargo", "test", "--locked", "--offline", "--no-run", "--message-format=json"]
    for package in PACKAGES:
        command += ["-p", package]
    out = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True)
    if out.returncode != 0:
        raise SystemExit(out.stderr[-3000:])
    found = []
    for line in out.stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact" or not message.get("executable") or not message["profile"]["test"]:
            continue
        package = message["package_id"].split("#")[0].rsplit("/", 1)[-1].split("@")[0]
        target_info = message["target"]
        kind = target_info["kind"][0]
        binary_id = package if kind == "lib" else (f"{package}::bin/{target_info['name']}" if kind == "bin" else f"{package}::{target_info['name']}")
        found.append((binary_id, message["executable"], os.path.dirname(message["manifest_path"])))
    return found


def listed(executable, cwd, env, ignored):
    command = [executable, "--list", "--format", "terse"] + (["--ignored"] if ignored else [])
    out = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True, check=True).stdout
    return {line[: -len(": test")] for line in out.splitlines() if line.endswith(": test")}


def cover_one(job):
    index, binary_id, executable, cwd, test, out_dir, llvm, root = job
    result_path = os.path.join(out_dir, "tests", f"{index}.json")
    if os.path.exists(result_path):
        return json.load(open(result_path))
    profiles = os.path.join(out_dir, "profiles", str(index))
    os.makedirs(profiles, exist_ok=True)
    env = dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, "%p.profraw"))
    for variable in THREAD_VARIABLES:
        env[variable] = "1"
    started = time.monotonic()
    run = subprocess.run([executable, "--exact", test, "--test-threads=1"], cwd=cwd, env=env, capture_output=True, text=True)
    wall = time.monotonic() - started
    files = sorted(os.path.join(profiles, f) for f in os.listdir(profiles))
    executed = []
    if files:
        data = os.path.join(profiles, "merged.profdata")
        subprocess.run([os.path.join(llvm, "llvm-profdata"), "merge", "-sparse", *files, "-o", data], check=True, capture_output=True)
        export = subprocess.run([os.path.join(llvm, "llvm-cov"), "export", executable, f"-instr-profile={data}"],
                                check=True, capture_output=True, text=True).stdout
        for function in json.loads(export)["data"][0]["functions"]:
            if function["count"] <= 0:
                continue
            regions = [r for r in function["regions"] if r[5] == 0]
            name = function["filenames"][0]
            # 同一个编译目录先后编过两份副本时，cargo 会沿用没改过的 crate 的产物，覆盖里的文件名指着前一份副本：按「/crates/」之后归一
            if not regions or "/crates/" not in name or "/.cargo/registry/" in name:
                continue
            executed.append(["crates/" + name.split("/crates/", 1)[1], min(r[0] for r in regions), max(r[2] for r in regions)])
    for f in os.listdir(profiles):
        os.remove(os.path.join(profiles, f))
    record = {"index": index, "binary_id": binary_id, "test": test, "exit": run.returncode, "wall": round(wall, 3),
              "profraw_files": len(files), "executed": sorted(set(map(tuple, executed)))}
    with open(result_path + ".tmp", "w") as handle:
        json.dump(record, handle)
    os.replace(result_path + ".tmp", result_path)
    return record


def cover(root, target, llvm, out_dir, workers):
    root = os.path.abspath(root)
    os.makedirs(os.path.join(out_dir, "tests"), exist_ok=True)
    found = binaries(root, target)
    listing_env = dict(os.environ, LLVM_PROFILE_FILE=os.path.join(out_dir, "listing-%p.profraw"))
    jobs = []
    for binary_id, executable, cwd in sorted(found):
        for test in sorted(listed(executable, cwd, listing_env, False) - listed(executable, cwd, listing_env, True)):
            jobs.append((len(jobs), binary_id, executable, cwd, test, out_dir, llvm, root))
    print(f"CRATES-COVER binaries={len(found)} tests={len(jobs)}", flush=True)
    started = time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        records = list(pool.map(cover_one, jobs))
    failed = [f"{r['binary_id']} {r['test']} exit={r['exit']}" for r in records if r["exit"] != 0]
    print(f"CRATES-COVER done tests={len(records)} failed={len(failed)} wall-seconds={time.monotonic() - started:.0f} "
          f"sum-test-wall={sum(r['wall'] for r in records):.0f} tests-with-child-profiles={sum(r['profraw_files'] > 1 for r in records)}")
    for line in failed:
        print("CRATES-COVER failed", line)


def baseline_seconds(library):
    seconds = {}
    pattern = re.compile(r"PASS \[\s*([0-9.]+)s\]\s+\(\s*\d+/\d+\)\s+(\S+)\s+(\S+)")
    for line in open(os.path.join(library, "logs", "baseline.one-test.log")):
        m = pattern.search(line)
        if m:
            seconds[f"{m.group(2)} {m.group(3)}"] = float(m.group(1))
    return seconds


def select(root, library, out_dir):
    records = [json.load(open(os.path.join(out_dir, "tests", f))) for f in os.listdir(os.path.join(out_dir, "tests")) if f.endswith(".json")]
    by_file = {}
    for record in records:
        key = f"{record['binary_id']} {record['test']}"
        for file, start, end in record["executed"]:
            by_file.setdefault(file, []).append((start, end, key))
    seconds = baseline_seconds(library)
    missing_seconds = sorted(f"{r['binary_id']} {r['test']}" for r in records if f"{r['binary_id']} {r['test']}" not in seconds)
    full = sum(seconds.values())
    results = {}
    for line in open(os.path.join(library, "results.tsv")):
        parts = line.rstrip("\n").split("\t")
        if parts and parts[0].isdigit():
            results[int(parts[0])] = parts
    rows = []
    for mutant in json.load(open(os.path.join(library, "list.json"))):
        line = mutant["start"][0]
        selected = sorted({key for start, end, key in by_file.get(mutant["file"], []) if start <= line <= end})
        library_row = results.get(mutant["sequence"])
        outcome = library_row[3] if library_row else "none"
        catcher = library_row[5] if library_row else ""
        rows.append({"sequence": mutant["sequence"], "package": mutant["package"], "file": mutant["file"], "line": line,
                     "selected": len(selected), "selected_seconds": round(sum(seconds.get(k, 0.0) for k in selected), 3),
                     "outcome": outcome, "stage": library_row[4] if library_row else "", "catcher": catcher,
                     "catcher_selected": (catcher in selected) if catcher else None})
    with open(os.path.join(out_dir, "selection.tsv"), "w") as handle:
        handle.write("sequence\tpackage\tfile\tline\tselected\tselected_seconds\toutcome\tstage\tcatcher\tcatcher_selected\n")
        for row in rows:
            handle.write("\t".join(str(row[k]) for k in ("sequence", "package", "file", "line", "selected", "selected_seconds",
                                                         "outcome", "stage", "catcher", "catcher_selected")) + "\n")
    caught = [r for r in rows if r["outcome"] == "caught" and r["stage"] == "one"]
    outside = [r for r in caught if not r["catcher_selected"]]
    print(f"CRATES-SELECT tests-covered={len(records)} baseline-tests={len(seconds)} covered-without-baseline-seconds={len(missing_seconds)} full-stage-one-seconds={full:.1f}")
    print(f"CRATES-SELECT caught-in-stage-one={len(caught)} catcher-in-selected={len(caught) - len(outside)} catcher-not-in-selected={len(outside)}")
    for r in outside:
        print(f"CRATES-SELECT catcher-not-selected sequence={r['sequence']} {r['file']}:{r['line']} selected={r['selected']} catcher={r['catcher']}")
    for package in ("singlefs-core", "singlefs-checker", "all"):
        chosen = [r for r in rows if package == "all" or r["package"] == package]
        sizes = sorted(r["selected"] for r in chosen)
        secs = sorted(r["selected_seconds"] for r in chosen)
        zero = sum(1 for s in sizes if s == 0)
        print(f"CRATES-SELECT package={package} mutants={len(chosen)} zero-coverage={zero} selected-median={sizes[len(sizes) // 2]} "
              f"selected-p90={sizes[int(len(sizes) * 0.9)]} selected-max={sizes[-1]} seconds-median={secs[len(secs) // 2]:.1f} "
              f"seconds-p90={secs[int(len(secs) * 0.9)]:.1f} seconds-max={secs[-1]:.1f} seconds-sum={sum(secs):.0f}")
    for line in missing_seconds[:10]:
        print("CRATES-SELECT covered-test-not-in-baseline", line)


if __name__ == "__main__":
    if sys.argv[1] == "cover":
        cover(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5], int(sys.argv[6]))
    elif sys.argv[1] == "select":
        select(os.path.abspath(sys.argv[2]), sys.argv[3], sys.argv[4])
