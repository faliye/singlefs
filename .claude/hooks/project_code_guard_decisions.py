# 项目代码闸 project-code-guard.sh 的 python 部分之一：共用库的读入、弄坏开关名单、clippy lint 表、直接跑项目代码的认法（扫命令文本与读进来的脚本、cp 与 tee 与 heredoc 写出的文件）、绕开服务、临时服务、范围声明与提交标识、拒绝说明、检出记录。
# 入口 project-code-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 PROJECT_CODE_GUARD_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import glob, importlib.util, json, os, re, shlex, shutil, subprocess, sys, tempfile, time, types
from datetime import datetime, timezone
from typing import NamedTuple

# 钩子目录：部分文件与入口在同一个目录里（入口的引导把 argv[1] 放在 sys.path 最前、再 import 它们），按本文件自己的位置取。
# 同目录的 lib_*.py 按名字 import 或按文件路径载入，路径都从这一个取；算键库认得 __file__、不认部分文件里的 sys.argv。
HOOK_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
PROJECT_CODE_RUNS_LIBRARY_NAME = "lib_project_code_runs.py"

def project_code_runs_library_path(hook_dir):
    return os.path.join(hook_dir, PROJECT_CODE_RUNS_LIBRARY_NAME)

def load_project_code_runs():
    """判定的共用模块（同目录的 lib_project_code_runs.py，它再导入同目录的 lib_shell_words.py）；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(HOOK_DIRECTORY, PROJECT_CODE_RUNS_LIBRARY_NAME))
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    project_code_runs = load_project_code_runs()
    shell_words, shared_library_error = project_code_runs.shell_words, None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    project_code_runs, shell_words, shared_library_error = None, None, error

def shared_library_problem(hook_dir):
    return f"读不到共用模块 {project_code_runs_library_path(hook_dir)}（它再导入同目录的 lib_shell_words.py）：{shared_library_error!r}"

try:   # 检出记录的默认位置在工作根下，工作根只在同目录的 lib_work_root.py 里定义（读不到时检出记录不写）
    sys.dont_write_bytecode = True
    import lib_work_root as work_roots
    DEFAULT_DETECTIONS = work_roots.default_detections_path()
except Exception:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, DEFAULT_DETECTIONS = None, None
try:   # 主工作区里直接起、编译目录会落进主仓的 cargo 的判法在同目录的 lib_workspace_cargo_lock.py（读不到时这一道不判，命令里有 cargo 的记一条检出）
    cargo_lock_spec = importlib.util.spec_from_file_location("lib_workspace_cargo_lock", os.path.join(HOOK_DIRECTORY, "lib_workspace_cargo_lock.py"))
    cargo_lock = importlib.util.module_from_spec(cargo_lock_spec)
    cargo_lock_spec.loader.exec_module(cargo_lock)
    cargo_lock_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    cargo_lock, cargo_lock_error = None, error
try:   # clippy lint 表的读法在同目录的 lib_code_discipline_lints.py，与检出钩子 bash-command-detector.sh 的 ㉓ 出路共用同一份（读不到时 ⑤ 不判）
    import lib_code_discipline_lints as discipline_lints
except Exception:  # 文件不在、语法错都算读不到
    discipline_lints = None
MAXIMUM_SCRIPT_DEPTH = 6        # 从命令文本起一层层读进去的脚本最多几层；再往里判「看不全」
MAXIMUM_SCRIPT_BYTES = 1 << 20  # 脚本超过这么大不读，判「读不到」
# 直接执行的文件先读开头这么多判是不是 shell 脚本（ELF 魔数、NUL 字节、`#!` 指到别的解释器、没有 `#!` 而扩展名不是 .sh），不是就不读也不记，之后才判大小上限：
# 编译产物常有几 MB，先判大小会把它记成「超过上限、没判」（例：scripts/experiment-apparatus/target/release/e49_chain_width 5124256 字节）
SCRIPT_HEAD_BYTES = 4096
SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有这个扩展名的当 shell 脚本读

# 弄坏开关 PROJECT_CODE_GUARD_BREAK=<项>（几个用逗号连；只给自检证明点名的几格会红，项名与红的格写在文件头）
GUARD_BREAK_VARIABLE = "PROJECT_CODE_GUARD_BREAK"
KNOWN_GUARD_BREAK_ITEMS = ("disable-check", "ignore-written-scripts",
                           "detection-outcome-dropped", "copy-without-run-scope",
                           "failure-unrecorded", "copy-by-cp-only", "heredoc-write-every-redirection", "cat-concatenation-unseen",
                           "unrecognized-writes-allowed", "path-scripts-unread", "copy-redirections-unseen", "tee-copy-unseen", "launcher-functions-unjudged",
                           "glob-scripts-unread", "wrapper-copies-trusted", "cat-missing-source-unexplained", "bash-layer-unrecorded", "unjudged-shell-input-allowed",
                           "cat-append-unreadable-destination-raises", "relative-script-undeterminable-directory-allowed",
                           "bash-env-forwarding-untracked", "copied-shell-binary-untracked", "compound-stdin-untracked", "cat-missing-source-allowed",
                           "python-missing-unrecorded", "program-files-unread", "embedded-unjudged-allowed",
                           "launcher-recorded-as-writer", "heredoc-targets-unexpanded", "program-file-variables-unexpanded",
                           "task-server-direct-allowed", "subagent-task-client-allowed", "task-server-bypass-allowed", "task-server-copy-dropped",
                           "temporary-service-bare-allowed", "temporary-service-wrapper-not-marked", "temporary-service-submit-inner-unread", "temporary-service-remedy-dropped",
                           "python-filter-code-judged", "copy-test-names-dropped", "copy-without-full-gate-flag",
                           "clippy-lint-table-unchecked", "clippy-lint-table-remedy-dropped", "temporary-service-stage-samples-allowed",
                           "clippy-lint-weakening-unchecked", "clippy-lint-unreadable-table-passed",
                           "part-unreadable-passes")   # 最后一项只在入口的引导里读（引导在这几份载入之前就要认它），名单里留着，免得「项都认得」一格把它当写错的项
def guard_break_is_set(item):
    return item in os.environ.get(GUARD_BREAK_VARIABLE, "").split(",")

class UseAt(NamedTuple):
    test: object           # lib_project_code_runs.ProgramUse
    frames: tuple          # 读到它经过的脚本：((脚本路径, 行号或 None), …)，最后一格是它自己所在的那一行；命令文本里直接写的是 ()

class Notice(NamedTuple):
    frames: tuple          # 同 UseAt.frames：出在哪个脚本的哪一行，命令文本里直接写的是 ()
    text: str              # 读不到、看不全的是什么

class Scan(NamedTuple):
    uses: list             # [UseAt]
    notices: list          # [Notice]：读不到、看不全的脚本，这条命令照常执行，记检出
    scripts_read: list     # 读进去判过的脚本路径，判一遍记一次，复用上一遍的结果不记（放行时分得出「读了、里面没有」与「根本没读」）
    hit_depth_limit: bool  # 这一层或更里面有脚本因为嵌套超过上限没读：结果跟着从第几层起走
    lowest_cycle_frame: int | None  # 这一层或更里面绕回外层脚本而没读时，绕回的那一格在帧里的最小下标；没绕回是 None
    direct_runs: list = [] # [DirectRun]：不经 task-server 跑项目代码的每一处（文件头「经 task-server 起」①），与子 agent 不起的那几个客户端子命令（②）
    bypasses: list = []    # [Bypass]：绕开 task-server 的写法（文件头「经 task-server 起」③）
    listed_scripts: list = []  # [(真实路径, "anyone" 或 "main-agent")]：起过的名单上直接跑的脚本（读它的正文时里面的那几行不另算不经 task-server）

# DirectRun.allowed 的一种：子 agent 起提交客户端的 submit、wait、cancel、clean（文件头「经 task-server 起」②；lib_project_code_runs.subagent_task_server_refusal），只给主 agent
SUBAGENT_CLIENT_LEVEL = "main-agent-client"
# DirectRun.allowed 的另一种：要临时 prune-server 的自证写成了裸的 --selftest（文件头「经 task-server 起」④；lib_project_code_runs.temporary_service_bare_selftest），谁起都拒
TEMPORARY_SERVICE_BARE_LEVEL = "temporary-service-bare"
# DirectRun.allowed 的另一种：门禁样本档点名的阶段要临时 prune-server、外面没套包装（文件头「经 task-server 起」⑥；lib_project_code_runs.temporary_service_bare_stage_samples），谁起都拒
TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL = "temporary-service-stage-samples"
# DirectRun.allowed 的另一种：cargo clippy 没带 -D warnings 与 check.sh 的 CODE_DISCIPLINE_LINTS 那张表的每一条（文件头「经 task-server 起」⑤；clippy_lint_table_missing），谁起都拒
CLIPPY_LINT_TABLE_LEVEL = "clippy-lint-table-missing"
# lint 表的来源：这一份钩子所在的仓里的 check.sh（钩子在 <仓>/.claude/hooks/ 下），每次判都现读
CLIPPY_LINT_TABLE_SOURCE = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(sys.argv[1]))), ".claude", "singlefs-ai-sop", "scripts", "check.sh")
CLIPPY_NOT_JUDGED_ARGUMENTS = frozenset({"--help", "-h", "--version", "-V", "--fix", "--explain"})   # 只问帮助与版本的、改代码的、讲一条 lint 的 clippy 不是门禁那一种
CLIPPY_DENY_WARNINGS = "warnings"


def code_discipline_lints():
    """check.sh 里 CODE_DISCIPLINE_LINTS=( … ) 那张表被点名的每个名字（lib_code_discipline_lints.table_names，与检出钩子的 ㉓ 出路读同一份）；读不到文件、没有那一段、一个词都没读到交 None。"""
    return discipline_lints.table_names(CLIPPY_LINT_TABLE_SOURCE) if discipline_lints is not None else None


def clippy_denied_names(arguments):
    """clippy 命令 `--` 之后的参数里被 -D / --deny 点名的那几个名字（lib_code_discipline_lints.denied_names）。"""
    return discipline_lints.denied_names(arguments)


def clippy_lint_table_missing(words, directory):
    """文件头「经 task-server 起」⑤：剥掉前缀与包装的一条命令是 cargo clippy，且 check.sh 里 CODE_DISCIPLINE_LINTS 有这一段而写法读不成、或它的 `--` 之后缺 `-D warnings` 与表里的哪几条、
    或在 deny 之后又 -A、-W、--allow、--warn 放行或降级了表里的 lint（点名组名与 warnings 的同样算）、或带 --cap-lints，交一句说明（点名读不成的原因、缺的、放行的）；
    不是 clippy、只问帮助与版本、带 --fix 或 --explain、`--` 前后写成变量或命令替换（认不出）、文件读不到或没有这一段、没问题的，交 None。"""
    if os.path.basename(words[0]) != "cargo":
        return None
    found = project_code_runs.cargo_subcommand(words[1:], directory)
    if found is None or found.subcommand != "clippy" or CLIPPY_NOT_JUDGED_ARGUMENTS & set(found.rest):
        return None
    if any("$" in word or "`" in word for word in found.rest):
        return None
    problem = clippy_lint_table_problem()
    if problem is not None:
        return (f"check.sh 里 CODE_DISCIPLINE_LINTS 那张表这一刻读不成（{problem}），这里读到的表与整轮门禁真传给 clippy 的会对不上，clippy 命令一概不过这一道：{' '.join(words)[:300]}")
    lints = code_discipline_lints()
    if lints is None:
        return None
    after_separator = found.rest[found.rest.index("--") + 1:] if "--" in found.rest else []
    denied = clippy_denied_names(after_separator)
    missing = [name for name in [CLIPPY_DENY_WARNINGS] + lints if name not in denied]
    if missing:
        return f"cargo clippy 的 `--` 之后缺 {'、'.join(missing)}（整轮门禁与合入后验证跑的 clippy 带 -D warnings 与 check.sh 的 CODE_DISCIPLINE_LINTS 那张表）：{' '.join(words)[:300]}"
    if not guard_break_is_set("clippy-lint-weakening-unchecked"):
        weakened = discipline_lints.weakened_names(after_separator, lints)
        if weakened:
            return (f"cargo clippy 的 `--` 之后在 deny 之后又用 -A、-W、--allow、--warn 放行或降级了 {'、'.join(weakened)}（deny 白写了，门禁带的是 deny、这一条命令跑出来不是）：{' '.join(words)[:300]}")
        if discipline_lints.caps_lints(after_separator):
            return f"cargo clippy 的 `--` 之后带了 --cap-lints（它把全部 lint 压到指定级别，前面的 -D 不再起作用）：{' '.join(words)[:300]}"
    return None


def clippy_lint_table_problem():
    """check.sh 里 CODE_DISCIPLINE_LINTS=( … ) 有这一段而写法读不成时的原因（lib_code_discipline_lints.table_problem）；读得成、没有这一段、读不了文件、读不到共用模块交 None。
    弄坏开关 clippy-lint-unreadable-table-passed 下一概交 None。"""
    if discipline_lints is None or guard_break_is_set("clippy-lint-unreadable-table-passed"):
        return None
    return discipline_lints.table_problem(CLIPPY_LINT_TABLE_SOURCE)


def clippy_lint_table_tail():
    """现读的 `-- -D warnings <表里的词…>` 整段（lib_code_discipline_lints.table_tail；表读不到、读不成交空串）。"""
    return discipline_lints.table_tail(CLIPPY_LINT_TABLE_SOURCE) if discipline_lints is not None else ""


class DirectRun(NamedTuple):
    detail: str            # lib_project_code_runs.runs_compiled_code、starts_a_build 的说明，project_code_use 认出的那一份，剥掉的内存包装，或子 agent 不起的客户端子命令
    frames: tuple          # 同 UseAt.frames
    allowed: str = "nobody"  # "main-agent"：这一处是只给主 agent 直接跑的会话工具（lib_project_code_runs.MAIN_AGENT_TOOLS_RUN_DIRECTLY）；SUBAGENT_CLIENT_LEVEL：见上

class Bypass(NamedTuple):
    detail: str            # lib_project_code_runs.task_server_text_bypass / task_server_command_bypass 的说明
    frames: tuple          # 同 UseAt.frames

class WrittenScript(NamedTuple):
    text: str | None          # heredoc 写出的正文；cp 拷来的是 None
    copied_from: str | None   # cp 拷来的：源文件的绝对路径；heredoc 写出的是 None

class FileOnDisk(NamedTuple):
    head: bytes            # 开头 SCRIPT_HEAD_BYTES 字节（判是不是 shell 脚本用）；读不了是 b""
    data: bytes | None     # 整份；超过 MAXIMUM_SCRIPT_BYTES 或读不了是 None
    size: int
    error: str | None      # 读不了的原因（OSError 的 strerror）；读得了是 None

class JudgingState:
    """一次判定里命令文本与读进去的各层脚本共用的：写出的文件、从盘上读过的文件、判过的脚本。"""
    def __init__(self):
        self.written = {}          # 实际路径 → WrittenScript：同一条命令里到这里为止写出的文件（record_files_written）
        self.written_version = 0   # written 每变一次加一；判过的脚本只在它自那一遍起没变时复用
        self.files = {}            # 实际路径 → FileOnDisk：一次判定里同一份文件只从盘上读一遍
        self.files_read = []       # 从盘上读过的文件的实际路径，按读的次序，每份一次
        self.judged = {}           # (实际路径, 当前目录, written_version, 经没经内存包装, 参数…) → Scan，
                                   # 帧去掉了起它那一行之上的几格（复用时接上这一处的）
        self.writers = []          # 命令文本这一层里每条命令写了什么：(序号, 它点到的路径与文字, 它认得出写出内容的实际路径集, 命令名)；
                                   # 后面执行的脚本若被前面认不出写法的命令写过，照认不出拒（unrecognized_write）
        self.serial = 0            # 命令文本这一层里到这里为止判过几条命令（writers 里的序号）
        self.missing_cat_sources = []   # 几个源的 cat 里盘上不在的源：(源词, 目的词)；拒的时候拒绝说明点名它（cat 报错退 1，`&&` 后面其实不跑）

class Decision(NamedTuple):
    code: int              # 0 放行，2 拒绝
    message: str | None    # 拒绝时写给 stderr 的原因与出路
    notices: list          # 读不到、看不全的脚本（写好的句子，出在脚本里的带路径与行号）
    scripts_read: list     # 同 Scan.scripts_read
    files_read: list       # 同 JudgingState.files_read
    scan: object = None    # 这一次判的 Scan（task_server_decision 接着用；disable-check 下是 None）

# 命令位置上的词里有这些字符，就是没展开的变量、命令替换、通配、引号残片或别的语言的源码残片（逗号、等号），不是一个能读的路径
NOT_A_LITERAL_PATH = set("$`()'\"<>|&;*?[]{}=,\n")

LAUNCHER_WORDS = {"shell": "bash / sh 起的", "source": "source 读进来的",
                  "startup": "shell 启动时先读的（BASH_ENV / ENV、--rcfile、~/.bashrc、登录 shell 的 /etc/profile 与 HOME 下的 profile、/etc/bash.bashrc、zsh 的 .zshenv 与 .zshrc 指的）",
                  "program": "别的程序读进去当代码的（awk -f、sed -f、Makefile）", None: "直接执行的"}

def frames_text(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)

def command_lines(script_text):
    """脚本里每条命令（剥掉前缀与包装之后的词序列）出现在哪几行，按出现次序；
    逐个逻辑行（反斜杠续行接上）单独切词，喂给非 shell 命令的 heredoc 正文先换成同样多的空行，行号不变。"""
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    found, number = {}, 0
    while number < len(lines):
        first_line, logical = number + 1, lines[number]
        while logical.endswith("\\") and number + 1 < len(lines):
            number += 1
            logical += "\n" + lines[number]
        number += 1
        for command in shell_words.commands_at_command_position(logical).commands:
            found.setdefault(tuple(command.words), []).append(first_line)
    return found

def literal_path(command, word):
    """命令里的一个词当文件路径（按这条命令的当前目录解开、取实际路径）；带没展开的变量、通配这类的，或目录认不出，交 None。"""
    if not word or NOT_A_LITERAL_PATH & set(word):
        return None
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    return None if resolved is None else os.path.realpath(resolved)

def heredoc_write_targets(command):
    """这一条 cat / tee 把喂给它的 heredoc 写进了哪些文件：[(目标词, 是不是追加)]；不是这种写法交 []。"""
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name not in ("cat", "tee") or not command.standard_input_heredocs:
        return []
    targets = []
    if name == "cat":
        if any(argument != "-" for argument in arguments):  # 带了文件参数（写出的是那些文件）或别的选项（-n 这类改了内容）
            return []
    else:
        only_files = False
        for argument in arguments:
            if not only_files and argument == "--":
                only_files = True
            elif only_files or not argument.startswith("-") or argument == "-":
                if argument != "-":
                    targets.append(argument)
        appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:])
                      for argument in arguments if argument != "--")
        targets = [(target, appends) for target in targets]
    if guard_break_is_set("heredoc-write-every-redirection"):   # 弄坏开关：输出重定向的目标不看 fd 号一律当写了正文
        for operator, target in command.output_redirections:
            if operator == ">&" and (target.isdigit() or target == "-"):
                continue
            targets.append((target, operator in (">>", "&>>")))
        return targets
    # cat / tee 把正文写到标准输出：只有标准输出最后落到的那个文件拿到正文（`2> 文件`、`3> 文件` 拿不到，`3>d.sh >&3` 拿得到；lib_shell_words.py 的 standard_output_destination）
    destination = shell_words.standard_output_destination(command.output_redirections)
    if destination is not None:
        targets.append((destination[0], destination[1] in (">>", "&>>")))
    return targets

# 同一条命令里先拷出再执行：cp 之外，ln（-s 的符号链接与硬链接，执行时读到的都是源的内容）、install、mv 的目标与 `cat 源 > 目的` 也按源的内容认
#   （弄坏开关 PROJECT_CODE_GUARD_BREAK=copy-by-cp-only 下只认 cp）；install 另有三个带值的选项，-d / --directory 建目录不拷文件
COPYING_COMMANDS = {"cp", "ln", "install", "mv"}
INSTALL_OPTIONS_WITH_VALUE = {"-m", "--mode", "-o", "--owner", "-g", "--group"}

def cat_destination(command):
    """`cat 源… > 目的`（不带选项、不读 heredoc）的目的词：标准输出最后落到的、由 `>` 或 `>|` 打开的那个文件（照 fd 号与 `n>&m` 跟，`2> 文件` 不算，
    `3>d.sh >&3` 算；lib_shell_words.py 的 standard_output_destination）；不是这种写法交 None。"""
    destination = shell_words.standard_output_destination(command.output_redirections)
    sources = command.words[1:]
    # `&>`、`>& 文件` 同时写标准输出与标准错误（源都在时错误输出是空的）、`>>`、`&>>` 追加；弄坏开关 copy-redirections-unseen 下只认 `>`、`>|`
    operators = (">", ">|") if guard_break_is_set("copy-redirections-unseen") else (">", ">|", "&>", ">&", ">>", "&>>")
    if (os.path.basename(command.words[0]) != "cat" or destination is None or destination[1] not in operators or not sources
            or any(source.startswith("-") for source in sources) or command.standard_input_heredocs):
        return None
    return destination[0]

def cat_copy_pairs(command):
    """`cat 源 > 目的`（只一个源，`&>`、`>& 文件` 同）：[(源词, 目的词)]；别的写法交 []（-n 这类改了内容；几个源连起来的、追加的另由 cat_concatenation 认）。"""
    destination = cat_destination(command)
    if destination is None or len(command.words) != 2 or cat_appends(command):
        return []
    return [(command.words[1], destination)]

def cat_concatenation(command):
    """`cat 源 源… > 目的`（两个以上的源，`cat s.sh "2"> d.sh` 里引号写的 2 也是一个源）：(源词表, 目的词)；不是这种写法交 None。
    目的里是各个源依次接起来的内容（不在的源 cat 报错、不写东西，照空的算；record_files_written）。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=cat-concatenation-unseen 下不认（几个源的不当拷贝）。"""
    destination = cat_destination(command)
    if (destination is None or (len(command.words) < 3 and not cat_appends(command)) or guard_break_is_set("cat-concatenation-unseen")
            or guard_break_is_set("copy-by-cp-only")):
        return None
    return command.words[1:], destination


def cat_appends(command):
    """`cat 源… >> 目的` / `&>> 目的`：标准输出落到的那个文件是追加打开的。"""
    destination = shell_words.standard_output_destination(command.output_redirections)
    return destination is not None and destination[1] in (">>", "&>>")

def copy_pairs(command):
    """`cp`（与 ln / install / mv、`cat 源 > 目的`）这一条拷了哪些文件：[(源词, 目的词)]；不是这几样、或参数认不出，交 []。"""
    name = os.path.basename(command.words[0])
    cp_only = guard_break_is_set("copy-by-cp-only")
    if name == "cat" and not cp_only:
        return cat_copy_pairs(command)
    if name != "cp" and (cp_only or name not in COPYING_COMMANDS):
        return []
    if name == "install" and any(argument in ("-d", "--directory") for argument in command.words[1:]):
        return []
    operands, target_directory, no_target_directory, only_operands = [], None, False, False
    arguments, position = command.words[1:], 0
    while position < len(arguments):
        argument = arguments[position]
        if not only_operands and argument == "--":
            only_operands = True
        elif not only_operands and name == "install" and argument in INSTALL_OPTIONS_WITH_VALUE:
            position += 1
        elif not only_operands and argument in ("-t", "--target-directory", "-S", "--suffix"):
            if argument in ("-t", "--target-directory") and position + 1 < len(arguments):
                target_directory = arguments[position + 1]
            position += 1
        elif not only_operands and argument.startswith("--target-directory="):
            target_directory = argument.split("=", 1)[1]
        elif not only_operands and argument.startswith("-") and argument != "-":
            no_target_directory = no_target_directory or argument == "--no-target-directory" or (
                not argument.startswith("--") and "T" in argument[1:])
        else:
            operands.append(argument)
        position += 1
    if target_directory is not None:
        pairs = [(source, os.path.join(target_directory, os.path.basename(source))) for source in operands]
    elif len(operands) < 2:
        return []
    else:
        sources, destination = operands[:-1], operands[-1]
        destination_path = literal_path(command, destination)
        into_directory = not no_target_directory and (len(sources) > 1 or destination.endswith("/")
                                                       or (destination_path is not None and os.path.isdir(destination_path)))
        pairs = ([(source, os.path.join(destination, os.path.basename(source))) for source in sources] if into_directory
                 else [(sources[0], destination)])
    if name == "ln":   # 符号链接里的相对源从链接所在的目录算（lib_shell_words.py 的 link_source_word）
        pairs = [(shell_words.link_source_word(command.words[1:], source, destination), destination) for source, destination in pairs]
    return pairs

def file_on_disk(path, state):
    """盘上一份普通文件的内容：一次判定里同一份（按实际路径认）只读一遍，读过的记进 state.files_read。"""
    real_path = os.path.realpath(path)
    known = state.files.get(real_path)
    if known is not None:
        return known
    try:
        size = os.path.getsize(real_path)
        with open(real_path, "rb") as handle:
            data = handle.read(MAXIMUM_SCRIPT_BYTES + 1) if size <= MAXIMUM_SCRIPT_BYTES else None
            head = handle.read(SCRIPT_HEAD_BYTES) if data is None else data[:SCRIPT_HEAD_BYTES]
        if data is not None and len(data) > MAXIMUM_SCRIPT_BYTES:  # 量过大小之后才长大的
            size, data = len(data), None
        known = FileOnDisk(head, data, size, None)
    except OSError as error:
        known = FileOnDisk(b"", None, 0, error.strerror)
    state.files[real_path] = known
    state.files_read.append(real_path)
    return known

def text_on_disk(path, state):
    """追加之前盘上那份的正文：还不在交 ""，不是普通文件、超过读的上限、读不了、不是 UTF-8 文本交 None。"""
    if not os.path.lexists(path):
        return ""
    if not os.path.isfile(path):
        return None
    found = file_on_disk(path, state)
    if found.data is None:
        return None
    try:
        return found.data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def written_target_word(command, word):
    """heredoc 写出的目标词：带 `$` 的照这段命令文本里前面赋过的值代入（与 note_writer 记重定向目标同一套，lib_shell_words.py 的 expand_known_variables）；
    展开不了交原词（literal_path 照认不出交 None）。例：`t=<字面路径>; cat > $t <<EOF … EOF; bash $t` 这里与记写者那里同样代入，同一个目标两处一致。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=heredoc-targets-unexpanded 下原样交回。"""
    if "$" not in word or guard_break_is_set("heredoc-targets-unexpanded"):
        return word
    expanded = shell_words.expand_known_variables(word, command.variables or {})
    return word if expanded is None else expanded

def record_files_written(command, state):
    """这一条命令写出了哪些文件，记进 state.written（实际路径 → WrittenScript），后面的命令执行它们时拿这份内容判；每变一处 written_version 加一。"""
    if guard_break_is_set("ignore-written-scripts"):
        return
    written = state.written
    state.recognized_now = set()
    def remember(path, entry):
        written[path] = entry
        state.written_version += 1
        state.recognized_now.add(path)
    def forget(path):
        if written.pop(path, None) is not None:
            state.written_version += 1
    body = command.standard_input_heredocs[-1].text if command.standard_input_heredocs else None
    for target, appends in heredoc_write_targets(command):
        path = literal_path(command, written_target_word(command, target))
        if path is None:
            continue
        if not appends:
            remember(path, WrittenScript(body, None))
            continue
        entry = written.get(path)
        before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None
                  else text_on_disk(entry.copied_from, state))
        if before is None:
            forget(path)  # 追加到读不了的文件上：不认，照旧读盘上那份
        else:
            remember(path, WrittenScript(before + body, None))
    for source, destination in copy_pairs(command):
        source_path, destination_path = literal_path(command, source), literal_path(command, destination)
        if source_path is None or destination_path is None:
            continue
        if source_path in written:
            remember(destination_path, written[source_path])
        elif os.path.isfile(source_path):
            remember(destination_path, WrittenScript(None, source_path))
        else:
            forget(destination_path)  # 源不在：认不出，执行时照旧按盘上有没有判
    concatenation = cat_concatenation(command)
    if concatenation is not None:
        sources, destination = concatenation
        destination_path = literal_path(command, destination)
        if destination_path is not None:
            pieces = []
            if cat_appends(command):   # 追加：接在这条命令里前面写出的、或盘上那份后面（读不了的不认）
                entry = written.get(destination_path)
                before = (text_on_disk(destination_path, state) if entry is None else entry.text if entry.text is not None
                          else text_on_disk(entry.copied_from, state))
                pieces = None if before is None else [before]
            missing_source = False
            for source in sources:
                if pieces is None and not guard_break_is_set("cat-append-unreadable-destination-raises"):
                    break   # 追加的目的盘上读不了（目录、超过读的上限、不是 UTF-8 文本）：目的不认，不再往 pieces 里接源（接了的话，判的那一刻抛 AttributeError、整条命令照常执行）
                source_path = literal_path(command, source)
                entry = written.get(source_path) if source_path is not None else None
                text = None if source_path is None else (
                    (entry.text if entry.text is not None else text_on_disk(entry.copied_from, state)) if entry is not None else text_on_disk(source_path, state))
                if text is None:
                    pieces = None   # 有一个源算不出（带变量、读不了、不是文本）：认不出，执行时照旧按盘上有没有判
                    break
                if entry is None and source_path is not None and not os.path.lexists(source_path):
                    # 源盘上不在、也不是同一条命令里前面认出写出的：读不到它的内容，照认不出（偏严：cat 不看源在不在那一类照拒）
                    state.missing_cat_sources.append((source, destination))
                    if not guard_break_is_set("cat-missing-source-allowed"):
                        missing_source = True
                pieces.append(text)   # 不在的源 text_on_disk 交 ""：cat 报错、不写东西
            if pieces is None or missing_source:
                forget(destination_path)
            else:
                remember(destination_path, WrittenScript("".join(pieces), None))
    tee_copy(command, state, remember, forget)


def piped_text(command, state):
    """这一条的标准输入里是什么（tee 拷出用）：`< 源`（读盘上或这条命令里前面写出的）、管道前一段是只带文件参数的 cat（几个源接起来）或 printf / echo / 读 heredoc 的 cat 的字面；
    算不出交 None。"""
    if command.input_redirections:
        operator, target = command.input_redirections[-1]
        if operator not in ("<", "<>"):
            return None
        sources = [target]
    elif command.piped_from is not None:
        words, heredocs = command.piped_from
        skipped, words = shell_words.skip_prefixes_expanding(list(words))
        if skipped.command_position is None:
            return None
        name, arguments = os.path.basename(words[skipped.command_position]), words[skipped.command_position + 1:]
        if name == "cat" and arguments and not heredocs and not any(argument.startswith("-") for argument in arguments):
            sources = arguments
        else:
            literal = shell_words.piped_literal_output(list(command.piped_from[0]), heredocs)
            return literal.code if literal.unjudged is None else None
    else:
        return None
    pieces = []
    for source in sources:
        source_path = literal_path(command, source)
        if source_path is None:
            return None
        entry = state.written.get(source_path)
        if entry is None and not os.path.lexists(source_path) and not guard_break_is_set("cat-missing-source-allowed"):
            return None   # tee 的源盘上不在、也不是同一条命令里前面认出写出的：读不到它的内容，照认不出（偏严：cat 不看源在不在那一类照拒）
        text = (entry.text if entry.text is not None else text_on_disk(entry.copied_from, state)) if entry is not None else text_on_disk(source_path, state)
        if text is None:
            return None
        pieces.append(text)
    return "".join(pieces)


def tee_copy(command, state, remember, forget):
    """`… | tee [-a] 目的…`、`tee 目的 < 源`（heredoc 喂给 tee 的由 heredoc_write_targets 认）：目的里是 tee 读到的标准输入（piped_text），-a 接在原来的后面；
    算不出的照旧不认（执行时按盘上判；前面写过的那份认不出写法，unrecognized_write 照拒）。弄坏开关 tee-copy-unseen 下不认。"""
    if guard_break_is_set("tee-copy-unseen") or os.path.basename(command.words[0]) != "tee" or command.standard_input_heredocs:
        return
    arguments = command.words[1:]
    appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:]) for argument in arguments)
    targets = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
    targets = [target for target in targets if target != "-"]
    text = piped_text(command, state)
    for target in targets:
        path = literal_path(command, target)
        if path is None:
            continue
        if text is None:
            forget(path)
            continue
        if appends:
            entry = state.written.get(path)
            before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None else text_on_disk(entry.copied_from, state))
            if before is None:
                forget(path)
                continue
            remember(path, WrittenScript(before + text, None))
        else:
            remember(path, WrittenScript(text, None))

# 同一条命令里先写出、再执行的脚本，写法认不出的（echo / printf / dd / sed / head /
# install /dev/stdin / python 写出再起）：命令文本这一层里前面有一条命令点到了这份脚本（重定向的目标是它，或参数、heredoc 正文里写着它），又不是只读的命令、
# 也没被认成写出了确定内容（record_files_written 记下的那几种），执行它时照认不出拒（unrecognized_write）。弄坏开关 PROJECT_CODE_GUARD_BREAK=unrecognized-writes-allowed 下不判。
READ_ONLY_COMMANDS = frozenset({"ls", "stat", "test", "[", "[[", "file", "wc", "grep", "egrep", "fgrep", "rg", "head", "tail", "cat", "less", "more", "sha256sum",
                                "sha1sum", "md5sum", "cksum", "b2sum", "diff", "cmp", "realpath", "readlink", "basename", "dirname", "echo", "printf", "chmod", "chown",
                                "touch", "shellcheck", "du", "sort", "uniq", "cut", "true", "false", ":", "cd", "pushd", "popd", "export", "unset", "local", "declare", "typeset", "readonly",
                                "bash", "sh", "source", ".", "command", "type", "which", "hash", "mkdir", "rm", "rmdir", "nl", "od", "xxd", "hexdump", "strings", "sed",
                                "tr", "column", "jq", "find", "tree"})
# 经 PATH 找到的命令词（不带斜杠）是 shell 脚本的照读正文；这几个前缀下面的是系统包装的程序，不读
PATH_UNRESOLVED_SUFFIX = "，而这一条的 PATH 写成展开不了的值"
DIRECTORY_UNRESOLVED_SUFFIX = "，而这一条的当前目录算不出"
SYSTEM_PATH_PREFIXES = ("/usr/", "/bin/", "/sbin/", "/lib", "/opt/", "/snap/", "/etc/", "/nix/")


# python3 的代码（-c 的那个词、喂给 python3 - 的 heredoc 正文）里没有任何能写文件或起别的程序的东西，就是只读输入、往标准输出打字的过滤器：
# 代码里提到的字样（`d['task']` 里的 task）不算点到后面执行的脚本（`task-server/task`）。有这些的（open(、.write(、shutil、os.、subprocess、pathlib、exec( / eval( / compile(、
# dump(、.save( / .copy( / .move( / .link( / .touch( / .extract( 这一类）照旧整段算点到。命令词不是 python、跑的是脚本文件（python3 gen.py x.sh）的，参数照旧算点到。
# 弄坏开关 PROJECT_CODE_GUARD_BREAK=python-filter-code-judged 下不分，代码里有字样就算点到
PYTHON_COMMAND = re.compile(r"python[0-9.]*")
PYTHON_CODE_OPTION = re.compile(r"-[A-Za-z]*c")
PYTHON_WRITE_MARKER = re.compile(r"\bopen\s*\(|(?<!sys\.stdout)(?<!sys\.stderr)\.write\w*\s*\(|\bshutil\b|\bos\s*\.|\bsubprocess\b|\bpathlib\b|\bPath\s*\(|\bexec\s*\(|\beval\s*\("
                                 r"|(?<!re\.)\bcompile\s*\(|__import__|\bimportlib\b|\bctypes\b|\bdump\s*\(|\.(?:save\w*|copy\w*|move|rename|symlink|link|unlink|touch|mkdir\w*|extract\w*|truncate|chmod)\s*\("
                                 r"|urlretrieve|\bfileinput\b|\bio\s*\.|\bpopen\b|\bsystem\s*\(|\bgetattr\s*\(|\bsetattr\s*\(|\bglobals\s*\(")


def python_filter_code(command):
    """python3 -c 的代码、喂给 python3 - 的 heredoc 正文里没有任何能写文件或起别的程序的东西（PYTHON_WRITE_MARKER）的：→ (要从点到的文字里去掉的词下标集, 要不要去掉喂给它的 heredoc 正文)。
    不是这种的交 (空集, False)。"""
    words = list(command.words)
    if guard_break_is_set("python-filter-code-judged") or not words or command.launcher is not None or not PYTHON_COMMAND.fullmatch(os.path.basename(words[0])):
        return set(), False
    for index in range(1, len(words)):
        if PYTHON_CODE_OPTION.fullmatch(words[index]) and index + 1 < len(words):
            return ({index + 1} if PYTHON_WRITE_MARKER.search(words[index + 1]) is None else set()), False   # -c 之后的词是代码，再往后是代码的参数
        if not words[index].startswith("-"):
            return set(), False   # 跑的是脚本文件或 -m 的模块
    return set(), all(PYTHON_WRITE_MARKER.search(heredoc.text) is None for heredoc in command.standard_input_heredocs)   # 没有脚本也没有 -c：heredoc 是代码


def command_text_blob(command):
    """一条命令点到的文字：参数、重定向目标、喂给它的 heredoc 正文连起来（找它写没写某份脚本用）。python3 的代码没有任何写文件的东西的，代码不算（python_filter_code）。"""
    skipped, heredocs_are_filter_code = python_filter_code(command)
    pieces = [word for index, word in enumerate(command.words) if index >= 1 and index not in skipped] + [target for _, target in command.output_redirections]   # 命令词自己不算点到（再起一遍同一份脚本不是写它）
    if not heredocs_are_filter_code:
        pieces += [heredoc.text for heredoc in command.standard_input_heredocs]
    return "\n".join(pieces)


def mentions_path(blob, word, real_path):
    """blob 里写着这份脚本：照写的样子（带斜杠的）、实际路径，或文件名前后是路径边界（空白、引号、斜杠、等号、开头结尾；执行时写的不带斜杠、经 PATH 找到的不按文件名认）。"""
    if word and "/" in word and word in blob or real_path in blob:
        return True
    if word and "/" not in word and os.path.basename(real_path) == word and not word.endswith(SHELL_SCRIPT_EXTENSION):
        return False
    name = os.path.basename(real_path)
    return bool(name) and re.search(r"(?:^|[\s'\"/=(])" + re.escape(name) + r"(?=$|[\s'\");,])", blob) is not None


def note_writer(command, state, recognized, own_words=None):
    """命令文本这一层里这一条写了什么，记进 state.writers（recognized 是 record_files_written 认出写出了确定内容的实际路径集）。
    own_words 不是 None 时这一条是剥掉之后里面那条接着判的启动器（launcher_own_words）：点到的文字只取启动器自己的词与这一条的重定向、heredoc，
    里面那条剥开之后在里层自己记。"""
    state.serial += 1
    targets = []
    for _, target in command.output_redirections:
        expanded = shell_words.expand_known_variables(target, command.variables or {}) if "$" in target else target
        if expanded and not re.fullmatch(r"[0-9]+|-", expanded):
            path = literal_path(command, expanded)
            if path is not None:
                targets.append(path)
    # bash / source 起的脚本这一条是在执行它（命令词已换成脚本路径），照只读算；python 起的程序可能写文件，照写的算
    name = os.path.basename(command.words[0]) if command.words and command.launcher is None else {"shell": "bash", "source": "source", "startup": "source", "python": "python3"}.get(command.launcher, "")
    if command.launcher is None and command.words and copied_shell_binary(command, state) is not None:
        name = "bash"   # 同一条命令里拷来 / 链来的 shell 起脚本是在执行它（同 bash <脚本>）、照只读算，不是写它
    blob = command_text_blob(command if own_words is None else command._replace(words=[command.words[0], *own_words]))
    state.writers.append((state.serial, blob, frozenset(recognized), name, tuple(targets)))


def launcher_own_words(command, state):
    """这一条是剥掉之后里面那条接着判的启动器（/usr/bin/time、systemd-run、chrt、ssh 这一类，gpu-quiesce-run.sh、两种派发起法；
    scan_uses 里剥它的那几处同一套）时，交启动器自己的词（命令词之后、里面那条之前的选项与位置参数）；里面那条的词要是外层词的尾巴
    （选项带进来的 NAME=VALUE 先去掉），或像 ssh、sg 那样把尾巴几个词用空格连成 `sh -c` 的一段。不是启动器、或对不上尾巴的交 None，照整条记写者。
    例：`systemd-run --user --wait bash <轻脚本>` 不把 systemd-run 整条记成写过这份脚本的认不出写法，里面那条照常判。
    task-server/task submit（lib_project_code_runs.task_server_submission 认的那一种，python3 起的同算）同样交它自己的词（`submit`、`--cwd <目录>` 这几个选项、`--`）：
    客户端只把 `--` 之后那条交给服务、自己不写文件，那条在 scan_uses 里另判、在里层自己记写者。
    例：子 agent 交 `task-server/task submit -- bash <不按名字判的脚本>`（草稿里自己的辅助脚本；hook-swap 草稿树里经符号链接的崩溃点重放入口），
    不把 task 整条记成写过这份脚本的认不出写法，里面那条照常判。
    弄坏开关 PROJECT_CODE_GUARD_BREAK=launcher-recorded-as-writer 下一律交 None。"""
    if guard_break_is_set("launcher-recorded-as-writer") or not command.words:
        return None
    submitted = project_code_runs.task_server_submission(command.words, command.directory, command.launcher)
    if submitted is not None:
        submitted_words = list(submitted[0])
        cut = len(command.words) - len(submitted_words)
        return list(command.words[1:cut]) if cut >= 1 and list(command.words[cut:]) == submitted_words else None
    if command.launcher is not None:
        return None
    launched = (project_code_runs.command_under_launcher(command.words, command.directory)
                or project_code_runs.command_under_gpu_quiesce(command.words, command.directory)
                or project_code_runs.command_under_dispatch_launcher(command.words, command.directory))
    if launched is None:
        return None
    words, inner = list(command.words), list(launched[0])
    for skipped in range(len(inner)):
        if skipped and not project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(inner[skipped - 1]):
            break
        tail = inner[skipped:]
        cut = len(words) - len(tail)
        if cut >= 1 and words[cut:] == tail:
            return words[1:cut]
        if len(tail) == 3 and tail[:2] == ["sh", "-c"]:
            for cut in range(1, len(words)):
                if " ".join(words[cut:]) == tail[2]:
                    return words[1:cut]
    return None


def unrecognized_write(path, word, state):
    """执行的脚本 path 在命令文本这一层里前面被认不出写法的命令写过：交那条命令的命令名；没有交 None（后来一条认出了确定内容的写，盖掉前面的）。"""
    if guard_break_is_set("unrecognized-writes-allowed"):
        return None
    real_path = os.path.realpath(path)
    flagged = None
    for serial, blob, recognized, name, targets in state.writers:
        if serial == state.serial:
            continue
        if real_path in recognized:
            flagged = None
            continue
        if real_path in targets or (name not in READ_ONLY_COMMANDS and mentions_path(blob, word, real_path)):
            flagged = name or "前面一条命令"
    return flagged


FUNCTION_DEFINITION = re.compile(r"(?:^|[\s;&|(])(?:function\s+([A-Za-z_][\w-]*)|([A-Za-z_][\w-]*)\s*\(\s*\))")


def path_lookup_script(command):
    """命令词不带斜杠、直接执行的：照这一条的 PATH（命令前、env、export 设的，`$PATH` 展开成钩子自己的）在 PATH 上找，找到的不在系统目录（SYSTEM_PATH_PREFIXES）下就交它的路径；
    PATH 写成展开不了的值交 (None, 说明)；找不到、在系统目录下交 (None, None)。弄坏开关 PROJECT_CODE_GUARD_BREAK=path-scripts-unread 下一律交 (None, None)。"""
    word = command.words[0]
    if guard_break_is_set("path-scripts-unread") or word in shell_words.BASH_BUILTINS or word in shell_words.COMMAND_KEYWORDS:
        return None, None
    search = command.environment.get("PATH")
    if search is None:
        # 外面带进来的 PATH（钩子自己的）只找 .sh 结尾的（`s.sh` 这种写出来就是要经 PATH 起的脚本）；cargo、git 这类程序名不在钩子的 PATH 上找脚本读（本机用户 bin 目录下的包装照程序算）
        if not word.endswith(SHELL_SCRIPT_EXTENSION):
            return None, None
        search = os.environ.get("PATH", "")
    else:
        search = shell_words.expand_known_variables(search, dict(command.variables or {}, PATH=os.environ.get("PATH", "")))
        if search is None:
            return None, f"命令词 {word} 经 PATH 找{PATH_UNRESOLVED_SUFFIX}"
        search = ":".join(shell_words.resolve_path(command.directory, entry) or entry if entry and not entry.startswith("/") else entry
                          for entry in search.split(":"))
    found = shutil.which(word, path=search)
    if not found:
        return None, None
    found = os.path.abspath(found)
    if any(found.startswith(prefix) or os.path.realpath(found).startswith(prefix) for prefix in SYSTEM_PATH_PREFIXES):
        return None, None
    return found, None


def has_shell_script_extension(path):
    """执行的名字、或它指到的文件，扩展名是 .sh。"""
    return path.endswith(SHELL_SCRIPT_EXTENSION) or os.path.realpath(path).endswith(SHELL_SCRIPT_EXTENSION)

def script_path_of(command, written, strict=False):
    """命令位置上这一条执行的是不是一个要读进去的脚本文件：返回 (路径, None) 要读；(None, 找不到的说明) 记检出；(None, None) 不读也不记。
    written 是同一条命令里前面写出的文件（record_files_written），在里面的算找得到；直接执行的还不在，扩展名不是 .sh 的不读也不记。"""
    word = command.words[0]
    if command.launcher in ("shell", "source") and not guard_break_is_set("glob-scripts-unread") and set("*?[") & set(word) and not set("$`()'\"<>|&;{}=,\n") & set(word):
        # bash / source 起的脚本路径带通配（`bash 目录/*s.sh`）：照当前目录展开，bash 起的是排在头一个的那一份；对不上文件的 bash 报错不跑。弄坏开关 glob-scripts-unread 下不读
        pattern = shell_words.resolve_path(command.directory, os.path.expanduser(word))
        matches = sorted(glob.glob(pattern)) if pattern is not None else []
        return (matches[0], None) if matches else (None, None)
    if command.launcher == "python" or word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None, None
    expanded = shell_words.expand_directory_tilde(word, command.directory)
    if command.launcher is None and expanded is not None and "/" not in expanded:
        return path_lookup_script(command) if strict else (None, None)
    resolved = shell_words.resolve_path(command.directory, expanded) if expanded is not None else None
    undeterminable_directory = (strict and not guard_break_is_set("relative-script-undeterminable-directory-allowed")
                                and command.launcher in ("shell", "source", None))
    if resolved is None:
        # 当前目录算不出（cd 到的目录是算不出的变量 / 命令替换、~- 的 OLDPWD）而起的是相对路径脚本：认不出起的是哪一份（
        # 弄坏开关 PROJECT_CODE_GUARD_BREAK=relative-script-undeterminable-directory-allowed 下照旧放行）
        if undeterminable_directory:
            return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {word} 的当前目录算不出{DIRECTORY_UNRESOLVED_SUFFIX}"
        return None, None
    if (undeterminable_directory and not os.path.isabs(expanded) and command.directory is not None and not os.path.isdir(command.directory)
            and not command.cd_failure_skips):
        # cd 到的目录是字面路径、盘上不在：cd 会失败、留在原来的目录，相对路径脚本认不出在哪个目录。
        # cd 失败了这一条就不跑的（`cd 目录 && 这一条`、`cd 目录 || exit` 之后；lib_shell_words.py 的 cd_failure_skips）不在此列：照 cd 成功了的那个目录往下判
        # （同一条命令里前面写出的照读，不在的照「脚本不存在」只记；弄坏开关 LIB_SHELL_WORDS_BREAK=cd-failure-guards-unseen 下照旧拒）
        return None, (f"{LAUNCHER_WORDS[command.launcher]}脚本 {word} 要先 cd 到 {command.directory}，那个目录此刻盘上不在（同一条命令里前面 mkdir 建的也照不在算），"
                      "cd 会失败而这一条照跑（cd 后面接的是 `;`、换行或括号；写成 `cd 目录 && …` 或 `cd 目录 || exit` 才照 cd 成功了的目录判）"
                      f"{DIRECTORY_UNRESOLVED_SUFFIX}")
    if os.path.realpath(resolved) in written or os.path.lexists(resolved):
        return resolved, None
    if command.launcher == "startup":
        return None, None   # BASH_ENV / ENV 指的文件不在：shell 照样起、什么都不读（不报错、不按 PATH 找），没有可判的，不记检出
    if command.launcher is not None and "/" not in expanded:
        found = shutil.which(expanded, mode=os.F_OK)
        if found:
            return os.path.abspath(found), None
    if command.launcher is None and not has_shell_script_extension(resolved):
        return None, None
    return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {resolved} 不存在"

def shebang_interpreter(data):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。解析在共用模块（shell_words.shebang_command，直接执行的脚本读不读启动文件也用它）。"""
    shebang = shell_words.shebang_command(data)
    return None if shebang is None else shebang[0]

