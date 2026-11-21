# 别的程序里起的命令（embedded_launches）：python、perl、ruby、node、php、lua、tclsh、awk、sed、make、git、tar、ssh、less、vim 一族的认法。
# lib_shell_words.py 把这里的名字全部再导出，调用方只 import lib_shell_words，不直接导入这一份。
import os
import re
import shlex
from typing import NamedTuple
from lib_shell_words_core import (
    ASSIGNMENT,
    PREFIX_OPTIONS_WITH_VALUE,
    PYTHON_NAME,
    RENAMED_SHELL_NAMES,
    SHELL_NAMES,
    VARIABLE_REFERENCE,
    WRAPPERS_TAKING_ONE_ARGUMENT,
    backquote_end,
    command_substitution_end,
    expand_known_variables,
    has_expansion,
    is_shell_name,
    python_invocation,
    resolve_path,
    shell_standard_input,
    shell_standard_input_files,
    words_break_is_set,
)


# 别的程序里起 shell 与命令（认不出的一律拒，偏严）：
# python / perl / ruby / node / php / lua 的 -c、-e 这类代码串与喂进标准输入的字面代码，awk 的程序文本，sed 的脚本，tclsh / expect 的代码，make 的 Makefile 与 --eval，
# git 的别名与带命令的配置项，tar、ssh、less、vim 带命令的选项。字面写着的命令照一段 shell 命令交出（调用方同 bash -c 那样再判）；起了命令而写法认不出的（变量、拼接、
# 动态求值）交一句说明，调用方照认不出办；程序从文件读的代码（awk -f、sed -f、Makefile）交文件，调用方读正文交 program_text_launches 判。
# python / perl / ruby / tclsh 起的脚本文件不读（.claude/agent-common.md「写」一节：python subprocess 起的、脚本文件里的写法由子 agent 自己守）。
class ProgramFile(NamedTuple):
    language: str          # "awk"、"sed"、"make"：program_text_launches 照哪种语言读
    words: tuple           # 候选路径词（照写的样子，没解开）：make 没给 -f 时按 GNUmakefile、makefile、Makefile 的次序取头一份在的，别的只有一个
    directory: str | None  # 解相对路径用的当前目录（make -C 换过的）
    variables: tuple = ()  # make / awk 命令行上的 `名字=值`：((名字, 值), …)，读 Makefile 时先于文件里的定义；awk 的值写成 shell 变量、命令替换的是 None


class EmbeddedLaunches(NamedTuple):
    texts: tuple           # 程序里字面写着、照一段 shell 命令判的文本
    unjudged: tuple        # 起了命令、写法认不出的说明（每处一句）
    program_files: tuple   # (ProgramFile, …)


EMBEDDED_PYTHON_SHELL_CALLS = {("os", "system"), ("os", "popen"), ("posix", "system"), ("subprocess", "getoutput"), ("subprocess", "getstatusoutput"),
                               ("commands", "getoutput"), ("commands", "getstatusoutput"), ("asyncio", "create_subprocess_shell")}
EMBEDDED_PYTHON_SUBPROCESS_CALLS = {"run", "call", "check_call", "check_output", "Popen"}
EMBEDDED_PYTHON_EXEC_LIST_CALLS = {"execl", "execle", "execlp", "execlpe"}        # (程序, argv0, 参数…)
EMBEDDED_PYTHON_EXEC_VECTOR_CALLS = {"execv", "execve", "execvp", "execvpe"}     # (程序, [argv0, 参数…])
EMBEDDED_PYTHON_SPAWN_LIST_CALLS = {"spawnl", "spawnle", "spawnlp", "spawnlpe"}  # (模式, 程序, argv0, 参数…)
EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS = {"spawnv", "spawnve", "spawnvp", "spawnvpe"}
EMBEDDED_PYTHON_LAUNCH_MODULES = {"os", "posix", "subprocess", "commands", "asyncio", "pty"}
EMBEDDED_PYTHON_OPAQUE_MODULES = {"ctypes", "cffi"}   # 能直接调 libc 的 system / exec：认不出它起什么
EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES = ({"system", "popen", "Popen", "check_output", "check_call", "getoutput", "getstatusoutput", "posix_spawn", "posix_spawnp",
                                          "create_subprocess_shell", "create_subprocess_exec"}
                                         | EMBEDDED_PYTHON_EXEC_LIST_CALLS | EMBEDDED_PYTHON_EXEC_VECTOR_CALLS | EMBEDDED_PYTHON_SPAWN_LIST_CALLS
                                         | EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS)
EMBEDDED_PYTHON_SYNTAX_HINTS = ("system", "popen", "Popen", "subprocess", "exec", "spawn", "pty", "ctypes", "__import__", "import_module")
PERL_LAUNCH_WORD = re.compile(r"(?<![\w$@%&>:.\-])(system|exec|readpipe|qx|open)\b")
RUBY_LAUNCH_WORD = re.compile(r"(?<![\w$@:.])(?:system|exec|spawn|IO\.popen|Open3\.\w+|Process\.spawn|Kernel\.(?:system|exec|spawn)|open)\b|%x")
NODE_LAUNCH_WORD = re.compile(r"(?<![\w$])(execSync|execFileSync|execFile|spawnSync|spawn|exec|fork)\s*\(")
PHP_LAUNCH_WORD = re.compile(r"(?<![\w$>:])(system|exec|shell_exec|passthru|popen|proc_open|pcntl_exec)\s*\(")
LUA_LAUNCH_WORD = re.compile(r"\b(os\.execute|io\.popen)\b")
TCL_LAUNCH_COMMANDS = {"exec", "spawn", "exp_spawn", "system", "exp_system"}
TCL_OPAQUE_COMMANDS = {"eval", "uplevel", "subst", "interp", "source"}
AWK_NAMES = {"awk", "gawk", "mawk", "nawk"}
PERL_NAME = re.compile(r"^perl[0-9.]*$")
# perl 带值的选项，值另起一个词（-I 的目录；-M、-m 的模块名也按这一类跳过）
PERL_OPTIONS_WITH_VALUE_WORD = ("-I", "-M", "-m")
# perl 的值只能贴着选项写的选项（perlrun：-i[后缀]、-l[八进制]、-0[八进制]、-x[目录]、-C[数字]、-d[:模块]、-D[标志]）：单独写时后面的词是下一个选项或脚本，不是它的值
PERL_OPTIONS_WITH_ATTACHED_VALUE = ("-x", "-0", "-l", "-i", "-C", "-d", "-D")
RUBY_NAME = re.compile(r"^ruby[0-9.]*$")
PHP_NAME = re.compile(r"^php[0-9.]*$")
LUA_NAME = re.compile(r"^(?:lua[0-9.]*|luajit)$")
TCL_NAMES = {"tclsh", "wish", "expect"}
TCL_NAME = re.compile(r"^(?:tclsh|wish)[0-9.]*$|^expect$")
NODE_NAMES = {"node", "nodejs"}
VIM_NAMES = {"vim", "vi", "nvim", "ex", "view", "vimdiff", "gvim", "rvim"}
MAKE_NAMES = {"make", "gmake"}
MAKE_DEFAULT_FILES = ("GNUmakefile", "makefile", "Makefile")
GIT_COMMAND_CONFIG_KEY = re.compile(r"^(?:core\.(?:pager|editor|sshcommand|fsmonitor|askpass|alternaterefscommand)|sequence\.editor|diff\.external|diff\..+\.(?:command|textconv)"
                                    r"|merge\..+\.driver|filter\..+\.(?:clean|smudge|process)|gpg\.program|gpg\..+\.program|pager\..+|interactive\.difffilter"
                                    r"|uploadpack\.packobjectshook)$", re.I)
GIT_CONFIG_QUERY_PATTERN = (r"^(alias\..*|core\.(pager|editor|sshcommand|fsmonitor|askpass|alternaterefscommand)|sequence\.editor|diff\.external|diff\..*\.(command|textconv)"
                            r"|merge\..*\.driver|filter\..*\.(clean|smudge|process)|credential\.helper|credential\..*\.helper|gpg\.program|gpg\..*\.program|pager\..*"
                            r"|interactive\.difffilter|uploadpack\.packobjectshook)$")
GIT_BOOLEAN_VALUES = {"true", "false", "yes", "no", "on", "off", "1", "0", ""}
GIT_COMMAND_ENVIRONMENT = ("GIT_EXTERNAL_DIFF", "GIT_PAGER", "PAGER", "GIT_EDITOR", "EDITOR", "VISUAL", "GIT_SEQUENCE_EDITOR", "GIT_SSH_COMMAND", "GIT_SSH",
                           "GIT_ASKPASS", "SSH_ASKPASS", "GIT_PROXY_COMMAND")
