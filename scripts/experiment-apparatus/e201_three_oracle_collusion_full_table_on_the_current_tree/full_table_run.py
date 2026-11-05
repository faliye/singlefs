#!/usr/bin/env python3
# task-server-shards: run
# task-server-gpu-arguments: run
# task-server-resumable: run
"""E201（三oracle合谋今树整表） 的入口（跑前登记 research/prompts/e201-preregistration.md 第五节 5.5「full_table_run.py」、第十四节；修订 1、2）。

只经统一任务服务的分片三步起（服务认文件头 `# task-server-shards: run`；`# task-server-gpu-arguments: run` 要卡；同一条命令重交就是续跑）：
  task-server/task submit -- python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run [--stream-three]
  task-server/task submit -- env PLAN_ONLY=1 python3 …/full_table_run.py run [--stream-three]   干跑：只做计划（拷树、记、核名单、算键、查库、抽哨兵、分片），
      计划行写进计划目录、合并那一步原样打进任务日志，0 片、不编不跑、不进 research/results
  python3 …/full_table_run.py --selftest        自证四格（复用三问、哨兵、复用账、A7；假驱动、假比对器、内存里的假判定库，不起 cargo、编译、装置、服务；
      末行 REUSE_SELFTEST 与 FULL_TABLE_SELFTEST cells=4 …）
三步（环境 TASK_SERVER_SHARD_PHASE 定是哪一步；没有这个变量退 2：不在服务外面起）：
  计划 plan：把此刻主工作区的 SYNCED_ENTRIES 那几样（scripts/hosts/lib_repository_copy.py 现算的拷贝范围：Cargo.toml、Cargo.lock、.cargo、crates 加闭包，
      prune-server 与 crates 编、跑时读的仓根文件；不带 target；各步起的子进程照那份库的 environment_for_commands_in_the_copy 带多机配置的绝对路径）
      拷进计划目录的 tree/（读法 1，登记修订 5、6），记快照摘要与份数、HEAD、
      `git status --porcelain -- <SYNCED_ENTRIES>` 全文、变异表与两张名单与装置与研究一侧各文件的 sha256；核名单（按名字取几次、锚点命中几次）；
      按读法 10 算每行的键、查 prune-server（族 apparatus-items、种类 three_oracle_collusion_full_table_on_the_current_tree）；库里有同键的不派，按 5.6 抽哨兵
      （k = max(2, ⌈0.02 × M⌉)，按 sha256(快照摘要 ‖ 键) 由小到大取，哨兵那几条的库值先抄进计划目录）；要现跑的行按行号交替分给两片；写 plan.json 与 plan-record.json。
  各片 shard：开头与收尾核计划记的树与各文件 sha256（Z3）、冻结的 O3（Z2）；每片先编不改坏的装置、跑基线（`baseline --streams …`，T 取它的进程挂钟），
      片 0 另跑第二次基线（Z1）、停机 S2 那一比（E170 导出装置 `dump` 与基线导出逐文件比）、O3 基线与九样阳性对照两遍（基线一遍、空变异一遍）；
      两片都跑空变异（两片互比是跨机对照，S7）；再起逐行驱动跑分到的行（⌊SINGLEFS_THREAD_CAP ÷ 6⌋ 条线道、每线道 6 线程），每行判完把它的
      row-result.tsv、o3.out、device.out 与导出目录里除三个大文件之外的文件打成一包写进库（键照读法 10）。空变异与各行都带 --keep-dumps 跑，
      入库之后哨兵以外的行删掉三个大文件；收尾对空变异与这一片的每一行哨兵核 A7（存进库那一份与完整导出在比对器上逐字相同，name=a7）。
      编译目录在服务给的 CARGO_TARGET_DIR（没给时在这一片的产物目录）下各线道一份，这一片收尾删掉。
  合并 merge：从库里取回全部行摆成行输出根，起 E171 的第二段比对器出 T1、合成 T2、调第一段比对器那几个函数出 T3，再出逐格比（contrast_against_r1.py）、
      归因（attribution.py）、Q2（q2_misses.py）、整表矩阵（whole_table_matrix.py）、阳性阴性对照、跨机对照、哨兵与复用账、NOT_RUN 合计（Z6）、
      编前编后核对（Z4）、A7；作废、停机、失败条款逐条由纯函数 clause_checks 判（单测手造读数）。
  作废 Z8（统一任务服务的结局不是跑成）不在这里判：由交任务的主 agent 读 `task-server/task wait` 末行 TASK_SERVER_RESULT 的 exit= 与 shard= 判（登记修订 3）。
产物（都是新文件，已存在就加 -rN）：research/results/e201-three-oracle-collusion-full-table-on-the-current-tree-<计划那一天>-<段>-<件>.out，
  段是 s1（流一、流二整表）或 s2（--stream-three：空变异加 r1 第一段十五行、三条流），件是 plan、t1、t2、t3、contrast、attribution、q2、matrix、controls、verdict。
退出码：0 跑完（判定在产物里）；1 有作废或停机条款触发、复用账对不上、有一片的 NOT_RUN 合计不为 0；2 用法错、不在服务的分片三步里起；
  75 prune-server 连不上（判定库只有那一份，不退回本地文件）。
弄坏开关只在自证里认（E201_FULL_TABLE_BREAK；平常跑设了退 2）：
  E201_FULL_TABLE_BREAK=sentinel-comparison-skipped  哨兵现跑之后不与库里那一条比
      自证项 「✗ sentinel-detects-a-corrupted-library-entry」
  E201_FULL_TABLE_BREAK=account-not-checked  复用账不核现判数加复用数等于计划登记的行数
      自证项 「✗ reuse-account」
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import io
import json
import math
import os
import shutil
import subprocess
import sys
import tarfile
from collections.abc import Callable

import e171_research_side
from result_lines import ResultLines, field

HERE = e171_research_side.HERE
REPOSITORY_ROOT = e171_research_side.REPOSITORY_ROOT
LIBRARY_FAMILY = "apparatus-items"
LIBRARY_KIND = "three_oracle_collusion_full_table_on_the_current_tree"
LIBRARY_DIRECTORY_NAME = "e201-full-table-rows"
PRODUCT_STEM = "e201-three-oracle-collusion-full-table-on-the-current-tree"
DEVICE_BINARY_NAME = "e171_three_oracle_collusion_after_the_oracle_fixes"
EXPORT_BINARY_NAME = "e170_o3_spec_executor_export"
DEVICE_ARGUMENTS = {"s1": ("row", "--streams", "1,2"), "s2": ("row", "--streams", "1,2,3")}
DEVICE_FEATURES = "gpu"
LANE_THREADS = 6
SHARD_COUNT = 2
# 拷进计划目录的仓根条目（目录或单个文件；与 current_tree_row_driver.SYNCED_ENTRIES 是同一组，单测核两份相同）：不在这里写，取 scripts/hosts/
# lib_repository_copy.py 的 copy_closure（共用清单加闭包：prune-server 是 crates 经 path 依赖到的目录（登记修订 5）；.claude/heavy-thread-default 是
# singlefs-unit-graph 与 singlefs-verification 的 heavy_threads.rs 编译期 include_str! 读的（登记修订 6，任务 5606 停在 S6）；
# scripts/crash/crash-point-replay-shard-configuration-check.sh 是 gpu_card_configuration.rs 运行期从编这份二进制的那棵树起的多机配置判法，
# 它 source、起的与装置 preflight 起的规范副本准入脚本由闭包传递地收（登记修订 6））。共用库按仓根找（变异驱动把本目录拷到仓外跑单测时仓根经环境交过来）
sys.path.append(os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import lib_repository_copy  # noqa: E402
SYNCED_ENTRIES = lib_repository_copy.copy_closure(REPOSITORY_ROOT).entries
WITHDRAWN_ROWS = frozenset({33, 107, 236})
R1_SEGMENT_ONE_ROWS = (15, 23, 42, 49, 110, 125, 155, 161, 162, 174, 242, 252, 269, 270, 271)
BULKY_DUMP_FILES = ("base-sectors.bin", "ops-payload.bin", "torn-payload.bin")
STORED_ROW_FILES = ("row-result.tsv", "o3.out", "device.out")
BREAK_VARIABLE = "E201_FULL_TABLE_BREAK"
JUDGED, REUSED, FORCED = "现判", "复用", "强制现判"
EXIT_INVALID, EXIT_USAGE, EXIT_UNAVAILABLE = 1, 2, 75
SENTINEL_FRACTION = 0.02
MINIMUM_SENTINELS = 2
# 干跑（经 `-- env PLAN_ONLY=1` 交）：计划照做（拷树、记、核名单、算键、查库、抽哨兵、分片），计划行写进计划目录里的这一份而不进 research/results，
# plan.json 写 0 片；合并那一步把这一份原样打进任务日志，不起任何一片
PLAN_ONLY_VARIABLE = "PLAN_ONLY"
PLAN_ONLY_PRODUCT = "plan-only.out"


def sha256_bytes(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def sha256_file(path: str) -> str:
    with open(path, "rb") as handle:
        return sha256_bytes(handle.read())


def broken(item: str) -> bool:
    return os.environ.get(BREAK_VARIABLE, "") == item


def git_text(*arguments: str) -> str:
    return subprocess.run(["git", "-C", REPOSITORY_ROOT, *arguments], check=True, capture_output=True, text=True).stdout


def product_path(segment: str, part: str, date: str, run_suffix: str) -> str:
    """产物路径：research/results/<名>-<日期><-rN>-<段>-<件>.out。"""
    return os.path.join(REPOSITORY_ROOT, "research", "results", f"{PRODUCT_STEM}-{date}{run_suffix}-{segment}-{part}.out")


def free_run_suffix(segment: str, date: str) -> str:
    """同一天同一段已有产物就加 -r2、-r3……（不覆盖 research/results 下已有的任何一份）。"""
    suffix, number = "", 1
    while os.path.exists(product_path(segment, "plan", date, suffix)):
        number += 1
        suffix = f"-r{number}"
    return suffix


def own_source_files() -> tuple[str, ...]:
    """本目录的源文件（相对仓根），进计划记的 sha256 与读法 10 的键（逐行驱动与 O3 一步的研究一侧文件）。"""
    names = sorted(name for name in os.listdir(HERE) if name.endswith((".py", ".tsv")) and not name.startswith("test_"))
    return tuple(os.path.relpath(os.path.join(HERE, name), REPOSITORY_ROOT) for name in names)


# ── 读法 10：键、判定库、哨兵、复用账（纯函数与一层薄的库，自证与单测都只碰这一层） ──

KEY_RESEARCH_FILES = (
    os.path.relpath(os.path.join(HERE, "current_tree_row_driver.py"), REPOSITORY_ROOT),
    os.path.relpath(os.path.join(HERE, "e171_research_side.py"), REPOSITORY_ROOT),
    *(os.path.join(e171_research_side.E171_RELATIVE, name)
      for name in ("frozen_o3.py", "device_timing.py", "bad_implementation_table.py", "run_directory.py", "o3_bad_image_judge.py")))


@dataclasses.dataclass(frozen=True)
class TreeInputs:
    """一趟里各行共用的那几样键的组成（读法 10）。"""

    snapshot_digest: str
    mutations_table_sha256: str
    device_arguments: tuple[str, ...]
    features: str
    toolchain: str
    judging_route: str
    research_files_sha256: tuple[tuple[str, str], ...]
    o3_freeze_digest: str


@dataclasses.dataclass(frozen=True)
class RowInput:
    """一行自己的那几样键的组成：整表编号、名单那一行全文、改坏用的变异行的 sha256（计划那一刻照名字取到、锚点恰好一次的那一行；
    取不到或锚点不唯一的行要补位，补位挑行跨行有关，这种行不复用，键里写 needs_substitution）。"""

    number: int
    list_line: str
    mutation_line_sha256: str


def row_key(tree: TreeInputs, row: RowInput) -> str:
    """读法 10 的键：各组成按固定次序拼成一份文字再取 sha256。"""
    parts = [f"snapshot={tree.snapshot_digest}", f"mutations_table={tree.mutations_table_sha256}", f"row={row.number}",
             f"list_line={row.list_line}", f"mutation_line={row.mutation_line_sha256}",
             f"device_arguments={' '.join(tree.device_arguments)}", f"features={tree.features}", f"toolchain={tree.toolchain}",
             f"judging_route={tree.judging_route}", *(f"research={path}:{digest}" for path, digest in tree.research_files_sha256),
             f"o3_freeze={tree.o3_freeze_digest}"]
    return sha256_bytes("\n".join(parts).encode("utf-8"))


def row_is_reusable(row: RowInput) -> bool:
    return row.mutation_line_sha256 != "needs_substitution"


class MemoryStore:
    """自证与单测用的假判定库：只有 read / write 两样，与 prune_server_client.LibraryStore 同形。"""

    def __init__(self):
        self.entries: dict[str, bytes] = {}

    def read(self, relative: str) -> bytes | None:
        return self.entries.get(relative)

    def write(self, relative: str, data: bytes) -> None:
        self.entries[relative] = data


def library_relative(key: str) -> str:
    return f"rows/{key}"


def open_library():
    """真跑用的那一份：prune-server 的 apparatus-items 族（连不上由客户端退 75）。"""
    sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "prune-server"))
    import prune_server_client
    return prune_server_client.library_store(LIBRARY_FAMILY, LIBRARY_KIND, LIBRARY_DIRECTORY_NAME)


@dataclasses.dataclass(frozen=True)
class ReusePlan:
    """计划那一步的复用决定：现跑的行（键对不上或不复用）、复用的行、哨兵（复用的行里抽出现跑、与库里那一条比的）。"""

    keys: dict[int, str]
    fresh: tuple[int, ...]
    reused: tuple[int, ...]
    sentinels: tuple[int, ...]


def sentinel_count(reused: int) -> int:
    """5.6：库里同键的 M ≥ 1 行时抽 k = max(2, ⌈0.02 × M⌉) 行（不超过 M）；M = 0 时 0。"""
    return 0 if reused == 0 else min(reused, max(MINIMUM_SENTINELS, math.ceil(SENTINEL_FRACTION * reused)))


def plan_reuse(tree: TreeInputs, rows: list[RowInput], store) -> ReusePlan:
    keys = {row.number: row_key(tree, row) for row in rows}
    reusable = [row.number for row in rows if row_is_reusable(row) and store.read(library_relative(keys[row.number])) is not None]
    order = sorted(reusable, key=lambda number: sha256_bytes(f"{tree.snapshot_digest}{keys[number]}".encode("utf-8")))
    sentinels = tuple(sorted(order[:sentinel_count(len(reusable))]))
    reused = tuple(sorted(set(reusable) - set(sentinels)))
    fresh = tuple(sorted(row.number for row in rows if row.number not in reused))
    return ReusePlan(keys, fresh, reused, sentinels)


def split_into_shards(numbers: tuple[int, ...], shard_count: int = SHARD_COUNT) -> list[list[int]]:
    """要现跑的行交替分给各片（没有逐行的用时可推，按行号由大到小交替；空的片也照分）。"""
    shards: list[list[int]] = [[] for _ in range(shard_count)]
    for position, number in enumerate(sorted(numbers, reverse=True)):
        shards[position % shard_count].append(number)
    return [sorted(shard) for shard in shards]


def pack_row_directory(row_directory: str) -> bytes:
    """一行存进库的那一份（读法 10）：row-result.tsv、o3.out、device.out 与 dump/ 下除三个大文件之外的全部文件，按相对路径排好序打成 tar（修改时刻置 0，同一份内容同一串字节）。"""
    import gzip
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w") as archive:
        members = [name for name in STORED_ROW_FILES if os.path.isfile(os.path.join(row_directory, name))]
        for base, _, names in os.walk(os.path.join(row_directory, "dump")):
            members.extend(os.path.relpath(os.path.join(base, name), row_directory) for name in names if name not in BULKY_DUMP_FILES)
        for relative in sorted(members):
            with open(os.path.join(row_directory, relative), "rb") as handle:
                content = handle.read()
            information = tarfile.TarInfo(relative)
            information.size, information.mtime, information.mode = len(content), 0, 0o644
            archive.addfile(information, io.BytesIO(content))
    return gzip.compress(buffer.getvalue(), compresslevel=6, mtime=0)


def unpack_row_directory(data: bytes, row_directory: str) -> None:
    os.makedirs(row_directory, exist_ok=False)
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        archive.extractall(row_directory, filter="data")


def account_problems(plan: ReusePlan, states: dict[int, str], registered_rows: int) -> list[str]:
    """复用账（第六节「复用账」）：现判数 N 加复用数 M 等于计划登记的行数；每一行都有结局。"""
    if broken("account-not-checked"):
        return []
    problems = []
    judged = sum(state in (JUDGED, FORCED) for state in states.values())
    reused = sum(state == REUSED for state in states.values())
    if judged + reused != registered_rows:
        problems.append(f"现判 {judged} + 复用 {reused} ≠ 登记 {registered_rows}")
    missing = sorted(set(plan.keys) - set(states))
    if missing:
        problems.append("没有结局的行：" + ",".join(map(str, missing)))
    return problems


def read_sentinel_entries(plan: ReusePlan, store) -> dict[int, bytes]:
    """计划那一步：哨兵那几行库里原有的那一份先取出来（各片现跑之后同一把键会被新写的盖掉，合并拿这一份比）。"""
    return {number: store.read(library_relative(plan.keys[number])) for number in plan.sentinels}


def judge_and_store(plan: ReusePlan, numbers: list[int], store, judge: Callable[[int], bytes | None]) -> dict[int, bytes]:
    """各片：这一片分到的行逐行现判，判完一行就把它那一份写进库（一块判完才写，写是原子的；被打断重交就是续跑）。judge 交 None 是这一行没跑成，不写。"""
    results = {}
    for number in sorted(numbers):
        data = judge(number)
        if data is None:
            continue
        store.write(library_relative(plan.keys[number]), data)
        results[number] = data
    return results


def sentinel_problems(plan: ReusePlan, before: dict[int, bytes], after: dict[int, bytes], decision_of: Callable[[bytes], object]) -> list[str]:
    """作废 Z7：哨兵现跑出的判定格与库里原有那一条逐格相同才算复用有效；不同的逐行列出。"""
    if broken("sentinel-comparison-skipped"):
        return []
    problems = []
    for number in plan.sentinels:
        if number not in after:
            problems.append(f"{number}:sentinel_not_rerun")
        elif decision_of(before[number]) != decision_of(after[number]):
            problems.append(f"{number}:decision_cells_differ")
    return problems


def outcome_states(plan: ReusePlan, judged: dict[int, bytes]) -> dict[int, str]:
    """每行这一趟的结局：现判（键对不上或不复用）、复用、强制现判（哨兵）；现判那一类没跑成的行不记结局（复用账会点名）。"""
    states = {number: REUSED for number in plan.reused}
    states.update({number: FORCED for number in plan.sentinels if number in judged})
    states.update({number: JUDGED for number in plan.fresh if number in judged and number not in plan.sentinels})
    return states


# ── 自证 ──

def load_reuse_checks():
    import importlib.util
    spec = importlib.util.spec_from_file_location("lib_reuse_checks_for_e201", os.path.join(REPOSITORY_ROOT, "scripts", "gate", "lib_reuse_checks.py"))
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def selftest_tree(root: str) -> TreeInputs:
    """自证的假树：快照摘要照 E171 逐行驱动的 snapshot_digest 在 root 上现算（只收 .rs 与 Cargo.*）。"""
    import row_driver
    digest, _ = row_driver.snapshot_digest(root)
    return TreeInputs(digest, "f" * 64, DEVICE_ARGUMENTS["s1"], DEVICE_FEATURES, "rustc fake", "cpu", (("fake.py", "e" * 64),), "d" * 64)


def selftest_rows(changed_row: int | None = None) -> list[RowInput]:
    return [RowInput(number, f"{number}\t1\tcrates/a/src/lib.rs\t0\tm{number}\tabc" + ("-changed" if number == changed_row else ""), f"{number:064x}")
            for number in (11, 12, 13, 14, 15, 16)]


def selftest_round(tree: TreeInputs, rows: list[RowInput], store) -> tuple[dict[int, str], list[str]]:
    """一趟（计划、各片、合并三步在一个进程里依次做）：交回 (每行的结局, 哨兵与复用账的问题)。"""
    plan = plan_reuse(tree, rows, store)
    before = read_sentinel_entries(plan, store)
    by_number = {row.number: row for row in rows}
    judged = judge_and_store(plan, list(plan.fresh), store,
                             lambda number: f"verdict:{number}:{by_number[number].mutation_line_sha256}".encode("utf-8"))
    states = outcome_states(plan, judged)
    return states, sentinel_problems(plan, before, judged, lambda data: data) + account_problems(plan, states, len(rows))


def cell_reuse_three_questions(printer=print) -> bool:
    """复用三问（.claude/rules/test-script-discipline.md「复用也要自证」；判法 scripts/gate/lib_reuse_checks.py）。"""
    import tempfile
    checks = load_reuse_checks()
    store = MemoryStore()
    with tempfile.TemporaryDirectory() as root:
        os.makedirs(os.path.join(root, "crates", "a", "src"))
        with open(os.path.join(root, "crates", "a", "src", "lib.rs"), "w", encoding="utf-8") as handle:
            handle.write("pub fn alpha() {}\n")
        with open(os.path.join(root, "crates", "a", "README.md"), "w", encoding="utf-8") as handle:
            handle.write("说明\n")
        first, first_problems = selftest_round(selftest_tree(root), selftest_rows(), store)
        second, second_problems = selftest_round(selftest_tree(root), selftest_rows(), store)
        with open(os.path.join(root, "crates", "a", "README.md"), "a", encoding="utf-8") as handle:
            handle.write("闭包外的改动：快照摘要只收 .rs 与 Cargo.*\n")
        unrelated, unrelated_problems = selftest_round(selftest_tree(root), selftest_rows(), store)
        unit, unit_problems = selftest_round(selftest_tree(root), selftest_rows(changed_row=13), store)
    for label, states in (("头一趟", first), ("第二趟", second), ("闭包外改动", unrelated), ("改一行的名单行文", unit)):
        for number, state in sorted(states.items()):
            printer(f"  · 第 {number} 行：绿，{state}（{label}）")
    passed, _ = checks.report_three_questions(os.path.relpath(__file__, REPOSITORY_ROOT), {str(key): value for key, value in first.items()},
                                              {str(key): value for key, value in second.items()}, {str(key): value for key, value in unrelated.items()},
                                              {str(key): value for key, value in unit.items()}, {"13"}, printer=printer,
                                              sentinels=sentinel_count(len(first)))
    problems = first_problems + second_problems + unrelated_problems + unit_problems
    for problem in problems:
        printer(f"    reuse-three-questions：{problem}")
    return passed and not problems


def cell_sentinel_detects_a_corrupted_library_entry(printer=print) -> bool:
    """哨兵（V14）：库里同键的结论被改坏一处，这一趟抽中的哨兵现跑之后判出不同（作废 Z7）。"""
    store = MemoryStore()
    tree = TreeInputs("c" * 64, "f" * 64, DEVICE_ARGUMENTS["s1"], DEVICE_FEATURES, "rustc fake", "cpu", (), "d" * 64)
    selftest_round(tree, selftest_rows(), store)
    for relative in list(store.entries):
        store.entries[relative] += b":corrupted"
    _, problems = selftest_round(tree, selftest_rows(), store)
    found = any(problem.endswith(":decision_cells_differ") for problem in problems)
    printer(f"    sentinel-detects-a-corrupted-library-entry：problems={problems}")
    return found


def cell_reuse_account(printer=print) -> bool:
    """复用账：结局齐、N + M 等于登记数时没有问题；少一行结局时点名。"""
    plan = ReusePlan({11: "k11", 12: "k12"}, (11,), (12,), ())
    complete = account_problems(plan, {11: JUDGED, 12: REUSED}, 2)
    short = account_problems(plan, {12: REUSED}, 2)
    printer(f"    reuse-account：complete={complete} short={short}")
    return complete == [] and len(short) == 2


def files_of_a_row_directory(rows_root: str, number: int, skipped: tuple[str, ...]) -> tuple[str, ...]:
    """自证用的假比对器：行目录里每份文件的相对路径与内容（名字在 skipped 里的不读）。"""
    row_directory = os.path.join(rows_root, f"row-{number}")
    found = []
    for base, _, names in os.walk(row_directory):
        for name in sorted(names):
            if name in skipped:
                continue
            with open(os.path.join(base, name), encoding="utf-8") as handle:
                found.append(f"{os.path.relpath(os.path.join(base, name), row_directory)}={handle.read()}")
    return tuple(sorted(found))


def cell_stored_row_matches_the_full_export(printer=print) -> bool:
    """A7（第七节）：比对器不读三个大文件时，存进库那一份与完整导出判出的逐字相同（绿）；比对器读了大文件时判出不同（红）。
    比对器换成假的（读行目录里的文件）：真比对器在真跑时由各片对空变异与哨兵各核一次（name=a7）。"""
    import tempfile
    with tempfile.TemporaryDirectory() as root:
        row_directory = os.path.join(root, "full", "row-5")
        os.makedirs(os.path.join(row_directory, "dump", STREAM_DIRECTORIES[0]))
        for relative, content in (("row-result.tsv", "row_number\t5\nstatus\tok\n"), ("device.out", "E7RESULT name=verdict o2_red_states=0\n"),
                                  (f"dump/{STREAM_DIRECTORIES[0]}/states.tsv", "1\t2\n"), (f"dump/{STREAM_DIRECTORIES[0]}/base-sectors.bin", "sectors\n")):
            with open(os.path.join(row_directory, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
        scratch = os.path.join(root, "scratch")
        reading_only_stored = stored_row_differences("-", row_directory, 5, scratch,
                                                     lambda _baseline, rows_root, number: files_of_a_row_directory(rows_root, number, BULKY_DUMP_FILES))
        reading_the_bulky_files = stored_row_differences("-", row_directory, 5, scratch,
                                                         lambda _baseline, rows_root, number: files_of_a_row_directory(rows_root, number, ()))
    printer(f"    stored-row-matches-the-full-export：reading_only_stored={reading_only_stored} reading_the_bulky_files={reading_the_bulky_files}")
    return reading_only_stored == [] and reading_the_bulky_files == ["5:stored_row_differs"]


# ── 计划 ──

def copy_tree(destination: str, source_root: str = REPOSITORY_ROOT) -> None:
    """读法 1：此刻主工作区的 SYNCED_ENTRIES 那几样（不带 target，含已暂存与未暂存的改动），连同 crates/mutations.tsv。
    单个文件的条目先建好它在树里的上级目录（.claude/、scripts/crash/ 这类；rsync 不建目的路径缺的上级目录，缺了退 3）。"""
    os.makedirs(destination, exist_ok=False)
    for entry in SYNCED_ENTRIES:
        source = os.path.join(source_root, entry)
        if os.path.isdir(source):
            subprocess.run(["rsync", "-a", "--exclude", "target", source + "/", os.path.join(destination, entry) + "/"], check=True)
        elif os.path.exists(source):
            copied_file = os.path.join(destination, entry)
            os.makedirs(os.path.dirname(copied_file), exist_ok=True)
            subprocess.run(["rsync", "-a", source, copied_file], check=True)


@dataclasses.dataclass(frozen=True)
class TreeSnapshot:
    """读法 1 计划那一步记的：拷进计划目录那一份的快照摘要与份数（E171 逐行驱动 snapshot_digest 的算法）、HEAD、
    `git status --porcelain -- <SYNCED_ENTRIES>` 全文（一行一条，修订 1 第 1.3 条归因要它；git 忽略的 .claude/singlefs-ai-sop 下那几份不出行）。"""

    digest: str
    file_count: int
    head: str
    status_lines: tuple[str, ...]


def take_tree_snapshot(tree: str, source_root: str = REPOSITORY_ROOT, git: Callable[..., str] = git_text) -> TreeSnapshot:
    """拷树（copy_tree）之后在拷进来的那一份上算快照摘要，再记那一刻的 HEAD 与改动清单（git 换得掉：单测给假的）。"""
    import row_driver
    copy_tree(tree, source_root)
    digest, file_count = row_driver.snapshot_digest(tree)
    head = git("rev-parse", "HEAD").strip()
    status_lines = tuple(git("status", "--porcelain", "--", *SYNCED_ENTRIES).splitlines())
    return TreeSnapshot(digest, file_count, head, status_lines)


def write_snapshot_status_lines(lines, snapshot: TreeSnapshot) -> None:
    """改动清单逐行进计划产物（一行一条 name=snapshot_status，不截不并）。"""
    for line in snapshot.status_lines:
        lines.result(f"name=snapshot_status line={field(line)}")


def toolchain_text(tree: str) -> str:
    """`rustc -vV` 与 `cargo -V` 全文（在树里起，认树里的工具链文件）。"""
    rustc = subprocess.run(["rustc", "-vV"], cwd=tree, check=True, capture_output=True, text=True).stdout
    cargo = subprocess.run(["cargo", "-V"], cwd=tree, check=True, capture_output=True, text=True).stdout
    return rustc + cargo


def judging_route() -> str:
    """给定状态引擎这一趟的判器路线：多机配置开着 GPU 记 gpu，没开记 cpu（多机配置的取法同各入口：SINGLEFS_MULTI_HOST_CONFIG，没设读仓根的 multi-host.env）。"""
    path = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG") or os.path.join(REPOSITORY_ROOT, "multi-host.env")
    with open(path, encoding="utf-8") as handle:
        values = dict(line.strip().split("=", 1) for line in handle if "=" in line and not line.lstrip().startswith("#"))
    return "gpu" if values.get("ENABLE_GPU", "0").strip().strip('"') == "1" else "cpu"


def o3_freeze_digest() -> str:
    import frozen_o3
    lines = frozen_o3.frozen_lines_of(os.path.join(REPOSITORY_ROOT, frozen_o3.FREEZE_REFERENCE_RELATIVE))
    return sha256_bytes("".join(lines).encode("utf-8"))


def recorded_files() -> tuple[str, ...]:
    """计划记 sha256、各片开头收尾核的仓里文件（相对仓根）：两张名单、第三节研究一侧各文件、本目录的源文件。装置源码在计划拷进来的树里，
    随快照摘要核（主工作区那一份之后被别的会话改了不影响这一趟编的那一份）。"""
    return tuple(sorted({*e171_research_side.E171_RESEARCH_SIDE_FILES, *own_source_files()}))


def list_lines_by_number() -> dict[int, str]:
    lines = {}
    for path in e171_research_side.NAME_LISTS:
        with open(os.path.join(REPOSITORY_ROOT, path), encoding="utf-8") as handle:
            for raw in handle:
                fields = raw.rstrip("\n").split("\t")
                if len(fields) == 6 and fields[0].isdigit():
                    lines[int(fields[0])] = raw.rstrip("\n")
    return lines


def segment_rows(segment: str, numbers: list[int]) -> list[int]:
    """第一段整表（读法 6 的行集）；第二段只排 r1 第一段那十五行里今天还在名单上的（读法 11）。空变异另跑，不在这里。"""
    return sorted(numbers) if segment == "s1" else sorted(set(numbers) & set(R1_SEGMENT_ONE_ROWS))


def row_inputs_and_list_lines(lines: ResultLines | None, tree: str, numbers: list[int]) -> list[RowInput]:
    """按名字取行、核锚点（不编不跑，A2 与 F8）：每行一条 name=list_check；照名字取到且锚点恰好一次的行，键里是那一行变异行的 sha256。"""
    from bad_implementation_table import STATUS_LOCATED, SelectedRow, anchor_hit_count, locate_by_name, read_mutation_rows
    table = read_mutation_rows(os.path.join(tree, "crates", "mutations.tsv"))
    texts = list_lines_by_number()
    inputs, not_once = [], []
    for number in numbers:
        number_text, slice_text, file, group_text, name, digest_prefix = texts[number].split("\t")
        located = locate_by_name(SelectedRow(number, int(slice_text), file, int(group_text), name, digest_prefix), table)
        name_hits = sum(row.name == name for row in table)
        anchor_hits = 0
        if located.status == STATUS_LOCATED and located.row is not None:
            with open(os.path.join(tree, located.row.file), encoding="utf-8") as handle:
                anchor_hits = anchor_hit_count(handle.read(), located.row)
        reusable = located.row is not None and anchor_hits == 1
        line_sha = sha256_bytes(located.row.raw_line.encode("utf-8")) if reusable else "needs_substitution"
        inputs.append(RowInput(number, texts[number], line_sha))
        if name_hits != 1 or anchor_hits != 1:
            not_once.append(number)
        if lines is not None:
            lines.result(f"name=list_check row={number} name={field(name)} name_hits={name_hits} anchor_hits={anchor_hits} status={located.status} "
                         f"mutation_line_sha256={line_sha}")
    if lines is not None:
        lines.result(f"name=list_check_total rows={len(numbers)} rows_not_hit_once_by_name_or_anchor={len(not_once)} "
                     f"rows={field(','.join(map(str, not_once)) or '-')}")
    return inputs


def today_in_tokyo() -> str:
    import datetime
    import zoneinfo
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")


def latest_first_segment_t1() -> str | None:
    """第二段的 Q2-流三要第一段的 T1（修订 1 第 1.2 条）：research/results 下最新的那一份第一段 T1。"""
    directory = os.path.join(REPOSITORY_ROOT, "research", "results")
    found = [os.path.join(directory, name) for name in os.listdir(directory) if name.startswith(PRODUCT_STEM) and name.endswith("-s1-t1.out")]
    return max(found, key=os.path.getmtime) if found else None


def plan_step(arguments: argparse.Namespace, plan_directory: str, store) -> int:
    segment = "s2" if arguments.stream_three else "s1"
    date = today_in_tokyo()
    suffix = free_run_suffix(segment, date)
    tree = os.path.join(plan_directory, "tree")
    snapshot = take_tree_snapshot(tree)
    digest, file_count, head, status_lines = snapshot.digest, snapshot.file_count, snapshot.head, list(snapshot.status_lines)
    recorded = {path: sha256_file(os.path.join(REPOSITORY_ROOT, path)) for path in recorded_files()}
    mutations_sha = sha256_file(os.path.join(tree, "crates", "mutations.tsv"))
    toolchain = toolchain_text(tree)
    tree_inputs = TreeInputs(digest, mutations_sha, DEVICE_ARGUMENTS[segment], DEVICE_FEATURES, toolchain, judging_route(),
                             tuple((path, recorded.get(path) or sha256_file(os.path.join(REPOSITORY_ROOT, path))) for path in KEY_RESEARCH_FILES),
                             o3_freeze_digest())
    first_t1 = latest_first_segment_t1() if segment == "s2" else None
    plan_only = os.environ.get(PLAN_ONLY_VARIABLE) == "1"
    lines = ResultLines(os.path.join(plan_directory, PLAN_ONLY_PRODUCT) if plan_only else product_path(segment, "plan", date, suffix))
    lines.result(f"name=config script=full_table_run step=plan segment={segment} date={date} run_suffix={suffix or '-'} "
                 f"first_segment_t1={field(os.path.basename(first_t1) if first_t1 else '-')}")
    lines.result(f"name=snapshot snapshot_digest={digest} files={file_count} head={head} status_lines={len(status_lines)} "
                 f"mutations_table_sha256={mutations_sha} toolchain_sha256={sha256_bytes(toolchain.encode('utf-8'))} "
                 f"judging_route={tree_inputs.judging_route} o3_freeze_digest={tree_inputs.o3_freeze_digest} "
                 f"device_sha256={sha256_file(os.path.join(tree, e171_research_side.DEVICE_SOURCE))}")
    write_snapshot_status_lines(lines, snapshot)
    for path, digest_of_file in sorted(recorded.items()):
        lines.result(f"name=snapshot_file path={path} sha256={digest_of_file}")
    texts = list_lines_by_number()
    expected = sorted(set(range(1, 282)) - WITHDRAWN_ROWS)
    lines.result(f"name=list_rows rows={len(texts)} expected_rows={len(expected)} holds={str(sorted(texts) == expected).lower()}")
    numbers = segment_rows(segment, sorted(texts))
    rows = row_inputs_and_list_lines(lines, tree, numbers)
    plan = plan_reuse(tree_inputs, rows, store)
    os.makedirs(os.path.join(plan_directory, "sentinels"))
    for number, data in read_sentinel_entries(plan, store).items():
        with open(os.path.join(plan_directory, "sentinels", f"{number}.tar.gz"), "wb") as handle:
            handle.write(data)
    shards = [] if plan_only else split_into_shards(plan.fresh)
    if plan_only:
        for index, shard in enumerate(split_into_shards(plan.fresh)):
            lines.result(f"name=plan_only_shard shard={index} rows={len(shard)} numbers={field(','.join(map(str, shard)) or '-')}")
    lines.result(f"name=reuse rows={len(rows)} fresh={len(plan.fresh)} reused={len(plan.reused)} sentinels={len(plan.sentinels)}")
    lines.result(f"name=reuse_sample k={len(plan.sentinels)} rows={field(','.join(map(str, plan.sentinels)) or '-')} "
                 f"rule=max(2,ceil(0.02*M))_by_sha256_of_snapshot_and_key")
    for index, shard in enumerate(shards):
        lines.result(f"name=shard_assignment shard={index} rows={len(shard)} numbers={field(','.join(map(str, shard)) or '-')}")
    record = {"segment": segment, "date": date, "run_suffix": suffix, "head": head, "status_lines": status_lines, "recorded": recorded,
              "tree_inputs": dataclasses.asdict(tree_inputs), "keys": {str(number): key for number, key in plan.keys.items()},
              "fresh": list(plan.fresh), "reused": list(plan.reused), "sentinels": list(plan.sentinels), "shards": shards,
              "rows": [dataclasses.asdict(row) for row in rows], "first_segment_t1": first_t1, "snapshot_files": file_count}
    with open(os.path.join(plan_directory, "plan-record.json"), "w", encoding="utf-8") as handle:
        json.dump(record, handle, ensure_ascii=False, indent=1)
    with open(os.path.join(plan_directory, "plan.json"), "w", encoding="utf-8") as handle:
        json.dump({"shards": [{"label": f"shard-{index} rows={len(shard)}"} for index, shard in enumerate(shards)],
                   "note": f"E201 {segment}: fresh={len(plan.fresh)} reused={len(plan.reused)} sentinels={len(plan.sentinels)}"}, handle, ensure_ascii=False)
    lines.finish()
    return 0


def read_plan_record(plan_directory: str) -> dict:
    with open(os.path.join(plan_directory, "plan-record.json"), encoding="utf-8") as handle:
        return json.load(handle)


def plan_of_record(record: dict) -> ReusePlan:
    return ReusePlan({int(number): key for number, key in record["keys"].items()}, tuple(record["fresh"]), tuple(record["reused"]), tuple(record["sentinels"]))


def drift_from_the_plan(record: dict, tree: str, repository_root: str = REPOSITORY_ROOT) -> list[str]:
    """作废 Z3：线道要编的树（计划目录里那一份）的快照摘要、变异表与计划记的各文件 sha256，与计划那一刻记的不同的逐项列出。"""
    import row_driver
    problems = []
    if row_driver.snapshot_digest(tree)[0] != record["tree_inputs"]["snapshot_digest"]:
        problems.append("snapshot_digest")
    if sha256_file(os.path.join(tree, "crates", "mutations.tsv")) != record["tree_inputs"]["mutations_table_sha256"]:
        problems.append("mutations_table")
    for path, digest in sorted(record["recorded"].items()):
        on_disk = os.path.join(repository_root, path)
        if not os.path.isfile(on_disk) or sha256_file(on_disk) != digest:
            problems.append(path)
    return problems


def frozen_o3_differences(repository_root: str) -> list[str]:
    import frozen_o3
    return frozen_o3.freeze_differences(repository_root)


def shard_check_problems(record: dict, tree: str, repository_root: str = REPOSITORY_ROOT,
                         freeze_differences: Callable[[str], list[str]] = frozen_o3_differences) -> list[str]:
    """每片开头与收尾各核一次：Z3（drift_from_the_plan）与 Z2（冻结的 O3 与冻结行逐文件比，不同的记 freeze:<项>）。"""
    return drift_from_the_plan(record, tree, repository_root) + [f"freeze:{item}" for item in freeze_differences(repository_root)]


# ── 各片 ──

S2_FILES = ("base-sectors.bin", "ops.tsv", "ops-payload.bin", "calls.tsv", "torn.tsv", "torn-payload.bin", "states.tsv", "in-place-subsets.tsv")
O3_CONTROL_SEED = "85441ae491419411"
DEVICE_CONTROLS = ("O1-对拍", "O1-对拍-卸载", "O1-崩溃判官", "RefFS", "O2", "交回值", "内存对象", "panic")
STREAM_DIRECTORIES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
POLL_SECONDS = 20


def heavy_threads() -> int:
    """这一片的线程数：服务经 SINGLEFS_THREAD_CAP 给的那一个数（没设取登记的每台默认重活线程数）。"""
    sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
    from heavy_threads import default_heavy_threads
    return default_heavy_threads(os.environ)


def streams_of(segment: str) -> str:
    return DEVICE_ARGUMENTS[segment][2]


def stream_names_of(segment: str) -> list[str]:
    return [STREAM_DIRECTORIES[int(index) - 1] for index in streams_of(segment).split(",")]


def compile_clean_binaries(tree: str, work: str, threads: int, log: str) -> tuple[int, str, str]:
    """不改坏的树编一次装置（带 gpu 特性，逐行驱动同一条命令）与 E170 导出装置（不走判段入口，不带特性）。交回 (退出码, 装置, 导出装置)。"""
    import current_tree_row_driver as driver
    lane_tree, lane_target = os.path.join(work, "clean", "tree"), os.path.join(work, "clean", "target")
    driver.sync_tree(tree, lane_tree)
    exit_code, _ = driver.run_compile(lane_tree, lane_target, REPOSITORY_ROOT, threads, log, log + ".artifacts.jsonl")
    if exit_code == 0:
        import frozen_o3
        environment = frozen_o3.thread_environment(threads, dict(os.environ, CARGO_TARGET_DIR=lane_target))
        with open(log, "a", encoding="utf-8") as handle:
            exit_code = subprocess.run(["cargo", "build", "--release", "--offline", "-p", "singlefs-verification", "--bin", EXPORT_BINARY_NAME],
                                       cwd=lane_tree, env=environment, stdout=handle, stderr=subprocess.STDOUT).returncode
    release = os.path.join(lane_target, "release")
    return exit_code, os.path.join(release, DEVICE_BINARY_NAME), os.path.join(release, EXPORT_BINARY_NAME)


def run_device(binary: str, mode_arguments: list[str], output_directory: str, device_out: str, threads: int) -> int:
    """装置（或导出装置）套 E171 的 device_timing.py 直接起（不套别的包装），标准输出写进 device_out。"""
    import frozen_o3
    import current_tree_row_driver as driver
    command = [sys.executable, "-B", driver.DEVICE_TIMING_SCRIPT, "run", "--", binary, *mode_arguments, "--output-directory", output_directory]
    with open(device_out, "w", encoding="utf-8") as out, open(device_out + ".err", "w", encoding="utf-8") as err:
        return subprocess.run(command, cwd=REPOSITORY_ROOT, env=frozen_o3.thread_environment(threads), stdout=out, stderr=err).returncode


def verdict_fields(device_out: str) -> dict[str, str]:
    from run_directory import result_lines_named
    found = result_lines_named(device_out, "verdict") if os.path.exists(device_out) else []
    return found[-1] if found else {}


def not_run_count(paths: list[str]) -> int:
    """Z6：装置的标准输出与它写出的每份产物里含 NOT_RUN 的行数合计（.gz 按 gzip 读；目录逐份数）。"""
    import gzip
    total = 0
    for path in paths:
        files = [os.path.join(base, name) for base, _, names in os.walk(path) for name in names] if os.path.isdir(path) else [path]
        for file in files:
            if not os.path.isfile(file):
                continue
            opener = gzip.open if file.endswith(".gz") else open
            with opener(file, "rb") as handle:
                total += sum(b"NOT_RUN" in line for line in handle)
    return total


def without_timing(path: str) -> bytes:
    """impl-verdicts.tsv 去掉耗时那一列（E171 第一段比对器 Z1 的同一种去法）。"""
    import segment_one_comparator
    return segment_one_comparator.lines_without_the_last_column(path)


def export_differences(left: str, right: str, stream_names: list[str], files: tuple[str, ...]) -> list[str]:
    """两份导出逐文件比（停机 S2、作废 Z1、跨机对照 S7 共用）：这几个文件 sha256 相同、impl-verdicts.tsv 去掉耗时那一列相同。交回不同的 <流>/<文件>。"""
    differing = []
    for stream in stream_names:
        for name in files:
            one, other = os.path.join(left, stream, name), os.path.join(right, stream, name)
            if not (os.path.exists(one) and os.path.exists(other) and sha256_file(one) == sha256_file(other)):
                differing.append(f"{stream}/{name}")
        one, other = os.path.join(left, stream, "impl-verdicts.tsv"), os.path.join(right, stream, "impl-verdicts.tsv")
        if not (os.path.exists(one) and os.path.exists(other) and without_timing(one) == without_timing(other)):
            differing.append(f"{stream}/impl-verdicts.tsv")
    return differing


# 作废 Z1 与跨机对照 S7 逐文件比的那几份（call-return-values.tsv 另比，见 stable_return_value_differences）
Z1_FILES = S2_FILES + ("oracle-verdicts.tsv", "call-outcomes.tsv")


def stable_return_value_differences(first: str, second: str, stream_names: list[str]) -> list[str]:
    """Z1 的 call-return-values.tsv：拿掉两次基线就不稳的内存对象（E171 第 1 次重跑读法 4）之后，交回值与其余对象逐次调用比，不同的记 <流>/call-return-values.tsv:<调用>。"""
    import collusion_matrix as matrix
    from run_directory import load_run_directory
    first_run, second_run = load_run_directory(first), load_run_directory(second)
    unstable = matrix.unstable_memory_objects(first_run, second_run)
    return [f"{difference.stream}/call-return-values.tsv:{difference.call_index}"
            for difference in matrix.return_value_differences(first_run, second_run, unstable, tuple(stream_names))]


def second_baseline_differences(first: str, second: str, stream_names: list[str],
                                return_values: Callable[[str, str, list[str]], list[str]] = stable_return_value_differences) -> list[str]:
    """作废 Z1：片 0 两次基线的导出逐文件比（impl-verdicts.tsv 去掉耗时那一列）加 call-return-values.tsv 去掉不稳对象之后比。交回不同的那几样。"""
    return export_differences(first, second, stream_names, Z1_FILES) + return_values(first, second, stream_names)


def export_against_the_baseline(dump_exit: int, baseline: str, export_dump: str, stream_names: list[str]) -> list[str]:
    """停机 S2：E170 导出装置 dump 写出的八个文件与基线导出逐文件比（流一、流二）；dump 没跑成记 dump_failed（不当相同）。"""
    if dump_exit != 0:
        return ["dump_failed"]
    return export_differences(baseline, export_dump, stream_names[:2], S2_FILES)


class CollectedLines:
    """把比对器一个函数写的结果行收进内存（不落盘）：A7 与哨兵比判定格时用。"""

    def __init__(self):
        self.bodies: list[str] = []

    def result(self, body: str) -> None:
        self.bodies.append(body)


def comparator_row_lines(baseline_path: str, rows_root: str, number: int) -> tuple[str, ...]:
    """E171 第二段比对器对一行出的东西：row_line 那一行，加这一行与基线逐次调用比出的交回值差（q2 那几种行由它而来）。"""
    import collusion_matrix as matrix
    import segment_two_comparator as segment_two
    from run_directory import TOUCH_STREAM_NAMES, load_run_directory
    baseline = load_run_directory(baseline_path)
    collected = CollectedLines()
    reading = segment_two.read_the_row(baseline, rows_root, number)
    segment_two.row_line(collected, reading)
    if reading.run is not None:
        collected.bodies.extend(repr(difference) for difference in matrix.return_value_differences(baseline, reading.run, frozenset(), TOUCH_STREAM_NAMES))
    return tuple(collected.bodies)


def stored_row_differences(baseline_path: str, row_directory: str, number: int, scratch: str,
                           row_lines_of: Callable[[str, str, int], tuple[str, ...]] = comparator_row_lines) -> list[str]:
    """第七节 A7：这一行存进库的那一份（pack_row_directory，不带三个大文件）摆成行目录，比对器在它上面与在完整导出上出的东西逐字相同。
    不同交回 [<行>:stored_row_differs]。完整导出的行目录名要是 row-<行>（比对器按这个名字找）。"""
    stored_root = os.path.join(scratch, f"stored-row-{number}")
    shutil.rmtree(stored_root, ignore_errors=True)
    unpack_row_directory(pack_row_directory(row_directory), os.path.join(stored_root, f"row-{number}"))
    full = row_lines_of(baseline_path, os.path.dirname(row_directory), number)
    stored = row_lines_of(baseline_path, stored_root, number)
    shutil.rmtree(stored_root, ignore_errors=True)
    return [] if full == stored else [f"{number}:stored_row_differs"]


def driver_command(record: dict, plan_directory: str, work_root: str, output_root: str, baseline_timing: str, timing_output: str,
                   lanes: int, extra: list[str]) -> list[str]:
    """逐行驱动的整条命令（不套包装；每线道 6 线程）。"""
    return [sys.executable, "-B", os.path.join(HERE, "current_tree_row_driver.py"), "--repo-root", REPOSITORY_ROOT, "--work-root", work_root,
            "--output-root", output_root, "--snapshot-from", os.path.join(plan_directory, "tree"), "--streams", streams_of(record["segment"]),
            "--threads", str(LANE_THREADS), "--lanes", str(lanes), "--baseline-timing", baseline_timing, "--timing-output", timing_output, *extra]


def run_driver(command: list[str], log: str, on_poll: Callable[[], None] | None = None) -> int:
    """起逐行驱动、把它的标准输出写进 log；给了 on_poll 就每 POLL_SECONDS 秒调一次（把判完的行入库：被打断重交时已入库的行不重跑）。"""
    import time
    with open(log, "w", encoding="utf-8") as handle:
        process = subprocess.Popen(command, cwd=REPOSITORY_ROOT, stdout=handle, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            while process.poll() is None:
                time.sleep(POLL_SECONDS)
                if on_poll is not None:
                    on_poll()
        except BaseException:
            os.killpg(process.pid, 9)
            process.wait()
            raise
    if on_poll is not None:
        on_poll()
    return process.returncode


def store_finished_rows(plan: ReusePlan, numbers: list[int], rows_root: str, store, stored: dict[int, int]) -> None:
    """判完的行（行目录里有 row-result.tsv）入库，记下它的 NOT_RUN 行数；已入库的不重复写。逐行驱动带 --keep-dumps 跑（A7 要完整导出），
    入库之后哨兵以外的行删掉三个大文件（哨兵留着，片收尾在它上面核 A7）。"""
    import current_tree_row_driver as driver
    for number in numbers:
        row_directory = os.path.join(rows_root, f"row-{number}")
        if number in stored or not os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            continue
        stored[number] = not_run_count([os.path.join(row_directory, "device.out"), os.path.join(row_directory, "dump")])
        store.write(library_relative(plan.keys[number]), pack_row_directory(row_directory))
        if number not in plan.sentinels:
            driver.remove_bulky_dump_files(os.path.join(row_directory, "dump"))


def control_units(record: dict, plan_directory: str, output_directory: str, work: str, baseline_timing: str, binary: str,
                  export_dump: str, threads: int) -> list[tuple[str, Callable[[], int]]]:
    """片 0 的前置里互相独立的单元（第十四节）：八样装置对照在基线上各一趟、九样对照在空变异上各一趟（逐行驱动，各用自己的工作根）。"""
    controls = os.path.join(output_directory, "controls")
    units: list[tuple[str, Callable[[], int]]] = []

    def device_control(control: str) -> int:
        directory = os.path.join(controls, f"baseline-{control}")
        os.makedirs(directory)
        return run_device(binary, ["baseline", "--streams", "1,2", "--positive-control", control], os.path.join(directory, "dump"),
                          os.path.join(directory, "device.out"), threads)

    def empty_mutation_control(control: str) -> int:
        root = os.path.join(controls, f"empty-{control}")
        extra = ["--empty-mutation", "--positive-control", control, "--rows", ""]
        if control == "O3":
            extra += ["--o3-control-bad-states", export_dump]
        return run_driver(driver_command(record, plan_directory, os.path.join(work, f"control-{control}"), root, baseline_timing,
                                         root + ".timing", 1, extra), root + ".driver.out")

    for control in DEVICE_CONTROLS:
        units.append((f"baseline:{control}", lambda control=control: device_control(control)))
    for control in (*DEVICE_CONTROLS, "O3"):
        units.append((f"empty:{control}", lambda control=control: empty_mutation_control(control)))
    return units


def run_units_in_parallel(units: list[tuple[str, Callable[[], int]]], width: int) -> dict[str, int]:
    """互相独立的单元按 width 路并行（每路一个子进程）；交回 {单元: 退出码}，派出去几个收回几个。"""
    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor(max_workers=max(1, width)) as pool:
        futures = {name: pool.submit(job) for name, job in units}
        results = {name: future.result() for name, future in futures.items()}
    assert len(results) == len(units), "并行单元收回的个数与派出去的不等"
    return results


def shard_preliminaries(lines: ResultLines, record: dict, plan_directory: str, output_directory: str, work: str, binary: str,
                        export_binary: str, baseline: str, baseline_timing: str, threads: int, lanes: int) -> None:
    """片 0 的前置（第五节 5.5、第十一节 Z1 与 S2）：第二次基线与第一次逐文件比、E170 导出装置 dump 与基线导出逐文件比、O3 基线、
    O3 阳性对照在基线上一趟，再把十七个对照单元按线道数并行跑完。结论都写成行，判在合并那一步。"""
    import frozen_o3
    names = stream_names_of(record["segment"])
    second = os.path.join(output_directory, "baseline-second")
    second_exit = run_device(binary, ["baseline", "--streams", streams_of(record["segment"])], second, second + ".device.out", threads)
    # 退 3 是装置报了停机条款、导出照写（S 类另判）；别的码没有可比的导出，不当相同
    differing = second_baseline_differences(baseline, second, names) if second_exit in (0, 3) else ["second_baseline_failed"]
    lines.result(f"name=z1_second_baseline exit={second_exit} differing={field(','.join(differing) or '-')}")
    export_dump = os.path.join(output_directory, "e170-dump")
    dump_exit = run_device(export_binary, ["dump"], export_dump, export_dump + ".device.out", threads)
    bad_exit = subprocess.run([export_binary, "bad-states", "--dump-directory", export_dump, "--seed-hex", O3_CONTROL_SEED], cwd=REPOSITORY_ROOT,
                              env=frozen_o3.thread_environment(threads), capture_output=True).returncode if dump_exit == 0 else -1
    s2 = export_against_the_baseline(dump_exit, baseline, export_dump, names)
    lines.result(f"name=s2_export_against_the_baseline dump_exit={dump_exit} bad_states_exit={bad_exit} differing={field(','.join(s2) or '-')}")
    o3_exit, _ = frozen_o3.run_executor(REPOSITORY_ROOT, baseline, names, threads, os.path.join(output_directory, "baseline-o3.out"),
                                        os.path.join(output_directory, "baseline-o3.log"))
    control_out = os.path.join(output_directory, "controls", "baseline-O3.out")
    os.makedirs(os.path.dirname(control_out), exist_ok=True)
    import current_tree_row_driver as driver
    o3_control_exit = subprocess.run([sys.executable, driver.IMAGE_JUDGE_SCRIPT, "control", "--repo-root", REPOSITORY_ROOT, "--dumps", baseline,
                                      "--e170-dumps", export_dump, "--row-name", driver.O3_CONTROL_ROW_NAME, "--output", control_out],
                                     cwd=REPOSITORY_ROOT, env=frozen_o3.thread_environment(threads), capture_output=True).returncode
    lines.result(f"name=o3_baseline exit={o3_exit} o3_control_exit={o3_control_exit}")
    if record["segment"] != "s1":
        lines.result("name=controls run=false why=positive_controls_run_only_in_the_first_segment_on_streams_1_2")
        return
    results = run_units_in_parallel(control_units(record, plan_directory, output_directory, work, baseline_timing, binary, export_dump,
                                                  LANE_THREADS), lanes)
    for name, exit_code in sorted(results.items()):
        lines.result(f"name=control_unit unit={field(name)} exit={exit_code}")


def shard_step(plan_directory: str, output_directory: str, shard_index: int, store) -> int:
    record = read_plan_record(plan_directory)
    plan = plan_of_record(record)
    tree = os.path.join(plan_directory, "tree")
    numbers = record["shards"][shard_index]
    threads = heavy_threads()
    lanes = max(1, threads // LANE_THREADS)
    lines = ResultLines(os.path.join(output_directory, "shard.out"))
    lines.result(f"name=shard_config shard={shard_index} host={field(os.environ.get('TASK_SERVER_SHARD_HOST', '-'))} rows={len(numbers)} "
                 f"threads={threads} lanes={lanes} threads_per_lane={LANE_THREADS}")
    problems = shard_check_problems(record, tree)
    lines.result(f"name=shard_check moment=start problems={field(','.join(problems) or '-')}")
    if problems:
        lines.finish()
        return EXIT_INVALID
    work = os.path.join(os.environ.get("CARGO_TARGET_DIR") or os.path.join(output_directory, "work"), f"e201-shard-{shard_index}")
    stored: dict[int, int] = {}
    try:
        compile_exit, binary, export_binary = compile_clean_binaries(tree, work, threads, os.path.join(output_directory, "clean-compile.log"))
        lines.result(f"name=clean_compile exit={compile_exit}")
        if compile_exit != 0:  # 停机 S6：计划拷进来的树编不过
            lines.result("name=stop clause=S6")
            lines.finish()
            return EXIT_INVALID
        baseline = os.path.join(output_directory, "baseline")
        baseline_out = baseline + ".device.out"
        baseline_exit = run_device(binary, ["baseline", "--streams", streams_of(record["segment"])], baseline, baseline_out, threads)
        verdict = verdict_fields(baseline_out)
        lines.result(f"name=baseline exit={baseline_exit} " + " ".join(f"{key}={field(value)}" for key, value in sorted(verdict.items())))
        if shard_index == 0:
            shard_preliminaries(lines, record, plan_directory, output_directory, work, binary, export_binary, baseline, baseline_out, threads, lanes)
        empty_root = os.path.join(output_directory, "empty")
        # 空变异与分到的行用同一个工作根（同一张快照，线道 0 的编译目录接着增量编；逐行驱动的编前编后核对管串行）
        # 空变异与分到的行都带 --keep-dumps：A7 要完整导出比，跨机对照 S7 比空变异导出里的三个大文件
        empty_exit = run_driver(driver_command(record, plan_directory, os.path.join(work, "rows"), empty_root, baseline_out,
                                               empty_root + ".timing", 1, ["--empty-mutation", "--keep-dumps", "--rows", ""]), empty_root + ".driver.out")
        lines.result(f"name=empty_mutation exit={empty_exit}")
        rows_root = os.path.join(output_directory, "rows")
        rows_exit = 0
        if numbers:
            rows_exit = run_driver(driver_command(record, plan_directory, os.path.join(work, "rows"), rows_root, baseline_out, rows_root + ".timing",
                                                  lanes, ["--keep-dumps", "--rows", ",".join(map(str, numbers))]),
                                   os.path.join(output_directory, "rows.driver.out"), lambda: store_finished_rows(plan, numbers, rows_root, store, stored))
        lines.result(f"name=rows_driver exit={rows_exit} rows={len(numbers)} stored={len(stored)}")
        for number in numbers:
            lines.result(f"name=shard_row row={number} stored={str(number in stored).lower()} not_run={stored.get(number, '-')}")
        # A7：空变异与这一片的每一行哨兵，各核一次存进库那一份与完整导出在比对器上逐字相同
        checked_rows = [(0, os.path.join(empty_root, "row-0"))] + [(number, os.path.join(rows_root, f"row-{number}"))
                                                                   for number in sorted(set(plan.sentinels) & set(numbers))]
        for number, row_directory in checked_rows:
            differing = (stored_row_differences(baseline, row_directory, number, os.path.join(work, "stored-row-check"))
                         if os.path.exists(os.path.join(row_directory, "row-result.tsv")) else [f"{number}:row_not_finished"])
            lines.result(f"name=a7 row={number} differing={field(','.join(differing) or '-')}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    problems = shard_check_problems(record, tree)
    lines.result(f"name=shard_check moment=end problems={field(','.join(problems) or '-')}")
    lines.finish()
    return EXIT_INVALID if problems or len(stored) != len(numbers) else 0


# ── 合并 ──

class FilteredLines:
    """把 E171 第一段比对器那几个函数写的行按这一段要的转给真的产物（读法 7 的 T3：只取流一、流二那一半，或第二段只取流三那一半）。"""

    def __init__(self, target: ResultLines, keep: Callable[[str], bool]):
        self.target, self.keep = target, keep

    def result(self, body: str) -> None:
        if self.keep(body):
            self.target.result(body)


def t3_keeps(segment: str) -> Callable[[str], bool]:
    first = segment == "s1"

    def keep(body: str) -> bool:
        padded = body + " "
        if body.startswith("name=q1 "):
            return (" streams=two_streams " if first else " streams=three_streams ") in padded
        if body.startswith("name=q1_row "):
            return not first
        if body.startswith("name=q2_named_row "):
            return (" streams=streams_1_2 " in padded or " ran=false " in padded) if first else " streams=stream_3 " in padded
        if body.startswith("name=q2_named_row_call "):
            return (f" stream={STREAM_DIRECTORIES[2]} " in padded) != first
        return True
    return keep


def write_t3(path: str, segment: str, baseline_path: str, rows_root: str, stability: frozenset[str]) -> None:
    """T3（读法 7）：照 E171 第一段比对器的 q1_lines、row_column_lines、q2_named_row_lines、q3_lines 原样调，第一段只取流一、流二，第二段只取流三。"""
    import collusion_matrix as matrix
    import segment_one_comparator as segment_one
    from run_directory import TOUCH_STREAM_NAMES, load_run_directory
    lines = ResultLines(path)
    lines.result(f"name=config script=full_table_run part=t3 segment={segment}")
    filtered = FilteredLines(lines, t3_keeps(segment))
    baseline = load_run_directory(baseline_path)
    verdicts = segment_one.q1_lines(filtered, baseline, rows_root)
    if segment == "s1":
        for number in segment_one.Q1_ROWS:
            row_directory = os.path.join(rows_root, f"row-{number}")
            result = segment_one.read_row_result(row_directory)
            if result.get("status") != "ok" or result.get("device_exit") not in ("0", "3"):
                continue
            run = load_run_directory(os.path.join(row_directory, "dump"))
            if all(run.streams[name].present() for name in TOUCH_STREAM_NAMES) and matrix.touch_report(baseline, run).touched():
                for batch in (matrix.FIRST_BATCH, matrix.SECOND_BATCH):
                    segment_one.row_column_lines(lines, number, run, segment_one.read_o3(os.path.join(row_directory, "o3.out")), batch,
                                                 TOUCH_STREAM_NAMES)
    segment_one.q2_named_row_lines(filtered, baseline, rows_root, stability)
    if segment == "s1":
        lines.result(f"name=verdict q3={segment_one.q3_lines(lines, baseline, rows_root)}")
    else:
        lines.result(f"name=verdict q1={verdicts.get('q1', 'not_run')}")
    lines.finish()


def write_t2(path: str, record: dict, rows_root: str, row_time_limit_line: str) -> None:
    """T2（与 P2 同形）：逐行驱动的头几行与每行一条 `row row_number=…`（各行的 row-result.tsv 原样，值里的空白换下划线）。"""
    import segment_one_comparator as segment_one
    numbers = sorted(int(name.removeprefix("row-")) for name in os.listdir(rows_root) if name.removeprefix("row-").isdigit())
    with open(path, "x", encoding="utf-8") as handle:
        handle.write(row_time_limit_line.rstrip("\n") + "\n")
        handle.write(f"snapshot_digest={record['tree_inputs']['snapshot_digest']} files={record['snapshot_files']} snapshot_ns=0 rows={len(numbers)}\n")
        for number in numbers:
            fields = segment_one.read_row_result(os.path.join(rows_root, f"row-{number}"))
            handle.write("row " + " ".join(f"{key}={str(value).replace(' ', '_')}" for key, value in fields.items()) + "\n")
        handle.write(f"table rows={len(numbers)}\n")


def row_decisions(data: bytes, baseline, scratch: str) -> tuple[tuple[str, str], ...]:
    """哨兵比的判定格（Z7）：一行存进库的那一份摆成行目录，照第二段比对器的 row_line 出那一行，取读法 7 归成判定格的字段。"""
    import contrast_against_r1 as contrast
    import segment_two_comparator as segment_two
    shutil.rmtree(scratch, ignore_errors=True)
    unpack_row_directory(data, os.path.join(scratch, "row-1"))
    collected = CollectedLines()
    segment_two.row_line(collected, segment_two.read_the_row(baseline, scratch, 1))
    classes = contrast.load_cell_classes()
    fields = contrast.line_fields(collected.bodies[0].split(" ")[1:])
    shutil.rmtree(scratch, ignore_errors=True)
    return tuple(sorted((name, value) for name, value in fields.items() if classes.class_of("P1", "row", name) in contrast.DECISION_CLASSES))


def shard_outputs() -> list[tuple[int, str, str]]:
    """TASK_SERVER_SHARD_OUTPUTS：每片一行「片号<TAB>台<TAB>产物目录」。"""
    rows = []
    with open(os.environ["TASK_SERVER_SHARD_OUTPUTS"], encoding="utf-8") as handle:
        for line in handle:
            if line.strip():
                index, host, directory = line.rstrip("\n").split("\t")
                rows.append((int(index), host, directory))
    return sorted(rows)


def shard_lines(directory: str) -> list[dict[str, str]]:
    import contrast_against_r1 as contrast
    path = os.path.join(directory, "shard.out")
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as handle:
        return [dict(fields, name=kind) for kind, fields in contrast.read_product_entries(handle.read())]


# K3（登记 5.2 第一张表）装置自己报计数的五样：判决行里哪个字段、恰几（两遍都是流一、流二；E171 第 1 次重跑登记 A8 空变异那一种）
DEVICE_COUNTED_CONTROLS = {"O1-对拍": ("o1_comparison_red_calls", 1), "O1-对拍-卸载": ("o1_comparison_red_calls", 1),
                           "O1-崩溃判官": ("o1_crash_judge_red_states", 2), "RefFS": ("o1_reference_filesystem_red_states", 2), "O2": ("o2_red_states", 1)}
# 阳性对照 panic 的造法：装置在流二从 0 数第 3 次调用的闭包里先 panic（入库装置 PANIC_CONTROL_CALL），这条流截在这次调用
PANIC_CONTROL_STREAM = STREAM_DIRECTORIES[1]
PANIC_CONTROL_CALL = 3


def device_control_judgement(control: str, device_out: str) -> tuple[str, bool]:
    """K3 的前五样：装置标准输出里 name=verdict 各行那个字段相加，等于登记的精确数。交回 (读数, 成不成立)；没有输出记 missing。"""
    from run_directory import result_lines_named
    if not os.path.exists(device_out):
        return "missing", False
    key, expected = DEVICE_COUNTED_CONTROLS[control]
    observed = sum(int(verdict.get(key, "0")) for verdict in result_lines_named(device_out, "verdict"))
    return str(observed), observed == expected


def dump_control_judgement(control: str, baseline, run) -> tuple[str, bool]:
    """K3 的交回值、内存对象、panic 三样，从这一趟的导出（run_directory.RunDirectory）现算：交回值恰 1 次调用不同且那一次 O1-对拍 agrees；
    内存对象恰 1 次调用不同；panic 是流二导出了、第一处 panic 在第 3 次调用（从 0 数）且这条流截在这次调用、之前 O1-对拍判过的调用恰 1 次、取样态数大于 0。"""
    import collusion_matrix as matrix
    from run_directory import TOUCH_STREAM_NAMES
    if control == "panic":
        stream = run.streams[PANIC_CONTROL_STREAM]
        calls = stream.calls if stream.present() else []
        panic_at = next((call.index for call in calls if call.panicked()), None)
        last_call = calls[-1].index if calls else None
        judged = matrix.judged_calls_before_the_first_panic(run)
        states = sum(len(run.streams[name].states) for name in TOUCH_STREAM_NAMES if run.streams[name].present())
        observed = f"exported={str(stream.exported).lower()}:panic_at={panic_at}:last_call={last_call}:judged={judged}:states={states}"
        holds = stream.exported and panic_at == PANIC_CONTROL_CALL and last_call == PANIC_CONTROL_CALL and judged == 1 and states > 0
        return observed, holds
    differences = matrix.return_value_differences(baseline, run, frozenset(), TOUCH_STREAM_NAMES)
    if control == "交回值":
        differing = [difference for difference in differences if difference.differing_fields]
        agreeing = [difference for difference in differing if segment_two_model_word(run, difference) == "agrees"]
        return str(len(differing)), len(differing) == 1 == len(agreeing)
    objects = [difference for difference in differences if difference.differing_objects]
    return str(len(objects)), len(objects) == 1


def control_lines(lines: ResultLines, shard_zero: str, baseline_path: str, segment: str) -> int:
    """阳性（K3，两遍：基线、空变异）与阴性（K2）对照；交回不等的格数（F1、F2 各自在产物里点名）。"""
    import segment_one_comparator as segment_one
    from run_directory import load_run_directory, result_lines_named
    baseline = load_run_directory(baseline_path)
    failing = segment_one.a1_lines(lines, baseline, segment_one.read_o3(os.path.join(shard_zero, "baseline-o3.out")), "k2_baseline")
    empty_row = os.path.join(shard_zero, "empty", "row-0")
    if os.path.isdir(os.path.join(empty_row, "dump")):
        reds, touched = segment_one.empty_mutation_a1(lines, baseline, load_run_directory(os.path.join(empty_row, "dump")),
                                                      segment_one.read_o3(os.path.join(empty_row, "o3.out")))
        failing += reds + touched
    else:
        lines.result("name=k2_empty_mutation ran=false")
        failing += 1
    if segment != "s1":
        return failing
    for pass_name in ("baseline", "empty"):
        for control in DEVICE_CONTROLS:
            if pass_name == "baseline":
                device_out, dump = (os.path.join(shard_zero, "controls", f"baseline-{control}", name) for name in ("device.out", "dump"))
            else:
                device_out, dump = (os.path.join(shard_zero, "controls", f"empty-{control}", "row-0", name) for name in ("device.out", "dump"))
            if control in DEVICE_COUNTED_CONTROLS:
                observed, holds = device_control_judgement(control, device_out)
            elif os.path.isdir(dump):
                observed, holds = dump_control_judgement(control, baseline, load_run_directory(dump))
            else:
                observed, holds = "missing", False
            failing += not holds
            lines.result(f"name=k3 pass={pass_name} control={field(control)} observed={field(observed)} holds={str(holds).lower()}")
        o3_out = os.path.join(shard_zero, "controls", "baseline-O3.out") if pass_name == "baseline" else \
            os.path.join(shard_zero, "controls", "empty-O3", "row-0", "o3-control.out")
        verdicts = result_lines_named(o3_out, "o3_control_verdict") if os.path.exists(o3_out) else []
        holds = bool(verdicts) and verdicts[-1].get("red_strict") == "true"
        failing += not holds
        lines.result(f"name=k3 pass={pass_name} control=O3 observed={field(verdicts[-1].get('strict', '-') if verdicts else 'missing')} holds={str(holds).lower()}")
    return failing


def segment_two_model_word(run, difference) -> str:
    import segment_two_comparator as segment_two
    return segment_two.model_word_of_the_call(run, difference.stream, difference.call_index)


def merge_step(plan_directory: str, store) -> int:
    if os.environ.get(PLAN_ONLY_VARIABLE) == "1":  # 干跑：计划写了几行就原样打进任务日志，不起任何一片、不出产物
        with open(os.path.join(plan_directory, PLAN_ONLY_PRODUCT), encoding="utf-8") as handle:
            sys.stdout.write(handle.read())
        return 0
    import attribution
    import contrast_against_r1 as contrast
    import q2_misses
    import segment_two_comparator as segment_two
    import whole_table_matrix
    from run_directory import load_run_directory
    record = read_plan_record(plan_directory)
    plan = plan_of_record(record)
    segment, date, suffix = record["segment"], record["date"], record["run_suffix"]
    merge = os.path.join(plan_directory, "merge")
    shutil.rmtree(merge, ignore_errors=True)
    rows_root = os.path.join(merge, "rows")
    os.makedirs(rows_root)
    outputs = shard_outputs()
    shard_zero = outputs[0][2]
    for name in ("baseline", "baseline-second"):
        if os.path.isdir(os.path.join(shard_zero, name)):
            shutil.copytree(os.path.join(shard_zero, name), os.path.join(merge, name))
    missing = []
    for number in sorted(plan.keys):
        data = store.read(library_relative(plan.keys[number]))
        if data is None:
            missing.append(number)
        else:
            unpack_row_directory(data, os.path.join(rows_root, f"row-{number}"))
    baseline_path = os.path.join(merge, "baseline")
    baseline = load_run_directory(baseline_path)
    sentinel_before = {}
    for number in plan.sentinels:
        with open(os.path.join(plan_directory, "sentinels", f"{number}.tar.gz"), "rb") as handle:
            sentinel_before[number] = handle.read()
    sentinel_after = {number: store.read(library_relative(plan.keys[number])) for number in plan.sentinels}
    sentinels = sentinel_problems(plan, sentinel_before, {number: data for number, data in sentinel_after.items() if data is not None},
                                  lambda data: row_decisions(data, baseline, os.path.join(merge, "sentinel-scratch")))
    states = outcome_states(plan, {number: b"" for number in plan.keys if number not in missing and number not in plan.reused})
    account = account_problems(plan, states, len(plan.keys))
    path_of = {part: product_path(segment, part, date, suffix) for part in ("t1", "t2", "t3", "contrast", "attribution", "q2", "matrix", "controls", "verdict")}
    with open(os.path.join(shard_zero, "rows.driver.out") if os.path.exists(os.path.join(shard_zero, "rows.driver.out"))
              else os.path.join(shard_zero, "empty.driver.out"), encoding="utf-8") as handle:
        time_limit_line = next((line for line in handle if line.startswith("name=row_time_limit ")), "name=row_time_limit seconds=-\n")
    write_t2(path_of["t2"], record, rows_root, time_limit_line)
    stability = [os.path.join(merge, "baseline-second")] if os.path.isdir(os.path.join(merge, "baseline-second")) else []
    t1_exit = segment_two.main(["--baseline", baseline_path, *[item for path in stability for item in ("--stability-baseline", path)],
                                "--rows-root", rows_root, "--driver-product", path_of["t2"],
                                "--mutations-table", os.path.join(plan_directory, "tree", "crates", "mutations.tsv"), "--dumps-root", merge,
                                "--product-prefix", f"{PRODUCT_STEM}-{date}{suffix}-{segment}-", "--product-suffix=-t2.out", "--output", path_of["t1"],
                                "--table-rows", str(len(plan.keys))])
    unstable = frozenset()
    if stability:
        import collusion_matrix as matrix
        unstable = matrix.unstable_memory_objects(baseline, load_run_directory(stability[0]))
    write_t3(path_of["t3"], segment, baseline_path, rows_root, unstable)
    texts = {}
    for part in ("t1", "t2", "t3"):
        with open(path_of[part], encoding="utf-8") as handle:
            texts[part] = handle.read()
    r1 = {}
    for product, (relative, _, _) in contrast.R1_PRODUCTS.items():
        with open(os.path.join(REPOSITORY_ROOT, relative), encoding="utf-8") as handle:
            r1[product] = handle.read()
    r1_problems = contrast.r1_product_problems(REPOSITORY_ROOT, {product: os.path.join(REPOSITORY_ROOT, relative) for product, (relative, _, _) in contrast.R1_PRODUCTS.items()})
    contrast_segment = contrast.SEGMENT_STREAMS_ONE_TWO if segment == "s1" else contrast.SEGMENT_STREAM_THREE
    pairs = [("P1", r1["P1"], texts["t1"]), ("P2", r1["P2"], texts["t2"]), ("P3", r1["P3"], texts["t3"])] if segment == "s1" else [("P3", r1["P3"], texts["t3"])]
    comparison = contrast.compare_segment(pairs, contrast.load_cell_classes(), contrast_segment)
    lines = ResultLines(path_of["contrast"])
    lines.result(f"name=config script=full_table_run part=contrast segment={contrast_segment} classes_sha256={contrast.CLASSES_SHA256} "
                 f"r1_problems={field(','.join(r1_problems) or '-')} unclassified={field(','.join(comparison.unclassified) or '-')}")
    contrast.write_comparison_lines(lines, comparison, contrast_segment)
    self_check = contrast.self_check_problems(comparison)
    lines.result(f"name=contrast_self_check segment={contrast_segment} problems={len(self_check)} detail={field(','.join(self_check) or '-')}")
    lines.finish()
    with open(path_of["contrast"], encoding="utf-8") as handle:
        contrast_text = handle.read()
    reference_root = os.path.join(merge, "reference")
    attribution.extract_reference_tree(REPOSITORY_ROOT, reference_root)
    uncommitted = frozenset(line[3:] for line in record["status_lines"])
    inputs = attribution.collect_inputs(REPOSITORY_ROOT, os.path.join(plan_directory, "tree"), reference_root, r1["P2"], texts["t2"], rows_root,
                                        uncommitted, os.path.join(merge, "attribution-scratch"))
    lines = ResultLines(path_of["attribution"])
    lines.result(f"name=config script=full_table_run part=attribution segment={contrast_segment}")
    q1 = attribution.attribute_cells(lines, contrast_text, inputs, contrast_segment, record["head"])
    lines.finish()
    first_t1 = texts["t1"] if segment == "s1" else open(record["first_segment_t1"], encoding="utf-8").read() if record["first_segment_t1"] else ""
    lines = ResultLines(path_of["q2"])
    lines.result(f"name=config script=full_table_run part=q2 segment={contrast_segment}")
    q2, q2_agrees = q2_misses.q2_lines(lines, r1["P1"], r1["P3"], first_t1, texts["t3"], contrast_segment)
    lines.finish()
    lines = ResultLines(path_of["matrix"])
    lines.result(f"name=config script=full_table_run part=matrix segment={segment}")
    whole_table_matrix.matrix_lines(lines, rows_root, texts["t1"], tuple(stream_names_of(segment)))
    lines.finish()
    return finish_merge(path_of, record, plan, outputs, shard_zero, baseline_path, missing, sentinels, account, t1_exit, self_check,
                        r1_problems, q1, q2, q2_agrees)


def cross_host_comparison(outputs: list[tuple[int, str, str]], stream_names: list[str]) -> tuple[str, list[str]]:
    """跨机对照（停机 S7）：两片落在两台时，两片的基线导出与空变异导出逐文件比（耗时那一列去掉）。交回 (same / differs / not_run_same_host, 不同的那几样)。"""
    if len(outputs) < 2 or outputs[0][1] == outputs[1][1]:
        return "not_run_same_host", []
    one, other = outputs[0][2], outputs[1][2]
    differing = export_differences(os.path.join(one, "baseline"), os.path.join(other, "baseline"), stream_names, Z1_FILES)
    differing += [f"empty:{item}" for item in export_differences(os.path.join(one, "empty", "row-0", "dump"), os.path.join(other, "empty", "row-0", "dump"),
                                                                   stream_names, Z1_FILES)]
    cross = "differs" if differing else "same"
    return cross, differing


def not_run_paths(outputs: list[tuple[int, str, str]], merged_rows_root: str) -> list[str]:
    """作废 Z6 要数的：每片每个走判段入口的装置单元的标准输出与它写出的导出（两次基线、空变异、对照），加合并那一步从库里摆出的各行（device.out 与导出）。
    E170 导出装置不走判段入口，不数。"""
    paths = []
    for _, _, directory in outputs:
        for name in ("baseline.device.out", "baseline", "baseline-second.device.out", "baseline-second", "empty", "controls"):
            paths.append(os.path.join(directory, name))
    return paths + [merged_rows_root]


# 读法 5 那几类（不进分母、照数报）：编不过、锚点不唯一、名字不唯一、找不到（定位失败，逐行驱动写的 status 名）、实现自崩、超时
READING_FIVE_CATEGORIES = ("compile_failed", "anchor_not_unique", "ambiguous_name", "not_found", "self_crash", "timeout")


def rows_not_run_today(t1_text: str, missing: list[int]) -> list[int]:
    """失败条款 F3：库里取不回的行，与 T1 里 ran=false 而登记读法批 1 的类不是读法 5 那几类的行（驱动报错、装置别的退出码、装置停机、导出缺流）。"""
    import contrast_against_r1 as contrast
    not_run = set(missing)
    for kind, fields in contrast.read_product_entries(t1_text):
        if kind == "row" and fields.get("ran") == "false" and fields.get("category_registration_batch_1") not in READING_FIVE_CATEGORIES:
            not_run.add(int(fields["row"]))
    return sorted(not_run)


def rebuild_check_problems(shard_records: dict[int, list[dict[str, str]]], t2_text: str) -> list[str]:
    """作废 Z4：逐行驱动退 5（编前编后核对不过，片里记 rows_driver / empty_mutation 那一行的 exit），或 T2 有编过的行（status=ok）没带 packages_checked_rebuilt。"""
    import current_tree_row_driver as driver
    problems = [f"{index}:{entry['name']}:exit_{entry.get('exit')}" for index, entries in sorted(shard_records.items()) for entry in entries
                if entry["name"] in ("rows_driver", "empty_mutation") and entry.get("exit") == str(driver.EXIT_MEASUREMENT_PIPE_DEFECT)]
    for line in t2_text.splitlines():
        if not line.startswith("row "):
            continue
        fields = dict(item.split("=", 1) for item in line.split(" ")[1:] if "=" in item)
        if fields.get("status") == driver.STATUS_OK and fields.get(driver.REBUILD_CHECK_FIELD, "-") in ("", "-"):
            problems.append(f"row_{fields.get('row_number')}:{driver.REBUILD_CHECK_FIELD}_missing")
    return problems


def stored_row_check_problems(shard_records: dict[int, list[dict[str, str]]], record: dict, plan: ReusePlan) -> list[str]:
    """第七节 A7：每片的空变异与这一片的每一行哨兵各有一行 name=a7，differing 都是 -；少一行或有不同都点名。"""
    problems = []
    for index, entries in sorted(shard_records.items()):
        numbers = record["shards"][index] if index < len(record["shards"]) else []
        expected = 1 + len(set(plan.sentinels) & set(numbers))
        found = [entry for entry in entries if entry["name"] == "a7"]
        if len(found) != expected:
            problems.append(f"{index}:a7_lines_{len(found)}_expected_{expected}")
        problems.extend(f"{index}:{entry.get('differing')}" for entry in found if entry.get("differing") != "-")
    return problems


@dataclasses.dataclass(frozen=True)
class MergeReadings:
    """合并那一步判作废、停机、失败条款要的读数（clause_checks 是纯函数，单测手造这一份）。"""

    shard_records: dict[int, list[dict[str, str]]]
    cross: str
    cross_differing: tuple[str, ...]
    r1_problems: tuple[str, ...]
    not_run: int
    sentinels: tuple[str, ...]
    control_failing: int
    rows_not_run: tuple[int, ...]
    t1_exit: int
    self_check: tuple[str, ...]
    q2_agrees: bool
    account: tuple[str, ...]
    rebuild_problems: tuple[str, ...]
    stored_row_problems: tuple[str, ...]


def clause_checks(readings: MergeReadings) -> list[tuple[str, bool, str]]:
    """第十、十一节逐条：(条款, 成不成立, 细节)。各片的读数取 shard.out 里第一次出现的那一行。"""
    records = readings.shard_records

    def first(index: int, name: str) -> dict[str, str]:
        return next((entry for entry in records.get(index, []) if entry["name"] == name), {})

    checks: list[tuple[str, bool, str]] = []
    baselines = {index: first(index, "baseline") for index in records}
    for clause, key in (("S1", "s1_failed_checks"), ("S3", "s3_o2_judgement_mismatches"), ("S5", "s9_failed_checks")):
        bad = [f"{index}:{entry.get(key, 'missing')}" for index, entry in baselines.items() if entry.get(key, "missing") not in ("0", "zero")]
        checks.append((clause, not bad, ",".join(bad)))
    # S4：基线退 3 而 S1、S3、S5 都过，就是装置的 S5（O1 不适用来自别的来源）或流没导出；装置不单报 S5 的数，按退出码判
    checks.append(("S4", bool(baselines) and all(entry.get("exit") in ("0",) for entry in baselines.values()),
                   ",".join(f"{index}:{entry.get('exit')}" for index, entry in baselines.items())))
    s2 = first(0, "s2_export_against_the_baseline").get("differing", "missing")
    checks.append(("S2", s2 == "-", s2))
    stops = [f"{index}:{entry.get('clause')}" for index, entries in sorted(records.items()) for entry in entries if entry["name"] == "stop"]
    checks.append(("S6", not stops, ",".join(stops)))
    checks.append(("S7", readings.cross != "differs", ",".join(readings.cross_differing)))
    z1 = first(0, "z1_second_baseline").get("differing", "missing")
    checks.append(("Z1", z1 == "-", z1))
    shard_problems = [f"{index}:{entry.get('moment')}:{entry.get('problems')}" for index, entries in sorted(records.items())
                      for entry in entries if entry["name"] == "shard_check" and entry.get("problems") != "-"]
    moments = [f"{index}:{moment}" for index in sorted(records) for moment in ("start", "end")
               if not any(entry["name"] == "shard_check" and entry.get("moment") == moment for entry in records[index])]
    checks.append(("Z2_Z3", not shard_problems and not moments, ",".join(shard_problems + [f"{item}:not_checked" for item in moments])))
    checks.append(("Z4", not readings.rebuild_problems, ",".join(readings.rebuild_problems)))
    checks.append(("Z5", not readings.r1_problems, ",".join(readings.r1_problems)))
    checks.append(("Z6", readings.not_run == 0, str(readings.not_run)))
    checks.append(("Z7", not readings.sentinels, ",".join(readings.sentinels)))
    checks.append(("F1_F2", readings.control_failing == 0, str(readings.control_failing)))
    checks.append(("F3", not readings.rows_not_run and readings.t1_exit == 0,
                   f"rows={','.join(map(str, readings.rows_not_run)) or '-'} t1_exit={readings.t1_exit}"))
    checks.append(("A5", not readings.self_check, ",".join(readings.self_check)))
    checks.append(("A6", readings.q2_agrees, "-"))
    checks.append(("A7", not readings.stored_row_problems, ",".join(readings.stored_row_problems)))
    checks.append(("reuse_account", not readings.account, ",".join(readings.account)))
    return checks


def finish_merge(path_of: dict[str, str], record: dict, plan: ReusePlan, outputs: list[tuple[int, str, str]], shard_zero: str, baseline_path: str,
                 missing: list[int], sentinels: list[str], account: list[str], t1_exit: int, self_check: list[str], r1_problems: list[str],
                 q1: str, q2: str, q2_agrees: bool) -> int:
    """对照、跨机对照、作废与停机条款逐条、NOT_RUN 合计、复用账，与问题单两行的判定。"""
    segment = record["segment"]
    lines = ResultLines(path_of["controls"])
    lines.result(f"name=config script=full_table_run part=controls segment={segment}")
    control_failing = control_lines(lines, shard_zero, baseline_path, segment)
    cross, differing = cross_host_comparison(outputs, stream_names_of(segment))
    lines.result(f"name=cross_host hosts={field(','.join(host for _, host, _ in outputs))} result={cross} differing={field(','.join(differing) or '-')}")
    lines.finish()
    shard_records = {index: shard_lines(directory) for index, _, directory in outputs}
    with open(path_of["t2"], encoding="utf-8") as handle:
        t2_text = handle.read()
    with open(path_of["t1"], encoding="utf-8") as handle:
        t1_text = handle.read()
    not_run = not_run_count(not_run_paths(outputs, os.path.join(os.path.dirname(baseline_path), "rows")))
    readings = MergeReadings(
        shard_records=shard_records, cross=cross, cross_differing=tuple(differing), r1_problems=tuple(r1_problems), not_run=not_run,
        sentinels=tuple(sentinels), control_failing=control_failing, rows_not_run=tuple(rows_not_run_today(t1_text, missing)), t1_exit=t1_exit,
        self_check=tuple(self_check), q2_agrees=q2_agrees, account=tuple(account), rebuild_problems=tuple(rebuild_check_problems(shard_records, t2_text)),
        stored_row_problems=tuple(stored_row_check_problems(shard_records, record, plan)))
    checks = clause_checks(readings)
    lines = ResultLines(path_of["verdict"])
    lines.result(f"name=config script=full_table_run part=verdict segment={segment} head={record['head']} snapshot_digest={record['tree_inputs']['snapshot_digest']}")
    for clause, holds, detail in checks:
        lines.result(f"name=check clause={clause} holds={str(holds).lower()} detail={field(detail or '-')}")
    lines.result(f"name=reuse_account rows={len(plan.keys)} fresh={len(plan.fresh) - len(plan.sentinels)} reused={len(plan.reused)} "
                 f"sentinels={len(plan.sentinels)} sentinel_differences={len(sentinels)}")
    failed = [clause for clause, holds, _ in checks if not holds]
    lines.result(f"name=verdict q1={q1} q2={q2} failed_checks={field(','.join(failed) or '-')} not_run_lines={not_run}")
    lines.finish()
    return EXIT_INVALID if failed else 0


SELFTEST_CELLS = (("reuse-three-questions", cell_reuse_three_questions),
                  ("sentinel-detects-a-corrupted-library-entry", cell_sentinel_detects_a_corrupted_library_entry),
                  ("reuse-account", cell_reuse_account),
                  ("stored-row-matches-the-full-export", cell_stored_row_matches_the_full_export))


def selftest() -> int:
    """自证：每格各自建自己的假库与临时树（格与格之间不共用状态）；末行 FULL_TABLE_SELFTEST cells=… passed=… failed=…。"""
    failed = []
    for name, cell in SELFTEST_CELLS:
        passed = cell()
        print(f"  {'✓' if passed else '✗'} {name}")  # gate-lint:detail
        if not passed:
            failed.append(name)
    print(f"FULL_TABLE_SELFTEST cells={len(SELFTEST_CELLS)} passed={len(SELFTEST_CELLS) - len(failed)} failed={len(failed)} "
          f"failed_cells={','.join(failed) or '-'}")
    if failed:
        print("  → 怎么办：逐个单看 failed_cells 点名的那几格的判法；设着弄坏开关的，这就是要的红")
    return 1 if failed else 0


def main(argument_list: list[str]) -> int:
    if argument_list == ["--selftest"]:
        return selftest()
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ {BREAK_VARIABLE} 只在自证里认，这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再交；要证红就跑 {BREAK_VARIABLE}=<项> python3 {os.path.relpath(__file__, REPOSITORY_ROOT)} --selftest",
              file=sys.stderr)
        return EXIT_USAGE
    # 线道树与计划树都不是 git 工作树：装置、多机配置判法与 prune-server 客户端要的多机配置显式交过去，之后起的子进程都继承
    os.environ.update(lib_repository_copy.environment_for_commands_in_the_copy(REPOSITORY_ROOT, os.environ))
    parser = argparse.ArgumentParser(description="E201 三oracle合谋今树整表的入口（只经统一任务服务的分片三步起）")
    parser.add_argument("mode", choices=("run",))
    parser.add_argument("--stream-three", action="store_true", help="段名：给了是第二段（空变异加 r1 第一段十五行、三条流，读法 11、修订 1 第 1.2 条），不给是第一段（流一、流二整表）")
    arguments = parser.parse_args(argument_list)
    phase = os.environ.get("TASK_SERVER_SHARD_PHASE", "")
    if phase not in ("plan", "shard", "merge"):
        print("  ✗ 只经统一任务服务的分片三步起（环境里没有 TASK_SERVER_SHARD_PHASE）", file=sys.stderr)
        print("     → 怎么办：task-server/task submit -- python3 " + os.path.relpath(__file__, REPOSITORY_ROOT) + " run", file=sys.stderr)
        return EXIT_USAGE
    store = open_library()
    plan_directory = os.environ["TASK_SERVER_SHARD_PLAN_DIRECTORY"]
    if phase == "plan":
        return plan_step(arguments, plan_directory, store)
    if phase == "shard":
        output_directory = os.environ["TASK_SERVER_SHARD_OUTPUT_DIRECTORY"]
        os.makedirs(output_directory, exist_ok=True)
        index, _, _count = os.environ["TASK_SERVER_SHARD"].partition("/")
        return shard_step(plan_directory, output_directory, int(index), store)
    return merge_step(plan_directory, store)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
