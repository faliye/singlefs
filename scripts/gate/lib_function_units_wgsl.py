"""WGSL 的切单元（WgslFile）与闭包（wgsl_closure）。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import os
import posixpath
import re
from lib_function_units_lexer import IDENT, match_brackets, rust_lex
from lib_function_units_base import FunctionUnitsError, Refusal, Unit, files_under, sha256_text, snippet_of
from lib_function_units_rust_workspace import discover_rust_crates


# ── WGSL ────────────────────────────────────────────────────────────────────

WGSL_PREDECLARED = frozenset("""
fn let var const override struct alias return if else loop for while break continue continuing switch case default discard true false
bitcast enable requires diagnostic const_assert
bool i32 u32 f32 f16 vec2 vec3 vec4 vec2i vec3i vec4i vec2u vec3u vec4u vec2f vec3f vec4f vec2h vec3h vec4h
mat2x2 mat2x3 mat2x4 mat3x2 mat3x3 mat3x4 mat4x2 mat4x3 mat4x4 mat2x2f mat3x3f mat4x4f array atomic ptr sampler sampler_comparison
texture_1d texture_2d texture_2d_array texture_3d texture_cube texture_cube_array texture_multisampled_2d texture_storage_1d
texture_storage_2d texture_storage_2d_array texture_storage_3d texture_depth_2d texture_depth_2d_array texture_depth_cube
texture_depth_cube_array texture_depth_multisampled_2d texture_external
function private workgroup uniform storage handle read write read_write
group binding compute workgroup_size vertex fragment builtin location interpolate invariant align size id must_use blend_src
global_invocation_id local_invocation_id local_invocation_index workgroup_id num_workgroups position vertex_index instance_index
front_facing frag_depth sample_index sample_mask subgroup_invocation_id subgroup_size
abs acos acosh all any arrayLength asin asinh atan atan2 atanh ceil clamp cos cosh countLeadingZeros countOneBits countTrailingZeros
cross degrees determinant distance dot dot4U8Packed dot4I8Packed exp exp2 extractBits faceForward firstLeadingBit firstTrailingBit
floor fma fract frexp insertBits inverseSqrt ldexp length log log2 max min mix modf normalize pow quantizeToF16 radians reflect refract
reverseBits round saturate select sign sin sinh smoothstep sqrt step tan tanh transpose trunc
atomicLoad atomicStore atomicAdd atomicSub atomicMax atomicMin atomicAnd atomicOr atomicXor atomicExchange atomicCompareExchangeWeak
workgroupBarrier storageBarrier textureBarrier workgroupUniformLoad
pack4x8snorm pack4x8unorm pack2x16snorm pack2x16unorm pack2x16float unpack4x8snorm unpack4x8unorm unpack2x16snorm unpack2x16unorm
unpack2x16float pack4xI8 pack4xU8 unpack4xI8 unpack4xU8
textureDimensions textureGather textureLoad textureNumLayers textureNumLevels textureNumSamples textureSample textureSampleLevel
textureStore dpdx dpdy fwidth
subgroupAdd subgroupAll subgroupAny subgroupBallot subgroupBroadcast subgroupMax subgroupMin
off on error warning info derivative_uniformity
_
""".split())


class WgslFile:
    def __init__(self, path, text):
        self.path = path
        self.source_rows = text.split("\n")
        self.kinds, self.texts, self.lines, _documents = rust_lex(text)
        self.count = len(self.texts)
        self.units, self.refusals = [], []
        self.pairs, bad = match_brackets(self.kinds, self.texts)
        if self.pairs is None:
            line = self.lines[bad] if bad is not None and bad < self.count else 1
            self.refusals.append(Refusal("unbalanced-brackets", path, line, self.row(line)))
            self.pairs = {}
            return
        self.scan()

    def row(self, line):
        return snippet_of(self.source_rows[line - 1]) if 0 < line <= len(self.source_rows) else ""

    def t(self, index):
        return self.texts[index] if 0 <= index < self.count else ""

    def kind(self, index):
        return self.kinds[index] if 0 <= index < self.count else ""

    def statement_end(self, i):
        c = i
        while c < self.count:
            text = self.t(c)
            if text in ("(", "[", "{") and c in self.pairs:
                c = self.pairs[c] + 1
                continue
            if text == ";":
                return c + 1
            c += 1
        return self.count

    def add(self, kind, name, start, end):
        end = max(end, start + 1)
        unit = Unit("wgsl", kind, name, self.path, self.lines[start], self.lines[min(end, self.count) - 1],
                    sha256_text("\x1f".join(self.texts[start:end])))
        unit.first_token, unit.last_token = start, min(end, self.count)
        plain, declared, name_lines = set(), set(), {}
        for index in range(start, unit.last_token):
            if self.kinds[index] != IDENT:
                continue
            text = self.texts[index]
            previous = self.t(index - 1)
            if previous in (".", "@"):
                continue
            if previous == "(" and self.t(index - 2) in ("builtin", "interpolate", "diagnostic"):
                continue
            if self.t(index + 1) == ":" and self.t(index + 2) != ":":
                declared.add(text)
            if previous in ("let", "var", "const", "override") or (previous == ">" and self.t(index - 2) != "-" and self.var_head(index)):
                declared.add(text)
            plain.add(text)
            name_lines.setdefault(text, self.lines[index])
        unit.plain, unit.declared, unit.cache = plain, declared, name_lines
        self.units.append(unit)
        return unit

    def var_head(self, index):
        """index 前面是不是 var<…> 的那个 >。"""
        j = index - 1
        depth = 0
        while j >= 0:
            text = self.t(j)
            if text == ">":
                depth += 1
            elif text == "<":
                depth -= 1
                if depth == 0:
                    return self.t(j - 1) == "var"
            elif text in (";", "{", "}"):
                return False
            j -= 1
        return False

    def scan(self):
        i = 0
        while i < self.count:
            start = i
            attributes = []
            while self.t(i) == "@" and self.kind(i + 1) == IDENT:
                attributes.append(self.t(i + 1))
                i += 2
                if self.t(i) == "(" and i in self.pairs:
                    i = self.pairs[i] + 1
            if i >= self.count:
                break
            word = self.t(i)
            line = self.lines[i]
            if word == ";":
                i += 1
                continue
            if word == "fn" and self.kind(i + 1) == IDENT:
                c, end = i + 2, self.count
                while c < self.count:
                    if self.t(c) == "(" and c in self.pairs:
                        c = self.pairs[c] + 1
                        continue
                    if self.t(c) == "{" and c in self.pairs:
                        end = self.pairs[c] + 1
                        break
                    c += 1
                # 只有 @compute 的是入口（wgsl_compute，与 crates/singlefs-unit-graph 同一种认法），@vertex / @fragment 照普通 fn
                kind = "wgsl_compute" if "compute" in attributes else "wgsl_fn"
                self.add(kind, self.t(i + 1), start, end)
                i = end
                continue
            if word in ("const", "override", "alias") and self.kind(i + 1) == IDENT:
                end = self.statement_end(i)
                self.add(f"wgsl_{word}", self.t(i + 1), start, end)
                i = end
                continue
            if word == "var":
                j = i + 1
                if self.t(j) == "<":
                    depth = 0
                    while j < self.count:
                        if self.t(j) == "<":
                            depth += 1
                        elif self.t(j) == ">":
                            depth -= 1
                            if depth == 0:
                                break
                        j += 1
                    j += 1
                end = self.statement_end(i)
                self.add("wgsl_var", self.t(j), start, end)
                i = end
                continue
            if word == "struct" and self.kind(i + 1) == IDENT:
                c, end = i + 2, self.count
                while c < self.count:
                    if self.t(c) == "{" and c in self.pairs:
                        end = self.pairs[c] + 1
                        break
                    c += 1
                if self.t(end) == ";":
                    end += 1
                self.add("wgsl_struct", self.t(i + 1), start, end)
                i = end
                continue
            if word in ("enable", "requires", "diagnostic"):
                end = self.statement_end(i)
                self.add("wgsl_directive", f"{word}@{line}", start, end)
                i = end
                continue
            if word == "const_assert":
                end = self.statement_end(i)
                self.add("wgsl_const_assert", f"const_assert@{line}", start, end)
                i = end
                continue
            end = self.statement_end(i)
            self.refusals.append(Refusal("unknown-item", self.path, line, self.row(line)))
            i = max(end, i + 1)


WGSL_FILES = {}
WGSL_TOP_LEVEL_DEFINING_KINDS = frozenset(("wgsl_fn", "wgsl_compute", "wgsl_const", "wgsl_override", "wgsl_alias", "wgsl_var", "wgsl_struct"))


def wgsl_top_level_names_in(root, path):
    """一份 .wgsl 顶层条目定义的名字（fn、const、override、alias、var、struct）；读不了的交空集（读它的那一处另由 include 那一条拒算）。"""
    try:
        item = wgsl_file(root, path)
    except OSError:
        return set()
    return {unit.short for unit in item.units if unit.kind in WGSL_TOP_LEVEL_DEFINING_KINDS}


def wgsl_file(root, path):
    absolute = os.path.join(root, path)
    with open(absolute, encoding="utf-8", errors="surrogateescape") as handle:
        text = handle.read()
    key = (absolute, sha256_text(text))
    if key not in WGSL_FILES:
        WGSL_FILES[key] = WgslFile(path, text)
        # 输出行里的限定名照 crates/singlefs-unit-graph：<所在 crate 名>::<文件名>::<名字>
        identifier = crate_identifier_of(root, path)
        for unit in WGSL_FILES[key].units:
            unit.full = f"{identifier}::{posixpath.basename(path)}::{unit.name}" if identifier else None
    return WGSL_FILES[key]


CRATE_IDENTIFIERS = {}


def crate_identifier_of(root, path):
    """path 所在 crate（工作区成员里目录是它最长前缀的那个）的 crate 名：[lib] name，没有就包名把 - 换成 _；不在哪个 crate 里交 None。"""
    real_root = os.path.realpath(root)
    if real_root not in CRATE_IDENTIFIERS:
        try:
            CRATE_IDENTIFIERS[real_root] = [(crate.directory, crate.lib_name) for crate in discover_rust_crates(root)]
        except FunctionUnitsError:
            CRATE_IDENTIFIERS[real_root] = []
    best = None
    for directory, identifier in CRATE_IDENTIFIERS[real_root]:
        prefix = directory + "/" if directory else ""
        if path.startswith(prefix) and (best is None or len(directory) > len(best[0])):
            best = (directory, identifier)
    return best[1] if best else None


def literal_mentions(literal_texts, prefixes, name):
    return any(name in text for text in literal_texts) or any(name.startswith(prefix) for prefix in prefixes)


def literal_prefixes(literal_texts):
    prefixes = set()
    for text in literal_texts:
        for found in re.finditer(r"([A-Za-z_][A-Za-z0-9_]*)\{", text):
            if len(found.group(1)) >= 2:
                prefixes.add(found.group(1))
    return prefixes


def wgsl_closure(wgsl_files, entry_units=None, sources=None):
    """一个 WGSL 模块（几份 .wgsl 拼成的）从入口按名字精确收；没有入口就收全部。没定义的名字（外名）要在 sources（WgslNameSources）里有着落，
    没有着落的拒算；sources 是 None（没给 Rust 入口）时一律拒算。→ (成员, 拒算, 给外名着落的 Rust 单元)。"""
    by_name = collections.defaultdict(list)
    all_units, refusals = [], []
    for item in wgsl_files:
        refusals += item.refusals
        for unit in item.units:
            by_name[unit.short].append(unit)
            all_units.append(unit)
    starts = list(entry_units) if entry_units else ([unit for unit in all_units if unit.kind == "wgsl_compute"] or list(all_units))
    starts += [unit for unit in all_units if unit.kind in ("wgsl_directive", "wgsl_const_assert")]
    declared = set()
    for unit in all_units:
        declared |= unit.declared
    seen, order, undefined = set(), [], {}
    todo = list(starts)
    while todo:
        unit = todo.pop()
        if id(unit) in seen:
            continue
        seen.add(id(unit))
        order.append(unit)
        for name in unit.plain:
            if name in by_name:
                todo.extend(by_name[name])
            elif name not in WGSL_PREDECLARED and name not in declared and name not in undefined:
                undefined[name] = (unit.file, unit.cache.get(name, unit.first_line))
    suppliers = {}
    for name in sorted(undefined):
        if sources is not None:
            supplied, units = sources.suppliers(name)
            if supplied:
                for unit in units:
                    suppliers[id(unit)] = unit
                continue
        file, line = undefined[name]
        refusals.append(Refusal("wgsl-undefined-name", file, line, name))
    return order, refusals, list(suppliers.values())


def wgsl_units(root):
    out = []
    for crate in discover_rust_crates(root):
        for path in files_under(root, (".wgsl",), [crate.directory or "."]):
            out.append(wgsl_file(root, path))
    return out
