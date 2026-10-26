#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
r"""全仓术语改名：把一个概念的旧名换成新名，源码、文档、冻结提示与留存产物一起换。

用法：
  sweep-term.py --check [<仓根>] [--defer <路径>]…   # 全仓还剩多少处旧名（豁免的不算），剩了就退出码 1
  sweep-term.py --apply [<仓根>] [--defer <路径>]…   # 换成新名
  sweep-term.py --selftest
选项与位置参数分开解析，次序随意：`--defer` 吃掉紧跟它的那一个参数（分批落地先放着的路径，可写多次），
剩下不以 `-` 起头的至多一个，是仓根；不给仓根时取当前目录所在 git 仓的根（`git rev-parse --show-toplevel`），
当前目录不在 git 仓里就退 2。认不出的选项、`--defer` 后面没有路径、给了不止一个仓根，都退 2。

为什么源码与产物要一起换：产物里的串来自源码里的串，只换一边，`replay.sh` 的 exact 判据下次复跑必然「对不上」；
两边一起换，复跑出来的字节与换后的产物仍然逐字节一致 —— 这一条要靠复跑证明，不是声称。

登记表（`.claude/kb/term-renames.md`）的「匹配」一列认三种：
  整串    「旧」一列逐字比
  词边界  「旧」一列两头加 \b（以 `_` 起头的只加尾、以 `_` 收尾的只加头），用在缩写上
  正则    「旧」一列是一条 Python 正则，用在同一个串在别处是另一个量的时候：例
          `(?<![每第内外里叶二])(?<![0-9] )层 1(?![0-9])` 打「层 1 常规」「层 1、层 2」，不打「每层 134」「第 1 层 2 个」「外层 16.33 ms」「内部层 110」。
          竖线写成 `\|`（表格的转义，读表时还原成 `|`）；不许有捕获组（分组用 `(?:…)`）、不许能匹配空串、标志只写带范围的 `(?i:…)`；
          「新」一列照字面换上，不认 `\1` 这类回引。写不对退 2。
「旧」「新」两列外面的反引号读表时去掉。

豁免写在项目根的 `.claude/term-rename-exempt`，一行一条、`#` 后写为什么，两种写法：
  <相对路径或目录>  # 为什么             整份文件或整个目录不扫
  <相对路径>:「<行内锚>」  # 为什么      只豁免那份文件里含这段原文的那一行，同一份文件别的行照扫、照换
行内锚逐字比（不是正则），锚里可以带「」，锚收尾的那个 」后面接空白加 # 或接行尾；它要在那份文件里恰好命中一行。
指向不存在的路径、行内锚命中 0 行或不止一行，都退 2（不起作用的豁免项会让人以为那批文件已经被绕开了，判据与 `.claude/doc-lint-exclude` 同）。

扫哪些文件：根在 git 工作树里时只扫 `git ls-files --cached --others --exclude-standard` 列出的（跟踪的，加未跟踪而未忽略的），
git 忽略的（`multi-host.env` 这类本机私有的）不读、不改；不在 git 工作树里时走整棵目录树
（跳过 .git、target、node_modules）。根下有 `.git` 而 git 起不来、判不出是不是工作树，或 `git ls-files` 失败：退 2，不退回走整棵树。

默认不扫的样本（写死在代码里，不进 `.claude/term-rename-exempt`）：门禁 doc-text 的 term-renames 格自己的判别力样本，
即 `.claude/gate.d/fixtures/doc-text.sh/` 下目录名以 `term-renames-` 起头的那几个（`OWN_CELL_FIXTURE_PARENT`、`OWN_CELL_FIXTURE_PREFIX`）。
那几份样本的登记表「旧」一列与带旧名的文件就是被换的那一侧，扫了它们，红样本换成绿样本，判别力归零，而那一格照样绿
（checks-owed C441）。`--check` 与 `--apply` 都不扫，成功句与失败句报出排除了几个目录；
别的格的样本目录、别的门禁文件名下叫 `term-renames-*` 的目录照扫。

路径形态的命中只报不换：旧名所在的那一段不间断文字（以空白、引号、反引号、括号、中英文逗号句号分号冒号顿号为界，见 PATH_TOKEN_DELIMITERS）以 `~/`、`/`、`./`、`../` 起头、
是带 `://` 的网址、带 `/` 而至少一段像文件名，或整段像一个文件名（`名字.md` / `.rs` / `.toml` / `.sh` / `.py` 等，PATH_FILE_SUFFIX；
行文里的斜杠词（读/写、增/删）不带这几样，照换；`.out` / `.log` / `.txt` / `.tsv` / `.json` / `.csv` 归下一段的已归档产物判法，不在这里；
带 `:行号` 的先去掉行号），就是路径形态：`/srv/旧名/data`、`crates/旧名/src/lib.rs`、`旧名说明.md` 都是。路径指的是磁盘上（第二台上）真实存在的目录或文件，
文字里换了名它就指空，2026-10-18 实测 --apply 把第二台上真实存在的目录路径也换了名。--apply 不换它、--check 照数（旧名还在，不装作没有），
两边都逐条报「文件:行  那一段文字」；出路是真实存在的路径登记 `.claude/term-rename-exempt` 的按行豁免，写错的引用手改（连同改名那份文件或目录本身）。
同一行里路径之外的旧名照换。--apply 的退出码不因它变，退 0；判红在随后的 --check。

已归档产物的文件名不换也不报：一个文件名形态的串（`名字.out` / `.log` / `.txt` / `.tsv` / `.json` / `.csv`，名字从一段 `[A-Za-z0-9_.-]` 连续串的开头起认），
版本库历史里 `research/results/` 下出现过、今天树里没有，它就是一份已归档产物的名字，产物改不了名，
引它的那一处留旧名 doc-experiments.sh 的 results-cited 格才在历史里找得到（`.claude/rules/path-moves.md`「留存产物分两类」）。按串判，不整份豁免。

--apply 的写回不在原文件上就地写：每份文件同目录排他新建临时文件、fsync、照原权限位与属主、改名换上
（`lib_atomic_replace.py`）。扫全仓会扫到 .sh 与 .py，正在按偏移边跑边读它的进程读的仍是旧 inode 的旧内容。
一份被拒（多个硬链接、当前用户只读、建不了临时文件、写或改名失败）就跳过它、接着换别的，最后逐份列出。

自证（--selftest）里有一格判长串掩码：一段 65536 字的连续 `[A-Za-z0-9_.-]` 串（不带扩展名）后面跟一个已归档产物的名字，
掩码在子进程里跑、1 秒内要做完，换出来的与不加左边界时逐字相同（逐字比在短串上做：不加左边界的那一条在长串上跑不完）。
弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一格的名字，括号里是那条「✗ 自检失败：」的开头）：
  SWEEP_TERM_BREAK=path-shaped-replaced  不识别路径形态，带 `/` 的与像文件名的命中也照旧名换掉（改前的样子）
      路径形态（「✗ 自检失败：路径形态那一格判错：」，--apply 之后路径形态的命中被换掉了、或 --check 报不出文件:行）
  SWEEP_TERM_BREAK=slash-words-protected  路径形态退回「带 `/` 就算」（收窄之前的认法），行文里的斜杠词（读/写）也被当路径掩起来不换
      斜杠词（「✗ 自检失败：斜杠词那一格判错：」，--apply 之后斜杠词里的旧名没被换掉）
  SWEEP_TERM_BREAK=archived-names-unprotected  不保护已归档产物的名字，照旧名换掉
      已归档产物（「✗ 自检失败：已归档产物的名字被换掉了，或换完仍判红」）
  SWEEP_TERM_BREAK=product-name-no-left-boundary  去掉产物名字的左边界（PRODUCT_FILE_NAME_PATTERN_WITHOUT_LEFT_BOUNDARY）
      长串掩码（「✗ 自检失败：长串掩码那一格判错：65536 字的连续串掩码用了」…「超时（上限 1 秒）」）
  SWEEP_TERM_BREAK=regex-assertions-dropped  正则那几行去掉前后断言（(?<!…)、(?<=…)、(?!…)、(?=…)）再用
      正则匹配（「✗ 自检失败：正则匹配那一格判错：误打了树层、外层那一行（「每层 134」「第 1 层 2 个」「外层 16.33 ms」）」）
  SWEEP_TERM_BREAK=line-exemption-covers-the-file  按行豁免的那份文件整份不扫（行内锚当成整份豁免）
      按行豁免（「✗ 自检失败：按行豁免那一格判错：同一份文件里没豁免的两行」）
  SWEEP_TERM_BREAK=ignored-files-walked  根在 git 工作树里也走整棵目录树，不按 git ls-files 列
      只扫 git 列出的（「✗ 自检失败：只扫 git 列出的那一格判错：git 忽略的 multi-host.env、memory-peaks.tsv 不该算进去」）
      连带红的格：「✗ 全仓还有 1 份文件、…处旧名，其中路径形态 0 处」
  SWEEP_TERM_BREAK=header-as-row  load_renames() 不跳登记表的表头行（「旧」「新」「匹配」「是什么」那一行），当数据行读
      表头不当记录（「✗ 自检失败：表头不当记录那一格判错：只有标记、表头与分隔行的登记表」…）
  SWEEP_TERM_BREAK=own-fixtures-swept  不排除 term-renames 格自己的样本目录，照登记表扫、照旧名换
      自己那一格的样本（「✗ 自检失败：自己那一格的样本那一格判错：--apply 之后样本里的旧名被换掉了」…）
  SWEEP_TERM_BREAK=defer-value-as-root  位置参数照旧从全部参数里挑不以 -- 起头的，--defer 的值被当成仓根
      参数解析（「✗ 自检失败：参数解析那一格判错：不带仓根、只带 --defer deferred.md」…）

退出码：--check 0 搜不出旧名、1 还有旧名；--apply 0 换完、3 有文件没换上（那几份原文件没动、临时文件已删）；
2 登记表或豁免表有问题（含写不对的正则、对不上的行内锚），或根在 git 工作树里而 git 列不出文件，或参数写错、不给仓根而当前目录不在 git 仓里。
"""
import os
import sys

