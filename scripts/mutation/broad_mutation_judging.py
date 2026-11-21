"""broad-mutation.py 拆出的一份。
选条、沿用、结论分类（Judgement）、GPU 一趟的判法（GpuJudging）、起一条变异的步、读命令行与自证变量的小函数（refuse、say、note、test_mode、break_is 这类）。只 import base。驱动（这一份与别的份）的代码不进键（文件头「键」）。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import datetime
import hashlib
import importlib.util
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import time
import zoneinfo
from typing import NamedTuple
from broad_mutation_base import (CARGO_FAILED_EXIT, COMPILE_ERROR_LINE, COMPILE_ERROR_LOCATION, COULD_NOT_COMPILE_LINE, DECISIVE_OUTCOMES,
    DRIVER_ERROR, EXIT_DIRTY_TREE, EXIT_INCONSISTENT, EXIT_USAGE, GPU_ATTRIBUTE_START, GPU_BUILD_STEP, GPU_FEATURE, GPU_FEATURE_NAME,
    GPU_NOT_RUN, GPU_NO_ADAPTER_MARK, GPU_PASSED_COUNT, GPU_TESTS_DIRECTORY, GPU_TEST_STEP,
    GPU_VERDICTS_TABLE, GPU_VERDICT_COLUMNS, LIBTEST_DOCTESTS_LINE, LIBTEST_RUNNING_LINE,
    MUTATED_PACKAGES, MUTATION_MARKER, NEXTEST_FAILURE_LINE, NEXTEST_TEST_FAILED_EXIT, NOT_RUN, OUTCOME_TEXT, PER_ENTRY_TEST_STEPS, PRUNE_BREAKS,
    REAL_SELF_PATH, REGISTERED_REASON, REPOSITORY, SCRIPT_DIRECTORY, SELFTEST_ANCESTOR_DEPTH, SELF_PATH, STAGE_ONE_ONLY, STAGE_ONE_PACKAGES,
    UNRECOGNISED_TEST, VERIFICATION_PACKAGE, libtest_result_lines)


class Refusal(Exception):
    def __init__(self, message, howto, code=EXIT_USAGE):
        super().__init__(message)
        self.message, self.howto, self.code = message, howto, code


def refuse(message, howto, code=EXIT_USAGE):
    raise Refusal(message, howto, code)


def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


# ── 只供自证的变量与弄坏开关：只在 --selftest 自己起的子进程里生效 ─────────────────────────────────────────

def process_command_line(process_id):
    """一个进程的 (argv, 当前目录, 父进程号)；读不到交 None。"""
    try:
        with open(f"/proc/{process_id}/cmdline", "rb") as handle:
            argv = [word.decode("utf-8", "replace") for word in handle.read().split(b"\0") if word]
        directory = os.readlink(f"/proc/{process_id}/cwd")
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            parent = int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, ValueError, IndexError):
        return None
    return argv, directory, parent


def selftest_ancestor_distance():
    """往上第几层祖先是「这份脚本带 --selftest」：父进程是 1；往上 SELFTEST_ANCESTOR_DEPTH 层里没有交 None。"""
    process_id = os.getppid()
    for distance in range(1, SELFTEST_ANCESTOR_DEPTH + 1):
        found = process_command_line(process_id)
        if found is None:
            return None
        argv, directory, parent = found
        if "--selftest" in argv[1:] and any(os.path.realpath(os.path.join(directory, word)) == REAL_SELF_PATH for word in argv[1:]):
            return distance
        if parent <= 1:
            return None
        process_id = parent
    return None


def raw_break(name):
    return name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",")


TEST_MODE = None    # 第一次问的时候判一次；探针把驱动当模块载入时可以直接设成 True


def test_mode():
    """只供自证的变量与弄坏开关认不认：这个进程的父进程是带 --selftest 的这份脚本才认。"""
    global TEST_MODE
    if TEST_MODE is None:
        distance = selftest_ancestor_distance()
        TEST_MODE = distance == 1 or (distance is not None and raw_break("test-variables-anywhere"))
    return TEST_MODE


def test_variable(name):
    return os.environ.get(name) if test_mode() else None


def stage_two_skipped():
    """自证造现场用（只供测试，文件头「只供测试」）：BROAD_MUTATION_STAGE_ONE_ONLY=1 时每条只跑第一段、GPU 那一趟不起，第一段没红的落 stage-one-only，
    造出「第二段被打断的条」那种现场。命令行没有这样的参数：真跑一律两段跑完、再跑 GPU 那一趟。"""
    return test_variable("BROAD_MUTATION_STAGE_ONE_ONLY") == "1"


def named_items_rejudged():
    """自证造现场用（只供测试，文件头「只供测试」）：BROAD_MUTATION_REJUDGE_NAMED=1 时点名的条沿用得了也现判（GPU 那一趟也不读库），追加新行，旧行留着，
    复用哨兵不抽；造出「同一把键再判出另一个结局」那种现场。命令行没有这样的参数：真跑的强制现判只走复用哨兵。"""
    return test_variable("BROAD_MUTATION_REJUDGE_NAMED") == "1"


def break_is(name):
    return test_mode() and raw_break(name)


def kill_point(name):
    """自证造强杀的时刻：BROAD_MUTATION_KILL_AT 点到这里就给自己发 KILL。"""
    if test_variable("BROAD_MUTATION_KILL_AT") == name:
        os.kill(os.getpid(), signal.SIGKILL)


def now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())   # clock-times:allow 结果行的「时刻」一段，造时间戳的格式串


def file_digest(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def fsync_directory(directory):
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_atomically(path, data):
    """同目录临时文件写完、fsync、改名，再 fsync 目录。data 是 bytes。"""
    temporary = f"{path}.partial.{os.getpid()}"
    with open(temporary, "wb") as handle:
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    fsync_directory(os.path.dirname(path))


def toolchain_text(directory, environment=None):
    """这一刻的工具链：在 directory 里跑 rustc -Vv 的输出（rustup 按目录与 RUSTUP_TOOLCHAIN 这类变量挑工具链）；environment 给了就在它下面跑。"""
    fake = test_variable("BROAD_MUTATION_TOOLCHAIN")
    if fake is not None:
        return fake.strip() + "\n"
    if test_mode():
        return "selftest-toolchain\n"
    child_environment = None if environment is None else dict(environment)
    try:
        found = subprocess.run(["rustc", "-Vv"], capture_output=True, text=True, cwd=directory, stdin=subprocess.DEVNULL, env=child_environment)
    except OSError as error:
        refuse(f"起不了 rustc -Vv：{error}", "PATH 上要有 rustc（rustup 装的那一个）")
    if found.returncode != 0 or not found.stdout.strip():
        refuse(f"rustc -Vv 退 {found.returncode}：{found.stderr.strip()[-300:]}", f"在 {directory} 里手跑 rustc -Vv 看报错")
    return found.stdout.strip() + "\n"


class ItemState(NamedTuple):
    state: str               # 这一条的状态（ROW_OUTCOMES 之一或 NOT_RUN），判法见 state_of_outcomes
    passed_stage_one: bool   # 这一版里有它的 stage-one-only 行：第一段跑过而且没红


def state_of_outcomes(outcomes):
    """一条各行的结局（按写入的次序）→ ItemState。不取最后一行：
    caught / missed / unviable 取其中最后一行；没有它们而有 stage-one-only 的是它；只有 driver-error 的是它。"""
    passed_stage_one = STAGE_ONE_ONLY in outcomes
    if break_is("state-is-last-row"):
        return ItemState(outcomes[-1] if outcomes else NOT_RUN, passed_stage_one)
    decisive = [outcome for outcome in outcomes if outcome in DECISIVE_OUTCOMES]
    if decisive:
        return ItemState(decisive[-1], passed_stage_one)
    if passed_stage_one:
        return ItemState(STAGE_ONE_ONLY, True)
    if DRIVER_ERROR in outcomes:
        return ItemState(DRIVER_ERROR, False)
    return ItemState(NOT_RUN, False)


def today_text():
    """判定日期：只写日期（东京时区），不写时刻。"""
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")
PRUNE_MODULE = None


def prune_rules():
    global PRUNE_MODULE
    if PRUNE_MODULE is None:
        specification = importlib.util.spec_from_file_location("mutation_prune_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))
        PRUNE_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(PRUNE_MODULE)
        PRUNE_MODULE.BREAKS = frozenset(name for name in PRUNE_BREAKS if break_is(name))
    return PRUNE_MODULE


def global_lines_of(exclusions_digest, root, environment, toolchain, tools):
    """全局那几行（mutation-prune.py 的 global_lines，平常是空的）；弄坏开关下才读的配置读不了抛 Refusal（退 2）。"""
    try:
        return prune_rules().global_lines(exclusions_digest, root, environment, toolchain, tools)
    except ValueError as error:
        refuse(f"算不出构建环境：{error}", "看报错里点的那份 cargo 配置或 Cargo.toml：要读得了、解析得了")


def verdict_state(outcomes):
    """一条在判定库里键相同、沿用判据认的那几行的结局（按写入的次序）→ ItemState（文件头「判定行」末段）。"""
    return state_of_outcomes(outcomes)


class Judgement:
    """一棵树上的判定：单元（UnitTree）、全局那几行、每条的键、判定库里键相同的行、沿用与状态。"""

    def __init__(self, library, root, arguments, for_run, registered=(), toolchain=None, tools=None):
        self.rules, self.library, self.root, self.arguments, self.for_run = prune_rules(), library, root, arguments, for_run
        try:
            self.tree = self.rules.UnitTree(root)
        except Exception as error:  # lib_function_units 起不来：根不在、清单读不出
            refuse(f"在 {root} 上切不出单元（scripts/gate/lib_function_units.py）：{error}", "看报错：Cargo.toml 要读得了；lib_function_units.py 坏了就先修它", EXIT_INCONSISTENT)
        for path in self.tree.files:
            self.tree.text(path)    # 先把原文都读进来：变异改树的时候算测试键、覆盖者键读的还是没改过的
        self.registered = set(registered)
        self.exclusions = registered_exclusions(self.rules, self.tree, self.registered)
        build_root = library.tree if (not for_run and os.path.isdir(library.tree)) else root    # 弄坏开关下的构建环境与 rustc -Vv 在驱动编译的那一处算
        # rustc -Vv 只供 global_lines 的弄坏开关 toolchain-version-keyed：平常不起 rustc、不进键
        self.toolchain = (toolchain if toolchain is not None else toolchain_text(build_root)) if break_is("toolchain-version-keyed") else ""
        self.tools = tools if tools is not None else tool_versions(arguments)
        exclusions_digest = hashlib.sha256("".join(f"{binary} {test}\n" for binary, test, _reason in self.exclusions).encode()).hexdigest()
        self.globals = global_lines_of(exclusions_digest, build_root, os.environ, self.toolchain, self.tools)
        self.mutants_version = next((line for line in self.tools if "mutants" in line), " | ".join(self.tools))
        self.mutants_configuration = self.rules.mutants_configuration_digest(root)
        self.keys, self._tests = {}, None
        self.gpu_decided = set()    # 真跑选条前由 gpu_decided_keys 填：键在里面的条，CPU 一趟的没红行不算红（要 GPU 趟、GPU 趟的真实状态不是没红），照沿用
        self.release_closures = {}   # 带 --release 那一步的判定的闭包（release_entry_key），按（单元, 目标）缓存
        self.table = library.read_verdicts()
        self.by_key, self.by_locator = {}, {}
        for row in self.table.rows:
            self.by_key.setdefault(row["key"], []).append(row)
            self.by_locator.setdefault(row["locator"], []).append(row)

    def package_files(self, packages):
        found = []
        for path in sorted(self.tree.files):
            crate = self.tree.workspace.crate_of_path(path)
            if crate is not None and crate.package in packages:
                found.append(path)
        return found

    def key_of(self, item):
        identity = (item["file"], tuple(item["start"]), tuple(item["end"]), item["replacement"])
        if identity not in self.keys:
            self.keys[identity] = self.rules.mutation_key(self.tree, item, self.globals)
        return self.keys[identity]

    def runnable(self, packages):
        """这几个包里驱动真跑得到的静态测试：标了 #[ignore] 的 cargo 不跑，不算；登记的崩溃枚举用例与放量分项（self.exclusions）经 --skip 排掉，也不算。
        弄坏开关 attribute-skipped-run 下标了 #[ignore] 的也算。"""
        return [entry for entry in self.rules.test_entries(self.tree, packages, self.exclusions)
                if not entry["skipped_by_attribute"] or break_is("attribute-skipped-run")]

    def tests(self):
        """静态列出的测试（覆盖者键用）：第一段（那三个包）、验证包、被变异包的文档测试；登记的崩溃枚举用例与放量分项不算。"""
        if self._tests is None:
            self._tests = self.runnable((*STAGE_ONE_PACKAGES, VERIFICATION_PACKAGE)) + self.rules.doctest_entries(self.tree, MUTATED_PACKAGES)
        return self._tests

    def test_key_for(self, caught_by):
        return self.rules.catching_test_key(self.tree, caught_by, self.globals, self.exclusions)

    def coverers_for(self, unit):
        if unit is None:
            return None, 0, "no-unit"
        return self.rules.coverers_key(self.tree, unit, self.tests(), self.globals)

    def row_holds(self, row, current):
        """判定库里键相同的这一行此刻还认不认（沿用判据）：→ (认不认, 为什么不认)。"""
        outcome = row["outcome"]
        if outcome not in DECISIVE_OUTCOMES or break_is("carry-without-store-check"):
            return True, ""
        unresolved = row["refused"].startswith("catching-test-unresolved")
        if outcome == "caught" and unresolved:
            if self.for_run and not break_is("refused-carried"):
                return False, row["refused"]
            coverers, _count, why = self.coverers_for(current["unit"])
            return (True, "") if coverers and coverers == row["coverage_key"] else (False, why or "coverers-changed")
        test_now, test_why, coverers_now, coverers_why = "", "", "", ""
        if outcome == "caught":
            test_now, test_why = self.test_key_for(row["caught_by"])
        if outcome == "missed":
            coverers_now, _count, coverers_why = self.coverers_for(current["unit"])
        current_view = dict(current, refused=[] if (not self.for_run or break_is("refused-carried")) else current["refused"])
        return self.rules.carry_decision(row, current_view, test_now or "", test_why, coverers_now or "", coverers_why)

    def state(self, item):
        """→ (ItemState, 为什么沿用不了, 定下状态的那一行)。真跑时拒算的条一律沿用不了（每趟现判）。"""
        current = self.key_of(item)
        if current["key"] is None:
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        if self.for_run and current["refused"] and not break_is("refused-carried"):
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        rows = self.by_key.get(current["key"], [])
        kept, reasons = [], []
        for row in rows:
            holds, why = self.row_holds(row, current)
            (kept if holds else reasons).append(row if holds else why)
        state = verdict_state([row["outcome"] for row in kept])
        deciding = next((row for row in reversed(kept) if row["outcome"] == state.state), None)
        reason = reasons[-1] if reasons else ("no-record" if not rows else "")
        if self.for_run and state.state in DECISIVE_OUTCOMES and not break_is("missed-carried") and current["key"] not in self.gpu_decided:
            # 记红、撤绿、下趟现判（文件头「判定行」末段）：本来沿用得了（有定论）的条，同一把键的定论里最后一个是没红的，红照键记着、前面的好结论撤掉，这一条不沿用、现判；
            # 看全部键相同的行，不看沿用判据认不认：认不认只定 --summary 的状态，撤绿不看覆盖者键变没变（没红行本身已不算数的，状态不是定论，走原来的路：只补第二段）
            decisive = [row["outcome"] for row in rows if row["outcome"] in DECISIVE_OUTCOMES]
            if decisive and decisive[-1] == "missed":
                return ItemState(NOT_RUN, state.passed_stage_one), "missed-rejudged", None
        return state, reason, deciding

    def changed_units(self, item, limit=5):
        """沿用不了时变了的是哪几个单元：同一处变异（位置索引相同）上一回判的那把键的组成清单与此刻的比，列出名字（文件头「库目录」compositions/）。"""
        current = self.key_of(item)
        previous = [row for row in self.by_locator.get(current["locator"], []) if row["key"] != current["key"]]
        if not previous:
            return ""
        old = self.library.read_composition(previous[-1]["key"])
        if old is None:
            return ""
        return composition_difference(old, current["composition"], limit)

    def refused_text(self, item, deciding=None):
        current = self.key_of(item)
        forms = list(current["refused"])
        if deciding is not None and deciding["refused"]:
            forms.append(deciding["refused"])
        return ",".join(forms)

    def coverage_jobs(self, stage):
        """覆盖那一步的各条（静态列出的）：{(二进制, 用例): 记录键或 None}。第一段逐条，记录键是那条测试的测试键；
        验证包逐个二进制（用例写 *），记录键取那个二进制里各条测试的测试键（有一条算不出就是 None，每趟现跑）。"""
        jobs = {}
        if stage == "one":
            for entry in self.runnable(STAGE_ONE_PACKAGES):
                key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
                jobs[(entry["binary"], entry["test"])] = hashlib.sha256(f"coverage-v1\none\n{key}\n".encode()).hexdigest() if key else None
            return jobs
        by_binary = {}
        for entry in self.runnable((VERIFICATION_PACKAGE,)):
            key, _forms = self.rules.test_key(self.tree, entry["unit"], entry["target"], self.globals)
            target = entry["target"].split(":", 2)
            label = "lib" if target[1] == "lib" else target[2]
            by_binary.setdefault(f"verification:{label}", []).append(key)
        for binary, keys in by_binary.items():
            jobs[(binary, "*")] = (hashlib.sha256(("coverage-v1\nverification\n" + "".join(f"{key}\n" for key in sorted(keys))).encode()).hexdigest()
                                   if all(keys) else None)
        return jobs

    def selection(self, item, runner):
        """C2 / E1 / E6：这条变异每一步跑哪些测试。{"one": None 全部 / [(二进制, 用例)], "verification": None / [目标], "reason": …}。"""
        current = self.key_of(item)
        everything = self.rules.run_everything_reason(self.tree, current)
        if everything or break_is("coverage-selection-dropped") or "one" not in runner.coverage_available:
            return {"one": None, "verification": None, "reason": everything or "no-coverage"}
        unit = current["unit"]
        fingerprint = f"{unit.kind}:{unit.fingerprint}"
        spawning = self.rules.spawning_binaries(self.tree, STAGE_ONE_PACKAGES, runner.profile_counts)
        chosen = {"one": self.rules.select_tests(runner.coverage["one"], fingerprint, spawning), "reason": "coverage"}
        verification_spawning = {binary.split("::", 1)[1] if "::" in binary else "lib" for binary in self.rules.spawning_binaries(self.tree, (VERIFICATION_PACKAGE,))}
        if "two" not in runner.coverage_available or break_is("stage-two-not-by-binary"):
            chosen["verification"] = None
            return chosen
        chosen["verification"] = sorted({job[0][len("verification:"):] for job, covered in runner.coverage["verification"].items() if fingerprint in covered} | verification_spawning)
        return chosen

    def baseline_key(self, step, threads):
        """基线键（文件头「基线」）：→ (键, "") 或 (None, 拒算的那一句)。键 = 这一步真跑到的每一条判定（baseline_entries）各自单元闭包键的多重集
        （闭包键里有闭包单元的种类与去注释指纹、闭包真引到的 cfg 原子、全局那几行与 edition）、去掉线程数的命令、全局那几行；
        不带步名、手写的版本号、包级与文件级指纹。在这一步的 cfg 与特性下不编的判定（entry-cfg-inactive）不算这一步跑到的、不进键。
        拒算（照跑、不沿用、不退回包级）：编译步（编得过与否、用时随包里每一个单元变，按单元闭包切不开）、认不出跑哪些判定的步、有一条判定的闭包拒算。
        弄坏开关 baseline-by-packages：取这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹（包级），带手写的 form 与步名。"""
        name, _stage, kind, command, _failed = step
        if break_is("baseline-by-packages"):
            return self.package_baseline_key(step, threads), ""
        if kind == "build":
            return None, "编译步：编得过与否、用时随这一步编的那几个包里每一个单元变，按单元闭包切不开"
        entries, release = self.baseline_entries(name)
        if entries is None:
            return None, f"认不出步 {name} 跑哪些判定"
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        global_lines = list(self.globals)
        keys = []
        for entry in entries:
            key, refused = self.release_entry_key(entry, global_lines) if release else self.rules.test_key(self.tree, entry["unit"], entry["target"], global_lines)
            if key is None and refused == ["entry-cfg-inactive"]:
                continue
            if key is None:
                return None, f"{entry['binary']} {entry['test']} 的单元闭包拒算（{','.join(refused) or '闭包算不出'}）"
            keys.append(key)
        lines = [f"command\t{shlex.join(words)}"] + global_lines + [f"entry\t{key}" for key in sorted(keys)]
        return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest(), ""

    def baseline_entry_plan(self, step, threads):
        """按条记基线（文件头「基线」）：基线键因为有判定的闭包拒算而整步算不出的测试步，拆成各自按键沿用的几份记录与每趟照跑的那几条。
        → None（不按条：编译步、认不出的步、不在 PER_ENTRY_TEST_STEPS 里的步，弄坏开关 baseline-refused-step-whole）或
        {"style": nextest / libtest, "records": [记录], "refused": [(二进制, 用例)], "whole": [整个照跑的二进制]}。
        nextest 那两步：一条测试一份记录（kind test，键 = 去掉线程数的命令、全局那几行、它自己的单元闭包键）；有条目级宏调用的二进制另一份
        「宏调用展开的那几条」（kind rest，键 = 那个二进制里宏调用的闭包键多重集），宏调用的闭包拒算的二进制整个照跑；拒算的测试每趟照跑。
        验证包（cargo test）：一个二进制一份记录（kind binary，键 = 它里面闭包算得出的判定与宏调用的闭包键多重集；
        有拒算的，这一份记的是 --exact --skip 掉它们的那一趟），拒算的那几条另起 --exact 照跑；宏调用拒算的二进制整个照跑。
        记录里的二进制、名字只当索引，不进键（rest 另按二进制找，改名只会重跑、不会错沿用）。"""
        name, _stage, kind, command, _failed = step
        if kind != "test" or name not in PER_ENTRY_TEST_STEPS or break_is("baseline-refused-step-whole"):
            return None
        entries, release = self.baseline_entries(name)
        if entries is None:
            return None
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        global_lines = list(self.globals)
        head = [f"command\t{shlex.join(words)}"] + global_lines
        style = "nextest" if "nextest" in command else "libtest"

        def digest(lines):
            return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()

        groups = {}
        for entry in entries:
            target = self.tree.workspace.targets.get(entry["target"]) if entry["target"] else None
            group = groups.setdefault(entry["binary"], {"target": target, "tests": [], "refused": [], "macros": [], "macro_refused": False})
            key, refused = self.release_entry_key(entry, global_lines) if release else self.rules.test_key(self.tree, entry["unit"], entry["target"], global_lines)
            is_macro = entry["unit"].kind == "item_macro"
            if key is None and refused == ["entry-cfg-inactive"]:
                continue
            if is_macro:
                group["macros"].append(key)
                group["macro_refused"] = group["macro_refused"] or key is None
            elif key is None:
                group["refused"].append(entry["test"])
            else:
                group["tests"].append((entry["test"], key))
        plan = {"style": style, "records": [], "refused": [], "whole": []}
        for binary, group in sorted(groups.items()):
            target = group["target"]
            selector = (["--lib"] if target is None or target.kind == "lib" else [f"--{target.kind}", target.name])
            plan["refused"] += [(binary, test) for test in group["refused"]]
            if style == "nextest":
                plan["records"] += [{"kind": "test", "binary": binary, "test": test, "key": digest(head + [f"entry\t{key}"])}
                                    for test, key in group["tests"]]
                if group["macro_refused"]:
                    plan["whole"].append({"binary": binary, "selector": selector})
                elif group["macros"]:
                    plan["records"].append({"kind": "rest", "binary": binary, "test": "",
                                            "key": digest(head + ["rest"] + [f"macro\t{key}" for key in sorted(group["macros"])])})
                continue
            if group["macro_refused"]:
                plan["whole"].append({"binary": binary, "selector": selector})
                continue
            plan["records"].append({"kind": "binary", "binary": binary, "test": "*", "selector": selector, "refused": sorted(group["refused"]),
                                    "key": digest(head + [f"entry\t{key}" for key in sorted(key for _test, key in group["tests"])]
                                                  + [f"macro\t{key}" for key in sorted(group["macros"])])})
        return plan

    def baseline_entries(self, name):
        """一步真跑到的判定：→ (判定列表, 按不按 --release 的 cfg 收闭包)；认不出的步交 (None, False)。测试函数照 mutation-prune.py 的 test_entries 静态列，
        另加这几个目标里的条目级宏调用（宏展开出来的测试静态列不出：把宏调用连同它的闭包算进去）；文档测试照 doctest_entries。"""
        if name == "one-test":
            return self.runnable(STAGE_ONE_PACKAGES) + self.macro_entries(STAGE_ONE_PACKAGES), False
        if name == "two-test-doc":
            return self.rules.doctest_entries(self.tree, MUTATED_PACKAGES), False
        if name == "two-test-verification":
            return self.runnable((VERIFICATION_PACKAGE,)) + self.macro_entries((VERIFICATION_PACKAGE,)), True
        return None, False

    def macro_entries(self, packages):
        """这几个包的 lib、bin、tests 目标里的条目级宏调用（item_macro 单元），照判定的写法交（二进制、名字只当报错的索引）。"""
        found = []
        for target in self.tree.workspace.targets.values():
            if target.crate.package not in packages or target.kind not in ("lib", "test", "bin"):
                continue
            for path in sorted(target.files):
                found += [{"binary": self.rules.target_binary(target), "test": unit.name, "unit": unit, "target": target.id}
                          for unit in self.tree.code_units(path) if unit.kind == "item_macro"]
        return found

    def release_entry_key(self, entry, global_lines):
        """带 --release 的那一步（验证包）的一条判定：cfg 去掉 debug_assertions 收闭包（试法照 mutation-prune.py 的 UnitTree.closure：
        先照默认、不编再加 test），键照 UnitTree.closure_key 的组法（闭包键、全局那几行与 edition）。→ (键或 None, 拒算形态)。"""
        library, unit, target_id = self.tree.library, entry["unit"], entry["target"]
        cache_key = (id(unit), target_id)
        if cache_key not in self.release_closures:
            release_cfg = [item for item in library.DEFAULT_RUST_CFG if item != "debug_assertions"]
            found = (None, ["entry-cfg-inactive"])
            for extra in ((), ("test",)):
                try:
                    found = (library.rust_closure_of(self.tree.workspace, unit, target_id, release_cfg + list(extra), no_default_cfg=True), [])
                    break
                except library.UsageError:
                    found = (None, ["entry-cfg-inactive"])
            self.release_closures[cache_key] = found
        closure, refusals = self.release_closures[cache_key]
        if closure is None:
            return None, refusals
        dependencies, dependency_refusals = self.tree.dependency_lines(closure)
        key, closure_refusals = library.closure_key(closure, toolchain=list(global_lines) + self.tree.edition_lines(closure), dependencies=dependencies,
                                                    allow_refused=True)
        forms = sorted({refusal.form for refusal in closure_refusals}) + dependency_refusals
        return (None, forms) if forms else (key, [])

    def package_baseline_key(self, step, threads):
        """包级的基线键（只给弄坏开关 baseline-by-packages）：这一步编、跑的那几个包（连同路径依赖）里全部单元的指纹多重集、去掉线程数的命令、
        全局那几行，带手写的 form 与步名。"""
        name, _stage, _kind, command, _failed = step
        packages = set(MUTATED_PACKAGES if name == "two-test-doc" else (VERIFICATION_PACKAGE,) if "verification" in name else STAGE_ONE_PACKAGES)
        crates = {crate.directory: crate for crate in self.tree.workspace.crates if crate.package in packages}
        pending = list(crates.values())
        while pending:
            crate = pending.pop()
            for section in ("normal", "dev", "build"):
                for dependency in crate.dependencies[section]:
                    if dependency.directory and dependency.directory not in crates and dependency.directory in self.tree.workspace.crate_by_directory:
                        crates[dependency.directory] = self.tree.workspace.crate_by_directory[dependency.directory]
                        pending.append(crates[dependency.directory])
        words = list(command)
        if "-j" in words and words.index("-j") + 1 < len(words) and words[words.index("-j") + 1] == str(threads):
            del words[words.index("-j"):words.index("-j") + 2]
        units = sorted(f"{unit.kind}:{unit.fingerprint}" for path in self.package_files({crate.package for crate in crates.values()})
                       for unit in self.tree.code_units(path))
        lines = ["form\tbroad-mutation-baseline-v1", f"step\t{name}", f"command\t{shlex.join(words)}"]
        lines += list(self.globals) + [f"unit\t{unit}" for unit in units]
        return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


