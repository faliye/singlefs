"""PythonModule 的一部分方法（混入类 PythonModuleLaunches）：调用的事实（call_facts）、子进程命令的折叠与候选取值、
起脚本与内联脚本的边（command_links）、程序词的登记核对。python_module.py 里的 PythonModule 继承它。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import os
import posixpath
import re
import shlex
from lib_function_units_base import (
    COMMAND_FORM, FOLD_LIMIT, FOLD_LIMIT_FORM, INLINE_FORM, InlineScript, OPAQUE, POSITIONAL, PREFLIGHT_MODULE, PYTHON_INTERPRETERS, SHELL_INTERPRETERS,
    TEST_RUNNER_MODULES, Unit, ValueElement, argv_command_offsets, command_positions, descriptor_of, interpreter_mode, sha256_bytes, sha256_text,
)
from lib_function_units_registry import (
    REGISTRY_COMMAND_TEXTS, built_tool_launch_source, launched_programs_registered, program_word_registered, registered_function_name,
)
from lib_function_units_python_facts import (
    CALLER_INPUT_ORIGINS, COPY_CALLS, EXEC_CALLS, REGISTRY_LISTED_PROGRAMS_TABLE, SHELL_STRING_CALLS, SOURCE_READERS, SUBPROCESS_CALLS,
    TOOLCHAIN_QUERY_WORDS, bound_names, dotted_name, enclosing_unit, launch_arguments, paired_elements, shell_launch_choices, unique_alternatives,
    walk_function_statements, walk_nodes, walk_targets,
)
from lib_function_units_shell_text import tree_copy_file
from lib_function_units_shell_index import repository_module


class PythonModuleLaunches:
    def call_facts(self, node, unit, facts, breaks):
        function = node.func
        canonical = self.canonical(function, unit)
        if isinstance(function, ast.Name) and function.id == "getattr" and len(node.args) >= 2 and self.binding("getattr", unit) is None:
            attribute = node.args[1]
            if isinstance(attribute, ast.Constant) and isinstance(attribute.value, str):
                facts.attributes.append((dotted_name(node.args[0]), attribute.value))
                if attribute.value == "__doc__" and "python-docstring-call-readers-ignored" not in breaks:
                    # getattr(<对象>, "__doc__") 同 <对象>.__doc__（文件头「文档串的读法」）
                    facts.doc_reads.append(dotted_name(node.args[0]) or "*")
            elif "python-getattr-allowed" not in breaks and not self.outside_module(node.args[0], unit):
                names = self.fold(attribute, unit) if "python-getattr-names-unfolded" not in breaks else None
                if names is None:
                    self.refuse(facts, "getattr-variable", node)
                else:
                    facts.attributes += [(dotted_name(node.args[0]), name) for name in sorted(names)]
        if isinstance(function, ast.Name) and function.id == "globals" and self.binding("globals", unit) is None:
            self.refuse(facts, "globals-lookup", node)
        if isinstance(function, ast.Name) and function.id == "exec" and self.binding("exec", unit) is None:
            self.exec_facts(node, unit, facts, breaks)
        if canonical == "inspect.getdoc" and node.args and "python-docstring-call-readers-ignored" not in breaks:
            # inspect.getdoc(<对象>) 读 <对象> 的文档串（文件头「文档串的读法」）
            facts.doc_reads.append(dotted_name(node.args[0]) or "*")
        if canonical == "importlib.util.spec_from_file_location" and "python-spec-paths-ignored" not in breaks:
            location = node.args[1] if len(node.args) > 1 else next((keyword.value for keyword in node.keywords if keyword.arg == "location"), None)
            values = self.path_values(location, unit) if location is not None else None
            if values is None and location is not None:
                values = self.registered_spec_path(location)
            if values is None:
                self.refuse(facts, "spec-path-unfolded", node)
            else:
                contained = None
                for value in sorted(values):
                    file = self.file_for_value(value, [posixpath.dirname(self.absolute), self.index.root])
                    if file is None:
                        self.refuse(facts, "spec-path-missing", node)
                    else:
                        facts.loads.append(file)
                        if contained is None:
                            contained = self.loaded_value_contained(node, unit)
                        if contained:
                            facts.contained_loads.append(file)
        if canonical in ("importlib.import_module", "__import__") and node.args:
            argument = node.args[0]
            names = {argument.value} if isinstance(argument, ast.Constant) and isinstance(argument.value, str) else (
                self.fold(argument, unit) if "python-import-names-unfolded" not in breaks else None)
            if names is not None:
                for name in sorted(names):
                    status, file = self.resolve_module(name, 0)
                    if status == "repo":
                        facts.loads += self.all_resolutions(name, 0) or [file]
                    elif status == "unresolved":
                        self.refuse(facts, "import-unresolved", node)
            else:
                self.refuse(facts, "import-module-variable", node)
        if canonical == "runpy.run_path" and node.args:
            values = self.path_values(node.args[0], unit)
            if values is None:
                self.refuse(facts, "subprocess-path-unfolded", node)
            else:
                for value in sorted(values):
                    file = self.file_for_value(value, [self.index.root, posixpath.dirname(self.absolute)])
                    if file is None:
                        self.refuse(facts, "subprocess-path-missing", node)
                    else:
                        facts.program_links.append(file)
        if canonical in ("sys.path.insert", "sys.path.append"):
            argument = node.args[1] if canonical.endswith("insert") and len(node.args) >= 2 else (node.args[0] if node.args else None)
            if argument is not None and self.path_values(argument, unit, directory=True) is None:
                self.refuse(facts, "sys-path-unfolded", node)
        if canonical == "sys.path.extend" and node.args:
            elements = list(node.args[0].elts) if isinstance(node.args[0], (ast.List, ast.Tuple)) else [None]
            if any(element is None or self.path_values(element, unit, directory=True) is None for element in elements):
                self.refuse(facts, "sys-path-unfolded", node)
        if canonical in SUBPROCESS_CALLS and node.args:
            shell = any(keyword.arg == "shell" and isinstance(keyword.value, ast.Constant) and keyword.value.value for keyword in node.keywords)
            self.subprocess_facts(node, node.args[0], unit, facts, shell)
        if canonical in EXEC_CALLS and len(node.args) >= 2:
            self.subprocess_facts(node, node.args[1], unit, facts, False)
        if canonical in SHELL_STRING_CALLS and node.args:
            self.subprocess_facts(node, node.args[0], unit, facts, True)
        if canonical in SOURCE_READERS or (isinstance(function, ast.Attribute) and function.attr in ("read_text", "read_bytes")):
            if canonical in ("inspect.getsource", "inspect.getsourcelines") or any(
                    isinstance(inner, ast.Name) and inner.id == "__file__" for inner in ast.walk(node)):
                facts.source_read = True
        if "python-data-reads-ignored" not in breaks:
            # 运行期读的数据文件（文件头「运行期读的数据文件」）：取法在闭包里落成数据单元、跳过或拒算（ScriptClosure.python_data_reads）
            target = self.read_target(node, unit)
            if target is not None:
                # 拷文件（COPY_CALLS）的源另记一笔：折不出的那一截不拒算（文件头「运行期读的数据文件」二）
                copied = self.canonical(node.func, unit) in COPY_CALLS and "python-root-joins-refused" not in breaks
                facts.data_reads.append((self.data_read_origins(target, unit), node.lineno, copied))

    def command_elements(self, command, unit, shell):
        if isinstance(command, (ast.List, ast.Tuple)):
            return list(command.elts)
        if isinstance(command, ast.Name):
            candidates = []
            if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                candidates = self.local_assignments(unit).get(command.id, [])
            if not candidates:
                candidates = [candidate.node.value for candidate in self.by_name.get(command.id, ())
                              if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
            if len(candidates) == 1 and isinstance(candidates[0], (ast.List, ast.Tuple)):
                return list(candidates[0].elts)
            return None
        if isinstance(command, ast.Constant) and isinstance(command.value, str) and shell:
            try:
                return [ast.Constant(value=word) for word in shlex.split(command.value)]
            except ValueError:
                return None
        return None

    def subprocess_facts(self, node, command, unit, facts, shell):
        """起子进程的一次调用：命令是交给 sh -c 的串（shell=True、os.system）的当内联 bash 认；是列表的折出它的几种取法（字面列表、
        拼起来的、按调用处或返回值折的），逐种按命令认；认不出 → 拒算 subprocess-command-unfolded。"""
        breaks = self.index.breaks
        keywords = {keyword.arg: keyword.value for keyword in node.keywords if keyword.arg}
        where = self.refusal("", node.lineno)
        if shell:
            if "python-commands-literal-only" in breaks:
                elements = self.command_elements(command, unit, True)
                if elements is not None:
                    self.command_links([(element, self, unit) for element in elements], unit, node, facts, keywords)
                return
            hits = self.index.fold_limit_hits
            texts = self.fold(command, unit)
            if texts is None:
                if "inline-scripts-ignored" not in breaks:
                    self.refuse_launch(facts, INLINE_FORM, node, (command, self, unit), unit, self.index.fold_limit_hits > hits)
                facts.handled.append(("shell-string", node.lineno))
                return
            facts.inline_scripts.append(InlineScript("shell", tuple(sorted(texts)), (None,), self.working_directories(keywords, unit), where.file, where.line,
                                                     where.snippet))
            facts.handled.append(("shell-string", node.lineno))
            return
        if self.inline is not None and isinstance(command, ast.Subscript) and self.canonical(command.value, unit) == "sys.argv" and \
                isinstance(command.slice, ast.Slice) and "inline-argv-commands-ignored" not in breaks:
            # 内联脚本起 sys.argv[k:]：那条命令在起这个内联脚本的那一边照命令认过（ShellIndex.resolve_inline、command_links 的 inline）
            facts.handled.append(("argv-command", node.lineno))
            return
        hits = self.index.fold_limit_hits
        if "python-commands-literal-only" in breaks:
            elements = self.command_elements(command, unit, False)
            alternatives = None if elements is None else [[(element, self, unit) for element in elements]]
            if alternatives is None:
                return
        else:
            alternatives = self.command_alternatives(command, unit)
        if alternatives is None:
            self.refuse_launch(facts, COMMAND_FORM, node, (command, self, unit), unit, self.index.fold_limit_hits > hits)
            facts.handled.append(("command-unfolded", node.lineno))
            return
        for elements in alternatives:
            self.command_links(elements, unit, node, facts, keywords)

    def exec_facts(self, node, unit, facts, breaks):
        """内建 exec 这一处：exec 进模块对象的（module_object_exec，文件头「exec 进模块对象」）源码折得出照载入那一份连——读出的文件进 loads
        （模块对象出不了这份模块的另进 contained_loads），内联正文进 module_object_texts——折不出拒算 compiled-part-exec；
        别的照「按部分文件 exec」认（compiled_part_exec）。弄坏开关 python-module-object-exec-refused 下不认模块对象这一种，
        python-module-object-exec-unfolded-allowed 下源码折不出的不拒算（不连也不拒算）。"""
        loaded = self.module_object_exec(node, unit) if "python-module-object-exec-refused" not in breaks else None
        if loaded is None:
            if "python-compiled-part-exec-allowed" not in breaks and self.compiled_part_exec(node, unit):
                # 按部分文件 exec（文件头「按部分文件 exec」）：exec 进这个命名空间的是哪份文件的哪些名字折不出，不连也不静默跳过
                self.refuse(facts, "compiled-part-exec", node)
            return
        kind, sources = loaded
        if kind is None:
            if "python-module-object-exec-unfolded-allowed" not in breaks:
                self.refuse(facts, "compiled-part-exec", node)
            return
        contained = self.module_object_contained(node, unit)
        if kind == "files":
            for file in sources:
                facts.loads.append(file)
                if contained:
                    facts.contained_loads.append(file)
        else:
            facts.module_object_texts.append((sources, contained))
            if "python-module-object-loader-call-sites-ignored" not in breaks:
                # 载入的正文里模块级函数的参数照这一份里 <模块对象>.<函数>(…) 那几处折（module_object_loader_call_sites）
                root = self.module_object_root(node.args[1].value.id, unit)
                self.index.module_object_loaders.setdefault(sources, {})[(id(self), id(root[0]))] = (self, root[0])

    def module_object_code(self, node, unit):
        """exec(compile(<源码>, <文件名>, "exec"), <名>.__dict__) 这一处（exec、compile 是内建的，mode 是字面 "exec"，只两个位置实参），
        <名> 是模块对象（module_object_root）→ 那个 compile 调用；别的 → None。"""
        if unit is None or len(node.args) != 2 or node.keywords:
            return None
        code, namespace = node.args
        if not (isinstance(code, ast.Call) and isinstance(code.func, ast.Name) and code.func.id == "compile" and self.binding("compile", unit) is None):
            return None
        keywords = {keyword.arg: keyword.value for keyword in code.keywords if keyword.arg is not None}
        mode = code.args[2] if len(code.args) > 2 else keywords.get("mode")
        if not (isinstance(mode, ast.Constant) and mode.value == "exec"):
            return None
        if not (isinstance(namespace, ast.Attribute) and namespace.attr == "__dict__" and isinstance(namespace.value, ast.Name)):
            return None
        return code if self.module_object_root(namespace.value.id, unit) is not None else None

    def module_object_root(self, name, unit, followed=frozenset()):
        """name 绑的是 types.ModuleType(<字面串，不是 "__main__">) 建的模块对象：只赋过一次（single_assignment），赋的是这一句，
        或是只赋过一次的 <名> = <别的名字>、照那个名字接着追（追回已追过的名字算不是）→ (那个 types.ModuleType 调用, 它的赋值单元)；别的 → None。
        模块名是 "__main__" 的 exec 进去时连 main-guard 也跑，不算这一种。"""
        found = self.single_assignment(name, unit) if name not in followed else None
        if found is None:
            return None
        value, owner = found
        if isinstance(value, ast.Name):
            return self.module_object_root(value.id, owner, followed | {name})
        if isinstance(value, ast.Call) and len(value.args) == 1 and not value.keywords and self.canonical(value.func, owner) == "types.ModuleType" \
                and isinstance(value.args[0], ast.Constant) and isinstance(value.args[0].value, str) and value.args[0].value != "__main__":
            return value, owner
        return None

    def module_object_exec(self, node, unit):
        """exec 进模块对象这一处（module_object_code；文件头「exec 进模块对象」）的源码：
        open(<路径>…).read() 的，路径折得出（path_values）、每一份都是根里的文件 → ("files", [相对根的文件…])；
        别的照 fold 折得出那几段正文 → ("texts", InlineScript)：正文在这个进程里跑，sys.argv、环境变量与工作目录照这份模块的（内联脚本照起它的那条命令），
        正文里的拒算落在原文里那段正文的那一行（起这个内联脚本的那条命令交进来的正文出自一份 here-doc 的，InlineScript 的 body_lines、body_files），
        别的落在这个 exec 那一行；折不出 → (None, None)；不是 exec 进模块对象 → None。"""
        code = self.module_object_code(node, unit)
        if code is None:
            return None
        source = code.args[0] if code.args else next((keyword.value for keyword in code.keywords if keyword.arg == "source"), None)
        if source is None:
            return None, None
        opened = source.func.value if isinstance(source, ast.Call) and not source.args and not source.keywords \
            and isinstance(source.func, ast.Attribute) and source.func.attr == "read" and isinstance(source.func.value, ast.Call) else None
        if opened is not None and opened.args and self.canonical(opened.func, unit) == "open":
            values = self.path_values(opened.args[0], unit)
            files = [self.file_for_value(value, [posixpath.dirname(self.absolute), self.index.root]) for value in sorted(values or ())]
            if not files or None in files or any(self.index.module(file) is None for file in files):
                return None, None
            return "files", sorted(set(files))
        texts = self.fold(source, unit)
        if not texts:
            return None, None
        where = self.refusal("compiled-part-exec", node.lineno)
        body_lines = body_files = listings = ()
        if self.inline is not None:
            argv, bases, environment, listings = self.inline.argv, self.inline.bases, self.inline.environment, self.inline.listings
            # 正文是起这个内联脚本的那条命令交进来的、出自原文里一份 here-doc 的（ShellIndex.resolve_inline 记在 body_lines 里）：拒算落在原文里正文那一行
            body_lines = tuple(pair for pair in self.inline.body_lines if pair[0] in texts)
            body_files = tuple(pair for pair in self.inline.body_files if pair[0] in texts)
        else:
            argv, bases, environment = (), (posixpath.dirname(self.absolute), self.index.root), ()
        return "texts", InlineScript("python", tuple(sorted(texts)), argv, bases, where.file, where.line, where.snippet, environment,
                                     body_lines, body_files, listings)

    def module_object_contained(self, node, unit):
        """exec 进模块对象这一处（node）的那个模块对象出不出这份模块（文件头「exec 进模块对象」，规矩照 loaded_value_contained）：只有内联脚本
        （不会被别的文件 import）许出不去；绑这个模块对象的每个名字（module_object_root 追得到同一个 types.ModuleType 调用的模块级赋值）每一处读，
        都只在 <名>.属性（不是 __dict__）、getattr / hasattr(<名>, …)、条件、exec 进模块对象的命名空间（<名>.__dict__）与 <别名> = <名> 里，才算出不去；
        有一处别的用法（当参数交出去、放进容器、return、and / or 的值）→ False，照整个闭包见过的属性名连。"""
        if self.inline is None or self.tree is None:
            return False
        root = self.module_object_root(node.args[1].value.id, unit)
        if root is None:
            return False
        aliases = set()
        for statement in ast.walk(self.tree):
            if isinstance(statement, ast.Assign) and len(statement.targets) == 1 and isinstance(statement.targets[0], ast.Name):
                found = self.module_object_root(statement.targets[0].id, self.module_body)
                if found is not None and found[0] is root[0]:
                    aliases.add(statement.targets[0].id)
        parents = {id(child): parent for parent in ast.walk(self.tree) for child in ast.iter_child_nodes(parent)}
        for name_node in ast.walk(self.tree):
            if not (isinstance(name_node, ast.Name) and name_node.id in aliases and isinstance(name_node.ctx, ast.Load)):
                continue
            parent = parents.get(id(name_node))
            if isinstance(parent, ast.Attribute) and parent.value is name_node:
                if parent.attr != "__dict__":
                    continue
                consumer = parents.get(id(parent))
                if isinstance(consumer, ast.Call) and isinstance(consumer.func, ast.Name) and consumer.func.id == "exec" and len(consumer.args) == 2 \
                        and consumer.args[1] is parent and self.module_object_code(consumer, self.module_body) is not None:
                    continue
                return False
            if isinstance(parent, ast.Call) and isinstance(parent.func, ast.Name) and parent.func.id in ("getattr", "hasattr") and parent.args[:1] == [name_node]:
                continue
            if isinstance(parent, ast.Assign) and parent.value is name_node and len(parent.targets) == 1 and isinstance(parent.targets[0], ast.Name) \
                    and parent.targets[0].id in aliases:
                continue
            if (isinstance(parent, (ast.If, ast.While, ast.IfExp, ast.Assert)) and parent.test is name_node) or \
                    (isinstance(parent, ast.UnaryOp) and isinstance(parent.op, ast.Not)):
                continue
            return False
        return True

    def compiled_part_exec(self, node, unit):
        """exec(<码>) 这一处是不是按部分文件 exec（文件头「按部分文件 exec」）：在模块级（module-body、main-guard 两个单元，连同它们里的 for、if、try、with），
        实参不是字面串，同一个单元里有一处 compile(<源码>, <文件名>, "exec")：compile 与 exec 都是内建的（这一份没另绑这两个名字），mode 是字面 "exec"，
        文件名那一格不是字面串（是读出那份文件的路径；字面串的文件名是一段别处交来的源码的标签，不算这一种）。
        exec 进模块对象的那几处（module_object_code）的 compile 是那一处自己的源码，不算（弄坏开关 python-module-object-exec-refused 下照算）。"""
        if unit is None or unit.kind not in ("module-body", "main-guard") or not node.args or isinstance(node.args[0], ast.Constant):
            return False
        consumed = set()
        if "python-module-object-exec-refused" not in self.index.breaks:
            for inner in walk_nodes(unit.nodes or []):
                if isinstance(inner, ast.Call) and isinstance(inner.func, ast.Name) and inner.func.id == "exec" and self.binding("exec", unit) is None:
                    code = self.module_object_code(inner, unit)
                    if code is not None:
                        consumed.add(id(code))
        for inner in walk_nodes(unit.nodes or []):
            if id(inner) in consumed or not (isinstance(inner, ast.Call) and isinstance(inner.func, ast.Name) and inner.func.id == "compile"
                                             and self.binding("compile", unit) is None):
                continue
            keywords = {keyword.arg: keyword.value for keyword in inner.keywords if keyword.arg is not None}
            filename = inner.args[1] if len(inner.args) > 1 else keywords.get("filename")
            mode = inner.args[2] if len(inner.args) > 2 else keywords.get("mode")
            if isinstance(mode, ast.Constant) and mode.value == "exec" and filename is not None and not isinstance(filename, ast.Constant):
                return True
        return False

    @staticmethod
    def launch_texts(node, item):
        """起子进程那一处拿去比登记表的词：折不出的那个元素，或这一处的头一个参数，照 ast.unparse。"""
        texts = [ast.unparse(item[0])] if isinstance(item, tuple) and isinstance(item[0], ast.AST) else []
        if node.args:
            texts.append(ast.unparse(node.args[0]))
        return texts

    def launch_word_registered(self, node, item, unit):
        """起子进程那一处折不出的词登在 UNKEYED_PROGRAM_WORDS 里（文件头「不拒算的程序词」）：折不出的那个元素，或这一处的头一个参数，照 ast.unparse 比。"""
        return program_word_registered(self.source_file(), registered_function_name(unit), self.launch_texts(node, item))

    def element_over_fold_limit(self, item):
        """起子进程那一处折不出的元素 item 是不是取值超过 FOLD_LIMIT 种才折不出的：(节点, 所在模块, 所在单元) 照它所在的模块再折一次（fold 不记答案），
        看 index.fold_limit_hits 涨没涨；OPAQUE、ValueElement 与 * 展开交 False（它们的取值在调用方那一层算过）。"""
        if not isinstance(item, tuple) or len(item) != 3 or isinstance(item[0], ast.Starred):
            return False
        element, module, owner = item
        hits = module.index.fold_limit_hits
        module.fold(element, owner)
        return module.index.fold_limit_hits > hits

    def refuse_launch(self, facts, form, node, item, unit, over_limit=False):
        """起子进程那一处折不出（文件头「调用方交进来的命令」）：item 是折不出的那个元素 (节点, 所在模块, 所在单元)（OPAQUE、ValueElement 与 * 展开当推不出），
        unit 是起子进程的那个单元。登在 UNKEYED_PROGRAM_WORDS 里的不拒算（文件头「不拒算的程序词」）。
        折不出是因为取值超过 FOLD_LIMIT 种（over_limit：调用方算整条命令时量的；或这个元素再折一次 fold_limit_hits 涨了）：当场拒算 FOLD_LIMIT_FORM，
        不当调用方交进来的（文件头「拒算」公共一行；弄坏开关 fold-limit-launches-skipped 下不判这一条）。
        折不出的那几截出处都推得出是函数参数、sys.argv、argparse 的属性、临时目录、根外前缀的，记进 facts.caller_launches，收闭包时判是不是入口参数
        （ScriptClosure.caller_launch_refusals）；推不出、有环境变量的当场拒算。弄坏开关 python-caller-launches-refused 下一律当场拒算。"""
        launched = launched_programs_registered(self.source_file(), registered_function_name(unit), self.launch_texts(node, item), self.index.root)
        if launched is not None:
            # 登记的被起程序（文件头）：整份连上（program_arguments 里没有它的参数，照 push_program 不剪）
            facts.program_links.extend(launched)
            facts.handled.append(("registered-launched-program", node.lineno))
            return
        if self.launch_word_registered(node, item, unit):
            facts.handled.append(("registered-program-word", node.lineno))
            return
        built = built_tool_launch_source(self.source_file(), registered_function_name(unit), self.launch_texts(node, item))
        if built is not None:
            # 起的是编出来的工具（BUILT_TOOL_LAUNCHES）：不拒算，编它的源码目录收闭包时收成数据目录
            facts.built_tools.append(built)
            facts.handled.append(("built-tool", node.lineno))
            return
        if "fold-limit-launches-skipped" not in self.index.breaks and (over_limit or self.element_over_fold_limit(item)):
            self.refuse(facts, FOLD_LIMIT_FORM, node)
            return
        origins = None
        if "python-caller-launches-refused" not in self.index.breaks and isinstance(item, tuple) and not isinstance(item[0], ast.Starred):
            element, module, owner = item
            origins = module.unfolded_part_origins(element, owner)
        if origins and all(origin[0] == "param" or (origin[0] == "skip" and origin[1] in CALLER_INPUT_ORIGINS) for origin in origins):
            facts.caller_launches.append((self.refusal(form, node.lineno), frozenset(origins)))
        else:
            self.refuse(facts, form, node)

    def registry_listed_programs(self, unit, facts, line):
        """起子进程的那个函数登在本模块的 REGISTRY_LISTED_PROGRAMS 里（文件头「登记表点名的程序」）→ 登记表点名的、根里在的脚本（相对根，排好序）；
        没登记 → None；登记表读不了、正则写错 → 拒算 subprocess-path-missing、交 []。弄坏开关 python-registry-programs-linked 下登记表另记进
        facts.registry_reads（收闭包时收成数据单元）。"""
        scope = unit
        while scope is not None and enclosing_unit(scope) is not None:
            scope = enclosing_unit(scope)
        found = self.module_dictionary(REGISTRY_LISTED_PROGRAMS_TABLE, None) if scope is not None else None
        if found is None:
            return None
        dictionary, assignment = found
        for key, value in zip(dictionary.keys, dictionary.values):
            if not (isinstance(key, ast.Constant) and key.value == scope.name):
                continue
            if not (isinstance(value, (ast.Tuple, ast.List)) and len(value.elts) == 2):
                return None
            paths = self.fold(value.elts[0], assignment)
            pattern = value.elts[1].value if isinstance(value.elts[1], ast.Constant) and isinstance(value.elts[1].value, str) else None
            if not paths or len(paths) != 1 or pattern is None:
                return None
            registry = next(iter(paths))
            relative = self.index.relative(posixpath.normpath(registry if posixpath.isabs(registry) else posixpath.join(self.index.root, registry)))
            try:
                if relative is None:
                    raise OSError(f"登记表 {registry} 不在根里")
                with open(os.path.join(self.index.root, relative), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                expression = re.compile(pattern)
            except (OSError, re.error):
                facts.refusals.append(self.refusal("subprocess-path-missing", line))
                return []
            if "python-registry-programs-linked" in self.index.breaks:
                facts.registry_reads.append(relative)
            return sorted({match.group(1) for match in expression.finditer(text)
                           if match.groups() and match.group(1) and os.path.isfile(os.path.join(self.index.root, match.group(1)))})
        return None

    def working_directories(self, keywords, unit):
        """subprocess 的 cwd=：折得出就是它（几个都算），不然是根与这份文件所在的目录（相对路径依次试）。"""
        fallback = (self.index.root, posixpath.dirname(self.absolute)) if self.inline is None else tuple(self.inline.bases)
        directory = keywords.get("cwd")
        folded = self.path_values(directory, unit, directory=True) if directory is not None else None
        if not folded:
            return fallback
        return tuple(sorted(value if posixpath.isabs(value) else posixpath.normpath(posixpath.join(self.index.root, value)) for value in folded))

    def command_alternatives(self, node, unit, depth=0):
        """命令表达式 → 它的几种取法 [[(元素节点, 所在模块, 所在单元) 或 OPAQUE…]…]；认不出 → None。
        认：列表 / 元组字面量（*展开的认得出就摊开，认不出占一格 OPAQUE）、a + b、x if c else y、推导式（OPAQUE）、list(…)；
        名字：本函数里的赋值（连同 += / .append / .extend 往后加的）、函数参数（按本模块里每一处调用给的实参）、模块级赋值；
        调本模块或 from 仓内模块 import 进来的函数：它各处 return 的值。"""
        found = self.command_alternatives_once(node, unit, depth)
        return unique_alternatives(found) if found is not None else None

    def command_alternatives_once(self, node, unit, depth):
        if depth > 20 or node is None:
            return None
        if isinstance(node, (ast.List, ast.Tuple)):
            alternatives = [[]]
            for element in node.elts:
                if isinstance(element, ast.Starred):
                    parts = self.command_alternatives(element.value, unit, depth + 1) or [[OPAQUE]]
                else:
                    parts = [[(element, self, unit)]]
                alternatives = unique_alternatives([left + right for left in alternatives for right in parts])
                if len(alternatives) > FOLD_LIMIT:
                    self.index.fold_limit_hits += 1
                    return None
            return alternatives
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            left, right = self.command_alternatives(node.left, unit, depth + 1), self.command_alternatives(node.right, unit, depth + 1)
            if "python-command-concatenation-unfolded" not in self.index.breaks:
                # 一边认不出（git + step 里循环取的 step）：那一段占一格 OPAQUE，同 *展开认不出；命令位置上的词落在它上面照旧拒算（command_links）
                left = left if left is not None else [[OPAQUE]]
                right = right if right is not None else [[OPAQUE]]
            if left is None or right is None:
                return None
            if len(left) * len(right) > FOLD_LIMIT:
                self.index.fold_limit_hits += 1
                return None
            return [first + second for first in left for second in right]
        if isinstance(node, ast.IfExp):
            body, other = self.command_alternatives(node.body, unit, depth + 1), self.command_alternatives(node.orelse, unit, depth + 1)
            return None if body is None or other is None else body + other
        if isinstance(node, (ast.ListComp, ast.GeneratorExp)):
            return [[OPAQUE]]
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in ("list", "tuple") and len(node.args) == 1 and \
                self.binding(node.func.id, unit) is None:
            return self.command_alternatives(node.args[0], unit, depth + 1)
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Slice) and node.slice.upper is None and node.slice.step is None and \
                isinstance(node.slice.lower, ast.Constant) and type(node.slice.lower.value) is int and node.slice.lower.value >= 1:
            # 内联脚本的 sys.argv[k:]：起它的那条命令里脚本之后从第 k 个起的那几个词
            return [[ValueElement(values) for values in self.inline.argv[node.slice.lower.value - 1:]]]
        if isinstance(node, ast.Constant) and node.value is None:
            # [*options["launcher"], …] 里取到 None 的那一种：那一支跑不到这里（*None 会抛），不添词
            return [[]]
        if isinstance(node, ast.Subscript) and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, str) and \
                "python-command-dictionaries-unfolded" not in self.index.breaks:
            # 字典[<字面键>]：字典折回它的字面量（经本函数的赋值、函数参数的每一处调用、dict(底, 键=…)、字典[键] = … 的写入），取那个键的值
            sources = self.dictionary_values(node.value, node.slice.value, unit, depth + 1)
            if sources is None:
                return None
            alternatives = []
            for value, module, owner in sources:
                found = module.command_alternatives(value, owner, depth + 1)
                if found is None:
                    return None
                alternatives += found
            return alternatives or None
        if isinstance(node, ast.Subscript) and "python-command-slices-unfolded" not in self.index.breaks:
            return self.subscript_command_alternatives(node, unit, depth)
        if isinstance(node, ast.Name):
            return self.name_command_alternatives(node.id, unit, depth)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
            return self.call_command_alternatives(node.func.id, unit, depth)
        return None

    def subscript_command_alternatives(self, node, unit, depth):
        """命令表达式是下标（文件头「起代码的写法逐种认」python 的命令）：f(…)[k] 取 f 各处 return 的元组里第 k 项再认；
        x[a:b]（a、b 是不小于 0 的整数字面量或不写，b 另认 x.index(<字面词>)：到头一个等于它的词之前）把 x 的每种取法照切；别的 → None。"""
        index = node.slice
        if isinstance(index, ast.Constant) and type(index.value) is int and index.value >= 0 and isinstance(node.value, ast.Call) \
                and isinstance(node.value.func, ast.Name):
            returned = self.returned_elements(node.value.func.id, unit, index.value)
            if returned is None:
                return None
            alternatives = []
            for value, module, owner in returned:
                found = module.command_alternatives(value, owner, depth + 1)
                if found is None:
                    return None
                alternatives += found
            return alternatives or None
        if not isinstance(index, ast.Slice) or index.step is not None:
            return None
        base = self.command_alternatives(node.value, unit, depth + 1)
        if base is None:
            return None
        out = []
        for alternative in base:
            cut = self.sliced_alternative(alternative, index, node.value, unit)
            if cut is None:
                return None
            if cut is not False:
                out.append(cut)
        return out or None

    def cannot_equal(self, node, unit, wanted):
        """折不出的元素一定不等于 wanted：f 串里有一段字面字不在 wanted 里（f"{X}={n}" 一类环境赋值），或路径的字面尾巴 wanted 不以它收尾。"""
        if isinstance(node, ast.JoinedStr):
            return any(isinstance(part, ast.Constant) and str(part.value) not in wanted for part in node.values)
        tail = self.literal_tail(node, unit)
        return bool(tail) and not wanted.endswith(tail)

    def environment_assignment_shape(self, node, unit):
        """折不出的元素是 env 的一条 NAME=VALUE：f"{名字常量}=…"、f"NAME=…" 一类，名字那一段折得出、后面跟着字面的 = → "NAME=…"（给 command_positions 认包装）；别的 → None。"""
        if not isinstance(node, ast.JoinedStr):
            return None
        prefix = ""
        for part in node.values:
            if isinstance(part, ast.Constant):
                text = str(part.value)
            elif isinstance(part, ast.FormattedValue) and part.format_spec is None and part.conversion in (-1, 115):
                folded = self.fold(part.value, unit)
                if folded is None or len(folded) != 1:
                    return None
                text = next(iter(folded))
            else:
                return None
            if "=" in text:
                name = prefix + text.split("=", 1)[0]
                return f"{name}=…" if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name) else None
            prefix += text
        return None

    def sliced_alternative(self, alternative, index, sequence, unit):
        """一种取法 [(元素节点, 所在模块, 所在单元) 或 OPAQUE…] 照切片 index 切 → 切出来的那一段；切点前有认不出个数或取值的元素 → None；
        上界是 x.index(<字面词>) 而这种取法里没有等于它的词（跑起来抛 ValueError、走不到起子进程）→ False。"""
        def bound(value):
            if value is None:
                return None, True
            if isinstance(value, ast.Constant) and type(value.value) is int and value.value >= 0:
                return value.value, True
            return None, False
        lower, lower_known = bound(index.lower)
        upper, upper_known = bound(index.upper)
        if not lower_known:
            return None
        if not upper_known:
            call = index.upper
            if not (isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and call.func.attr == "index" and len(call.args) == 1
                    and not call.keywords and isinstance(sequence, ast.Name) and isinstance(call.func.value, ast.Name)
                    and call.func.value.id == sequence.id):
                return None
            wanted = self.fold(call.args[0], unit)
            if wanted is None or len(wanted) != 1:
                return None
            wanted = next(iter(wanted))
            upper = None
            for position, item in enumerate(alternative):
                if item is OPAQUE or isinstance(item, ValueElement) or isinstance(item[0], ast.Starred):
                    return None
                value = {"python3"} if item[1].canonical(item[0], item[2]) == "sys.executable" else item[1].fold(item[0], item[2])
                if value is None:
                    if item[1].cannot_equal(item[0], item[2], wanted):
                        continue
                    return None
                if value == {wanted}:
                    upper = position
                    break
                if wanted in value:
                    return None
            if upper is None:
                return False
        reach = len(alternative) if upper is None else upper
        if any(item is OPAQUE or (not isinstance(item, ValueElement) and isinstance(item[0], ast.Starred)) for item in alternative[:max(reach, lower or 0)]):
            return None
        return alternative[lower or 0:upper]

    def dictionary_values(self, node, key, unit, depth):
        """node 折回字典字面量时，各字面量里 key 的值 [(值节点, 所在模块, 所在单元)…]（连同 node[key] = … 写进去的）；有一种来路认不出 → None。"""
        if depth > 30 or node is None:
            return None
        if isinstance(node, ast.Constant) and node.value is None:
            # 取到 None 的那一支取不了键（会抛），不添取值
            return []
        if isinstance(node, ast.IfExp):
            body, other = self.dictionary_values(node.body, key, unit, depth + 1), self.dictionary_values(node.orelse, key, unit, depth + 1)
            return None if body is None or other is None else body + other
        if isinstance(node, ast.Dict):
            return [(value, self, unit) for name, value in zip(node.keys, node.values) if isinstance(name, ast.Constant) and name.value == key]
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "dict" and self.binding("dict", unit) is None:
            given = [(keyword.value, self, unit) for keyword in node.keywords if keyword.arg == key]
            if given:
                return given
            for keyword in node.keywords:
                if keyword.arg is None:
                    # dict(底, **改动)：改动那一份也折回字面量，给了这个键就是它
                    spread = self.dictionary_values(keyword.value, key, unit, depth + 1)
                    if spread is None:
                        return None
                    given += spread
            base = self.dictionary_values(node.args[0], key, unit, depth + 1) if node.args else []
            return None if base is None else given + base
        if not isinstance(node, ast.Name) or unit is None:
            return None
        overrides = self.__dict__.setdefault("dictionary_overrides", {})
        if (id(unit), node.id) in overrides:
            return overrides[(id(unit), node.id)]
        progress = self.index.__dict__.setdefault("dictionary_progress", set())
        marker = (self.path, id(unit), node.id, key)
        if marker in progress:
            # 绕回来了（saved = options["launcher"] … options["launcher"] = saved 一类存了再放回去）：这一圈不添新的取值
            return []
        progress.add(marker)
        try:
            found = self.dictionary_name_values(node, key, unit, depth)
        finally:
            progress.discard(marker)
        if found is None:
            return None
        unique, seen = [], set()
        for item in found:
            if id(item[0]) not in seen:
                seen.add(id(item[0]))
                unique.append(item)
        return unique

    def dictionary_name_values(self, node, key, unit, depth):
        overrides = self.__dict__.setdefault("dictionary_overrides", {})
        function = unit.node if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None
        found, written, unpacked = [], [], False
        statements = walk_function_statements(function) if function is not None else walk_nodes(self.module_body.nodes or [])
        for statement in statements:
            if isinstance(statement, ast.Assign):
                for target in statement.targets:
                    if isinstance(target, ast.Name) and target.id == node.id:
                        found.append(statement.value)
                    elif isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name) and target.value.id == node.id and \
                            isinstance(target.slice, ast.Constant) and target.slice.value == key:
                        written.append((statement.value, self, unit))
                    elif isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name) and target.value.id == node.id:
                        # 字典[<变量键>] = …：键折得出、又没有这个键的，与这个键无关；折不出就认不出写进去的是哪一项
                        keys = self.fold(target.slice, unit)
                        table = target.slice.value if isinstance(target.slice, ast.Subscript) and isinstance(target.slice.value, ast.Name) else None
                        if keys is None and table is not None and function is not None:
                            # 键取自本函数里只赋过一次的字典字面量（flags[argument] 一类）：键是那张表的各个值
                            assigned = self.local_assignments(unit).get(table.id, [])
                            if len(assigned) == 1 and isinstance(assigned[0], ast.Dict):
                                keys = {value.value for value in assigned[0].values if isinstance(value, ast.Constant)} \
                                    if all(isinstance(value, ast.Constant) for value in assigned[0].values) else None
                        if keys is None or key in keys:
                            return None
                    elif isinstance(target, (ast.Tuple, ast.List)):
                        for position, item in enumerate(target.elts):
                            if isinstance(item, ast.Name) and item.id == node.id:
                                pairs = paired_elements(target, statement.value)
                                if pairs is not None:
                                    # a, options = x, {…}：对上的那一项
                                    found.append(dict(pairs)[node.id])
                                    continue
                                # a, options = f(…)：取 f 各处 return 的元组里这个位置的那一项
                                if not (isinstance(statement.value, ast.Call) and isinstance(statement.value.func, ast.Name)):
                                    return None
                                returned = self.returned_elements(statement.value.func.id, unit, position)
                                if returned is None:
                                    return None
                                for value, module, owner in returned:
                                    inner = module.dictionary_values(value, key, owner, depth + 1)
                                    if inner is None:
                                        return None
                                    written += inner
                                unpacked = True
                            if isinstance(item, ast.Subscript) and isinstance(item.value, ast.Name) and item.value.id == node.id and \
                                    isinstance(item.slice, ast.Constant) and item.slice.value == key:
                                pairs = statement.value.elts if isinstance(statement.value, (ast.Tuple, ast.List)) else None
                                if pairs is None or position >= len(pairs):
                                    return None
                                written.append((pairs[position], self, unit))
        values = list(written)
        # options = dict(options, …) 一类引用自己的赋值：里面的 options 是之前的值（别的赋值，没有就是参数 / 模块级那一份）
        own = [value for value in found if any(isinstance(inner, ast.Name) and inner.id == node.id for inner in ast.walk(value))]
        plain = [value for value in found if not any(value is item for item in own)]
        if plain:
            base = []
            for value in plain:
                inner = self.dictionary_values(value, key, unit, depth + 1)
                if inner is None:
                    return None
                base += inner
        elif unpacked:
            base = []
        else:
            base = self.dictionary_parameter_values(node, key, unit, function, depth)
            if base is None:
                return None
        if not own:
            return values + base
        overrides[(id(unit), node.id)] = base
        try:
            for value in own:
                inner = self.dictionary_values(value, key, unit, depth + 1)
                if inner is None:
                    return None
                values += inner
        finally:
            overrides.pop((id(unit), node.id), None)
        return values + (base if plain else [])

    def returned_elements(self, name, unit, position):
        """调仓内模块级函数 name：它各处 return 的元组里第 position 项 [(节点, 模块, 单元)…]；有一处不是元组字面量 → None。"""
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
        out = []
        for statement in walk_function_statements(candidates[0].node):
            if isinstance(statement, ast.Return) and statement.value is not None:
                if not isinstance(statement.value, (ast.Tuple, ast.List)) or position >= len(statement.value.elts):
                    return None
                out.append((statement.value.elts[position], module, candidates[0]))
        return out or None

    def returned_values(self, name, unit):
        """调仓内模块级函数 name（本模块的，或 from 仓内模块 import 进来的）：(它的模块, 它的单元, 各处 return 的值)；认不出、一处 return 值都没有 → None。"""
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
        values = [statement.value for statement in walk_function_statements(candidates[0].node) if isinstance(statement, ast.Return) and statement.value is not None]
        return (module, candidates[0], values) if values else None

    def dictionary_parameter_values(self, node, key, unit, function, depth):
        """dictionary_values 里名字不在本函数里赋值时：函数参数按每一处调用给的实参折，模块级的按它那一处赋值折。"""
        values = []
        if function is not None:
            arguments = function.args
            positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
            if node.id in positional and unit.container is None:
                position = positional.index(node.id)
                defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
                sites = self.function_call_sites(unit)
                if not sites:
                    return None
                for call, caller in sites:
                    argument = call.args[position] if position < len(call.args) else next(
                        (keyword.value for keyword in call.keywords if keyword.arg == node.id), None)
                    if argument is None and node.id in defaults:
                        # 这一处调用没给：取缺省值（在被调函数那边认）
                        inner = self.dictionary_values(defaults[node.id], key, unit, depth + 1)
                        if inner is None:
                            return None
                        values += inner
                        continue
                    if argument is None:
                        return None
                    inner = self.dictionary_values(argument, key, caller, depth + 1)
                    if inner is None:
                        return None
                    values += inner
                return values
            # **名字：本函数或套在里面的函数收的关键字参数，这个键的值是各处调用给的同名关键字参数
            nested = [function] + [inner for inner in ast.walk(function) if isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef)) and inner is not function]
            owner = next((definition for definition in nested if definition.args.kwarg is not None and definition.args.kwarg.arg == node.id), None)
            if owner is None:
                return None
            if owner is function:
                sites = self.function_call_sites(unit) if unit.container is None else []
            else:
                sites = [(call, unit) for call in ast.walk(function) if isinstance(call, ast.Call) and isinstance(call.func, ast.Name) and call.func.id == owner.name]
            if not sites:
                return None
            for call, caller in sites:
                for keyword in call.keywords:
                    if keyword.arg == key:
                        values.append((keyword.value, self, caller))
                    elif keyword.arg is None:
                        inner = self.dictionary_values(keyword.value, key, caller, depth + 1)
                        if inner is None:
                            return None
                        values += inner
            return values
        candidates = [candidate for candidate in self.by_name.get(node.id, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
        if len(candidates) != 1:
            return None
        inner = self.dictionary_values(candidates[0].node.value, key, candidates[0], depth + 1)
        return None if inner is None else values + inner

    def name_command_alternatives(self, name, unit, depth):
        overridden = getattr(self, "command_overrides", {}).get((id(unit), name))
        if overridden is not None:
            return overridden
        progress = self.index.__dict__.setdefault("command_progress", set())
        marker = (self.path, id(unit), name)
        if marker in progress:
            # 绕回来了（存了再放回去的那一类）：这一圈不添新的取法
            return [[]]
        progress.add(marker)
        try:
            return self.name_command_alternatives_once(name, unit, depth)
        finally:
            progress.discard(marker)

    def name_command_alternatives_once(self, name, unit, depth):
        function = unit.node if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) else None
        # 模块体的语句照函数体同一条认（循环、分支里赋的名字，for 取的字面列表）；弄坏开关 python-module-body-commands-unfolded 下不认
        scope = function
        if function is None and unit is not None and unit.kind == "module-body" and "python-module-body-commands-unfolded" not in self.index.breaks:
            scope = ast.Module(body=list(unit.nodes or []), type_ignores=[])
        if scope is not None:
            assigned, appended = [], []
            for statement in walk_function_statements(scope):
                if isinstance(statement, ast.Assign):
                    for target in statement.targets:
                        if isinstance(target, ast.Name) and target.id == name:
                            assigned.append(statement.value)
                        elif name in dict(paired_elements(target, statement.value) or ()):
                            assigned.append(dict(paired_elements(target, statement.value))[name])
                        elif name in bound_names(target):
                            return None
                elif isinstance(statement, (ast.AnnAssign, ast.NamedExpr)) and isinstance(statement.target, ast.Name) and statement.target.id == name:
                    assigned.append(statement.value)
                elif isinstance(statement, ast.AugAssign) and isinstance(statement.target, ast.Name) and statement.target.id == name:
                    if not isinstance(statement.op, ast.Add):
                        return None
                    appended.append(("extend", statement.value))
                elif isinstance(statement, (ast.For, ast.AsyncFor)) and isinstance(statement.target, ast.Name) and statement.target.id == name \
                        and isinstance(statement.iter, (ast.Tuple, ast.List)):
                    # for 命令 in (<列表>, …)：每个元素是一种取法
                    assigned += list(statement.iter.elts)
                elif isinstance(statement, (ast.For, ast.AsyncFor, ast.With, ast.AsyncWith)) and name in {inner.id for inner in walk_targets(statement)}:
                    return None
                elif isinstance(statement, ast.Call) and isinstance(statement.func, ast.Attribute) and isinstance(statement.func.value, ast.Name) \
                        and statement.func.value.id == name and statement.func.attr in ("append", "extend", "insert"):
                    if statement.func.attr == "insert" or len(statement.args) != 1:
                        return None
                    appended.append((statement.func.attr, statement.args[0]))
            arguments = function.args if function is not None else None
            parameters = [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs] if arguments is not None else []
            if assigned and function is None:
                # 模块体里循环、分支赋的之外，模块顶层赋的那几处也是取法；顶层绑的不是单个名字的赋值（def、import、拆包）认不出
                for candidate in self.by_name.get(name, ()):
                    if candidate.kind != "assign" or not isinstance(candidate.node, ast.Assign) or not all(isinstance(target, ast.Name) for target in candidate.node.targets):
                        return None
                    assigned.append(candidate.node.value)
            if assigned:
                # command = [..., *command] 一类引用自己的赋值：里面的 command 是别的那几处赋的值
                own = [value for value in assigned if any(isinstance(inner, ast.Name) and inner.id == name for inner in ast.walk(value))]
                bases = []
                for value in assigned:
                    if any(value is item for item in own):
                        continue
                    found = self.command_alternatives(value, unit, depth + 1)
                    if found is None:
                        return None
                    bases += found
                if own and bases:
                    if not hasattr(self, "command_overrides"):
                        self.command_overrides = {}
                    self.command_overrides[(id(unit), name)] = list(bases)
                    try:
                        for value in own:
                            found = self.command_alternatives(value, unit, depth + 1)
                            if found is None:
                                return None
                            bases += found
                    finally:
                        self.command_overrides.pop((id(unit), name), None)
                elif own:
                    return None
            elif name in parameters:
                bases = self.parameter_command_alternatives(unit, name, depth)
                if bases is None:
                    return None
            elif appended and function is not None:
                bases = [[]]
            else:
                bases = None
            if bases is not None:
                for method, value in appended:
                    parts = [[(value, self, unit)]] if method == "append" else (self.command_alternatives(value, unit, depth + 1) or [[OPAQUE]])
                    bases = [left + right for left in bases for right in parts]
                    if len(bases) > FOLD_LIMIT:
                        self.index.fold_limit_hits += 1
                        return None
                return bases
            if enclosing_unit(unit) is not None:
                # 套着的函数里没赋、不是参数的名字：外层函数的局部变量（闭包变量）或模块级的
                return self.name_command_alternatives(name, enclosing_unit(unit), depth + 1)
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
        if len(candidates) != 1 or name in self.body_defines:
            return None
        return self.command_alternatives(candidates[0].node.value, candidates[0], depth + 1)

    def parameter_command_alternatives(self, unit, name, depth):
        """命令是函数参数：本模块里调这个函数的每一处给的那个实参（在调用方那边认），合起来；有一处认不出、没有调用处 → None。
        类里的方法按同一个类里 self.<方法>(…) 那几处调用认（实参位置去掉 self）。"""
        function = unit.node
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        if unit.container is not None:
            if not positional or position == 0:
                return None
            position = position - 1 if position is not None else None
            sites = [(node, method) for method in self.methods if method.container is unit.container for node in walk_nodes(method.nodes or [])
                     if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == function.name
                     and isinstance(node.func.value, ast.Name) and node.func.value.id == "self"]
        else:
            sites = self.function_call_sites(unit)
        if not sites:
            return None
        alternatives = []
        for call, caller in sites:
            argument = None
            if position is not None and position < len(call.args):
                if any(isinstance(item, ast.Starred) for item in call.args[:position + 1]):
                    return None
                argument = call.args[position]
            for keyword in call.keywords:
                if keyword.arg is None:
                    return None
                if keyword.arg == name:
                    argument = keyword.value
            if argument is None:
                argument = defaults.get(name)
            if argument is None:
                return None
            found = self.command_alternatives(argument, caller, depth + 1)
            if found is None:
                return None
            alternatives += found
            if len(alternatives) > FOLD_LIMIT:
                self.index.fold_limit_hits += 1
                return None
        return alternatives

    def call_command_alternatives(self, name, unit, depth):
        """命令是调一个仓内模块级函数的返回值（本模块的，或 from 仓内模块 import 进来的）：它各处 return 的值（在它那边认）合起来。"""
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
        key = (module.path, id(callee), "<command>")
        if key in self.index.folding:
            return None
        self.index.folding.add(key)
        try:
            alternatives = []
            for statement in walk_function_statements(callee.node):
                if isinstance(statement, ast.Return):
                    if statement.value is None:
                        continue
                    found = module.command_alternatives(statement.value, callee, depth + 1)
                    if found is None:
                        return None
                    alternatives += found
            return alternatives or None
        finally:
            self.index.folding.discard(key)

    def command_links(self, elements, unit, node, facts, keywords=None):
        """一种取法的命令（[(元素节点, 所在模块, 所在单元) 或 OPAQUE…]）→ 连边、内联脚本与拒算。命令位置（经 env / nice / timeout… 之后）上的词
        折不出 → 拒算 subprocess-command-unfolded；解释器后面照它的写法认（脚本、-c 的串、-m 的模块、input= 喂的标准输入），脚本之后接着往后认。"""
        keywords = keywords or {}
        breaks = self.index.breaks
        values = []
        for item in elements:
            if item is OPAQUE:
                values.append(None)
                continue
            if isinstance(item, ValueElement):
                values.append(set(item.values) if item.values is not None and item.values is not POSITIONAL else None)
                continue
            element, module, owner = item
            if isinstance(element, ast.Starred):
                values.append(None)
            elif module.canonical(element, owner) == "sys.executable":
                values.append({"python3"})
            else:
                values.append(module.fold(element, owner))

        def single(position):
            value = values[position] if position < len(values) else None
            return next(iter(value)) if value is not None and len(value) == 1 else None

        def element_call(position, method, *arguments):
            item = elements[position]
            if item is OPAQUE or isinstance(item, ValueElement) or isinstance(item[0], ast.Starred):
                return None
            element, module, owner = item
            calls = {"path_values": module.path_values, "literal_tail": module.literal_tail, "directory_wildcard": module.directory_wildcard,
                     "script_tail_group": module.script_tail_group}
            return calls[method](element, owner, *arguments)

        def path_program(position):
            """命令位置上的词是 shutil.which(…) 找到的 PATH 上的程序（或本函数里只赋过一次这个的名字）。"""
            item = elements[position]
            if item is OPAQUE or isinstance(item, ValueElement):
                return False
            element, module, owner = item
            if isinstance(element, ast.Name) and owner is not None and isinstance(owner.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                assigned = module.local_assignments(owner).get(element.id, [])
                element = assigned[0] if len(assigned) == 1 else element
            return isinstance(element, ast.Call) and module.canonical(element.func, owner) == "shutil.which"

        bases = list(self.working_directories(keywords, unit))
        literal = [single(position) for position in range(len(values))]
        shape = list(literal)
        if "python-environment-assignments-unfolded" not in breaks:
            # env 后面折不出值的 NAME=VALUE（f"{X}={n}"）：名字折得出就照一条赋值认，命令位置照样往后找
            for position, item in enumerate(elements):
                if shape[position] is None and item is not OPAQUE and not isinstance(item, ValueElement) and not isinstance(item[0], ast.Starred):
                    shape[position] = item[1].environment_assignment_shape(item[0], item[2])
        positions = command_positions(shape) if "python-unfolded-heads-allowed" not in breaks else [0]
        head = positions[-1] if positions else 0
        if head >= len(values):
            facts.handled.append(("empty", node.lineno))
            return
        if values[head] is None and "python-unfolded-heads-allowed" not in breaks:
            if isinstance(elements[head], ValueElement) and elements[head].values is POSITIONAL:
                # 内联脚本 exec 它收到的 "$@"：命令就是起它的那条 shell 命令后面那几个词（调用处照命令认过）
                facts.handled.append(("positional", node.lineno))
                return
            if "python-path-programs-refused" not in breaks and (path_program(head) or (
                    elements[head] is not OPAQUE and not isinstance(elements[head], ValueElement)
                    and elements[head][1].outside_head(elements[head][0], elements[head][2]))):
                # shutil.which(<名字>) 找到的、os.path.join(<折不出的目录>, "verus") 一类根外的程序：不是仓里的代码
                facts.handled.append(("path-program", node.lineno))
                return
            copied = element_call(head, "path_values")
            if not copied:
                tail = element_call(head, "literal_tail") if "python-path-programs-refused" not in breaks else None
                if tail and tail.strip("/") and not tail.endswith((".py", ".sh")) and tree_copy_file(self.index.root, tail.lstrip("/")) is None \
                        and not os.path.exists(os.path.join(self.index.root, tail.lstrip("/"))):
                    # os.path.join(<折不出的目录>, "bin/llvm-cov") 一类：根外的程序（不带脚本后缀、根里也没有这一截）
                    facts.handled.append(("outside-program", node.lineno))
                    return
                if "python-toolchain-queries-refused" not in breaks and len(values) == head + 2 and literal[head + 1] in TOOLCHAIN_QUERY_WORDS:
                    # [<RUSTC 或 build.rustc 给的编译器>, "-V"] 一类版本询问：只打版本、不跑仓里的代码，打的版本不进键（文件头「版本询问」）
                    facts.toolchain_queries.append(node.lineno)
                    facts.handled.append(("toolchain-query", node.lineno))
                    return
                self.refuse_launch(facts, COMMAND_FORM, node, elements[head], unit)
                facts.handled.append(("head-unfolded", node.lineno))
                return
            values[head] = copied
            literal[head] = single(head)
        if values[head] is not None and len(values[head]) > 1 and "python-command-heads-split-ignored" not in breaks:
            # 命令位置上的词折出几个（[name, "--version"] 里 name 取 python3 或 bash）：每个取值当一种取法各认一遍
            for value in sorted(values[head]):
                self.command_links(elements[:head] + [ValueElement(frozenset((value,)))] + elements[head + 1:], unit, node, facts, keywords)
            facts.handled.append(("head-split", node.lineno))
            return
        if values[head] is not None and len(values[head]) > 1:
            # 命令位置上的词折出几个：都不是解释器的，是脚本的连上、别的是外部程序；有解释器的认不出后面怎么接，拒算
            if any(posixpath.basename(value) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS for value in values[head]):
                self.refuse(facts, COMMAND_FORM, node)
                facts.handled.append(("head-several", node.lineno))
                return
            for value in sorted(values[head]):
                file = self.file_for_value(value, bases) if value.endswith((".py", ".sh")) or "/" in value else None
                if file is not None:
                    facts.program_links.append(file)
            facts.handled.append(("head-several", node.lineno))
            return
        position, interpreters = head, 0
        while position < len(values):
            word = literal[position]
            base = posixpath.basename(word) if word else None
            if base not in PYTHON_INTERPRETERS and base not in SHELL_INTERPRETERS:
                if position > head and values[position] is None and "python-command-arguments-loaded" in breaks and elements[position] is not OPAQUE:
                    # 弄坏开关：脚本后面传给它的路径参数也当装代码（那是给被起的程序的数据）
                    tail = element_call(position, "literal_tail")
                    if tail and tail.endswith((".py", ".sh")) and element_call(position, "path_values") is None:
                        self.refuse(facts, "subprocess-path-unfolded", node)
                position += 1
                continue
            interpreters += 1
            python = base in PYTHON_INTERPRETERS
            mode, argument = interpreter_mode(literal, position, python)
            facts.handled.append((f"{'python' if python else 'shell'}-{mode}", node.lineno))
            if mode == "script":
                if descriptor_of(literal[argument]) is not None:
                    self.refuse(facts, INLINE_FORM, node)
                    return
                if values[argument] is None:
                    values[argument] = element_call(argument, "path_values")
                if values[argument] is None:
                    values[argument] = element_call(argument, "directory_wildcard", ".py" if python else ".sh")
                if values[argument] is None and "python-written-scripts-ignored" not in breaks and elements[argument] is not OPAQUE \
                        and not isinstance(elements[argument], ValueElement) and elements[argument][2] is not None \
                        and ast.dump(elements[argument][0]) in elements[argument][1].written_files(elements[argument][2]):
                    position = argument + 1
                    continue
                if values[argument] is None and "python-script-tail-groups-ignored" not in breaks:
                    # 目录折不出、脚本名那一截是字面的（os.path.join(<临时树>, "tool/entry.py")）：仓里以这一截收尾的脚本整组连上；
                    # 一份都没有的不是仓里的脚本（这一份自己写出来的夹具、根外的程序），不连
                    group = element_call(argument, "script_tail_group")
                    if group is not None and not group:
                        facts.handled.append(("outside-script", node.lineno))
                        position = argument + 1
                        continue
                    values[argument] = group
                listed = self.registry_listed_programs(unit, facts, node.lineno) \
                    if values[argument] is None and "python-registry-programs-ignored" not in breaks else None
                if listed is not None and "python-registry-programs-linked" in breaks:
                    # 弄坏开关 python-registry-programs-linked：登记表里点名的每一份都连（整份脚本体），登记表本身收成数据单元
                    facts.program_links.extend(listed)
                    facts.handled.append(("registry-listed", node.lineno))
                elif listed is not None:
                    # 登记表点名的程序是环境探针（文件头「登记表点名的程序」）：退出码与输出由调用方处置（前提每趟现判、版本经 environment-of: 进键），不连、不拒算
                    facts.handled.append(("environment-probe", node.lineno))
                elif values[argument] is None:
                    self.refuse_launch(facts, "subprocess-path-unfolded", node, elements[argument], unit)
                else:
                    cwd_unfolded = "cwd" in keywords and not self.path_values(keywords["cwd"], unit, directory=True)
                    launched = launch_arguments(elements, values, argument + 1)
                    for value in sorted(values[argument]):
                        file = self.file_for_value(value, bases)
                        if file is None and cwd_unfolded and not posixpath.isabs(value) and value.strip("/") \
                                and "python-script-tail-groups-ignored" not in breaks:
                            # cwd= 折不出、脚本写的是相对路径（["bash", "setup.sh"], cwd=<拷出来的样本>）：仓里以 /<它> 收尾的整组连上，一份都没有的不是仓里的脚本
                            stripped = posixpath.normpath(value).strip("/")
                            group = [relative for relative in self.index.repository_files() if relative.endswith("/" + stripped)]
                            facts.program_links.extend(group)
                            if not group:
                                facts.handled.append(("outside-script", node.lineno))
                            continue
                        if file is None:
                            # 折得出、根里没有的路径（{tree} 这类模板）：登了的照登记办，没登记的照旧拒算（文件头「登记的被起程序」）
                            self.refuse_launch(facts, "subprocess-path-missing", node, elements[argument], unit)
                        else:
                            facts.program_links.append(file)
                            # 程序入口带参数（文件头「程序入口带参数」）：脚本之后的词带过去，被起的那份照它们剪
                            facts.program_arguments.setdefault(file, []).append(launched)
                            facts.shell_launch_words.setdefault(file, []).append(shell_launch_choices(elements, values, argument + 1))
                position = argument + 1
                continue
            if mode == "inline":
                if "inline-scripts-ignored" in breaks:
                    return
                texts = values[argument + 1] if argument + 1 < len(values) else None
                if texts is None and not python:
                    texts = self.registry_command_texts(unit, facts)
                if texts is None and not python and argument + 1 < len(values) and self.declared_check_runner(elements[argument + 1]):
                    # 上游 preflight 跑脚本文件头登记的 check 判据：判据正文在各份声明它的脚本里，从那些脚本连（ScriptClosure.declared_checks）
                    facts.handled.append(("declared-checks", node.lineno))
                    return
                after = [values[position] if values[position] is not None else
                         (element_call(position, "path_values") or element_call(position, "path_values", True))
                         for position in range(argument + 2, len(values))]
                self.inline_from_command(facts, node, "python" if python else "shell", texts, after, bases, unit,
                                         source=elements[argument + 1] if argument + 1 < len(elements) else None)
                if python and texts is not None and "inline-argv-commands-ignored" not in breaks:
                    # 正文把 sys.argv[k:] 当命令起：sys.argv[1] 是串之后那一个元素，从第 k 个起是一条命令
                    for offset in sorted(set().union(*(argv_command_offsets(text) for text in texts))):
                        if argument + 1 + offset < len(elements):
                            self.command_links(elements[argument + 1 + offset:], unit, node, facts, keywords)
                return
            if mode == "module":
                self.module_from_command(facts, node, values, argument, bases)
                position = argument + 2
                continue
            if mode == "stdin" or (mode == "bare" and position in positions):
                if "inline-scripts-ignored" in breaks:
                    return
                feed = keywords.get("input")
                texts = self.fold(feed, unit) if feed is not None else None
                self.inline_from_command(facts, node, "python" if python else "shell", texts, values[argument + 1:], bases, unit, stdin=True)
                return
            return
        if interpreters == 0:
            word = literal[head]
            if word and (word.endswith((".py", ".sh")) or ("/" in word and not word.startswith(("/usr/", "/bin/")))):
                file = self.file_for_value(word, bases)
                if file is not None:
                    facts.program_links.append(file)
            facts.handled.append(("program", node.lineno))

    def registry_command_texts(self, unit, facts):
        """bash -c 的串是这个函数从登记表读进来的（文件头「登记表里的命令」）→ 那一列的命令（集合；表不在是空集合），登记表记进 facts.registry_reads；没登记 → None。"""
        if "python-registry-command-texts-refused" in self.index.breaks:
            return None
        registered = REGISTRY_COMMAND_TEXTS.get((self.source_file(), registered_function_name(unit)))
        if registered is None:
            return None
        table, column = registered
        path = os.path.join(self.index.root, table)
        if not os.path.isfile(path):
            return set()
        facts.registry_reads.append(table)
        with open(path, encoding="utf-8", errors="replace") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.lstrip().startswith("#")]
        return {row[column].strip() for row in rows if len(row) > column and row[column].strip()}

    def declared_check_runner(self, item):
        """上游 preflight.py 里跑 check 判据的那一处：bash -c declaration.command（判据是被判脚本文件头里登记的那几行）。"""
        if item is OPAQUE or isinstance(item, ValueElement) or "declared-checks-ignored" in self.index.breaks:
            return False
        element = item[0]
        return self.path.endswith(PREFLIGHT_MODULE) and isinstance(element, ast.Attribute) and element.attr == "command" and \
            isinstance(element.value, ast.Name) and element.value.id == "declaration"

    def inline_from_command(self, facts, node, language, texts, after, bases, unit, stdin=False, source=None):
        """命令里 -c 的串、input= 喂的标准输入：正文折得出记成一份 InlineScript，折不出拒算 inline-script-not-literal。
        after 是串之后（标准输入时是解释器选项之后）那几个元素的取值：python 的 sys.argv[1:]；bash 的 -c 是 $0 $1 …，标准输入是 $1 …。
        unit 是起子进程的那个单元；source 是 -c 那一串的元素（折不出时看它的出处：调用方交进来的不当场拒算，refuse_launch）。"""
        if texts is None:
            self.refuse_launch(facts, INLINE_FORM, node, source, unit)
            return
        if language == "shell" and "python-registered-inline-launches-analyzed" not in self.index.breaks:
            # 这一条起法整条登记过（文件头「登记的被起程序」「不拒算的程序词」）：正文是字面的也不拆开认（$1… 换不进去的那几处会拒算），照登记办
            launched = launched_programs_registered(self.source_file(), registered_function_name(unit), self.launch_texts(node, source), self.index.root)
            if launched is not None:
                facts.program_links.extend(launched)
                facts.handled.append(("registered-launched-program", node.lineno))
                return
            if self.launch_word_registered(node, source, unit):
                facts.handled.append(("registered-program-word", node.lineno))
                return
        argv = tuple(frozenset(value) if value is not None else None for value in after)
        if language == "shell" and stdin:
            argv = (None,) + argv
        where = self.refusal("", node.lineno)
        facts.inline_scripts.append(InlineScript(language, tuple(sorted(texts)), argv, tuple(bases), where.file, where.line, where.snippet))

    def module_from_command(self, facts, node, values, argument, bases):
        """[python3, -m, <模块>, …]：同 shell 那边的 resolve_module_run。"""
        if "python-modules-ignored" in self.index.breaks:
            return
        names = values[argument + 1] if argument + 1 < len(values) else None
        if names is None:
            self.refuse(facts, "import-module-variable", node)
            return
        for name in sorted(names):
            if name not in TEST_RUNNER_MODULES:
                status, file = repository_module(self.index.root, bases, name, main=True)
                if status == "repo":
                    facts.program_links.append(file)
                elif status == "unresolved":
                    self.refuse(facts, "import-unresolved", node)
                continue
            targets, position = [], argument + 2
            while position < len(values):
                value = values[position]
                one = next(iter(value)) if value is not None and len(value) == 1 else None
                if one is not None and one.startswith("-"):
                    position += 2 if one in ("-k", "-p", "-s", "-t", "-m", "--pattern", "--start-directory", "--top-level-directory") else 1
                    continue
                targets.append(value)
                position += 1
            if not targets or targets[0] == {"discover"}:
                self.refuse(facts, "import-unresolved", node)
                continue
            for target in targets:
                if target is None:
                    self.refuse(facts, "import-module-variable", node)
                    continue
                for value in sorted(target):
                    status, file = repository_module(self.index.root, bases, value)
                    if status == "repo":
                        facts.whole_modules.append(file)
                    elif status == "unresolved":
                        self.refuse(facts, "import-unresolved", node)

    def docstring_units(self, owner):
        owners = sorted(self.docstrings) if owner == "*" else [owner]
        out = []
        for name in owners:
            if name not in self.docstrings:
                continue
            if name not in self._data:
                text, first, last = self.docstrings[name]
                self._data[name] = Unit("python", "docstring", f"{name}.__doc__", self.path, first, last, sha256_text(text), data=True)
            out.append(self._data[name])
        return out

    def source_unit(self):
        if "<source-text>" not in self._data:
            self._data["<source-text>"] = Unit("python", "source-text", "<source-text>", self.path, 1, len(self.rows), sha256_bytes(self.raw), data=True)
        return self._data["<source-text>"]
