#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 8 那一趟：第 0–10 步按单元编排、第 11–13 步交逐行编排（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.4、8.9 节）。
由 run_segment_one_on_the_revision_8_tree_multi_host.sh 起，在仓根跑；值全从占位配置（revision_8_placeholders.json 那一形）读。

用法：
  segment_one_units_revision_8.py run  --config C
      跑。同一条命令再起就是续跑：做完的单元（序列日志有它的绿行、日志点名的产物都在且行数同、它绿那一刻写下的清单里每个文件 sha256 不变、
      它的导出目录里不多不少）跳过，没做完的整个重跑、产物名与导出目录加 -attempt<N>（N 从 2 起，上一次的半份原样留着），重跑的单元下游的单元一并重跑。
  segment_one_units_revision_8.py plan --config C
      干跑（PLAN_ONLY）：占位核对、前提核对、本机资源与槽、单元表与依赖图、按推的用时排出的次序、每个单元会跑的每一条命令（单元脚本 PRINT_ONLY=1）、
      逐行编排的 plan（各台槽数、每行推的用时）；一个装置都不起、不编、不写产物与日志。干跑配置（dry_run=true）只认这一种用法。
  segment_one_units_revision_8.py --selftest [--cells 格,…] [--rounds N]
      假单元脚本、假逐行编排、假资源、假前提走一遍各格；自证进程是收养者，每格之后收进程，末尾数遗留；逐个打开弄坏开关核红在点名格。

单元（第 8.4 节「续跑」第一条；第 9 步的 Z1 与第 10 步的 Z7 各另成一个小单元，因为它们要等两边都绿、而第 9 步装置那一趟与第 7 步同时起）：
  step-0、step-1、step-2、step-2b 依次；之后 step-3-<十项>、step-4-<十项>、step-5、step-6、step-7、step-9、step-10-run-1、step-10-run-2 同时起，
  step-8 等 step-7，step-9-z1 等 step-7 与 step-9，step-10-z7 等第 10 步两次；第 0 步判 research_side_differs=true 时另排 research-mutations（等 step-0）。
  第 11–13 步：step-0、1、2、2b、7（与 research-mutations，排了的话）都绿就起 rows_orchestrator_revision_8.py run，它与还在跑的单元共用本机，按它现查的余量分配。
次序与资源：线程额度取登记的每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，读 .claude/heavy-thread-default；
  SINGLEFS_THREAD_CAP 设了照它），不看此刻忙闲，登记读不出退 78；内存额度（内存护栏）每次起跑、续跑现查 scripts/hosts/host-resources.py --json：
  min(memory.slice_room, memory.available)；
  每个单元占 THREADS（修订 6 跑法的 10）个线程（只比对的小单元 1 个）、内存照修订 6 跑法的 CAP_LARGE 6G（流三在里面的）/ CAP_SMALL 4G，
  第 4 步每项另加编译 3 GiB（驱动在单元里冷编一次）；放得下就起，放不下的排队；队首是 step-7，其余按推的用时从长到短（推的用时只排先后，不进判定）。
  逐行编排起了之后，每起一个单元之前再现查一次本机内存，此刻可用的内存也要放得下（线程额度照旧是登记值）。此刻一个单元都放不下（本机一个都没在跑）退 78。
一步停：某个单元红，不再起新单元，并在日志目录写停派文件，逐行编排见到它不再派新行；在跑的跑完再收尾，退 1。
收到 TERM / INT / HUP：给在跑的单元（各自一个进程组）与逐行编排各发一次 TERM，等它们退出，退 143；同一条命令再起就是续跑。
第 7 步绿时往它的 -timing 产物末尾追加一行 E7TIMING name=concurrent_units_during_step_7（它跑的期间同时在跑的单元名，只描述，第 8.4 节）。

退出码：0 全部单元绿、逐行编排退 0；1 有单元红、逐行编排退 1、或前提不过（S16 ②、Z5、H7、H8、Z2）；2 用法错；
  78 运行条件不满足（占位没填或核不过、资源读不到、一个单元都放不下、逐行编排退 78）；143 收到停止信号。

弄坏开关 UNITS_R8_BREAK=<项>（只在 --selftest 起的子进程里认，UNITS_R8_SELFTEST=1；自证还认几个只换假件的变量，见 fake_world）。项名、弄坏哪一处、打开后红在哪一格：
  resume-reruns-green          续跑时做完的单元也重跑                              e2e-resume
  resume-ignores-manifest      续跑时不核清单里的 sha256                            e2e-resume-manifest-changed
  downstream-not-rerun         重跑的单元下游的单元不跟着重跑                      e2e-resume-manifest-changed
  dependency-ignored           不等依赖的单元绿就起                                e2e-order
  stop-ignored                 一个单元红了还接着起新单元                          e2e-unit-red
  rows-before-step-7           逐行编排不等第 7 步绿就起                            e2e-order
  budget-ignored               不看内存额度，线程放得下就起                        allocation-memory
  concurrency-line-missing     第 7 步绿时不追加同时在跑的单元那一行                e2e-two-rounds
  dry-run-runs                 干跑配置也照真的起跑                                e2e-dry-run-refused
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import re
import shlex
import signal
import subprocess
import sys
import time
import zoneinfo

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402

BREAK_VARIABLE = "UNITS_R8_BREAK"
SELFTEST_VARIABLE = "UNITS_R8_SELFTEST"
EXIT_RUN_CONDITION = 78
EXIT_STOPPED_BY_SIGNAL = 143
RESEARCH_RELATIVE = placeholders.RESEARCH_RELATIVE
UNIT_SCRIPT = os.path.join(HERE, "segment_one_unit_revision_8.sh")
ROWS_ORCHESTRATOR = os.path.join(HERE, "rows_orchestrator_revision_8.py")
HOSTS_DIRECTORY = os.path.join(HERE, "..", "..", "hosts")  # scripts/hosts：线程额度的登记取法 heavy_threads.py 在这里
GIB = 1 << 30
# 修订 6 跑法第 53–58 行：每个装置一趟 capped.sh 10，流三在里面的 6G、只有流一流二的 4G
THREADS = 10
CAP_SMALL, CAP_LARGE = "4G", "6G"
COMPILE_BYTES = 3 * GIB  # 第 4 步每项的驱动在单元里冷编一次（编排器文件头「资源与分配」同一个数：峰值表 2.54 GiB 向上取整）
LIGHT_BYTES = 1 * GIB
STOP_FILE = "stop-dispatching-rows"
MANIFEST = "resume-manifest.tsv"
STEP_7_CONCURRENCY_PREFIX = "E7TIMING name=concurrent_units_during_step_7 "
# 推的用时（秒），只排先后：修订 6 第 6.4 节步表（第 7、9 步 14 581.558；第 3 步带流三两项 9 698 与 8 128，其余八项合 384；第 4 步 718 秒十项；
# 第 8 步 5–30 分钟取 30；第 10 步两次共 233），没写的取几分钟（推的）
STEP_3_ITEMS = ("o1-comparison", "o1-comparison-unmount", "o2", "return-value", "memory-object", "panic", "witness", "o3-p1", "o1-crash-judge", "reference-filesystem")
STEP_4_ITEMS = ("o1-comparison", "o1-comparison-unmount", "o1-crash-judge", "reference-filesystem", "o2", "return-value", "memory-object", "panic", "witness", "o3-p1")
CHAIN = ("step-0", "step-1", "step-2", "step-2b")
ROWS_GATE = ("step-0", "step-1", "step-2", "step-2b", "step-7")


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def bytes_of(cap: str) -> int:
    return int(cap[:-1]) * GIB


