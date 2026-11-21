"""自证里在真仓拷贝上收闭包的格：import 时执行的语句、不走的边与准入边、卡租约客户端的程序词、main-guard、判法入口的拒算、E171、跨轮缓存、entry 行与键、产品行为视图。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import collections
import importlib.util
import os
import posixpath
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from lib_function_units_lexer import BREAK_VARIABLE, RUST_LEX_CACHE_DIRECTORY_VARIABLE, active_breaks, rust_lex
from lib_function_units_base import (
    DEFAULT_SCRIPT_DIRECTORIES, FORMAL_VERIFICATION, LIBRARY_ROOT, PRODUCT_BEHAVIOR, UNIT_GRAPH_DIRECTORY_VARIABLE, Unit, VERUS_GHOST_CODE_UNPARSED,
    sha256_text,
)
from lib_function_units_registry import ADMISSION_EDGE, EXCLUDED_PROGRAM_EDGES, NOT_JUDGING_FILES, UNKEYED_PROGRAM_WORDS
from lib_function_units_python_facts import is_docstring
from lib_function_units_rust_workspace import RustWorkspace
from lib_function_units_rust_closure import rust_closure_of, rust_declared_names
from lib_function_units_script_closure import Options, SHARED_SCRIPT_INDICES, ScriptClosure, closure_of, parse_entry
from lib_function_units_closure_keys import (
    closure_key, cross_run_cache_registry, dump_lines, edit_file, unit_graph_key, unit_graph_key_lines, work_root_library, write_tree,
)
from lib_function_units_selftest_fixture_trees import (
    compare_cell, fixture_key, judging_entry_refusal_workers, module_literal, module_literal_location, prepare_shared_script_index, real_tree,
    registered_judging_entries,
)
from lib_function_units_selftest_closure_rules import selftest_git_steps


def run_fixture_program(base, path):
    """夹具里的小程序跑一遍交退出码（60 秒没退交 None）。"""
    try:
        return subprocess.run([sys.executable, "-B", os.path.join(base, path)], cwd=base, capture_output=True, text=True, timeout=60).returncode
    except subprocess.TimeoutExpired:
        return None


def import_time_exit_cell(work):
    """每一种改动各在一棵新树上量（键、python3 tool/entry.py 的退出码）与原树比：退出码变了而键不变的记下；前四种一个都不许有，对照那一种必须有。"""
    base = os.path.join(work, "base")
    write_tree(base, IMPORT_TIME_FIXTURE)
    before_key = closure_key(closure_of(base, IMPORT_TIME_ENTRY), allow_refused=True)[0]
    before_exit = run_fixture_program(base, "tool/entry.py")
    blind, lines = [], []
    for name, path, old, new in IMPORT_TIME_EDITS + (IMPORT_TIME_CONTROL,):
        tree = os.path.join(work, f"edit-{len(lines)}")
        write_tree(tree, IMPORT_TIME_FIXTURE)
        edit_file(tree, path, old, new)
        key = closure_key(closure_of(tree, IMPORT_TIME_ENTRY), allow_refused=True)[0]
        exit_code = run_fixture_program(tree, "tool/entry.py")
        lines.append(f"{name}：退出码 {before_exit}→{exit_code}，键{'变' if key != before_key else '不变'}")
        if exit_code != before_exit and key == before_key:
            blind.append(name)
    want = [IMPORT_TIME_CONTROL[0]]
    return blind == want, f"键不变而退出码变的（要恰好是对照那一种 {want}）：{blind}；{'；'.join(lines)}"


def real_entries(root):
    """形式化入口：从登记现取，取 verus-proof-check.py 登记的头一个（入口文件是 JUDGING_ENTRIES 赋值所在的那一份，拆出的模块也认：module_literal_location）。"""
    entries = registered_judging_entries(root)
    _formal_entries, formal_file = module_literal_location(root, REAL_FORMAL_SCRIPT, "JUDGING_ENTRIES")
    formal = next((entry for entry in entries if entry.startswith(formal_file + "::")), None)
    if formal is None:
        raise AssertionError(f"登记现取的 {len(entries)} 个判法入口里没有 {REAL_FORMAL_SCRIPT} 的入口：{entries}")
    return formal


def real_import_time_results():
    """真仓三格一起量（一棵拷贝、形式化入口的原键算一次）：{格名: (通过, 说明)}；同一个进程、同一组弄坏开关只量一次。"""
    breaks = os.environ.get(BREAK_VARIABLE, "")
    if breaks not in _REAL_IMPORT_TIME_RESULTS:
        results = {}
        with tempfile.TemporaryDirectory(prefix="function-units-real-tree-") as work:
            tree = real_tree(work)
            entry = real_entries(tree)
            base_key = closure_key(closure_of(tree, entry), allow_refused=True)[0]
            for name, path, old, new in REAL_IMPORT_TIME_EDITS:
                absolute = os.path.join(tree, path)
                with open(absolute, encoding="utf-8") as handle:
                    original = handle.read()
                if original.count(old) != 1:
                    results[name] = (False, f"{path} 里「{old.strip()}」命中 {original.count(old)} 次（要恰好一次）：那份文件改了，照新原文改 REAL_IMPORT_TIME_EDITS")
                    continue
                with open(absolute, "w", encoding="utf-8") as handle:
                    handle.write(original.replace(old, new))
                try:
                    key = closure_key(closure_of(tree, entry), allow_refused=True)[0]
                finally:
                    with open(absolute, "w", encoding="utf-8") as handle:
                        handle.write(original)
                results[name] = (key != base_key, f"{entry} 的键改前 {base_key}、改后 {key}")
        _REAL_IMPORT_TIME_RESULTS[breaks] = results
    return _REAL_IMPORT_TIME_RESULTS[breaks]


def real_import_time_cell(name):
    def judge(_work):
        return real_import_time_results()[name]
    return judge


def excluded_edge_cell(entry, hint, kept):
    """夹具上证 EXCLUDED_PROGRAM_EDGES 的机制：入口单元起 hint 与 kept 两个程序。不登记时两个都进闭包；自证期间临时把夹具里 hint 那一条边登记进表
    （EXCLUDED_EDGE_FIXTURE_EDGES），登记了的 hint 不进闭包、没登记的 kept 照进；跑完把表还原。弄坏开关 program-edge-exclusions-ignored 下登记了也照连，
    program-edge-exclusions-shell-ignored 下 shell 函数起的登记了也照连。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, EXCLUDED_EDGE_FIXTURE)

        def closure_files():
            return {unit.file for unit in closure_of(base, entry).order}

        uncut = closure_files()
        registered = dict(EXCLUDED_PROGRAM_EDGES)
        EXCLUDED_PROGRAM_EDGES.update(EXCLUDED_EDGE_FIXTURE_EDGES)
        try:
            cut = closure_files()
        finally:
            EXCLUDED_PROGRAM_EDGES.clear()
            EXCLUDED_PROGRAM_EDGES.update(registered)
        ok = {hint, kept} <= uncut and kept in cut and hint not in cut
        return ok, (f"{entry}：不登记时 {sorted(uncut & {hint, kept})}、登记 {hint} 那一条边之后 {sorted(cut & {hint, kept})}"
                    f"（要：不登记时 {sorted({hint, kept})} 都在，登记之后只剩 {kept}）")
    return judge



def excluded_edge_exit_cell(work):
    """自证期间临时登记 EXCLUDED_EDGE_EXIT_EDGES；每个包装在原树与改坏 hint.py 的树上各量（键、python3 <包装> 的退出码），
    退出码变了而键不变的记下，要恰好是 leak.py。弄坏开关 excluded-edge-exit-unmeasured 下改坏之后不重跑、退出码照改坏之前记。"""
    base, edited = os.path.join(work, "base"), os.path.join(work, "edited")
    write_tree(base, EXCLUDED_EDGE_EXIT_FIXTURE)
    write_tree(edited, EXCLUDED_EDGE_EXIT_FIXTURE)
    edit_file(edited, *EXCLUDED_EDGE_EXIT_EDIT)
    registered = dict(EXCLUDED_PROGRAM_EDGES)
    EXCLUDED_PROGRAM_EDGES.update(EXCLUDED_EDGE_EXIT_EDGES)
    blind, lines = [], []
    try:
        for wrapper in EXCLUDED_EDGE_EXIT_WRAPPERS:
            entry = f"{wrapper}::run"
            before_key = closure_key(closure_of(base, entry), allow_refused=True)[0]
            after_key = closure_key(closure_of(edited, entry), allow_refused=True)[0]
            before_exit = run_fixture_program(base, wrapper)
            after_exit = before_exit if "excluded-edge-exit-unmeasured" in active_breaks() else run_fixture_program(edited, wrapper)
            lines.append(f"{wrapper}：退出码 {before_exit}→{after_exit}，键{'变' if after_key != before_key else '不变'}")
            if after_exit != before_exit and after_key == before_key:
                blind.append(wrapper)
    finally:
        EXCLUDED_PROGRAM_EDGES.clear()
        EXCLUDED_PROGRAM_EDGES.update(registered)
    want = ["tool/leak.py"]
    return blind == want, f"键不变而退出码变的（要恰好是 {want}）：{blind}；{'；'.join(lines)}"


def root_ignored_scripts_cell(work):
    """ROOT_IGNORED_SCRIPTS_CELL（文件头「仓根被忽略的私有文件」）：夹具的入口照 os.path.join(<根>, <折不出的名字>) 起 bash，列到仓根整层的两份脚本，
    两份都起同一个仓外路径；.gitignore 挡着的 ROOT_IGNORED_SCRIPT 不进闭包、闭包里没有落在它上面的拒算，没挡着的 ROOT_VISIBLE_SCRIPT 进闭包、照拒算 shell-path-outside-root。
    弄坏开关 root-ignored-scripts-linked 下挡着的那一份照连、跟着拒算，这一格红。"""
    base = os.path.join(work, "tree")
    write_tree(base, ROOT_IGNORED_SCRIPTS_FIXTURE)
    closure = closure_of(base, ROOT_IGNORED_SCRIPTS_ENTRY)
    files = {unit.file for unit in closure.order}
    refused = sorted({(refusal.form, refusal.file) for refusal in closure.refusals})
    ignored_kept_out = ROOT_IGNORED_SCRIPT not in files and all(file != ROOT_IGNORED_SCRIPT for _form, file in refused)
    visible_refused = ROOT_VISIBLE_SCRIPT in files and ("shell-path-outside-root", ROOT_VISIBLE_SCRIPT) in refused
    return ignored_kept_out and visible_refused, (f"闭包碰到的文件 {sorted(files)}，拒算 {refused}（要：没有 {ROOT_IGNORED_SCRIPT}、也没有落在它上面的拒算；"
                                                  f"有 {ROOT_VISIBLE_SCRIPT}、它拒算 shell-path-outside-root）")