GIT_OPAQUE_ENVIRONMENT = ("GIT_EXEC_PATH", "GIT_CONFIG_PARAMETERS")   # 换掉 git 的子命令程序、或带着 git 自己引号写法的配置：认不出起什么
GIT_OPTIONS_WITH_VALUE = {"-c", "-C", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env"}
GIT_CONFIG_QUERY_SECONDS = 5
GIT_CONFIG_QUERY_CACHE = {}
TAR_COMMAND_OPTIONS = ("--to-command", "--use-compress-program", "--info-script", "--new-volume-script", "--rsh-command", "--rmt-command")
SSH_COMMAND_OPTIONS = ("proxycommand", "localcommand", "knownhostscommand")
LESS_OPEN_ENVIRONMENT = ("LESSOPEN", "LESSCLOSE")
MAKE_DEFAULT_VARIABLES = {"MAKE": "make", "CC": "cc", "CXX": "g++", "CPP": "cc -E", "AR": "ar", "AS": "as", "LD": "ld", "RM": "rm -f", "SHELL": "/bin/sh",
                          "LEX": "lex", "YACC": "yacc", "FC": "f77", "M2C": "m2c", "PC": "pc", "CO": "co", "GET": "get", "TEX": "tex", "MAKEINFO": "makeinfo"}
MAKE_RECIPE_PREFIX = re.compile(r"^[@+\-\s]*")
MAKE_ASSIGNMENT = re.compile(r"^(?:(?:export|override|private)\s+)*([A-Za-z_][A-Za-z0-9_.\-]*)\s*(:::=|::=|:=|\?=|\+=|!=|=)\s*(.*)$")
MAKE_VARIABLE_PLACEHOLDER_PREFIX = "MAKE_UNRESOLVED_"   # 展开不了的 make 变量换成没展开的 shell 变量：落在命令词上的照认不出办
EMBEDDED_NESTING = 4


def embedded_launches_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen：别的程序里起的命令一条都不认。"""
    return words_break_is_set("embedded-launches-unseen")


def perl_attached_options_take_next_word():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=perl-bare-options-take-next-word：perl 里值只能贴着写的选项（-i、-l、-0 这类）单独写时也吃掉下一个词。"""
    return words_break_is_set("perl-bare-options-take-next-word")


def heredoc_variables_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded：heredoc 正文里前面赋过字面值的变量不代入。"""
    return words_break_is_set("heredoc-variables-unexpanded")


def heredoc_body_expanded(text, variables):
    """结束词不带引号的 heredoc 正文照 bash 展开之后的文字：`$名字`、`${名字}` 换成这段命令文本里前面赋过的值（variables，名字 → 已展开的值、None 是算不出；
    查不到的只认 HOME，同 expand_known_variables），`\\$`、`\\``、`\\\\` 换成后面那个字，`\\` 接换行去掉（续行），别的反斜杠照留。
    有一处展开不了交 None：没赋过或算不出的变量、`$(…)`、反引号、`$((…))`、`${名字:-…}` 这类别的写法、`$1`、`$@` 这类特殊参数，`$` 后面不接名字的也算
    （复跑分类报告 d137-replay-classify 二节 E1；用户口径「认不出的一律拒」：认得出的照代入，有一处认不出整段照旧判不了）。"""
    output, index = [], 0
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            following = text[index + 1]
            if following != "\n":
                output.append(following if following in "$`\\" else character + following)
            index += 2
            continue
        if character == "`":
            return None
        if character == "$":
            reference = VARIABLE_REFERENCE.match(text, index)
            if reference is None:
                return None
            name = reference.group(1) or reference.group(2)
            value = variables[name] if name in variables else (os.environ.get("HOME") if name == "HOME" else None)
            if value is None:
                return None
            output.append(value)
            index = reference.end()
            continue
        output.append(character)
        index += 1
    return "".join(output)


def interpreter_standard_input(command):
    """解释器从标准输入读代码时读到的：交 (代码文本或 None, 判不了的说明或 None, 是不是盘上的文件)。heredoc 取最后一个的正文：结束词不带引号、正文里有 `$` 或反引号的
    照 heredoc_body_expanded 代入前面赋过字面值的变量再交（弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-variables-unexpanded 下不代入），代入不了的交原文、另交说明；
    `< 文件`、`cat 文件 |` 是盘上的文件（这里不读）；here-string 与管道前一段的 printf / echo 照 shell_standard_input。"""
    if command.standard_input_heredocs:
        body = command.standard_input_heredocs[-1]
        if body.is_delimiter_quoted or not has_expansion([body.text]):
            return body.text, None, False
        expanded = None if heredoc_variables_unexpanded() else heredoc_body_expanded(body.text, command.variables or {})
        if expanded is not None:
            return expanded, None, False
        return body.text, "喂进去的 heredoc 结束词没带引号、正文里有变量或命令替换，展开之后是什么判不了", False
    if shell_standard_input_files(command.input_redirections, command.piped_from):
        return None, None, True
    fed = shell_standard_input(command.input_redirections, command.piped_from)
    return fed.code, fed.unjudged, False


def literal_quoted_string(text, start, interpolating_marks):
    """text[start] 是 ' 或 " 或反引号：交 (字符串的值或 None（带插值、认不出）, 收尾之后的下标)；没收尾交 (None, len(text))。单引号里只认 \\\\ 与 \\'；双引号与反引号里的反斜杠转义照收，
    值里有 interpolating_marks 里的哪一种（perl 的 $ 与 @、ruby 的 #{、js 的 ${）就交 None。"""
    quote = text[start]
    index, value, interpolated = start + 1, [], False
    while index < len(text):
        character = text[index]
        if character == "\\" and index + 1 < len(text):
            following = text[index + 1]
            if quote == "'" and following not in "\\'":
                value.append(character)
            else:
                value.append({"n": "\n", "t": "\t"}.get(following, following) if quote != "'" else following)
            index += 2
            continue
        if character == quote:
            return (None if interpolated else "".join(value)), index + 1
        if quote != "'" and any(text.startswith(mark, index) for mark in interpolating_marks):
            interpolated = True
        value.append(character)
        index += 1
    return None, len(text)


def literal_argument_list(text, start, interpolating_marks, extra_quotes=""):
    """从 text[start] 起读一串用逗号隔开的字面字符串参数（外面可以套一层圆括号，参数也可以写成一个 [ … ] 数组）：交 (值的列表或 None（有不是字面的）, 读到的终点)。
    读到 `)`、`;`、换行、`}` 或字符串之外的别的写法就停；一个都没读到交 None。"""
    index = start
    while index < len(text) and text[index] in " \t":
        index += 1
    parenthesized = index < len(text) and text[index] == "("
    if parenthesized:
        index += 1
    values, bracketed = [], False
    while index < len(text):
        while index < len(text) and text[index] in " \t\n":
            index += 1
        if index >= len(text):
            break
        character = text[index]
        if character == "[" and not bracketed:
            bracketed = True   # 数组参数（`spawn("cargo", ["test", "--all"])`、`IO.popen(["cargo", …])`）的元素接在前面的参数后面
            index += 1
            continue
        if character in "'\"" + extra_quotes:
            value, index = literal_quoted_string(text, index, interpolating_marks)
            if value is None:
                return None, index
            values.append(value)
            while index < len(text) and text[index] in " \t":
                index += 1
            if index < len(text) and text[index] == ",":
                index += 1
                continue
            if index < len(text) and text[index] == "]" and bracketed:
                bracketed = False
                index += 1
                continue
            break
        if character in ");}\n" or (character == "]" and bracketed):
            break
        return None, index
    return (values or None), index


def launched_command_text(values):
    """一串字面参数：只有一个的是交给 shell 的命令串；几个的是逐词交的 argv，照 shell 引号连成一条命令。"""
    return values[0] if len(values) == 1 else shlex.join(values)


def string_spans(text, quotes, comment_start=None):
    """text 里引号括起来的区间 [(起, 止), …]（转义照认），comment_start 起到行尾的注释也算；扫语言代码时跳过这些区间里的字。"""
    spans, index = [], 0
    while index < len(text):
        character = text[index]
        if comment_start and character == comment_start and (index == 0 or text[index - 1] in " \t\n;{}("):
            end = text.find("\n", index)
            end = len(text) if end < 0 else end
            spans.append((index, end))
            index = end
            continue
        if character in quotes:
            _, end = literal_quoted_string(text, index, ())
            spans.append((index, end))
            index = end
            continue
        index += 1
    return spans


def inside_spans(position, spans):
    return any(start <= position < end for start, end in spans)


def scan_quoted_language(language, text, pattern, interpolating_marks, quotes, comment_start, texts, unjudged):
    """perl / ruby / php / node / lua 这类：代码里起命令的写法（pattern 认，引号里、注释里的不算）后面跟的字面参数照命令交出，不是字面的交说明。"""
    spans = string_spans(text, quotes, comment_start)
    for match in pattern.finditer(text):
        if inside_spans(match.start(), spans):
            continue
        word = match.group(0).rstrip("( \t")
        call_start = match.end() - 1 if match.group(0).endswith("(") else match.end()
        rest = match.end()
        while rest < len(text) and text[rest] in " \t":
            rest += 1
        following = text[rest] if rest < len(text) else ""
        if word in ("qx", "%x") or word.endswith("%x"):
            closing = {"{": "}", "(": ")", "[": "]", "<": ">"}.get(following, following)
            end = text.find(closing, rest + 1) if following else -1
            body = text[rest + 1:end] if end > rest else None
            if body is None or (following != "'" and any(mark in body for mark in interpolating_marks)):
                unjudged.append(f"{language} 代码里的 {word}{following}…{closing} 认不出起的是什么")
            else:
                texts.append(body)
            continue
        if not match.group(0).endswith("(") and following and following not in "(\"'`[" and not (language == "perl" and following in "$@{"):
            continue   # `print if /exec/`、`system =>` 这类不是调用
        if word == "open":
            values, _ = literal_argument_list(text, call_start, interpolating_marks)
            if values is None:
                arguments_text = text[call_start:call_start + 200]
                if "|" in arguments_text.split(";")[0]:
                    unjudged.append(f"{language} 代码里 open 的管道命令认不出")
                continue
            pipe_values = [value for value in values if value.startswith("|") or value.endswith("|")]
            if pipe_values:
                texts.append(pipe_values[0].strip("|").strip())
            elif any(value in ("-|", "|-") for value in values):
                rest_values = values[[value in ("-|", "|-") for value in values].index(True) + 1:]
                if rest_values:
                    texts.append(launched_command_text(rest_values))
                else:
                    unjudged.append(f"{language} 代码里 open 的管道命令认不出")
            continue
        values, _ = literal_argument_list(text, call_start, interpolating_marks, "`" if language == "node" else "")
        if values is None:
            unjudged.append(f"{language} 代码里的 {word}(…) 起的命令不是字面的，认不出起的是什么")
        else:
            texts.append(launched_command_text(values))
    if language in ("perl", "ruby", "php"):
        for start, _ in spans:
            if text[start] != "`":
                continue
            value, _ = literal_quoted_string(text, start, interpolating_marks)
            if value is None:
                unjudged.append(f"{language} 代码里反引号起的命令不是字面的，认不出起的是什么")
            else:
                texts.append(value)


def python_literal(node):
    """ast 节点是字面的字符串、或字面字符串组成的列表 / 元组时交它的值，别的交 None。"""
    import ast
    try:
        value = ast.literal_eval(node)
    except (ValueError, TypeError, SyntaxError, MemoryError, RecursionError):
        return None
    if isinstance(value, str):
        return value
    if isinstance(value, (list, tuple)) and value and all(isinstance(item, str) for item in value):
        return list(value)
    return None


class PythonUnresolvedElement:
    """argv 列表里不是字面的一项（python_command_value 交的列表里的占位）：照一段 shell 命令判时换成 PYTHON_UNRESOLVED_ARGUMENT。"""


PYTHON_UNRESOLVED_ELEMENT = PythonUnresolvedElement()


class PythonUnresolvedTail:
    """argv 列表里 `+ 名字`、`*名字` 接上的、算不出的一段（几项都可能；python_list_tail）：照一段 shell 命令判时同样换成 PYTHON_UNRESOLVED_ARGUMENT，
    命令词要过 python_tail_command_word_judged 才交出。"""


PYTHON_UNRESOLVED_TAIL = PythonUnresolvedTail()
# argv 后面接了算不出的一段时，命令词是这些的照认不出（参数里能带要跑的命令）：启动器（lib_project_code_runs.py 的 LAUNCHER_OPTIONS_WITH_VALUE 与 EXTRA_LAUNCHER_NAMES，
# bash-command-detector.sh 的自证核两边一致）与参数里带命令的程序；shell、解释器、前缀与包装另按各自的名单认（python_tail_command_word_judged）
ARGUMENT_COMMAND_LAUNCHER_NAMES = frozenset({"time", "flock", "chrt", "prlimit", "systemd-run", "strace", "perf", "rustup",
                                             "bwrap", "cgexec", "chroot", "cpulimit", "dbus-run-session", "doas", "faketime", "find", "firejail", "gdb", "ltrace",
                                             "nsenter", "numactl", "parallel", "proot", "runuser", "scl", "script", "setpriv", "sg", "ssh", "su", "unbuffer",
                                             "unshare", "valgrind", "watch", "xargs"})
ARGUMENT_COMMAND_PROGRAM_NAMES = frozenset({"sed", "git", "tar", "less", "busybox", "eval", "source", ".", "builtin"})
SCRIPT_FILE_SUFFIXES = (".sh", ".bash", ".py", ".pl", ".rb")
PYTHON_UNRESOLVED_ARGUMENT = '"$(unresolved-python-argument)"'   # argv 里不是字面的一项换成的词：恰好一个词、值算不出（落在命令词上、cargo 的参数上照认不出办）
PYTHON_READ_ONLY_BUILTINS = frozenset({"print", "len", "str", "repr", "list", "tuple", "sorted"})   # 把列表交给它们不会改列表
PYTHON_LIST_MUTATING_METHODS = frozenset({"append", "extend", "insert", "pop", "remove", "clear", "sort", "reverse", "__setitem__", "__delitem__", "__iadd__"})


def python_argument_variables_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-argument-variables-unresolved：python 里起命令的参数只认整个字面的（argv 里有一项是变量就整条认不出、赋过字面值的变量与
    字面串的 .split() 不认）。"""
    return words_break_is_set("python-argument-variables-unresolved")


def python_constant_bindings(tree, launch_call):
    """python 代码里赋过字面值、之后没变过的名字：交 {名字: 赋给它的值的 ast 节点}。整段代码里只绑定过一次（赋值、for / with / except / import / 函数参数 / 函数与类名 /
    global / match 的捕获都算绑定，作用域不分）、那一次是 `名字 = 值`（或带注解、海象）的才收；值是可变的（列表、.split() 的结果）的另要求没被改过：
    `名字[…] = …`、`del 名字[…]`、`名字.append(…)` 这类，或当参数交给 launch_call 认的起命令调用与 print / len / str.join 这类之外的函数，都算改过。"""
    import ast
    bindings = {}
    def bind(name, times=1):
        bindings[name] = bindings.get(name, 0) + times
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            bind(node.id)
        elif isinstance(node, ast.arg):
            bind(node.arg)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                bind((alias.asname or alias.name).split(".")[0])
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            for name in node.names:
                bind(name, 2)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            bind(node.name)
        elif isinstance(node, ast.ExceptHandler) and node.name:
            bind(node.name)
        elif isinstance(node, (ast.MatchAs, ast.MatchStar)) and node.name:
            bind(node.name)
        elif isinstance(node, ast.MatchMapping) and node.rest:
            bind(node.rest)
    values = {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            values[node.targets[0].id] = node.value
        elif isinstance(node, (ast.AnnAssign, ast.NamedExpr)) and isinstance(node.target, ast.Name) and node.value is not None:
            values[node.target.id] = node.value
    constants = {name: value for name, value in values.items() if bindings.get(name) == 1}
    for node in ast.walk(tree):
        if isinstance(node, (ast.Subscript, ast.Attribute)) and isinstance(node.value, ast.Name):
            if isinstance(node.ctx, (ast.Store, ast.Del)) or (isinstance(node, ast.Attribute) and node.attr in PYTHON_LIST_MUTATING_METHODS):
                constants.pop(node.value.id, None)
        elif isinstance(node, ast.Call) and not launch_call(node):
            if ((isinstance(node.func, ast.Name) and node.func.id in PYTHON_READ_ONLY_BUILTINS and node.func.id not in bindings)
                    or (isinstance(node.func, ast.Attribute) and node.func.attr == "join")):
                continue
            for argument in list(node.args) + [keyword.value for keyword in node.keywords]:
                passed = argument.value if isinstance(argument, ast.Starred) else argument
                if isinstance(passed, ast.Name) and passed.id in constants and not isinstance(constants[passed.id], (ast.Constant, ast.Tuple)):
                    constants.pop(passed.id)
    return constants


def python_list_concatenation_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-list-concatenation-unresolved：argv 的 `+ 列表` 与 `*列表` 整条认不出。"""
    return words_break_is_set("python-list-concatenation-unresolved")


def python_list_tail(node, constants, modules, depth):
    """argv 列表里 `*名字` 与 `字面列表 + 名字` 接上的那一段：算得出是列表的照接上，算不出的换成一个 PYTHON_UNRESOLVED_TAIL（几项都可能）。"""
    value = python_command_value(node, constants, modules, depth)
    return list(value) if isinstance(value, list) else [PYTHON_UNRESOLVED_TAIL]


def python_tail_command_word_judged(word):
    """argv 后面接了算不出的一段（PYTHON_UNRESOLVED_TAIL）时，命令词能不能单独照命令判：字面的、不是 shell、解释器（python / perl / ruby / node / php / lua / tcl /
    awk / make / vim 与 ARGUMENT_COMMAND_PROGRAM_NAMES）、前缀与包装（PREFIX_OPTIONS_WITH_VALUE、WRAPPERS_TAKING_ONE_ARGUMENT）、启动器
    （ARGUMENT_COMMAND_LAUNCHER_NAMES），也不是脚本（SCRIPT_FILE_SUFFIXES 结尾）的才算；别的照认不出办（复跑分类报告 d137-replay-classify 二节 E6）。"""
    if not isinstance(word, str) or not word or set("$`") & set(word):
        return False
    name = os.path.basename(word)
    return not (is_shell_name(name) or PYTHON_NAME.match(name) or PERL_NAME.match(name) or RUBY_NAME.match(name) or PHP_NAME.match(name)
                or LUA_NAME.match(name) or TCL_NAME.match(name) or name in NODE_NAMES or name in AWK_NAMES or name in MAKE_NAMES or name in VIM_NAMES
                or name in ARGUMENT_COMMAND_PROGRAM_NAMES or name in PREFIX_OPTIONS_WITH_VALUE or name in WRAPPERS_TAKING_ONE_ARGUMENT
                or name in ARGUMENT_COMMAND_LAUNCHER_NAMES or name.endswith(SCRIPT_FILE_SUFFIXES))


def python_command_value(node, constants, modules, depth=0):
    """起命令的参数的值：字面的 str 交 str；列表 / 元组交 list，里面不是字面 str 的项换成 PYTHON_UNRESOLVED_ELEMENT；
    赋过字面值的名字（python_constant_bindings）、字面串的 `.split()` 与 `shlex.split(字面串)` 照算出的值；
    头一项不是展开的列表里的 `*名字` 与 `列表 + 名字` 照 python_list_tail 接上（复跑分类报告 d137-replay-classify 二节 E6；弄坏开关 python-list-concatenation-unresolved
    下整条认不出）；别的交 None（认不出）。"""
    import ast
    if node is None or depth > EMBEDDED_NESTING:
        return None
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.Name) and node.id in constants:
        return python_command_value(constants[node.id], constants, modules, depth + 1)
    if isinstance(node, (ast.List, ast.Tuple)):
        if not node.elts or isinstance(node.elts[0], ast.Starred) or (
                python_list_concatenation_unresolved() and any(isinstance(element, ast.Starred) for element in node.elts)):
            return None
        values = []
        for element in node.elts:
            if isinstance(element, ast.Starred):
                values += python_list_tail(element.value, constants, modules, depth + 1)
                continue
            value = python_command_value(element, constants, modules, depth + 1)
            values.append(value if isinstance(value, str) else PYTHON_UNRESOLVED_ELEMENT)
        return values
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add) and not python_list_concatenation_unresolved():
        head = python_command_value(node.left, constants, modules, depth + 1)
        if not isinstance(head, list) or not head or not isinstance(head[0], str):
            return None
        return head + python_list_tail(node.right, constants, modules, depth + 1)
    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "split" and not node.keywords and len(node.args) <= 1:
        if isinstance(node.func.value, ast.Name) and modules.get(node.func.value.id) == "shlex":
            text = python_command_value(node.args[0], constants, modules, depth + 1) if node.args else None
            if not isinstance(text, str):
                return None
            try:
                return shlex.split(text)
            except ValueError:
                return None
        text = python_command_value(node.func.value, constants, modules, depth + 1)
        separator = python_command_value(node.args[0], constants, modules, depth + 1) if node.args else None
        if isinstance(text, str) and (not node.args or (isinstance(separator, str) and separator)):
            return text.split(separator) if node.args else text.split()
    return None


