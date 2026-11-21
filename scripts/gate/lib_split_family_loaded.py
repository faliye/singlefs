#!/usr/bin/env python3
# admission: always 读的是调用方这一次给的入口与候选文件此刻在盘上的内容，不存结论
# run-condition: none 只用 python 标准库与同目录的 lib_lag_split_family.py，读文件、不起子进程
"""拆分族：一份入口连同它拆出去、并且真载入的部分文件。仓里认「入口加它拆出的部分文件」只用这一份。

族取两样的交集：
  命名：路径落在 lib_lag_split_family.in_family(入口, 路径) 认的命名族里（认法写在那一份的文件头：<模块名>_<部分>.py、<名>-<部分>.sh、<名>-<部分>.py 与门禁阶段 lib/ 下的 <名>-<部分>.sh|py）；
    交给它之前入口与候选先换成同一个仓内相对路径（见下面「路径」）；
  载入：入口、或已经在族里的某个文件真载入它，载入的写法只认下面几种。
两样缺一不算族：
  只按名字像的不算：名字像部分文件而入口没载入的（别的钩子单独导入的、留在目录里没人用的）不是它拆出来的；
  只被载入的不算：入口 import、source 的几份脚本共用的库，与字符串里点名的别的脚本，不是按入口的拆法命名的，各有自己的族。
族里的文件再载入的也算，一直追到不再多出新的为止（族是命名族的子集，追得完）。入口自己在最前，别的按载入的先后。

载入的几种写法（loaded_file_names(文本, 后缀)，交文件名，不看目录；在不在盘上、在不在命名族里由 loaded_family 看；后缀 .py 与 .sh 各按自己的写法认，别的后缀交空集）：
  python（.py）按 ast 认，注释、字符串、文档串、print 与报错说明里写的不算；整份文本语法错、含空字节的当没有载入任何东西：
    `import <模块>[, <模块>…]` 与 `from <模块> import …`：点号前面那一段模块名加 .py（缩进里、函数里的也算）；
    同目录的相对 import：`from . import <模块>[, …]` 与 `from .<模块> import …`（点号多于一个的是上层目录，不算）；
    字面量参数的 `importlib.import_module("<模块>")`（写成 ".<模块>" 的是同目录的相对写法；参数不是字面量的认不出）；
    赋给名字以 PART_NAMES 收尾的模块层元组或列表里写死的 `<名>.py` 与 `<名>.sh`：按文件名 exec、载入的部分文件
      （例：lib_project_code_runs.py 的 PROJECT_CODE_RUNS_PART_NAMES、lib_run_scope.py 的 RUN_SCOPE_PART_NAMES）；
    按路径载入的调用里写死的文件名：载入调用是 `spec_from_file_location`（路径是第二个参数或 location=）与 `runpy.run_path`（第一个参数或 path_name=），
      按名字认、不核它从哪个模块来；同一个作用域（函数或模块层）里有 `exec(…)` 时，那里 `open(…)` 的路径参数（先 open 再 exec）同样算载入调用。
      路径参数里认字面量 `<名>.py` 与 `<名>.sh`：字面量本身，与 os.path.join、joinpath、Path(…)、pathlib 的 `/`、str(…)、resolve() 这些拼路径的写法里的字面量段；
      路径参数里是变量的，追三种：同一个作用域里赋过的 `<名> = <含字面量的路径表达式>`；
      这个函数的参数进了载入调用的路径时，同一份文件里按函数名调它的地方在那个参数位置（或同名关键字）写的字面量；
      模块层的 `<名> = <含字面量的路径表达式>` 常量，族里任何一份文件的载入调用路径参数用到这个名字（名字对名字，不核 import）；
      别的追法（经返回值、属性、容器、再转一手的参数）认不出。注释、文档串、print、报错说明与没进载入调用的引号文件名都不算；
  shell（.sh）：先去掉注释（# 起头、前面是行首、空白或 ; & | ( 的），再认 `source` 与 `.` 后面写死的 `<名>.sh`（目录段是变量或没有，带 `..` 的是别的目录、不算）；
    命令行里有 python 的 heredoc（`<<'标记'`、`<<标记`、`<<-标记`，前面可带文件描述符）的正文当一份 python 文本，照上面 python 的几种认
      （例：prove-red.sh 与 e165-replay-mutation-kill-bounds.sh 把本体拆在同目录的 python 模块里、内嵌 python 只留载入器）；
    正文语法错的，只认行首顶格、写在一行里的 `<名>PART_NAMES = (…)` 与 `<名>PART_NAMES = […]`（钩子入口 .sh 的写法，例：write-guard.sh 的 WRITE_GUARD_PART_NAMES）；
  别的载入写法（按拼出来的名字载入、通配、__import__、拆成两步传的路径）认不出，那样载入的文件不在族里。

路径：载入那一半读文件，用调用方给的路径；命名那一半要仓内相对路径（门禁阶段的命名族认 .claude/gate.d/<名>.sh 与 .claude/gate.d/lib/<名>-<部分>.sh）：
  入口是绝对路径时，入口里最后一处 `/.claude/gate.d/` 之前的部分当仓根，入口与落在这个仓根下的候选都去掉它再比；相对路径、不含这一段的入口，与不在这个仓根下的候选，照原样交。

读不了的文件当没有载入任何东西（族只会变小）。族小了对各调用方的影响：
  source_family（要临时服务的自证登记核对）：记号找不到，多报「源码里没有核对」；
  agent-write-scope 格：查函数名那一侧取到的函数名少，少报手抄的函数；跳过被判仓自己那一族的那一侧跳过的文件少，多报；
  queue-health 的 entry_with_split_family：读到的源码少，核对的写法找不到，自证多红。

怎么用（路径按调用方手里有的写，入口与候选用同一种写法：都是绝对路径，或都是相对同一个根的路径；不读 sys.path，不按工作目录找）：
  loaded_family(入口路径, 候选路径们)        入口连同它的拆分族，交路径列表，入口在最前；候选由调用方给（sibling_files 列入口同目录的 .py 与 .sh）
  sibling_files(入口路径)                    入口同目录里的 .py 与 .sh 文件的路径，按文件名排序；目录读不了交 []
  loaded_file_names(文本, 后缀)              一份文本里载入的文件名的集合（模块层常量与用到它的载入调用在同一份文本里的也算）
  read_loaded_file_names(路径)               一份文件载入的文件名的集合（按路径的后缀认）；读不了交空集

调它的（各自留自己的弄坏开关，认族只在这一份）：
  .claude/hooks/lib_project_code_runs_launchers.py 的 source_family（要临时服务的自证登记核对读「入口与它的拆分族里有没有临时服务核对」）；
  .claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh 的 agent-write-scope 格（共用切词模块与共用项目代码判定模块的族，函数名取自族、族外的钩子文件不许再 def 一份）；
  scripts/hosts/queue-health.py 的 entry_with_split_family（任务服务的源码读入口加它的拆分族）。
这一份没有单独的自证：只按名字认、只按载入认、按文本行认载入、绝对路径不换、读到整个目录、只读入口，各由那三份的自证与它们的弄坏开关证；
  认法本身在真仓与合成布局上的预期，由新旧两版对比的探针逐格核（探针不进仓）。

弄坏开关的接口（只给调用方自己的弄坏开关用，平常传空）：defects 是下面几项的集合，各由调用方自己的开关名映射过来：
  name-only       只按名字认：命名族里的文件都算，不看入口载没载入
  loaded-only     只按载入认：载入的同目录文件都算，不看名字像不像部分文件
  text-scan       载入按文本行认：注释、字符串、文档串里的 import 行与引号里的 <名>.py、<名>.sh 都算载入，`from . import`、`import_module` 认不出
  absolute-paths  命名那一半不换仓内相对路径：门禁阶段入口给绝对路径时，lib/ 下的部分文件认不出
"""
import ast
import importlib.util
import os
import re
from collections import namedtuple

