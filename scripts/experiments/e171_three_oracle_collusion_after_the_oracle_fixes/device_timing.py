"""E171 第 1 次重跑：装置一趟的进程挂钟与逐行限时（登记 `research/prompts/e171-r1-prereg.md` 修订 2 第 2.5、2.6 节）。

  python3 device_timing.py run -- <命令…>
      在被测进程起止各取一次时刻（`time.monotonic_ns`）。命令的标准输出与标准错误原样继承：这里不读、不转打它的输出行
      （`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计，不在转发输出的循环里计」）。命令退出之后往标准输出补一行
      `E7TIMING name=process_wall seconds=<秒> exit=<退出码>`，照命令的退出码退出。经 `run-with-memory-cap.sh` 与 `capped.sh` 起时放在它们里面、
      紧挨着装置，排队与建 scope 的时间不算进去。
  python3 device_timing.py check <-timing 产物>
      读那份产物里的 `process_wall` 与装置自己的各流 `stream_time` 的 `elapsed_ns`，核进程挂钟包住各流之和（2.5「包不住的那一次不算，重量一次」），
      打一行 `name=baseline_timing seconds=… stream_time_sum_seconds=… streams=… contains_the_stream_times=…`；包不住、或读不到 `process_wall`、
      或一条 `stream_time` 都没有，退 1。
逐行限时（2.5）：`row_time_limit_seconds`，每行装置那一步的限时 = max(1800, ⌈3 × T⌉) 秒；超时的点名行单独再跑一次时再乘 3。
"""

from __future__ import annotations

import dataclasses
import math
import subprocess
import sys
import time

# 2.5：限时只升不降，不低于原登记读法 5 的 1800 秒
ROW_TIME_LIMIT_FLOOR_SECONDS = 1800
# 2.5：T 的倍数（不从任何一行的读数来）
BASELINE_MULTIPLIER = 3
# 2.5：超时的点名行单独再跑一次时，限时再乘这么多
TIMED_OUT_ROW_RERUN_MULTIPLIER = 3
NANOSECONDS_PER_SECOND = 1_000_000_000
PROCESS_WALL_PREFIX = "E7TIMING name=process_wall "
STREAM_TIME_PREFIX = "E7TIMING name=stream_time "


@dataclasses.dataclass(frozen=True)
class BaselineTiming:
    """一份 `-timing` 产物里的进程挂钟（秒）与装置各流 `stream_time` 的 `elapsed_ns` 之和；`process_wall_seconds` 是 None 时那份产物没有 `process_wall` 行。"""

    process_wall_seconds: float | None
    stream_time_sum_nanoseconds: int
    streams: int

    def contains_the_stream_times(self) -> bool:
        return (self.process_wall_seconds is not None and self.streams > 0
                and self.process_wall_seconds * NANOSECONDS_PER_SECOND >= self.stream_time_sum_nanoseconds)


def fields_of(line: str) -> dict[str, str]:
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def read_baseline_timing(lines: list[str]) -> BaselineTiming:
    """有几行 `process_wall` 取最后一行（一趟装置只打一行；多于一行说明一份产物里接了两趟，取最后一趟不对，交回 None 让调用方拒）。"""
    walls = [float(fields_of(line)["seconds"]) for line in lines if line.startswith(PROCESS_WALL_PREFIX)]
    elapsed = [int(fields_of(line)["elapsed_ns"]) for line in lines if line.startswith(STREAM_TIME_PREFIX)]
    return BaselineTiming(walls[0] if len(walls) == 1 else None, sum(elapsed), len(elapsed))


def row_time_limit_seconds(baseline_seconds: float, rerun_of_a_timed_out_row: bool) -> int:
    limit = max(ROW_TIME_LIMIT_FLOOR_SECONDS, math.ceil(BASELINE_MULTIPLIER * baseline_seconds))
    return limit * TIMED_OUT_ROW_RERUN_MULTIPLIER if rerun_of_a_timed_out_row else limit


def run_and_time(command: list[str]) -> int:
    started = time.monotonic_ns()
    completed = subprocess.run(command)
    wall_nanoseconds = time.monotonic_ns() - started
    sys.stdout.flush()
    print(f"{PROCESS_WALL_PREFIX}seconds={wall_nanoseconds / NANOSECONDS_PER_SECOND:.3f} exit={completed.returncode}", flush=True)
    return completed.returncode


def check(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        timing = read_baseline_timing(handle.read().splitlines())
    seconds = "none" if timing.process_wall_seconds is None else f"{timing.process_wall_seconds:.3f}"
    print(f"name=baseline_timing seconds={seconds} stream_time_sum_seconds={timing.stream_time_sum_nanoseconds / NANOSECONDS_PER_SECOND:.3f} "
          f"streams={timing.streams} contains_the_stream_times={str(timing.contains_the_stream_times()).lower()}")
    return 0 if timing.contains_the_stream_times() else 1


def main(argument_list: list[str]) -> int:
    if len(argument_list) >= 3 and argument_list[0] == "run" and argument_list[1] == "--":
        return run_and_time(argument_list[2:])
    if len(argument_list) == 2 and argument_list[0] == "check":
        return check(argument_list[1])
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
