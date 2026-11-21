"""check.sh 里 CODE_DISCIPLINE_LINTS=( … ) 那张 clippy lint 表的读法：项目代码闸 project-code-guard.sh 的 ⑤（cargo clippy 的 `--` 之后要带全这张表、不许再放行表里的 lint）与
检出钩子 bash-command-detector.sh 的 ㉓（出路里的 cargo clippy 写法带整段）共用这一份，两边读到的表逐条相同，也与整轮门禁真传给 clippy 的那张数组相同。

只认一种写法（严格读，别的写法一律判读不成，不猜 bash 会把它展开成什么）：
  CODE_DISCIPLINE_LINTS=(                       开头那一行（前面可带 readonly、declare -a、local、typeset、export 这类声明），`(` 之后只许空白或 # 注释
    -D clippy::<名>   # 说明                   每行恰好一条：`-D`、空白、`clippy::<小写字母起头的名字，字母数字下划线>`，行尾可带 # 注释；空行与整行注释行也许
  )                                             收尾的 `)` 独占一行（可缩进、可带 # 注释）
读不成的写法：开头那一行写着元素；一行写几条、`-D` 与名字贴着写、`--deny`、`-F`、不是 clippy:: 起头的 lint、引号里的词、花括号、数组元素（`"${…[@]}"`）、行末 `\\` 接下一行、
没有收尾的 `)`；文件里另有写这个数组的地方（`CODE_DISCIPLINE_LINTS+=(`、再次 `CODE_DISCIPLINE_LINTS=(`、`CODE_DISCIPLINE_LINTS[n]=`、`unset`、`read -a` 这类；
`"${CODE_DISCIPLINE_LINTS[@]}"` 这样读它的不算）。这些写法 bash 都读得懂、整轮门禁照样传给 clippy，而这里读到的与门禁真传的会对不上，所以只认上面那一种，读不成的由调用方拒或记检出。

用法：python 里把这份所在的目录放进 sys.path（或按文件路径导入）再用；不单独起（单独起一律退 2；环境里设着弄坏开关的，点名它只在自证里认）。
  table_state(check_script)      ("ok", 表里的词) / ("absent", 原因：读不了文件、没有 CODE_DISCIPLINE_LINTS=( 那一段) / ("unreadable", 原因：有这一段而写法读不成，点名行号）
  table_words(check_script)      表里的词（列表：每条是 `-D`、`clippy::<名>` 两个词）；不是 "ok" 交 None
  table_problem(check_script)    表有而读不成时的原因（一句）；读得成、没有这一段、读不了文件交 None
  table_names(check_script)      表里被点名的名字（clippy::<名>），按出现次序、去重；不是 "ok" 交 None
  table_tail(check_script)       `-- -D warnings <表里的词…>` 整段；不是 "ok" 交空串
  denied_names(arguments)        clippy 命令 `--` 之后的参数里被 -D / --deny 点名的名字（-D 名字、-D名字、--deny 名字、--deny=名字 四种写法）
  weakened_names(arguments, names)
                                 `--` 之后的参数里，在它盖住的名字最后一次 deny 之后又被 -A / -W / --allow / --warn 放行或降级的词（点名的是 names 里的名字、`warnings` 或 clippy 的 lint 组名）
  caps_lints(arguments)          `--` 之后的参数里带 `--cap-lints`（它把所有 lint 压到指定级别，deny 不再起作用）

弄坏开关（LIB_CODE_DISCIPLINE_LINTS_BREAK，只给导入它的钩子的自证判红用；两道钩子平常跑设了它一律退 2、点名它只在自证里认）。这一份没有自己的 --selftest，由导入它的检出钩子的自证判红；
每条开关在这里只留「名字、弄坏哪一处」，打开后哪几格红只登记一处：bash-command-detector.sh 文件头同名开关下（门禁 code-tooling 的 break-switch-replay 格逐条设上开关、跑那一份的 --selftest）。
  LIB_CODE_DISCIPLINE_LINTS_BREAK=entry-form-unchecked  表里每一行照 shell 切词、切出来的词都收（不查 `-D clippy::<名>` 一行一条，开头那一行 `(` 之后的元素也收）
      红格见 bash-command-detector.sh 文件头同名开关下
  LIB_CODE_DISCIPLINE_LINTS_BREAK=other-assignments-unscanned  不查文件里另有没有写这个数组的地方（`+=`、再次赋值、元素赋值照旧不看），只读第一处开头
      红格见 bash-command-detector.sh 文件头同名开关下
"""
import os
import re
import shlex
import sys