def python_argv_text(argv):
    """argv（python_command_value 的列表）照一段 shell 命令写出：字面的项照 shlex 加引号，不是字面的项换成 PYTHON_UNRESOLVED_ARGUMENT。"""
    return " ".join(PYTHON_UNRESOLVED_ARGUMENT if element is PYTHON_UNRESOLVED_ELEMENT or element is PYTHON_UNRESOLVED_TAIL else shlex.quote(element)
                    for element in argv)


def scan_python(text, texts, unjudged, depth=0):
    """python 代码（-c 的代码串、喂进标准输入的）里起命令的调用：os.system / os.popen / subprocess（shell=True 的照命令串）/ os.exec* / os.spawn* / pty.spawn 这类，
    命令串、argv 照 python_command_value 取值（字面的、赋过字面值没变过的名字、字面串的 .split()）：命令串要整个算得出，argv 里算不出的参数照一个值算不出的词交出
    （落在命令词上调用方照认不出办）；整个认不出的、经 __import__ / importlib / getattr / exec / eval 动态取的、用了 ctypes 的交说明。语法错的代码 python 自己不跑，带起命令字样的照认不出办。"""
    import ast
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError, MemoryError, RecursionError):
        if any(hint in text for hint in EMBEDDED_PYTHON_SYNTAX_HINTS):
            unjudged.append("python 代码解析不了、里面带起命令的字样，认不出起的是什么")
        return
    modules, functions = {}, {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                top = alias.name.split(".")[0]
                modules[alias.asname or top] = alias.name if alias.asname else top
                if top in EMBEDDED_PYTHON_OPAQUE_MODULES:
                    unjudged.append(f"python 代码用了 {top}，能直接调 libc 起命令，认不出起的是什么")
        elif isinstance(node, ast.ImportFrom) and node.module:
            top = node.module.split(".")[0]
            if top in EMBEDDED_PYTHON_OPAQUE_MODULES:
                unjudged.append(f"python 代码用了 {top}，能直接调 libc 起命令，认不出起的是什么")
            for alias in node.names:
                if alias.name == "*":
                    for name in EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES | EMBEDDED_PYTHON_SUBPROCESS_CALLS | {"spawn"}:
                        functions.setdefault(name, (top, name))
                else:
                    functions[alias.asname or alias.name] = (top, alias.name)
    def module_of(value):
        if isinstance(value, ast.Name):
            return modules.get(value.id)
        if (isinstance(value, ast.Call) and value.args and isinstance(value.args[0], ast.Constant) and isinstance(value.args[0].value, str)
                and ((isinstance(value.func, ast.Name) and value.func.id == "__import__")
                     or (isinstance(value.func, ast.Attribute) and value.func.attr == "import_module"))):
            return value.args[0].value.split(".")[0]
        return None
    def launch_call(call):
        if isinstance(call.func, ast.Attribute):
            return module_of(call.func.value) in EMBEDDED_PYTHON_LAUNCH_MODULES
        return isinstance(call.func, ast.Name) and call.func.id in functions and functions[call.func.id][0] in EMBEDDED_PYTHON_LAUNCH_MODULES
    literal_only = python_argument_variables_unresolved()
    constants = {} if literal_only else python_constant_bindings(tree, launch_call)
    def value_of(node):
        return python_literal(node) if literal_only else python_command_value(node, constants, modules)
    def argv_elements(values):
        if literal_only:
            return values if values and all(isinstance(value, str) for value in values) else None
        return [value if isinstance(value, str) else PYTHON_UNRESOLVED_ELEMENT for value in values] or None
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        target = None
        if isinstance(node.func, ast.Attribute):
            owner = module_of(node.func.value)
            if owner is not None:
                target = (owner, node.func.attr)
            elif node.func.attr in EMBEDDED_PYTHON_DISTINCT_LAUNCH_NAMES:
                unjudged.append(f"python 代码里的 ….{node.func.attr}(…) 认不出是哪个模块的、起的是什么")
                continue
        elif isinstance(node.func, ast.Name):
            name = node.func.id
            if name in functions:
                target = functions[name]
            elif name in ("exec", "eval", "compile"):
                code = python_literal(node.args[0]) if node.args else None
                if isinstance(code, str) and depth < EMBEDDED_NESTING:
                    scan_python(code, texts, unjudged, depth + 1)
                elif node.args and (name != "compile" or isinstance(code, str) or words_break_is_set("python-compile-as-execution")):
                    # 只调 compile、代码不是字面的，不当起命令（复跑分类报告 d137-replay-classify 二节 E7：compile 只编不跑，编出来的交给 exec / eval 时由那一处照判）；
                    # 字面的代码照旧扫。弄坏开关 LIB_SHELL_WORDS_BREAK=python-compile-as-execution 下 compile 照执行算
                    unjudged.append(f"python 代码里 {name}(…) 的代码不是字面的，认不出起的是什么")
                continue
            elif name in ("__import__", "getattr") and node.args and not (name == "__import__" and python_literal(node.args[0]) is not None):
                if name == "__import__" or module_of(node.args[0]) in EMBEDDED_PYTHON_LAUNCH_MODULES:
                    unjudged.append(f"python 代码里经 {name}(…) 动态取模块或函数，认不出起的是什么")
                continue
        if target is None:
            continue
        module, function = target
        if module == "importlib" and function == "import_module" and node.args and python_literal(node.args[0]) is None:
            unjudged.append("python 代码里经 importlib.import_module 动态取模块，认不出起的是什么")
            continue
        if module not in EMBEDDED_PYTHON_LAUNCH_MODULES:
            continue
        positional = node.args
        if any(isinstance(argument, ast.Starred) for argument in positional):
            unjudged.append(f"python 代码里 {module}.{function}(*…) 的参数是展开的，认不出起的是什么")
            continue
        keywords = {keyword.arg: keyword.value for keyword in node.keywords}
        if (module, function) in EMBEDDED_PYTHON_SHELL_CALLS:
            command = value_of(positional[0]) if positional else value_of(keywords.get("cmd")) if "cmd" in keywords else None
            if isinstance(command, str):
                texts.append(command)
            else:
                unjudged.append(f"python 代码里 {module}.{function}(…) 的命令不是字面的，认不出起的是什么")
            continue
        argv = None
        if module == "subprocess" and function in EMBEDDED_PYTHON_SUBPROCESS_CALLS:
            shell = keywords.get("shell")
            arguments = value_of(positional[0]) if positional else value_of(keywords["args"]) if "args" in keywords else None
            shell_value = None if shell is None else python_literal(shell) if not isinstance(shell, ast.Constant) else shell.value
            if shell is not None and not isinstance(shell_value, bool):
                unjudged.append(f"python 代码里 subprocess.{function}(…) 的 shell= 不是字面的，认不出起的是什么")
                continue
            if arguments is None:
                unjudged.append(f"python 代码里 subprocess.{function}(…) 的命令不是字面的，认不出起的是什么")
                continue
            if shell_value:
                # shell=True：命令串（列表的头一项）整个交给 sh -c，里面有一处算不出就认不出
                command_text = arguments if isinstance(arguments, str) else arguments[0]
                if isinstance(command_text, str):
                    texts.append(command_text)
                else:
                    unjudged.append(f"python 代码里 subprocess.{function}(…, shell=True) 的命令串不是字面的，认不出起的是什么")
                continue
            argv = [arguments] if isinstance(arguments, str) else arguments
        elif module in ("os", "posix") and function in EMBEDDED_PYTHON_EXEC_LIST_CALLS | EMBEDDED_PYTHON_SPAWN_LIST_CALLS:
            offset = 1 if function.startswith("spawn") else 0
            values = [value_of(argument) for argument in positional[offset:]]
            if function.endswith("e"):
                values = values[:-1]
            values = argv_elements(values)
            if values:
                argv = [values[0]] + values[2:]
        elif module in ("os", "posix") and function in EMBEDDED_PYTHON_EXEC_VECTOR_CALLS | EMBEDDED_PYTHON_SPAWN_VECTOR_CALLS | {"posix_spawn", "posix_spawnp"}:
            offset = 1 if function.startswith("spawn") else 0
            program = value_of(positional[offset]) if len(positional) > offset else None
            vector = value_of(positional[offset + 1]) if len(positional) > offset + 1 else None
            if isinstance(vector, list) and len(positional) > offset and (isinstance(program, str) or not literal_only):
                argv = [program if isinstance(program, str) else PYTHON_UNRESOLVED_ELEMENT] + vector[1:]
        elif module == "pty" and function == "spawn":
            value = value_of(positional[0]) if positional else None
            argv = [value] if isinstance(value, str) else value
        elif module == "asyncio" and function == "create_subprocess_exec":
            argv = argv_elements([value_of(argument) for argument in positional])
        else:
            continue
        if argv and any(element is PYTHON_UNRESOLVED_TAIL for element in argv) and not python_tail_command_word_judged(argv[0]):
            unjudged.append(f"python 代码里 {module}.{function}(…) 的命令词是 shell、解释器、启动器或脚本（或不是字面的），后面接上的参数算不出，认不出起的是什么")
        elif argv:
            texts.append(python_argv_text(argv))
        else:
            unjudged.append(f"python 代码里 {module}.{function}(…) 起的命令不是字面的，认不出起的是什么")


def tcl_words(command_text):
    """一条 tcl 命令切成词：{…} 照字面（可以套）；"…" 与光词里有 $ 或 [ … ] 的不是字面（那个词交 None）；交 (词表, [命令替换 [ … ] 里的代码…])。"""
    words, substitutions, index = [], [], 0
    while index < len(command_text):
        while index < len(command_text) and command_text[index] in " \t":
            index += 1
        if index >= len(command_text):
            break
        if command_text[index] == "{":
            level, end = 0, index
            while end < len(command_text):
                if command_text[end] == "\\":
                    end += 2
                    continue
                level += {"{": 1, "}": -1}.get(command_text[end], 0)
                if level == 0:
                    break
                end += 1
            words.append(command_text[index + 1:end])
            index = end + 1
            continue
        quoted = command_text[index] == '"'
        end = index + 1 if quoted else index
        value, literal = [], True
        while end < len(command_text):
            current = command_text[end]
            if not quoted and current in " \t":
                break
            if quoted and current == '"':
                end += 1
                break
            if current == "\\" and end + 1 < len(command_text):
                value.append(command_text[end + 1])
                end += 2
                continue
            if current == "$":
                literal = False
            if current == "[":
                level, inner = 0, end
                while inner < len(command_text):
                    level += {"[": 1, "]": -1}.get(command_text[inner], 0)
                    if level == 0:
                        break
                    inner += 1
                substitutions.append(command_text[end + 1:inner])
                literal = False
                end = inner + 1
                continue
            value.append(current)
            end += 1
        words.append("".join(value) if literal else None)
        index = end
    return words, substitutions


def scan_tcl(text, texts, unjudged, depth=0):
    """tcl（tclsh、wish、expect）代码：exec / spawn / system 这几条命令的词都是字面的照 argv 交出（exec 的 -ignorestderr、-keepnewline、-- 跳过，`|` `<` `>` `&` 起的重定向与后台之后不要），
    open 的 `|命令` 照命令交；词里有变量或命令替换的、eval / uplevel / subst / interp / source 动态起代码的交说明。命令替换 [ … ] 里的代码接着判。"""
    for line in re.split(r"[\n;]", text):
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        words, substitutions = tcl_words(stripped)
        if depth < EMBEDDED_NESTING:
            for substitution in substitutions:
                scan_tcl(substitution, texts, unjudged, depth + 1)
        if not words or words[0] is None:
            continue
        name = words[0]
        if name in TCL_OPAQUE_COMMANDS:
            unjudged.append(f"tcl 代码里的 {name} 动态起代码，认不出起的是什么")
            continue
        if name == "open" and len(words) > 1:
            if words[1] is None:
                unjudged.append("tcl 代码里 open 的目标不是字面的，认不出是不是起命令")
            elif words[1].startswith("|"):
                texts.append(words[1][1:].strip())
            continue
        if name not in TCL_LAUNCH_COMMANDS:
            continue
        arguments = words[1:]
        while arguments and arguments[0] in ("-ignorestderr", "-keepnewline", "-noecho", "-nottycopy", "-nottyinit", "-console", "-pty", "-open", "-leaveopen"):
            arguments = arguments[1:]
        if arguments and arguments[0] == "--":
            arguments = arguments[1:]
        kept = []
        for argument in arguments:
            if argument is not None and (argument in ("|", "&") or argument.startswith(("<", ">", "2>", "|"))):
                break
            kept.append(argument)
        if not kept or any(argument is None for argument in kept):
            unjudged.append(f"tcl 代码里的 {name} 起的命令不是字面的，认不出起的是什么")
        elif name in ("system", "exp_system"):
            texts.append(" ".join(kept))
        else:
            texts.append(shlex.join(kept))


def awk_tokens(text):
    """awk 程序切成 (种类, 值)：string（双引号串的值）、regex、name、number、punct；# 注释与换行之外的空白去掉。斜杠在值或右括号之后是除号，别处是正则。"""
    tokens, index = [], 0
    while index < len(text):
        character = text[index]
        if character in " \t\r":
            index += 1
            continue
        if character == "\\" and text[index + 1:index + 2] == "\n":
            index += 2
            continue
        if character == "#":
            end = text.find("\n", index)
            index = len(text) if end < 0 else end
            continue
        if character == '"':
            value, end = literal_quoted_string(text, index, ())
            tokens.append(("string", value if value is not None else ""))
            index = end
            continue
        if character == "/" and (not tokens or tokens[-1][0] == "punct" and tokens[-1][1] not in (")", "]")
                                 or tokens[-1][0] == "name" and tokens[-1][1] in ("print", "printf", "return", "in", "getline")):
            end = index + 1
            while end < len(text) and text[end] != "/" and text[end] != "\n":
                end += 2 if text[end] == "\\" else 1
            tokens.append(("regex", text[index + 1:end]))
            index = end + 1
            continue
        if character.isalpha() or character == "_":
            end = index
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            tokens.append(("name", text[index:end]))
            index = end
            continue
        if character.isdigit():
            end = index
            while end < len(text) and (text[end].isalnum() or text[end] == "."):
                end += 1
            tokens.append(("number", text[index:end]))
            index = end
            continue
        for operator in ("|&", "||", "&&", "==", "!=", "<=", ">=", ">>", "++", "--", "+=", "-=", "*=", "/=", "%=", "^=", "**"):
            if text.startswith(operator, index):
                tokens.append(("punct", operator))
                index += len(operator)
                break
        else:
            tokens.append(("punct", character))
            index += 1
    return tokens


AWK_ASSIGNMENT_ENDS = (";", "}", "\n")
AWK_CHANGING_OPERATORS = ("+=", "-=", "*=", "/=", "%=", "^=", "++", "--", "[")
AWK_TARGET_BUILTINS = frozenset({"sub", "gsub", "split", "patsplit", "match"})   # 参数里的变量会被它们改写（替换的目标、分出来的数组）


def awk_command_variables_unresolved():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=awk-command-variables-unresolved：awk 里当命令用的变量一律认不出（`cmd = "date"; cmd | getline`、命令行 `cmd=sort` 再
    `print | cmd` 都照写成变量拒）。"""
    return words_break_is_set("awk-command-variables-unresolved")


def awk_command_variables(tokens, assignments=()):
    """awk 程序里变量能取到的字面命令串：交 {名字: [值, …]}。程序里每一处赋值都是 `名字 = "字面串"`（后面接 ; } 换行或程序末尾）、命令行给的（-v 与操作数里的 `名字=值`，
    assignments 里 (名字, 值)，值是 None 的是写成了 shell 变量或命令替换）都是字面的，而别处没有改写它的写法（拼接赋值、+= 这类、++ / --、getline 名字、数组、for … in、
    sub / gsub / split / patsplit / match 的参数、函数参数）才收；收不了的名字不在里面。"""
    literal, broken = {}, set()
    for name, value in assignments:
        if value is None:
            broken.add(name)
        else:
            literal.setdefault(name, []).append(value)
    target_depths, depth = [], 0
    for position, (kind, value) in enumerate(tokens):
        before = tokens[position - 1] if position > 0 else None
        after = tokens[position + 1] if position + 1 < len(tokens) else None
        if (kind, value) == ("punct", "("):
            depth += 1
            if before is not None and before[0] == "name" and before[1] in AWK_TARGET_BUILTINS:
                target_depths.append(depth)
            continue
        if (kind, value) == ("punct", ")"):
            if target_depths and target_depths[-1] == depth:
                target_depths.pop()
            depth -= 1
            continue
        if kind != "name":
            continue
        if target_depths:
            broken.add(value)
        if before is not None and before[0] == "name" and before[1] in ("function", "func") and after == ("punct", "("):
            end = position + 2
            while end < len(tokens) and tokens[end] != ("punct", ")"):
                if tokens[end][0] == "name":
                    broken.add(tokens[end][1])
                end += 1
        if before == ("punct", "$"):
            continue   # $名字 是字段，不是这个变量本身
        if after == ("punct", "="):
            assigned = tokens[position + 2] if position + 2 < len(tokens) else None
            ending = tokens[position + 3] if position + 3 < len(tokens) else None
            if assigned is not None and assigned[0] == "string" and (ending is None or (ending[0] == "punct" and ending[1] in AWK_ASSIGNMENT_ENDS)):
                literal.setdefault(value, []).append(assigned[1])
            else:
                broken.add(value)
        elif ((after is not None and after[0] == "punct" and after[1] in AWK_CHANGING_OPERATORS) or after == ("name", "in")
              or before in (("punct", "++"), ("punct", "--"), ("name", "getline"))):
            broken.add(value)
    return {name: values for name, values in literal.items() if name not in broken}


def scan_awk(text, texts, unjudged, assignments=()):
    """awk 程序：system(…)、print … | …、… | getline、|& 协进程里的命令是字面字符串、或是赋过字面串而没改写过的变量（awk_command_variables；assignments 是命令行给的
    `名字=值`）的照命令交出（变量取到的每个值都交）；写成别的变量、拼接的交说明。"""
    tokens = awk_tokens(text)
    variables = {} if awk_command_variables_unresolved() else awk_command_variables(tokens, assignments)
    def command_values(token):
        if token is not None and token[0] == "string":
            return [token[1]]
        if token is not None and token[0] == "name" and token[1] in variables:
            return variables[token[1]]
        return None
    for position, (kind, value) in enumerate(tokens):
        following = tokens[position + 1:position + 4]
        if kind == "name" and value == "system" and following[:1] == [("punct", "(")]:
            values = command_values(following[1]) if len(following) >= 3 and following[2] == ("punct", ")") else None
            if values is not None:
                texts.extend(values)
            else:
                unjudged.append("awk 程序里 system(…) 的命令不是字面字符串，认不出起的是什么")
        elif kind == "punct" and value in ("|", "|&"):
            if following[:1] == [("name", "getline")]:
                previous = tokens[position - 1] if position > 0 else None
                before = tokens[position - 2] if position > 1 else None
                values = command_values(previous)
                if values is not None and (before is None or before[0] == "punct" and before[1] not in (")", "$")):
                    texts.extend(values)
                else:
                    unjudged.append("awk 程序里 … | getline 的命令不是字面字符串，认不出起的是什么")
            elif (following[:1] and command_values(following[0]) is not None
                  and (len(following) < 2 or following[1][0] == "punct" and following[1][1] in (";", "}", ")", "\n"))):
                texts.extend(command_values(following[0]))
            else:
                unjudged.append("awk 程序里管道 | 到的命令不是字面字符串，认不出起的是什么")


SED_SUBSTITUTION_ADDRESS = "\x01"   # sed 脚本词里的命令替换（`$(…)`、反引号）与变量（`${名字}`、`$名字`）展开之后换成的占位：只认在行号地址位上（sed_script_parse）
SED_ARITHMETIC_VALUE = "1"          # 算术展开（`$((…))`、`$[…]`）换成的值：结果只能是整数，哪个整数都不改 sed 脚本的切法（0 与负数让 sed 报错、什么都不跑）
SED_EXPANSION_LIMIT = 6             # 一份脚本里没展开的展开超过这么多处不逐个组合（2 的这么多次方趟），照「解析不下去」办
GNU_SED_COMMAND_CHARACTERS = frozenset("{}=:#!abcdDeFgGhHilLnNpPqQrRstTvwWxyz")   # GNU sed 认得的命令字：解析卡在这些字上是这里没读懂，不是 sed 一定报错
# GNU sed 的 s 命令标志区认得的字（compile.c 的 mark_subst_opts；空白实测 GNU sed 4.9 照认）：别的字编译就报 unknown option to `s'，整个脚本一行都不跑
SED_SUBSTITUTE_FLAG_CHARACTERS = frozenset("gpeiImM0123456789 \t\r")
SED_VARIABLE = re.compile(r"\$(?:\{[A-Za-z_][A-Za-z0-9_]*\}|[A-Za-z_][A-Za-z0-9_]*|[0-9#?$!*@-])")


def sed_expansions_unparsed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed：sed 脚本词里没展开的 bash 展开不另算，原样切、切不下去照「解析不下去」。"""
    return words_break_is_set("sed-expansions-unparsed")


