#!/usr/bin/env python3
# admission: always 每一趟照此刻的源码、输入与判定库现算每一项的键、现查库；上一趟的结论只经判定库按键复用，不替这一趟作保
# run-condition: command python3
"""实验长跑入口的按项判定：每一项（点、段、片、步）一条判定，存进 prune-server 的 apparatus-items 族，键相同、库里判绿的复用。
被长跑入口调（shell 入口经下面的命令行，python 入口 import 之后调 judge_items）；判定库只有 prune-server 那一份
（.claude/rules/verification.md「判定库只有一份」），本地不建、不读、不写库目录。

请求文件（制表符分隔，# 起头与空行跳过；一项可以有几行请求，按项名并起来）：
  <项><TAB><入口>[<TAB><请求项>…]   一条判定请求，写法同 scripts/gate/admission.py 文件头「判定请求」（root:、target:、feature:、profile:、
                                    judge:、line:、file: 这些）；项名只许字母、数字、. _ -
  @after<TAB><项><TAB><上游项>       这一项读上游那一项这一趟的产物：上游的键经 line:upstream=<上游的键> 进这一项的键（上游要写在它前面）；上游拒算，这一项也拒算
  @files<TAB><项><TAB><路径>         这一项读的输入：文件进一条 file:，目录取它底下每个普通文件（按相对路径排）各一条；路径相对 --root 或写绝对路径
                                    （file: 在键里是按内容排好的多重集，分不出哪份是哪个角色）
  @input<TAB><项><TAB><角色><TAB><路径>  这一项读的一份角色要紧的输入文件：内容的 sha256 经 line:input:<角色>=<sha256>（不在写「不在」）进键，路径不进
  line:compare=<exact|timing>        写在请求里：exact（缺省）比产物的 sha256，timing（带计时、线程数的产物）只比绿红与退出码；它照常进键
键（不另写算法）：每条请求经 admission.py 的 compute_unit_key 算（装置入口的单元闭包、工具链、闭包真引到的依赖版本、judge: 的判法闭包、line: 的参数、
  file: 的内容），一项几条请求经 admission.py 的 row_unit_key 合成一把。路径、名字、行号、在副本还是主树都不进键（file: 只进内容）。拒算的项照跑、不复用、不入库。

判定库：族 apparatus-items（scripts/prune-server/prune_server_client.py 文件头「键空间」），种类是调用方给的装置名（--kind）。每项两条：
  items/<键>.out       产物（判绿的才写，先写它）
  items/<键>.verdict   一条判定（JSON：verdict 是 green 或 red、key、item、exit、compare、script、judged（东京日期）；绿的另有 output_sha256）
  读：.verdict 是 green、.out 读得回且 sha256 对得上才算库里有；red 是「记红撤绿」——判红的照同一把键写 red，盖掉原来的绿，下一趟现判；
  .out 读不回或对不上的当没有、现判。服务连不上客户端退 75，调用方整趟退 75，不当「库里没有」。

一趟的三步（状态目录是调用方这一趟新建的空目录，跑完由调用方删；里面不是判定库：只放这一趟的计划、复用项取回的产物与逐项结局）：
  plan <状态目录> --script <入口相对仓根> --kind <装置名> --root <仓根> --requests <请求文件>
      算键、查库、抽复用哨兵；每项打一行「LONG_RUN_ITEM<TAB><项><TAB><judge|reuse|refused><TAB><键或-><TAB><为什么>」，
      另写 <状态目录>/decisions.tsv（「<项><TAB><决定>」）；复用的项把库里的产物与退出码放进 <状态目录>/outputs/<项>.out、<项>.exit。
  record <状态目录> <项> <退出码> <产物文件或 -> [--red]
      judge 或 refused 的项跑完调一次：不带 --red 是判绿；产物照原样存（refused 的不存）。复用的项不调；调了按「键相同却现判」记（哨兵那一条判红）。
  finish <状态目录>
      重算这一趟现判过的项的键（与 plan 时不同的：源码或输入在跑的中途变了，把这一趟写进库的那一条撤成红、判红）；判复用哨兵；
      逐项打「  · <项>：绿|红，现判|强制现判|复用」或「拒算（…）」「没跑到」，末行「LONG_RUN_ITEMS script=… kind=… 现判 N、复用 M、拒算 R」。
      退 0 没有红；退 1 键中途变了、或复用哨兵红。项自己判红不在这里算（调用方自己报）。
  python 入口：judge_items(script, kind, root, 请求正文, judge_item, printer) 一次走完三步（judge_item(项) 交 (退出码, 产物字节, 绿不绿)）。

复用哨兵（.claude/rules/verification.md「每一趟真跑开跑前……整道红」；判法是 scripts/gate/lib_reuse_checks.py 的 ReuseSentinel，这里不另写）：
  候选 = 键相同、库里判绿且产物读得回的项；种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取，打进输出），抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵、
  另抽 1 条强制现判（plan 答 judge）。强制现判那一条判完：与库里同一判定，库照旧；不同，库照旧、finish 红「判定不稳」。
  finish 红的三种：哨兵这一趟被现判（「复用失灵」）、强制现判的判定与库里不同（「判定不稳」）、强制现判那一条这一趟还是复用（「强制现判没起作用」）。

只在自证里认的变量（只在临时 prune-server 底下认：prune_server_client.selftest_service_problem() 判；不在临时服务底下设了任一个，退 2、什么都不做）：
  LONG_RUN_ITEMS_LIBRARY_DIRECTORY   命名空间按它给（自证一格一份；不设按 --root）。不在临时服务底下时命名空间一律是空的，主仓与副本读写同一份
  LONG_RUN_ITEMS_SELFTEST_ENTRIES    一份文件，一行「<项><TAB><入口>[<TAB><请求项>…]」：点名的项把请求里的程序那几样（入口与 root:、target:、feature:、cfg:、
                                     profile:、no-default-features）换成这一个入口，line:、file:、judge: 照留（复用三问的假装置入口）
  LONG_RUN_ITEMS_SELFTEST_FAULT      自证造的失灵：rejudge-sentinels（哨兵答 judge）、forced-answered-reuse（强制现判那一条答 reuse）、
                                     forced-verdict-altered（强制现判那一条的产物末尾多一行、退出码加 1：exact 与 timing 两种比法都比得出不同）
  LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE 设成 1：judge_items 不调入口给的 judge_item，每项照「退 0、产物一行 fake judgment of <项>、判绿」记（python 入口的假驱动）
  LONG_RUN_ITEMS_BREAK               弄坏开关（下面）

复用格（各长跑入口的自证都经它跑，入口只给一棵样板树和起法）：
  reuse-cells --script <入口相对仓根> --template <样板树> --items <项甲>,<项乙> --work <目录> [--green-exit <码>] [--fake-judge] -- <起法…>
      --fake-judge：每一趟设 LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE=1（python 入口经 judge_items 判的，判法换成假的）
      起法里的 {tree} 换成这一格的树；每一格把样板树拷一份（符号链接照留），往里写假装置入口 scripts/experiment-apparatus/long_run_reuse_fake_entries.py，
      项甲、项乙的入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成它的 measure_alpha、measure_beta；起的进程各在自己的进程组里，跑完核没留下进程。格：
      reuse-three-questions          四趟（库空、同一份输入、假入口文件里加一行注释和一个没人调的函数、改只在 measure_alpha 闭包里的 alpha_detail）；
                                     哨兵数设 1000（键相同的全当哨兵、不抽强制现判）；判法 lib_reuse_checks.report_three_questions，打 REUSE_SELFTEST script=<入口> …；
                                     另核头一趟项甲、项乙都现判、四趟都打总计「现判 N、复用 M、拒算 R」、四趟都退 --green-exit
                                     （入口这一趟别的项在样板树里有键的照算进三问：它们不在被改单元的闭包里，同样要复用；拒算的不算）
      key-independent-of-directory   判完的树整棵拷到另一层目录下再起，项甲、项乙连同别的有键的项都复用
      reuse-sentinel                 哨兵 1 条、种子 7：恰好一项强制现判、别的都复用，种子打进输出，哨兵与键都不红，退 --green-exit
      sentinel-rejudged-red          LONG_RUN_ITEMS_SELFTEST_FAULT=rejudge-sentinels：退 1、红「复用失灵」
      forced-verdict-differs-red     LONG_RUN_ITEMS_SELFTEST_FAULT=forced-verdict-altered：退 1、红「判定不稳」
      forced-still-reused-red        LONG_RUN_ITEMS_SELFTEST_FAULT=forced-answered-reuse：退 1、红「强制现判没起作用」
      末行「long_run_item_verdicts 复用格（<入口>）：N 格，红 M 格」；退 0 全绿、1 有红。
  selftest-service                   这一趟在不在临时 prune-server 底下（客户端 selftest_service_problem）：在退 0，不在打原因、退 2
  plant-fake-entries <树>            往树里写那份假装置入口（入口自证的别的格把全部项都换成 measure_alpha 时用，不然样板树里的真装置入口算不出键、全拒算）

自证（--selftest）：只在临时 prune-server 上跑（不在就退 2、一格都不跑）。起法：
    python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/long_run_item_verdicts.py --selftest
  拿一份假入口（样板树里的 long-run-fake-entry.sh，两项 alpha、beta）走一遍复用格（打 REUSE_SELFTEST script=scripts/experiment-apparatus/long_run_item_verdicts.py），
  另几格：red-withdraws-green（判红的项下一趟现判，判绿的复用）、key-changed-midway（跑的中途改了 alpha 的闭包：退 1、红「中途变了」；
  改回之后再起，alpha 现判（库里那一条已撤成红）、beta 复用）、selftest-variables-outside-temporary-service（不在临时服务底下设了自证的变量：退 2）、
  service-unavailable（服务连不上：plan 退 75）。末行「long_run_item_verdicts 自证：N 格，红 M 格」。

退出码：0 成；1 finish 判红、复用格有红；2 用法错、请求文件写错、自证的变量设在临时服务之外、自证没有临时服务；75 prune-server 不可用；3 客户端别的错（照客户端的码）。

弄坏开关 LONG_RUN_ITEMS_BREAK=<项>[,<项>…]（只在临时 prune-server 底下认，同上；调它的入口在自证里起的，同样照它坏）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格：
  LONG_RUN_ITEMS_BREAK=library-unread  plan 不读库，一律答 judge
      「reuse-three-questions」「key-independent-of-directory」「reuse-sentinel」「sentinel-rejudged-red」「forced-verdict-differs-red」
      「forced-still-reused-red」「red-withdraws-green」「key-changed-midway」
  LONG_RUN_ITEMS_BREAK=key-with-root-path  每一项的键另加 --root 的真实路径（line:root=…）
      「key-independent-of-directory」
  LONG_RUN_ITEMS_BREAK=red-recorded-green  record 带 --red 也照绿记
      「red-withdraws-green」
  LONG_RUN_ITEMS_BREAK=sentinel-check-skipped  finish 不判复用哨兵
      「sentinel-rejudged-red」「forced-verdict-differs-red」「forced-still-reused-red」
  LONG_RUN_ITEMS_BREAK=key-recheck-skipped  finish 不重算键（中途变了也不撤、不红）
      「key-changed-midway」
"""
import datetime
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "gate"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "prune-server"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
import lib_reuse_checks  # noqa: E402
import prune_server_client  # noqa: E402

