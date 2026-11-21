"""变异行重归与登记表扫描的判法（crates-split-pure-move.py 的 remap-mutations、registries 两个子命令）。

被拆文件的「拆后那一族」：留作根的原文件（还在的话）加它目录下新出现的 .rs。目录取法：foo.rs 的是 foo/；crate 根与 mod.rs（lib.rs、main.rs、mod.rs）的是它所在的目录
（crate 根拆出的模块是同目录的 <名>.rs）。给了拆前的树，族里只收拆前没有的文件（拆前就有的别的文件里碰巧有同样的原文，不算命中）；没给就收目录下全部 .rs。
变异表读法与 scripts/mutation/crates-mutation-rows.py 一致：每行六段、制表符分隔，原文与替换文里的 \\n 表示换行。
给了测试改名表（lib_crates_split_renames.py 的 --test-renames，可以按「包:目标」分表），变异表每行的第 5、6 段里的测试名与登记表里的测试旧全名也照表判，每行只用与它的测试目标对得上的表。
"""
import os
import posixpath
import re
import tempfile
import collections

import lib_crates_split_common as common
import lib_crates_split_renames as test_renames
import lib_function_units as function_units

broken = common.broken
UsageProblem = common.UsageProblem

RUST_SUFFIX = ".rs"
TABLE_FIELD_COUNT = 6
NAME_FIELD, FILE_FIELD, ANCHOR_FIELD, REPLACEMENT_FIELD, ARGUMENTS_FIELD, TEST_FIELD = 0, 1, 2, 3, 4, 5
ANCHOR_PREVIEW_CHARACTERS = 60
ROOT_FILE_NAMES = ("lib.rs", "main.rs", "mod.rs")
MAIN_FILE_NAME = "main.rs"
REGISTRY_DIRECTORY = ".claude/gate.d"
REGISTRY_SUFFIX = ".tsv"
ZERO_HITS = "拆后那一族文件里 0 处命中"
PATH_CHARACTERS_BEFORE = r"A-Za-z0-9_./-"
PATH_CHARACTERS_AFTER = r"A-Za-z0-9_-"

# tests_changed：第 5 或第 6 段换了名的行数；test_rows：[(变异名, 列出的说明表)]，只列出不改筛选词
RemapResult = collections.namedtuple("RemapResult", "text split_rows changed kept unresolved malformed tests_changed test_rows")


read_text = common.read_text


def rust_files_of(root):
    return set(function_units.files_under(root, (RUST_SUFFIX,)))


def family_directory(old_path):
    if posixpath.basename(old_path) in ROOT_FILE_NAMES:
        return posixpath.dirname(old_path)
    return posixpath.splitext(old_path)[0]


def family_of(old_path, after_files, before_files):
    """被拆文件拆后的那一族：→ 相对树根的路径，根在前、其余按字典序。"""
    members = []
    if old_path in after_files and "family-root-excluded" not in broken:
        members.append(old_path)
    if "crate-root-siblings-unsearched" in broken and posixpath.basename(old_path) in ROOT_FILE_NAMES:
        return members
    directory = family_directory(old_path)
    prefix = directory + "/" if directory else ""
    for path in sorted(after_files):
        if path == old_path or not path.startswith(prefix):
            continue
        if before_files is not None and path in before_files and "existing-files-searched" not in broken:
            continue
        members.append(path)
    return members


def declared_children(after_root, path, members):
    """crate 根或 mod.rs 的族成员里，被它自己（拆后的内容）用 mod <名>; 声明了的那几份。"""
    absolute = os.path.join(after_root, path)
    text = read_text(absolute) if os.path.isfile(absolute) else ""
    directory = posixpath.dirname(path)
    declared = []
    for member in members:
        parts = (posixpath.relpath(member, directory) if directory else member).split("/")
        if len(parts) == 1 and not member.endswith(RUST_SUFFIX):
            continue
        name = parts[0][:-len(RUST_SUFFIX)] if len(parts) == 1 else parts[0]
        if re.search(rf"\bmod\s+{re.escape(name)}\s*;", text):
            declared.append(member)
    return declared


def detect_splits(after_root, before_files, after_files):
    """拆前的 .rs 里被拆了的：拆后同名目录下多出了拆前没有的 .rs（根挪成 <名>/main.rs 的也在这一条里）。"""
    found = []
    for path in sorted(before_files):
        members = [member for member in family_of(path, after_files, before_files) if member != path]
        if "main-root-undetected" in broken:
            members = [member for member in members if posixpath.basename(member) != MAIN_FILE_NAME]
        if "new-submodules-undetected" in broken:
            members = [member for member in members if posixpath.basename(member) == MAIN_FILE_NAME]
        if posixpath.basename(path) in ROOT_FILE_NAMES:
            members = declared_children(after_root, path, members)
        if members:
            found.append(path)
    return found