CHECK_SCRIPT_RELATIVE = os.path.join(".claude", "singlefs-ai-sop", "scripts", "check.sh")
TABLE_NAME = "CODE_DISCIPLINE_LINTS"
TABLE_START = re.compile(r"^\s*(?:(?:readonly|declare|typeset|local|export)(?:\s+-[A-Za-z-]+)*\s+)*CODE_DISCIPLINE_LINTS=\((.*)$")
TABLE_ENTRY = re.compile(r"^\s*-D\s+(clippy::[a-z][a-z0-9_]*)(?:\s+#.*)?\s*$")
TABLE_BLANK = re.compile(r"^\s*(?:#.*)?$")
TABLE_END = re.compile(r"^\s*\)\s*(?:#.*)?$")
TABLE_EXPANSION = re.compile(r"\$\{#?CODE_DISCIPLINE_LINTS\[[@*]\]\}")
DENY_WARNINGS = "warnings"
LINT_GROUP_NAMES = frozenset(("warnings", "clippy::all", "clippy::correctness", "clippy::suspicious", "clippy::style", "clippy::complexity", "clippy::perf",
                              "clippy::pedantic", "clippy::restriction", "clippy::nursery", "clippy::cargo"))   # 放行或降级一整组，也盖住表里的 lint
DENY_OPTIONS = ("-D", "--deny")
WEAKEN_OPTIONS = ("-A", "--allow", "-W", "--warn")
SHOWN_LINE_CHARACTERS = 80


def break_item(item):
    return item in os.environ.get("LIB_CODE_DISCIPLINE_LINTS_BREAK", "").split(",")


def lenient_words(text):
    """弄坏开关 entry-form-unchecked 下的读法：一行按 shell 的规矩切开（注释去掉）；引号没收尾、行末 `\\` 交 None。"""
    try:
        return shlex.split(text, comments=True)
    except ValueError:
        return None