SCRIPT_KEY = "scripts/experiment-apparatus/long_run_item_verdicts.py"   # 自证规范输出认的脚本名
FAMILY = prune_server_client.FAMILY_APPARATUS_ITEMS
LIBRARY_NAME = "long-run-item-verdicts"   # 只当临时服务底下的命名空间用（文件头「只在自证里认的变量」）
LIBRARY_DIRECTORY_VARIABLE = "LONG_RUN_ITEMS_LIBRARY_DIRECTORY"
SUBSTITUTION_VARIABLE = "LONG_RUN_ITEMS_SELFTEST_ENTRIES"
FAULT_VARIABLE = "LONG_RUN_ITEMS_SELFTEST_FAULT"
BREAK_VARIABLE = "LONG_RUN_ITEMS_BREAK"
FAKE_JUDGE_VARIABLE = "LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE"
SELFTEST_ONLY_VARIABLES = (LIBRARY_DIRECTORY_VARIABLE, SUBSTITUTION_VARIABLE, FAULT_VARIABLE, FAKE_JUDGE_VARIABLE, BREAK_VARIABLE)
FAULTS = ("rejudge-sentinels", "forced-answered-reuse", "forced-verdict-altered")
BREAK_SWITCHES = ("library-unread", "key-with-root-path", "red-recorded-green", "sentinel-check-skipped", "key-recheck-skipped")
ITEM_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*\Z")
PROGRAM_TOKEN_KINDS = ("root", "target", "feature", "cfg", "profile")
NO_DEFAULT_FEATURES = "no-default-features"
COMPARE_TOKEN_PREFIX = "line:compare="
COMPARE_EXACT, COMPARE_TIMING = "exact", "timing"
JUDGE, REUSE, REFUSED = "judge", "reuse", "refused"
REUSED_STATE, JUDGED_STATE, FORCED_STATE = lib_reuse_checks.REUSED, lib_reuse_checks.JUDGED, lib_reuse_checks.FORCED
REFUSED_STATE = "拒算"
PLAN_FILE = "plan.json"
DECISIONS_FILE = "decisions.tsv"
OUTPUTS_DIRECTORY = "outputs"
OBSERVED_DIRECTORY = "observed"
PLAN_LINE = "LONG_RUN_ITEM"
SUMMARY_LINE = "LONG_RUN_ITEMS"
FORCED_ALTERATION = b"forced-verdict-altered\n"
TOKYO = datetime.timezone(datetime.timedelta(hours=9))
EXIT_RED, EXIT_USAGE = 1, 2
SELFTEST_RUN_SECONDS = 900    # 自证里起一趟入口的上限（卡住时那一格照超时记红，不一直等）
# 复用格（文件头「复用格」）
FAKE_MODULE = "scripts/experiment-apparatus/long_run_reuse_fake_entries.py"
FAKE_MODULE_TEXT = ('"""复用格的假装置入口：long_run_item_verdicts.py reuse-cells 写进每一格的树里，项甲、项乙的入口换成这里的两个函数。"""\n\n\n'
                    "def alpha_detail():\n    return 1\n\n\n"
                    "def beta_detail():\n    return 2\n\n\n"
                    "def measure_alpha():\n    return alpha_detail()\n\n\n"
                    "def measure_beta():\n    return beta_detail()\n")