def admission_edge_anomaly_cell(work):
    """每个假准入各起一次本仓跑法脚本的计划那一步（服务给的变量清掉、只设计划那一步要的；运行根、计划目录、产物目录都在临时目录；
    PATH 最前面放记一笔就退 1 的假 cargo）。每一个都要：问到了准入、退非 0、计划目录空、没起编译、没出产物。"""
    script = os.path.join(LIBRARY_ROOT, ADMISSION_EDGE[0])
    if not os.path.isfile(script):
        return False, f"{ADMISSION_EDGE[0]} 不在：准入边的起点没了，从 EXCLUDED_PROGRAM_EDGES 删掉这一条、连同这三格"
    fakes = os.path.join(work, "fakes")
    os.makedirs(fakes)
    cargo = os.path.join(fakes, "cargo")
    with open(cargo, "w", encoding="utf-8") as handle:
        handle.write(ADMISSION_EDGE_FAKE_CARGO)
    os.chmod(cargo, 0o755)
    samples = ADMISSION_EDGE_ANOMALIES + ((ADMISSION_EDGE_ADMITTED,) if "admission-edge-admitted-sample" in active_breaks() else ())
    clean = {name: value for name, value in os.environ.items()
             if not name.endswith("_BREAK") and not name.startswith("TASK_SERVER_") and name not in ADMISSION_EDGE_DROPPED_VARIABLES}
    unasked, leaked, lines = [], [], []
    for number, (name, body) in enumerate(samples):
        cell = os.path.join(work, f"sample-{number}")
        plan, scratch, product = (os.path.join(cell, part) for part in ("plan", "tmp", "product"))
        os.makedirs(plan)
        os.makedirs(scratch)
        admission, mark, calls = (os.path.join(cell, part) for part in ("admission.py", "asked", "cargo-calls"))
        with open(admission, "w", encoding="utf-8") as handle:
            handle.write(ADMISSION_EDGE_MARK_LINE + body)
        environment = dict(clean, PATH=fakes + os.pathsep + clean.get("PATH", ""), TASK_SERVER_SHARD_PHASE="plan",
                           TASK_SERVER_SHARD_PLAN_DIRECTORY=plan, E183_RUN_ROOT=os.path.join(cell, "run-root"), E183_PRODUCT_DIRECTORY=product,
                           E183_ADMISSION_SCRIPT=admission, ADMISSION_EDGE_MARK=mark, ADMISSION_EDGE_CARGO_CALLS=calls, TMPDIR=scratch)
        try:
            completed = subprocess.run(["bash", script, "run"], env=environment, capture_output=True, text=True, timeout=120)
            exit_code, said = completed.returncode, (completed.stdout + completed.stderr).strip().splitlines()[-1:]
        except subprocess.TimeoutExpired:
            exit_code, said = None, ["120 秒没退"]
        planned = sorted(os.listdir(plan))
        produced = os.path.isdir(product) and bool(os.listdir(product))
        compiled = os.path.exists(calls)
        lines.append(f"{name}：退 {exit_code}，计划目录 {planned or '空'}，{'起了编译' if compiled else '没起编译'}，{'出了产物' if produced else '没出产物'}，末行 {said}")
        if not os.path.exists(mark):
            unasked.append(name)
        elif exit_code in (0, None) or planned or compiled or produced:
            leaked.append(name)
    return not unasked and not leaked, (f"没问到准入的（跑法脚本在问准入之前就退了，这一格量不到）：{unasked}；往下走了的：{leaked}"
                                        f"（要都空）；{'；'.join(lines)}")


