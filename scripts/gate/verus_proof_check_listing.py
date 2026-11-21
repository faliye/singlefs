"""形式化证明判法（入口 verus-proof-check.py）的一块：MIR 文本的读法、被验区域的函数与常量列表、组成键的各项（函数组件、依赖行）、判定库的计划与记账（plan_done、lookup 等）、项的点名与汇总。
入口 verus-proof-check.py 把这一块的名字再导出。
"""

import collections
import glob
import hashlib
import json
import os
import re
import sys
import tomllib
from lib_verdict_library import FunctionUnitIndex, Tally, UnitKey, digest_text, digest_tree
from lib_verdict_library import key_of, refused_component  # noqa: E402
import lib_verdict_library  # noqa: E402  library_root（库根）、locked_dependency_lines（闭包的依赖版本行算不出时退回整个 crate 的）
import lib_reuse_checks as reuse_checks  # noqa: E402

from verus_proof_check_base import (
     SCRIPT_DIRECTORY, REGISTRY, SCRIPT_KEY, EXIT_GREEN, TRUST_KINDS, CFG_KIND, BREAK, FORMAL_BREAK, ITEM_TALLY_FILE_VARIABLE,
     ITEM_DONE_FILE_VARIABLE, Red, crate_identifier, read_registry, kernel_files, verus_directory, read_version_json, crate_manifest,
     check_dependencies, RUST_IDENTIFIER, kernel_module, crate_package_name, innermost_unit, joined_tokens, body_open, body_open_past_unit_end,
     whole_entry_end, assume_specification_parts,
)

def unit_path(crate, module, unit):
    """条目的全路径 <crate>::<模块>::<名>：fn 照 verus_point_name（impl 里的写 <类型>::<方法>），别的条目是它的名字（套在别的条目里的写嵌套路径）。"""
    if unit is None:
        return "::".join(part for part in (crate, module, "（不在任何条目里）") if part)
    name = (verus_point_name(unit) if unit.kind == "fn" else None) or (unit.short if unit.container is None else unit.name)
    return "::".join(part for part in (crate, module, name) if part)


def region_spans(rust_file):
    """被验区域在这份文件里的词法单元区间：每个 verus! { … }，与属性写法标的条目（attribute_verified）。"""
    spans = list(verus_ranges(rust_file))
    for unit in rust_file.units:
        if not unit.data and unit.first_token is not None and unit.last_token is not None and attribute_verified(rust_file, unit):
            spans.append((unit.first_token, unit.last_token))
    return spans


def trusted_declarations(rust_file, relative, crate, module, spans, cfg_macros):
    """被验区域里每一处要登记的可信声明与 cfg 原子：[{kind, file, point, digest, line}]，次序照文件里的位置。
    assume_specification：点名的是方括号里那一项（词法单元连写，例 <[T]>::binary_search），规格原文是从它到同一层的 ; 为止；
    external_body：点名的是带它的 fn 的全路径，规格原文是函数体的 { 之前那一截（签名、requires、ensures；函数体的 { 照 body_open 认，
    一个 impl 里最后一个 fn 也取得到签名，改签名 sha256 就变、只改函数体不变）；带它的不是 fn（类型规格）时规格原文是整个条目；
    uninterp 与 external_*_specification：点名的是那个条目的全路径，规格原文是整个条目（整个条目都照半开区间取到 whole_entry_end，不连下一个条目的头一个词）；
    cfg 原子（#[cfg(…)]、#[cfg_attr(…)]、cfg!(…)，与定义里有 cfg 的 macro_rules! 宏在被验区域里的调用）：点名的是 <所在条目的全路径>#<原子>（例 crate::m::pick#cfg(feature="wide")、
    crate::m::pick#plus_span!），规格原文是所在条目整个（与 uninterp 同一种半开区间，取到 whole_entry_end）。"""
    found = []
    seen = set()
    texts = rust_file.texts
    for low, high in spans:
        for index in range(low, min(high + 1, rust_file.count)):
            word = texts[index]
            if rust_file.kinds[index] != "ident" or (index, word) in seen or not (word in TRUST_KINDS or word in ("cfg", "cfg_attr") or word in cfg_macros):
                continue
            seen.add((index, word))
            unit = innermost_unit(rust_file, index)
            if word in TRUST_KINDS:
                if word == "assume_specification":
                    point, _closing, end = assume_specification_parts(rust_file, index, high)
                    spec = joined_tokens(rust_file, index, end + 1)
                elif word == "external_body":
                    point = unit_path(crate, module, unit)
                    # fn 取签名到函数体的 { 之前；类型规格（external_type_specification 那几个 struct）取整个条目，与别的种类同一种取法
                    if "body-open-past-unit-end" in BREAK:
                        opening = body_open_past_unit_end(rust_file, unit)
                    else:
                        opening = body_open(rust_file, unit) if unit is not None and unit.kind == "fn" else None
                    spec = joined_tokens(rust_file, unit.first_token, opening if opening is not None else whole_entry_end(unit)) if unit is not None else word
                else:
                    point = unit_path(crate, module, unit)
                    spec = joined_tokens(rust_file, unit.first_token, whole_entry_end(unit)) if unit is not None else word
                found.append({"kind": word, "file": relative, "point": point, "digest": hashlib.sha256(spec.encode()).hexdigest(), "line": rust_file.lines[index]})
                continue
            atom = None
            if word in ("cfg", "cfg_attr") and index + 1 < rust_file.count and texts[index + 1] in ("(", "!"):
                opening = index + 1 if texts[index + 1] == "(" else index + 2
                closing = rust_file.pairs.get(opening)
                if closing is not None:
                    atom = word + ("!" if texts[index + 1] == "!" else "") + "".join(texts[opening:closing + 1])
            elif word in cfg_macros and index + 1 < rust_file.count and texts[index + 1] == "!" and "cfg-macro-unchecked" not in FORMAL_BREAK:
                atom = f"{word}!"
            if atom is not None:
                spec = joined_tokens(rust_file, unit.first_token, whole_entry_end(unit, cfg_atom=True)) if unit is not None else atom
                found.append({"kind": CFG_KIND, "file": relative, "point": f"{unit_path(crate, module, unit)}#{atom}",
                              "digest": hashlib.sha256(spec.encode()).hexdigest(), "line": rust_file.lines[index]})
    return found


# ── 产品构建与 verus --compile 构建的 MIR 逐函数比（cfg 分叉、宏里的 cfg 让两边编出的函数体不同，两次产品编译看不见）──

MIR_ITEM_HEAD = re.compile(r"^(fn|const|static|promoted\[\d+\] in) ")
# MIR 规整表：同一个函数两边印法不同的名字，(另一种印法, 规整成的印法)；比之前两边都把前一个整词写成后一个（mir_path_pattern 卡词边界）。
# std::rt::panic_fmt 是 core::panicking::panic_fmt 的再导出：Verus 构建里 verus! 的展开经 ::core 用了一次 core，rustc 印路径时改取 core 自己的路径，
# 产品构建照旧印 std 的再导出。core 上 103 个条目 124 行不同全是这一处，两份 rlib 引用的都是 core::panicking::panic_fmt 的符号（nm），
# 规整之后 5956 个条目逐字相同，在 verus! 外面造的一处真分叉规整之后照报。表外的名字照红：核过两边是同一个函数（rlib 引用的符号相同）再加一行，不按前缀整片当成同一个。
MIR_PATH_ALIASES = (("std::rt::panic_fmt", "core::panicking::panic_fmt"),)


