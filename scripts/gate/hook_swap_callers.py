"""先验后换（入口 hook-swap.py）的一块：引到的名字——调用方的流分析（CallerFlow、SearchPaths）、调用方文件的收集与 --check-callers。
入口 hook-swap.py 把这一块的名字再导出。
"""

import ast
import os
from typing import NamedTuple

from hook_swap_base import (
     HOOKS_DIRECTORY, EXIT_BAD_INPUT, EXIT_DRAFT_RED, EXIT_SWAPPED, BadInput, is_inside, read_text, embedded_python_segments, CALLER_SCAN_DIRECTORIES,
     CALLER_SKIPPED_DIRECTORIES, LOADER_CALLEE, HARMLESS_CALLEES, LOADER_BODY_CALLEES, CONTAINER_METHODS, MODULE_REGISTRIES, DYNAMIC_NAME_CALLEES,
     FLOW_ROUNDS, CROSS_FILE_ROUNDS, PATH_FOLD_DEPTH, PATH_PASS_THROUGH_CALLEES, PATH_JOIN_CALLEES, PATH_IDENTITY_METHODS, ENVIRONMENT_READ_CALLEES,
     UNRECOGNIZED_LOADER_CALLEES,
)

def callee_name(function_node):
    if isinstance(function_node, ast.Name):
        return function_node.id
    if isinstance(function_node, ast.Attribute):
        return function_node.attr
    return None


def dotted_path(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return None


def annotate_parents(tree):
    """每个节点记下父节点（_flow_parent）和外面最近的函数（_flow_scope）；父节点先于子节点处理（ast.walk 是自上而下的）。"""
    tree._flow_scope = None
    for node in ast.walk(tree):
        inside = node if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)) else node._flow_scope
        for child in ast.iter_child_nodes(node):
            child._flow_parent = node
            child._flow_scope = inside


def enclosing_scope(node):
    """node 外面最近的函数（FunctionDef、AsyncFunctionDef、Lambda）；在模块层是 None。"""
    return node._flow_scope


def import_aliases(imports):
    """import / from … import 绑的名字 → 它指的点路径（import os as o → o: os；from os.path import join → join: os.path.join；import os.path → os: os）。
    imports 是 ast.Import 与 ast.ImportFrom 节点的列表；整份源码一张表，不分作用域；相对导入与 * 不收。"""
    aliases = {}
    for node in imports:
        if isinstance(node, ast.Import):
            for alias in node.names:
                aliases[alias.asname or alias.name.split(".")[0]] = alias.name if alias.asname else alias.name.split(".")[0]
        elif isinstance(node, ast.ImportFrom) and node.module and not node.level:
            for alias in node.names:
                if alias.name != "*":
                    aliases[alias.asname or alias.name] = f"{node.module}.{alias.name}"
    return aliases


def canonical_path(node, aliases):
    """Name / Attribute 链的点路径，头一段按 import 别名还原（s.modules 在 import sys as s 之后是 sys.modules）；不是这种节点交 None。"""
    path = dotted_path(node)
    if path is None:
        return None
    head, dot, rest = path.partition(".")
    return aliases.get(head, head) + dot + rest


def assignment_scan(tree):
    """tree（模块或一个函数）里的名字绑定 → (名字 → 写成 `名字 = 表达式` 的每一处赋值的右边, 有别的绑定的名字：形参、循环变量、import、def、class、global、增量赋值与解包)。
    嵌在里面的函数与类里的绑定一并扫进来，赋值的右边属于哪个函数由调用方按 enclosing_scope 筛。tree 要先 annotate_parents。"""
    found, spoiled = {}, set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
            parent = getattr(node, "_flow_parent", None)
            if isinstance(parent, ast.Assign) and len(parent.targets) == 1 and parent.targets[0] is node:
                found.setdefault(node.id, []).append(parent.value)
            else:
                spoiled.add(node.id)
        elif isinstance(node, ast.arg):
            spoiled.add(node.arg)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            spoiled.add(node.name)
        elif isinstance(node, (ast.Global, ast.Nonlocal)):
            spoiled.update(node.names)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            spoiled.update((alias.asname or alias.name).split(".")[0] for alias in node.names)
    return found, spoiled


def target_names(target, names):
    if isinstance(target, ast.Name):
        names.add(target.id)
    elif isinstance(target, (ast.Tuple, ast.List)):
        for element in target.elts:
            target_names(element, names)
    elif isinstance(target, ast.Starred):
        target_names(target.value, names)


def module_level_names(source):
    """库源码 → (模块层定义的名字集合, 动态定义名字的写法说明列表)；语法错抛 SyntaxError。
    模块层 = 不在 def / class 里面的语句（含 if / try / with / for / while 里的）；函数里 global 声明过的名字也算（运行时才有，但调用方引得到）。"""
    tree = ast.parse(source)
    names, dynamic = set(), []

    def visit(statements):
        for statement in statements:
            if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                names.add(statement.name)
                if statement.name == "__getattr__":
                    dynamic.append("定义了模块层的 __getattr__")
            elif isinstance(statement, ast.Import):
                names.update((alias.asname or alias.name.split(".")[0]) for alias in statement.names)
            elif isinstance(statement, ast.ImportFrom):
                for alias in statement.names:
                    if alias.name == "*":
                        dynamic.append("有 from … import *")
                    else:
                        names.add(alias.asname or alias.name)
            elif isinstance(statement, (ast.For, ast.AsyncFor)):
                target_names(statement.target, names)
                visit(statement.body)
                visit(statement.orelse)
            elif isinstance(statement, (ast.While, ast.If)):
                visit(statement.body)
                visit(statement.orelse)
            elif isinstance(statement, (ast.With, ast.AsyncWith)):
                for item in statement.items:
                    if item.optional_vars is not None:
                        target_names(item.optional_vars, names)
                visit(statement.body)
            elif isinstance(statement, ast.Try) or type(statement).__name__ == "TryStar":
                visit(statement.body)
                for handler in statement.handlers:
                    if handler.name:
                        names.add(handler.name)
                    visit(handler.body)
                visit(statement.orelse)
                visit(statement.finalbody)
            elif isinstance(statement, ast.Match):
                for case in statement.cases:
                    visit(case.body)
            else:
                if isinstance(statement, ast.Assign):
                    for target in statement.targets:
                        target_names(target, names)
                elif isinstance(statement, (ast.AnnAssign, ast.AugAssign)):
                    target_names(statement.target, names)
                for inner in ast.walk(statement):
                    if isinstance(inner, ast.Call) and callee_name(inner.func) in DYNAMIC_NAME_CALLEES:
                        dynamic.append(f"模块层调了 {callee_name(inner.func)}()")
                        break
    visit(tree.body)
    for node in ast.walk(tree):
        if isinstance(node, ast.Global):
            names.update(node.names)
    return names, sorted(set(dynamic))