def executed_file_head(path, state):
    """直接执行的文件开头 SCRIPT_HEAD_BYTES 字节（读它的 #! 行用）：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；不是普通文件、读不了交 None。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is not None and entry.copied_from is None:
        return (entry.text or "").encode("utf-8")[:SCRIPT_HEAD_BYTES]
    source = entry.copied_from if entry is not None else path
    if not os.path.isfile(source):
        return None
    found = file_on_disk(source, state)
    return None if found.error is not None else found.head

def executed_file_is_shell_script(executed_path, head):
    """直接执行的文件当不当 shell 脚本读，只看开头：编译产物（ELF 魔数、NUL 字节）不算；有 `#!` 的看它指不指到 shell；
    没有 `#!` 的看扩展名是不是 .sh（`.rs`、`.py`、`.md`、没扩展名的文本都不算）。"""
    if head.startswith(b"\x7fELF") or b"\0" in head:
        return False
    interpreter = shebang_interpreter(head)
    if interpreter is not None:
        return interpreter in shell_words.SHELL_NAMES
    return has_shell_script_extension(executed_path)

def decoded_script_text(data, path, launcher, executed_path):
    """已经读到手的脚本字节（盘上读的、或同一条命令里 heredoc 写出的）：返回值同 script_text。"""
    how = LAUNCHER_WORDS[launcher]
    if launcher is None and not executed_file_is_shell_script(executed_path, data[:SCRIPT_HEAD_BYTES]):
        return None, None
    if len(data) > MAXIMUM_SCRIPT_BYTES:
        return None, f"{how}脚本 {path} 有 {len(data)} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    if b"\0" in data:
        return None, f"{how}脚本 {path} 不是文本（有 NUL 字节）"
    try:
        return data.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, f"{how}脚本 {path} 不是文本（不是 UTF-8）"

def script_text(path, launcher, state, executed_path):
    """盘上 path 那份当脚本读（executed_path 是执行的名字，直接执行的按它的扩展名判）：返回 (正文, None) 要读；(None, 读不到的原因) 记检出；
    (None, None) 不是普通文件（目录、设备）或直接执行的不是 shell 脚本，不读也不记。"""
    how = LAUNCHER_WORDS[launcher]
    if not os.path.isfile(path):
        return None, None
    found = file_on_disk(path, state)
    if found.error is not None:
        return None, f"{how}脚本 {path} 读不了（{found.error}）"
    if launcher is None and not executed_file_is_shell_script(executed_path, found.head):
        return None, None
    if found.data is None:
        return None, f"{how}脚本 {path} 有 {found.size} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    return decoded_script_text(found.data, path, launcher, executed_path)

def script_text_to_judge(path, launcher, state):
    """执行时这个脚本里是什么：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；返回值同 script_text。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is None:
        return script_text(path, launcher, state, path)
    if entry.copied_from is not None:
        return script_text(entry.copied_from, launcher, state, path)
    return decoded_script_text(entry.text.encode("utf-8"), path, launcher, path)

