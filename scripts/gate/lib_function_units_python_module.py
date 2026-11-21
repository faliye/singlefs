"""Python 的模块与索引：PythonIndex、PythonModule（切单元、名字绑定与解析、值的折叠），其余方法在 lib_function_units_python_data_reads.py 与
lib_function_units_python_launches.py 的混入类里。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import collections
import copy
import os
import posixpath
import re
import sys
import lib_lag_split_family as split_family
from lib_function_units_lexer import active_breaks
from lib_function_units_base import (
    COMPREHENSION_UNPACKED, FOLD_LIMIT, POSITIONAL, Refusal, SKIPPED_DIRECTORIES, Unit, files_under, inside_root, sha256_bytes, sha256_text,
    snippet_of,
)
from lib_function_units_registry import SPEC_PATH_ARGUMENTS
from lib_function_units_python_facts import (
    ElementsOf, PATH_CONSTRUCTORS, PATH_TRANSFORMS, PYTHON_BUILTIN_CALLS, assignment_names, bound_names, dump_fingerprint, enclosing_unit,
    function_local_names, guard_assignments, import_bound_names, is_docstring, is_main_guard, paired_elements, stored_names,
    walk_function_statements, walk_nodes, without_docstrings,
)
from lib_function_units_python_data_reads import PythonModuleDataReads
from lib_function_units_python_launches import PythonModuleLaunches

# 内联 python 正文里 import 的顶层模块名（from x import …、import x）：PythonModule.inherited_environment 按它认哪几份内联脚本载入了这一族
INLINE_IMPORTED_NAME = re.compile(r"^[ \t]*(?:from[ \t]+([A-Za-z_]\w*)[ \t]+import|import[ \t]+([A-Za-z_]\w*))", re.M)


class PythonIndex:
    def __init__(self, root):
        self.root = os.path.abspath(root)
        self.breaks = active_breaks()
        self.modules = {}
        self.folding = set()
        # 折出的取值超过 FOLD_LIMIT 种的次数（PythonModule.fold、path_tails、command_alternatives 那几处加一，只增不减）：
        # 读文件、起子进程那几处比前后两次的数，认「折得出、只是太多」，拒算 FOLD_LIMIT_FORM（文件头「拒算」）
        self.fold_limit_hits = 0
        self._repository_names = None
        self._repository_files = None
        # 运行期读的数据文件与目录的单元（ScriptClosure.data_unit、data_directory）：共用这个索引的闭包共用
        self.data_units, self.data_directories = {}, {}
        # shell 脚本（相对根）→ 它起的内联 python（[InlineScript]）：ScriptClosure 给（entry_inline_scripts），拆出的模块照它认入口设的环境变量（PythonModule.inherited_environment）
        self.entry_inline_scripts = None
        # exec 进模块对象的一段正文（InlineScript）→ {键: (载入它的模块, 那个 types.ModuleType 调用)}：PythonModuleLaunches.exec_facts 记，
        # 正文里模块级函数的参数照载入方的调用处折（PythonModule.module_object_loader_call_sites）
        self.module_object_loaders = {}

    def repository_files(self):
        """根里的全部文件（相对根，跳过 SKIPPED_DIRECTORIES）：运行期读的路径只认得出字面尾巴时，按尾巴对上的那一组（PythonModule.read_origins 的 tail）。"""
        if self._repository_files is None:
            self._repository_files = files_under(self.root, ("",))
        return self._repository_files

    def module(self, path):
        if path not in self.modules:
            self.modules[path] = PythonModule(self, path) if os.path.isfile(os.path.join(self.root, path)) else None
        return self.modules[path]

    def inline_modules(self, script):
        """InlineScript（python 的）→ 每份正文一个 PythonModule：单元都标成内联（闭包穿过它们连下去、自己不进闭包，正文已在起它的那个单元里），
        sys.argv[k] 折成 script.argv[k-1]，import 先在 script.bases（工作目录）里找，拒算落在起它的那一行。"""
        key = ("inline", script)
        if key not in self.modules:
            self.modules[key] = [PythonModule(self, f"{script.file}#inline@{script.line}#{position}", inline=script, text=text)
                                 for position, text in enumerate(script.texts)]
        return self.modules[key]

    def repository_names(self):
        if self._repository_names is None:
            names = set()
            for directory, subdirectories, files in os.walk(self.root):
                subdirectories[:] = [name for name in subdirectories if name not in SKIPPED_DIRECTORIES]
                for name in files:
                    if name.endswith(".py"):
                        names.add(name[:-3])
                if "__init__.py" in files:
                    names.add(os.path.basename(directory))
            self._repository_names = names
        return self._repository_names

    def relative(self, absolute):
        return inside_root(self.root, absolute)


class PythonModule(PythonModuleDataReads, PythonModuleLaunches):
    def __init__(self, index, path, inline=None, text=None):
        self.index, self.path, self.inline = index, path, inline
        if inline is not None:
            self.absolute = posixpath.join(inline.bases[0], "<inline>")
            self.raw = text.encode("utf-8", "surrogateescape")
        else:
            self.absolute = os.path.join(index.root, path)
            with open(self.absolute, "rb") as handle:
                self.raw = handle.read()
        self.text = self.raw.decode("utf-8", "surrogateescape")
        self.rows = self.text.split("\n")
        self.units, self.refusals, self.methods = [], [], []
        self.by_name = collections.defaultdict(list)
        self.body_defines, self.docstrings = set(), {}
        self.module_body = self.main_guard = None
        self.star_imports = []
        self._bindings = self._call_sites = self._sys_path = self._importer_directories = None
        self._data = {}
        breaks = index.breaks
        self.keep_comments = "python-comments-kept" in breaks
        self.keep_docstrings = "python-docstrings-kept" in breaks
        try:
            self.tree = ast.parse(self.raw, filename=path)
        except (SyntaxError, ValueError) as error:
            line = getattr(error, "lineno", None) or 1
            self.refusals.append(self.refusal("python-syntax", line))
            self.tree = None
            body_unit = Unit("python", "module-body", "<module-body>", path, 1, 1, sha256_bytes(self.raw))
            body_unit.module, body_unit.nodes = self, []
            self.module_body = body_unit
            self.units.append(body_unit)
        else:
            self.split()
        if inline is not None:
            for unit in self.units:
                unit.scope, unit.file = "inline", inline.file

    def row(self, line):
        if self.inline is not None:
            return self.inline.snippet
        return snippet_of(self.rows[line - 1]) if 0 < line <= len(self.rows) else ""

    def refusal(self, form, line):
        """拒算落在哪：正文出自原文里一份 here-doc 的内联脚本落在原文里那一行（InlineScript 的 body_lines 里这份正文的行号加正文里的偏移）；字面 -c 串与原文里找不到正文的
        落在起它的那一行（它的正文就在那个单元里）；别的落在这份文件的那一行。弄坏开关 inline-refusal-lines-at-launch 下 here-doc 的也落在起它的那一行。"""
        if self.inline is not None:
            body_line = dict(self.inline.body_lines).get(self.text)
            if body_line is not None and 0 < line <= len(self.rows):
                return Refusal(form, self.source_file(), body_line + line - 1, snippet_of(self.rows[line - 1]))
            return Refusal(form, self.inline.file, self.inline.line, self.inline.snippet)
        return Refusal(form, self.path, line, self.row(line))

    def add_unit(self, kind, name, node, nodes, fingerprint, container=None, short=None):
        unit = Unit("python", kind, name, self.path, node.lineno, getattr(node, "end_lineno", node.lineno) or node.lineno, fingerprint)
        decorators = getattr(node, "decorator_list", None)
        if decorators:
            unit.first_line = min(unit.first_line, min(decorator.lineno for decorator in decorators))
        unit.node, unit.nodes, unit.module, unit.container = node, nodes, self, container
        unit.short = short or name
        self.units.append(unit)
        if container is None and kind != "main-guard":
            self.by_name[name].append(unit)
        return unit

    def add_def_head(self, statement, name):
        """import 时就求值的 def 头（文件头「Python」一段的 def-head）：带装饰器或缺省值的 def（模块级的与模块级类里的方法）另切一个单元，
        只收装饰器与缺省值那几个表达式（指纹与边都只看它们），不收函数体、不进 by_name（不被按名字点到）、不挂类头（不让类算被点名）。
        限定名 <def 的限定名>.<head>。弄坏开关 python-def-heads-ignored 下不切。"""
        arguments = statement.args
        evaluated = list(statement.decorator_list) + list(arguments.defaults) + [value for value in arguments.kw_defaults if value is not None]
        if not evaluated or "python-def-heads-ignored" in self.index.breaks:
            return None
        first = min(node.lineno for node in evaluated)
        last = max(getattr(node, "end_lineno", None) or node.lineno for node in evaluated)
        head = Unit("python", "def-head", f"{name}.<head>", self.path, first, last,
                    dump_fingerprint(ast.Module(body=[ast.Expr(value=node) for node in evaluated], type_ignores=[])))
        head.node, head.nodes, head.module, head.container = None, evaluated, self, None
        self.units.append(head)
        return head

    def function_fingerprint(self, node):
        if self.keep_comments:
            return sha256_text(ast.get_source_segment(self.text, node) or "")
        return dump_fingerprint(without_docstrings(node, self.keep_docstrings))

    def note_docstring(self, owner, node):
        if node.body and is_docstring(node.body[0]):
            statement = node.body[0]
            self.docstrings[owner] = (statement.value.value, statement.lineno, statement.end_lineno or statement.lineno)

    def split(self):
        body = list(self.tree.body)
        if body and is_docstring(body[0]):
            statement = body[0]
            self.docstrings["<module>"] = (statement.value.value, statement.lineno, statement.end_lineno or statement.lineno)
            body = body[1:]
        rest, guards = [], []
        for statement in body:
            if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.add_unit("def", statement.name, statement, [statement], self.function_fingerprint(statement))
                self.add_def_head(statement, statement.name)
                self.note_docstring(statement.name, statement)
            elif isinstance(statement, ast.ClassDef):
                head_body = [member for member in statement.body if not isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef))]
                clone = copy.copy(statement)
                clone.body = head_body
                head = self.add_unit("class", statement.name, statement,
                                     list(statement.decorator_list) + list(statement.bases) + list(statement.keywords) + head_body,
                                     dump_fingerprint(without_docstrings(clone, self.keep_docstrings)))
                self.note_docstring(statement.name, statement)
                for member in statement.body:
                    if isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        method = self.add_unit("def", f"{statement.name}.{member.name}", member, [member], self.function_fingerprint(member),
                                               container=head, short=member.name)
                        self.add_def_head(member, f"{statement.name}.{member.name}")
                        self.note_docstring(method.name, member)
                        self.methods.append(method)
            elif isinstance(statement, (ast.Assign, ast.AnnAssign, ast.AugAssign)) and assignment_names(statement):
                for name in assignment_names(statement):
                    self.add_unit("assign", name, statement, [statement], dump_fingerprint(statement))
            elif isinstance(statement, (ast.Import, ast.ImportFrom)):
                for name in import_bound_names(statement):
                    unit = self.add_unit("import", name, statement, [statement], dump_fingerprint(statement))
                    if name == "*":
                        self.star_imports.append(unit)
            elif is_main_guard(statement, self.index.breaks):
                if statement.orelse and "python-main-guard-else-dropped" not in self.index.breaks:
                    # 标准写法带 else（或 elif）：只有 if 那一支是 main-guard；else 那一支被 import 时执行，照别的模块级语句进 module-body。
                    # 弄坏开关 python-main-guard-else-dropped：连 else 整段当 main-guard（被 import 时 else 那一支哪儿都不收）
                    main_branch = copy.copy(statement)
                    main_branch.orelse = []
                    main_branch.end_lineno = statement.body[-1].end_lineno or statement.body[-1].lineno
                    guards.append(main_branch)
                    rest.extend(statement.orelse)
                else:
                    guards.append(statement)
            else:
                rest.append(statement)
        if "python-main-guards-last-only" in self.index.breaks:
            guards = guards[-1:]
        if guards:
            # 一份文件里几段 main-guard 合成一个单元（被当程序起时每一段都执行）：节点是那几段，只有一段时指纹照旧取那一段；
            # 弄坏开关 python-main-guards-last-only：只留末一段（前面几段被后面的盖掉、哪儿都不收）
            fingerprint = dump_fingerprint(guards[0]) if len(guards) == 1 else dump_fingerprint(ast.Module(body=guards, type_ignores=[]))
            self.main_guard = self.add_unit("main-guard", "<main-guard>", guards[0], guards, fingerprint)
            self.main_guard.last_line = guards[-1].end_lineno or guards[-1].lineno
        module = ast.Module(body=without_docstrings(ast.Module(body=rest, type_ignores=[]), self.keep_docstrings).body, type_ignores=[])
        first = rest[0].lineno if rest else 1
        last = (rest[-1].end_lineno or rest[-1].lineno) if rest else 1
        self.module_body = Unit("python", "module-body", "<module-body>", self.path, first, last, dump_fingerprint(module))
        self.module_body.node, self.module_body.nodes, self.module_body.module = None, rest, self
        self.units.append(self.module_body)
        for statement in rest:
            self.body_defines |= stored_names(statement)

    def program_entries(self):
        return [unit for unit in (self.main_guard, self.module_body) if unit is not None]

    # 绑定与解析

    def bindings(self):
        if self._bindings is None:
            self._bindings = {}
            if self.tree is not None:
                statements = [unit.node for unit in self.units if unit.kind == "import"] + list(self.module_body.nodes)
                for node in walk_nodes(statements):
                    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
                        continue
                    self.bind(node, self._bindings)
        return self._bindings

    @staticmethod
    def bind(node, table):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.asname:
                    table[alias.asname] = ("import", alias.name, 0, node)
                else:
                    table[alias.name.split(".")[0]] = ("import", alias.name.split(".")[0], 0, node)
        elif isinstance(node, ast.ImportFrom):
            for alias in node.names:
                if alias.name != "*":
                    table[alias.asname or alias.name] = ("from", node.module or "", node.level, node, alias.name)

    def local_bindings(self, unit):
        if unit.cache is None:
            unit.cache = {}
        if "bindings" not in unit.cache:
            table = {}
            if unit.kind in ("def", "class", "main-guard"):
                for node in walk_nodes(unit.nodes):
                    self.bind(node, table)
            unit.cache["bindings"] = table
        return unit.cache["bindings"]

    def binding(self, name, unit):
        local = self.local_bindings(unit).get(name) if unit is not None else None
        if local is None and unit is not None and enclosing_unit(unit) is not None:
            # 套在函数里的函数（nested_unit）：自己没绑的名字到外层函数里找
            return self.binding(name, enclosing_unit(unit))
        return local if local is not None else self.bindings().get(name)

    def nested_unit(self, definition, enclosing):
        """套在 enclosing 里的函数 definition 自成一层作用域（文件头「起代码的写法逐种认」嵌套函数）：一个只用来折叠的单元，不进单元表、不进闭包，
        局部赋值与参数照它自己的，别的名字到 enclosing 里找（enclosing_unit），参数按 enclosing 里调它的那几处折（function_call_sites）。"""
        nested = self.__dict__.setdefault("_nested_units", {})
        if id(definition) not in nested:
            unit = Unit("python", "def", f"{enclosing.name}.<locals>.{definition.name}", self.path, definition.lineno,
                        getattr(definition, "end_lineno", None) or definition.lineno, "")
            unit.node, unit.nodes, unit.module, unit.container = definition, [definition], self, None
            unit.cache = {"enclosing": enclosing}
            nested[id(definition)] = unit
        return nested[id(definition)]

    def scope_of(self, node, unit):
        """单元里一个节点所在的那一层函数作用域：套在单元里的 def 里面的落到那个 def 的 nested_unit（最里层那一个），别的就是单元自己。
        弄坏开关 python-nested-scopes-unfolded 下一律是单元自己（套着的函数里的局部变量与参数折不出）。"""
        if unit is None or "python-nested-scopes-unfolded" in self.index.breaks:
            return unit
        if unit.cache is None:
            unit.cache = {}
        scopes = unit.cache.get("scopes")
        if scopes is None:
            scopes = {}
            own = unit.node if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None

            def visit(current, scope):
                for child in ast.iter_child_nodes(current):
                    inner = scope
                    if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)) and child is not own:
                        inner = self.nested_unit(child, scope)
                    scopes[id(child)] = inner
                    visit(child, inner)
            for top in unit.nodes or []:
                scopes[id(top)] = unit
                visit(top, unit)
            unit.cache["scopes"] = scopes
        return scopes.get(id(node), unit)

    def function_call_sites(self, unit):
        """调 unit 这个函数的那几处 [(调用节点, 调用处的作用域单元)]：模块级函数取本模块里按名字调它的每一处（call_sites），连同同一个拆分族里 import 它的模块里的
        （importer_call_sites），exec 进模块对象的正文里的连同载入方里的（module_object_loader_call_sites）；套在函数里的（nested_unit）只取外层函数里调它的那几处。"""
        function = unit.node
        enclosing = enclosing_unit(unit)
        if enclosing is None:
            return list(self.call_sites().get(function.name, [])) + self.importer_call_sites(function.name) \
                + self.module_object_loader_call_sites(function.name)
        return [(node, self.scope_of(node, enclosing)) for node in walk_nodes(enclosing.nodes or [])
                if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == function.name]

    def search_directories(self):
        # 内联脚本（python3 -c、python3 - <<'PY'）的 sys.path[0] 是工作目录
        directories = list(self.inline.bases) if self.inline is not None else [posixpath.dirname(self.absolute)]
        if self._sys_path is None:
            self._sys_path = []
            if self.tree is not None:
                self._sys_path = ["computing"]
                found = []
                for unit in self.units:
                    for node in walk_nodes(unit.nodes):
                        arguments = []
                        if isinstance(node, ast.Call):
                            canonical = self.canonical(node.func, unit)
                            if canonical == "sys.path.insert" and len(node.args) >= 2:
                                arguments = [node.args[1]]
                            elif canonical == "sys.path.append" and node.args:
                                arguments = [node.args[0]]
                            elif canonical == "sys.path.extend" and node.args and isinstance(node.args[0], (ast.List, ast.Tuple)):
                                arguments = list(node.args[0].elts)
                        elif isinstance(node, ast.Assign) and isinstance(node.value, (ast.List, ast.Tuple)) and any(
                                isinstance(target, ast.Subscript) and self.canonical(target.value, unit) == "sys.path" for target in node.targets):
                            arguments = list(node.value.elts)
                        for argument in arguments:
                            values = self.path_values(argument, unit, directory=True)
                            if values:
                                found += sorted(values)
                self._sys_path = found
                if "python-imported-sys-path-ignored" not in self.index.breaks:
                    # 模块级 import 进来的、照 SPEC_PATH_ARGUMENTS 载入的仓内模块在载入时往 sys.path 加的目录，对这份模块同样生效（sys.path 是进程里的一份）
                    files = []
                    for statement in self.tree.body:
                        names = []
                        if isinstance(statement, ast.Import):
                            names = [(alias.name, 0) for alias in statement.names]
                        elif isinstance(statement, ast.ImportFrom) and statement.module:
                            names = [(statement.module, statement.level)]
                        for dotted, level in names:
                            status, file = self.resolve_module(dotted, level)
                            if status == "repo":
                                files.append(file)
                    for (loader, _expression), targets in SPEC_PATH_ARGUMENTS.items():
                        if loader == self.source_file():
                            files += [targets] if isinstance(targets, str) else list(targets)
                    for file in files:
                        imported = self.index.module(file) if file != self.path and os.path.isfile(os.path.join(self.index.root, file)) else None
                        if imported is not None and imported is not self:
                            imported.search_directories()
                            found += [directory for directory in imported._sys_path if directory != "computing" and directory not in found]
        return directories + [path for path in self._sys_path if path != "computing"]

    def imports_module(self, name):
        """这份模块里（模块级与函数里的都算：函数里的 import 跑在模块级插过 sys.path 之后）import 了名字是 name 的模块（import name、from name import …，不带点）。"""
        if self.tree is None:
            return False
        for node in ast.walk(self.tree):
            if isinstance(node, ast.Import) and any(alias.name == name for alias in node.names):
                return True
            if isinstance(node, ast.ImportFrom) and not node.level and node.module == name:
                return True
        return False

    def importer_directories(self):
        """同一个拆分族里 import 这份模块的那几份（importer_modules；拆成入口加同目录模块：入口先往 sys.path 插目录、再 import 拆出的模块）往 sys.path 加的目录，
        连同 import 它们的那几份加的（传递地往上找）：拆出的模块自己不插 sys.path，它 import 的东西照入口插的目录解（文件头「import 它的模块加的 sys.path」）。
        内联脚本与解不开的交空。弄坏开关 python-importer-sys-path-ignored 下一律交空。"""
        if self._importer_directories is not None:
            return self._importer_directories
        self._importer_directories = []   # 先占上：互相 import 的两份不绕圈
        if self.inline is not None or self.tree is None or "python-importer-sys-path-ignored" in self.index.breaks:
            return self._importer_directories
        found = []
        for importer in self.importer_modules():
            for base in importer.search_directories() + importer.importer_directories():
                if base not in found:
                    found.append(base)
        self._importer_directories = found
        return found

    def split_family_root(self, relative):
        """relative（同目录里的一份 .py）归到的拆分族入口：lib_lag_split_family.family_owner，基准取同目录里别的 .py、.sh（拆法照它文件头：
        入口 <名>.py / <名>.sh 加 <名去掉后缀、「-」换「_」>_<部分>.py，最长前缀取最近一层）；都不归的是它自己。按路径缓存在索引上。"""
        roots = self.index.__dict__.setdefault("_split_family_roots", {})
        if relative not in roots:
            directory = posixpath.dirname(relative)
            listings = self.index.__dict__.setdefault("_split_family_listings", {})
            if directory not in listings:
                absolute = posixpath.join(self.index.root, directory) if directory else self.index.root
                listings[directory] = frozenset(posixpath.join(directory, entry) if directory else entry for entry in sorted(os.listdir(absolute))
                                                if entry.endswith((".py", ".sh")) and os.path.isfile(posixpath.join(absolute, entry)))
            roots[relative] = split_family.family_owner(relative, listings[directory] - {relative}) or relative
        return roots[relative]

    def same_split_family(self, other):
        """这份模块与同目录的 other 是不是同一个拆分族（split_family_root）：归到同一个入口，或一份是另一份归到的入口。
        拆分前是同一份文件的那几份才算：同目录里别的 import 它的脚本（共用库的调用方）不算。弄坏开关 python-importer-sites-outside-family 下一律算。"""
        if "python-importer-sites-outside-family" in self.index.breaks:
            return True
        own, theirs = self.index.relative(self.absolute), self.index.relative(other.absolute)
        if own is None or theirs is None:
            return False
        own_root, their_root = self.split_family_root(own), self.split_family_root(theirs)
        return own_root == their_root or own_root == theirs or their_root == own

    def importer_modules(self):
        """同目录里 import 这份模块、又与它同一个拆分族（same_split_family）的那几份（imports_module；入口与拆出的模块互相 import）→ [PythonModule]，按文件名排。
        先按原文里有没有这份模块的名字筛，有的才切单元；内联脚本与解不开的交空。共用库（lib_process_group.py 这类）的调用方不在这里：
        库里函数的参数只照库自己里的调用处折，折不出的由收闭包的一侧照闭包里调它的实参往上认（算键库文件头「调用方交进来的命令」），
        不把同目录每个调用方交的命令都连进每一个经它的闭包。"""
        cached = self.__dict__.get("_importer_modules")
        if cached is not None:
            return cached
        self._importer_modules = []
        if self.inline is not None or self.tree is None:
            return self._importer_modules
        directory = posixpath.dirname(self.absolute)
        own_name = posixpath.basename(self.path)
        stem = own_name[:-len(".py")] if own_name.endswith(".py") else own_name
        # 同一个目录的原文一个索引只读一遍（目录里每份模块各问一次）
        directory_texts = self.index.__dict__.setdefault("_importer_scan_texts", {})
        if directory not in directory_texts:
            texts = {}
            for entry in sorted(os.listdir(directory)):
                if entry.endswith(".py"):
                    try:
                        with open(posixpath.join(directory, entry), "rb") as handle:
                            texts[entry] = handle.read()
                    except OSError:
                        continue
            directory_texts[directory] = texts
        found = []
        for entry, raw in sorted(directory_texts[directory].items()):
            if entry == own_name or stem.encode("utf-8") not in raw:
                continue
            relative = self.index.relative(posixpath.join(directory, entry))
            importer = self.index.module(relative) if relative is not None else None
            if importer is not None and importer.imports_module(stem) and self.same_split_family(importer):
                found.append(importer)
        self._importer_modules = found
        return found

    def importer_call_sites(self, name, seen=None):
        """模块级函数 name 在同一个拆分族里 import 这份模块的模块（importer_modules）里被调的那几处 → [(调用节点, 调用处的作用域单元)]：拆分把函数挪进拆出的模块，
        调它的留在入口或别的拆出模块里（文件头「拆出的模块里的函数，参数照 import 它的模块里的调用处折」）。那份模块里按 from <这份> import name [as 别名] 绑的名字、
        from <这份> import *（自己没定义 name 时）调的、import <这份> [as 别名] 之后 <别名>.name(…) 调的都算；那份模块又把这个名字按名字或星号再交出去的，
        接着往 import 它的那几份找。弄坏开关 python-importer-call-sites-ignored 下交空。"""
        if self.inline is not None or self.tree is None or "python-importer-call-sites-ignored" in self.index.breaks:
            return []
        seen = set() if seen is None else seen
        if (self.path, name) in seen:
            return []
        seen.add((self.path, name))
        own_name = posixpath.basename(self.path)
        stem = own_name[:-len(".py")] if own_name.endswith(".py") else own_name
        sites = []
        for importer in self.importer_modules():
            aliases, module_aliases, star = set(), set(), False
            for node in ast.walk(importer.tree):
                if isinstance(node, ast.ImportFrom) and not node.level and node.module == stem:
                    for alias in node.names:
                        if alias.name == "*":
                            star = True
                        elif alias.name == name:
                            aliases.add(alias.asname or alias.name)
                elif isinstance(node, ast.Import):
                    module_aliases |= {alias.asname or alias.name for alias in node.names if alias.name == stem}
            # 星号只在那份模块自己没另绑这个名字时绑到这一份的 name（自己定义的、从别处 import 的都盖过它）
            if star and not any(not (unit.kind == "import" and isinstance(unit.node, ast.ImportFrom) and unit.node.module == stem and not unit.node.level)
                                for unit in importer.by_name.get(name, ())):
                aliases.add(name)
            calls = importer.call_sites()
            for alias in sorted(aliases):
                sites += calls.get(alias, [])
            if module_aliases:
                for unit in importer.units:
                    for node in walk_nodes(unit.nodes or []):
                        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == name \
                                and isinstance(node.func.value, ast.Name) and node.func.value.id in module_aliases:
                            sites.append((node, importer.scope_of(node, unit)))
            for alias in sorted(aliases):
                sites += importer.importer_call_sites(alias, seen)
        return sites

    def importer_method_call_sites(self, method_name):
        """类里的方法 method_name 在拆分族里被调的那几处 → [(调用节点, 调用处的作用域单元)]：import 这份模块的同族模块（importer_modules），
        连同那几份 import 的同族模块（拆成混入类时，几个混入类由同一份模块 import 进来拼成一个类，方法在哪一份里调都一样），每一处 <任何东西>.<方法名>(…)。
        弄坏开关 python-importer-call-sites-ignored 下交空。"""
        if self.inline is not None or self.tree is None or "python-importer-call-sites-ignored" in self.index.breaks:
            return []
        family = {}
        for importer in self.importer_modules():
            family.setdefault(importer.path, importer)
            for unit in importer.units:
                if unit.kind != "import" or unit.container is not None:
                    continue
                for binding in importer.local_import_bindings(unit).values():
                    status, file = importer.resolve_module(binding[1], binding[2])
                    sibling = self.index.module(file) if status == "repo" and posixpath.dirname(file) == posixpath.dirname(self.path) else None
                    if sibling is not None and sibling is not self and sibling.inline is None and self.same_split_family(sibling):
                        family.setdefault(sibling.path, sibling)
        sites = []
        for path in sorted(family):
            member = family[path]
            for owner in member.units:
                for node in walk_nodes(owner.nodes or []):
                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == method_name:
                        sites.append((node, member.scope_of(node, owner)))
        return sites

    def module_object_loader_call_sites(self, name):
        """这份是 exec 进模块对象的一段正文（PythonIndex.module_object_loaders 记了载入它的那几处；文件头「exec 进模块对象」）时，模块级函数 name
        在载入方里被调的那几处 → [(调用节点, 调用处的作用域单元)]：载入方里绑这个模块对象的名字（module_object_root 追得到同一个 types.ModuleType 调用的
        模块级名字，连同 <别名> = <名>）上的 <名>.name(…)。只看 exec 它的那一份载入方，别的载入方载入同一段正文的调用处不算。
        弄坏开关 python-module-object-loader-call-sites-ignored 下交空（exec_facts 不记载入方）。"""
        if self.inline is None:
            return []
        sites = []
        for loader, root in self.index.module_object_loaders.get(self.inline, {}).values():
            if loader.tree is None:
                continue
            aliases = set()
            for statement in ast.walk(loader.tree):
                if isinstance(statement, ast.Assign) and len(statement.targets) == 1 and isinstance(statement.targets[0], ast.Name):
                    found = loader.module_object_root(statement.targets[0].id, loader.module_body)
                    if found is not None and found[0] is root:
                        aliases.add(statement.targets[0].id)
            for owner in loader.units:
                for node in walk_nodes(owner.nodes or []):
                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == name \
                            and isinstance(node.func.value, ast.Name) and node.func.value.id in aliases:
                        sites.append((node, loader.scope_of(node, owner)))
        return sites

    def resolve_module(self, dotted, level):
        """→ ("repo", 文件) / ("stdlib", None) / ("external", None) / ("unresolved", None)。"""
        parts = [part for part in dotted.split(".") if part] if dotted else []
        if level:
            base = posixpath.dirname(self.absolute)
            for _step in range(level - 1):
                base = posixpath.dirname(base)
            bases = [base]
        else:
            bases = self.search_directories()
        for base in bases:
            candidate = posixpath.join(base, *parts) if parts else base
            for option in (candidate + ".py", posixpath.join(candidate, "__init__.py")):
                if parts or option.endswith("__init__.py"):
                    if os.path.isfile(option):
                        relative = self.index.relative(option)
                        if relative is not None:
                            return ("repo", relative)
        if level:
            return ("unresolved", None)
        top = parts[0] if parts else ""
        if parts and top not in sys.stdlib_module_names and top not in sys.builtin_module_names and top in self.index.repository_names():
            # 仓里有这个名字、自己的搜索目录里没有：拆分族里 import 这份模块的入口插的目录（importer_directories）
            for base in self.importer_directories():
                if base in bases:
                    continue
                candidate = posixpath.join(base, *parts)
                for option in (candidate + ".py", posixpath.join(candidate, "__init__.py")):
                    if os.path.isfile(option):
                        relative = self.index.relative(option)
                        if relative is not None:
                            return ("repo", relative)
        if top in sys.stdlib_module_names or top in sys.builtin_module_names:
            return ("stdlib", None)
        if top in self.index.repository_names():
            return ("unresolved", None)
        return ("external", None)

    def all_resolutions(self, dotted, level):
        """同一个名字在各个搜索目录里各解到的仓内文件（sys.path 里插的目录折出几个，运行期是其中哪一个都可能）；resolve_module 只交第一个。"""
        if level or "python-imports-first-directory-only" in self.index.breaks:
            status, file = self.resolve_module(dotted, level)
            return [file] if status == "repo" else []
        parts = [part for part in dotted.split(".") if part] if dotted else []
        found = []
        for base in self.search_directories():
            candidate = posixpath.join(base, *parts) if parts else base
            for option in (candidate + ".py", posixpath.join(candidate, "__init__.py")):
                if (parts or option.endswith("__init__.py")) and os.path.isfile(option):
                    relative = self.index.relative(option)
                    if relative is not None and relative not in found:
                        found.append(relative)
                    break
        return found

    def canonical(self, node, unit):
        if isinstance(node, ast.Name):
            binding = self.binding(node.id, unit)
            if binding is None:
                return node.id if node.id in PYTHON_BUILTIN_CALLS else None
            if binding[0] == "import":
                return binding[1]
            if binding[2]:
                return None
            return f"{binding[1]}.{binding[4]}"
        if isinstance(node, ast.Attribute):
            base = self.canonical(node.value, unit)
            return f"{base}.{node.attr}" if base else None
        return None

    def binding_targets(self, binding, unit, facts):
        """一个绑定 → 它指到的仓内单元（那份模块的 module-body 与同名单元）；仓内认不出的记拒算。"""
        node = binding[3]
        if binding[0] == "import":
            status, file = self.resolve_module(binding[1], 0)
            if status == "unresolved":
                facts.refusals.append(self.refusal("import-unresolved", node.lineno))
            if status != "repo":
                return []
            targets = []
            for each in self.all_resolutions(binding[1], 0) or [file]:
                module = self.index.module(each)
                targets += module.program_entries()[-1:] if module else []
            return targets
        status, file = self.resolve_module(binding[1], binding[2])
        if status == "unresolved":
            facts.refusals.append(self.refusal("import-unresolved", node.lineno))
        if status != "repo":
            return []
        others = [each for each in self.all_resolutions(binding[1], binding[2]) if each != file]
        targets = []
        for each in others:
            other_module = self.index.module(each)
            if other_module is not None:
                targets += [other_module.module_body] + list(other_module.by_name.get(binding[4], ()))
        module = self.index.module(file)
        if module is None:
            return targets
        attribute = binding[4]
        targets += [module.module_body] + list(module.by_name.get(attribute, ()))
        if attribute in module.body_defines:
            targets.append(module.module_body)
        sub_status, sub_file = self.resolve_module(f"{binding[1]}.{attribute}" if binding[1] else attribute, binding[2])
        if sub_status == "repo" and sub_file != file:
            sub_module = self.index.module(sub_file)
            if sub_module is not None:
                targets.append(sub_module.module_body)
        return targets

    def module_of_binding(self, binding):
        if binding is None:
            return None
        if binding[0] == "import":
            status, file = self.resolve_module(binding[1], 0)
        else:
            dotted = f"{binding[1]}.{binding[4]}" if binding[1] else binding[4]
            status, file = self.resolve_module(dotted, binding[2])
        return self.index.module(file) if status == "repo" else None

    def resolve_name(self, name, unit, facts):
        binding = self.local_bindings(unit).get(name)
        if binding is not None:
            return self.binding_targets(binding, unit, facts)
        found = [candidate for candidate in self.by_name.get(name, ()) if candidate is not unit]
        if found:
            return found
        if name in self.body_defines and unit is not self.module_body:
            return [self.module_body]
        out = []
        for star in self.star_imports:
            for target in self.binding_targets(("from", star.node.module or "", star.node.level, star.node, "*"), unit, facts):
                module = target.module
                if module is not None:
                    out += list(module.by_name.get(name, ()))
        return out

    def resolve_attribute(self, base, attribute, unit, facts):
        """base.attribute：base 是 import 进来的仓内模块时 → 那份模块的同名单元；不是 → None（交 RTA）。"""
        if not base:
            return None
        parts = base.split(".")
        binding = self.binding(parts[0], unit)
        first = self.module_of_binding(binding)
        if first is None:
            return None
        modules = [first]
        if binding[0] == "import" and len(parts) == 1:
            # 同一个模块名在 sys.path 插的几个目录里各有一份：每一份的同名单元都连
            modules += [self.index.module(file) for file in self.all_resolutions(binding[1], 0) if file != first.path]
        targets = []
        for module in modules:
            if module is None:
                continue
            for part in parts[1:]:
                status, file = module.resolve_module(part, 1) if module.path.endswith("__init__.py") else ("unresolved", None)
                if status != "repo":
                    return None
                module = self.index.module(file)
                if module is None:
                    return None
            targets += [module.module_body] + list(module.by_name.get(attribute, ()))
            if module.path.endswith("__init__.py"):
                status, file = module.resolve_module(attribute, 1)
                if status == "repo":
                    sub_module = self.index.module(file)
                    if sub_module is not None:
                        targets.append(sub_module.module_body)
        return targets

    # 折叠

    def local_assignments(self, unit):
        if unit.cache is None:
            unit.cache = {}
        if "assignments" not in unit.cache:
            result = collections.defaultdict(list)
            if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                stack = list(unit.node.body)
                own_scope = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)
                if "python-comprehension-scope-leaks" not in self.index.breaks:
                    # 推导式自成一层作用域：里面的循环变量不是这个函数的赋值
                    own_scope += (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)
                while stack:
                    node = stack.pop()
                    if isinstance(node, own_scope):
                        continue
                    if isinstance(node, ast.Assign):
                        for target in node.targets:
                            if isinstance(target, ast.Name):
                                result[target.id].append(node.value)
                            elif paired_elements(target, node.value) is not None:
                                # a, b = x, y：逐个对上
                                for name, value in paired_elements(target, node.value):
                                    result[name].append(value)
                            else:
                                # 拆包（a, b = …）绑的是里面的名字；x[k] = …、x.y = … 只改 x 那个对象，不重新绑 k、x
                                rebound = ([inner.id for inner in ast.walk(target) if isinstance(inner, ast.Name)]
                                           if "python-subscript-targets-rebind" in self.index.breaks else bound_names(target))
                                for inner in rebound:
                                    result[inner].append(None)
                    elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                        result[node.target.id].append(node.value)
                    elif isinstance(node, ast.AugAssign) and isinstance(node.target, ast.Name):
                        result[node.target.id].append(None)
                    elif isinstance(node, (ast.For, ast.AsyncFor, ast.comprehension)):
                        if isinstance(node.target, ast.Name) and "python-loop-variables-unfolded" not in self.index.breaks:
                            # for x in <元组 / 列表，或绑着元组 / 列表的名字>：x 取遍那几个元素
                            result[node.target.id].append(ElementsOf(sequence=node.iter))
                        else:
                            for inner in ast.walk(node.target):
                                if isinstance(inner, ast.Name):
                                    result[inner.id].append(None)
                    elif isinstance(node, (ast.With, ast.AsyncWith)):
                        for item in node.items:
                            if item.optional_vars is not None:
                                for inner in ast.walk(item.optional_vars):
                                    if isinstance(inner, ast.Name):
                                        result[inner.id].append(None)
                    elif isinstance(node, ast.NamedExpr):
                        result[node.target.id].append(node.value)
                    stack.extend(ast.iter_child_nodes(node))
            unit.cache["assignments"] = result
        return unit.cache["assignments"]

    def call_sites(self):
        if self._call_sites is None:
            self._call_sites = collections.defaultdict(list)
            for unit in self.units:
                for node in walk_nodes(unit.nodes or []):
                    if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                        # 调用处在套着的函数里的，实参照那一层作用域折（scope_of）
                        self._call_sites[node.func.id].append((node, self.scope_of(node, unit)))
                    elif isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.args and \
                            node.func.attr in ("submit", "map") and isinstance(node.args[0], ast.Name):
                        if "python-submitted-functions-unfolded" not in self.index.breaks:
                            # pool.submit(f, a, b, k=v)：照 f(a, b, k=v) 算一处调用；pool.map(f, 序列)：实参是逐个取的元素，记成 *序列（折不出，照拒算）
                            submitted = node.func.attr == "submit"
                            call = ast.Call(func=node.args[0], args=list(node.args[1:]) if submitted else [ast.Starred(value=item) for item in node.args[1:]],
                                            keywords=list(node.keywords) if submitted else [], lineno=node.lineno, col_offset=node.col_offset)
                            self._call_sites[node.args[0].id].append((call, self.scope_of(node, unit)))
                    elif isinstance(node, ast.Call) and "python-dispatch-calls-ignored" not in self.index.breaks:
                        # 分派表[键](…) / 分派表.get(键)(…)：表（模块级只赋过一次的字典字面量）里当值写的每个函数都算被这一处调了
                        table = node.func.value if isinstance(node.func, ast.Subscript) else (
                            node.func.func.value if isinstance(node.func, ast.Call) and isinstance(node.func.func, ast.Attribute)
                            and node.func.func.attr == "get" else None)
                        found = self.module_dictionary(table.id, None) if isinstance(table, ast.Name) else None
                        if found is not None:
                            for value in found[0].values:
                                if isinstance(value, ast.Name):
                                    self._call_sites[value.id].append((node, self.scope_of(node, unit)))
        return self._call_sites

    def method_call_sites(self, unit):
        """类里的方法 unit：本模块里每一处 <任何东西>.<方法名>(…)（不只 self.，经别的对象调的同样算），连同拆分族里的（importer_method_call_sites）
        → [(调用节点, 调用处的作用域单元)]。"""
        cached = self.__dict__.setdefault("_method_sites", {})
        if unit.node.name not in cached:
            cached[unit.node.name] = [(node, self.scope_of(node, owner)) for owner in self.units for node in walk_nodes(owner.nodes or [])
                                      if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == unit.node.name] \
                + self.importer_method_call_sites(unit.node.name)
        return cached[unit.node.name]

    def fold_parameter(self, unit, name, depth):
        function = unit.node
        if not isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return None
        method = unit.container is not None
        if method and "python-method-parameters-unfolded" in self.index.breaks:
            return None
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        if method and not any(isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in function.decorator_list):
            # 方法的头一个参数（self / cls）是点前面那个对象，调用处的实参从第二个参数起对
            if not positional or positional[0] == name:
                return None
            positional = positional[1:]
        keyword_only = [argument.arg for argument in arguments.kwonlyargs]
        defaults = {}
        for argument, default in zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults):
            defaults[argument] = default
        for argument, default in zip(keyword_only, arguments.kw_defaults):
            if default is not None:
                defaults[argument] = default
        if name not in positional and name not in keyword_only:
            return None
        position = positional.index(name) if name in positional else None
        # 方法：本模块里按方法名调它的每一处（method_call_sites）；套在函数里的：外层函数里调它的那几处（function_call_sites）
        sites = self.method_call_sites(unit) if method else self.function_call_sites(unit)
        if not sites:
            return None
        values = set()
        for call, caller in sites:
            argument = None
            if position is not None and position < len(call.args):
                argument = call.args[position]
                if any(isinstance(item, ast.Starred) for item in call.args[:position + 1]):
                    return None
            elif any(isinstance(item, ast.Starred) for item in call.args):
                return None
            for keyword in call.keywords:
                if keyword.arg is None:
                    return None
                if keyword.arg == name:
                    argument = keyword.value
            # 调用处在 import 这份模块的别的模块里（importer_call_sites）：实参照那份模块折；缺省值照这一份折
            folder, scope = getattr(caller, "module", None) or self, caller
            if argument is None:
                argument = defaults.get(name)
                if folder is not self:
                    folder, scope = self, unit
            if argument is None:
                return None
            folded = folder.fold(argument, scope, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def comprehension_binding(self, node, unit):
        """一个读名字的节点落在绑这个名字的推导式里（[… name … for name in <序列>]）→ 最里层那个推导式里绑它的那一节 generator；
        绑它的是拆包（for name, x in …）交 COMPREHENSION_UNPACKED；不在这样的推导式里 → None。
        弄坏开关 python-comprehension-variables-unfolded 下一律 None（推导式的循环变量折不出）。"""
        if unit is None or "python-comprehension-variables-unfolded" in self.index.breaks:
            return None
        if unit.cache is None:
            unit.cache = {}
        bindings = unit.cache.get("comprehension-bindings")
        if bindings is None:
            bindings = {}
            # ast.walk 先走外层：里层的推导式后记，同一个名字节点落在里层的那一节上
            for comprehension in walk_nodes(unit.nodes or []):
                if not isinstance(comprehension, (ast.ListComp, ast.SetComp, ast.GeneratorExp, ast.DictComp)):
                    continue
                # 头一节的序列在外层作用域里求值，里面的名字不归这个推导式
                outer = {id(inner) for inner in ast.walk(comprehension.generators[0].iter)}
                for generator in comprehension.generators:
                    names = bound_names(generator.target)
                    for inner in ast.walk(comprehension):
                        if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Load) and inner.id in names and id(inner) not in outer:
                            bindings[id(inner)] = generator if isinstance(generator.target, ast.Name) else COMPREHENSION_UNPACKED
            unit.cache["comprehension-bindings"] = bindings
        return bindings.get(id(node))

    def fold_name(self, name, unit, depth):
        if name == "__file__":
            return None if self.inline is not None else {self.absolute}
        key = (self.path, id(unit), name)
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                local = self.local_assignments(unit)
                if name in local:
                    return self.fold_all(local[name], unit, depth)
                arguments = unit.node.args
                if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    return self.fold_parameter(unit, name, depth)
                if enclosing_unit(unit) is not None:
                    # 套着的函数里没赋、不是参数的名字：外层函数的局部变量（闭包变量）或模块级的
                    return self.fold_name(name, enclosing_unit(unit), depth + 1)
            guarded = guard_assignments(name, unit) if "python-guard-names-unfolded" not in self.index.breaks else []
            if guarded:
                return self.fold_all(guarded, unit, depth)
            values, found = set(), False
            for candidate in self.by_name.get(name, ()):
                if candidate.kind == "assign":
                    statement = candidate.node
                    value = statement.value if isinstance(statement, (ast.Assign, ast.AnnAssign)) and len(assignment_names(statement)) == 1 else None
                    if value is None and isinstance(statement, ast.Assign) and len(statement.targets) == 1:
                        # 模块级的 a, b = x, y：取对上 name 的那一个
                        value = dict(paired_elements(statement.targets[0], statement.value) or ()).get(name)
                        if value is not None:
                            folded = self.fold(value, candidate, depth + 1)
                            if folded is None:
                                return None
                            values |= folded
                            found = True
                            continue
                    if isinstance(statement, ast.Assign) and not all(isinstance(target, ast.Name) for target in statement.targets):
                        value = None
                    folded = self.fold(value, candidate, depth + 1) if value is not None else None
                    if folded is None:
                        return None
                    values |= folded
                    found = True
                elif candidate.kind == "import":
                    binding = self.bindings().get(name)
                    if binding is None or binding[0] != "from":
                        return None
                    status, file = self.resolve_module(binding[1], binding[2])
                    other = self.index.module(file) if status == "repo" else None
                    if other is None:
                        return None
                    folded = other.fold_name(binding[4], other.module_body, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                    found = True
                else:
                    return None
            if not found and name in self.body_defines:
                for node in walk_nodes(self.module_body.nodes):
                    if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
                        folded = self.fold(node.value, self.module_body, depth + 1)
                        if folded is None:
                            return None
                        values |= folded
                        found = True
            return values if found else None
        finally:
            self.index.folding.discard(key)

    def fold_all(self, nodes, unit, depth):
        values = set()
        for node in nodes:
            if node is None:
                return None
            folded = self.fold(node, unit, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def fold(self, node, unit, depth=0):
        """表达式 → 它可能取到的字符串值（路径按 posix 规范化）；折叠不出 → None。取值超过 FOLD_LIMIT 种也交 None，另把 index.fold_limit_hits 加一
        （读文件、起子进程那几处照它拒算 FOLD_LIMIT_FORM，不当推不出）。"""
        if depth > 16 or node is None:
            return None
        values = self.fold_inner(node, unit, depth)
        if values is not None and len(values) > FOLD_LIMIT:
            self.index.fold_limit_hits += 1
            return None
        return values

    def module_dictionary(self, name, unit):
        """name 是本模块里只赋过一次（模块级、不在函数里另赋）的字典字面量 → (那个 ast.Dict, 它的赋值单元)；别的 → None。"""
        if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) and name in self.local_assignments(unit):
            return None
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign"]
        if len(candidates) != 1 or not isinstance(candidates[0].node, ast.Assign) or not isinstance(candidates[0].node.value, ast.Dict):
            return None
        if name in self.body_defines:
            return None
        return candidates[0].node.value, candidates[0]

    def single_assignment(self, name, unit):
        """name 只赋过一次时的那个值与它所在的单元：本函数（连同外层函数）里的局部赋值优先，没有再看模块级；赋过几次、拆包赋的 → None。"""
        scope = unit
        while scope is not None and isinstance(scope.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(scope).get(name)
            if assigned is not None:
                return (assigned[0], scope) if len(assigned) == 1 and assigned[0] is not None and not isinstance(assigned[0], ElementsOf) else None
            arguments = scope.node.args
            if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                return None
            scope = enclosing_unit(scope)
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign"]
        if len(candidates) != 1 or not isinstance(candidates[0].node, ast.Assign) or len(candidates[0].node.targets) != 1 \
                or not isinstance(candidates[0].node.targets[0], ast.Name):
            return None
        return candidates[0].node.value, candidates[0]

    def single_sequence(self, node, unit):
        """元组 / 列表字面量，或只赋过一次这样一个字面量的名字 → (字面量, 所在单元)；别的 → None。"""
        if isinstance(node, (ast.Tuple, ast.List)):
            return node, unit
        if isinstance(node, ast.Name):
            found = self.single_assignment(node.id, unit)
            if found is not None and isinstance(found[0], (ast.Tuple, ast.List)):
                return found
        return None

    def loaded_module_of(self, name, unit):
        """name 绑的是 importlib.util.module_from_spec(<spec>) 载入的模块、<spec> 是 spec_from_file_location(…, <路径>) 而路径折出恰好一份仓内 .py
        → 那份模块（PythonModule）；别的 → None。"""
        found = self.single_assignment(name, unit)
        if found is None or not isinstance(found[0], ast.Call) or self.canonical(found[0].func, found[1]) != "importlib.util.module_from_spec" \
                or len(found[0].args) != 1 or not isinstance(found[0].args[0], ast.Name):
            return None
        specification = self.single_assignment(found[0].args[0].id, found[1])
        if specification is None or not isinstance(specification[0], ast.Call) or \
                self.canonical(specification[0].func, specification[1]) != "importlib.util.spec_from_file_location":
            return None
        call, owner = specification
        location = call.args[1] if len(call.args) > 1 else next((keyword.value for keyword in call.keywords if keyword.arg == "location"), None)
        values = self.path_values(location, owner) if location is not None else None
        if not values or len(values) != 1:
            return None
        file = self.file_for_value(next(iter(values)), [posixpath.dirname(self.absolute), self.index.root])
        return self.index.module(file) if file is not None and file.endswith(".py") else None

    def loaded_value_contained(self, call, unit):
        """spec_from_file_location 这一处（call，在 unit 那一层作用域里）载入的模块值出不出这份模块（文件头「按路径载入的模块只连载入方调到的属性」）。
        出不去要同时满足：模块名折得出、没有一个是 "__main__"（载入时不跑 main-guard）；<spec> = 这一处、<名> = importlib.util.module_from_spec(<spec>)
        都赋给名字；<spec> 只用在 <spec>.属性 与 module_from_spec(<spec>)；模块值只用在 <值>.属性、getattr / hasattr(<值>, …)、
        <spec>.loader.exec_module(<值>) 与条件里，赋给别的名字或放进容器（<容器>.append / insert / add(<值>)、<容器>[k] = <值>）的照同样的规矩追下去，
        容器只用在 <容器>[k]、<容器>.append / insert / add / clear、len(<容器>) 与条件里。名字在哪一层：函数自己那一层绑过（不算 global、nonlocal 声明的）
        看那个函数，别的看整份模块，只有内联脚本（不会被别的文件 import）许这一种。交出去的（return、当参数、登进 sys.modules、赋给属性、
        普通 .py 里模块级的名字）一律算出得去 → False，照整个闭包见过的属性名连（ScriptClosure.attribute_seen）。"""
        argument = call.args[0] if call.args else next((keyword.value for keyword in call.keywords if keyword.arg == "name"), None)
        if argument is None:
            return False
        names = {argument.value} if isinstance(argument, ast.Constant) and isinstance(argument.value, str) else self.fold(argument, unit)
        if not names or "__main__" in names:
            return False
        if self.inline is not None:
            tree = self.tree
        else:
            top = unit
            while enclosing_unit(top) is not None:
                top = enclosing_unit(top)
            if top is None or not isinstance(top.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                return False
            tree = top.node
        parents = {}
        for node in ast.walk(tree):
            for child in ast.iter_child_nodes(node):
                parents[id(child)] = node
        if id(call) not in parents:
            return False
        scopes = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)

        def name_root(name, node):
            current = parents.get(id(node))
            while current is not None:
                if isinstance(current, scopes):
                    local, _declared = function_local_names(current)
                    if name in local:
                        return current
                current = parents.get(id(current))
            return tree if self.inline is not None else None

        work, followed = [], set()

        def follow(kind, name, node):
            root = name_root(name, node)
            if root is None:
                return False
            if (kind, name, id(root)) not in followed:
                followed.add((kind, name, id(root)))
                work.append((kind, name, root))
            return True

        def condition(node, parent):
            return (isinstance(parent, (ast.If, ast.While, ast.IfExp, ast.Assert)) and parent.test is node) or \
                (isinstance(parent, ast.UnaryOp) and isinstance(parent.op, ast.Not)) or \
                (isinstance(parent, ast.Compare) and all(isinstance(operator, (ast.Is, ast.IsNot)) for operator in parent.ops))

        def value_use(node):
            """node 求出来可能是那个模块值：这一处用法出不出得去。"""
            parent = parents.get(id(node))
            if isinstance(parent, ast.Attribute) and parent.value is node:
                return parent.attr != "__dict__"
            if isinstance(parent, ast.Call) and node in parent.args:
                function = parent.func
                if isinstance(function, ast.Name) and function.id in ("getattr", "hasattr") and parent.args[0] is node:
                    return True
                if isinstance(function, ast.Attribute) and function.attr == "exec_module" and parent.args == [node]:
                    return True
                if isinstance(function, ast.Attribute) and function.attr in ("append", "insert", "add") and isinstance(function.value, ast.Name):
                    return follow("container", function.value.id, parent)
                return False
            if isinstance(parent, (ast.Assign, ast.AnnAssign)) and parent.value is node:
                for target in (parent.targets if isinstance(parent, ast.Assign) else [parent.target]):
                    if isinstance(target, ast.Name):
                        if not follow("alias", target.id, parent):
                            return False
                    elif isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name):
                        if not follow("container", target.value.id, parent):
                            return False
                    else:
                        return False
                return True
            return condition(node, parent)

        def name_use(kind, node):
            parent = parents.get(id(node))
            if kind == "alias":
                return value_use(node)
            if kind == "spec":
                if isinstance(parent, ast.Attribute) and parent.value is node:
                    return True
                return isinstance(parent, ast.Call) and parent.args == [node] and self.canonical(parent.func, unit) == "importlib.util.module_from_spec" \
                    and created(parent)
            if isinstance(parent, ast.Subscript) and parent.value is node:
                return value_use(parent) if isinstance(parent.ctx, ast.Load) else True
            if isinstance(parent, ast.Attribute) and parent.value is node:
                return parent.attr in ("append", "insert", "add", "clear") and isinstance(parents.get(id(parent)), ast.Call) \
                    and parents[id(parent)].func is parent
            if isinstance(parent, ast.Call) and isinstance(parent.func, ast.Name) and parent.func.id == "len" and parent.args == [node]:
                return True
            return condition(node, parent)

        def created(module_call):
            """importlib.util.module_from_spec(<spec>)：载入的模块值打这一处来，要赋给名字。"""
            parent = parents.get(id(module_call))
            return isinstance(parent, ast.Assign) and parent.value is module_call and len(parent.targets) == 1 \
                and isinstance(parent.targets[0], ast.Name) and follow("alias", parent.targets[0].id, parent)

        parent = parents[id(call)]
        if isinstance(parent, ast.Assign) and parent.value is call and len(parent.targets) == 1 and isinstance(parent.targets[0], ast.Name):
            if not follow("spec", parent.targets[0].id, parent):
                return False
        elif not (isinstance(parent, ast.Call) and parent.args == [call] and self.canonical(parent.func, unit) == "importlib.util.module_from_spec"
                  and created(parent)):
            return False
        while work:
            kind, name, root = work.pop()
            for node in ast.walk(root):
                if isinstance(node, ast.Name) and node.id == name and isinstance(node.ctx, ast.Load) and not name_use(kind, node):
                    return False
        return True

    def fold_elements(self, sequence, unit, depth):
        """元组 / 列表字面量，或绑着它的名字（本函数里只赋过一次的、模块级的）→ 各元素折出来的并；有一个折不出就整个折不出。"""
        if isinstance(sequence, ast.Name):
            assigned = self.local_assignments(unit).get(sequence.id, []) if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else []
            module, owner = self, unit
            if not assigned:
                candidates = [candidate for candidate in self.by_name.get(sequence.id, ()) if candidate.kind == "assign"]
                assigned = [candidate.node.value for candidate in candidates if isinstance(candidate.node, ast.Assign)]
                owner = candidates[0] if len(candidates) == 1 else None
            if len(assigned) != 1 or not isinstance(assigned[0], (ast.Tuple, ast.List)):
                return None
            return module.fold_elements(assigned[0], owner, depth + 1)
        if isinstance(sequence, ast.Attribute) and isinstance(sequence.value, ast.Name) and depth <= 16 \
                and "python-module-sequence-elements-unfolded" not in self.index.breaks:
            # import 进来的仓内模块的模块级元组 / 列表（for name in module.NAMES、[… for name in module.NAMES]）：取那份模块里 NAMES 的元素，
            # 同 fold_inner 认 module.NAME 那一条；弄坏开关 python-module-sequence-elements-unfolded 下不认
            other = self.module_of_binding(self.binding(sequence.value.id, unit))
            if other is None or other is self:
                return None
            return other.fold_elements(ast.Name(id=sequence.attr, ctx=ast.Load()), other.module_body, depth + 1)
        if isinstance(sequence, ast.Call) and depth <= 16 and "python-element-calls-unfolded" not in self.index.breaks:
            function = sequence.func
            if isinstance(function, ast.Attribute) and function.attr == "get" and isinstance(function.value, ast.Dict) and 1 <= len(sequence.args) <= 2 \
                    and not sequence.keywords:
                # {…}.get(键, 缺省)：键折不折得出都取遍字典里每个值的元素，连同缺省（没写缺省时取到 None、迭代就抛，不添元素）
                values = set()
                for part in list(function.value.values) + list(sequence.args[1:]):
                    elements = self.fold_elements(part, unit, depth + 1)
                    if elements is None:
                        return None
                    values |= elements
                return values
            if isinstance(function, ast.Name) and not sequence.keywords:
                # 调仓内模块级函数：它各处 return 的序列的元素
                return self.elements_of_call_returns(function.id, unit, depth)
            return None
        if isinstance(sequence, ast.Subscript) and isinstance(sequence.value, ast.Name) and "python-getattr-names-unfolded" not in self.index.breaks:
            # 模块级字典字面量[键]：键折不折得出都取遍每个值的元素
            found = self.module_dictionary(sequence.value.id, unit)
            if found is None:
                return None
            table, owner = found
            values = set()
            for part in table.values:
                elements = self.fold_elements(part, owner, depth + 1)
                if elements is None:
                    return None
                values |= elements
            return values
        if not isinstance(sequence, (ast.Tuple, ast.List)):
            return None
        values = set()
        for element in sequence.elts:
            folded = self.fold(element, unit, depth + 1)
            if folded is None:
                return None
            values |= folded
        return values

    def fold_format_value(self, node, unit, depth):
        """f 串里 {…} 那一段的取值：照 fold；另认 sys.executable（折成 python3，同 command_links 的命令词）与整数常量（字面量或只赋过一次它的名字，折成十进制）。"""
        folded = self.fold(node, unit, depth + 1)
        if folded is not None or "python-text-pieces-unfolded" in self.index.breaks:
            return folded
        if self.canonical(node, unit) == "sys.executable":
            return {"python3"}
        if isinstance(node, ast.Name):
            found = self.single_assignment(node.id, unit)
            node = found[0] if found is not None else node
        return {str(node.value)} if isinstance(node, ast.Constant) and type(node.value) is int else None

    def fold_inner(self, node, unit, depth):
        if isinstance(node, ElementsOf):
            return self.fold_elements(node.sequence, unit, depth)
        if isinstance(node, ast.Constant):
            return {node.value} if isinstance(node.value, str) else None
        if isinstance(node, ast.JoinedStr):
            results = {""}
            for part in node.values:
                if isinstance(part, ast.Constant):
                    results = {result + str(part.value) for result in results}
                elif isinstance(part, ast.FormattedValue) and part.format_spec is None and part.conversion in (-1, 115):
                    folded = self.fold_format_value(part.value, unit, depth)
                    if folded is None:
                        return None
                    results = {result + value for result in results for value in folded}
                else:
                    return None
            return results
        known = self.inline_environment(node, unit)
        if known is not None:
            return set(known)
        if isinstance(node, ast.BoolOp) and isinstance(node.op, ast.Or) and "python-environment-defaults-unfolded" not in self.index.breaks:
            # os.environ.get("X") or <缺省>：环境变量顶替的那一边照 shell 的 ${X:-缺省} 办，按缺省折；别的每一边都要折得出
            values = set()
            for operand in node.values:
                if self.environment_read(operand, unit):
                    continue
                folded = self.fold(operand, unit, depth + 1)
                if folded is None:
                    return None
                values |= folded
            return values or None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            left, right = self.fold(node.left, unit, depth + 1), self.fold(node.right, unit, depth + 1)
            if left is None or right is None:
                return None
            if isinstance(node.op, ast.Add):
                return {a + b for a in left for b in right}
            return {posixpath.normpath(posixpath.join(a, b)) for a in left for b in right}
        if isinstance(node, ast.Name):
            generator = self.comprehension_binding(node, unit)
            if generator is COMPREHENSION_UNPACKED:
                return None
            if generator is not None:
                # 推导式的循环变量：取遍它那一节迭代的元素
                return self.fold_elements(generator.iter, unit, depth + 1)
            return self.fold_name(node.id, unit, depth)
        if isinstance(node, ast.Attribute) and node.attr == "stdout" and isinstance(node.value, ast.Call):
            return self.git_output(node.value, unit)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and "python-module-constants-unfolded" not in self.index.breaks:
            # import 进来的仓内模块的模块级常量：module.NAME 按那份模块里 NAME 的赋值折
            other = self.module_of_binding(self.binding(node.value.id, unit))
            if other is not None and other is not self:
                return other.fold_name(node.attr, other.module_body, depth + 1)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and "python-loaded-module-constants-unfolded" not in self.index.breaks:
            # spec_from_file_location 载入的仓内模块的模块级常量：module.NAME 按那份模块里 NAME 的赋值折
            loaded = self.loaded_module_of(node.value.id, unit)
            if loaded is not None and loaded is not self:
                return loaded.fold_name(node.attr, loaded.module_body, depth + 1)
        if isinstance(node, ast.Attribute):
            if node.attr in ("parent", "name", "stem"):
                folded = self.fold(node.value, unit, depth + 1)
                if folded is None:
                    return None
                if node.attr == "parent":
                    return {posixpath.dirname(value) for value in folded}
                if node.attr == "name":
                    return {posixpath.basename(value) for value in folded}
                return {posixpath.splitext(posixpath.basename(value))[0] for value in folded}
            return None
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and node.slice.value >= 1:
            # 内联脚本的 sys.argv[k]：起它的那条命令里脚本之后的第 k 个词
            position = node.slice.value - 1
            values = self.inline.argv[position] if position < len(self.inline.argv) else None
            return set(values) if values is not None and values is not POSITIONAL else None
        if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name) and "python-path-tables-unfolded" not in self.index.breaks:
            # 路径表（模块级只赋过一次的字典字面量）[键]：键折得出取那几项，折不出取遍全部值
            found = self.module_dictionary(node.value.id, unit)
            if found is not None:
                table, owner = found
                keys = self.fold(node.slice, unit, depth + 1)
                if any(key is None for key in table.keys):
                    return None
                chosen = [value for key, value in zip(table.keys, table.values)
                          if keys is None or (isinstance(key, ast.Constant) and key.value in keys)]
                values = set()
                for value in chosen:
                    folded = self.fold(value, owner, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                return values or None
        if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and \
                "python-sequence-indexes-unfolded" not in self.index.breaks:
            # 只赋过一次的元组 / 列表字面量[整数]（模块级常量、本函数里的局部变量）：取那一项
            found = self.single_sequence(node.value, unit)
            if found is not None:
                literal, owner = found
                index = node.slice.value
                if -len(literal.elts) <= index < len(literal.elts) and not any(isinstance(element, ast.Starred) for element in literal.elts):
                    return self.fold(literal.elts[index], owner, depth + 1)
        if isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute) and node.value.attr == "parents":
            index = node.slice
            if isinstance(index, ast.Constant) and isinstance(index.value, int):
                folded = self.fold(node.value.value, unit, depth + 1)
                if folded is None:
                    return None
                out = set()
                for value in folded:
                    for _step in range(index.value + 1):
                        value = posixpath.dirname(value)
                    out.add(value)
                return out
            return None
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical == "subprocess.check_output":
                return self.git_output(node, unit)
            if canonical == "shutil.which" and len(node.args) == 1 and isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
                # PATH 上找到的那一个程序：名字就是给的这个（装不装仓里的代码按名字认）
                return {node.args[0].value}
            if isinstance(node.func, ast.Name) and (node.args or node.keywords) and "python-call-returns-unfolded" not in self.index.breaks:
                # 带参数调仓内模块级函数：按它各处 return 的值折，参数按它的每一处调用给的实参折（不分是哪一处调用）
                returned = self.fold_call_return(node.func.id, unit, depth)
                if returned is not None:
                    return returned
            if isinstance(node.func, ast.Attribute) and node.func.attr in ("strip", "rstrip", "decode") and not node.keywords and (
                    not node.args or (node.func.attr == "decode" and len(node.args) == 1 and isinstance(node.args[0], ast.Constant))):
                return self.fold(node.func.value, unit, depth + 1)
            if isinstance(node.func, ast.Attribute) and node.func.attr == "join" and len(node.args) == 1 and not node.keywords \
                    and isinstance(node.func.value, ast.Constant) and isinstance(node.func.value.value, str) \
                    and "python-text-pieces-unfolded" not in self.index.breaks:
                # "<分隔>".join(<字面元组或列表>)：元素排好序拼成一个串（这个串拿去切内联脚本认边，不进指纹）
                elements = self.fold_elements(node.args[0], unit, depth + 1)
                return {node.func.value.value.join(sorted(elements))} if elements is not None else None
            if node.keywords:
                return None
            if canonical in ("os.path.join", "posixpath.join") or canonical in PATH_CONSTRUCTORS:
                results = {""}
                for argument in node.args:
                    folded = self.fold(argument, unit, depth + 1)
                    if folded is None:
                        return None
                    results = {posixpath.join(result, value) if result else value for result in results for value in folded}
                return {posixpath.normpath(value) for value in results} if node.args else None
            if canonical in PATH_TRANSFORMS and len(node.args) == 1:
                folded = self.fold(node.args[0], unit, depth + 1)
                if folded is None:
                    return None
                if canonical.endswith("dirname"):
                    return {posixpath.dirname(value) for value in folded}
                if canonical.endswith("basename"):
                    return {posixpath.basename(value) for value in folded}
                if canonical in ("str", "os.fspath"):
                    return folded
                return {posixpath.normpath(value) for value in folded}
            if isinstance(node.func, ast.Attribute):
                method = node.func.attr
                if method in ("resolve", "absolute", "as_posix") and not node.args:
                    folded = self.fold(node.func.value, unit, depth + 1)
                    return {posixpath.normpath(value) for value in folded} if folded is not None else None
                if method == "joinpath":
                    folded = self.fold(node.func.value, unit, depth + 1)
                    for argument in node.args:
                        part = self.fold(argument, unit, depth + 1)
                        if folded is None or part is None:
                            return None
                        folded = {posixpath.join(a, b) for a in folded for b in part}
                    return {posixpath.normpath(value) for value in folded} if folded is not None else None
            if isinstance(node.func, ast.Name) and not node.args:
                return self.fold_call_return(node.func.id, unit, depth)
            return None
        return None

    def elements_of_call_returns(self, name, unit, depth):
        """调仓内模块级函数 name（本模块的，或 from 仓内模块 import 进来的）：它各处 return 的序列（fold_elements 认的写法）的元素的并；
        return / return None 不添元素；有一处折不出 → None。"""
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return None
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return None
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return None
        callee = candidates[0]
        key = (module.path, id(callee), "<elements>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            values = set()
            for statement in walk_function_statements(callee.node):
                if isinstance(statement, ast.Return):
                    if statement.value is None or (isinstance(statement.value, ast.Constant) and statement.value.value is None):
                        continue
                    elements = module.fold_elements(statement.value, callee, depth + 1)
                    if elements is None:
                        return None
                    values |= elements
            return values
        finally:
            self.index.folding.discard(key)

    def fold_call_return(self, name, unit, depth):
        """不带参数调一个仓内的模块级函数（本模块的，或 from 仓内模块 import 进来的）→ 它各处 return 的值的并；
        return / return None 那几处是「没找到」的出口，不算一种取值；有一处 return 折不出就整个折不出。"""
        if "python-call-returns-unfolded" in self.index.breaks:
            return None
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return None
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return None
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return None
        callee = candidates[0]
        key = (module.path, id(callee), "<return>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            values, stack = set(), list(callee.node.body)
            while stack:
                statement = stack.pop()
                if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)):
                    continue
                if isinstance(statement, ast.Return):
                    if statement.value is None or (isinstance(statement.value, ast.Constant) and statement.value.value is None):
                        continue
                    folded = module.fold(statement.value, callee, depth + 1)
                    if folded is None:
                        return None
                    values |= folded
                    continue
                stack.extend(ast.iter_child_nodes(statement))
            return values or None
        finally:
            self.index.folding.discard(key)

    def inline_environment(self, node, unit):
        """内联脚本里读起它的那条命令设的环境变量（os.environ["X"]、os.environ.get("X")、os.getenv("X")）→ 那个值（frozenset）；别的 → None。
        拆成 shell 入口加同目录模块的，拆出的模块读入口起内联 python 时设的那几个（inherited_environment）。"""
        if not self.environment_read(node, unit):
            return None
        environment = self.inline.environment if self.inline is not None else self.inherited_environment()
        if not environment:
            return None
        key = node.slice if isinstance(node, ast.Subscript) else (node.args[0] if node.args else None)
        if not (isinstance(key, ast.Constant) and isinstance(key.value, str)):
            return None
        found = None
        for name, values in environment:
            if name == key.value:
                found = values
        return found

    def inherited_environment(self):
        """拆成 shell 入口加同目录模块的（入口 <名>.sh 起内联 python，内联 python import 拆出的 <名，「-」换「_」>_<部分>.py）：拆出的模块与内联 python 在同一个进程里，
        入口起那几份内联 python 时设的环境变量（命令前设的、同一个单元里 export 过的：InlineScript 的 environment）它同样读得到 → ((名字, 取值 frozenset)…)。
        只认正文 import 了这一族里某一份模块的内联 python；同名的几份取值合起来，有一份折不出的那个名字不交（照旧折不出）。入口取 split_family_root，
        不是 .sh、不在拆分族里、内联脚本、解不开的与没有闭包给的 entry_inline_scripts（索引不在 ScriptClosure 里）交空。弄坏开关 python-split-part-environment-unread 下交空。"""
        cached = self.__dict__.get("_inherited_environment")
        if cached is not None:
            return cached
        self._inherited_environment = ()   # 先占上：取入口的内联脚本时又问到这一份不绕圈
        provider = self.index.entry_inline_scripts
        own = self.index.relative(self.absolute) if self.inline is None else None
        if own is None or self.tree is None or provider is None or "python-split-part-environment-unread" in self.index.breaks:
            return self._inherited_environment
        root = self.split_family_root(own)
        if root == own or not root.endswith(".sh"):
            return self._inherited_environment
        listing = self.index.__dict__["_split_family_listings"][posixpath.dirname(own)]
        members = {posixpath.basename(path)[:-len(".py")] for path in listing | {own} if path.endswith(".py") and self.split_family_root(path) == root}
        merged, unfolded = {}, set()
        for script in provider(root):
            if script.language != "python" or not script.environment:
                continue
            if not any(set(name for pair in INLINE_IMPORTED_NAME.findall(text) for name in pair if name) & members for text in script.texts):
                continue
            final = {}
            for name, values in script.environment:
                final[name] = values   # 同名的后设的盖过先设的（命令前设的排在 export 过的后面，与内联脚本自己读的同一条）
            for name, values in final.items():
                if isinstance(values, frozenset):
                    merged[name] = merged.get(name, frozenset()) | values
                else:
                    unfolded.add(name)
        self._inherited_environment = tuple(sorted((name, values) for name, values in merged.items() if name not in unfolded))
        return self._inherited_environment

    def environment_read(self, node, unit):
        """os.environ.get(…) / os.getenv(…) / os.environ[…]：读环境变量的那一个表达式。"""
        if isinstance(node, ast.Call):
            return self.canonical(node.func, unit) in ("os.environ.get", "os.getenv")
        if isinstance(node, ast.Subscript):
            return self.canonical(node.value, unit) == "os.environ"
        return False


def python_units(root, under=None):
    index = PythonIndex(root)
    modules = [index.module(path) for path in files_under(root, (".py",), under)]
    return index, [module for module in modules if module is not None]
