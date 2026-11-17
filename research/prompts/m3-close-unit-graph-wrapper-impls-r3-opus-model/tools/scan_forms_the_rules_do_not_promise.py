#!/usr/bin/env python3
"""三方攻方腿（m3-close-unit-graph-wrapper-impls-r3）第二条「未定项清单的完整性」的真仓扫描：单元按 `unit-graph dump` 的切分取
（每行 `<种类>\t<限定名>\t<文件>:<起>-<止>\t<指纹>`），每个单元的词法单元从它那几行的源码现切（去注释、串与字符字面量照整个算一个词），
逐类数「这套规则不保证收到」的写法在 crates/ 下今天有几处。只读，不起子进程。

用法：scan_forms_the_rules_do_not_promise.py --dump <unit-graph dump 的输出文件> --root <仓根>
输出：每一类一行 `class=<类名> instances=<处数> …`，接着这一类每一处 `instance class=<类名> at=<文件>:<行> unit=<种类> <限定名> detail=<…>`；
      末行 `name=scan rust_units=<n> files=<n> classes=<n> registered_entries=<n> entries_found=<n>`。
退出码：0 扫完；2 用法错、dump 读不了或一行 Rust 单元都没有、登记表读不了。"""
import argparse
import bisect
import collections
import os
import re
import sys

SKIPPED_BEFORE_THE_SELF_TYPE = frozenset(("dyn", "mut", "const", "impl", "for", "where", "as", "unsafe"))
AUTO_TRAITS = frozenset(("Send", "Sync", "Unpin", "UnwindSafe", "RefUnwindSafe"))
CALLABLE_TRAITS = frozenset(("Fn", "FnMut", "FnOnce"))
DOWNCAST_METHODS = frozenset(("downcast", "downcast_ref", "downcast_mut", "downcast_unchecked", "downcast_ref_unchecked", "downcast_mut_unchecked"))
TYPE_KINDS = frozenset(("struct", "enum", "union", "type"))
NOMINAL_KINDS = frozenset(("struct", "enum", "union"))
REGISTRY = "crates/singlefs-verification/src/judging_units_entries.rs"
# 只给这几种单元切词（模块头一类的单元盖住整个模块，切了只是把同一批词再引一遍）
KINDS_READ = frozenset(("fn", "impl", "trait", "const", "static", "struct", "enum", "union", "type", "macro_rules", "use", "item_macro"))
RAW_STRING_START = re.compile(r'(?:b|c)?r(#*)"')
BREADTH_LIMIT = 20000   # 沿类型名往下找回调时最多看几个类型单元（同名的都看，按名字走、不按模块筛）


def strip_raw(name):
    return name[2:] if name.startswith("r#") else name


def is_identifier_start(character):
    return character.isalpha() or character == "_" or ord(character) > 127


def is_identifier_part(character):
    return character.isalnum() or character == "_" or ord(character) > 127


