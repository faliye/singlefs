"""调用图收闭包：ScriptClosure（Python 与 shell 单元沿名字、import、子进程边、数据读收闭包）、入口写法、Options、closure_of、工具链询问。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import collections
import importlib.util
import itertools
import os
import posixpath
import re
import subprocess
from lib_function_units_lexer import active_breaks
from lib_function_units_base import (
    DEFAULT_SCRIPT_DIRECTORIES, FORMAL_VERIFICATION, FunctionUnitsError, JUDGMENT_KINDS, LIBRARY_ROOT, PYTHON_INTERPRETERS, SHELL_INTERPRETERS,
    SHELL_OPEN_TAIL, Unit, UsageError, data_directory_unit, data_file_unit, declared_checks, external_program_name, sha256_text,
)
from lib_function_units_registry import (
    BUILT_TOOL_SOURCES, CALLER_SUPPLIED_COMMANDS, EXCLUDED_CALL_EDGES, EXCLUDED_PROGRAM_EDGES, IMPORT_TIME_CLASS_RULE, IMPORT_TIME_RULE,
    registered_function_name, root_file_ignored, root_gitignore_rules,
)
from lib_function_units_python_facts import (
    ARGV_BINDING, CALLER_INPUT_ORIGINS, DATA_FORM, ElementsOf, IMPORT_TIME_BINDING_VIEW, KnownWords, SHALLOW_IMPORT_VIEW, TOOLCHAIN_QUERY_SECONDS,
    TOOLCHAIN_UNIT_KIND, enclosing_unit, function_local_names, named_calls_in, partial_bound_call, walk_nodes, walk_nodes_skipping,
)
from lib_function_units_shell_text import SHELL_ROW_SELECTED_TABLES, table_drivers_of_unselected_rows, table_row_selection, written_only_as_table_row
from lib_function_units_shell_index import ShellIndex, inline_shell_units, language_of_file, shell_files_under
from lib_function_units_python_module import PythonIndex
from lib_function_units_rust_workspace import Closure, RustWorkspace
from lib_function_units_wgsl import wgsl_closure, wgsl_file
from lib_function_units_rust_closure import WgslNameSources, rust_closure_of


# 闭包 run() 的轮数上限（收完一轮、认调用方交进来的命令、折出新程序再收一轮）：超了抛 FunctionUnitsError 点名入口，不一直转
CLOSURE_ROUND_LIMIT = 64
# 并行之前父进程建好的脚本索引：(根的绝对路径, under) → ScriptClosure；closure_of 取它的 python 与 shell（prepare_shared_script_index）
SHARED_SCRIPT_INDICES = {}
# 弄坏开关 data-units-process-wide 下数据单元的单元表：(根的绝对路径, data_units 或 data_directories) → {路径: 单元}
PROCESS_WIDE_DATA_UNITS = {}


def default_script_directories(root):
    return [place for place in DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(root, place))]


class ScriptClosure:
    def __init__(self, root, under, excluded_files=()):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.python = PythonIndex(self.root)
        self.python.entry_inline_scripts = self.entry_inline_scripts
        self.under = list(under)
        self.shell = None
        self.inline_scripts_of_entry = {}   # entry_inline_scripts 的缓存：{shell 脚本: [InlineScript]}
        self.closure = Closure()
        # 按角色排除的文件（文件头「按角色排除的文件」）：单元不进闭包，也不从它们往下连
        self.excluded_files = frozenset(excluded_files) if "excluded-files-followed" not in self.breaks else frozenset()
        self.closure.excluded_files = self.excluded_files
        # 仓根被忽略的私有文件（文件头「仓根被忽略的私有文件」）：仓根 .gitignore 里落得到仓根这一层的那几条，头一次问时读（private_root_file）
        self.root_ignore_rules = None
        self.named_classes, self.attributes_seen, self.loaded_modules, self.modules_seen = set(), set(), set(), set()
        # 按路径载入、模块值出不了载入方的（文件头「按路径载入的模块只连载入方调到的属性」）：{载入方模块的路径: 载入的文件} 与 {模块的路径: 它的单元上见过的属性名}
        self.contained_loads, self.attributes_by_module = collections.defaultdict(set), collections.defaultdict(set)
        # contained_loads 与 loaded_modules 里的名字 → 那份模块：按路径载入的是相对根的文件，exec 进模块对象的内联正文是 <模块的路径>#<正文的 sha256>（connect_loaded）
        self.loaded_objects = {}
        self.import_time_classes = set()   # 只当类头收进来（IMPORT_TIME_CLASS_RULE）、还没被点名的类头：之后被点名时再连方法
        self.methods_by_name = collections.defaultdict(list)
        self.closure_sourcers, self.processed_shell = collections.defaultdict(set), collections.defaultdict(list)
        # 按名字连进来的函数所在的文件 → 连它的单元所在的那几份（visible_files：被叫到的库照叫它的那一份看得见的文件找函数与变量）
        self.closure_name_reachers = collections.defaultdict(set)
        self.todo = []
        # 内联脚本切出来的单元：穿过它们连下去，自己不进闭包（正文已在起它的那个单元里）
        self.inline_seen, self.inline_units = set(), {}
        # 程序入口带参数（文件头「程序入口带参数」）：给了参数才剪；收过边的 (单元 id, 视图)（视图 None 是整份，盖住别的视图）
        self.narrowing, self.views_done = False, set()
        # 带着已知的词起的 python 程序（文件头「shell 起 python」）：这几份模块的 import 时赋值照「给了参数」收
        self.narrowed_programs = set()
        # 按第一列选行的表（文件头「shell 起法带字面参数」）：{脚本: 按实验号起过的那几组词}，整份起过的是 None、没有这一项的也照整张表，只起过不定义表的那几种（table_launched）是空集合、读表一行都不连；读表的每一处 {脚本: [(表的变量单元, 来处, 规则名)]}
        self.table_selections, self.table_references = {}, {}
        # 表行之外写到的驱动函数（table_unselected_drivers）：{(脚本, 这几次起法的词): 只写在这几次起法都跑不到的那几行的 @驱动函数}
        self.unselected_drivers = {}
        # 运行期读的文件里本模块没有调用处的参数 [(函数单元, 参数名, 读它的模块, 读它的单元, 行, 是不是拷文件的源)] 与调用方交进来的命令 [(拒算, 出处)]：收完闭包再判
        self.pending_parameters, self.pending_launches = [], []
        # 读 <参数>/<字面尾巴> 的每一处 [(取法, 读它的单元)] 与已经照尾巴收过的那几处（下标）：每收完一轮闭包按调用处重认一遍
        self.tail_reads, self.tail_reads_collected = [], set()
        # 闭包里按名字认的调用处（closure_call_arguments）：{被调的名字: [(调用节点, 所在单元)]}；新收的（单元, 视图）记在 pending，用的时候并进来，不整份重建
        self.calls_by_name = collections.defaultdict(list)
        self.call_index_pending, self.call_nodes_indexed = [], set()
        # call_reaches 问过的每一层 def 自己绑的名字：{id(节点): (节点, 局部名字, global 与 nonlocal 声明过的)}
        self.scope_names = {}

    def shell_index(self, extra=()):
        if self.shell is None:
            self.shell = ShellIndex(self.root, shell_files_under(self.root, self.under) + list(extra))
        elif extra:
            self.shell.add(list(extra))
        return self.shell

    def entry_inline_scripts(self, path):
        """shell 脚本 path（相对根）的脚本体与每个函数起的内联 python（python-inline 边的 InlineScript），按单元在文件里的次序：
        PythonModule.inherited_environment 照它们认入口给拆出的模块设的环境变量（算键库文件头「拆出的模块继承入口给内联 python 设的环境变量」）。
        问的时候别的单元的折叠可能正开着（ShellIndex.overlays 是那一趟的临时赋值），取边时先放下、取完放回。"""
        if path not in self.inline_scripts_of_entry:
            self.inline_scripts_of_entry[path] = scripts = []
            index = self.shell_index([path])
            shell_file = index.files.get(path)
            if shell_file is not None:
                saved, index.overlays = index.overlays, []
                try:
                    for unit in shell_file.units:
                        if unit.kind in ("script-body", "function"):
                            scripts += [script for kind, found, _refusal in index.links(unit) if kind == "python-inline" for script in found]
                finally:
                    index.overlays = saved
        return self.inline_scripts_of_entry[path]

    def push(self, unit, via, rule, view=None):
        """view：None 整份收；frozenset 的已知值表是程序入口带参数剪过的那一种；SHALLOW_IMPORT_VIEW 是给了参数时 import 时执行的赋值（文件头「程序入口带参数」）。"""
        if unit is None or unit.file in self.excluded_files:
            return
        if id(unit) in self.closure.members and self.view_done(unit, view):
            self.named_after_import_time(unit, rule)
        else:
            self.todo.append((unit, via, rule, view))

    def view_done(self, unit, view):
        return (id(unit), None) in self.views_done or (id(unit), view) in self.views_done

    def named_after_import_time(self, unit, rule):
        """先只当类头收进来（import 时执行）的类头，之后被点名（别的规则又推到它）：照被点名的类连方法。"""
        if unit.kind == "class" and rule != IMPORT_TIME_CLASS_RULE and id(unit) in self.import_time_classes:
            self.import_time_classes.discard(id(unit))
            self.class_entered(unit)

    def private_root_file(self, file):
        """file（相对根）在仓根这一层（不带 /）、被仓根那一份 .gitignore 挡着（文件头「仓根被忽略的私有文件」）：不当程序连、不当 source 读。
        弄坏开关 root-ignored-scripts-linked 下一律交 False（照连）。"""
        if "/" in file or "root-ignored-scripts-linked" in self.breaks:
            return False
        if self.root_ignore_rules is None:
            self.root_ignore_rules = root_gitignore_rules(self.root)
        return root_file_ignored(self.root_ignore_rules, file)

    def push_program(self, file, via, launched=None, shell_arguments=None):
        """launched：程序入口带参数时起它的那几处带过去的参数（KnownWords 的集合）；None 是不知道、整份收（文件头「程序入口带参数」子进程边）。
        shell_arguments：shell 起 shell 的每一处带的字面参数（{(词…)}，ShellIndex.shell_launch_arguments）；None 是有一处不知道，脚本体整份收。
        仓根被 .gitignore 挡着的私有文件不连（private_root_file）。"""
        if file in self.excluded_files or self.private_root_file(file):
            return
        language = language_of_file(self.root, file)
        if language == "python":
            module = self.python.module(file)
            if module is not None:
                self.closure.refusals.update(module.refusals)
                if launched:
                    self.narrowed_programs.add(file)
                for unit in module.program_entries():
                    if launched and unit.kind == "main-guard":
                        for words in sorted(launched, key=repr):
                            self.push(unit, via, "program", frozenset({(ARGV_BINDING, KnownWords((file, *words.words), words.exact))}))
                    else:
                        self.push(unit, via, "program")
        elif language == "shell":
            index = self.shell_index([file])
            shell_file = index.files.get(file)
            if shell_file is not None:
                self.closure.refusals.update(shell_file.refusals)
                if shell_arguments is None and launched and "python-shell-launch-arguments-unfolded" not in self.breaks \
                        and all(words.exact and None not in words.words for words in launched):
                    shell_arguments = {tuple(words.words) for words in launched}
                if shell_arguments and shell_file.script_body is not None and "shell-program-arguments-unnarrowed" not in self.breaks:
                    # 文件头「shell 起法带字面参数」：边照按参数剪过的脚本体认（单元与指纹照原来那一份）
                    for words in sorted(shell_arguments):
                        self.table_launched(file, None if SHELL_OPEN_TAIL in words else words, via=via)
                        self.push(index.narrowed_script_body(file, words), via, "program")
                else:
                    self.table_launched(file, None, via=via)
                    self.push(shell_file.script_body, via, "program")

    def table_launched(self, file, words, undecided_dropped=False, via=None):
        """file 在闭包里按 words（实验号那几个词）起了一次；words 是 None 是整份起了（入口就是它、source 了它、起法折不出）。读表的已经连过的按新的取法补上（文件头「shell 起法带字面参数」）。
        照 words 剪过的脚本体里表的赋值行一行都不剩的（ShellIndex.launch_defines_tables；undecided_dropped 照它）不算一次起法：这个进程里表没定义，只记下这份脚本起过
        （table_selections 里至少有一个空集合，读表的不再照「没起过」连整张）。弄坏开关 shell-table-launch-without-assignment-counted 下照旧记这组词。"""
        if file not in SHELL_ROW_SELECTED_TABLES:
            return
        if words is not None and "shell-table-launch-without-assignment-counted" not in self.breaks \
                and not self.shell_index([file]).launch_defines_tables(file, words, undecided_dropped):
            self.closure.table_launches.append((file, words, via, "no-table"))
            self.table_selections.setdefault(file, set())
            return
        self.closure.table_launches.append((file, words, via, "whole" if words is None or not any(table_row_selection(words)) else "rows"))
        selected = self.table_selections.setdefault(file, set())
        if selected is None or words in selected:
            return
        if words is None:
            self.table_selections[file] = None
        else:
            selected.add(words)
        for variable, via, rule in self.table_references.get(file, ()):
            self.push(variable if words is None else self.shell_index().selected_table_variable(variable, words), via, rule)

    def table_unselected_drivers(self, file):
        """file 是按第一列选行的表脚本、闭包里只按实验号与行项起过时，只写在这几次起法都跑不到的那几行的 @驱动函数（文件头「shell 起法带字面参数」表行之外写到的驱动函数；
        跑得到哪几行照 table_rows_selected：实验号整个跑，--item 点到的实验号只跑点名的行）；别的情形是空集合。
        后来起到那几行时，驱动由那一趟视图里的表行连上，所以这里此刻排掉的不用补。"""
        tables = SHELL_ROW_SELECTED_TABLES.get(file)
        if tables is None or "shell-row-driver-mentions-linked" in self.breaks or "shell-table-rows-unselected" in self.breaks:
            return frozenset()
        launches = tuple(sorted(self.table_selections.get(file) or ()))
        whole = any(not any(table_row_selection(words)) for words in launches)
        if (not launches or whole) and "shell-row-driver-whole-launch-dropped" not in self.breaks:
            return frozenset()
        if (file, launches) not in self.unselected_drivers:
            self.unselected_drivers[(file, launches)] = table_drivers_of_unselected_rows(self.shell.files[file].script_body.body or "", tables, launches)
        return self.unselected_drivers[(file, launches)]

    def push_variable(self, variable, via, rule):
        """单元读到一个 shell 变量：按第一列选行的表照这份脚本在闭包里的起法连（table_launched），别的连它本身。弄坏开关 shell-table-rows-unselected 下连整张表。"""
        tables = SHELL_ROW_SELECTED_TABLES.get(variable.file, ())
        if variable.name not in tables or "shell-table-rows-unselected" in self.breaks:
            self.push(variable, via, rule)
            return
        self.table_references.setdefault(variable.file, []).append((variable, via, rule))
        selected = self.table_selections.get(variable.file)
        if selected is None:
            self.push(variable, via, rule)
            return
        for words in sorted(selected):
            self.push(self.shell_index().selected_table_variable(variable, words), via, rule)

    def run(self, entries, program_arguments=None):
        """program_arguments：程序入口带的参数（文件头「程序入口带参数」）：python 的 main-guard 照 sys.argv 剪，shell 的 script-body 照字面参数剪
        （narrowed_script_body，按第一列选行的表照这组词记一次起法）；None 是没给，照整份收。"""
        self.closure.entry_units.extend(entry for entry in entries if entry is not None)
        shell_words = None
        if program_arguments is not None and "shell-entry-arguments-unnarrowed" not in self.breaks:
            shell_words = tuple(program_arguments)
        for entry in entries:
            if entry is not None and entry.language == "shell":
                self.table_launched(entry.file, shell_words if entry.kind == "script-body" else None,
                                    "shell-entry-undecided-branches-dropped" in self.breaks)
        # 给了参数时 import 时执行的赋值只收赋值本身（文件头「程序入口带参数」）：只在入口是 python 时才这样收
        self.narrowing = program_arguments is not None and "python-program-arguments-unnarrowed" not in self.breaks \
            and any(entry is not None and entry.language == "python" for entry in entries)
        for entry in entries:
            unit, view = entry, None
            if self.narrowing and entry is not None and entry.language == "python" and entry.kind == "main-guard":
                view = frozenset({(ARGV_BINDING, KnownWords((entry.file, *program_arguments), True))})
            elif shell_words is not None and entry is not None and entry.language == "shell" and entry.kind == "script-body":
                unit = self.shell_index().narrowed_script_body(entry.file, shell_words, "shell-entry-undecided-branches-dropped" in self.breaks)
            self.push(unit, None, "entry", view)
        rounds = 0
        while True:
            rounds += 1
            if rounds > CLOSURE_ROUND_LIMIT and "closure-round-limit-unchecked" not in self.breaks:
                names = "、".join(f"{entry.file}::{entry.name}" for entry in self.closure.entry_units)
                raise FunctionUnitsError(f"闭包收了 {CLOSURE_ROUND_LIMIT} 轮还没收完（入口 {names}；此刻 {len(self.closure.order)} 个单元、{len(self.todo)} 个待收）："
                                         "每轮都折出新的程序，看 closure <根> <入口> --explain 的 round 那几行与最后收进来的单元")
            self.drain_pending_units()
            # 调用方交进来的命令在调用处折出的根里脚本（supplied_program_linked）推进 todo：连完再判一轮
            self.resolve_parameter_reads()
            self.caller_launch_refusals()
            self.closure.round_sizes.append(len(self.closure.order))
            if not self.todo:
                return self.closure

    def drain_pending_units(self):
        while self.todo:
            unit, via, rule, view = self.todo.pop()
            if unit.scope == "inline":
                if id(unit) in self.inline_seen:
                    self.named_after_import_time(unit, rule)
                    continue
                self.inline_seen.add(id(unit))
            else:
                self.closure.add(unit, via, rule)
                if self.view_done(unit, view):
                    self.named_after_import_time(unit, rule)
                    continue
                self.views_done.add((id(unit), view))
            if unit.data:
                continue
            if unit.language == "python":
                if unit.scope != "inline":
                    self.call_index_pending.append((unit, view))
                self.python_edges(unit, rule, view)
            elif unit.language == "shell":
                self.shell_edges(unit)

    def push_inline(self, script, via):
        """一份内联脚本：python 的切成单元，从它的程序入口穿过去；bash 的当一个内联 shell 单元（sys.argv / $1… 已折进去）。"""
        if script.language == "python":
            for module in self.python.inline_modules(script):
                self.closure.refusals.update(module.refusals)
                for unit in module.program_entries():
                    self.push(unit, via, "inline")
            return
        for unit in inline_shell_units(self.inline_units, script):
            self.push(unit, via, "inline")

    def push_whole(self, file, via):
        """整份连上的模块（-m unittest 点名的测试模块：测试方法按名字反射着调）：它的每个单元。"""
        if file in self.excluded_files:
            return
        module = self.python.module(file)
        if module is None:
            return
        self.closure.refusals.update(module.refusals)
        for unit in module.units:
            if not unit.data:
                self.push(unit, via, "whole-module")

    # python

    def module_entered(self, module):
        self.modules_seen.add(module.path)
        self.closure.refusals.update(module.refusals)
        for method in module.methods:
            self.methods_by_name[method.short].append(method)
            if id(method.container) in self.named_classes and (method.short in self.attributes_seen or "python-attributes-unnarrowed" in self.breaks):
                self.push(method, method.container, f"method {method.short}")
        if "python-import-time-statements-ignored" in self.breaks:
            return
        # 模块有单元进闭包，它就被载入过：import 时执行的模块级语句一起进来（文件头「Python 闭包：import 时执行的模块级语句」）——
        # 模块级 import、赋值、类头本身（IMPORT_TIME_CLASS_RULE：不算被点名、不连方法）与 def 头（装饰器、缺省值）；其余模块级语句在 module-body 里，本来就连
        # 内联脚本（正文在起它的那个单元里）的单元只穿过一回（inline_seen 不分走法），它的 import 照旧载入时就连到绑的名字
        bindings_followed = "python-import-time-bindings-followed" in self.breaks or module.inline is not None
        for unit in module.units:
            if unit.data or unit.container is not None:
                continue
            if unit.kind == "import":
                # 模块级 import 在载入时只执行被 import 的那份模块；绑定的名字（from x import f 的 f）有单元读到它时才连到 f（文件头「import 时执行的模块级语句」）
                self.push(unit, module.module_body, IMPORT_TIME_RULE, None if bindings_followed else IMPORT_TIME_BINDING_VIEW)
            elif unit.kind in ("assign", "def-head"):
                # 给了参数时 import 时执行的赋值只收赋值本身，值里当元素写的函数名由取这张表的那一处连（文件头「程序入口带参数」）
                self.push(unit, module.module_body, IMPORT_TIME_RULE, SHALLOW_IMPORT_VIEW if (self.narrowing or module.path in self.narrowed_programs) and unit.kind == "assign" else None)
            elif unit.kind == "class":
                self.push(unit, module.module_body, IMPORT_TIME_CLASS_RULE)

    def class_entered(self, head):
        self.named_classes.add(id(head))
        for method in head.module.methods:
            if method.container is head and (method.short in self.attributes_seen or (method.short.startswith("__") and method.short.endswith("__"))):
                self.push(method, head, f"method {method.short}")

    def attribute_seen(self, name, via):
        seen_here = self.attributes_by_module[via.module.path]
        if name not in seen_here:
            seen_here.add(name)
            # 模块值出不了载入方的那几份：只照载入方自己的单元上见过的属性名连（文件头「按路径载入的模块只连载入方调到的属性」）
            for path in sorted(self.contained_loads.get(via.module.path, ())):
                module = self.loaded_objects.get(path)
                if module is not None:
                    for unit in module.by_name.get(name, ()):
                        self.push(unit, via, f"loaded {name}")
        if name in self.attributes_seen:
            return
        self.attributes_seen.add(name)
        unnarrowed = "python-attributes-unnarrowed" in self.breaks
        for method in self.methods_by_name.get(name, ()):
            if unnarrowed or id(method.container) in self.named_classes:
                self.push(method, via, f"method {name}")
        for path in sorted(self.loaded_modules):
            module = self.loaded_objects.get(path)
            if module is not None:
                for unit in module.by_name.get(name, ()):
                    self.push(unit, via, f"loaded {name}")

    def connect_loaded(self, module, unit, name, loaded, contained):
        """module 的单元 unit 载入了 loaded（按路径载入的文件、exec 进模块对象的那一份；name 是它在 contained_loads 与 loaded_modules 里的名字）：
        连它的 module-body（载入时执行的几样随 module_entered 进来）与按属性调到的单元。contained（模块值出不了 module）的只照 module 的单元上见过的属性名连，
        main 与 main-guard 不跟着别处的 .main 进来；别的照整个闭包见过的属性名连（文件头「按路径载入的模块只连载入方调到的属性」）。"""
        self.loaded_objects[name] = loaded
        if contained and "loaded-attributes-closure-wide" not in self.breaks:
            if name not in self.contained_loads[module.path]:
                self.contained_loads[module.path].add(name)
                for attribute in sorted(self.attributes_by_module.get(module.path, ())):
                    for target in loaded.by_name.get(attribute, ()):
                        self.push(target, unit, f"loaded {attribute}")
        elif name not in self.loaded_modules:
            self.loaded_modules.add(name)
            for attribute in sorted(self.attributes_seen):
                for target in loaded.by_name.get(attribute, ()):
                    self.push(target, unit, f"loaded {attribute}")
        self.push(loaded.module_body, unit, "load")

    def python_edges(self, unit, rule=None, view=None):
        module = unit.module
        if module.path not in self.modules_seen:
            self.module_entered(module)
        facts = module.view_facts(unit, view) if isinstance(view, frozenset) else module.facts(unit)
        self.closure.refusals.update(facts.refusals)
        self.pending_launches.extend(facts.caller_launches)
        for line in facts.toolchain_queries:
            self.closure.toolchain_queries.add((unit.file, line))
            if "python-toolchain-queries-keyed" in self.breaks:
                self.closure.add(self.toolchain_unit(unit.file, line), unit, "toolchain-query")
        for relative in facts.registry_reads:
            self.closure.add(self.data_unit(relative), unit, "registry")
        for callee, callee_view in facts.narrowed_calls:
            if not self.call_edge_excluded(unit, callee.name):
                self.push(callee, unit, f"call {callee.name}", callee_view)
        for table in facts.shallow_tables:
            self.push(table, unit, "table-keys", SHALLOW_IMPORT_VIEW)
        if unit.container is not None:
            self.push(unit.container, unit, "container")
        if unit.kind != "module-body":
            self.push(module.module_body, unit, "module-body")
        if unit.kind == "class":
            if rule == IMPORT_TIME_CLASS_RULE and "python-import-time-classes-named" not in self.breaks:
                self.import_time_classes.add(id(unit))   # 只当类头收进来：不连方法，之后被点名再连（named_after_import_time）
            else:
                self.class_entered(unit)
        for target in facts.import_targets:
            if view == IMPORT_TIME_BINDING_VIEW and target.kind != "module-body":
                continue   # 只因载入收进来的 import：执行被 import 的模块，名字绑到的单元等有人读它再连（module_entered）
            self.push(target, unit, "import")
        names = facts.names
        if view == SHALLOW_IMPORT_VIEW and "python-import-time-tables-followed" not in self.breaks:
            names = names - module.table_reference_names(unit)
        for name in sorted(names):
            if self.call_edge_excluded(unit, name):
                continue   # 登记了的那一条调用边不走（EXCLUDED_CALL_EDGES）
            for target in module.resolve_name(name, unit, facts):
                self.push(target, unit, f"name {name}")
        self.closure.refusals.update(facts.refusals)
        for base, attribute in facts.attributes:
            targets = module.resolve_attribute(base, attribute, unit, facts)
            if targets is not None:
                for target in targets:
                    self.push(target, unit, f"attribute {base}.{attribute}")
            else:
                self.attribute_seen(attribute, unit)
        if "python-attributes-unnarrowed" in self.breaks:
            for base, attribute in facts.attributes:
                for method in self.methods_by_name.get(attribute, ()):
                    self.push(method, unit, f"method {attribute}")
        for file in facts.loads:
            loaded = self.python.module(file)
            if loaded is not None:
                self.connect_loaded(module, unit, file, loaded, facts.contained_loads.count(file) == facts.loads.count(file))
        for script, contained in facts.module_object_texts:
            # exec 进模块对象的内联正文（文件头「exec 进模块对象」）：照按路径载入的那一份连；正文已在交它的那个单元里（shell 变量、字面串），这几份的单元只穿过去
            for loaded in self.python.inline_modules(script):
                self.closure.refusals.update(loaded.refusals)
                self.connect_loaded(module, unit, f"{loaded.path}#{sha256_text(loaded.text)}", loaded, contained)
        for file in dict.fromkeys(facts.program_links):
            if (unit.file, unit.name, file) in EXCLUDED_PROGRAM_EDGES and "program-edge-exclusions-ignored" not in self.breaks:
                continue   # 登记了的那一条子进程边不走（EXCLUDED_PROGRAM_EDGES）
            # 每一处起它的都带着参数（一处不知道就整份收）才照参数剪（文件头「程序入口带参数」子进程边）
            launched = facts.program_arguments.get(file, [])
            # shell 脚本体按字面参数剪不看入口带没带参数（文件头「shell 起法带字面参数」）：每一处起它的都折得出才剪
            choices = facts.shell_launch_words.get(file, [])
            shell_arguments = None
            if language_of_file(self.root, file) == "shell" and choices and len(choices) == facts.program_links.count(file) and None not in choices \
                    and "python-shell-launch-arguments-unfolded" not in self.breaks:
                shell_arguments = set().union(*choices)
            self.push_program(file, unit, set(launched) if "python-program-arguments-unnarrowed" not in self.breaks and launched and len(launched) == facts.program_links.count(file) else None, shell_arguments)
        for script in facts.inline_scripts:
            self.push_inline(script, unit)
        for file in facts.whole_modules:
            self.push_whole(file, unit)
        self.python_data_reads(module, unit, facts)
        self.built_tool_sources(unit, [BUILT_TOOL_SOURCES.get((unit.file, unit.name))] + facts.built_tools)
        if unit.kind == "main-guard" and module.inline is None and "declared-checks-ignored" not in self.breaks:
            # 程序入口调 preflight：它先在仓根跑这份文件头登记的 check 判据
            for script in declared_checks(self.root, module.path, module.rows):
                self.push_inline(script, unit)
        if module.inline is not None:
            # 内联脚本没有文档串可读、也没有自己的原文文件：读 __doc__ / __file__ 的那几样不收
            for text in facts.strings:
                program = external_program_name(text, self.breaks)
                if program is not None:
                    self.closure.programs.setdefault(program, (unit.file, unit.first_line))
            return
        if "python-docstring-readers-ignored" not in self.breaks:
            for owner in facts.doc_reads:
                if owner in module.docstrings or owner in ("<module>", "*"):
                    for target in module.docstring_units(owner):
                        self.closure.add(target, unit, "docstring")
                    continue
                other = module.module_of_binding(module.binding(owner.split(".")[0], unit))
                for target in (other.docstring_units("<module>") if other is not None else module.docstring_units("*")):
                    self.closure.add(target, unit, "docstring")
        if facts.source_read:
            self.closure.add(module.source_unit(), unit, "source-text")
        for text in facts.strings:
            program = external_program_name(text, self.breaks)
            if program is not None:
                self.closure.programs.setdefault(program, (unit.file, unit.first_line))

    def call_edge_excluded(self, unit, name):
        """unit 按名字连 name 的那一条边登在 EXCLUDED_CALL_EDGES 里（文件头「不走的调用边」）；弄坏开关 call-edge-exclusions-ignored 下一律照连。"""
        return (unit.file, unit.name, name) in EXCLUDED_CALL_EDGES and "call-edge-exclusions-ignored" not in self.breaks

    def toolchain_unit(self, file, line):
        """只在弄坏开关 python-toolchain-queries-keyed 下收（文件头「版本询问」）：工具链询问那一处的工具链单元，指纹是根里起的 PATH 上 rustc -vV 那一行
        （toolchain_version_line）的 sha256，询问打的版本因此进键；一个闭包只问一次。"""
        if "toolchain_fingerprint" not in self.__dict__:
            self.toolchain_fingerprint = sha256_text(toolchain_version_line("rustc -vV", ["rustc", "-vV"], self.root))
        return Unit("data", TOOLCHAIN_UNIT_KIND, TOOLCHAIN_UNIT_KIND, file, line, line, self.toolchain_fingerprint, data=True)

    def shared_data_units(self, name):
        """data_unit 与 data_directory 的单元表（name 是 data_units 或 data_directories）：共用一个脚本索引（PythonIndex）的闭包共用一张，
        一趟里几十个闭包读同一批数据文件时，每份只读一遍、算一遍摘要；索引随这一趟的文件内容一起建，下一趟（新索引）重读。
        弄坏开关 data-units-process-wide 下整个进程共用一张（按根分），同一棵树里文件改了也认旧的。"""
        if "data-units-process-wide" in self.breaks:
            return PROCESS_WIDE_DATA_UNITS.setdefault((self.root, name), {})
        return getattr(self.python, name)

    def data_unit(self, path):
        """运行期读的仓内文件 → 它的数据单元（data-file，字节 sha256；同一份只造一个，共用同一个脚本索引的闭包之间也只造一个）。"""
        units = self.shared_data_units("data_units")
        if path not in units:
            units[path] = data_file_unit(self.root, path)
        return units[path]

    def built_tool_sources(self, unit, sources):
        """编工具程序的函数进了闭包（BUILT_TOOL_SOURCES，python 函数与 shell 函数都认）、起编出来的工具的那一处（BUILT_TOOL_LAUNCHES）：
        sources 里每个源码目录（相对根，None 跳过）整棵收成数据目录，规则名 tool-source。弄坏开关 built-tool-sources-ignored 下不收。"""
        if "built-tool-sources-ignored" in self.breaks:
            return
        for source in dict.fromkeys(source for source in sources if source is not None):
            if os.path.isdir(os.path.join(self.root, source)):
                self.closure.add(self.data_directory(source), unit, "tool-source")

    def data_directory(self, path):
        """运行期读的根里固定目录（文件头「运行期读的数据文件」一）→ 它的数据单元（data-directory，指纹同 directory_fingerprint；同一个只造一个，
        共用同一个脚本索引的闭包之间也只造一个）。"""
        units = self.shared_data_units("data_directories")
        if path not in units:
            units[path] = data_directory_unit(self.root, path)
        return units[path]

    def python_data_reads(self, module, unit, facts):
        """单元里读文件的每一处（PythonFacts.data_reads）：照文件头「运行期读的数据文件」三类落（data_read_origin）。"""
        for origins, line, copied in facts.data_reads:
            for origin in sorted(origins, key=lambda item: tuple(str(part) for part in item if not isinstance(part, Unit))):
                self.data_read_origin(origin, module, unit, line, copied)

    def data_read_origin(self, origin, module, unit, line, copied=False):
        """一种取法：文件、目录收成数据单元（规则名 data-read）；拒算的记 data-path-unfolded（("refuse", <形态>) 记那个形态：取值超过 FOLD_LIMIT 种的
        FOLD_LIMIT_FORM），落在读它的那一行（copied：拷文件的源，不拒算）；
        本模块里没有调用处的参数留到收完闭包再认（resolve_parameter_reads）；别的（临时目录、调用方的输入、根外、推不出的）不收也不拒算。"""
        kind = origin[0]
        if kind == "file":
            self.closure.add(self.data_unit(origin[1]), unit, "data-read")
        elif kind == "directory":
            self.closure.add(self.data_directory(origin[1]), unit, "data-read")
        elif kind == "refuse" and not copied:
            self.closure.refusals.add(module.refusal(origin[1] if len(origin) > 1 else DATA_FORM, line))
        elif kind == "param":
            self.pending_parameters.append((origin[1], origin[2], module, unit, line, copied))
        elif kind == "param-tail":
            # 前缀是参数的字面尾巴（PythonModule.prefix_parameter）：收完闭包再按调用处认（resolve_parameter_reads）
            self.tail_reads.append((origin, unit))

    def tail_read_supplied(self, function, name, resolved, depth=0):
        """读 <参数>/<字面尾巴> 的那个参数（文件头「运行期读的数据文件」二）在闭包里每一处调用给的都是这一趟自己建的临时目录、入口的参数（被判的输入），
        或又是这样的参数（接着往上一层认）：尾巴那一组不收。闭包里一处调用都没有的算调用方的输入。给的是 sys.argv、环境变量、根里的路径、推不出的
        （仓副本、根本身都可能）→ False，照尾巴收仓里那一组。弄坏开关 python-tail-reads-unanchored 下不走这一条（读的时候就整组收）。"""
        if depth > 8:
            return False
        if id(function) in {id(entry) for entry in self.closure.entry_units}:
            return True
        key = (id(function), name)
        if key not in resolved:
            resolved[key] = False   # 正在判：绕回来的当不是
            supplied = True
            for argument, caller in self.closure_call_arguments(function, name):
                anchor = caller.module.prefix_parameter(argument, caller) if caller is not None and caller.module is not None else None
                if anchor is not None:
                    supplied = self.tail_read_supplied(anchor[0], anchor[1], resolved, depth + 1)
                else:
                    origins = caller.module.head_origin(argument, caller, 0) if caller is not None and caller.module is not None else None
                    supplied = origins is not None and all(
                        origin == ("skip", "temporary") or (origin[0] == "param" and self.tail_read_supplied(origin[1], origin[2], resolved, depth + 1))
                        for origin in origins)
                if not supplied:
                    break
            resolved[key] = supplied
        return resolved[key]

    def resolve_parameter_reads(self):
        """本模块里没有调用处的参数读的文件（文件头「运行期读的数据文件」一）：入口的参数算调用方的输入；别的在闭包里按名字找调用处（closure_call_arguments），
        逐处照实参认（实参又是这种参数的接着找）；闭包里一处都找不到的算调用方的输入。弄坏开关 python-parameter-callers-unresolved 下不是入口的拒算。"""
        entries = {id(entry) for entry in self.closure.entry_units}
        seen = set()
        while self.pending_parameters:
            function, name, module, reader, line, copied = self.pending_parameters.pop()
            if id(function) in entries or (id(function), name, line, module.path) in seen:
                continue
            seen.add((id(function), name, line, module.path))
            if "python-parameter-callers-unresolved" in self.breaks:
                self.closure.refusals.add(module.refusal(DATA_FORM, line))
                continue
            for argument, caller in self.closure_call_arguments(function, name):
                for origin in sorted(caller.module.data_read_origins(argument, caller), key=lambda item: tuple(str(part) for part in item if not isinstance(part, Unit))):
                    self.data_read_origin(origin, module, reader, line, copied)
        resolved = {}
        for position, (origin, reader) in enumerate(self.tail_reads):
            # 闭包里有一处给的不是这一趟的临时目录、也不是被判的输入：照尾巴收仓里那一组
            if position not in self.tail_reads_collected and not self.tail_read_supplied(origin[1], origin[2], resolved):
                self.tail_reads_collected.add(position)
                for relative in sorted(origin[3]):
                    self.closure.add(self.data_unit(relative), reader, "data-read")

    def closure_call_arguments(self, function, name):
        """闭包里调 function 的每一处给参数 name 的实参 [(实参节点, 调用处的作用域单元)]：按名字认（f(…)、x.f(…)；__init__ 认 类名(…)），
        只留真解析得到 function 的那几处（call_reaches）；方法去掉 self；带 * / ** 的那一处认不出对哪个参数、跳过；没给的取缺省值（作用域是 function 自己）。
        functools.partial(f, …) 照 f(…) 算一处（partial_bound_call；弄坏开关 python-partial-call-sites-ignored 下不算）。"""
        definition = function.node
        if not isinstance(definition, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return []
        arguments = definition.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        method = function.container is not None and not any(
            isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in definition.decorator_list)
        if method:
            positional = positional[1:]
        called = function.container.name.split(".")[-1] if method and definition.name == "__init__" else definition.name
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        self.index_pending_calls()
        out = []
        sites = [(call, call, member) for call, member in self.calls_by_name.get(called, ())]
        if "python-partial-call-sites-ignored" not in self.breaks:
            for written, member in self.calls_by_name.get("partial", ()):
                bound = partial_bound_call(written, called)
                if bound is not None:
                    sites.append((bound, written, member))
        for call, written, member in sites:
            if not self.call_reaches(call, member, function, written):
                continue
            if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
                continue
            argument = call.args[position] if position is not None and position < len(call.args) else None
            argument = next((keyword.value for keyword in call.keywords if keyword.arg == name), argument)
            if argument is not None:
                out.append((argument, member.module.scope_of(written, member)))
            elif defaults.get(name) is not None:
                out.append((defaults[name], function))
        return out

    def call_reaches(self, call, member, function, written=None):
        """按名字认到的一处调用（calls_by_name）真解析得到 function 才算它的调用处（文件头「调用方交进来的命令」调用处那一条）。判不了的照旧算。
        f(…)，function 不是类里的方法：从调用处那一层往外，头一层把 f 当局部名字的函数作用域就是 f 解析到的地方，function 正是那一层里定义的才算
        （套着的同名函数遮住外层的）；那一层 global / nonlocal 声明过 f 的照旧算。没有一层绑它、到了模块级的：function 是套着的函数不算；
        function 是模块级函数时，同一个模块算，别的模块里 import 了 f 的照 import 解到的是不是 function 那一份，没 import 而自己有模块级同名 def / class 的不算，别的照旧算。
        x.f(…)：x 往外每一层都不是局部名字、import 绑的是标准库或外部包、或解到的仓内模块不是 function 那一份的不算（subprocess.run 不是模块级 run 的调用处）；
        x 是 self、局部变量、载入的模块、属性链与调用的结果都判不了，照旧算。弄坏开关 python-call-sites-matched-by-name 下一律照名字算。
        written 是源码里真写着的那个节点（functools.partial(f, …) 那一处，call 是照 f(…) 写的那一份）：作用域照它找，没给就是 call。"""
        if "python-call-sites-matched-by-name" in self.breaks:
            return True
        module = getattr(member, "module", None)
        if module is None or getattr(function, "module", None) is None:
            return True
        if isinstance(call.func, ast.Name):
            name, attribute = call.func.id, False
            if function.container is not None:
                return True
        elif isinstance(call.func, ast.Attribute) and isinstance(call.func.value, ast.Name):
            name, attribute = call.func.value.id, True
        else:
            return True
        # function 定义在哪一层函数里（套着的函数）；定义在 main-guard 这类模块级代码里的照模块级函数认
        defining = enclosing_unit(function)
        defining_node = defining.node if defining is not None and isinstance(defining.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None
        place = module.scope_of(call if written is None else written, member)
        scope = place
        while scope is not None:
            if isinstance(scope.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                names, declared = self.scope_local_names(scope.node)
                if name in declared:
                    return True
                if name in names:
                    return attribute or (defining_node is not None and defining_node is scope.node)
            scope = enclosing_unit(scope)
        if attribute:
            return self.binding_reaches(module, module.binding(name, place), function, True)
        if defining_node is not None:
            return False
        if module is function.module:
            return True
        binding = module.binding(name, place)
        if binding is not None:
            return self.binding_reaches(module, binding, function, True)
        return not any(candidate.kind in ("def", "class") and candidate.container is None for candidate in module.by_name.get(name, ()))

    @staticmethod
    def binding_reaches(module, binding, function, unresolved):
        """module 里一个 import 绑定解到的模块是不是 function 那一份（call_reaches）：标准库、外部包 → False；仓内的照每一份解到的文件比；解不出的交 unresolved。"""
        if binding is None:
            return unresolved
        if binding[0] == "import":
            dotted, level = binding[1], 0
        else:
            dotted, level = (f"{binding[1]}.{binding[4]}" if binding[1] else binding[4]), binding[2]
            status, _file = module.resolve_module(dotted, level)
            if status != "repo":
                # from 包 import 名字：名字不是子模块时，照包本身那一份比（名字是它里面的函数或类）
                dotted = binding[1]
        status, file = module.resolve_module(dotted, level)
        if status in ("stdlib", "external"):
            return False
        if status != "repo":
            return unresolved
        return function.module.path in (module.all_resolutions(dotted, level) or [file])

    def scope_local_names(self, node):
        """一层 def 自己绑的名字（function_local_names），按节点记下来：同一层在 call_reaches 里反复问。"""
        cached = self.scope_names.get(id(node))
        if cached is None or cached[0] is not node:
            cached = (node, *function_local_names(node))
            self.scope_names[id(node)] = cached
        return cached[1], cached[2]

    def index_pending_calls(self):
        """新收进闭包的（单元, 视图）里的调用并进 calls_by_name：程序入口带参数剪掉的那几截（narrowing_plan）
        里的调用不算；同一个节点只记一次。弄坏开关 closure-call-index-frozen 下头一次建完就不再并（后收进来的调用处看不见）。"""
        if "closure-call-index-frozen" in self.breaks and self.call_nodes_indexed:
            return
        while self.call_index_pending:
            member, view = self.call_index_pending.pop()
            if getattr(member, "module", None) is None:
                continue
            for node, callee in self.named_calls_of(member, view):
                if id(node) not in self.call_nodes_indexed:
                    self.call_nodes_indexed.add(id(node))
                    self.calls_by_name[callee].append((node, member))

    def named_calls_of(self, member, view):
        """member 在 view 下的调用 [(ast.Call 节点, 被调的名字)]，按层走的次序，只留认得出名字的（f(…) 取 f，x.f(…) 取 f）。
        不剪的（view 不是程序入口带参数剪过的那一种）存在单元自己的 cache["named-calls"] 里：单元随它的模块（文件内容）一起建，每个闭包收进同一个单元都不再各走一遍语法树；
        剪过的按 narrowing_plan 现走。"""
        if isinstance(view, frozenset) and "python-call-sites-whole-view" not in self.breaks:
            skipped, extra = member.module.narrowing_plan(member, dict(view))[:2]
            return named_calls_in(itertools.chain(walk_nodes_skipping(member.nodes or [], skipped), walk_nodes(extra)))
        if member.cache is None:
            member.cache = {}
        if "named-calls" not in member.cache:
            member.cache["named-calls"] = named_calls_in(walk_nodes_skipping(member.nodes or [], ()))
        return member.cache["named-calls"]

    def caller_launch_refusals(self):
        """调用方交进来的命令（文件头「调用方交进来的命令」）：折不出的那几截出处都是这一次入口的参数，或 sys.argv、argparse 的属性、临时目录、根外前缀的
        不拒算（被起的命令由请求那一侧的单元键管）；别的函数的参数照闭包里每一处调它的实参往上认（launch_supplied_by_caller），推不出的照旧拒算。"""
        resolved = {}
        while self.pending_launches:
            refusal, origins = self.pending_launches.pop()
            if not self.launch_supplied_by_caller(origins, resolved):
                self.closure.refusals.add(refusal)

    def launch_supplied_by_caller(self, origins, resolved):
        """起子进程那一处折不出的几截都是调用方交进来的：出处是 sys.argv、argparse 的属性、临时目录、根外前缀，或入口的参数，
        或别的函数的参数而闭包里每一处调它的实参又都是这几样（接着往上找；resolved 按（函数, 参数名）记下判过的，绕回来的不算推得出）。
        闭包里没人调的别的函数的参数推不出。"""
        entries = {id(entry) for entry in self.closure.entry_units}
        for origin in origins:
            if origin[0] == "skip" and origin[1] in CALLER_INPUT_ORIGINS:
                continue
            if origin[0] != "param":
                return False
            function, name = origin[1], origin[2]
            if id(function) in entries:
                continue
            if not self.parameter_supplied(function, name, resolved):
                return False
        return True

    def parameter_supplied(self, function, name, resolved, depth=0):
        """别的函数的参数 name 给的命令：闭包里调 function 的每一处的实参都认得出（launch_argument_supplied）；入口的参数算调用方交进来的；
        闭包里没人调的推不出。resolved 按（函数, 参数名）记下判过的，绕回来的不算推得出。"""
        if id(function) in {id(entry) for entry in self.closure.entry_units}:
            return True
        key = (id(function), name)
        if key not in resolved:
            resolved[key] = False   # 正在判：绕回来的当推不出
            sites = self.closure_call_arguments(function, name)
            if "caller-supplied-commands-callee-wide" in self.breaks and any(self.caller_supplied_registered(argument, caller) for argument, caller in sites):
                resolved[key] = True   # 弄坏开关：一处登记了就当整个被调函数都登记了
                return True
            resolved[key] = bool(sites) and all(self.launch_argument_supplied(argument, caller, resolved, depth + 1)
                                                or self.caller_supplied_registered(argument, caller) for argument, caller in sites)
        return resolved[key]

    def caller_supplied_registered(self, argument, caller):
        """调用方登记（文件头「调用方登记」）：这一处调用（调用处的文件、调用处单元的限定名 registered_function_name、实参的 ast.unparse）登在 CALLER_SUPPLIED_COMMANDS 里。
        只在这一处往下认不出时问（认得出的照常连），只管这一处；弄坏开关 caller-supplied-commands-refused 下一律不算。"""
        if "caller-supplied-commands-refused" in self.breaks or caller is None or getattr(caller, "module", None) is None:
            return False
        return (caller.module.source_file(), registered_function_name(caller), ast.unparse(argument)) in CALLER_SUPPLIED_COMMANDS

    def launch_argument_supplied(self, argument, caller, resolved, depth=0):
        """闭包里一处调用给命令参数的实参，在调用处折（文件头「调用方交进来的命令」：被调函数的参数按每个调用点的实参折）：
        折得出字面值的，是根里的脚本就当程序连上（解释器词不算：脚本在命令的别处，认不出）；根外的程序（outside_head）；
        名字照调用处那一层的参数（接着往上一层调用处认）、本函数里的赋值、外层函数的变量认；调仓内函数的，照它每一处 return 的值在它那边认；
        别的照调用方交进来的那几样认（unfolded_part_origins）。弄坏开关 python-launch-arguments-unfolded 下只认最后一样。"""
        if depth > 12 or caller is None or getattr(caller, "module", None) is None:
            return False
        module = caller.module
        if "python-launch-arguments-unfolded" not in self.breaks:
            folded = module.fold(argument, caller)
            if folded is not None:
                return all(self.supplied_program_linked(value, caller) for value in folded)
            if module.outside_head(argument, caller):
                return True
            if isinstance(argument, ast.Name) and isinstance(caller.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                arguments = caller.node.args
                assigned = module.local_assignments(caller).get(argument.id)
                if assigned is None and argument.id in [item.arg for item in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    return self.parameter_supplied(caller, argument.id, resolved, depth + 1)
                if assigned:
                    return all(value is not None and not isinstance(value, ElementsOf) and self.launch_argument_supplied(value, caller, resolved, depth + 1)
                               for value in assigned)
                if assigned is None and enclosing_unit(caller) is not None:
                    return self.launch_argument_supplied(argument, enclosing_unit(caller), resolved, depth + 1)
            if isinstance(argument, ast.Call) and isinstance(argument.func, ast.Name):
                returned = module.returned_values(argument.func.id, caller)
                if returned is not None:
                    callee_module, callee, values = returned
                    return all(self.launch_argument_supplied(value, callee, resolved, depth + 1) for value in values)
        found = module.unfolded_part_origins(argument, caller)
        return found is not None and self.launch_supplied_by_caller(found, resolved)

    def supplied_program_linked(self, value, caller):
        """调用处折出来的命令词（launch_argument_supplied）：根里的 .py、.sh 当程序连上；python3、bash 这类解释器词不算认得出
        （脚本写在命令的别处）；别的（cargo、根外的程序）不连。"""
        if posixpath.basename(value) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS:
            return False
        relative = value if not posixpath.isabs(value) else posixpath.relpath(value, self.root)
        if not relative.startswith("../") and os.path.isfile(os.path.join(self.root, relative)) and relative.endswith((".py", ".sh")):
            self.push_program(posixpath.normpath(relative), caller)
        return True

    # shell

    def visible_roots(self, path):
        """visible_files 从哪几份往下展开：它那份文件，这个闭包里（传递地）source 了它的那几份，与这个闭包里按名字叫到它那份文件里的函数的单元所在的那几份
        （closure_name_reachers；叫它的那一份又照它自己的这几样往上找）。入口是库里的一个函数、source 它的脚本体不在闭包里时，库里的函数调同一个入口 source 的
        另一份库里的函数，照叫到这份库的那一份看得见的文件找（文件头「shell 里被 source 的库读别的文件里赋的变量」）。
        弄坏开关 shell-name-reach-context-dropped 下不认按名字叫到的那几份。"""
        name_reach = "shell-name-reach-context-dropped" not in self.breaks
        roots, todo = {path}, [path]
        while todo:
            item = todo.pop()
            reachers = set(self.closure_sourcers.get(item, ()))
            if name_reach:
                reachers |= self.closure_name_reachers.get(item, set())
            for reacher in sorted(reachers):
                if reacher not in roots:
                    roots.add(reacher)
                    todo.append(reacher)
        return roots

    def visible_files(self, path):
        """一个单元能叫到哪几份文件里的函数与变量：visible_roots 那几份，连同它们（传递地）source 进来的。
        同一份库被别的脚本也 source 了，那些脚本的函数不算可见（它们不在这一次的调用链上）。"""
        return self.shell.downward(self.visible_roots(path))

    def note_name_reach(self, target_file, reacher_file):
        """reacher_file 里的单元按名字连到 target_file 里的函数：记进 closure_name_reachers；头一次记的，把已收过的、看得见的文件因此变多的那几份单元的名字重认一遍
        （同 source 边后到时的重认）。"""
        if target_file == reacher_file or reacher_file in self.closure_name_reachers[target_file]:
            return
        before = self.visible_files(target_file)
        self.closure_name_reachers[target_file].add(reacher_file)
        if "shell-name-reach-context-dropped" in self.breaks or self.visible_files(target_file) == before:
            return   # 叫它的那一份本来就在它看得见的那几份里：已收的单元看得见的文件没变多
        for path in sorted(self.processed_shell):
            if target_file in self.visible_roots(path):
                for processed in list(self.processed_shell.get(path, ())):
                    self.shell_names(processed, self.shell.facts(processed), self.visible_files(path))

    def shell_names(self, unit, facts, visible):
        index = self.shell
        shell_file = index.files[unit.file]
        names = facts.command_words if "shell-names-command-position-only" in self.breaks else facts.words
        unselected = self.table_unselected_drivers(unit.file)
        for name in sorted(names & index.function_names_in(visible)):
            if name in unselected and written_only_as_table_row(unit.body or "", name):
                continue   # 文件头「shell 起法带字面参数」表行之外写到的驱动函数
            for target in index.functions_in(visible, name):
                if target is not unit and self.shell_nested_function_named(unit, target, name, facts):
                    self.push(target, unit, f"name {name}")
                    self.note_name_reach(target.file, unit.file)
        for name in sorted(facts.variable_references):
            targets = shell_file.variables.get(name) or [variable for path in sorted(visible) if path in index.files
                                                          for variable in index.files[path].variables.get(name, [])]
            for target in targets:
                self.push_variable(target, unit, f"variable {name}")

    def shell_nested_function_named(self, unit, target, name, facts):
        """单元里写到 target 的名字算不算连它（文件头「shell 里套在函数里的函数」）：顶层函数算；套在函数里的，单元在同一个外层函数（连同它里面）的算，外面的只认命令位置上的词。"""
        owner = target.container
        if owner is None or "shell-nested-functions-by-any-word" in self.breaks:
            return True
        scope = unit
        while scope is not None:
            if scope is owner:
                return True
            scope = scope.container
        return name in facts.command_words

    def shell_edges(self, unit):
        index = self.shell_index()
        inline = unit.scope == "inline"
        shell_file = index.files.get(unit.file) if not inline else None
        if shell_file is not None:
            self.closure.refusals.update(shell_file.refusals)
        facts = index.facts(unit)
        if not inline:
            self.processed_shell[unit.file].append(unit)
        if unit.container is not None:
            self.push(unit.container, unit, "container")
        if shell_file is not None and shell_file.kept is not None:
            self.closure.add(shell_file.kept, unit, "kept-comments")
        if not inline:
            self.shell_names(unit, facts, self.visible_files(unit.file))
            self.built_tool_sources(unit, [BUILT_TOOL_SOURCES.get((unit.file, unit.name))])
        if unit.kind == "script-body" and shell_file is not None and "declared-checks-ignored" not in self.breaks:
            # 脚本体调 preflight：它先在仓根跑这份文件头登记的 check 判据
            for script in declared_checks(self.root, unit.file, shell_file.rows):
                self.push_inline(script, unit)
        sourced, prefixes = set(), []
        for kind, files, refusal in index.links(unit):
            if refusal is not None:
                self.closure.refusals.add(refusal)
            for file in files:
                if kind == "function-prefix":
                    prefixes.append(file)
                elif kind == "python-inline":
                    self.push_inline(file, unit)
                elif kind == "python-whole":
                    self.push_whole(file, unit)
                elif kind == "source":
                    if self.private_root_file(file):
                        continue   # 仓根被 .gitignore 挡着的私有文件不读（文件头「仓根被忽略的私有文件」）
                    added = index.files.get(file)
                    if added is None:
                        index.add([file])
                        added = index.files.get(file)
                    if added is None:
                        continue
                    self.closure.refusals.update(added.refusals)
                    self.table_launched(file, None, via=unit)
                    self.push(added.script_body, unit, "source")
                    sourced.add(file)
                    if not inline and unit.file not in self.closure_sourcers[file]:
                        self.closure_sourcers[file].add(unit.file)
                        for path in sorted(index.downward([file])):
                            for processed in list(self.processed_shell.get(path, ())):
                                self.shell_names(processed, index.facts(processed), self.visible_files(path))
                else:
                    if (unit.file, unit.name, file) in EXCLUDED_PROGRAM_EDGES and "program-edge-exclusions-ignored" not in self.breaks \
                            and "program-edge-exclusions-shell-ignored" not in self.breaks:
                        continue   # 登记了的那一条子进程边不走（EXCLUDED_PROGRAM_EDGES；shell 函数起的同样认）
                    self.push_program(file, unit, index.python_launch_words(unit, file) if kind == "python" else None,
                                      index.shell_launch_arguments(unit, file) if kind == "shell" else None)
        if prefixes:
            # "前缀_$1" 派活：看得见的函数里名字以这个前缀起头的都连
            visible = index.downward(sourced) if inline else self.visible_files(unit.file)
            for prefix in sorted(set(prefixes)):
                for name in sorted(name for name in index.function_names_in(visible) if name.startswith(prefix)):
                    for target in index.functions_in(visible, name):
                        if target is not unit:
                            self.push(target, unit, f"prefix {prefix}")
                            if not inline:
                                self.note_name_reach(target.file, unit.file)
        if inline:
            # 内联 bash（python 起的 bash -c）：叫得到的函数与变量只在它 source 进来的那几份里
            visible = index.downward(sourced)
            for name in sorted(facts.words & index.function_names_in(visible)):
                for target in index.functions_in(visible, name):
                    self.push(target, unit, f"name {name}")
            for name in sorted(facts.variable_references):
                for path in sorted(visible):
                    for target in index.files[path].variables.get(name, []) if path in index.files else []:
                        self.push(target, unit, f"variable {name}")
        for program in facts.programs:
            self.closure.programs.setdefault(program, (unit.file, unit.first_line))


# ── 入口、键与输出 ───────────────────────────────────────────────────────────

ENTRY_PATTERN = re.compile(r"^(.+?\.(?:rs|wgsl|py|sh|bash))(?:::(.+))?$")


def parse_entry(root, entry):
    """→ (语言, 文件或 None, 限定名或 None)。"""
    found = ENTRY_PATTERN.match(entry)
    if found:
        path, qualified = found.group(1), found.group(2)
        suffix = path.rsplit(".", 1)[1]
        language = {"rs": "rust", "wgsl": "wgsl", "py": "python", "sh": "shell", "bash": "shell"}[suffix]
        return language, path, qualified
    path, _separator, qualified = entry.partition("::")
    if "/" in path and os.path.isfile(os.path.join(root, path)):
        language = language_of_file(root, path)
        if language is None:
            raise UsageError(f"认不出 {path} 是哪种语言")
        return language, path, qualified or None
    if "::" in entry and "/" not in entry:
        return "rust", None, entry
    raise UsageError(f"认不出的入口：{entry}（写 <文件>::<限定名>、<文件> 或 Rust 的 <crate>::<模块…>::<名>）")


class Options:
    def __init__(self):
        self.language, self.under, self.target = None, [], None
        self.cfg, self.no_default_cfg, self.features, self.no_default_features = [], False, [], False
        self.wgsl_modules, self.rust_strings_entry, self.explain = [], None, False
        # 判定种类（文件头「判定种类」）；product_dump 只给库接口的调用方交 Rust 那一份 dump 行用，命令行不认
        self.judgment_kind, self.product_dump = FORMAL_VERIFICATION, None
        # 程序入口（python 与 shell）带的参数（文件头「程序入口带参数」）；None 是没给
        self.program_arguments = None
        # 按角色排除的文件（文件头「按角色排除的文件」）；judging 是命令行 --judging（照判定算键的口径定排哪几份，command_closure）
        self.excluded_files = ()
        self.judging = False


def judging_excluded_files(entry):
    """命令行 closure / key --judging（文件头「按角色排除的文件」）：这个入口照判定算键的口径按角色排除哪几份。取法只在 admission.py 的
    not_judging_files_of_entry 一处（KeyContext.collected_script_closure 用的同一个函数、同一种载入法：它自己的 function_units() 交的库），
    这里按路径载入这份库同目录的 admission.py 调它，不另抄。弄坏开关 closure-judging-exclusions-dropped 下交空（照不带 --judging 的口径收）。"""
    if "closure-judging-exclusions-dropped" in active_breaks():
        return ()
    specification = importlib.util.spec_from_file_location("admission_for_judging_closure", os.path.join(LIBRARY_ROOT, "scripts", "gate", "admission.py"))
    admission = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(admission)
    return tuple(admission.not_judging_files_of_entry(admission.function_units(), entry.partition("::")[0]))


def command_closure(root, entry, options):
    """命令行 closure / key 收的闭包：带 --judging 的先照判定算键的口径定按角色排除哪几份（judging_excluded_files），再 closure_of。"""
    if options.judging:
        options.excluded_files = judging_excluded_files(entry)
    return closure_of(root, entry, options)


def closure_of(root, entry, options=None):
    options = options or Options()
    if not os.path.isdir(root):
        raise FunctionUnitsError(f"根不在：{root}")
    language, path, qualified = parse_entry(root, entry)
    if language == "rust":
        workspace = RustWorkspace(root, options.judgment_kind, options.product_dump)
        unit, target_id = workspace.find_entry(path, qualified, options.target)
        return rust_closure_of(workspace, unit, target_id, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
    if language == "wgsl":
        files = [wgsl_file(root, item) for item in [path] + list(options.wgsl_modules)]
        entries = [unit for unit in files[0].units if unit.name == qualified] if qualified else None
        if qualified and not entries:
            raise UsageError(f"{path} 里没有 {qualified}")
        sources, workspace, rust = None, None, None
        closure = Closure()
        closure.entry_units.extend(entries or ())
        if options.rust_strings_entry:
            rust_language, rust_path, rust_qualified = parse_entry(root, options.rust_strings_entry)
            workspace = RustWorkspace(root, options.judgment_kind, options.product_dump)
            if rust_language == "rust":
                rust_unit, rust_target = workspace.find_entry(rust_path, rust_qualified, options.target)
                rust = rust_closure_of(workspace, rust_unit, rust_target, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
            else:
                rust = closure_of(root, options.rust_strings_entry, options)
                for unit in rust.order:
                    if unit.language == "rust" and unit.first_token is not None:
                        workspace.file(unit.file)
            sources = WgslNameSources(workspace, rust.order)
        members, refusals, suppliers = wgsl_closure(files, entries, sources)
        for member in members:
            closure.add(member, None, "wgsl")
        closure.refusals.update(refusals)
        if suppliers and "wgsl-external-name-sources-unlinked" not in active_breaks():
            # 着落做成边：给外名着落的 Rust 单元连同它自己的闭包并进来（改了拼名字的那个 Rust 单元或它调到的，WGSL 入口的键就变）
            for supplier in sorted(suppliers, key=Unit.sort_key):
                target_id = rust.unit_targets.get(id(supplier))
                if target_id is None:
                    continue
                supplied = rust_closure_of(workspace, supplier, target_id, options.cfg, options.no_default_cfg, options.features, options.no_default_features)
                for member in supplied.order:
                    closure.add(member, supplier, "wgsl-name-source")
                closure.refusals.update(supplied.refusals)
                closure.data_sites |= supplied.data_sites
                closure.cfg_atoms = (closure.cfg_atoms or set()) | supplied.cfg_atoms
        return closure
    under = options.under or default_script_directories(root)
    directory = posixpath.dirname(path) or "."
    if not any(directory == place or directory.startswith(place.rstrip("/") + "/") for place in under):
        under = list(dict.fromkeys(list(under) + [directory]))
    script = ScriptClosure(root, under, options.excluded_files)
    shared = SHARED_SCRIPT_INDICES.get((os.path.abspath(root), tuple(under))) if "judging-entry-index-not-shared" not in script.breaks else None
    if shared is not None:
        script.python, script.shell = shared.python, shared.shell
    if language == "python":
        module = script.python.module(path)
        if module is None:
            raise UsageError(f"入口文件不在：{path}")
        if qualified:
            entries = [unit for unit in module.units if unit.name == qualified and not unit.data]
            if len(entries) != 1:
                raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个；dump 看限定名）")
        else:
            entries = module.program_entries()[:1]
        script.closure.refusals.update(module.refusals)
        return script.run(entries, None if qualified else options.program_arguments)
    index = script.shell_index([path])
    shell_file = index.files.get(path)
    if shell_file is None:
        raise UsageError(f"入口文件不在：{path}")
    script.closure.refusals.update(shell_file.refusals)
    if shell_file.script_body is None:
        return script.closure
    if qualified:
        entries = [unit for unit in shell_file.units if unit.name == qualified and not unit.data]
        if len(entries) != 1:
            raise UsageError(f"{path} 里限定名 {qualified} 命中 {len(entries)} 个单元（要恰好一个；dump 看限定名）")
    else:
        entries = [shell_file.script_body]
    return script.run(entries, None if qualified else options.program_arguments)


def closure_unit_lines(closure):
    """closure 与 key 命令打的单元行（dump_text）：数据目录单元照 crates/singlefs-unit-graph 按读它的那一处各一行
    （data-directory\\t<目录>\\t<读它的文件>:<行>-<行>\\t<目录指纹>），别的单元照常。弄坏开关 closure-directory-lines-span-files：
    数据目录单元打成它自己那一行（<目录>:1-<文件数>）。"""
    if "closure-directory-lines-span-files" in active_breaks():
        return [unit.dump_text() for unit in closure.units()]
    units = [unit for unit in closure.units() if unit.kind != "data-directory"]
    units += [Unit("data", "data-directory", site[2], site[0], site[1], site[1], site[3], data=True) for site in closure.data_sites if site[5] == "data-directory"]
    return [unit.dump_text() for unit in sorted(units, key=Unit.sort_key)]


def toolchain_version_line(label, command, root):
    """弄坏开关 python-toolchain-queries-keyed 下工具链单元的指纹那一行（ScriptClosure.toolchain_unit）：<label>: <输出>（多行用 | 连起来）；
    起不来、退非 0 的也记一行。"""
    try:
        completed = subprocess.run(command, cwd=root, capture_output=True, timeout=TOOLCHAIN_QUERY_SECONDS)
    except (OSError, subprocess.SubprocessError) as error:
        return f"{label}: 起不来（{type(error).__name__}）"
    output = " | ".join(line for line in completed.stdout.decode("utf-8", "replace").split("\n") if line.strip())
    return f"{label}: 退 {completed.returncode}: {output}"


# ── 命令行 ──────────────────────────────────────────────────────────────────

VALUE_OPTIONS = ("--language", "--under", "--target", "--cfg", "--feature", "--wgsl-module", "--rust-strings-entry", "--judgment-kind",
                 "--program-argument")


def parse_options(arguments):
    options, positional = Options(), []
    repeated = {"--under": options.under, "--cfg": options.cfg, "--feature": options.features, "--wgsl-module": options.wgsl_modules}
    i = 0
    while i < len(arguments):
        argument = arguments[i]
        if argument == "--no-default-cfg":
            options.no_default_cfg = True
            i += 1
            continue
        if argument == "--no-default-features":
            options.no_default_features = True
            i += 1
            continue
        if argument == "--explain":
            options.explain = True
            i += 1
            continue
        if argument == "--judging":
            options.judging = True
            i += 1
            continue
        if argument in VALUE_OPTIONS:
            if i + 1 >= len(arguments):
                raise UsageError(f"{argument} 后面要一个值")
            value = arguments[i + 1]
            if argument in repeated:
                repeated[argument].append(value)
            elif argument == "--language":
                options.language = value
            elif argument == "--target":
                options.target = value
            elif argument == "--judgment-kind":
                options.judgment_kind = value
            elif argument == "--program-argument":
                options.program_arguments = (options.program_arguments or []) + [value]
            else:
                options.rust_strings_entry = value
            i += 2
            continue
        if argument.startswith("--"):
            raise UsageError(f"认不出的选项：{argument}")
        positional.append(argument)
        i += 1
    if options.language not in (None, "rust", "python", "shell", "wgsl"):
        raise UsageError(f"--language 只认 rust、python、shell、wgsl：{options.language}")
    if options.judgment_kind not in JUDGMENT_KINDS:
        raise UsageError(f"--judgment-kind 只认 {'、'.join(JUDGMENT_KINDS)}：{options.judgment_kind}")
    return options, positional