class CallerFlow:
    """一份 python 源码里，载入了目标库的变量怎么流动、用到了库里的哪些名字。
    种类是 (kind, 库名)：kind 是 "module"（已载入的库）或 "spec"（spec_from_file_location 的结果，还没变成库）。
    名字按函数分层（global / nonlocal 照认），属性路径（self.library）不分层；一份源码内不分先后，反复传播到不再多绑为止。"""

    def __init__(self, tree, targets, exports, hints, own_stem, mode=""):
        self.targets = targets          # {库名: 库文件名}
        self.exports = exports          # {库名: {库里的名字: 它绑着的另一份目标库的库名}}
        self.own_stem = own_stem        # 这份源码若是一份目标库整份，它的库名；别的是 ""
        self.mode = mode                # 弄坏开关（subscript-key-ignored：按键取容器里的库时不看键；registry-read-unseen、registry-get-ignored、registry-alias-ignored、
                                        # from-import-only-skipped、unrecognized-loads-unlisted、module-attribute-key-unfolded：见文件头开关表）
        self.bound = {}                 # 键 → 种类
        self.containers = {}            # 按下标取的容器（append 进去的列表、写死的数作下标存进去的）的键 → 库名
        self.keyed_stores = {}          # 按字符串键存过库的容器（sys.modules 这类）的键 → {id(存的那一处的键): (键的节点, 库名)}
        self.returns = {}               # 函数名 → 种类（返回的是库）
        self.uses, self.unjudged = [], []
        self.hint_records = []          # ((库名, 函数名), 第几个位置参数或 None, 关键字或 None, 种类)：把库交给另一份目标库里的函数
        # spec_from_file_location 载入时给的名字（spec_load_names 填；它自己折键时 .name 还折不出，所以先放空的）
        self.load_names, self.spec_names, self.unfoldable_spec_stems = {}, {}, set()
        if not hasattr(tree, "_flow_index"):
            tree._flow_index = self.build_index(tree)   # 只跟这棵树有关，几轮之间共用
        (self.nodes, self.local_names, self.global_names, self.nonlocal_names, self.definitions, self.classes, self.constants,
         self.literal_names, self.loader_functions, self.statements, self.imports) = tree._flow_index
        self.aliases = import_aliases(self.imports)
        self.bind_imports()
        self.apply_hints(hints)
        self.unrecognized = self.unrecognized_loads()    # 没绑定、没载入调用的文件也要记：下面的早退不能漏掉它们
        if (not self.bound and not any(self.load_call_kind(node) for node in self.statements if isinstance(node, ast.Call))
                and not self.names_library_without_binding()):
            return    # 没载入任何目标库，也没有谁把库交进来，也没有 from 目标库 import、按库名取 sys.modules 的写法
        self.load_names, self.spec_names, self.unfoldable_spec_stems = self.spec_load_names()
        for _ in range(FLOW_ROUNDS):
            before = self.flow_size()
            for node in self.statements:
                self.flow(node)
            if before == self.flow_size():
                break
        self.collect()

    def flow_size(self):
        return (len(self.bound), len(self.containers), sum(len(stores) for stores in self.keyed_stores.values()), len(self.returns))

    def names_library_without_binding(self):
        """没有绑定、也没有载入调用的文件里，还有没有提到目标库的写法：from <目标库> import 名字（含 *），或往 sys.modules 这类容器按写死的库名取（下标与 .get 两种取法）。
        有就要往下核引到的名字（collect 里逐个收）。弄坏开关：from-import-only-skipped 下不认前一种，registry-read-unseen 下不认后一种。"""
        if self.mode != "from-import-only-skipped":
            for node in self.imports:
                if isinstance(node, ast.ImportFrom) and (node.module or "").rsplit(".", 1)[-1] in self.targets:
                    return True
        if self.mode != "registry-read-unseen":
            for node in self.nodes:
                read = self.keyed_read(node)
                if read is None or (isinstance(node, ast.Subscript) and not isinstance(node.ctx, ast.Load)) or self.registry_name(read[0]) is None:
                    continue
                if any(self.registry_stems(name) for name in self.fold_key(read[1]) or ()):
                    return True
        return False

    def unrecognized_loads(self):
        """文本里点名了目标库、载入与绑定却没认出的写法 → [(行号, 库名, 一句说明)]：不带 as 的带点 import（import a.b.库 绑的是 a，用到的名字没跟）、
        名字以 Loader 收尾的调用（SourceFileLoader(…)）与 runpy 的 run_path、run_module，参数里有库名或库文件名（strings_in_arguments）。
        库名在不在这一次要核的库里由调用的一侧筛。弄坏开关 unrecognized-loads-unlisted：不记。"""
        if self.mode == "unrecognized-loads-unlisted":
            return []
        found = []
        for node in self.nodes:
            if isinstance(node, ast.Import):
                for alias in node.names:
                    stem = alias.name.rsplit(".", 1)[-1]
                    if "." in alias.name and not alias.asname and stem in self.targets:
                        found.append((node.lineno, stem, f"import {alias.name} 不带 as，绑的是 {alias.name.split('.')[0]}，{stem} 的用法没跟"))
            elif isinstance(node, ast.Call):
                name = callee_name(node.func)
                if name is None or not (name in UNRECOGNIZED_LOADER_CALLEES or name.endswith("Loader")):
                    continue
                for text in sorted(self.strings_in_arguments(node)):
                    stem = self.library_named(text)
                    if stem:
                        found.append((node.lineno, stem, f"{name}(…) 载入 {stem}，载入的库的用法没跟"))
                        break
        return found

    def target_imported_as(self, name):
        """from <目标库> import <name>（含 as 之后的名字、from 包.目标库 import …）绑的 name 是从哪份目标库 import 进来的 → 库名；不是交 None。"""
        for node in self.imports:
            if isinstance(node, ast.ImportFrom):
                base = (node.module or "").rsplit(".", 1)[-1]
                if base in self.targets and any((alias.asname or alias.name) == name for alias in node.names):
                    return base
        return None

    def registry_name(self, node):
        """node 是 sys.modules 这类登记着全进程的库的容器（MODULE_REGISTRIES）→ 它的点路径，不是交 None。容器名照 import 的别名认（import sys as s 之后的 s.modules、
        from sys import modules 之后的 modules）。弄坏开关 registry-alias-ignored：别名不认，只认字面写的 sys.modules。"""
        path = dotted_path(node) if self.mode == "registry-alias-ignored" else canonical_path(node, self.aliases)
        return path if path in MODULE_REGISTRIES else None

    def keyed_read(self, node):
        """node 是 <容器>[<键>]（Subscript）或 <容器>.get(<键>[, 缺省])（Call）→ (容器节点, 键节点)，别的交 None。弄坏开关 registry-get-ignored：.get 不认。"""
        if isinstance(node, ast.Subscript):
            return node.value, node.slice
        if (self.mode != "registry-get-ignored" and isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "get"
                and len(node.args) in (1, 2) and not node.keywords and not isinstance(node.args[0], ast.Starred)):
            return node.func.value, node.args[0]
        return None

    def registry_stems(self, name):
        """容器的键 name 对得上哪几份目标库：库名、库文件名、以 /库文件名 结尾的路径，或本文件 spec_from_file_location 载入时给的名字。"""
        return {stem for stem in {self.library_named(name), *self.load_names.get(name, ())} if stem}

    def spec_load_names(self):
        """本文件里 spec_from_file_location 载入时给的名字（第一个参数或 name=，折得出的）→ (
        {名字: {库名，不跟着走的库是 None}}, {库名: {名字}}, 有一处名字折不出的库名集合)。"""
        load_names, spec_names, unfoldable = {}, {}, set()
        for node in self.statements:
            if not (isinstance(node, ast.Call) and callee_name(node.func) == "spec_from_file_location"):
                continue
            argument = node.args[0] if node.args else next((keyword.value for keyword in node.keywords if keyword.arg == "name"), None)
            loaded = self.load_call_kind(node)
            stem = loaded[1] if loaded else None
            folded = self.fold_key(argument) if argument is not None else None
            if folded is None:
                if stem:
                    unfoldable.add(stem)
                continue
            for name in folded:
                load_names.setdefault(name, set()).add(stem)
                if stem:
                    spec_names.setdefault(stem, set()).add(name)
        return load_names, spec_names, unfoldable

    def fold_key(self, node):
        """容器的键 → 折出的字符串集合；折不出交 None。认：写死的字符串；同一层函数里只赋过写死字符串的名字；
        spec_from_file_location 的结果的 .name（这一份库在本文件里载入时给的名字，有一处折不出就整个折不出）；
        X.__module__，X 是 from <目标库> import X 引进来的名字（折成那份目标库的库名，target_imported_as；弄坏开关 module-attribute-key-unfolded 下折不出）。"""
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return frozenset({node.value})
        if isinstance(node, ast.Name):
            return self.literal_names.get(self.resolve(node.id, enclosing_scope(node)))
        if isinstance(node, ast.Attribute) and node.attr == "name":
            kind = self.kind_of(node.value)
            if kind and kind[0] == "spec" and kind[1] not in self.unfoldable_spec_stems and self.spec_names.get(kind[1]):
                return frozenset(self.spec_names[kind[1]])
        if isinstance(node, ast.Attribute) and node.attr == "__module__" and isinstance(node.value, ast.Name) and self.mode != "module-attribute-key-unfolded":
            stem = self.target_imported_as(node.value.id)
            return frozenset({stem}) if stem else None
        return None

    def container_entry(self, node):
        """node 是 <容器>[<键>] 或 <容器>.get(<键>)（取出，keyed_read）→ (种类或 None, 本次未判的一句或 None)；认法在 CALLER_SCAN_DIRECTORIES 前的注释「按键存取的容器」。"""
        read = self.keyed_read(node)
        if read is None or not isinstance(read[0], (ast.Name, ast.Attribute)):
            return None, None
        container_node, index = read
        container = self.key_of(container_node)
        positional = self.containers.get(container)
        stores = self.keyed_stores.get(container)
        if isinstance(index, ast.Constant) and not isinstance(index.value, str):
            return (("module", positional) if positional else None), None
        if not stores:
            return (("module", positional), None) if positional else self.registry_entry(container_node, index)
        keyed, opaque = {}, set()
        for store_key, stem in stores.values():
            folded = self.fold_key(store_key)
            if folded is None:
                opaque.add(stem)
            for name in folded or ():
                keyed.setdefault(name, set()).add(stem)
        where = dotted_path(container_node)
        held = "、".join(sorted({stem for _, stem in stores.values()}))
        folded = self.fold_key(index)
        if folded is None:
            return None, f"从装过 {held} 的 {where} 里按折不出的键取，取出的是哪份库认不出，它的用法没跟"
        stems = set().union(*(keyed[name] for name in folded if name in keyed))
        if not stems:
            if opaque:
                return None, f"从 {where} 里按键 {'、'.join(sorted(folded))} 取，可 {held} 存进去时有的键折不出，取出的是哪份库认不出，它的用法没跟"
            unknown = sorted(name for name in folded if name not in self.load_names)
            if unknown:
                return None, f"从装过 {held} 的 {where} 里按键 {'、'.join(unknown)} 取，本文件里哪份库存进去或载入时都不叫这个名字，取出的是什么认不出，它的用法没跟"
            stems = {stem for name in folded for stem in self.load_names[name] if stem}
            if not stems:
                return None, None    # 别的库（不跟着走的）载入时用的名字：不是被核的库
        if len(stems) > 1:
            return None, f"从 {where} 里按键 {'、'.join(sorted(folded))} 取，对得上 {'、'.join(sorted(stems))} 不止一份库，它的用法没跟"
        return ("module", next(iter(stems))), None

    def registry_entry(self, container_node, key_node):
        """从 container_node 里按 key_node 取出（下标或 .get），而本文件没按键存过这个容器 → (种类或 None, 本次未判的一句或 None)。只看 sys.modules 这类登记着全进程的库的容器
        （MODULE_REGISTRIES，registry_name 认别名）：键折得出而对得上某份目标库（registry_stems）的，取出的就是那份库；折得出而哪份库都对不上的不是被核的库；
        折不出的列成本次未判（键是 __name__ 的是调用方自己，不算）。弄坏开关 registry-read-unseen：这种取法一概不认。"""
        where = self.registry_name(container_node)
        if self.mode == "registry-read-unseen" or where is None:
            return None, None
        if isinstance(key_node, ast.Name) and key_node.id == "__name__":
            return None, None
        folded = self.fold_key(key_node)
        if folded is None:
            return None, f"从 {where} 里按折不出的键取，取出的可能是被核的库，它的用法没跟"
        stems = set().union(*(self.registry_stems(name) for name in folded))
        if len(stems) > 1:
            return None, f"从 {where} 里按键 {'、'.join(sorted(folded))} 取，对得上 {'、'.join(sorted(stems))} 不止一份库，它的用法没跟"
        return (("module", next(iter(stems))), None) if stems else (None, None)

    @classmethod
    def build_index(cls, tree):
        """这棵树里名字归哪个函数、有哪些函数与类、字符串常量赋给了谁、哪些函数里起了载入的调用。"""
        nodes = list(ast.walk(tree))
        local_names, global_names, nonlocal_names, definitions, classes, constants, imports = {}, {}, {}, {}, {}, {}, []
        for node in nodes:
            scope = enclosing_scope(node)
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
                local_names.setdefault(scope, set()).add(node.id)
            elif isinstance(node, ast.arg):
                local_names.setdefault(scope, set()).add(node.arg)
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                local_names.setdefault(scope, set()).add(node.name)
                if isinstance(node, ast.ClassDef):
                    classes.setdefault(node.name, []).append(node)
                else:
                    definitions.setdefault(node.name, []).append(node)
            elif isinstance(node, ast.Global):
                global_names.setdefault(scope, set()).update(node.names)
            elif isinstance(node, ast.Nonlocal):
                nonlocal_names.setdefault(scope, set()).update(node.names)
            elif isinstance(node, (ast.Import, ast.ImportFrom)):
                imports.append(node)
                local_names.setdefault(scope, set()).update(cls.imported_names(node))
            elif isinstance(node, ast.ExceptHandler) and node.name:
                local_names.setdefault(scope, set()).add(node.name)
        owner = cls.__new__(cls)    # 只借 resolve 用：它读这三张表
        owner.local_names, owner.global_names, owner.nonlocal_names = local_names, global_names, nonlocal_names
        for node in nodes:
            if isinstance(node, ast.Assign):
                texts = [inner.value for inner in ast.walk(node.value) if isinstance(inner, ast.Constant) and isinstance(inner.value, str)]
                for target in node.targets:
                    if texts and isinstance(target, ast.Name):
                        constants.setdefault(owner.resolve(target.id, enclosing_scope(node)), set()).update(texts)
        literals, rebound = {}, set()    # 名字 → 赋过的写死字符串；rebound：还有别的赋法（形参、循环变量、非字面的赋值……）的名字，折不出
        for node in nodes:
            if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store):
                key = owner.resolve(node.id, enclosing_scope(node))
                parent = getattr(node, "_flow_parent", None)
                if isinstance(parent, ast.Assign) and node in parent.targets and isinstance(parent.value, ast.Constant) and isinstance(parent.value.value, str):
                    literals.setdefault(key, set()).add(parent.value.value)
                else:
                    rebound.add(key)
            elif isinstance(node, ast.arg):
                rebound.add(owner.resolve(node.arg, enclosing_scope(node)))
        literal_names = {key: frozenset(values) for key, values in literals.items() if key not in rebound}
        loader_functions = set()    # 函数体里（含里层的函数）起了载入的调用的函数名
        for node in nodes:
            if isinstance(node, ast.Call) and callee_name(node.func) in LOADER_BODY_CALLEES:
                function = enclosing_scope(node)
                while function is not None:
                    if isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        loader_functions.add(function.name)
                    function = enclosing_scope(function)
        statements = [node for node in nodes if isinstance(node, (ast.Assign, ast.AnnAssign, ast.NamedExpr, ast.Return, ast.Call))]
        return nodes, local_names, global_names, nonlocal_names, definitions, classes, constants, literal_names, loader_functions, statements, imports

    @staticmethod
    def imported_names(node):
        if isinstance(node, ast.Import):
            return [alias.asname or alias.name.split(".")[0] for alias in node.names]
        return [alias.asname or alias.name for alias in node.names if alias.name != "*"]

    def resolve(self, identifier, scope):
        current = scope
        while current is not None:
            if identifier in self.global_names.get(current, ()):
                return (None, identifier)
            if identifier not in self.nonlocal_names.get(current, ()) and identifier in self.local_names.get(current, ()):
                return (current, identifier)
            current = enclosing_scope(current)
        return (None, identifier)

    def key_of(self, node):
        if isinstance(node, ast.Name):
            return ("name", self.resolve(node.id, enclosing_scope(node)))
        path = dotted_path(node) if isinstance(node, ast.Attribute) else None
        return ("path", path) if path else None

    def library_named(self, text):
        """字符串常量 → 库名：库名本身（lib_x）、库文件名（lib_x.py）、以 /库文件名 结尾的路径。"""
        for stem, file_name in self.targets.items():
            if text == stem or text == file_name or text.endswith("/" + file_name):
                return stem
        return None

    def strings_in_arguments(self, call):
        found = set()
        scope = enclosing_scope(call)
        for argument in list(call.args) + [keyword.value for keyword in call.keywords]:
            for inner in ast.walk(argument):
                if isinstance(inner, ast.Constant) and isinstance(inner.value, str):
                    found.add(inner.value)
                elif isinstance(inner, ast.Name):
                    found.update(self.constants.get(self.resolve(inner.id, scope), ()))
        return found

    def load_call_kind(self, call):
        name = callee_name(call.func)
        if name is None or name == "module_from_spec":
            return None
        if not ((LOADER_CALLEE.search(name) and (name[0].islower() or name[0] == "_")) or name in self.loader_functions):
            return None
        for text in sorted(self.strings_in_arguments(call)):
            stem = self.library_named(text)
            if stem:
                return ("spec" if "spec_from" in name else "module", stem)
        return None

    def kind_of(self, node):
        if isinstance(node, ast.Call):
            loaded = self.load_call_kind(node)
            if loaded:
                return loaded
            name = callee_name(node.func)
            if name == "module_from_spec" and node.args:
                inner = self.kind_of(node.args[0])
                return ("module", inner[1]) if inner and inner[0] == "spec" else None
            if self.keyed_read(node) is not None:
                entry = self.container_entry(node)[0]
                if entry:
                    return entry
            return self.returns.get(name) if name else None
        if isinstance(node, (ast.Name, ast.Attribute)):
            key = self.key_of(node)
            if key in self.bound:
                return self.bound[key]
            if isinstance(node, ast.Attribute):
                inner = self.kind_of(node.value)
                if inner and inner[0] == "module":
                    exported = self.exports.get(inner[1], {}).get(node.attr)
                    if exported:
                        return ("module", exported)
            return None
        if isinstance(node, ast.Subscript):
            return self.container_entry(node)[0]
        if isinstance(node, ast.IfExp):
            return self.kind_of(node.body) or self.kind_of(node.orelse)
        if isinstance(node, ast.BoolOp):
            for value in node.values:
                kind = self.kind_of(value)
                if kind:
                    return kind
            return None
        if isinstance(node, ast.NamedExpr):
            return self.kind_of(node.value)
        return None

    def bind(self, key, kind):
        if key is not None and kind is not None and key not in self.bound:
            self.bound[key] = kind

    def bind_imports(self):
        for node in self.imports:
            scope = enclosing_scope(node)
            if isinstance(node, ast.Import):
                for alias in node.names:
                    if alias.name.rsplit(".", 1)[-1] in self.targets and (alias.asname or "." not in alias.name):
                        self.bind(("name", self.resolve(alias.asname or alias.name, scope)), ("module", alias.name.rsplit(".", 1)[-1]))
            elif isinstance(node, ast.ImportFrom):
                base = (node.module or "").rsplit(".", 1)[-1]
                for alias in node.names:
                    if base not in self.targets and alias.name in self.targets:
                        self.bind(("name", self.resolve(alias.asname or alias.name, scope)), ("module", alias.name))

    def apply_hints(self, hints):
        """别的文件把库交给了这份目标库的模块层函数：对应的形参在这里当库。"""
        if not self.own_stem:
            return
        for name, definitions in self.definitions.items():
            for position, keyword, kind in hints.get((self.own_stem, name), ()):
                for definition in definitions:
                    if not isinstance(getattr(definition, "_flow_parent", None), ast.Module):
                        continue
                    parameters = [argument.arg for argument in definition.args.posonlyargs + definition.args.args]
                    chosen = parameters[position] if position is not None and position < len(parameters) else keyword
                    if chosen in parameters or chosen in {argument.arg for argument in definition.args.kwonlyargs}:
                        self.bind(("name", (definition, chosen)), kind)

    def bind_target(self, target, value):
        if isinstance(target, (ast.Tuple, ast.List)):
            if isinstance(value, (ast.Tuple, ast.List)) and len(value.elts) == len(target.elts) and not any(isinstance(item, ast.Starred) for item in target.elts + value.elts):
                for inner_target, inner_value in zip(target.elts, value.elts):
                    self.bind_target(inner_target, inner_value)
            return
        kind = self.kind_of(value)
        if kind is None:
            return
        if isinstance(target, (ast.Name, ast.Attribute)):
            self.bind(self.key_of(target), kind)
        elif isinstance(target, ast.Subscript) and kind[0] == "module" and isinstance(target.value, (ast.Name, ast.Attribute)):
            container = self.key_of(target.value)
            if container is None:
                return
            index = target.slice
            if self.mode == "subscript-key-ignored" or (isinstance(index, ast.Constant) and not isinstance(index.value, str)):
                self.containers.setdefault(container, kind[1])
            else:
                self.keyed_stores.setdefault(container, {}).setdefault(id(index), (index, kind[1]))

    def flow(self, node):
        if isinstance(node, ast.Assign):
            for target in node.targets:
                self.bind_target(target, node.value)
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            self.bind_target(node.target, node.value)
        elif isinstance(node, ast.NamedExpr):
            self.bind_target(node.target, node.value)
        elif isinstance(node, ast.Return) and node.value is not None:
            function = enclosing_scope(node)
            kind = self.kind_of(node.value)
            if kind and kind[0] == "module" and isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.returns.setdefault(function.name, kind)
        elif isinstance(node, ast.Call):
            self.flow_call(node)

    @staticmethod
    def globals_assignment(call):
        """globals().__setitem__("名字", 值) → 名字（写死的字符串）；不是这种写法交 None。"""
        function = call.func
        if (isinstance(function, ast.Attribute) and function.attr == "__setitem__" and len(call.args) == 2 and isinstance(function.value, ast.Call)
                and callee_name(function.value.func) == "globals" and isinstance(call.args[0], ast.Constant) and isinstance(call.args[0].value, str)):
            return call.args[0].value
        return None

    def flow_call(self, call):
        name = callee_name(call.func)
        if name is None:
            return
        assigned = self.globals_assignment(call)
        if assigned is not None:
            kind = self.kind_of(call.args[1])
            if kind and kind[0] == "module":
                self.bind(("name", (None, assigned)), kind)
        if name in CONTAINER_METHODS and isinstance(call.func, ast.Attribute):
            container = self.key_of(call.func.value)
            if container is not None:
                for argument in call.args:
                    kind = self.kind_of(argument)
                    if kind and kind[0] == "module":
                        self.containers.setdefault(container, kind[1])
        for definition in self.definitions.get(name, ()):
            self.flow_arguments(call, definition, skip_first=isinstance(getattr(definition, "_flow_parent", None), ast.ClassDef) and isinstance(call.func, ast.Attribute))
        for class_definition in self.classes.get(name, ()):
            for member in class_definition.body:
                if isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef)) and member.name == "__init__":
                    self.flow_arguments(call, member, skip_first=True)

    def flow_arguments(self, call, definition, skip_first):
        parameters = [argument.arg for argument in definition.args.posonlyargs + definition.args.args]
        if skip_first:
            parameters = parameters[1:]
        for position, argument in enumerate(call.args):
            if isinstance(argument, ast.Starred):
                break
            if position < len(parameters):
                kind = self.kind_of(argument)
                if kind and kind[0] == "module":
                    self.bind(("name", (definition, parameters[position])), kind)
        known = set(parameters) | {argument.arg for argument in definition.args.kwonlyargs}
        for keyword in call.keywords:
            if keyword.arg in known:
                kind = self.kind_of(keyword.value)
                if kind and kind[0] == "module":
                    self.bind(("name", (definition, keyword.arg)), kind)

    def context_is_known(self, node):
        """载入的库出现在这个位置，接下来怎么用能跟得上（属性、赋值、返回、比较、交给本文件的函数……），跟不上的是未判。"""
        parent = getattr(node, "_flow_parent", None)
        if isinstance(parent, ast.Attribute):
            return parent.value is node
        if isinstance(parent, (ast.Assign, ast.AnnAssign, ast.NamedExpr)):
            return parent.value is node
        if isinstance(parent, (ast.Tuple, ast.List)):
            grandparent = getattr(parent, "_flow_parent", None)
            return isinstance(grandparent, ast.Assign) and grandparent.value is parent and any(isinstance(target, (ast.Tuple, ast.List)) for target in grandparent.targets)
        if isinstance(parent, (ast.Return, ast.Compare, ast.BoolOp, ast.IfExp, ast.Expr, ast.Assert, ast.Delete, ast.FormattedValue)):
            return True
        if isinstance(parent, ast.UnaryOp):
            return isinstance(parent.op, ast.Not)
        if isinstance(parent, (ast.If, ast.While)):
            return parent.test is node
        if isinstance(parent, ast.keyword):
            call = getattr(parent, "_flow_parent", None)
            return isinstance(call, ast.Call) and self.call_takes_library(call)
        if isinstance(parent, ast.Call) and parent.func is not node:
            return self.call_takes_library(parent)
        return False

    def call_takes_library(self, call):
        name = callee_name(call.func)
        if name is None:
            return False
        if name in HARMLESS_CALLEES or name in self.definitions or name in self.classes or self.globals_assignment(call) is not None:
            return True
        if isinstance(call.func, ast.Attribute):
            receiver = self.kind_of(call.func.value)
            if receiver and receiver[0] == "module":
                return True   # 另一份目标库里的函数：形参在那份库里接着当库（hint_records）
            return name in CONTAINER_METHODS and self.key_of(call.func.value) in self.containers
        return False

    def collect(self):
        kinds = {}

        def kind_cached(node):
            if id(node) not in kinds:
                kinds[id(node)] = self.kind_of(node)
            return kinds[id(node)]

        for node in self.nodes:
            if isinstance(node, ast.ImportFrom):
                base = (node.module or "").rsplit(".", 1)[-1]
                if base in self.targets:
                    for alias in node.names:
                        if alias.name == "*":
                            self.unjudged.append((node.lineno, f"from {base} import *：引到哪些名字认不出"))
                        else:
                            self.uses.append((node.lineno, base, alias.name))
            if (isinstance(node, ast.Call) or (isinstance(node, ast.Subscript) and isinstance(node.ctx, ast.Load))) and self.keyed_read(node) is not None:
                entry_unjudged = self.container_entry(node)[1]
                if entry_unjudged:
                    self.unjudged.append((node.lineno, entry_unjudged))
            if isinstance(node, ast.Attribute) and isinstance(node.ctx, ast.Load):
                kind = kind_cached(node.value)
                if kind and kind[0] == "module":
                    self.uses.append((node.lineno, kind[1], node.attr))
            elif isinstance(node, ast.Call):
                if (callee_name(node.func) == "getattr" and len(node.args) == 2 and isinstance(node.args[1], ast.Constant) and isinstance(node.args[1].value, str)):
                    kind = kind_cached(node.args[0])
                    if kind and kind[0] == "module":
                        self.uses.append((node.lineno, kind[1], node.args[1].value))
                if isinstance(node.func, ast.Attribute):
                    receiver = kind_cached(node.func.value)
                    if receiver and receiver[0] == "module":
                        for position, argument in enumerate(node.args):
                            kind = kind_cached(argument)
                            if kind and kind[0] == "module":
                                self.hint_records.append(((receiver[1], node.func.attr), position, None, kind))
                        for keyword in node.keywords:
                            kind = kind_cached(keyword.value)
                            if keyword.arg and kind and kind[0] == "module":
                                self.hint_records.append(((receiver[1], node.func.attr), None, keyword.arg, kind))
            if isinstance(node, (ast.Name, ast.Attribute, ast.Call, ast.Subscript, ast.IfExp, ast.BoolOp, ast.NamedExpr)) and not isinstance(getattr(node, "ctx", None), (ast.Store, ast.Del)):
                kind = kind_cached(node)
                if kind and kind[0] == "module" and not self.context_is_known(node):
                    parent = getattr(node, "_flow_parent", None)
                    self.unjudged.append((node.lineno, f"载入的 {kind[1]} 出现在认不出去向的位置（{type(parent).__name__}"
                                                       f"{'，交给 ' + str(callee_name(parent.func)) if isinstance(parent, ast.Call) else ''}），它的用法没跟"))


