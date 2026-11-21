"""准入（入口 admission.py）的一块：崩溃枚举用例——登记表自查、测试目标源码扫描、日志判定、线程数、起用例的命令、块账与各 crash-case-* 子命令。
入口 admission.py 把这一块的名字再导出。
"""

import glob
import os
import re
import subprocess
import sys

from admission_base import (
     REGISTRATION_TABLE, EXIT_REGISTRATION_ERROR, EXPERIMENT_KEY_FORM, STAGE_CELL_KEY_FORM, ENTRY_FORM, CRASH_CASE_KEY_FORM,
     UNREGISTERED_FEATURE_OF_BREAK, GPU_FEATURE, MULTI_HOST_CONFIGURATION_VARIABLE, GPU_QUIESCE_SCRIPT, SERVICE_CARD_VARIABLE, BINDGEN_LIBRARY_SCRIPT,
     BINDGEN_VARIABLE, BINDGEN_ASSIGNMENT_FORM, CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST, CRASH_CASE_FINDINGS_NOT_WRITTEN,
     CRASH_CASE_BLOCKS_LINE_NOT_PRINTED, CRASH_CASE_NARROWING_VARIABLES, CRASH_SHARE_VARIABLE, CRASH_CASE_SHARE_FORM, CRASH_CASE_STAGES,
     CRASH_STAGE_VARIABLE, CRASH_HOST_WEIGHTS_VARIABLE, CRASH_LIBRARY_RUN_KIND_VARIABLE, CRASH_CASE_BLOCKS_PREFIX, CRASH_CASE_BLOCKS_FIELDS,
     EXIT_BLOCKS_RED, EXIT_BLOCKS_LINE_BAD, CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX, PASSED_ONE_TEST_FORM, POSITIVE_INTEGER_FORM,
     CRASH_POINT_REPLAY_THREADS_VARIABLE, DEFAULT_WORKER_THREAD_SOURCES, IGNORE_ATTRIBUTE_FORM, IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE, ATTRIBUTE_START,
     FUNCTION_QUALIFIER_AT_END, VERIFICATION_PACKAGE, RegistrationError, InputManifestError, read_registration_rows, resolve_entries, entry_file,
     experiment_condition_file_problems, parse_gate_condition, break_is_set, say, parse_condition, parse_crash_case, crash_cases_of,
     crash_case_of_key, fields_of_line, machine_core_count, registered_heavy_threads, multi_host_configuration_file, configuration_value,
)
from admission_unit_key import workspace_package_directories

def lint_registration_rows(rows, root=None, all_rows=None):
    """登记表自己写对没有：键的形状、入口的写法（给了 root 还核入口的文件在不在）、准入条件认不认得（实验行只认实验的种类、门禁行只认门禁的种类）、
    @ 引用解不解得开、崩溃枚举用例第二列写 -；给了 root 还核门禁行 probe= / environment= 指的脚本在不在、实验行 question-row= / product-field= 点名的文件在不在
    （弄坏开关 condition-files-unchecked 下不核这一样）。返回问题清单。"""
    problems = []
    for row in rows:
        is_experiment = bool(EXPERIMENT_KEY_FORM.match(row.key))
        is_crash_case = bool(CRASH_CASE_KEY_FORM.match(row.key))
        if not is_experiment and not is_crash_case and not row.key.endswith(".sh") and not STAGE_CELL_KEY_FORM.match(row.key):
            problems.append(f"第 {row.line_number} 行：键 {row.key} 既不是门禁阶段文件名（*.sh）、阶段里的一格（<阶段>.sh:<格名>）、实验键（E<号>[/<方式>]），"
                            "也不是崩溃枚举用例（crash-case:<小写字母、数字、->）")
        for entry in row.entries:
            if not ENTRY_FORM.match(entry):
                problems.append(f"第 {row.line_number} 行：{row.key} 的入口 {entry} 认不出（写 <文件>::<限定名>、<文件>、tests-of:<包>、vm:<档名> 或 @<别的键>）")
                continue
            path = entry_file(entry)
            if root is not None and path is not None and not os.path.exists(os.path.join(root, path)):
                problems.append(f"第 {row.line_number} 行：{row.key} 的入口 {entry} 指的文件 {path} 不在")
        if is_crash_case:
            if row.entries:
                problems.append(f"第 {row.line_number} 行：崩溃枚举用例 {row.key} 的第二列写 -（入口就是第三列的 test=），实际写了 {' '.join(row.entries)}")
            try:
                crash_case = parse_crash_case(row)
                if root is not None:
                    crash_case_test_files(root, crash_case)
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
            continue
        for token in row.conditions:
            try:
                if is_experiment:
                    kind, parts = parse_condition(token)
                    if root is not None and not break_is_set("condition-files-unchecked"):
                        problems.extend(f"第 {row.line_number} 行：{row.key} 的 {token} {problem}"
                                        for problem in experiment_condition_file_problems(root, kind, parts))
                    continue
                kind, parts = parse_gate_condition(token)
                if root is not None and kind in ("probe", "environment") and not os.path.isfile(os.path.join(root, parts["script"])):
                    problems.append(f"第 {row.line_number} 行：门禁行 {row.key} 的 {token} 指的脚本 {parts['script']} 不在")
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
        try:
            resolve_entries(all_rows or rows, row.key)
        except RegistrationError as error:
            problems.append(f"第 {row.line_number} 行：{error}")
    crash_case_lines = {}
    for row in rows:
        if CRASH_CASE_KEY_FORM.match(row.key):
            crash_case_lines.setdefault(row.key, []).append(row.line_number)
    for key, line_numbers in crash_case_lines.items():
        if len(line_numbers) > 1:
            problems.append(f"第 {'、'.join(str(number) for number in line_numbers)} 行：崩溃枚举用例 {key} 登记了 {len(line_numbers)} 行，一条用例只许一行")
    return problems


def test_target_source_files(root, package_directory, target):
    """包目录（相对 root）下测试目标 target 的根文件（相对 root）：tests/<目标>.rs，或目录目标的 tests/<目标>/main.rs（子模块由 files_brought_in_by_target 顺着 mod 带进来，
    再嵌一层的 mod 在 files_brought_in_by_target 里按「所在目录/文件名」算位置，目录里的别的 .rs 不当根文件）；目标不在交空表。
    弄坏开关 directory-target-every-file-as-root 下目录目标把目录里全部 .rs 都当根文件交出去。"""
    single_file = os.path.join(package_directory, "tests", target + ".rs")
    directory_target = os.path.join(package_directory, "tests", target)
    if os.path.isfile(os.path.join(root, single_file)):
        return [single_file]
    if os.path.isfile(os.path.join(root, directory_target, "main.rs")):
        return sorted(os.path.relpath(path, root) for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True)) \
            if break_is_set("directory-target-every-file-as-root") else [os.path.join(directory_target, "main.rs")]
    return []


