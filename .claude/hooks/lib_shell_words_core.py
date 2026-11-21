# 切词、引号与替换的收尾、花括号展开、简单命令的切分、前缀与包装的剥法、标准输入与 heredoc、变量与 cd 的跟踪、函数体的认法：lib_shell_words.py 的共用底层。
# lib_shell_words.py 把这里的名字全部再导出，调用方只 import lib_shell_words，不直接导入这一份。
import os
import re
import shlex
from typing import NamedTuple


SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
# 改名 / 别的 shell（认不出的一律拒，偏严）：rbash 是限制 bash、ash 是 busybox / BSD 的 POSIX shell，都当 shell 判
# （rbash 的启动文件、posix 判定照 bash）；busybox 的 sh / ash 这类 applet 由 skip_prefixes 剥掉 busybox 这一层。弄坏开关 LIB_SHELL_WORDS_BREAK=renamed-shells-unseen 下都不认。
RENAMED_SHELL_NAMES = {"rbash", "ash"}
BUSYBOX_SHELL_APPLETS = {"sh", "ash", "bash", "dash", "rbash"}
# shell 起的脚本路径写成标准输入设备（读的是标准输入，不是盘上一份脚本）：/dev/stdin、/dev/fd/0、/proc/self/fd/0、/proc/<pid>/fd/0
STANDARD_INPUT_DEVICE = re.compile(r"/dev/stdin|/dev/fd/0|/proc/(?:self|\d+)/fd/0")


def renamed_shells_unseen():
    return words_break_is_set("renamed-shells-unseen")


def is_shell_name(name):
    """name（已取 basename）当不当 shell 判：bash / sh / dash / zsh / ksh，连同 rbash、ash（弄坏开关 renamed-shells-unseen 下只认前五个）。"""
    return name in SHELL_NAMES or (not renamed_shells_unseen() and name in RENAMED_SHELL_NAMES)


def is_standard_input_device(word):
    """word 是不是指到标准输入的设备路径（/dev/stdin、/dev/fd/0、/proc/self/fd/0 这类）。"""
    return bool(STANDARD_INPUT_DEVICE.fullmatch(word))


def expand_directory_tilde(word, directory):
    """词开头的 `~+`（$PWD，取 directory）、`~-`（$OLDPWD，算不出）照 bash 展开，其余的 `~` / `~user` 照 os.path.expanduser；
    `~+` 的 directory 是 None 时交 None，`~-` 一律交 None。弄坏开关 LIB_SHELL_WORDS_BREAK=tilde-plus-minus-unexpanded 下 `~+` / `~-` 不展开（当普通路径，`~+/x` 落不到实处）。"""
    if not words_break_is_set("tilde-plus-minus-unexpanded"):
        if word == "~+" or word.startswith("~+/"):
            return None if directory is None else directory + word[2:]
        if word == "~-" or word.startswith("~-/"):
            return None
    return os.path.expanduser(word)
SHELL_OPERATORS = sorted(["&&", "||", ";;", "|&", "&>>", "&>", ">>", ">&", "<&", ">|", "<<<", "<<-", "<<", "<>",
                          ";", "&", "|", "(", ")", "<", ">", "\n"], key=len, reverse=True)
OPERATOR_START = set(";&|()<>\n")
COMMAND_SEPARATORS = {"&&", "||", ";;", "|&", ";", "&", "|", "(", ")", "\n"}
OUTPUT_REDIRECTS = {">", ">>", ">|", "&>", "&>>", ">&"}
REDIRECT_OPERATORS = OUTPUT_REDIRECTS | {"<", "<>", "<&", "<<", "<<-", "<<<"}   # 后面跟一个目标词的重定向运算符
COMMAND_KEYWORDS = {"if", "then", "else", "elif", "do", "while", "until", "!", "{", "}", "time"}
# 命令前面可以挂的前缀：名字 → 要带一个值的选项；timeout 另有一个时长位置参数，taskset 另有一个 CPU 掩码位置参数
PREFIX_OPTIONS_WITH_VALUE = {
    "nohup": set(), "setsid": set(), "command": set(), "exec": {"-a"},
    "nice": {"-n", "--adjustment"}, "ionice": {"-c", "-n", "-p", "-P", "-u", "--class", "--classdata"},
    "timeout": {"-s", "-k", "--signal", "--kill-after"}, "env": {"-u", "--unset", "-C", "--chdir"},
    "stdbuf": {"-i", "-o", "-e", "--input", "--output", "--error"}, "sudo": {"-u", "-g", "--user", "--group"},
    "taskset": set(),
}
PREFIX_POSITIONAL_COUNT = {"timeout": 1, "taskset": 1}
ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$", re.S)
ARRAY_ASSIGNMENT_START = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\+?=$")  # `名字=(` 与 `名字+=(` 的前半，后面紧跟左括号
PYTHON_NAME = re.compile(r"^python(?:\d+(?:\.\d+)?)?$")
# python 解释器带一个值的选项：-c <代码>、-m <模块> 定执行什么；-W <警告设置>、-X <实现选项> 与 --check-hash-based-pycs <模式> 的值只是解释器的设置
PYTHON_SHORT_OPTIONS_WITH_VALUE = "cmWX"
PYTHON_LONG_OPTIONS_WITH_VALUE = {"--check-hash-based-pycs"}
# source / . 后面跟的得像一个路径才算读脚本（`source = open(…)` 这类 python 源码里的写法不算）
# bash 带一个值的长选项（`--rcfile 文件`、`--init-file 文件`）：值不是被执行的脚本
SHELL_LONG_OPTIONS_WITH_VALUE = {"--rcfile", "--init-file"}
SOURCE_TARGET = re.compile(r"^[^\s=,-][^\s=,]*$")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1([^\n]*)\n(.*?)\n\s*\2[ \t]*(?=\n|$)", re.S)
MAXIMUM_NESTING = 8
# 先带一个参数、再接那条命令的包装脚本：capped.sh N（线程上限；N 是正整数或 default，default 取 .claude/heavy-thread-default 的登记值）、
# run-with-memory-cap.sh <上限>（内存上限）。参数不以 - 开头就剥掉它们两个词（不看是数还是 default），往里判那条命令
WRAPPERS_TAKING_ONE_ARGUMENT = {"capped.sh", "run-with-memory-cap.sh"}
# 其中给命令定内存上限的那一个：剥掉它的命令记 memory_capped（project-code-guard.sh 判子 agent 跑编译出来的代码经没经它）
MEMORY_CAP_WRAPPER = "run-with-memory-cap.sh"
# 剥掉的 heredoc 在剥完的文字里换成这个记号（NUL 开头，不是 \w，不会被 HEREDOC 再认一次），切词之后它是 `<<` 的目标词，按编号找回正文
HEREDOC_BODY_MARKER = re.compile(r"^\x00heredoc-body-(\d+)\x00$")


class RedirectOperator(str):
    """重定向运算符，带着写在它前面的 fd 号（`2>` 的 "2"）：与运算符本身逐字相等，按字面比运算符的地方照旧；没写 fd 号的运算符是普通的 str。"""
    fd = None

    def __new__(cls, operator, fd):
        instance = super().__new__(cls, operator)
        instance.fd = fd
        return instance


def redirection_fd(operator):
    """写在重定向运算符前面的 fd 号，照 bash 按十进制数认（`01>` 是 1 号）；没写交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-number-literal 下照字面比：写法不是最简十进制的（`01`）当作哪个号都对不上。"""
    fd = getattr(operator, "fd", None)
    if fd is None:
        return None
    if words_break_is_set("redirect-fd-number-literal") and fd != str(int(fd)):
        return -1
    return int(fd)


def redirects_standard_output(operator):
    """这个输出重定向改的是不是这一条的标准输出：`>`、`>>`、`>|` 前面没写 fd 号或写的是 1（照数认，`01>` 也是），`&>`、`&>>`（`2> 文件` 不算；fd 号见 RedirectOperator）。"""
    return operator in OUTPUT_REDIRECTS and operator != ">&" and redirection_fd(operator) in (None, 1)


def standard_output_destination(redirections):
    """一条简单命令跑起来时标准输出最后落到哪个文件：照 bash 从左到右处理输出重定向（((运算符, 目标词), …)，CommandAtPosition.output_redirections 那种）。
    `n>文件`、`n>>文件`、`n>|文件` 改 n 号（不写是 1）；`&>`、`&>>` 与 `>&文件`（不写 n 或写 1）同时改 1、2 号；`n>&m` 把 n 号接到 m 号此刻指的地方
    （`cat 源 3>d.sh >&3` 的标准输出进 d.sh）；`n>&-` 关掉 n 号；`n>&文件`（n 不是 1）bash 报 ambiguous redirect、整条不跑。
    交 (目标词, 打开那个文件的运算符：`>`、`>>`、`>|`、`&>`、`&>>`、`>&` 之一)；标准输出没落到文件（没改、接到别的继承来的号、关了、整条不跑）交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=redirect-fd-duplication-unseen 下不跟 `n>&m`：只取直接改标准输出（redirects_standard_output）的最后一个。"""
    if words_break_is_set("redirect-fd-duplication-unseen"):
        direct = [(target, str(operator)) for operator, target in redirections if redirects_standard_output(operator)]
        return direct[-1] if direct else None
    table = {}   # fd 号 → ("file", 目标词, 运算符) / ("fd", 继承来的号) / ("closed",)；没写的号是继承来的
    for operator, target in redirections:
        plain, fd = str(operator), redirection_fd(operator)
        if plain in ("&>", "&>>"):
            table[1] = table[2] = ("file", target, plain)
        elif plain == ">&":
            number = 1 if fd is None else fd
            if re.fullmatch(r"[0-9]+", target):
                table[number] = table.get(int(target), ("fd", int(target)))
            elif target == "-":
                table[number] = ("closed",)
            elif number == 1:
                table[1] = table[2] = ("file", target, plain)
            else:
                return None
        elif plain in (">", ">>", ">|"):
            table[1 if fd is None else fd] = ("file", target, plain)
    entry = table.get(1, ("fd", 1))
    return (entry[1], entry[2]) if entry[0] == "file" else None


BASH_BUILTINS = frozenset(". : [ alias bg bind break builtin caller cd command compgen complete compopt continue declare dirs disown echo enable eval exec exit "
                          "export false fc fg getopts hash help history jobs kill let local logout mapfile popd printf pushd pwd read readarray readonly return "
                          "set shift shopt source suspend test times trap true type typeset ulimit umask unalias unset wait".split())   # bash 5.2 的 compgen -b


class HeredocBody(NamedTuple):
    operator: str                 # "<<" 或 "<<-"（后者的正文已经去掉每行开头的制表符）
    is_delimiter_quoted: bool     # 结束词带引号：正文照原样写出；不带引号时 shell 还要展开正文里的 $ 与命令替换，这里给的是展开之前的原文
    text: str                     # 正文，最后一行之后带一个换行（写进文件的就是这些字节）


class PrefixSkip(NamedTuple):
    command_position: int | None  # 命令词的下标；这一条只有关键字、赋值与前缀时是 None
    prefix_names: set[str]         # 路过的前缀名（nohup、setsid、nice……）
    assignments: dict[str, str]    # 路过的赋值：命令前的 `名字=值` 与 env 参数里的 `名字=值`
    chdir: str | None = None       # env 的 `-C 目录` / `-C目录` / `--chdir=目录` / `--chdir 目录` 里最后一个目录（照写的样子，没解开）；没有是 None
    split: tuple[int, int, str] | None = None   # 只在 skip_prefixes(…, expand_split=True) 下有：env 的 -S / --split-string 在词表里占的 [起, 止) 与它的字符串，由调用方拆开换进去再判
    environment_cleared: bool = False   # 路过的 env 带 -i / --ignore-environment / 单独的 -：外面带进来的变量与它之前的赋值都清掉，assignments 里只剩它之后的
    login_shell: bool = False          # 路过的 exec 带 -l、或 -a 给的名字以 - 起头：后面那个 shell 照登录 shell 起（读 /etc/profile 与 HOME 下的 profile）


class ShellInvocation(NamedTuple):
    code_string: str | None   # `bash -c '…'` 的那段代码
    script_index: int | None  # 起的是脚本时，脚本名在参数里的下标；只查语法（-n）或没给脚本时是 None
    runs_nothing: bool = False   # 照 bash 什么都不跑：带 -n（只查语法，-c 的代码、脚本、标准输入都不执行），或短选项之后又写长选项（bash 报 --: invalid option）


class PythonInvocation(NamedTuple):
    code_string: str | None   # `python3 -c '…'` 的那段代码
    module: str | None        # `python3 -m <模块>` 的模块名（`-m runpy <模块>` 取后面那个）
    script_index: int | None  # 起的是脚本文件时，脚本名在参数里的下标；-c / -m 时是程序自己的第一个参数的下标（可以等于参数个数）；
                              # 程序从标准输入读（`-`）或只有选项时是 None