def lex(text):
    """→ [(种类, 原文, 行号)]，种类是 ident、lifetime、literal、punct；注释丢掉；`r#名` 是一个 ident（原文带着 r#）。"""
    tokens, position, count, line = [], 0, len(text), 1

    def skip_quoted(start, quote):
        """从 start（引号之后）扫到配对的 quote，交（结束位置, 跨过的换行数）；认反斜杠转义。"""
        cursor, newlines = start, 0
        while cursor < count and text[cursor] != quote:
            if text[cursor] == "\\":
                cursor += 1
            if cursor < count and text[cursor] == "\n":
                newlines += 1
            cursor += 1
        return min(cursor + 1, count), newlines

    while position < count:
        character = text[position]
        if character == "\n":
            line += 1
            position += 1
            continue
        if character.isspace():
            position += 1
            continue
        if text.startswith("//", position):
            end = text.find("\n", position)
            position = count if end < 0 else end
            continue
        if text.startswith("/*", position):
            depth, cursor = 1, position + 2
            while cursor < count and depth:
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                else:
                    line += text[cursor] == "\n"
                    cursor += 1
            position = cursor
            continue
        raw_string = RAW_STRING_START.match(text, position) if character in "bcr" else None
        if raw_string:
            closing = '"' + raw_string.group(1)
            end = text.find(closing, raw_string.end())
            end = count if end < 0 else end + len(closing)
            tokens.append(("literal", text[position:end], line))
            line += text.count("\n", position, end)
            position = end
            continue
        if text.startswith("r#", position) and position + 2 < count and is_identifier_start(text[position + 2]):
            cursor = position + 2
            while cursor < count and is_identifier_part(text[cursor]):
                cursor += 1
            tokens.append(("ident", text[position:cursor], line))
            position = cursor
            continue
        if character in "bc" and position + 1 < count and text[position + 1] == '"':
            end, newlines = skip_quoted(position + 2, '"')
            tokens.append(("literal", text[position:end], line))
            line += newlines
            position = end
            continue
        if character == "b" and position + 1 < count and text[position + 1] == "'":
            end, newlines = skip_quoted(position + 2, "'")
            tokens.append(("literal", text[position:end], line))
            line += newlines
            position = end
            continue
        if character == '"':
            end, newlines = skip_quoted(position + 1, '"')
            tokens.append(("literal", text[position:end], line))
            line += newlines
            position = end
            continue
        if character == "'":
            if position + 1 < count and text[position + 1] == "\\":
                end, newlines = skip_quoted(position + 1, "'")
                tokens.append(("literal", text[position:end], line))
                line += newlines
                position = end
                continue
            if position + 2 < count and text[position + 2] == "'":
                tokens.append(("literal", text[position:position + 3], line))
                position += 3
                continue
            cursor = position + 1
            while cursor < count and is_identifier_part(text[cursor]):
                cursor += 1
            tokens.append(("lifetime", text[position:cursor], line))
            position = max(cursor, position + 1)
            continue
        if is_identifier_start(character):
            cursor = position + 1
            while cursor < count and is_identifier_part(text[cursor]):
                cursor += 1
            tokens.append(("ident", text[position:cursor], line))
            position = cursor
            continue
        if character.isdigit():
            cursor = position + 1
            while cursor < count and (is_identifier_part(text[cursor]) or (
                    text[cursor] == "." and cursor + 1 < count and text[cursor + 1].isdigit())):
                cursor += 1
            tokens.append(("literal", text[position:cursor], line))
            position = cursor
            continue
        tokens.append(("punct", character, line))
        position += 1
    return tokens


class Unit:
    def __init__(self, kind, qualified, file, start, end):
        self.kind, self.qualified, self.file, self.start, self.end = kind, qualified, file, start, end
        self.tokens = []
        segments = split_qualified(qualified)
        cut = next((index for index, segment in enumerate(segments) if segment.startswith(("<impl", "<trait"))), None)
        self.module = (file, tuple(segments[:cut] if cut is not None else segments[:-1]))
        self.name = strip_raw(segments[-1]) if segments else ""

    def idents(self):
        return [strip_raw(text) for kind, text, _ in self.tokens if kind == "ident"]

    def where(self):
        return f"{self.file}:{self.start} unit={self.kind} {self.qualified}"


def split_qualified(name):
    """限定名按尖括号外的 `::` 切段（impl 标签 `<impl fmt::Display for X>` 里的 `::` 不切）。"""
    segments, depth, current, position = [], 0, [], 0
    while position < len(name):
        character = name[position]
        if character == "<":
            depth += 1
        elif character == ">" and not (position > 0 and name[position - 1] == "-"):
            depth -= 1
        if depth == 0 and name.startswith("::", position):
            segments.append("".join(current))
            current, position = [], position + 2
            continue
        current.append(character)
        position += 1
    segments.append("".join(current))
    return segments


def read_dump(path, root):
    units = []
    with open(path, encoding="utf-8", errors="surrogateescape") as dump:
        for line in dump:
            fields = line.rstrip("\n").split("\t")
            if len(fields) < 4 or fields[0] in ("refused", "key", "data", "data-directory", "doctest"):
                continue
            place = re.fullmatch(r"(.+):(\d+)-(\d+)", fields[2])
            if not place or not place.group(1).endswith(".rs") or not place.group(1).startswith("crates/"):
                continue
            units.append(Unit(fields[0], fields[1], place.group(1), int(place.group(2)), int(place.group(3))))
    by_file = collections.defaultdict(list)
    for unit in units:
        by_file[unit.file].append(unit)
    for file, file_units in by_file.items():
        with open(os.path.join(root, file), encoding="utf-8", errors="surrogateescape") as source:
            tokens = lex(source.read())
        lines = [token[2] for token in tokens]
        for unit in file_units:
            if unit.kind not in KINDS_READ:
                continue
            unit.tokens = tokens[bisect.bisect_left(lines, unit.start):bisect.bisect_right(lines, unit.end)]
    return units, len(by_file)