# 测试目标顺着带进来的源文件（files_brought_in_by_target）：include!（宏名前带不带 ::core:: / std:: 都算）、mod <名>; 与 #[path = "…"] mod <名>;
INCLUDE_CODE_MACRO = re.compile(r"\binclude\s*!\s*[(\[{]\s*")
MODULE_DECLARATION = re.compile(r"\bmod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTRIBUTE = re.compile(r'^#\s*\[\s*path\s*=\s*(b?r?#*".*")\s*\]$', re.S)
# 用 use 把 include 族宏（include!、include_str!、include_bytes!）引进来（改不改名都算）：之后怎么写调用认不出
USE_OF_INCLUDE_MACRO = re.compile(r"\buse\b[^;]*\binclude(?:_str|_bytes)?\b[^;]*;")
# 带进来的源文件最多顺几份（防成环的 include!、mod 互相带）
TARGET_FILES_FOLLOWED_AT_MOST = 500


def string_literal_value(literal):
    """Rust 字符串字面量（"…" 或 r#"…"#）的值；带转义的（反斜杠）交 None（路径里用不到，判不出就按判不出算）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    if plain and "\\" not in plain.group(1):
        return plain.group(1)
    return None


def files_brought_in_by_target(root, relative_files):
    """测试目标的源文件（relative_files，相对 root）顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod <名>; 与 mod <名>;（<名>.rs 或 <名>/mod.rs）
    带进来的全部源文件（相对 root，按找到的次序，头几份是 relative_files 本身）：返回 (文件清单, 判不出的原因或 None)。
    判不出：include! 的参数不是字符串字面量（套 concat!、env! 这一类）、带进来的文件找不到或读不了、有代码用 use 把 include 族宏引进来、带进来的超过上限。
    路径的起点照 rustc：include! 与 #[path] 从写它的那份文件所在的目录起；mod <名>; 在测试目标的根文件与 mod.rs 里从所在目录起，别的文件从「所在目录/文件名」起。
    弄坏开关 target-own-files-only 下只交 relative_files 本身。"""
    if break_is_set("target-own-files-only"):
        return list(relative_files), None
    roots = set(relative_files)
    found, queue = [], list(relative_files)
    while queue:
        relative = os.path.normpath(queue.pop(0))
        if relative in found:
            continue
        if len(found) >= TARGET_FILES_FOLLOWED_AT_MOST:
            return found, f"带进来的源文件超过 {TARGET_FILES_FOLLOWED_AT_MOST} 份"
        try:
            with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
                code = rust_code_without_comments(handle.read())
        except OSError as error:
            return found, f"读不了 {relative}：{error}"
        found.append(relative)
        directory = os.path.dirname(relative)
        if USE_OF_INCLUDE_MACRO.search(code):
            return found, f"{relative} 里用 use 把 include 族宏引进来了（之后的调用认不出读的是哪一份）"
        for call in INCLUDE_CODE_MACRO.finditer(code):
            literal_end = end_of_literal(code, call.end())
            value = string_literal_value(code[call.end():literal_end]) if literal_end is not None and code[call.end()] in 'br"' else None
            if value is None or not re.match(r"\s*[)\]}]", code[literal_end:]):
                return found, f"{relative} 里 include! 的参数不是一个字符串字面量（{code[call.start():call.end() + 40].strip()}…）：读的是哪一份判不出"
            queue.append(os.path.relpath(value, root) if os.path.isabs(value) else os.path.join(directory, value))
        attribute_end_to_start = {end: start for start, end in attribute_spans(code)}
        module_base = directory if (relative in roots or os.path.basename(relative) in ("mod.rs", "main.rs", "lib.rs")) \
            else os.path.join(directory, os.path.splitext(os.path.basename(relative))[0])
        for declaration in MODULE_DECLARATION.finditer(code):
            head = code[:declaration.start()].rstrip()
            qualifier = re.search(r"\bpub(?:\s*\([^()]*\))?$", head)
            if qualifier:
                head = head[:qualifier.start()].rstrip()
            path_value, cursor = None, len(head)
            while cursor in attribute_end_to_start:
                attribute = head[attribute_end_to_start[cursor]:cursor]
                path_attribute = PATH_ATTRIBUTE.match(attribute)
                if path_attribute:
                    path_value = string_literal_value(path_attribute.group(1))
                    if path_value is None:
                        return found, f"{relative} 里 {attribute} 的路径判不出"
                cursor = len(head[:attribute_end_to_start[cursor]].rstrip())
            if path_value is not None:
                queue.append(os.path.join(directory, path_value))
                continue
            name = declaration.group(1)
            candidates = [os.path.join(module_base, name + ".rs"), os.path.join(module_base, name, "mod.rs")]
            existing = [candidate for candidate in candidates if os.path.isfile(os.path.join(root, candidate))]
            if not existing:
                return found, f"{relative} 里 mod {name}; 找不到 {' 或 '.join(candidates)}"
            queue += existing
    return found, None


def crash_case_test_files(root, case):
    """这条用例的测试目标的源文件（仓根起）：tests/<目标>.rs，或目录目标的 tests/<目标>/main.rs，连同它们顺着
    include! / #[path] / mod 带进来的（files_brought_in_by_target）。
    包不是工作区成员、目标不在、带进来的判不出、用例函数在目标里找不到、用例函数有一处标了 #[ignore]，都抛 RegistrationError。"""
    package_directory = workspace_package_directories(root).get(case.package)
    if package_directory is None:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
    files = test_target_source_files(root, package_directory, case.target)
    if not files:
        single_file = os.path.join(package_directory, "tests", case.target + ".rs")
        directory_target = os.path.join(package_directory, "tests", case.target)
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
    files, undecided = files_brought_in_by_target(root, files)
    if undecided:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 带进来的源文件判不出（{undecided}）：用例函数有没有标 #[ignore] "
                                "判不出；include! / #[path] 写成字符串字面量、不用 use 改名 include 族宏")
    marks = definitions_marked_ignored(root, files, case.function)
    if not marks:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到字面的 fn {case.function}(："
                                "改了名？宏生成的用例判不出有没有标 #[ignore]，要写成字面的函数")
    marked = [relative for relative, is_marked in marks if is_marked]
    if marked:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 有 {len(marked)} 处定义标了 #[ignore]（{'、'.join(marked)}；"
                                f"共 {len(marks)} 处，cfg 二选一的、子模块里同名的都算）：崩溃枚举用例不分档，不带 --ignored 的 cargo test 就是它的跑法；"
                                "去掉每一处的 #[ignore]（门禁与入口按条复用那一趟用 --exclude-registered 把它排掉，交按块那一路）")
    return files


def rust_code_without_comments(text):
    """去掉 Rust 源码里的注释（行注释、可嵌套的块注释），字符串、原始字符串、字符字面量原样留着：判「别处有没有代码点名一个测试目标」时，
    注释里提到它不算，字符串里写它的路径（include_str!、#[path]、运行期按路径读）算。"""
    pieces = []
    position = 0
    length = len(text)
    while position < length:
        if text.startswith("//", position):
            line_end = text.find("\n", position)
            position = length if line_end < 0 else line_end
            continue
        if text.startswith("/*", position):
            depth = 0
            while position < length:
                if text.startswith("/*", position):
                    depth += 1
                    position += 2
                elif text.startswith("*/", position):
                    depth -= 1
                    position += 2
                    if depth == 0:
                        break
                else:
                    position += 1
            pieces.append(" ")
            continue
        end = end_of_literal(text, position)
        if end is not None:
            pieces.append(text[position:end])
            position = end
            continue
        pieces.append(text[position])
        position += 1
    return "".join(pieces)


