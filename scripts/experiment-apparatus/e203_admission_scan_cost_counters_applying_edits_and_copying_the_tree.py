import difflib
import hashlib
import os
import pathlib
import re
import shutil
import sys
from e203_admission_scan_cost_counters_insertion_edits import A, ALLOCATOR_HELPERS, HARNESS_FEATURES_FROM, HARNESS_FEATURES_TO, T, WHOLE_COPY_COUNTED, WHOLE_COPY_NOT_COUNTED, WHOLE_COPY_STRUCTS, edits, fields_of_the_struct, whole_copy_gaps
from e203_admission_scan_cost_counters_row_table_and_module_template import MODULE_TEMPLATE, P, ROWS, SCRIPT_FILE_NAMES


class AnchorError(Exception):
    pass


def module_text(build):
    module = f"e203_scan_{build}"
    variants = "\n".join(f"        {camel}," for camel, _, _ in ROWS)
    listing = "\n".join(f"        Row::{camel}," for camel, _, _ in ROWS)
    names = "\n".join(f'                Row::{camel} => "{snake}",' for camel, snake, _ in ROWS)
    keep, drop = ("//C ", "//T ") if build == "counts" else ("//T ", "//C ")
    lines = []
    for line in MODULE_TEMPLATE.splitlines():
        if line.startswith(drop):
            continue
        lines.append(line[len(keep):] if line.startswith(keep) else line)
    text = "\n".join(lines) + "\n"
    return (text.replace("@MODULE@", module).replace("@ROW_VARIANTS@", variants)
            .replace("@ROW_COUNT@", str(len(ROWS))).replace("@ROW_LIST@", listing).replace("@ROW_NAMES@", names))


def body_span(text, edit):
    if text.count(edit.head) != 1:
        raise AnchorError(f"{edit.ident}: 函数头在 {edit.file} 里命中 {text.count(edit.head)} 次，不是一次：{edit.head.strip()}")
    start = text.index(edit.head)
    stop = text.index(edit.end, start + len(edit.head)) + len(edit.end)
    return start, stop


def locate(text, edit):
    start, stop = body_span(text, edit)
    hits = text[start:stop].count(edit.anchor)
    if hits != 1:
        raise AnchorError(f"{edit.ident}: 锚点在 {edit.head.strip()} 的函数体里命中 {hits} 次，不是一次：{edit.anchor.strip()[:100]}")
    return start + text[start:stop].index(edit.anchor)


def apply_one(text, edit, build):
    position = locate(text, edit)
    inserted = edit.counts if build == "counts" else edit.timing
    if inserted is None:
        return text
    end = position + len(edit.anchor)
    if edit.mode == "before":
        return text[:position] + inserted + text[position:]
    if edit.mode == "after":
        return text[:end] + inserted + text[end:]
    return text[:position] + inserted + text[end:]


def scan_rust(text, positions=()):
    """不编译，从头读一遍 Rust 文字（跳过 // 与 /* */ 注释、字符串、原始字符串、字符字面量；生命周期不当字符），
    交回 (快照, 终栈, 第一处配平错误)。快照 {位置: (栈顶括号, 该位置前最后一个代码字符)}，栈顶 "" 表示不在任何括号里；
    配平错误是 None 或 (偏移, 说明)。只认 ( [ { 三种括号。判不出的写法（宏里的括号、泛型的尖括号）不管。"""
    wanted = sorted(set(positions))
    snapshots, stack, last, error = {}, [], "", None
    pairs = {")": "(", "]": "[", "}": "{"}
    index, length, which = 0, len(text), 0
    while True:
        while which < len(wanted) and wanted[which] <= index:
            snapshots[wanted[which]] = (stack[-1][0] if stack else "", last)
            which += 1
        if index >= length:
            break
        char = text[index]
        if text.startswith("//", index):
            newline = text.find("\n", index)
            index = length if newline < 0 else newline
        elif text.startswith("/*", index):
            depth, index = 1, index + 2
            while index < length and depth:
                if text.startswith("/*", index):
                    depth, index = depth + 1, index + 2
                elif text.startswith("*/", index):
                    depth, index = depth - 1, index + 2
                else:
                    index += 1
        elif char in "rb" and (index == 0 or not (text[index - 1].isalnum() or text[index - 1] == "_")) and re.match(r'b?r#*"', text[index:index + 12]):
            hashes = re.match(r'b?r(#*)"', text[index:index + 12]).group(1)
            opening = text.index('"', index) + 1
            closing = text.find('"' + hashes, opening)
            index = length if closing < 0 else closing + 1 + len(hashes)
            last = '"'
        elif char == '"':
            index += 1
            while index < length and text[index] != '"':
                index += 2 if text[index] == "\\" else 1
            index += 1
            last = '"'
        elif char == "'":
            if index + 1 < length and text[index + 1] == "\\":
                closing = text.find("'", index + 3)
                index = length if closing < 0 else closing + 1
            elif index + 2 < length and text[index + 2] == "'":
                index += 3
            else:
                index += 1
            last = "'"
        elif char in "([{":
            stack.append((char, index))
            last = char
            index += 1
        elif char in ")]}":
            if stack and stack[-1][0] == pairs[char]:
                stack.pop()
            elif error is None:
                error = (index, f"多出来的 {char}" if not stack else f"{char} 对不上栈顶的 {stack[-1][0]}")
            last = char
            index += 1
        else:
            if not char.isspace():
                last = char
            index += 1
    return snapshots, stack, error