def mir_path_pattern(alias):
    """规整表一行的认法：整词，前面不接标识符字符或 :，后面接 ( 或 ::<（调用与带类型参数的调用）。
    弄坏开关：mir-alias-without-boundary 下不卡词边界（前后接什么都改）；mir-alias-by-prefix 下按路径前缀认（std::rt::<名> 一律当 core::panicking::<名>）。"""
    if "mir-alias-by-prefix" in BREAK:
        return re.compile(r"(?<![A-Za-z0-9_:])" + re.escape(alias.rsplit("::", 1)[0] + "::") + r"(?=[A-Za-z_])")
    if "mir-alias-without-boundary" in BREAK:
        return re.compile(re.escape(alias))
    return re.compile(r"(?<![A-Za-z0-9_:])" + re.escape(alias) + r"(?=\(|::<)")


def normalized_mir(text):
    """照 MIR_PATH_ALIASES 规整一份 MIR 文本：交 (规整后的文本, 改写了几处)。弄坏开关 mir-panic-path-unnormalized 下不改。"""
    if "mir-panic-path-unnormalized" in BREAK:
        return text, 0
    total = 0
    for alias, canonical in MIR_PATH_ALIASES:
        replacement = canonical.rsplit("::", 1)[0] + "::" if "mir-alias-by-prefix" in BREAK else canonical
        text, count = mir_path_pattern(alias).subn(replacement.replace("\\", "\\\\"), text)
        total += count
    return text, total


def mir_alias_text(product_rewrites, verus_rewrites):
    """成功句与红句里报规整了几处：表里每一行写成「甲 → 乙」，两边各几处。"""
    table = "、".join(f"{alias} → {canonical}" for alias, canonical in MIR_PATH_ALIASES)
    if "mir-panic-path-unnormalized" in BREAK:
        return f"没照规整表改写（弄坏开关 mir-panic-path-unnormalized；表里 {table}）"
    return f"比之前照规整表改写了产品构建 {product_rewrites} 处、Verus 构建 {verus_rewrites} 处（{table}，同一个函数的两种印法）"


def mir_items(text):
    """--emit=mir 的文本按条目切：{条目名: [整段原文…]}（fn 的名字取到 ( 之前；同名的几段按出现的次序收）。"""
    items, current, buffer = {}, None, []
    for line in text.split("\n"):
        if current is None:
            if MIR_ITEM_HEAD.match(line) and line.endswith("{"):
                current = re.sub(r"\(.*", "", line) if line.startswith("fn ") else line
                buffer = [line]
            continue
        buffer.append(line)
        if line == "}":
            items.setdefault(current, []).append("\n".join(buffer))
            current = None
    return items


def newest_mir(target, identifier):
    """编译目录 debug/deps 下这个 crate 的 .mir（几份取最新的一份）；没有交 None。"""
    found = sorted(glob.glob(os.path.join(target, "debug", "deps", f"{identifier}-*.mir")), key=os.path.getmtime)
    return found[-1] if found else None


def verus_executable(registry, options):
    directory = verus_directory(registry, options.get("verus_root"))
    executable = os.path.join(directory, "verus")
    version = read_version_json(directory)
    if not os.access(executable, os.X_OK) or version is None:
        raise Red(f"找不到 Verus（{executable} 不可执行或旁边没有 version.json）",
                  f"装登记的发布 {registry.tag}（{REGISTRY} 的 release 行），或用 SINGLEFS_VERUS_ROOT 指到它的安装目录")
    return executable, version


def crate_registry(root, options):
    """这一趟判的那一个 crate 的登记：run_per_crate 交的 options["crate_view"]；没有时读登记表（只登记了一个 crate）。"""
    if options.get("crate_view") is not None:
        return options["crate_view"]
    registry = read_registry(root)
    if len(registry.crates) != 1:
        raise Red(f"登记表 {REGISTRY} 登记了 {len(registry.crates)} 个 crate，这一步只能判一个", "经 run_check 起（它逐个 crate 判）")
    return registry


def prepared(root, options):
    registry = crate_registry(root, options)
    kernels = kernel_files(root, registry)
    manifest = crate_manifest(root, registry)
    check_dependencies(registry, manifest)
    executable, _version = verus_executable(registry, options)
    return registry, kernels, manifest, executable


class UsageError(Exception):
    """--item 点名了没有的项、或给了这一格不收的 --item：退 2，列出可点名的项。"""

    def __init__(self, message, available):
        super().__init__(message)
        self.message, self.available = message, available


def unit_index(root, options):
    """被判的仓的函数单元索引（lib_function_units.py 的切法，Rust 工作区读一次）：一格里各项共用，存进 options；
    run_per_crate 与 list-items 把同一份交给逐 crate、逐格的各次调用，一趟里 Rust 工作区只载入一次（索引只按根建，与 crate 无关）。
    弄坏开关 unit-index-not-shared：每次调都另建一份、不存进 options。"""
    if "unit-index-not-shared" in BREAK:
        return FunctionUnitIndex(root)
    if options.get("unit_index") is None:
        options["unit_index"] = FunctionUnitIndex(root)
    return options["unit_index"]


def verus_ranges(rust_file):
    """一份核里每个 verus! { … } 的词号范围（{ 与配对的 }）。"""
    return [(position + 2, rust_file.pairs[position + 2]) for position in range(rust_file.count - 2)
            if rust_file.texts[position] == "verus" and rust_file.texts[position + 1] == "!" and rust_file.texts[position + 2] == "{"
            and position + 2 in rust_file.pairs]


class VerusFunction(collections.namedtuple("VerusFunction", "name relative unit path")):
    """被验区域里的一个 fn：name 是交给 --verify-function 的名字（自由 fn 是它的名字，impl 与 trait 里的是 <类型或 trait>::<方法>，与 Verus 列的可点名函数同一种写法），
    relative 是核相对仓根的路径，unit 是 lib_function_units 切出的单元（算键的入口是 <核>::<unit.name>），path 是全路径
    <crate>::<模块>::<name>（项名用它，与 Verus 的 func-details 同一种写法；核是 crate 入口的没有模块那一段）。"""


def impl_parts(label):
    """impl 单元的嵌套路径（lib_function_units 的写法：<impl …>，去掉了 impl 后面的泛型参数表与 where 子句）交 (trait 名或 None, 类型名)：都取路径的最后一段、去掉类型参数。
    认不出（&T、dyn、元组这类）交 (trait, None)。"""
    inner = label[len("<impl "):-1] if label.startswith("<impl ") and label.endswith(">") else label
    trait, _for, target = inner.rpartition(" for ") if " for " in inner else ("", "", inner)

    def last_segment(text):
        text = text.strip().split("<", 1)[0].strip()
        segment = text.split("::")[-1].strip()
        return segment if RUST_IDENTIFIER.fullmatch(segment or "") else None
    return (last_segment(trait) if trait else None), last_segment(target)


