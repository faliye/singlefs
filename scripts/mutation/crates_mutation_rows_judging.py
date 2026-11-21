"""crates-mutation-rows.py 拆出的一份。
判「抓没抓到」的那几个入口与它们调到的函数：libtest 结果行的取法、本次未跑与设备丢失的认法、基线参数与单条基线结局、单条变异的结局分类、起 cargo 的命令、拷每片源码、判一行与判一批、覆盖复用判据。只 import base。驱动（这一份与别的份）的代码不进行键。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import collections
import os
import re
import shutil
import tempfile
import time
from crates_mutation_rows_base import (CARGO_CONFIGURATION_DIRECTORY, DEVICE_LOST_LINE, DEVICE_LOST_LINE_OUTSIDE_SECTIONS, GPU_TEST_FEATURES,
    HeavyThreadDefaultError, IGNORED_WHEN_COPYING_INTO_EACH_SHARD, LIBTEST_SHOW_OUTPUT, LINK_FAILURE_LINE, LINK_FAILURE_LINKER, NOT_RUN_MARK,
    NOT_RUN_REASON_FIELD, NOT_RUN_TEST_FIELD, NOT_RUN_WORD, REUSABLE_VERDICT, STAGE, TABLE, VERDICTS_COUNTED, VERDICTS_UNCOUNTED, WORKER_STATE,
    baseline_cargo_environment, break_is, default_heavy_threads, libtest_result_lines, mutation_units, note, split_at_double_dash)


def anchor_broken_text(row, why):
    """锚点失效那一行的判定（run_stage 的「锚点失效」那一档与 --list-items 末行的逐行点名同一句）。"""
    return f"{TABLE}:{row.line_number} {row.name}：锚点失效——{why}（这一行没跑、不入库、不复用）"


def executed_set_is_current(keys, coverage):
    """覆盖记录里那一份执行过的单元（执行过的 fn 单元，连同点名测试闭包里不是 fn 的单元：常量、类型、静态项、宏、数据文件）此刻逐个还是那个指纹。"""
    if not isinstance(coverage, dict) or not coverage.get("complete"):
        return False
    references = [mutation_units.UnitReference(*item) for item in coverage.get("executed", [])]
    if not references:
        return False
    current = keys.current_fingerprints(references)
    return all(reference.fingerprint in current.get((reference.kind, reference.file, reference.name), set()) for reference in references)


def baseline_reuse_problem(record, components, keys, coverage, snapshot, environment):
    """覆盖复用判据（基线 ok 那一条复用不复用）：成立交 None，不成立交那一句。判据（按次序，头一条成立就复用）：
      覆盖记录里点名测试执行过的单元此刻逐个没变（executed_set_is_current）；或者点名测试的闭包键与记录里的相同。
    不按整树快照比：同一个文件里加一个没人调的函数、改别的测试，基线照样复用。弄坏开关 baseline-by-tree-snapshot：只按整树快照比。"""
    if record is None or record.get("verdict") != REUSABLE_VERDICT["baseline"]:
        return "没有基线 ok 的记录"
    if break_is(environment, "baseline-by-tree-snapshot"):
        return None if record.get("tree_snapshot") == snapshot else "源码树快照与上次基线那一份不同（弄坏开关 baseline-by-tree-snapshot）"
    if executed_set_is_current(keys, coverage):
        return None
    if record.get("test_closure") and record.get("test_closure") == components.test_closure:
        return None
    return "点名测试执行过的单元与它的闭包都与上次基线那一份不同"


def baseline_arguments(key, rows):
    """这一批基线的 cargo test 参数：键里的两段，加这一批各行过滤词的并集（有一行没有过滤词就一个都不带）。"""
    before, switches = key
    filters = []
    for row in rows:
        row_filters = [word for word in split_at_double_dash(row.arguments)[1] if not word.startswith("-")]
        if not row_filters:
            filters = []
            break
        filters += [word for word in row_filters if word not in filters]
    tail = list(switches) + filters
    return list(before) + (["--"] + tail if tail else [])


def row_libtest_selection(arguments):
    """一行 cargo 参数交给 libtest 的选择：(过滤词, 带不带 --exact, --skip=<名> 的那几个名)，写法同 baseline_arguments（`--` 之后不以 - 起头的是过滤词）。"""
    after = split_at_double_dash(arguments)[1]
    return ([word for word in after if not word.startswith("-")], "--exact" in after,
            [word.split("=", 1)[1] for word in after if word.startswith("--skip=")])


def libtest_selects(full_name, filters, exact, skips):
    """libtest 按这几样选不选得中这条用例：带 --exact 时名字要逐字相同，不带时是子串；没有过滤词就是全部，--skip 的照同一种比法剔掉。"""
    def matches(pattern):
        return full_name == pattern if exact else pattern in full_name
    if filters and not any(matches(word) for word in filters):
        return False
    return not any(matches(word) for word in skips)


def row_baseline_output(row, output, environment):
    """一批基线的输出里这一行看得见的那一份（文件头「基线」）：只留这一行自己的过滤词选得中的用例的结果行，别的结果行去掉。
    一批的过滤词是这一趟拼进来的各行的并集，拼进了哪几行随这一趟选了哪几行变；这样一行的基线结局与它单跑时相同，点名单跑与整张表对同一行判同一个结论。
    弄坏开关 baseline-batch-composition：照整批的输出判。结果词被隔开的结果行先接回一行再按名字选（joined_result_lines）。"""
    if break_is(environment, "baseline-batch-composition"):
        return output
    filters, exact, skips = row_libtest_selection(row.arguments)
    kept = []
    for line in joined_result_lines(output, environment).split("\n"):
        parsed = libtest_result_lines.result_line(line)
        if parsed is not None and not libtest_selects(parsed.full_name, filters, exact, skips):
            continue
        kept.append(line)
    return "\n".join(kept)


def joined_result_lines(output, environment=None):
    """结果词被测试中途打的行隔开的结果行接回一行：`test <名> ... <测试自己打的字>`，后面隔着别的输出，结局另起一行单独写（`ok`、`FAILED`、`ignored`，可带「, 理由」或用时）的，
    把结局接到起头那一行上、单独那一行去掉，别的行原样留着（libtest 打 `test <名> ... ` 与结局是两次写，别的线程的输出插在中间）。
    起头之后先来了别的 `test <名> ... ` 或整行结果行的，前一条认不出结局、不接；这一段另有没有起头可接的单独一行结局，接出来的都不认
    （同 lib_libtest_result_lines.py 的 account_binary），原文照旧交回。
    弄坏开关 split-result-line-unseen（environment 没给取进程环境）：不接，只认同一行写全的结果行。"""
    environment = os.environ if environment is None else environment
    if break_is(environment, "split-result-line-unseen"):
        return output
    lines = output.split("\n")
    pending = None
    joined_at, orphans = {}, 0
    for index, line in enumerate(lines):
        stripped = line.rstrip("\r")
        if libtest_result_lines.result_line(stripped) is not None:
            pending = None
            continue
        started = libtest_result_lines.SCATTERED_START_FORM.match(stripped)
        if started is not None and not stripped.startswith("test result: "):
            pending = (index, started.start("rest"))
            continue
        if libtest_result_lines.STANDALONE_OUTCOME_FORM.match(stripped):
            if pending is None:
                orphans += 1
            else:
                joined_at[pending[0]] = (index, pending[1])
                pending = None
    if orphans or not joined_at:
        return output
    outcome_lines = {outcome_index for outcome_index, _rest_start in joined_at.values()}
    kept = []
    for index, line in enumerate(lines):
        if index in outcome_lines:
            continue
        if index in joined_at:
            outcome_index, rest_start = joined_at[index]
            line = line.rstrip("\r")[:rest_start] + lines[outcome_index].strip()
        kept.append(line)
    return "\n".join(kept)


def results_by_test(expected, output, environment=None):
    """libtest 的结果行 `test <模块路径>::<名字> ... ok|FAILED|ignored` 里对上点名的那几条：{完整名: [结果…]}。
    #[should_panic] 的用例在名字与 ` ... ` 之间多一段 ` - should panic`；点名可以省掉模块路径；结果词被测试中途打的行隔开的先接回一行（joined_result_lines）。
    解析在 lib_libtest_result_lines.py（全仓一份）。"""
    return libtest_result_lines.results_of_named_test(expected, joined_result_lines(output, environment))


def output_tail(output, lines=8):
    return "\n".join(output.splitlines()[-lines:])


def failure_excerpt(output, full_name):
    """libtest 给这一条打的 `---- <名字> stdout ----` 那一段的头几行；没有就交 None。"""
    marker = f"---- {full_name} stdout ----"
    index = output.find(marker)
    if index < 0:
        return None
    return "\n".join(output[index:].splitlines()[:6])


def baseline_problem(row, returncode, output):
    """这一行的点名测试在基线输出里不是「恰好一个用例、结果都是 ok」时交那一句（接在「在没改坏的副本上」后面），是的交 None。"""
    results = results_by_test(row.expected, output)
    if len(results) == 1 and all(result == "ok" for result in next(iter(results.values()))):
        return None
    if len(results) > 1:
        return f"对上了 {len(results)} 个不同的用例（{'、'.join(sorted(results))}），判不出是哪一个；把点名写全到模块路径"
    if results:
        full_name, outcomes = next(iter(results.items()))
        return f"的结果是 {'、'.join(outcomes)}（{full_name}）"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        return f"一行都没出现：那一批编不过（退出码 {returncode}）"
    return f"一行都没出现（那一批退出码 {returncode}；参数筛不到它，或名字认不出）"


def with_shown_output(arguments):
    """变异那一趟的 cargo test 参数：libtest 参数里补 --show-output（插在第一个 `--` 之后；没有 `--` 就补 `-- --show-output`；已经有了不再补）。"""
    words = list(arguments)
    if "--" not in words:
        return words + ["--", LIBTEST_SHOW_OUTPUT]
    index = words.index("--")
    if LIBTEST_SHOW_OUTPUT in words[index + 1:]:
        return words
    return words[:index + 1] + [LIBTEST_SHOW_OUTPUT] + words[index + 1:]


def not_run_marks_in(output, environment=None):
    """输出里每一处 NOT_RUN_NO_GPU_ADAPTER 记号（文件头「本次未跑」）：[(test= 点名的名字，没写是 None, reason= 之后那一句，没写是空串)]，次序照出现的。
    记号的认法是门禁那一份库的 marker_pieces（整词、一行里每一处各是一截、颜色码先去掉），不要求记号在行首；一截里的 test=、reason= 是这一处自己的，不取同一行别处的。
    弄坏开关 not-run-line-start-only（environment 给了才认）：只认行首（行首空白不算）的那一处。"""
    start_only = break_is(environment or {}, "not-run-line-start-only")
    marks = []
    for line in output.splitlines():
        if "NOT_RUN_" not in line and "\x1b" not in line:
            continue
        for kind, piece in libtest_result_lines.marker_pieces(line):
            if kind != "not-run" or NOT_RUN_WORD.match(piece).group() != NOT_RUN_MARK or (start_only and not line.lstrip().startswith(piece)):
                continue
            named, reasoned = NOT_RUN_TEST_FIELD.search(piece), NOT_RUN_REASON_FIELD.search(piece)
            marks.append((named.group(1) if named else None, reasoned.group(1).strip() if reasoned else ""))
    return marks


def not_run_reason(expected, output, environment=None):
    """输出里有 NOT_RUN_NO_GPU_ADAPTER 记号时交接在「打了 NOT_RUN_NO_GPU_ADAPTER」后面括号里的那一句（文件头「本次未跑」），一处都没有交 None：
    test= 对上点名的测试（只比 :: 分开的末一段）的那一处先取，交它 reason= 之后那一句（没写交「没写为什么」）；
    都对不上的（崩溃枚举用例判段不跑时打的是 test=crash_state_enumeration::<项名>，项名不是测试名；记号没写 test=）照样交，写明 test= 是什么、对不上测试名：
    宁可记本次未跑，不记没红。弄坏开关 not-run-test-name-only（environment 给了才认）：只认 test= 对得上的。"""
    wanted = expected.split("::")[-1]
    marks = not_run_marks_in(output, environment)
    for test, reason in marks:
        if test is not None and test.split("::")[-1] == wanted:
            return reason or "没写为什么"
    if not marks or break_is(environment or {}, "not-run-test-name-only"):
        return None
    test, reason = marks[0]
    return f"那一处是 test={test}，对不上点名的测试名，照本次未跑记：{reason or '没写为什么'}" if test is not None else f"那一处没写 test=，对不上点名的测试名，照本次未跑记：{reason or '没写为什么'}"


def device_lost_line_of_test(full_name, output):
    """点名的测试自己那一段输出（named_test_stdout：libtest 的「---- <全名> stdout ----」那一段，full_name 是 libtest 结果行里的名字）里第一行行首 GPU_DEVICE_LOST 的行
    （文件头「本次未跑」）：交记号后面的字段原文（`test=… card=… reason=…`，不拆），没有交 None。别的用例那一段里的行不算。"""
    found = DEVICE_LOST_LINE.search(named_test_stdout(full_name, output))
    return None if found is None else found.group(1)


def device_lost_line_outside_sections(output):
    """不在任何一段（libtest_stdout_sections 的段外）里的第一行行首 GPU_DEVICE_LOST 的行，交记号后面的字段原文，没有交 None：
    打它的线程没带点名用例的输出捕获，行写到了真标准输出，归属不明。任何一段里的行（含别的用例那一段）不算。"""
    _sections, outside = libtest_stdout_sections(output)
    found = DEVICE_LOST_LINE_OUTSIDE_SECTIONS.search("\n".join(outside))
    return None if found is None else found.group(1)


def verdict_of_run(row, returncode, output, environment=None):
    """一条变异跑完之后判哪一档：(档, 那一条要打印的话)。environment 只看弄坏开关（notrunasmissed、not-run-test-name-only、not-run-line-start-only、device-lost-ignored、link-failure-as-invalid）。"""
    tail = output_tail(output)
    # 点名省掉模块路径时在输出里只许对上一个用例：对上两个就分不出红的是不是点名的那一个，按没判成记，不按「有一个红了」算抓到。
    results = results_by_test(row.expected, output, environment)
    if len(results) > 1:
        return "failure", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 在输出里对上了 {len(results)} 个不同的用例"
                           f"（{'、'.join(sorted(results))}），判不出红的是哪一个；把点名写全到模块路径\n{tail}")
    if returncode != 0 and any("FAILED" in outcomes for outcomes in results.values()):
        # 点名的测试红了、它自己那一段输出里、或所有段之外（归属不明）又有行首 GPU_DEVICE_LOST 的行：红的是设备丢失、不是断言，判不出变异有没有被抓，
        # 记本次未跑（文件头「本次未跑」）；显存不够而红的没有这一行、只在别的用例的段里有的，照记抓到
        lost, place = None, ""
        if not break_is(environment or {}, "device-lost-ignored"):
            lost, place = device_lost_line_of_test(next(iter(results)), output), "它自己那一段输出里"
            if lost is None:
                lost, place = device_lost_line_outside_sections(output), "所有 ---- <名> stdout ---- 段之外（归属不明）的输出里"
        if lost is not None:
            return "notrun", (f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} 红了，{place}有行首 GPU_DEVICE_LOST 的行"
                              f"（{lost}），红的是设备丢失、不是断言；不算抓到、也不算没红")
        return "caught", caught_text(row)
    # 点名的测试是 ok、又打了 NOT_RUN_NO_GPU_ADAPTER：没跑到被改坏的那一处，记本次未跑（文件头「本次未跑」），不记没红
    reason = not_run_reason(row.expected, output, environment) if results and not break_is(environment or {}, "notrunasmissed") else None
    if reason is not None and all(outcome == "ok" for outcomes in results.values() for outcome in outcomes):
        return "notrun", (f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} 打了 NOT_RUN_NO_GPU_ADAPTER（{reason}）后返回成功，"
                          "没跑到被改坏的那一处；不算抓到、也不算没红")
    if results:
        return "failure", f"{TABLE}:{row.line_number} {row.name}：{row.expected} 没红（退出码 {returncode}）\n{tail}"
    if (LINK_FAILURE_LINE in output and LINK_FAILURE_LINKER in output and "error[E" not in output
            and not break_is(environment or {}, "link-failure-as-invalid")):
        # 编过了、失败在链接（链接器报未定义的符号这一类）：不是替换文编不过，不记无效（那样会说这条变异今天零变异覆盖、叫人去改替换文）。
        # 多半是编译目录里别的命令编出的旧产物与这一趟对不上：删掉编译目录重跑这一行；没判到被改坏的那一处，也不是没红，走与本次未跑同一路（VERDICTS_UNCOUNTED）
        return "linkfailed", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 一行都没出现，失败在链接（输出里有 {LINK_FAILURE_LINE}、{LINK_FAILURE_LINKER}，没有 error[E<码>]；"
                              f"退出码 {returncode}），不是替换文编不过；删掉这一趟用的编译目录，重跑这一行\n{tail}")
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异（.claude/rules/mutation-sampling.md 第八类）。
        return "invalid", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 一行都没出现，输出里有 error 行"
                           f"（替换文编不过，或测试进程在跑到它之前被杀，或名字认不出；退出码 {returncode}）\n{tail}")
    return "failure", f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 没跑到（退出码 {returncode}）\n{tail}"


def caught_text(row, how=""):
    """抓到那一行；how 是「（现判：<栏>）」「（复用）」或「（拒算）」，判定本身在它前面（比对两趟的判定时把它去掉，selftest_verdict_view）。"""
    return f"  ✓ {row.name}：{row.expected} 红了{how}"


def single_full_name(expected, output):
    """输出里点名的测试恰好对上一个用例时交它的全名（带模块路径），不然交 None。"""
    results = results_by_test(expected, output)
    return next(iter(results)) if len(results) == 1 else None


def prepare_shard(worker_sequence, shard_root, context):
    """一个工作进程自己的一份源码副本加一个编译目录。第 1 片用共用的编译目录（跨轮热着），其余片 -w<片号> 自己冷编译一次，不从共用目录拷种子。
    ⚠️ 拷的时候不留原来的修改时刻（shutil.copy，不用 copy2）：编译目录跨轮复用，里面最后一次编的可能是上一轮改坏的源码、
    带着上一轮副本的 CARGO_MANIFEST_DIR；副本留着原来较早的修改时刻，cargo 就判它不用重编，没改坏的基线跑的是上一轮改坏的二进制
    （基线会报 attempt to subtract with overflow、读 litmus 读到上一轮已经删掉的副本路径）。
    修改时刻取拷的那一刻，每片头一次编译把工作区的几个 crate 重编一遍（外部依赖不在副本里，照旧热着）。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in context["copied"]:
        source = os.path.join(context["root"], item)
        if not os.path.exists(source):
            continue
        if item == CARGO_CONFIGURATION_DIRECTORY and break_is(context["environment"], "cargoconfignotcopied"):
            continue   # 弄坏开关（文件头）：不拷 .cargo/，副本里没有它
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD), copy_function=shutil.copy)
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy(source, target)
    shared = context["shared_target_directory"]
    return work, (shared if worker_sequence == 1 else f"{shared}-w{worker_sequence}")