def module_level_imports(statements):
    """语句表里属于模块层的 import：顶层的，加上 if / try / with / for / while / match 里的（def 与 class 里面的不算）。"""
    for statement in statements:
        if isinstance(statement, (ast.Import, ast.ImportFrom)):
            yield statement
        elif not isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            for field in ("body", "orelse", "finalbody"):
                yield from module_level_imports(getattr(statement, field, None) or ())
            for handler in getattr(statement, "handlers", None) or ():
                yield from module_level_imports(handler.body)
            for case in getattr(statement, "cases", None) or ():
                yield from module_level_imports(case.body)


class SearchPaths:
    """一份调用方源码运行时怎么找库：往 sys.path 放了哪些目录、按路径载入了哪些文件。认法在 CALLER_SCAN_DIRECTORIES 前的注释「同名的库」。
    路径要折得出才算数（fold）；折不出的往 sys.path 放（unfolded_path）与按路径载入（unfolded_loads）只记下有这么一回事，library_file 遇到就认不出是哪一份。
    modules 是这一批读进来的源码（ModuleSources）：模块级 import 进来的仓内模块往 sys.path 放的目录（imported_reach）与它们的模块级常量（fold）从那里取。
    名字的赋值按作用域找（name_values）：从用它的那个函数往外一层层看，第一层绑了这个名字的就在那一层取。
    往 sys.path 放的目录折不出一处时，directory_choices 再试两种半折：几种可能（名字在一个函数里赋了几次、本文件里不带参数的函数有几处返回），
    与前半截折不出、末尾几段写死（os.path.join(<形参>, "scripts", "x")）；这两种记进 vague，library_file 只在仓里叫这个名字的文件落在其中一种里时才认不出。
    mode 是弄坏开关：imported-sys-path-ignored 下 import 进来的模块放的目录不算；imported-constants-unfolded 下 X.NAME 折不出；environment-default-unfolded 下
    <读环境变量> or <缺省> 折不出；function-local-names-ignored 下名字只在模块层找；several-choices-unfolded 下几种可能的目录折不出；known-tail-unfolded 下
    只知道末尾几段的目录折不出；vague-directories-ignored 下 vague 里的目录一概不挡库的解析。"""

    def __init__(self, root, unit, modules, mode=""):
        self.root = root
        self.relative = unit.relative
        self.modules, self.mode = modules, mode
        self.file_path = os.path.join(root, unit.relative) if unit.relative.endswith(".py") else None   # __file__；.sh 里的 heredoc 没有
        self.own = os.path.dirname(unit.relative) if self.file_path else None                          # 调用方自己所在的目录（相对仓根）
        self.tree = unit.tree
        self.aliases = import_aliases([node for node in ast.walk(unit.tree) if isinstance(node, (ast.Import, ast.ImportFrom))])
        self.scopes = {}                                              # 函数（模块层是 None）→ scope_binding 的结果
        self.functions = self.module_functions()
        self.requests = self.import_requests()
        self.front, self.back, self.unfolded_path = [], [], False    # 往 sys.path 最前、后面放的目录（相对仓根，先找的在前）；有一处折不出
        self.vague = []                                               # 往 sys.path 放的目录里半折的：(几种可能的相对仓根的目录, 只知道的末尾几段)
        self.loaded, self.unfolded_loads = {}, set()                  # 库名 → 按路径载入时折出的文件（相对仓根）；按路径载入而路径折不出的库名
        self.reached = None                                           # imported_reach 的结果（算过一次就记下）
        self.scan(unit.tree)

    def scope_binding(self, scope):
        """scope（一个函数，模块层是 None）里的名字绑定 → (名字 → 在这一层赋的每一处的右边, 在这一层有别的绑定的名字)；嵌在里面的函数里的赋值不算这一层的。"""
        if scope not in self.scopes:
            found, spoiled = assignment_scan(self.tree if scope is None else scope)
            here = {name: [value for value in values if enclosing_scope(value) is scope] for name, values in found.items()}
            spoiled |= {name for name, values in here.items() if not values}
            self.scopes[scope] = ({name: values for name, values in here.items() if values}, spoiled)
        return self.scopes[scope]

    def name_values(self, node):
        """名字 node 在它的作用域里赋过的每一处的右边：从用它的那个函数往外一层层看，第一层绑了这个名字的就在那一层取，那一层里还有别的绑定的交 None；
        一层都没绑的看模块层。弄坏开关 function-local-names-ignored：只看模块层。"""
        scope = None if self.mode == "function-local-names-ignored" else enclosing_scope(node)
        while True:
            values, spoiled = self.scope_binding(scope)
            if node.id in spoiled:
                return None
            if node.id in values:
                return values[node.id]
            if scope is None:
                return None
            scope = enclosing_scope(scope)

    def name_value(self, node):
        """名字 node 在它的作用域里只赋过一次的那一处的右边；赋了几次或别的绑定交 None。"""
        values = self.name_values(node)
        return values[0] if values is not None and len(values) == 1 else None

    def module_constant(self, name):
        """模块层只赋过一次、没有别的绑定的名字 → 它赋的右边；不是这样的交 None。"""
        values, spoiled = self.scope_binding(None)
        found = values.get(name, ())
        return found[0] if len(found) == 1 and name not in spoiled else None

    def module_functions(self):
        """模块层只用 def 绑过一次、没有别的绑定的函数名 → 它的 def。"""
        found = {}
        for statement in self.tree.body:
            if isinstance(statement, ast.FunctionDef):
                found.setdefault(statement.name, []).append(statement)
        assigned = self.scope_binding(None)[0]
        return {name: defs[0] for name, defs in found.items() if len(defs) == 1 and name not in assigned and name not in self.aliases}

    def import_requests(self):
        """模块层的 import 要找的模块 → [(绑的名字或 None, 模块名拆成的段, 相对导入往上数的层数)]：import a.b 不带 as 绑的是 a，不当模块名；
        from a import b 绑的 b 不一定是模块（None）；from . import b 的 b 当模块找。"""
        requests = []
        for node in module_level_imports(self.tree.body):
            if isinstance(node, ast.Import):
                requests += [(alias.asname or (alias.name if "." not in alias.name else None), alias.name.split("."), 0) for alias in node.names]
            elif node.module:
                requests.append((None, node.module.split("."), node.level))
            else:
                requests += [(alias.asname or alias.name, [alias.name], node.level) for alias in node.names if alias.name != "*"]
        return requests

    def resolve_module(self, parts, level, directories):
        """import 的模块（名字拆成的段）→ 相对仓根的 .py（读进来的源码里有的，包就是 __init__.py）；找不到交 None。
        level 是相对导入往上数的层数，写了的只看调用方自己所在的目录往上数那一层；绝对导入（0）按 directories 的先后找。"""
        if level:
            if self.own is None:
                return None
            base = self.own
            for _ in range(level - 1):
                base = os.path.dirname(base)
            directories = [base]
        for directory in directories:
            for candidate in (os.path.join(directory, *parts) + ".py", os.path.join(directory, *parts, "__init__.py")):
                candidate = os.path.normpath(candidate)
                if candidate in self.modules.texts:
                    return candidate
        return None

    def imported_module(self, name):
        """模块层 import 绑在 name 上的仓内模块（在调用方自己所在的目录里找）→ 它的 SearchPaths；name 不是这样绑的、找不到、读不了交 None。"""
        for bound, parts, level in self.requests:
            if bound == name:
                file = self.resolve_module(parts, level, [self.own] if self.own is not None else [])
                return self.modules.search(file) if file else None
        return None

    def imported_reach(self):
        """模块层 import 进来的仓内模块往 sys.path 放的目录（它们 import 的也算，循环 import 只走一遍）→ (放在最前的, 放在后面的, 半折的目录, 放的目录有折不出的那几份模块)。
        这些模块按调用方自己放在最前的、自己所在的目录、自己放在后面的目录依次找（resolve_module）。弄坏开关 imported-sys-path-ignored：交空的。"""
        if self.reached is None:
            front, back, vague, opaque = [], [], [], []
            seen, pending = {self.relative}, []
            directories = [*self.front, *([self.own] if self.own is not None else []), *self.back]
            if self.mode != "imported-sys-path-ignored":
                pending = [self.resolve_module(parts, level, directories) for _, parts, level in self.requests]
            while pending:
                relative = pending.pop(0)
                other = self.modules.search(relative) if relative and relative not in seen else None
                if other is None:
                    continue
                seen.add(relative)
                front += [directory for directory in other.front if directory not in front]
                back += [directory for directory in other.back if directory not in back]
                vague += [group for group in other.vague if group not in vague]
                if other.unfolded_path:
                    opaque.append(relative)
                other_directories = [*other.front, *([other.own] if other.own is not None else []), *other.back]
                pending += [other.resolve_module(parts, level, other_directories) for _, parts, level in other.requests]
            self.reached = (front, back, vague, opaque)
        return self.reached

    @staticmethod
    def normalized(path):
        return os.path.normpath(path) if os.path.isabs(path) else path

    def environment_read(self, node):
        """node 是读环境变量：os.environ.get(…)、os.getenv(…)、os.environ[…]。"""
        if isinstance(node, ast.Call):
            return canonical_path(node.func, self.aliases) in ENVIRONMENT_READ_CALLEES
        return isinstance(node, ast.Subscript) and canonical_path(node.value, self.aliases) == "os.environ"

    def fold(self, node, depth=0):
        """路径表达式 → 折出的路径串（绝对路径已归一）；折不出交 None。认：写死的字符串、__file__、只赋过一次的名字、os.path.dirname / join / realpath / abspath /
        normpath、pathlib 的 Path、/、.parent、.parents[n]、.resolve()、.joinpath(…)；X.NAME（X 是模块层 import 进来的仓内模块，按那份模块里 NAME 唯一一次的赋值折）；
        <读环境变量> or <缺省>（读环境变量的那一边丢掉，剩下恰好一边，照它折：环境变量没设时的取值）。"""
        if node is None or depth > PATH_FOLD_DEPTH:
            return None
        if isinstance(node, ast.Constant):
            return node.value if isinstance(node.value, str) else None
        if isinstance(node, ast.Name):
            return self.file_path if node.id == "__file__" else self.fold(self.name_value(node), depth + 1)
        if isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and self.mode != "imported-constants-unfolded":
            other = self.imported_module(node.value.id)
            if other is not None:
                return other.fold(other.module_constant(node.attr), depth + 1)
        if isinstance(node, ast.BoolOp) and isinstance(node.op, ast.Or) and self.mode != "environment-default-unfolded":
            rest = [operand for operand in node.values if not self.environment_read(operand)]
            return self.fold(rest[0], depth + 1) if len(rest) == 1 and len(rest) < len(node.values) else None
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            return self.joined([node.left, node.right], depth)
        if isinstance(node, ast.Attribute) and node.attr == "parent":
            inner = self.fold(node.value, depth + 1)
            return None if inner is None else os.path.dirname(self.normalized(inner))
        if (isinstance(node, ast.Subscript) and isinstance(node.value, ast.Attribute) and node.value.attr == "parents"
                and isinstance(node.slice, ast.Constant) and isinstance(node.slice.value, int) and node.slice.value >= 0):
            inner = self.fold(node.value.value, depth + 1)
            if inner is None:
                return None
            inner = self.normalized(inner)
            for _ in range(node.slice.value + 1):
                inner = os.path.dirname(inner)
            return inner
        if isinstance(node, ast.Call):
            return self.fold_call(node, depth)
        return None

    def joined(self, nodes, depth):
        parts = [self.fold(inner, depth + 1) for inner in nodes]
        return None if not parts or any(part is None for part in parts) else self.normalized(os.path.join(*parts))

    def fold_call(self, call, depth):
        name = canonical_path(call.func, self.aliases)
        if name in PATH_JOIN_CALLEES:
            return self.joined(call.args, depth) if call.args and not call.keywords else None
        if name == "os.path.dirname" and len(call.args) == 1:
            inner = self.fold(call.args[0], depth + 1)
            return None if inner is None else os.path.dirname(self.normalized(inner))
        if name in PATH_PASS_THROUGH_CALLEES and len(call.args) == 1:
            inner = self.fold(call.args[0], depth + 1)
            return None if inner is None else self.normalized(inner)
        if isinstance(call.func, ast.Attribute) and not call.keywords:
            if call.func.attr in PATH_IDENTITY_METHODS and not call.args:
                return self.fold(call.func.value, depth + 1)
            if call.func.attr == "joinpath":
                return self.joined([call.func.value, *call.args], depth)
        return None

    def directory_choices(self, node, depth=0):
        """往 sys.path 放的目录的表达式 → 它可能取的目录 [(种类, 值)]：("exact", 相对仓根的目录；折得出而落在仓根外的是 None) 是折出来的一处，
        ("tail", 末尾几段) 是前半截折不出、末尾几段写死的（os.path.join(<形参>, "scripts", "x") 的 scripts/x）；有一种取法认不出交 None。
        折不出时依次试：名字在一个函数里赋的每一处（name_values）、本文件里不带参数的函数的每一处返回（None 不算）、原样交回的路径函数（normpath 这类）里的那个、
        join 的末尾几段。弄坏开关 several-choices-unfolded：名字赋了几处与函数返回几处的不认；known-tail-unfolded：末尾几段的不认。"""
        if depth > PATH_FOLD_DEPTH:
            return None
        folded = self.fold(node)
        if folded is not None:
            if not os.path.isabs(folded):
                return None
            relative = os.path.relpath(os.path.normpath(folded), self.root)
            return [("exact", None if relative == ".." or relative.startswith(".." + os.sep) else relative)]
        alternatives = self.alternatives(node)
        if alternatives is not None:
            choices = []
            for alternative in alternatives:
                inner = self.directory_choices(alternative, depth + 1)
                if inner is None:
                    return None
                choices += inner
            return choices
        tail = self.known_tail(node)
        return None if tail is None else [("tail", tail)]

    def alternatives(self, node):
        """node 可能是哪几个表达式 → 表达式表；不是名字、本文件里的函数调用、原样交回的路径函数，或认不出交 None。"""
        if isinstance(node, ast.Name):
            values = self.name_values(node)
            return values if values is not None and (len(values) == 1 or self.mode != "several-choices-unfolded") else None
        if not isinstance(node, ast.Call) or node.keywords:
            return None
        if isinstance(node.func, ast.Name) and not node.args and node.func.id in self.functions and self.mode != "several-choices-unfolded":
            return self.returned_values(self.functions[node.func.id]) or None
        if len(node.args) == 1 and canonical_path(node.func, self.aliases) in PATH_PASS_THROUGH_CALLEES:
            return [node.args[0]]
        return None

    @staticmethod
    def returned_values(function):
        """函数自己的 return 的值（嵌在里面的函数不算；不带值与 return None 不算）。"""
        found, pending = [], list(function.body)
        while pending:
            node = pending.pop()
            if isinstance(node, ast.Return):
                if node.value is not None and not (isinstance(node.value, ast.Constant) and node.value.value is None):
                    found.append(node.value)
            elif not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)):
                pending.extend(ast.iter_child_nodes(node))
        return found

    def join_operands(self, node):
        """os.path.join(a, b, …)、Path(a, b)、a / b 拼出来的路径 → 操作数表；不是这样拼的交 None。"""
        if isinstance(node, ast.Call) and node.args and not node.keywords and canonical_path(node.func, self.aliases) in PATH_JOIN_CALLEES:
            return list(node.args)
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div):
            return [*(self.join_operands(node.left) or [node.left]), node.right]
        return None

    def known_tail(self, node):
        """拼出来的路径里，从后往前连着折得出的几个操作数拼成的末尾几段（前面的操作数折不出）；不是拼的、一个也折不出、末尾几段是绝对路径或空的交 None。"""
        operands = self.join_operands(node)
        if operands is None or self.mode == "known-tail-unfolded":
            return None
        pieces = []
        for operand in reversed(operands):
            folded = self.fold(operand)
            if folded is None:
                break
            pieces.insert(0, folded)
        if not pieces or len(pieces) == len(operands) or any(os.path.isabs(piece) for piece in pieces):
            return None
        components = [part for part in os.path.normpath(os.path.join(*pieces)).split(os.sep) if part not in ("", ".")]
        while components and components[0] == "..":
            components.pop(0)
        return os.sep.join(components) or None

    @staticmethod
    def list_elements(node):
        """[a, b]、(a, b) → 元素表；不是这种写法（含带星号的）交 None。"""
        if isinstance(node, (ast.List, ast.Tuple)) and not any(isinstance(element, ast.Starred) for element in node.elts):
            return list(node.elts)
        return None

    def sys_path_events(self, node):
        """node 是往 sys.path 放目录的写法 → [(位置, 目录表达式表)]：位置 "front"（最前）、"back"（后面）、None（放在哪认不出）；表是 None 表示放的什么认不出；不是交空表。
        认：sys.path.insert(<数>, 目录)、append、extend([…])；sys.path[:0] = […]；sys.path = […] + sys.path 与 sys.path + […]；sys.path += […]。别的改写 sys.path 的写法算认不出。"""
        if isinstance(node, ast.Call):
            name = canonical_path(node.func, self.aliases)
            if name == "sys.path.insert" and len(node.args) == 2 and not node.keywords:
                index = node.args[0]
                position = ("front" if index.value == 0 else "back") if isinstance(index, ast.Constant) and isinstance(index.value, int) else None
                return [(position, [node.args[1]])]
            if name == "sys.path.append" and len(node.args) == 1 and not node.keywords:
                return [("back", [node.args[0]])]
            if name == "sys.path.extend" and len(node.args) == 1 and not node.keywords:
                return [("back", self.list_elements(node.args[0]))]
            return [(None, None)] if name in ("sys.path.insert", "sys.path.append", "sys.path.extend") else []
        if isinstance(node, ast.AugAssign) and canonical_path(node.target, self.aliases) == "sys.path":
            return [("back", self.list_elements(node.value)) if isinstance(node.op, ast.Add) else (None, None)]
        if isinstance(node, ast.Assign):
            events = []
            for target in node.targets:
                if isinstance(target, ast.Subscript) and canonical_path(target.value, self.aliases) == "sys.path":
                    bounds = target.slice
                    front = (isinstance(bounds, ast.Slice) and bounds.lower is None and bounds.step is None
                             and isinstance(bounds.upper, ast.Constant) and bounds.upper.value == 0)
                    events.append(("front", self.list_elements(node.value)) if front else (None, None))
                elif canonical_path(target, self.aliases) == "sys.path":
                    value = node.value
                    if isinstance(value, ast.BinOp) and isinstance(value.op, ast.Add) and canonical_path(value.right, self.aliases) == "sys.path":
                        events.append(("front", self.list_elements(value.left)))
                    elif isinstance(value, ast.BinOp) and isinstance(value.op, ast.Add) and canonical_path(value.left, self.aliases) == "sys.path":
                        events.append(("back", self.list_elements(value.right)))
                    else:
                        events.append((None, None))
            return events
        return []

    def scan(self, tree):
        """tree 里往 sys.path 放目录的写法与按路径载入的调用，按它们在源码里的先后记下来。"""
        events = []
        for node in ast.walk(tree):
            for position, values in self.sys_path_events(node):
                events.append((node.lineno, node.col_offset, position, values))
            if isinstance(node, ast.Call):
                self.note_load(node)
        for _, _, position, values in sorted(events, key=lambda event: event[:2]):
            if position is None or values is None:
                self.unfolded_path = True
                continue
            block = []
            for value in values:
                choices = self.directory_choices(value)
                choices = None if choices is None else list(dict.fromkeys(choices))
                if choices is None:
                    self.unfolded_path = True
                elif len(choices) == 1 and choices[0][0] == "exact":
                    if choices[0][1] is not None:
                        block.append(choices[0][1])
                else:
                    group = (tuple(sorted({found for kind, found in choices if kind == "exact" and found is not None})),
                             tuple(sorted({found for kind, found in choices if kind == "tail"})))
                    if group not in self.vague:
                        self.vague.append(group)
            if position == "front":
                self.front[:0] = block
            else:
                self.back.extend(block)

    def note_load(self, call):
        """名字里带 load / import 的调用（LOADER_CALLEE）与载入函数（LOADER_BODY_CALLEES）的参数里写了 .py 文件：路径折得出的记进 loaded，折不出的记进 unfolded_loads。"""
        name = callee_name(call.func)
        if name is None or not (LOADER_CALLEE.search(name) or name in LOADER_BODY_CALLEES):
            return
        for argument in [*call.args, *(keyword.value for keyword in call.keywords)]:
            folded = self.fold(argument)
            if folded is not None and os.path.isabs(folded) and folded.endswith(".py"):
                relative = os.path.relpath(os.path.normpath(folded), self.root)
                self.loaded.setdefault(os.path.basename(relative)[:-3], set()).add(relative)
                continue
            for inner in ast.walk(argument):
                if isinstance(inner, ast.Constant) and isinstance(inner.value, str) and inner.value.endswith(".py"):
                    self.unfolded_loads.add(os.path.basename(inner.value)[:-3])

    def library_file(self, stem, same_name):
        """调用方里引的库名 stem，运行时解到仓里哪一份 → (相对仓根的路径, None)；认不出是哪一份交 (None, 原因)。same_name 是仓里叫这个名字的 .py（相对仓根的路径集合）。
        只有一份就是它；不止一份时依次取：按路径载入时写死的那一份、往 sys.path 最前放的目录（调用方自己放的，再加 import 进来的模块放的）、调用方自己所在的目录、
        往 sys.path 后面放的目录（同样先自己的、再 import 进来的模块的）。"""
        if len(same_name) <= 1:
            return next(iter(same_name), None), None
        loaded = self.loaded.get(stem, set())
        if len(loaded) == 1:
            return next(iter(loaded)), None
        if len(loaded) > 1:
            return None, f"它按路径载入 {stem}.py 时点了不止一份（{'、'.join(sorted(loaded))}）"
        if stem in self.unfolded_loads:
            return None, f"它按路径载入 {stem}.py 时路径折不出"
        if self.unfolded_path:
            return None, "它往 sys.path 放的目录有折不出的"
        imported_front, imported_back, imported_vague, opaque = self.imported_reach()
        if opaque:
            return None, f"它 import 的 {'、'.join(sorted(opaque))} 往 sys.path 放的目录有折不出的"
        for exacts, tails in ([] if self.mode == "vague-directories-ignored" else [*self.vague, *imported_vague]):
            for candidate in sorted(same_name):
                directory = os.path.dirname(candidate)
                if directory in exacts or any(directory == tail or directory.endswith(os.sep + tail) for tail in tails):
                    shown = "、".join([*exacts, *(f"…/{tail}" for tail in tails)])
                    return None, f"它往 sys.path 放的目录有一处有几种可能或只知道末尾几段（{shown}），{candidate} 在其中一种里"
        for directory in [*self.front, *imported_front, *([self.own] if self.own is not None else []), *self.back, *imported_back]:
            candidate = os.path.normpath(os.path.join(directory, stem + ".py"))
            if candidate in same_name:
                return candidate, None
        return None, "它所在的目录与它（含它 import 进来的模块）往 sys.path 放的目录里都没有这个名字的文件"


