#!/usr/bin/env bash
# 门禁 code-tooling 的 heavy-thread-defaults 格认合规写法的几段判法：python 正文放在 HEAVY_THREAD_RECOGNITION_LIBRARY 里。
# 由 lib/code-tooling-source-scan-cells.sh 在它自己被 source 时 source，不单独跑；那一格把这份正文 exec 进一个模块对象
# （heavy_thread_recognition = types.ModuleType("lib_heavy_thread_recognition")，exec(compile(<正文>, "heavy-thread-recognition-library", "exec"), heavy_thread_recognition.__dict__)；
# 算键库认的写法，scripts/gate/lib_function_units.py 文件头「exec 进模块对象」），按 heavy_thread_recognition.<函数>(…) 调。
# 正文自己只 import re；函数里读的那一格的判法、表与常量，由那一格在 exec 之后、调之前逐个赋到模块对象上（heavy_thread_recognition.<名> = …）：
# HALF、SAMPLE_SCOPE_FUNCTION_NAME、SHELL_PATHS、rust_code_lines、rust_test_module_starts、scanned_files、whole_test_paths，
# 与 comment_stripped_shell_lines（共用库 SOURCE_CODE_LINES_LIBRARY 的 shell_code_lines，第二个参数是认出路文字的式子）；正文里加读那一格的名字，那一格要跟着加一行赋值。
# 弄坏开关也由那一格读（这一份里不读 CODE_TOOLING_BREAK）。判据写在那一格上方的注释里「认得出、不点名的」那几条。
# 三段：
#   shell 自证函数的行（shell_selftest_lines、single_quoted_mask）：顶格写的、名字里带 selftest / self_test 的 shell 函数（shell_function_spans）；
#   登记位包装函数（registry_wrapper_functions、calls_registry_wrapper）：顶格写的 shell 函数里调 heavy_thread_default，往标准输出写的别的命令
#     只在条件里有弄坏开关的 if 块里、或写到标准错误（registry_wrapper_body）；同一个名字在几份被扫的文件里都有定义的，每一份都要是；
#   rust 从登记位取好的线程数再分（registered_split）：`/ 2` 左边是登记位读法 default_heavy_threads()（可带路径与 .get()），或是一个名字，
#     这份文件里它的每一处 let 绑定右边整条是登记位读法、没有别的赋值，或它是外面那个函数的参数、函数不是方法，被扫的 rust 文件里
#     （行首 #[cfg(test)] 罩着的测试模块往后不算）每一处调那个函数都在那个位置上交登记位读法（rust_registry_sourced）。
# 判不了的：shell 函数不是顶格写、收尾的 } 不单独占一行；if 块写成 case、[[ … ]] && 一类；rust 的签名超过 8 行、参数是模式、
#   函数经函数指针或 trait 调、在宏里调、调用处的实参跨在没被扫的文件里；这些照旧判红（不靠登表）。rust 的名字被 for、闭包参数、match 的模式同名遮蔽，
#   这一份认不出（照参数或 let 绑定认）。
HEAVY_THREAD_RECOGNITION_LIBRARY="$(cat <<'PY_RECOGNITION_BODY'
import re

SHELL_TOP_FUNCTION = re.compile(r"(?:function\s+)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(\)\s*\{")


def shell_function_spans(code_lines):
    """顶格写的 shell 函数：[(名字, 起行, 止行, 函数体的行)]，行号从 1 起。`名() {` 起头，到下一行只有一个 `}` 为止（套在里面的函数算在外层那一个里）；
    起头那一行里大括号配平、以 `}` 收尾的是写在一行里的函数（`名() { …; }`），函数体就是两个大括号之间那一段。"""
    spans, index = [], 0
    while index < len(code_lines):
        line = code_lines[index]
        found = SHELL_TOP_FUNCTION.match(line)
        if not found:
            index += 1
            continue
        rest = line[found.end():].rstrip()
        if rest.endswith("}") and line.count("{") == line.count("}"):
            spans.append((found.group("name"), index + 1, index + 1, [rest[:-1]]))
            index += 1
            continue
        end = next((row for row in range(index + 1, len(code_lines)) if code_lines[row].rstrip() == "}"), len(code_lines) - 1)
        spans.append((found.group("name"), index + 1, end + 1, code_lines[index + 1:end]))
        index = end + 1
    return spans


def shell_selftest_lines(code_lines):
    """shell 自证函数（名字里带 selftest / self_test，同 python_sample_scope_lines 的认法）的行号集合（1 起）。"""
    return frozenset(number for name, first, last, _body in shell_function_spans(code_lines) if SAMPLE_SCOPE_FUNCTION_NAME.search(name)
                     for number in range(first, last + 1))


def single_quoted_mask(line):
    """每个字符在不在 shell 的单引号里（单引号里的 $ 不展开：自证里当样本正文写出去的那几段）；双引号里认反斜杠转义，单引号不在双引号里起头。"""
    mask, quote, index = [False] * len(line), None, 0
    while index < len(line):
        character = line[index]
        if quote:
            mask[index] = quote == "'"
            if character == "\\" and quote == '"' and index + 1 < len(line):
                index += 2
                continue
            if character == quote:
                quote = None
        elif character in "\"'":
            quote = character
            mask[index] = quote == "'"
        elif character == "\\":
            index += 2
            continue
        index += 1
    return mask