import re

from lib_atomic_replace import ReplaceRefused, problems_with_replacement_by_rename, replace_file_contents_by_rename
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SKIP_DIRS = {".git", "target", "node_modules"}
RESULTS_DIRECTORY = "research/results"
# 左边界 (?<![A-Za-z0-9_.-])：名字只从一段 [A-Za-z0-9_.-] 连续串的开头起认。不加它时，一段不带扩展名的长串
# 在每个起点都要吃到串尾再退回来找「.扩展名」，耗时随串长平方涨（research/results/ 逐份扫下来合计 1041 秒，加上 1.76 秒）；
# 能从串中间起的匹配，从串开头起也匹配得上（中间那几个字符同样被 + 吃进去），所以换下来的串与不加边界时逐字相同。
PRODUCT_FILE_NAME_BODY = r"[A-Za-z0-9_.-]+\.(?:out|log|txt|tsv|json|csv)\b"
PRODUCT_FILE_NAME_PATTERN = re.compile(r"(?<![A-Za-z0-9_.-])" + PRODUCT_FILE_NAME_BODY)
# 不带左边界的那一条只给自证比输出、给弄坏开关 SWEEP_TERM_BREAK=product-name-no-left-boundary 用
PRODUCT_FILE_NAME_PATTERN_WITHOUT_LEFT_BOUNDARY = re.compile(PRODUCT_FILE_NAME_BODY)
PRODUCT_NAME_PROBE_RUN_LENGTH = 65536
PRODUCT_NAME_PROBE_SECONDS = 1.0
ARCHIVED_PLACEHOLDER_PATTERN = re.compile("\x00archived(\\d+)\x00")
# 路径形态：旧名所在的那一段不间断文字（token）以 ~/、/、./、../ 起头、是带 :// 的网址、带 `/` 而至少一段像文件名，或整段像一个文件名。界：空白、引号、反引号、括号、中英文逗号句号分号冒号顿号与 | ! ? 和占位符的 \x00
PATH_TOKEN_DELIMITERS = frozenset(" \t\r\n\"'`<>()[]{}（）「」『』【】《》，。；：、|,;!?！？\x00")
PATH_FILE_SUFFIX = re.compile(r"^[^/]+\.(?:md|rs|toml|sh|py|wgsl|yaml|yml|lock|cfg|conf|env|service|rules|html|css|js|ts)$")
PATH_LINE_NUMBER_SUFFIX = re.compile(r"(?::\d+)+$")
PATH_ROOT_PREFIXES = ("~/", "/", "./", "../")
PATH_PLACEHOLDER_PATTERN = re.compile("\x00path(\\d+)\x00")
PATH_REPORT_LIMIT = 50
EXEMPT_FILE = ".claude/term-rename-exempt"
# 门禁 doc-text 的 term-renames 格自己的判别力样本：这个目录下、目录名以这个前缀起头的，默认不扫（文件头有说明，checks-owed C441）
OWN_CELL_FIXTURE_PARENT = ".claude/gate.d/fixtures/doc-text.sh"
OWN_CELL_FIXTURE_PREFIX = "term-renames-"
RENAMES_PATH = ".claude/kb/term-renames.md"
TABLE_MARK = "<!-- term-renames:table -->"
MATCH_KINDS = ("整串", "词边界", "正则")
# 表格的一行按没转义的竖线切格；正则里的竖线写成 \|，切完还原成 |
TABLE_CELL_SEPARATOR = re.compile(r"(?<!\\)\|")
# 按行豁免：<相对路径>:「<行内锚>」  # 为什么。锚收尾的 」后面接空白加 # 或接行尾，锚里可以带「」
LINE_EXEMPTION = re.compile(r"^(?P<path>.+?):「(?P<anchor>.+?)」(?=\s+#|\s*$)")
# 弄坏开关 regex-assertions-dropped 去掉的前后断言（不带嵌套括号的那种）
LOOKAROUND_ASSERTION = re.compile(r"\(\?<?[=!][^()]*\)")
# 按行豁免的那一行换成的占位符：私用区的字，序号的每一位数字是 U+E010 起的十个字；登记的旧名与正则打不进去
EXEMPT_LINE_OPEN, EXEMPT_LINE_CLOSE, EXEMPT_LINE_DIGIT_ZERO = "", "", 0xE010
EXEMPT_LINE_PLACEHOLDER_PATTERN = re.compile("([-]+)")


def broken(switch):
    return os.environ.get("SWEEP_TERM_BREAK") == switch


def table_cells(line):
    """一行表格的各格：按没转义的竖线切，去掉两头的空格与反引号，\\| 还原成 |。"""
    parts = TABLE_CELL_SEPARATOR.split(line.strip())
    if parts and not parts[0].strip():
        parts = parts[1:]
    if parts and not parts[-1].strip():
        parts = parts[:-1]
    return [part.strip().strip("`").replace("\\|", "|") for part in parts]


def minimum_width(pattern):
    """正则至少吃几个字（前后断言不算宽度）。"""
    try:
        from re import _parser as regex_parser
    except ImportError:
        import sre_parse as regex_parser
    return regex_parser.parse(pattern).getwidth()[0]


def regex_row_problem(pattern):
    """「匹配」写正则的那一行写得对不对：空串是对的，否则是哪里不对。"""
    try:
        compiled = re.compile("(?:%s)" % pattern)
    except re.error as error:
        return "编译不过（%s）" % error
    if compiled.groups:
        return "带了 %d 个捕获组" % compiled.groups
    if minimum_width(pattern) == 0:
        return "能匹配空串"
    return ""


def load_renames(root):
    """从 .claude/kb/term-renames.md 的登记表读「旧 → 新」，一行一对。
    加一条术语改名 = 往那张表加一行，这个脚本与门禁自动管到它，不用再改代码。"""
    path = os.path.join(root, RENAMES_PATH)
    if not os.path.exists(path):
        print("  ✗ 找不到术语对照表 %s" % RENAMES_PATH)
        print("     → 怎么办：建一份，表上方写 %s，四列：旧、新、匹配（整串 / 词边界 / 正则）、是什么。" % TABLE_MARK)
        sys.exit(2)
    text = open(path, encoding="utf-8").read()
    if TABLE_MARK not in text:
        print("  ✗ %s 里没有 %s 标记，找不到哪张表是登记表" % (RENAMES_PATH, TABLE_MARK))
        print("     → 怎么办：在登记表上方单起一行写这个标记。")
        sys.exit(2)
    pairs = []
    for line in text[text.index(TABLE_MARK):].split("\n"):
        if not line.startswith("|"):
            continue
        cells = table_cells(line)
        # 表头（旧 | 新 | 匹配 | 是什么）与分隔行不是记录（C443）；header-as-row 开关打开时表头当数据行读，给自证证红
        is_header = cells[0] == "旧" and not broken("header-as-row")
        if len(cells) < 3 or is_header or cells[0] == "" or set(cells[0]) <= set("-: "):
            continue
        old, new, how = cells[0], cells[1], cells[2]
        if how not in MATCH_KINDS:
            print("  ✗ %s：「%s」那一行的「匹配」写的是「%s」，只认「整串」「词边界」「正则」" % (RENAMES_PATH, old, how))
            print("     → 怎么办：整串就写整串；缩写这类怕打中别的词的写词边界；同一个串在别处是另一个量的写正则（写法在 sweep-term.py 文件头）。")
            sys.exit(2)
        if how == "正则":
            problem = regex_row_problem(old)
            if problem:
                print("  ✗ %s：「%s」那一行的正则%s" % (RENAMES_PATH, old, problem))
                print("     → 怎么办：竖线写成 \\|；分组用 (?:…)，不用捕获组；标志写成带范围的 (?i:…)；"
                      "前后断言不算宽度，断言之外至少要吃一个字（写法在 sweep-term.py 文件头）。")
                sys.exit(2)
        pairs.append((old, new, how))
    if not pairs:
        print("  ✗ %s 的登记表一行数据都没有" % RENAMES_PATH)
        print("     → 怎么办：每改一个全仓术语加一行，门禁才管得到它。")
        sys.exit(2)
    return pairs


def compile_rules(pairs):
    """替换规则按表的顺序；检测正则**独立合成、比替换表宽**——替换漏掉哪种形态，检测也要报得出来
    （自检与被检共用同一条判据，判据错了两边一起错，见 checks-owed C428）。"""
    replacements, detectors = [], []
    for old, new, how in pairs:
        if how == "正则":
            pattern = LOOKAROUND_ASSERTION.sub("", old) if broken("regex-assertions-dropped") else old
        elif how == "词边界":
            if old.endswith("_"):
                pattern = r"\b" + re.escape(old)
            elif old.startswith("_"):
                pattern = re.escape(old) + r"\b"
            else:
                pattern = r"\b" + re.escape(old) + r"\b"
        else:
            pattern = re.escape(old)
        replacements.append((re.compile(pattern), new))
        detectors.append("(?:%s)" % pattern)
    return replacements, re.compile("|".join(detectors))


