"""纯挪动检查的判法（crates-split-pure-move.py 的 pure-move 子命令）：拆前、拆后两棵树里每个包的单元多重集对比。
单元与去注释的指纹照 lib_function_units.py（RustFile、WgslFile、discover_rust_crates），可见性修饰（pub、pub(crate)、pub(super)、pub(self)、pub(in …)）
从指纹的词法单元里剥掉再比；不起编译，不起别的进程（读文件的并行用 fork 出来的工作进程）。

单元怎么认作「同一个」：（种类，名字，剥掉可见性之后的指纹）三样都相同。名字取单元在它那份文件里的嵌套路径，去掉行内模块那一截前缀
（行内模块挪成文件模块，里面的单元名字不变），名字里 @<行号> 与文档测试的 <doctest line 行号> 的行号折掉（use、条目级宏调用、extern 块挪了位置行号会变）。
多重集：同一个键在拆前几个、拆后就要几个。键相同配不上、而 rustfmt 排版抹平（lib_crates_split_layout.py）之后相同的单元配成一对，单列成「只差 rustfmt 排版」，不算差异。
拆前拆后逐字节相同的文件里的单元不进配对（按文件内容的 sha256 判）：配不上的余数落在哪份文件是配对次序定的，不排除就会记到没碰过的文件上。
拆后新加的 mod 声明与行内 mod 头（mod_head，不含文件开头内属性那个 <module attributes>）是脚手架，按它们所在的被拆文件族（族根见 family_root）列出。
"""
import collections
import concurrent.futures
import multiprocessing
import os
import re

import lib_crates_split_common as common
import lib_crates_split_layout as rustfmt_layout
import lib_function_units as function_units

broken = common.broken

RUST_SUFFIX = ".rs"
WGSL_SUFFIX = ".wgsl"
SOURCE_SUFFIXES = (RUST_SUFFIX, WGSL_SUFFIX)
LINE_NUMBER_SUFFIX = re.compile(r"@\d+$")
DOCTEST_LINE = re.compile(r"<doctest line \d+>$")
VISIBILITY_KEYWORD = "pub"
VISIBILITY_SCOPE_WORDS = ("crate", "super", "self")
MODULE_KINDS = ("mod", "mod_head")
FILES_PER_WORKER_CHUNK = 4

MODULE_ATTRIBUTES_NAME = "<module attributes>"
FAMILY_ROOT_NAMES = ("main.rs", "mod.rs", "lib.rs")

# plain 是剥掉可见性之后的指纹，layout 是再抹平 rustfmt 排版（lib_crates_split_layout）之后的指纹；没有排版可抹的单元两个相同
Record = collections.namedtuple("Record", "kind key name file start_line end_line raw plain layout tokens")
# files：包目录到 {文件路径: 文件内容的 sha256}（这个包的每份 .rs 与 .wgsl）
TreeUnits = collections.namedtuple("TreeUnits", "units refusals known files")


def visibility_stripped(tokens):
    """词法单元表里去掉 pub 与紧跟的限定组（(crate)、(super)、(self)、(in 路径)）；pub 后面的括号是别的东西（元组类型）的原样留下。"""
    stripped, index, count = [], 0, len(tokens)
    while index < count:
        token = tokens[index]
        if token != VISIBILITY_KEYWORD:
            stripped.append(token)
            index += 1
            continue
        index += 1
        if index < count and tokens[index] == "(" and index + 1 < count:
            scope_word = tokens[index + 1]
            if scope_word in VISIBILITY_SCOPE_WORDS and index + 2 < count and tokens[index + 2] == ")":
                index += 3
            elif scope_word == "in":
                close = index + 1
                while close < count and tokens[close] != ")":
                    close += 1
                index = close + 1
    return stripped


def unit_key_name(unit):
    name = unit.name
    scope = unit.scope
    if scope and "module-scope-kept-in-names" not in broken:
        prefix = "::".join(scope) + "::"
        if name.startswith(prefix):
            name = name[len(prefix):]
    if "line-numbers-in-names" not in broken:
        name = LINE_NUMBER_SUFFIX.sub("@", name)
        name = DOCTEST_LINE.sub("<doctest line>", name)
    return name