def texts(tokens):
    return [text for _, text, _ in tokens]


def matching_angle(words, opening):
    """words[opening] 是 `<`：交配对的 `>` 的下标（`->` 的 `>` 不算），找不到交 len(words)。"""
    depth = 0
    for index in range(opening, len(words)):
        if words[index] == "<":
            depth += 1
        elif words[index] == ">" and not (index > 0 and words[index - 1] == "-"):
            depth -= 1
            if depth == 0:
                return index
    return len(words)


def leading_cfg(words):
    """单元开头的外属性里有没有 `#[cfg(…)]`。"""
    index = 0
    while index + 1 < len(words) and words[index] == "#" and words[index + 1] == "[":
        if index + 2 < len(words) and words[index + 2] == "cfg":
            return True
        depth, index = 0, index + 1
        while index < len(words):
            depth += words[index] == "["
            depth -= words[index] == "]"
            index += 1
            if depth == 0:
                break
    return False


def implementation_shape(unit):
    """impl 头单元 → dict(generics, first, path_root, self_names, trait, blanket)；认不出 impl 关键字的交 None。"""
    words = texts(unit.tokens)
    if "impl" not in words:
        return None
    index = words.index("impl") + 1
    generics = set()
    if index < len(words) and words[index] == "<":
        close = matching_angle(words, index)
        depth = 0
        for position in range(index, close + 1):
            if words[position] == "<":
                depth += 1
            elif words[position] == ">" and words[position - 1] != "-":
                depth -= 1
            elif depth == 1 and unit.tokens[position][0] == "ident" and words[position - 1] in ("<", ",", "const") \
                    and words[position] != "const":
                generics.add(strip_raw(words[position]))
        index = close + 1
    end, depth, split = len(words), 0, None
    for position in range(index, len(words)):
        word = words[position]
        if word == "<":
            depth += 1
        elif word == ">" and words[position - 1] != "-":
            depth -= 1
        elif depth == 0 and word in ("{", ";", "where"):
            end = position
            break
        elif depth == 0 and word == "for" and split is None and (position + 1 >= len(words) or words[position + 1] != "<"):
            split = position
    trait, trait_depth = None, 0
    for position in range(index, split if split is not None else index):
        if words[position] == "<":
            trait_depth += 1
        elif words[position] == ">" and words[position - 1] != "-":
            trait_depth -= 1
        elif trait_depth == 0 and unit.tokens[position][0] == "ident" and words[position] != "dyn":
            trait = strip_raw(words[position])
    self_range = range(split + 1 if split is not None else index, end)
    names = [strip_raw(words[p]) for p in self_range if unit.tokens[p][0] == "ident" and words[p] not in SKIPPED_BEFORE_THE_SELF_TYPE]
    first_position = next((p for p in self_range if unit.tokens[p][0] == "ident" and words[p] not in SKIPPED_BEFORE_THE_SELF_TYPE), None)
    path_root = first_position is not None and first_position + 2 < len(words) and words[first_position + 1] == ":" \
        and words[first_position + 2] == ":"
    return dict(generics=generics, first=names[0] if names else None, path_root=path_root, self_names=set(names),
                trait=trait if split is not None else None, blanket=split is not None and bool(set(names) & generics),
                first_position=first_position)


