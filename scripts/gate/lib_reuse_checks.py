#!/usr/bin/env python3
# admission: always 判的是这一次给的几趟结局、这一趟的判定库与此刻的登记表，上一次的结论不替这一次作保
# run-condition: command python3 bash
"""带判定库的门禁与入口共用的三样判法：复用三问、复用哨兵、上次红的清单；连同门禁 code-tooling 的 reuse-selftests 格按登记表逐份核（audit）。
口径照 .claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」里「复用也要自证」那一条（三语仓 zh 第 69 行一带）。
全仓只在这里判一份；各份脚本只接入口（python 按路径 importlib 导入，shell 经下面的命令行）。

一、复用三问（格名 reuse-three-questions，全仓统一；每份声称按判定入库 / 复用的脚本，自证里各有这一格）
  四趟用假驱动、小输入：头一趟（库是空的）、同一份输入第二趟、改一处不在任何判定闭包里的输入（注释、文件名、路径、没人引的函数）之后一趟、
  改闭包里一个函数单元之后一趟；每趟交「项 → 这一项的结局」（复用、现判、强制现判、拒算……）。
  ① same-inputs       第二趟现判 0，复用的项恰好是头一趟现判的那几项（条数相等、名字相同）；头一趟现判 0 条的问不出，判红
  ② unrelated-change  闭包外改动之后现判 0、每一项都复用
  ③ unit-change       闭包内改一个单元之后，现判的项恰好是 affected（闭包含它的那几项），其余全复用；affected 一条都没有、或是全部项的问不出，判红
  少复用一条判红，多现判一条也判红。复用哨兵抽中的那一条「强制现判」按复用算（它键相同、只是这一趟被抽去复判），一趟至多每份哨兵一条（几格各抽一份的给 sentinels=格数）。
  末尾打一行规范输出（门禁 code-tooling 的 reuse-selftests 格只认这一行）：
    REUSE_SELFTEST script=<脚本相对仓根的路径> same-inputs=reused unrelated-change=reused unit-change=rejudged-only-affected items=<项数> affected=<条数>
  哪一问不成立，那一问的值写 red，另起几行 ✗ 写红在哪、差的是哪几项。

二、复用哨兵（每一趟真跑都插：带判定库的门禁与入口开跑前抽、跑完判）
  候选 = 这一刻键相同、库里有判定的项（调用方拿与复用判法分开的那一份查法给：VerdictLibrary 形态的库用 verdict_library_candidate，别的库各给各的）。
  种子取 SINGLEFS_REUSE_SENTINEL_SEED（整数），不设就现取一个随机数；种子打进日志，凭它能复现这一趟抽中的是哪几条。
  抽 K 条当哨兵（K 取 SINGLEFS_REUSE_SENTINEL_COUNT，默认 3），再从剩下的里抽 1 条强制现判（只这一条不读库）；
    这一趟结束时：哨兵有一条被现判 ⇒ 整道红「复用失灵：<项> 键相同却现判了」；强制现判那一条的判定与库里那一条不同 ⇒ 整道红「判定不稳：<项>」；
    抽中强制现判的那一条这一趟还是复用 ⇒ 整道红（强制现判没起作用）。
  键相同的不足 K 条（库是空的也算）报「哨兵不足 K 条」，不红；这一趟不读库（全量、--no-reuse、没给库）报「不抽哨兵」，不红；
  抽中的项这一趟没报结局（没跑到）的逐条报出来，不红。
  python：s = ReuseSentinel(脚本, 标签); s.plan({项: 库里的判定}, reuse=…); s.forced(项); s.observe(项, 结局, 判定); problems = s.finish()
  shell：sentinel-plan / sentinel-forced / sentinel-observe / sentinel-finish 四条命令行，状态放调用方给的一份文件（见「命令行」）。

三、上次红的清单（每道门禁与入口跑完写，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）
  ~/.local/share/singlefs/last-runs/<脚本相对仓根的路径，/ 换成 __>.tsv（SINGLEFS_LAST_RUNS_DIRECTORY 设了放它底下）：一行一个红的项名或测试名，全绿写空文件。
  写入原子（同目录建临时文件、写完 fsync、改名换上）。整轮跑（ran 给 None）照这一趟的红整份写；只跑了几项的（--item、--check）
  上一份里这一趟没跑到的项照留、跑到的换成这一趟的结局。
  测试目标（跑一个 cargo 测试目标的驱动：crash-amplification-item.sh、crash-amplification-two-hosts.sh，经 last-run-target）：项名是「<测试目标>」，
  带 --test-filter 的是「<测试目标>::<用例函数>」；退 0 是绿、退 2 / 77 / 130 / 143（用法错、本次未跑、被停）不写、照留，别的退出码是红。
  整个目标跑的那一趟换掉这个目标的全部项（「<目标>」与「<目标>::…」），点名用例的那一趟只换那一项。没显式设 SINGLEFS_LAST_RUNS_DIRECTORY、
  而测试目标的源文件不在（别的自证拿造出来的假目标起真驱动）时不写，免得自证写进共用的清单。

四、门禁 code-tooling 的 reuse-selftests 格（audit）：按登记表 .claude/gate.d/verdict-library-scripts.tsv 逐份核
  登记表一行一份：路径<TAB>自证命令<TAB>判定库放哪<TAB>复用三问格名[<TAB># 说明]；表头 # scan: <glob…> 那几行是现扫的射程。
  第三列三种写法：判定库放哪（有判定库）、「无判定库：<欠账 C 号或「待立」>」（没有判定库的测试入口）、「不算：<理由>」（射程里扫到、只是提到别人的复用）。
  判：路径不在、同一路径登记两行、列数不对 ⇒ 红；射程里注释行写了「判定库」或「复用」（^#.*(判定库|复用)）的脚本登记表里没有 ⇒ 红（漏登记）、
    登记成「无判定库」⇒ 红（声称复用而没有这一格）；有判定库的第四列不是 reuse-three-questions ⇒ 红；
    自证在 .claude/hooks/selftest-registry.tsv 登记为 light（heavy-if-set 而命令里没设那个变量的同算 light；门禁阶段登记 samples-light 与 samples-light-marker 的同算）的现跑，
      输出里要有那一份脚本的规范输出且三问都过（认的脚本名：自证命令里那一份、这一行的路径、这一行自己第三列里点名的判法脚本），没有、或有一问 red ⇒ 红；
    登记为 heavy、heavy-if-set 而命令里设了那个变量、samples-heavy 与没登记的，读上一次输出
      （SINGLEFS_SELFTEST_OUTPUTS_DIRECTORY，默认 ~/.local/share/singlefs/selftest-outputs/<路径 / 换 __>.log，由 keep-output 写：头一行带那一份脚本当时的 sha256）：
      没有、或脚本已经变了 ⇒ 这一份本次未跑；有就照判那一行。无判定库的报「本次未跑：没有判定库」；不算的只列名。
  退 0 至少一份过、没有红；1 有红；77 一份都没判（全是本次未跑或不算）。成功句报过几份、本次未跑几份（逐个列名）、不算几份。

命令行（退出码：0 成、1 红、2 用法错）：
  lib_reuse_checks.py three-questions --script <路径> --first <文件> --second <文件> --unrelated <文件> --unit-change <文件> --affected <项>[,<项>…] [--sentinels <几份哨兵>] [--from-output]
      四份文件各是一趟的「项<TAB>结局」；带 --from-output 时四份是那一趟的原样输出，按「· <项>：绿|红，现判|复用|强制现判」那一行取（ITEM_LINE）
  lib_reuse_checks.py sentinel-plan <状态文件> --script <路径> [--label <名>] --candidates <文件>|--off <为什么>
      候选文件一行「项<TAB>库里的判定」；打抽中的哨兵、强制现判那一条与种子
  lib_reuse_checks.py sentinel-forced <状态文件> <项>          这一项是强制现判的那一条退 0，不是退 1
  lib_reuse_checks.py sentinel-observe <状态文件> <项> <结局> <判定>
  lib_reuse_checks.py sentinel-observe-many <状态文件> <结局文件>   结局文件一行「项<TAB>结局<TAB>判定」，一次记好几条
  lib_reuse_checks.py sentinel-finish <状态文件>              打判定；退 0 不红、1 红（✗ 行带出路）
  lib_reuse_checks.py verdict-library-candidate <目录> <种类> <组成表>   VerdictLibrary 形态（scripts/crash/lib_verdict_library.py）的库里这把键有判绿的记录退 0，没有退 1
  lib_reuse_checks.py last-run <脚本相对仓根的路径> <红的清单文件，- 是标准输入> [--ran <跑到的项清单文件>]
  lib_reuse_checks.py last-run-target <脚本相对仓根的路径> <测试目标> <退出码> [--test-filter <用例函数>] [--target-file <测试目标的源文件>]   文件头「三」「测试目标」
  lib_reuse_checks.py audit <根> --jobs <N> [--timeout <秒>]  门禁 code-tooling 的 reuse-selftests 格
  lib_reuse_checks.py sample <阶段文件名> <样本名>       门禁样本档（登记成轻的阶段）里的一份：拷进临时目录、跑 setup.sh、起那一道，输出照打、退出码照交
  lib_reuse_checks.py keep-output <根> <脚本相对仓根的路径> -- <命令…>   跑一条（登记为 heavy 的）自证，输出照原样打、另存成上一次输出，退出码照原样交
  lib_reuse_checks.py --selftest

弄坏开关 REUSE_CHECKS_BREAK=<项>[,<项>…]（只给证红用；自证逐个打开，核红在点名那一格）：
  short-reuse-passes          ① 只看第二趟现判 0、不比复用条数                     「三问：第二趟少复用一条」那一格判错
  extra-rejudge-passes        ③ 现判的项包住 affected 就算过（多现判的不红）       「三问：闭包内改动多现判一条」那一格判错
  rejudged-sentinel-passes    哨兵被现判不红                                        「哨兵：被现判判红」那一格判错；各份脚本自证里 reuse-sentinel 那一格同样判错
  forced-mismatch-passes      强制现判的判定与库里不同不红                          「哨兵：强制现判判定不同判红」那一格判错
  last-run-partial-overwrites 只跑了几项时整份覆盖上一份（没跑到的红丢了）          「上次红：只跑几项照留没跑到的」那一格判错
  last-run-in-place           不经临时文件、就地写                                  「上次红：原子换上」那一格判错
  audit-line-unchecked        audit 不看规范输出那一行，自证退 0 就算过              「audit：缺规范输出判红」那一格判错；门禁 code-tooling 的样本 reuse-selftests-red 判绿
  audit-scan-skipped          audit 不现扫射程，漏登记的不红                        「audit：漏登记判红」那一格判错
  audit-leaked-library        audit 另认的判法脚本取登记表最后一行的第三列，不取这一行自己的   「audit：每一行认自己第三列点名的判法脚本」那一格判错
"""
import concurrent.futures
import fnmatch
import glob
import hashlib
import importlib.util
import json
import os
import random
import re
import shlex
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