OUR_STATEMENT = re.compile(r"^[ \t]*(?:crate::transaction::e20\d_\w+::|let _?e20\d_\w*|drop\(_e20\d_)")


def insertion_problems(after_files):
    """不起编译的判法之一：插完之后的每份 .rs 文件里，每一行我们插的语句（行首是 crate::transaction::e20N_…:: 的读数调用、`let e20N_…` 或 `let _e20N_…`、`drop(_e20N_…`）
    要落在语句层：它前一个代码字符是语句边界（; { }），且最内层括号是 {（不在 ( [ 里面，即不在表达式、实参表、链式调用中间）。
    只看插完之后的文字，不依赖每条改动的锚点怎么取、什么模式，所以锚文命中恰好一次但插进去落在表达式中间（锚文被别处的补丁包进了实参）也认得出。
    交回 (判了几行, 问题清单)；问题一行一条：文件、行号、原因、那一行。判法的限度：`Name {` 之后插（结构体字面量、match）前一个字符也是 {，判不出，靠编译兜；
    我们插的语句不是上面这几种开头的（例如接在别的语句同一行里）也不在这里判。"""
    checked, problems = 0, []
    for path in sorted(after_files):
        text = after_files[path]
        if not path.endswith(".rs"):
            continue
        starts, offset = [], 0
        for line in text.splitlines(keepends=True):
            if OUR_STATEMENT.match(line):
                starts.append(offset)
            offset += len(line)
        snapshots, _, _ = scan_rust(text, starts)
        for start in starts:
            checked += 1
            top, last = snapshots[start]
            reasons = []
            if top != "{":
                reasons.append(f"在 {top or '括号之外'} 里面（表达式、实参表或链式调用中间）")
            if last not in (";", "{", "}", ""):
                reasons.append(f"前一个代码字符是 {last!r}，不是语句边界")
            if reasons:
                line_number = text.count("\n", 0, start) + 1
                problems.append(f"name=insertion_problem file={path} line={line_number} reason={'；'.join(reasons)} text={text[start:text.index(chr(10), start)].strip()[:100]}")
    return checked, problems


def balance_problems(before_files, after_files):
    """不起编译的判法之二：插完之后每份改过的 .rs 文件括号配平：终栈空、没有多出来或对不上的右括号；插之前那份本来就配不平的（读法没读准）也报，不当绿。"""
    problems = []
    for path in sorted(after_files):
        if not path.endswith(".rs") or after_files[path] == before_files.get(path):
            continue
        for label, text in (("插之前", before_files.get(path, "")), ("插之后", after_files[path])):
            _, stack, error = scan_rust(text)
            if error or stack:
                where = error[1] if error else f"文件结束时还有没关的 {stack[-1][0]}（第 {text.count(chr(10), 0, stack[-1][1]) + 1} 行起）"
                problems.append(f"name=balance_problem file={path} stage={label} reason={where}")
    return problems


