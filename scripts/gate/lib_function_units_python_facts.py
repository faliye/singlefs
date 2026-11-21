"""Python 切单元用的小函数（文档串、绑定名、局部名、调用点）、折叠用的已知值（KnownWords）、PythonFacts 与单元收集用的常量。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import collections
import copy
from lib_function_units_lexer import active_breaks
from lib_function_units_base import FOLD_LIMIT, OPAQUE, POSITIONAL, SHELL_OPEN_TAIL, ValueElement, sha256_text


# ── Python ──────────────────────────────────────────────────────────────────

SUBPROCESS_CALLS = frozenset(("subprocess.run", "subprocess.call", "subprocess.check_call", "subprocess.check_output", "subprocess.Popen"))
EXEC_CALLS = frozenset(("os.execv", "os.execvp", "os.execve", "os.execvpe"))   # 第二个参数是 argv
SHELL_STRING_CALLS = frozenset(("os.system", "os.popen"))                       # 参数是一段交给 sh -c 的串
SOURCE_READERS = frozenset(("open", "io.open", "tokenize.open", "inspect.getsource", "inspect.getsourcelines", "linecache.getlines"))
PATH_TRANSFORMS = frozenset(("os.path.dirname", "os.path.abspath", "os.path.realpath", "os.path.normpath", "os.path.basename",
                             "posixpath.dirname", "posixpath.normpath", "posixpath.basename", "str", "os.fspath"))
PATH_CONSTRUCTORS = frozenset(("pathlib.Path", "pathlib.PurePath", "pathlib.PurePosixPath", "pathlib.PosixPath"))
# 新建临时目录、临时文件的调用：读它们底下的文件读的是这一趟自己写的（文件头「运行期读的数据文件」）
TEMPORARY_CALLS = frozenset(("tempfile.mkdtemp", "tempfile.mkstemp", "tempfile.gettempdir", "tempfile.TemporaryDirectory", "tempfile.NamedTemporaryFile",
                             "tempfile.TemporaryFile", "tempfile.SpooledTemporaryFile"))
# 路径对象上只换形状、不换出处的方法：前缀从哪来照点前面那个对象
PATH_METHODS = frozenset(("resolve", "absolute", "as_posix", "expanduser", "joinpath", "with_suffix", "with_name", "relative_to"))
DATA_FORM = "data-path-unfolded"
# 拷文件的调用（文件头「运行期读的数据文件」）：源照读文件认，折不出的那一截不拒算（拷进副本、样本树的那一份归起在副本上的那一趟管）
COPY_CALLS = frozenset(("shutil.copy", "shutil.copy2", "shutil.copyfile"))
# 门禁样本档的根：按尾巴认的读法不收这底下的文件（文件头「运行期读的数据文件」一）
STAGE_SAMPLE_DIRECTORY = ".claude/gate.d/fixtures/"
# 版本询问（文件头「版本询问」）：命令位置上的程序折不出、后面只有这里头的一个词的，记成工具链询问
TOOLCHAIN_QUERY_WORDS = frozenset(("-V", "--version", "-vV"))
# 工具链单元的种类与名字（文件头「版本询问」：只在弄坏开关 python-toolchain-queries-keyed 下收）
TOOLCHAIN_UNIT_KIND = "rust-toolchain"
TOOLCHAIN_QUERY_SECONDS = 60
# 登记表点名的程序（文件头「登记表点名的程序」）：起子进程的模块在模块级写的那张表的名字
REGISTRY_LISTED_PROGRAMS_TABLE = "REGISTRY_LISTED_PROGRAMS"
# 程序入口带参数（文件头「程序入口带参数」）：已知值表里 sys.argv 的名字（不是合法的 python 名字，撞不上参数名）
ARGV_BINDING = "<sys.argv>"
# 给了参数时 import 时执行的赋值只收赋值本身（PythonModule.table_reference_names）：ScriptClosure 里这一种走法记成这个视图
SHALLOW_IMPORT_VIEW = "import-time-shallow"
# 模块被载入时它的模块级 import 只执行被 import 的那份模块（连 module-body），绑定的名字有单元读到才连到那个名字的单元（ScriptClosure.module_entered）
IMPORT_TIME_BINDING_VIEW = "import-time-binding"
# 运行期读的文件、起子进程的命令里折不出的那一截，出处是这几样的算调用方交进来的输入（文件头「运行期读的数据文件」二、「调用方交进来的命令」）
CALLER_INPUT_ORIGINS = frozenset(("input", "temporary", "outside"))
PYTHON_BUILTIN_CALLS = frozenset(("open", "str", "getattr", "globals", "__import__"))


def is_docstring(statement):
    return isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Constant) and isinstance(statement.value.value, str)


def without_docstrings(node, keep):
    if keep:
        return node
    clone = copy.deepcopy(node)
    for inner in ast.walk(clone):
        if isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Module)) and inner.body and is_docstring(inner.body[0]):
            inner.body = inner.body[1:]
    return clone


def dump_fingerprint(node):
    return sha256_text(ast.dump(node, include_attributes=False))


def assignment_names(statement):
    targets = statement.targets if isinstance(statement, ast.Assign) else [statement.target]
    names = []
    for target in targets:
        if isinstance(target, ast.Name):
            names.append(target.id)
        elif isinstance(target, (ast.Tuple, ast.List)):
            names += [element.id for element in target.elts if isinstance(element, ast.Name)]
    return list(dict.fromkeys(names))


def import_bound_names(statement):
    names = []
    for alias in statement.names:
        if isinstance(statement, ast.Import):
            names.append(alias.asname or alias.name.split(".")[0])
        else:
            names.append(alias.asname or alias.name)
    return names


def is_main_name_test(test):
    return (isinstance(test, ast.Compare) and isinstance(test.left, ast.Name) and test.left.id == "__name__" and len(test.comparators) == 1
            and isinstance(test.comparators[0], ast.Constant) and test.comparators[0].value == "__main__")


def is_main_guard(statement, breaks=frozenset()):
    """只在这份文件被当程序起时才执行的那一段（文件头「Python 闭包：import 时执行的模块级语句」的 main-guard 两种写法）：
    if __name__ == "__main__":，与不带 else 的 if __name__ == "__main__" and <别的条件>:（and 从左往右短路，import 时头一项是假的，后面的条件与块都不执行）。
    and 写法里 __name__ 那一项不在最左、带 else 的认不出，留在 module-body 照收。弄坏开关 python-main-guard-conjunctions-unrecognized：只认头一种。
    头一种带 else（或 elif）的照认：PythonModule.split 只把 if 那一支切成 main-guard，else 那一支放进 module-body（被 import 时执行）。"""
    if not isinstance(statement, ast.If):
        return False
    test = statement.test
    if isinstance(test, ast.BoolOp) and isinstance(test.op, ast.And) and not statement.orelse \
            and "python-main-guard-conjunctions-unrecognized" not in breaks:
        test = test.values[0]
    return is_main_name_test(test)


def stored_names(node):
    names = set()
    for inner in ast.walk(node):
        if isinstance(inner, ast.Name) and isinstance(inner.ctx, ast.Store):
            names.add(inner.id)
        elif isinstance(inner, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            names.add(inner.name)
        elif isinstance(inner, (ast.Import, ast.ImportFrom)):
            names.update(import_bound_names(inner))
    return names


def dotted_name(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return None


class ElementsOf(ast.AST):
    """局部赋值表里的一项：for 循环变量取的是 sequence 里的元素（折叠时取遍那几个元素）。"""
    _fields = ("sequence",)


def bound_names(target):
    """赋值目标里重新绑定的名字：Name、拆包（Tuple / List / Starred）里的 Name；下标与属性目标不绑名字。"""
    if isinstance(target, ast.Name):
        return [target.id]
    if isinstance(target, ast.Starred):
        return bound_names(target.value)
    if isinstance(target, (ast.Tuple, ast.List)):
        return [name for element in target.elts for name in bound_names(element)]
    return []


def walk_nodes(nodes):
    for node in nodes:
        yield from ast.walk(node)


def function_local_names(function):
    """def 或 lambda 自己这一层作用域绑定的名字 → (局部名字, global 与 nonlocal 声明过的名字)。import 绑的不算（resolve_name 照 import 认）；推导式、lambda、
    套在里面的 def 与类体自成一层，里面的赋值不算这一层的（海象赋值同样不算，宁可多连）。"""
    arguments = function.args
    names = {argument.arg for argument in arguments.posonlyargs + arguments.args + arguments.kwonlyargs}
    names |= {argument.arg for argument in (arguments.vararg, arguments.kwarg) if argument is not None}
    declared = set()
    pending = list(function.body) if isinstance(function.body, list) else [function.body]
    while pending:
        node = pending.pop()
        if isinstance(node, (ast.Global, ast.Nonlocal)):
            declared.update(node.names)
        elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            names.add(node.name)
        elif not isinstance(node, (ast.Lambda, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
                names.add(node.id)
            elif isinstance(node, ast.ExceptHandler) and node.name:
                names.add(node.name)
            pending.extend(ast.iter_child_nodes(node))
    return names - declared, declared


def local_name_reads(unit):
    """单元里读局部量的那几个 ast.Name（按 id，文件头「函数里读局部量的名字」）；缓存在单元上。"""
    if unit.cache is None:
        unit.cache = {}
    if "local-name-reads" in unit.cache:
        return unit.cache["local-name-reads"]
    reads = set()

    def visit(node, scopes):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            arguments = node.args
            outer = list(arguments.defaults) + [default for default in arguments.kw_defaults if default is not None]
            if not isinstance(node, ast.Lambda):
                outer += node.decorator_list
            for item in outer:
                visit(item, scopes)
            names, declared = function_local_names(node)
            inner = tuple(scope - declared for scope in scopes) + (frozenset(names),)
            for statement in node.body if isinstance(node.body, list) else [node.body]:
                visit(statement, inner)
            return
        if isinstance(node, (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)):
            # 头一节的序列在外层作用域里求值，别的都在推导式这一层
            visit(node.generators[0].iter, scopes)
            inner = scopes + (frozenset(name for generator in node.generators for name in bound_names(generator.target)),)
            for child in ast.iter_child_nodes(node):
                if not isinstance(child, ast.comprehension):
                    visit(child, inner)
            for position, generator in enumerate(node.generators):
                if position:
                    visit(generator.iter, inner)
                for condition in generator.ifs:
                    visit(condition, inner)
            return
        if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Load, ast.Del)) and any(node.id in scope for scope in scopes):
            reads.add(id(node))
        for child in ast.iter_child_nodes(node):
            visit(child, scopes)
    for root in unit.nodes or []:
        visit(root, ())
    unit.cache["local-name-reads"] = frozenset(reads)
    return unit.cache["local-name-reads"]


def named_calls_in(nodes):
    """nodes 里认得出被调名字的 ast.Call：[(节点, 名字)]，次序同 nodes。"""
    found = []
    for node in nodes:
        if isinstance(node, ast.Call):
            callee = node.func.id if isinstance(node.func, ast.Name) else node.func.attr if isinstance(node.func, ast.Attribute) else None
            if callee:
                found.append((node, callee))
    return found


def partial_bound_call(call, called):
    """call 是 functools.partial(f, …) / partial(f, …)、f 的名字是 called：交回照 f(…) 写的那一处调用（实参与关键字去掉头一个 f），不是就 None。"""
    function = call.func
    named_partial = (isinstance(function, ast.Name) and function.id == "partial") or (
        isinstance(function, ast.Attribute) and function.attr == "partial" and isinstance(function.value, ast.Name) and function.value.id == "functools")
    if not named_partial or not call.args:
        return None
    target = call.args[0]
    name = target.id if isinstance(target, ast.Name) else target.attr if isinstance(target, ast.Attribute) else None
    if name != called:
        return None
    return ast.copy_location(ast.Call(func=target, args=list(call.args[1:]), keywords=list(call.keywords)), call)


def walk_nodes_skipping(nodes, skipped):
    """同 walk_nodes（每个根各自按层走），skipped 里（按 id）的节点连同它底下的都不走（程序入口带参数时剪掉的那几支）。"""
    for root in nodes:
        if id(root) in skipped:
            continue
        pending = collections.deque([root])
        while pending:
            node = pending.popleft()
            yield node
            pending.extend(child for child in ast.iter_child_nodes(node) if id(child) not in skipped)


class KnownWords(collections.namedtuple("KnownWords", "words exact")):
    """程序入口带参数时已知的一串词（文件头「程序入口带参数」）：words 逐个是词（不知道的写 None），exact 是长度知不知道（不知道的是开口的一串）。"""

    def index(self, position):
        """[k]：k 是整数；落在不知道的那一截上交 None。"""
        if position >= 0:
            return self.words[position] if position < len(self.words) else None
        return self.words[position] if self.exact and -position <= len(self.words) else None

    def sliced(self, lower, upper):
        """[lower:upper]（都是不小于 0 的整数或 None，不带步长）。"""
        if upper is None:
            return KnownWords(self.words[lower:], self.exact)
        if self.exact or upper <= len(self.words):
            return KnownWords(self.words[lower:upper], True)
        return KnownWords(self.words[lower:], False)


def launch_arguments(elements, values, start):
    """起 python 脚本的命令里脚本之后那几个元素 → KnownWords（文件头「程序入口带参数」子进程边）：折得出、恰好一个取值的是那个词，别的是 None；
    遇到 OPAQUE、* 展开、POSITIONAL（个数不知道）就到此为止、记成开口的一串。"""
    words = []
    for position in range(start, len(elements)):
        item = elements[position]
        if item is OPAQUE or (isinstance(item, ValueElement) and item.values is POSITIONAL) or (
                isinstance(item, tuple) and isinstance(item[0], ast.Starred)):
            return KnownWords(tuple(words), False)
        value = values[position] if position < len(values) else None
        words.append(next(iter(value)) if value is not None and len(value) == 1 else None)
    return KnownWords(tuple(words), True)


def shell_launch_choices(elements, values, start):
    """python 起 shell 的命令里脚本之后那几个元素（* 展开已由 command_alternatives 摊开成一种种取法）→ 每一种取法的词（{(词…)}；折出几个取值的逐个展开）；
    折不出的元素之前有字面词的，字面词接 SHELL_OPEN_TAIL（文件头「shell 起法带字面参数」）；头一个元素就折不出、展开超过 FOLD_LIMIT 种 → None（那份脚本体整份收）。"""
    choices = {()}
    for position in range(start, len(elements)):
        value = values[position] if position < len(values) else None
        if not value:
            if position > start and "shell-open-tail-unnarrowed" not in active_breaks():
                return {choice + (SHELL_OPEN_TAIL,) for choice in choices}
            return None
        choices = {choice + (word,) for choice in choices for word in value}
        if len(choices) > FOLD_LIMIT:
            return None
    return choices


def known_equal(left, right):
    """两个已知值相不相等：True / False；判不出交 None。"""
    if left is None or right is None:
        return None
    if isinstance(left, str) and isinstance(right, str):
        return left == right
    if isinstance(left, str) or isinstance(right, str):
        return False
    if left.exact and right.exact and len(left.words) != len(right.words):
        return False
    for mine, theirs in zip(left.words, right.words):
        if mine is not None and theirs is not None and mine != theirs:
            return False
    if left.exact and right.exact and None not in left.words and None not in right.words:
        return True
    return None


def literal_integer(node):
    """不小于 0 的整数字面量 → 它；不写（None）→ None 交 None 的区分由调用方做；别的 → False。"""
    if isinstance(node, ast.Constant) and type(node.value) is int and node.value >= 0:
        return node.value
    return False


def unique_alternatives(alternatives):
    """命令的几种取法去重（同一串元素节点算一种），次序不变。"""
    out, seen = [], set()
    for alternative in alternatives:
        key = tuple(id(item[0]) if isinstance(item, tuple) else id(item) for item in alternative)
        if key not in seen:
            seen.add(key)
            out.append(alternative)
    return out


def paired_elements(target, value):
    """a, b = x, y：→ [(名字, 对上的值节点)…]；两边不是等长的元组 / 列表、有 *、目标里有不是名字的 → None。
    a, b = s[k:k+2] 照 a = s[k]、b = s[k+1] 对上（sliced_elements；文件头「起代码的写法逐种认」）。"""
    if isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, ast.Subscript) and isinstance(value.slice, ast.Slice) \
            and "python-slice-unpacking-unpaired" not in active_breaks():
        return sliced_elements(target, value)
    if not (isinstance(target, (ast.Tuple, ast.List)) and isinstance(value, (ast.Tuple, ast.List)) and len(target.elts) == len(value.elts)):
        return None
    if any(not isinstance(item, ast.Name) for item in target.elts) or any(isinstance(item, ast.Starred) for item in value.elts):
        return None
    return [(item.id, element) for item, element in zip(target.elts, value.elts)]


def sliced_elements(target, value):
    """a, b = s[k:k+2]：→ [(a, s[k]), (b, s[k+1])]，下标节点是另造的 s[<整数>]（挂在切片节点上，同一处拆包每次交同一批）。
    下界不写或是不小于 0 的整数字面量、不带步长、上界不写或等于下界加目标个数、目标全是名字，才对上；别的 → None。
    上界不写时 s 的长度对不上，跑起来就抛 ValueError、走不到后面，所以照对上算。"""
    bounds, names = value.slice, target.elts
    if bounds.step is not None or not names or any(not isinstance(item, ast.Name) for item in names):
        return None
    lower = 0 if bounds.lower is None else (
        bounds.lower.value if isinstance(bounds.lower, ast.Constant) and type(bounds.lower.value) is int else None)
    if lower is None or lower < 0:
        return None
    if bounds.upper is not None and not (isinstance(bounds.upper, ast.Constant) and type(bounds.upper.value) is int
                                         and bounds.upper.value == lower + len(names)):
        return None
    elements = getattr(value, "sliced_elements", None)
    if elements is None:
        elements = [ast.copy_location(ast.Subscript(value=value.value, slice=ast.copy_location(ast.Constant(value=lower + offset), value),
                                                    ctx=ast.Load()), value) for offset in range(len(names))]
        value.sliced_elements = elements   # 不在 _fields 里：ast.dump 与 walk_nodes 都看不见它，指纹不变
    return [(item.id, element) for item, element in zip(names, elements)]


OWN_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda, ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)


def walk_function_statements(function):
    """函数体里的每个节点，照源码次序，不进套在里面的函数、类、lambda 与推导式（它们自成作用域）。"""
    def visit(node):
        yield node
        for child in ast.iter_child_nodes(node):
            if not isinstance(child, OWN_SCOPES):
                yield from visit(child)
    for statement in function.body:
        if not isinstance(statement, OWN_SCOPES):
            yield from visit(statement)


def guard_assignments(name, unit):
    """main-guard 单元里给 name 的赋值的值（照写的次序）；单元不是 main-guard 或没赋过 → []。"""
    if unit is None or unit.kind != "main-guard":
        return []
    return [node.value for node in walk_nodes(unit.nodes or []) if isinstance(node, (ast.Assign, ast.AnnAssign)) and node.value is not None
            and any(isinstance(target, ast.Name) and target.id == name for target in (node.targets if isinstance(node, ast.Assign) else [node.target]))]


def enclosing_unit(unit):
    """PythonModule.nested_unit 造的作用域单元：它外面那一层（模块级的单元交 None）。"""
    return unit.cache.get("enclosing") if unit is not None and unit.cache else None


def walk_targets(statement):
    """for 的循环变量、with 的 as 绑的那几个名字。"""
    targets = [statement.target] if isinstance(statement, (ast.For, ast.AsyncFor)) else [item.optional_vars for item in statement.items if item.optional_vars]
    for target in targets:
        yield from (inner for inner in ast.walk(target) if isinstance(inner, ast.Name))


class PythonFacts:
    """inline_scripts：起的内联脚本（InlineScript，python 的在闭包里切成单元、shell 的当一段 bash 认）；whole_modules：整份连上的模块
    （-m unittest 点名的测试模块）；handled：每个起子进程的调用各记一条认成了什么（只给量全仓的人看，不进判定）。"""
    __slots__ = ("names", "attributes", "loads", "program_links", "doc_reads", "source_read", "strings", "refusals", "import_targets", "inline_scripts",
                 "whole_modules", "handled", "data_reads", "toolchain_queries", "caller_launches", "narrowed_calls", "registry_reads",
                 "program_arguments", "shallow_tables", "shell_launch_words", "built_tools", "contained_loads", "module_object_texts")

    def __init__(self):
        self.names, self.attributes, self.loads, self.program_links = set(), [], [], []
        # loads 里载入的模块值出不了这份模块的那几处（文件头「按路径载入的模块只连载入方调到的属性」，PythonModule.loaded_value_contained）
        self.contained_loads = []
        # exec 进模块对象的内联正文（文件头「exec 进模块对象」）：[(InlineScript, 模块对象出不了这份模块)]；源码是读出的文件的进 loads
        self.module_object_texts = []
        self.doc_reads, self.source_read, self.strings, self.refusals, self.import_targets = [], False, set(), [], []
        self.inline_scripts, self.whole_modules, self.handled = [], [], []
        # 运行期读的数据文件（文件头「运行期读的数据文件」）：[(取法集合, 行, 是不是拷文件的源)]，取法见 PythonModule.data_read_origins
        self.data_reads = []
        # 版本询问的行（文件头「版本询问」）；调用方交进来的命令：[(拒算, 折不出那几截的出处)]，收闭包时判是不是入口参数（ScriptClosure.caller_launch_refusals）
        self.toolchain_queries, self.caller_launches = [], []
        # 程序入口带参数（文件头「程序入口带参数」）：带着已知实参往下剪的调用 [(被调的函数单元, 已知值表)]；
        # 子进程边带过去的参数：{被起的文件: {KnownWords 或 None（不知道）…}}
        self.narrowed_calls, self.program_arguments = [], {}
        # python 起 shell 的每一处带的字面参数（文件头「shell 起法带字面参数」）：{被起的文件: [{(词…)} 或 None（有一处折不出）…]}
        self.shell_launch_words = {}
        # 程序入口带参数时只问键在不在（<词> in 表）的那几张本模块字典字面量的赋值单元：照 SHALLOW_IMPORT_VIEW 收（值里的函数名不连）
        self.shallow_tables = []
        # 登记表点名的程序（文件头「登记表点名的程序」）读的登记表（相对根的路径）
        self.registry_reads = []
        # 起编出来的工具的那几处（BUILT_TOOL_LAUNCHES）：编它的源码目录（相对根），收闭包时收成数据目录
        self.built_tools = []