NAME_ONLY = "name-only"
LOADED_ONLY = "loaded-only"
TEXT_SCAN = "text-scan"
ABSOLUTE_PATHS = "absolute-paths"
KNOWN_DEFECTS = (NAME_ONLY, LOADED_ONLY, TEXT_SCAN, ABSOLUTE_PATHS)

MAXIMUM_SCAN_BYTES = 16 << 20   # 一份文件读到这么多字节为止；脚本都在这以内，超过的后半截里的载入认不出
SCRIPT_SUFFIXES = (".py", ".sh")
NAMING_LIBRARY_FILE = "lib_lag_split_family.py"
PART_NAMES_SUFFIX = "PART_NAMES"

# 载入调用的名字 → 路径参数的（位置，关键字）
LOADER_CALL_PATH_ARGUMENTS = {"spec_from_file_location": (1, "location"), "run_path": (0, "path_name")}
PATH_BUILDER_CALLS = frozenset({"join", "joinpath", "Path", "PurePath", "with_name"})   # 拼路径：参数里的字面量段都算
PATH_PASSTHROUGH_CALLS = frozenset({"str", "fspath", "abspath", "realpath", "normpath", "resolve", "absolute", "as_posix", "expanduser"})   # 套在外面的：里面的算
FUNCTION_NODES = (ast.FunctionDef, ast.AsyncFunctionDef)

