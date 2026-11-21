"""rustfmt 的排版会改词法单元序列的两处，纯挪动检查（crates-split-pure-move.py）把只差这两处的单元单列成「只差 rustfmt 排版」：
  一、紧挨 )、]、}、> 之前的尾逗号（折行时加上、并成一行时去掉）；
  二、闭包体与 match 臂体外面只包一个表达式的那一对花括号（|x| { e } 与 |x| e、=> { e } 与 => e,）。
normalized(词法单元表) 把这两处抹平后交回新的词法单元表；两个单元抹平之后相同，就只差这两处。
词法单元一律是单个标点字符（=> 是 = 与 >，:: 是两个 :，|| 是两个 |），与 lib_function_units.py 的 rust_lex 同一种切法。
不起进程，不读文件；弄坏开关只在那份入口的 --selftest 里认（common.broken）。
"""
import re

import lib_crates_split_common as common

broken = common.broken

OPENERS = ("(", "[", "{")
CLOSER_OF = {"(": ")", "[": "]", "{": "}"}
CLOSERS_AFTER_TRAILING_COMMA = (")", "]", "}", ">")
IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\Z")
# 后面紧跟 ( 的是元组（表达式、类型、模式）而不是调用的关键字：单元素元组 (x,) 与带括号的 (x) 不是同一个东西，它们的 ( 里那个逗号不是尾逗号
KEYWORDS_BEFORE_A_TUPLE = frozenset((
    "as", "async", "await", "box", "break", "const", "continue", "do", "dyn", "else", "enum", "extern", "for", "if", "impl", "in", "let",
    "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "static", "struct", "trait", "type", "union", "unsafe", "use", "where", "while", "yield"))


def matching_brackets(tokens):
    """() [] {} 配对：→ {开的下标: 闭的下标}（只记开的一边）；括号配不平交 None。"""
    pairs, stack = {}, []
    for index, token in enumerate(tokens):
        if token in CLOSER_OF:
            stack.append(index)
        elif token in (")", "]", "}"):
            if not stack or CLOSER_OF[tokens[stack[-1]]] != token:
                return None
            pairs[stack.pop()] = index
    return None if stack else pairs


def has_top_level_semicolon(tokens, first, last):
    """tokens[first:last] 里不在括号里面的位置有没有分号（有就是语句块，不是单个表达式）。"""
    depth = 0
    for token in tokens[first:last]:
        if token in CLOSER_OF:
            depth += 1
        elif token in (")", "]", "}"):
            depth -= 1
        elif token == ";" and depth == 0:
            return True
    return False


def droppable_braces(tokens, pairs):
    """闭包体（{ 前面是 |）与 match 臂体（{ 前面是 = >）外面只包一个表达式的 { }：→ {开的下标: 是不是 match 臂}。
    只包单个表达式 = 花括号里不空、没有不在括号里面的分号；花括号里有分号的是语句块，不动。"""
    found = {}
    for open_index, close_index in pairs.items():
        if tokens[open_index] != "{" or close_index == open_index + 1:
            continue
        arm = open_index >= 2 and tokens[open_index - 2] == "=" and tokens[open_index - 1] == ">"
        closure = open_index >= 1 and tokens[open_index - 1] == "|"
        if not (arm or closure):
            continue
        if has_top_level_semicolon(tokens, open_index + 1, close_index) and "statement-braces-dropped" not in broken:
            continue
        found[open_index] = arm
    return found


def without_single_expression_braces(tokens):
    """去掉 droppable_braces 点到的花括号；match 臂体去掉花括号之后臂尾没有逗号的补一个（rustfmt 写成 => e,，尾逗号之后再统一抹）。"""
    pairs = matching_brackets(tokens)
    if pairs is None or "single-expression-braces-kept" in broken:
        return list(tokens)
    dropped = droppable_braces(tokens, pairs)
    closing = {pairs[open_index]: arm for open_index, arm in dropped.items()}
    result = []
    for index, token in enumerate(tokens):
        if index in dropped:
            continue
        if index in closing:
            comma_follows = index + 1 < len(tokens) and tokens[index + 1] == ","
            if closing[index] and not comma_follows and "arm-comma-not-added" not in broken:
                result.append(",")
            continue
        result.append(token)
    return result


def generic_opener(tokens, index):
    """tokens[index] 是 < 时，它是泛型参数的开头吗（前面是 :: 或大写开头的标识符；别的当比较运算符）。"""
    if index == 0:
        return False
    previous = tokens[index - 1]
    return previous == ":" or (IDENTIFIER.match(previous) is not None and previous[0].isupper())


def has_other_top_level_comma(tokens, open_index, comma_index):
    """( 到那个逗号之间，括号与泛型尖括号外面还有没有别的逗号。"""
    depth, angle = 0, 0
    for index in range(open_index + 1, comma_index):
        token = tokens[index]
        if token in CLOSER_OF:
            depth += 1
        elif token in (")", "]", "}"):
            depth -= 1
        elif depth == 0:
            if token == "<" and generic_opener(tokens, index):
                angle += 1
            elif token == ">" and angle > 0 and tokens[index - 1] not in ("-", "="):
                angle -= 1
            elif token == "," and angle == 0:
                return True
    return False


def is_list_trailing_comma(tokens, opened_by, comma_index):
    """这个逗号是不是一串里最后一个、紧挨着收尾符（)、]、}、>）的尾逗号，去掉它不改意思。
    ( 里只有它一个逗号、( 前面又不是调用（标识符、!、)、]）时，它是单元素元组 (x,) 的逗号，不算。"""
    if comma_index == 0 or comma_index + 1 >= len(tokens):
        return False
    closer, previous = tokens[comma_index + 1], tokens[comma_index - 1]
    if closer not in CLOSERS_AFTER_TRAILING_COMMA or previous in OPENERS or previous == ",":
        return False
    if closer != ")" or "single-element-tuple-comma-stripped" in broken:
        return True
    open_index = opened_by.get(comma_index + 1)
    if open_index is None:
        return False
    return is_call_parenthesis(tokens, open_index) or has_other_top_level_comma(tokens, open_index, comma_index)


def is_call_parenthesis(tokens, open_index):
    """( 是调用（或函数的形参表）的括号吗：前面是标识符（不是元组前面的关键字）、!（宏）、)、]，或泛型参数的 >（不是 -> 与 => 的 >）。"""
    before_group = tokens[open_index - 1] if open_index else ""
    if before_group == ">":
        return open_index >= 2 and tokens[open_index - 2] not in ("-", "=")
    return before_group in ("!", ")", "]") or (IDENTIFIER.match(before_group) is not None and before_group not in KEYWORDS_BEFORE_A_TUPLE)


def without_trailing_commas(tokens):
    pairs = matching_brackets(tokens)
    if pairs is None or "trailing-comma-kept" in broken:
        return list(tokens)
    opened_by = {close: open_index for open_index, close in pairs.items()}
    return [token for index, token in enumerate(tokens) if not (token == "," and is_list_trailing_comma(tokens, opened_by, index))]


def normalized(tokens):
    """排版抹平之后的词法单元表：先去只包单个表达式的花括号（match 臂补上逗号），再去尾逗号；括号配不平的原样交回。"""
    return without_trailing_commas(without_single_expression_braces(tokens))