def lower_cycle_frame(first, second):
    """两个「绕回外层的那一格」取靠外的那个；None 是没绕回。"""
    if first is None:
        return second
    return first if second is None else min(first, second)

def copied_shell_binary(command, state):
    """同一条命令里前面 cp / ln / install 拷来、链来的可执行文件（state.written 记下 copied_from），源解到已知 shell 二进制名（bash / sh / dash / zsh / ksh / rbash / ash）时交那个名；
    不是这种的（裸命令词、没拷来、源不是 shell）交 None（源算不出的那几种写法由 unrecognized_write 拦）。"""
    word = command.words[0]
    if word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None
    path = literal_path(command, word)
    if path is None:
        return None
    entry = state.written.get(path)
    if entry is None or entry.copied_from is None:
        return None
    name = os.path.basename(os.path.realpath(entry.copied_from))
    return name if shell_words.is_shell_name(name) else None


def program_file_text(program_file, state, variables=None):
    """别的程序从文件读的代码（awk -f、sed -f、Makefile；lib_shell_words.py 的 ProgramFile）：交 (正文, 读的路径, 说明, 说明是不是「认不出」)。
    候选路径按次序取头一份在的（同一条命令里前面写出的也算）；都不在交 (None, None, None, False)（程序自己报错、什么都不跑）；路径写成变量、命令替换，或当前目录算不出的，
    交认不出的说明（调用方在命令文本这一层照认不出拒）；读不了、太大、不是文本的同脚本，交只记的说明。
    variables 是这段命令文本里前面赋过的 shell 变量（CommandAtPosition.variables）：路径里的 `$名字`、`${名字}` 照它代入再判（d137-replay-classify 误拒 E5：
    `W=<字面目录>; awk -f $W/p.awk f.rs`）；代入不了的、带命令替换或反引号的照认不出。弄坏开关 PROJECT_CODE_GUARD_BREAK=program-file-variables-unexpanded 下不代入。"""
    for written in program_file.words:
        word = written
        if "$" in written and "`" not in written and "(" not in written and not guard_break_is_set("program-file-variables-unexpanded"):
            word = shell_words.expand_known_variables(written, variables or {}) or written
        if "$" in word or "`" in word or "(" in word:
            return None, None, f"{program_file.language} 读的文件 {written} 写成变量或命令替换，认不出读的是哪一份", True
        expanded = shell_words.expand_directory_tilde(word, program_file.directory)
        resolved = shell_words.resolve_path(program_file.directory, os.path.expanduser(expanded)) if expanded is not None else None
        if resolved is None:
            return None, None, f"{program_file.language} 读的文件 {word} 的当前目录算不出，认不出读的是哪一份", True
        if os.path.realpath(resolved) in state.written or os.path.isfile(resolved):
            body, problem = script_text_to_judge(resolved, "program", state)
            return body, resolved, problem, False
    return None, None, None, False