RAW_STRING_START = re.compile(r'b?r(?P<hashes>#*)"')


def end_of_literal(code, position):
    """code[position] 起是字符串、原始字符串或字符字面量时交回它之后那个下标；不是时交 None（生命周期不算字面量）。"""
    character = code[position]
    if character in "br":
        raw_string = RAW_STRING_START.match(code, position)
        previous_is_identifier = position > 0 and (code[position - 1].isalnum() or code[position - 1] == "_")
        if raw_string and not previous_is_identifier:
            closing = '"' + raw_string.group("hashes")
            end = code.find(closing, raw_string.end())
            return len(code) if end < 0 else end + len(closing)
    if character == '"':
        end = position + 1
        while end < len(code) and code[end] != '"':
            end += 2 if code[end] == "\\" else 1
        return min(end + 1, len(code))
    if character == "'":
        if code.startswith("\\", position + 1):
            closing = code.find("'", position + 3)
            return len(code) if closing < 0 else closing + 1
        if position + 2 < len(code) and code[position + 2] == "'":
            return position + 3
    return None


def attribute_spans(code):
    """去掉注释之后的代码里每一条属性（`#[…]`、`#![…]`）的 (起, 止)：方括号按层数配对，字符串、原始字符串与字符字面量里的括号不算。"""
    spans = []
    position = 0
    while position < len(code):
        end = end_of_literal(code, position)
        if end is not None:
            position = end
            continue
        attribute = ATTRIBUTE_START.match(code, position)
        if not attribute:
            position += 1
            continue
        depth, cursor = 0, attribute.end() - 1
        while cursor < len(code):
            end = end_of_literal(code, cursor)
            if end is not None:
                cursor = end
                continue
            if code[cursor] == "[":
                depth += 1
            elif code[cursor] == "]":
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        spans.append((position, min(cursor + 1, len(code))))
        position = cursor + 1
    return spans


def attributes_before_each_definition(code, function):
    """去掉注释之后的代码里每一处 `fn <function>(`（cfg 二选一的、子模块里同名的都算，按出现的次序）前面紧挨着的那串属性：
    [[属性原文，一条一个，按出现的次序], …]；一处都没有交空表。属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。
    弄坏开关 first-definition-only 下只交第一处。"""
    attribute_lists = []
    for found in re.finditer(r"\bfn\s+" + re.escape(function) + r"\s*\(", code):
        head = code[:found.start()].rstrip()
        while True:
            qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
            if not qualifier:
                break
            head = head[:qualifier.start()].rstrip()
        start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
        attributes = []
        cursor = len(head)
        while cursor in start_of_attribute_ending_at:
            start = start_of_attribute_ending_at[cursor]
            attributes.append(head[start:cursor])
            cursor = len(head[:start].rstrip())
        attribute_lists.append(list(reversed(attributes)))
        if break_is_set("first-definition-only"):
            break
    return attribute_lists


def attributes_mark_ignored(attributes):
    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]（# 与 [ 之间许空白）。"""
    form = IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE if break_is_set("ignore-attribute-without-space") else IGNORE_ATTRIBUTE_FORM
    return any(form.match(attribute) for attribute in attributes)


def definitions_marked_ignored(root, relative_files, function):
    """测试目标的源文件（relative_files，相对 root）里每一处 `fn <function>(` 标没标 #[ignore]：[(文件, 标了?)]，按文件与出现的次序；
    一处都没有交空表，读不了抛 OSError。弄坏开关 first-definition-only 下只看头一份有定义的文件里的第一处。"""
    marks = []
    for relative in relative_files:
        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
            attribute_lists = attributes_before_each_definition(rust_code_without_comments(handle.read()), function)
        marks += [(relative, attributes_mark_ignored(attributes)) for attributes in attribute_lists]
        if marks and break_is_set("first-definition-only"):
            break
    return marks


def judge_worker_threads(prefix, count_line, log_lines, machine_cores, threads_explicitly_one, default_threads):
    """按计数行的 states= 找那一行 CRASH_POINT_REPLAY_PARALLEL_FINISHED，判工作线程：返回 (原因或 None, 一句说明)。
    计数行自己带 worker_threads= 与 slices= 的（跑完那一行就是计数行：crash-injection 默认批的 CRASH_INJECTION_FINISHED 这一类）拿它自己判，
    它不续跑、不带 resumed_slices= / freshly_run_slices= 的按读回 0 片、这一趟跑了全部片算（弄坏开关 threads-skip-self-contained-finish-line 下
    这一类不判线程）。
    判红只在这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、线程数没显式设成 1 时：全部片从进度文件读回时起 0 个线程，
    只剩 1 片要跑时最多起 1 个，这两种都不是「线程数没传进去」。
    另判红：那一行的 worker_threads_source= 是用例自己取的默认（DEFAULT_WORKER_THREAD_SOURCES）而 configured_worker_threads= 不等于
    default_threads（每台默认重活线程数，registered_heavy_threads）：线程变量没传进去，或用例的默认还按核数算（弄坏开关 threads-default-unchecked 下不判）。"""
    count_fields = fields_of_line(count_line)
    if "worker_threads" in count_fields and "slices" in count_fields:
        if break_is_set("threads-skip-self-contained-finish-line"):
            return None, f"{prefix}：跑完那一行自己带线程数，没判（弄坏开关 threads-skip-self-contained-finish-line）"
        finished_line, fields = count_line, dict(count_fields)
        if "resumed_slices" not in fields and "freshly_run_slices" not in fields:
            fields.update(resumed_slices="0", freshly_run_slices=fields["slices"])
    else:
        states = count_fields.get("states")
        finished = [line for line in log_lines if line.startswith(f"{CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX}states={states} ")] if states else []
        if not finished:
            return (f"{prefix} 那一行（states={states or '读不到'}）找不到状态数对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行，判不出起了几个工作线程"
                    "（全量要经 crates/singlefs-verification/src/crash.rs 的 enumerate_crash_states_in_state_slices 跑）"), ""
        finished_line, fields = finished[0], fields_of_line(finished[0])
    try:
        worker_threads = int(fields["worker_threads"])
        slices = int(fields["slices"])
        resumed_slices = int(fields["resumed_slices"])
        freshly_run_slices = int(fields["freshly_run_slices"])
    except (KeyError, ValueError):
        return (f"跑完那一行缺 worker_threads= / slices= / resumed_slices= / freshly_run_slices=，或不是整数："
                f"{finished_line}"), ""
    if resumed_slices + freshly_run_slices != slices:
        return (f"跑完那一行读回的片 {resumed_slices} + 这一趟跑的片 {freshly_run_slices} ≠ 总片数 {slices}："
                f"{finished_line}"), ""
    if freshly_run_slices > 0 and worker_threads == 0:
        return f"这一趟跑了 {freshly_run_slices} 片，却报起了 0 个工作线程：{finished_line}", ""
    if "shards" in fields and not break_is_set("threads-ignore-shards"):
        return judge_threads_of_each_shard(prefix, finished_line, fields, default_threads)
    source = fields.get("worker_threads_source")
    if source in DEFAULT_WORKER_THREAD_SOURCES and fields.get("configured_worker_threads") != str(default_threads) and not break_is_set("threads-default-unchecked"):
        return (f"{prefix}：线程数没显式设（worker_threads_source={source}，用例自己取的默认），配的 {fields.get('configured_worker_threads') or '读不到'} 个"
                f"不等于每台默认重活线程数 {default_threads}（.claude/heavy-thread-default；多半是线程变量没传进去，或用例的默认还按核数算）：{finished_line}"), ""
    if break_is_set("threads-by-worker-count"):
        judged_on_one_thread = worker_threads == 1
    else:
        judged_on_one_thread = freshly_run_slices >= 2 and worker_threads == 1
    if judged_on_one_thread and machine_cores > 1 and not threads_explicitly_one:
        return (f"本机 {machine_cores} 核、线程数没显式设成 1，这一趟跑了 {freshly_run_slices} 片却只起了 1 个工作线程"
                f"（多半是线程数没传进去）：{finished_line}"), ""
    if freshly_run_slices == 0:
        note = f"{prefix}：全部 {slices} 片从进度文件读回，这一趟没起工作线程"
    else:
        note = f"{prefix}：{worker_threads} 个工作线程跑了 {freshly_run_slices} 片、从进度文件读回 {resumed_slices} 片（共 {slices} 片）"
    return None, note