# 登记位包装函数（shell）：一个函数里调 heavy_thread_default，往标准输出写的别的命令只在弄坏开关护着的 if 块里、或写到标准错误
SHELL_REGISTRY_CALL = re.compile(r"(?:^|[;&|({!]|\b(?:then|do|else|if)\b)[ \t]*heavy_thread_default\b(?![ \t]*(?:=|\())")
SHELL_OUTPUT_COMMAND = re.compile(r"(?:^|[;&|({!]|\b(?:then|do|else)\b)[ \t]*(?:echo|printf|cat)\b")
SHELL_BREAK_GUARD = re.compile(r"\$\{?[A-Z][A-Z0-9_]*_BREAK\b|\b[a-z_]*break_is\b")
SHELL_IF_START = re.compile(r"^[ \t]*if\b")
SHELL_ELSE = re.compile(r"^[ \t]*(?:else\b|elif\b)")
SHELL_FI = re.compile(r"(?:^|[;&|])[ \t]*fi\b")
REGISTRY_WRAPPER_CALL = re.compile(r"\$\([ \t]*([A-Za-z_][A-Za-z0-9_]*)[ \t]*\)|`[ \t]*([A-Za-z_][A-Za-z0-9_]*)[ \t]*`")
NO_OUTPUT_TEXT = re.compile(r"(?!)")
registry_wrapper_cache = []


def registry_wrapper_body(body):
    """shell 函数体（去掉注释、不去出路文字的行）是不是只经登记位取数：有一处调 heavy_thread_default；往标准输出写的每一处（echo、printf、cat）
    要么同一行写到标准错误（>&2），要么落在条件里有弄坏开关的 if 块里（那一支只在自证打开开关时走到；else / elif 那一支不算护着）。"""
    if not any(SHELL_REGISTRY_CALL.search(line) for line in body):
        return False
    guards = []
    for line in body:
        if SHELL_IF_START.match(line):
            guards.append(SHELL_BREAK_GUARD.search(line) is not None)
        elif SHELL_ELSE.match(line) and guards:
            guards[-1] = line.lstrip().startswith("elif") and SHELL_BREAK_GUARD.search(line) is not None
        if SHELL_OUTPUT_COMMAND.search(line) and ">&2" not in line and not any(guards):
            return False
        if SHELL_FI.search(line) and guards:
            guards.pop()
    return True


def registry_wrapper_functions():
    """被扫的 shell 文件里顶格写的登记位包装函数的名字（registry_wrapper_body；同一个名字在几份文件里都有定义的，每一份都要是）；
    整趟只算一次。"""
    if not registry_wrapper_cache:
        verdicts = {}
        for path in SHELL_PATHS:
            try:
                lines = comment_stripped_shell_lines(open(path, encoding="utf-8", errors="replace").read(), NO_OUTPUT_TEXT)
            except OSError:
                continue
            for name, _first, _last, body in shell_function_spans(lines):
                verdicts[name] = verdicts.get(name, True) and registry_wrapper_body(body)
        registry_wrapper_cache.append(frozenset(name for name, ok in verdicts.items() if ok))
    return registry_wrapper_cache[0]


def calls_registry_wrapper(statement):
    """一条 shell 语句里有 $(<登记位包装函数>) 或 `<登记位包装函数>`。"""
    wrappers = registry_wrapper_functions()
    return any((found.group(1) or found.group(2)) in wrappers for found in REGISTRY_WRAPPER_CALL.finditer(statement))


# 从登记位取好的线程数再分（rust）：`/ 2` 左边是登记位读法，或是一个追得到登记位的名字（rust_registry_sourced）
RUST_REGISTRY_READ = r"(?:[A-Za-z_][A-Za-z0-9_]*::)*default_heavy_threads\(\)(?:\.get\(\))?"
RUST_REGISTRY_READ_WHOLE = re.compile(RUST_REGISTRY_READ)
RUST_REGISTRY_READ_AT_END = re.compile(RUST_REGISTRY_READ + r"$")
RUST_NAME_AT_END = re.compile(r"(?<![\w.:])([a-z_][A-Za-z0-9_]*)$")
RUST_FUNCTION_HEAD = re.compile(r"\bfn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{]*>)?\s*\(")
SIGNATURE_LINE_LIMIT = 8
rust_scanned_code_cache = {}


def rust_code_before_test_module(code_lines):
    """rust 代码行里这一格判的那一截：行首 #[cfg(test)] 罩着的测试模块往后不算（rust_test_module_starts，同主循环）。"""
    for index, code in enumerate(code_lines):
        if code.startswith("#[cfg(test)]") and rust_test_module_starts(code_lines, index):
            return code_lines[:index]
    return code_lines