CELL_NAME = "reuse-three-questions"
SENTINEL_CELL_NAME = "reuse-sentinel"
LAST_RUN_CELL_NAME = "last-run-list"
LINE_PREFIX = "REUSE_SELFTEST"
QUESTIONS = (("same-inputs", "reused"), ("unrelated-change", "reused"), ("unit-change", "rejudged-only-affected"))
REUSED, JUDGED, FORCED = "复用", "现判", "强制现判"
ITEM_LINE = re.compile(r"^\s*· (?P<item>.+?)：(?P<verdict>绿|红)，(?P<state>强制现判|现判|复用)")
CANONICAL_LINE = re.compile(r"^REUSE_SELFTEST script=(?P<script>\S+) same-inputs=(?P<same>\S+) unrelated-change=(?P<unrelated>\S+) unit-change=(?P<unit>\S+)")
SENTINEL_COUNT_VARIABLE = "SINGLEFS_REUSE_SENTINEL_COUNT"
SENTINEL_SEED_VARIABLE = "SINGLEFS_REUSE_SENTINEL_SEED"
DEFAULT_SENTINEL_COUNT = 3
LAST_RUNS_VARIABLE = "SINGLEFS_LAST_RUNS_DIRECTORY"
DEFAULT_LAST_RUNS_DIRECTORY = "~/.local/share/singlefs/last-runs"
SELFTEST_OUTPUTS_VARIABLE = "SINGLEFS_SELFTEST_OUTPUTS_DIRECTORY"
DEFAULT_SELFTEST_OUTPUTS_DIRECTORY = "~/.local/share/singlefs/selftest-outputs"
REGISTRY = os.path.join(".claude", "gate.d", "verdict-library-scripts.tsv")
SELFTEST_REGISTRY = os.path.join(".claude", "hooks", "selftest-registry.tsv")
CLAIM_LINE = re.compile(r"^#.*(判定库|复用)")
NO_LIBRARY_PREFIX = "无判定库："
NOT_COUNTED_PREFIX = "不算："
DEFAULT_AUDIT_TIMEOUT_SECONDS = 3600
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
BREAK = {item for item in os.environ.get("REUSE_CHECKS_BREAK", "").split(",") if item}
SCRIPT_PATH = os.path.realpath(__file__)
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(SCRIPT_PATH)))
VERDICT_LIBRARY_PATH = os.path.join(REPOSITORY_ROOT, "scripts", "crash", "lib_verdict_library.py")
_VERDICT_LIBRARY = []


# ── 一、复用三问 ──

def counted_as_reused(state):
    return state in (REUSED, FORCED)


def three_questions(script, first, second, unrelated, unit_change, affected, sentinels=1):
    """交 (过没过, 规范输出那一行, [红在哪的句子])。四趟各是 {项: 结局}；affected 是闭包含被改单元的那几项；
    sentinels 是一趟里有几份复用哨兵（几格各抽一份的写格数）：一趟的强制现判至多这么多条。"""
    problems = []
    values = {}
    affected = set(affected)
    first_judged = sorted(item for item, state in first.items() if state == JUDGED)
    items = sorted(first)

    def forced_count(run):
        return sum(1 for state in run.values() if state == FORCED)

    # ① 同一份输入第二趟
    second_judged = sorted(item for item, state in second.items() if not counted_as_reused(state))
    second_reused = sorted(item for item, state in second.items() if counted_as_reused(state))
    same = []
    if not first_judged:
        same.append("头一趟一条都没现判（库不是空的，或驱动没跑到任何项），问不出第二趟复用了没有")
    if second_judged:
        same.append(f"第二趟现判 {len(second_judged)} 条（{'、'.join(second_judged)}），应为 0")
    if "short-reuse-passes" not in BREAK and second_reused != first_judged:
        missing = sorted(set(first_judged) - set(second_reused))
        extra = sorted(set(second_reused) - set(first_judged))
        same.append(f"第二趟复用 {len(second_reused)} 条，应等于头一趟现判的 {len(first_judged)} 条"
                    + (f"；少复用：{'、'.join(missing)}" if missing else "") + (f"；多出：{'、'.join(extra)}" if extra else ""))
    if forced_count(second) > sentinels:
        same.append(f"第二趟强制现判 {forced_count(second)} 条，一趟至多 {sentinels} 条（每份哨兵一条）")
    values["same-inputs"] = "red" if same else "reused"
    problems += [f"① 同一份输入跑两趟：{text}" for text in same]
    # ② 闭包外改动
    unrelated_judged = sorted(item for item, state in unrelated.items() if not counted_as_reused(state))
    other = []
    if unrelated_judged:
        other.append(f"现判 {len(unrelated_judged)} 条（{'、'.join(unrelated_judged)}），应为 0")
    if sorted(unrelated) != items:
        other.append(f"这一趟的项与头一趟不同（{len(unrelated)} 条对 {len(items)} 条），比不了")
    if forced_count(unrelated) > sentinels:
        other.append(f"强制现判 {forced_count(unrelated)} 条，一趟至多 {sentinels} 条（每份哨兵一条）")
    values["unrelated-change"] = "red" if other else "reused"
    problems += [f"② 改一处不在任何判定闭包里的输入：{text}" for text in other]
    # ③ 闭包内改一个单元
    unit_judged = {item for item, state in unit_change.items() if not counted_as_reused(state)}
    unit = []
    if not affected:
        unit.append("affected 一条都没有：改的单元不在任何项的闭包里，问不出只现判了含它的那几项")
    elif affected >= set(items):
        unit.append(f"affected 是全部 {len(items)} 项：问不出其余是不是复用，改一个只在部分项闭包里的单元")
    if affected - unit_judged:
        unit.append(f"闭包含被改单元的 {'、'.join(sorted(affected - unit_judged))} 没现判（少现判：照旧复用了）")
    extra = unit_judged - affected
    if extra and "extra-rejudge-passes" not in BREAK:
        unit.append(f"多现判 {len(extra)} 条：{'、'.join(sorted(extra))}（闭包里没有被改的单元，应复用）")
    if sorted(unit_change) != items:
        unit.append(f"这一趟的项与头一趟不同（{len(unit_change)} 条对 {len(items)} 条），比不了")
    if forced_count(unit_change) > sentinels:
        unit.append(f"强制现判 {forced_count(unit_change)} 条，一趟至多 {sentinels} 条（每份哨兵一条）")
    values["unit-change"] = "red" if unit else "rejudged-only-affected"
    problems += [f"③ 改闭包里一个函数单元：{text}" for text in unit]
    line = (f"{LINE_PREFIX} script={script} " + " ".join(f"{name}={values[name]}" for name, _green in QUESTIONS)
            + f" items={len(items)} affected={len(affected)}")
    return not problems, line, problems


def report_three_questions(script, first, second, unrelated, unit_change, affected, printer=print, sentinels=1):
    """判三问、打判定与规范输出那一行；交 (过没过, [红在哪的句子])。"""
    passed, line, problems = three_questions(script, first, second, unrelated, unit_change, affected, sentinels)
    for problem in problems:
        printer(f"    {CELL_NAME}：{problem}")  # gate-lint:detail
    printer(line)
    return passed, problems


def states_from_output(text):
    """一趟原样输出 → {项: 结局}：认「· <项>：绿|红，现判|复用|强制现判」那一行（同一项出现几次取最后一次）。"""
    states = {}
    for line in text.split("\n"):
        found = ITEM_LINE.match(line)
        if found:
            states[found.group("item")] = found.group("state")
    return states