def decode_anchor(text):
    return text if "newline-escape-undecoded" in broken else text.replace("\\n", "\n")


def resolve_anchor(anchor, family_texts):
    """锚点在族里恰好一份文件、恰好命中一次 → (那份文件, None)；否则 (None, 原因)。"""
    hits = [(path, text.count(anchor)) for path, text in family_texts.items() if anchor and anchor in text]
    if not hits:
        return None, ZERO_HITS
    if len(hits) == 1:
        path, count = hits[0]
        if count == 1 or "repeated-hit-in-one-file-accepted" in broken:
            return path, None
        return None, f"{path} 里命中 {count} 次，要恰好 1 次"
    if "ambiguous-first-hit-taken" in broken:
        return hits[0][0], None
    return None, "几份文件里都有：" + "、".join(f"{path}×{count}" for path, count in hits)


def mutation_rows(table_text):
    """变异表里六段的行 → [(变异名, 第 5 段, 第 6 段)]；注释、空行与段数不对的行不收。"""
    rows = []
    for line in table_text.split("\n"):
        fields = line.split("\t")
        if line and not line.startswith("#") and len(fields) == TABLE_FIELD_COUNT:
            rows.append((fields[NAME_FIELD], fields[ARGUMENTS_FIELD], fields[TEST_FIELD]))
    return rows


def remap_table(table_text, split_paths, families, renames):
    """整份表的新文本：点名被拆文件的行改第 2 段；给了改名表（renames：RenameTable）的话，每行再照表里与它的测试目标对得上的改名换第 5、6 段里的测试名，
    列出筛选词筛不到、目标认不出或分不清的行（test_rows）；别的段、别的行、行序、结尾换行一个字节不动。families：{被拆文件: {族成员路径: 文本}}。"""
    output, split_rows, changed, kept, unresolved, malformed = [], 0, 0, 0, [], 0
    tests_changed, test_rows = 0, []
    places = test_renames.landing_places(mutation_rows(table_text), renames) if renames else {}
    for line in table_text.split("\n"):
        fields = line.split("\t")
        if not line or line.startswith("#"):
            output.append(line)
            continue
        if len(fields) != TABLE_FIELD_COUNT:
            malformed += 1
            output.append(line)
            continue
        old_path = fields[FILE_FIELD]
        if old_path in split_paths:
            split_rows += 1
            target, reason = resolve_anchor(decode_anchor(fields[ANCHOR_FIELD]), families[old_path])
            if target is None:
                if not (reason == ZERO_HITS and "zero-hit-dropped" in broken):
                    unresolved.append((fields[NAME_FIELD], fields[ANCHOR_FIELD][:ANCHOR_PREVIEW_CHARACTERS], reason))
            elif target == old_path and "unchanged-counted-as-changed" not in broken:
                kept += 1
            else:
                fields[FILE_FIELD] = target
                if "fields-reserialized" in broken:
                    fields[ANCHOR_FIELD], fields[REPLACEMENT_FIELD] = decode_anchor(fields[ANCHOR_FIELD]), decode_anchor(fields[REPLACEMENT_FIELD])
                changed += 1
        if renames:
            applied, listing, target_notes = test_renames.renames_for_row(renames, fields[ARGUMENTS_FIELD], fields[TEST_FIELD], places)
            arguments, test_name, notes = test_renames.rewrite_test_names(fields[ARGUMENTS_FIELD], fields[TEST_FIELD], applied, listing)
            if (arguments, test_name) != (fields[ARGUMENTS_FIELD], fields[TEST_FIELD]):
                tests_changed += 1
            fields[ARGUMENTS_FIELD], fields[TEST_FIELD] = arguments, test_name
            if target_notes or notes:
                test_rows.append((fields[NAME_FIELD], target_notes + notes))
        output.append("\t".join(fields))
    if "row-order-sorted" in broken:
        output.sort()
    return RemapResult("\n".join(output), split_rows, changed, kept, unresolved, malformed, tests_changed, test_rows)


