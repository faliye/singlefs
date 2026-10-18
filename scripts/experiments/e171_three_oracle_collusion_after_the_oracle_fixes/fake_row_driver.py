#!/usr/bin/env python3
"""rows_orchestrator.py --selftest 用的假逐行驱动：命令行与产物格式照 row_driver.py（标准输出那几种行、--timing-output 的分块、行目录里的
row-result.tsv、device.out、dump/<流>/ 五个文件、o3.out），不编、不跑装置。限时与快照摘要用真的 device_timing.row_time_limit_seconds 与
row_driver.snapshot_digest（FAKE_RESEARCH_DIRECTORY 指到真的驱动目录）。
环境变量：FAKE_DRIVER_LOG（每行追加一行 ran / skipped / failed）、FAKE_DRIVER_HOST（台号，默认 1）、FAKE_DRIVER_ROW_SECONDS（每行睡几秒）、
FAKE_DRIVER_TIMEOUT_ROWS（这几行 device_exit=253，带 --rerun-of-timed-out-rows 时不超时）、FAKE_DRIVER_FAIL_ROWS（这几行驱动出错、不写 row-result.tsv、退 1）、
FAKE_DRIVER_SALT_HOST 与 FAKE_DRIVER_SALT_PART（在那一台上往 device / dump / o3 / row 那一样里掺一个不同）。
第二段（rows_orchestrator_segment_two.py）另用：--no-executor（不写 o3.out、o3_status=skipped_by_request、不打 freeze_check 行，同真驱动）；
--other-selected-rows（可给几次，名字用真的 bad_implementation_table.read_selected_rows 读，补位时跳过）；
FAKE_DRIVER_SUBSTITUTES=<行>:<候选,候选…>;…（这几行原名编不过，按候选次序换第一个不在 --other-selected-rows 里的，记 substituted_from、
failed_attempt_1=compile_failed:<原名>；候选全被占就记 status=compile_failed）。
"""
import argparse
import hashlib
import os
import random
import shutil
import sys
import time

# FAKE_RESEARCH_DIRECTORY 是编排器自证给的真驱动目录（就是这一份所在的研究目录，自证把这一份拷进假世界里起）；
# 缺省取这一份所在的目录
sys.path.insert(0, os.environ.get("FAKE_RESEARCH_DIRECTORY") or os.path.dirname(os.path.abspath(__file__)))
import device_timing  # noqa: E402
import row_driver  # noqa: E402

STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
Z1_FILES = ("oracle-verdicts.tsv", "call-outcomes.tsv", "writable-mount-probe.tsv", "ops.tsv", "states.tsv")


def numbers(name):
    return {int(token) for token in os.environ.get(name, "").split(",") if token}


def log(text):
    if os.environ.get("FAKE_DRIVER_LOG"):
        with open(os.environ["FAKE_DRIVER_LOG"], "a", encoding="utf-8") as handle:
            handle.write(text + "\n")


def salted(part):
    return os.environ.get("FAKE_DRIVER_SALT_PART") == part and os.environ.get("FAKE_DRIVER_SALT_HOST") == os.environ.get("FAKE_DRIVER_HOST", "1")


def substitute_candidates():
    found = {}
    for part in os.environ.get("FAKE_DRIVER_SUBSTITUTES", "").split(";"):
        if ":" in part:
            row, names = part.split(":", 1)
            found[int(row)] = [name for name in names.split(",") if name]
    return found


def names_in_other_tables(arguments):
    import bad_implementation_table  # noqa: E402（FAKE_RESEARCH_DIRECTORY 已在 sys.path 里）
    return {row.name for path in arguments.other_selected_rows for row in bad_implementation_table.read_selected_rows(path)}


def substitution_fields(arguments, number, name):
    """原名编不过时照补位规则换候选：交回要并进 row-result.tsv 的字段与这一行还跑不跑装置。"""
    candidates = substitute_candidates().get(number)
    if candidates is None:
        return {"substituted_from": "-", "mutated_row_name": name, "failed_attempts": 0}, True
    taken = names_in_other_tables(arguments)
    chosen = next((candidate for candidate in candidates if candidate not in taken), None)
    fields = {"substituted_from": name if chosen else "-", "mutated_row_name": chosen or "-", "failed_attempts": 1,
              "failed_attempt_1": f"compile_failed:{name}"}
    return fields, chosen is not None


