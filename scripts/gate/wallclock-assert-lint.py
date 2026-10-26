#!/usr/bin/env python3
# admission: always 判的是此刻 scripts/experiment-apparatus/*.rs 里的 #[test]，几秒跑完；改了测试或登记位就要现判
# run-condition: none 只读 .rs 源码，纯 python3 标准库，不起 cargo
"""按挂钟判的断言要有登记位（checks-owed C465）。登记位只是一行注释，装置源码里不加任何代码。

用法：
    wallclock-assert-lint.py [仓根]          # 不给就判本仓；扫 <仓根>/scripts/experiment-apparatus/*.rs 里每个 #[test] 函数
    wallclock-assert-lint.py --list [仓根]   # 打登记了的测试，一行「<文件>TAB<函数名>」；门禁的单测格靠它按名给 cargo test 传 --skip
    wallclock-assert-lint.py --selftest      # WALLCLOCK_ASSERT_LINT_BREAK=<开关> 时必须红在点名的格

什么算按挂钟判的断言：#[test] 函数里的 assert! / assert_eq! / assert_ne!（含 debug_ 前缀），它的参数（连同格式串里 {名字} 的内联捕获）
  先过读钟这一关：测试函数里、或它（传递地）调到的同一份文件里的函数里，有 CLOCK_READ_PATTERNS 的写法（Instant::now、SystemTime::now、.elapsed(、.duration_since(），
  否则它量不到挂钟，断言不算。过了这一关，断言的参数（连同格式串里 {名字} 的内联捕获）
  ① 直接含挂钟量：MEASUREMENT_PATTERNS 的写法（.elapsed(、.duration_since(），或词表 WALLCLOCK_VOCABULARY 里的词（elapsed、Instant、Duration、per_second、throughput、nanoseconds，可扩）
     出现在一个值标识符的词序列里（four_thread_nanoseconds 算；函数名、Duration::from_secs 这类路径头、全大写的常量、`let 名字 = <只有字面量与运算>;` 绑的名字都不算，它们不是量到的）；
  ② 或含一个被「污染」的变量：一条语句（连同包着它的 if / for / while / match 头）含 MEASUREMENT_PATTERNS 的写法、含挂钟词的值标识符、
     或调了「读钟的同文件函数」而被调的函数名或语句里的值标识符含挂钟词，那条语句里出现的值标识符都被污染，污染沿语句往下传到不动点。
  判的是字面与传递，不是数据流：字面上抓不到的挂钟断言（跨文件的计时辅助函数、名字里没有挂钟词的量），登记位是人的判断，这条判法兜不住。

登记位（两种，都是 // 起头的普通注释行，只认这两个位置：#[test] 属性与 fn 之间，或 #[test] 上方紧挨着的一行）：
  // wallclock-assert: <量的是什么，至少 8 个字>   断言量挂钟，负载高会误红。门禁的单测格按 --list 列出的函数名给 cargo test 传 --skip，登记了的测试本次不跑、逐条报本次未跑。
  // wallclock-assert-load-safe: <为什么负载再高也不会红>   断言里有挂钟词，但判的方向不怕负载（量到的只会更大才过的下界，或比的是配置值、字面量算出来的数）。门禁里照跑，不跳过。

--skip 是子串匹配：登记的函数名是别的 #[test] 函数名的子串（含同名）时，--skip 会把那一条一起跳过。判法把扫到的全部 #[test] 函数名当全集，
  登记名撞上别的、没登记的测试名就判红，点名撞上的那一条；出路是改名，或者把撞上的那一条也登记（它也得是有挂钟断言的测试）。全集只含 scripts/experiment-apparatus/ 顶层的 .rs，子目录里的不扫。

判红：
  ① 有挂钟断言而两种登记位都没有（点名文件:行号）；两种都有
  ② 登记位挂在没有挂钟断言的函数上（写错地方）；理由不到 8 个字（wallclock-assert:）、不到 4 个字（load-safe）
  ③ 登记位的注释行下面不是 #[test] 函数（落空），或是 #[test] 函数而不在上面说的两个位置
  ④ 登记名是别的未登记 #[test] 函数名的子串

滞后表 .claude/gate.d/wallclock-assert-lag.tsv（本判法立之前就在仓里、还没人复核的挂钟断言；只缩不涨，门禁不判涨，靠 review）：
  一行一个测试，`scripts/experiment-apparatus/<文件>.rs::<函数名>  # 理由`（理由至少 8 个字，写它量的是什么、为什么还没登记）。落在表里的没登记的挂钟断言不判红，
  成功行另报「另有 N 条在滞后表里」；表里的行对应的测试已经登记、不再有挂钟断言或函数不在了，判红（陈旧行，删掉）。新写的挂钟断言一律直接写登记位，不往表里加。

退出码：0 全对；1 有红（逐条点名文件:行号）；2 用法错；77 scripts/experiment-apparatus/ 下一份 .rs 都没有、或一个 #[test] 都没扫到。

弄坏开关 WALLCLOCK_ASSERT_LINT_BREAK=<项>（逗号连几个；只在 --selftest 里认，别处设了只打一句提醒）：
  registration-unchecked 有挂钟断言没登记位的也判绿，unregistered 与 propagated-unregistered 两格该红
  misplaced-unchecked    登记位挂在没有挂钟断言的函数上、或下面不是 #[test] 的也判绿，misplaced 与 orphan 与 load-safe-misplaced 三格该红
  placement-unchecked    登记位不在 #[test] 与 fn 之间、也不是 #[test] 上方紧挨的一行，也判绿，marker-too-far 一格该红
  reason-unchecked       理由太短也判绿，reason-short 一格该红
  collision-unchecked    登记名是别的测试名的子串也判绿，name-collision 一格该红
  propagation-off        只看断言参数字面上的挂钟量、不传污染：propagated-unregistered 一格该红
  lag-unchecked          滞后表里已经登记了或不再是挂钟断言的陈旧行不报：lag-stale 一格该红

查过的已有判据：code-source-discipline.sh 的实验源码纪律只判有没有钉绝对值的断言、种子与读数，不看断言量的是不是挂钟；
  checker-tier-research-build-and-replay.sh 的 research-unit-tests 格是用它的 --list 按名传 --skip 的一方，两边读同一份登记。
"""
import os
import re
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