FAKE_UNIT_CHANGE = ("    return 1\n", "    return 3\n")   # 只在 measure_alpha 的闭包里
FAKE_UNRELATED_CHANGE = "\n\n# 闭包外的改动：一行注释与一个没人调的函数\ndef spare_unused():\n    return 9\n"
SENTINEL_SEED_IN_CELLS = "7"
SENTINEL_RED_PHRASES = ("复用失灵：", "判定不稳：", "强制现判没起作用", "键在这一趟中途变了")
ALL_SENTINELS = "1000"        # 键相同的全当哨兵、不抽强制现判（复用三问那四趟）


class UsageError(Exception):
    """命令行、请求文件写错，自证的变量设在临时服务之外：退 2。"""


# ── 只在自证里认的变量 ──

def selftest_only_problem(environment=None):
    """设了只在自证里认的变量、又不在临时 prune-server 底下：交一句为什么；没事交空串。"""
    environment = os.environ if environment is None else environment
    setting = [name for name in SELFTEST_ONLY_VARIABLES if environment.get(name)]
    if not setting:
        return ""
    service = prune_server_client.selftest_service_problem(environment)
    return f"{'、'.join(setting)} 只在临时 prune-server 底下的自证里认（{service}）" if service else ""


def require_selftest_variables_allowed():
    problem = selftest_only_problem()
    if problem:
        raise UsageError(problem + "；平常跑不设它们")


def break_is(name):
    return name in os.environ.get(BREAK_VARIABLE, "").split(",")


def fault_is(name):
    return name in os.environ.get(FAULT_VARIABLE, "").split(",")


# ── 请求 ──

class Requests:
    """请求文件读出来的：项的次序、每项的请求行、上游、要进键的输入路径。"""

    def __init__(self):
        self.order = []
        self.lines = {}
        self.upstream = {}
        self.files = {}
        self.inputs = {}

    def item(self, name, where):
        if not ITEM_NAME.match(name):
            raise UsageError(f"{where}：项名「{name}」只许字母、数字、. _ -，不以符号起头")
        if name not in self.lines:
            self.order.append(name)
            self.lines[name], self.upstream[name], self.files[name], self.inputs[name] = [], [], [], []
        return name


def parse_requests(text, where):
    requests = Requests()
    for number, line in enumerate(text.split("\n"), start=1):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.split("\t")
        place = f"{where} 第 {number} 行"
        if columns[0] == "@input":
            if len(columns) != 4 or not ITEM_NAME.match(columns[2]) or not columns[3]:
                raise UsageError(f"{place}：要写成「@input<TAB><项><TAB><角色><TAB><路径>」（角色只许字母、数字、. _ -）")
            requests.inputs[requests.item(columns[1], place)].append((columns[2], columns[3]))
            continue
        if columns[0] in ("@after", "@files"):
            if len(columns) != 3 or not columns[1] or not columns[2]:
                raise UsageError(f"{place}：要写成「{columns[0]}<TAB><项><TAB><…>」")
            if columns[0] == "@after" and (columns[2] not in requests.lines or columns[2] == columns[1]):
                raise UsageError(f"{place}：上游「{columns[2]}」要是写在前面的另一项")
            item = requests.item(columns[1], place)
            (requests.upstream if columns[0] == "@after" else requests.files)[item].append(columns[2])
            continue
        if len(columns) < 2 or not columns[1]:
            raise UsageError(f"{place}：请求行要写成「<项><TAB><入口>[<TAB><请求项>…]」")
        requests.lines[requests.item(columns[0], place)].append("\t".join(column for column in columns if column))
    return requests


def substitute_entries(requests, path):
    """自证换掉的装置入口（文件头「只在自证里认的变量」LONG_RUN_ITEMS_SELFTEST_ENTRIES）。"""
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            columns = [column for column in line.rstrip("\n").split("\t") if column]
            if not columns or columns[0].startswith("#"):
                continue
            if len(columns) < 2:
                raise UsageError(f"{SUBSTITUTION_VARIABLE} 那份文件里「{line.strip()}」少了入口")
            if columns[0] not in requests.lines:
                continue   # 这一趟没点到的项（入口只判了一部分项）
            kept = []
            for request in requests.lines[columns[0]]:
                for token in request.split("\t")[2:]:
                    if token == NO_DEFAULT_FEATURES or token.partition(":")[0] in PROGRAM_TOKEN_KINDS or token in kept:
                        continue
                    kept.append(token)
            requests.lines[columns[0]] = ["\t".join([columns[0], *columns[1:], *kept])]


def input_file_tokens(root, path):
    """@files 点的路径 → file: 请求项：文件一条；目录取它底下每个普通文件（按相对路径排）；不在照写一条（admission 记成「不在」）。"""
    absolute = path if os.path.isabs(path) else os.path.join(root, path)
    if not os.path.isdir(absolute):
        return [f"file:{path}"]
    tokens = []
    for directory, subdirectories, names in os.walk(absolute):
        subdirectories.sort()
        for name in sorted(names):
            full = os.path.join(directory, name)
            if os.path.isfile(full):
                tokens.append(f"file:{os.path.join(path, os.path.relpath(full, absolute))}")
    return tokens


