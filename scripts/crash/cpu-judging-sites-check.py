#!/usr/bin/env python3
# admission: always 判的是此刻 crates/ 的 Rust 源码与判器调用点登记表，几秒读完；两边有一边改了都要现判
# run-condition: none 只读仓里的 Rust 源码与 .claude/gate.d/cpu-judging-sites.tsv，除了 python3 之外没有环境要求
"""没打标记的 CPU 就地判：checker 档与 harness 档的源码里每一处调判器的地方，要么在判定段里，要么带了标记（课题 4「能走两段的都走两段」，
records/2026-10-13-复用键与全仓改名.md「课题 4」出口 ④：豁免表的核法 scripts/crash/gpu-exemptions-check.py 只核标记与表行对上，管不到没打标记的）。
门禁 checker-independence-and-sync 的 cpu-judging-sites 格调它。

判器（调用点按名字认，名字是一个标识符；`fn <名>` 的定义与 `use …;` 里的那一处不算调用点，注释与串字面量里的不算）：
  池级 checker            check_pool_image 起头的（singlefs_checker::walk::check_pool_image 一族）
  记录核对器              check_records 起头的（crates/singlefs-checker-tier/src/crash.rs 的 check_records、check_records_against）
  RefFS 允许集合比对      allowed_set_for_a_crash_in、crash_and_recover（crates/singlefs-reference-filesystem）
  登记表里的判器本体      judging-body 那几行的函数名：它们的函数体就是判法（把上面几样包一层），里面那几处调用不判，调它们的地方照判器调用点判
扫的范围：crates/singlefs-checker-tier/{src,tests}、crates/singlefs-harness/{src,tests} 下全部 .rs（target/ 除外）。

一处调用点算合规，有一样成立就行：
  ① 落在判定段里：登记表 judging-stage 那几行，写整份文件（可带 * 通配）就是那份文件里的每一处，写 <文件>::<函数名> 就是那个函数（连同它里面嵌套的）；
  ② 落在判器本体里：登记表 judging-body 那几行（<文件>::<函数名>）；
  ③ 包住它的某一个函数（最里层到最外层都算）前面写了标记：函数的属性、文档注释与签名之前那一串紧挨着的 // 注释行，连同属性与签名之间的注释行，
     有一行去掉行首空白之后是 `// gpu-exempt: <截名>`（不走 GPU 的那一截，截名与豁免表 .claude/gate.d/gpu-exemptions.tsv 对不对得上归 gpu-exemptions-check.py）
     或恰好是 `// cpu-reference-for-gpu-comparison`（GPU 对拍用例的 CPU 参照那一侧）；
  ④ 文件在登记表 excluded 那几行里：整份不判（harness 档池级 checker 自己的单测，手造一张镜像、断言判器的结论，不是产出再判；每行写为什么）；
  ⑤ 落在判器自测里：登记表 judge-self-test 那几行（<文件>::<函数名>，函数级，连同它里面嵌套的）：函数判的是判器自己的契约（手摆盘、手给根、给判器塞已知坏法），
     不是「产出再判」，所以不走两段也不打 gpu-exempt 标记。只登函数，不整份登：同一份文件里别的函数照判。
其余的每一处判红，逐处列「文件:行 函数 调 名字（哪一类）」。
判器本体的名字按名字认，所以有一条同名的例外：一份文件自己定义了一个与登记的判器本体同名的函数、而这份文件不是登记的那一份（实验二进制里局部的 evaluate_state 与 crash.rs 的同名），
那份文件里调这个名字调的是它自己的函数，不算判器本体的调用点（那个局部函数里调判器的地方照判）。

登记表 .claude/gate.d/cpu-judging-sites.tsv：一行「种类<TAB>文件或 文件::函数名<TAB># 为什么」，种类 judging-stage、judging-body、judge-self-test、excluded；
  文件是仓根起的路径，judging-stage 与 excluded 可带 * 通配（fnmatch），judging-body 与 judge-self-test 必须写 ::函数名。核法（problems 里 table_problems）：
  段数、种类认不认得、为什么至少 8 个字、同一个目标登记两次、文件（通配）一份都对不上（扫的范围里没有）、::函数名在那份文件里找不到定义；
  表读不了、表不在都算问题（表随仓走）。
切函数用 scripts/mutation/mutation-prune.py 的 tokenize 与 FileIndex（同一份切词：注释与空白去掉、串字面量是一个单元，函数项连同属性与文档注释）。
管不到的：判器换了名字、经函数指针或 trait 对象调（名字不出现在调用处）、宏展开出来的调用；标记写在真走 GPU 的那一截上、为什么写得对不对，靠 review。

用法：
    cpu-judging-sites-check.py check [仓根]    只核，核过打一句成功句（扫了几份 .rs、几处调用点、各按哪一样合规）；核不过逐处列、退 1
    cpu-judging-sites-check.py --selftest      拿造出来的小仓核每一条核法；CPU_JUDGING_SITES_CHECK_BREAK=<项> 时点名的格必须红
退出码：0 核过；1 有问题（逐处列）；2 用法错。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 selftest() 里那一格，自证打「✗ 自证 <格名>：…」）：
  CPU_JUDGING_SITES_CHECK_BREAK=unmarked-sites-accepted  ③ 之外的调用点不判红
      unmarked-site、wrapper-call、marker-elsewhere、self-test-other-function
  CPU_JUDGING_SITES_CHECK_BREAK=marker-anywhere-in-file  文件里哪里写了标记都算这份文件里每一处调用点合规（不看是不是写在包住它的函数前面）
      marker-elsewhere
  CPU_JUDGING_SITES_CHECK_BREAK=judging-bodies-not-followed  登记表里判器本体的名字不当判器认（调它的地方不判）
      clean（数出的调用点少一处）、wrapper-call、local-function-of-the-same-name、self-test-nested
  CPU_JUDGING_SITES_CHECK_BREAK=table-targets-unchecked  登记表的文件（通配）与 ::函数名对不对得上不核
      stale-exclusion、missing-function、missing-self-test-function
  CPU_JUDGING_SITES_CHECK_BREAK=judge-self-tests-not-followed  登记表里 judge-self-test 的函数不认（里面的调用点照判）
      clean、nested-function-marker、local-function-of-the-same-name、self-test-nested（这几格的样本里都带着没登记就会判红的自测函数）
  CPU_JUDGING_SITES_CHECK_BREAK=local-function-of-the-same-name-followed  文件自己定义了与判器本体同名的函数时，调它仍算判器本体的调用点
      local-function-of-the-same-name
  CPU_JUDGING_SITES_CHECK_BREAK=judge-self-test-covers-whole-file  judge-self-test 登了一个函数，那份文件里每个函数里的调用点都算合规（函数级退成整份）
      self-test-other-function
"""
import fnmatch
import importlib.util
import os
import re
import shutil
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("CPU_JUDGING_SITES_CHECK_BREAK", "")
TABLE = os.path.join(".claude", "gate.d", "cpu-judging-sites.tsv")
SCANNED_DIRECTORIES = ("crates/singlefs-checker-tier/src", "crates/singlefs-checker-tier/tests", "crates/singlefs-harness/src", "crates/singlefs-harness/tests")
JUDGE_FAMILIES = (("池级 checker", re.compile(r"^check_pool_image\w*$")), ("记录核对器", re.compile(r"^check_records\w*$")),
                  ("RefFS 允许集合比对", re.compile(r"^(allowed_set_for_a_crash_in|crash_and_recover)$")))
