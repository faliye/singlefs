#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓里代码与脚本的注释，上一次的结论不替这一次作保
# run-condition: command git
# gate-similar: .claude/gate.d/doc-text.sh 它的 dated-status、date-once、clock-times 三格也判带日期的写法，但对象分别是 kb 中文页里「现状」挨着日期的句子、这次改动碰到的记录与 kb 页里重复标题的日期、全仓文本里的钟点与时区词；这一份判代码与脚本的注释、文件头、文档串与 README 里的历史写法（日期、出处、改动经过），词表、取文本的方式与滞后表的键（路径、命中行数）都不同，只共用滞后表与 gate 基准比的读法（lib-lag-baseline.py），在同一道的 comment-history 格里调它
# gate-similar: .claude/singlefs-ai-sop/scripts/doc-lint.sh 它判登记的 kb 目录里的位置指代、时间指代与自称，不读代码注释；上游副本不许加本工程的判据
# gate-similar: .claude/singlefs-ai-sop/scripts/rules-lint.sh 它判规则文件里的写法（相对路径、引用），对象是规则正文，不读代码注释
# gate-similar: .claude/singlefs-ai-sop/scripts/code-name-lint.py 它判代码里的命名，读的是标识符，不读注释里的历史写法
# gate-similar: .claude/gate.d/lib-forbidden-notations.py 它登记全仓文本里的禁用写法（撇号类角标、钟点、会话号），逐行判、没有「只取注释」这一层，也没有滞后表；这一份的词表只在注释文字里判，两份词表不重叠
# gate-similar: .claude/singlefs-ai-sop/scripts/comment-only-change.py 它的切法交回的是去掉注释之后的词元流，不交注释的位置、行号与原文；这一份要的恰是每一处注释的行号与原文，另写一份切法
"""判代码与脚本的注释、文件头、文档串与 README 里有没有写历史。
判据是 .claude/agent-common.md「代码与脚本的注释、说明只写现状，不写历史」那一条；门禁 doc-text 的 comment-history 格只调这一份。

用法：
    comment-history-check.py [根]                                判；不给根就判这份脚本所在的仓
    comment-history-check.py --list <根>                         逐行打全部命中：路径:行号:命中的词:那一行去首尾空白后的原文
                                                                 （豁免表里的文件不扫，按标记放过的行不列，标记理由太短的行列出）
    comment-history-check.py --lag-table <根> [--out <文件>]     按此刻的命中现算滞后表全文：此刻的滞后表开头的注释行原样带上，
                                                                 再逐文件一行（路径、命中行数、理由）；给 --out 写到那份文件，不给打到标准输出
    comment-history-check.py --selftest                          自证：红绿格，再逐个打开弄坏开关
并行度（按文件切给几个进程）取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，登记位 .claude/heavy-thread-default），取不到退 2，不退回按核数算。

射程：被判的根下 git 跟踪的与未跟踪而未被忽略的文件（根不是 git 仓、或根不是那个仓的顶层时按目录遍历，跳过 .git 与 target），
  落在 crates/、scripts/、task-server/、prune-server/、.claude/hooks/、.claude/gate.d/、.claude/scripts/ 之下，路径里没有 target 一段，
  不在 .claude/gate.d/fixtures/ 之下（上游副本 .claude/singlefs-ai-sop/ 不在任何一个射程目录下），符号链接不判（成功句报个数）；
  取下面五类：.rs 与 .wgsl；.py；.sh；没有后缀而首行是 sh、bash、python 的 shebang 的脚本；文件名是 README.md 的。
取哪些文本：
  Rust 与 WGSL  行注释与块注释（含 ///、//!，块注释可嵌套）；字符串、原始字符串与字符字面量里的 // 不算
  Python        # 注释（首行 shebang 不算，字符串里的 # 不算）与模块、类、函数的文档串（用 ast 取，行号取串里那一行；语法读不了的文件文档串没取，成功句报份数）
  shell         # 注释：不在引号里、前面是行首或空白与 ; & | (（${#x}、$#、${x#y} 里的 # 不算）；首行 shebang 不算；
                heredoc 正文逐行照 # 认（里面嵌的 python 注释同样判）
  README.md     全文；围栏代码块（``` 与 ~~~）里的不判，围栏没收尾判红
  同一行只算一行（几处注释、一行里几个词都只记一处，词取最靠前的那一个）。
不许写的形态（命中即一处；Python re，词表只在这一份里写一处：DATE_PATTERN 加 WORD_PATTERNS）：
  日期 YYYY-MM-DD，前面不接 / _ 字母数字 -，后面不接「-」或「.」再接字母、汉字、_、/ 或直接接这些字符（路径与文件名里的日期是名字，不算；
    判法与弄坏开关 path-dates-flagged 的样本钉住两边）
  用户……定 / 说 / 要求 / 拍板 / 弹窗（「用户」之后十四个字以内）
  定案  攻方腿  辩方腿  正推腿  攻方 ?v<数字>  第<数>轮  判决第  第 ?<数>[a-z]? ?批
  改前  改之前  此前  原先  原为  起初  最初  以前  曾经  修过  改成了  换成了  改为了  旧做法  旧写法
  「不再」「原来」「后来」不判。
特殊缘由标记：那一行的注释（文档串、README 同一行同样认）里有 history-kept: <理由>，理由去掉空白后至少 8 个字，那一行放过；
  理由不够长、又有命中的那一行判红并点名（不进滞后表的计数）。
豁免表 .claude/gate.d/comment-history-exempt.tsv（被判的根下）：一行「<路径><TAB># <理由，至少 8 个字>」，整份文件不判；
  只给主题就是这些词的文件（这一格自己的判法脚本、词表文件、处理历史条目的脚本）；登记的路径要在射程里的文件中、不重复、两列。
滞后表 .claude/gate.d/comment-history-lag.tsv（被判的根下）：一行「<路径><TAB><命中行数><TAB># <理由，至少 4 个字>」：
  文件此刻命中数大于登记数判红（列出那份文件的全部命中）；小于登记数也判红（改小到现数，不留余量给新写的）；不在表里的文件命中数要是 0；
  与 gate 基准那一版比只缩不涨：基准里没有的行判红，数比基准大判红（基准那一版经 .claude/gate.d/lib-lag-baseline.py 的 gate_base_of 与 baseline_rows 取）；
  基准里整张表还不存在（这一格落地的那一次）时只判写法、不判涨；被判的根不是 git 仓时不比、成功句写明没比；
  列数不对、路径不在、数不是正整数、理由不到 4 个字、重复、路径在豁免表里，都判红。
成功句报：扫了多少份文件、多少行注释、命中几行、其中按标记放过几行、滞后几行；扫到 0 份文件退 77，不报绿。
出口：0 都对；1 有不对的（逐条列出，附怎么办）；2 用法错或弄坏开关没有放行；77 射程里一份文件都没有。
打印的路径与行文片段不经私有值打码（射程里是代码路径与注释，注释里写私有值由 private-values 格拦在写进去的时候）。

弄坏开关（只给自证的开关轮与样本根用：--selftest-inner 认；外面设了开关再起 --selftest 退 2，开关轮由 --selftest 自己逐个设上起 --selftest-inner；
  判一个样本根时根里要有 .gate-cells 或 .gate-break-sample，而且这个根不是这份脚本所在的仓、没设 GATE_STAGED_FROM；
  其余情形设了就拒，退 2 并点名只在自证里认。变量 COMMENT_HISTORY_CHECK_BREAK，值写一个开关名；
  单跑一条：COMMENT_HISTORY_CHECK_BREAK=<开关> python3 scripts/gate/comment-history-check.py --selftest-inner）。
  前十二条也列在 .claude/gate.d/doc-text.sh 文件头的开关表里，由门禁 doc-text 的样本复跑；后五条只在这一份自证里复跑：
  COMMENT_HISTORY_CHECK_BREAK=marker-ignored  不认 history-kept 标记：标记行照样按命中算
      comment-history-green
      comment-history-marker-short-red
      「✗ 自证 marker-long：」
  COMMENT_HISTORY_CHECK_BREAK=marker-reason-unchecked  标记理由不核长度：写什么都放过
      comment-history-marker-short-red
      「✗ 自证 marker-short：」
  COMMENT_HISTORY_CHECK_BREAK=docstrings-skipped  不取 Python 文档串
      comment-history-python-docstring-red
      「✗ 自证 python-docstring：」
  COMMENT_HISTORY_CHECK_BREAK=path-dates-flagged  日期不核前后字符：路径与文件名里的日期也判
      comment-history-green
      「✗ 自证 path-dates：」
  COMMENT_HISTORY_CHECK_BREAK=strings-as-comments  字符串里的注释符也当注释
      comment-history-strings-green
      「✗ 自证 strings-not-comments：」
  COMMENT_HISTORY_CHECK_BREAK=lag-growth-unjudged  滞后表不与 gate 基准那一版比
      comment-history-lag-growth-red
      「✗ 自证 lag-growth：」
  COMMENT_HISTORY_CHECK_BREAK=lag-slack-allowed  命中少于登记的不判红
      comment-history-lag-slack-red
      「✗ 自证 lag-slack：」
  COMMENT_HISTORY_CHECK_BREAK=lag-table-form-unchecked  滞后表的行不核写法（列数、数、理由、重复、路径在不在），只取能取到的路径与数
      comment-history-lag-table-red
      「✗ 自证 lag-table-form：」
  COMMENT_HISTORY_CHECK_BREAK=exempt-table-form-unchecked  豁免表的行不核写法，路径在射程里就放过
      comment-history-exempt-table-red
      「✗ 自证 exempt-table-form：」
  COMMENT_HISTORY_CHECK_BREAK=exempt-table-ignored  不读豁免表
      comment-history-green
      comment-history-exempt-table-red
      「✗ 自证 exempt-table：」
  COMMENT_HISTORY_CHECK_BREAK=shebang-unscoped  没有后缀的 shebang 脚本不在射程里
      comment-history-shebang-red
      「✗ 自证 shebang-script：」
  COMMENT_HISTORY_CHECK_BREAK=readme-fences-judged  README 的围栏代码块里也判
      comment-history-green
      「✗ 自证 readme-fence：」
  COMMENT_HISTORY_CHECK_BREAK=rust-comments-skipped  不取 .rs 里的注释
      「✗ 自证 rust-line-date：」
  COMMENT_HISTORY_CHECK_BREAK=wgsl-skipped  不取 .wgsl 里的注释
      「✗ 自证 wgsl：」
  COMMENT_HISTORY_CHECK_BREAK=python-comments-skipped  不取 .py 与 python shebang 脚本里的 # 注释
      「✗ 自证 python-hash：」
  COMMENT_HISTORY_CHECK_BREAK=shell-comments-skipped  不取 shell 里的 # 注释
      「✗ 自证 shell-hash：」
  COMMENT_HISTORY_CHECK_BREAK=readme-skipped  不取 README.md
      「✗ 自证 readme：」

样本：.claude/gate.d/fixtures/doc-text.sh/comment-history-*（每份只点名 comment-history 一格，写法与每份判什么在 doc-text.sh 文件头「comment-history」一段）。
管不到的：字符串里的话（日期、历史词写在字符串、测试输入里的不判）；注释里不在词表里的历史说法（「后来」「不再」「原来」这类，靠 review）；
  围栏之外的 Markdown 缩进代码块；Python 文档串里续行符之后的行号可能差一两行（只影响打印的行号）；
  shell 的引号状态靠逐行扫（跨 heredoc 的引号、嵌套的 $( ) 里的引号只近似），误判与漏判都落在注释符上，不落在字符串里的话上。
"""
import ast
import concurrent.futures
import importlib.util
import multiprocessing
import os
import re
import subprocess
import sys
import tempfile
import warnings

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
import heavy_threads  # noqa: E402