def unit_attribute_words(rust_file, unit):
    """单元开头那几个外属性 #[…] 里的词（文档注释不算）。"""
    words, index = [], unit.first_token
    while index is not None and index + 1 < rust_file.count and rust_file.texts[index] == "#" and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
        close = rust_file.pairs[index + 1]
        words += rust_file.texts[index + 2:close]
        index = close + 1
    return words


def attribute_verified(rust_file, unit):
    """属性写法标的被验单元：它自己或包它的 impl / trait 带 #[verus_verify] 或 #[verus_spec(…)]（弄坏开关 attribute-mode-unrecognized 下不认）。"""
    if "attribute-mode-unrecognized" in FORMAL_BREAK:
        return False
    for owner in (unit, unit.container):
        if owner is not None and owner.first_token is not None and any(word in ("verus_verify", "verus_spec") for word in unit_attribute_words(rust_file, owner)):
            return True
    return False


def in_verified_region(rust_file, unit, ranges):
    """单元在被验区域里：落在某个 verus! { … } 里，或是属性写法标的（attribute_verified）。"""
    if unit.first_token is None:
        return False
    return any(low < unit.first_token < high for low, high in ranges) or attribute_verified(rust_file, unit)


def verus_point_name(unit):
    """交给 --verify-function 的名字：自由 fn 是它的名字；impl 里的方法是 <类型>::<方法>（trait impl 也写类型，Verus 0.2026.09.20 列可点名函数就是这么写的，
    例：impl Device for GoodDevice 的 read_slot 列成 GoodDevice::read_slot，同一个类型的固有方法与 trait 方法同名时按名点两个一起验）；
    trait 里的方法是 <trait>::<方法>。认不出类型的交 None。
    弄坏开关 methods-refused 下 impl 与 trait 里的一律交 None（判红要挪成自由 fn）。"""
    container = unit.container
    if container is None:
        return unit.short
    if "methods-refused" in FORMAL_BREAK:
        return None
    if container.kind == "trait":
        return f"{container.short}::{unit.short}"
    if container.kind == "impl":
        _trait, target = impl_parts(container.name)
        return f"{target}::{unit.short}" if target else None
    return None


def verus_functions(root, registry, kernels, options):
    """核的被验区域（verus! { … } 里，与属性写法标的）里的每个 fn 单元（spec、proof、exec 都算；impl 与 trait 里的方法也算），按核与行号的次序：[VerusFunction]。
    空真认所属函数、必红世界认受影响的函数用它；证明成哪几项另由 proof_items 从里面挑（spec fn 没有证明义务的不成项，trait 里没有函数体的声明不成项）。
    同一个模块里两个 fn 交给 --verify-function 的名字相同（同一个类型的固有方法与 trait 方法同名，Verus 按名点会把两个一起验）判红，出路是改名。
    认不出类型的 impl（&T、dyn 这类）里的方法判红。项名是全路径（VerusFunction.path），不同模块里同名的各是各的项。"""
    index = unit_index(root, options)
    manifest = crate_manifest(root, registry)
    crate = crate_identifier(registry, manifest)
    functions = []
    try:
        workspace = index.workspace()
        for kernel in kernels:
            relative = os.path.relpath(kernel, root).replace(os.sep, "/")
            module = kernel_module(registry, manifest, relative)
            rust_file = workspace.file(relative)
            ranges = verus_ranges(rust_file)
            names = {}
            for unit in rust_file.units:
                if unit.kind != "fn" or not in_verified_region(rust_file, unit, ranges):
                    continue
                if unit.container is not None and unit.container.kind not in ("impl", "trait"):
                    continue   # fn 里套的 fn 不另成函数（随外层验）
                name = verus_point_name(unit)
                if name is None:
                    raise Red(f"{relative}:{unit.first_line} 的 {unit.name} 在 verus! 里的 impl 或 trait 里，认不出交给 --verify-function 的名字"
                              + ("（弄坏开关 methods-refused）" if "methods-refused" in FORMAL_BREAK else "（impl 的类型不是一个具名的类型）"),
                              "impl 写成 impl <Trait> for <类型名> 或 impl <类型名>；确要别的写法，给 verus-proof-check.py 的 verus_point_name 补上认法")
                if name in names:
                    raise Red(f"{relative} 的被验区域里有两个 fn 交给 --verify-function 的名字都是 {name}（第 {names[name].first_line} 行与第 {unit.first_line} 行）："
                              "Verus 按名点会把两个一起验，逐项点不准",
                              "给其中一个改名（同一个类型的固有方法与 trait 方法同名最常见），同步改必红世界表里引它的行")
                names[name] = unit
                functions.append(VerusFunction(name, relative, unit, "::".join(part for part in (crate, module, name) if part)))
    except (OSError, KeyError, ValueError) as error:
        raise Red(f"切不出核里的函数单元（scripts/gate/lib_function_units.py）：{error}", "先让 python3 scripts/gate/lib_function_units.py dump <根> --language rust 在这棵树上跑通")
    return functions


# verus! 里的 derive 只有 Clone 生成 Verus 要验的函数（<类型>::clone）。依据是真 Verus 探针（0.2026.09.20，逐函数 verified 数在
# research/prompts/pruning-survey-2026-10-15/verus-probe.log）：--verify-function 点不中时 Verus 列的可点名函数是 90 个自由 fn、8 个 <类型>::clone
# （核里 derive(Clone, …) 的正好 8 个类型）与 2 个 CoreFailure::arrow_*（spec，没有义务）；Copy、Debug、PartialEq、Eq 的 derive 不在表里。
# 整 crate 验出 56 条 = 逐函数验出的 48 条 + 这 8 个 clone 各 1 条（推的：clone 那 8 条没有逐个量过）。
# 不同时 derive(Copy) 的也一样成项：Verus 对它们报「autoderive Clone impl when the clone is not a copy … without adding a specification」，不给规格，
# 函数体照验、各 1 条义务（0.2026.09.20 在探针 crate 上逐个 --verify-function 点：Copy 的、只有 u64 字段不带 Copy 的、
# 带 Vec 字段的、带 Vec 字段又标了 verifier::allow(autoderive_clone_without_spec) 的、带泛型参数的、不带 Copy 的枚举，<类型>::clone 都是 verified 1；整 crate 9 条 = 1 个 exec fn + 8 个 clone）。
DERIVED_VERIFIED_TRAITS = ("Clone",)
PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE = "proof fn", "exec fn", "带 decreases 的 spec fn", "derive(Clone) 生成的 clone"
# Verus 整 crate 验时另算条数的两类（Verus 0.2026.09.20）：
# verus! 里带值的 const（impl 里的关联 const 也算）各 1 条，可点名函数表里自由 const 列成它的名字、trait 里的列成 <trait>::<名>、
#   impl 里的（固有 impl 与 trait impl）列成 impl&%<号>::<名>（号数这个模块里的全部 impl：探针里 verus! 外面一个 impl、一个 derive(Clone, Copy) 的类型
#   排在前面，verus! 里第一个 impl 就是 impl&%4），按 *::<名> 点（valued_const_point_name；探针 crate 的根模块里关联 const FIRST 与自由 const FIRST_ROOT_COUNTER 同在，
#   点 *::FIRST 验出 1 条，点 FIRST 报 more than one match；同一个模块里两个 impl 各有一个 FIRST 时 *::FIRST 两个一起验、验出 2 条）；
# assume_specification 各 0 或 1 条：指向常量、写 returns 的 1 条（core 的 singlefs_format::SLOT_BYTES 点它验出 1 条，探针 core::u8::MAX 也是 1 条），
#   指向函数的（<[T]>::binary_search、u64::reverse_bits 写 returns）与指向常量写 ensures 的 0 条；可点名函数表里外部路径照写（singlefs_format::SLOT_BYTES），
#   std 的方法列成 core::slice::impl&%0::binary_search 这种，点最后一段 binary_search 点得中。
VALUED_CONST, ASSUME_SPECIFICATION = "带值的 const", "assume_specification"
PLAIN_PATH = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*")