def syntax_report(label, edit_count, before_files, after_files):
    """两道不起编译的判法一起过（改动条数只用来打摘要行）；问题行与摘要行都打出来，交回问题条数。"""
    checked, lines = insertion_problems(after_files)
    lines = lines + balance_problems(before_files, after_files)
    for line in lines:
        print(line)
    print(f"name=insertion_check scope={label} edits={edit_count} inserted_statement_lines_checked={checked} problems={len(lines)}")
    return len(lines)


def transform(files, build, edit_list):
    """files: {相对路径: 原文}。交回 {相对路径: 新文}；先把所有锚点核一遍，有一个不是恰好一次就一个字节都不改。"""
    by_file = {}
    for edit in edit_list:
        by_file.setdefault(edit.file, []).append(edit)
    for path, group in by_file.items():
        if path not in files:
            raise AnchorError(f"文件不在：{path}")
        for edit in group:
            locate(files[path], edit)
    result = dict(files)
    for path, group in by_file.items():
        text = files[path]
        for edit in group:
            text = apply_one(text, edit, build)
        result[path] = text
    result[T] = result[T] + module_text(build)
    result[A] = result[A] + ALLOCATOR_HELPERS
    if build == "timing":
        harness = "crates/singlefs-verification/Cargo.toml"
        if result[harness].count(HARNESS_FEATURES_FROM) != 1:
            raise AnchorError(f"{harness} 里 lookup-work-counters 的特性行命中 {result[harness].count(HARNESS_FEATURES_FROM)} 次，不是一次")
        result[harness] = result[harness].replace(HARNESS_FEATURES_FROM, HARNESS_FEATURES_TO)
    return result


def unified(files, changed):
    out = []
    for path in sorted(changed):
        if files[path] != changed[path]:
            out.extend(difflib.unified_diff(files[path].splitlines(keepends=True), changed[path].splitlines(keepends=True),
                                            f"a/{path}", f"b/{path}"))
    return "".join(out)


def check_builds_are_not_mixed(build, diff_text):
    """作废条款第 7 条：计时构建的 diff 里不许有计数模块，计数构建里不许有计时。"""
    counting_calls = [f"{P}{name}(" for name in ("call", "step", "steps", "scale", "delta", "call_attributed", "steps_attributed", "round_begin",
                                                 "contains_counted", "query_attributed", "delta_if_attributed",
                                                 "scale_to_the_hit", "steps_to_the_hit", "step_to_the_hit")]
    if build == "timing" and ("e203_scan_counts" in diff_text or any(call in diff_text for call in counting_calls)):
        return "计时构建的 diff 里出现了计数"
    if build == "counts" and ("e203_scan_timing" in diff_text or "Instant" in diff_text or f"{P}timer(" in diff_text
                              or f"{P}timer_attributed(" in diff_text):
        return "计数构建的 diff 里出现了计时"
    return None


def read_tree(root, needed):
    files = {}
    for path in needed:
        file = pathlib.Path(root) / path
        if not file.is_file():
            print(f"不在：{file}", file=sys.stderr)
            sys.exit(2)
        files[path] = file.read_text(encoding="utf-8")
    return files


def needed_paths():
    paths = {edit.file for edit in edits()}
    paths.add("crates/singlefs-verification/Cargo.toml")
    return sorted(paths)


# 副本要编得起来必须有的：工作区清单与锁文件、所有包、cargo 配置、harness 包编译时读的每台默认线程数登记（`include_str!`）、
# 仓根 `Cargo.toml` 的 `exclude` 里点名而 验证包用 path 依赖带进来的 `prune-server`（缺了 `cargo metadata --offline` 解析不出）。
COPIED_DIRECTORIES = ("crates", ".cargo", "prune-server")
COPIED_FILES = ("Cargo.toml", "Cargo.lock", ".claude/heavy-thread-default")


class CopyError(Exception):
    pass


def manifest_paths_not_in(root):
    """root 底下每份清单里的 path 依赖与工作区成员：指到的目录（或 .rs 文件）不在 root 里的，逐条列出。"""
    import re
    root = pathlib.Path(root)
    problems = []
    manifests = [root / "Cargo.toml"]
    for name in ("crates", "prune-server"):
        if (root / name).is_dir():
            manifests.extend(sorted(path for path in (root / name).rglob("Cargo.toml") if "target" not in path.parts))
    for manifest in manifests:
        if not manifest.is_file():
            problems.append(f"{manifest.relative_to(root)} 不在")
            continue
        text = manifest.read_text(encoding="utf-8")
        for target in re.findall(r'path\s*=\s*"([^"]+)"', text):
            resolved = (manifest.parent / target).resolve()
            if not resolved.exists():
                problems.append(f"{manifest.relative_to(root)} 的 path 依赖 {target} 指到副本里不存在的位置")
        if manifest == root / "Cargo.toml":
            members = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
            for member in re.findall(r'"([^"]+)"', members.group(1) if members else ""):
                if not (root / member / "Cargo.toml").is_file():
                    problems.append(f"工作区成员 {member} 在副本里没有 Cargo.toml")
    return problems


