#!/usr/bin/env python3
"""E171 多机跑法（第一段 rows_orchestrator.py、第二段 rows_orchestrator_segment_two.py）的别的台那一头：编排那一片经 shard_relay.py
把它交给 task-server 派到那一台的那一片代跑（rows_orchestrator.py 文件头「机器」「经 task-server」）。仓与工作根经 NFS 两台挂在同一绝对路径：
那一台在同一棵共享的树里、同一个绝对路径上起它（工作目录就是仓根），读编排器写在共享根下的 expected.json 与冻结树快照，产物直接写进
--out 给的共享目录，编排那一片照同一个绝对路径读；不送、不取回。编译与逐行的工作树放那一台的本地盘：代跑那一片给每条命令的
MULTI_HOST_RUN_PEER_SCRATCH（不在共享根下，那条命令跑完由代跑那一片连同编译目录删掉）。只用标准库与研究目录里的 row_driver、frozen_o3。

  peer_side.py preflight  --expected <json> --out <目录> --slots N [--threads T] [--compile]   （--threads 不给取登记的重活默认并行度）
      逐项核：Python 主次版本、google_crc32c 装没装（都与本机相同）、rsync 在、基线计时与驱动各文件的 sha256、冻结树快照摘要、O3 冻结行（Z2）、
      本地盘目录给了、空余 ≥ 槽数 × 3 GiB；--compile 时把冻结快照拷进本地盘、照驱动的编法离线编一遍（cargo build --release --offline
      -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes，直接起；给了 T 时线程变量设成 T，不给照服务设的）。报告写 <out>/preflight.txt，
      一项一行 `preflight check=<项> ok=<true|false> detail=<…>`，末行 `preflight verdict=pass|refused failed=<项,…>`。全过退 0，否则退 1。
  peer_side.py run-driver --expected <json> --out <目录> -- <驱动参数…>
      先核驱动各文件、基线计时、快照摘要与本地盘目录（不对就写 <out>/driver.refused、退 3，驱动不起），再在仓根起驱动：
      python3 -B <驱动> --repo-root <仓根> --work-root <本地盘>/e171-peer-work --output-root <out>/rows
        --snapshot-from <快照> --baseline-timing <基线计时> --timing-output <out>/timing.out <驱动参数…>
      标准输出进 <out>/product.out，标准错误进 <out>/driver.stderr，退出码写 <out>/driver.exit，照驱动的退出码退出。
expected.json 的键（路径都是两台相同的绝对路径）：python、google_crc32c、baseline_timing [路径, sha256]、files {路径: sha256}、
  snapshot [路径, 摘要, 文件数]、research_directory（要正是这一份所在的目录）、driver、disk_bytes_per_slot。
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SCRATCH_VARIABLE = "MULTI_HOST_RUN_PEER_SCRATCH"
WORK_DIRECTORY_NAME = "e171-peer-work"
PREFLIGHT_TREE_DIRECTORY_NAME = "e171-preflight-tree"
PREFLIGHT_TARGET_DIRECTORY_NAME = "e171-preflight-target"
DEVICE_PACKAGE = "singlefs-checker-tier"
DEVICE_BINARY_NAME = "e171_three_oracle_collusion_after_the_oracle_fixes"


def sha256_of_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def research_directory_on_path(expected: dict) -> None:
    """研究目录（这一份所在的目录）放进 sys.path 第一位；expected.json 的 research_directory 要正是它（编排器写的就是它），
    路径照字面拼，函数单元库认得出装的是哪几份。"""
    if os.path.realpath(expected["research_directory"]) != os.path.realpath(HERE):
        raise SystemExit(f"expected.json 的 research_directory 是 {expected['research_directory']}，不是这一份所在的 {HERE}")
    sys.path.insert(0, HERE)


def research_row_driver(expected: dict):
    research_directory_on_path(expected)
    import row_driver
    return row_driver


def research_frozen_o3(expected: dict):
    research_directory_on_path(expected)
    import frozen_o3
    return frozen_o3


def local_scratch() -> str | None:
    """那一台本地盘上这条命令的目录（代跑那一片给的）；没给或不是目录交 None。"""
    directory = os.environ.get(SCRATCH_VARIABLE, "")
    return directory if directory and os.path.isdir(directory) else None


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
    out = arguments.out
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
    scratch = local_scratch()
    checks.append(("local_scratch", scratch is not None, "given" if scratch else f"{SCRATCH_VARIABLE} 没给或不是目录"))
    if scratch is not None:
        free = shutil.disk_usage(scratch).free
        need = arguments.slots * int(expected["disk_bytes_per_slot"])
        checks.append(("disk", free >= need, f"free={free} need={need}"))
    if arguments.compile and scratch is not None:
        tree = os.path.join(scratch, PREFLIGHT_TREE_DIRECTORY_NAME)
        subprocess.run(["rsync", "-a", "--delete", expected["snapshot"][0] + "/", tree + "/"], check=True)
        log_path = os.path.join(out, "preflight-compile.log")
        environment = dict(os.environ, CARGO_TARGET_DIR=os.path.join(scratch, PREFLIGHT_TARGET_DIRECTORY_NAME))
        if arguments.threads is not None:   # 不给 --threads 照服务给这一片设的线程变量（登记的重活默认并行度）
            environment = research_frozen_o3(expected).thread_environment(arguments.threads, environment)
        with open(log_path, "w", encoding="utf-8") as log:
            code = subprocess.run(["nice", "-n", "19", "cargo",
                                   "build", "--release", "--offline", "-p", DEVICE_PACKAGE, "--bin", DEVICE_BINARY_NAME],
                                  cwd=tree, env=environment, stdout=log, stderr=subprocess.STDOUT).returncode
        checks.append(("offline_compile", code == 0, f"cargo_exit={code}"))
    failed = [name for name, ok, _ in checks if not ok]
    with open(os.path.join(out, "preflight.txt"), "w", encoding="utf-8") as report:
        for name, ok, detail in checks:
            report.write(f"preflight check={name} ok={'true' if ok else 'false'} detail={detail}\n")
        report.write(f"preflight verdict={'refused' if failed else 'pass'} failed={','.join(failed) or 'none'}\n")
    return 1 if failed else 0


def run_driver(arguments: argparse.Namespace, expected: dict, driver_arguments: list[str]) -> int:
    out = arguments.out
    os.makedirs(out, exist_ok=True)
    problems = file_problems(expected)
    problem = snapshot_problem(expected)
    if problem:
        problems.append(problem)
    scratch = local_scratch()
    if scratch is None:
        problems.append(f"local_scratch:{SCRATCH_VARIABLE} 没给或不是目录")
    if problems:
        with open(os.path.join(out, "driver.refused"), "w", encoding="utf-8") as handle:
            handle.write("\n".join(problems) + "\n")
        return 3
    command = ["python3", "-B", expected["driver"], "--repo-root", os.getcwd(), "--work-root", os.path.join(scratch, WORK_DIRECTORY_NAME),
               "--output-root", os.path.join(out, "rows"), "--snapshot-from", expected["snapshot"][0],
               "--baseline-timing", expected["baseline_timing"][0], "--timing-output", os.path.join(out, "timing.out")]
    with open(os.path.join(out, "product.out"), "x", encoding="utf-8") as product, open(os.path.join(out, "driver.stderr"), "x", encoding="utf-8") as err:
        code = subprocess.run(command + driver_arguments, stdout=product, stderr=err).returncode
    with open(os.path.join(out, "driver.exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")
    return code


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 多机跑法：别的台那一头")
    parser.add_argument("mode", choices=("preflight", "run-driver"))
    parser.add_argument("--expected", required=True)
    parser.add_argument("--out", required=True, help="这一个分片的输出目录（共享根下的绝对路径，本机照同一个路径读）")
    parser.add_argument("--slots", type=int, default=1)
    parser.add_argument("--threads", type=int, default=None, help="离线编的线程数；不给照服务设的线程变量（登记的重活默认并行度）")
    parser.add_argument("--compile", action="store_true")
    if "--" in argument_list:
        split = argument_list.index("--")
        own, driver_arguments = argument_list[:split], argument_list[split + 1:]
    else:
        own, driver_arguments = argument_list, []
    arguments = parser.parse_args(own)
    if not os.path.isabs(arguments.out):
        parser.error(f"--out 要给绝对路径（两台照同一个路径读写），给的是 {arguments.out}")
    with open(arguments.expected, encoding="utf-8") as handle:
        expected = json.load(handle)
    if arguments.mode == "preflight":
        return preflight(arguments, expected)
    return run_driver(arguments, expected, driver_arguments)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