def write_text_atomically(path, text):
    directory = os.path.dirname(os.path.abspath(path))
    handle, temporary = tempfile.mkstemp(prefix=".partial-", dir=directory)
    try:
        with os.fdopen(handle, "w", encoding="utf-8", errors="surrogateescape", newline="") as stream:
            stream.write(text)
        os.replace(temporary, path)
    except BaseException:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


def split_paths_for(after_root, before_root, given, renames):
    """要处理的被拆文件：给了 --split 取它们；没给就从两棵树认（给了测试改名表而没给 --before 的，没有被拆文件也行）。→ (被拆文件, 拆前文件集或 None, 拆后文件集)。"""
    after_files = rust_files_of(after_root)
    before_files = rust_files_of(before_root) if before_root else None
    if given:
        return sorted({posixpath.normpath(path) for path in given}), before_files, after_files
    if before_files is None:
        if renames:
            return [], None, after_files
        raise UsageProblem("没给 --split，也没给 --before：认不出哪些文件被拆了", "用 --split <旧路径> 点名被拆的文件（可给几次），或同时给 --before <拆前的树> 让它从两棵树里认")
    return detect_splits(after_root, before_files, after_files), before_files, after_files


def remap_command(after_root, before_root, given_splits, table_path, out_path, renames=None):
    """→ (退出码, 输出行, RemapResult 或 None)。0 全部归到位（改了或本来就在）、测试筛选都对，1 有行没改或有测试筛选列出（逐行列出），
    77 没认出被拆的文件（给了测试改名表的，没有被拆文件也照改测试名）。"""
    renames = renames or test_renames.RenameTable()
    if os.path.realpath(out_path) == os.path.realpath(table_path):
        raise UsageProblem(f"--out 与变异表是同一份文件：{table_path}", "--out 写成另一个路径，改完的整份表由主 agent 看过再换进 crates/mutations.tsv")
    splits, before_files, after_files = split_paths_for(after_root, before_root, given_splits, renames)
    if not splits and (not renames or "renames-alone-refused" in broken):
        return 77, [f"  ! 本次无对象可判：没认出被拆的文件（{after_root} 里没有哪份拆前的 .rs 在同名目录下多出子模块、也没有根挪成 <名>/main.rs 的）"], None
    texts, families = {}, {}
    for old_path in splits:
        members = sorted(after_files) if "unrelated-files-searched" in broken else family_of(old_path, after_files, before_files)
        for member in members:
            if member not in texts:
                texts[member] = read_text(os.path.join(after_root, member))
        families[old_path] = {member: texts[member] for member in members}
    table_text = read_text(table_path)
    result = remap_table(table_text, set(splits), families, renames)
    write_text_atomically(out_path, result.text)
    lines = [f"  ✗ 没改 {name}：{reason}；原文前 {ANCHOR_PREVIEW_CHARACTERS} 字：{preview}" for name, preview, reason in result.unresolved]
    lines += [f"  ✗ 测试筛选 {name}：{note}" for name, notes in result.test_rows for note in notes]
    summary = (f"点名被拆文件（{len(splits)} 份）的 {result.split_rows} 行里：改了 {result.changed} 行，没改 {result.kept + len(result.unresolved)} 行"
               f"（锚点仍在原文件命中一次 {result.kept} 行，0 次或多处命中、逐行列出 {len(result.unresolved)} 行）；整份表写到 {out_path}")
    if renames:
        summary += f"；照 {len(renames)} 条测试改名，第 5、6 段改了 {result.tests_changed} 行，列出 {len(result.test_rows)} 行（列出的筛选词不改）"
    if result.unresolved or result.test_rows:
        lines.append(f"  ✗ 变异行重归：{summary}")
        if result.unresolved:
            lines.append("     → 怎么办：列出的行锚点跨了几份文件或不在族里，由拆分的实现员改锚点（不并进别的行为改动）或点名它该落进哪份文件；其余行用 --out 那份整表换进 crates/mutations.tsv")
        if result.test_rows:
            lines.append("     → 怎么办：列出的测试筛选由拆分的实现员改第 5 段：筛选词改成拆后还筛得到那些测试的写法（子串要落在新全名里，或写成 --exact 加整名）；"
                         "说同一个全名在几个测试目标里都有的，改名表按目标写成 --test-renames <包>:<目标>=<改名表>（目标 lib、test:<名>、bin:<名>）；"
                         "说目标认不出的，第 5 段写全 -p <包> 加 --lib、--test <名> 或 --bin <名> 之一；其余行用 --out 那份整表换进 crates/mutations.tsv")
        unresolved_status = 0 if "unresolved-exit-zero" in broken else 1
        tests_status = 0 if "listed-tests-exit-zero" in broken else 1
        return max(unresolved_status if result.unresolved else 0, tests_status if result.test_rows else 0), lines, result
    lines.append(f"  ✓ 变异行重归：{summary}")
    return 0, lines, result