def composition_difference(old, new, limit=5):
    """两份组成清单之间变了的单元：→「改了 <限定名>（文件）、新进 …、出了 …」；只比单元行（种类、限定名、文件:起止、指纹）。"""
    def units(lines):
        found = {}
        for line in lines:
            fields = line.split("\t")
            if len(fields) == 4 and ":" in fields[2]:
                found.setdefault((fields[0], fields[1], fields[2].rsplit(":", 1)[0]), []).append(fields[3])
        return found
    before, after = units(old), units(new)
    changed = [f"改了 {name}（{path}）" for kind, name, path in sorted(set(before) & set(after)) if sorted(before[(kind, name, path)]) != sorted(after[(kind, name, path)])]
    added = [f"新进 {name}（{path}）" for kind, name, path in sorted(set(after) - set(before))]
    dropped = [f"出了 {name}（{path}）" for kind, name, path in sorted(set(before) - set(after))]
    others = [] if changed or added or dropped else [f"{line.split(chr(9), 1)[0]} 那几行" for line in sorted(set(old) ^ set(new)) if not line.startswith(("unit\t",))][:2]
    found = changed + added + dropped + others
    return "、".join(found[:limit]) + (f" 等 {len(found)} 处" if len(found) > limit else "")


# ── 清单的区间与 diff ───────────────────────────────────────────────────────────────────────────────────