def gpu_test_run(command, environment=None):
    """这一次起的是不是带 gpu 特性、真跑测试的 cargo test：`--` 之前开了上卡的特性（--features / -F 里有 GPU_TEST_FEATURES 里的名字或 <包>/<名字>，或 --all-features），
    而且不带 --no-run。收尾补 --test-threads（with_gpu_test_threads）与核用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）都照它认。
    弄坏开关 product-gpu-feature-unquiesced（environment 给了才认）：只认 gpu，产品 GPU 包的 tests-on-a-real-gpu 不算。"""
    if "cargo" not in command:
        return False
    words = command[command.index("cargo") + 1:]
    if "test" not in words:
        return False
    words = words[words.index("test") + 1:]
    before = words[:words.index("--")] if "--" in words else words
    if "--no-run" in before:
        return False
    features = set()
    for index, word in enumerate(before):
        if word in ("--features", "-F") and index + 1 < len(before):
            word = "--features=" + before[index + 1]
        elif word.startswith("-F") and len(word) > 2:
            word = "--features=" + word[2:]
        if word.startswith("--features="):
            features |= {feature.split("/")[-1] for feature in word[len("--features="):].replace(",", " ").split()}
    wanted = {"gpu"} if environment is not None and break_is(environment, "product-gpu-feature-unquiesced") else GPU_TEST_FEATURES
    return bool(features & wanted) or "--all-features" in before


