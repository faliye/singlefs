"""准入（入口 admission.py）的一块：verdict-plan、test-plan、verdict-record、plan-failure、plan-skip-names、plan-green-names 与测试函数清单，以及门禁复用答法、复跑表行的请求与行键、实验键、gate-reuse、gate-preconditions、replay-row-inputs、replay-row-keys 子命令。
入口 admission.py 把这一块的名字再导出。
"""

import collections
import hashlib
import importlib.util
import json
import os
import re
import sys

from admission_base import (
     REGISTRATION_TABLE, REPLAY_SCRIPT, RESULTS_DIRECTORY, GATE_GREEN_TREE_RECORD, VERDICT_LIBRARY_DIRECTORY_NAME, CHANGED_UNITS_SHOWN_AT_MOST,
     EXIT_REGISTRATION_ERROR, EXIT_GATE_MUST_RUN, EXIT_GATE_MAY_SKIP, EXIT_GATE_PRECONDITION_MISSING, EXIT_UNIT_KEY_REFUSED, EXPERIMENT_KEY_FORM,
     REPLAY_ROW_FORM, REPLAY_FROZEN_ROOT_CALL, REPLAY_FROZEN_ROOTS_DIRECTORY, FROZEN_ROOT_SKIPPED_DIRECTORIES, GPU_FEATURE, TEST_ATTRIBUTE_FORM,
     TEST_IGNORE_FORM, REPLAY_RESEARCH_DIRECTORY, REPLAY_JUDGING_FUNCTIONS, REPLAY_GPU_FEATURE_VARIABLE, REPLAY_PACKAGE_MENTION,
     REPLAY_PACKAGE_MENTION_DASHES_UNDER_BREAK, REPLAY_BIN_MENTION, REPLAY_BIN_VARIABLE_MENTION, REPLAY_FUNCTION_HEADER, REPLAY_DEVICE_KEY_MENTION,
     VERIFICATION_PACKAGE, RegistrationError, InputManifestError, read_registration_rows, rows_of_key, resolve_entries, entry_file, function_units,
     UnitKeyError, break_is_set, say, stored_fingerprints, replay_rows_with_keys, replay_products_of, judge_gate_preconditions, git_output,
     git_common_directory, registered_heavy_threads, multi_host_configuration_file, configuration_value, TEST_PLAN_SHARED, ENTRY_SCRIPT_NAME,
)
from admission_unit_key import (
     Request, parse_request_line, UnitKey, KeyContext, rust_cfg_for_profile, UNIT_LINE_TEXTS, domain_variable_tokens, closure_literals,
     compute_unit_key, changed_composition, VerdictLibrary, kind_directory_name, judged_date, write_atomically, composition_packed,
     REUSE_CHECKS_LIBRARY, SENTINEL_STATE_SUFFIX, UNIT_VERDICT_SAMPLE_DIRECTORY, reuse_checks_module, sentinel_candidates, plan_line, sha256_of_file,
     workspace_package_directories, research_binary_sources, research_package_name, workspace_binary_entry, test_plan_memoized_readers,
     file_library_store,
)

def plan_requests(root, kind, requests, context=None, unit_keys=None):
    """逐条算键、查库：交 ([(请求, UnitKey, 决定, 原因)], 判定库)，照请求的次序；算过的键各把组成清单落盘一份。
    unit_keys 给了就是调用方已算好的键（与 requests 一一对应，test-plan 在子进程里算的），不再算。
    复用哨兵（文件头「复用哨兵」）在查库之前抽：抽中强制现判的那一条不读库、照现判；抽到的哨兵查完库就记下它的结局。
    哨兵本身、要打的几行与红句挂在交出去的判定库上（sentinel、sentinel_lines、sentinel_problems）。"""
    context = context or KeyContext(root)
    library = VerdictLibrary(root, kind)
    keyed = list(zip(requests, unit_keys)) if unit_keys is not None else [(request, compute_unit_key(context, request)) for request in requests]
    library.sentinel, library.sentinel_lines, library.sentinel_problems = None, [], []
    checks = reuse_checks_module()
    if checks is None:
        library.sentinel_lines.append(f"  · 复用哨兵（{kind}）：找不到 {REUSE_CHECKS_LIBRARY}（这一份 admission.py 跑在不带它的树里），不抽哨兵")
    else:
        sentinel = checks.ReuseSentinel(f"scripts/gate/{ENTRY_SCRIPT_NAME}", kind)
        off_reason = (f"样本档（根下有 {UNIT_VERDICT_SAMPLE_DIRECTORY}/，样本钉死了现判与复用的条数）"
                      if os.path.isdir(os.path.join(root, UNIT_VERDICT_SAMPLE_DIRECTORY)) else "")
        sentinel.plan(sentinel_candidates(library, keyed) if not off_reason else {}, reuse=not off_reason, off_reason=off_reason,
                      printer=library.sentinel_lines.append)
        library.sentinel = sentinel
    settled = test_plan_settled_lookups(library, keyed)
    planned = []
    for position, (request, unit_key) in enumerate(keyed):
        if library.sentinel is not None and library.sentinel.forced(request.name):
            decision, reason = "judge", (f"强制现判（复用哨兵抽中，种子 {library.sentinel.seed}；库里键相同的判定是绿，这一趟不读库）")
        else:
            decision, reason = settled[position] if position in settled else library.lookup(request, unit_key)
        if library.sentinel is not None and request.name in library.sentinel.sentinels:
            library.sentinel.observe(request.name, "复用" if decision == "reuse" else "现判", "绿" if decision == "reuse" else "-")
        if position not in settled:
            library.store_composition(unit_key, request.name)
        planned.append((request, unit_key, decision, reason))
    if library.sentinel is not None and library.sentinel.state == "planned":
        # 开跑前就能判的那一半：哨兵有一条被现判 ⇒ 复用失灵；强制现判那一条的判定在 verdict-record 里比
        forced_item, library.sentinel.forced_item = library.sentinel.forced_item, None
        library.sentinel_problems = library.sentinel.finish(printer=lambda line: None)
        library.sentinel.forced_item = forced_item
    return planned, library


# ── 测试函数（tests-of:<包> 的展开；门禁按条入库的那几格要的名单）──────────────

class TestFunction:
    def __init__(self, entry, target_id, target_root, libtest_name, state):
        self.entry = entry
        self.target_id = target_id
        self.target_root = target_root
        self.libtest_name = libtest_name
        self.state = state
        self.name = entry   # 判定的名字：同一个函数编进几个目标（lib 与 bin 都 mod 了同一份文件）时另接「@<目标号>」，各是一条判定

    def line(self):
        return "\t".join([self.entry, self.target_id, self.target_root, self.libtest_name, self.state])


def unit_text_before_body(root, unit):
    """单元从它的第一个外属性起到第一个 { 之前的原文（看属性用）。"""
    try:
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")[unit.first_line - 1:unit.last_line]
    except OSError:
        return ""
    text = "\n".join(lines)
    return text.split("{", 1)[0]


def test_functions_of_package(context, package, closure_root=None, features=(), profile=None, resolve=True):
    """这个包 lib、各 bin 与各测试目标里带 #[test] 的函数（cargo test --tests 跑的就是这几种目标）：[TestFunction]，按（目标号、libtest 全名）排。
    state：compiled、not-compiled（这一组特性下 cfg 关着）、ignored。libtest 认的全名 = 那份文件在目标里的模块路径 + 单元的限定名（内联 mod 已在限定名里）。
    同一个函数编进几个目标的，名字另接「@<目标号>」（入口不变）。
    resolve=False 时不收闭包：不标 ignore 的 state 先写 pending，另挂 closure_arguments（交给 KeyContext.rust_closure 的那几样），由 test-plan 分进子进程判（文件头 test-plan）。"""
    library = function_units()
    closure_root = closure_root or context.root
    workspace = context.workspace(closure_root)
    try:
        crate = workspace.crate_by_package(package)
    except library.UsageError as error:
        raise RegistrationError(f"包 {package} 不在工作区里：{error}") from error
    cfg = rust_cfg_for_profile(closure_root, profile)
    found = []
    for target in sorted(workspace.targets.values(), key=lambda target: target.id):
        if target.crate is not crate or target.kind not in ("lib", "bin", "test"):
            continue
        for path, (module_path, _chain) in sorted(target.files.items()):
            for unit in workspace.files[path].units:
                if unit.kind != "fn":
                    continue
                head = unit_text_before_body(closure_root, unit)
                if not TEST_ATTRIBUTE_FORM.search(head):
                    continue
                libtest_name = "::".join(list(module_path) + [unit.name])
                entry = f"{path}::{unit.name}"
                closure_arguments = None
                if TEST_IGNORE_FORM.search(head):
                    state = "ignored"
                elif not resolve:
                    state, closure_arguments = "pending", (closure_root, workspace, unit, target.id, cfg, list(features), False)
                else:
                    try:
                        context.rust_closure(closure_root, workspace, unit, target.id, cfg, list(features), False)
                        state = "compiled"
                    except library.UsageError:
                        state = "not-compiled"
                target_root = posixpath_relative(target.root_file, crate.directory)
                found.append(TestFunction(entry, target.id, target_root, libtest_name, state))
                found[-1].closure_arguments = closure_arguments
    entry_counts = collections.Counter(function.entry for function in found)
    for function in found:
        if entry_counts[function.entry] > 1:
            function.name = f"{function.entry}@{function.target_id}"
    return sorted(found, key=lambda item: (item.target_id, item.libtest_name))


def posixpath_relative(path, directory):
    if not directory:
        return path
    return path[len(directory) + 1:] if path.startswith(directory + "/") else path


def expand_entries(context, entries):
    """登记的入口展开成一条条判定的入口：tests-of:<包> 展开成那个包里不标 ignore、这一组 cfg 下编得进的测试函数；别的原样。"""
    expanded = []
    for entry in entries:
        if entry.startswith("tests-of:"):
            expanded += [item.entry for item in test_functions_of_package(context, entry[len("tests-of:"):]) if item.state == "compiled"]
        else:
            expanded.append(entry)
    return list(dict.fromkeys(expanded))


# ── 门禁的复用答法（gate-reuse）────────────────────────────────────────────────

def gate_reuse(root, key, request_file=None):
    """返回 (退出码, 一句依据, 要现判的名字)。退 EXIT_GATE_MUST_RUN 要跑，EXIT_GATE_MAY_SKIP 可跳过。"""
    if break_is_set("tree-compare-reuse"):
        return tree_compare_reuse(root, key)
    library = VerdictLibrary(root, key)
    context = KeyContext(root)
    if request_file:
        with open(request_file, encoding="utf-8", errors="surrogateescape") as handle:
            requests = [parse_request_line(line) for line in handle if line.strip() and not line.startswith("#")]
    else:
        recorded = dict(library.items())
        requests = []
        for entry in expand_entries(context, resolve_entries(context.rows(), key)):
            record = recorded.get(entry)
            requests.append(parse_request_line(record["request"]) if record and record.get("request") else Request(entry, entry, []))
    if not requests:
        return EXIT_GATE_MUST_RUN, f"{REGISTRATION_TABLE} 里 {key} 一个入口都没登记，按要跑处理", []
    to_judge, notes = [], []
    for request in requests:
        unit_key = compute_unit_key(context, request)
        decision, reason = library.lookup(request, unit_key)
        if decision != "reuse":
            to_judge.append(request.name)
            notes.append(f"{request.name}：{reason}")
    if not to_judge:
        return EXIT_GATE_MAY_SKIP, f"{key} 登记的 {len(requests)} 条判定在判定库 {library.directory} 里都有键相同的判定（复用 {len(requests)}、现判 0）", []
    shown = "；".join(notes[:CHANGED_UNITS_SHOWN_AT_MOST])
    return EXIT_GATE_MUST_RUN, (f"{key} 登记的 {len(requests)} 条判定里键对不上的 {len(to_judge)} 条要现判"
                                f"（复用 {len(requests) - len(to_judge)}）：{shown}{' …' if len(notes) > CHANGED_UNITS_SHOWN_AT_MOST else ''}"), to_judge


def tree_compare_reuse(root, key):
    """弄坏开关 tree-compare-reuse：按树比的复用路——refs/sop/staged-green 那棵树（没有取 HEAD 的树）与工作区在登记入口的文件上逐字相同就答可跳过。"""
    entries = resolve_entries(read_registration_rows(root), key)
    files = sorted({path for path in (entry_file(entry) for entry in entries) if path})
    for reference in (f"{GATE_GREEN_TREE_RECORD}^{{tree}}", "HEAD^{tree}"):
        code, _output = git_output(root, "rev-parse", "-q", "--verify", reference)
        if code == 0:
            break
    else:
        return EXIT_GATE_MUST_RUN, "（弄坏开关 tree-compare-reuse）没有可比的树", files
    code, output = git_output(root, "diff", "--name-only", reference, "--", *files)
    if code == 0 and not output.strip():
        return EXIT_GATE_MAY_SKIP, f"（弄坏开关 tree-compare-reuse）登记入口的文件与 {reference} 逐字相同", []
    return EXIT_GATE_MUST_RUN, f"（弄坏开关 tree-compare-reuse）与 {reference} 不同：{output.strip()}", files


def shell_function_text(context, function):
    """replay.sh 里一个顶格函数的闭包（lib_function_units 的 shell 闭包）里 replay.sh 那几个单元的原文拼起来：找驱动点名的 bin、包与 CHECKER_TIER_FEATURES 用。算不出交 None。"""
    try:
        closure = context.script_closure(context.root, f"{REPLAY_SCRIPT}::{function}")
    except UnitKeyError:
        return None
    return "\n".join(unit_source_text(context.root, unit) for unit in closure.order
                     if getattr(unit, "language", None) == "shell" and unit.file == REPLAY_SCRIPT and not getattr(unit, "data", False))


