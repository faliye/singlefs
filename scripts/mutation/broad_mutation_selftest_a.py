"""broad-mutation.py 拆出的一份。
自证（一）：Selftest 类与基础、状态、表、环境、同步、剪枝、复用、算键提速几组格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import argparse
import difflib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from broad_mutation_base import (BASELINE_SEQUENCE, DRIVER_ERROR, ENVIRONMENT_SEQUENCE, EXIT_BASELINE_RED, EXIT_GATE_RED, EXIT_INCONSISTENT,
    EXIT_LOCKED, EXIT_RUN_DIRECTORY_KEPT, EXIT_STOPPED, EXIT_USAGE, GPU_BASELINE_SEQUENCE, GPU_TEST_STEP,
    MODULE_SOURCE_PATHS, MUTATION_MARKER, NOT_RUN, REGISTERED_CHILD, REGISTERED_REASON, SCRIPT_KEY, SELFTEST_STEP_DRIVER_NAME,
    SELFTEST_WORK_VARIABLE, SELF_PATH, STAGE_ONE_ONLY, TEST_ONLY_VARIABLES, VERDICTS_TABLE, VERDICT_COLUMNS, VERIFICATION_PACKAGE)
from broad_mutation_judging import (ItemState, Judgement, prune_rules, read_gpu_verdicts, replace_span, write_atomically)
from broad_mutation_library import (Library, files_in_tree, prune_server_client, reuse_checks_library)
from broad_mutation_runner import (step_table)
from broad_mutation_run import (changed_units_of, key_identity, memorise_compositions, prefill_keys, read_key_cache, shard_piles, speed_up_judgement,
    write_key_cache)
import broad_mutation_judging


def selftest_mutants(sources=None):
    """备一份假的原始清单：每个文件里每个返回值各一条，把那个数字换成 MUTATED_<序号>()。"""
    mutants, sequence = [], 0
    for package, path in (("singlefs-core", "crates/singlefs-core/src/lib.rs"), ("singlefs-checker", "crates/singlefs-checker/src/lib.rs")):
        original = (sources or selftest_fixtures.SELFTEST_SOURCES)[path]
        for line_index, line in enumerate(original.split("\n")):
            if not line.strip().isdigit():
                continue
            sequence += 1
            start, end = (line_index + 1, 5), (line_index + 1, len(line) + 1)
            replacement = f"MUTATED_{sequence}()"
            mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
            diff = "".join(difflib.unified_diff(original.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{path}", f"b/{path}"))
            mutants.append({"package": package, "file": path, "name": f"{path}:{line_index + 1}:5: replace the value with {replacement}",
                            "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": end[0], "column": end[1]}},
                            "replacement": replacement, "diff": diff})
    return mutants


def process_is_running(process_id):
    """进程还在跑（不算已经退出、等着被收的）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
    except (OSError, IndexError):
        return False


def write_sources(directory, sources):
    for path, text in sources.items():
        os.makedirs(os.path.dirname(os.path.join(directory, path)), exist_ok=True)
        with open(os.path.join(directory, path), "w") as handle:
            handle.write(text)