def use_shape(unit):
    """use 单元 → dict(names, aliases, glob, passes)：引进的名字（路径段与 self/super/crate 不算，`a as b` 只算 b）、(别名, 原名)、有没有 `*`、
    后面还接着 `::` 的名字（路径从它身上穿过去）。与 crates/singlefs-unit-graph 的 use_imports 与 names_a_use_path_passes_through 同一个取法。"""
    words, kinds = texts(unit.tokens), [kind for kind, _, _ in unit.tokens]
    if "use" not in words:
        return None
    start = words.index("use") + 1
    shape = dict(names=set(), aliases=[], glob=False, passes=set())
    for index in range(start, len(words)):
        word = words[index]
        if word == "*":
            shape["glob"] = True
            continue
        if kinds[index] != "ident":
            continue
        name = strip_raw(word)
        following = words[index + 1:index + 3]
        if following == [":", ":"]:
            shape["passes"].add(name)
            continue
        if name == "as":
            if index + 1 < len(words) and kinds[index + 1] == "ident" and index - 1 >= start and kinds[index - 1] == "ident":
                shape["aliases"].append((strip_raw(words[index + 1]), strip_raw(words[index - 1])))
            continue
        if following[:1] == ["as"]:
            continue
        if name not in ("self", "super", "crate"):
            shape["names"].add(name)
    return shape


class Index:
    def __init__(self, units):
        self.types_by_name = collections.defaultdict(list)
        self.types_by_module = collections.defaultdict(list)
        self.uses_by_module = collections.defaultdict(list)
        self.implementations = []
        self.implementations_by_first_name = collections.defaultdict(list)
        self.traits = set()
        self.renamed, self.passes = set(), set()
        for unit in units:
            unit.use = unit.shape = None
            if unit.kind in TYPE_KINDS:
                self.types_by_name[unit.name].append(unit)
                self.types_by_module[unit.module].append(unit)
            elif unit.kind == "use":
                unit.use = use_shape(unit)
                if unit.use:
                    self.uses_by_module[unit.module].append(unit)
                    self.renamed.update(original for _alias, original in unit.use["aliases"])
                    self.passes.update(unit.use["passes"])
            elif unit.kind == "impl":
                unit.shape = implementation_shape(unit)
                if unit.shape:
                    self.implementations.append(unit)
                    if unit.shape["first"]:
                        self.implementations_by_first_name[unit.shape["first"]].append(unit)
            elif unit.kind == "trait":
                self.traits.add(unit.name)

    def waits_for_its_wrapper_type(self, unit):
        """verify-copy 的 implementation_waits_for_its_wrapper_type 五条的近似（照这份扫描自己的切词与模块认法）。"""
        shape = unit.shape
        first = shape["first"]
        if not shape["blanket"] or first is None or shape["path_root"] or first in shape["generics"] \
                or first in self.renamed or first in self.passes:
            return False
        same = [candidate for candidate in self.types_by_module[unit.module] if candidate.name == first]
        if not same or not all(candidate.kind in NOMINAL_KINDS for candidate in same):
            return False
        return not any(first in use.use["names"] for use in self.uses_by_module[unit.module])

    def aliases_holding_a_trait_object(self, name):
        return [alias for alias in self.types_by_name.get(name, ()) if alias.kind == "type" and "dyn" in texts(alias.tokens)]


def matching_close(words, opening):
    pairs = {"(": ")", "[": "]", "{": "}"}
    closing, depth = pairs[words[opening]], 0
    for index in range(opening, len(words)):
        if words[index] == words[opening]:
            depth += 1
        elif words[index] == closing:
            depth -= 1
            if depth == 0:
                return index
    return len(words)


def bound_regions(words, kinds):
    """泛型参数表、where 子句、trait 头的父 trait 段（词的下标，左闭右开）：这几处的 `:`、`+` 后面是约束。"""
    regions = []
    for index, word in enumerate(words):
        if word in ("fn", "struct", "enum", "union", "trait", "type") and index + 2 < len(words) and kinds[index + 1] == "ident" \
                and words[index + 2] == "<":
            regions.append((index + 2, matching_angle(words, index + 2)))
        elif word == "impl" and index + 1 < len(words) and words[index + 1] == "<":
            regions.append((index + 1, matching_angle(words, index + 1)))
        elif word == "where":
            cursor, depth = index + 1, 0
            while cursor < len(words):
                if words[cursor] in ("(", "["):
                    depth += 1
                elif words[cursor] in (")", "]"):
                    depth -= 1
                elif depth == 0 and words[cursor] in ("{", ";"):
                    break
                cursor += 1
            regions.append((index, cursor))
        if word == "trait" and index + 1 < len(words) and kinds[index + 1] == "ident":
            cursor = index + 2
            if cursor < len(words) and words[cursor] == "<":
                cursor = matching_angle(words, cursor) + 1
            if cursor < len(words) and words[cursor] == ":":
                end = cursor
                while end < len(words) and words[end] not in ("{", "where", ";"):
                    end += 1
                regions.append((cursor, end))
    return regions