def load_exempt(root):
    path = os.path.join(root, EXEMPT_FILE)
    if not os.path.exists(path):
        print("  ✗ 找不到豁免登记表 %s" % EXEMPT_FILE)
        print("     → 怎么办：建一张，一行一条 <相对路径或目录>  # 为什么；没有豁免就留一张只有注释的空表。")
        sys.exit(2)
    entries, line_exemptions, missing, malformed = [], {}, [], []
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        anchored = LINE_EXEMPTION.match(line)
        if anchored:
            line_exemptions.setdefault(os.path.normpath(anchored.group("path")), []).append(anchored.group("anchor"))
            continue
        relative = line.split("#")[0].strip()
        if ":「" in relative:
            malformed.append(line)
            continue
        entries.append(relative)
        if not os.path.exists(os.path.join(root, relative)):
            missing.append(relative)
    if malformed:
        print("  ✗ 豁免登记表里有 %d 行像按行豁免、却读不出行内锚：" % len(malformed))
        for line in malformed:                                 # gate-lint:detail
            print("      %s" % line)
        print("     → 怎么办：写成 <相对路径>:「<那一行里的一段原文>」  # 为什么；锚收尾的 」后面接空白加 # 或接行尾。")
        sys.exit(2)
    unmatched = []
    for relative, anchors in sorted(line_exemptions.items()):
        target = os.path.join(root, relative)
        if not os.path.isfile(target):
            missing.append("%s（按行豁免）" % relative)
            continue
        try:
            lines = open(target, encoding="utf-8").read().split("\n")
        except (UnicodeDecodeError, OSError) as error:
            unmatched.append("%s：读不成 UTF-8 文本（%s），按行豁免用不上" % (relative, error))
            continue
        for anchor in anchors:
            hits = [number for number, text in enumerate(lines, 1) if anchor in text]
            if len(hits) != 1:
                unmatched.append("%s:「%s」命中 %d 行%s" % (relative, anchor, len(hits),
                                 "（第 %s 行）" % "、".join(str(number) for number in hits[:10]) if hits else ""))
    if unmatched:
        print("  ✗ 豁免登记表里有 %d 条按行豁免对不上（行内锚要在那份文件里恰好命中一行）：" % len(unmatched))
        for problem in unmatched:                              # gate-lint:detail
            print("      %s" % problem)
        print("     → 怎么办：命中 0 行的（那一行改了或删了）照现在的原文改锚，或删掉这一条；命中不止一行的把锚写长到只剩要留旧名的那一行。")
        sys.exit(2)
    # 对照表与这个脚本自己都写得出旧名（表里那一列就是旧名，脚本要从表里读），
    # 不豁免就会把自己吃掉 ⇒ 强制要求，不靠写表的人记得。
    myself = os.path.relpath(os.path.abspath(__file__), root)
    # 脚本在 root 外面（门禁拿真脚本扫判别力样本就是这样）时它根本不会被扫到，不用要求豁免
    required_paths = [RENAMES_PATH] + ([myself] if not myself.startswith("..") else [])
    for required in required_paths:
        if not any(required == e or required.startswith(e.rstrip("/") + "/") for e in entries):
            print("  ✗ %s 没登记进 %s" % (required, EXEMPT_FILE))
            print("     → 怎么办：加一行豁免它。对照表那一列就是旧名，这个脚本要从表里读旧名，")
            print("               不豁免的话第一次 --apply 就把它们自己换掉，之后再也换不动东西。")
            sys.exit(2)
    if missing:
        print("  ✗ 豁免登记表里有 %d 条指不到文件：" % len(missing))
        for relative in missing:                               # gate-lint:detail
            print("      %s" % relative)
        print("     → 怎么办：指不到的豁免项删掉或改成现存路径；不起作用的豁免会让人以为那批文件已经被绕开了。")
        sys.exit(2)
    return entries, line_exemptions


def own_cell_fixture_directory(relative):
    """relative 是 term-renames 格自己的样本目录里的文件时，返回那个样本目录（相对根的路径），否则 None。
    弄坏开关 own-fixtures-swept 打开时一律 None（不排除）。"""
    if broken("own-fixtures-swept"):
        return None
    parent = OWN_CELL_FIXTURE_PARENT.split("/")
    parts = os.path.normpath(relative).split(os.sep)
    if len(parts) > len(parent) + 1 and parts[:len(parent)] == parent and parts[len(parent)].startswith(OWN_CELL_FIXTURE_PREFIX):
        return "/".join(parts[:len(parent) + 1])
    return None


def exempt(relative, entries):
    for entry in entries:
        if relative == entry or relative.startswith(entry.rstrip("/") + "/"):
            return True
    return False