COPY_BEGUN = ".e203-copy-begun"
COPY_DONE = ".e203-copy-done"
COPY_RESUME_OFF = "copy-resume-off"


def write_atomically(path, text):
    """先写同目录的 .partial，再换名：被杀时目标名下要么是旧的整份、要么没有，没有半份。"""
    path = pathlib.Path(path)
    temporary = path.with_name(path.name + ".partial")
    temporary.write_text(text, encoding="utf-8")
    os.replace(temporary, path)


def check_source_is_complete(source):
    missing = [name for name in COPIED_DIRECTORIES + COPIED_FILES if not (pathlib.Path(source) / name).exists()]
    if missing:
        raise CopyError(f"仓根里缺这几份，副本编不起来：{missing}")


def source_files_to_copy(source):
    """要拷的每一份文件的路径（相对 source，按次序），与 copytree 的 ignore 同口径：名叫 target 的目录与文件不要。"""
    source = pathlib.Path(source)
    paths = []
    for name in COPIED_DIRECTORIES:
        for directory, subdirectories, names in os.walk(source / name, followlinks=True):
            subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target")
            paths.extend(pathlib.Path(directory, entry).relative_to(source) for entry in sorted(names) if entry != "target")
    paths.extend(pathlib.Path(name) for name in COPIED_FILES)
    return paths


def copy_key(source, kind, diff_text):
    """副本的输入键：构建种类、本脚本全文（入口与拆出的各部分）、改动的 diff 全文、要拷的每份文件的路径与内容。键相同，上一次拷完的副本就是这一次要的副本。"""
    digest = hashlib.sha256()
    for part in (kind.encode("utf-8"), pathlib.Path(__file__).read_bytes(), diff_text.encode("utf-8")):
        digest.update(hashlib.sha256(part).digest())
    for name in SCRIPT_FILE_NAMES:
        if name != pathlib.Path(__file__).name:
            digest.update(hashlib.sha256(pathlib.Path(__file__).with_name(name).read_bytes()).digest())
    try:
        for relative in source_files_to_copy(source):
            digest.update(str(relative).encode("utf-8") + b"\0")
            digest.update(hashlib.sha256((pathlib.Path(source) / relative).read_bytes()).digest())
    except OSError as error:
        raise CopyError(f"读不了要拷的文件：{error}") from error
    return digest.hexdigest()


def prepare_copy(source, into, key):
    """决定这一次要不要拷。交回 "reused"（上一次拷完、键相同的副本，原样用）或 "fresh"（目录是空的，或上一次没拷完、键不同的副本已清掉，可以拷）。
    非空而没有开拷标记的目录不是本脚本拷的：抛 CopyError，什么也不清。"""
    source, into = pathlib.Path(source), pathlib.Path(into)
    check_source_is_complete(source)
    into.mkdir(parents=True, exist_ok=True)
    entries = [entry.name for entry in into.iterdir()]
    if not entries:
        return "fresh"
    resume_off = os.environ.get("E203_RUN_BREAK") == COPY_RESUME_OFF
    done = into / COPY_DONE
    if not resume_off and done.is_file() and done.read_text(encoding="utf-8") == key + "\n":
        return "reused"
    if resume_off or not (COPY_BEGUN in entries or COPY_BEGUN + ".partial" in entries):
        raise CopyError(f"工作目录不空：{into}（没有开拷标记 {COPY_BEGUN}，不是本脚本拷的，不清）")
    # 半拷或输入变了：先拿掉完成标记（清到一半被杀也不会被当成拷完），再清其余，开拷标记留到最后由重拷改写
    if done.exists():
        done.unlink()
    for entry in sorted(into.iterdir()):
        if entry.name == COPY_BEGUN:
            continue
        if entry.is_dir() and not entry.is_symlink():
            shutil.rmtree(entry)
        else:
            entry.unlink()
    return "fresh"