def sed_text_first_line_only():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=sed-text-first-line-only：a / i / c 的文本只读到头一个换行（`a\\` 换行之后的文本行被当命令读）。"""
    return words_break_is_set("sed-text-first-line-only")


class SedParse(NamedTuple):
    texts: list              # e 命令后面的字面命令
    unjudged: list           # 起命令而认不出的说明（单独的 e、s///e、解析不下去）
    failed_at: str | None    # 解析不下去时卡在的那个字（脚本读完了还缺东西是 ""）；切完了是 None
    misplaced: bool          # 占位 SED_SUBSTITUTION_ADDRESS 落在了行号地址位之外（正则地址、文本、s 的正文这类）


def sed_script_expansions(script):
    """sed 脚本词里没展开的 bash 展开（切词时引号已去掉、分不出原来在单引号里还是双引号里）：交 [(起, 止, 种类)]，种类 "arithmetic"（`$((…))`、`$[…]`）、
    "substitution"（`$(…)`、反引号）、"variable"（`${名字}`、`$名字`、`$1` 这类）；有认不出的（没配上的、算术里套命令替换、`${名字:-…}` 这类写法）交 None。"""
    found, index = [], 0
    while index < len(script):
        if script.startswith("$((", index):
            end = command_substitution_end(script, index + 2)
            if end < 0 or "$(" in script[index + 3:end] or "`" in script[index + 3:end]:
                return None
            found.append((index, end, "arithmetic"))
            index = end
        elif script.startswith("$[", index):
            depth, end = 0, index + 1
            while end < len(script):
                depth += 1 if script[end] == "[" else -1 if script[end] == "]" else 0
                end += 1
                if depth == 0:
                    break
            if depth != 0 or "$(" in script[index + 2:end] or "`" in script[index + 2:end]:
                return None
            found.append((index, end, "arithmetic"))
            index = end
        elif script.startswith("$(", index) or script[index] == "`":
            end = command_substitution_end(script, index + 2) if script[index] == "$" else backquote_end(script, index + 1)
            if end < 0:
                return None
            found.append((index, end, "substitution"))
            index = end
        elif script.startswith("${", index):
            variable = SED_VARIABLE.match(script, index)
            if variable is None:
                return None
            found.append((index, variable.end(), "variable"))
            index = variable.end()
        elif script[index] == "$" and SED_VARIABLE.match(script, index):
            end = SED_VARIABLE.match(script, index).end()
            found.append((index, end, "variable"))
            index = end
        else:
            index += 1
    return found


def sed_expanded_variants(script):
    """sed 脚本词里有没展开的 bash 展开、原样切不下去时（sed_script_commands）：每一处按「照字面留着」「照 bash 展开」两种各切一遍（单引号里的是字面、双引号里的展开，切词时分不出），
    算术展开换成 SED_ARITHMETIC_VALUE，命令替换与变量换成占位 SED_SUBSTITUTION_ADDRESS（展开的值当行号读）。
    每一种组合要么切得完、占位都在行号地址位上，要么卡在 GNU sed 没有的命令字上（sed 编译就报错、什么都不跑）；都是的交 (字面命令列表, 认不出的说明列表)，有一种不是的交 None。"""
    import itertools
    pieces = sed_script_expansions(script)
    if not pieces or len(pieces) > SED_EXPANSION_LIMIT:
        return None
    texts, unjudged = [], []
    for choice in itertools.product((False, True), repeat=len(pieces)):
        rebuilt, last = [], 0
        for (start, end, kind), expanded in zip(pieces, choice):
            rebuilt.append(script[last:start])
            rebuilt.append((SED_ARITHMETIC_VALUE if kind == "arithmetic" else SED_SUBSTITUTION_ADDRESS) if expanded else script[start:end])
            last = end
        rebuilt.append(script[last:])
        parsed = sed_script_parse("".join(rebuilt))
        if parsed.failed_at is not None:
            if parsed.failed_at == "" or parsed.failed_at in GNU_SED_COMMAND_CHARACTERS or parsed.failed_at == SED_SUBSTITUTION_ADDRESS:
                return None
            continue
        if parsed.misplaced:
            return None
        texts += parsed.texts
        unjudged += parsed.unjudged
    return texts, unjudged


def sed_script_commands(script, texts, unjudged, shell_word=False):
    """GNU sed 脚本里起的命令（sed_script_parse）交进 texts / unjudged。shell_word：脚本是命令行上的一个词（-e 的值、头一个参数），里面可能留着没展开的 bash 展开；
    原样切不下去时照 sed_expanded_variants 逐种展开再切（弄坏开关 LIB_SHELL_WORDS_BREAK=sed-expansions-unparsed 下不另算）。sed -f 读的文件里没有 bash 展开，不给。"""
    parsed = sed_script_parse(script)
    if parsed.failed_at is not None and shell_word and not sed_expansions_unparsed():
        variants = sed_expanded_variants(script)
        if variants is not None:
            texts.extend(variants[0])
            unjudged.extend(variants[1])
            return
    texts.extend(parsed.texts)
    unjudged.extend(parsed.unjudged)


def sed_script_parse(script):
    """GNU sed 脚本：e 命令后面到脚本末尾的文字照命令交出（实测 GNU sed 4.9 连换行一起交给 sh）；单独的 e、s///e 执行模式空间里的文字（内容是数据，认不出）交说明；
    a / i / c 读文本（`a\\` 换行之后的文本行、行末带 `\\` 的续行都算文本，到不续行的换行为止），r / R / w / W / : / b / t / T / # 读到行尾，s 与 y 照分隔符读。
    认不出的写法（解析不下去）交说明，SedParse.failed_at 记卡在哪个字。"""
    texts, unjudged = [], []
    index, placeholders_in_addresses = 0, 0
    def finished(failed_at=None):
        return SedParse(texts, unjudged, failed_at, placeholders_in_addresses != script.count(SED_SUBSTITUTION_ADDRESS))
    def skip_spaces(position):
        while position < len(script) and script[position] in " \t":
            position += 1
        return position
    def read_address(position):
        nonlocal placeholders_in_addresses
        if position < len(script) and script[position] == SED_SUBSTITUTION_ADDRESS:
            # 占位单独占一个行号地址（前后不接数字，后面可以接 ~步长）：展开的值整个当这一个行号读
            placeholders_in_addresses += 1
            position += 1
            if position < len(script) and script[position] == "~":
                position += 1
                while position < len(script) and script[position].isdigit():
                    position += 1
        elif position < len(script) and script[position].isdigit():
            while position < len(script) and (script[position].isdigit() or script[position] == "~"):
                position += 1
        elif position < len(script) and script[position] == "$":
            position += 1
        elif position < len(script) and script[position] in "/\\":
            delimiter = script[position] if script[position] == "/" else script[position + 1:position + 2]
            position += 1 if script[position] == "/" else 2
            while position < len(script) and script[position] != delimiter:
                position += 2 if script[position] == "\\" else 1
            position += 1
            while position < len(script) and script[position] in "IM":
                position += 1
        return position
    while index < len(script):
        index = skip_spaces(index)
        if index >= len(script):
            break
        if script[index] in ";\n}":
            index += 1
            continue
        index = read_address(index)
        index = skip_spaces(index)
        if index < len(script) and script[index] == ",":
            index = skip_spaces(index + 1)
            if index < len(script) and script[index] in "+~":
                index += 1
            index = read_address(index)
        index = skip_spaces(index)
        while index < len(script) and script[index] == "!":
            index = skip_spaces(index + 1)
        if index >= len(script):
            break
        command = script[index]
        index += 1
        if command == "e":
            rest = script[index:].strip()
            if rest:
                texts.append(rest)
            else:
                unjudged.append("sed 脚本里单独的 e 把模式空间里的文字当命令执行，认不出起的是什么")
            return finished()
        if command in ":btTv":
            while index < len(script) and script[index] not in ";\n}":
                index += 1
            continue
        if command in "aic" and not sed_text_first_line_only():
            # 文本从命令字之后起（`a\\` 紧跟换行的，文本在下一行）；行末的 `\\` 把下一行接进文本，到不续行的换行为止
            index = skip_spaces(index)
            if script.startswith("\\\n", index):
                index += 2
            while index < len(script) and script[index] != "\n":
                index += 2 if script[index] == "\\" else 1
            index += 1
            continue
        if command in "aicrRwW#":
            end = script.find("\n", index)
            index = len(script) if end < 0 else end + 1
            continue
        if command in "sy":
            if index >= len(script):
                unjudged.append("sed 脚本解析不下去，认不出里面起不起命令")
                return finished("")
            delimiter = script[index]
            index += 1
            for _ in range(2):
                while index < len(script) and script[index] != delimiter:
                    index += 2 if script[index] == "\\" else 1
                index += 1
            if command == "s":
                flags_start = index
                unknown_flags_as_execution = words_break_is_set("sed-unknown-flags-as-execution")
                while index < len(script) and script[index] not in (";\n}w" if unknown_flags_as_execution else ";\n}w#"):
                    index += 1
                if not unknown_flags_as_execution and set(script[flags_start:index]) - SED_SUBSTITUTE_FLAG_CHARACTERS:
                    # 标志区有 GNU sed 不认的字（复跑分类报告 d137-replay-classify 二节 E8：分隔符被转义的反斜杠提前收尾，后面的字落进标志区）：编译就报错、一行都不跑
                    # （实测 GNU sed 4.9 报 unknown option to `s'、不读输入）；弄坏开关 LIB_SHELL_WORDS_BREAK=sed-unknown-flags-as-execution 下区里有 e 就算 e 标志
                    texts.clear()
                    unjudged.clear()
                    return finished()
                if "e" in script[flags_start:index]:
                    unjudged.append("sed 脚本里 s///e 把替换之后的模式空间当命令执行，认不出起的是什么")
                if index < len(script) and script[index] == "w":
                    end = script.find("\n", index)
                    index = len(script) if end < 0 else end + 1
            continue
        if command in "{=dDgGhHxnNpPlLqQzF":
            while index < len(script) and script[index].isdigit():
                index += 1
            continue
        unjudged.append("sed 脚本解析不下去，认不出里面起不起命令")
        return finished(command)
    return finished()


def make_expanded(text, variables, depth, shell_texts, unjudged):
    """make 文本里的变量引用照这份 Makefile 与命令行上的定义展开：$$ 是 $；$(shell …) 里的命令交进 shell_texts、结果换成占位；别的函数与展开不了的变量换成没展开的 shell 变量
    （落在命令词上调用方照认不出办）；自动变量 $@ $< 这类换成占位。"""
    output, index = [], 0
    while index < len(text):
        character = text[index]
        if character != "$" or index + 1 >= len(text):
            output.append(character)
            index += 1
            continue
        following = text[index + 1]
        if following == "$":
            output.append("$$")
            index += 2
            continue
        if following in "({":
            closing = ")" if following == "(" else "}"
            level, end = 0, index + 1
            while end < len(text):
                level += 1 if text[end] == following else -1 if text[end] == closing else 0
                if level == 0:
                    break
                end += 1
            inner = text[index + 2:end]
            index = end + 1
            name_match = re.match(r"^([A-Za-z_][A-Za-z0-9_.\-]*)$", inner)
            if inner.startswith("shell ") or inner.startswith("shell\t"):
                shell_texts.append(make_expanded(inner[6:], variables, depth + 1, shell_texts, unjudged) if depth < EMBEDDED_NESTING else inner[6:])
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "SHELL_RESULT")
            elif name_match and name_match.group(1) in variables and depth < EMBEDDED_NESTING:
                output.append(make_expanded(variables[name_match.group(1)], variables, depth + 1, shell_texts, unjudged))
            elif name_match:
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + re.sub(r"[^A-Za-z0-9_]", "_", name_match.group(1)))
            else:
                if depth < EMBEDDED_NESTING:
                    make_expanded(inner, variables, depth + 1, shell_texts, unjudged)   # 函数参数里的 $(shell …) 照收
                output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "FUNCTION")
            continue
        output.append("$" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + ("AUTOMATIC" if following in "@<^?*+|%" else re.sub(r"[^A-Za-z0-9_]", "_", following)))
        index += 2
    return "".join(output).replace("$$", "$")


def scan_makefile(text, texts, unjudged, command_line_variables=()):
    """Makefile 正文：配方行（制表符起头的行，与 `目标: 依赖 ; 配方`）与 $(shell …) 照 shell 命令交出，变量照文件与命令行的定义展开（make_expanded）；
    include 别的文件、SHELL 换成不是 sh / bash 的、.RECIPEPREFIX 换了配方前缀的交说明。"""
    joined = re.sub(r"\\\n[ \t]*", " ", text)
    variables = dict(MAKE_DEFAULT_VARIABLES)
    lines = joined.split("\n")
    define_name, define_lines = None, []
    for line in lines:
        if define_name is not None:
            if line.strip() == "endef":
                variables[define_name] = "\n".join(define_lines)
                define_name = None
            else:
                define_lines.append(line)
            continue
        define = re.match(r"^(?:(?:export|override)\s+)*define\s+([A-Za-z_][A-Za-z0-9_.\-]*)", line)
        if define:
            define_name, define_lines = define.group(1), []
            continue
        if line.startswith("\t"):
            continue
        assignment = MAKE_ASSIGNMENT.match(line)
        if assignment:
            name, operator, value = assignment.group(1), assignment.group(2), assignment.group(3).strip()
            if operator == "+=" and name in variables:
                variables[name] = variables[name] + " " + value
            elif operator == "?=" and name in variables and name not in MAKE_DEFAULT_VARIABLES:
                pass
            elif operator == "!=":
                variables[name] = "$(" + MAKE_VARIABLE_PLACEHOLDER_PREFIX + "SHELL_ASSIGNMENT)"
            else:
                variables[name] = value
    variables.update(dict(command_line_variables))
    shell_value = variables.get("SHELL", "/bin/sh").strip()
    if os.path.basename(shell_value.split()[0] if shell_value.split() else "sh") not in SHELL_NAMES | RENAMED_SHELL_NAMES:
        unjudged.append(f"Makefile 把 SHELL 换成了 {shell_value}，认不出配方交给谁跑")
    shell_texts = []
    in_define = False
    for line in lines:
        stripped = line.strip()
        if re.match(r"^(?:(?:export|override)\s+)*define\s", line):
            in_define = True
            continue
        if in_define:
            in_define = stripped != "endef"
            continue
        if re.match(r"^-?(?:include|sinclude)\s", stripped):
            unjudged.append(f"Makefile 里 `{stripped[:80]}` 读进别的文件，那里的配方没读")
            continue
        if stripped.startswith(".RECIPEPREFIX"):
            unjudged.append("Makefile 换了配方前缀（.RECIPEPREFIX），认不出哪几行是配方")
            continue
        recipe = None
        shell_assignment = MAKE_ASSIGNMENT.match(line) if not line.startswith("\t") else None
        if shell_assignment and shell_assignment.group(2) == "!=":
            texts.append(make_expanded(shell_assignment.group(3), variables, 0, shell_texts, unjudged))   # `名字 != 命令`：读 Makefile 时就跑这条命令
            continue
        if line.startswith("\t"):
            recipe = line[1:]
        elif ";" in line and ":" in line.split(";", 1)[0] and not MAKE_ASSIGNMENT.match(line):
            recipe = line.split(";", 1)[1]
        else:
            make_expanded(line, variables, 0, shell_texts, unjudged)   # 变量定义与规则行里的 $(shell …) 读 Makefile 时就跑
            continue
        recipe = MAKE_RECIPE_PREFIX.sub("", recipe)
        if recipe.strip():
            texts.append(make_expanded(recipe, variables, 0, shell_texts, unjudged))
    texts.extend(shell_texts)


def program_text_launches(language, text, variables=()):
    """程序从文件读的代码（awk -f、sed -f、Makefile）里起的命令：交 (字面命令的列表, 认不出的说明列表)。variables 是 make 命令行上的定义。"""
    texts, unjudged = [], []
    if language == "awk":
        scan_awk(text, texts, unjudged, variables)
    elif language == "sed":
        sed_script_commands(text, texts, unjudged)
    elif language == "make":
        scan_makefile(text, texts, unjudged, variables)
    return texts, unjudged


def git_config_commands(arguments, directory, environment):
    """git 这一条要读的配置里带命令的几项与别名：同一个进程里按 (目录, 配置相关的环境变量) 缓存，交 [(键, 值), …]。经 `git config --get-regexp` 照 git 自己的读法取
    （全局、系统、仓里的配置与 include 一并读）；git 起不来、超时、不在仓里交读到的那些（可以为空）。这一份库里只有这一处起子进程。"""
    import subprocess
    directory_arguments = []
    for option, value in arguments:
        if option in ("-C", "--git-dir", "--work-tree"):
            directory_arguments += [option, value]
    keyed_environment = tuple(sorted((name, value) for name, value in environment.items() if name.startswith("GIT_CONFIG") or name in ("HOME", "XDG_CONFIG_HOME", "GIT_DIR")))
    key = (directory, tuple(directory_arguments), keyed_environment)
    if key not in GIT_CONFIG_QUERY_CACHE:
        found = []
        try:
            result = subprocess.run(["git", *directory_arguments, "config", "--null", "--get-regexp", GIT_CONFIG_QUERY_PATTERN], cwd=directory if directory and os.path.isdir(directory) else None,
                                    env=dict(os.environ, **dict(keyed_environment)), capture_output=True, text=True, timeout=GIT_CONFIG_QUERY_SECONDS)
            for entry in result.stdout.split("\0"):
                if "\n" in entry:
                    name, value = entry.split("\n", 1)
                    found.append((name.lower(), value))
        except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
            pass
        GIT_CONFIG_QUERY_CACHE[key] = found
    return GIT_CONFIG_QUERY_CACHE[key]


def embedded_option_variables_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-option-variables-unexpanded：make -C 的目录、git -c 的配置写成变量、命令替换的不代入也不交说明（目录照字面拼、配置照字面认；只给自检证红用）。"""
    return words_break_is_set("embedded-option-variables-unexpanded")