class DeclarationSite(collections.namedtuple("DeclarationSite", "first_token first_line")):
    """assume_specification 那一项在核里的位置（它不是 lib_function_units 切出的单元）：排序用词法单元下标，列项用行号。"""


class ProofItem(collections.namedtuple("ProofItem", "item verus_name relative unit kind entry estimate")):
    """证明的一项：项名 proof:<verus_name>、交给 --verify-function 的名字、核的相对路径、单元、哪一类（PROOF_FN 这几样）、
    算键的入口（<核>::<单元名>：fn 是它自己，clone 是那个类型，带值的 const 是它自己，assume_specification 是 lib_function_units 切出的那一条单元，找不到交 None、那一项拒算）、
    静态估的义务条数（只给计数之和对不上时点名用，不判每一项）。assume_specification 那一项的 unit 是 DeclarationSite。"""


def leading_words(rust_file, unit, stop):
    """单元开头到 stop 那个词之前的词（外属性 #[…] 整段跳过）。"""
    words, index = [], unit.first_token
    while index < unit.last_token and rust_file.texts[index] != stop:
        if rust_file.texts[index] == "#" and index + 1 < rust_file.count and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
            index = rust_file.pairs[index + 1] + 1
            continue
        words.append(rust_file.texts[index])
        index += 1
    return words


def function_mode(rust_file, unit):
    """verus! 里一个 fn 的模式：spec、proof 或 exec（fn 前面没写模式的是 exec）。"""
    words = leading_words(rust_file, unit, "fn")
    return "spec" if "spec" in words else "proof" if "proof" in words else "exec"


def derived_traits(rust_file, unit):
    """类型单元开头的 #[derive(…)] 里点名的 trait。"""
    traits, index = [], unit.first_token
    while index + 1 < unit.last_token and rust_file.texts[index] == "#" and rust_file.texts[index + 1] == "[" and index + 1 in rust_file.pairs:
        close = rust_file.pairs[index + 1]
        if rust_file.texts[index + 2] == "derive":
            traits += [text for text in rust_file.texts[index + 3:close] if text not in ("(", ")", ",", ":")]
        index = close + 1
    return traits


def valued_const_entries(rust_file, relative, crate, module, ranges):
    """被验区域里带值的 const 各一项（VALUED_CONST，静态估 1 条）：自由的与 impl、trait 里的关联 const 都算，spec const 也算；没写值的（trait 里的声明）不成项
    （也不进「没成项的 verus! fn」那一份清单：那一份只列 fn）。
    spec const 照真 Verus 0.2026.09.20：自由的整 crate 验多 1 条
    （去掉它 19 条变 18 条），可点名函数表列成 <名>，点它验出 1 条；impl 里的整 crate 也多 1 条，列成 impl&%<号>::<名>，点 *::<名> 验出 1 条，与 impl 里的关联 const 一样；
    trait 里写 spec const Verus 报语法错（expected `!`），写不出来。所以 spec const 与别的带值的 const 同一种成项、同一种点名，没有点不了名、要另算进之和的。
    弄坏开关 spec-const-not-item 下 spec const 不成项（之和少 1）。
    交给 --verify-function 的照 valued_const_point_name（自由的写名字、trait 里的写 <trait>::<名>、impl 里的写 *::<名>）；项名的全路径是
    <crate>::<模块>::<类型或 trait>::<名>（自由 const 没有类型那一段）。算键的入口是 <核>::<单元名>，与 fn 同一种。弄坏开关 valued-const-not-item 下一项都不成。"""
    if "valued-const-not-item" in BREAK:
        return []
    entries = []
    for unit in rust_file.units:
        if unit.kind != "const" or unit.first_token is None or not in_verified_region(rust_file, unit, ranges):
            continue
        container = unit.container
        if container is not None and container.kind not in ("impl", "trait"):
            continue   # fn 里的 const 随外层验
        owner = None if container is None else container.short if container.kind == "trait" else impl_parts(container.name)[1]
        if container is not None and owner is None:
            raise Red(f"{relative}:{unit.first_line} 的 const {unit.short} 在 verus! 里的 impl 里，认不出 impl 的类型",
                      "impl 写成 impl <Trait> for <类型名> 或 impl <类型名>；确要别的写法，给 verus-proof-check.py 的 valued_const_entries 补上认法")
        shown = f"{owner}::{unit.short}" if owner else unit.short
        full = "::".join(part for part in (crate, module, shown) if part)
        words = rust_file.texts[unit.first_token:unit.last_token]
        if "=" not in words or ("spec-const-not-item" in BREAK and "spec" in leading_words(rust_file, unit, "const")):
            continue
        entries.append((unit.first_token, ProofItem(proof_item(full), valued_const_point_name(unit), relative, unit, VALUED_CONST, f"{relative}::{unit.name}", 1)))
    return entries


def valued_const_point_name(unit):
    """带值的 const 交给 --verify-function 的名字（Verus 0.2026.09.20 的可点名函数表）：自由 const 列成 <名>，照写；
    trait 里带默认值的列成 <trait>::<名>，照写；impl 里的（固有 impl 与 trait impl）列成 impl&%<号>::<名>，号数这个模块里的全部 impl（verus! 外面的、derive 生成的也数），
    从源码算不准，写 *::<名>（以 ::<名> 收尾的都点中、一起验；同一个模块里只有它一个以 ::<名> 收尾才点得准，point_name_problems 判）。
    <类型>::<名> 与带模块、带 crate 的全路径 Verus 都报 could not find。弄坏开关 associated-const-bare-name 下一律交 <名>（
    与以 <名> 起头的自由 const 同在一个模块时 Verus 报 more than one match）。"""
    container = unit.container
    if container is None or "associated-const-bare-name" in BREAK:
        return unit.short
    if container.kind == "trait":
        return f"{container.short}::{unit.short}"
    return f"*::{unit.short}"