# merge 那一行逐片报的数（crates/singlefs-verification/src/crash.rs 的 merge_the_shard_ledgers，逗号分隔、按第几片排）
SHARD_FIELD_NAMES = ("worker_threads", "configured_worker_threads", "worker_threads_sources", "available_parallelism",
                     "resumed_slices", "freshly_run_slices")


def judge_threads_of_each_shard(prefix, finished_line, fields, default_threads):
    """双机分片 merge 那一趟的 CRASH_POINT_REPLAY_PARALLEL_FINISHED（带 shards= 与逐片的 shard_…=）：每一片照单机的判法判。返回 (原因或 None, 一句说明)。
    判红：逐片字段缺、片数不一、不是整数；某一片跑了片却报 0 个线程；某一片这一趟跑了至少两片、只起了 1 个工作线程、那台机器多于 1 核、
    那一片的线程数不是显式设成 1（来源是环境变量、配的是 1）；某一片的来源是用例自己取的默认而配的数不等于 default_threads（每台默认重活线程数，
    两台各取一份；弄坏开关 threads-default-unchecked 下不判）。机器核数与「显式设成 1」都取那一片账本里记的，不取 merge 这台的。"""
    try:
        shard_count = int(fields["shards"])
        columns = {name: fields[f"shard_{name}"].split(",") for name in SHARD_FIELD_NAMES}
        if shard_count < 1 or any(len(values) != shard_count for values in columns.values()):
            raise ValueError("片数不一")
        numbers = {name: [int(value) for value in values] for name, values in columns.items() if name != "worker_threads_sources"}
    except (KeyError, ValueError):
        return (f"{prefix} 对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 带 shards=，逐片的 shard_…= 字段缺、片数与 shards= 不一或不是整数：{finished_line}"), ""
    notes = []
    for shard_index in range(shard_count):
        spawned = numbers["worker_threads"][shard_index]
        configured = numbers["configured_worker_threads"][shard_index]
        source = columns["worker_threads_sources"][shard_index]
        cores = numbers["available_parallelism"][shard_index]
        resumed = numbers["resumed_slices"][shard_index]
        freshly_run = numbers["freshly_run_slices"][shard_index]
        shard_text = f"第 {shard_index}/{shard_count} 片"
        if freshly_run > 0 and spawned == 0:
            return f"{shard_text}跑了 {freshly_run} 片，却报起了 0 个工作线程：{finished_line}", ""
        if source in DEFAULT_WORKER_THREAD_SOURCES and configured != default_threads and not break_is_set("threads-default-unchecked"):
            return (f"{shard_text}：线程数没显式设（来源 {source}，用例自己取的默认），配的 {configured} 个不等于每台默认重活线程数 {default_threads}"
                    f"（.claude/heavy-thread-default；多半是线程变量没传进去，或用例的默认还按核数算）：{finished_line}"), ""
        explicitly_one = source == "environment_variable" and configured == 1
        if freshly_run >= 2 and spawned == 1 and cores > 1 and not explicitly_one:
            return (f"{shard_text}：那台机器 {cores} 核、线程数没显式设成 1，跑了 {freshly_run} 片却只起了 1 个工作线程"
                    f"（多半是线程数没传进去）：{finished_line}"), ""
        notes.append(f"{shard_text} {spawned} 个线程跑了 {freshly_run} 片、读回 {resumed} 片（那台 {cores} 核）")
    return None, f"{prefix}：双机分片 {shard_count} 片再 merge，{'；'.join(notes)}"


def crash_case_narrowing_settings(environment):
    """这一趟设了的缩域变量，逐个写成「名=值」（设了就算，空值也算：与门禁 crash-point-replay 收尾那一判同一个算法）；没缩域是空的。
    弄坏开关 exhaustive-judged-when-narrowed 下一律当没缩域。"""
    if break_is_set("exhaustive-judged-when-narrowed"):
        return ()
    return tuple(f"{name}={environment[name]}" for name in CRASH_CASE_NARROWING_VARIABLES if name in environment)


def judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one, default_threads, narrowing=()):
    """判跑一条用例的日志：返回 (原因的清单, 记录行（门禁原样打出来）, 线程那几句)。原因的清单为空才算判绿。
    default_threads 是每台默认重活线程数（registered_heavy_threads），判线程来源是默认的那几行用。
    narrowing 是这一趟设了的缩域变量（crash_case_narrowing_settings）：不空的那一趟枚举本来就不是全量，计数行 exhaustive=false 不判；
    这一行没有 exhaustive= 字段的照判红。弄坏开关 exhaustive-unjudged-without-narrowing 下一律不判。"""
    log_lines = log_text.split("\n")
    problems, recorded_lines, thread_notes = [], [], []
    results = [line for line in log_lines if line.startswith("test result: ")]
    if len(results) != 1 or not PASSED_ONE_TEST_FORM.match(results[0]):
        problems.append(f"读不到恰好一行「test result: ok. 1 passed; 0 failed; …」：过滤到 {case.function} 应当恰好跑一条用例，"
                        f"读到 {len(results)} 行 test result（{'｜'.join(results[:3]) or '一行都没有'}）")
    else:
        recorded_lines.append("test_result=" + results[0])
    count_line_of_prefix = {}
    for prefix in case.count_lines:
        found = [line for line in log_lines if line.startswith(prefix + " ")]
        if len(found) != 1:
            problems.append(f"以「{prefix} 」开头的计数行要恰好一行，读到 {len(found)} 行")
            continue
        count_line_of_prefix[prefix] = found[0]
        recorded_lines.append(found[0])
    for prefix in case.exhaustive_lines:
        line = count_line_of_prefix.get(prefix)
        if line is None:
            continue
        exhaustive = fields_of_line(line).get("exhaustive")
        if exhaustive == "true" or (narrowing and exhaustive == "false") or break_is_set("exhaustive-unjudged-without-narrowing"):
            continue
        problems.append(f"{prefix} 那一行不带 exhaustive=true，不是全量（枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)）：{line}")
    for prefix in case.thread_lines:
        line = count_line_of_prefix.get(prefix)
        if line is None:
            continue
        problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one, default_threads)
        if problem:
            problems.append(problem)
        else:
            thread_notes.append(note)
    recorded_lines += ["parallel_finished=" + line for line in log_lines if line.startswith(CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX)]
    if not break_is_set("single-worker-threads-field"):
        recorded_lines.append("started_worker_threads=" + started_worker_threads_text(case, thread_notes))
    return problems, recorded_lines, thread_notes