def table_state(check_script):
    try:
        with open(check_script, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except OSError as error:
        return "absent", f"读不了 {check_script}（{error.__class__.__name__}）"
    starts = [number for number, line in enumerate(lines) if TABLE_START.match(line)]
    if not starts:
        return "absent", f"{check_script} 里没有 {TABLE_NAME}=( 那一段"
    if not break_item("other-assignments-unscanned"):
        if len(starts) > 1:
            return "unreadable", f"第 {starts[1] + 1} 行又写了一次 {TABLE_NAME}=(（重新赋值，门禁传给 clippy 的是最后一次的）"
        for number, line in enumerate(lines):
            if number in starts or line.lstrip().startswith("#"):
                continue
            if TABLE_NAME in TABLE_EXPANSION.sub("", line):
                return "unreadable", f"第 {number + 1} 行另有写 {TABLE_NAME} 的地方（追加、元素赋值、unset 这类）：{line.strip()[:SHOWN_LINE_CHARACTERS]}"
    start = starts[0]
    remainder = TABLE_START.match(lines[start]).group(1)
    words = []
    if break_item("entry-form-unchecked"):
        found = lenient_words(remainder)
        if found is None:
            return "unreadable", f"第 {start + 1} 行切不开"
        words.extend(found)
    elif not TABLE_BLANK.match(remainder):
        return "unreadable", f"第 {start + 1} 行开头 `(` 之后写着元素（表里每行恰好一条 -D clippy::<名>）"
    for number in range(start + 1, len(lines)):
        line = lines[number]
        if TABLE_END.match(line):
            if not words:
                return "unreadable", f"第 {start + 1} 行起的表里一条都没有"
            return "ok", words
        if break_item("entry-form-unchecked"):
            found = lenient_words(line)
            if found is None:
                return "unreadable", f"第 {number + 1} 行切不开：{line.strip()[:SHOWN_LINE_CHARACTERS]}"
            words.extend(found)
            continue
        if TABLE_BLANK.match(line):
            continue
        entry = TABLE_ENTRY.match(line)
        if entry is None:
            return "unreadable", f"第 {number + 1} 行不是 `-D clippy::<名>` 一行一条的写法：{line.strip()[:SHOWN_LINE_CHARACTERS]}"
        words.extend(("-D", entry.group(1)))
    return "unreadable", f"第 {start + 1} 行起的表没有收尾的 `)`（收尾的 `)` 要独占一行）"


def table_words(check_script):
    state, value = table_state(check_script)
    return value if state == "ok" else None


def table_problem(check_script):
    state, value = table_state(check_script)
    return value if state == "unreadable" else None


def denied_names(arguments):
    """clippy 命令 `--` 之后的参数里被 -D / --deny 点名的那几个名字（-D 名字、-D名字、--deny 名字、--deny=名字 四种写法）。"""
    denied, position = set(), 0
    while position < len(arguments):
        word = arguments[position]
        if word in ("-D", "--deny") and position + 1 < len(arguments):
            denied.add(arguments[position + 1])
            position += 2
            continue
        if word.startswith("--deny="):
            denied.add(word[len("--deny="):])
        elif word.startswith("-D") and len(word) > 2:
            denied.add(word[2:])
        position += 1
    return denied


def level_named(arguments, options):
    """arguments 里用 options 里的选项点名的 (位置, 名字)（-A 名字、-A名字、--allow 名字、--allow=名字 四种写法，-W、--warn 同）。"""
    found, position = [], 0
    while position < len(arguments):
        word = arguments[position]
        if word in options and position + 1 < len(arguments):
            found.append((position, arguments[position + 1]))
            position += 2
            continue
        for option in options:
            if option.startswith("--") and word.startswith(option + "="):
                found.append((position, word[len(option) + 1:]))
            elif not option.startswith("--") and word.startswith(option) and len(word) > len(option):
                found.append((position, word[len(option):]))
        position += 1
    return found


def weakened_names(arguments, names):
    """`--` 之后的参数里被放行或降级（-A、--allow、-W、--warn）的词（按写的次序，去重）：点名的是 names 里的名字、`warnings` 或 clippy 的组名（整组放行也盖住表里的 lint），
    而且点名的那一处在它盖住的某个名字最后一次 deny 之后（先放行、后 deny 的，deny 照样起作用，不算）。"""
    denies = level_named(arguments, DENY_OPTIONS)
    targets = list(names) + [DENY_WARNINGS]
    weakened = []
    for position, named in level_named(arguments, WEAKEN_OPTIONS):
        covered = [target for target in targets if named == target or named in LINT_GROUP_NAMES]
        if any(max((at for at, denied in denies if denied == target), default=-1) < position for target in covered) and named not in weakened:
            weakened.append(named)
    return weakened


def caps_lints(arguments):
    return any(word == "--cap-lints" or word.startswith("--cap-lints=") for word in arguments)


def table_names(check_script):
    words = table_words(check_script)
    if words is None:
        return None
    names = []
    for position, word in enumerate(words):
        window = words[position:position + 2] if word in ("-D", "--deny") else [word]
        for name in sorted(denied_names(window)):
            if name not in names:
                names.append(name)
    return names or None


def table_tail(check_script):
    words = table_words(check_script)
    return "" if words is None else shlex.join(["--", "-D", DENY_WARNINGS, *words])


def main():
    """共用库没有命令行入口：单独起一律退 2；设着弄坏开关的，点名它只在自证里认（只有导入这一份的钩子的 --selftest 认它）。"""
    value = os.environ.get("LIB_CODE_DISCIPLINE_LINTS_BREAK", "")
    if value:
        print(f"✗ lib_code_discipline_lints.py：LIB_CODE_DISCIPLINE_LINTS_BREAK={value} 是弄坏开关，只在自证（导入这一份的 bash-command-detector.sh、project-code-guard.sh 的 --selftest）里认；平常跑设了就拒", file=sys.stderr)
        print("→ 怎么办：unset LIB_CODE_DISCIPLINE_LINTS_BREAK 再来；要看开关红在哪几格，跑 LIB_CODE_DISCIPLINE_LINTS_BREAK=<项> bash .claude/hooks/bash-command-detector.sh --selftest", file=sys.stderr)
        return 2
    print("✗ lib_code_discipline_lints.py 是共用库，由钩子 import，不单独起", file=sys.stderr)
    print("→ 怎么办：钩子里 import 它（文件头「用法」）；核认法跑导入它的钩子的 --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.exit(main())