def offset_of(text_lines, line, column):
    """1 起的行、列（按字符数）在整份文本里的下标；text_lines 是 text.split('\\n')。"""
    return sum(len(text_line) + 1 for text_line in text_lines[:line - 1]) + column - 1


def replace_span(text, start, end, replacement):
    """把 [start, end)（各是 (行, 列)，1 起、列按字符数、end 不含）换成 replacement。"""
    lines = text.split("\n")
    start_offset, end_offset = offset_of(lines, *start), offset_of(lines, *end)
    if not 0 <= start_offset <= end_offset <= len(text):
        raise ValueError(f"区间 {start}–{end} 越出文件")
    return text[:start_offset] + replacement + text[end_offset:]


def registered_exclusions(rules, tree, registered):
    """上面那一张单在这棵树的 验证包里对上的静态测试：[(二进制 id, 用例名, 理由)]；覆盖者键、基线键与测试清单都不算它们（它们不在这里跑）。
    弄坏开关 registered-exclusion-dropped 下一条都不排（排除单是空的）。"""
    found = []
    if break_is("registered-exclusion-dropped"):
        return found
    for entry in rules.test_entries(tree, (VERIFICATION_PACKAGE,)):
        if (entry["binary"].rsplit("::", 1)[-1], entry["test"].rsplit("::", 1)[-1]) in registered:
            found.append((entry["binary"], entry["test"], REGISTERED_REASON))
    return found