def input_file_digest(root, path):
    """@input 点的那一份文件内容的 sha256（file: 进键是按内容排好的多重集、分不出谁是谁；角色要紧的输入写 @input）；不在写「不在」。"""
    absolute = path if os.path.isabs(path) else os.path.join(root, path)
    if not os.path.isfile(absolute):
        return "不在"
    digest = hashlib.sha256()
    with open(absolute, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def compare_mode(lines):
    for line in lines:
        for token in line.split("\t")[2:]:
            if token.startswith(COMPARE_TOKEN_PREFIX):
                mode = token[len(COMPARE_TOKEN_PREFIX):]
                if mode not in (COMPARE_EXACT, COMPARE_TIMING):
                    raise UsageError(f"line:compare= 只认 {COMPARE_EXACT}、{COMPARE_TIMING}，写的是「{mode}」")
                return mode
    return COMPARE_EXACT


def keys_of_lines(context, lines):
    """几条请求行 → admission.py 的 row_unit_key（每条 compute_unit_key、合成一把）。"""
    import admission
    try:
        parsed = [admission.parse_request_line(line) for line in lines]
    except admission.RegistrationError as error:
        raise UsageError(str(error)) from error
    return admission.row_unit_key(context, parsed, [])


def item_keys(root, requests):
    """{项: (键或 None, 进键的请求行, 拒算说明)}：上游的键经 line:upstream= 进来，@files 的文件经 file: 进来。"""
    import admission
    context = admission.KeyContext(root)
    keyed = {}
    for item in requests.order:
        lines = list(requests.lines[item])
        if not lines:
            keyed[item] = (None, lines, "这一项一条请求都没有（只写了 @after / @files）")
            continue
        missing = [upstream for upstream in requests.upstream[item] if keyed[upstream][0] is None]
        extra = [f"line:upstream={keyed[upstream][0]}" for upstream in requests.upstream[item]]
        for path in requests.files[item]:
            extra += input_file_tokens(root, path)
        for role, path in requests.inputs[item]:
            extra.append(f"line:input:{role}={input_file_digest(root, path)}")
        if break_is("key-with-root-path"):
            extra.append(f"line:root={os.path.realpath(root)}")
        if extra:
            lines[0] = "\t".join([lines[0], *extra])
        if missing:
            keyed[item] = (None, lines, f"上游 {'、'.join(missing)} 拒算")
            continue
        unit_key = keys_of_lines(context, lines)
        reason = "；".join(refusal.split("\t", 1)[-1].replace("\t", " ") for refusal in unit_key.refusals[:3])
        keyed[item] = (unit_key.key, lines, reason)
    return keyed


# ── 判定库 ──

def library_directory(root):
    return os.environ.get(LIBRARY_DIRECTORY_VARIABLE) or os.path.join(os.path.realpath(root), LIBRARY_NAME)


def library_of(kind, directory):
    return prune_server_client.library_store(FAMILY, kind, directory)


def verdict_relative(key):
    return f"items/{key}.verdict"


def output_relative(key):
    return f"items/{key}.out"


def verdict_text(green, exit_code, output_sha256, compare):
    """哨兵比的那一份判定：绿的带退出码（exact 另带产物的 sha256），红的带退出码。"""
    if not green:
        return f"red exit={exit_code}"
    return f"green exit={exit_code}" + (f" sha256={output_sha256}" if compare == COMPARE_EXACT else "")


def parsed_record(raw, key):
    try:
        record = json.loads(raw.decode("utf-8")) if raw is not None else None
    except (UnicodeDecodeError, ValueError):
        return None
    return record if isinstance(record, dict) and record.get("key") == key else None


def today_in_tokyo():
    return datetime.datetime.now(TOKYO).date().isoformat()


def write_record(store, key, item, verdict, exit_code, compare, script, output=None, reason=""):
    """先写产物（判绿的）、再写判定：读的一方核 sha256，产物没写完时判定还是旧的。"""
    record = {"verdict": verdict, "key": key, "item": item, "exit": exit_code, "compare": compare, "script": script, "judged": today_in_tokyo()}
    if reason:
        record["reason"] = reason
    if output is not None:
        record["output_sha256"] = hashlib.sha256(output).hexdigest()
        store.write(output_relative(key), output)
    store.write(verdict_relative(key), json.dumps(record, ensure_ascii=False, sort_keys=True))


# ── 状态目录 ──

def write_json(path, value):
    temporary = path + ".writing"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False, indent=1, sort_keys=True)
    os.replace(temporary, path)


def read_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def read_plan(state_directory):
    path = os.path.join(state_directory, PLAN_FILE)
    if not os.path.isfile(path):
        raise UsageError(f"{state_directory} 里没有计划：先起 plan")
    return read_json(path)


def observed_path(state_directory, item):
    return os.path.join(state_directory, OBSERVED_DIRECTORY, item + ".json")


