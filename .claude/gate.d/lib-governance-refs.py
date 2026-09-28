#!/usr/bin/env python3
"""治理文档、kb 与仓根 README.md 里对门禁的指向还指不指得到：门禁 doc-registries 的 governance-refs 那一格调它，不单独成阶段。

治理文档：CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、.claude/agents/*.md、.claude/rules/*.md、.claude/skills/*/SKILL.md。
现状文字：治理文档加 .claude/kb/ 下全部 .md 与仓根 README.md，去掉「## 历史版本」那一节（到下一个「## 」标题或文件末尾）与
decisions-history.md、experiments-history.md、CHANGELOG.md 三种变更史；research/prompts/、records/、.claude/warnings/ 不在射程里（冻结的证据与记事）。
仓根 README.md 只判 ④ ⑤（它不是治理文档，① ② ③ 不扫它）。

只扫治理文档的三类（原样）：
  ① 门禁号：「门禁 65 号」「54、55、57、59 号」这类两位数加「号」，门禁目录里要有 <号>-*.sh。
     「、」或「/」连着的每个号都判（「20 / 21 号」判 20 与 21）；前面紧挨「第」「月」「日」（可隔空白）的不算门禁号（「第 13 号」「9 月 23 号」），
     紧挨「-」「–」的也不算（「2026-09-26 号」；「54–59 号」这种区间写法不判，要判就逐个写出来）。
     门禁目录取本文件所在的目录，不取被扫的仓根：样本目录里没有门禁脚本。去编号之前的旧号不归这一条，归 ⑤。
  ② 反引号里的仓内路径：反引号里按空白切开的每个词（命令里的脚本路径也算），ASCII 写成、中间带 `/`、
     而且第一段在解析基准下现存、或末段带常见的文件扩展名（`origin/master`、`text/plain` 两样都不满足，不算仓内路径）。
     解析基准：仓根、所在文件的目录、`.claude/`、`.claude/kb/`，外加同一个反引号里 `cd 目录` 之后的那个目录；一处在就算指得到。
     跳过（计数报在末行）：被 .gitignore 挡着的（规范副本、构建目录）、`../` `/` `~` `http` 起头的、`refs/` 起头的 git 引用、带占位（`<` `*` `{`）的。
     已归档的写裸文件名（`.claude/agent-common.md`「找不到历史实验的数据」那一条），不带目录，不在射程里。
     指向去编号之前的门禁文件（`gate.d/<旧号>-…`）的不归这一条，归 ⑤。
  ③ `文件「小节」`：文件在仓里、而「小节」里的字（去掉空白、反引号、星号、「」之后）在那份文件的全文里一处都找不到。
     判的是「全文任意位置出现」，不只看标题：定义常点正文里的一句；小节改了名而原名的字还散在正文里时抓不到。
     小节名里嵌套「」的（「「无效」那一栏」），只核到第一个」之前那一截。

扫现状文字的两类（2026-09-28 门禁去编号之后加，判决 research/prompts/gate-shrink-r1-main-verification.md 第 4 问「按全名指门禁从此没人判」）：
  ④ 按全名指门禁与格：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼。
     「门禁 <名>」（名是小写字母、数字与连字符、至少带一个连字符，可带 .sh、可套反引号）里的名要是门禁目录顶层的一道门禁，
     或上游规范副本 .claude/singlefs-ai-sop/scripts/ 下的一份脚本（共享门禁的阶段，「门禁 doc-lint」这类）；
     「<门禁名> 的 <格名> 格」（「那一格」「这一格」同）与反引号里「gate.d/<门禁名>.sh … --check <格名>[,<格名>…]」里的格名，
     要在那道门禁文件头的格名表里（`# gate-cell:` 行，经 research/scripts/gate-structure-check.py 的 header_cells 读）。
     「<名> 的 <x> 格」里 <名> 不是门禁目录里的门禁的不判（分不清是不是在说门禁）。
  ⑤ 用旧编号称呼门禁：「NN 号」（排除与 ① 相同）、「门禁 NN」、「gate.d/NN-…」（含 gate.d/fixtures/NN-…），NN 是下面 OLD_GATES 里的旧号，54 除外
     （54 号的全名就是 54-layer0-replay）。出路给那个旧号今天在哪道门禁、哪几格。
     「NN 号」「门禁 NN」两种写法先把带目录的文件路径与「日期-」起头的记录文件名遮掉再找（`records/2026-09-28-门禁59号提速与双机分片.md`
     这种文件名里的号不是在称呼门禁；① 同样遮）；「gate.d/NN-」那一种就是路径，不遮。
     OLD_GATES 照调度记录 records/2026-09-28-门禁59号提速与双机分片.md「旧编号与现在的门禁」那张表，外加表里漏了的 96 号
     （实验源码纪律，第一轮并进 33 号，今天是 code-source-discipline 的 reading-discipline 格）；
     表里点名的门禁与格本身也现判：指到门禁目录里不在的门禁或格，这一段自己判红（门禁再改名时这张表要跟着改）。

判不到、逐条列进「没判的」（不算红）：③ 里文件解析不到的（多半是只写了文件名、住在四个基准之外）；
② 里不全是路径字符（非 ASCII、引号、`$` 这类）、中间带 `/`、又指不到的记号（悬空的中文文件名路径与 `NN-简称.md` 这类占位分不开；指得到的照常算判过）。
够不着的：不带「门禁」二字也不带「的 … 格」的裸名（「跑 doc-kb」）；裸写的旧门禁文件名（「`47-research-script-selftests.sh`」，不带 gate.d/）。

输出：每处红一行「文件:行: 说明」；「没判的」逐条一行；末两行
「扫 N 份治理文档：门禁号 A 处、路径 B 处、小节 C 处；没判 D 处；跳过 E 处（被 .gitignore 挡着 F、不像仓内路径 G、仓外或带占位 H）」与
「扫 M 份现状文字（治理文档、kb 与仓根 README.md，历史版本节与变更史除外）：门禁全名 I 处、格名 J 处、旧编号 K 处」。
退出码：0 全指得到；1 有指不到的；3 一份治理文档都没扫到（没有对象，不许当通过）；4 本脚本自己出错（读不了某份文件这类，打出 traceback）；
python 起不来时退 2（打不开脚本）。调用方把 0、1、3 之外的都当「没跑成」。

弄坏开关 GOVERNANCE_REFS_BREAK=<项>（doc-registries 的样本判错）：
  old-numbers-ignored   ⑤ 整条不判：governance-refs-red 里「用旧编号称呼门禁」那几行 want 找不到
  names-ignored         ④ 整条不判：governance-refs-red 里「不是门禁目录里的门禁」「格名表里没有」那几行 want 找不到
  history-scanned       「## 历史版本」一节与变更史也扫：governance-refs-green 判红
  readme-skipped        仓根 README.md 不进现状文字：governance-refs-red 里「README.md:」那两行 want 找不到，
                        governance-refs-green 的现状文字份数与全名、格名处数对不上
"""
import glob
import importlib.util
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True

