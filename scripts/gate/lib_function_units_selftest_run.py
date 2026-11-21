"""自证的并行与编排：各格在 fork 出来的子进程里跑（gather_forked）、格表（selftest_cells、silent_form_cells）、自证夹具的载入与 selftest 本身。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import collections
import gc
import importlib.util
import os
import pickle
import signal
import sys
import tempfile
import time
import traceback
from lib_function_units_lexer import BREAK_VARIABLE, RUST_LEX_CACHE_DIRECTORY_VARIABLE, active_breaks
from lib_function_units_base import COMMAND_FORM, FOLD_LIMIT_FORM, FunctionUnitsError, INLINE_FORM, LIBRARY_ROOT, UsageError, c_library_calls_allowed_in_extern_blocks
from lib_function_units_python_facts import DATA_FORM
from lib_function_units_selftest_fixture_trees import (
    OPEN_TAIL_FIRST_WORD_CELL, OPEN_TAIL_PYTHON_CELL, OPEN_TAIL_SKIPPED_CELL, SHELL_ARGUMENTS_PREFLIGHT_CELL, SHELL_ARGUMENTS_SKIPPED_CELL,
    SHELL_ENTRY_ARGUMENTS_CELL, SHELL_ENTRY_EXIT_TABLE_CELL, SHELL_ENTRY_TABLE_CELL, SHELL_FOLDS_FIXTURE, SHELL_PYTHON_ARGUMENTS_OPEN_CELL,
    SHELL_PYTHON_ARGUMENTS_SKIPPED_CELL, SHELL_TABLE_FIXTURE, SHELL_TABLE_SELFTEST_ENTRY_CELL, SHELL_TABLE_SELFTEST_FIXTURE, TAIL_VIEW_CELL,
    compare_cell, heavy_threads_module, refused_cell, rust_doctest_cell, rust_dump_cell, rust_string_list_cell,
)
from lib_function_units_selftest_rust_cells import (
    alignment_cells, closure_edge_cells, closure_rule_cells, current_exe_cell, current_exe_constant_cell, current_exe_path_only_cell,
    current_exe_spawn_cell, excluded_call_edge_cell, manifest_binding_cell, manifest_directory_cell, manifest_root_cell, python_rta_cell,
    python_spec_cell, rust_lex_cache_cell, rust_lex_namespace_cell, rust_lex_namespace_closure_cell, rust_unbalanced_cell,
    selftest_fixture_names_cell, selftest_fixtures_outside_cell, shell_data_dot_cell, shell_data_nested_cell, shell_data_source_cell,
    shell_folding_cell, shell_heredoc_cell, shell_names_cell, unit_divergence_cells, unit_graph_digest_cell, unit_graph_inputs_cell,
    wgsl_undefined_cell,
)
from lib_function_units_selftest_program_registries import (
    HEREDOC_BLANK_ROW_CELL, built_tool_launch_cell, caller_supplied_registered_cell, caller_supplied_unregistered_cell, directory_closure_line_cell,
    inline_refusal_line_cell, inline_sourced_body_cell, launched_programs_cell, launched_shell_cell, module_object_refusal_lines_cell, narrowed_view_shared_cell,
    nested_program_words_registered_cell, program_words_qualified_cell, test_service_code_entry_unkeyed_cell,
    nested_program_words_unregistered_cell, program_words_cell, refusal_lines_cell, refusal_word_lines_cell, registered_inline_launch_cell,
    registered_missing_path_cell,
    registered_functions_real_cell, registered_words_real_cell, runtime_data_files_cell, spec_path_arguments_cell, stage_cell_functions_real_cell,
    tail_read_root_scope_cell, tail_read_scopes_cell,
    tail_read_scopes_real_cell, tail_reads_records_real_cell, test_service_code_real_cell, test_service_code_table_cell, test_service_code_unkeyed_cell,
    tool_source_cell, unit_graph_failed_step_cell, unit_graph_present_program_cell, unit_graph_shared_directories_cell, unit_graph_text_busy_cell,
)
from lib_function_units_selftest_closure_rules import (
    ARGUMENTS_FIXTURE, caller_launch_cell, closure_round_limit_cell, excluded_file_cell, extern_build_script_link_cell, extern_link_beyond_cell,
    extern_listed_cell, extern_refused_cell, extern_refused_cell_for, judging_closure_cell, key_external_cell, key_toolchain_cell,
    launch_argument_cell, loaded_attributes_cell, no_external_program_cell, no_refusal_cell, no_refusal_no_toolchain_query_cell, not_judging_files_cell,
    own_program_entry_cell, own_program_keyed_cell, own_program_path_cell, own_program_refused_cell, preflight_loader_cell, preflight_worktree_cell,
    program_arguments_cell, registry_programs_cell, row_keyed_cell, shared_script_index_cell, shell_entry_edits_cell, shell_entry_exit_table_cell,
    shell_entry_table_cell, shell_table_copy_cell, shell_table_real_cell, shell_table_tree_copy_cell, tail_view_cell, toolchain_query_cell,
    workspace_dot_member_cell, workspace_multiline_members_cell, workspace_root_with_brackets_cell, workspace_same_named_directories_cell,
)
from lib_function_units_selftest_real_repo import (
    cross_run_cache_cell, cross_run_cache_scan_cell, entry_lines_rust_keys_cell, entry_lines_shape_cell, entry_unit_key_cell, import_time_cells,
    product_view_cells, selftest_modules_outside_cell, unit_graph_parity_cells,
)


# ── 自证的并行：每条活在自己 fork 出来的子进程里跑（gather_forked） ─────────────────

ForkedOutcome = collections.namedtuple("ForkedOutcome", "value raised stdout stderr")
LostTask = collections.namedtuple("LostTask", "index reason stdout stderr")


def gather_forked(tasks, jobs, run_one):
    """tasks 每条在自己 fork 出来的子进程里跑 run_one(task)，同时至多 jobs 个：模块级缓存、环境变量、登记表的临时改动都是那个子进程自己的一份，
    不带到别的条、不带回父进程；临时目录各建各的。子进程的标准输出与标准错误（连同它起的子进程写的）各写进一份文件，
    交回值或抛出的回溯 pickle 进 <下标>.result（forked_child），退出码 0 且读得出这一份才算收回。
    → (outcomes, lost)：outcomes 按 tasks 的次序，收回的是 ForkedOutcome（raised 是抛出时的回溯文本，没抛是 None），没收回的是 None；
    lost 是没收回的每一条 LostTask（原因与它写下的输出），派出去的条数与收回的对不上它就不空，调用方整份判红。
    父进程这一段抛了（中断）：只停自己 fork 的那几个进程号、收完尸再往外抛。
    弄坏开关 selftest-lost-results-unchecked：不比条数，lost 总是空的（没收回的那几条只在 outcomes 里是 None）。"""
    statuses, running, position, refused_fork = {}, {}, 0, None
    with tempfile.TemporaryDirectory(prefix="function-units-forked-") as directory:
        sys.stdout.flush()
        sys.stderr.flush()
        gc.collect()
        gc.freeze()   # 子进程共用 fork 之前的对象，回收不逐页抄
        try:
            while (position < len(tasks) and refused_fork is None) or running:
                while position < len(tasks) and refused_fork is None and len(running) < max(1, jobs):
                    try:
                        pid = os.fork()
                    except OSError as error:
                        refused_fork = f"fork 不出子进程：{error}"
                        break
                    if pid == 0:
                        forked_child(directory, position, run_one, tasks[position])
                    running[pid] = position
                    position += 1
                if running:
                    pid, status = os.waitpid(-1, 0)
                    if pid in running:
                        statuses[running.pop(pid)] = status
        finally:
            gc.unfreeze()
            for pid in running:
                os.kill(pid, signal.SIGTERM)
            for pid in running:
                os.waitpid(pid, 0)
        outcomes, lost = [], []
        for index in range(len(tasks)):
            outcome, missing = read_forked_outcome(directory, index, statuses.get(index), refused_fork)
            outcomes.append(outcome)
            if missing is not None:
                lost.append(missing)
    if "selftest-lost-results-unchecked" in active_breaks():
        return outcomes, []
    return outcomes, lost


def forked_child(directory, index, run_one, task):
    """gather_forked 的子进程：标准输出与标准错误（文件描述符 1、2）改到 <下标>.stdout、<下标>.stderr，跑 run_one(task)，
    (交回值, None) 或 (None, 回溯) pickle 进 <下标>.result（先写 .part 再改名），然后 os._exit：不回到父进程的调用栈、不跑父进程的 finally 与 atexit。"""
    code = 1
    try:
        for descriptor, suffix in ((1, "stdout"), (2, "stderr")):
            target = os.open(os.path.join(directory, f"{index}.{suffix}"), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            os.dup2(target, descriptor)
            os.close(target)
        try:
            record = pickle.dumps((run_one(task), None))
        except BaseException:   # 交回值 pickle 不了的也落在这里：照抛了记
            record = pickle.dumps((None, traceback.format_exc()))
        partial = os.path.join(directory, f"{index}.result.part")
        with open(partial, "wb") as handle:
            handle.write(record)
        os.replace(partial, os.path.join(directory, f"{index}.result"))
        code = 0
    finally:
        try:
            sys.stdout.flush()
            sys.stderr.flush()
        finally:
            os._exit(code)


def read_forked_outcome(directory, index, status, refused_fork):
    """第 index 条的结局 → (ForkedOutcome, None) 或 (None, LostTask)。status 是 waitpid 交的状态，没派出去的是 None。"""
    streams = []
    for suffix in ("stdout", "stderr"):
        try:
            with open(os.path.join(directory, f"{index}.{suffix}"), "rb") as handle:
                streams.append(handle.read())
        except OSError:
            streams.append(b"")
    if status is None:
        return None, LostTask(index, f"没派出去：{refused_fork}", *streams)
    if os.WIFSIGNALED(status):
        return None, LostTask(index, f"子进程被信号 {os.WTERMSIG(status)} 打断", *streams)
    if os.waitstatus_to_exitcode(status) != 0:
        return None, LostTask(index, f"子进程退出码 {os.waitstatus_to_exitcode(status)}", *streams)
    try:
        with open(os.path.join(directory, f"{index}.result"), "rb") as handle:
            value, raised = pickle.load(handle)
    except FileNotFoundError:
        return None, LostTask(index, "子进程退出码 0 却没写结果", *streams)
    except (OSError, pickle.UnpicklingError, EOFError, ValueError, TypeError) as error:
        return None, LostTask(index, f"结果读不出：{type(error).__name__}：{error}", *streams)
    return ForkedOutcome(value, raised, *streams), None


def forked_gather_cell(work):
    """FORKED_GATHER_CELL：五条假活、同时至多 3 个。头一条睡 0.3 秒、最后才收回；第二条退出码 0 不写结果，第三条退出码 3，
    第四条往标准输出打一行，第五条抛 ValueError。work 不用（gather_forked 自己建临时目录）。"""
    def run_one(task):
        if task == "slow":
            time.sleep(0.3)
        elif task == "vanished":
            os._exit(0)
        elif task == "crashed":
            os._exit(3)
        elif task == "printed":
            print("printed")
        elif task == "raised":
            raise ValueError("假活抛的")
        return task
    outcomes, lost = gather_forked(("slow", "vanished", "crashed", "printed", "raised"), 3, run_one)
    values = [None if outcome is None else outcome.value for outcome in outcomes]
    printed = outcomes[3].stdout if outcomes[3] is not None else None
    raised = outcomes[4].raised if outcomes[4] is not None else None
    reasons = {item.index: item.reason for item in lost}
    ok = (values == ["slow", None, None, "printed", None] and printed == b"printed\n" and raised is not None and "ValueError: 假活抛的" in raised
          and sorted(reasons) == [1, 2] and "没写结果" in reasons[1] and "退出码 3" in reasons[2])
    return ok, (f"收回的值 {values}（要 ['slow', None, None, 'printed', None]）、第四条的标准输出 {printed!r}、第五条抛出 {raised!r}、"
                f"没收回的 {reasons}（要第 1、2 条：退出码 0 没写结果、退出码 3）")


def run_selftest_task(task):
    """自证的一条活（gather_forked 的子进程里跑）：(开关, 格函数)。开关不是 None 的，在这个子进程里把 FUNCTION_UNITS_BREAK 换成它
    （只有这个进程与它起的子进程看得见，不用改回去）。"""
    switch, judge = task
    if switch is not None:
        os.environ[BREAK_VARIABLE] = switch
    return run_cell(judge)


def replay_forked_output(stdout, stderr):
    """子进程写下的输出照原样打到这个进程的标准输出与标准错误（在它那一格的结论行之前）。"""
    for stream, content in ((sys.stdout, stdout), (sys.stderr, stderr)):
        if content:
            stream.flush()
            stream.buffer.write(content)
            stream.buffer.flush()


def forked_cell_result(outcome, lost):
    """一条自证活的结局 → (通过, 说明, 跑完了)，先把它写下的输出打出来。没收回的（lost）与抛了 run_cell 不接的异常的：不通过、没跑完。
    没收回、又不在 lost 里的（弄坏开关 selftest-lost-results-unchecked 下）交 None。"""
    if lost is not None:
        replay_forked_output(lost.stdout, lost.stderr)
        return False, f"没收回结果：{lost.reason}", False
    if outcome is None:
        return None
    replay_forked_output(outcome.stdout, outcome.stderr)
    if outcome.raised is not None:
        return False, f"抛了 run_cell 不接的异常：\n{outcome.raised}", False
    ok, detail = outcome.value
    return ok, detail, True


def listed_pattern_cell(fixture, entry, matched, unmatched):
    """按名字模式列的清单那两格（PATTERN_LISTING_GLOB_CELL、PATTERN_LISTING_FIND_CELL）：改合模式的那一份（matched：路径、原文、新文）键要变，
    改同一目录里不合模式的那一份（unmatched）键要不变，两趟都不许拒算；各在自己的子目录里现造一棵树。"""
    checks = (("matched", "改合模式的那一份", compare_cell(fixture, entry, *matched, "changed")),
              ("unmatched", "改不合模式的那一份", compare_cell(fixture, entry, *unmatched, "same")))

    def judge(work):
        results = []
        for name, label, check in checks:
            directory = os.path.join(work, name)
            os.makedirs(directory)
            results.append((label, check(directory)))
        return all(ok for _label, (ok, _detail) in results), "；".join(f"{label}：{detail}" for label, (_ok, detail) in results)
    return judge


def selftest_cells():
    cells = []
    for name, entry, path, old, new, want in RUST_FIXTURE_CASES:
        cells.append((name, compare_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::{entry}", path, old, new, want)))
    cells += [
        ("改文档注释里的代码块：文档测试入口作废", rust_doctest_cell),
        ("认不出的 cfg 谓词拒算", refused_cell(RUST_FIXTURE, f"{RUST_FIXTURE_ENTRY_FILE}::judge_speed", "cfg-unrecognized",
                                         ("crates/fextra/src/lib.rs", '#[cfg(feature = "fast")]', "#[cfg(accessible(fast))]"))),
        ("括号配不上的文件整份拒算", rust_unbalanced_cell),
        (MANIFEST_ROOT_CELL, manifest_root_cell),
        (MANIFEST_DIRECTORY_CELL, manifest_directory_cell),
        (MANIFEST_BINDING_CELL, manifest_binding_cell),
        (SHELL_DATA_SOURCE_CELL, shell_data_source_cell),
        (SHELL_DATA_DOT_CELL, shell_data_dot_cell),
        (SHELL_DATA_NESTED_CELL, shell_data_nested_cell),
        (SHELL_DATA_UNFOLDED_CELL, refused_cell(SHELL_DATA_FIXTURE, f"{SHELL_DATA_FILE}::runs_the_unfolded_script", "shell-path-unfolded")),
        (CURRENT_EXE_CELL, current_exe_cell),
        (CURRENT_EXE_CONSTANT_CELL, current_exe_constant_cell),
        (CURRENT_EXE_PATH_ONLY_CELL, current_exe_path_only_cell),
        (CURRENT_EXE_SPAWN_CELL, current_exe_spawn_cell),
        (DIRECTORY_CLOSURE_LINE_CELL, directory_closure_line_cell),
        (RUST_LEX_CACHE_CELL, rust_lex_cache_cell),
        (RUST_LEX_NAMESPACE_CHANGED_CELL, rust_lex_namespace_cell(RUST_LEX_NAMESPACE_CHANGED_EDITS, "changed")),
        (RUST_LEX_NAMESPACE_KEPT_CELL, rust_lex_namespace_cell(RUST_LEX_NAMESPACE_KEPT_EDITS, "same")),
        (RUST_LEX_NAMESPACE_CLOSURE_CELL, rust_lex_namespace_closure_cell),
        (EXCLUDED_CALL_EDGE_CELL, excluded_call_edge_cell),
        (UNIT_GRAPH_INPUTS_CELL, unit_graph_inputs_cell),
        (UNIT_GRAPH_DIGEST_CELL, unit_graph_digest_cell),
        (SELFTEST_FIXTURES_OUTSIDE_CELL, selftest_fixtures_outside_cell),
        (SELFTEST_FIXTURE_NAMES_CELL, selftest_fixture_names_cell),
        (SELFTEST_MODULES_OUTSIDE_CELL, selftest_modules_outside_cell),
        (SHELL_SOURCED_OWNER_CELL, compare_cell(SHELL_SOURCED_FIXTURE, "g/owned_entry.sh", "tools/owned.py", "print(1)", "print(10)", "changed")),
        (SHELL_SOURCED_ZERO_CELL, compare_cell(SHELL_SOURCED_FIXTURE, "g/started_entry.sh", "tools/started.py", "print(2)", "print(20)", "changed")),
        (SHELL_SOURCED_SIBLING_CELL, compare_cell(SHELL_SOURCED_FIXTURE, "g/sibling_entry.sh", "tools/sibling.py", "print(3)", "print(30)", "changed")),
        (PYTHON_IMPORTER_SYS_PATH_CELL, compare_cell(PYTHON_IMPORTER_FIXTURE, "tool/entry_part.py::judge", "shared/helper.py", "    return 0\n",
                                                     "    return 1\n", "changed")),
        (PYTHON_IMPORT_BINDING_UNUSED_CELL, compare_cell(PYTHON_IMPORT_BINDING_FIXTURE, "tool/user.py::judge", "tool/helpers.py", "    return 2\n",
                                                         "    return 20\n", "same")),
        (PYTHON_IMPORT_BINDING_USED_CELL, compare_cell(PYTHON_IMPORT_BINDING_FIXTURE, "tool/user.py::judge", "tool/helpers.py", "    return 1\n",
                                                       "    return 10\n", "changed")),
        (SHELL_REACHED_LIBRARY_CELL, compare_cell(SHELL_REACHED_FIXTURE, "g/reach_entry.sh::reach_main", "g/lib/reach_second.sh", "echo second",
                                                  "echo second changed", "changed")),
        (PYTHON_IMPORTER_CALL_SITES_CELL, compare_cell(PYTHON_IMPORTER_CALL_SITES_FIXTURE, "tool/load.py::run", "tool/load_target.py", "return 1",
                                                       "return 2", "changed")),
        (PYTHON_IMPORTER_METHOD_SITES_CELL, compare_cell(PYTHON_IMPORTER_METHOD_SITES_FIXTURE, "tool/mixed.py::run", "tool/mixed_target.py", "return 1",
                                                         "return 2", "changed")),
        (COMPILED_PART_EXEC_CELL, refused_cell(COMPILED_PART_EXEC_FIXTURE, "tool/parted.py", "compiled-part-exec")),
        (COMPILED_PART_SYNTAX_CHECK_CELL, no_refusal_cell(COMPILED_PART_EXEC_FIXTURE, "tool/checked.py")),
        (MODULE_OBJECT_EXEC_TEXT_CELL, compare_cell(MODULE_OBJECT_EXEC_FIXTURE, "s/module_exec_text.sh", "s/exec_helper.sh", "echo helper",
                                                    "echo helper changed", "changed")),
        (MODULE_OBJECT_EXEC_FILE_CELL, compare_cell(MODULE_OBJECT_EXEC_FIXTURE, "s/module_exec_file.sh", "s/exec_file_helper.sh", "echo file helper",
                                                    "echo file helper changed", "changed")),
        (MODULE_OBJECT_EXEC_UNCALLED_CELL, compare_cell(MODULE_OBJECT_EXEC_FIXTURE, "s/module_exec_file.sh", "s/exec_unused.sh", "echo unused",
                                                        "echo unused changed", "same")),
        (MODULE_OBJECT_EXEC_UNFOLDED_CELL, refused_cell(MODULE_OBJECT_EXEC_FIXTURE, "s/module_exec_unfolded.sh", "compiled-part-exec")),
        (MODULE_OBJECT_PRINTED_TEXT_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/printed_library.sh", "s/printed_used.sh", "echo printed used",
                                                       "echo printed used changed", "changed")),
        (MODULE_OBJECT_PRINTED_UNCALLED_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/printed_library.sh", "s/printed_unused.sh",
                                                           "echo printed unused", "echo printed unused changed", "same")),
        (MODULE_OBJECT_PRINTED_UNFOLDED_CELL, refused_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/printed_unfolded.sh", "compiled-part-exec")),
        (MODULE_OBJECT_READ_TEXT_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/read_library.sh", "s/read_used.sh", "echo read used",
                                                    "echo read used changed", "changed")),
        (MODULE_OBJECT_READ_UNCALLED_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/read_library.sh", "s/read_unused.sh", "echo read unused",
                                                        "echo read unused changed", "same")),
        (MODULE_OBJECT_READ_UNFOLDED_CELL, refused_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/read_unfolded.sh", "compiled-part-exec")),
        (MODULE_OBJECT_READ_AFTER_LITERAL_CELL, refused_cell(MODULE_OBJECT_READ_AFTER_LITERAL_FIXTURE, "s/read_after_literal.sh", "compiled-part-exec")),
        (MODULE_OBJECT_LOADER_SITES_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/loader_sites.sh", "s/site_loaded.py", "VALUE = 1", "VALUE = 2",
                                                       "changed")),
        (MODULE_OBJECT_LOADER_SITES_OTHER_CELL, compare_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/loader_sites.sh", "s/site_described.py", "VALUE = 3",
                                                             "VALUE = 4", "same")),
        (MODULE_OBJECT_LOADER_SITES_UNFOLDED_CELL, refused_cell(MODULE_OBJECT_TEXT_SOURCES_FIXTURE, "s/loader_sites_unfolded.sh", "spec-path-unfolded")),
        (MODULE_OBJECT_REFUSAL_LINES_CELL, module_object_refusal_lines_cell),
        (PYTHON_IMPORTER_OUTSIDE_FAMILY_CELL, compare_cell(PYTHON_IMPORTER_OUTSIDE_FAMILY_FIXTURE, "tool/stage.py::run", "tool/second.sh", "echo second",
                                                           "echo second changed", "same")),
        ("WGSL 里没定义、Rust 串里也没有的名字拒算", wgsl_undefined_cell),
        ("dump 的单元行形态（带路径的宏、verus! 两个 fn、impl 头与方法）", rust_dump_cell),
        ("python：只改注释不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", "    # 只是注释\n", "    # 只是注释，改了措辞\n", "same")),
        ("python：改文档串不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/a.py", '"""helper 的文档串"""', '"""helper 的文档串，改了"""', "same")),
        ("python：有单元读 __doc__ 时改文档串作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::described", "pkg/a.py", '"""helper 的文档串"""',
                                                         '"""helper 的文档串，改了"""', "changed")),
        (DOCSTRING_WRITE_CELL, compare_cell(DOCSTRING_READS_FIXTURE, "pkg/docs.py::plain", "pkg/docs.py", '"""other 的文档串"""', '"""other 的文档串，改了"""', "same")),
        (DOCSTRING_AUGMENTED_CELL, compare_cell(DOCSTRING_READS_FIXTURE, "pkg/docs.py::augmented", "pkg/docs.py", '"""described 的文档串"""',
                                                '"""described 的文档串，改了"""', "changed")),
        (DOCSTRING_GETATTR_CELL, compare_cell(DOCSTRING_READS_FIXTURE, "pkg/docs.py::by_getattr", "pkg/docs.py", '"""described 的文档串"""',
                                              '"""described 的文档串，改了"""', "changed")),
        (DOCSTRING_GETDOC_CELL, compare_cell(DOCSTRING_READS_FIXTURE, "pkg/docs.py::by_getdoc", "pkg/docs.py", '"""other 的文档串"""', '"""other 的文档串，改了"""', "changed")),
        ("python：getattr 变量名拒算", refused_cell(PYTHON_FIXTURE, "pkg/a.py::dynamic", "getattr-variable")),
        ("python：globals 只查询 __file__ 不拒算", no_refusal_cell(
            {"sample.py": 'def entry():\n    return globals().get("__file__")\n'}, "sample.py::entry")),
        ("python：globals 动态查询仍拒算", refused_cell(
            {"sample.py": 'def entry(name):\n    return globals().get(name)\n'}, "sample.py::entry", "globals-lookup")),
        ("python：globals 查询其他固定名字仍拒算", refused_cell(
            {"sample.py": 'def entry():\n    return globals().get("handler")\n'}, "sample.py::entry", "globals-lookup")),
        ("python：globals()[固定名字] 读连到那个名字的单元，不拒算（改它键变）", compare_cell(
            {"sample.py": 'def helper():\n    return 5\n\n\ndef entry():\n    return globals()["helper"]()\n'}, "sample.py::entry", "sample.py", "return 5", "return 50", "changed")),
        ("python：globals()[折不出的名字] 读连本模块每个名字，不拒算（改其中一个键变）", compare_cell(
            {"sample.py": 'import os\n\n\ndef helper():\n    return 5\n\n\ndef entry():\n    return globals()[os.environ["NAME"]]()\n'}, "sample.py::entry", "sample.py",
            "return 5", "return 50", "changed")),
        ("python：globals()[键] = 值、del 与 globals().update(…) 只写不读：不连、不拒算", no_refusal_cell(
            {"sample.py": 'HOOK = None\n\n\ndef entry(replaced):\n    globals()["HOOK"] = len\n    globals().update(replaced)\n    del globals()["HOOK"]\n'}, "sample.py::entry")),
        ("python：getattr(<标准库模块>, 变量名) 不拒算（够不到仓里的代码）", no_refusal_cell(PYTHON_FIXTURE, "pkg/f.py::patched_os_function")),
        ("python：os.environ.get(…) or <缺省路径> 按缺省折，连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/h.py::stop", "pkg/c.py", "    return 5", "    return 50", "changed")),
        ("python：自己写出字面文本再起的脚本不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/h.py::run_stub")),
        ("python：for 循环变量取遍元组里的目录，插进 sys.path 不拒算、各目录里的同名模块都连", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_named", "pkg/tools/tool.py",
                                                                                       "    return 7", "    return 70", "changed")),
        ("python：推导式里的循环变量不算函数里的赋值，for 取遍元组拼出的脚本连上", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_each", "pkg/b.py", "    return 2",
                                                                           "    return 20", "changed")),
        ("python：os.path.join(<仓里的目录>, <折不出的文件名>) 起的脚本连到那个目录里每一份", compare_cell(PYTHON_FIXTURE, "pkg/k.py::run_tool_named", "pkg/tools/main_tool.py",
                                                                                  "    return 9", "    return 90", "changed")),
        ("python：路径表（字典字面量）[键] 取遍表里的路径，spec 载入连上", compare_cell(PYTHON_FIXTURE, "pkg/j.py::load_named", "pkg/b.py", "    return 2",
                                                                      "    return 20", "changed")),
        ("python：importlib.import_module(<模块级常量>) 折出模块名", compare_cell(PYTHON_FIXTURE, "pkg/i.py::load_by_constant", "pkg/more_tools/tool.py",
                                                                            "    return 8", "    return 80", "changed")),
        ("python：表[路径] = … 不当成重新绑定路径，spec 载入照样折得出", compare_cell(PYTHON_FIXTURE, "pkg/g.py::cached_b", "pkg/b.py", "    return 2", "    return 20",
                                                                    "changed")),
        ("python：spec_from_file_location 字面路径折叠出跨模块边", python_spec_cell),
        ("python：spec 载入的模块里够不到的函数改了不作废", compare_cell(PYTHON_FIXTURE, "pkg/a.py::entry", "pkg/b.py", "    return 3", "    return 30", "same")),
        ("python：subprocess 起 sys.executable 字面路径连到那份脚本的程序入口", compare_cell(PYTHON_FIXTURE, "pkg/a.py::launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        ("python：obj.方法 按闭包里出现过的类收窄", python_rta_cell),
        ("shell：here-doc 正文里有一行 `};` 不许被切开", shell_heredoc_cell),
        ("shell：函数名当参数传照样连上", shell_names_cell),
        ("shell：`bash \"$x\"` 折不出拒算", refused_cell(SHELL_FIXTURE, "s/dynamic.sh", "shell-path-unfolded")),
        ("shell：$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd) 折叠出 source 与 bash 的边", shell_folding_cell),
        ("shell：bash -n 不过的整份拒算", refused_cell(SHELL_FIXTURE, "s/broken.sh", "bash-syntax")),
        ("shell：只改注释不作废", compare_cell(SHELL_FIXTURE, "s/main.sh::cell_beta", "s/main.sh", "set -euo pipefail\n", "# 一行新注释\nset -euo pipefail\n", "same")),
        ("shell：脚本体只改注释不作废", compare_cell(SHELL_FIXTURE, "s/main.sh", "s/main.sh", "set -euo pipefail\n", "# 一行新注释\nset -euo pipefail\n", "same")),
        ("shell：顶层变量里 here-doc 的正文改了，读它的函数作废", compare_cell(SHELL_FIXTURE, "s/main.sh::use_library", "s/main.sh", 'print("library")',
                                                                'print("library changed")', "changed")),
        ("键：工具链行进键", key_toolchain_cell),
        ("键：闭包点名了 cargo 不拒算，外部程序的版本不进键：不给 externals 也算得出键", key_external_cell),
        (ENTRY_UNIT_KEY_CELL, entry_unit_key_cell),
        (ENTRY_LINES_SHAPE_CELL, entry_lines_shape_cell),
        (ENTRY_LINES_RUST_KEYS_CELL, entry_lines_rust_keys_cell),
        ("shell：`bash \"$tree/<仓里的相对路径>\"` 连到仓里那一份", compare_cell(SHELL_FIXTURE, "s/copies.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (TREE_COPY_EXACT_CELL, compare_cell(dict(SHELL_FIXTURE, **{"elsewhere/s/child.sh": "#!/usr/bin/env bash\necho elsewhere\n"}), "s/copies.sh", "elsewhere/s/child.sh",
                                            "echo elsewhere", "echo elsewhere changed", "same")),
        (SHELL_PWD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/pwd_script.sh", "top_child.sh", "echo top", "echo top changed", "changed")),
        (SHELL_PWD_EXACT_CELL, compare_cell(dict(SHELL_FOLDS_FIXTURE, **{"elsewhere/top_child.sh": "#!/usr/bin/env bash\necho elsewhere\n"}), "s/pwd_script.sh", "elsewhere/top_child.sh",
                                            "echo elsewhere", "echo elsewhere changed", "same")),
        (SHELL_QUOTED_BRACE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/quoted_brace.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_TAIL_GROUP_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/tail_group.sh", "fixtures/one/.sample/driver.sh", "echo driver", "echo driver changed", "changed")),
        (SHELL_TAIL_OUTSIDE_CELL, no_refusal_cell(SHELL_FOLDS_FIXTURE, "s/tail_outside.sh")),
        (SHELL_ARGUMENTS_SKIPPED_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (SHELL_ARGUMENTS_TAKEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/tools/one.py", "return 1", "return 10", "changed")),
        (SHELL_ARGUMENTS_OTHER_CASE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_caller.sh", "s/tools/two.py", "return 2", "return 20", "same")),
        (SHELL_ARGUMENTS_PREFLIGHT_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_preflight_caller.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (SHELL_ARGUMENTS_EXIT_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_exit_caller.sh", "s/tools/two.py", "return 2", "return 20", "same")),
        (SHELL_ARGUMENTS_EXIT_TAKEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/dispatch_exit_caller.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (PYTHON_SHELL_ARGUMENTS_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/launcher.py::judge", "s/child.sh", "echo child", "echo child changed", "same")),
        (SHELL_NESTED_WORD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/nested_word.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (SHELL_NESTED_INSIDE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/nested_called_inside.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_NESTED_OUTSIDE_COMMAND_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/nested_called_outside.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_PYTHON_ARGUMENTS_SKIPPED_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/python_words.sh", "s/commands/dispatch.py", "return 302", "return 3020", "same")),
        (SHELL_PYTHON_ARGUMENTS_TAKEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/python_words.sh", "s/commands/dispatch.py", "return 301", "return 3010", "changed")),
        (SHELL_PYTHON_ARGUMENTS_OPEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/python_words_open.sh", "s/commands/dispatch.py", "return 302", "return 3020", "changed")),
        (PYTHON_LAUNCH_WITHOUT_ENTRY_ARGUMENTS_CELL, compare_cell(ARGUMENTS_FIXTURE, "tool/launcher.py", "tool/dispatch.py", "    return 2\n", "    return 20\n", "same")),
        (SHELL_TABLE_UNSELECTED_CELL, compare_cell(SHELL_TABLE_FIXTURE, "s/caller.sh", "s/two.sh", "echo two", "echo two changed", "same")),
        (SHELL_TABLE_SELECTED_CELL, compare_cell(SHELL_TABLE_FIXTURE, "s/caller.sh", "s/one.sh", "echo one", "echo one changed", "changed")),
        (SHELL_TABLE_MIXED_CELL, compare_cell(SHELL_TABLE_FIXTURE, "s/caller_mixed.sh", "s/two.sh", "echo two", "echo two changed", "changed")),
        (SHELL_TABLE_ENTRY_CELL, compare_cell(SHELL_TABLE_FIXTURE, "scripts/experiment-apparatus/replay.sh", "s/two.sh", "echo two", "echo two changed", "changed")),
        (SHELL_ENTRY_ARGUMENTS_CELL, shell_entry_edits_cell("s/dispatch.sh", ["run"], [("s/child.sh", "echo child", "echo child changed", "same"),
                                                                                       ("s/tools/two.py", "return 2", "return 20", "same"),
                                                                                       ("s/tools/one.py", "return 1", "return 10", "changed")])),
        (SHELL_ENTRY_UNDECIDED_CELL, shell_entry_edits_cell("s/dispatch_undecided.sh", ["run"], [("s/child.sh", "echo child", "echo child changed", "changed")])),
        (SHELL_ENTRY_TABLE_CELL, shell_entry_table_cell),
        (SHELL_TABLE_ITEM_CELL, shell_entry_edits_cell("scripts/experiment-apparatus/replay.sh", ["--item", "E1.2"],
                                                       [("s/one_sweep.sh", "echo one-sweep", "echo one-sweep changed", "changed"),
                                                        ("s/one_grid.sh", "echo one-grid", "echo one-grid changed", "same"),
                                                        ("s/two.sh", "echo two", "echo two changed", "same")], SHELL_TABLE_ITEM_FIXTURE)),
        (SHELL_TABLE_SELFTEST_ENTRY_CELL, shell_entry_edits_cell("scripts/experiment-apparatus/replay.sh", ["--selftest"],
                                                                 [("s/one.sh", "echo one", "echo one changed", "changed"),
                                                                  ("s/two.sh", "echo two", "echo two changed", "same")], SHELL_TABLE_SELFTEST_FIXTURE)),
        (SHELL_ENTRY_EXIT_TABLE_CELL, shell_entry_exit_table_cell),
        (SHELL_TABLE_TREE_COPY_CELL, shell_table_tree_copy_cell),
        (SHELL_TABLE_REAL_CELL, shell_table_real_cell),
        (SHELL_TABLE_COPY_CELL, shell_table_copy_cell),
        (SHELL_TABLE_COPY_FRAMEWORK_CELL, compare_cell(SHELL_TABLE_COPY_FIXTURE, "scripts/gate/copy_launcher.py::judge", "s/framework.sh", "echo framework", "echo framework changed", "changed")),
        (SHELL_TABLE_COPY_CALLED_CELL, compare_cell(SHELL_TABLE_COPY_FIXTURE, "scripts/gate/copy_launcher.py::judge", "s/called.sh", "echo called", "echo called changed", "changed")),
        (SHELL_TABLE_COPY_PLAIN_WORD_CELL, compare_cell(SHELL_TABLE_COPY_FIXTURE, "scripts/gate/copy_launcher.py::judge", "s/plain_word.sh", "echo plain-word", "echo plain-word changed", "changed")),
        (SHELL_TABLE_COPY_WHOLE_CELL, compare_cell(SHELL_TABLE_COPY_FIXTURE, "scripts/gate/copy_launcher_whole.py::judge", "s/row_only.sh", "echo row-only", "echo row-only changed", "changed")),
        (SHELL_TABLE_COPY_REAL_ROW_CELL, compare_cell(SHELL_TABLE_COPY_FIXTURE, "scripts/gate/copy_launcher_real_row.py::judge", "s/shared.sh", "echo shared", "echo shared changed", "changed")),
        (SHELL_PATTERN_REMOVAL_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/pattern_removal.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_DIRECTORY_COPY_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/directory_copy.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        (SHELL_RELATIVE_COPY_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/relative_copy.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_DIRECTORY_WILDCARD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/directory_wildcard.sh", "s/tools/two.py", "    return 2", "    return 20", "changed")),
        (SHELL_POSITIONAL_SCRIPT_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/positional_wrapper.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        (SHELL_RELATIVE_CD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/relative_cd.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        ("python：os.path.join(<折不出的目录>, <仓里的相对路径>) 连到仓里那一份", compare_cell(PYTHON_FIXTURE, "pkg/d.py::run_copy", "pkg/c.py", "    return 5",
                                                                                "    return 50", "changed")),
        ("shell：git rev-parse --git-common-dir 折成根，经它 source 的库连上", compare_cell(SHELL_FIXTURE, "s/common.sh", "s/lib.sh", "echo lib", "echo lib changed", "changed")),
        ("python：git rev-parse --git-common-dir 经无参函数的返回值折成根，插进 sys.path 的目录不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/d.py::main_repository_value")),
        ("shell：重定向到变量路径不拒算", no_refusal_cell(SHELL_FIXTURE, "s/redirect.sh")),
        ("shell：把路径当参数传给自己的函数不拒算", no_refusal_cell(SHELL_FIXTURE, "s/arguments.sh")),
        ("python：写数据的路径与传给 subprocess 的参数不拒算", no_refusal_cell(PYTHON_FIXTURE, "pkg/d.py::write_data")),
        ("shell：自己用 cat <<词 写出来再起的脚本不拒算、正文里装的连上", compare_cell(SHELL_FIXTURE, "s/generator.sh", "s/child.sh", "echo child",
                                                                       "echo child changed", "changed")),
        ("shell：bash -c '…\"$1\"…' <名字> <路径> 里的 $1 换成那个路径", compare_cell(SHELL_FIXTURE, "s/inline.sh", "s/lib.sh", "echo lib", "echo lib changed", "changed")),
        ("shell：自己 cp 出来再起的脚本连到源那一份", compare_cell(SHELL_FIXTURE, "s/selfcopy.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        ("shell：套在外层函数里的函数看得见外层的 local", compare_cell(SHELL_FIXTURE, "s/nested.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        ("shell：cd 过去的目录折出几个，相对路径在每一个底下都连", compare_cell(SHELL_FIXTURE, "s/versions.sh", "s/beta/tool.sh", "echo beta tool",
                                                              "echo beta tool changed", "changed")),
        ("python：分派表（字典字面量）里的函数改了作废", compare_cell(PYTHON_FIXTURE, "pkg/d.py::dispatch", "pkg/d.py", "    return 12", "    return 13", "changed")),
        ("python：import 进来的模块常量（module.NAME）折进路径，插进 sys.path 的目录连上", compare_cell(PYTHON_FIXTURE, "pkg/e.py::load_tool", "pkg/tools/tool.py",
                                                                                     "    return 7", "    return 70", "changed")),
        ("preflight 装载器：只拷了 .claude/ 的小树里 source 与 import 都解到树里的规范副本，两份装载器的闭包不拒算", preflight_loader_cell),
        ("preflight 装载器：worktree 里没有规范副本时 source 与 import 解到主仓那一份", preflight_worktree_cell),
        (CROSS_RUN_CACHE_CELL, cross_run_cache_cell),
        (CROSS_RUN_CACHE_SCAN_CELL, cross_run_cache_scan_cell),
    ]
    cells += [(name, judge) for name, judge in silent_form_cells()]
    cells += alignment_cells()
    cells += closure_rule_cells()
    cells += closure_edge_cells()
    cells += unit_divergence_cells()
    cells += unit_graph_parity_cells()
    cells += import_time_cells()
    cells += product_view_cells()
    return cells



def silent_form_cells():
    """不许既不连边也不拒算的几种写法：每种一格连边、一格拒算（或「认出、不装仓里的代码」），各配弄坏开关（BREAK_CELLS）。"""
    helper = ("s/py/helper_mod.py", "    return 1", "    return 10", "changed")
    child = ("s/child.sh", "echo child", "echo child changed", "changed")
    return [
        ("shell：python3 -c '<字面串>' 当内联脚本切，sys.argv 折成后面的词，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/inline_python.sh", *helper)),
        ("shell：python3 - <<'PY' 喂的 here-doc 当内联脚本切，连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/heredoc_python.sh", *helper)),
        ("shell：{ python3 /dev/fd/3 …; } 3<<'PY'（重定向写在组后面）照内联脚本切", compare_cell(SHELL_FIXTURE, "s/group.sh", *helper)),
        ("shell：exec python3 -B /dev/fd/3 … 3<<'PY' 里 (目录, 标签) = sys.argv[1:3] 拆包逐个对上，sys.path.insert(0, 目录) 折得出、连到它 import 的仓内模块", compare_cell(SHELL_FIXTURE, "s/argv_slice.sh", *helper)),
        ("shell：内联 python 拆包 sys.argv[1:4] 给两个名字（上界对不上）不对上，照旧拒算 sys-path-unfolded", refused_cell(SHELL_FIXTURE, "s/argv_slice_mismatched.sh", "sys-path-unfolded")),
        ("shell：python3 -c / bash -c 的串带折不出的展开拒算 inline-script-not-literal", refused_cell(SHELL_FIXTURE, "s/inline_expanded.sh", INLINE_FORM)),
        ("shell：bash -c \"$串\" 的串折得出就切开再认，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/inline_bash_folded.sh", *child)),
        ("shell：一条命令里第二个解释器（bash 包装脚本 N python3 x.py）照第一个的规则认，连到 x.py", compare_cell(SHELL_FIXTURE, "s/second.sh", "s/py/tool.py",
                                                                                         "    return 2", "    return 20", "changed")),
        ("shell：起同目录脚本连边（写在命令替换里：x=\"$(bash 同目录/y.sh)\"）", compare_cell(SHELL_FIXTURE, "s/substitution.sh", *child)),
        ("shell：< <(命令 \"$x\") 里的命令照命令认，前面那个 < 不吞掉它的第一个词", no_refusal_cell(SHELL_FIXTURE, "s/process_substitution.sh")),
        ("shell：die 消息里出现 qemu-system-x86_64 不算外部程序", no_external_program_cell(SHELL_FIXTURE, "s/message.sh")),
        ("shell：python3 -m <仓内模块> 连到那份模块的程序入口", compare_cell(SHELL_FIXTURE, "s/module.sh", "s/py/tool.py", "    return 2", "    return 20", "changed")),
        ("shell：命令位置上折不出的程序拒算 shell-command-unfolded", refused_cell(SHELL_FIXTURE, "s/head.sh", "shell-command-unfolded")),
        ("shell：包装命令（timeout 60）后面直接执行的脚本连上", compare_cell(SHELL_FIXTURE, "s/wrapped.sh", *child)),
        ("shell：[[ … || … ]]、(( … ))、case 分支的模式、数组字面量里的词不当命令", no_refusal_cell(SHELL_FIXTURE, "s/structure.sh")),
        ("shell：名字[下标]=值、名字[下标]+=值（下标里带空格与展开）照赋值认、不当命令", no_refusal_cell(SHELL_FIXTURE, "s/subscript.sh")),
        ("shell：\"${数组[@]}\" 当命令时按数组的字面赋值摊开，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/array.sh", *child)),
        ("shell：python3 -c \"$(函数体是一条 cat <<'词' … 词 的函数)\" 折出那份 here-doc 的正文当内联脚本", compare_cell(SHELL_FIXTURE, "s/capture.sh", *helper)),
        (NESTED_POSITIONALS_CELL, compare_cell(SHELL_FIXTURE, "s/nested_inline_positionals.sh", *helper)),
        ("shell：文件头登记的 check 判据（preflight 开跑前跑）照内联 bash 认，连到它起的脚本", compare_cell(SHELL_FIXTURE, "s/declared.sh", *child)),
        ("shell：内联 python exec sys.argv[1:] 时后面那几个词照命令认", compare_cell(SHELL_FIXTURE, "s/argv_exec.sh", *child)),
        ("python：[sys.executable, \"-c\", <字面串>, …] 当内联脚本切，连到它 import 的仓内模块", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_inline", "pkg/b.py",
                                                                                         "    return 2", "    return 20", "changed")),
        ("python：-c 的串折不出（出处是环境变量）拒算 inline-script-not-literal",
         refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_dynamic_from_environment", INLINE_FORM)),
        ("python：变量解释器折不出（出处是环境变量）拒算 subprocess-command-unfolded",
         refused_cell(PYTHON_FIXTURE, "pkg/m.py::run_unknown_from_environment", COMMAND_FORM)),
        ("python：命令是 [解释器] + 参数（按调用处给的列表折）连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/m.py::launch_tool", "pkg/c.py", "    return 5",
                                                                        "    return 50", "changed")),
        ("python：认不出的命令（对象的属性）拒算 subprocess-command-unfolded", refused_cell(PYTHON_FIXTURE, "pkg/m.py::launch_job", COMMAND_FORM)),
        ("python：-m unittest <测试模块> 整份连上（测试方法改了作废）", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_tests", "pkg/test_sample.py", "        return 3",
                                                                    "        return 30", "changed")),
        ("python：shutil.which(…) 与 os.path.join(<折不出的目录>, \"bin/x\") 起的根外程序不拒算、不当工具链询问", no_refusal_no_toolchain_query_cell(PYTHON_FIXTURE, "pkg/m.py::version_of")),
        ("python：内联脚本起 sys.argv[k]（取的是起它那份脚本的 \"$1\"）不拒算、不当工具链询问：程序是调用处给的词", no_refusal_no_toolchain_query_cell(SHELL_FIXTURE, "s/positional_tool.sh")),
        ("python：上游 preflight 跑文件头登记的判据（bash -c declaration.command）不拒算，判据从声明它的脚本连",
         no_refusal_cell(PYTHON_FIXTURE, ".claude/singlefs-ai-sop/scripts/preflight.py::run_check")),
        ("python：命令里的 *选项[\"键\"] 折回调用处给的字典字面量，连到那份脚本", compare_cell(PYTHON_FIXTURE, "pkg/m.py::run_default", "pkg/c.py", "    return 5",
                                                                               "    return 50", "changed")),
        ("python：经分派表 表[键](…) 调到的函数，命令参数按那一处调用折", compare_cell(PYTHON_FIXTURE, "pkg/m.py::launch_from_table", "pkg/c.py", "    return 5",
                                                                     "    return 50", "changed")),
        (FOLD_NESTED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::nested_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_CONCATENATION_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::concatenated")),
        (FOLD_SLICE_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::sliced_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_VERSIONS_CELL, no_refusal_no_toolchain_query_cell(FOLDS_FIXTURE, "pkg/folds.py::Versions.interpreter")),
        (FOLD_LOADED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::loaded_constant", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_TAIL_GROUP_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::run_from_copy", "pkg/tools/runner.py", "    return 7", "    return 70", "changed")),
        (FOLD_OUTSIDE_SCRIPT_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::run_generated")),
        (FOLD_CWD_GROUP_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::run_in_copy", "pkg/sample/setup.sh", "echo prepared", "echo prepared again", "changed")),
        (FOLD_SUBMITTED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::submitted_run", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (FOLD_ENVIRONMENT_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::environment_launch", "pkg/c.py", "    return 5", "    return 50", "changed")),
        (DATA_READ_CHANGED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::table_rows", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_UNRELATED_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::table_rows", "pkg/other.tsv", "gamma", "delta", "same")),
        (DATA_READ_TAIL_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::rows_from_argument", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_SCRATCH_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::scratch_rows")),
        (DATA_READ_REFUSED_CELL, refused_cell(FOLDS_FIXTURE, "pkg/folds.py::listed_rows", DATA_FORM)),
        (DATA_READ_INPUT_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::judge_file")),
        (DATA_READ_PARSED_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::parsed_rows")),
        (DATA_READ_LISTING_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::listed_members")),
        (DATA_READ_COPY_CELL, no_refusal_cell(FOLDS_FIXTURE, "pkg/folds.py::copy_listed")),
        (DATA_READ_DIRECTORY_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::template_rows", "pkg/templates/second.txt", "second", "changed", "changed")),
        (DATA_READ_RETURNED_LISTING_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::templates_through_a_listing_function", "pkg/templates/first.txt",
                                                       "first", "altered", "changed")),
        (DATA_READ_MODULE_SEQUENCE_CELL, compare_cell(MODULE_SEQUENCE_FIXTURE, "pkg/tables.py::all_rows", "pkg/beta.tsv", "beta", "beta changed", "changed")),
        (FOLD_LIMIT_READ_REFUSED_CELL, refused_cell(FOLD_LIMIT_FIXTURE, "pkg/limits_over.py::table_digest", FOLD_LIMIT_FORM)),
        (FOLD_LIMIT_READ_UNDER_CELL, compare_cell(FOLD_LIMIT_FIXTURE, "pkg/limits_under.py::table_digest", "pkg/t07.tsv", "row 7", "row 7 changed", "changed")),
        (FOLD_LIMIT_LAUNCH_REFUSED_CELL, refused_cell(FOLD_LIMIT_FIXTURE, "pkg/limits_launch.py::run_all", FOLD_LIMIT_FORM)),
        (FOLD_LIMIT_SHELL_TAIL_CELL, compare_cell(FOLD_LIMIT_FIXTURE, "s/limits_tail.sh", "s/a07/tools/run.sh", "echo a07", "echo a07 changed", "changed")),
        (FOLD_LIMIT_UNMATCHED_TAILS_CELL, no_refusal_cell(FOLD_LIMIT_TAILS_FIXTURE, "pkg/dump_compare.py::compare_all")),
        (FOLD_LIMIT_MATCHED_TAILS_CELL, refused_cell(FOLD_LIMIT_MATCHED_TAILS_FIXTURE, "pkg/dump_compare.py::compare_all", FOLD_LIMIT_FORM)),
        (SHELL_QUOTED_INLINE_CELL, compare_cell(SHELL_QUOTED_INLINE_FIXTURE, "s/quoted_inline.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_QUOTED_INLINE_LAUNCH_CELL, refused_cell(SHELL_QUOTED_INLINE_FIXTURE, "s/quoted_launch.sh", "shell-path-unfolded")),
        (PREFIX_SCOPED_CELL, compare_cell(PREFIX_SCOPED_FIXTURE, "s/prefix_scoped.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (TEMPORARY_SITE_CELL, compare_cell(PREFIX_SCOPED_FIXTURE, "s/temporary_site.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (DATA_READ_UNLISTED_DIRECTORY_CELL, refused_cell(FOLDS_FIXTURE, "pkg/folds.py::template_named_in_a_file", DATA_FORM)),
        (PATTERN_LISTING_GLOB_CELL, listed_pattern_cell(PATTERN_LISTING_FIXTURE, "pkg/listings.py::stage_texts", ("pkg/stages/build.sh", "echo build", "echo rebuilt"),
                                                        ("pkg/stages/notes.txt", "notes", "notes changed"))),
        (PATTERN_LISTING_GLOB_TRANSFORMED_CELL, refused_cell(PATTERN_LISTING_FIXTURE, "pkg/listings.py::stage_tables", DATA_FORM)),
        (PATTERN_LISTING_FIND_CELL, listed_pattern_cell(PATTERN_LISTING_FIXTURE, "s/find_listing.sh", ("s/stages/check.sh", "echo check", "echo checked"),
                                                        ("s/stages/notes.txt", "notes", "notes changed"))),
        (PATTERN_LISTING_FIND_OPERATORS_CELL, refused_cell(PATTERN_LISTING_FIXTURE, "s/find_listing_any.sh", DATA_FORM)),
        (DATA_READ_LIBRARY_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::rows_through_library", "pkg/table.tsv", "alpha", "beta", "changed")),
        (DATA_READ_ATTRIBUTE_CELL, compare_cell(FOLDS_FIXTURE, "pkg/folds.py::TableReader.lines", "pkg/table.tsv", "alpha", "beta", "changed")),
        (TOOLCHAIN_QUERY_CELL, toolchain_query_cell),
        (CALLER_LAUNCH_CELL, caller_launch_cell),
        (LAUNCH_ARGUMENT_CELL, launch_argument_cell),
        (TAIL_SAMPLE_CELL, compare_cell(dict(TAIL_ANCHOR_FIXTURE, **{".claude/gate.d/fixtures/stage.sh/one/Manifest.toml": "name = sample\n"}),
                                        "pkg/manifest_reader.py::from_argument", ".claude/gate.d/fixtures/stage.sh/one/Manifest.toml", "sample", "changed", "same")),
        (TAIL_VIEW_CELL, tail_view_cell),
        (EXCLUDED_FILE_DATA_CELL, excluded_file_cell("pkg/judge.py", "# 只改这一行注释", "# 只改这一行注释（改了措辞）", "same", ("pkg/role.py",))),
        (EXCLUDED_FILE_PROGRAM_CELL, excluded_file_cell("pkg/only_via_role.py", "return 401", "return 4010", "same", ("pkg/role.py",))),
        (EXCLUDED_FILE_KEPT_CELL, excluded_file_cell("pkg/judge.py", "return 402", "return 4020", "changed", ("pkg/role.py",))),
        (EXCLUDED_FILE_UNSET_CELL, excluded_file_cell("pkg/judge.py", "# 只改这一行注释", "# 只改这一行注释（改了措辞）", "changed", ())),
        (NOT_JUDGING_FILES_CELL, not_judging_files_cell),
        (JUDGING_CLOSURE_CELL, judging_closure_cell),
        (LOADED_ATTRIBUTES_CELL, loaded_attributes_cell),
        (OPEN_TAIL_SKIPPED_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/open_tail_caller.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (OPEN_TAIL_TAKEN_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/open_tail_caller.sh", "s/tools/one.py", "return 1", "return 10", "changed")),
        (OPEN_TAIL_FIRST_WORD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/open_tail_first_word.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (OPEN_TAIL_PYTHON_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/open_tail_launcher.py::judge", "s/child.sh", "echo child", "echo child changed", "same")),
        (DEFERRED_HEREDOC_BODY_CELL, compare_cell(SHELL_FIXTURE, "s/heredoc_in_if.sh", "s/py/helper_mod.py", "    return 1", "    return 10", "changed")),
        (DEFERRED_HEREDOC_THEN_CELL, compare_cell(SHELL_FIXTURE, "s/heredoc_in_if.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (TEXT_PIECES_EXECUTABLE_CELL, compare_cell(TEXT_PIECES_FIXTURE, "pkg/launch.py::by_executable", "pkg/child.py", "return 1", "return 2", "changed")),
        (TEXT_PIECES_INTEGER_CELL, compare_cell(TEXT_PIECES_FIXTURE, "pkg/launch.py::by_integer", "pkg/child.sh", "echo child", "echo child changed", "changed")),
        (PROGRAM_WORDS_CELL, program_words_cell),
        (PROGRAM_WORDS_QUALIFIED_CELL, program_words_qualified_cell),
        (LAUNCHED_PROGRAMS_CELL, launched_programs_cell),
        (REFUSAL_LINES_CELL, refusal_lines_cell),
        (REFUSAL_WORD_LINES_CELL, refusal_word_lines_cell),
        (CASE_SUBJECT_CELL, no_refusal_cell(CASE_SUBJECT_FIXTURE, "s/case_caller.sh")),
        (ENV_ARRAY_CELL, no_refusal_cell(ENV_ARRAY_FIXTURE, "s/env_array.sh")),
        (PROCESS_SUBSTITUTION_HEREDOC_CELL, no_refusal_cell(PROCESS_SUBSTITUTION_HEREDOC_FIXTURE, "s/psub_heredoc.sh")),
        (LAUNCHED_SHELL_CELL, launched_shell_cell),
        (PROCESS_SUBSTITUTION_ARGUMENT_CELL, no_refusal_cell(PROCESS_SUBSTITUTION_ARGUMENT_FIXTURE, "s/psub.sh")),
        (REGISTERED_INLINE_LAUNCH_CELL, registered_inline_launch_cell),
        (IMPORTED_SYS_PATH_CELL, compare_cell(IMPORTED_SYS_PATH_FIXTURE, "pkg/main.py::entry", "pkg/lib/helper_mod.py", "return 3", "return 30", "changed")),
        (REGISTERED_WORDS_REAL_CELL, registered_words_real_cell),
        (REGISTERED_FUNCTIONS_REAL_CELL, registered_functions_real_cell),
        (TEST_SERVICE_CODE_TABLE_CELL, test_service_code_table_cell),
        (TEST_SERVICE_CODE_REAL_CELL, test_service_code_real_cell),
        (TEST_SERVICE_CODE_UNKEYED_CELL, test_service_code_unkeyed_cell),
        (TEST_SERVICE_CODE_ENTRY_UNKEYED_CELL, test_service_code_entry_unkeyed_cell),
        (RUNTIME_DATA_FILES_CELL, runtime_data_files_cell),
        (BUILD_OUTPUT_CELL, compare_cell(BUILD_OUTPUT_FIXTURE, BUILD_OUTPUT_ENTRY, "target/release/tool", "one", "two", "same")),
        (TOOL_SOURCE_CELL, tool_source_cell),
        (BUILT_TOOL_LAUNCH_CELL, built_tool_launch_cell),
        (CALLER_SUPPLIED_REGISTERED_CELL, caller_supplied_registered_cell),
        (CALLER_SUPPLIED_UNREGISTERED_CELL, caller_supplied_unregistered_cell),
        (CALL_SITE_SHADOWED_CELL, compare_cell(CALL_SITE_FIXTURE, "tool/shadowed_run.py::main", "tool/child.sh", "echo child", "echo child changed", "changed")),
        (CALL_SITE_ATTRIBUTE_CELL, compare_cell(CALL_SITE_FIXTURE, "tool/attribute_caller.py::main", "tool/child.sh", "echo child", "echo child changed", "changed")),
        (UNIT_GRAPH_TEXT_BUSY_CELL, unit_graph_text_busy_cell),
        (UNIT_GRAPH_FAILED_STEP_CELL, unit_graph_failed_step_cell),
        (UNIT_GRAPH_PRESENT_PROGRAM_CELL, unit_graph_present_program_cell),
        (UNIT_GRAPH_SHARED_DIRECTORIES_CELL, unit_graph_shared_directories_cell),
        (CALL_SITE_PARTIAL_CELL, compare_cell(CALL_SITE_FIXTURE, "tool/partial_caller.py::main", "tool/child.sh", "echo child", "echo child changed", "changed")),
        (CALL_SITE_UNDECIDED_CELL, refused_cell(CALL_SITE_FIXTURE, "tool/holder_caller.py::main", COMMAND_FORM)),
        (NESTED_PROGRAM_WORDS_REGISTERED_CELL, nested_program_words_registered_cell),
        (NESTED_PROGRAM_WORDS_UNREGISTERED_CELL, nested_program_words_unregistered_cell),
        (REGISTERED_MISSING_PATH_CELL, registered_missing_path_cell),
        (INLINE_REFUSAL_LINE_CELL, inline_refusal_line_cell),
        (INLINE_SOURCED_BODY_CELL, inline_sourced_body_cell),
        (REGISTRY_COMMAND_TEXTS_CELL, compare_cell(REGISTRY_COMMAND_FIXTURE, ".claude/gate.d/doc-registries.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (REGISTRY_COMMAND_TABLE_CELL, compare_cell(REGISTRY_COMMAND_FIXTURE, ".claude/gate.d/doc-registries.sh", ".claude/gate.d/counted-claims.tsv", "(1)", "(2)", "changed")),
        (SPEC_PATH_ARGUMENTS_CELL, spec_path_arguments_cell),
        (TAIL_READ_SCOPES_CELL, tail_read_scopes_cell),
        (PATH_TAILS_JOINED_CELL, compare_cell(PATH_TAILS_FIXTURE, "pkg/stub_reader.py::joined", "data/remove_stub.tsv", "stub", "stub changed", "same")),
        (PATH_TAILS_JOINED_KEPT_CELL, compare_cell(PATH_TAILS_FIXTURE, "pkg/stub_reader.py::joined", "data/b.tsv", "b", "b changed", "changed")),
        (PATH_TAILS_CONCATENATED_CELL, compare_cell(PATH_TAILS_FIXTURE, "pkg/stub_reader.py::concatenated", "data/remove_stub.tsv", "stub", "stub changed",
                                                    "changed")),
        (TAIL_READ_ROOT_SCOPE_CELL, tail_read_root_scope_cell),
        (TAIL_READ_SCOPES_REAL_CELL, tail_read_scopes_real_cell),
        (TAIL_READS_RECORDS_REAL_CELL, tail_reads_records_real_cell),
        (GETATTR_NAMES_CELL, no_refusal_cell(GETATTR_NAMES_FIXTURE, "pkg/options.py::missing")),
        (FUNCTION_PRINTED_WORD_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/printed_word_wrapper.sh", "s/tools/one.py", "    return 1", "    return 10", "changed")),
        (QUOTED_ASSIGNMENT_ENV_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/quoted_assignment_env.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (QUOTED_ASSIGNMENT_ARRAY_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/quoted_assignment_array.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (LOCAL_NAME_SHADOW_CELL, compare_cell(LOCAL_NAME_FIXTURE, "pkg/shadow.py::shadowing", "pkg/shadow.py", "return 501", "return 5010", "same")),
        (STAGE_CELL_FUNCTIONS_REAL_CELL, stage_cell_functions_real_cell),
        (MODULE_BODY_COMMAND_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/module_body_command.py", "s/child.sh", "echo child", "echo child changed", "changed")),
        (EXPORTED_VARIABLE_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/exported_path.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (PYTHON_SPLIT_PART_ENVIRONMENT_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/exported_part.sh", "s/child.sh", "echo child", "echo child changed", "changed")),
        (SHELL_SELF_ASSIGNMENT_CELL, compare_cell(SHELL_SELF_ASSIGNMENT_FIXTURE, "s/self_assigned.sh", "s/child.sh", "echo child", "echo child changed",
                                                  "changed")),
        (GUARD_NAME_PATH_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/guard_path/run.py", "s/guard_helpers/guard_helper.py", "return 601", "return 602", "changed")),
        (HEREDOC_BLANK_ROW_CELL, compare_cell(SHELL_FOLDS_FIXTURE, "s/heredoc_blank_row_caller.sh", "s/child.sh", "echo child", "echo child changed", "same")),
        (NARROWED_VIEW_SHARED_CELL, narrowed_view_shared_cell),
        (OWN_PROGRAM_KEYED_CELL, own_program_keyed_cell),
        (OWN_PROGRAM_ENTRY_CELL, own_program_entry_cell),
        (OWN_PROGRAM_PATH_CELL, own_program_path_cell),
        (OWN_PROGRAM_BLOCK_CELL, own_program_refused_cell(OWN_PROGRAM_RESTARTS_REFUSED_BY_BLOCK)),
        (OWN_PROGRAM_DIVERGENCE_CELL, own_program_refused_cell(OWN_PROGRAM_RESTARTS_REFUSED_BY_DIVERGENCE)),
        (EXTERN_LISTED_CELL, extern_listed_cell),
        (EXTERN_REFUSED_CELL, extern_refused_cell),
        (EXTERN_LINK_ATTRIBUTE_CELL, extern_refused_cell_for("link-in-cfg-attr", EXTERN_LINK_ATTRIBUTE_SOURCE)),
        (EXTERN_OTHER_BLOCK_LINK_CELL, extern_refused_cell_for("link-on-another-block", EXTERN_OTHER_BLOCK_LINK_SOURCE)),
        (EXTERN_BUILD_SCRIPT_LINK_CELL, extern_build_script_link_cell),
        (WORKSPACE_DOT_MEMBER_CELL, workspace_dot_member_cell),
        (WORKSPACE_BRACKET_CELL, workspace_root_with_brackets_cell),
        (WORKSPACE_MULTILINE_CELL, workspace_multiline_members_cell),
        (WORKSPACE_SAME_NAME_CELL, workspace_same_named_directories_cell),
        (EXTERN_OTHER_TARGET_LINK_CELL, extern_link_beyond_cell(EXTERN_OTHER_TARGET_LINK_REFUSED, EXTERN_OTHER_TARGET_LINK_KEYED)),
        (EXTERN_DEPENDENCY_LINK_CELL, extern_link_beyond_cell(EXTERN_DEPENDENCY_LINK_REFUSED, EXTERN_DEPENDENCY_LINK_KEYED)),
        (EXTERN_LINK_ARGUMENT_CELL, extern_link_beyond_cell(EXTERN_LINK_ARGUMENT_REFUSED, EXTERN_LINK_ARGUMENT_KEYED)),
        (EXTERN_EXTERNAL_DEPENDENCY_CELL, extern_link_beyond_cell(EXTERN_EXTERNAL_DEPENDENCY_REFUSED, EXTERN_EXTERNAL_DEPENDENCY_KEYED)),
        (EXTERN_INSTRUCTION_LITERAL_CELL, extern_link_beyond_cell({}, EXTERN_INSTRUCTION_LITERAL_KEYED)),
        (EXTERN_SYSTEM_LIBRARY_LIST_CELL, rust_string_list_cell("SYSTEM_LIBRARY_CALLS_ALLOWED_IN_EXTERN_BLOCKS", c_library_calls_allowed_in_extern_blocks)),
        (LOCAL_NAME_REAL_CALL_CELL, compare_cell(LOCAL_NAME_FIXTURE, "pkg/shadow.py::real_call", "pkg/shadow.py", "return 501", "return 5010", "changed")),
        (LOCAL_NAME_GLOBAL_CELL, compare_cell(LOCAL_NAME_FIXTURE, "pkg/shadow.py::by_global", "pkg/shadow.py", "return 502", "return 5020", "changed")),
        (CLOSURE_ROUND_LIMIT_CELL, closure_round_limit_cell),
        (CLOSURE_CALL_INDEX_CELL, compare_cell(CLOSURE_ROUNDS_FIXTURE, "pkg/launch.py::check", "pkg/table.tsv", "alpha", "beta", "changed")),
        (SHARED_SCRIPT_INDEX_CELL, shared_script_index_cell),
        (TAIL_ANCHOR_SCRATCH_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::scratch_run", "fixtures/sample/Manifest.toml", "sample", "changed", "same")),
        (TAIL_ANCHOR_JUDGE_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::judge", "fixtures/sample/Manifest.toml", "sample", "changed", "same")),
        (TAIL_ANCHOR_ARGUMENT_CELL, compare_cell(TAIL_ANCHOR_FIXTURE, "pkg/manifest_reader.py::from_argument", "fixtures/sample/Manifest.toml", "sample",
                                                 "changed", "changed")),
        (LAUNCH_ARGUMENT_SCRIPT_CELL, compare_cell(LAUNCH_ARGUMENT_FIXTURE, "pkg/launch.py::check_script", "pkg/tool.sh", "echo tool", "echo tool again", "changed")),
        (LAUNCH_ARGUMENT_REFUSED_CELL, refused_cell(LAUNCH_ARGUMENT_FIXTURE, "pkg/launch.py::check_unfolded", COMMAND_FORM)),
        (REGISTRY_PROGRAMS_CELL, registry_programs_cell),
        (ROW_KEYED_CELL, row_keyed_cell),
        (PROGRAM_ARGUMENTS_CELL, program_arguments_cell),
        (FORKED_GATHER_CELL, forked_gather_cell),
    ]



def run_cell(judge):
    with tempfile.TemporaryDirectory(prefix="function-units-selftest-") as work:
        try:
            return judge(work)
        except (FunctionUnitsError, UsageError, AssertionError, OSError, KeyError, ValueError) as error:
            return False, f"抛了 {type(error).__name__}：{error}"


def load_selftest_fixture_file(file_name, present):
    """同目录的一份夹具（file_name）按路径载入：先把 present 里的名字放进它、再执行它 → 它新定义的名字。"""
    specification = importlib.util.spec_from_file_location(file_name[:-len(".py")], os.path.join(LIBRARY_ROOT, "scripts", "gate", file_name))
    fixtures = importlib.util.module_from_spec(specification)
    vars(fixtures).update(present)
    specification.loader.exec_module(fixtures)
    return {name: value for name, value in vars(fixtures).items() if not name.startswith("__") and name not in present}


SELFTEST_MODULE_NAMES = ("lib_function_units_selftest_fixture_trees", "lib_function_units_selftest_rust_cells", "lib_function_units_selftest_program_registries",
                         "lib_function_units_selftest_closure_rules", "lib_function_units_selftest_real_repo")


def load_selftest_fixtures():
    """文件头「自证夹具」：自证开跑前按次序按路径载入同目录的三份夹具（lib_function_units_selftest_fixtures.py、
    lib_function_units_selftest_fixtures_closure_rules.py、lib_function_units_selftest_fixtures_real_repo.py：只给自证读的夹具文本、格名、开关到格的对照表），
    每份先把自证各模块（这份与 SELFTEST_MODULE_NAMES 里的几份）此刻的名字、已载入的几份新定义的名字放进它、再执行它，
    新定义的名字放回自证各模块 → 放回了几个。fork 出来的各格进程照父进程这一份读。"""
    namespaces = [load_selftest_fixtures.__globals__] + [vars(sys.modules[name]) for name in SELFTEST_MODULE_NAMES]
    present = {}
    for namespace in namespaces:
        present.update({name: value for name, value in namespace.items() if not name.startswith("__")})
    added = {}
    added.update(load_selftest_fixture_file("lib_function_units_selftest_fixtures.py", {**present, **added}))
    added.update(load_selftest_fixture_file("lib_function_units_selftest_fixtures_closure_rules.py", {**present, **added}))
    added.update(load_selftest_fixture_file("lib_function_units_selftest_fixtures_real_repo.py", {**present, **added}))
    for namespace in namespaces:
        namespace.update(added)
    return len(added)


def selftest():
    started = time.monotonic()
    failures, checked = [], 0
    load_selftest_fixtures()
    os.environ[RUST_LEX_CACHE_DIRECTORY_VARIABLE] = "off"   # 词法单元缓存只在它自己那一格用（那一格临时换一个目录）；别的格都现切，弄坏开关也只波及那一格
    if active_breaks():
        print(f"  ℹ {BREAK_VARIABLE}={os.environ.get(BREAK_VARIABLE)} 设着：点名的那几格本来就该红")
    heavy = heavy_threads_module()
    try:
        threads = int(heavy.default_heavy_threads(os.environ))
    except heavy.HeavyThreadDefaultError as error:
        print(f"  ✗ lib_function_units.py 自证开不了并行：{error}")
        print(f"    → 怎么办：{error.howto}")
        return 2
    cells = selftest_cells()
    by_name = dict(cells)
    # 各格与各开关下的重判都是一条活，各在自己 fork 出来的进程里跑（gather_forked）；结论照登记次序打：先各格，再各开关
    tasks, labels, switched = [], [], {}
    for name, judge in cells:
        tasks.append((None, judge))
        labels.append(name)
    for switch, red_cells in BREAK_CELLS:
        for cell in red_cells:
            if cell in by_name and (switch, cell) not in switched:
                switched[(switch, cell)] = len(tasks)
                tasks.append((switch, by_name[cell]))
                labels.append(f"弄坏开关 {switch} 下「{cell}」")
    jobs = max(1, min(threads, len(tasks)))
    print(f"  ℹ 并行 {jobs} 个进程（每台默认重活线程数 {threads}）：{len(cells)} 格、{len(switched)} 次开关下重判，各在自己 fork 出来的进程里跑")
    outcomes, lost = gather_forked(tasks, jobs, run_selftest_task)
    lost_by_index = {item.index: item for item in lost}
    for index, name in enumerate(labels[:len(cells)]):
        result = forked_cell_result(outcomes[index], lost_by_index.get(index))
        if result is None:
            continue
        checked += 1
        ok, detail, _finished = result
        print(f"  {'✓' if ok else '✗'} {name}" + ("" if ok else f"：{detail}"))  # gate-lint:detail
        if not ok:
            failures.append(name)
    for switch, red_cells in BREAK_CELLS:
        checked += 1
        missing = [cell for cell in red_cells if cell not in by_name]
        flipped, unfinished = [], []
        for cell in red_cells:
            index = switched.get((switch, cell))
            result = None if index is None else forked_cell_result(outcomes[index], lost_by_index.get(index))
            if result is None:
                continue
            ok, detail, finished = result
            if not finished:
                unfinished.append(f"{cell}：{detail}")
            elif not ok:
                flipped.append(cell)
        ok = not missing and not unfinished and flipped == list(red_cells)
        label = f"弄坏开关 {switch} 下「{'」「'.join(red_cells)}」转红"
        detail = f"：只红了 {flipped}，表里没有 {missing}" + (f"，没跑完 {unfinished}" if unfinished else "")
        print(f"  {'✓' if ok else '✗'} {label}" + ("" if ok else detail))  # gate-lint:detail
        if not ok:
            failures.append(label)
    if lost:
        print(f"  ✗ lib_function_units.py 自证并行：派出去 {len(tasks)} 条、收回 {len(tasks) - len(lost)} 条，对不上："
              + "；".join(f"{labels[item.index]}（{item.reason}）" for item in lost))
        print("    → 怎么办：看上面那几条没收回的输出（子进程被信号打断多是撞了内存上限、退出码非 0 多是格里调了 os._exit 或 fork 之后崩了），"
              "照它改那一格或交任务服务放大上限重排；收回的条数对不上整份不算过")
    elapsed = time.monotonic() - started
    if failures or lost:
        print(f"  ✗ lib_function_units.py 自证没过：{len(failures)} 格判错" + (f"、{len(lost)} 条没收回" if lost else "")
              + f"（共 {checked} 格，并行 {jobs} 个进程，{elapsed:.1f} 秒）")
        print("    → 怎么办：照上面每一格改对应语言那一段的切分或闭包规则；FUNCTION_UNITS_BREAK 设着的话点名的那几格本来就该红")
        return 1
    print(f"  ✓ lib_function_units.py 自证通过：{checked} 格都对（Rust 夹具 {len(RUST_FIXTURE_CASES)} 格加拒算与形态、python、shell、键；"
          f"弄坏开关 {len(BREAK_CELLS)} 个各红在点名的格；并行 {jobs} 个进程；{elapsed:.1f} 秒）")
    return 0