def make_record(source, unit, is_rust):
    plain, tokens = unit.fingerprint, None
    layout = plain
    if unit.first_token is not None:
        span = source.texts[unit.first_token:unit.last_token]
        stripped = visibility_stripped(span) if is_rust and "visibility-not-stripped" not in broken else span
        if stripped != span:
            plain = function_units.sha256_text("\x1f".join(stripped))
        layout = plain
        if is_rust:
            flattened = rustfmt_layout.normalized(stripped)
            if flattened != stripped:
                layout = function_units.sha256_text("\x1f".join(flattened))
        if unit.kind in MODULE_KINDS:
            tokens = tuple(stripped)
    return Record(unit.kind, unit_key_name(unit), unit.name, unit.file, unit.first_line, unit.last_line, unit.fingerprint, plain, layout, tokens)


def read_source_file(job):
    root, crate_directory, path = job
    with open(os.path.join(root, path), encoding="utf-8", errors="surrogateescape") as handle:
        text = handle.read()
    digest = function_units.sha256_text(text)
    if path.endswith(WGSL_SUFFIX):
        source, is_rust = function_units.WgslFile(path, text), False
    else:
        source, is_rust = function_units.RustFile(root, path, text, crate_directory), True
    records = [make_record(source, unit, is_rust) for unit in source.units]
    refusals = [(refusal.form, refusal.file, refusal.line, refusal.snippet) for refusal in source.refusals]
    return crate_directory, path, records, refusals, digest


def run_jobs(jobs, workers):
    if workers <= 1 or len(jobs) <= 1:
        return [read_source_file(job) for job in jobs]
    context = multiprocessing.get_context("fork")
    with concurrent.futures.ProcessPoolExecutor(max_workers=min(workers, len(jobs)), mp_context=context) as pool:
        return list(pool.map(read_source_file, jobs, chunksize=FILES_PER_WORKER_CHUNK))


def crate_owning(crates, path):
    """路径落在哪个包：包目录是它最长前缀的那个。"""
    best = None
    for crate in crates:
        prefix = crate.directory + "/" if crate.directory else ""
        if path.startswith(prefix) and (best is None or len(crate.directory) > len(best.directory)):
            best = crate
    return best


def collect_tree(root, crate_filter, workers):
    """一棵树里每个包的单元记录与拒算条目；crate_filter 非空时只收点名的包。"""
    crates = function_units.discover_rust_crates(root)
    known = sorted(crate.directory for crate in crates)
    wanted = set() if "crate-filter-ignored" in broken else set(crate_filter)
    suffixes = (RUST_SUFFIX,) if "wgsl-skipped" in broken else SOURCE_SUFFIXES
    units, refusals, files, jobs = {}, {}, {}, []
    for crate in crates:
        if wanted and crate.directory not in wanted:
            continue
        units[crate.directory], refusals[crate.directory], files[crate.directory] = [], [], {}
        for path in function_units.files_under(root, suffixes, [crate.directory or "."]):
            if crate_owning(crates, path) is crate:
                jobs.append((root, crate.directory, path))
    results = run_jobs(jobs, workers)
    if len(results) != len(jobs):
        raise function_units.FunctionUnitsError(f"{root} 下读回 {len(results)} 份源文件，派出去 {len(jobs)} 份，对不上")
    for directory, path, records, found, digest in results:
        units[directory].extend(records)
        refusals[directory].extend(found)
        files[directory][path] = digest
    return TreeUnits(units, refusals, known, files)


class CrateDiff:
    """一个包两棵树的比较结果。"""

    def __init__(self, directory):
        self.directory = directory
        self.before_count = 0
        self.after_count = 0
        self.identical = 0
        self.visibility_pairs = []
        self.layout_pairs = []
        self.changed = []
        self.missing = []
        self.extra = []
        self.before_paths = frozenset()
        self.added_modules = []
        self.added_mod_heads = []
        self.added_uses = []
        self.added_impl_headers = []
        self.inline_modules = []
        self.removed_uses = []
        self.refusal_changes = []

    def red_count(self):
        return len(self.changed) + len(self.missing) + len(self.extra) + len(self.refusal_changes)

    def scaffold_count(self):
        return len(self.added_modules) + len(self.added_mod_heads) + len(self.added_uses) + len(self.added_impl_headers)


def grouped(records):
    groups = collections.defaultdict(list)
    for record in sorted(records, key=lambda item: (item.file, item.start_line, item.kind, item.name)):
        groups[(record.kind, record.key, record.plain)].append(record)
    if "duplicates-collapsed" in broken:
        return {key: members[:1] for key, members in groups.items()}
    return groups