EXPERIMENT_DIRECTORY = os.path.join("scripts", "experiment-apparatus")
# 词表：标识符按下划线与驼峰拆成小写词，连续命中这些词序列就算挂钟量。要扩就往这里加。
WALLCLOCK_VOCABULARY = ("elapsed", "Instant", "Duration", "per_second", "throughput", "nanoseconds")
# 取时刻的写法：出现在语句里，那条语句就是挂钟量的来源。
# 读钟的写法：函数里有这些，或调了有这些的函数（同一份文件里），才可能量到挂钟；测试够不到任何一处读钟，它的断言不算挂钟断言。
CLOCK_READ_PATTERNS = (r"\bInstant::now\b", r"\bSystemTime::now\b", r"\.elapsed\s*\(", r"\.duration_since\s*\(")
# 量出一段时间的写法：出现在语句或断言参数里，那条语句就是挂钟量的来源（Instant::now() 本身只是造一个时刻，不算量到）。
MEASUREMENT_PATTERNS = (r"\.elapsed\s*\(", r"\.duration_since\s*\(")
REGISTRATION = re.compile(r"^\s*//\s*wallclock-assert:\s*(.*?)\s*$")
LOAD_SAFE = re.compile(r"^\s*//\s*wallclock-assert-load-safe:\s*(.*?)\s*$")
ANY_MARKER = re.compile(r"^\s*//\s*wallclock-assert(?:-load-safe)?:")
MINIMUM_REASON_CHARACTERS = 8
MINIMUM_LOAD_SAFE_REASON_CHARACTERS = 4
LAG_TABLE = os.path.join(".claude", "gate.d", "wallclock-assert-lag.tsv")
LAG_REASON_CHARACTERS = 8
ASSERT_MACRO = re.compile(r"\b(?:debug_)?assert(?:_eq|_ne)?!\s*\(")
IDENTIFIER = re.compile(r"(?<![\w.])[A-Za-z_]\w*|(?<=\.)[A-Za-z_]\w*")
INLINE_CAPTURE = re.compile(r"\{([A-Za-z_]\w*)(?::[^}]*)?\}")
NOT_VALUES = frozenset("""as let mut ref if else for while loop match in return break continue move fn pub use mod struct enum impl where
    self Self super crate true false u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64 bool char str String Vec Option Some None Ok Err""".split())
BREAKS = ("registration-unchecked", "misplaced-unchecked", "placement-unchecked", "reason-unchecked", "collision-unchecked", "propagation-off", "lag-unchecked")
broken = set()