def admission_edge_strip_cell(_work):
    """载入本仓 admission.py（另起一份模块），实验键换成假的（键算出、拒算两样；不收闭包、不起 rustc）、「输入没变」换成一律变了，
    起 command_experiment 收它的标准输出：每一行都要以 E7INPUT 起头；接上假读数过判法去头的那条命令（admission.py 的 REPLAY_STRIP_HEADER_COMMAND），
    剩下的要与假读数逐行相同。弄坏开关 admission-edge-header-unstripped 下不去头。"""
    import contextlib
    import io
    import types
    path = os.path.join(LIBRARY_ROOT, ADMISSION_EDGE[2])
    specification = importlib.util.spec_from_file_location("admission_for_excluded_edge_selftest", path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    module.judge_inputs_unchanged = lambda _root, _key, _fingerprint: (False, "（自证：不比上一份产物）")
    problems, lines = [], []
    for label, key in (("键算出", "ab" * 32), ("拒算", None)):
        fake = types.SimpleNamespace(key=key, composition=[], refusals=["selftest-form\tselftest.py:1\tselftest"])
        module.experiment_key = lambda _root, _key, fake=fake: (fake, "自证假键")
        printed, messages = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(printed), contextlib.redirect_stderr(messages):
            exit_code = module.command_experiment([LIBRARY_ROOT, ADMISSION_EDGE_KEY])
        header = printed.getvalue().splitlines()
        composed = "".join(line + "\n" for line in header + list(ADMISSION_EDGE_READINGS))
        if "admission-edge-header-unstripped" in active_breaks():
            stripped = composed
        else:
            stripped = subprocess.run(["sh", "-c", module.REPLAY_STRIP_HEADER_COMMAND], input=composed, capture_output=True, text=True,
                                      timeout=60).stdout
        stray = [line for line in header if not line.startswith(module.HEADER_PREFIX)]
        lines.append(f"{label}：退 {exit_code}，打了 {header}，不以 E7INPUT 起头的 {stray}，去头之后 {stripped.splitlines()}")
        if exit_code != module.EXIT_ADMITTED or not header or stray or stripped.splitlines() != list(ADMISSION_EDGE_READINGS):
            problems.append(f"{label}（准入的说明末行 {messages.getvalue().strip().splitlines()[-1:]}）")
    return not problems, f"判错的：{problems}（要空；去头之后要恰好是 {list(ADMISSION_EDGE_READINGS)}）；{'；'.join(lines)}"


def admission_edge_reach_cell(work):
    """本仓 scripts/ 与 .claude/ 的拷贝上收跑法脚本整份的闭包：登记着（切边）与临时摘掉这一条（不切）各一次，少掉的单元都要在
    admission.py 当程序起的闭包里，少掉的里面要有 admission.py 的单元，切边之后不许多出单元。弄坏开关 admission-edge-reach-unmeasured 下
    不量 admission.py 的闭包（当它是空的）。"""
    tree = real_tree(work)
    entry, _unit, program = ADMISSION_EDGE

    def names(closure):
        return {(unit.file, unit.name) for unit in closure.order}

    cut_closure = closure_of(tree, entry)
    cut = names(cut_closure)
    registered = dict(EXCLUDED_PROGRAM_EDGES)
    EXCLUDED_PROGRAM_EDGES.pop(ADMISSION_EDGE, None)
    try:
        uncut = names(closure_of(tree, entry))
    finally:
        EXCLUDED_PROGRAM_EDGES.clear()
        EXCLUDED_PROGRAM_EDGES.update(registered)
    reach = set() if "admission-edge-reach-unmeasured" in active_breaks() else names(closure_of(tree, program))
    removed, added = uncut - cut, cut - uncut
    outside = sorted(removed - reach)
    cut_program = any(file == program for file, _name in removed)
    return cut_program and not outside and not added, (
        f"{entry}：不切 {len(uncut)} 个单元、切边 {len(cut)} 个，少掉 {len(removed)} 个（{'有' if cut_program else '没有'} {program} 的单元），"
        f"少掉的不在 {program} 当程序起的闭包里的 {len(outside)} 个 {outside[:5]}，切边后多出的 {sorted(added)[:5]}；切边后拒算 {len(cut_closure.refusal_list())} 处 "
        f"{[refusal.text() for refusal in cut_closure.refusal_list()[:3]]}（要：有 {program} 的单元、另两样都空）")


CARD_LEASE_CLIENT_WORDS_NOISE = "CARD_LEASE_CLIENT_WORDS_NOISE"


def card_lease_client_words_text():
    """本仓 device_card_lease.py 的原文。转走还卡输出的原文（CARD_LEASE_CLIENT_WORDS_RETURN 的左边）要恰好一次，不是就抛 AssertionError 点名；
    弄坏开关 card-lease-client-words-return-output-kept 下换成右边。"""
    with open(os.path.join(LIBRARY_ROOT, CARD_LEASE_CLIENT_WORDS_LAUNCHER), encoding="utf-8") as handle:
        text = handle.read()
    kept, passed = CARD_LEASE_CLIENT_WORDS_RETURN
    if text.count(kept) != 1:
        raise AssertionError(f"{CARD_LEASE_CLIENT_WORDS_LAUNCHER} 里「{kept}」命中 {text.count(kept)} 次（要恰好一次）：还卡那一回客户端的标准输出要转走、"
                             "不进调用方的标准输出；原文改了就照新原文改 CARD_LEASE_CLIENT_WORDS_RETURN")
    if "card-lease-client-words-return-output-kept" in active_breaks():
        text = text.replace(kept, passed)
    return text


def card_lease_client_words_bench(work):
    """一格的目录：task-server/task 是假领卡客户端（device_card_lease.py 照 <树根>/task-server/task 起它，树根就是这个目录），本仓 device_card_lease.py
    的原文与起法写在旁边；→ (目录, 环境)。环境去掉弄坏开关、服务给的变量与 GPU_CARDS_BY_PRIORITY。"""
    bench = os.path.join(work, "card-lease-client-words")
    write_tree(bench, {"task-server/task": CARD_LEASE_CLIENT_WORDS_FAKE_CLIENT, "device_card_lease.py": card_lease_client_words_text(),
                       "harness.py": CARD_LEASE_CLIENT_WORDS_HARNESS})
    os.chmod(os.path.join(bench, "task-server", "task"), 0o755)
    environment = {name: value for name, value in os.environ.items()
                   if not name.endswith("_BREAK") and not name.startswith("TASK_SERVER_") and name != "GPU_CARDS_BY_PRIORITY"}
    environment.update(CARD_LEASE_CLIENT_WORDS_READINGS="\n".join(CARD_LEASE_CLIENT_WORDS_READINGS),
                       CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT=str(CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT))
    return bench, environment


def card_lease_client_words_run(bench, environment, take, give_back, device_exit):
    """在 bench 上起一趟：子进程载入 device_card_lease.py 领卡、装置、还卡；假客户端领卡那一回演 take、还卡那一回演 give_back，装置退 device_exit。
    → (退出码，120 秒没退是 None；标准输出；标准错误；事件行)。"""
    events = os.path.join(bench, f"events-{take}-{give_back}-{device_exit}")
    run_environment = dict(environment, CARD_LEASE_CLIENT_WORDS_EVENTS=events, CARD_LEASE_CLIENT_WORDS_TAKE=take, CARD_LEASE_CLIENT_WORDS_RETURN=give_back,
                           CARD_LEASE_CLIENT_WORDS_DEVICE_EXIT=str(device_exit))
    command = [sys.executable, os.path.join(bench, "harness.py"), os.path.join(bench, "device_card_lease.py"), os.path.join(bench, "task-server", "task"), bench]
    try:
        completed = subprocess.run(command, cwd=bench, env=run_environment, capture_output=True, text=True, timeout=120)
        exit_code, stdout, stderr = completed.returncode, completed.stdout, completed.stderr
    except subprocess.TimeoutExpired:
        exit_code, stdout, stderr = None, "", "120 秒没退"
    seen = []
    if os.path.exists(events):
        with open(events, encoding="utf-8") as handle:
            seen = handle.read().splitlines()
    return exit_code, stdout, stderr, seen


def card_lease_client_words_anomaly_cell(work):
    """领卡那几个异常（还卡照常、装置退 0）都要问到了领卡、take_card 抛 CardLeaseRefused（起法退 CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT）、装置没起、
    标准输出空着；还卡那几个异常（领卡照常）在装置退 0 与退 3 下都要装置照卡串起一回、照卡串还一回、退出码是装置的、标准输出恰好是装置的读数。
    弄坏开关 card-lease-client-words-granted-sample 下领卡那几个里多一个领到了的。"""
    bench, environment = card_lease_client_words_bench(work)
    takes = CARD_LEASE_CLIENT_WORDS_TAKE_ANOMALIES + (
        (CARD_LEASE_CLIENT_WORDS_GRANTED_SAMPLE,) if "card-lease-client-words-granted-sample" in active_breaks() else ())
    wrong, lines = [], []
    for label, take in takes:
        exit_code, stdout, _stderr, seen = card_lease_client_words_run(bench, environment, take, "granted", 0)
        started = [line for line in seen if line.startswith("device:")]
        asked = any(line.startswith("lease:take:") for line in seen)
        lines.append(f"领卡{label}：退 {exit_code}，{'问了' if asked else '没问'}领卡，装置起了 {len(started)} 回，标准输出 {stdout.splitlines()[:3]}")
        if not asked or exit_code != CARD_LEASE_CLIENT_WORDS_REFUSED_EXIT or started or stdout:
            wrong.append(f"领卡{label}")
    for label, give_back in CARD_LEASE_CLIENT_WORDS_RETURN_ANOMALIES:
        for device_exit in CARD_LEASE_CLIENT_WORDS_DEVICE_EXITS:
            exit_code, stdout, _stderr, seen = card_lease_client_words_run(bench, environment, "granted", give_back, device_exit)
            started = [line for line in seen if line.startswith("device:")]
            returned = [line for line in seen if line.startswith("lease:return:")]
            lines.append(f"还卡{label}、装置退 {device_exit}：退 {exit_code}，装置 {started}，还卡 {returned}，标准输出 {stdout.splitlines()[:3]}")
            if exit_code != device_exit or started != [f"device:{CARD_LEASE_CLIENT_WORDS_CARD}"] \
                    or returned != [f"lease:return:{CARD_LEASE_CLIENT_WORDS_CARD}"] or stdout.splitlines() != list(CARD_LEASE_CLIENT_WORDS_READINGS):
                wrong.append(f"还卡{label}、装置退 {device_exit}")
    return not wrong, f"判错的：{wrong}（要空）；{'；'.join(lines)}"


def card_lease_client_words_strip_cell(work):
    """起一趟：领卡照常，还卡那一回假客户端往标准输出与标准错误各打一行 CARD_LEASE_CLIENT_WORDS_NOISE，装置退 0。要：退 0、标准输出恰好是装置的读数、
    那一行在标准错误里出现两回（客户端自己打的与 return_card 转过来的）。弄坏开关 card-lease-client-words-return-output-kept 下 return_card 不转走
    （card_lease_client_words_text）。"""
    bench, environment = card_lease_client_words_bench(work)
    exit_code, stdout, stderr, seen = card_lease_client_words_run(bench, environment, "granted", "noisy", 0)
    routed = stderr.splitlines().count(CARD_LEASE_CLIENT_WORDS_NOISE)
    readings = list(CARD_LEASE_CLIENT_WORDS_READINGS)
    return exit_code == 0 and stdout.splitlines() == readings and routed == 2, (
        f"退 {exit_code}（要 0），标准输出 {stdout.splitlines()}（要恰好是 {readings}），标准错误里 {CARD_LEASE_CLIENT_WORDS_NOISE} {routed} 行（要 2），"
        f"事件 {seen}")


def card_lease_client_words_parse(path, text):
    """解析一份原文；解析不了抛 AssertionError 点名那份文件。"""
    try:
        return ast.parse(text, filename=path)
    except SyntaxError as error:
        raise AssertionError(f"{path} 解析不了：{error}") from error


def card_lease_client_words_launches(call):
    """这一处调用起不起子进程：subprocess 上的任何函数，os.system、os.popen、os.exec*、os.spawn*、os.posix_spawn*。"""
    function = call.func
    if not (isinstance(function, ast.Attribute) and isinstance(function.value, ast.Name)):
        return False
    if function.value.id == "subprocess":
        return True
    return function.value.id == "os" and (function.attr in ("system", "popen") or function.attr.startswith(("exec", "spawn", "posix_spawn")))


def card_lease_client_words_named(call, name):
    """这一处调用是不是 device_card_lease.<name>(…)。"""
    function = call.func
    return (isinstance(function, ast.Attribute) and function.attr == name and isinstance(function.value, ast.Name)
            and function.value.id == "device_card_lease")


def card_lease_client_words_problems(texts):
    """texts：{CARD_LEASE_CLIENT_WORDS_LAUNCHER 与 CARD_LEASE_CLIENT_WORDS_CALLERS 各份: 原文}。→ 判出的问题（去重、照判出的次序），每条以那份文件起头。
    device_card_lease.py：UNKEYED_PROGRAM_WORDS 里登着这两个函数的 client；这两个函数各有起子进程的一处，每一处头一个参数是列表、头一个元素是登记的词；
    别的函数与模块级语句一处子进程都不起。调用方：经 device_card_lease.take_card、return_card 的调用点各至少一处；device_card_lease.client_of(…) 的结果
    只交给这两个函数当头一个参数（直接写在实参里，或赋给一个名字、所在函数里读这个名字的每一处都是这样）；不碰 device_card_lease.CLIENT_RELATIVE；
    文档串之外没有等于 "lease" 或带「lease take」「lease return」的字符串。弄坏开关 card-lease-client-words-other-launches-unchecked 下只看登记、
    这两个函数起的头一个词与调用点，别处起客户端的那几样不核。"""
    unchecked = "card-lease-client-words-other-launches-unchecked" in active_breaks()
    launcher, word, functions = CARD_LEASE_CLIENT_WORDS_LAUNCHER, CARD_LEASE_CLIENT_WORDS_WORD, CARD_LEASE_CLIENT_WORDS_FUNCTIONS
    problems = [f"{launcher}：UNKEYED_PROGRAM_WORDS 里没有 ({function}, {word}) 那一行" for function in functions
                if (launcher, function, word) not in UNKEYED_PROGRAM_WORDS]
    launching = collections.defaultdict(list)
    for statement in card_lease_client_words_parse(launcher, texts[launcher]).body:
        owner = statement.name if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)) else "<模块级语句>"
        launching[owner] += [call for call in ast.walk(statement) if isinstance(call, ast.Call) and card_lease_client_words_launches(call)]
    for function in functions:
        if not launching[function]:
            problems.append(f"{launcher}：{function} 里没有起子进程的那一处（登记的词落空）")
        for call in launching[function]:
            head = call.args[0] if call.args else None
            if not (isinstance(head, ast.List) and head.elts and isinstance(head.elts[0], ast.Name) and head.elts[0].id == word):
                problems.append(f"{launcher}：{function} 第 {call.lineno} 行起的头一个词不是登记的 {word}：{ast.unparse(call)[:120]}")
    others = sorted(owner for owner, calls in launching.items() if calls and owner not in functions)
    if others and not unchecked:
        problems.append(f"{launcher}：{others} 也起子进程（领卡客户端只许经 {list(functions)} 起）")
    for caller in CARD_LEASE_CLIENT_WORDS_CALLERS:
        tree = card_lease_client_words_parse(caller, texts[caller])
        calls = [node for node in ast.walk(tree) if isinstance(node, ast.Call)]
        for function in functions:
            if not any(card_lease_client_words_named(call, function) for call in calls):
                problems.append(f"{caller}：没有经 device_card_lease.{function} 的调用点")
        if unchecked:
            continue
        handed = {id(call.args[0]) for call in calls if call.args and any(card_lease_client_words_named(call, function) for function in functions)}
        assigned = set()
        for scope in ast.walk(tree):
            if not isinstance(scope, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            names = set()
            for node in ast.walk(scope):
                if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call) and card_lease_client_words_named(node.value, "client_of") \
                        and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
                    names.add(node.targets[0].id)
                    assigned.add(id(node.value))
            for node in ast.walk(scope):
                if isinstance(node, ast.Name) and node.id in names and isinstance(node.ctx, ast.Load) and id(node) not in handed:
                    problems.append(f"{caller}：第 {node.lineno} 行把 client_of 给的 {node.id} 交给了 take_card、return_card 之外的地方")
        for call in calls:
            if card_lease_client_words_named(call, "client_of") and id(call) not in handed and id(call) not in assigned:
                problems.append(f"{caller}：第 {call.lineno} 行 client_of 的结果没有交给 take_card、return_card")
        docstrings = {id(node.body[0].value) for node in ast.walk(tree)
                      if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and node.body and is_docstring(node.body[0])}
        for node in ast.walk(tree):
            if isinstance(node, ast.Attribute) and node.attr == "CLIENT_RELATIVE" and isinstance(node.value, ast.Name) and node.value.id == "device_card_lease":
                problems.append(f"{caller}：第 {node.lineno} 行自己拼领卡客户端的路径（device_card_lease.CLIENT_RELATIVE）")
            elif isinstance(node, ast.Constant) and isinstance(node.value, str) and id(node) not in docstrings \
                    and (node.value == "lease" or re.search(r"\blease (take|return)\b", node.value)):
                problems.append(f"{caller}：第 {node.lineno} 行文档串之外写着领卡客户端的子命令 {node.value[:60]!r}")
    return list(dict.fromkeys(problems))