class CommandAtPosition(NamedTuple):
    words: list[str]              # words[0] 是命令词（照写的样子，没取 basename），其后是参数
    directory: str | None         # 跑这条命令时的当前目录；cd 到展开不了的变量路径之后认不出，是 None（能展开的见 expand_known_variables）
    environment: dict[str, str]   # 这条命令看得到的、这段命令文本里设过的变量：前面 export 的、命令前与 env 里写的、外层传进来的
    launcher: str | None = None   # words[0] 是经什么起的脚本："shell"（bash / sh 这类后面跟的）、"source"（source / .）、
                                  # "python"（python3 后面跟的程序）、"startup"（shell 启动时先读的 BASH_ENV / ENV 指的文件，startup_file_commands）；
                                  # 直接执行的命令（含 capped.sh N 后面那条）是 None
    output_redirections: tuple = ()      # 这一条简单命令上的输出重定向：((运算符, 目标词), …)，运算符是 OUTPUT_REDIRECTS 之一；fd 号不在里面
    standard_input_heredocs: tuple = ()  # 喂给这一条的、被当数据剥掉的 heredoc：(HeredocBody, …)，按出现次序（最后一个才是它读到的标准输入）
    memory_capped: bool = False          # 经 run-with-memory-cap.sh 跑：它自己的包装里有，或外层（bash -c 的那段代码、起它所在脚本的那一条）有
    variables: dict | None = None        # 跑这条命令时这段命令文本里记下的变量（名字 → 已展开的值，None 是算不出）：调用方拿它给 expand_known_variables 展开参数
    piped_from: tuple | None = None      # 管道前一段：(词, 喂给它的 heredoc 正文)；不在管道后面是 None（认 `cat 源 | tee 目的` 这类拷出用）
    input_redirections: tuple = ()       # 这一条上改标准输入的重定向 ((运算符, 目标词), …)，STANDARD_INPUT_REDIRECTS 里的、heredoc 除外（认 `tee 目的 < 源` 用）
    wrappers: tuple = ()                 # 剥掉的 capped.sh / run-with-memory-cap.sh：((照写的命令词, 那一刻的当前目录), …)，外层（bash -c 的那段代码所在那一条）剥的也在里面；
                                         # 调用方核它是不是本仓那一份（project-code-guard.sh：路径尾部相同的别处拷贝照读正文、不算经内存包装）
    cd_failure_skips: bool = False       # 前面每一处可能失败的 cd / pushd（目标盘上不在）失败了，这一条都不会跑（`cd 目录 && 这一条`、`cd 目录 || exit` 之后同一层的）：
                                         # 目录盘上不在时调用方照 cd 成功了的那个目录判；False 是认不准，照「目录不在、cd 会失败」办
    interactive: bool = False            # 跑这一条的 shell 可能是交互的（带 -i，或不带 -c、不带脚本、从标准输入读）：它 source 的文件里 `case $- in *i*)…return` 这类守卫不截
                                         # （non_interactive_return_offset）


class ShellScan(NamedTuple):
    commands: list[CommandAtPosition]  # 命令位置上的每一条命令，包装（bash -c、capped.sh、source……）剥掉之后的那一条
    prefix_names: set[str]             # 哪一层、哪一条命令路过的前缀名都算
    has_lone_ampersand: bool           # 哪一层有单独的 &（放后台；不是 &&、>&、&>、|&）
    output_targets: list[str]          # 哪一层的输出重定向目标都算
    unjudged_shell_inputs: tuple = ()  # 从标准输入读命令的 shell 收到的、这里判不了的输入（每处一句说明，哪一层的都算；shell_standard_input）：
                                       # 调用方按自己的「判不了」处置（放行并记检出）


def heredoc_feeds_shell(head):
    """heredoc 所在那一行 `<<` 之前的文字：它挂在的那条简单命令（最后一条），剥掉前缀与 capped.sh N 之后，
    是不是从标准输入读命令的 bash / sh 这类（不带 -c、不带脚本，或带 -s）。同一行别处出现 shell 的名字（`x.sh`、前一条 bash）不算。
    `<<` 落在一个还没收尾的命令替换或进程替换里面（`x=$(bash <<EOF`、`diff <(bash <<EOF`、反引号同）时，挂的命令在那一层里找（open_substitution_tail）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=heredoc-open-substitution-unseen 下照旧拿整行切（替换里的 bash 认不出，正文当数据剥掉）。"""
    if not words_break_is_set("heredoc-open-substitution-unseen"):
        head = open_substitution_tail(head)
    simple = simple_commands(shell_tokens(head))[0]
    if not simple or not simple[-1]:
        return False
    words = simple[-1]
    for _ in range(MAXIMUM_NESTING):
        skipped, words = skip_prefixes_expanding(words)
        if skipped.command_position is None:
            return False
        words = words[skipped.command_position:]
        name, arguments = os.path.basename(words[0]), words[1:]
        if name in WRAPPERS_TAKING_ONE_ARGUMENT and len(arguments) >= 2 and not arguments[0].startswith("-"):
            words = arguments[1:]
            continue
        if not is_shell_name(name):
            return False
        return shell_reads_standard_input(arguments)
    return False


def open_substitution_tail(text):
    """text 末尾还开着的最里层命令替换或进程替换（`$(`、`<(`、`>(`、反引号；收尾了的、引号里的括号不算）之后的文字；都收尾了交回 text 原样。
    引号、反斜杠、替换的收尾与 shell_tokens 用的是同一套扫描（command_substitution_end、double_quote_end、backquote_end，没配上时由它们记下最里层还开着的正文从哪起）：
    双引号里收了尾的 `$(…)` 不算开着（`cd "$(git rev-parse --show-toplevel)" && bash <<EOF` 挂的是顶层的 bash）；`<(`、`>(` 只在引号外认，词首的 `#` 起到行尾是注释。
    弄坏开关 LIB_SHELL_WORDS_BREAK=double-quoted-substitution-unclosed 下双引号里的 `$(` 一律当没收尾。"""
    position, at_word_start = 0, True
    while position < len(text):
        character = text[position]
        unclosed, end = [], None
        if character == "\\":
            position, at_word_start = position + 2, False
            continue
        if text.startswith("$'", position) and not ansi_c_quotes_unseen():
            end = ansi_c_quote_end(text, position + 2)   # ANSI-C 引号里的 `\'` 不收尾
        elif character == "'":
            end = text.find("'", position + 1)
            end = -1 if end < 0 else end + 1
        elif character == '"':
            if words_break_is_set("double-quoted-substitution-unclosed"):
                closing = text.find('"', position + 1)
                opening = text.find("$(", position + 1, len(text) if closing < 0 else closing)
                if opening >= 0:
                    return text[opening + 2:]
            end = double_quote_end(text, position + 1, unclosed)
        elif text.startswith("$(", position) or text.startswith(("<(", ">("), position):
            end = command_substitution_end(text, position + 2, unclosed)
        elif character == "`":
            end = backquote_end(text, position + 1, unclosed)
        elif character == "#" and at_word_start:
            return text
        if end is None:
            at_word_start = character in " \t\r\n;&|()<>"
            position += 1
            continue
        if end < 0:
            return text[unclosed[0]:] if unclosed else text
        position, at_word_start = end, False
    return text


def shell_reads_standard_input(arguments):
    """bash / sh 这类后面的参数：是不是从标准输入读命令（不带 -c、不带脚本，或带 -s）。heredoc 喂给 shell 与 here-string、管道喂给 shell 共用这一判。"""
    invocation = shell_invocation(arguments)
    if invocation.code_string is not None or invocation.runs_nothing:
        return False
    reads_standard_input = any(argument.startswith("-") and not argument.startswith("--") and "s" in argument[1:]
                               for argument in arguments)
    return reads_standard_input or invocation.script_index is None


def strip_data_heredocs(command, keep_line_count=False, stripped_bodies=None, shell_heredocs=None):
    """喂给非 shell 命令的 heredoc 正文剥掉（它是数据，不是在跑的命令）；喂给 bash / sh 这类的照留（判据在 heredoc_feeds_shell）。
    keep_line_count 为真时正文换成同样多的空行，剥完之后每一行的行号不变（报脚本里第几行要用）。
    stripped_bodies 是一个列表时，每剥掉一段就把它的 HeredocBody 追加进去，剥完的文字里结束词换成 HEREDOC_BODY_MARKER 那种记号、带上它在列表里的下标。
    shell_heredocs 是一个列表时，喂给 shell 的那些也剥掉，(它所在那一行 `<<` 之前的文字, 正文) 按次序追加进去，由调用方另判
    （正文里的命令算不算在挂它的那条命令的包装里，比如外面有没有 timeout）。"""
    def replace(match):
        head = command[command.rfind("\n", 0, match.start()) + 1:match.start()]
        line_breaks = "\n" * match.group(0).count("\n") if keep_line_count else "\n"
        delimiter = f"{match.group(1)}{match.group(2)}{match.group(1)}"
        if heredoc_feeds_shell(head):
            if shell_heredocs is None:
                return match.group(0)
            shell_heredocs.append((head, match.group(4)))
        elif stripped_bodies is not None:
            operator = "<<-" if match.group(0).startswith("<<-") else "<<"
            body = match.group(4)
            if operator == "<<-":
                body = "\n".join(line.lstrip("\t") for line in body.split("\n"))
            delimiter = f"\x00heredoc-body-{len(stripped_bodies)}\x00"
            stripped_bodies.append(HeredocBody(operator, bool(match.group(1)), body + "\n"))
        return f"<<{delimiter}{match.group(3)}{line_breaks}{match.group(2)}"
    return HEREDOC.sub(replace, command)


ANSI_C_ESCAPES = {"a": "\a", "b": "\b", "e": "\x1b", "E": "\x1b", "f": "\f", "n": "\n", "r": "\r", "t": "\t", "v": "\v",
                  "\\": "\\", "'": "'", '"': '"', "?": "?"}


def ansi_c_quotes_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=ansi-c-quotes-unseen：`$'…'` 照普通单引号配对（里面的 `\\'` 不认；只给自检证红用）。"""
    return words_break_is_set("ansi-c-quotes-unseen")


def ansi_c_quote_end(text, start):
    """ANSI-C 引号 `$'…'` 里（start 指向开头那个单引号后面第一个字符）找收尾的单引号，交回它后面那个下标；里面反斜杠转义下一个字符（`\\'` 不收尾）；没配上交 -1。
    调用方在弄坏开关 ansi-c-quotes-unseen 下不调它（`$` 当普通字符、后面的单引号照普通单引号配对）。"""
    position = start
    while position < len(text):
        if text[position] == "\\":
            position += 2
        elif text[position] == "'":
            return position + 1
        else:
            position += 1
    return -1


def ansi_c_decoded(body):
    """`$'…'` 里的正文照 bash 换掉转义：\\a \\b \\e \\E \\f \\n \\r \\t \\v \\\\ \\' \\" \\?、\\nnn（八进制一到三位）、\\xHH、\\uHHHH、\\UHHHHHHHH、\\cX；
    认不出的照原样留下反斜杠与那个字符；换出 NUL 的，bash 在那里截断。"""
    pieces, index = [], 0
    while index < len(body):
        character = body[index]
        following = body[index + 1:index + 2]
        if character != "\\" or not following:
            pieces.append(character)
            index += 1
            continue
        octal = re.match(r"[0-7]{1,3}", body[index + 1:])
        hexadecimal = {"x": 2, "u": 4, "U": 8}.get(following)
        digits = re.match(r"[0-9A-Fa-f]{1,%d}" % hexadecimal, body[index + 2:]) if hexadecimal else None
        if following in ANSI_C_ESCAPES:
            pieces.append(ANSI_C_ESCAPES[following])
            index += 2
        elif octal:
            pieces.append(chr(int(octal.group(0), 8) & 0xFF))
            index += 1 + len(octal.group(0))
        elif digits:
            value = int(digits.group(0), 16)
            pieces.append(chr(value) if value <= 0x10FFFF else "�")
            index += 2 + len(digits.group(0))
        elif following == "c" and index + 2 < len(body):
            pieces.append(chr(ord(body[index + 2]) & 0x1F))
            index += 3
        else:
            pieces.append(character + following)
            index += 2
    return "".join(pieces).split("\x00", 1)[0]


def backquote_end(text, start, unclosed=None):
    """反引号之后（start 指向开头那个反引号后面第一个字符）找收尾的反引号，交回它后面那个下标；没配上交 -1。
    unclosed 是一个列表、而这一段没配上时，往里追加还开着的最里层替换的正文从哪起（先追加的是最里层；open_substitution_tail 用）。"""
    position = start
    while position < len(text):
        if text[position] == "\\":
            position += 2
        elif text[position] == "`":
            return position + 1
        else:
            position += 1
    if unclosed is not None:
        unclosed.append(start)
    return -1


def double_quote_end(text, start, unclosed=None):
    """双引号之后找收尾的双引号，交回它后面那个下标；里面的 `$(…)` 与反引号整段跳过（它们里面的双引号不算收尾）；没配上交 -1。unclosed 同 backquote_end。"""
    position = start
    while position < len(text):
        character = text[position]
        if character == "\\":
            position += 2
        elif character == '"':
            return position + 1
        elif text.startswith("$(", position):
            position = command_substitution_end(text, position + 2, unclosed)
        elif character == "`":
            position = backquote_end(text, position + 1, unclosed)
        else:
            position += 1
        if position < 0:
            return -1
    return -1


def substitution_case_and_comment_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen：命令替换里不认 case 的模式与注释，模式的 `)`、注释里的 `)` 照括号配对（只给自检证红用）。"""
    return words_break_is_set("substitution-case-and-comment-unseen")


SUBSTITUTION_WORD_END = set(" \t\r\n;&|()<>")
SUBSTITUTION_COMMAND_KEYWORDS = {"do", "then", "else", "elif", "if", "while", "until", "!", "{", "time", "coproc"}