def scan_git(arguments, directory, environment, texts, unjudged, variables=None):
    """git：-c 给的与配置里读到的别名（`!` 起头的照命令交出，别的接着照别名链找，最多 EMBEDDED_NESTING 层）、带命令的配置项（core.pager、core.fsmonitor、diff.external……）
    与这一条设的带命令的环境变量（GIT_EXTERNAL_DIFF、GIT_PAGER、GIT_SSH_COMMAND……）照命令交出；GIT_EXEC_PATH、GIT_CONFIG_PARAMETERS 与 --config-env 交说明。
    -c 的配置写成变量、命令替换的：variables（这段命令文本里前面赋过的）代入得了的照代入；代入不了而配置名本身不是字面、或配置名是 alias.* 的交说明（认不出设的是哪一项、别名起什么）。"""
    options, position, given = [], 0, {}
    while position < len(arguments):
        argument = arguments[position]
        if not argument.startswith("-") or argument == "--":
            break
        option, has_value, value = argument.partition("=") if argument.startswith("--") else (argument, False, "")
        if option in GIT_OPTIONS_WITH_VALUE and not has_value:
            value = arguments[position + 1] if position + 1 < len(arguments) else ""
            position += 1
        position += 1
        if option == "-c" and ("$" in value or "`" in value) and not embedded_option_variables_unexpanded():
            expanded = expand_known_variables(value, variables or {})
            if expanded is not None:
                value = expanded
            else:
                written_name = value.partition("=")[0]
                if "$" in written_name or "`" in written_name:
                    unjudged.append(f"git -c {value} 的配置名写成变量或命令替换，认不出设的是哪一项、起不起命令")
                    continue
                if written_name.lower().startswith("alias."):
                    unjudged.append(f"git -c {value} 的别名值写成变量或命令替换，认不出这个别名起的是什么")
                    continue
        options.append((option, value))
        if option == "-c":
            name, _, config_value = value.partition("=")
            given[name.lower()] = config_value
        elif option == "--config-env":
            unjudged.append(f"git --config-env {value} 从环境变量取配置值，认不出它起不起命令")
        elif option == "--exec-path" and has_value:
            unjudged.append(f"git --exec-path={value} 换掉了 git 的子命令程序，认不出起的是什么")
    for name in GIT_OPAQUE_ENVIRONMENT:
        if name in environment:
            unjudged.append(f"git 这一条设了 {name}，认不出 git 起的是什么")
    count = environment.get("GIT_CONFIG_COUNT")
    if count is not None:
        if not count.isdigit():
            unjudged.append("git 这一条的 GIT_CONFIG_COUNT 不是字面的数，认不出它给的配置")
        else:
            for number in range(int(count)):
                name, value = environment.get(f"GIT_CONFIG_KEY_{number}"), environment.get(f"GIT_CONFIG_VALUE_{number}")
                if name is None or value is None:
                    unjudged.append(f"git 这一条的 GIT_CONFIG_KEY_{number} / GIT_CONFIG_VALUE_{number} 没给全，认不出它给的配置")
                else:
                    given[name.lower()] = value
    for name in GIT_COMMAND_ENVIRONMENT:
        value = environment.get(name)
        if value:
            texts.append(value)
    configured = dict(git_config_commands(options, directory, environment))
    configured.update(given)
    for name, value in configured.items():
        if name.startswith("alias."):
            continue
        if name in ("credential.helper",) or (name.startswith("credential.") and name.endswith(".helper")):
            if value.startswith("!"):
                texts.append(value[1:])
            continue
        if GIT_COMMAND_CONFIG_KEY.match(name) and not ((name == "core.pager" or name.startswith("pager.") or name == "core.fsmonitor") and value.lower() in GIT_BOOLEAN_VALUES):
            texts.append(value)
    subcommand_words = arguments[position:]
    for _ in range(EMBEDDED_NESTING):
        if not subcommand_words:
            break
        alias = configured.get("alias." + subcommand_words[0].lower())
        if alias is None:
            break
        if alias.startswith("!"):
            texts.append(alias[1:] + "".join(" " + shlex.quote(word) for word in subcommand_words[1:]))
            break
        try:
            subcommand_words = shlex.split(alias) + subcommand_words[1:]
        except ValueError:
            unjudged.append(f"git 别名 {subcommand_words[0]} 的值切不开，认不出起的是什么")
            break