def started_worker_threads_text(case, thread_notes):
    """记录行里「实际起的工作线程」那一格：登记了 threads= 的取判线程那几句（CRASH_POINT_REPLAY_PARALLEL_FINISHED 里记的 worker_threads），
    没登记的写「读不到」——配的是几个另记在 configured_worker_threads=，那不是起了几个。"""
    if thread_notes:
        return "；".join(thread_notes)
    if case.thread_lines:
        return "读不到：日志没判绿，线程那一格没判"
    return ("读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；"
            "日志里有 CRASH_POINT_REPLAY_PARALLEL_FINISHED 的另原样记在 parallel_finished=）")


def configured_worker_threads_text(case, machine_cores, threads, threads_origin):
    """记录行里「配的线程数」那一格：传给用例的线程变量（这条用例读的那一个，登记行的 threads-variable=，没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）
    与它的值、配的数是显式设的（调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS）还是取的每台默认重活线程数、本机几核。"""
    origin = "显式设的" if threads_origin == "explicit" else "没设，取每台默认重活线程数"
    return f"{case.thread_variable}={threads}（{origin}），本机 {machine_cores} 核"


def crash_case_worker_threads(case, environment):
    """跑崩溃枚举用例的线程数：(本机核数, 线程数, "explicit" 或 "default")。调用方设了 SINGLEFS_CRASH_POINT_REPLAY_THREADS（非空）就用它，
    没设取每台默认重活线程数（registered_heavy_threads）；用例读的线程变量（case.thread_variable）调用方设着什么都不认，由 crash-case-command 盖成这个数。
    本机核数（machine_core_count）只用来判「本机多于 1 核」，不定线程数（弄坏开关 threads-default-from-cores 下拿它当线程数）。
    crash-case-command、crash-case-judge 各自现取（crash-point-replay 不转线程参数）。
    核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数取不了，都抛 InputManifestError。"""
    cores = machine_core_count(environment)
    configured = environment.get(CRASH_POINT_REPLAY_THREADS_VARIABLE, "")
    if not configured:
        if break_is_set("threads-default-from-cores"):
            return cores, cores, "default"
        return cores, registered_heavy_threads(environment), "default"
    if not POSITIVE_INTEGER_FORM.fullmatch(configured):
        raise InputManifestError(f"{CRASH_POINT_REPLAY_THREADS_VARIABLE}={configured} 不是正整数")
    return cores, int(configured), "explicit"


def crash_case_launch(root, case, environment, share=None, stage=None):
    """跑一条崩溃枚举用例的命令与环境（门禁 crash-point-replay 原样执行）：返回 (本机核数, 线程数, "explicit" 或 "default", 整条命令的词)。
    库：判定库只有 prune-server 那一份，用例里的引擎经客户端读写，命令不设库的变量，调用方环境里的跑法变量（SINGLEFS_CRASH_LIBRARY_RUN_KIND）清掉
    （接口在文件头「崩溃枚举用例的复用不在这里」）。领哪一份块（SINGLEFS_CRASH_AMPLIFICATION_SHARE）只由 share 定：给了（门禁的一片，「<片号>/<片数>」）设成它，
    没给（合并那一步的核账，整份）清掉调用方环境里的；给定状态引擎的 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与
    SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS 同样一律清掉（弄坏开关 keep-caller-stage-variables 下不清）；
    stage（--stage：record 只录、judge 只判）给了就在清掉之后把 SINGLEFS_CRASH_AMPLIFICATION_STAGE 设成它（弄坏开关 case-stage-not-set 下不设）。线程数设进 SINGLEFS_CRASH_POINT_REPLAY_THREADS，这条用例读的线程变量
    （登记行的 threads-variable=）不是它时另设成同一个数，盖掉调用方环境里的（弄坏开关 thread-variable-ignored 下只设 SINGLEFS_CRASH_POINT_REPLAY_THREADS）。
    cargo test 带的特性：登记行的 features=，多机配置写了 ENABLE_GPU=1 的一律加 gpu（崩溃枚举用例的判段也上卡）；env 里一律另设 BINDGEN_EXTRA_CLANG_ARGS（验证包常编 librocksdb-sys）；
    包是验证包的另带 --no-default-features（不编装置二进制，弄坏开关 verification-default-features-kept 下不带）；
    上卡的用例（crash_case_cargo_features）不经清场包装（卡与清场归 GPU 批处理验证服务），环境里交调用方环境里那一份 GPU_CARDS_BY_PRIORITY（没有就不设）与多机配置的路径；
    弄坏开关 gpu-case-quiesced-by-itself 下整条前面接清场包装，gpu-cards-from-configuration 下从多机配置取显卡先后。
    测试参数不带 --test-threads，GPU 开关都一样：测试线程照服务交下来的 RUST_TEST_THREADS（.claude/rules/verification.md 里 GPU 那一条；弄坏开关 gpu-test-threads-one 下特性里有 gpu 的带 --test-threads 1）。
    线程数取不到、多机配置在而读不了抛 InputManifestError。"""
    machine_cores, threads, threads_origin = crash_case_worker_threads(case, environment)
    thread_settings = [f"{CRASH_POINT_REPLAY_THREADS_VARIABLE}={threads}"]
    if case.thread_variable != CRASH_POINT_REPLAY_THREADS_VARIABLE:
        thread_settings.append(f"{case.thread_variable}={threads}")
    # env 的 -u 要写在第一个 NAME=VALUE 之前（之后的 -u 被当成要起的命令）
    share_unsets = ["-u", CRASH_SHARE_VARIABLE]
    if share is not None:
        thread_settings.append(f"{CRASH_SHARE_VARIABLE}={share}")
    if stage is not None and not break_is_set("case-stage-not-set"):
        thread_settings.append(f"{CRASH_STAGE_VARIABLE}={stage}")
    # 给定状态引擎读的阶段与各台权重：调用方环境里漏进 record 就只录不判（块账 blocks=0），漏进权重就按权重把表分给别的机器；弄坏开关 keep-caller-stage-variables 下不清
    if not break_is_set("keep-caller-stage-variables"):
        share_unsets += ["-u", CRASH_STAGE_VARIABLE, "-u", CRASH_HOST_WEIGHTS_VARIABLE]
    library_unsets = ["-u", CRASH_LIBRARY_RUN_KIND_VARIABLE]
    features, on_card, configuration_file = crash_case_cargo_features(root, case, environment)
    feature_words = ["--features", ",".join(features)] if features else []
    bindgen_setting = bindgen_include_setting(root)
    card_settings = []
    if on_card:
        if break_is_set("gpu-cards-from-configuration"):   # 弄坏开关：从多机配置取显卡先后
            cards = (configuration_value(configuration_file, "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED")
                     or configuration_value(configuration_file, "GPU_CARDS_BY_PRIORITY"))
        else:
            cards = environment.get(SERVICE_CARD_VARIABLE, "")
        card_settings = ([f"{SERVICE_CARD_VARIABLE}={cards}"] if cards else [])
        if os.path.isfile(configuration_file):
            card_settings.append(f"{MULTI_HOST_CONFIGURATION_VARIABLE}={os.path.abspath(configuration_file)}")
    # 弄坏开关 gpu-test-threads-one 下特性里有 gpu 的带 --test-threads 1
    test_threads_words = ["--test-threads", "1"] if GPU_FEATURE in features and break_is_set("gpu-test-threads-one") else []
    # 验证包的集成测试目标：cargo 给它编目标时把本包 src/bin/ 下的装置二进制全编一遍，那些二进制挂缺省开着的特性 apparatus-binaries，关掉缺省特性就不编
    #   （crates/singlefs-verification/Cargo.toml 的 [features]）。别的包不带。弄坏开关 verification-default-features-kept 下不带
    default_feature_words = ["--no-default-features"] if case.package == VERIFICATION_PACKAGE and not break_is_set("verification-default-features-kept") else []
    command = ["env", *share_unsets, *library_unsets, *thread_settings, *bindgen_setting, *card_settings,
               "cargo", "test", "--release", "-p", case.package, *default_feature_words, *feature_words, "--test", case.target,
               "--", "--include-ignored", "--exact", case.function, "--nocapture", *test_threads_words]
    if on_card and break_is_set("gpu-case-quiesced-by-itself"):   # 弄坏开关：整条经清场包装起
        command = ["bash", os.path.join(root, GPU_QUIESCE_SCRIPT), "--", *command]
    return machine_cores, threads, threads_origin, command


