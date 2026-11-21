"""各语言共用的常量、错误种类、拒算与单元（Refusal、Unit）、数据文件单元、命令词与起法的认法（外部程序、解释器、命令包装）、库所在仓的根（LIBRARY_ROOT）。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import collections
import hashlib
import os
import posixpath
import re
from lib_function_units_lexer import active_breaks


KEY_FORM = "singlefs-function-units-v1"
# 判定种类（文件头「判定种类」）：与 crates/singlefs-unit-graph 的 --judgment-kind 同两个写法
FORMAL_VERIFICATION = "formal-verification"
PRODUCT_BEHAVIOR = "product-behavior"
JUDGMENT_KINDS = (FORMAL_VERIFICATION, PRODUCT_BEHAVIOR)
UNIT_GRAPH_DIRECTORY_VARIABLE = "FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY"
UNIT_GRAPH_KEPT_PROGRAMS = 3
VERUS_GHOST_CODE_UNPARSED = "verus-ghost-code-unparsed"

# 函数里出现这几个词算起了子进程：此时 current_exe() 折不出要点名的那一条测试，拒算 current-exe（文件头「测试二进制起自己」）
SPAWNING_WORDS = frozenset(("Command", "spawn", "posix_spawn", "fork", "execv", "execve", "execvp"))
# 属性里没有 link 的 extern "C" 块里只声明这几个符号时切分不拒算（文件头「extern 块」；逐项与 crates/singlefs-unit-graph 的 SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 相同，自证读那份源码比）
C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS = ("pwrite", "fdatasync")


def c_library_calls_allowed_in_extern_blocks():
    """上面那张名单；弄坏开关 rust-extern-system-library-list-short 下少最后一项。"""
    if "rust-extern-system-library-list-short" in active_breaks():
        return C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS[:-1]
    return C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS


# 构建脚本叫 cargo 多链接一个库的指令名（cargo:rustc-link-lib=…、cargo::rustc-link-lib=… 里都有它）；crates/singlefs-unit-graph 的 RUSTC_LINK_LIBRARY_INSTRUCTION
RUSTC_LINK_LIBRARY_INSTRUCTION = "rustc-link-lib"
# 构建脚本叫 cargo 往链接里多放库或链接参数的指令名（cargo:rustc-link-arg=-l…、cargo:rustc-flags=-l … 一样多链接一个库；rustc-flags 只认 -l 与 -L，这里写了就算）
LINK_INSTRUCTION_NAMES = (RUSTC_LINK_LIBRARY_INSTRUCTION, "rustc-link-arg", "rustc-flags")
# 构建脚本与构建依赖的库代码打指令时，指令写在字面串里（cargo:rustc-link-lib=…、cargo::rustc-link-lib=…）；注释里提到指令名、没有 cargo: 前缀的裸词不算
LINK_INSTRUCTION_IN_A_STRING = re.compile(r"cargo::?rustc-(?:link-lib|link-arg|flags)")
LINK_LIBRARY_INSTRUCTION_IN_A_STRING = re.compile(r"cargo::?rustc-link-lib")
SINGLE_COLON_LINK_INSTRUCTION_IN_A_STRING = re.compile(r"cargo:rustc-(?:link-lib|link-arg|flags)")
SINGLE_COLON_LINK_LIBRARY_INSTRUCTION_IN_A_STRING = re.compile(r"cargo:rustc-link-lib")


def link_instruction_pattern(breaks):
    """字面串里算「打了多链接一个库的指令」的写法（LINK_INSTRUCTION_IN_A_STRING）；弄坏开关 rust-extern-link-arguments-unchecked 下只认 rustc-link-lib，
    rust-extern-two-colon-instructions-ignored 下只认 cargo: 一个冒号的写法。"""
    only_library = "rust-extern-link-arguments-unchecked" in breaks
    if "rust-extern-two-colon-instructions-ignored" in breaks:
        return SINGLE_COLON_LINK_LIBRARY_INSTRUCTION_IN_A_STRING if only_library else SINGLE_COLON_LINK_INSTRUCTION_IN_A_STRING
    return LINK_LIBRARY_INSTRUCTION_IN_A_STRING if only_library else LINK_INSTRUCTION_IN_A_STRING
REFUSED_EXIT = 4
SHELL_TIMEOUT_SECONDS = 600
SNIPPET_CHARACTERS = 160
FOLD_LIMIT = 64
# 折出的取值超过 FOLD_LIMIT 种、那一处又没有收全的认法（整组、整个目录）时记的拒算形态（文件头「拒算」公共一行）：取值认得出、只是太多，不当推不出
FOLD_LIMIT_FORM = "fold-limit-exceeded"
# 清单目录拼出的目录绑到 let 变量、变量的用法再绑到变量，最多跟这么多层，再深的拒算（crates/singlefs-unit-graph 的 DEEPEST_DIRECTORY_BINDING_CHAIN 同值）
DEEPEST_MANIFEST_BINDING_CHAIN = 4
# fold_manifest_chain 交出的「这一处折不出」，记成一条 runtime-path 拒算
MANIFEST_FOLD_REFUSED = (None, None)
SKIPPED_DIRECTORIES = frozenset((".git", "target", "__pycache__", "node_modules"))
DEFAULT_SCRIPT_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", ".claude/scripts")
EXTERNAL_PROGRAMS = ("herd7", "busybox", "cargo")
EXTERNAL_PROGRAM_PREFIXES = ("qemu-system-",)
PYTHON_INTERPRETERS = frozenset(("python3", "python"))
SHELL_INTERPRETERS = frozenset(("bash", "sh"))
INLINE_FORM = "inline-script-not-literal"
COMMAND_FORM = "subprocess-command-unfolded"
# 只打印的命令：后面的词是消息，不是程序名（外部程序那一条不看它们）
MESSAGE_HEADS = frozenset(("echo", "printf", "die", "bad", "howto"))
# 包装命令：名字 → (带一个值的选项, 选项之后先跳过几个操作数)；包装之后的那个词在命令位置上
COMMAND_WRAPPERS = {
    "env": (frozenset(("-u", "--unset", "-C", "--chdir")), 0),
    "nice": (frozenset(("-n", "--adjustment")), 0),
    "ionice": (frozenset(("-c", "--class", "-n", "--classdata", "-p", "-P", "-u")), 0),
    "timeout": (frozenset(("-s", "--signal", "-k", "--kill-after")), 1),
    "nohup": (frozenset(), 0),
    "setsid": (frozenset(), 0),
    "stdbuf": (frozenset(("-i", "-o", "-e")), 0),
    "taskset": (frozenset(), 1),
    "chrt": (frozenset(), 1),
    "sudo": (frozenset(("-u", "-g", "-C", "-D", "-h", "-p", "-r", "-t", "-U", "-T")), 0),
    "xargs": (frozenset(("-a", "-d", "-E", "-I", "-L", "-n", "-P", "-s", "--arg-file", "--delimiter", "--max-args", "--max-procs", "--max-lines",
                         "--max-chars")), 0),
    "exec": (frozenset(("-a",)), 0),
    "command": (frozenset(), 0),
    "time": (frozenset(), 0),
}
PYTHON_VALUE_OPTIONS = frozenset(("-X", "-W", "--check-hash-based-pycs"))
SHELL_VALUE_OPTIONS = frozenset(("-o", "-O", "+o", "+O", "--rcfile", "--init-file"))
TEST_RUNNER_MODULES = frozenset(("unittest", "pytest"))
PREFLIGHT_MODULE = "singlefs-ai-sop/scripts/preflight.py"
DECLARED_CHECK = re.compile(r"^#\s*(?:admission|run-condition):\s*check\s+(.*?)(?:\s+::\s.*)?$")


def declared_checks(root, path, rows):
    """一份脚本文件头（#! 之后、第一行代码之前的注释块）里登记的 check 判据 → [InlineScript…]：preflight 开跑前在仓根用 bash -c 跑它们，
    环境里有 PREFLIGHT_SCRIPT 与 PREFLIGHT_SCRIPT_DIRECTORY（preflight-discipline.md「写法」）。"""
    out = []
    absolute = posixpath.join(root, path)
    environment = (("PREFLIGHT_SCRIPT", frozenset((absolute,))), ("PREFLIGHT_SCRIPT_DIRECTORY", frozenset((posixpath.dirname(absolute),))))
    for number, row in enumerate(rows, 1):
        if number == 1 and row.startswith("#!"):
            continue
        stripped = row.strip()
        if stripped and not stripped.startswith("#"):
            break
        found = DECLARED_CHECK.match(stripped)
        if found and found.group(1).strip():
            out.append(InlineScript("shell", (found.group(1).strip(),), (None,), (root,), path, number, snippet_of(row), environment))
    return out
SHELL_POSITIONAL_WORD = re.compile(r'^"?\$(?:\{(?:[@*]|[1-9][0-9]*)(?::[^}]*)?\}|[@*1-9])"?$')


def external_program_name(value, breaks):
    """一个词的字面值 → 它点名的登记外部程序名（等于 cargo / herd7 / busybox，或以 qemu-system- 起头、后面还有字）；不是 → None。
    词里有空白的（消息串、带空格的串）不算，只是前缀本身（qemu-system-）也不算。"""
    if not value:
        return None
    base = posixpath.basename(value)
    if "external-program-words-unfiltered" in breaks:
        return base if base in EXTERNAL_PROGRAMS or base.startswith(EXTERNAL_PROGRAM_PREFIXES) else None
    if any(character.isspace() for character in value):
        return None
    if base in EXTERNAL_PROGRAMS:
        return base
    if any(base.startswith(prefix) and len(base) > len(prefix) for prefix in EXTERNAL_PROGRAM_PREFIXES):
        return base
    return None


def command_positions(values):
    """一条命令里在命令位置上的那几个下标：0 起，经包装命令（env、nice、timeout、sudo、xargs、exec…）跳过它们的选项与操作数之后的那一个。
    values 是逐词字面值（折不出的是 None：它就是命令位置上的词，后面不再往下认）；command -v / -V 是查询，后面不算命令位置。"""
    positions, position = [], 0
    while position < len(values):
        positions.append(position)
        value = values[position]
        name = posixpath.basename(value) if value else None
        if name not in COMMAND_WRAPPERS:
            break
        value_options, operands = COMMAND_WRAPPERS[name]
        position += 1
        while position < len(values):
            option = values[position]
            if option is None:
                break
            if name == "command" and option in ("-v", "-V"):
                return positions
            if option == "--":
                position += 1
                break
            if name == "env" and ASSIGNMENT_WORD.match(option):
                position += 1
                continue
            if option.startswith("-") and len(option) > 1:
                position += 2 if option in value_options else 1
                continue
            break
        position += operands
    return positions


def interpreter_mode(values, position, python):
    """解释器词（values[position]）之后：→ (写法, 下标)。写法：script（下标是脚本那个词）、inline（-c，下标是 -c）、module（python 的 -m）、
    stdin（- 或 bash 的 -s：脚本从标准输入来）、syntax-only（bash -n：只查语法不跑）、no-run（--version、--help、python3 -V：打完就退）、
    bare（后面没有词了，下标是末尾）。
    合写的短选项（bash -ec、python3 -Bc）按最后一个字母认；带值的选项（bash -o x、python3 -X y）跳两格；折不出的词当脚本。"""
    argument = position + 1
    value_options = PYTHON_VALUE_OPTIONS if python else SHELL_VALUE_OPTIONS
    while argument < len(values):
        option = values[argument]
        if option is None:
            return "script", argument
        if option == "-":
            return "stdin", argument
        if option in ("--version", "--help") or (option[:1] == "-" and len(option) > 1 and set(option[1:]) <= ({"V", "h"} if python else set())):
            return "no-run", argument
        if option in value_options:
            argument += 2
            continue
        if option.startswith("--"):
            argument += 1
            continue
        if option[:1] in "-+" and len(option) > 1:
            letters = option[1:]
            if letters.isalnum():
                last = letters[-1]
                if last == "c":
                    return "inline", argument
                if python and last == "m":
                    return "module", argument
                if not python and last == "s":
                    return "stdin", argument
                if not python and "n" in letters:
                    return "syntax-only", argument
            argument += 1
            continue
        return "script", argument
    return "bare", argument


ARGV_SPAWNS = frozenset(("os.execvp", "os.execv", "os.execve", "os.execvpe", "subprocess.run", "subprocess.call", "subprocess.check_call",
                         "subprocess.check_output", "subprocess.Popen"))


def argv_command_offsets(text):
    """python 内联脚本正文里把 sys.argv[k:] 当命令起的（os.execvp(sys.argv[1], sys.argv[1:])、subprocess.run(sys.argv[2:])）→ 那几个 k：
    起它的那条命令里第 k 个参数起就是一条命令，在起它的那边照命令认。正文解析不了 → 空。"""
    try:
        tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return set()
    aliases = {"os": "os", "subprocess": "subprocess"}
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                aliases[alias.asname or alias.name] = alias.name
    offsets = set()
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name)):
            continue
        if f"{aliases.get(node.func.value.id, node.func.value.id)}.{node.func.attr}" not in ARGV_SPAWNS:
            continue
        for argument in node.args[:2]:
            if isinstance(argument, ast.Subscript) and isinstance(argument.value, ast.Attribute) and argument.value.attr == "argv" and \
                    isinstance(argument.slice, ast.Slice) and isinstance(argument.slice.lower, ast.Constant) and type(argument.slice.lower.value) is int \
                    and argument.slice.upper is None and argument.slice.step is None and argument.slice.lower.value >= 1:
                offsets.add(argument.slice.lower.value)
    return offsets


def descriptor_of(script):
    """脚本那个词是 /dev/fd/<N> 或 /dev/stdin（从文件描述符读脚本）→ N；不是 → None。"""
    if script == "/dev/stdin":
        return 0
    found = re.match(r"^/dev/fd/([0-9]+)$", script or "")
    return int(found.group(1)) if found else None


InlineScript = collections.namedtuple("InlineScript", "language texts argv bases file line snippet environment body_lines body_files listings",
                                      defaults=((), (), (), ()))
InlineScript.__doc__ = """内联脚本（python3 -c / bash -c 的串、喂给解释器标准输入的 here-doc、here-string）：texts 是它可能取到的几份正文，
argv 是 sys.argv[1:] / $1… 逐个折出来的值（frozenset，折不出 None），bases 是相对路径与 import 先试的目录（当时的工作目录），
file / line / snippet 是起它的那一行，environment 是同一条命令里给它设的环境变量（(名字, 取值或 None)…：
X=… python3、env X=… python3），body_lines 是 ((正文, 那份正文的头一行的行号)…)：正文出自原文里的一份 here-doc（直接喂给解释器的，
或先装进变量、$(cat <<词 …) 再交 -c 的）才有那一份的行号（ShellIndex.inline_body_lines）；正文里的拒算落在那个行号加正文里的偏移那一行，
没有行号的（字面 -c 串、原文里找不到那份正文）落在起它的那一行（PythonModule.refusal）。body_files 是 ((正文, 那份 here-doc 所在的文件)…)：
正文写在别的文件里（入口把程序装进变量、被 source 的库里起它）的才有，body_lines 的行号是那份文件里的，拒算落在那份文件里，
这份内联模块也算在那份文件上（PythonModule.source_file：按文件查的登记表照正文写在哪份文件里查）；没有的算在 file 上。
listings 是 ((argv 里的下标, frozenset(名字模式…))…)：那个参数是一份 find 清单（变量只赋成 $(find <目录> … -name <模式> … [| sort])，
ShellIndex.listing_patterns），每一行的文件名合其中一个模式；正文里 sys.argv[下标 + 1].split("\\n") 挑出来的名字照它认（PythonModuleDataReads.argv_listing）。"""
OPAQUE = object()
MEMO_IN_PROGRESS = object()   # PythonModule.memoized：这一项正在算
COMPREHENSION_UNPACKED = object()   # 推导式拆包绑的循环变量（for a, b in …）：PythonModule.comprehension_binding 交它，折不出
POSITIONAL = "\0positional"   # 内联脚本的参数是 "$@" / "$1" 一类：取值就是调用处那几个词（那边已照命令认过）


class ValueElement:
    """命令里一个已经折好的元素（内联脚本的 sys.argv[k:] 摊开的那几个）：values 是 frozenset 或 None。"""
    __slots__ = ("values",)

    def __init__(self, values):
        self.values = values


class FunctionUnitsError(Exception):
    """算不出：根不在、bash 起不来、清单读不出。调用方按「判不出来」处置（要跑），退出码 3。"""


class UsageError(Exception):
    """参数不对，退出码 2。"""


def sha256_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def sha256_bytes(content):
    return hashlib.sha256(content).hexdigest()


def snippet_of(text):
    return " ".join(str(text).split())[:SNIPPET_CHARACTERS]


class Refusal(collections.namedtuple("Refusal", "form file line snippet")):
    __slots__ = ()

    def text(self):
        return f"refused\t{self.form}\t{self.file}:{self.line}\t{self.snippet}"


UNIT_SLOTS = ("language", "kind", "name", "file", "first_line", "last_line", "fingerprint", "data",
              "short", "container", "conditions", "scope", "public", "first_token", "last_token",
              "plain", "methods", "qualified", "dyn_traits", "references", "data_references", "refusals",
              "self_types", "trait_name", "is_trait_impl", "blanket", "implicit_keys", "imports", "aliases", "import_glob", "defines",
              "declared", "node", "nodes", "facts", "body", "module", "cache", "full", "header", "module_head", "doc_module", "cfg_leaves",
              "open_keys", "ghost_only", "self_path_root", "listed_c_library_calls", "whole_body")

# 弄坏开关 rust-kind-names-hyphenated 下输出行退回的旧拼法（与 crates/singlefs-unit-graph 对齐之前 python 这一份的写法）
HYPHENATED_KIND_NAMES = {"macro_rules": "macro", "item_macro": "item-macro", "mod": "mod-decl", "extern_block": "extern-block",
                         "extern_crate": "extern-crate", "broadcast_group": "broadcast-group", "mod_head": "mod-head",
                         "wgsl_fn": "wgsl-fn", "wgsl_compute": "wgsl-entry", "wgsl_const": "wgsl-const", "wgsl_override": "wgsl-override",
                         "wgsl_struct": "wgsl-struct", "wgsl_alias": "wgsl-alias", "wgsl_var": "wgsl-var", "wgsl_directive": "wgsl-directive",
                         "wgsl_const_assert": "wgsl-const-assert"}


class Unit:
    __slots__ = UNIT_SLOTS

    def __init__(self, language, kind, name, file, first_line, last_line, fingerprint, data=False):
        for slot in UNIT_SLOTS:
            setattr(self, slot, None)
        self.language, self.kind, self.name, self.file = language, kind, name, file
        self.first_line, self.last_line, self.fingerprint, self.data = first_line, last_line, fingerprint, data
        self.short = name
        self.refusals = []

    def output_kind(self):
        """输出行里的种类名：Rust 与 WGSL 照 crates/singlefs-unit-graph 的拼法（unit.kind 本身就是那一种）；弄坏开关下退回旧拼法。"""
        if self.language in ("rust", "wgsl") and "rust-kind-names-hyphenated" in active_breaks():
            return HYPHENATED_KIND_NAMES.get(self.kind, self.kind)
        return self.kind

    def output_name(self):
        """输出行里的限定名：Rust 与 WGSL 带 crate 与模块路径前缀（full，照 crates/singlefs-unit-graph）；python、shell 与数据单元是 name。
        弄坏开关 rust-qualified-names-file-local 下 Rust 与 WGSL 退回文件里的嵌套路径（name）。"""
        if self.full is not None and "rust-qualified-names-file-local" not in active_breaks():
            return self.full
        return self.name

    def text(self):
        """进 UnitKey 组成清单与各判法按（种类, 限定名, 文件）认单元用的那一行：限定名是文件里的嵌套路径（name）。"""
        return f"{self.kind}\t{self.name}\t{self.file}:{self.first_line}-{self.last_line}\t{self.fingerprint}"

    def dump_text(self):
        """dump 与命令行 closure / key 打的那一行：Rust 与 WGSL 照 crates/singlefs-unit-graph 的写法（种类名、带前缀的限定名），别的语言同 text()。"""
        return f"{self.output_kind()}\t{self.output_name()}\t{self.file}:{self.first_line}-{self.last_line}\t{self.fingerprint}"

    def sort_key(self):
        return (self.file, self.first_line, self.kind, self.name)


def data_file_unit(root, path):
    with open(os.path.join(root, path), "rb") as handle:
        content = handle.read()
    rows = content.count(b"\n") + (0 if content.endswith(b"\n") or not content else 1)
    return Unit("data", "data-file", path, path, 1, max(rows, 1), sha256_bytes(content), data=True)


def directory_fingerprint(root, path):
    """数据目录的指纹：目录下（递归，什么目录都不跳过）全部文件按相对目录的路径排序，每个文件一行「<相对路径>\\t<字节 sha256>\\n」连起来的 sha256 → (指纹, 文件数)。
    指向目录的符号链接不进去，指向文件的读它指到的内容，断了的不算（crates/singlefs-unit-graph 的 directory_digest 同）。"""
    top = os.path.join(root, path)
    entries = []
    for directory, _names, files in os.walk(top):
        for name in files:
            absolute = os.path.join(directory, name)
            if os.path.islink(absolute) and not os.path.isfile(absolute):
                continue
            with open(absolute, "rb") as handle:
                entries.append((os.path.relpath(absolute, top).replace(os.sep, "/"), sha256_bytes(handle.read())))
    entries.sort()
    return sha256_text("".join(f"{relative}\t{digest}\n" for relative, digest in entries)), len(entries)


def data_directory_unit(root, path):
    """数据目录单元：种类 data-directory，限定名与文件都是目录相对根的路径，起止 1-<文件数>（至少 1），指纹 directory_fingerprint。"""
    fingerprint, count = directory_fingerprint(root, path)
    return Unit("data", "data-directory", path, path, 1, max(count, 1), fingerprint, data=True)


def files_under(root, suffixes, under=None):
    """根下（或 under 给的几处下）后缀对得上的文件，相对根、排好序；跳过 .git、target、__pycache__、node_modules。"""
    tops = [os.path.join(root, place) for place in under] if under else [root]
    found = set()
    for top in tops:
        if os.path.isfile(top):
            if top.endswith(suffixes):
                found.add(os.path.relpath(top, root).replace(os.sep, "/"))
            continue
        for directory, names, files in os.walk(top):
            names[:] = sorted(name for name in names if name not in SKIPPED_DIRECTORIES)
            for name in files:
                if name.endswith(suffixes):
                    found.add(os.path.relpath(os.path.join(directory, name), root).replace(os.sep, "/"))
    return sorted(found)


def inside_root(root, absolute):
    """绝对路径 → 相对根的路径（在根里面时），否则 None。"""
    real_root = os.path.normpath(os.path.abspath(root))
    candidate = os.path.normpath(absolute)
    if candidate == real_root or candidate.startswith(real_root + os.sep):
        return os.path.relpath(candidate, real_root).replace(os.sep, "/")
    return None


ASSIGNMENT_WORD = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)(?:\[(?:[^\[\]]|\[[^\]]*\])*\])?\+?=(.*)$", re.S)

# 起法里折不出的词从这里起（文件头「shell 起法带字面参数」）：(字面词…, SHELL_OPEN_TAIL)
SHELL_OPEN_TAIL = "\0open-tail"

# ── import 时执行的模块级语句与不走的子进程边（文件头「Python 闭包：import 时执行的模块级语句」「不走的子进程边」）的自证 ──

LIBRARY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
