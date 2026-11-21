"""PythonModule 的一部分方法（混入类 PythonModuleDataReads）：运行期读的数据文件的路径折叠与出处、按调用处认的前缀，
单元的事实收集（facts）、程序入口带参数时的剪枝（narrowing_plan）。python_module.py 里的 PythonModule 继承它。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import fnmatch
import os
import posixpath
from lib_function_units_base import FOLD_LIMIT, FOLD_LIMIT_FORM, MEMO_IN_PROGRESS, POSITIONAL, PYTHON_INTERPRETERS, SHELL_INTERPRETERS
from lib_function_units_registry import SPEC_PATH_ARGUMENTS, TAIL_READ_SCOPES, in_tail_read_scope, record_unscoped_tail_read, registered_function_name
from lib_function_units_python_facts import (
    ARGV_BINDING, COPY_CALLS, ElementsOf, KnownWords, PATH_CONSTRUCTORS, PATH_METHODS, PATH_TRANSFORMS, PythonFacts, STAGE_SAMPLE_DIRECTORY,
    SUBPROCESS_CALLS, TEMPORARY_CALLS, bound_names, dotted_name, enclosing_unit, guard_assignments, known_equal, literal_integer, local_name_reads,
    paired_elements, walk_function_statements, walk_nodes, walk_nodes_skipping, walk_targets,
)
from lib_function_units_shell_text import git_repository_location, in_skipped_directory, tree_copy_file

# 列目录的几种调用（fixed_directory_read：从这个目录的清单里挑一份读，读的范围是整个目录）
LISTING_CALLS = frozenset(("os.listdir", "os.scandir", "os.walk"))
# 按名字模式列的清单（listing_collection）：每一项的最后一段合模式的最后一段
GLOB_CALLS = frozenset(("glob.glob", "glob.iglob"))
# 只改次序、去重、不改元素的内置调用：清单经它们之后照旧是那份清单（listing_collection）
ORDER_ONLY_BUILTINS = frozenset(("sorted", "list", "tuple", "set", "frozenset", "reversed"))
LISTING_COMPREHENSIONS = (ast.ListComp, ast.SetComp, ast.GeneratorExp)
# 名字往上找绑它的那一层时停在这几种作用域上（loop_binding）
OWN_NAME_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda, ast.ClassDef)


class PythonModuleDataReads:
    def written_files(self, unit):
        """这个单元自己写出来的文件：open(<表达式>, "w" / "x" / "wb" / "xb") 打开、写进去的是折得出的字面文本（只写这一样）的那几个表达式（ast.dump）。
        子进程起的脚本正是其中之一，代码就是这个单元里的那段文本（不再往里看它装了什么）。"""
        if unit.cache is None:
            unit.cache = {}
        if "written" not in unit.cache:
            written = set()
            for node in walk_nodes(unit.nodes or []):
                if not isinstance(node, (ast.With, ast.AsyncWith)):
                    continue
                for item in node.items:
                    call = item.context_expr
                    if not (isinstance(call, ast.Call) and self.canonical(call.func, unit) in ("open", "io.open") and call.args
                            and isinstance(item.optional_vars, ast.Name)):
                        continue
                    mode = call.args[1] if len(call.args) > 1 else next((keyword.value for keyword in call.keywords if keyword.arg == "mode"), None)
                    if not (isinstance(mode, ast.Constant) and mode.value in ("w", "x", "wb", "xb")):
                        continue
                    handle = item.optional_vars.id
                    writes = [inner for inner in walk_nodes(node.body) if isinstance(inner, ast.Call) and isinstance(inner.func, ast.Attribute)
                              and isinstance(inner.func.value, ast.Name) and inner.func.value.id == handle]
                    if writes and all(inner.func.attr == "write" and len(inner.args) == 1 and self.fold(inner.args[0], unit) is not None for inner in writes):
                        written.add(ast.dump(call.args[0]))
            unit.cache["written"] = written
        return unit.cache["written"]

    def git_output(self, call, unit):
        """subprocess.run([...git… rev-parse… --show-toplevel|--git-common-dir…]).stdout 与 check_output 同样的命令 → {根} / {根/.git}。"""
        if self.canonical(call.func, unit) not in SUBPROCESS_CALLS or not call.args:
            return None
        elements = self.command_elements(call.args[0], unit, False)
        if elements is None:
            return None
        words = []
        for element in elements:
            folded = self.fold(element, unit) if not isinstance(element, ast.Starred) else None
            words.append(next(iter(folded)) if folded is not None and len(folded) == 1 else None)
        return git_repository_location(words, self.index.root, self.index.breaks)

    def literal_tail(self, node, unit, depth=0):
        """路径表达式 → 「折不出的前缀之后的字面尾巴」（不带前缀；整个折不出、又没有字面尾巴 → ""）；尾巴里夹着折不出的 → None。"""
        if depth > 8 or node is None:
            return None
        folded = self.fold(node, unit)
        if folded is not None:
            return next(iter(folded)) if len(folded) == 1 else None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            parts = []
            for argument in reversed(node.args):
                value = self.fold(argument, unit)
                if value is None or len(value) != 1:
                    head = self.literal_tail(argument, unit, depth + 1)
                    if head is None:
                        return None
                    parts.append(head)
                    break
                parts.append(next(iter(value)))
            else:
                return None
            return posixpath.join(*reversed(parts)) if parts else None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            right = self.fold(node.right, unit)
            if right is None or len(right) != 1:
                return None
            left = self.literal_tail(node.left, unit, depth + 1)
            if left is None:
                return None
            value = next(iter(right))
            return left + value if isinstance(node.op, ast.Add) else posixpath.join(left, value)
        if isinstance(node, ast.JoinedStr) and node.values and isinstance(node.values[0], ast.FormattedValue) and all(
                isinstance(part, ast.Constant) for part in node.values[1:]):
            head = self.literal_tail(node.values[0].value, unit, depth + 1)
            return None if head is None else head + "".join(str(part.value) for part in node.values[1:])
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id, [])
            if len(assigned) == 1 and assigned[0] is not None:
                return self.literal_tail(assigned[0], unit, depth + 1)
        return ""

    def outside_tail(self, value):
        """一个程序路径的字面尾巴（os.path.join(<折不出的目录>, "bin/x") 的 "bin/x"）是不是根外的程序：不带脚本后缀，根里也没有这一截。"""
        tail = value.strip("/")
        if not tail or tail.endswith((".py", ".sh", ".bash")) or posixpath.basename(tail) in PYTHON_INTERPRETERS | SHELL_INTERPRETERS:
            return False
        if tail.startswith("target/") or "/target/" in tail:
            # cargo 编出来的二进制：不是源码（它的源码由 Rust 那一份闭包管）
            return True
        return tree_copy_file(self.index.root, tail) is None and not os.path.exists(os.path.join(self.index.root, tail))

    def outside_head(self, node, unit, depth=0):
        """命令位置上折不出的元素是不是根外的程序：shutil.which(…)、os.path.join(<折不出的目录>, <outside_tail 认的字面尾巴>)，
        或经本函数里的赋值（连同 a, b = f() 按位置拆包）、函数参数（每一处调用给的实参）、调仓内函数的返回值到这几种；有一种取法不是 → False。"""
        if depth > 10 or node is None or "python-unfolded-heads-allowed" in self.index.breaks:
            return False
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical == "shutil.which":
                return True
            if canonical in ("os.path.join", "posixpath.join") and node.args:
                tail = self.fold(node.args[-1], unit)
                return tail is not None and len(tail) == 1 and self.outside_tail(next(iter(tail)))
            if isinstance(node.func, ast.Name):
                return self.returns_outside(node.func.id, unit, None, depth)
            return False
        if isinstance(node, ast.BoolOp):
            for operand in node.values:
                folded = self.fold(operand, unit)
                if not (self.outside_head(operand, unit, depth + 1) or (folded is not None and all(self.outside_tail(value) for value in folded))):
                    return False
            return True
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            return self.outside_tail(node.value)
        if self.inline is not None and isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv" and \
                isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and 1 <= node.slice.value <= len(self.inline.argv) and \
                self.inline.argv[node.slice.value - 1] is POSITIONAL:
            # 内联脚本的参数取的是起它那份脚本的 "$1" 一类：程序是调用处给的那个词（同 shell 那边命令位置上的 "$1"）
            return True
        if isinstance(node, ast.Name) and (unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef))):
            # 模块级的名字（连同 a, b = x, y 拆包）：每一处赋值都得是根外的程序
            candidates = [candidate for candidate in self.by_name.get(node.id, ()) if candidate.kind == "assign" and isinstance(candidate.node, ast.Assign)]
            if not candidates:
                return False
            for candidate in candidates:
                statement = candidate.node
                value = statement.value if len(statement.targets) == 1 and isinstance(statement.targets[0], ast.Name) else \
                    dict(paired_elements(statement.targets[0], statement.value) or ()).get(node.id) if len(statement.targets) == 1 else None
                if value is None or not self.outside_head(value, candidate, depth + 1):
                    return False
            return True
        if not isinstance(node, ast.Name) or unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return False
        sources = []
        for statement in walk_function_statements(unit.node):
            if isinstance(statement, ast.Assign):
                for target in statement.targets:
                    if isinstance(target, ast.Name) and target.id == node.id:
                        sources.append((statement.value, None))
                    elif isinstance(target, (ast.Tuple, ast.List)) and node.id in [item.id for item in target.elts if isinstance(item, ast.Name)]:
                        names = [item.id if isinstance(item, ast.Name) else None for item in target.elts]
                        sources.append((statement.value, names.index(node.id)))
            elif isinstance(statement, ast.AugAssign) and isinstance(statement.target, ast.Name) and statement.target.id == node.id:
                return False
            elif isinstance(statement, (ast.For, ast.AsyncFor, ast.With, ast.AsyncWith)) and node.id in {inner.id for inner in walk_targets(statement)}:
                return False
        if not sources:
            arguments = unit.node.args
            if node.id not in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs] or unit.container is not None:
                return False
            positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
            position = positional.index(node.id) if node.id in positional else None
            sites = self.function_call_sites(unit)
            if not sites:
                return False
            for call, caller in sites:
                argument = call.args[position] if position is not None and position < len(call.args) else None
                argument = next((keyword.value for keyword in call.keywords if keyword.arg == node.id), argument)
                if argument is None:
                    return False
                if "python-written-scripts-ignored" not in self.index.breaks and ast.dump(argument) in self.written_files(caller):
                    # 调用方自己 open(…, "w") 写出字面文本的那个程序：代码就是调用方里那段文本
                    continue
                if not self.outside_head(argument, caller, depth + 1):
                    return False
            return True
        for value, index in sources:
            if index is None:
                if not self.outside_head(value, unit, depth + 1):
                    return False
            elif not (isinstance(value, ast.Call) and isinstance(value.func, ast.Name) and self.returns_outside(value.func.id, unit, index, depth)):
                return False
        return True

    def returns_outside(self, name, unit, index, depth):
        """调仓内模块级函数 name：它每一处 return 的值（index 不是 None 时取元组的第 index 个）都是 outside_head 认的根外程序。"""
        binding = self.binding(name, unit)
        module = self
        if binding is not None:
            if binding[0] != "from":
                return False
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return False
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return False
        callee = candidates[0]
        returns = [statement.value for statement in walk_function_statements(callee.node) if isinstance(statement, ast.Return) and statement.value is not None]
        if not returns:
            return False
        for value in returns:
            if index is not None:
                if not isinstance(value, (ast.Tuple, ast.List)) or index >= len(value.elts):
                    return False
                value = value.elts[index]
            if not module.outside_head(value, callee, depth + 1):
                return False
        return True

    def path_values(self, node, unit, directory=False):
        """折得出的 → 那几个值；折不出、但「折不出的目录」后面接着一段在根里存在的字面尾巴（临时树、仓副本里的同一相对路径）→ {根/尾巴}；都不行 → None。"""
        values = self.fold(node, unit)
        if values is not None or "tree-copy-tails-ignored" in self.index.breaks:
            return values
        tail = self.literal_tail(node, unit)
        copied = tree_copy_file(self.index.root, tail, directory) if tail else None
        if copied is not None and in_skipped_directory(copied) and "python-data-reads-follow-build-outputs" not in self.index.breaks:
            copied = None   # 编译产物、缓存（SKIPPED_DIRECTORIES）随机器与编译目录变，不是判法自己的数据（文件头「运行期读的数据文件」编译产物那一条）
        return {posixpath.join(self.index.root, copied)} if copied is not None else None

    def directory_wildcard(self, node, unit, suffix, depth=0):
        """os.path.join(<目录>, <折不出的文件名>)，目录折得出（或照临时树尾巴认得出）是根里的一个目录 → 那个目录里全部 <suffix> 文件（绝对路径）；
        起的是「那个目录里调用方点名的那一份」，哪一份都可能，就都连。名字是本函数里只赋过一次的局部变量的，照它的值认。"""
        if "python-directory-wildcards-ignored" in self.index.breaks or depth > 4:
            return None
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id, [])
            return self.directory_wildcard(assigned[0], unit, suffix, depth + 1) if len(assigned) == 1 and assigned[0] is not None else None
        if not (isinstance(node, ast.Call) and not node.keywords and len(node.args) >= 2
                and self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")):
            return None
        if self.fold(node.args[-1], unit) is not None:
            return None
        prefix = ast.Call(func=node.func, args=list(node.args[:-1]), keywords=[]) if len(node.args) > 2 else node.args[0]
        directories = self.path_values(prefix, unit, directory=True)
        if not directories:
            return None
        files = set()
        for directory in directories:
            if self.index.relative(directory) is None or not os.path.isdir(directory):
                return None
            files |= {posixpath.join(directory, name) for name in os.listdir(directory)
                      if name.endswith(suffix) and os.path.isfile(os.path.join(directory, name))}
        return files or None

    def file_for_value(self, value, relative_bases):
        """折出来的路径值 → 相对根的文件；relative_bases 是相对路径依次试的几处。"""
        if posixpath.isabs(value):
            candidates = [value]
        else:
            candidates = [posixpath.normpath(posixpath.join(base, value)) for base in relative_bases]
        for candidate in candidates:
            relative = self.index.relative(candidate)
            if relative is not None and os.path.isfile(candidate):
                return relative
        return None

    def under_fold_limit(self, values):
        """values 不超过 FOLD_LIMIT 种交它；超过交 None、index.fold_limit_hits 加一（同 PythonModule.fold：读文件那几处照它拒算 FOLD_LIMIT_FORM）。"""
        if len(values) <= FOLD_LIMIT:
            return values
        self.index.fold_limit_hits += 1
        return None

    def unmatched_tail_ends(self, joined, suffixes, anchored):
        """os.path.join(<折不出的前缀>, <折得出的几段>) 的尾巴 joined 超过 FOLD_LIMIT 种时：每一种都是「前缀的尾巴/<suffixes 里的一种>」，
        对得上其中一种的文件必以「/<那一截>」收尾（或就是那一截）；suffixes 里每一截在仓里一份文件都对不上（比法同 data_read_origins_once 收尾巴那一步，
        不跳样本档，宽的一边），整组就一份都对不上：交那几截（anchored 接在前头，不超上限、计数不涨），读文件那一步照「不是仓里的文件」认。
        没超上限、有一截是空的、或有一截对得上的交 None（照原样走 under_fold_limit）。弄坏开关 fold-limit-unmatched-tails-refused 下一律交 None。"""
        if len(joined) <= FOLD_LIMIT or len(suffixes) > FOLD_LIMIT or "fold-limit-unmatched-tails-refused" in self.index.breaks:
            return None
        stripped = [suffix.strip("/") for suffix in suffixes]
        if not all(stripped):
            return None
        for relative in self.index.repository_files():
            if any(relative == end or relative.endswith("/" + end) or (not anchored and relative.endswith(end)) for end in stripped):
                return None
        return {anchored + end for end in stripped}

    def path_tails(self, node, unit, depth=0):
        """路径表达式 → 它可能取到的「折不出的前缀之后那一截字面尾巴」的集合（几处调用各给一种的取遍；整个折得出交它的取值）；
        尾巴认不出、或有一种取法一点字面尾巴都没有 → None。os.path.join / Path(…) / a / b / a + b / f 串、本函数里只赋过一次的名字、
        函数参数（调它的每一处给的实参）照认。os.path.join / Path(…) / a / b 拼的、折不出的前缀一点尾巴都没给的，尾巴带前导 /（按整段路径比：
        os.path.join(<折不出>, "b.py") 不收 remove_stub.py）；a + b 与 f 串拼的照字符串后缀比，不带。弄坏开关 python-joined-tails-unanchored 下 join 与 / 拼的也不带。"""
        anchored = "/" if "python-joined-tails-unanchored" not in self.index.breaks else ""
        if depth > 10 or node is None:
            return None
        folded = self.fold(node, unit)
        if folded is not None:
            return set(folded) if folded else None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            suffixes = {""}
            for argument in reversed(node.args):
                value = self.fold(argument, unit)
                if value is None:
                    heads = self.path_tails(argument, unit, depth + 1) or {""}
                    joined = {(posixpath.join(head, suffix) if head else anchored + suffix) if suffix else head for head in heads for suffix in suffixes}
                    if not all(item.strip("/") for item in joined):
                        return None
                    ends = self.unmatched_tail_ends(joined, suffixes, anchored)
                    return ends if ends is not None else self.under_fold_limit(joined)
                suffixes = {posixpath.join(item, suffix) if suffix else item for item in value for suffix in suffixes}
                if self.under_fold_limit(suffixes) is None:
                    return None
            return suffixes or None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            right = self.fold(node.right, unit)
            if right is None:
                return None
            heads = self.path_tails(node.left, unit, depth + 1) or {""}
            joined = {(head + value if isinstance(node.op, ast.Add) else posixpath.join(head, value) if head else anchored + value) for head in heads for value in right}
            return self.under_fold_limit(joined) if all(item.strip("/") for item in joined) else None
        if isinstance(node, ast.JoinedStr):
            tail = self.literal_tail(node, unit)
            return {tail} if tail and tail.strip("/") else None
        if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            assigned = self.local_assignments(unit).get(node.id)
            if assigned is not None:
                tails = set()
                for value in assigned:
                    if value is None or isinstance(value, ElementsOf):
                        return None
                    found = self.path_tails(value, unit, depth + 1)
                    if not found:
                        return None
                    tails |= found
                return self.under_fold_limit(tails)
            arguments = unit.node.args
            if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args] and unit.container is None:
                position = [argument.arg for argument in arguments.posonlyargs + arguments.args].index(node.id)
                sites = self.function_call_sites(unit)
                if not sites:
                    return None
                tails = set()
                for call, caller in sites:
                    if any(isinstance(item, ast.Starred) for item in call.args):
                        return None
                    argument = call.args[position] if position < len(call.args) else next(
                        (keyword.value for keyword in call.keywords if keyword.arg == node.id), None)
                    found = self.path_tails(argument, caller, depth + 1) if argument is not None else None
                    if not found:
                        return None
                    tails |= found
                return self.under_fold_limit(tails)
            if enclosing_unit(unit) is not None:
                return self.path_tails(node, enclosing_unit(unit), depth + 1)
        return None

    def script_tail_group(self, node, unit):
        """解释器后面的脚本路径折不出、只认得出字面尾巴（path_tails）时，仓里以这几截尾巴收尾的文件（绝对路径的集合；一份都没有交空集合）；
        尾巴认不出、只剩后缀（".py"）的 → None（照旧拒算）。"""
        tails = self.path_tails(node, unit)
        if not tails or any(not tail.strip("/") or posixpath.basename(tail.rstrip("/")).startswith(".") for tail in tails):
            return None
        files = set()
        for tail in tails:
            stripped = tail.strip("/")
            files |= {posixpath.join(self.index.root, relative) for relative in self.index.repository_files() if relative.endswith(stripped)}
        return files

    # 运行期读的数据文件（文件头「运行期读的数据文件」）

    def read_target(self, node, unit):
        """一次调用是不是在读文件：open / io.open / codecs.open（模式里没有 w a x +）、<路径>.read_text() / .read_bytes()、Path(…).open()（同样看模式）、
        shutil.copy / copy2 / copyfile 的源 → 被读的路径表达式；别的 → None。"""
        canonical = self.canonical(node.func, unit)
        keywords = {keyword.arg: keyword.value for keyword in node.keywords if keyword.arg}

        def writes(mode):
            return isinstance(mode, ast.Constant) and isinstance(mode.value, str) and any(letter in mode.value for letter in "wax+")
        if canonical in ("open", "io.open", "codecs.open"):
            path = node.args[0] if node.args else keywords.get("file", keywords.get("filename"))
            if writes(node.args[1] if len(node.args) > 1 else keywords.get("mode")):
                return None
            return path
        function = node.func
        if isinstance(function, ast.Attribute) and function.attr in ("read_text", "read_bytes") and not node.args:
            return function.value
        if isinstance(function, ast.Attribute) and function.attr == "open" and isinstance(function.value, ast.Call) and \
                self.canonical(function.value.func, unit) in PATH_CONSTRUCTORS:
            return None if writes(node.args[0] if node.args else keywords.get("mode")) else function.value
        if canonical in COPY_CALLS and node.args:
            return node.args[0]
        return None

    def data_read_origins(self, node, unit, depth=0):
        """data_read_origins_once 的结果按（节点, 作用域单元）记下：同一处实参经几条调用链问到只算一次；正在算的又问到自己（绕回来）交推不出（照调用方交进来的输入认）。"""
        return self.memoized("_read_origins", node, unit, lambda: self.data_read_origins_once(node, unit, depth), frozenset({("skip", "unknown")}))

    def memoized(self, table, node, unit, compute, cycle):
        """按（节点, 单元）记下 compute() 的结果（存在这份模块上的 table 里）；正在算的又问到自己交 cycle。"""
        memo = self.__dict__.setdefault(table, {})
        key = (id(node), id(unit))
        if key in memo:
            return cycle if memo[key] is MEMO_IN_PROGRESS else memo[key]
        memo[key] = MEMO_IN_PROGRESS
        try:
            memo[key] = compute()
        except BaseException:
            del memo[key]
            raise
        return memo[key]

    def data_read_origins_once(self, node, unit, depth=0):
        """被读的路径表达式 → 取法的集合：("file", 相对根的文件) 收进键；("skip", 为什么) 不收也不拒算（不在根里、临时目录、调用方交进来的输入）；
        ("directory", 相对根的目录) 那个目录整棵收进键；("param", 函数单元, 参数名) 是本模块里没有调用处的函数参数（收闭包时在闭包里按名字找调用处，
        找不到或是入口的算调用方的输入；ScriptClosure.resolve_parameter_reads）；("refuse",) 拒算 data-path-unfolded（文件头「运行期读的数据文件」三）。
        次序：整个折得出（path_values）→ 名字照赋值、参数照每一处调用的实参 → self.<属性> 照本类里的赋值 → 字面尾巴（path_tails：仓里以它收尾的文件整组收，
        一份都没有的不是仓里的文件）→ 前缀折得出的固定目录（fixed_directory_read）→ 折不出的前缀从哪来（head_origin）→ 都推不出的照调用方交进来的输入认。"""
        if depth > 10 or node is None:
            return frozenset({("skip", "unknown")})
        if isinstance(node, ast.Constant) and not isinstance(node.value, str):
            return frozenset({("skip", "descriptor")})
        hits = self.index.fold_limit_hits
        values = self.path_values(node, unit)
        if values is not None:
            # 「<折不出的目录>/<仓里的相对路径>」那一种（tree_copy_file：整个折不出）登在 TAIL_READ_SCOPES 里的，只留登记前缀之下的（文件头「按尾巴读的登记」）
            scope = self.registered_tail_scope(node, unit) if "tail-read-scopes-copies-unscoped" not in self.index.breaks else None
            if scope is not None and self.fold(node, unit) is not None:
                scope = None
            out = set()
            for value in values:
                expanded = os.path.expanduser(value) if value.startswith("~") else value
                file = self.file_for_value(expanded, [self.index.root, posixpath.dirname(self.absolute)])
                if file is not None and in_skipped_directory(file) and "python-data-reads-follow-build-outputs" not in self.index.breaks:
                    out.add(("skip", "build-output"))
                    continue
                # tree_copy_file 收的就是根里同一相对路径的那一份：前缀登了 TAIL_READ_ROOT_COPY 的照留（文件头「按尾巴读的登记」）
                if file is not None and scope is not None and not in_tail_read_scope(file, scope, (file,), self.index.breaks):
                    out.add(("skip", "registered-scope"))
                    continue
                out.add(("file", file) if file is not None else ("skip", "not-a-repository-file"))
            return frozenset(out) or frozenset({("skip", "no-value")})
        if isinstance(node, ast.Name):
            aliased = self.name_read_origins(node.id, unit, depth)
            if aliased is not None:
                return aliased
        if isinstance(node, ast.Attribute) and "python-attribute-reads-unfolded" not in self.index.breaks:
            assigned = self.self_attribute_values(node, unit)
            if assigned:
                return frozenset().union(*(method.module.data_read_origins(value, method, depth + 1) for value, method in assigned))
        if isinstance(node, (ast.BoolOp, ast.IfExp)):
            # registration_file or os.path.join(<根>, "固定表")、a if c else b：每一边都可能是读的那一份，各认各的、并起来
            sides = node.values if isinstance(node, ast.BoolOp) else [node.body, node.orelse]
            return frozenset().union(*(self.data_read_origins(side, unit, depth + 1) for side in sides))
        tails = self.path_tails(node, unit)
        if tails and all(tail.strip("/") and not posixpath.basename(tail.rstrip("/")).startswith(".") for tail in tails):
            scope = self.registered_tail_scope(node, unit)
            out = set()
            for tail in tails:
                stripped = tail.strip("/")
                matched = [relative for relative in self.index.repository_files() if (relative == stripped or relative.endswith("/" + stripped)
                           or (not tail.startswith("/") and relative.endswith(stripped)))
                           and (not relative.startswith(STAGE_SAMPLE_DIRECTORY) or "python-tail-reads-samples-included" in self.index.breaks)
                           and (scope is None or in_tail_read_scope(relative, scope, (stripped,), self.index.breaks))]
                out |= {("file", relative) for relative in matched}
                if not matched:
                    out.add(("skip", "not-a-repository-file" if scope is None else "registered-scope"))
            if scope is None:
                record_unscoped_tail_read(self, node, unit, out)
            anchor = self.prefix_parameter(node, unit) if "python-tail-reads-unanchored" not in self.index.breaks else None
            if anchor is not None and any(origin[0] == "file" for origin in out):
                # 折不出的前缀是本函数的参数：仓里这一截尾巴的那一组留到收完闭包再按调用处认（ScriptClosure.tail_read_supplied）
                return frozenset({("param-tail", anchor[0], anchor[1], frozenset(origin[1] for origin in out if origin[0] == "file"))})
            return frozenset(out)
        directory = self.fixed_directory_read(node, unit, depth)
        if directory is not None:
            return self.fold_limit_refused(directory, hits)
        origins = self.head_origin(node, unit, depth + 1)
        # 出处推不出、前缀也推不出的：照调用方交进来的输入认（文件头「运行期读的数据文件」二的末一种），不收也不拒算
        return self.fold_limit_refused(frozenset(origins) if origins is not None else frozenset({("skip", "unknown")}), hits)

    def fold_limit_refused(self, origins, hits):
        """这一处读的路径算的过程中有取值超过 FOLD_LIMIT 种（index.fold_limit_hits 比 hits 多），落到的又只是「推不出」那几样
        （根本身 + 折不出的一截的 skip listing、出处推不出的 skip unknown、只照前缀认的参数）：取值认得出、只是太多，交 ("refuse", FOLD_LIMIT_FORM)，
        不当成调用方的输入不收（文件头「拒算」公共一行）。收得全的（文件、整个目录、尾巴那一组、已拒算的）与推得出是临时目录、输入、根外的照原样交。
        弄坏开关 fold-limit-reads-skipped 下照原样交（超了的既不收也不拒算）。"""
        if self.index.fold_limit_hits == hits or "fold-limit-reads-skipped" in self.index.breaks:
            return origins
        if origins and all(origin in (("skip", "listing"), ("skip", "unknown")) or origin[0] == "param" for origin in origins):
            return frozenset({("refuse", FOLD_LIMIT_FORM)})
        return origins

    def registered_tail_scope(self, node, unit):
        """按字面尾巴读的这一处（连同 tree_copy_file 那一种）登在 TAIL_READ_SCOPES 里（文件头「按尾巴读的登记」）→ 仓里那一组只留的前缀（元组，相对根、不带收尾的 /；
        空元组是调用方交进来的输入，一份都不收；TAIL_READ_ROOT_COPY 只留根上那一份，比法在 in_tail_read_scope）；没登记 → None。弄坏开关 tail-read-scopes-ignored 下一律当没登记。
        先比文件与函数：每一处折得出的读都问到这里，那一对不在表里的不做 ast.unparse。"""
        if unit is None or not getattr(unit, "short", None) or "tail-read-scopes-ignored" in self.index.breaks:
            return None
        file, function = self.source_file(), registered_function_name(unit)
        if not any(registered_file == file and registered_function == function for registered_file, registered_function, _expression in TAIL_READ_SCOPES):
            return None
        registered = TAIL_READ_SCOPES.get((file, function, ast.unparse(node)))
        return None if registered is None else tuple(prefix.strip("/") for prefix in registered[0])

    def prefix_parameter(self, node, unit, depth=0):
        """被读路径最左那一截折不出的前缀是本函数的哪个参数（os.path.join / Path 的头一个实参、os.path.dirname 这类、x[…]、x / …、本函数里给它的赋值，
        赋值里引用它自己的那几处不算）→ (函数单元, 参数名)；推不到一个参数上 → None。"""
        if depth > 8 or node is None or unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            return None
        if isinstance(node, ast.Call) and node.args and (self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")
                                                         or self.canonical(node.func, unit) in PATH_CONSTRUCTORS | PATH_TRANSFORMS):
            return self.prefix_parameter(node.args[0], unit, depth + 1)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr in PATH_METHODS:
            return self.prefix_parameter(node.func.value, unit, depth + 1)
        if isinstance(node, ast.Subscript):
            return self.prefix_parameter(node.value, unit, depth + 1)
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            return self.prefix_parameter(node.left, unit, depth + 1)
        if not isinstance(node, ast.Name):
            return None
        assigned = self.local_assignments(unit).get(node.id)
        arguments = unit.node.args
        parameter = node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]
        if assigned is None:
            return (unit, node.id) if parameter else None
        # 参数自己算一处（arguments = [a for a in arguments if …] 这种只引用自己的赋值不另算）
        anchors, anchor = ({(id(unit), node.id)}, (unit, node.id)) if parameter else (set(), None)
        for value in assigned:
            if value is not None and not isinstance(value, ElementsOf) and any(
                    isinstance(inner, ast.Name) and inner.id == node.id for inner in ast.walk(value)):
                continue
            found = self.prefix_parameter(value, unit, depth + 1) if value is not None and not isinstance(value, ElementsOf) else None
            if found is None:
                return None
            anchors.add((id(found[0]), found[1]))
            anchor = found
        return anchor if len(anchors) == 1 else None

    def fixed_directory_read(self, node, unit, depth):
        """os.path.join(<前缀>, <折不出的几截>)（文件头「运行期读的数据文件」一、二、三）：前缀是开头折得出的那几个实参，折不出的几截看出处
        （unfolded_part_origins）：出处都推得出 → 那几样出处（调用方的输入、参数）；推不出 → 前缀是根里的目录、折不出的那一截是从同一个目录的清单里挑的
        （directory_listing_derived）交 ("directory", 相对根的目录)；折不出的那一截是一份按名字模式列的清单里取出的文件名（pattern_listed_files：
        glob 的模式、内联脚本收到的 find 清单）交目录里名字合模式的那几份 ("file", 相对根的文件)（一份都没有交 ("skip", "not-a-repository-file")）；
        别的交 ("refuse",)（读的是哪几份折不出，不按整目录收）；
        就是根本身交 ("refuse",) 或 ("skip", "listing")、在根外交 ("skip", "outside")。不是这种写法、前缀一截都折不出 → None。
        弄坏开关 python-unlisted-directory-reads-whole 下不是从清单里挑的也交 ("directory", 相对根的目录)。"""
        if not (isinstance(node, ast.Call) and not node.keywords and len(node.args) >= 2
                and self.canonical(node.func, unit) in ("os.path.join", "posixpath.join")):
            return None
        folded = 0
        while folded < len(node.args) and self.fold(node.args[folded], unit) is not None:
            folded += 1
        if folded == 0 or folded == len(node.args):
            return None
        prefix = ast.Call(func=node.func, args=list(node.args[:folded]), keywords=[]) if folded > 1 else node.args[0]
        directories = self.path_values(prefix, unit, directory=True)
        if not directories:
            return None
        origins = set()
        for part in node.args[folded:]:
            found = self.unfolded_part_origins(part, unit, depth + 1)
            if found is None:
                origins = None
                break
            origins |= found
        if origins is not None:
            return frozenset(origins)
        out, listed = set(), None
        for directory in directories:
            absolute = directory if posixpath.isabs(directory) else posixpath.join(self.index.root, directory)
            relative = self.index.relative(posixpath.normpath(absolute))
            if relative is None:
                out.add(("skip", "outside"))
            elif relative in ("", "."):
                # 根本身 + 折不出的一截：那一截是读根里的文件得来的（登记行点名的一份）才拒算；清单遍历、算出来的名单、参数给的是被判的输入，不收不拒算
                derived = "python-root-joins-refused" in self.index.breaks or any(
                    self.repository_content_derived(part, unit, depth + 1) for part in node.args[folded:])
                out.add(("refuse",) if derived else ("skip", "listing"))
            elif "python-directory-reads-refused" in self.index.breaks:
                out.add(("refuse",))
            elif os.path.isdir(absolute):
                if listed is None:
                    listed = self.listed_directories(directories) if "python-unlisted-directory-reads-whole" not in self.index.breaks else None
                # 从这个目录的清单里挑的一份：读的范围是整个目录（清单本身就读了它），收成数据目录；按名字模式列的清单里取的文件名：读的是目录里合模式的那几份，
                # 逐份收；别的折不出的一截读的是哪几份推不出，拒算
                whole = listed is None or any(self.directory_listing_derived(part, unit, listed & {relative}, depth + 1) for part in node.args[folded:])
                matched = None if whole else self.pattern_listed_files(node.args[folded:], unit, relative, depth + 1)
                if whole:
                    out.add(("directory", relative))
                elif matched is None:
                    out.add(("refuse",))
                else:
                    out |= {("file", file) for file in matched} or {("skip", "not-a-repository-file")}
            else:
                out.add(("skip", "not-a-repository-file"))
        return frozenset(out)

    def listed_directories(self, directories):
        """path_values 交的目录（绝对或相对根）→ 落在根里、不是根本身的那几个，相对根的路径（fixed_directory_read 与 directory_listing_derived 比的那一种写法）。"""
        out = set()
        for directory in directories:
            absolute = directory if posixpath.isabs(directory) else posixpath.join(self.index.root, directory)
            relative = self.index.relative(posixpath.normpath(absolute))
            if relative not in (None, "", "."):
                out.add(relative)
        return out

    def directory_listing_derived(self, node, unit, directories, depth=0, seen=None):
        """路径里折不出的那一截是不是从 directories（相对根的目录）里某一个的清单挑出来的：沿名字往回找来处（name_sources，同 repository_content_derived）——
        碰到一处 os.listdir / os.scandir / os.walk（LISTING_CALLS），它的头一个实参折得出、是 directories 里的一个目录，就是。"""
        seen = set() if seen is None else seen
        if depth > 10 or node is None or not directories:
            return False
        for inner in ast.walk(node):
            if isinstance(inner, ast.Call) and self.canonical(inner.func, unit) in LISTING_CALLS and inner.args:
                if self.listed_directories(self.path_values(inner.args[0], unit, directory=True) or ()) & directories:
                    return True
            elif (isinstance(inner, ast.Call) and isinstance(inner.func, ast.Name) and "python-returned-listings-ignored" not in self.index.breaks
                  and self.returned_listing_of(inner, unit, directories, depth)):
                return True
            elif isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Load) and inner.id not in seen:
                seen.add(inner.id)
                if any(self.directory_listing_derived(source, unit, directories, depth + 1, seen) for source in self.name_sources(inner.id, unit)):
                    return True
        return False

    def returned_listing_of(self, call, unit, directories, depth):
        """call 调的是仓内模块级函数（本模块里定义的，或 from … import 进来的），它每一处 return 的值里都有一处 os.listdir / os.scandir / os.walk，
        列的目录折得出、是 directories 里的一个：清单经函数的返回值交出来（sorted(name for name in os.listdir(directory) if …) 这一类）。
        列的目录是那个函数的参数的，照这一处调用给的实参（没给的照缺省值）折；别的照那份模块折。弄坏开关 python-returned-listings-ignored 下不认。"""
        name, module = call.func.id, self
        binding = self.binding(name, unit)
        if binding is not None:
            if binding[0] != "from":
                return False
            status, file = self.resolve_module(binding[1], binding[2])
            module = self.index.module(file) if status == "repo" else None
            name = binding[4]
            if module is None:
                return False
        candidates = [candidate for candidate in module.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None]
        if len(candidates) != 1:
            return False
        callee = candidates[0]
        returns = [statement.value for statement in walk_function_statements(callee.node) if isinstance(statement, ast.Return) and statement.value is not None]
        if not returns:
            return False
        parameters = callee.node.args.posonlyargs + callee.node.args.args
        defaults = dict(zip([parameter.arg for parameter in parameters][len(parameters) - len(callee.node.args.defaults):], callee.node.args.defaults))
        supplied = {parameter.arg: argument for parameter, argument in zip(parameters, call.args)}
        supplied.update((keyword.arg, keyword.value) for keyword in call.keywords if keyword.arg is not None)

        def listed_here(listing):
            first = listing.args[0]
            if isinstance(first, ast.Name) and first.id in supplied:
                values = self.path_values(supplied[first.id], unit, directory=True)
            elif isinstance(first, ast.Name) and first.id in defaults:
                values = module.path_values(defaults[first.id], callee, directory=True)
            else:
                values = module.path_values(first, callee, directory=True)
            return bool(self.listed_directories(values or ()) & directories)

        for value in returns:
            listings = [inner for inner in ast.walk(value)
                        if isinstance(inner, ast.Call) and module.canonical(inner.func, callee) in LISTING_CALLS and inner.args]
            if depth > 10 or not any(listed_here(listing) for listing in listings):
                return False
        return True

    def pattern_listed_files(self, parts, unit, relative, depth=0):
        """os.path.join(<根里的目录 relative>, <折不出的一截>)：那一截只有一段、是一份按名字模式列的清单里只剩文件名的一项（listing_element）→
        目录里名字合其中一个模式的普通文件（跟着符号链接认），相对根的路径的集合；不是 → None（照拒算）。
        读的是 <目录>/<文件名>，文件名合模式：在哪个目录里列的、列了几层都不改这一点，所以不比列的目录。模式照 fnmatch 比，点开头的文件也算（宁可多收）。"""
        if len(parts) != 1:
            return None
        found = self.listing_element(parts[0], unit, depth)
        if found is None or not found[1]:
            return None
        absolute = posixpath.join(self.index.root, relative)
        return {posixpath.join(relative, name) for name in sorted(os.listdir(absolute))
                if any(fnmatch.fnmatchcase(name, pattern) for pattern in found[0]) and os.path.isfile(os.path.join(absolute, name))}

    def listing_element(self, node, unit, depth=0):
        """一个表达式是不是一份按名字模式列的清单里的一项 → (那几个名字模式的 frozenset, 是不是只剩文件名)；认不出 → None。
        认的：os.path.basename(<一项>)（只剩文件名）；名字照它绑在哪认：最近一层把它当循环变量绑的推导式或 for（loop_binding），那一层的迭代对象是一份清单
        （listing_collection）；没有这样一层的，本函数或模块里只赋过一次的照那个值认。拼接、splitext、下标一类改过的不认（读的那一份不一定合模式）。
        弄坏开关 python-pattern-listing-transforms-accepted 下改过的写法里只要有一个名字是清单里的一项，也当只剩文件名的一项。"""
        if depth > 12 or node is None:
            return None
        if isinstance(node, ast.Call) and len(node.args) == 1 and not node.keywords and \
                self.canonical(node.func, unit) in ("os.path.basename", "posixpath.basename"):
            found = self.listing_element(node.args[0], unit, depth + 1)
            return None if found is None else (found[0], True)
        if isinstance(node, ast.Name):
            bound = self.loop_binding(node, unit)
            if bound is not None:
                return self.listing_collection(bound, unit, depth + 1) if bound is not False else None
            found = self.single_assignment(node.id, unit)
            return self.listing_element(found[0], found[1], depth + 1) if found is not None else None
        if "python-pattern-listing-transforms-accepted" in self.index.breaks:
            for inner in ast.walk(node):
                found = self.listing_element(inner, unit, depth + 1) if isinstance(inner, ast.Name) else None
                if found is not None:
                    return (found[0], True)
        return None

    def node_parents(self):
        """这份模块（内联脚本是它那段正文）的语法树里每个节点的上一层：{id(节点): 上一层节点}，算一次记在模块上。"""
        parents = self.__dict__.get("_node_parents")
        if parents is None:
            parents = {id(child): node for node in ast.walk(self.tree) for child in ast.iter_child_nodes(node)} if self.tree is not None else {}
            self.__dict__["_node_parents"] = parents
        return parents

    def loop_binding(self, name, unit):
        """名字节点 name 往上找最近一层把它当循环变量绑的推导式或 for → 它是哪个迭代对象里的一项（迭代对象的节点）：循环变量就是这个名字，
        或 for i, x in enumerate(<迭代对象>) 的 x；for 的那一层只认名字在循环体里、循环体里不再给它赋值。那一层绑它的写法认不出（别的拆包、
        几层生成器、循环体里又赋值）→ False；到函数、lambda、类或模块都没有这样一层 → None（照赋值认）。"""
        parents, chain = self.node_parents(), [name]
        current = parents.get(id(name))
        while current is not None and not isinstance(current, OWN_NAME_SCOPES):
            child = chain[-1]
            if isinstance(current, LISTING_COMPREHENSIONS + (ast.DictComp,)):
                binders = [generator for generator in current.generators if name.id in bound_names(generator.target)]
                in_iteration = isinstance(child, ast.comprehension) and len(chain) >= 2 and chain[-2] is child.iter
                if binders and len(current.generators) != 1:
                    return False
                if binders and not in_iteration:
                    return self.loop_element_source(binders[0].target, binders[0].iter, name.id, unit)
            elif isinstance(current, (ast.For, ast.AsyncFor)) and name.id in bound_names(current.target) and any(child is statement for statement in current.body):
                rebound = any(isinstance(inner, ast.Name) and inner.id == name.id and isinstance(inner.ctx, (ast.Store, ast.Del))
                              for statement in current.body for inner in ast.walk(statement))
                return False if rebound else self.loop_element_source(current.target, current.iter, name.id, unit)
            chain.append(current)
            current = parents.get(id(current))
        return None

    def loop_element_source(self, target, iteration, name, unit):
        """循环变量 target 绑在 iteration 上：name 是 iteration 里的一项 → iteration；是 enumerate(<序列>) 交的 (下标, 项) 里的项 → 那个序列；别的 → False。"""
        if isinstance(target, ast.Name):
            return iteration
        if isinstance(target, ast.Tuple) and len(target.elts) == 2 and isinstance(target.elts[1], ast.Name) and target.elts[1].id == name \
                and name not in bound_names(target.elts[0]) and isinstance(iteration, ast.Call) and isinstance(iteration.func, ast.Name) \
                and iteration.func.id == "enumerate" and self.binding("enumerate", unit) is None and len(iteration.args) == 1 and not iteration.keywords:
            return iteration.args[0]
        return False

    def listing_collection(self, node, unit, depth=0):
        """一个可迭代对象是不是一份按名字模式列的清单 → (名字模式的 frozenset, 每一项是不是只剩文件名)；认不出 → None。认的：
        glob.glob / glob.iglob(<折得出的模式>)（每一项是路径，最后一段合模式的最后一段）；内联脚本里 sys.argv[k].split("\\n") / .splitlines()，
        起它的那条命令交的第 k 个参数是一份 find 清单（argv_listing）；sorted / list / tuple / set / frozenset / reversed(<清单>)；
        只有一层生成器的推导式，每一项是 listing_element 认得出的清单里的一项；只赋过一次的名字照那个值认。弄坏开关 python-glob-listings-ignored 下 glob 不认。"""
        if depth > 12 or node is None:
            return None
        if isinstance(node, ast.Name):
            found = self.single_assignment(node.id, unit)
            return self.listing_collection(found[0], found[1], depth + 1) if found is not None else None
        if isinstance(node, LISTING_COMPREHENSIONS):
            return self.listing_element(node.elt, unit, depth + 1) if len(node.generators) == 1 else None
        if not isinstance(node, ast.Call):
            return None
        if self.canonical(node.func, unit) in GLOB_CALLS and len(node.args) == 1 and all(keyword.arg == "recursive" for keyword in node.keywords) \
                and "python-glob-listings-ignored" not in self.index.breaks:
            values = self.fold(node.args[0], unit)
            patterns = frozenset(posixpath.basename(value) for value in values or ())
            return (patterns, False) if values and all(patterns) else None
        if isinstance(node.func, ast.Name) and node.func.id in ORDER_ONLY_BUILTINS and self.binding(node.func.id, unit) is None \
                and len(node.args) == 1 and (node.func.id == "sorted" or not node.keywords):
            return self.listing_collection(node.args[0], unit, depth + 1)
        split = isinstance(node.func, ast.Attribute) and not node.keywords and (
            (node.func.attr == "split" and len(node.args) == 1 and isinstance(node.args[0], ast.Constant) and node.args[0].value == "\n")
            or (node.func.attr == "splitlines" and not node.args))
        return self.argv_listing(node.func.value, unit, depth + 1) if split else None

    def argv_listing(self, node, unit, depth=0):
        """内联脚本里 sys.argv[k]（或只赋过一次它的名字）：起它的那条命令交的第 k 个参数是一份 find 清单（InlineScript.listings）→ (名字模式, False)；
        不是 → None。"""
        if depth > 12 or node is None:
            return None
        if isinstance(node, ast.Name):
            found = self.single_assignment(node.id, unit)
            return self.argv_listing(found[0], found[1], depth + 1) if found is not None else None
        if self.inline is None or not (isinstance(node, ast.Subscript) and self.canonical(node.value, unit) == "sys.argv"
                                       and isinstance(node.slice, ast.Constant) and type(node.slice.value) is int and node.slice.value >= 1):
            return None
        patterns = dict(self.inline.listings).get(node.slice.value - 1)
        return (patterns, False) if patterns else None

    def repository_content_derived(self, node, unit, depth=0, seen=None):
        """路径里折不出的那一截是不是读根里的文件得来的（文件头「运行期读的数据文件」三）：沿名字往回找来处（name_sources）——碰到一处读文件
        （read_target 认得出的读，拷贝不算）、它的取法里有根里的文件或目录，就是。参数、调用的返回值、认不出的都不算（照被判的输入认）。"""
        seen = set() if seen is None else seen
        if depth > 10 or node is None:
            return False
        for inner in ast.walk(node):
            if isinstance(inner, ast.Call):
                target = self.read_target(inner, unit) if self.canonical(inner.func, unit) not in COPY_CALLS else None
                if target is not None and any(origin[0] in ("file", "directory", "param-tail") for origin in self.data_read_origins(target, unit, depth + 1)):
                    return True
            elif isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Load) and inner.id not in seen:
                seen.add(inner.id)
                if any(self.repository_content_derived(source, unit, depth + 1, seen) for source in self.name_sources(inner.id, unit)):
                    return True
        return False

    def name_sources(self, name, unit):
        """名字在 unit 里的来处（repository_content_derived 用）：赋值的右边（拆包赋的取整个右边）、for 与推导式里绑它的迭代对象、
        with … as 它的那个对象；单元里一处都没有的取模块级只赋过一次的值。"""
        sources = []
        for node in walk_nodes(unit.nodes or []) if unit is not None else ():
            if isinstance(node, ast.Assign) and any(name in bound_names(target) for target in node.targets):
                sources.append(node.value)
            elif isinstance(node, (ast.AnnAssign, ast.AugAssign, ast.NamedExpr)) and node.value is not None and name in bound_names(node.target):
                sources.append(node.value)
            elif isinstance(node, (ast.For, ast.AsyncFor, ast.comprehension)) and name in bound_names(node.target):
                sources.append(node.iter)
            elif isinstance(node, (ast.With, ast.AsyncWith)):
                sources += [item.context_expr for item in node.items if item.optional_vars is not None and name in bound_names(item.optional_vars)]
        if not sources:
            found = self.single_assignment(name, None)
            if found is not None:
                sources.append(found[0])
        return sources

    def unfolded_part_origins(self, node, unit, depth=0):
        """路径或命令里折不出的那一截 → 它里头每一处折不出的东西的出处（head_origin 的取法）；有一处推不出交 None。
        os.path.join、Path(…)、a + b、a / b、f 串逐段拆开，折得出的那几段不算。"""
        if depth > 12 or node is None:
            return None
        if self.fold(node, unit) is not None:
            return set()
        parts = None
        if isinstance(node, ast.Call) and not node.keywords and node.args and (
                self.canonical(node.func, unit) in ("os.path.join", "posixpath.join") or self.canonical(node.func, unit) in PATH_CONSTRUCTORS):
            parts = list(node.args)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            parts = [node.left, node.right]
        elif isinstance(node, ast.JoinedStr):
            parts = [part.value for part in node.values if isinstance(part, ast.FormattedValue)]
        if parts is None:
            if isinstance(node, ast.Name) and unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                arguments = unit.node.args
                if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs] \
                        and node.id not in self.local_assignments(unit):
                    # 参数：给它实参的是哪几处，收完闭包再按闭包里的调用处认（入口的参数算调用方交进来的；ScriptClosure.launch_supplied_by_caller）
                    return {("param", unit, node.id)}
            if isinstance(node, ast.Name):
                # 只赋过一次的名字（key = f".claude/gate.d/{stage}" 一类）：照它的值逐段拆
                found = self.single_assignment(node.id, unit)
                if found is not None:
                    return self.unfolded_part_origins(found[0], found[1], depth + 1)
            return self.head_origin(node, unit, depth + 1)
        out = set()
        for part in parts:
            found = self.unfolded_part_origins(part, unit, depth + 1)
            if found is None:
                return None
            out |= found
        return out

    def self_attribute_values(self, node, unit):
        """self.<属性>（方法里）→ 本类方法里给 self.<属性> 的每一处赋值 [(值节点, 那个方法的单元)]（拆包赋的逐个对上）；
        不是这种写法、一处赋值都找不到、有一处对不上 → []。弄坏开关 python-attribute-reads-unfolded 下一律交 []。"""
        if "python-attribute-reads-unfolded" in self.index.breaks or not (
                isinstance(node, ast.Attribute) and isinstance(node.value, ast.Name) and node.value.id == "self"):
            return []
        scope = unit
        while scope is not None and enclosing_unit(scope) is not None:
            scope = enclosing_unit(scope)
        owner = scope.container if scope is not None else None
        if owner is None:
            return []
        out = []
        for method in self.methods:
            if method.container is not owner or not isinstance(method.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for statement in walk_function_statements(method.node):
                if not isinstance(statement, ast.Assign):
                    continue
                for target in statement.targets:
                    if isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id == "self" and target.attr == node.attr:
                        out.append((statement.value, method))
                    elif isinstance(target, (ast.Tuple, ast.List)) and any(
                            isinstance(item, ast.Attribute) and isinstance(item.value, ast.Name) and item.value.id == "self" and item.attr == node.attr
                            for item in target.elts):
                        if not isinstance(statement.value, (ast.Tuple, ast.List)) or len(statement.value.elts) != len(target.elts):
                            return []
                        out += [(value, method) for item, value in zip(target.elts, statement.value.elts)
                                if isinstance(item, ast.Attribute) and isinstance(item.value, ast.Name) and item.value.id == "self" and item.attr == node.attr]
        return out

    def name_read_origins(self, name, unit, depth):
        """被读的路径就是一个名字：本函数（连同外层）里的赋值逐个照 data_read_origins 认、with tempfile.… as 名字算临时目录、
        参数照每一处调用的实参认（没有调用处交 ("param", 函数单元)）、模块级只赋过一次的照那个值认；认不出 → None（交给尾巴与前缀那两步）。"""
        if unit is None or not isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            guarded = guard_assignments(name, unit)
            if guarded:
                return frozenset().union(*(self.data_read_origins(value, unit, depth + 1) for value in guarded))
            found = self.single_assignment(name, None)
            return self.data_read_origins(found[0], found[1], depth + 1) if found is not None else None
        if self.bound_to_temporary(name, unit):
            return frozenset({("skip", "temporary")})
        assigned = self.local_assignments(unit).get(name)
        if assigned is not None:
            out = set()
            for value in assigned:
                if value is None or isinstance(value, ElementsOf):
                    return None
                out |= self.data_read_origins(value, unit, depth + 1)
            return frozenset(out)
        arguments = unit.node.args
        if name in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
            sites = self.parameter_sites(unit, name)
            if sites is None:
                return None
            if not sites:
                return frozenset({("param", unit, name)})
            out = set()
            for argument, caller in sites:
                out |= caller.module.data_read_origins(argument, caller, depth + 1) if caller is not None and caller.module is not None \
                    else self.data_read_origins(argument, caller, depth + 1)
            return frozenset(out)
        if enclosing_unit(unit) is not None:
            return self.name_read_origins(name, enclosing_unit(unit), depth + 1)
        found = self.single_assignment(name, None)
        return self.data_read_origins(found[0], found[1], depth + 1) if found is not None else None

    def parameter_sites(self, unit, name):
        """函数（方法、套着的函数）参数 name 在每一处调用里拿到的实参 [(实参节点, 调用处的作用域单元)]：没给的取缺省值；
        一处都没有交 []；有一处带 * / ** 认不出对哪个参数 → None。"""
        function = unit.node
        arguments = function.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        method = unit.container is not None
        if method and not any(isinstance(decorator, ast.Name) and decorator.id == "staticmethod" for decorator in function.decorator_list):
            if not positional or positional[0] == name:
                return None
            positional = positional[1:]
        defaults = dict(zip(positional[len(positional) - len(arguments.defaults):], arguments.defaults))
        defaults.update({argument.arg: default for argument, default in zip(arguments.kwonlyargs, arguments.kw_defaults) if default is not None})
        position = positional.index(name) if name in positional else None
        out = []
        for call, caller in (self.method_call_sites(unit) if method else self.function_call_sites(unit)):
            if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
                return None
            argument = call.args[position] if position is not None and position < len(call.args) else None
            argument = next((keyword.value for keyword in call.keywords if keyword.arg == name), argument)
            if argument is None:
                argument, caller = defaults.get(name), unit
            if argument is None:
                return None
            out.append((argument, caller))
        return out

    def bound_to_temporary(self, name, unit):
        """name 在这个函数里绑的是新建的临时目录或临时文件：with tempfile.TemporaryDirectory() / NamedTemporaryFile() as name、
        <描述符>, name = tempfile.mkstemp()。"""
        for statement in walk_function_statements(unit.node):
            if isinstance(statement, (ast.With, ast.AsyncWith)):
                for item in statement.items:
                    if isinstance(item.optional_vars, ast.Name) and item.optional_vars.id == name and isinstance(item.context_expr, ast.Call) \
                            and self.canonical(item.context_expr.func, unit) in TEMPORARY_CALLS:
                        return True
            elif isinstance(statement, ast.Assign) and isinstance(statement.value, ast.Call) and \
                    self.canonical(statement.value.func, unit) in TEMPORARY_CALLS and \
                    any(name in bound_names(target) for target in statement.targets if isinstance(target, (ast.Tuple, ast.List))):
                return True
        return False

    def outside_prefix(self, prefix):
        """一段字面前缀（f"/proc/{pid}/stat" 的 "/proc/"）落在根外：是绝对路径，目录那一截不在根里、根也不在它底下。"""
        if not prefix.startswith("/"):
            return False
        directory = posixpath.normpath(prefix.rsplit("/", 1)[0] or "/")
        if directory == "/":
            return False
        return self.index.relative(directory) is None and not (self.index.root + "/").startswith(directory + "/")

    def head_origin(self, node, unit, depth):
        """head_origin_once 的结果按（节点, 作用域单元）记下（同 data_read_origins）。"""
        return self.memoized("_head_origins", node, unit, lambda: self.head_origin_once(node, unit, depth), None)

    def head_origin_once(self, node, unit, depth):
        """被读路径里折不出的那一截前缀从哪来 → {("skip", "temporary" | "input" | "environment" | "outside")、("param", 函数单元, 参数名)…}；认不出 → None。
        临时目录：tempfile 的 mkdtemp / gettempdir / TemporaryDirectory / NamedTemporaryFile / mkstemp 新建的（读的是这一趟自己写的）；
        输入：sys.argv、argparse 解出来的属性（调用方交进来的，判定的请求行管它们）；环境变量另记 ("skip", "environment")（读文件时同输入，
        起子进程时不算调用方交的命令）；根外：字面的绝对路径前缀不在根里（/proc/…）；self.<属性> 照本类里给它的赋值认；没有调用处的参数交 ("param", 函数单元, 参数名)。"""
        if depth > 12 or node is None:
            return None
        breaks = self.index.breaks
        if isinstance(node, ast.Call):
            canonical = self.canonical(node.func, unit)
            if canonical in TEMPORARY_CALLS:
                return {("skip", "temporary")}
            if canonical in ("os.environ.get", "os.getenv"):
                return {("skip", "environment")}
            if (canonical in ("os.path.join", "posixpath.join") or canonical in PATH_CONSTRUCTORS or canonical in PATH_TRANSFORMS) and node.args:
                return self.head_origin(node.args[0], unit, depth + 1)
            if isinstance(node.func, ast.Attribute) and node.func.attr in PATH_METHODS:
                return self.head_origin(node.func.value, unit, depth + 1)
            if isinstance(node.func, ast.Name) and self.binding(node.func.id, unit) is None:
                candidates = [candidate for candidate in self.by_name.get(node.func.id, ()) if candidate.kind == "def" and candidate.container is None]
                returns = [statement.value for statement in walk_function_statements(candidates[0].node)
                           if isinstance(statement, ast.Return) and statement.value is not None] if len(candidates) == 1 else []
                if not returns:
                    return None
                out = set()
                for value in returns:
                    found = candidates[0].module.head_origin(value, candidates[0], depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            return None
        if isinstance(node, ast.JoinedStr):
            prefix = ""
            for part in node.values:
                if isinstance(part, ast.Constant):
                    prefix += str(part.value)
                    continue
                folded = self.fold(part.value, unit) if isinstance(part, ast.FormattedValue) else None
                if folded is not None and len(folded) == 1:
                    prefix += next(iter(folded))
                    continue
                if self.outside_prefix(prefix):
                    return {("skip", "outside")}
                return self.head_origin(part.value, unit, depth + 1) if not prefix and isinstance(part, ast.FormattedValue) else None
            return None
        if isinstance(node, ast.BinOp) and isinstance(node.op, (ast.Add, ast.Div)):
            left = self.fold(node.left, unit)
            if left is None:
                return self.head_origin(node.left, unit, depth + 1)
            return {("skip", "outside")} if left and all(self.outside_prefix(value + "/") for value in left) else None
        if isinstance(node, ast.Attribute):
            if node.attr in ("parent", "name", "stem"):
                return self.head_origin(node.value, unit, depth + 1)
            if isinstance(node.value, ast.Name):
                found = self.single_assignment(node.value.id, unit)
                if found is not None and isinstance(found[0], ast.Call) and isinstance(found[0].func, ast.Attribute) and \
                        found[0].func.attr in ("parse_args", "parse_known_args"):
                    # 弄坏开关 python-parsed-arguments-unread：argparse 解出来的属性照判法自己的数据认，拒算
                    return {("refuse",)} if "python-parsed-arguments-unread" in breaks else {("skip", "input")}
            assigned = self.self_attribute_values(node, unit)
            if assigned:
                out = set()
                for value, method in assigned:
                    found = method.module.head_origin(value, method, depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            return None
        if isinstance(node, ast.Subscript):
            if self.canonical(node.value, unit) == "sys.argv":
                return {("skip", "input")}
            if self.canonical(node.value, unit) == "os.environ":
                return {("skip", "environment")}
            return self.head_origin(node.value, unit, depth + 1)
        if isinstance(node, ast.Name):
            if self.canonical(node, unit) == "sys.argv":
                return {("skip", "input")}
            if unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if self.bound_to_temporary(node.id, unit):
                    return {("skip", "temporary")}
                assigned = self.local_assignments(unit).get(node.id)
                if assigned is not None:
                    out = set()
                    for value in assigned:
                        source = value.sequence if isinstance(value, ElementsOf) else value
                        found = self.head_origin(source, unit, depth + 1) if source is not None else None
                        if found is None:
                            return None
                        out |= found
                    return out
                arguments = unit.node.args
                if node.id in [argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs]:
                    sites = self.parameter_sites(unit, node.id)
                    if sites is None:
                        return None
                    if not sites:
                        return {("param", unit, node.id)}
                    out = set()
                    for argument, caller in sites:
                        found = caller.module.head_origin(argument, caller, depth + 1)
                        if found is None:
                            return None
                        out |= found
                    return out
                if enclosing_unit(unit) is not None:
                    return self.head_origin(node, enclosing_unit(unit), depth + 1)
            guarded = guard_assignments(node.id, unit)
            if guarded:
                out = set()
                for value in guarded:
                    found = self.head_origin(value, unit, depth + 1)
                    if found is None:
                        return None
                    out |= found
                return out
            found = self.single_assignment(node.id, None)
            if found is not None:
                return self.head_origin(found[0], found[1], depth + 1)
        return None

    # 单元读到的东西

    def facts(self, unit):
        if unit.facts is not None:
            return unit.facts
        facts = PythonFacts()
        unit.facts = facts
        if unit.data or self.tree is None:
            return facts
        return self.collect_facts(unit, facts, frozenset(), ())

    def view_facts(self, unit, view):
        """程序入口带参数（文件头「程序入口带参数」）：照 view（已知值表，frozenset 的 (名字, 值)）剪过的这个单元读到的东西；同一单元同一张表只算一次。"""
        cache = self.__dict__.setdefault("_view_facts", {})
        key = (id(unit), view)
        if key not in cache:
            facts = PythonFacts()
            if not unit.data and self.tree is not None:
                skipped, extra, calls, tables = self.narrowing_plan(unit, dict(view))
                self.collect_facts(unit, facts, skipped, extra)
                facts.narrowed_calls, facts.shallow_tables = calls, tables
            cache[key] = facts
        return cache[key]

    @staticmethod
    def reads_docstring(node, augmented, breaks):
        """__doc__ 这一处（名字或属性）读不读文档串：ast.Load 读，X.__doc__ += … 的目标读（augmented 是这个单元里各个 AugAssign 目标的 id）；
        X.__doc__ = … 与 del 只写不读。弄坏开关 python-docstring-writes-counted：写也算读；python-docstring-augmented-assignment-not-read：只认 ast.Load。"""
        if "python-docstring-writes-counted" in breaks:
            return True
        if isinstance(node.ctx, ast.Load):
            return True
        return id(node) in augmented and "python-docstring-augmented-assignment-not-read" not in breaks

    def collect_facts(self, unit, facts, skipped, extra):
        """单元读到的东西填进 facts、交回它：skipped（按 id）里的节点连同底下的不看，extra 是另外要看的节点（分派表取到的那一项）。"""
        breaks = self.index.breaks
        nodes = list(walk_nodes_skipping(unit.nodes or [], skipped)) + list(walk_nodes(extra)) if skipped or extra else list(walk_nodes(unit.nodes or []))
        if unit.kind == "import":
            for name, binding in self.local_import_bindings(unit).items():
                if name == unit.name or unit.name == "*":
                    facts.import_targets += self.binding_targets(binding, unit, facts)
        # 只查询固定的 __file__ 与直接读 __file__ 一样，不会按字符串挑到任意函数。
        # 仍遍历外层表达式；别的 globals()（动态键、传参、默认值）照旧拒算。
        file_queries = set()
        if self.binding("globals", unit) is None:
            for query in nodes:
                if (isinstance(query, ast.Call) and isinstance(query.func, ast.Attribute) and query.func.attr == "get"
                        and len(query.args) == 1 and not query.keywords
                        and isinstance(query.args[0], ast.Constant) and query.args[0].value == "__file__"):
                    receiver = query.func.value
                    if (isinstance(receiver, ast.Call) and isinstance(receiver.func, ast.Name) and receiver.func.id == "globals"
                            and not receiver.args and not receiver.keywords):
                        file_queries.add(id(receiver))
            if file_queries:
                facts.names.add("__file__")
        # globals()[键] 与 globals().update(…)（文件头「globals 的下标与 update」）：读连名字，写不连，都不拒算
        if self.binding("globals", unit) is None and "python-globals-subscripts-refused" not in breaks:
            for query in nodes:
                if isinstance(query, ast.Subscript):
                    receiver, key, written = query.value, query.slice, isinstance(query.ctx, (ast.Store, ast.Del))
                elif isinstance(query, ast.Call) and isinstance(query.func, ast.Attribute) and query.func.attr == "update":
                    receiver, key, written = query.func.value, None, True
                else:
                    continue
                if not (isinstance(receiver, ast.Call) and isinstance(receiver.func, ast.Name) and receiver.func.id == "globals"
                        and not receiver.args and not receiver.keywords):
                    continue
                file_queries.add(id(receiver))
                if not written:
                    read = {key.value} if isinstance(key, ast.Constant) and isinstance(key.value, str) else self.fold(key, self.scope_of(query, unit))
                    facts.names |= set(read) if read is not None else set(self.by_name)
        dropped = set()
        if "python-dispatch-tables-ignored" in breaks:
            # 弄坏开关：分派表（字典字面量）里当值写的函数名不连边
            dropped = {id(value) for node in nodes if isinstance(node, ast.Dict)
                       for value in node.values if isinstance(value, ast.Name)}
        # 函数里读局部量的名字不连模块里的同名单元（文件头「函数里读局部量的名字」）
        local_reads = local_name_reads(unit) if "python-local-names-resolved" not in breaks else ()
        # 读文档串只认读（文件头「文档串的读法」）：ast.Load，加 X.__doc__ += … 的目标（先读后写）；X.__doc__ = … 只写、不读
        augmented = {id(node.target) for node in nodes if isinstance(node, ast.AugAssign)}
        for node in nodes:
            if isinstance(node, ast.Name):
                if isinstance(node.ctx, (ast.Load, ast.Del)) and id(node) not in dropped and id(node) not in local_reads:
                    facts.names.add(node.id)
                if node.id == "__doc__" and self.reads_docstring(node, augmented, breaks):
                    facts.doc_reads.append("<module>")
            elif isinstance(node, ast.Attribute):
                facts.attributes.append((dotted_name(node.value), node.attr))
                if node.attr == "__doc__" and self.reads_docstring(node, augmented, breaks):
                    base = dotted_name(node.value)
                    facts.doc_reads.append(base if base else "*")
            elif isinstance(node, (ast.Import, ast.ImportFrom)) and unit.kind != "import":
                table = {}
                self.bind(node, table)
                for binding in table.values():
                    facts.import_targets += self.binding_targets(binding, unit, facts)
            elif isinstance(node, ast.Call):
                if id(node) not in file_queries:
                    # 套在单元里的函数里的调用：名字照那一层作用域折（scope_of）
                    self.call_facts(node, self.scope_of(node, unit), facts, breaks)
            elif isinstance(node, ast.Constant) and isinstance(node.value, str):
                facts.strings.add(node.value)
            elif isinstance(node, ast.Assign) and any(isinstance(target, ast.Subscript) and self.canonical(target.value, unit) == "sys.path"
                                                      for target in node.targets):
                elements = list(node.value.elts) if isinstance(node.value, (ast.List, ast.Tuple)) else [None]
                if any(element is None or self.path_values(element, unit, directory=True) is None for element in elements):
                    self.refuse(facts, "sys-path-unfolded", node)
        return facts

    # 程序入口带参数（文件头「程序入口带参数」）

    def narrowing_plan(self, unit, bindings):
        """照已知值表剪这个单元：交 (要跳过的节点 id 集合, 分派表取到的那几项的值节点, 带着已知实参往下剪的调用 [(函数单元, 已知值表)],
        只问键在不在的字典字面量的赋值单元)。只剪函数体与 main-guard；套在里面的函数不剪（那一层的名字另有绑定）。"""
        skipped, extra, calls, tables = set(), [], [], []
        if isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            self.prune_statements(unit.node.body, unit, bindings, skipped)
        elif unit.kind == "main-guard":
            for guard in unit.nodes or []:
                if isinstance(guard, ast.If):
                    self.prune_statements(guard.body, unit, bindings, skipped)
        else:
            return skipped, extra, calls, tables
        for node in walk_nodes_skipping(unit.nodes or [], skipped):
            if self.scope_of(node, unit) is not unit:
                continue
            if isinstance(node, ast.Compare) and len(node.ops) == 1 and isinstance(node.ops[0], (ast.In, ast.NotIn)) \
                    and isinstance(node.comparators[0], ast.Name) and "python-membership-tables-followed" not in self.index.breaks:
                # <词> in 表：只用到表的键，表的赋值照 SHALLOW_IMPORT_VIEW 收、值里写的函数名不连（文件头「程序入口带参数」分派表）
                found = self.module_dictionary(node.comparators[0].id, unit)
                if found is not None:
                    skipped.add(id(node.comparators[0]))
                    tables.append(found[1])
            elif isinstance(node, ast.Call):
                callee = None
                if isinstance(node.func, ast.Name):
                    callee = self.narrowable_callee(node.func.id, unit)
                elif isinstance(node.func, ast.Subscript):
                    entry = self.table_entry(node.func, unit, bindings)
                    if entry is not None and isinstance(entry[1], ast.Name):
                        callee = self.narrowable_callee(entry[1].id, None)
                view = self.call_view(callee, node, unit, bindings) if callee is not None else None
                if view is not None:
                    skipped.add(id(node.func))   # 被调的那一个按 view 另收（ScriptClosure.python_edges），名字不当普通引用
                    calls.append((callee, view))
            elif isinstance(node, ast.Subscript) and isinstance(node.ctx, ast.Load):
                entry = self.table_entry(node, unit, bindings)
                if entry is not None:
                    skipped.add(id(node.value))   # 不连整张表，只连取到的那一项
                    if entry[1] is not None:
                        extra.append(entry[1])
        return skipped, extra, calls, tables

    def prune_statements(self, statements, unit, bindings, skipped):
        """一串语句里判得出走不到的记进 skipped；交这一串是不是一定在这里收尾（return、raise、sys.exit、os._exit，或两支都收尾的 if）。"""
        reachable, ends = True, False
        for statement in statements:
            if not reachable:
                skipped.add(id(statement))
                continue
            if isinstance(statement, ast.If):
                decided = self.known_condition(statement.test, unit, bindings)
                if decided is True:
                    skipped.update(id(item) for item in statement.orelse)
                    branch_ends = self.prune_statements(statement.body, unit, bindings, skipped)
                elif decided is False:
                    skipped.update(id(item) for item in statement.body)
                    branch_ends = self.prune_statements(statement.orelse, unit, bindings, skipped)
                else:
                    body_ends = self.prune_statements(statement.body, unit, bindings, skipped)
                    branch_ends = self.prune_statements(statement.orelse, unit, bindings, skipped) and body_ends
                if branch_ends:
                    reachable, ends = False, True
            elif isinstance(statement, (ast.Return, ast.Raise)) or (
                    isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Call)
                    and self.canonical(statement.value.func, unit) in ("sys.exit", "os._exit")):
                reachable, ends = False, True
            elif isinstance(statement, (ast.For, ast.AsyncFor, ast.While)):
                self.prune_statements(statement.body, unit, bindings, skipped)
                self.prune_statements(statement.orelse, unit, bindings, skipped)
            elif isinstance(statement, (ast.With, ast.AsyncWith)):
                if self.prune_statements(statement.body, unit, bindings, skipped):
                    reachable, ends = False, True
            elif isinstance(statement, ast.Try):
                self.prune_statements(statement.body, unit, bindings, skipped)
                for handler in statement.handlers:
                    self.prune_statements(handler.body, unit, bindings, skipped)
                self.prune_statements(statement.orelse, unit, bindings, skipped)
                self.prune_statements(statement.finalbody, unit, bindings, skipped)
        return ends

    def known_value(self, node, unit, bindings, depth=0):
        """程序入口带参数时一个表达式的已知值：串、KnownWords；不知道交 None。"""
        if depth > 8 or node is None:
            return None
        if isinstance(node, ast.Constant):
            return node.value if isinstance(node.value, str) else None
        if isinstance(node, (ast.List, ast.Tuple)):
            if any(isinstance(element, ast.Starred) for element in node.elts):
                return None
            values = [self.known_value(element, unit, bindings, depth + 1) for element in node.elts]
            return KnownWords(tuple(value if isinstance(value, str) else None for value in values), True)
        if isinstance(node, (ast.Name, ast.Attribute)) and self.canonical(node, unit) == "sys.argv":
            return bindings.get(ARGV_BINDING)
        if isinstance(node, ast.Name):
            if node.id in bindings:
                return bindings[node.id]
            if unit is not None and unit.kind == "main-guard":
                assigned = guard_assignments(node.id, unit)
            elif unit is not None and isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                assigned = self.local_assignments(unit).get(node.id) or []
            else:
                assigned = []
            if len(assigned) == 1 and assigned[0] is not None and not isinstance(assigned[0], ElementsOf):
                return self.known_value(assigned[0], unit, bindings, depth + 1)
            return None
        if isinstance(node, ast.Subscript):
            base = self.known_value(node.value, unit, bindings, depth + 1)
            if not isinstance(base, KnownWords):
                return None
            index = node.slice
            if isinstance(index, ast.Slice):
                lower = 0 if index.lower is None else literal_integer(index.lower)
                upper = None if index.upper is None else literal_integer(index.upper)
                if index.step is not None or lower is False or upper is False:
                    return None
                return base.sliced(lower, upper)
            if isinstance(index, ast.Constant) and type(index.value) is int:
                return base.index(index.value)
        return None

    def known_members(self, node, unit, bindings):
        """in 右边的已知成员：元素都是字面串的列表、元组、集合，键都是字面串的字典字面量（本模块只赋过一次的也算），长度知道、词都知道的一串；判不出交 None。"""
        if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
            if all(isinstance(element, ast.Constant) and isinstance(element.value, str) for element in node.elts):
                return {element.value for element in node.elts}
            return None
        keys = None
        if isinstance(node, ast.Dict):
            keys = node.keys
        elif isinstance(node, ast.Name):
            found = self.module_dictionary(node.id, unit)
            keys = found[0].keys if found is not None else None
        if keys is None:
            value = self.known_value(node, unit, bindings)
            if isinstance(value, KnownWords) and value.exact and None not in value.words:
                return set(value.words)
            return None
        if all(isinstance(key, ast.Constant) and isinstance(key.value, str) for key in keys):
            return {key.value for key in keys}
        return None

    def known_condition(self, test, unit, bindings):
        """if 的条件照已知值判：True / False；判不出交 None。"""
        if isinstance(test, ast.BoolOp):
            results = [self.known_condition(value, unit, bindings) for value in test.values]
            if isinstance(test.op, ast.And):
                return False if False in results else (True if all(result is True for result in results) else None)
            return True if True in results else (False if all(result is False for result in results) else None)
        if isinstance(test, ast.UnaryOp) and isinstance(test.op, ast.Not):
            result = self.known_condition(test.operand, unit, bindings)
            return None if result is None else not result
        if isinstance(test, ast.Compare) and len(test.ops) == 1:
            operator, right = test.ops[0], test.comparators[0]
            if isinstance(operator, (ast.Eq, ast.NotEq)):
                equal = known_equal(self.known_value(test.left, unit, bindings), self.known_value(right, unit, bindings))
                return None if equal is None else (equal if isinstance(operator, ast.Eq) else not equal)
            if isinstance(operator, (ast.In, ast.NotIn)):
                left = self.known_value(test.left, unit, bindings)
                members = self.known_members(right, unit, bindings) if isinstance(left, str) else None
                return None if members is None else ((left in members) == isinstance(operator, ast.In))
            return None
        value = self.known_value(test, unit, bindings)
        if isinstance(value, str):
            return bool(value)
        if isinstance(value, KnownWords):
            return True if value.words else (False if value.exact else None)
        return None

    def table_entry(self, subscript, unit, bindings):
        """表[已知的串]，表是本模块只赋过一次、键都是字面串的字典字面量：交 (表的赋值单元, 那一项的值节点；表里没有这个键是 None)；别的交 None。"""
        if not isinstance(subscript.value, ast.Name):
            return None
        key = self.known_value(subscript.slice, unit, bindings)
        if not isinstance(key, str):
            return None
        found = self.module_dictionary(subscript.value.id, unit)
        if found is None or not all(isinstance(item, ast.Constant) and isinstance(item.value, str) for item in found[0].keys):
            return None
        dictionary, assignment = found
        return assignment, next((value for item, value in zip(dictionary.keys, dictionary.values) if item.value == key), None)

    def narrowable_callee(self, name, unit):
        """本模块里恰好一个、调用处没另绑这个名字的模块级函数 → 它的单元；别的 → None。"""
        if unit is not None and (self.local_bindings(unit).get(name) is not None or (
                isinstance(unit.node, (ast.FunctionDef, ast.AsyncFunctionDef)) and name in self.local_assignments(unit))):
            return None
        candidates = [candidate for candidate in self.by_name.get(name, ()) if candidate.kind == "def" and candidate.container is None
                      and isinstance(candidate.node, (ast.FunctionDef, ast.AsyncFunctionDef))]
        return candidates[0] if len(candidates) == 1 and name not in self.body_defines else None

    def call_view(self, callee, call, unit, bindings):
        """调 callee 的这一处带了哪些已知实参 → 已知值表（frozenset 的 (参数名, 值)）；一个都没有、带 * / ** 的交 None；callee 里给那个参数另赋过值的不带。"""
        if any(isinstance(item, ast.Starred) for item in call.args) or any(keyword.arg is None for keyword in call.keywords):
            return None
        arguments = callee.node.args
        positional = [argument.arg for argument in arguments.posonlyargs + arguments.args]
        named = set(positional) | {argument.arg for argument in arguments.kwonlyargs}
        reassigned = callee.module.local_assignments(callee)
        pairs = list(zip(positional, call.args)) + [(keyword.arg, keyword.value) for keyword in call.keywords if keyword.arg in named]
        known = [(name, value) for name, value in ((name, self.known_value(argument, unit, bindings)) for name, argument in pairs)
                 if value is not None and name not in reassigned]
        return frozenset(known) if known else None

    def table_reference_names(self, unit):
        """import 时执行的赋值（文件头「程序入口带参数」）：值里只当字典值、列表、元组、集合元素写的名字（import 时不调它们）；别的单元交空集合。"""
        if unit.cache is None:
            unit.cache = {}
        if "table_references" not in unit.cache:
            elements, others = set(), set()

            def visit(node, element):
                if isinstance(node, ast.Name):
                    (elements if element else others).add(node.id)
                elif isinstance(node, ast.Dict):
                    for key in node.keys:
                        if key is not None:
                            visit(key, False)
                    for value in node.values:
                        visit(value, True)
                elif isinstance(node, (ast.List, ast.Tuple, ast.Set)):
                    for item in node.elts:
                        visit(item, True)
                else:
                    for child in ast.iter_child_nodes(node):
                        visit(child, False)
            if unit.kind == "assign" and isinstance(unit.node, (ast.Assign, ast.AnnAssign)) and unit.node.value is not None:
                visit(unit.node.value, False)
            unit.cache["table_references"] = frozenset(elements - others)
        return unit.cache["table_references"]

    def outside_module(self, node, unit):
        """node 是 import 进来的标准库或外部模块（os、subprocess…）：getattr(它, <变量>) 取到的只会是那个模块里的东西，够不到仓里的代码。"""
        if "python-getattr-stdlib-refused" in self.index.breaks or not isinstance(node, ast.Name):
            return False
        binding = self.binding(node.id, unit)
        if binding is None or binding[0] != "import":
            return False
        return self.resolve_module(binding[1], 0)[0] in ("stdlib", "external")

    def local_import_bindings(self, unit):
        table = {}
        self.bind(unit.node, table)
        return table

    def refuse(self, facts, form, node):
        facts.refusals.append(self.refusal(form, node.lineno))

    def source_file(self):
        """这份模块算在哪份文件上：内联脚本算正文写在的那份（InlineScript 的 body_files；正文写在起它的那份文件里、或找不到出处的算起它的那份）。"""
        if self.inline is None:
            return self.path
        return dict(self.inline.body_files).get(self.text, self.inline.file)

    def registered_spec_path(self, location):
        """spec_from_file_location 折不出的路径在 SPEC_PATH_ARGUMENTS 里（文件头「载入路径的登记」）→ {根里那一份的绝对路径}；没登记、登记的文件不在 → None。"""
        target = SPEC_PATH_ARGUMENTS.get((self.source_file(), ast.unparse(location)))
        if target is None or "spec-path-arguments-refused" in self.index.breaks:
            return None
        targets = (target,) if isinstance(target, str) else tuple(target)
        if "spec-path-tuple-first-only" in self.index.breaks:
            targets = targets[:1]
        if not all(os.path.isfile(os.path.join(self.index.root, item)) for item in targets):
            return None
        return {posixpath.join(self.index.root, item) for item in targets}