LAG_BASELINE_LIBRARY = os.path.join(REPOSITORY_ROOT, ".claude", "gate.d", "lib-lag-baseline.py")
BREAK_VARIABLE = "COMMENT_HISTORY_CHECK_BREAK"
STAGE_BREAKS = ("marker-ignored", "marker-reason-unchecked", "docstrings-skipped", "path-dates-flagged", "strings-as-comments",
                "lag-growth-unjudged", "lag-slack-allowed", "lag-table-form-unchecked", "exempt-table-form-unchecked",
                "exempt-table-ignored", "shebang-unscoped", "readme-fences-judged")
SELFTEST_BREAKS = ("rust-comments-skipped", "wgsl-skipped", "python-comments-skipped", "shell-comments-skipped", "readme-skipped")
KNOWN_BREAKS = STAGE_BREAKS + SELFTEST_BREAKS

LAG_TABLE = ".claude/gate.d/comment-history-lag.tsv"
EXEMPT_TABLE = ".claude/gate.d/comment-history-exempt.tsv"
JUDGED_ROOTS = ("crates/", "scripts/", "task-server/", "prune-server/", ".claude/hooks/", ".claude/gate.d/", ".claude/scripts/")
EXCLUDED_PREFIXES = (".claude/gate.d/fixtures/",)
SKIPPED_DIRECTORIES = (".git", "target", "__pycache__")
SUFFIX_KINDS = {".rs": "rust", ".wgsl": "wgsl", ".py": "python", ".sh": "shell"}
README_NAME = "README.md"
SHEBANG = re.compile(r"^#!\s*(?:\S+/)?(?:env\s+(?:-\S+\s+)*)?(sh|bash|python[0-9.]*)(?:\s|$)")

DATE_PATTERN = r"(?<![/_A-Za-z0-9-])\d{4}-\d\d-\d\d(?!-[^\d\s]|\.[A-Za-z0-9]|[/_A-Za-z])"
LOOSE_DATE_PATTERN = r"\d{4}-\d\d-\d\d"
WORD_PATTERNS = (
    r"用户.{0,14}?(定|说|要求|拍板|弹窗)",
    "定案", "攻方腿", "辩方腿", "正推腿", r"攻方 ?v\d", r"第[一二三四五六七八九十百\d]+轮", "判决第", r"第 ?\d+[a-z]? ?批",
    "改前", "改之前", "此前", "原先", "原为", "起初", "最初", "以前", "曾经", "修过", "改成了", "换成了", "改为了", "旧做法", "旧写法",
)
HISTORY = re.compile("|".join((DATE_PATTERN,) + WORD_PATTERNS))
HISTORY_LOOSE_DATE = re.compile("|".join((LOOSE_DATE_PATTERN,) + WORD_PATTERNS))
MARKER = re.compile(r"history-kept:\s*(.*)$")
MINIMUM_MARKER_REASON = 8
MINIMUM_EXEMPT_REASON = 8
MINIMUM_LAG_REASON = 4
LAG_ROW_REASON = "# 存量，改净后删这一行"
LISTED_LINES = 200
SOURCE_SHOWN_CHARACTERS = 120

RUST_TOKEN = re.compile(r"""//|/\*|(?<!\w)(?:b|c)?r(#*)"|"|'""")
RUST_BLOCK_EDGE = re.compile(r"/\*|\*/")
RUST_STRING_END = re.compile(r'[^"\\]*(?:\\.[^"\\]*)*"', re.S)
PYTHON_TOKEN = re.compile(r"""#|\"\"\"|'''|"|'""")
PYTHON_STRING_END = {
    '"""': re.compile(r'[^"\\]*(?:(?:\\.|"(?!""))[^"\\]*)*"""', re.S),
    "'''": re.compile(r"[^'\\]*(?:(?:\\.|'(?!''))[^'\\]*)*'''", re.S),
    '"': re.compile(r'[^"\\\n]*(?:\\.[^"\\\n]*)*"', re.S),
    "'": re.compile(r"[^'\\\n]*(?:\\.[^'\\\n]*)*'", re.S),
}
SHELL_UNQUOTED = re.compile(r"""\\.|'|"|#|<<<?""")
SHELL_DOUBLE = re.compile(r'\\.|"')
SHELL_HEREDOC_TAG = re.compile(r"""(-?)[ \t]*(?:'([^']+)'|"([^"]+)"|\\?([A-Za-z_][A-Za-z0-9_]*))""")
SHELL_COMMENT_PRECEDERS = " \t;&|("
README_FENCE_OPEN = re.compile(r"^ {0,3}(?:(`{3,})[^`]*|(~{3,}).*)$")
README_FENCE_CLOSE = re.compile(r"^ {0,3}(`{3,}|~{3,})[ \t]*$")


def add_text(found, line, text):
    """同一行的几段文字并成一段。"""
    found[line] = found[line] + " " + text if line in found else text


