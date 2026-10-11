#!/usr/bin/env python3
"""E171 第 1 次重跑第二段（整表，登记 research/prompts/e171-r1-prereg.md 第五节 5.5 第二段、修订 2 第 2.5 节、修订 6 第 6.6 节「第二段（整表）照原文，
在修订 6 树上跑」）的多机编排：281 行（第 1–117 行与第 118–281 行两张表）加补位，流一、流二，批 1 批 2，不跑执行器、不带探针。
由 run_segment_two_on_the_revision_6_tree_multi_host.sh 起，在仓根跑。

为什么另写一层、不改第一段那一份编排器：rows_orchestrator.py 被 cross_host_o3_check.py 按 sha256 7bfbdb84… 钉住（改一个字节它退 3，复跑行
driver_e171_r1_s1_revision_6_cross_host_check 跟着红）。这一层按路径载入那一份（开跑先核 sha256，不是就退 3），机器清单、资源、分配、预检、单台配置、
发给别的台的快照与工具包、装产物都调它原来的函数；第二段不同的几处（行集、产物名、不跑执行器的合并与判法、补位协调、跨机对照那一行、逐行限时的超时重跑）写在这里。

用法：
  rows_orchestrator_segment_two.py run  --repo R --prefix P --suffix S --dumps D --tree TREE --snapshot-digest H --baseline-timing B --check C
                                        --logs L --work-root W [--research-root RR]
      跑整表。被停、被杀之后同一条命令再起就是续跑：已做完的行（本机的、已取回的别的台的）不重跑，写到一半的重做，资源与分配重新现查现算。
  rows_orchestrator_segment_two.py plan <同上>
      只现查机器与资源、算分配、按每行推的用时推各台挂钟，打出来；一行都不起，不写产物。
  rows_orchestrator_segment_two.py --selftest [--cells 格,…]
      假驱动、假 multi-host-run、假机器清单与资源走一遍各格（），连同第一段编排器自己的 --selftest；自证进程是子进程的收养者
      （PR_SET_CHILD_SUBREAPER），每一格之后收掉它起的、还活着的每个进程，末尾数一遍自己的进程树，临时目录（自证给子进程的 TMPDIR）删掉；
      末尾逐个打开弄坏开关核它红在点名的格。末行「自证 N 项，红 M」。

退出码：0 跑完（合并、判法、跨机对照、超时重跑都过）；1 停机或某一行没跑成（日志与 sequence.log 写停在哪、为什么）；2 用法错；
  3 第一段编排器不是 7bfbdb84… 那一份；78 运行条件不满足（多机配置判不过、某台连不上或没读到资源、别的台的预检没过、此刻一条线道都放不下、
  跨机对照那一行此刻放不下）——不悄悄退回单机，出路是修好那台或改配置，再用同一条命令起；143 收到 TERM / INT / HUP，收完尾退出。

行集：两张坏实现表（bad-implementations-117.tsv、bad-implementations-expansion-164.tsv）的整表编号，必须恰是 1–281（登记第一节）。不跑空变异（第二段登记里没有）。
每行一个单线道驱动（--lanes 1），驱动参数：--rows <行> --streams 1,2 --no-executor --lanes 1 --threads <每槽线程> --memory-cap 6G --skip-finished
  --other-selected-rows <这一行的补位排除表>；不带 --keep-dumps（登记 5.3「第二段的行导出判完即删」，驱动删三个大文件、留判定表与 manifest）；
  限时由驱动照 --baseline-timing 现算（修订 2 第 2.5 节第二段：T 取修订 6 树上 S2 那一趟 --streams 1,2、不带探针的基线的进程挂钟）。
机器、资源、分配：照第一段编排器文件头「机器」「资源与分配」，调它的 hosts_from_configuration、resources_from_json、allocate（每槽 5 线程起、
  本机 slice 账 6G、别的台 10G、实占 10 GiB、可用线程至多 31）。起跑时一台一槽都放不下的（例如本机正忙），每 REGROW_SECONDS 秒重查一次资源，
  那台放得下了就给它加槽（别的台先过预检）；已有槽的台不增不减。
派发：每行推的用时 = T × 权重 + 120 秒；权重 = 那一行在 s4 整表（research/results 下 S4_TIMING_PRODUCTS 那份）里的 device_ns ÷ 各行 device_ns 的中位数，
  读不到的行取 1（推的：s4 是旧树、旧装置，只拿来排先后，不进任何判据）。队列最长先做，用时相同按行号；哪一槽空了就取队头。
  跨机对照那一行 R 取权重最接近 1 的行（平手取行号小的）；R 在本机跑时用新的工作根（冷编译，跨机对照比得了编译那几列之外的一切）。
补位协调（登记第一节读法 5 的补位规则「换同一文件组里下标 j + 1、两张表里都没有、这一次跑里别的补位也没用过的那一行」）：每行一个驱动时，
  「这一次跑里别的补位用过的」不在同一个进程里，这一层替驱动记：
  ① 起一行时，把输出根里已做完各行用过的补位名（row-result.tsv 的 failed_attempt_<序> 里的名字与补位上的 mutated_row_name）写成一张表（驱动的
    --other-selected-rows 读的格式），驱动补位时跳过它们；给了哪张表记进 exclusions-given.tsv；
  ② 整表做完之后核一遍：一个补位名被两行用了，行号小的留着，大的挪进 superseded/、带上别的行用过的名字重做，直到没有重复（一次一行）；
  ③ 产物 <P>-rows-substitution-coordination<S>.out：每个用过补位的行一行、重做过的行、末行 name=verdict substitute_collisions=0。
产物（照第一段「产物与合并」，按不跑执行器改了两处：分片里没有 freeze_check 行，有就拒；判法换成 check_product.py rows_without_executor，
  它照 rows 判，只把不跑执行器的那两处换掉、补位补不上的几类不算红；合成的先写进分片目录 merged/ 判，判过了才装进 research/results）：
  <P>-rows-driver<S>.out 与 -timing.out：这一趟各行的分片 + 更早几趟已做完的行由本机一个 --skip-finished 驱动重报的分片，按写死的规则合成：
  ① 各分片的 name=row_time_limit 行逐字相同；② snapshot_digest 与 files 等于冻结树快照；③ row 行按序号排，不许重复，行集恰是 1–281；
  ④ 没有 freeze_check 行；⑤ table 行：lanes、rows、sum_of_row_ns 相加，table_wall_ns 取最大，threads_per_lane 去重升序，加 shards=；
  ⑥ 每个分片一行 merged_shard，计时按 row_device 分块、块按序号排、行集与产物的相等；⑦ 认不得的行拒。
跨机一致对照（照「主 agent 认定：-resume2 跨机一致对照 S17 停机的处置」第 3 条与第一段编排器的比法）：整表做完、补位没有重复之后，R 在另一台再跑一次
  （同一张补位排除表），比三样：row 行去四个耗时列、device.out 的 E7RESULT 行、导出 z1 五个文件；o3 那一格不跑执行器就没有，报 identical=not_run。
  有一样不同就停机（退 1）；产物 <P>-rows-cross-host-check<S>.out 与 R 那次的 <P>-rows-cross-host-row-<R>-driver<S>.out。只有一台时报 not_run。
超时重跑（主 agent 派发写定；登记修订 2 第 2.5 节原文「第二段照原登记读法 5 与补位规则，不重跑」，所以这一份只另出产物、不改合成的整表那一份）：
  整表产物里 device_exit=253 的行，单线道、限时再乘 3（--rerun-of-timed-out-rows），挑有槽的台里可用线程最多的那台，用那一行原来那张补位排除表；
  产物 <P>-rows-timeout-rerun-driver<S>.out。
续跑：同第一段（取回目录里有 row-result.tsv 的行先收进本机输出根，写到一半的删掉重做；已装的产物不重合）。

弄坏开关 ROWS_ORCHESTRATOR_TWO_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，别处设了退 2）。项名、弄坏哪一处、打开后红在哪一格：
  sweep-skipped               每一格之后与末尾不收进程                          no-leftover-processes
  temporary-kept              末尾不删自证给子进程的临时目录                    temporary-directory-removed
  merge-allows-freeze-lines   合并时分片里有 freeze_check 行也收                merge-refuse-freeze-line
  merge-allows-duplicates     合并时同一序号两行取前一行、不拒                  merge-refuse-duplicate
  exclusions-not-given        起一行时不给补位排除表（空表）                    e2e-substitution-sequential
  collisions-ignored          整表做完不核补位重复                              e2e-substitution-collision
  cross-host-ignores-dumps    跨机对照不比导出那一样                            cross-host-compare
  resume-reruns-finished      续跑时已做完的行也重跑                            e2e-resume
  harvest-skipped             别的台取回的行不收进本机行输出根                  e2e-two-hosts
  regrow-skipped              起跑时放不下的台之后放得下了也不加槽              e2e-regrow
"""
from __future__ import annotations

import argparse
import concurrent.futures
import ctypes
import dataclasses
import functools
import hashlib
import importlib.util
import json
import os
import re
import shlex
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time
import types
import urllib.parse

HERE = os.path.dirname(os.path.abspath(__file__))
SEGMENT_ONE_ORCHESTRATOR_PATH = os.path.join(HERE, "rows_orchestrator.py")
SEGMENT_ONE_ORCHESTRATOR_SHA256 = "7bfbdb84559476033853794499836f752fdf8a3e803e17cd9539ae49ee3b7e0e"
SEGMENT_ONE_SELFTEST_LAST_LINE = "自证 34 项，红 0"
TABLE_FILES = ("bad-implementations-117.tsv", "bad-implementations-expansion-164.tsv")
WHOLE_TABLE_ROWS = tuple(range(1, 282))
ROW_FLAGS = ("--streams", "1,2", "--no-executor")
# s4 整表那一趟各行的计时（旧树、旧装置，流一流二）：只拿来排先后（文件头「派发」），不进任何判据
S4_TIMING_PRODUCTS = ("e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-rows-timing.out",)
ROW_OVERHEAD_SECONDS = 120
REGROW_SECONDS = 600
PEER_LAUNCH_RELATIVE = "research/e171-s2-launch"
EXCLUSIONS_FILE = "exclusions.tsv"
EXCLUSION_ROW_NUMBER_BASE = 900000
EXIT_WRONG_SEGMENT_ONE_ORCHESTRATOR = 3
BREAK_VARIABLE = "ROWS_ORCHESTRATOR_TWO_BREAK"
SELFTEST_VARIABLE = "ROWS_ORCHESTRATOR_TWO_SELFTEST"
SELFTEST_ROWS_VARIABLE = "ROWS_ORCHESTRATOR_TWO_ROWS"
PR_SET_CHILD_SUBREAPER = 36


class WrongSegmentOneOrchestrator(Exception):
    """第一段编排器文件的 sha256 不是 7bfbdb84…：这一层调的那几个函数不再是第一段跑的那一份。"""


def load_segment_one_orchestrator(path: str = SEGMENT_ONE_ORCHESTRATOR_PATH) -> types.ModuleType:
    """按路径载入第一段编排器（不改它、不写 .pyc）；sha256 不对就拒。"""
    with open(path, "rb") as handle:
        actual = hashlib.sha256(handle.read()).hexdigest()
    if actual != SEGMENT_ONE_ORCHESTRATOR_SHA256:
        raise WrongSegmentOneOrchestrator(f"{path} 的 sha256 是 {actual}，不是 {SEGMENT_ONE_ORCHESTRATOR_SHA256}")
    specification = importlib.util.spec_from_file_location("rows_orchestrator_7bfbdb84", path)
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    sys.modules[specification.name] = module  # dataclasses 在类定义时按 __module__ 去 sys.modules 里找这个模块
    specification.loader.exec_module(module)
    return module


try:
    one = load_segment_one_orchestrator()
except WrongSegmentOneOrchestrator as wrong_orchestrator:
    print(f"✗ {wrong_orchestrator}（这一层调的是第一段跑的那一份的函数；第一段编排器改过就先核这一层，再改钉的 sha256）", file=sys.stderr)
    sys.exit(EXIT_WRONG_SEGMENT_ONE_ORCHESTRATOR)


def broken(item: str) -> bool:
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


@dataclasses.dataclass(frozen=True)
class StepTwo:
    """第二段的一步：key 在账与分片记录里用；stem 是产物名里 <PREFIX>- 之后、<SUFFIX> 之前那一段；root_suffix 接在行输出根后面；flags 是每行驱动的参数。"""
    key: str
    stem: str
    root_suffix: str
    flags: tuple[str, ...]


ROWS_STEP = StepTwo("rows", "rows-driver", "", ROW_FLAGS)
RERUN_STEP = StepTwo("rerun", "rows-timeout-rerun-driver", "-timeout-rerun", ("--rerun-of-timed-out-rows",) + ROW_FLAGS)


def cross_host_step(row: int) -> StepTwo:
    return StepTwo("cross", f"rows-cross-host-row-{row}-driver", "-cross-host", ROW_FLAGS)


CROSS_HOST_CHECK_STEM = "rows-cross-host-check"
COORDINATION_STEM = "rows-substitution-coordination"