def card_lease_client_words_reach_cell(_work):
    """本仓 device_card_lease.py、row_driver.py、shared_steps.py 的原文判一次（card_lease_client_words_problems），要一条问题都没有；
    CARD_LEASE_CLIENT_WORDS_BYPASSES 每个绕道样本各判一次，要判出点名那份文件的、本仓那一次没有的问题。
    弄坏开关 card-lease-client-words-other-launches-unchecked 下绕道判不出、这一格红。"""
    texts = {}
    for path in (CARD_LEASE_CLIENT_WORDS_LAUNCHER,) + CARD_LEASE_CLIENT_WORDS_CALLERS:
        with open(os.path.join(LIBRARY_ROOT, path), encoding="utf-8") as handle:
            texts[path] = handle.read()
    real = card_lease_client_words_problems(texts)
    lines, missed = [f"本仓：{real or '没有问题'}"], []
    for label, path, old, new in CARD_LEASE_CLIENT_WORDS_BYPASSES:
        if texts[path].count(old) != 1:
            raise AssertionError(f"绕道样本「{label}」的原文在 {path} 里命中 {texts[path].count(old)} 次（要恰好一次）：照新原文改 CARD_LEASE_CLIENT_WORDS_BYPASSES")
        found = [problem for problem in card_lease_client_words_problems({**texts, path: texts[path].replace(old, new)})
                 if problem.startswith(f"{path}：") and problem not in real]
        lines.append(f"{label}：{found or '判不出'}")
        if not found:
            missed.append(label)
    return not real and not missed, f"本仓的问题 {real}、判不出的绕道样本 {missed}（都要空）；{'；'.join(lines)}"


# library.py 照 memory_peaks_lib.py 的写法：if __name__ == "__main__" and globals().get("__file__"): 里按 sys.argv 分派到自证；
# fallback.py 是认不出、要照收的两种（带 else、__name__ 那一项不在最左）；twice.py 有两段 main-guard
MAIN_GUARD_FIXTURE = {
    "tool/library.py": ("import sys\n\nHOOKS = []\nLIMIT = 3\n\n\ndef work():\n    return LIMIT\n\n\ndef registered():\n    return 1\n\n\n"
                        "def self_check():\n    return 0\n\n\nHOOKS.append(registered)\n\n"
                        "if __name__ == \"__main__\" and globals().get(\"__file__\"):\n    if sys.argv[1:] == [\"--selftest\"]:\n"
                        "        sys.exit(self_check())\n    sys.exit(work())\n"),
    "tool/fallback.py": ("def flag():\n    return False\n\n\ndef otherwise():\n    return 1\n\n\ndef leftover():\n    return 2\n\n\n"
                         "def value():\n    return 0\n\n\nif __name__ == \"__main__\" and flag():\n    value()\nelse:\n    otherwise()\n\n"
                         "if flag() and __name__ == \"__main__\":\n    leftover()\n"),
    "tool/twice.py": ("import sys\n\n\ndef early():\n    return 1\n\n\ndef late():\n    return 2\n\n\n"
                      "if __name__ == \"__main__\" and len(sys.argv) > 1:\n    early()\n\nif __name__ == \"__main__\":\n    late()\n"),
    "tool/user.py": "import fallback\nimport library\n\n\ndef judge():\n    return library.work() + fallback.value()\n",
    "tool/launcher.py": ("import os\nimport subprocess\nimport sys\n\nHERE = os.path.dirname(os.path.abspath(__file__))\n\n\n"
                         "def launch(flag):\n    subprocess.run([sys.executable, os.path.join(HERE, \"twice.py\")], check=False)\n"
                         "    return subprocess.run([sys.executable, os.path.join(HERE, \"library.py\"), flag], check=False).returncode\n"),
}


def main_guard_closure_names(closure):
    return {(unit.file, unit.name) for unit in closure.order}


def main_guard_fixture_cell(entry, wanted, unwanted):
    """夹具上收 entry 的闭包：wanted 里的（文件, 限定名）都要在，unwanted 里的一个都不许在。"""
    def judge(work):
        base = os.path.join(work, "tree")
        write_tree(base, MAIN_GUARD_FIXTURE)
        names = main_guard_closure_names(closure_of(base, entry))
        missing, extra = sorted(set(wanted) - names), sorted(set(unwanted) & names)
        return not missing and not extra, f"{entry} 的闭包里缺 {missing}、多 {extra}（要：{sorted(wanted)} 都在，{sorted(unwanted)} 都不在）"
    return judge


def merged_judging_closure(root, entries):
    """几个判法入口从一个 ScriptClosure 一起收（main_guard_real_cell 用）：比逐个入口各收再合起来只多不少（类与属性的收窄跨入口共用），
    这里不含的单元逐个入口收也不含。"""
    parsed = [parse_entry(root, entry) for entry in entries]
    for entry, (language, _path, qualified) in zip(entries, parsed):
        if language != "python" or not qualified:
            raise AssertionError(f"判法入口 {entry} 不是 python 的 <文件>::<限定名>")
    under = [place for place in DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(root, place))]
    script = ScriptClosure(root, list(dict.fromkeys(under + [posixpath.dirname(path) or "." for _language, path, _qualified in parsed])))
    units = []
    for _language, path, qualified in parsed:
        module = script.python.module(path)
        found = [unit for unit in module.units if unit.name == qualified and not unit.data] if module is not None else []
        if len(found) != 1:
            raise AssertionError(f"{path} 里限定名 {qualified} 命中 {len(found)} 个单元（要恰好一个）")
        script.closure.refusals.update(module.refusals)
        units += found
    return script.run(units)


def judging_units_fingerprint(closure):
    """判法单元指纹的口径：闭包里的单元按（文件, 种类, 限定名, 起行）去重，（种类, 指纹）的多重集排好连起来取 sha256。"""
    members = {(unit.file, unit.kind, unit.name, unit.first_line): unit for unit in closure.order}
    return sha256_text("".join(sorted(f"{unit.kind}\t{unit.fingerprint}\n" for unit in members.values())))


def via_chain(closure, unit):
    """闭包里一个单元经哪几步收进来（closure.via 一路往回到入口；内联脚本的单元不在 via 里，走到它为止）：诊断用。"""
    steps, seen = [f"{unit.file}:{unit.name}"], set()
    while id(unit) in closure.via and id(unit) not in seen:
        seen.add(id(unit))
        via, rule = closure.via[id(unit)]
        if via is None:
            break
        steps.append(f"←{rule}— {via.file}:{via.name}")
        unit = via
    return " ".join(steps)


def body_statement_inserted(text, name):
    """text 里模块级 def name 的函数体头一句之前插一句赋值（这个单元的指纹变）→ 改过的原文；找不到这个 def、函数体与 def 在同一行、插完解不开交 None。"""
    for node in ast.parse(text).body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == name and node.body[0].lineno > node.lineno:
            rows = text.split("\n")
            row = rows[node.body[0].lineno - 1]
            rows.insert(node.body[0].lineno - 1, row[:len(row) - len(row.lstrip())] + "selftest_inserted = 1")
            edited = "\n".join(rows)
            try:
                ast.parse(edited)
            except SyntaxError:
                return None
            return edited
    return None


def main_guard_real_cell(work):
    """真仓拷贝上，以 memory_peaks_lib.py 的 command_key 为判法入口收闭包：它的 run_self_test、self_test_cells 不许在（自证只在库末尾 main-guard 里，
    入口在库里时库的 module-body 照收、main-guard 不收）；再改拷贝里 self_test_cells 的函数体，判法单元指纹不变。"""
    tree = real_tree(work)
    entries = [MAIN_GUARD_REAL_ENTRY]
    closure = merged_judging_closure(tree, entries)
    notes = [f"入口 {MAIN_GUARD_REAL_ENTRY} 收 {len(closure.order)} 个单元"]
    unwanted = [unit for unit in closure.order if unit.file == MAIN_GUARD_REAL_LIBRARY and unit.name in MAIN_GUARD_REAL_SELFTEST_UNITS]
    for unit in sorted(unwanted, key=Unit.sort_key):
        notes.append(f"不该在的 {unit.file}:{unit.name} 的来路 {via_chain(closure, unit)}")
    path = os.path.join(tree, MAIN_GUARD_REAL_LIBRARY)
    with open(path, encoding="utf-8") as handle:
        edited = body_statement_inserted(handle.read(), MAIN_GUARD_REAL_EDITED)
    if edited is None:
        return False, "；".join(notes + [f"{MAIN_GUARD_REAL_LIBRARY} 里找不到能插一句的模块级 def {MAIN_GUARD_REAL_EDITED}：照新原文改 MAIN_GUARD_REAL_EDITED"])
    before = judging_units_fingerprint(closure)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(edited)
    after = judging_units_fingerprint(merged_judging_closure(tree, entries))
    notes.append(f"改拷贝里 {MAIN_GUARD_REAL_LIBRARY} 的 {MAIN_GUARD_REAL_EDITED}：判法单元指纹 {before}→{after}")
    return not unwanted and before == after, "；".join(notes)


def real_entries_cell(work):
    tree = real_tree(work)
    entries = registered_judging_entries(tree)
    formal = real_entries(tree)
    unknown = []
    for entry in entries:
        language, path, qualified = parse_entry(tree, entry)
        if not os.path.isfile(os.path.join(tree, path)):
            unknown.append(entry)
    return not unknown and len(entries) > 2, f"登记现取 {len(entries)} 个判法入口（形式化入口 {formal}），文件不在的 {unknown}"



def judging_entry_refusals_of(job):
    """(入口, 拒算行)：一个入口的闭包里算进判法单元指纹的拒算——去掉 lib_verdict_library.py 的 NOT_JUDGING_FILES 那几份
    （照 judging_unit_key 的口径；弄坏开关 judging-entry-refusals-ignored 下一条都不报）。"""
    root, entry, not_judging = job
    options = Options()
    options.excluded_files = not_judging
    closure = closure_of(root, entry, options)
    if "judging-entry-refusals-ignored" in active_breaks():
        return entry, []
    return entry, sorted(refusal.text() for refusal in closure.refusals if refusal.file not in not_judging)


def judging_entry_refusals(root, entries, not_judging_by_entry):
    """{入口: 拒算行}，只列有拒算的；按每台默认重活线程数并行（judging_entry_refusal_workers；一个入口要几秒到十几秒）。
    fork 之前父进程解析好全部脚本、冻进永久代（prepare_shared_script_index、gc.freeze）：子进程共用解析过的模块，不各解析一份、回收也不逐页抄。"""
    import concurrent.futures
    import gc
    import multiprocessing
    workers = judging_entry_refusal_workers(os.environ)
    jobs = [(root, entry, not_judging_by_entry.get(entry, ())) for entry in entries]
    _prepared, key = prepare_shared_script_index(root)
    gc.collect()
    gc.freeze()
    try:
        with concurrent.futures.ProcessPoolExecutor(max_workers=max(1, min(workers, len(jobs))), mp_context=multiprocessing.get_context("fork")) as pool:
            results = list(pool.map(judging_entry_refusals_of, jobs))
    finally:
        gc.unfreeze()
        SHARED_SCRIPT_INDICES.pop(key, None)
    return {entry: refusals for entry, refusals in results if refusals}