EXEMPT_MARKER_PREFIX = "// gpu-exempt:"
REFERENCE_MARKER = "// cpu-reference-for-gpu-comparison"
KINDS = {"judging-stage": "判定段", "judging-body": "判器本体", "judge-self-test": "判器自测", "excluded": "整份不判"}
FUNCTION_LEVEL_KINDS = ("judging-body", "judge-self-test")
MINIMUM_REASON_CHARACTERS = 8
PRUNE_LIBRARY = os.path.join(REPOSITORY_ROOT, "scripts", "mutation", "mutation-prune.py")


def broken(switch):
    return switch in BROKEN.split(",")


def prune_library():
    """mutation-prune.py 的切词与切函数项（按文件路径载入，它自己不起子进程）。"""
    specification = importlib.util.spec_from_file_location("mutation_prune_for_cpu_judging_sites", PRUNE_LIBRARY)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def scanned_files(root):
    found = []
    for top in SCANNED_DIRECTORIES:
        for directory, subdirectories, files in os.walk(os.path.join(root, top)):
            subdirectories[:] = sorted(name for name in subdirectories if name != "target")
            found += [os.path.relpath(os.path.join(directory, name), root) for name in sorted(files) if name.endswith(".rs")]
    return found


def read_table(root):
    """→ ([(行号, 种类, 文件模式, 函数名或 None)], 问题)。"""
    rows, found = [], []
    try:
        lines = open(os.path.join(root, TABLE), encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了判器调用点登记表 {TABLE}：{error}"]
    seen = {}
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        where = f"{TABLE}:{number}"
        fields = line.split("\t")
        if len(fields) != 3 or not fields[2].startswith("#"):
            found.append(f"{where} 要三段（种类、文件或 文件::函数名、# 为什么，制表符分隔），实际 {len(fields)} 段")
            continue
        kind, target, reason = fields[0].strip(), fields[1].strip(), fields[2][1:].strip()
        if kind not in KINDS:
            found.append(f"{where} 种类「{kind}」认不得，只认 {'、'.join(KINDS)}")
            continue
        if len(reason) < MINIMUM_REASON_CHARACTERS:
            found.append(f"{where} {target} 的为什么只有 {len(reason)} 个字，至少 {MINIMUM_REASON_CHARACTERS} 个字")
        if target in seen:
            found.append(f"{where} {target} 登记了两次（第 {seen[target]} 行已经有）")
            continue
        seen[target] = number
        pattern, _, function = target.partition("::")
        if kind in FUNCTION_LEVEL_KINDS and not function:
            found.append(f"{where} {KINDS[kind]}要写成 <文件>::<函数名>，实际「{target}」")
            continue
        if kind == "excluded" and function:
            found.append(f"{where} 整份不判的只写文件（可带 * 通配），不写 ::函数名，实际「{target}」")
            continue
        rows.append((number, kind, pattern, function or None))
    return rows, found


def preamble_lines(lines, index, function):
    """包住调用点的这个函数前面的那一串行（文件头「一处调用点算合规」③）：属性、文档注释与签名之前紧挨着的 // 注释行，连同函数项起头到 fn 那一行之间的行。"""
    tokens = index.tokens
    item_line = index.text.count("\n", 0, tokens[function.first][2])
    keyword_line = item_line
    for position in range(function.first, function.last):
        if tokens[position][0] == "ident" and tokens[position][1] == "fn":
            keyword_line = index.text.count("\n", 0, tokens[position][2])
            break
    first = item_line
    while first > 0 and lines[first - 1].lstrip().startswith("//"):
        first -= 1
    return lines[first:keyword_line + 1]


def has_marker(text_lines):
    for line in text_lines:
        stripped = line.strip()
        if (stripped.startswith(EXEMPT_MARKER_PREFIX) and stripped[len(EXEMPT_MARKER_PREFIX):].strip()) or stripped == REFERENCE_MARKER:
            return True
    return False


def use_declaration_positions(tokens):
    """`use …;` 里的词法单元下标（花括号里的一组也算，到同一层的分号为止）。"""
    positions, index = set(), 0
    while index < len(tokens):
        if tokens[index][0] == "ident" and tokens[index][1] == "use" and (index == 0 or tokens[index - 1][1] not in (".", "::")):
            depth, cursor = 0, index + 1
            while cursor < len(tokens):
                text = tokens[cursor][1]
                depth += {"{": 1, "}": -1}.get(text, 0)
                if text == ";" and depth <= 0:
                    break
                positions.add(cursor)
                cursor += 1
            index = cursor
        index += 1
    return positions


def judge_sites(root, rows, prune):
    """→ (问题, 计数)。计数：files、excluded、sites，以及各按哪一样合规的数。"""
    found, counts = [], dict(files=0, excluded=0, sites=0, stage=0, body=0, self_test=0, marked=0)
    bodies = {} if broken("judging-bodies-not-followed") else {function: pattern for _number, kind, pattern, function in rows if kind == "judging-body"}
    for relative in scanned_files(root):
        if any(kind == "excluded" and fnmatch.fnmatch(relative, pattern) for _number, kind, pattern, _function in rows):
            counts["excluded"] += 1
            continue
        counts["files"] += 1
        text = open(os.path.join(root, relative), encoding="utf-8", errors="replace").read()
        whole_file_stage = any(kind == "judging-stage" and function is None and fnmatch.fnmatch(relative, pattern) for _number, kind, pattern, function in rows)
        stage_functions = {function for _number, kind, pattern, function in rows if function and kind == "judging-stage" and fnmatch.fnmatch(relative, pattern)}
        body_functions = {function for _number, kind, pattern, function in rows if function and kind == "judging-body" and fnmatch.fnmatch(relative, pattern)}
        self_test_functions = set() if broken("judge-self-tests-not-followed") else {function for _number, kind, pattern, function in rows
                                                                                      if function and kind == "judge-self-test" and fnmatch.fnmatch(relative, pattern)}
        index = prune.FileIndex(relative, text)
        lines = text.split("\n")
        in_use = use_declaration_positions(index.tokens)
        file_has_marker = broken("marker-anywhere-in-file") and has_marker(lines)
        for position, token in enumerate(index.tokens):
            if token[0] != "ident":
                continue
            family = next((name for name, form in JUDGE_FAMILIES if form.match(token[1])), None)
            if family is None and token[1] in bodies:
                family = "判器本体"
            if family is None or position in in_use or (position > 0 and index.tokens[position - 1][1] == "fn"):
                continue
            if family == "判器本体" and not broken("local-function-of-the-same-name-followed") and not fnmatch.fnmatch(relative, bodies[token[1]]) \
                    and any(function.name == token[1] for function in index.functions):
                continue  # 这份文件自己定义了同名的函数、而它不是登记的那份：调的是它自己的那个，不是登记的判器本体
            counts["sites"] += 1
            containing = sorted((function for function in index.functions if function.first <= position < function.last), key=lambda function: function.last - function.first)
            if whole_file_stage or any(function.name in stage_functions for function in containing):
                counts["stage"] += 1
                continue
            if any(function.name in body_functions for function in containing):
                counts["body"] += 1
                continue
            if self_test_functions and (broken("judge-self-test-covers-whole-file") or any(function.name in self_test_functions for function in containing)):
                counts["self_test"] += 1
                continue
            if file_has_marker or any(has_marker(preamble_lines(lines, index, function)) for function in containing):
                counts["marked"] += 1
                continue
            if broken("unmarked-sites-accepted"):
                continue
            line = text.count("\n", 0, token[2]) + 1
            found.append(f"{relative}:{line} {containing[0].qualified if containing else '<函数外>'} 调 {token[1]}（{family}）没落在判定段里、包住它的函数前面也没有"
                         f"「{EXEMPT_MARKER_PREFIX} <截名>」或「{REFERENCE_MARKER}」")
    return found, counts


def table_problems(root, rows):
    found = []
    if broken("table-targets-unchecked"):
        return found
    files = scanned_files(root)
    prune = None
    for number, kind, pattern, function in rows:
        matched = [relative for relative in files if fnmatch.fnmatch(relative, pattern)]
        if not matched:
            found.append(f"{TABLE}:{number} {kind} 的「{pattern}」在 {'、'.join(SCANNED_DIRECTORIES)} 里一份文件都对不上（文件挪了、改名了就改这一行，没了就删这一行）")
            continue
        if function:
            prune = prune or prune_library()
            defined = any(item.name == function for relative in matched
                          for item in prune.FileIndex(relative, open(os.path.join(root, relative), encoding="utf-8", errors="replace").read()).functions)
            if not defined:
                found.append(f"{TABLE}:{number} {kind} 的 {pattern}::{function}：那份文件里没有 fn {function}（改名了就改这一行，没了就删这一行）")
    return found


def problems(root):
    rows, found = read_table(root)
    found += table_problems(root, rows)
    site_problems, counts = judge_sites(root, rows, prune_library())
    return found + site_problems, counts


def command_check(root):
    found, counts = problems(root)
    if found:
        for problem in found:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 判器调用点有 {len(found)} 处不合规（逐处列在上面；扫了 {counts['files']} 份 .rs，调用点 {counts['sites']} 处）")  # gate-lint:summary
        print(f"  → 怎么办：真跑实现、只能在 CPU 上判的那一截在函数上一行写「{EXEMPT_MARKER_PREFIX} <截名>」并在 .claude/gate.d/gpu-exemptions.tsv 登记同名一行（每加一行先问用户）；"
              f"GPU 对拍的 CPU 参照那一侧写「{REFERENCE_MARKER}」；能走两段的搬进判定段（从库里读回再判）；判定段与判器本体登记在 {TABLE}；"
              "登记表写坏的照上面那一句改；改完单跑 python3 scripts/crash/cpu-judging-sites-check.py check 到它退 0")
        return 1
    print(f"  ✓ 判器调用点都合规：扫了 {counts['files']} 份 .rs（登记表里整份不判的 {counts['excluded']} 份另算），调用点 {counts['sites']} 处："
          f"落在判定段 {counts['stage']}、判器本体里 {counts['body']}、判器自测里 {counts['self_test']}、包住它的函数带标记 {counts['marked']}")
    return 0


SELFTEST_TABLE_HEAD = "# 样本登记表\n"
SELFTEST_ROWS = (
    "judging-stage\tcrates/singlefs-checker-tier/src/crash_judge_*.rs\t# 判定段：从库里读回再判的那几份",
    "judging-stage\tcrates/singlefs-checker-tier/src/crash.rs::judge_read_back_range\t# 判定段：判从库里读回的一段状态",
    "judging-body\tcrates/singlefs-checker-tier/src/crash.rs::judge_crash_image\t# 判器本体：池级 checker 加记录核对器包一层",
    "judge-self-test\tcrates/singlefs-checker-tier/tests/crash_record_checker_contract.rs::a_hand_laid_disk_is_refused\t# 判器自测：手摆的盘断言记录核对器的结论",
    "excluded\tcrates/singlefs-harness/tests/checker_*.rs\t# 池级 checker 自己的单测：手造一张镜像断言判器的结论",
)
SELFTEST_SOURCES = {
    "crates/singlefs-checker-tier/src/crash_judge_tables.rs": "use singlefs_checker::walk::check_pool_image;\npub fn judge_one_block() { check_pool_image(&reader); }\n",
    "crates/singlefs-checker-tier/src/crash.rs": (
        "use singlefs_checker::walk::{check_pool_image, Other};\n"
        "/// 判器本体\npub fn judge_crash_image() { check_pool_image(&reader); check_records(&records); }\n"
        "pub fn judge_read_back_range() { judge_crash_image(); }\n"
        "// check_pool_image 在注释里不算\npub fn quotes() { let text = \"check_pool_image(\"; }\n"
        "fn check_pool_image_given_a_knob() {}\n"),
    "crates/singlefs-checker-tier/tests/crash_mount_observer.rs": (
        "// gpu-exempt: writable-mount\n#[test]\nfn the_mount_is_judged() { check_pool_image(&mounted); }\n"
        "// cpu-reference-for-gpu-comparison\nfn cpu_reference_side() { let verdicts = vec![1].iter().map(|_| check_records(&r)).count(); }\n"),
    "crates/singlefs-harness/tests/checker_known_bad_images.rs": "#[test]\nfn a_bad_image_is_refused() { check_pool_image(&bad); }\n",
    "crates/singlefs-checker-tier/tests/crash_record_checker_contract.rs": (
        "#[test]\nfn a_hand_laid_disk_is_refused() { check_records_against(&hand_laid, root); }\n"
        "fn another_judged_here() { let judged = 1; }\n"),
}


def selftest():
    work = tempfile.mkdtemp(prefix="cpu-judging-sites-check-selftest-")
    failures, cells = [], []
    try:
        def world(name, rows=SELFTEST_ROWS, extra=None):
            root = os.path.join(work, name)
            sources = dict(SELFTEST_SOURCES, **(extra or {}))
            for relative, source in sources.items():
                os.makedirs(os.path.dirname(os.path.join(root, relative)), exist_ok=True)
                with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                    handle.write(source)
            os.makedirs(os.path.join(root, ".claude", "gate.d"), exist_ok=True)
            with open(os.path.join(root, TABLE), "w", encoding="utf-8") as handle:
                handle.write(SELFTEST_TABLE_HEAD + "".join(row + "\n" for row in rows))
            return root

        def expect(cell, root, wanted, sites=None):
            cells.append(cell)
            found, counts = problems(root)
            if wanted is None and found:
                failures.append((cell, f"应当核过，实际 {found}"))
            elif wanted is not None and not any(wanted in problem for problem in found):
                failures.append((cell, f"应当报一处含「{wanted}」的问题，实际 {found}"))
            elif sites is not None and counts["sites"] != sites:
                failures.append((cell, f"应当数出 {sites} 处调用点，实际 {counts['sites']}（{counts}）"))

        # clean：判定段整份与按函数、判器本体里的、带两种标记的各一处，排除表里的不判；use、注释、串字面量、fn 定义不算调用点
        expect("clean", world("clean"), None, sites=7)
        expect("unmarked-site", world("unmarked-site", extra={"crates/singlefs-harness/src/history.rs": "pub fn violations_on() -> usize { check_pool_image(&image).len() }\n"}),
               "crates/singlefs-harness/src/history.rs:1 violations_on 调 check_pool_image（池级 checker）")
        expect("wrapper-call", world("wrapper-call", extra={"crates/singlefs-checker-tier/tests/crash_direct.rs": "#[test]\nfn judged_here() { judge_crash_image(); }\n"}),
               "crates/singlefs-checker-tier/tests/crash_direct.rs:2 judged_here 调 judge_crash_image（判器本体）")
        expect("marker-elsewhere", world("marker-elsewhere", extra={"crates/singlefs-checker-tier/tests/crash_other.rs":
                                                                   "// gpu-exempt: writable-mount\nfn somewhere_else() {}\n\nfn judged_without_marker() {\n    // gpu-exempt: writable-mount\n    allowed_set_for_a_crash_in(hi);\n}\n"}),
               "crash_other.rs:6 judged_without_marker 调 allowed_set_for_a_crash_in（RefFS 允许集合比对）")
        expect("nested-function-marker", world("nested-function-marker", extra={"crates/singlefs-checker-tier/tests/crash_nested.rs":
                                                                               "// cpu-reference-for-gpu-comparison\nfn outer() {\n    fn inner() { check_records_against(&r); }\n}\n"}), None)
        expect("local-function-of-the-same-name", world("local-function-of-the-same-name", extra={"crates/singlefs-checker-tier/src/bin/e_local_judge.rs":
                                                                                    "// gpu-exempt: experiment-measures-the-cpu-judge-itself\nfn judge_crash_image() { check_pool_image(&r); }\nfn caller() { judge_crash_image(); }\n"}), None, sites=8)
        expect("self-test-nested", world("self-test-nested", extra={"crates/singlefs-checker-tier/tests/crash_record_checker_contract.rs":
                                                                    "#[test]\nfn a_hand_laid_disk_is_refused() {\n    fn inside() { check_pool_image(&hand_laid); }\n    inside();\n}\n"}), None, sites=7)
        expect("self-test-other-function", world("self-test-other-function", extra={"crates/singlefs-checker-tier/tests/crash_record_checker_contract.rs":
                                                                                   "#[test]\nfn a_hand_laid_disk_is_refused() { check_records_against(&hand_laid, root); }\n"
                                                                                   "fn judged_a_real_output() { check_records_against(&produced, root); }\n"}),
               "crash_record_checker_contract.rs:3 judged_a_real_output 调 check_records_against（记录核对器）")
        expect("stale-exclusion", world("stale-exclusion", rows=SELFTEST_ROWS + ("excluded\tcrates/singlefs-harness/tests/pool_checker_*.rs\t# 池级 checker 自己的单测：手造镜像",)),
               "「crates/singlefs-harness/tests/pool_checker_*.rs」在")
        expect("missing-function", world("missing-function", rows=SELFTEST_ROWS + ("judging-stage\tcrates/singlefs-checker-tier/src/crash.rs::gone_away\t# 判定段：已经没了的函数",)),
               "crash.rs::gone_away：那份文件里没有 fn gone_away")
        expect("missing-self-test-function", world("missing-self-test-function", rows=SELFTEST_ROWS + ("judge-self-test\tcrates/singlefs-checker-tier/tests/crash_record_checker_contract.rs::gone_away\t# 判器自测：已经没了的函数",)),
               "crash_record_checker_contract.rs::gone_away：那份文件里没有 fn gone_away")
        expect("self-test-without-function", world("self-test-without-function", rows=SELFTEST_ROWS + ("judge-self-test\tcrates/singlefs-checker-tier/tests/crash_record_checker_contract.rs\t# 判器自测没写函数名",)),
               "判器自测要写成 <文件>::<函数名>")
        expect("unknown-kind", world("unknown-kind", rows=SELFTEST_ROWS + ("judged-elsewhere\tcrates/singlefs-checker-tier/src/crash.rs\t# 写错的种类名字",)), "种类「judged-elsewhere」认不得")
        expect("reason-too-short", world("reason-too-short", rows=SELFTEST_ROWS[:3] + ("excluded\tcrates/singlefs-harness/tests/checker_*.rs\t# 单测",)), "的为什么只有 2 个字")
        expect("body-without-function", world("body-without-function", rows=SELFTEST_ROWS + ("judging-body\tcrates/singlefs-checker-tier/src/crash_judge_tables.rs\t# 判器本体没写函数名",)),
               "判器本体要写成 <文件>::<函数名>")
        cells.append("table-missing")
        missing_root = world("table-missing")
        os.unlink(os.path.join(missing_root, TABLE))
        if not any("读不了判器调用点登记表" in problem for problem in problems(missing_root)[0]):
            failures.append(("table-missing", "登记表不在应当报一处「读不了判器调用点登记表」"))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for cell, failure in failures:
        print(f"  ✗ 自证 {cell}：{failure}")  # gate-lint:detail
    if failures:
        print(f"  ✗ 自证 {len(cells)} 格里 {len(failures)} 格判错")  # gate-lint:summary
        print("  → 怎么办：看 judge_sites、table_problems、read_table 那几条核法；CPU_JUDGING_SITES_CHECK_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ cpu-judging-sites-check 自证通过：判定段（整份与按函数）、判器本体里、判器自测里（函数级）、带 gpu-exempt 与 CPU 参照标记、外层函数带标记、排除表里的都核过，"
          f"use 与注释、串字面量、fn 定义不算调用点；没标记的、调判器本体的、标记写在别处的、排除表与按函数的登记对不上的、种类写错、为什么太短、"
          f"判器本体与判器自测没写函数名、登记表不在各报出来（查了 {len(cells)} 格）")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if arguments and arguments[0] == "check" and len(arguments) <= 2:
        return command_check(os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT)
    print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
    print("  → 怎么办：cpu-judging-sites-check.py check [仓根] | --selftest")
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