def rust_comments(text, switches):
    """Rust 与 WGSL：{行号: 注释文字}；行注释去掉 // 与紧接的 / 或 !，块注释逐行。"""
    found = {}
    length = len(text)
    counted = [1, 0]

    def line_of(position):
        counted[0] += text.count("\n", counted[1], position)
        counted[1] = position
        return counted[0]

    string_aware = "strings-as-comments" not in switches
    position = 0
    while True:
        match = RUST_TOKEN.search(text, position)
        if match is None:
            return found
        start, token = match.start(), match.group()
        if token == "//":
            end = text.find("\n", start)
            end = length if end < 0 else end
            add_text(found, line_of(start), text[start + 2:end].lstrip("/!"))
            position = end
        elif token == "/*":
            depth, scan = 1, start + 2
            while depth:
                edge = RUST_BLOCK_EDGE.search(text, scan)
                if edge is None:
                    scan = length
                    break
                depth += 1 if edge.group() == "/*" else -1
                scan = edge.end()
            body = text[start + 2:scan - 2] if depth == 0 else text[start + 2:]
            first = line_of(start)
            for offset, piece in enumerate(body.split("\n")):
                add_text(found, first + offset, piece)
            position = scan
        elif token == "'":
            if text.startswith("\\", start + 1):
                close = text.find("'", start + 3)
                position = length if close < 0 else close + 1
            elif text.startswith("'", start + 2):
                position = start + 3
            else:
                position = start + 1
        elif not string_aware:
            position = match.end()
        elif token == '"':
            closing = RUST_STRING_END.match(text, start + 1)
            position = closing.end() if closing else length
        else:
            raw_closing = '"' + match.group(1)
            end = text.find(raw_closing, match.end())
            position = length if end < 0 else end + len(raw_closing)


def python_comments(text, switches):
    """Python：{行号: # 注释文字}；首行 shebang 不算。"""
    found = {}
    length = len(text)
    counted = [1, 0]

    def line_of(position):
        counted[0] += text.count("\n", counted[1], position)
        counted[1] = position
        return counted[0]

    string_aware = "strings-as-comments" not in switches
    position = 0
    while True:
        match = PYTHON_TOKEN.search(text, position)
        if match is None:
            return found
        start, token = match.start(), match.group()
        if token == "#":
            end = text.find("\n", start)
            end = length if end < 0 else end
            if not (start == 0 and text.startswith("#!")):
                add_text(found, line_of(start), text[start + 1:end])
            position = end
        elif not string_aware:
            position = match.end()
        else:
            closing = PYTHON_STRING_END[token].match(text, match.end())
            if closing:
                position = closing.end()
            else:
                position = length if len(token) == 3 else match.end()


def python_docstrings(text):
    """模块、类、函数的文档串：{行号: 文字}；语法读不了交 None。"""
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            tree = ast.parse(text)
    except (SyntaxError, ValueError, RecursionError, MemoryError):
        return None
    found = {}
    for node in ast.walk(tree):
        if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
            body = node.body
            if body and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
                constant = body[0].value
                for offset, piece in enumerate(constant.value.split("\n")):
                    add_text(found, constant.lineno + offset, piece)
    return found


def scan_shell_line(line, quote, quotes_matter):
    """一行 shell：(注释文字或 None, 行尾的引号状态, 这一行里开出的 heredoc [(标记, 是不是 <<-)])。"""
    heredocs = []
    position = 0
    while position < len(line):
        if quote == "'":
            close = line.find("'", position)
            if close < 0:
                return None, quote, heredocs
            quote, position = None, close + 1
            continue
        match = (SHELL_DOUBLE if quote == '"' else SHELL_UNQUOTED).search(line, position)
        if match is None:
            break
        token, start = match.group(), match.start()
        position = match.end()
        if quote == '"':
            if token == '"':
                quote = None
            continue
        if token[0] == "\\":
            continue
        if token in ("'", '"'):
            if quotes_matter:
                quote = token
            continue
        if token == "#":
            if start == 0 or line[start - 1] in SHELL_COMMENT_PRECEDERS:
                return line[start + 1:], quote, heredocs
            continue
        if token == "<<":
            tag = SHELL_HEREDOC_TAG.match(line, position)
            if tag:
                heredocs.append((tag.group(2) or tag.group(3) or tag.group(4), tag.group(1) == "-"))
                position = tag.end()
    return None, quote, heredocs


def shell_comments(text, switches):
    """shell：{行号: # 注释文字}；heredoc 正文逐行照 # 认。"""
    found = {}
    quotes_matter = "strings-as-comments" not in switches
    quote, heredoc, queue = None, None, []
    for number, line in enumerate(text.split("\n"), 1):
        if number == 1 and line.startswith("#!"):
            continue
        if heredoc is not None:
            tag, dash = heredoc
            bare = line.rstrip("\r")
            if bare == tag or (dash and bare.lstrip("\t") == tag):
                heredoc = queue.pop(0) if queue else None
                continue
            comment, _quote, _nested = scan_shell_line(line, None, quotes_matter)
        else:
            comment, quote, opened = scan_shell_line(line, quote, quotes_matter)
            queue.extend(opened)
        if comment is not None:
            add_text(found, number, comment)
        if heredoc is None and queue:
            heredoc = queue.pop(0)
    return found


def readme_text(text, switches):
    """README：({行号: 文字}, 没收尾的围栏起于第几行或 None)；围栏代码块里的不取。"""
    found = {}
    fence, opened = None, 0
    for number, line in enumerate(text.split("\n"), 1):
        if fence is None:
            opening = README_FENCE_OPEN.match(line)
            if opening and "readme-fences-judged" not in switches:
                fence, opened = opening.group(1) or opening.group(2), number
                continue
            if line.strip():
                add_text(found, number, line)
        else:
            closing = README_FENCE_CLOSE.match(line)
            if closing and closing.group(1)[0] == fence[0] and len(closing.group(1)) >= len(fence):
                fence = None
    return found, (opened if fence is not None else None)


def analyse(text, kind, switches):
    """一份文件：{comment_lines, hits [(行号, 词, 原文)], marked [行号], short [(行号, 理由, 原文)], unparsed, open_fence}。"""
    lines = text.split("\n")
    found, unparsed, open_fence = {}, False, None
    if kind == "readme":
        if "readme-skipped" not in switches:
            found, open_fence = readme_text(text, switches)
    elif kind == "rust":
        if "rust-comments-skipped" not in switches:
            found = rust_comments(text, switches)
    elif kind == "wgsl":
        if "wgsl-skipped" not in switches:
            found = rust_comments(text, switches)
    elif kind == "python":
        if "python-comments-skipped" not in switches:
            found = python_comments(text, switches)
        if "docstrings-skipped" not in switches:
            documents = python_docstrings(text)
            if documents is None:
                unparsed = True
            else:
                for number, piece in documents.items():
                    add_text(found, number, piece)
    else:
        if "shell-comments-skipped" not in switches:
            found = shell_comments(text, switches)
    pattern = HISTORY_LOOSE_DATE if "path-dates-flagged" in switches else HISTORY
    comment_lines, hits, marked, short = 0, [], [], []
    for number in sorted(found):
        piece = found[number]
        if not piece.strip():
            continue
        comment_lines += 1
        match = pattern.search(piece)
        if match is None:
            continue
        source = lines[number - 1].strip() if 0 < number <= len(lines) else piece.strip()
        marker = None if "marker-ignored" in switches else MARKER.search(piece)
        if marker is None:
            hits.append((number, match.group(), source))
        elif len(re.sub(r"\s+", "", marker.group(1))) >= MINIMUM_MARKER_REASON or "marker-reason-unchecked" in switches:
            marked.append(number)
        else:
            short.append((number, marker.group(1).strip(), source))
    return {"comment_lines": comment_lines, "hits": hits, "marked": marked, "short": short, "unparsed": unparsed, "open_fence": open_fence}