@dataclasses.dataclass(frozen=True)
class Unit:
    """一个单元：deps 是要先绿的单元；threads、memory_bytes 是它在本机占的额度；estimated_seconds 只排先后；
    exports 是它写出、下游要的名字（环境变量名 → 不带 TAG 的名字，编排给下游时接上它这一次的 TAG）。"""
    name: str
    deps: tuple[str, ...]
    threads: int
    memory_bytes: int
    estimated_seconds: float
    exports: tuple[tuple[str, str], ...] = ()


def unit_table() -> dict[str, Unit]:
    after_chain = ("step-2b",)
    units = [
        Unit("step-0", (), THREADS, bytes_of(CAP_SMALL), 900),
        Unit("step-1", ("step-0",), THREADS, bytes_of(CAP_SMALL), 120, (("S2_BASELINE", "s2-device-baseline-streams-1-2"),)),
        Unit("step-2", ("step-1",), THREADS, bytes_of(CAP_SMALL), 60, (("S2_E170_EXPORT", "s2-e170-export"),)),
        Unit("step-2b", ("step-2",), THREADS, bytes_of(CAP_SMALL), 120, (("E170_SECOND_RUN", "e170-bad-states-second-run"),)),
    ]
    for item in STEP_3_ITEMS:
        stream_three = item in ("o1-crash-judge", "reference-filesystem")
        seconds = {"o1-crash-judge": 9698, "reference-filesystem": 8128}.get(item, 48)
        units.append(Unit(f"step-3-{item}", after_chain, THREADS, bytes_of(CAP_LARGE if stream_three else CAP_SMALL), seconds))
    for item in STEP_4_ITEMS:
        units.append(Unit(f"step-4-{item}", after_chain, THREADS, bytes_of(CAP_SMALL) + COMPILE_BYTES, 72))
    units += [
        Unit("step-5", after_chain, THREADS, bytes_of(CAP_SMALL), 300),
        Unit("step-6", after_chain, THREADS, bytes_of(CAP_SMALL), 100),
        Unit("step-7", after_chain, THREADS, bytes_of(CAP_LARGE), 14582, (("BASELINE_RUN1", "baseline-run1"),)),
        Unit("step-8", ("step-7",), THREADS, bytes_of(CAP_LARGE), 1800),
        Unit("step-9", after_chain, THREADS, bytes_of(CAP_LARGE), 14582, (("BASELINE_RUN2", "baseline-run2"),)),
        Unit("step-9-z1", ("step-7", "step-9"), 1, LIGHT_BYTES, 60),
        Unit("step-10-run-1", after_chain, THREADS, bytes_of(CAP_SMALL), 117, (("Q5_MAIN_RUN_1", "q5-main-before-the-first-barrier-of-call-3-run-1"),)),
        Unit("step-10-run-2", after_chain, THREADS, bytes_of(CAP_SMALL), 117, (("Q5_MAIN_RUN_2", "q5-main-before-the-first-barrier-of-call-3-run-2"),)),
        Unit("step-10-z7", ("step-10-run-1", "step-10-run-2"), 1, LIGHT_BYTES, 5),
        Unit("research-mutations", ("step-0",), THREADS, 2 * GIB, 600),
    ]
    return {unit.name: unit for unit in units}


def downstream(units: dict[str, Unit], names: set[str]) -> set[str]:
    """names 连同依赖它们的全部单元（传递）。"""
    found = set(names)
    changed = True
    while changed:
        changed = False
        for unit in units.values():
            if unit.name not in found and any(dep in found for dep in unit.deps):
                found.add(unit.name)
                changed = True
    return found


def queue_order(units: list[Unit]) -> list[Unit]:
    """step-7 最先，其余按推的用时从长到短，平手按名字。"""
    return sorted(units, key=lambda unit: (unit.name != "step-7", -unit.estimated_seconds, unit.name))


def attempt_tag(attempt: int) -> str:
    return "" if attempt == 1 else f"-attempt{attempt}"


@dataclasses.dataclass
class Settings:
    repo: str
    config_path: str
    config: dict
    tree: str
    run_date: str
    suffix: str
    dumps: str
    logs: str
    snapshot_digest: str
    mutations_sha256: str
    q5_reverse_states: str
    q5_main_states: str
    dry_run: bool

    @property
    def prefix(self) -> str:
        return placeholders.product_prefix(self.run_date)

    def product(self, stem: str) -> str:
        return f"{self.prefix}-{stem}{self.suffix}.out"

    def timing(self, stem: str) -> str:
        return f"{self.prefix}-{stem}{self.suffix}-timing.out"


def settings_from_config(repo: str, config_path: str) -> Settings:
    config = judge.load_config(config_path)
    h16, h14 = judge.section(config, "H16"), judge.section(config, "H14")
    return Settings(repo=repo, config_path=os.path.abspath(config_path), config=config, tree=judge.tree_of(config),
                    run_date=str(h16.get("run_date") or "unfilled"), suffix=str(h16.get("run_suffix") or "-r8"),
                    dumps=str(h16.get("dumps_root") or "scripts/experiment-apparatus/target/e171-r1-dumps/unfilled"),
                    logs=str(h16.get("logs_directory") or os.path.join(placeholders.WORK_ROOT, "e171-r1-revision-8-unfilled-logs")),
                    snapshot_digest=str(judge.section(config, "H3").get("snapshot_digest") or "unfilled"),
                    mutations_sha256=str(judge.section(config, "H8").get("mutations_table_sha256") or "unfilled"),
                    q5_reverse_states=str(h14.get("reverse_states") or "unfilled"), q5_main_states=str(h14.get("main_states") or "unfilled"),
                    dry_run=bool(config.get("dry_run")))


def today() -> str:
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).date().isoformat()


def say(settings: Settings, text: str, quiet: bool = False) -> None:
    line = f"{today()} {text}"
    print(line, flush=True)
    if not quiet:
        os.makedirs(settings.logs, exist_ok=True)
        with open(os.path.join(settings.logs, "sequence.log"), "a", encoding="utf-8") as handle:
            handle.write(line + "\n")


# ── 续跑：哪些单元做完了 ──
GREEN_LINE = re.compile(r"✓ 单元 (\S+) 绿 attempt=(\d+) tag=(\S+)(.*)$")
PRODUCT_IN_LINE = re.compile(r" 产物 (\S+?)（(\d+) 行）")


def sha256_of_file(path: str) -> str:
    return judge.sha256_of_file(path)


def line_count(path: str) -> int:
    with open(path, "rb") as handle:
        return handle.read().count(b"\n")


def attempts_started(settings: Settings, unit: str) -> list[int]:
    directory = os.path.join(settings.logs, "units")
    found = [int(match.group(1)) for name in (os.listdir(directory) if os.path.isdir(directory) else [])
             for match in [re.fullmatch(re.escape(unit) + r"-attempt-(\d+)", name)] if match]
    return sorted(found)


def green_records(settings: Settings) -> dict[str, tuple[int, str, list[tuple[str, int]]]]:
    """序列日志里每个单元最后一条绿行：单元 → (第几次, TAG, [(产物, 行数)])。"""
    path = os.path.join(settings.logs, "sequence.log")
    records: dict[str, tuple[int, str, list[tuple[str, int]]]] = {}
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        match = GREEN_LINE.search(line.rstrip("\n"))
        if match:
            tag = "" if match.group(3) == "-" else match.group(3)
            records[match.group(1)] = (int(match.group(2)), tag, [(item[0], int(item[1])) for item in PRODUCT_IN_LINE.findall(match.group(4))])
    return records


