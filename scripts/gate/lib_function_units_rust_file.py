"""Rust 源文件的切单元：词法单元上的小解析（签名、cfg 谓词、裸名与 use 路径）、RustFile（切条目、模块、impl、宏、文档测试）与文档测试代码块。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import os
import posixpath
import re
from lib_function_units_lexer import IDENT, LIFETIME, LITERAL, active_breaks, match_brackets, rust_lex, rust_lex_cached
from lib_function_units_base import (
    DEEPEST_MANIFEST_BINDING_CHAIN, MANIFEST_FOLD_REFUSED, Refusal, SPAWNING_WORDS, Unit, c_library_calls_allowed_in_extern_blocks, sha256_text,
    snippet_of,
)


def rust_plain_string(token):
    """普通串字面量去掉两头引号之后的原文（不解转义）；原始串、字节串、别的词 → None。"""
    return token[1:-1] if len(token) >= 2 and token[0] == token[-1] == '"' else None


def rust_string_value(token):
    """串字面量的值（普通串、原始串、字节串、C 串）；不是串回 None。"""
    match = re.match(r'^[bc]?r(#*)"(.*)"\1$', token, re.S)
    if match:
        return match.group(2)
    if token[:2] in ('b"', 'c"'):
        token = token[1:]
    if len(token) >= 2 and token[0] == '"' and token[-1] == '"':
        escapes = {"n": "\n", "t": "\t", "\\": "\\", '"': '"', "'": "'", "0": "\0", "r": "\r", "\n": ""}
        return re.sub(r"\\(.)", lambda found: escapes.get(found.group(1), found.group(1)), token[1:-1], flags=re.S)
    return None


FORMAT_CAPTURE = re.compile(r"(?<!\{)\{([A-Za-z_][A-Za-z0-9_]*)(?=[:}])")


def format_captures(literal):
    """串字面量里 {名} / {名:…} 形式的格式捕获（照 crates/singlefs-unit-graph 的 add_format_captures 逐字节扫）：{{ 是转义、跳过两个字节，
    转义之后紧跟的 { 照样开一个捕获（{{{名}}} 里的名算）。弄坏开关 rust-escaped-brace-captures-skipped 下退回 (?<!\\{)\\{ 的口径（{{{名}}} 不算）。"""
    if "rust-escaped-brace-captures-skipped" in active_breaks():
        return set(FORMAT_CAPTURE.findall(literal))
    names, position, count = set(), 0, len(literal)
    while position < count:
        if literal[position] != "{":
            position += 1
            continue
        if position + 1 < count and literal[position + 1] == "{":
            position += 2
            continue
        start = end = position + 1
        while end < count and literal[end].isascii() and (literal[end].isalnum() or literal[end] == "_"):
            end += 1
        if end > start and not literal[start].isdigit() and end < count and literal[end] in "}:":
            names.add(literal[start:end])
        position = max(end, position + 1)
    return names


# 函数体里有这几个词的（可能嵌套条目，里面同名的不在绑定的作用域里）不按局部绑定遮（bare_name_only_a_local_binding；crates/singlefs-unit-graph 的 WORDS_STARTING_A_NESTED_ITEM）
WORDS_STARTING_A_NESTED_ITEM = frozenset(("fn", "impl", "trait", "mod", "const", "static", "struct", "enum", "union", "type", "use", "macro_rules", "extern"))
# 名字落在这几个标准库宏的调用括号里照算局部（bare_name_only_a_local_binding）：它们把实参当表达式就地求值，不把实参接在路径后面、不放进嵌套条目里；
# 别的宏把实参展开到哪看不出，落在它们括号里的名字不算遮住。逐项与 crates/singlefs-unit-graph/src/closure.rs 的 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 相同（自证读那份源码逐项比）
STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE = (
    "format", "format_args", "print", "println", "eprint", "eprintln", "write", "writeln", "panic",
    "assert", "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne", "vec",
)


def standard_macros_evaluating_their_arguments_in_place():
    """上面那张名单；弄坏开关 rust-macro-list-short 下少最后一项（vec）。"""
    if "rust-macro-list-short" in active_breaks():
        return STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE[:-1]
    return STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE


def function_signature_positions(texts, first, last):
    """函数单元（词 first..last）的签名：深度 0 上头一个 fn、后面的名字、跳过尖括号里的泛型参数之后的 ( 与配对的 )、再往后深度 0 上头一个 {
    → (名字, 参数表的开括号, 参数表的闭括号, 函数体的开括号) 的下标；哪一步对不上（没有函数体的声明也算）→ None。"""
    def t(position):
        return texts[position] if first <= position < last else ""
    depth, keyword = 0, None
    for position in range(first, last):
        text = t(position)
        if text in ("(", "[", "{"):
            depth += 1
        elif text in (")", "]", "}"):
            depth = max(depth - 1, 0)
        elif text == "fn" and depth == 0:
            keyword = position
            break
    if keyword is None:
        return None
    own_name = keyword + 1
    parameters_open = own_name + 1
    if t(parameters_open) == "<":
        angle = 0
        while True:
            text = t(parameters_open)
            if text == "":
                return None
            if text == "<":
                angle += 1
            elif text == ">" and t(parameters_open - 1) != "-":
                angle = max(angle - 1, 0)
            parameters_open += 1
            if angle == 0:
                break
    if t(parameters_open) != "(":
        return None
    depth, parameters_close = 0, None
    for position in range(parameters_open, last):
        text = t(position)
        if text == "{" and parameters_close is not None and depth == 0:
            return own_name, parameters_open, parameters_close, position
        if text in ("(", "[", "{"):
            depth += 1
        elif text in (")", "]", "}"):
            depth = max(depth - 1, 0)
            if depth == 0 and parameters_close is None:
                parameters_close = position
    return None


def parameter_binding_position(kinds, texts, parameters_open, parameters_close, name, last):
    """参数表里绑定 name 的那一处：括号深度 0（相对参数表）上表头或 , 之后的头一个词（跳过 mut）是它、后面接单个 :。解构、&a: … 不认。没有 → None。"""
    def t(position):
        return texts[position] if position < last else ""
    depth, at_start = 0, True
    for position in range(parameters_open + 1, parameters_close):
        word = t(position)
        if word in ("(", "[", "{"):
            depth += 1
        elif word in (")", "]", "}"):
            depth = max(depth - 1, 0)
        elif word == "," and depth == 0:
            at_start = True
            continue
        if not at_start or word == "mut":
            continue
        at_start = False
        if kinds[position] == IDENT and identifier_name(word) == name and t(position + 1) == ":" and t(position + 2) != ":":
            return position
    return None


def identifier_name(text):
    return text[2:] if text.startswith("r#") else text


def extern_block_attributes_name_link(kinds, texts, first, last):
    """extern 块的词（first 起、到 last 之前；从它的外属性起，extern 关键字之前）里有没有标识符 link：#[link(…)]、#[cfg_attr(条件, link(…))] 里的都算
    （link_name 是别的词，不算）；带它的块改链接的库（crates/singlefs-unit-graph 的 extern_block_attributes_name_link 同一个判法）。
    弄坏开关 rust-extern-link-first-attribute-only 下只认紧挨 #[ 的 link。"""
    first_attribute_only = "rust-extern-link-first-attribute-only" in active_breaks()
    for position in range(first, last):
        if kinds[position] != IDENT:
            continue
        if texts[position] == "extern":
            return False
        if texts[position] == "link" and (not first_attribute_only or (position >= 2 and texts[position - 1] == "[" and texts[position - 2] == "#")):
            return True
    return False


def bare_name_only_a_local_binding(kinds, texts, first, last, name):
    """函数单元（词 first..last）里 name 只是局部绑定（文件头 Rust 闭包「局部 let 与参数绑定遮住的名字」；crates/singlefs-unit-graph 的
    is_bare_name_only_a_local_binding_of_the_function 同一个判法）：至少有一处绑定（语句开头的 let [mut] 名，或参数），别的每一处都落在绑定的作用域里。
    let 从它的 ; 起到外层花括号收尾，参数从参数表的 ) 起到单元收尾。不算遮住的（宁可多连）：绑定之前或作用域之外的、带路径的（前后挨 ::）、
    宏名（后接 ! 与开括号）、格式串捕获（{名}）在作用域外的、宏调用括号里的 let、落在宏调用括号里的名字（宏可能把它接在路径后面或放进嵌套条目；
    只有 STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE 那几个宏的括号里、在作用域里的照算局部）、函数体里有嵌套条目（WORDS_STARTING_A_NESTED_ITEM）的。
    单个 . 后面的是字段或方法，不看；函数自己的名字不看。"""
    signature = function_signature_positions(texts, first, last)
    if signature is None:
        return False
    own_name, parameters_open, parameters_close, body_open = signature

    def t(position):
        return texts[position] if first <= position < last else ""
    parameter = parameter_binding_position(kinds, texts, parameters_open, parameters_close, name, last)
    has_binding = parameter is not None
    movable_macros_widen = "rust-macro-arguments-all-local" not in active_breaks()
    listed_macros = standard_macros_evaluating_their_arguments_in_place()
    # 还开着的每一层括号：[是不是花括号, 在不在宏调用括号里, 此刻名字绑定了没有, 有没有一条 let 在等分号,
    #                     在不在名单（STANDARD_MACROS_EVALUATING_THEIR_ARGUMENTS_IN_PLACE）之外的宏调用括号里（外层在里面的算）]
    stack = [[False, False, False, False, False]]
    for position in range(first, last):
        text, kind = texts[position], kinds[position]
        innermost = stack[-1]
        if kind == LITERAL:
            if not innermost[2] and name in text and "{" in text and text[:1] in ('"', "r") and name in format_captures(text):
                return False
        elif kind == IDENT:
            if identifier_name(text) != name or position in (own_name, parameter):
                continue
            let_distance = 2 if t(position - 1) == "mut" else 1
            if innermost[0] and not innermost[1] and t(position - let_distance) == "let" and t(position - let_distance - 1) in ("{", "}", ";") and (
                    t(position + 1) in ("=", ";") or (t(position + 1) == ":" and t(position + 2) != ":")):
                innermost[3] = True
                has_binding = True
                continue
            if t(position - 1) == "." and t(position - 2) != ".":
                continue
            if (t(position - 1) == ":" and t(position - 2) == ":") or (t(position + 1) == ":" and t(position + 2) == ":") or (
                    t(position + 1) == "!" and t(position + 2) in ("(", "[", "{")):
                return False
            if innermost[4] and movable_macros_widen:
                return False
            if not innermost[2]:
                return False
        elif kind != LIFETIME:
            if text in ("(", "[", "{"):
                follows_a_macro_name = position >= first + 2 and t(position - 1) == "!" and kinds[position - 2] == IDENT
                may_move_its_arguments = follows_a_macro_name and identifier_name(t(position - 2)) not in listed_macros
                stack.append([text == "{", innermost[1] or follows_a_macro_name, innermost[2], False, innermost[4] or may_move_its_arguments])
            elif text in (")", "]", "}"):
                if len(stack) > 1:
                    stack.pop()
                if position == parameters_close and parameter is not None:
                    stack[-1][2] = True
            elif text == ";" and innermost[3]:
                innermost[3] = False
                innermost[2] = True
    return has_binding and not any(kinds[position] == IDENT and texts[position] in WORDS_STARTING_A_NESTED_ITEM for position in range(body_open, last))


def use_path_leading_to(rust_file, use_keyword, path_start):
    """use 语句里 path_start 那一段前面的路径，花括号展开：use a::{b::{c, d}, e} 里 d 前面是 a、b。顺着「段 ::」收到 use 为止；use_keyword 是 use 的下标。"""
    path, position = [], path_start
    while True:
        if position >= use_keyword + 4 and rust_file.t(position - 1) == ":" and rust_file.t(position - 2) == ":" and rust_file.kind(position - 3) == IDENT:
            path.append(rust_file.t(position - 3))
            position -= 3
            continue
        nested, brace = 0, None
        for cursor in range(position - 1, use_keyword, -1):
            text = rust_file.t(cursor)
            if text == "}":
                nested += 1
            elif text == "{":
                if nested == 0:
                    brace = cursor
                    break
                nested -= 1
        if brace is None:
            path.reverse()
            return path
        position = brace


def token_references(kinds, texts, low, high):
    """→ (裸名, 方法名, (限定, 名), dyn trait)：R-裸名、R-方法、R-路径读前三样，dyn trait 只在弄坏开关 rust-dyn-traits-open-every-implementation 下读；
    字段访问 .f（后面不接 ( 或 ::）不收。
    串字面量里的 {名} / {名:…}（format! 一族的内联捕获）也算裸名：不认它，format!("{CONST}") 读的常量就漏了。"""
    plain, methods, qualified, dyn_traits = set(), set(), set(), set()
    count = len(texts)
    captures = "rust-format-captures-ignored" not in active_breaks()

    def text_at(index):
        return texts[index] if 0 <= index < count else ""

    for i in range(low, high):
        if kinds[i] == LITERAL and captures and "{" in texts[i] and texts[i][:1] in ('"', "r"):
            plain.update(format_captures(texts[i]))
            continue
        if kinds[i] != IDENT:
            continue
        name = texts[i]
        previous = text_at(i - 1)
        if previous == "." and text_at(i - 2) != ".":
            following = text_at(i + 1)
            if following == "(" or (following == ":" and text_at(i + 2) == ":"):
                methods.add(name)
            continue
        if previous == ":" and text_at(i - 2) == ":":
            if i >= 3 and kinds[i - 3] == IDENT:
                qualified.add((texts[i - 3], name))
            elif text_at(i - 3) == ">" and text_at(i - 5) == "as" and i >= 4 and kinds[i - 4] == IDENT:
                qualified.add((texts[i - 4], name))
        if name == "dyn":
            j = i + 1
            while j + 2 < count and kinds[j] == IDENT and texts[j + 1] == ":" and texts[j + 2] == ":":
                j += 3
            if j < count and kinds[j] == IDENT:
                dyn_traits.add(texts[j])
        plain.add(name)
    return plain, methods, qualified, dyn_traits


# ── Rust：一份文件切成条目单元 ───────────────────────────────────────────────

RUST_QUALIFIERS = frozenset(("async", "unsafe", "default", "safe", "open", "closed", "spec", "proof", "exec", "broadcast",
                             "tracked", "ghost", "uninterp"))
RUST_ITEM_STARTS = frozenset(("pub", "fn", "struct", "enum", "union", "impl", "trait", "mod", "use", "const", "static", "type",
                              "macro_rules", "extern", "auto")) | RUST_QUALIFIERS
RUST_NAMED_KINDS = frozenset(("fn", "const", "static", "struct", "enum", "union", "type", "trait", "macro_rules", "broadcast_group"))
# 按名字找得到时用的是 defines 里那几个名字、不是自己名字的种类（crates/singlefs-unit-graph 的 declared_names）：条目级宏调用与 extern 块
# 是它里面定义的名字，assume_specification 是方括号里点名路径末段的名字
RUST_KINDS_DEFINING_NAMES = frozenset(("item_macro", "extern_block", "assume_specification"))
RUST_TYPE_KINDS = frozenset(("struct", "enum", "union", "type"))
def use_path_root(rust_file, first, last):
    """use 语句（词 first..last）的路径根名：use 之后头一个不是 : 的词，是标识符才算（use {…} 没有根名）→ 文本或 None。"""
    position = first
    while position < last and not (rust_file.kinds[position] == IDENT and rust_file.texts[position] == "use"):
        position += 1
    position += 1
    while position < last and rust_file.texts[position] == ":":
        position += 1
    return rust_file.texts[position] if position < last and rust_file.kinds[position] == IDENT else None


# R-裸名按模块筛候选时，路径根名是这几个的指标准库，不点本目标里的哪一份（另几个是看得见的别的 crate 的库名；RustWorkspace.candidates_seen_from_module）
STANDARD_CRATE_PATH_ROOTS = ("std", "core", "alloc")
RUST_TYPE_OR_TRAIT_KINDS = RUST_TYPE_KINDS | {"trait"}   # 前缀::名 的前缀是这几种之一的，名是关联项（RustWorkspace.candidates_seen_from_module）
# 只是结构、里面的名字不往下解的种类（R-结构；与 crates/singlefs-unit-graph 的 UnitKind::resolves_its_own_names 为假的那几种同一份）
RUST_STRUCTURE_ONLY_KINDS = frozenset(("use", "mod", "mod_head", "extern_crate"))
RUST_ITEM_MACROS_HOLDING_ITEMS = frozenset(("thread_local",))
PRIMITIVE_TYPES = frozenset(("u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
                             "bool", "char", "str", "String", "Vec", "Option", "Result", "Box", "Self"))
SELF_TYPE_SKIPPED = frozenset(("dyn", "mut", "const", "impl", "for", "where", "as", "unsafe"))
RUST_KEYWORDS = frozenset(("as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for",
                           "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
                           "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "macro_rules"))
# union 是上下文关键字（fn union() 合法），不在 RUST_KEYWORDS 里（与 crates/singlefs-unit-graph 的 RUST_KEYWORDS 同一份）；弄坏开关 rust-union-as-keyword 下另去掉它
UNION_KEYWORD = frozenset(("union",))


# #[test] 带来的条件（只在 cfg(test) 下编）的原文标记：键里的 cfg 原子只算 #[cfg] / #[cfg_attr] / cfg! 真引到的，它不算
TEST_ATTRIBUTE_RAW = "#[test]"


def cfg_tree_leaves(tree):
    """cfg 谓词树 → 叶子（("name", 名) 与 ("pair", 键, 值)），由左到右。"""
    if tree[0] in ("all", "any", "not"):
        return [leaf for item in tree[1] for leaf in cfg_tree_leaves(item)]
    return [tree]


def cfg_atom_text(leaf):
    """叶子写成原子：无值名字写名字，键值写 键="值"（与 crates/singlefs-unit-graph 的 atoms 同一种写法）。"""
    return leaf[1] if leaf[0] == "name" else f'{leaf[1]}="{leaf[2]}"'


def cfg_predicates_in_tokens(kinds, texts, lines, pairs, low, high):
    """一段词法单元里写出来的 cfg 谓词 → [(树或 None, 原文, 行)]：每个属性 #[…] / #![…] 里的 cfg(谓词)、cfg_attr(条件, 属性…) 的条件与
    它属性里套的 cfg(…) / cfg_attr(…)，以及 cfg!(谓词)（括号写成 ( [ { 都认）。树是 None 的是写不成谓词的原文。"""
    out = []

    def text(index):
        return texts[index] if 0 <= index < len(texts) else ""

    def predicate(start, end):
        raw = " ".join(texts[start:end])
        try:
            tree = parse_cfg_tokens(kinds[start:end], texts[start:end])
        except ValueError:
            tree = None
        out.append((tree, raw, lines[start] if start < len(lines) else (lines[-1] if lines else 1)))

    def parts(start, end):
        found, begin, index = [], start, start
        while index < end:
            if text(index) in ("(", "[", "{") and index in pairs:
                index = pairs[index] + 1
                continue
            if text(index) == ",":
                found.append((begin, index))
                begin = index + 1
            index += 1
        found.append((begin, end))
        return [part for part in found if part[0] < part[1]]

    def attribute(start, end):
        if text(start) == "cfg" and text(start + 1) == "(" and (start + 1) in pairs:
            predicate(start + 2, pairs[start + 1])
        elif text(start) == "cfg_attr" and text(start + 1) == "(" and (start + 1) in pairs:
            pieces = parts(start + 2, pairs[start + 1])
            if pieces:
                predicate(*pieces[0])
                for piece in pieces[1:]:
                    attribute(*piece)

    index = low
    while index < high:
        if text(index) == "#" and text(index + 1) == "[" and (index + 1) in pairs:
            attribute(index + 2, pairs[index + 1])
        elif text(index) == "#" and text(index + 1) == "!" and text(index + 2) == "[" and (index + 2) in pairs:
            attribute(index + 3, pairs[index + 2])
        elif kinds[index] == IDENT and text(index) == "cfg" and text(index + 1) == "!" and text(index + 2) in ("(", "[", "{") and (index + 2) in pairs:
            predicate(index + 3, pairs[index + 2])
        index += 1
    return out


def parse_cfg_tokens(kinds, texts):
    """cfg 谓词的词法单元 → 树：("name", 名) / ("pair", 键, 值) / ("all"|"any"|"not", [子树…])；认不出抛 ValueError。"""
    position = 0
    total = len(texts)

    def expression():
        nonlocal position
        if position >= total or kinds[position] != IDENT:
            raise ValueError("要一个名字")
        name = texts[position]
        position += 1
        if name in ("all", "any", "not") and position < total and texts[position] == "(":
            position += 1
            items = []
            while True:
                if position >= total:
                    raise ValueError("括号没收")
                if texts[position] == ")":
                    position += 1
                    break
                items.append(expression())
                if position < total and texts[position] == ",":
                    position += 1
                elif position < total and texts[position] == ")":
                    continue
                else:
                    raise ValueError("要逗号或右括号")
            if name == "not" and len(items) != 1:
                raise ValueError("not 只收一项")
            return (name, items)
        if position < total and texts[position] == "=":
            position += 1
            if position >= total or kinds[position] != LITERAL:
                raise ValueError("= 后面要串")
            value = rust_string_value(texts[position])
            position += 1
            if value is None:
                raise ValueError("= 后面要串")
            return ("pair", name, value)
        if position < total and texts[position] == "(":
            raise ValueError(f"认不出 {name}(…)")
        return ("name", name)

    tree = expression()
    if position != total:
        raise ValueError("后面还有")
    return tree


class RustFile:
    def __init__(self, root, path, text, crate_directory):
        self.root, self.path = root, path
        self.directory = posixpath.dirname(path)
        self.crate_directory = crate_directory
        self.breaks = active_breaks()
        self._manifest_constants = None   # manifest_constants 第一次用到时算
        self.source_rows = text.split("\n")
        self.kinds, self.texts, self.lines, self.documents = rust_lex_cached(text)
        self.count = len(self.texts)
        self.units, self.refusals, self.mod_declarations, self.doctests = [], [], [], []
        self.verus_spans = []           # [(verus! 块的 { 下标, 配对的 } 下标)]：产品行为视图认只给 Verus 看的单元用（文件头「判定种类」）
        # 文档测试挂在哪（照 crates/singlefs-unit-graph）：条目的外文档注释，与模块开头（文件开头、行内模块体开头到内属性收尾）的内文档注释
        self.module_documents = []      # [(内文档注释可落的下标区间 低, 高, 文件里的行内模块路径)]
        self.current_header, self.current_module_head = 0, None
        self.pairs, bad = match_brackets(self.kinds, self.texts)
        if self.pairs is None:
            line = self.lines[bad] if bad is not None and bad < self.count else 1
            self.refusals.append(Refusal("unbalanced-brackets", path, line, self.row(line)))
            self.pairs = {}
            return
        inner_end, inner_conditions = self.inner_attributes(0, self.count)
        if inner_end > 0 and "rust-module-heads-dropped" not in self.breaks:
            # 文件开头的内属性 #![…]：一个模块头单元（限定名 <module attributes>），这份文件里的单元都连它
            self.current_module_head = self.add("mod_head", "<module attributes>", 0, inner_end, [], [], None, inner_conditions, False)
        self.module_documents.append((0, inner_end, []))
        self.scan(0, self.count, [], [], None, [], False)
        self.units.sort(key=lambda unit: unit.first_token)
        for unit in self.units:
            self.analyse(unit)
        self.extract_doctests()
        self.units.extend(self.doctests)

    def row(self, line):
        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def kind(self, index):
        return self.kinds[index] if 0 <= index < self.count else ""

    def inside_verus(self, unit):
        """单元落在某一块 verus! { … } 里。"""
        return unit.first_token is not None and any(opening < unit.first_token < close for opening, close in self.verus_spans)

    def refuse(self, unit, form, index):
        line = self.lines[index] if 0 <= index < self.count else unit.first_line
        unit.refusals.append(Refusal(form, self.path, line, self.row(line)))

    def add(self, kind, name, start, end, context, module_path, container, conditions, public):
        end = max(end, start + 1)
        pieces = self.texts[start:end]
        if "rust-doc-comments-kept" in self.breaks:
            pieces = pieces + ["\n".join(rows) for _line, _style, rows, next_index, _raw in self.documents if start <= next_index < end]
        unit = Unit("rust", kind, "::".join(context + [name]), self.path, self.lines[start], self.lines[min(end, self.count) - 1],
                    sha256_text("\x1f".join(pieces)))
        unit.short = name
        unit.first_token, unit.last_token = start, min(end, self.count)
        unit.container, unit.conditions, unit.scope, unit.public = container, list(conditions), tuple(module_path), public
        unit.self_types, unit.implicit_keys, unit.is_trait_impl, unit.blanket = set(), set(), False, False
        # header：属性之后、可见性起的那个词法单元（外文档注释落在 [首词法单元, header] 里的挂这个条目）；module_head：所在模块的模块头单元
        unit.header, unit.module_head = max(self.current_header, start), self.current_module_head
        self.units.append(unit)
        return unit

    # 条目的边界

    def function_end(self, k, high):
        c = k + 2
        while c < high:
            text = self.t(c)
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == "{" and c in self.pairs:
                return self.pairs[c] + 1
            if text == ";":
                return c + 1
            c += 1
        return high

    def head_end(self, k, high):
        c = k
        while c < high and self.t(c) not in ("{", ";"):
            c = self.pairs[c] + 1 if self.t(c) in ("(", "[") and c in self.pairs else c + 1
        return c

    def statement_end(self, k, high):
        c = k
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def type_end(self, c, high):
        while c < high:
            text = self.t(c)
            if text == "{" and c in self.pairs:
                end = self.pairs[c] + 1
                return end + 1 if self.t(end) == ";" else end
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def unknown_end(self, k, high):
        c = k
        while c < high:
            text = self.t(c)
            if text == "{" and c in self.pairs:
                end = self.pairs[c] + 1
                return end + 1 if self.t(end) == ";" else end
            if text in ("(", "[") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return high

    def macro_path_at(self, c):
        """c 起是不是一个宏调用 a::b::m!(…)：是就回宏名那一个词法单元的下标，否则 None。"""
        j = c
        if self.t(j) == ":" and self.t(j + 1) == ":":
            j += 2
        while self.kind(j) == IDENT and self.t(j + 1) == ":" and self.t(j + 2) == ":":
            j += 3
        if self.kind(j) == IDENT and self.t(j + 1) == "!" and self.t(j + 2) in ("(", "[", "{") and (j + 2) in self.pairs:
            return j
        return None

    def item_starts_at(self, c, high):
        if c >= high:
            return True
        text = self.t(c)
        if text == "#" and self.t(c + 1) in ("[", "!"):
            return True
        if text == ";":
            # ; 本身不是起头：} 之后紧跟的 ; 归前一个条目（const X: S = S { … }; 的指纹带它，crates/singlefs-unit-graph 的 verus_item_starts 同）
            return "rust-verus-semicolon-split-off" in self.breaks
        if text in RUST_ITEM_STARTS:
            return True
        if self.is_assume_specification_keyword(c):
            return True
        return self.macro_path_at(c) is not None

    def is_assume_specification_keyword(self, c):
        """c 是 verus! 里 assume_specification 条目的关键字：这个词后面接 [（点名的路径）或 <（泛型参数表）
        （crates/singlefs-unit-graph 的 is_assume_specification_keyword）。弄坏开关 rust-assume-specifications-unrecognized 下不认。"""
        return (self.t(c) == "assume_specification" and self.t(c + 1) in ("[", "<")
                and "rust-assume-specifications-unrecognized" not in self.breaks)

    def assume_specification_target_name(self, after_keyword, end):
        """assume_specification 之后（可带一段 <…> 泛型参数表）那对方括号里点名的路径，末段的名字（尖括号里的名字不算）：
        [ singlefs_format::WIDE_CHECKSUM_BYTES ] 交 WIDE_CHECKSUM_BYTES，[ Vec::<Element>::len ] 交 len；
        认不出那对方括号、方括号里尖括号外一个名字都没有的交 None（crates/singlefs-unit-graph 的 assume_specification_target_name）。"""
        open_bracket = after_keyword
        if self.t(after_keyword) == "<":
            depth, open_bracket = 0, None
            for position in range(after_keyword, end):
                if self.t(position) == "<":
                    depth += 1
                elif self.t(position) == ">" and (position == 0 or self.t(position - 1) != "-"):
                    depth -= 1
                    if depth < 0:
                        return None
                    if depth == 0:
                        open_bracket = position + 1
                        break
            if open_bracket is None:
                return None
        if self.t(open_bracket) != "[" or open_bracket not in self.pairs:
            return None
        angle_depth, last_name = 0, None
        for position in range(open_bracket + 1, self.pairs[open_bracket]):
            text = self.t(position)
            if text == "<":
                angle_depth += 1
            elif text == ">" and self.t(position - 1) != "-":
                angle_depth = max(angle_depth - 1, 0)
            elif angle_depth == 0 and self.kind(position) == IDENT:
                last_name = text
        return last_name

    def verus_end(self, k, high):
        """verus! 里一个条目：到下一个同层条目起头之前（; 或 } 之后接条目起头）。"""
        c = k
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                if text == "{" and self.item_starts_at(c, high):
                    return c
                continue
            c += 1
            if text == ";" and self.item_starts_at(c, high):
                return c
        return high

    def top_level_parts(self, low, high):
        parts, start, c = [], low, low
        while c < high:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ",":
                parts.append((start, c))
                start = c + 1
            c += 1
        parts.append((start, high))
        return [part for part in parts if part[0] < part[1]]

    # 属性

    def condition(self, low, high, line):
        raw = " ".join(self.texts[low:high])
        try:
            tree = parse_cfg_tokens(self.kinds[low:high], self.texts[low:high])
        except ValueError:
            tree = None
        return (tree, raw, line)

    def attribute_conditions(self, low, high):
        word = self.t(low)
        line = self.lines[low] if low < self.count else 1
        if word == "cfg" and self.t(low + 1) == "(" and (low + 1) in self.pairs:
            return [self.condition(low + 2, self.pairs[low + 1], line)]
        if word == "test" and high == low + 1:
            return [(("name", "test"), TEST_ATTRIBUTE_RAW, line)]
        if word == "cfg_attr" and self.t(low + 1) == "(" and (low + 1) in self.pairs:
            parts = self.top_level_parts(low + 2, self.pairs[low + 1])
            conditions = []
            if len(parts) >= 2:
                predicate = self.condition(parts[0][0], parts[0][1], line)
                for part_low, _part_high in parts[1:]:
                    if self.t(part_low) == "cfg" and self.t(part_low + 1) == "(" and (part_low + 1) in self.pairs:
                        inner = self.condition(part_low + 2, self.pairs[part_low + 1], line)
                        tree = None
                        if predicate[0] is not None and inner[0] is not None:
                            tree = ("any", [("not", [predicate[0]]), inner[0]])
                        conditions.append((tree, f"cfg_attr({predicate[1]}, cfg({inner[1]}))", line))
            return conditions
        return []

    def path_attribute(self, attributes):
        for low, _high in attributes:
            if self.t(low) == "path" and self.t(low + 1) == "=" and self.kind(low + 2) == LITERAL:
                return rust_string_value(self.t(low + 2))
        return None

    # 切

    def skip_qualifiers(self, k):
        while True:
            word = self.t(k)
            if word in RUST_QUALIFIERS and self.t(k + 1) == "(" and (k + 1) in self.pairs and self.kind(self.pairs[k + 1] + 1) == IDENT:
                k = self.pairs[k + 1] + 1
                continue
            if word in RUST_QUALIFIERS and self.kind(k + 1) == IDENT:
                k += 1
                continue
            if word == "const" and self.t(k + 1) in ("fn", "unsafe", "async", "extern"):
                k += 1
                continue
            if word == "extern" and self.kind(k + 1) == LITERAL and self.t(k + 2) in ("fn", "unsafe"):
                k += 2
                continue
            if word == "extern" and self.t(k + 1) == "fn":
                k += 1
                continue
            return k

    def inner_attributes(self, low, high):
        """low 起连着的内属性 #![…] → (它们之后的下标, 它们的 cfg 条件)；模块头单元从条目起头到这里。"""
        i, conditions = low, []
        while i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
            close = self.pairs[i + 2]
            conditions += self.attribute_conditions(i + 3, close)
            i = close + 1
        return i, conditions

    def scan(self, low, high, context, module_path, container, conditions, verus):
        i = low
        while i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
            close = self.pairs[i + 2]
            conditions = conditions + self.attribute_conditions(i + 3, close)
            i = close + 1
        while i < high:
            start = i
            item_conditions = list(conditions)
            attributes = []
            while i < high and self.t(i) == "#" and self.t(i + 1) == "[" and (i + 1) in self.pairs:
                close = self.pairs[i + 1]
                item_conditions += self.attribute_conditions(i + 2, close)
                attributes.append((i + 2, close))
                i = close + 1
            if i < high and self.t(i) == "#" and self.t(i + 1) == "!" and self.t(i + 2) == "[" and (i + 2) in self.pairs:
                i = self.pairs[i + 2] + 1
                continue
            if i >= high:
                break
            if self.t(i) == ";":
                i += 1
                continue
            self.current_header = i
            keyword, public = i, False
            if self.t(keyword) == "pub":
                keyword += 1
                if self.t(keyword) == "(" and keyword in self.pairs:
                    keyword = self.pairs[keyword] + 1
                else:
                    public = True
            keyword = self.skip_qualifiers(keyword)
            end = self.item(start, keyword, high, context, module_path, container, item_conditions, public, attributes, verus)
            i = max(end, i + 1)

    def item(self, start, k, high, context, module_path, container, conditions, public, attributes, verus):
        word = self.t(k)
        member_visible = container is not None and (container.kind == "trait" or bool(container.is_trait_impl))
        line = self.lines[k] if k < self.count else self.lines[start]
        if word == "fn" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.function_end(k, high)
            self.add("fn", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        keyword = k + 1 if word == "auto" and self.t(k + 1) == "trait" else k
        if self.t(keyword) in ("impl", "trait") and word in ("impl", "trait", "auto"):
            head_end = self.head_end(keyword, high)
            is_impl = self.t(keyword) == "impl"
            if self.t(head_end) != "{" or head_end not in self.pairs:
                end = head_end + 1 if self.t(head_end) == ";" else head_end
                if is_impl:
                    head = self.impl_head(keyword, head_end)
                    unit = self.add("impl", head["label"], start, end, context, module_path, container, conditions, True)
                    self.set_impl_head(unit, head)
                else:
                    self.add("trait", self.t(keyword + 1), start, end, context, module_path, container, conditions, public or member_visible)
                return end
            close = self.pairs[head_end]
            if is_impl:
                head = self.impl_head(keyword, head_end)
                unit = self.add("impl", head["label"], start, head_end, context, module_path, container, conditions, True)
                self.set_impl_head(unit, head)
                label = head["label"]
            else:
                name = self.t(keyword + 1)
                unit = self.add("trait", name, start, head_end, context, module_path, container, conditions, public or member_visible)
                label = f"<trait {name}>"
            self.scan(head_end + 1, close, context + [label], module_path, unit, conditions, verus)
            return close + 1
        if word == "mod" and self.kind(k + 1) == IDENT:
            name = self.t(k + 1)
            if self.t(k + 2) == "{" and (k + 2) in self.pairs:
                close = self.pairs[k + 2]
                # 行内模块头：条目起头（外属性）到 { 与紧跟的内属性 #![…]（verus! 里不收内属性，到 { 为止）；模块里的单元都连它
                inner_end, inner_conditions = (k + 3, []) if verus else self.inner_attributes(k + 3, close)
                outer_head = self.current_module_head
                if "rust-module-heads-dropped" not in self.breaks:
                    self.current_module_head = self.add("mod_head", name, start, inner_end, context, module_path, container,
                                                        conditions + inner_conditions, public)
                if not verus:
                    self.module_documents.append((k + 3, inner_end, list(context) + [name]))
                self.scan(k + 3, close, context + [name], module_path + [name], None, conditions, verus)
                self.current_module_head = outer_head
                return close + 1
            if self.t(k + 2) == ";":
                unit = self.add("mod", name, start, k + 3, context, module_path, container, conditions, public)
                self.mod_declarations.append((unit, name, self.path_attribute(attributes), list(module_path)))
                return k + 3
        if word == "macro_rules" and self.t(k + 1) == "!" and self.kind(k + 2) == IDENT and (k + 3) in self.pairs:
            end = self.pairs[k + 3] + 1
            if self.t(end) == ";":
                end += 1
            exported = any(self.t(low) == "macro_export" for low, _high in attributes)
            self.add("macro_rules", self.t(k + 2), start, end, context, module_path, container, conditions, exported)
            return end
        if word == "use":
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            # verus! 里的 broadcast use 照 crates/singlefs-unit-graph 写 broadcast-use@<broadcast 那一行>
            label = f"broadcast-use@{self.lines[k - 1]}" if verus and k - 1 >= start and self.t(k - 1) == "broadcast" else f"use@{line}"
            unit = self.add("use", label, start, end, context, module_path, container, conditions, public)
            unit.imports, unit.aliases, unit.import_glob = self.use_imports(k + 1, end - 1 if self.t(end - 1) == ";" else end)
            return end
        if verus and word == "group" and k - 1 >= start and self.t(k - 1) == "broadcast" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high)
            self.add("broadcast_group", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if verus and self.is_assume_specification_keyword(k):
            # 整条（连属性）一个单元，名字 assume_specification@<关键字那一行>，按点名路径末段的名字找得到它（defines）；
            # 名字认不出时这份文件里谁连得到它说不全，记文件级拒算（Rust 那一份记 unrecognized-item-start）
            end = self.verus_end(k, high)
            target_name = self.assume_specification_target_name(k + 1, end)
            if target_name is None:
                self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
                return end
            unit = self.add("assume_specification", f"assume_specification@{line}", start, end, context, module_path, container, conditions,
                            public or member_visible)
            unit.defines = set() if "rust-assume-specification-names-unlinked" in self.breaks else {target_name}
            return end
        if word in ("struct", "enum", "union") and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.type_end(k + 2, high)
            self.add(word, self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word == "type" and self.kind(k + 1) == IDENT:
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            self.add("type", self.t(k + 1), start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word in ("const", "static") and self.kind(k + 1) == IDENT:
            name = self.t(k + 2) if word == "static" and self.t(k + 1) == "mut" else self.t(k + 1)
            end = self.verus_end(k, high) if verus else self.statement_end(k, high)
            self.add(word, name, start, end, context, module_path, container, conditions, public or member_visible)
            return end
        if word == "extern" and self.t(k + 1) == "crate":
            end = self.statement_end(k, high)
            self.add("extern_crate", self.t(k + 2), start, end, context, module_path, container, conditions, public)
            return end
        if word == "extern" and (self.t(k + 1) == "{" or (self.kind(k + 1) == LITERAL and self.t(k + 2) == "{")):
            brace = k + 1 if self.t(k + 1) == "{" else k + 2
            if brace in self.pairs:
                end = self.pairs[brace] + 1
                unit = self.add("extern_block", f"extern@{line}", start, end, context, module_path, container, conditions, True)
                # fn / static 之后的名字（static mut 跳过 mut，与 crates/singlefs-unit-graph 的 extern_block_declared_names 同一口径）
                unit.defines = {self.t(j + 2) if self.t(j) == "static" and self.t(j + 1) == "mut" else self.t(j + 1) for j in range(brace, end)
                                if self.t(j) in ("fn", "static") and self.kind(j + 2 if self.t(j) == "static" and self.t(j + 1) == "mut" else j + 1) == IDENT}
                unit.listed_c_library_calls = self.extern_block_declares_only_listed_c_library_calls(start, k, brace, self.pairs[brace])
                return end
        name_index = self.macro_path_at(k)
        if name_index is not None:
            name = self.t(name_index)
            opening = name_index + 2
            close = self.pairs[opening]
            end = close + 1
            if self.t(end) == ";":
                end += 1
            if name == "verus" and self.t(opening) == "{":
                self.verus_spans.append((opening, close))
                self.scan(opening + 1, close, context, module_path, container, conditions, "rust-verus-segments-ignored" not in self.breaks)
                return end
            if name in RUST_ITEM_MACROS_HOLDING_ITEMS and self.t(opening) == "{":
                self.scan(opening + 1, close, context, module_path, container, conditions, verus)
                return end
            unit = self.add("item_macro", f"{name}!@{self.lines[name_index]}", start, end, context, module_path, container, conditions, True)
            keywords = RUST_KEYWORDS | (UNION_KEYWORD if "rust-union-as-keyword" in self.breaks else frozenset())
            unit.defines = {self.texts[j] for j in range(opening + 1, close) if self.kinds[j] == IDENT} - keywords
            return end
        end = self.verus_end(k, high) if verus else self.unknown_end(k, high)
        self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
        return end

    def set_impl_head(self, unit, head):
        unit.self_types, unit.trait_name = head["self_types"], head["trait_name"]
        unit.is_trait_impl, unit.blanket, unit.implicit_keys = head["is_trait_impl"], head["blanket"], head["implicit_keys"]
        unit.open_keys, unit.self_path_root = head["open_keys"], head["self_path_root"]

    def impl_head(self, keyword, head_end):
        tokens = list(range(keyword + 1, head_end))
        generics = set()
        if tokens and self.t(tokens[0]) == "<":
            depth, cut = 0, len(tokens)
            for position, index in enumerate(tokens):
                text = self.t(index)
                if text == "<":
                    depth += 1
                elif text == ">" and self.t(index - 1) != "-":
                    depth -= 1
                elif depth == 1 and self.kind(index) == IDENT and self.t(index - 1) in ("<", ",", "const") and text != "const":
                    generics.add(text)
                if depth == 0:
                    cut = position + 1
                    break
            tokens = tokens[cut:]
        where = next((position for position, index in enumerate(tokens) if self.t(index) == "where"), None)
        if where is not None:
            tokens = tokens[:where]
        label = "<impl " + re.sub(r"\s*: :\s*", "::", " ".join(self.t(index) for index in tokens)) + ">"
        depth, split = 0, None
        for position, index in enumerate(tokens):
            text = self.t(index)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(index - 1) != "-":
                depth -= 1
            elif text == "for" and depth == 0 and self.t(index + 1) != "<":
                split = position
                break
        trait_tokens = tokens[:split] if split is not None else []
        self_tokens = tokens[split + 1:] if split is not None else tokens
        trait_name, trait_arguments, depth = None, set(), 0
        for index in trait_tokens:
            text = self.t(index)
            if text == "<":
                depth += 1
            elif text == ">" and self.t(index - 1) != "-":
                depth -= 1
            elif self.kind(index) == IDENT:
                if depth == 0 and text != "dyn":
                    trait_name = text
                elif depth > 0:
                    trait_arguments.add(text)
        self_names = [self.t(index) for index in self_tokens if self.kind(index) == IDENT and self.t(index) not in SELF_TYPE_SKIPPED]
        self_types = set(self_names)
        # 自身类型写成路径 a::B 时的头一段（外部类型的 impl 认法用，RustWorkspace.index 的 foreign_trait_impls）；不是路径的是 None
        first_name = next((index for index in self_tokens if self.kind(index) == IDENT and self.t(index) not in SELF_TYPE_SKIPPED), None)
        self_path_root = self.t(first_name) if first_name is not None and self.t(first_name + 1) == ":" and self.t(first_name + 2) == ":" else None
        # blanket：自身类型里任何深度出现 impl 的任一类型或常量泛型参数（for T、&T、Box<T>、(T, u32)、[T]、[T; N]、Option<Box<T>>）；全是具体类型的按被点名的类型。
        # 弄坏开关 rust-blanket-self-parameter-only 退回旧判法：自身类型里头一个名字是泛型参数才算（Box<T>、Option<Box<T>> 不算）
        if "rust-blanket-self-parameter-only" in self.breaks:
            blanket = split is not None and bool(self_names) and self_names[0] in generics
        else:
            blanket = split is not None and bool(self_types & generics)
        return {"label": label, "trait_name": trait_name, "is_trait_impl": split is not None, "self_types": self_types,
                "blanket": blanket, "self_path_root": self_path_root,
                # R-隐式按自身类型的名字认 trait impl（crates/singlefs-unit-graph 的 implementations_by_self_type_name）：trait 参数里的类型被点名不带进来
                "implicit_keys": self_types | (trait_arguments if "rust-trait-impls-by-trait-arguments" in self.breaks else set()),
                # R-开放按 trait 自己的名字找 impl（impl Wrap<dyn Sink> for H 不是 Sink 的 impl）；
                # 弄坏开关 rust-open-by-trait-arguments 下另按 trait 参数里的名字找（crates/singlefs-unit-graph 现在按 trait_part_names 建索引的写法）
                "open_keys": ({trait_name} if trait_name else set())
                             | (trait_arguments - {"dyn"} if "rust-open-by-trait-arguments" in self.breaks else set())}

    def use_imports(self, low, high):
        """→ (引进的名字, [(别名, 原名)], 有没有 *)：路径段（后面接 ::）与 self / super / crate 不算名字，a as b 两边都算名字、记一条别名
        （as 两边都是标识符才记）；花括号里的 self 引进的是 { 前面那一段（use std::io::{self, Seek} 引进 io，self as c 记别名 (c, io)）。
        弄坏开关 rust-use-self-imports-ignored 下花括号里的 self 不引进名字（crates/singlefs-unit-graph 的 use_imports 现在的写法）。"""
        names, aliases, glob_import = set(), [], False
        self_imports = "rust-use-self-imports-ignored" not in self.breaks
        braces = []           # 套着的每一层 { 前面那一段（不是名字的记 None）

        def original(j):
            if self_imports and self.t(j) == "self" and braces and self.t(j + 1) != ":":
                return braces[-1]
            return self.t(j)

        for j in range(low, high):
            text = self.t(j)
            if text == "*":
                glob_import = True
            elif text == "{":
                segment = self.t(j - 3) if self.t(j - 1) == ":" and self.t(j - 2) == ":" and self.kind(j - 3) == IDENT else None
                braces.append(segment if segment not in ("self", "super", "crate") else None)
            elif text == "}":
                if braces:
                    braces.pop()
            elif self.kind(j) == IDENT:
                if text == "as":
                    if self.kind(j + 1) == IDENT and j > low and self.kind(j - 1) == IDENT and original(j - 1) is not None:
                        aliases.append((self.t(j + 1), original(j - 1)))
                    continue
                if self.t(j + 1) == ":" and self.t(j + 2) == ":":
                    continue
                # `a::b as c`：原名 b 不进当前作用域，进的只有别名 c（Rust 那一份 rust_items.rs 的 use_imports 同一条）
                if self.t(j + 1) == "as":
                    continue
                if text not in ("self", "super", "crate"):
                    names.add(text)
                elif text == "self" and original(j) not in (None, "self"):
                    names.add(original(j))
        return names, aliases, glob_import

    # 单元读到的名字与数据

    def analyse(self, unit):
        unit.plain, unit.methods, unit.qualified, unit.dyn_traits = token_references(self.kinds, self.texts, unit.first_token, unit.last_token)
        unit.cfg_leaves = cfg_predicates_in_tokens(self.kinds, self.texts, self.lines, self.pairs, unit.first_token, unit.last_token)
        unit.references = unit.plain | unit.methods
        unit.data_references = []
        self.data_references(unit)

    def data_references(self, unit):
        i, end = unit.first_token, unit.last_token
        while i < end:
            text, kind = self.texts[i], self.kinds[i]
            if kind == IDENT and self.t(i + 1) == "!" and self.t(i + 2) in ("(", "[", "{") and (i + 2) in self.pairs:
                close = self.pairs[i + 2]
                if text in ("include_str", "include_bytes"):
                    self.include_reference(unit, i, i + 3, close)
                    i = close + 1
                    continue
                if text == "include":
                    self.refuse(unit, "include-code", i)
                    i = close + 1
                    continue
                if text in ("concat_idents", "paste"):
                    self.refuse(unit, "macro-generated-names", i)
                    i = close + 1
                    continue
                if text == "concat":
                    nested = [] if "rust-concatenated-includes-skipped" in self.breaks else self.concatenated_includes(unit, i + 3, close)
                    self.runtime_concat(unit, i, i + 3, close, nested)
                    i = close + 1
                    continue
                if text == "env" and self.kind(i + 3) == LITERAL:
                    value = rust_string_value(self.t(i + 3))
                    if value == "CARGO_MANIFEST_DIR":
                        self.runtime_manifest_path(unit, i, close)
                    elif value and value.startswith("CARGO_BIN_EXE_"):
                        unit.data_references.append(("bin-executable", value[len("CARGO_BIN_EXE_"):], i))
                    elif value == "OUT_DIR":
                        unit.data_references.append(("out-directory", None, i))
                    elif value and not value.startswith("CARGO"):
                        # cargo 不给的变量：值只能来自构建脚本的 cargo::rustc-env（文件头 Rust 闭包「构建脚本交的值」）
                        unit.data_references.append(("build-environment", value, i))
                    i = close + 1
                    continue
                if text == "option_env" and self.kind(i + 3) == LITERAL:
                    value = rust_string_value(self.t(i + 3))
                    if value and value != "OUT_DIR" and not value.startswith("CARGO"):
                        unit.data_references.append(("build-environment", value, i))
                    i = close + 1
                    continue
            if kind == IDENT and text in self.manifest_constants()[0] and self.t(i - 1) != "const":
                self.manifest_constant_use(unit, i)
            if kind == IDENT and text == "current_exe" and self.t(i + 1) == "(":
                child = None if "rust-current-exe-refused" in self.breaks else self.current_exe_child(unit, i)
                if child is not None:
                    unit.data_references.append(("current-exe-child", child, i))
                elif "rust-current-exe-path-only-refused" in self.breaks:
                    self.refuse(unit, "current-exe", i)
                elif "rust-own-program-restart-refused" not in self.breaks and self.own_program_path_is_handed_to_command_unchanged(i, unit.last_token):
                    unit.data_references.append(("own-program-restart", None, i))
                elif self.starts_a_process(unit) and "rust-current-exe-spawn-unrefused" not in self.breaks:
                    self.refuse(unit, "current-exe", i)
            if kind == IDENT and text in ("var_os", "var") and self.t(i + 1) == "(" and rust_string_value(self.t(i + 2)) == "CARGO_MANIFEST_DIR":
                self.refuse(unit, "runtime-path", i)
            i += 1

    # current_exe()：测试二进制起自己（子进程里只跑一条点名的测试）

    def current_exe_child(self, unit, index):
        """current_exe 这个词在 index。写成 Command::new(…current_exe()…) 后接 .arg(…) / .args([…]) 链、参数里恰好有一个字面 "--exact"、
        紧跟在它后面的那个参数折得出测试名的，→ 要连的测试：("pieces", 几段)（几段是 ("literal", 串) 与 ("constant", 常量名)：字面串、常量名、
        全由这两样组成的 format!("…{}…{名}…", …) 与 concat!(…)；常量名在 rust_closure_of 的 constant_string 里解，不在这里找）、("parameter", 函数名, 位置)
        （名字是本单元这个 fn 自己的参数：rust_closure_of 取闭包里每一处调用它的那个实参各折一遍）；折不出的回 None（拒算 current-exe）。
        测试名写在 --exact 前面、用 module_path!() 拼的、套着 as_str() 的、在单元里另有 let 绑定的，都折不出。"""
        j = index
        while self.t(j - 1) == ":" and self.t(j - 2) == ":" and self.kind(j - 3) == IDENT:
            j -= 3
        opening = j - 1
        if not (self.t(opening) == "(" and self.t(opening - 1) == "new" and self.t(opening - 2) == ":" and self.t(opening - 3) == ":"
                and self.t(opening - 4) == "Command" and opening in self.pairs):
            return None
        arguments, c = [], self.pairs[opening] + 1
        while self.t(c) == "." and self.kind(c + 1) == IDENT and self.t(c + 2) == "(" and (c + 2) in self.pairs:
            method, inner_low, inner_close = self.t(c + 1), c + 3, self.pairs[c + 2]
            if method == "arg":
                arguments += self.top_level_parts(inner_low, inner_close)
            elif method == "args":
                k = inner_low + 1 if self.t(inner_low) == "&" else inner_low
                if not (self.t(k) == "[" and self.pairs.get(k) == inner_close - 1):
                    return None
                arguments += self.top_level_parts(k + 1, inner_close - 1)
            c = inner_close + 1
        flags = [position for position, (part_low, part_high) in enumerate(arguments)
                 if part_high == part_low + 1 and self.kind(part_low) == LITERAL and rust_string_value(self.t(part_low)) == "--exact"]
        if len(flags) != 1 or flags[0] + 1 >= len(arguments):
            return None
        return self.fold_test_name(unit, arguments[flags[0] + 1][0], arguments[flags[0] + 1][1], 0)

    def starts_a_process(self, unit):
        """这个单元里有没有起子进程的词（SPAWNING_WORDS）。没有的话 current_exe() 只是在问路径，不拒算；单元没有词法范围的按起了算。"""
        if unit.first_token is None:
            return True
        return any(self.kind(i) == IDENT and self.t(i) in SPAWNING_WORDS for i in range(unit.first_token, unit.last_token))

    # 程序起自己与 extern 块（文件头同名两节；crates/singlefs-unit-graph 的 rust_items.rs 同名函数逐条同义，门禁 code-tooling 的 function-units-two-implementations-agree 对拍）

    def expression_ending_at_diverges(self, last_token, first_allowed):
        """last_token 是一个表达式的最后一个词法单元时，这个表达式是否一定不交值：写成 panic!(…)、…process::exit(…)，或以它们收尾的块。
        块只认紧跟闭包参数（开括号前面是 |）与紧跟 => 的两种：块前面是 else、)、if 的条件这类的，是分支里的一支，别的分支可能交值。
        弄坏开关 rust-own-program-divergence-any-block 下任何块都往里看。"""
        text = self.t(last_token)
        if text == "}":
            if "rust-own-program-divergence-any-block" not in self.breaks:
                opening = next((position for position in range(last_token - 1, first_allowed - 1, -1) if self.t(position) == "{" and self.pairs.get(position) == last_token), None)
                if opening is None:
                    return False
                follows_closure_parameters = self.t(opening - 1) == "|"
                follows_a_match_arrow = opening >= 2 and self.t(opening - 1) == ">" and self.t(opening - 2) == "="
                if not (follows_closure_parameters or follows_a_match_arrow):
                    return False
            before_the_brace = max(last_token - 1, 0)
            return self.expression_ending_at_diverges(max(before_the_brace - 1, 0) if self.t(before_the_brace) == ";" else before_the_brace, first_allowed)
        if text != ")":
            return False
        opening = next((position for position in range(last_token - 1, first_allowed - 1, -1) if self.t(position) == "(" and self.pairs.get(position) == last_token), None)
        if opening is None:
            return False
        is_panic = opening >= 2 and self.t(opening - 1) == "!" and self.t(opening - 2) == "panic"
        is_process_exit = (opening >= 4 and self.t(opening - 1) == "exit" and self.t(opening - 2) == ":" and self.t(opening - 3) == ":"
                           and self.t(opening - 4) == "process")
        return is_panic or is_process_exit

    def own_path_expression_range(self, call_start, after_call, unit_end):
        """current_exe() 调用（闭括号之后是 after_call）连同把路径取出来的写法整个是什么表达式 → (起, 止)。认的写法：后接 ?、.unwrap()、.expect("字面串")、
        闭包一定不交值的 .unwrap_or_else(…)；或 match current_exe() { Ok(名字) => 名字, Err(…) => <return … 或一定不交值的表达式> }。别的写法 → None。"""
        if self.t(after_call) == "?":
            return call_start, after_call + 1
        if self.t(after_call) == "." and self.t(after_call + 2) == "(":
            opening = after_call + 2
            close = self.pairs.get(opening)
            if close is None:
                return None
            method = self.t(after_call + 1)
            is_taken = (method == "unwrap" and close == opening + 1) or (method == "expect" and close == opening + 2 and rust_plain_string(self.t(opening + 1)) is not None) or (
                method == "unwrap_or_else" and self.t(opening + 1) == "|" and close > opening + 1 and self.expression_ending_at_diverges(close - 1, opening + 1))
            return (call_start, close + 1) if is_taken else None
        if self.t(after_call) != "{" or self.t(call_start - 1) != "match":
            return None
        close = self.pairs.get(after_call)
        if close is None or close > unit_end or self.kind(after_call + 3) != IDENT or self.t(after_call + 3) != self.t(after_call + 7) \
                or [self.t(after_call + offset) for offset in (1, 2, 4, 5, 6, 8, 9, 10)] != ["Ok", "(", ")", "=", ">", ",", "Err", "("]:
            return None
        error_pattern_close = self.pairs.get(after_call + 10)
        if error_pattern_close is None or self.t(error_pattern_close + 1) != "=" or self.t(error_pattern_close + 2) != ">":
            return None
        arm_start, arm_end = error_pattern_close + 3, close - 1 if self.t(close - 1) == "," else close
        if arm_start >= arm_end or len(self.top_level_parts(arm_start, arm_end)) != 1:
            return None
        return (call_start - 1, close + 1) if self.t(arm_start) == "return" or self.expression_ending_at_diverges(arm_end - 1, arm_start) else None

    def is_command_new_open(self, opening):
        """opening 是 Command::new( 的开括号。"""
        return (opening >= 4 and self.t(opening) == "(" and self.t(opening - 1) == "new" and self.t(opening - 2) == ":" and self.t(opening - 3) == ":"
                and self.t(opening - 4) == "Command")

    def let_binding_name(self, expression_start, after_expression):
        """从 expression_start 起、到 after_expression 之前的表达式整个是 let [mut] 名字 [: 类型] = …; 的右边时交那个名字，别的交 None。"""
        if self.t(after_expression) != ";" or self.t(expression_start - 1) != "=":
            return None
        equals = expression_start - 1
        let_index = next((position for position in range(equals - 1, -1, -1) if self.t(position) in ("let", ";", "{", "}", "=")), None)
        if let_index is None or self.t(let_index) != "let":
            return None
        name_index = let_index + 2 if self.t(let_index + 1) == "mut" else let_index + 1
        if self.kind(name_index) != IDENT:
            return None
        is_typed = self.t(name_index + 1) == ":" and self.t(name_index + 2) != ":"
        return self.t(name_index) if name_index + 1 == equals or is_typed else None

    def own_program_path_is_handed_to_command_unchanged(self, current_exe_index, unit_end):
        """current_exe() 的路径是不是一个字不改地交给了 Command::new：就地写在 Command::new(…) 的括号里；或绑给 let 名字（单元里这个名字不再 let），
        名字的每一处用法只是 Command::new(名字)、Command::new(&名字) 与 名字.display()，至少起一次。拿路径拼别的路径、交给别的函数、当参数交给别的程序的不算。
        弄坏开关 rust-own-program-path-unchecked 下路径怎么走都算。"""
        if "rust-own-program-path-unchecked" in self.breaks:
            return True
        call_start = current_exe_index
        while call_start >= 3 and self.t(call_start - 1) == ":" and self.t(call_start - 2) == ":" and self.kind(call_start - 3) == IDENT:
            call_start -= 3
        if self.t(current_exe_index + 2) != ")":
            return False
        found = self.own_path_expression_range(call_start, current_exe_index + 3, unit_end)
        if found is None:
            return False
        expression_start, expression_end = found
        if expression_start > 0 and self.is_command_new_open(expression_start - 1) and self.pairs.get(expression_start - 1) == expression_end:
            return True
        name = self.let_binding_name(expression_start, expression_end)
        if name is None:
            return False
        if any(self.t(index) == "let" and (self.t(index + 1) == name or (self.t(index + 1) == "mut" and self.t(index + 2) == name)) for index in range(expression_end, unit_end)):
            return False
        uses = [index for index in range(expression_end, unit_end) if self.kind(index) == IDENT and self.t(index) == name and self.t(index - 1) != "."
                and not (self.t(index + 1) == ":" and self.t(index + 2) == ":")]

        def innermost_brace_around(index):
            return next((position for position in range(index - 1, -1, -1) if self.t(position) == "{" and self.pairs.get(position, -1) > index), None)

        # 每一处用法最内的那一层花括号都要是这条 let 所在的那一层：出了这个块的同名用法指的是别的绑定；
        # 进了更深的块（if、循环、闭包体）的用法走不走得到、块里起的是不是它，看不出。弄坏开关 rust-own-program-uses-any-block 下用法落在哪一层都不看
        block_of_the_let = innermost_brace_around(expression_start)
        if block_of_the_let is None:
            return False
        if "rust-own-program-uses-any-block" not in self.breaks and any(innermost_brace_around(index) != block_of_the_let for index in uses):
            return False

        def is_handed_to_command(index):
            opening = index - 2 if self.t(index - 1) == "&" else index - 1
            return self.is_command_new_open(opening) and self.pairs.get(opening) == index + 1

        def is_only_displayed(index):
            return [self.t(index + offset) for offset in (1, 2, 3, 4)] == [".", "display", "(", ")"]

        return any(is_handed_to_command(index) for index in uses) and all(is_handed_to_command(index) or is_only_displayed(index) for index in uses)

    def extern_block_declares_only_listed_c_library_calls(self, item_start, keyword, opening, close):
        """extern 块（keyword 是 extern，块在 opening..close）不用拒算：ABI 是 "C"，条目前面没有 #[link…]，块里只有 fn 声明，每条的前面最多一个 #[link_name = "…"]、
        符号（link_name 或函数名）都在 C_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS 里，至少声明一条。有 static、别的属性、safe fn 这类写法的一律不算。
        弄坏开关 rust-extern-listed-calls-refused 下一律不算，rust-extern-any-block-keyed 下一律算。"""
        if "rust-extern-listed-calls-refused" in self.breaks:
            return False
        if "rust-extern-any-block-keyed" in self.breaks:
            return True
        if rust_plain_string(self.t(keyword + 1)) != "C" or keyword + 2 != opening:
            return False
        if extern_block_attributes_name_link(self.kinds, self.texts, item_start, keyword):
            return False
        position, declared_any, allowed_calls = opening + 1, False, c_library_calls_allowed_in_extern_blocks()
        while position < close:
            link_name = None
            if self.t(position) == "#":
                if self.t(position + 1) != "[" or self.t(position + 2) != "link_name" or self.t(position + 3) != "=" or self.t(position + 5) != "]":
                    return False
                link_name = rust_plain_string(self.t(position + 4))
                if link_name is None:
                    return False
                position += 6
            if self.t(position) == "pub":
                position += 1
            if self.t(position) != "fn" or self.kind(position + 1) != IDENT or self.t(position + 2) != "(" or (position + 2) not in self.pairs:
                return False
            if (link_name if link_name is not None else self.t(position + 1)) not in allowed_calls:
                return False
            declared_any = True
            semicolon = next((index for index in range(self.pairs[position + 2], close) if self.t(index) == ";"), None)
            if semicolon is None:
                return False
            position = semicolon + 1
        return declared_any

    def fold_test_name(self, unit, low, high, depth):
        """测试名那一段词法单元 [low, high) → ("pieces", 几段) | ("parameter", 函数名, 位置) | None（折不出）；几段是 ("literal", 串) 与 ("constant", 常量名)。
        认：串字面量；常量名（只记名字，值在 rust_closure_of 的 constant_string 里解）；concat!(…)；format!("…{}…{名}…", …)（只认 {} 与 {名}，各块都折得出）；
        本单元这个 fn 的参数名。"""
        if depth > 6 or low >= high:
            return None
        if high == low + 1 and self.kind(low) == LITERAL:
            value = rust_string_value(self.t(low))
            return ("pieces", (("literal", value),)) if value is not None else None
        if self.kind(low) == IDENT and self.t(low + 1) == "!" and self.t(low + 2) == "(" and self.pairs.get(low + 2) == high - 1:
            return self.fold_test_name_macro(unit, self.t(low), low + 3, high - 1, depth)
        names = [low]
        while names[-1] + 3 < high and self.t(names[-1] + 1) == ":" and self.t(names[-1] + 2) == ":" and self.kind(names[-1] + 3) == IDENT:
            names.append(names[-1] + 3)
        if self.kind(low) == IDENT and names[-1] + 1 == high:
            return self.fold_test_name_binding(unit, self.t(names[-1]), depth)
        return None

    def fold_test_name_macro(self, unit, macro, low, high, depth):
        parts = self.top_level_parts(low, high)
        if macro == "concat":
            pieces = [self.fold_test_name(unit, part_low, part_high, depth + 1) for part_low, part_high in parts]
            if not pieces or any(piece is None or piece[0] != "pieces" for piece in pieces):
                return None
            return ("pieces", tuple(segment for piece in pieces for segment in piece[1]))
        if macro != "format" or not parts or parts[0][1] != parts[0][0] + 1 or self.kind(parts[0][0]) != LITERAL:
            return None
        template = rust_string_value(self.t(parts[0][0]))
        positional = parts[1:]
        if template is None or any(part[1] - part[0] >= 3 and self.kind(part[0]) == IDENT and self.t(part[0] + 1) == "=" for part in positional):
            return None
        pieces, next_argument = [], 0
        for found in re.finditer(r"\{\{|\}\}|\{([A-Za-z_][A-Za-z0-9_]*)?\}|[{}]|[^{}]+", template):
            text = found.group(0)
            if text in ("{{", "}}"):
                pieces.append(("literal", text[0]))
            elif text.startswith("{") and text.endswith("}") and len(text) >= 2:
                if found.group(1) is None:
                    if next_argument >= len(positional):
                        return None
                    piece = self.fold_test_name(unit, positional[next_argument][0], positional[next_argument][1], depth + 1)
                    next_argument += 1
                else:
                    piece = self.fold_test_name_binding(unit, found.group(1), depth + 1)
                if piece is None or piece[0] != "pieces":
                    return None
                pieces.extend(piece[1])
            elif text in ("{", "}"):
                return None
            else:
                pieces.append(("literal", text))
        if next_argument != len(positional):
            return None
        return ("pieces", tuple(pieces))

    def fold_test_name_binding(self, unit, name, depth):
        """一个名字折成什么：单元里另有 let 绑定的 None（折不出）；是本单元这个 fn 的参数的，("parameter", 函数名, 位置)；
        别的当常量名，("pieces", (("constant", 名),))，值由 rust_closure_of 的 constant_string 解（本单元里的 const，再是同一个包链接集里的常量单元）。"""
        if unit.first_token is not None and any(
                self.texts[i] == "let" and (self.t(i + 1) == name or (self.t(i + 1) == "mut" and self.t(i + 2) == name))
                for i in range(unit.first_token, unit.last_token)):
            return None
        position = self.parameter_position(unit, name)
        return ("parameter", unit.short, position) if position is not None else ("pieces", (("constant", name),))

    def string_constant_in(self, low, high, name):
        """[low, high) 里第一处 const <名> : … = "…" ; 的串值（普通串字面量，不折表达式）；没有这一处、或写法不是它回 None
        （crates/singlefs-unit-graph 的 constant_string_value 里 definition_value 同写）。"""
        if low is None:
            return None
        for i in range(low, high):
            if self.texts[i] == "const" and self.t(i + 1) == name and self.t(i + 2) == ":":
                equals = next((c for c in range(i + 3, high) if self.texts[c] == "="), None)
                if equals is not None and self.kind(equals + 1) == LITERAL and self.t(equals + 1)[:1] == '"' and self.t(equals + 2) == ";":
                    return rust_string_value(self.t(equals + 1))
                return None
        return None

    def parameter_position(self, unit, name):
        """unit 是 fn 单元、<名> 是它的参数（不数 self）时，它是第几个参数（从 0 数）；不是回 None。"""
        if unit.kind != "fn" or unit.first_token is None:
            return None
        i = unit.first_token
        while i < unit.last_token and self.texts[i] != "fn":
            i += 1
        i += 2
        if self.t(i) == "<":
            depth = 0
            while i < unit.last_token:
                depth += 1 if self.texts[i] == "<" else -1 if self.texts[i] == ">" and self.t(i - 1) != "-" else 0
                i += 1
                if depth == 0:
                    break
        if self.t(i) != "(" or i not in self.pairs:
            return None
        close, parts, start, depth, c = self.pairs[i], [], i + 1, 0, i + 1
        while c < close:
            text = self.texts[c]
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == "<":
                depth += 1
            elif text == ">" and self.t(c - 1) != "-":
                depth -= 1
            elif text == "," and depth == 0:
                parts.append((start, c))
                start = c + 1
            c += 1
        parts.append((start, close))
        count = 0
        for low, high in parts:
            words = [self.texts[k] for k in range(low, high) if self.kinds[k] != LIFETIME and self.texts[k] not in ("&", "mut")]
            if not words or words[0] == "self":
                continue
            if len(words) >= 2 and words[0] == name and words[1] == ":":
                return count
            count += 1
        return None

    def call_site_test_names(self, unit, function_name, position):
        """unit 的词法单元里每一处 <函数名>(…) 调用（不是 fn 定义、不是 .方法 调用）的第 position 个实参折出的测试名的几段 → [几段]（常量名由调用方在 unit 的上下文里解）；
        有一处折不出（实参不够、又是参数）回 None。"""
        found = []
        for i in range(unit.first_token or 0, unit.last_token or 0):
            if self.texts[i] != function_name or self.t(i + 1) != "(" or (i + 1) not in self.pairs or self.t(i - 1) in ("fn", "."):
                continue
            parts = self.top_level_parts(i + 2, self.pairs[i + 1])
            if position >= len(parts):
                return None
            reached = self.fold_test_name(unit, parts[position][0], parts[position][1], 0)
            if reached is None or reached[0] != "pieces":
                return None
            found.append(reached[1])
        return found

    def concat_value(self, low, high):
        """concat!(…) 的参数 → ("path", 相对根的路径) / ("relative", 串) / ("out", None) / None（认不出）。"""
        pieces, manifest = [], False
        for part_low, part_high in self.top_level_parts(low, high):
            if part_high == part_low + 1 and self.kind(part_low) == LITERAL:
                value = rust_string_value(self.t(part_low))
                if value is None:
                    return None
                pieces.append(value)
                continue
            if self.t(part_low) == "env" and self.t(part_low + 1) == "!" and self.t(part_low + 2) == "(" and part_high == part_low + 5:
                name = rust_string_value(self.t(part_low + 3))
                if name == "CARGO_MANIFEST_DIR" and not pieces and not manifest:
                    manifest = True
                    continue
                if name == "OUT_DIR":
                    return ("out", None)
            return None
        text = "".join(pieces)
        if manifest:
            return ("path", posixpath.normpath(posixpath.join(self.crate_directory or ".", text.lstrip("/"))))
        return ("relative", text)

    def include_reference(self, unit, macro_index, low, high):
        parts = self.top_level_parts(low, high)
        path = None
        if len(parts) == 1:
            part_low, part_high = parts[0]
            if part_high == part_low + 1 and self.kind(part_low) == LITERAL:
                value = rust_string_value(self.t(part_low))
                if value is not None:
                    path = posixpath.normpath(posixpath.join(self.directory, value))
            elif self.t(part_low) == "concat" and self.t(part_low + 1) == "!" and self.t(part_low + 2) == "(" and (part_low + 2) in self.pairs:
                folded = self.concat_value(part_low + 3, self.pairs[part_low + 2])
                if folded is not None and folded[0] == "out":
                    unit.data_references.append(("out-directory", None, macro_index))
                    return
                if folded is not None and folded[0] == "path":
                    path = folded[1]
                elif folded is not None:
                    path = posixpath.normpath(posixpath.join(self.directory, folded[1]))
        if path is None:
            self.refuse(unit, "include-argument", macro_index)
            return
        # 根外的文件（include_str!("../../.claude/heavy-thread-default") 从根下一层的工作区读仓根下的登记文件）照 crates/singlefs-unit-graph 收：
        #   读得到就是数据单元（键只取字节摘要），路径写成绝对路径（unit-graph 的 display_path 对根外的路径同样不去前缀），不当 .wgsl 模块拼；
        #   读不到才拒算 include-missing。弄坏开关 rust-includes-outside-root-refused 下根外一律拒算
        outside = path == ".." or path.startswith("../")
        if outside and "rust-includes-outside-root-refused" not in active_breaks():
            path = os.path.normpath(os.path.join(os.path.abspath(self.root), path))
            if not os.path.isfile(path):
                self.refuse(unit, "include-missing", macro_index)
                return
            unit.data_references.append(("file", path, macro_index))
            return
        if outside or not os.path.isfile(os.path.join(self.root, path)):
            self.refuse(unit, "include-missing", macro_index)
            return
        unit.data_references.append(("wgsl" if path.endswith(".wgsl") else "file", path, macro_index))

    def concatenated_includes(self, unit, low, high):
        """concat!(…) 参数里的每一处 include_str! / include_bytes!（套在几层宏里的也算）各照 include_reference 收一处读，include! 照拒算 include-code
        （crates/singlefs-unit-graph 的 reads_and_refusals_of 同：concat! 里的词照样逐个看，一个宏调用里的几个 include_str! 各切一个数据单元）
        → 这几处宏各占的 (宏名那个词, 闭括号) 区间（runtime_concat 不在里面找 env!：读的路径归那一处 include_*!）。
        弄坏开关 rust-concatenated-includes-skipped 下不看，concat! 整段跳过。"""
        spans, j = [], low
        while j < high:
            if self.kind(j) == IDENT and self.t(j) in ("include_str", "include_bytes", "include") and self.t(j + 1) == "!" \
                    and self.t(j + 2) in ("(", "[", "{") and (j + 2) in self.pairs:
                close = self.pairs[j + 2]
                if self.t(j) == "include":
                    self.refuse(unit, "include-code", j)
                else:
                    self.include_reference(unit, j, j + 3, close)
                spans.append((j, close))
                j = close + 1
                continue
            j += 1
        return spans

    def runtime_concat(self, unit, macro_index, low, high, nested=()):
        folded = self.concat_value(low, high)
        manifest_reads = [j for j in range(low, high) if self.t(j) == "env" and rust_string_value(self.t(j + 3)) == "CARGO_MANIFEST_DIR"
                          and not any(start <= j <= close for start, close in nested)]
        if not manifest_reads:
            return
        # 读的那一处落在 env! 那个词上（与 crates/singlefs-unit-graph 记的行相同：concat!( 与 env! 分两行写时取 env! 那一行）
        if folded is not None and folded[0] == "path" and os.path.isfile(os.path.join(self.root, folded[1])):
            unit.data_references.append(("file", folded[1], manifest_reads[0]))
            return
        self.refuse(unit, "runtime-path", manifest_reads[0])

    def runtime_manifest_path(self, unit, environment_index, environment_close):
        """env!("CARGO_MANIFEST_DIR")（或指到它的私有常量，下标取常量名那一个词）所在的一处（crates/singlefs-unit-graph 的 fold_manifest_directory_read）：
        Path::new(…) / PathBuf::from(…) 里再接 .join("字面") 链的，交 fold_manifest_chain 折（文件头「数据目录」）：折出文件的收成数据文件（文件的字节进键），
        crates/ 之下（不含 crates/ 整个）的目录收成数据目录，绑给 let 变量的目录按这个变量每一处用法各折一样；
        折出来的每一样都记在这一处：读记一条（同一样只记一次），折不出的那一样拒算 runtime-path、别的照收。别的写法拒算 runtime-path。"""
        opening = environment_index - 1
        if self.t(opening) == "(" and self.pairs.get(opening) == environment_close + 1 and self.t(opening - 1) in ("new", "from"):
            folds = self.fold_manifest_chain(unit, self.crate_directory or ".", environment_close + 2, self.manifest_path_expression_start(opening - 1), 0)
            recorded = []
            for fold in folds:
                if fold == MANIFEST_FOLD_REFUSED:
                    self.refuse(unit, "runtime-path", environment_index)
                elif fold not in recorded:
                    recorded.append(fold)
            for kind, path in recorded:
                unit.data_references.append((kind, path, environment_index))
            return
        if environment_index in self.manifest_constants()[1]:
            return   # const <名>: &str = env!("CARGO_MANIFEST_DIR");：常量本身不读什么，用到它的每一处各自折（manifest_constant_use）
        self.refuse(unit, "runtime-path", environment_index)

    def fold_manifest_chain(self, unit, base, chain_start, expression_start, depth):
        """从 chain_start 起接字面 .join("…") 链拼在 base 后面（crates/singlefs-unit-graph 的 fold_path_chain）→ [("file" 或 "directory", 路径) 或 MANIFEST_FOLD_REFUSED]。
        链后还接任何方法（.parent()、.join(<不是字面>) 这一类）、折出根外或盘上没有的路径：MANIFEST_FOLD_REFUSED。折出文件：那份文件。
        折出目录、表达式 [expression_start, 链尾) 又整个是 let <名> [: 类型] = …; 的右边（不论是不是能收的目录）：<名> 每一处用法各折的（fold_manifest_binding_use；
        后面没用到的照目录本身判），已经跟了 DEEPEST_MANIFEST_BINDING_CHAIN 层的拒算；不绑 let 的目录：在 crates/ 之下的（is_data_directory）收成目录，别的拒算。
        depth 是已经跟过的 let 层数（env! 那一处是 0）。
        弄坏开关 rust-manifest-binding-refused-whole：不是能收的目录不跟 let、拒算；有一处用法折不出就整处只交一个 MANIFEST_FOLD_REFUSED。"""
        path, c = base, chain_start
        while self.t(c) == "." and self.t(c + 1) == "join" and self.t(c + 2) == "(" and self.kind(c + 3) == LITERAL and self.t(c + 4) == ")":
            value = rust_string_value(self.t(c + 3))
            if value is None:
                break
            path = posixpath.normpath(posixpath.join(path, value))
            c += 5
        if self.t(c) == "." or path == ".." or path.startswith("../"):
            return [MANIFEST_FOLD_REFUSED]
        if os.path.isfile(os.path.join(self.root, path)):
            return [("file", path)]
        if not os.path.isdir(os.path.join(self.root, path)):
            return [MANIFEST_FOLD_REFUSED]
        # 弄坏开关：env! 那一处拼出的是目录也照放行（目录底下读什么键里都看不见）；平常目录一律拒算，要仓根的点名清单目录里的一份文件再往上取
        if depth == 0 and "rust-manifest-directory-reads-unrefused" in active_breaks():
            return []
        directory_itself = ("directory", path) if self.is_data_directory(path) else MANIFEST_FOLD_REFUSED
        variable = self.manifest_binding_name(expression_start, c)
        refused_whole = "rust-manifest-binding-refused-whole" in self.breaks
        if variable is None or (refused_whole and directory_itself == MANIFEST_FOLD_REFUSED):
            return [directory_itself]
        if depth >= DEEPEST_MANIFEST_BINDING_CHAIN:
            return [MANIFEST_FOLD_REFUSED]
        uses = self.manifest_binding_uses(variable, c + 1, unit.last_token)
        if not uses:
            return [directory_itself]
        folds = [fold for use_index in uses for fold in self.fold_manifest_binding_use(unit, path, use_index, depth + 1)]
        if refused_whole and MANIFEST_FOLD_REFUSED in folds:
            return [MANIFEST_FOLD_REFUSED]
        return folds

    def is_data_directory(self, path):
        """折出的路径是不是能收成数据目录的目录：仓内、真是目录、在 crates/ 之下、不是 crates/ 整个。"""
        if "rust-manifest-directories-refused" in self.breaks:
            return False
        return path.startswith("crates/") and not path.startswith("../") and os.path.isdir(os.path.join(self.root, path))

    def manifest_path_expression_start(self, constructor_index):
        """std::path::Path::new 这类写法起头的那个词法单元：从 new / from 往回跨过 <名字> ::（crates/singlefs-unit-graph 的 path_expression_start）。"""
        start = constructor_index
        while self.t(start - 1) == ":" and self.t(start - 2) == ":" and self.kind(start - 3) == IDENT:
            start -= 3
        return start

    def manifest_binding_name(self, expression_start, after_expression):
        """[expression_start, after_expression) 整个是 let [mut] <名> [: 类型] = …; 的右边时交 <名>，别的交 None（crates/singlefs-unit-graph 的 let_binding_name）。"""
        if self.t(after_expression) != ";" or self.t(expression_start - 1) != "=":
            return None
        equals = expression_start - 1
        let_index = next((position for position in range(equals - 1, -1, -1) if self.texts[position] in ("let", ";", "{", "}", "=")), None)
        if let_index is None or self.texts[let_index] != "let":
            return None
        name_index = let_index + 1
        if self.t(name_index) == "mut":
            name_index += 1
        if self.kind(name_index) != IDENT:
            return None
        plain = name_index + 1 == equals
        typed = self.t(name_index + 1) == ":" and self.t(name_index + 2) != ":"
        return self.t(name_index) if plain or typed else None

    def manifest_binding_uses(self, variable, low, high):
        """<变量> 在 [low, high) 里的每一处用法的下标（crates/singlefs-unit-graph 的 uses_of_the_binding）：.<变量> 的字段或方法、<变量>:: 的路径段不算；
        碰到再 let 这个名字就停（之后是另一个值）。"""
        uses = []
        for j in range(low, high):
            if self.kind(j) != IDENT or self.t(j) != variable:
                continue
            previous = self.t(j - 1)
            if previous == "let" or (previous == "mut" and self.t(j - 2) == "let"):
                break
            if previous == "." or (self.t(j + 1) == ":" and self.t(j + 2) == ":"):
                continue
            uses.append(j)
        return uses

    def fold_manifest_binding_use(self, unit, value, use_index, depth):
        """绑着路径 value 的变量的一处用法（crates/singlefs-unit-graph 的 fold_use_of_a_path）：Path::new(<变量>) / PathBuf::from(<变量>) 之后、
        或变量本身之后接的字面 .join 链照 fold_manifest_chain 折；变量单独交给别的函数的，折出的就是它绑着的目录。"""
        if self.t(use_index - 1) == "(" and self.t(use_index - 2) in ("new", "from") and self.t(use_index + 1) == ")":
            return self.fold_manifest_chain(unit, value, use_index + 2, self.manifest_path_expression_start(use_index - 2), depth)
        return self.fold_manifest_chain(unit, value, use_index + 1, use_index, depth)

    def manifest_constants(self):
        """→ ({私有常量名}, {它初始值里那个 env 词的下标})：const <名>: <类型> = env!("CARGO_MANIFEST_DIR"); 初始值整个就是那一个宏、没写 pub 的。
        公开的常量别的文件也能用到，折不全，照拒算。"""
        if self._manifest_constants is None:
            sites = {}   # env 词的下标 → 常量名
            for i in range(self.count):
                if not (self.texts[i] == "env" and self.t(i + 1) == "!" and self.t(i + 2) == "(" and self.kind(i + 3) == LITERAL
                        and rust_string_value(self.t(i + 3)) == "CARGO_MANIFEST_DIR" and self.t(i + 4) == ")" and self.t(i - 1) == "=" and self.t(i + 5) == ";"):
                    continue
                j = i - 2
                while j >= 0 and self.texts[j] not in (";", "{", "}", "]") and self.texts[j] != "const":
                    j -= 1
                if self.t(j) != "const" or self.kind(j + 1) != IDENT:
                    continue
                before = 0
                while before < 6 and j - 1 - before >= 0 and self.texts[j - 1 - before] not in (";", "{", "}", "]"):
                    before += 1
                if any(self.texts[k] == "pub" for k in range(j - before, j)):
                    continue
                sites[i] = self.texts[j + 1]
            # 串字面量里有 {<名> 的（format! 一族的内联捕获）：名字不经 Path::new 就用到了，照拒算
            captured = {name for name in sites.values() if any(self.kinds[k] == LITERAL and "{" + name in self.texts[k] for k in range(self.count))}
            sites = {i: name for i, name in sites.items() if name not in captured}
            self._manifest_constants = (set(sites.values()), set(sites))
        return self._manifest_constants

    def manifest_constant_use(self, unit, index):
        """私有常量（manifest_constants）出现在 Path::new(<常量>) / PathBuf::from(<常量>) 里的，照 env!("CARGO_MANIFEST_DIR") 同一处折；别的写法拒算 runtime-path。"""
        opening = index - 1
        if self.t(opening) == "(" and self.pairs.get(opening) == index + 1 and self.t(opening - 1) in ("new", "from"):
            self.runtime_manifest_path(unit, index, index)
        else:
            self.refuse(unit, "runtime-path", index)

    # 文档测试（照 crates/singlefs-unit-graph 的 push_documentation_tests 与 rust_code_blocks）

    def extract_doctests(self):
        """条目的外文档注释（下一个词法单元落在 [条目首词法单元, header] 里的）与模块开头的内文档注释（文件开头、行内模块体开头到内属性收尾）
        各自拼起来找 Rust 代码块，一个代码块一个文档测试单元。字段、枚举变体、函数体里的条目与行内模块头上的外文档注释不挂（Rust 那一份也不挂）。"""
        if not self.documents:
            return
        outer = [(next_index, line, raw) for line, style, _rows, next_index, raw in self.documents if style == "outer"]
        inner = [(next_index, line, raw) for line, style, _rows, next_index, raw in self.documents if style == "inner"]
        for unit in list(self.units):
            if unit.kind == "mod_head" or unit.first_token is None:
                continue
            comments = [(line, raw) for next_index, line, raw in outer if unit.first_token <= next_index <= unit.header]
            for code, opening, closing in rust_code_blocks(comments):
                self.add_doctest(code, opening, closing, unit, None)
        for low, high, module_path in self.module_documents:
            comments = [(line, raw) for next_index, line, raw in inner if low <= next_index <= high]
            for code, opening, closing in rust_code_blocks(comments):
                self.add_doctest(code, opening, closing, None, module_path)

    def add_doctest(self, code, open_line, close_line, owner, module_path):
        """限定名：挂条目的 <条目>::<doctest line 行>，挂模块的 <模块路径>::<doctest line 行>（文件开头的就是 <doctest line 行>）。
        指纹：代码块（去掉隐藏行记号）重新切词、去注释，词法单元以 0x1f 连的 sha256；弄坏开关 rust-doctest-fingerprint-from-rows 下取代码块原文各行以 \\n 连。"""
        tail = f"<doctest line {open_line}>"
        name = f"{owner.name}::{tail}" if owner is not None else "::".join(list(module_path) + [tail])
        kinds, texts, _lines, _documents = rust_lex(code)
        if "rust-doctest-fingerprint-from-rows" in self.breaks:
            fingerprint = sha256_text(code)
        else:
            fingerprint = sha256_text("\x1f".join(texts))
        unit = Unit("rust", "doctest", name, self.path, open_line, close_line, fingerprint)
        unit.plain, unit.methods, unit.qualified, unit.dyn_traits = token_references(kinds, texts, 0, len(texts))
        unit.references = unit.plain | unit.methods
        code_pairs, _bad = match_brackets(kinds, texts)
        unit.cfg_leaves = cfg_predicates_in_tokens(kinds, texts, [open_line + line for line in _lines], code_pairs or {}, 0, len(texts))
        unit.container = owner
        unit.conditions = list(owner.conditions) if owner is not None else []
        unit.scope = owner.scope if owner is not None else tuple(module_path)
        unit.doc_module = None if owner is not None else tuple(module_path)
        unit.public, unit.data_references = False, []
        unit.self_types, unit.implicit_keys = set(), set()
        unit.first_token = unit.last_token = None
        self.doctests.append(unit)


RUST_CODE_BLOCK_WORDS = frozenset(("rust", "ignore", "should_panic", "no_run", "compile_fail", "test_harness", "allow_fail", "standalone_crate", "custom"))


def is_rust_code_block_information(information):
    """代码块的信息串是不是 rustdoc 当 Rust 文档测试编的那一种（空的也是）；照 crates/singlefs-unit-graph 的同名函数。"""
    for word in re.split(r"[, \t]", information):
        if not word:
            continue
        if word in RUST_CODE_BLOCK_WORDS or word.startswith("edition") or word.startswith("ignore-"):
            continue
        if len(word) > 1 and word[0] == "E" and all("0" <= character <= "9" for character in word[1:]):
            continue
        return False
    return True


def code_fence_of(trimmed):
    """行首围栏：三个以上的 ` 或 ~ → (围栏字符, 个数)，否则 None。"""
    if not trimmed or trimmed[0] not in "`~":
        return None
    count = len(trimmed) - len(trimmed.lstrip(trimmed[0]))
    return (trimmed[0], count) if count >= 3 else None


def without_hidden_line_marker(line):
    """rustdoc 的隐藏行：# 代码 照编；单独一个 # 是空行；## 开头的是一个字面的 #。"""
    trimmed = line.lstrip()
    if trimmed == "#":
        return ""
    if trimmed.startswith("# "):
        return trimmed[2:]
    if trimmed.startswith("##"):
        return trimmed[1:]
    return line


def rust_code_blocks(comments):
    """[(注释起行, [各行原文])] 拼起来找代码块 → [(代码, 开围栏那一行, 闭围栏那一行)]，只留 Rust 代码块；没闭上的到最后一行。"""
    blocks, opened = [], None
    for first_line, rows in comments:
        for offset, raw in enumerate(rows):
            line = first_line + offset
            trimmed = raw.lstrip()
            fence = code_fence_of(trimmed)
            if opened is None:
                if fence is not None:
                    opened = (fence[0], fence[1], line, [], is_rust_code_block_information(trimmed[fence[1]:].strip()))
                continue
            if fence is not None and fence[0] == opened[0] and fence[1] >= opened[1] and not trimmed[fence[1]:].strip():
                if opened[4]:
                    blocks.append(("\n".join(opened[3]), opened[2], line))
                opened = None
            else:
                opened[3].append(without_hidden_line_marker(raw))
    if opened is not None and opened[4]:
        blocks.append(("\n".join(opened[3]), opened[2], opened[2] + len(opened[3])))
    return blocks