def verus_listing(rust_file, relative, ranges):
    """一个核（一个模块）在 Verus 可点名函数表里的样子：[(列出的名字, 说明)]，说明写它在核里的位置。照 Verus 0.2026.09.20 列的：
    被验区域里的每个 fn（spec、proof、exec，trait 里的声明与 external_body 的也列）照 verus_point_name；verus! 里 derive(Clone) 的类型列 <类型>::clone；
    const（带不带值、spec 不 spec 都列）自由的列 <名>、trait 里的列 <trait>::<名>、impl 里的列 impl&%?::<名>（号算不准，写 ?：点名里不出现号，只按收尾比）；
    enum 变体的字段 Verus 另生成的 spec fn 照 enum_arrow_names 列；
    assume_specification 点名的条目只有标识符与 :: 的列它自己，别的列 core::impl&%?::<最后一个标识符>（真 Verus 列成 core::slice::impl&%0::binary_search 这种）。
    fn 里套的 fn 与 const 不列（随外层验）。"""
    listing = []
    for unit in rust_file.units:
        if unit.first_token is None or unit.container is not None and unit.container.kind not in ("impl", "trait"):
            continue
        if unit.kind == "fn" and in_verified_region(rust_file, unit, ranges):
            name = verus_point_name(unit)
            if name is not None:
                listing.append((name, f"fn {name}（{relative}:{unit.first_line}）"))
        elif unit.kind == "const" and in_verified_region(rust_file, unit, ranges):
            container = unit.container
            name = unit.short if container is None else f"{container.short}::{unit.short}" if container.kind == "trait" else f"impl&%?::{unit.short}"
            shown = unit.short if container is None else f"{container.short if container.kind == 'trait' else impl_parts(container.name)[1]}::{unit.short}"
            listing.append((name, f"const {shown}（{relative}:{unit.first_line}）"))
        elif unit.kind in ("struct", "enum", "union") and unit.container is None and any(low < unit.first_token < high for low, high in ranges):
            for trait in DERIVED_VERIFIED_TRAITS:
                if trait in derived_traits(rust_file, unit):
                    name = f"{unit.short}::{trait.lower()}"
                    listing.append((name, f"derive({trait}) {name}（{relative}:{unit.first_line}）"))
            if unit.kind == "enum":
                listing += [(name, f"enum 字段的 spec fn {name}（{relative}:{unit.first_line}）") for name in enum_arrow_names(rust_file, unit)]
    texts = rust_file.texts
    for low, high in region_spans(rust_file):
        for index in range(low, min(high + 1, rust_file.count)):
            if texts[index] == "assume_specification" and rust_file.kinds[index] == "ident":
                point, _closing, _end = assume_specification_parts(rust_file, index, high)
                identifiers = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", point)
                name = point if PLAIN_PATH.fullmatch(point) else f"core::impl&%?::{identifiers[-1] if identifiers else point}"
                listing.append((name, f"assume_specification[{point}]（{relative}:{rust_file.lines[index]}）"))
    return listing


def top_level_segments(rust_file, low, high):
    """词法单元 [low, high) 按同一层的 , 切成几段（配对的括号整段跳过，<…> 里的 , 不算；-> 的 > 不算收尾），交 [(段首, 段尾)]，空段不交。"""
    texts, segments, start, angle, index = rust_file.texts, [], low, 0, low
    while index < high:
        word = texts[index]
        if word in ("(", "[", "{") and index in rust_file.pairs:
            index = rust_file.pairs[index] + 1
            continue
        if word == "<":
            angle += 1
        elif word == ">" and angle and not (index > low and texts[index - 1] == "-"):
            angle -= 1
        elif word == "," and angle == 0:
            if index > start:
                segments.append((start, index))
            start = index + 1
        index += 1
    if high > start:
        segments.append((start, high))
    return segments


def enum_arrow_names(rust_file, unit):
    """verus! 里的 enum，Verus 给变体的每个字段生成一个 spec fn、列进可点名函数表（Verus 0.2026.09.20，block_device、logged_operations、
    pointer、records、verified_model 几个模块的表里都有）：<类型>::arrow_<变体>_<字段>（元组变体的字段写下标 0、1…），只在一个变体里出现的字段名另列 <类型>::arrow_<字段>。
    外属性 #[…] 与 pub、pub(…) 跳过；判别值（= <数>）不算字段。"""
    texts, pairs = rust_file.texts, rust_file.pairs
    keyword = next((index for index in range(unit.first_token, unit.last_token) if texts[index] == "enum"), None)
    opening = next((index for index in range(keyword, unit.last_token) if texts[index] == "{" and index in pairs), None) if keyword is not None else None
    if opening is None:
        return []
    variants = []
    for start, end in top_level_segments(rust_file, opening + 1, pairs[opening]):
        index = start
        while index < end and texts[index] == "#" and index + 1 < end and texts[index + 1] == "[" and index + 1 in pairs:
            index = pairs[index + 1] + 1
        if index >= end or rust_file.kinds[index] != "ident":
            continue
        variant, fields, body = texts[index], [], index + 1
        if body < end and texts[body] in ("(", "{") and body in pairs:
            for position, (field_start, field_end) in enumerate(top_level_segments(rust_file, body + 1, pairs[body])):
                if texts[body] == "(":
                    fields.append(str(position))
                    continue
                cursor = field_start
                while cursor < field_end and (texts[cursor] == "#" and cursor + 1 < field_end and texts[cursor + 1] == "[" and cursor + 1 in pairs):
                    cursor = pairs[cursor + 1] + 1
                if cursor < field_end and texts[cursor] == "pub":
                    cursor = pairs[cursor + 1] + 1 if cursor + 1 < field_end and texts[cursor + 1] == "(" and cursor + 1 in pairs else cursor + 1
                if cursor < field_end and rust_file.kinds[cursor] == "ident":
                    fields.append(texts[cursor])
        variants.append((variant, fields))
    seen = collections.Counter(field for _variant, fields in variants for field in set(fields))
    names = [f"{unit.short}::arrow_{variant}_{field}" for variant, fields in variants for field in fields]
    return names + [f"{unit.short}::arrow_{field}" for field in sorted(seen) if seen[field] == 1]


def verus_function_matches(argument, names):
    """照 Verus 0.2026.09.20 的 --verify-function 在可点名函数表 names 里点 argument，交点中的下标（帮助原文
    「matches on unique substring (foo) or wildcards at ends of the argument (*foo, foo*, *foo*)」）：
    *<串>* 是包含它、*<串> 是以它收尾、<串>* 是以它起头，带 * 的点中几个 Verus 就一起验几个；不带 * 的先找恰好同名的，没有再找包含它的
    （包含它的不止一个 Verus 报 more than one match，一个都没有报 could not find）。"""
    if len(argument) > 1 and argument.startswith("*") and argument.endswith("*"):
        return [index for index, name in enumerate(names) if argument[1:-1] in name]
    if argument.startswith("*"):
        return [index for index, name in enumerate(names) if name.endswith(argument[1:])]
    if argument.endswith("*"):
        return [index for index, name in enumerate(names) if name.startswith(argument[:-1])]
    exact = [index for index, name in enumerate(names) if name == argument]
    return exact or [index for index, name in enumerate(names) if argument in name]