def real_judging_closures_cell(work):
    tree = real_tree(work)
    entries = registered_judging_entries(tree)
    not_judging = tuple(module_literal(tree, "scripts/crash/lib_verdict_library.py", "NOT_JUDGING_FILES"))
    real = judging_entry_refusals(tree, entries, {entry: not_judging for entry in entries})
    fixture = os.path.join(work, "fixture")
    write_tree(fixture, JUDGING_REFUSAL_FIXTURE)
    probe = judging_entry_refusals(fixture, ["tool/judge.py::judge"], {})
    probe_forms = {line.split("\t")[1] for lines in probe.values() for line in lines}
    return not real and probe_forms == {"globals-lookup"}, (f"真仓 {len(entries)} 个判法入口里拒算的 {len(real)} 个：{real}（照拒算那一行改成算得出键的写法：拒算的判定不入库、下一趟照判）；"
                                      f"夹具里 globals() 查找判出 {probe}（要判出一条）")


def selftest_modules_outside_cell(work):
    """SELFTEST_MODULES_OUTSIDE_CELL：真仓拷一份，收按路径载入这份库的那几个入口（SELFTEST_MODULES_OUTSIDE_ENTRIES：lib_mutation_units.py 的 UnitKeys 方法，
    经 function_units() 载入 lib_function_units.py；不去掉 NOT_JUDGING_FILES）的闭包：lib_function_units_selftest*.py 的单元一个都不在、一处拒算都没有。
    弄坏开关 selftest-modules-loaded-at-import：在拷贝上给这份库末尾加一句模块级的 from lib_function_units_selftest_run import selftest（import 时就载入自证模块），这一格必红。"""
    tree = real_tree(work)
    if "selftest-modules-loaded-at-import" in active_breaks():
        with open(os.path.join(tree, "scripts", "gate", "lib_function_units.py"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_MODULES_IMPORT_LINE)
    entries = list(SELFTEST_MODULES_OUTSIDE_ENTRIES)
    problems = []
    for entry in entries:
        closure = closure_of(tree, entry)
        inside = sorted(f"{unit.file}:{unit.name}" for unit in closure.order if unit.file.startswith("scripts/gate/lib_function_units_selftest"))
        refusals = [refusal.text() for refusal in closure.refusal_list()]
        if inside:
            problems.append(f"{entry} 的闭包里有自证模块的单元 {len(inside)} 个（头几个 {inside[:4]}）")
        if refusals:
            problems.append(f"{entry} 的闭包拒算 {len(refusals)} 处（头几处 {refusals[:3]}）")
    return not problems, ("；".join(problems) + "（自证模块只在 --selftest 那一支载入：照文件头「自证模块」改）") if problems else \
        f"{len(entries)} 个入口的闭包里没有自证模块的单元、没有拒算"


E171_RESEARCH_DIRECTORY = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"


def e171_closure_cell(work):
    """E171_CLOSURE_CELL：真仓拷一份，两份入口脚本各收一遍闭包（去掉 NOT_JUDGING_FILES，同复跑行的口径）：拒算为空、该不在的单元都不在、名单核对不连 reuse_library.py。
    弄坏开关各在拷贝上把一处切掉的边接回去，对应的那一项必红：e171-selftest-table-at-module-level（reuse_library.py 加一张模块级的自证格表）、
    e171-admission-selftest-at-module-level（admission.py 加一张模块级的表点名 run_selftest）、e171-list-check-imports-reuse-library（list_check.py 模块级 import reuse_library）。"""
    tree = real_tree(work)
    breaks = active_breaks()
    reuse_library = f"{E171_RESEARCH_DIRECTORY}/reuse_library.py"
    if "e171-selftest-table-at-module-level" in breaks:
        with open(os.path.join(tree, reuse_library), "a", encoding="utf-8") as handle:
            handle.write("\n\nSELFTEST_CELLS_PROBE = {\"probe\": cell_rp1_layers_end_to_end}\n")
    if "e171-admission-selftest-at-module-level" in breaks:
        with open(os.path.join(tree, "scripts/gate/admission.py"), "a", encoding="utf-8") as handle:
            handle.write("\n\nSELFTEST_PROBE = {\"probe\": run_selftest}\n")
    if "e171-list-check-imports-reuse-library" in breaks:
        edit_file(tree, f"{E171_RESEARCH_DIRECTORY}/list_check.py", "from result_lines import ResultLines, field\n",
                  "from result_lines import ResultLines, field\nimport reuse_library  # noqa: F401\n")
    problems = []
    for script in E171_ENTRY_SCRIPTS:
        options = Options()
        options.excluded_files = NOT_JUDGING_FILES
        closure = closure_of(tree, f"{E171_RESEARCH_DIRECTORY}/{script}", options)
        refusals = sorted(refusal.text() for refusal in closure.refusals if refusal.file not in NOT_JUDGING_FILES)
        members = {(unit.file, unit.name) for unit in closure.order}
        problems += [f"{script} 拒算 {line}" for line in refusals]
        problems += [f"{script} 的闭包里有 {file}:{name}" for file, name in E171_CLOSURE_ABSENT if (f"{E171_RESEARCH_DIRECTORY}/{file}", name) in members]
        problems += [f"{script} 的闭包里有 admission.py:{name}" for name in E171_ADMISSION_ABSENT if ("scripts/gate/admission.py", name) in members]
        if script == "list_check.py":
            reuse_units = sorted(name for file, name in members if file == reuse_library)
            problems += [f"list_check.py 的闭包连上了 reuse_library.py（{len(reuse_units)} 个单元，头几个 {reuse_units[:3]}）"] if reuse_units else []
    return not problems, f"{len(problems)} 处不对：{problems[:12]}（拒算的照那一行改成算得出键的写法或登记 UNKEYED_PROGRAM_WORDS；不该在的单元沿 closure --explain 的 via 找进来的边）"



def cross_run_cache_layout(work, tree, label):
    """另建空目录 <work>/<label> 当 TMPDIR，摘掉 GATE_CROSS_RUN_TMPDIR 与几份缓存自己的改放处变量，起子进程跑 tree 里这份库的命令行 cross-run-caches
    → (过没过, 说明, {名字: 现取的路径})：子进程退 0、它的 tempfile.gettempdir() 是那个目录、登记的每一份都算得出、没有一份落在它之下才算过。"""
    temporary = os.path.join(work, label)
    os.makedirs(temporary)
    overrides = {"GATE_CROSS_RUN_TMPDIR", RUST_LEX_CACHE_DIRECTORY_VARIABLE}
    environment = {name: value for name, value in os.environ.items() if name not in overrides}
    environment["TMPDIR"] = temporary
    work_root = work_root_library()
    # 拷贝里没有 multi-host.env：数据根没经环境给时，把本仓多机配置的位置带过去（test-environment 那一份缓存放数据根下）
    environment.setdefault(work_root.MULTI_HOST_CONFIGURATION_VARIABLE, work_root.multi_host_configuration_path(os.environ))
    try:
        completed = subprocess.run([sys.executable, "-B", os.path.join(tree, "scripts", "gate", "lib_function_units.py"), "cross-run-caches"],
                                   capture_output=True, text=True, errors="surrogateescape", env=environment, stdin=subprocess.DEVNULL,
                                   timeout=CROSS_RUN_CACHE_PROBE_SECONDS)
    except subprocess.TimeoutExpired:
        return False, f"{label}：子进程 {CROSS_RUN_CACHE_PROBE_SECONDS} 秒没退（拷贝里的这份库或它载入的 lib_work_root.py 卡住）", {}
    output = completed.stdout.splitlines()
    temporary_root = os.path.realpath(temporary)
    seen_tempdirs = [line.split("\t", 1)[1] for line in output if line.startswith("tempdir\t")]
    rows = [fields for fields in (line.split("\t", 3) for line in output if line.startswith(("cache\t", "refused-cache\t"))) if len(fields) == 4]

    def under_temporary(path):
        resolved = os.path.realpath(path)
        return resolved == temporary_root or resolved.startswith(temporary_root + os.sep)

    under = [f"{name}（{script}）→ {path}" for kind, name, script, path in rows if kind == "cache" and under_temporary(path)]
    refused = [f"{name}（{script}）：{reason}" for kind, name, script, reason in rows if kind == "refused-cache"]
    registered = [name for name, _script, _locate in cross_run_cache_registry()]
    tempdir_held = len(seen_tempdirs) == 1 and os.path.realpath(seen_tempdirs[0]) == temporary_root
    ok = completed.returncode == 0 and tempdir_held and [fields[1] for fields in rows] == registered and not under and not refused
    listed = "；".join(f"{name} → {path}" for _kind, name, _script, path in rows)
    return ok, (f"{label}：子进程退 {completed.returncode}；它的 tempfile.gettempdir() {seen_tempdirs}（要恰好一行 {temporary_root}）；"
                f"现取 {len(rows)} 份（登记 {len(registered)} 份）：{listed}；落在 TMPDIR 之下的 {under}；算不出的 {refused}；"
                f"stderr 末尾：{completed.stderr.strip()[-300:]}"), {name: path for kind, name, _script, path in rows if kind == "cache"}


def cross_run_cache_cell(work):
    """两趟 cross_run_cache_layout：本仓 scripts/ 与 .claude/ 拷进临时目录（real_tree）、建空的 .git 起一趟；git 造一个主仓与它的 worktree
    （worktree 里 .git 是文件），拷进 worktree 再起一趟。两趟都要过；头一趟落在 <拷贝>/.git 底下的那几份（至少一份），第二趟要落在主仓 .git
    （git common-dir）底下同一相对路径。弄坏开关 cross-run-cache-under-tmpdir 下 cross_run_cache_directory 放回 TMPDIR，两趟都有落进去的；
    cross-run-cache-git-directory-only 下只认 .git 是目录，worktree 那一趟本库两份落进那一趟的 TMPDIR；都判错。"""
    tree = real_tree(work)
    os.makedirs(os.path.join(tree, ".git"))
    plain_ok, plain_detail, plain_paths = cross_run_cache_layout(work, tree, "tmpdir")
    main, linked = os.path.join(work, "worktree-main"), os.path.join(work, "worktree-linked")
    problem = selftest_git_steps(work, (["init", "-q", main], ["-C", main, "commit", "-q", "--allow-empty", "-m", "base"],
                                        ["-C", main, "worktree", "add", "-q", "--detach", linked]))
    if problem is not None:
        return False, f"{plain_detail}；造 worktree：{problem}"
    if not os.path.isfile(os.path.join(linked, ".git")):
        return False, f"{plain_detail}；git worktree add 造出的 {linked}/.git 不是文件，样本没造对"
    real_tree(work, linked)
    linked_ok, linked_detail, linked_paths = cross_run_cache_layout(work, linked, "worktree-tmpdir")
    plain_git, main_git = os.path.realpath(os.path.join(tree, ".git")), os.path.realpath(os.path.join(main, ".git"))
    in_git = {name: os.path.relpath(os.path.realpath(path), plain_git) for name, path in plain_paths.items()
              if os.path.realpath(path).startswith(plain_git + os.sep)}
    unshared = sorted(f"{name} → {linked_paths.get(name)}（要 {os.path.join(main_git, relative)}）" for name, relative in in_git.items()
                      if name not in linked_paths or os.path.realpath(linked_paths[name]) != os.path.join(main_git, relative))
    ok = plain_ok and linked_ok and bool(in_git) and not unshared
    return ok, (f"{plain_detail}；{linked_detail}；.git 是目录那一趟落在 .git 底下的 {sorted(in_git)}（要至少一份）；"
                f"worktree 那一趟没落到主仓 common-dir 同一处的 {unshared}")


def cross_run_cache_mentions(root):
    """root 下 CROSS_RUN_CACHE_SCANNED_DIRECTORIES 里（跳过 CROSS_RUN_CACHE_SKIPPED_DIRECTORIES）提到 GATE_CROSS_RUN_TMPDIR 的 .py / .sh / .bash，相对 root、排好序。
    弄坏开关 cross-run-cache-mentions-unscanned：交空表。"""
    if "cross-run-cache-mentions-unscanned" in active_breaks():
        return []
    found = []
    for top in CROSS_RUN_CACHE_SCANNED_DIRECTORIES:
        for directory, subdirectories, names in os.walk(os.path.join(root, top)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in CROSS_RUN_CACHE_SKIPPED_DIRECTORIES)
            for name in sorted(names):
                if not name.endswith(CROSS_RUN_CACHE_SCANNED_SUFFIXES):
                    continue
                path = os.path.join(directory, name)
                try:
                    with open(path, "rb") as handle:
                        mentioned = CROSS_RUN_CACHE_MENTION.encode("ascii") in handle.read()
                except OSError:
                    continue
                if mentioned:
                    found.append(os.path.relpath(path, root))
    return sorted(found)


def cross_run_cache_scan_cell(work):
    """本仓里提到 GATE_CROSS_RUN_TMPDIR 的脚本都是登记表里的、或在 CROSS_RUN_CACHE_NOT_CACHES 里写明为什么不登记；不登记表里的每一份都在、都还提到它；
    夹具里一份没登记的（scripts/sample/keeps-cache.sh）判得出来。弄坏开关 cross-run-cache-mentions-unscanned 下一份都扫不出，这一格判错。"""
    registered = {script for _name, script, _locate in cross_run_cache_registry()}
    mentioned = cross_run_cache_mentions(LIBRARY_ROOT)
    unaccounted = [path for path in mentioned if path not in registered and path not in CROSS_RUN_CACHE_NOT_CACHES]
    stale = [path for path in sorted(CROSS_RUN_CACHE_NOT_CACHES) if path not in mentioned]
    fixture = os.path.join(work, "fixture")
    write_tree(fixture, CROSS_RUN_CACHE_SCAN_FIXTURE)
    probe = cross_run_cache_mentions(fixture)
    ok = not unaccounted and not stale and probe == ["scripts/sample/keeps-cache.sh"]
    return ok, (f"本仓提到 {CROSS_RUN_CACHE_MENTION} 的 {len(mentioned)} 份：{mentioned}；没登记也没写明为什么的 {unaccounted}"
                f"（往 cross_run_cache_registry 加一行，或不是缓存的写进 CROSS_RUN_CACHE_NOT_CACHES）；不登记表里指不到或不再提到它的 {stale}（删掉那一行）；"
                f"夹具里扫出 {probe}（要恰好 scripts/sample/keeps-cache.sh）")



def entry_unit_key_cell(work):
    """每一对入口先核夹具造对了（两个闭包的单元多重集相同），再核两把键都算得出、不相同。弄坏开关 key-entry-units-dropped 下键里不出 entry 行，两把键相同，这一格判错。"""
    base = os.path.join(work, "tree")
    write_tree(base, ENTRY_UNIT_KEY_FIXTURE)
    problems, lines = [], []
    for language, entries in ENTRY_UNIT_KEY_PAIRS:
        closures = [closure_of(base, entry) for entry in entries]
        multisets = [sorted(f"{unit.kind}\t{unit.fingerprint}" for unit in closure.order) for closure in closures]
        keys = [closure_key(closure)[0] for closure in closures]
        lines.append(f"{language}：两个闭包{'相同' if multisets[0] == multisets[1] else '不同'}（各 {len(multisets[0])}、{len(multisets[1])} 个单元），键 {keys}")
        if multisets[0] != multisets[1]:
            problems.append(f"{language} 两个入口的闭包不同：夹具没造出互相收进对方闭包的样子，照 {entries} 的闭包改夹具")
        elif None in keys:
            problems.append(f"{language} 两个入口的键算不出")
        elif keys[0] == keys[1]:
            problems.append(f"{language} 两个入口的键相同：键里没有入口单元自己（entry_key_lines）")
    return not problems, "；".join(problems + lines)



def entry_lines_closures(work):
    """写出 ENTRY_LINES_FIXTURE → (documented 挂的文档测试单元（按 Unit.sort_key）, {"impl-method" / "doctest": (入口单元, 闭包)})；
    文档测试入口取头一个文档测试。夹具里 documented 挂的不是两个就抛 AssertionError。"""
    base = os.path.join(work, "tree")
    write_tree(base, ENTRY_LINES_FIXTURE)
    workspace = RustWorkspace(base)
    doctests = sorted((unit for unit in workspace.file(ENTRY_LINES_FILE).units if unit.kind == "doctest"), key=Unit.sort_key)
    if len(doctests) != 2:
        raise AssertionError(f"夹具里 documented 要挂两个文档测试，实际 {[unit.name for unit in doctests]}：照 ENTRY_LINES_FIXTURE 改")
    closures = {}
    for label, qualified in (("impl-method", ENTRY_LINES_METHOD), ("doctest", doctests[0].name)):
        unit, target_id = workspace.find_entry(ENTRY_LINES_FILE, qualified, None)
        closures[label] = (unit, rust_closure_of(workspace, unit, target_id))
    return doctests, closures


def entry_lines_shape_cell(work):
    """两个入口的 unit_graph_key_lines：entry 行在 entry-target-kind、cfg、unit、data 行之后、toolchain 行之前，排好序、不重；
    impl 方法入口一行（fn 连 impl 头）；文档测试入口两行（各是一个 doctest），Closure.entry_units 恰好是那两个文档测试、闭包里两个都在。"""
    doctests, closures = entry_lines_closures(work)
    problems, notes = [], []
    want_kinds = {"impl-method": [["fn", "impl"]], "doctest": [["doctest"], ["doctest"]]}
    for label, (_unit, closure) in closures.items():
        lines = unit_graph_key_lines(closure, [ENTRY_LINES_HOST_LINE])
        entry_positions = [index for index, line in enumerate(lines) if line.startswith("entry\t")]
        entry_rows = [lines[index] for index in entry_positions]
        before = [index for index, line in enumerate(lines) if line.startswith(("entry-target-kind\t", "cfg\t", "unit\t", "data\t"))]
        after = [index for index, line in enumerate(lines) if line.startswith("toolchain\t")]
        kinds = [row.split("\t")[1::2] for row in entry_rows]
        notes.append(f"{label}：entry 行的种类链 {kinds}，在第 {entry_positions} 行（共 {len(lines)} 行）")
        if kinds != want_kinds[label]:
            problems.append(f"{label} 的 entry 行种类链要 {want_kinds[label]}")
        if entry_positions and not (max(before) < min(entry_positions) and max(entry_positions) < min(after)):
            problems.append(f"{label} 的 entry 行不在数据行之后、工具链行之前")
        if entry_rows != sorted(set(entry_rows)):
            problems.append(f"{label} 的 entry 行没排序去重")
    doctest_closure = closures["doctest"][1]
    recorded = sorted(unit.name for unit in doctest_closure.entry_units)
    collected = [unit.name for unit in doctests if id(unit) not in doctest_closure.members]
    if recorded != sorted(unit.name for unit in doctests) or collected:
        problems.append(f"文档测试入口记的入口 {recorded}，要 documented 挂的全部 {[unit.name for unit in doctests]}；闭包里缺 {collected}")
    return not problems, "；".join(problems + notes)


def entry_lines_rust_keys_cell(work):
    """两个入口的 unit_graph_key 与 ENTRY_LINES_RUST_KEYS 录下的 Rust 键逐字相同；还没录的判红，说明里打出 python 这一边的键。"""
    _doctests, closures = entry_lines_closures(work)
    problems, notes = [], []
    for label, (_unit, closure) in closures.items():
        key, refusals = unit_graph_key(closure, [ENTRY_LINES_HOST_LINE])
        recorded = ENTRY_LINES_RUST_KEYS.get(label)
        notes.append(f"{label}：python {key or '拒算：' + '；'.join(refusal.text() for refusal in refusals[:2])}，Rust 录下的 {recorded}")
        if recorded is None:
            problems.append(f"{label} 的 Rust 键还没录（照 ENTRY_LINES_RUST_KEYS 上面的注释用 unit-graph closure 录）")
        elif key != recorded:
            problems.append(f"{label} 两份的键不同")
    return not problems, "；".join(problems + notes)



def main_guard_else_cell(work):
    """import either 的 judge：imported_work（else 那一支调的）在、main_work（if 那一支调的）与 <main-guard> 不在；
    起 either.py 的 run：main_work 在（if 那一支照旧是 main-guard、当程序起时连）。"""
    base = os.path.join(work, "tree")
    write_tree(base, MAIN_GUARD_ELSE_FIXTURE)
    imported = main_guard_closure_names(closure_of(base, "tool/importer.py::judge"))
    launched = main_guard_closure_names(closure_of(base, "tool/runner.py::run"))
    missing = sorted({("tool/either.py", "imported_work"), ("tool/either.py", "value")} - imported)
    extra = sorted({("tool/either.py", "main_work"), ("tool/either.py", "<main-guard>")} & imported)
    unlaunched = sorted({("tool/either.py", "main_work"), ("tool/either.py", "<main-guard>")} - launched)
    return not missing and not extra and not unlaunched, (f"被 import 时闭包里缺 {missing}、多 {extra}；被当程序起时缺 {unlaunched}")



def judging_entry_workers_cell(work):
    """自证造一份登记文件（头一行注释，再一行「7  # 理由」）：没设 SINGLEFS_THREAD_CAP 取到 7，设成 3 取到 3；真仓的登记照样取得到。"""
    registered = 7
    path = os.path.join(work, "heavy-thread-default")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(f"# 自证造的登记：头一个词是 #\n{registered}  # 自证样本\n")
    unset = judging_entry_refusal_workers({}, registration_file=path)
    capped = judging_entry_refusal_workers({"SINGLEFS_THREAD_CAP": "3"}, registration_file=path)
    real = judging_entry_refusal_workers({})
    return unset == registered and capped == 3 and real >= 1, (f"没设 SINGLEFS_THREAD_CAP 取到 {unset}（要登记的 {registered}）、"
                                                              f"设成 3 取到 {capped}、真仓的登记取到 {real}")



def assume_specification_forms_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, ASSUME_SPECIFICATION_FIXTURE)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    units = [unit for unit in RustWorkspace(base).files[ASSUME_SPECIFICATION_FILE].units]
    got = [(unit.short, unit.first_line, unit.last_line, sorted(rust_declared_names(unit)), unit.public)
           for unit in units if unit.kind == "assume_specification"]
    want = [("assume_specification@14", 13, 14, ["LIMIT_IN_BYTES"], True), ("assume_specification@16", 16, 17, ["width_of"], False),
            ("assume_specification@19", 19, 19, ["len"], True)]
    dumped = [line.split("\t")[0] for line in unit_lines if line.startswith("assume_specification\t")]
    spec_function = [(unit.first_line, unit.last_line) for unit in units if unit.kind == "fn" and unit.short == "spec_limit"]
    ok = not refusal_lines and got == want and len(dumped) == len(want) and spec_function == [(21, 23)]
    return ok, f"拒算 {refusal_lines}；assume_specification 单元 {got}（要 {want}）；dump 里 {len(dumped)} 行；spec_limit 起止 {spec_function}"