def libtest_failed_name(line):
    """libtest 结果行结局是 FAILED 的全名（` - should panic` 那一段不进名字），别的交 None。弄坏开关 own-libtest-failure-line 下用另一条正则（名字带上 ` - should panic`）。"""
    if break_is("own-libtest-failure-line"):
        found = re.match(r"^test (\S.*?) \.\.\. FAILED\s*$", line)
        return found.group(1) if found else None
    found = libtest_result_lines.result_line(line)
    return found.full_name if found and found.outcome == "FAILED" else None


def nextest_failed_name(line):
    found = NEXTEST_FAILURE_LINE.match(line)
    return found.group(1) if found else None


def log_shows_compile_error(log_path):
    """编译那一步的日志里有没有真的编译错误：一条带源码位置的 rustc 错误（error 行之后两行内有 `--> 文件:行:列`），又有 cargo 的「could not compile」。
    磁盘满、锁文件不符、链接器被杀这一类也让 cargo 退 101，但没有带源码位置的错误。"""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return False
    located = any(COMPILE_ERROR_LINE.match(line) and any(COMPILE_ERROR_LOCATION.match(later) for later in lines[index + 1:index + 3])
                  for index, line in enumerate(lines))
    return located and any(COULD_NOT_COMPILE_LINE.match(line) for line in lines)