def with_gpu_test_threads(command, environment, released_targets):
    """带 gpu 特性、真跑测试的 cargo test（gpu_test_run 认的那一种）的 libtest 参数收尾补 --test-threads（文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：
    `--` 之后已经有 --test-threads（--test-threads=N 或分开写）的原样不动、不重复补；没有的补 `--test-threads 1`，没有 `--` 的先补 `--`；
    cargo 参数点名的测试目标（--test <名>）全在放开名单里的补每台默认重活线程数
    （scripts/hosts/heavy_threads.py 的 default_heavy_threads；取不到打一行说明、照旧带 1）。只编不跑的、不带 gpu 特性的原样交回。
    released_targets 是调用方交的取名单的函数（init_worker 交的、照被判的根调 gpu_released_test_targets：本模块的 GPU_TEST_THREADS_RELEASED_TARGETS
    加验证包登记的只经卡池取卡的测试目标；名单不进判法单元的闭包）。
    补的是起命令时的参数，不进行键（文件头「行键」）。弄坏开关 gpu-test-threads-unadded：不补。"""
    if break_is(environment, "gpu-test-threads-unadded") or not gpu_test_run(command, environment):
        return list(command)
    words = list(command)
    test_index = words.index("test", words.index("cargo") + 1)
    separator = words.index("--") if "--" in words else None
    after = words[separator + 1:] if separator is not None else []
    if any(word == "--test-threads" or word.startswith("--test-threads=") for word in after):
        return words
    targets = mutation_units.parse_cargo_arguments(words[test_index + 1:separator] if separator is not None else words[test_index + 1:]).targets
    released = set(released_targets())
    threads = "1"
    if targets and all(kind == "test" and value in released for kind, value in targets):
        try:
            threads = str(default_heavy_threads(environment))
        except HeavyThreadDefaultError as error:
            note(f"  ! 取不出每台默认重活线程数（{error}），放开 --test-threads 1 的测试目标 {'、'.join(sorted(released))} 照旧带 --test-threads 1")
    return words + ([] if separator is not None else ["--"]) + ["--test-threads", threads]