def name_after_a_bound_opening(words, kinds, position):
    """words[position] 是 `(` 或 `for`：交括号里或 `for<…>` 之后头一个名字（路径取末段）。"""
    cursor = position + 1
    if words[position] == "for" and cursor < len(words) and words[cursor] == "<":
        cursor = matching_angle(words, cursor) + 1
    name = None
    while cursor < len(words) and kinds[cursor] == "ident":
        name = strip_raw(words[cursor])
        if words[cursor + 1:cursor + 3] == [":", ":"]:
            cursor += 3
            continue
        break
    return name


class Report:
    def __init__(self):
        self.instances = collections.OrderedDict()
        self.counters = collections.defaultdict(collections.Counter)

    def declare(self, name):
        self.instances.setdefault(name, [])

    def add(self, name, unit, detail, counter=None):
        self.instances.setdefault(name, []).append(f"instance class={name} at={unit.where()} detail={detail}")
        if counter:
            self.counters[name][counter] += 1

    def print(self):
        for name, lines in self.instances.items():
            extra = "".join(f" {key}={value}" for key, value in sorted(self.counters[name].items()))
            print(f"class={name} instances={len(lines)}{extra}")
            for line in lines:
                print(line)


def signature_words(unit):
    words = texts(unit.tokens)
    return words[:words.index("{")] if "{" in words else words


def callback_markers(words):
    found = []
    for index, word in enumerate(words[:-1]):
        if word in ("dyn", "impl", ":", "+") and strip_raw(words[index + 1]) in CALLABLE_TRAITS:
            found.append(f"{word}{strip_raw(words[index + 1])}")
        elif word == "fn" and words[index + 1] == "(":
            found.append("fn(")
    return found