def tool_versions(arguments):
    """全局键里的工具版本：nextest、cargo-mutants、llvm-tools（llvm-profdata --version）。"""
    fake = test_variable("BROAD_MUTATION_TOOL_VERSIONS")
    if fake is not None or test_mode():
        return [fake or "selftest-tools"]
    found = []
    for command in (["cargo", "nextest", "--version"], ["cargo", "mutants", "--version"], [os.path.join(llvm_bin(arguments), "llvm-profdata"), "--version"]):
        try:
            done = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL)
            found.append(" ".join(done.stdout.split()[:6]) or f"{command[-2]} 退 {done.returncode}")
        except OSError:
            found.append(f"{command[0]} 不在")
    return found


def llvm_bin(arguments):
    """llvm-profdata、llvm-cov 在哪：--llvm-bin 给的，没给取 rustc 的 sysroot 下 rustup 组件 llvm-tools 的 bin（与编译器同一份 LLVM，profraw 格式对得上）。"""
    if arguments.llvm_bin:
        return arguments.llvm_bin
    try:
        sysroot = subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
        host = next(line.split(":", 1)[1].strip() for line in toolchain_text(REPOSITORY).splitlines() if line.startswith("host:"))
    except (OSError, StopIteration):
        return ""
    return os.path.join(sysroot, "lib", "rustlib", host, "bin")


def libtest_binary(line, current, package_of_step):
    """libtest 那几步的日志里认出接下来的测试属于哪个二进制：Running <源文件> (<可执行文件>) 与 Doc-tests <crate> 两种行；认不出交 current。"""
    running = LIBTEST_RUNNING_LINE.match(line)
    if running and package_of_step:
        source = running.group("source")
        if source.startswith("src/lib.rs") or source == "src/lib.rs":
            return package_of_step
        name = os.path.basename(source)[:-3] if source.endswith(".rs") else source
        return f"{package_of_step}::bin/{name}" if source.startswith("src/") else f"{package_of_step}::{name}"
    doctests = LIBTEST_DOCTESTS_LINE.match(line)
    if doctests:
        return doctests.group("crate").replace("_", "-") + "::doctest"
    return current


def first_failing_test(log_path, test_failed_exit, package_of_step=""):
    """这一步第一条没过的测试：nextest 的「二进制 id 用例名」原样；libtest 的照前面最近的 Running / Doc-tests 行补上二进制 id（认不出就只有用例名）。"""
    nextest = test_failed_exit == NEXTEST_TEST_FAILED_EXIT
    binary = ""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.rstrip("\n")
                if nextest:
                    name = nextest_failed_name(line)
                    if name:
                        return name
                    continue
                binary = libtest_binary(line, binary, package_of_step)
                name = libtest_failed_name(line)
                if name:
                    return f"{binary} {name}" if binary else name
    except OSError:
        pass
    return UNRECOGNISED_TEST


def apply_mutation(library, item):
    """就地改树里的一个文件：先存原件、记在途记录，再写改后的内容。"""
    target = os.path.join(library.tree, item["file"])
    original_digest = (library.manifest or {}).get(item["file"])
    if original_digest is None or file_digest(target) != original_digest:
        refuse(f"要改的 {item['file']} 此刻与这一趟换进来的源码对不上：上一条没恢复干净",
               "重跑同一条命令（开跑时照在途记录恢复、再照源码换树）", EXIT_DIRTY_TREE)
    with open(target, encoding="utf-8") as handle:
        original = handle.read()
    mutated = replace_span(original, tuple(item["start"]), tuple(item["end"]), item["replacement"] + MUTATION_MARKER)
    if mutated == original:
        refuse(f"第 {item['sequence']} 条改完与原文相同，没有命中", "清单与专用树对不上：重跑同一条命令（开跑时重新换树、重新生成清单）", EXIT_INCONSISTENT)
    shutil.copyfile(target, library.inflight_original)
    with open(library.inflight_original, "rb") as handle:
        os.fsync(handle.fileno())
    write_atomically(library.inflight_record, json.dumps(
        {"sequence": item["sequence"], "file": item["file"], "original_digest": original_digest, "started_at": now_text()}).encode())
    with open(target, "w", encoding="utf-8") as handle:
        handle.write(mutated)
    if file_digest(target) == original_digest:
        refuse(f"第 {item['sequence']} 条写进 {target} 之后回读还是原文", "看树所在的盘是不是满了或只读")


def restore_original(library):
    """照在途记录把原件拷回树里、核 sha256、删掉在途记录：交回那份在途记录，没有在途记录交 None。临时件建在库目录里，不进树。
    原件不在或对不上：不拷，删掉在途记录、说一句（开跑的换树照源码把那份文件重写回去）。"""
    if not os.path.exists(library.inflight_record):
        if os.path.exists(library.inflight_original):
            os.remove(library.inflight_original)   # 存了原件、还没记在途记录就停了：树没动过
        return None
    with open(library.inflight_record, encoding="utf-8") as handle:
        record = json.load(handle)
    target = os.path.join(library.tree, record["file"])
    if not os.path.isfile(library.inflight_original) or file_digest(library.inflight_original) != record["original_digest"]:
        note(f"  · 在途记录说 {record['file']} 被第 {record['sequence']} 条改过，但存下的原件不在或对不上：不拷，换树时照源码重写那份文件")
        for path in (library.inflight_record, library.inflight_original):
            if os.path.exists(path):
                os.remove(path)
        return record
    shutil.copyfile(library.inflight_original, library.inflight_restore)
    kill_point("restore-after-copy")
    os.replace(library.inflight_restore, target)
    if file_digest(target) != record["original_digest"]:
        refuse(f"把原件拷回 {target} 之后 sha256 还是对不上", "看树所在的盘是不是满了或只读；修好之后重跑同一条命令（换树照源码重写那份文件）", EXIT_DIRTY_TREE)
    os.remove(library.inflight_record)
    os.remove(library.inflight_original)
    fsync_directory(library.home)
    return record


class ItemRow(NamedTuple):
    fields: list            # 落下的那一行（十六段）
    step_name: str          # 定下结局的那一步（第二段全绿的 missed 是 ""）
    step_seconds: float     # 那一步的秒数
    environment_doubt: str  # 不空：这一条的结局像是环境造成的（编译步退 101 而没有编译错误），调用方当场在原件上验一次


def process_item(library, runner, item, with_stage_one, with_stage_two):
    """判一条：交回这一条落下的最后一行（ItemRow）。中途被停（Stopped）不落行，原件照样拷回。"""
    sequence = item["sequence"]
    judgement = runner.judgement
    current = judgement.key_of(item)
    item["verdict_key"] = current["key"] or ""
    totals = {"build": 0.0, "test": 0.0}
    selection = judgement.selection(item, runner)
    ran_any_test = []
    method = "fresh"

    def record(outcome, stage, test_name="", note_text="", step_name="", step_seconds=0.0, environment_doubt=""):
        if outcome == "missed" and not ran_any_test:
            note_text = (note_text + "；" if note_text else "") + "零覆盖：没有一条测试执行到它（C2 记没红）"
        refused = list(current["refused"])
        test_key = coverage_key = ""
        if outcome == "caught":
            test_key, why = judgement.test_key_for(test_name)
            test_key = test_key or ""
            if not test_key:
                # 抓到它的测试认不出（文件头「键」）：这一条每趟现判，另记覆盖者键给提交门禁核
                refused.append(f"catching-test-unresolved:{why}")
                coverage_key = judgement.coverers_for(current["unit"])[0] or ""
            else:
                unit, target_id, _why = judgement.rules.resolve_test(judgement.tree, test_name)
                if unit is not None:
                    _key, composition, _forms = judgement.tree.closure_key(unit, judgement.globals, target_id)
                    library.write_composition(test_key, composition)
        if outcome == "missed":
            coverage_key = judgement.coverers_for(current["unit"])[0] or ""
        fields = [current["key"] or "", sequence, item["package"], item["name"], outcome, stage, test_name, test_key, coverage_key,
                  f"{totals['build']:.1f}", f"{totals['test']:.1f}", today_text(), method, current["locator"], ",".join(refused), note_text]
        library.write_composition(current["key"], current["composition"])
        library.record_verdict(fields)
        return ItemRow(fields, step_name, step_seconds, environment_doubt)

    def run_stage(stage):
        """跑这一段的每一步：交 None（全绿）或已经落下的那一行。"""
        for step in runner.steps:
            step_name, step_stage, kind, _command, test_failed_exit = step
            if step_stage != stage:
                continue
            command, selection_text = runner.command_for(step, selection)
            if command is None:
                continue
            if step_name in PER_ENTRY_TEST_STEPS:
                ran_any_test.append(step_name)
            exit_code, seconds = runner.run_step(step, sequence, command=command, selection_text=selection_text)
            totals[kind] += seconds
            log_path = runner.log_path(sequence, step_name)
            if exit_code == 0:
                os.remove(log_path)
                continue
            if kind == "build" and exit_code == CARGO_FAILED_EXIT:
                if log_shows_compile_error(log_path) or break_is("unviable-by-exit-code"):
                    return record("unviable", stage, note_text=f"{step_name} 编不过", step_name=step_name, step_seconds=seconds)
                return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效", step_name=step_name,
                              step_seconds=seconds, environment_doubt=f"第 {sequence} 条的 {step_name} 退 {exit_code} 而没有编译错误")
            if kind == "test" and exit_code == test_failed_exit:
                package_of_step = VERIFICATION_PACKAGE if "verification" in step_name else ""
                return record("caught", stage, first_failing_test(log_path, test_failed_exit, package_of_step), note_text=step_name,
                              step_name=step_name, step_seconds=seconds)
            if break_is("unknown-exit-stops"):
                refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（认不出的退出码）：不落行", f"看日志 {log_path}", EXIT_INCONSISTENT)
            return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code}（认不出的退出码，不是这条变异的结局；日志 {os.path.basename(log_path)}）",
                          step_name=step_name, step_seconds=seconds)
        return None

    try:
        apply_mutation(library, item)
        if with_stage_one:
            row = run_stage("one")
            if row is not None:
                return row
            row = record(STAGE_ONE_ONLY, "one", note_text="第一段没红；第二段接着跑" if with_stage_two else "第一段没红；这一趟没跑第二段")
            if not with_stage_two:
                return row
        row = run_stage("two")
        if row is not None:
            return row
        return record("caught" if break_is("stage-two-missed-as-caught") else "missed", "two")
    finally:
        restore_original(library)