def inline_module_became_file(head, declaration):
    """行内模块头（外属性、mod 名、{）挪成了同名的 mod 声明（同样的外属性、mod 名、;）。"""
    if "inline-module-attributes-unchecked" in broken:
        return True
    if head.tokens is None or declaration.tokens is None or not head.tokens or head.tokens[-1] != "{":
        return False
    return tuple(head.tokens[:-1]) + (";",) == tuple(declaration.tokens)


def refusal_changes(before_refusals, after_refusals):
    """拒算条目（切不开的写法）按（形态，片段）的多重集比：没被比的单元不能悄悄多出或少掉。"""
    if "refusals-unchecked" in broken:
        return []
    remaining_after = list(after_refusals)
    changes = []
    for refusal in before_refusals:
        twin = next((index for index, candidate in enumerate(remaining_after) if (candidate[0], candidate[3]) == (refusal[0], refusal[3])), None)
        if twin is None:
            changes.append(("拆前有、拆后没有", refusal))
        else:
            remaining_after.pop(twin)
    changes.extend(("拆后多出", refusal) for refusal in remaining_after)
    return changes


def match_groups(before_records, after_records, diff):
    """键相同的按多重集配对：原始指纹也相同的是完全相同，只差可见性的配成可见性对；配不上的返回（拆前独有，拆后独有）。"""
    before_groups, after_groups = grouped(before_records), grouped(after_records)
    lost, gained = [], []
    for group_key in sorted(set(before_groups) | set(after_groups)):
        remaining_after = list(after_groups.get(group_key, []))
        unmatched_before = []
        for record in before_groups.get(group_key, []):
            twin = next((index for index, candidate in enumerate(remaining_after) if candidate.raw == record.raw), None)
            if twin is None:
                unmatched_before.append(record)
            else:
                remaining_after.pop(twin)
                diff.identical += 1
        paired = min(len(unmatched_before), len(remaining_after))
        diff.visibility_pairs.extend(zip(unmatched_before[:paired], remaining_after[:paired]))
        lost.extend(unmatched_before[paired:])
        gained.extend(remaining_after[paired:])
    if "visibility-counted-as-identical" in broken:
        diff.identical += len(diff.visibility_pairs)
        diff.visibility_pairs = []
    return lost, gained


def pair_by_layout(lost, gained, diff):
    """配不上的单元里，（种类，名字，排版抹平之后的指纹）相同的是只差 rustfmt 排版（lib_crates_split_layout）：配成一对记进 layout_pairs，不算差异。"""
    if "layout-not-paired" in broken:
        return lost, gained
    waiting = collections.defaultdict(list)
    for record in gained:
        waiting[(record.kind, record.key, record.layout)].append(record)
    still_lost, used = [], set()
    for record in lost:
        candidates = waiting.get((record.kind, record.key, record.layout))
        if candidates:
            twin = candidates.pop(0)
            used.add(id(twin))
            diff.layout_pairs.append((record, twin))
        else:
            still_lost.append(record)
    if "layout-counted-as-identical" in broken:
        diff.identical += len(diff.layout_pairs)
        diff.layout_pairs = []
    return still_lost, [record for record in gained if id(record) not in used]


def family_root(path, before_paths):
    """一份拆后文件属于哪个被拆文件的族：它自己在拆前就有的算它自己；拆前没有的往上找，目录同名的 <目录>.rs、<目录>/main.rs、mod.rs、lib.rs 在拆前有的就是族根，找不到算它自己。"""
    if path in before_paths:
        return path
    parts = path.split("/")
    for depth in range(len(parts) - 1, 0, -1):
        directory = "/".join(parts[:depth])
        for candidate in [directory + ".rs"] + [f"{directory}/{name}" for name in FAMILY_ROOT_NAMES]:
            if candidate in before_paths:
                return candidate
    return path