def scan_options_for_commands(name, arguments, environment, texts, unjudged):
    """tar、ssh、less、vim 这几样带命令的选项与环境变量：值是字面的照命令交出。"""
    if name in ("tar", "bsdtar", "gtar"):
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            option, has_value, value = argument.partition("=")
            matched = next((known for known in TAR_COMMAND_OPTIONS if known.startswith(option) and option.startswith("--") and len(option) > 3), None)
            if matched:
                if not has_value:
                    value = arguments[position + 1] if position + 1 < len(arguments) else ""
                    position += 1
                texts.append(value)
            elif argument.startswith("--checkpoint-action=exec="):
                texts.append(argument[len("--checkpoint-action=exec="):])
            elif argument in ("-I", "-F"):
                texts.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                position += 1
            elif argument.startswith("-") and not argument.startswith("--") and ("I" in argument[1:] or "F" in argument[1:]):
                unjudged.append(f"tar 的短选项 {argument} 里带 -I / -F（起命令的选项），认不出起的是什么")
            position += 1
    elif name == "ssh":
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            value = None
            if argument == "-o" and position + 1 < len(arguments):
                value = arguments[position + 1]
                position += 1
            elif argument.startswith("-o") and len(argument) > 2:
                value = argument[2:]
            if value is not None:
                key, _, setting = value.replace("=", " ", 1).partition(" ")
                if key.lower() in SSH_COMMAND_OPTIONS and setting.strip().lower() != "none":
                    texts.append(setting.strip())
            elif not argument.startswith("-"):
                break
            position += 1
    elif name == "less":
        for argument in arguments:
            if argument.startswith("+") and "!" in argument:
                texts.append(argument.split("!", 1)[1])
        for variable in LESS_OPEN_ENVIRONMENT:
            value = environment.get(variable)
            if value:
                texts.append(value.lstrip("|-").replace("%s", "file"))
    elif name in VIM_NAMES:
        position = 0
        while position < len(arguments):
            argument = arguments[position]
            ex_command = None
            if argument in ("-c", "--cmd") and position + 1 < len(arguments):
                ex_command = arguments[position + 1]
                position += 1
            elif argument.startswith("+") and len(argument) > 1:
                ex_command = argument[1:]
            if ex_command is not None:
                stripped = ex_command.lstrip(": ")
                if stripped.startswith("!"):
                    texts.append(stripped[1:])
                elif re.search(r"!|system\s*\(|\bterm|\bsh(?:ell)?\b|\bpy|\blua|\bperl|\bruby", stripped):
                    unjudged.append(f"vim 的 ex 命令 `{stripped[:80]}` 能起命令，认不出起的是什么")
            position += 1