class GpuOutcome(NamedTuple):
    outcome: str       # missed / caught / driver-error / not-run
    caught_by: str     # 抓到它的测试（「<二进制 id> <用例名>」），别的结局是空
    note: str


# gate-overlap:copy-kept crash-point-replay.sh 的 list_feature_gated_test_files 是嵌在 bash 里的 python，读不进来；这里同一套判法照搬一份，两边都认 tests/ 里的 cfg 点到的特性
def gated_test_files(tests_directory):
    """验证包 tests/ 顶层每个 .rs 里 #[cfg(…)] / #![cfg(…)] 点到的特性：[(文件名, whole-file 或 items, [特性])]，照文件名排序；挂特性的一个都没有交空表。
    括号按层数配对（rustfmt 把长的 cfg 折成几行也认），// 起头的行不读。"""
    found = []
    try:
        names = sorted(name for name in os.listdir(tests_directory) if name.endswith(".rs") and os.path.isfile(os.path.join(tests_directory, name)))
    except OSError:
        return found
    for name in names:
        with open(os.path.join(tests_directory, name), encoding="utf-8", errors="replace") as handle:
            text = "\n".join(line for line in handle.read().splitlines() if not line.lstrip().startswith("//"))
        features, whole_file = set(), False
        for start in GPU_ATTRIBUTE_START.finditer(text):
            depth, position = 1, start.end()
            while position < len(text) and depth > 0:
                depth += {"(": 1, ")": -1}.get(text[position], 0)
                position += 1
            named = GPU_FEATURE_NAME.findall(text[start.end():position])
            features.update(named)
            if named and start.group(1) == "!":
                whole_file = True
        if features:
            found.append((name, "whole-file" if whole_file else "items", sorted(features)))
    return found


def gpu_pass_command(kind, features, labels):
    """GPU 那一趟一步的命令（verification.md 里带特性那一趟）：编译步 cargo test --release -p singlefs-verification --features <特性> <目标> --no-run；
    测试步在它后面加 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起），直接起、不另套清场（文件头「GPU 那一趟」）。
    labels 是要跑的目标：lib 是 --lib，别的是 --test <文件名>。弄坏开关 gpu-test-threads-dropped 不加 --test-threads 1。"""
    targets = []
    for label in sorted(labels, key=lambda text: (text != "lib", text)):
        targets += ["--lib"] if label == "lib" else ["--test", label]
    head = ["cargo", "test", "--locked", "--release", "-p", VERIFICATION_PACKAGE, "--features", ",".join(features), *targets]
    if kind == "build":
        return head + ["--no-run"]
    threads = [] if break_is("gpu-test-threads-dropped") else ["--test-threads", "1"]
    return head + ["--", "--nocapture", *threads]


def gpu_log_text(log_path):
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""


def gpu_failing_test(log_path):
    """GPU 那一趟第一条没过的测试：先照 libtest 结果行认（first_failing_test）；--nocapture 加 --test-threads 1 下结果行常被测试自己的输出打散，认不出就读收尾的
    「failures:」名单（缩进四格的测试名），二进制照前面最近的 Running 行补。认不出交 UNRECOGNISED_TEST。"""
    name = first_failing_test(log_path, CARGO_FAILED_EXIT, VERIFICATION_PACKAGE)
    if name != UNRECOGNISED_TEST:
        return name
    binary, listing = "", False
    for line in gpu_log_text(log_path).splitlines():
        binary = libtest_binary(line, binary, VERIFICATION_PACKAGE)
        if line.rstrip() == "failures:":
            listing = True
            continue
        if listing:
            named = re.match(r"^    (\S+)\s*$", line)
            if named:
                return f"{binary} {named.group(1)}" if binary else named.group(1)
            if line.strip() and not line.startswith(" "):
                listing = False
    return UNRECOGNISED_TEST


def gpu_classify_build(exit_code, log_path):
    """GPU 那一趟编译步的结局：→ (None 编得过 | missed 不会有 | unviable / driver-error, 备注)。
    编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类）不记无效，记驱动错。"""
    if exit_code == 0:
        return None, ""
    if exit_code == CARGO_FAILED_EXIT:
        if log_shows_compile_error(log_path):
            return "unviable", f"{GPU_BUILD_STEP} 编不过"
        return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效"
    return DRIVER_ERROR, f"{GPU_BUILD_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）"


def gpu_classify_test(exit_code, log_path):
    """GPU 那一趟测试步的结局（GpuOutcome）：退 0 且跑到了测试、日志里没有 NOT_RUN_NO_GPU_ADAPTER 才是没红（missed）；
    日志里有 NOT_RUN_NO_GPU_ADAPTER（GPU 测试没找到卡，打一行就返回成功）是本次未跑（not-run，不落行、不算没红）；
    退 0 而一个测试都没跑到不算没红，记驱动错；退 101 是有测试没过 = 抓到，带第一条没过的测试。"""
    text = gpu_log_text(log_path)
    passed = sum(int(count) for count in GPU_PASSED_COUNT.findall(text))
    if exit_code == 0:
        if GPU_NO_ADAPTER_MARK in text and not break_is("gpu-not-run-as-missed"):
            return GpuOutcome(GPU_NOT_RUN, "", f"no-adapter：日志里有 {GPU_NO_ADAPTER_MARK}，GPU 测试没找到卡、没跑")
        if passed == 0:
            return GpuOutcome(DRIVER_ERROR, "", "GPU 趟一个测试都没跑到（日志里没有 passed 的计数）：不算没红")
        return GpuOutcome("missed", "", f"{passed} 条测试都过了")
    if exit_code == CARGO_FAILED_EXIT:
        return GpuOutcome("caught", gpu_failing_test(log_path), GPU_TEST_STEP)
    return GpuOutcome(DRIVER_ERROR, "", f"{GPU_TEST_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）")


def gpu_combined_state(cpu, needed, gpu):
    """两趟合起来的状态（ItemState）与没结论时的理由：CPU 那一趟没红、又要 GPU 那一趟的条，看 GPU 那一趟的状态——抓到 / 没红 / 无效就是定论，
    驱动错照样是没结论，库里还没有它的结论是 none（理由 gpu-not-run，不算没红）；别的条就是 CPU 那一趟的状态。
    一条两趟里任一趟红算抓到：CPU 那一趟抓到的，GPU 那一趟不用跑。弄坏开关 gpu-red-not-caught：GPU 那一趟红不算抓到；gpu-not-run-as-missed：本次未跑记成没红。"""
    if cpu.state != "missed" or not needed:
        return cpu, ""
    if break_is("gpu-red-not-caught") and gpu.state == "caught":
        return ItemState("missed", cpu.passed_stage_one), ""
    if gpu.state in DECISIVE_OUTCOMES:
        return ItemState(gpu.state, cpu.passed_stage_one), ""
    if gpu.state == DRIVER_ERROR:
        return ItemState(gpu.state, cpu.passed_stage_one), f"gpu-{gpu.state}"
    if break_is("gpu-not-run-as-missed"):
        return ItemState("missed", cpu.passed_stage_one), ""
    return ItemState(NOT_RUN, cpu.passed_stage_one), "gpu-not-run"