@dataclasses.dataclass
class ContextTwo(one.Context):
    """第一段的 Context 加第二段的行集与跨机对照那一行；行输出根是 segment2-rows，产物名不接 tag（tag 只进别的台那一趟的标签）。"""
    rows: tuple[int, ...] = WHOLE_TABLE_ROWS
    cross_host_row: int = 0
    row_seconds: dict = dataclasses.field(default_factory=dict)

    @property
    def rows_root(self) -> str:
        return os.path.join(self.repo, self.dumps, "segment2-rows")

    def product(self, stem: str) -> str:
        return os.path.join(self.repo, f"{self.prefix}-{stem}{self.suffix}.out")

    def timing(self, stem: str) -> str:
        return os.path.join(self.repo, f"{self.prefix}-{stem}{self.suffix}-timing.out")


def step_of(context: ContextTwo, key: str) -> StepTwo:
    return {"rows": ROWS_STEP, "rerun": RERUN_STEP, "cross": cross_host_step(context.cross_host_row)}[key]


def whole_table_rows(research_directory: str) -> tuple[int, ...]:
    """两张表的整表编号；不是恰好 1–281 就拒（登记第一节）。"""
    sys.path.insert(0, research_directory)
    import bad_implementation_table  # noqa: E402
    numbers = sorted(row.number for name in TABLE_FILES for row in bad_implementation_table.read_selected_rows(os.path.join(research_directory, name)))
    if tuple(numbers) != WHOLE_TABLE_ROWS:
        raise one.Stopped(f"两张坏实现表的整表编号不是恰好 1–281（{len(numbers)} 行）")
    return tuple(numbers)


# ── 每行推的用时、跨机对照那一行、队列（纯函数）──
def s4_device_seconds(lines: list[str]) -> dict[int, float]:
    """s4 计时产物的 name=row_time 行：行号 → device_ns（秒）；device_ns 是 - 的（编不过）不收。"""
    found: dict[int, float] = {}
    for line in lines:
        if line.startswith("E7TIMING name=row_time "):
            fields = one.key_values(line)
            if fields.get("device_ns", "").isdigit():
                found[int(fields["row"])] = int(fields["device_ns"]) / 1e9
    return found


def row_weights(rows: tuple[int, ...], measured: dict[int, float]) -> dict[int, float]:
    """权重 = 那一行的 s4 device 秒数 ÷ 量到的各行的中位数；没量到的行取 1。"""
    median = statistics.median(measured.values()) if measured else 0.0
    return {row: (measured[row] / median if median > 0 and row in measured else 1.0) for row in rows}


def estimated_row_seconds(baseline_seconds: float, weights: dict[int, float]) -> dict[int, float]:
    return {row: baseline_seconds * weight + ROW_OVERHEAD_SECONDS for row, weight in weights.items()}


def choose_cross_host_row(weights: dict[int, float]) -> int:
    """权重最接近 1 的那一行（推的用时最接近中位），平手取行号小的。"""
    return min(weights, key=lambda row: (abs(weights[row] - 1.0), row))


@dataclasses.dataclass(frozen=True)
class JobTwo:
    step: str
    row: int
    seconds: float


def queue_order_two(jobs: list[JobTwo]) -> list[JobTwo]:
    """最长先做；用时相同按步（整表、补位重做、跨机、超时重跑）与行号。"""
    rank = {"rows": 0, "cross": 1, "rerun": 2}
    return sorted(jobs, key=lambda job: (-job.seconds, rank[job.step], job.row))


def simulate_two(slots: list[tuple[int, int, int]], jobs: list[JobTwo]) -> list[tuple[JobTwo, int, float, float]]:
    """按派发器同一个取法（哪一槽先空就取队头）推每行放在哪台、何时起止。slots 是 (台, 槽, 每槽线程)。交回 (行, 台, 起, 止)。"""
    free = sorted((0.0, host, index) for host, index, _ in slots)
    placed = []
    for job in queue_order_two(jobs):
        if not free:
            break
        start, host, index = free.pop(0)
        placed.append((job, host, start, start + job.seconds))
        free.append((start + job.seconds, host, index))
        free.sort()
    return placed


def prediction_lines(slots: list[tuple[int, int, int]], placed: list[tuple[JobTwo, int, float, float]]) -> list[str]:
    lines = []
    for host in sorted({host for host, _, _ in slots}):
        mine = [item for item in placed if item[1] == host]
        wall = max((item[3] for item in mine), default=0.0)
        lines.append(f"推的挂钟 {one.host_name(host)}：{wall:.0f} 秒（约 {wall / 3600:.1f} 小时），{len(mine)} 行，"
                     f"槽 {sum(1 for slot_host, _, _ in slots if slot_host == host)}")
    return lines


# ── 补位协调（文件头「补位协调」；纯函数）──
def substitute_names_used(fields: dict[str, str]) -> set[str]:
    """一行 row-result.tsv 里这一行试过的补位名：failed_attempt_<序> 的名字与补位上的 mutated_row_name，去掉这一行自己在表里的名字。"""
    names = {value.split(":", 1)[1] for key, value in fields.items() if re.fullmatch(r"failed_attempt_[0-9]+", key) and ":" in value}
    if fields.get("substituted_from", "-") != "-" and fields.get("mutated_row_name", "-") != "-":
        names.add(fields["mutated_row_name"])
    names.discard(fields.get("row_name", ""))
    return names


def exclusion_table_text(names: set[str]) -> str:
    """驱动 --other-selected-rows 读的格式（read_selected_rows：序号、刀、文件、组内下标、变异名、行文摘要前缀，制表符分隔）；序号从 900000 起，不与整表编号撞。"""
    return "".join(f"{EXCLUSION_ROW_NUMBER_BASE + index}\t2\t-\t0\t{name}\t-\n" for index, name in enumerate(sorted(names)))


def collisions(used_by_row: dict[int, set[str]]) -> dict[str, list[int]]:
    """被两行以上用过的补位名 → 用过它的行（升序）。"""
    rows_of: dict[str, list[int]] = {}
    for row in sorted(used_by_row):
        for name in used_by_row[row]:
            rows_of.setdefault(name, []).append(row)
    return {name: rows for name, rows in sorted(rows_of.items()) if len(rows) > 1}


def rows_to_redo(found: dict[str, list[int]]) -> list[int]:
    """每个重复的名字留行号最小的那一行，其余重做。"""
    return sorted({row for rows in found.values() for row in rows[1:]})


def quoted_name(name: str) -> str:
    """变异名里有空格、逗号、竖线：结果行里按 URL 百分号编码写（urllib.parse.unquote 还原），不含空白与逗号。"""
    return urllib.parse.quote(name, safe="")


def coordination_product(used_by_row: dict[int, set[str]], fields_by_row: dict[int, dict[str, str]], redone: dict[int, int],
                         remaining: dict[str, list[int]]) -> str:
    lines = ["E7RESULT name=substitution_columns used=failed_attempt_names_and_substitute exclusions=other_selected_rows_given_at_launch"]
    for row in sorted(used_by_row):
        fields = fields_by_row[row]
        if not used_by_row[row] and row not in redone:
            continue
        lines.append(f"E7RESULT name=substitution row={row} status={fields.get('status')} substituted_from={quoted_name(fields.get('substituted_from', '-'))} "
                     f"mutated_row_name={quoted_name(fields.get('mutated_row_name', '-'))} "
                     f"used={','.join(quoted_name(name) for name in sorted(used_by_row[row])) or '-'} redone={redone.get(row, 0)}")
    lines.append(f"E7RESULT name=verdict substitute_collisions={len(remaining)} rows_with_substitute_names={sum(1 for names in used_by_row.values() if names)} "
                 f"redone_rows={','.join(str(row) for row in sorted(redone)) or 'none'}")
    return "\n".join(lines) + "\n"


# ── 合并（文件头「产物」①–⑦；不跑执行器，分片里没有 freeze_check 行；纯函数）──
@dataclasses.dataclass(frozen=True)
class ParsedShardTwo:
    label: str
    host: int
    limit_line: str
    snapshot: dict[str, str]
    rows: dict[int, str]
    table: dict[str, str]
    sha256: str


def parse_shard_two(shard) -> ParsedShardTwo:
    lines = shard.product.splitlines()
    kinds = []
    for line in lines:
        if line.startswith("name=row_time_limit "):
            kinds.append("limit")
        elif line.startswith("snapshot_digest="):
            kinds.append("snapshot")
        elif line.startswith("row "):
            kinds.append("row")
        elif line.startswith("table "):
            kinds.append("table")
        elif line.startswith("freeze_check ") and broken("merge-allows-freeze-lines"):
            kinds.append("freeze")
        elif line.startswith("freeze_check "):
            raise one.MergeRefused(f"④ 分片 {shard.label} 有 freeze_check 行：不跑执行器的驱动不打它，这一份不是第二段的跑法")
        else:
            raise one.MergeRefused(f"⑦ 分片 {shard.label} 有认不得的行：{line[:120]!r}")
    kinds = [kind for kind in kinds if kind != "freeze"]
    lines = [line for line in lines if not line.startswith("freeze_check ")]
    row_count = kinds.count("row")
    if kinds != ["limit", "snapshot"] + ["row"] * row_count + ["table"] or row_count == 0:
        raise one.MergeRefused(f"⑦ 分片 {shard.label} 的行不是驱动打的次序（row_time_limit、snapshot_digest、row…、table）")
    rows: dict[int, str] = {}
    for line in lines[2:2 + row_count]:
        number = int(one.key_values(line)["row_number"])
        if number in rows and not broken("merge-allows-duplicates"):
            raise one.MergeRefused(f"③ 分片 {shard.label} 里第 {number} 行出现两次")
        rows.setdefault(number, line)
    snapshot = one.key_values(lines[1])
    if int(snapshot.get("rows", -1)) != row_count:
        raise one.MergeRefused(f"② 分片 {shard.label} 的 snapshot_digest 行 rows={snapshot.get('rows')}，row 行却有 {row_count} 条")
    return ParsedShardTwo(shard.label, shard.host, lines[0], snapshot, rows, one.key_values(lines[-1]),
                          hashlib.sha256(shard.product.encode("utf-8")).hexdigest())


def merge_products_two(shards: list, expected_rows: tuple[int, ...], expected_snapshot: tuple[str, str]) -> tuple[str, str]:
    """交回 (合成的产物, 合成的计时)。shards 是第一段编排器的 Shard；expected_snapshot 是 (摘要, 文件数)。"""
    if not shards:
        raise one.MergeRefused("没有分片")
    parsed = [parse_shard_two(shard) for shard in shards]
    limits = sorted({shard.limit_line for shard in parsed})
    if len(limits) != 1:
        raise one.MergeRefused("① 各分片的 row_time_limit 行不同：" + " | ".join(limits))
    for shard in parsed:
        if (shard.snapshot.get("snapshot_digest"), shard.snapshot.get("files")) != expected_snapshot:
            raise one.MergeRefused(f"② 分片 {shard.label} 的快照 {shard.snapshot.get('snapshot_digest')}/{shard.snapshot.get('files')} 不是冻结树那一份 "
                                   f"{expected_snapshot[0]}/{expected_snapshot[1]}")
    rows: dict[int, str] = {}
    for shard in parsed:
        for number, line in shard.rows.items():
            if number in rows and not broken("merge-allows-duplicates"):
                raise one.MergeRefused(f"③ 第 {number} 行在两个分片里都有")
            rows.setdefault(number, line)
    if sorted(rows) != sorted(expected_rows):
        missing, extra = sorted(set(expected_rows) - set(rows)), sorted(set(rows) - set(expected_rows))
        raise one.MergeRefused(f"③ 行集不等：缺 {missing or '无'}，多 {extra or '无'}")
    threads = sorted({int(shard.table.get("threads_per_lane", "0")) for shard in parsed})
    lines = [limits[0], f"snapshot_digest={expected_snapshot[0]} files={expected_snapshot[1]} "
                        f"snapshot_ns={max(int(shard.snapshot.get('snapshot_ns', '0')) for shard in parsed)} rows={len(rows)}"]
    lines += [rows[number] for number in sorted(rows)]
    lines.append(f"table lanes={sum(int(shard.table.get('lanes', '0')) for shard in parsed)} threads_per_lane={','.join(str(value) for value in threads)} "
                 f"rows={sum(int(shard.table.get('rows', '0')) for shard in parsed)} "
                 f"table_wall_ns={max(int(shard.table.get('table_wall_ns', '0')) for shard in parsed)} "
                 f"sum_of_row_ns={sum(int(shard.table.get('sum_of_row_ns', '0')) for shard in parsed)} shards={len(parsed)}")
    for shard in sorted(parsed, key=lambda item: (min(item.rows), item.label)):
        lines.append(f"merged_shard host={shard.host} rows={','.join(str(number) for number in sorted(shard.rows))} sha256={shard.sha256}")
    blocks: dict[int, list[str]] = {}
    for shard, parsed_shard in zip(shards, parsed):
        mine = one.timing_blocks(shard)
        if sorted(mine) != sorted(parsed_shard.rows):
            raise one.MergeRefused(f"⑥ 分片 {shard.label} 的计时行集 {sorted(mine)} 与产物的 {sorted(parsed_shard.rows)} 不等")
        for number, block in mine.items():
            blocks.setdefault(number, block)
    timing_lines = [line for number in sorted(blocks) for line in blocks[number]]
    return "\n".join(lines) + "\n", "\n".join(timing_lines) + ("\n" if timing_lines else "")