def archived_product_file_names(root):
    """历史里 research/results/ 下出现过、今天树里没有的文件名。不在 git 仓里（或 git 失败）时是空集：一个都不保护。"""
    import subprocess
    try:
        listing = subprocess.run(["git", "-C", root, "-c", "core.quotePath=false", "log", "--all", "--format=", "--name-only", "--", RESULTS_DIRECTORY],
                                 capture_output=True, text=True, check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return set()
    in_history = {os.path.basename(line) for line in listing.splitlines() if line.strip()}
    results_directory = os.path.join(root, RESULTS_DIRECTORY)
    in_tree = set(os.listdir(results_directory)) if os.path.isdir(results_directory) else set()
    return in_history - in_tree


def mask_archived_product_names(text, archived_names):
    """把已归档产物的名字换成占位符，替换与计数都看不见它们；返回 (换过的文本, 被换下来的原串)。"""
    if os.environ.get("SWEEP_TERM_BREAK") == "archived-names-unprotected":
        return text, []
    masked = []
    def to_placeholder(match):
        if match.group(0) not in archived_names:
            return match.group(0)
        masked.append(match.group(0))
        return "\x00archived%d\x00" % (len(masked) - 1)
    pattern = PRODUCT_FILE_NAME_PATTERN
    if os.environ.get("SWEEP_TERM_BREAK") == "product-name-no-left-boundary":
        pattern = PRODUCT_FILE_NAME_PATTERN_WITHOUT_LEFT_BOUNDARY
    return pattern.sub(to_placeholder, text), masked


def restore_archived_product_names(text, masked):
    return ARCHIVED_PLACEHOLDER_PATTERN.sub(lambda match: masked[int(match.group(1))], text)


def path_shaped(token):
    """token 是不是路径形态：以 ~/、/、./、../ 起头，或是带 :// 的网址，或（去掉行号和收尾的标点之后）按 `/` 切开至少一段像文件名；
    不带 `/` 的整段像文件名也算。只带 `/` 不够：行文里的斜杠词（读/写）不是路径。弄坏开关 slash-words-protected 退回带 `/` 就算。"""
    if "/" in token and broken("slash-words-protected"):
        return True
    if token.startswith(PATH_ROOT_PREFIXES) or "://" in token:
        return True
    stripped = PATH_LINE_NUMBER_SUFFIX.sub("", token).rstrip(".,:")
    return any(PATH_FILE_SUFFIX.match(segment) for segment in stripped.split("/"))


def mask_path_shaped(text, detect):
    """路径形态的命中：旧名所在的那一段文字换成占位符，替换与计数都看不见它；返回 (换过的文本, 被换下来的原串, [(行号, 原串)])。
    行号照传进来的文本数（前面的掩码都不改行数）。弄坏开关 path-shaped-replaced 打开时什么都不掩。"""
    if broken("path-shaped-replaced"):
        return text, [], []
    spans = set()
    for match in detect.finditer(text):
        start, end = match.span()
        while start > 0 and text[start - 1] not in PATH_TOKEN_DELIMITERS:
            start -= 1
        while end < len(text) and text[end] not in PATH_TOKEN_DELIMITERS:
            end += 1
        if path_shaped(text[start:end]):
            spans.add((start, end))
    kept, found = [], []
    pieces, position = [], 0
    for start, end in sorted(spans):
        kept.append(text[start:end])
        found.append((text.count("\n", 0, start) + 1, text[start:end]))
        pieces.append(text[position:start] + "\x00path%d\x00" % (len(kept) - 1))
        position = end
    pieces.append(text[position:])
    return "".join(pieces), kept, found


def restore_path_shaped(text, kept):
    return PATH_PLACEHOLDER_PATTERN.sub(lambda match: kept[int(match.group(1))], text)


def exempt_line_placeholder(number):
    return EXEMPT_LINE_OPEN + "".join(chr(EXEMPT_LINE_DIGIT_ZERO + int(digit)) for digit in str(number)) + EXEMPT_LINE_CLOSE


def mask_exempt_lines(text, anchors):
    """按行豁免：含行内锚的那一行换成占位符，替换与计数都看不见它；返回 (换过的文本, 被换下来的原行)。"""
    if not anchors:
        return text, []
    lines = text.split("\n")
    kept = []
    for index, line in enumerate(lines):
        if broken("line-exemption-covers-the-file") or any(anchor in line for anchor in anchors):
            kept.append(line)
            lines[index] = exempt_line_placeholder(len(kept) - 1)
    return "\n".join(lines), kept


def restore_exempt_lines(text, kept):
    if not kept:
        return text
    return EXEMPT_LINE_PLACEHOLDER_PATTERN.sub(
        lambda match: kept[int("".join(str(ord(digit) - EXEMPT_LINE_DIGIT_ZERO) for digit in match.group(1)))], text)


def candidate_files(root):
    """要扫的文件（相对根的路径，排好序）与一句「怎么列的」。根在 git 工作树里：git 跟踪的加未跟踪而未忽略的；不在：整棵目录树。"""
    import subprocess
    has_git_directory = os.path.exists(os.path.join(root, ".git"))
    if not broken("ignored-files-walked"):
        try:
            probe = subprocess.run(["git", "-C", root, "rev-parse", "--is-inside-work-tree"], capture_output=True, text=True)
            inside = probe.returncode == 0 and probe.stdout.strip() == "true"
            probe_problem = (probe.stderr.strip().splitlines() or ["git rev-parse 退 %d" % probe.returncode])[-1]
        except OSError as error:
            inside, probe_problem = False, "起不了 git（%s）" % error
        if not inside and has_git_directory:
            print("  ✗ %s 下有 .git，却判不出它是 git 工作树：%s" % (root, probe_problem))
            print("     → 怎么办：在根下单跑 git rev-parse --is-inside-work-tree 看报什么，修好再跑；不退回走整棵树，"
                  "那样会把 multi-host.env 这类 git 忽略的本机文件一起改掉。")
            sys.exit(2)
        if inside:
            listing = subprocess.run(["git", "-C", root, "-c", "core.quotePath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                                     capture_output=True)
            if listing.returncode != 0:
                print("  ✗ git ls-files 在 %s 下退 %d：%s" % (root, listing.returncode,
                      (listing.stderr.decode("utf-8", "replace").strip().splitlines() or ["（没有报错）"])[-1]))
                print("     → 怎么办：在根下单跑 git ls-files --cached --others --exclude-standard 看报什么，修好再跑；不退回走整棵树。")
                sys.exit(2)
            names = {os.path.normpath(os.fsdecode(name)) for name in listing.stdout.split(b"\0") if name}
            return sorted(names), "git 跟踪的与未跟踪而未忽略的文件"
    names = []
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in files:
            names.append(os.path.relpath(os.path.join(base, name), root))
    if broken("ignored-files-walked"):
        return sorted(names), "整棵目录树（弄坏开关 ignored-files-walked：不按 git 列）"
    return sorted(names), "整棵目录树（不在 git 工作树里，跳过 %s）" % "、".join(sorted(SKIP_DIRS))


def walk(root, entries):
    for relative in candidate_files(root)[0]:
        if not exempt(relative, entries) and own_cell_fixture_directory(relative) is None:
            yield relative


def sweep(root, apply_changes, defer=(), tally=None):
    """defer 是**分批落地**用的，不是豁免：别的会话正在读那批文件时先放着，那一批照样要换，换完 --check 才绿。
    豁免只有 .claude/term-rename-exempt 一处，别把 defer 当第二处豁免用。
    tally 给一个 dict 时填进 files（有旧名的份数）、occurrences（处数）、scanned（扫了的文本文件数）、
    excluded_fixture_directories（term-renames 格自己的样本目录排除了几个），自证用。"""
    entries, line_exemptions = load_exempt(root)
    entries = list(entries) + list(defer)
    exempt_line_count = sum(len(anchors) for anchors in line_exemptions.values())
    pairs = load_renames(root)
    replacements, detect = compile_rules(pairs)
    archived_names = archived_product_file_names(root)
    candidates, listing_note = candidate_files(root)
    files, occurrences, scanned = 0, 0, 0
    own_fixture_directories = set()
    worst = []
    refused = []
    path_hits = []          # (相对路径, 行号, 路径形态的那一段文字)：只报不换
    for relative in candidates:
        if exempt(relative, entries):
            continue
        own_fixture_directory = own_cell_fixture_directory(relative)
        if own_fixture_directory is not None:
            own_fixture_directories.add(own_fixture_directory)
            continue
        try:
            text = open(os.path.join(root, relative), encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        scanned += 1
        text, kept_lines = mask_exempt_lines(text, line_exemptions.get(relative, ()))
        text, masked = mask_archived_product_names(text, archived_names)
        text, kept_paths, found_paths = mask_path_shaped(text, detect)
        replaceable = len(detect.findall(text))
        path_hits += [(relative, number, token) for number, token in found_paths]
        # --check 数旧名还在的全部处（含路径形态的）；--apply 只换并只数能换的，只剩路径形态命中的文件不重写
        count = replaceable if apply_changes else replaceable + len(found_paths)
        if not count:
            continue
        if apply_changes:
            for pattern, new in replacements:
                text = pattern.sub(lambda match, new=new: new, text)
            text = restore_path_shaped(text, kept_paths)
            text = restore_archived_product_names(text, masked)
            text = restore_exempt_lines(text, kept_lines)
            try:
                notes = replace_file_contents_by_rename(os.path.join(root, relative), text)
            except ReplaceRefused as refusal:
                refused.append((relative, count, str(refusal)))
                continue
            for note in notes:
                print("  ! %s" % note)
        files += 1
        occurrences += count
        worst.append((count, relative))
    if tally is not None:
        tally.update(files=files, occurrences=occurrences, scanned=scanned, excluded_fixture_directories=len(own_fixture_directories), path_shaped=len(path_hits))
    excluded_note = "；默认不扫 term-renames 格自己的样本目录 %d 个" % len(own_fixture_directories)
    if apply_changes:
        print("  ✓ 换了 %d 份文件、%d 处（登记的改名 %d 条、豁免 %d 条、按行豁免 %d 行；范围：%s%s）" % (files, occurrences, len(pairs), len(entries), exempt_line_count, listing_note, excluded_note))
        report_path_shaped(path_hits, True)
        if refused:
            for relative, count, message in refused:                # gate-lint:detail
                print("      %s（%d 处）：%s" % (relative, count, message))
            print("  ✗ 有 %d 份文件没换上，那几份原文件没动、临时文件已删（逐份的原因与下一步在上面）" % len(refused))
            print("     → 怎么办：按每份后面那行「→ 怎么办」处理（多半是有多个硬链接或当前用户只读），再跑一次 --apply；")
            print("               已经换好的那些搜不出旧名，不会重复换。")
            return 3
        print("     → 下一步：cargo build 与 cargo test --workspace，再 bash scripts/experiment-apparatus/replay.sh 全量复跑，")
        print("               产物必须仍然逐字节一致 —— 源码与产物一起换，这一条才成立，要跑出来看，不能声称。")
        return 0
    if occurrences:
        for count, relative in sorted(worst, reverse=True)[:20]:   # gate-lint:detail
            print("      %-72s %d 处" % (relative, count))
        report_path_shaped(path_hits, False)
        print("  ✗ 全仓还有 %d 份文件、%d 处旧名，其中路径形态 %d 处（登记的改名 %d 条、豁免 %d 条与按行豁免 %d 行不算；范围：%s%s）"
              % (files, occurrences, len(path_hits), len(pairs), len(entries), exempt_line_count, listing_note, excluded_note))
        print("     → 怎么办：跑 python3 scripts/kb/sweep-term.py --apply 换掉（路径形态的 --apply 不换，上面逐条列了文件:行：路径是真实存在的，"
              "登记按行豁免；是写错的引用，手改连同那份文件或目录本身）；")
        print("               确实该留旧名的，登记进 .claude/term-rename-exempt 并写明为什么：整份都该留的写路径，")
        print("               只有一句该留的写 <路径>:「<那一行里的一段原文>」只豁免那一行。")
        return 1
    print("  ✓ 全仓搜不出登记过的旧名了：查了 %d 份文本文件（%s）、%d 条改名登记" % (scanned, listing_note, len(pairs)))
    print("     没扫的 %d 处（.claude/term-rename-exempt 登记，各有理由）：%s" % (len(entries), "、".join(entries)))
    print("     按行豁免 %d 行（%d 份文件里，那几行留原文、别的行照扫）" % (exempt_line_count, len(line_exemptions)))
    print("     默认没扫 term-renames 格自己的样本目录 %d 个（%s/%s*，代码里写死，判别力样本里要留着旧名）" % (len(own_fixture_directories), OWN_CELL_FIXTURE_PARENT, OWN_CELL_FIXTURE_PREFIX))
    return 0


def report_path_shaped(path_hits, applied):
    """路径形态的命中逐条列「文件:行  那一段文字」（上限 PATH_REPORT_LIMIT 条，其余报个数）；没有就什么都不打。applied 是 --apply 那一趟：说明没换。"""
    if not path_hits:
        return
    for relative, number, token in path_hits[:PATH_REPORT_LIMIT]:      # gate-lint:detail
        print("      %s:%d  %s" % (relative, number, token))
    if len(path_hits) > PATH_REPORT_LIMIT:
        print("      …另有 %d 处没列" % (len(path_hits) - PATH_REPORT_LIMIT))
    if applied:
        print("  ! 路径形态的命中 %d 处只报不换（上面逐条列了文件:行）：路径指的是真实存在的目录或文件，文字里换了名它就指空" % len(path_hits))
        print("     → 怎么办：真实存在的路径登记 .claude/term-rename-exempt 的按行豁免（<路径>:「<那一行里的一段原文>」  # 为什么）；"
              "是写错的引用，手改，连同改名那份文件或目录本身；随后的 --check 照数它们。")


def selftest():
    import tempfile
    # 表头那一格放最前：header-as-row 打开时下面所有格的登记表都带表头，读表当场退 2，排在后面就走不到这一格
    header_problem = check_header_only_table()
    if header_problem:
        print("  ✗ 自检失败：表头不当记录那一格判错：%s" % header_problem)
        print("     → 怎么办：看 load_renames() 跳表头与分隔行那一句、与 0 条数据行时退 2 的那一句。")
        return 1
    with tempfile.TemporaryDirectory() as work:
        os.makedirs(os.path.join(work, ".claude"))
        os.makedirs(os.path.join(work, "keep"))
        open(os.path.join(work, "keep", "quoted.md"), "w", encoding="utf-8").write("btrfs 的 superblock\n")
        # 每一种变体各一行：漏掉哪一种，下面「换完判绿」那一步就红
        open(os.path.join(work, "code.rs"), "w", encoding="utf-8").write(
            "const SUPERBLOCK_BYTES: u64 = 481;\n"
            "const SUPER_BLOCK_BYTES: u64 = 481;\n"
            "struct SuperBlock;\nstruct Superblock;\nstruct Super_Block;\n"
            "let superBlock = 1;\nlet super_block = 2;\nlet superblock = 3;\n"
            "// super-block 与 超级块 两种写法\n"
            "let sb_mac = 4;\nlet tail_sb = 5;\nlet sb = 6;\n")
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("keep/  # 别家术语\n.claude/kb/term-renames.md  # 表自己写着旧名\n%s  # 脚本自己写着旧名\n"
            % os.path.relpath(os.path.abspath(__file__), work))
        os.makedirs(os.path.join(work, ".claude", "kb"))
        open(os.path.join(work, ".claude", "kb", "term-renames.md"), "w", encoding="utf-8").write(
            "<!-- term-renames:table -->\n"
            "| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n"
            "| 超级块 | 系统配置 | 整串 | 中文 |\n"
            "| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量带下划线 |\n"
            "| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量 |\n"
            "| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |\n"
            "| SuperBlock | SystemConfiguration | 整串 | 大驼峰 |\n"
            "| Superblock | SystemConfiguration | 整串 | 大驼峰 b 小写 |\n"
            "| superBlock | systemConfiguration | 整串 | 小驼峰 |\n"
            "| super_block | system_configuration | 整串 | 蛇形带下划线 |\n"
            "| super-block | system-configuration | 整串 | 连字符 |\n"
            "| superblock | system_configuration | 整串 | 蛇形 |\n"
            "| sb_ | system_configuration_ | 词边界 | 缩写前缀 |\n"
            "| _sb | _system_configuration | 词边界 | 缩写后缀 |\n"
            "| sb | system_configuration | 词边界 | 裸缩写 |\n")
        if sweep(work, False) != 1:
            print("  ✗ 自检失败：带旧名的小仓 --check 没判红")
            print("     → 怎么办：看 walk() 与 MAP 的匹配。")
            return 1
        sweep(work, True)
        if sweep(work, False) != 0:
            print("  ✗ 自检失败：--apply 之后 --check 仍判红")
            print("     → 怎么办：看 MAP 的替换顺序，大小写形态是不是漏了一种。")
            return 1
        if "superblock" not in open(os.path.join(work, "keep", "quoted.md"), encoding="utf-8").read():
            print("  ✗ 自检失败：豁免目录里的旧名被换掉了")
            print("     → 怎么办：看 exempt() 的目录前缀判法。")
            return 1
        # 已归档产物的名字：历史里有、树里没有的那份不换不报；树里还在的那份照换
        import subprocess
        git_environment = dict(os.environ, GIT_AUTHOR_NAME="selftest", GIT_AUTHOR_EMAIL="selftest@example.invalid",
                               GIT_COMMITTER_NAME="selftest", GIT_COMMITTER_EMAIL="selftest@example.invalid")
        results = os.path.join(work, "research", "results")
        os.makedirs(results)
        for name in ("e1-superblock-run.out", "e1-superblock-live.out"):
            open(os.path.join(results, name), "w", encoding="utf-8").write("产物\n")
        subprocess.run(["git", "-C", work, "init", "-q"], check=True)
        subprocess.run(["git", "-C", work, "add", "-A"], check=True, env=git_environment)
        subprocess.run(["git", "-C", work, "commit", "-q", "-m", "产物"], check=True, env=git_environment)
        os.remove(os.path.join(results, "e1-superblock-run.out"))
        open(os.path.join(work, "cites.md"), "w", encoding="utf-8").write(
            "E1 的产物 e1-superblock-run.out 已归档\n另一份 e1-superblock-live.out 还在树里\n")
        if sweep(work, False) != 1:
            print("  ✗ 自检失败：树里还在的产物 e1-superblock-live.out 带旧名，--check 没判红")
            print("     → 怎么办：看 mask_archived_product_names：只该保护树里没有的那些。")
            return 1
        sweep(work, True)
        cites = open(os.path.join(work, "cites.md"), encoding="utf-8").read()
        if "e1-superblock-run.out" not in cites or "e1-superblock-live.out" in cites or sweep(work, False) != 0:
            print("  ✗ 自检失败：已归档产物的名字被换掉了，或换完仍判红")
            print("     → 怎么办：看 archived_product_file_names 与 mask / restore 那一对。")
            return 1
        open(os.path.join(work, ".claude", "term-rename-exempt"), "w", encoding="utf-8").write("nowhere/  # 指不到\n")
        code = None
        try:
            sweep(work, False)
        except SystemExit as stop:
            code = stop.code
        if code != 2:
            print("  ✗ 自检失败：豁免登记表指不到文件时没判红")
            print("     → 怎么办：看 load_exempt() 的存在性断言。")
            return 1
    long_run_problem = check_long_run_masking()
    if long_run_problem:
        print("  ✗ 自检失败：长串掩码那一格判错：" + long_run_problem)
        print("     → 怎么办：看 PRODUCT_FILE_NAME_PATTERN 开头的左边界 (?<![A-Za-z0-9_.-]) 还在不在；没有它，一段不带扩展名的长串"
              "在每个起点都要吃到串尾再退回来，耗时随串长平方涨。")
        return 1
    for cell_name, check, remedy in (
            ("正则匹配", check_regex_rows, "看 compile_rules() 里「正则」那一支有没有原样用「旧」一列（前后断言要留着）、table_cells() 的 \\| 还原、regex_row_problem() 的三样拒绝"),
            ("路径形态", check_path_shaped, "看 mask_path_shaped() 与 path_shaped()：旧名所在那一段文字带 / 或像文件名的，--apply 要掩起来不换，--check 要数并逐条报文件:行"),
            ("斜杠词", check_slash_words, "看 path_shaped()：只带 / 不算路径，行文里的斜杠词（读/写）旧名要换；以 ~/、/、./、../ 起头的、带 :// 的、有一段像文件名的才是路径形态"),
            ("按行豁免", check_line_exemptions, "看 load_exempt() 读 <路径>:「<行内锚>」那一支与锚恰好命中一行的判法、mask_exempt_lines() 只换含锚的那一行"),
            ("自己那一格的样本", check_own_cell_fixtures, "看 own_cell_fixture_directory() 与 sweep() 里 --check / --apply 跳过它的那一句：OWN_CELL_FIXTURE_PARENT 下目录名以 OWN_CELL_FIXTURE_PREFIX 起头的样本目录默认不扫"),
            ("只扫 git 列出的", check_git_listing, "看 candidate_files()：根在 git 工作树里要走 git ls-files --cached --others --exclude-standard，不走整棵目录树"),
            ("参数解析", check_argument_parsing, "看 parse_arguments()：--defer 要吃掉紧跟它的那一个参数，仓根只从剩下的里取；不给仓根时 root_of_current_directory() 取当前目录所在 git 仓的根")):
        problem = check()
        if problem:
            print("  ✗ 自检失败：%s那一格判错：%s" % (cell_name, problem))
            print("     → 怎么办：" + remedy + "。")
            return 1
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接与只读拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = "#!/usr/bin/env bash\n" + "echo 前面这几行正在被 bash 边跑边读\n" * 20 + "run_step 超级块\nexit $?\n"
    problems = problems_with_replacement_by_rename(
        apply_to_this_file_only, running_script, running_script.replace("超级块", "系统配置", 1))
    if problems:
        print("  ✗ 自检失败：--apply 写回的方式不对")
        for problem in problems:
            print("     " + problem)  # gate-lint:detail
        print("     → 怎么办：按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename，"
              "以及 sweep() 里 --apply 那一段是不是还有就地写。")
        return 1
    print("  ✓ 自检：带旧名判红、换完判绿、豁免目录不被换、已归档产物的名字不换不报而树里还在的照换、豁免指不到文件判红、"
          "term-renames 格自己的样本目录（.claude/gate.d/fixtures/doc-text.sh/term-renames-*）默认不扫、--apply 之后样本里的旧名原样在、在那份样本根上 --check 仍判红而别处的旧名与别的格、别的门禁文件名下的同名目录照换、"
          "%d 字的连续串掩码在 %.0f 秒内做完且与不加左边界时逐字相同；--apply 换上的是新 inode、权限位不变、"
          "改之前打开文件的读者接着读到旧内容、fsync 在改名之前、经符号链接改的是它指向的文件、"
          "有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件；"
          "「匹配」写正则的只打 D13 那一层的「层 1」「层 2」4 处、不打树层与外层那一行，\\| 还原成竖线，带捕获组、能匹配空串、编译不过的退 2；"
          "路径形态（带 / 的与像文件名的）的命中 --apply 不换、--check 照数并逐条报文件:行，同一行里路径之外的旧名照换；"
          "按行豁免只留锚那一行、同一份文件里另外 2 行照判照换，锚命中 0 行或不止一行、指不到文件、读不出锚都退 2；"
          "在 git 工作树里只扫跟踪的与未跟踪而未忽略的 2 份，git 忽略的 multi-host.env、memory-peaks.tsv 不读不写；"
          "登记表只有标记、表头与分隔行时不把表头当规则、退 2 报一行数据都没有；"
          "不带仓根只带 --defer 时仓根取当前目录所在的仓、--defer 的值不当仓根，仓根写在 --defer 之后照认，认不出的选项与缺值的 --defer 退 2"
          % (PRODUCT_NAME_PROBE_RUN_LENGTH, PRODUCT_NAME_PROBE_SECONDS))
    return 0


# 长串掩码那一格在子进程里跑的那一段：载入这份脚本，掩一段 <长度> 字的连续串加一个已归档产物的名字，打「耗时 same|different」
LONG_RUN_PROBE = r"""
import importlib.util, os, sys, time
sys.dont_write_bytecode = True
script_path, run_length = sys.argv[1], int(sys.argv[2])
sys.path.insert(0, os.path.dirname(script_path))
spec = importlib.util.spec_from_file_location("sweep_term_long_run_probe", script_path)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
text = "x" * run_length + " e1-run.out"
started = time.monotonic()
masked_text, masked = module.mask_archived_product_names(text, {"e1-run.out"})
elapsed = time.monotonic() - started
same = masked_text == "x" * run_length + " \x00archived0\x00" and masked == ["e1-run.out"]
print("%.3f %s" % (elapsed, "same" if same else "different"))
"""


def check_long_run_masking():
    """自证的长串掩码那一格：返回空串是判对，否则是判错的说明。
    逐字比在短串上做（带与不带左边界各掩一遍）；耗时在子进程里量，到点杀掉——正则在 C 里跑，同一个进程里的信号打断不了它。"""
    import subprocess
    archived = {"e1-run.out", "e2-old.log"}
    sample = ("见 e1-run.out、x.e1-run.out 与 abc-e1-run.out.bak；e1-live.out-e1-run.out，(e2-old.log) "
              + "y" * 2048 + " e2-old.log.gz e2-old.logx e2-old.log\n")

    def mask_without_left_boundary(text):
        masked = []

        def to_placeholder(match):
            if match.group(0) not in archived:
                return match.group(0)
            masked.append(match.group(0))
            return "\x00archived%d\x00" % (len(masked) - 1)
        return PRODUCT_FILE_NAME_PATTERN_WITHOUT_LEFT_BOUNDARY.sub(to_placeholder, text), masked
    if mask_archived_product_names(sample, archived) != mask_without_left_boundary(sample):
        return "短串上带左边界掩出来的与不带左边界时不一样"
    try:
        probe = subprocess.run([sys.executable, "-c", LONG_RUN_PROBE, os.path.abspath(__file__), str(PRODUCT_NAME_PROBE_RUN_LENGTH)],
                               capture_output=True, text=True, timeout=PRODUCT_NAME_PROBE_SECONDS + 4)
    except subprocess.TimeoutExpired:
        return "%d 字的连续串掩码超时（子进程 %.0f 秒还没做完，已杀掉）" % (PRODUCT_NAME_PROBE_RUN_LENGTH, PRODUCT_NAME_PROBE_SECONDS + 4)
    fields = probe.stdout.split()
    if probe.returncode != 0 or len(fields) != 2:
        return "子进程退出 %d，没打出「耗时 same|different」：%s" % (probe.returncode, (probe.stderr.strip().splitlines() or ["（没有报错）"])[-1])
    elapsed, verdict = float(fields[0]), fields[1]
    if elapsed >= PRODUCT_NAME_PROBE_SECONDS:
        return "%d 字的连续串掩码用了 %.3f 秒，超时（上限 %.0f 秒）" % (PRODUCT_NAME_PROBE_RUN_LENGTH, elapsed, PRODUCT_NAME_PROBE_SECONDS)
    if verdict != "same":
        return "%d 字的连续串后面那个已归档产物的名字没被掩成占位符，或串本身被改了" % PRODUCT_NAME_PROBE_RUN_LENGTH
    return ""


def quietly(function, *arguments, **keywords):
    """自证里跑一趟 sweep 不打它的输出；退出码（含 SystemExit 的）原样交回。"""
    import contextlib
    import io
    with contextlib.redirect_stdout(io.StringIO()):
        try:
            return function(*arguments, **keywords)
        except SystemExit as stop:
            return stop.code


def write_selftest_tables(root, rename_rows, exempt_lines):
    """自证的小仓里写登记表（rename_rows 每行四格）与豁免表（登记表自己那一行之外，exempt_lines 原样逐行写）。"""
    os.makedirs(os.path.join(root, ".claude", "kb"), exist_ok=True)
    with open(os.path.join(root, RENAMES_PATH), "w", encoding="utf-8") as handle:
        handle.write("%s\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n" % TABLE_MARK)
        for row in rename_rows:
            handle.write("| %s |\n" % " | ".join(row))
    with open(os.path.join(root, EXEMPT_FILE), "w", encoding="utf-8") as handle:
        handle.write("%s  # 表自己写着旧名\n%s  # 豁免表自己写着旧名\n" % (RENAMES_PATH, EXEMPT_FILE))
        for line in exempt_lines:
            handle.write(line + "\n")


# 正则匹配那一格的样本：D13 那一层的写法、同串异义的树层与外层、带竖线的选一
REGEX_SAMPLE_D13_LINE = "D13 已定项 9：层 1 常规、层 2 全量；C409 说层 1、层 2 两层"
REGEX_SAMPLE_D13_LINE_RENAMED = ("D13 已定项 9：中等负载崩溃点剪枝穷举测试 常规、大负载多设备崩溃点穷举测试 全量；"
                                 "C409 说中等负载崩溃点剪枝穷举测试、大负载多设备崩溃点穷举测试 两层")
REGEX_SAMPLE_OTHER_LINE = "树表每层 134 棵，第 1 层 2 个、第 2 层 2 个；外层 16.33 ms，内部层 110 个孩子"
REGEX_SAMPLE_PIPE_LINE = "旧称甲与旧称乙"
REGEX_SAMPLE_ROWS = [
    ("`(?<![每第内外里叶二])(?<![0-9] )层 1(?![0-9])`", "中等负载崩溃点剪枝穷举测试", "正则", "D13 那一层"),
    ("`(?<![每第内外里叶二])(?<![0-9] )层 2(?![0-9])`", "大负载多设备崩溃点穷举测试", "正则", "D13 那一层"),
    ("`旧称甲\\|旧称乙`", "新称", "正则", "竖线写成 \\|"),
]


def check_regex_rows():
    """正则匹配那一格：只打 D13 那一层的「层 1」「层 2」，不打树层与外层；\\| 还原成竖线；写不对的正则退 2。返回空串是判对。"""
    import tempfile
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, REGEX_SAMPLE_ROWS, [])
        _, detect = compile_rules(load_renames(root))
        wrong = detect.findall(REGEX_SAMPLE_OTHER_LINE)
        if wrong:
            return "误打了树层、外层那一行（「每层 134」「第 1 层 2 个」「外层 16.33 ms」）：打中 %d 处（%s）" % (len(wrong), "、".join(wrong))
        hits = len(detect.findall(REGEX_SAMPLE_D13_LINE))
        if hits != 4:
            return "D13 那一层的「层 1 常规」「层 2 全量」「层 1、层 2」应当打中 4 处，实际 %d 处" % hits
        hits = len(detect.findall(REGEX_SAMPLE_PIPE_LINE))
        if hits != 2:
            return "竖线写成 \\| 的那一行应当打中「旧称甲」「旧称乙」2 处，实际 %d 处" % hits
        sample = os.path.join(root, "doc.md")
        with open(sample, "w", encoding="utf-8") as handle:
            handle.write("\n".join([REGEX_SAMPLE_D13_LINE, REGEX_SAMPLE_OTHER_LINE, REGEX_SAMPLE_PIPE_LINE, ""]))
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 1 or tally.get("occurrences") != 6:
            return "带「层 1」「层 2」的样本 --check 应当退 1、报 6 处，实际退 %s、%s 处" % (code, tally.get("occurrences"))
        quietly(sweep, root, True)
        expected = "\n".join([REGEX_SAMPLE_D13_LINE_RENAMED, REGEX_SAMPLE_OTHER_LINE, "新称与新称", ""])
        actual = open(sample, encoding="utf-8").read()
        if actual != expected:
            return "--apply 换出来的不对：应当是 %r，实际 %r" % (expected, actual)
        code = quietly(sweep, root, False)
        if code != 0:
            return "--apply 之后 --check 应当退 0，实际退 %s" % code
        for bad, why in (("`(层) 1`", "带捕获组"), ("`(?<!每)`", "只有断言、能匹配空串"), ("`层 [1`", "编译不过")):
            write_selftest_tables(root, [(bad, "x", "正则", why)], [])
            code = quietly(sweep, root, False)
            if code != 2:
                return "%s的正则 %s 应当退 2，实际退 %s" % (why, bad, code)
    return ""


LINE_EXEMPTION_SAMPLE_LINES = ["第一行：用户原话「超级块要改名」，原样留着", "第二行：超级块的别处说法照新名", "第三行：再提一次超级块", ""]


PATH_SHAPED_SAMPLE_LINES = ["目录 /srv/超级块/data 在第二台上真实存在", "文件 超级块说明.md:12 在别处", "行文里的超级块要换，路径旁的 超级块 也要换", "引用 [说明](docs/x.md) 与 crates/超级块/src/lib.rs", ""]


def check_path_shaped():
    """路径形态那一格：旧名所在的那一段文字带 / 或像文件名的，--apply 不换、--check 照数并逐条报文件:行，同一行里路径之外的旧名照换；
    只剩路径形态命中的文件 --apply 不重写（内容与权限不动）。返回空串是判对。"""
    import contextlib
    import io
    import tempfile
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, [("超级块", "系统配置", "整串", "中文")], [])
        sample = os.path.join(root, "notes.md")
        only_paths = os.path.join(root, "only-paths.md")
        with open(sample, "w", encoding="utf-8") as handle:
            handle.write("\n".join(PATH_SHAPED_SAMPLE_LINES))
        with open(only_paths, "w", encoding="utf-8") as handle:
            handle.write("只有路径：~/超级块/notes\n")
        os.utime(only_paths, (1, 1))
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 1 or tally.get("occurrences") != 6 or tally.get("path_shaped") != 4:
            return "--check 应当退 1、报 6 处其中路径形态 4 处，实际退 %s、%s 处、路径形态 %s 处" % (code, tally.get("occurrences"), tally.get("path_shaped"))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            sweep(root, True)
        applied_output = buffer.getvalue()
        actual = open(sample, encoding="utf-8").read().split("\n")
        want = PATH_SHAPED_SAMPLE_LINES[:2] + ["行文里的系统配置要换，路径旁的 系统配置 也要换"] + PATH_SHAPED_SAMPLE_LINES[3:]
        if actual != want:
            return "--apply 之后应当只换行文里的两处、路径形态的原样在：应当是 %r，实际 %r" % (want, actual)
        if open(only_paths, encoding="utf-8").read() != "只有路径：~/超级块/notes\n" or os.stat(only_paths).st_mtime != 1:
            return "只剩路径形态命中的文件 --apply 不该重写它（内容或修改时间变了）"
        for want_line in ("notes.md:1  /srv/超级块/data", "notes.md:2  超级块说明.md:12", "notes.md:4  crates/超级块/src/lib.rs", "only-paths.md:1  ~/超级块/notes"):
            if want_line not in applied_output:
                return "--apply 应当逐条报路径形态的命中「%s」（文件:行  那一段文字），输出：%s" % (want_line, applied_output.strip()[-400:])
        if "notes.md:3" in applied_output:
            return "行文里能换的那一行（第 3 行）不该被报成路径形态"
        buffer = io.StringIO()
        tally = {}
        with contextlib.redirect_stdout(buffer):
            code = sweep(root, False, tally=tally)
        checked_output = buffer.getvalue()
        if code != 1 or tally.get("occurrences") != 4 or tally.get("path_shaped") != 4 or "notes.md:1  /srv/超级块/data" not in checked_output:
            return "--apply 之后 --check 应当仍退 1、报 4 处且全是路径形态并逐条报文件:行，实际退 %s、%s 处、路径形态 %s 处" % (code, tally.get("occurrences"), tally.get("path_shaped"))
    return ""


SLASH_WORD_SAMPLE_LINES = ["行文里的超级块/读写 与 增删/超级块 是斜杠词，要换", "目录 ~/超级块/notes 与 ./超级块/x 与 ../超级块/y 与 /超级块/z 是路径", "文件 docs/超级块/说明.md 与网址 https://example.com/超级块 也是路径", ""]


def check_slash_words():
    """斜杠词那一格：带 `/` 不够算路径。行文里的斜杠词（超级块/读写）里的旧名照换；以 ~/、/、./、../ 起头的、带 :// 的网址、
    有一段像文件名的（docs/超级块/说明.md）仍是路径形态，--apply 不换、--check 照数。返回空串是判对。"""
    import tempfile
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, [("超级块", "系统配置", "整串", "中文")], [])
        sample = os.path.join(root, "notes.md")
        with open(sample, "w", encoding="utf-8") as handle:
            handle.write("\n".join(SLASH_WORD_SAMPLE_LINES))
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 1 or tally.get("occurrences") != 8 or tally.get("path_shaped") != 6:
            return "--check 应当退 1、报 8 处其中路径形态 6 处（斜杠词 2 处不算路径），实际退 %s、%s 处、路径形态 %s 处" % (code, tally.get("occurrences"), tally.get("path_shaped"))
        quietly(sweep, root, True)
        actual = open(sample, encoding="utf-8").read().split("\n")
        want = ["行文里的系统配置/读写 与 增删/系统配置 是斜杠词，要换"] + SLASH_WORD_SAMPLE_LINES[1:]
        if actual != want:
            return "--apply 之后应当只换斜杠词里的两处、路径形态的原样在：应当是 %r，实际 %r" % (want, actual)
    return ""


def check_line_exemptions():
    """按行豁免那一格：只豁免锚那一行，同一份文件别的行照判照换；锚命中 0 行、不止一行、指向不存在的文件都退 2。返回空串是判对。"""
    import tempfile
    with tempfile.TemporaryDirectory() as root:
        rows = [("超级块", "系统配置", "整串", "中文")]
        write_selftest_tables(root, rows, ["records/quote.md:「用户原话「超级块要改名」」  # 用户原话，换了就成假话"])
        os.makedirs(os.path.join(root, "records"))
        quote = os.path.join(root, "records", "quote.md")
        with open(quote, "w", encoding="utf-8") as handle:
            handle.write("\n".join(LINE_EXEMPTION_SAMPLE_LINES))
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 1 or tally.get("occurrences") != 2:
            return "同一份文件里没豁免的两行（第二、三行）--check 应当退 1、报 2 处，实际退 %s、%s 处" % (code, tally.get("occurrences"))
        quietly(sweep, root, True)
        lines = open(quote, encoding="utf-8").read().split("\n")
        if lines[0] != LINE_EXEMPTION_SAMPLE_LINES[0]:
            return "豁免的那一行被换了：%s" % lines[0]
        if lines[1:3] != ["第二行：系统配置的别处说法照新名", "第三行：再提一次系统配置"]:
            return "同一份文件里没豁免的两行没换：%s" % "／".join(lines[1:3])
        code = quietly(sweep, root, False)
        if code != 0:
            return "--apply 之后 --check 应当退 0，实际退 %s" % code
        for entry, why in (("records/quote.md:「没有这一句」", "行内锚命中 0 行"), ("records/quote.md:「行：」", "行内锚命中不止一行"),
                           ("records/nowhere.md:「用户原话」", "按行豁免指向不存在的文件"), ("records/quote.md:「用户原话 junk", "读不出行内锚")):
            write_selftest_tables(root, rows, ["%s  # 自证" % entry])
            code = quietly(sweep, root, False)
            if code != 2:
                return "%s时应当退 2，实际退 %s" % (why, code)
    return ""


OWN_FIXTURE_SAMPLE_RENAME_ROWS = [("超级块", "系统配置", "整串", "中文")]
OWN_FIXTURE_SAMPLE_FILE = os.path.join("crates", "singlefs-core", "src", "sample.rs")


def write_own_fixture_sample(base, with_tables):
    """照 term-renames 格的样本布局在 base 下写一份样本根：一份带旧名的 rs，with_tables 时另有自己的登记表（旧名在「旧」一列）与排除表。"""
    if with_tables:
        write_selftest_tables(base, OWN_FIXTURE_SAMPLE_RENAME_ROWS, [])
    os.makedirs(os.path.join(base, os.path.dirname(OWN_FIXTURE_SAMPLE_FILE)))
    with open(os.path.join(base, OWN_FIXTURE_SAMPLE_FILE), "w", encoding="utf-8") as handle:
        handle.write("// 超级块槽宽 4096\n")


def check_own_cell_fixtures():
    """自己那一格的样本：--apply 默认不扫门禁 doc-text 的 term-renames 格自己的样本目录（样本里的旧名原样在、仓里别处的换掉），
    在那份样本根上 --check 仍判红；别的格的样本目录、别的门禁文件名下的 term-renames-* 目录照扫照换。返回空串是判对。"""
    import tempfile
    sample_roots = [os.path.join(OWN_CELL_FIXTURE_PARENT, name) for name in ("term-renames-red", "term-renames-green")]
    swept_roots = [os.path.join(".claude", "gate.d", "fixtures", "doc-text.sh", "other-cell-red"),
                   os.path.join(".claude", "gate.d", "fixtures", "other.sh", "term-renames-red")]
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, OWN_FIXTURE_SAMPLE_RENAME_ROWS, [])
        elsewhere = os.path.join(root, "crates", "elsewhere.rs")
        os.makedirs(os.path.dirname(elsewhere))
        with open(elsewhere, "w", encoding="utf-8") as handle:
            handle.write("// 超级块在别处\n")
        for relative in sample_roots:
            write_own_fixture_sample(os.path.join(root, relative), True)
        for relative in swept_roots:
            write_own_fixture_sample(os.path.join(root, relative), False)
        quietly(sweep, root, True)
        original = "// 超级块槽宽 4096\n"
        for relative in sample_roots:
            kept = open(os.path.join(root, relative, OWN_FIXTURE_SAMPLE_FILE), encoding="utf-8").read()
            if kept != original:
                return "--apply 之后样本里的旧名被换掉了（%s 里的 sample.rs 变成 %r），红样本和绿样本换成了一样的内容" % (relative, kept)
        if open(elsewhere, encoding="utf-8").read() != "// 系统配置在别处\n":
            return "--apply 没换掉仓里别处的旧名"
        for relative in swept_roots:
            if open(os.path.join(root, relative, OWN_FIXTURE_SAMPLE_FILE), encoding="utf-8").read() != "// 系统配置槽宽 4096\n":
                return "%s 不是 term-renames 格自己的样本，应当照扫照换" % relative
        for relative in sample_roots:
            code = quietly(sweep, os.path.join(root, relative), False)
            if code != 1:
                return "在样本根 %s 上跑 --check 应当退 1（红样本仍然红），实际退 %s" % (relative, code)
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 0 or tally.get("excluded_fixture_directories") != 2:
            return "--apply 之后仓根上 --check 应当退 0、排除 2 个样本目录，实际退 %s、排除 %s 个" % (code, tally.get("excluded_fixture_directories"))
    return ""