def command_substitution_end(text, start, unclosed=None):
    """`$(` 之后（start 指向它后面第一个字符）找配对的 `)`，交回它后面那个下标；引号、反斜杠、反引号与嵌套的括号都跳过；没配上交 -1。unclosed 同 backquote_end。
    照 bash 认两处不配对的右括号（`echo "$(case a in a) cargo test --all;; esac)"` 不在 `a)` 处收尾，否则 cargo 漏判）：
    词首的 `#` 起到行尾是注释（里面的 `)` 不算）；命令位置上的 `case 词 in` 之后、`;;` / `;&` / `;;&` 之后到右括号为止是模式，模式的 `(`、`)` 不算括号，
    `esac` 收尾（case 可以套 case）。弄坏开关 LIB_SHELL_WORDS_BREAK=substitution-case-and-comment-unseen 下不认这两处。"""
    depth, position = 1, start
    aware = not substitution_case_and_comment_unseen()
    cases = []                 # 套着的 case：每层 "subject"（等 in）、"pattern"（在模式里）、"body"（在那一项的命令里）
    command_start = True       # 下一个词在命令位置上
    while position < len(text):
        character = text[position]
        word_start = position == start or text[position - 1] in SUBSTITUTION_WORD_END
        if aware and word_start and character == "#":
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
            continue
        if aware and word_start and (character.isalpha() or character == "_"):
            end = position
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            if end == len(text) or text[end] in SUBSTITUTION_WORD_END:
                word = text[position:end]
                if word == "case" and command_start:
                    cases.append("subject")
                elif word == "in" and cases and cases[-1] == "subject":
                    cases[-1] = "pattern"
                    position, command_start = end, False
                    continue
                elif word == "esac" and cases and (command_start or cases[-1] == "pattern"):
                    cases.pop()
                    position, command_start = end, False
                    continue
                command_start = word in SUBSTITUTION_COMMAND_KEYWORDS
                position = end
                continue
        if aware and cases and cases[-1] == "body" and text.startswith(";;", position) or aware and cases and cases[-1] == "body" and text.startswith(";&", position):
            cases[-1] = "pattern"
            position += 3 if text.startswith(";;&", position) else 2
            command_start = False
            continue
        if aware and cases and cases[-1] == "pattern" and character in "()":
            if character == ")":
                cases[-1] = "body"
                command_start = True
            position += 1
            continue
        if character in ";&|(\n":
            command_start = True
        elif character not in " \t\r":
            command_start = False
        if character == "\\":
            position += 2
        elif text.startswith("$'", position) and not ansi_c_quotes_unseen():
            position = ansi_c_quote_end(text, position + 2)   # ANSI-C 引号里的 `\'` 不收尾
        elif character == "'":
            end = text.find("'", position + 1)
            position = -1 if end < 0 else end + 1
        elif character == '"':
            position = double_quote_end(text, position + 1, unclosed)
        elif character == "`":
            position = backquote_end(text, position + 1, unclosed)
        elif character == "(":
            depth += 1
            position += 1
        elif character == ")":
            depth -= 1
            position += 1
            if depth == 0:
                return position
        else:
            position += 1
        if position < 0:
            break
    if unclosed is not None:
        unclosed.append(start)
    return -1


def starts_substitution(text, position):
    """text[position:] 是不是以命令替换（`$(`、反引号）或进程替换（`<(`、`>(`）开头；引号外用，双引号里只有前两种算。
    `>>(`、`&>(`、`<<(` 这类运算符在前的不在内：切词在前一个字符上就把运算符认走了（bash 也按运算符读，后面的 `(` 是语法错）。"""
    return text.startswith(("$(", "<(", ">("), position) or text.startswith("`", position)


def substitution_span(text, position):
    """text[position:] 以 `$(`、`$((`、`<(`、`>(` 或反引号开头：交回 (这一段结束的下标, 里面要当命令判的正文或 None)。
    算术 `$((…))` 的正文不是命令，交 None；进程替换 `<(…)`、`>(…)` 的正文是命令（`<((…))` 是里面套了一层子 shell，不是算术）；没配上的，剩下的整段都算它。"""
    if text.startswith("`", position):
        end = backquote_end(text, position + 1)
        return (len(text), text[position + 1:]) if end < 0 else (end, text[position + 1:end - 1])
    end = command_substitution_end(text, position + 2)
    inner = text[position + 2:] if end < 0 else text[position + 2:end - 1]
    return (len(text) if end < 0 else end), (None if text.startswith("$((", position) else inner)


def shell_tokens(text):
    """切成 (是不是运算符, 文本)：引号去掉、词首的 # 注释去掉、重定向前的 fd 号去掉；引号没配对时剩下的当一个词。
    命令替换（`$(…)`、反引号，引号里外都算）与进程替换（`<(…)`、`>(…)`，只在引号外算，在词首、词中都算）照原样留在所在的词里，
    里面的正文作为 (None, 正文) 紧跟在那个词后面交出；`)` 之后紧跟的字符接在同一个词上，隔了空白的是下一个词。
    引号外的花括号照 bash 展开成几个词（`cp X{,.bak}` 是 `cp X X.bak`；`{a,b}`、`{1..3}`、`{a..c}`，嵌套的照展，`${…}` 不算；brace_expand），
    整词像赋值（`名字=…`）的不展开（bash 不对赋值词做花括号展开）。弄坏开关 LIB_SHELL_WORDS_BREAK=braces-unexpanded 下照旧整个留一个词。"""
    tokens, word, mask, position, pending_substitutions = [], None, "", 0, []
    def add(piece, quoted):
        nonlocal word, mask
        if quoted and not piece:
            piece, quoted = QUOTED_EMPTY, None   # 引号写出的空串：占一个位置（掩码 "e"），花括号展开之后据它判这一份是不是引号写出的空串，再去掉
        word = (word or "") + piece
        mask += {True: "q", False: "u", None: "e"}[quoted] * len(piece)
    def flush():
        nonlocal word, mask
        if word is not None:
            expansions = brace_expand(word, mask)
            target_of = tokens[-1][1] if tokens and tokens[-1][0] is True and tokens[-1][1] in REDIRECT_OPERATORS else None
            if (target_of is not None and (target_of in ("<<", "<<-", "<<<") or len(expansions) != 1)
                    and not words_break_is_set("redirect-target-braces-expanded")):
                # 重定向的目标只有一个词：here-string 与 heredoc 的结束词 bash 不做花括号展开；别的展开成几个词（或一个不剩）bash 报 ambiguous redirect、什么都不写。
                # 都照原样留一个词：拷出再执行那一类认不出这个落点（不当写出了文件），写范围那一类照「落点算不出」拒（偏严，bash 本来也不写）。
                # 弄坏开关 redirect-target-braces-expanded 下照旧展开（多出来的词成了这一条的参数）
                expansions = ["".join(character for character, kind in zip(word, mask) if kind != "e")]
            tokens.extend((False, expanded) for expanded in expansions)
            word, mask = None, ""
        tokens.extend((None, inner) for inner in pending_substitutions)
        pending_substitutions.clear()
    def take_substitution(start):
        end, inner = substitution_span(text, start)
        if inner is not None:
            pending_substitutions.append(inner)
        return end
    while position < len(text):
        character = text[position]
        if character in " \t\r":
            flush()
            position += 1
        elif character == "\\":
            if not text.startswith("\\\n", position):
                add(text[position + 1:position + 2], True)
            position += 2
        elif character == "'":
            end = text.find("'", position + 1)
            end = len(text) if end < 0 else end
            add(text[position + 1:end], True)
            position = end + 1
        elif character == '"':
            position += 1
            piece = ""
            while position < len(text) and text[position] != '"':
                if text[position] == "\\" and text[position + 1:position + 2] in ("$", "`", '"', "\\", "\n"):
                    piece += text[position + 1]
                    position += 2
                elif text.startswith("$(", position) or text[position] == "`":
                    end = take_substitution(position)
                    piece += text[position:end]
                    position = end
                else:
                    piece += text[position]
                    position += 1
            add(piece, True)
            position += 1
        elif character == "#" and word is None:
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
        elif text.startswith("$'", position) and not ansi_c_quotes_unseen():
            # ANSI-C 引号：反斜杠转义照 bash 换掉（`$'a\'b'` 是 a'b、`$'\x63argo'` 是 cargo），整段算引号里的（不做花括号展开、不当 fd 号）
            end = ansi_c_quote_end(text, position + 2)
            end = len(text) + 1 if end < 0 else end
            add(ansi_c_decoded(text[position + 2:end - 1]), True)
            position = end
        elif starts_substitution(text, position):
            end = take_substitution(position)
            add(text[position:end], True)
            position = end
        elif character in OPERATOR_START:
            operator = next(candidate for candidate in SHELL_OPERATORS if text.startswith(candidate, position))
            # 紧贴在前面、整个都是引号外的十进制数字的词才是 fd 号（`2>`、`01>`）；引号或反斜杠写出的数字（`"2">`、`\2>`）是参数（弄坏开关 redirect-fd-quoted-digits 下照旧当 fd 号）
            if (operator[0] in "<>" and word is not None and re.fullmatch(r"[0-9]+", word)
                    and (set(mask) == {"u"} or words_break_is_set("redirect-fd-quoted-digits"))):
                if not words_break_is_set("redirect-fd-dropped"):
                    operator = RedirectOperator(operator, word)
                word, mask = None, ""
            flush()
            tokens.append((True, operator))
            position += len(operator)
        else:
            add(character, False)
            position += 1
    flush()
    return case_patterns_and_conditionals_folded(tokens)


class UnfoldableCase(Exception):
    """case 语句的模式切不出来（模式里夹了别的运算符、到头没有收尾的右括号）：整段照原样交回。"""


def case_patterns_and_conditionals_folded(tokens):
    """切好的记号里，bash 不当命令跑的两处照 bash 改写，免得被当成命令位置：
    ① case 语句（命令位置上的 `case 词 in`）的模式：`in` 之后、`;;` / `;&` / `;;&` 之后到右括号为止的词连同 `|`、开头可写的左括号都去掉，
       收尾的右括号换成一个 `;`（模式之后是那一项的命令，在命令位置上）；模式里的命令替换照留（bash 展开模式时照跑它），`;;&` 后面的 `&` 不当放后台；
       命令位置上的 `esac` 收尾，case 可以套 case。弄坏开关 LIB_SHELL_WORDS_BREAK=case-patterns-as-commands 下不改（`;;` 之后的模式词当命令词）。
    ② 条件表达式（命令位置上的 `[[ … ]]`）里的 `&&`、`||`、`(`、`)`、`<`、`>` 是表达式的一部分，不是命令分隔与重定向：换成普通的词（`2>` 那种 fd 号还原成一个词），
       里面的换行去掉，整段是 `[[` 这一条命令的参数。没有收尾的 `]]` 不改。弄坏开关 LIB_SHELL_WORDS_BREAK=conditional-expression-split 下不改。
    模式切不出来的（夹了别的运算符、到头没有收尾的右括号）整段照原样交回。"""
    fold_cases = not words_break_is_set("case-patterns-as-commands")
    fold_conditionals = not words_break_is_set("conditional-expression-split")
    if not (fold_cases and (False, "case") in tokens) and not (fold_conditionals and (False, "[[") in tokens):
        return tokens
    output, index, at_command, cases = [], 0, True, []   # cases：每层没收尾的 case 此刻在模式里（"pattern"）还是在一项的命令里（"body"）
    try:
        while index < len(tokens):
            kind, value = tokens[index]
            if cases and cases[-1] == "pattern":
                if (kind, value) == (True, "\n"):
                    output.append((kind, value))
                    index += 1
                    continue
                if (kind, value) == (False, "esac") and tokens[index + 1:index + 2] not in ([(True, ")")], [(True, "|")]):
                    cases.pop()
                    output.append((kind, value))
                    index, at_command = index + 1, False
                    continue
                if (kind, value) == (True, "("):
                    index += 1
                while index < len(tokens) and tokens[index] != (True, ")"):
                    if tokens[index][0] is None:
                        output.append(tokens[index])
                    elif tokens[index][0] is True and tokens[index][1] != "|":
                        raise UnfoldableCase()
                    index += 1
                if index >= len(tokens):
                    raise UnfoldableCase()
                output.append((True, ";"))
                cases[-1] = "body"
                index, at_command = index + 1, True
                continue
            if kind is False and at_command and value == "[[" and fold_conditionals:
                closing = next((position for position in range(index + 1, len(tokens)) if tokens[position] == (False, "]]")), None)
                if closing is not None:
                    output.append((kind, value))
                    for inner_kind, inner_value in tokens[index + 1:closing]:
                        if inner_kind is not True:
                            output.append((inner_kind, inner_value))
                        elif inner_value != "\n":
                            if getattr(inner_value, "fd", None) is not None:
                                output.append((False, inner_value.fd))
                            output.append((False, str(inner_value)))
                    output.append(tokens[closing])
                    index, at_command = closing + 1, False
                    continue
            if kind is False and at_command and value == "case" and fold_cases and index + 1 < len(tokens) and tokens[index + 1][0] is False:
                cursor = index + 2
                while cursor < len(tokens) and (tokens[cursor][0] is None or tokens[cursor] == (True, "\n")):
                    cursor += 1
                if cursor < len(tokens) and tokens[cursor] == (False, "in"):
                    output.extend(tokens[index:cursor + 1])
                    cases.append("pattern")
                    index, at_command = cursor + 1, False
                    continue
            if cases and (kind, value) == (True, ";;"):
                output.append((kind, value))
                index += 1
                if tokens[index:index + 1] == [(True, "&")]:
                    index += 1   # `;;&`：接着试后面的模式，不是放后台
                cases[-1], at_command = "pattern", True
                continue
            if cases and (kind, value) == (True, ";") and tokens[index + 1:index + 2] == [(True, "&")]:
                output.append((True, ";;"))   # `;&`：落到下一项的命令接着跑，下一项照样有模式
                index += 2
                cases[-1], at_command = "pattern", True
                continue
            if cases and kind is False and at_command and value == "esac":
                cases.pop()
                output.append((kind, value))
                index, at_command = index + 1, False
                continue
            output.append((kind, value))
            if kind is True:
                at_command = value in COMMAND_SEPARATORS
            elif kind is False:
                at_command = at_command and value in COMMAND_KEYWORDS
            index += 1
    except UnfoldableCase:
        return tokens
    return output


BRACE_EXPANSION_LIMIT = 64   # 一个词最多展开成这么多个；再多的照原样留一个词（bash 照展，这里只防判定卡住）
BRACE_SEQUENCE = re.compile(r"([+-]?[0-9]+|[A-Za-z])\.\.([+-]?[0-9]+|[A-Za-z])(?:\.\.([+-]?[0-9]+))?")   # 整数起止、步长认带 + 号的（bash 也认）；匹配用 fullmatch