def scan_one(task):
    """(根, 路径, 种类, 开关) → (路径, 分析结果或 None, 读不了的原因或 None)；读文件与分析的异常都交成原因，不让整趟崩。"""
    root, path, kind, switches = task
    try:
        with open(os.path.join(root, path), encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        return path, analyse(text, kind, switches), None
    except (OSError, ValueError, RecursionError, MemoryError) as error:
        return path, None, str(getattr(error, "strerror", None) or error)


def run_tasks(tasks, jobs):
    if jobs <= 1 or len(tasks) < 64:
        return [scan_one(task) for task in tasks]
    with concurrent.futures.ProcessPoolExecutor(max_workers=jobs, mp_context=multiprocessing.get_context("fork")) as pool:
        return list(pool.map(scan_one, tasks, chunksize=16))


def tracked_paths(root):
    """根下的文件（相对根、用 / 分隔）：根是 git 仓的顶层时取跟踪的与未跟踪而未忽略的，否则只在射程目录里走目录树。"""
    top = subprocess.run(["git", "-C", root, "rev-parse", "--show-toplevel"], capture_output=True, text=True)
    if top.returncode == 0 and os.path.realpath(top.stdout.strip()) == os.path.realpath(root):
        listed = subprocess.run(["git", "-c", "core.quotepath=false", "-C", root, "ls-files", "-co", "--exclude-standard", "-z"],
                                capture_output=True, text=True)
        if listed.returncode == 0:
            return [path for path in listed.stdout.split("\0") if path]
    found = []
    for judged in JUDGED_ROOTS:
        top_directory = os.path.join(root, judged.rstrip("/"))
        for current, subdirectories, names in os.walk(top_directory):
            subdirectories[:] = sorted(name for name in subdirectories if name not in SKIPPED_DIRECTORIES)
            for name in sorted(names):
                found.append(os.path.relpath(os.path.join(current, name), root).replace(os.sep, "/"))
    return found


def in_range(path):
    if not path.startswith(JUDGED_ROOTS) or path.startswith(EXCLUDED_PREFIXES):
        return False
    return "target" not in path.split("/")[:-1]


def classify(full, path, switches):
    """这份文件取哪一种文本：rust、wgsl、python、shell、readme，或 None（不在射程）。"""
    name = path.rsplit("/", 1)[-1]
    if name == README_NAME:
        return "readme"
    suffix = os.path.splitext(name)[1]
    if suffix in SUFFIX_KINDS:
        return SUFFIX_KINDS[suffix]
    if suffix == "" and "shebang-unscoped" not in switches:
        try:
            with open(full, "rb") as handle:
                first_line = handle.readline(200).decode("utf-8", "replace")
        except OSError:
            return None
        found = SHEBANG.match(first_line)
        if found:
            return "python" if found.group(1).startswith("python") else "shell"
    return None


def list_candidates(root, switches):
    """([(路径, 种类)] 排好序, 符号链接数)。"""
    candidates, symlinks = [], 0
    for path in tracked_paths(root):
        if not in_range(path):
            continue
        full = os.path.join(root, path)
        if os.path.islink(full):
            symlinks += 1
            continue
        if not os.path.isfile(full):
            continue
        kind = classify(full, path, switches)
        if kind is not None:
            candidates.append((path, kind))
    return sorted(set(candidates)), symlinks


LOADED_LAG_BASELINE = []


def load_lag_baseline():
    """滞后表与基准比的共用读法（lib-lag-baseline.py），一个进程里只载入一次。"""
    if not LOADED_LAG_BASELINE:
        specification = importlib.util.spec_from_file_location("lib_lag_baseline", LAG_BASELINE_LIBRARY)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        LOADED_LAG_BASELINE.append(library)
    return LOADED_LAG_BASELINE[0]


def reason_long_enough(text, minimum):
    text = text.strip()
    return text.startswith("#") and len(text.lstrip("#").strip()) >= minimum


def read_table_rows(root, table):
    """表的内容行 [(行号, 各列)]，表不在交 None。"""
    path = os.path.join(root, table)
    if not os.path.isfile(path):
        return None
    with open(path, encoding="utf-8", errors="replace") as handle:
        return load_lag_baseline().content_rows(handle.read())


def read_exempt_table(root, scope, switches):
    """(整份不判的路径集合, [(表的行号, 一句问题)])。"""
    rows = None if "exempt-table-ignored" in switches else read_table_rows(root, EXEMPT_TABLE)
    exempt, problems, seen = set(), [], {}
    lenient = "exempt-table-form-unchecked" in switches
    for number, columns in rows or []:
        path = columns[0].strip()
        problem = None
        if len(columns) != 2:
            problem = f"有 {len(columns)} 列，要两列：<路径><TAB># <理由>"
        elif not reason_long_enough(columns[1], MINIMUM_EXEMPT_REASON):
            problem = f"第二列要以 # 起头、写为什么整份不判（至少 {MINIMUM_EXEMPT_REASON} 个字）"
        elif path in seen:
            problem = f"{path} 与第 {seen[path]} 行重复登记"
        elif path not in scope:
            problem = f"登记的路径不在判法射程里的文件中：{path}"
        if lenient:
            if path in scope:
                exempt.add(path)
        elif problem:
            problems.append((number, problem))
        else:
            seen[path] = number
            exempt.add(path)
    return exempt, problems


def read_lag_table(root, scope, exempt, switches):
    """(表里有没有这个文件, 此刻表的内容行数, {路径: (登记数, 表的行号)}, [(表的行号, 一句问题)])。"""
    rows = read_table_rows(root, LAG_TABLE)
    if rows is None:
        return False, 0, {}, []
    accepted, problems = {}, []
    lenient = "lag-table-form-unchecked" in switches
    for number, columns in rows:
        path = columns[0].strip()
        count_text = columns[1].strip() if len(columns) > 1 else ""
        counted = re.fullmatch(r"[1-9][0-9]*", count_text) is not None
        if lenient:
            if len(columns) > 1 and counted and path in scope and path not in accepted:
                accepted[path] = (int(count_text), number)
            continue
        problem = None
        if len(columns) != 3:
            problem = f"有 {len(columns)} 列，要三列：<路径><TAB><命中行数><TAB># <理由>"
        elif not counted:
            problem = f"命中行数要是正整数，写的是「{count_text}」"
        elif not reason_long_enough(columns[2], MINIMUM_LAG_REASON):
            problem = f"第三列要以 # 起头、写为什么还没改（至少 {MINIMUM_LAG_REASON} 个字）"
        elif path in accepted:
            problem = f"{path} 与第 {accepted[path][1]} 行重复登记"
        elif path not in scope:
            problem = f"登记的路径不在判法射程里的文件中：{path}"
        elif path in exempt:
            problem = f"{path} 在豁免表里（整份不判），不登记进滞后表"
        if problem:
            problems.append((number, problem))
        else:
            accepted[path] = (int(count_text), number)
    return True, len(rows), accepted, problems


def lag_growth(root, accepted, switches, current_rows):
    """(涨了的 [(表的行号, 路径, 一句)], 基准那一句说明, 取不到基准的问题或 None, 基准的名字或 "")。"""
    if "lag-growth-unjudged" in switches:
        return [], "弄坏开关 lag-growth-unjudged：没与 gate 基准比", None, ""
    library = load_lag_baseline()
    base, problem = library.gate_base_of(root)
    if problem:
        return [], "", f"与 gate 基准比不了：{problem}", ""
    if base is None:
        return [], "被判的根不在 git 仓里，滞后表没与 gate 基准比", None, ""
    base_rows, base_problem = library.baseline_rows(base, LAG_TABLE, cwd=root)
    if base_problem is not None:
        return [], "", f"与 gate 基准比不了：{base_problem.sentence()}", base
    if base_rows is None:
        return [], f"gate 基准 {base} 里还没有滞后表（初次登记），只判写法、不判涨", None, base
    base_counts = {}
    for _number, columns in base_rows:
        if len(columns) >= 2 and columns[1].strip().isdigit():
            base_counts.setdefault(columns[0].strip(), int(columns[1].strip()))
    grown = []
    for path, (count, number) in sorted(accepted.items(), key=lambda item: item[1][1]):
        if path not in base_counts:
            grown.append((number, path, "基准里没有这一行"))
        elif count > base_counts[path]:
            grown.append((number, path, f"登记 {count} 行，基准里是 {base_counts[path]} 行"))
    return grown, f"与 gate 基准 {base} 那一版比没涨（基准 {len(base_rows)} 行，此刻 {current_rows} 行）", None, base


def scan_root(root, switches, jobs):
    """扫一棵树：文件、豁免表、符号链接数、读不了的、没取文档串的、围栏没收尾的。"""
    candidates, symlinks = list_candidates(root, switches)
    scope = {path for path, _kind in candidates}
    exempt, exempt_problems = read_exempt_table(root, scope, switches)
    tasks = [(root, path, kind, switches) for path, kind in candidates if path not in exempt]
    files, unreadable, unparsed, open_fences = {}, [], [], []
    for path, outcome, error in run_tasks(tasks, jobs):
        if error is not None:
            unreadable.append((path, error))
            continue
        files[path] = outcome
        if outcome["unparsed"]:
            unparsed.append(path)
        if outcome["open_fence"]:
            open_fences.append((path, outcome["open_fence"]))
    return {"files": files, "scope": scope, "exempt": exempt, "exempt_problems": exempt_problems, "symlinks": symlinks,
            "unreadable": unreadable, "unparsed": unparsed, "open_fences": open_fences}


def shown_source(source):
    return source if len(source) <= SOURCE_SHOWN_CHARACTERS else source[:SOURCE_SHOWN_CHARACTERS]


def list_lines(scan):
    """--list 的行：路径:行号:命中的词:原文，标记理由太短的行也列。"""
    rows = []
    for path in sorted(scan["files"]):
        info = scan["files"][path]
        rows.extend((path, number, word, source) for number, word, source in info["hits"])
        rows.extend((path, number, "（标记理由太短）", source) for number, _reason, source in info["short"])
    return [f"{path}:{number}:{word}:{source}" for path, number, word, source in sorted(rows, key=lambda row: (row[0], row[1]))]


def lag_table_text(scan, root):
    """--lag-table：此刻的滞后表开头的注释行，加逐文件一行。"""
    header = []
    path = os.path.join(root, LAG_TABLE)
    if os.path.isfile(path):
        with open(path, encoding="utf-8", errors="replace") as handle:
            for line in handle.read().split("\n"):
                if not line.startswith("#"):
                    break
                header.append(line)
    rows = [f"{path}\t{len(info['hits'])}\t{LAG_ROW_REASON}" for path, info in sorted(scan["files"].items()) if info["hits"]]
    return header + rows


def remedy(lines, texts):
    lines.append("     → 怎么办：" + texts[0])
    for text in texts[1:]:
        lines.append("               " + text)


HITS_REMEDY = (
    "改成只写现状：删掉日期、谁定的与哪一轮打中的、改动经过、旧值与旧做法；要指依据写现行登记位的编号与名字（决策分项、欠账号、不变量号、规则文件与小节标题）；",
    "历史的落点：commit message、.claude/kb/decisions-history.md、.claude/kb/experiments-history.md、records/（.claude/agent-common.md「代码与脚本的注释、说明只写现状，不写历史」那一条）；",
    "确属特殊缘由（日期本身是数据、引外部文档写读到它的那一天、绕开外部工具缺陷写从哪个版本起）的，在那一行注释里写 history-kept: <理由，至少 8 个字>；",
    f"存量一时改不完的照 {LAG_TABLE} 的写法登记（只缩不涨，新写的不许往里加）；整份文件只讲这些词的登记进 {EXEMPT_TABLE}；逐行看命中：python3 scripts/gate/comment-history-check.py --list <根>。",
)


def judge(root, switches, jobs):
    """判一棵树：(退出码, 输出行)。"""
    scan = scan_root(root, switches, jobs)
    files = scan["files"]
    lag_present, lag_rows, accepted, lag_problems = read_lag_table(root, scan["scope"], scan["exempt"], switches)
    grown, base_note, base = [], "没有滞后表", ""
    if lag_present:
        grown, base_note, base_problem, base = lag_growth(root, accepted, switches, lag_rows)
        if base_problem:
            lag_problems.append((0, base_problem))
    red_files, slack, lagged = [], [], 0
    marked_total = short_total = unmarked_total = comment_lines = 0
    for path in sorted(files):
        info = files[path]
        count = len(info["hits"])
        comment_lines += info["comment_lines"]
        marked_total += len(info["marked"])
        short_total += len(info["short"])
        unmarked_total += count
        registered = accepted.get(path)
        if count == 0 and registered is None:
            continue
        if registered is None or count > registered[0]:
            if count:
                red_files.append(path)
        elif count == registered[0]:
            lagged += count
        elif "lag-slack-allowed" in switches:
            lagged += count
        else:
            slack.append((path, registered[0], count, registered[1]))
    short_files = [path for path in sorted(files) if files[path]["short"]]
    lines = []
    if red_files:
        total = sum(len(files[path]["hits"]) for path in red_files)
        lines.append(f"  ✗ {len(red_files)} 份文件的注释里写了历史，共 {total} 行（词表命中，没有 history-kept 标记，滞后表也放不过：没登记，或此刻命中比登记的多；这几份文件的全部命中都列在下面）：")
        listed = 0
        for path in red_files:
            for number, word, source in files[path]["hits"]:
                if listed < LISTED_LINES:
                    lines.append(f"      {path}:{number}  {word}  …{shown_source(source)}…")
                listed += 1
        if listed > LISTED_LINES:
            lines.append(f"      （另有 {listed - LISTED_LINES} 行没列出，每份文件的行数见下）")
            for path in red_files[:LISTED_LINES]:
                lines.append(f"      {path}：{len(files[path]['hits'])} 行")
        remedy(lines, HITS_REMEDY)
    if short_files:
        short_total_lines = sum(len(files[path]["short"]) for path in short_files)
        lines.append(f"  ✗ {short_total_lines} 行的 history-kept 标记理由不到 {MINIMUM_MARKER_REASON} 个字（去掉空白后算），这几行不放过：")
        for path in short_files:
            for number, reason, _source in files[path]["short"]:
                reason_length = len(re.sub(r"\s+", "", reason))
                lines.append(f"      {path}:{number}  理由「{reason}」只有 {reason_length} 个字")
        remedy(lines, (f"标记写成 history-kept: <理由>，理由至少 {MINIMUM_MARKER_REASON} 个字，说清是哪一类特殊缘由（日期本身是数据、引外部文档的读取日、绕开外部工具缺陷的版本）；",
                       "「留着方便查来历」不算理由，那种话改写成现状或删掉。"))
    if slack:
        lines.append(f"  ✗ 滞后表登记的行数比此刻命中多，{len(slack)} 份文件的登记要改小（不留余量给新写的）：")
        for path, registered, count, number in slack:
            what = f"改小到 {count}" if count else "改小到 0，即删掉这一行"
            lines.append(f"      {LAG_TABLE} 第 {number} 行：{path}：登记 {registered} 行，此刻命中 {count} 行 → {what}")
        remedy(lines, (f"改好了几行就把那份文件在 {LAG_TABLE} 里的数改成此刻的命中数，改净了就删那一行；只缩不涨。",))
    if lag_problems:
        lines.append(f"  ✗ 滞后表 {LAG_TABLE} 有 {len(lag_problems)} 处写得不对：")
        for number, sentence in lag_problems:
            lines.append(f"      {'第 ' + str(number) + ' 行：' if number else ''}{sentence}")
        remedy(lines, (f"一行「<路径><TAB><命中行数，正整数><TAB># <为什么还没改，至少 {MINIMUM_LAG_REASON} 个字>」；路径要是射程里的文件、不重复、不在豁免表里；",
                       "路径不在多半是文件改了名或删了，删掉那一行；初始表由 python3 scripts/gate/comment-history-check.py --lag-table <根> 现算。"))
    if scan["exempt_problems"]:
        lines.append(f"  ✗ 豁免表 {EXEMPT_TABLE} 有 {len(scan['exempt_problems'])} 处写得不对：")
        for number, sentence in scan["exempt_problems"]:
            lines.append(f"      第 {number} 行：{sentence}")
        remedy(lines, (f"一行「<路径><TAB># <为什么整份不判，至少 {MINIMUM_EXEMPT_REASON} 个字>」；路径要是射程里的文件、不重复；只给主题就是这些词的文件，存量不往这里放。",))
    if grown:
        lines.append(f"  ✗ 滞后表比 gate 基准 {base} 那一版涨了 {len(grown)} 处（只缩不涨；路径改了也算新加）：")
        for number, path, sentence in grown:
            lines.append(f"      第 {number} 行：{path}  {sentence}")
        remedy(lines, ("新写的历史写法当场改成现状，不往滞后表里加；改过名的文件，在同一次改动里把那一份改净、删它那一行。",))
    if scan["open_fences"]:
        lines.append(f"  ✗ {len(scan['open_fences'])} 份 README 的围栏代码块没收尾，围栏之后的内容没判：")
        for path, number in scan["open_fences"]:
            lines.append(f"      {path}:{number}")
        remedy(lines, ("在围栏代码块结束的地方补上收尾的围栏（与起头的同一种符号、不少于起头的个数）。",))
    if scan["unreadable"]:
        lines.append(f"  ✗ {len(scan['unreadable'])} 份文件读不了，这几份没判：")
        for path, error in scan["unreadable"]:
            lines.append(f"      {path}（{error}）")
        remedy(lines, ("给自己补上读权限再跑这一格。",))
    if lines:
        return 1, lines
    if not files and not scan["exempt"]:
        return 77, ["  ! 射程里（crates/、scripts/、task-server/、prune-server/、.claude/hooks/、.claude/gate.d/、.claude/scripts/ 下的 .rs、.wgsl、.py、.sh、"
                    "shebang 脚本与 README.md）一份文件都没有，这一格本次无对象可判（这不是通过）"]
    hits_total = marked_total + short_total + unmarked_total
    return 0, [
        f"  ✓ 注释里没有历史写法：扫了 {len(files)} 份文件（豁免表整份不判的 {len(scan['exempt'])} 份不算在内）的 {comment_lines} 行注释，"
        f"命中 {hits_total} 行：按标记放过 {marked_total} 行、滞后表放过 {lagged} 行、红 0 行（滞后表 {lag_rows} 行，{base_note}）",
        f"     没扫的：符号链接 {scan['symlinks']} 份、读不了 0 份、Python 文档串语法读不了没取 {len(scan['unparsed'])} 份"]


def write_atomically(path, text):
    directory = os.path.dirname(os.path.abspath(path))
    handle, temporary = tempfile.mkstemp(dir=directory, prefix=".comment-history-lag.")
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as stream:
            stream.write(text)
        os.chmod(temporary, 0o644)
        os.replace(temporary, path)
    except OSError:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


# ── 自证 ──

def build_tree(root, files):
    for path, content in files.items():
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(content)


def judge_tree(files, switches, jobs=1):
    with tempfile.TemporaryDirectory() as root:
        build_tree(root, files)
        code, lines = judge(root, switches, jobs)
    return code, "\n".join(lines)


def git_in(root, *arguments):
    return subprocess.run(["git", "-c", "user.email=t@example.com", "-c", "user.name=t", "-c", "commit.gpgsign=false", "-C", root, *arguments],
                          capture_output=True, text=True, check=True)


def judge_repository(base_files, later_files, switches):
    """临时 git 仓：基准提交放 base_files，再把 later_files 写进工作区（值是 None 的删掉），判。"""
    saved = os.environ.pop("GATE_BASE", None)
    try:
        with tempfile.TemporaryDirectory() as root:
            git_in(root, "init", "-q", ".")
            build_tree(root, base_files)
            git_in(root, "add", "-A")
            git_in(root, "commit", "-qm", "base")
            build_tree(root, {path: text for path, text in later_files.items() if text is not None})
            for path, text in later_files.items():
                if text is None:
                    os.unlink(os.path.join(root, path))
            code, lines = judge(root, switches, 1)
    finally:
        if saved is not None:
            os.environ["GATE_BASE"] = saved
    return code, "\n".join(lines)


def hit_lines_of(text, kind, switches):
    return [number for number, _word, _source in analyse(text, kind, switches)["hits"]]


def run_subprocess_judge(root, break_value):
    environment = dict(os.environ)
    environment[BREAK_VARIABLE] = break_value
    environment.pop("GATE_STAGED_FROM", None)
    completed = subprocess.run([sys.executable, os.path.realpath(__file__), root], capture_output=True, text=True, env=environment)
    return completed.returncode, completed.stdout + completed.stderr


def selftest_cells(switches):
    cells = []

    def cell(name, passed, message):
        cells.append((name, bool(passed), message))

    stock_table_header = "# 样本\n"
    lag_row = "{}\t{}\t# 样本存量，等这一批改完\n"

    # 射程：射程里各类文件都在、射程外带历史词的都不判，路径里的日期不判
    code, text = judge_tree({
        "crates/demo/src/lib.rs": "//! 只写现状，依据在 records/2026-10-29-sample.md。\n/// 说明。\nfn f() {}\n/* 块注释，只写现状。 */\n",
        "crates/demo/shaders/fill.wgsl": "// 填充着色器。\n",
        "scripts/demo/tool.py": "#!/usr/bin/env python3\n\"\"\"工具。\"\"\"\n# 只写现状\n",
        "scripts/demo/run.sh": "#!/usr/bin/env bash\n# 只写现状\n",
        "scripts/demo/launch": "#!/bin/sh\n# 只写现状\n",
        "scripts/demo/README.md": "说明\n\n```text\n此前 2026-10-26\n```\n",
        "scripts/demo/data": "# 此前 这份不是脚本\n",
        "scripts/demo/notes.txt": "此前\n",
        "docs/x.rs": "// 此前\n",
        "README.md": "此前\n",
        ".claude/gate.d/fixtures/a/b.rs": "// 此前\n",
        "crates/demo/target/gen.rs": "// 此前\n",
    }, switches)
    cell("scope", code == 0 and "扫了 6 份文件" in text and "命中 0 行：按标记放过 0 行、滞后表放过 0 行、红 0 行" in text,
         f"射程里的 6 份文件只写现状；docs/、根下 README.md、fixtures/、target/、.txt、没有 shebang 的无后缀文件都不判，实判 {code}：{text}")

    # 取文本：每一种形态
    cell("rust-line-date", hit_lines_of("// 这个常量 2026-10-26 改过\nconst A: u32 = 1;\n", "rust", switches) == [1], "Rust 行注释里的日期要命中第 1 行")
    cell("rust-doc-and-block", hit_lines_of("//! 此前只认 --bin\n/// 原先按行读\npub fn read() {}\n/* 以前\n   这样 */\n", "rust", switches) == [1, 2, 4],
         "Rust 的 //!、///、块注释各命中一处，行号 1、2、4")
    cell("python-docstring", hit_lines_of('"""模块。\n\n改前只认 --bin。\n"""\n\n\ndef f():\n    """曾经读文件。"""\n    return 1\n', "python", switches) == [3, 8],
         "Python 模块与函数文档串里的词各命中一处，行号 3、8")
    cell("python-hash", hit_lines_of("#!/usr/bin/env python3 曾经\nvalue = 1  # 此前读\n", "python", switches) == [2], "Python 的 # 注释命中，首行 shebang 里的词不算")
    cell("shell-hash", hit_lines_of("#!/usr/bin/env bash 曾经\n# 这一步用户 2026-10-26 定了\necho ok  # 此前读\nname=a#b\ncount=${#name}\n", "shell", switches) == [2, 3],
         "shell 的 # 注释命中第 2、3 行；a#b 与 ${#name} 里的 # 不是注释")
    cell("shell-heredoc", hit_lines_of("cat <<'TEXT'\n# 此前 正文里井号在词首\nTEXT\n# 原先 之后的注释\nx=$(cat <<EOF\n# 起初 命令替换里的 heredoc\nEOF\n)\n", "shell", switches) == [2, 4, 6],
         "heredoc 正文里词首的 # 照注释认（含命令替换里的）、结束标记之后恢复正常")
    cell("wgsl", hit_lines_of("// 旧写法\n@fragment\nfn main() {}\n", "wgsl", switches) == [1], "WGSL 的行注释命中第 1 行")
    cell("readme", hit_lines_of("第三轮判决里改了这一条。\n", "readme", switches) == [1], "README 全文判，第 1 行命中")
    cell("readme-fence", hit_lines_of("说明\n```text\n此前 2026-10-26\n```\n尾巴\n", "readme", switches) == [], "围栏代码块里的不判")
    fenced = analyse("说明\n```\n此前\n", "readme", switches)
    cell("readme-open-fence", fenced["open_fence"] == 2 and fenced["hits"] == [] or "readme-fences-judged" in switches, "围栏没收尾要报起头的行号 2")
    cell("strings-not-comments",
         hit_lines_of("let s = \"// 此前\";\nlet r = r#\"// 原先 \"x\"\"#;\nlet c = '\"';\nfn f<'a>(x: &'a str) {}\n/* 曾经 */\n", "rust", switches) == [5]
         and hit_lines_of("u = \"# 此前\"\nv = \"\"\"\n# 原先\n\"\"\"\nw = '# 曾经'  # 起初\n", "python", switches) == [5]
         and hit_lines_of("name=\"a # 此前\"\necho 'b # 原先' \"c # 曾经\"\ncount=${#name}\necho $#\n# 起初\n", "shell", switches) == [5],
         "字符串、原始字符串、字符字面量、三引号与引号里的井号都不是注释，三种语言都只命中末尾那一处真注释")
    cell("path-dates",
         hit_lines_of("// 见 records/2026-10-29-sample.md\n// 见 data-2026-10-26.json 与 notes_2026-10-26.txt\n// 见 2026-10-26.md\n// 这一天 2026-10-26。\n",
                      "rust", switches) == [4],
         "路径与文件名里的日期不算，句子里的日期算：只命中第 4 行")

    # 标记
    long_reason = "history-kept: 日期本身就是被测的数据"
    marked_rust = analyse(f"// 样本 2026-10-26  {long_reason}\n", "rust", switches)
    marked_docstring = analyse(f'"""模块。\n\n样本 2026-10-26  {long_reason}\n"""\n', "python", switches)
    marked_readme = analyse(f"样本 2026-10-26  {long_reason}\n", "readme", switches)
    cell("marker-long", marked_rust["hits"] == [] and marked_rust["marked"] == [1] and marked_docstring["hits"] == [] and marked_docstring["marked"] == [3]
         and marked_readme["hits"] == [] and marked_readme["marked"] == [1], "理由够长的标记放过那一行：Rust 注释、Python 文档串、README 同一行同样认")
    short_rust = analyse("// 样本 2026-10-26  history-kept: 太短了\n", "rust", switches)
    boundary_ok = analyse("// 样本 2026-10-26  history-kept: 一 二 三 四 五 六 七 八\n", "rust", switches)
    boundary_short = analyse("// 样本 2026-10-26  history-kept: 一 二 三 四 五 六 七\n", "rust", switches)
    cell("marker-short", len(short_rust["short"]) == 1 and short_rust["hits"] == [] and short_rust["marked"] == [] and boundary_ok["marked"] == [1]
         and len(boundary_short["short"]) == 1, "理由不到 8 个字（去空白后）的标记点名判红：8 个字放过、7 个字不放过")
    code, text = judge_tree({"crates/demo/src/lib.rs": "// 样本 2026-10-26  history-kept: 太短了\n"}, switches)
    cell("marker-short-report", code == 1 and "1 行的 history-kept 标记理由不到 8 个字" in text and "crates/demo/src/lib.rs:1  理由「太短了」只有 3 个字" in text,
         f"标记理由太短的整树判红并点名，实判 {code}：{text}")

    # 豁免表
    stock = {"scripts/w/words.py": "# 此前 原先 曾经：只讲这些词\n", "scripts/w/clean.sh": "# 只写现状\n"}
    exempt_row = "scripts/w/words.py\t# 样本里只讲这些词的词表文件\n"
    code, text = judge_tree({**stock, EXEMPT_TABLE: stock_table_header + exempt_row}, switches)
    cell("exempt-table", code == 0 and "豁免表整份不判的 1 份不算在内" in text, f"豁免表里的文件整份不判，实判 {code}：{text}")
    code, text = judge_tree({
        "scripts/w/words.py": "# 此前\n", "scripts/w/word-list.py": "# 此前\n", "scripts/w/three.py": "# 此前\n", "scripts/w/clean.sh": "# 只写现状\n",
        EXEMPT_TABLE: stock_table_header + exempt_row + exempt_row + "scripts/w/word-list.py\t# 词表\n"
        + "scripts/w/gone.py\t# 样本里整份不判的词表文件\n" + "scripts/w/three.py\ta\t# 样本里整份不判的词表文件\n"}, switches)
    cell("exempt-table-form", code == 1 and "豁免表 .claude/gate.d/comment-history-exempt.tsv 有 4 处写得不对" in text
         and "第 3 行：scripts/w/words.py 与第 2 行重复登记" in text and "第 4 行：第二列要以 # 起头" in text
         and "第 5 行：登记的路径不在判法射程里的文件中：scripts/w/gone.py" in text and "第 6 行：有 3 列，要两列" in text,
         f"豁免表写错的四处逐条点名，实判 {code}：{text}")

    # 滞后表
    three = "// 此前\n// 原先\n// 起初\n"
    code, text = judge_tree({"crates/c/a.rs": three, LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 3)}, switches)
    cell("lag-table", code == 0 and "滞后表放过 3 行" in text, f"命中数等于登记数放过，实判 {code}：{text}")
    code, text = judge_tree({"crates/c/a.rs": three, "crates/c/b.rs": "// 最初\n", LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 2)}, switches)
    cell("lag-count-over", code == 1 and "2 份文件的注释里写了历史，共 4 行" in text and "crates/c/a.rs:3  起初" in text and "crates/c/b.rs:1  最初" in text,
         f"命中比登记的多、与不在表里的各判红并列出全部命中，实判 {code}：{text}")
    code, text = judge_tree({"crates/c/a.rs": "// 此前\n// 原先\n", LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 3)}, switches)
    cell("lag-slack", code == 1 and "改小到 2" in text, f"命中比登记的少判红并写改小到 2，实判 {code}：{text}")
    code, text = judge_tree({"crates/c/a.rs": "// 只写现状\n", LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 3)}, switches)
    cell("lag-slack-clean", code == 1 and "改小到 0，即删掉这一行" in text or "lag-slack-allowed" in switches, f"改净了的文件登记要删，实判 {code}：{text}")
    code, text = judge_tree({
        "scripts/d/stock.sh": "# 此前\n# 原先\n", "scripts/d/short.sh": "# 曾经\n", "scripts/d/short2.sh": "# 起初\n", "scripts/d/clean.py": "# 只写现状\n",
        LAG_TABLE: stock_table_header + lag_row.format("scripts/d/stock.sh", 2) + lag_row.format("scripts/d/stock.sh", 2)
        + "scripts/d/short.sh\t1\n" + "scripts/d/clean.py\t二\t# 样本存量，等这一批改完\n" + "scripts/d/short2.sh\t1\t# 短\n"
        + lag_row.format("scripts/d/gone.sh", 1) + "scripts/d/clean.py\t0\t# 样本存量，等这一批改完\n"}, switches)
    cell("lag-table-form", code == 1 and "滞后表 .claude/gate.d/comment-history-lag.tsv 有 6 处写得不对" in text
         and "第 3 行：scripts/d/stock.sh 与第 2 行重复登记" in text and "第 4 行：有 2 列，要三列" in text and "第 5 行：命中行数要是正整数，写的是「二」" in text
         and "第 6 行：第三列要以 # 起头" in text and "第 7 行：登记的路径不在判法射程里的文件中：scripts/d/gone.sh" in text
         and "第 8 行：命中行数要是正整数，写的是「0」" in text, f"滞后表写错的六处逐条点名，实判 {code}：{text}")
    base_files = {"crates/c/a.rs": "// 此前\n// 原先\n", "crates/c/b.rs": "// 曾经\n", LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 2)}
    code, text = judge_repository(base_files, {LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 2) + lag_row.format("crates/c/b.rs", 1)}, switches)
    cell("lag-growth", code == 1 and "滞后表比 gate 基准 HEAD 那一版涨了 1 处" in text and "第 3 行：crates/c/b.rs  基准里没有这一行" in text,
         f"滞后表比基准多一行判红，实判 {code}：{text}")
    base_files = {"crates/c/a.rs": "// 此前\n// 原先\n", "crates/c/b.rs": "// 曾经\n",
                  LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 2) + lag_row.format("crates/c/b.rs", 1)}
    code, text = judge_repository(base_files, {"crates/c/b.rs": "// 只写现状\n", LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 2)}, switches)
    cell("lag-shrink", code == 0 and "与 gate 基准 HEAD 那一版比没涨（基准 2 行，此刻 1 行）" in text, f"只缩不涨：改净一份、删它的行判绿，实判 {code}：{text}")
    code, text = judge_repository({"crates/c/a.rs": "// 此前\n"}, {LAG_TABLE: stock_table_header + lag_row.format("crates/c/a.rs", 1)}, switches)
    cell("lag-first-registration", code == 0 and "里还没有滞后表（初次登记），只判写法、不判涨" in text, f"基准里没有滞后表只判写法，实判 {code}：{text}")

    # 现算滞后表、列表、数
    with tempfile.TemporaryDirectory() as root:
        build_tree(root, {"crates/c/a.rs": three, "scripts/c/b.sh": "# 曾经\n", "scripts/c/clean.py": "# 只写现状\n", LAG_TABLE: stock_table_header})
        scan = scan_root(root, switches, 1)
        rows = lag_table_text(scan, root)
        listing = list_lines(scan)
        build_tree(root, {LAG_TABLE: "\n".join(rows) + "\n"})
        code, lines = judge(root, switches, 1)
    cell("lag-round-trip", len(rows) == 3 and rows[0] == stock_table_header.strip() and rows[1] == f"crates/c/a.rs\t3\t{LAG_ROW_REASON}" and rows[2] == f"scripts/c/b.sh\t1\t{LAG_ROW_REASON}"
         and code == 0 and "滞后表放过 4 行" in "\n".join(lines), f"现算的滞后表照原样装上就绿：{rows}，{code}，{lines}")
    cell("list-format", listing == ["crates/c/a.rs:1:此前:// 此前", "crates/c/a.rs:2:原先:// 原先", "crates/c/a.rs:3:起初:// 起初", "scripts/c/b.sh:1:曾经:# 曾经"],
         f"--list 逐行打路径:行号:词:原文，实打 {listing}")
    code, text = judge_tree({"crates/c/a.rs": "// 只写现状\n/// 说明\nfn f() {}\n", "scripts/c/b.sh": "#!/bin/sh\n# 只写现状\n",
                             "scripts/c/README.md": "说明\n\n正文\n", "scripts/c/hit.py": "#!/usr/bin/env python3\n# 此前读\n",
                             LAG_TABLE: stock_table_header + lag_row.format("scripts/c/hit.py", 1)}, switches)
    cell("counts", code == 0 and "扫了 4 份文件（豁免表整份不判的 0 份不算在内）的 6 行注释，命中 1 行：按标记放过 0 行、滞后表放过 1 行、红 0 行"
         "（滞后表 1 行，被判的根不在 git 仓里，滞后表没与 gate 基准比）" in text, f"成功句的文件数、注释行数与命中数，实判 {code}：{text}")
    code, text = judge_tree({"docs/x.md": "此前\n"}, switches)
    cell("empty-scope", code == 77 and "无对象可判" in text, f"射程里一份文件都没有退 77，实判 {code}：{text}")
    cell("scope-shebang", [path for path, _kind in classify_tree({"scripts/a/tool": "#!/usr/bin/env python3\n# 曾经\n", "scripts/a/data": "# 曾经\n",
                                                                  "scripts/a/sh-tool": "#!/bin/sh\n# 此前\n", "scripts/a/text-tool": "#!/usr/bin/env -S bash -e\n# 此前\n"}, switches)]
         == ["scripts/a/sh-tool", "scripts/a/text-tool", "scripts/a/tool"], "没有后缀的文件首行是 sh、bash、python 的 shebang 才进射程，没有 shebang 的不进")
    shebang_scripts = classify_tree({"scripts/a/tool": "#!/usr/bin/env python3\n# 曾经\n"}, switches)
    cell("shebang-script", shebang_scripts == [("scripts/a/tool", "python")], f"没有后缀的 python shebang 脚本按 Python 取，实取 {shebang_scripts}")

    # 弄坏开关只在样本根与自证里认
    with tempfile.TemporaryDirectory() as bare_root, tempfile.TemporaryDirectory() as sample_root:
        build_tree(bare_root, {"crates/demo/src/lib.rs": "// 样本 2026-10-26  history-kept: 日期本身就是被测的数据\n"})
        build_tree(sample_root, {"crates/demo/src/lib.rs": "// 样本 2026-10-26  history-kept: 日期本身就是被测的数据\n", ".gate-cells": "comment-history\n"})
        refused_code, refused_text = run_subprocess_judge(bare_root, "marker-ignored")
        honored_code, honored_text = run_subprocess_judge(sample_root, "marker-ignored")
        unknown_code, unknown_text = run_subprocess_judge(sample_root, "no-such-switch")
    cell("break-refused", refused_code == 2 and "只在自证" in refused_text, f"没有标记的根上设弄坏开关要拒（退 2、点名只在自证里认），实判 {refused_code}：{refused_text}")
    cell("break-honored-in-sample", honored_code == 1 and "1 份文件的注释里写了历史" in honored_text, f"样本根上认弄坏开关，实判 {honored_code}：{honored_text}")
    cell("break-unknown", unknown_code == 2 and "认不出" in unknown_text, f"认不出的开关值退 2，实判 {unknown_code}：{unknown_text}")
    return cells