def rust_scanned_code(path):
    """一份被扫的 rust 文件此刻的 rust_code_before_test_module；读不了交 []。"""
    if path not in rust_scanned_code_cache:
        try:
            lines = rust_code_lines(open(path, encoding="utf-8", errors="replace").read())
        except OSError:
            lines = []
        rust_scanned_code_cache[path] = rust_code_before_test_module(lines)
    return rust_scanned_code_cache[path]


def balanced_arguments(text, opening):
    """text[opening] 是 `(`：到配平的 `)` 为止按顶层逗号切开的实参（去掉两头空白）；配不平交 None。"""
    depth, start, parts = 0, opening + 1, []
    for index in range(opening, len(text)):
        character = text[index]
        if character in "([{":
            depth += 1
        elif character in ")]}":
            depth -= 1
            if depth == 0:
                parts.append(text[start:index].strip())
                return [part for part in parts if part] if parts != [""] else []
        elif character == "," and depth == 1:
            parts.append(text[start:index].strip())
            start = index + 1
    return None


def enclosing_rust_parameter(code_lines, number, name):
    """第 number 行（1 起）往上最近的一个 fn 头（签名可以跨行，至多 SIGNATURE_LINE_LIMIT 行）里 name 是第几个参数（0 起）与函数名；
    name 不是它的参数、它带 self（方法）、签名读不全交 None。"""
    for index in range(number - 1, -1, -1):
        found = RUST_FUNCTION_HEAD.search(code_lines[index])
        if not found:
            continue
        signature = "\n".join(code_lines[index:index + SIGNATURE_LINE_LIMIT])
        parameters = balanced_arguments(signature, signature.index("(", found.end() - 1))
        if parameters is None or any(re.match(r"&?\s*(?:mut\s+)?self\b", parameter) for parameter in parameters):
            return None
        position = next((place for place, parameter in enumerate(parameters) if re.match(r"(?:mut\s+)?" + re.escape(name) + r"\s*:", parameter)), None)
        return None if position is None else (found.group("name"), position)
    return None


def rust_registry_sourced(path, code_lines, number, name):
    """rust 里第 number 行读的 name 追得到登记位（这份文件里 name 都没有 let 之外的赋值）：name 是外面那个函数（enclosing_rust_parameter）的参数、函数不是方法、
    文件里也没有同名的 let 绑定，被扫的 rust 文件里（这一格判的那一截）每一处调那个函数都在那个位置上交登记位读法（至少一处）；
    name 不是那个函数的参数的，这份文件里它的每一处 let 绑定右边整条是登记位读法（至少一处）。"""
    text = "\n".join(rust_code_before_test_module(code_lines))
    bindings = [found.group("value").strip() for found in re.finditer(r"\blet\s+(?:mut\s+)?" + re.escape(name) + r"\b\s*(?::[^=;]*)?=(?!=)\s*(?P<value>[^;]*);", text)]
    for found in re.finditer(r"(?<![\w.:])" + re.escape(name) + r"\s*[-+*/%]?=(?![=>])", text):
        if not re.search(r"\blet\s+(?:mut\s+)?$", text[:found.start()]):
            return False      # 别的赋值写法：值不只来自登记位
    parameter = enclosing_rust_parameter(code_lines, number, name)
    if parameter is None:
        return bool(bindings) and all(RUST_REGISTRY_READ_WHOLE.fullmatch(value) for value in bindings)
    if bindings:
        return False      # 参数又有同名的 let 绑定：读的是哪一个分不清
    function, position = parameter
    call = re.compile(r"(?<![\w.])(?:[A-Za-z_][A-Za-z0-9_]*::)*" + re.escape(function) + r"\s*\(")
    arguments_seen = 0
    for _directory, other, language in scanned_files():
        if language != "rust" or other in whole_test_paths:
            continue
        try:
            if function not in open(other, encoding="utf-8", errors="replace").read():
                continue
        except OSError:
            return False
        other_text = "\n".join(rust_scanned_code(other))
        for found in call.finditer(other_text):
            if re.search(r"\bfn\s+$", other_text[:found.start()]):
                continue      # 函数自己的定义那一行
            arguments = balanced_arguments(other_text, found.end() - 1)
            if arguments is None or len(arguments) <= position or not RUST_REGISTRY_READ_WHOLE.fullmatch(arguments[position]):
                return False
            arguments_seen += 1
    return arguments_seen > 0


def registered_split(path, code_lines, number):
    """rust 里第 number 行每一处除以 2（HALF 认的那几处）的左边都是登记位读法、或追得到登记位的名字（rust_registry_sourced）：是从已经按登记位取好的线程数分出去，
    不是自己取核数定并行度。"""
    code = code_lines[number - 1]
    matches = list(HALF.finditer(code))
    for found in matches:
        prefix = code[:found.start()].rstrip()
        if RUST_REGISTRY_READ_AT_END.search(prefix):
            continue
        operand = RUST_NAME_AT_END.search(prefix)
        if operand is None or not rust_registry_sourced(path, code_lines, number, operand.group(1)):
            return False
    return bool(matches)
PY_RECOGNITION_BODY
)"