def libtest_stdout_sections(output):
    """把 libtest 的输出切成 (段, 段外的行)：段是 [(段头那一行, 它的行)]，段外是不在任何一段里的行。
    形态（--show-output、默认的 pretty 格式）：先是实时打印的部分（「running N tests」「test <全名> ... ok|FAILED」，没被捕获的输出——
    不是点名用例这条线程派生的线程、子进程——也夹在这里），再是「successes:」下成功用例的段，再是「failures:」下失败用例的段与「failures:」名单，最后「test result:」。
    一段从「---- <全名> stdout ----」那一行起，到下一个段头、「successes:」「failures:」或「test result:」为止（这几行本身与之后的行算段外）。"""
    sections, outside, current = [], [], None
    for line in output.splitlines():
        if line.startswith("---- ") and line.endswith(" stdout ----"):
            current = []
            sections.append((line, current))
        elif current is not None and (line in ("successes:", "failures:") or line.startswith("test result:")):
            current = None
            outside.append(line)
        else:
            (outside if current is None else current).append(line)
    return sections, outside


def named_test_stdout(full_name, output):
    """libtest 给点名的测试交出来的那一段输出（libtest_stdout_sections 里「---- <全名> stdout ----」那一段）；
    有几段（成功与失败各交一次）就接起来。输出里一段都没有（--nocapture 那种起法）交整份；有别的用例的段而没有它的，交空串（它没打字）。"""
    sections, _outside = libtest_stdout_sections(output)
    header = f"---- {full_name} stdout ----"
    own = [lines for section_header, lines in sections if section_header == header]
    if own:
        return "\n".join(line for lines in own for line in lines)
    return "" if sections else output


