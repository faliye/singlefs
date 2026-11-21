"""broad-mutation.py 拆出的一份。
GPU 那一趟：GPU 结局的分类之外的部分（门控测试文件、编译参数、卡池取卡、判定行的读写、GpuPass、GPU 一趟的编译与跑）。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import json
import os
import re
import shutil
import subprocess
import sys
import time
from typing import NamedTuple
from broad_mutation_base import (BINDGEN_LIBRARY, DECISIVE_OUTCOMES, DRIVER_ERROR, EXIT_BASELINE_RED, GPU_BASELINE_SEQUENCE, GPU_BUILD_STEP,
    GPU_NOT_RUN, GPU_ROW_OUTCOMES, GPU_TESTS_DIRECTORY, GPU_TEST_STEP, GPU_VERDICTS_TABLE, GPU_VERDICT_COLUMNS, SCRIPT_KEY, SUMMARY_KEYS)
from broad_mutation_judging import (GpuJudging, GpuOutcome, ItemState, apply_mutation, break_is, cpu_state_for_gpu, gpu_classify_build,
    gpu_classify_test, gpu_combined_state, gpu_log_text, gpu_pass_command, named_items_rejudged, refuse, restore_original, say, stage_two_skipped,
    today_text, toolchain_text)
from broad_mutation_library import (in_run_scope, remove_tree, reuse_checks_library)


class GpuTotals(NamedTuple):
    needed: int        # 要这一趟的条数（CPU 那一趟没红、又有带特性的测试够得着它）
    judged: int        # 这一趟现判了几条
    carried: int       # 键相同、沿用得了的几条
    not_run: int       # 本次未跑几条（没卡、中途没卡）
    failed: bool       # 复用哨兵判红
    outcomes: dict     # 现判的各结局条数


def gpu_bindgen_arguments():
    """librocksdb-sys 的 bindgen 要的头文件目录：与崩溃放量取同一个值（scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS），
    编译产物才共用。在子 shell 里 source 那份库、只读这一个变量；读不到交空串（调用方说一句、照调用方的环境）。"""
    if not os.path.isfile(BINDGEN_LIBRARY):
        return ""
    done = subprocess.run(["bash", "-c", 'source "$1" >/dev/null 2>&1; printf "%s" "${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}"', "bindgen", BINDGEN_LIBRARY],
                          capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return done.stdout if done.returncode == 0 else ""


def record_gpu_verdict(library, fields):
    """落一行 GPU 判定（十三段）：写进判定库、等服务回确认。"""
    library.append_rows(GPU_VERDICTS_TABLE, [fields[:len(GPU_VERDICT_COLUMNS)]])


def refresh_verdict_rows(judgement):
    """判定库重读一遍，换进 Judgement（构造时读的那几行之后又落了新行）；单元树、全局那几行与已算好的键不动。"""
    judgement.table = judgement.library.read_verdicts()
    judgement.by_key, judgement.by_locator = {}, {}
    for row in judgement.table.rows:
        judgement.by_key.setdefault(row["key"], []).append(row)
        judgement.by_locator.setdefault(row["locator"], []).append(row)


def gpu_views(library, arguments, judgement, items, states, for_run):
    """每一条两趟合起来的样子：→ (GpuJudging 或 None, {序号: GpuView})。没有 CPU 那一趟没红的条就不碰 GPU 判定（不算带特性的闭包）。"""
    views, judging = {}, None
    for item in items:
        cpu = states[item["sequence"]][0]
        if cpu.state == "missed":
            if judging is None:
                judging = GpuJudging(library, judgement, arguments)
            views[item["sequence"]] = judging.view(item, cpu, for_run)
    return judging, views


def combined_states(states, views):
    """{序号: (合起来的 ItemState, 理由, 定下状态的 CPU 那一行)}：要 GPU 那一趟的条用合起来的状态（理由 gpu-…），别的照 CPU 那一趟。"""
    found = {}
    for sequence, (state, reason, deciding) in states.items():
        view = views.get(sequence)
        if view is not None and view.needed:
            found[sequence] = (view.combined, view.reason or reason, deciding if view.combined.state == state.state else None)
        else:
            found[sequence] = (state, reason, deciding)
    return found


# ── GPU 那一趟的跑：基线、逐条、本次未跑 ─────────────────────────────────────────────────────────────────────

class GpuPass:
    """GPU 那一趟的步（文件头「GPU 那一趟」）：编译步与测试步各自经 Runner.run_step 直接起（不另套清场）。
    基线 = 在没改过的树上把这一趟要跑的目标跑绿一遍（编译步、测试步各一次）；这一趟的基线不记库，每趟现跑。每一步都不设内存上限、不限时（文件头「内存与限时」）。"""

    def __init__(self, runner, judging):
        self.runner, self.judging = runner, judging
        self.arguments, self.library = runner.arguments, runner.library
        self.bindgen = gpu_bindgen_arguments()
        self.build_step = (GPU_BUILD_STEP, "gpu", "build", [], None)
        self.test_step = (GPU_TEST_STEP, "gpu", "test", [], None)

    def run(self, step, sequence, labels):
        command = gpu_pass_command("build" if step is self.build_step else "test", self.judging.features, labels)
        extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}
        if self.runner.fake_driver:
            extra["BROAD_MUTATION_STEP_COMMAND"] = json.dumps(command)
        return self.runner.run_step(step, sequence, command=command, extra=extra)

    def run_test_from_stash(self, sequence, labels, stash):
        """先编后跑的测试步（文件头「GPU 那一趟」先编后跑）：命令照 run 的测试步（gpu_pass_command 不变），环境另交两样：
        BROAD_MUTATION_GPU_STASH 指这一条编完留下可执行文件的目录；真跑时 cargo 起测试二进制的执行器（CARGO_TARGET_<host>_RUNNER）指 stash 里的执行器，
        它把 cargo 要起的那一份换成目录里同名的一份。cargo 的工作目录、环境、起的次序、Running 行与第一个二进制红就停的规矩都照 cargo test。→ (退出码, 秒数)。"""
        command = gpu_pass_command("test", self.judging.features, labels)
        extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}
        extra["BROAD_MUTATION_GPU_STASH"] = stash["directory"]
        if self.runner.fake_driver:
            extra["BROAD_MUTATION_STEP_COMMAND"] = json.dumps(command)
        else:
            extra[stash["runner_variable"]] = stash["runner"]
        return self.runner.run_step(self.test_step, sequence, command=command, extra=extra)

    def ensure_baseline_build(self, labels):
        """GPU 趟基线的编译步：→ 用了几秒。没改过的树上编译红：拒（退 6，基线不绿后面每一条抓到都不作数）。
        先编后跑时它排在逐条的编译之后：这一步编完，编译目录里是没改过的树的产物，之后跑的测试步（基线与逐条）cargo 不再编。"""
        sequence = GPU_BASELINE_SEQUENCE
        exit_code, build_seconds = self.run(self.build_step, sequence, labels)
        build_log = self.runner.log_path(sequence, GPU_BUILD_STEP)
        outcome, note_text = gpu_classify_build(exit_code, build_log)
        if outcome is not None:
            refuse(f"GPU 趟基线 {GPU_BUILD_STEP} 在没改过的树上退 {exit_code}（{note_text}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {build_log}；代码本来就编不过就先修代码（带 --features {','.join(self.judging.features)} 的 验证包），是环境（盘满、缺 libclang、BINDGEN_EXTRA_CLANG_ARGS）就修环境，重跑同一条命令",
                   EXIT_BASELINE_RED)
        os.remove(build_log)
        return build_seconds

    def ensure_baseline_test(self, labels, build_seconds):
        """GPU 趟基线的测试步（领到卡之后跑）：→ (跑得了, 为什么跑不了)。没改过的树上测试红 / 一个测试都没跑到：拒（退 6）；没找到卡：这一趟本次未跑，不拒。"""
        sequence = GPU_BASELINE_SEQUENCE
        exit_code, seconds = self.run(self.test_step, sequence, labels)
        test_log = self.runner.log_path(sequence, GPU_TEST_STEP)
        result = gpu_classify_test(exit_code, test_log)
        if result.outcome == GPU_NOT_RUN:
            return False, result.note
        if result.outcome != "missed":
            refuse(f"GPU 趟基线 {GPU_TEST_STEP} 在没改过的树上不绿（{'有测试没过：' + result.caught_by if result.outcome == 'caught' else result.note}）：基线不绿，后面每一条 GPU 趟的抓到都不作数，一条都不跑",
                   f"看日志 {test_log}；代码本来就红就先修代码，是卡的毛病（显存、别的会话占着）就等卡空了重跑同一条命令",
                   EXIT_BASELINE_RED)
        os.remove(test_log)
        say(f"  · GPU 趟基线绿：{GPU_BUILD_STEP} {build_seconds:.1f} 秒、{GPU_TEST_STEP} {seconds:.1f} 秒（目标 {','.join(labels)}）；每一步都不设内存上限、不限时")
        return True, ""


def plan_gpu_sentinel(candidates, arguments):
    """GPU 那一趟的复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：候选是这一趟罩得住、沿用判据此刻认定论的 GPU 判定，项名「gpu:<序号>」。
    点名的照样现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）时不读库不抽。弄坏开关 gpu-carry-dropped（沿用得了的照样重跑）下这一趟判红「复用失灵」。"""
    sentinel = reuse_checks_library().ReuseSentinel(SCRIPT_KEY, "广谱变异 GPU 趟")
    off = "点名的照样现判（BROAD_MUTATION_REJUDGE_NAMED）" if named_items_rejudged() else ""
    sentinel.plan({f"gpu:{sequence}": outcome for sequence, outcome in candidates.items()}, reuse=not off, off_reason=off, printer=say)
    return sentinel


def gpu_stash_prepare(library):
    """先编后跑留可执行文件的地方（文件头「GPU 那一趟」先编后跑）：编译目录底下的 broad-mutation-gpu-stash/（与 cargo 编出的可执行文件同一个盘，留的时候硬链接），
    上一趟（被 KILL、服务给的编译目录接着用）留下的先删掉，写好 cargo 起测试二进制时经的执行器 runner。→ dict(root, runner, runner_variable)。
    runner_variable 是 CARGO_TARGET_<这一台 rustc -vV 的 host，大写、- 换成 _>_RUNNER；读不出 host（假驱动的工具链行没有它）交空串。"""
    root = os.path.join(library.target, "broad-mutation-gpu-stash")
    remove_tree(root)
    os.makedirs(root)
    runner = os.path.join(root, "runner")
    with open(runner, "w", encoding="utf-8") as handle:
        handle.write("#!/bin/sh\n"
                     "# broad-mutation.py 写的执行器：GPU 那一趟先编后跑，cargo test 起测试二进制时经这一层，换成 BROAD_MUTATION_GPU_STASH 里同名的那一份（这一条编完留下的）；\n"
                     "# 没有同名的那一份打一行 BROAD_MUTATION_GPU_STASH_MISSING、退 97，驱动见到这一行记驱动错\n"
                     'executable="$1"; shift\n'
                     'stashed="${BROAD_MUTATION_GPU_STASH:-}/${executable##*/}"\n'
                     'if [ -z "${BROAD_MUTATION_GPU_STASH:-}" ] || [ ! -x "$stashed" ]; then\n'
                     '  echo "BROAD_MUTATION_GPU_STASH_MISSING executable=$executable stash=${BROAD_MUTATION_GPU_STASH:-}"\n'
                     "  exit 97\n"
                     "fi\n"
                     'exec "$stashed" "$@"\n')
    os.chmod(runner, 0o755)
    host = next((line.split(":", 1)[1].strip() for line in toolchain_text(library.tree).splitlines() if line.startswith("host:")), "")
    return {"root": root, "runner": runner, "runner_variable": f"CARGO_TARGET_{host.upper().replace('-', '_')}_RUNNER" if host else ""}


def record_gpu_item(library, judging, item, outcome, caught_by, test_key, coverage_key, test_seconds, note_text):
    """落一行这一条的 GPU 判定（record_gpu_verdict 的列：键、序号、包、名字、结局、抓到它的测试与测试键、覆盖者键、GPU 趟的键、测试秒、日期、现判、备注）。"""
    current = judging.judgement.key_of(item)
    record_gpu_verdict(library, [current["key"], item["sequence"], item["package"], item["name"], outcome, caught_by, test_key, coverage_key, judging.pass_key(),
                                 f"{test_seconds:.1f}", today_text(), "fresh", note_text])


def build_gpu_item(library, gpu_pass, item, need, stash_place):
    """先编后跑的头一半（文件头「GPU 那一趟」先编后跑）：改坏、跑编译步、拷回原件。编得过的把编译步日志里 cargo 报的每个测试可执行文件
    （「Executable <源文件> (<可执行文件>)」那几行）硬链接进 stash_place 根下的 <序号>/（链不了就拷），记下各自的大小与修改时刻。不落行：编不过的等基线测试步绿了才记。
    → dict(outcome=None 编得过 | unviable | driver-error, note, build_seconds, directory, kept={名字: (大小, 修改纳秒)})。
    真跑而日志里一个 Executable 行都没有、或报的可执行文件不在：记驱动错（认不出这一条编出了什么，跑不了它）；假驱动不编，没有就不留。树里的原件无论怎么退都拷回。"""
    runner = gpu_pass.runner
    sequence = item["sequence"]
    directory = os.path.join(stash_place["root"], str(sequence))
    built = {"outcome": None, "note": "", "build_seconds": 0.0, "directory": directory, "kept": {}}
    try:
        apply_mutation(library, item)
        exit_code, built["build_seconds"] = gpu_pass.run(gpu_pass.build_step, sequence, need["labels"])
        build_log = runner.log_path(sequence, GPU_BUILD_STEP)
        built["outcome"], built["note"] = gpu_classify_build(exit_code, build_log)
        if built["outcome"] is not None:
            return built
        os.makedirs(directory, exist_ok=True)
        absent = []
        for line in gpu_log_text(build_log).splitlines():
            found = re.match(r"^\s*Executable (?:unittests )?\S+ \(([^)]+)\)\s*$", line)
            if not found:
                continue
            built_path = os.path.join(library.tree, found.group(1))
            kept_path = os.path.join(directory, os.path.basename(built_path))
            if not os.path.isfile(built_path):
                absent.append(found.group(1))
                continue
            if os.path.lexists(kept_path):
                continue
            try:
                os.link(built_path, kept_path)
            except OSError:
                shutil.copy2(built_path, kept_path)
            status = os.stat(kept_path)
            built["kept"][os.path.basename(kept_path)] = (status.st_size, status.st_mtime_ns)
        if absent or (not built["kept"] and not runner.fake_driver):
            built["outcome"] = DRIVER_ERROR
            built["note"] = (f"{GPU_BUILD_STEP} 编得过而" + (f"日志里报的可执行文件不在（{'、'.join(absent)}）" if absent else "日志里没有 cargo 报的 Executable 行")
                             + f"：认不出这一条编出的测试可执行文件，先编后跑跑不了它（日志 {build_log}）")
            return built
        os.remove(build_log)
        return built
    finally:
        restore_original(library)


def run_gpu_item(library, gpu_pass, item, need, built, stash_place):
    """先编后跑的后一半：这一条留下的可执行文件大小与修改时刻都没变才跑（变了是后面的编译就地改写了硬链接的那一份：记驱动错，不跑），
    照 GpuPass.run_test_from_stash 跑测试步、判结局、落一行 GPU 判定（本次未跑的不落行）；日志里有执行器报的 BROAD_MUTATION_GPU_STASH_MISSING
    （cargo 要起的可执行文件这一条没留下）记驱动错。→ (GpuOutcome, 测试秒)。被 TERM 停（Stopped）不落行。"""
    runner, judging = gpu_pass.runner, gpu_pass.judging
    sequence = item["sequence"]
    changed = []
    for name, (size, modified) in sorted(built["kept"].items()):
        try:
            status = os.stat(os.path.join(built["directory"], name))
        except OSError:
            changed.append(name)
            continue
        if (status.st_size, status.st_mtime_ns) != (size, modified):
            changed.append(name)
    if changed:
        note_text = f"先编后跑留下的可执行文件 {'、'.join(changed)} 在跑之前变了（后面的编译就地改写了它，或它被删了）：不是这一条的结局"
        record_gpu_item(library, judging, item, DRIVER_ERROR, "", "", "", 0.0, note_text)
        return GpuOutcome(DRIVER_ERROR, "", note_text), 0.0
    exit_code, test_seconds = gpu_pass.run_test_from_stash(sequence, need["labels"], dict(stash_place, directory=built["directory"]))
    test_log = runner.log_path(sequence, GPU_TEST_STEP)
    if "BROAD_MUTATION_GPU_STASH_MISSING" in gpu_log_text(test_log):
        note_text = f"{GPU_TEST_STEP} 要起的测试可执行文件这一条编完没留下（执行器报 BROAD_MUTATION_GPU_STASH_MISSING，日志 {test_log}）：不是这一条的结局"
        record_gpu_item(library, judging, item, DRIVER_ERROR, "", "", "", test_seconds, note_text)
        return GpuOutcome(DRIVER_ERROR, "", note_text), test_seconds
    result = gpu_classify_test(exit_code, test_log)
    if result.outcome == GPU_NOT_RUN:
        return result, test_seconds
    if result.outcome == "missed":
        os.remove(test_log)
        record_gpu_item(library, judging, item, "missed", "", "", need["coverers"], test_seconds, result.note)
    elif result.outcome == "caught":
        test_key, why = judging.catching_key(result.caught_by)
        if test_key:
            record_gpu_item(library, judging, item, "caught", result.caught_by, test_key, "", test_seconds, result.note)
        else:
            # 抓到它的测试认不出（同 CPU 那一趟）：每趟现判，另记覆盖者键给只读的 --summary 核
            record_gpu_item(library, judging, item, "caught", result.caught_by, "", need["coverers"], test_seconds, f"catching-test-unresolved:{why}")
    else:
        record_gpu_item(library, judging, item, result.outcome, "", "", "", test_seconds, result.note)
    return result, test_seconds


def run_gpu_phase(library, runner, items, arguments):
    """CPU 那一趟全跑完之后的 GPU 那一趟（文件头「GPU 那一趟」）：→ GpuTotals。
    要这一趟的条 = 这一趟罩得住、CPU 那一趟最后落的结论是没红、又有带特性的测试够得着的；键相同、沿用得了的复用；沿用不了的现判。
    开跑先在没改过的树上跑一遍基线；没找到卡：要现判的条逐条打 BROAD_MUTATION_GPU_NOT_RUN（本次未跑，不落行、不算没红）。"""
    outcomes = dict.fromkeys(GPU_ROW_OUTCOMES, 0)

    def totals(needed=0, judged=0, carried=0, not_run=0, failed=False, report=True):
        if report:
            say(f"BROAD_MUTATION_GPU_TOTAL needed={needed} judged={judged} carried={carried} not_run={not_run} "
                + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in outcomes.items()))
        return GpuTotals(needed, judged, carried, not_run, failed, outcomes)

    if stage_two_skipped():
        say("  · GPU 趟接在第二段后面，这一趟的第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），不跑")
        return totals(report=False)
    judgement = runner.judgement
    refresh_verdict_rows(judgement)      # CPU 那一趟刚落的行要读进来（单元树、每条的键、测试的闭包照旧用这一趟已经算好的：树里的源码这一趟没变）
    judging = GpuJudging(library, judgement, arguments)
    if not judging.enabled:
        say(f"  · GPU 趟：{GPU_TESTS_DIRECTORY} 下没有文件的 cfg 点到 gpu 特性，没有带 gpu 特性的测试可跑")
        return totals(report=False)
    pending, carried, refused, candidates = [], [], 0, {}
    for item in items:
        if not in_run_scope(item, arguments):
            continue
        cpu = cpu_state_for_gpu(judgement, item)[0]
        if cpu.state != "missed" and not (break_is("gpu-also-when-cpu-caught") and cpu.state == "caught"):
            continue
        current = judgement.key_of(item)
        if current["key"] is None:
            continue
        need = judging.need(current)
        if not need["needed"]:
            continue
        gpu, reason, _deciding = judging.state(current, need, for_run=True)
        if current["refused"]:
            refused += 1
        if gpu.state in DECISIVE_OUTCOMES:
            candidates[item["sequence"]] = gpu.state     # 沿用判据此刻认定论的：复用哨兵的候选（弄坏开关 gpu-carry-dropped 下它们被现判，哨兵就判红）
        if gpu.state in DECISIVE_OUTCOMES and not named_items_rejudged() and not break_is("gpu-carry-dropped"):
            carried.append((item, gpu, need))
        else:
            pending.append((item, need, reason or "no-record"))
    needed = len(pending) + len(carried)
    say(f"BROAD_MUTATION_GPU_PLAN needed={needed} judge={len(pending)} carried={len(carried)} refused={refused} features={','.join(judging.features)}")
    if not needed:
        return totals()
    if getattr(arguments, "shard_phase", "") == "merge":
        # 合并那一步核账（文件头「分片」merge）：GPU 那一趟没有结论的条在各片里判，这一步不跑、记本次未跑
        for item, _need, _reason in pending:
            say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason=not-judged-in-shards name={item['name']}")
        return totals(needed=needed, carried=len(carried), not_run=len(pending))
    sentinel = plan_gpu_sentinel(candidates, arguments)
    forced = next((item for item, _gpu, need in carried if sentinel.forced(f"gpu:{item['sequence']}")), None)
    if forced is not None:
        moved = next(entry for entry in carried if entry[0] is forced)
        carried.remove(moved)
        pending.append((forced, moved[2], "sentinel-forced"))
    for item, need, reason in pending:
        say(f"BROAD_MUTATION_GPU_JUDGE sequence={item['sequence']} reason={reason} labels={','.join(need['labels'])}")
    if not pending:
        return totals(needed=needed, carried=len(carried))
    gpu_pass = GpuPass(runner, judging)
    labels = sorted({label for _item, need, _reason in pending for label in need["labels"]}, key=lambda text: (text != "lib", text))
    # 先编后跑（文件头「GPU 那一趟」）：逐条的编译步、随后基线的编译步都编完，才只跑测试步（基线与逐条，cargo 不再编）。不领卡（docstring「不领卡」）：
    #   环境里的 GPU_CARDS_BY_PRIORITY 原样交给每一步。弄坏开关 gpu-builds-interleaved：基线先编先跑，每一条在跑它的测试步之前才编
    builds_interleaved = break_is("gpu-builds-interleaved")
    stash_place = gpu_stash_prepare(library)
    judged_outcomes, not_run_names, stopped_for = {}, [], ""
    try:
        if not runner.fake_driver and not stash_place["runner_variable"]:
            refuse(f"GPU 那一趟先编后跑要 cargo 起测试二进制时经执行器（CARGO_TARGET_<host>_RUNNER），而在 {library.tree} 里 rustc -Vv 读不出 host 那一行，定不了变量名",
                   f"在 {library.tree} 里手跑 rustc -Vv 看输出里有没有「host: 」那一行")
        built = {}

        def build_item(item, need):
            item_built = built[item["sequence"]] = build_gpu_item(library, gpu_pass, item, need, stash_place)
            say(f"  · GPU 趟编完第 {item['sequence']} 条（{item_built['build_seconds']:.1f} 秒）："
                + (f"编得过，留下 {len(item_built['kept'])} 个测试可执行文件，都编完之后只跑" if item_built["outcome"] is None
                   else f"{item_built['outcome']}（{item_built['note']}），基线测试步绿了就记、不跑测试步"))
        for item, need, _reason in [] if builds_interleaved else pending:
            build_item(item, need)
        baseline_build_seconds = gpu_pass.ensure_baseline_build(labels)
        available, why = gpu_pass.ensure_baseline_test(labels, baseline_build_seconds)
        if not available:
            for item, _need, _reason in pending:
                say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={why.split('：')[0]} name={item['name']}")
            say(f"  · GPU 趟本次未跑 {len(pending)} 条（{why}）：它们没有 GPU 那一趟的结论，不算没红，也不算通过；有卡之后重跑同一条命令")
            return totals(needed=needed, carried=len(carried), not_run=len(pending))
        for item, need, _reason in pending:
            if builds_interleaved:
                build_item(item, need)
            item_built = built[item["sequence"]]
            started = time.monotonic()
            if item_built["outcome"] is not None:
                # 编不过（无效）与编完认不出可执行文件（驱动错）：基线测试步绿了才记，不跑测试步
                record_gpu_item(library, judging, item, item_built["outcome"], "", "", "", 0.0, item_built["note"])
                result, test_seconds = GpuOutcome(item_built["outcome"], "", item_built["note"]), 0.0
            elif stopped_for:
                not_run_names.append(item)
                say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={stopped_for} name={item['name']}")
                continue
            else:
                result, test_seconds = run_gpu_item(library, gpu_pass, item, need, item_built, stash_place)
                if result.outcome == GPU_NOT_RUN:
                    stopped_for = result.note.split("：")[0]
                    not_run_names.append(item)
                    say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={stopped_for} name={item['name']}")
                    continue
            outcomes[result.outcome] = outcomes.get(result.outcome, 0) + 1
            judged_outcomes[f"gpu:{item['sequence']}"] = result.outcome
            cpu = cpu_state_for_gpu(judgement, item)[0]
            combined, _reason = gpu_combined_state(cpu, True, ItemState(result.outcome, False))
            say(f"BROAD_MUTATION_GPU_ITEM sequence={item['sequence']} cpu={cpu.state} gpu={result.outcome} combined={combined.state} build_seconds={item_built['build_seconds']:.1f} "
                f"test_seconds={test_seconds:.1f} wall_seconds={item_built['build_seconds'] + time.monotonic() - started:.1f} judged=1 "
                f"red={1 if result.outcome == 'missed' else 0} reused=0 name={item['name']}"
                + (f" first_failing_test={result.caught_by}" if result.caught_by else ""))
    finally:
        # 这一趟的条都跑完（含拒绝、被停）就删留下的可执行文件
        remove_tree(stash_place["root"])
    failed = False
    skipped = {f"gpu:{item['sequence']}" for item in not_run_names}      # 没跑成的（卡中途没了）：不当复用也不当现判，哨兵不比
    for key, stored in sentinel.candidates.items():
        if key in skipped:
            continue
        if key not in judged_outcomes:
            sentinel.observe(key, reuse_checks_library().REUSED, stored)
        elif judged_outcomes[key] in DECISIVE_OUTCOMES:
            sentinel.observe(key, reuse_checks_library().FORCED if sentinel.forced(key) else reuse_checks_library().JUDGED, judged_outcomes[key])
    for message, howto in sentinel.finish(printer=say):
        print(f"  ✗ {message}", file=sys.stderr)
        print(f"  → 怎么办：{howto}", file=sys.stderr)
        failed = True
    return totals(needed=needed, judged=sum(outcomes.values()), carried=len(carried), not_run=len(not_run_names), failed=failed)