MODULE_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
SCRIPT_FILE_NAME = re.compile(r"[\w.-]+\.(?:py|sh)")
SHELL_COMMENT = re.compile(r"(^|[ \t;&|(])#[^\n]*", re.MULTILINE)
SHELL_SOURCE_LINE = re.compile(r"""(?:^|[;&|(\s])(?:source|\.)[ \t]+["']?(\$\(dirname "\$\{BASH_SOURCE\[0\]\}"\)/|[^\s"';&|)]*/)?([\w.-]+\.sh)["']?(?=[\s;&|)]|$)""", re.MULTILINE)   # 目录段是变量、没有，或同目录的 $(dirname "${BASH_SOURCE[0]}")
SHELL_PART_NAMES_LINE = re.compile(r"^([A-Z0-9_]*" + PART_NAMES_SUFFIX + r")[ \t]*=[ \t]*([(\[][^\n#]*[)\]])", re.MULTILINE)
HEREDOC_START = re.compile(r"""(?<!<)<<(-?)[ \t]*(?:'([^'\n]+)'|"([^"\n]+)"|\\?([A-Za-z_][A-Za-z0-9_]*))""")
PYTHON_COMMAND = re.compile(r"(?<![\w-])python[0-9.]*(?![\w-])")
TEXT_SCAN_IMPORT_LINE = re.compile(r"^[ \t]*import[ \t]+([^\n#;]+)", re.MULTILINE)
TEXT_SCAN_FROM_IMPORT_LINE = re.compile(r"^[ \t]*from[ \t]+([A-Za-z_][A-Za-z0-9_]*)[ \t]+import\b", re.MULTILINE)
TEXT_SCAN_QUOTED_SCRIPT_NAME = re.compile(r"""["']([\w.-]+\.(?:py|sh))["']""")

# 一份文件的载入：names 是它直接载入的文件名；constants 是它模块层 `<名> = <含字面量的路径表达式>` 的 {名: {文件名}}；uses 是它的载入调用路径参数里没追到来源的变量名
FileLoads = namedtuple("FileLoads", ["names", "constants", "uses"])
NO_LOADS = FileLoads(frozenset(), {}, frozenset())


def load_naming_library():
    """同目录的 lib_lag_split_family.py（命名族的认法只在那一份里写）：按这一份自己所在的目录载入，不经 sys.path。"""
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), NAMING_LIBRARY_FILE)
    specification = importlib.util.spec_from_file_location("lib_lag_split_family_for_split_family_loaded", path)
    if specification is None or specification.loader is None:
        raise ImportError(f"载不进命名族的认法 {path}")
    library = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(library)
    return library


naming = load_naming_library()
GATE_STAGE_MARKER = "/" + naming.GATE_STAGE_DIRECTORY + "/"


def module_file_name(dotted_name):
    """点号写法的模块名 → 它的顶层模块对应的文件名（<模块>.py）；写法认不出（空的、含不是标识符的段）交 None。"""
    top = dotted_name.split(".")[0]
    return f"{top}.py" if MODULE_NAME.fullmatch(top) else None


def is_import_module_function(function):
    """调用的函数是不是 importlib.import_module，或 from importlib import import_module 之后的 import_module。"""
    if isinstance(function, ast.Attribute):
        return function.attr == "import_module" and isinstance(function.value, ast.Name) and function.value.id == "importlib"
    return isinstance(function, ast.Name) and function.id == "import_module"


def imported_module_names(node):
    """一个语法树结点载入的模块名（点号写法）：import、同目录之内的 from … import、字面量参数的 import_module；别的结点交空表。"""
    if isinstance(node, ast.Import):
        return [alias.name for alias in node.names]
    if isinstance(node, ast.ImportFrom):
        if node.level == 0 and node.module:
            return [node.module]
        if node.level == 1:
            return [node.module] if node.module else [alias.name for alias in node.names]
        return []
    if isinstance(node, ast.Call) and is_import_module_function(node.func) and node.args:
        argument = node.args[0]
        if isinstance(argument, ast.Constant) and isinstance(argument.value, str):
            name = argument.value
            if name.startswith(".") and not name.startswith(".."):
                return [name[1:]]
            return [name]
    return []


def part_file_names(tree):
    """模块层赋给名字以 PART_NAMES 收尾的元组或列表里写死的 <名>.py 与 <名>.sh。"""
    found = set()
    for node in tree.body:
        if isinstance(node, ast.Assign):
            targets, value = node.targets, node.value
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            targets, value = [node.target], node.value
        else:
            continue
        if not any(isinstance(target, ast.Name) and target.id.endswith(PART_NAMES_SUFFIX) for target in targets):
            continue
        if isinstance(value, (ast.Tuple, ast.List)):
            found.update(element.value for element in value.elts
                         if isinstance(element, ast.Constant) and isinstance(element.value, str) and SCRIPT_FILE_NAME.fullmatch(element.value))
    return found


def call_name(function):
    """被调用的东西的末一个名字：`a.b.c(…)` 是 c，`c(…)` 是 c，别的交 None。"""
    if isinstance(function, ast.Attribute):
        return function.attr
    return function.id if isinstance(function, ast.Name) else None


def literal_script_names(expression):
    """路径表达式里写死的 <名>.py 与 <名>.sh（文件头「按路径载入的调用」）：字面量本身，与拼路径、套在外面的几种写法里的字面量段。"""
    if isinstance(expression, ast.Constant):
        value = expression.value
        return {value} if isinstance(value, str) and SCRIPT_FILE_NAME.fullmatch(value) else set()
    if isinstance(expression, ast.BinOp) and isinstance(expression.op, ast.Div):
        return literal_script_names(expression.left) | literal_script_names(expression.right)
    if isinstance(expression, ast.Call):
        name = call_name(expression.func)
        operands = []
        if name in PATH_BUILDER_CALLS:
            operands = list(expression.args)
        elif name in PATH_PASSTHROUGH_CALLS:
            operands = list(expression.args) or ([expression.func.value] if isinstance(expression.func, ast.Attribute) else [])
        found = set()
        for operand in operands:
            found |= literal_script_names(operand)
        return found
    return set()


def argument_of(call, position, keyword):
    """调用里第 position 个位置参数（没有就取关键字 keyword 的那个）的表达式；位置参数里有 * 展开的、没有这一个参数的交 None。"""
    if position is not None and len(call.args) > position and not any(isinstance(argument, ast.Starred) for argument in call.args[:position + 1]):
        return call.args[position]
    return next((entry.value for entry in call.keywords if entry.arg == keyword), None)


def loader_path_argument(call, opens_for_exec):
    """载入调用的路径参数的表达式：spec_from_file_location、run_path，与（作用域里有 exec 时的）open；不是载入调用或没写路径参数交 None。"""
    name = call_name(call.func)
    if name in LOADER_CALL_PATH_ARGUMENTS:
        return argument_of(call, *LOADER_CALL_PATH_ARGUMENTS[name])
    if opens_for_exec and isinstance(call.func, ast.Name) and call.func.id == "open":
        return argument_of(call, 0, "file")
    return None


def scope_nodes(body):
    """一个作用域里的语法树结点：不进嵌套的函数与 lambda（它们各是自己的作用域），类体算在外面那一层里。"""
    pending, nodes = list(body), []
    while pending:
        node = pending.pop()
        nodes.append(node)
        if not isinstance(node, FUNCTION_NODES + (ast.Lambda,)):
            pending.extend(ast.iter_child_nodes(node))
    return nodes


def assigned_script_names(nodes):
    """作用域里 `<名> = <含字面量的路径表达式>` 赋过的 {名: {文件名}}。"""
    assigned = {}
    for node in nodes:
        if isinstance(node, ast.Assign):
            targets, value = node.targets, node.value
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            targets, value = [node.target], node.value
        else:
            continue
        names = literal_script_names(value)
        if names:
            for target in targets:
                if isinstance(target, ast.Name):
                    assigned.setdefault(target.id, set()).update(names)
    return assigned


def scope_loads(nodes, parameters):
    """一个作用域里载入调用的路径参数：(直接写死或同作用域赋过的文件名, 没追到来源的变量名, 进了路径的参数名, 作用域里赋过的 {名: {文件名}})。"""
    assigned = assigned_script_names(nodes)
    opens_for_exec = any(isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "exec" for node in nodes)
    names, uses, used_parameters = set(), set(), set()
    for node in nodes:
        argument = loader_path_argument(node, opens_for_exec) if isinstance(node, ast.Call) else None
        if argument is None:
            continue
        names |= literal_script_names(argument)
        for variable in {inner.id for inner in ast.walk(argument) if isinstance(inner, ast.Name)}:
            if variable in assigned:
                names |= assigned[variable]
            elif variable in parameters:
                used_parameters.add(variable)
            else:
                uses.add(variable)
    return names, uses, used_parameters, assigned


def python_file_loads(text):
    """python 文本载入的（文件头「载入的几种写法」）；语法错、含空字节的交没有载入。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError, RecursionError):
        return NO_LOADS
    names = {module_file_name(module) for node in ast.walk(tree) for module in imported_module_names(node)}
    names.discard(None)
    names |= part_file_names(tree)
    uses, loader_parameters = set(), {}
    module_names, module_uses, _parameters, constants = scope_loads(scope_nodes(tree.body), ())
    names |= module_names
    uses |= module_uses
    for function in (node for node in ast.walk(tree) if isinstance(node, FUNCTION_NODES)):
        positional = [argument.arg for argument in function.args.posonlyargs + function.args.args]
        parameters = positional + [argument.arg for argument in function.args.kwonlyargs]
        function_names, function_uses, used_parameters, _assigned = scope_loads(scope_nodes(function.body), parameters)
        names |= function_names
        uses |= function_uses
        loader_parameters.setdefault(function.name, []).extend((positional.index(name) if name in positional else None, name) for name in sorted(used_parameters))
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            for position, keyword in loader_parameters.get(node.func.id, ()):
                argument = argument_of(node, position, keyword)
                if argument is not None:
                    names |= literal_script_names(argument)
    return FileLoads(names, constants, uses)


def heredoc_bodies(text):
    """shell 文本里的 heredoc：[(起这个 heredoc 的那一行, 正文)]，次序照文本；注释行里的 << 不算，没等到结束标记的正文取到文末。"""
    lines, found, index = text.split("\n"), [], 0
    while index < len(lines):
        line = lines[index]
        index += 1
        if line.lstrip().startswith("#"):
            continue
        for match in HEREDOC_START.finditer(line):
            strip_tabs, tag = bool(match.group(1)), match.group(2) or match.group(3) or match.group(4)
            body = []
            while index < len(lines):
                current = lines[index]
                index += 1
                if (current.lstrip("\t") if strip_tabs else current) == tag:
                    break
                body.append(current)
            found.append((line, "\n".join(body)))
    return found


def shell_file_loads(text):
    """shell 文本载入的（文件头「载入的几种写法」）：去掉注释之后的 source 与 .，命令行里有 python 的 heredoc 的正文按 python 认，
    正文语法错时认行首顶格的 PART_NAMES 一行。"""
    names, constants, uses = set(), {}, set()
    for match in SHELL_SOURCE_LINE.finditer(SHELL_COMMENT.sub(r"\1", text)):
        if ".." not in (match.group(1) or "").split("/"):
            names.add(match.group(2))
    for match in SHELL_PART_NAMES_LINE.finditer(text):
        try:
            names |= part_file_names(ast.parse(f"{match.group(1)} = {match.group(2)}"))
        except (SyntaxError, ValueError):
            continue
    for command, body in heredoc_bodies(text):
        if not PYTHON_COMMAND.search(command):
            continue
        loads = python_file_loads(body)
        names |= loads.names
        uses |= loads.uses
        for name, files in loads.constants.items():
            constants.setdefault(name, set()).update(files)
    return FileLoads(names, constants, uses)


def text_scanned_loads(text):
    """弄坏开关 text-scan 的认法：不分后缀，按文本行认 import、from … import、引号里的 <名>.py 与 <名>.sh、shell 的 source。"""
    names = set()
    for match in TEXT_SCAN_IMPORT_LINE.finditer(text):
        for piece in match.group(1).split(","):
            words = piece.split()
            module = words[0].split(".")[0] if words else ""
            if MODULE_NAME.fullmatch(module):
                names.add(f"{module}.py")
    for match in TEXT_SCAN_FROM_IMPORT_LINE.finditer(text):
        names.add(f"{match.group(1)}.py")
    for match in TEXT_SCAN_QUOTED_SCRIPT_NAME.finditer(text):
        names.add(match.group(1))
    for match in SHELL_SOURCE_LINE.finditer(text):
        if ".." not in (match.group(1) or "").split("/"):
            names.add(match.group(2))
    return FileLoads(names, {}, frozenset())


def file_loads(text, suffix, defects=frozenset()):
    """一份文本的载入（FileLoads）；suffix 是这份文本所在文件的后缀（.py 或 .sh），别的后缀交没有载入。"""
    if TEXT_SCAN in defects:
        return text_scanned_loads(text)
    if suffix == ".py":
        return python_file_loads(text)
    if suffix == ".sh":
        return shell_file_loads(text)
    return NO_LOADS


def resolved_names(loads):
    """一份文件自己载入的文件名：直接的，加它自己的模块层常量里被它自己的载入调用用到的。"""
    return set(loads.names) | {name for variable in loads.uses for name in loads.constants.get(variable, ())}


def loaded_file_names(text, suffix, defects=frozenset()):
    """一份文本里载入的文件名（文件头「载入的几种写法」）：{文件名}；suffix 是这份文本所在文件的后缀（.py 或 .sh），别的后缀交空集。"""
    return resolved_names(file_loads(text, suffix, defects))


def read_file_loads(path, defects=frozenset()):
    """path 这一份文件的载入（FileLoads）；读不了交没有载入。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read(MAXIMUM_SCAN_BYTES)
    except OSError:
        return NO_LOADS
    return file_loads(text, os.path.splitext(path)[1], defects)


def read_loaded_file_names(path, defects=frozenset()):
    """path 这一份文件载入的文件名；读不了交空集。"""
    return resolved_names(read_file_loads(path, defects))


def sibling_files(entry):
    """入口同目录里的 .py 与 .sh 文件的路径（按入口的写法接目录），按文件名排序；目录读不了交 []。"""
    directory = os.path.dirname(entry)
    try:
        names = sorted(os.listdir(directory or "."))
    except OSError:
        return []
    return [os.path.join(directory, name) for name in names if name.endswith(SCRIPT_SUFFIXES) and os.path.isfile(os.path.join(directory or ".", name))]


def repository_relative(entry_key, candidate_key, defects):
    """交给命名族认法的（入口, 候选）：入口是绝对路径时，最后一处 /.claude/gate.d/ 之前的部分当仓根去掉；弄坏开关 absolute-paths 下照原样交。"""
    if ABSOLUTE_PATHS in defects or not os.path.isabs(entry_key):
        return entry_key, candidate_key
    index = entry_key.rfind(GATE_STAGE_MARKER)
    if index < 0:
        return entry_key, candidate_key
    root = entry_key[:index + 1]
    return entry_key[len(root):], candidate_key[len(root):] if candidate_key.startswith(root) else candidate_key


def in_naming_family(entry_key, candidate_key, defects):
    """候选是不是入口按命名认的族里的文件（路径先换成仓内相对路径）。"""
    return naming.in_family(*repository_relative(entry_key, candidate_key, defects))


def loaded_family(entry, candidates, defects=frozenset()):
    """入口 entry 连同它的拆分族：[entry] 加 candidates 里（命名族里的、被入口或族里别的文件载入的）文件，入口在最前，别的按载入的先后；路径照 os.path.normpath 写，入口照给的样子。
    族里各份文件的模块层路径常量与它们载入调用里用到的变量名合在一起对（文件头「按路径载入的调用」）。
    弄坏开关 name-only 下取命名族里的全部候选，loaded-only 下取被载入的全部候选（不看名字），text-scan 下载入按文本行认，absolute-paths 下命名那一半不换仓内相对路径。"""
    entry_key = os.path.normpath(entry)
    pool = []
    for candidate in candidates:
        key = os.path.normpath(candidate)
        if key != entry_key and key not in pool:
            pool.append(key)
    if NAME_ONLY in defects:
        return [entry] + [candidate for candidate in pool if in_naming_family(entry_key, candidate, defects)]
    members, pending, seen = [entry], [entry], {entry_key}
    direct, constants, uses = set(), {}, set()
    while pending:
        loads = read_file_loads(pending.pop(0), defects)
        direct |= loads.names
        uses |= loads.uses
        for name, files in loads.constants.items():
            constants.setdefault(name, set()).update(files)
        loaded = direct | {name for variable in uses for name in constants.get(variable, ())}
        for candidate in pool:
            if candidate in seen or os.path.basename(candidate) not in loaded:
                continue
            if LOADED_ONLY not in defects and not in_naming_family(entry_key, candidate, defects):
                continue
            seen.add(candidate)
            members.append(candidate)
            pending.append(candidate)
    return members