def given_states_gpu_counts(text):
    """text 里每一行 CRASH_POINT_REPLAY_GIVEN_STATES（给定状态引擎每一趟收尾打的那一行，crate singlefs-verification 的
    crash_amplification::GIVEN_STATES_LINE_PREFIX；_KEY_REFUSED 这类同前缀的别的行不算）的 states_judged_by_the_gpu_here，按出现的次序。"""
    line = re.compile(r"^\s*CRASH_POINT_REPLAY_GIVEN_STATES\s(?:.*\s)?states_judged_by_the_gpu_here=(\d+)(?=\s|$)", re.M)
    return [int(found.group(1)) for found in line.finditer(text)]


def gpu_card_unused_reason(row, output, environment):
    """带 gpu 特性、判出没红的那一条用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：用不上交接在「点名的测试 <名字>」后面的那一句，
    用上了或判不到这一层（点名的测试不是恰好对上一个用例、结果不都是 ok）交 None。
    不打 CRASH_POINT_REPLAY_GIVEN_STATES 行的照常判（gpu_test_start 取卡：没卡打 NOT_RUN_NO_GPU_ADAPTER、有卡起不来就 panic，不悄悄改交 CPU）。
    弄坏开关 gpu-unused-as-missed：一律交 None；given-states-count-ignored：有那一行就当没用上卡，不读数。"""
    if break_is(environment, "gpu-unused-as-missed"):
        return None
    results = results_by_test(row.expected, output)
    if len(results) != 1 or not all(outcome == "ok" for outcomes in results.values() for outcome in outcomes):
        return None
    counts = given_states_gpu_counts(named_test_stdout(next(iter(results)), output))
    if counts and (break_is(environment, "given-states-count-ignored") or not any(counts)):
        return (f"带 gpu 特性，它打的 {len(counts)} 行 CRASH_POINT_REPLAY_GIVEN_STATES 里 states_judged_by_the_gpu_here 全是 0："
                "给定状态全交 CPU 判、GPU 一个状态都没判，改坏的 GPU 那一处没跑到")
    return None