GATE_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_OF_GATES = os.path.dirname(os.path.dirname(GATE_DIRECTORY))
UPSTREAM_SCRIPTS = os.path.join(REPOSITORY_OF_GATES, ".claude", "singlefs-ai-sop", "scripts")
STRUCTURE_CHECK = os.path.join(REPOSITORY_OF_GATES, "research", "scripts", "gate-structure-check.py")
BROKEN = os.environ.get("GOVERNANCE_REFS_BREAK", "")
CARRIER_PATTERNS = ["CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md",
                    ".claude/agents/*.md", ".claude/rules/*.md", ".claude/skills/*/SKILL.md"]
KB_PATTERN = ".claude/kb/**/*.md"
# 仓根的 README.md：只进现状文字（④ ⑤），不当治理文档
README_PATTERNS = ["README.md"]
HISTORY_FILES = ("decisions-history.md", "experiments-history.md", "CHANGELOG.md")
HISTORY_HEADING = re.compile(r"^##\s*历史版本")
LEVEL_TWO_HEADING = re.compile(r"^##\s")
GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.\-–])((?:[0-9]{2}\s*[、/]\s*)*[0-9]{2})\s*号")
NOT_A_GATE_BEFORE = re.compile(r"(第|月|日)\s*$")
GATE_WORD_NUMBER = re.compile(r"门禁\s*`?([0-9]{2})(?![0-9])")
OLD_GATE_PATH = re.compile(r"gate\.d/(?:fixtures/)?([0-9]{2})-")
NAME = r"[0-9a-z][a-z0-9]*(?:-[a-z0-9]+)+"
GATE_BY_NAME = re.compile(rf"门禁\s*`?({NAME})(?:\.sh)?`?(?![A-Za-z0-9_-])")
CELL_OF_GATE = re.compile(rf"(?<![A-Za-z0-9_-])({NAME})(?:\.sh)?`?\s*的\s*`?([a-z0-9][a-z0-9-]*)`?\s*(?:那一|这一)?格")
BACKTICK = re.compile(r"`([^`\n]+)`")
REPO_PATH = re.compile(r"^[A-Za-z0-9_.\-/]+$")
FILE_WITH_SECTIONS = re.compile(r"`?([^\s`「」]+\.(?:md|py|sh))`?\s*(?:的)?\s*((?:「[^」]+」(?:、|与|和)?)+)")
DECORATION = re.compile(r"[\s`*「」]")
BASE_DIRECTORIES = ("", ".claude", ".claude/kb")
KEPT_NUMBER = 54