def check_git_listing():
    """只扫 git 列出的那一格：git 跟踪的与未跟踪而未忽略的照判照换，git 忽略的不读不写。返回空串是判对。"""
    import subprocess
    import tempfile
    git_environment = dict(os.environ, GIT_AUTHOR_NAME="selftest", GIT_AUTHOR_EMAIL="selftest@example.invalid",
                           GIT_COMMITTER_NAME="selftest", GIT_COMMITTER_EMAIL="selftest@example.invalid")
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, [("超级块", "系统配置", "整串", "中文")], [])
        ignored = {"multi-host.env": "PEER_REPOSITORY_DIRECTORY=/home/peer/超级块\n",
                   os.path.join("scripts", "hosts", "memory-peaks.tsv"): "超级块\t1G\n"}
        os.makedirs(os.path.join(root, "scripts", "hosts"))
        with open(os.path.join(root, ".gitignore"), "w", encoding="utf-8") as handle:
            handle.write("multi-host.env\nmemory-peaks.tsv\n")
        for relative, text in ignored.items():
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(text)
        with open(os.path.join(root, "tracked.md"), "w", encoding="utf-8") as handle:
            handle.write("跟踪的：超级块\n")
        subprocess.run(["git", "-C", root, "init", "-q"], check=True, env=git_environment)
        subprocess.run(["git", "-C", root, "add", "-A"], check=True, env=git_environment)
        subprocess.run(["git", "-C", root, "-c", "commit.gpgsign=false", "commit", "-q", "-m", "样本"], check=True, env=git_environment)
        with open(os.path.join(root, "untracked.md"), "w", encoding="utf-8") as handle:
            handle.write("没跟踪也没忽略：超级块\n")
        tally = {}
        code = quietly(sweep, root, False, tally=tally)
        if code != 1 or tally.get("files") != 2:
            return ("git 忽略的 multi-host.env、memory-peaks.tsv 不该算进去（--check 应当退 1、只报跟踪的与未跟踪而未忽略的 2 份），"
                    "实际退 %s、%s 份" % (code, tally.get("files")))
        quietly(sweep, root, True)
        for relative, text in ignored.items():
            if open(os.path.join(root, relative), encoding="utf-8").read() != text:
                return "--apply 改了 git 忽略的 %s" % relative
        renamed = [open(os.path.join(root, name), encoding="utf-8").read() for name in ("tracked.md", "untracked.md")]
        if renamed != ["跟踪的：系统配置\n", "没跟踪也没忽略：系统配置\n"]:
            return "跟踪的与未跟踪而未忽略的应当都换掉，实际 %r" % renamed
        code = quietly(sweep, root, False)
        if code != 0:
            return "--apply 之后 --check 应当退 0，实际退 %s" % code
    return ""