def scan_uses(text, directory, environment=None, frames_above=(), script=None, state=None, memory_capped=False, definitions=None, strict=None, positional=(),
               directory_guarded=True, interactive=False):
    """text 里命令位置上的每一处外部程序用法与不经 task-server 的直接起；执行的是脚本文件就读进去，用同一套切词与判定递归判。
    definitions 是外层命令文本里的变量定义（剥掉 /usr/bin/time 这一类之后里面那条接着判时带进来）；命令词没展开的，连同这一段自己的定义交给
    lib_project_code_runs.unexpanded_command_word_use 判（点到 验证包测试目标名的照直接执行它的测试二进制算）。读进去的脚本正文只用它自己的定义。
    script 是 text 所在的脚本（命令文本本身是 None），frames_above 是读到它为止经过的外层脚本与起它的那一行；
    state 是这一次判定里外各层共用的 JudgingState：脚本里写出的，外层后面的命令也看得见；同一份脚本只读一遍，判过的按键复用。
    memory_capped：text 整个跑在 run-with-memory-cap.sh 里（起它所在脚本的那一条经它包着）：只交给切词与复用的键，判不判都不看它（经没经内存包装不再判，文件头「经 task-server 起」）。
    跑项目代码的每一处（编译、编出来的二进制、本仓脚本、剥掉的内存包装）与子 agent 不起的客户端子命令记进 direct_runs，task-server/task submit 交给服务的那条
    （`--` 之后）另判一遍、只留它的检出与绕开服务的写法，绕开 task-server 的写法记进 bypasses，名单上直接跑的脚本记进 listed_scripts（文件头「经 task-server 起」）。
    strict：text 是命令文本这一层（Bash 调用的命令、它 bash -c / eval / 替换 / 剥掉启动器之后的那几段，不是读进来的脚本正文；不给时按 script is None 定）：
    认不出起什么的写法（lib_project_code_runs.unrecognized_launch）与认不出写法写出再执行的脚本（unrecognized_write）只在这一层判；脚本正文里只判命令词或脚本路径解不出、
    而定义或起它时给的参数（positional：起这份脚本的那一条的参数）里写着自证、门禁阶段、入口脚本或 cargo 的（lib_project_code_runs.script_body_unresolved_launch，文件头「认不出的写法」）。
    directory_guarded：起这段文本的那一条的 cd_failure_skips（lib_shell_words.py：前面可能失败的 cd 失败了它就不跑）；interactive：跑这段文本的 shell 可能是交互的。
    两样都交给切词，读进去的正文照起它的那一条的往下传。"""
    if state is None:
        state = JudgingState()
    if strict is None:
        strict = script is None
    launchers = frozenset() if guard_break_is_set("launcher-functions-unjudged") else shell_words.launcher_functions(text)
    shell_functions = {} if guard_break_is_set("bash-env-forwarding-untracked") else shell_words.shell_starting_functions(text)
    defined_functions = {name for pair in FUNCTION_DEFINITION.findall(text) for name in pair if name} if strict else set()
    own_definitions = project_code_runs.variable_definitions(text)
    for name, values in (definitions or {}).items():
        own_definitions.setdefault(name, []).extend(values)
    uses, notices, scripts_read, occurrences, direct_runs = [], [], [], {}, []
    bypasses, listed_scripts = [], []
    hit_depth_limit, lowest_cycle_frame = False, None
    line_index = None
    def frames_of(key, occurrence):
        nonlocal line_index
        if script is None:
            return frames_above
        if line_index is None:
            line_index = command_lines(text)
        lines = line_index.get(key, [])
        return frames_above + ((script, lines[occurrence] if occurrence < len(lines) else None),)
    # 命令文本本身（不是读进来的脚本正文：那份可能被 source 进别处、函数在外面调）里只定义、调不到的 function 的函数体不判
    scan = shell_words.commands_at_command_position(text, directory, environment, memory_capped=memory_capped, uncalled_functions_skipped=script is None,
                                                    directory_guarded=directory_guarded, interactive=interactive)
    # 喂给从标准输入读命令的 shell、这里判不了的输入（here-string 或管道前一段里有变量、命令替换，管道前一段是 cat 文件这类；lib_shell_words.py 的 shell_standard_input）：
    # 照常执行、记检出（字面的那一部分已经当命令切过、照判）
    notices += [Notice(frames_above if script is None else frames_above + ((script, None),), f"{entry}，这一处没判全")
                for entry in scan.unjudged_shell_inputs]
    # 命令文本这一层里喂给 shell 的不是字面的（管道前一段是 grep 这类别的命令、here-string 里有命令替换……）：认不出喂进去的是什么，照认不出拒（照样记检出）；
    # 弄坏开关 PROJECT_CODE_GUARD_BREAK=unjudged-shell-input-allowed 下只记不拒
    if strict and not guard_break_is_set("unjudged-shell-input-allowed"):
        uses += [UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{entry}，认不出喂进 shell 跑的是什么"),
                          frames_above) for entry in scan.unjudged_shell_inputs]
    def merge(inner):
        nonlocal hit_depth_limit, lowest_cycle_frame
        uses.extend(inner.uses)
        notices.extend(inner.notices)
        direct_runs.extend(inner.direct_runs)
        bypasses.extend(inner.bypasses)
        listed_scripts.extend(inner.listed_scripts)
        scripts_read.extend(inner.scripts_read)
        hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
        lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
    # exec < 文件 / { … } < 文件 / ( … ) < 文件 / while … done < 文件：把 < 的文件下发给里层读标准输入的 bash / sh，照 `bash 文件` 读那份文件判
    # （弄坏开关 PROJECT_CODE_GUARD_BREAK=compound-stdin-untracked 下不认）
    if strict and not guard_break_is_set("compound-stdin-untracked"):
        feeds = shell_words.compound_redirected_stdin_files(text)
        for feed_file in feeds.files:
            merge(scan_uses(shlex.join(["bash", feed_file]), directory, environment, frames_above, None, state, memory_capped, own_definitions, strict=strict,
                             directory_guarded=directory_guarded))
        if feeds.undeterminable:
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                 "exec < / 复合命令上的输入重定向把标准输入下发给里层 shell，来源写成变量或命令替换、算不出起的是哪一份"),
                                 frames_above))
        # 从别的描述符、here-string、heredoc 下发，或管道接进 ( … ) / if / case / 循环：认不出里层读标准输入的 shell 读到什么，照认不出拒
        # （lib_shell_words.py 的 compound_redirected_stdin_files；弄坏开关 LIB_SHELL_WORDS_BREAK=compound-stdin-closers-unseen 下不交这几句）
        uses += [UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{entry}，认不出里层的 shell 跑的是什么"), frames_above)
                 for entry in feeds.unrecognized]
    pending = list(scan.commands)   # 直接执行的脚本先读的启动文件在循环里接在后面（executed_script_startup）
    for command in pending:
        key = tuple(command.words)
        occurrence = occurrences.get(key, 0)
        occurrences[key] = occurrence + 1
        state.recognized_now = set()
        record_files_written(command, state)
        if strict:
            # 剥掉之后里面那条接着判的启动器（systemd-run、chrt、ssh、/usr/bin/time 这一类）只记它自己的词：里面那条起的脚本不当它写过（d137-replay-classify 误拒 E2）
            note_writer(command, state, state.recognized_now, launcher_own_words(command, state))
        # 剥掉的 capped.sh / run-with-memory-cap.sh 盘上有、又不是本仓那一份的（路径尾部相同的拷贝、经符号链接起的）：照读它的正文，这一条不算经内存包装
        # （第一百一十八节「门禁阶段与包装脚本同一口径」；弄坏开关 wrapper-copies-trusted 下照旧只按名字剥）
        if not guard_break_is_set("wrapper-copies-trusted"):
            for wrapper_word, wrapper_directory in command.wrappers:
                if project_code_runs.existing_copy_elsewhere(wrapper_word, wrapper_directory):
                    if os.path.basename(wrapper_word) == shell_words.MEMORY_CAP_WRAPPER:
                        command = command._replace(memory_capped=False)
                    merge(scan_uses(shlex.join(["bash", wrapper_word]), wrapper_directory, command.environment, frames_of(key, occurrence), None, state,
                                     False, None, strict=False, directory_guarded=command.cd_failure_skips))
        # 命令词写在变量里的先用这段命令文本里前面赋过的值展开再判（lib_project_code_runs.py 的 command_word_expanded）；
        # cargo 的参数写成前面赋过值的变量的同样展开（lib_project_code_runs.py 的 cargo_arguments_expanded）
        command = command._replace(words=project_code_runs.cargo_arguments_expanded(project_code_runs.command_word_expanded(command.words, command.variables), command.variables))
        # 文件头「经 task-server 起」③：绕开服务的写法（设准入开关、直接起服务本体与每次尝试的那一层、systemctl 停改它的单元），经不经 submit 都拒
        # （task_server_lib.py 载不进时这里什么都认不出，task_server_decision 照「判不了」记一条检出，不在每条命令上各记）
        if not guard_break_is_set("task-server-bypass-allowed"):
            bypass_details, _bypass_problem = project_code_runs.task_server_command_bypass(command.words, command.environment, command.directory, command.launcher)
            bypasses += [Bypass(detail, frames_of(key, occurrence)) for detail in bypass_details]
        # 文件头「经 task-server 起」②：子 agent 起提交客户端只许 status、log（lib_project_code_runs.subagent_task_server_refusal），别的记下由 task_server_decision 按谁在跑判
        client_refusal = project_code_runs.subagent_task_server_refusal(command.words, command.directory, command.launcher)
        if client_refusal is not None:
            direct_runs.append(DirectRun(client_refusal, frames_of(key, occurrence), SUBAGENT_CLIENT_LEVEL))
        # 文件头「经 task-server 起」①：task-server/task submit -- <命令>：交给服务的那条不经内存包装，这里只另判一遍、留下它的检出与绕开服务的写法（③）
        submitted = project_code_runs.task_server_submission(command.words, command.directory, command.launcher)
        if submitted is not None:
            submitted_words, submitted_directory = submitted
            inner = scan_uses(shlex.join(submitted_words), submitted_directory or command.directory, command.environment, frames_of(key, occurrence), None, state,
                               False, own_definitions, strict=strict, directory_guarded=command.cd_failure_skips)
            notices.extend(inner.notices)
            bypasses.extend(inner.bypasses)
            scripts_read.extend(inner.scripts_read)
            if not guard_break_is_set("temporary-service-submit-inner-unread"):
                direct_runs.extend(run for run in inner.direct_runs if run.allowed in (TEMPORARY_SERVICE_BARE_LEVEL, TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL, CLIPPY_LINT_TABLE_LEVEL))   # 文件头「经 task-server 起」④⑤⑥：交给服务的那条里裸写的要临时服务的自证、没套包装的样本档、没带全 lint 表的 clippy 也拒
            continue
        # 本仓的代码（lib_project_code_runs.project_code_use）：名单上谁都能直接跑的不记，只给主 agent 的记下由 task_server_decision 按谁在跑判；
        # 剥掉的 capped.sh / run-with-memory-cap.sh 也是本仓的代码（经它们直接起照拒，交给服务的命令不带它们）
        project = project_code_runs.project_code_use(command.words, command.directory, command.launcher)
        if project is not None and project.allowed != "nobody":
            listed_scripts.append((project.real_path, project.allowed))
        if project is not None and project.allowed != "anyone":
            direct_runs.append(DirectRun(f"本仓的 {project.detail}", frames_of(key, occurrence), project.allowed))
        for wrapper_word, _wrapper_directory in command.wrappers:
            direct_runs.append(DirectRun(f"经 {os.path.basename(wrapper_word)} 直接起（交给服务的命令不带包装）", frames_of(key, occurrence)))
        # 认不出起什么的写法（命令文本这一层；lib_project_code_runs.py 的 unrecognized_launch）：照「认不出起什么的写法」那一类拒
        unrecognized = project_code_runs.unrecognized_launch(command.words, command.launcher) if strict else None
        # 脚本正文里命令词或脚本路径由变量拼出、解不出，而定义或位置参数里写着自证、门禁阶段、入口脚本或 cargo 的（例：经脚本里的变量起改过的钩子自证）
        if not strict:
            unrecognized = project_code_runs.script_body_unresolved_launch(command.words, command.launcher, own_definitions, positional,
                                                                   script if script is not None else (frames_above[-1][0] if frames_above else None))
        if unrecognized:
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, unrecognized),
                                 frames_of(key, occurrence)))
        # 文件头「经 task-server 起」④：要临时 prune-server 的自证写成裸的单独一个 --selftest：裸跑退 2、什么都不判，执行前拒（包在临时服务的包装里的，剥开重判时环境里带着标记，不记）
        if not guard_break_is_set("temporary-service-bare-allowed"):
            bare_selftest = project_code_runs.temporary_service_bare_selftest(command.words, command.directory, command.environment)
            if bare_selftest is not None:
                direct_runs.append(DirectRun(bare_selftest, frames_of(key, occurrence), TEMPORARY_SERVICE_BARE_LEVEL))
        # 文件头「经 task-server 起」⑥：门禁样本档（stage-selftest.sh --item 点名）点名的阶段源码里调了临时服务核对、外面没套包装：执行前拒（判法与 ④ 同一套标记）
        if not guard_break_is_set("temporary-service-stage-samples-allowed"):
            bare_stage_samples = project_code_runs.temporary_service_bare_stage_samples(command.words, command.directory, command.environment)
            if bare_stage_samples is not None:
                direct_runs.append(DirectRun(bare_stage_samples, frames_of(key, occurrence), TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL))
        # 文件头「经 task-server 起」⑤：命令文本这一层（读进来的脚本正文不判）里 cargo clippy 的 `--` 之后没带 -D warnings 与 check.sh 的 CODE_DISCIPLINE_LINTS 那张表的每一条
        if strict and not guard_break_is_set("clippy-lint-table-unchecked"):
            missing_lints = clippy_lint_table_missing(command.words, command.directory)
            if missing_lints is not None:
                direct_runs.append(DirectRun(missing_lints, frames_of(key, occurrence), CLIPPY_LINT_TABLE_LEVEL))
        # 命令词是这段文本里定义的、把参数当命令跑的函数（shell_words.launcher_functions：`run() { "$@"; }`、run_logged）：参数从每一处起的一段各当一条命令判
        if command.launcher is None and os.path.basename(command.words[0]) in launchers:
            for start in range(1, len(command.words)):
                merge(scan_uses(shlex.join(command.words[start:]), command.directory, command.environment, frames_of(key, occurrence), None, state,
                                 command.memory_capped, own_definitions, strict=strict, directory_guarded=command.cd_failure_skips))
        # 命令词是这段文本里定义的、函数体起 shell 的函数，且这一条前缀设了 BASH_ENV / ENV（函数体就地判时看不到调用处的 BASH_ENV）：
        # 拿调用处的 environment 重判函数体，补上里层 shell 读的启动文件（弄坏开关 PROJECT_CODE_GUARD_BREAK=bash-env-forwarding-untracked 下不认）
        if (command.launcher is None and os.path.basename(command.words[0]) in shell_functions
                and (shell_words.STARTUP_FILE_BASH_VARIABLE in command.environment or shell_words.STARTUP_FILE_POSIX_VARIABLE in command.environment)):
            merge(scan_uses(shell_functions[os.path.basename(command.words[0])], command.directory, command.environment,
                             frames_of(key, occurrence), None, state, command.memory_capped, own_definitions, strict=strict,
                             directory_guarded=command.cd_failure_skips))
        # 别的程序里起的命令（python / perl / awk / sed / tclsh / make / git / tar / ssh 这类；lib_shell_words.py 的 embedded_launches）：
        # 字面写着的照一段命令文本接着判（同 bash -c）；程序从文件读的代码（awk -f、sed -f、Makefile）读正文判（program_text_launches，照脚本正文那样只判解得出的）；
        # 写法认不出的，命令文本这一层照认不出拒，读进来的正文里照常执行、记检出（弄坏开关 PROJECT_CODE_GUARD_BREAK=program-files-unread 下不读程序文件，
        # embedded-unjudged-allowed 下认不出的只记不拒）
        embedded = shell_words.embedded_launches(command)
        embedded_frames = frames_of(key, occurrence)
        embedded_unjudged = list(embedded.unjudged)
        for embedded_text in embedded.texts:
            merge(scan_uses(embedded_text, command.directory, command.environment, embedded_frames, None, state, command.memory_capped, own_definitions,
                             strict=strict, directory_guarded=command.cd_failure_skips))
        for program_file in () if guard_break_is_set("program-files-unread") else embedded.program_files:
            body, location, problem, unrecognized_problem = program_file_text(program_file, state, command.variables)
            if problem and unrecognized_problem:
                embedded_unjudged.append(problem)
            elif problem:
                notices.append(Notice(embedded_frames, f"{problem}，里面起的命令没判"))
            if body is None:
                continue
            scripts_read.append(location)
            file_texts, file_unjudged = shell_words.program_text_launches(program_file.language, body, program_file.variables)
            for file_text in file_texts:
                merge(scan_uses(file_text, program_file.directory, command.environment, embedded_frames + ((location, None),), None, state, command.memory_capped,
                                 None, strict=False, directory_guarded=command.cd_failure_skips))
            notices += [Notice(embedded_frames + ((location, None),), f"{entry}，这一处没判全") for entry in file_unjudged]
        for entry in embedded_unjudged:
            if strict and not guard_break_is_set("embedded-unjudged-allowed"):
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{entry}（{os.path.basename(command.words[0])} 里起的命令）"),
                                     embedded_frames))
            else:
                notices.append(Notice(embedded_frames, f"{entry}，这一处没判全"))
        # /usr/bin/time、flock、systemd-run 这一类包在外面照样起那条命令的：剥掉它，里面那条按一段命令文本接着判（包装、脚本、内存包装照认）；
        inner_capped = command.memory_capped
        launched = project_code_runs.command_under_launcher(command.words, command.directory)
        if launched is None:
            launched = project_code_runs.command_under_gpu_quiesce(command.words, command.directory)
        # 派发起法 multi-host-run.sh --peer … --（lib_project_code_runs.py 的 command_under_dispatch_launcher）：里面那条经它的内存包装起，
        if launched is None:
            launched = project_code_runs.command_under_dispatch_launcher(command.words, command.directory)
            if launched is not None:
                inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            # 临时 prune-server 的包装（给了 --server-binary 的才剥得开）给里面那条原命令设了临时服务的标记：重判里面那条时补上，免得把包在包装里的要临时服务的自证当成裸写（文件头「经 task-server 起」④）
            inner_environment = command.environment
            if not guard_break_is_set("temporary-service-wrapper-not-marked") and project_code_runs.command_under_temporary_service(command.words, command.directory) is not None:
                inner_environment = project_code_runs.environment_under_temporary_service(command.environment)
            inner = scan_uses(shlex.join(inner_words), inner_start, inner_environment, frames_of(key, occurrence), None, state, inner_capped,
                               own_definitions, strict=strict, directory_guarded=command.cd_failure_skips)
            merge(inner)
            continue
        # 同一条命令里 cp / ln / install 拷来、链来的、或盘上符号链接指到的可执行文件，解到已知 shell 二进制（bash / sh / dash / zsh / ksh / rbash / ash）：
        # 按那种 shell 重判它的参数（-c 的代码、起的脚本、启动文件）。弄坏开关 PROJECT_CODE_GUARD_BREAK=copied-shell-binary-untracked 下不认
        if command.launcher is None and not guard_break_is_set("copied-shell-binary-untracked"):
            shell_binary = copied_shell_binary(command, state)
            if shell_binary is not None:
                merge(scan_uses(shlex.join([shell_binary] + list(command.words[1:])), command.directory, command.environment,
                                 frames_of(key, occurrence), None, state, command.memory_capped, own_definitions, strict=strict,
                                 directory_guarded=command.cd_failure_skips))
                continue
        # 跑编译出来的代码、起编译的（文件头「经 task-server 起」①：经没经内存包装都一样，不经 task-server 的拒）
        compiled = project_code_runs.runs_compiled_code(command.words, command.directory)
        building = project_code_runs.starts_a_build(command.words, command.directory)
        if compiled or building:
            direct_runs.append(DirectRun(compiled or building, frames_of(key, occurrence)))
        # 本仓之外要经服务起的外部程序、判不出的通配（lib_project_code_runs.classify）：记一条 use，由 scan_decision 拒；已经拒了的不再读它的脚本正文
        test = project_code_runs.classify(command.words, command.directory, command.environment)
        if test:
            uses.append(UseAt(test, frames_of(key, occurrence)))
            continue
        # 按名字认的仓内入口脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的）起它已经记成 ① 的直接起；里面起什么由它们自己的自证与门禁管，正文不读
        if project_code_runs.judged_by_name(command.words[0], command.directory):
            continue
        if strict and (command.launcher in ("shell", "source", "startup") or (command.launcher is None and ("/" in command.words[0] or command.words[0].endswith(SHELL_SCRIPT_EXTENSION)))):
            # 同一条命令里前面认不出写法写过的脚本（还不在盘上的也算）：执行前算不出它里面是什么，照认不出拒（unrecognized_write）
            candidate = literal_path(command, command.words[0])
            written_by = unrecognized_write(candidate, command.words[0], state) if candidate is not None else None
            if written_by:
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                                            f"{LAUNCHER_WORDS[command.launcher if command.launcher in LAUNCHER_WORDS else None]}脚本 {candidate} 是同一条命令里前面的 {written_by} "
                                                            + ("拷出或链出的，而源此刻盘上不在、或源与目的的写法认不出（ln -s 的相对源从链接所在的目录算），执行前算不出它里面是什么"
                                                               if written_by in COPYING_COMMANDS else
                                                               "写的，写法认不出（不是 cat / tee 读 heredoc、cp / ln / install / mv、cat 源 > 目的、tee 拷出这几种），执行前算不出它里面是什么")),
                                     frames_of(key, occurrence)))
                continue
        path, missing = script_path_of(command, state.written, strict and os.path.basename(command.words[0]) not in defined_functions)
        if path is None and missing is None:
            continue
        here = frames_of(key, occurrence)
        if missing and missing.endswith((PATH_UNRESOLVED_SUFFIX, DIRECTORY_UNRESOLVED_SUFFIX)):
            uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND, f"{missing}，认不出起的是哪一份"),
                                 here))
            continue
        if missing:
            notices.append(Notice(here, f"{missing}，里面的命令没判"))
            continue
        real_path = os.path.realpath(path)
        enclosing = [os.path.realpath(frame_path) for frame_path, _ in here]
        if real_path in enclosing:
            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, enclosing.index(real_path))
            continue
        if len(here) + 1 > MAXIMUM_SCRIPT_DEPTH:
            hit_depth_limit = True
            notices.append(Notice(here, f"{LAUNCHER_WORDS[command.launcher]}脚本 {path} 是第 {len(here) + 1} 层，"
                                        f"超过读的上限 {MAXIMUM_SCRIPT_DEPTH} 层，看不全，里面的命令没判"))
            continue
        if command.launcher is None:
            # 直接执行的脚本：它的 shell 起来时先读 BASH_ENV / ENV / .bashrc / zsh 的启动文件（lib_shell_words.py 的 executed_script_startup），
            # 读到的启动文件接在后面当被起的脚本判；#! 指到的解释器认不出、而环境里设了 BASH_ENV 或 ENV 的，照认不出拒
            startups, unknown_interpreter = shell_words.executed_script_startup(command, executed_file_head(path, state))
            pending.extend(startups)
            if unknown_interpreter is not None:
                uses.append(UseAt(project_code_runs.program_use(project_code_runs.UNRECOGNIZED_KIND,
                                                            f"脚本 {path} 的 #! 行指到 `{unknown_interpreter}`，认不出它起来时读不读 BASH_ENV / ENV 指的启动文件，而这条命令设了它们"),
                                     here))
        body, problem = script_text_to_judge(path, command.launcher, state)
        if problem:
            notices.append(Notice(here, f"{problem}，里面的命令没判"))
            continue
        if body is None:
            continue
        # 入口按自己所在的目录 source 的库换成字面路径，照读进来判（lib_project_code_runs.py 的 script_directory_expanded）
        body = project_code_runs.script_directory_expanded(body, path)
        # 跑这份正文的 shell 交不交互：source 与启动文件是起它的那个 shell，bash / sh 起的是新起的那个 shell（切词记在 command.interactive），直接执行的看 #! 行的选项里有没有 i。
        # 非交互的 shell source 的文件读到 `case $- in *i*) ;; *) return;; esac` 这类守卫就 return 了，后面的不读（lib_shell_words.py 的 non_interactive_return_offset；
        # ~/.bashrc 开头的守卫之后才 source bash-completion 这类大文件，非交互的登录 shell 读登录启动文件时照读全份，每次多两三秒、多记一条检出）
        if command.launcher in ("source", "startup", "shell"):
            body_interactive = command.interactive
        else:
            shebang = shell_words.shebang_command(executed_file_head(path, state) or b"")
            body_interactive = shebang is not None and any(option.startswith("-") and not option.startswith("--") and "i" in option[1:] for option in shebang[1])
        return_offset = shell_words.non_interactive_return_offset(body) if command.launcher in ("source", "startup") and not body_interactive else None
        if return_offset is not None:
            body = body[:return_offset]
        judged_key = (real_path, command.directory, state.written_version, command.memory_capped, tuple(command.words[1:]),
                      command.cd_failure_skips, body_interactive, return_offset)
        earlier = state.judged.get(judged_key)
        if earlier is not None:
            uses += [use._replace(frames=here + use.frames) for use in earlier.uses]
            notices += [notice._replace(frames=here + notice.frames) for notice in earlier.notices]
            direct_runs += [run._replace(frames=here + run.frames) for run in earlier.direct_runs]
            bypasses += [bypass._replace(frames=here + bypass.frames) for bypass in earlier.bypasses]
            listed_scripts += earlier.listed_scripts
            continue
        scripts_read.append(path)
        inner = scan_uses(body, command.directory, command.environment, here, path, state, command.memory_capped, strict=False,
                           positional=tuple(command.words[1:]), directory_guarded=command.cd_failure_skips, interactive=body_interactive)
        merge(inner)
        # 它自己在帧里的下标是 len(here)：绕回比它更外的、撞上嵌套上限的、判的时候写出了文件的，换一处起就可能不一样，不复用
        depends_on_where_started = inner.hit_depth_limit or (inner.lowest_cycle_frame is not None and inner.lowest_cycle_frame < len(here))
        if not depends_on_where_started and state.written_version == judged_key[2]:
            state.judged[judged_key] = Scan([use._replace(frames=use.frames[len(here):]) for use in inner.uses],
                                                 [notice._replace(frames=notice.frames[len(here):]) for notice in inner.notices],
                                                 [], False, None,
                                                 [run._replace(frames=run.frames[len(here):]) for run in inner.direct_runs],
                                                 [bypass._replace(frames=bypass.frames[len(here):]) for bypass in inner.bypasses],
                                                 list(inner.listed_scripts))
    return Scan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, direct_runs, bypasses, listed_scripts)

