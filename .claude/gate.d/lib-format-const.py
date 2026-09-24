#!/usr/bin/env python3
"""format-const 标记与 Rust const 声明的解析：门禁 27、39、92 号共用这一份，不各抄一份。

三道各写一份解析器时口径分叉过：27 号要求标记整条按文法收尾，标记里多写一个键它就当这条不存在、
静默不登记，而 39、92 号只匹配前缀照认——同一行标记，一道看不见、两道照用；27 号同一份文件里
同名标记写两次不报，第二个值静默丢掉。所以这里只有一条文法，读不出来的与重复的都报给调用方判红，
不许跳过。

标记的文法：

    <!-- format-const: 名字 = 整数 stale=旧串|旧串 -->

`stale=` 可省，别的键一律不认。以 `<!-- format-const` 开头而按这条文法读不出来的，进 unparsable。

Rust const：一行以 `[pub[(…)]] const 名字: 类型 = 值;` 起头，值可以跨行，读到第一个分号为止。
值怎么读由调用方显式选（value_reading）：
  integer_literal —— 整数字面量（可带 `_` 分隔与整型后缀），读不出来的（`16 * 1024`）值是 None；
  normalized_text —— 按空白归一后的原文，只当变更探测器用，不问它等于几。
"""
import re
from dataclasses import dataclass

MARK_OPENING = re.compile(r"<!--\s*format-const\b")
MARK_GRAMMAR = re.compile(
    r"<!--\s*format-const:\s*(?P<name>\w+)\s*=\s*(?P<value>-?\d+)\s*(?:stale=(?P<stale>[^>]*?))?\s*-->")

# 类型那一格允许方括号里的分号（`[u32; 3]`），不许方括号外的分号：没有默认值的关联常量
# `const X: u64;` 若让类型一路吃过分号，会把下一段代码里的第一个 `=` 当成它的值。
RUST_CONST_DECLARATION = re.compile(
    r"^(?P<indentation>[ \t]*)(?P<visibility>pub(?:\([^)]*\))?\s+)?const\s+(?P<name>\w+)\s*:\s*"
    r"(?P<type>(?:[^=;\[\]]|\[[^\]]*\])+?)\s*=\s*(?P<value>[^;]+?)\s*;",
    re.M)
INTEGER_LITERAL = re.compile(r"(?P<digits>-?[0-9][0-9_]*)(?:[iu](?:8|16|32|64|128|size))?")
VALUE_READINGS = ("integer_literal", "normalized_text")


@dataclass(frozen=True)
class FormatConstMark:
    name: str
    value: int
    value_text: str
    stale_literals: tuple
    line_number: int


@dataclass(frozen=True)
class UnparsableMark:
    line_number: int
    excerpt: str


@dataclass(frozen=True)
class DuplicateMark:
    name: str
    line_numbers: tuple
    values: tuple


@dataclass(frozen=True)
class ParsedMarks:
    marks: tuple
    unparsable: tuple
    duplicates: tuple


@dataclass(frozen=True)
class RustConst:
    name: str
    type_text: str
    value: object
    value_text: str
    line_number: int


def line_number_at(text, offset):
    return text.count("\n", 0, offset) + 1


def parse_marks(text):
    """按出现次序返回这段文本里的标记、读不出来的标记、同名登记了不止一次的名字。"""
    marks, unparsable = [], []
    for opening in MARK_OPENING.finditer(text):
        line_number = line_number_at(text, opening.start())
        match = MARK_GRAMMAR.match(text, opening.start())
        if not match:
            closing = text.find("-->", opening.start())
            end = closing + len("-->") if closing >= 0 else len(text)
            excerpt = text[opening.start():end].split("\n")[0][:160]
            unparsable.append(UnparsableMark(line_number, excerpt))
            continue
        stale_literals = tuple(piece for piece in (match.group("stale") or "").split("|") if piece.strip())
        marks.append(FormatConstMark(match.group("name"), int(match.group("value")), match.group("value"),
                                     stale_literals, line_number))
    places_by_name = {}
    for mark in marks:
        places_by_name.setdefault(mark.name, []).append(mark)
    duplicates = tuple(
        DuplicateMark(name, tuple(mark.line_number for mark in same_name), tuple(mark.value for mark in same_name))
        for name, same_name in places_by_name.items() if len(same_name) > 1)
    return ParsedMarks(tuple(marks), tuple(unparsable), duplicates)


def strip_marks(text):
    """去掉按文法读得出来的标记，留下正文：标记自己的 stale= 串不能被当成正文里残留的旧值。"""
    return MARK_GRAMMAR.sub("", text)


def integer_literal_value(value_text):
    literal = INTEGER_LITERAL.fullmatch(value_text.strip())
    if not literal:
        return None
    return int(literal.group("digits").replace("_", ""))


def normalized_value_text(value_text):
    return re.sub(r"\s+", " ", value_text).strip()


def read_rust_consts(text, value_reading="integer_literal", only_top_level_public=False, only_scalar_types=False):
    """按出现次序返回 const 声明。

    only_top_level_public 只留顶格、可见性恰是 `pub` 的那些；only_scalar_types 只留类型是单个标识符
    （`u64`、`usize`）的那些——实验源码里同名的扫描表（`const NODE_BYTES: [u64; 2] = [4096, 16384];`）
    不是那个格式常量。
    """
    if value_reading not in VALUE_READINGS:
        raise ValueError(f"value_reading 只能是 {VALUE_READINGS} 之一，拿到的是 {value_reading!r}")
    declarations = []
    for match in RUST_CONST_DECLARATION.finditer(text):
        if only_top_level_public and (match.group("indentation") or (match.group("visibility") or "").strip() != "pub"):
            continue
        if only_scalar_types and not re.fullmatch(r"\w+", match.group("type")):
            continue
        value_text = match.group("value")
        if value_reading == "integer_literal":
            value = integer_literal_value(value_text)
        else:
            value = normalized_value_text(value_text)
        declarations.append(RustConst(match.group("name"), match.group("type"), value, value_text, line_number_at(text, match.start("name"))))
    return declarations