def classify_tree(files, switches):
    with tempfile.TemporaryDirectory() as root:
        build_tree(root, files)
        return list_candidates(root, switches)[0]


WHEEL = {
    "marker-ignored": "marker-long", "marker-reason-unchecked": "marker-short", "docstrings-skipped": "python-docstring",
    "path-dates-flagged": "path-dates", "strings-as-comments": "strings-not-comments", "lag-growth-unjudged": "lag-growth",
    "lag-slack-allowed": "lag-slack", "lag-table-form-unchecked": "lag-table-form", "exempt-table-form-unchecked": "exempt-table-form",
    "exempt-table-ignored": "exempt-table", "shebang-unscoped": "shebang-script", "readme-fences-judged": "readme-fence",
    "rust-comments-skipped": "rust-line-date", "wgsl-skipped": "wgsl", "python-comments-skipped": "python-hash",
    "shell-comments-skipped": "shell-hash", "readme-skipped": "readme",
}


def run_selftest(switches):
    cells = selftest_cells(switches)
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（射程、各形态取文本、字符串不误判、路径日期、标记、豁免表、滞后表写法与数、与基准比、现算滞后表、列表、数、开关只在样本根上认）")
    return 0


def switch_wheel():
    """逐个打开文件头登记的弄坏开关，各自要红在点名的格。"""
    assert set(WHEEL) == set(KNOWN_BREAKS)
    failures = 0
    for switch, expected in WHEEL.items():
        environment = dict(os.environ)
        environment[BREAK_VARIABLE] = switch
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True, env=environment)
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        if completed.returncode == 1 and expected in red_cells:
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，含点名的 {expected}")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的是 {expected}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(WHEEL)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(WHEEL)} 个开关逐个打开，各红在点名的格")
    return 0