def assume_specification_entries(rust_file, relative, crate, module):
    """被验区域里每一处 assume_specification 一项（ASSUME_SPECIFICATION）：项名 proof:<crate>::<模块>::assume_specification[<点名的条目>]，
    交给 --verify-function 的是点名的条目（是一段只有标识符与 :: 的路径时照写，例 singlefs_format::SLOT_BYTES；别的写法取最后一个标识符，例 binary_search）。
    静态估：指向常量（] 后面不是参数表）而写了 returns 的 1 条，别的 0 条；这一类验出 0 条也是绿（指向函数的规格没有义务）。
    算键的入口是 lib_function_units 切出的那个 assume_specification 单元（<核>::assume_specification@<关键字那一行>，单元从属性起、含这个词）；
    找不到的交 None，那一项拒算（check_proof 里的 assume_specification_components）。弄坏开关 assume-specification-not-item 下一项都不成。"""
    if "assume-specification-not-item" in BREAK:
        return []
    entries = []
    texts = rust_file.texts
    declared = [unit for unit in rust_file.units if unit.kind == "assume_specification" and unit.first_token is not None]
    for low, high in region_spans(rust_file):
        for index in range(low, min(high + 1, rust_file.count)):
            if texts[index] != "assume_specification" or rust_file.kinds[index] != "ident":
                continue
            point, closing, end = assume_specification_parts(rust_file, index, high)
            identifiers = re.findall(r"[A-Za-z_][A-Za-z0-9_]*", point)
            verus_name = point if PLAIN_PATH.fullmatch(point) else (identifiers[-1] if identifiers else point)
            to_constant = closing is not None and closing + 1 < rust_file.count and texts[closing + 1] != "("
            estimate = 1 if to_constant and "returns" in texts[index:end] else 0
            full = "::".join(part for part in (crate, module, f"assume_specification[{point}]") if part)
            owners = [unit for unit in declared if unit.first_token <= index < unit.last_token]
            entries.append((index, ProofItem(proof_item(full), verus_name, relative, DeclarationSite(index, rust_file.lines[index]), ASSUME_SPECIFICATION,
                                             f"{relative}::{owners[0].name}" if len(owners) == 1 else None, estimate)))
    return entries


def owner_function(functions, relative, line):
    """核 relative 第 line 行落在哪个被验区域里的 fn 单元里，交那个 VerusFunction；不在任何一个里交 None。"""
    for function in functions:
        if function.relative == relative and function.unit.first_line <= line <= function.unit.last_line:
            return function
    return None


def crate_build_text(root, registry):
    """这个 crate 怎么交给 Verus 编（键里「crate 起法」那一样）：起法（crate 行第四段 cargo 的经 cargo verus focus，不写的直接起 verus）、
    Cargo.toml 的 edition 与包名（直接起的那一种照它们拼 --edition、--crate-name；cargo 那一种由 cargo 照它们编）。edition 写 workspace = true 的取仓根 [workspace.package]。"""
    package = crate_manifest(root, registry).get("package", {})
    edition = package.get("edition", "未写")
    if isinstance(edition, dict) and edition.get("workspace"):
        try:
            with open(os.path.join(root, "Cargo.toml"), "rb") as handle:
                edition = ((tomllib.load(handle).get("workspace") or {}).get("package") or {}).get("edition", "未写")
        except (OSError, tomllib.TOMLDecodeError):
            edition = "仓根 Cargo.toml 读不出"
    return f"mode={'cargo' if registry.cargo_mode else 'verus'}\tedition={edition}\tname={package.get('name', '未写')}"


_MUTATION_UNITS = []


def mutation_units_module():
    """scripts/mutation/lib_mutation_units.py（闭包真点名的外部 crate 落到 Cargo.lock 的哪几个包，与 crates 变异的行键同一份 dependency_lines），第一次用到时才读进来。"""
    if not _MUTATION_UNITS:
        import importlib.util
        specification = importlib.util.spec_from_file_location("lib_mutation_units_for_verus_proof_check", os.path.join(SCRIPT_DIRECTORY, "..", "mutation", "lib_mutation_units.py"))
        module = importlib.util.module_from_spec(specification)
        sys.modules[specification.name] = module
        specification.loader.exec_module(module)
        _MUTATION_UNITS.append(module)
    return _MUTATION_UNITS[0]


def closure_dependency_lines(root, registry, options, unit_key):
    """键里「依赖版本」那一样：单元闭包（unit_key 的组成清单）里各单元真点名的外部 crate 在 Cargo.lock 里落到的包与传递依赖，一包一行
    （lib_mutation_units.py 的 dependency_lines，口径与 crates 变异的行键相同）。点名了而 Cargo.lock 里落不到的（拒算），退回这个 crate 在 Cargo.lock 里
    传递引到的全部外部包（lib_verdict_library.py 的 locked_dependency_lines，只多作废、不少）。每个单元的那几行按单元存进 options，几项共用。"""
    mutation_units = mutation_units_module()
    workspace = unit_index(root, options).workspace()
    cache = options.setdefault("unit_dependency_lines", {})
    try:
        if options.get("lock_packages") is None:
            options["lock_packages"] = mutation_units.read_lock_packages(root)
    except mutation_units.MutationUnitsError:
        return ["按整个 crate 收（Cargo.lock 读不懂）"] + lib_verdict_library.locked_dependency_lines(root, crate_package_name(registry))
    lines, refused = set(), False
    for text in unit_key.units:
        kind, name, location = text.split("\t")[:3]
        path, _colon, span = location.rpartition(":")
        if not path.endswith(".rs"):
            continue
        if (path, kind, name, span) not in cache:
            try:
                found = [unit for unit in workspace.file(path).units if unit.kind == kind and unit.name == name and f"{unit.first_line}-{unit.last_line}" == span]
            except (OSError, ValueError):
                found = []
            cache[(path, kind, name, span)] = mutation_units.dependency_lines(workspace, found, options["lock_packages"])
        unit_lines, unit_refusals = cache[(path, kind, name, span)]
        lines |= set(unit_lines)
        refused = refused or bool(unit_refusals)
    if refused:
        return ["按整个 crate 收（闭包里有点名的外部 crate 在 Cargo.lock 里落不到）"] + lib_verdict_library.locked_dependency_lines(root, crate_package_name(registry))
    return sorted(lines)


def dependency_component(root, registry, options, unit_key):
    """「依赖版本」那一样（closure_dependency_lines）；单元闭包拒算的不算（那一项本来就每趟现判）。弄坏开关 reuse-key-without-closure-dependencies：不进。"""
    if "reuse-key-without-closure-dependencies" in BREAK or not isinstance(unit_key, UnitKey) or unit_key.key is None:
        return []
    return [("依赖版本", digest_text("\n".join(closure_dependency_lines(root, registry, options, unit_key))))]


def function_components(root, registry, options, entries, label):
    """被验函数那一样：entries（<核>::<fn 名>）的单元闭包键（UnitKey：那个 fn 单元，连同它 requires / ensures / decreases 与函数体里用到的
    spec fn、proof fn、类型、常量、broadcast use 点名的引理，lib_function_units.py 的八条闭包规则）。没给判定库交空表。
    弄坏开关：reuse-key-whole-crate 换回核 crate 整个目录的原文摘要；reuse-key-function-only 只取入口那几个单元自己的指纹；
    reuse-key-without-units 这一样不进。"""
    if not options.get("verdict_library") or "reuse-key-without-units" in BREAK:
        return []
    if "reuse-key-whole-crate" in BREAK:
        return [("核 crate", digest_tree(os.path.join(root, registry.crate_directory)))]
    try:
        key = unit_index(root, options).key(entries)
    except ValueError as error:
        raise Red(f"算不出 {'、'.join(entries)} 的单元闭包：{error}", "先让 python3 scripts/gate/lib_function_units.py closure <根> <入口> 在这棵树上跑通")
    if "reuse-key-function-only" in BREAK:
        names = {entry.rsplit("::", 1)[1] for entry in entries}
        own = tuple(line for line in key.units if line.split("\t")[1] in names)
        return [(label, UnitKey(digest_text("\n".join(line.split("\t")[0] + "\t" + line.split("\t")[3] for line in own)), own, ()))]
    return [(label, key)] + dependency_component(root, registry, options, key)