def plan(state_directory, script, kind, root, requests_text, where="请求", printer=print):
    """一趟的头一步：算键、查库、抽哨兵，结局写进状态目录；交 (项的次序, {项: (决定, 为什么)})。"""
    require_selftest_variables_allowed()
    if os.path.exists(os.path.join(state_directory, PLAN_FILE)):
        raise UsageError(f"{state_directory} 里已有一份计划：一趟一个新的状态目录")
    requests = parse_requests(requests_text, where)
    if not requests.order:
        raise UsageError(f"{where}里一项都没有")
    if os.environ.get(SUBSTITUTION_VARIABLE):
        substitute_entries(requests, os.environ[SUBSTITUTION_VARIABLE])
    keyed = item_keys(root, requests)
    compare = {item: compare_mode(requests.lines[item]) for item in requests.order}
    directory = library_directory(root)
    store = library_of(kind, directory)
    keyed_items = [item for item in requests.order if keyed[item][0]]
    stored, outputs = {}, {}
    if keyed_items and not break_is("library-unread"):
        for item, raw in zip(keyed_items, store.read_many([verdict_relative(keyed[item][0]) for item in keyed_items])):
            record = parsed_record(raw, keyed[item][0])
            if record is not None:
                stored[item] = record
        green = [item for item in keyed_items if stored.get(item, {}).get("verdict") == "green"]
        for item, output in zip(green, store.read_many([output_relative(keyed[item][0]) for item in green]) if green else []):
            if output is not None and hashlib.sha256(output).hexdigest() == stored[item].get("output_sha256"):
                outputs[item] = output
    candidates = {item: verdict_text(True, stored[item].get("exit", 0), stored[item]["output_sha256"], compare[item]) for item in outputs}
    sentinel = lib_reuse_checks.ReuseSentinel(script, kind)
    sentinel.plan(candidates, printer=printer)
    decisions = {}
    for item in requests.order:
        key, _lines, reason = keyed[item]
        if key is None:
            decisions[item] = (REFUSED, f"拒算：{reason}")
        elif item == sentinel.forced_item and not fault_is("forced-answered-reuse"):
            decisions[item] = (JUDGE, f"强制现判（复用哨兵抽中，种子 {sentinel.seed}）")
        elif item in candidates and fault_is("rejudge-sentinels") and item in sentinel.sentinels:
            decisions[item] = (JUDGE, "自证造的失灵：哨兵照现判")
        elif item in candidates:
            decisions[item] = (REUSE, "键相同、库里判绿")
        elif stored.get(item, {}).get("verdict") == "red":
            decisions[item] = (JUDGE, "上次判红：键记着红，这一趟现判")
        elif item in stored:
            decisions[item] = (JUDGE, "库里有判定、产物读不回或对不上，现判")
        else:
            decisions[item] = (JUDGE, "库里没有这把键")
    os.makedirs(os.path.join(state_directory, OUTPUTS_DIRECTORY), exist_ok=True)
    os.makedirs(os.path.join(state_directory, OBSERVED_DIRECTORY), exist_ok=True)
    for item in requests.order:
        if decisions[item][0] != REUSE:
            continue
        with open(os.path.join(state_directory, OUTPUTS_DIRECTORY, item + ".out"), "wb") as handle:
            handle.write(outputs[item])
        with open(os.path.join(state_directory, OUTPUTS_DIRECTORY, item + ".exit"), "w", encoding="utf-8") as handle:
            handle.write(f"{stored[item].get('exit', 0)}\n")
        write_json(observed_path(state_directory, item), {"state": REUSED_STATE, "verdict": candidates[item]})
    write_json(os.path.join(state_directory, PLAN_FILE), {
        "script": script, "kind": kind, "root": os.path.realpath(root), "library_directory": directory, "order": requests.order,
        "forced": sentinel.forced_item if sentinel.state == "planned" else None, "sentinel": sentinel.to_json(),
        "items": {item: {"key": keyed[item][0], "lines": keyed[item][1], "upstream": requests.upstream[item], "compare": compare[item],
                         "decision": decisions[item][0], "reason": decisions[item][1], "stored": candidates.get(item)} for item in requests.order}})
    with open(os.path.join(state_directory, DECISIONS_FILE), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{item}\t{decisions[item][0]}\n" for item in requests.order))
    for item in requests.order:
        printer(f"{PLAN_LINE}\t{item}\t{decisions[item][0]}\t{keyed[item][0] or '-'}\t{decisions[item][1]}")
    return requests.order, decisions


def reused_output(state_directory, item):
    """复用的项从状态目录取回 (退出码, 产物字节)。"""
    with open(os.path.join(state_directory, OUTPUTS_DIRECTORY, item + ".exit"), encoding="utf-8") as handle:
        exit_code = int(handle.read().strip())
    with open(os.path.join(state_directory, OUTPUTS_DIRECTORY, item + ".out"), "rb") as handle:
        return exit_code, handle.read()


def record(state_directory, item, exit_code, output, green, printer=print):
    """一项跑完：记结局；不是拒算、不是强制现判的照写进库（判绿写产物与绿，判红写红、盖掉原有的绿）。"""
    require_selftest_variables_allowed()
    planned = read_plan(state_directory)
    entry = planned["items"].get(item)
    if entry is None:
        raise UsageError(f"项「{item}」不在这一趟的计划里")
    forced = item == planned["forced"]
    if forced and fault_is("forced-verdict-altered"):
        output, exit_code = output + FORCED_ALTERATION, exit_code + 1
    if break_is("red-recorded-green"):
        green = True
    verdict = verdict_text(green, exit_code, hashlib.sha256(output).hexdigest(), entry["compare"])
    if entry["key"] is None:
        state = REFUSED_STATE
    else:
        state = JUDGED_STATE
        if not forced:
            store = library_of(planned["kind"], planned["library_directory"])
            write_record(store, entry["key"], item, "green" if green else "red", exit_code, entry["compare"], planned["script"],
                         output if green else None, "" if green else "判红：记红撤绿，下一趟现判")
    write_json(observed_path(state_directory, item), {"state": state, "verdict": verdict})
    return verdict


def finish(state_directory, printer=print):
    """一趟的末一步：重算现判过的项的键、判复用哨兵、逐项报、打总计；交退出码（0 没红、1 红）。"""
    require_selftest_variables_allowed()
    planned = read_plan(state_directory)
    order, items = planned["order"], planned["items"]
    observed = {}
    for item in order:
        if os.path.isfile(observed_path(state_directory, item)):
            observed[item] = read_json(observed_path(state_directory, item))
    problems = []
    judged = [item for item in order if observed.get(item, {}).get("state") == JUDGED_STATE]
    if judged and not break_is("key-recheck-skipped"):
        import admission
        context = admission.KeyContext(planned["root"])
        withdrawn = set()
        store = library_of(planned["kind"], planned["library_directory"])
        for item in order:
            if item not in judged:
                continue
            upstream_withdrawn = [upstream for upstream in items[item]["upstream"] if upstream in withdrawn]
            again = keys_of_lines(context, items[item]["lines"]).key
            if again == items[item]["key"] and not upstream_withdrawn:
                continue
            withdrawn.add(item)
            if item != planned["forced"]:
                write_record(store, items[item]["key"], item, "red", -1, items[item]["compare"], planned["script"],
                             reason="键在这一趟中途变了：撤成红，下一趟现判")
            cause = f"上游 {'、'.join(upstream_withdrawn)} 已撤" if upstream_withdrawn else f"开跑时 {items[item]['key'][:16]}…，收尾时 {(again or '拒算')[:16]}…"
            problems.append((f"{item} 的键在这一趟中途变了（{cause}）：这一项写进库的那一条已撤成红",
                             "这一趟跑着的时候源码或输入被改了：等改动定下来再起同一条命令，这几项照新键现判"))
    sentinel = lib_reuse_checks.ReuseSentinel.from_json(planned["sentinel"])
    for item, value in observed.items():
        sentinel.observe(item, value["state"], value["verdict"])
    if not break_is("sentinel-check-skipped"):
        problems += sentinel.finish(printer)
    counts = {JUDGED_STATE: 0, REUSED_STATE: 0}
    refused = 0
    for item in order:
        value = observed.get(item)
        if items[item]["decision"] == REFUSED:
            refused += 1
        if value is None:
            printer(f"  · {item}：没跑到（计划是 {items[item]['decision']}：{items[item]['reason']}）")
            continue
        color = "绿" if value["verdict"].startswith("green") else "红"
        if value["state"] == REFUSED_STATE:
            printer(f"  · {item}：{color}，拒算（{items[item]['reason']}）")
        elif value["state"] == REUSED_STATE:
            counts[REUSED_STATE] += 1
            printer(f"  · {item}：{color}，复用")
        else:
            counts[JUDGED_STATE] += 1
            printer(f"  · {item}：{color}，{FORCED_STATE if item == planned['forced'] else JUDGED_STATE}")
    printer(f"{SUMMARY_LINE} script={planned['script']} kind={planned['kind']} "
            f"现判 {counts[JUDGED_STATE]}、复用 {counts[REUSED_STATE]}、拒算 {refused}")
    for text, remedy in problems:
        printer(f"  ✗ {text}")
        printer(f"     → 怎么办：{remedy}")
    return EXIT_RED if problems else 0


def judge_items(script, kind, root, requests_text, judge_item, printer=print):
    """python 入口一趟走完三步：judge_item(项) 交 (退出码, 产物字节, 绿不绿)。
    交 (finish 的退出码, {项: (退出码, 产物字节)})；状态目录是这一趟的临时目录，用完删。"""
    state_directory = tempfile.mkdtemp(prefix="long-run-items-")
    try:
        order, decisions = plan(state_directory, script, kind, root, requests_text, printer=printer)
        results = {}
        for item in order:
            if decisions[item][0] == REUSE:
                results[item] = reused_output(state_directory, item)
                continue
            if os.environ.get(FAKE_JUDGE_VARIABLE) == "1":
                exit_code, output, green = 0, f"fake judgment of {item}\n".encode("utf-8"), True
            else:
                exit_code, output, green = judge_item(item)
            record(state_directory, item, exit_code, output, green, printer)
            results[item] = (exit_code, output)
        return finish(state_directory, printer), results
    finally:
        shutil.rmtree(state_directory, ignore_errors=True)


# ── 复用格（文件头「复用格」）──

def run_in_tree(tree, command, changes):
    """起一趟：{tree} 换成这一格的树、工作目录是它；在自己的进程组里起，超时或跑完都核这一组没留下进程。交 (退出码, 输出)。"""
    argv = [part.replace("{tree}", tree) for part in command]
    environment = dict(os.environ)
    for name, value in changes.items():
        if value is None:
            environment.pop(name, None)
        else:
            environment[name] = value.replace("{tree}", tree)
    process = subprocess.Popen(argv, cwd=tree, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, start_new_session=True)
    try:
        output, _ = process.communicate(timeout=SELFTEST_RUN_SECONDS)
        code = process.returncode
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        return 124, output.decode("utf-8", "replace") + f"\n（{SELFTEST_RUN_SECONDS} 秒没跑完，整组停了）"
    text = output.decode("utf-8", "replace")
    try:
        os.killpg(process.pid, 0)
    except ProcessLookupError:
        return code, text
    os.killpg(process.pid, signal.SIGKILL)
    return 125, text + "\n（跑完还留着进程，整组停了）"


def plant_fake_module(tree):
    path = os.path.join(tree, FAKE_MODULE)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(FAKE_MODULE_TEXT)
    return path


def edit_once(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(old) != 1:
        raise AssertionError(f"{path} 里「{old.strip()}」应恰好一处，有 {text.count(old)} 处")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new, 1))