# 一切跑项目代码的命令只认经 task-server/task 起；拒绝说明里的规矩、出路按谁在跑、被拒的是哪一类写。
TASK_SERVER_RULE = ("→ 规矩：一切跑项目代码的命令只认经 task-server/task 起，主 agent 与子 agent 同判（records/2026-10-30-编译绕过内存总上限.md 批 3b ①）："
                      "会编的 cargo（build、check、clippy、test、run、bench、doc，别名与插件照会编算；fmt、metadata、tree、clean 不编，直接跑）、直接执行编出来的二进制、"
                      "本仓里的脚本（门禁阶段与格、入口脚本、实验装置与复跑、变异、各脚本的 --selftest 与样本档与弄坏开关、喂钩子）、本仓之外的外部程序（qemu-system-*、herd7、"
                      "e152-file-system-benchmark，只问版本的 herd7 除外）；"
                      "经 run-with-memory-cap.sh、capped.sh、multi-host-run.sh 直接起的照拒（交给服务的命令不带这几样，服务自己套包装、定额度、挑机器、分卡）。"
                      "直接跑的只剩只读命令（只读的 git、ls、grep、cat、sed -n、wc、sha256sum、diff、du、ps 这类）、语法检查（bash -n、python3 -m py_compile）"
                      "与名单上的文本工具（.claude/hooks/lib_project_code_runs.py 的 TEXT_TOOLS_RUN_DIRECTLY，子 agent 起其中的 task-server/task 只许 status、log；"
                      "主 agent 另有 MAIN_AGENT_TOOLS_RUN_DIRECTLY 那几样会话工具）。")