def run_row(arguments, number, host):
    row_directory = os.path.join(arguments.output_root, f"row-{number}")
    if arguments.skip_finished and os.path.isdir(row_directory):
        if os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            log(f"skipped row={number} host={host} root={os.path.basename(arguments.output_root)}")
            return dict(line.rstrip("\n").split("\t", 1) for line in open(os.path.join(row_directory, "row-result.tsv"), encoding="utf-8"))
        shutil.rmtree(row_directory)
    os.makedirs(row_directory)
    time.sleep(float(os.environ.get("FAKE_DRIVER_ROW_SECONDS", "0")))
    if number in numbers("FAKE_DRIVER_FAIL_ROWS"):
        log(f"failed row={number} host={host}")
        return {"row_number": number, "status": "driver_error", "error": "fake"}
    name = f"假 第 {number} 行{' 掺' if salted('row') else ''}"
    substitution, runnable = substitution_fields(arguments, number, name)
    if not runnable:
        fields = {"row_number": number, "row_name": name, "lane": 0, "line_matches_the_registration": "false", **substitution, "status": "compile_failed",
                  "device_exit": "-", "copy_ns": 1, "compile_ns": 1, "compile_kind": "cold", "device_ns": "-", "o3_ns": "-", "o3_status": "not_reached"}
        with open(os.path.join(row_directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
            for key, value in fields.items():
                handle.write(f"{key}\t{value}\n")
        log(f"ran row={number} host={host} root={os.path.basename(arguments.output_root)} rerun={str(arguments.rerun_of_timed_out_rows).lower()}")
        return fields
    exit_code = 253 if number in numbers("FAKE_DRIVER_TIMEOUT_ROWS") and not arguments.rerun_of_timed_out_rows else 0
    probe = "true" if arguments.writable_mount_probe else "false"
    with open(os.path.join(row_directory, "device.out"), "w", encoding="utf-8") as handle:
        handle.write(f"E7RESULT name=config segment=s1 mode=row streams={arguments.streams} writable_mount_probe={probe}\n")
        handle.write(f"E7RESULT name=row_result row={number} probe={probe}{' salt=1' if salted('device') else ''}\n")
        handle.write(f"E7TIMING name=stage_time stream=x total_ns={random.randint(1, 10**9)}\n")
        handle.write(f"E7TIMING name=process_wall seconds={random.random():.3f} exit={exit_code}\n")
    for stream in STREAMS:
        os.makedirs(os.path.join(row_directory, "dump", stream))
        for name in Z1_FILES:
            with open(os.path.join(row_directory, "dump", stream, name), "w", encoding="utf-8") as handle:
                handle.write(f"{stream} {name} row={number}{' salt' if salted('dump') and name == 'states.tsv' else ''}\n")
    if not arguments.no_executor:
        with open(os.path.join(row_directory, "o3.out"), "w", encoding="utf-8") as handle:
            handle.write(f"name=config dumps={row_directory}/dump workers={arguments.threads} python=fake\n")
            handle.write(f"name=load stream=new_pool_file_creation load_ns={random.randint(1, 10**9)} ops=3\n")
            handle.write(f"name=verdict_summary row={number} strict_not_in=0{' salt=1' if salted('o3') else ''}\n")
            handle.write(f"name=state_time batch=1 workers={arguments.threads}\n")
    fields = {"row_number": number, "row_name": name, "lane": 0, "packages_checked_rebuilt": "crates/singlefs-core",
              "substituted_from": substitution["substituted_from"], "line_matches_the_registration": "true" if substitution["substituted_from"] == "-" else "false",
              "mutated_row_name": substitution["mutated_row_name"], "failed_attempts": substitution["failed_attempts"],
              **{key: value for key, value in substitution.items() if key.startswith("failed_attempt_")},
              "mutation_line_sha256": hashlib.sha256(str(number).encode()).hexdigest(), "copy_ns": random.randint(1, 10**6),
              "compile_ns": random.randint(1, 10**9), "compile_kind": "cold", "status": "ok", "device_exit": exit_code, "device_ns": random.randint(1, 10**9)}
    if exit_code == 0 and arguments.no_executor:
        fields.update({"o3_status": "skipped_by_request", "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    elif exit_code == 0:
        fields.update({"o3_status": "ran", "o3_exit": 0, "o3_ns": random.randint(1, 10**9), "o3_streams": ",".join(STREAMS)})
    else:
        fields.update({"o3_status": "not_reached", "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    with open(os.path.join(row_directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
        for key, value in fields.items():
            handle.write(f"{key}\t{value}\n")
    log(f"ran row={number} host={host} root={os.path.basename(arguments.output_root)} rerun={str(arguments.rerun_of_timed_out_rows).lower()}")
    return fields


def main():
    parser = argparse.ArgumentParser()
    for name in ("--repo-root", "--work-root", "--output-root", "--snapshot-from", "--baseline-timing", "--timing-output"):
        parser.add_argument(name, required=True)
    parser.add_argument("--rows", default="")
    parser.add_argument("--streams", default="1,2,3")
    parser.add_argument("--threads", type=int, default=6)
    parser.add_argument("--lanes", type=int, default=1)
    parser.add_argument("--memory-cap", default="4G")
    for flag in ("--empty-mutation", "--writable-mount-probe", "--keep-dumps", "--skip-finished", "--rerun-of-timed-out-rows", "--no-executor"):
        parser.add_argument(flag, action="store_true")
    parser.add_argument("--other-selected-rows", action="append", default=[])
    arguments = parser.parse_args()
    if os.path.exists(arguments.timing_output):
        parser.error("timing output exists")
    baseline = device_timing.read_baseline_timing(open(arguments.baseline_timing, encoding="utf-8").read().splitlines())
    limit = device_timing.row_time_limit_seconds(baseline.process_wall_seconds, arguments.rerun_of_timed_out_rows)
    print(f"name=row_time_limit seconds={limit} baseline_seconds={baseline.process_wall_seconds:.3f} source={os.path.basename(arguments.baseline_timing)} "
          f"rerun_of_timed_out_rows={str(arguments.rerun_of_timed_out_rows).lower()}", flush=True)
    if not arguments.no_executor:
        print("freeze_check before=true differences=0 detail=none")
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    row_driver.sync_tree(arguments.snapshot_from, snapshot_root)
    digest, count = row_driver.snapshot_digest(snapshot_root)
    jobs = ([0] if arguments.empty_mutation else []) + sorted(int(token) for token in arguments.rows.split(",") if token)
    print(f"snapshot_digest={digest} files={count} snapshot_ns={random.randint(1, 10**6)} rows={len(jobs)}")
    os.makedirs(arguments.output_root, exist_ok=True)
    host = os.environ.get("FAKE_DRIVER_HOST", "1")
    results = [run_row(arguments, number, host) for number in jobs]
    for fields in results:
        print("row " + " ".join(f"{key}={str(value).replace(' ', '_')}" for key, value in fields.items()))
    print(f"table lanes={arguments.lanes} threads_per_lane={arguments.threads} rows={len(jobs)} table_wall_ns={random.randint(1, 10**9)} sum_of_row_ns=7")
    with open(arguments.timing_output, "x", encoding="utf-8") as handle:
        for fields in results:
            handle.write(f"E7TIMING name=row_device row={fields['row_number']} device_exit={fields.get('device_exit', '-')} device_ns={fields.get('device_ns', '-')}\n")
            device_out = os.path.join(arguments.output_root, f"row-{fields['row_number']}", "device.out")
            if os.path.exists(device_out):
                handle.writelines(line for line in open(device_out, encoding="utf-8") if line.startswith("E7TIMING "))
    if not arguments.no_executor:
        print("freeze_check before=false differences=0 detail=none")
    return 1 if any(fields.get("status") == "driver_error" for fields in results) else 0


if __name__ == "__main__":
    sys.exit(main())