def reuse_cells(script, template, items, work, green_exit, command, printer=print, fake_judge=False):
    """文件头「复用格」那六格；交红了几格。"""
    alpha, beta = items
    failures = []

    def report_cell(name, passed, detail):
        if passed:
            printer(f"  ✓ {name}")
            return
        failures.append(name)
        printer(f"  ✗ {name}：{detail}")
        printer(f"     → 怎么办：照这一格点名的条件看 {script} 与 {SCRIPT_KEY} 的 plan / record / finish 那一段，"
                f"再在临时服务底下单独起 {script} 的 --selftest 到它退 0")

    def new_cell(name, tree_parent=None):
        where = os.path.join(work, name)
        os.makedirs(where)
        tree = os.path.join(tree_parent or where, "tree")
        shutil.copytree(template, tree, symlinks=True)
        plant_fake_module(tree)
        entries = os.path.join(where, "entries.tsv")
        with open(entries, "w", encoding="utf-8") as handle:
            handle.write(f"{alpha}\t{FAKE_MODULE}::measure_alpha\n{beta}\t{FAKE_MODULE}::measure_beta\n")
        settings = {LIBRARY_DIRECTORY_VARIABLE: os.path.join(where, "library"), SUBSTITUTION_VARIABLE: entries, FAULT_VARIABLE: None,
                    FAKE_JUDGE_VARIABLE: "1" if fake_judge else None,
                    lib_reuse_checks.SENTINEL_COUNT_VARIABLE: ALL_SENTINELS, lib_reuse_checks.SENTINEL_SEED_VARIABLE: None}
        return where, tree, settings

    def summary(text):
        found = re.findall(rf"{SUMMARY_LINE} .*?(现判 \d+、复用 \d+、拒算 \d+)", text)
        return found[-1] if found else "（没有总计那一行）"

    # reuse-three-questions
    _where, tree, settings = new_cell("three-questions")
    runs, results = [], []
    for step in ("first", "second", "unrelated", "unit-change"):
        if step == "unrelated":
            with open(os.path.join(tree, FAKE_MODULE), "a", encoding="utf-8") as handle:
                handle.write(FAKE_UNRELATED_CHANGE)
        elif step == "unit-change":
            edit_once(os.path.join(tree, FAKE_MODULE), *FAKE_UNIT_CHANGE)
        code, text = run_in_tree(tree, command, settings)
        runs.append(lib_reuse_checks.states_from_output(text))
        results.append((code, summary(text), text))
    passed, problems = lib_reuse_checks.report_three_questions(script, *runs, [alpha], printer=printer)
    codes = [code for code, _summary, _text in results]
    summaries = [found for _code, found, _text in results]
    report_cell("reuse-three-questions：库空现判、同一份输入全复用、闭包外改动全复用、改 alpha_detail 只现判项甲，四趟都打总计",
         passed and codes == [green_exit] * 4 and "（没有总计那一行）" not in summaries and runs[0].get(alpha) == JUDGED_STATE and runs[0].get(beta) == JUDGED_STATE,
         f"{'；'.join(problems) or '三问都过'}；退出码 {codes}（应都是 {green_exit}）；总计 {summaries}；"
         f"末趟输出末尾：{results[-1][2].strip()[-400:]}")

    # key-independent-of-directory
    where, tree, settings = new_cell("moved")
    first_code, first_text = run_in_tree(tree, command, settings)
    moved = os.path.join(where, "moved", "one-level-deeper", "tree")
    shutil.copytree(tree, moved, symlinks=True)
    code, text = run_in_tree(moved, command, settings)
    states = lib_reuse_checks.states_from_output(text)
    report_cell("key-independent-of-directory：判完的树拷到另一层目录下再起，项甲、项乙连同别的有键的项都复用",
         first_code == green_exit and code == green_exit and states.get(alpha) == REUSED_STATE and states.get(beta) == REUSED_STATE
         and all(state == REUSED_STATE for state in states.values()),
         f"头一趟退 {first_code}（{summary(first_text)}），拷走之后退 {code}、结局 {states}；输出末尾：{text.strip()[-400:]}")

    # reuse-sentinel 与三种红
    _where, tree, settings = new_cell("sentinel")
    first_code, first_text = run_in_tree(tree, command, settings)
    seeded = dict(settings, **{lib_reuse_checks.SENTINEL_COUNT_VARIABLE: "1", lib_reuse_checks.SENTINEL_SEED_VARIABLE: SENTINEL_SEED_IN_CELLS})
    code, text = run_in_tree(tree, command, seeded)
    states = sorted(lib_reuse_checks.states_from_output(text).values())
    report_cell("reuse-sentinel：哨兵 1 条、种子 7：恰好一项强制现判、别的都复用，种子打进输出，哨兵不红",
         first_code == green_exit and code == green_exit and states.count(FORCED_STATE) == 1 and len(states) >= 2
         and all(state in (REUSED_STATE, FORCED_STATE) for state in states)
         and f"种子 {SENTINEL_SEED_IN_CELLS}（" in text and not any(phrase in text for phrase in SENTINEL_RED_PHRASES),
         f"头一趟退 {first_code}（{summary(first_text)}），这一趟退 {code}、结局 {states}；输出末尾：{text.strip()[-400:]}")
    for name, fault, phrase in (("sentinel-rejudged-red", "rejudge-sentinels", "复用失灵："),
                                ("forced-verdict-differs-red", "forced-verdict-altered", "判定不稳："),
                                ("forced-still-reused-red", "forced-answered-reuse", "强制现判没起作用")):
        code, text = run_in_tree(tree, command, dict(seeded, **{FAULT_VARIABLE: fault}))
        report_cell(f"{name}：自证造的 {fault} 那一趟退 1、红「{phrase}」", code == EXIT_RED and phrase in text,
             f"退 {code}；输出末尾：{text.strip()[-400:]}")
    printer(f"long_run_item_verdicts 复用格（{script}）：6 格，红 {len(failures)} 格")
    return len(failures)


