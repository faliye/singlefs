#!/usr/bin/env python3
"""rows_orchestrator.py --selftest 用的假逐行驱动：命令行与产物格式照 row_driver.py（标准输出那几种行、--timing-output 的分块、行目录里的
row-result.tsv、device.out、dump/<流>/ 五个文件、o3.out），不编、不跑装置。限时与快照摘要用真的 device_timing.row_time_limit_seconds 与
row_driver.snapshot_digest（FAKE_RESEARCH_DIRECTORY 指到真的驱动目录）。
环境变量：FAKE_DRIVER_LOG（每行追加一行 ran / skipped / failed）、FAKE_DRIVER_HOST（台号，默认 1）、FAKE_DRIVER_ROW_SECONDS（每行睡几秒）、
FAKE_DRIVER_TIMEOUT_ROWS（这几行 device_exit=253；同一输出根里同键的上一次结果是超时的行，这一趟限时乘 3、不超时）、FAKE_DRIVER_FAIL_ROWS（这几行驱动出错、不写 row-result.tsv、退 1）、
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
    name = f"假 第 {number} 行{' 掺' if salted('row') else ''}"
    # 按键复用，照真驱动（row_driver.py）：同键、做完了、不是编不过 / 测量管子缺陷 / 超时的行目录直接读回；别的行目录先清再做；上一次同键是超时的，限时乘 3
    key = row_driver.reuse_key_of(arguments, number, name)
    earlier = row_driver.earlier_result_with_the_key(row_directory, key)
    if earlier is not None and row_driver.result_is_reusable(earlier):
        log(f"reused row={number} host={host} root={os.path.basename(arguments.output_root)}")
        return earlier
    timed_out_before = ((earlier is not None and earlier.get("device_exit") == str(row_driver.ROW_TIMEOUT_EXIT_CODE))
                        or row_driver.marked_as_timed_out_before(row_directory, key))
    if os.path.isdir(row_directory):
        shutil.rmtree(row_directory)
    os.makedirs(row_directory)
    if timed_out_before:
        with open(os.path.join(row_directory, row_driver.TIMED_OUT_MARKER_FILE), "w", encoding="utf-8") as handle:
            handle.write(key + "\n")
    time.sleep(float(os.environ.get("FAKE_DRIVER_ROW_SECONDS", "0")))
    if number in numbers("FAKE_DRIVER_FAIL_ROWS"):
        log(f"failed row={number} host={host}")
        return {"row_number": number, "status": "driver_error", "error": "fake"}
    substitution, runnable = substitution_fields(arguments, number, name)
    if not runnable:
        fields = {"row_number": number, "row_name": name, "lane": 0, row_driver.REUSE_KEY_FIELD: key, "line_matches_the_registration": "false", **substitution,
                  "status": "compile_failed",
                  "device_exit": "-", "copy_ns": 1, "compile_ns": 1, "compile_kind": "cold", "device_ns": "-", "o3_ns": "-", "o3_status": "not_reached"}
        with open(os.path.join(row_directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
            for key, value in fields.items():
                handle.write(f"{key}\t{value}\n")
        log(f"ran row={number} host={host} root={os.path.basename(arguments.output_root)} rerun={str(timed_out_before).lower()}")
        return fields
    time_limit_seconds = device_timing.row_time_limit_seconds(arguments.baseline_process_wall_seconds, timed_out_before)
    exit_code = 253 if number in numbers("FAKE_DRIVER_TIMEOUT_ROWS") and not timed_out_before else 0
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
    fields = {"row_number": number, "row_name": name, "lane": 0, row_driver.REUSE_KEY_FIELD: key, "packages_checked_rebuilt": "crates/singlefs-core",
              "substituted_from": substitution["substituted_from"], "line_matches_the_registration": "true" if substitution["substituted_from"] == "-" else "false",
              "mutated_row_name": substitution["mutated_row_name"], "failed_attempts": substitution["failed_attempts"],
              **{key: value for key, value in substitution.items() if key.startswith("failed_attempt_")},
              "mutation_line_sha256": hashlib.sha256(str(number).encode()).hexdigest(), "copy_ns": random.randint(1, 10**6),
              "compile_ns": random.randint(1, 10**9), "compile_kind": "cold", "status": "ok", "device_exit": exit_code, "device_ns": random.randint(1, 10**9),
              "time_limit_seconds": time_limit_seconds, "timed_out_before": str(timed_out_before).lower()}
    if exit_code == 0 and arguments.no_executor:
        fields.update({"o3_status": "skipped_by_request", "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    elif exit_code == 0:
        fields.update({"o3_status": "ran", "o3_exit": 0, "o3_ns": random.randint(1, 10**9), "o3_streams": ",".join(STREAMS)})
    else:
        fields.update({"o3_status": "not_reached", "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    with open(os.path.join(row_directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
        for key, value in fields.items():
            handle.write(f"{key}\t{value}\n")
    log(f"ran row={number} host={host} root={os.path.basename(arguments.output_root)} rerun={str(timed_out_before).lower()}")
    return fields


def main():
    parser = argparse.ArgumentParser()
    for name in ("--repo-root", "--work-root", "--output-root", "--snapshot-from", "--baseline-timing", "--timing-output"):
        parser.add_argument(name, required=True)
    parser.add_argument("--rows", default="")
    parser.add_argument("--streams", default="1,2,3")
    parser.add_argument("--threads", type=int, default=6)
    parser.add_argument("--lanes", type=int, default=1)
    for flag in ("--empty-mutation", "--writable-mount-probe", "--keep-dumps", "--no-executor"):
        parser.add_argument(flag, action="store_true")
    parser.add_argument("--other-selected-rows", action="append", default=[])
    parser.set_defaults(positive_control=None, disable_oracle=None, o3_control_bad_states=None)   # 复用键里的项，真驱动有这几个参数、这里固定为没给
    arguments = parser.parse_args()
    if os.path.exists(arguments.timing_output):
        parser.error("timing output exists")
    baseline = device_timing.read_baseline_timing(open(arguments.baseline_timing, encoding="utf-8").read().splitlines())
    arguments.baseline_process_wall_seconds = baseline.process_wall_seconds
    limit = device_timing.row_time_limit_seconds(baseline.process_wall_seconds, False)
    print(f"name=row_time_limit seconds={limit} baseline_seconds={baseline.process_wall_seconds:.3f} source={os.path.basename(arguments.baseline_timing)}", flush=True)
    if not arguments.no_executor:
        print("freeze_check before=true differences=0 detail=none")
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    row_driver.sync_tree(arguments.snapshot_from, snapshot_root)
    digest, count = row_driver.snapshot_digest(snapshot_root)
    arguments.snapshot_digest = digest
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
