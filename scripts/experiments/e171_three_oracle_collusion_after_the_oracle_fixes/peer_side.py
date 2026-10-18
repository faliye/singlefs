#!/usr/bin/env python3
"""E171 第一段多机跑法的第二台（及以后各台）那一头：rows_orchestrator.py 经 scripts/hosts/multi-host-run.sh --peer 把它与 expected.json
送进那边的树（research/e171-s1-peer-kit/），在那棵树的树根跑。只用标准库与树里的 row_driver、frozen_o3。

  peer_side.py preflight  --expected <json> --slots N [--threads T] [--compile]   （--threads 不给取登记的重活默认并行度）
      逐项核：Python 主次版本、google_crc32c 装没装（都与本机相同）、rsync 在、基线计时与驱动各文件的 sha256、发来的冻结快照摘要、O3 冻结行（Z2）、
      树所在的盘空余 ≥ 槽数 × 3 GiB；--compile 时把冻结快照拷一份、照驱动的编法离线编一遍（cargo build --release --offline -p singlefs-checker-tier
      --bin e171_three_oracle_collusion_after_the_oracle_fixes，经 capped.sh T）。报告写 <out>/preflight.txt（multi-host-run.sh --fetch 取回），
      一项一行 `preflight check=<项> ok=<true|false> detail=<…>`，末行 `preflight verdict=pass|refused failed=<项,…>`。全过退 0，否则退 1。
  peer_side.py run-driver --expected <json> -- <驱动参数…>
      先核驱动各文件、基线计时、快照摘要（不对就写 <out>/driver.refused、退 3，驱动不起），再在树根起驱动：
      python3 -B <驱动> --repo-root <树根> --work-root <树根>/scripts/experiments/target/e171-s1-peer-work --output-root <out>/rows
        --snapshot-from <树根>/<快照> --baseline-timing <树根>/<基线计时> --timing-output <out>/timing.out <驱动参数…>
      标准输出进 <out>/product.out，标准错误进 <out>/driver.stderr，退出码写 <out>/driver.exit，照驱动的退出码退出。
expected.json 的键：python、google_crc32c、baseline_timing [相对路径, sha256]、files {相对路径: sha256}、snapshot [相对路径, 摘要, 文件数]、
  research_directory、driver、disk_bytes_per_slot、out。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys

WORK_RELATIVE = "scripts/experiments/target/e171-s1-peer-work"
PREFLIGHT_TREE_RELATIVE = "scripts/experiments/target/e171-s1-preflight-tree"
PREFLIGHT_TARGET_RELATIVE = "scripts/experiments/target/e171-s1-preflight-target"
DEVICE_PACKAGE = "singlefs-checker-tier"
DEVICE_BINARY_NAME = "e171_three_oracle_collusion_after_the_oracle_fixes"


def sha256_of_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


RESEARCH_RELATIVE = "scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes"


def research_directory_on_path(expected: dict) -> None:
    """那边树里的研究目录放进 sys.path 第一位；expected.json 的 research_directory 要正是 RESEARCH_RELATIVE（编排器发来的就是它），
    路径照字面拼，函数单元库认得出装的是哪几份。"""
    if expected["research_directory"] != RESEARCH_RELATIVE:
        raise SystemExit(f"expected.json 的 research_directory 是 {expected['research_directory']}，不是 {RESEARCH_RELATIVE}")
    sys.path.insert(0, os.path.join(os.getcwd(), RESEARCH_RELATIVE))


def research_row_driver(expected: dict):
    research_directory_on_path(expected)
    import row_driver
    return row_driver


def research_frozen_o3(expected: dict):
    research_directory_on_path(expected)
    import frozen_o3
    return frozen_o3


def file_problems(expected: dict) -> list[str]:
    problems = []
    timing_path, timing_sha = expected["baseline_timing"]
    for path, sha in list(expected["files"].items()) + [(timing_path, timing_sha)]:
        if not os.path.exists(path):
            problems.append(f"missing:{path}")
        elif sha256_of_file(path) != sha:
            problems.append(f"sha256_differs:{path}")
    return problems


def snapshot_problem(expected: dict) -> str | None:
    path, digest, count = expected["snapshot"]
    if not os.path.isdir(path):
        return f"missing:{path}"
    found_digest, found_count = research_row_driver(expected).snapshot_digest(path)
    if (found_digest, str(found_count)) != (digest, str(count)):
        return f"digest:{found_digest}/{found_count}"
    return None


def preflight(arguments: argparse.Namespace, expected: dict) -> int:
    out = expected["out"]
    os.makedirs(out, exist_ok=True)
    checks: list[tuple[str, bool, str]] = []
    version = f"{sys.version_info.major}.{sys.version_info.minor}"
    checks.append(("python", version == expected["python"], f"here={version} expected={expected['python']}"))
    try:
        import google_crc32c  # noqa: F401
        crc = True
    except ImportError:
        crc = False
    checks.append(("google_crc32c", crc == expected["google_crc32c"], f"here={crc} expected={expected['google_crc32c']}"))
    checks.append(("rsync", shutil.which("rsync") is not None, "on_path" if shutil.which("rsync") else "absent"))
    problems = file_problems(expected)
    checks.append(("driver_and_baseline_files", not problems, ",".join(problems) or f"files={len(expected['files']) + 1}"))
    problem = snapshot_problem(expected)
    checks.append(("snapshot", problem is None, problem or "digest_matches"))
    differences = research_frozen_o3(expected).freeze_differences(os.getcwd())
    checks.append(("o3_freeze", not differences, ",".join(differences) or "differences=0"))
    free = shutil.disk_usage(".").free
    need = arguments.slots * int(expected["disk_bytes_per_slot"])
    checks.append(("disk", free >= need, f"free={free} need={need}"))
    if arguments.compile:
        subprocess.run(["rsync", "-a", "--delete", expected["snapshot"][0] + "/", PREFLIGHT_TREE_RELATIVE + "/"], check=True)
        log_path = os.path.join(out, "preflight-compile.log")
        environment = dict(os.environ, CARGO_TARGET_DIR=os.path.join(os.getcwd(), PREFLIGHT_TARGET_RELATIVE))
        with open(log_path, "w", encoding="utf-8") as log:
            threads_for_capped = "default" if arguments.threads is None else str(arguments.threads)   # 不给 --threads 取登记的重活默认并行度（capped.sh default）
            code = subprocess.run(["bash", os.path.join(os.getcwd(), "scripts/hosts/capped.sh"), threads_for_capped, "nice", "-n", "19", "cargo",
                                   "build", "--release", "--offline", "-p", DEVICE_PACKAGE, "--bin", DEVICE_BINARY_NAME],
                                  cwd=PREFLIGHT_TREE_RELATIVE, env=environment, stdout=log, stderr=subprocess.STDOUT).returncode
        checks.append(("offline_compile", code == 0, f"cargo_exit={code}"))
    failed = [name for name, ok, _ in checks if not ok]
    with open(os.path.join(out, "preflight.txt"), "w", encoding="utf-8") as report:
        for name, ok, detail in checks:
            report.write(f"preflight check={name} ok={'true' if ok else 'false'} detail={detail}\n")
        report.write(f"preflight verdict={'refused' if failed else 'pass'} failed={','.join(failed) or 'none'}\n")
    return 1 if failed else 0


def run_driver(expected: dict, driver_arguments: list[str]) -> int:
    out = expected["out"]
    os.makedirs(out, exist_ok=True)
    problems = file_problems(expected)
    problem = snapshot_problem(expected)
    if problem:
        problems.append(problem)
    if problems:
        with open(os.path.join(out, "driver.refused"), "w", encoding="utf-8") as handle:
            handle.write("\n".join(problems) + "\n")
        return 3
    root = os.getcwd()
    command = ["python3", "-B", expected["driver"], "--repo-root", root, "--work-root", os.path.join(root, WORK_RELATIVE),
               "--output-root", os.path.join(root, out, "rows"), "--snapshot-from", os.path.join(root, expected["snapshot"][0]),
               "--baseline-timing", os.path.join(root, expected["baseline_timing"][0]), "--timing-output", os.path.join(root, out, "timing.out")]
    with open(os.path.join(out, "product.out"), "x", encoding="utf-8") as product, open(os.path.join(out, "driver.stderr"), "x", encoding="utf-8") as err:
        code = subprocess.run(command + driver_arguments, stdout=product, stderr=err).returncode
    with open(os.path.join(out, "driver.exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")
    return code


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 第一段多机跑法：别的台那一头")
    parser.add_argument("mode", choices=("preflight", "run-driver"))
    parser.add_argument("--expected", required=True)
    parser.add_argument("--slots", type=int, default=1)
    parser.add_argument("--threads", type=int, default=None, help="离线编的线程数；不给取登记的重活默认并行度（capped.sh default）")
    parser.add_argument("--compile", action="store_true")
    if "--" in argument_list:
        split = argument_list.index("--")
        own, driver_arguments = argument_list[:split], argument_list[split + 1:]
    else:
        own, driver_arguments = argument_list, []
    arguments = parser.parse_args(own)
    with open(arguments.expected, encoding="utf-8") as handle:
        expected = json.load(handle)
    if arguments.mode == "preflight":
        return preflight(arguments, expected)
    return run_driver(expected, driver_arguments)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
