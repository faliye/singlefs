"""准入自证（入口 admission.py 的 --selftest）的一块：实验键、复跑表行键、replay.sh 接准入、前提判定、plan-failure、library-files 等格，以及崩溃枚举用例的登记、日志、线程、特性、块账与起用例命令等格。只在自证里 import。
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

from admission_base import (
     REGISTRATION_TABLE, REPLAY_SCRIPT, RESULTS_DIRECTORY, BREAK_VARIABLE, VERDICT_LIBRARY_DIRECTORY_NAME,
     EXIT_REGISTRATION_ERROR, EXIT_PRECONDITION_MISSING, EXIT_INPUTS_UNCHANGED, EXIT_GATE_PRECONDITION_MISSING, REPLAY_FROZEN_ROOTS_DIRECTORY,
     REPLAY_ROW_KEYS_START, REPLAY_ROW_KEYS_END_LINE, MULTI_HOST_CONFIGURATION_VARIABLE, MULTI_HOST_CONFIGURATION_NAME, GPU_QUIESCE_SCRIPT,
     SERVICE_CARD_VARIABLE, BINDGEN_LIBRARY_SCRIPT, BINDGEN_VARIABLE, CRASH_CASE_BLOCKS_LINE_NOT_PRINTED, CRASH_CASE_NARROWING_VARIABLES,
     CRASH_CASE_STAGES, CRASH_STAGE_VARIABLE, CRASH_HOST_WEIGHTS_VARIABLE, CRASH_LIBRARY_RUN_KIND_VARIABLE, EXIT_BLOCKS_RED, EXIT_BLOCKS_LINE_BAD,
     CRASH_POINT_REPLAY_THREADS_VARIABLE, HEAVY_THREAD_DEFAULT_FILE, THREAD_CAP_VARIABLE, REPLAY_RESEARCH_DIRECTORY, REPLAY_JUDGING_FUNCTIONS,
     VERIFICATION_PACKAGE, RegistrationError, InputManifestError, RegistrationRow, read_registration_rows, FUNCTION_UNITS_MODULE, function_units,
     UnitKeyError, break_is_set, SAME_RUN_PRODUCT_VARIABLE, CrashCase, parse_crash_case, crash_cases_of, crash_case_of_key, machine_core_count,
     registered_heavy_threads, REPLAY_STRIP_HEADER_COMMAND,
)
from admission_unit_key import KeyContext, manifest_sections, workspace_package_directories, transit_files
from admission_crash_case import (
     lint_registration_rows, judge_crash_case_log, configured_worker_threads_text, crash_case_worker_threads, crash_case_launch,
)
from admission_commands import (
     replay_packages_mentioned, replay_binaries_mentioned, replay_device_key_binaries, stored_product_row_key_status, PLAN_FAILURE_SAMPLE_MARKER,
     PLAN_SERVICE_UNAVAILABLE_EXIT,
)
from admission_selftest_support import (
     SELFTEST_HERE, PASSED_ONE_LINE, link_sop_copy, link_replay_build_log_tools, unit_sample_requests, edit_once, build_unit_sample_repository,
     EnvironmentPatch, RESEARCH_SAMPLE_FILES, build_research_sample_repository, run_quietly, write_text, write_questions, GIT_IDENTITY,
     write_executable, run_in_environment, BreakSwitch, FAKE_TOOLCHAIN_SCRIPTS, build_crash_case_repository, ENTRY_SCRIPT, copy_admission_modules,
     copy_function_units_library,
)

def run_experiment_cells(selftest, module):
    """实验的准入（行键去掉期望产物）：老产物没有实验键放行；存一份带它的产物之后判「输入没变」退 77；改装置 main 放行、改回去又 77；
    装置只加注释、kb 决策文档改一句（T19）、登记表别的行与注释（N22）都照 77；设了删掉的强制重跑变量退 2；前提；登记错退 2；给 doc-experiments 的四种状态。
    弄坏开关 experiment-key-with-kb-documents 下 T19 那一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-experiment-")
    try:
        environment = build_research_sample_repository(work)
        copied = os.path.join(work, "scripts/gate/admission.py")

        def experiment(key, changes=None):
            return run_in_environment([sys.executable, copied, "experiment", work, key], dict(environment, **(changes or {})))

        exit_code, output, messages = experiment("E900")
        first_header = output.strip().split("\n")[0] if output.strip() else ""
        selftest.expect("实验准入：老产物头上没有实验键时放行并报判不了", exit_code == 0 and "判不了" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        selftest.expect("实验准入：放行时产物头第一行是带键的实验键行（sha256= 64 位十六进制、units= 单元数）",
                        re.fullmatch(r"E7INPUT name=input_fingerprint key=E900 sha256=[0-9a-f]{64} units=[1-9][0-9]*", first_header) is not None,
                        f"头一行是「{first_header}」")
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"), first_header + "\nE7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
        edit_once(os.path.join(work, REPLAY_SCRIPT), "E900|@driver_e900||e900-2026-09-01.out|exact", "E900|@driver_e900||e900-2026-09-02.out|exact")
        exit_code, output, messages = experiment("E900")
        selftest.expect("实验准入：存了带这把实验键的产物、复跑表指到它 ⇒ 输入没变，退 77、stdout 为空、出路写沿用那一份的结论（改复跑表的产物文件名不动实验键）",
                        exit_code == EXIT_INPUTS_UNCHANGED and output == "" and "沿用那一份的结论" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900", {BREAK_VARIABLE: "skip-unchanged"})
        selftest.expect("弄坏开关 skip-unchanged 下「输入没变」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        device = os.path.join(work, "scripts/experiment-apparatus/e900.rs")
        edit_once(device, "let _ = 1;", "let _ = 2;")
        exit_code, _output, messages = experiment("E900")
        selftest.expect("实验准入：装置 main 改一个字 ⇒ 放行并报变了", exit_code == 0 and "变了" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        edit_once(device, "let _ = 2;", "let _ = 1;")
        edit_once(device, "    let _ = 1;\n", "    // 只是注释\n    let _ = 1;\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：装置 main 只加一行注释 ⇒ 照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        decision = os.path.join(work, ".claude/kb/decisions/08-sample.md")
        edit_once(decision, "一句。", "改了一句。")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：T19（kb 决策文档改一句）⇒ 实验键不变，照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        exit_code, _output, _messages = experiment("E900", {BREAK_VARIABLE: "experiment-key-with-kb-documents"})
        selftest.expect("弄坏开关 experiment-key-with-kb-documents 下 T19 那一格转红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        table_path = os.path.join(work, REGISTRATION_TABLE)
        with open(table_path, encoding="utf-8") as handle:
            good_table = handle.read()
        write_text(table_path, good_table + "# 加一行注释\nE901\tscripts/experiment-apparatus/e23_journal_geom.rs::main scripts/experiment-apparatus/e900.rs::main\t# 别的一行\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("实验准入：N22（登记表加注释、别的实验行多登记入口）⇒ 照判没变（退 77）", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")
        write_text(table_path, good_table)
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("实验准入：前提没齐时拒绝（退 3）并点名那一行", exit_code == EXIT_PRECONDITION_MISSING and "第 6 行的状态是「开着」" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900/second", {BREAK_VARIABLE: "skip-preconditions"})
        selftest.expect("弄坏开关 skip-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        write_questions(work, "**够判：装置的期望值过时、对照本身是好的**（样本）")
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("实验准入：前提齐了放行（E900/second 不在复跑表里，按登记的入口算键）", exit_code == 0, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        # same-run-field（文件头「准入条件」）：前一段这一趟的输出经 SAME_RUN_PRODUCT_VARIABLE 交来；复跑表登记给 E900 的留存产物里写着 control_ok=true，不算数
        same_run_product = os.path.join(work, ".same-run", "first-segment.out")
        environment.pop(SAME_RUN_PRODUCT_VARIABLE, None)   # 调用这一份自证的环境里设着它也不算
        exit_code, _output, messages = experiment("E900/third")
        selftest.expect("same-run-field：没交同一趟的输出时前提没齐（退 3）并点名那个环境变量", exit_code == EXIT_PRECONDITION_MISSING and SAME_RUN_PRODUCT_VARIABLE in messages,
                        f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        write_text(same_run_product, "E7RESULT name=verdict control_ok=false\nE7RESULT name=done emitted=2\n")
        exit_code, _output, messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product})
        selftest.expect("same-run-field：同一趟的输出里值不等时前提没齐（退 3），留存产物里的 control_ok=true 不算数",
                        exit_code == EXIT_PRECONDITION_MISSING and "control_ok=false" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product, BREAK_VARIABLE: "same-run-field-unchecked"})
        selftest.expect("弄坏开关 same-run-field-unchecked 下「same-run-field：同一趟的输出里值不等」那一格红（放行了）", exit_code == 0, f"弄坏之后仍退 {exit_code}")
        write_text(same_run_product, "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
        exit_code, _output, messages = experiment("E900/third", {SAME_RUN_PRODUCT_VARIABLE: same_run_product})
        selftest.expect("same-run-field：同一趟的输出里值相等时放行，说明点名那份输出",
                        exit_code == 0 and f"同一趟的输出 {same_run_product} 的 name=verdict control_ok=true" in messages, f"退 {exit_code}，stderr：{messages.strip()[-400:]}")
        exit_code, _output, _messages = experiment("E901")
        selftest.expect("实验准入：没登记的实验键退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        write_text(table_path, good_table + "E902\tscripts/experiment-apparatus/e902_nowhere.rs::main\t# 入口文件不在\nE903\tscripts/experiment-apparatus/e900.rs::main\tbogus=1\t# 认不出的条件\n")
        exit_code, _output, _messages = experiment("E902")
        selftest.expect("实验准入：登记的入口文件不在时退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        exit_code, _output, _messages = experiment("E903")
        selftest.expect("实验准入：认不出的准入条件退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("登记表自查点得出入口文件不在、认不出的条件", any("e902_nowhere.rs" in problem for problem in problems) and any("bogus=1" in problem for problem in problems),
                        f"自查结果：{problems}")
        # 前提点名的文件不在仓里（文件头「准入条件」）：question-row 的问题单、product-field 指的留存产物（E901 那一行的产物挪走）
        write_text(table_path, good_table + "E904\t@E900\tquestion-row=research/no-such-questions.md#6:够判\t# 问题单不在\n"
                   "E905\t@E900\tproduct-field=E901:e901:rows=1\t# 指的留存产物挪走了\n")
        stored_e901 = os.path.join(work, RESULTS_DIRECTORY, "e901-2026-09-01.out")
        os.rename(stored_e901, stored_e901 + ".moved")
        problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("登记表自查点得出前提点名的文件不在：question-row 的问题单、product-field 指的留存产物",
                        any("research/no-such-questions.md 不在仓里" in problem for problem in problems)
                        and any("e901-2026-09-01.out 不在仓里" in problem for problem in problems), f"自查结果：{problems}")
        with EnvironmentPatch({BREAK_VARIABLE: "condition-files-unchecked"}):
            problems = lint_registration_rows(read_registration_rows(work), work)
        selftest.expect("弄坏开关 condition-files-unchecked 下「登记表自查点得出前提点名的文件不在」那一格红（一处都不报）",
                        not any("不在仓里" in problem for problem in problems), f"弄坏之后自查结果：{problems}")
        os.rename(stored_e901 + ".moved", stored_e901)
        write_text(table_path, good_table)
        with EnvironmentPatch({"PATH": environment["PATH"], "CARGO_HOME": environment["CARGO_HOME"]}):
            status, _detail = stored_product_row_key_status(work, "E900")
            selftest.expect("给 doc-experiments：产物头的实验键对得上时状态是 matched", status == "matched", f"实际 {status}")
            edit_once(device, "let _ = 1;", "let _ = 3;")
            status, _detail = stored_product_row_key_status(work, "E900")
            selftest.expect("给 doc-experiments：装置变了时状态是 stale", status == "stale", f"实际 {status}")
            status, _detail = stored_product_row_key_status(work, "E900/second")
            selftest.expect("给 doc-experiments：没有一份带这个键的产物时状态是 unfingerprinted", status == "unfingerprinted", f"实际 {status}")
            status, _detail = stored_product_row_key_status(work, "E904")
            selftest.expect("给 doc-experiments：没登记的键状态是 unregistered", status == "unregistered", f"实际 {status}")
    except AssertionError as error:
        selftest.expect("实验准入的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)

def run_row_key_cells(selftest, module):
    """复跑表的行键（controls L25）：R01 装置函数体变只那一行变；R02 只加注释不变；R03 只改产物文件名不变（弄坏开关 row-key-with-product-name 下转红）；
    R04 共用库里两行都不用的函数变两行都不变；R05 aes-gcm 抬版本哪一行都不变（依赖版本不进键）；T18 共用库里 E23 用的 Emitter::emit 变只 E23 变；
    E22 切 GPU 开关只点到 CHECKER_TIER_FEATURES 的那一行变（弄坏开关 row-key-without-gpu 下转红）；期望产物内容变那一行变；判法变每一行都变。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-rows-")
    try:
        environment = build_research_sample_repository(work)
        copied = os.path.join(work, "scripts/gate/admission.py")

        def row_keys(arguments=(), changes=None):
            exit_code, output, messages = run_in_environment([sys.executable, copied, "replay-row-inputs", work, *arguments], dict(environment, **(changes or {})))
            keys = {line.split("\t")[0].split("|")[0]: line.split("\t")[1] for line in output.split("\n") if line.count("\t") >= 3}
            return exit_code, keys, output + messages

        exit_code, baseline, output = row_keys(["--gpu", "0"])
        selftest.expect("replay-row-inputs：四行都算得出行键（不拒算）", exit_code == 0 and len(baseline) == 4 and all(re.fullmatch(r"[0-9a-f]{64}", key) for key in baseline.values()),
                        f"退 {exit_code}，{output.strip()[-600:]}")

        def cell(label, path, old, new, changed_rows, arguments=("--gpu", "0"), changes=None):
            if path is not None:
                edit_once(os.path.join(work, path), old, new)
            try:
                _exit_code, after, output = row_keys(arguments, changes)
            finally:
                if path is not None:
                    edit_once(os.path.join(work, path), new, old)
            changed = sorted(row for row in baseline if after.get(row) != baseline[row])
            selftest.expect(f"行键：{label} ⇒ 变的是 {changed_rows or '无'}", changed == sorted(changed_rows), f"实际变的是 {changed or '无'}：{output.strip()[-400:]}")

        cell("R01 E23 装置函数体改一个字", "scripts/experiment-apparatus/e23_journal_geom.rs", "let start = 0;", "let start = 0u64;", ["E23"])
        cell("R02 E23 装置函数里只加一行注释", "scripts/experiment-apparatus/e23_journal_geom.rs", "    let start = 0;\n", "    // 只是注释\n    let start = 0;\n", [])
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e23-renamed-2026-09-01.out"), RESEARCH_SAMPLE_FILES[RESULTS_DIRECTORY + "/e23-2026-09-01.out"])
        cell("R03 E23 产物改名（内容相同）", REPLAY_SCRIPT, "|e23-2026-09-01.out|", "|e23-renamed-2026-09-01.out|", [])
        broken = {BREAK_VARIABLE: "row-key-with-product-name"}
        _exit_code, broken_before, _output = row_keys(["--gpu", "0"], broken)
        edit_once(os.path.join(work, REPLAY_SCRIPT), "|e23-2026-09-01.out|", "|e23-renamed-2026-09-01.out|")
        _exit_code, broken_after, _output = row_keys(["--gpu", "0"], broken)
        edit_once(os.path.join(work, REPLAY_SCRIPT), "|e23-renamed-2026-09-01.out|", "|e23-2026-09-01.out|")
        selftest.expect("弄坏开关 row-key-with-product-name 下 R03 那一格转红（只改产物文件名，E23 的行键变了）",
                        broken_before.get("E23") and broken_before.get("E23") != broken_after.get("E23"), f"改名前 {broken_before.get('E23')}，改名后 {broken_after.get('E23')}")
        cell("R04 共用库里两行都不用的 lru_new 改断言消息", "scripts/experiment-apparatus/lib.rs", "\"容量\"", "\"容量上限\"", [])
        cell("R05 Cargo.lock 里 aes-gcm 抬版本（依赖版本不进键）", "scripts/experiment-apparatus/Cargo.lock", 'version = "0.11.1"', 'version = "0.11.2"', [])
        cell("T18 共用库里 E23 用到的 Emitter::emit 改一个字", "scripts/experiment-apparatus/lib.rs", "        1\n", "        2\n", ["E23"])
        cell("E22 切 GPU 开关（--gpu 1）", None, None, None, ["E901"], arguments=("--gpu", "1"))
        broken = {BREAK_VARIABLE: "row-key-without-gpu"}
        _exit_code, gpu_off, _output = row_keys(["--gpu", "0"], broken)
        _exit_code, gpu_on, _output = row_keys(["--gpu", "1"], broken)
        selftest.expect("弄坏开关 row-key-without-gpu 下 E22 那一格转红（切 GPU 开关，点到 CHECKER_TIER_FEATURES 的 E901 行键不变）",
                        gpu_off.get("E901") and gpu_off.get("E901") == gpu_on.get("E901"), f"关 {gpu_off.get('E901')}，开 {gpu_on.get('E901')}")
        cell("期望产物的内容加一行", RESULTS_DIRECTORY + "/e140-2026-09-01.out", "E7RESULT name=done", "E7RESULT name=extra\nE7RESULT name=done", ["E140"])
        cell("驱动 driver_e900 改一个字", REPLAY_SCRIPT, "control_ok=true\\n", "control_ok=true \\n", ["E900"])
        cell("判法 replay_judge_fresh_product 改一个字", REPLAY_SCRIPT, 'diff -q "$1" "$2"', 'diff -q -- "$1" "$2"', ["E140", "E23", "E900", "E901"])
        # 驱动在研究工作区里点名 -p <研究包> --bin <名>（E174 那四个驱动的写法）：行键从 --bin 的名字解到 scripts/experiment-apparatus 里那份源码，不拒算
        package_line = "  cargo build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml -p e7-index-bench --bin e900 >&2 || return 1\n"
        edit_once(os.path.join(work, REPLAY_SCRIPT), "driver_e900() {\n", "driver_e900() {\n" + package_line)
        try:
            exit_code, package_keys, output = row_keys(["--gpu", "0"])
            selftest.expect("行键：驱动点名 -p e7-index-bench --bin e900（研究工作区的包）⇒ E900 算得出行键、不拒算",
                            exit_code == 0 and re.fullmatch(r"[0-9a-f]{64}", package_keys.get("E900", "")) is not None, f"退 {exit_code}，{output.strip()[-400:]}")
            _exit_code, broken_keys, output = row_keys(["--gpu", "0"], {BREAK_VARIABLE: "research-package-unresolved"})
            selftest.expect("弄坏开关 research-package-unresolved 下那一格转红（E900 拒算 replay-package-without-binary）",
                            broken_keys.get("E900") == "refused" and "replay-package-without-binary" in output, f"E900 {broken_keys.get('E900')}：{output.strip()[-400:]}")
        finally:
            edit_once(os.path.join(work, REPLAY_SCRIPT), "driver_e900() {\n" + package_line, "driver_e900() {\n")
        # 驱动点名冻结根（"$(replay_frozen_root <名>)"）：冻结根的内容摘要进这一行的行键。工作根给样本里的一个目录（SINGLEFS_WORK_ROOT），工作根的定义取样本仓里的 lib_work_root.py
        hooks_directory = os.path.join(work, ".claude", "hooks")
        os.makedirs(hooks_directory, exist_ok=True)
        shutil.copy(os.path.join(os.path.dirname(os.path.dirname(SELFTEST_HERE)), ".claude", "hooks", "lib_work_root.py"), hooks_directory)
        frozen_work_root = os.path.join(work, "frozen-work-root")
        frozen_member = os.path.join(frozen_work_root, REPLAY_FROZEN_ROOTS_DIRECTORY, "e900-frozen", "crates", "member.rs")
        write_text(frozen_member, "fn member() {}\n")
        frozen_definition = "replay_frozen_root() { printf '%s\\n' \"$1\"; }\ndriver_e900() {\n  frozen_root=\"$(replay_frozen_root e900-frozen)\" || return 1\n"
        edit_once(os.path.join(work, REPLAY_SCRIPT), "driver_e900() {\n", frozen_definition)
        try:
            frozen_environment = {"SINGLEFS_WORK_ROOT": frozen_work_root}
            broken_environment = dict(frozen_environment, **{BREAK_VARIABLE: "frozen-root-unkeyed"})
            exit_code, frozen_before, output = row_keys(["--gpu", "0"], frozen_environment)
            _exit_code, frozen_again, _output = row_keys(["--gpu", "0"], frozen_environment)
            _exit_code, broken_frozen_before, _output = row_keys(["--gpu", "0"], broken_environment)
            write_text(frozen_member, "fn member() { 1; }\n")
            _exit_code, frozen_after, _output = row_keys(["--gpu", "0"], frozen_environment)
            _exit_code, broken_frozen_after, _output = row_keys(["--gpu", "0"], broken_environment)
            selftest.expect("行键：驱动点名冻结根 ⇒ E900 算得出行键；冻结根一个字节没改，两趟键相同",
                            exit_code == 0 and re.fullmatch(r"[0-9a-f]{64}", frozen_before.get("E900", "")) is not None and frozen_before == frozen_again,
                            f"退 {exit_code}，第一趟 {frozen_before.get('E900')}，第二趟 {frozen_again.get('E900')}：{output.strip()[-400:]}")
            frozen_changed = sorted(row for row in frozen_before if frozen_after.get(row) != frozen_before[row])
            selftest.expect("行键：驱动点名冻结根 ⇒ 改冻结根里一个文件的一个字节，变的只有 E900", frozen_changed == ["E900"], f"实际变的是 {frozen_changed or '无'}")
            selftest.expect("弄坏开关 frozen-root-unkeyed 下那一格转红（冻结根里的字节改了，E900 的行键不变）",
                            broken_frozen_before.get("E900") and broken_frozen_before.get("E900") == broken_frozen_after.get("E900"),
                            f"改前 {broken_frozen_before.get('E900')}，改后 {broken_frozen_after.get('E900')}")
        finally:
            edit_once(os.path.join(work, REPLAY_SCRIPT), frozen_definition, "driver_e900() {\n")
    except AssertionError as error:
        selftest.expect("行键的样本改得动", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)

def crash_blocks_log(blocks, reused, fresh, red, refused, lines=1):
    return "running 1 test\n" + f"CRASH_CASE_BLOCKS blocks={blocks} reused={reused} fresh={fresh} red={red} refused={refused}\n" * lines + PASSED_ONE_LINE + "\n"

def crash_blocks_streams_log(*streams):
    """多流用例的日志：每条流一行 CRASH_CASE_BLOCKS，streams 每项是 (blocks, reused, fresh, red, refused)。"""
    return ("running 1 test\n" + "".join(f"CRASH_CASE_BLOCKS blocks={blocks} reused={reused} fresh={fresh} red={red} refused={refused}\n"
                                         for blocks, reused, fresh, red, refused in streams) + PASSED_ONE_LINE + "\n")

def run_crash_case_blocks_cells(selftest, module):
    """CRASH_CASE_BLOCKS 行（文件头「崩溃枚举用例的复用不在这里」）：有红退 1，无红退 0；
    多流用例每条流一行、逐行收按用例合计；一行都没有、有一行数对不上（blocks = reused + fresh + refused）退 4
    （弄坏开关 blocks-count-unchecked、blocks-count-without-refused、blocks-single-line-only 下各自那一格转红）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-blocks-")
    try:
        log_path = os.path.join(work, "case.log")

        def blocks(log_text, changes=None):
            write_text(log_path, log_text)
            arguments = [sys.executable, module, "crash-case-blocks", work, log_path]
            return run_quietly(arguments, changes)[:2]
        cells = [("平常跑：复用 3 块、现判 1 块、无红", crash_blocks_log(4, 3, 1, 0, 0), 0),
                 ("平常跑：现判的块里 1 块红", crash_blocks_log(4, 3, 1, 1, 0), EXIT_BLOCKS_RED),
                 ("日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", EXIT_BLOCKS_LINE_BAD),
                 ("平常跑：拒算 1 块计在 refused、数对得上", crash_blocks_log(4, 3, 0, 0, 1), 0),
                 ("多流用例每条流一行：逐行收、按用例合计", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), 0),
                 ("多流用例里一条流有红", crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 1, 0)), EXIT_BLOCKS_RED),
                 ("多流用例里一行数对不上", crash_blocks_streams_log((3, 3, 0, 0, 0), (3, 1, 1, 0, 0)), EXIT_BLOCKS_LINE_BAD),
                 ("blocks 不等于 reused + fresh + refused", crash_blocks_log(5, 3, 1, 0, 0), EXIT_BLOCKS_LINE_BAD),
                 ("平常跑：blocks = 0（一块都没判）", crash_blocks_log(0, 0, 0, 0, 0), EXIT_BLOCKS_LINE_BAD),
                 ("多流用例两行合计 blocks = 0", crash_blocks_streams_log((0, 0, 0, 0, 0), (0, 0, 0, 0, 0)), EXIT_BLOCKS_LINE_BAD),
                 ("多流用例一条流 0 块、另一条流 2 块有判定：合计不是 0", crash_blocks_streams_log((0, 0, 0, 0, 0), (2, 2, 0, 0, 0)), 0)]
        # 合计 blocks = 0 的两格另核退 4 的原因是「一块都没判」：多流用例两行都是 0 块时，按行数判的那一支（弄坏开关 blocks-single-line-only）也退 4，
        # 只看退出码分不出是合计那一判在起作用
        reasons = {"平常跑：blocks = 0（一块都没判）": "一块都没判", "多流用例两行合计 blocks = 0": "一块都没判"}
        for label, log_text, expected in cells:
            exit_code, output = blocks(log_text)
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected and reasons.get(label, "") in output, f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)))
        selftest.expect("crash-case-blocks：多流用例的一句报两条流合计的块数（块 5、复用 4、现判 1）",
                        "块 5（2 条流合计）：复用 4、现判 1" in output, output.strip())
        exit_code, output = blocks(crash_blocks_log(0, 0, 0, 0, 0), {BREAK_VARIABLE: "zero-blocks-covered"})
        selftest.expect("弄坏开关 zero-blocks-covered 下「平常跑：blocks = 0」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_log(5, 3, 1, 0, 0), {BREAK_VARIABLE: "blocks-count-unchecked"})
        selftest.expect("弄坏开关 blocks-count-unchecked 下「blocks 不等于 reused + fresh + refused」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_log(4, 3, 0, 0, 1), {BREAK_VARIABLE: "blocks-count-without-refused"})
        selftest.expect("弄坏开关 blocks-count-without-refused 下「平常跑：拒算 1 块计在 refused」那一格转红（退 4）", exit_code == EXIT_BLOCKS_LINE_BAD,
                        f"退 {exit_code}：{output.strip()}")
        exit_code, output = blocks(crash_blocks_streams_log((3, 3, 0, 0, 0), (2, 1, 1, 0, 0)), {BREAK_VARIABLE: "blocks-single-line-only"})
        selftest.expect("弄坏开关 blocks-single-line-only 下「多流用例每条流一行」那一格转红（退 4）", exit_code == EXIT_BLOCKS_LINE_BAD,
                        f"退 {exit_code}：{output.strip()}")

        # --ran（跑登记了 red-blocks=judged-by-test 的用例）：只核行在、数对得上，红块与现判块不判
        def ran_blocks(log_text, extra=(), changes=None):
            write_text(log_path, log_text)
            return run_quietly([sys.executable, module, "crash-case-blocks", work, log_path, "--ran", *extra], changes)[:2]
        ran_cells = [("--ran：4 块里现判 3、红 2（用例认下的红）", crash_blocks_log(4, 1, 3, 2, 0), 0),
                     ("--ran：日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", EXIT_BLOCKS_LINE_BAD),
                     ("--ran：blocks 不等于 reused + fresh + refused", crash_blocks_log(5, 1, 3, 2, 0), EXIT_BLOCKS_LINE_BAD),
                     ("--ran：多流用例两行合计", crash_blocks_streams_log((3, 3, 0, 1, 0), (2, 1, 1, 1, 0)), 0),
                     ("--ran：blocks = 0（一块都没判）", crash_blocks_log(0, 0, 0, 0, 0), EXIT_BLOCKS_LINE_BAD)]
        for label, log_text, expected in ran_cells:
            exit_code, output = ran_blocks(log_text)
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}：{output.strip()}")
        exit_code, output = ran_blocks(crash_blocks_log(4, 1, 3, 2, 0))
        selftest.expect("crash-case-blocks --ran：一句照平常跑的写法报块数（块 4：复用 1、现判 3、红 2、拒算 0）",
                        output.strip() == "块 4：复用 1、现判 3、红 2、拒算 0", output.strip())
        exit_code, output = ran_blocks(crash_blocks_log(4, 4, 0, 0, 0), ("--" + "query",))
        selftest.expect("crash-case-blocks：带已删的 --ran 之外的选项 ⇒ 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}：{output.strip()}")
        exit_code, output = ran_blocks(crash_blocks_log(0, 0, 0, 0, 0), changes={BREAK_VARIABLE: "zero-blocks-covered"})
        selftest.expect("弄坏开关 zero-blocks-covered 下「--ran：blocks = 0」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")
        exit_code, output = ran_blocks("running 1 test\n" + PASSED_ONE_LINE + "\n", changes={BREAK_VARIABLE: "ran-blocks-line-optional"})
        selftest.expect("弄坏开关 ran-blocks-line-optional 下「--ran：日志里没有那一行」那一格转红（退 0）", exit_code == 0, f"退 {exit_code}：{output.strip()}")

        # --no-line-registered（跑登记了 blocks-line=not-printed-by-the-case 的用例）：只放宽一行都没有那一种，有那一行的照判；没带这个选项的 0 行由上面「日志里没有那一行」钉退 4
        no_line_cells = [("--no-line-registered：日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", 0, CRASH_CASE_BLOCKS_LINE_NOT_PRINTED),
                         ("--no-line-registered：有那一行、无红", crash_blocks_log(4, 3, 1, 0, 0), 0, "块 4：复用 3、现判 1、红 0、拒算 0"),
                         ("--no-line-registered：有那一行、现判的块里 1 块红", crash_blocks_log(4, 3, 1, 1, 0), EXIT_BLOCKS_RED, "红 1")]
        for label, log_text, expected, reason in no_line_cells:
            write_text(log_path, log_text)
            exit_code, output = run_quietly([sys.executable, module, "crash-case-blocks", work, log_path, "--no-line-registered"])[:2]
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected and reason in output, f"退 {exit_code}：{output.strip()}")

        # --recorded-only（只录的那一趟，crash-case-command --stage record 起的）：已录、待判，不判红也不当判过；录入段判了块、数对不上退 4
        recorded_only_cells = [("--recorded-only：日志里没有那一行", "running 1 test\n" + PASSED_ONE_LINE + "\n", 0, "只录：已录、待判"),
                               ("--recorded-only：块那一行现判 0、红 0", crash_blocks_log(4, 4, 0, 0, 0), 0, "只录：已录、待判"),
                               ("--recorded-only：块那一行现判 1（录入段判了块）", crash_blocks_log(4, 3, 1, 0, 0), EXIT_BLOCKS_LINE_BAD, "录入段不该判"),
                               ("--recorded-only：块那一行数对不上", crash_blocks_log(5, 4, 0, 0, 0), EXIT_BLOCKS_LINE_BAD, "blocks 不等于")]
        for label, log_text, expected, reason in recorded_only_cells:
            write_text(log_path, log_text)
            exit_code, output = run_quietly([sys.executable, module, "crash-case-blocks", work, log_path, "--recorded-only"])[:2]
            selftest.expect(f"crash-case-blocks：{label} ⇒ 退 {expected}", exit_code == expected and reason in output, f"退 {exit_code}：{output.strip()}")
        write_text(log_path, "running 1 test\n" + PASSED_ONE_LINE + "\n")
        exit_code, output = run_quietly([sys.executable, module, "crash-case-blocks", work, log_path, "--recorded-only"], {BREAK_VARIABLE: "recorded-only-judged"})[:2]
        selftest.expect("弄坏开关 recorded-only-judged 下「--recorded-only：日志里没有那一行」那一格红（退 4）", exit_code == EXIT_BLOCKS_LINE_BAD, f"退 {exit_code}：{output.strip()}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_gate_precondition_cells(selftest, module):
    """门禁行的前提：齐了一个字不打；command= / readwrite= / probe= 各缺一样判红并点名；没登记退 2；种类写串了自查点得出。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-gate-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        present_tool = os.path.join(tools, "selftest-present-tool")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        device = os.path.join(work, "device.bin")
        kernel = os.path.join(work, "kernel.img")
        write_text(device, "")
        write_text(kernel, "")
        write_text(os.path.join(work, "tools/check.sh"),
                   'if [[ "${1:-}" == --check && -f kernel.img ]]; then exit 0; fi\necho "  ✗ 找不到内核镜像（样本）"\nexit 1\n')
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "55-demo.sh\tcrates/\tcommand=selftest-present-tool readwrite=device.bin probe=tools/check.sh:--check\t# 样本：三个前置\n")

        def preconditions(stage="55-demo.sh", extra_environment=None):
            changes = {"PATH": tools + os.pathsep + os.environ.get("PATH", "")}
            changes.update(extra_environment or {})
            return run_quietly([sys.executable, module, "gate-preconditions", work, stage], changes)

        exit_code, output, messages = preconditions()
        selftest.expect("门禁前提都齐时退 0、stdout 一个字不打", exit_code == 0 and output == "",
                        f"退 {exit_code}，stdout「{output.strip()}」，stderr：{messages.strip()}")
        os.remove(present_tool)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 command= 缺了判红（退 1）：点名那个可执行名，带出路与那一行的注释",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "✗ 前提没齐" in output and "selftest-present-tool 不在 PATH 里" in output
                        and "→ 怎么办" in output and "样本：三个前置" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = preconditions(extra_environment={BREAK_VARIABLE: "skip-gate-preconditions"})
        selftest.expect("弄坏开关 skip-gate-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0,
                        f"弄坏之后仍退 {exit_code}：这一格判不出前提被跳过")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        os.chmod(device, 0o444)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 readwrite= 不可写判红（退 1）并点名那个路径（以 root 跑这一格判不出）",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "device.bin 不在，或当前用户不可读写" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        os.chmod(device, 0o644)
        os.remove(kernel)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 probe= 退非 0 判红（退 1），明细带出探针自己打的那几行",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "tools/check.sh --check 退 1" in output and "找不到内核镜像（样本）" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = preconditions(stage="99-unregistered.sh")
        selftest.expect("登记表里没有那一道时退 2，照样打 ✗ 与出路", exit_code == EXIT_REGISTRATION_ERROR and "✗" in output and "→ 怎么办" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        mixed_rows = [RegistrationRow("55-demo.sh", ["crates/"], ["question-row=a.md#1:x"], 1, ""),
                      RegistrationRow("E950", ["crates/"], ["command=cargo"], 2, ""),
                      RegistrationRow("57-demo.sh", ["crates/"], ["probe=tools/nowhere.sh"], 3, "")]
        problems = lint_registration_rows(mixed_rows, work)
        selftest.expect("自查点得出门禁行写实验的种类、实验行写门禁的种类、probe= 指的脚本不在",
                        any(problem.startswith("第 1 行") and "认不出的门禁准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 2 行") and "认不出的准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 3 行") and "tools/nowhere.sh 不在" in problem for problem in problems),
                        f"自查结果：{problems}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)


FAKE_HERD7_SCRIPT = '#!/usr/bin/env bash\nif [[ "${1:-}" == -version ]]; then echo "herd7 version 7.57-selftest-%s"; exit 0; fi\nexit 2\n'


def run_crash_cases_listing_cells(selftest, module):
    """crash-cases：逐条打键、包、测试目标、用例函数；登记行自查（写法、第二列写 -、用例函数找不到、目标不在、包不在工作区、test= 两条、同键两行）；
    用例函数的每一处定义都不许标 #[ignore]（cfg 二选一、子模块里同名、# [ignore]、顺着 include! / #[path] / mod 带进来的源文件）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-crash-case-")
    try:
        own_row = "crash-case:own\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY exhaustive=CRASH_POINT_REPLAY\t# 样本"
        build_crash_case_repository(work, [own_row])
        table_path = os.path.join(work, REGISTRATION_TABLE)

        def admission(*arguments, changes=None):
            return run_quietly([sys.executable, module, *arguments], changes)

        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases 逐条打「键、包、测试目标、用例函数」", exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        lint_rows = [
            ("用例函数在测试目标里找不到", "crash-case:own\t-\ttest=pkg:own_case:no_such_function\t# 样本", "找不到"),
            ("测试目标不在", "crash-case:own\t-\ttest=pkg:no_such_target:the_case\t# 样本", "不在"),
            ("包不是工作区成员", "crash-case:own\t-\ttest=nowhere:own_case:the_case\t# 样本", "不是仓根 Cargo.toml 的工作区成员"),
            ("test= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case test=pkg:own_case:the_case\t# 样本", "恰好一条 test="),
            ("exhaustive= 点名的前缀没登记成 count-line=", "crash-case:own\t-\ttest=pkg:own_case:the_case exhaustive=CRASH_POINT_REPLAY\t# 样本", "没有登记成 count-line="),
            ("认不出的条件", "crash-case:own\t-\ttest=pkg:own_case:the_case command=cargo\t# 样本", "认不出"),
            ("同一个键两行", own_row + "\n" + own_row, "一条用例只许一行"),
            ("第二列写了路径（用例一级不按路径登记输入）", "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case\t# 样本", "第二列写 -"),
            ("shards-at-least= 写了 0", "crash-case:own\t-\ttest=pkg:own_case:the_case shards-at-least=0\t# 样本", "要写正整数"),
            ("shards-at-least= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case shards-at-least=4 shards-at-least=8\t# 样本",
             "shards-at-least= 至多一条"),
            ("标记名写错（red-block= 少一个 s）", "crash-case:own\t-\ttest=pkg:own_case:the_case red-block=judged-by-test\t# 样本", "认不出"),
            ("red-blocks= 写了 judged-by-test 之外的值", "crash-case:own\t-\ttest=pkg:own_case:the_case red-blocks=judged-by-library\t# 样本",
             "red-blocks= 只认 judged-by-test"),
            ("red-blocks= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case red-blocks=judged-by-test red-blocks=judged-by-test\t# 样本",
             "red-blocks= 至多一条"),
            ("findings= 写了 not-written-by-the-case 之外的值", "crash-case:own\t-\ttest=pkg:own_case:the_case findings=written-elsewhere\t# 样本",
             "findings= 只认 not-written-by-the-case"),
            ("findings= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case findings=not-written-by-the-case findings=not-written-by-the-case\t# 样本",
             "findings= 至多一条"),
            ("blocks-line= 写了 not-printed-by-the-case 之外的值", "crash-case:own\t-\ttest=pkg:own_case:the_case blocks-line=optional\t# 样本",
             "blocks-line= 只认 not-printed-by-the-case"),
            ("blocks-line= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case blocks-line=not-printed-by-the-case blocks-line=not-printed-by-the-case\t# 样本",
             "blocks-line= 至多一条"),
            ("separate-run= 的理由只有 7 个字", "crash-case:own\t-\ttest=pkg:own_case:the_case separate-run=七个字的理由啊\t# 样本",
             "separate-run= 的理由至少 8 个字"),
            ("separate-run= 写了两条", "crash-case:own\t-\ttest=pkg:own_case:the_case separate-run=八个字的理由正好 separate-run=另一条八个字的理由\t# 样本",
             "separate-run= 至多一条"),
            ("separate-run= 的理由带了空白（切开之后后半截认不出）", "crash-case:own\t-\ttest=pkg:own_case:the_case separate-run=CPU判器参照全域 跑得久单独交\t# 样本",
             "认不出 跑得久单独交"),
        ]
        for label, rows_text, expected_phrase in lint_rows:
            write_text(table_path, rows_text + "\n")
            exit_code, output, _messages = admission("crash-cases", work)
            selftest.expect(f"crash-cases 自查：{label}，退 2 并说出来", exit_code == EXIT_REGISTRATION_ERROR and expected_phrase in output,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        # red-blocks=judged-by-test（文件头「崩溃枚举用例行」）：带标记的行 crash-cases 照常列出，crash-case-judged-by-test 对它退 0、对没带的退 1
        marked_row = "crash-case:marked\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY red-blocks=judged-by-test\t# 样本"
        write_text(table_path, own_row + "\n" + marked_row + "\n")
        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases：带 red-blocks=judged-by-test 的行解析得出、照常逐条列出",
                        exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\ncrash-case:marked\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        marked_case = crash_case_of_key(read_registration_rows(work), "crash-case:marked")
        plain_case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
        selftest.expect("parse_crash_case：带标记的行 red_blocks_judged_by_test 为真、没带的为假",
                        marked_case.red_blocks_judged_by_test and not plain_case.red_blocks_judged_by_test,
                        f"带标记的 {marked_case.red_blocks_judged_by_test}，没带的 {plain_case.red_blocks_judged_by_test}")
        for key, expected in (("crash-case:marked", 0), ("crash-case:own", 1), ("crash-case:missing", EXIT_REGISTRATION_ERROR)):
            exit_code, output, _messages = admission("crash-case-judged-by-test", work, key)
            selftest.expect(f"crash-case-judged-by-test {key} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}，stdout「{output.strip()}」")
        # shards-at-least=（文件头「崩溃枚举用例行」）：点名的取点到的里最大的，不给键取全部登记的里最大的，一条都没登记打 0，键不在退 2
        many_shards_row = "crash-case:many-shards\t-\ttest=pkg:own_case:the_case shards-at-least=16\t# 样本"
        some_shards_row = "crash-case:some-shards\t-\ttest=pkg:own_case:the_case shards-at-least=7\t# 样本"
        write_text(table_path, own_row + "\n" + many_shards_row + "\n" + some_shards_row + "\n")
        for label, keys, expected_exit, expected_output in (
                ("点名登记了 16 的那一条 ⇒ 16", ("crash-case:many-shards",), 0, "16\n"),
                ("点名登记了 7 的与没登记的 ⇒ 7", ("crash-case:some-shards", "crash-case:own"), 0, "7\n"),
                ("点名没登记的那一条 ⇒ 0", ("crash-case:own",), 0, "0\n"),
                ("不给键 ⇒ 全部登记的里最大的 16", (), 0, "16\n")):
            exit_code, output, _messages = admission("crash-case-shards-at-least", work, *keys)
            selftest.expect(f"crash-case-shards-at-least：{label}", exit_code == expected_exit and output == expected_output,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = admission("crash-case-shards-at-least", work, "crash-case:missing")
        selftest.expect("crash-case-shards-at-least：键不在 ⇒ 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}，stdout「{output.strip()}」")
        # findings=、blocks-line=（文件头「崩溃枚举用例行」）：带标记的行答退 0，没带的退 1，键不在退 2；两个标记各答各的
        write_text(table_path, own_row + "\n"
                   + "crash-case:not-written\t-\ttest=pkg:own_case:the_case findings=not-written-by-the-case\t# 样本\n"
                   + "crash-case:not-printed\t-\ttest=pkg:own_case:the_case blocks-line=not-printed-by-the-case\t# 样本\n")
        mark_cells = [("crash-case-findings-not-written", "crash-case:not-written", 0), ("crash-case-findings-not-written", "crash-case:not-printed", 1),
                      ("crash-case-findings-not-written", "crash-case:own", 1), ("crash-case-findings-not-written", "crash-case:missing", EXIT_REGISTRATION_ERROR),
                      ("crash-case-blocks-not-printed", "crash-case:not-printed", 0), ("crash-case-blocks-not-printed", "crash-case:not-written", 1),
                      ("crash-case-blocks-not-printed", "crash-case:own", 1), ("crash-case-blocks-not-printed", "crash-case:missing", EXIT_REGISTRATION_ERROR)]
        for subcommand, key, expected in mark_cells:
            exit_code, output, _messages = admission(subcommand, work, key)
            selftest.expect(f"{subcommand} {key} ⇒ 退 {expected}", exit_code == expected, f"退 {exit_code}，stdout「{output.strip()}」")
        # separate-run=（文件头「崩溃枚举用例行」）：crash-cases 照常四列列出带标记的行；带 --separate-run 只打登了的那几条「键、理由」，理由恰好 8 个字照认；
        # 一条都没登记一行不打、退 0；--separate-run 之外的第二个参数退 2
        separated_row = "crash-case:separated\t-\ttest=pkg:own_case:the_case findings=not-written-by-the-case separate-run=八个字的理由正好\t# 样本"
        write_text(table_path, own_row + "\n" + separated_row + "\n")
        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases：带 separate-run= 的行解析得出、照常逐条列出四列",
                        exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\ncrash-case:separated\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = admission("crash-cases", work, "--separate-run")
        selftest.expect("crash-cases --separate-run：只答登了的那一条「键、理由」，理由恰好 8 个字照认",
                        exit_code == 0 and output == "crash-case:separated\t八个字的理由正好\n", f"退 {exit_code}，stdout「{output.strip()}」")
        separated_case = crash_case_of_key(read_registration_rows(work), "crash-case:separated")
        selftest.expect("parse_crash_case：带 separate-run= 的行 separate_run_reason 是理由、同一行的 findings= 照认",
                        separated_case.separate_run_reason == "八个字的理由正好" and separated_case.findings_not_written_by_the_case,
                        f"理由「{separated_case.separate_run_reason}」，findings_not_written_by_the_case {separated_case.findings_not_written_by_the_case}")
        exit_code, output, _messages = admission("crash-cases", work, "--separate")
        selftest.expect("crash-cases：项目根之后的参数不是 --separate-run ⇒ 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}，stdout「{output.strip()}」")
        write_text(table_path, own_row + "\n")
        exit_code, output, _messages = admission("crash-cases", work, "--separate-run")
        selftest.expect("crash-cases --separate-run：一条都没登记 ⇒ 一行不打、退 0", exit_code == 0 and output == "", f"退 {exit_code}，stdout「{output.strip()}」")

        # 登记的用例函数不许标 #[ignore]：标了的（含 # [ignore]、同名几处里有一处标了的）crash-cases 退 2，没标的、只有 cfg_attr 里条件 ignore 的、ignore 挂在前一个函数上的、只写在注释里的退 0
        own_case_path = os.path.join(work, "crates/pkg/tests/own_case.rs")
        with open(own_case_path, encoding="utf-8") as handle:
            own_case_text = handle.read()
        modules = "mod common;\nmod helper_target;\n"
        cfg_split_case = "#[cfg(debug_assertions)]\n#[test]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\n#[ignore]\nfn the_case() {}\n"
        submodule_after_case = "#[test]\nfn the_case() {}\nmod slow {\n    #[test]\n    #[ignore]\n    fn the_case() {}\n}\n"
        spaced_ignore_case = "#[test]\n# [ignore]\nfn the_case() {}\n"
        ignore_cells = [
            ("用例函数没标 #[ignore]", modules + "#[test]\nfn the_case() {}\n", False),
            ("只有 cfg_attr 里的条件 ignore", modules + "#[test]\n#[cfg_attr(miri, ignore)]\nfn the_case() {}\n", False),
            ("#[ignore] 挂在前一个函数上", modules + "#[test]\n#[ignore]\nfn other() {}\n#[test]\nfn the_case() {}\n", False),
            ("#[ignore] 只写在注释里", modules + "#[test]\n// #[ignore]\nfn the_case() {}\n", False),
            ("#[ignore] 在 #[test] 前面、fn 前面有 pub", modules + "#[ignore]\n#[test]\npub fn the_case() {}\n", True),
            ("ignore 的理由里带方括号、引号与 //", modules + '#[test]\n#[ignore = "样本 [x] \\"y\\" // z"]\nfn the_case() {}\n', True),
            ("同名函数 cfg 二选一：debug 那一份不标、release 那一份标 ignore", modules + cfg_split_case, True),
            ("子模块里同名标 ignore 的写在顶层那一份（--exact the_case 跑的）后面", modules + submodule_after_case, True),
            ("# [ignore]（# 与 [ 之间有空格，rustc 认）", modules + spaced_ignore_case, True),
            ("同名的两处都没标 ignore（cfg 二选一）", modules + cfg_split_case.replace("#[test]\n#[ignore]\nfn the_case", "#[test]\nfn the_case"), False),
        ]
        for label, source_text, marked in ignore_cells:
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work)
            green = (exit_code == EXIT_REGISTRATION_ERROR and "标了 #[ignore]" in output) if marked else exit_code == 0
            selftest.expect(f"crash-cases 自查：{label} ⇒ {'退 2 并说标了 #[ignore]' if marked else '退 0'}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        for switch, label, source_text, broken_exit_code in (
                ("first-definition-only", "同名函数 cfg 二选一：debug 那一份不标、release 那一份标 ignore", modules + cfg_split_case, 0),
                ("first-definition-only", "子模块里同名标 ignore 的写在顶层那一份（--exact the_case 跑的）后面", modules + submodule_after_case, 0),
                ("ignore-attribute-without-space", "# [ignore]", modules + spaced_ignore_case, 0)):
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: switch})
            selftest.expect(f"弄坏开关 {switch} 下「{label}」那一格红（crash-cases 退 {broken_exit_code}）", exit_code == broken_exit_code,
                            f"弄坏之后退 {exit_code}：这一格分不出差别；stdout「{output.strip()}」")
        # 顺着 include!("<字面路径>") / #[path = "…"] mod / mod <名>; 带进来的源文件一起判；判不出的报判不出（弄坏开关 target-own-files-only 下头一格红）
        unmarked_here = "#[test]\nfn the_case() {}\n"
        release_marked = {"crates/pkg/tests/common/release_case.rs": "#[test]\n#[ignore]\nfn the_case() {}\n"}
        cfg_include = ("#[cfg(debug_assertions)]\n" + unmarked_here + "#[cfg(not(debug_assertions))]\ninclude!(\"common/release_case.rs\");\n")
        included_cells = [
            ("debug 下不标、release 下 include!(\"common/release_case.rs\") 进来一份标 ignore 的同名用例", modules + cfg_include, release_marked,
             "标了 #[ignore]"),
            ("include! 的参数是 concat!(…)（读的是哪一份判不出）", modules + unmarked_here + 'include!(concat!("common/", "release_case.rs"));\n',
             release_marked, "判不出"),
            ("#[path = \"common/p.rs\"] mod p; 里有一份标 ignore 的同名用例", modules + unmarked_here + '#[path = "common/p.rs"]\nmod p;\n',
             {"crates/pkg/tests/common/p.rs": "#[test]\n#[ignore]\nfn the_case() {}\n"}, "标了 #[ignore]"),
            ("mod 指的文件找不到", modules + unmarked_here + "mod missing_module;\n", {}, "判不出"),
            ("use core::include as pull; 把 include! 改名引进来", modules + "use core::include as pull;\n" + unmarked_here, {}, "判不出"),
            ("对照：include! 进来的同名用例也没标 ignore", modules + cfg_include.replace("release_case.rs", "plain_case.rs"),
             {"crates/pkg/tests/common/plain_case.rs": unmarked_here}, None),
        ]
        broken_exit_code, broken_output = None, ""
        for label, source_text, extra_files, phrase in included_cells:
            write_text(own_case_path, source_text)
            for relative, content in extra_files.items():
                write_text(os.path.join(work, relative), content)
            exit_code, output, _messages = admission("crash-cases", work)
            if label == included_cells[0][0]:
                broken_exit_code, broken_output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: "target-own-files-only"})
            for relative in extra_files:
                os.remove(os.path.join(work, relative))
            green = exit_code == 0 if phrase is None else (exit_code == EXIT_REGISTRATION_ERROR and phrase in output)
            selftest.expect(f"crash-cases 自查（带进来的源文件）：{label} ⇒ {'退 0' if phrase is None else '退 2 并说' + phrase}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        selftest.expect("弄坏开关 target-own-files-only 下「release 下 include! 进来一份标 ignore 的同名用例」那一格红（crash-cases 退 0）", broken_exit_code == 0,
                        f"弄坏之后退 {broken_exit_code}：这一格分不出差别；stdout「{broken_output.strip()}」")
        # 目录式测试目标 tests/own_case/main.rs：只交 main.rs，子模块 flows.rs 顺着 mod 带进来，flows.rs 里再嵌一层的 mod nested; 住在 flows/ 底下：找得到、里面标 ignore 的同名用例报出来
        #   （弄坏开关 directory-target-every-file-as-root 下目录里每份 .rs 都当根文件，flows.rs 里的 mod nested; 按目录算成 own_case/nested.rs，报判不出）
        os.remove(own_case_path)
        for relative, source_text in (("main.rs", "mod flows;\n"), ("flows.rs", unmarked_here + "mod nested;\n"), ("flows/nested.rs", "#[test]\n#[ignore]\nfn the_case() {}\n")):
            write_text(os.path.join(work, "crates/pkg/tests/own_case", relative), source_text)
        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases 自查（目录式测试目标）：tests/own_case/main.rs 声明 mod flows;、flows.rs 里再声明 mod nested; ⇒ 找得到嵌套的文件、退 2 并说标了 #[ignore]",
                        exit_code == EXIT_REGISTRATION_ERROR and "标了 #[ignore]" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        shutil.rmtree(os.path.join(work, "crates/pkg/tests/own_case"))
        write_text(own_case_path, own_case_text)
        exit_code, output, _messages = admission("crash-case-command", work, "crash-case:own")
        words = output.split("\0")
        selftest.expect("crash-case-command：调用方环境里的跑法变量清掉、命令里不设它的值",
                        exit_code == 0 and CRASH_LIBRARY_RUN_KIND_VARIABLE in words and not any(word.startswith(CRASH_LIBRARY_RUN_KIND_VARIABLE + "=") for word in words),
                        f"退 {exit_code}，交的词 {words}")
        exit_code, output, _messages = admission("crash-case-command", work, "crash-case:own", "--" + "start-over")
        selftest.expect("crash-case-command：带已删的参数 ⇒ 当认不出的参数，退 2、不交命令", exit_code == EXIT_REGISTRATION_ERROR and "cargo" not in output.split("\0"),
                        f"退 {exit_code}，输出 {output.strip()[:200]}")
        # 交命令之前判 RocksDB 预编库的取值（rocksdb_entry_problem）：小仓另放一份依赖 rocksdb 的 验证包清单、进程环境里四个取值是空的
        #   ⇒ 不交命令（stdout 里没有 cargo 那个词）、退 2、说出 entry 的红句；别的格小仓里没有这份清单，entry 报不判
        rocksdb_manifest = os.path.join(work, "crates", "singlefs-verification", "Cargo.toml")
        os.makedirs(os.path.dirname(rocksdb_manifest), exist_ok=True)
        write_text(rocksdb_manifest, '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
        exit_code, output, _messages = admission("crash-case-command", work, "crash-case:own",
                                                 changes={"ROCKSDB_LIB_DIR": "", "ROCKSDB_STATIC": "", "SNAPPY_LIB_DIR": "", "SNAPPY_STATIC": ""})
        selftest.expect("crash-case-command：验证包依赖 rocksdb 而进程环境里没有预编库的取值 ⇒ 不交命令、退 2",
                        exit_code == EXIT_REGISTRATION_ERROR and "cargo" not in output.split("\0") and "rocksdb-prebuilt entry：" in output,
                        f"退 {exit_code}，stdout「{output.strip()[:600]}」")
        shutil.rmtree(os.path.dirname(rocksdb_manifest))
        run_crash_case_command_stage_variable_cells(selftest, work)
    finally:
        shutil.rmtree(work, ignore_errors=True)

SAMPLE_MACHINE_CORES = 32


def selftest_default_threads():
    """自证里当「每台默认重活线程数」用的数：本仓登记的那一个（不认调用方的 SINGLEFS_THREAD_CAP；判日志的纯函数那几格显式传它）；读不出交 None。"""
    try:
        return registered_heavy_threads({})
    except InputManifestError:
        return None


def synthetic_crash_point_replay_log(states=100, slices=64, worker_threads=None, resumed_slices=0, freshly_run_slices=64, exhaustive="true",
                         result="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s",
                         crash_point_replay_lines=1, with_checker=True, with_finished=True, finished_fields=None, configured_worker_threads=None,
                         worker_threads_source="environment_variable"):
    """合成的一趟用例日志：CRASH_POINT_REPLAY_PARALLEL_FINISHED、CRASH_POINT_REPLAY 计数行、CHECKER 行、test result 各几行由参数定。
    起的与配的线程数不给都取每台默认重活线程数（selftest_default_threads）。"""
    default_threads = selftest_default_threads() or 1
    worker_threads = default_threads if worker_threads is None else worker_threads
    configured_worker_threads = default_threads if configured_worker_threads is None else configured_worker_threads
    lines = [f"CRASH_POINT_REPLAY_PARALLEL_START states={states} slices={slices}"]
    if with_finished:
        fields = finished_fields or (f"worker_threads={worker_threads} configured_worker_threads={configured_worker_threads} "
                                     f"worker_threads_source={worker_threads_source} "
                                     f"resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices}")
        lines.append(f"CRASH_POINT_REPLAY_PARALLEL_FINISHED states={states} slices={slices} {fields} progress_file_after_completion=deleted elapsed_seconds=1.0")
    lines += [f"CRASH_POINT_REPLAY states={states} closed_form={states} violations=0 exhaustive={exhaustive}"] * crash_point_replay_lines
    if with_checker:
        lines.append("CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0")
    if result is not None:
        lines.append(result)
    return "\n".join(lines) + "\n"


def run_crash_case_narrowing_cells(selftest, module):
    """缩过域的那一趟（文件头 crash-case-judge）：环境里设了 CRASH_CASE_NARROWING_VARIABLES 之一的，计数行 exhaustive=false 不判、成功句写缩过域、不算门禁通过；
    这一行没有 exhaustive= 字段的照判红；没设的 exhaustive=false 判红。环境是假的（从 os.environ 去掉这两个变量再设），不看起自证那一方设没设。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-narrowing-")
    try:
        build_crash_case_repository(work, ["crash-case:own\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY exhaustive=CRASH_POINT_REPLAY\t# 样本"])
        log_path = os.path.join(work, "case.log")
        recorded_path = os.path.join(work, "recorded")
        no_narrowing = {name: value for name, value in os.environ.items() if name not in CRASH_CASE_NARROWING_VARIABLES}
        count_line = "CRASH_POINT_REPLAY states=53 closed_form=16777268 violations=0"
        narrowed_sentence = "这一趟缩过域、不算门禁通过（"
        expand_up_to, judge_expand_up_to = CRASH_CASE_NARROWING_VARIABLES
        cells = [(f"设了 {expand_up_to}=5、那一行 exhaustive=false", {expand_up_to: "5"}, " exhaustive=false", 0, narrowed_sentence + f"{expand_up_to}=5）", True),
                 (f"设了 {judge_expand_up_to}=3、那一行 exhaustive=false", {judge_expand_up_to: "3"}, " exhaustive=false", 0, narrowed_sentence + f"{judge_expand_up_to}=3）", True),
                 (f"设了 {expand_up_to}=5、那一行 exhaustive=true", {expand_up_to: "5"}, " exhaustive=true", 0, narrowed_sentence + f"{expand_up_to}=5）", True),
                 (f"设了 {expand_up_to}=5、那一行没有 exhaustive= 字段", {expand_up_to: "5"}, "", 1, "不带 exhaustive=true", False),
                 ("没设缩域变量、那一行 exhaustive=false", {}, " exhaustive=false", 1, "不带 exhaustive=true", False),
                 ("没设缩域变量、那一行 exhaustive=true", {}, " exhaustive=true", 0, "", False)]
        for label, settings, exhaustive_field, expected, reason, says_narrowed in cells:
            write_text(log_path, count_line + exhaustive_field + "\n" + PASSED_ONE_LINE + "\n")
            exit_code, output, _messages = run_in_environment(
                [sys.executable, module, "crash-case-judge", work, "crash-case:own", log_path, recorded_path], dict(no_narrowing, **settings))
            selftest.expect(f"crash-case-judge 缩域：{label} ⇒ 退 {expected}",
                            exit_code == expected and reason in output and (narrowed_sentence in output) == says_narrowed,
                            f"退 {exit_code}，stdout「{output.strip()}」")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_crash_case_log_cells(selftest):
    """跑一条用例的日志怎么判（纯函数，合成日志）：test result 恰好 1 passed、计数行恰好一行、exhaustive=true、工作线程按这一趟跑了几片判。"""
    case = parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", "count-line=CRASH_POINT_REPLAY",
                                                                             "count-line=CHECKER", "exhaustive=CRASH_POINT_REPLAY", "threads=CRASH_POINT_REPLAY"], 1, ""))
    default = selftest_default_threads()
    selftest.expect("每台默认重活线程数：本仓的登记读得出（判日志那几格的线程数从它来）", default is not None, f"{HEAVY_THREAD_DEFAULT_FILE} 读不出")
    default = default or 1
    cores = SAMPLE_MACHINE_CORES
    cells = [
        (f"全量、{default} 个线程跑了 64 片", synthetic_crash_point_replay_log(), cores, False, True),
        ("全部 64 片从进度文件读回、起 0 个线程", synthetic_crash_point_replay_log(worker_threads=0, resumed_slices=64, freshly_run_slices=0), cores, False, True),
        ("读回 63 片、只剩 1 片要跑、起 1 个线程", synthetic_crash_point_replay_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), cores, False, True),
        (f"跑了 64 片却只起 1 个线程（本机 {cores} 核、线程数没显式设）", synthetic_crash_point_replay_log(worker_threads=1), cores, False, False),
        ("跑了 64 片只起 1 个线程，线程数显式设成 1", synthetic_crash_point_replay_log(worker_threads=1), cores, True, True),
        ("跑了 64 片只起 1 个线程，本机只有 1 核", synthetic_crash_point_replay_log(worker_threads=1), 1, False, True),
        ("跑了 5 片却报起了 0 个线程", synthetic_crash_point_replay_log(worker_threads=0, resumed_slices=59, freshly_run_slices=5), cores, False, False),
        ("读回的片 + 这一趟跑的片 ≠ 总片数", synthetic_crash_point_replay_log(resumed_slices=10, freshly_run_slices=10), cores, False, False),
        ("CRASH_POINT_REPLAY_PARALLEL_FINISHED 缺续跑片数字段", synthetic_crash_point_replay_log(finished_fields=f"worker_threads={default}"), cores, False, False),
        ("没有状态数对得上的 CRASH_POINT_REPLAY_PARALLEL_FINISHED", synthetic_crash_point_replay_log(with_finished=False), cores, False, False),
        ("过滤之后一条用例都没跑（0 passed）",
         synthetic_crash_point_replay_log(result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s"), cores, False, False),
        ("没有 test result 行", synthetic_crash_point_replay_log(result=None), cores, False, False),
        ("不是全量（exhaustive=false）", synthetic_crash_point_replay_log(exhaustive="false"), cores, False, False),
        ("CRASH_POINT_REPLAY 计数行抄了两遍", synthetic_crash_point_replay_log(crash_point_replay_lines=2), cores, False, False),
        ("没有 CHECKER 行", synthetic_crash_point_replay_log(with_checker=False), cores, False, False),
        (f"线程数没显式设（来源 available_parallelism）、用例取的默认等于每台默认重活线程数 {default}",
         synthetic_crash_point_replay_log(worker_threads_source="available_parallelism"), cores, False, True),
        (f"线程数没显式设（来源 available_parallelism）、用例按核数取了 {default + 1}，不等于每台默认重活线程数 {default}",
         synthetic_crash_point_replay_log(worker_threads=default + 1, configured_worker_threads=default + 1, worker_threads_source="available_parallelism"), cores, False, False),
        (f"线程数显式设成 {default + 1}（来源 environment_variable），不按每台默认判", synthetic_crash_point_replay_log(worker_threads=default + 1,
         configured_worker_threads=default + 1), cores, False, True),
    ]
    for label, log_text, machine_cores, explicitly_one, expected_green in cells:
        problems, recorded_lines, _notes = judge_crash_case_log(case, log_text, machine_cores, explicitly_one, default)
        green = not problems
        recorded_ok = not green or (recorded_lines[0].startswith("test_result=test result: ok. 1 passed")
                                    and any(line.startswith("CRASH_POINT_REPLAY ") for line in recorded_lines)
                                    and any(line.startswith("parallel_finished=CRASH_POINT_REPLAY_PARALLEL_FINISHED ") for line in recorded_lines))
        selftest.expect(f"判用例日志：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green and recorded_ok,
                        f"判成{'绿' if green else '红'}：{problems}；记下的行 {recorded_lines}")
    with BreakSwitch("threads-by-worker-count"):
        problems, _recorded, _notes = judge_crash_case_log(
            case, synthetic_crash_point_replay_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), cores, False, default)
    selftest.expect("弄坏开关 threads-by-worker-count 下「只剩 1 片要跑、起 1 个线程」那一格红（照旧只看 worker_threads=1）", bool(problems),
                    "弄坏之后仍判绿：这一格分不出「这一趟跑了几片」")
    with BreakSwitch("threads-default-unchecked"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[-2][1], cores, False, default)
    selftest.expect(f"弄坏开关 threads-default-unchecked 下「来源 available_parallelism、按核数取了 {default + 1}」那一格判绿（不比每台默认重活线程数）",
                    not problems, f"弄坏之后仍判红：{problems}")
    # 标记里「实际起的工作线程」那一格（started_worker_threads=）：登记了 threads= 的取 CRASH_POINT_REPLAY_PARALLEL_FINISHED 里起了几个，没登记的写读不到
    problems, recorded_lines, _notes = judge_crash_case_log(case, synthetic_crash_point_replay_log(worker_threads=24), cores, False, default)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect(f"判绿时记 started_worker_threads=：登记了 threads= 的记 CRASH_POINT_REPLAY_PARALLEL_FINISHED 里起了几个（24 个，不是配的 {default}）",
                    not problems and started == ["started_worker_threads=CRASH_POINT_REPLAY：24 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片）"],
                    f"判出 {problems}，记下 {started}")
    unthreaded = parse_crash_case(RegistrationRow("crash-case:plain", [], ["test=pkg:target:function", "count-line=PLAIN"], 1, ""))
    problems, recorded_lines, _notes = judge_crash_case_log(unthreaded, "PLAIN states=4\n" + PASSED_ONE_LINE + "\n", cores, False, default)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect("判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）",
                    not problems and len(started) == 1 and started[0].startswith("started_worker_threads=读不到：这条用例没登记 threads="),
                    f"判出 {problems}，记下 {started}")
    run_shard_merge_log_cells(selftest, case)


def synthetic_merge_log(shard_worker_threads=None, shard_configured=None, shard_sources=("environment_variable", "environment_variable"),
                        shard_cores=(32, 32), shard_resumed=(0, 0), shard_freshly_run=(32, 32), shards=None):
    """合成的双机分片 merge 那一趟日志：CRASH_POINT_REPLAY_PARALLEL_FINISHED 的合计照 crates/singlefs-verification/src/crash.rs 的 merge_the_shard_ledgers
    （线程与片数是各片之和、来源写 shard_ledgers），另带 shards= 与逐片的 shard_…=。逐片起的与配的线程数不给都取每台默认重活线程数（两台各一份）。"""
    default_threads = selftest_default_threads() or 1
    shard_worker_threads = shard_worker_threads or (default_threads, default_threads)
    shard_configured = shard_configured or (default_threads, default_threads)
    joined = lambda values: ",".join(str(value) for value in values)
    finished_fields = (f"worker_threads={sum(shard_worker_threads)} configured_worker_threads={sum(shard_configured)} worker_threads_source=shard_ledgers "
                       f"resumed_slices={sum(shard_resumed)} freshly_run_slices={sum(shard_freshly_run)} "
                       f"shards={shards if shards is not None else len(shard_worker_threads)} shard_worker_threads={joined(shard_worker_threads)} "
                       f"shard_configured_worker_threads={joined(shard_configured)} shard_worker_threads_sources={joined(shard_sources)} "
                       f"shard_available_parallelism={joined(shard_cores)} shard_resumed_slices={joined(shard_resumed)} "
                       f"shard_freshly_run_slices={joined(shard_freshly_run)} shard_elapsed_milliseconds=1000,1000")
    return synthetic_crash_point_replay_log(finished_fields=finished_fields)


def run_shard_merge_log_cells(selftest, case):
    """双机分片 merge 那一趟的日志：带 shards= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 逐片判工作线程（机器核数与「显式设成 1」取那一片自己的）；
    登记行写 shard= 的拒（第三列不认这个键）。"""
    default = selftest_default_threads() or 1
    cells = [
        (f"merge：两片各 {default} 个线程各跑 32 片", synthetic_merge_log(), True),
        ("merge：第 1 片那台 32 核、线程数没显式设成 1（来源 crash_amplification_threads_variable、配的 32），跑了 32 片却只起 1 个线程",
         synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 32),
                             shard_sources=("environment_variable", "crash_amplification_threads_variable")), False),
        ("merge：第 1 片只起 1 个线程，那一片的线程数显式设成 1",
         synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 1)), True),
        ("merge：第 1 片只起 1 个线程，那台只有 1 核", synthetic_merge_log(shard_worker_threads=(default, 1), shard_configured=(default, 1),
                                                          shard_sources=("environment_variable", "crash_amplification_threads_variable"),
                                                          shard_cores=(32, 1)), True),
        ("merge：第 1 片跑了 32 片却报起了 0 个线程", synthetic_merge_log(shard_worker_threads=(default, 0)), False),
        ("merge：shards=3 而逐片字段只有两片", synthetic_merge_log(shards=3), False),
        (f"merge：两片线程数都没显式设（来源 available_parallelism）、各取每台默认重活线程数 {default}",
         synthetic_merge_log(shard_sources=("available_parallelism", "available_parallelism")), True),
        (f"merge：第 1 片线程数没显式设（来源 available_parallelism）、按那台核数取了 {default + 1}，不等于每台默认重活线程数 {default}",
         synthetic_merge_log(shard_worker_threads=(default, default + 1), shard_configured=(default, default + 1),
                             shard_sources=("environment_variable", "available_parallelism")), False),
    ]
    for label, log_text, expected_green in cells:
        problems, _recorded, notes = judge_crash_case_log(case, log_text, SAMPLE_MACHINE_CORES, False, default)
        green = not problems
        selftest.expect(f"判用例日志：{label} ⇒ {'绿' if expected_green else '红'}",
                        green == expected_green and (not green or any("双机分片 2 片再 merge" in note for note in notes)),
                        f"判成{'绿' if green else '红'}：{problems}；线程那一句 {notes}")
    with BreakSwitch("threads-ignore-shards"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[1][1], SAMPLE_MACHINE_CORES, False, default)
    selftest.expect(f"弄坏开关 threads-ignore-shards 下「第 1 片只起 1 个线程跑了 32 片」那一格判绿（只看各片之和 {default + 1} 个线程，分不出是哪一片只起了 1 个）",
                    not problems, f"弄坏之后仍判红：{problems}")
    with BreakSwitch("threads-default-unchecked"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[-1][1], SAMPLE_MACHINE_CORES, False, default)
    selftest.expect(f"弄坏开关 threads-default-unchecked 下「merge：第 1 片来源 available_parallelism、取了 {default + 1}」那一格判绿（不比每台默认重活线程数）",
                    not problems, f"弄坏之后仍判红：{problems}")
    try:
        parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", "shard=across-machines"], 1, ""))
        outcome = "读得出"
    except RegistrationError as error:
        outcome = f"拒：{error}"
    selftest.expect("崩溃枚举用例登记行：写了 shard=across-machines ⇒ 拒（每条用例都由门禁分片，登记行不认这个键）", outcome.startswith("拒"), outcome)



def run_crash_case_features_cells(selftest):
    """登记行 features= 带进 crash-case-command：带特性的行拼出 -p <包> 之后带 --features、env 里设 bindgen 头文件目录的命令（多机配置 ENABLE_GPU=1 的另加 gpu、
    直接起 env、交调用方环境里的那张卡），不带的行在 GPU 关着时不带 --features（bindgen 照设：验证包常编 librocksdb-sys），GPU 开着时一律加 gpu；登记行写错（小写以外、写两条、有空项或重复项）拒。
    弄坏开关：features-not-passed（带特性那一格红）、features-passed-to-every-case（不带特性那一格红）、gpu-case-quiesced-by-itself 与 gpu-cards-from-configuration（开 GPU 那几格红）、
    gpu-test-threads-one（开 GPU 的命令不带测试线程参数那一格红）、verification-default-features-kept（验证包的用例带 --no-default-features 那几格红）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-features-")
    try:
        plain_row = "crash-case:plain\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY\t# 样本"
        engine_row = "crash-case:engine\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY features=sample-engine\t# 样本"
        build_crash_case_repository(work, [plain_row, engine_row])
        write_text(os.path.join(work, BINDGEN_LIBRARY_SCRIPT), 'CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="-I/sample/gcc/13/include -I/sample/gcc/14/include"\n')
        gpu_off = os.path.join(work, "multi-host-gpu-off.env")
        gpu_on = os.path.join(work, "multi-host-gpu-on.env")
        write_text(gpu_off, "ENABLE_GPU=0\n")
        write_text(gpu_on, 'ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY="127.0.0.1:0:8G"\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"\n')
        rows = read_registration_rows(work)
        cargo_head = ["cargo", "test", "--release", "-p", "pkg"]
        cargo_tail = ["--test", "own_case", "--", "--include-ignored", "--exact", "the_case", "--nocapture"]
        gpu_tail = list(cargo_tail)   # 开了 GPU 的命令收尾与 GPU 关的相同：不带测试线程参数（弄坏开关 gpu-test-threads-one 下多一个 --test-threads 1）

        service_card = "127.0.0.1:1:20G"   # 调用方环境里的那张卡；多机配置里写的显卡先后（127.0.0.1:0:8G、127.0.0.1:0:24G）都不是它

        def launched(key, configuration, card=service_card):
            caller = dict(os.environ, **{MULTI_HOST_CONFIGURATION_VARIABLE: configuration})
            caller.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
            caller.pop(SERVICE_CARD_VARIABLE, None)
            if card is not None:
                caller[SERVICE_CARD_VARIABLE] = card
            return crash_case_launch(work, crash_case_of_key(rows, key), caller)[3]

        def has_bindgen(command):
            return f"{BINDGEN_VARIABLE}=-I/sample/gcc/13/include -I/sample/gcc/14/include" in command

        plain = launched("crash-case:plain", gpu_off)
        plain_is_old = (plain[0] == "env" and plain[plain.index("cargo"):] == cargo_head + cargo_tail
                        and has_bindgen(plain) and not any(word.startswith("GPU_") for word in plain))
        selftest.expect("crash-case-command：没登记 features= 的行、多机配置没开 GPU：命令照旧，不带 --features、设 bindgen、不设显卡变量",
                        plain_is_old, f"命令是 {plain}")
        plain_on_gpu = launched("crash-case:plain", gpu_on)
        plain_on_gpu_ok = (plain_on_gpu[0] == "env" and plain_on_gpu[plain_on_gpu.index("cargo"):] == cargo_head + ["--features", "gpu"] + gpu_tail
                           and has_bindgen(plain_on_gpu) and f"{SERVICE_CARD_VARIABLE}={service_card}" in plain_on_gpu)
        selftest.expect("crash-case-command：没登记 features= 的行、多机配置开着 GPU：一律带 --features gpu（崩溃枚举用例的判段上卡）、设 bindgen、直接起 env（不另套清场包装）、交调用方环境里的那张卡",
                        plain_on_gpu_ok, f"命令是 {plain_on_gpu}")
        with BreakSwitch("features-passed-to-every-case"):
            broken_plain = launched("crash-case:plain", gpu_off)
        selftest.expect("弄坏开关 features-passed-to-every-case 下「不带特性的行命令照旧」那一格红（带上了 --features）", "--features" in broken_plain,
                        f"弄坏之后命令是 {broken_plain}")

        engine = launched("crash-case:engine", gpu_off)
        engine_ok = (engine[0] == "env" and engine[engine.index("cargo"):] == cargo_head + ["--features", "sample-engine"] + cargo_tail and has_bindgen(engine))
        selftest.expect("crash-case-command：登记了 features=sample-engine 的行（GPU 关）：-p pkg 之后带 --features sample-engine、env 里设 bindgen 头文件目录、不经包装",
                        engine_ok, f"命令是 {engine}")
        with BreakSwitch("features-not-passed"):
            broken_engine = launched("crash-case:engine", gpu_off)
        selftest.expect("弄坏开关 features-not-passed 下「带特性的行带 --features」那一格红（没带）", "--features" not in broken_engine, f"弄坏之后命令是 {broken_engine}")

        gpu = launched("crash-case:engine", gpu_on)
        gpu_ok = (gpu[0] == "env" and "--features" in gpu
                  and gpu[gpu.index("--features") + 1] == "sample-engine,gpu" and f"{SERVICE_CARD_VARIABLE}={service_card}" in gpu
                  and f"{MULTI_HOST_CONFIGURATION_VARIABLE}={gpu_on}" in gpu)
        selftest.expect("crash-case-command：登记了 features=sample-engine 的行、多机配置 ENABLE_GPU=1：特性加 gpu，直接起 env（不经清场包装），交调用方环境里的那张卡",
                        gpu_ok, f"命令是 {gpu}")
        with BreakSwitch("gpu-case-quiesced-by-itself"):
            broken_gpu = launched("crash-case:engine", gpu_on)
        selftest.expect("弄坏开关 gpu-case-quiesced-by-itself 下「开 GPU 的带特性行直接起 env」那一格红（前面接回清场包装）",
                        broken_gpu[:3] == ["bash", os.path.join(work, GPU_QUIESCE_SCRIPT), "--"], f"弄坏之后命令是 {broken_gpu}")
        with BreakSwitch("gpu-cards-from-configuration"):
            broken_cards = launched("crash-case:engine", gpu_on)
        selftest.expect("弄坏开关 gpu-cards-from-configuration 下「交调用方环境里的那张卡」那一格红（取了多机配置清场之后那一份）",
                        f"{SERVICE_CARD_VARIABLE}=127.0.0.1:0:24G" in broken_cards and f"{SERVICE_CARD_VARIABLE}={service_card}" not in broken_cards, f"弄坏之后命令是 {broken_cards}")
        gpu_without_card = launched("crash-case:engine", gpu_on, card=None)
        selftest.expect("crash-case-command：开着 GPU 而调用方环境里没有 GPU_CARDS_BY_PRIORITY：命令里不设显卡变量（不从多机配置里取卡，没卡由门禁判红）",
                        gpu_without_card[0] == "env" and not any(word.startswith(f"{SERVICE_CARD_VARIABLE}=") for word in gpu_without_card),
                        f"命令是 {gpu_without_card}")
        engine_on_gpu_tail = gpu[gpu.index("cargo"):][-len(gpu_tail):]
        selftest.expect("crash-case-command：开着 GPU（带特性行与不带特性行）与 GPU 关的命令都不带 --test-threads（测试线程照服务交下来的 RUST_TEST_THREADS）",
                        engine_on_gpu_tail == gpu_tail and "--test-threads" not in plain_on_gpu
                        and "--test-threads" not in plain and "--test-threads" not in engine, f"开 GPU 带特性行的收尾是 {engine_on_gpu_tail}，不带特性行是 {plain_on_gpu[-2:]}，GPU 关的是 {plain[-2:]}、{engine[-2:]}")
        with BreakSwitch("gpu-test-threads-one"):
            broken_threads_gpu = launched("crash-case:engine", gpu_on)
            broken_threads_off = launched("crash-case:engine", gpu_off)
        selftest.expect("弄坏开关 gpu-test-threads-one 下「开 GPU 的 cargo 参数不带测试线程」那一格红（带上 --test-threads 1），GPU 关的不带", broken_threads_gpu[-2:] == ["--test-threads", "1"] and "--test-threads" not in broken_threads_off,
                        f"弄坏之后开 GPU 的命令是 {broken_threads_gpu}，GPU 关的是 {broken_threads_off}")
        # 验证包的用例：-p <包> 之后另带 --no-default-features（不编装置二进制），在 --features 之前；别的包（上面 pkg 的几格）不带
        plain_case = crash_case_of_key(rows, "crash-case:plain")
        verification_case = CrashCase(plain_case.row, VERIFICATION_PACKAGE, plain_case.target, plain_case.function, plain_case.count_lines, plain_case.exhaustive_lines, plain_case.thread_lines)
        verification_engine_case = CrashCase(plain_case.row, VERIFICATION_PACKAGE, plain_case.target, plain_case.function, plain_case.count_lines, plain_case.exhaustive_lines,
                                             plain_case.thread_lines, features=("sample-engine",))
        verification_head = ["cargo", "test", "--release", "-p", VERIFICATION_PACKAGE, "--no-default-features"]

        def launched_case(case, configuration):
            caller = dict(os.environ, **{MULTI_HOST_CONFIGURATION_VARIABLE: configuration})
            caller.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
            caller.pop(SERVICE_CARD_VARIABLE, None)
            return crash_case_launch(work, case, caller)[3]
        verification_plain = launched_case(verification_case, gpu_off)
        verification_engine_on_gpu = launched_case(verification_engine_case, gpu_on)
        selftest.expect("crash-case-command：验证包的用例（GPU 关、不登记 features=）：-p singlefs-verification 之后带 --no-default-features，不带 --features",
                        verification_plain[verification_plain.index("cargo"):] == verification_head + cargo_tail, f"命令是 {verification_plain}")
        selftest.expect("crash-case-command：验证包的用例（登记了 features=sample-engine、GPU 开）：--no-default-features 在 --features sample-engine,gpu 之前，别的照旧",
                        verification_engine_on_gpu[verification_engine_on_gpu.index("cargo"):] == verification_head + ["--features", "sample-engine,gpu"] + gpu_tail,
                        f"命令是 {verification_engine_on_gpu}")
        with BreakSwitch("verification-default-features-kept"):
            broken_verification = launched_case(verification_case, gpu_off)
        selftest.expect("弄坏开关 verification-default-features-kept 下「验证包的用例带 --no-default-features」那一格红（没带）", "--no-default-features" not in broken_verification,
                        f"弄坏之后命令是 {broken_verification}")
        # 多机配置在而读不了：判不出开没开 GPU，抛错，不当成关
        unreadable = os.path.join(work, "multi-host-unreadable.env")
        with open(unreadable, "wb") as handle:
            handle.write(b"ENABLE_GPU=\xff\xfe\n")
        try:
            launched("crash-case:engine", unreadable)
            outcome = "没抛错"
        except InputManifestError as error:
            outcome = f"拒：{error}"
        selftest.expect("crash-case-command：多机配置在而读不了 ⇒ 抛 InputManifestError（不当成 GPU 关）", outcome.startswith("拒"), outcome)

        written = [("features= 写成大写", ["features=Sample_Engine"]), ("features= 写了两条", ["features=sample-engine", "features=gpu"]),
                   ("features= 里有空项", ["features=sample-engine,,gpu"]), ("features= 里有重复项", ["features=gpu,gpu"])]
        for label, conditions in written:
            try:
                parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", *conditions], 1, ""))
                outcome = "读得出"
            except RegistrationError as error:
                outcome = f"拒：{error}"
            selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ 拒", outcome.startswith("拒") and "features=" in outcome, outcome)
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_real_crash_case_features_cells(selftest, rows, repository_root):
    """真仓登记表里崩溃枚举用例登记的 features= 每一项都是那个包 Cargo.toml [features] 里声明着的（包里删掉的特性留在登记表里，
    crash-case-command 拼出的 cargo 起不来）。"""
    packages = workspace_package_directories(repository_root)
    undeclared = []
    cases = crash_cases_of(rows)
    for case in cases:
        if not case.features:
            continue
        directory = packages.get(case.package)
        if directory is None:
            undeclared.append(f"{case.key}：包 {case.package} 不是工作区成员")
            continue
        declared = (manifest_sections(os.path.join(repository_root, directory, "Cargo.toml")) or {}).get("features") or {}
        missing = [feature for feature in case.features if feature not in declared]
        if missing:
            undeclared.append(f"{case.key}：{'、'.join(missing)} 不在 {directory}/Cargo.toml 的 [features] 里")
    selftest.expect("真仓登记表里崩溃枚举用例登记的 features= 都是那个包 Cargo.toml [features] 里声明着的特性", not undeclared, "；".join(undeclared))


def heavy_threads_library():
    """scripts/hosts/heavy_threads.py（与 HEAVY_THREAD_DEFAULT_FILE 同一棵树的那一份）按路径载入；载不进交 (None, 原因)。只给自证对拍用。"""
    import importlib.util
    path = os.path.join(os.path.dirname(os.path.dirname(HEAVY_THREAD_DEFAULT_FILE)), "scripts", "hosts", "heavy_threads.py")
    try:
        specification = importlib.util.spec_from_file_location("heavy_threads_for_admission_selftest", path)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
    except (OSError, ImportError, SyntaxError, AttributeError) as error:
        return None, f"{path} 载不进：{error}"
    return library, ""


def run_heavy_thread_default_cells(selftest, case, environment, cores, work):
    """没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 时线程数取每台默认重活线程数：SINGLEFS_THREAD_CAP=3 ⇒ 3、不是正整数拒，都没设 ⇒ 登记值（heavy_threads.py 读同一份交的数）；
    弄坏开关 threads-default-from-cores 下「都没设」那一格红（交本机核数）。registered_heavy_threads 与 heavy_threads.py 的 default_heavy_threads 拿同几份登记逐格对拍。"""
    without_cap = {name: value for name, value in environment.items() if name not in (THREAD_CAP_VARIABLE, "HEAVY_THREADS_BREAK")}
    library, why = heavy_threads_library()
    selftest.expect("每台默认重活线程数：scripts/hosts/heavy_threads.py 载得进（对拍要它）", library is not None, why)
    got = crash_case_worker_threads(case, dict(without_cap, **{THREAD_CAP_VARIABLE: "3"}))
    selftest.expect("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=3 ⇒ 线程数 3、default", got == (cores, 3, "default"), f"交的是 {got}")
    try:
        got = crash_case_worker_threads(case, dict(without_cap, **{THREAD_CAP_VARIABLE: "abc"}))
    except InputManifestError as error:
        got = f"拒：{error}"
    selftest.expect("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、SINGLEFS_THREAD_CAP=abc ⇒ 拒（抛 InputManifestError）", str(got).startswith("拒") and THREAD_CAP_VARIABLE in str(got),
                    f"交的是 {got}")
    registered = library.default_heavy_threads(without_cap) if library is not None else None
    got = crash_case_worker_threads(case, without_cap)
    selftest.expect(f"没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 与 SINGLEFS_THREAD_CAP ⇒ 线程数是登记值（heavy_threads.py 读同一份交 {registered}）、default",
                    got == (cores, registered, "default"), f"交的是 {got}")
    with BreakSwitch("threads-default-from-cores"):
        broken_got = crash_case_worker_threads(case, without_cap)
    selftest.expect("弄坏开关 threads-default-from-cores 下「都没设 ⇒ 登记值」那一格红（交本机核数）", broken_got[1] != registered or cores == registered,
                    f"弄坏之后交的是 {broken_got}，登记值 {registered}")
    if library is None:
        return
    variants = [("登记 7、带理由", "7  # 样本理由\n", None), ("登记前有注释行与空行", "# 注释\n\n7  # 样本理由\n8  # 第二条不认\n", None),
                ("登记文件不在", None, None), ("写坏：不是数", "abc  # 样本理由\n", None), ("写坏：0", "0  # 样本理由\n", None),
                ("写坏：没写理由", "7\n", None), ("写坏：数与 # 之间没有空白", "7# 样本理由\n", None), ("写坏：只有注释", "# 只有注释\n", None),
                ("SINGLEFS_THREAD_CAP=5 盖过登记", "7  # 样本理由\n", "5"), ("SINGLEFS_THREAD_CAP=0", "7  # 样本理由\n", "0"),
                ("SINGLEFS_THREAD_CAP 设成空串当没设", "7  # 样本理由\n", "")]
    for index, (label, registration, cap) in enumerate(variants):
        path = os.path.join(work, f"heavy-thread-default-{index}")
        if registration is not None:
            write_text(path, registration)
        variant_environment = dict(without_cap) if cap is None else dict(without_cap, **{THREAD_CAP_VARIABLE: cap})
        try:
            mine = registered_heavy_threads(variant_environment, registration_file=path)
        except InputManifestError:
            mine = "refused"
        try:
            theirs = library.default_heavy_threads(variant_environment, registration_file=path)
        except library.HeavyThreadDefaultError:
            theirs = "refused"
        selftest.expect(f"每台默认重活线程数：与 heavy_threads.py 对拍，{label} ⇒ 两边都交「{theirs}」", mine == theirs, f"准入模块交「{mine}」，heavy_threads.py 交「{theirs}」")


def run_thread_source_cells(selftest, module):
    """线程从哪来、判法自己现取。
    FM：登记行的 threads-variable= 由 crash-case-command 设成配的线程数（调用方设着的不认，弄坏开关 thread-variable-ignored 下漏进来）、
    标记里 configured_worker_threads= 记这个变量；本机核数不认 OMP_NUM_THREADS 与 PATH 里的 nproc（弄坏开关 cores-from-nproc 下认）。
    FD3：crash-case-judge 自己现取线程数：显式设 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 的 64 片 1 线程判绿、没设的判红；转过来的 --threads 1 --threads-origin explicit
    是用法错（退 2），翻不了判定（弄坏开关 judge-takes-forwarded-threads 下照旧收、判绿）。
    每台默认重活线程数：没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS 时线程数取它（SINGLEFS_THREAD_CAP 设了照它、不是正整数拒；弄坏开关 threads-default-from-cores 下取本机核数），
    registered_heavy_threads 与 scripts/hosts/heavy_threads.py 的 default_heavy_threads 拿同几份登记逐格对拍。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-threads-")
    try:
        row = ("crash-case:own\t-\ttest=pkg:own_case:the_case count-line=CRASH_POINT_REPLAY count-line=CHECKER exhaustive=CRASH_POINT_REPLAY threads=CRASH_POINT_REPLAY "
               "threads-variable=SINGLEFS_SAMPLE_CASE_THREADS\t# 样本")
        build_crash_case_repository(work, [row])
        case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
        cores = machine_core_count(os.environ)
        caller = crash_case_launch_environment(SINGLEFS_SAMPLE_CASE_THREADS="1")
        caller.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
        try:
            default = registered_heavy_threads(caller)
        except InputManifestError as error:
            selftest.expect("每台默认重活线程数：调用方这份环境里取得出", False, str(error))
            default = 1

        def seen_by_the_case(case_now):
            command = crash_case_launch(work, case_now, caller)[3]
            launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s %s" "${SINGLEFS_SAMPLE_CASE_THREADS-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_THREADS-unset}"']
            return subprocess.run(launched, env=caller, capture_output=True, text=True, check=False).stdout
        seen = seen_by_the_case(case)
        selftest.expect(f"crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1 ⇒ 起的用例看到配的 {default}（每台默认重活线程数，盖掉调用方的）",
                        seen == f"{default} {default}", f"用例看到「{seen}」")
        with BreakSwitch("thread-variable-ignored"):
            broken_case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
            broken_seen = seen_by_the_case(broken_case)
            broken_text = configured_worker_threads_text(broken_case, cores, default, "default")
        selftest.expect("弄坏开关 thread-variable-ignored 下「调用方设着用例读的线程变量」那一格红（用例看到 1）", broken_seen.startswith("1 "),
                        f"弄坏之后用例看到「{broken_seen}」：这一格分不出命令设没设它")
        text = configured_worker_threads_text(case, cores, default, "default")
        selftest.expect("标记里 configured_worker_threads= 记的是这条用例读的那个变量（SINGLEFS_SAMPLE_CASE_THREADS=…），不是 SINGLEFS_CRASH_POINT_REPLAY_THREADS",
                        text.startswith(f"SINGLEFS_SAMPLE_CASE_THREADS={default}（没设，取每台默认重活线程数）"), f"记成「{text}」")
        selftest.expect("弄坏开关 thread-variable-ignored 下「标记记的变量」那一格红（记成 SINGLEFS_CRASH_POINT_REPLAY_THREADS）", broken_text.startswith("SINGLEFS_CRASH_POINT_REPLAY_THREADS="),
                        f"弄坏之后记成「{broken_text}」")
        registration_cells = [("threads-variable= 写成小写", "threads-variable=sample_threads"), ("threads-variable= 写了两条",
                              "threads-variable=A_THREADS threads-variable=B_THREADS")]
        for label, conditions in registration_cells:
            try:
                parse_crash_case(RegistrationRow("crash-case:demo", [], ["test=pkg:target:function", *conditions.split()], 1, ""))
                outcome = "读得出"
            except RegistrationError as error:
                outcome = f"拒：{error}"
            selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ 拒", outcome.startswith("拒") and "threads-variable=" in outcome, outcome)
        # 本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 都不认
        tools = os.path.join(work, ".tools")
        write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 1\n")
        leaky = dict(os.environ, PATH=tools + os.pathsep + os.environ.get("PATH", ""), OMP_NUM_THREADS="1")
        leaky.pop(CRASH_POINT_REPLAY_THREADS_VARIABLE, None)
        got = crash_case_worker_threads(case, leaky)
        selftest.expect(f"本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 ⇒ 仍是 os.cpu_count() 与 CPU 亲和的小者（{cores}），线程数取每台默认重活线程数（{default}）、default",
                        got == (cores, default, "default"), f"交的是 {got}")
        run_heavy_thread_default_cells(selftest, case, leaky, cores, work)
        with BreakSwitch("cores-from-nproc"):
            broken_got = crash_case_worker_threads(case, leaky)
        selftest.expect("弄坏开关 cores-from-nproc 下「nproc 打 1」那一格红（交的核数是 1）", broken_got[0] == 1, f"弄坏之后交的是 {broken_got}")
        # FD3：crash-case-judge 自己现取线程数
        one_thread_log = os.path.join(work, "one-thread.log")
        write_text(one_thread_log, synthetic_crash_point_replay_log(worker_threads=1))
        judged = os.path.join(work, "judged.out")
        plain_environment = {name: value for name, value in os.environ.items() if name != CRASH_POINT_REPLAY_THREADS_VARIABLE}

        def judge(extra_arguments=(), changes=None):
            return run_in_environment([sys.executable, module, "crash-case-judge", work, "crash-case:own", one_thread_log, judged, *extra_arguments],
                                      dict(plain_environment, **(changes or {})))[0]
        forwarded = ("--machine-cores", str(cores), "--threads", "1", "--threads-origin", "explicit")
        cells = [("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS：64 片 1 个线程", (), None, 1 if cores > 1 else 0),
                 ("调用方显式设 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1：64 片 1 个线程", (), {CRASH_POINT_REPLAY_THREADS_VARIABLE: "1"}, 0),
                 ("没设 SINGLEFS_CRASH_POINT_REPLAY_THREADS、转过来 --threads 1 --threads-origin explicit（crash-point-replay 改前的转法）", forwarded, None, EXIT_REGISTRATION_ERROR)]
        for label, extra_arguments, changes, expected in cells:
            got_exit = judge(extra_arguments, changes)
            selftest.expect(f"crash-case-judge 自己现取线程数：{label} ⇒ 退 {expected}", got_exit == expected, f"退 {got_exit}")
        broken_exit = judge(forwarded, {BREAK_VARIABLE: "judge-takes-forwarded-threads"})
        selftest.expect("弄坏开关 judge-takes-forwarded-threads 下「转过来 --threads 1 explicit」那一格红（照旧收、判绿退 0）", broken_exit == 0,
                        f"弄坏之后退 {broken_exit}")
    finally:
        shutil.rmtree(work, ignore_errors=True)



C561_SIGMA_STATES = 262144


def c561_sigma_log(exhaustive="true", worker_threads=None, configured_worker_threads=None):
    """合成的 crash-case:c561-sigma-full 一趟用例日志：用例打的计数行与同形的线程行（照 crates/singlefs-verification/tests/
    record_checker_judges_absence_by_the_persisted_set.rs 的 count_line 与线程行；来源 available_parallelism，即线程数没显式设）。
    起的与配的线程数不给都取每台默认重活线程数。"""
    default_threads = selftest_default_threads() or 1
    worker_threads = default_threads if worker_threads is None else worker_threads
    configured_worker_threads = default_threads if configured_worker_threads is None else configured_worker_threads
    return (f"CRASH_POINT_REPLAY_PARALLEL_FINISHED states={C561_SIGMA_STATES} slices=64 worker_threads={worker_threads} configured_worker_threads={configured_worker_threads} "
            f"worker_threads_source=available_parallelism resumed_slices=0 freshly_run_slices=64 progress_file_after_completion=none elapsed_seconds=1.0\n"
            f"C561_SIGMA_FULL states={C561_SIGMA_STATES} closed_form_states={C561_SIGMA_STATES} exhaustive={exhaustive} "
            f"record_claimed_state_missing_unit=0\n{PASSED_ONE_LINE}\n")


def run_real_crash_case_row_cells(selftest, rows):
    """真仓登记表里两条崩溃枚举用例的第三列拿合成日志判：c561-sigma-full 的计数行不带 exhaustive=true、1 个线程跑了 64 片都判红，两样都对判绿
    （只登记 count-line= 时这两种都判绿）；crash-injection-default-batch 登记着，计数行 CRASH_INJECTION_FINISHED 恰好一行判绿、两行判红，
    跑完那一行报 1 个线程跑了 24 片判红（它登记了 threads=CRASH_INJECTION_FINISHED；弄坏开关 threads-skip-self-contained-finish-line 下这一格转绿）；
    它登记的 threads-variable= 是 crates/singlefs-verification/src/crash_injection.rs 与 crash_injection/ 下那几份里的 CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE。"""
    default_batch_line = "CRASH_INJECTION_FINISHED seeds=[1,25) slices=24 worker_threads=4 elapsed_seconds=1.0\n"
    cells = [
        ("crash-case:c561-sigma-full", "两行都对", c561_sigma_log(), True),
        ("crash-case:c561-sigma-full", "计数行 exhaustive=false（状态数不等于闭式）", c561_sigma_log(exhaustive="false"), False),
        ("crash-case:c561-sigma-full", "本机 32 核、线程数没显式设，1 个线程跑了 64 片", c561_sigma_log(worker_threads=1), False),
        ("crash-case:c561-sigma-full", "线程数没显式设，用例按本机核数取了 32 个（不是每台默认重活线程数）",
         c561_sigma_log(worker_threads=32, configured_worker_threads=32), (selftest_default_threads() or 1) == 32),
        ("crash-case:crash-injection-default-batch", "计数行恰好一行", default_batch_line + PASSED_ONE_LINE + "\n", True),
        ("crash-case:crash-injection-default-batch", "计数行打了两行", default_batch_line * 2 + PASSED_ONE_LINE + "\n", False),
        ("crash-case:crash-injection-default-batch", "本机 32 核、线程数没显式设，跑完那一行报 1 个线程跑了 24 片",
         default_batch_line.replace("worker_threads=4", "worker_threads=1") + PASSED_ONE_LINE + "\n", False),
    ]
    for key, label, log_text, expected_green in cells:
        try:
            problems = judge_crash_case_log(crash_case_of_key(rows, key), log_text, SAMPLE_MACHINE_CORES, False, selftest_default_threads() or 1)[0]
        except RegistrationError as error:
            problems = [f"真仓登记表里读不出这一条：{error}"]
        green = not problems
        selftest.expect(f"真仓登记的 {key}：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green,
                        f"判成{'绿' if green else '红'}：{problems}")
    # 判法的放宽只给登记了标记的用例（门禁 crash-point-replay 文件头「复用」）：真仓这几条登记了，没点名的这一条一样都没带
    registered_marks = [("findings_not_written_by_the_case", key) for key in (
                            "crash-case:unlink-after-the-multi-record-publish-unlink-batches-on-the-cpu-judge",
                            "crash-case:unlink-after-the-multi-record-publish-unlink-final-batch-on-the-cpu-judge",
                            "crash-case:findings-log-red-states-deduplicated-by-signature",
                            "crash-case:findings-log-byte-identical-when-rerun-from-the-library",
                            "crash-case:findings-log-observer-version-recorded-on-the-reused-recording",
                            "crash-case:record-spill-over-share-with-every-judge-conclusion-reused")]
    registered_marks += [("blocks_line_not_printed_by_the_case", "crash-case:record-spill-over-observer-red-on-the-other-hosts-blocks"),
                         ("red_blocks_judged_by_test", "crash-case:make-filesystem-over-a-used-pool-stream"),
                         ("red_blocks_judged_by_test", "crash-case:new-pool-file-creation-removing-the-barrier-before-the-root-slot")]
    for attribute, key in registered_marks:
        try:
            marked = vars(crash_case_of_key(rows, key))[attribute]
        except RegistrationError as error:
            marked = f"真仓登记表里读不出这一条：{error}"
        selftest.expect(f"真仓登记的 {key}：带 {attribute} 那个标记", marked is True, f"读到 {marked}")
    try:
        plain_case = crash_case_of_key(rows, "crash-case:new-pool-file-creation-stream")
        plain_marks = [plain_case.red_blocks_judged_by_test, plain_case.findings_not_written_by_the_case, plain_case.blocks_line_not_printed_by_the_case]
    except RegistrationError as error:
        plain_marks = [f"真仓登记表里读不出这一条：{error}"]
    selftest.expect("真仓登记的 crash-case:new-pool-file-creation-stream：三个标记都没带", plain_marks == [False, False, False], f"读到 {plain_marks}")
    one_thread_default_batch = cells[-1][2]
    with BreakSwitch("threads-skip-self-contained-finish-line"):
        problems = judge_crash_case_log(crash_case_of_key(rows, "crash-case:crash-injection-default-batch"), one_thread_default_batch, SAMPLE_MACHINE_CORES, False,
                                        selftest_default_threads() or 1)[0]
    selftest.expect("弄坏开关 threads-skip-self-contained-finish-line 下「crash-injection 默认批 1 个线程跑了 24 片」那一格红（判绿）", not problems,
                    f"弄坏之后仍判红：{problems}")
    source_directory = os.path.join(os.path.dirname(os.path.dirname(SELFTEST_HERE)), "crates/singlefs-verification/src")
    source_paths = [os.path.join(source_directory, "crash_injection.rs")]
    for directory, _directories, file_names in os.walk(os.path.join(source_directory, "crash_injection")):
        source_paths += [os.path.join(directory, name) for name in file_names if name.endswith(".rs")]
    read_variables = []
    try:
        for source_path in sorted(source_paths):
            with open(source_path, encoding="utf-8") as handle:
                read_variables += re.findall(r'CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =\s*"([A-Z0-9_]+)"', handle.read())
        registered = crash_case_of_key(rows, "crash-case:crash-injection-default-batch").thread_variable
    except (OSError, RegistrationError) as error:
        read_variables, registered = [], f"读不出：{error}"
    selftest.expect("真仓登记的 crash-case:crash-injection-default-batch 的 threads-variable= 就是用例读的那个变量", read_variables == [registered],
                    f"登记的 {registered}，{source_directory} 里 crash_injection.rs 与 crash_injection/ 下读的 {read_variables}")


def crash_case_launch_environment(**changes):
    """照 crash_case_launch 交的命令起一趟、看用例环境的那几格的调用方环境：os.environ 套上 changes，去掉多机配置的位置（SINGLEFS_MULTI_HOST_CONFIG），
    多机配置就取样本仓里的（没有，GPU 关）。不去掉，起自证那台的多机配置写着 ENABLE_GPU=1 时命令上卡，这几格判的随机器变：
    弄坏开关 gpu-case-quiesced-by-itself 下前面接回的清场包装在样本仓里不在，它们跟着红。"""
    environment = dict(os.environ, **changes)
    environment.pop(MULTI_HOST_CONFIGURATION_VARIABLE, None)
    return environment


def stage_variables_seen_by_the_case(work, stage=None):
    """照 crash_case_launch 交的命令起一趟（cargo 起往后换成打两个变量的 sh），调用方环境里设着 SINGLEFS_CRASH_AMPLIFICATION_STAGE=record 与
    SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=1,3；stage 照 crash-case-command --stage 交：交回那一趟看到的两个值（没设是 unset），空格隔开。"""
    case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
    environment = crash_case_launch_environment(SINGLEFS_CRASH_POINT_REPLAY_THREADS="2", **{CRASH_STAGE_VARIABLE: "record", CRASH_HOST_WEIGHTS_VARIABLE: "1,3"})
    command = crash_case_launch(work, case, environment, stage=stage)[3]
    launched = command[:command.index("cargo")] + ["sh", "-c", f'printf "%s %s" "${{{CRASH_STAGE_VARIABLE}-unset}}" "${{{CRASH_HOST_WEIGHTS_VARIABLE}-unset}}"']
    return subprocess.run(launched, env=environment, capture_output=True, text=True, check=False).stdout


def run_crash_case_command_stage_variable_cells(selftest, work):
    """crash-case-command 交的命令起用例时清掉调用方环境里给定状态引擎的阶段与各台权重；弄坏开关 keep-caller-stage-variables 下那一格转红。"""
    seen = stage_variables_seen_by_the_case(work)
    selftest.expect("crash-case-command：调用方环境里设着给定状态引擎的阶段 record 与各台权重 1,3 ⇒ 起的用例两个都看不到（不然只录不判、块账 0 块）",
                    seen == "unset unset", f"用例看到的是「{seen}」")
    with BreakSwitch("keep-caller-stage-variables"):
        seen = stage_variables_seen_by_the_case(work)
    selftest.expect("弄坏开关 keep-caller-stage-variables 下「调用方设着阶段与权重」那一格红（用例看到 record 1,3）", seen == "record 1,3",
                    f"弄坏之后用例看到的是「{seen}」：这一格分不出命令清没清它们")
    # --stage（录入段与判段分两次起的调用方用，admission.py 文件头 crash-case-command 那一条）：清掉调用方那一份之后照给的值设；值写错退 2、不交命令
    for stage in CRASH_CASE_STAGES:
        seen = stage_variables_seen_by_the_case(work, stage)
        selftest.expect(f"crash-case-command --stage {stage}：调用方设着阶段 record、各台权重 1,3 ⇒ 起的用例看到阶段 {stage}、权重 unset", seen == f"{stage} unset",
                        f"用例看到的是「{seen}」")
        with BreakSwitch("case-stage-not-set"):
            seen = stage_variables_seen_by_the_case(work, stage)
        selftest.expect(f"弄坏开关 case-stage-not-set 下「crash-case-command --stage {stage}」那一格红（用例看到 unset unset）", seen == "unset unset",
                        f"弄坏之后用例看到的是「{seen}」：这一格分不出命令设没设阶段")
    exit_code, output, _messages = run_quietly([sys.executable, ENTRY_SCRIPT, "crash-case-command", work, "crash-case:own", "--stage", "both"])
    selftest.expect("crash-case-command --stage both：值不是 record、judge ⇒ 退 2、不交命令", exit_code == EXIT_REGISTRATION_ERROR and "cargo" not in output.split("\0"),
                    f"退 {exit_code}，输出 {output.strip()[:200]}")
    exit_code, output, _messages = run_quietly([sys.executable, ENTRY_SCRIPT, "crash-case-command", work, "crash-case:own", "--stage", "record", "--stage", "judge"])
    selftest.expect("crash-case-command 给了两次 --stage ⇒ 退 2、不交命令", exit_code == EXIT_REGISTRATION_ERROR and "cargo" not in output.split("\0"),
                    f"退 {exit_code}，输出 {output.strip()[:200]}")


REPLAY_SELFTEST_DRIVERS = r'''
# ── admission.py --selftest 拼进来的桩驱动（真仓的 replay.sh 没有这一段）──
driver_e900() {
  : >"$OUT_DIR/driver-ran-E900"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e901() {
  : >"$OUT_DIR/driver-ran-E901"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e902() {
  : >"$OUT_DIR/driver-ran-E902"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e903() {
  : >"$OUT_DIR/driver-ran-E903"
  python3 scripts/gate/admission.py experiment . E903 || return $?
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e903_second() {
  : >"$OUT_DIR/driver-ran-E903-second"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
'''
REPLAY_TABLE_START = "TABLE=$(cat <<'TSV'\n"
REPLAY_TABLE_END = "\nTSV\n)\n"
# 临时仓里跑 replay.sh 的副本要的计时行负载字段滞后表（replay.sh 开跑前读它，不在整份退 2）：自证的复跑表一行 timing 都没有，表只放一行表头
REPLAY_TIMING_LAG_TABLE = ".claude/gate.d/timing-load-field-lag.tsv"
REPLAY_TIMING_LAG_TABLE_HEAD = "# 自证：计时行产物的负载字段滞后表，一行都不登记\n"


def replay_copy_text(table_rows, break_header_strip, row_keys=()):
    """真仓 replay.sh 的副本：只换掉复跑表与按行的准入键那一段（row_keys，一行一条、制表符分隔；不给就是空的，真仓 E142 那几条不带进来）、
    在表后面拼上桩驱动；break_header_strip 时把比对前删产物头那一步换成 cat。
    replay.sh 的形状变了（表的起止、删头那一句、按行的准入键那一段不是恰好一处）抛 RegistrationError，那一格判错。"""
    with open(os.path.join(SELFTEST_HERE, "..", "experiment-apparatus", "replay.sh"), encoding="utf-8") as handle:
        text = handle.read()
    if (text.count(REPLAY_TABLE_START) != 1 or text.count(REPLAY_STRIP_HEADER_COMMAND) != 1
            or text.count(REPLAY_ROW_KEYS_START) != 1):
        raise RegistrationError("replay.sh 里复跑表的开头、比对前删产物头那一句或按行的准入键那一段不是恰好一处，自证拼不出副本")
    start = text.index(REPLAY_TABLE_START) + len(REPLAY_TABLE_START)
    end = text.index(REPLAY_TABLE_END, start)
    copy = (text[:start] + "\n".join(table_rows) + REPLAY_TABLE_END + REPLAY_SELFTEST_DRIVERS
            + text[end + len(REPLAY_TABLE_END):])
    keys_start = copy.index(REPLAY_ROW_KEYS_START) + len(REPLAY_ROW_KEYS_START)
    keys_end = copy.index("\n" + REPLAY_ROW_KEYS_END_LINE + "\n", keys_start - 1) + 1
    copy = copy[:keys_start] + "".join(line + "\n" for line in row_keys) + copy[keys_end:]
    if break_header_strip:
        copy = copy.replace(REPLAY_STRIP_HEADER_COMMAND, "cat")
    return copy



REPLAY_LIBRARY_PREFIX = "replay-"   # replay.sh 载入时按 $REPLAY_LIBRARY_DIRECTORY/replay-<名>.sh source 的几份库，与它同目录


def write_replay_copy(work, text):
    """replay.sh 的副本写进样本仓的 REPLAY_SCRIPT，同目录的 replay-*.sh（它 source 的各份库）一并拷过去：只写入口时副本载入那一刻 source 不到库，驱动与判法的函数一个都没有。"""
    write_text(os.path.join(work, REPLAY_SCRIPT), text)
    real_directory = os.path.join(SELFTEST_HERE, "..", "experiment-apparatus")
    for file_name in sorted(os.listdir(real_directory)):
        if file_name.startswith(REPLAY_LIBRARY_PREFIX) and file_name.endswith(".sh"):
            shutil.copy(os.path.join(real_directory, file_name), os.path.join(work, os.path.dirname(REPLAY_SCRIPT), file_name))


def run_replay_cells(selftest, module):
    """replay.sh 接准入的那几处，在临时仓里拿真仓 replay.sh 的副本（只换复跑表、加桩驱动）跑：cargo 与 rustc 用桩。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-replay-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        write_executable(os.path.join(tools, "cargo"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "cargo 0.0.0-selftest"\nexit 0\n')
        write_executable(os.path.join(tools, "rustc"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "rustc 0.0.0-selftest"\nexit 0\n')
        os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "scripts/gate/admission.py"))
        copy_admission_modules(work)
        copy_function_units_library(work)
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "E900\tresearch/src/e900.sh\t# 样本\n"
                   "E901\tresearch/src/e901.sh\tquestion-row=research/questions.md#6:够判\t# 样本：前提没齐\n"
                   "E902\tresearch/src/e902.sh\t# 样本：装置没打指纹行\n")
        stored_body = "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n"
        for number in ("900", "901", "902"):
            write_text(os.path.join(work, f"research/src/e{number}.sh"), f"#!/usr/bin/env bash\necho 输入 {number}\n")
            write_text(os.path.join(work, RESULTS_DIRECTORY, f"e{number}-2026-09-01.out"), stored_body)
        write_questions(work, "开着")
        write_text(os.path.join(work, REPLAY_TIMING_LAG_TABLE), REPLAY_TIMING_LAG_TABLE_HEAD)
        link_sop_copy(work)
        link_replay_build_log_tools(work)
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
        table_rows = [f"E{number}|@driver_e{number}||e{number}-2026-09-01.out|exact" for number in ("900", "901", "902")]
        # E900、E901 的产物头由 replay.sh 接（驱动不问准入：驱动闭包里点了 admission.py，它的闭包收不全，实验键就拒算了）；E902 照装置自己打、却没打
        writer_rows = ["E900\tE900|@driver_e900|\treplay", "E901\tE901|@driver_e901|\treplay"]

        def replay(experiment, rows, break_header_strip=False, keep_multi_host_configuration=False):
            write_replay_copy(work, replay_copy_text(rows, break_header_strip, writer_rows))
            output_directory = tempfile.mkdtemp(prefix="out-", dir=work)
            exit_code, output, messages = run_quietly(
                ["bash", os.path.join(work, REPLAY_SCRIPT), experiment],
                {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "REPLAY_OUT": output_directory, "REPLAY_JOBS": "1"},
                keep_multi_host_configuration=keep_multi_host_configuration)
            return exit_code, output + messages, output_directory

        exit_code, output, first_run = replay("E900", table_rows)
        selftest.expect("replay.sh：老产物没有产物头、新跑的有，比对前删掉产物头之后判字节一致",
                        exit_code == 0 and re.search(r"^E900 .*字节一致", output, re.M) is not None
                        and os.path.exists(os.path.join(first_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        # 起自证的那一台环境里设着多机配置变量（task-server 派任务时设成那一台树里的 multi-host.env；这里指到临时目录里一份在的配置）：
        # 样本仓里没有多机配置的共用库，replay.sh 只在变量没设时不读它，所以起样本仓的 replay.sh 时清掉这个变量（run_quietly），照样跑、判字节一致
        # （与上一格同一份输入，这一行可能按行复用，复用来的那一行同样写「字节一致」）。弄坏开关 multi-host-config-inherited 下这一格不清，replay.sh 一条都没跑
        configured_by_caller = os.path.join(tools, MULTI_HOST_CONFIGURATION_NAME)
        write_text(configured_by_caller, "# 自证：起自证那一台的多机配置（样本仓的 replay.sh 不该读到它）\nENABLE_GPU=0\n")
        saved_configuration = os.environ.get(MULTI_HOST_CONFIGURATION_VARIABLE)
        os.environ[MULTI_HOST_CONFIGURATION_VARIABLE] = configured_by_caller
        try:
            exit_code, output, _run = replay("E900", table_rows, keep_multi_host_configuration=break_is_set("multi-host-config-inherited"))
        finally:
            if saved_configuration is None:
                os.environ.pop(MULTI_HOST_CONFIGURATION_VARIABLE, None)
            else:
                os.environ[MULTI_HOST_CONFIGURATION_VARIABLE] = saved_configuration
        selftest.expect("replay.sh：起自证的环境里设着多机配置变量（指到一份在的配置）时，样本仓的 replay.sh 照样跑、判字节一致",
                        exit_code == 0 and re.search(r"^E900 .*字节一致", output, re.M) is not None, f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E900", table_rows, break_header_strip=True)
        selftest.expect("弄坏 replay.sh 比对前删产物头那一步（换成 cat）判对不上：上一格分得出差别",
                        exit_code == 1 and re.search(r"^E900 .*对不上", output, re.M) is not None, f"退 {exit_code}，输出：{output.strip()[-600:]}")
        unchanged_cell = "replay.sh：留存产物头上的实验键与今天的相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行（只改复跑表里的产物文件名不动实验键）"
        if not os.path.isfile(os.path.join(first_run, "E900.1.out")):
            selftest.expect(unchanged_cell, False, f"头一趟没出 E900.1.out（{first_run}），这一格接不上；先看头一格为什么没跑")
        else:
            shutil.copy(os.path.join(first_run, "E900.1.out"), os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"))
            rows_with_fingerprinted_product = [row.replace("e900-2026-09-01.out", "e900-2026-09-02.out") for row in table_rows]
            exit_code, output, unchanged_run = replay("E900", rows_with_fingerprinted_product)
            selftest.expect(unchanged_cell,
                            exit_code == 0 and re.search(r"^E900 .*输入没变", output, re.M) is not None and "输入没变没跑 1" in output
                            and not os.path.exists(os.path.join(unchanged_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, refused_run = replay("E901", table_rows)
        selftest.expect("replay.sh：前提没齐时判跑不了（准入退 3）、装置一次都没跑",
                        exit_code == 1 and re.search(r"^E901 .*跑不了 .*准入没放行（退 3）", output, re.M) is not None
                        and not os.path.exists(os.path.join(refused_run, "driver-ran-E901")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E902", table_rows)
        selftest.expect("replay.sh：登记了准入、装置却没打实验键行时判跑不了",
                        exit_code == 1 and "产物头却没有 key=E902 的实验键行" in output, f"退 {exit_code}，输出：{output.strip()[-600:]}")
    except RegistrationError as error:
        selftest.expect("replay.sh 的副本拼得出来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)



REPLAY_JUDGE_FUNCTION_START = "replay_judge_fresh_product() {"
# 弄坏开关 replay-store-inside-judge 往拷来的判法开头接回的那一句（判完当场经 admission.py library-files store 存库）
REPLAY_STORE_INSIDE_JUDGE_UNDER_BREAK = '  python3 "$ADMISSION_MODULE" library-files store "$REPLAY_ROW_LIBRARY" "$REPLAY_ROW_RECORDS" >/dev/null 2>&1\n'
REPLAY_JUDGING_CLOSURE_CELL = "replay.sh 判法闭包：REPLAY_JUDGING_FUNCTIONS 每个都算得出键、没有拒算、闭包里没有 admission.py 的单元（存库不在判法里）"


REPLAY_PACKAGE_MENTION_CELL = "复跑表的行键：驱动里 mkdir -p -- 的 -- 不当包名，-p <包> 照认"
REPLAY_BIN_WRAPPER_CELL = "复跑表的行键：--bin \"$变量\" 写在包装函数里：认出每一处调用点名的 bin"
REPLAY_DEVICE_KEY_CELL = "复跑表的行键：device-code-key.py run --package-directory <目录> --binary <名>：认出包目录与 bin"


def run_replay_package_mention_cell(selftest):
    text = 'mkdir -p -- "$out"\ncargo run -q --release -p singlefs-verification --bin e1\nmkdir -p "$out"\n'
    found = replay_packages_mentioned(text)
    selftest.expect(REPLAY_PACKAGE_MENTION_CELL, found == ["singlefs-verification"], f"认出的包 {found}")
    wrapper = ('sample_run() { # <bin> <参数…>\n  local bin="$1"; shift\n  cargo run -q --release -p singlefs-verification --bin "$bin" -- "$@"\n}\n'
               'driver_sample() {\n  sample_run sample_device dump\n  sample_run other_device\n}\ncargo run -q --bin literal_device\n')
    named = sorted(replay_binaries_mentioned(wrapper))
    selftest.expect(REPLAY_BIN_WRAPPER_CELL, named == ["literal_device", "other_device", "sample_device"], f"认出的 bin {named}")
    device = ('python3 scripts/experiment-apparatus/device-code-key.py run . --package-directory crates/singlefs-verification --binary sample_device \\\n'
              '    --entry singlefs_verification::verdict -- cells\n')
    keyed = replay_device_key_binaries(device)
    selftest.expect(REPLAY_DEVICE_KEY_CELL, keyed == [("crates/singlefs-verification", "sample_device")], f"认出的 {keyed}")


def run_replay_judging_closure_cell(selftest, module):
    """replay.sh 的判法（REPLAY_JUDGING_FUNCTIONS，文件头「复跑表的行键」判法那一行）闭包里没有拒算、没有 admission.py 的单元：存库（起 admission.py library-files store）
    在 replay_one 里做、不进判法闭包（进了就连上 admission.py 的程序入口，每一行的实验键与行键都拒算）。临时仓里拷真仓 replay.sh、admission.py 与
    lib_function_units.py，逐个算判法键。弄坏开关 replay-store-inside-judge 下往拷来的 replay_judge_fresh_product 开头接回那一句，这一格必须红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-replay-judge-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        with open(os.path.join(SELFTEST_HERE, "..", "experiment-apparatus", "replay.sh"), encoding="utf-8") as handle:
            text = handle.read()
        if text.count(REPLAY_JUDGE_FUNCTION_START) != 1:
            raise RegistrationError(f"replay.sh 里「{REPLAY_JUDGE_FUNCTION_START}」不是恰好一处，这一格接不上")
        if break_is_set("replay-store-inside-judge"):
            body = text.index("\n", text.index(REPLAY_JUDGE_FUNCTION_START)) + 1
            text = text[:body] + REPLAY_STORE_INSIDE_JUDGE_UNDER_BREAK + text[body:]
        write_replay_copy(work, text)
        os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "scripts/gate/admission.py"))
        copy_admission_modules(work)
        copy_function_units_library(work)
        context = KeyContext(work)
        outcomes = {name: context.judge_key(f"{REPLAY_SCRIPT}::{name}") for name in REPLAY_JUDGING_FUNCTIONS}
        refused = {name: refusals[:2] for name, (_key, _lines, refusals) in outcomes.items() if refusals}
        reaching = {name: [line for line in lines if "scripts/gate/admission.py" in line][:2] for name, (_key, lines, _refusals) in outcomes.items()}
        reaching = {name: lines for name, lines in reaching.items() if lines}
        selftest.expect(REPLAY_JUDGING_CLOSURE_CELL, not refused and not reaching and all(key for key, _lines, _refusals in outcomes.values()),
                        f"拒算 {refused}；闭包里 admission.py 的单元 {reaching}")
    except (OSError, RegistrationError) as error:
        selftest.expect(REPLAY_JUDGING_CLOSURE_CELL, False, f"这一格的现场出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_replay_row_key_cells(selftest, module):
    """复跑表按行的准入键（records/2026-09-16-subagent拆分提案.md 第六十七节）：临时仓里一个实验 E903 在复跑表里两行，拿真仓 replay.sh 的副本跑。
    外面设 ADMISSION_BREAK=ignore-row-keys 或 REPLAY_BREAK=row-key-by-experiment 时，「两行各给键」那几格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-row-keys-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
            write_executable(os.path.join(tools, name), script)
        os.makedirs(os.path.join(work, "scripts", "gate"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "scripts/gate/admission.py"))
        copy_admission_modules(work)
        copy_function_units_library(work)
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "E903\tresearch/src/e903.sh\t# 样本：E903 两行里默认那一种（装置自己打产物头）\n"
                   "E903/second\tresearch/src/e903-second.sh\t# 样本：E903 第二行（跑的东西不接准入，产物头由 replay.sh 写）\n"
                   "E903/third\t@E903\tproduct-field=E903:verdict:control_ok=true\t# 样本：按实验号点名产物，复跑表里键是 E903 的只一行\n")
        stored_body = "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n"
        for name in ("e903", "e903-second"):
            write_text(os.path.join(work, f"research/src/{name}.sh"), f"#!/usr/bin/env bash\necho 输入 {name}\n")
            write_text(os.path.join(work, RESULTS_DIRECTORY, f"{name}-2026-09-01.out"), stored_body)
        write_text(os.path.join(work, REPLAY_TIMING_LAG_TABLE), REPLAY_TIMING_LAG_TABLE_HEAD)
        link_sop_copy(work)
        link_replay_build_log_tools(work)
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
        table_rows = ["E903|@driver_e903||e903-2026-09-01.out|exact", "E903|@driver_e903_second||e903-second-2026-09-01.out|exact"]
        row_keys = ["E903\tE903|@driver_e903|\tdevice", "E903/second\tE903|@driver_e903_second|\treplay"]

        def replay(arguments, keys):
            write_replay_copy(work, replay_copy_text(table_rows, False, keys))
            output_directory = tempfile.mkdtemp(prefix="out-", dir=work)
            exit_code, output, messages = run_quietly(
                ["bash", os.path.join(work, REPLAY_SCRIPT), *arguments],
                {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "REPLAY_OUT": output_directory, "REPLAY_JOBS": "1"})
            ran = sorted(name for name in os.listdir(output_directory) if name.startswith("driver-ran-"))
            return exit_code, output + messages, output_directory, ran

        def head_of(path):
            if not os.path.isfile(path):
                return ""
            with open(path, encoding="utf-8", errors="replace") as handle:
                return handle.read()[:400]

        exit_code, output, _run, ran = replay(["E903"], ())
        selftest.expect("复跑表按行的准入键：同一实验两行、只按实验号登记时 replay.sh 整份退 2，红在「准入只按实验号比产物」那一句，一行都没跑",
                        exit_code == 2 and "E903 登记了准入，而这张复跑表里它有 2 行：准入只按实验号比产物，分不清哪一行对哪一份" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        exit_code, output, keyed_run, ran = replay(["E903"], row_keys)
        first_head, second_head = head_of(os.path.join(keyed_run, "E903.1.out")), head_of(os.path.join(keyed_run, "E903.2.out"))
        selftest.expect("复跑表按行的准入键：两行各给键、各自登记时两行都跑、都判字节一致；第一行产物头是装置打的 key=E903，第二行是 replay.sh 接上的 key=E903/second",
                        exit_code == 0 and len(re.findall(r"^E903 .*字节一致", output, re.M)) == 2
                        and "E7INPUT name=input_fingerprint key=E903 " in first_head
                        and first_head.startswith("E7INPUT name=input_fingerprint key=E903 ")
                        and second_head.startswith("E7INPUT name=input_fingerprint key=E903/second "),
                        f"退 {exit_code}，跑了 {ran}，第一行产物头「{first_head[:120]}」，第二行「{second_head[:120]}」，输出：{output.strip()[-600:]}")
        exit_code, output, _run, ran = replay(["--admission-only", "E903"], row_keys)
        selftest.expect("复跑表按行的准入键：--admission-only 逐行列出各自的键与判定，一行都没跑",
                        exit_code == 0 and re.search(r"^E903 +@driver_e903 +放行 +准入键 E903，", output, re.M) is not None
                        and re.search(r"^E903 +@driver_e903_second +放行 +准入键 E903/second，", output, re.M) is not None and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        unregistered_keys = [row_keys[0], "E903/fourth\tE903|@driver_e903_second|\treplay"]
        exit_code, output, _run, ran = replay(["E903"], unregistered_keys)
        selftest.expect("复跑表按行的准入键：两行各给键而其中一个没在登记表登记时整份退 2，点名那个键，一行都没跑",
                        exit_code == 2 and f"指明的准入键 E903/fourth 没在 {REGISTRATION_TABLE} 登记" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        dangling_keys = [row_keys[0], "E903/second\tE903|@driver_e903_missing|\treplay"]
        exit_code, output, _run, ran = replay(["E903"], dangling_keys)
        selftest.expect("复跑表按行的准入键：指到复跑表里没有的行时整份退 2，点名那一条",
                        exit_code == 2 and "「E903|@driver_e903_missing|」在复跑表里有 0 行，要恰好一行" in output and not ran,
                        f"退 {exit_code}，跑了 {ran}，输出：{output.strip()[-600:]}")
        write_replay_copy(work, replay_copy_text(table_rows, False, row_keys))
        exit_code, _output, messages = run_quietly(["python3", os.path.join(work, "scripts/gate/admission.py"), "experiment", work, "E903/third"])
        selftest.expect("复跑表按行的准入键：product-field=E903 只认键是 E903 的那一行产物（复跑表里 E903 两行），前提判得出、放行",
                        exit_code == 0 and "e903-2026-09-01.out 的 name=verdict control_ok=true" in messages, f"退 {exit_code}，stderr：{messages.strip()[-600:]}")
    except RegistrationError as error:
        selftest.expect("复跑表按行的准入键：replay.sh 的副本拼得出来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)




def run_plan_failure_cells(selftest, module):
    """plan-failure（文件头「按条复用失灵」）：拿造出来的几趟输出起它，判红的退 1、点名哪一条哨兵或退了几；外面设了 ADMISSION_BREAK=plan-failure-not-red，
    判红的那几格转错（子进程照传外面的环境）。两种样本标记在单元键那几格的临时小仓上起真的 verdict-plan。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-plan-failure-")
    try:
        def failure(command, exit_code, text, changes=None):
            path = os.path.join(work, f"output.{len(os.listdir(work))}")
            write_text(path, text)
            return run_in_environment([sys.executable, module, "plan-failure", "样本调用方的", command, str(exit_code), path], dict(os.environ, **(changes or {})))
        cases = (
            ("哨兵报复用失灵：verdict-plan 退 1、红句点名那一项与「复用失灵」", "verdict-plan", 1,
             "a\treuse\tk\t键相同\n  · 复用哨兵：…\nPLAN judge=0 reuse=1 refused=0 library=x\n  ✗ 复用失灵：a 键相同却现判了（复用哨兵，种子 7：这一趟报的是「现判」）\n     → 怎么办：…\n",
             1, ("样本调用方的按条复用失灵：复用哨兵报红（复用失灵）：复用失灵：a 键相同却现判了", "文件头「复用哨兵」")),
            ("哨兵报判定不稳：verdict-record 退 1 也判红", "verdict-record", 1, "  ✗ 判定不稳：b（库里那一条是「绿」，这一趟强制现判是「红」）\nRECORD recorded=0 changed=0 refused=0 library=x\n",
             1, ("复用哨兵报红（判定不稳）：判定不稳：b", "文件头「单元键」")),
            ("哨兵报强制现判没起作用：test-plan 退 1 判红", "test-plan", 1, "  ✗ 复用哨兵（x）：抽中强制现判的 c 这一趟还是复用了，强制现判没起作用（种子 3）\n",
             1, ("复用哨兵报红（强制现判没起作用）：复用哨兵（x）：抽中强制现判的 c", "先问 forced")),
            ("verdict-plan 退 2（读不了请求文件）：红句写退 2 与那一行 ✗、出路指到子命令那一条", "verdict-plan", 2, "  ✗ 读不了请求文件 r：不在\n     → 怎么办：…\n",
             1, ("scripts/gate/admission.py verdict-plan 退 2（✗ 读不了请求文件 r：不在）", "文件头子命令「verdict-plan」")),
            ("test-plan 退 0 却没打 PLAN 那一行：判红", "test-plan", 0, "x\tjudge\tk\n", 1, ("test-plan 退 0 却没打 PLAN 那一行",)),
            ("verdict-plan 退 0、打了 PLAN、没有哨兵红句：不判、一个字不打", "verdict-plan", 0, "a\tjudge\tk\t空库\nPLAN judge=1 reuse=0 refused=0 library=x\n", 0, ()),
            ("verdict-record 退 2 而没有哨兵红句（记不进库）：不算失灵、退 0", "verdict-record", 2, "  ✗ 读不了计划文件或名字文件：不在\n", 0, ()),
            ("verdict-record 打了「判红没记进判定库」而没带 --red：判红、说撤不了", "verdict-record", 1,
             "  ✗ 判红没记进判定库：a（键 0123456789ab）：样本档造的记红没成\nRECORD recorded=0 changed=0 refused=0 red=0 red-unrecorded=1 library=x\n",
             1, ("样本调用方的按条复用失灵：判红没记进判定库：a（键 0123456789ab）", "调用方没交 --red", "整道判红")),
            ("verdict-record 退 3（prune-server 客户端没配服务地址）：判红、点名客户端那一行与它的出路，不当记不进库放过", "verdict-record", 3,
             "  ✗ prune-server 客户端：没有 socket 文件（样本），也没有 TCP 地址（退 3）\n     → 怎么办：本机要有服务的 socket（样本）\n",
             1, ("样本调用方的：prune-server 客户端退 3", "没有 socket 文件（样本）", "→ 怎么办：本机要有服务的 socket（样本）")),
            ("test-plan 退 3（prune-server 客户端没配服务地址）：判红、点名配置错，不说复用失灵", "test-plan", 3,
             "  ✗ prune-server 客户端：没有 socket 文件（样本），也没有 TCP 地址（退 3）\n     → 怎么办：本机要有服务的 socket（样本）\n",
             1, ("样本调用方的：prune-server 客户端退 3", "→ 怎么办：本机要有服务的 socket（样本）")),
        )
        # 三条子命令退 75：prune-server 不可用，不是复用失灵（文件头「按条复用失灵」）；弄坏开关 plan-unavailable-as-failure 下转错
        for command in ("verdict-plan", "test-plan", "verdict-record"):
            got_exit, output, messages = failure(command, PLAN_SERVICE_UNAVAILABLE_EXIT, "  ✗ prune-server 连不上：重连到上限（样本）\n")
            selftest.expect(f"plan-failure：{command} 退 75：判 prune-server 不可用、退 75、不说复用失灵",
                            got_exit == PLAN_SERVICE_UNAVAILABLE_EXIT and "样本调用方的：prune-server 不可用" in output and "python3 prune-server/control check" in output
                            and "按条复用失灵：" not in output, f"退 {got_exit}（要 75），输出：{(output + messages).strip()[-400:]}")
        for label, command, exit_code, text, wanted_exit, needles in cases:
            got_exit, output, messages = failure(command, exit_code, text)
            selftest.expect(f"plan-failure：{label}", got_exit == wanted_exit and all(needle in output for needle in needles) and (wanted_exit or not output.strip()),
                            f"退 {got_exit}（要 {wanted_exit}），输出：{(output + messages).strip()[-500:]}")
        got_exit, output, _messages = failure("verdict-plan", 1, "  ✗ 复用失灵：a 键相同却现判了\n", {BREAK_VARIABLE: "plan-failure-not-red"})
        selftest.expect("plan-failure：弄坏开关 plan-failure-not-red 在自己那一趟里打开，复用失灵也退 0、只打一行 !（改前不判红的样子）",
                        got_exit == 0 and "弄坏开关 plan-failure-not-red：退回全部现判、不判红" in output, f"退 {got_exit}，输出：{output.strip()[-300:]}")
        # 两种样本标记：单元键那几格的临时小仓上起真的 verdict-plan
        repository = os.path.join(work, "repository")
        environment = build_unit_sample_repository(repository)
        requests_file = os.path.join(work, "marker.requests")
        write_text(requests_file, "".join(request.text() + "\n" for request in unit_sample_requests()))
        write_text(os.path.join(repository, PLAN_FAILURE_SAMPLE_MARKER), "exit-75\n")
        plan_exit, plan_output, plan_messages = run_in_environment(
            [sys.executable, module, "verdict-plan", repository, "plan-failure-sample.sh", requests_file, os.path.join(work, "marker-exit-75.plan")], environment)
        write_text(os.path.join(work, "marker-exit-75.out"), plan_output + plan_messages)
        judged_exit, judged_output, _messages = run_in_environment(
            [sys.executable, module, "plan-failure", "样本调用方的", "verdict-plan", str(plan_exit), os.path.join(work, "marker-exit-75.out")], dict(os.environ))
        selftest.expect("plan-failure：样本标记 exit-75 下 verdict-plan 退 75、plan-failure 判不可用、退 75",
                        plan_exit == PLAN_SERVICE_UNAVAILABLE_EXIT and "样本档造的服务不可用" in plan_output and judged_exit == PLAN_SERVICE_UNAVAILABLE_EXIT
                        and "prune-server 不可用" in judged_output and "按条复用失灵：" not in judged_output,
                        f"verdict-plan 退 {plan_exit}：{(plan_output + plan_messages).strip()[-300:]}；plan-failure 退 {judged_exit}：{judged_output.strip()[-300:]}")
        for form, needles in (("exit-1", ("样本档造的失灵", "写着 exit-1")),
                              ("sentinel-reuse-failed", ("PLAN judge=", "✗ 复用失灵：crates/demo/tests/setting.rs::", "样本档造的假哨兵"))):
            write_text(os.path.join(repository, PLAN_FAILURE_SAMPLE_MARKER), form + "\n")
            plan_exit, plan_output, plan_messages = run_in_environment(
                [sys.executable, module, "verdict-plan", repository, "plan-failure-sample.sh", requests_file, os.path.join(work, f"marker-{form}.plan")], environment)
            output_path = os.path.join(work, f"marker-{form}.out")
            write_text(output_path, plan_output + plan_messages)
            judged_exit, judged_output, _messages = run_in_environment([sys.executable, module, "plan-failure", "样本调用方的", "verdict-plan", str(plan_exit), output_path],
                                                                       dict(os.environ))
            selftest.expect(f"plan-failure：样本标记 {form} 下 verdict-plan 退 1、plan-failure 判红",
                            plan_exit == 1 and all(needle in plan_output for needle in needles) and judged_exit == 1 and "样本调用方的按条复用失灵：" in judged_output,
                            f"verdict-plan 退 {plan_exit}：{(plan_output + plan_messages).strip()[-400:]}；plan-failure 退 {judged_exit}：{judged_output.strip()[-300:]}")
        write_text(os.path.join(repository, PLAN_FAILURE_SAMPLE_MARKER), "exit-2\n")
        plan_exit, plan_output, _messages = run_in_environment(
            [sys.executable, module, "verdict-plan", repository, "plan-failure-sample.sh", requests_file, os.path.join(work, "marker-bad.plan")], environment)
        selftest.expect("plan-failure：样本标记写的不是认得的三样，verdict-plan 退 2、点名那份标记", plan_exit == EXIT_REGISTRATION_ERROR and "认不出" in plan_output,
                        f"退 {plan_exit}：{plan_output.strip()[-300:]}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


EXPERIMENT_ADMISSION_ENTRY = "lib.rs::admit_experiment_run_or_exit"   # 研究工作区（REPLAY_RESEARCH_DIRECTORY）里每个接了准入的装置 main 都调它
PARENT_DIRECTORIES_ROOT_ENTRY = '''
pub fn root_by_parent_directories() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}
'''


def run_experiment_admission_entry_cells(selftest):
    """实验准入入口不拒算（它拒算时每个调它的实验键都是 refused，「输入没变不重跑」用不上）：真仓研究工作区里 EXPERIMENT_ADMISSION_ENTRY 的闭包没有拒算；
    同一份 lib.rs 拷进临时目录、补一个取路径的函数（写法：清单目录往上两级拼出仓根），它要拒算 runtime-path。
    lib_function_units.py 的弄坏开关 FUNCTION_UNITS_BREAK=rust-manifest-directory-reads-unrefused（拼出目录也不拒算）下这一格转红。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-admission-entry-")
    try:
        library = function_units()
        research = os.path.join(os.path.dirname(os.path.dirname(SELFTEST_HERE)), REPLAY_RESEARCH_DIRECTORY)
        real_refusals = [refusal.text() for refusal in library.closure_of(research, EXPERIMENT_ADMISSION_ENTRY, library.Options()).refusal_list()]
        with open(os.path.join(research, "lib.rs"), encoding="utf-8") as handle:
            source = handle.read()
        write_text(os.path.join(work, "Cargo.toml"), '[package]\nname = "e7-index-bench"\nversion = "0.1.0"\n\n[lib]\npath = "lib.rs"\n')
        write_text(os.path.join(work, "lib.rs"), source + PARENT_DIRECTORIES_ROOT_ENTRY)
        forms = {refusal.form for refusal in library.closure_of(work, "lib.rs::root_by_parent_directories", library.Options()).refusal_list()}
        selftest.expect(f"实验准入入口 {REPLAY_RESEARCH_DIRECTORY}/{EXPERIMENT_ADMISSION_ENTRY} 的闭包不拒算；清单目录往上拼出仓根取路径的入口拒算 runtime-path",
                        not real_refusals and "runtime-path" in forms,
                        f"准入入口的拒算 {real_refusals or '无'}；往上拼目录的那个入口拒算 {sorted(forms) or '无'}（要有 runtime-path）")
    except (library_errors() + (OSError, UnitKeyError)) as error:
        selftest.expect("实验准入入口的闭包算得出", False, f"{type(error).__name__}：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def unreachable_service_environment(work):
    """连不上的服务（socket 不在、TCP 端口刚放掉、重连总时长 2 秒）：子进程照它连，读一次就退 75。
    socket 在函数里 import：模块级的 import 进每一个含本模块单元的闭包，加一行会让那些判定的键全部作废。"""
    import socket
    probe = socket.socket()
    probe.bind(("127.0.0.1", 0))
    address = "127.0.0.1:%d" % probe.getsockname()[1]
    probe.close()
    return {"SINGLEFS_PRUNE_SERVER_SOCKET": os.path.join(work, "no-such.sock"), "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS": address,
            "SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS": "2"}


def run_library_files_cells(selftest, module):
    """library-files（文件头「library-files」）在临时服务上：存、查、拉回逐字节相同、再存全部跳过、库名所在的目录一个文件都没建；服务连不上 fetch 退 75。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-library-files-")
    try:
        library = os.path.join(work, "tree", ".git", VERDICT_LIBRARY_DIRECTORY_NAME, "replay-row")
        transit, fetched = os.path.join(work, "transit"), os.path.join(work, "fetched")
        record = {"keys/k1/product": "E7RESULT name=done emitted=1\n", "keys/k1/verdict": "pass\n", "items/d1": "k1\n"}
        for relative, content in record.items():
            write_text(os.path.join(transit, *relative.split("/")), content)
        write_text(os.path.join(transit, "keys", ".k1.tmp.1", "verdict"), "临时目录不搬\n")
        stored = run_quietly([sys.executable, module, "library-files", "store", library, transit, "keys/k1/product", "items/d1", "keys/k1/verdict"])
        again = run_quietly([sys.executable, module, "library-files", "store", library, transit])
        has = run_quietly([sys.executable, module, "library-files", "has", library, "keys/k1/verdict", "keys/k2/verdict"])
        pulled = run_quietly([sys.executable, module, "library-files", "fetch", library, fetched, "keys/k1/", "items/"])
        pulled_files = {relative: open(path, encoding="utf-8").read() for relative, path in transit_files(fetched)}
        selftest.expect("library-files：存进服务、按相对路径查得到、拉回逐字节相同，再存一遍全部跳过，库名所在的目录一个文件都没建",
                        stored[0] == 0 and "written=3 skipped=0" in stored[1] and again[0] == 0 and "written=0 skipped=3" in again[1]
                        and has[1].split() == ["keys/k1/verdict"] and pulled[0] == 0 and "entries=3" in pulled[1] and pulled_files == record
                        and not os.path.exists(os.path.dirname(library)),
                        f"存那一趟退 {stored[0]}：{stored[1].strip()}；再存退 {again[0]}：{again[1].strip()}；查到 {has[1].split()}；拉回退 {pulled[0]}：{pulled[1].strip()}，"
                        f"拉回的 {sorted(pulled_files)}；库名的父目录{'建了' if os.path.exists(os.path.dirname(library)) else '没建'}")
        unreachable = os.path.join(work, "unreachable")
        code, out, error = run_quietly([sys.executable, module, "library-files", "fetch", library, unreachable], unreachable_service_environment(work))
        selftest.expect("library-files：服务连不上时 fetch 退 75、中转目录里一个文件都没写",
                        code == 75 and not transit_files(unreachable) and "entries=" not in out,
                        f"退 {code}（要 75），中转目录里 {len(transit_files(unreachable))} 个文件，输出 {(out + error).strip()[-300:]}")
        code, out, _error = run_quietly([sys.executable, module, "library-files", "fetch", os.path.join(work, "not-a-library"), fetched])
        selftest.expect("library-files：库名不在 singlefs-unit-verdicts/ 底下退 2", code == EXIT_REGISTRATION_ERROR, f"退 {code}：{out.strip()}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def library_errors():
    module = FUNCTION_UNITS_MODULE.get("module")
    return (module.FunctionUnitsError, module.UsageError) if module is not None else ()


def run_cell_selection_cells(selftest, module):
    """选格（文件头「选格」）：起这份自证自己，只交这一格管的两条开关（外面设的别的开关不进那一趟）；点名的格是只有一项的 composition-packing。
    弄坏开关 selftest-cells-run-extra 下那一趟多跑 runner-script，selftest-cells-unknown-accepted 下写错的格名不退 2。"""
    switches = ",".join(name for name in ("selftest-cells-run-extra", "selftest-cells-unknown-accepted") if break_is_set(name))
    exit_code, output, messages = run_quietly([sys.executable, module, "--selftest"], {"ADMISSION_SELFTEST_CELLS": "composition-packing", BREAK_VARIABLE: switches})
    selftest.expect("选格：点名 composition-packing 只跑它：退 0、一项都对、别的格没跑",
                    exit_code == 0 and "只跑点名的 1 格（composition-packing）：1 项都对" in output and output.count("  ✓ ") == 2,
                    f"退 {exit_code}，打了 {output.count('  ✓ ')} 行 ✓，末行：{output.strip()[-200:]}{messages.strip()[-200:]}")
    exit_code, output, messages = run_quietly([sys.executable, module, "--selftest"], {"ADMISSION_SELFTEST_CELLS": "composition-packing", BREAK_VARIABLE: "test-plan-packing-literals-moved"})
    selftest.expect("选格：点名的格被开关弄坏：退 1、失败行点名那一格",
                    exit_code == 1 and "✗ 自证：composition-packing：" in output, f"退 {exit_code}，末行：{output.strip()[-200:]}{messages.strip()[-200:]}")
    exit_code, output, messages = run_quietly([sys.executable, module, "--selftest"], {"ADMISSION_SELFTEST_CELLS": "no-such-cell", BREAK_VARIABLE: switches})
    selftest.expect("选格：写错的格名退 2、点名写错的那个", exit_code == EXIT_REGISTRATION_ERROR and "no-such-cell" in output,
                    f"退 {exit_code}，末行：{output.strip()[-200:]}{messages.strip()[-200:]}")