def entry_unit_component(rust_unit, label):
    """这一项自己是闭包里的哪一个单元（键里「被验入口单元」「所属函数单元」那一样）：入口单元连同包它的 impl / trait 头一层层往外，各取（种类，去注释指纹），
    不带名字、路径与行号。闭包是单元的多重集，同一个 impl 里几个方法经 R-结构、R-隐式互相收进对方的闭包，闭包一模一样，没有这一样就几项共用一把键、
    一份判定（core 的 impl BlockDevice for FileBackedBlockDevice 八个方法共用一把，复用时 write_zeroes_at 拿到 size_in_bytes 那一份的 1 条）。
    弄坏开关 reuse-key-without-entry-unit：不进。"""
    if "reuse-key-without-entry-unit" in BREAK or rust_unit is None:
        return []
    lines, owner = [], rust_unit
    while owner is not None:
        lines.append(f"{owner.kind}\t{owner.fingerprint}")
        owner = owner.container
    return [(label, digest_text("\n".join(lines)))]


def proof_entry_unit(root, options, entry):
    """证明一项算键的入口单元：fn、clone 的类型与带值的 const 是 ProofItem 的 unit 本身；assume_specification 那一项的 unit 是 DeclarationSite，
    按 entry（<核>::<单元名>）到核里找那个单元，找不到交 None（那一项本来就拒算）。"""
    if entry.kind != ASSUME_SPECIFICATION:
        return entry.unit
    if entry.entry is None:
        return None
    name = entry.entry[len(entry.relative) + 2:]
    found = [unit for unit in unit_index(root, options).workspace().file(entry.relative).units if unit.name == name]
    return found[0] if len(found) == 1 else None


ASSUME_TARGET_KINDS = ("const", "static", "fn")


def assume_specification_components(root, registry, options, entry):
    """assume_specification 那一项的被验函数那一样：入口是这一条 assume_specification 单元（lib_function_units 切出的，assume_specification_entries 交的 entry），
    闭包经 R-路径跨包收到它点名的那个条目（工作区包 singlefs_format 里的常量连同它引到的单元），与 fn 同一种算法、同样带依赖版本；它验出几条取决于那个条目。
    找不到这一条的单元、或闭包里没有点名的条目（种类是 const、static、fn，限定名最后一段是点名的最后一个标识符；std 的方法这类不在工作区里的）拒算：
    这一项照判、不复用，拒算行点名是哪一处。弄坏开关：assume-specification-refused 下一律拒算；assume-specification-target-unkeyed 下
    点名的条目不进键（改它的值照旧复用）。没给判定库交空表。"""
    place = f"{entry.relative}:{entry.unit.first_line}\t{entry.verus_name}"
    if "assume-specification-refused" in BREAK or entry.entry is None:
        return [("被验函数单元闭包", UnitKey(None, (), (f"refused\tassume-specification\t{place}",)))]
    components = function_components(root, registry, options, [entry.entry], "被验函数单元闭包")
    keyed = dict(components).get("被验函数单元闭包")
    if not isinstance(keyed, UnitKey) or keyed.key is None:
        return components
    target = (RUST_IDENTIFIER.findall(entry.verus_name) or [entry.verus_name])[-1]
    pointed = [line.split("\t") for line in keyed.units]
    pointed = [fields for fields in pointed if fields[0] in ASSUME_TARGET_KINDS and fields[1].rsplit("::", 1)[-1] == target]
    if not pointed:
        return [("被验函数单元闭包", UnitKey(None, keyed.units, (f"refused\tassume-specification-target\t{place}",)))]
    if "assume-specification-target-unkeyed" in BREAK:
        excluded = [f"{fields[2].rsplit(':', 1)[0]}::{fields[1]}" for fields in pointed]
        reduced = unit_index(root, options).key([entry.entry], excluded_units=excluded)
        return [("被验函数单元闭包", reduced)] + dependency_component(root, registry, options, reduced)
    return components


FORMAL_LIBRARY_DIRECTORY_NAME = "singlefs-formal-unit-verdicts"


def formal_library_directory(root):
    """门禁 formal-verification 的判定库目录（文件头「逐项点名与按条复用」的库位置），只由被判那棵仓定：被判那棵树的 git common-dir 底下的
    singlefs-formal-unit-verdicts/（共用库 lib_verdict_library.py 的 library_directory）；不是 git 工作树退 2（共用库的 LibraryUnavailable）。"""
    return lib_verdict_library.library_directory(root, FORMAL_LIBRARY_DIRECTORY_NAME)


FAKE_TOOLCHAIN = "1.98.1-x86_64-unknown-linux-gnu"   # 假安装 version.json 里 toolchain 那一段（门禁样本里没有 rustc 登记行时不比它）


def library_directory_command(root):
    """library-directory <根>：stdout 打判定库目录（门禁定库位置读它）；不是 git 工作树退 2（stderr 说为什么）。"""
    print(formal_library_directory(os.path.abspath(root)))
    return EXIT_GREEN


PLAN_STATES = ("键对不上", "无记录", "键相同", "拒算")


def peek(store, item, components):
    """只数键（list-items 给了判定库）：这一项照现在的键在库里是哪一种，不写库：键相同（会复用）、键对不上（这一项判绿过，键变了）、无记录、拒算。"""
    if refused_component(components, store.judging_units) is not None:
        return "拒算"
    # 键文件与 items/ 那一份经判定库读（prune-server 里那一份：scripts/crash/lib_verdict_library.py 文件头「存放」；服务不可用客户端退 75），不写库
    if "key-mismatch-by-item-only" in BREAK:   # 弄坏开关：这一项判绿过就算键相同，不比键
        return "键相同" if store.item_record(item) is not None else "无记录"
    if store.key_exists(key_of(components, store.judging_units)):
        return "键相同"
    return "键对不上" if store.item_record(item) is not None else "无记录"


def plan_done(options, cell, store):
    """只数键的那一趟：这一格各项的键都比过了，记下各几条，不起 Verus。"""
    options["plan"].append((cell, store.plan))
    if options.get("plan_items") is not None:
        options["plan_items"].extend(store.plan_items)
    return EXIT_GREEN


def lookup(store, item, components):
    if getattr(store, "plan", None) is not None:
        state = peek(store, item, components)
        store.plan[state] += 1
        store.plan_items.append((item, state))
        return False, "只数键"
    if "reuse-ignores-key" in BREAK:
        return store.lookup_by_item_only(item)
    return store.lookup(item, components)