class CallerUnit(NamedTuple):
    relative: str    # 相对仓根的路径
    offset: int      # 段里第 1 行在文件里是第几行减一（.py 整份是 0；.sh 里的 heredoc 段是正文前一行的行号，起头那一行带续行的是续行结束的那一行）
    tree: object
    stem: str        # .py 整份的库名；.sh 里的段是 ""


class ModuleSources:
    """这一批读进来的源码（相对仓根的路径 → 文本，overlay 与换上的库已按新版）：按路径解析（标过父节点）、造 SearchPaths 记下，
    SearchPaths 沿模块层 import 找别的模块往 sys.path 放的目录、折别的模块里的常量时来取。"""

    def __init__(self, root, texts, mode=""):
        self.root, self.texts, self.mode = root, texts, mode
        self.trees, self.searches = {}, {}

    def tree(self, relative):
        """相对仓根的 .py → 解析过的树；不在这批源码里、不是 .py 或解析不了交 None。"""
        if relative not in self.trees:
            tree = None
            if relative.endswith(".py") and relative in self.texts:
                try:
                    tree = ast.parse(self.texts[relative])
                except (SyntaxError, ValueError):
                    tree = None
                else:
                    annotate_parents(tree)
            self.trees[relative] = tree
        return self.trees[relative]

    def search(self, relative):
        """这份 .py 自己的 SearchPaths；读不了交 None。正在造它的时候（两份模块互相 import）又问到它也交 None，不绕圈。"""
        if relative not in self.searches:
            self.searches[relative] = None
            tree = self.tree(relative)
            if tree is not None:
                self.searches[relative] = SearchPaths(self.root, CallerUnit(relative, 0, tree, os.path.basename(relative)[:-3]), self, self.mode)
        return self.searches[relative]