# 旧号 → (今天的门禁, 那一道里的哪几格；空元组表示整道或拆得说不清，只给门禁名)
OLD_GATES = {
    10: ("doc-registries", ("experiment-refs", "decision-refs", "declared-counts", "governance-refs")),
    11: ("doc-process-records", ("batch-scope",)),
    12: ("doc-text", ("prime-marks", "clock-times")),
    13: ("code-source-discipline", ("vague-names",)),
    15: ("checker-tier-research-build-and-replay", ("research-unit-tests",)),
    16: ("doc-decisions", ("freeze-layer-membership",)),
    19: ("doc-process-records", ("verdict-names-local-samples",)),
    20: ("doc-decisions", ()),
    21: ("doc-decisions", ("decision-items-sync",)),
    22: ("doc-decisions", ("item-ref-status",)),
    24: ("doc-decisions", ("status-redundancy",)),
    27: ("code-source-discipline", ("format-constants",)),
    28: ("doc-decisions", ("cross-decision-status",)),
    29: ("doc-decisions", ("settled-item-self-open",)),
    30: ("doc-decisions", ("entry-added", "shape", "status-sync")),
    31: ("doc-decisions", ("blocking-verdict",)),
    32: ("doc-registries", ("field-refs",)),
    33: ("code-source-discipline", ("mutation-tables",)),
    34: ("doc-experiments", ("index-sync",)),
    35: ("doc-decisions", ("user-verdict-owed",)),
    36: ("doc-registries", ("invariant-count-elsewhere",)),
    37: ("doc-decisions", ("decision-summary-width",)),
    38: ("doc-registries", ("field-projection",)),
    39: ("doc-registries", ("field-table-sums",)),
    40: ("doc-experiments", ("results-cited",)),
    42: ("doc-registries", ("first-txn-hooks",)),
    43: ("doc-registries", ("table-shape",)),
    44: ("doc-decisions", ("settled-ref-says-open",)),
    47: ("code-tooling", ("research-script-selftests", "research-script-selftest-coverage")),
    48: ("doc-decisions", ()),
    49: ("doc-decisions", ()),
    50: ("code-tooling", ("rules-manifest",)),
    51: ("doc-registries", ("admission-terms",)),
    52: ("doc-registries", ("segment-registry",)),
    53: ("doc-registries", ("format-const-placeholders",)),
    55: ("checker-tier-qemu-device-streams", ("device-streams",)),
    56: ("doc-process-records", ("crates-adversarial-review",)),
    57: ("checker-tier-lkmm", ("litmus-verdicts",)),
    58: ("doc-process-records", ("implementation-premise",)),
    59: ("checker-tier-crates-mutation-replay", ("crates-mutation-rows",)),
    60: ("doc-decisions", ("stale-open-items",)),
    61: ("doc-decisions", ("settled-same-file",)),
    62: ("code-tooling", ("stage-owners",)),
    63: ("code-tooling", ("agent-write-scope",)),
    64: ("code-tooling", ("change-range-single-source",)),
    66: ("doc-process-records", ("abandoned-rounds",)),
    67: ("doc-registries", ("closeout-collects-open",)),
    68: ("doc-process-records", ("knowledge-sync",)),
    69: ("doc-experiments", ("evidence-in-repo",)),
    70: ("doc-registries", ("citations",)),
    72: ("doc-process-records", ("agent-def-adversarial-review",)),
    73: ("code-tooling", ("research-gate-lint",)),
    74: ("harness-model-differential-and-scenarios", ("model-fast-tier", "model-reuse-sampling", "model-rollback-sampling",
                                                       "model-allocation-record-wall", "model-unit-area-wall",
                                                       "model-unit-area-wall-admission-judged")),
    75: ("doc-experiments", ("decision-links",)),
    76: ("doc-registries", ("second-txn-hooks",)),
    77: ("harness-test-environment", ("machine-clean",)),
    78: ("doc-registries", ("paid-cited-tests",)),
    79: ("doc-registries", ("tree-table-reserve",)),
    80: ("code-source-discipline", ("absolute-assertions",)),
    81: ("doc-registries", ("audit-contradictions",)),
    82: ("code-source-discipline", ("clause-enums",)),
    84: ("doc-experiments", ("verdict-false-named",)),
    85: ("doc-experiments", ("repro-command",)),
    86: ("doc-experiments", ("experiment-orphans",)),
    87: ("checker-tier-research-build-and-replay", ("experiment-replay",)),
    88: ("doc-experiments", ("quoted-result-lines",)),
    89: ("doc-registries", ("closeout-row27-preconditions",)),
    90: ("doc-text", ("term-renames",)),
    91: ("doc-experiments", ("archive-past-rounds",)),
    92: ("checker-independence-and-sync", ("layout-checker-sync",)),
    93: ("code-source-discipline", ("feature-bits",)),
    94: ("checker-independence-and-sync", ("checker-implementation-disjoint",)),
    95: ("code-tooling", ("fixture-claims",)),
    96: ("code-source-discipline", ("reading-discipline",)),
    97: ("doc-registries", ("invariant-anchors",)),
    98: ("code-tooling", ("kb-registry",)),
    99: ("doc-experiments", ("multipath-registry",)),
}
# 14 号拆进了三道，单列
SPLIT_OLD_GATES = {
    14: "拆开了：「一条用例一个场景」是门禁 harness-model-differential-and-scenarios 的 one-scenario 格，"
        "「测试文件不按里程碑起名」是门禁 code-source-discipline 的 test-file-names 格，"
        "「崩溃枚举用例住 checker 档且登记」「测试文件声明模块」归门禁 54-layer0-replay",
}