def interpreter_code_options(arguments, code_options, value_options, flag_takes_code=""):
    """解释器的命令行：code_options 里的选项（`-e` 这类，值贴着写或另起一个词，短选项可以合写在一簇的末尾）给的代码串都收；value_options 带一个值、不是代码。
    交 (代码串列表, 第一个不是选项的词的下标或 None)。"""
    codes, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--":
            return codes, position + 1 if position + 1 < len(arguments) else None
        if not argument.startswith("-") or argument == "-":
            return codes, position
        matched = False
        for option in code_options:
            if argument == option:
                codes.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                position += 2
                matched = True
                break
            if option.startswith("--") and argument.startswith(option + "="):
                codes.append(argument[len(option) + 1:])
                position += 1
                matched = True
                break
        if matched:
            continue
        if argument in value_options:
            position += 2
            continue
        if not argument.startswith("--") and flag_takes_code:
            letters = argument[1:]
            for offset, letter in enumerate(letters):
                if letter in flag_takes_code:
                    rest = letters[offset + 1:]
                    if rest:
                        codes.append(rest)
                        position += 1
                    else:
                        codes.append(arguments[position + 1] if position + 1 < len(arguments) else "")
                        position += 2
                    break
            else:
                position += 1
            continue
        position += 1
    return codes, None