def crash_case_cargo_features(root, case, environment):
    """这条用例的 cargo 特性：返回 (特性列表, 是否上卡, 多机配置文件的路径)。
    没登记 features= 又没开 GPU 的是空列表、不上卡；登记了的照登记；多机配置里 ENABLE_GPU 等于 1 的一律另加 gpu（崩溃枚举用例的判段也上卡），特性里有 gpu 的（ENABLE_GPU 等于 1）上卡。
    弄坏开关 features-not-passed：登记的 features= 不认；features-passed-to-every-case：没登记的也带一项占位特性 UNREGISTERED_FEATURE_OF_BREAK。"""
    features = list(case.features)
    if break_is_set("features-not-passed"):
        features = []
    elif break_is_set("features-passed-to-every-case") and not features:
        features = [UNREGISTERED_FEATURE_OF_BREAK]
    configuration_file = multi_host_configuration_file(root, environment)
    gpu_enabled = configuration_value(configuration_file, "ENABLE_GPU") == "1"
    if gpu_enabled and GPU_FEATURE not in features:
        features.append(GPU_FEATURE)
    on_card = gpu_enabled and GPU_FEATURE in features
    return features, on_card, configuration_file


def bindgen_include_setting(root):
    """librocksdb-sys 的 bindgen 找头文件的目录：崩溃放量那份库（BINDGEN_LIBRARY_SCRIPT）里 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS 的值，打成 env 的一个词；库不在、读不到是空（照调用方环境）。"""
    try:
        with open(os.path.join(root, BINDGEN_LIBRARY_SCRIPT), encoding="utf-8") as handle:
            found = BINDGEN_ASSIGNMENT_FORM.search(handle.read())
    except (OSError, UnicodeDecodeError):
        return []
    return [f"{BINDGEN_VARIABLE}={found.group(1)}"] if found else []


# ── 崩溃枚举用例的块判定（CRASH_CASE_BLOCKS 那一行；文件头「崩溃枚举用例的复用不在这里」）──────────────

def judge_crash_case_blocks(log_text, ran=False, no_line_registered=False, recorded_only=False):
    """日志里的 CRASH_CASE_BLOCKS 行：交 (退出码, 一句)。退出码见文件头 crash-case-blocks。
    recorded_only（--recorded-only，只录的那一趟）：一行都没有、或每一行现判 0、红 0 而数对得上，交 (0, 「只录：已录、待判」那一句)；录入段判了块、缺字段、数对不上退 4。
    弄坏开关 recorded-only-judged 下不认它，照平常跑判。
    多流的用例每条流各打一行：逐行认（每一行 blocks = reused + fresh + refused），按用例合计再判。
    ran（--ran，跑登记了 red-blocks=judged-by-test 的用例）：只核行在、数对得上，红块与现判块不判（用例的退出码由门禁判）。
    no_line_registered（--no-line-registered，跑登记了 blocks-line=not-printed-by-the-case 的用例）：一行都没有退 0，有的照判；
    弄坏开关 blocks-line-required-when-marked 下一行都没有照退 4。
    弄坏开关 blocks-single-line-only 下要恰好一行，blocks-count-without-refused 下核 blocks = reused + fresh；
    ran-blocks-line-optional 下 --ran 一行都没有也退 0。
    按用例合计 blocks == 0（只录、配了 GPU 而没编进 gpu 特性、全部表留给别的机器，引擎都打 blocks=0）不算判过：退 4；
    弄坏开关 zero-blocks-covered 下放行。"""
    lines = [line.strip() for line in log_text.split("\n") if line.strip().startswith(CRASH_CASE_BLOCKS_PREFIX)]
    if recorded_only and not break_is_set("recorded-only-judged"):
        return judge_recorded_only_blocks(lines)
    if ran and not lines and break_is_set("ran-blocks-line-optional"):
        return 0, "块那一行没有（弄坏开关 ran-blocks-line-optional：没有也当对得上）"
    if not lines and no_line_registered and not break_is_set("blocks-line-required-when-marked"):
        return 0, f"块那一行：这条用例登记了 blocks-line={CRASH_CASE_BLOCKS_LINE_NOT_PRINTED}，日志里没有这一行不判，判定是 cargo test 的退出码"
    if not lines or (len(lines) != 1 and break_is_set("blocks-single-line-only")):
        return EXIT_BLOCKS_LINE_BAD, f"日志里以「{CRASH_CASE_BLOCKS_PREFIX.strip()} 」开头的行读到 {len(lines)} 行（每条流一行，至少一行）"
    counts = dict.fromkeys(CRASH_CASE_BLOCKS_FIELDS, 0)
    for line in lines:
        fields = fields_of_line(line)
        try:
            line_counts = {name: int(fields[name]) for name in CRASH_CASE_BLOCKS_FIELDS}
        except (KeyError, ValueError):
            return EXIT_BLOCKS_LINE_BAD, f"{line} 缺 {'、'.join(CRASH_CASE_BLOCKS_FIELDS)} 之一，或不是整数"
        accounted = line_counts["reused"] + line_counts["fresh"] + (0 if break_is_set("blocks-count-without-refused") else line_counts["refused"])
        if line_counts["blocks"] != accounted and not break_is_set("blocks-count-unchecked"):
            return EXIT_BLOCKS_LINE_BAD, f"{line}：blocks 不等于 reused + fresh + refused"
        for name in CRASH_CASE_BLOCKS_FIELDS:
            counts[name] += line_counts[name]
    summary = (f"块 {counts['blocks']}{f'（{len(lines)} 条流合计）' if len(lines) > 1 else ''}：复用 {counts['reused']}、"
               f"现判 {counts['fresh']}、红 {counts['red']}、拒算 {counts['refused']}")
    if counts["blocks"] == 0 and not break_is_set("zero-blocks-covered"):
        return EXIT_BLOCKS_LINE_BAD, f"{summary}：一块都没判也没查，没判过任何块（给定状态引擎只录、没编进 gpu 特性或表全留给别的机器时打 blocks=0）"
    if ran:
        return 0, summary
    if counts["red"]:
        return EXIT_BLOCKS_RED, summary
    return 0, summary