def check_header_only_table():
    """表头不当记录那一格（C443）：登记表只有标记、表头与分隔行（0 条数据行）时，load_renames 不把表头「旧 | 新 | 匹配 | 是什么」当一条规则，
    --check 与 --apply 都报「一行数据都没有」退 2、不去全仓找「旧」，一份含「旧」「新」字样的文件原样不动；
    表头之下加一条数据行后，只解析出那一条。返回空串是判对。"""
    import contextlib
    import io
    import tempfile

    def loudly(function, *arguments):
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            try:
                result = function(*arguments)
            except SystemExit as stop:
                result = stop.code
        return result, buffer.getvalue()
    with tempfile.TemporaryDirectory() as root:
        write_selftest_tables(root, [], [])
        sample = os.path.join(root, "word.md")
        text = "旧的说法，新的说法，匹配与是什么都在这一份里\n"
        with open(sample, "w", encoding="utf-8") as handle:
            handle.write(text)
        for label, function, arguments in (("load_renames", load_renames, (root,)), ("--check", sweep, (root, False)), ("--apply", sweep, (root, True))):
            code, output = loudly(function, *arguments)
            if code != 2 or "一行数据都没有" not in output:
                return "只有标记、表头与分隔行的登记表，%s 应当退 2 并报「一行数据都没有」，实际退 %s：%s" % (label, code, output.strip()[-200:])
        if open(sample, encoding="utf-8").read() != text:
            return "只有表头的登记表，--apply 改了含「旧」「新」字样的文件：表头被当成了一条改名规则"
        write_selftest_tables(root, [("超级块", "系统配置", "整串", "中文")], [])
        pairs, output = loudly(load_renames, root)
        if pairs != [("超级块", "系统配置", "整串")]:
            return "表头之下一条数据行，load_renames 应当只解析出 [('超级块', '系统配置', '整串')]，实际 %r：%s" % (pairs, output.strip()[-200:])
    return ""