def embedded_launches(command):
    """命令位置上的一条命令（CommandAtPosition）是别的程序时，它里面起的命令（文件头「别的程序里起 shell」一段；EmbeddedLaunches）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=embedded-launches-unseen 下一条都不认。"""
    if embedded_launches_unseen() or not command.words:
        return EmbeddedLaunches((), (), ())
    name, arguments = os.path.basename(command.words[0]), list(command.words[1:])
    environment = command.environment or {}
    texts, unjudged, files = [], [], []
    awk_assignments = []   # awk 命令行上的 `名字=值`（-v 与操作数）：(名字, 值或 None)，值写成 shell 变量、命令替换的是 None
    def scan_code(language, code_texts, reads_standard_input):
        if reads_standard_input:
            code, problem, from_file = interpreter_standard_input(command)
            if code is not None:
                code_texts = list(code_texts) + [code]
            if problem:
                unjudged.append(f"{name} 从标准输入读代码：{problem}")
        for code in code_texts:
            if language == "python":
                scan_python(code, texts, unjudged)
            elif language == "tcl":
                scan_tcl(code, texts, unjudged)
            elif language == "awk":
                scan_awk(code, texts, unjudged, awk_assignments)
            elif language == "perl":
                scan_quoted_language("perl", code, PERL_LAUNCH_WORD, ("$", "@"), "'\"`", "#", texts, unjudged)
            elif language == "ruby":
                scan_quoted_language("ruby", code, RUBY_LAUNCH_WORD, ("#{",), "'\"`", "#", texts, unjudged)
            elif language == "php":
                scan_quoted_language("php", code, PHP_LAUNCH_WORD, ("$",), "'\"`", "#", texts, unjudged)
            elif language == "lua":
                scan_quoted_language("lua", code, LUA_LAUNCH_WORD, (), "'\"", None, texts, unjudged)
            elif language == "node":
                if "child_process" in code:
                    before = len(texts) + len(unjudged)
                    scan_quoted_language("node", code, NODE_LAUNCH_WORD, ("${",), "'\"`", None, texts, unjudged)
                    if len(texts) + len(unjudged) == before:
                        unjudged.append("node 代码用了 child_process，认不出起的是什么")
    if PYTHON_NAME.match(name):
        invocation = python_invocation(arguments)
        if invocation.code_string is not None:
            scan_code("python", [invocation.code_string], False)
        elif invocation.module is None and invocation.script_index is None:
            scan_code("python", [], True)
    elif PERL_NAME.match(name):
        perl_value_options = PERL_OPTIONS_WITH_VALUE_WORD + (PERL_OPTIONS_WITH_ATTACHED_VALUE if perl_attached_options_take_next_word() else ())
        codes, script_index = interpreter_code_options(arguments, ("-e", "-E"), perl_value_options, flag_takes_code="eE")
        scan_code("perl", codes, not codes and script_index is None)
    elif RUBY_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-e",), ("-I", "-r", "-C", "-E", "-F", "-K", "-T", "-W", "-x", "--encoding"), flag_takes_code="e")
        scan_code("ruby", codes, not codes and script_index is None)
    elif name in NODE_NAMES:
        codes, script_index = interpreter_code_options(arguments, ("-e", "--eval", "-p", "--print"), ("-r", "--require", "--import", "--loader", "-C", "--conditions"))
        scan_code("node", codes, not codes and script_index is None)
    elif PHP_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-r", "-B", "-R", "-E"), ("-c", "-d", "-z", "-t", "-F"))
        scan_code("php", codes, False)
    elif LUA_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-e",), ("-l",))
        scan_code("lua", codes, not codes and script_index is None)
    elif TCL_NAME.match(name):
        codes, script_index = interpreter_code_options(arguments, ("-c",), ("-f", "-encoding", "-D"))
        scan_code("tcl", codes, not codes and script_index is None)
    elif name in AWK_NAMES or (name == "busybox" and arguments[:1] == ["awk"]):
        if name == "busybox":
            arguments = arguments[1:]
        programs, program_files, position = [], [], 0
        while position < len(arguments):
            argument = arguments[position]
            if argument == "--":
                position += 1
                break
            if argument in ("-f", "--file") or argument.startswith("--file="):
                program_files.append(argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else ""))
                position += 1 if "=" in argument else 2
                continue
            if argument in ("-e", "--source") or argument.startswith("--source="):
                programs.append(argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else ""))
                position += 1 if "=" in argument else 2
                continue
            if argument in ("-v", "--assign") or argument.startswith("--assign=") or (argument.startswith("-v") and len(argument) > 2):
                given = argument.split("=", 1)[1] if argument.startswith("--assign=") else argument[2:] if argument.startswith("-v") and len(argument) > 2 else (
                    arguments[position + 1] if position + 1 < len(arguments) else "")
                assignment = ASSIGNMENT.match(given)
                if assignment:
                    awk_assignments.append((assignment.group(1), None if "$" in assignment.group(2) or "`" in assignment.group(2) else assignment.group(2)))
                position += 2 if argument in ("-v", "--assign") else 1
                continue
            if argument in ("-F", "--field-separator", "-i", "--include", "-l", "--load", "-W"):
                position += 2
                continue
            if argument.startswith("-f") and len(argument) > 2:
                program_files.append(argument[2:])
                position += 1
                continue
            if argument.startswith("-") and argument != "-":
                position += 1
                continue
            break
        if not programs and not program_files and position < len(arguments):
            programs.append(arguments[position])
            position += 1
        for operand in arguments[position:]:
            assignment = ASSIGNMENT.match(operand)
            if assignment:
                awk_assignments.append((assignment.group(1), None if "$" in assignment.group(2) or "`" in assignment.group(2) else assignment.group(2)))
        scan_code("awk", programs, False)
        files += [ProgramFile("awk", (program_file,), command.directory, tuple(awk_assignments)) for program_file in program_files]
    elif name in ("sed", "gsed") or (name == "busybox" and arguments[:1] == ["sed"]):
        if name == "busybox":
            arguments = arguments[1:]
        scripts, script_files, position, sandbox = [], [], 0, False
        while position < len(arguments):
            argument = arguments[position]
            if argument == "--":
                position += 1
                break
            if argument == "--sandbox":
                sandbox = True
            elif argument.startswith("--expression") or argument.startswith("--file"):
                value = argument.split("=", 1)[1] if "=" in argument else (arguments[position + 1] if position + 1 < len(arguments) else "")
                (scripts if argument.startswith("--expression") else script_files).append(value)
                position += 1 if "=" in argument else 2
                continue
            elif argument.startswith("--"):
                if argument in ("--line-length",):
                    position += 1
            elif argument.startswith("-") and argument != "-":
                letters = argument[1:]
                consumed = False
                for offset, letter in enumerate(letters):
                    if letter in "efl":
                        value = letters[offset + 1:] or (arguments[position + 1] if position + 1 < len(arguments) else "")
                        if letter == "e":
                            scripts.append(value)
                        elif letter == "f":
                            script_files.append(value)
                        position += 1 if letters[offset + 1:] else 2
                        consumed = True
                        break
                    if letter == "i":
                        break
                if consumed:
                    continue
            else:
                break
            position += 1
        if not scripts and not script_files and position < len(arguments):
            scripts.append(arguments[position])
        if not sandbox:
            for script in scripts:
                sed_script_commands(script, texts, unjudged, shell_word=True)
            files += [ProgramFile("sed", (script_file,), command.directory) for script_file in script_files]
    elif name in MAKE_NAMES:
        makefiles, evaluations, directory, variables, position, runs_nothing = [], [], command.directory, [], 0, False
        directory_unresolved = False   # -C 的目录写成变量而代入不了：Makefile 认不出在哪（交说明，不按目录找）
        while position < len(arguments):
            argument = arguments[position]
            if argument in ("-f", "--file", "--makefile", "-C", "--directory", "-E", "--eval", "-I", "--include-dir", "-o", "--old-file", "--assume-old", "-W", "--what-if", "--new-file", "--assume-new"):
                value = arguments[position + 1] if position + 1 < len(arguments) else ""
                position += 2
            elif argument.startswith("--") and "=" in argument:
                argument, value = argument.split("=", 1)
                position += 1
            elif argument[:2] in ("-f", "-C", "-E", "-I", "-o", "-W") and len(argument) > 2:
                argument, value = argument[:2], argument[2:]
                position += 1
            else:
                value = None
                position += 1
            if argument in ("-f", "--file", "--makefile"):
                makefiles.append(value)
            elif argument in ("-C", "--directory"):
                if ("$" in value or "`" in value) and not embedded_option_variables_unexpanded():
                    # 目录写成变量、命令替换：前面赋过字面值的照代入；代入不了的认不出读哪一份 Makefile（照认不出拒），默认的几份 Makefile 不再按别的目录找
                    expanded = expand_known_variables(value, command.variables or {})
                    if expanded is None:
                        unjudged.append(f"make -C {value} 的目录写成变量或命令替换，认不出读的是哪一份 Makefile")
                        directory_unresolved = True
                        continue
                    value = expanded
                directory = resolve_path(directory, os.path.expanduser(value)) if directory is not None or os.path.isabs(value) else None
            elif argument in ("-E", "--eval"):
                evaluations.append(value)
            elif argument in ("-v", "--version", "-h", "--help"):
                runs_nothing = True
            elif value is None and ASSIGNMENT.match(argument):
                assignment = ASSIGNMENT.match(argument)
                variables.append((assignment.group(1), assignment.group(2)))
        if not runs_nothing:
            for evaluation in evaluations:
                scan_makefile(evaluation, texts, unjudged, variables)
            standard_input_makefiles = [makefile for makefile in makefiles if makefile in ("-", "/dev/stdin")]
            if standard_input_makefiles:
                code, problem, from_file = interpreter_standard_input(command)
                if code is not None:
                    scan_makefile(code, texts, unjudged, variables)
                if problem:
                    unjudged.append(f"make 从标准输入读 Makefile：{problem}")
            other_makefiles = [makefile for makefile in makefiles if makefile not in ("-", "/dev/stdin")]
            if directory_unresolved:
                files += [ProgramFile("make", (makefile,), directory, tuple(variables)) for makefile in other_makefiles if os.path.isabs(makefile)]
            elif other_makefiles:
                files += [ProgramFile("make", (makefile,), directory, tuple(variables)) for makefile in other_makefiles]
            elif not makefiles:
                files.append(ProgramFile("make", MAKE_DEFAULT_FILES, directory, tuple(variables)))
    elif name == "git":
        scan_git(arguments, command.directory, environment, texts, unjudged, command.variables)
    else:
        scan_options_for_commands(name, arguments, environment, texts, unjudged)
    return EmbeddedLaunches(tuple(text for text in texts if text.strip()), tuple(dict.fromkeys(unjudged)), tuple(files))