def judge_recorded_only_blocks(lines):
    """只录那一趟的块那一行（judge_crash_case_blocks 的 recorded_only）：没有行、或每一行现判 0、红 0 而数对得上退 0（已录、待判）；别的退 4。"""
    for line in lines:
        fields = fields_of_line(line)
        try:
            line_counts = {name: int(fields[name]) for name in CRASH_CASE_BLOCKS_FIELDS}
        except (KeyError, ValueError):
            return EXIT_BLOCKS_LINE_BAD, f"只录：{line} 缺 {'、'.join(CRASH_CASE_BLOCKS_FIELDS)} 之一，或不是整数"
        if line_counts["blocks"] != line_counts["reused"] + line_counts["fresh"] + line_counts["refused"]:
            return EXIT_BLOCKS_LINE_BAD, f"只录：{line}：blocks 不等于 reused + fresh + refused"
        if line_counts["fresh"] or line_counts["red"]:
            return EXIT_BLOCKS_LINE_BAD, f"只录：{line}：只录的那一趟判了块（现判 {line_counts['fresh']}、红 {line_counts['red']}），录入段不该判"
    return 0, f"只录：已录、待判（块那一行 {len(lines)} 行，现判 0、红 0；判段那一趟 --stage judge 再判块）"


def command_crash_case_blocks(arguments):
    ran = "--ran" in arguments
    no_line_registered = "--no-line-registered" in arguments
    recorded_only = "--recorded-only" in arguments
    arguments = [argument for argument in arguments if argument not in ("--ran", "--no-line-registered", "--recorded-only")]
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py crash-case-blocks <项目根> <日志> [--ran] [--no-line-registered] [--recorded-only]")
        print("     → 怎么办：日志取那一趟用例的全量输出；跑登记了 red-blocks=judged-by-test 的用例那一趟带 --ran，"
              "登记了 blocks-line=not-printed-by-the-case 的那一趟带 --no-line-registered，只录的那一趟（crash-case-command --stage record）带 --recorded-only")
        return EXIT_REGISTRATION_ERROR
    _root, log_file = arguments
    try:
        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
            log_text = handle.read()
    except OSError as error:
        print(f"读不了日志 {log_file}：{error}")
        return EXIT_BLOCKS_LINE_BAD
    exit_code, summary = judge_crash_case_blocks(log_text, ran, no_line_registered, recorded_only)
    print(summary)
    return exit_code


def command_crash_cases(arguments):
    """核登记的崩溃枚举用例，逐条打「键、包、测试目标、用例函数」；带 --separate-run 只给登了 separate-run= 的那几条打「键、理由」。
    登记有错退 2，原因不带 ✗（阶段自己打 ✗ 与出路）。"""
    separate_run_only = len(arguments) == 2 and arguments[1] == "--separate-run"
    if len(arguments) != 1 and not separate_run_only:
        print("  ✗ 用法：admission.py crash-cases <项目根> [--separate-run]")
        print("     → 怎么办：给项目根一个参数；要登了 separate-run= 的那几条，项目根之后另加 --separate-run")
        return EXIT_REGISTRATION_ERROR
    root = arguments[0]
    rows = read_registration_rows(root)
    crash_case_rows = [row for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]
    problems = [problem for problem in lint_registration_rows(crash_case_rows, root)]
    if problems:
        for problem in problems:
            print(f"{REGISTRATION_TABLE} {problem}")
        return EXIT_REGISTRATION_ERROR
    for case in crash_cases_of(crash_case_rows):
        if not separate_run_only:
            print(f"{case.key}\t{case.package}\t{case.target}\t{case.function}")
        elif case.separate_run_reason:
            print(f"{case.key}\t{case.separate_run_reason}")
    return 0

def option_values(options, names):
    """--名字 值 成对的参数：返回 {名字: 值}；有不认得的、缺值的返回 None。"""
    values = {}
    index = 0
    while index < len(options):
        if options[index] not in names or index + 1 >= len(options):
            return None
        values[options[index]] = options[index + 1]
        index += 2
    return values


def forwarded_worker_threads(arguments):
    """弄坏开关 judge-takes-forwarded-threads 下 crash-case-judge 照旧收的 --machine-cores / --threads / --threads-origin
    （crash-point-replay 转过来的）：交 (核数, 线程数, 来源) 或 None。开关没开时交 None、调用方不认这三个参数。"""
    if not break_is_set("judge-takes-forwarded-threads"):
        return None
    values = option_values(arguments, ("--machine-cores", "--threads", "--threads-origin"))
    if not values or set(values) != {"--machine-cores", "--threads", "--threads-origin"}:
        return None
    return int(values["--machine-cores"]), int(values["--threads"]), values["--threads-origin"]