TASK_SERVER_POLICY = (TASK_SERVER_RULE + "\n"
                      "→ 怎么办：`task-server/task submit -- <原命令>`（当场打出任务号；范围声明写在 submit 前面：`SINGLEFS_RUN_SCOPE='…' task-server/task submit -- <命令>`），"
                      "再用 run_in_background 起 `task-server/task wait <号>`，结束本轮等完成通知（wait 的退出码就是任务的，末行 TASK_SERVER_RESULT 写错误类型与日志）；"
                      "命令里有 &&、管道、cd 的写成 `task-server/task submit [--cwd <目录>] -- bash -c '<命令>'`。提交写法在 task-server/README.md「提交写法」。")
# 子 agent 不往统一任务服务交任务（文件头「经 task-server 起」②）：直接跑被拒（①）与起客户端被拒（②）的出路是同一句，写在 lib_project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT
SUBAGENT_TASK_SERVER_RULE = ("→ 规矩：子 agent 一律不往统一任务服务交任务（用户 2026-11-01「现在需要发给 task-server的任务 都不能自己启动」；服务分不出提交方是不是子 agent，"
                             "拒在这道闸，records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节）：task-server/task 只许起只读的 status、log，"
                             "交什么命令都一样（带 --item 的、harness 测试、clippy / build、自证、样本档、弄坏开关、命名检查都算）。")