def manifest_entries(settings: Settings) -> dict[tuple[str, int], list[tuple[str, str, str]]]:
    """清单：(单元, 第几次) → [(sha256, 路径, 种类 product|directory:<目录>)]。"""
    path = os.path.join(settings.logs, MANIFEST)
    entries: dict[tuple[str, int], list[tuple[str, str, str]]] = {}
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        unit, attempt, digest, relative, kind = line.rstrip("\n").split("\t")
        entries.setdefault((unit, int(attempt)), []).append((digest, relative, kind))
    return entries


def files_under(repo: str, directory: str) -> list[str]:
    found = []
    for root, _, files in os.walk(os.path.join(repo, directory)):
        found += [os.path.relpath(os.path.join(root, name), repo) for name in files]
    return sorted(found)


def unit_problems(settings: Settings, unit: str, records, manifest) -> list[str]:
    """这个单元算不算做完：交回不成立的那几样（空表就是做完了）。"""
    if unit not in records:
        return ["no_green_line"]
    attempt, _, products = records[unit]
    problems = []
    for relative, lines in products:
        path = os.path.join(settings.repo, relative)
        if not os.path.isfile(path):
            problems.append(f"product_missing:{relative}")
        elif line_count(path) != lines:
            problems.append(f"product_line_count:{relative}")
    entries = manifest.get((unit, attempt), [])
    if not entries:
        problems.append("no_manifest_entries")
    if not broken("resume-ignores-manifest"):
        for digest, relative, kind in entries:
            if kind == "nothing":  # 只做比对、不写产物的小单元（Z1、Z7）绿时记的一行
                continue
            path = os.path.join(settings.repo, relative)
            if not os.path.isfile(path) or sha256_of_file(path) != digest:
                problems.append(f"manifest_sha256:{relative}")
        listed = {relative for _, relative, _ in entries}
        for directory in sorted({kind.split(":", 1)[1] for _, _, kind in entries if kind.startswith("directory:")}):
            problems += [f"file_not_in_manifest:{relative}" for relative in files_under(settings.repo, directory) if relative not in listed]
    return problems


def resume_state(settings: Settings, units: dict[str, Unit]) -> tuple[dict[str, tuple[int, str]], dict[str, list[str]]]:
    """交回 (做完的单元 → (第几次, TAG), 没做完的单元 → 不成立的那几样)。没做完的单元下游一并算没做完（它们吃的输入要换）。"""
    records, manifest = green_records(settings), manifest_entries(settings)
    done, not_done = {}, {}
    for name in units:
        problems = unit_problems(settings, name, records, manifest)
        if broken("resume-reruns-green"):
            problems = problems or ["rerun_by_break_switch"]
        if problems:
            not_done[name] = problems
        else:
            done[name] = records[name][:2]
    if not broken("downstream-not-rerun"):
        for name in downstream(units, set(not_done)) - set(not_done):
            if name in done:
                del done[name]
                not_done[name] = ["upstream_rerun"]
    return done, not_done


def next_attempt(settings: Settings, unit: str) -> int:
    return max(attempts_started(settings, unit), default=0) + 1


# ── 假件替换（只在自证里认）、资源、前提 ──
def override(name: str, default: str) -> str:
    return os.environ.get(name, default) if os.environ.get(SELFTEST_VARIABLE) == "1" else default


class RunCondition(Exception):
    """运行条件不满足：退 78。"""


@dataclasses.dataclass(frozen=True)
class Budget:
    threads: int
    memory_bytes: int


def registered_threads() -> int:
    """线程额度（文件头「次序与资源」）：登记的每台默认重活线程数，不看此刻忙闲；登记读不出抛 RunCondition（退 78）。"""
    if HOSTS_DIRECTORY not in sys.path:
        sys.path.insert(0, HOSTS_DIRECTORY)
    import heavy_threads  # noqa: E402（现用现引：只在算额度时要它）
    try:
        return heavy_threads.default_heavy_threads(os.environ)
    except heavy_threads.HeavyThreadDefaultError as error:
        raise RunCondition(f"线程额度的登记读不出：{error}（{error.howto}）") from error


def idle_budget(settings: Settings) -> Budget | None:
    """干跑里另排一遍用的「本机空着时」的额度：登记的线程额度与 min(slice 总量, 内存总量)（只给人看挂钟，起跑不用它）；读不到交回 None。"""
    command = override("UNITS_R8_RESOURCES_COMMAND", "python3 scripts/hosts/host-resources.py --json")
    completed = subprocess.run(["bash", "-c", command], cwd=settings.repo, capture_output=True, text=True)
    hosts = [host for host in json.loads(completed.stdout).get("hosts", []) if host.get("number") == 1] if completed.returncode == 0 else []
    if not hosts or not hosts[0].get("read"):
        return None
    memory = hosts[0]["memory"]
    return Budget(registered_threads(), min(int(memory.get("slice_total", 0)), int(memory.get("mem_total", 0))))


def local_budget(settings: Settings) -> Budget:
    command = override("UNITS_R8_RESOURCES_COMMAND", "python3 scripts/hosts/host-resources.py --json")
    completed = subprocess.run(["bash", "-c", command], cwd=settings.repo, capture_output=True, text=True)
    if completed.returncode != 0:
        raise RunCondition(f"host-resources.py --json 退 {completed.returncode}")
    hosts = [host for host in json.loads(completed.stdout).get("hosts", []) if host.get("number") == 1]
    if not hosts or not hosts[0].get("read"):
        raise RunCondition("本机的资源没读到")
    memory = hosts[0]["memory"]
    memory_bytes = (1 << 60) if broken("budget-ignored") else min(int(memory["slice_room"]), int(memory["available"]))
    return Budget(registered_threads(), memory_bytes)


def fits(unit: Unit, running: list[Unit], budget: Budget) -> bool:
    threads = sum(item.threads for item in running) + unit.threads
    memory = sum(item.memory_bytes for item in running) + unit.memory_bytes
    return threads <= budget.threads and memory <= budget.memory_bytes


def prerequisite_commands(settings: Settings) -> list[tuple[str, list[str]]]:
    """每次起跑与续跑都核：占位（start 阶段）、第 0 步前提（H3、H7、S16 ②、H8）、O3 冻结行（Z2）。"""
    replaced = override("UNITS_R8_PREREQUISITES_COMMAND", "")
    if replaced:
        return [("selftest", ["bash", "-c", replaced])]
    research = os.path.join(settings.repo, RESEARCH_RELATIVE)
    return [("placeholders", ["python3", "-B", os.path.join(research, "revision_8_placeholders.py"), "check", "--config", settings.config_path,
                              "--phase", "start", "--repo", settings.repo]),
            ("frozen_tree_prerequisites", ["python3", "-B", os.path.join(research, "frozen_tree_lines_revision_8.py"), "prerequisites", "--config",
                                           settings.config_path, "--repo", settings.repo]),
            ("o3_freeze_z2", ["python3", "-B", "-c", f"import sys; sys.path.insert(0, {research!r}); import frozen_o3; "
                                                     f"differences = frozen_o3.freeze_differences({settings.repo!r}); print('o3_freeze_differences', len(differences)); "
                                                     "sys.exit(1 if differences else 0)"])]