def counted_value_lines(base, entry):
    """入口闭包里 crates/fcore 的 counted_value 是哪一份（起行）与闭包的 cfg 行里有没有 fcore 的 counters。"""
    closure = closure_of(base, entry)
    lines = sorted(unit.first_line for unit in closure.order if unit.file == "crates/fcore/src/lib.rs" and unit.short == "counted_value")
    return lines, 'feature="counters"@fcore' in closure.cfg_lines


def feature_sections_entry_crate_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, FEATURE_SECTIONS_FIXTURE)
    binary = counted_value_lines(base, "crates/fbin/src/main.rs::main")
    test = counted_value_lines(base, "crates/fbin/tests/counting.rs::counting_sees_the_counters")
    ok = binary == ([FEATURE_SECTIONS_UNCOUNTED_LINE], False) and test == ([FEATURE_SECTIONS_COUNTED_LINE], True)
    return ok, f"bin 入口 {binary}（要 [{FEATURE_SECTIONS_UNCOUNTED_LINE}] 不开）、集成测试入口 {test}（要 [{FEATURE_SECTIONS_COUNTED_LINE}] 开着）"


def feature_sections_linked_crate_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, FEATURE_SECTIONS_FIXTURE)
    via = counted_value_lines(base, "crates/fvia/tests/via.rs::via_the_helper")
    return via == ([FEATURE_SECTIONS_UNCOUNTED_LINE], False), f"经 fhelper 的集成测试入口 {via}（要 [{FEATURE_SECTIONS_UNCOUNTED_LINE}] 不开）"