# ── 自证 ──

FAKE_ENTRY = "scripts/experiment-apparatus/long-run-fake-entry.sh"
FAKE_ENTRY_TEXT = r"""#!/usr/bin/env bash
# 自证用的假入口：两项 alpha、beta，照 long_run_item_verdicts.py 的 plan / record / finish 走一遍；产物是一行字。
# LONG_RUN_FAKE_RED_ITEM=<项> 那一项判红；LONG_RUN_FAKE_EDIT_MIDWAY=1 判完、finish 之前改 alpha_detail（键在跑的中途变了）
set -uo pipefail
root=$(cd "$(dirname "$0")/../.." && pwd)
helper="$root/scripts/experiment-apparatus/long_run_item_verdicts.py"
state=$(mktemp -d)
trap 'rm -rf -- "$state"' EXIT
printf 'alpha\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_alpha\tline:item=alpha\n' > "$state/requests.tsv"
printf 'beta\tscripts/experiment-apparatus/long_run_reuse_fake_entries.py::measure_beta\tline:item=beta\n' >> "$state/requests.tsv"
python3 "$helper" plan "$state/run" --script scripts/experiment-apparatus/long-run-fake-entry.sh --kind long-run-fake --root "$root" --requests "$state/requests.tsv" || exit $?
for item in alpha beta; do
  [[ "$(awk -F'\t' -v item="$item" '$1 == item {print $2}' "$state/run/decisions.tsv")" == reuse ]] && continue
  printf 'output of %s\n' "$item" > "$state/$item.out"
  red=()
  [[ "${LONG_RUN_FAKE_RED_ITEM:-}" == "$item" ]] && red=(--red)
  python3 "$helper" record "$state/run" "$item" 0 "$state/$item.out" ${red[@]+"${red[@]}"} || exit $?
done
if [[ -n "${LONG_RUN_FAKE_EDIT_MIDWAY:-}" ]]; then
  sed -i 's/^    return 1$/    return 5/' "$root/scripts/experiment-apparatus/long_run_reuse_fake_entries.py"
fi
python3 "$helper" finish "$state/run"
"""