def mark_copy_done(into, key):
    write_atomically(pathlib.Path(into) / COPY_DONE, key + "\n")


def copy_tree(source, into):
    """拷 COPIED_DIRECTORIES 与 COPIED_FILES；不保留修改时刻（往副本里拷源文件的纪律）。目录里除开拷标记外已有东西、缺一份、或拷完清单里还有指到副本外的 path，抛 CopyError。
    先写开拷标记；完成标记不在这里写（还要改文件），由调用方改完之后 mark_copy_done。"""
    source, into = pathlib.Path(source), pathlib.Path(into)
    check_source_is_complete(source)
    into.mkdir(parents=True, exist_ok=True)
    if any(entry.name != COPY_BEGUN for entry in into.iterdir()):
        raise CopyError(f"工作目录不空：{into}")
    write_atomically(into / COPY_BEGUN, "e203 副本拷到一半的标记：没有 .e203-copy-done 就是没拷完，可清\n")
    for name in COPIED_DIRECTORIES:
        shutil.copytree(source / name, into / name, copy_function=shutil.copyfile,
                        ignore=shutil.ignore_patterns("target"))
    for name in COPIED_FILES:
        (into / name).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / name, into / name)
    problems = manifest_paths_not_in(into)
    if problems:
        raise CopyError("拷完的副本里清单指到副本外：" + "；".join(problems))


def check_anchors(tree):
    files = read_tree(tree, needed_paths())
    bad = 0
    for edit in edits():
        text = files[edit.file]
        try:
            position = locate(text, edit)
            print(f"name=anchor id={edit.ident} file={edit.file} line={text.count(chr(10), 0, position) + 1} hits=1")
        except AnchorError as error:
            bad += 1
            print(f"name=anchor id={edit.ident} file={edit.file} error={error}")
    for struct_name, (unlisted, absent) in whole_copy_gaps(files[A]).items():
        print(f"name=whole_copy_fields struct={struct_name} fields={len(fields_of_the_struct(files[A], dict((n, h) for n, h, _ in WHOLE_COPY_STRUCTS)[struct_name]))}"
              f" counted={len(WHOLE_COPY_COUNTED[struct_name])} not_counted_with_reason={len(WHOLE_COPY_NOT_COUNTED[struct_name])}"
              f" unlisted={unlisted} listed_but_absent={absent}")
        bad += 1 if unlisted or absent else 0
    if not bad:
        bad += syntax_gate(files)
    return 3 if bad else 0


def build(args):
    kind, source, into = args["--build"], args["--source"], args["--into"]
    if kind not in ("counts", "timing"):
        print("--build 只认 counts 或 timing", file=sys.stderr)
        return 2
    files = read_tree(source, needed_paths())
    try:
        changed = transform(files, kind, edits())
    except AnchorError as error:
        print(f"锚点不是恰好命中一次，一个字节没写：{error}", file=sys.stderr)
        return 3
    diff_text = unified(files, changed)
    mixed = check_builds_are_not_mixed(kind, diff_text)
    if mixed:
        print(mixed, file=sys.stderr)
        return 3
    try:
        check_source_is_complete(source)
        key = copy_key(source, kind, diff_text)
        state = prepare_copy(source, into, key)
        if state == "fresh":
            copy_tree(source, into)
            for path, text in changed.items():
                if text != files[path]:
                    write_atomically(pathlib.Path(into) / path, text)
    except CopyError as error:
        print(f"副本拷不全：{error}", file=sys.stderr)
        return 2
    if "--diff-out" in args:
        write_atomically(args["--diff-out"], diff_text)
    if state == "fresh":
        mark_copy_done(into, key)
    print(f"name=counters_applied build={kind} edits={len(edits())} files={sum(1 for p in changed if changed[p] != files[p])} diff_lines={diff_text.count(chr(10))} copy={state}")
    return 0


def syntax_gate(files):
    """--check-anchors 里的一段：两种构建各插一遍，再过不起编译的判法，交回有问题的构建数。"""
    return sum(1 for kind in ("counts", "timing") if syntax_report(f"e203_{kind}", len(edits()), files, transform(files, kind, edits())))
