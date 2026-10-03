#!/usr/bin/env python3
"""攻方 J2-a 驱动一侧的探针：载入驱动，只换 Runner.run_step（不起任何子进程：交一个秒数，并照包装的 finish 往这一版
baseline-peaks.<线程数>.tsv 写这一步的峰值、照 runner 的 measure 模式写逐条时间表），看 ensure_baseline 换了 --threads 之后
往 baseline.tsv 追加不追加新行、recorded_baseline_seconds（升档取的、门禁对的那一个）变不变。
用法：j2a-baseline-probe.py <驱动路径> <草稿目录>"""
import importlib.util, os, shutil, sys, types
driver_path, work = sys.argv[1:3]
spec = importlib.util.spec_from_file_location("broad_mutation_probe", driver_path)
d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
d.TEST_MODE = False
home = os.path.join(work, "home"); shutil.rmtree(home, ignore_errors=True)
library = d.Library(home); library.fingerprint = "f" * 64
os.makedirs(library.version_directory())
seconds_of = {}          # 每一趟 run_step 交的秒数：第一趟冷编，之后热编
def fake_run_step(self, step, sequence, time_limit, command=None, selection_text="all", runner_mode=None, extra=None, memory_cap=None, attempt="first"):
    name = step[0]
    seconds = seconds_of[name].pop(0)
    with open(self.baseline_peaks_path(), "a", encoding="utf-8") as handle:     # 包装 finish 写的（这里只追加一行，driver 取第一行匹配的）
        handle.write(f"{512 * 1024 * 1024}\t2026-10-04T00:00:00Z\t24G\t{d.BASELINE_PEAK_KEY_PREFIX}{name}\t{self.arguments.threads}\n")
    if runner_mode == "measure":
        open(self.times_path(name), "a").close()
    return 0, seconds
d.Runner.run_step = fake_run_step
def runner_with(threads):
    runner = d.Runner.__new__(d.Runner)
    runner.library = library
    runner.arguments = types.SimpleNamespace(threads=threads, timeout_floor=900, timeout_multiplier=5, cpu_multiplier=3, cpu_floor=30,
                                             wall_multiplier=10, wall_floor=300, skip_stage_two=True, stage_two_eatmydata=False)
    runner.steps = d.step_table(runner.arguments, [])
    runner.fake_driver = None; runner.fake_driver_via_wrapper = False
    runner.baseline_seconds = {}; runner.stages_with_fresh_baseline = set(); runner.pruner = None; runner.runner_path = None
    return runner
seconds_of.update({"one-build": [312.4, 1.2, 1.3], "one-test": [95.0, 88.0, 90.0]})
for threads in (8, 8, 16):
    runner = runner_with(threads)
    runner.ensure_baseline("one")
    rows = d.read_table(library.version_file("baseline.tsv"), len(d.BASELINE_COLUMNS)).rows
    print(f"--threads {threads} 之后：baseline.tsv {len(rows)} 行；one-build 最后一行绿记录 = {runner.recorded_baseline_seconds('one-build')}，"
          f"这一趟 one-build 的整步限时 = {runner.time_limit_of('one-build')} 秒")