# ── 跨机一致对照（文件头「跨机一致对照」；前三样调第一段编排器的比法，o3 不跑执行器就没有）──
def cross_host_parts_two(line_a: str, directory_a: str, line_b: str, directory_b: str) -> list[tuple[str, str, str]]:
    """四样各一行：(样, true|false|not_run, 细节)。"""
    parts = [("row_line", str(one.row_line_without_timing(line_a) == one.row_line_without_timing(line_b)).lower(), "dropped=" + ",".join(one.TIMING_COLUMNS))]
    device_a = [line for line in one.device_result_lines(directory_a) if line.startswith("E7RESULT ")]
    device_b = [line for line in one.device_result_lines(directory_b) if line.startswith("E7RESULT ")]
    parts.append(("device_results", str(bool(device_a) and device_a == device_b).lower(), f"lines={len(device_a)}/{len(device_b)}"))
    compared, differing = one.dump_differences(directory_a, directory_b)
    if broken("cross-host-ignores-dumps"):
        compared, differing = max(compared, 1), []
    parts.append(("dump_files", str(compared > 0 and not differing).lower(), f"compared={compared} differing={','.join(differing) or 'none'}"))
    present = [os.path.exists(os.path.join(directory, "o3.out")) for directory in (directory_a, directory_b)]
    parts.append(("o3", "not_run" if not any(present) else "false", "reason=no_executor" if not any(present) else f"o3_out_present={present}"))
    return parts


def cross_host_product_two(parts: list[tuple[str, str, str]] | None, row: int, host_a: int | None, host_b: int | None, reason: str) -> tuple[str, bool]:
    """交回 (产物原文, 是否停机)。parts 是 None 时这一格本次未跑。not_run 的那一样不算不同。"""
    lines = [f"E7RESULT name=cross_host_columns row={row} row_line_dropped=" + ",".join(one.TIMING_COLUMNS)
             + " device=E7RESULT_lines dump=" + ",".join(one.Z1_FILES) + " o3=not_run_without_executor"]
    if parts is None:
        lines.append(f"E7RESULT name=verdict cross_host_identical=not_run row={row} reason={reason}")
        return "\n".join(lines) + "\n", False
    for part, identical, detail in parts:
        lines.append(f"E7RESULT name=cross_host_part part={part} identical={identical} {detail}")
    differing = [part for part, identical, _ in parts if identical == "false"]
    lines.append(f"E7RESULT name=verdict cross_host_identical={'false' if differing else 'true'} row={row} host_of_rows_step={host_a} "
                 f"host_of_cross_host_run={host_b} parts={len(parts)} compared_parts={sum(1 for _, identical, _ in parts if identical != 'not_run')} "
                 f"differing_parts={','.join(differing) or 'none'}")
    return "\n".join(lines) + "\n", bool(differing)


# ── 跑的时候：账、补位排除表、起驱动、取回 ──
def read_row_result(directory: str) -> dict[str, str]:
    with open(os.path.join(directory, "row-result.tsv"), encoding="utf-8") as handle:
        return dict(line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line)


def done_row_results(context: ContextTwo, step: StepTwo) -> dict[int, dict[str, str]]:
    root = context.step_root(step)
    found = {}
    for entry in sorted(os.listdir(root)) if os.path.isdir(root) else []:
        if re.fullmatch(r"row-[0-9]+", entry) and os.path.exists(os.path.join(root, entry, "row-result.tsv")):
            found[int(entry.split("-", 1)[1])] = read_row_result(os.path.join(root, entry))
    return found


def exclusions_log_path(context: ContextTwo) -> str:
    return os.path.join(context.shards, "exclusions-given.tsv")


def exclusions_given(context: ContextTwo) -> dict[tuple[str, int], tuple[str, ...]]:
    """exclusions-given.tsv：步、行、分片标签、名字（JSON 列表：变异名里有逗号与空格）；同一 (步, 行) 后写的算。"""
    given: dict[tuple[str, int], tuple[str, ...]] = {}
    if os.path.exists(exclusions_log_path(context)):
        for line in open(exclusions_log_path(context), encoding="utf-8"):
            step, row, _, names = line.rstrip("\n").split("\t")
            given[(step, int(row))] = tuple(json.loads(names))
    return given


def exclusions_for(context: ContextTwo, step: str, row: int) -> set[str]:
    """整表的行：输出根里别的已做完各行用过的补位名；跨机对照与超时重跑：那一行在整表里最后一次起时给的那一张（同一张表，补位才选得一样）。"""
    if step == "rows":
        if broken("exclusions-not-given"):
            return set()
        return {name for number, fields in done_row_results(context, ROWS_STEP).items() if number != row for name in substitute_names_used(fields)}
    given = exclusions_given(context)
    if ("rows", row) not in given:
        raise one.Stopped(f"第 {row} 行在整表里起时给的补位排除表没记下（exclusions-given.tsv），{step} 那一次选不出同一个补位")
    return set(given[("rows", row)])


def driver_arguments(step: StepTwo, rows: tuple[int, ...], threads: int, exclusions_path: str) -> list[str]:
    return (["--rows", ",".join(str(row) for row in sorted(rows))] + list(step.flags)
            + ["--lanes", "1", "--threads", str(threads), "--memory-cap", one.DEVICE_MEMORY_CAP, "--skip-finished", "--other-selected-rows", exclusions_path])


def prepare_launch(context: ContextTwo, label: str, step: StepTwo, rows: tuple[int, ...], exclusion_names: set[str],
                   log_exclusions: bool = True) -> tuple[str, str]:
    """建分片目录与要送的那一份（send/exclusions.tsv），真跑一行的记进 exclusions-given.tsv（重报不记）。交回 (分片目录, 排除表路径)。"""
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(os.path.join(directory, "send"))
    path = os.path.join(directory, "send", EXCLUSIONS_FILE)
    with open(path, "x", encoding="utf-8") as handle:
        handle.write(exclusion_table_text(exclusion_names))
    if log_exclusions:
        assert len(rows) == 1, "真跑的分片一次一行"
        os.makedirs(context.shards, exist_ok=True)
        with open(exclusions_log_path(context), "a", encoding="utf-8") as handle:
            handle.write(f"{step.key}\t{rows[0]}\t{label}\t{json.dumps(sorted(exclusion_names), ensure_ascii=False)}\n")
    return directory, path