def selftest():
    problem = prune_server_client.selftest_service_problem()
    if problem:
        print(f"  ✗ long_run_item_verdicts 自证没跑：{problem}")
        print(f"     → 怎么办：{prune_server_client.SELFTEST_SERVICE_REMEDY}")
        return EXIT_USAGE
    failures = 0
    work = tempfile.mkdtemp(prefix="long-run-item-verdicts-selftest-")
    try:
        template = os.path.join(work, "template")
        os.makedirs(os.path.join(template, "scripts", "experiment-apparatus"))
        with open(os.path.join(template, FAKE_ENTRY), "w", encoding="utf-8") as handle:
            handle.write(FAKE_ENTRY_TEXT)
        os.symlink(os.path.realpath(__file__), os.path.join(template, "scripts", "experiment-apparatus", os.path.basename(__file__)))
        command = ["bash", "{tree}/" + FAKE_ENTRY]
        failures += reuse_cells(SCRIPT_KEY, template, ["alpha", "beta"], os.path.join(work, "cells"), 0, command)

        def local_cell(name, passed, detail):
            nonlocal failures
            if passed:
                print(f"  ✓ {name}")
                return
            failures += 1
            print(f"  ✗ {name}：{detail}")
            print(f"     → 怎么办：照这一格的条件看 {SCRIPT_KEY} 的 plan / record / finish，再单独起它的 --selftest 到它退 0")

        def tree_of(name):
            where = os.path.join(work, name)
            tree = os.path.join(where, "tree")
            shutil.copytree(template, tree, symlinks=True)
            plant_fake_module(tree)
            return tree, {LIBRARY_DIRECTORY_VARIABLE: os.path.join(where, "library"), SUBSTITUTION_VARIABLE: None, FAULT_VARIABLE: None,
                          lib_reuse_checks.SENTINEL_COUNT_VARIABLE: ALL_SENTINELS, lib_reuse_checks.SENTINEL_SEED_VARIABLE: None}

        tree, settings = tree_of("red")
        first_code, first_text = run_in_tree(tree, command, dict(settings, LONG_RUN_FAKE_RED_ITEM="alpha"))
        code, text = run_in_tree(tree, command, settings)
        local_cell("red-withdraws-green：alpha 判红那一趟之后再起，alpha 现判、beta 复用",
                   first_code == 0 and lib_reuse_checks.states_from_output(first_text).get("alpha") == JUDGED_STATE and "· alpha：红，现判" in first_text
                   and code == 0 and lib_reuse_checks.states_from_output(text) == {"alpha": JUDGED_STATE, "beta": REUSED_STATE},
                   f"头一趟退 {first_code}，再起退 {code}、结局 {lib_reuse_checks.states_from_output(text)}；输出末尾：{text.strip()[-300:]}")

        tree, settings = tree_of("midway")
        first_code, first_text = run_in_tree(tree, command, dict(settings, LONG_RUN_FAKE_EDIT_MIDWAY="1"))
        edit_once(os.path.join(tree, FAKE_MODULE), "    return 5\n", "    return 1\n")
        code, text = run_in_tree(tree, command, settings)
        local_cell("key-changed-midway：跑的中途改了 alpha 的闭包，那一趟退 1、红「中途变了」；改回之后 alpha 现判、beta 复用",
                   first_code == EXIT_RED and "alpha 的键在这一趟中途变了" in first_text
                   and code == 0 and lib_reuse_checks.states_from_output(text) == {"alpha": JUDGED_STATE, "beta": REUSED_STATE},
                   f"中途改的那一趟退 {first_code}，改回之后退 {code}、结局 {lib_reuse_checks.states_from_output(text)}；"
                   f"输出末尾：{first_text.strip()[-200:]} / {text.strip()[-200:]}")

        tree, settings = tree_of("outside")
        outside = dict(settings, **{prune_server_client.TEMPORARY_SERVICE_VARIABLE: None, FAULT_VARIABLE: "rejudge-sentinels"})
        code, text = run_in_tree(tree, command, outside)
        local_cell("selftest-variables-outside-temporary-service：不在临时服务底下设了自证的变量，plan 退 2、点名只在自证里认",
                   code == EXIT_USAGE and "只在临时 prune-server 底下的自证里认" in text, f"退 {code}；输出末尾：{text.strip()[-300:]}")

        tree, settings = tree_of("unavailable")
        # 连接地址改指到不在的 socket 与放掉的端口（临时服务的标记照旧，客户端照认）；只在自证里认的变量（含自证本身带着的弄坏开关）这一格全不设，
        # selftest_service_problem 不因地址对不上先拒
        unreachable = dict(settings, **{name: None for name in SELFTEST_ONLY_VARIABLES},
                           **{prune_server_client.SOCKET_VARIABLE: os.path.join(work, "no-such.sock"),
                              prune_server_client.TCP_ADDRESS_VARIABLE: "127.0.0.1:1",
                              prune_server_client.RECONNECT_LIMIT_VARIABLE: "2"})
        code, text = run_in_tree(tree, command, unreachable)
        local_cell("service-unavailable：prune-server 连不上，plan 退 75", code == prune_server_client.EXIT_UNAVAILABLE,
                   f"退 {code}；输出末尾：{text.strip()[-300:]}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    print(f"long_run_item_verdicts 自证：10 格，红 {failures} 格")
    return EXIT_RED if failures else 0


# ── 命令行 ──

def options_of(arguments, names, flags=()):
    """交 ({选项: 值}, [位置参数])；认不得的 -- 选项、缺值都算用法错。"""
    given, positional, index = {}, [], 0
    while index < len(arguments):
        word = arguments[index]
        if word in flags:
            given[word] = True
            index += 1
        elif word in names:
            if index + 1 >= len(arguments):
                raise UsageError(f"{word} 后面缺值")
            given[word] = arguments[index + 1]
            index += 2
        elif word.startswith("--"):
            raise UsageError(f"认不得的选项 {word}")
        else:
            positional.append(word)
            index += 1
    return given, positional


def main(arguments):
    try:
        require_selftest_variables_allowed()
        unknown = [name for name in os.environ.get(BREAK_VARIABLE, "").split(",") if name and name not in BREAK_SWITCHES]
        unknown += [name for name in os.environ.get(FAULT_VARIABLE, "").split(",") if name and name not in FAULTS]
        if unknown:
            raise UsageError(f"认不得的弄坏开关或自证失灵：{'、'.join(unknown)}")
        command, rest = (arguments[0], arguments[1:]) if arguments else ("", [])
        if command == "--selftest" and not rest:
            return selftest()
        if command == "plan":
            given, positional = options_of(rest, ("--script", "--kind", "--root", "--requests"))
            if len(positional) != 1 or len(given) != 4:
                raise UsageError("plan <状态目录> --script <入口> --kind <装置名> --root <仓根> --requests <请求文件>")
            os.makedirs(positional[0], exist_ok=True)
            with open(given["--requests"], encoding="utf-8", errors="surrogateescape") as handle:
                text = handle.read()
            plan(positional[0], given["--script"], given["--kind"], given["--root"], text, given["--requests"])
            return 0
        if command == "record":
            given, positional = options_of(rest, (), ("--red",))
            if len(positional) != 4 or not re.fullmatch(r"-?[0-9]+", positional[2]):
                raise UsageError("record <状态目录> <项> <退出码> <产物文件或 -> [--red]")
            output = b""
            if positional[3] != "-" and os.path.isfile(positional[3]):
                with open(positional[3], "rb") as handle:
                    output = handle.read()
            record(positional[0], positional[1], int(positional[2]), output, "--red" not in given)
            return 0
        if command == "finish":
            if len(rest) != 1:
                raise UsageError("finish <状态目录>")
            return finish(rest[0])
        if command == "plant-fake-entries" and len(rest) == 1:
            plant_fake_module(rest[0])
            return 0
        if command == "selftest-service" and not rest:
            problem = prune_server_client.selftest_service_problem()
            if problem:
                print(f"  ✗ 自证没跑：{problem}")
                print(f"     → 怎么办：{prune_server_client.SELFTEST_SERVICE_REMEDY}")
                return EXIT_USAGE
            return 0
        if command == "reuse-cells":
            if "--" not in rest:
                raise UsageError("reuse-cells … -- <起法…>")
            split = rest.index("--")
            given, positional = options_of(rest[:split], ("--script", "--template", "--items", "--work", "--green-exit"), ("--fake-judge",))
            items = given.get("--items", "").split(",")
            if positional or not rest[split + 1:] or len(items) != 2 or not all(items) or any(name not in given for name in ("--script", "--template", "--work")):
                raise UsageError("reuse-cells --script <入口> --template <样板树> --items <项甲>,<项乙> --work <目录> [--green-exit <码>] [--fake-judge] -- <起法…>")
            os.makedirs(given["--work"], exist_ok=True)
            red = reuse_cells(given["--script"], given["--template"], items, given["--work"], int(given.get("--green-exit", "0")), rest[split + 1:],
                              fake_judge="--fake-judge" in given)
            return EXIT_RED if red else 0
        raise UsageError("用法见 scripts/experiment-apparatus/long_run_item_verdicts.py 文件头")
    except UsageError as error:
        print(f"  ✗ long_run_item_verdicts：{error}", file=sys.stderr)
        print("     → 怎么办：照 scripts/experiment-apparatus/long_run_item_verdicts.py 文件头的写法改", file=sys.stderr)
        return EXIT_USAGE


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
