"""broad-mutation.py 拆出的一份。
步的跑法：按步取命令、跑一步（Runner）、libtest 与 nextest 结果的认法、工具版本、改坏与还原源码。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""
import hashlib
import importlib.util
import json
import os
import secrets
import signal
import subprocess
import time
from broad_mutation_base import (ADMISSION, BASELINES_TABLE, BASELINE_ENTRIES_TABLE, BASELINE_ENTRY_COLUMNS, BASELINE_SEQUENCE, CARGO_FAILED_EXIT,
    COMPILING_LINE, COVERAGE_COLUMNS, COVERAGE_TABLE, ENVIRONMENT_SEQUENCE, EXIT_BASELINE_RED, EXIT_INCONSISTENT, GPU_TEST_STEP, MUTATED_PACKAGES,
    NEXTEST_FAILURE_LINE, NEXTEST_PASS_LINE, NEXTEST_TEST_FAILED_EXIT, PROGRESS_INTERVAL_SECONDS, SCRIPT_DIRECTORY, SELFTEST_STEP_DRIVER_NAME,
    SELFTEST_WORK_VARIABLE, SELF_PATH, STAGE_ONE_PACKAGES, UNRECOGNISED_TEST, VERIFICATION_PACKAGE, libtest_result_lines)
from broad_mutation_judging import (Refusal, break_is, libtest_failed_name, llvm_bin, nextest_failed_name, note, now_text, prune_rules, refuse, say,
    stage_two_skipped, test_variable, today_text, write_atomically)
from broad_mutation_library import (Stopped, remove_tree)


# ── 登记的崩溃枚举用例与放量分项：第二段 验证包与 GPU 那一趟经 --skip 排掉 ─────────────────────────────────────────────

def registered_functions(root):
    """第二段 验证包与 GPU 那一趟用 --skip 排掉的 {(测试目标, 用例函数)}：登记成 crash-case: 的崩溃枚举用例与放量分项，
    就是门禁按条那一趟与入口按条那一趟用 --exclude-registered 排掉的同一张排除单（scripts/gate/admission.py 的 registered_test_functions，读法只在那里一份）；
    它们另走按块那一路（门禁 crash-point-replay），这里不跑。读不了退 1。
    只供测试的 BROAD_MUTATION_REGISTERED=<目标:用例函数,…> 换掉读法（自证的假树里没有登记表）。"""
    given = test_variable("BROAD_MUTATION_REGISTERED")
    if given is not None:
        return {tuple(item.split(":", 1)) for item in given.split(",") if ":" in item}
    if not ADMISSION:
        specification = importlib.util.spec_from_file_location("admission_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        ADMISSION.append(module)
    try:
        return set(ADMISSION[0].registered_test_functions(root))
    except (ADMISSION[0].RegistrationError, OSError) as error:
        refuse(f"读不了登记的崩溃枚举用例与放量分项（{error}）：第二段不知道该用 --skip 排掉哪几条",
               f"看 {root} 的 .claude/gate.d/stage-inputs.tsv 与 scripts/crash/crash-amplification.sh 还在不在、成不成形（门禁 crash-point-replay 读同一份）", EXIT_INCONSISTENT)


# ── 每一步命令 ─────────────────────────────────────────────────────────────────────────────────────────────

def package_arguments(packages):
    return [word for package in packages for word in ("-p", package)]


def step_table(arguments):
    """每一步：(步名, 哪一段, build / test, 命令, 测试没过的退出码)。
    第二段只补第一段没跑到的：文档测试（被变异的两个包）、验证包里的全部测试函数（登记的崩溃枚举用例与放量分项另走按块那一路，
    起命令时经 --skip 排掉：Runner.command_with_skips）。第一段跑过的那三个包的用例第二段不再跑。"""
    stage_one_arguments = package_arguments(STAGE_ONE_PACKAGES)
    threads = str(arguments.threads)
    sync_free = ["eatmydata"]
    stage_two_prefix = sync_free if arguments.stage_two_eatmydata else []
    verification_arguments = ["--release", "-p", VERIFICATION_PACKAGE, "--lib", "--tests"]
    stage_one_build = ["cargo", "nextest", "run", "--locked", *stage_one_arguments, "--no-run"]
    stage_one_test = sync_free + ["cargo", "nextest", "run", "--locked", *stage_one_arguments, "-j", threads]
    stage_one = [
        ("one-build", "one", "build", stage_one_build, None),
        ("one-test", "one", "test", stage_one_test, NEXTEST_TEST_FAILED_EXIT),
    ]
    if break_is("stage-two-full-cargo-test"):
        return stage_one + [
            ("two-build-debug", "two", "build", ["cargo", "test", "--locked", *stage_one_arguments, "--no-run"], None),
            ("two-test-debug", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *stage_one_arguments], CARGO_FAILED_EXIT),
            ("two-build-verification", "two", "build", ["cargo", "test", "--locked", *verification_arguments, "--no-run"], None),
            ("two-test-verification", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *verification_arguments], CARGO_FAILED_EXIT),
        ]
    return stage_one + [
        ("two-test-doc", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", "--doc", *package_arguments(MUTATED_PACKAGES)], CARGO_FAILED_EXIT),
        ("two-build-verification", "two", "build", ["cargo", "test", "--locked", *verification_arguments, "--no-run"], None),
        ("two-test-verification", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *verification_arguments], CARGO_FAILED_EXIT),
    ]


def progress_line(sequence, step_name, elapsed, log_path):
    """一步跑着时的一行进度（文件头「输出」）：哪一条、哪一步、跑了多久，与这一步此刻的日志里数得出的已编 crate、已过与没过的用例。"""
    label = "baseline" if sequence == BASELINE_SEQUENCE else sequence
    compiled = passed = failed = 0
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.rstrip("\n")
                if COMPILING_LINE.match(line):
                    compiled += 1
                elif NEXTEST_PASS_LINE.match(line) or libtest_outcome(line) == "ok":
                    passed += 1
                elif NEXTEST_FAILURE_LINE.match(line) or libtest_outcome(line) == "FAILED":
                    failed += 1
    except OSError:
        pass
    return (f"BROAD_MUTATION_PROGRESS sequence={label} step={step_name} elapsed_seconds={elapsed:.0f} "
            f"compiled_crates={compiled} tests_passed={passed} tests_failed={failed}")


class Runner:
    def __init__(self, library, arguments):
        self.library, self.arguments = library, arguments
        # 登记的崩溃枚举用例与放量分项（上面「登记的崩溃枚举用例与放量分项」）：只跑第一段的趟用不着，读不了也不挡它
        try:
            self.registered = registered_functions(arguments.source_root)
        except Refusal:
            if not stage_two_skipped():
                raise
            self.registered = set()
        self.skip_names = sorted({function for _target, function in self.registered})
        self.steps = step_table(arguments)
        self.fake_driver = test_variable("BROAD_MUTATION_STEP_DRIVER") == "1"
        self.selftest_work = test_variable(SELFTEST_WORK_VARIABLE) or ""   # 假一步在它下面（文件头「只供测试」）
        self.stages_with_fresh_baseline = set()
        self.judgement = None
        self.coverage = {"one": {}, "verification": {}}
        self.coverage_available = set()
        self.profile_counts = {}
        self.baseline_keys = {}
        self.baseline_refusals = {}    # 步名 → 基线键拒算的那一句（Judgement.baseline_key）
        self.run_token = secrets.token_hex(16)   # 拒算的基线这一趟独有的那一把键从它算：记下的基线行以后哪一趟都对不上、不沿用

    def baseline_key_of(self, step_name):
        """这一步此刻的基线键（Judgement.baseline_key：这一步真跑到的判定的单元闭包）。拒算的（文件头「基线」）交这一趟独有的一把（run_token 算的）：
        这一趟照跑、记的基线行在这一趟里照常对得上，以后哪一趟都对不上、不沿用。"""
        if step_name not in self.baseline_keys:
            step = next((step for step in self.steps if step[0] == step_name), (step_name, "", "test", [], None))   # 覆盖那一步不在步表里
            key, refused = self.judgement.baseline_key(step, self.arguments.threads)
            if key is None:
                self.baseline_refusals[step_name] = refused
                key = hashlib.sha256(f"refused-baseline\0{step_name}\0{self.run_token}".encode()).hexdigest()
            self.baseline_keys[step_name] = key
        return self.baseline_keys[step_name]

    def command_for(self, step, selection):
        """这一条这一步的命令与「跑了哪些」的一句：按覆盖挑出来的测试（C2）、第二段按二进制（E6）；交 (None, …) 是这一步没有要跑的。"""
        name, command = step[0], list(step[3])
        selection = selection or {}
        if name == "one-test" and selection.get("one") is not None:
            if not selection["one"]:
                return None, "none"
            filters = prune_rules().nextest_filters(selection["one"])
            return command + [word for text in filters for word in ("-E", text)], f"{len(selection['one'])} tests"
        if name == "two-test-verification" and selection.get("verification") is not None:
            if not selection["verification"]:
                return None, "none"
            command = [word for word in command if word not in ("--lib", "--tests")]
            for target in selection["verification"]:
                command += ["--lib"] if target == "lib" else ["--test", target]
            return command, ",".join(selection["verification"])
        if name == "two-test-doc" and break_is("doc-tests-skipped"):
            return None, "none"
        return command, "all"

    def ensure_coverage(self, stages):
        """C2：没改过的树上的覆盖（第一段逐条测试、验证包逐个二进制），按记录键（那条测试的测试键）在判定库 coverage/ 里照用，
        对不上的那几条插桩跑（--coverage-step）；跑红拒（退 6）。之后 self.coverage[段] = {(二进制, 用例): {种类:指纹}}。"""
        judgement, rules = self.judgement, self.judgement.rules
        records = {}
        for entry in self.library.read_rows(COVERAGE_TABLE, COVERAGE_COLUMNS).rows:
            try:
                records[entry["record_key"]] = (set(json.loads(entry["covered"])), int(entry["profiles"] or 1))
            except ValueError:
                continue
        for stage in stages:
            jobs = judgement.coverage_jobs(stage)
            known = {job: record_key for job, record_key in jobs.items() if record_key and record_key in records}
            name = f"coverage-{stage}"
            out = os.path.join(self.library.scratch + "-coverage", f"{name}.json")
            os.makedirs(os.path.dirname(out), exist_ok=True)
            if os.path.exists(out):
                os.remove(out)
            llvm = llvm_bin(self.arguments)
            if not self.fake_driver and not all(os.path.isfile(os.path.join(llvm, tool)) for tool in ("llvm-profdata", "llvm-cov")):
                refuse(f"按覆盖挑测试要 llvm-profdata 与 llvm-cov，{llvm or 'rustc 的 sysroot'} 下没有",
                       "装 rustup 组件：rustup component add llvm-tools（与编译器同一份 LLVM，profraw 格式对得上）；装在别处就加 --llvm-bin <目录>")
            specification = os.path.join(os.path.dirname(out), f"{name}.spec.json")
            write_atomically(specification, json.dumps({"kind": stage, "tree": self.library.tree, "target": self.library.target_coverage,
                                                        "scratch": self.library.scratch, "llvm_bin": llvm, "workers": self.arguments.threads, "out": out,
                                                        "skips": self.skip_names, "known": [list(job) for job in sorted(known)]}).encode())
            say(f"  · 覆盖 {name}：在没改过的树上插桩列一遍测试，记录键对得上的 {len(known)} 条照用、其余现跑")
            step = (name, stage, "test", ["python3", SELF_PATH, "--coverage-step", specification], None)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, extra={"BROAD_MUTATION_STEP_SPEC": specification})
            if exit_code != 0 or not os.path.isfile(out):
                refuse(f"覆盖 {name} 在没改过的树上退 {exit_code}：拿不到「单元 → 执行到它的测试」，一条变异都不跑",
                       f"看日志 {self.log_path(BASELINE_SEQUENCE, name)}（插桩下跑红的测试逐条列在里面）；是环境就修环境重跑同一条命令")
            with open(out, encoding="utf-8") as handle:
                result = json.load(handle)
            if result is None:
                say(f"  · 覆盖 {name}：这一趟拿不到覆盖，每条变异跑全部测试")
                continue
            self.coverage_available.add(stage)
            for job in result.get("tests", []):
                job = tuple(job)
                if job in known:
                    covered, profiles = records[known[job]]
                    self.coverage[self.coverage_part(stage, job[0])][job] = covered
                    self.profile_counts[job] = profiles
            new_rows = []
            for record in result.get("jobs", []):
                job = (record["binary"], record["test"])
                covered = set(rules.fingerprints_of(judgement.tree, record.get("covered", [])))
                self.coverage[self.coverage_part(stage, job[0])][job] = covered
                self.profile_counts[job] = record.get("profiles", 1)
                record_key = jobs.get(job)
                if record_key:
                    new_rows.append([record_key, stage, job[0], job[1], json.dumps(sorted(covered)), record.get("profiles", 1), today_text()])
            if new_rows:
                self.library.append_rows(COVERAGE_TABLE, new_rows)
            say(f"  · 覆盖 {name} 好了，{seconds:.1f} 秒：照用 {len(known)} 条、现跑 {len(result.get('jobs', []))} 条（记进库的 {len(new_rows)} 条）")

    @staticmethod
    def coverage_part(stage, binary):
        return "one" if stage == "one" else "verification"

    def step_named(self, step_name):
        return next(step for step in self.steps if step[0] == step_name)

    def log_path(self, sequence, step_name):
        prefix = "baseline" if sequence == BASELINE_SEQUENCE else str(sequence)
        return os.path.join(self.library.logs, f"{prefix}.{step_name}.log")

    def progress_interval(self):
        fake = test_variable("BROAD_MUTATION_PROGRESS_INTERVAL")
        return float(fake) if fake else PROGRESS_INTERVAL_SECONDS

    def wait_reporting_progress(self, child, step_name, sequence, started):
        """等这一步退出，交回它的退出码；跑着时每过一个进度间隔往 stderr 报一行 BROAD_MUTATION_PROGRESS（progress_line）。"""
        interval = self.progress_interval()
        next_report = started + interval
        while True:
            if break_is("progress-never"):
                return child.wait()
            try:
                return child.wait(timeout=max(0.0, next_report - time.monotonic()))
            except subprocess.TimeoutExpired:
                pass
            now = time.monotonic()
            note(progress_line(sequence, step_name, now - started, self.log_path(sequence, step_name)))
            while next_report <= now:
                next_report += interval

    def baseline_rows_of(self, step_name):
        """基线表 baselines/ 里这把基线键下这一步的行：[(写入的次序号, {列名: 值})]。线程数（--threads）只记在行里，不进键、不挑行；弄坏开关 baseline-by-threads 下只认同一个线程数的行。"""
        threads = str(self.arguments.threads)
        baseline_key = self.baseline_key_of(step_name)
        return [(number, row) for number, row in enumerate(self.library.baseline_rows(), start=1)
                if row["baseline_key"] == baseline_key and row["step"] == step_name and (row["threads"] == threads or not break_is("baseline-by-threads"))]

    def command_with_skips(self, step_name, command):
        """第二段 验证包那一步与 GPU 那一趟的测试步：登记的崩溃枚举用例与放量分项经 --skip 排掉（接在测试二进制的参数后面：命令里已有 `--` 的接在它后面，没有的补一个）；
        别的步原样。弄坏开关 registered-not-skipped 下一律原样。"""
        if step_name not in ("two-test-verification", GPU_TEST_STEP) or not self.skip_names or break_is("registered-not-skipped"):
            return command
        tail = [word for name in self.skip_names for word in ("--skip", name)]
        return list(command) + (tail if "--" in command else ["--"] + tail)

    def run_step(self, step, sequence, command=None, selection_text="all", extra=None):
        """跑一步：交 (退出码, 秒数)。输出落进这一条这一步的日志。command 不给就用步表里的。
        不经线程包装、内存包装与清场包装，不设内存上限、不限时（文件头「内存与限时」）：命令直接起，起在自己新开的进程组里（被停时照组停，见下面 Stopped 那一支）。"""
        step_name, _stage, _kind, table_command, _test_failed_exit = step
        command = self.command_with_skips(step_name, table_command if command is None else command)
        remove_tree(self.library.scratch)
        os.makedirs(self.library.scratch)
        os.makedirs(self.library.target, exist_ok=True)
        os.makedirs(self.library.logs, exist_ok=True)
        environment = dict(os.environ, CARGO_TARGET_DIR=self.library.target, TMPDIR=self.library.scratch, CARGO_TERM_COLOR="never", **(extra or {}))
        # 调用方环境里的多机配置不往下传：只有真跑测试的步照下一句设（编译步不读它，测试步收到的只能是这里设的）
        environment.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        # 真跑测试的步把多机配置交给测试（文件头「多机配置交给测试」）：专用树里它自己找不到；弄坏开关 test-steps-configuration-unpassed 下不设
        if step[2] == "test" and not break_is("test-steps-configuration-unpassed"):
            environment["SINGLEFS_MULTI_HOST_CONFIG"] = self.arguments.multi_host_configuration
        if self.fake_driver:
            environment["BROAD_MUTATION_STEP_SELECTION"] = selection_text
            argv = [os.path.join(self.selftest_work, SELFTEST_STEP_DRIVER_NAME), step_name, str(sequence), self.library.tree]
        else:
            argv = list(command)
        started = time.monotonic()
        with open(self.log_path(sequence, step_name), "wb") as log:
            child = subprocess.Popen(argv, cwd=self.library.tree, env=environment, stdin=subprocess.DEVNULL, stdout=log,
                                     stderr=subprocess.STDOUT, pass_fds=(self.library.lock_descriptor,), start_new_session=True)
            try:
                exit_code = self.wait_reporting_progress(child, step_name, sequence, started)
            except Stopped:
                # 驱动被 TERM / INT 停：在途这一步连同它的进程组一起停，不留着它跑完（文件头「中断续接」）。这一步起在自己新开的进程组里
                #   （上面 Popen 的 start_new_session），组号就是 child 的进程号：cargo 与它起的测试进程都在这一组里，只给 child 发 TERM 的话
                #   cargo 退了、测试进程照跑。发信号只点自己起的那一组：先 TERM，到 --stop-grace 秒还有没退的就 KILL；组里一个进程都不剩时不报错
                grace = self.arguments.stop_grace
                if break_is("term-leaves-step-running"):
                    child.terminate()   # 弄坏开关：只点自己起的这一个进程，组里别的进程照跑
                    try:
                        child.wait(timeout=grace)
                    except subprocess.TimeoutExpired:
                        pass
                    raise
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=grace)
                except subprocess.TimeoutExpired:
                    pass
                try:
                    os.killpg(child.pid, signal.SIGKILL)    # 组里还活着的（child 退了而它起的进程没退的也算）一并停掉
                except ProcessLookupError:
                    pass
                child.wait()
                raise
        return exit_code, time.monotonic() - started

    def ensure_baseline(self, stage):
        """这一段的每一步在这把基线键、这个线程数下、没改过的树上都跑绿过；没有就现跑。
        基线表 baselines/ 只追加，一行记基线键、步名、结局、时刻、线程数（文件头「基线」）。"""
        ran_every_step = True
        for step in self.steps:
            step_name, step_stage = step[0], step[1]
            if step_stage != stage:
                continue
            rows = self.baseline_rows_of(step_name)
            row = rows[-1][1] if rows else None
            if row is not None and row["outcome"] == "ok":
                ran_every_step = False
                continue
            plan = self.judgement.baseline_entry_plan(step, self.arguments.threads) if step_name in self.baseline_refusals else None
            if plan is not None:
                self.ensure_entry_baseline(step, stage, plan)
                continue
            if step_name in self.baseline_refusals:
                say(f"  · 基线 {step_name}：基线键拒算（{self.baseline_refusals[step_name]}），在没改过的树上照跑、不沿用")
            else:
                say(f"  · 基线 {step_name}：在没改过的树上跑（这把基线键下、这个线程数下还没有它的绿记录）")
            command = list(step[3]) + (["--no-fail-fast"] if step_name == "two-test-verification" else [])
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, command=command)
            outcome = "ok" if exit_code == 0 else f"exit-{exit_code}"
            self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, outcome, now_text(), self.arguments.threads]])
            if exit_code != 0:
                self.refuse_red_baseline(step_name, stage, exit_code)
            say(f"  · 基线 {step_name} 绿，{seconds:.1f} 秒")
        if ran_every_step:
            self.stages_with_fresh_baseline.add(stage)

    def refuse_red_baseline(self, step_name, stage, exit_code, part_text=""):
        """基线那一步在没改过的树上退了非 0（文件头「基线」：基线红了一律退 6）：第二段的另点名红的测试与已知的那一条。part_text 是按条记基线时照跑的是哪一部分。"""
        log_path = self.log_path(BASELINE_SEQUENCE, step_name)
        if stage == "two" and not break_is("stage-two-baseline-red-generic"):
            failing = []
            try:
                with open(log_path, encoding="utf-8", errors="replace") as handle:
                    for line in handle:
                        name = libtest_failed_name(line.rstrip("\n")) or nextest_failed_name(line.rstrip("\n"))
                        if name and name not in failing:
                            failing.append(name)
            except OSError:
                pass
            refuse(f"第二段基线 {step_name}{part_text} 在没改过的树上退 {exit_code}，红的测试：{'、'.join(failing[:8]) or UNRECOGNISED_TEST}；第二段一条结论都给不出",
                   "已知的一条：crates/singlefs-verification/src/crash_identity.rs 里读 .claude/gate.d/stage-inputs.tsv（导出树之外）的那条 lib 测试，"
                   "要实现员改成不读导出树之外的文件（或改门禁 crates-mutation-replay 登记的拷贝范围把登记表并进导出树），修好之后重跑同一条命令；"
                   f"第二段基线不绿，后面每一条第二段的抓到都不作数，修好之前这一道过不去；全文看日志 {log_path}",
                   EXIT_BASELINE_RED)
        refuse(f"基线 {step_name}{part_text} 在没改过的树上退 {exit_code}：基线不绿，后面每一条「抓到」都不作数，一条变异都不跑",
               f"看日志 {log_path}；代码本来就红就先修代码再重跑，是环境（缺工具、盘满）就修环境，重跑同一条命令",
               EXIT_BASELINE_RED)

    def entry_baseline_rows(self, step_name):
        """按条记基线的表 baseline-entries/ 里这一步跑绿过的记录：{(记录键, kind, rest 的二进制或空)}（线程数只记在行里，不挑行；弄坏开关 baseline-by-threads 下只认同一个线程数的行）。段数不对的行不认，那一份照跑。"""
        threads = str(self.arguments.threads)
        found = set()
        for row in self.library.read_rows(BASELINE_ENTRIES_TABLE, BASELINE_ENTRY_COLUMNS).rows:
            if row["step"] != step_name or (row["threads"] != threads and break_is("baseline-by-threads")):
                continue
            found.add((row["record_key"], row["kind"], row["binary"] if row["kind"] == "rest" else ""))
        return found

    def ensure_entry_baseline(self, step, stage, plan):
        """按条记基线（文件头「基线」）：基线键拒算的测试步，Judgement.baseline_entry_plan 拆出来的每份记录照（记录键、kind、线程数）在 baseline-entries/
        里找，找得到的沿用（不再跑），找不到的与拒算的那几条在没改过的树上照跑（nextest 按 -E 只点它们；验证包按二进制起，
        有拒算的二进制另起 --exact --skip 掉它们的一趟与只跑它们的一趟），照跑的那一趟绿了，它点到的每份记录照键追加一行；
        baselines/ 照常追加这一趟那一行（键是这一趟独有的那一把）。"""
        step_name = step[0]
        recorded = self.entry_baseline_rows(step_name)
        rules = prune_rules()

        def recorded_before(record):
            return (record["key"], record["kind"], record["binary"] if record["kind"] == "rest" else "") in recorded

        runs, fresh_records = [], []
        if plan["style"] == "nextest":
            tests, binaries = list(plan["refused"]), [item["binary"] for item in plan["whole"]]
            for record in plan["records"]:
                if recorded_before(record):
                    continue
                fresh_records.append(record)
                if record["kind"] == "test":
                    tests.append((record["binary"], record["test"]))
                else:
                    binaries.append(record["binary"])
            if len(fresh_records) == len(plan["records"]):
                runs.append({"text": "这一步全部", "command": list(step[3]), "selection": {"all": True}, "records": list(plan["records"])})
            else:
                binaries = sorted(set(binaries))
                tests = [test for test in tests if test[0] not in binaries]
                command, existing = list(step[3]), None
                if "-E" in command:
                    index = command.index("-E")
                    existing = command[index + 1]
                    del command[index:index + 2]
                filters = rules.nextest_filters(tests) + [f"binary_id(={binary})" for binary in binaries]
                if existing:
                    filters = [f"({text}) & ({existing})" for text in filters]
                command += [word for text in filters for word in ("-E", text)]
                covered = fresh_records + [record for record in plan["records"] if record["binary"] in binaries and record not in fresh_records]
                runs.append({"text": f"{len(tests)} 条测试、{len(binaries)} 个整个照跑的二进制", "command": command,
                             "selection": {"tests": [list(test) for test in tests], "binaries": binaries}, "records": covered})
        else:
            base = [word for word in step[3] if word not in ("--lib", "--tests")] + ["--no-fail-fast"]
            whole = list(plan["whole"])
            for record in plan["records"]:
                exact = {"text": f"{record['binary']} 里拒算的 {len(record['refused'])} 条（--exact）",
                         "command": base + record["selector"] + ["--", "--exact", *record["refused"]],
                         "selection": {"binaries": [record["binary"]], "part": "exact"}, "records": []}
                if recorded_before(record):
                    if record["refused"]:
                        runs.append(exact)
                    continue
                fresh_records.append(record)
                if record["refused"]:
                    runs.append({"text": f"{record['binary']} 去掉拒算的 {len(record['refused'])} 条（--exact --skip）",
                                 "command": base + record["selector"] + ["--", "--exact", *[word for test in record["refused"] for word in ("--skip", test)]],
                                 "selection": {"binaries": [record["binary"]], "part": "skip"}, "records": [record]})
                    runs.append(exact)
                else:
                    whole.append(record)
            # 整个照跑的二进制一个一趟、跑绿一趟当场记下它的那份（下面的循环）：被停、撞上限重来的那一次只补没记下的，不从头再跑整个包
            for position, item in enumerate(whole):
                runs.insert(position, {"text": f"整个照跑的二进制 {item['binary']}", "command": base + list(item["selector"]),
                                       "selection": {"binaries": [item["binary"]], "part": "whole"},
                                       "records": [item] if "key" in item else []})
        refused_names = [f"{binary} {test}" for binary, test in plan["refused"]]
        say(f"  · 基线 {step_name}：基线键拒算（{self.baseline_refusals[step_name]}），按条记基线：{len(plan['records'])} 份记录沿用 "
            f"{len(plan['records']) - len(fresh_records)}、照跑 {len(fresh_records)}；拒算的 {len(refused_names)} 条每趟照跑"
            + (f"（{'、'.join(refused_names[:5])}{'…' if len(refused_names) > 5 else ''}）" if refused_names else ""))
        total_seconds, entry_rows = 0.0, []
        for run in runs:
            selection_text = "all" if run["selection"].get("all") else "per-entry " + json.dumps(run["selection"], ensure_ascii=False, sort_keys=True)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, command=run["command"], selection_text=selection_text)
            total_seconds += seconds
            if exit_code != 0:
                self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, f"exit-{exit_code}", now_text(), self.arguments.threads]])
                self.refuse_red_baseline(step_name, stage, exit_code, f"（按条记基线照跑的那一部分：{run['text']}）")
            if run["records"]:   # 跑绿一趟当场记下它点到的那几份：之后被停、撞上限重来的那一次照键沿用
                self.library.append_rows(BASELINE_ENTRIES_TABLE,
                                         [[record["key"], step_name, self.arguments.threads, record["kind"], record["binary"], now_text()] for record in run["records"]])
            entry_rows += run["records"]
        self.library.append_rows(BASELINES_TABLE, [[self.baseline_key_of(step_name), step_name, "ok", now_text(), self.arguments.threads]])
        say(f"  · 基线 {step_name} 绿（按条记基线：照跑 {len(runs)} 趟共 {total_seconds:.1f} 秒，记下 {len(entry_rows)} 份）")

    def verify_on_pristine_tree(self, step_names, why, suspects=()):
        """在没改过的树上把这几步各跑一遍（调用时树必须是原样）：退非 0 就是环境坏了，退 6。suspects 是因此不可信的那几条的序号。"""
        for step_name in step_names:
            step = self.step_named(step_name)
            exit_code, seconds = self.run_step(step, ENVIRONMENT_SEQUENCE)
            if exit_code == 0:
                os.remove(self.log_path(ENVIRONMENT_SEQUENCE, step_name))
                continue
            distrusted = ""
            if suspects:
                distrusted = (f"；这 {len(suspects)} 条（第 {'、'.join(str(sequence) for sequence in suspects)} 条）的行是环境坏着判的、不可信："
                              "驱动没有强制重判的参数，服务里也没有删键，交主 agent 定怎么作废那几行")
            refuse(f"疑似环境坏了（{why}）：{step_name} 在没改过的树上退 {exit_code}（{seconds:.1f} 秒），基线绿过的这一步现在过不了",
                   f"看日志 {self.log_path(ENVIRONMENT_SEQUENCE, step_name)}：盘满、编译目录被清、工具链换了、scratch 写不进这一类；修好之后重跑同一条命令{distrusted}",
                   EXIT_BASELINE_RED)

    def verify_environment_at_start(self, stages):
        """每次重起：这一趟没现跑基线的那几段，先在没改过的树上把编译那几步各跑一遍。"""
        if break_is("no-environment-check"):
            return
        step_names = [step[0] for step in self.steps if step[2] == "build" and step[1] in stages and step[1] not in self.stages_with_fresh_baseline]
        if step_names:
            self.verify_on_pristine_tree(step_names, "重起先验编译")
            say(f"  · 重起先验编译：{'、'.join(step_names)} 在没改过的树上照过")


def libtest_outcome(line):
    """libtest 结果行的结局（ok / FAILED 等），不是结果行交 None（解析在 lib_libtest_result_lines.py）。"""
    found = libtest_result_lines.result_line(line)
    return found.outcome if found else None