def lines_digest(lines):
    return hashlib.sha256("".join(line + "\n" for line in lines).encode("utf-8", "surrogateescape")).hexdigest()


# ── GPU 判定表：判定库 gpu-verdicts/（只追加，文件头「判定库」） ─────────────────────────────────────────────

def read_gpu_verdicts(library):
    """GPU 判定表：段数对的行才算数，不对的条目列进 malformed。→ LibraryRows。"""
    return library.read_rows(GPU_VERDICTS_TABLE, GPU_VERDICT_COLUMNS)


# ── GPU 那一趟的判定：要不要跑、键、沿用、两趟合起来 ───────────────────────────────────────────────────────────

class GpuView(NamedTuple):
    needed: bool           # CPU 那一趟没红、又有带特性的测试够得着它：要 GPU 那一趟
    labels: list           # 要跑的目标（lib、挂特性的测试文件名）
    need: dict             # GpuJudging.need 的原样（coverers 键等）
    gpu: ItemState         # GPU 那一趟自己的状态（没要这一趟的条是 none）
    combined: ItemState    # 两趟合起来的状态
    reason: str            # 合起来没有结论时的理由
    deciding: object       # 定下 GPU 状态的那一行（没有是 None）


def cpu_state_for_gpu(judgement, item):
    """CPU 那一趟的状态，不论拒算：真跑时 Judgement.state 对拒算的条一律给 none（每趟现判），GPU 那一趟要看的是 CPU 那一趟最后落的结论，所以临时按只读的判法问一次。"""
    saved = judgement.for_run
    judgement.for_run = False
    try:
        return judgement.state(item)
    finally:
        judgement.for_run = saved


def gpu_decided_keys(library, judgement, items, arguments):
    """选条之前问：items（CPU 一趟的没红行被撤绿的那几条）里哪些由 GPU 一趟定红不红、而 GPU 趟的真实状态此刻不是没红 → 它们的单元键。
    要 GPU 趟（CPU 一趟的真实状态是没红、又有带特性的测试够得着）的条，红不红看合起来的状态（文件头「记红、撤绿、下趟现判」）：GPU 趟没红才红，
    抓到 / 无效 / 还没有结论 / 没结论的，CPU 一趟的没红行照沿用（GPU 趟自己补或重判）。没有带特性的测试、或这些条用不着 GPU 趟：交空集合。"""
    decided = set()
    judging = GpuJudging(library, judgement, arguments)
    if not judging.enabled:
        return decided
    for item in items:
        if cpu_state_for_gpu(judgement, item)[0].state != "missed":
            continue
        current = judgement.key_of(item)
        if current["key"] is None:
            continue
        need = judging.need(current)
        if need["needed"] and judging.state(current, need, for_run=False)[0].state != "missed":
            decided.add(current["key"])
    return decided


class GpuJudging:
    """一棵树上 GPU 那一趟的判定（文件头「GPU 那一趟」）：带特性的测试够得着哪些单元、这一趟要跑哪几个目标、判定库里键相同的行沿用不沿用。
    judgement 是同一棵树上 CPU 那一趟的 Judgement（共用它的单元树、全局那几行、排除表）。"""

    def __init__(self, library, judgement, arguments):
        self.library, self.judgement, self.arguments = library, judgement, arguments
        self.rules = judgement.rules
        self.gated = gated_test_files(os.path.join(judgement.root, GPU_TESTS_DIRECTORY))
        self.features = sorted({feature for _name, _scope, features in self.gated for feature in features})
        self.enabled = GPU_FEATURE in self.features
        self.gated_labels = {name[:-3] for name, _scope, _features in self.gated}
        self.feature_arguments = [f"{VERIFICATION_PACKAGE}/{feature}" for feature in self.features]
        self.entry_list, self.closures, self.keys, self.cached_pass_key = None, {}, {}, None
        self.table = read_gpu_verdicts(library)
        self.by_key = {}
        for row in self.table.rows:
            self.by_key.setdefault(row["key"], []).append(row)

    def pass_key(self):
        """GPU 那一趟自己的键（每一行 GPU 判定都带）：这一趟的写法、编进来的特性。"""
        if self.cached_pass_key is None:
            self.cached_pass_key = lines_digest(["broad-mutation-gpu-pass-v1", "features\t" + ",".join(self.features)])
        return self.cached_pass_key

    def entries(self):
        """这一趟跑得到的测试（静态列出，不含 #[ignore] 的）：验证包的 lib 与挂特性的测试文件里的测试，带上要跑的目标（lib / 文件名）。"""
        if self.entry_list is None:
            found = []
            for entry in self.judgement.runnable((VERIFICATION_PACKAGE,)):
                parts = entry["target"].split(":", 2)
                if parts[1] == "lib":
                    label = "lib"
                elif parts[1] == "test" and parts[2] in self.gated_labels:
                    label = parts[2]
                else:
                    continue
                found.append(dict(entry, label=label))
            self.entry_list = found
        return self.entry_list

    def closure_of(self, entry):
        """一条测试在这一趟的 cfg 与特性下的闭包（带 --release 的 cfg 去掉 debug_assertions，特性是挂特性的文件点到的那几个）；入口在这一组 cfg 下不编交 None。"""
        cache_key = (id(entry["unit"]), entry["target"])
        if cache_key not in self.closures:
            tree = self.judgement.tree
            library = tree.library
            release_cfg = [item for item in library.DEFAULT_RUST_CFG if item != "debug_assertions"]
            found = None
            for extra in ((), ("test",)):
                try:
                    found = library.rust_closure_of(tree.workspace, entry["unit"], entry["target"], release_cfg + list(extra), no_default_cfg=True,
                                                    features=self.feature_arguments)
                    break
                except library.UsageError:
                    found = None
            self.closures[cache_key] = found
        return self.closures[cache_key]

    @staticmethod
    def references_gpu(closure):
        """闭包真引到 gpu 原子（带这一趟的真值 true）：闭包里有单元的 cfg 点到 feature = "gpu"。"""
        return any(atom == f'feature="{GPU_FEATURE}"\ttrue' for atom in closure.cfg_atoms)

    def test_key_of(self, entry):
        """一条测试在这一趟的测试键 = 它的单元闭包键（闭包键里有单元的种类与去注释指纹、闭包真引到的 cfg 原子与真值、全局那几行与 edition）：→ (键或 None, 拒算形态)。"""
        cache_key = (id(entry["unit"]), entry["target"])
        if cache_key not in self.keys:
            closure = self.closure_of(entry)
            if closure is None:
                self.keys[cache_key] = (None, ["entry-cfg-inactive"])
            else:
                tree = self.judgement.tree
                dependencies, dependency_refusals = tree.dependency_lines(closure)
                key, closure_refusals = tree.library.closure_key(closure, toolchain=list(self.judgement.globals) + tree.edition_lines(closure),
                                                                 dependencies=dependencies, allow_refused=True)
                forms = sorted({refusal.form for refusal in closure_refusals}) + dependency_refusals
                self.keys[cache_key] = (None, forms) if forms else (key, [])
        return self.keys[cache_key]

    def need(self, current):
        """这一条要不要 GPU 那一趟：→ {needed, labels, coverers, why}。要 = 有一条这一趟跑得到的测试，它带特性的闭包里有这条变异的容器单元、
        又真引到 gpu 原子（闭包不碰容器单元的测试、碰到了却不引 gpu 的，这一趟帮不上）；labels 是这些测试所在的目标；coverers 是它们各自测试键的多重集取 sha256
        （沿用没红的要它相同），有一条的键拒算就是空串、why 写原因。弄坏开关 gpu-all-items：不看闭包，每一条 CPU 那一趟没红的都要、目标取全部。"""
        unit = current["unit"]
        if unit is None or not self.enabled:
            return {"needed": False, "labels": [], "coverers": "", "why": "no-unit"}
        keys, labels, why = [], set(), ""
        for entry in self.entries():
            if break_is("gpu-all-items"):
                labels.add(entry["label"])
                continue
            closure = self.closure_of(entry)
            if closure is None or id(unit) not in closure.members or not self.references_gpu(closure):
                continue
            labels.add(entry["label"])
            key, refused = self.test_key_of(entry)
            if key is None and not why:
                why = f"coverer-refused:{entry['binary']} {entry['test']}:" + ",".join(refused or ["closure"])
            elif key is not None:
                keys.append(key)
        if not labels:
            return {"needed": False, "labels": [], "coverers": "", "why": ""}
        coverers = "" if why or break_is("gpu-all-items") else lines_digest(["gpu-coverers-v1"] + sorted(keys))
        return {"needed": True, "labels": sorted(labels), "coverers": coverers, "why": why}

    def catching_key(self, caught_by):
        """抓到它的测试在这一趟此刻的测试键：→ (键或 "", 为什么)。"""
        unit, target_id, why = self.rules.resolve_test(self.judgement.tree, caught_by)
        if unit is None:
            return "", why
        key, refused = self.test_key_of({"unit": unit, "target": target_id})
        return (key, "") if key else ("", "catching-test-refused:" + ",".join(refused))

    def row_holds(self, row, current, need, for_run):
        """库里键相同的这一行 GPU 判定此刻还认不认（沿用判据，同 mutation-prune.py 的 carry_decision）：→ (认不认, 为什么不认)。
        这一趟自己的键（pass_key）对不上的不认；抓到的要那条测试此刻的测试键相同；没红的要覆盖者键相同；无效的键相同就认；
        驱动错不是结论，认下来只为定状态；抓到它的测试认不出的，真跑每趟现判、只读时看覆盖者键。"""
        if row["gpu_key"] != self.pass_key():
            return False, "gpu-pass-changed"
        outcome = row["outcome"]
        if outcome not in DECISIVE_OUTCOMES:
            return True, ""
        if outcome == "caught" and row["note"].startswith("catching-test-unresolved"):
            if for_run:
                return False, row["note"]
            return (True, "") if need["coverers"] and need["coverers"] == row["coverage_key"] else (False, "coverers-changed")
        test_now, test_why = self.catching_key(row["caught_by"]) if outcome == "caught" else ("", "")
        record = {"key": row["key"], "outcome": outcome, "caught_by": row["caught_by"], "test_key": row["test_key"], "coverage_key": row["coverage_key"]}
        return self.rules.carry_decision(record, {"key": current["key"], "refused": []}, test_now, test_why, need["coverers"], need["why"])

    def state(self, current, need, for_run):
        """GPU 那一趟的状态：→ (ItemState, 为什么沿用不了, 定下状态的那一行)。真跑时拒算的条一律沿用不了（每趟现判）。"""
        if current["key"] is None or (for_run and current["refused"]):
            return ItemState(NOT_RUN, False), "refused:" + ",".join(current["refused"]), None
        rows = self.by_key.get(current["key"], [])
        kept, reasons = [], []
        for row in rows:
            holds, why = self.row_holds(row, current, need, for_run)
            (kept if holds else reasons).append(row if holds else why)
        state = state_of_outcomes([row["outcome"] for row in kept])
        deciding = next((row for row in reversed(kept) if row["outcome"] == state.state), None)
        if for_run and state.state in DECISIVE_OUTCOMES and not break_is("missed-carried"):
            # 记红、撤绿、下趟现判（文件头「判定行」末段）：这一趟自己的键（pass_key）下，同一把键的定论里最后一个是没红的，不沿用、现判；认不认只定 --summary 的状态，撤绿不看覆盖者键变没变
            decisive = [row["outcome"] for row in rows if row["gpu_key"] == self.pass_key() and row["outcome"] in DECISIVE_OUTCOMES]
            if decisive and decisive[-1] == "missed":
                return ItemState(NOT_RUN, False), "missed-rejudged", None
        return state, (reasons[-1] if reasons else ("no-record" if not rows else "")), deciding

    def view(self, item, cpu, for_run):
        """一条两趟合起来的样子（GpuView）。cpu 是 CPU 那一趟的 ItemState。"""
        nothing = ItemState(NOT_RUN, False)
        if cpu.state != "missed" or not self.enabled:
            return GpuView(False, [], {"needed": False, "labels": [], "coverers": "", "why": ""}, nothing, cpu, "", None)
        current = self.judgement.key_of(item)
        if current["key"] is None:
            return GpuView(False, [], {"needed": False, "labels": [], "coverers": "", "why": "no-key"}, nothing, cpu, "", None)
        need = self.need(current)
        if not need["needed"]:
            return GpuView(False, [], need, nothing, cpu, "", None)
        gpu, _why, deciding = self.state(current, need, for_run)
        combined, reason = gpu_combined_state(cpu, True, gpu)
        return GpuView(True, need["labels"], need, gpu, combined, reason, deciding)