QUOTED_EMPTY = "\x02"   # 引号写出的空串（`''`、`""`）在切词时占的一个位置（掩码 "e"），brace_expand 交回之前去掉


class ShellWord(str):
    """切好的一个词，带着它里面还留着的花括号是不是都在引号里（`'echo a{,}'` 的花括号是数据，不会再展开成别的）：与词本身逐字相等，按字面比的地方照旧；
    unquoted_braces 为真是还留着引号外的 `{` 或 `}`（`{}`、超过 BRACE_EXPANSION_LIMIT 照原样留的、`${…}`），不是 ShellWord 的普通 str 一律当为真。"""
    unquoted_braces = True

    def __new__(cls, text, unquoted_braces):
        instance = super().__new__(cls, text)
        instance.unquoted_braces = unquoted_braces
        return instance

    def __reduce__(self):   # 拷贝、深拷贝、pickle 照带着这一格（不写的话 str 子类的 __new__ 少一个参数，copy.deepcopy 抛 TypeError）
        return (ShellWord, (str(self), self.unquoted_braces))


def quoted_braces_as_unknown():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=quoted-braces-as-unknown：切好的词不带引号信息，引号里的花括号也当「还留着、认不准」（只给自检证红用）。"""
    return words_break_is_set("quoted-braces-as-unknown")


def brace_expand(word, mask):
    """一个词的花括号展开：mask 与 word 等长，'u' 是引号外的字符、'q' 是引号里、转义过或替换正文里的（不参与展开），'e' 是引号写出的空串占的位置。
    交回展开出的词表（没有可展开的交 [word]），'e' 那几个位置去掉。展开出来的空串照 bash 去掉（`{cargo,} test` 是 `cargo test`、`{,}` 整个没了），
    这一份里有引号写出的空串的留着（`''{,}` 是两个空词、`{"",}` 是一个）；弄坏开关 LIB_SHELL_WORDS_BREAK=brace-empties-kept 下空串都留着。
    交回的每个词是 ShellWord：照掩码记下它还留着的花括号里有没有引号外的（函数调用认不认得准用它：function_never_called）。"""
    def without_quoted_empties(text, text_mask):
        return ShellWord("".join(character for character, kind in zip(text, text_mask) if kind != "e"),
                         quoted_braces_as_unknown() or any(character in "{}" and kind == "u" for character, kind in zip(text, text_mask)))
    if "{" not in word or words_break_is_set("braces-unexpanded") or ASSIGNMENT.match(word) or re.match(r"^[A-Za-z_][A-Za-z0-9_]*\+=", word):
        return [without_quoted_empties(word, mask)]
    results = brace_expand_once(word, mask, BRACE_EXPANSION_LIMIT)
    if not results or results == [(word, mask)]:
        return [without_quoted_empties(word, mask)]
    expanded = [(without_quoted_empties(text, text_mask), "e" in text_mask) for text, text_mask in results]
    return [text for text, quoted_empty in expanded if text or quoted_empty or words_break_is_set("brace-empties-kept")]


def brace_expand_once(word, mask, limit):
    """展开 word 里第一处成立的花括号（引号外的 `{`，与它配对的引号外的 `}` 之间顶层有引号外的逗号、或是 `a..b` 序列；`${` 起的跳过），
    再把每个结果接着展开；交回 [(词, 掩码)]，超过 limit 个交 None。"""
    index = 0
    while index < len(word):
        if word[index] == "{" and mask[index] == "u" and not (index > 0 and word[index - 1] == "$" and mask[index - 1] == "u"):
            depth, commas, cursor = 1, [], index + 1
            while cursor < len(word) and depth:
                if mask[cursor] == "u":
                    if word[cursor] == "{":
                        depth += 1
                    elif word[cursor] == "}":
                        depth -= 1
                    elif word[cursor] == "," and depth == 1:
                        commas.append(cursor)
                cursor += 1
            if depth == 0:
                close = cursor - 1
                prefix, suffix = (word[:index], mask[:index]), (word[close + 1:], mask[close + 1:])
                alternatives = None
                if commas:
                    bounds = [index] + commas + [close]
                    alternatives = [(word[bounds[i] + 1:bounds[i + 1]], mask[bounds[i] + 1:bounds[i + 1]]) for i in range(len(bounds) - 1)]
                else:
                    alternatives = brace_sequence(word[index + 1:close], mask[index + 1:close], limit)
                if alternatives is not None:
                    if len(alternatives) > limit:
                        return None
                    results = []
                    for alternative, alternative_mask in alternatives:
                        expanded = brace_expand_once(prefix[0] + alternative + suffix[0], prefix[1] + alternative_mask + suffix[1], limit)
                        if expanded is None or len(results) + len(expanded) > limit:
                            return None
                        results += expanded
                    return results
        elif word[index] == "{" and mask[index] == "u":
            closing = word.find("}", index)   # `${…}` 是变量，不是花括号展开：整段跳过
            index = len(word) if closing < 0 else closing
        index += 1
    return [(word, mask)]


def brace_sequence_width(first, last):
    """bash 的整数序列补零：起或止有一边写成 0 起头的两位以上（或 -0 起头的三位以上）时，每一份补零到起、止两边写法的长度里大的那个
    （`{01..03}`、`{-01..2}`；另一边没补零而写得更长的照它，`{01..100..3}` 补到三位，同 bash 5.2 的 expand_seqterm）；不补零交 0。
    brace_sequence 和 lib_project_code_runs.py 的花括号展开都调这一份。弄坏开关 LIB_SHELL_WORDS_BREAK=brace-sequence-unpadded 下一律交 0；
    brace-sequence-width-padded-ends-only 下只取补零写的那几边的长度。"""
    if words_break_is_set("brace-sequence-unpadded"):
        return 0
    padded = [len(text) for text in (first, last) if (len(text) > 1 and text[0] == "0") or (len(text) > 2 and text[:2] == "-0")]
    if not padded:
        return 0
    return max(padded) if words_break_is_set("brace-sequence-width-padded-ends-only") else max(len(first), len(last))


def brace_sequence(inner, inner_mask, limit):
    """`{1..3}`、`{a..c}`、`{1..10..2}`、`{+1..3}` 的展开（全部在引号外才算；整数照 bash 补零：brace_sequence_width）：交回 [(词, 掩码)]，
    最多 limit + 1 份：交回的比 limit 多，就是展开出来超过了 limit（多出来的不是全部），超过了怎么办由调用方定；不是序列交 None。"""
    match = BRACE_SEQUENCE.fullmatch(inner) if "q" not in inner_mask else None
    if match is None:
        return None
    first, last, step = match.group(1), match.group(2), abs(int(match.group(3) or 1)) or 1
    numeric = first[-1].isdigit()
    if numeric != last[-1].isdigit():
        return None
    low, high = (int(first), int(last)) if numeric else (ord(first), ord(last))
    direction = 1 if high >= low else -1
    values = range(low, high + direction, step * direction)[:limit + 1]
    width = brace_sequence_width(first, last) if numeric else 0
    texts = [(f"{value:0{width}d}" if width else str(value)) if numeric else chr(value) for value in values]
    return [(text, "u" * len(text)) for text in texts]


def simple_commands(tokens, separators=None, boundaries=None):
    """按命令边界切成一条条简单命令（只留词，重定向连同目标去掉），另交整段里输出重定向的目标，
    每条简单命令里的命令替换正文，以及每条简单命令上的重定向 ((运算符, 目标词), …)（后两样与命令一一对应；只有重定向、没有词的那一条，词是空列表）。
    separators 是一个列表时，按命令的次序追加每一条前面的那个分隔符（第一条是 None；`|` 之后换行的，下一条照旧记 `|`），调用方拿它认管道后一段。
    boundaries 是一个列表时，按命令的次序追加每一条与前一条之间的全部分隔符（元组，按出现次序；括号、换行都在里面），调用方拿它认 cd 失败之后哪几条不跑。"""
    commands, substitutions, current, current_substitutions, output_targets = [], [], [], [], []
    redirections, current_redirections = [], []
    preceding = None
    between = []   # 上一条之后到这里的分隔符
    position = 0
    while position < len(tokens):
        is_operator, text = tokens[position]
        if (is_operator is False and ARRAY_ASSIGNMENT_START.match(text) and position + 1 < len(tokens)
                and tokens[position + 1] == (True, "(")):
            # 数组赋值 `名字=(元素 …)`：元素是值，不是命令；整段收成一个赋值词
            elements, position = [], position + 2
            while position < len(tokens) and tokens[position] != (True, ")"):
                if tokens[position][0] is None:
                    current_substitutions.append(tokens[position][1])
                elif tokens[position][0] is False:
                    elements.append(tokens[position][1])
                position += 1
            current.append(f"{text}({' '.join(elements)})")
            position += 1
            continue
        if is_operator is None:
            current_substitutions.append(text)
        elif is_operator and text in COMMAND_SEPARATORS:
            if current or current_substitutions:
                commands.append(current)
                substitutions.append(current_substitutions)
                redirections.append(current_redirections)
                if separators is not None:
                    separators.append(preceding)
                if boundaries is not None:
                    boundaries.append(tuple(between))
                between = []
                preceding = text
            elif not (text == "\n" and preceding in ("|", "|&")):
                preceding = text
            between.append(text)
            current, current_substitutions, current_redirections = [], [], []
        elif is_operator:
            if position + 1 < len(tokens) and tokens[position + 1][0] is False:
                if text in OUTPUT_REDIRECTS:
                    output_targets.append(tokens[position + 1][1])
                current_redirections.append((text, tokens[position + 1][1]))
                position += 1
        else:
            current.append(text)
        position += 1
    if current or current_substitutions:
        commands.append(current)
        substitutions.append(current_substitutions)
        redirections.append(current_redirections)
        if separators is not None:
            separators.append(preceding)
        if boundaries is not None:
            boundaries.append(tuple(between))
    return commands, output_targets, substitutions, redirections


def words_break_is_set(item):
    """弄坏开关 LIB_SHELL_WORDS_BREAK 里有没有这一项（几个用逗号连；只给自检证红用）。"""
    return item in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_chdir_ignored():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-chdir-ignored：env 的 -C / --chdir 换的目录不跟（按外面的当前目录判，只给自检证红用）。"""
    return "env-chdir-ignored" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_directory(directory, target, variables=None):
    """env -C 的目标解开成绝对路径：相对路径按命令开始时的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的）；
    展开不了（没赋过的变量、命令替换）或当前目录认不出时交 directory 原样（照旧按外面的当前目录判，
    不像 cd 那样交 None：交 None 会让这一条命令的目录类判定全部放过，比交 directory 原样还松；罩不住的：env -C 的目录写在展开不了的变量里）。"""
    expanded = os.path.expanduser(target)
    if "$" in expanded or "`" in expanded:
        expanded = expand_known_variables(expanded, variables or {})
        if expanded is None:
            return directory
    return resolve_path(directory, expanded) or directory


class EnvOption(NamedTuple):
    width: int                     # 这个选项连同它的值词占几个词
    chdir: str | None = None       # -C / --chdir 的目录
    split_string: str | None = None   # -S / --split-string 的字符串


ENV_SHORT_OPTIONS_WITH_VALUE = "uCS"                                # env 的 -u NAME、-C 目录、-S 字符串；短选项可以合写（-iC 目录、-iS 字符串、-0u NAME），值贴着写或另起一个词
ENV_LONG_OPTIONS_WITH_VALUE = ("chdir", "unset", "split-string")    # getopt_long 认唯一前缀的缩写（--chd 目录、--spl=字符串、--u NAME）；env 别的长选项没有值词


def env_option_clusters_unparsed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-option-clusters-unparsed：env 的选项退回只认整词 -u / --unset / -C / --chdir（合写的 -iC、缩写的 --chd、单独的 - 与 -- 当成不带值的开关或命令词；只给自检证红用）。"""
    return "env-option-clusters-unparsed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_split_string_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded：env 的 -S / --split-string 不拆（-S 当不带值的开关，后面那个词整个当命令词；只给自检证红用）。"""
    return "env-split-string-unexpanded" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def env_option(words, position):
    """env 的第 position 个词若是选项（含合写的短选项簇）交 EnvOption，不是（赋值、命令词）交 None。
    单独一个 `-` 等于 -i，`--` 之后不再有选项（由调用方停手）。带值的：-u、-C、-S（短选项簇里头一个带值的字母之后，贴着的是值、没有就取下一个词）和
    --unset / --chdir / --split-string（`=值` 或下一个词，认唯一前缀的缩写）；别的字母与长选项（-i、-0、-v、--block-signal[=SIG]……）不带值词。"""
    argument = words[position]
    if argument in ("-", "--"):
        return EnvOption(1)
    if argument.startswith("--"):
        name, equals, value = argument[2:].partition("=")
        matched = next((long for long in ENV_LONG_OPTIONS_WITH_VALUE if name and long.startswith(name)), None)
        if matched is None:
            return EnvOption(1)
        width = 1
        if not equals:
            value, width = (words[position + 1], 2) if position + 1 < len(words) else (None, 1)
        return EnvOption(width, value if matched == "chdir" else None, value if matched == "split-string" else None)
    if not argument.startswith("-"):
        return None
    letters = argument[1:]
    for index, letter in enumerate(letters):
        if letter in ENV_SHORT_OPTIONS_WITH_VALUE:
            value, width = letters[index + 1:], 1
            if not value:
                value, width = (words[position + 1], 2) if position + 1 < len(words) else (None, 1)
            return EnvOption(width, value if letter == "C" else None, value if letter == "S" else None)
    return EnvOption(1)


def env_option_clears_environment(argument):
    """env 的这个选项词清不清掉整个环境：单独的 `-`、`--ignore-environment`（认唯一前缀的缩写 `--i`、`--ign`……）、短选项簇里在头一个带值的字母之前有 i（`-i`、`-iC 目录`、`-0i`）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=env-ignore-environment-kept 下调用方不认（env -i 之前的赋值照旧算在后面那条命令上）。"""
    if argument == "-":
        return True
    if argument.startswith("--"):
        name = argument[2:].partition("=")[0]
        return bool(name) and "ignore-environment".startswith(name) and not any(long.startswith(name) for long in ENV_LONG_OPTIONS_WITH_VALUE)
    if not argument.startswith("-"):
        return False
    for letter in argument[1:]:
        if letter == "i":
            return True
        if letter in ENV_SHORT_OPTIONS_WITH_VALUE:
            return False
    return False


ENV_SPLIT_ESCAPES = {"n": "\n", "t": "\t", "f": "\f", "r": "\r", "v": "\v"}


def env_split_string(text):
    """GNU env -S 的拆词：空白（空格、制表、换行、\\v、\\f、\\r）与不在引号里的 `\\_` 隔开词；单引号里只认 `\\'` 与 `\\\\`；双引号里认 \\n \\t \\f \\r \\v \\_（空格）与 \\" \\$ \\\\；
    不在引号里的词首 `#` 起到结尾是注释；`\\c` 到此为止；`$NAME`、`${NAME}` 不展开（照原文留着）；不认的反斜杠转义（env 自己会报错不跑）宽松地留下那个字符。"""
    words, current, started, quote, index = [], [], False, None, 0
    while index < len(text):
        character = text[index]
        if quote == "'":
            if character == "'":
                quote = None
            elif character == "\\" and text[index + 1:index + 2] in ("'", "\\"):
                index += 1
                current.append(text[index])
            else:
                current.append(character)
        elif character == "\\":
            index += 1
            following = text[index:index + 1]
            if following == "c":
                break
            if following == "_" and quote is None:
                if started:
                    words.append("".join(current))
                current, started = [], False
            elif following == "_":
                current.append(" ")
            elif following in ENV_SPLIT_ESCAPES:
                current.append(ENV_SPLIT_ESCAPES[following])
                started = True
            elif following:
                current.append(following)
                started = True
            else:
                current.append("\\")
                started = True
        elif quote == '"':
            if character == '"':
                quote = None
            else:
                current.append(character)
        elif character in "'\"":
            quote, started = character, True
        elif character in " \t\n\v\f\r":
            if started:
                words.append("".join(current))
            current, started = [], False
        elif character == "#" and not started:
            break
        else:
            current.append(character)
            started = True
        index += 1
    if started:
        words.append("".join(current))
    return words


def skip_prefixes_expanding(words):
    """skip_prefixes，env 的 -S / --split-string 先按 env 的规则拆成词、原地换进命令里再判（拆出来的词里还可以有选项、赋值与命令词，MAXIMUM_NESTING 层为限）；
    交回 (PrefixSkip, 换完之后的词表)：命令词的下标指的是换完之后的词表。（弄坏开关 LIB_SHELL_WORDS_BREAK=env-split-string-unexpanded 下不拆。）"""
    for _ in range(MAXIMUM_NESTING):
        skipped = skip_prefixes(words, expand_split=True)
        if skipped.split is None:
            return skipped, words
        start, end, text = skipped.split
        words = words[:start] + env_split_string(text) + words[end:]
    return skip_prefixes(words), words


def skip_prefixes(words, expand_split=False):
    """跳过关键字、赋值与 nohup / setsid / nice / timeout / env / taskset 这类前缀，交回命令词的下标、路过的前缀名与赋值；
    env 的 `-C 目录`、`-C目录`、`--chdir=目录`、`--chdir 目录` 换的目录（最后一个，GNU env 取最后一个）交在 chdir 里，由调用方按当前目录解开
    （弄坏开关 LIB_SHELL_WORDS_BREAK=env-chdir-ignored 下不交）。env 的选项照 env_option 切（合写的 -iC、缩写、单独的 -）；-S / --split-string 的字符串：
    expand_split 为真时交在 split 里（PrefixSkip.split，不往下判，由 skip_prefixes_expanding 拆开换进词表重来），为假时只跳过它与它的值词。"""
    prefix_names, assignments, chdir, cleared, login = set(), {}, None, False, False
    position = 0
    while position < len(words):
        word = words[position]
        name = os.path.basename(word)
        assignment = ASSIGNMENT.match(word)
        if word in COMMAND_KEYWORDS:
            position += 1
            if word == "time" and not words_break_is_set("time-options-as-command"):
                # bash 的 time 关键字只认一个 -p，之后可以再跟一个 --（`time -p -p x` 的第二个 -p 是命令词）
                if words[position:position + 1] == ["-p"]:
                    position += 1
                if words[position:position + 1] == ["--"]:
                    position += 1
            continue
        if word == "builtin" and not words_break_is_set("builtin-as-command"):
            position += 1   # `builtin source 脚本`、`builtin cd 目录`：builtin 不是命令词，后面那个内建才是（`builtin -- 名字` 同）
            if words[position:position + 1] == ["--"]:
                position += 1
            following = words[position] if position < len(words) else None
            if (following is not None and following not in BASH_BUILTINS and not set("$`*?[") & set(following)
                    and not words_break_is_set("builtin-non-builtin-run")):
                return PrefixSkip(None, prefix_names, assignments, chdir, None, cleared)   # `builtin cargo …`：bash 报 not a shell builtin，什么都不跑
            continue
        if word == "coproc" and not words_break_is_set("coproc-as-command"):
            # bash 的 coproc 关键字：`coproc 命令 …` 把后面那条放进协进程照样跑，`coproc 名字 { …; }` 的名字也不是命令词（
            # 弄坏开关 coproc-as-command 下照旧把 coproc 当命令词）
            position += 1
            prefix_names.add("coproc")   # 记进路过的前缀名：bash-command-detector.sh 的 ③ 按它认把活放出追踪
            if (position + 1 < len(words) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", words[position]) and words[position + 1] == "{"):
                position += 1
            continue
        if name == "busybox" and not renamed_shells_unseen():
            # `busybox sh / ash / bash / dash … applet`：busybox 这一层剥掉，后面那个 applet 当命令词（它的 -c 代码、脚本照 shell 判）
            following = words[position + 1] if position + 1 < len(words) else None
            if following is not None and os.path.basename(following) in BUSYBOX_SHELL_APPLETS:
                prefix_names.add("busybox")
                position += 1
                continue
        if word == "function" and not words_break_is_set("function-keyword-as-command"):
            position += 2   # `function 名字 { …; }`、`function 名字() …`：关键字与函数名都不是命令词，函数体照命令判（同 `名字() { …; }`）
            continue
        if assignment:
            assignments[assignment.group(1)] = assignment.group(2)
            position += 1
            continue
        if (APPEND_ASSIGNMENT.match(word) or SUBSCRIPTED_ASSIGNMENT.match(word)) and not words_break_is_set("extended-assignments-as-command"):
            position += 1   # `名字+=值`、`名字+=(…)`、`名字[下标]=值`：bash 照赋值认，不是命令词（记下的值由 record_array_writes 管；弄坏开关下照旧当命令词）
            continue
        if name not in PREFIX_OPTIONS_WITH_VALUE:
            return PrefixSkip(position, prefix_names, assignments, chdir, None, cleared, login)
        if name == "command" and words[position + 1:position + 2] in (["-v"], ["-V"]):
            return PrefixSkip(position, prefix_names, assignments, chdir, None, cleared)  # command -v / -V 是查名字，不执行后面那个词
        prefix_names.add(name)
        position += 1
        positional_left = PREFIX_POSITIONAL_COUNT.get(name, 0)
        options_done, env_assigned = False, False
        while position < len(words):
            argument = words[position]
            if name == "env" and env_assigned and "=" not in argument and not words_break_is_set("env-options-after-assignment"):
                break   # GNU env 认到第一个 `名字=值` 就不再认选项：其后头一个不带 `=` 的词是命令词，`-u`、`-i`、`--` 也是（env 报 No such file、退 127）
            if name == "env" and not options_done and not env_assigned and not env_option_clusters_unparsed():
                option = env_option(words, position)
                if option is not None:
                    if env_option_clears_environment(argument) and not words_break_is_set("env-ignore-environment-kept"):
                        cleared = True          # env -i：之前的赋值（命令前写的 `BASH_ENV=x env -i …`）与外面带进来的都不给后面那条命令
                        assignments.clear()
                    if option.split_string is not None and not env_split_string_unexpanded():
                        if expand_split:
                            return PrefixSkip(None, prefix_names, assignments, chdir, (position, position + option.width, option.split_string), cleared)
                    elif option.chdir is not None and not env_chdir_ignored():
                        chdir = option.chdir
                    options_done = argument == "--"
                    position += option.width
                    continue
            if name == "env" and not env_chdir_ignored():
                if argument in ("-C", "--chdir") and position + 1 < len(words):
                    chdir = words[position + 1]
                elif argument.startswith("--chdir="):
                    chdir = argument[len("--chdir="):]
                elif argument.startswith("-C") and len(argument) > 2:
                    chdir = argument[2:]
            if name == "exec" and not words_break_is_set("login-profiles-unread") and (
                    (argument == "-a" and words[position + 1:position + 2] and words[position + 1].startswith("-"))
                    or (argument.startswith("-") and not argument.startswith("--") and "l" in argument[1:] and argument != "-a")):
                login = True   # `exec -l bash`、`exec -a -bash bash`：argv[0] 以 - 起头，bash / sh 照登录 shell 起
            if argument in PREFIX_OPTIONS_WITH_VALUE[name]:
                position += 2
            elif argument.startswith("-") and argument != "-":
                position += 1
            elif name == "env" and ASSIGNMENT.match(argument):
                environment_assignment = ASSIGNMENT.match(argument)
                assignments[environment_assignment.group(1)] = environment_assignment.group(2)
                position += 1
                env_assigned = True
            elif (name == "env" and "=" in argument[1 if words_break_is_set("env-leading-equals-as-command") else 0:]
                  and not words_break_is_set("env-loose-assignments-as-command")):
                position += 1   # GNU env 把带 `=` 的词都当 NAME=VALUE 设进环境（`env -- a-b=1 命令`、开头就是 `=` 的 `env =1 命令`），都不是命令词
                env_assigned = True
            elif positional_left:
                positional_left -= 1
                position += 1
            else:
                break
    return PrefixSkip(None, prefix_names, assignments, chdir, None, cleared)


def arguments_as_executed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed：`bash -n 脚本` 与 `python3 - 词…` 里的参数照旧当被执行的脚本（只给自检证红用）。"""
    return "arguments-as-executed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def shell_invocation(arguments):
    """bash / sh 这类后面的参数：是 -c 的一段代码、是起一个脚本，还是都不是（只查语法、从标准输入读）。
    -c 之后照 bash 接着认选项，选项完了（`--`、单独的 `-` 之后，或头一个不是选项的词）的那个词是代码（`bash -c -- '…'`、`bash -c -x '…'`；
    弄坏开关 LIB_SHELL_WORDS_BREAK=shell-c-options-unparsed 下取紧跟 -c 的那个词）；带 -s 的从标准输入读，后面的词是位置参数、不是脚本
    （`bash -s -- 词`；弄坏开关 LIB_SHELL_WORDS_BREAK=shell-s-positionals-as-script 下照旧当脚本）。
    bash 的长选项只认写在短选项前面的：短选项（`-c`、`-x`、`-o 值` 这类）之后再写 `--名字`（单独的 `--` 除外），bash 报 `--: invalid option`、什么都不跑
    （`bash -c --posix '…'`、`bash -x --posix 脚本`；弄坏开关 shell-long-options-after-short 下照旧当选项跳过）；
    带 -n（`-n`、`-cn`、`-xn`，`+n` 撤回）只查语法，-c 的代码、脚本、标准输入都不执行（弄坏开关 shell-syntax-only-code-run 下 -c 的代码照判）。
    这两种交 runs_nothing。"""
    option_count, only_checks_syntax, takes_code, reads_standard_input, short_option_seen = 0, False, False, False, False
    option_values_read = not words_break_is_set("shell-option-values-as-script")
    while option_count < len(arguments):
        argument = arguments[option_count]
        if argument in ("-", "--") and option_values_read:
            option_count += 1   # `bash -- 脚本`、`bash - 脚本`：选项到此为止，下一个词是脚本（后面没有词的从标准输入读）
            break
        if not argument.startswith(("-", "+")) or argument in ("-", "--"):
            break
        if argument.startswith("--") and short_option_seen and not words_break_is_set("shell-long-options-after-short"):
            return ShellInvocation(None, None, True)   # 短选项之后的长选项：bash 报 --: invalid option，什么都不跑
        option_count += 1
        short_option_seen = short_option_seen or not argument.startswith("--")
        if argument in ("-o", "+o", "-O", "+O") or (argument in SHELL_LONG_OPTIONS_WITH_VALUE and option_values_read):
            option_count += 1
            continue
        letters = "" if argument.startswith("--") else argument[1:]
        if (option_values_read and not words_break_is_set("shell-option-clusters-unvalued")
                and argument.startswith("-") and argument not in ("-o", "+o", "-O", "+O")):
            # 合写的短选项簇里每个 -o / -O 各吃后面一个词（`bash -euo pipefail -c …` 的 pipefail、`bash -oo pipefail errexit …` 两个词）：
            # 不吃掉的话 pipefail 被当成 -c 的代码或脚本，整条 -c 的代码没进判
            option_count += letters.count("o") + letters.count("O")
        if "n" in letters:
            only_checks_syntax = argument.startswith("-") and not arguments_as_executed()
        reads_standard_input = reads_standard_input or (argument.startswith("-") and "s" in letters)
        if "c" in letters:
            if words_break_is_set("shell-c-options-unparsed"):
                return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)
            takes_code = True
    if takes_code:
        if only_checks_syntax and not words_break_is_set("shell-syntax-only-code-run"):
            return ShellInvocation(None, None, True)
        return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)
    if only_checks_syntax:
        return ShellInvocation(None, None, True)
    if option_count >= len(arguments) or (reads_standard_input and not words_break_is_set("shell-s-positionals-as-script")):
        return ShellInvocation(None, None)
    return ShellInvocation(None, option_count)


def python_options_by_dash():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash：python 的选项只按「以 - 开头」认，带值选项的值被当成脚本名（只给自检证红用）。"""
    return "python-options-by-dash" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def python_invocation(arguments):
    """python 解释器后面的参数：是 -c 的一段代码、是 -m 的一个模块、是起一个脚本文件，还是都不是（从标准输入读、只有选项）。
    短选项逐个字母看（可以合写）：头一个要值的字母（PYTHON_SHORT_OPTIONS_WITH_VALUE）之后剩下的是它的值，没剩下就取下一个词；
    `--` 之后的第一个词是脚本。"""
    position = 0
    if python_options_by_dash():
        while position < len(arguments) and arguments[position].startswith("-"):
            if arguments[position] == "-" and not arguments_as_executed():
                return PythonInvocation(None, None, None)
            position += 1
        return PythonInvocation(None, None, position if position < len(arguments) else None)
    while position < len(arguments):
        argument = arguments[position]
        if argument == "-":
            if arguments_as_executed():
                position += 1
                continue
            return PythonInvocation(None, None, None)   # `python3 -`：程序从标准输入读，后面的词都是它的参数，没有被执行的脚本
        if argument == "--":
            position += 1
            break
        if not argument.startswith("-"):
            break
        if argument.startswith("--"):
            position += 2 if argument in PYTHON_LONG_OPTIONS_WITH_VALUE else 1
            continue
        letters = argument[1:]
        value_letter = next((index for index, letter in enumerate(letters) if letter in PYTHON_SHORT_OPTIONS_WITH_VALUE), None)
        if value_letter is None:
            position += 1
            continue
        attached = letters[value_letter + 1:]
        value = attached if attached else (arguments[position + 1] if position + 1 < len(arguments) else "")
        position += 1 if attached else 2
        if letters[value_letter] == "c":
            return PythonInvocation(value, None, min(position, len(arguments)))
        if letters[value_letter] == "m":
            if value == "runpy" and position < len(arguments) and not arguments[position].startswith("-"):
                value, position = arguments[position], position + 1   # `python3 -m runpy <模块>`：runpy 再按名字起后面那个模块
            return PythonInvocation(None, value, min(position, len(arguments)))
    return PythonInvocation(None, None, position if position < len(arguments) else None)


def resolve_path(directory, path):
    """相对路径按 directory 解成绝对路径；directory 认不出（None）时交 None。"""
    if os.path.isabs(path):
        return single_leading_slash(os.path.normpath(path))
    if directory is None:
        return None
    return single_leading_slash(os.path.normpath(os.path.join(directory, path)))


def single_leading_slash(path):
    """os.path.normpath 照 POSIX 留着开头恰好两个斜杠（`//tmp/x`），Linux 上它与 `/tmp/x` 是同一个文件：并成一个，免得与记下的路径对不上。"""
    if path.startswith("//") and not words_break_is_set("double-slash-kept"):
        return "/" + path.lstrip("/")
    return path


def link_source_word(arguments, source, link):
    """`ln` 的参数里 source 拷到 link（链接的路径，照写的样子）时，按当前目录解开就指到链接真读的那个文件的写法：
    符号链接（-s / --symbolic，不带 -r / --relative）里存的相对路径从链接所在的目录算（`ln -s ../s.sh sub/l.sh` 读的是 sub/../s.sh），交回 链接所在目录/源；
    硬链接、带 -r、绝对路径、`~` 起头、带 `$` 的照原样交回。弄坏开关 LIB_SHELL_WORDS_BREAK=link-source-from-current-directory 下一律照原样（源按当前目录解）。"""
    options = []
    for argument in arguments:
        if argument == "--":
            break
        if argument.startswith("-") and argument != "-":
            options.append(argument)
    symbolic = any(option == "--symbolic" or (not option.startswith("--") and "s" in option[1:]) for option in options)
    relative = any(option == "--relative" or (not option.startswith("--") and "r" in option[1:]) for option in options)
    if (not symbolic or relative or source.startswith(("/", "~")) or "$" in source
            or words_break_is_set("link-source-from-current-directory")):
        return source
    return os.path.join(os.path.dirname(link), source)


# 从标准输入读命令的 shell（shell_reads_standard_input）收到的输入：here-string `<<<` 与管道前一段是 printf / echo / cat 喂进来的字面文本当 shell 代码再切一遍；
# 认不出是什么的（带变量、命令替换，管道前一段是别的命令或读文件，标准输入从文件来）交一句判不了的说明
STANDARD_INPUT_REDIRECTS = {"<", "<<", "<<-", "<<<", "<>", "<&"}   # 改标准输入的重定向，最后一个说了算
PIPE_SEPARATORS = ("|", "|&")
BACKSLASH_ESCAPES = {"n": "\n", "t": "\t", "\\": "\\", "a": "\a", "b": "\b", "f": "\f", "r": "\r", "v": "\v", "e": "\x1b", "E": "\x1b", '"': '"', "'": "'"}
PRINTF_CONVERSION = re.compile(r"%[-+ #0]*\d*(?:\.\d*)?([diouxXfFeEgGaAcsbq%])")
ECHO_OPTION = re.compile(r"^-[neE]+$")
UNJUDGED_SNIPPET_CHARACTERS = 120


class StandardInputCode(NamedTuple):
    code: str | None       # 当 shell 代码再切的文本；没有可切的是 None
    unjudged: str | None   # 判不了的说明（喂进去的不是字面）；判得了是 None


def shell_standard_input_unread():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-unread：here-string 与管道喂给 shell 的文本不当命令再切、也不交判不了的说明（只给自检证红用）。"""
    return "shell-standard-input-unread" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def backslash_escapes(text, octal_needs_zero):
    """printf 的格式串（octal_needs_zero 为假：\\NNN）与 %b 的值、echo -e 的参数（为真：\\0NNN）里的反斜杠转义；\\c 到此为止，认不出的照原样留下。"""
    pieces, index = [], 0
    while index < len(text):
        character = text[index]
        if character != "\\" or index + 1 >= len(text):
            pieces.append(character)
            index += 1
            continue
        following = text[index + 1]
        if following == "c":
            break
        if following in BACKSLASH_ESCAPES:
            pieces.append(BACKSLASH_ESCAPES[following])
            index += 2
        elif following == "x" and re.match(r"[0-9A-Fa-f]{1,2}", text[index + 2:]):
            digits = re.match(r"[0-9A-Fa-f]{1,2}", text[index + 2:]).group(0)
            pieces.append(chr(int(digits, 16)))
            index += 2 + len(digits)
        elif following in "01234567" and (following == "0" or not octal_needs_zero):
            start = index + 2 if octal_needs_zero else index + 1
            digits = re.match(r"[0-7]{0,3}", text[start:]).group(0)
            pieces.append(chr(int(digits or "0", 8) & 0xFF))
            index = start + len(digits)
        else:
            pieces.append(character)
            index += 1
    return "".join(pieces)


def printf_output(arguments):
    """printf 的参数交给标准输出的文本：格式串里的转义照 printf 换，%… 依次取参数（%b 的参数再换一遍转义），参数没用完就把格式串再套一遍；
    `-v 名字` 写进变量、不往标准输出写，交空串。"""
    if arguments[:1] == ["--"]:
        arguments = arguments[1:]
    if not arguments or arguments[0].startswith("-v"):
        return ""
    form, values, pieces = backslash_escapes(arguments[0], False), list(arguments[1:]), []
    while True:
        used = 0
        def convert(match):
            nonlocal used
            if match.group(1) == "%":
                return "%"
            value = values[used] if used < len(values) else ""
            used += 1
            return backslash_escapes(value, True) if match.group(1) == "b" else value
        pieces.append(PRINTF_CONVERSION.sub(convert, form))
        values = values[used:]
        if not values or used == 0:
            return "".join(pieces)


def echo_output(arguments):
    """echo 的参数交给标准输出的文本：开头的 -n / -e / -E（可以合写）照 bash 的内建 echo 认，-e 时换转义，不带 -n 末尾补换行。"""
    interpret, newline, position = False, True, 0
    while position < len(arguments) and ECHO_OPTION.match(arguments[position]):
        for letter in arguments[position][1:]:
            newline = newline and letter != "n"
            interpret = (interpret or letter == "e") and letter != "E"
        position += 1
    text = " ".join(arguments[position:])
    return (backslash_escapes(text, True) if interpret else text) + ("\n" if newline else "")


def has_expansion(texts):
    """这些词（引号已去掉）里有没有 `$` 或反引号：外层 shell 先展开它们，展开之后是什么这里算不出（单引号里写的同样算，切词时分不出）。"""
    return any("$" in text or "`" in text for text in texts)


def unjudged_snippet(text):
    text = " ".join(text.split())
    return text if len(text) <= UNJUDGED_SNIPPET_CHARACTERS else text[:UNJUDGED_SNIPPET_CHARACTERS] + "…"


def piped_literal_output(words, heredocs):
    """管道前一段交给后一段的文本：printf、echo 照它们的参数现算，不带文件操作数的 cat 交它自己读到的 heredoc 正文（最后一段）。
    交 StandardInputCode：参数里有 `$` 或反引号的照算、另交判不了的说明；别的命令、cat 读文件或读更前一段的输出，只交判不了的说明。"""
    skipped, words = skip_prefixes_expanding(words)
    if skipped.command_position is None:
        return StandardInputCode(None, "管道前一段不是一条简单命令（花括号组、子 shell 这类），它交出来的输出算不出")
    name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
    shown = unjudged_snippet(" ".join(words[skipped.command_position:]))
    if name in ("printf", "echo"):
        text = printf_output(arguments) if name == "printf" else echo_output(arguments)
        unjudged = f"管道前一段 `{shown}` 的参数里有变量或命令替换，展开之后喂进去的是什么判不了" if has_expansion(arguments) else None
        return StandardInputCode(text, unjudged)
    if name == "cat" and all(argument.startswith("-") for argument in arguments if argument != "-") and heredocs:
        body = heredocs[-1]
        unjudged = None if body.is_delimiter_quoted or not has_expansion([body.text]) else \
            "管道前一段 cat 读的 heredoc 结束词没带引号、正文里有变量或命令替换，展开之后喂进去的是什么判不了"
        return StandardInputCode(body.text, unjudged)
    return StandardInputCode(None, f"管道前一段是 `{shown}`，它交出来的输出这里算不出")


def shell_standard_input_files(redirections, piped_from):
    """从标准输入读命令的 shell 读的是不是盘上的文件：改标准输入的最后一个是 `< 文件` / `<> 文件`（/dev/null 除外）交 [文件]；
    没改标准输入、管道前一段是只带文件参数的 cat（不带选项、不读 heredoc）交那几份文件（照次序接起来喂进去）；别的交 []（here-string、printf / echo 的字面照 shell_standard_input 切）。
    `bash -s < 文件`、`bash < 文件`、`sh -s < 文件`、`cat 文件 | bash` 交出文件，调用方读正文判。
    弄坏开关 LIB_SHELL_WORDS_BREAK=shell-standard-input-file-unread 下一律交 []：这几种只记判不了、放行。"""
    if words_break_is_set("shell-standard-input-file-unread"):
        return []
    stdin_redirections = [(operator, target) for operator, target in redirections if operator in STANDARD_INPUT_REDIRECTS]
    if stdin_redirections:
        operator, target = stdin_redirections[-1]
        return [target] if operator in ("<", "<>") and target != "/dev/null" else []
    if piped_from is None:
        return []
    words, heredocs = piped_from
    skipped, words = skip_prefixes_expanding(words)
    if skipped.command_position is None or heredocs:
        return []
    name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
    if name != "cat" or not arguments or any(argument.startswith("-") for argument in arguments):
        return []
    return list(arguments)


def shell_standard_input(redirections, piped_from):
    """从标准输入读命令的 shell 收到的输入：redirections 是它那一条简单命令上的重定向（((运算符, 目标词), …)，改标准输入的最后一个说了算），
    piped_from 是管道前一段（(词, 喂给它的 heredoc 正文)；不在管道后面是 None）。交 StandardInputCode：
    here-string 交它的文字（末尾补换行）；管道照 piped_literal_output；标准输入从文件或别的描述符来的（< 文件、<& 号、<> 文件）只交判不了的说明（< /dev/null 不算）；
    heredoc（<<、<<-）不归这里（喂给 shell 的正文已留在原文里当后续的行切），交两个 None。"""
    stdin_redirections = [(operator, target) for operator, target in redirections if operator in STANDARD_INPUT_REDIRECTS]
    if stdin_redirections:
        operator, target = stdin_redirections[-1]
        if operator == "<<<":
            unjudged = f"here-string `{unjudged_snippet(target)}` 里有变量或命令替换，展开之后喂进去的是什么判不了" if has_expansion([target]) else None
            return StandardInputCode(target + "\n", unjudged)
        if operator in ("<<", "<<-") or target == "/dev/null":
            return StandardInputCode(None, None)
        return StandardInputCode(None, f"标准输入从 `{operator} {unjudged_snippet(target)}` 来，读进去的是什么这里判不了")
    if piped_from is None:
        return StandardInputCode(None, None)
    return piped_literal_output(*piped_from)


AMBIENT_STDIN_OPERATORS = {"<", "<>"}   # exec / 复合命令上把标准输入从一份文件下发的重定向（fd 0 或没写 fd）


class CompoundStdinFeeds(NamedTuple):
    files: tuple           # 下发给里层 shell 的 stdin 文件词（按调用方的目录解；算得出的）
    has_bare_shell: bool   # 这一层里有没有裸的读标准输入的 shell（不带 -c、不带脚本、自己没有 < 的）
    undeterminable: bool   # 有没有算不出来源的 stdin 文件（写成变量 / 命令替换）
    unrecognized: tuple = ()   # 认不出下发给里层 shell 的是什么的几处，每处一句（调用方照认不出拒）：从别的描述符（`<&3`）、here-string 或 heredoc 下发，管道接进复合命令


COMPOUND_CLOSING_WORDS = ("}", "done", "fi", "esac")   # 收尾词上的输入重定向管整个复合命令
COMPOUND_PIPED_OPENINGS = frozenset({"if", "while", "until", "for", "select", "case"})   # 管道接进来、里层裸 shell 读到的是管道（花括号组由 piped_from 认）


def compound_stdin_closers_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen：只认 exec、`}`、`done` 与 `)` 上的 `< 文件`；fi / esac 收尾、调这段文本里定义的函数、
    `<&` / here-string / heredoc 下发、管道接进复合命令、bash -c 与 eval 的代码里的都不认（只给自检证红用）。"""
    return words_break_is_set("compound-stdin-closers-unseen")


def compound_redirected_stdin_files(text, depth=0):
    """exec < 文件、{ … } < 文件、( … ) < 文件、while / until / for … done < 文件、if … fi < 文件、case … esac < 文件、调这段文本里定义的函数时 `函数 < 文件`：
    这几种把标准输入从一份文件下发给里层读标准输入的 bash / sh。交 CompoundStdinFeeds。没有裸 shell 时 files 为空（`while read … done < 文件`、`{ echo; } < 文件` 这类不下发）；
    下发的文件照 `bash 文件` 由调用方读正文判。下发的是别的描述符（`<&3`）、here-string 与 heredoc，或管道接进 `( … )`、if / case / 循环的，认不出里层 shell 读到什么，
    每处一句交在 unrecognized（认不出的一律拒，偏严）；bash -c 与 eval 的代码照同一套再找一遍、并进来。弄坏开关 LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen 下只认 exec、`}`、`done` 与 `)` 上的 `< 文件`。"""
    tokens = shell_tokens(strip_data_heredocs(text))
    simple, _targets, _substitutions, redirections = simple_commands(tokens)
    closers_seen = not compound_stdin_closers_unseen()
    closing_words = COMPOUND_CLOSING_WORDS if closers_seen else ("}", "done")
    functions = set(function_bodies(text)) if closers_seen else set()
    ambient, has_bare_shell, unrecognized, nested = [], False, [], []
    for words, redirs in zip(simple, redirections):
        skipped = skip_prefixes(words)
        if skipped.command_position is not None:
            name, rest = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
            if is_shell_name(name) and shell_reads_standard_input(rest):
                if not any(operator in STANDARD_INPUT_REDIRECTS for operator, _ in redirs):
                    has_bare_shell = True
            if closers_seen and depth < MAXIMUM_NESTING:
                code = shell_invocation(rest).code_string if is_shell_name(name) else (" ".join(rest) if name == "eval" else None)
                if code:
                    nested.append(compound_redirected_stdin_files(code, depth + 1))
        exec_only = skipped.command_position is None and "exec" in skipped.prefix_names
        group_close = bool(words) and words[-1] in closing_words
        function_call = skipped.command_position is not None and words[skipped.command_position] in functions
        if exec_only or group_close or function_call:
            ambient += [target for operator, target in redirs if operator in AMBIENT_STDIN_OPERATORS and redirection_fd(operator) in (None, 0)]
            if closers_seen:
                carrier = "exec" if exec_only else words[skipped.command_position] if function_call else words[-1]
                unrecognized += [f"`{carrier} {operator} {unjudged_snippet(target)}` 把标准输入从"
                                 f"{'别的描述符' if operator == '<&' else 'here-string' if operator == '<<<' else 'heredoc'}下发给里层读标准输入的 shell"
                                 for operator, target in redirs
                                 if redirection_fd(operator) in (None, 0) and ((operator == "<&" and target != "-") or operator in ("<<<", "<<", "<<-"))]
    # ( … ) < 文件：) 后面的 < 文件被 simple_commands 当没有词的尾命令丢掉，另扫一遍 token；`( … ) <&3`、`( … ) <<< 字` 同样认不出
    for index in range(len(tokens) - 2):
        if (tokens[index] == (True, ")") and tokens[index + 1][0] is True and tokens[index + 1][1] in AMBIENT_STDIN_OPERATORS
                and redirection_fd(tokens[index + 1][1]) in (None, 0) and tokens[index + 2][0] is False):
            ambient.append(tokens[index + 2][1])
        if (closers_seen and tokens[index] == (True, ")") and tokens[index + 1][0] is True and redirection_fd(tokens[index + 1][1]) in (None, 0)
                and tokens[index + 2][0] is False and ((tokens[index + 1][1] == "<&" and tokens[index + 2][1] != "-") or tokens[index + 1][1] in ("<<<", "<<", "<<-"))):
            unrecognized.append(f"`( … ) {tokens[index + 1][1]} {unjudged_snippet(tokens[index + 2][1])}` 把标准输入从别的描述符、here-string 或 heredoc 下发给里层读标准输入的 shell")
    # 管道接进 ( … ) 或 if / case / 循环：里层的裸 shell 读的是管道前一段交出来的东西
    if closers_seen:
        for index in range(len(tokens) - 1):
            if tokens[index][0] is True and tokens[index][1] in PIPE_SEPARATORS:
                following = index + 1
                while following < len(tokens) and tokens[following] == (True, "\n"):
                    following += 1
                if following < len(tokens) and (tokens[following] == (True, "(") or (tokens[following][0] is False and tokens[following][1] in COMPOUND_PIPED_OPENINGS)):
                    unrecognized.append(f"管道接进 `{tokens[following][1]} …` 复合命令，里层读标准输入的 shell 读到的是管道前一段交出来的东西")
    files, undeterminable, found = [], False, []
    if has_bare_shell:   # 这一层没有裸 shell 的，这一层的下发不算（下发给的是别的命令）
        for target in ambient:
            if target == "/dev/null":
                continue
            if has_expansion([target]):
                undeterminable = True
            else:
                files.append(target)
        found += unrecognized
    for inner in nested:
        files += inner.files
        undeterminable = undeterminable or inner.undeterminable
        found += inner.unrecognized
    return CompoundStdinFeeds(tuple(files), has_bare_shell or any(inner.has_bare_shell for inner in nested), undeterminable, tuple(dict.fromkeys(found)))


# 目录词里认的变量引用：`$名字`、`${名字}`；别的写法（`${名字:-默认}`、`$1`、`$$`、`$(…)`）不认，留着的 `$` 算展开不了
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
# 改写变量、改成什么这里算不出的命令：for / select 的循环变量、read 读进来的（unset 在 commands_at_command_position 里另记）
REBINDING_COMMANDS = {"for", "select", "read", "mapfile", "readarray"}
APPEND_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\+=(.*)$", re.S)
ELEMENT_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\[[^\]]*\]\+?=(.*)$", re.S)
SUBSCRIPTED_ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*\[.*\]\+?=", re.S)   # 下标里还可以套展开（`a[${b[$k]}]+=x`）：skip_prefixes 认它是赋值词用
# 声明命令：参数里的 `名字=值` 照赋值记
DECLARATION_COMMANDS = {"local", "declare", "typeset", "readonly"}


def directory_variables_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded：cd / pushd / env -C 的目录带 `$` 一律展开不了（只给自检证红用）。"""
    return "directory-variables-unexpanded" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def variable_rebinding_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen：for / select / read / unset 改写过的变量照旧用之前记下的值（只给自检证红用）。"""
    return "variable-rebinding-unseen" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def expand_known_variables(word, variables):
    """词里的 `$名字`、`${名字}` 换成值：先查 variables（这段命令文本里前面赋过的，值已经展开；值是 None 的是改写过、算不出的），
    查不到的只认 HOME（取钩子进程自己的环境：钩子与它判的命令是同一个用户起的）。有一处展开不了（没赋过的变量、算不出的值、别的写法、命令替换、反引号）交 None。
    弄坏开关 LIB_SHELL_WORDS_BREAK=directory-variables-unexpanded 下带 `$` 的一律交 None。"""
    if "`" in word:
        return None
    if "$" not in word:
        return word
    if directory_variables_unexpanded():
        return None
    unexpanded = []
    def replace(match):
        name = match.group(1) or match.group(2)
        value = variables[name] if name in variables else (os.environ.get("HOME") if name == "HOME" else None)
        if value is None:
            unexpanded.append(name)
            return match.group(0)
        return value
    expanded = VARIABLE_REFERENCE.sub(replace, word)
    return None if unexpanded or "$" in expanded else expanded


def assigned_value(value, variables):
    """`名字=值` 赋进去的是什么：开头的 `~` 展开，变量照 expand_known_variables 展开；展开不了交 None（记成算不出）。"""
    return expand_known_variables(os.path.expanduser(value), variables)


def rebind_variables(name, arguments, variables):
    """会改写变量的命令记进 variables：local / declare 这类声明里的 `名字=值` 照赋值记；for / select 的循环变量、read 读进来的几个名字记成 None（算不出）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=variable-rebinding-unseen 下只记声明里的赋值。"""
    if name in DECLARATION_COMMANDS:
        for argument in arguments:
            assignment = ASSIGNMENT.match(argument)
            if assignment:
                variables[assignment.group(1)] = assigned_value(assignment.group(2), variables)
        return
    if variable_rebinding_unseen():
        return
    names = arguments[:1] if name in ("for", "select") else [argument for argument in arguments if not argument.startswith("-")]
    for argument in names:
        if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", argument):
            variables[argument] = None


def array_appends_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=array-appends-unseen：`名字+=值`、`名字[下标]=值` 不记（只给自检证红用）。"""
    return "array-appends-unseen" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def record_array_writes(words, variables):
    """一条简单命令只有关键字与赋值（没有命令词）时，把里面的 `名字+=值`（数组接数组、字符串接字符串）与 `名字[下标]=值`（记成算不出）记进 variables；
    `名字=值` 由调用方照旧记。有命令词的（`名字+=值 命令`）不记。弄坏开关 array-appends-unseen 下什么都不记。"""
    if array_appends_unseen():
        return
    writes = []
    for word in words:
        if word in COMMAND_KEYWORDS or ASSIGNMENT.match(word):
            continue
        append, element = APPEND_ASSIGNMENT.match(word), ELEMENT_ASSIGNMENT.match(word)
        if append is None and element is None:
            return
        writes.append((append, element))
    for append, element in writes:
        if element is not None:
            variables[element.group(1)] = None
            continue
        name = append.group(1)
        before, added = variables.get(name), assigned_value(append.group(2), variables)
        if before is None or added is None:
            variables[name] = None
        elif before.startswith("(") and before.endswith(")") and added.startswith("(") and added.endswith(")"):
            variables[name] = "(" + " ".join(part for part in (before[1:-1].strip(), added[1:-1].strip()) if part) + ")"
        elif not before.startswith("(") and not added.startswith("("):
            variables[name] = before + added
        else:
            variables[name] = None


GIT_TOPLEVEL_SUBSTITUTION = re.compile(r"^(?:\$\(\s*git\s+rev-parse\s+--show-toplevel\s*\)|`\s*git\s+rev-parse\s+--show-toplevel\s*`)")
PWD_SUBSTITUTION = re.compile(r"^(?:\$\(\s*pwd\s*\)|`\s*pwd\s*`)")
GIT_LOCATION_VARIABLES = ("GIT_DIR", "GIT_WORK_TREE", "GIT_CEILING_DIRECTORIES", "GIT_DISCOVERY_ACROSS_FILESYSTEM", "GIT_COMMON_DIR")


def cd_known_targets_unexpanded():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded：cd 的目标里 $PWD、$OLDPWD、`cd -`、`$(git rev-parse --show-toplevel)`、`$(pwd)` 照旧算不出。"""
    return words_break_is_set("cd-known-targets-unexpanded")


def cd_failure_guards_unseen():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen：cd 失败之后哪几条不跑一概不认（CommandAtPosition.cd_failure_skips 都是 False）。"""
    return words_break_is_set("cd-failure-guards-unseen")


def git_toplevel(directory):
    """在 directory 里跑 `git rev-parse --show-toplevel` 打出什么：从它（解开符号链接）往上找头一个 .git（带 HEAD 的目录，或以 `gitdir: ` 起头的文件），交那一层目录；
    一直找到 / 都没有交 ""（git 报错、标准输出空）；directory 是 None、盘上不在、找到的 .git 认不准（目录里没有 HEAD、文件不是 gitdir: 起头、那一层不归这个用户，git 会拒）交 None。"""
    if directory is None or not os.path.isdir(directory):
        return None
    current = os.path.realpath(directory)
    while True:
        marker = os.path.join(current, ".git")
        if os.path.lexists(marker):
            try:
                if os.stat(current).st_uid != os.getuid():
                    return None
                if os.path.isdir(marker):
                    return current if os.path.isfile(os.path.join(marker, "HEAD")) else None
                with open(marker, encoding="utf-8", errors="replace") as handle:
                    return current if handle.read(8) == "gitdir: " else None
            except OSError:
                return None
        parent = os.path.dirname(current)
        if parent == current:
            return ""
        current = parent


def changed_directory(directory, arguments, variables=None, previous_directory=None):
    """cd / pushd 之后的当前目录。目标里的变量照 expand_known_variables 展开（variables 是这段命令文本里前面赋过的；不给时只认 HOME）；
    $PWD 是 directory、$OLDPWD 与 `-` 是 previous_directory（上一处 cd 之前的目录，None 是算不出），起头的 `$(git rev-parse --show-toplevel)` 照 git_toplevel 算、`$(pwd)` 是 directory
    （弄坏开关 LIB_SHELL_WORDS_BREAK=cd-known-targets-unexpanded 下这几样照旧算不出）；cd 到空串（git 不在仓里时）bash 5.2 不换目录。
    展开不了的变量或别的命令替换认不出，交 None。"""
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    if not targets:
        return os.path.expanduser("~")
    target = os.path.expanduser(targets[0])
    known = dict(variables or {})
    if not cd_known_targets_unexpanded():
        if target == "-":
            return previous_directory
        if directory is not None:
            known.setdefault("PWD", directory)
        if previous_directory is not None:
            known.setdefault("OLDPWD", previous_directory)
        leading = GIT_TOPLEVEL_SUBSTITUTION.match(target) or PWD_SUBSTITUTION.match(target)
        if leading is not None:
            if leading.re is GIT_TOPLEVEL_SUBSTITUTION:
                replacement = None if any(name in known for name in GIT_LOCATION_VARIABLES) else git_toplevel(directory)
            else:
                replacement = directory
            if replacement is None:
                return None
            target = replacement + target[leading.end():]
            if target == "":
                return directory
    if target == "-":
        return None
    target = expand_known_variables(target, known)
    if target is None:
        return None
    return resolve_path(directory, target)


SUBSHELL_PREFIX_KEYWORDS = frozenset({"time", "-p", "--", "!"})   # 子 shell 前面只隔着这几个关键字（`time ( … )`、`! ( … )`）的，照紧跟在 `&&` 后面算


def guarded_subshell_boundary(boundary, previous_words, guards, cd_and_chain, cd_failure_poisoned, cd_seen):
    """commands_at_command_position 里一条简单命令前面的分隔符 boundary 先过这一道：`cd 目录 && ( … )` 的子 shell 在 cd 失败时整个不跑，组里的命令照 `&&` 守着算
    （`cd 不存在 && …` 因 cd 失败不跑，不当拒）。
    guards 是 {"depth": 括号层数, "groups": [(开组之前的层数, 组外的 cd_failure_poisoned, 组外的 cd_seen), …]}，跨命令带着走。
    先收掉 boundary 开头合上这样一组的右括号：组外接着算在 `&&` 一串里（cd_and_chain 回到真），组里 cd 过的照旧不守后面的（组里的 cd 这里当对后面也生效）；
    再认这一处是不是开了这样一组：`&&` 一串还在、右括号都收掉了，剩下的是 `&&` 后面紧跟左括号，或前一条只有 time / ! 这类关键字（SUBSHELL_PREFIX_KEYWORDS）、这里紧跟左括号。
    开了组的交 (()，组里从头记 `&&` 一串与 cd)；收了组的交右括号后面那几个分隔符（换行去掉），由调用方照原来那两条判据接着判；两样都没有的原样交回 boundary。
    交 (留给原来那两条判据的分隔符, cd_and_chain, cd_failure_poisoned, cd_seen)。"""
    separators = [separator for separator in boundary if separator != "\n"]
    position, kept_closings, changed = 0, [], False
    while position < len(separators) and separators[position] == ")":
        guards["depth"] -= 1
        if guards["groups"] and guards["depth"] == guards["groups"][-1][0]:
            _, outer_poisoned, outer_cd_seen = guards["groups"].pop()
            cd_failure_poisoned = outer_poisoned or cd_failure_poisoned or cd_seen
            cd_seen = outer_cd_seen or cd_seen
            cd_and_chain, changed = True, True
        else:
            kept_closings.append(")")
        position += 1
    rest = separators[position:]
    after_keywords = bool(previous_words) and set(previous_words) <= SUBSHELL_PREFIX_KEYWORDS and previous_words[0] in ("time", "!")
    opens = cd_and_chain and not kept_closings and (
        (len(rest) > 1 and rest[0] == "&&" and set(rest[1:]) == {"("}) or (bool(rest) and set(rest) == {"("} and after_keywords))
    if opens:
        guards["groups"].append((guards["depth"], cd_failure_poisoned, cd_seen))
        guards["depth"] += rest.count("(")
        return (), False, cd_failure_poisoned, False
    guards["depth"] += rest.count("(") - rest.count(")")
    return (tuple(kept_closings + rest) if changed else boundary), cd_and_chain, cd_failure_poisoned, cd_seen


def cd_failure_exits(simple, boundaries, position):
    """simple[position] 是不是一条单独的 `exit [n]`、跑完之后这一层就结束（后面跟的是 `;`、换行或这段文本的末尾，不在管道、后台、括号里）：
    `cd 目录 || exit` 的 cd 失败了，后面同一层的都不跑（`return` 在顶层不退出、照跑，不算）。"""
    if position >= len(simple) or not simple[position] or simple[position][0] != "exit" or len(simple[position]) > 2:
        return False
    after = boundaries[position + 1] if position + 1 < len(boundaries) else ()
    return all(separator in (";", "\n") for separator in after)


FUNCTION_CALLING_WORDS = frozenset({"eval", "source", ".", "trap", "compgen", "declare", "typeset", "set", "alias", "enable"})   # 不点名字也调得到函数、或让子进程看得见函数的写法
IMPLICITLY_CALLED_FUNCTIONS = frozenset({"command_not_found_handle"})   # bash 自己会调的函数名：命令找不到时调它


def drop_uncalled_function_bodies(tokens, text, stripped_heredocs):
    """切好的词里只定义、不会被调到的 `function 名字 { …; }`（`function 名字() { …; }` 同）连同函数体去掉：bash 只定义不跑函数体。
    只在这几样都成立时去掉，否则照留（函数体照命令判，偏严）：函数体是一个花括号组、里面没有另一个 `{`（到头一个 `}` 为止）；名字是字面的、
    不是 bash 自己会调的（IMPLICITLY_CALLED_FUNCTIONS）；整段命令文本 text 里这个名字只出现这一次（照词的边界数，heredoc 正文、引号里的都算），
    函数体之外切好的词里也没有一个是它（拆开拼写的 `"fo"o`、`fo{o,}` 解开之后就是它；还留着成对花括号的词认不准，照调到了算：function_never_called）；
    函数体之外没有命令替换、变量、反引号、通配，没有 FUNCTION_CALLING_WORDS 里的词（eval、source、set -a 这类），剥掉的 heredoc 正文里没有 `$` 与反引号。
    不带 function 关键字的 `名字() { …; }`（命令位置上的名字紧跟一对空的圆括号）照同一套判据（复跑分类报告 d137-replay-classify 二节末 legA-r2 零星；
    弄坏开关 LIB_SHELL_WORDS_BREAK=posix-uncalled-functions-run 下只认 function 关键字）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=uncalled-functions-run 下照留。"""
    posix_definitions = not words_break_is_set("posix-uncalled-functions-run")
    if words_break_is_set("uncalled-functions-run") or ((False, "function") not in tokens and not (posix_definitions and (True, "(") in tokens)):
        return tokens
    at_command_position, index = True, 0
    while index < len(tokens):
        kind, value = tokens[index]
        definition = None   # (函数名, 函数体从哪个下标起)
        if kind is False and at_command_position and value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
            definition = (tokens[index + 1][1], index + 2)
            if tokens[definition[1]:definition[1] + 2] == [(True, "("), (True, ")")]:
                definition = (definition[0], definition[1] + 2)
        elif (posix_definitions and kind is False and at_command_position and value not in COMMAND_KEYWORDS and "=" not in value
              and tokens[index + 1:index + 3] == [(True, "("), (True, ")")]):
            definition = (value, index + 3)
        if definition is not None:
            name, body = definition
            while tokens[body:body + 1] == [(True, "\n")]:
                body += 1
            closing = next((position for position in range(body + 1, len(tokens)) if tokens[position] in ((False, "{"), (False, "}"))), None)
            if (tokens[body:body + 1] == [(False, "{")] and closing is not None and tokens[closing] == (False, "}")
                    and function_never_called(name, tokens[:index] + tokens[closing + 1:], text, stripped_heredocs)):
                tokens = tokens[:index] + tokens[closing + 1:]
                continue
        if kind is True:
            at_command_position = value in COMMAND_SEPARATORS
        elif kind is False:
            at_command_position = at_command_position and value in COMMAND_KEYWORDS
        index += 1
    return tokens


def function_never_called(name, outside, text, stripped_heredocs):
    """drop_uncalled_function_bodies 的判据：函数体之外的词 outside 与整段命令文本 text 里，这个名字确定调不到。
    名字在原文里照字面只出现定义那一次；函数体之外切好的词（引号、反斜杠、反斜杠续行、花括号展开都已照 bash 解开）里有一个正是这个名字的算调到了
    （`"fo"o`、`fo{o,}`、`fo\\o` 这类拆开拼写的调用）；词里还留着成对的花括号（超过 BRACE_EXPANSION_LIMIT 照原样留的、`{}`）认不准展开成什么，照调到了算（偏严）。
    弄坏开关 LIB_SHELL_WORDS_BREAK=function-calls-literal-only 下只数原文里的字面（拆开拼写的调用认不出）。"""
    if not name or set("$`*?[\\{}") & set(name) or name in IMPLICITLY_CALLED_FUNCTIONS:
        return False
    if len(re.findall(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", text)) != 1:
        return False
    literal_only = words_break_is_set("function-calls-literal-only")
    for kind, value in outside:
        if kind is None or (kind is False and (set("$`*?[") & set(value) or os.path.basename(value) in FUNCTION_CALLING_WORDS)):
            return False
        # 还留着的花括号都在引号里（ShellWord.unquoted_braces 为假，`'echo a{,}'`）的是数据，不会再展开成这个名字（修补报告 debt-tw-shell-words-fix2 第 95 行那一条；
        # 弄坏开关 quoted-braces-as-unknown 下引号里的也当认不准）
        if kind is False and not literal_only and (value == name or ("{" in value and "}" in value and getattr(value, "unquoted_braces", True))):
            return False
    return not any("$" in heredoc.text or "`" in heredoc.text for heredoc in stripped_heredocs)


POSITIONAL_COMMAND_WORD = re.compile(r"^\$(?:[@*0-9]|\{[@*0-9]+\})")   # 命令词是位置参数：`"$@"`、`$1`、`${@}`（引号已去掉）


def function_bodies(text):
    """text 里定义的每个函数名 → 函数体文本（`名字() { … }` 或 `function 名字 [()] { … }`，到配对的 `}` 为止；认不出函数体的——子 shell 当函数体、花括号没配上——不收）。
    把函数体里切好的词重新用空格连起来、引号去掉，照一段命令文本再切（launcher_functions、shell_starting_functions 共用这一份）。"""
    if "(" not in text and "function" not in text:
        return {}
    tokens = shell_tokens(strip_data_heredocs(text))
    found, index, at_command_position = {}, 0, True
    while index < len(tokens):
        kind, value = tokens[index]
        name, body = None, None
        if kind is False and at_command_position:
            if value == "function" and index + 1 < len(tokens) and tokens[index + 1][0] is False:
                name, body = tokens[index + 1][1], index + 2
                if tokens[body:body + 2] == [(True, "("), (True, ")")]:
                    body += 2
            elif tokens[index + 1:index + 3] == [(True, "("), (True, ")")]:
                name, body = value, index + 3
        if name is not None:
            while tokens[body:body + 1] == [(True, "\n")]:
                body += 1
            if tokens[body:body + 1] == [(False, "{")]:
                depth, cursor = 0, body
                while cursor < len(tokens):
                    if tokens[cursor] == (False, "{"):
                        depth += 1
                    elif tokens[cursor] == (False, "}"):
                        depth -= 1
                        if depth == 0:
                            break
                    cursor += 1
                pieces = []
                for piece_kind, piece in tokens[body + 1:cursor]:
                    if piece_kind is True:
                        pieces.append(piece if piece == "\n" else f" {piece} ")
                    elif piece_kind is False:
                        pieces.append(" " + shlex.quote(piece))
                found[name] = "".join(pieces)
        if kind is True:
            at_command_position = value in COMMAND_SEPARATORS
        elif kind is False:
            at_command_position = at_command_position and value in COMMAND_KEYWORDS
        index += 1
    return found


def function_body_starts_shell(body):
    """函数体文本里有没有一条命令起一个会读启动文件的 shell：命令词剥前缀后 basename 是 is_shell_name，且不是只查语法 / 只打印版本（shell_invocation.runs_nothing 为假）。"""
    for words in simple_commands(shell_tokens(strip_data_heredocs(body)))[0]:
        skipped = skip_prefixes(words)
        if skipped.command_position is None:
            continue
        name = os.path.basename(words[skipped.command_position])
        if is_shell_name(name) and not shell_invocation(words[skipped.command_position + 1:]).runs_nothing:
            return True
    return False


def shell_starting_functions(text):
    """text 里函数体会起一个读启动文件的 shell 的函数名 → 函数体文本（`f(){ bash -c true; }; BASH_ENV=脚本 f` 的函数体就地按词法环境判，看不到调用处前缀设的 BASH_ENV / ENV；
    调用方拿调用处的环境重判这份函数体，补上里层 shell 的启动文件）。"""
    return {name: body for name, body in function_bodies(text).items() if function_body_starts_shell(body)}
def heredocs_fed(redirections, stripped_heredocs):
    """一条简单命令上 `<<` / `<<-` 喂给它的、被当数据剥掉的 heredoc 正文（HeredocBody，按出现次序）：重定向目标是 HEREDOC_BODY_MARKER 记号的，按编号从 stripped_heredocs 取。"""
    found = [HEREDOC_BODY_MARKER.match(target) for operator, target in redirections if operator in ("<<", "<<-")]
    return tuple(stripped_heredocs[int(marker.group(1))] for marker in found if marker and int(marker.group(1)) < len(stripped_heredocs))