def scan_shapes(units, index, report):
    for name in ("cfg-arm-name-supplied-by-a-glob-or-an-item-macro", "bound-after-a-parenthesis-or-for", "trait-object-with-an-auto-trait-first",
                 "static-inside-a-function-body-holding-a-trait-object", "static-typed-through-an-alias-holding-a-trait-object",
                 "downcast-to-a-trait-object-outside-the-any-rule", "alias-of-any", "variant-brought-in-by-a-use",
                 "implementation-written-on-a-type-alias", "type-renamed-by-a-use", "trait-object-inside-the-self-type-of-a-trait-implementation",
                 "code-held-by-a-static", "raw-identifier-read-differently-by-the-two-implementations", "use-written-inside-a-macro-expansion"):
        report.declare(name)
    modules_with_a_cfg_supplier = {unit.module for unit in units if leading_cfg(texts(unit.tokens)) and (
        (unit.kind == "use" and unit.use and unit.use["glob"]) or unit.kind == "item_macro")}
    for unit in units:
        words, kinds = texts(unit.tokens), [kind for kind, _, _ in unit.tokens]
        idents = set(unit.idents())
        if unit.kind in NOMINAL_KINDS and leading_cfg(words) and unit.module in modules_with_a_cfg_supplier:
            waiting = [head for head in index.implementations_by_first_name.get(unit.name, ())
                       if head.module == unit.module and index.waits_for_its_wrapper_type(head)]
            report.add("cfg-arm-name-supplied-by-a-glob-or-an-item-macro", unit, f"waiting_implementations={len(waiting)}",
                       "with_a_waiting_implementation" if waiting else None)
        if unit.kind in ("fn", "impl", "trait", "struct", "enum", "union", "type"):
            for start, end in bound_regions(words, kinds):
                for position in range(start, min(end, len(words) - 1)):
                    single_colon = words[position] == ":" and words[position - 1] != ":" and words[position + 1] != ":"
                    if (single_colon or words[position] == "+") and words[position + 1] in ("(", "for"):
                        trait = name_after_a_bound_opening(words, kinds, position + 1)
                        report.add("bound-after-a-parenthesis-or-for", unit, f"{words[position]}{words[position + 1]} trait={trait}",
                                   "naming_a_workspace_trait" if trait in index.traits else None)
        for position, word in enumerate(words[:-1]):
            if word in ("impl", "dyn") and words[position + 1] == "(":
                trait = name_after_a_bound_opening(words, kinds, position + 1)
                report.add("bound-after-a-parenthesis-or-for", unit, f"{word}( trait={trait}",
                           "naming_a_workspace_trait" if trait in index.traits else None)
            if word == "dyn" and strip_raw(words[position + 1]) in AUTO_TRAITS:
                cursor, depth = position + 1, 0
                while cursor < len(words) and not (depth == 0 and words[cursor] in (">", ",", ")", ";", "=", "{", "]")):
                    depth += words[cursor] in ("<", "(")
                    depth -= words[cursor] in (">", ")") and depth > 0
                    if depth == 0 and words[cursor] == "+" and cursor + 1 < len(words) and kinds[cursor + 1] == "ident" \
                            and strip_raw(words[cursor + 1]) not in AUTO_TRAITS:
                        in_reach = unit.kind in ("static", "item_macro") or "Any" in idents
                        report.add("trait-object-with-an-auto-trait-first", unit, f"trait={strip_raw(words[cursor + 1])}",
                                   "in_a_static_or_any_unit" if in_reach else None)
                        break
                    cursor += 1
            if unit.kind == "fn" and word == "static" and kinds[position] == "ident":
                name_position = position + 2 if words[position + 1] == "mut" else position + 1
                if name_position + 1 < len(words) and kinds[name_position] == "ident" and words[name_position + 1] == ":":
                    cursor, depth = name_position, 0
                    while cursor < len(words) and not (depth == 0 and words[cursor] == ";"):
                        depth += words[cursor] in ("(", "[", "{")
                        depth -= words[cursor] in (")", "]", "}")
                        cursor += 1
                    if "dyn" in words[name_position:cursor]:
                        report.add("static-inside-a-function-body-holding-a-trait-object", unit, f"static={words[name_position]}")
            if unit.kind == "fn" and word == "thread_local" and words[position + 1] == "!" and position + 2 < len(words) \
                    and words[position + 2] in ("(", "[", "{") and "dyn" in words[position + 2:matching_close(words, position + 2)]:
                report.add("static-inside-a-function-body-holding-a-trait-object", unit, "thread_local!")
            if word in DOWNCAST_METHODS and position > 1 and (words[position - 1] == "." or words[position - 2:position] == [":", ":"]):
                report.counters["downcast-to-a-trait-object-outside-the-any-rule"]["downcasts_seen"] += 1
                if words[position + 1:position + 4] == [":", ":", "<"]:
                    target = words[position + 3:matching_angle(words, position + 3) + 1]
                    aliases = [alias.where() for name in set(target) for alias in index.aliases_holding_a_trait_object(strip_raw(name))]
                    if "dyn" in target and "Any" not in idents:
                        report.add("downcast-to-a-trait-object-outside-the-any-rule", unit, "unit_does_not_name_Any", "unit_does_not_name_any")
                    elif "dyn" not in target and aliases:
                        report.add("downcast-to-a-trait-object-outside-the-any-rule", unit, f"target_alias={aliases[0]}", "target_is_an_alias")
        if unit.kind == "macro_rules":
            for position, word in enumerate(words[:-1]):
                if word != "use" or kinds[position] != "ident":
                    continue
                cursor = position + 1
                while cursor < len(words) and words[cursor] != ";":
                    cursor += 1
                passed = {strip_raw(words[p]) for p in range(position + 1, cursor) if kinds[p] == "ident" and words[p + 1:p + 3] == [":", ":"]}
                types = sorted(name for name in passed if any(candidate.kind in NOMINAL_KINDS for candidate in index.types_by_name.get(name, ())))
                renamed = "as" in words[position + 1:cursor]
                report.add("use-written-inside-a-macro-expansion", unit, f"passes_through_types={types} renames={renamed}",
                           "passing_through_or_renaming_a_type" if types or renamed else None)
        if unit.kind == "type" and "Any" in idents:
            report.add("alias-of-any", unit, f"has_dyn={'dyn' in words}")
        if unit.kind in ("static", "item_macro"):
            if "dyn" not in words:
                for name in sorted(idents):
                    for alias in index.aliases_holding_a_trait_object(name):
                        report.add("static-typed-through-an-alias-holding-a-trait-object", unit, f"alias={alias.where()}")
            markers = callback_markers(words)
            if markers:
                report.add("code-held-by-a-static", unit, f"markers={','.join(sorted(set(markers)))}")
        if unit.kind == "use" and unit.use:
            for passed in sorted(unit.use["passes"]):
                enums = [candidate for candidate in index.types_by_name.get(passed, ()) if candidate.kind == "enum"]
                if enums:
                    heads = [head for head in index.implementations_by_first_name.get(passed, ()) if head.shape["trait"]]
                    concrete = [head for head in heads if not head.shape["blanket"]]
                    report.add("variant-brought-in-by-a-use", unit, f"enum={passed} trait_implementations={len(heads)} concrete={len(concrete)}",
                               "enum_with_a_concrete_trait_implementation" if concrete else None)
            for alias, original in unit.use["aliases"]:
                if any(candidate.kind in NOMINAL_KINDS for candidate in index.types_by_name.get(original, ())):
                    heads = [head for head in index.implementations_by_first_name.get(original, ()) if head.shape["trait"]]
                    report.add("type-renamed-by-a-use", unit, f"{original}_as_{alias} trait_implementations={len(heads)}",
                               "type_with_a_trait_implementation" if heads else None)
        if unit.kind == "impl" and unit.shape and unit.shape["first"] and not unit.shape["path_root"] \
                and unit.shape["first"] not in unit.shape["generics"]:
            first = unit.shape["first"]
            same = [candidate for candidate in index.types_by_module[unit.module] if candidate.name == first]
            everywhere = index.types_by_name.get(first, ())
            if (same and not any(candidate.kind in NOMINAL_KINDS for candidate in same)) or (
                    not same and everywhere and all(candidate.kind == "type" for candidate in everywhere)):
                report.add("implementation-written-on-a-type-alias", unit, f"self_type={first} blanket={unit.shape['blanket']}")
        if unit.kind in NOMINAL_KINDS:
            for position, word in enumerate(words[:-1]):
                if word != "dyn":
                    continue
                cursor, trait = position + 1, None
                while cursor < len(words) and kinds[cursor] == "ident":
                    trait = strip_raw(words[cursor])
                    if words[cursor + 1:cursor + 3] != [":", ":"]:
                        break
                    cursor += 3
                heads = [head for head in index.implementations_by_first_name.get(unit.name, ()) if head.shape["trait"]]
                if trait in index.traits and heads:
                    report.add("trait-object-inside-the-self-type-of-a-trait-implementation", unit,
                               f"dyn={trait} trait_implementations_of_the_type={len(heads)}")
        for position, (kind, text, _line) in enumerate(unit.tokens):
            if kind != "ident" or not text.startswith("r#"):
                continue
            report.counters["raw-identifier-read-differently-by-the-two-implementations"]["raw_identifiers_seen"] += 1
            previous = words[position - 1] if position > 0 else ""
            if strip_raw(text) == "Any":
                report.add("raw-identifier-read-differently-by-the-two-implementations", unit, "r#Any", "raw_any")
            elif unit.kind == "fn" and position < len(signature_words(unit)) and previous in ("dyn", "impl", ":", "+"):
                report.add("raw-identifier-read-differently-by-the-two-implementations", unit, f"signature {previous} {text}", "raw_signature_trait")