def registry_files(root):
    """.claude/gate.d/ 顶层的 *.tsv（样本档在 fixtures/ 下，不在顶层）→ [(相对树根的路径, 绝对路径)]。"""
    directory = os.path.join(root, REGISTRY_DIRECTORY)
    if not os.path.isdir(directory):
        return None
    found = []
    if "registry-subdirectories-scanned" in broken:
        for current, names, files in os.walk(directory):
            names.sort()
            found.extend(os.path.join(current, name) for name in sorted(files) if name.endswith(REGISTRY_SUFFIX))
    else:
        for name in sorted(os.listdir(directory)):
            path = os.path.join(directory, name)
            if os.path.isfile(path) and (name.endswith(REGISTRY_SUFFIX) or "registry-non-tsv-scanned" in broken):
                found.append(path)
    if "registry-non-tsv-scanned" in broken and "registry-subdirectories-scanned" in broken:
        found = sorted(os.path.join(current, name) for current, _names, files in os.walk(directory) for name in files)
    return [(os.path.relpath(path, root).replace(os.sep, "/"), path) for path in found]


def registry_pattern(paths):
    alternatives = "|".join(re.escape(path) for path in paths)
    if "registry-path-boundary-ignored" in broken:
        return re.compile(alternatives)
    return re.compile(rf"(?<![{PATH_CHARACTERS_BEFORE}])(?:{alternatives})(?![{PATH_CHARACTERS_AFTER}]|\.[A-Za-z0-9_])")


def registries_command(root, before_root, after_root, given_splits, renames=None):
    """→ (退出码, 输出行, 命中行表)。0 没有行点名被拆文件或测试旧全名（renames：RenameTable），1 有（只报不改），
    77 没有被拆文件也没有测试改名表，或没有登记表目录。"""
    renames = renames or test_renames.RenameTable()
    if given_splits:
        splits = sorted({posixpath.normpath(path) for path in given_splits})
    elif before_root and after_root:
        splits = detect_splits(after_root, rust_files_of(before_root), rust_files_of(after_root))
    elif renames:
        splits = []
    else:
        raise UsageProblem("没给 --split，也没给 --before 与 --after：认不出哪些文件被拆了", "用 --split <旧路径> 点名被拆的文件，或同时给 --before 与 --after")
    if not splits and not renames:
        return 77, ["  ! 本次无对象可判：没有被拆的文件"], []
    files = registry_files(root)
    if files is None:
        return 77, [f"  ! 本次无对象可判：{root} 下没有 {REGISTRY_DIRECTORY}/"], []
    split_pattern = registry_pattern(splits) if splits else None
    name_pattern = test_renames.old_name_pattern(renames) if renames else None
    split_hits, name_hits = [], []
    for relative, path in files:
        for number, line in enumerate(read_text(path).split("\n"), start=1):
            if split_pattern and split_pattern.search(line):
                split_hits.append(f"{relative}:{number}\t{line}")
            name_hit = test_renames.registry_name_hit(name_pattern, renames, relative, number, line) if name_pattern else None
            if name_hit:
                name_hits.append(name_hit)
    hits = split_hits + name_hits
    lines = [f"  ✗ 登记表点名被拆文件：{hit}" for hit in split_hits]
    lines += [f"  ✗ 登记表点名测试旧全名：{hit}" for hit in name_hits]
    scope = f"扫了 {len(files)} 份登记表、{len(splits)} 个被拆文件的旧路径" + (f"、{len(renames.olds())} 个测试旧全名" if renames else "")
    if hits:
        counts = f"{len(split_hits)} 行点名被拆文件" + (f"、{len(name_hits)} 行点名测试旧全名" if renames else "")
        lines.append(f"  ✗ 登记表扫描：{scope}，{counts}，逐行列在上面（只报不改）")
        lines.append("     → 怎么办：这些行按拆后的位置改（测试旧全名照 ← 后面写的新全名换；它们是门禁登记，不在拆分补丁里顺手改）；路径与被点名的单元对不上的，交主 agent 判")
        return (0 if "red-exits-zero" in broken else 1), lines, hits
    lines.append(f"  ✓ 登记表扫描：{scope}，没有行点名被拆文件" + ("或测试旧全名" if renames else ""))
    return 0, lines, hits