def compare_crate(directory, before_records, after_records, before_refusals, after_refusals, strict_use, before_files, after_files):
    """before_files、after_files：这个包拆前、拆后各份源文件的 {路径: sha256}。拆前拆后逐字节相同的文件，里面的单元不进配对：
    单元的键相同就按多重集配，配不上的余数落在哪份文件是配对次序定的，让它们参与会把差异记到没碰过的文件上。"""
    diff = CrateDiff(directory)
    diff.before_count, diff.after_count = len(before_records), len(after_records)
    diff.before_paths = frozenset(before_files)
    unchanged = set() if "unchanged-files-compared" in broken else {path for path, digest in before_files.items() if after_files.get(path) == digest}
    moved_before = [record for record in before_records if record.file not in unchanged]
    moved_after = [record for record in after_records if record.file not in unchanged]
    diff.identical += len(before_records) - len(moved_before)
    lost, gained = match_groups(moved_before, moved_after, diff)
    lost, gained = pair_by_layout(lost, gained, diff)
    before_impl_headers = {record.plain for record in before_records if record.kind == "impl"}
    scaffold_as_extra = "scaffold-counted-as-extra" in broken
    gained_other = []
    for record in gained:
        if record.kind == "mod" and not scaffold_as_extra:
            diff.added_modules.append(record)
        elif record.kind == "use" and not scaffold_as_extra:
            diff.added_uses.append(record)
        elif record.kind == "impl" and record.plain in before_impl_headers and "split-impl-header-extra" not in broken:
            diff.added_impl_headers.append(record)
        else:
            gained_other.append(record)
    strict = strict_use and "strict-use-not-red" not in broken
    lost_other, lost_heads = [], []
    for record in lost:
        if record.kind == "use" and not strict:
            diff.removed_uses.append(record)
        elif record.kind == "mod_head":
            lost_heads.append(record)
        else:
            lost_other.append(record)
    unpaired_declarations = list(diff.added_modules)
    for head in lost_heads:
        declaration = next((item for item in unpaired_declarations if item.key == head.key and inline_module_became_file(head, item)), None)
        if declaration is None:
            lost_other.append(head)
        else:
            unpaired_declarations.remove(declaration)
            diff.inline_modules.append((head, declaration))
    lost_by_key, gained_by_key = collections.defaultdict(list), collections.defaultdict(list)
    for record in lost_other:
        lost_by_key[(record.kind, record.key)].append(record)
    for record in gained_other:
        gained_by_key[(record.kind, record.key)].append(record)
    for key in sorted(set(lost_by_key) | set(gained_by_key)):
        lost_members, gained_members = lost_by_key.get(key, []), gained_by_key.get(key, [])
        paired = min(len(lost_members), len(gained_members))
        diff.changed.extend(zip(lost_members[:paired], gained_members[:paired]))
        diff.missing.extend(lost_members[paired:])
        for record in gained_members[paired:]:
            new_module_head = record.kind == "mod_head" and (MODULE_ATTRIBUTES_NAME not in record.key or "module-attributes-head-scaffolded" in broken)
            (diff.added_mod_heads if new_module_head and "mod-head-extra-red" not in broken else diff.extra).append(record)
    if "changed-ignored" in broken:
        diff.changed = []
    if "missing-ignored" in broken:
        diff.missing = []
    if "extra-ignored" in broken:
        diff.extra = []
    diff.refusal_changes = refusal_changes([found for found in before_refusals if found[1] not in unchanged], [found for found in after_refusals if found[1] not in unchanged])
    return diff


def location(record):
    return f"{record.file}:{record.start_line}"


def red_lines(diff):
    directory = diff.directory or "."
    lines = []
    for before, after in diff.changed:
        lines.append(f"  ✗ 指纹变了：{directory} {before.kind} {before.name}  拆前 {location(before)} → 拆后 {location(after)}")
    for record in diff.missing:
        lines.append(f"  ✗ 拆后没有：{directory} {record.kind} {record.name}  拆前 {location(record)}")
    for record in diff.extra:
        lines.append(f"  ✗ 拆后多出：{directory} {record.kind} {record.name}  拆后 {location(record)}")
    for side, (form, file, line, snippet) in diff.refusal_changes:
        lines.append(f"  ✗ 拒算条目对不上（{side}）：{directory} {form} {file}:{line} {snippet}")
    return lines


def module_scaffold_lines(diff):
    """拆后新加的 mod 声明与行内 mod 头（脚手架）按它们所在的被拆文件族列出：族根、两样各几个、mod 名。"""
    declarations, heads = collections.defaultdict(list), collections.defaultdict(list)
    for records, groups in ((diff.added_modules, declarations), (diff.added_mod_heads, heads)):
        for record in records:
            root = record.file if "mod-head-family-by-own-file" in broken else family_root(record.file, diff.before_paths)
            groups[root].append(record.key)
    return [f"  · 新加的 mod 头（脚手架）：{diff.directory or '.'} 被拆文件 {root} 的族里 mod 声明 {len(declarations[root])} 个、行内 mod 头 {len(heads[root])} 个"
            f"（mod {'、'.join(sorted(set(declarations[root] + heads[root])))}）"
            for root in sorted(set(declarations) | set(heads))]