def registered_entries(root):
    with open(os.path.join(root, REGISTRY), encoding="utf-8") as registry:
        text = registry.read()
    start = text.find("pub const JUDGING_UNITS_ENTRY_GROUPS")
    if start < 0:
        raise SystemExit(f"  ✗ {REGISTRY} 里找不到 JUDGING_UNITS_ENTRY_GROUPS")
    body = text[start:text.find("\n];", start)]
    groups = []
    for block in body.split("JudgingUnitsEntryGroup {")[1:]:
        name = re.search(r'group_name:\s*"([^"]+)"', block)
        listed = block.split("entry_functions", 1)[-1]
        groups.append((name.group(1) if name else "?", re.findall(r'"([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)"', listed)))
    return groups


def entry_unit(units_by_qualified, function_units, path):
    if path in units_by_qualified:
        return units_by_qualified[path]
    segments = path.split("::")
    for unit in function_units:
        parts = split_qualified(unit.qualified)
        if len(parts) == len(segments) and parts[:-2] == segments[:-2] and parts[-1] == segments[-1] \
                and parts[-2].startswith(("<impl", "<trait")) and re.search(r"\b" + re.escape(segments[-2]) + r"\b", parts[-2]):
            return unit
    return None


def scan_callbacks_of_the_registered_entries(units, index, root, report):
    name = "code-supplied-by-the-caller-of-a-registered-entry"
    report.declare(name)
    function_units = [unit for unit in units if unit.kind == "fn"]
    units_by_qualified = {unit.qualified: unit for unit in function_units}
    heads_by_qualified = {unit.qualified: unit for unit in units if unit.kind in ("impl", "trait")}
    registered = found = 0
    reported = set()
    for group, entries in registered_entries(root):
        for path in entries:
            registered += 1
            entry = entry_unit(units_by_qualified, function_units, path)
            if entry is None:
                report.counters[name]["entries_not_found"] += 1
                continue
            found += 1
            signature = signature_words(entry)
            for marker in callback_markers(signature):
                if (group, entry.qualified, marker) not in reported:
                    reported.add((group, entry.qualified, marker))
                    report.add(name, entry, f"group={group} entry_signature {marker}", "in_an_entry_signature")
            container = heads_by_qualified.get("::".join(split_qualified(entry.qualified)[:-1]))
            pending = [strip_raw(word) for word, (kind, _, _) in zip(signature, entry.tokens) if kind == "ident"]
            if container is not None and container.shape:
                pending.extend(sorted(container.shape["self_names"]))
            looked_into = set()
            while pending and len(looked_into) < BREADTH_LIMIT:
                for type_unit in index.types_by_name.get(pending.pop(), ()):
                    if id(type_unit) in looked_into:
                        continue
                    looked_into.add(id(type_unit))
                    markers = callback_markers(texts(type_unit.tokens))
                    if markers and (group, type_unit.qualified) not in reported:
                        reported.add((group, type_unit.qualified))
                        report.add(name, type_unit, f"group={group} reached_from={entry.qualified} markers={','.join(sorted(set(markers)))}",
                                   "in_a_type_reached_from_an_entry")
                    pending.extend(type_unit.idents())
    return registered, found