class CallerNames(NamedTuple):
    missing: list      # 每项一句：谁在哪一行引了哪个库的哪个名字，库里没有
    unjudged: list     # 每项一句：认不出的写法
    references: int    # 查了几处引用
    files: int         # 读了几份调用方（含 .sh 里的 python 段所在的文件）


def caller_source_files(root, overlay):
    """扫描目录下的 .py、.sh 与带 python 的 #! 行的无后缀文件 → [(相对路径, 文本)]；overlay 里有的相对路径读 overlay 那一版。"""
    found = []
    for directory in CALLER_SCAN_DIRECTORIES:
        for current, subdirectories, files in os.walk(os.path.join(root, directory)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in CALLER_SKIPPED_DIRECTORIES)
            for name in sorted(files):
                path = os.path.join(current, name)
                relative = os.path.relpath(path, root)
                if relative in overlay:
                    found.append((relative, overlay[relative]))
                    continue
                if not os.path.isfile(path):
                    continue
                if not name.endswith((".py", ".sh")):
                    if "." in name:
                        continue
                    try:
                        with open(path, "rb") as handle:
                            if b"python" not in handle.readline(200):
                                continue
                    except OSError:
                        continue
                try:
                    found.append((relative, read_text(path)))
                except (OSError, UnicodeDecodeError):
                    continue
    return found