def judge_pure_move(before_root, after_root, crate_filter, strict_use, list_visibility, workers):
    """→ (退出码, 输出行, 数字汇总)。退出码：0 纯挪动、1 有差异、77 没有可比的包；点名的包两棵树里都没有抛 UsageProblem。"""
    before = collect_tree(before_root, crate_filter, workers)
    after = collect_tree(after_root, crate_filter, workers)
    unknown = sorted(set(crate_filter) - set(before.known) - set(after.known))
    if unknown:
        raise common.UsageProblem(f"--crate 点名的包两棵树里都没有：{'、'.join(unknown)}",
                                  f"包目录写相对树根的路径；两棵树里有的是 {'、'.join(sorted(set(before.known) | set(after.known))) or '（一个都没有）'}")
    directories = sorted(set(before.units) | set(after.units))
    totals = dict(crates=0, units_before=0, units_after=0, identical=0, visibility=0, layout=0, scaffold=0, mod_heads=0, inline=0, removed_uses=0, red=0)
    if not directories:
        return 77, [f"  ! 本次无对象可判：{before_root} 与 {after_root} 里都没有点到的包"], totals
    diffs, lines = [], []
    for directory in directories:
        if directory not in before.units or directory not in after.units:
            if "lone-crate-ignored" not in broken:
                side = "拆前" if directory in before.units else "拆后"
                lines.append(f"  ✗ 包只在{side}那棵树里：{directory or '.'}")
                totals["red"] += 1
            continue
        diff = compare_crate(directory, before.units[directory], after.units[directory], before.refusals[directory], after.refusals[directory], strict_use,
                             before.files[directory], after.files[directory])
        diffs.append(diff)
        lines.extend(red_lines(diff))
        totals["red"] += diff.red_count()
    totals["crates"] = len(diffs)
    for diff in diffs:
        totals["units_before"] += diff.before_count
        totals["units_after"] += diff.after_count
        totals["identical"] += diff.identical
        totals["visibility"] += len(diff.visibility_pairs)
        totals["layout"] += len(diff.layout_pairs)
        totals["mod_heads"] += len(diff.added_mod_heads)
        totals["scaffold"] += diff.scaffold_count()
        totals["inline"] += len(diff.inline_modules)
        totals["removed_uses"] += len(diff.removed_uses)
    if list_visibility:
        for diff in diffs:
            for before_record, after_record in diff.visibility_pairs:
                lines.append(f"  · 只因可见性变了：{diff.directory or '.'} {before_record.kind} {before_record.name}  拆前 {location(before_record)} → 拆后 {location(after_record)}")
    for diff in diffs:
        for before_record, after_record in diff.layout_pairs:
            lines.append(f"  · 只差 rustfmt 排版：{diff.directory or '.'} {before_record.kind} {before_record.name}  拆前 {location(before_record)} → 拆后 {location(after_record)}")
        lines.extend(module_scaffold_lines(diff))
    breakdown = (f"mod 声明 {sum(len(diff.added_modules) for diff in diffs)}、新加的行内 mod 头 {totals['mod_heads']}、use {sum(len(diff.added_uses) for diff in diffs)}、"
                 f"拆 impl 块多出的 impl 头 {sum(len(diff.added_impl_headers) for diff in diffs)}")
    counts = (f"完全相同 {totals['identical']} 个；只因可见性变了指纹 {totals['visibility']} 个；只差 rustfmt 排版 {totals['layout']} 个"
              f"（这两类的指纹都变了，复用键由闭包里各单元的（种类，指纹）多重集组成，含这些单元的判定会作废）；脚手架 {totals['scaffold']} 个（{breakdown}）；"
              f"行内模块挪成文件模块 {totals['inline']} 个；拆前有、拆后没有的 use {totals['removed_uses']} 个（脚手架之外，加 --strict-use 按差异判）")
    scope = f"比了 {totals['crates']} 个包、{totals['units_before']} 个单元（拆后 {totals['units_after']} 个）"
    if totals["red"]:
        lines.append(f"  · {counts}")
        lines.append(f"  ✗ 纯挪动检查：{scope}，{totals['red']} 处不是纯挪动，逐处列在上面")
        lines.append("     → 怎么办：指纹变了与拆后没有、拆后多出的单元照上面点名的文件:行改回原文；确要改行为，不并进拆分补丁")
        status = 0 if "red-exits-zero" in broken else 1
        return status, lines, totals
    lines.append(f"  ✓ 纯挪动检查：{scope}，除脚手架、可见性与 rustfmt 排版之外一个单元都没变；{counts}")
    return 0, lines, totals
