"""Rust 与 WGSL 共用的词法切分 rust_lex 与它的磁盘缓存：缓存放哪个目录取「切词、读写缓存的代码」的现场指纹（live_closure_fingerprint），
跨轮缓存目录的算法（git_common_directory_of、cross_run_cache_directory）、弄坏开关变量与取法（BREAK_VARIABLE、active_breaks）也在这一份。
这一份的模块名字表就是词法缓存目录名的指纹范围，只依赖标准库。
这一份代码里读的弄坏开关（FUNCTION_UNITS_BREAK=<项>，只给自证证红用；全部开关与点名的项在 lib_function_units.py 文件头）：
  FUNCTION_UNITS_BREAK=cross-run-cache-under-tmpdir  cross_run_cache_directory 没设 GATE_CROSS_RUN_TMPDIR 时放这一趟的 TMPDIR 底下（缓存被一例自证的临时目录带走）
      「真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处」
  FUNCTION_UNITS_BREAK=cross-run-cache-git-directory-only  git_common_directory_of 只认 <仓>/.git 是目录（worktree 里 .git 是文件，本库两份退回这一趟的 TMPDIR、每趟重算）
      「真仓：跨轮缓存登记表（cross_run_cache_registry）里的每一份，在 TMPDIR 指到一个临时目录、GATE_CROSS_RUN_TMPDIR 没设、在 git 仓里（.git 是目录，或 git worktree 里 .git 是文件）时现取的位置都不落在 TMPDIR 之下；worktree 里落在 git 目录底下的几份与主仓共用 common-dir 那一处」
  FUNCTION_UNITS_BREAK=rust-lex-cache-unverified  Rust 词法单元的磁盘缓存文件名只取文本长度（缓存不核内容）
      「Rust 词法单元的磁盘缓存：同一份文本第二次取缓存、与现切的逐项相同；改一个字符（长度不变）不沿用、切出来的随文本变；缓存文件坏了照常现切」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-lexer-only  词法缓存的目录名只从 rust_lex 起收（读写缓存的两个函数与格式号不在里面）
      「Rust 词法单元缓存的目录名：改格式号 RUST_LEX_CACHE_FORM、改一个词种名常量（IDENT 的值）、改 load_rust_lex 的读法，目录名各换」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-whole-module  词法缓存的目录名把这份模块的每个函数都当起点收
      「Rust 词法单元缓存的目录名：改 rust_lex 里一行注释、这份库别处一行注释、一个不在 rust_lex_cached 闭包里的函数（match_brackets），目录名都不换」
  FUNCTION_UNITS_BREAK=rust-lex-cache-namespace-module-placement-read  词法缓存的目录名把 MODULE_PLACEMENT_NAMES（cross_run_cache_directory 读的 __file__）照取值
      「Rust 词法单元缓存的目录名：改 rust_lex 里一行注释、这份库别处一行注释、一个不在 rust_lex_cached 闭包里的函数（match_brackets），目录名都不换」
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import array
import base64
import bisect
import dis
import hashlib
import json
import os
import re
import shutil
import tempfile
import types
import zlib


BREAK_VARIABLE = "FUNCTION_UNITS_BREAK"


def active_breaks():
    return frozenset(name for name in os.environ.get(BREAK_VARIABLE, "").split(",") if name)


# ── Rust 与 WGSL 的词法 ─────────────────────────────────────────────────────

IDENT, LITERAL, LIFETIME, PUNCT = "ident", "literal", "lifetime", "punct"
OPENING_OF = {")": "(", "]": "[", "}": "{"}


def block_document_rows(body):
    return [re.sub(r"^\s*\*? ?", "", row, count=1) for row in body.split("\n")]


def is_identifier_start(character):
    """标识符起头：ASCII 字母、下划线，或任何非 ASCII 字符（Rust 那一份 lexer.rs 的 is_identifier_start：字节 >= 0x80 都算）。"""
    return (character.isascii() and (character.isalpha() or character == "_")) or ord(character) >= 0x80


def is_identifier_continue(character):
    """标识符续字符：ASCII 字母数字、下划线，或任何非 ASCII 字符（lexer.rs 的 is_identifier_continue）。"""
    return (character.isascii() and (character.isalnum() or character == "_")) or ord(character) >= 0x80


def rust_lex(text, spans=None, document_spans=None):
    """→ (kinds, texts, lines, documents)；documents 是 [(行, outer|inner, [各行正文], 下一个词法单元的下标, [各行原文])]：
    各行正文去掉了块注释每行开头的 * 号，各行原文是注释记号之后的原样（与 crates/singlefs-unit-graph 的 documentation_comment_of 同一份，文档测试取它）。
    spans / document_spans 给了列表的，逐个词法单元 / 文档注释往里追加它在 text 里的 (起, 止) 字符下标（剥只给 Verus 看的代码用：verus_ghost_code_blanked）。"""
    kinds, texts, lines, documents = [], [], [], []
    count = len(text)
    newlines = [match.start() for match in re.finditer("\n", text)]

    def line_at(position):
        return bisect.bisect_left(newlines, position) + 1

    def quoted_end(start, quote):
        j = start + 1
        while j < count:
            character = text[j]
            if character == "\\":
                j += 2
                continue
            if character == quote:
                return j + 1
            j += 1
        return count

    def raw_end(start):
        j, hashes = start, 0
        while j < count and text[j] == "#":
            hashes += 1
            j += 1
        if j >= count or text[j] != '"':
            return None
        closing = '"' + "#" * hashes
        found = text.find(closing, j + 1)
        return count if found < 0 else found + len(closing)

    def emit(kind, start, end):
        kinds.append(kind)
        texts.append(text[start:end])
        lines.append(line_at(start))
        if spans is not None:
            spans.append((start, end))

    def document(entry, start, end):
        documents.append(entry)
        if document_spans is not None:
            document_spans.append((start, end))

    i = 0
    while i < count:
        character = text[i]
        if character.isspace():
            i += 1
            continue
        following = text[i + 1] if i + 1 < count else ""
        if character == "/" and following == "/":
            end = text.find("\n", i)
            end = count if end < 0 else end
            body = text[i:end]
            if body.startswith("///") and not body.startswith("////"):
                document((line_at(i), "outer", [body[3:]], len(texts), [body[3:]]), i, end)
            elif body.startswith("//!"):
                document((line_at(i), "inner", [body[3:]], len(texts), [body[3:]]), i, end)
            i = end
            continue
        if character == "/" and following == "*":
            depth, j = 0, i
            while j < count:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                    if depth == 0:
                        break
                else:
                    j += 1
            body = text[i:j]
            style = None
            if body.startswith("/**") and not body.startswith("/***") and body != "/**/":
                style = "outer"
            elif body.startswith("/*!"):
                style = "inner"
            if style is not None:
                if style == "outer":
                    raw = body[3:-2] if body.endswith("*/") and len(body) >= 5 else ""
                else:
                    raw = body[3:-2] if body.endswith("*/") and len(body) >= 5 else body[3:]
                document((line_at(i), style, block_document_rows(body[3:-2]), len(texts), raw.split("\n")), i, j)
            i = j
            continue
        start = i
        if character in "bc" and following == "r":
            end = raw_end(i + 2)
            if end is not None:
                emit(LITERAL, start, end)
                i = end
                continue
        if character == "r" and following in ('#', '"'):
            if following == "#" and i + 2 < count and is_identifier_start(text[i + 2]):
                j = i + 2
                while j < count and is_identifier_continue(text[j]):
                    j += 1
                # 原始标识符的词按源码原样取、带着 r#（与 Rust 那一份 lexer.rs 同一条：词是源码里的那一段）
                emit(IDENT, i, j)
                i = j
                continue
            end = raw_end(i + 1)
            if end is not None:
                emit(LITERAL, start, end)
                i = end
                continue
        if character in "bc" and following in ('"', "'"):
            end = quoted_end(i + 1, following)
            emit(LITERAL, start, end)
            i = end
            continue
        if character == '"':
            end = quoted_end(i, '"')
            emit(LITERAL, start, end)
            i = end
            continue
        if character == "'":
            if following == "\\":
                end = quoted_end(i, "'")
                emit(LITERAL, start, end)
                i = end
                continue
            if i + 2 < count and text[i + 2] == "'":
                emit(LITERAL, start, i + 3)
                i += 3
                continue
            j = i + 1
            while j < count and (text[j].isalnum() or text[j] == "_"):
                j += 1
            emit(LIFETIME, start, j)
            i = j
            continue
        if is_identifier_start(character):
            j = i
            while j < count and is_identifier_continue(text[j]):
                j += 1
            emit(IDENT, start, j)
            i = j
            continue
        if character.isdigit():
            j = i
            hexadecimal = text.startswith(("0x", "0X"), i)
            while j < count and (text[j].isalnum() or text[j] == "_"
                                 or (text[j] == "." and j + 1 < count and text[j + 1].isdigit() and not hexadecimal)
                                 or (text[j] in "+-" and text[j - 1] in "eE" and not hexadecimal)):
                j += 1
            emit(LITERAL, start, j)
            i = j
            continue
        emit(PUNCT, start, i + 1)
        i += 1
    return kinds, texts, lines, documents


# Rust 词法单元的磁盘缓存（文件头「Rust 词法单元的缓存」）
RUST_LEX_CACHE_FORM = "singlefs-rust-lex-cache-v1"
RUST_LEX_CACHE_DIRECTORY_NAME = "singlefs-rust-lex-cache"
RUST_LEX_CACHE_DIRECTORY_VARIABLE = "FUNCTION_UNITS_LEX_CACHE_DIRECTORY"   # 设了放这里；写 off 不用缓存（自证里除缓存那一格外都设它）
RUST_LEX_CACHE_KEPT_NAMESPACES = 3
RUST_LEX_CACHE_STATS = {"lexed": 0, "reused": 0}   # 本进程里 rust_lex_cached 现切了几份、取了几份缓存（自证数它）
_RUST_LEX_CACHE_NAMESPACES = {}
# 词法缓存的目录名不取值的两样（rust_lex_cache_namespace）：本进程的计数与已算好的目录名，跑着会变，不定缓存里存的是什么
RUST_LEX_CACHE_PROCESS_STATE = ("RUST_LEX_CACHE_STATS", "_RUST_LEX_CACHE_NAMESPACES")
# 字节码里按全局名取值、赋值的几种指令（live_closure_fingerprint）
GLOBAL_NAME_OPERATIONS = frozenset(("LOAD_GLOBAL", "LOAD_NAME", "STORE_GLOBAL", "DELETE_GLOBAL", "LOAD_FROM_DICT_OR_GLOBALS"))
# 类的属性表里随载入的方式、所在的行变的几项（live_closure_fingerprint 不取）
CLASS_PLACEMENT_ATTRIBUTES = frozenset(("__module__", "__firstlineno__", "__dict__", "__weakref__"))
# 模块名字表里记这份模块放在哪、怎么载入的几项（live_closure_fingerprint 只记名字、不取值）
MODULE_PLACEMENT_NAMES = frozenset(("__file__", "__cached__", "__name__", "__spec__", "__loader__", "__package__"))


def git_common_directory_of(root):
    """root 这棵工作树的 git common-dir，照 git 自己在盘上的约定读、不起 git：root/.git 是目录就是它；是文件（git worktree、子模块）读它的
    「gitdir: <路径>」一行（相对 root），那个目录里有 commondir 文件就照它的内容（相对那个目录）找到主仓的 git 目录，没有就是那个目录自己。
    不是 git 工作树、读不出、指到的目录不在的交 None。
    弄坏开关 cross-run-cache-git-directory-only：只认 root/.git 是目录（worktree 里 .git 是文件，判成不在 git 仓里）。"""
    dot_git = os.path.join(root, ".git")
    if os.path.isdir(dot_git):
        return dot_git
    if "cross-run-cache-git-directory-only" in active_breaks() or not os.path.isfile(dot_git):
        return None
    try:
        with open(dot_git, encoding="utf-8") as handle:
            pointer = handle.readline().strip()
        if not pointer.startswith("gitdir:"):
            return None
        git_directory = os.path.normpath(os.path.join(root, pointer[len("gitdir:"):].strip()))
        common_file = os.path.join(git_directory, "commondir")
        common = git_directory
        if os.path.isfile(common_file):
            with open(common_file, encoding="utf-8") as handle:
                common = os.path.normpath(os.path.join(git_directory, handle.readline().strip()))
    except (OSError, UnicodeDecodeError):
        return None
    return common if os.path.isdir(common) else None


def cross_run_cache_directory(name):
    """跨轮缓存 name 放哪：设了 GATE_CROSS_RUN_TMPDIR 放它底下；没设放这份库所在仓的 git common-dir 底下（git_common_directory_of；与各道判定库一处，
    git worktree 里是主仓那一份、与主仓共用；不随这一趟的 TMPDIR 走，跑完不留在那一趟的临时目录里）；这份库不在 git 仓里才放 TMPDIR 底下。"""
    configured = os.environ.get("GATE_CROSS_RUN_TMPDIR", "").strip()
    if configured:
        return os.path.join(configured, name)
    if "cross-run-cache-under-tmpdir" in active_breaks():
        return os.path.join(tempfile.gettempdir(), name)   # 弄坏开关：没设 GATE_CROSS_RUN_TMPDIR 就落在这一趟的 TMPDIR 里
    common = git_common_directory_of(os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))))
    return os.path.join(common if common is not None else tempfile.gettempdir(), name)


def rust_lex_cache_directory():
    """缓存放哪个目录；交 None 是不用缓存。设了 FUNCTION_UNITS_LEX_CACHE_DIRECTORY 取它（off 是不用），没设照 cross_run_cache_directory 放 singlefs-rust-lex-cache。"""
    configured = os.environ.get(RUST_LEX_CACHE_DIRECTORY_VARIABLE, "").strip()
    if configured == "off":
        return None
    return configured or cross_run_cache_directory(RUST_LEX_CACHE_DIRECTORY_NAME)


def live_closure_fingerprint(namespace, entries, skipped=()):
    """namespace（一份模块的名字表）里 entries 那几个名字照字节码往下收、交一把 sha256（文件头「Rust 词法单元的缓存」）。
    函数取它的代码（字节码、常量、名字表与局部量表，套在里面的代码一样；不取文件名与行号表：只改注释、上下挪了行不变）、缺省值与闭包变量，
    字节码里按全局名读到的、namespace 里有的名字逐个接着收；namespace 里定义的类取它的属性表（CLASS_PLACEMENT_ATTRIBUTES 那几项不取）；
    串、数、元组、表、集合取内容；模块与别处定义的函数、类只取名字；skipped 与 MODULE_PLACEMENT_NAMES 里的名字只记名字、不取值。不读源码文件。
    弄坏开关 rust-lex-cache-namespace-module-placement-read：MODULE_PLACEMENT_NAMES 照取值（__file__ 的路径进指纹，换个路径执行同一份代码目录名就换）。"""
    texts, pending, entered = {}, list(entries), set()
    unvalued = set(skipped) if "rust-lex-cache-namespace-module-placement-read" in active_breaks() else set(skipped) | MODULE_PLACEMENT_NAMES

    def reach(code):
        for instruction in dis.get_instructions(code):
            name = instruction.argval
            if instruction.opname in GLOBAL_NAME_OPERATIONS and name in namespace and name not in texts and name not in pending:
                pending.append(name)
        for constant in code.co_consts:
            if isinstance(constant, types.CodeType):
                reach(constant)

    def code_text(code):
        constants = [code_text(constant) if isinstance(constant, types.CodeType) else value_text(constant) for constant in code.co_consts]
        return repr((code.co_qualname, code.co_argcount, code.co_posonlyargcount, code.co_kwonlyargcount, code.co_flags, code.co_code.hex(),
                     code.co_exceptiontable.hex(), code.co_names, code.co_varnames, code.co_freevars, code.co_cellvars, constants))

    def cell_text(cell):
        try:
            return value_text(cell.cell_contents)
        except ValueError:   # 还没赋值的闭包变量
            return "empty-cell"

    def value_text(value):
        if isinstance(value, types.FunctionType):
            if value.__globals__ is not namespace or id(value) in entered:
                return f"function {value.__qualname__}"
            entered.add(id(value))
            reach(value.__code__)
            return repr(("function", code_text(value.__code__), value_text(value.__defaults__), value_text(value.__kwdefaults__),
                         [cell_text(cell) for cell in value.__closure__ or ()]))
        if isinstance(value, type):
            if value.__module__ != namespace.get("__name__") or id(value) in entered:
                return f"class {value.__qualname__}"
            entered.add(id(value))
            members = sorted((name, value_text(member)) for name, member in vars(value).items() if name not in CLASS_PLACEMENT_ATTRIBUTES)
            return repr(("class", value.__qualname__, [base.__qualname__ for base in value.__bases__], members))
        if isinstance(value, types.ModuleType):
            return f"module {value.__name__}"
        if isinstance(value, (staticmethod, classmethod)):
            return repr((type(value).__name__, value_text(value.__func__)))
        if isinstance(value, property):
            return repr(("property", value_text(value.fget), value_text(value.fset), value_text(value.fdel)))
        if isinstance(value, (tuple, list)):
            return repr((type(value).__name__, [value_text(item) for item in value]))
        if isinstance(value, (set, frozenset)):
            return repr((type(value).__name__, sorted(value_text(item) for item in value)))
        if isinstance(value, dict):
            return repr(("dict", sorted((value_text(key), value_text(item)) for key, item in value.items())))
        if isinstance(value, re.Pattern):
            return repr(("pattern", value.pattern, value.flags))
        if value is None or isinstance(value, (str, bytes, int, float, complex, bool)):
            return repr(value)
        return f"value {type(value).__qualname__}"

    while pending:
        name = pending.pop()
        texts[name] = None
        texts[name] = "skipped" if name in unvalued else value_text(namespace[name])
    return hashlib.sha256("\n".join(f"{name}\0{texts[name]}" for name in sorted(texts)).encode("utf-8", "surrogateescape")).hexdigest()


def rust_lex_cache_namespace():
    """缓存按「切词法单元、读写缓存的代码」分目录（文件头「Rust 词法单元的缓存」）：rust_lex_cached 经全局名够得到的函数与常量的现场指纹
    （live_closure_fingerprint：rust_lex 与它的帮手、rust_lex_cache_path、load_rust_lex、store_rust_lex、RUST_LEX_CACHE_FORM、词种名常量都在里面）加此刻的弄坏开关。
    不读这份库的原文：只改这份库别处的注释与函数，目录不换。弄坏开关 rust-lex-cache-namespace-lexer-only：只从 rust_lex 起收（读写缓存的两个函数与格式号不在里面）；
    rust-lex-cache-namespace-whole-module：这份模块里的每个函数都当起点（改不在闭包里的函数目录名也换）。"""
    breaks = active_breaks()
    label = ",".join(sorted(breaks))
    if label not in _RUST_LEX_CACHE_NAMESPACES:
        namespace = rust_lex_cached.__globals__
        if "rust-lex-cache-namespace-whole-module" in breaks:
            entries = sorted(name for name, value in namespace.items() if isinstance(value, types.FunctionType) and value.__globals__ is namespace)
        else:
            entries = [rust_lex.__name__ if "rust-lex-cache-namespace-lexer-only" in breaks else rust_lex_cached.__name__]
        fingerprint = live_closure_fingerprint(namespace, entries, RUST_LEX_CACHE_PROCESS_STATE)
        _RUST_LEX_CACHE_NAMESPACES[label] = hashlib.sha256(f"{fingerprint}\0{label}".encode("utf-8")).hexdigest()[:24]
    return _RUST_LEX_CACHE_NAMESPACES[label]


def rust_lex_cache_path(text):
    """这份文本的缓存文件；不用缓存交 None。文件名是文本的 sha256（只看内容，不看路径与修改时刻）；
    弄坏开关 rust-lex-cache-unverified：文件名只取文本长度（缓存不核内容）。"""
    root = rust_lex_cache_directory()
    if root is None:
        return None
    name = f"length-{len(text)}" if "rust-lex-cache-unverified" in active_breaks() else hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()
    return os.path.join(root, rust_lex_cache_namespace(), name + ".json.z")


def load_rust_lex(path):
    """缓存文件读成 (kinds, texts, lines, documents)；读不了、不是这一版的写法、各表对不上都交 None（照常现切）。"""
    try:
        with open(path, "rb") as handle:
            stored = json.loads(zlib.decompress(handle.read()))
        if stored["form"] != RUST_LEX_CACHE_FORM:
            return None
        kind_names, table = stored["kind_names"], stored["table"]
        kinds = [kind_names[code] for code in base64.b64decode(stored["kinds"])]
        text_codes, line_numbers = array.array("I"), array.array("I")
        text_codes.frombytes(base64.b64decode(stored["texts"]))
        line_numbers.frombytes(base64.b64decode(stored["lines"]))
        texts = [table[code] for code in text_codes]
        documents = [tuple(entry) for entry in stored["documents"]]
        if not (len(kinds) == len(texts) == len(line_numbers)) or any(len(entry) != 5 for entry in documents):
            return None
        return kinds, texts, list(line_numbers), documents
    except (OSError, ValueError, KeyError, IndexError, TypeError, EOFError, zlib.error):
        return None


def store_rust_lex(path, result):
    """rust_lex 的结果原子写成缓存文件（同目录排他建临时文件再改名）；新开一个目录时只留最近 RUST_LEX_CACHE_KEPT_NAMESPACES 个；写不成不影响这一趟。"""
    kinds, texts, lines, documents = result
    kind_names = sorted(set(kinds))
    kind_codes = {name: code for code, name in enumerate(kind_names)}
    table = list(dict.fromkeys(texts))
    text_codes = {text: code for code, text in enumerate(table)}
    payload = json.dumps(dict(form=RUST_LEX_CACHE_FORM, kind_names=kind_names, kinds=base64.b64encode(bytes(kind_codes[kind] for kind in kinds)).decode("ascii"),
                              table=table, texts=base64.b64encode(array.array("I", [text_codes[text] for text in texts]).tobytes()).decode("ascii"),
                              lines=base64.b64encode(array.array("I", lines).tobytes()).decode("ascii"), documents=documents))
    directory = os.path.dirname(path)
    try:
        fresh = not os.path.isdir(directory)
        os.makedirs(directory, exist_ok=True)
        handle, temporary = tempfile.mkstemp(prefix=".partial-", dir=directory)
        try:
            with os.fdopen(handle, "wb") as stream:
                stream.write(zlib.compress(payload.encode("ascii"), 1))
            os.replace(temporary, path)
        except BaseException:
            if os.path.exists(temporary):
                os.unlink(temporary)
            raise
        if fresh:
            root = os.path.dirname(directory)
            kept = sorted((name for name in os.listdir(root) if os.path.isdir(os.path.join(root, name))),
                          key=lambda name: os.path.getmtime(os.path.join(root, name)), reverse=True)
            for name in kept[RUST_LEX_CACHE_KEPT_NAMESPACES:]:
                shutil.rmtree(os.path.join(root, name), ignore_errors=True)
    except OSError:
        return


def rust_lex_cached(text):
    """rust_lex(text) 的磁盘缓存版：→ 与 rust_lex(text) 逐项相同的 (kinds, texts, lines, documents)，每次是新的列表。有缓存取缓存，没有现切再存。"""
    path = rust_lex_cache_path(text)
    loaded = load_rust_lex(path) if path is not None else None
    if loaded is not None:
        RUST_LEX_CACHE_STATS["reused"] += 1
        return loaded
    result = rust_lex(text)
    RUST_LEX_CACHE_STATS["lexed"] += 1
    if path is not None:
        store_rust_lex(path, result)
    return result


def match_brackets(kinds, texts):
    """→ (左括号下标 → 右括号下标, None)，配不上时 → (None, 出错的下标)。"""
    pairs, stack = {}, []
    for index, (kind, text) in enumerate(zip(kinds, texts)):
        if kind != PUNCT:
            continue
        if text in ("(", "[", "{"):
            stack.append(index)
        elif text in (")", "]", "}"):
            if not stack or texts[stack[-1]] != OPENING_OF[text]:
                return None, index
            pairs[stack.pop()] = index
    if stack:
        return None, stack[-1]
    return pairs, None