# ── 选条、列项、汇总 ───────────────────────────────────────────────────────────────────────────────────────

def changed_since_head(source):
    """弄坏开关 select-by-commit 用的旧选法：被判那棵树里相对 HEAD 改过的文件（git diff --name-only HEAD）。"""
    found = subprocess.run(["git", "-C", source, "diff", "--name-only", "HEAD"], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return set(found.stdout.split()) if found.returncode == 0 else set()


def select_items(judgement, items, arguments, states):
    """交 [(条, 要不要跑第一段, 要不要跑第二段)]，按序号（点名的按点名的次序）。states 是 {序号: (ItemState, 理由, 定下状态的那一行)}。"""
    nothing = (ItemState(NOT_RUN, False), "", None)
    if break_is("rerun-done") or (break_is("no-carry-without-arguments") and not arguments.item):
        states = {}
    forced = set()
    if break_is("select-by-commit"):
        changed = changed_since_head(arguments.source_root)
        forced = {item["sequence"] for item in items if item["file"] in changed}
    want_stage_two = not stage_two_skipped()
    how_to_look = f"python3 {SELF_PATH} --list-items 逐行列「序号、包、变异名、状态」；序号写数字，变异名整条照抄"
    if arguments.item:
        if arguments.start_from is not None or arguments.package:
            refuse("--item 与 --from / --package 同时给了", "点名就只给 --item（可给多次）；按范围跑就不给 --item")
        by_sequence = {str(item["sequence"]): item for item in items}
        by_name = {item["name"]: item for item in items}
        named = []
        for wanted in arguments.item:
            item = by_sequence.get(wanted) or by_name.get(wanted)
            if item is None:
                if break_is("accept-unknown-item"):
                    continue
                refuse(f"不认得的项：{wanted}（此刻清单 {len(items)} 条）", how_to_look)
            named.append(item)
        selected = []
        for item in named:
            state = states.get(item["sequence"], nothing)[0]
            if named_items_rejudged() or break_is("named-items-always-rerun") or item["sequence"] in forced:
                selected.append((item, True, want_stage_two))
            elif state.state in DECISIVE_OUTCOMES:
                say(f"  · 第 {item['sequence']} 条沿用得了（{OUTCOME_TEXT[state.state]}），跳过；键相同的结论不重判，复用哨兵会抽查")
            elif state.state == STAGE_ONE_ONLY:
                if want_stage_two:
                    selected.append((item, False, True))
                else:
                    say(f"  · 第 {item['sequence']} 条已过第一段，这一趟的第二段没跑（只供自证的 BROAD_MUTATION_STAGE_ONE_ONLY），跳过")
            else:
                selected.append((item, True, want_stage_two))
        return selected
    if arguments.package and arguments.package not in MUTATED_PACKAGES:
        refuse(f"--package 给的「{arguments.package}」不在清单的包里", f"只有这几个：{'、'.join(MUTATED_PACKAGES)}；{how_to_look}")
    if arguments.start_from is not None and not 1 <= arguments.start_from <= len(items):
        refuse(f"--from {arguments.start_from} 不在 1..{len(items)} 里", how_to_look)
    selected = []
    for item in items:
        if arguments.start_from is not None and item["sequence"] < arguments.start_from:
            continue
        if arguments.package and item["package"] != arguments.package:
            continue
        state = states.get(item["sequence"], nothing)[0]
        if item["sequence"] in forced:
            selected.append((item, True, want_stage_two))
        elif state.state in (NOT_RUN, DRIVER_ERROR):
            selected.append((item, True, want_stage_two))
        elif state.state == STAGE_ONE_ONLY and want_stage_two:
            selected.append((item, False, True))
    return selected