class Selftest:
    def __init__(self, work):
        self.work = work
        self.cells = 0
        self.failures = 0
        self.driver_failure_lines = []   # 上一格判完以来 run() 起的驱动在完整输出里报的 ✗ 行，check() 判红时整行接在失败行末尾
        self.source = os.path.join(work, "source")
        write_sources(self.source, selftest_fixtures.SELFTEST_SOURCES)
        self.mutants = selftest_mutants()
        self.write_mutants(self.mutants)
        for name, text in ((SELFTEST_STEP_DRIVER_NAME, selftest_fixtures.SELFTEST_STEP_DRIVER), ("list-driver.py", selftest_fixtures.SELFTEST_LIST_DRIVER)):
            with open(os.path.join(work, name), "w") as handle:
                handle.write(text)
            os.chmod(os.path.join(work, name), 0o755)
        self.write_plan(selftest_fixtures.SELFTEST_PLAN)
        # 真跑测试的步交给测试的多机配置（文件头「多机配置交给测试」）：调用方设的不带进来，换成临时目录里的一份；多机配置那几格自己改
        self.multi_host_configuration = os.path.join(work, "multi-host.env")
        with open(self.multi_host_configuration, "w") as handle:
            handle.write("# 自证用的多机配置：假的一步不读它，多机配置那几格只核它的路径到没到测试步的环境里\nENABLE_GPU=0\n")
        # 假的一步记下每一步收到的多机配置（<序号>\t<步名>\t<收到的路径或 ->）：[(记录文件, 那一份 Selftest 自己的多机配置)]；
        #   各串收回之后，本进程那一份接上各串的（selftest_collect_chains），多机配置那一组照全部的判
        self.configuration_logs = [(os.path.join(work, "multi-host-configuration.log"), self.multi_host_configuration)]
        self.environment = dict(os.environ, BROAD_MUTATION_SELFTEST_WORK=work, BROAD_MUTATION_STEP_DRIVER="1", SINGLEFS_MULTI_HOST_CONFIG=self.multi_host_configuration,
                                BROAD_MUTATION_LIST_DRIVER=os.path.join(work, "list-driver.py"), BROAD_MUTATION_EXPORT_EVERYTHING="1",
                                BROAD_MUTATION_ALLOW_VOLATILE_HOME="1", BROAD_MUTATION_REGISTERED="")
        # 复用哨兵：别的格比的是现判与沿用的条数，沿用得了的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.environment[reuse_checks_library().SENTINEL_COUNT_VARIABLE] = "1000"
        self.environment.pop(reuse_checks_library().SENTINEL_SEED_VARIABLE, None)
        for name in ("BROAD_MUTATION_KILL_AT",
                     "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS", "RUSTUP_TOOLCHAIN"):
            self.environment.pop(name, None)

    def write_plan(self, plan):
        with open(os.path.join(self.work, "plan.tsv"), "w") as handle:
            handle.write("".join("\t".join(row) + "\n" for row in plan))

    def write_mutants(self, mutants):
        with open(os.path.join(self.work, "mutants.json"), "w") as handle:
            json.dump(mutants, handle)


    def command(self, home, *arguments, source=None):
        return [sys.executable, SELF_PATH, "--home", os.path.join(self.work, home), "--source", source or self.source, "--lock-wait", "30",
                "--stop-grace", "2", *arguments]

    def run(self, home, *arguments, environment=None, source=None, **extra_environment):
        environment = dict(environment or self.environment, **extra_environment)
        done = subprocess.run(self.command(home, *arguments, source=source), capture_output=True, text=True, stdin=subprocess.DEVNULL, env=environment)
        self.driver_failure_lines += [line.strip() for line in f"{done.stdout}\n{done.stderr}".replace("\r", "\n").split("\n") if "✗" in line]
        return done

    def calls(self):
        """调用记录：[(序号, 步名, mutated / pristine)]。"""
        path = os.path.join(self.work, "calls.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def called_sequences(self):
        """被调过的序号（基线是 0）；在没改过的树上验环境的那几步不算在里面，编译步的基线（基线键拒算、每趟照跑，文件头「基线」）也不算。"""
        return {call[0] for call in self.calls() if call[0] != ENVIRONMENT_SEQUENCE and not call[1].startswith("gpu-")
                and not (call[0] == str(BASELINE_SEQUENCE) and call[1] in selftest_fixtures.SELFTEST_BUILD_STEPS)}

    def judged_sequences(self):
        """被调过的变异序号（不算基线 0 与验环境；GPU 那一趟的步另在 gpu_calls 里）。"""
        return self.called_sequences() - {str(BASELINE_SEQUENCE)}

    def environment_steps(self):
        return [call[1] for call in self.calls() if call[0] == ENVIRONMENT_SEQUENCE]

    def gpu_calls(self):
        """GPU 那一趟的假步记的 [(序号, 步名, mutated / pristine)]（序号 gpu-baseline 是在没改过的树上的基线）。"""
        return [call for call in self.calls() if call[1].startswith("gpu-")]

    def gpu_judged_sequences(self):
        """GPU 那一趟起过测试步的变异序号（不算 gpu-baseline）。先编后跑：要现判的条在测试步之前逐条都起编译步，编译步不算；编不过的条不起测试步，也不算。"""
        return {call[0] for call in self.gpu_calls() if call[0] != GPU_BASELINE_SEQUENCE and call[1] == GPU_TEST_STEP}

    def gpu_environment(self):
        """GPU 那一趟的假步收到的 [(序号, 步名, BINDGEN_EXTRA_CLANG_ARGS, 命令)]。"""
        path = os.path.join(self.work, "gpu-env.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def gpu_rows(self, home):
        """这份库的 GPU 判定表里成形的行（{列名: 值}）。"""
        return read_gpu_verdicts(self.library(home)).rows

    def clear_calls(self):
        for name in ("calls.log", "selections.log", "coverage-known.log", "gpu-env.log"):
            path = os.path.join(self.work, name)
            if os.path.exists(path):
                os.remove(path)

    def library(self, home):
        """自证工作目录下 <home> 那份库在 prune-server 里的判定库（自证的库目录都在临时目录底下，各占一份命名空间；文件头「判定库」）。"""
        return Library(os.path.join(self.work, home))

    def append_rows(self, home, table, rows):
        """往这份库的一张表里手写几行（造现场用；同驱动写行一样经客户端）。"""
        self.library(home).append_rows(table, rows)

    @staticmethod
    def copy_library(source_home, destination_home):
        """一份库在服务里的全部条目（各张表、条目名照旧）拷进另一份库的命名空间（两个都是库目录的绝对路径）：交拷了几条。"""
        entries = Library(source_home).scan("")
        if entries:
            Library(destination_home).store.write_many(entries)
        return len(entries)

    def rows(self, home):
        """这份库的判定表里成形的行（{列名: 值}），按写入的次序。"""
        return self.library(home).read_verdicts().rows

    def rows_of(self, home, sequence):
        return [row for row in self.rows(home) if row["sequence"] == str(sequence)]

    def listed(self, home, source=None):
        """--list-items 报的各条：{序号: {state, stale, key, name}}。"""
        listed = self.run(home, "--list-items", source=source)
        found = {}
        for fields in (line.split("\t") for line in listed.stdout.splitlines()):
            if len(fields) == 6:
                found[fields[0]] = {"name": fields[2], "state": fields[3], "stale": fields[4], "key": fields[5].removeprefix("key=")}
        return found

    def states(self, home, source=None):
        return {sequence: entry["state"] for sequence, entry in self.listed(home, source).items()}

    def build_leftovers(self, home):
        """跑完之后库目录里还剩的编译目录与树副本（相对库目录）：runs/ 下的每一个运行目录，与库目录里直接放着的那几样（tree、target、target-coverage、scratch；弄坏开关 build-directory-in-home 下才有）。"""
        home_path = os.path.join(self.work, home)
        runs = os.path.join(home_path, "runs")
        found = [os.path.join("runs", name) for name in sorted(os.listdir(runs))] if os.path.isdir(runs) else []
        return found + [name for name in ("tree", "target", "target-coverage", "scratch", "scratch-coverage") if os.path.lexists(os.path.join(home_path, name))]

    def leftover_run_directories(self, home):
        """库目录 runs/ 下此刻的运行目录（绝对路径）。"""
        runs = os.path.join(self.work, home, "runs")
        return [os.path.join(runs, name) for name in sorted(os.listdir(runs))] if os.path.isdir(runs) else []

    def tree_is_pristine(self, tree, sources=None):
        """tree（一棵专用树的绝对路径）里的源码与自证的源码逐份相同、没有多出来的文件。"""
        if not os.path.isdir(tree):
            return False
        for path, text in (sources or selftest_fixtures.SELFTEST_SOURCES).items():
            with open(os.path.join(tree, path)) as handle:
                if handle.read() != text:
                    return False
        return files_in_tree(tree) == set(sources or selftest_fixtures.SELFTEST_SOURCES)

    def check(self, condition, wrong, howto):
        """判一格。判红时打失败行「✗ 自证：<wrong>」：wrong 里转述的驱动输出截在哪由临时路径长短定，转述里的 ✗ 换成 SELFTEST_TRANSCRIBED_FAILURE_MARK，
        不让截进来的半行另成一行失败行；上一格判完以来 run() 起的驱动在完整输出里报的 ✗ 行整行接在失败行那一行末尾（文件头开关表前那一段「转述」）。"""
        self.cells += 1
        driver_lines, self.driver_failure_lines = self.driver_failure_lines, []
        if not condition:
            self.failures += 1
            head, newline, rest = wrong.replace("✗", selftest_fixtures.SELFTEST_TRANSCRIBED_FAILURE_MARK).partition("\n")
            if driver_lines:
                # 去掉数字之后相同的几行（各趟报的「复用失灵：<项> …种子 <数>」这类）并成一行、注明同样的另几行，再取前 SELFTEST_DRIVER_FAILURE_LINES_SHOWN 种：
                #   不让前面几趟的同一句把这一格后面那一句挤出去
                kinds, repeats = [], {}
                for line in driver_lines:
                    key = re.sub(r"[0-9]+", "#", line)
                    if key in repeats:
                        repeats[key] += 1
                    else:
                        repeats[key] = 0
                        kinds.append((key, line))
                shown = [line + (f"（同样的另 {repeats[key]} 行）" if repeats[key] else "") for key, line in kinds[:selftest_fixtures.SELFTEST_DRIVER_FAILURE_LINES_SHOWN]]
                more = len(kinds) - len(shown)
                head += f"；这一格起的驱动报的失败行 {len(driver_lines)} 行：{'；'.join(shown)}{f'；另 {more} 种没列' if more else ''}"
            print(f"  ✗ 自证：{head}{newline}{rest}", file=sys.stderr)
            print(f"     → 怎么办：{howto}", file=sys.stderr)


def wait_until(condition, seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if condition():
            return True
        time.sleep(0.05)
    return condition()


def summary_field(text, key):
    found = re.search(rf"\b{key}=(\d+)", text)
    return int(found.group(1)) if found else None


def plan_field(text, key, line="BROAD_MUTATION_PLAN"):
    found = re.search(rf"^{line} .*\b{key}=(\d+)", text, re.M)
    return int(found.group(1)) if found else None


def selftest_basic_cells(selftest):
    """那几格基本的：临时库目录拒、生成清单与列项、项写错、点名与两段、杀掉再起、各种结局、汇总、--from、上一趟被 KILL 留下的运行目录开跑先删。"""
    work = selftest.work
    selftest.clear_calls()   # 前一组（拒算那一组）留下的假一步调用记录不带进来：下面三格数的是这一组自己的调用
    # 库目录落在开机会清空的目录里拒（不带只供测试的放行）
    volatile_home = f"/dev/shm/broad-mutation-selftest-{os.getpid()}"
    # 不带任何只供测试的变量：带着的话库目录在自证工作目录之外，会先被「只供自证的变量不碰别的库」那一拒拦下，验不到这一格要验的「开机会清空」
    volatile_environment = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    refused = subprocess.run([sys.executable, SELF_PATH, "--home", volatile_home, "--source", selftest.source, "--refresh-list"], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=volatile_environment)
    selftest.check(refused.returncode == EXIT_USAGE and not os.path.exists(volatile_home) and "开机会清空" in refused.stderr,
                   f"库目录落在 /dev/shm 里没拒（退 {refused.returncode}）：{refused.stderr}", "看 resolve_home：/tmp、/run、/dev/shm 与 tmpfs 都要拒")
    # 生成清单、按单元存；--list-items 逐行六段、一步都不跑
    refreshed = selftest.run("first", "--refresh-list")
    listed = selftest.run("first", "--list-items")
    listed_lines = [line.split("\t") for line in listed.stdout.splitlines()]
    selftest.check(refreshed.returncode == 0 and listed.returncode == 0 and len(listed_lines) == len(selftest.mutants)
                   and all(len(fields) == 6 and fields[3] == NOT_RUN and fields[4] == "stale=0" and re.fullmatch(r"key=[0-9a-f]{64}", fields[5]) for fields in listed_lines)
                   and [fields[0] for fields in listed_lines] == [str(index + 1) for index in range(len(selftest.mutants))] and not selftest.calls(),
                   f"--refresh-list / --list-items 不对（退 {refreshed.returncode} / {listed.returncode}，{len(listed_lines)} 行，调了 {len(selftest.calls())} 步）：{refreshed.stderr}{listed.stderr}",
                   "看 generate_list、record_list_cache、restore_list 与 list_items：清单每条一行「序号、包、变异名、状态、stale=、key=」，列项不许起任何一步")
    # --list-items 接 | head（读的一头先关了）：被 SIGPIPE 安静停掉，stderr 一个字都没有
    read_end, write_end = os.pipe()
    os.close(read_end)
    closed_pipe = subprocess.run(selftest.command("first", "--list-items"), stdout=write_end, stderr=subprocess.PIPE, text=True,
                                 stdin=subprocess.DEVNULL, env=selftest.environment)
    os.close(write_end)
    selftest.check(closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "",
                   f"--list-items 接 | head 时没照 SIGPIPE 惯例安静退（退 {closed_pipe.returncode}）：{closed_pipe.stderr.strip()[-300:]}",
                   "看 main 里 --list-items 那一支：开头要把 SIGPIPE 设回默认")
    # 只读的 --summary 不写库：库目录落在开机会清空的目录里也照读（共享门禁的 stage-selftest.sh 把样本拷进 mktemp -d 再判）；要写库的用法照拒（上面那一格）
    if os.path.isdir("/dev/shm"):
        shm_work = tempfile.mkdtemp(prefix="broad-mutation-selftest-shm.", dir="/dev/shm")
        try:
            selftest.copy_library(os.path.join(work, "first"), os.path.join(shm_work, "first"))
            shm_environment = {name: value for name, value in selftest.environment.items() if name != "BROAD_MUTATION_ALLOW_VOLATILE_HOME"}
            shm_environment[SELFTEST_WORK_VARIABLE] = shm_work
            shm_summary = subprocess.run([sys.executable, SELF_PATH, "--home", os.path.join(shm_work, "first"), "--source", selftest.source, "--summary"],
                                         capture_output=True, text=True, stdin=subprocess.DEVNULL, env=shm_environment)
            selftest.check(shm_summary.returncode in (0, EXIT_GATE_RED) and "开机会清空" not in shm_summary.stderr and "BROAD_MUTATION_GATE" in shm_summary.stdout,
                           f"库目录落在 /dev/shm 里时只读的 --summary 也拒了（退 {shm_summary.returncode}）：{shm_summary.stderr.strip()[-300:]}",
                           "看 resolve_home：只读的用法（read_only）不写库，不拒开机会清空的目录")
        finally:
            shutil.rmtree(shm_work, ignore_errors=True)
    # 项写错退 2、一步都不跑
    unknown = selftest.run("first", "--item", "no-such-mutant", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(unknown.returncode == EXIT_USAGE and not selftest.calls(), f"不认得的项没拒（退 {unknown.returncode}，调了 {len(selftest.calls())} 步）",
                   "看 select_items：清单里找不到的序号或变异名要退 2 并写怎么查")
    # 点名（一条按序号、一条按变异名）只跑第一段：只有基线与这两条被调，第二段的步一次都没调；基线在原样的树上、每条在改过的树上跑（判完拷回原件，下一条才对得上）
    name_of_seven = listed_lines[6][2] if len(listed_lines) >= 7 and len(listed_lines[6]) == 6 else "?"
    named = selftest.run("first", "--item", "1", "--item", name_of_seven, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    calls = selftest.calls()
    rows = {row["sequence"]: row for row in selftest.rows("first")}
    selftest.check(named.returncode == 0 and selftest.called_sequences() == {"0", "1", "7"} and not any(call[1].startswith("two-") for call in calls)
                   and all(call[2] == ("pristine" if call[0] == "0" else "mutated") for call in calls)
                   and set(rows) == {"1", "7"} and (rows["1"]["outcome"], rows["1"]["stage"], rows["1"]["caught_by"]) == ("caught", "one", selftest_fixtures.SELFTEST_NEXTEST_CASE)
                   and re.fullmatch(r"[0-9a-f]{64}", rows["1"]["test_key"]) and (rows["7"]["outcome"], rows["7"]["stage"]) == (STAGE_ONE_ONLY, "one"),
                   f"点名两条只跑第一段不对（退 {named.returncode}；调用 {calls}；行 {sorted((row['sequence'], row['outcome'], row['stage']) for row in rows.values())}）：{named.stderr}",
                   "看 select_items 的点名那一支、process_item 的第一段与 stage_two_skipped（BROAD_MUTATION_STAGE_ONE_ONLY）：基线在没改过的树上跑、每条在改过的树上跑、判完拷回原件；抓到的那一行带测试键")
    # 点名「只过了第一段」的那一条（第二段被打断留下的现场）：只补第二段，第一段的步不再调；第二段里 验证包那一步抓到它，带测试名
    selftest.clear_calls()
    second = selftest.run("first", "--item", name_of_seven)
    calls = selftest.calls()
    seven = [(row["outcome"], row["stage"], row["caught_by"]) for row in selftest.rows_of("first", 7)]
    selftest.check(second.returncode == 0 and selftest.called_sequences() == {"0", "7"} and all(call[1].startswith("two-") for call in calls)
                   and seven == [(STAGE_ONE_ONLY, "one", ""), ("caught", "two", selftest_fixtures.SELFTEST_STAGE_TWO_CASE)],
                   f"只补第二段不对（退 {second.returncode}；调用 {calls}；第 7 条的行 {seven}）：{second.stderr}",
                   "看 select_items：状态是 stage-one-only 的只补第二段、不跑第一段；process_item 第二段哪一步红了就记抓到并带测试名")
    # 跑到一半 KILL：第 3 条的测试挂住时杀驱动。杀之前前两条的行已经落盘；杀之后它那一趟的运行目录留着（树里有改过的文件，在途记录在运行目录里）
    selftest.clear_calls()
    selftest.write_plan([row for row in selftest_fixtures.SELFTEST_PLAN if row[0] != "3"] + [("3", "one-test", "hang")])
    driver = subprocess.Popen(selftest.command("second"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, env=selftest.environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 60)
    rows_before_stop = {row["sequence"] for row in selftest.rows("second")}
    driver.kill()   # 只点自己起的这一个进程
    driver.wait()
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and rows_before_stop == {"1", "2"},
                   f"杀驱动之前，前两条的判定行没有都落盘（落了 {sorted(rows_before_stop)}）", "看 Library.record_verdict：每条判完立刻追加并 fsync，不攒到收尾")
    killed_runs = selftest.leftover_run_directories("second")
    selftest.check(len(killed_runs) == 1 and not selftest.tree_is_pristine(os.path.join(killed_runs[0], "tree"))
                   and os.path.exists(os.path.join(killed_runs[0], "inflight.json")),
                   f"驱动被杀之后它那一趟的运行目录里没有留着改过的树与在途记录（后面几格的前提不成立；runs/ 下 {killed_runs}）",
                   "看 apply_mutation：先存原件、记在途记录，再改文件；Library.begin_run：树与在途记录都在这一趟的运行目录里")
    # 被杀的驱动留下的那一步还活着时锁还占着：不等就退 4、一步都不跑
    selftest.clear_calls()
    locked = selftest.run("second", "--lock-wait", "0")
    selftest.check(locked.returncode == EXIT_LOCKED and not selftest.calls(), f"上一趟留下的进程还占着锁时没拒（退 {locked.returncode}）",
                   "看 acquire_lock 与 run_step 的 pass_fds：锁的描述符要传给每一步")
    # 放开挂住的那一步，重跑同一条命令：沿用得了的条调用 0 次、基线不重跑；被打断的那一条重跑；开跑先删上一趟留下的运行目录，跑完这一趟的也删了
    open(os.path.join(work, "release"), "w").close()
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    resumed = selftest.run("second")
    rows = selftest.rows("second")
    called = selftest.called_sequences()
    selftest.check(resumed.returncode == 0 and not called & {"0", "1", "2"},
                   f"重起之后沿用得了的条或基线被重跑了（退 {resumed.returncode}；被调的序号 {sorted(called)}）：{resumed.stderr}",
                   "看 select_items 与 Judgement.state：判定库里键相同、沿用判据认的条一条都不选；基线有这把基线键的绿记录的不重跑")
    item_lines = [line for line in resumed.stdout.splitlines() if line.startswith("BROAD_MUTATION_ITEM ")]
    selftest.check(item_lines and all(re.search(r" judged=1 red=[01] reused=0 ", line) for line in item_lines)
                   and all((" red=1 " in line) == (" outcome=missed " in line) for line in item_lines) and any(" outcome=missed " in line for line in item_lines),
                   f"BROAD_MUTATION_ITEM 每条一行没带判了、红了、复用了多少，或 red= 与结局对不上（现判 {sorted(called - {'0'})}）：{item_lines[:3]}",
                   "看 run_items 打 BROAD_MUTATION_ITEM 那一句：每条补 judged=1 red=<没红记 1> reused=0（入口脚本那一行，文件头「输出」）")
    selftest.check("3" in called and [row["outcome"] for row in rows if row["sequence"] == "3"] == [DRIVER_ERROR] and "上一趟被打断在第 3 条" in resumed.stdout,
                   f"被打断的第 3 条没有重跑、或被当成已完成（它的行：{[(row['outcome'], row['stage']) for row in rows if row['sequence'] == '3']}）",
                   "看 Library.begin_run：上一趟留下的运行目录里有在途记录就报被打断在哪一条、删掉那个运行目录，不给被打断的那一条落行")
    selftest.check(not selftest.build_leftovers("second"),
                   f"重起跑完之后库目录里还剩上一趟被 KILL 留下的、或这一趟的运行目录（{selftest.build_leftovers('second')}）",
                   "看 Library.begin_run（开跑先删上一趟留下的）与 main 的 finally 里的 Library.end_run（跑完删这一趟的）")
    os.remove(os.path.join(work, "release"))
    try:
        os.remove(os.path.join(work, "hanging"))
    except FileNotFoundError:
        pass   # 挂住的那一步没起到（前面的格已经判红）：不在这里抛出去，后面的格照判
    # 两段式的结局：抓到（第一段、文档测试、验证包各一）、没红、无效、驱动错（第一段两条、第二段一条），逐条对
    final = {row["sequence"]: (row["outcome"], row["stage"]) for row in rows if row["outcome"] != STAGE_ONE_ONLY}
    five = [row for row in rows if row["sequence"] == "5" and row["outcome"] == "caught"]
    selftest.check(final == selftest_fixtures.SELFTEST_EXPECTED and [row["caught_by"] for row in five] == [selftest_fixtures.SELFTEST_STAGE_TWO_CASE]
                   and five and five[0]["refused"].startswith("catching-test-unresolved") and re.fullmatch(r"[0-9a-f]{64}", five[0]["coverage_key"]),
                   f"两段式的结局不对（第 5 条 {[(row['caught_by'], row['refused'], row['coverage_key'][:12]) for row in five]}）：{sorted(final.items())}",
                   "看 process_item 的 run_stage：第二段全绿才记 missed，抓到要带第一条失败的测试名（libtest 结果行经 lib_libtest_result_lines.py，should panic 那一段不进名字）；"
                   "抓到它的测试认不出（没有 Running / Doc-tests 行）的记 catching-test-unresolved 并另记覆盖者键")
    # 汇总六栏加起来等于清单条数，没红的逐条列名；第 5、7 条（抓到它们的测试认不出）算拒算、照覆盖者键认
    summary = selftest.run("second", "--summary")
    selftest.check(summary.returncode == EXIT_GATE_RED and selftest_fixtures.SELFTEST_EXPECTED_SUMMARY in summary.stdout and "verdict=red" in summary.stdout
                   and len([line for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]) == 1
                   and [line.split("\t")[1] for line in summary.stdout.splitlines() if line.startswith("REFUSED\t")] == ["5", "7"],
                   f"汇总不对（退 {summary.returncode}）：{summary.stdout}{summary.stderr}", "看 summarise 的六栏、refused=、MISSED 与 REFUSED 明细")
    # 第二段只补第一段没跑到的：没红的第 4 条第二段依次调的就是这三步（文档测试、验证包的编译与测试），命令里没有重跑第一段的那一遍
    stage_two_of_four = [call[1] for call in selftest.calls() if call[0] == "4" and call[1].startswith("two-")]
    table = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False))}
    commands_are_right = (
        table.get("two-test-doc") == ["cargo", "test", "--locked", "--doc", "-p", "singlefs-core", "-p", "singlefs-checker"]
        and table.get("two-test-verification") == ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]
        and table.get("one-test", [])[:3] == ["eatmydata", "cargo", "nextest"]
        and not any(name.startswith("two-test") and command[-len(table["one-test"]):] == table["one-test"] for name, command in table.items()))
    selftest.check(stage_two_of_four == selftest_fixtures.SELFTEST_STAGE_TWO_STEPS and commands_are_right,
                   f"第二段的步不对：第 4 条调的是 {stage_two_of_four}；各步的命令 {table}",
                   "看 step_table：第二段 = 文档测试（被变异的两个包）+ 验证包的全部测试（登记的崩溃枚举用例与放量分项在起命令时经 --skip 排掉），不重跑第一段那一遍")
    # 汇总对不上必须报：给第 4 条的键追加一行认不出的结局
    four_rows = selftest.rows_of("second", 4)
    if not four_rows:
        # 前面几格红了、第 4 条一行判定都没有：这一格判不了，记判错，不在这里抛出去（后面的格照判）
        selftest.check(False, "判定库里有认不出的结局时汇总没判红（前提不成立：second 里第 4 条一行判定都没有，照上面几格先修）",
                       "看 run_items：second 那几趟要给第 4 条落判定行，这一格拿它最后一行造认不出的结局")
    else:
        four = four_rows[-1]
        selftest.append_rows("second", VERDICTS_TABLE, [[four[column] if column != "outcome" else "no-such-outcome" for column in VERDICT_COLUMNS]])
        mismatched = selftest.run("second", "--summary")
        selftest.check(mismatched.returncode == EXIT_INCONSISTENT, f"判定库里有认不出的结局时汇总没判红（退 {mismatched.returncode}）",
                       "看 summarise 末尾的 problems：对不上要退 1")
    # --from（只跑第一段）
    selftest.clear_calls()
    from_seven = selftest.run("third", "--from", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(from_seven.returncode == 0 and selftest.called_sequences() == {"0", "7", "8"},
                   f"--from 7 跑的不是第 7、8 条（退 {from_seven.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 start_from")
    # 上一趟被 KILL 留下的运行目录（树里手改过一份、多出一份没登记的测试文件，编译目录里有旧产物，没有在途记录）：开跑先删掉它，
    # 这一趟在新建的运行目录里照源码换树、照常判，跑完这一趟的也删掉
    selftest.clear_calls()
    stale_run = os.path.join(work, "third", "runs", "run-4242-left-by-a-killed-run")
    stale_tree = os.path.join(stale_run, "tree")
    shutil.copytree(selftest.source, stale_tree, ignore=shutil.ignore_patterns(".git"))
    with open(os.path.join(stale_tree, "crates/singlefs-core/src/lib.rs"), "a") as handle:
        handle.write("// 手改\n")
    extra = os.path.join(stale_tree, "crates/singlefs-core/tests/new_case.rs")
    os.makedirs(os.path.dirname(extra), exist_ok=True)
    with open(extra, "w") as handle:
        handle.write("#[test]\nfn always_red() { panic!() }\n")
    os.makedirs(os.path.join(stale_run, "target", "debug"))
    with open(os.path.join(stale_run, "target", "debug", "stale-product"), "w") as handle:
        handle.write("上一趟编的产物\n")
    dirty = selftest.run("third", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    third_calls = [call for call in selftest.calls() if call[0] == "1"]
    selftest.check(dirty.returncode == 0 and "1" in selftest.called_sequences() and third_calls and all(call[2] == "mutated" for call in third_calls)
                   and f"删了这一趟的编译目录 {stale_run}" in dirty.stderr and not selftest.build_leftovers("third"),
                   f"上一趟被 KILL 留下的运行目录（手改过的树、多出的测试文件、旧产物）没在开跑时删掉、或这一趟没照常判（退 {dirty.returncode}；"
                   f"调用 {sorted(selftest.called_sequences())}；库目录里还剩 {selftest.build_leftovers('third')}）：{dirty.stdout[-400:]}{dirty.stderr[-300:]}",
                   "看 Library.begin_run：拿到锁之后删 runs/ 下上一趟留下的每一个运行目录（删之前核没有进程开着），再新建这一趟的（弄坏开关 interrupted-run-directory-kept 下不删）")
    return listed_lines


def selftest_state_cells(selftest):
    """一条的状态不取最后一行、点名跳过沿用得了的条（库「first」接着用：第 1 条第一段抓到，第 7 条第二段抓到）。"""
    selftest.clear_calls()
    skipped = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rows_of_one = selftest.rows_of("first", 1)
    selftest.check(skipped.returncode == 0 and "1" not in selftest.called_sequences() and len(rows_of_one) == 1 and "跳过" in skipped.stdout,
                   f"点名沿用得了的条没跳过（退 {skipped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；第 1 条 {len(rows_of_one)} 行）：{skipped.stdout}",
                   "看 select_items 的点名那一支：沿用得了的条不选，逐条说一句")
    again = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    selftest.check(again.returncode == 0 and len(selftest.rows_of("first", 1)) == 2,
                   f"点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）没有重跑并追加新行（退 {again.returncode}）：{again.stderr}",
                   "看 select_items：设了 BROAD_MUTATION_REJUDGE_NAMED 的点名不看状态，判定库只追加")
    # 第二段的定论不被后来的 stage-one-only 行降级：第 7 条（第二段抓到）点名重判只跑第一段，状态还是 caught；
    # 之后的续跑照旧现判它（抓到它的测试认不出是拒算，拒算的条每趟现判），续跑本身要成功
    downgrade = selftest.run("first", "--item", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    last_of_seven = [row["outcome"] for row in selftest.rows_of("first", 7)][-1:]
    state_of_seven = selftest.states("first").get("7")   # 续跑之前取：续跑会给第 7 条补上新的定论行，之后取就看不出降级
    selftest.clear_calls()
    resumed = selftest.run("first", "--from", "7", "--package", "singlefs-checker")
    selftest.check(downgrade.returncode == 0 and last_of_seven == [STAGE_ONE_ONLY] and state_of_seven == "caught"
                   and resumed.returncode == 0 and {"7", "8"} <= selftest.called_sequences(),
                   f"第二段的定论被后来的 stage-one-only 行降级了（第 7 条最后一行 {last_of_seven}，状态 {state_of_seven}；续跑里被调的 {sorted(selftest.called_sequences())}）",
                   "看 verdict_state：caught / missed / unviable 取其中最后一行，stage-one-only 行不盖它们")
    # 抓到的条点名重判时一步退了认不出的码、记了 driver-error：状态还是 caught
    selftest.write_plan([("1", "one-test", "exit:104")])
    busy = selftest.run("first", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    outcomes_of_one = [row["outcome"] for row in selftest.rows_of("first", 1)]
    selftest.check(busy.returncode == 0 and outcomes_of_one == ["caught", "caught", DRIVER_ERROR] and selftest.states("first").get("1") == "caught",
                   f"抓到的条被后来的驱动错行盖掉了（第 1 条各行 {outcomes_of_one}，状态 {selftest.states('first').get('1')}）",
                   "看 verdict_state：driver-error 不是定论，不盖掉前面的 caught / missed / unviable")
    # 认不出的选项（用一个从没用过的名字）：退 2、一步都不调
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    unknown_option = selftest.run("first", "--option-never-used-before")
    selftest.check(unknown_option.returncode == EXIT_USAGE and not selftest.calls() and len(selftest.rows("first")) == rows_before and "--option-never-used-before" in unknown_option.stderr,
                   f"认不出的选项没拒（退 {unknown_option.returncode}，调了 {len(selftest.calls())} 步）：{unknown_option.stderr}",
                   "看 parse_arguments：只登记现有的选项，别的一律走 argparse 的用法错（ArgumentParser.error 退 2）")


def selftest_table_cells(selftest):
    """判定库与清单表：清单表被改坏、认不出的退出码。"""
    # 清单按单元存的表被改坏（第 1 条那个单元的那一行换成原文对不上的一条）：--summary 判红退 1、--list-items 少那一条
    selftest.run("names", "--refresh-list")
    names = selftest.library("names")
    one = next(((unit_key, entries) for unit_key, entries in names.read_list_cache().items() if "MUTATED_1()" in json.dumps(entries, ensure_ascii=False)), None)
    if one is not None:
        entries = one[1]
        entries[0]["original"] = "不是这一段原文"
        names.write_list_cache([[one[0], json.dumps(entries, ensure_ascii=False)]])
    summary = selftest.run("names", "--summary")
    listed = selftest.listed("names")
    selftest.check(one is not None and summary.returncode == EXIT_INCONSISTENT and "复原不回去" in summary.stderr and len(listed) == len(selftest.mutants) - 1,
                   f"清单按单元存的表被改坏了没判红（汇总退 {summary.returncode}；列项 {len(listed)} 条）：{summary.stderr[-300:]}",
                   "看 restore_list 与 summarise 的 unrestorable：表里的一条在此刻的源码上复原不回去要报出来、退 1")
    # cargo-mutants 列了被变异的两个包之外的一条（singlefs-format 里的）：那个单元不记清单，复原不回去，--refresh-list 拒（退 1）
    format_path = "crates/singlefs-format/src/lib.rs"
    format_text = selftest_fixtures.SELFTEST_SOURCES[format_path]
    marker = "fn in_lib_case() {}"
    line_index = format_text.split("\n").index("    " + marker)
    start = (line_index + 1, 5 + len(marker) - 2)
    mutated = replace_span(format_text, start, (line_index + 1, 5 + len(marker)), "{ panic!() }" + MUTATION_MARKER)
    stray = {"package": "singlefs-format", "file": format_path, "name": f"{format_path}:{start[0]}:{start[1]}: replace in_lib_case with panic",
             "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": start[0], "column": 5 + len(marker)}}, "replacement": "{ panic!() }",
             "diff": "".join(difflib.unified_diff(format_text.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{format_path}", f"b/{format_path}"))}
    selftest.write_mutants([dict(stray, package="singlefs-core")] + selftest.mutants)
    try:
        outside = selftest.run("outside-list", "--refresh-list")
    finally:
        selftest.write_mutants(selftest.mutants)
    selftest.check(outside.returncode == EXIT_INCONSISTENT and "复原不回去" in outside.stderr,
                   f"清单里有被变异包之外的一条时 --refresh-list 没拒（退 {outside.returncode}）：{outside.stderr[-300:]}",
                   "看 record_list_cache：清单按单元记进判定库 lists/ 之后照它复原一遍，与 cargo-mutants 给的逐条比，复原不回去就拒（弄坏开关 list-cache-unchecked 下不比）")
    # 一步退了认不出的码：落一行 driver-error、往下走，汇总单列；编译步退 101 而没有编译错误的不算无效；续跑时重试这两条
    selftest.write_plan([("2", "one-build", "exit:101"), ("3", "one-test", "exit:104")])
    selftest.clear_calls()
    first = selftest.run("unknown", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("unknown")}
    summary = selftest.run("unknown", "--summary")
    doubt_checked = selftest.environment_steps()
    selftest.clear_calls()
    retried = selftest.run("unknown", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and outcomes.get("2") == DRIVER_ERROR and outcomes.get("3") == DRIVER_ERROR and outcomes.get("4") == STAGE_ONE_ONLY
                   and len(outcomes) == 8 and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "driver_error") == 2
                   and len([line for line in summary.stdout.splitlines() if line.startswith("DRIVER_ERROR\t")]) == 2
                   and doubt_checked == ["one-build"] and retried.returncode == 0 and selftest.judged_sequences() == {"2", "3"},
                   f"认不出的退出码没处理对（退 {first.returncode}；各条 {sorted(outcomes.items())}；汇总退 {summary.returncode}：{summary.stdout.splitlines()[:1]}；"
                   f"当场验的步 {doubt_checked}；续跑退 {retried.returncode}、被调的序号 {sorted(selftest.judged_sequences())}）：{first.stderr}",
                   "看 process_item 的 run_stage：认不出的码记 driver-error 往下走；编译步退 101 要日志里有带源码位置的编译错误才记 unviable；"
                   "select_items 把 driver-error 的条当沿用不了")


def selftest_environment_cells(selftest):
    """基线绿过之后环境坏了要发现得了：重起先验编译、连着几条无效或秒红的抓到就在原件上重验。"""
    selftest.run("environment", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN + [(ENVIRONMENT_SEQUENCE, "one-build", "exit:101")])
    selftest.clear_calls()
    broken = selftest.run("environment", "--item", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(broken.returncode == EXIT_BASELINE_RED and selftest.environment_steps() == ["one-build"] and not selftest.called_sequences()
                   and not selftest.rows_of("environment", 3) and "疑似环境坏了" in broken.stderr,
                   f"基线绿过之后编译坏了，重起没发现（退 {broken.returncode}；验的步 {selftest.environment_steps()}；被调的序号 {sorted(selftest.called_sequences())}）：{broken.stderr}",
                   "看 Runner.verify_environment_at_start：这一趟没现跑基线的那一段，开跑前在没改过的树上把编译步各跑一遍")
    # 连续 K 条无效而原件上编译照过：不是环境，接着跑
    selftest.write_plan([])
    selftest.run("streak", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([(str(sequence), "one-build", "fail-compile") for sequence in (1, 2, 3)])
    selftest.clear_calls()
    continued = selftest.run("streak", "--package", "singlefs-core", "--suspect-streak", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    outcomes = {row["sequence"]: row["outcome"] for row in selftest.rows("streak")}
    selftest.check(continued.returncode == 0 and selftest.environment_steps() == ["one-build", "one-build"]
                   and [outcomes.get(str(sequence)) for sequence in (1, 2, 3, 4, 5, 6)] == ["unviable"] * 3 + [STAGE_ONE_ONLY] * 3 and "不是环境" in continued.stdout,
                   f"连续无效而原件上编译照过时没有接着跑（退 {continued.returncode}；验的步 {selftest.environment_steps()}；各条 {sorted(outcomes.items())}）：{continued.stderr}",
                   "看 SuspectStreaks.saw：到了 K 条就在没改过的树上重跑那一步，绿了清掉这一串接着跑")
    # 连续 K 条秒红的抓到而原件上那一步也红：退 6 报疑似环境坏了、列出这 K 条不可信的行，后面的条不跑
    selftest.write_plan([])
    selftest.run("fast", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in (1, 2, 3, 4)] + [(ENVIRONMENT_SEQUENCE, "one-test", "exit:100")])
    selftest.clear_calls()
    stopped = selftest.run("fast", "--suspect-streak", "3", "--suspect-test-seconds", "30", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(stopped.returncode == EXIT_BASELINE_RED and selftest.called_sequences() == {"1", "2", "3"} and selftest.environment_steps() == ["one-build", "one-test"]
                   and "这 3 条（第 1、2、3 条）的行是环境坏着判的、不可信" in stopped.stderr,
                   f"连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了（退 {stopped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；"
                   f"验的步 {selftest.environment_steps()}）：{stopped.stderr}",
                   "看 SuspectStreaks.saw 与 Runner.verify_on_pristine_tree：原件上那一步红了退 6，出路里列出这几条不可信的行")


def selftest_sync_cells(selftest):
    """恢复原件的临时件不进树、恢复半途被杀之后下一趟照常判；换了工具链每把键都变。"""
    work = selftest.work
    killed = selftest.run("extras", "--item", "3", BROAD_MUTATION_KILL_AT="restore-after-copy", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    killed_runs = selftest.leftover_run_directories("extras")
    stray = sorted(files_in_tree(os.path.join(killed_runs[0], "tree")) - set(selftest_fixtures.SELFTEST_SOURCES)) if len(killed_runs) == 1 else ["（runs/ 下不是恰好一个运行目录）"]
    restore_beside_tree = len(killed_runs) == 1 and os.path.exists(os.path.join(killed_runs[0], "inflight.restore"))
    rerun = selftest.run("extras", "--item", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(killed.returncode == -signal.SIGKILL and not stray and restore_beside_tree and rerun.returncode == 0 and not selftest.build_leftovers("extras"),
                   f"恢复原件半途被杀之后树里多出了文件、临时件不在运行目录里、或重跑没照常判完（被杀那趟退 {killed.returncode}；多出 {stray}；"
                   f"重跑退 {rerun.returncode}；库目录里还剩 {selftest.build_leftovers('extras')}）：{rerun.stderr}",
                   "看 restore_original：临时件建在这一趟的运行目录里（inflight.restore），不建在树里；Library.begin_run 开跑先删上一趟留下的运行目录")
    # 工具链版本不进每一把键：换了 rustc -vV 的输出，判过的条照样沿用（现判 0、沿用 8）；换回来也照沿用
    first_toolchain, second_toolchain = "rustc 1.0.0 (selftest a)", "rustc 2.0.0 (selftest b)"
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    judged = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=first_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.clear_calls()
    changed = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=second_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rejudged = selftest.judged_sequences()
    selftest.clear_calls()
    back = selftest.run("toolchain", BROAD_MUTATION_TOOLCHAIN=first_toolchain, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(judged.returncode == 0 and changed.returncode == 0 and not rejudged and plan_field(changed.stdout, "carried") == 8
                   and back.returncode == 0 and not selftest.judged_sequences() and plan_field(back.stdout, "carried") == 8,
                   f"工具链版本进了键（换了 rustc -vV 之后现判的 {sorted(rejudged)}、沿用 {plan_field(changed.stdout, 'carried')}；换回来现判的 {sorted(selftest.judged_sequences())}、"
                   f"沿用 {plan_field(back.stdout, 'carried')}）：{changed.stderr[-300:]}",
                   "看 Judgement 的 globals（mutation-prune.py 的 global_lines：平常是空的，rustc -vV 各行与工具版本不进键，弄坏开关 toolchain-version-keyed 把它们收进来）")
    # exclude_re（等价变异的出路）走一遍：被判那棵树的 .cargo/mutants.toml 写一条 exclude_re 排掉第 3 条，再跑：清单少那一条（--list-items、--summary 都是 7 条）、
    # 清单整版重判（mutants.toml 解析后的值进清单按单元存的键：lists/ 每个单元重写一条），别的条照沿用（判定的键里没有 mutants.toml，现判 0）。
    # 假清单驱动照 cargo-mutants 的读法在专用树里读 .cargo/mutants.toml、按变异名 re.search 去掉匹配上的。弄坏开关 exclude-re-dropped（换树不拷 .cargo/mutants.toml）下清单照旧 8 条，这一格判错
    excluding_driver = os.path.join(work, "list-driver-exclude-re.py")
    with open(excluding_driver, "w") as handle:
        handle.write("#!/usr/bin/env python3\n"
                     "# 假的 cargo mutants --list --json --diff：照专用树里 .cargo/mutants.toml 的 exclude_re 去掉变异名匹配上的那几条（cargo-mutants 按变异名 re.search）\n"
                     "import json, os, re, sys, tomllib\n"
                     "with open(os.path.join(os.environ['BROAD_MUTATION_SELFTEST_WORK'], 'mutants.json')) as handle:\n"
                     "    mutants = [mutant for mutant in json.load(handle) if mutant['package'] == sys.argv[1]]\n"
                     "patterns = []\n"
                     "if os.path.isfile(os.path.join('.cargo', 'mutants.toml')):\n"
                     "    with open(os.path.join('.cargo', 'mutants.toml'), 'rb') as handle:\n"
                     "        patterns = tomllib.load(handle).get('exclude_re', [])\n"
                     "print(json.dumps([mutant for mutant in mutants if not any(re.search(pattern, mutant['name']) for pattern in patterns)]))\n")
    os.chmod(excluding_driver, 0o755)
    excluded_source = selftest_source_copy(selftest, "exclude-re-source")
    excluded_name = next(mutant["name"] for mutant in selftest.mutants if mutant["replacement"] == "MUTATED_3()")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    before_exclusion = selftest.run("exclude-re", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    before_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    write_sources(excluded_source, {".cargo/mutants.toml": "# 自证：第 3 条当等价变异排掉\nexclude_re = ['MUTATED_3\\(\\)']\n"})
    selftest.clear_calls()
    after_exclusion = selftest.run("exclude-re", source=excluded_source, BROAD_MUTATION_LIST_DRIVER=excluding_driver, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rejudged_after_exclusion = selftest.judged_sequences()
    after_names = sorted(entry["name"] for entry in selftest.listed("exclude-re", excluded_source).values())
    excluded_summary = selftest.run("exclude-re", "--summary", source=excluded_source)
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.clear_calls()
    list_line = re.search(r"清单：(\d+) 条，按单元记进判定库 lists/（(\d+) 个单元，这一趟新写 (\d+) 条）", after_exclusion.stdout)
    selftest.check(before_exclusion.returncode == 0 and len(before_names) == 8 and excluded_name in before_names
                   and after_exclusion.returncode == 0 and after_names == sorted(name for name in before_names if name != excluded_name)
                   and list_line is not None and list_line.group(1) == "7" and list_line.group(2) == list_line.group(3) and not rejudged_after_exclusion
                   and "BROAD_MUTATION_SUMMARY total=7 " in excluded_summary.stdout,
                   f"exclude_re 没排掉那一条、清单没整版重判、或别的条没沿用（头一趟退 {before_exclusion.returncode}、{len(before_names)} 条；写了 exclude_re 之后退 "
                   f"{after_exclusion.returncode}、{len(after_names)} 条、还在不在 {excluded_name in after_names}；清单那一行 {list_line.group(0) if list_line else '（没打）'}；"
                   f"现判的 {sorted(rejudged_after_exclusion)}；--summary {next((line for line in excluded_summary.stdout.splitlines() if line.startswith('BROAD_MUTATION_SUMMARY')), '（没打）')}）："
                   f"{after_exclusion.stdout[-300:]}{after_exclusion.stderr[-300:]}",
                   "看 sync_tree（.cargo/mutants.toml 要随导出范围拷进专用树，cargo-mutants 才读得到 exclude_re）与 record_list_cache（清单按单元存的键带 mutants.toml 解析后的值）")


def selftest_selections(selftest):
    """假一步记的 [(序号, 步名, 跑了哪些)]。"""
    path = os.path.join(selftest.work, "selections.log")
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]


def selftest_source_copy(selftest, name, changes=None, git=False):
    """另一份被判的工作树：照 SELFTEST_SOURCES 写一份，再照 changes 换掉几份文件；git 为真时先 git init 提交一次（之后的改动是「某提交之后改过」）。"""
    directory = os.path.join(selftest.work, name)
    write_sources(directory, selftest_fixtures.SELFTEST_SOURCES)
    if git:
        quiet = {"stdout": subprocess.DEVNULL, "stderr": subprocess.DEVNULL, "stdin": subprocess.DEVNULL}
        environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        subprocess.run(["git", "init", "-q", directory], check=True, env=environment, **quiet)
        subprocess.run(["git", "-C", directory, "add", "-A"], check=True, env=environment, **quiet)
        subprocess.run(["git", "-C", directory, "-c", "user.email=selftest@invalid", "-c", "user.name=selftest", "commit", "-q", "-m", "自证"], check=True,
                       env=environment, **quiet)
    write_sources(directory, changes or {})
    return directory


def selftest_prune_cells(selftest):
    """剪枝：按覆盖挑测试（C2）、第二段按二进制（E6）、文档测试每次跑（E1）；
    不带参数就是剪枝（沿用数 > 0）、改了一个函数只现判那个函数里的条、所在文件在某提交之后改过别处照样沿用、点名现判（自证变量）判成没红、提交门禁判绿、第二段基线红的出路。"""
    work, core = selftest.work, "crates/singlefs-core/src/lib.rs"
    coverage_one = {"tests": [["fake-package::fake_target", "covers_one"], ["fake-package::fake_target", "covers_two"]],
                    "jobs": [{"binary": "fake-package::fake_target", "test": "covers_one", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "fake-package::fake_target", "test": "covers_two", "covered": [[core, "two"]], "profiles": 1, "exit": 0}], "failed": []}
    coverage_two = {"tests": [["verification:covering_target", "*"], ["verification:lib", "*"]],
                    "jobs": [{"binary": "verification:covering_target", "test": "*", "covered": [[core, "one"]], "profiles": 1, "exit": 0},
                             {"binary": "verification:lib", "test": "*", "covered": [], "profiles": 1, "exit": 0}], "failed": []}
    for name, content in (("coverage-one", coverage_one), ("coverage-two", coverage_two)):
        with open(os.path.join(work, name + ".json"), "w") as handle:
            json.dump(content, handle)
    selftest.write_plan([])
    selftest.clear_calls()
    ran = selftest.run("prune", "--item", "1", "--item", "3")
    for name in ("coverage-one", "coverage-two"):
        os.remove(os.path.join(work, name + ".json"))
    picked = {(entry[0], entry[1]): list(entry[2:]) for entry in selftest_selections(selftest)}
    rows = {row["sequence"]: row for row in selftest.rows("prune")}
    expected = {("1", "one-test"): ["1 tests"],
                ("1", "two-test-verification"): ["covering_target"], ("1", "two-test-doc"): ["all"],
                ("3", "two-test-doc"): ["all"], ("0", "one-test"): ["all"]}
    selftest.check(ran.returncode == 0 and all(picked.get(key) == value for key, value in expected.items())
                   and not any(key in picked for key in (("3", "one-test"), ("3", "two-test-verification")))
                   and ("3", "one-build") in picked and rows.get("3", {}).get("outcome") == "missed" and "零覆盖" in rows.get("3", {}).get("note", ""),
                   f"按覆盖挑测试、第二段按二进制、文档测试每次跑不对（退 {ran.returncode}；各步 {sorted(picked.items())}；第 3 条 {rows.get('3')}）：{ran.stderr[-400:]}",
                   "看 Judgement.selection（执行到容器单元的测试、第二段执行到它的二进制）、Runner.ensure_coverage、Runner.command_for（挑空的步不跑，文档测试一直跑）")
    # 不带参数就是剪枝：头一趟全部现判（抓到 6 条、无效 1 条、没红 1 条），第二趟不带任何选条参数：抓到的 6 条、无效的 1 条沿用，没红的第 4 条不沿用、现判（记红、撤绿、下趟现判）。
    # 弄坏开关 missed-carried（没红照旧沿用）下第二趟一条都不现判，这一格判错
    catching = f"fail-nextest:{selftest_fixtures.SELFTEST_CATCHING_CASE}"
    plan = [(str(sequence), "one-test", catching) for sequence in (1, 3, 5, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
    selftest.write_plan(plan)
    pruned_source = selftest_source_copy(selftest, "pruned-source", git=True)
    first = selftest.run("carry", source=pruned_source)
    selftest.clear_calls()
    second = selftest.run("carry", source=pruned_source)
    second_judged = selftest.judged_sequences()
    second_judge_lines = [line for line in second.stdout.splitlines() if line.startswith("BROAD_MUTATION_JUDGE ")]
    selftest.check(first.returncode == 0 and second.returncode == 0 and second_judged == {"4"} and plan_field(second.stdout, "carried") == 7
                   and "现判 1、沿用 7、拒算 0" in second.stdout and any("sequence=4 reason=missed-rejudged" in line for line in second_judge_lines),
                   f"单独一条没红（CPU 一趟）下一趟没现判、或抓到的、无效的没沿用（头一趟退 {first.returncode}；第二趟退 {second.returncode}、现判的 {sorted(second_judged)}（应只有第 4 条）、"
                   f"沿用 {plan_field(second.stdout, 'carried')}（应是 7）；{second_judge_lines}）：{second.stdout[-400:]}{second.stderr[-300:]}",
                   "看 Judgement.state：真跑时同一把键的定论里最后一个是没红的，状态换成沿用不了（理由 missed-rejudged），抓到、无效照 row_holds 沿用；select_items 把沿用不了的现判")
    # 改了一个函数（five 的体）：只有它里面那一条现判；同一份文件在上一次提交之后改过别处，别的函数里的条照样沿用（不按提交选条）
    with open(os.path.join(pruned_source, core), "w") as handle:
        handle.write(selftest_fixtures.SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n"))
    selftest.write_mutants(selftest_mutants(dict(selftest_fixtures.SELFTEST_SOURCES, **{core: selftest_fixtures.SELFTEST_CORE.replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n")})))
    try:
        selftest.clear_calls()
        changed = selftest.run("carry", source=pruned_source)
        changed_lines = [line for line in changed.stdout.splitlines() if line.startswith("BROAD_MUTATION_JUDGE ")]
        selftest.check(changed.returncode == 0 and selftest.judged_sequences() == {"4", "5"} and plan_field(changed.stdout, "carried") == 6
                       and "现判 2、沿用 6、拒算 0" in changed.stdout and any("改了 five" in line for line in changed_lines),
                       f"同一条变异内容没变、判定库里有可沿用的行、所在文件在某提交之后改过别处，没照样沿用（退 {changed.returncode}；现判的 {sorted(selftest.judged_sequences())}；"
                       f"沿用 {plan_field(changed.stdout, 'carried')}；{changed_lines}）：{changed.stderr[-300:]}",
                       "看 select_items：只按单元键选条（键对不上的现判，作废时 BROAD_MUTATION_JUDGE 报变了的单元），不按 git diff 选（弄坏开关 select-by-commit 恢复按提交选条）")
        # 抓到它的那条测试（catching_one）改了函数体：抓到的那六条的测试键都变了，现判；没红的第 4 条每趟现判；无效的一条照沿用
        cases = "crates/singlefs-format/tests/catching_cases.rs"
        with open(os.path.join(pruned_source, cases), "w") as handle:
            handle.write(selftest_fixtures.SELFTEST_SOURCES[cases].replace("fn catching_one() {}", "fn catching_one() {\n    assert_eq!(2 + 2, 4);\n}"))
        selftest.clear_calls()
        retested = selftest.run("carry", source=pruned_source)
        selftest.check(retested.returncode == 0 and selftest.judged_sequences() == {"1", "3", "4", "5", "6", "7", "8"} and plan_field(retested.stdout, "carried") == 1
                       and "catching-test-changed" in retested.stdout,
                       f"抓到它的那条测试改了之后那几条没现判（现判的 {sorted(selftest.judged_sequences())}；沿用 {plan_field(retested.stdout, 'carried')}）：{retested.stdout[-400:]}",
                       "看 Judgement.row_holds 与 mutation-prune.py 的 catching_test_key：抓到的判定要那条测试的测试键（它的单元闭包键）与此刻的相同才沿用")
        # 点名现判（自证变量 BROAD_MUTATION_REJUDGE_NAMED）：第 6 条先抓到，这一回重判成没红（同一把键），只现判它；汇总按没红数（第 4、6 条）
        selftest.write_plan([row for row in plan if row[0] != "6"])
        selftest.clear_calls()
        rerun = selftest.run("carry", "--item", "6", source=pruned_source, BROAD_MUTATION_REJUDGE_NAMED="1")
        rerun_judged = selftest.judged_sequences()
        summary = selftest.run("carry", "--summary", source=pruned_source)
        selftest.check(rerun.returncode == 0 and rerun_judged == {"6"} and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "missed") == 2,
                       f"先抓到的第 6 条点名现判（自证变量）判成没红之后，汇总没按没红数、或这一趟现判的不是只有它（退 {rerun.returncode}；现判的 {sorted(rerun_judged)}）：{rerun.stdout[-400:]}；"
                       f"汇总 {[line for line in summary.stdout.splitlines() if 'GATE' in line]}",
                       "看 select_items（--item 加 BROAD_MUTATION_REJUDGE_NAMED 沿用得了也重判）与 Judgement.state：同一把键最后一个定论是没红，汇总按没红数")
        # 记红、撤绿、下趟现判：第 6 条在库里先抓到（两回）、点名现判这一回同一把键又判没红 ⇒ 红照键记着，前面的抓到撤掉（不会重新冒出来当状态，--summary 已按没红数），
        # 下一趟第 4 条（单独一条没红）与第 6 条（先抓到、同键改判没红）都现判，抓到、无效的 6 条沿用；现判又出没红，再下一趟还是现判这两条。
        # 弄坏开关 missed-carried（没红照旧沿用）下这两趟一条都不现判
        selftest.clear_calls()
        flipped = selftest.run("carry", source=pruned_source)
        flipped_judged = selftest.judged_sequences()
        selftest.clear_calls()
        again_red = selftest.run("carry", source=pruned_source)
        again_red_judged = selftest.judged_sequences()
        selftest.check(flipped.returncode == 0 and flipped_judged == {"4", "6"} and plan_field(flipped.stdout, "carried") == 6 and "现判 2、沿用 6、拒算 0" in flipped.stdout,
                       f"先抓到、同一把键又判没红（CPU 一趟）下一趟没现判、或现判之后没红仍沿用（退 {flipped.returncode}；现判的 {sorted(flipped_judged)}（应是第 4、6 条）；"
                       f"沿用 {plan_field(flipped.stdout, 'carried')}（应是 6）；再下一趟退 {again_red.returncode}、现判的 {sorted(again_red_judged)}）：{flipped.stdout[-300:]}",
                       "看 Judgement.state：同一把键全部定论里最后一个是没红的，不看更早的抓到还认不认；现判又出没红的下一趟照样现判（没红每趟现判，直到判出抓到或无效）")
        selftest.check(again_red.returncode == 0 and again_red_judged == {"4", "6"} and plan_field(again_red.stdout, "carried") == 6,
                       f"现判又出没红的条（CPU 一趟）下一趟没接着现判（退 {again_red.returncode}；现判的 {sorted(again_red_judged)}（应是第 4、6 条）；沿用 {plan_field(again_red.stdout, 'carried')}）：{again_red.stdout[-300:]}",
                       "看 Judgement.state：没红每趟现判，不因为最后两个定论一致就沿用")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    # 全部有结论、没有没红：提交门禁绿
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    selftest.run("green", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    green = selftest.run("green", "--summary")
    selftest.check(green.returncode == 0 and "verdict=green" in green.stdout and "refused=0" in green.stdout,
                   f"每条都抓到的提交门禁没判绿（退 {green.returncode}）：{green.stdout[-300:]}{green.stderr[-300:]}",
                   "看 summarise：抓到、没红、无效都算有结论，没红为零、没有单元缺清单才绿")
    # 第二段基线红：报拒绝、点名红的测试、给出路
    selftest.write_plan([("0", "two-test-verification", "fail-cargo")])
    red = selftest.run("stage-two-red", "--item", "4")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(red.returncode == EXIT_BASELINE_RED and selftest_fixtures.SELFTEST_STAGE_TWO_CASE in red.stderr and "修好之前这一道过不去" in red.stderr and "crash_identity" in red.stderr,
                   f"第二段基线红没报成带出路的拒绝（退 {red.returncode}）：{red.stderr[-400:]}",
                   "看 Runner.refuse_red_baseline 第二段那一支：列出红的测试，出路写已知的那条测试要谁改、修好之前这一道过不去")
    selftest.clear_calls()


def selftest_reuse_cells(selftest):
    """复用三问（reuse-three-questions）与复用哨兵（reuse-sentinel），写法在文件头「复用三问与复用哨兵」。项里除了 CPU 那一趟的 1–8 条，
    还有 GPU 那一趟的（项名 gpu:<序号>：CPU 那一趟没红、又有带 gpu 特性的测试够得着的第 4、5 条），两趟的复用一起问；复用哨兵 CPU 一份、GPU 一份。"""
    core = "crates/singlefs-core/src/lib.rs"
    head = "pub fn five() -> u32 {\n"
    # 第 4、5 条 CPU 一趟没红、GPU 一趟抓到：合起来是抓到，两趟的行都沿用（CPU 一趟的没红行只在 GPU 趟也没红时才算红、每趟现判，文件头「记红、撤绿、下趟现判」）；
    # 要是 GPU 趟也没红，这两条每趟现判，问不了「同一份输入第二趟一条都不现判」
    plan = ([(str(sequence), "one-test", f"fail-nextest:{selftest_fixtures.SELFTEST_CATCHING_CASE}") for sequence in (1, 3, 6, 7, 8)] + [("2", "one-build", "fail-compile")]
            + [(str(sequence), GPU_TEST_STEP, "gpu-fail") for sequence in (4, 5)])
    selftest.write_plan(plan)
    source = selftest_source_copy(selftest, "three-questions-source", selftest_fixtures.SELFTEST_GPU_FILES)
    commented = selftest_fixtures.SELFTEST_CORE.replace(head, head + "    // 只加一行注释\n")
    changed = commented.replace(head, head + "    let _unused = 0;\n")

    def judged_run(text=None):
        if text is not None:
            with open(os.path.join(source, core), "w") as handle:
                handle.write(text)
            selftest.write_mutants(selftest_mutants(dict(selftest_fixtures.SELFTEST_SOURCES, **{core: text})))
        selftest.clear_calls()
        ran = selftest.run("three-questions", source=source)
        return ran, selftest.judged_sequences() | {f"gpu:{sequence}" for sequence in selftest.gpu_judged_sequences()}

    try:
        runs = [judged_run(), judged_run(), judged_run(commented), judged_run(changed)]
        items = sorted(runs[0][1])
        states = [{sequence: reuse_checks_library().JUDGED if sequence in judged else reuse_checks_library().REUSED for sequence in items} for _ran, judged in runs]
        # 改 five 的体：CPU 那一趟第 5 条的键变、第 4 条的覆盖者键变（够得着它的 gpu_case 的测试键含 five）；GPU 那一趟第 4、5 条的覆盖者键同样变
        passed, problems = reuse_checks_library().report_three_questions(SCRIPT_KEY, *states, ["4", "5", "gpu:4", "gpu:5"])
        selftest.check(passed and all(ran.returncode == 0 for ran, _judged in runs),
                       f"{reuse_checks_library().CELL_NAME}：四趟退 {[ran.returncode for ran, _judged in runs]}，现判的 {[sorted(judged) for _ran, judged in runs]}：{'；'.join(problems)}",
                       "照规范输出那一行 red 的那一问查：① 看 select_items 与 Judgement.state（键相同、沿用判据认的条不选）；② 看 mutation-prune.py 的单元键（注释不进指纹）；"
                       "③ 看容器单元的闭包（只改 five 只现判它里面那一条）")
        # 复用哨兵：库里 8 条都有定论之后，同一个种子照常跑：抽中的那一条强制现判、判出同样的结局、整趟绿；弄坏开关 rerun-done（沿用得了的照样重跑）下判红「复用失灵」
        sentinel_environment = {reuse_checks_library().SENTINEL_COUNT_VARIABLE: "1", reuse_checks_library().SENTINEL_SEED_VARIABLE: "5"}
        selftest.clear_calls()
        green = selftest.run("three-questions", source=source, **sentinel_environment)
        forced_lines = re.findall(r"强制现判 1 条：(\S+)", green.stdout)
        green_judged, green_gpu_judged = selftest.judged_sequences(), selftest.gpu_judged_sequences()
        selftest.clear_calls()
        skipped = selftest.run("three-questions", source=source, BROAD_MUTATION_BREAK="rerun-done", **sentinel_environment)
        selftest.clear_calls()
        gpu_skipped = selftest.run("three-questions", source=source, BROAD_MUTATION_BREAK="gpu-carry-dropped", **sentinel_environment)
        selftest.check(green.returncode == 0 and "复用哨兵（广谱变异）：种子 5" in green.stdout and "复用哨兵（广谱变异 GPU 趟）：种子 5" in green.stdout and len(forced_lines) == 2
                       and green_judged == {forced_lines[0]} and forced_lines[1].startswith("gpu:") and green_gpu_judged == {forced_lines[1].removeprefix("gpu:")}
                       and "reason=sentinel-forced" in green.stdout and skipped.returncode == 1 and "复用失灵：" in skipped.stderr
                       and gpu_skipped.returncode == 1 and "复用失灵：gpu:" in gpu_skipped.stderr,
                       f"{reuse_checks_library().SENTINEL_CELL_NAME}：照常那一趟退 {green.returncode}、抽中 {forced_lines}、现判的 {sorted(green_judged)}、GPU 趟现判的 {sorted(green_gpu_judged)}："
                       f"{green.stdout[-500:]}{green.stderr[-200:]}；CPU 沿用得了也重跑的那一趟退 {skipped.returncode}：{skipped.stderr[-300:]}；"
                       f"GPU 沿用得了也重跑的那一趟退 {gpu_skipped.returncode}：{gpu_skipped.stderr[-300:]}",
                       "看 plan_reuse_sentinel、finish_reuse_sentinel（CPU 那一趟）与 plan_gpu_sentinel、run_gpu_phase 里候选与 observe（GPU 那一趟，项名 gpu:<序号>）：候选是沿用判据认定论的条，"
                       "抽中强制现判的那一条加进选中的条；判法在 scripts/gate/lib_reuse_checks.py 文件头「二」")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
        selftest.clear_calls()


def selftest_global_cells(selftest):
    """全局那几行：多登记一条崩溃枚举用例（排除单多一条）、只改 Cargo.toml 的注释，别的条照样沿用；改了构建环境的生效值（RUSTFLAGS），判过的条也照样沿用。"""
    selftest.write_plan([(str(sequence), "one-test", f"fail-nextest:{selftest_fixtures.SELFTEST_CATCHING_CASE}") for sequence in range(1, 9)])
    source = selftest_source_copy(selftest, "global-source", selftest_fixtures.SELFTEST_ENTRY_BASELINE_SOURCES)   # 带一个 验证包：登记的那一条在它的 tests/quick.rs 里对得上
    first = selftest.run("globals", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases = []
    selftest.clear_calls()
    excluded = selftest.run("globals", source=source, BROAD_MUTATION_REGISTERED="quick:kept_in_a_test_target", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases.append(("多登记一条", excluded, selftest.judged_sequences(), "exclusions-in-global-key"))
    with open(os.path.join(source, "Cargo.toml"), "w") as handle:
        handle.write("# 只改注释：成员与 profile 一样没变\n" + selftest_fixtures.SELFTEST_SOURCES["Cargo.toml"])
    selftest.clear_calls()
    commented = selftest.run("globals", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    cases.append(("只改 Cargo.toml 注释", commented, selftest.judged_sequences(), "manifest-text-in-global-key"))
    for what, ran, judged, switch in cases:
        selftest.check(first.returncode == 0 and ran.returncode == 0 and not judged and plan_field(ran.stdout, "carried") == 8,
                       f"{what}之后别的条没照样沿用（退 {ran.returncode}；现判的 {sorted(judged)}；沿用 {plan_field(ran.stdout, 'carried')}）：{ran.stderr[-300:]}",
                       f"看 mutation-prune.py 的 global_lines：排除单、Cargo.toml 原文都不进键（弄坏开关 {switch} 把它放进键）")
    selftest.clear_calls()
    flags = selftest.run("globals", source=source, RUSTFLAGS="-C selftest-flag", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    flags_judged = selftest.judged_sequences()
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and flags.returncode == 0 and not flags_judged and plan_field(flags.stdout, "carried") == 8,
                   f"改了 RUSTFLAGS 之后判过的条没有照样沿用（现判的 {sorted(flags_judged)}；沿用 {plan_field(flags.stdout, 'carried')}）：{flags.stderr[-300:]}",
                   "看 mutation-prune.py 的 global_lines：平常是空的，编译变量的生效值不进键（弄坏开关 build-environment-keyed 把它收进来）")


def selftest_key_speed_cells(selftest):
    """算键与读库的提速（文件头「算键提速」）：参照是没装提速、串行、逐条读库的原写法。键缓存的读写与抽查（摘要不符、被改的一行、缺行都不读）、prefill 填进去的键与 mutation_key 逐项相同、
    计划分堆的用时取同一处变异的行、批量读上一回组成清单与 Judgement.changed_units 逐条相同、库的写组成清单记住已写的。
    这一组在自证进程里直接调函数，弄坏开关在这里临时打开认（broad_mutation_judging.TEST_MODE）。"""
    import types
    previous_mode, broad_mutation_judging.TEST_MODE = broad_mutation_judging.TEST_MODE, True
    try:
        rules = prune_rules()
        root = selftest.source
        plain, fast = rules.UnitTree(root), rules.UnitTree(root)
        global_lines = rules.global_lines("", root, {}, "rustc 0.0.0-selftest\nhost: x86_64-unknown-linux-gnu", ["cargo-nextest 0.0.0-selftest"])
        selftest.check(not any(line.startswith("judging-units") for line in global_lines),
                       f"全局那几行带了判法单元指纹 {[line[:40] for line in global_lines if line.startswith('judging-units')]}：驱动与算键库的代码进了键",
                       "看 mutation-prune.py 的 global_lines：平常是空的（弄坏开关 judging-units-in-global-key 把这份库的源码文本收进来）")
        profile_and_jobs = {"CARGO_PROFILE_DEV_DEBUG": "false", "CARGO_PROFILE_TEST_OPT_LEVEL": "1", "CARGO_BUILD_JOBS": "3", "CARGO_BUILD_INCREMENTAL": "true"}
        selftest.check(rules.global_lines("", root, profile_and_jobs) == rules.global_lines("", root, {}),
                       "环境变量里的编译档（CARGO_PROFILE_*）与线程数（CARGO_BUILD_JOBS）、增量（CARGO_BUILD_INCREMENTAL）进了构建环境的生效值",
                       "看 mutation-prune.py 的 global_lines：环境变量里的编译档、线程数、增量不收（弄坏开关 profile-keyed、build-jobs-keyed 把它们收进来）")
        items = []
        for number, mutant in enumerate(selftest.mutants, start=1):
            span = mutant["span"]
            items.append({"package": mutant["package"], "name": mutant["name"], "file": mutant["file"], "start": [span["start"]["line"], span["start"]["column"]],
                          "end": [span["end"]["line"], span["end"]["column"]], "replacement": mutant["replacement"], "sequence": number})
        library = types.SimpleNamespace(manifest={"crates/singlefs-core/src/lib.rs": "0" * 64}, write_composition=lambda key, lines: None)
        fake = types.SimpleNamespace(rules=rules, tree=fast, globals=global_lines, keys={}, library=library)
        speed_up_judgement(fake)
        reference = [rules.key_record(plain, item, global_lines) for item in items]
        directory = os.path.join(selftest.work, "key-speed")
        os.makedirs(directory, exist_ok=True)
        path = os.path.join(directory, "key-cache.json")
        records = rules.prefill_records(fast, items, global_lines, 1)
        write_key_cache(path, "digest-one", items, records)
        loaded, why = read_key_cache(fake, items, path, "digest-one")
        selftest.check(len(items) >= 4 and all(record[0] for record in reference) and records == reference and loaded == reference and not why,
                       f"算键缓存写进去再读出来的与现算的不同（{len(items)} 条；缓存读出 {loaded is not None}，原因「{why}」）",
                       "看 write_key_cache、read_key_cache：每条的（键、拒算形态、相对位置、原文、定位、找得到容器单元）原样进出 JSON")
        other, other_why = read_key_cache(fake, items, path, "digest-two")
        selftest.check(other is None and other_why,
                       f"摘要对不上的算键缓存也读了（读出 {other is not None}）",
                       "看 read_key_cache：缓存里的摘要与此刻 key_cache_digest 算的不同就不读（弄坏开关 key-cache-digest-unchecked）")
        with open(path, encoding="ascii") as handle:
            stored = json.load(handle)
        stored["rows"][0][5] = "f" * 64
        write_atomically(path, json.dumps(stored).encode("ascii"))
        tampered, tampered_why = read_key_cache(fake, items, path, "digest-one")
        selftest.check(tampered is None and "抽查" in tampered_why,
                       f"被改了一行键的算键缓存读了（读出 {tampered is not None}，原因「{tampered_why}」）",
                       "看 read_key_cache：读了的按条次序抽查（至多十二条），现算与记录逐项相同才用（弄坏开关 key-cache-sample-unchecked）")
        stored["rows"] = stored["rows"][:-1]
        write_atomically(path, json.dumps(stored).encode("ascii"))
        short, short_why = read_key_cache(fake, items, path, "digest-one")
        selftest.check(short is None and "缺" in short_why,
                       f"缺了一条的算键缓存读了（读出 {short is not None}，原因「{short_why}」）",
                       "看 read_key_cache：这一趟要算的条在缓存里找不全（身份或变异名对不上）就不读")
        filled = types.SimpleNamespace(rules=rules, tree=fast, globals=global_lines, keys={}, library=types.SimpleNamespace(manifest=None, write_composition=lambda key, lines: None))
        speed_up_judgement(filled)
        prefill_keys(filled, items, types.SimpleNamespace(shard_phase="", threads=2), "")
        differences = []
        for item in items:
            view, original = filled.keys.get(key_identity(item)), rules.mutation_key(plain, item, global_lines)
            same = view is not None and all(view[name] == original[name] for name in ("key", "refused", "relative", "original", "locator"))
            same = same and list(view["composition"]) == list(original["composition"]) and (view["unit"] is None) == (original["unit"] is None)
            same = same and (original["unit"] is None or (view["unit"].file, view["unit"].name) == (original["unit"].file, original["unit"].name))
            if not same:
                differences.append(item["sequence"])
        selftest.check(not differences and len(filled.keys) == len(items),
                       f"prefill_keys 填进 Judgement.keys 的键与 mutation_key 不同（第 {differences} 条；填了 {len(filled.keys)}/{len(items)}）",
                       "看 prefill_keys、key_view：键、拒算形态、相对位置、原文、定位、组成清单、容器单元都要与 mutation_key 的返回值一致")
        entries = [{"sequence": number, "name": f"item-{number}", "package": "p", "file": "f", "start": [1, 1], "end": [1, 2], "replacement": "r"} for number in (1, 2, 3, 4)]

        def timed(build, test):
            return {"outcome": "caught", "build_seconds": str(build), "test_seconds": str(test)}
        no_state = ItemState(NOT_RUN, False)
        states = {1: (no_state, "x", timed(60, 40)), 2: (no_state, "x", None), 3: (no_state, "x", timed(10, 20)), 4: (no_state, "x", None)}
        by_locator = {"item-2": [timed(200, 300)], "item-4": [{"outcome": STAGE_ONE_ONLY, "build_seconds": "9", "test_seconds": "9"}]}
        pile_judgement = types.SimpleNamespace(by_locator=by_locator, key_of=lambda item: {"locator": item["name"]})
        piles = shard_piles(entries, states, [(entry, True, True) for entry in entries], types.SimpleNamespace(item=[], start_from=None, package=None), pile_judgement)
        selftest.check([names for _label, names in piles] == [["item-2"], ["item-1", "item-4", "item-3"]]
                       and piles[0][0].startswith("头一堆：现判 1 条约 500 秒、共 1 条") and piles[1][0].startswith("另一堆：现判 3 条约 230 秒、共 3 条"),
                       f"计划分堆没按同一处变异最后一行有定论的用时估（分成 {piles}）",
                       "看 shard_piles：键对不上时取同一处变异（by_locator）最后一行有定论的编译加测试秒数，都没有才按中位数（弄坏开关 shard-seconds-without-locator）")
        old_lines = {"k_old": ["fn\talpha\tcrates/x.rs:1-3\taaa", "fn\tbeta\tcrates/x.rs:5-7\tbbb"], "k_other": ["fn\tgamma\tcrates/x.rs:9-11\tggg"]}
        new_lines = ["fn\talpha\tcrates/x.rs:1-3\taaa2", "fn\tbeta\tcrates/x.rs:5-7\tbbb"]
        listed = [{"sequence": number, "locator": f"L{number}", "key": f"n{number}", "composition": new_lines} for number in (1, 2, 3, 4)]
        rows_by_locator = {"L1": [{"key": "k_old"}], "L2": [{"key": "k_gone"}], "L3": [], "L4": [{"key": "k_old"}, {"key": "k_other"}]}

        def read_many(relatives):
            names = [relative.split("/", 1)[1] for relative in relatives]
            return [("\n".join(old_lines[name]) + "\n").encode() if name in old_lines else None for name in names]
        batch_judgement = types.SimpleNamespace(
            key_of=lambda item: item, by_locator=rows_by_locator,
            library=types.SimpleNamespace(store=types.SimpleNamespace(read_many=read_many),
                                          read_composition=lambda key: old_lines.get(key)))
        wanted = {item["sequence"]: Judgement.changed_units(batch_judgement, item) for item in listed}
        found = changed_units_of(batch_judgement, listed)
        selftest.check(found == {number: text for number, text in wanted.items() if text} and len(found) == 2,
                       f"批量读上一回的组成清单得的「变了的单元」与 Judgement.changed_units 逐条的不同（批量 {found}，逐条 {wanted}）",
                       "看 changed_units_of：取同一处变异最后一行的键、读不到的跳过、比法同 composition_difference（弄坏开关 changed-units-batch-oldest-row）")
        written = []
        memory_library = types.SimpleNamespace(write_composition=lambda key, lines: written.append(key))
        memorise_compositions(memory_library)
        for key in ("k1", "k1", "", "k2"):
            memory_library.write_composition(key, [])
        selftest.check(written == ["k1", "k2"],
                       f"库的写组成清单没记住写过的键（写了 {written}，该是 ['k1', 'k2']）",
                       "看 memorise_compositions：同一把键只交给库一次，空键不写（弄坏开关 composition-write-unmemorised）")
    finally:
        broad_mutation_judging.TEST_MODE = previous_mode


def selftest_fixture_readers(source, fixture_names):
    """驱动源码里读自证夹具的地方 → (不经 selftest_fixtures、直接读夹具名字的 [(行, 名字)]，经 selftest_fixtures 读而不是自证函数的 [顶层单元名])。
    夹具只许自证读：名字以 selftest_ 起头的函数、Selftest、run_selftest 之外的单元读 selftest_fixtures，运行期（不在 --selftest 里）它还是 None；
    夹具的名字在驱动里没有赋值，直接读就是读不到的名字。"""
    import ast
    bare, readers = [], set()
    for node in ast.parse(source).body:
        in_selftest = isinstance(node, (ast.FunctionDef, ast.ClassDef)) and (node.name.startswith("selftest_") or node.name in ("Selftest", "run_selftest"))
        for child in ast.walk(node):
            if not (isinstance(child, ast.Name) and isinstance(child.ctx, ast.Load)):
                continue
            if child.id in fixture_names:
                bare.append((child.lineno, child.id))
            elif child.id == "selftest_fixtures" and not in_selftest:
                readers.add(getattr(node, "name", f"第 {node.lineno} 行起的模块级语句"))
    return bare, sorted(readers)


def selftest_refused_cells(selftest):
    """拒算：容器单元的闭包里有认不出的形态（include_str! 的参数不是字面量），那一条每趟现判、成功那句报拒算条数；别的条照沿用。"""
    core = "crates/singlefs-core/src/lib.rs"
    refused_core = selftest_fixtures.SELFTEST_CORE.replace("pub fn four() -> u32 {\n", "pub fn four() -> u32 {\n    let _ = runtime_text();\n") + \
        'fn runtime_text() -> usize {\n    include_str!(env!("SELFTEST_INCLUDE")).len()\n}\n'
    source = selftest_source_copy(selftest, "refused-source", {core: refused_core})
    selftest.write_mutants(selftest_mutants(dict(selftest_fixtures.SELFTEST_SOURCES, **{core: refused_core})))
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    try:
        first = selftest.run("refused", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        selftest.clear_calls()
        second = selftest.run("refused", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        summary = selftest.run("refused", "--summary", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and second.returncode == 0 and selftest.judged_sequences() == {"4"} and "现判 0、沿用 7、拒算 1" in second.stdout
                   and "REFUSED\t4\tinclude-argument" in summary.stdout,
                   f"拒算的条没有每趟现判、或别的条没沿用（第二趟现判的 {sorted(selftest.judged_sequences())}）：{second.stdout[-300:]}；汇总 {summary.stdout[-300:]}",
                   "看 Judgement.state：真跑时键里有拒算形态的条沿用不了（每趟现判，弄坏开关 refused-carried 下照沿用），--summary 逐条列 REFUSED")


def selftest_baseline_threads_cells(selftest):
    """线程数不进基线的键：--threads 2 下跑过一趟，换 --threads 3 之后测试那一步的基线照 2 线程那一行沿用、不重跑、不追加新行
    （弄坏开关 baseline-by-threads 下它在新线程数下重跑、追加新行）。"""
    selftest.write_plan([("2", "one-test", "fail-nextest")])
    first = selftest.run("threads", "--item", "2", "--threads", "2", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan([("3", "one-test", "fail-nextest")])
    selftest.clear_calls()
    second = selftest.run("threads", "--item", "3", "--threads", "3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    rerun_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    threads = sorted(row["threads"] for row in selftest.library("threads").baseline_rows()
                     if row["step"] == "one-test" and row["outcome"] == "ok")
    selftest.check(first.returncode == 0 and second.returncode == 0 and threads == ["2"] and "one-test" not in rerun_baselines,
                   f"换了 --threads 之后基线没有沿用（头一趟退 {first.returncode}、换线程那一趟退 {second.returncode}；"
                   f"基线表里 one-test 绿行的线程数 {threads}；换线程那一趟重跑的基线 {rerun_baselines}）：{second.stderr[-300:]}",
                   "看 Runner.baseline_rows_of 与 ensure_baseline：基线按（基线键, 步名）取最后一行，不挑线程数；线程数只记在行里")


def selftest_baseline_key_cells(selftest):
    """基线键（文件头「基线」）：编译步拒算、每趟照跑；测试步的键按它跑到的测试的单元闭包取——改一个不在任何测试闭包里的函数（core 的 six），
    第一段测试那一步的基线照样沿用、不重跑；改抓到它的那条第一段测试的函数体，那一步的基线重跑。弄坏开关 baseline-by-packages（包级）下这一格必须判错。"""
    core, catching = "crates/singlefs-core/src/lib.rs", "crates/singlefs-format/tests/catching.rs"
    changed_core = selftest_fixtures.SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-key-source")
    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
        first_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(selftest_fixtures.SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
        outside_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
        write_sources(source, {catching: selftest_fixtures.SELFTEST_SOURCES[catching].replace("assert_eq!(1 + 1, 2);", "assert_eq!(2 + 2, 4);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-key", "--item", "1", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")
        inside_baselines = sorted(call[1] for call in selftest.calls() if call[0] == str(BASELINE_SEQUENCE))
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.check((first.returncode, outside.returncode, inside.returncode) == (0, 0, 0) and first_baselines == ["one-build", "one-test"]
                   and outside_baselines == ["one-build"] and "基线 one-build：基线键拒算" in outside.stdout and inside_baselines == ["one-build", "one-test"],
                   f"基线键没按这一步跑到的判定的单元闭包取（头一趟退 {first.returncode}、基线 {first_baselines}；改了不在测试闭包里的 six 之后退 {outside.returncode}、"
                   f"基线 {outside_baselines}；改了那条第一段测试之后退 {inside.returncode}、基线 {inside_baselines}）：{outside.stdout[-400:]}{inside.stderr[-300:]}",
                   "看 Judgement.baseline_key 与 Runner.baseline_key_of：测试步的键只取它跑到的测试（连同条目级宏调用）各自的单元闭包键、命令与全局那几行，"
                   "不取包里全部单元的指纹、不带步名与 form；编译步拒算，这一趟用 run_token 算一把独有的键照跑")


def selftest_baseline_entry_cells(selftest):
    """按条记基线（文件头「基线」）：第一段测试步里 per_entry 一个测试二进制三条、refused_third 起自己这个可执行文件（闭包拒算）；验证包库里两条
    （一条拒算）、tests/quick.rs 一条。头一趟都照跑、照条记下；改一个不在任何测试闭包里的函数（core 的 six）之后，第一段测试那一步只照跑 refused_third、
    别的三条沿用，验证包只另起只跑拒算那一条的一趟（--exact）；再改 kept_first 的断言，
    第一段测试那一步照跑它与 refused_third。弄坏开关 baseline-refused-step-whole（一条拒算整步照跑）下这一格必须判错。"""
    core = "crates/singlefs-core/src/lib.rs"
    changed_core = selftest_fixtures.SELFTEST_CORE.replace("pub fn six() -> u32 {\n", "pub fn six() -> u32 {\n    let _unused = 0;\n")
    source = selftest_source_copy(selftest, "baseline-entry-source", selftest_fixtures.SELFTEST_ENTRY_BASELINE_SOURCES)

    def baseline_selections():
        """基线那几次假的一步收到的 BROAD_MUTATION_STEP_SELECTION：{步名: [all 或 per-entry 点的那几样（dict）]}。"""
        found = {}
        path = os.path.join(selftest.work, "selections.log")
        if os.path.exists(path):
            with open(path) as handle:
                for line in handle:
                    fields = line.rstrip("\n").split("\t")
                    if len(fields) >= 3 and fields[0] == str(BASELINE_SEQUENCE) and fields[1] in ("one-test", "two-test-verification"):
                        text = fields[2]
                        found.setdefault(fields[1], []).append(json.loads(text[len("per-entry "):]) if text.startswith("per-entry ") else text)
        return found

    selftest.write_plan([])
    try:
        selftest.clear_calls()
        first = selftest.run("baseline-entry", "--item", "4", source=source)
        first_selections = baseline_selections()
        write_sources(source, {core: changed_core})
        selftest.write_mutants(selftest_mutants(dict(selftest_fixtures.SELFTEST_SOURCES, **{core: changed_core})))
        selftest.clear_calls()
        outside = selftest.run("baseline-entry", "--item", "4", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        outside_selections = baseline_selections()
        write_sources(source, {selftest_fixtures.SELFTEST_ENTRY_BASELINE_FILE: selftest_fixtures.SELFTEST_ENTRY_BASELINE_SOURCES[selftest_fixtures.SELFTEST_ENTRY_BASELINE_FILE].replace("assert_eq!(2 + 2, 4);", "assert_eq!(4 + 4, 8);")})
        selftest.clear_calls()
        inside = selftest.run("baseline-entry", "--item", "4", source=source, BROAD_MUTATION_REJUDGE_NAMED="1")
        inside_selections = baseline_selections()
    finally:
        selftest.write_mutants(selftest.mutants)
        selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    stage_one_binary = "singlefs-format::per_entry"
    verification_parts = [selection.get("part") if isinstance(selection, dict) else selection for selection in first_selections.get("two-test-verification", [])]
    inside_tests = inside_selections.get("one-test", [{}])
    selftest.check((first.returncode, outside.returncode, inside.returncode) == (0, 0, 0)
                   and first_selections.get("one-test") == ["all"] and verification_parts == ["whole", "skip", "exact"]
                   and outside_selections.get("one-test") == [{"tests": [[stage_one_binary, "refused_third"]], "binaries": []}]
                   and outside_selections.get("two-test-verification") == [{"binaries": ["singlefs-verification"], "part": "exact"}]
                   and "按条记基线" in outside.stdout
                   and len(inside_tests) == 1 and isinstance(inside_tests[0], dict)
                   and sorted(test for _binary, test in inside_tests[0].get("tests", [])) == ["kept_first", "refused_third"],
                   f"基线没按条记：一步里收得全的测试没各自沿用、或拒算的那几条没照跑（头一趟退 {first.returncode}、第一段测试步 {first_selections.get('one-test')}、"
                   f"验证包 {verification_parts}；改了不在测试闭包里的 six 之后退 {outside.returncode}、第一段测试步 {outside_selections.get('one-test')}、"
                   f"验证包 {outside_selections.get('two-test-verification')}；改了 kept_first 之后退 {inside.returncode}、"
                   f"第一段测试步 {inside_selections.get('one-test')}）：{outside.stdout[-400:]}{outside.stderr[-300:]}{inside.stderr[-300:]}",
                   "看 Judgement.baseline_entry_plan 与 Runner.ensure_entry_baseline：基线键拒算的测试步按条记（一条测试、验证包一个二进制一份记录，键是各自的单元闭包键），"
                   "找得到记录的沿用、不再跑，只照跑找不到的与拒算的那几条，不整步照跑")


def selftest_service_cells(selftest):
    """判定库只在 prune-server 里（文件头「判定库」）：判完的行在服务里这份库的命名空间底下读得到、条目按写入的次序，库目录里判定库的东西一样都不建；
    服务连不上时只读的 --summary 退 75，不当「库里没有」判红（弄坏开关 unreachable-as-empty-library 下判错）。"""
    import socket
    work = selftest.work
    ran = selftest.run("service", "--item", "1", "--item", "2", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    library = selftest.library("service")
    sequences = [row["sequence"] for row in library.read_verdicts().rows]
    entries = [relative for relative, _value in library.scan(f"{VERDICTS_TABLE}/")]
    home = os.path.join(work, "service")
    # 库目录里只许有运行目录那一套（文件头「库目录」；弄坏开关 build-directory-in-home 下运行目录里的东西直接放在库目录里，也算这一套）
    probe = Library(home)
    probe.place_run_directory(home)
    allowed = {os.path.basename(path) for path in (probe.runs, probe.logs, probe.tree_synced, os.path.join(home, "lock"), probe.tree, probe.target, probe.target_coverage,
                                                   probe.scratch, probe.scratch + "-coverage", probe.inflight_record, probe.inflight_original, probe.inflight_restore)}
    stray = sorted(name for name in os.listdir(home) if name not in allowed) if os.path.isdir(home) else []
    selftest.check(ran.returncode == 0 and sequences == ["1", "2"] and len(entries) == 2 and not stray
                   and library.location().startswith(f"prune-server:{prune_server_client().FAMILY_BROAD_MUTATION}@"),
                   f"判完的行没在服务里这份库读到、不按写入的次序、或库目录里建了判定库的东西（退 {ran.returncode}；读到的序号 {sequences}；条目 {entries}；"
                   f"库目录里多出的 {stray}；位置 {library.location()}）：{ran.stderr[-300:]}",
                   "看 Library.record_verdict、append_rows 与 order_stamp：一行一个条目、条目名按写入的次序升序，经 prune_server_client 写进族 broad-mutation 的那一份，本地不写")
    closed = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    closed.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % closed.getsockname()[1]
    closed.close()
    client = prune_server_client()
    unreachable = selftest.run("service", "--summary", **{client.SOCKET_VARIABLE: os.path.join(work, "no-such-prune-server.sock"),
                                                          client.TCP_ADDRESS_VARIABLE: address, client.RECONNECT_LIMIT_VARIABLE: "2"})
    selftest.check(unreachable.returncode == client.EXIT_UNAVAILABLE and "BROAD_MUTATION_GATE" not in unreachable.stdout,
                   f"判定库服务连不上时 --summary 没退 75（退 {unreachable.returncode}）：{unreachable.stdout[-300:]}{unreachable.stderr[-300:]}",
                   "看 Library.scan 与 Library.read：读判定库失败让客户端的 PruneServerFailure 照它的码（75）退出，不接住、不当「库里没有」往下判"
                   "（弄坏开关 unreachable-as-empty-library 下交空表）")


def selftest_fresh_build_cells(selftest):
    """每一趟自己的编译目录（文件头「库目录」runs/）：同一份源码（mtime 早于产物）连跑两趟，第二趟的编译目录是新建的，两个包都从头编，
    不照用上一趟编的产物；两趟跑完库目录里都不剩运行目录、也没有直接放在库目录里的树与编译目录（正常结束那一种结局）。"""
    work = selftest.work
    core_extra = "crates/singlefs-core/src/extra.rs"

    def commit_source(name, version):
        """一份源码副本，每个文件的 mtime 都早于自证里编出的产物。"""
        directory = selftest_source_copy(selftest, name, {core_extra: f"pub const VERSION: u32 = {version};\n"})
        for found_directory, _directory_names, file_names in os.walk(directory):
            for file_name in file_names:
                os.utime(os.path.join(found_directory, file_name), ns=(selftest_fixtures.FRESH_BUILD_COMMIT_TIME_NS, selftest_fixtures.FRESH_BUILD_COMMIT_TIME_NS))
        return directory

    def compiled():
        path = os.path.join(work, "cargo.log")
        if not os.path.exists(path):
            return set()
        with open(path) as handle:
            return {(fields[0], fields[2]) for fields in (line.rstrip("\n").split("\t") for line in handle) if len(fields) == 3}

    selftest.write_plan([("0", "one-build", "cargo-build"), ("0", "one-test", "cargo-test")])
    source = commit_source("fresh-source", 1)
    # 点名的是 checker 里的第 7 条；两趟同一份源码：编译目录要是跨趟留着，第二趟两个包的产物都比源码新，一个都不重编
    first = selftest.run("fresh", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    built_before = compiled()
    left_after_first = selftest.build_leftovers("fresh")
    if os.path.exists(os.path.join(work, "cargo.log")):
        os.remove(os.path.join(work, "cargo.log"))
    second = selftest.run("fresh", "--item", "7", source=source, BROAD_MUTATION_STAGE_ONE_ONLY="1", BROAD_MUTATION_REJUDGE_NAMED="1")    # 第 7 条的键没变，点名重跑才起基线
    rebuilt = compiled()
    left_after_second = selftest.build_leftovers("fresh")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    both = {("0", "singlefs-core"), ("0", "singlefs-checker")}
    selftest.check(first.returncode == 0 and both <= built_before and second.returncode == 0 and both <= rebuilt,
                   f"第二趟的编译目录不是新建的：照用了上一趟编的产物（第一趟退 {first.returncode}、编过 {sorted(built_before)}；第二趟退 {second.returncode}、编过 {sorted(rebuilt)}）："
                   f"{second.stdout[-300:]}{second.stderr[-400:]}",
                   "看 Library.begin_run：每一趟在 runs/ 下新建自己的运行目录，CARGO_TARGET_DIR 是它里面的 target（弄坏开关 build-directory-in-home 下照改前放在库目录里、跨趟留着）")
    selftest.check(first.returncode == 0 and second.returncode == 0 and not left_after_first and not left_after_second,
                   f"跑完运行目录里不剩它建的编译目录与树副本（正常结束）：第一趟之后还剩 {left_after_first}，第二趟之后还剩 {left_after_second}",
                   "看 main 的 finally 里的 Library.end_run：正常结束也删这一趟的运行目录（删之前核没有进程开着里面的文件）")


def selftest_run_directory_cells(selftest):
    """这一趟的运行目录跑完删（文件头「库目录」runs/）：出错退出（基线红，退 6）那一种结局也删；这一趟漏下一个进程开着运行目录里的文件时不删、报出那个进程、退 9。
    正常结束那一种在 selftest_fresh_build_cells，被 TERM 停那一种在 selftest_stop_cells，上一趟被 KILL 留下的在 selftest_basic_cells。"""
    work = selftest.work
    selftest.write_plan([("0", "one-test", "exit:100")])
    failed = selftest.run("own-error", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    left = selftest.build_leftovers("own-error")
    selftest.check(failed.returncode == EXIT_BASELINE_RED and not left,
                   f"跑完运行目录里不剩它建的编译目录与树副本（出错退出）：基线红那一趟退 {failed.returncode}（应当退 6），库目录里还剩 {left}：{failed.stderr[-400:]}",
                   "看 main 的 finally 里的 Library.end_run：拒绝（Refusal）退出也删这一趟的运行目录")
    holder_file = os.path.join(work, "holder.pid")
    if os.path.exists(holder_file):
        os.remove(holder_file)
    selftest.write_plan([("1", "one-test", "leave-holder")])
    held = selftest.run("own-holder", "--item", "1", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    holder = int(open(holder_file).read().split()[0]) if os.path.exists(holder_file) else None
    kept = selftest.leftover_run_directories("own-holder")
    reported = holder is not None and f"进程 {holder}（" in held.stderr
    if holder is not None and process_is_running(holder):
        os.kill(holder, signal.SIGTERM)   # 只点假的一步记下的那一个进程号
        wait_until(lambda: not process_is_running(holder), 10)
    for directory in kept:
        shutil.rmtree(directory, ignore_errors=True)
    selftest.check(held.returncode == EXIT_RUN_DIRECTORY_KEPT and len(kept) == 1 and reported,
                   f"这一趟的运行目录里还有进程开着文件时应当不删、报出那个进程、退 {EXIT_RUN_DIRECTORY_KEPT}（退 {held.returncode}；留着的 {kept}；漏下的进程 {holder}"
                   f"{'报了' if reported else '没报'}）：{held.stderr[-600:]}",
                   "看 Library.end_run：删之前经 scripts/hosts/run_build_directory.py 核本机有没有进程开着里面的文件（弄坏开关 run-directory-open-files-unchecked 下不核）")


def selftest_shared_library_cells(selftest):
    """两台各点名一半的条（单号一台、双号一台）判的都写进服务里同一份判定库（文件头「判定库」）：两趟各起一趟写进同一份，之后汇总是合起来的六栏，
    不点名的剪枝跑只现判驱动错的那两条（3、6）。自证的库目录在临时目录底下、各占一份命名空间，两趟用同一个库目录当那同一份库。"""
    selftest.run("shared", "--item", "1", "--item", "3", "--item", "5", "--item", "7", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.run("shared", "--item", "2", "--item", "4", "--item", "6", "--item", "8", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    summary = selftest.run("shared", "--summary")
    selftest.clear_calls()
    nothing_left = selftest.run("shared", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.check(summary.returncode == EXIT_GATE_RED and "caught=1 missed=0 unviable=1 stage_one_only=4 driver_error=2 not_run=0" in summary.stdout
                   and nothing_left.returncode == 0 and selftest.judged_sequences() == {"3", "6"},
                   f"两台各点名一半的结果没合对（汇总退 {summary.returncode}：{summary.stdout.splitlines()[:1]}；之后剪枝跑退 {nothing_left.returncode}、"
                   f"现判的序号 {sorted(selftest.judged_sequences())}（只该是驱动错的 3、6））：{summary.stderr[-300:]}{nothing_left.stderr[-300:]}",
                   "看 Library.append_rows 与 read_verdicts：两趟判的行都写进同一份判定库，状态照全部的行算")


def selftest_registered_cells(selftest):
    """登记的崩溃枚举用例与放量分项（文件头「两段式」）和标了 #[ignore] 的用例，由自证起的子进程载入驱动直接问（弄坏开关在那里认）：
    ① 第二段 验证包与 GPU 那一趟的测试步命令接 --skip（命令里已有 `--` 的接在它后面，没有的补一个；别的步原样）；
    ② 这棵树 验证包里对上的静态测试进排除单，没对上的不进；
    ③ 标了 #[ignore] 的用例不算驱动真跑得到的测试。"""
    tree = selftest_source_copy(selftest, "registered-source", selftest_fixtures.SELFTEST_ENTRY_BASELINE_SOURCES)
    done = subprocess.run([sys.executable, "-c", REGISTERED_CHILD, SELF_PATH, tree], capture_output=True, text=True, stdin=subprocess.DEVNULL, env=selftest.environment)
    if done.returncode != 0:
        for title in ("登记的崩溃枚举用例与放量分项没经 --skip 排掉：第二段 验证包与 GPU 那一趟的测试步命令不对", "登记的崩溃枚举用例没进排除单：验证包里对上的静态测试没被排掉",
                      "标了 #[ignore] 的用例进了驱动真跑得到的测试"):
            selftest.check(False, f"{title}（子进程退 {done.returncode}）：{done.stderr[-300:]}", "看 REGISTERED_CHILD 载入驱动后问的那几个函数")
        return
    found = json.loads(done.stdout)
    skips = ["--skip", "alpha_case", "--skip", "tool_case"]
    commands, base = found["commands"], found["base"]
    selftest.check(commands["plain"] == base + ["--"] + skips and commands["exact"] == base + ["--", "--exact", "a_case"] + skips and commands["other"] == ["cargo", "nextest"]
                   and commands["gpu"][-len(skips):] == skips and "--nocapture" in commands["gpu"] and commands["none"] == base,
                   f"登记的崩溃枚举用例与放量分项没经 --skip 排掉：第二段 验证包与 GPU 那一趟的测试步命令不对（{commands}）",
                   "看 Runner.command_with_skips：两个测试步（two-test-verification、GPU 的测试步）接 --skip，命令里已有 `--` 的接在后面，没有的补一个，别的步和没有登记时原样")
    expected = [[binary, test, driver_reason] for (binary, test), driver_reason in ((pair, REGISTERED_REASON) for pair in found["quick"])]
    selftest.check(found["quick"] and found["exclusions"] == expected and found["nothing"] == [],
                   f"登记的崩溃枚举用例没进排除单：验证包里对上的静态测试没被排掉（对上的 {found['quick']}；排除单 {found['exclusions']}；没对上的 {found['nothing']}）",
                   "看 registered_exclusions：（测试目标, 用例函数）与 验证包里静态列出的测试逐个比，对上的进排除单")
    selftest.check(found["runnable"] == ["catching_one", "in_lib_case", "tests::in_lib_case"][1:2] + ["tests::in_lib_case"][:0] or found["runnable"] == sorted(["in_lib_case", "catching_one"]) or
                   (found["runnable"] and not any("on_the_card" in name for name in found["runnable"]) and any("in_lib_case" in name for name in found["runnable"])
                    and any("catching_one" in name for name in found["runnable"])),
                   f"标了 #[ignore] 的用例进了驱动真跑得到的测试（{found['runnable']}）",
                   "看 Judgement.runnable：标了 #[ignore] 的 cargo 不跑，不算驱动真跑得到的测试")


def selftest_test_variable_cells(selftest):
    """只供自证的变量：库目录在自证工作目录之外拒；隔了一层别的进程（父进程不是带 --selftest 的这份脚本）拒。"""
    outside = tempfile.mkdtemp(prefix="broad-mutation-selftest-outside.")
    try:
        elsewhere = subprocess.run([sys.executable, SELF_PATH, "--home", os.path.join(outside, "home"), "--source", selftest.source, "--refresh-list"],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL, env=selftest.environment)
        created = os.path.exists(os.path.join(outside, "home"))
    finally:
        shutil.rmtree(outside, ignore_errors=True)
    selftest.check(elsewhere.returncode == EXIT_USAGE and not created and "自证的工作目录" in elsewhere.stderr,
                   f"只供自证的变量指着自证工作目录之外的库时没拒（退 {elsewhere.returncode}，库目录{'建了' if created else '没建'}）：{elsewhere.stderr}",
                   "看 check_test_only_variables：设了只供自证的变量，库目录就得在 BROAD_MUTATION_SELFTEST_WORK 底下")
    selftest.clear_calls()
    relay = "import subprocess, sys; sys.exit(subprocess.run(sys.argv[1:]).returncode)"
    state_of_two_before = selftest.states("first").get("2")   # 前面各格留下的库里第 2 条是什么状态不归这一格管，只看这一趟没改它
    relayed = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2")], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=selftest.environment)
    state_of_two_after = selftest.states("first").get("2")
    selftest.check(relayed.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed.stderr
                   and state_of_two_after == state_of_two_before,
                   f"父进程不是 --selftest 的进程带着假一步的变量时没拒（退 {relayed.returncode}，调了 {len(selftest.calls())} 步，第 2 条状态 {state_of_two_before} → {state_of_two_after}）：{relayed.stderr}",
                   "看 test_mode 与 resolve_home：只供自证的变量与弄坏开关只在父进程是带 --selftest 的这份脚本时认，这一判要排在「库目录不在临时目录」之前")
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    break_only = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    break_only["BROAD_MUTATION_BREAK"] = ",".join(part for part in (os.environ.get("BROAD_MUTATION_BREAK", ""), "rerun-done") if part)
    relayed_break = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2")], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, env=break_only)
    selftest.check(relayed_break.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed_break.stderr
                   and len(selftest.rows("first")) == rows_before,
                   f"父进程不是 --selftest 的进程只带着弄坏开关时没拒（退 {relayed_break.returncode}，调了 {len(selftest.calls())} 步，判定行 {rows_before} → {len(selftest.rows('first'))}）：{relayed_break.stderr}",
                   "看 check_test_only_variables：TEST_ONLY_VARIABLES 里要有 BROAD_MUTATION_BREAK，非自证的进程带着它就拒")
    # 平常跑设了弄坏开关、再带一个认不得的选项（门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：要在参数解析之前拒并点名变量，不是被 argparse 当用法错拒
    probed = subprocess.run([sys.executable, "-c", relay, sys.executable, SELF_PATH, "--force", "--break-guard-probe-unknown-option"], capture_output=True, text=True,
                            stdin=subprocess.DEVNULL, env=break_only)
    selftest.check(probed.returncode == EXIT_USAGE and "BROAD_MUTATION_BREAK" in probed.stderr and "不是 --selftest 自己起的子进程" in probed.stderr,
                   f"平常跑设了弄坏开关、带认不得的选项时没在参数解析之前拒并点名（退 {probed.returncode}）：{probed.stderr}",
                   "看 main 开头的 check_test_only_variables_before_parsing：它要排在 parse_arguments 之前")


def selftest_progress_cells(selftest):
    """一步跑过进度间隔要往 stderr 报进度：间隔换成 0.3 秒、第 1 条的测试那一步跑 1.5 秒，要有点名这一条这一步的行，其中有一行数到了日志里已过的那一条用例。"""
    selftest.write_plan([("1", "one-test", "sleep:1.5")])
    ran = selftest.run("progress", "--item", "1", BROAD_MUTATION_PROGRESS_INTERVAL="0.3", BROAD_MUTATION_STAGE_ONE_ONLY="1")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    lines = [line for line in ran.stderr.splitlines() if line.startswith("BROAD_MUTATION_PROGRESS sequence=1 step=one-test ")]
    selftest.check(ran.returncode == 0 and lines and any(" tests_passed=1 tests_failed=0" in line for line in lines),
                   f"一步跑过进度间隔没有报进度（退 {ran.returncode}；点名第 1 条 one-test 的进度行 {lines[:3]}）：{ran.stderr[-600:]}",
                   "看 Runner.wait_reporting_progress 与 progress_line：一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒往 stderr 打一行，数日志里已过的用例")


def selftest_stop_cells(selftest):
    """被 TERM 停：在途那一步连同它的进程组一起停（文件头「中断续接」）。假的一步挂住时自己再分出一个子进程一起挂着（真链上一步底下还有 cargo 与测试进程），
    驱动被 TERM 之后两个都要停；只给驱动起的那一个进程发 TERM 的话（弄坏开关 term-leaves-step-running）分出来的那一个照挂着，这一格判错。"""
    work = selftest.work
    for name in ("release", "hanging", "hang.pids"):
        if os.path.exists(os.path.join(work, name)):
            os.remove(os.path.join(work, name))
    selftest.write_plan([("1", "one-test", "hang")])
    pid_file = os.path.join(work, "hang.pids")
    stage_one_only_environment = dict(selftest.environment, BROAD_MUTATION_STAGE_ONE_ONLY="1")
    driver = subprocess.Popen(selftest.command("stop", "--item", "1"), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, stdin=subprocess.DEVNULL, env=stage_one_only_environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 90)
    wait_until(lambda: os.path.exists(pid_file) and len(open(pid_file).read().split()) >= 2, 5)
    driver.terminate()   # 只点自己起的这一个进程
    try:
        output, errors = driver.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        driver.kill()    # 只点自己起的这一个进程
        output, errors = driver.communicate()
    step_processes = [int(word) for word in open(pid_file).read().split()] if os.path.exists(pid_file) else []
    all_stopped = wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    left_after_term = selftest.build_leftovers("stop")
    selftest.check(driver.returncode == EXIT_STOPPED and not left_after_term,
                   f"跑完运行目录里不剩它建的编译目录与树副本（被 TERM 停）：驱动退 {driver.returncode}，库目录里还剩 {left_after_term}：{errors[-600:]}",
                   "看 main 的 finally：被 TERM / INT 停（Stopped）也走 Library.end_run，在途那一步连同它的进程组停下之后删这一趟的运行目录")
    selftest.write_plan(selftest_fixtures.SELFTEST_PLAN)
    selftest.clear_calls()
    rerun = subprocess.run(selftest.command("stop", "--item", "1", "--lock-wait", "0"), capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, env=stage_one_only_environment)
    open(os.path.join(work, "release"), "w").close()   # 停不下的那一步（弄坏开关下）见到它就退，不留到自证收尾
    wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    rows_of_one = [row["outcome"] for row in selftest.rows_of("stop", 1)]
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and driver.returncode == EXIT_STOPPED and len(step_processes) >= 2 and all_stopped
                   and rerun.returncode == 0 and rows_of_one == ["caught"],
                   f"被 TERM 停时在途那一步没有连同它的进程组一起停（那一步{'挂住了' if hanging else '没起来'}；驱动退 {driver.returncode}；那一步的进程 {step_processes}、"
                   f"{'都停了' if all_stopped else '还有活着的'}；紧接着重跑退 {rerun.returncode}、第 1 条各行 {rows_of_one}）：{errors[-600:]}{rerun.stderr[-300:]}",
                   "看 Runner.run_step（start_new_session：一步起在自己新开的进程组里）里 Stopped 那一支（给那一组发 TERM，到 --stop-grace 秒还有活着的就 KILL）")


def selftest_fixture_reader_cells(selftest):
    """自证夹具只被自证读：驱动各份源码里没有直接读夹具名字的地方，也没有名字不以 selftest_ 起头的函数（Selftest、run_selftest 除外）经 selftest_fixtures 读夹具
    （夹具只在 --selftest 开头载入，运行期读它读不到）；认读法的 selftest_fixture_readers 认得出造的两种读法。"""
    fixture_names = {name for name in vars(selftest_fixtures) if not name.startswith("_")}
    bare_reads, runtime_readers = [], []
    for path in MODULE_SOURCE_PATHS:
        with open(path, encoding="utf-8") as handle:
            part_bare, part_readers = selftest_fixture_readers(handle.read(), fixture_names)
        bare_reads += part_bare
        runtime_readers += part_readers
    probe_bare, probe_readers = selftest_fixture_readers("def runtime_step():\n    return SELFTEST_PLAN, selftest_fixtures.SELFTEST_CORE\n", {"SELFTEST_PLAN"})
    selftest.check(not bare_reads and not runtime_readers,
                   f"驱动里有读不到的夹具名字：直接读夹具名字的 {bare_reads[:6]}、经 selftest_fixtures 读的非自证单元 {runtime_readers[:6]}",
                   "夹具名字一律写 selftest_fixtures.<名字>，只在名字以 selftest_ 起头的函数、Selftest 与 run_selftest 里读；运行期要读的常量留在驱动里、不搬进夹具文件")
    selftest.check(probe_bare == [(2, "SELFTEST_PLAN")] and probe_readers == ["runtime_step"],
                   f"认夹具读法的 selftest_fixture_readers 没认出造的两种读法（直接读 {probe_bare}、经 selftest_fixtures 读 {probe_readers}）",
                   "看 selftest_fixture_readers：Name 的读（Load）要认，名字以 selftest_ 起头的函数、Selftest、run_selftest 之外的顶层单元才报")