def check_prerequisites(settings: Settings, quiet: bool) -> tuple[bool, int]:
    """交回 (都过没有, 不过时的退出码：占位 78、别的 1)。"""
    for name, command in prerequisite_commands(settings):
        completed = subprocess.run(command, cwd=settings.repo, capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        for line in (completed.stdout + completed.stderr).splitlines():
            say(settings, f"  前提 {name}：{line}", quiet)
        if completed.returncode != 0:
            say(settings, f"  前提 {name} 不过（退 {completed.returncode}）", quiet)
            return False, EXIT_RUN_CONDITION if completed.returncode == EXIT_RUN_CONDITION else 1
    return True, 0


# ── 起一个单元、收一个单元 ──
@dataclasses.dataclass
class Running:
    unit: Unit
    attempt: int
    process: subprocess.Popen
    directory: str
    started: float
    ended: float | None = None


def unit_environment(settings: Settings, unit: Unit, attempt: int, directory: str, names: dict[str, str], print_only: bool) -> dict[str, str]:
    h7 = judge.section(settings.config, "H7")
    tree = settings.tree
    environment = dict(os.environ, REPO=settings.repo, RESEARCH=RESEARCH_RELATIVE, PREFIX=settings.prefix, SUFFIX=settings.suffix, TAG=attempt_tag(attempt),
                       DUMPS=settings.dumps, TREE=tree, BIN=f"{tree}/target/release/{judge.E171_BINARY}", E170_BIN=f"{tree}/target/release/{judge.E170_BINARY}",
                       SNAPSHOT_DIGEST=settings.snapshot_digest, MUTATIONS_SHA256=settings.mutations_sha256, CONFIG=settings.config_path,
                       THREADS=str(THREADS), CAP_SMALL=CAP_SMALL, CAP_LARGE=CAP_LARGE, WORK_ROOT=os.path.join(settings.logs, "work", f"{unit.name}-attempt-{attempt}"),
                       UNIT_LOG=os.path.join(directory, "unit.log"), UNIT_PRODUCTS=os.path.join(directory, "products.tsv"),
                       UNIT_DIRECTORIES=os.path.join(directory, "directories.tsv"), Q5_REVERSE_STATES=settings.q5_reverse_states,
                       Q5_MAIN_STATES=settings.q5_main_states, PRINT_ONLY="1" if print_only else "0", PYTHONDONTWRITEBYTECODE="1",
                       H7_E171=str(h7.get("e171_binary_sha256")), H7_E170=str(h7.get("e170_binary_sha256")))
    environment.update(names)
    return environment


def exported_names(units: dict[str, Unit], done: dict[str, tuple[int, str]]) -> dict[str, str]:
    """下游要的名字：做完的单元的 exports 接上它绿的那一次的 TAG。"""
    names = {}
    for name, (attempt, tag) in done.items():
        for variable, stem in units[name].exports:
            names[variable] = stem + tag
    return names


def launch(settings: Settings, unit: Unit, attempt: int, names: dict[str, str]) -> Running:
    directory = os.path.join(settings.logs, "units", f"{unit.name}-attempt-{attempt}")
    os.makedirs(directory)
    environment = unit_environment(settings, unit, attempt, directory, names, False)
    script = override("UNITS_R8_UNIT_SCRIPT", UNIT_SCRIPT)
    with open(os.path.join(directory, "stdout"), "x", encoding="utf-8") as out, open(os.path.join(directory, "stderr"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(["bash", script, unit.name], cwd=settings.repo, stdout=out, stderr=err, env=environment, start_new_session=True)
    say(settings, f"▶ 单元 {unit.name} 起（第 {attempt} 次{'，TAG ' + attempt_tag(attempt) if attempt > 1 else ''}）：{unit.threads} 线程、{unit.memory_bytes // GIB} GiB")
    return Running(unit, attempt, process, directory, time.monotonic())


def read_list(path: str) -> list[str]:
    items = []
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        item = line.rstrip("\n").split("\t")[0]
        if item and item not in items:
            items.append(item)
    return items


def concurrency_line(item: Running, intervals: dict[str, tuple[float, float | None]]) -> str:
    """第 7 步跑的期间同时在跑的单元（区间有交叠的），只描述。"""
    start, end = item.started, item.ended or time.monotonic()
    overlapping = sorted(name for name, (other_start, other_end) in intervals.items()
                         if name != item.unit.name and other_start < end and (other_end is None or other_end > start))
    return f"{STEP_7_CONCURRENCY_PREFIX}units={','.join(overlapping) or 'none'} judged=false\n"


def finish_unit(settings: Settings, item: Running, intervals: dict[str, tuple[float, float | None]]) -> bool:
    """一个单元退出：红就记一行交回 False；绿就（第 7 步先追加同时在跑的那一行）写清单、记绿行，交回 True。"""
    code = item.process.returncode
    tag = attempt_tag(item.attempt)
    if code != 0:
        say(settings, f"✗ 单元 {item.unit.name} 红（第 {item.attempt} 次，退 {code}；日志 {os.path.join(item.directory, 'unit.log')}）")
        return False
    products = read_list(os.path.join(item.directory, "products.tsv"))
    directories = read_list(os.path.join(item.directory, "directories.tsv"))
    if item.unit.name == "step-7" and not broken("concurrency-line-missing"):
        with open(os.path.join(settings.repo, settings.timing(f"baseline-run1{tag}")), "a", encoding="utf-8") as handle:
            handle.write(concurrency_line(item, intervals))
    lines = []
    for relative in products:
        lines.append(f"{item.unit.name}\t{item.attempt}\t{sha256_of_file(os.path.join(settings.repo, relative))}\t{relative}\tproduct\n")
    for directory in directories:
        for relative in files_under(settings.repo, directory):
            lines.append(f"{item.unit.name}\t{item.attempt}\t{sha256_of_file(os.path.join(settings.repo, relative))}\t{relative}\tdirectory:{directory}\n")
    if not lines:
        lines.append(f"{item.unit.name}\t{item.attempt}\t{'0' * 64}\t-\tnothing\n")
    with open(os.path.join(settings.logs, MANIFEST), "a", encoding="utf-8") as handle:
        handle.writelines(lines)
        handle.flush()
        os.fsync(handle.fileno())
    named = "".join(f" 产物 {relative}（{line_count(os.path.join(settings.repo, relative))} 行）" for relative in products)
    say(settings, f"✓ 单元 {item.unit.name} 绿 attempt={item.attempt} tag={tag or '-'}{named}")
    return True


def research_side_differs(settings: Settings, done: dict[str, tuple[int, str]]) -> bool:
    """第 0 步产物的 verdict 行 research_side_differs（没读到按 true：多跑一趟研究一侧变异表只收严）。"""
    if "step-0" not in done:
        return True
    path = os.path.join(settings.repo, settings.product(f"frozen-tree{done['step-0'][1]}"))
    return judge.verdict_fields(path).get("research_side_differs", "true") != "false" if os.path.isfile(path) else True


def rows_command(settings: Settings, done: dict[str, tuple[int, str]], mode: str) -> list[str]:
    script = override("UNITS_R8_ROWS_ORCHESTRATOR", ROWS_ORCHESTRATOR)
    tag = done.get("step-7", (1, ""))[1]
    interpreter = ["bash"] if script.endswith(".sh") else ["python3", "-B"]  # 自证的假逐行编排是 bash
    return ["nice", "-n", "19"] + interpreter + [script, mode, "--config", settings.config_path, "--repo", settings.repo,
            "--baseline-timing", settings.timing(f"baseline-run1{tag}"), "--stop-file", os.path.join(settings.logs, STOP_FILE)]


STOP_REQUESTED = {"signal": None}


def on_signal(signal_number: int, frame) -> None:  # noqa: ARG001
    STOP_REQUESTED["signal"] = signal_number


def terminate(process: subprocess.Popen) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass


def poll_seconds() -> float:
    return float(override("UNITS_R8_POLL_SECONDS", "10"))


def run(settings: Settings) -> int:
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, on_signal)
    if settings.dry_run and not broken("dry-run-runs"):
        raise RunCondition(f"这是干跑配置（{settings.config.get('dry_run_note') or '没写说明'}），只给 plan 用")
    os.makedirs(settings.logs, exist_ok=True)
    say(settings, f"开跑：配置 {settings.config_path}（sha256 {sha256_of_file(settings.config_path)[:16]}…），树 {settings.tree}，产物 {settings.prefix}-*{settings.suffix}，"
                  f"导出 {settings.dumps}，日志 {settings.logs}")
    ok, code = check_prerequisites(settings, quiet=False)
    if not ok:
        return code
    stop_file = os.path.join(settings.logs, STOP_FILE)
    if os.path.exists(stop_file):
        os.remove(stop_file)
        say(settings, "  上一趟留下的停派文件删了（这一趟重新起）")
    units = unit_table()
    budget = local_budget(settings)
    say(settings, f"  本机额度：{budget.threads} 线程（登记值）、{budget.memory_bytes / GIB:.1f} GiB（内存每次起跑、续跑现查）")
    done, not_done = resume_state(settings, units)
    for name in sorted(done):
        say(settings, f"  复用 {name}（第 {done[name][0]} 次绿的那一份，清单核过）")
    for name in sorted(not_done):
        if not_done[name] != ["no_green_line"]:
            say(settings, f"  {name} 要重跑：{','.join(not_done[name][:5])}")
    pending = {name for name in not_done}
    running: list[Running] = []
    intervals: dict[str, tuple[float, float | None]] = {}
    failures: list[str] = []
    orchestrator: subprocess.Popen | None = None
    orchestrator_exit: int | None = None
    last_requery, fresh = 0.0, budget
    while True:
        if STOP_REQUESTED["signal"] is not None:
            say(settings, "收到停止信号：给在跑的单元与逐行编排各发一次 TERM，等它们退出")
            for item in running:
                terminate(item.process)
            if orchestrator is not None and orchestrator.poll() is None:
                terminate(orchestrator)
            for item in running:
                item.process.wait()
            if orchestrator is not None:
                orchestrator.wait()
            return EXIT_STOPPED_BY_SIGNAL
        for item in list(running):
            if item.process.poll() is None:
                continue
            running.remove(item)
            item.ended = time.monotonic()
            intervals[item.unit.name] = (item.started, item.ended)
            if finish_unit(settings, item, intervals):
                done[item.unit.name] = (item.attempt, attempt_tag(item.attempt))
                pending.discard(item.unit.name)
            else:
                failures.append(item.unit.name)
                pending.discard(item.unit.name)  # 这一趟里不再重起它；同一条命令再起时它照没做完的算、带下一次的 TAG 重跑
                with open(stop_file, "w", encoding="utf-8") as handle:
                    handle.write(f"单元 {item.unit.name} 红\n")
        if orchestrator is not None and orchestrator_exit is None and orchestrator.poll() is not None:
            orchestrator_exit = orchestrator.returncode
            say(settings, f"  逐行编排退 {orchestrator_exit}")
        required = set(units) - ({"research-mutations"} if not research_side_differs(settings, done) else set())
        gate = set(CHAIN) if broken("rows-before-step-7") else set(ROWS_GATE) | ({"research-mutations"} & required)
        if orchestrator is None and not failures and gate <= set(done):
            orchestrator = subprocess.Popen(rows_command(settings, done, "run"), cwd=settings.repo, start_new_session=True,
                                            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
            say(settings, f"▶ 第 11–13 步：逐行编排起（{', '.join(sorted(gate))} 都绿）")
        stopping = bool(failures) or orchestrator_exit not in (None, 0)
        if not stopping or broken("stop-ignored"):
            ready = [units[name] for name in pending & required if name not in {item.unit.name for item in running}
                     and (broken("dependency-ignored") or all(dep in done for dep in units[name].deps))]
            for unit in queue_order(ready):
                if orchestrator is not None and orchestrator.poll() is None and time.monotonic() - last_requery > float(override("UNITS_R8_REQUERY_SECONDS", "60")):
                    last_requery = time.monotonic()
                    try:
                        fresh = local_budget(settings)
                    except RunCondition as error:  # 单元在跑时读不到资源：不起新单元、等下一次再查，不丢下在跑的
                        say(settings, f"  现查本机没读到（{error}），这一轮不起新单元")
                        fresh = Budget(0, 0)
                if not fits(unit, [item.unit for item in running], budget):
                    continue
                if orchestrator is not None and orchestrator.poll() is None and not fits(unit, [], fresh):
                    continue
                attempt = next_attempt(settings, unit.name)
                item = launch(settings, unit, attempt, exported_names(units, done))
                running.append(item)
                intervals[unit.name] = (item.started, None)
        orchestrator_finished = orchestrator is None or orchestrator.poll() is not None
        if not running and orchestrator_finished:
            if stopping:
                break
            if required <= set(done) and orchestrator_exit is not None:
                break
            ready = [name for name in pending & required if all(dep in done for dep in units[name].deps)]
            if ready:
                raise RunCondition(f"此刻一个单元都放不下（本机额度 {budget.threads} 线程、{budget.memory_bytes / GIB:.1f} GiB；排着的 {','.join(sorted(ready))}）")
            if not ready and not required <= set(done) and orchestrator is None:
                raise RunCondition("还有单元没做完、却没有一个依赖齐了（单元表有环或依赖缺）")
        time.sleep(poll_seconds())
    if failures:
        say(settings, f"✗ 停：单元 {','.join(failures)} 红；在跑的都已收尾，做完的留着，同一条命令再起只补没做完的")
        return 1
    if orchestrator_exit != 0:
        say(settings, f"✗ 停在第 11–13 步：逐行编排退 {orchestrator_exit}（78 是运行条件，143 是被停，1 是停机或某行没跑成）")
        return EXIT_RUN_CONDITION if orchestrator_exit == EXIT_RUN_CONDITION else (EXIT_STOPPED_BY_SIGNAL if orchestrator_exit == EXIT_STOPPED_BY_SIGNAL else 1)
    say(settings, "✓ 第 0–13 步全部跑完。比对器（segment_one_comparator.py）与两趟对照（contrast_revision_6_against_revision_8.py）归下一段")
    return 0


# ── 干跑（PLAN_ONLY）：只打分配、次序、依赖图与各单元的命令 ──
def simulate(units: dict[str, Unit], names: list[str], budget: Budget) -> list[tuple[str, float, float]]:
    """按推的用时在本机额度里排一遍（同 run 的放法：依赖齐了、放得下就起，队首 step-7、其余最长先）。交回 [(单元, 起, 止)]，放不下的不在表里。"""
    finished: dict[str, float] = {}
    active: list[tuple[str, float, float]] = []
    placed: list[tuple[str, float, float]] = []
    clock, remaining = 0.0, set(names)
    while remaining or active:
        ready = [units[name] for name in remaining if all(dep in finished for dep in units[name].deps if dep in names)
                 and all(finished.get(dep, 0.0) <= clock for dep in units[name].deps if dep in names)]
        for unit in queue_order(ready):
            if fits(unit, [units[name] for name, _, _ in active], budget):
                active.append((unit.name, clock, clock + unit.estimated_seconds))
                placed.append(active[-1])
                remaining.discard(unit.name)
        if not active:
            break
        clock = min(end for _, _, end in active)
        for entry in [entry for entry in active if entry[2] <= clock]:
            active.remove(entry)
            finished[entry[0]] = entry[2]
    return placed


def plan(settings: Settings) -> int:
    print(f"干跑（PLAN_ONLY）：配置 {settings.config_path}（dry_run={str(settings.dry_run).lower()}；{settings.config.get('dry_run_note') or '-'}）")
    print(f"树 {settings.tree}；产物 {settings.prefix}-<步名>{settings.suffix}.out；导出 {settings.dumps}；日志 {settings.logs}")
    print("── 占位与前提（只核、不停）──")
    check_prerequisites(settings, quiet=True)
    print("── 本机资源与额度 ──")
    budget = local_budget(settings)
    print(f"本机额度：{budget.threads} 线程、{budget.memory_bytes / GIB:.1f} GiB；每个单元 {THREADS} 线程（Z1、Z7 两个小单元 1 线程），"
          f"内存 {CAP_LARGE}（流三在里面的）/ {CAP_SMALL}，第 4 步每项另加编译 {COMPILE_BYTES // GIB} GiB")
    units = unit_table()
    done, not_done = resume_state(settings, units)
    research = research_side_differs(settings, done)
    names = [name for name in units if research or name != "research-mutations"]
    print(f"── 单元表（{len(names)} 个；做完的 {len(done)} 个）──")
    for name in names:
        unit = units[name]
        state = f"做完（第 {done[name][0]} 次）" if name in done else f"要跑（第 {next_attempt(settings, name)} 次）"
        print(f"单元 {name}：依赖 {','.join(unit.deps) or '无'}；{unit.threads} 线程；{unit.memory_bytes / GIB:.0f} GiB；推的用时 {unit.estimated_seconds:.0f} 秒；{state}")
    print("── 依赖图（边：先 → 后）──")
    for name in names:
        for dep in units[name].deps:
            print(f"{dep} → {name}")
    print(f"{' + '.join(ROWS_GATE)}{' + research-mutations' if research else ''} → 第 11–13 步（rows_orchestrator_revision_8.py）")
    idle = idle_budget(settings)
    for title, room in (("此刻的本机额度（登记线程、现查内存）", budget), ("本机空着时（登记线程、slice 总量；只给人看挂钟）", idle)):
        print(f"── 按推的用时在{title}里排（只排先后，不进判定）──")
        placed = simulate(units, [name for name in names if name not in done], room) if room else []
        for name, start, end in sorted(placed, key=lambda entry: (entry[1], entry[0])):
            print(f"{start / 3600:6.2f} h 起 → {end / 3600:6.2f} h 止  {name}")
        print(f"排进去 {len(placed)} 个（要 {len([name for name in names if name not in done])} 个）" + (f"；额度 {room.threads} 线程、{room.memory_bytes / GIB:.1f} GiB" if room else "；读不到"))
    print("── 各单元的命令（单元脚本 PRINT_ONLY=1）──")
    predicted = dict(done)
    for name in names:
        if name not in predicted:
            predicted[name] = (next_attempt(settings, name), attempt_tag(next_attempt(settings, name)))
    for name in names:
        if name in done:
            continue
        attempt = predicted[name][0]
        environment = unit_environment(settings, units[name], attempt, os.path.join(settings.logs, "units", f"{name}-attempt-{attempt}"),
                                       exported_names(units, predicted), True)
        completed = subprocess.run(["bash", override("UNITS_R8_UNIT_SCRIPT", UNIT_SCRIPT), name], cwd=settings.repo, env=environment, capture_output=True, text=True)
        print(f"单元 {name}（第 {attempt} 次）：")
        print(completed.stdout.rstrip("\n") + (f"\n  ✗ 单元脚本退 {completed.returncode}：{completed.stderr.strip()}" if completed.returncode else ""))  # gate-lint:detail
    print("── 第 11–13 步：逐行编排的 plan ──")
    command = rows_command(settings, predicted, "plan")
    print("$ " + " ".join(shlex.quote(word) for word in command))
    completed = subprocess.run(command, cwd=settings.repo, capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    print(completed.stdout.rstrip("\n"))
    if completed.returncode:
        print(f"逐行编排 plan 退 {completed.returncode}：{completed.stderr.strip()}")
    return 0


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 8 那一趟：第 0–10 步按单元编排、第 11–13 步交逐行编排")
    parser.add_argument("mode", choices=("run", "plan"))
    parser.add_argument("--config", required=True)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    try:
        settings = settings_from_config(os.path.abspath(arguments.repo), arguments.config)
        return plan(settings) if arguments.mode == "plan" else run(settings)
    except (RunCondition, judge.RunConditionError) as error:
        print(f"{today()} ✗ 运行条件不满足（退 78）：{error}", flush=True)
        return EXIT_RUN_CONDITION


# ── 自证：假单元脚本、假逐行编排、假资源都写在格的临时目录里，编排本身起子进程真跑 ──
FAKE_UNIT_SCRIPT = r'''#!/bin/bash
set -u
unit=$1
if [ "${PRINT_ONLY:-0}" = 1 ]; then echo "  \$ fake-device $unit S2_BASELINE=${S2_BASELINE:-unset} BASELINE_RUN1=${BASELINE_RUN1:-unset}"; exit 0; fi
echo "start $unit ${TAG:--} $(date +%s.%N)" >> "$FAKE_LOG"
seconds=${FAKE_SECONDS:-0.05}; variable="FAKE_SECONDS_${unit//-/_}"; [ -n "${!variable:-}" ] && seconds=${!variable}
sleep "$seconds"
mkdir -p "$DUMPS/$unit$TAG" && echo "$unit" > "$DUMPS/$unit$TAG/data" && echo "$DUMPS/$unit$TAG" >> "$UNIT_DIRECTORIES"
product="$PREFIX-$unit$TAG$SUFFIX.out"
case $unit in
  step-0) product="$PREFIX-frozen-tree$TAG$SUFFIX.out"; printf 'E7RESULT name=verdict research_side_differs=%s triggered=none\n' "${FAKE_RESEARCH_DIFFERS:-false}" > "$product" ;;
  step-7) timing="$PREFIX-baseline-run1$TAG$SUFFIX-timing.out"; printf 'E7TIMING name=process_wall seconds=1 exit=0\n' > "$timing"
          printf '%s\t1\n' "$timing" >> "$UNIT_PRODUCTS"; echo "E7RESULT name=x unit=$unit" > "$product" ;;
  *) echo "E7RESULT name=x unit=$unit" > "$product" ;;
esac
printf '%s\t1\n' "$product" >> "$UNIT_PRODUCTS"
case $unit in step-2|step-3-*|step-4-*|step-5) [ -n "${S2_BASELINE:-}" ] || { echo "end $unit ${TAG:--} $(date +%s.%N) missing-S2_BASELINE" >> "$FAKE_LOG"; exit 9; } ;; esac
case $unit in step-8|step-9-z1) [ -n "${BASELINE_RUN1:-}" ] || { echo "end $unit ${TAG:--} $(date +%s.%N) missing-BASELINE_RUN1" >> "$FAKE_LOG"; exit 9; } ;; esac
if [ "${FAKE_RED:-}" = "$unit" ]; then echo "end $unit ${TAG:--} $(date +%s.%N) red" >> "$FAKE_LOG"; exit 1; fi
echo "end $unit ${TAG:--} $(date +%s.%N)" >> "$FAKE_LOG"
'''
FAKE_ROWS = r'''#!/bin/bash
mode=$1
echo "rows-$mode start $(date +%s.%N)" >> "$FAKE_LOG"
if [ "$mode" = plan ]; then echo "fake rows plan"; exit 0; fi
sleep "${FAKE_ROWS_SECONDS:-0.1}"
echo "rows-$mode end $(date +%s.%N)" >> "$FAKE_LOG"
exit "${FAKE_ROWS_EXIT:-0}"
'''


@dataclasses.dataclass
class FakeWorld:
    root: str
    repo: str
    config: str
    log: str
    environment: dict[str, str]

    def command(self, mode: str = "run") -> list[str]:
        return [sys.executable, "-B", os.path.abspath(__file__), mode, "--config", self.config, "--repo", self.repo]


def fake_world(root: str, threads: int = 30, room_gib: float = 40, extra: dict[str, str] | None = None, dry_run: bool = False) -> FakeWorld:
    repo = os.path.join(root, "repo")
    os.makedirs(os.path.join(repo, "research/results"))
    unit_script = judge.write_text(os.path.join(root, "fake-unit.sh"), FAKE_UNIT_SCRIPT)
    rows = judge.write_text(os.path.join(root, "fake-rows.sh"), FAKE_ROWS)
    resources = judge.write_text(os.path.join(root, "resources.json"), json.dumps({"hosts": [{"number": 1, "read": True, "cpu": {"available_threads": threads},
                                                                                            "memory": {"available": int(room_gib * 2 * GIB), "slice_room": int(room_gib * GIB)}}]}))
    config = {"dry_run": dry_run, "dry_run_note": "自证" if dry_run else None, "tree": "scripts/experiment-apparatus/target/fake-tree", "H3": {"snapshot_digest": "f" * 64},
              "H8": {"mutations_table_sha256": "e" * 64}, "H14": {"reverse_states": 72, "main_states": 1207959552}, "H7": {},
              "H16": {"run_date": "2026-10-09", "run_suffix": "-r8", "dumps_root": "scripts/experiment-apparatus/target/e171-r1-dumps/fake", "logs_directory": os.path.join(root, "logs")}}
    config_path = judge.write_text(os.path.join(root, "config.json"), json.dumps(config))
    log = os.path.join(root, "fake.log")
    environment = dict(os.environ, UNITS_R8_SELFTEST="1", UNITS_R8_UNIT_SCRIPT=unit_script, UNITS_R8_ROWS_ORCHESTRATOR=rows,
                       UNITS_R8_RESOURCES_COMMAND=f"cat {shlex.quote(resources)}", UNITS_R8_PREREQUISITES_COMMAND="true",
                       UNITS_R8_POLL_SECONDS="0.02", UNITS_R8_REQUERY_SECONDS="0", FAKE_LOG=log, PYTHONDONTWRITEBYTECODE="1")
    environment.update(extra or {})
    return FakeWorld(root, repo, config_path, log, environment)


def world_run(world: FakeWorld, mode: str = "run", timeout: float = 240) -> tuple[int, str]:
    completed = subprocess.run(world.command(mode), cwd=world.repo, env=world.environment, capture_output=True, text=True, timeout=timeout)
    return completed.returncode, completed.stdout + completed.stderr


def fake_events(world: FakeWorld) -> list[tuple[str, str, str, float, str]]:
    """假件的日志：(start|end, 单元或 rows-run, TAG, 时刻, 备注)。"""
    events = []
    for line in open(world.log, encoding="utf-8") if os.path.exists(world.log) else []:
        words = line.split()
        if words[0].startswith("rows-"):
            events.append((words[1], words[0], "-", float(words[2]), ""))
        else:
            events.append((words[0], words[1], words[2], float(words[3]), " ".join(words[4:])))
    return events


def times_of(events, kind: str) -> dict[str, list[float]]:
    found: dict[str, list[float]] = {}
    for event, name, _, moment, _ in events:
        if event == kind:
            found.setdefault(name, []).append(moment)
    return found


def cell_allocation_memory(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, threads=30, room_gib=14)
    os.environ.update({key: world.environment[key] for key in ("UNITS_R8_RESOURCES_COMMAND",)})
    settings = settings_from_config(world.repo, world.config)
    budget = local_budget(settings)
    units = unit_table()
    placed = simulate(units, ["step-7", "step-9", "step-3-o1-crash-judge"], budget)
    at_zero = sorted(name for name, start, _ in placed if start == 0)
    return at_zero == ["step-7", "step-9"], f"额度 {budget.threads} 线程、{budget.memory_bytes / GIB:.0f} GiB：同时起的 {at_zero}（三个 6G 的只放得下两个）"


def cell_queue_order(scratch: str) -> tuple[bool, str]:
    units = unit_table()
    order = [unit.name for unit in queue_order([units[name] for name in ("step-5", "step-9", "step-3-o1-crash-judge", "step-7", "step-10-z7")])]
    return order == ["step-7", "step-9", "step-3-o1-crash-judge", "step-5", "step-10-z7"], f"排出来 {order}"


def cell_downstream(scratch: str) -> tuple[bool, str]:
    units = unit_table()
    below_two = downstream(units, {"step-2"})
    below_seven = downstream(units, {"step-7"})
    good = ("step-0" not in below_two and "step-1" not in below_two and "step-2b" in below_two and "step-8" in below_two
            and below_seven == {"step-7", "step-8", "step-9-z1"} and len(units) == 34)
    return good, f"step-2 下游 {len(below_two)} 个（不含 step-0、1）；step-7 下游 {sorted(below_seven)}；单元共 {len(units)} 个"


def cell_e2e_two_rounds(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, extra={"FAKE_RESEARCH_DIFFERS": "true", "FAKE_SECONDS_step_7": "0.4"})
    code, output = world_run(world)
    events = fake_events(world)
    started = times_of(events, "start")
    timing = os.path.join(world.repo, placeholders.product_prefix("2026-10-09") + "-baseline-run1-r8-timing.out")
    concurrency = [line for line in open(timing, encoding="utf-8") if line.startswith(STEP_7_CONCURRENCY_PREFIX)] if os.path.exists(timing) else []
    units_started = {name: moments for name, moments in started.items() if name != "rows-run"}
    good = (code == 0 and len(units_started) == 34 and all(len(moments) == 1 for moments in started.values()) and "research-mutations" in started
            and len(concurrency) == 1 and "step-9" in concurrency[0] and "rows-run" in started)
    return good, f"退 {code}；起过的单元 {len(units_started)}（要 34，各一次；逐行编排 {len(started.get('rows-run', []))} 次）；第 7 步计时末行 {concurrency[0].strip()[:90] if concurrency else '没有'}；末行 {output.strip().splitlines()[-1:]}"


def cell_e2e_order(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, extra={"FAKE_SECONDS_step_7": "0.3", "FAKE_SECONDS_step_1": "0.2"})
    code, _ = world_run(world)
    events = fake_events(world)
    started, ended = times_of(events, "start"), times_of(events, "end")
    units = unit_table()
    early = [name for name in started if name in units and any(dep in ended and started[name][0] < ended[dep][0] for dep in units[name].deps)]
    rows_early = "rows-run" in started and "step-7" in ended and started["rows-run"][0] < ended["step-7"][0]
    return code == 0 and not early and not rows_early and "research-mutations" not in started, \
        f"退 {code}；依赖没绿就起的 {early or '无'}；逐行编排早于第 7 步绿起 {rows_early}；研究一侧变异表没排（第 0 步报没有不同）{'research-mutations' not in started}"


def cell_e2e_unit_red(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, threads=20, extra={"FAKE_RED": "step-3-o2", "FAKE_SECONDS_step_7": "0.5", "FAKE_SECONDS_step_9": "0.5"})
    code, output = world_run(world)
    events = fake_events(world)
    red_end = [moment for event, name, _, moment, note in events if event == "end" and name == "step-3-o2"]
    later = [name for event, name, _, moment, _ in events if event == "start" and red_end and moment > red_end[0]]
    stop_file = os.path.exists(os.path.join(world.root, "logs", STOP_FILE))
    return code == 1 and bool(red_end) and not later and stop_file, \
        f"退 {code}；红了之后又起的 {later or '无'}；停派文件 {stop_file}；末行 {output.strip().splitlines()[-1:]}"


def cell_e2e_resume(scratch: str) -> tuple[bool, str]:
    """跑到一半收 TERM（退 143），同一条命令再起（退 0）：TERM 之前绿了的单元不重跑；被打断的那几个重跑、带 -attempt2。"""
    world = fake_world(scratch, threads=20, extra={"FAKE_SECONDS": "0.15"})
    process = subprocess.Popen(world.command(), cwd=world.repo, env=world.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    deadline = time.monotonic() + 120
    while time.monotonic() < deadline and len(times_of(fake_events(world), "end")) < 8:
        time.sleep(0.02)
    process.send_signal(signal.SIGTERM)
    process.communicate(timeout=120)
    first = process.returncode
    green_before = set(green_records(settings_from_config(world.repo, world.config)))
    second, output = world_run(world)
    started = times_of(fake_events(world), "start")
    rerun_green = sorted(name for name in green_before if len(started.get(name, [])) > 1)
    tags = {tag for event, name, tag, _, _ in fake_events(world) if event == "start" and tag != "-"}
    good = first == EXIT_STOPPED_BY_SIGNAL and second == 0 and len(green_before) >= 6 and not rerun_green and "-attempt2" in tags
    return good, f"第一次退 {first}（停前绿了 {len(green_before)} 个），续跑退 {second}；停前绿了又重跑的 {rerun_green or '无'}；重跑带的 TAG {sorted(tags)}"


def cell_e2e_resume_manifest_changed(scratch: str) -> tuple[bool, str]:
    """跑完之后改第 1 步导出里一个文件，再起：第 1 步与它下游全部重跑（-attempt2），第 0 步不重跑。"""
    world = fake_world(scratch)
    first, _ = world_run(world)
    changed = os.path.join(world.repo, "scripts/experiment-apparatus/target/e171-r1-dumps/fake/step-1/data")
    with open(changed, "a", encoding="utf-8") as handle:
        handle.write("改过\n")
    second, output = world_run(world)
    started = times_of(fake_events(world), "start")
    twice = sorted(name for name, moments in started.items() if len(moments) == 2 and name != "rows-run")
    units = unit_table()
    expected = sorted(name for name in downstream(units, {"step-1"}) if name != "research-mutations")
    return first == 0 and second == 0 and twice == expected and len(started.get("step-0", [])) == 1, \
        f"两次都退 0（{first}、{second}）；第二次重跑的 {len(twice)} 个（要第 1 步与它下游 {len(expected)} 个）；第 0 步跑了 {len(started.get('step-0', []))} 次"


def cell_e2e_dry_run_refused(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, dry_run=True)
    code, output = world_run(world)
    return code == EXIT_RUN_CONDITION and not os.path.exists(world.log), f"干跑配置真起跑：退 {code}，假件起过 {os.path.exists(world.log)}"


def cell_e2e_no_room(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch, threads=5, room_gib=3)
    code, output = world_run(world)
    return (code == EXIT_RUN_CONDITION and "放不下" in output,
            f"本机内存额度只有 3 GiB（此刻空闲 5 线程不看，线程额度是登记值）：退 {code}；{output.strip().splitlines()[-1:]}")


def cell_plan_prints_commands(scratch: str) -> tuple[bool, str]:
    world = fake_world(scratch)
    code, output = world_run(world, "plan")
    listed = sum(1 for line in output.splitlines() if line.startswith("单元 ") and "（第 1 次）：" in line)
    edges = sum(1 for line in output.splitlines() if " → " in line and not line.strip()[0].isdigit())
    good = code == 0 and listed == 34 and "fake rows plan" in output and not os.path.exists(os.path.join(world.root, "logs")) \
        and "S2_BASELINE=s2-device-baseline-streams-1-2" in output and edges >= 34
    return good, f"退 {code}；打出命令的单元 {listed}（要 34：第 0 步还没跑，研究一侧变异表按会排列出）；依赖边 {edges}；日志目录没建 {not os.path.exists(os.path.join(world.root, 'logs'))}"


def cell_unit_script_commands(scratch: str) -> tuple[bool, str]:
    """真的单元脚本 PRINT_ONLY=1：每个单元都打得出命令、退 0；几处修订 8 改了的地方打出来的就是登记写的那一样（已知答案）。"""
    world = fake_world(scratch)
    settings = settings_from_config(world.repo, world.config)
    units = unit_table()
    names = {variable: stem + "-attempt2" for unit in units.values() for variable, stem in unit.exports}
    printed, failed = {}, []
    for name in units:
        environment = unit_environment(settings, units[name], 2, os.path.join(scratch, name), names, True)
        completed = subprocess.run(["bash", UNIT_SCRIPT, name], cwd=world.repo, env=environment, capture_output=True, text=True)
        printed[name] = completed.stdout
        if completed.returncode != 0 or "  $ " not in completed.stdout:
            failed.append(name)
    expectations = {
        "step-0": "frozen_tree_lines_revision_8.py step-0", "step-2b": "bad-states-second-run --dump-directory scripts/experiment-apparatus/target/e171-r1-dumps/fake/e170-bad-states-second-run-attempt2 --seed-hex 85441ae491419411",
        "step-3-o3-p1": "--e170-dumps scripts/experiment-apparatus/target/e171-r1-dumps/fake/e170-bad-states-second-run-attempt2", "step-4-o3-p1": "e170-bad-states-second-run-attempt2",
        "step-6": "states=72", "step-10-run-1": "states=1207959552", "step-7": "baseline-run1-attempt2", "step-9-z1": "z1 scripts/experiment-apparatus/target/e171-r1-dumps/fake/baseline-run1-attempt2",
        "step-2": "s2-e170-export-attempt2 scripts/experiment-apparatus/target/e171-r1-dumps/fake/s2-device-baseline-streams-1-2-attempt2",
    }
    missing = [name for name, text in expectations.items() if text not in printed.get(name, "")]
    s15 = [name for name, text in printed.items() if "s15_compare" in text]
    good = not failed and not missing and not s15
    return good, f"打不出命令或退非 0 的单元 {failed or '无'}；修订 8 改的那几处没打出预期的 {missing or '无'}；还接 S15 比对的 {s15 or '无'}"


SELFTEST_CELLS = {
    "unit-script-commands": cell_unit_script_commands,
    "allocation-memory": cell_allocation_memory, "queue-order": cell_queue_order, "downstream": cell_downstream,
    "e2e-two-rounds": cell_e2e_two_rounds, "e2e-order": cell_e2e_order, "e2e-unit-red": cell_e2e_unit_red, "e2e-resume": cell_e2e_resume,
    "e2e-resume-manifest-changed": cell_e2e_resume_manifest_changed, "e2e-dry-run-refused": cell_e2e_dry_run_refused, "e2e-no-room": cell_e2e_no_room,
    "plan-prints-commands": cell_plan_prints_commands,
}
SELFTEST_SWITCHES = {
    "resume-reruns-green": "e2e-resume", "resume-ignores-manifest": "e2e-resume-manifest-changed", "downstream-not-rerun": "e2e-resume-manifest-changed",
    "dependency-ignored": "e2e-order", "stop-ignored": "e2e-unit-red", "rows-before-step-7": "e2e-order", "budget-ignored": "allocation-memory",
    "concurrency-line-missing": "e2e-two-rounds", "dry-run-runs": "e2e-dry-run-refused",
}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