def canonical_lines(text):
    """输出里的规范输出行 → [(脚本, {问: 值})]。"""
    found = []
    for line in text.split("\n"):
        match = CANONICAL_LINE.match(line.strip())
        if match:
            found.append((match.group("script"), {"same-inputs": match.group("same"), "unrelated-change": match.group("unrelated"),
                                                  "unit-change": match.group("unit")}))
    return found


def canonical_verdict(text, script, also=()):
    """这份脚本的规范输出判过没过：交 (过没过, 一句为什么)。also 是另认的脚本名（登记表那一行的路径：门禁那一行指到入口脚本的自证、
    自证另放一份的那一行，与第三列里点名的判法脚本）。"""
    found = canonical_lines(text)
    lines = [values for name, values in found if name == script] or [values for name, values in found if name in also]
    if not lines:
        return False, f"输出里没有 {LINE_PREFIX} script={script} 那一行（自证里没有 {CELL_NAME} 那一格，或那一格没跑到）"
    values = lines[-1]
    red = [name for name, green in QUESTIONS if values.get(name) != green]
    if red:
        return False, f"{LINE_PREFIX} 那一行红在：{'、'.join(f'{name}={values.get(name)}' for name in red)}"
    return True, "三问都过"


# ── 二、复用哨兵 ──

def sentinel_count(environment=None):
    text = (os.environ if environment is None else environment).get(SENTINEL_COUNT_VARIABLE, "")
    return int(text) if text.isdigit() and int(text) >= 1 else DEFAULT_SENTINEL_COUNT


def sentinel_seed(environment=None):
    text = (os.environ if environment is None else environment).get(SENTINEL_SEED_VARIABLE, "")
    if re.fullmatch(r"-?[0-9]+", text):
        return int(text)
    return random.SystemRandom().randrange(1 << 32)


class ReuseSentinel:
    """一趟真跑里的复用哨兵：plan 抽、forced 问、observe 记、finish 判。"""

    def __init__(self, script, label="", count=None, seed=None, environment=None):
        """environment：种子与条数从这一份环境取（调用方自己带一份环境跑的，例如 run_stage(root, environment)）；不给取 os.environ。"""
        self.script = script
        self.label = label
        self.count = count if count is not None else sentinel_count(environment)
        self.seed = seed if seed is not None else sentinel_seed(environment)
        self.state = "unplanned"
        self.off_reason = ""
        self.candidates = {}
        self.sentinels = []
        self.forced_item = None
        self.observed = {}

    def title(self):
        return f"复用哨兵（{self.label}）" if self.label else "复用哨兵"

    def plan(self, candidates, reuse=True, off_reason="这一趟不读判定库", printer=print):
        """candidates：{项: 库里那一条的判定}，只放此刻键相同、有判定的项。reuse 为假（全量、--no-reuse、没给库）不抽。"""
        if not reuse:
            self.state, self.off_reason = "off", off_reason
            printer(f"  · {self.title()}：{off_reason}，不抽哨兵")  # gate-lint:detail
            return
        self.state = "planned"
        self.candidates = dict(candidates)
        generator = random.Random(self.seed)
        names = sorted(self.candidates)
        self.sentinels = sorted(generator.sample(names, min(self.count, len(names))))
        rest = [name for name in names if name not in self.sentinels]
        self.forced_item = generator.choice(rest) if rest else None
        shown = "、".join(self.sentinels) if self.sentinels else "一条都没有"
        forced = self.forced_item if self.forced_item else "没有可抽的（键相同的都当了哨兵或一条都没有）"
        printer(f"  · {self.title()}：种子 {self.seed}（{SENTINEL_SEED_VARIABLE}={self.seed} 复现）；键相同、有判定的 {len(names)} 条里抽哨兵 "
                f"{len(self.sentinels)} 条：{shown}；强制现判 1 条：{forced}")  # gate-lint:detail
        if len(self.sentinels) < self.count:
            printer(f"  · {self.title()}：哨兵不足 {self.count} 条（键相同、有判定的只有 {len(names)} 条），这一趟不因此判红")  # gate-lint:detail

    def forced(self, item):
        return self.state == "planned" and item == self.forced_item

    def observe(self, item, state, verdict):
        self.observed[item] = (state, verdict)

    def finish(self, printer=print):
        """交 [(红句, 出路)]；同时打判定（✗ 行由调用方随红句打）。"""
        problems = []
        if self.state != "planned":
            return problems
        for item in self.sentinels:
            if item not in self.observed:
                printer(f"  · {self.title()}：哨兵 {item} 这一趟没报结局（没跑到），不判")  # gate-lint:detail
                continue
            state, _verdict = self.observed[item]
            if state != REUSED and "rejudged-sentinel-passes" not in BREAK:
                problems.append((f"复用失灵：{item} 键相同却现判了（{self.title()}，种子 {self.seed}：这一趟报的是「{state}」）",
                                 "这一项此刻的键与库里那一条判绿的记录相同：查这一趟为什么没读到它（复用开关、库目录、读库与报结局那一段），"
                                 f"修好后带 {SENTINEL_SEED_VARIABLE}={self.seed} 重跑同一条命令复现、看它转成复用"))
        if self.forced_item is not None:
            stored = self.candidates.get(self.forced_item)
            if self.forced_item not in self.observed:
                printer(f"  · {self.title()}：强制现判的 {self.forced_item} 这一趟没报结局（没跑到），不判")  # gate-lint:detail
            else:
                state, verdict = self.observed[self.forced_item]
                if state == REUSED:
                    problems.append((f"复用哨兵（{self.label or self.script}）：抽中强制现判的 {self.forced_item} 这一趟还是复用了，强制现判没起作用（种子 {self.seed}）",
                                     "读库之前先问 forced（shell 是 sentinel-forced），抽中的那一条跳过读库、照现判跑"))
                elif verdict != stored and "forced-mismatch-passes" not in BREAK:
                    problems.append((f"判定不稳：{self.forced_item}（库里那一条是「{stored}」，这一趟强制现判是「{verdict}」；{self.title()}，种子 {self.seed}）",
                                     "同一把键两次判出不同的结论：判法依赖了键之外的输入（时序、随机源、环境、外部状态）；把那样输入加进键或去掉依赖，"
                                     "再删掉这一项在库里的记录重判"))
        if not problems:
            forced = (f"；强制现判 {self.forced_item} 与库里同为「{self.candidates.get(self.forced_item)}」"
                      if self.forced_item is not None and self.forced_item in self.observed else "")
            reused = sum(1 for item in self.sentinels if item in self.observed)
            printer(f"  · {self.title()}：哨兵 {reused} 条都复用{forced}（种子 {self.seed}）")  # gate-lint:detail
        return problems

    def to_json(self):
        return {"script": self.script, "label": self.label, "count": self.count, "seed": self.seed, "state": self.state, "off_reason": self.off_reason,
                "candidates": self.candidates, "sentinels": self.sentinels, "forced": self.forced_item}

    @classmethod
    def from_json(cls, data):
        sentinel = cls(data["script"], data.get("label", ""), count=data["count"], seed=data["seed"])
        sentinel.state, sentinel.off_reason = data["state"], data.get("off_reason", "")
        sentinel.candidates, sentinel.sentinels, sentinel.forced_item = data["candidates"], data["sentinels"], data["forced"]
        return sentinel