def existing_gate_numbers():
    numbers = set()
    for name in os.listdir(GATE_DIRECTORY):
        match = re.match(r"([0-9]+)-.*\.sh$", name)
        if match:
            numbers.add(int(match.group(1)))
    return numbers


def load_structure_check():
    specification = importlib.util.spec_from_file_location("gate_structure_check", STRUCTURE_CHECK)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def local_gates(structure):
    """门禁目录顶层每道门禁的全名（文件名去掉 .sh）→ 它文件头格名表里的格名集合。"""
    gates = {}
    for name in sorted(os.listdir(GATE_DIRECTORY)):
        path = os.path.join(GATE_DIRECTORY, name)
        if name.endswith(".sh") and os.path.isfile(path):
            gates[name[:-len(".sh")]] = {cell for _number, cell, _title in structure.header_cells(path)}
    return gates


def upstream_stage_names():
    if not os.path.isdir(UPSTREAM_SCRIPTS):
        return set()
    return {os.path.splitext(name)[0] for name in os.listdir(UPSTREAM_SCRIPTS)
            if name.endswith((".sh", ".py")) and os.path.isfile(os.path.join(UPSTREAM_SCRIPTS, name))}


def today_name_of(number):
    if number in SPLIT_OLD_GATES:
        return SPLIT_OLD_GATES[number]
    gate, cells = OLD_GATES[number]
    if cells:
        return f"门禁 {gate} 的 {'、'.join(cells)} 格"
    return f"门禁 {gate}"