# ── 命令行 ──

def environment_switches():
    """环境里的弄坏开关：(开关集合, 认不出的值)。"""
    given = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]
    return frozenset(item for item in given if item in KNOWN_BREAKS), [item for item in given if item not in KNOWN_BREAKS]


def root_argument(arguments):
    """命令行里的根：第一个不是选项、也不是 --out 的值的参数。"""
    skip = False
    for argument in arguments:
        if skip:
            skip = False
        elif argument == "--out":
            skip = True
        elif not argument.startswith("-"):
            return argument
    return None


def sample_root(root):
    real = os.path.realpath(root)
    marked = any(os.path.isfile(os.path.join(real, name)) for name in (".gate-cells", ".gate-break-sample"))
    return marked and real != REPOSITORY_ROOT and not os.environ.get("GATE_STAGED_FROM")


def settle_break_switches(arguments):
    """弄坏开关只在自证的开关轮（--selftest-inner）与样本根里认（参数解析之前先过这一道）：放行交回 None，拒绝交回退出码 2。"""
    if not os.environ.get(BREAK_VARIABLE):
        return None
    if arguments == ["--selftest-inner"]:
        return None
    root = root_argument(arguments)
    if root is not None and os.path.isdir(root) and sample_root(root):
        return None
    print(f"  ✗ {BREAK_VARIABLE} 是弄坏开关，只在自证的开关轮（--selftest 起的 --selftest-inner）与样本根（根里有 .gate-cells 或 .gate-break-sample，根不是这份脚本所在的仓）里认；平常跑设了就拒，这一次什么都没做")
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一项，跑 python3 scripts/gate/comment-history-check.py --selftest（开关轮逐个打开）")
    return 2