def check_argument_parsing():
    """参数解析那一格：小 git 仓里只有 deferred.md 带旧名，在仓的子目录里不带仓根跑 --check --defer deferred.md 退 0，
    不带 --defer 退 1（默认仓根确是当前目录所在的仓）；仓外的目录里把仓根写在 --defer 之后照样退 0；
    认不出的选项、--defer 后面没有路径都退 2。返回空串是判对。"""
    import subprocess
    import tempfile
    here = os.getcwd()
    with tempfile.TemporaryDirectory() as root, tempfile.TemporaryDirectory() as outside:
        subprocess.run(["git", "-C", root, "init", "-q"], check=True)
        write_selftest_tables(root, [("超级块", "系统配置", "整串", "中文")], [])
        os.makedirs(os.path.join(root, "sub"))
        with open(os.path.join(root, "deferred.md"), "w", encoding="utf-8") as handle:
            handle.write("这一份先放着：超级块\n")
        with open(os.path.join(root, "sub", "clean.md"), "w", encoding="utf-8") as handle:
            handle.write("系统配置\n")
        cases = (
            (os.path.join(root, "sub"), ["--check", "--defer", "deferred.md"], 0, "不带仓根、只带 --defer deferred.md"),
            (os.path.join(root, "sub"), ["--check"], 1, "不带仓根、不带 --defer（deferred.md 里的旧名该判红）"),
            (outside, ["--defer", "deferred.md", "--check", root], 0, "仓根写在 --defer 之后"),
            (os.path.join(root, "sub"), ["--check", "--no-such-option"], 2, "认不出的选项"),
            (os.path.join(root, "sub"), ["--check", "--defer"], 2, "--defer 后面没有路径"))
        try:
            for directory, argv, want, label in cases:
                os.chdir(directory)
                code = quietly(main, argv)
                if code != want:
                    return "%s（当前目录 %s，参数 %s）应当退 %d，实际退 %s" % (label, directory, " ".join(argv), want, code)
        finally:
            os.chdir(here)
    return ""