# 文件头「经 task-server 起」④：要临时 prune-server 的自证写成了裸的 --selftest。弄坏开关 temporary-service-remedy-dropped 下不贴这一段（说明只剩哪一份）
TEMPORARY_SERVICE_POLICY = "" if guard_break_is_set("temporary-service-remedy-dropped") else (
    "→ 规矩：.claude/hooks/selftest-temporary-service.tsv 里判定是 required 的自证，--selftest 开跑先核临时 prune-server（自证不写真服务），"
    "没有就退 2、什么都不判；要在包装 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 底下起，包装起一份临时服务、在它底下起原命令。\n"
    "→ 怎么办：照上面「整条写成」的那一条：主 agent 经 `task-server/task submit --` 交，子 agent 把那一条写进报告「要主 agent 跑的命令」一节（子 agent 自己不交），"
    "命令里不写内存包装、host-dispatch.py 与绕开包装的起法。")

# 文件头「经 task-server 起」⑥：门禁样本档点名的阶段要临时 prune-server、外面没套包装
TEMPORARY_SERVICE_STAGE_SAMPLES_POLICY = (
    "→ 规矩：stage-selftest.sh 用 --item 点名的门禁阶段，源码里调了临时服务核对的（lib_project_code_runs.temporary_service_bare_stage_samples 每次现读阶段源码，不另写名单），"
    "进样本档先核临时 prune-server（样本的判定不写真服务），没有就退 2、一份样本都不判；要在包装 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 底下起，"
    "包装起一份临时服务、在它底下起原命令。\n"
    "→ 怎么办：照上面「整条写成」的那一条：主 agent 经 `task-server/task submit --` 交，子 agent 把那一条写进报告「要主 agent 跑的命令」一节（子 agent 自己不交），"
    "命令里不写内存包装、host-dispatch.py 与绕开包装的起法。")

def subagent_task_server_policy(direct_run):
    """子 agent 被拒时的规矩与出路：direct_run 为真是直接跑项目代码被拒（①，先贴 TASK_SERVER_RULE），为假是起客户端被拒（②）。
    出路那一句取 lib_project_code_runs（共用模块读不到时这一道整个不判，到不了这里）。"""
    return ((TASK_SERVER_RULE + "\n") if direct_run else "") + SUBAGENT_TASK_SERVER_RULE + "\n→ 怎么办：" + project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT + "。"
# 绕开 task-server 的写法（文件头「经 task-server 起」③）
TASK_SERVER_BYPASS_POLICY = ("→ 规矩：加任务、取消任务、看任务、删任务现场都走 task-server/task 的 submit / wait / status / cancel / log / clean（用户定「主agent 增加redis任务和取消redis任务都要走标准接口 不可以绕过」）："
                             "库文件、签名密钥、socket 只有服务读写；服务给的编译目录归树、不归任务，只由服务删：树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层（哪一层都没有就是解开后的 --cwd 本身），树删了服务跟着删它那一整组（task-server/README.md「编译目录」）；「已由 task-server 准入」开关只由服务设给内存包装；服务本体与每一次尝试的那一层只由 systemd 与服务起；"
                             "服务的单元只经 task-server/control 的 install / start / stop / check / watch 起停。\n"
                             "→ 怎么办：改用 task-server/task 的那几个子命令或 task-server/control；要查服务的状态用 `task-server/control check`、`task-server/task status`；"
                             "服务给的编译目录不用自己删：要它没，删掉它的树（在自己建的仓副本里交的，副本根上有 .git，删那份副本；--cwd 解开符号链接后往上哪一层都没有 .git 的，删解开后的那个 --cwd 目录），"
                             "服务下一回扫到树不在了就删它那一整组；主工作树那一组留给之后的任务接着编。"
                             "子 agent 起 task-server/task 只许 status、log：要取消、要删任务现场的写进报告「要主 agent 跑的命令」一节交回。"
                             "命令里只是搜这几个名字的（grep 库文件名这类），把名字拆开写（`grep -n 'task-server\\.sqlite'`）。")
SCRIPT_POLICY = ("→ 这一步写在脚本里：命令位置上执行的脚本（bash / sh / ./ / source 起的，经 capped.sh / nice / timeout / env 包一层的，脚本里再起的脚本）"
                 "读进去逐条照同一张表判。把那一步从脚本里拿掉再跑，或照上一条交主 agent。")
UNRECOGNIZED_POLICY = ("→ 规矩：认不出起的是什么的写法一律拒（用户 2026-10-29 定「认不出的一律拒（偏严）」）：命令词、bash / source 起的脚本路径、shell 启动文件（BASH_ENV / ENV、--rcfile、HOME 与 ZDOTDIR 下的）路径、直接执行的脚本 #! 行指到认不出的解释器而环境里设了 BASH_ENV / ENV 的、cargo test / run / bench 的子命令与参数"
                       "由变量、命令替换、位置参数（`\"$@\"`、`$1`）、进程替换拼出而执行前展开不了的；xargs、find -exec、parallel 把读进来的词接进命令里的；"
                       "同一条命令里先用 echo / printf / dd / sed / head / python 这类写法写出、再起的脚本；读进来的脚本正文里命令词或脚本路径解不出、"
                       "而定义或起它的参数里写着自证、门禁阶段、入口脚本或 cargo 的。\n"
                       "→ 怎么改：写成展开之后的字面命令（`cargo test -p <包> --test <目标> -- <测试全名> --exact`、`bash <脚本的字面路径>`），变量只用在同一条命令里前面赋过字面值的；"
                       "要先写出脚本的，用 `cat > 文件 <<'EOF'` 或 cp 写出、再起它；要把参数交给命令的，直接写那条命令，不经 `bash -c '…\"$@\"' _ …`、"
                       "自己定义的转发函数或 xargs；脚本里起自证、门禁阶段、入口脚本或 cargo 的，每一条写成字面路径与字面参数（不经数组、变量或 \"$@\" 拼），"
                       "要喂给 shell 跑的写成 `bash <脚本的字面路径>` 或 `bash < 文件`、`… | bash`（不经复合命令、函数、别的描述符转交标准输入）；"
                       "别名、子 shell / 管道段 / 放后台 / `&&` `||` 之后 / if 与循环里赋的变量不拿来当命令词，直接写那条命令；make -C、git -c 的值写成字面的；"
                       "改过的钩子或脚本的自证交主 agent 在主工作区或集成副本里跑。子 agent 改不成的写进交回报告交主 agent；主 agent 确实要这样跑的，经 `task-server/task submit --` 提交（服务那边不判这一条）。")
# 主 agent 被拒：规矩只贴出处一行（规矩正文在下面这几份里，不在拒绝信息里整段贴）
MAIN_AGENT_RULE_POINTER = ("→ 规矩只贴出处：.claude/rules/verification.md（一切跑项目代码的命令经 task-server/task 起、谁起、什么时候起）；"
                           "整轮门禁照 .claude/skills/gate/staged-gate.md 跑 scripts/gate/gate-staged.sh，提交时的崩溃点重放照 .claude/skills/crash-test/commit-stages.md。")
COPY_COMMAND_LIMIT = 2000   # 照抄的那条命令超过这么多字就不整条贴，只写怎么改

def single_scope_run(candidate, directory):
    """照抄的那一条命令文本里直接起的要声明的验证命令恰好一条、不在驱动脚本里时交 (lib_run_scope 模块, 那一条)，别的交 (None, None)；
    判法取同目录的 lib_run_scope.py（读不到、判的时候抛异常都交 (None, None)）。run_scope_declaration 与 submit_loosening_flag 共用。"""
    try:
        run_scope_spec = importlib.util.spec_from_file_location("lib_run_scope", os.path.join(HOOK_DIRECTORY, "lib_run_scope.py"))
        run_scope = importlib.util.module_from_spec(run_scope_spec)
        run_scope_spec.loader.exec_module(run_scope)
        run_scope.load_judging_libraries()
        runs = run_scope.scope_runs_in_text(candidate, directory)
    except Exception:
        return None, None
    if len(runs) != 1 or runs[0].frames:
        return None, None
    return run_scope, runs[0]

def run_scope_declaration(candidate, directory):
    """照抄的那条要带的四段范围声明（写法照 .claude/hooks/run-scope-guard.sh 文件头；为什么一段是占位，照抄的人要填）：
    命令文本里直接起的要声明的验证命令恰好一条、还没带声明时交声明原文；不是验证命令、几条、在驱动脚本里、已带声明的交 None。
    判法取同目录的 lib_run_scope.py（读不到、判的时候抛异常都交 None）；弄坏开关 copy-without-run-scope 下一律交 None。"""
    if guard_break_is_set("copy-without-run-scope"):
        return None
    run_scope, run = single_scope_run(candidate, directory)
    if run is None or run.declaration is not None:
        return None
    shape = run.shape
    if any(mark in text for text in (shape.what, shape.range_hint) for mark in ("'", ";")):
        return None
    return (f"{run_scope.DECLARATION_VARIABLE}='种类={shape.kind}; 跑什么={shape.what}; 范围={shape.range_hint}; "
            "为什么=<为什么这么跑、这个范围为什么合适>'")

LOOSENING_FLAG = "--full-gate"   # 提交时那一趟不指定范围地跑整道门禁或入口的 submit 标识（lib_run_scope.LOOSENING_SUBMIT_FLAGS 里的一个）

def submit_loosening_flag(candidate, directory):
    """照抄的那一条是提交时才许不指定范围地跑的整道门禁或入口（lib_run_scope.UNSCOPED_COMMIT_ENTRIES，种类 whole）时交 --full-gate，别的交空串：
    经 submit 交上去不带它的，范围声明闸的 commit_gate_problem 当场拒（提交时才许跑）；带它的由主 agent 提交时弹窗要用户确认（run-scope-guard.sh 文件头「放宽检查的提交标识」），
    不是这里替它放行。判法同 run_scope_declaration；弄坏开关 copy-without-full-gate-flag 下一律交空串。"""
    if guard_break_is_set("copy-without-full-gate-flag"):
        return ""
    run_scope, run = single_scope_run(candidate, directory)
    if run is None:
        return ""
    return LOOSENING_FLAG if run.shape.kind == "whole" and run.shape.script_key in run_scope.UNSCOPED_COMMIT_ENTRIES else ""

def launcher_or_dispatch_launcher(words, directory=None):
    """command_under_launcher 剥得开的剥一层，剥不开再试派发起法 multi-host-run.sh --peer … --（lib_project_code_runs.py 的 command_under_dispatch_launcher）。"""
    return project_code_runs.command_under_launcher(words, directory) or project_code_runs.command_under_dispatch_launcher(words, directory)

def dispatch_launchers_seen():
    """交给 lib_workspace_cargo_lock.py 的判定库：command_under_launcher 一项换成 launcher_or_dispatch_launcher，里面起的 cargo 照样判编译目录；
    它用到的另外三样（command_under_gpu_quiesce、command_under_run_build_directory、cargo_subcommand）直接取 lib_project_code_runs 那一份
    （逐个写名字，不按名字现取：门禁按条复用收闭包时 getattr(<模块>, <变量>) 拒算）。"""
    return types.SimpleNamespace(
        command_under_launcher=launcher_or_dispatch_launcher,
        command_under_gpu_quiesce=project_code_runs.command_under_gpu_quiesce,
        command_under_run_build_directory=project_code_runs.command_under_run_build_directory,
        cargo_subcommand=project_code_runs.cargo_subcommand)

def scan_decision(hook_input, project_root):
    """返回 Decision：code 0 放行、2 拒绝；notices 是读不到、看不全的脚本，放不放行都照记。
    判三样：认不出起什么的写法与判不出的通配（UNRECOGNIZED_KIND、pattern-unresolved）、要经 task-server 起的外部程序（external-program，lib_project_code_runs.classify）、
    主工作区里直接起且编译目录会落进主仓的 cargo；本仓的代码、子 agent 起客户端与绕开服务的写法由 task_server_decision 接着判。"""
    if guard_break_is_set("disable-check"):
        return Decision(0, None, [], [], [])
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = JudgingState()
    scan = scan_uses(command, directory, state=state)
    notices = [f"脚本 {frames_text(notice.frames)} 里{notice.text}" if notice.frames else notice.text for notice in scan.notices]
    external_refused, in_script, unrecognized_refused = [], False, []
    for use in scan.uses:
        where = f"脚本 {frames_text(use.frames)} 里的 " if use.frames else ""
        in_script = in_script or bool(use.frames)
        if use.test.kind in ("external-program", "project-code"):
            if not frames_mention_listed(use.frames, scan.listed_scripts, agent_type):
                external_refused.append(f"{where}{use.test.detail}")
            continue
        unrecognized_refused.append(f"{where}{use.test.detail}")
    target_refusal = None
    if "cargo" in command:   # 粗筛：命令文本里连这个词都没有的不用再切一遍
        if cargo_lock is None:
            notices.append(f"读不到 {os.path.join(sys.argv[1], 'lib_workspace_cargo_lock.py')}：{cargo_lock_error!r}，这条命令里的 cargo 没判编译目录会不会落进主仓")
        else:
            # 主工作区里直接起、编译目录会落进主仓的 cargo（文件头「再一道」编译目录那一段）：说明单独成一段，后面照它那样拒（不给「照抄」那一条）
            target_refusal, target_notices = cargo_lock.workspace_cargo_target_refusal(command, directory, shell_words, dispatch_launchers_seen(), project_root)
            notices += target_notices
    external_refused = list(dict.fromkeys(external_refused))
    if not external_refused and not unrecognized_refused:
        if target_refusal:   # 只有编译目录这一处要拒：不给「照抄」那一条
            return Decision(2, target_refusal, notices, scan.scripts_read, state.files_read, scan)
        return Decision(0, None, notices, scan.scripts_read, state.files_read, scan)
    lines = []
    if external_refused:
        who = agent_type or "主 agent"
        lines += ([f"✗ 不经 task-server 起要经服务的外部程序被拒（{who}）：{external_refused[0]}"] + [f"  另有：{entry}" for entry in external_refused[1:]]
                  + [subagent_task_server_policy(True) if agent_type else TASK_SERVER_POLICY])
    if unrecognized_refused:
        lines += [f"✗ 认不出起什么的写法被拒：{unrecognized_refused[0]}"] + [f"  另有：{entry}" for entry in unrecognized_refused[1:]]
        lines.append(UNRECOGNIZED_POLICY if agent_type is not None else MAIN_AGENT_RULE_POINTER)
    if in_script:
        lines.append(SCRIPT_POLICY)
    message = "\n".join(lines)
    if target_refusal:
        message = target_refusal + "\n" + message
    if state.missing_cat_sources and not guard_break_is_set("cat-missing-source-unexplained"):
        # cat / tee 的源盘上不在、读不到它的内容，那个目的照认不出拒（偏严：cat 不看源在不在那一类照拒），说明点名这一种写法与改法
        named = "、".join(f"`{source}`（写进 {destination}）" for source, destination in state.missing_cat_sources)
        message += (f"\n→ 这一条里 cat / tee 的源 {named} 盘上不在、也不是同一条命令里前面认出写出的：真跑时 cat 报 No such file、退 1，用 `&&` 接的后一条其实不跑；"
                    "读不到这个源的内容，写出的那份目的照认不出、执行它照拒。要跑的话把不在的源去掉，或把引号写的 `\"2\">` 改成重定向 `2>` 本来想写的样子，再拆成两条命令。")
    return Decision(2, message, notices, scan.scripts_read, state.files_read, scan)