def usage_error(arguments):
    print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
    print("     → 怎么办：用法 comment-history-check.py [根] | --list <根> | --lag-table <根> [--out <文件>] | --selftest")
    return 2


def main(arguments):
    switches, unknown = environment_switches()
    if unknown:
        print(f"  ✗ {BREAK_VARIABLE} 认不出 {'、'.join(unknown)}（只认 {'、'.join(KNOWN_BREAKS)}）")
        print(f"     → 怎么办：unset {BREAK_VARIABLE}，或只写文件头开关表里的值")
        return 2
    if arguments == ["--selftest"]:
        status = run_selftest(frozenset())
        return status or switch_wheel()
    if arguments == ["--selftest-inner"]:
        return run_selftest(switches)
    rest, mode, output = list(arguments), "judge", None
    if rest[:1] == ["--list"]:
        mode, rest = "list", rest[1:]
    elif rest[:1] == ["--lag-table"]:
        mode, rest = "lag-table", rest[1:]
        if "--out" in rest:
            index = rest.index("--out")
            if index + 1 >= len(rest):
                return usage_error(arguments)
            output = rest[index + 1]
            rest = rest[:index] + rest[index + 2:]
    if len(rest) > 1 or any(argument.startswith("-") for argument in rest) or (mode != "judge" and not rest):
        return usage_error(arguments)
    root = os.path.realpath(rest[0]) if rest else REPOSITORY_ROOT
    if not os.path.isdir(root):
        print(f"  ✗ 根不在：{root}")
        print("     → 怎么办：给仓根，或不给参数判这份脚本所在的仓")
        return 2
    try:
        jobs = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return 2
    if mode == "judge":
        code, lines = judge(root, switches, jobs)
        print("\n".join(lines))
        return code
    scan = scan_root(root, switches, jobs)
    if not scan["files"] and not scan["exempt"]:
        print("  ! 射程里一份文件都没有，本次无对象可判（这不是通过）", file=sys.stderr)
        return 77
    if mode == "list":
        listing = list_lines(scan)
        print("\n".join(listing))
        print(f"  ✓ 逐行列出：{len(listing)} 行命中，在 {len({line.split(':', 1)[0] for line in listing})} 份文件里（扫了 {len(scan['files'])} 份，豁免表整份不判 {len(scan['exempt'])} 份）", file=sys.stderr)
        return 0
    text = "\n".join(lag_table_text(scan, root)) + "\n"
    rows = sum(1 for info in scan["files"].values() if info["hits"])
    if output is None:
        print(text, end="")
    else:
        write_atomically(output, text)
    print(f"  ✓ 现算滞后表：{rows} 份文件有命中（命中共 {sum(len(info['hits']) for info in scan['files'].values())} 行）{'，写到 ' + output if output else ''}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