def apply_to_this_file_only(path):
    """自证用：把 path 所在的目录当仓根、登记一张只有「超级块 → 系统配置」的改名表，目录里别的条目全登记豁免，
    让 --apply 只换 path 这一份（经符号链接那一格就一定走链接那一个名字）。返回退出码。"""
    root = os.path.dirname(path)
    target_name = os.path.basename(path)
    other_names = sorted(name for name in os.listdir(root) if name not in (target_name, ".claude"))
    os.makedirs(os.path.join(root, ".claude", "kb"), exist_ok=True)
    with open(os.path.join(root, RENAMES_PATH), "w", encoding="utf-8") as handle:
        handle.write("%s\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n| 超级块 | 系统配置 | 整串 | 中文 |\n" % TABLE_MARK)
    with open(os.path.join(root, EXEMPT_FILE), "w", encoding="utf-8") as handle:
        handle.write("%s  # 表自己写着旧名\n" % RENAMES_PATH)
        for name in other_names:
            handle.write("%s  # 自证只换 %s 这一份\n" % (name, target_name))
    try:
        return sweep(root, True)
    except SystemExit as stop:
        return stop.code


def argument_error(problem):
    print("  ✗ 参数写错：%s" % problem)
    print("     → 怎么办：写成 sweep-term.py --check|--apply [<仓根>] [--defer <路径>]…（--defer 可写多次），或 --selftest；"
          "不给仓根就在要扫的仓里跑。")
    sys.exit(2)


def parse_arguments(argv):
    """选项与位置参数分开解析：--defer 吃掉紧跟它的那一个，剩下不以 - 起头的至多一个是仓根。
    返回 (仓根或 None, 是否 --apply, defer 列表)。弄坏开关 defer-value-as-root 照旧从全部参数里挑仓根，给自证证红。"""
    options, defer, positional = set(), [], []
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--defer":
            if index + 1 >= len(argv):
                argument_error("--defer 后面没有路径")
            defer.append(argv[index + 1])
            index += 2
            continue
        if argument in ("--check", "--apply"):
            options.add(argument)
        elif argument.startswith("-"):
            argument_error("认不出的选项 %s" % argument)
        else:
            positional.append(argument)
        index += 1
    if broken("defer-value-as-root"):
        positional = [a for a in argv if not a.startswith("--")]
    elif len(positional) > 1:
        argument_error("给了不止一个仓根：%s" % "、".join(positional))
    return (positional[0] if positional else None), "--apply" in options, defer


def root_of_current_directory():
    """不给仓根时取当前目录所在 git 仓的根；不在 git 仓里退 2。"""
    import subprocess
    try:
        probe = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    except OSError as error:
        argument_error("没给仓根，又起不了 git 判当前目录在哪个仓（%s）" % error)
    if probe.returncode != 0 or not probe.stdout.strip():
        argument_error("没给仓根，当前目录 %s 又不在 git 仓里" % os.getcwd())
    return probe.stdout.strip()


def main(argv):
    if "--selftest" in argv:
        return selftest()
    root_argument, apply_changes, defer = parse_arguments(argv)
    root = os.path.abspath(root_argument) if root_argument is not None else root_of_current_directory()
    if defer:
        print("  ⚠️ 这一批先放着（分批落地，不是豁免）：%s" % "、".join(defer))
    return sweep(root, apply_changes, defer)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