# 交给 task-server 的那一条照抄时：开头这几样变量写在 submit 前面（客户端照白名单带进任务）
SUBMIT_CARRIED_PREFIX_VARIABLES = ("SINGLEFS_RUN_SCOPE",)
LEADING_ASSIGNMENT = re.compile(r"\A\s*([A-Za-z_][A-Za-z0-9_]*)=(?:'[^']*'|\"(?:[^\"\\]|\\.)*\"|\S*)[ \t]+")
LEADING_MEMORY_CAP = re.compile(r"\A(?:bash\s+)?(?:\S*/)?scripts/hosts/run-with-memory-cap\.sh\s+\S+\s+")
SHELL_SYNTAX_MARKS = ("&&", "||", ";", "|", "\n", "$(", "`", "<", ">", "cd ")

def submit_candidate(command, flag=""):
    """被拒的那条改写成交给 task-server 的样子：开头的范围声明挪到 submit 前面，开头一层 run-with-memory-cap.sh <上限> 去掉
    （交给服务的命令不带包装）；剩下的有 &&、管道、cd、重定向这类 shell 写法的包进 `bash -c '…'`；flag 不空时写在 submit 与 -- 之间（submit_loosening_flag）。"""
    carried, rest = [], command.strip()
    while True:
        found = LEADING_ASSIGNMENT.match(rest)
        if not found or found.group(1) not in SUBMIT_CARRIED_PREFIX_VARIABLES:
            break
        carried.append(found.group(0).strip())
        rest = rest[found.end():]
    rest = LEADING_MEMORY_CAP.sub("", rest, count=1)
    if guard_break_is_set("copy-test-names-dropped") and " -- " in rest:   # 弄坏开关：照抄的 cargo test 丢了测试全名与 --exact
        rest = rest.split(" -- ", 1)[0]
    inner = f"bash -c {shlex.quote(rest)}" if any(mark in rest for mark in SHELL_SYNTAX_MARKS) else rest
    return " ".join(carried + [f"task-server/task submit {flag} --" if flag else "task-server/task submit --", inner])

def task_server_copy_lines(command, directory, agent_type):
    """被拒时放在最前面的照抄一行：改写成经 task-server 提交的那条（submit_candidate）；是要声明范围的验证命令、开头还没带声明的，四段声明写在 submit 前面
    （run_scope_declaration，为什么一段是占位）；照主 agent 交它再判过放行才给，太长的只写怎么改。主 agent 照它提交；子 agent 不交（文件头「经 task-server 起」②），
    照它写进报告「要主 agent 跑的命令」一节交回。弄坏开关 task-server-copy-dropped 下不给。"""
    if guard_break_is_set("task-server-copy-dropped"):
        return []
    candidate = submit_candidate(command)
    loosening_flag = submit_loosening_flag(candidate, directory)
    if loosening_flag:
        candidate = submit_candidate(command, loosening_flag)
    declaration = None if candidate.startswith(SUBMIT_CARRIED_PREFIX_VARIABLES[0] + "=") else run_scope_declaration(candidate, directory)
    if declaration is not None:
        candidate = f"{declaration} {candidate}"
    judged = task_server_decision({"tool_name": "Bash", "cwd": directory, "tool_input": {"command": candidate}}, directory, copy=False)
    if judged.code != 0:
        return []
    if agent_type:
        if len(candidate) > COPY_COMMAND_LIMIT:
            return [f"→ 写进报告「要主 agent 跑的命令」一节：这条命令 {len(candidate)} 字，不整条贴；写成 `task-server/task submit -- <原命令>`（范围声明挪到 submit 前面，内存包装去掉），子 agent 自己不交。"]
        return ["→ 写进报告「要主 agent 跑的命令」一节的那一条（子 agent 自己不交，由主 agent 交）：", candidate]
    if len(candidate) > COPY_COMMAND_LIMIT:
        return [f"→ 照抄：这条命令 {len(candidate)} 字，不整条贴；写成 `task-server/task submit -- <原命令>`（范围声明挪到 submit 前面，内存包装去掉），提交之后用 run_in_background 起 `task-server/task wait <号>`。"]
    return ["→ 照抄（提交之后当场打出任务号，再用 run_in_background 起 `task-server/task wait <号>`、结束本轮等完成通知）：", candidate]

def frames_mention_listed(frames, listed, agent_type):
    """这一处出在名单上直接跑的脚本的正文里（谁都能跑的那几份，或主 agent 跑的会话工具）：它整份照名单放行，正文里的几行不另算。"""
    allowed = {path for path, level in listed if level == "anyone" or (level == "main-agent" and agent_type is None)}
    return any(os.path.realpath(path) in allowed for path, _line in frames if path)

def task_server_decision(hook_input, project_root, copy=True):
    """文件头「经 task-server 起」：先判认不出的写法、外部程序与编译目录（scan_decision），再加这三样——① 不经 task-server 跑项目代码的拒，
    经 task-server/task submit 交给服务的不判内存包装；② 子 agent 起 task-server/task 的 submit、wait、cancel、clean 拒；③ 绕开服务的写法拒。
    返回 Decision。弄坏开关 task-server-direct-allowed 下 ① 的直接跑不判，subagent-task-client-allowed 下 ② 不判，
    task-server-bypass-allowed 下 ③ 不判（scan_uses 里不记、文本这一层不看）。"""
    category = scan_decision(hook_input, project_root)
    scan = category.scan
    if scan is None:   # disable-check：整道不判
        return category
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    notices = list(category.notices)
    guarded, guarded_problem = project_code_runs.task_server_guarded()
    if guarded_problem and not guard_break_is_set("task-server-bypass-allowed") and command:
        notices.append(f"{guarded_problem}：绕开 task-server 的写法这一条没判")
    bypass = [] if guard_break_is_set("task-server-bypass-allowed") else project_code_runs.task_server_text_bypass(command)[0]
    bypass += [f"脚本 {frames_text(item.frames)} 里的 {item.detail}" if item.frames else item.detail for item in scan.bypasses]
    where = lambda run: f"脚本 {frames_text(run.frames)} 里的 {run.detail}" if run.frames else run.detail
    direct = []
    if not guard_break_is_set("task-server-direct-allowed"):
        for run in scan.direct_runs:
            if ((run.allowed == "main-agent" and agent_type is None) or run.allowed in (SUBAGENT_CLIENT_LEVEL, TEMPORARY_SERVICE_BARE_LEVEL, TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL, CLIPPY_LINT_TABLE_LEVEL)
                    or frames_mention_listed(run.frames, scan.listed_scripts, agent_type)):
                continue
            direct.append(where(run))
    # 要经服务的外部程序（scan_decision 判成 external-program 的）也是直接跑项目之外的程序，照 ① 给照抄的提交写法
    external = any(use.test.kind in ("external-program", "project-code") and not frames_mention_listed(use.frames, scan.listed_scripts, agent_type) for use in scan.uses)
    # ② 子 agent 起提交客户端的 submit、wait、cancel、clean（scan_uses 记成 SUBAGENT_CLIENT_LEVEL 的那几处）；主 agent 不判
    client = []
    if agent_type and not guard_break_is_set("subagent-task-client-allowed"):
        client = [where(run) for run in scan.direct_runs
                  if run.allowed == SUBAGENT_CLIENT_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    # 文件头「经 task-server 起」④：要临时 prune-server 的自证写成裸的 --selftest（scan_uses 记成 TEMPORARY_SERVICE_BARE_LEVEL，submit 里那一条的也收），谁起都拒
    bare = [] if guard_break_is_set("temporary-service-bare-allowed") else [
        where(run) for run in scan.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    # 文件头「经 task-server 起」⑥：门禁样本档点名的阶段要临时 prune-server、外面没套包装（scan_uses 记成 TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL，submit 里那一条的也收），谁起都拒
    stage_samples = [] if guard_break_is_set("temporary-service-stage-samples-allowed") else [
        where(run) for run in scan.direct_runs if run.allowed == TEMPORARY_SERVICE_STAGE_SAMPLES_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    # 文件头「经 task-server 起」⑤：cargo clippy 没带全 lint 表（scan_uses 记成 CLIPPY_LINT_TABLE_LEVEL，submit 里那一条的也收），谁起都拒
    clippy = [] if guard_break_is_set("clippy-lint-table-unchecked") else [
        where(run) for run in scan.direct_runs if run.allowed == CLIPPY_LINT_TABLE_LEVEL and not frames_mention_listed(run.frames, scan.listed_scripts, agent_type)]
    unique = lambda entries: list(dict.fromkeys(entries))
    bypass, direct, client, bare, stage_samples, clippy = unique(bypass), unique(direct), unique(client), unique(bare), unique(stage_samples), unique(clippy)
    if not (bypass or direct or client or bare or stage_samples or clippy):
        if external and copy and category.message:
            return category._replace(message="\n".join(task_server_copy_lines(command, directory, agent_type) + [category.message]), notices=notices)
        return category._replace(notices=notices)
    lines = []
    if bypass:
        lines += [f"✗ 绕开 task-server 的写法被拒：{bypass[0]}"] + [f"  另有：{entry}" for entry in bypass[1:]] + [TASK_SERVER_BYPASS_POLICY]
    if client:
        lines += ([f"✗ {agent_type} 起统一任务服务的客户端被拒（子 agent 只许 status、log）：{client[0]}"] + [f"  另有：{entry}" for entry in client[1:]]
                  + [subagent_task_server_policy(False)])
    if direct:
        who = agent_type or "主 agent"
        lines += ([f"✗ 不经 task-server 跑项目代码被拒（{who}）：{direct[0]}"] + [f"  另有：{entry}" for entry in direct[1:]]
                  + [subagent_task_server_policy(True) if agent_type else TASK_SERVER_POLICY])
    if (direct or external) and copy and not bypass:
        lines = task_server_copy_lines(command, directory, agent_type) + lines
    if bare:
        lines += [f"✗ 要临时 prune-server 的自证写成了裸的 --selftest 被拒：{bare[0]}"] + [f"  另有：{entry}" for entry in bare[1:]] + ([TEMPORARY_SERVICE_POLICY] if TEMPORARY_SERVICE_POLICY else [])
    if stage_samples:
        lines += [f"✗ 门禁样本档要临时 prune-server、外面没套包装被拒：{stage_samples[0]}"] + [f"  另有：{entry}" for entry in stage_samples[1:]] + [TEMPORARY_SERVICE_STAGE_SAMPLES_POLICY]
    if clippy:
        lines += [f"✗ cargo clippy 没带全 check.sh 的 lint 表、放行了表里的 lint、或 check.sh 的表读不成，被拒：{clippy[0]}"] + [f"  另有：{entry}" for entry in clippy[1:]]
        if not guard_break_is_set("clippy-lint-table-remedy-dropped"):
            table_tail = clippy_lint_table_tail()
            if table_tail:
                lines.append(f"  出路：clippy 命令的 `--` 之后写 check.sh 现在的整段（每次现读），不再放行表里的 lint、不带 --cap-lints：{table_tail}；整轮门禁与合入后验证都带这张表，写手交的命令少一条，合入后验证才红。")
            else:
                lines.append("  出路：check.sh 是上游副本（.claude/singlefs-ai-sop/scripts/check.sh），本仓不改它：交上游把 CODE_DISCIPLINE_LINTS 改成 `CODE_DISCIPLINE_LINTS=(` 独占一行、"
                             "每行恰好一条 `-D clippy::<名>`（行尾可带 # 注释）、`)` 独占一行的写法，读法的全文在 .claude/hooks/lib_code_discipline_lints.py 文件头；"
                             "子 agent 把这一条写进报告交主 agent；上游改好之前 clippy 过不了这一道。")
    if category.code == 2 and category.message:
        lines.append(category.message)   # 认不出的写法、外部程序、编译目录那一段说明照贴
    return Decision(2, "\n".join(lines), notices, category.scripts_read, category.files_read, scan)


HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节
# hook 输入读不出来时记检出用的那一份：会话号、agent 都读不到，照实写读不到
UNREADABLE_HOOK_INPUT = {"session_id": "读不到", "agent_id": "读不到", "agent_type": "读不到", "transcript_path": "读不到", "tool_input": {"command": ""}}

def parsed_hook_input(raw):
    """标准输入读到的字节 → (hook 输入, None)；不是 JSON 对象（解析出错、解出来不是对象）→ (None, 一行诊断：钩子名、为什么、开头若干字节)。"""
    try:
        hook_input = json.loads(raw)
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        why = f"解析出错：{error!r}"
    else:
        if isinstance(hook_input, dict):
            return hook_input, None
        why = f"解出来是 {type(hook_input).__name__}，不是对象"
    head = raw[:HOOK_INPUT_HEAD_BYTES]
    return None, (f"project-code-guard.sh 收到的 hook 输入不是 JSON 对象（{why}），这条命令没判、照常执行；"
                  f"开头 {len(head)} 字节（共 {len(raw)} 字节）：{head!r}")

def record_detection(hook_input, findings, detections_path, outcome):
    """检出记录追加一行；outcome 是这条命令的结局：「被拒」（执行前拒了、没执行）或「只记」（照常执行）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": (command if isinstance(command, str) else repr(command))[:500],
        "findings": findings,
    }
    if not guard_break_is_set("detection-outcome-dropped"):
        entry["outcome"] = outcome
    if not detections_path:   # 读不到 lib_work_root.py、又没设 AGENT_HOOK_DETECTIONS：不知道往哪写
        return
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass
