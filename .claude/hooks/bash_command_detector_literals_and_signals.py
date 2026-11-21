# 检出钩子 bash-command-detector.sh 的 python 部分之一：账本追加（⑯）、records 追加（⑲）、字面内容里的会话号与批号（⑰）、就地改脚本（⑦）、终止进程（⑥）、扫脚本、通删工作根（⑪）、手拷仓副本（⑳）、selftest 入口。
# 入口 bash-command-detector.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 DETECTOR_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import ast
import glob
import importlib.util
import io
import itertools
import math
import os
import re
import shlex
import sys
import tokenize
from typing import NamedTuple
from __main__ import detector_break_is_set
from bash_command_detector_hooks_and_overwrites import (
    APPENDING_REDIRECTS, BRACE_PIECE, COMPRESSOR_NAMES, COPY_SHORT_OPTIONS_WITH_VALUE, FILE_DESCRIPTOR_WORD, LEG_PROMPTS_PREFIX,
    OVERWRITING_REDIRECTS, OverwriteCandidate, PreservingStep, PythonCode, READ_WRITE_REDIRECT, RESULTS_DIRECTORY_WORDS, RESULTS_PRESERVING_KINDS,
    SCRATCH_OWNER_IDENTITY, SHRED_LONG_OPTIONS_WITH_VALUE, SHRED_SHORT_OPTIONS_WITH_VALUE, awk_in_place_files, command_pieces,
    compressor_removed_files, destination_words, find_path_word, find_start_paths, git_added_paths, git_apply_directory, in_place_editor_files,
    is_inside, is_main_workspace_crate_source, is_preserved, is_three_way_leg, known_write_script_steps, launcher_inner_text, launcher_input_words,
    ledger_append_root, leg_round_problem, mktemp_directory_value, mktemp_substitutions_expanded, parse_copy_arguments, patch_target,
    piped_output_bodies, plain_operands, preserved_paths, python_script_bodies, remember_working_directory, resolve_destination, rsync_removal_steps,
    scope_launcher_inner_text, service_directory_copy_hits, sort_output_files, substitute_assigned_variables, tar_extraction_directory,
    target_pattern, tee_files, tracked_verdicts, truncate_files, untracked_among, unzip_extraction_directory, writer_lock_refusal,
)
from bash_command_detector_scope_and_index import (
    CLIPPY_LINT_TABLE_RELATIVE, COMPILE_FIRST_AGENTS, COMPRESSOR_FORM, DEVICE_PREFIXES, HOOK_DIRECTORY, InPlaceEditVerdict, LOOP_BINDING_COMMANDS, LOOP_VALUE_MARK,
    LedgerAppendTarget, MAXIMUM_LOOP_LANDINGS, PYTHON_UNLITERAL_LISTED, READING_COMMANDS, SCRIPT_SCRATCH_ROOTS, SERVICE_FORM_MODES,
    SERVICE_SOURCE_AGENTS, ScopeVerdict, ServiceSourceView, UnknownScriptTarget, UnresolvedScriptTarget, agent_has_definition, agent_has_edit_tool,
    agent_scope_patterns, bind_loop_variables, clippy_lint_tail, definition_can_write, home_expanded, loop_keyword_changes, loop_value_choices,
    python_repository_write_regex, python_repository_writes, readable_loop_values, sed_in_place_targets, sed_target_resolved_in_scratch,
    shell_code_and_timeout, shell_layers, shell_words, with_loop_values, work_roots, wrapped_commands, write_scope, write_scope_error,
)

# ⑯ 往现行账本直接追加节标题（文件头 ⑯）：登记表读法与 write-guard.sh 第十道各写一份，节标题的写法同门禁 ledger-section-numbers 格
LEDGER_REGISTRY_PATH = os.path.join(".claude", "gate.d", "split-proposal-ledgers.tsv")
LEDGER_HEADING_TEXT = re.compile(r"##[ \t]*[零〇一二两三四五六七八九十百千]+、")
LEDGER_WRITING_FORMS = OVERWRITING_REDIRECTS | APPENDING_REDIRECTS | {"tee"}

def current_ledger_relative(repository_root):
    """登记表里 current 那一行点名的账本 → (相对仓根的路径, None)；表读不到或 current 不是恰好一行 → (None, 原因)。"""
    registry = os.path.join(repository_root, LEDGER_REGISTRY_PATH)
    try:
        with open(registry, encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError as error:
        return None, f"读不到 {registry}（{type(error).__name__}: {error}）"
    current = [parts[1] for parts in rows if len(parts) == 4 and parts[0] == "current"]
    if len(current) != 1:
        return None, f"{registry} 里 current 的行有 {len(current)} 行，要恰好一行"
    return current[0], None

def lands_on_ledger(path, base, ledger):
    """路径（相对的从 base 起算，~ 展开）解开符号链接之后是不是那份账本。"""
    expanded = os.path.expanduser(path)
    return os.path.realpath(expanded if os.path.isabs(expanded) else os.path.join(base, expanded)) == ledger

# ⑲ 往 records/ 下的 .md 追加（文件头 ⑲）：追加只经 scripts/kb/append-record.py，它写之前照同一份判据库判要追加的文本；这里不看内容，认出追加就拒
RECORDS_DIRECTORY_NAME = "records"
RECORD_APPEND_TOOL = "scripts/kb/append-record.py"
RECORD_APPEND_UNLISTED_TOOL = "scripts/kb/history-consolidate.py"   # 弄坏开关 record-howto-unlisted-tool 下出路写成它：不在 lib_project_code_runs.py 的直接跑名单上，照抄会被 project-code-guard.sh 拒
RECORD_HOWTO_TEXT_PLACEHOLDER = "<草稿目录里写好的文本文件>"   # 出路里文本文件那一处的占位，自证照抄出路喂 project-code-guard.sh 时换成真文件
# ⑲ 出路喂给同一触发点上别的闸（project-code-guard.sh）时用的环境：本钩子这一趟自证设着的弄坏开关（名字以 _BREAK 收尾的变量，含 LIB_SHELL_WORDS_BREAK 这类共用库的）不带过去。
# 别的钩子见到不是它自己自证设的弄坏开关就拒（退 2），出路过不了闸是被这个开关挡的，不是出路写错。弄坏开关 howto-guard-environment-keeps-breaks 下原样带过去
def howto_guard_environment(base, detections_path):
    environment = dict(base, AGENT_HOOK_DETECTIONS=detections_path)
    if detector_break_is_set("howto-guard-environment-keeps-breaks"):
        return environment
    return {name: value for name, value in environment.items() if not name.endswith("_BREAK")}

RECORD_HOWTO_COMMAND = re.compile(r"→ 怎么办：追加的文本先写进草稿目录里的文件，再 `([^`]*)`")   # 自证从拒绝说明里取出路的整条写法

def record_howto_tool():
    """⑲ 出路里写的追加工具；弄坏开关 record-howto-unlisted-tool 下换成不在直接跑名单上的脚本。"""
    return RECORD_APPEND_UNLISTED_TOOL if detector_break_is_set("record-howto-unlisted-tool") else RECORD_APPEND_TOOL
PYTHON_OPENING_MODE = re.compile(r"^python (?:open\(…, |Path\.open\()'([^']*)'\)$")   # python_in_place_writes 报的 open 写法，引号里是模式

def record_append_landing(path, directory, repository_root):
    """追加的落点（python 里的字面路径，或 shell 目标词算出的样式）落在本仓 records/ 下一份 .md（后缀不分大小写，子目录里的也算）上就交它相对仓根的路径，否则交 None。
    相对路径从 directory 起算，directory 是 None（cd 算不出）的交 None；落点规整 .. 并解开符号链接，与同样解开链接的 <仓根>/records 比
    （records 同名前缀的兄弟目录不算；仓根下的 records 是链接的，比的是它指到的目录，hook-swap 的草稿树就是这样）。"""
    if directory is None and not os.path.isabs(path):
        return None
    records = os.path.realpath(os.path.join(repository_root, RECORDS_DIRECTORY_NAME))
    landing = os.path.realpath(path if os.path.isabs(path) else os.path.join(directory, path))
    if not landing.lower().endswith(".md") or not landing.startswith(records + os.sep):
        return None
    return os.path.join(RECORDS_DIRECTORY_NAME, os.path.relpath(landing, records))

def repository_root_of(hook_dir):
    """这个 hook 所在的仓：`.claude/hooks/` 往上两级。"""
    return os.path.realpath(os.path.join(hook_dir, os.pardir, os.pardir))

# ⑰ 往仓里的文件写字面内容：会话号、钟点、批次代号、任务号与会话短号与批号照写闸 write-guard.sh 第五道、第七道同一份判据库（../gate.d/lib-forbidden-notations.py）判，命中就拒（文件头 ⑰）
LITERAL_NOTATION_FORMS = (("session-names", "会话号"), ("clock-times", "钟点"), ("batch-codes", "批次代号"),
                          ("task-and-session-numbers", "任务号、会话短号或批号"))
LITERAL_WHOLE_FILE_FORM = "task-and-session-numbers"   # 这一形态判写之前、之后两份整份目标（库的 new_violation_hits），别的形态判写进去的内容
LITERAL_NOTATION_LISTED = 20          # 拒绝说明里逐处列出的上限，多出的只报处数
NOTATIONS_LIBRARY = {}                # 库路径 → 装好的模块：一个进程里只装一次

class LiteralWrite(NamedTuple):
    """一条命令往一个目标里写的字面内容：写法（「cat >>」「echo >」「tee -a」这类）、目标词（前面赋过的变量已代入）、那一刻的当前目录（算不出是 None）与内容。"""
    form: str
    target: str
    directory: str | None
    text: str

NOTATIONS_LIBRARY_FILE = os.path.join(os.path.dirname(HOOK_DIRECTORY), "gate.d", "lib-forbidden-notations.py")   # 判据库在钩子所在仓的 .claude/gate.d/ 里

def load_notations_library():
    """写闸、门禁 doc-text 与这一道共用的判据库（NOTATIONS_LIBRARY_FILE）：装过的直接交回；读不到、语法错抛异常（由 main() 记判不了）。不留 __pycache__。"""
    library = NOTATIONS_LIBRARY.get(NOTATIONS_LIBRARY_FILE)
    if library is not None:
        return library
    specification = importlib.util.spec_from_file_location("lib_forbidden_notations", NOTATIONS_LIBRARY_FILE)
    library = importlib.util.module_from_spec(specification)
    before = sys.dont_write_bytecode
    sys.dont_write_bytecode = True
    try:
        specification.loader.exec_module(library)
    finally:
        sys.dont_write_bytecode = before
    NOTATIONS_LIBRARY[NOTATIONS_LIBRARY_FILE] = library
    return library

ESCAPE_LETTERS = {"a": "\a", "b": "\b", "e": "\x1b", "E": "\x1b", "f": "\f", "n": "\n", "r": "\r", "t": "\t", "v": "\v", "\\": "\\", '"': '"', "'": "'", "?": "?"}
ESCAPE_NUMBER = re.compile(r"x([0-9A-Fa-f]{1,2})|u([0-9A-Fa-f]{1,4})|U([0-9A-Fa-f]{1,8})")
PRINTF_CONVERSION = re.compile(r"%(?P<flags>[-+ #0]*)(?P<width>\*|\d+)?(?:\.(?P<precision>\*|\d*))?(?P<kind>[diouxXfFeEgGaAcsbq%])")

def backslash_escapes_expanded(text, octal_after_zero):
    """echo -e 与 printf %b（octal_after_zero 为真：八进制写成 \\0nnn）、printf 格式串（为假：\\nnn）照 bash 换掉反斜杠转义：\\a \\b \\e \\f \\n \\r \\t \\v \\\\ 与八进制、\\xHH、
    \\uHHHH、\\UHHHHHHHH；认不得的照原样留（\\q 还是 \\q）。\\c 之后的不再输出：交回 (换好的文字, 有没有碰到 \\c)。"""
    pieces, index = [], 0
    while index < len(text):
        character = text[index]
        if character != "\\" or index + 1 >= len(text):
            pieces.append(character)
            index += 1
            continue
        following = text[index + 1]
        if following == "c":
            return "".join(pieces), True
        octal = re.match(r"0([0-7]{0,3})" if octal_after_zero else r"([0-7]{1,3})", text[index + 1:])
        number = ESCAPE_NUMBER.match(text, index + 1)
        if octal:
            pieces.append(chr(int(octal.group(1) or "0", 8) & 0xFF))
            index += 1 + octal.end()
        elif number:
            code = int(next(group for group in number.groups() if group), 16)
            pieces.append(chr(code) if code <= 0x10FFFF else "�")
            index = number.end()
        elif following in ESCAPE_LETTERS:
            pieces.append(ESCAPE_LETTERS[following])
            index += 2
        else:
            pieces.append(character)
            index += 1
    return "".join(pieces), False

def printf_conversion_text(match, value, starred=()):
    """printf 的一个转换（%s、%d、%b 这类）把一个参数换成的字：数字类按 bash 的写法认（0x、'字符），认不出的照参数原样交（宁可多判）；
    %b 换转义；宽度与精度照 Python 的格式化，写成 * 的依次取 starred 里的参数（认不出数的当没写）。交回 (文字, 有没有碰到 \\c)。"""
    kind, flags, precision, width = match.group("kind"), match.group("flags") or "", match.group("precision"), match.group("width") or ""
    starred = list(starred)
    if width == "*":
        width = starred.pop(0) if starred else ""
        width = width if width.isdigit() else ""
    if precision == "*":
        precision = starred.pop(0) if starred else ""
        precision = precision if precision.isdigit() else ""
    stopped = False
    if kind in "sq":
        body = value if value is not None else ""
    elif kind == "b":
        body, stopped = backslash_escapes_expanded(value or "", True)
    elif kind == "c":
        body = (value or "")[:1]
    else:
        sign = "".join(flag for flag in "+ " if flag in flags)[:1]
        right_width = "" if "-" in flags else ("0" if "0" in flags else "") + width
        try:
            if kind in "diouxX":
                number = ord(value[1]) if value and value[0] in "'\"" and len(value) > 1 else int(value or "0", 0)
                body = format(number, sign + right_width + ("d" if kind in "iu" else kind))
            else:
                decimals = "." + precision if precision not in (None, "", "*") else ""
                body = format(float(value or "0"), sign + right_width + decimals + ("e" if kind in "aA" else kind))
            return (body.ljust(int(width)) if width and "-" in flags else body), False
        except (ValueError, IndexError):
            body = value or ""
    if precision not in (None, "", "*") and kind in "sbq":
        body = body[:int(precision)]
    if width:
        body = body.ljust(int(width)) if "-" in flags else body.rjust(int(width))
    return body, stopped

def printf_rendered(arguments):
    """printf 的参数照 bash 拼出它写到标准输出的字：格式串换掉转义、每个转换依次吃一个参数，参数没吃完就从头再套一遍格式；`-v 名字` 写进变量、不输出，交空串。
    格式里有认不出的转换（%* 之外的写法）交 None（由调用方退回逐个参数各占一行）。"""
    if arguments and arguments[0] == "--":
        arguments = arguments[1:]
    if arguments and arguments[0] == "-v":
        return ""
    if not arguments:
        return None
    template, values = arguments[0], list(arguments[1:])
    pieces, position = [], 0
    while True:
        index, consumed = 0, 0
        while index < len(template):
            if template[index] == "%":
                match = PRINTF_CONVERSION.match(template, index)
                if match is None:
                    return None
                if match.group("kind") == "%":
                    pieces.append("%")
                else:
                    starred = []
                    for star in (match.group("width"), match.group("precision")):
                        if star == "*":
                            starred.append(values[position] if position < len(values) else "")
                            position, consumed = position + 1, consumed + 1
                    value = values[position] if position < len(values) else None
                    position, consumed = position + 1, consumed + 1
                    text, stopped = printf_conversion_text(match, value, starred)
                    pieces.append(text)
                    if stopped:
                        return "".join(pieces)
                index = match.end()
                continue
            end = template.find("%", index)
            end = len(template) if end < 0 else end
            text, stopped = backslash_escapes_expanded(template[index:end], False)
            pieces.append(text)
            if stopped:
                return "".join(pieces)
            index = end
        if consumed == 0 or position >= len(values):
            return "".join(pieces)

def literal_output(words, stdin_bodies, variables=None):
    """一条命令写到标准输出的字面内容：cat（没有文件操作数，或只有 -）交它读到的 heredoc、here-string 或管道前一段的字面；
    echo 交参数（空格接起来；-n 不带收尾的换行，-e 照 bash 换掉反斜杠转义、\\c 之后不输出，-E 与没写的不换）；
    printf 照格式串拼出它真写出的字（printf_rendered：格式串里的转义、每个转换代入参数、参数没吃完就再套一遍格式），格式里有认不出的转换时退回格式串与每个参数各占一行；
    echo 与 printf 的参数里前面赋过字面值的变量（variables，这条命令里前面赋的）先代入；别的命令交 None（输出算不出，不判）。
    弄坏开关：literal-echo-escapes-unexpanded 下 echo -e 不换转义；literal-printf-format-unexpanded 下 printf 一律交格式串与每个参数各占一行；
    literal-variables-unsubstituted 下参数里的变量不代入。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if variables and not detector_break_is_set("literal-variables-unsubstituted"):
        arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
    if name == "cat":
        operands = [argument for argument in arguments if not argument.startswith("-") or argument == "-"]
        return stdin_bodies[-1].text if stdin_bodies and all(operand == "-" for operand in operands) else None
    if name == "echo":
        start = next((index for index, argument in enumerate(arguments) if not re.fullmatch(r"-[neE]+", argument)), len(arguments))
        flags = "".join(argument[1:] for argument in arguments[:start])
        text, stopped = " ".join(arguments[start:]), False
        if flags.replace("n", "").endswith("e") and not detector_break_is_set("literal-echo-escapes-unexpanded"):
            text, stopped = backslash_escapes_expanded(text, True)
        return text if stopped or "n" in flags else text + "\n"
    if name == "printf" and arguments:
        rendered = None if detector_break_is_set("literal-printf-format-unexpanded") else printf_rendered(arguments)
        return rendered if rendered is not None else "\n".join(arguments) + "\n"
    return None

def literal_write_landing(write):
    """这一处写的落点（绝对路径，解开符号链接）；目标词里有算不出的段、通配、循环变量，或当前目录算不出的，交 None（不判）。"""
    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(write.target))
    if has_unknown_piece or has_glob or LOOP_VALUE_MARK in pattern:
        return None
    path = shell_words.resolve_path(write.directory, pattern)
    return os.path.realpath(path) if path else None

def literal_landing_text(landing, written):
    """这一处写之前目标里的文字：同一条命令里前面几处已经写过这个落点的，取它们在内存里依次施加之后的样子（written，落点 → 文字）；没写过的读现文件，不在或读不了交空串。
    弄坏开关 literal-notations-before-from-disk 下一律读现文件（看不到同一条命令里前几处写进的东西）。"""
    if landing in written and not detector_break_is_set("literal-notations-before-from-disk"):
        return written[landing]
    try:
        with open(landing, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return ""

def literal_notation_remedies(hits):
    """拒绝说明的出路：命中的每种形态各一行，取库里那一形态登记的整条 remedy（各条去掉收尾的「；」「。」，用「；」接起来）。"""
    library = load_notations_library()
    named = {form_name for _, _, form_name, _, _ in hits}
    return [f"{form_name}：" + "；".join(item.rstrip("；。") for item in library.FORMS_BY_KEY[form_key]["remedy"])
            for form_key, form_name in LITERAL_NOTATION_FORMS if form_name in named]

# ⑦ 在同一个 inode 上改一个已经存在的脚本：shell 那一半沿用 ⑤ 的 overwrite_steps（同一份写法、cd 与变量），另判喂给 python 的代码
SCRIPT_SUFFIXES = (".sh", ".py")
INODE_REPLACING_FORMS = {"mv", "install"}               # 实测换 inode：已经打开着文件的读者接着读到旧内容
PYTHON_OPTIONS_WITH_VALUE = {"-W", "-X", "--check-hash-based-pycs"}
PYTHON_STANDARD_INPUT_SCRIPTS = ("/dev/fd/", "/dev/stdin")
PYTHON_OPEN_MODULES = {"io", "builtins", "codecs"}      # 这几个模块的 open 与内建 open 同一个签名：(文件, 模式)
PYTHON_PATH_CONSTRUCTORS = {"Path", "PosixPath", "PurePath", "PurePosixPath"}
PYTHON_PATH_WRITERS = {"write_text", "write_bytes"}
PYTHON_FILE_COPIERS = {"copyfile", "copy", "copy2"}      # shutil 的这三个用 'wb' 打开目标，实测在原 inode 上写
PYTHON_PATH_MAXIMUM_DEPTH = 8

class PythonWrite(NamedTuple):
    form: str            # 认出的写法，报给人看
    path: str            # 目标路径（从字面量算出来的）
    source: str | None   # shutil.copy / copy2 的源：目标是已有目录时写的是 目标/源的文件名
    mode: str | None = None   # 只在 service_writes 下交的几种：holding（删掉、挪走：落点或包着已有服务源码的目录就拒）、directory-copy（copytree 往已有目录逐份盖，path 是目标、source 是源）；别的是 None

def called_name_and_owner(function):
    """ast 调用对象的 (名字, 属主)：`open` → ("open", None)，`io.open` → ("open", "io")，`os.path.join` → ("join", "path")，属主不是名字或属性时为 None。"""
    if isinstance(function, ast.Name):
        return function.id, None
    if isinstance(function, ast.Attribute):
        receiver = function.value
        owner = receiver.id if isinstance(receiver, ast.Name) else receiver.attr if isinstance(receiver, ast.Attribute) else None
        return function.attr, owner
    return None, None

def name_binding_counts(tree):
    """整段代码里每个名字被绑定的次数：赋值、for 与推导式的目标、with … as、函数与 lambda 的参数、import、增量赋值、海象与 del 各算一次。
    `except … as`、def、class 的名字与 match 里绑的名字不数（它们在语法树里是字符串、不是 Name 或 arg），文件头 ⑦ 照这个写。"""
    bindings = {}
    for node in ast.walk(tree):
        bound = []
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            bound = [node.id]
        elif isinstance(node, ast.arg):
            bound = [node.arg]
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            bound = [(alias.asname or alias.name).split(".")[0] for alias in node.names]
        for name in bound:
            bindings[name] = bindings.get(name, 0) + 1
    return bindings

def single_assignments(tree):
    """整段代码里只绑定过一次、而且是 `名字 = 值` 这一种绑法的名字：交回 {名字: 值的节点}。for、with … as、函数参数、import、
    增量赋值与海象这些别的绑法都算一次绑定，绑过不止一次的不认（运行期拿到的是哪个值，这里说不准）。"""
    bindings, values = name_binding_counts(tree), {}
    for node in ast.walk(tree):
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            values[node.targets[0].id] = node.value
    return {name: value for name, value in values.items() if bindings.get(name) == 1}

class LoopCandidates(NamedTuple):
    paths: tuple         # for 的字面元素逐个算出的路径，按元素次序
    body_nodes: frozenset   # 循环体里每个节点的 id：只有在这里面的调用，循环变量才取这几个值（else 那一段与循环之后不算）

def literal_loop_candidates(tree, assignments):
    """整段代码里只被一个 `for 名字 in <元组、列表或集合字面量>:` 绑定过的名字：交回 {名字: LoopCandidates}。
    名字另有绑定、for 的目标不是单个名字、元素带星号或有一个照 python_literal_path 算不出的，不认（循环变量照旧算不出）；
    range、推导式、变量里的列表这些不是字面量的可迭代对象同样不认；推导式自己的 for 子句与 lambda 的参数不是 ast.For，不看。
    循环体里的条件（if、continue、break、try）不看，体里每个调用都按每个候选判。弄坏开关 python-loop-variable-unexpanded 下一个都不认。"""
    if detector_break_is_set("python-loop-variable-unexpanded"):
        return {}
    bindings, loops = name_binding_counts(tree), {}
    for node in ast.walk(tree):
        if not (isinstance(node, ast.For) and isinstance(node.target, ast.Name) and isinstance(node.iter, (ast.Tuple, ast.List, ast.Set))
                and bindings.get(node.target.id) == 1 and node.iter.elts):
            continue
        paths = [None if isinstance(element, ast.Starred) else python_literal_path(element, assignments) for element in node.iter.elts]
        if None not in paths:
            loops[node.target.id] = LoopCandidates(tuple(paths), frozenset(id(item) for statement in node.body for item in ast.walk(statement)))
    return loops

def python_literal_path(node, assignments, depth=0, bound=None):
    """路径表达式在这一刻算得出的值：字符串字面量、只赋过一次的名字、Path(…)、Path(…) / '…'、os.path.join(…)；算不出交 None。
    bound 是这一次代入的循环变量 {名字: 路径}（python_path_choices 给），名字照它取值。"""
    if node is None or depth > PYTHON_PATH_MAXIMUM_DEPTH:
        return None
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.Name) and bound and node.id in bound:
        return bound[node.id]
    if isinstance(node, ast.Name) and node.id in assignments:
        return python_literal_path(assignments[node.id], assignments, depth + 1, bound)
    if isinstance(node, ast.Call) and node.args and not node.keywords:
        name, owner = called_name_and_owner(node.func)
        if name in PYTHON_PATH_CONSTRUCTORS or (name == "join" and owner == "path"):
            parts = [python_literal_path(argument, assignments, depth + 1, bound) for argument in node.args]
            return None if None in parts else os.path.join(*parts)
    if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
        left, right = python_literal_path(node.left, assignments, depth + 1, bound), python_literal_path(node.right, assignments, depth + 1, bound)
        return None if left is None or right is None else os.path.join(left, right)
    return None

def referenced_loop_names(node, assignments, loops, depth=0):
    """路径表达式里用到的循环变量（literal_loop_candidates 认得的那几个），连同经只赋过一次的名字引到的，按出现次序、不重复。"""
    if node is None or depth > PYTHON_PATH_MAXIMUM_DEPTH:
        return []
    found = []
    for item in ast.walk(node):
        if not isinstance(item, ast.Name):
            continue
        nested = [item.id] if item.id in loops else referenced_loop_names(assignments.get(item.id), assignments, loops, depth + 1)
        found += [name for name in nested if name not in found]
    return found

def python_path_choices(call, expressions, assignments, loops):
    """调用 call 的几个路径表达式的取值：交回 [(各表达式算出的路径或 None, …), …]，一种取值一组。
    表达式里用到的循环变量，call 在那个 for 的循环体里的，按它的字面元素逐个代入，几个循环变量取遍组合（同一个循环变量在几个表达式里取同一个值）；
    组合多过 MAXIMUM_LOOP_LANDINGS 种的照算不出（一组全是 None）。没有这样的循环变量的，交回只有一组、照 python_literal_path 算的。"""
    names = []
    for expression in expressions:
        names += [name for name in referenced_loop_names(expression, assignments, loops) if name not in names and id(call) in loops[name].body_nodes]
    if not names:
        return [tuple(python_literal_path(expression, assignments) for expression in expressions)]
    if math.prod(len(loops[name].paths) for name in names) > MAXIMUM_LOOP_LANDINGS:
        return [tuple(None for _ in expressions)]
    return [tuple(python_literal_path(expression, assignments, bound=dict(zip(names, values))) for expression in expressions)
            for values in itertools.product(*(loops[name].paths for name in names))]

def call_argument(node, position, keyword):
    """调用的第 position 个位置参数，没有就找名为 keyword 的关键字参数；都没有交 None。"""
    if len(node.args) > position:
        return node.args[position]
    return next((item.value for item in node.keywords if item.arg == keyword), None)

def opening_mode(node, default="r"):
    """open 的模式：没给是 default，给的是字符串字面量交它，给的是别的表达式（算不出）交 None。"""
    if node is None:
        return default
    return node.value if isinstance(node, ast.Constant) and isinstance(node.value, str) else None

def mode_writes_in_place(mode):
    """'w' 截断重写、'r+' 从头改写，都在原 inode 上；'a' 追加、'x' 只建新文件，不算。"""
    return mode is not None and ("w" in mode or ("r" in mode and "+" in mode))

def os_open_flags_write(flags):
    """os.open 的 flags 表达式要写：里面有 O_WRONLY 或 O_RDWR，或整数字面量的低两位不是 0。"""
    if isinstance(flags, ast.Constant) and isinstance(flags.value, int) and not isinstance(flags.value, bool):
        return flags.value & 3 != 0
    names = {item.attr if isinstance(item, ast.Attribute) else item.id for item in ast.walk(flags) if isinstance(item, (ast.Attribute, ast.Name))}
    return bool(names & {"O_WRONLY", "O_RDWR"})

def python_service_writes(node, name, owner, receiver, choices):
    """⑭ 另认的 python 写法（一个调用节点）：os.remove / os.unlink / Path(…).unlink / shutil.rmtree，os.rename / os.replace / shutil.move 的源（都是 holding：落点或包着已有服务源码的目录就拒；
    弄坏开关 service-source-python-delete-unseen 下不认），带写标志的 os.open、shutil.copytree(…, dirs_exist_ok=True) 往已有目录逐份盖（弄坏开关 service-source-python-in-place-unseen 下不认）。
    choices(表达式…) 交这个调用里几个路径表达式的每一种取值（python_path_choices），每一种取值各交一份写。"""
    writes = []
    if not detector_break_is_set("service-source-python-delete-unseen"):
        if (name in ("remove", "unlink") and owner == "os") or (name == "rmtree" and owner == "shutil"):
            writes += [PythonWrite(f"python {owner}.{name}", path, None, "holding") for (path,) in choices(call_argument(node, 0, "path")) if path is not None]
        elif name == "unlink" and receiver is not None:
            writes += [PythonWrite("python Path.unlink", path, None, "holding") for (path,) in choices(receiver) if path is not None]
        elif (name in ("rename", "replace") and owner == "os") or (name == "move" and owner == "shutil"):
            writes += [PythonWrite(f"python {owner}.{name} 挪走", path, None, "holding") for (path,) in choices(call_argument(node, 0, "src")) if path is not None]
    if not detector_break_is_set("service-source-python-in-place-unseen"):
        if name == "open" and owner == "os":
            flags = call_argument(node, 1, "flags")
            if flags is not None and os_open_flags_write(flags):
                writes += [PythonWrite("python os.open(…, 写标志)", path, None) for (path,) in choices(call_argument(node, 0, "path")) if path is not None]
        elif name == "copytree" and owner == "shutil":
            existing_allowed = next((item.value for item in node.keywords if item.arg == "dirs_exist_ok"), None)
            if isinstance(existing_allowed, ast.Constant) and existing_allowed.value is True:
                writes += [PythonWrite("python shutil.copytree(…, dirs_exist_ok=True)", destination, source, "directory-copy")
                           for source, destination in choices(call_argument(node, 0, "src"), call_argument(node, 1, "dst"))
                           if source is not None and destination is not None]
    return writes

def python_in_place_writes(code, every_write=False, service_writes=False):
    """python 代码里在原 inode 上写文件的调用：open(路径, 带 w 或 r+ 的模式)、Path(路径).open(同上)、Path(路径).write_text / write_bytes、
    shutil.copyfile / copy / copy2 的目标。路径与模式算不出的不判，解析不了的代码不判。
    路径里用到 `for 名字 in <字面元组、列表或集合>:` 的循环变量、调用又在那个循环体里的，按每个元素各交一份写（literal_loop_candidates、python_path_choices）。
    every_write（⑨ 用）另交带 a、x、+ 的模式与换 inode 的写：shutil.move、os.replace、os.rename 的目标。
    service_writes（⑭ 用）另交 python_service_writes 那几种。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    assignments = single_assignments(tree)
    loops = literal_loop_candidates(tree, assignments)
    writes = []
    mode_writes = (lambda mode: mode is not None and any(letter in mode for letter in "wax+")) if every_write else mode_writes_in_place
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        def choices(*expressions, call=node):
            return python_path_choices(call, expressions, assignments, loops)
        name, owner = called_name_and_owner(node.func)
        receiver = node.func.value if isinstance(node.func, ast.Attribute) else None
        if name == "open" and (receiver is None or (isinstance(receiver, ast.Name) and receiver.id in PYTHON_OPEN_MODULES)):
            mode = opening_mode(call_argument(node, 1, "mode"))
            if mode_writes(mode):
                writes += [PythonWrite(f"python open(…, '{mode}')", path, None) for (path,) in choices(call_argument(node, 0, "file")) if path is not None]
        elif name == "open":
            mode = opening_mode(call_argument(node, 0, "mode"))
            if mode_writes(mode):
                writes += [PythonWrite(f"python Path.open('{mode}')", path, None) for (path,) in choices(receiver) if path is not None]
        elif name in PYTHON_PATH_WRITERS and receiver is not None:
            writes += [PythonWrite(f"python Path.{name}", path, None) for (path,) in choices(receiver) if path is not None]
        elif name in PYTHON_FILE_COPIERS and owner == "shutil":
            source_expression = call_argument(node, 0, "src") if name != "copyfile" else None
            writes += [PythonWrite(f"python shutil.{name}", path, source)
                       for path, source in choices(call_argument(node, 1, "dst"), source_expression) if path is not None]
        elif every_write and ((name == "move" and owner == "shutil") or (name in ("replace", "rename") and owner == "os")):
            writes += [PythonWrite(f"python {owner}.{name}", path, source)
                       for path, source in choices(call_argument(node, 1, "dst"), call_argument(node, 0, "src")) if path is not None]
        if service_writes:
            writes += python_service_writes(node, name, owner, receiver, choices)
    return writes

def is_existing_script(path, repository_root, scratch_root):
    """path 是仓里或 scratch_root（一个根，或几个根的元组）下一个已经存在的脚本：.sh、.py，或带执行位的普通文件（符号链接按它指向的文件算）。
    弄坏开关 in-place-script-session-temporary-root-unseen 下会话临时目录根不算。"""
    roots = (scratch_root,) if isinstance(scratch_root, str) else tuple(scratch_root)
    if work_roots and detector_break_is_set("in-place-script-session-temporary-root-unseen"):
        roots = tuple(root for root in roots if root != work_roots.claude_session_temporary_root())
    if not (is_inside(path, repository_root) or any(is_inside(path, root) for root in roots)):
        return False
    real = os.path.realpath(path)
    if not os.path.isfile(real):
        return False
    return path.endswith(SCRIPT_SUFFIXES) or real.endswith(SCRIPT_SUFFIXES) or bool(os.stat(real).st_mode & 0o111)

def script_in_place_verdict(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOTS):
    """整条命令里在原 inode 上写一个已有脚本的每一步，交回 `写法 路径`（仓里的写相对仓库根的路径）。目标这一刻算不出（没赋过值的变量、
    命令替换、通配）的不判；同一条命令里排在前面被 mv 挪走或 rm 掉的那个名字，之后写的是一个新 inode，不算。"""
    replaced_names, refused = [], []
    for step in overwrite_steps(command, repository_root):
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PreservingStep):
            if step.kind in ("mv", "rm"):
                replaced_names += preserved_paths(step, base)
            continue
        targets = []
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code):
                path = shell_words.resolve_path(base, write.path)
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                targets.append((write.form, path))
        elif step.form not in INODE_REPLACING_FORMS and not step.replaces_inode:
            for word in destination_words(step, base):
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                if not has_unknown_piece and not has_glob:
                    targets.append((step.form, shell_words.resolve_path(base, pattern)))
        for form, path in targets:
            if not is_preserved(path, replaced_names) and is_existing_script(path, repository_root, scratch_root):
                refused.append(f"{form} {os.path.relpath(path, repository_root) if is_inside(path, repository_root) else path}")
    return list(dict.fromkeys(refused))

def script_in_place_refusal(command, repository_root, scratch_root=SCRIPT_SCRATCH_ROOTS):
    """交回要拒的写法与目标；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-script-in-place-write")):
        return []
    return script_in_place_verdict(command, repository_root, scratch_root)

# ⑥ 终止进程只许点名一个自己起的进程号或任务号；同一份判定也交给门禁 code-tooling 的 research-gate-lint 格扫脚本（--scan-scripts），hook 与门禁不各写一份
BY_NAME_KILLERS = {"pkill", "killall", "killall5", "skill"}           # 按名字挑进程：一次命中一批，谁起的都算
FAN_OUT_RUNNERS = {"xargs", "parallel"}                                # 把一串进程号逐个喂给后面那条命令
FIND_EXECUTING_OPTIONS = {"-exec", "-execdir", "-ok", "-okdir"}
TERMINATING_NAMES = {"kill", "proc.py"} | BY_NAME_KILLERS
MACHINE_STOPPING_COMMANDS = {"reboot", "poweroff", "halt", "shutdown", "telinit"}
SYSTEMCTL_STOPPING_VERBS = {"stop", "kill", "restart", "try-restart", "reload-or-restart", "try-reload-or-restart",
                            "condrestart", "force-reload", "freeze", "set-property", "clean"}
SYSTEMCTL_STOPPING_WITH_NOW = {"disable", "mask"}                      # 带 --now 时顺手把单元停掉
SYSTEMCTL_MACHINE_VERBS = {"isolate", "poweroff", "reboot", "halt", "kexec", "suspend", "hibernate", "hybrid-sleep",
                           "suspend-then-hibernate", "rescue", "emergency", "default", "soft-reboot", "exit", "switch-root"}
SYSTEMCTL_OPTIONS_WITH_VALUE = {"-t", "--type", "-s", "--signal", "-p", "--property", "-P", "-H", "--host", "-M", "--machine",
                                "-n", "--lines", "-o", "--output", "--kill-whom", "--kill-value", "--job-mode", "--root", "--image",
                                "--state", "--what", "--timestamp", "--message", "--reboot-argument", "--kill-mode", "--drop-in", "--when"}
# SSH 会话、登录会话、用户级 systemd 与它们依赖的系统服务；本地模型服务的单元另从私有配置读（private_words）；systemctl 不写后缀的单元按 .service 补上再比
PROTECTED_UNIT = re.compile(r"^(?:sshd?(?:@[^/]*)?(?:\.service|\.socket)|session-[^/]*\.scope|user@[^/]*\.service|user(?:-[^/]*)?\.slice"
                            r"|-\.slice|init\.scope|systemd-logind\.service|dbus(?:-broker)?\.(?:service|socket))$")

def private_config_path():
    configured = os.environ.get("BASH_COMMAND_DETECTOR_PRIVATE_CONFIG")
    return configured or os.path.join(os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config"),
                                      "singlefs", "bash-command-detector.env")

def private_words(key):
    """私有配置里 key 的值按空白与逗号切开的词；文件不在、读不了、没有这个键、弄坏开关设着，都交回空表（那一项不判、不报错）。"""
    if detector_break_is_set("ignore-private-config"):
        return []
    try:
        with open(private_config_path(), encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError):
        return []
    words = []
    for line in lines:
        name, separator, value = line.strip().partition("=")
        if line.strip().startswith("#") or not separator or name.strip() != key:
            continue
        words += [word for word in re.split(r"[\s,]+", value.strip().strip("'\"")) if word]
    return words

def unit_with_suffix(unit):
    return unit if "." in os.path.basename(unit) else unit + ".service"
LOGINCTL_TERMINATING_VERB = re.compile(r"^(?:terminate|kill)-")
CGROUP_CONTROL_FILES = {"cgroup.kill", "cgroup.freeze"}               # 写进去一次停掉（冻住）那个 cgroup 里的全部进程
PROCESS_LIST_READING = re.compile(r"cgroup\.(?:procs|threads)|/proc/(?:\*|\[)|\b(?:ls|find)\s+/proc(?:/|\s|$)")
PROCESS_PICKERS = {"pgrep", "pidof"}
PROTECTED_PROCESS_NAMES = {"systemd", "init", "sshd", "sshd-session", "sshd-auth", "dbus-daemon", "dbus-broker", "systemd-logind", "claude"}
EXPANSION = re.compile(r"\$\{[^}]*\}")
WHOLE_LIST_EXPANSION = re.compile(r"\$\{?[@*]|\$\{[^}]*\[[@*]\][^}]*\}")
BRACE_LIST = re.compile(r"\{[^{}]*(?:,|\.\.)[^{}]*\}")
LITERAL_PROCESS_ID = re.compile(r"^\d+$")
SIGNAL_ZERO_NAMES = {"0", "SIG0"}

class TerminationFinding(NamedTuple):
    words: tuple              # 认出它的那条简单命令（引号去掉）；python 代码里的是空
    problem: str              # 给人看的一句：哪种写法、为什么打得到别人
    kind: str                 # "loop"（循环里逐个发）、"cgroup"（按 cgroup / 进程表挑）、"other"；脚本里的 own-scope 标注只放行前两类
    python_code: str = ""     # python 代码里认出的：那段代码（脚本扫描按它在文件里的位置换算行号）
    python_line: int = 0      # python 代码里认出的：它在那段代码里的行号

def innermost_call(words):
    """一条简单命令剥开前缀、capped.sh N / run-with-memory-cap.sh <上限>、`bash 包装脚本`、builtin、python 解释器之后：交回 (名字, 参数)。
    python -c 那一条交回 python 本身与全部参数，代码由 python 那一侧判。"""
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    for _ in range(shell_words.MAXIMUM_NESTING):
        if name in ("builtin", "command") and arguments:
            name, arguments = os.path.basename(arguments[0]), arguments[1:]
            continue
        if name and shell_words.PYTHON_NAME.match(name):
            index = 0
            while index < len(arguments) and arguments[index].startswith("-") and arguments[index] != "-":
                if arguments[index] == "-c":
                    return name, arguments
                index += 2 if arguments[index] in ("-m", "-W", "-X") else 1
            if index < len(arguments) and arguments[index] != "-":
                name, arguments = os.path.basename(arguments[index]), arguments[index + 1:]
                continue
        break
    return name, arguments

def kill_arguments(arguments):
    """kill 的参数：交回 (信号, 目标)；只列信号（-l、-L）时交 (None, None)。信号给过之后再出现的 -N 是负的进程号（进程组）。"""
    signal_name, targets, index, options_done = None, [], 0, False
    while index < len(arguments):
        argument = arguments[index]
        if options_done or not argument.startswith("-") or argument == "-":
            targets.append(argument)
        elif argument == "--":
            options_done = True
        elif argument in ("-l", "-L", "--list", "--table"):
            return None, None
        elif argument in ("-s", "-n", "--signal"):
            signal_name = arguments[index + 1] if index + 1 < len(arguments) else ""
            index += 1
        elif argument.startswith("--signal="):
            signal_name = argument.split("=", 1)[1]
        elif argument in ("-q", "--queue"):
            index += 1
        elif argument == "--timeout":
            index += 2
        elif signal_name is None:
            signal_name = argument[1:]
        else:
            targets.append(argument)
        index += 1
    return signal_name, targets

def target_problem(target):
    """一个 kill / proc.py stop 的目标打得到不止一个进程、或打到谁这一刻说不清，就交回原因；点名一个的交 None。"""
    if target.startswith("-") and target != "-":
        return f"目标 {target} 带负号，是整个进程组（-1 是能发到的全部进程）"
    if target == "0":
        return "目标 0 是发这条命令的整个进程组"
    if target in ("$PPID", "${PPID}"):
        return "目标 $PPID 是起这条命令的进程（在会话里就是 Claude 会话本身）"
    if "$(" in target or "`" in target or "<(" in target:
        return f"目标 {target} 是命令替换：展开出几个进程、是谁起的，这一刻都不知道"
    if WHOLE_LIST_EXPANSION.search(target):
        return f"目标 {target} 是数组或位置参数的整批展开"
    bare = EXPANSION.sub("", target)
    if any(character in bare for character in "*?[") or BRACE_LIST.search(bare):
        return f"目标 {target} 里有通配或花括号展开"
    return None

def read_process_table():
    """/proc 下每个进程的父进程号、名字与命令行（只读，不发信号）：判写死的进程号打到的是谁。"""
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", encoding="utf-8", errors="replace") as handle:
                status = handle.read()
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                command_line = handle.read().replace(b"\0", b" ").decode("utf-8", "replace").strip()
            table[int(entry)] = {"parent": int(status.rsplit(")", 1)[1].split()[1]),
                                 "name": status[status.index("(") + 1:status.rindex(")")], "command_line": command_line}
        except (OSError, ValueError, IndexError):
            continue
    return table

def ancestors_in(table, process_id):
    """process_id 与它在表里的全部祖先。"""
    found, current = [], process_id
    while current and current in table and current not in found:
        found.append(current)
        current = table[current]["parent"]
    return found

class ProcessTableOnDemand:
    """进程表与发这条命令的会话自己的祖先：第一次有人要才读（判 kill / proc.py stop 的写死的进程号打到谁时才要），读过的不再读。
    process_target_findings 认它：有它就按需取，是字典就照旧用字典与 own_ancestors。"""
    def __init__(self, reader, own_process_id=None):
        self.reader, self.own_process_id, self.loaded = reader, own_process_id, None

    def load(self):
        """→ (进程表, 这个会话自己的祖先)。"""
        if self.loaded is None:
            table = self.reader()
            self.loaded = (table, ancestors_in(table, self.own_process_id or os.getpid()))
        return self.loaded

def protected_process_reason(process_id, table, own_ancestors):
    """写死的进程号打到的是 SSH、VSCode、Claude 会话、本地模型服务、系统服务，或别的 Claude 会话起的进程，就交回原因。不在表里的不拦（kill 自己会报错）。"""
    information = table.get(process_id)
    if information is None:
        return None
    label = f"进程 {process_id}（{information['name']}）"
    if process_id in own_ancestors:
        return f"{label}是发这条命令的会话自己的祖先（Claude 会话、VSCode 扩展宿主或更上层）"
    if process_id == 1 or information["name"] in PROTECTED_PROCESS_NAMES:
        return f"{label}是系统、SSH 或 Claude 会话的进程"
    if ".vscode-server" in information["command_line"]:
        return f"{label}是 VSCode 服务端里的进程（服务端、扩展宿主、ptyHost 或经它起的 Claude）"
    if any(marker in information["command_line"] for marker in private_words("LOCAL_MODEL_SERVICE_PROCESS_MARKERS")):
        return f"{label}是本地模型服务的进程（命令行里有私有配置登记的标记）"
    own_sessions = {ancestor for ancestor in own_ancestors if table.get(ancestor, {}).get("name") == "claude"}
    for ancestor in ancestors_in(table, information["parent"]):
        if table[ancestor]["name"] == "claude" and ancestor not in own_sessions:
            return f"{label}是另一个 Claude 会话（进程 {ancestor}）起的"
    return None

def process_target_findings(words, targets, in_loop, what, process_table, own_ancestors, own_group_variables=frozenset()):
    """kill 与 proc.py stop 共用：目标个数、每个目标的形状、写死的进程号打到谁、是不是在循环里。
    own_group_variables 是这一层里记着「自己用 setsid 起的后台任务的进程号」的变量名（scan_shell_text 记）：`-$名字` 打的是那一组，不算带负号。"""
    findings = []
    if len(targets) > 1:
        findings.append(TerminationFinding(tuple(words), f"{what} 一次给了 {len(targets)} 个目标（{' '.join(targets)}）：一次只停点名的一个", "other"))
    for target in targets:
        own_group = OWN_GROUP_TARGET.match(target)
        if own_group and own_group.group(1) in own_group_variables and not detector_break_is_set("own-session-group-unrecognized"):
            continue
        problem = target_problem(target)
        if problem:
            kind = "cgroup" if PROCESS_LIST_READING.search(target) or any(picker in target for picker in PROCESS_PICKERS) else "other"
            findings.append(TerminationFinding(tuple(words), f"{what}：{problem}", kind))
        elif process_table is not None and LITERAL_PROCESS_ID.match(target):
            table, ancestors = process_table.load() if isinstance(process_table, ProcessTableOnDemand) else (process_table, own_ancestors)
            reason = protected_process_reason(int(target), table, ancestors)
            if reason:
                findings.append(TerminationFinding(tuple(words), f"{what}：{reason}", "other"))
    if in_loop:
        findings.append(TerminationFinding(tuple(words), f"{what} 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个", "loop"))
    return findings

def systemctl_findings(words, arguments):
    options, positional, index, options_done = [], [], 0, False
    while index < len(arguments):
        argument = arguments[index]
        if not options_done and argument == "--":
            options_done = True
        elif not options_done and argument.startswith("-") and argument != "-":
            options.append(argument)
            index += 1 if argument in SYSTEMCTL_OPTIONS_WITH_VALUE else 0
        else:
            positional.append(argument)
        index += 1
    if not positional:
        return []
    verb, units = positional[0], positional[1:]
    if verb in SYSTEMCTL_MACHINE_VERBS:
        return [TerminationFinding(tuple(words), f"systemctl {verb} 停的是整台机器或整个服务管理器上的东西（SSH 会话、VSCode、Claude 会话都在里面）", "other")]
    if not (verb in SYSTEMCTL_STOPPING_VERBS or (verb in SYSTEMCTL_STOPPING_WITH_NOW and "--now" in options)):
        return []
    findings = []
    private_units = {unit_with_suffix(unit) for unit in private_words("LOCAL_MODEL_SERVICE_UNITS")}
    for unit in units[:1] if verb == "set-property" else units:
        bare = EXPANSION.sub("", unit)
        named = unit_with_suffix(unit)
        if "$(" in unit or "`" in unit or WHOLE_LIST_EXPANSION.search(unit):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 是命令替换或整批展开：一次停一批，是谁的这一刻不知道", "other"))
        elif any(character in bare for character in "*?[") or BRACE_LIST.search(bare):
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} 的单元 {unit} 带通配：一次命中一批单元，别的会话的也算", "other"))
        elif PROTECTED_UNIT.match(named) or named in private_units:
            findings.append(TerminationFinding(tuple(words), f"systemctl {verb} {unit}：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务", "other"))
    return findings

def call_findings(words, in_loop, process_table, own_ancestors, own_group_variables=frozenset()):
    """一条简单命令（剥开包装之后）里的终止写法；python -c 的代码另判。交回 (findings, 这一条是不是在给进程发终止信号)。
    own_group_variables 只给 kill 用（process_target_findings）：proc.py stop 一次停一个进程，不认进程组。"""
    name, arguments = innermost_call(words)
    if not name:
        return [], False
    if name == "kill":
        signal_name, targets = kill_arguments(arguments)
        if targets is None or signal_name in SIGNAL_ZERO_NAMES or not targets:
            return [], False
        return process_target_findings(words, targets, in_loop, "kill", process_table, own_ancestors, own_group_variables), True
    if name == "proc.py" and arguments[:1] == ["stop"]:
        targets, index = [], 1
        while index < len(arguments):
            if arguments[index] == "--grace":
                index += 1
            elif not arguments[index].startswith("--grace="):
                targets.append(arguments[index])
            index += 1
        return process_target_findings(words, targets, in_loop, "proc.py stop", process_table, own_ancestors), True
    if name in BY_NAME_KILLERS:
        return [TerminationFinding(tuple(words), f"{name} 按名字挑进程发信号：一次命中一批，别的会话起的同名进程也算", "other")], True
    if name in FAN_OUT_RUNNERS and any(os.path.basename(argument) in TERMINATING_NAMES for argument in arguments):
        return [TerminationFinding(tuple(words), f"{name} 把一串进程号逐个喂给 kill / proc.py：这是批量停", "loop")], True
    if name == "find" and any(argument in FIND_EXECUTING_OPTIONS and os.path.basename(following) in TERMINATING_NAMES
                              for argument, following in zip(arguments, arguments[1:])):
        return [TerminationFinding(tuple(words), "find -exec 把找到的每个进程都停掉：这是批量停", "loop")], True
    if name == "fuser" and any(argument == "--kill" or (argument.startswith("-") and not argument.startswith("--") and "k" in argument[1:])
                               for argument in arguments):
        return [TerminationFinding(tuple(words), "fuser -k 停掉用着那个文件或端口的全部进程，是谁的都算", "other")], True
    if name == "systemctl":
        return systemctl_findings(words, arguments), False
    if name == "loginctl" and any(LOGINCTL_TERMINATING_VERB.match(argument) for argument in arguments if not argument.startswith("-")):
        return [TerminationFinding(tuple(words), "loginctl terminate-* / kill-* 停掉整个登录会话或用户（SSH 会话就在里面）", "other")], False
    if name in MACHINE_STOPPING_COMMANDS and not (name == "shutdown" and "-c" in arguments):
        return [TerminationFinding(tuple(words), f"{name} 停的是整台机器", "other")], False
    if name == "tee" and any(os.path.basename(argument) in CGROUP_CONTROL_FILES for argument in arguments):
        return [TerminationFinding(tuple(words), "往 cgroup.kill / cgroup.freeze 写：一次停掉（冻住）那个 cgroup 里的全部进程", "cgroup")], True
    return [], False

def embedded_code_strings(words):
    """一条简单命令里任何位置上的 `bash -c '…'` 与 `python3 -c '…'`（前面是 shell 函数、变量里的脚本、别的包装也算）：交回 [(种类, 代码)]。"""
    found = []
    for index, word in enumerate(words):
        name, rest = os.path.basename(word), words[index + 1:]
        if name in shell_words.SHELL_NAMES:
            code = shell_words.shell_invocation(rest).code_string
            if code is not None:
                found.append(("shell", code))
        elif shell_words.PYTHON_NAME.match(name) and "-c" in rest:
            position = rest.index("-c")
            if position + 1 < len(rest) and all(item.startswith("-") for item in rest[:position]):
                found.append(("python", rest[position + 1]))
    return found

def python_heredoc_bodies(text):
    """喂给 python 的 heredoc 正文（python3 - <<EOF、python3 /dev/fd/3 3<<'PY' 这类）。"""
    return [heredoc.text for command in shell_words.commands_at_command_position(text).commands
            if command.launcher == "python" or shell_words.PYTHON_NAME.match(os.path.basename(command.words[0]))
            for heredoc in command.standard_input_heredocs]

PYTHON_SIGNAL_CALLS = {"kill", "killpg", "pidfd_send_signal"}
PYTHON_LOOP_NODES = (ast.For, ast.AsyncFor, ast.While, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)

# 放行一种进程组：自己用新会话起的那个子进程的组（上游 .claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」：
#   每条样本 start_new_session=True 起、超时与结束时对整组发信号）。认得的写法：
#   - 子进程：一个名字，它在自己所在的作用域里每一处绑定都是带 start_new_session=True（或 process_group=0）的 subprocess.Popen、
#     asyncio.create_subprocess_exec / create_subprocess_shell（带 await 的也算），或 `with <那样的调用> as 名字`，
#     或调同一份代码里的一个函数、那个函数（同样是每一处绑定都是这一个 def）自己这一层的每条 return 都交回那样起的子进程；
#   - 进程组号：<子进程>.pid、os.getpgid(<子进程>.pid)，或一个每一处绑定都是这两样之一的名字；
#   - os.killpg(<进程组号>, …) 与 os.kill(-<进程组号>, …) 放行，在循环里调也放行（发的是那一个子进程的组）。
#   照旧拒：参数、循环变量、属性（self.process.pid）、改绑过的名字、0、负的写死号、os.getpgid(0)、从别处读来的进程号；
#   同一个函数里读 cgroup.procs 或 /proc 的那一条另判，目标是自己起的也照报。
SESSION_CHILD_CALLS = {"Popen", "create_subprocess_exec", "create_subprocess_shell"}
PYTHON_SCOPE_NODES = (ast.Module, ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef,
                      ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)
OWN_SESSION_RESOLUTION_DEPTH = 6   # 名字套名字、函数套函数往下追的层数上限；绕回自己的写法追到这里就当认不出

def python_scope_bindings(scope):
    """一个作用域（模块、函数、lambda、类体、推导式）自己这一层的名字绑定，按源码次序：{名字: [(种类, 节点)]}；
    种类 value（赋进去的值节点）、def（函数定义节点）、other（参数、循环变量、import、except、global 这类说不出值的）。
    不进嵌套的作用域；推导式里的海象赋值绑在外层，记成 other。"""
    bindings = {}
    def bind(name, kind, node):
        bindings.setdefault(name, []).append((kind, node))
    def bind_target(target, value):
        if isinstance(target, ast.Name):
            bind(target.id, "value" if value is not None else "other", value)
        elif isinstance(target, (ast.Tuple, ast.List)):
            for element in target.elts:
                bind_target(element, None)
        elif isinstance(target, ast.Starred):
            bind_target(target.value, None)
    def walk(node):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            bind(node.name, "def", node)
            return
        if isinstance(node, ast.ClassDef):
            bind(node.name, "other", None)
            return
        if isinstance(node, ast.Lambda):
            return
        if isinstance(node, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
            for inner in ast.walk(node):
                if isinstance(inner, ast.NamedExpr) and isinstance(inner.target, ast.Name):
                    bind(inner.target.id, "other", None)
            return
        if isinstance(node, ast.Assign):
            for target in node.targets:
                bind_target(target, node.value)
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            bind_target(node.target, node.value)
        elif isinstance(node, ast.AugAssign):
            bind_target(node.target, None)
        elif isinstance(node, ast.NamedExpr) and isinstance(node.target, ast.Name):
            bind(node.target.id, "value", node.value)
        elif isinstance(node, (ast.For, ast.AsyncFor)):
            bind_target(node.target, None)
        elif isinstance(node, (ast.With, ast.AsyncWith)):
            for item in node.items:
                if isinstance(item.optional_vars, ast.Name):
                    bind(item.optional_vars.id, "value", item.context_expr)
                elif item.optional_vars is not None:
                    bind_target(item.optional_vars, None)
        elif isinstance(node, ast.ExceptHandler) and node.name:
            bind(node.name, "other", None)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                bind((alias.asname or alias.name).split(".")[0], "other", None)
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            for name in node.names:
                bind(name, "other", None)
        elif isinstance(node, ast.Delete):
            for target in node.targets:
                bind_target(target, None)
        elif type(node).__name__ in ("MatchAs", "MatchStar") and getattr(node, "name", None):
            bind(node.name, "other", None)
        elif type(node).__name__ == "MatchMapping" and getattr(node, "rest", None):
            bind(node.rest, "other", None)
        for child in ast.iter_child_nodes(node):
            walk(child)
    if isinstance(scope, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
        arguments = scope.args
        for argument in [*arguments.posonlyargs, *arguments.args, *arguments.kwonlyargs, arguments.vararg, arguments.kwarg]:
            if argument is not None:
                bind(argument.arg, "other", None)
        for statement in (scope.body if isinstance(scope.body, list) else [scope.body]):
            walk(statement)
    elif isinstance(scope, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
        for generator in scope.generators:
            bind_target(generator.target, None)
    else:
        for statement in scope.body:
            walk(statement)
    return bindings

def function_returns(definition):
    """函数自己这一层的 return 交回的表达式（裸 return 交 None）；lambda 交它的式子；生成器（有 yield）交 []。"""
    if isinstance(definition, ast.Lambda):
        return [definition.body]
    values, generator = [], False
    def walk(node):
        nonlocal generator
        for child in ast.iter_child_nodes(node):
            if isinstance(child, PYTHON_SCOPE_NODES):
                continue
            if isinstance(child, ast.Return):
                values.append(child.value)
            elif isinstance(child, (ast.Yield, ast.YieldFrom)):
                generator = True
            walk(child)
    walk(definition)
    return [] if generator else values

class OwnSessionGroups:
    """一段 python 代码里认「自己用新会话起的子进程的进程组」（上面 SESSION_CHILD_CALLS 那段注释）。scope_chain 是从模块到当前作用域的节点列表。
    弄坏开关：own-session-group-unrecognized 一样都不认；own-session-group-session-unchecked 不看 start_new_session 与 process_group，
    凡 Popen 这一类起的都认；own-session-group-first-binding-only 名字只看第一处绑定，后面改绑的看不见。"""
    def __init__(self):
        self.bindings_by_scope = {}

    def bindings(self, scope):
        if id(scope) not in self.bindings_by_scope:
            self.bindings_by_scope[id(scope)] = python_scope_bindings(scope)
        return self.bindings_by_scope[id(scope)]

    def resolve(self, name, scope_chain):
        """名字绑在哪一层：交 (到那一层为止的作用域链, 它在那一层的全部绑定)；哪一层都没绑（内置、别处来的）交 None。类体里绑的只在类体自己里看得见。"""
        innermost = len(scope_chain) - 1
        for index in range(innermost, -1, -1):
            scope = scope_chain[index]
            if isinstance(scope, ast.ClassDef) and index != innermost:
                continue
            entries = self.bindings(scope).get(name)
            if entries:
                if detector_break_is_set("own-session-group-first-binding-only"):
                    entries = entries[:1]
                return scope_chain[:index + 1], entries
        return None

    @staticmethod
    def starts_own_session(call):
        function = call.func
        name = function.attr if isinstance(function, ast.Attribute) else function.id if isinstance(function, ast.Name) else None
        if name not in SESSION_CHILD_CALLS:
            return False
        if detector_break_is_set("own-session-group-session-unchecked"):
            return True
        return any(isinstance(keyword.value, ast.Constant) and (
                       (keyword.arg == "start_new_session" and keyword.value.value is True)
                       or (keyword.arg == "process_group" and type(keyword.value.value) is int and keyword.value.value == 0))
                   for keyword in call.keywords)

    def is_own_child(self, node, scope_chain, depth=0):
        """node 这个表达式的值是自己用新会话起的子进程。"""
        if depth > OWN_SESSION_RESOLUTION_DEPTH:
            return False
        if isinstance(node, ast.Await):
            node = node.value
        if isinstance(node, ast.Name):
            found = self.resolve(node.id, scope_chain)
            return found is not None and all(kind == "value" and self.is_own_child(value, found[0], depth + 1) for kind, value in found[1])
        if not isinstance(node, ast.Call):
            return False
        if self.starts_own_session(node):
            return True
        if not isinstance(node.func, ast.Name):
            return False
        found = self.resolve(node.func.id, scope_chain)
        if found is None or not all(kind == "def" for kind, _ in found[1]):
            return False
        for _, definition in found[1]:
            returns = function_returns(definition)
            if not returns or not all(value is not None and self.is_own_child(value, found[0] + [definition], depth + 1) for value in returns):
                return False
        return True

    def is_own_group(self, node, scope_chain, through_names=True):
        """node 是自己用新会话起的子进程的进程组号：<子进程>.pid、os.getpgid(<子进程>.pid)，或每一处绑定都是这两样之一的名字。"""
        if detector_break_is_set("own-session-group-unrecognized"):
            return False
        if isinstance(node, ast.Attribute) and node.attr == "pid":
            return self.is_own_child(node.value, scope_chain)
        if isinstance(node, ast.Call) and len(node.args) == 1 and not node.keywords:
            function = node.func
            asks_group = ((isinstance(function, ast.Attribute) and function.attr == "getpgid" and isinstance(function.value, ast.Name) and function.value.id == "os")
                          or (isinstance(function, ast.Name) and function.id == "getpgid"))
            argument = node.args[0]
            return asks_group and isinstance(argument, ast.Attribute) and argument.attr == "pid" and self.is_own_child(argument.value, scope_chain)
        if isinstance(node, ast.Name) and through_names:
            found = self.resolve(node.id, scope_chain)
            return found is not None and all(kind == "value" and self.is_own_group(value, found[0], through_names=False) for kind, value in found[1])
        return False

def python_termination_findings(code):
    """python 代码里的终止写法：os.killpg、os.kill 的目标是 0 或负数、在循环或推导式里调、同一个函数里读 cgroup.procs 或 /proc 再调。解析不了的不判。
    发给自己用新会话起的那个子进程的组的放行（OwnSessionGroups）。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    findings = []
    groups = OwnSessionGroups()
    def reads_process_list(scope):
        return any(isinstance(node, ast.Constant) and isinstance(node.value, str) and PROCESS_LIST_READING.search(node.value)
                   for node in ast.walk(scope))
    def visit(node, loop_depth, scope_reads, scope_chain):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            loop_depth, scope_reads = 0, reads_process_list(node)
        if isinstance(node, PYTHON_SCOPE_NODES) and node is not scope_chain[-1]:
            scope_chain = scope_chain + [node]
        if isinstance(node, ast.Call):
            function = node.func
            owner = function.value.id if isinstance(function, ast.Attribute) and isinstance(function.value, ast.Name) else None
            called = function.attr if isinstance(function, ast.Attribute) else function.id if isinstance(function, ast.Name) else None
            probe = len(node.args) >= 2 and isinstance(node.args[1], ast.Constant) and node.args[1].value == 0
            if called in PYTHON_SIGNAL_CALLS and owner in ("os", "signal", None) and not probe:
                first = node.args[0] if node.args else None
                negative = (isinstance(first, ast.UnaryOp) and isinstance(first.op, ast.USub)) or (
                    isinstance(first, ast.Constant) and isinstance(first.value, int) and first.value <= 0)
                own_group = first is not None and (
                    (called == "killpg" and groups.is_own_group(first, scope_chain))
                    or (called == "kill" and isinstance(first, ast.UnaryOp) and isinstance(first.op, ast.USub)
                        and groups.is_own_group(first.operand, scope_chain)))
                if called == "killpg":
                    if not own_group:
                        findings.append(TerminationFinding((), "python 里 os.killpg 发给整个进程组", "other", code, node.lineno))
                elif negative and not own_group:
                    findings.append(TerminationFinding((), "python 里 os.kill 的目标是 0 或负数（整个进程组或全部进程）", "other", code, node.lineno))
                if loop_depth and not own_group:
                    findings.append(TerminationFinding((), f"python 里在循环或推导式里调 {called}：这是批量停", "loop", code, node.lineno))
                if scope_reads:
                    findings.append(TerminationFinding((), f"python 里同一个函数先读 cgroup.procs 或 /proc 挑进程、再调 {called}", "cgroup", code, node.lineno))
        for child in ast.iter_child_nodes(node):
            visit(child, loop_depth + (1 if isinstance(node, PYTHON_LOOP_NODES) and child is not getattr(node, "iter", None) else 0), scope_reads, scope_chain)
    visit(tree, 0, reads_process_list(tree), [tree])
    return findings

# shell 这一侧放行的进程组：同一层里先用命令位置上的 setsid（不带 -f / --fork）放后台起一条（`setsid --wait <命令> &`），紧跟着 `名字=$!`
#   （前面可以挂 local、declare 这类）记下它的进程号，之后 `kill -<信号> -- -"$名字"`（`-$名字`、`-${名字}` 同）打的就是这个新会话的组。
#   按文本次序认：先 kill 后赋值的、赋值之前最近一次放后台的不是 setsid 起的、之后又被别的赋值、read、for 的循环变量、unset 改过的，照旧按带负号拒；
#   kill 0、kill -1、负的写死号、别的变量照旧拒。弄坏开关同 OwnSessionGroups 的三条：unrecognized 不认、session-unchecked 不看 setsid、
#   first-binding-only 后面的改绑看不见。
OWN_GROUP_TARGET = re.compile(r"^-\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?$")
SHELL_VARIABLE_BINDING = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(?:\[[^\]]*\])?\+?=(.*)$", re.S)
DECLARATION_BUILTINS = {"local", "declare", "typeset", "readonly", "export"}
VARIABLE_READING_BUILTINS = {"read", "unset", "mapfile", "readarray"}
LAST_BACKGROUND_JOB = ("$!", "${!}")

def setsid_started_without_fork(words):
    """这条简单命令是经命令位置上的 setsid 起的，setsid 不带 -f / --fork（带了的那一份 fork 出去，$! 就不是新会话的组长）。"""
    skipped = shell_words.skip_prefixes(words)
    if "setsid" not in skipped.prefix_names:
        return False
    end = skipped.command_position if skipped.command_position is not None else len(words)
    for position, word in enumerate(words[:end]):
        if os.path.basename(word) != "setsid":
            continue
        for option in words[position + 1:end]:
            if not option.startswith("-") or option == "--":
                break
            if option == "--fork" or (not option.startswith("--") and "f" in option[1:]):
                return False
        return True
    return False

def update_own_group_variables(words, own_group_variables, last_background_own_session):
    """一条不放后台的简单命令改了哪些变量：`名字=$!` 在最近一次放后台的是 setsid 起的时记进 own_group_variables；
    别的赋值、read、for 的循环变量、unset 把那个名字拿掉。挂在命令前面的赋值只给那一条命令，不改 shell 里的变量。"""
    position = 0
    while position < len(words) and words[position] in shell_words.COMMAND_KEYWORDS:
        position += 1
    rest = words[position:]
    if not rest:
        return
    rebound = set()
    if rest[0] == "for" and len(rest) > 1:
        rebound.add(rest[1])
    elif rest[0] in VARIABLE_READING_BUILTINS:
        rebound.update(word for word in rest[1:] if not word.startswith("-"))
    else:
        candidates = rest[1:] if rest[0] in DECLARATION_BUILTINS else rest
        if rest[0] not in DECLARATION_BUILTINS and not all(SHELL_VARIABLE_BINDING.match(word) for word in candidates):
            return
        for word in candidates:
            binding = SHELL_VARIABLE_BINDING.match(word)
            if binding is None and word.startswith("-"):
                continue
            name = binding.group(1) if binding else word
            if last_background_own_session and word in tuple(f"{name}={value}" for value in LAST_BACKGROUND_JOB):
                own_group_variables.add(name)
            else:
                rebound.add(name)
    if not detector_break_is_set("own-session-group-first-binding-only"):
        own_group_variables.difference_update(rebound)

def scan_shell_text(text, process_table, own_ancestors, depth):
    """termination_findings 的本体：交回 (发现, 有没有给进程发终止信号, 有没有 pgrep / pidof 挑进程)。
    包装认不出的 `bash -c '…'`（前面是 shell 函数、变量里的脚本）另按一段完整的代码递归判。"""
    findings, sends_signal, picks_processes, python_codes = [], False, False, []
    for layer in shell_layers(text):
        boundaries = []
        commands, _, _, redirections = shell_words.simple_commands(layer.tokens, boundaries=boundaries)
        loop_depth = 0
        own_group_variables, last_background_own_session = set(), False   # 这一层里记着自己用 setsid 起的后台任务进程号的变量（OWN_GROUP_TARGET 那段注释）
        for index, (words, command_redirections) in enumerate(zip(commands, redirections)):
            opened, closed, _ = loop_keyword_changes(words) if words else (0, 0, False)
            for operator, target in command_redirections:
                if operator in shell_words.OUTPUT_REDIRECTS and os.path.basename(target) in CGROUP_CONTROL_FILES:
                    findings.append(TerminationFinding(tuple(words), f"往 {target} 写：一次停掉（冻住）那个 cgroup 里的全部进程", "cgroup"))
            if words:
                if index + 1 < len(boundaries) and boundaries[index + 1][:1] == ("&",):
                    last_background_own_session = (setsid_started_without_fork(words)
                                                   or detector_break_is_set("own-session-group-session-unchecked"))
                else:
                    update_own_group_variables(words, own_group_variables, last_background_own_session)
                found, sends = call_findings(words, loop_depth + opened > 0, process_table, own_ancestors, frozenset(own_group_variables))
                findings += found
                sends_signal = sends_signal or sends
                picks_processes = picks_processes or innermost_call(words)[0] in PROCESS_PICKERS
                reached = shell_code_and_timeout(words)[0]
                for kind, code in embedded_code_strings(words):
                    if kind == "python":
                        python_codes.append(code)
                    elif code != reached and depth < shell_words.MAXIMUM_NESTING:
                        inner, inner_sends, inner_picks = scan_shell_text(code, process_table, own_ancestors, depth + 1)
                        findings += inner
                        sends_signal, picks_processes = sends_signal or inner_sends, picks_processes or inner_picks
            loop_depth = max(0, loop_depth + opened - closed)
    for code in dict.fromkeys(python_codes + python_heredoc_bodies(text)):
        findings += python_termination_findings(code)
    return findings, sends_signal, picks_processes

def termination_findings(text, process_table=None, own_ancestors=(), whole_command=True):
    """整段 shell 文本（一条命令，或一整个脚本文件）里的终止写法。whole_command 为真（hook 判一条命令）时另判
    「同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号」；扫脚本文件时不判这一条，靠循环与命令替换两条。"""
    findings, sends_signal, picks_processes = scan_shell_text(text, process_table, own_ancestors, 0)
    if whole_command and sends_signal and (picks_processes or PROCESS_LIST_READING.search(shell_words.strip_data_heredocs(text))):
        findings.append(TerminationFinding((), "同一条命令里先从 cgroup.procs、/proc 或 pgrep / pidof 挑进程、再发信号：挑出来的是谁的，这一刻不知道", "cgroup"))
    return findings

def process_signal_refusal(command, process_table_reader=read_process_table, own_process_id=None):
    """命令里有打得到别人的终止写法就交回每一处的原因；否则交 []。进程表只在判到 kill / proc.py stop 的写死的进程号时才读（ProcessTableOnDemand，读一次）。
    前台、run_in_background 一样拒。弄坏开关 always-read-process-table：命令里有数字就先整扫 /proc。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-process-signals")):
        return []
    process_table = ProcessTableOnDemand(process_table_reader, own_process_id)
    if detector_break_is_set("always-read-process-table") and re.search(r"(?<![\w$-])\d+\b", command):
        process_table.load()
    return list(dict.fromkeys(f"{finding.problem}（{' '.join(finding.words)[:120]}）" if finding.words else finding.problem
                              for finding in termination_findings(command, process_table, ())))

# 扫脚本（门禁 code-tooling 的 research-gate-lint 格调 --scan-scripts）：同一份判定，按文件里的行号报；own-scope 标注与待改清单
OWN_SCOPE_ANNOTATION = re.compile(r"(?<!`)#\s*process-safety:own-scope\b[ \t]*(.*)$")   # 反引号里的是说明里举的例子，不算
OWN_SCOPE_GUARD = re.compile(r"singlefs-memory-cap-\*\.scope")
OWN_SCOPE_EXEMPTABLE_KINDS = {"loop", "cgroup"}
OWN_SCOPE_GUARD_LOOKBACK_LINES = 40
MINIMUM_ANNOTATION_REASON_CHARACTERS = 6
PENDING_LIST = os.path.join(".claude", "process-safety-pending")
SCANNED_SUFFIXES = (".sh", ".py")

def normalized_shell_line(line):
    return re.sub(r"\s+", " ", line.replace('"', "").replace("'", "").replace("\\", "")).strip()

def finding_lines(finding, text, lines, used):
    """一处发现落在文件的哪几行：python 的按代码在文件里的位置换算；shell 的按那条简单命令的词在去掉引号的行里找，
    同样的词出现几次就按次序分给几处。找不到的交 [0]。"""
    if finding.python_code:
        start = text.find(finding.python_code.rstrip("\n"))
        if start < 0:
            return [0]
        return [text.count("\n", 0, start) + finding.python_line]
    normalized = [normalized_shell_line(line) for line in lines]
    for size in (len(finding.words), 2, 1):
        joined = " ".join(finding.words[:size])
        if not joined:
            continue
        hits = [number for number, line in enumerate(normalized, 1) if joined in line]
        if hits:
            key = (joined, finding.problem)
            taken = used.setdefault(key, 0)
            used[key] = taken + 1
            return [hits[min(taken, len(hits) - 1)]]
    return [0]

def scan_script_file(path, text):
    """一个脚本文件里的发现（每条带行号）与 own-scope 标注的行号。"""
    lines = text.split("\n")
    if path.endswith(".py"):
        findings = [(finding.python_line, finding) for finding in python_termination_findings(text)]
    else:
        used = {}
        findings = [(line, finding) for finding in termination_findings(text, whole_command=False)
                    for line in finding_lines(finding, text, lines, used)]
    return findings, own_scope_annotations(path, text, lines), lines

def python_comment_annotations(code, first_line=1):
    """python 代码里写在注释（COMMENT 记号）里的 own-scope 标注：{文件行号: 理由}；字符串里写的不算。切不了记号的不认。"""
    found = {}
    try:
        for token in tokenize.generate_tokens(io.StringIO(code).readline):
            match = OWN_SCOPE_ANNOTATION.search(token.string) if token.type == tokenize.COMMENT else None
            if match:
                found[first_line + token.start[0] - 1] = match.group(1).strip()
    except (tokenize.TokenError, IndentationError, SyntaxError):
        pass
    return found

def own_scope_annotations(path, text, lines):
    """own-scope 标注的行号与理由。.py 只认注释；.sh 认 shell 那一层的行（bash -c 的代码串里也算），
    喂给别的命令的 heredoc 正文里的不算——喂给 python 的只认它的注释。"""
    if path.endswith(".py"):
        return python_comment_annotations(text)
    stripped = shell_words.strip_data_heredocs(text, keep_line_count=True).split("\n")
    data_lines = {number for number, (original, kept) in enumerate(zip(lines, stripped), 1) if original != kept and kept == ""}
    found = {number: OWN_SCOPE_ANNOTATION.search(line).group(1).strip()
             for number, line in enumerate(lines, 1) if number not in data_lines and OWN_SCOPE_ANNOTATION.search(line)}
    for body in python_heredoc_bodies(text):
        start = text.find(body.rstrip("\n"))
        if start >= 0:
            found.update(python_comment_annotations(body, text.count("\n", 0, start) + 1))
    return found

# ⑪ rm -r 作用在工作根本身、它的上级或工作根下第一层带通配：工作根下的目录分属不同会话，按名字通删认不出归属。
#   出路是 scripts/hosts/work-directory-ownership.sh sweep（只删归属是本会话、没有进程开着的，删前逐项连大小写日志）。
import fnmatch
WORK_ROOT_SWEEP_SCRIPT = "scripts/hosts/work-directory-ownership.sh"
REMOVAL_GLOB_CHARACTERS = set("*?[")
REMOVAL_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")

def work_root_removal_roots():
    """⑪ 认的根：配置的工作根（不看弄坏开关）与会话临时目录根 /tmp/claude-<uid>（里面是各会话的临时目录），连同解开符号链接的写法。
    同目录的 lib_work_root.py 读不到时只认 SINGLEFS_WORK_ROOT（不补缺省、不退回家目录：没设就只认会话临时目录根）。"""
    if work_roots:
        roots = [work_roots.configured_work_root(), work_roots.claude_session_temporary_root()]
    else:
        configured = os.environ.get("SINGLEFS_WORK_ROOT")
        roots = ([os.path.normpath(os.path.abspath(configured))] if configured else []) + [f"/tmp/claude-{os.getuid()}"]
        roots = [os.path.normpath(os.path.abspath(os.path.expanduser(os.environ.get("SINGLEFS_WORK_ROOT") or os.path.join(data_root, "work")))),
                 f"/tmp/claude-{os.getuid()}"]
    return list(dict.fromkeys(roots + [os.path.realpath(root) for root in roots]))

def recursive_removal_operands(arguments):
    """rm 的参数里带 -r / -R / --recursive（与别的短选项合写的也算，写在操作数后面的也算）时交回操作数；不带交 []。"""
    recursive, operands, options_ended = False, [], False
    for argument in arguments:
        if not options_ended and argument == "--":
            options_ended = True
        elif not options_ended and argument.startswith("--"):
            recursive = recursive or argument == "--recursive"
        elif not options_ended and argument.startswith("-") and argument != "-":
            recursive = recursive or "r" in argument[1:] or "R" in argument[1:]
        else:
            operands.append(argument)
    return operands if recursive else []

def removal_target_path(word, directory, environment, configured_root):
    """rm 的一个操作数换成绝对路径（通配原样留着）：~、$HOME、$SINGLEFS_WORK_ROOT 与这条命令里设过的变量代入，相对路径按当时的目录；算不出交 None。"""
    if word == "~" or word.startswith("~/"):
        word = os.path.expanduser("~") + word[1:]
    unknown = []
    def substitute(match):
        name = match.group(1) or match.group(2)
        if name in environment:
            return environment[name]
        if name == "HOME":
            return os.path.expanduser("~")
        if name == "SINGLEFS_WORK_ROOT":
            return os.environ.get(name) or configured_root
        unknown.append(name)
        return ""
    word = REMOVAL_VARIABLE.sub(substitute, word)
    if unknown or "$" in word or "`" in word:
        return None
    if not os.path.isabs(word):
        if directory is None:
            return None
        word = os.path.join(directory, word)
    return os.path.normpath(word)

def removal_component_matches(name, pattern):
    """一段路径名对一段可能带通配的写法：照 bash 的默认（不开 dotglob），通配起头的不认点开头的名字。"""
    if not REMOVAL_GLOB_CHARACTERS & set(pattern):
        return name == pattern
    if name.startswith(".") and not pattern.startswith("."):
        return False
    return fnmatch.fnmatchcase(name, pattern)

def removal_hits_work_root(path, root):
    """rm -r 的一个目标（绝对路径，可能带通配）打到工作根本身、它的上级或工作根下第一层的通配，交回是哪一种；不是交 None。"""
    parts = [part for part in path.split("/") if part]
    root_parts = [part for part in root.split("/") if part]
    if not all(removal_component_matches(name, pattern) for name, pattern in zip(root_parts, parts)):
        return None
    covered_by_glob = any(REMOVAL_GLOB_CHARACTERS & set(pattern) for pattern in parts[:len(root_parts)])
    if len(parts) < len(root_parts):
        return "通配盖住了工作根的上级" if covered_by_glob else "工作根的上级"
    if len(parts) == len(root_parts):
        return "通配盖住了工作根本身" if covered_by_glob else "工作根本身"
    if REMOVAL_GLOB_CHARACTERS & set(parts[len(root_parts)]):
        return "工作根下第一层带通配"
    return None

def work_root_removal_refusal(command, directory):
    """命令位置上的 rm -r（每一层：bash -c、喂给 shell 的 heredoc、命令替换；前缀与 capped.sh 剥开；cd 与 export 跟着算）
    作用在工作根本身、它的上级或工作根下第一层带通配的路径上，交回认出的写法；否则交 []。前台、run_in_background、主 agent 与子 agent 一样拒。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-work-root-removal")):
        return []
    roots = work_root_removal_roots()
    found = []
    for at in shell_words.commands_at_command_position(command, directory).commands:
        if not at.words or os.path.basename(at.words[0]) != "rm":
            continue
        for operand in recursive_removal_operands(at.words[1:]):
            path = removal_target_path(operand, at.directory, at.environment, roots[0])
            if path is None:
                continue
            hit = next(((root, reason) for root in roots for reason in [removal_hits_work_root(path, root)] if reason), None)
            if hit:
                found.append(f"rm -r {operand}（{hit[1]}：{hit[0]}）")
    return list(dict.fromkeys(found))

# ⑳ 手拷仓副本（文件头 ⑳）：命令位置上的 rsync、cp -r / -R / -a 的源是本仓的 crates/ 或仓根整个。仓副本只经 scripts/hosts/minimal-copy.py 建，
#   拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算；手列的范围跟不上编译与测试要读的仓根条目。
MINIMAL_COPY_TOOL = "scripts/hosts/minimal-copy.py"
MANUAL_COPY_UNLISTED_TOOL = "scripts/hosts/compile-then-swap.py"   # 弄坏开关 manual-copy-howto-unlisted-tool 下出路写成它：不在 lib_project_code_runs.py 的直接跑名单上，照抄会被 project-code-guard.sh 拒
MANUAL_COPY_HOWTO_COPY_PLACEHOLDER = "<草稿目录>/copy"   # 出路里副本那一处的占位，自证照抄出路喂 project-code-guard.sh 时换成自证草稿里的一个路径
MANUAL_COPY_HOWTO_COMMAND = re.compile(r"→ 怎么办：仓副本只用 `([^`]*)` 建")   # 自证从拒绝说明里取出路的整条写法
# rsync 带值的选项（值另起一个词时跳过它；写成 --选项=值 的不用跳）：短的是 -e、-f、-T、-B、-M、-@
RSYNC_SHORT_OPTIONS_WITH_VALUE = "efTBM@"
RSYNC_LONG_OPTIONS_WITH_VALUE = frozenset({
    "--exclude", "--include", "--exclude-from", "--include-from", "--filter", "--files-from", "--rsh", "--rsync-path", "--temp-dir",
    "--compare-dest", "--copy-dest", "--link-dest", "--backup-dir", "--suffix", "--chmod", "--chown", "--usermap", "--groupmap", "--timeout",
    "--contimeout", "--port", "--sockopts", "--out-format", "--log-file", "--log-file-format", "--password-file", "--bwlimit", "--partial-dir",
    "--max-size", "--min-size", "--max-delete", "--block-size", "--modify-window", "--checksum-choice", "--compress-choice", "--compress-level",
    "--skip-compress", "--iconv", "--info", "--debug", "--stop-after", "--stop-at", "--write-batch", "--only-write-batch", "--read-batch",
    "--protocol", "--outbuf", "--remote-option", "--address", "--copy-as", "--max-alloc"})
RSYNC_LISTING_OPTIONS = frozenset({"--dry-run", "--list-only"})   # 只演一遍或只列，不拷
RSYNC_REMOTE_OPERAND = re.compile(r"^[^/]*:")   # 主机:路径、rsync://主机/…（本地路径里带冒号的写成 ./ 起头）

def manual_copy_howto(repository_root):
    """⑳ 拒绝说明里的出路整条：minimal-copy.py 照这一份钩子所在的仓根建副本、带测试输入；弄坏开关 manual-copy-howto-unlisted-tool 下换成不在直接跑名单上的脚本。"""
    tool = MANUAL_COPY_UNLISTED_TOOL if detector_break_is_set("manual-copy-howto-unlisted-tool") else MINIMAL_COPY_TOOL
    return f"python3 {tool} {repository_root} {MANUAL_COPY_HOWTO_COPY_PLACEHOLDER} --with-test-inputs"

def rsync_copied_sources(arguments):
    """rsync 的参数切开交回源（操作数里除最后一个目的之外的）；带 -n / --dry-run / --list-only 的、只给一个操作数的（只列不拷）交 []。"""
    operands, position, listing_only = [], 0, False
    while position < len(arguments):
        argument = arguments[position]
        position += 1
        if argument == "--":
            operands += arguments[position:]
            break
        if argument == "-" or not argument.startswith("-"):
            operands.append(argument)
            continue
        if argument.startswith("--"):
            option, has_value, _value = argument.partition("=")
            if option in RSYNC_LISTING_OPTIONS:
                listing_only = True
            elif option in RSYNC_LONG_OPTIONS_WITH_VALUE and not has_value:
                position += 1
            continue
        letters = argument[1:]
        for index, letter in enumerate(letters):
            if letter in RSYNC_SHORT_OPTIONS_WITH_VALUE:
                if index + 1 == len(letters):
                    position += 1
                break
            if letter == "n":
                listing_only = True
    return [] if listing_only or len(operands) < 2 else operands[:-1]

def copy_arguments_recursive(arguments):
    """cp 的参数里带 -r / -R / -a（与别的短选项合写的也算，写在操作数后面的也算）、--recursive、--archive；`--` 之后的不算。"""
    for argument in arguments:
        if argument == "--":
            return False
        if argument in ("--recursive", "--archive"):
            return True
        if argument.startswith("-") and not argument.startswith("--") and argument != "-":
            for letter in argument[1:]:
                if letter in COPY_SHORT_OPTIONS_WITH_VALUE["cp"]:
                    break
                if letter in "rRa":
                    return True
    return False

def copied_sources(name, arguments):
    """rsync 与递归的 cp 的源操作数；别的命令、不拷的写法（rsync -n、只给一个操作数、cp 不带递归）交 []。"""
    if name == "rsync":
        return rsync_copied_sources(arguments)
    if name == "cp" and copy_arguments_recursive(arguments):
        parsed = parse_copy_arguments("cp", arguments)
        return list(parsed.operands) if parsed.target_directory else list(parsed.operands[:-1])
    return []

def copied_source_hits(word, directory, variables, protected):
    """一个源操作数打到 protected（{真实路径: 说法}）里的哪一处：交说法的列表；远端写法、这一刻算不出的、没打到的交 []。
    ~、前面赋过值的变量与 $HOME 展开，相对路径按当时的目录，规整 .. 与结尾的 /；带通配的照 bash 的默认展开，通配写在 protected 那一处下一层整层的也算。"""
    if RSYNC_REMOTE_OPERAND.match(word):
        return []
    if word == "~" or word.startswith("~/"):
        word = os.path.expanduser("~") + word[1:]
    expanded = shell_words.expand_known_variables(word, variables or {})
    if expanded is None:
        return []
    if not os.path.isabs(expanded):
        if directory is None:
            return []
        expanded = os.path.join(directory, expanded)
    expanded = os.path.normpath(expanded)
    if not REMOVAL_GLOB_CHARACTERS & set(expanded):
        what = protected.get(os.path.realpath(expanded))
        return [what] if what else []
    hits = []
    parent, last = os.path.split(expanded)
    covered = protected.get(os.path.realpath(parent)) if REMOVAL_GLOB_CHARACTERS & set(last) and not REMOVAL_GLOB_CHARACTERS & set(parent) else None
    if covered:
        hits.append(f"通配盖住{covered}的整层")
    for match in sorted(glob.glob(expanded)):
        what = protected.get(os.path.realpath(match))
        if what:
            hits.append(f"通配展开到{what}")
    return hits

def manual_repository_copy_refusal(command, repository_root, start_directory):
    """命令位置上的 rsync、cp -r / -R / -a（每一层：bash -c、喂给 shell 的 heredoc、命令替换；前缀剥开；cd 与前面赋过值的变量跟着算）的源是本仓的 crates/
    或仓根整个（文件头 ⑳），交回认出的写法；否则交 []。前台、run_in_background、主 agent 与子 agent 一样拒。弄坏开关 allow-manual-repository-copy 下一概放行。"""
    if detector_break_is_set("allow-manual-repository-copy"):
        return []
    root = os.path.realpath(repository_root)
    protected = {root: "仓根整个", os.path.realpath(os.path.join(root, "crates")): "本仓的 crates/"}
    found = []
    for at in shell_words.commands_at_command_position(command, start_directory).commands:
        if not at.words:
            continue
        name = os.path.basename(at.words[0])
        for word in copied_sources(name, at.words[1:]):
            for hit in copied_source_hits(word, at.directory, at.variables, protected):
                found.append(f"{name} {word}（{hit}）")
    return list(dict.fromkeys(found))

def embedded_python_codes(command, repository_root):
    """命令里真要被 python 跑的代码：`python3 -c` 的那一段，和喂给 python 的 heredoc 正文（含 bash -c、命令替换、喂给 shell 的 heredoc 里的那几层）。
    喂给 cat、tee 这类别的命令的 heredoc 正文是数据，引号里当数据写的文字也不是代码，都不算（往草稿目录写一份脚本、笔记，正文里出现 open(…, 'w') 不是在写）。"""
    codes = [step.code for step in overwrite_steps(command, repository_root, every_write=False) if isinstance(step, PythonCode)]
    for layer in shell_layers(command):
        for words in layer.commands:
            codes += [code for kind, code in embedded_code_strings(words) if kind == "python"]
    return list(dict.fromkeys(codes))

def python_code_directories(command, start_directory):
    """⑧ python 一半：每段喂给 python 的代码 → 跑它那一刻的当前目录（几处都跑的列全；cd 的目录算不出的那一处是 None）。取 overwrite_steps 的 PythonCode，与 sed -i 一半同一处的 cd 跟踪。"""
    directories = {}
    for step in overwrite_steps(command, start_directory, every_write=False):
        if isinstance(step, PythonCode):
            directories.setdefault(step.code, []).append(step.directory)
    return directories

def repository_in_place_edit_refusal(command, agent_type, repository_root, start_directory=None, script_roots=None):
    """⑧ 要拒的写法与目标（repository_in_place_edit_verdict 的 refused）。"""
    return repository_in_place_edit_verdict(command, agent_type, repository_root, start_directory, script_roots).refused

def repository_in_place_edit_verdict(command, agent_type, repository_root, start_directory=None, script_roots=None):
    """start_directory 是命令开头的当前目录（hook 输入里的 cwd，没有就是仓库根）；sed -i / perl -i 的相对路径从 cd 之后的目录起算（跟 ⑫ 同一处的 cd 跟踪，overwrite_steps）。
    script_roots 是读 python 脚本正文的那几个根（python3 <脚本文件> 的脚本落在其中之一底下才读）；不给是工作根（work_roots.scratch_roots）。"""
    if (not agent_type or detector_break_is_set("disable-check")
            or detector_break_is_set("allow-repository-in-place-edit")
            or not agent_has_edit_tool(repository_root, agent_type)):
        return InPlaceEditVerdict([], [])
    found = []
    in_place_forms = {"sed -i", "perl -i"}
    if detector_break_is_set("perl-in-place-unseen"):   # 弄坏开关：⑧ 不认 perl -i
        in_place_forms.discard("perl -i")
    if detector_break_is_set("sed-in-place-ignores-cd"):   # 弄坏开关：sed -i 的相对路径不跟 cd
        in_place_forms.discard("sed -i")
        for layer in shell_layers(command):
            for words in layer.commands:
                for _, _, name, arguments in wrapped_commands(words):
                    if name == "sed":
                        found += [f"sed -i {path}" for path in sed_in_place_targets(arguments)]
    found += [f"{step.form} {step.target}" for step in overwrite_steps(command, start_directory or repository_root, every_write=True)
              if isinstance(step, OverwriteCandidate) and step.form in in_place_forms and not sed_target_resolved_in_scratch(step, repository_root)]
    pattern = python_repository_write_regex(repository_root)
    # 相对路径从跑那段代码时的当前目录起算（弄坏开关 python-relative-path-ignores-cd 下不跟 cd，一律按仓内判）
    code_directories = {} if detector_break_is_set("python-relative-path-ignores-cd") else python_code_directories(command, start_directory or repository_root)
    for code in (embedded_python_codes(command, repository_root) if not detector_break_is_set("heredoc-body-judged-as-commands") else [command]):
        found += python_repository_writes(code, pattern, code_directories.get(code), repository_root, "python")
    noted = []
    for script, directory, body in python_script_bodies(command, start_directory or repository_root, script_roots):
        writes = python_repository_writes(body, pattern, [directory] if not detector_break_is_set("python-relative-path-ignores-cd") else None,
                                          repository_root, f"python 脚本 {script}")
        found += writes
        unliteral = [] if writes else python_unliteral_write_targets(body)
        if unliteral and not detector_break_is_set("python-script-unliteral-write-unnoted"):
            noted.append(f"判不了 ⑧ python 脚本 {script} 里写文件的目标不是字面量（{'；'.join(unliteral[:PYTHON_UNLITERAL_LISTED])}"
                         f"{f'；另有 {len(unliteral) - PYTHON_UNLITERAL_LISTED} 处' if len(unliteral) > PYTHON_UNLITERAL_LISTED else ''}），"
                         "判不出写不写仓内文件，照常执行；仓内文件该用 Edit 改")
    return InPlaceEditVerdict(list(dict.fromkeys(found)), list(dict.fromkeys(noted)))

def python_unliteral_write_targets(code):
    """python 代码里写文件、而目标不是字符串字面量的调用（sys.argv、变量、拼出来的路径）：open(…, 带 w / a / x / + 的模式)、Path(…).write_text / write_bytes、
    Path(…).open(同上)、shutil.copyfile / copy / copy2 / move 与 os.replace / os.rename 的目标。交 ["写法(目标的源码)"…]；解析不了的代码交 []。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    def is_literal(node):
        return isinstance(node, ast.Constant) and isinstance(node.value, str)
    def shown(node):
        return ast.unparse(node)[:80] if node is not None else "?"
    found = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        name, owner = called_name_and_owner(node.func)
        receiver = node.func.value if isinstance(node.func, ast.Attribute) else None
        if name == "open" and (receiver is None or (isinstance(receiver, ast.Name) and receiver.id in PYTHON_OPEN_MODULES)):
            mode, target = opening_mode(call_argument(node, 1, "mode")), call_argument(node, 0, "file")
            if mode is not None and any(letter in mode for letter in "wax+") and target is not None and not is_literal(target):
                found.append(f"open({shown(target)}, '{mode}')")
        elif name in PYTHON_PATH_WRITERS | {"open"} and receiver is not None and owner != "os":   # os.open(路径, 标志) 不是 Path.open
            constructed = isinstance(receiver, ast.Call) and called_name_and_owner(receiver.func)[0] in PYTHON_PATH_CONSTRUCTORS
            target = receiver.args[0] if constructed and receiver.args else receiver
            mode = opening_mode(call_argument(node, 0, "mode")) if name == "open" else "w"
            if mode is not None and any(letter in mode for letter in "wax+") and not is_literal(target):
                found.append(f"Path({shown(target)}).{name}")
        elif (name in PYTHON_FILE_COPIERS | {"move"} and owner == "shutil") or (name in ("replace", "rename") and owner == "os"):
            target = call_argument(node, 1, "dst")
            if target is not None and not is_literal(target):
                found.append(f"{owner}.{name}(…, {shown(target)})")
    return found

def compile_first_verdict(command, repository_root, start_directory=None):
    """整条命令里写主工作区 crates/ 下 .rs 的每一步，交回 `写法 相对仓库根的路径`。
    相对路径从 cd 之后的目录起算，起点是 start_directory（hook 输入的 cwd，同 ⑧ ⑫；不给是仓库根）；cd 算不出之后的照仓库根解。
    弄坏开关 BASH_COMMAND_DETECTOR_BREAK=compile-first-ignores-cwd 下起点一律是仓库根。"""
    crates_directory = os.path.join(repository_root, "crates")
    refused = []
    if start_directory is None or detector_break_is_set("compile-first-ignores-cwd"):
        start_directory = repository_root
    for step in overwrite_steps(command, start_directory, every_write=True):
        base = step.directory if step.directory is not None else repository_root
        targets = []
        if isinstance(step, PreservingStep):
            if step.kind in ("mv", "rm"):
                for path in preserved_paths(step, base):
                    if is_main_workspace_crate_source(path, repository_root) or (
                            os.path.isdir(path) and (os.path.normpath(path) == crates_directory or is_inside(path, crates_directory))):
                        refused.append(f"{step.kind} {os.path.relpath(path, repository_root)}")
            continue
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code, every_write=True):
                path = shell_words.resolve_path(base, write.path)
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                targets.append((write.form, path))
        else:
            for word in destination_words(step, base):
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                path = shell_words.resolve_path(base, pattern)
                if has_unknown_piece or has_glob:
                    # `"$变量"/crates/…` 这类变量在前的，另从 crates/ 那一段起按仓库根接（与 ⑤ 认 research/results/ 同一个办法）
                    anchor = pattern.find("crates/")
                    patterns = [path] + ([os.path.join(repository_root, pattern[anchor:])] if anchor > 0 and "*" in pattern[:anchor] else [])
                    targets += [(step.form, match) for candidate in patterns for match in sorted(glob.glob(candidate))]
                else:
                    targets.append((step.form, path))
        for form, path in targets:
            if is_main_workspace_crate_source(path, repository_root):
                refused.append(f"{form} {os.path.relpath(path, repository_root)}")
    return list(dict.fromkeys(refused))

def compile_first_refusal(command, agent_type, repository_root, start_directory=None):
    """交回要拒的写法与目标；只判 COMPILE_FIRST_AGENTS 里的 agent，前台、run_in_background 一样判。start_directory 是 hook 输入的 cwd（compile_first_verdict）。"""
    if (agent_type not in COMPILE_FIRST_AGENTS or detector_break_is_set("disable-check")
            or detector_break_is_set("allow-compile-first-bypass")):
        return []
    return compile_first_verdict(command, repository_root, start_directory)

def bash_scope_verdict(command, repository_root, start_directory, patterns, outside_repository_judged=False, service_view=None):
    """整条命令里每一处写，算出最终落点：落点在本仓里、又不在 patterns（写范围表里的模式）里的交进 out_of_scope（`写法 相对仓库根的路径`）；
    outside_repository_judged（没有项目定义的 agent）时落在仓外、不在 patterns 里、也不在 /dev/、/proc/ 下的同样交进 out_of_scope（`写法 绝对路径`）；
    落点这一刻算不出的（目标里有没赋过值或来源不明的变量、命令替换、通配，cd 到算不出的目录之后的相对路径，读不到规格的 replace-batch.py）交进 undeterminable。
    能算的照算：同一条命令里前面赋的写死值、`for 名字 in 写死的词` 逐个取值、cd 之后的 $PWD、mktemp -d 赋的变量、$HOME；落点先规整 `..`、解开符号链接再判。
    service_view（⑭ 用，ServiceSourceView）给了就不照写范围判：每一种落点交 service_view 判，patterns 与 undeterminable 的判法不起作用——落点是已有的服务源码（或与它是同一个文件的硬链接）的交进 out_of_scope；
    rm、mv 挪走、给源建硬链接的写法，落点是包着已有服务源码的目录也交；解包与打补丁的写法（tar -x、patch、git apply），落点目录碰得到已有服务源码也交（包里、补丁里写了什么不读）；
    cp -r、rsync 的源是目录时，按源里的文件逐份对目标目录里已有的服务源码（service_directory_copy_hits）。
    弄坏开关 service-source-overwrite-only 下只认整份覆盖的写法（不认追加、sed -i、定点写脚本、python 的追加、rsync），service-source-lexical-only 下落点只按字面规整。"""
    expanded = write_scope.expanded_scope_patterns(patterns, work_roots)
    refused, undeterminable = [], []
    roots = list(dict.fromkeys((os.path.normpath(repository_root), os.path.realpath(repository_root))))
    lexical_only = detector_break_is_set("scope-landing-lexical-only")   # 弄坏开关：落点只按字面规整
    service_judged = service_view is not None
    overwrite_only = service_judged and detector_break_is_set("service-source-overwrite-only")
    service_lexical_only = service_judged and detector_break_is_set("service-source-lexical-only")
    def inside_repository(path):
        return next((os.path.relpath(path, root) for root in roots if path.startswith(root + os.sep)), None)
    def judge(form, raw_path, mode=None):
        """与写闸同一份判法（lib_write_scope.landings_out_of_scope）：每一种落点（照 shell 逐段解开符号链接的、规整 `..` 之后再解开的）都要在范围里；
        不在范围里的落点落在本仓里的拒，落在仓外的不判。字面路径在范围里、解开之后落在仓里范围外的照拒。解不开（读不了链接）的照算不出拒。
        ⑭（service_view）：落点是已有的服务源码、或与它是同一个文件的硬链接（落在仓外的也认）的拒；mode 是 holding（rm、mv 挪走、硬链接的源）时落点是包着已有服务源码的目录也拒，
        tree（tar -x、patch、git apply）时落点目录碰得到已有服务源码也拒。"""
        if service_judged:
            try:
                service_landings = write_scope.landing_forms(raw_path, lexical_only=service_lexical_only)
            except OSError:
                return   # 落点解不开：⑫ 照算不出拒，这一道不另判
            for landing in service_landings:
                shown = inside_repository(landing)
                if (service_view.is_source(shown, landing) or (mode == "holding" and service_view.holds_source(landing))
                        or (mode == "tree" and service_view.touches(landing))):
                    refused.append(f"{form} {shown or landing}")
                fixture = service_view.gate_fixture_note(shown, landing, mode)   # ⑭ 另一半：已有门禁的样本档目录
                if fixture:
                    refused.append(f"{form} {shown or landing}（{fixture}）")
            return
        lock_problem = writer_lock_refusal(raw_path)
        if lock_problem:
            refused.append(f"{form} {raw_path}（{lock_problem}）")
        try:
            outside = write_scope.landings_out_of_scope(expanded, raw_path, repository_root, lexical_only=lexical_only)
        except OSError as error:
            undeterminable.append(f"{form} {raw_path}：解开符号链接时出错（{error!r}），落点算不出")
            return
        for landing in outside:
            shown = inside_repository(landing)
            if shown is None:
                if outside_repository_judged and not any(path.startswith(DEVICE_PREFIXES) for path in (landing, os.path.normpath(raw_path))):
                    refused.append(f"{form} {landing}")   # 没有项目定义的 agent：仓外别处同样拒
                continue   # 落在仓外：有写范围的项目 agent 不判
            literal = inside_repository(os.path.normpath(raw_path)) or raw_path
            refused.append(f"{form} {shown}" if literal == shown else f"{form} {literal}（解开符号链接之后落在 {shown}）")
        if SCRATCH_OWNER_IDENTITY.get("leg_agent_type") and not detector_break_is_set("scope-leg-round-unchecked"):
            # 三方腿：写范围表放行的 research/prompts/ 下的落点，再收窄到这一轮自己的报告与模型目录（leg_round_problem）
            try:
                round_landings = [landing for landing in write_scope.landing_forms(raw_path, lexical_only=lexical_only) if landing not in outside]
            except OSError:
                round_landings = []   # 落点解不开：上面已照算不出拒
            for landing in round_landings:
                shown = inside_repository(landing)
                round_problem = leg_round_problem(shown[len(LEG_PROMPTS_PREFIX):]) if shown is not None and shown.startswith(LEG_PROMPTS_PREFIX) else None
                if round_problem:
                    refused.append(f"{form} {shown}（{round_problem}）")
    for step in overwrite_steps(command, start_directory, every_write=not overwrite_only, write_scripts=not overwrite_only, service_writes=service_judged):
        if isinstance(step, UnknownScriptTarget):
            undeterminable.append(step.reason)
            continue
        if isinstance(step, UnresolvedScriptTarget):
            # 脚本词算不出：第一个参数算得出、落在本仓里的（在不在写范围里都一样）照算不出拒；参数本身算不出、cd 算不出之后的相对路径、落在仓外的不判
            word = home_expanded(step.target)
            pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
            if has_unknown_piece or has_glob or (step.directory is None and not os.path.isabs(pattern)):
                continue
            path = os.path.normpath(pattern if os.path.isabs(pattern) else os.path.join(step.directory, pattern))
            if inside_repository(path) is not None or inside_repository(os.path.realpath(path)) is not None:
                undeterminable.append(f"{step.form} {step.target}：脚本词里有这一刻算不出的段，认不出起的是不是 replace-once.py / insert-row.py / replace-batch.py，"
                                      "第一个参数又落在本仓里")
            continue
        if isinstance(step, LedgerAppendTarget):
            # ledger-append-section.py：落点是那个仓登记表 current 那一份账本；仓根算不出的拒，登记表读不到的不判（脚本自己退 2、什么都不写）
            ledger_root, problem = ledger_append_root(step, repository_root)
            if problem:
                undeterminable.append(f"{step.form} {problem}，要写哪一份账本算不出")
                continue
            ledger, _ = current_ledger_relative(ledger_root)
            if ledger is not None:
                judge(f"{step.form}（登记表 current 那一份账本）", os.path.join(ledger_root, ledger))
            continue
        if isinstance(step, PreservingStep):
            continue   # git add、mv 挪走的源、rm：不是写进去的
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PythonCode):
            for write in python_in_place_writes(step.code, every_write=not overwrite_only, service_writes=service_judged):
                if step.directory is None and not os.path.isabs(os.path.expanduser(write.path)):
                    undeterminable.append(f"{write.form} {write.path}：前面有 cd 到这一刻算不出的目录，相对路径落在哪算不出")
                    continue
                path = write.path if os.path.isabs(write.path) else os.path.join(base, write.path)
                if write.mode == "directory-copy":
                    # shutil.copytree(源, 目标, dirs_exist_ok=True)：源里的内容直接落在目标目录下，逐份对目标目录里已有的服务源码（同 cp -r 源/. 目标/）
                    source_path = write.source if os.path.isabs(write.source) else os.path.join(base, write.source)
                    if os.path.isdir(source_path) and os.path.isdir(path):
                        for hit in service_view.overwritten_by_directory_copy(source_path, path, True):
                            judge(write.form, hit)
                    continue
                if write.source is not None and os.path.isdir(path):
                    path = os.path.join(path, os.path.basename(write.source))
                judge(write.form, path, write.mode)
            continue
        choices = loop_value_choices((step.target, *step.sources))
        if choices is None:
            undeterminable.append(f"{step.form} {readable_loop_values(step.target)}：循环变量的取值组合多过 {MAXIMUM_LOOP_LANDINGS} 种")
            continue
        for choice in choices:
            candidate = step._replace(target=home_expanded(with_loop_values(step.target, choice)),
                                      sources=tuple(home_expanded(with_loop_values(source, choice)) for source in step.sources))
            if service_judged:
                # ⑭：源是目录的 cp -r、rsync 逐份对目标目录里已有的服务源码；cp -l、不带 -s 的 ln 另判源（硬链接的源是服务源码，往那个名字写就是往服务源码写）
                for hit in service_directory_copy_hits(service_view, candidate, base):
                    judge(f"{step.form}（整目录拷）", hit)
                if candidate.links_sources and not detector_break_is_set("service-source-link-unseen"):
                    for source in candidate.sources:
                        source_pattern, source_unknown, source_glob = target_pattern(os.path.expanduser(source))
                        if source_unknown or (step.directory is None and not os.path.isabs(source_pattern)):
                            continue
                        source_path = source_pattern if os.path.isabs(source_pattern) else os.path.join(base, source_pattern)
                        for linked in (sorted(glob.glob(source_path)) if source_glob else [source_path]):
                            judge(f"{step.form}（给源建硬链接）", linked, "holding")
            for word in destination_words(candidate, base):
                word = mktemp_substitutions_expanded(word, step.directory)   # 直接写在目标里、模板在自己草稿目录里的 $(mktemp -d …)
                pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
                if (has_unknown_piece and step.form in OVERWRITING_REDIRECTS | APPENDING_REDIRECTS and BRACE_PIECE.search(word)
                        and not detector_break_is_set("brace-target-reason-generic")):
                    # 重定向目标上的花括号照拒，说明点名 bash 报 ambiguous redirect、什么都不写
                    undeterminable.append(f"{step.form} {word}：重定向目标上有花括号。引号外的 bash 展开成几个词、报 ambiguous redirect、一个字节都不写；"
                                          "引号里的（`> 'o.{a,b}'`）bash 当一个字面文件名，这道闸切词已分得出引号里外，对重定向目标上的花括号一律照算不出拒（偏严，用户 2026-10-29 定）。"
                                          "写成一个不带花括号的字面文件名；要写几份就逐份写 `> 文件1`、`> 文件2`，或用 tee 带几个文件参数")
                    continue
                if has_unknown_piece and any(placeholder in word for placeholder in launcher_input_words()):
                    undeterminable.append(f"{step.form} {word}：落点是 xargs、find -exec 或 parallel 读进来接进命令的词，这一刻算不出（认不出的一律拒）。"
                                          "改成逐个写字面路径（`sed -i … <文件1> <文件2>`）；要先找文件的，先 find / grep -l 列出来看，再逐个写字面路径")
                    continue
                if (service_judged and has_glob and not has_unknown_piece and SERVICE_FORM_MODES.get(step.form) == "holding"
                        and (step.directory is not None or os.path.isabs(pattern))):
                    # ⑭：rm、mv 挪走的路径带通配（`rm prune-server/src/*.rs`）：这一刻有哪些文件展得开，就逐个判
                    for matched in sorted(glob.glob(pattern if os.path.isabs(pattern) else os.path.join(base, pattern))):
                        judge(step.form, matched, "holding")
                    continue
                if has_unknown_piece or has_glob:
                    undeterminable.append(f"{step.form} {word}：目标里有这一刻算不出的段（没赋过值或来源不明的变量、命令替换、通配）")
                    continue
                if step.directory is None and not os.path.isabs(pattern):
                    undeterminable.append(f"{step.form} {word}：前面有 cd 到这一刻算不出的目录，相对路径落在哪算不出")
                    continue
                judge(step.form, pattern if os.path.isabs(pattern) else os.path.join(base, pattern), SERVICE_FORM_MODES.get(step.form) if service_judged else None)
    return list(dict.fromkeys(refused)), list(dict.fromkeys(undeterminable))

def bash_scope_refusal(command, agent_type, repository_root, start_directory, table_path, identity=None):
    """交回 ScopeVerdict；前台、run_in_background 一样判。只判有写范围的项目子 agent（文件头 ⑫ 的「没有写范围的不判」）。
    identity：hook 输入里的 agent_id、session_id、transcript_path，认 mktemp 模板在不在这个子 agent 认领的草稿目录里用（不给就都不在）。
    弄坏开关 allow-bash-scope-write 一概放行；scope-main-agent-judged 把主 agent 也当成一条模式都没有的子 agent 判；
    scope-undeterminable-only-noted 下落点算不出的只记检出、不拒。"""
    if detector_break_is_set("disable-check") or detector_break_is_set("allow-bash-scope-write"):
        return ScopeVerdict([], [], [], [])
    if agent_type:
        patterns = agent_scope_patterns(repository_root, agent_type, table_path)
    else:
        patterns = [] if detector_break_is_set("scope-main-agent-judged") else None
    if patterns is None or (not patterns and agent_type):
        return ScopeVerdict([], [], [], [])
    if write_scope is None:
        raise RuntimeError(f"读不到写范围的判法 lib_write_scope.py（{write_scope_error!r}）")
    builtin = bool(agent_type) and not agent_has_definition(repository_root, agent_type)   # 没有项目定义的 agent：只许写工作根，仓外别处也拒
    SCRATCH_OWNER_IDENTITY.clear()
    SCRATCH_OWNER_IDENTITY.update({key: value for key, value in (identity or {}).items() if isinstance(value, str) and value})
    if agent_type and not builtin and not definition_can_write(repository_root, agent_type):
        SCRATCH_OWNER_IDENTITY["read_only"] = "1"   # 只带 Read、Bash 的定义：写闸第九道看不见它，写工作根下的草稿目录由这里照写者锁判
    if agent_type and not builtin and not detector_break_is_set("scope-leg-round-unchecked") and is_three_way_leg(repository_root, agent_type):
        SCRATCH_OWNER_IDENTITY["leg_agent_type"] = agent_type   # 三方腿：往 research/prompts/ 写再收窄到这一轮自己的报告与模型目录（leg_round_problem）
    try:
        refused, undeterminable = bash_scope_verdict(command, repository_root, start_directory, patterns, outside_repository_judged=builtin)
    finally:
        SCRATCH_OWNER_IDENTITY.clear()
    if detector_break_is_set("scope-undeterminable-only-noted"):
        return ScopeVerdict(refused, [], [f"{item}，写范围没判，没拦" for item in undeterminable], patterns, builtin)
    return ScopeVerdict(refused, undeterminable, [], patterns, builtin)

def service_source_refusal(command, agent_type, repository_root, start_directory=None):
    """交回要拒的写法与目标（`写法 相对仓库根的路径`，落在仓外的硬链接写绝对路径）；只判 SERVICE_SOURCE_AGENTS 里的 agent，前台、run_in_background 一样判；
    主 agent、别的 agent 类型与认不出 agent 类型的调用不判。start_directory 是 hook 输入的 cwd（不给是仓库根）。落点这一刻算不出的不在这里判（⑫ 对有写范围的 agent 照算不出拒）。
    弄坏开关 allow-service-source-write 一概放行。"""
    if agent_type not in SERVICE_SOURCE_AGENTS or detector_break_is_set("allow-service-source-write"):
        return []
    if write_scope is None:
        raise RuntimeError(f"读不到写范围的判法 lib_write_scope.py（{write_scope_error!r}）")
    refused, _ = bash_scope_verdict(command, repository_root, start_directory or repository_root, [], service_view=ServiceSourceView(repository_root))
    return refused

def whole_target_build_messages(findings):
    """㉓ 拒绝说明的两行（原因，出路）：逐条列交给服务的那一条与点到的选项；出路写只查不链接（`--` 之后带 check.sh 现在的 lint 表整段）、点名目标、只编库与二进制三种写法。
    弄坏开关 whole-target-howto-unnamed-test 下出路里的 cargo test 不点名测试全名（照抄交上去被 run-scope-guard.sh 的「经 submit 交的要点名」拒）；
    whole-target-howto-clippy-lints-dropped 下出路里的 cargo clippy 不带 lint 表、也不提它（照抄交上去被 project-code-guard.sh 的 clippy lint 表那一道拒）。"""
    listed = "；".join(f"{shown}（{'、'.join(flags)}）" for shown, flags in findings)
    reason = (f"  ✗ 交给任务服务的 cargo build 点了全部目标：{listed}：调试构建把测试、示例与基准逐个链接成可执行文件，验证包一趟写两百多 GiB，"
              "盘顶满时别的任务写不进、等判定库的任务被服务判卡住")
    test_form = ("cargo test -p <包> --test <目标>" if detector_break_is_set("whole-target-howto-unnamed-test")
                 else "cargo test -p <包> --test <目标> -- <测试全名…> --exact")
    lints_dropped = detector_break_is_set("whole-target-howto-clippy-lints-dropped")
    lint_tail = "" if lints_dropped else clippy_lint_tail(os.path.join(repository_root_of(sys.argv[1]), CLIPPY_LINT_TABLE_RELATIVE))
    clippy_form = f"cargo clippy -p <包> --all-targets {lint_tail}" if lint_tail else "cargo clippy -p <包> --all-targets"
    if lints_dropped:
        lint_note = ""
    elif lint_tail:
        lint_note = "；-- 之后是 check.sh 的 CODE_DISCIPLINE_LINTS 现在的整段（表里的词原样，整轮门禁与合入后验证都带它，project-code-guard.sh 读同一份表、要它写全）"
    else:
        lint_note = "；-- 之后要照 check.sh 的 CODE_DISCIPLINE_LINTS 补 -D warnings 与表里的每一条（这一次没读到那张表；project-code-guard.sh 读同一份表、读不到时也不判）"
    way = (f"     → 怎么办：只核编得过写 `{clippy_form}`（只查不链接{lint_note}）；要编、要跑的测试点名目标写 "
           f"`{test_form}`（库里的测试把 --test <目标> 换成 --lib）；构建只编库与二进制写 `cargo build -p <包>`；"
           "都经 task-server/task submit -- <写法> 交（跑测试的范围声明写在 submit 前面；子 agent 写进报告「要主 agent 跑的命令」一节交回）。"
           "提交时的整轮门禁带 --full-gate 交，不拒；交给服务的是脚本（合入后验证、门禁阶段）的，构建归脚本自己管，不拒。这一道只拒这种写法，不停你在跑的任何东西")
    return reason, way

def find_removal_steps(arguments, directory, variables, depth, every_write, write_scripts):
    """find（⑭）：-delete 删起点目录之下的文件；-exec / -execdir / -ok / -okdir 起的命令里对找到的路径（FIND_PATH_WORD 占位词）写、删、挪走的，起点目录也当写进去的路径判；
    两种都落在起点目录上（起点是包着已有服务源码的目录就拒），起点之下哪些文件被匹配这一刻不展开。弄坏开关 service-source-unlink-find-unseen 下不认。"""
    if detector_break_is_set("service-source-unlink-find-unseen"):
        return []
    arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
    starts = find_start_paths(arguments)
    steps = []
    if "-delete" in arguments:
        steps += [OverwriteCandidate("find -delete", start, directory) for start in starts]
    inner_text = launcher_inner_text("find", arguments, directory)
    placeholder = find_path_word()
    if inner_text is not None and placeholder:
        for step in overwrite_steps(inner_text, directory, variables, depth + 1, every_write, write_scripts, True):
            if isinstance(step, OverwriteCandidate) and placeholder in step.target:
                steps += [OverwriteCandidate("find -exec", start, directory) for start in starts]
                break
    return steps

def python_code_arguments(arguments, heredoc_bodies):
    """python 的参数：交回它要跑的代码（`-c` 的那一段；没给脚本、脚本是 `-` 或 /dev/fd/N、/dev/stdin 时是喂给它的 heredoc 正文）。
    `-m` 起模块、给了脚本文件的，交 []（脚本文件里的写法这里判不到）。"""
    position = 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "-c":
            return arguments[position + 1:position + 2]
        if argument == "-m":
            return []
        if argument in PYTHON_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if argument.startswith("-") and argument != "-":
            position += 1
            continue
        if argument != "-" and not argument.startswith(PYTHON_STANDARD_INPUT_SCRIPTS):
            return []
        break
    return [body.text for body in heredoc_bodies]

def follow_simple_command(words, directory, variables, steps, depth, heredoc_bodies=(), every_write=False, write_scripts=False, service_writes=False):
    """一条简单命令：会整份覆盖文件的写法（tee、cp、mv、install、dd、truncate；⑦ 另认喂给 python 的代码）与让旧字节留住的一步
    （git add、mv 挪走源；⑦ 另认 rm）追加进 steps；跟着 cd / pushd 换目录，跟着独立的赋值与 export 记变量（改 variables）；
    bash -c 的那段代码按这一刻的目录与变量递归进去。heredoc_bodies 是喂给这一条的、被当数据剥掉的 heredoc 正文。交回这条之后的当前目录。
    service_writes（⑭ 用）另交：带 -b / --backup 的 cp、mv、install（旧文件换了名字、这个名字照样是新写进去的）、ln、mv 挪走的源、rm 删掉的路径、tar -x、patch、git apply。"""
    skipped = shell_words.skip_prefixes(words)
    if skipped.command_position is None:
        for name, value in skipped.assignments.items():
            variables[name] = mktemp_directory_value(substitute_assigned_variables(value, variables), directory) or substitute_assigned_variables(value, variables)
        return directory
    name, arguments = None, []
    for _, _, name, arguments in wrapped_commands(words):
        pass
    if name is None:
        return directory
    if service_writes and name == "find":
        steps += find_removal_steps(arguments, directory, variables, depth, every_write, write_scripts)
    if write_scripts:
        steps += known_write_script_steps(name, arguments, directory, variables)
        inner_text = scope_launcher_inner_text(name, arguments, directory)
        if inner_text is not None:
            steps += overwrite_steps(inner_text, directory, variables, depth + 1, every_write, write_scripts, service_writes)
            return directory
    if shell_words.PYTHON_NAME.match(name):
        steps += [PythonCode(code, directory) for code in python_code_arguments(arguments, heredoc_bodies)]
        return directory
    if write_scripts and (name in LOOP_BINDING_COMMANDS or name in READING_COMMANDS):
        bind_loop_variables(name, arguments, variables)   # ⑫：循环变量逐个取值，read 读进来的作废
        return directory
    arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
    if service_writes and name == "rsync":
        steps += rsync_removal_steps(arguments, directory)
    if name == "export":
        for argument in arguments:
            assignment = shell_words.ASSIGNMENT.match(argument)
            if assignment:
                variables[assignment.group(1)] = mktemp_directory_value(assignment.group(2), directory) or assignment.group(2)
    elif name in ("cd", "pushd"):
        directory = shell_words.changed_directory(directory, arguments)
        if directory is not None and LOOP_VALUE_MARK in directory:
            directory = None   # ⑫ cd 进循环变量拼的目录：不逐个取值跟进去，之后的相对路径照算不出判
        remember_working_directory(variables, directory)
        return directory
    elif name == "popd":
        remember_working_directory(variables, None)
        return None
    elif name == "tee":
        files = tee_files(arguments)
        appending = every_write and not files   # tee -a / --append：tee_files 对它交空表
        if appending:
            files = [argument for argument in arguments if argument != "-" and not argument.startswith("-")]
        steps += [OverwriteCandidate("tee", file, directory, appends=appending) for file in files]
    elif every_write and name in ("sed", "perl"):
        steps += [OverwriteCandidate(f"{name} -i", file, directory) for file in in_place_editor_files(name, arguments)]
    elif (name == "rsync" and any(argument == "--inplace" or argument.startswith("--inplace=") for argument in arguments)
          and not detector_break_is_set("rsync-inplace-unseen")):
        # rsync --inplace 往已有的文件里就地写、不换 inode，⑤ ⑦ 照整份覆盖认；弄坏开关 rsync-inplace-unseen 下只有 ⑨ ⑫ 认它
        operands = [argument for argument in arguments if not argument.startswith("-")]
        if len(operands) >= 2:
            steps.append(OverwriteCandidate("rsync --inplace", operands[-1], directory, tuple(operands[:-1]), None))
    elif every_write and name == "rsync":
        operands = [argument for argument in arguments if not argument.startswith("-")]
        if len(operands) >= 2:
            steps.append(OverwriteCandidate("rsync", operands[-1], directory, tuple(operands[:-1]), None))
    elif name in COPY_SHORT_OPTIONS_WITH_VALUE:
        parsed = parse_copy_arguments(name, arguments)
        if parsed.creates_directories:
            return directory
        sources = tuple(parsed.operands if parsed.target_directory is not None else parsed.operands[:-1])
        # -b / --backup：⑤ ⑦ 当已有的目标不被盖掉；⑭ 当写（旧文件换了名字留底，这个名字照样是新写进去的）。弄坏开关 service-source-backup-kept 下 ⑭ 也当不盖掉
        keeps_target = parsed.keeps_existing_target or (parsed.makes_backup and not (service_writes and not detector_break_is_set("service-source-backup-kept")))
        if not keeps_target:
            if parsed.target_directory is not None:
                steps.append(OverwriteCandidate(name, parsed.target_directory, directory, sources, True, parsed.replaces_target_inode,
                                                links_sources=service_writes and parsed.links_hard))
            elif len(parsed.operands) >= 2:
                steps.append(OverwriteCandidate(name, parsed.operands[-1], directory, sources, False if parsed.treats_target_as_file else None,
                                                parsed.replaces_target_inode, links_sources=service_writes and parsed.links_hard))
        if name == "mv" and not (parsed.keeps_existing_target or parsed.makes_backup) and sources:
            steps.append(PreservingStep(sources, directory, "mv"))
        if name == "mv" and service_writes and sources and not detector_break_is_set("service-source-removal-unseen"):
            steps += [OverwriteCandidate("mv 挪走", source, directory) for source in sources]   # ⑭：挪走已有的服务源码，之后同名新建就是整份换掉
    elif service_writes and name == "ln" and not detector_break_is_set("service-source-link-unseen"):
        # ln：目标已有的文件被换成链接（-f、-b）；不带 -s 的是硬链接，源与目标成了同一个文件，往目标写就是往源写（links_sources，bash_scope_verdict 另判源）
        parsed = parse_copy_arguments(name, arguments)
        sources = tuple(parsed.operands if parsed.target_directory is not None else parsed.operands[:-1])
        if parsed.target_directory is not None:
            steps.append(OverwriteCandidate("ln", parsed.target_directory, directory, sources, True, True, links_sources=not parsed.links_symbolic))
        elif len(parsed.operands) >= 2:
            steps.append(OverwriteCandidate("ln", parsed.operands[-1], directory, sources, False if parsed.treats_target_as_file else None, True,
                                            links_sources=not parsed.links_symbolic))
    elif service_writes and name == "tar" and not detector_break_is_set("service-source-extract-unseen"):
        extraction_directory = tar_extraction_directory(arguments)
        if extraction_directory is not None:
            steps.append(OverwriteCandidate("tar -x", extraction_directory, directory))
    elif service_writes and name == "patch" and not detector_break_is_set("service-source-extract-unseen"):
        patched = patch_target(arguments)
        if patched is not None:
            steps.append(OverwriteCandidate(patched[0], patched[1], directory))
    elif name == "dd":
        steps += [OverwriteCandidate("dd", argument[len("of="):], directory) for argument in arguments if argument.startswith("of=")]
    elif name == "truncate":
        steps += [OverwriteCandidate("truncate", file, directory) for file in truncate_files(arguments)]
    elif service_writes and name == "unlink" and not detector_break_is_set("service-source-unlink-find-unseen"):
        steps += [OverwriteCandidate("unlink", word, directory) for word in plain_operands(arguments)]
    elif service_writes and name in COMPRESSOR_NAMES and not detector_break_is_set("service-source-compressor-unseen"):
        steps += [OverwriteCandidate(COMPRESSOR_FORM, word, directory) for word in compressor_removed_files(name, arguments)]
    elif service_writes and name == "shred" and not detector_break_is_set("service-source-in-place-tools-unseen"):
        steps += [OverwriteCandidate("shred", word, directory) for word in plain_operands(arguments, SHRED_SHORT_OPTIONS_WITH_VALUE, SHRED_LONG_OPTIONS_WITH_VALUE)]
    elif service_writes and name == "sort" and not detector_break_is_set("service-source-in-place-tools-unseen"):
        steps += [OverwriteCandidate("sort -o", word, directory) for word in sort_output_files(arguments)]
    elif service_writes and name in ("gawk", "awk") and not detector_break_is_set("service-source-in-place-tools-unseen"):
        steps += [OverwriteCandidate(f"{name} -i inplace", word, directory) for word in awk_in_place_files(arguments)]
    elif service_writes and name == "unzip" and not detector_break_is_set("service-source-unzip-unseen"):
        extraction_directory = unzip_extraction_directory(arguments)
        if extraction_directory is not None:
            steps.append(OverwriteCandidate("unzip", extraction_directory, directory))
    elif name == "rm":
        removed = tuple(argument for argument in arguments if not argument.startswith("-"))
        if removed:
            steps.append(PreservingStep(removed, directory, "rm"))
            if service_writes and not detector_break_is_set("service-source-removal-unseen"):
                steps += [OverwriteCandidate("rm", word, directory) for word in removed]   # ⑭：删掉已有的服务源码，之后同名新建就是整份换掉
    elif name == "git":
        added = git_added_paths(arguments)
        if added is not None:
            git_directory = directory
            for change in added[0]:
                git_directory = shell_words.changed_directory(git_directory, [change])
            steps.append(PreservingStep(tuple(added[1]), git_directory))
        applied = git_apply_directory(arguments) if service_writes and not detector_break_is_set("service-source-extract-unseen") else None
        if applied is not None:
            steps.append(OverwriteCandidate("git apply", applied, directory))
    elif name in shell_words.SHELL_NAMES:
        code = shell_words.shell_invocation(arguments).code_string
        if code is not None:
            steps += overwrite_steps(code, directory, variables, depth + 1, every_write, write_scripts, service_writes)
        elif heredoc_bodies and shell_words.heredoc_feeds_shell(shlex.join([name, *arguments])):
            # 管道喂进来的：`cat <<'EOF' | bash` 的正文被当数据剥掉了，bash 从标准输入读它，照命令逐行判
            steps += overwrite_steps(heredoc_bodies[-1].text, directory, variables, depth + 1, every_write, write_scripts, service_writes)
    return directory

def overwrite_steps(text, directory, variables=None, depth=0, every_write=False, write_scripts=False, service_writes=False):
    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换与进程替换）命令位置上，会整份覆盖文件的写法
    （OverwriteCandidate：重定向、tee、cp、mv、install、dd、truncate）与让旧字节留住的一步（PreservingStep），按执行的先后交回。
    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。
    every_write（⑨ 与 ⑫ 用）另交追加与别的改法：>>、&>>、tee -a、sed -i、perl -i、rsync 的目标。
    write_scripts（⑫ 用）另交 KNOWN_WRITE_SCRIPTS 那几个定点写脚本要改的文件（OverwriteCandidate），目标读不出的交 UnknownScriptTarget。
    service_writes（⑭ 用）另交 follow_simple_command 说的那几种：带 -b 的拷与挪、ln、mv 挪走、rm、tar -x、patch、git apply。"""
    if depth > shell_words.MAXIMUM_NESTING:
        return []
    variables = dict(variables or {})
    steps = []
    data_heredocs = []
    heredoc_text = text if detector_break_is_set("heredoc-body-judged-as-commands") else shell_words.strip_data_heredocs(text, stripped_bodies=data_heredocs)
    pieces, piped = command_pieces(shell_words.shell_tokens(heredoc_text))
    carried = ()   # 前一段经管道交出的文本
    for piece, after_pipe in zip(pieces, piped):
        for is_operator, substitution in piece:
            if is_operator is None:
                steps += overwrite_steps(substitution, directory, variables, depth + 1, every_write, write_scripts, service_writes)
        for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
            if (is_operator and (operator in OVERWRITING_REDIRECTS or (every_write and operator in APPENDING_REDIRECTS)) and next_is_operator is False
                    and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
                steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))
            elif (service_writes and is_operator and operator == READ_WRITE_REDIRECT and next_is_operator is False
                    and not detector_break_is_set("service-source-read-write-redirect-unseen")):
                steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))   # ⑭：`<>` 不截断、在原 inode 上从头改写
        commands, _, _, redirections = shell_words.simple_commands(piece)
        if commands and commands[0]:
            fed_heredocs = [data_heredocs[int(marker.group(1))] for operator, target in redirections[0] if operator in ("<<", "<<-")
                            for marker in [shell_words.HEREDOC_BODY_MARKER.match(target)] if marker and int(marker.group(1)) < len(data_heredocs)]
            stdin_bodies = fed_heredocs or (list(carried) if after_pipe and not detector_break_is_set("heredoc-pipe-fed-as-data") else [])
            directory = follow_simple_command(commands[0], directory, variables, steps, depth, stdin_bodies, every_write, write_scripts, service_writes)
            carried = piped_output_bodies(commands[0], stdin_bodies)
        else:
            carried = ()
    return steps

def results_overwrite_verdict(command, repository_root):
    """整条命令里会整份覆盖 research/results/ 下文件的每一步：目标确定、已存在、没进 git、前面也没被 git add 或 mv 挪走的，交进要拒的
    （`写法 相对仓库根的路径`）；目标这一刻算不出、样式对得上已有的未跟踪文件的，与 git 判不了的，交进要记检出的。"""
    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
    preserved, refused, noted = [], [], []
    candidates = []   # 按出现的次序：("exact", 写法, 目标) 或 ("pattern", 写法, 原词, 前面没被 git add / mv 留住的对得上的文件)；留住与否在步骤出现那一刻判
    for step in overwrite_steps(command, repository_root):
        base = step.directory if step.directory is not None else repository_root
        if isinstance(step, PreservingStep):
            if step.kind in RESULTS_PRESERVING_KINDS:
                preserved += preserved_paths(step, base)
            continue
        if isinstance(step, PythonCode):
            continue
        for word in destination_words(step, base):
            exact, matches = resolve_destination(word, base, repository_root)
            if exact is not None:
                if is_inside(exact, results_directory) and os.path.isfile(exact) and not is_preserved(exact, preserved):
                    candidates.append(("exact", step.form, exact))
                continue
            candidates.append(("pattern", step.form, word, [match for match in matches if not is_preserved(match, preserved)]))
    verdicts = tracked_verdicts(repository_root, [item[2] for item in candidates if item[0] == "exact"])   # 确定的目标一次查完
    for item in candidates:
        if item[0] == "exact":
            _, form, exact = item
            tracked = verdicts[exact]
            relative = os.path.relpath(exact, repository_root)
            if tracked is False:
                refused.append(f"{form} {relative}")
            elif tracked is None:
                noted.append(f"判不了 {relative} 进没进 git（git ls-files 出错），{form} 覆盖它这一步没拦")
        else:
            _, form, word, remaining = item
            untracked = untracked_among(repository_root, remaining)
            if untracked is None:
                noted.append(f"判不了 {form} {word} 对得上的文件进没进 git（git ls-files 出错），没拦")
            elif untracked:
                shown = "、".join(os.path.relpath(path, repository_root) for path in untracked[:3])
                noted.append(f"{form} {word} 的目标这一刻算不出来（变量、命令替换或通配），没拦；"
                             f"它对得上 research/results/ 下 {len(untracked)} 份已有的未跟踪文件：{shown}{' 等' if len(untracked) > 3 else ''}")
    return list(dict.fromkeys(refused)), list(dict.fromkeys(noted))

def results_overwrite_refusal(command, repository_root):
    """交回 (要拒的写法与目标, 要记的检出)；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
    if (detector_break_is_set("disable-check")
            or detector_break_is_set("allow-results-overwrite")):
        return [], []
    return results_overwrite_verdict(command, repository_root)