def caller_units(sources, stems, mode=""):
    """sources 里提到某个库名的 python 源码（.py 整份、.sh 里的 heredoc 段）解析成 [CallerUnit]；解析不了的交说明。mode 是弄坏开关，交给 embedded_python_segments。"""
    units, unparsed = [], []
    for relative, text in sources:
        stem_of_file = os.path.basename(relative)[:-3] if relative.endswith(".py") else ""
        is_library = stem_of_file in stems    # 库自己的函数可能收到别处交来的库，不管文本里提没提别的库名都要读
        if not is_library and not any(stem in text for stem in stems):
            continue
        if relative.endswith(".sh"):
            pieces = [(start, body) for start, body, closed in embedded_python_segments(text, mode) if closed]
        else:
            pieces = [(0, text)]
        for offset, body in pieces:
            if not is_library and not any(stem in body for stem in stems):
                continue
            try:
                tree = ast.parse(body)
            except (SyntaxError, ValueError) as error:
                unparsed.append(f"{relative}：python 解析不了（{getattr(error, 'msg', error)}，第 {getattr(error, 'lineno', '?')} 行），它引到的名字没核")
                continue
            annotate_parents(tree)
            units.append(CallerUnit(relative, offset, tree, stem_of_file))
    return units, unparsed


def caller_names_report(root, libraries, overlay, mode=""):
    """libraries：{相对路径: 换上之后的库文本}；overlay：{相对路径: 换上之后的别的文本}（调用方也按这一版读）。
    调用方里 <载入它的变量>.<名字> 的每个名字，在它运行时真解到的那一份库（libraries 里的，SearchPaths.library_file）的模块层里都要有；
    库名对得上不止一份文件而认不出解到哪一份的，列成本次未判。mode 是弄坏开关，交给 CallerFlow；library-by-stem-only 下库只按文件名认（同名的几份当作一份）。"""
    unjudged, targets, definitions, stem_library = [], {}, {}, {}
    for relative, text in libraries.items():
        stem = os.path.basename(relative)[:-3]
        try:
            names, dynamic = module_level_names(text)
        except (SyntaxError, ValueError) as error:
            unjudged.append(f"{relative}：编译不过（{getattr(error, 'msg', error)}），谁引了它的哪些名字没核")
            continue
        targets[stem] = stem + ".py"
        definitions[relative] = (names, dynamic)
        stem_library[stem] = relative
    tracked = dict(targets)    # 跟着走的库：换上的库，加上钩子目录里别的 .py（它们的函数收到库、导出别的库时，名字要跟到底；它们自己的名字不核）
    hooks_directory = os.path.join(root, HOOKS_DIRECTORY)
    for name in (sorted(os.listdir(hooks_directory)) if os.path.isdir(hooks_directory) else []):
        if name.endswith(".py"):
            tracked.setdefault(name[:-3], name)
    sources = caller_source_files(root, {**overlay, **libraries})
    same_name_files = {}    # 库名 → 仓里叫这个名字的 .py（扫描目录下的、加上这一次要核的库；相对仓根的路径）
    for relative in [relative for relative, _ in sources] + list(definitions):
        if relative.endswith(".py"):
            same_name_files.setdefault(os.path.basename(relative)[:-3], set()).add(relative)
    units, unparsed = caller_units(sources, list(tracked), mode)
    unjudged += unparsed
    modules = ModuleSources(root, {**dict(sources), **libraries}, mode)
    exports, hints, flows = {}, {}, []
    for _ in range(CROSS_FILE_ROUNDS):
        flows = [(unit, CallerFlow(unit.tree, tracked, exports, hints, unit.stem if unit.stem in tracked else "", mode)) for unit in units]
        derived_exports, derived_hints = {}, {}
        for unit, flow in flows:
            if unit.stem in tracked:
                for key, kind in flow.bound.items():
                    if key[0] == "name" and key[1][0] is None and kind[0] == "module" and kind[1] != unit.stem:
                        derived_exports.setdefault(unit.stem, {})[key[1][1]] = kind[1]
            for hint_key, position, keyword, kind in flow.hint_records:
                derived_hints.setdefault(hint_key, set()).add((position, keyword, kind))
        if derived_exports == exports and derived_hints == hints:
            break
        exports, hints = derived_exports, derived_hints
    missing, references, unresolved = [], 0, {}
    for unit, flow in flows:
        searches = []    # 这份调用方的 SearchPaths（用到才造，只有同名的库不止一份时用得上）
        for line, stem, name in flow.uses:
            if stem not in targets:
                continue
            same_name = same_name_files.get(stem, set())
            if mode == "library-by-stem-only":
                library_relative, reason = stem_library[stem], None
            elif len(same_name) <= 1:
                library_relative, reason = next(iter(same_name), None), None
            else:
                if not searches:
                    searches.append(SearchPaths(root, unit, modules, mode))
                library_relative, reason = searches[0].library_file(stem, same_name)
            if reason is not None:
                entry = unresolved.setdefault((unit.relative, stem), [unit.offset + line, set(), reason, same_name])
                entry[0] = min(entry[0], unit.offset + line)
                entry[1].add(name)
                continue
            if library_relative not in definitions:
                continue    # 解到的是同名的另一份文件，不是这一次要核的库
            references += 1
            names, dynamic = definitions[library_relative]
            if name in names or (name.startswith("__") and name.endswith("__")):
                continue
            where = f"{unit.relative}:{unit.offset + line}"
            if dynamic:
                unjudged.append(f"{where} 引了 {stem}.{name}，{library_relative} 里找不到这个名字，但它{'、'.join(dynamic)}，名字可能是运行时才有的")
            else:
                missing.append(f"{where} 引了 {stem}.{name}，可 {library_relative} 里没有这个名字")
        for line, text in flow.unjudged:
            unjudged.append(f"{unit.relative}:{unit.offset + line} {text}")
        for line, stem, text in flow.unrecognized:
            if stem in targets:    # 点名的是这一次要核的库才列（跟着走的别的库不列）
                unjudged.append(f"{unit.relative}:{unit.offset + line} {text}")
    for (relative, stem), (line, names, reason, same_name) in unresolved.items():
        unjudged.append(f"{relative}:{line} 引了 {stem} 的 {'、'.join(sorted(names))}，仓里叫 {stem}.py 的有 {len(same_name)} 份（{'、'.join(sorted(same_name))}），"
                        f"这份调用方运行时解到哪一份认不出（{reason}），它引的名字没核")
    return CallerNames(sorted(set(missing)), sorted(set(unjudged)), references, len({unit.relative for unit in units}))


