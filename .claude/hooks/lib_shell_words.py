"""Bash 命令文本的切词、切简单命令、认命令位置、剥前缀与包装、跟 cd。

.claude/hooks/ 下判 Bash 命令的 hook（bash-command-detector.sh、heavy-test-guard.sh）共用这一份：
各自按文件路径用 importlib 导入，只留自己的判定。这里 def 的函数名在 .claude/hooks/ 别的文件里再 def 一份，
门禁 code-tooling 的 agent-write-scope 格判红——一份改了另一份不跟，同一条命令一个 hook 认得出、另一个认不出。

认得出的：引号与反斜杠、词首的 # 注释、重定向（连同目标去掉，输出重定向的目标另交）、
命令替换 `$(…)` 与反引号（引号里外都认：整段留在所在的词里，里面的命令另按一条完整的命令递归判；`$((…))` 算术整段留在词里、不判）、
进程替换 `<(…)` 与 `>(…)`（只在引号外认，办法同命令替换：整段留在所在的词里，`)` 之后的词照旧是外层命令的参数）、
命令前的关键字与赋值、nohup / setsid / nice / ionice / timeout / env / stdbuf / sudo / taskset / command / exec 这类前缀、
`bash [选项] 脚本` 与 `bash -c '…'`（递归进去）、`python3 [选项] 脚本`、`source` / `.`（后面跟的像一个路径才算）、`capped.sh N …` 与 `run-with-memory-cap.sh <上限> …`，
（被执行的只是脚本位上那一个词：`bash -n 脚本` 只查语法、`python3 - 词…` 的程序从标准输入读，后面的词都是参数，都不当被执行的脚本交出；
弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed 把这两种里的参数照旧当被执行的脚本，哪几份自证的哪几项红见「弄坏开关」一段），
python 的选项照解释器的语义切（python_invocation）：短选项可以合写（`-Bu`），`-W`、`-X` 与 `--check-hash-based-pycs` 带一个值（贴着写或另起一个词），
值不当脚本名；`-c <代码>` 的程序是那段代码、没有被执行的脚本，交出的是 python 这一条本身（代码由调用方从参数里取）；`-m <模块>` 交出
`<模块里的点换成斜杠>.py` 当被执行的脚本（launcher 是 "python"），`-m runpy <模块>` 取后面那个模块，调用方拿它与仓内脚本的位置比
（弄坏开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」，哪几份自证的哪几项红见「弄坏开关」一段），
数组赋值 `名字=(…)`（元素是值，不当命令）、`command -v` / `-V`（查名字，后面那个词不当命令），
以及 cd / pushd 换目录、export / unset 改变量对后面命令的影响。喂给非 shell 命令的 heredoc 正文每一层都先剥掉。
剥掉的 heredoc 正文与输出重定向挂回它所在的那条命令（CommandAtPosition 的 standard_input_heredocs、output_redirections）：
要认「同一条命令里先由 heredoc 写出文件、再执行它」的 hook 从这两格读，不自己再切一遍。重定向前的 fd 号切词时丢掉了，`2> 文件` 与 `> 文件` 分不开。
剥掉包装之后命令词是一个脚本时，记下它是怎么起的（launcher：bash / sh 这类后面跟的、source / . 读进来的、python3 后面跟的）；
经 `run-with-memory-cap.sh <上限>` 剥出来的命令记 memory_capped，连同它 `bash -c` 里的命令；调用方往里读它起的脚本时，把这个标记当 memory_capped 参数传回来。
脚本文件本身不在这里读，要读的 hook 自己读，再把正文交回 commands_at_command_position。
只有一处例外：detached_run_script 认 scripts/hosts/detached-run.sh（名字、符号链接、文件开头的签名行），读命令词那份文件的开头 4 KiB，两个 hook 共用这一份认法。
detached-run.sh 的参数先照它的 preflight 摘掉每一个 `--force`（连 `--` 之后的、连写在选项值位置上的），再切 start（detached_run_arguments_after_preflight）：
闸看的那条命令与单元里真跑的逐字相同（弄坏开关 LIB_SHELL_WORDS_BREAK=detached-run-force-kept 退回只跳过子命令前与选项之间的 --force，
哪几份自证的哪几项红见「弄坏开关」一段）。
判不到的：变量里拼出来的命令、eval、`<<<` 喂给 shell 的字符串、进程替换的输出再交给 shell 执行（`bash <(…)`、`source <(…)`，括号里的命令照判，它打印出来的不判）、Makefile 与 xargs 起的命令；
喂给 shell 的 heredoc 正文里引号没配对时，其后的切词会错；heredoc 挂的命令从上一行续行过来时认不出它喂给谁（当数据剥掉）；heredoc 挂在命令替换或进程替换里面的命令上（`$(bash <<EOF`、`<(bash <<EOF`）认不出它喂给 shell（当数据剥掉）；圆括号子 shell 里的 cd 与 export 当成对后面的命令也生效；
命令替换与进程替换里的命令按替换所在那条命令开始时的目录与 export 判（替换里的 cd 不影响外面）。
弄坏开关（LIB_SHELL_WORDS_BREAK，几个用逗号连；只给自证判红用）。这一份没有自己的 --selftest，由导入它的几份的自证判红：
lib_heavy_tests.py、heavy-test-guard.sh、bash-command-detector.sh、scripts/hosts/detached-run.sh 各自的 --selftest。
逐项写开关名、弄坏哪一处、打开后哪几份自证的哪几项红（项名是那份自证的用例标签，括号里是失败行在标签之后的开头；
失败行 lib_heavy_tests.py 以「✗ lib_heavy_tests 自检：」起头，heavy-test-guard.sh、bash-command-detector.sh 以「✗ 自检：」起头，
detached-run.sh 以「✗ 自证格不对：」起头、标签之后不另写期望值；heavy-test-guard.sh 的自证还原样转出 lib_heavy_tests.py 自证的失败行）：
  LIB_SHELL_WORDS_BREAK=arguments-as-executed  `bash -n 脚本` 的目标与 `python3 - 词…` 后面的词照旧当被执行的脚本交出
      lib_heavy_tests.py --selftest：
        python3 - 后面的门禁文件名是参数（门禁目录里跑）（「应当是 None，实际 commit-check-stage」）
        bash -n 只查门禁的语法（「应当是 None，实际 commit-check-stage」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那两项）：
        工具实现员 bash -n 查门禁的语法（「应当是 0，实际 2」）
        工具实现员在门禁目录里 python3 - 读几份门禁（2026-09-28 误拦的那一条原样，目录换成自检的临时工作区）（「应当是 0，实际 2」）
        实现员 bash -n 只查语法（「应当是 0，实际 2」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        看门狗:前台 bash -n 只查语法（「应当是 0，实际 1」）
      detached-run.sh --selftest 不红
  LIB_SHELL_WORDS_BREAK=python-options-by-dash  python_invocation() 退回「以 - 开头的词都当选项跳过、第一个不以 - 开头的词当脚本」：带值选项的值当成脚本名，-m、-c 不认
      lib_heavy_tests.py --selftest：
        python3 -W ignore 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -X dev 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -BW default（合写、值另起一个词）起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 --check-hash-based-pycs never 起广谱变异（「应当是 broad-mutation，实际 None」）
        cd 进 scripts 再 python3 -m broad-mutation（「应当是 broad-mutation，实际 None」）
        python3 -m research.scripts.broad-mutation（命名空间包）（「应当是 broad-mutation，实际 None」）
        python3 -m runpy broad-mutation（「应当是 broad-mutation，实际 None」）
        PYTHONPATH 指到 scripts 再 python3 -m broad-mutation（「应当是 broad-mutation，实际 None」）
        python3 -c runpy.run_path 起广谱变异（「应当是 broad-mutation，实际 None」）
        python3 -c exec(open(驱动).read())（「应当是 broad-mutation，实际 None」）
        python3 -c runpy.run_module 按模块名起（「应当是 broad-mutation，实际 None」）
        PYTHONPATH 指到 scripts 再 python3 -m 起判法脚本跑必红世界（「应当是 verus-stage，实际 None」）
        export PYTHONPATH 再 python3 -m 起红例重放（「应当是 verus-stage，实际 None」）
        python3 -c runpy.run_path 起判法脚本跑必红世界（「应当是 verus-stage，实际 None」）
        经 PYTHONPATH 起判法脚本的 --selftest：不在登记表的位置上，照旧算提交时才跑的检查（「应当是 commit-check-selftest，实际 None」）
        python3 -c 里改 sys.argv 成 proof、代码之后写 tool-versions（「应当是 verus-stage，实际 None」）
        python3 -c 里改 sys.argv 成 --replay、代码之后不写词（「应当是 verus-stage，实际 None」）
        python3 -c 里 __import__ 按模块名起判法脚本验证明（「应当是 verus-stage，实际 None」）
        python3 -c 里 importlib.import_module 起红例重放（「应当是 verus-stage，实际 None」）
        python3 -c 里 spec_from_file_location 载入判法脚本再调 run_check、代码之后写 toolchain（「应当是 verus-stage，实际 None」）
        python3 -c 的 print 包着 __import__ 起判法脚本（「应当是 verus-stage，实际 None」）
        python3 -c 的 print 包着 __import__、代码只有这一句（「应当是 verus-stage，实际 None」）
        python3 -c runpy.run_path 起红例钉表、代码之后写 --check（「应当是 verus-stage，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那 23 项）：
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh、detached-run.sh 的 --selftest 不红
  LIB_SHELL_WORDS_BREAK=detached-run-force-kept  detached_run_arguments_after_preflight() 原样交回参数：-- 之后、选项值位置上的 --force 不摘
      lib_heavy_tests.py --selftest：
        detached-run.sh start 的 -- 之后先写 --force 照摘（「应当是 full-cargo，实际 None」）
      heavy-test-guard.sh --selftest（另转出 lib_heavy_tests.py 那一项）：
        主 agent 不带前缀经 detached-run.sh start、-- 之后先写 --force 再起全量照拒（「应当是 2，实际 0」）
        主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒（「应当是 2，实际 0」）
        主 agent 不带前缀经 detached-run.sh start、--label 后面先写 --force 再起全量照拒：读进去几份脚本（「应当是 0，实际 1」）
        lib_heavy_tests.py 的自检（「应当是 0，实际 1」）
      bash-command-detector.sh --selftest：
        放出追踪:detached-run.sh start 的 -- 之后先写 --force、里面 setsid -f 照拒（「应当是 1，实际 0」）
      detached-run.sh --selftest：
        Ⓟ -- 之后的 --force 照 preflight 摘：命令收到 a b，闸切出的词与登记里的命令逐字相同
"""
import os
import re
from typing import NamedTuple

SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}
SHELL_OPERATORS = sorted(["&&", "||", ";;", "|&", "&>>", "&>", ">>", ">&", "<&", ">|", "<<<", "<<-", "<<", "<>",
                          ";", "&", "|", "(", ")", "<", ">", "\n"], key=len, reverse=True)
OPERATOR_START = set(";&|()<>\n")
COMMAND_SEPARATORS = {"&&", "||", ";;", "|&", ";", "&", "|", "(", ")", "\n"}
OUTPUT_REDIRECTS = {">", ">>", ">|", "&>", "&>>", ">&"}
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
SOURCE_TARGET = re.compile(r"^[^\s=,-][^\s=,]*$")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)(\w+)\1([^\n]*)\n(.*?)\n\s*\2[ \t]*(?=\n|$)", re.S)
MAXIMUM_NESTING = 8
# 先带一个参数、再接那条命令的包装脚本：capped.sh N（线程上限）、run-with-memory-cap.sh <上限>（内存上限）。剥掉它们两个词，往里判那条命令
WRAPPERS_TAKING_ONE_ARGUMENT = {"capped.sh", "run-with-memory-cap.sh"}
# 其中给命令定内存上限的那一个：剥掉它的命令记 memory_capped（heavy-test-guard.sh 判子 agent 跑编译出来的代码经没经它）
MEMORY_CAP_WRAPPER = "run-with-memory-cap.sh"
# scripts/hosts/detached-run.sh（脱离会话长跑的入口）：两个 hook 都要认它，按名字（路径不论）、解到的真实路径（符号链接）与文件开头的签名行
# （行首 `# detached-run-script-signature:`，改名或拷走的一份）认；读文件开头这么多字节找签名行。这份文件自己的开头里没有以它起头的行
DETACHED_RUN_SCRIPT_NAME = "detached-run.sh"
DETACHED_RUN_SCRIPT_LOCATION = "scripts/hosts/detached-run.sh"
DETACHED_RUN_SIGNATURE_LINE = re.compile(rb"^# detached-run-script-signature:", re.MULTILINE)
DETACHED_RUN_HEAD_BYTES = 4096
# 它的 start 带值的选项（`--wall-clock <within-2h|over-2h>`、`--label <名>`、`--log <文件>`、`--stop-timeout <秒>`，也可以写成 `--选项=值`）
DETACHED_RUN_OPTIONS_WITH_VALUE = frozenset({"--wall-clock", "--label", "--log", "--stop-timeout"})
# 剥掉的 heredoc 在剥完的文字里换成这个记号（NUL 开头，不是 \w，不会被 HEREDOC 再认一次），切词之后它是 `<<` 的目标词，按编号找回正文
HEREDOC_BODY_MARKER = re.compile(r"^\x00heredoc-body-(\d+)\x00$")


class HeredocBody(NamedTuple):
    operator: str                 # "<<" 或 "<<-"（后者的正文已经去掉每行开头的制表符）
    is_delimiter_quoted: bool     # 结束词带引号：正文照原样写出；不带引号时 shell 还要展开正文里的 $ 与命令替换，这里给的是展开之前的原文
    text: str                     # 正文，最后一行之后带一个换行（写进文件的就是这些字节）


class PrefixSkip(NamedTuple):
    command_position: int | None  # 命令词的下标；这一条只有关键字、赋值与前缀时是 None
    prefix_names: set[str]         # 路过的前缀名（nohup、setsid、nice……）
    assignments: dict[str, str]    # 路过的赋值：命令前的 `名字=值` 与 env 参数里的 `名字=值`


class ShellInvocation(NamedTuple):
    code_string: str | None   # `bash -c '…'` 的那段代码
    script_index: int | None  # 起的是脚本时，脚本名在参数里的下标；只查语法（-n）或没给脚本时是 None


class PythonInvocation(NamedTuple):
    code_string: str | None   # `python3 -c '…'` 的那段代码
    module: str | None        # `python3 -m <模块>` 的模块名（`-m runpy <模块>` 取后面那个）
    script_index: int | None  # 起的是脚本文件时，脚本名在参数里的下标；-c / -m 时是程序自己的第一个参数的下标（可以等于参数个数）；
                              # 程序从标准输入读（`-`）或只有选项时是 None


class CommandAtPosition(NamedTuple):
    words: list[str]              # words[0] 是命令词（照写的样子，没取 basename），其后是参数
    directory: str | None         # 跑这条命令时的当前目录；cd 到变量路径之后认不出，是 None
    environment: dict[str, str]   # 这条命令看得到的、这段命令文本里设过的变量：前面 export 的、命令前与 env 里写的、外层传进来的
    launcher: str | None = None   # words[0] 是经什么起的脚本："shell"（bash / sh 这类后面跟的）、"source"（source / .）、
                                  # "python"（python3 后面跟的程序）；直接执行的命令（含 capped.sh N 后面那条）是 None
    output_redirections: tuple = ()      # 这一条简单命令上的输出重定向：((运算符, 目标词), …)，运算符是 OUTPUT_REDIRECTS 之一；fd 号不在里面
    standard_input_heredocs: tuple = ()  # 喂给这一条的、被当数据剥掉的 heredoc：(HeredocBody, …)，按出现次序（最后一个才是它读到的标准输入）
    memory_capped: bool = False          # 经 run-with-memory-cap.sh 跑：它自己的包装里有，或外层（bash -c 的那段代码、起它所在脚本的那一条）有


class ShellScan(NamedTuple):
    commands: list[CommandAtPosition]  # 命令位置上的每一条命令，包装（bash -c、capped.sh、source……）剥掉之后的那一条
    prefix_names: set[str]             # 哪一层、哪一条命令路过的前缀名都算
    has_lone_ampersand: bool           # 哪一层有单独的 &（放后台；不是 &&、>&、&>、|&）
    output_targets: list[str]          # 哪一层的输出重定向目标都算


def heredoc_feeds_shell(head):
    """heredoc 所在那一行 `<<` 之前的文字：它挂在的那条简单命令（最后一条），剥掉前缀与 capped.sh N 之后，
    是不是从标准输入读命令的 bash / sh 这类（不带 -c、不带脚本，或带 -s）。同一行别处出现 shell 的名字（`x.sh`、前一条 bash）不算。"""
    simple = simple_commands(shell_tokens(head))[0]
    if not simple or not simple[-1]:
        return False
    words = simple[-1]
    for _ in range(MAXIMUM_NESTING):
        skipped = skip_prefixes(words)
        if skipped.command_position is None:
            return False
        words = words[skipped.command_position:]
        name, arguments = os.path.basename(words[0]), words[1:]
        if name in WRAPPERS_TAKING_ONE_ARGUMENT and len(arguments) >= 2 and not arguments[0].startswith("-"):
            words = arguments[1:]
            continue
        if name not in SHELL_NAMES:
            return False
        invocation = shell_invocation(arguments)
        if invocation.code_string is not None:
            return False
        reads_standard_input = any(argument.startswith("-") and not argument.startswith("--") and "s" in argument[1:]
                                   for argument in arguments)
        return reads_standard_input or invocation.script_index is None
    return False


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


def backquote_end(text, start):
    """反引号之后（start 指向开头那个反引号后面第一个字符）找收尾的反引号，交回它后面那个下标；没配上交 -1。"""
    position = start
    while position < len(text):
        if text[position] == "\\":
            position += 2
        elif text[position] == "`":
            return position + 1
        else:
            position += 1
    return -1


def double_quote_end(text, start):
    """双引号之后找收尾的双引号，交回它后面那个下标；里面的 `$(…)` 与反引号整段跳过（它们里面的双引号不算收尾）；没配上交 -1。"""
    position = start
    while position < len(text):
        character = text[position]
        if character == "\\":
            position += 2
        elif character == '"':
            return position + 1
        elif text.startswith("$(", position):
            position = command_substitution_end(text, position + 2)
        elif character == "`":
            position = backquote_end(text, position + 1)
        else:
            position += 1
        if position < 0:
            return -1
    return -1


def command_substitution_end(text, start):
    """`$(` 之后（start 指向它后面第一个字符）找配对的 `)`，交回它后面那个下标；引号、反斜杠、反引号与嵌套的括号都跳过；没配上交 -1。"""
    depth, position = 1, start
    while position < len(text):
        character = text[position]
        if character == "\\":
            position += 2
        elif character == "'":
            end = text.find("'", position + 1)
            position = -1 if end < 0 else end + 1
        elif character == '"':
            position = double_quote_end(text, position + 1)
        elif character == "`":
            position = backquote_end(text, position + 1)
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
            return -1
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
    里面的正文作为 (None, 正文) 紧跟在那个词后面交出；`)` 之后紧跟的字符接在同一个词上，隔了空白的是下一个词。"""
    tokens, word, position, pending_substitutions = [], None, 0, []
    def flush():
        nonlocal word
        if word is not None:
            tokens.append((False, word))
            word = None
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
                word = (word or "") + text[position + 1:position + 2]
            position += 2
        elif character == "'":
            end = text.find("'", position + 1)
            end = len(text) if end < 0 else end
            word = (word or "") + text[position + 1:end]
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
            word = (word or "") + piece
            position += 1
        elif character == "#" and word is None:
            end = text.find("\n", position)
            position = len(text) if end < 0 else end
        elif starts_substitution(text, position):
            end = take_substitution(position)
            word = (word or "") + text[position:end]
            position = end
        elif character in OPERATOR_START:
            operator = next(candidate for candidate in SHELL_OPERATORS if text.startswith(candidate, position))
            if operator[0] in "<>" and word is not None and word.isdigit():
                word = None
            flush()
            tokens.append((True, operator))
            position += len(operator)
        else:
            word = (word or "") + character
            position += 1
    flush()
    return tokens


def simple_commands(tokens):
    """按命令边界切成一条条简单命令（只留词，重定向连同目标去掉），另交整段里输出重定向的目标，
    每条简单命令里的命令替换正文，以及每条简单命令上的重定向 ((运算符, 目标词), …)（后两样与命令一一对应；只有重定向、没有词的那一条，词是空列表）。"""
    commands, substitutions, current, current_substitutions, output_targets = [], [], [], [], []
    redirections, current_redirections = [], []
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
    return commands, output_targets, substitutions, redirections


def skip_prefixes(words):
    """跳过关键字、赋值与 nohup / setsid / nice / timeout / env / taskset 这类前缀，交回命令词的下标、路过的前缀名与赋值。"""
    prefix_names, assignments = set(), {}
    position = 0
    while position < len(words):
        word = words[position]
        name = os.path.basename(word)
        assignment = ASSIGNMENT.match(word)
        if word in COMMAND_KEYWORDS:
            position += 1
            continue
        if assignment:
            assignments[assignment.group(1)] = assignment.group(2)
            position += 1
            continue
        if name not in PREFIX_OPTIONS_WITH_VALUE:
            return PrefixSkip(position, prefix_names, assignments)
        if name == "command" and words[position + 1:position + 2] in (["-v"], ["-V"]):
            return PrefixSkip(position, prefix_names, assignments)  # command -v / -V 是查名字，不执行后面那个词
        prefix_names.add(name)
        position += 1
        positional_left = PREFIX_POSITIONAL_COUNT.get(name, 0)
        while position < len(words):
            argument = words[position]
            if argument in PREFIX_OPTIONS_WITH_VALUE[name]:
                position += 2
            elif argument.startswith("-") and argument != "-":
                position += 1
            elif name == "env" and ASSIGNMENT.match(argument):
                environment_assignment = ASSIGNMENT.match(argument)
                assignments[environment_assignment.group(1)] = environment_assignment.group(2)
                position += 1
            elif positional_left:
                positional_left -= 1
                position += 1
            else:
                break
    return PrefixSkip(None, prefix_names, assignments)


def arguments_as_executed():
    """弄坏开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed：`bash -n 脚本` 与 `python3 - 词…` 里的参数照旧当被执行的脚本（只给自检证红用）。"""
    return "arguments-as-executed" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(",")


def shell_invocation(arguments):
    """bash / sh 这类后面的参数：是 -c 的一段代码、是起一个脚本，还是都不是（只查语法、从标准输入读）。"""
    option_count, only_checks_syntax = 0, False
    while option_count < len(arguments):
        argument = arguments[option_count]
        if not argument.startswith(("-", "+")) or argument in ("-", "--"):
            break
        option_count += 1
        if argument in ("-o", "+o", "-O", "+O"):
            option_count += 1
            continue
        letters = "" if argument.startswith("--") else argument[1:]
        only_checks_syntax = (only_checks_syntax or "n" in letters) and not arguments_as_executed()
        if "c" in letters:
            return ShellInvocation(arguments[option_count] if option_count < len(arguments) else "", None)
    if option_count >= len(arguments) or only_checks_syntax:
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
        return os.path.normpath(path)
    if directory is None:
        return None
    return os.path.normpath(os.path.join(directory, path))


def carries_detached_run_signature(head):
    """一份文件开头的字节（只看前 DETACHED_RUN_HEAD_BYTES 个）里有没有 detached-run.sh 的签名行（行首）。"""
    return bool(DETACHED_RUN_SIGNATURE_LINE.search(head[:DETACHED_RUN_HEAD_BYTES]))


def detached_run_script(word, directory, by_name_only=False):
    """命令词是不是 scripts/hosts/detached-run.sh：是就交一句凭什么认出来的，不是交 None。三种办法：名字叫 detached-run.sh（路径不论）；
    解到的文件的真实路径落在 scripts/hosts/detached-run.sh（符号链接）；文件开头有签名行（改名或拷走的一份）。
    后两种读盘：相对路径按 directory 解（directory 是 None 时只认绝对路径），带没展开的变量、命令替换、通配的词不解；文件不在就认不出。
    by_name_only 为真时只按名字认（两个 hook 的弄坏开关用）。"""
    expanded = os.path.expanduser(word)
    if os.path.basename(expanded) == DETACHED_RUN_SCRIPT_NAME:
        return word
    if by_name_only or not expanded or set("$`*?[") & set(expanded):
        return None
    resolved = resolve_path(directory, expanded)
    if resolved is None or not os.path.isfile(resolved):
        return None
    real = os.path.realpath(resolved)
    if real.endswith("/" + DETACHED_RUN_SCRIPT_LOCATION):
        return f"{word}（符号链接，指到 {DETACHED_RUN_SCRIPT_LOCATION}）"
    try:
        with open(real, "rb") as handle:
            head = handle.read(DETACHED_RUN_HEAD_BYTES)
    except OSError:
        return None
    if carries_detached_run_signature(head):
        return f"{word}（开头带 {DETACHED_RUN_SCRIPT_LOCATION} 的签名行：改名或拷走的一份）"
    return None


def detached_run_arguments_after_preflight(arguments):
    """detached-run.sh 脚本名之后的参数经它开头的 preflight 之后是什么：每一个整词 `--force` 都摘掉（.claude/singlefs-ai-sop/scripts/preflight.sh
    逐词比，连 `--` 之后的、连写在 `--label` 这类选项值位置上的）。弄坏开关 LIB_SHELL_WORDS_BREAK=detached-run-force-kept 下原样交回。"""
    if "detached-run-force-kept" in os.environ.get("LIB_SHELL_WORDS_BREAK", "").split(","):
        return list(arguments)
    return [argument for argument in arguments if argument != "--force"]


def detached_run_start_command(arguments):
    """detached-run.sh 脚本名之后的参数是 `[--force] start <选项> -- <命令…>` 时交回 <命令…> 的词（它把这条命令挂到用户级 systemd 下跑）；
    先照 preflight 摘掉每一个 --force（detached_run_arguments_after_preflight），交回的就是单元里真跑的那几个词。
    别的子命令（list、watch、stop、--selftest）、没有 `--`、`--` 之后是空的、带认不出的选项（它按用法错拒、什么都不跑）交 None。"""
    arguments = detached_run_arguments_after_preflight(arguments)
    position = 0
    while position < len(arguments) and arguments[position] == "--force":
        position += 1
    if arguments[position:position + 1] != ["start"]:
        return None
    position += 1
    while position < len(arguments):
        word = arguments[position]
        if word == "--":
            return arguments[position + 1:] or None
        if word in DETACHED_RUN_OPTIONS_WITH_VALUE:
            position += 2
        elif word == "--force" or word.partition("=")[0] in DETACHED_RUN_OPTIONS_WITH_VALUE:
            position += 1
        else:
            return None
    return None


def changed_directory(directory, arguments):
    """cd / pushd 之后的当前目录；目标是 `-` 或变量、命令替换时认不出，交 None。"""
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    if not targets:
        return os.path.expanduser("~")
    target = os.path.expanduser(targets[0])
    if target == "-" or "$" in target or "`" in target:
        return None
    return resolve_path(directory, target)


def commands_at_command_position(text, directory=None, environment=None, depth=0, stripped_heredocs=None, memory_capped=False):
    """整段命令文本里命令位置上的每一条命令（前缀与包装剥掉、cd 与 export 跟着走），连同各层的放后台与输出重定向写法。
    stripped_heredocs 是递归时往里传的、各层剥掉的 heredoc 正文（记号里的下标指向它），调用方不用给。
    memory_capped：这段文本整个跑在 run-with-memory-cap.sh 里（它是经它起的脚本的正文）；每条命令的 memory_capped 从它起算。
    喂给 shell 的 heredoc 正文当同一段文本里的后续命令切，挂 heredoc 的那条命令上的包装不算在正文的命令上。"""
    if stripped_heredocs is None:
        stripped_heredocs = []
    tokens = shell_tokens(strip_data_heredocs(text, stripped_bodies=stripped_heredocs))
    simple, output_targets, substitutions, redirections = simple_commands(tokens)
    commands, prefix_names = [], set()
    has_lone_ampersand = (True, "&") in tokens
    exported = dict(environment or {})
    for words, command_substitutions, command_redirections in zip(simple, substitutions, redirections):
        for substitution in command_substitutions:
            if depth < MAXIMUM_NESTING:
                inner = commands_at_command_position(substitution, directory, exported, depth + 1, stripped_heredocs, memory_capped)
                commands += inner.commands
                prefix_names |= inner.prefix_names
                has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                output_targets += inner.output_targets
        if not words:
            continue
        heredoc_indices = [HEREDOC_BODY_MARKER.match(target) for operator, target in command_redirections if operator in ("<<", "<<-")]
        attached = {
            "output_redirections": tuple((operator, target) for operator, target in command_redirections if operator in OUTPUT_REDIRECTS),
            "standard_input_heredocs": tuple(stripped_heredocs[int(found.group(1))] for found in heredoc_indices
                                             if found and int(found.group(1)) < len(stripped_heredocs)),
        }
        command_environment = dict(exported)
        remaining = words
        launcher = None
        command_memory_capped = memory_capped
        for _ in range(MAXIMUM_NESTING):
            skipped = skip_prefixes(remaining)
            prefix_names |= skipped.prefix_names
            command_environment.update(skipped.assignments)
            if skipped.command_position is None:
                break
            remaining = remaining[skipped.command_position:]
            name, arguments = os.path.basename(remaining[0]), remaining[1:]
            if name == "export":
                for argument in arguments:
                    assignment = ASSIGNMENT.match(argument)
                    if assignment:
                        exported[assignment.group(1)] = assignment.group(2)
                break
            if name == "unset":
                for argument in arguments:
                    if not argument.startswith("-"):
                        exported.pop(argument, None)
                break
            if name in ("cd", "pushd"):
                directory = changed_directory(directory, arguments)
                break
            if name in SHELL_NAMES:
                invocation = shell_invocation(arguments)
                if invocation.code_string is not None:
                    if depth < MAXIMUM_NESTING:
                        inner = commands_at_command_position(invocation.code_string, directory, command_environment, depth + 1, stripped_heredocs,
                                                             command_memory_capped)
                        commands += inner.commands
                        prefix_names |= inner.prefix_names
                        has_lone_ampersand = has_lone_ampersand or inner.has_lone_ampersand
                        output_targets += inner.output_targets
                    break
                if invocation.script_index is None:
                    commands.append(CommandAtPosition(remaining, directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments[invocation.script_index:]
                launcher = "shell"
                continue
            if PYTHON_NAME.match(name):
                invocation = python_invocation(arguments)
                if invocation.module:
                    # `python3 -m <模块>`：被执行的是 sys.path 上的 <模块>.py（当前目录排第一），交出这个名字，调用方按位置认
                    remaining = [invocation.module.replace(".", "/") + ".py"] + arguments[invocation.script_index:]
                    launcher = "python"
                    continue
                if invocation.code_string is not None or invocation.script_index is None:
                    # `python3 -c '…'`、`python3 -`、只有选项：没有被执行的脚本文件，交出 python 这一条本身
                    commands.append(CommandAtPosition(remaining, directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments[invocation.script_index:]
                launcher = "python"
                continue
            if name in ("source", "."):
                if not arguments or not SOURCE_TARGET.match(arguments[0]):
                    commands.append(CommandAtPosition(remaining, directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments
                launcher = "source"
                continue
            if name in WRAPPERS_TAKING_ONE_ARGUMENT:
                if not arguments or arguments[0].startswith("-") or len(arguments) < 2:
                    commands.append(CommandAtPosition(remaining, directory, command_environment, **attached, memory_capped=command_memory_capped))
                    break
                remaining = arguments[1:]
                launcher = None
                command_memory_capped = command_memory_capped or name == MEMORY_CAP_WRAPPER
                continue
            commands.append(CommandAtPosition(remaining, directory, command_environment, launcher, **attached, memory_capped=command_memory_capped))
            break
    return ShellScan(commands, prefix_names, has_lone_ampersand, output_targets)