def command_crash_case_judge(arguments):
    """判跑那一条的日志：核数、线程数与它是不是显式设的由这里现取（crash_case_worker_threads，与 crash-case-command 交给用例的同一个算法、
    同一份环境），不收调用方转过来的。"""
    forwarded = forwarded_worker_threads(arguments[4:]) if len(arguments) > 4 else None
    if len(arguments) != 4 and forwarded is None:
        print("  ✗ 用法：admission.py crash-case-judge <项目根> <键> <日志> <记录行文件>")
        print("     → 怎么办：核数与线程数由它自己现取（与 crash-case-command 同一份环境），不再带 --machine-cores / --threads / --threads-origin")
        return EXIT_REGISTRATION_ERROR
    root, key, log_file, recorded_file = arguments[:4]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
            log_text = handle.read()
        machine_cores, threads, threads_origin = forwarded or crash_case_worker_threads(case, os.environ)
        default_threads = registered_heavy_threads(os.environ)
    except (RegistrationError, InputManifestError, OSError) as error:
        print(f"判不了 {key} 的日志：{error}")
        return EXIT_REGISTRATION_ERROR
    threads_explicitly_one = threads_origin == "explicit" and threads == 1
    narrowing = crash_case_narrowing_settings(os.environ)
    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one, default_threads, narrowing)
    if problems:
        for problem in problems:
            print(problem)
        return 1
    recorded_lines.append("configured_worker_threads=" + configured_worker_threads_text(case, machine_cores, threads, threads_origin))
    with open(recorded_file, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(line + "\n" for line in recorded_lines))
    for note in thread_notes:
        print(note)
    if narrowing:
        for prefix in case.exhaustive_lines:
            print(f"{prefix} 那一行不判 exhaustive=true：这一趟缩过域、不算门禁通过（{'、'.join(narrowing)}）")
    return 0


ROCKSDB_PREBUILT_SCRIPT = os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts", "rocksdb-prebuilt.py")


def rocksdb_entry_problem(root):
    """crash-case-command 交命令之前的 RocksDB 预编库取值判（scripts/hosts/rocksdb-prebuilt.py entry --root <root>）：判得过（连同它报不判的两种：树里没有依赖 rocksdb 的清单、
    PATH 上是带标记的假 cargo）交 None；判红交它打的红句（几行并成一行）；判法脚本不在（这一份被拷到不带 scripts/hosts/ 的树里）交 None 并在 stderr 说一句。"""
    if not os.path.isfile(ROCKSDB_PREBUILT_SCRIPT):
        say(f"  ! RocksDB 预编库的取值没判：{ROCKSDB_PREBUILT_SCRIPT} 不在（这一份准入模块被拷到了不带 scripts/hosts/ 的树里）")
        return None
    completed = subprocess.run([sys.executable, ROCKSDB_PREBUILT_SCRIPT, "entry", "--root", root], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if completed.returncode == 0:
        return None
    return " ".join(line.strip() for line in (completed.stdout + completed.stderr).splitlines() if line.strip())


def command_crash_case_command(arguments):
    """跑一条崩溃枚举用例的命令与环境（crash_case_launch），stdout 以 NUL 分隔：本机核数、线程数、explicit 或 default，其后是整条命令的词。
    有错退 2，原因打在 stdout（不带 ✗，crash-point-replay 自己打 ✗ 与出路）。"""
    share = stage = None
    options = arguments[2:]
    while len(options) >= 2 and options[0] in ("--share", "--stage") and (share if options[0] == "--share" else stage) is None:
        if options[0] == "--share":
            share = options[1]
        else:
            stage = options[1]
        options = options[2:]
    if len(arguments) < 2 or options:
        print("  ✗ 用法：admission.py crash-case-command <项目根> <键> [--share <片号>/<片数>] [--stage record|judge]")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写；--share 只由门禁 crash-point-replay 的一片给，"
              "--stage 只由它先录后判的那两遍给；两个选项各至多一次")
        return EXIT_REGISTRATION_ERROR
    if share is not None:
        match = CRASH_CASE_SHARE_FORM.fullmatch(share)
        if not match or int(match.group(1)) >= int(match.group(2)):
            print(f"  ✗ --share 要写成 <片号>/<片数>（十进制，片号小于片数），收到「{share}」")
            print("     → 怎么办：照门禁 crash-point-replay 那一片的 TASK_SERVER_SHARD 写")
            return EXIT_REGISTRATION_ERROR
    if stage is not None and stage not in CRASH_CASE_STAGES:
        print(f"  ✗ --stage 只认 {'、'.join(CRASH_CASE_STAGES)}（record 只录、judge 只判），收到「{stage}」")
        print("     → 怎么办：照 admission.py 文件头 crash-case-command 那一条 --stage 的写法写（record 只录、judge 只判）")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments[:2]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        machine_cores, threads, threads_origin, command = crash_case_launch(root, case, os.environ, share, stage)
    except (RegistrationError, InputManifestError) as error:
        print(f"交不出 {key} 的命令：{error}")
        return EXIT_REGISTRATION_ERROR
    # 交的命令起 cargo test（验证包的崩溃枚举用例，常在依赖 prune-server 编到 librocksdb-sys）：交之前判一次这台的 RocksDB 预编库取值
    #   （scripts/hosts/rocksdb-prebuilt.py 文件头「entry」），判红不交命令、退 2，门禁一个 cargo 都不起。弄坏开关 rocksdb-entry-unchecked 下不判
    problem = None if break_is_set("rocksdb-entry-unchecked") else rocksdb_entry_problem(root)
    if problem:
        print(f"交不出 {key} 的命令：起 cargo 之前 RocksDB 预编库的取值没判过（rocksdb-prebuilt.py entry 的红句：{problem}）")
        return EXIT_REGISTRATION_ERROR
    words = [str(machine_cores), str(threads), threads_origin, *command]
    sys.stdout.write("".join(word + "\0" for word in words))
    return 0



def command_crash_case_shards_at_least(arguments):
    if not arguments:
        print("  ✗ 用法：admission.py crash-case-shards-at-least <项目根> [<键>…]")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写；不给键是全部登记的")
        return EXIT_REGISTRATION_ERROR
    root, keys = arguments[0], arguments[1:]
    try:
        rows = read_registration_rows(root)
        cases = [crash_case_of_key(rows, key) for key in keys] if keys else crash_cases_of(rows)
    except RegistrationError as error:
        print(f"答不出至少分几片：{error}")
        return EXIT_REGISTRATION_ERROR
    print(max((case.shards_at_least for case in cases), default=0))
    return 0


def command_crash_case_judged_by_test(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py crash-case-judged-by-test <项目根> <键>")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    if case.red_blocks_judged_by_test:
        print(f"{key} 登记了 red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}：起用例按退出码判")
        return 0
    print(f"{key} 没登记 red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}：有红块判红")
    return 1


def answer_crash_case_mark(subcommand, arguments, attribute, mark):
    """crash-case-findings-not-written、crash-case-blocks-not-printed 共用的答法：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错。"""
    if len(arguments) != 2:
        print(f"  ✗ 用法：admission.py {subcommand} <项目根> <键>")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    if getattr(case, attribute):
        print(f"{key} 登记了 {mark}")
        return 0
    print(f"{key} 没登记 {mark}")
    return 1


def command_crash_case_findings_not_written(arguments):
    return answer_crash_case_mark("crash-case-findings-not-written", arguments, "findings_not_written_by_the_case", f"findings={CRASH_CASE_FINDINGS_NOT_WRITTEN}")


def command_crash_case_blocks_not_printed(arguments):
    return answer_crash_case_mark("crash-case-blocks-not-printed", arguments, "blocks_line_not_printed_by_the_case", f"blocks-line={CRASH_CASE_BLOCKS_LINE_NOT_PRINTED}")