def caller_names_check(root, relatives, overlay, mode):
    """换上的 .py 库（relatives 里以 .py 结尾的）：调用方引到的名字它们里都要有。交 (问题列表, 一句做了什么, 本次未判列表)。
    库读 overlay 里换上之后的文本，overlay 里没有的读 root 里此刻那一份；调用方同样按 overlay 优先读。
    弄坏开关 skip-caller-names：缺名字也不报；subscript-key-ignored：按键取容器里的库时不看键（都只给自证判红用）。"""
    libraries = {relative: overlay[relative] if relative in overlay else read_text(os.path.join(root, relative)) for relative in relatives if relative.endswith(".py")}
    if not libraries:
        return [], "", []
    report = caller_names_report(root, libraries, overlay, mode)
    problems = [] if mode == "skip-caller-names" else [f"调用方引到的名字：{line}" for line in report.missing]
    summary = (f"引到的名字：{len(libraries)} 份库、{report.files} 份调用方文件里的 {report.references} 处引用，缺 {len(report.missing)} 处，"
               f"本次未判 {len(report.unjudged)} 处（认不出的写法不算查过）")
    return problems, summary, report.unjudged


def unjudged_names_lines(unjudged):
    return [f"  ? 本次未判（引到的名字）：{item}" for item in unjudged]