def launch_local_two(context: ContextTwo, expected_hashes: dict[str, str], label: str, step: StepTwo, rows: tuple[int, ...], threads: int,
                     work_root: str, kind: str, exclusion_names: set[str]) -> tuple[subprocess.Popen, dict]:
    if one.driver_hashes(context) != expected_hashes:
        raise one.Stopped("驱动或它引的模块、行表在这一趟里改过了（每行的命令一字不改）：停")
    directory, exclusions_path = prepare_launch(context, label, step, rows, exclusion_names, log_exclusions=kind != "carry")
    product, timing = os.path.join(directory, "product.out"), os.path.join(directory, "timing.out")
    command = ["nice", "-n", "19", "python3", "-B", context.driver_path, "--repo-root", context.repo, "--work-root", work_root,
               "--output-root", context.step_root(step), "--snapshot-from", os.path.join(context.repo, context.tree),
               "--baseline-timing", os.path.join(context.repo, context.baseline_timing), "--timing-output", timing] + driver_arguments(step, rows, threads, exclusions_path)
    record = {"label": label, "step": step.key, "rows": list(rows), "host": 1, "kind": kind, "product": product, "timing": timing,
              "attempt": os.path.basename(context.attempt_dir), "exclusions": sorted(exclusion_names), "command": command[5:]}
    one.write_record(directory, record)
    with open(product, "x", encoding="utf-8") as out, open(os.path.join(directory, "driver.stderr"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=context.repo, stdout=out, stderr=err, env=one.driver_environment(), start_new_session=True)
    return process, record


def peer_label_two(context: ContextTwo, label: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]", "-", f"e171s2-{os.path.basename(context.attempt_dir)}-{label}")


def launch_peer_two(context: ContextTwo, kit: str, label: str, step: StepTwo, row: int, threads: int, host: int, configuration_file: str | None,
                    exclusion_names: set[str]) -> tuple[subprocess.Popen, dict]:
    directory, _ = prepare_launch(context, label, step, (row,), exclusion_names)
    fetch = os.path.join(directory, "fetch")
    inner = (["run-driver", "--expected", f"{one.PEER_KIT_RELATIVE}/expected.json", "--"]
             + driver_arguments(step, (row,), threads, f"{PEER_LAUNCH_RELATIVE}/{EXCLUSIONS_FILE}"))
    command = (["bash", context.multi_host_run, "--peer", "--memory-cap", one.PEER_OUTER_MEMORY_CAP, "--label", peer_label_two(context, label),
                "--send", kit, one.PEER_KIT_RELATIVE, "--send", os.path.join(context.shards, "frozen-snapshot"), one.PEER_SNAPSHOT_RELATIVE,
                "--send", os.path.join(directory, "send"), PEER_LAUNCH_RELATIVE, "--fetch", one.PEER_OUT_RELATIVE, "--", "env", "PYTHONDONTWRITEBYTECODE=1",
                f"RUN_WITH_MEMORY_CAP_PARALLEL_REASON={one.PARALLEL_REASON}", "nice", "-n", "19", "python3", "-B", f"{one.PEER_KIT_RELATIVE}/peer_side.py"] + inner)
    record = {"label": label, "step": step.key, "rows": [row], "host": host, "kind": "peer", "fetch_dir": fetch,
              "product": os.path.join(fetch, one.PEER_OUT_RELATIVE, "product.out"), "timing": os.path.join(fetch, one.PEER_OUT_RELATIVE, "timing.out"),
              "attempt": os.path.basename(context.attempt_dir), "exclusions": sorted(exclusion_names), "command": inner}
    one.write_record(directory, record)
    with open(os.path.join(directory, "multi-host-run.out"), "x", encoding="utf-8") as out, \
            open(os.path.join(directory, "multi-host-run.err"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=context.repo, stdout=out, stderr=err, env=one.multi_host_environment(fetch, configuration_file),
                                   start_new_session=True)
    return process, record


def superseded_entries(context: ContextTwo) -> set[tuple[str, int, str]]:
    """superseded.tsv（步、行、写那个行目录的分片标签、挪到哪）：补位重做挪走的那几次。"""
    path = os.path.join(context.shards, "superseded.tsv")
    entries = set()
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        step, row, label, _ = line.rstrip("\n").split("\t")
        entries.add((step, int(row), label))
    return entries


def harvest_two(context: ContextTwo, record: dict) -> list[int]:
    """别的台分片取回目录里有 row-result.tsv 的行收进本机那一步的输出根：先记账、硬链接拷进 <行>.harvest-partial、再改名；本机写到一半的同名行目录删掉。"""
    if broken("harvest-skipped"):
        return []
    source_root = os.path.join(record["fetch_dir"], one.PEER_OUT_RELATIVE, "rows")
    step = step_of(context, record["step"])
    superseded = superseded_entries(context)
    harvested = []
    for entry in sorted(os.listdir(source_root)) if os.path.isdir(source_root) else []:
        source = os.path.join(source_root, entry)
        target = os.path.join(context.step_root(step), entry)
        if not re.fullmatch(r"row-[0-9]+", entry) or not os.path.exists(os.path.join(source, "row-result.tsv")) \
                or os.path.exists(os.path.join(target, "row-result.tsv")) or int(entry.split("-", 1)[1]) not in record["rows"] \
                or (step.key, int(entry.split("-", 1)[1]), record["label"]) in superseded:
            continue  # 补位重做顶掉的那一次不再收回来
        one.append_ledger(context, step.key, int(entry.split("-", 1)[1]), record["host"], "fetched")
        os.makedirs(context.step_root(step), exist_ok=True)
        partial = target + ".harvest-partial"
        shutil.rmtree(partial, ignore_errors=True)
        shutil.copytree(source, partial, copy_function=one.link_or_copy)
        if os.path.isdir(target):
            shutil.rmtree(target)
        os.rename(partial, target)
        harvested.append(int(entry.split("-", 1)[1]))
    return harvested


# ── 派发、收尾、重报、合并 ──
@dataclasses.dataclass
class RunningTwo:
    job: JobTwo
    slot: tuple[int, int, int]
    process: subprocess.Popen
    record: dict


def finish_shard_two(context: ContextTwo, record: dict, code: int) -> bool:
    """一个分片结束：记退出码；别的台的把取回的行收进本机；本机的把做完的行记账。交回跑成没有（退 0、点名的行都有 row-result.tsv、产物在）。"""
    with open(os.path.join(context.attempt_dir, record["label"], "exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")
    step = step_of(context, record["step"])
    if record["kind"] == "peer":
        harvest_two(context, record)
    elif record["kind"] == "local":
        for row in record["rows"]:
            if one.is_done(context, step, row):
                one.append_ledger(context, step.key, row, 1, "local")
    done = all(one.is_done(context, step, row) for row in record["rows"])
    shard_ran_through = code == 0 and done and os.path.exists(record["product"])
    note = "（250–254、245 是内存包装自己的结局，那一次不算）" if code == 245 or 250 <= code <= 254 else ""
    one.say(context, f"  {step.key} 第 {one.rows_text(record['rows'])} 行 {one.host_name(record['host'])} {'跑完' if shard_ran_through else '没跑成'}："
                     f"退 {code}{note}，行目录 {'都有' if done else '不全有'} row-result.tsv")
    return shard_ran_through


def stop_running_two(context: ContextTwo, running: list[RunningTwo]) -> None:
    """收到停止信号：给每个在跑的分片（各自一个进程组）发一次 TERM；multi-host-run.sh 收到后先停那边、取回、再删树。等它们都退出，再把取回的行收进来。"""
    for item in running:
        try:
            os.killpg(item.process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    for item in running:
        finish_shard_two(context, item.record, item.process.wait())


def work_root_for(context: ContextTwo, job: JobTwo, slot: tuple[int, int, int], label: str) -> str:
    """本机每槽一个固定工作根（同一槽上后一行增量编译，驱动自己核）；跨机对照那一行（整表那一次与跨机那一次）、超时重跑用新的工作根（冷编译）。"""
    fresh = job.step in ("cross", "rerun") or (job.step == "rows" and job.row == context.cross_host_row)
    return os.path.join(context.work_root, f"{os.path.basename(context.attempt_dir)}-{label}" if fresh else f"host1-slot{slot[1]}")


def unique_label(context: ContextTwo, label: str) -> str:
    """同一趟里同一行在同一台再起（补位重做）：分片目录接 -again<序>，不与前一次撞。"""
    candidate, again = label, 1
    while os.path.exists(os.path.join(context.attempt_dir, candidate)):
        again += 1
        candidate = f"{label}-again{again}"
    return candidate


def regrow_interval_seconds() -> float:
    """REGROW_SECONDS；只在 --selftest 起的子进程里认 ROWS_ORCHESTRATOR_TWO_REGROW_SECONDS（假世界几秒就跑完）。"""
    override = os.environ.get("ROWS_ORCHESTRATOR_TWO_REGROW_SECONDS", "")
    return float(override) if override and os.environ.get(SELFTEST_VARIABLE) == "1" else float(REGROW_SECONDS)


def run_queue_two(context: ContextTwo, slots: list[tuple[int, int, int]], jobs: list[JobTwo], expected_hashes: dict[str, str], setup,
                  regrow=None, avoid_host: int | None = None) -> tuple[list[dict], list[JobTwo], list[str]]:
    """交回 (跑成的分片记录, 没派出去的, 没跑成的说明)。哪一槽空了就取队头；某一行没跑成就不再派新行，在跑的跑完。
    regrow 给了时每 REGROW_SECONDS 秒调它一次，交回的新槽并进来（文件头「机器、资源、分配」）。avoid_host 给了时那台的槽不派（跨机对照）。"""
    free = [slot for slot in slots if slot[0] != avoid_host]
    queue = queue_order_two(jobs)
    running: list[RunningTwo] = []
    succeeded: list[dict] = []
    failures: list[str] = []
    next_regrow = time.monotonic() + regrow_interval_seconds()
    while not one.STOP.is_set():
        if regrow is not None and queue and not failures and time.monotonic() >= next_regrow:
            next_regrow = time.monotonic() + regrow_interval_seconds()
            free.extend(slot for slot in regrow() if slot[0] != avoid_host)
        for slot in sorted(free) if not failures else []:
            if not queue:
                break
            job = queue[0]
            step = step_of(context, job.step)
            label = unique_label(context, f"{job.step}-row-{job.row}-h{slot[0]}")
            try:
                one.snapshot_check(context)  # 作废 Z5：每个驱动起之前核一次
                exclusion_names = exclusions_for(context, job.step, job.row)
                if slot[0] == 1:
                    process, record = launch_local_two(context, expected_hashes, label, step, (job.row,), slot[2], work_root_for(context, job, slot, label),
                                                       "local", exclusion_names)
                else:
                    process, record = launch_peer_two(context, setup.kit, label, step, job.row, slot[2], slot[0], setup.configurations.get(slot[0]),
                                                      exclusion_names)
            except one.Stopped as error:
                failures.append(str(error))
                break
            queue.pop(0)
            free.remove(slot)
            running.append(RunningTwo(job, slot, process, record))
            one.say(context, f"  ▶ {job.step} 第 {job.row} 行 → {one.host_name(slot[0])} 第 {slot[1]} 槽（{slot[2]} 线程，补位排除 {len(exclusion_names)} 个名字）")
        for item in list(running):
            code = item.process.poll()
            if code is None:
                continue
            running.remove(item)
            free.append(item.slot)
            if finish_shard_two(context, item.record, code):
                succeeded.append(item.record)
            else:
                failures.append(f"{item.job.step} 第 {item.job.row} 行（{one.host_name(item.slot[0])}，退 {code}，见 {item.record['label']}/）")
            try:
                one.snapshot_check(context)  # 作废 Z5：每个驱动收尾再核一次
            except one.Stopped as error:
                failures.append(str(error))
        if not running and (failures or not queue or not free):
            break
        time.sleep(context.poll_seconds)
    if one.STOP.is_set():
        stop_running_two(context, running)
        raise one.Interrupted()
    return succeeded, queue, failures


def run_carry_two(context: ContextTwo, step: StepTwo, rows: tuple[int, ...], expected_hashes: dict[str, str]) -> dict:
    """更早几趟（或这一趟里被重做顶掉的分片）已做完的行：本机一个 --skip-finished 驱动从行目录重报（不编、不跑装置）。"""
    work_root = os.path.join(context.attempt_dir, f"carry-work-{step.key}")
    process, record = launch_local_two(context, expected_hashes, f"{step.key}-carry", step, rows, one.CARRY_THREADS, work_root, "carry", set())
    code = process.wait()
    if one.STOP.is_set():
        raise one.Interrupted()
    if not finish_shard_two(context, record, code):
        raise one.Stopped(f"{step.key}：本机重报已做完的第 {one.rows_text(rows)} 行那个驱动没跑成（退 {code}）")
    return record


def merge_step_two(context: ContextTwo, step: StepTwo, expected_rows: tuple[int, ...], records: list[dict], snapshot: tuple[str, str],
                   expected_hashes: dict[str, str]) -> str:
    """这一趟跑成、且行目录还是它写的那一份（没被重做顶掉）的分片，加一个本机重报其余各行的分片，按文件头的规则合成、装进 research/results。"""
    latest = latest_labels(context)
    mine = [record for record in records if record["step"] == step.key and all(latest.get((step.key, row)) == record["label"] for row in record["rows"])]
    covered = {row for record in mine for row in record["rows"]}
    carry_rows = tuple(sorted(row for row in expected_rows if row not in covered))
    if carry_rows:
        mine.append(run_carry_two(context, step, carry_rows, expected_hashes))
    shards = [one.Shard(record["label"], record["host"], open(record["product"], encoding="utf-8").read(),
                        open(record["timing"], encoding="utf-8").read()) for record in mine]
    try:
        product_text, timing_text = merge_products_two(shards, expected_rows, snapshot)
    except one.MergeRefused as error:
        raise one.Stopped(f"{step.key} 合并被拒：{error}")
    staging = os.path.join(context.shards, "merged", f"{os.path.basename(context.product(step.stem))}.{os.path.basename(context.attempt_dir)}.before-install")
    os.makedirs(os.path.dirname(staging), exist_ok=True)
    with open(staging, "w", encoding="utf-8") as handle:
        handle.write(product_text)
    check_rows_two(context, staging, "产物没装：判过了才装进 research/results")  # 判完才交出产物（command-safety.md「带闸的脚本，判完了才交出产物」）
    one.install(context, context.timing(step.stem), timing_text)
    one.install(context, context.product(step.stem), product_text)
    one.say(context, f"  {step.stem} 合成：{len(shards)} 个分片，产物 {os.path.relpath(context.product(step.stem), context.repo)}"
                     f"（{product_text.count(chr(10))} 行），计时 {os.path.relpath(context.timing(step.stem), context.repo)}")
    return context.product(step.stem)


def check_rows_two(context: ContextTwo, product: str, note: str = "") -> None:
    completed = subprocess.run(["python3", "-B", context.check, "rows_without_executor", product], cwd=context.repo, capture_output=True, text=True)
    one.say(context, f"  check_product.py rows_without_executor {os.path.relpath(product, context.repo)}：{completed.stdout.strip()}")
    if completed.returncode != 0:
        raise one.Stopped(f"判法不过：check_product.py rows_without_executor {os.path.relpath(product, context.repo)}{('（' + note + '）') if note else ''}")


def latest_labels(context: ContextTwo) -> dict[tuple[str, int], str]:
    """每个 (步, 行) 最后一次真跑的分片标签（exclusions-given.tsv）：行目录是它写的；被补位重做顶掉的那一次不再算。"""
    labels: dict[tuple[str, int], str] = {}
    if os.path.exists(exclusions_log_path(context)):
        for line in open(exclusions_log_path(context), encoding="utf-8"):
            step, row, label, _ = line.rstrip("\n").split("\t")
            labels[(step, int(row))] = label
    return labels


def slots_of(allocations: list) -> list[tuple[int, int, int]]:
    return [(allocation.number, index, allocation.threads_per_slot) for allocation in allocations for index in range(allocation.slots)]


# ── 主流程 ──
def make_regrow(context: ContextTwo, hosts: list, setup, slots: list[tuple[int, int, int]]):
    """起跑时一槽都没有的台：重查资源，放得下了就给它加槽（别的台先过预检）。重查或预检不过只记一行、这一回不加，不停这一趟。"""
    waiting = {host.number for host in hosts} - {slot[0] for slot in slots}

    def regrow() -> list[tuple[int, int, int]]:
        if not waiting or broken("regrow-skipped"):
            return []
        try:
            resources = one.query_resources(context, hosts)
        except one.RunCondition as error:
            one.say(context, f"  重查资源没成（这一回不加槽）：{error}")
            return []
        added = []
        for allocation in one.allocate([host for host in hosts if host.number in waiting], resources):
            if allocation.slots == 0:
                continue
            try:
                setup.ready(allocation.number, allocation.slots, allocation.threads_per_slot)
            except one.RunCondition as error:
                one.say(context, f"  {one.host_name(allocation.number)}放得下了，预检没过（这一回不加槽）：{error}")
                continue
            waiting.discard(allocation.number)
            added += slots_of([allocation])
            one.say(context, f"  加槽：{one.host_name(allocation.number)} {allocation.slots} 槽，每槽线程 {allocation.threads_per_slot}（{allocation.terms}）")
        return added
    return regrow


def resolve_collisions(context: ContextTwo, slots: list[tuple[int, int, int]], setup, expected_hashes: dict[str, str]) -> tuple[dict, dict, dict, list[dict]]:
    """文件头「补位协调」②：一个补位名被两行用了，行号小的留着，大的挪进 superseded/、带上别的行用过的名字重做，一次一行，直到没有重复。
    交回 (各行用过的补位名, 各行 row-result, 还剩的重复, 重做跑成的分片记录)。"""
    records: list[dict] = []
    for _ in range(len(context.rows) + 1):
        results = done_row_results(context, ROWS_STEP)
        used = {row: substitute_names_used(fields) for row, fields in results.items()}
        found = {} if broken("collisions-ignored") else collisions(used)
        if not found:
            return used, results, found, records
        redo = rows_to_redo(found)[0]
        moved = os.path.join(context.shards, "superseded", f"{os.path.basename(context.attempt_dir)}-row-{redo}")
        os.makedirs(os.path.dirname(moved), exist_ok=True)
        os.rename(os.path.join(context.step_root(ROWS_STEP), f"row-{redo}"), moved)
        with open(os.path.join(context.shards, "superseded.tsv"), "a", encoding="utf-8") as handle:
            handle.write(f"rows\t{redo}\t{latest_labels(context).get(('rows', redo), '-')}\t{os.path.relpath(moved, context.repo)}\n")
        one.say(context, f"  补位重复：{'；'.join(f'{name} 被第 {one.rows_text(rows)} 行用了' for name, rows in found.items())}；第 {redo} 行挪进 superseded/ 重做")
        done, deferred, failures = run_queue_two(context, slots, [JobTwo("rows", redo, 0.0)], expected_hashes, setup)
        if failures or deferred:
            raise one.Stopped("补位重做没跑成：" + "；".join(failures or ["放不下"]))
        records += done
    raise one.Stopped("补位重做到行数那么多次还有重复（不该发生：每一回行号最小的那一行不动）")


def redone_counts(context: ContextTwo) -> dict[int, int]:
    path = os.path.join(context.shards, "superseded.tsv")
    counts: dict[int, int] = {}
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        row = int(line.split("\t")[1])
        counts[row] = counts.get(row, 0) + 1
    return counts


def phase_rows(context: ContextTwo, hosts: list, slots: list[tuple[int, int, int]], setup, expected_hashes: dict[str, str]) -> None:
    """整表：派发、补位协调、合成、判法。"""
    if broken("resume-reruns-finished"):
        for row in context.rows:
            shutil.rmtree(os.path.join(context.step_root(ROWS_STEP), f"row-{row}"), ignore_errors=True)
    records: list[dict] = []
    if not os.path.exists(context.product(ROWS_STEP.stem)):
        jobs = [JobTwo("rows", row, context.row_seconds[row]) for row in context.rows if not one.is_done(context, ROWS_STEP, row)]
        if jobs:
            if not slots:
                raise one.RunCondition("此刻各台都放不下一条线道（每槽要 5 线程、slice 里 6G（别的台 10G）、实占 10 GiB）")
            one.say(context, f"  这一趟要跑 {len(jobs)} 行（已做完的不重跑），跨机对照那一行是第 {context.cross_host_row} 行")
            for line in prediction_lines(slots, simulate_two(slots, jobs)):
                one.say(context, "  " + line)
            for host in sorted({slot[0] for slot in slots}):
                setup.ready(host, sum(1 for slot in slots if slot[0] == host), max(slot[2] for slot in slots if slot[0] == host))
            records, deferred, failures = run_queue_two(context, slots, jobs, expected_hashes, setup, regrow=make_regrow(context, hosts, setup, slots))
            if failures or deferred:
                raise one.Stopped("这几行没跑成：" + "；".join(failures or [f"{len(deferred)} 行没派出去"]) + "。做完的行都留着，同一条命令再起只补没做完的")
        missing = [row for row in context.rows if not one.is_done(context, ROWS_STEP, row)]
        if missing:
            raise one.Stopped(f"整表还有行没做完：{one.rows_text(missing)}")
        used, results, remaining, redone_records = resolve_collisions(context, slots, setup, expected_hashes)
        one.install(context, context.product(COORDINATION_STEM), coordination_product(used, results, redone_counts(context), remaining))
        merge_step_two(context, ROWS_STEP, context.rows, records + redone_records, setup.snapshot, expected_hashes)
    check_rows_two(context, context.product(ROWS_STEP.stem))


def current_allocations(context: ContextTwo, hosts: list) -> list:
    resources = one.query_resources(context, hosts)
    allocations = one.allocate(hosts, resources)
    for line in one.allocation_text(allocations, resources):
        one.say(context, "  " + line)
    return allocations


def phase_cross_host(context: ContextTwo, hosts: list, setup, expected_hashes: dict[str, str]) -> None:
    """文件头「跨机一致对照」：R 在整表那一次之外的另一台再跑一次，比三样，o3 报 not_run。"""
    row, path = context.cross_host_row, context.product(CROSS_HOST_CHECK_STEM)
    if len(hosts) == 1:
        text, _ = cross_host_product_two(None, row, None, None, "只有一台（双机开关没开或清单里只有本机）")
        one.install(context, path, text)
        one.say(context, f"  跨机一致对照：本次未跑（只有一台），产物 {os.path.relpath(path, context.repo)}")
        return
    step = cross_host_step(row)
    host_a = one.host_of(context, one.read_ledger(context), "rows", row)
    if not os.path.exists(context.product(step.stem)):
        records: list[dict] = []
        if not one.is_done(context, step, row):
            slots = [slot for slot in slots_of(current_allocations(context, hosts)) if slot[0] != host_a]
            if not slots:
                raise one.RunCondition(f"跨机对照（第 {row} 行在{one.host_name(host_a)}之外的另一台再跑一次）这一趟放不下：那一台之外此刻没有槽。"
                                       "资源空出来后同一条命令再起只补这一行")
            for host in sorted({slot[0] for slot in slots}):
                setup.ready(host, sum(1 for slot in slots if slot[0] == host), max(slot[2] for slot in slots if slot[0] == host))
            records, deferred, failures = run_queue_two(context, slots, [JobTwo("cross", row, 0.0)], expected_hashes, setup, avoid_host=host_a)
            if failures or deferred:
                raise one.Stopped("跨机对照那一行没跑成：" + "；".join(failures or ["放不下"]))
        merge_step_two(context, step, (row,), records, setup.snapshot, expected_hashes)
    host_b = one.host_of(context, one.read_ledger(context), "cross", row)
    if host_a == host_b:
        raise one.Stopped(f"跨机对照的两次都在{one.host_name(host_a)}（账 row-hosts.tsv）：对照没做成")
    parts = cross_host_parts_two(one.row_line(context.product(ROWS_STEP.stem), row), os.path.join(context.step_root(ROWS_STEP), f"row-{row}"),
                                 one.row_line(context.product(step.stem), row), os.path.join(context.step_root(step), f"row-{row}"))
    text, stop = cross_host_product_two(parts, row, host_a, host_b, "")
    one.install(context, path, text)
    one.say(context, f"  跨机一致对照（第 {row} 行，{one.host_name(host_a)} 对 {one.host_name(host_b)}）：" + "；".join(f"{part}={identical}" for part, identical, _ in parts))
    if stop:
        raise one.Stopped(f"跨机一致对照不同（停机，不作废、不当结果；见 {os.path.relpath(path, context.repo)}）")


def phase_timeout_reruns(context: ContextTwo, hosts: list, setup, expected_hashes: dict[str, str]) -> None:
    """文件头「超时重跑」：整表产物里 device_exit=253 的行单线道再跑一次，限时再乘 3；另出产物，不改整表那一份。"""
    timed = one.timed_out_rows(context.product(ROWS_STEP.stem))
    one.say(context, f"  超时的行：{one.rows_text(timed) or 'none'}")
    if not timed or os.path.exists(context.product(RERUN_STEP.stem)):
        return
    remaining = tuple(row for row in timed if not one.is_done(context, RERUN_STEP, row))
    records: list[dict] = []
    if remaining:
        chosen = one.single_lane_host(current_allocations(context, hosts))
        one.say(context, f"  超时重跑第 {one.rows_text(remaining)} 行 → {one.host_name(chosen.number)}，单线道 {chosen.threads} 线程（限时再乘 3 由驱动现算）")
        setup.ready(chosen.number, 1, chosen.threads)
        records, deferred, failures = run_queue_two(context, [(chosen.number, 0, chosen.threads)], [JobTwo("rerun", row, 0.0) for row in remaining],
                                                    expected_hashes, setup)
        if failures or deferred:
            raise one.Stopped("超时重跑没跑成：" + "；".join(failures or ["放不下"]))
    merge_step_two(context, RERUN_STEP, timed, records, setup.snapshot, expected_hashes)
    check_rows_two(context, context.product(RERUN_STEP.stem))


def rows_of_this_run(context: ContextTwo) -> tuple[int, ...]:
    """整表 1–281；只在 --selftest 起的子进程里认 ROWS_ORCHESTRATOR_TWO_ROWS（假世界里只有几行）。"""
    override = os.environ.get(SELFTEST_ROWS_VARIABLE, "")
    if override and os.environ.get(SELFTEST_VARIABLE) == "1":
        return tuple(sorted(int(token) for token in override.split(",") if token))
    return whole_table_rows(os.path.join(context.research_root, one.RESEARCH_RELATIVE))


def plan_rows(context: ContextTwo) -> None:
    """行集、权重、跨机对照那一行、每行推的用时（纯读，plan 与 run 共用）。"""
    context.rows = rows_of_this_run(context)
    measured: dict[int, float] = {}
    for name in S4_TIMING_PRODUCTS:
        path = os.path.join(context.repo, "research", "results", name)
        if os.path.exists(path):
            measured.update(s4_device_seconds(open(path, encoding="utf-8").read().splitlines()))
    weights = row_weights(context.rows, {row: seconds for row, seconds in measured.items() if row in context.rows})
    context.cross_host_row = choose_cross_host_row(weights)
    context.row_seconds = estimated_row_seconds(one.process_wall_seconds(os.path.join(context.repo, context.baseline_timing)) or 0.0, weights)


def run_two(context: ContextTwo, plan_only: bool) -> int:
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, one.on_signal)
    one.snapshot_check(context)
    hosts, values, note = one.load_hosts(context)
    plan_rows(context)
    if plan_only:
        if note:
            print(note)
        resources = one.query_resources(context, hosts)
        allocations = one.allocate(hosts, resources)
        slots = slots_of(allocations)
        jobs = [JobTwo("rows", row, context.row_seconds[row]) for row in context.rows if not one.is_done(context, ROWS_STEP, row)]
        for line in one.allocation_text(allocations, resources) + prediction_lines(slots, simulate_two(slots, jobs)):
            print(line)
        longest = sorted(context.row_seconds.items(), key=lambda item: (-item[1], item[0]))[:10]
        print(f"行 {len(context.rows)}，没做完 {len(jobs)}；跨机对照那一行第 {context.cross_host_row} 行；推的用时最长十行（秒）："
              + " ".join(f"{row}={seconds:.0f}" for row, seconds in longest))
        return 0
    context.attempt_dir = one.next_attempt_directory(context)
    one.say(context, f"第二段多机编排开跑：{os.path.basename(context.attempt_dir)}，行输出根 {os.path.relpath(context.rows_root, context.repo)}，"
                     f"{len(context.rows)} 行，跨机对照那一行第 {context.cross_host_row} 行")
    if note:
        one.say(context, "  " + note)
    for record in one.shard_records(context):
        if record["kind"] == "peer" and record["attempt"] != os.path.basename(context.attempt_dir):
            harvested = harvest_two(context, record)
            if harvested:
                one.say(context, f"  收进以前取回的行：{record['label']}（{record['attempt']}）第 {one.rows_text(harvested)} 行")
    allocations = current_allocations(context, hosts)
    expected_hashes = one.driver_hashes(context)
    setup = one.PeerSetup(context, hosts, values, one.build_snapshot_staging(context))
    phase_rows(context, hosts, slots_of(allocations), setup, expected_hashes)
    phase_cross_host(context, hosts, setup, expected_hashes)
    phase_timeout_reruns(context, hosts, setup, expected_hashes)
    one.snapshot_check(context)
    one.say(context, "✓ 第二段整表绿（合成、判法、补位协调、跨机一致对照、超时重跑）")
    return 0


def context_from_arguments_two(arguments: argparse.Namespace) -> ContextTwo:
    arguments.resume_tag = "-s2"  # 只进别的台那一趟的标签（第一段编排器的 peer_label 用它），产物名与行输出根不接它
    base = one.context_from_arguments(arguments)
    return ContextTwo(**{field.name: getattr(base, field.name) for field in dataclasses.fields(one.Context)})


def main(argument_list: list[str]) -> int:
    if os.environ.get(BREAK_VARIABLE) and os.environ.get(SELFTEST_VARIABLE) != "1":
        print(f"✗ {BREAK_VARIABLE} 只在 --selftest 起的子进程里认；平常跑时设了它就拒（退 2）", file=sys.stderr)
        return 2
    if os.environ.get("ROWS_ORCHESTRATOR_BREAK") and os.environ.get("ROWS_ORCHESTRATOR_SELFTEST") != "1":
        print("✗ ROWS_ORCHESTRATOR_BREAK（第一段编排器的弄坏开关，这一层调的函数里也认）只在自证里认；平常跑时设了它就拒（退 2）", file=sys.stderr)
        return 2
    if os.environ.get(SELFTEST_ROWS_VARIABLE) and os.environ.get(SELFTEST_VARIABLE) != "1":
        print(f"✗ {SELFTEST_ROWS_VARIABLE} 只在 --selftest 起的子进程里认（平常跑整表 1–281）；设了它就拒（退 2）", file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 第二段整表的多机编排")
    parser.add_argument("mode", choices=("run", "plan"))
    for name in ("--repo", "--prefix", "--suffix", "--dumps", "--tree", "--snapshot-digest", "--baseline-timing", "--check", "--logs", "--work-root"):
        parser.add_argument(name, required=True)
    parser.add_argument("--research-root")
    arguments = parser.parse_args(argument_list)
    context = context_from_arguments_two(arguments)
    try:
        return run_two(context, arguments.mode == "plan")
    except one.RunCondition as error:
        one.say(context, f"✗ 运行条件不满足（退 78）：{error}")
        return one.EXIT_RUN_CONDITION
    except one.Stopped as error:
        one.say(context, f"✗ 停：{error}")
        return 1
    except one.Interrupted:
        one.say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已取回收进来；同一条命令再起就是续跑")
        return one.EXIT_STOPPED_BY_SIGNAL


# ── 自证：进程收尾（自证进程是子进程的收养者，收的是它自己那一支进程树；不按名字、不按模式挑进程）──
def become_subreaper() -> None:
    """PR_SET_CHILD_SUBREAPER：自证起的进程的父进程先退了，它们过继给自证进程（不过继给 init），收尾时从自证进程往下数得到。"""
    library = ctypes.CDLL(None, use_errno=True)
    if library.prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), "prctl(PR_SET_CHILD_SUBREAPER) 没设上")


def process_table() -> dict[int, tuple[int, str]]:
    """/proc 里每个进程的 (父进程号, 状态)；读的时候退了的跳过。"""
    table: dict[int, tuple[int, str]] = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                after_name = handle.read().rsplit(")", 1)[1].split()
        except (FileNotFoundError, ProcessLookupError, IndexError):
            continue
        table[int(entry)] = (int(after_name[1]), after_name[0])
    return table


def live_descendants(root: int) -> list[int]:
    """root 往下整棵进程树里还活着的（不算僵尸），不含 root 自己。"""
    table = process_table()
    children: dict[int, list[int]] = {}
    for process_id, (parent, _) in table.items():
        children.setdefault(parent, []).append(process_id)
    found, pending = [], list(children.get(root, []))
    while pending:
        process_id = pending.pop()
        if table[process_id][1] not in ("Z", "X"):
            found.append(process_id)
        pending.extend(children.get(process_id, []))
    return sorted(found)


def reap_zombies() -> None:
    """过继来的、已经退了的子进程：收掉它们的退出码（只在一格做完之后调，那时没有自证自己还在等的子进程）。"""
    while True:
        try:
            reaped_process_id, _ = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            return
        if reaped_process_id == 0:
            return


def command_line_of(process_id: int) -> str:
    try:
        with open(f"/proc/{process_id}/cmdline", "rb") as handle:
            return handle.read().replace(b"\0", b" ").decode("utf-8", "replace").strip()[:160]
    except (FileNotFoundError, ProcessLookupError):
        return "-"


def sweep_own_processes() -> list[str]:
    """收掉自证进程那一支进程树里还活着的每一个：TERM，等 5 秒，还在的 KILL，再收僵尸。交回收掉的（进程号与命令行）。"""
    if broken("sweep-skipped"):
        return []
    stopped: list[str] = []
    for _ in range(3):
        live = live_descendants(os.getpid())
        if not live:
            break
        for process_id in live:
            stopped.append(f"{process_id}:{command_line_of(process_id)}")
            try:
                os.kill(process_id, signal.SIGTERM)
            except ProcessLookupError:
                pass
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline and live_descendants(os.getpid()):
            reap_zombies()
            time.sleep(0.05)
        for process_id in live_descendants(os.getpid()):
            try:
                os.kill(process_id, signal.SIGKILL)
            except ProcessLookupError:
                pass
        time.sleep(0.1)
        reap_zombies()
    return stopped


def start_straggler(marker: str) -> None:
    """有意留一个父进程先退了的进程（照「TERM 后续跑」那几格留下的那种空转的假驱动）：中间那一层起它之后就退，它睡 600 秒；进程号写进 marker。"""
    # 标准输入输出都接 /dev/null：它要是拿着自证的输出管道，起自证的那一方读不到管道关上、要陪它等满 600 秒
    code = ("import os, subprocess, sys; process = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(600)'], start_new_session=True, "
            "stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); "
            f"open({marker!r}, 'w').write(str(process.pid))")
    subprocess.run([sys.executable, "-c", code], check=True, timeout=60, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


# ── 自证：纯函数各格 ──
REAL_REPOSITORY = one.REAL_REPOSITORY
FAKES = os.path.join(HERE, "selftest_fakes")


def cells_planning() -> list[tuple[str, bool, str]]:
    results = []
    measured = s4_device_seconds(["E7TIMING name=row_time row=1 copy_ns=1 compile_ns=1 compile_kind=cold device_ns=40000000000 o3_ns=1",
                                  "E7TIMING name=row_time row=2 copy_ns=1 compile_ns=1 compile_kind=cold device_ns=- o3_ns=-",
                                  "E7TIMING name=row_time row=3 copy_ns=1 compile_ns=1 compile_kind=cold device_ns=400000000000 o3_ns=1",
                                  "E7TIMING name=row_time row=4 copy_ns=1 compile_ns=1 compile_kind=cold device_ns=20000000000 o3_ns=1"])
    weights = row_weights((1, 2, 3, 4, 5), measured)
    seconds = estimated_row_seconds(75.0, weights)
    results.append(("plan-weights", measured == {1: 40.0, 3: 400.0, 4: 20.0} and weights == {1: 1.0, 2: 1.0, 3: 10.0, 4: 0.5, 5: 1.0}
                    and seconds[3] == 75.0 * 10 + 120 and choose_cross_host_row(weights) == 1,
                    f"量到 {measured}；权重 {weights}；第 3 行推 {seconds[3]:.0f} 秒（要 870）；跨机对照那一行 {choose_cross_host_row(weights)}（要 1）"))
    order = [job.row for job in queue_order_two([JobTwo("rows", row, seconds[row]) for row in seconds])]
    results.append(("plan-longest-first", order == [3, 1, 2, 5, 4], f"派发次序 {order}，要 [3, 1, 2, 5, 4]"))
    placed = simulate_two([(1, 0, 10), (2, 0, 10)], [JobTwo("rows", row, seconds[row]) for row in seconds])
    hosts = {job.row: host for job, host, _, _ in placed}
    results.append(("plan-simulate", hosts == {3: 1, 1: 2, 2: 2, 5: 2, 4: 2} and len(placed) == 5, f"各行落在 {hosts}"))
    return results


def cells_substitution() -> list[tuple[str, bool, str]]:
    results = []
    sys.path.insert(0, os.path.join(REAL_REPOSITORY, one.RESEARCH_RELATIVE))
    import bad_implementation_table  # noqa: E402
    import row_driver  # noqa: E402
    names = {"实二五 准入：漏掉树表那一项, 第二处", "C375 实现 | 全池承诺量", "g1"}
    with tempfile.NamedTemporaryFile("w", suffix=".tsv", delete=False, encoding="utf-8") as handle:
        handle.write(exclusion_table_text(names))
    read_back = {row.name for row in bad_implementation_table.read_selected_rows(handle.name)}
    skipped = row_driver.names_in_the_tables([], [handle.name])
    os.remove(handle.name)
    results.append(("exclusions-round-trip", read_back == names and skipped == names,
                    f"写进去 {len(names)} 个名字（带逗号、空格、竖线），驱动读回 {len(read_back)} 个、补位跳过 {len(skipped)} 个"))
    used = substitute_names_used({"row_name": "原", "substituted_from": "原", "mutated_row_name": "g2", "failed_attempts": "2",
                                  "failed_attempt_1": "compile_failed:原", "failed_attempt_2": "anchor_not_unique:g1"})
    plain = substitute_names_used({"row_name": "甲", "substituted_from": "-", "mutated_row_name": "甲", "failed_attempts": "0"})
    exhausted = substitute_names_used({"row_name": "乙", "substituted_from": "-", "mutated_row_name": "-", "failed_attempts": "2",
                                       "failed_attempt_1": "compile_failed:乙", "failed_attempt_2": "compile_failed:g5"})
    results.append(("substitution-used-names", used == {"g1", "g2"} and plain == set() and exhausted == {"g5"},
                    f"补上的 {sorted(used)}、没补位的 {sorted(plain)}、补不上的 {sorted(exhausted)}"))
    found = collisions({2: {"g1"}, 3: {"g1", "g4"}, 7: {"g4"}, 9: set()})
    results.append(("substitution-collisions", found == {"g1": [2, 3], "g4": [3, 7]} and rows_to_redo(found) == [3, 7],
                    f"重复 {found}，重做 {rows_to_redo(found)}"))
    text = coordination_product({2: {"g1"}, 3: {"g 2,x"}, 4: set()}, {row: {"status": "ok", "substituted_from": "原 名", "mutated_row_name": "g1"} for row in (2, 3, 4)},
                                {3: 1}, {})
    lines = text.splitlines()
    results.append(("substitution-product", len(lines) == 4 and "used=g%202%2Cx" in lines[2] and lines[-1].endswith("redone_rows=3")
                    and "substitute_collisions=0" in lines[-1] and all(len(line.split(" ")) == len([token for token in line.split(" ") if token]) for line in lines),
                    f"末行 {lines[-1]}"))
    return results


def fake_shard_two(label: str, host: int, rows: tuple[int, ...], limit: str = "1800", digest: str = "d" * 64, extra: str = "") -> object:
    product = [f"name=row_time_limit seconds={limit} baseline_seconds=75.274 source=x-timing.out rerun_of_timed_out_rows=false",
               f"snapshot_digest={digest} files=9 snapshot_ns=5 rows={len(rows)}"]
    product += [f"row row_number={row} row_name=r{row} lane=0 status=ok device_exit=0 device_ns={row + 1} o3_status=skipped_by_request" for row in rows]
    product += [f"table lanes=1 threads_per_lane=10 rows={len(rows)} table_wall_ns=10 sum_of_row_ns=3"]
    if extra:
        product.insert(1, extra)
    timing = [line for row in rows for line in (f"E7TIMING name=row_device row={row} device_exit=0 device_ns=1", f"E7TIMING name=process_wall seconds=1 row{row}")]
    return one.Shard(label, host, "\n".join(product) + "\n", "\n".join(timing) + "\n")


def merge_refused_two(shards: list, rows: tuple[int, ...]) -> str:
    try:
        merge_products_two(shards, rows, ("d" * 64, "9"))
    except one.MergeRefused as error:
        return str(error)
    return ""


def cells_merge_two() -> list[tuple[str, bool, str]]:
    product, timing = merge_products_two([fake_shard_two("a", 2, (42,)), fake_shard_two("b", 1, (1, 15))], (1, 15, 42), ("d" * 64, "9"))
    lines = product.splitlines()
    rows = [one.key_values(line)["row_number"] for line in lines if line.startswith("row ")]
    cell_holds = (rows == ["1", "15", "42"] and lines[5] == "table lanes=2 threads_per_lane=10 rows=3 table_wall_ns=10 sum_of_row_ns=6 shards=2"
            and not any(line.startswith("freeze_check") for line in lines) and len([line for line in lines if line.startswith("merged_shard ")]) == 2
            and [one.key_values(line)["row"] for line in timing.splitlines() if "row_device" in line] == ["1", "15", "42"])
    results = [("merge-two-good", cell_holds, f"行 {rows}；table 行 {lines[5] if len(lines) > 5 else '-'}")]
    cases = [("merge-refuse-freeze-line", [fake_shard_two("a", 1, (1,), extra="freeze_check before=true differences=0 detail=none")], (1,), "④"),
             ("merge-refuse-duplicate", [fake_shard_two("a", 1, (1, 15)), fake_shard_two("b", 2, (15,))], (1, 15), "两个分片"),
             ("merge-refuse-missing", [fake_shard_two("a", 1, (1,))], (1, 15), "缺 [15]"),
             ("merge-refuse-time-limit", [fake_shard_two("a", 1, (1,)), fake_shard_two("b", 2, (15,), limit="1801")], (1, 15), "①"),
             ("merge-refuse-snapshot", [fake_shard_two("a", 1, (1,)), fake_shard_two("b", 2, (15,), digest="e" * 64)], (1, 15), "②"),
             ("merge-refuse-unknown-line", [fake_shard_two("a", 1, (1,), extra="rerun_compile_failed rows=1")], (1,), "⑦")]
    for name, shards, rows_wanted, mark in cases:
        refused = merge_refused_two(shards, rows_wanted)
        results.append((name, mark in refused, f"拒因：{refused or '没拒'}"))
    bad_timing = one.Shard("a", 1, fake_shard_two("a", 1, (1, 15)).product, "E7TIMING name=row_device row=1 device_exit=0 device_ns=1\n")
    refused = merge_refused_two([bad_timing], (1, 15))
    results.append(("merge-refuse-timing-two", "⑥" in refused, f"拒因：{refused or '没拒'}"))
    return results


def judge(scratch: str, name: str, lines: list[str]) -> tuple[int, str]:
    path = os.path.join(scratch, f"judge-{name}.out")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    completed = subprocess.run(["python3", "-B", os.path.join(HERE, "check_product.py"), "rows_without_executor", path], capture_output=True, text=True)
    return completed.returncode, completed.stdout.strip()


def cells_judge(scratch: str) -> list[tuple[str, bool, str]]:
    head = ["name=row_time_limit seconds=1800 baseline_seconds=75.274 source=x rerun_of_timed_out_rows=false", "snapshot_digest=d files=9 snapshot_ns=1 rows=4"]
    good_rows = ["row row_number=1 status=ok device_exit=0 o3_status=skipped_by_request", "row row_number=2 status=compile_failed device_exit=- o3_status=not_reached",
                 "row row_number=3 status=ok device_exit=253 o3_status=not_reached", "row row_number=5 status=not_found device_exit=- o3_status=not_reached"]
    tail = ["table lanes=1 threads_per_lane=10 rows=4 table_wall_ns=1 sum_of_row_ns=1"]
    code, output = judge(scratch, "good", head + good_rows + tail)
    results = [("judge-good", code == 0 and "rows=4 not_ok=[]" in output and "timed_out=3" in output and "wrapper_outcomes=none" in output
                and "not_located=2,5" in output, output)]
    for name, rows in (("judge-refuse-o3-ran", ["row row_number=1 status=ok device_exit=0 o3_status=ran"]),
                       ("judge-refuse-wrapper-outcome", ["row row_number=4 status=ok device_exit=252 o3_status=not_reached"]),
                       ("judge-refuse-stop-clause", ["row row_number=1 status=ok device_exit=3 o3_status=skipped_by_request"]),
                       ("judge-refuse-driver-error", ["row row_number=1 status=driver_error"]),
                       ("judge-refuse-freeze-line", ["freeze_check before=true differences=0 detail=none", "row row_number=1 status=ok device_exit=0 o3_status=skipped_by_request"])):
        code, output = judge(scratch, name, head + rows + tail)
        results.append((name, code == 1, f"退 {code}；{output}"))
    return results


def write_fake_row_two(directory: str, salt_part: str = "", threads: int = 5) -> str:
    """跨机对照格用的一个行目录（不跑执行器：没有 o3.out），交回它的 row 行；salt_part 往那一样里掺一个不同，耗时两边本来就不同。"""
    os.makedirs(directory)
    with open(os.path.join(directory, "device.out"), "w", encoding="utf-8") as handle:
        handle.write("E7RESULT name=row_result row=7" + (" salt=1" if salt_part == "device" else "") + f"\nE7TIMING name=process_wall seconds={threads}.5 exit=0\n")
    for stream in one.STREAMS[:2]:
        os.makedirs(os.path.join(directory, "dump", stream))
        for name in one.Z1_FILES[:2]:
            with open(os.path.join(directory, "dump", stream, name), "w", encoding="utf-8") as handle:
                handle.write(f"{stream} {name}" + (" salt" if salt_part == "dump" and name == "call-outcomes.tsv" else "") + "\n")
    return (f"row row_number=7 row_name=r7{'_掺' if salt_part == 'row' else ''} lane=0 copy_ns={threads} compile_ns={threads} status=ok device_exit=0 "
            f"device_ns={threads} o3_status=skipped_by_request o3_exit=- o3_ns=- o3_streams=-")


def cells_cross_host_two(scratch: str) -> list[tuple[str, bool, str]]:
    base = write_fake_row_two(os.path.join(scratch, "cross-a"), threads=5)
    outcomes = {}
    for part in ("", "device", "dump", "row"):
        other_directory = os.path.join(scratch, f"cross-b-{part or 'same'}")
        other = write_fake_row_two(other_directory, part, threads=7)
        parts = cross_host_parts_two(base, os.path.join(scratch, "cross-a"), other, other_directory)
        outcomes[part or "same"] = (sorted(name for name, identical, _ in parts if identical == "false"), [identical for name, identical, _ in parts if name == "o3"])
    expected = {"same": ([], ["not_run"]), "device": (["device_results"], ["not_run"]), "dump": (["dump_files"], ["not_run"]), "row": (["row_line"], ["not_run"])}
    same_parts = cross_host_parts_two(base, os.path.join(scratch, "cross-a"), write_fake_row_two(os.path.join(scratch, "cross-c"), threads=9),
                                      os.path.join(scratch, "cross-c"))
    text, stop = cross_host_product_two(same_parts, 7, 1, 2, "")
    single, single_stop = cross_host_product_two(None, 7, None, None, "只有一台")
    return [("cross-host-compare", outcomes == expected, f"各掺一样时不同的那几样与 o3 那一格：{outcomes}"),
            ("cross-host-verdict-two", not stop and "cross_host_identical=true" in text and "compared_parts=3" in text and "differing_parts=none" in text
             and not single_stop and "cross_host_identical=not_run" in single, text.splitlines()[-1])]


# ── 自证：假世界端到端各格 ──
@dataclasses.dataclass
class WorldTwo:
    root: str
    repo: str
    command: list[str]
    environment: dict[str, str]
    prefix: str
    dumps: str


def fake_world_two(root: str, rows: str = "1,2,3,4,5,6", hosts_code: int = 0, resources: list[dict] | None = None,
                   extra_environment: dict[str, str] | None = None, resources_command: str | None = None) -> WorldTwo:
    repo = os.path.join(root, "repo")
    results = os.path.join(repo, "research", "results")
    os.makedirs(results)
    tree = os.path.join(repo, "scripts", "experiments", "target", "fake-frozen-tree")
    for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "# lock\n"), (".cargo/config.toml", "[env]\n"),
                           ("crates/a/src/lib.rs", "pub fn a() {}\n"), ("target/release/build/x.rs", "// 不进快照\n")):
        os.makedirs(os.path.dirname(os.path.join(tree, relative)), exist_ok=True)
        with open(os.path.join(tree, relative), "w", encoding="utf-8") as handle:
            handle.write(text)
    prefix, suffix = "research/results/fake-e171-s2", "-r6"
    baseline = "research/results/fake-e171-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out"
    with open(os.path.join(repo, baseline), "w", encoding="utf-8") as handle:
        handle.write("E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds=75.274 exit=0\n")
    frozen_reference = "e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
    os.symlink(os.path.join(REAL_REPOSITORY, "research", "results", frozen_reference), os.path.join(results, frozen_reference))
    os.makedirs(os.path.join(repo, "scripts", "experiments", "target", "fake-dumps"))
    sys.path.insert(0, os.path.join(REAL_REPOSITORY, one.RESEARCH_RELATIVE))
    import row_driver  # noqa: E402
    digest = row_driver.snapshot_digest(tree)[0]
    assignments = ("PEER_SSH_HOST=fakepeer\nPEER_REPOSITORY_DIRECTORY=/fake/peer\nPEER_CARGO_BIN_DIRECTORY=/fake/cargo\nPEER_MEMORY_CAP=50G\n"
                   "QUIESCE_STOP_COMMAND=''\nQUIESCE_STOPPED_CHECK_COMMAND=''\nQUIESCE_START_COMMAND=''\nQUIESCE_STARTED_CHECK_COMMAND=''\n"
                   "ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=0\nMULTI_HOST_COUNT=1\nMULTI_HOST_2_SSH_HOST=fakepeer\n"
                   "MULTI_HOST_2_REPOSITORY_DIRECTORY=/fake/peer\nMULTI_HOST_2_CARGO_BIN_DIRECTORY=/fake/cargo\nMULTI_HOST_2_MEMORY_CAP=50G\nMULTI_HOST_2_IS_PEER=1\n")
    with open(os.path.join(root, "hosts.txt"), "w", encoding="utf-8") as handle:
        handle.write(assignments)
    with open(os.path.join(root, "resources.json"), "w", encoding="utf-8") as handle:
        json.dump({"hosts": resources or [one.resources_entry(1, 10, 25, 25), one.resources_entry(2, 10, 25, 25)]}, handle)
    os.makedirs(os.path.join(root, "peer"))
    environment = dict(os.environ, ROWS_ORCHESTRATOR_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=os.path.join(FAKES, "fake_row_driver.py"),
                       ROWS_ORCHESTRATOR_MULTI_HOST_RUN=os.path.join(FAKES, "fake_multi_host_run.sh"),
                       ROWS_ORCHESTRATOR_HOSTS_COMMAND=f"cat {shlex.quote(os.path.join(root, 'hosts.txt'))}; exit {hosts_code}",
                       ROWS_ORCHESTRATOR_RESOURCES_COMMAND=resources_command or f"cat {shlex.quote(os.path.join(root, 'resources.json'))}",
                       ROWS_ORCHESTRATOR_POLL_SECONDS="0.05", ROWS_ORCHESTRATOR_PEER_COMPILE_CHECK="0",
                       FAKE_RESEARCH_DIRECTORY=os.path.join(REAL_REPOSITORY, one.RESEARCH_RELATIVE), FAKE_REAL_REPOSITORY=REAL_REPOSITORY,
                       FAKE_RESULTS_DIRECTORY=results, FAKE_PEER_ROOT=os.path.join(root, "peer"), FAKE_DRIVER_LOG=os.path.join(root, "driver.log"),
                       FAKE_MHR_LOG=os.path.join(root, "mhr.log"), PYTHONDONTWRITEBYTECODE="1", **{SELFTEST_VARIABLE: "1", SELFTEST_ROWS_VARIABLE: rows})
    environment.update(extra_environment or {})
    dumps = "scripts/experiments/target/fake-dumps"
    command = ["python3", "-B", os.path.abspath(__file__), "run", "--repo", repo, "--prefix", prefix, f"--suffix={suffix}", "--dumps", dumps,
               "--tree", "scripts/experiments/target/fake-frozen-tree", "--snapshot-digest", digest, "--baseline-timing", baseline,
               "--check", os.path.join(HERE, "check_product.py"), "--logs", os.path.join(root, "logs"), "--work-root", os.path.join(root, "work"),
               "--research-root", REAL_REPOSITORY]
    return WorldTwo(root, repo, command, environment, prefix + "-", dumps)


def world_run_two(world: WorldTwo, timeout: float = 180) -> tuple[int, str]:
    """起编排器、等它退；到点还没退就停它（TERM，10 秒后 KILL），它起的、还活着的由每一格之后的收尾收掉。"""
    process = subprocess.Popen(world.command, cwd=world.repo, env=world.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        output, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.terminate()
        try:
            output, _ = process.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            output, _ = process.communicate()
        return 124, (output or "") + f"\n（{timeout} 秒没退，停了）"
    return process.returncode, output


def world_product_two(world: WorldTwo, stem: str) -> str:
    return os.path.join(world.repo, f"{world.prefix}{stem}-r6.out")


def ran_counts_two(world: WorldTwo) -> dict[tuple[int, str], int]:
    """假驱动真跑过的次数，按 (行, 是不是超时重跑) 数（别的台那一头的输出根一律叫 rows，分不出步）：整表一次加跨机对照那一行再一次。"""
    counts: dict[tuple[int, str], int] = {}
    path = os.path.join(world.root, "driver.log")
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        if line.startswith("ran "):
            fields = one.key_values(line)
            key = (int(fields["row"]), fields["rerun"])
            counts[key] = counts.get(key, 0) + 1
    return counts


def ran_hosts_two(world: WorldTwo) -> set[str]:
    path = os.path.join(world.root, "driver.log")
    return {one.key_values(line)["host"] for line in open(path, encoding="utf-8") if line.startswith("ran ")} if os.path.exists(path) else set()


def product_rows(path: str) -> dict[int, dict[str, str]]:
    return {int(one.key_values(line)["row_number"]): one.key_values(line) for line in open(path, encoding="utf-8") if line.startswith("row ")} \
        if os.path.exists(path) else {}


def last_verdict(path: str) -> str:
    if not os.path.exists(path):
        return "产物不在"
    found = [line.strip() for line in open(path, encoding="utf-8") if " name=verdict " in line]
    return found[-1] if found else "没有 verdict 行"


def plain_counts(rows: tuple[int, ...], cross_row: int | None, reruns: tuple[int, ...] = ()) -> dict[tuple[int, str], int]:
    counts = {(row, "false"): 1 for row in rows}
    if cross_row is not None:
        counts[(cross_row, "false")] += 1
    for row in reruns:
        counts[(row, "true")] = 1
    return counts


def two_hosts_cell(scratch: str) -> tuple[str, bool, str]:
    world = fake_world_two(os.path.join(scratch, "two-hosts"), extra_environment={"FAKE_DRIVER_TIMEOUT_ROWS": "4"})
    code, output = world_run_two(world)
    counts, rows = ran_counts_two(world), product_rows(world_product_two(world, "rows-driver"))
    ledger_path = os.path.join(world.repo, world.dumps, "segment2-rows-shards", "row-hosts.tsv")
    ledger = open(ledger_path, encoding="utf-8").read() if os.path.exists(ledger_path) else ""
    expected = plain_counts((1, 2, 3, 4, 5, 6), 1, (4,))
    cell_holds = (code == 0 and counts == expected and sorted(rows) == [1, 2, 3, 4, 5, 6] and "fetched" in ledger and ran_hosts_two(world) == {"1", "2"}
            and "cross_host_identical=true" in last_verdict(world_product_two(world, CROSS_HOST_CHECK_STEM))
            and "substitute_collisions=0" in last_verdict(world_product_two(world, COORDINATION_STEM))
            and sorted(product_rows(world_product_two(world, "rows-timeout-rerun-driver"))) == [4]
            and os.path.exists(world_product_two(world, "rows-cross-host-row-1-driver")))
    return ("e2e-two-hosts", cell_holds, f"退 {code}；真跑过的次数与预期{'相同' if counts == expected else '不同：' + str(counts)}；整表行 {sorted(rows)}；"
                                   f"跨机 {last_verdict(world_product_two(world, CROSS_HOST_CHECK_STEM))[-60:]}；末行 {output.strip().splitlines()[-1:]}")


def substitution_sequential_cell(scratch: str) -> tuple[str, bool, str]:
    """一台一槽、一次一行：第 2 行补上 g1，第 3 行起的时候排除表里已有 g1，补上 g2，不用重做。"""
    world = fake_world_two(os.path.join(scratch, "substitution-sequential"), rows="1,2,3,4", hosts_code=3, resources=[one.resources_entry(1, 5, 25, 25)],
                           extra_environment={"FAKE_DRIVER_SUBSTITUTES": "2:g1,g2;3:g1,g2"})
    code, output = world_run_two(world)
    rows = product_rows(world_product_two(world, "rows-driver"))
    mutated = {row: rows[row].get("mutated_row_name") for row in rows}
    verdict = last_verdict(world_product_two(world, COORDINATION_STEM))
    cell_holds = code == 0 and mutated.get(2) == "g1" and mutated.get(3) == "g2" and "redone_rows=none" in verdict and ran_counts_two(world) == plain_counts((1, 2, 3, 4), None)
    return ("e2e-substitution-sequential", cell_holds, f"退 {code}；补上的 {mutated}；{verdict[-80:]}；末行 {output.strip().splitlines()[-1:]}")


def substitution_collision_cell(scratch: str) -> tuple[str, bool, str]:
    """一台两槽、两行同时起、都补上 g1：核出重复，行号大的第 3 行挪走、带着 g1 重做，补上 g2。"""
    world = fake_world_two(os.path.join(scratch, "substitution-collision"), rows="2,3", hosts_code=3, resources=[one.resources_entry(1, 10, 25, 25)],
                           extra_environment={"FAKE_DRIVER_SUBSTITUTES": "2:g1,g2;3:g1,g2", "FAKE_DRIVER_ROW_SECONDS": "0.3"})
    code, output = world_run_two(world)
    rows = product_rows(world_product_two(world, "rows-driver"))
    mutated = {row: rows[row].get("mutated_row_name") for row in rows}
    verdict = last_verdict(world_product_two(world, COORDINATION_STEM))
    counts = ran_counts_two(world)
    cell_holds = (code == 0 and mutated == {2: "g1", 3: "g2"} and "redone_rows=3" in verdict and "substitute_collisions=0" in verdict
            and counts == {(2, "false"): 1, (3, "false"): 2}
            and os.path.isdir(os.path.join(world.repo, world.dumps, "segment2-rows-shards", "superseded")))
    return ("e2e-substitution-collision", cell_holds, f"退 {code}；补上的 {mutated}；真跑过 {counts}；{verdict[-80:]}；末行 {output.strip().splitlines()[-1:]}")


def resume_cell_two(scratch: str) -> tuple[str, bool, str]:
    """跑到一半收 TERM（退 143），同一条命令再起（退 0）：每个 (行, 输出根) 只真跑过一次；第二台已做完的行在 TERM 时取回、续跑时收进来。"""
    world = fake_world_two(os.path.join(scratch, "resume"), extra_environment={"FAKE_DRIVER_ROW_SECONDS": "0.4"})
    process = subprocess.Popen(world.command, cwd=world.repo, env=world.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        deadline = time.monotonic() + 120
        log_path = os.path.join(world.root, "driver.log")
        while time.monotonic() < deadline:
            lines = open(log_path, encoding="utf-8").read().splitlines() if os.path.exists(log_path) else []
            if sum(line.startswith("ran ") and " host=2 " in line + " " for line in lines) >= 2 and sum(line.startswith("ran ") for line in lines) >= 3:
                break
            time.sleep(0.05)
        process.send_signal(signal.SIGTERM)
        process.communicate(timeout=120)
    except subprocess.TimeoutExpired:
        process.kill()
        process.communicate()
    first = process.returncode
    before = sum(ran_counts_two(world).values())
    world.environment["FAKE_DRIVER_ROW_SECONDS"] = "0"
    second, output = world_run_two(world)
    counts = ran_counts_two(world)
    ledger = os.path.join(world.repo, world.dumps, "segment2-rows-shards", "row-hosts.tsv")
    fetched = os.path.exists(ledger) and "fetched" in open(ledger, encoding="utf-8").read()
    expected = plain_counts((1, 2, 3, 4, 5, 6), 1)
    cell_holds = first == one.EXIT_STOPPED_BY_SIGNAL and second == 0 and counts == expected and before >= 3 and fetched
    return ("e2e-resume", cell_holds, f"第一次退 {first}（停前做完 {before} 行），续跑退 {second}；真跑过的次数与预期{'相同' if counts == expected else '不同：' + str(counts)}；"
                                f"取回过第二台的行 {fetched}；续跑末行 {output.strip().splitlines()[-1:]}")


def refusal_cells_two(scratch: str) -> list[tuple[str, bool, str]]:
    results = []
    unreachable = fake_world_two(os.path.join(scratch, "unreachable"), resources=[one.resources_entry(1, 10, 25, 25), one.resources_entry(2, 0, 0, 0, read=False)])
    code, output = world_run_two(unreachable)
    results.append(("e2e-unreachable", code == one.EXIT_RUN_CONDITION and "没读到资源" in output and not os.path.exists(os.path.join(unreachable.root, "driver.log")),
                    f"退 {code}；{output.strip().splitlines()[-1:]}"))
    refused = fake_world_two(os.path.join(scratch, "config-refused"), hosts_code=1)
    code, output = world_run_two(refused)
    results.append(("e2e-config-refused", code == one.EXIT_RUN_CONDITION and "多机配置判不过" in output, f"退 {code}"))
    single = fake_world_two(os.path.join(scratch, "switch-off"), rows="1,2", hosts_code=3, resources=[one.resources_entry(1, 10, 25, 25)])
    code, output = world_run_two(single)
    results.append(("e2e-switch-off", code == 0 and "cross_host_identical=not_run" in last_verdict(world_product_two(single, CROSS_HOST_CHECK_STEM))
                    and "只用本机" in output and ran_hosts_two(single) == {"1"}, f"退 {code}"))
    preflight = fake_world_two(os.path.join(scratch, "preflight-refused"), extra_environment={"FAKE_MHR_EXIT_FOR_LABEL": "preflight:255"})
    code, output = world_run_two(preflight)
    results.append(("e2e-preflight-refused", code == one.EXIT_RUN_CONDITION and "预检没过" in output and not os.path.exists(os.path.join(preflight.root, "driver.log")),
                    f"退 {code}"))
    return results


def regrow_cell(scratch: str) -> tuple[str, bool, str]:
    """起跑时本机 4 个可用线程（一槽都放不下）、第二台一槽；重查时本机放得下了，整表的行有落到本机的。"""
    root = os.path.join(scratch, "regrow")
    os.makedirs(root)
    for name, entries in (("first.json", [one.resources_entry(1, 4, 25, 25), one.resources_entry(2, 5, 25, 25)]),
                          ("later.json", [one.resources_entry(1, 10, 25, 25), one.resources_entry(2, 5, 25, 25)])):
        with open(os.path.join(root, name), "w", encoding="utf-8") as handle:
            json.dump({"hosts": entries}, handle)
    script = os.path.join(root, "resources.sh")
    with open(script, "w", encoding="utf-8") as handle:
        handle.write('counter="$(dirname "$0")/calls"; calls=$(cat "$counter" 2>/dev/null || echo 0); echo $((calls + 1)) > "$counter"\n'
                     'if [ "$calls" -eq 0 ]; then cat "$(dirname "$0")/first.json"; else cat "$(dirname "$0")/later.json"; fi\n')
    world = fake_world_two(os.path.join(root, "world"), resources_command=f"bash {shlex.quote(script)}",
                           extra_environment={"FAKE_DRIVER_ROW_SECONDS": "0.3", "ROWS_ORCHESTRATOR_TWO_REGROW_SECONDS": "0.2"})
    code, output = world_run_two(world)
    path = os.path.join(world.root, "driver.log")
    # 本机整表的行写在 segment2-rows，第二台那一头的输出根叫 rows；跨机对照那一次（本机，segment2-rows-cross-host）不算
    rows_hosts = {one.key_values(line)["host"] for line in open(path, encoding="utf-8")
                  if line.startswith("ran ") and (" root=segment2-rows " in line or " root=rows " in line)} \
        if os.path.exists(path) else set()
    cell_holds = code == 0 and rows_hosts == {"1", "2"} and "加槽：本机" in output
    return ("e2e-regrow", cell_holds, f"退 {code}；整表的行落在 {sorted(rows_hosts)}；末行 {output.strip().splitlines()[-1:]}")


def segment_one_selftest_cell() -> tuple[str, bool, str]:
    """第一段编排器自己的 --selftest（34 项）在这一个收养者底下跑：它留下的进程由之后的收尾收掉、算进 no-leftover-processes。"""
    try:
        completed = subprocess.run(["python3", "-B", SEGMENT_ONE_ORCHESTRATOR_PATH, "--selftest"], capture_output=True, text=True, timeout=900,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    except subprocess.TimeoutExpired:
        return ("segment-one-selftest", False, "900 秒没退")
    last = (completed.stdout.strip().splitlines() or ["-"])[-1]
    return ("segment-one-selftest", completed.returncode == 0 and last == SEGMENT_ONE_SELFTEST_LAST_LINE, f"退 {completed.returncode}；末行「{last}」")


def straggler_cell(scratch: str) -> tuple[str, bool, str]:
    """有意留一个父进程先退了的进程，收尾要收得到它（收尾这一道检查分得出「有遗留」）。"""
    marker = os.path.join(scratch, "straggler.pid")
    start_straggler(marker)
    straggler_process_id = int(open(marker, encoding="utf-8").read())
    adopted = straggler_process_id in live_descendants(os.getpid())
    stopped = sweep_own_processes()
    gone = straggler_process_id not in live_descendants(os.getpid())
    return ("straggler-swept", adopted and gone and any(entry.startswith(f"{straggler_process_id}:") for entry in stopped),
            f"留下的进程 {straggler_process_id} 过继到自证进程 {adopted}；收尾收掉 {len(stopped)} 个；之后不在了 {gone}")


SWITCH_CELLS_TWO = {"sweep-skipped": "no-leftover-processes", "temporary-kept": "temporary-directory-removed",
                    "merge-allows-freeze-lines": "merge-refuse-freeze-line", "merge-allows-duplicates": "merge-refuse-duplicate",
                    "exclusions-not-given": "e2e-substitution-sequential", "collisions-ignored": "e2e-substitution-collision",
                    "cross-host-ignores-dumps": "cross-host-compare", "resume-reruns-finished": "e2e-resume", "harvest-skipped": "e2e-two-hosts",
                    "regrow-skipped": "e2e-regrow"}
END_TO_END_CELLS = {"e2e-two-hosts": two_hosts_cell, "e2e-substitution-sequential": substitution_sequential_cell,
                    "e2e-substitution-collision": substitution_collision_cell, "e2e-resume": resume_cell_two, "e2e-regrow": regrow_cell}
REFUSAL_CELLS = ("e2e-unreachable", "e2e-config-refused", "e2e-switch-off", "e2e-preflight-refused")


class SelftestInterrupted(Exception):
    """自证收到 TERM / INT / HUP：收掉它起的进程、删掉临时目录再退（不然它一退，过继给它的那些就过继给 init、没人收）。"""


def selftest(argument_list: list[str]) -> int:
    wanted = set(argument_list[argument_list.index("--cells") + 1].split(",")) if "--cells" in argument_list else None
    become_subreaper()
    private = tempfile.mkdtemp(prefix="e171-rows-orchestrator-two-selftest-")
    os.environ["TMPDIR"] = private
    tempfile.tempdir = private
    scratch = os.path.join(private, "cells")
    os.makedirs(scratch)
    leaks: list[str] = []

    def collect(cell: str) -> None:
        for entry in sweep_own_processes():
            leaks.append(f"{cell}→{entry}")

    def interrupt(signal_number, frame) -> None:  # noqa: ARG001
        raise SelftestInterrupted(signal_number)

    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, interrupt)
    try:
        results = (cells_planning() + cells_substitution() + cells_merge_two() + cells_judge(scratch) + cells_cross_host_two(scratch))
        # 端到端各格、第一段自证互不相干（各自一个假世界、一个临时目录），并起来跑；收尾在这一批都退了之后做
        batch = [(name, functools.partial(cell, scratch), False) for name, cell in END_TO_END_CELLS.items() if wanted is None or name in wanted]
        if wanted is None or wanted & set(REFUSAL_CELLS):
            batch.append(("e2e-refusals", functools.partial(refusal_cells_two, scratch), True))
        if wanted is None or "segment-one-selftest" in wanted:
            batch.append(("segment-one-selftest", segment_one_selftest_cell, False))
        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, len(batch))) as pool:
            futures = [(pool.submit(function), several) for _, function, several in batch]
            for future, several in futures:
                results += future.result() if several else [future.result()]
        collect("端到端那一批")
        results.append(straggler_cell(scratch))
        switch_results = []
        if wanted is None and not os.environ.get(BREAK_VARIABLE):
            switches = [(switch, cell, subprocess.Popen(["python3", "-B", os.path.abspath(__file__), "--selftest", "--cells", cell], stdout=subprocess.PIPE,
                                                         stderr=subprocess.STDOUT, text=True, env=dict(os.environ, **{BREAK_VARIABLE: switch, SELFTEST_VARIABLE: "1"})))
                        for switch, cell in SWITCH_CELLS_TWO.items()]
            for switch, cell, process in switches:
                try:
                    output, _ = process.communicate(timeout=900)
                except subprocess.TimeoutExpired:
                    process.kill()
                    output, _ = process.communicate()
                switch_results.append((switch, cell, process.returncode == 1 and f"✗ 自证 {cell}：" in output, process.returncode))
            sweep_own_processes()  # 弄坏开关那几趟有意留下的（例如 sweep-skipped 留下的那个）过继到这里，收掉，不算这一趟的遗留
        collect("末尾")
    except SelftestInterrupted as interrupted:
        sweep_own_processes()
        shutil.rmtree(private, ignore_errors=True)
        print(f"自证被停（信号 {interrupted.args[0]}）：它起的进程已收掉、临时目录已删；没判完，不算通过", flush=True)
        return one.EXIT_STOPPED_BY_SIGNAL
    still = live_descendants(os.getpid())
    results.append(("no-leftover-processes", not leaks and not still,
                    f"各格之后收尾收到的遗留 {len(leaks)} 个{('：' + '；'.join(leaks[:4])) if leaks else ''}；收尾之后还活着 {len(still)} 个"))
    if not broken("temporary-kept"):
        shutil.rmtree(private, ignore_errors=True)
    results.append(("temporary-directory-removed", not os.path.exists(private),
                    f"核了 1 个目录（自证给子进程的 TMPDIR {os.path.basename(private)}）：{'还在' if os.path.exists(private) else '已删'}"))
    red = 0
    for name, cell_is_green, detail in results:
        if wanted is not None and name not in wanted:
            continue
        red += 0 if cell_is_green else 1
        print(f"{'✓' if cell_is_green else '✗'} 自证 {name}：{detail}", flush=True)
    for switch, cell, caught, code in switch_results:
        red += 0 if caught else 1
        print(f"{'✓' if caught else '✗'} 弄坏开关 {switch} → 红在 {cell}：{'红了' if caught else f'没红（退 {code}）'}", flush=True)
    counted = len([1 for name, _, _ in results if wanted is None or name in wanted]) + len(switch_results)
    print(f"自证 {counted} 项，红 {red}")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