def run_cargo_command(command, temporary_directory, label, extra_environment=None, on_real_service=False):
    """起一条 cargo 命令（cargo test --offline … 这一类）：(subprocess 的结果, 用时)。不套内存包装与清场包装、不设上限与限时：一整条门禁交给统一任务服务（task-server/），
    内存上限由服务按记录给、撞了放大重排，卡住的由服务按日志与 CPU 都不涨停整条任务，清场与分卡由服务在最外层套（文件头「带 gpu 特性的格不另套清场」）。TMPDIR 是这一次自己的；extra_environment 只加在这一次的环境里
    （覆盖那一格的编译目录、插桩开关与 LLVM_PROFILE_FILE）。label 只用在 stderr 的那几句里（基线的批、变异表第几行、覆盖）。
    真跑测试的那一次对它自己的临时 prune-server 跑（mutation_run_prune_server，经 init_worker 交的引用调，不进判法单元的闭包；文件头「变异复跑的崩溃判定」「判法单元」），
    跑完停、删。on_real_service（基线那一趟）：不起临时服务、连调用方的那一份，环境换成 baseline_cargo_environment 取的（不带 release 增量覆盖；文件头「基线连调用方的判定库服务」）。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    command = with_gpu_test_threads(command, context["environment"], state["gpu_released_targets"])
    base_environment = (baseline_cargo_environment(state["environment"], context.get("release_incremental_of_the_caller")) if on_real_service
                        else state["environment"])
    environment = dict(base_environment, TMPDIR=temporary_directory)
    environment.update(extra_environment or {})
    # 真跑测试的每一次把多机配置交给测试（文件头「真跑测试的格把多机配置交给测试」）：副本里它自己找不到，crates 那侧找不到就报错；
    # 文件在不在 run_stage 起 cargo 之前判过。弄坏开关 test-runs-configuration-unpassed：不设
    if "--no-run" not in command and not break_is(context["environment"], "test-runs-configuration-unpassed"):
        environment["SINGLEFS_MULTI_HOST_CONFIG"] = state["multi_host_configuration"](context["environment"])[0]   # init_worker 交的 test_runs_multi_host_configuration
    with state["temporary_service"](context["prune_server_binary"], command, temporary_directory, context["environment"], on_real_service) as service_settings:   # init_worker 交的 mutation_run_prune_server
        environment.update(service_settings)
        started = time.time()
        run = state["launch"](command, cwd=state["work"], env=environment)   # init_worker 交的 run_capturing_output
        seconds_running = time.time() - started
    return run, {"seconds_running": f"{seconds_running:.3f}"}


def uncounted_verdict_keeps_counted_record(existing, verdict, row_key, environment):
    """这一条要写进 records/<row_key> 的结局 verdict 不算数（VERDICTS_UNCOUNTED），而库里同一把行键原有的记录 existing（读不到交 None）是算数的结论
    （VERDICTS_COUNTED）时交 True：这一条不写，原有的留着（文件头「复用判据」记红撤绿那一段）；judge_row 与 judge_batch 共用这一份。
    弄坏开关 uncounted-overwrites-counted：一律交 False，不算数的照写、盖掉原有的算数结论。"""
    if break_is(environment, "uncounted-overwrites-counted"):
        return False
    return (verdict in VERDICTS_UNCOUNTED and existing is not None and existing.get("row_key") == row_key
            and existing.get("verdict") in VERDICTS_COUNTED)


def write_record_quietly(kind, key, fields):
    """写一条判定库记录；写不进去交那一句（交回父进程报在 stderr，下一趟这一条照跑），写成交 None。"""
    try:
        WORKER_STATE["library"].write(kind, key, fields)
    except OSError as error:
        return f"{WORKER_STATE['library'].path(kind, key)}：{error}"
    return None


def how_text(context, kind):
    """记录里的跑法：哪一道、什么起法、第几片（只是一样属性，不进键）。"""
    return f"{STAGE} {kind}（直接起 cargo，不设上限与限时；第 {WORKER_STATE['worker_sequence']} 片，共 {context['worker_count']} 片）"


def run_baseline_batch(key, batch_rows, context):
    """一批基线在没改坏的副本上跑一趟：(cargo test 的参数, 跑的结果, 用时)。连调用方的那一份判定库服务、不带 release 增量覆盖（文件头「基线连调用方的判定库服务」）。"""
    arguments = baseline_arguments(key, batch_rows)
    temporary_directory = tempfile.mkdtemp(prefix=f"baseline-{batch_rows[0].line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    try:
        run, seconds = run_cargo_command(["cargo", "test", "--offline"] + arguments, temporary_directory, f"基线（批首行是变异表第 {batch_rows[0].line_number} 行）",
                                         on_real_service=True)
    finally:
        WORKER_STATE["remove_temporary"](temporary_directory, context)   # init_worker 交的 measure_and_remove_temporary_directory（文件头「判法单元」）
    return arguments, run, seconds


def judge_batch(batch):
    """进程池里的一格活：一批基线（没改坏的副本上跑一遍这一批各行点名的测试）。回 (每行的 (行号, 问题或 None), 那一批的尾巴, 写不进去的记录)。
    一行的基线结局不随这一趟拼进了哪几行而变（文件头「基线」）：每行只看它自己的过滤词选得中的结果行（row_baseline_output）。
    弄坏开关 baseline-batch-composition：照整批那一趟的输出判。"""
    key, batch_rows = batch
    context = WORKER_STATE["context"]
    arguments, run, seconds = run_baseline_batch(key, batch_rows, context)
    judged, record_problems = [], []
    for row in batch_rows:
        output = row_baseline_output(row, run.stdout + run.stderr, context["environment"])
        problem = baseline_problem(row, run.returncode, output)
        excerpt = None
        if problem is not None:
            results = results_by_test(row.expected, output)
            excerpt = failure_excerpt(output, next(iter(results))) if len(results) == 1 else None
        judged.append((row.line_number, problem, excerpt or output_tail(output)))
        components = context["row_records"].get(row.line_number) or {}
        if components.get("baseline_key"):
            failed_to_write = write_record_quietly("baselines", components["baseline_key"], dict(
                kind="baseline", baseline_key=components["baseline_key"], test_closure=components.get("test_closure", ""),
                coverage_id=components.get("coverage_id", ""), tree_snapshot=WORKER_STATE["snapshot"], test_full_name=single_full_name(row.expected, output) or "",
                verdict="ok" if problem is None else "not-ok", problem=problem or "", batch_arguments=" ".join(arguments),
                batch_exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "基线"), **seconds))
            if failed_to_write:
                record_problems.append(failed_to_write)
        # 记红、撤绿（文件头「复用判据」）：基线不绿的这一行，按它的行键记一条「基线不绿」，盖掉同一把行键原有的「抓到」，下一趟它照现判（先跑基线）。
        # 弄坏开关 baseline-red-keeps-caught：不记，同一把行键上次抓到的那一份照复用、基线也不再跑
        if problem is not None and components.get("row_key") and not break_is(context["environment"], "baseline-red-keeps-caught"):
            fields = collections.OrderedDict(
                kind="mutation", row_key=components["row_key"], verdict="baseline-not-ok",
                text=f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}",
                exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "基线"),
                test_closure=components.get("test_closure", ""), mutated=components.get("mutated", ""))
            failed_to_write = write_record_quietly("records", components["row_key"], fields)
            if failed_to_write:
                record_problems.append(failed_to_write)
    return judged, record_problems


def judge_row(row):
    """进程池里的一格活：一条变异。建它自己的 TMPDIR、改坏那一处、跑点名的测试、还原、删掉那个 TMPDIR，判完立刻写判定库记录与用时表。
    回 (((行号, 档, 那一条要打印的话, 删 TMPDIR 的结果), {test_full_name}), 写不进去的记录)。"""
    judge_started = time.monotonic()
    context = WORKER_STATE["context"]
    full = os.path.join(WORKER_STATE["work"], row.path)
    with open(full, encoding="utf-8") as handle:
        pristine = handle.read()
    if pristine.count(row.old) != 1:
        # 派活之后源码被改、这一份工作副本里原文不再恰好一次（文件头「锚点失效」）：照锚点失效记这一行，不改坏、不跑、不入库，别的行照判
        # 弄坏开关 anchor-mid-run-asserts：在这里抛 AssertionError，整个进程池跟着炸
        assert not break_is(context["environment"], "anchor-mid-run-asserts"), (row.path, row.name)
        why = f"派活之后源码被改了：工作副本里的原文命中 {pristine.count(row.old)} 次，要恰好 1 次"
        return ((row.line_number, "anchor", anchor_broken_text(row, why), None), {}), []
    temporary_directory = tempfile.mkdtemp(prefix=f"line-{row.line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(pristine.replace(row.old, row.new))
    try:
        run, seconds = run_cargo_command(["cargo", "test", "--offline"] + with_shown_output(row.arguments.split()), temporary_directory,
                                         f"变异表第 {row.line_number} 行")
    finally:
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(pristine)
        # 不论退出码是几都删（含被信号杀掉的测试进程）：被杀的测试 Drop 守卫不跑，它建的镜像只有这里删得掉。
        # init_worker 交的 measure_and_remove_temporary_directory（文件头「判法单元」：记账不进判法单元）
        removal = WORKER_STATE["remove_temporary"](temporary_directory, context)
    output = run.stdout + run.stderr
    verdict, text = verdict_of_run(row, run.returncode, output, context["environment"])
    # 带 gpu 特性、判出没红的那一条核用没用上卡（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：用不上记本次未跑，不进没红
    unused = gpu_card_unused_reason(row, output, context["environment"]) if verdict == "failure" and gpu_test_run(
        ["cargo", "test"] + row.arguments.split(), context["environment"]) else None
    if unused is not None:
        verdict, text = "notrun", f"{TABLE}:{row.line_number} {row.name}：本次未跑——点名的测试 {row.expected} {unused}；不算抓到、也不算没红"
    components = context["row_records"].get(row.line_number) or {}
    full_name = single_full_name(row.expected, output) or ""
    record_problems = []
    # 判出的每一档都照行键写（文件头「复用判据」）：没红、无效这些不是「抓到」的盖掉同一把行键原有的「抓到」，下一趟照现判。
    # 弄坏开关 red-not-recorded：只记「抓到」（不是抓到的不写，原有的「抓到」留着照复用）
    # 不算数的结局（VERDICTS_UNCOUNTED）不盖同一把行键原有的算数结论：键相同仍现判（复用哨兵强制现判）时才碰得到；库里没有算数结论的照写
    keeps_counted = bool(components.get("row_key")) and verdict in VERDICTS_UNCOUNTED and uncounted_verdict_keeps_counted_record(
        WORKER_STATE["library"].read("records", components["row_key"]), verdict, components["row_key"], context["environment"])
    if components.get("row_key") and not keeps_counted and (verdict == REUSABLE_VERDICT["mutation"] or not break_is(context["environment"], "red-not-recorded")):
        fields = collections.OrderedDict(
            kind="mutation", row_key=components["row_key"], verdict=verdict, text=text,
            removal_files=removal[0], removal_allocated_bytes=removal[1], removal_apparent_bytes=removal[2], removal_left=int(removal[3]),
            exit_code=run.returncode, judged_date=mutation_units.today_text(), how=how_text(context, "现判"), test_full_name=full_name,
            test_closure=components.get("test_closure", ""), mutated=components.get("mutated", ""))
        fields.update(seconds)
        failed_to_write = write_record_quietly("records", components["row_key"], fields)
        if failed_to_write:
            record_problems.append(failed_to_write)
    record_problems += WORKER_STATE["write_timing"](row, seconds, context)   # init_worker 交的 write_row_timing（文件头「判法单元」：记账不进判法单元）
    return ((row.line_number, verdict, text, removal), dict(test_full_name=full_name, wall_seconds=time.monotonic() - judge_started)), record_problems