def check_callers(root, libraries, mode=""):
    """只核不换：此刻的库与调用方，调用方引到的名字库里都有没有；libraries 是库的路径，空的核 .claude/hooks/ 下全部 .py。交 (退出码, 提示行)，不写任何文件。
    缺名字退 1；认不出的写法逐处列出、不影响退出码，也不算查过。mode 是弄坏开关，自证直接调时带进来。"""
    root = os.path.realpath(root)
    if libraries:
        relatives = []
        for library in libraries:
            path = os.path.realpath(library if os.path.isabs(library) else os.path.join(root, library))
            if not path.endswith(".py") or not os.path.isfile(path) or not is_inside(path, root):
                raise BadInput(f"{library} 不是 {root} 下已有的 .py\n→ 怎么办：hook-swap.py --check-callers [<库.py>…] [--root <仓根>]，不写库就核 {HOOKS_DIRECTORY}/ 下全部 .py")
            relatives.append(os.path.relpath(path, root))
    else:
        hooks = os.path.join(root, HOOKS_DIRECTORY)
        relatives = [os.path.join(HOOKS_DIRECTORY, name) for name in sorted(os.listdir(hooks)) if name.endswith(".py")] if os.path.isdir(hooks) else []
    problems, summary, unjudged = caller_names_check(root, relatives, {}, mode)
    lines = [f"  ✗ {problem}" for problem in problems]  # gate-lint:detail
    lines += unjudged_names_lines(unjudged)
    if not summary:
        return EXIT_BAD_INPUT, ["✗ 没有可核的库：只核 .py", f"→ 怎么办：库路径写 {HOOKS_DIRECTORY}/ 下的 .py，或不写（核 {HOOKS_DIRECTORY}/ 下全部 .py）"]
    lines.append(f"{'✗' if problems else '✓'} 只核不换：{summary}")  # gate-lint:summary
    if problems:
        lines.append("→ 怎么办：缺的名字补回库里，或改引它的那几处；换钩子之前先用规格把库和调用方一起改好")
    return (EXIT_DRAFT_RED if problems else EXIT_SWAPPED), lines