def scan_procedural_macro_crates(root, report):
    name = "procedural-macro-crate"
    report.declare(name)
    crates_directory = os.path.join(root, "crates")
    for crate in sorted(os.listdir(crates_directory)):
        manifest = os.path.join(crates_directory, crate, "Cargo.toml")
        if os.path.isfile(manifest):
            with open(manifest, encoding="utf-8") as text:
                if re.search(r"^\s*proc[-_]macro\s*=\s*true", text.read(), re.MULTILINE):
                    report.instances[name].append(f"instance class={name} at=crates/{crate}/Cargo.toml detail=proc-macro")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dump", required=True)
    parser.add_argument("--root", required=True)
    arguments = parser.parse_args()
    units, files = read_dump(arguments.dump, arguments.root)
    if not units:
        print(f"  ✗ {arguments.dump} 里一个 crates/ 下的 Rust 单元都没有", file=sys.stderr)
        return 2
    index = Index(units)
    report = Report()
    scan_shapes(units, index, report)
    registered, found = scan_callbacks_of_the_registered_entries(units, index, arguments.root, report)
    scan_procedural_macro_crates(arguments.root, report)
    report.print()
    print(f"name=scan rust_units={len(units)} files={files} classes={len(report.instances)} registered_entries={registered} entries_found={found}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