def judged_how(store, item, reason, sentinel=None):
    """现判那一行的括号：原因，连同判定库报的变了的单元（作废时点名是哪几个单元变了）；复用哨兵抽中强制现判的那一项写「强制现判」。"""
    if sentinel is not None and sentinel.forced(item):
        return f"{reuse_checks.FORCED}（复用哨兵抽中：键相同，这一趟不读库）"
    detail = store.detail(item)
    return f"现判（{reason}{'：' + detail if detail else ''}）"


def plan_sentinel(options, store, kind, cell, entries):
    """复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：entries 是 [(项, 组成)]，候选是此刻键相同、库里有判绿记录的项
    （lib_reuse_checks.verdict_library_candidate 按键文件查，与 lookup 分开）；只数键的那一趟（list-items 给了库）不抽。"""
    sentinel = reuse_checks.ReuseSentinel(SCRIPT_KEY, cell)
    if options.get("plan") is not None:
        return sentinel
    candidates = {item: "绿" for item, components in entries
                  if components and reuse_checks.verdict_library_candidate(options.get("verdict_library"), kind, components, store.judging_units)}
    sentinel.plan(candidates)
    return sentinel


def sentinel_lookup(store, sentinel, item, components):
    """抽中强制现判的那一项不读库；别的照 lookup。弄坏开关 reuse-lookup-skipped 下一律不读库（复用哨兵该判红「复用失灵」）。"""
    if sentinel.forced(item):
        return False, "复用哨兵抽中强制现判"
    if "reuse-lookup-skipped" in BREAK:
        return False, "无记录（弄坏开关 reuse-lookup-skipped：没读库）"
    return lookup(store, item, components)


def selected_items(options, available, cell):
    """--item 点名的项（给几次取并集），按 available 的次序交回；没给交 available；点了没有的抛 UsageError。弄坏开关 item-filter-ignored 下一律交 available。"""
    wanted = options.get("items") or []
    if not wanted or "item-filter-ignored" in BREAK:
        return list(available)
    unknown = [item for item in wanted if item not in available]
    if unknown:
        raise UsageError(f"{cell} 格里没有点名的项：{'、'.join(unknown)}", available)
    return [item for item in available if item in wanted]


def record_item_done(item, seconds, judged, reused, exit_code):
    """环境里有 FORMAL_ITEM_DONE_FILE 的（门禁给的）往那份文件追加一行「项<TAB>秒<TAB>现判条数<TAB>复用条数<TAB>退出码」，一次 write 写完一整行（几处同时追加不拧）。
    弄坏开关 item-done-unrecorded 下不写。"""
    path = os.environ.get(ITEM_DONE_FILE_VARIABLE)
    if not path or "item-done-unrecorded" in BREAK:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"{item}\t{seconds:.1f}\t{judged}\t{reused}\t{exit_code}\n")


def report_item(item, verdict, how, seconds=None):
    """每项一行：项名、判定（绿 / 红）、现判还是复用、现判的用了几秒。同时记一行 ITEM_DONE 的原料（record_item_done）：复用的 0 秒、复用 1 条；
    现判的取这一项自己的秒数（verify_in_parallel 里这一处从起到出结局的挂钟，不是整格的）、现判 1 条；退出码绿 0、红 1。"""
    print(f"  · {item}：{verdict}，{how}" + (f"，{seconds:.1f} 秒" if seconds is not None else ""))  # gate-lint:detail
    reused = how.startswith("复用")
    record_item_done(item, 0.0 if reused else (seconds or 0.0), 0 if reused else 1, 1 if reused else 0, 0 if verdict == "绿" else 1)


def cell_tally(options):
    """这一格（登记了几个 crate 时是这个 crate 那一趟）的现判、复用数；几个 crate 一格时记进 options["crate_tallies"]，由 run_per_crate 合起来报一次。"""
    tally = Tally()
    if options.get("crate_tallies") is not None:
        options["crate_tallies"].append(tally)
    return tally


def finish_tally(tally, cell, options=None):
    """一格判完报「按条：现判 N、复用 M」，并往门禁给的 FORMAL_ITEM_TALLY_FILE 追加一行；几个 crate 一格时只报这个 crate 的、不追加（run_per_crate 合起来追加一行）。"""
    if options is not None and options.get("crate_tallies") is not None:
        print(f"  · 按条（{options['crate_view'].crate_directory}）：{tally.text()}")  # gate-lint:detail
        return
    print(f"  · 按条：{tally.text()}")  # gate-lint:detail
    tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), cell)


def merged_tally(tallies):
    total = Tally()
    for tally in tallies:
        total.reused += tally.reused
        for reason, count in tally.reasons.items():
            total.reasons[reason] = total.reasons.get(reason, 0) + count
    return total


def proof_item(name):
    return f"proof:{name}"


def results_text(results):
    """JSON 的 verification-results 照原样报几样：verified、errors、encountered-error、vir 错，有 success 键时另报它（--verify-function 下 Verus 不给）。"""
    def shown(value):
        return "没有这个键" if value is None else json.dumps(value)
    pieces = [f"verified {shown(results.get('verified'))}", f"errors {shown(results.get('errors'))}",
              f"encountered-error {shown(results.get('encountered-error'))}", f"vir 错 {shown(results.get('encountered-vir-error'))}"]
    if "success" in results:
        pieces.append(f"success {shown(results.get('success'))}")
    return "、".join(pieces)


def verification_clean(results, function_mode):
    """Verus 这一次验过了、一条错都没有：没有 vir 错、encountered-error 不是 true、errors 是整数 0，有 success 键时它是 true。
    --verify-function 下真 Verus（0.2026.09.20）不给 success 键、is-verifying-entire-crate 报 false（整 crate 那一趟
    给 success true 与 is-verifying-entire-crate true，逐函数那几趟只有 encountered-error、encountered-vir-error、verified、errors、is-verifying-entire-crate）：
    两样都不当成没验过。弄坏开关：require-success-key-per-function 下没有 success 键当没验过，
    require-entire-crate-per-function 下按函数验也要 is-verifying-entire-crate 是 true。"""
    if results.get("encountered-vir-error") or results.get("encountered-error"):
        return False
    errors = results.get("errors")
    if not isinstance(errors, int) or isinstance(errors, bool) or errors != 0:
        return False
    if "success" in results:
        if results["success"] is not True:
            return False
    elif "require-success-key-per-function" in BREAK:
        return False
    if function_mode and "require-entire-crate-per-function" in BREAK and results.get("is-verifying-entire-crate") is not True:
        return False
    return True


def item_kinds_text(items):
    """成项的几类各几个，次序固定：proof fn、exec fn、带 decreases 的 spec fn、derive(Clone) 生成的 clone、带值的 const、assume_specification
    （弄坏开关 spec-fn-as-item 下另有 spec fn）。"""
    kinds = collections.Counter(entry.kind for entry in items)
    return "、".join(f"{kind} {kinds[kind]}" for kind in (PROOF_FN, EXEC_FN, SPEC_FN_WITH_DECREASES, DERIVED_CLONE, VALUED_CONST, ASSUME_SPECIFICATION, "spec fn")
                    if kinds[kind])


def listed_names(names, limit=20):
    """点名用：至多 limit 个名字，多的报个数。"""
    names = list(names)
    return "、".join(names[:limit]) + (f" 等 {len(names)} 个" if len(names) > limit else "") if names else "没有"