def unit_source_text(root, unit):
    try:
        with open(os.path.join(root, unit.file), encoding="utf-8", errors="replace") as handle:
            return "\n".join(handle.read().split("\n")[unit.first_line - 1:unit.last_line])
    except OSError:
        return ""


FROZEN_ROOT_DIGESTS = {}


def frozen_root_directory(context, name):
    """驱动点名的冻结根在哪：<工作根>/replay-frozen-roots/<名>。工作根照 replay.sh 的 replay_frozen_root 同一份定义（<仓根>/.claude/hooks/lib_work_root.py 的 work_root）；
    那一份不在、载不进、定不出工作根交 UnitKeyError。"""
    path = os.path.join(context.root, ".claude", "hooks", "lib_work_root.py")
    if not os.path.isfile(path):
        raise UnitKeyError(f"{path} 不在，定不出工作根")
    specification = importlib.util.spec_from_file_location("singlefs_frozen_root_work_root", path)
    module = importlib.util.module_from_spec(specification)
    try:
        specification.loader.exec_module(module)
        work = module.work_root()
    except Exception as error:   # 载入或定位置的任何问题都按算不出键处理
        raise UnitKeyError(f"{path} 定不出工作根：{error}") from error
    return os.path.join(work, REPLAY_FROZEN_ROOTS_DIRECTORY, name)


def tree_content_rows(base, relative):
    """base 底下 relative 那一条（文件、符号链接或目录）的内容行：「相对 base 的路径<TAB>字节 sha256」，符号链接写「<路径><TAB>-><指到哪>」；
    目录递归，跳过 FROZEN_ROOT_SKIPPED_DIRECTORIES 和 target-* 编译目录。不在的交空表。"""
    absolute = os.path.join(base, relative)
    if os.path.islink(absolute):
        return [f"{relative}\t->{os.readlink(absolute)}"]
    if os.path.isfile(absolute):
        try:
            digest = sha256_of_file(absolute)
        except OSError:
            digest = "读不了"
        return [f"{relative}\t{digest}"]
    rows = []
    for current, names, files in os.walk(absolute):
        names[:] = sorted(item for item in names if item not in FROZEN_ROOT_SKIPPED_DIRECTORIES and not item.startswith("target-"))
        for item in names + sorted(files):
            path = os.path.join(current, item)
            if os.path.islink(path) or os.path.isfile(path):
                rows += tree_content_rows(base, os.path.relpath(path, base))
    return rows


def frozen_root_digest(context, name):
    """冻结根 <名> 的内容摘要：目录底下每个文件（跳过 FROZEN_ROOT_SKIPPED_DIRECTORIES）的相对路径与字节 sha256 排好序连起来取 sha256；
    目录不在交「不在」（那一行本来就记跑不了，根重建出来键才变）。同一个进程里同一个目录只算一次。"""
    directory = frozen_root_directory(context, name)
    if directory not in FROZEN_ROOT_DIGESTS:
        if not os.path.isdir(directory):
            FROZEN_ROOT_DIGESTS[directory] = "不在"
        else:
            rows = tree_content_rows(directory, ".")
            FROZEN_ROOT_DIGESTS[directory] = hashlib.sha256("\n".join(sorted(rows)).encode("utf-8", "surrogateescape")).hexdigest()
    return FROZEN_ROOT_DIGESTS[directory]


def replay_packages_mentioned(text):
    """驱动正文里 -p <包> 点名的包（排好序）：包名首字符不认 -（mkdir -p -- … 的 -- 不是包名）；弄坏开关 replay-package-mention-dashes 下照认。"""
    pattern = REPLAY_PACKAGE_MENTION_DASHES_UNDER_BREAK if break_is_set("replay-package-mention-dashes") else REPLAY_PACKAGE_MENTION
    return sorted(set(pattern.findall(text)))


def replay_binaries_mentioned(text):
    """驱动正文里点名的 bin：--bin <字面>，加上 --bin "$变量" 写在包装函数里的（e170_r2_cargo_run <bin> <参数…>）：那个函数每一处调用的头一个词。
    弄坏开关 replay-bin-wrappers-unread 下只认字面的。"""
    found = REPLAY_BIN_MENTION.findall(text)
    if break_is_set("replay-bin-wrappers-unread"):
        return found
    for mention in REPLAY_BIN_VARIABLE_MENTION.finditer(text):
        headers = list(REPLAY_FUNCTION_HEADER.finditer(text, 0, mention.start()))
        if headers:
            found += re.findall(r"(?:^|[\s;&|(])" + re.escape(headers[-1].group(1)) + r"\s+([A-Za-z0-9_-]+)", text)
    return found


def replay_device_key_binaries(text):
    """驱动正文里 device-code-key.py 的 --package-directory <目录> --binary <名>：[(目录, 名)…]。弄坏开关 replay-device-key-binaries-unread 下一个都不认。"""
    return [] if break_is_set("replay-device-key-binaries-unread") else REPLAY_DEVICE_KEY_MENTION.findall(text)