def mapping_problems(gates):
    problems = []
    for number, (gate, cells) in sorted(OLD_GATES.items()):
        if gate not in gates:
            problems.append(f"{os.path.basename(__file__)}: OLD_GATES 里 {number} 号指的门禁 {gate} 不在门禁目录里")
            continue
        for cell in cells:
            if cell not in gates[gate]:
                problems.append(f"{os.path.basename(__file__)}: OLD_GATES 里 {number} 号指的 {gate} 的 {cell} 格不在那道门禁的格名表里")
    return problems


def ignored_by_git(path):
    result = subprocess.run(["git", "check-ignore", "-q", "--no-index", path],
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return result.returncode == 0


def bases_for(carrier, changed_directories):
    return list(BASE_DIRECTORIES) + [os.path.dirname(carrier)] + list(changed_directories)


def resolve(path, carrier, changed_directories=()):
    bare = path.rstrip("/")
    for base in bases_for(carrier, changed_directories):
        candidate = os.path.normpath(os.path.join(base, bare))
        if os.path.exists(candidate):
            return candidate
    return None


FILE_EXTENSION = re.compile(r"\.(md|sh|py|rs|tsv|toml|json|txt|out|lock)$")


def looks_like_repo_path(path, carrier, changed_directories):
    """第一段在解析基准下现存，或末段带常见的文件扩展名，才当仓内路径：`origin/master`、`text/plain` 两样都不满足。"""
    first = path.split("/", 1)[0]
    if any(os.path.exists(os.path.join(base, first)) for base in bases_for(carrier, changed_directories)):
        return True
    return bool(FILE_EXTENSION.search(path.rstrip("/")))


OUTSIDE_TOKEN = "outside"


def path_token(token):
    """反引号里的一个词若形如一条仓内路径，返回去掉 `:行号`、`@标题`、`~正则` 之后的路径；
    中间不带 `/` 的返回 None（不是路径，不计数）；起头在仓外或带占位的返回 OUTSIDE_TOKEN（计进跳过）。"""
    path = re.split(r"[:@~]", token, maxsplit=1)[0]
    if "/" not in path.rstrip("/"):
        return None
    if path.startswith(("../", "/", "~", "refs/", "http")):
        return OUTSIDE_TOKEN
    if any(mark in path for mark in "<*{"):
        return OUTSIDE_TOKEN
    return path


def is_old_gate_path(path):
    match = OLD_GATE_PATH.search(path)
    return match is not None and int(match.group(1)) != KEPT_NUMBER and int(match.group(1)) in OLD_GATES.keys() | SPLIT_OLD_GATES.keys()


# 带目录的文件路径与「日期-」起头的记录文件名整个遮掉再找号：
# `records/2026-09-28-门禁59号提速与双机分片.md` 这种文件名里的「门禁59号」不是在称呼门禁。
PATH_LIKE = re.compile(r"[^\s`「」（）()，。；：、,;]*(?:/[^\s`「」（）()，。；：、,;]*\.(?:md|sh|py|tsv|txt|out|rs)"
                       r"|[0-9]{4}-[0-9]{2}-[0-9]{2}-[^\s`「」（）()，。；：、,;]*\.md)")


def without_paths(line):
    return PATH_LIKE.sub(lambda match: " " * len(match.group(0)), line)


def gate_numbers_in(line):
    """一行里「NN 号」的每个号：[(起点, 号)]，前面紧挨「第」「月」「日」的不算，文件路径里的不算。"""
    found = []
    line = without_paths(line)
    for match in GATE_NUMBER.finditer(line):
        if NOT_A_GATE_BEFORE.search(line[:match.start()]):
            continue
        offset = match.start(1)
        for piece in re.finditer(r"[0-9]{2}", match.group(1)):
            found.append((offset + piece.start(), int(piece.group(0))))
    return found


def current_text_lines(carrier):
    """现状文字：跳过「## 历史版本」一节；变更史整份跳过。返回 [(行号, 行)]。"""
    if os.path.basename(carrier) in HISTORY_FILES and BROKEN != "history-scanned":
        return []
    with open(carrier, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().split("\n")
    kept, in_history = [], False
    for number, line in enumerate(lines, 1):
        if LEVEL_TWO_HEADING.match(line):
            in_history = bool(HISTORY_HEADING.match(line)) and BROKEN != "history-scanned"
        if not in_history:
            kept.append((number, line))
    return kept


def judge_governance(carriers, gates_on_disk, problems, unjudged, counts, skipped):
    """① ② ③：只扫治理文档。"""
    for carrier in carriers:
        with open(carrier, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
        for number, line in enumerate(lines, 1):
            for _start, gate in gate_numbers_in(line):
                if gate != KEPT_NUMBER and (gate in OLD_GATES or gate in SPLIT_OLD_GATES):
                    continue   # 旧号归 ⑤
                counts["gate"] += 1
                if gate not in gates_on_disk:
                    problems.append(f"{carrier}:{number}: 「{gate:02d} 号」在门禁目录里没有 {gate:02d}-*.sh")
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                # 命令里 `cd 目录 && …` 之后的相对路径按那个目录解析
                changed_directories = [words[i + 1] for i in range(len(words) - 1) if words[i] == "cd"]
                for word in words:
                    path = path_token(word)
                    if path is None:
                        continue
                    if path is OUTSIDE_TOKEN:
                        skipped["outside"] += 1
                        continue
                    if is_old_gate_path(path):
                        continue   # 旧门禁文件归 ⑤
                    if not REPO_PATH.match(path):
                        # 非 ASCII 的记号：指得到就算判过；指不到的与 `NN-简称.md` 这类占位分不开，列进没判的
                        if resolve(path, carrier, changed_directories) is not None:
                            counts["path"] += 1
                        else:
                            unjudged.append(f"{carrier}:{number}: `{word}` 不全是路径字符（非 ASCII、引号、`$` 这类）、指不到，分不清是占位还是悬空，路径没判")
                        continue
                    if not looks_like_repo_path(path, carrier, changed_directories):
                        skipped["not_repo_path"] += 1
                        continue
                    if ignored_by_git(path):
                        skipped["ignored"] += 1
                        continue
                    counts["path"] += 1
                    if resolve(path, carrier, changed_directories) is None:
                        problems.append(f"{carrier}:{number}: `{match.group(1)}` 里的 {path} 在仓里不存在")
            for match in FILE_WITH_SECTIONS.finditer(line):
                target = resolve(match.group(1), carrier)
                if target is None or not os.path.isfile(target):
                    unjudged.append(f"{carrier}:{number}: {match.group(1)}{match.group(2)} 的文件在四个解析基准下都找不到，小节没判")
                    continue
                with open(target, encoding="utf-8") as handle:
                    haystack = DECORATION.sub("", handle.read())
                for section in re.findall(r"「([^」]+)」", match.group(2)):
                    counts["section"] += 1
                    if DECORATION.sub("", section) not in haystack:
                        problems.append(f"{carrier}:{number}: {match.group(1)}「{section}」在那份文件里一处都找不到")


def judge_current_text(carriers, gates, upstream, problems, counts):
    """④ ⑤：扫现状文字。"""
    known_names = set(gates) | upstream
    for carrier in carriers:
        for number, line in current_text_lines(carrier):
            if BROKEN != "old-numbers-ignored":
                reported = set()
                spans = [(start, gate) for start, gate in gate_numbers_in(line)]
                spans += [(match.start(1), int(match.group(1))) for match in GATE_WORD_NUMBER.finditer(without_paths(line))]
                spans += [(match.start(1), int(match.group(1))) for match in OLD_GATE_PATH.finditer(line)]
                for start, gate in sorted(spans):
                    if start in reported or gate == KEPT_NUMBER or (gate not in OLD_GATES and gate not in SPLIT_OLD_GATES):
                        continue
                    reported.add(start)
                    counts["old"] += 1
                    problems.append(f"{carrier}:{number}: 用旧编号称呼门禁（…{line[max(0, start - 12):start + 16].strip()}…）：{gate} 号今天是{today_name_of(gate)}")
            if BROKEN == "names-ignored":
                continue
            for match in GATE_BY_NAME.finditer(line):
                name = match.group(1)
                if re.match(r"[0-9]{2}-", name) and name != f"{KEPT_NUMBER}-layer0-replay" and int(name[:2]) in OLD_GATES.keys() | SPLIT_OLD_GATES.keys():
                    continue   # 带旧号的文件名，⑤ 不判裸名，这里也不重复报
                counts["name"] += 1
                if name not in known_names:
                    problems.append(f"{carrier}:{number}: 「门禁 {name}」不是门禁目录里的门禁（也不是上游共享门禁的脚本）")
            for match in CELL_OF_GATE.finditer(line):
                gate, cell = match.group(1), match.group(2)
                if gate not in gates:
                    continue
                counts["cell"] += 1
                if cell not in gates[gate]:
                    problems.append(f"{carrier}:{number}: 「{gate} 的 {cell} 格」：{gate} 的格名表里没有 {cell}")
            for match in BACKTICK.finditer(line):
                words = match.group(1).split()
                gate = None
                for index, word in enumerate(words):
                    found = re.search(r"gate\.d/([^/\s]+)\.sh$", word)
                    if found:
                        gate = found.group(1)
                    elif word in ("--check",) and gate in gates and index + 1 < len(words):
                        for cell in words[index + 1].split(","):
                            if not cell or any(mark in cell for mark in "<*{…"):
                                continue   # 占位（`--check <格名>`）不判
                            counts["cell"] += 1
                            if cell not in gates[gate]:
                                problems.append(f"{carrier}:{number}: `{match.group(1)}`：{gate} 的格名表里没有 {cell}")


def main():
    carriers = sorted({found for pattern in CARRIER_PATTERNS for found in glob.glob(pattern)})
    if not carriers:
        print("  ✗ 一份治理文档都没扫到（CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md、agents、rules、skills）")
        print("     → 怎么办：在仓库根上跑；治理文档搬了家的话，改本文件的 CARRIER_PATTERNS。")
        return 3
    structure = load_structure_check()
    gates = local_gates(structure)
    problems, unjudged = [], []
    counts = {"gate": 0, "path": 0, "section": 0, "name": 0, "cell": 0, "old": 0}
    skipped = {"ignored": 0, "not_repo_path": 0, "outside": 0}
    problems.extend(mapping_problems(gates))
    judge_governance(carriers, existing_gate_numbers(), problems, unjudged, counts, skipped)
    readmes = set() if BROKEN == "readme-skipped" else {found for pattern in README_PATTERNS for found in glob.glob(pattern)}
    current_carriers = sorted(path for path in set(carriers) | set(glob.glob(KB_PATTERN, recursive=True)) | readmes
                              if BROKEN == "history-scanned" or os.path.basename(path) not in HISTORY_FILES)
    judge_current_text(current_carriers, gates, upstream_stage_names(), problems, counts)
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    if problems:
        print("     → 怎么办：门禁一律按门禁目录顶层的文件名去掉 .sh 称呼（「门禁 doc-registries」「门禁 doc-registries 的 governance-refs 格」）；"
              "旧编号照上面每一行给的今天的名字改（对照表在 records/2026-09-28-门禁59号提速与双机分片.md「旧编号与现在的门禁」），"
              "格名用 bash .claude/gate.d/<门禁>.sh --list 看；门禁号改成现存的号或共享 gate.sh 里那一道的名字；"
              "路径改成现存的，已归档的写裸文件名；小节按那份文件今天的标题改。")
    for item in unjudged:
        print(f"  没判的：{item}")
    print(f"  扫 {len(carriers)} 份治理文档：门禁号 {counts['gate']} 处、路径 {counts['path']} 处、小节 {counts['section']} 处；没判 {len(unjudged)} 处；跳过 {sum(skipped.values())} 处（被 .gitignore 挡着 {skipped['ignored']}、不像仓内路径 {skipped['not_repo_path']}、仓外或带占位 {skipped['outside']}）")
    print(f"  扫 {len(current_carriers)} 份现状文字（治理文档、kb 与仓根 README.md，历史版本节与变更史除外）：门禁全名 {counts['name']} 处、格名 {counts['cell']} 处、旧编号 {counts['old']} 处")
    return 1 if problems else 0


if __name__ == "__main__":
    try:
        exit_code = main()
    except Exception:
        import traceback
        traceback.print_exc()
        print("  ✗ lib-governance-refs.py 自己出错（上面是 traceback）——这一段没判完")
        print("     → 怎么办：按 traceback 修本脚本或那份读不了的文件；没判完就是没判，不许当成判过了。")
        exit_code = 4
    sys.exit(exit_code)