def verdict_library():
    """scripts/crash/lib_verdict_library.py，第一次用到时才读进来（这里只调它的 key_of 与存放的写法，不另算一份键）。"""
    if not _VERDICT_LIBRARY:
        # 调用方已经 import 过的那一份优先：组成里的 UnitKey 是那一份的类，另读一份进来 isinstance 认不出、算出的键就对不上
        for module in list(sys.modules.values()):
            if os.path.realpath(getattr(module, "__file__", None) or "") == VERDICT_LIBRARY_PATH:
                _VERDICT_LIBRARY.append(module)
                return module
        specification = importlib.util.spec_from_file_location("lib_verdict_library_for_reuse_checks", VERDICT_LIBRARY_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _VERDICT_LIBRARY.append(module)
    return _VERDICT_LIBRARY[0]


def verdict_library_candidate(directory, kind, components, judging_units):
    """VerdictLibrary 形态的库（<目录>/<种类>/keys/<键>.json）里此刻这把键有判绿的记录：交 True。
    与复用判法（VerdictLibrary.lookup）分开查：只看键文件在不在、记的是不是绿、键对不对得上；拒算的不是候选。"""
    if not directory:
        return False
    library = verdict_library()
    if library.refused_component(components, judging_units) is not None:
        return False
    key = library.key_of(components, judging_units)
    record = library.read_json(os.path.join(directory, kind, "keys", key + ".json"))
    return isinstance(record, dict) and record.get("verdict") == "green" and record.get("key") == key


# ── 三、上次红的清单 ──

def last_runs_directory():
    return os.environ.get(LAST_RUNS_VARIABLE) or os.path.expanduser(DEFAULT_LAST_RUNS_DIRECTORY)


def last_run_path(script_key):
    return os.path.join(last_runs_directory(), script_key.replace("/", "__") + ".tsv")


def read_last_run(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return [line.strip() for line in handle if line.strip() and not line.strip().startswith("#")]
    except (OSError, UnicodeDecodeError):
        return []


def write_atomically(path, text):
    directory = os.path.dirname(path)
    os.makedirs(directory, exist_ok=True)
    if "last-run-in-place" in BREAK:
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        return
    handle_number, temporary = tempfile.mkstemp(prefix="." + os.path.basename(path) + ".", dir=directory)
    try:
        with os.fdopen(handle_number, "w", encoding="utf-8") as handle:
            handle.write(text)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


def write_last_run(script_key, red_names, ran_names=None):
    """写上次红的清单，交路径。ran_names 给 None 是整轮；给了是这一趟只跑到的那几项（上一份里别的项照留）。"""
    path = last_run_path(script_key)
    red = list(dict.fromkeys(name for name in red_names if name))
    if ran_names is not None and "last-run-partial-overwrites" not in BREAK:
        ran = set(ran_names) | set(red)
        kept = [name for name in read_last_run(path) if name not in ran]
        red = kept + red
    write_atomically(path, "".join(name + "\n" for name in red))
    return path


TARGET_NOT_RUN_EXITS = (2, 77, 130, 143)


def write_target_last_run(script_key, target, exit_code, test_filter="", target_file=""):
    """跑一个测试目标的驱动（crash-amplification-item.sh、crash-amplification-two-hosts.sh）跑完调，写法照文件头「三」「测试目标」那一段；交 (路径或 None, 为什么没写)。"""
    if exit_code in TARGET_NOT_RUN_EXITS:
        return None, f"退 {exit_code}（用法错、本次未跑或被停）：清单照留"
    if not os.environ.get(LAST_RUNS_VARIABLE) and target_file and not os.path.isfile(target_file):
        return None, f"测试目标的源文件 {target_file} 不在（自证造的假目标），没显式设 {LAST_RUNS_VARIABLE}：不写共用的清单"
    name = f"{target}::{test_filter}" if test_filter else target
    ran = [name] if test_filter else [name] + [item for item in read_last_run(last_run_path(script_key)) if item.startswith(target + "::")]
    return write_last_run(script_key, [] if exit_code == 0 else [name], ran), ""


# ── 四、门禁 code-tooling 的 reuse-selftests 格 ──

def read_registry(root):
    """交 (行 [(行号, 路径, 自证命令, 判定库, 格名)], 射程 [glob], [格式问题])。"""
    rows, scans, problems = [], [], []
    path = os.path.join(root, REGISTRY)
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except OSError as error:
        return rows, scans, [f"读不了 {REGISTRY}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        if line.startswith("#"):
            found = re.match(r"^#\s*scan:\s*(.+)$", line)
            if found:
                scans += found.group(1).split()
            continue
        fields = line.split("\t")
        if fields and fields[-1].startswith("#"):
            fields = fields[:-1]
        if len(fields) != 4 or not all(field.strip() for field in fields):
            problems.append(f"{REGISTRY} 第 {number} 行不是四列（路径、自证命令、判定库放哪、复用三问格名，制表符隔开，说明另起一列 # 开头）")
            continue
        rows.append((number, *[field.strip() for field in fields]))
    if not scans:
        problems.append(f"{REGISTRY} 里没有 # scan: 那一行，现扫的射程是空的")
    return rows, scans, problems


def scanned_claims(root, scans):
    """射程里注释行写了「判定库」或「复用」的脚本（相对根的路径，排好序）。"""
    found = set()
    for pattern in scans:
        for path in glob.glob(os.path.join(root, pattern)):
            if not os.path.isfile(path):
                continue
            try:
                with open(path, encoding="utf-8", errors="replace") as handle:
                    if any(CLAIM_LINE.match(line) for line in handle):
                        found.add(os.path.relpath(path, root))
            except OSError:
                continue
    return sorted(found)


def selftest_tiers(root):
    tiers = {}
    try:
        with open(os.path.join(root, SELFTEST_REGISTRY), encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("#") or "\t" not in line:
                    continue
                fields = line.rstrip("\n").split("\t")
                if len(fields) >= 2:
                    tiers[fields[0]] = fields[1]
    except OSError:
        pass
    return tiers


def command_script(command):
    """自证命令里被测的那份脚本（头一个以 .py 或 .sh 收尾、不以 - 起头的词），与命令开头设的变量名。"""
    try:
        words = shlex.split(command)
    except ValueError:
        return None, set()
    assigned = set()
    for word in words:
        if re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", word):
            assigned.add(word.split("=", 1)[0])
            continue
        if not word.startswith("-") and re.search(r"\.(py|sh)$", word):
            return os.path.normpath(word), assigned
    return None, assigned


def command_is_light(tier, assigned):
    if tier == "light" or tier in ("samples-light",) or (tier or "").startswith("samples-light-marker:"):
        return True
    if (tier or "").startswith("heavy-if-set:"):
        return tier.split(":", 1)[1] not in assigned
    return False


def selftest_outputs_directory():
    return os.environ.get(SELFTEST_OUTPUTS_VARIABLE) or os.path.expanduser(DEFAULT_SELFTEST_OUTPUTS_DIRECTORY)


def selftest_output_path(script_key):
    return os.path.join(selftest_outputs_directory(), script_key.replace("/", "__") + ".log")


def file_sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def stored_output(root, script_key):
    """上一次输出：交 (正文或 None, 为什么没有)。头一行「# selftest-output script=<路径> sha256=<那时的 sha256> …」，脚本变了不认。"""
    path = selftest_output_path(script_key)
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None, f"没有上一次输出（{path}；主 agent 提交前用 python3 scripts/gate/lib_reuse_checks.py keep-output . {script_key} -- <自证命令> 跑一次）"
    found = re.match(r"^# selftest-output script=(\S+) sha256=([0-9a-f]{64})", text)
    if not found or found.group(1) != script_key:
        return None, f"{path} 头一行不是 keep-output 写的那种，不认"
    current = file_sha256(os.path.join(root, script_key))
    if found.group(2) != current:
        return None, f"{path} 是 {script_key} 旧版本（sha256 {found.group(2)[:12]}…，此刻 {current[:12]}…）的输出，脚本变了之后还没重跑"
    return text, ""


def run_command(root, command, timeout):
    try:
        completed = subprocess.run(["bash", "-c", command], cwd=root, capture_output=True, text=True, timeout=timeout,
                                   stdin=subprocess.DEVNULL, errors="replace")
        return completed.returncode, completed.stdout + completed.stderr
    except subprocess.TimeoutExpired as error:
        output = (error.stdout or "") + (error.stderr or "") if isinstance(error.stdout, str) else ""
        return None, output + f"\n（{timeout} 秒没跑完）"


def audit(root, jobs, timeout):
    root = os.path.abspath(root)
    rows, scans, problems = read_registry(root)
    red = [(problem, f"照 {REGISTRY} 表头的写法改这一行") for problem in problems]
    tiers = selftest_tiers(root)
    by_path = {}
    for number, path, command, library, cell in rows:
        if path in by_path:
            red.append((f"{REGISTRY} 第 {number} 行：{path} 已在第 {by_path[path][0]} 行登记过", "一份脚本只登记一行；两行说的是同一份就删掉一行"))
            continue
        by_path[path] = (number, command, library, cell)
    claims = [] if "audit-scan-skipped" in BREAK else scanned_claims(root, scans)
    for path in claims:
        if path not in by_path:
            red.append((f"漏登记：{path} 的注释写了「判定库」或「复用」，{REGISTRY} 里没有它",
                        f"真有判定库的：自证里加 {CELL_NAME} 那一格（scripts/gate/lib_reuse_checks.py 的 report_three_questions），再登记一行；"
                        f"只是提到别人的复用：登记一行、第三列写「{NOT_COUNTED_PREFIX}<为什么>」"))
        elif by_path[path][2].startswith(NO_LIBRARY_PREFIX):
            red.append((f"{path} 的注释写了「判定库」或「复用」，登记成了「{NO_LIBRARY_PREFIX}」：声称复用而没有 {CELL_NAME} 那一格的不许登记成无判定库",
                        f"真有判定库就在它的自证里加 {CELL_NAME} 那一格、第三列写判定库放哪；只是提到别人的复用写「{NOT_COUNTED_PREFIX}<为什么>」"))
    passed, not_run, not_counted, to_run = [], [], [], []
    leaked_library = ""  # 这一圈走到的最后一行的第三列：只给弄坏开关 audit-leaked-library 用（照旧取漏出来的变量）
    for path, (number, command, library, cell) in by_path.items():
        leaked_library = library
        if not os.path.exists(os.path.join(root, path)):
            red.append((f"{REGISTRY} 第 {number} 行指到不存在的路径：{path}", "脚本搬了家就改这一行，删了就删这一行"))
            continue
        if library.startswith(NO_LIBRARY_PREFIX):
            not_run.append(f"{path}（本次未跑：没有判定库，{library[len(NO_LIBRARY_PREFIX):]}）")
            continue
        if library.startswith(NOT_COUNTED_PREFIX):
            not_counted.append(path)
            continue
        if cell != CELL_NAME:
            red.append((f"{REGISTRY} 第 {number} 行：{path} 的复用三问格名写的是「{cell}」，全仓统一是 {CELL_NAME}", f"第四列写 {CELL_NAME}，自证里那一格也叫这个名字"))
            continue
        script, assigned = command_script(command)
        if script is None or not os.path.isfile(os.path.join(root, script)):
            red.append((f"{REGISTRY} 第 {number} 行：自证命令「{command}」里认不出被测脚本，或它不在", "命令写成 python3 <脚本> --selftest 或 bash <脚本> --selftest 这种样子"))
            continue
        to_run.append((path, command, script, command_is_light(tiers.get(script), assigned), tiers.get(script), library))
    light_commands = sorted({command for _path, command, _script, light, _tier, _library in to_run if light})
    outputs = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(jobs, len(light_commands) or 1))) as pool:
        futures = {command: pool.submit(run_command, root, command, timeout) for command in light_commands}
        for command, future in futures.items():
            outputs[command] = future.result()
    if len(outputs) != len(light_commands):
        red.append((f"现跑的自证派出 {len(light_commands)} 条、收回 {len(outputs)} 条", "整格重跑一次；收不全的一趟不算判过"))
    for path, command, script, light, tier, row_library in sorted(to_run):
        if light:
            exit_code, text = outputs.get(command, (None, ""))
            how = f"现跑「{command}」退 {exit_code}"
        else:
            text, why = stored_output(root, script)
            if text is None:
                not_run.append(f"{path}（本次未跑：自证登记为 {tier or '没登记'}，{why}）")
                continue
            exit_code, how = 0, f"读 {script} 的上一次输出（自证登记为 {tier or '没登记'}）"
        if "audit-line-unchecked" in BREAK:
            verdict, why = exit_code == 0, "没看规范输出那一行"
        else:
            named = leaked_library if "audit-leaked-library" in BREAK else row_library  # 每一行认它自己第三列点名的判法脚本
            verdict, why = canonical_verdict(text, script, also=(path, *re.findall(r"(?:\.claude|scripts)/[\w./-]+\.(?:sh|py)", named)))
        if verdict:
            passed.append(f"{path}（{how}）")
            print(f"  · {path}：{CELL_NAME} 过，{how}")  # gate-lint:detail
        else:
            tail = " | ".join(line for line in text.strip().split("\n")[-4:] if line.strip())[-500:]
            red.append((f"{path}：{why}（{how}；输出末尾：{tail}）",
                        f"在 {script} 的自证里补 {CELL_NAME} 那一格（同一份输入两趟、闭包外改动、闭包内改一个单元），照 scripts/gate/lib_reuse_checks.py 的 report_three_questions 打规范输出；"
                        f"那一行写 red 的就照它点名的那一问修复用"))
    for path in not_run:
        print(f"  · {path}")  # gate-lint:detail
    if not_counted:
        print(f"  · 不算（只提到别人的复用）{len(not_counted)} 份：{'、'.join(sorted(not_counted))}")  # gate-lint:detail
    if red:
        for message, howto in red:
            print(f"  ✗ {message}")
            print(f"     → 怎么办：{howto}")
        print(f"  ✗ reuse-selftests：登记 {len(rows)} 行里 {len(red)} 处红（上面逐条列着）")
        print(f"     → 怎么办：逐条照上面的出路改；判据写在 scripts/gate/lib_reuse_checks.py 文件头「四」")
        return EXIT_RED
    if not passed:
        print(f"  ! reuse-selftests：登记 {len(rows)} 行，一份都没判（本次未跑 {len(not_run)} 份、不算 {len(not_counted)} 份），这一格本次未跑")
        return EXIT_NOT_RUN
    not_run_text = "；".join(not_run) if not_run else "无"
    print(f"  ✓ reuse-selftests：登记 {len(rows)} 行，复用三问过 {len(passed)} 份；本次未跑 {len(not_run)} 份：{not_run_text}；不算 {len(not_counted)} 份；"
          f"射程里声称复用的 {len(claims)} 份都登记了")
    return EXIT_GREEN


def keep_output(root, script_key, command):
    root = os.path.abspath(root)
    if not os.path.isfile(os.path.join(root, script_key)):
        print(f"  ✗ keep-output：{script_key} 不在 {root} 底下")
        print("     → 怎么办：脚本路径写相对仓根的，例如 scripts/crash/crash-point-replay-shard-run.sh")
        return EXIT_USAGE
    digest = file_sha256(os.path.join(root, script_key))
    completed = subprocess.run(command, cwd=root, capture_output=True, text=True, errors="replace")
    text = completed.stdout + completed.stderr
    sys.stdout.write(text)
    write_atomically(selftest_output_path(script_key),
                     f"# selftest-output script={script_key} sha256={digest} exit={completed.returncode} command={shlex.join(command)}\n{text}")
    return completed.returncode


def run_sample(stage, name):
    """门禁样本档里的一份（登记成 samples-light、samples-light-marker 的阶段）：拷进临时目录、有 setup.sh 先跑它，再在那里起这一道阶段，输出照原样打，退出码照交。
    门禁 code-tooling 的 reuse-selftests 格经它起样本档里的复用三问（只判那一份样本的规范输出，不比 expect）。"""
    import shutil
    key = f".claude/gate.d/{stage}"
    tier = selftest_tiers(REPOSITORY_ROOT).get(key, "")
    fixture = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "fixtures", stage, name)
    if not (tier == "samples-light" or tier.startswith("samples-light-marker:")):
        print(f"  ✗ sample：{key} 的样本档在 {SELFTEST_REGISTRY} 里登记的是「{tier or 没登记}」，不是轻的，这里不起")
        print("     → 怎么办：登记成 samples-heavy 的样本档归提交前；轻的照 selftest-registry.tsv 表尾的写法登记")
        return EXIT_USAGE
    if not os.path.isdir(fixture):
        print(f"  ✗ sample：没有样本 {os.path.relpath(fixture, REPOSITORY_ROOT)}")
        print(f"     → 怎么办：样本名照 ls .claude/gate.d/fixtures/{stage}/ 写")
        return EXIT_USAGE
    environment = {name_: value for name_, value in os.environ.items() if name_ not in ("GATE_STAGED_FROM", "GATE_NOT_RUN_FILE")}
    with tempfile.TemporaryDirectory(prefix="reuse-checks-sample-") as work:
        sample = os.path.join(work, name)
        shutil.copytree(fixture, sample, symlinks=True)
        if os.path.isfile(os.path.join(sample, "setup.sh")):
            setup = subprocess.run(["bash", "setup.sh"], cwd=sample, capture_output=True, text=True, errors="replace", env=environment, stdin=subprocess.DEVNULL)
            if setup.returncode != 0:
                print(f"  ✗ sample：样本 {name} 的 setup.sh 退 {setup.returncode}：{(setup.stdout + setup.stderr).strip()[-400:]}")
                print("     → 怎么办：照它的报错修样本的 setup.sh")
                return EXIT_RED
        completed = subprocess.run(["bash", os.path.join(REPOSITORY_ROOT, key), sample], cwd=sample, capture_output=True, text=True, errors="replace",
                                   env=environment, stdin=subprocess.DEVNULL)
        sys.stdout.write(completed.stdout + completed.stderr)
        return completed.returncode


# ── 命令行 ──

def read_states_file(path, from_output):
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if from_output:
        return states_from_output(text)
    states = {}
    for line in text.split("\n"):
        if line.strip() and not line.startswith("#"):
            item, _separator, state = line.partition("\t")
            states[item] = state.strip()
    return states


def option_value(arguments, name, default=None):
    if name in arguments:
        position = arguments.index(name)
        if position + 1 < len(arguments):
            return arguments[position + 1]
        raise ValueError(f"{name} 后面缺值")
    return default


def load_sentinel(path):
    with open(path, encoding="utf-8") as handle:
        return ReuseSentinel.from_json(json.load(handle))


def observed_lines(path):
    observed = {}
    try:
        with open(path + ".observed", encoding="utf-8") as handle:
            for line in handle:
                fields = line.rstrip("\n").split("\t")
                if len(fields) == 3:
                    observed[fields[0]] = (fields[1], fields[2])
    except OSError:
        pass
    return observed


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    if not arguments:
        print("  ✗ 用法：lib_reuse_checks.py three-questions|sentinel-plan|sentinel-forced|sentinel-observe|sentinel-finish|verdict-library-candidate|last-run|audit|keep-output … | --selftest")
        print("     → 怎么办：看文件头「命令行」一段")
        return EXIT_USAGE
    command, rest = arguments[0], arguments[1:]
    try:
        if command == "three-questions":
            files = {name: option_value(rest, f"--{name}") for name in ("first", "second", "unrelated", "unit-change")}
            script, affected = option_value(rest, "--script"), option_value(rest, "--affected", "")
            if not script or any(value is None for value in files.values()):
                raise ValueError("要 --script 与 --first、--second、--unrelated、--unit-change 四份文件")
            runs = {name: read_states_file(path, "--from-output" in rest) for name, path in files.items()}
            passed, _problems = report_three_questions(script, runs["first"], runs["second"], runs["unrelated"], runs["unit-change"],
                                                       [item for item in affected.split(",") if item], sentinels=int(option_value(rest, "--sentinels", "1")))
            return EXIT_GREEN if passed else EXIT_RED
        if command == "sentinel-plan":
            if not rest:
                raise ValueError("要 <状态文件>")
            state_path = rest[0]
            sentinel = ReuseSentinel(option_value(rest, "--script", "?"), option_value(rest, "--label", ""))
            off = option_value(rest, "--off")
            candidates = {}
            if off is None:
                candidates_path = option_value(rest, "--candidates")
                if candidates_path is None:
                    raise ValueError("要 --candidates <文件> 或 --off <为什么>")
                with open(candidates_path, encoding="utf-8") as handle:
                    for line in handle:
                        item, separator, verdict = line.rstrip("\n").partition("\t")
                        if item and separator:
                            candidates[item] = verdict
            sentinel.plan(candidates, reuse=off is None, off_reason=off or "")
            with open(state_path, "w", encoding="utf-8") as handle:
                json.dump(sentinel.to_json(), handle, ensure_ascii=False)
            open(state_path + ".observed", "w", encoding="utf-8").close()
            return EXIT_GREEN
        if command == "sentinel-forced":
            if len(rest) != 2:
                raise ValueError("要 <状态文件> <项>")
            return EXIT_GREEN if load_sentinel(rest[0]).forced(rest[1]) else EXIT_RED
        if command == "sentinel-observe":
            if len(rest) != 4:
                raise ValueError("要 <状态文件> <项> <结局> <判定>")
            with open(rest[0] + ".observed", "a", encoding="utf-8") as handle:
                handle.write("\t".join(field.replace("\t", " ").replace("\n", " ") for field in rest[1:]) + "\n")
            return EXIT_GREEN
        if command == "sentinel-observe-many":
            if len(rest) != 2:
                raise ValueError("要 <状态文件> <结局文件>")
            with open(rest[1], encoding="utf-8") as source, open(rest[0] + ".observed", "a", encoding="utf-8") as handle:
                for line in source:
                    fields = line.rstrip("\n").split("\t")
                    if len(fields) == 3 and fields[0]:
                        handle.write("\t".join(fields) + "\n")
            return EXIT_GREEN
        if command == "sentinel-finish":
            if len(rest) != 1:
                raise ValueError("要 <状态文件>")
            sentinel = load_sentinel(rest[0])
            sentinel.observed = observed_lines(rest[0])
            problems = sentinel.finish()
            for message, howto in problems:
                print(f"  ✗ {message}")
                print(f"     → 怎么办：{howto}")
            return EXIT_RED if problems else EXIT_GREEN
        if command == "verdict-library-candidate":
            if len(rest) != 3:
                raise ValueError("要 <目录> <种类> <组成表>")
            components, judging_units = verdict_library().read_components(rest[2])
            return EXIT_GREEN if verdict_library_candidate(rest[0], rest[1], components, judging_units) else EXIT_RED
        if command == "last-run":
            if len(rest) < 2:
                raise ValueError("要 <脚本相对仓根的路径> <红的清单文件或 ->")
            red_text = sys.stdin.read() if rest[1] == "-" else open(rest[1], encoding="utf-8").read()
            ran_path = option_value(rest, "--ran")
            ran = None if ran_path is None else [line.strip() for line in open(ran_path, encoding="utf-8") if line.strip()]
            path = write_last_run(rest[0], [line.strip() for line in red_text.split("\n") if line.strip()], ran)
            print(f"  · 上次红的清单写进 {path}")  # gate-lint:detail
            return EXIT_GREEN
        if command == "last-run-target":
            positional = [argument for index, argument in enumerate(rest)
                          if not argument.startswith("--") and (index == 0 or rest[index - 1] not in ("--test-filter", "--target-file"))]
            if len(positional) != 3 or not re.fullmatch(r"[0-9]+", positional[2]):
                raise ValueError("要 <脚本相对仓根的路径> <测试目标> <退出码>（退出码是非负整数）")
            path, why = write_target_last_run(positional[0], positional[1], int(positional[2]), option_value(rest, "--test-filter", ""),
                                              option_value(rest, "--target-file", ""))
            print(f"  · 上次红的清单写进 {path}" if path else f"  · 上次红的清单没写：{why}")  # gate-lint:detail
            return EXIT_GREEN
        if command == "audit":
            if not rest or rest[0].startswith("-"):
                raise ValueError("要 <根>")
            jobs = option_value(rest, "--jobs")
            if jobs is None or not jobs.isdigit() or int(jobs) < 1:
                raise ValueError("要 --jobs <正整数>（门禁 code-tooling 取每台默认重活线程数给）")
            timeout = int(option_value(rest, "--timeout", str(DEFAULT_AUDIT_TIMEOUT_SECONDS)))
            return audit(rest[0], int(jobs), timeout)
        if command == "sample":
            if len(rest) != 2:
                raise ValueError("要 <阶段文件名> <样本名>")
            return run_sample(rest[0], rest[1])
        if command == "keep-output":
            if len(rest) < 4 or "--" not in rest:
                raise ValueError("要 <根> <脚本相对仓根的路径> -- <命令…>")
            separator = rest.index("--")
            return keep_output(rest[0], rest[1], rest[separator + 1:])
    except (ValueError, OSError, json.JSONDecodeError, KeyError) as error:
        print(f"  ✗ {command}：{error}")
        print("     → 怎么办：看文件头「命令行」一段的写法")
        return EXIT_USAGE
    print(f"  ✗ 认不出的子命令 {command}")
    print("     → 怎么办：看文件头「命令行」一段")
    return EXIT_USAGE


# ── 自证 ──

def selftest():
    cases, wrong = 0, []

    def cell(label, passed, detail):
        nonlocal cases
        cases += 1
        if not passed:
            wrong.append(label)
            print(f"    {label} 这一格判错：{detail}")  # gate-lint:detail

    quiet = lambda *_arguments, **_keywords: None   # noqa: E731
    items = ("甲", "乙", "丙")
    fresh = {item: JUDGED for item in items}
    reused = {item: REUSED for item in items}
    unit_only = {"甲": JUDGED, "乙": REUSED, "丙": REUSED}
    passed, line, _problems = three_questions("scripts/x.sh", fresh, reused, reused, unit_only, ["甲"])
    cell("三问：三问都过", passed and line == "REUSE_SELFTEST script=scripts/x.sh same-inputs=reused unrelated-change=reused unit-change=rejudged-only-affected items=3 affected=1",
         line)
    passed, line, problems = three_questions("scripts/x.sh", fresh, dict(reused, 乙=JUDGED), reused, unit_only, ["甲"])
    cell("三问：第二趟多现判一条", not passed and "same-inputs=red" in line and "unrelated-change=reused" in line, f"{line}；{problems}")
    shorter = {"甲": REUSED, "乙": REUSED}
    passed, line, problems = three_questions("scripts/x.sh", fresh, shorter, reused, unit_only, ["甲"])
    cell("三问：第二趟少复用一条", not passed and "same-inputs=red" in line and any("少复用：丙" in problem for problem in problems), f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, reused, dict(reused, 丙=JUDGED), unit_only, ["甲"])
    cell("三问：闭包外改动现判一条", not passed and "unrelated-change=red" in line and "same-inputs=reused" in line, f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, reused, reused, dict(unit_only, 乙=JUDGED), ["甲"])
    cell("三问：闭包内改动多现判一条", not passed and "unit-change=red" in line and any("多现判 1 条：乙" in problem for problem in problems), f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, reused, reused, reused, ["甲"])
    cell("三问：闭包内改动少现判一条", not passed and "unit-change=red" in line and any("甲 没现判" in problem for problem in problems), f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", reused, reused, reused, unit_only, ["甲"])
    cell("三问：头一趟没现判问不出", not passed and "same-inputs=red" in line, f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, reused, reused, fresh, ["甲", "乙", "丙"])
    cell("三问：affected 是全部项问不出", not passed and "unit-change=red" in line, f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, dict(reused, 乙=FORCED), dict(reused, 丙=FORCED), dict(unit_only, 乙=FORCED), ["甲"])
    cell("三问：强制现判按复用算", passed, f"{line}；{problems}")
    passed, line, problems = three_questions("scripts/x.sh", fresh, dict(reused, 乙=FORCED, 丙=FORCED), reused, unit_only, ["甲"])
    cell("三问：一趟两条强制现判判红", not passed and "same-inputs=red" in line, f"{line}；{problems}")
    output = "  · 甲：绿，现判（无记录）\n  · 乙：红，复用（10-15 判绿）\n  · 丙：绿，强制现判（复用哨兵）\n别的行\n"
    cell("三问：从输出取结局", states_from_output(output) == {"甲": JUDGED, "乙": REUSED, "丙": FORCED}, repr(states_from_output(output)))
    good = "x\nREUSE_SELFTEST script=scripts/x.sh same-inputs=reused unrelated-change=reused unit-change=rejudged-only-affected items=3 affected=1\n"
    bad_line = "REUSE_SELFTEST script=scripts/x.sh same-inputs=reused unrelated-change=red unit-change=rejudged-only-affected items=3 affected=1\n"
    cell("规范输出：三问都过认得", canonical_verdict(good, "scripts/x.sh")[0], canonical_verdict(good, "scripts/x.sh")[1])
    cell("规范输出：有一问 red 判红", not canonical_verdict(bad_line, "scripts/x.sh")[0] and "unrelated-change=red" in canonical_verdict(bad_line, "scripts/x.sh")[1],
         canonical_verdict(bad_line, "scripts/x.sh")[1])
    cell("规范输出：别的脚本那一行不算", not canonical_verdict(good, "scripts/y.sh")[0], canonical_verdict(good, "scripts/y.sh")[1])

    # 哨兵
    candidates = {name: "绿" for name in ("a", "b", "c", "d", "e")}
    lines = []
    first, second = ReuseSentinel("scripts/x.sh", "格", count=3, seed=7), ReuseSentinel("scripts/x.sh", "格", count=3, seed=7)
    first.plan(candidates, printer=lines.append)
    second.plan(candidates, printer=quiet)
    cell("哨兵：同一个种子抽同几条、种子打进日志", first.sentinels == second.sentinels and first.forced_item == second.forced_item and len(first.sentinels) == 3
         and first.forced_item not in first.sentinels and first.forced_item is not None and any("种子 7" in text for text in lines), f"{first.to_json()}；{lines}")

    def finished(sentinel, observed):
        for item, (state, verdict) in observed.items():
            sentinel.observe(item, state, verdict)
        return sentinel.finish(printer=quiet)

    green_run = {item: (REUSED, "绿") for item in candidates}
    green_run[first.forced_item] = (FORCED, "绿")
    planned = lambda: (lambda sentinel: (sentinel.plan(candidates, printer=quiet), sentinel)[1])(ReuseSentinel("scripts/x.sh", "格", count=3, seed=7))  # noqa: E731
    cell("哨兵：都复用、强制现判同判定不红", finished(planned(), green_run) == [], "红了")
    rejudged = dict(green_run)
    rejudged[first.sentinels[0]] = (JUDGED, "绿")
    problems = finished(planned(), rejudged)
    cell("哨兵：被现判判红", len(problems) == 1 and problems[0][0].startswith(f"复用失灵：{first.sentinels[0]} 键相同却现判了"), repr(problems))
    unstable = dict(green_run)
    unstable[first.forced_item] = (FORCED, "红")
    problems = finished(planned(), unstable)
    cell("哨兵：强制现判判定不同判红", len(problems) == 1 and problems[0][0].startswith(f"判定不稳：{first.forced_item}"), repr(problems))
    ignored = dict(green_run)
    ignored[first.forced_item] = (REUSED, "绿")
    problems = finished(planned(), ignored)
    cell("哨兵：强制现判没起作用判红", len(problems) == 1 and "强制现判没起作用" in problems[0][0], repr(problems))
    lines = []
    short = ReuseSentinel("scripts/x.sh", "格", count=3, seed=1)
    short.plan({"a": "绿"}, printer=lines.append)
    cell("哨兵：不足 K 条报出、不红", any("哨兵不足 3 条" in text for text in lines) and finished(short, {"a": (REUSED, "绿")}) == [], repr(lines))
    lines = []
    off = ReuseSentinel("scripts/x.sh", "格", count=3, seed=1)
    off.plan(candidates, reuse=False, off_reason="全量（不读判定库）", printer=lines.append)
    cell("哨兵：不读库的一趟不抽", off.state == "off" and not off.forced("a") and finished(off, {"a": (JUDGED, "绿")}) == [] and "不抽哨兵" in lines[0], repr(lines))

    with tempfile.TemporaryDirectory(prefix="lib-reuse-checks-selftest-") as work:
        command = [sys.executable, SCRIPT_PATH]
        environment = dict(os.environ, **{SENTINEL_SEED_VARIABLE: "11", SENTINEL_COUNT_VARIABLE: "2"})
        candidates_path, state_path = os.path.join(work, "candidates.tsv"), os.path.join(work, "sentinel.json")
        with open(candidates_path, "w", encoding="utf-8") as handle:
            handle.write("".join(f"{name}\t绿\n" for name in ("p", "q", "r", "s")))
        planned_run = subprocess.run(command + ["sentinel-plan", state_path, "--script", "scripts/x.sh", "--label", "命令行", "--candidates", candidates_path],
                                     capture_output=True, text=True, env=environment)
        state = json.load(open(state_path, encoding="utf-8")) if os.path.exists(state_path) else {}
        forced_item = state.get("forced")
        forced_exit = subprocess.run(command + ["sentinel-forced", state_path, forced_item or "?"], env=environment).returncode
        other = next((name for name in "pqrs" if name != forced_item), "p")
        other_exit = subprocess.run(command + ["sentinel-forced", state_path, other], env=environment).returncode
        for name in "pqrs":
            subprocess.run(command + ["sentinel-observe", state_path, name, FORCED if name == forced_item else REUSED, "绿"], env=environment)
        green_finish = subprocess.run(command + ["sentinel-finish", state_path], capture_output=True, text=True, env=environment)
        subprocess.run(command + ["sentinel-observe", state_path, state.get("sentinels", ["p"])[0], JUDGED, "绿"], env=environment)
        red_finish = subprocess.run(command + ["sentinel-finish", state_path], capture_output=True, text=True, env=environment)
        cell("哨兵：shell 命令行一个来回", planned_run.returncode == 0 and "种子 11" in planned_run.stdout and len(state.get("sentinels", [])) == 2
             and forced_exit == 0 and other_exit == 1 and green_finish.returncode == 0 and red_finish.returncode == 1 and "复用失灵" in red_finish.stdout,
             f"plan 退 {planned_run.returncode}「{planned_run.stdout.strip()}」，forced 退 {forced_exit}/{other_exit}，finish 退 {green_finish.returncode}/{red_finish.returncode}"
             f"「{red_finish.stdout.strip()[-200:]}」")

        # 一次记好几条结局（shell 那几道跑完逐行收的写法）：同一份计划，全复用、强制现判那一条同判定，不红；把一条哨兵写成现判，红
        many_state = os.path.join(work, "sentinel-many.json")
        subprocess.run(command + ["sentinel-plan", many_state, "--script", "scripts/x.sh", "--candidates", candidates_path], capture_output=True, text=True, env=environment)
        many_plan = json.load(open(many_state, encoding="utf-8"))
        outcomes_path = os.path.join(work, "outcomes.tsv")
        with open(outcomes_path, "w", encoding="utf-8") as handle:
            handle.write("".join(f"{name}\t{FORCED if name == many_plan['forced'] else REUSED}\t绿\n" for name in "pqrs"))
        observed_many = subprocess.run(command + ["sentinel-observe-many", many_state, outcomes_path], capture_output=True, text=True, env=environment)
        many_green = subprocess.run(command + ["sentinel-finish", many_state], capture_output=True, text=True, env=environment)
        with open(outcomes_path, "w", encoding="utf-8") as handle:
            handle.write(f"{many_plan['sentinels'][0]}\t{JUDGED}\t绿\n")
        subprocess.run(command + ["sentinel-observe-many", many_state, outcomes_path], capture_output=True, text=True, env=environment)
        many_red = subprocess.run(command + ["sentinel-finish", many_state], capture_output=True, text=True, env=environment)
        cell("哨兵：一次记好几条结局", observed_many.returncode == 0 and many_green.returncode == 0 and many_red.returncode == 1 and "复用失灵" in many_red.stdout,
             f"observe-many 退 {observed_many.returncode}，finish 退 {many_green.returncode}/{many_red.returncode}：{many_red.stdout.strip()[-200:]}")

        # VerdictLibrary 形态的候选：记绿之后键相同是候选，换一样输入不是
        library = verdict_library()
        store = library.VerdictLibrary(os.path.join(work, "verdicts"), "kind", "判法")
        components = [("这一项", library.digest_text("甲"))]
        before = verdict_library_candidate(os.path.join(work, "verdicts"), "kind", components, "判法")
        store.record("甲", components, "判绿", run="自证")
        after = verdict_library_candidate(os.path.join(work, "verdicts"), "kind", components, "判法")
        changed = verdict_library_candidate(os.path.join(work, "verdicts"), "kind", [("这一项", library.digest_text("乙"))], "判法")
        cell("哨兵：判定库形态的候选按键查", not before and after and not changed, f"记之前 {before}、记之后 {after}、换输入 {changed}")

        # 上次红的清单
        os.environ[LAST_RUNS_VARIABLE] = os.path.join(work, "last-runs")
        try:
            path = write_last_run("scripts/crash/x.sh", ["甲", "乙"])
            whole = read_last_run(path)
            inode = os.stat(path).st_ino
            write_last_run("scripts/crash/x.sh", [])
            emptied = open(path, encoding="utf-8").read()
            cell("上次红：整轮照这一趟写、全绿写空文件", whole == ["甲", "乙"] and emptied == ""
                 and os.path.basename(path) == "scripts__crash__x.sh.tsv", f"{whole}；全绿之后「{emptied}」；{path}")
            write_last_run("scripts/crash/x.sh", ["甲", "乙"])
            write_last_run("scripts/crash/x.sh", ["丙"], ran_names=["甲", "丙"])
            merged = read_last_run(path)
            cell("上次红：只跑几项照留没跑到的", merged == ["乙", "丙"], repr(merged))
            inode = os.stat(path).st_ino
            write_last_run("scripts/crash/x.sh", ["丁"])
            cell("上次红：原子换上", os.stat(path).st_ino != inode and read_last_run(path) == ["丁"]
                 and not [name for name in os.listdir(os.path.dirname(path)) if name.startswith(".")], f"inode {inode} → {os.stat(path).st_ino}")
            piped = subprocess.run(command + ["last-run", "scripts/crash/y.sh", "-"], input="戊\n", capture_output=True, text=True)
            cell("上次红：命令行从标准输入读", piped.returncode == 0 and read_last_run(last_run_path("scripts/crash/y.sh")) == ["戊"], piped.stdout)
            # 测试目标：整个目标红、点名用例绿（目标那一项照留）、点名用例红、本次未跑照留、整个目标绿（这个目标的项全换掉，别的目标的照留）
            steps = [("beta", "", 101), ("gamma", "", 101), ("beta", "one", 0), ("beta", "one", 101), ("beta", "", 77), ("beta", "", 0)]
            seen = []
            for target, test_filter, exit_code in steps:
                arguments = ["last-run-target", "scripts/crash/z.sh", target, str(exit_code)] + (["--test-filter", test_filter] if test_filter else [])
                subprocess.run(command + arguments, capture_output=True, text=True)
                seen.append(read_last_run(last_run_path("scripts/crash/z.sh")))
            wanted = [["beta"], ["beta", "gamma"], ["beta", "gamma"], ["beta", "gamma", "beta::one"], ["beta", "gamma", "beta::one"], ["gamma"]]
            os.environ.pop(LAST_RUNS_VARIABLE, None)
            unset = subprocess.run(command + ["last-run-target", "scripts/crash/w.sh", "fake", "101", "--target-file", os.path.join(work, "no-such.rs")],
                                   capture_output=True, text=True, env={name: value for name, value in os.environ.items() if name != LAST_RUNS_VARIABLE})
            cell("上次红：测试目标", seen == wanted and "不写共用的清单" in unset.stdout, f"每一步之后 {seen}，应为 {wanted}；假目标那一趟：{unset.stdout.strip()}")
        finally:
            os.environ.pop(LAST_RUNS_VARIABLE, None)

        # audit：一棵小仓
        def audit_root(name, registry_rows, scripts):
            root = os.path.join(work, name)
            os.makedirs(os.path.join(root, ".claude", "gate.d"))
            os.makedirs(os.path.join(root, ".claude", "hooks"))
            os.makedirs(os.path.join(root, "scripts", "gate"))
            with open(os.path.join(root, REGISTRY), "w", encoding="utf-8") as handle:
                handle.write("# 自证用\n# scan: scripts/gate/*.sh\n" + "".join("\t".join(row) + "\n" for row in registry_rows))
            with open(os.path.join(root, SELFTEST_REGISTRY), "w", encoding="utf-8") as handle:
                handle.write("".join(f"scripts/gate/{script}\tlight\t# 自证用\n" for script in scripts))
            for script, body in scripts.items():
                with open(os.path.join(root, "scripts", "gate", script), "w", encoding="utf-8") as handle:
                    handle.write(body)
            return root

        green_body = ("#!/usr/bin/env bash\n# 判定库：自证用\necho 'REUSE_SELFTEST script=scripts/gate/a.sh same-inputs=reused unrelated-change=reused "
                      "unit-change=rejudged-only-affected items=3 affected=1'\n")
        red_body = "#!/usr/bin/env bash\n# 判定库：自证用\necho 没有那一格\n"
        mention = "#!/usr/bin/env bash\n# 只提到别人的复用\necho\n"
        row = lambda script, library="<临时目录>/a": [f"scripts/gate/{script}", f"bash scripts/gate/{script} --selftest", library, CELL_NAME]  # noqa: E731

        def audit_of(root):
            buffer = []
            completed = subprocess.run(command + ["audit", root, "--jobs", "2"], capture_output=True, text=True)
            buffer.append(completed.stdout)
            return completed.returncode, "".join(buffer)

        root = audit_root("audit-green", [row("a.sh"), row("c.sh", f"{NOT_COUNTED_PREFIX}只提到别人"), ["scripts/gate/d.sh", "-", f"{NO_LIBRARY_PREFIX}待立", "-"]],
                          {"a.sh": green_body, "c.sh": mention, "d.sh": "#!/usr/bin/env bash\necho\n"})
        got, text = audit_of(root)
        cell("audit：过的、不算的、无判定库的", got == 0 and "复用三问过 1 份" in text and "本次未跑 1 份" in text and "不算 1 份" in text, f"退 {got}：{text.strip()[-400:]}")
        root = audit_root("audit-no-line", [row("b.sh")], {"b.sh": red_body})
        got, text = audit_of(root)
        cell("audit：缺规范输出判红", got == 1 and "输出里没有 REUSE_SELFTEST script=scripts/gate/b.sh" in text, f"退 {got}：{text.strip()[-400:]}")
        root = audit_root("audit-missing", [row("a.sh")], {"a.sh": green_body, "e.sh": red_body})
        got, text = audit_of(root)
        cell("audit：漏登记判红", got == 1 and "漏登记：scripts/gate/e.sh" in text, f"退 {got}：{text.strip()[-400:]}")
        root = audit_root("audit-claimed-no-library", [row("a.sh"), ["scripts/gate/e.sh", "-", f"{NO_LIBRARY_PREFIX}待立", "-"]], {"a.sh": green_body, "e.sh": red_body})
        got, text = audit_of(root)
        cell("audit：声称复用登记成无判定库判红", got == 1 and "不许登记成无判定库" in text, f"退 {got}：{text.strip()[-400:]}")
        root = audit_root("audit-gone", [row("a.sh"), row("gone.sh")], {"a.sh": green_body})
        got, text = audit_of(root)
        cell("audit：指到不存在的路径判红", got == 1 and "指到不存在的路径：scripts/gate/gone.sh" in text, f"退 {got}：{text.strip()[-400:]}")
        # 两行第三列点名不同的判法脚本，自证各打它点名的那一份；按登记次序最后一行点名的不是前一行的（取漏出来的变量时前一行判红）
        judged_by = lambda judge: green_body.replace("scripts/gate/a.sh", f"scripts/judges/{judge}")  # noqa: E731
        root = audit_root("audit-own-library", [row("x.sh", "<临时目录>/x（判法在 scripts/judges/judge-x.sh）"), row("y.sh", "<临时目录>/y（判法在 scripts/judges/judge-y.sh）")],
                          {"x.sh": judged_by("judge-x.sh"), "y.sh": judged_by("judge-y.sh")})
        got, text = audit_of(root)
        cell("audit：每一行认自己第三列点名的判法脚本", got == 0 and "复用三问过 2 份" in text, f"退 {got}：{text.strip()[-400:]}")
        root = audit_root("audit-heavy", [row("h.sh")], {"h.sh": green_body.replace("a.sh", "h.sh")})
        with open(os.path.join(root, SELFTEST_REGISTRY), "w", encoding="utf-8") as handle:
            handle.write("scripts/gate/h.sh\theavy\t# 偏离上游：自证用\n")
        os.environ[SELFTEST_OUTPUTS_VARIABLE] = os.path.join(work, "outputs")
        try:
            got, text = audit_of(root)
            not_stored = got == 77 and "没有上一次输出" in text
            kept = subprocess.run(command + ["keep-output", root, "scripts/gate/h.sh", "--", "bash", "scripts/gate/h.sh", "--selftest"], capture_output=True, text=True)
            got, text = audit_of(root)
            stored = got == 0 and "读 scripts/gate/h.sh 的上一次输出" in text
            with open(os.path.join(root, "scripts", "gate", "h.sh"), "a", encoding="utf-8") as handle:
                handle.write("# 改了一行\n")
            got, text = audit_of(root)
            stale = got == 77 and "旧版本" in text
        finally:
            os.environ.pop(SELFTEST_OUTPUTS_VARIABLE, None)
        cell("audit：heavy 读上一次输出、脚本变了本次未跑", not_stored and kept.returncode == 0 and stored and stale,
             f"没存 {not_stored}，keep-output 退 {kept.returncode}，存了 {stored}，变了 {stale}：{text.strip()[-300:]}")

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        for item, label in (("short-reuse-passes", "三问：第二趟少复用一条"), ("extra-rejudge-passes", "三问：闭包内改动多现判一条"),
                            ("rejudged-sentinel-passes", "哨兵：被现判判红"), ("forced-mismatch-passes", "哨兵：强制现判判定不同判红"),
                            ("last-run-partial-overwrites", "上次红：只跑几项照留没跑到的"), ("last-run-in-place", "上次红：原子换上"),
                            ("audit-line-unchecked", "audit：缺规范输出判红"), ("audit-scan-skipped", "audit：漏登记判红"),
                            ("audit-leaked-library", "audit：每一行认自己第三列点名的判法脚本")):
            completed = subprocess.run([sys.executable, SCRIPT_PATH, "--selftest"], capture_output=True, text=True, env=dict(os.environ, REUSE_CHECKS_BREAK=item))
            if completed.returncode == 0 or f"{label} 这一格判错" not in completed.stdout:
                breaks_missed.append(f"{item}（要红在「{label}」）")
            else:
                breaks_proven += 1
    if wrong:
        print(f"  ✗ lib_reuse_checks 自证 {cases} 格里 {len(wrong)} 格判错：{'、'.join(wrong)}")
        print("     → 怎么办：按判错那几格的标签找对应的函数修（三问 three_questions、哨兵 ReuseSentinel、上次红 write_last_run、audit）；判据写在文件头")
        return EXIT_RED
    if breaks_missed:
        print(f"  ✗ 弄坏开关打开之后没红在点名那一格：{'；'.join(breaks_missed)}")
        print("     → 怎么办：那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都红在点名那一格" if not BREAK else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ lib_reuse_checks 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