def replay_row_request(context, row_text, gpu_switch, registered_entries, without_product=False):
    """复跑表一行的判定请求：交 (Request 列表, 拒算说明的清单, 驱动闭包点没点到 CHECKER_TIER_FEATURES)。一行可能有几个入口（驱动 + 它点名的装置）：每个入口一条请求，行键是它们各自的键再合起来。"""
    root = context.root
    match = REPLAY_ROW_FORM.match(row_text)
    experiment, bin_text, arguments_text, product, compare = match.group(1, 2, 3, 4, 5)
    common = [f"line:arguments={arguments_text}", f"line:compare={compare}"]
    common += [f"judge:{REPLAY_SCRIPT}::{name}" for name in REPLAY_JUDGING_FUNCTIONS]
    if not without_product:
        common.append(f"file:{RESULTS_DIRECTORY}/{product}")
        if break_is_set("row-key-with-product-name"):
            common.append(f"line:product-name={product}")
    binaries = research_binary_sources(root)
    entries, refusals = [], []
    uses_gpu = False
    if bin_text.startswith("@"):
        driver = bin_text[1:]
        entries.append((f"{REPLAY_SCRIPT}::{driver}", []))
        text = shell_function_text(context, driver)
        if text is None:
            refusals.append(f"refused\treplay-driver-unreadable\t{REPLAY_SCRIPT}\t{driver}")
            text = ""
        uses_gpu = REPLAY_GPU_FEATURE_VARIABLE in text
        if not break_is_set("frozen-root-unkeyed"):
            for frozen_name in sorted(set(REPLAY_FROZEN_ROOT_CALL.findall(text))):
                try:
                    common.append(f"line:frozen-root={frozen_name} {frozen_root_digest(context, frozen_name)}")
                except UnitKeyError as error:
                    refusals.append(f"refused\treplay-frozen-root-unreadable\t{REPLAY_SCRIPT}\t{driver} 点名的冻结根 {frozen_name}：{error}")
        for name in sorted(binaries):
            if re.search(r"(?<![A-Za-z0-9_-])" + re.escape(name) + r"(?![A-Za-z0-9_-])", text):
                entries.append((f"{binaries[name]}::main", [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        named_bins = replay_binaries_mentioned(text)
        research_package = research_package_name(root)
        for package in replay_packages_mentioned(text):
            if package == research_package and not break_is_set("research-package-unresolved"):
                # 研究工作区的包（驱动经 --manifest-path scripts/experiment-apparatus/Cargo.toml 编它）：--bin 的名字解到那里 [[bin]] 的源码，与字面点名研究二进制同一种入口
                found = [f"{binaries[name]}::main" for name in named_bins if name in binaries]
                if not found:
                    refusals.append(f"refused\treplay-package-without-binary\t{REPLAY_SCRIPT}\t{driver} 点名了 -p {package}，点不出 bin")
                for entry in found:
                    entries.append((entry, [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
                continue
            found = [workspace_binary_entry(root, package, name) for name in named_bins]
            found = [entry for entry in found if entry]
            if not found:
                refusals.append(f"refused\treplay-package-without-binary\t{REPLAY_SCRIPT}\t{driver} 点名了 -p {package}，点不出 bin")
            for entry in found:
                features = []
                if package == VERIFICATION_PACKAGE and gpu_switch == "1" and uses_gpu:
                    features = [f"feature:{VERIFICATION_PACKAGE}/{GPU_FEATURE}"]
                entries.append((entry, ["profile:release", *features]))
        for directory, binary in replay_device_key_binaries(text):
            package = next((name for name, place in workspace_package_directories(root).items() if place == directory), None)
            entry = workspace_binary_entry(root, package, binary) if package is not None else None
            if entry is None:
                refusals.append(f"refused\treplay-package-without-binary\t{REPLAY_SCRIPT}\t{driver} 点名了 --package-directory {directory} --binary {binary}，点不出 bin")
                continue
            features = [f"feature:{VERIFICATION_PACKAGE}/{GPU_FEATURE}"] if package == VERIFICATION_PACKAGE and gpu_switch == "1" and uses_gpu else []
            entries.append((entry, ["profile:release", *features]))
    elif bin_text in binaries:
        entries.append((f"{binaries[bin_text]}::main", [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
    else:
        refusals.append(f"refused\treplay-binary-unknown\t{REPLAY_SCRIPT}\t{bin_text} 不是 scripts/experiment-apparatus/Cargo.toml 的 [[bin]]、也不是 @<驱动函数>")
    if uses_gpu and not break_is_set("row-key-without-gpu"):
        common.append(f"line:gpu={gpu_switch}")
    for entry in registered_entries:
        if entry.startswith(REPLAY_RESEARCH_DIRECTORY + "/") and entry.split("::", 1)[0].endswith(".rs"):
            entries.append((entry[len(REPLAY_RESEARCH_DIRECTORY) + 1:], [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        else:
            entries.append((entry, []))
    identity = "|".join((experiment, bin_text, arguments_text))
    requests = [Request(identity, entry, tokens + common) for entry, tokens in dict((entry, tokens) for entry, tokens in entries).items()]
    return requests, refusals, uses_gpu


def row_unit_key(context, requests, refusals):
    """一行的行键：各入口的键（入口自己、它的判法与内容输入都在里面）排好合起来；有一个拒算整行拒算。交 UnitKey。"""
    keys, composition, seen, all_refusals = [], [], set(), list(refusals)
    for request in requests:
        unit_key = compute_unit_key(context, request)
        for line in unit_key.composition:
            if line not in seen:
                seen.add(line)
                composition.append(line)
        if unit_key.key is None:
            all_refusals += unit_key.refusals
        else:
            keys.append(unit_key.key)
    if all_refusals or not requests:
        return UnitKey(None, composition, sorted(set(all_refusals)) or ["refused\treplay-row-without-entry\t-\t这一行一个入口都没有"])
    return UnitKey(hashlib.sha256("".join(key + "\n" for key in sorted(keys)).encode()).hexdigest(), composition, [])


def replay_table_rows(root, wanted_experiments=()):
    replay_path = os.path.join(root, REPLAY_SCRIPT)
    with open(replay_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    rows = []
    for line in text.split("\n"):
        match = REPLAY_ROW_FORM.match(line)
        if match and (not wanted_experiments or match.group(1) in wanted_experiments):
            rows.append(line)
    return rows


def gpu_switch_of(root):
    """多机配置的 ENABLE_GPU（找法同 crash_case_cargo_features）：「1」或「0」；读不了按 0（replay.sh 开跑前自己另判配置读不读得出）。"""
    try:
        return "1" if configuration_value(multi_host_configuration_file(root, os.environ), "ENABLE_GPU") == "1" else "0"
    except InputManifestError:
        return "0"


def shell_index_outputs_added(index, before):
    """index 的 $(cat <<词 …) 输出里比 before 多的那几个：子进程只交回新认出来的，父进程并进它那一份。"""
    added = {}
    for inner, values in index.substitution_outputs.items():
        new = values - before.get(inner, set())
        if new:
            added[inner] = new
    return added


def shell_index_file_parsed(path):
    """进程池的一件：一份脚本读进来、bash 规范化、切单元、算好每个单元的 facts（与 ShellIndex.add、compute_groups 开头做的同一样）：→ (ShellFile, 新认出来的 $(cat <<词 …) 输出)。"""
    index = TEST_PLAN_SHARED["shell_index"]
    library = function_units()
    before = {inner: set(values) for inner, values in index.substitution_outputs.items()}
    with open(os.path.join(index.root, path), "rb") as handle:
        content = handle.read()
    shell_file = library.ShellFile(path, content, library.shell_normal_forms([content])[0], index.breaks)
    for unit in shell_file.units:
        if not unit.data:
            index.facts(unit)
    return shell_file, shell_index_outputs_added(index, before)


def shell_index_links_of_file(path):
    """进程池的一件：一份脚本每个单元的边（ShellIndex.compute_groups 里一趟做的那一步）→ (文件, source 到的文件, [(单元下标, 单元的缓存, 单元的 programs)], 新认出来的 $(cat <<词 …) 输出)。"""
    index = TEST_PLAN_SHARED["shell_index"]
    before = {inner: set(values) for inner, values in index.substitution_outputs.items()}
    targets, units = set(), []
    for position, unit in enumerate(index.files[path].units):
        if unit.data:
            continue
        for kind, files, _refusal in index.links(unit):
            if kind == "source":
                targets.update(target for target in files if target in index.files and target != path)
        units.append((position, unit.cache, unit.facts.programs))
    return path, targets, units, shell_index_outputs_added(index, before)


def shell_index_filled(root, paths, jobs, maximum_rounds):
    """与 lib_function_units.py 的 ShellIndex(root, paths) 逐项相同的索引（ShellIndex.add 与 compute_groups 同一判法，那边至多三趟，maximum_rounds 照写 3），各件分到 jobs 个 fork 子进程里算：→ (索引, 算了几趟)。
    父进程读全部文件的路径表（不然子进程各走一遍）；source 的边要照上一趟的边折变量，所以一趟一个进程池，算到不再变。
    交回的单元与拒算行是那份库里的类，pickle 按模块名认，这份库按路径载入，先把它登进 sys.modules。"""
    import concurrent.futures
    import multiprocessing
    library = function_units()
    sys.modules.setdefault(library.__name__, library)
    index = library.ShellIndex(root, [])
    TEST_PLAN_SHARED["shell_index"] = index
    fork = multiprocessing.get_context("fork")
    wanted = [path for path in dict.fromkeys(paths) if os.path.isfile(os.path.join(index.root, path))]
    with concurrent.futures.ProcessPoolExecutor(max_workers=jobs, mp_context=fork) as pool:
        for shell_file, outputs in pool.map(shell_index_file_parsed, wanted):
            index.files[shell_file.path] = shell_file
            for inner, values in outputs.items():
                index.substitution_outputs[inner] |= values
    index.repository_files = library.files_under(index.root, ("",))
    index.sources_of = {path: frozenset() for path in index.files}
    rounds = 0
    for _round in range(maximum_rounds):
        rounds += 1
        edges = {}
        with concurrent.futures.ProcessPoolExecutor(max_workers=jobs, mp_context=fork) as pool:
            for path, targets, units, outputs in pool.map(shell_index_links_of_file, list(index.files)):
                edges[path] = targets
                for position, cache, programs in units:
                    unit = index.files[path].units[position]
                    unit.cache, unit.facts.programs = cache, programs
                for inner, values in outputs.items():
                    index.substitution_outputs[inner] |= values
        sources = {path: frozenset(edges.get(path, ())) for path in index.files}
        if sources == index.sources_of:
            break
        index.sources_of = sources
        for shell_file in index.files.values():
            for unit in shell_file.units:
                if unit.cache is not None:
                    unit.cache.pop("links", None)
    index.sourcers_of = collections.defaultdict(set)
    for path, targets in index.sources_of.items():
        for target in targets:
            index.sourcers_of[target].add(path)
    return index, rounds


def replay_row_shell_index_prefilled(context):
    """复跑行键的固定前缀里最重的 shell 索引分进子进程建好，登进 context.script_indices（文件头 replay-row-inputs「并行」）。
    线程数是 1 或取不了、进程池没跑完的留给 KeyContext.script_closure 在本进程里建，行键不变。弄坏开关 replay-row-shell-index-serial：不分。"""
    note = "processes=0"
    try:
        jobs = registered_heavy_threads(os.environ)
    except InputManifestError:
        jobs = 1
    if jobs > 1 and not break_is_set("replay-row-shell-index-serial"):
        try:
            library = function_units()
            under = [place for place in library.DEFAULT_SCRIPT_DIRECTORIES if os.path.isdir(os.path.join(context.root, place))]
            base = library.ScriptClosure(context.root, under)
            base.shell, rounds = shell_index_filled(context.root, library.shell_files_under(context.root, under), jobs, 3)
            context.script_indices[(context.root, tuple(under))] = base
            note = f"processes={jobs} files={len(base.shell.files)} rounds={rounds}"
        except Exception as error:  # 进程池没跑完不影响算得对不对：本进程里照常建，只是慢
            say(f"  ! shell 索引的进程池没跑完（{type(error).__name__}：{error}），这一趟在本进程里建")
    if os.environ.get("ADMISSION_REPLAY_ROW_WORKERS_FILE"):
        with open(os.environ["ADMISSION_REPLAY_ROW_WORKERS_FILE"], "w", encoding="utf-8") as handle:
            handle.write(note + "\n")


def replay_row_keys(root, wanted_experiments=(), gpu_switch=None, without_product=False):
    """复跑表每一行的行键：[(前三列, UnitKey, 点没点到 CHECKER_TIER_FEATURES)]，照表里的次序。"""
    context = KeyContext(root)
    replay_row_shell_index_prefilled(context)
    gpu_switch = gpu_switch if gpu_switch is not None else gpu_switch_of(root)
    replay_rows, _problems = replay_rows_with_keys(root)
    key_of_identity = {row.identity: row.key for row in replay_rows}
    registered_rows = context.rows()
    out = []
    for row_text in replay_table_rows(root, wanted_experiments):
        identity = "|".join(REPLAY_ROW_FORM.match(row_text).group(1, 2, 3))
        admission_key = key_of_identity.get(identity, identity.split("|")[0])
        try:
            registered = resolve_entries(registered_rows, admission_key) if rows_of_key(registered_rows, admission_key) else []
        except RegistrationError:
            registered = []
        requests, refusals, uses_gpu = replay_row_request(context, row_text, gpu_switch, registered, without_product)
        out.append((identity, row_unit_key(context, requests, refusals), uses_gpu))
    return out


# ── 实验的准入（行键去掉期望产物那一项）──────────────────────────────────────────

def experiment_key(root, key):
    """实验键：复跑表里准入键是 key 的那一行的行键，不带期望产物（产物头记着它）。那一行不在复跑表里的，按登记的入口算（同一套请求，不带行的参数）。
    交 (UnitKey, 说明)。"""
    rows, problems = replay_rows_with_keys(root)
    if problems:
        raise RegistrationError("；".join(problems))
    identities = [row.identity for row in rows if row.key == key]
    context = KeyContext(root)
    registered = resolve_entries(context.rows(), key)
    if len(identities) == 1:
        experiment = identities[0].split("|")[0]
        for identity, unit_key, _uses_gpu in replay_row_keys(root, (experiment,), without_product=True):
            if identity == identities[0]:
                return unit_key, f"复跑表「{identity}」那一行的行键（不带期望产物）"
    if not registered:
        raise RegistrationError(f"{key} 在复跑表里没有恰好一行（{len(identities)} 行），登记表那一行也没登记入口：算不出实验键")
    requests = []
    for entry in registered:
        if entry.startswith(REPLAY_RESEARCH_DIRECTORY + "/") and entry.split("::", 1)[0].endswith(".rs"):
            requests.append(Request(key, entry[len(REPLAY_RESEARCH_DIRECTORY) + 1:], [f"root:{REPLAY_RESEARCH_DIRECTORY}", "profile:release"]))
        else:
            requests.append(Request(key, entry, []))
    return row_unit_key(context, requests, []), f"登记的 {len(registered)} 个入口"



def judge_inputs_unchanged(root, key, fingerprint):
    """输入没变：返回 (没变?, 说明)。research/results/ 里有一份产物头上记着同一个键、同一把实验键就算没变。"""
    stored = stored_fingerprints(root, key)
    replay_products = replay_products_of(root, key)
    matching = [entry for entry in stored if entry.fingerprint == fingerprint]
    if matching:
        preferred = next((entry for entry in matching if entry.product_path in replay_products), matching[0])
        return True, f"{preferred.product_path} 头上的实验键与今天的相同（{fingerprint[:12]}）"
    if stored:
        newest = max(stored, key=lambda entry: entry.modified_at)
        return False, f"实验键自 {newest.product_path} 以来变了（那一份 {newest.fingerprint[:12]}，今天 {fingerprint[:12]}）"
    return False, f"上一份产物头上没有 {key} 的实验键，判不了（research/results/ 里没有一份头上带它的产物）"


def stored_product_row_key_status(root, key):
    """给门禁 doc-experiments 的 evidence-in-repo 格：这个实验键的产物跟不跟得上今天的实验键（行键去掉期望产物）。
    返回 (状态, 说明)：unregistered（没登记准入）、unfingerprinted（没有一份带实验键的产物，或今天的实验键拒算）、
    matched（有一份记的与今天的相同）、stale（带实验键的都与今天的不同）。登记写错抛 RegistrationError。"""
    rows = read_registration_rows(root)
    if not rows_of_key(rows, key):
        return "unregistered", ""
    stored = stored_fingerprints(root, key)
    if not stored:
        return "unfingerprinted", ""
    unit_key, _basis = experiment_key(root, key)
    if unit_key.key is None:
        return "unfingerprinted", "今天的实验键拒算"
    matching = [entry for entry in stored if entry.fingerprint == unit_key.key]
    if matching:
        return "matched", f"{matching[0].product_path}（实验键 {unit_key.key[:12]}）"
    newest = max(stored, key=lambda entry: entry.modified_at)
    return "stale", f"最新带实验键的产物 {newest.product_path} 记的是 {newest.fingerprint[:12]}，今天的是 {unit_key.key[:12]}"


# ── 子命令 ────────────────────────────────────────────────────────────────────

def command_entries(arguments, files_only=False):
    if len(arguments) != 2:
        say(f"  ✗ 用法：admission.py {'paths' if files_only else 'entries'} <项目根> <键>")
        say("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 的第一列写")
        return EXIT_REGISTRATION_ERROR
    try:
        entries = resolve_entries(read_registration_rows(arguments[0]), arguments[1])
    except RegistrationError as error:
        say(f"  ✗ {error}")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 给这个键登记它的入口（制表符分隔，照别的行写）")
        return EXIT_REGISTRATION_ERROR
    shown = list(dict.fromkeys(path for path in (entry_file(entry) for entry in entries) if path)) if files_only else entries
    for item in shown:
        print(item)
    return 0


def command_test_functions(arguments):
    values = option_pairs(arguments[2:], ("--root", "--feature", "--profile")) if len(arguments) >= 2 else None
    if values is None:
        say("  ✗ 用法：admission.py test-functions <项目根> <包> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>]")
        say("     → 怎么办：包写 Cargo.toml 的包名；research 工作区写 --root scripts/experiment-apparatus")
        return EXIT_REGISTRATION_ERROR
    root = os.path.abspath(arguments[0])
    context = KeyContext(root)
    closure_root = os.path.normpath(os.path.join(root, values.get("--root", ["."])[-1]))
    try:
        functions = test_functions_of_package(context, arguments[1], closure_root, values.get("--feature", []), values.get("--profile", [None])[-1])
    except (RegistrationError, UnitKeyError) as error:
        print(f"  ✗ 列不出 {arguments[1]} 的测试函数：{error}")
        print("     → 怎么办：包名照 Cargo.toml 写；工作区读不出时先单跑 python3 scripts/gate/lib_function_units.py dump <根> 看它报什么")
        return EXIT_REGISTRATION_ERROR
    for function in functions:
        print(function.line())
    return 0


def option_pairs(options, names):
    """--名字 值 成对的参数，同一个名字可以给几次：返回 {名字: [值…]}；有不认得的、缺值的返回 None。"""
    values = {}
    index = 0
    while index < len(options):
        if options[index] not in names or index + 1 >= len(options):
            return None
        values.setdefault(options[index], []).append(options[index + 1])
        index += 2
    return values


def command_unit_key(arguments):
    if len(arguments) < 2:
        say("  ✗ 用法：admission.py unit-key <项目根> <入口> [<项>…]")
        say("     → 怎么办：入口照 lib_function_units.py 的写法；项见文件头「判定请求」")
        return EXIT_REGISTRATION_ERROR
    try:
        request = Request(arguments[1], arguments[1], arguments[2:])
    except RegistrationError as error:
        say(f"  ✗ {error}")
        say("     → 怎么办：项照文件头「判定请求」写")
        return EXIT_REGISTRATION_ERROR
    unit_key = compute_unit_key(KeyContext(os.path.abspath(arguments[0])), request)
    if unit_key.key is None:
        for line in unit_key.refusals:
            print(line)
        return EXIT_UNIT_KEY_REFUSED
    print(f"key\t{unit_key.key}")
    for line in unit_key.composition:
        print(line)
    return 0


def read_requests(path):
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        return [parse_request_line(line) for line in handle if line.strip() and not line.startswith("#")]


def print_plan(root, planned, library, plan_file, extra_columns=None, extra_counts=None):
    """verdict-plan 与 test-plan 共用的收尾：逐条打、写计划文件、打末行 PLAN。"""
    counts = collections.Counter(decision for _request, _key, decision, _reason in planned)
    plan_lines = []
    for request, unit_key, decision, reason in planned:
        middle = extra_columns(request) if extra_columns else []
        print("\t".join([request.name, decision, unit_key.key or "-", *middle, reason]))
        plan_lines.append(plan_line(root, request, unit_key, decision))
    problem = write_atomically(plan_file, "".join(line + "\n" for line in plan_lines)) if plan_file != "-" else ""
    if problem:
        print(f"  ✗ 计划文件没写成：{problem}")
        print("     → 怎么办：看那个目录可不可写；计划文件写不成，这一趟判绿的也记不进库")
        return EXIT_REGISTRATION_ERROR
    # 复用哨兵（文件头「复用哨兵」）：抽中了谁、种子打在逐条那几行之后、PLAN 之前（调用方取头一行或按列取的都不碰到它）；
    # 状态存在计划文件旁边，verdict-record 拿它比强制现判那一条
    sentinel = getattr(library, "sentinel", None)
    for line in getattr(library, "sentinel_lines", []):
        print(line)
    if sentinel is not None and plan_file != "-":
        observed = "".join(f"{item}\t{state}\t{verdict}\n" for item, (state, verdict) in sentinel.observed.items())
        problem = (write_atomically(plan_file + SENTINEL_STATE_SUFFIX, json.dumps(sentinel.to_json(), ensure_ascii=False))
                   or write_atomically(plan_file + SENTINEL_STATE_SUFFIX + ".observed", observed))
        if problem:
            print(f"  ! 复用哨兵的状态没存下（{problem}）：verdict-record 比不了强制现判那一条")
    elif plan_file != "-":
        for stale in (plan_file + SENTINEL_STATE_SUFFIX, plan_file + SENTINEL_STATE_SUFFIX + ".observed"):
            if os.path.exists(stale):
                os.unlink(stale)
    extra = "".join(f" {name}={value}" for name, value in (extra_counts or {}).items())
    print(f"PLAN judge={counts['judge']} reuse={counts['reuse']} refused={counts['refused']}{extra} library={library.directory}")
    sentinel_problems = getattr(library, "sentinel_problems", [])
    for message, howto in sentinel_problems:
        print(f"  ✗ {message}")
        print(f"     → 怎么办：{howto}")
    return 1 if sentinel_problems else 0


def command_verdict_plan(arguments):
    values = option_pairs(arguments[4:], ()) if len(arguments) >= 4 else None
    if values is None:
        print("  ✗ 用法：admission.py verdict-plan <项目根> <种类> <请求文件> <计划文件>")
        print("     → 怎么办：请求文件一行一条「<名字><制表符><入口>[<制表符><项>…]」（文件头「判定请求」）；种类写门禁的登记键")
        return EXIT_REGISTRATION_ERROR
    root, kind, request_file, plan_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    try:
        requests = read_requests(request_file)
    except (OSError, RegistrationError) as error:
        print(f"  ✗ 读不了请求文件 {request_file}：{error}")
        print("     → 怎么办：照文件头「判定请求」写请求文件")
        return EXIT_REGISTRATION_ERROR
    sample_failure = plan_failure_sample(root)
    if sample_failure != "sentinel-reuse-failed" and sample_failure is not None:
        return sample_plan_failure_exit("verdict-plan", root, sample_failure)
    if settle_owed_withdrawals(root, kind):   # 欠撤（文件头「按条复用失灵」）：撤不掉整道退 75，不算计划
        return PLAN_SERVICE_UNAVAILABLE_EXIT
    if break_is_set("verdict-key-with-entry-file"):
        # 弄坏开关：每条请求的键另加它入口所在的整份文件（闭包外的改动——同文件里没人调的函数、注释——也让它作废）：自证 reuse-three-questions 的 ② 红
        requests = [Request(request.name, request.entry,
                            [*request.tokens, f"file:{os.path.normpath(os.path.join(request.root, request.entry.split('::', 1)[0]))}"])
                    for request in requests]
    planned, library = plan_requests(root, kind, requests)
    if sample_failure == "sentinel-reuse-failed":
        add_sample_sentinel_failure(root, planned, library)
    return print_plan(root, planned, library, plan_file)


TEST_PLAN_WORKERS_FILE_VARIABLE = "ADMISSION_TEST_PLAN_WORKERS_FILE"   # 只给自证：写一行这一趟分了几个子进程、算了几条、攒没攒闭包、fork 之前算了几条、冻了多少对象


def test_plan_resolve(indices):
    """算 indices 那几条测试函数：→ [(序号, compiled 或 not-compiled, (键, 组成清单, 拒算行) 或 None)]。编得进的照 command_test_plan 拼的请求算键。"""
    context, functions, request_of = TEST_PLAN_SHARED["context"], TEST_PLAN_SHARED["functions"], TEST_PLAN_SHARED["request_of"]
    library = function_units()
    resolved = []
    for index in indices:
        function = functions[index]
        try:
            context.rust_closure(*function.closure_arguments)
        except library.UsageError:
            resolved.append((index, "not-compiled", None))
            continue
        unit_key = compute_unit_key(context, request_of(function))
        # 这一条的 Rust 闭包只在判编不编与算键之间复用，算完键就放掉（不然一趟几千条的闭包全攒在 rust_closures 里）
        if not break_is_set("test-plan-closures-kept"):
            context.rust_closures.clear()
        line_ids = TEST_PLAN_SHARED.get("line_ids")
        composition = unit_key.composition if line_ids is None else composition_packed(unit_key.composition, line_ids)
        resolved.append((index, "compiled", (unit_key.key, composition, unit_key.refusals)))
    return resolved


def test_plan_private_bytes():
    """这个进程此刻写过的私有页（/proc/self/smaps_rollup 的 Private_Dirty，抄过的页都算）的字节数；读不出交 0。"""
    try:
        with open("/proc/self/smaps_rollup", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("Private_Dirty:"):
                    return int(line.split()[1]) * 1024
    except (OSError, ValueError, IndexError):
        return 0
    return 0


def test_plan_resolve_chunk(indices):
    """进程池的一件：算完放掉这一块留下的循环引用；→ (test_plan_resolve 交的, 设了 ADMISSION_TEST_PLAN_WORKERS_FILE 时这个进程的私有页字节数)。"""
    import gc
    resolved = test_plan_resolve(indices)
    gc.collect()
    return resolved, test_plan_private_bytes() if os.environ.get(TEST_PLAN_WORKERS_FILE_VARIABLE) else 0


def test_plan_parent_memory_shared(context, prewarmed):
    """fork 之前（文件头 test-plan「内存」）：铺开工作区全部 Rust 单元的组成清单行、源文件行与字面串，建行表（单元行加上父进程先算的那一条的组成清单）；词串进 sys.intern、行号共用一个对象。
    → 这一步的数 {prefilled: 个, tokens: 个}。弄坏开关 test-plan-lines-not-prefilled：不铺；test-plan-tokens-unshared：不换；test-plan-compositions-whole：不建行表。"""
    import types
    facts = dict(prefilled=0, tokens=0)
    lines = {}
    if not break_is_set("test-plan-lines-not-prefilled"):
        for closure_root, workspace in context.workspaces.items():
            for rust_file in workspace.files.values():
                closure_literals(context, closure_root, types.SimpleNamespace(order=rust_file.units))
                for unit in rust_file.units:
                    text = UNIT_LINE_TEXTS.get(("", unit))
                    if text is None:
                        text = UNIT_LINE_TEXTS[("", unit)] = unit.text()
                    lines[text] = None
                    facts["prefilled"] += 1
    for _index, _state, computed in prewarmed:
        if computed is not None:
            lines.update(dict.fromkeys(computed[1]))
    if not break_is_set("test-plan-compositions-whole"):
        table = list(lines)
        TEST_PLAN_SHARED.update(line_table=table, line_ids={line: position for position, line in enumerate(table)})
    if not break_is_set("test-plan-tokens-unshared"):
        numbers = {}
        for workspace in context.workspaces.values():
            for rust_file in workspace.files.values():
                rust_file.texts[:] = [sys.intern(text) for text in rust_file.texts]
                rust_file.lines[:] = [numbers.setdefault(line, line) for line in rust_file.lines]
                rust_file.pairs = {numbers.setdefault(opening, opening): numbers.setdefault(closing, closing) for opening, closing in rust_file.pairs.items()}
                facts["tokens"] += len(rust_file.texts)
    return facts


def test_plan_parent_objects_immortal():
    """进程里全部对象（gc.get_objects 与它们顺着够得到的，连同函数的代码对象里的常量）的引用计数设为永生值（文件头 test-plan「内存」）：解释器不写永生对象的引用计数，
    子进程读它们不抄页。→ 改了几个。这些对象到进程退出都不放（这一趟只活到写完计划）。永生值与判法是 CPython 3.12 在 64 位上的（_Py_IMMORTAL_REFCNT 是 32 位全 1，
    引用计数字段的低 32 位）：别的解释器版本不做，交 0。弄坏开关 test-plan-objects-not-immortal：不做。"""
    if break_is_set("test-plan-objects-not-immortal") or sys.implementation.name != "cpython" or sys.version_info[:2] != (3, 12) or sys.maxsize < (1 << 32):
        return 0
    import ctypes
    import gc
    import types
    immortal = 0xFFFFFFFF
    field_at, referents_of, reference_count, code_type = ctypes.c_uint32.from_address, gc.get_referents, sys.getrefcount, types.CodeType
    changed = 0
    todo = gc.get_objects()
    while todo:
        item = todo.pop()
        if reference_count(item) >= immortal:
            continue
        field_at(id(item)).value = immortal
        changed += 1
        if type(item) is code_type:
            todo += (item.co_consts, item.co_names, item.co_varnames, item.co_freevars, item.co_cellvars)
        else:
            todo += referents_of(item)
    return changed


def test_plan_settle_chunk(positions):
    """进程池的一件：这几条查库、存组成清单（VerdictLibrary.lookup 与 store_composition，客户端在这个进程里另连一条）：→ ([(下标, (决定, 原因))], 服务不可用时客户端的退出码或 None)。"""
    library, keyed = TEST_PLAN_SHARED["library"], TEST_PLAN_SHARED["keyed"]
    settled = []
    try:
        for position in positions:
            request, unit_key = keyed[position]
            settled.append((position, library.lookup(request, unit_key)))
            library.store_composition(unit_key, request.name)
    except SystemExit as error:   # 客户端已在标准错误打过 ✗ 与出路；退出码交回父进程照退，不让异常对象过进程边界
        return settled, error.code
    return settled, None


def test_plan_settled_lookups(library, keyed):
    """test-plan 并行那一趟：逐条查库、存组成清单分进子进程（文件头 test-plan「内存」）：→ {下标: (决定, 原因)}；别的调用方与本进程里算的交空。弄坏开关 test-plan-lookups-in-parent：交空。"""
    if not TEST_PLAN_SHARED.get("resolved_in_children") or not keyed or break_is_set("test-plan-lookups-in-parent"):
        return {}
    import concurrent.futures
    import multiprocessing
    jobs = TEST_PLAN_SHARED["resolved_in_children"]
    TEST_PLAN_SHARED.update(library=library, keyed=keyed)
    size = max(1, len(keyed) // (jobs * 16))
    chunks = [list(range(offset, min(offset + size, len(keyed)))) for offset in range(0, len(keyed), size)]
    settled = {}
    with concurrent.futures.ProcessPoolExecutor(max_workers=min(jobs, len(chunks)), mp_context=multiprocessing.get_context("fork")) as pool:
        for part, exit_code in pool.map(test_plan_settle_chunk, chunks):
            if exit_code is not None:
                raise SystemExit(exit_code)
            settled.update(part)
    TEST_PLAN_SHARED["lookups_in_children"] = len(settled)
    return settled


def test_plan_resolve_all(context, functions, request_of):
    """pending 的那几条分到每台默认重活线程数个子进程里算（文件头 test-plan「并行」），交 {序号: UnitKey}，同时把 state 写回 compiled / not-compiled。"""
    pending = [index for index, function in enumerate(functions) if function.state == "pending"]
    try:
        jobs = registered_heavy_threads(os.environ)
    except InputManifestError as error:
        if not break_is_set("test-plan-jobs-fallback-one"):   # 弄坏开关：取不了照 1 个子进程算
            raise UnitKeyError(f"取不了每台默认重活线程数（{error}），test-plan 不退回 1 个子进程照算：照 .claude/heavy-thread-default 文件头写好登记，"
                               "或 unset 写坏的 SINGLEFS_THREAD_CAP 再跑") from error
        jobs = 1
    jobs = max(1, min(jobs, len(pending)))
    TEST_PLAN_SHARED.update(context=context, functions=functions, request_of=request_of)
    test_plan_memoized_readers()
    serial = jobs == 1 or break_is_set("test-plan-single-process")
    computed_before_fork, frozen_before_fork, private_peak = 0, 0, 0
    memory_facts, immortal_objects = dict(prefilled=0, tokens=0), 0
    if serial:
        resolved = test_plan_resolve(pending)
    else:
        import concurrent.futures
        import gc
        import multiprocessing
        # fork 之前父进程先算一条：判法闭包（脚本索引）、工具链行、Cargo.lock、解释器版本与外部程序摘要建在父进程里，子进程照抄、不各建一份；
        #   铺开全部单元与共用行号、再把对象标成永生（文件头 test-plan「内存」）；再冻进永久代：子进程的回收器不走它们，不因回收把父进程的页逐页抄成私有的。
        #   不解冻：plan_requests 查库那一趟还要 fork，进程写完计划就退
        prewarmed = not break_is_set("test-plan-parent-not-prewarmed")
        resolved = test_plan_resolve(pending[:1]) if prewarmed else []
        rest = pending[len(resolved):]
        memory_facts = test_plan_parent_memory_shared(context, resolved)
        if prewarmed:
            gc.collect()
        immortal_objects = test_plan_parent_objects_immortal()
        if prewarmed:
            gc.freeze()
        computed_before_fork, frozen_before_fork = len(resolved), gc.get_freeze_count()
        TEST_PLAN_SHARED["resolved_in_children"] = jobs
        size = max(1, len(rest) // (jobs * 16))
        chunks = [rest[offset:offset + size] for offset in range(0, len(rest), size)]
        with concurrent.futures.ProcessPoolExecutor(max_workers=jobs, mp_context=multiprocessing.get_context("fork")) as pool:
            for part, private_bytes in pool.map(test_plan_resolve_chunk, chunks):
                resolved += part
                private_peak = max(private_peak, private_bytes)
    if len(resolved) != len(pending):
        raise UnitKeyError(f"test-plan 分出去 {len(pending)} 条、收回 {len(resolved)} 条")
    if os.environ.get(TEST_PLAN_WORKERS_FILE_VARIABLE):
        with open(os.environ[TEST_PLAN_WORKERS_FILE_VARIABLE], "w", encoding="utf-8") as handle:
            compacted = sum(1 for _index, _state, computed in resolved if computed is not None and isinstance(computed[1], bytes))
            handle.write(f"processes={0 if serial else jobs} items={len(pending)} rust-closures-left={len(context.rust_closures)} "
                         f"computed-before-fork={computed_before_fork} frozen-before-fork={frozen_before_fork} "
                         f"prefilled-units={memory_facts['prefilled']} tokens-shared={memory_facts['tokens']} immortal-objects={immortal_objects} compacted={compacted} "
                         f"reader-memo-hits={TEST_PLAN_SHARED.get('reader_memo_hits', 0)} worker-private-peak-bytes={private_peak}\n")
    keys = {}
    for index, state, computed in resolved:
        functions[index].state = state
        if computed is not None:
            keys[index] = UnitKey(*computed)
    return keys


REGISTERED_ITEMS_LIBRARY = os.path.join("..", "crash", "lib_crash_point_replay_items.py")   # 相对本文件所在的目录


def registered_test_functions(root):
    """test-plan --exclude-registered 排掉的 {(测试目标, 用例函数)}：登记成 crash-case: 的用例（stage-inputs.tsv 的 test=）与放量分项
    （scripts/crash/crash-amplification.sh 的 ITEMS 的测试目标与用例函数）。读法用 lib_crash_point_replay_items.py 的（合表那一份），读不到抛 RegistrationError。"""
    path = os.path.normpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), REGISTERED_ITEMS_LIBRARY))
    specification = importlib.util.spec_from_file_location("lib_crash_point_replay_items_for_admission", path)
    if specification is None or specification.loader is None or not os.path.isfile(path):
        raise RegistrationError(f"找不到 {path}：--exclude-registered 要用它读登记的崩溃枚举用例与放量分项")
    module = importlib.util.module_from_spec(specification)
    try:
        specification.loader.exec_module(module)
        found = set(module.crash_case_rows(root).values()) | set(module.pipeline_items(root).values())
    except OSError as error:
        raise RegistrationError(f"读不了登记的崩溃枚举用例与放量分项：{error}") from error
    return found


def command_test_plan(arguments):
    """test-plan：test-functions 与 verdict-plan 在一个进程里做（闭包只收一次）。每条测试函数一行
    「<名字><制表符><reuse|judge|refused|not-compiled><制表符><键或 -><制表符><目标号><制表符><目标根文件><制表符><libtest 全名><制表符><原因>」。"""
    exclude_registered = "--exclude-registered" in arguments
    arguments = [argument for argument in arguments if argument != "--exclude-registered"]
    values = option_pairs(arguments[4:], ("--root", "--feature", "--profile", "--token")) if len(arguments) >= 4 else None
    if values is None:
        print("  ✗ 用法：admission.py test-plan <项目根> <种类> <包> <计划文件> [--root <目录>] [--feature <包>/<特性>]… [--profile <名>] [--token <项>]… "
              "[--exclude-registered]")
        print("     → 怎么办：包写 Cargo.toml 的包名；--token 是每条请求另加的项（文件头「判定请求」：judge:、line: 这一类）")
        return EXIT_REGISTRATION_ERROR
    root, kind, package, plan_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    closure_relative = values.get("--root", ["."])[-1]
    closure_root = os.path.normpath(os.path.join(root, closure_relative))
    features, profile = values.get("--feature", []), values.get("--profile", [None])[-1]
    sample_failure = plan_failure_sample(root)
    if sample_failure != "sentinel-reuse-failed" and sample_failure is not None:
        return sample_plan_failure_exit("test-plan", root, sample_failure)
    if settle_owed_withdrawals(root, kind):   # 欠撤（文件头「按条复用失灵」）：撤不掉整道退 75，不算计划
        return PLAN_SERVICE_UNAVAILABLE_EXIT
    context = KeyContext(root)
    tokens_common = ([f"root:{closure_relative}"] if closure_relative != "." else []) + ([f"profile:{profile}"] if profile else [])
    tokens_common += [f"feature:{feature}" for feature in features] + values.get("--token", [])
    def whole_file(function):
        # 弄坏开关 test-key-with-whole-file：每条测试函数的键另加它所在的整份文件（按文件收：改同文件里别的测试也作废）
        if not break_is_set("test-key-with-whole-file"):
            return []
        return [f"file:{os.path.normpath(os.path.join(closure_relative, function.entry.split('::', 1)[0]))}"]
    def request_of(function):
        return Request(function.name, function.entry, [f"target:{function.target_id}", *tokens_common, *whole_file(function)])
    try:
        # 工作区读一次；编不编与键分进子进程算（文件头 test-plan「并行」）。读工作区与后面查库、写计划文件是一个进程里按原序做的
        functions = test_functions_of_package(context, package, closure_root, features, profile, resolve=False)
        if exclude_registered and not break_is_set("exclude-registered-ignored"):
            registered = registered_test_functions(root)
            for function in functions:
                if function.state == "pending" and any(function.target_id.rsplit(":", 1)[-1] == target
                                                      and (function.libtest_name == name or function.libtest_name.endswith("::" + name))
                                                      for target, name in registered):
                    function.state = "registered"
        keys_by_index = test_plan_resolve_all(context, functions, request_of)
    except (RegistrationError, UnitKeyError) as error:
        print(f"  ✗ 列不出 {package} 的测试函数：{error}")
        print("     → 怎么办：包名照 Cargo.toml 写；工作区读不出时先单跑 python3 scripts/gate/lib_function_units.py dump <根> 看它报什么")
        return EXIT_REGISTRATION_ERROR
    by_name = {function.name: function for function in functions}
    compiled = [index for index, function in enumerate(functions) if function.state == "compiled"]
    requests = [request_of(functions[index]) for index in compiled]
    TEST_PLAN_SHARED["file_digests"] = {}
    planned, library = plan_requests(root, kind, requests, context,
                                     unit_keys=[keys_by_index[index] for index in compiled])
    if sample_failure == "sentinel-reuse-failed":
        add_sample_sentinel_failure(root, planned, library)
    not_compiled = [function for function in functions if function.state == "not-compiled"]
    for function in not_compiled:
        print("\t".join([function.name, "not-compiled", "-", function.target_id, function.target_root, function.libtest_name, "这一组特性下 cfg 关着，不编"]))
    registered_functions = [function for function in functions if function.state == "registered"]
    for function in registered_functions:
        print("\t".join([function.name, "registered", "-", function.target_id, function.target_root, function.libtest_name,
                         "登记成 crash-case: 的用例或放量分项，交按块那一路，不在这里判"]))
    exit_code = print_plan(root, planned, library, plan_file,
                           lambda request: [by_name[request.name].target_id, by_name[request.name].target_root, by_name[request.name].libtest_name],
                           {"not-compiled": len(not_compiled), "registered": len(registered_functions), "ignored": sum(1 for function in functions if function.state == "ignored")})
    if os.environ.get(TEST_PLAN_WORKERS_FILE_VARIABLE):
        with open(os.environ[TEST_PLAN_WORKERS_FILE_VARIABLE], "a", encoding="utf-8") as handle:
            handle.write(f"file-digest-reads={TEST_PLAN_SHARED.get('file_digest_reads', 0)} lookups-in-children={TEST_PLAN_SHARED.get('lookups_in_children', 0)}\n")
    return exit_code


def command_verdict_record(arguments):
    values = option_pairs(arguments[4:], ("--how", "--summary", "--red-names")) if len(arguments) >= 4 else None
    if values is None or "--how" not in values:
        print("  ✗ 用法：admission.py verdict-record <项目根> <种类> <计划文件> <判绿的名字文件> --how <跑法> [--summary <一句>] [--red-names <判红的名字文件>]")
        print("     → 怎么办：计划文件取 verdict-plan / test-plan 那一趟写的；名字文件一行一个判绿的名字（--red-names 那一份一行一个判红的）；--how 写这一趟是怎么跑的（命令的样子，不写路径与时刻）")
        return EXIT_REGISTRATION_ERROR
    root, kind, plan_file, names_file = os.path.abspath(arguments[0]), arguments[1], arguments[2], arguments[3]
    try:
        with open(plan_file, encoding="utf-8") as handle:
            planned = [json.loads(line) for line in handle if line.strip()]
        with open(names_file, encoding="utf-8", errors="surrogateescape") as handle:
            green = {line.rstrip("\n") for line in handle if line.strip()}
        red_names = set()
        for red_file in values.get("--red-names", []):
            with open(red_file, encoding="utf-8", errors="surrogateescape") as handle:
                red_names |= {line.rstrip("\n") for line in handle if line.strip()}
    except (OSError, ValueError) as error:
        print(f"  ✗ 读不了计划文件或名字文件：{error}")
        print("     → 怎么办：计划文件取 verdict-plan 那一趟写的；名字文件一行一个名字")
        if values.get("--red-names") and not break_is_set(RED_RECORD_FAILURE_BREAK):
            # 记红没成（文件头「按条复用失灵」）：交了判红的名字却一条都没记，plan-failure 认这一句判红、照 --red 撤绿
            print(f"  ✗ {RED_RECORD_FAILURE_HEAD}这一趟交了 --red-names，判红的一条都没记（读不了计划文件或名字文件）")
            print(f"     → 怎么办：{RED_RECORD_FAILURE_HOWTO}")
        return EXIT_REGISTRATION_ERROR
    green -= red_names   # 两份都有的按红办（文件头 verdict-record）
    context = KeyContext(root)
    library = VerdictLibrary(root, kind)
    recorded = changed = refused = red_recorded = red_unrecorded = 0
    sample_red_failure = os.path.join(root, RED_RECORD_FAILURE_SAMPLE_MARKER)
    for item in planned:
        request = parse_request_line(item["request"])
        if request.name in red_names and item.get("key"):
            # 记红、撤绿（文件头「判定库」）：照计划里的键记，不重算（跑的过程中输入变了也只是多现判一次）；
            #   客户端退 75、3（SystemExit）照退，别的异常与没记成都是记红没成（文件头「按条复用失灵」）
            if os.path.isfile(os.path.join(root, RED_RECORD_UNAVAILABLE_SAMPLE_MARKER)):
                # 样本档造的：记红那一帧 prune-server 不可用，照客户端退 75（文件头「按条复用失灵」欠撤）
                print(f"  ✗ prune-server 不可用（样本档造的：{request.name} 记红那一帧连不上服务）")
                print(f"     → 怎么办：样本根里有 {RED_RECORD_UNAVAILABLE_SAMPLE_MARKER} 才走到这里，plan-failure --red 照退 {PLAN_SERVICE_UNAVAILABLE_EXIT} 记一笔欠撤；"
                      "真跑时遇到连不上，python3 prune-server/control check 看服务在不在，起不来报给主 agent")
                raise SystemExit(PLAN_SERVICE_UNAVAILABLE_EXIT)
            try:
                problem = (f"样本档造的记红没成（{sample_red_failure} 在）" if os.path.isfile(sample_red_failure)
                           else library.record_red(item["request"], request.name, item["key"], values["--how"][-1], values.get("--summary", [""])[-1]))
            except Exception as error:   # 记红那一帧出的任何异常都按记红没成办，不让同一把键的绿留着
                problem = f"写进服务时出错（{type(error).__name__}：{error}）"
            if problem and break_is_set(RED_RECORD_FAILURE_BREAK):
                # 弄坏开关：只打一行 !、照退 0，同一把键原有的绿留着
                print(f"  ! {request.name} 判红没记进判定库：{problem}（同一把键原有的绿还在，下一趟可能照复用）")
            elif problem:
                red_unrecorded += 1
                print(f"  ✗ {RED_RECORD_FAILURE_HEAD}{request.name}（键 {item['key'][:12]}）：{problem}")
                print(f"     → 怎么办：{RED_RECORD_FAILURE_HOWTO}")
            elif not break_is_set("red-verdict-not-recorded"):
                red_recorded += 1
            continue
        if request.name not in green:
            continue
        if not item.get("key"):
            refused += 1
            continue
        files = item.get("files") or {}
        unchanged = bool(files) and all((sha256_of_file(os.path.join(root, relative)) if os.path.isfile(os.path.join(root, relative)) else "不在") == digest
                                        for relative, digest in files.items())
        if not unchanged:
            unit_key = compute_unit_key(context, request)
            if unit_key.key != item["key"]:
                changed += 1
                print(f"  ! {request.name}：跑的过程中键变了（开跑 {item['key'][:12]}，跑完 {(unit_key.key or 'refused')[:12]}），不记")
                continue
        problem = library.record(item["request"], request.name, item["key"], values["--how"][-1], values.get("--summary", [""])[-1])
        if problem:
            print(f"  ! {request.name} 没记进判定库：{problem}（下一趟这一条照判）")
            continue
        recorded += 1
    sentinel_red = report_forced_rejudge(plan_file, green)
    print(f"RECORD recorded={recorded} changed={changed} refused={refused} red={red_recorded} red-unrecorded={red_unrecorded} library={library.directory}")
    return 1 if sentinel_red or red_unrecorded else 0


def report_forced_rejudge(plan_file, green):
    """复用哨兵收尾（文件头「复用哨兵」）：verdict-plan 存在计划文件旁边的状态在，就比强制现判那一条——判绿了与库里同为绿，记下；
    没判绿的（红了或没跑到）不在这里判，这一条的红由门禁这一趟自己报。打一行判定（在 RECORD 那一行之前），交红没红。"""
    checks = reuse_checks_module()
    state_path = plan_file + SENTINEL_STATE_SUFFIX
    if checks is None or not os.path.isfile(state_path):
        return False
    try:
        sentinel = checks.load_sentinel(state_path)
    except (OSError, ValueError, KeyError) as error:
        print(f"  ! 复用哨兵的状态读不了（{error}）：这一趟不比强制现判那一条")
        return False
    sentinel.observed = checks.observed_lines(state_path)
    if sentinel.forced_item is not None:
        if sentinel.forced_item in green:
            sentinel.observe(sentinel.forced_item, "强制现判", "绿")
        else:
            print(f"  · {sentinel.title()}：强制现判的 {sentinel.forced_item} 这一趟没判绿（红了或没跑到）：不在这里比，它的红由门禁这一趟自己报")
    problems = sentinel.finish(printer=print)
    for message, howto in problems:
        print(f"  ✗ {message}")
        print(f"     → 怎么办：{howto}")
    return bool(problems)


# ── 按条复用失灵（文件头「按条复用失灵」）：四处调用方共用这一份判法 ──
PLAN_FAILURE_SAMPLE_MARKER = os.path.join(UNIT_VERDICT_SAMPLE_DIRECTORY, "plan-failure")
PLAN_FAILURE_SAMPLE_FORMS = ("exit-1", "exit-75", "sentinel-reuse-failed")
# 记红没成（文件头「按条复用失灵」）：verdict-record 照计划里的键记红没成时打的红句头（plan-failure 认它）、样本档造它的标记、弄坏开关
RED_RECORD_FAILURE_HEAD = "判红没记进判定库："
RED_RECORD_FAILURE_FORM = re.compile(r"^\s*✗ (判红没记进判定库：.*)$")
RED_RECORD_FAILURE_SAMPLE_MARKER = os.path.join(UNIT_VERDICT_SAMPLE_DIRECTORY, "red-record-failure")
RED_RECORD_FAILURE_BREAK = "red-record-failure-swallowed"
RED_RECORD_FAILURE_HOWTO = ("调用方把这一趟的输出交 plan-failure，带 --red <根> <种类> <计划文件> <判红的名字文件>：它判红、撤掉这几把键原有的绿（文件头「按条复用失灵」）；"
                            "记红没成的原因看这一句，修好之后重跑这一格（下一趟这几条现判）")
# 欠撤（文件头「按条复用失灵」）：记红那一趟 prune-server 不可用（verdict-record 退 75）时 plan-failure --red 记下判红那几条的键与种类，下一趟 verdict-plan / test-plan
#   开跑先撤这几把键原有的绿。记在运行记录里（不是判定库）：上次红的清单那一层（lib_reuse_checks.py 的 last_runs_directory）底下的这个目录，再按库名的 sha256 分一层，一把键一份
OWED_WITHDRAWALS_DIRECTORY = "withdrawals-owed"
OWED_WITHDRAWAL_BREAK = "withdrawal-owed-unrecorded"
RED_RECORD_UNAVAILABLE_SAMPLE_MARKER = os.path.join(UNIT_VERDICT_SAMPLE_DIRECTORY, "red-record-unavailable")
OWED_WITHDRAWAL_UNAVAILABLE_SAMPLE_MARKER = os.path.join(UNIT_VERDICT_SAMPLE_DIRECTORY, "owed-withdrawal-unavailable")


def verdict_library_name(root, kind):
    """这一种类的判定库名（VerdictLibrary.directory 的同一种取法，不开库、不连服务）；不是 git 工作树交 None。"""
    common_directory = git_common_directory(root)
    return None if common_directory is None else os.path.join(common_directory, VERDICT_LIBRARY_DIRECTORY_NAME, kind_directory_name(kind))


def owed_withdrawals_directory(root, kind):
    """这份库的欠撤放哪：交 (目录, 库名, 取不到的原因)；不是 git 工作树交 (None, None, "")（开库那一步自己报）。"""
    library_name = verdict_library_name(root, kind)
    if library_name is None:
        return None, None, ""
    checks = reuse_checks_module()
    if checks is None:
        return None, library_name, f"同目录没有 {REUSE_CHECKS_LIBRARY}，上次红的清单那一层取不到"
    try:
        base = checks.last_runs_directory()
    except OSError as error:
        return None, library_name, str(error)
    digest = hashlib.sha256(library_name.encode("utf-8", "surrogateescape")).hexdigest()
    return os.path.join(base, OWED_WITHDRAWALS_DIRECTORY, digest), library_name, ""


def record_owed_withdrawals(withdrawal):
    """plan-failure --red 收到 verdict-record 退 75 时调（文件头「按条复用失灵」欠撤）：withdrawal 是（根、种类、计划文件、判红的名字文件），
    计划里这几个名字各有键的，一把键写一份 <键>.json（种类、库名、名字、键、日期），写完回读。交 (记下的条数, 没记成的原因或空串, 目录)。"""
    root, kind, plan_file, red_file = withdrawal
    try:
        with open(red_file, encoding="utf-8", errors="surrogateescape") as handle:
            names = {line.rstrip("\n") for line in handle if line.strip()}
        with open(plan_file, encoding="utf-8") as handle:
            planned = [json.loads(line) for line in handle if line.strip()]
    except (OSError, ValueError) as error:
        return 0, f"读不了计划文件或判红的名字文件：{error}", ""
    keys = {}
    for item in planned:
        name = parse_request_line(item["request"]).name
        if name in names and item.get("key"):
            keys[name] = item["key"]
    if not keys:
        return 0, "", ""
    directory, library_name, problem = owed_withdrawals_directory(os.path.abspath(root), kind)
    if directory is None:
        return 0, problem or f"{os.path.abspath(root)} 不是 git 工作树，判定库没有名字", ""
    written = 0
    for name, key in sorted(keys.items()):
        entry = {"kind": kind, "library": library_name, "name": name, "key": key, "recorded": judged_date()}
        path = os.path.join(directory, f"{key}.json")
        problem = write_atomically(path, json.dumps(entry, ensure_ascii=False) + "\n")
        if not problem and read_owed_withdrawal(path) != entry:
            problem = f"{path} 写完回读对不上"
        if problem:
            return written, problem, directory
        written += 1
    return written, "", directory


def read_owed_withdrawal(path):
    """一份欠撤记录；读不了、不是那几个字段交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            entry = json.load(handle)
    except (OSError, ValueError):
        return None
    return entry if isinstance(entry, dict) and all(isinstance(entry.get(field), str) and entry.get(field) for field in ("kind", "library", "name", "key")) else None


def settle_owed_withdrawals(root, kind):
    """verdict-plan / test-plan 算计划之前（文件头「按条复用失灵」欠撤）：这份库记着欠撤的，逐把键撤掉原有的绿（VerdictLibrary.withdraw_green，删完回读）、
    删掉那一份欠撤再核它不在了。全撤成交 0；有一份撤不掉（服务不可用、回读还是绿、记录读不了、删不掉）打 ✗ 与出路、交 75（整道退 75，不判绿）。"""
    if break_is_set(OWED_WITHDRAWAL_BREAK):   # 弄坏开关：不读欠撤
        return 0
    directory, library_name, problem = owed_withdrawals_directory(root, kind)
    if directory is None:
        if problem:
            print(f"  ! 欠撤：{kind} 的欠撤记录放哪取不到（{problem}），这一趟没查欠撤")
        return 0
    try:
        files = sorted(name for name in os.listdir(directory) if name.endswith(".json"))
    except FileNotFoundError:
        return 0
    except OSError as error:
        files, problem = [], f"列不了 {directory}：{error}"
    remove = (f"python3 scripts/prune-server/prune_server_client.py delete-prefix --dry-run unit-verdicts {kind_directory_name(kind)} {library_name} keys/<键>.json"
              "（先 --dry-run 数、再不带它删、再数到 0；记着红的那一份不删），删完再删 " + directory + " 里那一份")
    if not problem and files and os.path.isfile(os.path.join(root, OWED_WITHDRAWAL_UNAVAILABLE_SAMPLE_MARKER)):
        problem = "样本档造的服务不可用（撤欠撤那一帧连不上服务）"
    withdrawn, library = 0, None
    for file_name in files if not problem else []:
        path = os.path.join(directory, file_name)
        entry = read_owed_withdrawal(path)
        if entry is None or entry["library"] != library_name or entry["kind"] != kind or file_name != f"{entry['key']}.json":
            problem = f"{path} 读不了或不是这份库的欠撤"
            break
        try:
            library = library or VerdictLibrary(root, kind)
            withdrawn += library.withdraw_green(entry["name"], entry["key"])
            os.unlink(path)
        except (Exception, SystemExit) as error:   # 客户端退 75、3 与别的异常：这一把没撤成
            problem = f"键 {entry['key'][:12]}（{entry['name']}）：{type(error).__name__} {getattr(error, 'code', error)}"
            break
        if os.path.exists(path):
            problem = f"{path} 删完还在"
            break
    if problem:
        print(f"  ✗ 欠撤：{kind} 上一趟记红时 prune-server 不可用、记下的 {len(files)} 把键原有的绿撤不掉（{problem}），这一趟不算计划、整道退 75，不判绿")
        print(f"     → 怎么办：python3 prune-server/control check 看服务，服务好了再跑这一格（开跑先撤）；服务好了还撤不掉的照键手删：{remove}")
        return PLAN_SERVICE_UNAVAILABLE_EXIT
    if files:
        print(f"  · 欠撤：{kind} 上一趟记红时 prune-server 不可用、记下的 {len(files)} 把键，开跑先撤：撤掉原有的绿 {withdrawn} 条（回读都不是绿），欠撤记录已删，这几条现判")
    return 0
# prune-server 客户端退 3 的那一行（scripts/prune-server/prune_server_client.py 的 fail：没写服务地址、多机配置读不了、服务拒了这一帧这一类）：配置错，
#   不是服务不可用，也不是「记不进库」（文件头「按条复用失灵」）
PLAN_CLIENT_FAILURE_EXIT = 3
PLAN_CLIENT_FAILURE_FORM = re.compile(r"✗ prune-server 客户端：.*（退 3）\s*$")
PLAN_SERVICE_UNAVAILABLE_EXIT = 75   # prune-server 不可用（scripts/prune-server/prune_server_client.py 文件头「退出码」）：不是复用失灵（文件头「按条复用失灵」）
PLAN_SERVICE_UNAVAILABLE_HOWTO = ("python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好之后再跑这一格"
                                  "（服务不可用不是复用失灵：这一趟不现判、不判红也不判绿，整道退 75）")
PLAN_COMMANDS = ("verdict-plan", "test-plan")
# 复用哨兵的三种红句（scripts/gate/lib_reuse_checks.py 的 ReuseSentinel.finish；verdict-plan / verdict-record 打成「  ✗ <红句>」一行）
SENTINEL_FAILURE_FORMS = (
    (re.compile(r"^\s*✗ (复用失灵：.*)$"), "复用失灵",
     "这一项此刻的键与库里那一条判绿的记录相同、这一趟却现判了：看 scripts/gate/admission.py 文件头「复用哨兵」一段，查 plan_requests 与 VerdictLibrary.lookup 那一段为什么没读到它"),
    (re.compile(r"^\s*✗ (判定不稳：.*)$"), "判定不稳",
     "同一把键两次判出不同的结论：看 scripts/gate/admission.py 文件头「单元键」一段，判法依赖了键之外的输入就把它加进键或去掉依赖，再删掉库里那一条重判"),
    (re.compile(r"^\s*✗ (复用哨兵.*强制现判没起作用.*)$"), "强制现判没起作用",
     "抽中强制现判的那一条这一趟还是复用了：看 scripts/gate/admission.py 文件头「复用哨兵」一段与 plan_requests 里先问 forced 的那一句"),
)


def plan_failure_sample(root):
    """门禁样本档造的失灵（文件头「按条复用失灵」）：<根>/.unit-verdict-sample/plan-failure 不在交 None；在就交它写的那一样（写的不是认得的两样也照交，调用方退 2）。"""
    try:
        with open(os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER), encoding="utf-8") as handle:
            return handle.read().strip()
    except OSError:
        return None


def sample_plan_failure_exit(command, root, form):
    """样本档的 exit-1：不算键、不查库、退 1；exit-75：不连服务、退 75（装 prune-server 不可用）；标记写坏了退 2。"""
    marker = os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER)
    if form not in PLAN_FAILURE_SAMPLE_FORMS:
        print(f"  ✗ 样本档标记 {marker} 写的是「{form}」，认不出")
        print(f"     → 怎么办：只写一行 {' 或 '.join(PLAN_FAILURE_SAMPLE_FORMS)}（文件头「按条复用失灵」）；真仓里不该有这份文件")
        return EXIT_REGISTRATION_ERROR
    if form == "exit-75":
        print(f"  ✗ 样本档造的服务不可用：{marker} 写着 exit-75，{command} 不连 prune-server、退 75")
        print("     → 怎么办：这是门禁样本档装的 prune-server 不可用（文件头「按条复用失灵」），调用方要照「服务不可用」退 75；真仓里不该有这份文件，有就删掉")
        return PLAN_SERVICE_UNAVAILABLE_EXIT
    print(f"  ✗ 样本档造的失灵：{marker} 写着 exit-1，{command} 不算键、不查库、退 1")
    print("     → 怎么办：这是门禁样本档的假失灵（文件头「按条复用失灵」），调用方要把这一趟判红；真仓里不该有这份文件，有就删掉")
    return 1


def add_sample_sentinel_failure(root, planned, library):
    """样本档的 sentinel-reuse-failed：照常算完，再添一条复用哨兵的「复用失灵」红句（print_plan 照它退 1）。"""
    item = planned[0][0].name if planned else "（这一趟没有请求）"
    library.sentinel_problems = list(getattr(library, "sentinel_problems", [])) + [(
        f"复用失灵：{item} 键相同却现判了（样本档造的假哨兵：{os.path.join(root, PLAN_FAILURE_SAMPLE_MARKER)} 写着 sentinel-reuse-failed）",
        "这是门禁样本档的假失灵（文件头「按条复用失灵」），调用方要把这一趟判红；真仓里不该有这份文件，有就删掉")]


def plan_failure_findings(command, exit_code, text):
    """那一趟按条复用失灵在哪：[(红句, 出路)]；没失灵交 []。"""
    findings = []
    for line in text.split("\n"):
        for pattern, label, howto in SENTINEL_FAILURE_FORMS:
            match = pattern.match(line)
            if match is not None:
                findings.append((f"复用哨兵报红（{label}）：{match.group(1).strip()}", howto))
                break
    if findings or command not in PLAN_COMMANDS:
        return findings
    failure_lines = [line.strip() for line in text.split("\n") if line.strip().startswith("✗ ")]
    shown = failure_lines[-1] if failure_lines else " ".join(line.strip() for line in text.strip().split("\n")[-2:]) or "没有输出"
    if exit_code != 0:
        findings.append((f"scripts/gate/admission.py {command} 退 {exit_code}（{shown[:300]}）",
                         f"看 scripts/gate/admission.py 文件头子命令「{command}」那一条：退 2 是登记或用法错（请求文件照文件头「判定请求」写），"
                         "退 1 是复用哨兵或样本档造的失灵（文件头「复用哨兵」「按条复用失灵」），别的退出码是它自己崩了，单跑同一条命令看它报什么"))
    elif not re.search(r"^PLAN ", text, re.M):
        findings.append((f"scripts/gate/admission.py {command} 退 0 却没打 PLAN 那一行",
                         f"看 scripts/gate/admission.py 文件头子命令「{command}」那一条（末行 PLAN judge=… reuse=… refused=…）与 print_plan，单跑同一条命令看它打了什么"))
    return findings


def red_record_failure_findings(exit_code, text, withdrawal):
    """记红没成（文件头「按条复用失灵」）：verdict-record 那一趟判红的几条没记进库 ⇒ [(红句, 出路)]，并照 withdrawal（--red 给的 根、种类、计划文件、
    判红的名字文件）撤掉这几把键原有的绿；记红都成了、这一趟没有判红的（只交判绿那一支记不进库，下一趟照判）交 []。
    认记红没成：输出里有 verdict-record 的「✗ 判红没记进判定库：」，或交了 --red、名字文件不空、退非 0 而输出里没有 RECORD 那一行（没跑完）。"""
    marked = [match.group(1).strip() for match in (RED_RECORD_FAILURE_FORM.match(line) for line in text.split("\n")) if match is not None]
    finished = re.search(r"^RECORD ", text, re.M) is not None
    names = set()
    if withdrawal is not None:
        try:
            with open(withdrawal[3], encoding="utf-8", errors="surrogateescape") as handle:
                names = {line.rstrip("\n") for line in handle if line.strip()}
        except OSError as error:
            marked.append(f"读不了判红的名字文件 {withdrawal[3]}：{error}")
    if not marked and (finished or exit_code == 0 or not names):
        return []
    said = marked[0] if marked else f"verdict-record 退 {exit_code}、没跑完（输出里没有 RECORD 那一行），判红的 {len(names)} 条记没记进库不知道"
    if len(marked) > 1:
        said += f"（另有 {len(marked) - 1} 句同样的）"
    if withdrawal is None:
        return [(f"{said}；调用方没交 --red，同一把键原有的绿撤不了，整道判红",
                 "调用方交 plan-failure 时带上 --red <根> <种类> <计划文件> <判红的名字文件>（文件头「按条复用失灵」），这一趟就撤得掉；"
                 "撤之前下一趟会照复用那几把键原有的绿，先修好调用方，再重跑这一格")]
    withdrawn, kept, library_name = withdraw_red_greens(*withdrawal, names)
    if not kept:
        return [(f"{said}；同一把键原有的绿已撤掉 {withdrawn} 条，下一趟这几条现判",
                 "记红没成的原因看这一句（服务的写帧被拒、调用方的名字文件或计划文件不对、verdict-record 被停在半路），修好再重跑这一格")]
    shown = "；".join(f"{name}：{reason}" for name, reason in kept[:3])
    remove = (f"python3 scripts/prune-server/prune_server_client.py delete-prefix --dry-run unit-verdicts {kind_directory_name(withdrawal[1])} {library_name} keys/<键>.json"
              if library_name else "python3 scripts/prune-server/prune_server_client.py delete-prefix --dry-run unit-verdicts <种类> <库目录> keys/<键>.json")
    return [(f"{said}；同一把键原有的绿也撤不掉（{len(kept)} 条：{shown}），下一趟会照复用，整道判红",
             f"服务好了之后照计划文件里那几条的键逐条删掉原有的绿：{remove}（先 --dry-run 数、再不带它删、再数到 0），再重跑这一格；"
             "删之前不许当这几条判过绿")]


def withdraw_red_greens(root, kind, plan_file, _red_file, names):
    """记红没成时改走撤绿（文件头「按条复用失灵」）：计划里这几个名字各自的键，库里记着绿的删掉（VerdictLibrary.withdraw_green）。
    交 (撤掉的条数, 没撤成的 [(名字, 原因)], 库名或 "")；读不了计划、开不了库的整份算没撤成。拒算（计划里没有键）的那几条库里没有它的绿，不算。"""
    try:
        with open(plan_file, encoding="utf-8") as handle:
            planned = [json.loads(line) for line in handle if line.strip()]
    except (OSError, ValueError) as error:
        return 0, [("整份计划", f"读不了计划文件 {plan_file}：{error}")], ""
    keys = {}
    for item in planned:
        name = parse_request_line(item["request"]).name
        if name in names and item.get("key"):
            keys[name] = item["key"]
    try:
        library = VerdictLibrary(os.path.abspath(root), kind)
    except SystemExit as error:   # LibraryUnavailable 与客户端退出（75、3）
        return 0, [(name, f"开不了库（退 {error.code}）") for name in sorted(keys)] or [("整份", f"开不了库（退 {error.code}）")], ""
    withdrawn, kept = 0, []
    for name, key in sorted(keys.items()):
        try:
            withdrawn += library.withdraw_green(name, key)
        except (Exception, SystemExit) as error:   # 客户端退 75、3 与别的异常：这一条没撤成，接着撤下一条
            kept.append((name, f"键 {key[:12]}：{type(error).__name__} {getattr(error, 'code', error)}"))
    return withdrawn, kept, library.directory


def command_plan_failure(arguments):
    """plan-failure <调用方> <子命令> <退出码> [--red <根> <种类> <计划文件> <判红的名字文件>] [<输出文件>…]（文件头「按条复用失灵」）。"""
    if len(arguments) < 3 or arguments[1] not in (*PLAN_COMMANDS, "verdict-record") or not re.fullmatch(r"[0-9]+", arguments[2]) \
            or (arguments[3:4] == ["--red"] and (arguments[1] != "verdict-record" or len(arguments) < 8)):
        print("  ✗ 用法：admission.py plan-failure <调用方> <verdict-plan|test-plan|verdict-record> <退出码> [--red <根> <种类> <计划文件> <判红的名字文件>] [<输出文件>…]")
        print("     → 怎么办：输出文件给那一趟 admission.py 的标准输出与标准错误落的文件；--red 只跟 verdict-record，给那一趟交的根、种类、计划文件与 --red-names 那一份（文件头「按条复用失灵」）")
        return EXIT_REGISTRATION_ERROR
    caller, command, exit_code = arguments[0], arguments[1], int(arguments[2])
    withdrawal, output_paths = (arguments[4:8], arguments[8:]) if arguments[3:4] == ["--red"] else (None, arguments[3:])
    texts = []
    for path in output_paths:
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
        except OSError as error:
            texts.append(f"✗ 读不了那一趟的输出 {path}：{error}")
    if exit_code == PLAN_SERVICE_UNAVAILABLE_EXIT and not break_is_set("plan-unavailable-as-failure"):
        # prune-server 不可用不是复用失灵（文件头「按条复用失灵」）：退 75，调用方照「服务不可用」退 75
        shown = [line.strip() for line in "\n".join(texts).split("\n") if line.strip().startswith("✗ ")]
        print(f"  ✗ {caller}：prune-server 不可用（scripts/gate/admission.py {command} 退 75{'：' + shown[-1][:300] if shown else ''}），这一趟没判、不是复用失灵")
        print(f"     → 怎么办：{PLAN_SERVICE_UNAVAILABLE_HOWTO}")
        if command == "verdict-record" and withdrawal is not None and not break_is_set(OWED_WITHDRAWAL_BREAK):
            # 欠撤（文件头「按条复用失灵」）：判红那几条记红时服务不可用，同一把键原有的绿还在；记下键与种类，下一趟开跑先撤（弄坏开关 withdrawal-owed-unrecorded 下不记）
            written, problem, directory = record_owed_withdrawals(withdrawal)
            if problem:
                print(f"  ✗ {caller}：判红那几条的欠撤没记全（记下 {written} 条；{problem}），同一把键原有的绿还在，下一趟可能照复用")
                print("     → 怎么办：服务好了之后照计划文件里那几条的键逐条删掉原有的绿（python3 scripts/prune-server/prune_server_client.py delete-prefix --dry-run "
                      "unit-verdicts <种类> <库名> keys/<键>.json，先数、再删、再数到 0），删完之前不许当这几条判过绿；欠撤记不下的原因看括号里那一句")
            elif written:
                print(f"  · {caller}：判红的 {written} 条记红时服务不可用，欠撤（键与种类）记在 {directory}，下一趟开跑先撤这几把键原有的绿、撤不掉整道退 75")
        return PLAN_SERVICE_UNAVAILABLE_EXIT
    lines = "\n".join(texts).split("\n")
    client_failures = [index for index, line in enumerate(lines) if PLAN_CLIENT_FAILURE_FORM.search(line)]
    if exit_code == PLAN_CLIENT_FAILURE_EXIT and client_failures and not break_is_set("client-failure-swallowed"):
        # 客户端退 3（配置错、请求被拒）：三条子命令都判红，verdict-record 也不当「记不进库」放过；弄坏开关 client-failure-swallowed 下不认这一行
        shown = lines[client_failures[-1]].strip()
        remedy = next((line.strip() for line in lines[client_failures[-1] + 1:client_failures[-1] + 3] if line.strip().startswith("→ 怎么办：")), "")
        print(f"  ✗ {caller}：prune-server 客户端退 3（配置错或请求被拒，不是服务不可用，也不是复用失灵；scripts/gate/admission.py {command}：{shown[:300]}），这一趟判红")
        print(f"     {remedy or '→ 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「退出码」那一行配好（本机要有服务的 socket，第二台要有多机配置里的服务地址），再重交同一条命令'}")
        return 1
    red_findings = [] if command != "verdict-record" or break_is_set(RED_RECORD_FAILURE_BREAK) \
        else red_record_failure_findings(exit_code, "\n".join(texts), withdrawal)
    for message, howto in red_findings:
        # 记红没成（文件头「按条复用失灵」）：不归弄坏开关 plan-failure-not-red（撤绿照做、照判红）；弄坏开关 red-record-failure-swallowed 下退 0
        print(f"  ✗ {caller}按条复用失灵：{RED_RECORD_FAILURE_HEAD}{message.removeprefix(RED_RECORD_FAILURE_HEAD)}")
        print(f"     → 怎么办：{howto}")
    findings = plan_failure_findings(command, exit_code, "\n".join(texts))
    if not findings:
        return 1 if red_findings else 0
    if break_is_set("plan-failure-not-red"):
        # 弄坏开关：认出失灵也不判红（四处调用方退回全部现判、不判红）
        for message, _howto in findings:
            print(f"  ! {caller}按条复用失灵（弄坏开关 plan-failure-not-red：退回全部现判、不判红）：{message}")
        return 1 if red_findings else 0
    for message, howto in findings:
        print(f"  ✗ {caller}按条复用失灵：{message}；这一趟照旧全部现判跑完，结论判红（不能复用就判红）")
        print(f"     → 怎么办：{howto}")
    return 1


# ── 按条复用的共用段（文件头子命令 test-functions-request、plan-skip-names、plan-green-names）：门禁 crash-point-replay 按条复用的几格与
#    scripts/crash/crash-point-replay.sh 按条复用的那一段都只调这几条，种类、请求、跳过与记库的名单不另写第二份 ──
TEST_FUNCTIONS_VERDICT_KIND_SUFFIX = "test-functions"
TEST_FUNCTIONS_PACKAGE = "singlefs-verification"
TEST_FUNCTIONS_PROFILE = "release"
TEST_FUNCTIONS_JUDGE_ENTRY = "scripts/gate/lib_libtest_result_lines.py::recordable_results"


def test_functions_request_lines(stage_file_name, judge_root, environment=None):
    """按条复用的请求：种类、包与交给 test-plan 的参数，一行一样（文件头 test-functions-request）。environment 不给取本进程的环境
    （调用方在交给 cargo 的同一份环境里起 test-functions-request）：里面设了的改判定域的变量逐个打 --token variable:<名>=<值>。"""
    arguments = ["--profile", TEST_FUNCTIONS_PROFILE, "--token", f"judge:{TEST_FUNCTIONS_JUDGE_ENTRY}", "--token", f"judge-root:{judge_root}"]
    for token in domain_variable_tokens(os.environ if environment is None else environment):
        arguments += ["--token", token]
    return [f"kind\t{stage_file_name}:{TEST_FUNCTIONS_VERDICT_KIND_SUFFIX}", f"package\t{TEST_FUNCTIONS_PACKAGE}"] + [f"argument\t{argument}" for argument in arguments]


def command_test_functions_request(arguments):
    if len(arguments) != 2 or not arguments[0] or "/" in arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py test-functions-request <阶段文件名> <判法根>")
        print("     → 怎么办：阶段文件名写 crash-point-replay.sh（不带目录）；判法根写判法 lib_libtest_result_lines.py 所在的仓根（门禁写它自己所在的仓）")
        return EXIT_REGISTRATION_ERROR
    for line in test_functions_request_lines(arguments[0], arguments[1]):
        print(line)
    return 0


def read_plan_rows(plan_output, target_roots=None):
    """test-plan 的输出里逐条测试函数那几行（七列）；target_roots 给了只留第五列（目标根文件）在里面的。读不了抛 OSError。"""
    rows = []
    with open(plan_output, encoding="utf-8", errors="surrogateescape") as handle:
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if len(columns) >= 7 and (target_roots is None or columns[4] in target_roots):
                rows.append(columns)
    return rows


def plan_skip_names(rows, except_roots=None):
    """要 --skip 的 libtest 全名：reuse、not-compiled、registered 的，减去同名而要现判（judge、refused）的；照 C 字节序（文件头 plan-skip-names）。
    except_roots 给了（不起的目标根文件）：它们的 reuse、not-compiled、registered 行不收，现判的同名照旧看全部行。弄坏开关 skip-names-of-dropped-targets-kept：不看 except_roots。"""
    if break_is_set("skip-names-of-dropped-targets-kept"):
        except_roots = None
    skipped = {columns[5] for columns in rows if columns[1] in ("reuse", "not-compiled", "registered") and (not except_roots or columns[4] not in except_roots)}
    kept = {columns[5] for columns in rows if columns[1] in ("judge", "refused")}
    return sorted(skipped - kept, key=lambda name: name.encode("utf-8", "surrogateescape"))


def command_plan_skip_names(arguments):
    target_roots, except_files, positional, index, malformed = [], [], [], 0, False
    while index < len(arguments):
        if arguments[index] in ("--target-root", "--except-target-roots-file"):
            if index + 1 >= len(arguments):
                malformed = True
                break
            (target_roots if arguments[index] == "--target-root" else except_files).append(arguments[index + 1])
            index += 2
        else:
            positional.append(arguments[index])
            index += 1
    if malformed or len(positional) != 1 or len(except_files) > 1:
        print("  ✗ 用法：admission.py plan-skip-names <test-plan 的输出> [--target-root <目标根文件>]… [--except-target-roots-file <文件>]")
        print("     → 怎么办：输出给 test-plan 那一趟的标准输出落的文件；--target-root 照 test-plan 的第五列写（例 tests/alpha.rs），几个就给几次；"
              "--except-target-roots-file 至多给一次，文件里一行一个不起的目标根文件")
        return EXIT_REGISTRATION_ERROR
    try:
        rows = read_plan_rows(positional[0], set(target_roots) if target_roots else None)
        except_roots = None
        if except_files:
            with open(except_files[0], encoding="utf-8", errors="surrogateescape") as handle:
                except_roots = {line.rstrip("\n") for line in handle if line.strip()}
    except OSError as error:
        print(f"  ✗ 读不了 test-plan 的输出或不起的测试目标清单：{error}")
        print("     → 怎么办：给 test-plan 那一趟标准输出落的文件（文件头 test-plan）；--except-target-roots-file 指的文件要在")
        return EXIT_REGISTRATION_ERROR
    for name in plan_skip_names(rows, except_roots):
        print(name)
    return 0


def plan_green_names(rows, accounting_lines):
    """（判绿可记库的名字，[(名字, 不记的原因)]）：清单行按（目标根文件，libtest 全名）对到计划里 judge 的那一行（文件头 plan-green-names）。"""
    by_binary = {(columns[4], columns[5]): columns[0] for columns in rows if columns[1] == "judge"}
    green, withheld = [], []
    for line in accounting_lines:
        columns = line.rstrip("\n").split("\t")
        name = by_binary.get(tuple(columns[1:3])) if len(columns) >= 3 else None
        if name and columns[0] == "green":
            green.append(name)
        elif name and columns[0] == "withheld" and len(columns) >= 4:
            withheld.append((name, columns[3]))
    return green, withheld


def command_plan_green_names(arguments):
    if len(arguments) != 3:
        print("  ✗ 用法：admission.py plan-green-names <test-plan 的输出> <清单文件> <不记的落进的文件>")
        print("     → 怎么办：清单文件给 lib_libtest_result_lines.py recordable 那一趟的输出（文件头 plan-green-names）")
        return EXIT_REGISTRATION_ERROR
    try:
        rows = read_plan_rows(arguments[0])
        with open(arguments[1], encoding="utf-8", errors="surrogateescape") as handle:
            green, withheld = plan_green_names(rows, handle.readlines())
        with open(arguments[2], "w", encoding="utf-8", errors="surrogateescape") as handle:
            handle.write("".join(f"{name}\t{reason}\n" for name, reason in withheld))
    except OSError as error:
        print(f"  ✗ 读不了 test-plan 的输出或清单文件、写不了不记的那一份：{error}")
        print("     → 怎么办：三个参数照文件头 plan-green-names 给；这一趟一条都不记，下一趟照判")
        return EXIT_REGISTRATION_ERROR
    for name in green:
        print(name)
    return 0


def command_gate_reuse(arguments):
    if len(arguments) not in (2, 3) or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py gate-reuse <项目根> <登记键> [<请求文件>]")
        print("     → 怎么办：登记键照 .claude/gate.d/stage-inputs.tsv 的第一列写，例 crates-mutation-replay.sh。")
        return EXIT_GATE_MUST_RUN
    try:
        exit_code, reason, to_judge = gate_reuse(os.path.abspath(arguments[0]), arguments[1], arguments[2] if len(arguments) == 3 else None)
    except Exception as error:  # 模块自己出错一律按要跑处理：出错就跳过等于把这道门禁关掉
        print(f"判不出来：准入模块出错（{type(error).__name__}：{error}），按要跑处理")
        return EXIT_GATE_MUST_RUN
    print(reason)
    for name in to_judge:
        print(f"  judge\t{name}")
    return exit_code


def command_replay_row_inputs(arguments):
    """replay.sh 按行复用调：复跑表每一行打「<前三列><制表符><行键或 refused><制表符><单元数><制表符><要不要卡 0|1><制表符><说明>」；算不出整份退 2，✗ 与出路打在 stdout。"""
    if not arguments:
        print("  ✗ 用法：admission.py replay-row-inputs <项目根> [--gpu 0|1] [--library <库名>] [--compositions <目录>] [实验号…]")
        print("     → 怎么办：第一个参数给项目根，后面可以跟几个实验号（只算那几个实验的行）")
        return EXIT_REGISTRATION_ERROR
    root, rest = os.path.abspath(arguments[0]), list(arguments[1:])
    gpu_switch, library_directory, compositions = None, None, None
    wanted = []
    while rest:
        option = rest.pop(0)
        if option in ("--gpu", "--library", "--compositions") and rest:
            value = rest.pop(0)
            if option == "--gpu":
                gpu_switch = value
            elif option == "--library":
                library_directory = value
            else:
                compositions = value
        else:
            wanted.append(option)
    row_store = None
    if library_directory:
        row_store, problem = file_library_store(library_directory)
        if row_store is None:
            print(f"  ✗ --library {library_directory}：{problem}")
            print(f"     → 怎么办：给 replay-row 的库名 <git common-dir>/{VERDICT_LIBRARY_DIRECTORY_NAME}/replay-row（文件头「library-files」）")
            return EXIT_REGISTRATION_ERROR
    try:
        keys = replay_row_keys(root, tuple(wanted), gpu_switch)
    except (OSError, RegistrationError) as error:
        print(f"  ✗ 复跑表每一行的行键算不出：{error}")
        print("     → 怎么办：单跑 python3 scripts/gate/admission.py replay-row-inputs . 看它报什么；工具链起不来的先修好 rustc -vV 与 cargo -V")
        return EXIT_REGISTRATION_ERROR
    for identity, unit_key, uses_gpu in keys:
        unit_count = sum(1 for line in unit_key.composition if line.count("\t") == 3 and ":" in line.split("\t")[2])
        gpu_column = "1" if uses_gpu else "0"
        if unit_key.key is None:
            note = "拒算：" + "；".join(line.split("\t", 1)[1].replace("\t", " ") for line in unit_key.refusals)
            print(f"{identity}\trefused\t{unit_count}\t{gpu_column}\t{note}")
            continue
        note = ""
        if row_store is not None:
            note = replay_row_change_note(row_store, identity, unit_key)
        if compositions:
            problem = write_atomically(os.path.join(compositions, unit_key.key + ".tsv"), "".join(line + "\n" for line in unit_key.composition))
            note = note or (f"组成清单没写成：{problem}" if problem else "")
        print(f"{identity}\t{unit_key.key}\t{unit_count}\t{gpu_column}\t{note}")
    return 0


def replay_row_change_note(row_store, identity, unit_key):
    """这一行上一次判过的那一条（服务里 items/<前三列的 sha256> 记着键）与今天的组成清单比：键相同交空串，不同交变了的单元。服务连不上客户端退 75。"""
    index = row_store.read(f"items/{hashlib.sha256(identity.encode('utf-8', 'surrogateescape')).hexdigest()}")
    if index is None:
        return "库里没有这一行判过的记录"
    earlier_key = index.decode("utf-8", "surrogateescape").strip()
    if earlier_key == unit_key.key:
        return ""
    stored = row_store.read(f"keys/{earlier_key}/composition")
    if stored is None:
        return f"键对不上（上一次 {earlier_key[:12]}，那一份没留组成清单）"
    earlier = [line for line in stored.decode("utf-8", "surrogateescape").splitlines() if line.strip()]
    notes, total = changed_composition(earlier, unit_key.composition)
    return f"键对不上（上一次 {earlier_key[:12]}），变了 {total} 处：{'；'.join(notes)}"


def command_gate_preconditions(arguments):
    if len(arguments) != 2 or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py gate-preconditions <阶段所在的仓根> <阶段文件名>")
        print("     → 怎么办：阶段里写成 python3 <仓根>/scripts/gate/admission.py gate-preconditions <仓根> \"$(basename \"$0\")\"")
        return EXIT_REGISTRATION_ERROR
    root, stage = arguments
    try:
        stage_rows = rows_of_key(read_registration_rows(root), stage)
        if not stage_rows:
            raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {stage} 这一行，判不出这一道开跑前要什么")
        missing = [] if break_is_set("skip-gate-preconditions") else judge_gate_preconditions(root, stage_rows)
    except RegistrationError as error:
        print(f"  ✗ {stage} 的准入登记有错：{error}")
        print(f"     → 怎么办：在 {REGISTRATION_TABLE} 给 {stage} 登记一行，或改那一行（门禁行第三列只认 command=、readwrite=、probe=、environment=，"
              "写法见 scripts/gate/admission.py 文件头）；python3 scripts/gate/admission.py --selftest 的最后一格点得出登记表哪一行写错")
        return EXIT_REGISTRATION_ERROR
    except Exception as error:  # 判不出前提就不开跑：出错就放行等于把前提这一道关掉
        print(f"  ✗ 判不出 {stage} 的前提：准入模块出错（{type(error).__name__}：{error}）")
        print("     → 怎么办：单跑 python3 scripts/gate/admission.py gate-preconditions <仓根> <阶段文件名> 看报错，修准入模块或登记表那一行")
        return EXIT_GATE_PRECONDITION_MISSING
    if missing:
        why = "；".join(row.comment for row in stage_rows if row.comment) or "那一行没写注释"
        print(f"  ✗ 前提没齐，{stage} 不开跑：共 {len(missing)} 条（登记在 {REGISTRATION_TABLE} 的 {stage} 那一行第三列）")  # gate-lint:summary
        for description in missing:
            print(f"       {description}")  # gate-lint:detail
        print(f"     → 怎么办：把这几条办齐再跑这一道（这一道为什么要它们：{why}）；"
              "前提写错了就改登记表那一行，不许绕过。这一道判红、不记本次未跑：前提没齐时它一格都没判")
        return EXIT_GATE_PRECONDITION_MISSING
    return 0

def command_keys(arguments):
    if len(arguments) != 1:
        say("  ✗ 用法：admission.py keys <项目根>")
        say("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    seen = []
    for row in read_registration_rows(arguments[0]):
        if EXPERIMENT_KEY_FORM.match(row.key) and row.key not in seen:
            seen.append(row.key)
    for key in seen:
        print(key)
    return 0


def replay_row_key_problems(root):
    """复跑表每一行的准入键，连同 replay-row-keys 判红的三类：返回 ([ReplayRow], 登记了准入的键集合, 写法与登记的说明, 被几行共用的 [(键, 行数)])。"""
    rows, problems = replay_rows_with_keys(root)
    registered = {row.key for row in read_registration_rows(root) if EXPERIMENT_KEY_FORM.match(row.key)}
    for row in rows:
        if row.explicit and row.key not in registered:
            problems.append(f"复跑表「{row.identity}」那一行指明的准入键 {row.key} 没在 {REGISTRATION_TABLE} 登记")
    counts = {}
    for row in rows:
        if row.key in registered:
            counts[row.key] = counts.get(row.key, 0) + 1
    shared = [(key, count) for key, count in counts.items() if count > 1]
    return rows, registered, problems, shared


def command_replay_row_keys(arguments):
    """replay.sh 开跑前调：复跑表每一行打「<前三列><制表符><准入键><制表符><产物头谁写><制表符><登记了准入：yes|no>」；判错退 2，✗ 与出路打在 stdout。"""
    if len(arguments) != 1:
        print("  ✗ 用法：admission.py replay-row-keys <项目根>")
        print("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    rows, registered, problems, shared = replay_row_key_problems(arguments[0])
    if problems:
        print(f"  ✗ 复跑表按行的准入键有 {len(problems)} 处不对：")  # gate-lint:summary
        for problem in problems:
            print(f"       {problem}")  # gate-lint:detail
        print(f"     → 怎么办：照上面每一条改 {REPLAY_SCRIPT} 里 ADMISSION_ROW_KEYS 那一段（键、复跑表那一行的前三列原样抄、产物头谁写 device 或 replay），"
              f"指明的键在 {REGISTRATION_TABLE} 各登记一行它那一行读的输入；写法见 {os.path.join('scripts', 'gate', 'admission.py')} 文件头「复跑表按行的准入键」")
    for key, count in shared:
        print(f"  ✗ {key} 登记了准入，而这张复跑表里它有 {count} 行：准入只按实验号比产物，分不清哪一行对哪一份")
        print("     → 怎么办：给每一行起一个带方式的准入键（照 .claude/gate.d/stage-inputs.tsv 的 E142/p8g-crates-dump 写），在准入模块里按行认键之后再登记："
              f"写进 {REPLAY_SCRIPT} 的 ADMISSION_ROW_KEYS（一行一条：键、那一行的前三列、产物头谁写），每个键在 {REGISTRATION_TABLE} 各登记一行")
    if problems or shared:
        return EXIT_REGISTRATION_ERROR
    for row in rows:
        print(f"{row.identity}\t{row.key}\t{row.header_writer}\t{'yes' if row.key in registered else 'no'}")
    return 0