def sanitize(text):
    """交回等长的两份：只抹注释的、连字符串内容也抹的（引号留着）。换行都留着，偏移就是行位置。"""
    keep_strings, drop_strings = list(text), list(text)
    length, index = len(text), 0

    def blank(buffer, start, stop):
        for position in range(start, stop):
            if buffer[position] != "\n":
                buffer[position] = " "

    while index < length:
        pair = text[index:index + 2]
        if pair == "//":
            stop = text.find("\n", index)
            stop = length if stop < 0 else stop
            blank(keep_strings, index, stop), blank(drop_strings, index, stop)
            index = stop
        elif pair == "/*":
            depth, position = 1, index + 2
            while position < length and depth:
                if text[position:position + 2] == "/*":
                    depth, position = depth + 1, position + 2
                elif text[position:position + 2] == "*/":
                    depth, position = depth - 1, position + 2
                else:
                    position += 1
            blank(keep_strings, index, position), blank(drop_strings, index, position)
            index = position
        elif text[index] == '"' or (text[index] in "rb" and re.match(r'(?:br|rb|r|b)#*"', text[index:index + 12]) and (index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_"))):
            raw = re.match(r'(?:br|rb|r)(#*)"', text[index:index + 12])
            if raw:
                opener = raw.end()
                closer = '"' + raw.group(1)
                stop = text.find(closer, index + opener)
                stop = length if stop < 0 else stop + len(closer)
                blank(drop_strings, index + opener, max(index + opener, stop - len(closer)))
            else:
                position = index + (2 if text[index] == "b" else 1)
                while position < length and text[position] != '"':
                    position += 2 if text[position] == "\\" else 1
                stop = min(position + 1, length)
                blank(drop_strings, index + (2 if text[index] == "b" else 1), max(index + 1, stop - 1))
            index = stop
        elif text[index] == "'":
            if text[index + 1:index + 2] == "\\":
                stop = text.find("'", index + 3)
                stop = index + 1 if stop < 0 else stop + 1
                blank(drop_strings, index + 1, stop - 1)
                index = stop
            elif text[index + 2:index + 3] == "'":
                blank(drop_strings, index + 1, index + 2)
                index += 3
            else:
                index += 1
        else:
            index += 1
    return "".join(keep_strings), "".join(drop_strings)


def matching(text, start, opener, closer):
    """text[start] 是 opener，交回配对的 closer 的位置；配不上交回 -1。"""
    depth = 0
    for position in range(start, len(text)):
        if text[position] == opener:
            depth += 1
        elif text[position] == closer:
            depth -= 1
            if depth == 0:
                return position
    return -1


def words_of(identifier):
    spaced = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", identifier)
    return [word for word in spaced.lower().split("_") if word]


VOCABULARY_WORDS = tuple(tuple(words_of(entry)) for entry in WALLCLOCK_VOCABULARY)


def has_vocabulary(identifier):
    words = words_of(identifier)
    for entry in VOCABULARY_WORDS:
        width = len(entry)
        if any(tuple(words[start:start + width]) == entry for start in range(len(words) - width + 1)):
            return True
    return False


def value_identifiers(text):
    """text 里作为「值」出现的标识符：后面不跟 (、不跟 ::（函数名、路径头不是量到的值），也不是关键字与基本类型。"""
    found = []
    for match in IDENTIFIER.finditer(text):
        name = match.group(0)
        if name in NOT_VALUES:
            continue
        rest = text[match.end():match.end() + 2].lstrip()
        if rest.startswith("(") or rest.startswith("::") or re.match(r"\s*!(?!=)", text[match.end():]):
            continue
        if re.fullmatch(r"[A-Z][A-Z0-9_]*", name):   # 全大写是常量，不是量到的值
            continue
        found.append(name)
    return found


def reads_clock(text):
    return any(re.search(pattern, text) for pattern in CLOCK_READ_PATTERNS)


def has_measurement(text):
    return any(re.search(pattern, text) for pattern in MEASUREMENT_PATTERNS)


def callee_names(text):
    return set(re.findall(r"(?<![\w.])([A-Za-z_]\w*)\s*\(", text))


def statements_of(body):
    """函数体切成语句：在圆括号与方括号之外按 ; 与花括号切；每条语句带上包着它的头（if / for / while / match 的条件、let 的左边）。
    交回 [(语句文本, 头文本)]。"""
    statements, stack, buffer, depth = [], [], [], 0
    for character in body:
        if character in "([":
            depth += 1
        elif character in ")]":
            depth -= 1
        if depth == 0 and character == ";":
            statements.append(("".join(buffer), " ".join(stack)))
            buffer = []
        elif depth == 0 and character == "{":
            stack.append("".join(buffer))
            buffer = []
        elif depth == 0 and character == "}":
            if "".join(buffer).strip():
                statements.append(("".join(buffer), " ".join(stack)))
            buffer = []
            if stack:
                stack.pop()
        else:
            buffer.append(character)
    if "".join(buffer).strip():
        statements.append(("".join(buffer), " ".join(stack)))
    return statements


def is_constant_binding(statement):
    """let 名字 = <只有字面量与运算>：右边没有任何值标识符、没有函数调用。"""
    match = re.match(r"\s*let\b[^=]*?=(?!=)(.*)\Z", statement, re.S)
    if not match or re.match(r"\s*let\s+mut\b", statement):   # let mut 的名字后面还会被改，不当常量
        return False
    right = match.group(1)
    return not [name for name in IDENTIFIER.findall(right) if name not in NOT_VALUES] and "(" not in re.sub(r"\([\d_\s+\-*/%<>&|^.xa-fA-F]*\)", "", right)


def constant_names(body_without_strings):
    """函数里由字面量常量绑定的名字（let 名字 = 只有字面量与运算）：名字里有挂钟词也不是量到的。"""
    found = set()
    for statement, _ in statements_of(body_without_strings):
        if is_constant_binding(statement):
            found |= set(value_identifiers(statement.split("=", 1)[0]))
    return found


def tainted_names(body_without_strings, clock_reaching):
    """一步传递的污染集合，见文件头 ②。"""
    statements = statements_of(body_without_strings)
    constants = constant_names(body_without_strings)
    infos = []
    for statement, header in statements:
        if ASSERT_MACRO.search(statement.lstrip()[:20]):
            continue
        names = set(value_identifiers(statement + " " + header)) - constants
        calls = callee_names(statement) & clock_reaching
        seed = (has_measurement(statement) or has_measurement(header) or any(has_vocabulary(name) for name in names)
                or bool(calls and any(has_vocabulary(name) for name in calls)))
        if is_constant_binding(statement):
            seed, names = False, set()
        infos.append((names, seed))
    tainted = set()
    changed = True
    while changed:
        changed = False
        for names, seed in infos:
            if (seed or names & tainted) and not names <= tainted:
                tainted |= names
                changed = True
    return tainted


def wallclock_assertions(function, clock_reaching):
    """函数体里按挂钟判的断言：[(行号, 缘由)]。"""
    keep_body, drop_body, base = function["body_keep"], function["body_drop"], function["body_offset"]
    if not (reads_clock(drop_body) or callee_names(drop_body) & clock_reaching):
        return []
    tainted = set() if "propagation-off" in broken else tainted_names(drop_body, clock_reaching)
    constants = constant_names(drop_body)
    found = []
    for match in ASSERT_MACRO.finditer(drop_body):
        opener = match.end() - 1
        closer = matching(drop_body, opener, "(", ")")
        if closer < 0:
            continue
        arguments_drop, arguments_keep = drop_body[opener + 1:closer], keep_body[opener + 1:closer]
        line = function["text"].count("\n", 0, base + match.start()) + 1
        values = [name for name in value_identifiers(arguments_drop) + INLINE_CAPTURE.findall(arguments_keep) if name not in constants]
        reason = None
        if has_measurement(arguments_drop):
            reason = "断言参数里直接量了时间"
        else:
            direct = [name for name in values if has_vocabulary(name)]
            if direct:
                reason = f"断言参数里有挂钟量 {direct[0]}"
            else:
                carried = [name for name in values if name in tainted]
                if carried:
                    reason = f"断言参数里的 {carried[0]} 由挂钟量传来"
        if reason:
            found.append((line, reason))
    return found


def functions_of(path, text):
    keep, drop = sanitize(text)
    lines = text.split("\n")
    found = []
    for match in re.finditer(r"\bfn\s+([A-Za-z_]\w*)", drop):
        name = match.group(1)
        position, depth, start = match.end(), 0, -1
        while position < len(drop):
            character = drop[position]
            if character in "([":
                depth += 1
            elif character in ")]":
                depth -= 1
            elif depth == 0 and character == ";":
                break
            elif depth == 0 and character == "{":
                start = position
                break
            position += 1
        if start < 0:
            continue
        stop = matching(drop, start, "{", "}")
        if stop < 0:
            continue
        line_number = text.count("\n", 0, match.start()) + 1
        # 上方与 #[test] 属性、文档注释连成一块的行
        attribute_lines, above = [], line_number - 2
        before_fn_on_line = lines[line_number - 1][:match.start() - (text.rfind("\n", 0, match.start()) + 1)]
        while above >= 0 and re.match(r"^\s*(#\[|#!\[|//)", lines[above]):
            attribute_lines.append((above + 1, lines[above]))
            above -= 1
        test_attribute_lines = [number for number, line in attribute_lines if re.match(r"^\s*#\[test\]\s*$", line)]
        is_test = "#[test]" in before_fn_on_line or bool(test_attribute_lines)
        # 登记位只认两个位置：#[test] 属性与 fn 之间，或 #[test] 上方紧挨着的一行（#[test] 与 fn 写在同一行时，只认 fn 上方紧挨着的一行）
        test_line = max(test_attribute_lines) if test_attribute_lines else line_number
        attached_numbers = {number for number, _ in attribute_lines}
        proper_lines = {number for number in attached_numbers if test_line < number < line_number}
        if test_line - 1 in attached_numbers:
            proper_lines.add(test_line - 1)
        found.append({"path": path, "name": name, "line": line_number, "is_test": is_test, "attached": attribute_lines, "proper_lines": proper_lines, "text": text,
                      "body_keep": keep[start + 1:stop], "body_drop": drop[start + 1:stop], "body_offset": start + 1})
    return found


def clock_reaching_names(functions):
    """同一份文件里读钟的函数，以及（传递地）调了它们的函数。"""
    defined = {function["name"] for function in functions}
    reaching = {function["name"] for function in functions if reads_clock(function["body_drop"])}
    changed = True
    while changed:
        changed = False
        for function in functions:
            if function["name"] not in reaching and callee_names(function["body_drop"]) & defined & reaching:
                reaching.add(function["name"])
                changed = True
    return reaching


def judge_file(path, text, shown, lag, lag_hits):
    """交回 (红的行, 登记了的 [(函数名, 行号)], load-safe 的条数, 扫到的 [(函数名, 行号)])；落在滞后表里的没登记的测试往 lag_hits 记键。"""
    problems, registered, safe_count, tests = [], [], 0, []
    attached_marker_lines = set()
    functions = functions_of(path, text)
    clock_reaching = clock_reaching_names(functions)
    for function in functions:
        if function["is_test"]:
            tests.append((function["name"], function["line"]))
        marker_lines = [(number, line) for number, line in function["attached"] if ANY_MARKER.match(line)]
        for number, _ in marker_lines:
            attached_marker_lines.add(number)
        if function["is_test"] and "placement-unchecked" not in broken:
            for number, _ in marker_lines:
                if number not in function["proper_lines"]:
                    problems.append(f"{shown}:{number}：登记位不在 #[test] 与 fn 之间，也不是 #[test] 上方紧挨着的一行（{function['name']}）")
        registration = next(((number, REGISTRATION.match(line).group(1)) for number, line in function["attached"] if REGISTRATION.match(line)), None)
        load_safe = next(((number, LOAD_SAFE.match(line).group(1)) for number, line in function["attached"] if LOAD_SAFE.match(line)), None)
        if not function["is_test"]:
            for marker in (registration, load_safe):
                if marker and "misplaced-unchecked" not in broken:
                    problems.append(f"{shown}:{marker[0]}：登记位下面的 {function['name']} 不是 #[test] 函数（登记位落空）")
            continue
        assertions = wallclock_assertions(function, clock_reaching)
        where = f"{shown}:{function['line']}（{function['name']}）"
        if assertions:
            places = "、".join(f"{shown}:{line}（{reason}）" for line, reason in assertions[:3])
            if registration and load_safe:
                problems.append(f"{where}：wallclock-assert: 与 wallclock-assert-load-safe: 只能写一种")
            elif not registration and not load_safe and f"{shown}::{function['name']}" in lag:
                lag_hits.append(f"{shown}::{function['name']}")
            elif not registration and not load_safe and "registration-unchecked" not in broken:
                problems.append(f"{where}：有按挂钟判的断言，没有登记位。断言在 {places}")
            elif registration:
                if len(registration[1]) < MINIMUM_REASON_CHARACTERS and "reason-unchecked" not in broken:
                    problems.append(f"{shown}:{registration[0]}：wallclock-assert: 后面要写量的是什么（至少 {MINIMUM_REASON_CHARACTERS} 个字）")
                registered.append((function["name"], function["line"]))
            elif load_safe:
                safe_count += 1
                if len(load_safe[1]) < MINIMUM_LOAD_SAFE_REASON_CHARACTERS and "reason-unchecked" not in broken:
                    problems.append(f"{shown}:{load_safe[0]}：wallclock-assert-load-safe: 后面要写为什么负载再高也不会红（至少 {MINIMUM_LOAD_SAFE_REASON_CHARACTERS} 个字）")
        elif (registration or load_safe) and "misplaced-unchecked" not in broken:
            marker = registration or load_safe
            problems.append(f"{shown}:{marker[0]}：{function['name']} 里没有按挂钟判的断言，登记位写错地方了")
    for number, line in enumerate(text.split("\n"), 1):
        if ANY_MARKER.match(line) and number not in attached_marker_lines and "misplaced-unchecked" not in broken:
            problems.append(f"{shown}:{number}：登记位下面不是 #[test] 函数（落空）")
    return problems, registered, safe_count, tests


def read_lag(root):
    """滞后表 <根>/.claude/gate.d/wallclock-assert-lag.tsv：{键: 行号}，键是「<相对仓根的 .rs>::<函数名>」；写坏的行交回 problems。"""
    path = os.path.join(root, LAG_TABLE)
    rows, problems = {}, []
    if not os.path.isfile(path):
        return rows, problems
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            match = re.match(r"^(scripts/experiment-apparatus/[\w.-]+\.rs::\w+)\s+#\s*(.*\S)\s*$", line)
            if not match or len(match.group(2)) < LAG_REASON_CHARACTERS:
                problems.append(f"{LAG_TABLE}:{number}：这一行不是「<相对仓根的 .rs>::<函数名>  # <理由，至少 {LAG_REASON_CHARACTERS} 个字>」")
            elif match.group(1) in rows:
                problems.append(f"{LAG_TABLE}:{number}：{match.group(1)} 重复登记")
            else:
                rows[match.group(1)] = number
    return rows, problems


def collision_problems(registered, all_tests):
    """--skip <登记名> 是子串匹配：登记名是别的 #[test] 函数名的子串（含同名）而那一条没登记，--skip 会把它一起跳过。
    registered、all_tests 的元素都是 (显示路径, 函数名, 行号)。"""
    registered_keys = {(shown, name, line) for shown, name, line in registered}
    problems = []
    for shown, name, line in registered:
        for other_shown, other_name, other_line in all_tests:
            if (other_shown, other_name, other_line) == (shown, name, line) or (other_shown, other_name, other_line) in registered_keys or name not in other_name:
                continue
            how = "同名" if other_name == name else f"名字 {other_name} 含登记名"
            problems.append(f"{shown}:{line}（{name}）：cargo test 的 --skip 是子串匹配，会连 {other_shown}:{other_line} 一起跳过（{how}），而那一条没登记；改名，或把那一条也登记")
    return problems


def scan(root):
    directory = os.path.join(root, EXPERIMENT_DIRECTORY)
    names = sorted(name for name in os.listdir(directory) if name.endswith(".rs")) if os.path.isdir(directory) else []
    lag, problems = read_lag(root)
    registered, lag_hits, safe_total, all_tests = [], [], 0, []
    for name in names:
        shown = os.path.join(EXPERIMENT_DIRECTORY, name)
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            file_problems, file_registered, file_safe, file_tests = judge_file(os.path.join(directory, name), handle.read(), shown, lag, lag_hits)
        problems += file_problems
        registered += [(shown, function_name, line) for function_name, line in file_registered]
        safe_total += file_safe
        all_tests += [(shown, function_name, line) for function_name, line in file_tests]
    if "collision-unchecked" not in broken:
        problems += collision_problems(registered, all_tests)
    if "lag-unchecked" not in broken:
        for key, number in lag.items():
            if key not in lag_hits:
                problems.append(f"{LAG_TABLE}:{number}：{key} 已经登记了、不再有挂钟断言或函数不在了，滞后表只缩不涨，删掉这一行")
    return names, problems, registered, safe_total, len(all_tests), lag_hits


def judge(root, list_only):
    names, problems, registered, safe_total, tests, lag_hits = scan(root)
    if not problems and (not names or not tests):
        print(f"  ! 本次无对象可判：{os.path.join(root, EXPERIMENT_DIRECTORY)} 下{'一份 .rs 都没有' if not names else '没扫到 #[test] 函数'}")
        return 77
    if problems:
        for problem in problems:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 挂钟断言的登记位有 {len(problems)} 处不对（扫了 {len(names)} 份 .rs、{tests} 个 #[test]）")
        print("     → 怎么办：量挂钟、负载高会误红的断言，在 #[test] 与 fn 之间（或 #[test] 上方紧挨着的一行）写 `// wallclock-assert: <量的是什么，至少 8 个字>`，只写这一行注释、不改装置代码：")
        print("               门禁的单测格按登记名给 cargo test 传 --skip，登记了的测试本次不跑、报本次未跑。")
        print("               判的方向不怕负载（下界、比配置值）的写 `// wallclock-assert-load-safe: <为什么负载再高也不会红>`，门禁里照跑；登记位写错地方的挪走或删掉。")
        print("               登记名撞上别的测试名（--skip 是子串匹配）的：改名，或把撞上的那一条也登记。")
        return 1
    if list_only:
        for shown, name, _ in registered:
            print(f"{shown}\t{name}")
        return 0
    listed = "、".join(name for _, name, _ in registered) or "（没有）"
    print(f"  ✓ 挂钟断言的登记位都对：扫了 {len(names)} 份 .rs、{tests} 个 #[test]；登记 {len(registered)} 条（{listed}），load-safe {safe_total} 条")
    if lag_hits:
        print(f"  ! 另有 {len(lag_hits)} 条在滞后表 {LAG_TABLE} 里，有挂钟断言而还没复核登记（不算已登记，也不在门禁单测里 --skip）：{'、'.join(lag_hits)}")
    return 0



# 样本里测试要调的、读了钟的辅助函数（判法的读钟关靠它们过）
SAMPLE_CLOCK_HELPERS = """
    fn rate() -> u64 { std::time::Instant::now().elapsed().as_nanos() as u64 }
    fn run_sync() -> u64 { rate() }
    fn run_nosync() -> u64 { rate() }
"""


def sample_files():
    """自证用的 .rs 样本：格名 -> (文件内容, 期望：None 是绿，字符串是红的输出里要有的一段)。"""
    return {
        "green": ("""
    fn measure() -> u64 { std::time::Instant::now().elapsed().as_nanos() as u64 }
    #[test]
    // wallclock-assert: 4 线程比 1 线程快
    fn faster_with_threads() {
        let one_thread_nanoseconds = measure();
        let four_thread_nanoseconds = measure();
        assert!(four_thread_nanoseconds < one_thread_nanoseconds, "实测 {four_thread_nanoseconds}");
    }
""", None),
        "green-above-attribute": ("""
    // wallclock-assert: 每秒吞吐要超过 10 块
    #[test]
    fn rate_is_high() {
        let throughput = rate();
        assert!(throughput > 10);
    }
""", None),
        "unregistered": ("""
    #[test]
    fn faster_with_threads() {
        let started = std::time::Instant::now();
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }
""", "有按挂钟判的断言，没有登记位"),
        "propagated-unregistered": ("""
    #[test]
    fn nosync_is_faster() {
        let mut faster = false;
        for _ in 0..3 {
            let (sync_nanoseconds, nosync_nanoseconds) = (run_sync(), run_nosync());
            let (sync_rate, nosync_rate) = (rate(60, sync_nanoseconds), rate(60, nosync_nanoseconds));
            if nosync_rate > sync_rate * 2 {
                faster = true;
                break;
            }
        }
        assert!(faster, "不 fdatasync 该更快");
    }
""", "由挂钟量传来"),
        "reason-short": ("""
    #[test]
    // wallclock-assert: 太短
    fn rate_is_high() {
        let throughput = rate();
        assert!(throughput > 10);
    }
""", "后面要写量的是什么（至少 8 个字）"),
        "marker-too-far": ("""
    // wallclock-assert: 每秒吞吐要超过 10 块
    /// 文档注释插在登记位与 #[test] 之间
    #[test]
    fn rate_is_high() {
        let throughput = rate();
        assert!(throughput > 10);
    }
""", "不在 #[test] 与 fn 之间，也不是 #[test] 上方紧挨着的一行"),
        "misplaced": ("""
    #[test]
    // wallclock-assert: 写在了没有挂钟量的断言上
    fn sums() {
        assert_eq!(1 + 1, 2);
    }
""", "登记位写错地方了"),
        "orphan": ("""
    // wallclock-assert: 下面不是测试函数
    fn helper() -> u64 { 5 }
""", "不是 #[test] 函数（登记位落空）"),
        "load-safe-green": ("""
    // wallclock-assert-load-safe: 只比下界，睡得再久也不会红
    #[test]
    fn sleeps_at_least() {
        let started = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(started.elapsed() >= std::time::Duration::from_millis(5));
    }
""", None),
        "load-safe-misplaced": ("""
    // wallclock-assert-load-safe: 这条断言里根本没有挂钟量
    #[test]
    fn adds() {
        assert_eq!(2 + 2, 4);
    }
""", "登记位写错地方了"),
        "pure-values-green": ("""
    #[test]
    fn lifetime_from_literal_rate() {
        let rate_of_one_million_per_second = 1_000_000_000u64;
        assert_eq!(lifetime_years(48, rate_of_one_million_per_second), 8);
        assert_eq!(rate_in_thousandths_per_second(1, 1), 1_000_000_000_000);
        assert_eq!(parse("a").limit, Duration::from_secs(120));
    }
""", None),
        "name-collision": ("""
    #[test]
    // wallclock-assert: 每秒吞吐要超过 10 块
    fn rate_is_high() {
        let throughput = rate();
        assert!(throughput > 10);
    }

    #[test]
    fn rate_is_high_on_an_empty_input() {
        assert_eq!(1 + 1, 2);
    }
""", "会连 scripts/experiment-apparatus/sample.rs:"),
        "collision-both-registered-green": ("""
    #[test]
    // wallclock-assert: 每秒吞吐要超过 10 块
    fn rate_is_high() {
        let throughput = rate();
        assert!(throughput > 10);
    }

    #[test]
    // wallclock-assert: 空输入时每秒吞吐也要超过 10 块
    fn rate_is_high_on_an_empty_input() {
        let throughput = rate();
        assert!(throughput > 10);
    }
""", None),
    }


def run_selftest():
    cells = []
    with tempfile.TemporaryDirectory(prefix="wallclock-assert-lint-") as scratch:
        for cell, (content, expected) in sample_files().items():
            root = os.path.join(scratch, cell)
            directory = os.path.join(root, EXPERIMENT_DIRECTORY)
            os.makedirs(directory)
            with open(os.path.join(directory, "sample.rs"), "w", encoding="utf-8") as handle:
                handle.write("#[cfg(test)]\nmod tests {\n" + SAMPLE_CLOCK_HELPERS + content + "}\n")
            import io
            import contextlib
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                status = judge(root, False)
            text = output.getvalue()
            if expected is None:
                cells.append((cell, status == 0, f"该绿，实得退 {status}：{text.strip()[:200]}"))
            else:
                cells.append((cell, status == 1 and expected in text, f"该红并点名「{expected}」，实得退 {status}：{text.strip()[:200]}"))
        # --list 只列登记了的
        root = os.path.join(scratch, "green")
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = judge(root, True)
        cells.append(("list", status == 0 and output.getvalue() == "scripts/experiment-apparatus/sample.rs\tfaster_with_threads\n",
                      f"--list 该只列一行「scripts/experiment-apparatus/sample.rs<TAB>faster_with_threads」，实得 {output.getvalue()!r}"))
        # 滞后表：没登记的挂钟断言落在表里不判红（成功行另报）；表里的行对应的测试已经登记就是陈旧行，判红
        unmarked = sample_files()["unregistered"][0]
        for cell, content, lag_row, expected in (
                ("lag-listed", unmarked, "scripts/experiment-apparatus/sample.rs::faster_with_threads  # 立表之前就在仓里，还没复核", "另有 1 条在滞后表"),
                ("lag-stale", sample_files()["green"][0], "scripts/experiment-apparatus/sample.rs::faster_with_threads  # 已经登记了，这一行该删", "滞后表只缩不涨"),
                ("lag-malformed", unmarked, "scripts/experiment-apparatus/sample.rs::faster_with_threads  # 短", "这一行不是")):
            root = os.path.join(scratch, cell)
            os.makedirs(os.path.join(root, EXPERIMENT_DIRECTORY))
            os.makedirs(os.path.join(root, os.path.dirname(LAG_TABLE)))
            with open(os.path.join(root, EXPERIMENT_DIRECTORY, "sample.rs"), "w", encoding="utf-8") as handle:
                handle.write("#[cfg(test)]\nmod tests {\n" + SAMPLE_CLOCK_HELPERS + content + "}\n")
            with open(os.path.join(root, LAG_TABLE), "w", encoding="utf-8") as handle:
                handle.write("# 样本\n" + lag_row + "\n")
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                status = judge(root, False)
            cells.append((cell, status == (0 if cell == "lag-listed" else 1) and expected in output.getvalue(), f"该含「{expected}」，实得退 {status}：{output.getvalue().strip()[:200]}"))
        # 空目录退 77
        empty = os.path.join(scratch, "empty")
        os.makedirs(os.path.join(empty, EXPERIMENT_DIRECTORY))
        with contextlib.redirect_stdout(io.StringIO()):
            status = judge(empty, False)
        cells.append(("empty", status == 77, f"没有 .rs 该退 77，实得 {status}"))
    failed = [cell for cell in cells if not cell[1]]
    for name, passed, message in cells:
        if not passed:
            print(f"  ✗ 自证 {name}：{message}")  # gate-lint:detail
    if failed:
        print(f"  ✗ --selftest：{len(cells)} 格里 {len(failed)} 格不对")
        print("     → 怎么办：照上面点名的格查判法；弄坏开关打开时红在点名的格是对的")
        return 1
    print(f"  ✓ --selftest：{len(cells)} 格都对（绿、登记位两个位置、没登记、一步传递、理由太短、登记位太远、登记位写错地方、落空、load-safe、纯字面量不误报、撞名、滞后表、--list、空目录）")
    return 0


def switch_wheel():
    expected = {"registration-unchecked": ("unregistered", "propagated-unregistered"), "misplaced-unchecked": ("misplaced", "orphan", "load-safe-misplaced"),
                "placement-unchecked": ("marker-too-far",), "reason-unchecked": ("reason-short",), "collision-unchecked": ("name-collision",),
                "propagation-off": ("propagated-unregistered",), "lag-unchecked": ("lag-stale",)}
    failures = 0
    for switch, wanted in expected.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest-inner"], capture_output=True, text=True,
                                   env=dict(os.environ, WALLCLOCK_ASSERT_LINT_BREAK=switch))
        red_cells = re.findall(r"✗ 自证 ([a-z-]+)：", completed.stdout)
        if completed.returncode == 1 and sorted(red_cells) == sorted(wanted):
            print(f"  · 开关轮：{switch} 红在「{'、'.join(red_cells)}」，与点名的一致")
        else:
            failures += 1
            print(f"  ✗ 开关轮：{switch} 打开后退 {completed.returncode}、红在「{'、'.join(red_cells) or '（没有）'}」，点名的是「{'、'.join(wanted)}」")  # gate-lint:detail
    if failures:
        print(f"  ✗ 开关轮 {len(expected)} 个开关里 {failures} 个没红在点名的格")
        print("     → 怎么办：改那一处判法，或照实改文件头的开关表")
        return 1
    print(f"  ✓ 开关轮：{len(expected)} 个开关逐个打开，各红在点名的格")
    return 0


def main(arguments):
    if arguments[:1] == ["--selftest-inner"]:
        broken.update(item for item in os.environ.get("WALLCLOCK_ASSERT_LINT_BREAK", "").split(",") if item in BREAKS)
        return run_selftest()
    if arguments[:1] == ["--selftest"]:
        broken.update(item for item in os.environ.get("WALLCLOCK_ASSERT_LINT_BREAK", "").split(",") if item in BREAKS)
        if broken:   # 设了弄坏开关：只跑这一轮，红在点名的格是对的
            return run_selftest()
        status = run_selftest()
        return status or switch_wheel()
    if os.environ.get("WALLCLOCK_ASSERT_LINT_BREAK"):
        print("  ! WALLCLOCK_ASSERT_LINT_BREAK 只在 --selftest 里认，这一次不理它")
    list_only = arguments[:1] == ["--list"]
    rest = arguments[1:] if list_only else arguments
    if any(argument.startswith("-") for argument in rest) or len(rest) > 1:
        print(f"  ✗ 认不得的参数：{' '.join(arguments)}")
        print("     → 怎么办：用法 wallclock-assert-lint.py [--list] [仓根] 或 --selftest")
        return 2
    root = os.path.realpath(rest[0]) if rest else REPOSITORY_ROOT
    return judge(root, list_only)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