def verus_semicolon_cell(work):
    base = os.path.join(work, "tree")
    write_tree(base, VERUS_SEMICOLON_FIXTURE)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    units = RustWorkspace(base).files[VERUS_SEMICOLON_FILE].units
    got = sorted((unit.kind, unit.short, unit.first_line, unit.last_line, unit.fingerprint) for unit in units if unit.kind in ("const", "fn"))
    want = sorted([("const", "ORIGIN", 6, 6, sha256_text("\x1f".join(rust_lex(VERUS_SEMICOLON_CONSTANT)[1]))),
                   ("const", "UNIT", 8, 8, sha256_text("\x1f".join(rust_lex(VERUS_SEMICOLON_ASSOCIATED)[1]))),
                   ("fn", "total", 10, 12, sha256_text("\x1f".join(rust_lex("pub fn total(pair: Pair) -> u64 {\n    pair.left + pair.right\n}")[1])))])
    return not refusal_lines and got == want, f"拒算 {refusal_lines}；const 与 fn 单元 {[item[:4] + (item[4][:12],) for item in got]}（要 {[item[:4] + (item[4][:12],) for item in want]}）"


def unit_graph_parity_cells():
    """与 crates/singlefs-unit-graph 对齐的 assume_specification 切分、dev 依赖特性与 verus! 里 } 之后的 ; 那几格（BREAK_CELLS 里各配弄坏开关）。"""
    return [
        (VERUS_SEMICOLON_CELL, verus_semicolon_cell),
        (ASSUME_SPECIFICATION_FORMS_CELL, assume_specification_forms_cell),
        (ASSUME_SPECIFICATION_KEY_CELL, compare_cell(ASSUME_SPECIFICATION_FIXTURE, f"{ASSUME_SPECIFICATION_FILE}::judge_limit", ASSUME_SPECIFICATION_FILE,
                                                     "returns 32u64;", "returns 33u64;", "changed")),
        (FEATURE_SECTIONS_ENTRY_CRATE_CELL, feature_sections_entry_crate_cell),
        (FEATURE_SECTIONS_LINKED_CRATE_CELL, feature_sections_linked_crate_cell),
    ]


def import_time_cells():
    return [
        (IMPORT_TIME_EXIT_CELL, import_time_exit_cell),
        (IMPORT_TIME_CLASS_CELL, compare_cell(IMPORT_TIME_FIXTURE, IMPORT_TIME_ENTRY, "tool/helper.py", "        self.value = 1\n", "        self.value = 2\n", "same")),
        (REAL_ENTRIES_CELL, real_entries_cell),
        (REAL_JUDGING_CLOSURES_CELL, real_judging_closures_cell),
        (E171_CLOSURE_CELL, e171_closure_cell),
    ] + [(name, real_import_time_cell(name)) for name, _path, _old, _new in REAL_IMPORT_TIME_EDITS] + [
        (EXCLUDED_EDGE_PYTHON_CELL, excluded_edge_cell(EXCLUDED_EDGE_PYTHON_ENTRY, "tool/hint.py", "tool/kept.py")),
        (EXCLUDED_EDGE_SHELL_CELL, excluded_edge_cell(EXCLUDED_EDGE_SHELL_ENTRY, "tool/hint.sh", "tool/kept.sh")),
        (EXCLUDED_EDGE_EXIT_CELL, excluded_edge_exit_cell),
        (ROOT_IGNORED_SCRIPTS_CELL, root_ignored_scripts_cell),
        (ADMISSION_EDGE_ANOMALY_CELL, admission_edge_anomaly_cell),
        (ADMISSION_EDGE_STRIP_CELL, admission_edge_strip_cell),
        (ADMISSION_EDGE_REACH_CELL, admission_edge_reach_cell),
        (CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL, card_lease_client_words_anomaly_cell),
        (CARD_LEASE_CLIENT_WORDS_STRIP_CELL, card_lease_client_words_strip_cell),
        (CARD_LEASE_CLIENT_WORDS_REACH_CELL, card_lease_client_words_reach_cell),
        (MAIN_GUARD_IMPORT_CELL, main_guard_fixture_cell(
            "tool/user.py::judge",
            (("tool/library.py", "work"), ("tool/library.py", "LIMIT"), ("tool/library.py", "registered"),
             ("tool/fallback.py", "otherwise"), ("tool/fallback.py", "leftover")),
            (("tool/library.py", "self_check"), ("tool/library.py", "<main-guard>")))),
        (MAIN_GUARD_PROGRAM_CELL, main_guard_fixture_cell(
            "tool/launcher.py::launch",
            (("tool/library.py", "self_check"), ("tool/library.py", "work"), ("tool/twice.py", "early"), ("tool/twice.py", "late")), ())),
        (MAIN_GUARD_REAL_CELL, main_guard_real_cell),
        (MAIN_GUARD_ELSE_CELL, main_guard_else_cell),
        (JUDGING_ENTRY_WORKERS_CELL, judging_entry_workers_cell),
    ]


# 产品行为视图（文件头「判定种类」）那几格：Rust 那一份的替身是「照 crates/singlefs-unit-graph 的剥法手工剥过的拷贝（剥掉的换成空格、换行留着）
# 照原样切出的 dump」，不起 cargo、不编；起 unit-graph 的那一条路由一个只认 dump --judgment-kind product-behavior 的假程序走一趟
PRODUCT_VIEW_FILE = "crates/fverus/src/lib.rs"
PRODUCT_VIEW_ENTRY = f"{PRODUCT_VIEW_FILE}::caller"


def product_view_files(edit=None):
    """→ (原样那一份的文件表, 剥过的那一份)：edit 是 (旧串, 新串)，两份照改（落在被剥那几段里的，剥过的那一份照样剥掉）。"""
    source, pieces = PRODUCT_VIEW_SOURCE, list(PRODUCT_VIEW_GHOST_PIECES)
    if edit is not None:
        old, new = edit
        if source.count(old) != 1:
            raise AssertionError(f"产品行为视图样本里「{old.strip()}」命中 {source.count(old)} 次（要恰好一次）")
        source = source.replace(old, new)
        pieces = [piece.replace(old, new) for piece in pieces]
    blanked = source
    for piece in pieces:
        if blanked.count(piece) != 1:
            raise AssertionError(f"产品行为视图样本里要剥的「{piece[:40]}」命中 {blanked.count(piece)} 次（要恰好一次）")
        blanked = blanked.replace(piece, "".join(character if character == "\n" else " " for character in piece))
    return dict(PRODUCT_VIEW_FIXTURE, **{PRODUCT_VIEW_FILE: source}), dict(PRODUCT_VIEW_FIXTURE, **{PRODUCT_VIEW_FILE: blanked})


def product_view_dump_of(work, blanked_files):
    """Rust 那一份的替身：剥过的那一份照原样切出的 dump 行（单元行与拒算行）。"""
    base = os.path.join(work, "blanked")
    shutil.rmtree(base, ignore_errors=True)
    write_tree(base, blanked_files)
    unit_lines, refusal_lines, _counts = dump_lines(base, "rust")
    return unit_lines + refusal_lines


def product_view_key(work, files, dump, kind=PRODUCT_BEHAVIOR, entry=PRODUCT_VIEW_ENTRY):
    """→ (键或 None, 拒算形态的集合)：dump 是交给库的 Rust 那一份（None 是要用时起 unit-graph）。"""
    base = os.path.join(work, "tree")
    shutil.rmtree(base, ignore_errors=True)
    write_tree(base, files)
    options = Options()
    options.judgment_kind, options.product_dump = kind, dump
    return fixture_key(base, entry, options)


def product_view_lemma_cell(work):
    original, blanked = product_view_files()
    edited, edited_blanked = product_view_files(("    assert(doubled(x) == x * 2);\n", "    assert(true);\n    assert(doubled(x) == x * 2);\n"))
    before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked))
    after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked))
    formal_before, _ = product_view_key(work, original, None, FORMAL_VERIFICATION)
    formal_after, _ = product_view_key(work, edited, None, FORMAL_VERIFICATION)
    ok = before is not None and before == after and formal_before is not None and formal_after is not None and formal_before != formal_after
    return ok, (f"产品行为键改前 {before}、改后 {after}（拒算 {sorted(before_forms | after_forms)}）；形式化键改前 {formal_before}、改后 {formal_after}")


def product_view_compare_cell(edit, want):
    def judge(work):
        original, blanked = product_view_files()
        edited, edited_blanked = product_view_files(edit)
        before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked))
        after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked))
        if before is None or after is None:
            return False, f"拒算了：改前 {sorted(before_forms)}，改后 {sorted(after_forms)}"
        got = "changed" if before != after else "same"
        return got == want, f"期望 {want}，实际 {got}"
    return judge


