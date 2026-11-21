"""broad-mutation.py 拆出的一份。
自证（二）：基线、服务、编译目录、登记、GPU 等几组格，格函数表、串并行、run_selftest。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import importlib.util
import json
import os
import re
import shutil
import signal
import sys
import tempfile
import time
from broad_mutation_base import (DRIVER_ERROR, EXIT_BASELINE_RED, EXIT_GATE_RED, EXIT_STOPPED, EXIT_USAGE, GPU_BASELINE_SEQUENCE, GPU_BUILD_STEP,
    GPU_TEST_STEP, HeavyThreadDefaultError, SCRIPT_DIRECTORY, SELFTEST_STEP_DRIVER_NAME,
    SELF_PATH, STAGE_ONE_ONLY, VERIFICATION_PACKAGE, default_heavy_threads)
from broad_mutation_judging import (raw_break, refuse, say)
from broad_mutation_library import (prune_server_client)
from broad_mutation_gpu import (gpu_bindgen_arguments)
from broad_mutation_selftest_a import (Selftest, selftest_baseline_entry_cells, selftest_baseline_key_cells,
    selftest_baseline_threads_cells, selftest_basic_cells, selftest_environment_cells, selftest_fresh_build_cells, selftest_global_cells,
    selftest_fixture_reader_cells, selftest_key_speed_cells, selftest_progress_cells, selftest_prune_cells, selftest_refused_cells,
    selftest_registered_cells, selftest_reuse_cells, selftest_run_directory_cells, selftest_service_cells, selftest_shared_library_cells,
    selftest_source_copy, selftest_state_cells, selftest_stop_cells, selftest_sync_cells, selftest_table_cells, selftest_test_variable_cells,
    summary_field)
import broad_mutation_selftest_a


# ── 自证 ───────────────────────────────────────────────────────────────────────────────────────────────────

# 自证的夹具（假的一步与清单驱动的脚本正文、假源码树、计划表）在同目录的 lib_broad_mutation_selftest_fixtures.py：run_selftest 开头载入，
# 只有自证的函数读它。
selftest_fixtures = None


# 夹具要读的模块各存一份 selftest_fixtures（载入之后放进去）
FIXTURE_MODULES = (broad_mutation_selftest_a,)


def load_selftest_fixtures():
    global selftest_fixtures
    specification = importlib.util.spec_from_file_location("lib_broad_mutation_selftest_fixtures", os.path.join(SCRIPT_DIRECTORY, "lib_broad_mutation_selftest_fixtures.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    selftest_fixtures = module
    for part in FIXTURE_MODULES:
        vars(part)["selftest_fixtures"] = module


def gpu_item_lines(output, prefix="BROAD_MUTATION_GPU_ITEM"):
    """输出里每条一行的 GPU 趟结论：{序号: 那一行}。"""
    found = {}
    for line in output.splitlines():
        if line.startswith(prefix + " "):
            found[int(re.search(r"sequence=(\d+)", line).group(1))] = line
    return found


def gpu_lines_of(output, prefix):
    """输出里以 prefix 起头的行，每行拆成 {键: 值}（制表符分段的明细行拆不了，交原样）。"""
    return [dict(token.split("=", 1) for token in line.split()[1:] if "=" in token) for line in output.splitlines() if line.startswith(prefix + " ")]


def selftest_gpu_cells(selftest):
    """GPU 那一趟（C489）：只有 CPU 红、只有 GPU 红、两趟都没红、没卡，各一格；GPU 只给 CPU 没红又有带 gpu 特性的测试够得着的条跑；
    命令的写法（特性、目标、不另套清场、--test-threads 1、BINDGEN）；认不出的退出码 / 无效 / 一个测试都没跑到；中途没卡、基线红；复用与点名现判（自证变量）；
    第二段没跑（自证变量）时 GPU 趟不起；--list-items 的状态是两趟合起来的。假的一步当假 GPU（SELFTEST_STEP_DRIVER 的 gpu- 那几步）。"""
    source = selftest_source_copy(selftest, "gpu-source", selftest_fixtures.SELFTEST_GPU_FILES)
    covered = {"1", "2", "4", "5"}
    two = ("--item", "1", "--item", "2")

    def gpu_run(home, plan, *arguments, **environment):
        selftest.write_plan(plan)
        selftest.clear_calls()
        return selftest.run(home, *arguments, source=source, **environment)

    def summary_of(home):
        return selftest.run(home, "--summary", source=source)

    def gate_fields(text):
        found = gpu_lines_of(text, "BROAD_MUTATION_GATE")
        return found[0] if found else {}

    try:
        # ① 只有 CPU 红：第 1 条 CPU 那一趟（第一段）抓到，GPU 那一趟不起；第 2 条 CPU 没红、GPU 也没红 ⇒ 没红
        ran = gpu_run("gpu-cpu-red", [("1", "one-test", "fail-nextest")], *two)
        cpu_rows = {row["sequence"]: (row["outcome"], row["stage"]) for row in selftest.rows("gpu-cpu-red")}
        gpu_rows = [(row["sequence"], row["outcome"]) for row in selftest.gpu_rows("gpu-cpu-red")]
        items = gpu_item_lines(ran.stdout)
        summary = summary_of("gpu-cpu-red")
        selftest.check(ran.returncode == 0 and cpu_rows == {"1": ("caught", "one"), "2": ("missed", "two")} and gpu_rows == [("2", "missed")]
                       and selftest.gpu_judged_sequences() == {"2"} and set(items) == {2} and " cpu=missed gpu=missed combined=missed " in items[2]
                       and " judged=1 red=1 reused=0 " in items[2] and "BROAD_MUTATION_GPU_PLAN needed=1 judge=1 carried=0 " in ran.stdout
                       and "GPU 趟（要 1 条）：现判 1、复用 0、本次未跑 0" in ran.stdout
                       and summary_field(summary.stdout, "caught") == 1 and summary_field(summary.stdout, "missed") == 1 and summary.returncode == EXIT_GATE_RED
                       and (gate_fields(summary.stdout).get("gpu_needed"), gate_fields(summary.stdout).get("gpu_reused")) == ("1", "1"),
                       f"只有 CPU 红：CPU 那一趟抓到的条也起了 GPU 那一趟、或两趟的结局没分开记（退 {ran.returncode}；CPU 行 {cpu_rows}；GPU 行 {gpu_rows}；GPU 趟现判过 "
                       f"{sorted(selftest.gpu_judged_sequences())}）：{ran.stdout[-500:]}{ran.stderr[-300:]}",
                       "看 run_gpu_phase：只给 CPU 那一趟落的结论是没红的条起 GPU 那一趟（一条两趟里任一趟红算抓到，CPU 红了就不用跑）；GPU 的结局落 gpu-verdicts/，"
                       "不进 verdicts/，BROAD_MUTATION_GPU_ITEM 一行分开写 cpu= 与 gpu=")
        selftest.check(bool(selftest.gpu_rows("gpu-cpu-red"))
                       and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-cpu-red")} == {("1", "caught"), ("2", STAGE_ONE_ONLY), ("2", "missed")},
                       "GPU 那一趟的结局没有单独存进 GPU 判定表（判定表 verdicts/ 里只该有 CPU 那一趟的三行）", "看 record_gpu_verdict：GPU 趟的判定行落 gpu-verdicts/，CPU 那一趟的 verdicts/ 一行不改")

        # ② 只有 GPU 红：CPU 那一趟都没红；第 1 条 GPU 的结果行被 --nocapture 打散（认名字靠 failures: 名单），第 2 条是整齐的结果行 ⇒ 合起来都是抓到
        ran = gpu_run("gpu-gpu-red", [("1", "gpu-test-verification", "gpu-fail-scrambled"), ("2", "gpu-test-verification", "gpu-fail")], *two)
        gpu_rows = {row["sequence"]: row for row in selftest.gpu_rows("gpu-gpu-red")}
        items = gpu_item_lines(ran.stdout)
        summary = summary_of("gpu-gpu-red")
        listed = selftest.states("gpu-gpu-red", source)
        selftest.check(ran.returncode == 0 and set(gpu_rows) == {"1", "2"} and all(row["outcome"] == "caught" and row["caught_by"] == selftest_fixtures.SELFTEST_GPU_CASE
                                                                                  and re.fullmatch(r"[0-9a-f]{64}", row["test_key"]) for row in gpu_rows.values())
                       and all(" cpu=missed gpu=caught combined=caught " in line and " red=0 " in line for line in items.values()) and len(items) == 2
                       and summary_field(summary.stdout, "caught") == 2 and summary_field(summary.stdout, "missed") == 0
                       and (listed.get("1"), listed.get("2")) == ("caught", "caught"),
                       f"只有 GPU 红：GPU 那一趟红了没算抓到、抓到它的测试没认出来、或 --list-items 的状态不是两趟合起来的（退 {ran.returncode}；GPU 行 "
                       f"{[(sequence, row['outcome'], row['caught_by']) for sequence, row in sorted(gpu_rows.items())]}；--list-items {listed}）：{ran.stdout[-400:]}{ran.stderr[-300:]}",
                       "看 gpu_combined_state（GPU 抓到就是抓到）、gpu_failing_test（--nocapture 下结果行被打散，认 failures: 名单）、GpuJudging.catching_key 与 list_items 的 combined_states")

        # ③ 两趟都没红：CPU 没红、GPU 也没红 ⇒ 没红（汇总的没红栏）；GPU 那一趟只给有带特性的测试够得着的 4 条里 CPU 没红的跑（全量跑：8 条 CPU 都没红）
        ran = gpu_run("gpu-neither", [])
        gpu_rows = {row["sequence"]: row for row in selftest.gpu_rows("gpu-neither")}
        summary = summary_of("gpu-neither")
        missed_lines = [line.split("\t")[1] for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]
        environment = selftest.gpu_environment()
        selftest.check(ran.returncode == 0 and set(gpu_rows) == covered and all(row["outcome"] == "missed" and re.fullmatch(r"[0-9a-f]{64}", row["coverage_key"])
                                                                                  for row in gpu_rows.values())
                       and selftest.gpu_judged_sequences() == covered and summary_field(summary.stdout, "missed") == 8 and sorted(missed_lines) == sorted(str(n) for n in range(1, 9))
                       and (gate_fields(summary.stdout).get("gpu_needed"), gate_fields(summary.stdout).get("gpu_reused")) == ("4", "4") and summary.returncode == EXIT_GATE_RED
                       and gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_TOTAL")[0].get("judged") == "4",
                       f"两趟都没红：没记成没红、或 GPU 那一趟跑了闭包够不着的条（GPU 行 {sorted(gpu_rows)}；GPU 趟现判过 {sorted(selftest.gpu_judged_sequences())}；"
                       f"汇总没红 {summary_field(summary.stdout, 'missed')}、MISSED {missed_lines}）：{ran.stdout[-400:]}{ran.stderr[-300:]}",
                       "看 GpuJudging.need：只给「有一条跑得到的测试，它带特性的闭包含容器单元、又真引到 gpu 原子」的条跑 GPU 那一趟；没红的 CPU + GPU 两趟都没红才记没红，汇总与门禁行照合起来的状态数")
        # 命令的写法：特性是 tests/ 里 cfg 点到的、目标是够得着的那几个、测试步直接起（不另套清场）并加 --nocapture --test-threads 1、BINDGEN_EXTRA_CLANG_ARGS 取崩溃放量那一份
        bindgen = gpu_bindgen_arguments()
        build_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_BUILD_STEP]
        test_commands = [json.loads(entry[3]) for entry in environment if entry[1] == GPU_TEST_STEP]
        right_test = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", "gpu", "--test", "gpu_pair",
                      "--", "--nocapture", "--test-threads", "1"]
        right_build = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", "gpu", "--test", "gpu_pair", "--no-run"]
        selftest.check(bindgen and len(environment) == 2 * (1 + len(covered)) and build_commands == [right_build] * (1 + len(covered)) and test_commands == [right_test] * (1 + len(covered))
                       and all(entry[2] == bindgen for entry in environment),
                       f"GPU 那一趟的命令不对（BINDGEN {bindgen[:40]!r}；收到 {len(environment)} 步；编译步 {build_commands[:1]}；测试步 {test_commands[:1]}）",
                       "看 gpu_pass_command：编译步 cargo test --locked --release -p singlefs-verification --features <特性> <目标> --no-run；测试步同一条直接起（不套清场包装），"
                       "后面加 -- --nocapture --test-threads 1；BINDGEN_EXTRA_CLANG_ARGS 取 gpu_bindgen_arguments（crash-amplification-lib.sh 的那一份）")

        # ④ 没卡：基线那一趟日志里有 NOT_RUN_NO_GPU_ADAPTER ⇒ 两条逐条报本次未跑、不落 GPU 行、不记没红；之后有卡了重跑同一条命令补上
        ran = gpu_run("gpu-no-card", [("gpu-baseline", GPU_TEST_STEP, "gpu-no-adapter")], *two)
        summary = summary_of("gpu-no-card")
        not_run = gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")
        no_conclusion = [line.split("\t") for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t")]
        gate = gate_fields(summary.stdout)
        selftest.check(ran.returncode == 0 and not selftest.gpu_rows("gpu-no-card") and sorted(entry["sequence"] for entry in not_run) == ["1", "2"]
                       and all(entry["reason"] == "no-adapter" for entry in not_run) and selftest.gpu_judged_sequences() == set()
                       and "GPU 趟（要 2 条）：现判 0、复用 0、本次未跑 2" in ran.stdout and summary_field(summary.stdout, "missed") == 0
                       and (gate.get("gpu_needed"), gate.get("gpu_reused"), gate.get("gpu_not_run")) == ("2", "0", "2")
                       and {entry[1] for entry in no_conclusion if entry[-1].startswith("gpu-not-run")} == {"1", "2"} and summary.returncode == EXIT_GATE_RED,
                       f"没卡：没报本次未跑、记成了没红、或落了 GPU 行（退 {ran.returncode}；本次未跑 {not_run}；GPU 行 {selftest.gpu_rows('gpu-no-card')}；汇总没红 {summary_field(summary.stdout, 'missed')}）："
                       f"{ran.stdout[-500:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test（日志里有 NOT_RUN_NO_GPU_ADAPTER 是 not-run，不是没红）与 GpuPass.ensure_baseline、run_gpu_phase 的 BROAD_MUTATION_GPU_NOT_RUN：本次未跑的不落行、不进没红栏，"
                       "汇总里它们是 none（理由 gpu-not-run），门禁行的 gpu_not_run= 数得出")
        # 有卡了：同一条命令 CPU 那一趟沿用跳过，GPU 那一趟补上
        selftest.write_plan([])
        selftest.clear_calls()
        again = selftest.run("gpu-no-card", *two, source=source)
        selftest.check(again.returncode == 0 and {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-no-card")} == {"1": "missed", "2": "missed"}
                       and "GPU 趟（要 2 条）：现判 2、复用 0、本次未跑 0" in again.stdout,
                       f"有卡之后重跑同一条命令没把 GPU 那一趟补上（退 {again.returncode}）：{again.stdout[-400:]}{again.stderr[-300:]}",
                       "看 run_items 里「选中 0 条要跑」那一支：CPU 那一趟没有要现判的，也要接着跑 run_gpu_phase")

        # 中途没卡（基线有、第 1 条测试步打了 NOT_RUN_NO_GPU_ADAPTER）：这一条和后面的都本次未跑，已落的行不动
        ran = gpu_run("gpu-card-gone", [("1", GPU_TEST_STEP, "gpu-no-adapter")], *two)
        not_run = gpu_lines_of(ran.stdout, "BROAD_MUTATION_GPU_NOT_RUN")
        selftest.check(ran.returncode == 0 and not selftest.gpu_rows("gpu-card-gone") and sorted(entry["sequence"] for entry in not_run) == ["1", "2"]
                       and selftest.gpu_judged_sequences() == {"1"},
                       f"中途没卡：这一条之后的没停下、或落了行（GPU 行 {selftest.gpu_rows('gpu-card-gone')}；本次未跑 {not_run}；GPU 趟现判过 {sorted(selftest.gpu_judged_sequences())}）",
                       "看 run_gpu_phase 的 stopped_for：测试步见到 NOT_RUN_NO_GPU_ADAPTER，这一条不落行、后面的条不再起，逐条报本次未跑；树里的原件照拷回")

        # 基线红：没改过的树上 GPU 那一趟就不绿 ⇒ 拒（退 6）、一条 GPU 行都不落、树回到原样；CPU 那一趟已落的行在
        ran = gpu_run("gpu-baseline-red", [("gpu-baseline", GPU_TEST_STEP, "gpu-fail")], *two)
        selftest.check(ran.returncode == EXIT_BASELINE_RED and "GPU 趟基线" in ran.stderr and not selftest.gpu_rows("gpu-baseline-red") and {(row["sequence"], row["outcome"]) for row in selftest.rows("gpu-baseline-red")} == {("1", STAGE_ONE_ONLY), ("1", "missed"), ("2", STAGE_ONE_ONLY), ("2", "missed")}
                       and selftest.gpu_judged_sequences() == set(),
                       f"GPU 趟基线红没拒成「基线不绿」（退 {ran.returncode}）：{ran.stderr[-400:]}", "看 GpuPass.ensure_baseline：没改过的树上编译红、测试红、一个测试都没跑到，退 6 并写出路")

        # 认不出的退出码、一个测试都没跑到、编不过：没结论的（驱动错）不当定论，编不过是无效
        ran = gpu_run("gpu-without-conclusion", [("1", GPU_TEST_STEP, "exit:104"), ("2", GPU_TEST_STEP, "gpu-zero")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-without-conclusion")}
        summary = summary_of("gpu-without-conclusion")
        reasons = {line.split("\t")[1]: line.split("\t")[-1] for line in summary.stdout.splitlines() if line.startswith("NO_CONCLUSION\t")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": DRIVER_ERROR, "2": DRIVER_ERROR} and reasons.get("1") == f"gpu-{DRIVER_ERROR}" and reasons.get("2") == f"gpu-{DRIVER_ERROR}"
                       and gate_fields(summary.stdout).get("gpu_without_conclusion") == "2" and summary_field(summary.stdout, "missed") == 0,
                       f"GPU 趟认不出的退出码 / 一个测试都没跑到不是没结论（GPU 行 {gpu_rows}；理由 {reasons}）：{ran.stdout[-300:]}{ran.stderr[-300:]}",
                       "看 gpu_classify_test：认不出的退出码、退 0 而一个测试都没跑到都是驱动错，不是没红；汇总里它们是没结论（gpu_without_conclusion=）")
        ran = gpu_run("gpu-unviable", [("1", GPU_BUILD_STEP, "fail-compile")], *two)
        gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-unviable")}
        selftest.check(ran.returncode == 0 and gpu_rows == {"1": "unviable", "2": "missed"} and selftest.states("gpu-unviable", source).get("1") == "unviable",
                       f"GPU 趟编不过没记成无效（GPU 行 {gpu_rows}）：{ran.stdout[-300:]}{ran.stderr[-300:]}", "看 gpu_classify_build 与 build_gpu_item、run_gpu_phase 里记编不过的那一支：编译步退 101 又有带源码位置的编译错误才是无效")

        # 记红、撤绿、下趟现判（GPU 一趟）：gpu-neither 里 CPU 一趟、GPU 一趟都没红，同一条命令再跑，没红的每趟现判（CPU 八条、GPU 趟四条都重判，一条都不沿用）；
        # 弄坏开关 missed-carried（没红照旧沿用）下一条都不现判
        selftest.write_plan([])
        selftest.clear_calls()
        reused = selftest.run("gpu-neither", source=source)
        selftest.check(reused.returncode == 0 and selftest.gpu_judged_sequences() == covered and selftest.judged_sequences() == {str(number) for number in range(1, 9)}
                       and "BROAD_MUTATION_GPU_PLAN needed=4 judge=4 carried=0 " in reused.stdout
                       and "GPU 趟（要 4 条）：现判 4、复用 0、本次未跑 0" in reused.stdout and len(selftest.gpu_rows("gpu-neither")) == 8,
                       f"GPU 趟没红的条下一趟没现判（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 趟现判的 {sorted(selftest.gpu_judged_sequences())}，应是 CPU 八条、GPU 趟 {sorted(covered)}）："
                       f"{reused.stdout[-400:]}{reused.stderr[-300:]}",
                       "看 GpuJudging.state：真跑时这一趟自己的键下同一把键的定论里最后一个是没红的，状态换成沿用不了（理由 missed-rejudged）；Judgement.state 对 CPU 一趟同样（GPU 趟也没红的条 CPU 一趟的没红行不沿用）")
        # 抓到照沿用、点名现判（自证变量）：gpu-gpu-red 里 CPU 一趟没红、GPU 一趟抓到（合起来是抓到）：同一条命令再跑，CPU 一趟的没红行与 GPU 趟的抓到都沿用，一步都不起；
        # 点名现判第 2 条（BROAD_MUTATION_REJUDGE_NAMED）：GPU 趟追加一行（这一回没红）
        selftest.write_plan([])
        selftest.clear_calls()
        carried_green = selftest.run("gpu-gpu-red", *two, source=source)
        selftest.check(carried_green.returncode == 0 and not selftest.gpu_calls() and not selftest.judged_sequences()
                       and "BROAD_MUTATION_GPU_PLAN needed=2 judge=0 carried=2 " in carried_green.stdout
                       and "GPU 趟（要 2 条）：现判 0、复用 2、本次未跑 0" in carried_green.stdout and len(selftest.gpu_rows("gpu-gpu-red")) == 2,
                       f"GPU 趟抓到的条（CPU 一趟没红）下一趟没沿用（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 步 {selftest.gpu_calls()[:3]}）：{carried_green.stdout[-400:]}{carried_green.stderr[-300:]}",
                       "看 GpuJudging.state 与 row_holds：键相同、gpu_key 相同、抓到的测试键相同才沿用；Judgement.state：要 GPU 趟、GPU 趟的真实状态不是没红的条（gpu_decided_keys），CPU 一趟的没红行照沿用；run_gpu_phase 里沿用得了的不选")
        rerun = selftest.run("gpu-gpu-red", "--item", "2", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        selftest.check(rerun.returncode == 0 and selftest.gpu_judged_sequences() == {"2"} and len([row for row in selftest.gpu_rows("gpu-gpu-red") if row["sequence"] == "2"]) == 2,
                       f"点名现判（自证变量）没有重跑 GPU 那一趟并追加新行（GPU 步 {selftest.gpu_calls()}）：{rerun.stdout[-300:]}{rerun.stderr[-300:]}",
                       "看 run_gpu_phase：设了 BROAD_MUTATION_REJUDGE_NAMED 时沿用得了的也现判，GPU 判定库只追加")
        # 先抓到、同一把键又判没红（GPU 一趟）：第 2 条的 GPU 行是抓到、没红两行，最后一个定论是没红 ⇒ 红照键记着、前面的抓到撤掉：下一趟 CPU 一趟、GPU 一趟都现判
        selftest.clear_calls()
        flipped_gpu = selftest.run("gpu-gpu-red", "--item", "2", source=source)
        selftest.check(flipped_gpu.returncode == 0 and selftest.gpu_judged_sequences() == {"2"} and selftest.judged_sequences() == {"2"}
                       and "BROAD_MUTATION_GPU_PLAN needed=1 judge=1 carried=0 " in flipped_gpu.stdout,
                       f"GPU 趟先抓到、同一把键又判没红的条下一趟没现判（CPU 一趟现判的 {sorted(selftest.judged_sequences())}、GPU 趟现判的 {sorted(selftest.gpu_judged_sequences())}，应都是 [2]）："
                       f"{flipped_gpu.stdout[-400:]}{flipped_gpu.stderr[-300:]}",
                       "看 GpuJudging.state：同一把键（这一趟自己的键下）全部定论里最后一个是没红的，不看更早的抓到还认不认；gpu_decided_keys：GPU 趟的真实状态是没红的，CPU 一趟的没红行也不沿用")
        # 第二段没跑（BROAD_MUTATION_STAGE_ONE_ONLY）：GPU 趟接在第二段后面，不起
        ran = gpu_run("gpu-stage-one", [], *two, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        selftest.check(ran.returncode == 0 and not selftest.gpu_calls() and "第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），不跑" in ran.stdout and not selftest.gpu_rows("gpu-stage-one"),
                       f"第二段没跑时还是起了 GPU 那一趟（{selftest.gpu_calls()[:2]}）", "看 run_gpu_phase 开头的 stage_two_skipped")
    finally:
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
        selftest.clear_calls()


def selftest_gpu_build_first_cells(selftest):
    """先编后跑（文件头「GPU 那一趟」）：第 1、2、4 条 CPU 那一趟都没红、都要 GPU 那一趟，第 4 条 GPU 趟编不过。假的一步外面套一层：记序号、步名、收到的
    GPU_CARDS_BY_PRIORITY 与测试步经 BROAD_MUTATION_GPU_STASH 读到的那一份可执行文件的内容；编译步照 cargo test --no-run 打一行 Executable，
    把「可执行文件」（内容是树里那份源码此刻改没改过）先删再写进编译目录（同链接器：换一个新文件，不就地改写），再换成原来那一份接着跑。
    环境里给 GPU_CARDS_BY_PRIORITY=selftest-card-0（不领卡，docstring「不领卡」）。判：GPU 趟的步依次是编译步（逐条三次、基线一次）、再是测试步（基线、第 1、2 条），
    第 4 条不起测试步，每一步收到的卡串都是环境里给的那一串；第 1、2 条的测试步各照自己留下的那一份跑（内容是改过的），基线的测试步不经留下的；
    GPU 行第 1、2 条没红、第 4 条无效。弄坏开关 gpu-builds-interleaved（每一条在跑它的测试步之前才编）下这一格判错。"""
    source = selftest_source_copy(selftest, "gpu-build-first-source", selftest_fixtures.SELFTEST_GPU_FILES)
    step_driver = os.path.join(selftest.work, SELFTEST_STEP_DRIVER_NAME)
    inner_driver = step_driver + ".inner"
    steps_log = os.path.join(selftest.work, "gpu-build-first-steps.log")
    executable_name = "gpu_pair-0123456789abcdef"
    wrapper = ("#!/usr/bin/env python3\nimport os, sys\n"
               "step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]\n"
               "stash, kept = os.environ.get('BROAD_MUTATION_GPU_STASH', ''), '-'\n"
               f"if step == {GPU_TEST_STEP!r} and stash:\n"
               f"    path = os.path.join(stash, {executable_name!r})\n"
               "    kept = open(path).read().strip() if os.path.exists(path) else 'absent'\n"
               f"with open({steps_log!r}, 'a') as log:\n"
               "    log.write(f\"{sequence}\\t{step}\\t{os.environ.get('GPU_CARDS_BY_PRIORITY', '-')}\\t{kept}\\n\")\n"
               f"if step == {GPU_BUILD_STEP!r}:\n"
               "    mutated = any('MUTATED_' in open(os.path.join(tree, path)).read() for path in ('crates/singlefs-core/src/lib.rs', 'crates/singlefs-checker/src/lib.rs'))\n"
               "    deps = os.path.join(os.environ['CARGO_TARGET_DIR'], 'release', 'deps')\n"
               "    os.makedirs(deps, exist_ok=True)\n"
               f"    executable = os.path.join(deps, {executable_name!r})\n"
               "    if os.path.exists(executable):\n"
               "        os.remove(executable)\n"
               "    with open(executable, 'w') as handle:\n"
               "        handle.write('mutated' if mutated else 'pristine')\n"
               "    os.chmod(executable, 0o755)\n"
               "    print(f'  Executable tests/gpu_pair.rs ({executable})', flush=True)\n"
               f"os.execv({inner_driver!r}, [{inner_driver!r}] + sys.argv[1:])\n")
    if os.path.exists(steps_log):
        os.remove(steps_log)
    os.replace(step_driver, inner_driver)
    try:
        with open(step_driver, "w") as handle:
            handle.write(wrapper)
        os.chmod(step_driver, 0o755)
        selftest.write_plan([("4", GPU_BUILD_STEP, "fail-compile")])
        selftest.clear_calls()
        ran = selftest.run("gpu-build-first", "--item", "1", "--item", "2", "--item", "4", source=source, GPU_CARDS_BY_PRIORITY="selftest-card-0")
    finally:
        os.replace(inner_driver, step_driver)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    steps = [tuple(line.rstrip("\n").split("\t")) for line in open(steps_log)] if os.path.exists(steps_log) else []
    gpu_steps = [(entry[0], entry[1], entry[2], entry[3]) for entry in steps if entry[1].startswith("gpu-")]
    gpu_rows = {row["sequence"]: row["outcome"] for row in selftest.gpu_rows("gpu-build-first")}
    selftest.check(ran.returncode == 0
                   and gpu_steps == [("1", GPU_BUILD_STEP, "selftest-card-0", "-"), ("2", GPU_BUILD_STEP, "selftest-card-0", "-"), ("4", GPU_BUILD_STEP, "selftest-card-0", "-"),
                                     (GPU_BASELINE_SEQUENCE, GPU_BUILD_STEP, "selftest-card-0", "-"), (GPU_BASELINE_SEQUENCE, GPU_TEST_STEP, "selftest-card-0", "-"),
                                     ("1", GPU_TEST_STEP, "selftest-card-0", "mutated"), ("2", GPU_TEST_STEP, "selftest-card-0", "mutated")]
                   and gpu_rows == {"1": "missed", "2": "missed", "4": "unviable"},
                   f"GPU 趟先编后跑：逐条与基线的编译没有全在测试步之前、某一步收到的卡串不是环境里给的那一串、或测试步没照这一条留下的可执行文件跑（退 {ran.returncode}；"
                   f"GPU 步 {gpu_steps}；GPU 行 {gpu_rows}）：{ran.stdout[-400:]}{ran.stderr[-300:]}",
                   "看 run_gpu_phase：要现判的条逐条 build_gpu_item（改坏、编、留下 Executable 报的可执行文件、拷回原件），再 ensure_baseline_build，之后才只起测试步"
                   "（ensure_baseline_test、run_gpu_item 经 GpuPass.run_test_from_stash 交 BROAD_MUTATION_GPU_STASH），编不过的条照 gpu_classify_build 记、不起测试步；"
                   "环境里的 GPU_CARDS_BY_PRIORITY 不改、原样交给每一步")


def selftest_service_build_directory_cells(selftest):
    """服务给的编译目录（docstring「库目录」runs/ 那一段）：设了 CARGO_TARGET_DIR，同一份源码（mtime 早于产物）连跑两趟，运行目录都在 <它>/broad-mutation-run/、
    跑完不删，第二趟照用第一趟编的产物：没改过的 singlefs-core 不重编（第一趟改坏又拷回的 singlefs-checker 重编）；库目录 runs/ 下不剩运行目录。
    弄坏开关 service-build-directory-ignored 下判错。"""
    service = os.path.join(selftest.work, "service-build")
    source = selftest_source_copy(selftest, "service-build-source", {"crates/singlefs-core/src/extra.rs": "pub const VERSION: u32 = 1;\n"})
    for found_directory, _directory_names, file_names in os.walk(source):
        for file_name in file_names:
            os.utime(os.path.join(found_directory, file_name), ns=(selftest_fixtures.FRESH_BUILD_COMMIT_TIME_NS, selftest_fixtures.FRESH_BUILD_COMMIT_TIME_NS))
    cargo_log = os.path.join(selftest.work, "cargo.log")

    def compiled():
        if not os.path.exists(cargo_log):
            return set()
        with open(cargo_log) as handle:
            return {(fields[0], fields[2]) for fields in (line.rstrip("\n").split("\t") for line in handle) if len(fields) == 3}

    selftest.write_plan([("0", "one-build", "cargo-build"), ("0", "one-test", "cargo-test")])
    try:
        if os.path.exists(cargo_log):
            os.remove(cargo_log)
        first = selftest.run("service-build", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", CARGO_TARGET_DIR=service)
        built_before = compiled()
        if os.path.exists(cargo_log):
            os.remove(cargo_log)
        second = selftest.run("service-build", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1",
                              CARGO_TARGET_DIR=service)
        rebuilt = compiled()
    finally:
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    kept = os.path.isdir(os.path.join(service, "broad-mutation-run", "target")) and os.path.isdir(os.path.join(service, "broad-mutation-run", "tree"))
    left = selftest.build_leftovers("service-build")
    selftest.check(first.returncode == 0 and second.returncode == 0 and {("0", "singlefs-core"), ("0", "singlefs-checker")} <= built_before
                   and ("0", "singlefs-core") not in rebuilt and kept and not left,
                   f"服务给的编译目录：没用它、跑完删了它，或第二趟没照用第一趟的产物（第一趟退 {first.returncode}、编过 {sorted(built_before)}；第二趟退 {second.returncode}、"
                   f"编过 {sorted(rebuilt)}；{service}/broad-mutation-run 里的树与编译目录{'都在' if kept else '不全'}；库目录里还剩 {left}）：{second.stdout[-300:]}{second.stderr[-300:]}",
                   "看 service_build_directory 与 Library.begin_run_in_service_build_directory：设了 CARGO_TARGET_DIR 运行目录用 <它>/broad-mutation-run/、"
                   "Library.end_run 不删它（弄坏开关 service-build-directory-ignored 下同没给时一样在 runs/ 下新建、跑完删）")


def run_selftest_cells(selftest, only=None):
    """自证的各组格，照 selftest_cell_functions 的次序；only 给了只跑点名的几组（环境变量 BROAD_MUTATION_SELFTEST_CELLS，开发时分段跑用：有的组接着前面组留下的库目录，单跑要连前面那一组一起点）。"""
    for cell in selftest_cell_functions():
        if only is None or cell.__name__ in only:
            cell(selftest)


def selftest_rocksdb_entry_cells(selftest):
    """真跑起 cargo 之前判 RocksDB 预编库的取值：被判的树里 验证包依赖 rocksdb、进程环境里四个取值是空的 ⇒ 拒（退 2）、假的一步一次都没起、报出 entry 的红句。
    别的格的树（SELFTEST_SOURCES）没有依赖 rocksdb 的清单，entry 报不判。"""
    tree = selftest_source_copy(selftest, "rocksdb-source", {"crates/singlefs-verification/Cargo.toml": selftest_fixtures.selftest_package("singlefs-verification")
                                                            + '\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n'})
    calls_before = len(selftest.calls())
    refused = selftest.run("rocksdb", source=tree, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check(refused.returncode == EXIT_USAGE and len(selftest.calls()) == calls_before and "rocksdb-prebuilt entry：" in refused.stdout + refused.stderr,
                   f"被判的树里 验证包依赖 rocksdb 而进程环境里没有预编库的取值，没在起 cargo 之前拒（退 {refused.returncode}，"
                   f"假的一步起了 {len(selftest.calls()) - calls_before} 次）：{(refused.stdout + refused.stderr)[-600:]}",
                   "看 main 里 check_real_run_is_allowed 之后的 check_rocksdb_entry（scripts/hosts/rocksdb-prebuilt.py entry）")


def selftest_multi_host_configuration_cells(selftest):
    """多机配置交给测试（文件头同名一段）：各串起过的每一步里，种类是 test 的（名字里不带 build 的）都收到那一串自证那一份多机配置的绝对路径、编译步没收到；
    多机配置不在 ⇒ 真跑起 cargo 之前拒（退 2）、说是多机配置、假的一步一次都没起。弄坏开关 test-steps-configuration-unpassed 下两格判错。"""
    records = []   # [(那一串自己的多机配置, (序号, 步名, 收到的路径或 -))]：各串的假的一步各记各的（Selftest.configuration_logs）
    for path, configuration in selftest.configuration_logs:
        if os.path.exists(path):
            with open(path) as handle:
                records += [(configuration, tuple(line.rstrip("\n").split("\t"))) for line in handle if line.strip()]
    test_records = [(configuration, record) for configuration, record in records if len(record) == 3 and "build" not in record[1]]
    build_records = [record for _configuration, record in records if len(record) == 3 and "build" in record[1]]
    wrong = sorted({f"{record[1]}={record[2]}（要 {configuration}）" for configuration, record in test_records if record[2] != configuration})
    selftest.check(test_records and not wrong and all(record[2] == "-" for record in build_records),
                   f"真跑测试的步没收到多机配置（{len(selftest.configuration_logs)} 份记录里测试步 {len(test_records)} 次，收到的不是那一串自己那一份多机配置的：{wrong[:5]}；"
                   f"编译步收到的：{sorted({record[2] for record in build_records})[:3]}）",
                   "看 Runner.run_step 里设 SINGLEFS_MULTI_HOST_CONFIG 那一段与 main 里的 check_multi_host_configuration")
    calls_before = len(selftest.calls())
    absent = os.path.join(selftest.work, "multi-host-absent.env")
    refused = selftest.run("multi-host-absent", SINGLEFS_MULTI_HOST_CONFIG=absent)
    selftest.check(refused.returncode == EXIT_USAGE and len(selftest.calls()) == calls_before and f"多机配置 {absent} 不在" in refused.stdout + refused.stderr,
                   f"多机配置不在时没在起 cargo 之前拒（退 {refused.returncode}，假的一步起了 {len(selftest.calls()) - calls_before} 次）：{(refused.stdout + refused.stderr)[-600:]}",
                   "看 main 里 check_rocksdb_entry 之后的 check_multi_host_configuration")


def selftest_cell_functions():
    """自证的各组格（函数体里的表）。"""
    return (
        selftest_fresh_build_cells,
        selftest_service_build_directory_cells,
        selftest_run_directory_cells,
        selftest_prune_cells,
        selftest_reuse_cells,
        selftest_global_cells,
        selftest_fixture_reader_cells,
        selftest_key_speed_cells,
        selftest_gpu_cells,
        selftest_gpu_build_first_cells,
        selftest_refused_cells,
        selftest_basic_cells,
        selftest_state_cells,
        selftest_table_cells,
        selftest_environment_cells,
        selftest_sync_cells,
        selftest_baseline_threads_cells,
        selftest_baseline_key_cells,
        selftest_baseline_entry_cells,
        selftest_service_cells,
        selftest_shared_library_cells,
        selftest_registered_cells,
        selftest_test_variable_cells,
        selftest_progress_cells,
        selftest_stop_cells,
        selftest_rocksdb_entry_cells,
        selftest_multi_host_configuration_cells,
    )


def selftest_cells_run_together():
    """要在同一串里照次序跑、不能拆开的几组（文件头「--selftest 的并行」）：后两组接着用 selftest_basic_cells 判出的库「first」（第 1 条第一段抓到、第 7 条第二段抓到这些现场）。"""
    return ((selftest_basic_cells, selftest_state_cells, selftest_test_variable_cells),)


def selftest_cells_after_chains():
    """等各串都收回之后才在本进程里跑的组：多机配置那一组判的是各串起过的每一步，要读各串记下的记录，不能与各串同时跑。"""
    return (selftest_multi_host_configuration_cells,)


def selftest_cell_chains(only):
    """自证的各组分成几串（文件头「--selftest 的并行」）→ [组的元组]：照 selftest_cell_functions 的次序，一串排在它头一组的位置；
    selftest_cells_run_together 里的几组同一串，selftest_cells_after_chains 里的不进任何一串，别的组各自一串。only 不是 None 时只留点名的组，留不下组的串不要。"""
    after, together = selftest_cells_after_chains(), selftest_cells_run_together()
    chains, placed = [], set()
    for cell in selftest_cell_functions():
        if cell in after or cell in placed:
            continue
        members = next((group for group in together if cell in group), (cell,))
        placed.update(members)
        kept = tuple(member for member in members if only is None or member.__name__ in only)
        if kept:
            chains.append(kept)
    return chains


def selftest_chain_name(chain):
    return "、".join(cell.__name__ for cell in chain)


def selftest_chain_jobs():
    """同时跑几串：每台默认重活线程数（default_heavy_threads，设了 SINGLEFS_THREAD_CAP 照它）；取不到打一行说明、一串一串跑。不按内存上限收（文件头「--selftest 的并行」）。"""
    try:
        return max(1, int(default_heavy_threads()))
    except (HeavyThreadDefaultError, ValueError) as error:
        print(f"  · 取不到每台默认重活线程数（{error}），自证的各串一串一串跑", file=sys.stderr)
        return 1


def selftest_chain_child(chain, chain_work, output_prefix):
    """fork 出来的子进程：标准输出、标准错误各落进 <output_prefix>.stdout、.stderr（父进程收齐之后按串的登记次序回放），用自己的 Selftest（临时目录 chain_work）
    照次序走 chain 里的组（run_selftest_cells 点名这几组），格数、判错数、抛出的异常与多机配置记录存进 <output_prefix>.json（写完改名换上）就 os._exit；什么都不往外抛。"""
    import traceback
    status = 1
    try:
        for descriptor, suffix in ((1, ".stdout"), (2, ".stderr")):
            handle = os.open(output_prefix + suffix, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
            os.dup2(handle, descriptor)
            os.close(handle)
        signal.signal(signal.SIGTERM, signal.SIG_DFL)   # 父进程为收尾装的处理器不带进格里
        os.makedirs(chain_work)
        selftest = Selftest(chain_work)
        error = None
        try:
            run_selftest_cells(selftest, [cell.__name__ for cell in chain])
        except BaseException:
            error = traceback.format_exc()[-1500:]
        finally:
            open(os.path.join(chain_work, "release"), "w").close()   # 挂住的那一步（有的话）见到它就退
            time.sleep(0.2)
        with open(output_prefix + ".json.tmp", "w", encoding="utf-8") as handle:
            json.dump({"cells": selftest.cells, "failures": selftest.failures, "error": error, "configuration_logs": selftest.configuration_logs}, handle)
        os.replace(output_prefix + ".json.tmp", output_prefix + ".json")
        status = 0
    finally:
        try:
            sys.stdout.flush()
            sys.stderr.flush()
        except Exception:   # 收尾时标准输出写不进去也照样退出子进程，不让异常把它带回父进程的派活循环里
            pass
        os._exit(status)


def selftest_run_chains(chains, work, jobs):
    """每串一个 fork 出来的子进程（selftest_chain_child），至多 jobs 个同时跑；→ {串的下标: 结果 dict}，没留下结果的串不在里面（selftest_collect_chains 核）。
    每串收回时往 stderr 打一行用时、CPU 秒数、峰值内存（ru_maxrss，含它等过的孙进程）。本进程被 TERM 时把还在跑的子进程逐个停掉、收掉（只对自己 fork 的进程号发）。"""
    pending, running, results = list(range(len(chains))), {}, {}
    previous_handler = signal.signal(signal.SIGTERM, lambda _number, _frame: sys.exit(EXIT_STOPPED))
    try:
        while pending or running:
            while pending and len(running) < jobs:
                index = pending.pop(0)
                prefix = os.path.join(work, "chain-" + chains[index][0].__name__)
                sys.stdout.flush()
                sys.stderr.flush()
                pid = os.fork()
                if pid == 0:
                    selftest_chain_child(chains[index], prefix, prefix + "-output")   # 不返回
                running[pid] = (index, prefix, time.monotonic())
            pid, status, usage = os.wait4(-1, 0)
            if pid not in running:
                continue
            index, prefix, started = running.pop(pid)
            try:
                with open(prefix + "-output.json", encoding="utf-8") as handle:
                    results[index] = json.load(handle)
            except (OSError, ValueError):
                pass
            print(f"  · 自证串 {selftest_chain_name(chains[index])}：{time.monotonic() - started:.0f} 秒（退出状态 {status}），"
                  f"CPU {usage.ru_utime + usage.ru_stime:.0f} 秒、峰值内存 {usage.ru_maxrss // 1024} MiB", file=sys.stderr)
    finally:
        for pid in list(running):
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:   # 已经退了、还没收：照样往下收
                pass
            try:
                os.waitpid(pid, 0)
            except ChildProcessError:
                pass
        signal.signal(signal.SIGTERM, previous_handler)
    return results


def selftest_collect_chains(selftest, chains, results, work):
    """各串的结果按串的登记次序并进 selftest（本进程那一份）：先回放那一串的标准输出与标准错误，再加格数与判错数、接上它的多机配置记录；
    串里抛了异常的记一格判错。派出去几串就要收回几串：没收回的逐个点名、记一格判错。
    弄坏开关 selftest-chain-result-dropped（本进程认）：照登记次序排在最后的那一串的结果丢掉。"""
    if raw_break("selftest-chain-result-dropped") and chains:
        results = {index: result for index, result in results.items() if index != len(chains) - 1}
    for index, chain in enumerate(chains):
        prefix = os.path.join(work, "chain-" + chain[0].__name__) + "-output"
        for suffix, stream in ((".stdout", sys.stdout), (".stderr", sys.stderr)):
            if os.path.exists(prefix + suffix):
                with open(prefix + suffix, encoding="utf-8", errors="replace") as handle:
                    stream.write(handle.read())
                stream.flush()
        result = results.get(index)
        if result is None:
            continue
        selftest.cells += result["cells"]
        selftest.failures += result["failures"]
        selftest.configuration_logs += [tuple(entry) for entry in result["configuration_logs"]]
        if result["error"] is not None:
            selftest.check(False, f"串 {selftest_chain_name(chain)} 跑到一半抛了异常：{result['error']}",
                           "照异常的尾巴改那一组格；单跑那几组：BROAD_MUTATION_SELFTEST_CELLS=<组名,…> 照原命令再交")
    missing = [selftest_chain_name(chain) for index, chain in enumerate(chains) if index not in results]
    selftest.check(not missing, f"派出去的串与收回的结果对不上：派出去 {len(chains)} 串、收回 {len(chains) - len(missing)} 串；没收回的：{'；'.join(missing)}",
                   "看那几串回放出来的标准错误与它们那一行的退出状态（被 KILL、撞了内存上限这一类）；selftest_run_chains 收、selftest_chain_child 写结果")


def run_selftest(only=None):
    """整份自证：→ (退出码, 并行几路)。"""
    load_selftest_fixtures()   # 各串是从这个进程 fork 出去的，夹具在这里载入一次，每串都带着
    known_cells = [cell.__name__ for cell in selftest_cell_functions()]
    unknown = [name for name in (only or []) if name not in known_cells]
    if unknown:
        refuse(f"BROAD_MUTATION_SELFTEST_CELLS 点了不认得的组：{'、'.join(unknown)}", f"只认这几组：{'、'.join(known_cells)}")
    service_problem = prune_server_client().selftest_service_problem()   # 判定库只在临时 prune-server 上写（客户端文件头「命令行」--with-temporary-service）
    if service_problem:
        refuse(f"--selftest 不在临时 prune-server 底下：{service_problem}", prune_server_client().SELFTEST_SERVICE_REMEDY)
    # 自证不用服务给这一次尝试的编译目录、不用调用方给的卡串：编译目录（CARGO_TARGET_DIR）与卡串从本进程的环境里去掉
    #   （Selftest 照本进程的环境给每一趟驱动；先编后跑与服务给的编译目录那两组格自己设假的）
    for name in ("CARGO_TARGET_DIR", "GPU_CARDS_BY_PRIORITY"):
        os.environ.pop(name, None)
    work = tempfile.mkdtemp(prefix="broad-mutation-selftest.")
    chains = selftest_cell_chains(only)
    jobs = selftest_chain_jobs()
    parallel_ways = min(jobs, len(chains)) if chains else 1
    print(f"  · 自证并行：{len(chains)} 串，每串一个子进程，至多 {jobs} 串同时跑", file=sys.stderr)
    after_work = os.path.join(work, "after-chains")
    try:
        results = selftest_run_chains(chains, work, jobs)
        os.makedirs(after_work)
        selftest = Selftest(after_work)
        selftest_collect_chains(selftest, chains, results, work)
        for cell in selftest_cells_after_chains():
            if only is None or cell.__name__ in only:
                cell(selftest)
    finally:
        if os.path.isdir(after_work):
            open(os.path.join(after_work, "release"), "w").close()   # 挂住的那一步（有的话）见到它就退
            time.sleep(0.2)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ --selftest：{selftest.cells} 格里 {selftest.failures} 格判错", file=sys.stderr)
        print(f"     → 怎么办：照上面每一格给的出路改，再跑 python3 {SELF_PATH} --selftest", file=sys.stderr)
        return 1, parallel_ways
    say("  ✓ --selftest：临时库目录拒、生成清单与列项、项写错拒、点名与两段、杀掉再起只补没做完的、各种结局与汇总六栏、第二段只补第一段没跑到的、--from、"
        "上一趟被 KILL 留下的运行目录开跑先删、点名跳过沿用得了的条与点名现判（自证变量）、定论不被降级、认不出的选项退 2、清单按单元存的表改坏判红、认不出的退出码、"
        "重起验编译与连着无效或秒红、恢复原件半途被杀之后下一趟照常判、工具链版本不进键、"
        "不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判它里面的条（同文件在某提交之后改过别处照样沿用）、点名现判（自证变量）判成没红、"
        "多登记一条 / 只改 Cargo.toml 注释照样沿用、改 RUSTFLAGS 全部现判、改编译档与线程数照样沿用、全局那几行不带驱动与算键库的指纹、拒算的条每趟现判、"
        "mutants.toml 写 exclude_re 之后清单少那一条且整版重记而别的条沿用、换了 --threads 基线照样沿用、"
        "基线键按测试的单元闭包取（改不在测试闭包里的函数测试步基线照沿用、改测试就重跑、编译步拒算照跑）、"
        "基线键拒算的测试步按条记基线（收得全的测试各自沿用、只照跑拒算与改过的那几条，验证包拒算的那几条另起 --exact）、"
        "判定库只在 prune-server 里（按写入次序读回、库目录里不建判定库、连不上退 75）、每一趟自己的编译目录（不照用上一趟的产物）、跑完运行目录里不剩编译目录与树副本（正常、出错、被 TERM 停三种结局都删，有进程开着就不删、报出来、退 9）、两趟写进同一份判定库、"
        "登记的崩溃枚举用例与放量分项（测试步命令接 --skip、对上的静态测试进排除单、标了 #[ignore] 的不算真跑得到的测试）、只供自证的变量不出自证、一步跑过进度间隔要报进度、被 TERM 连进程组一起停、"
        "GPU 那一趟（只有 CPU 红 / 只有 GPU 红 / 两趟都没红 / 没卡 / 中途没卡 / 基线红 / 认不出的退出码与无效与一个测试都没跑到、命令的写法、复用与点名现判（自证变量）、"
        "第二段没跑时不起 GPU 趟、复用三问与复用哨兵带上 GPU 趟）、"
        "GPU 趟先编后跑（逐条与基线的编译都在测试步之前，测试步各照自己编完留下的可执行文件跑，编不过的照记无效；不领卡，每一步收到的卡串是环境里给的那一串）、"
        "服务给的编译目录（接着用上一趟的产物、跑完不删），"
        f"{selftest.cells} 格都对，{len(chains)} 串")
    return 0, parallel_ways