def product_view_refused_cell(mangle, form):
    """Rust 那一份的 dump 经 mangle 改一处，入口的闭包要拒算 form。"""
    def judge(work):
        original, blanked = product_view_files()
        key, forms = product_view_key(work, original, mangle(product_view_dump_of(work, blanked)))
        return key is None and form in forms, f"要拒算 {form}，实际键 {'有' if key else '无'}、拒算 {sorted(forms)}"
    return judge


def product_view_extra_unit(lines):
    return lines + [f"fn\tfverus::phantom\t{PRODUCT_VIEW_FILE}:30-31\t{'0' * 64}"]


def product_view_outside_unit_missing(lines):
    return [line for line in lines if not line.startswith("fn\tfverus::caller\t")]


def product_view_block_unparsed(lines):
    return lines + [f"refused\t{VERUS_GHOST_CODE_UNPARSED}\t{PRODUCT_VIEW_FILE}:1\tverus! {{"]


def with_unit_graph_directory(directory, action):
    """FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 临时设成 directory 跑 action，跑完照原样恢复。"""
    previous = os.environ.get(UNIT_GRAPH_DIRECTORY_VARIABLE)
    os.environ[UNIT_GRAPH_DIRECTORY_VARIABLE] = directory
    try:
        return action()
    finally:
        if previous is None:
            os.environ.pop(UNIT_GRAPH_DIRECTORY_VARIABLE, None)
        else:
            os.environ[UNIT_GRAPH_DIRECTORY_VARIABLE] = previous


def product_view_unavailable_cell(work):
    original, _blanked = product_view_files()

    def keys():
        return product_view_key(work, original, None), product_view_key(work, original, None, entry="crates/fplain/src/lib.rs::plain")
    (key, forms), (plain_key, plain_forms) = with_unit_graph_directory(os.path.join(work, "nowhere"), keys)
    return (key is None and "product-view-unavailable" in forms and plain_key is not None,
            f"有 verus! 块那一份的入口：键 {'有' if key else '无'}、拒算 {sorted(forms)}（要拒算 product-view-unavailable）；没有的那一份：键 {plain_key}、拒算 {sorted(plain_forms)}")


def product_view_program_cell(work):
    original, blanked = product_view_files()
    dump = product_view_dump_of(work, blanked)
    expected, _forms = product_view_key(work, original, dump)
    program_directory = os.path.join(work, "bin")
    os.makedirs(program_directory)
    dump_file, call_log = os.path.join(work, "product-view.txt"), os.path.join(work, "calls.txt")
    with open(dump_file, "w", encoding="utf-8") as handle:
        handle.write("".join(line + "\n" for line in dump))
    program = os.path.join(program_directory, "unit-graph")
    with open(program, "w", encoding="utf-8") as handle:
        handle.write(f'#!/bin/bash\n[[ "$1 $3 $4" == "dump --judgment-kind {PRODUCT_BEHAVIOR}" && $# == 4 ]] || exit 2\n'
                     f'printf "%s\\n" "$2" >> {shlex.quote(call_log)}\ncat {shlex.quote(dump_file)}\n')
    os.chmod(program, 0o755)
    got, forms = with_unit_graph_directory(program_directory, lambda: product_view_key(work, original, None))
    formal, _formal_forms = with_unit_graph_directory(program_directory, lambda: product_view_key(work, original, None, FORMAL_VERIFICATION))
    calls = open(call_log, encoding="utf-8").read().split("\n")[:-1] if os.path.isfile(call_log) else []
    listed = dump_lines(os.path.join(work, "tree"), "rust", judgment_kind=PRODUCT_BEHAVIOR, product_dump=dump)[0]
    ghost_listed = [line for line in listed if "doubled" in line.split("\t")[1]]
    ok = got is not None and got == expected and formal is not None and calls == [os.path.join(work, "tree")] and not ghost_listed
    return ok, (f"起假程序算的键 {got}（拒算 {sorted(forms)}），交 dump 算的键 {expected}；假程序被起 {len(calls)} 次 {calls}（只该产品行为那一趟起一次、根是 tree）；"
                f"产品行为视图的 dump 里还列着只给 Verus 看的 {ghost_listed}")



def product_view_edges_closures(work, extra_pieces=()):
    """→ (产品行为视图下入口的闭包行, Rust 那一份的替身：手工剥过的拷贝照原样收的闭包行)；extra_pieces 是 Rust 那一份另剥的几截（这一份认不出的写法）。"""
    blanked = PRODUCT_VIEW_EDGES_SOURCE
    for piece in PRODUCT_VIEW_EDGES_GHOST_PIECES + tuple(extra_pieces):
        if blanked.count(piece) != 1:
            raise AssertionError(f"要剥的「{piece[:40]}」命中 {blanked.count(piece)} 次（要恰好一次）")
        blanked = blanked.replace(piece, "".join(character if character == "\n" else " " for character in piece))
    fixture = {"crates/fghostedges/Cargo.toml": '[package]\nname = "fghostedges"\n', "Cargo.lock": "# 夹具不解析依赖\n"}
    blanked_base, base = os.path.join(work, "blanked"), os.path.join(work, "tree")
    write_tree(blanked_base, dict(fixture, **{PRODUCT_VIEW_EDGES_FILE: blanked}))
    write_tree(base, dict(fixture, **{PRODUCT_VIEW_EDGES_FILE: PRODUCT_VIEW_EDGES_SOURCE}))
    unit_lines, refusal_lines, _counts = dump_lines(blanked_base, "rust")
    options = Options()
    options.judgment_kind, options.product_dump = PRODUCT_BEHAVIOR, unit_lines + refusal_lines
    product = closure_of(base, PRODUCT_VIEW_EDGES_ENTRY, options)
    expected = closure_of(blanked_base, PRODUCT_VIEW_EDGES_ENTRY)
    return (sorted(unit.dump_text() for unit in product.order), sorted(refusal.form for refusal in product.refusals),
            sorted(unit.dump_text() for unit in expected.order))


def closure_names(lines):
    return sorted(line.split("\t")[1].rsplit("::", 1)[-1] for line in lines)


def product_view_edges_cell(work):
    product, forms, expected = product_view_edges_closures(work)
    ok = product == expected and not forms and "HeaderModel" not in closure_names(product) and "Recorded" in closure_names(product)
    return ok, f"产品行为视图的闭包 {closure_names(product)}（拒算 {forms}）；unit-graph 剥完再收的 {closure_names(expected)}"


def product_view_unverified_cell(work):
    product, forms, expected = product_view_edges_closures(work, ("total = total + 1;",))
    names = closure_names(product)
    ok = not forms and "HeaderModel" in names and set(closure_names(expected)) <= set(names)
    return ok, f"产品行为视图的闭包 {names}（拒算 {forms}，要有 HeaderModel、不拒算）；Rust 那一份剥完再收的 {closure_names(expected)}"



def product_view_attribute_cell(inside_verus):
    """函数头上只多一行 PRODUCT_VIEW_ATTRIBUTE_LINE：Rust 那一份的替身是把这一行换成空格的拷贝照原样切出的 dump；产品行为键改动前后相同且都不拒算，
    形式化键变（属性进形式化的键）。inside_verus 为真加在 verus! 里被调到的 helper 上，否则加在没有 verus! 块的 crate 的 plain 上。"""
    def judge(work):
        original, blanked = product_view_files()
        file, head, entry = ((PRODUCT_VIEW_FILE, "fn helper(x: u64) -> u64 {\n", PRODUCT_VIEW_ENTRY) if inside_verus
                             else ("crates/fplain/src/lib.rs", "pub fn plain() -> u32 {\n", "crates/fplain/src/lib.rs::plain"))
        if original[file].count(head) != 1 or blanked[file].count(head) != 1:
            raise AssertionError(f"产品行为视图样本里「{head.strip()}」命中 {original[file].count(head)} 次（要恰好一次）")
        edited = dict(original, **{file: original[file].replace(head, f"{PRODUCT_VIEW_ATTRIBUTE_LINE}\n{head}")})
        edited_blanked = dict(blanked, **{file: blanked[file].replace(head, f"{' ' * len(PRODUCT_VIEW_ATTRIBUTE_LINE)}\n{head}")})
        before, before_forms = product_view_key(work, original, product_view_dump_of(work, blanked), entry=entry)
        after, after_forms = product_view_key(work, edited, product_view_dump_of(work, edited_blanked), entry=entry)
        formal_before, _ = product_view_key(work, original, None, FORMAL_VERIFICATION, entry)
        formal_after, _ = product_view_key(work, edited, None, FORMAL_VERIFICATION, entry)
        ok = before is not None and before == after and formal_before is not None and formal_after is not None and formal_before != formal_after
        return ok, (f"产品行为键改前 {before}、改后 {after}（拒算 {sorted(before_forms | after_forms)}，要不拒算、键相同）；"
                    f"形式化键改前 {formal_before}、改后 {formal_after}（要变）")
    return judge


def product_view_cells():
    return [
        (PRODUCT_VIEW_ATTRIBUTE_INSIDE_CELL, product_view_attribute_cell(True)),
        (PRODUCT_VIEW_ATTRIBUTE_OUTSIDE_CELL, product_view_attribute_cell(False)),
        (PRODUCT_VIEW_EDGES_CELL, product_view_edges_cell),
        (PRODUCT_VIEW_UNVERIFIED_CELL, product_view_unverified_cell),
        (PRODUCT_VIEW_LEMMA_CELL, product_view_lemma_cell),
        (PRODUCT_VIEW_ENSURES_CELL, product_view_compare_cell(("        r == doubled(x),", "        r == doubled(x) + 0,"), "same")),
        (PRODUCT_VIEW_EXECUTABLE_CELL, product_view_compare_cell(("    x + 0\n", "    x + 1\n"), "changed")),
        ("产品行为视图：Rust 那一份交的单元这一份没切出，拒算 product-view-unmatched", product_view_refused_cell(product_view_extra_unit, "product-view-unmatched")),
        ("产品行为视图：verus! 块外的单元 Rust 那一份没交，拒算 product-view-unmatched", product_view_refused_cell(product_view_outside_unit_missing,
                                                                                                "product-view-unmatched")),
        ("产品行为视图：Rust 那一份解不开 verus! 那一块，链接到它的闭包拒算 verus-ghost-code-unparsed",
         product_view_refused_cell(product_view_block_unparsed, VERUS_GHOST_CODE_UNPARSED)),
        ("产品行为视图：拿不到 Rust 那一份，有 verus! 块的文件拒算 product-view-unavailable，别的文件照算", product_view_unavailable_cell),
        ("产品行为视图：没交 dump 时起 FUNCTION_UNITS_UNIT_GRAPH_DIRECTORY 里的 unit-graph（只起产品行为那一趟），dump 不列只给 Verus 看的单元",
         product_view_program_cell),
    ]
