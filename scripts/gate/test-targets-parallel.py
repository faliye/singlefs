#!/usr/bin/env python3
# admission: always 每次调都在这一刻的树上编、跑这一次给的包，上一次的结论不替这一次作保
# run-condition: command cargo
"""一趟 cargo test 拆成按测试目标并行起：合入后验证（scripts/gate/merge-and-verify.sh）测试那一步用它。

用法：
  test-targets-parallel.py --jobs <N> -- <cargo test 的参数…>      例：--jobs 31 -- --offline --no-fail-fast -p singlefs-core -p singlefs-harness
  test-targets-parallel.py --selftest
为什么：cargo test 一个测试二进制接一个地跑，用例少的二进制跑的时候核就空着；2026-10-21 合入后验证在第二台只用了约 5 核（登记 31）
（records/2026-10-18-里程碑三收尾与近期欠账.md 第 780 行）。
做法：
  ① 编：cargo test <参数> --no-run --message-format=json-render-diagnostics，一次把全部测试二进制编好（cargo 自己按它的 -j 并行编）；
     Compiling 那几行（cargo 打在标准错误）照原样转出去，合入后验证的重编核对读它们。编不过照 cargo 的退出码退，一个目标都不跑。
  ② 列目标：编译输出里 profile.test 为真、带 executable 的产物，一个产物一个目标；包名经 cargo metadata --no-deps 按 package_id 取；
     挑法照 cargo test 认的写：lib 类 → --lib，test → --test <名>，bin → --bin <名>，example → --example <名>，bench → --bench <名>。
  ③ 并行跑：每个目标一条 cargo test <参数里去掉 -p 那几对> -p <包> <挑法>（照 scripts/gate/rerun-failed-tests.py 那样按目标分开起；产物已编好，
     cargo 只核新鲜就跑，运行时的环境与工作目录照 cargo 自己给的）。线程按 N 条记账（.claude/kb/milestone/03-third-txn.md 记的尾巴空核：
     剩下的目标少于 N 时空出来的核没人用）：
       次序：长的先起。用时表（每个目标上一次跑了多少秒，见下）里没有的目标先起（照编译输出的次序），有的按用时从长到短；
       线程：每起一条，RUST_TEST_THREADS 取 max(1, ⌈此刻空着的线程 ÷ 还没起的目标数⌉)、不超过空着的；一条跑完把它的线程放回，再起下一条。
       还没起的比空着的线程多时每条 1 条线程；空着的比还没起的多（开头 N 比目标数大，或尾巴上几条一齐跑完）时把多出来的分给后起的、先起的多分一条。
       同时在算的用例线程合起来不超过 N。
     用时表：cargo metadata 给的 target_directory 下的 singlefs-test-target-durations.tsv（一行「<秒><制表符><包> <挑法>」，跑完收回时把这一趟每个目标的用时
       写回、别的行照留，先写临时文件再改名）；TEST_TARGETS_PARALLEL_DURATIONS 设了照它（自证用）；metadata 里没有 target_directory 又没设的，不读不写、照编译输出的次序起。
     每条的输出（标准输出与标准错误合成一份）写自己的文件、退出码写另一个文件，
     全部跑完按派活的次序逐个回读、转打到标准输出（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
     收回来的退出码文件数与派出去的目标数对不上，整趟判红、退 1。
  ④ 文档测试：没有显式目标选择（--lib、--test 等）时，要测的包里库目标 doctest 为真的（cargo metadata），最后一条 cargo test <参数> -p <这些包> --doc 一起跑（③ 都跑完了，RUST_TEST_THREADS 取 N）。
  测试二进制的参数：<cargo test 的参数…> 里第二个 `--` 之后的（--nocapture、--include-ignored、--exact、--skip <名> 这类）原样接在 ③ 每一条与 ④ 那一条的 `--` 之后，
     一个字不改、不挑（checker 档快档拿 --exact --skip 跳过键相同的测试函数，合入后验证拿 --include-ignored 把慢档一起跑）。
  ⑤ 数慢档（测试二进制的参数里有 --include-ignored 时）：③ 每一条开跑之前、④ 开跑之前，各先起一次同一条 cargo test，`--` 之后只交选用例的参数
     （--exact、--skip <名>、不带 - 的筛选词）再接 --list --ignored，数它打的「<名>: test」行，就是这一条里标 #[ignore] 而这一趟会跑到的用例条数；
     各条加起来写进末行的 slow=。列的那一趟退非 0 的，末行写 slow=?（数不出，不算红）；参数里没有 --include-ignored 的写 slow=-。
     列的那一趟起的是测试二进制本身：harness = false 的目标不认 --list、会照跑它的 main（工作区里此刻没有这种目标，加了要改这一步）。
  串行段声明（scripts/hosts/core_usage.py 文件头「声明」，SINGLEFS_SERIAL_DECLARATIONS 设了才写）：③ 的目标都派出去、只剩还没跑完的那几条时
     声明一次（空出来的线没有目标可派，剩下的各自跑完自己的用例）；④ 开跑时再声明一次（文档测试只有一个 cargo，逐个编、逐个跑）。
  末行 TEST_TARGETS_PARALLEL dispatched=<派出> collected=<收回> failed=<红的目标数> jobs=<同时至多几条> test_threads=<每条的线程数，各条不同时写 最少-最多> doc=<yes|no|failed> slow=<慢档条数|?|->。
退出码：都绿 0；有目标或文档测试红 101；编不过照 cargo 的；收回与派出对不上、cargo metadata 读不出 1；用法错 2。
线程数只照 --jobs（调用方经 capped.sh 给登记值），这里不现算、不写常量。
弄坏开关 TEST_TARGETS_PARALLEL_BREAK（只给自证证红用），一项一行，下一行写打开后判错的自证格：
  serial            同时只起一条（改前的样子：一个测试二进制接一个地跑）
    「四个目标同时在跑」「空出来的线程分给后起的、长的多分」（同时只起一条，每条都拿满 N 条线程）
  drop-collection   回读时丢掉最后派出的那一条、也不核收回的条数
    「收回的条数等于派出的」
  ignore-red        目标退非 0 也按绿算、退 0
    「红的目标照报、退 101」
  even-threads      每条的 RUST_TEST_THREADS 一律取 ⌈N ÷ min(N, 目标数)⌉（改前的样子：空出来的线程不分给后起的）
    「空出来的线程分给后起的、长的多分」
  history-ignored   不看用时表，照编译输出的次序起（改前的样子）
    「长的先起」「空出来的线程分给后起的、长的多分」（最长的 tool 不先起，多出来的那条线程落到先起的 lib 上）
  test-arguments-dropped  ③ 每一条与 ④ 那一条都不接测试二进制的参数（--exact --skip 那几对丢了，键相同的测试函数照跑）
    「测试二进制的参数原样交给每个目标」
  slow-count-skipped  带 --include-ignored 也不数慢档，末行照写 slow=-
    「带 --include-ignored 时逐个目标数出慢档条数」
自证（--selftest）：PATH 最前面放一份假 cargo——metadata 打一个包 demo（库带文档测试），--no-run 打四个测试目标的产物（lib、test alpha、test beta、bin tool，
  另有一份不是测试的库产物与一份构建脚本产物，不算目标），跑某个目标时记下起止时刻与 RUST_TEST_THREADS、睡 3 秒，beta 那一个打一条 FAILED、退 101；--jobs 4：
  「四个目标同时在跑」四条的起止区间两两重叠（最晚的起点早于最早的终点）、每条的 RUST_TEST_THREADS 是 1；
  「收回的条数等于派出的」末行 dispatched=4 collected=4，四个目标的 test result 行都在标准输出里；
  「红的目标照报、退 101」退 101，FAILED 那一行与「-p demo --test beta」都在输出里；
  「编译与文档测试」Compiling 那一行转到了标准错误，文档测试跑了恰好一次。
  另两趟各带一份用时表（tool 30 秒、beta 20 秒、lib 5 秒、alpha 1 秒）：
  「长的先起」--jobs 2：tool 与 beta 先起，lib 与 alpha 的起点都不早于前两条里先跑完的那一条的终点；跑完用时表里四个目标都写回了这一趟的用时；
  「空出来的线程分给后起的、长的多分」--jobs 5：最长的 tool 拿 2 条线程、另三条各 1 条，合起来正好 5 条。
  再一趟带测试二进制的参数 `-- --include-ignored --nocapture --exact --skip alpha_case --skip tool_case`（--jobs 4；假 cargo 跑某个目标时另记一行收到的参数，
  遇到 --list 只打一条「<目标>_slow: test」、不睡、不记起止）：
  「测试二进制的参数原样交给每个目标」四个目标与文档测试那一条收到的参数都以 `-- --include-ignored --nocapture --exact --skip alpha_case --skip tool_case` 收尾；
  「带 --include-ignored 时逐个目标数出慢档条数」四个目标与文档测试各列了一次、列的那一趟 `--` 之后是 `--exact --skip alpha_case --skip tool_case --list --ignored`
  （不带 --include-ignored 与 --nocapture），末行 slow=5。
"""
import concurrent.futures
import fnmatch
import json
import math
import os
import subprocess
import sys
import tempfile
import time
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "TEST_TARGETS_PARALLEL_BREAK"
DECLARATIONS_VARIABLE = "SINGLEFS_SERIAL_DECLARATIONS"   # 写法同 scripts/hosts/core_usage.py 的 SERIAL_MARK
SERIAL_MARK = "此刻串行："
LIBRARY_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
SELECTOR_OPTION_BY_KIND = {"test": "--test", "bin": "--bin", "example": "--example", "bench": "--bench"}
SUMMARY_PREFIX = "TEST_TARGETS_PARALLEL"
SELFTEST_CELLS = ("四个目标同时在跑", "收回的条数等于派出的", "红的目标照报、退 101", "编译与文档测试", "长的先起",
                  "空出来的线程分给后起的、长的多分", "测试二进制的参数原样交给每个目标", "带 --include-ignored 时逐个目标数出慢档条数")   # 自证的格名，判错行照抄
SLOW_TIER_OPTION = "--include-ignored"   # 测试二进制的参数里有它就数慢档（文件头 ⑤）
# 数慢档那一趟不交的 libtest 参数里带一个值的（文件头 ⑤：只留选用例的参数，这几样只管怎么跑）
RUN_OPTIONS_WITH_VALUE = {"--test-threads", "--color", "--format", "--logfile", "--shuffle-seed", "-Z"}
DURATIONS_VARIABLE = "TEST_TARGETS_PARALLEL_DURATIONS"   # 用时表换位置（自证用）；不设取 target_directory 下的 DURATIONS_FILE_NAME
DURATIONS_FILE_NAME = "singlefs-test-target-durations.tsv"


def broken(item):
    return os.environ.get(BREAK_VARIABLE, "") == item


def split_arguments(arguments):
    """cargo test 的参数 → (去掉 -p / --package 之后的参数, 点名的包, `--` 之后交给测试二进制的参数)。"""
    if "--" in arguments:
        cut = arguments.index("--")
        cargo_arguments, test_arguments = arguments[:cut], arguments[cut + 1:]
    else:
        cargo_arguments, test_arguments = arguments, []
    kept, packages, position = [], [], 0
    while position < len(cargo_arguments):
        argument = cargo_arguments[position]
        if argument in ("-p", "--package") and position + 1 < len(cargo_arguments):
            packages.append(cargo_arguments[position + 1])
            position += 2
            continue
        if argument.startswith("--package="):
            packages.append(argument.split("=", 1)[1])
        elif argument.startswith("-p") and len(argument) > 2:
            packages.append(argument[2:])
        else:
            kept.append(argument)
        position += 1
    return kept, packages, test_arguments


def package_options(packages):
    return [word for package in packages for word in ("-p", package)]


def execution_arguments(arguments):
    """目标已经由编译产物选定；逐目标执行时去掉会扩大范围的选择参数。"""
    kept, selected_targets, position = [], False, 0
    target_flags = {"--lib", "--bins", "--tests", "--examples", "--benches", "--all-targets", "--doc"}
    target_values = {"--bin", "--test", "--example", "--bench"}
    while position < len(arguments):
        argument = arguments[position]
        option = argument.split("=", 1)[0]
        if option in target_flags or option in target_values:
            selected_targets = True
            position += 2 if option in target_values and "=" not in argument else 1
            continue
        if option in {"--workspace", "--all", "--exclude"}:
            position += 2 if option == "--exclude" and "=" not in argument else 1
            continue
        kept.append(argument)
        position += 1
    return kept, selected_targets


def listing_arguments(test_arguments):
    """数慢档那一趟交给测试二进制的参数（文件头 ⑤）：只留选用例的（--exact、--skip <名>、不带 - 的筛选词），再接 --list --ignored。"""
    kept, position = [], 0
    while position < len(test_arguments):
        argument = test_arguments[position]
        if argument == "--skip" and position + 1 < len(test_arguments):
            kept += test_arguments[position:position + 2]
            position += 2
            continue
        if argument == "--exact" or argument.startswith("--skip=") or not argument.startswith("-"):
            kept.append(argument)
        elif argument in RUN_OPTIONS_WITH_VALUE:
            position += 1   # 跳过它的值
        position += 1
    return kept + ["--list", "--ignored"]


def slow_count(command, environment):
    """数一条 cargo test 里标 #[ignore] 而这一趟会跑到的用例（文件头 ⑤）：交条数；cargo 退非 0 交 None（数不出）。"""
    completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, env=environment)
    if completed.returncode != 0:
        return None
    return sum(1 for line in completed.stdout.splitlines() if line.endswith(": test"))


def declare_serial(reason):
    """往 SINGLEFS_SERIAL_DECLARATIONS 追加一行串行段声明；没设、写不进都不碍跑测试。"""
    path = os.environ.get(DECLARATIONS_VARIABLE, "")
    if not path:
        return
    try:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{time.time():.3f}\t{SERIAL_MARK}{reason}\n")
    except OSError:
        pass


def selector_of(target):
    """编译产物的 target → cargo test 挑它的参数；认不出的种类交 None。"""
    kinds = set(target.get("kind") or [])
    if kinds & LIBRARY_KINDS:
        return ["--lib"]
    for kind, option in SELECTOR_OPTION_BY_KIND.items():
        if kind in kinds:
            return [option, target.get("name", "")]
    return None


def read_metadata(cargo_arguments):
    """元数据跟随清单与配置：交 (ID 到包名, 文档测试开关, 默认或 --workspace 选中的成员, target 目录)；读不出交 None 与原因。"""
    command = ["cargo", "metadata", "--no-deps", "--format-version", "1"]
    position = 0
    while position < len(cargo_arguments):
        argument = cargo_arguments[position]
        option = argument.split("=", 1)[0]
        if option in {"--manifest-path", "--config", "--features", "-F", "-Z"}:
            command.append(argument)
            if "=" not in argument and position + 1 < len(cargo_arguments):
                position += 1
                command.append(cargo_arguments[position])
        elif argument in {"--offline", "--locked", "--frozen", "--all-features", "--no-default-features"}:
            command.append(argument)
        position += 1
    completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if completed.returncode != 0:
        return None, f"cargo metadata 退 {completed.returncode}：{completed.stderr.strip()[-300:]}"
    try:
        metadata = json.loads(completed.stdout)
    except ValueError as error:
        return None, f"cargo metadata 打出来的不是 JSON（{error}）"
    names, doctests = {}, {}
    for package in metadata.get("packages", []):
        names[package.get("id")] = package.get("name")
        doctests[package.get("name")] = any(set(target.get("kind") or []) & LIBRARY_KINDS and target.get("doctest")
                                            for target in package.get("targets", []))
    workspace = "--workspace" in cargo_arguments or "--all" in cargo_arguments
    selected = metadata.get("workspace_members", []) if workspace else metadata.get("workspace_default_members", metadata.get("workspace_members", []))
    excluded = [argument.split("=", 1)[1] for argument in cargo_arguments if argument.startswith("--exclude=")]
    excluded += [cargo_arguments[index + 1] for index, argument in enumerate(cargo_arguments[:-1]) if argument == "--exclude"]
    members = [names[member] for member in selected if member in names
               and not any(fnmatch.fnmatchcase(names[member], pattern) for pattern in excluded)]
    return (names, doctests, members, metadata.get("target_directory")), None


def durations_path(target_directory):
    """用时表在哪（文件头 ③「用时表」）：TEST_TARGETS_PARALLEL_DURATIONS 设了照它，不设取 target_directory 下的那一份；都没有交 None（不读不写）。"""
    if os.environ.get(DURATIONS_VARIABLE):
        return os.environ[DURATIONS_VARIABLE]
    return os.path.join(target_directory, DURATIONS_FILE_NAME) if target_directory else None


def read_durations(path):
    """用时表 → {「<包> <挑法>」: 秒}；文件不在、行写坏的跳过。"""
    durations = {}
    if not path or not os.path.isfile(path):
        return durations
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            seconds, _, key = line.rstrip("\n").partition("\t")
            try:
                durations[key] = float(seconds)
            except ValueError:
                continue
    return durations


def write_durations(path, durations):
    """用时表整份写回：先写同目录临时文件再改名；写不进不碍这一趟的结论。"""
    if not path:
        return
    try:
        temporary = f"{path}.{os.getpid()}.tmp"
        with open(temporary, "w", encoding="utf-8") as handle:
            for key, seconds in sorted(durations.items()):
                handle.write(f"{seconds:.3f}\t{key}\n")
        os.replace(temporary, path)
    except OSError:
        pass


def unit_key(package, selector):
    return f"{package} {' '.join(selector)}"


def threads_for_next(free, remaining, jobs, total):
    """下一条拿几条线程（文件头 ③「线程」）：max(1, ⌈空着的 ÷ 还没起的⌉)、不超过空着的；弄坏开关 even-threads 照改前一律 ⌈N ÷ min(N, 目标数)⌉。"""
    if broken("even-threads"):
        return max(1, math.ceil(jobs / max(1, min(jobs, total))))
    return max(1, min(free, math.ceil(free / max(1, remaining))))


def build(cargo_arguments, packages):
    """① 编：交 (退出码, [产物 JSON…])；非 JSON 的标准输出行照原样打出去。"""
    command = ["cargo", "test"] + cargo_arguments + package_options(packages) + ["--no-run", "--message-format=json-render-diagnostics"]
    completed = subprocess.run(command, stdout=subprocess.PIPE, text=True)
    artifacts = []
    for line in completed.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            print(line)
            continue
        if isinstance(message, dict) and message.get("reason") == "compiler-artifact":
            artifacts.append(message)
        elif isinstance(message, dict) and message.get("reason") == "compiler-message":
            rendered = (message.get("message") or {}).get("rendered")
            if rendered:
                sys.stderr.write(rendered)
    sys.stdout.flush()
    return completed.returncode, artifacts


def run(jobs, arguments):
    cargo_arguments, packages, test_arguments = split_arguments(arguments)
    unit_arguments, selected_targets = execution_arguments(cargo_arguments)
    build_exit, artifacts = build(cargo_arguments, packages)
    if build_exit != 0:
        print(f"  ✗ cargo test --no-run 退 {build_exit}：测试二进制没编成，一个目标都没跑")
        print("  → 怎么办：看上面 cargo 打的编译错误；编成了再跑这一步")
        return build_exit
    metadata, problem = read_metadata(cargo_arguments)
    if metadata is None:
        print(f"  ✗ 读不出包名与文档测试：{problem}")
        print("  → 怎么办：在树根跑 cargo metadata --no-deps --format-version 1 看它为什么退；读不出包名就派不了按目标分开的 cargo test")
        return 1
    names, doctests, members, target_directory = metadata
    selected_packages = members
    if packages and not ({"--workspace", "--all"} & set(cargo_arguments)):
        selected_packages = []
        for specification in packages:
            pattern = specification.split("#")[-1].split("@")[0]
            matched = [name for name in doctests if fnmatch.fnmatchcase(name, pattern)]
            matched += [name for identity, name in names.items() if identity == specification and name not in matched]
            if not matched and not selected_targets:
                print(f"  ✗ 无法把包选择 {specification} 对应到元数据，文档测试范围判不了")
                print("  → 怎么办：用 cargo metadata 列出的包名传 -p，不能把未识别的包当成没有文档测试")
                return 1
            selected_packages.extend(name for name in matched if name not in selected_packages)
    units = []
    for artifact in artifacts:
        if not artifact.get("executable") or not (artifact.get("profile") or {}).get("test"):
            continue
        selector = selector_of(artifact.get("target") or {})
        package = names.get(artifact.get("package_id"))
        if selector is None or package is None:
            print(f"  ✗ 认不出这个测试产物属于哪个包、怎么挑：{artifact.get('package_id')} {artifact.get('target')}")
            print("  → 怎么办：看 cargo metadata 与编译产物的 package_id 对不对得上；对不上就是 cargo 的输出格式变了，照它改 selector_of / read_metadata")
            return 1
        units.append((package, selector))
    concurrent_jobs = 1 if broken("serial") else max(1, min(jobs, len(units)))
    history_path = durations_path(target_directory)
    history = {} if broken("history-ignored") else read_durations(history_path)
    # 长的先起（文件头 ③「次序」）：用时表里没有的先起、照编译输出的次序，有的按用时从长到短
    order = sorted(range(len(units)), key=lambda index: (unit_key(*units[index]) in history, -history.get(unit_key(*units[index]), 0.0), index))
    thread_counts, elapsed, slow_of = {}, {}, {}
    environment = dict(os.environ)
    scratch = tempfile.mkdtemp(prefix="test-targets-parallel-")
    # 测试二进制的参数原样接在 `--` 之后（文件头「测试二进制的参数」）；弄坏开关 test-arguments-dropped 下丢掉
    passed_arguments = [] if broken("test-arguments-dropped") else test_arguments
    # 数慢档（文件头 ⑤）；弄坏开关 slow-count-skipped 下不数
    counting_slow = SLOW_TIER_OPTION in test_arguments and not broken("slow-count-skipped")

    def run_unit(index, package, selector, threads):
        command = ["cargo", "test"] + unit_arguments + ["-p", package] + selector + (["--"] + passed_arguments if passed_arguments else [])
        if counting_slow:
            slow_of[index] = slow_count(["cargo", "test"] + unit_arguments + ["-p", package] + selector + ["--"] + listing_arguments(test_arguments),
                                        dict(environment, RUST_TEST_THREADS=str(threads)))
        unit_started = time.time()
        with open(os.path.join(scratch, f"{index}.log"), "w", encoding="utf-8") as output:
            exit_code = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, env=dict(environment, RUST_TEST_THREADS=str(threads))).returncode
        elapsed[index] = time.time() - unit_started
        with open(os.path.join(scratch, f"{index}.exit"), "w", encoding="utf-8") as handle:
            handle.write(f"{exit_code}\n")

    # 派活（文件头 ③「线程」）：线程按 --jobs 记账，每起一条按此刻空着的与还没起的分；一条跑完放回它的线程再起下一条
    pending, running, free = list(order), {}, jobs
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrent_jobs) as executor:
        while pending or running:
            while pending and len(running) < concurrent_jobs and (free > 0 or not running or broken("even-threads")):
                index = pending.pop(0)
                threads = jobs if broken("serial") else threads_for_next(free, len(pending) + 1, jobs, len(units))
                thread_counts[index] = threads
                free -= threads
                running[executor.submit(run_unit, index, *units[index], threads)] = threads
                if not pending and len(units) > 1:
                    declare_serial("测试目标都派出去了，剩下的几条各自跑完自己的用例，空出来的线没有目标可派")
            done, _ = concurrent.futures.wait(list(running), return_when=concurrent.futures.FIRST_COMPLETED)
            for future in done:
                free += running.pop(future)
                future.result()
    if history_path and not broken("history-ignored"):
        merged = read_durations(history_path)
        merged.update({unit_key(*units[index]): seconds for index, seconds in elapsed.items()})
        write_durations(history_path, merged)
    collected, failed = 0, []
    collected_units = units[:-1] if broken("drop-collection") and units else units
    for index, (package, selector) in enumerate(collected_units):
        log_path, exit_path = os.path.join(scratch, f"{index}.log"), os.path.join(scratch, f"{index}.exit")
        if not os.path.exists(exit_path):
            continue
        collected += 1
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            sys.stdout.write(handle.read())
        with open(exit_path, encoding="utf-8") as handle:
            exit_text = handle.read().strip()
        if exit_text != "0":
            failed.append(f"-p {package} {' '.join(selector)}（退 {exit_text}）")
    sys.stdout.flush()
    doc_state = "no"
    doc_packages = [] if selected_targets else [package for package in selected_packages if doctests.get(package)]
    if doc_packages:
        declare_serial("文档测试只有一个 cargo，逐个编、逐个跑，没法按目标分开")
        if counting_slow:
            slow_of["doc"] = slow_count(["cargo", "test"] + unit_arguments + package_options(doc_packages) + ["--doc", "--"] + listing_arguments(test_arguments),
                                        dict(environment, RUST_TEST_THREADS=str(jobs)))
        doc_exit = subprocess.run(["cargo", "test"] + unit_arguments + package_options(doc_packages) + ["--doc"]
                                  + (["--"] + passed_arguments if passed_arguments else []), env=dict(environment, RUST_TEST_THREADS=str(jobs))).returncode
        doc_state = "yes" if doc_exit == 0 else "failed"
        if doc_exit != 0:
            failed.append(f"--doc（退 {doc_exit}）")
    for name in os.listdir(scratch):
        os.remove(os.path.join(scratch, name))
    os.rmdir(scratch)
    counts = sorted(set(thread_counts.values())) or [1]
    threads_text = str(counts[0]) if len(counts) == 1 else f"{counts[0]}-{counts[-1]}"
    slow_text = "-" if not counting_slow else "?" if None in slow_of.values() else str(sum(slow_of.values()))
    print(f"{SUMMARY_PREFIX} dispatched={len(units)} collected={collected} failed={len(failed)} jobs={concurrent_jobs} test_threads={threads_text} doc={doc_state} slow={slow_text}")
    if collected != len(units) and not broken("drop-collection"):
        print(f"  ✗ 派出去 {len(units)} 个测试目标，收回 {collected} 个退出码：有目标的结局丢了，这一趟不算数")
        print("  → 怎么办：整趟重跑；还对不上就看临时目录里各目标的 .exit 为什么没写（目标被杀、磁盘写不进）")
        return 1
    if failed and not broken("ignore-red"):
        print(f"  ✗ 红的测试目标 {len(failed)} 个：{'；'.join(failed)}")
        print("  → 怎么办：只重跑红的那几条用例：python3 scripts/gate/rerun-failed-tests.py <这一趟的日志> -p <包>")
        return 101
    return 0


FAKE_CARGO = r'''#!/usr/bin/env bash
# test-targets-parallel.py 自证的假 cargo：metadata、--no-run 打录好的 JSON；跑一个目标时记起止时刻与 RUST_TEST_THREADS、睡 3 秒，beta 打一条 FAILED 退 101
case "$1" in
  metadata) cat "$SELFTEST_METADATA"; exit 0 ;;
  test) ;;
  *) echo "假 cargo 不认：$*" >&2; exit 2 ;;
esac
shift
arguments=" $* "
if [[ "$arguments" == *" --no-run "* ]]; then
  echo "   Compiling demo v0.1.0 (/selftest/demo)" >&2
  cat "$SELFTEST_ARTIFACTS"
  exit 0
fi
name=lib
if [[ "$arguments" == *" --doc "* ]]; then name=doc; elif [[ "$arguments" =~ --(test|bin)\ ([a-z_]+) ]]; then name="${BASH_REMATCH[2]}"; fi
# 数慢档那一趟（--list）：记一行收到的参数，只列一条标了 ignore 的用例，不睡、不记起止
if [[ "$arguments" == *" --list "* ]]; then
  echo "list $name $*" >> "$SELFTEST_CALLS"
  echo "${name}_slow: test"
  echo
  echo "1 test, 0 benchmarks"
  exit 0
fi
if [[ "$name" == doc ]]; then
  echo "doc" >> "$SELFTEST_CALLS"
  echo "args doc $*" >> "$SELFTEST_CALLS"
  echo "   Doc-tests demo"
  echo "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
  exit 0
fi
echo "args $name $*" >> "$SELFTEST_CALLS"
echo "start $name $(date +%s.%N) ${RUST_TEST_THREADS:-unset}" >> "$SELFTEST_CALLS"
sleep 3
echo "     Running $name (target/debug/deps/$name-0123456789abcdef)" >&2
if [[ "$name" == beta ]]; then
  echo "test beta_case ... FAILED"
  echo "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"
  code=101
else
  echo "test ${name}_case ... ok"
  echo "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
  code=0
fi
echo "end $name $(date +%s.%N)" >> "$SELFTEST_CALLS"
exit "$code"
'''


def selftest_run(jobs, history=None, test_arguments=()):
    """拿假 cargo 跑一趟：交 (完成的进程, 假 cargo 记的调用行, 跑完的用时表或 None)。history 给了就写成这一趟的用时表（{「<包> <挑法>」: 秒}）；
    test_arguments 给了就接在第二个 `--` 之后（测试二进制的参数）。"""
    package_id = "path+file:///selftest/demo#0.1.0"
    metadata = {"packages": [{"id": package_id, "name": "demo", "targets": [
        {"kind": ["lib"], "name": "demo", "doctest": True}, {"kind": ["test"], "name": "alpha", "doctest": False},
        {"kind": ["test"], "name": "beta", "doctest": False}, {"kind": ["bin"], "name": "tool", "doctest": False}]}],
        "workspace_members": [package_id]}

    def artifact(kinds, name, test, executable=True):
        return json.dumps({"reason": "compiler-artifact", "package_id": package_id, "target": {"kind": kinds, "name": name},
                           "profile": {"test": test}, "executable": f"/selftest/target/debug/deps/{name}-0123" if executable else None})
    artifacts = [artifact(["lib"], "demo", False, executable=False), artifact(["custom-build"], "build-script-build", False),
                 artifact(["lib"], "demo", True), artifact(["test"], "alpha", True), artifact(["test"], "beta", True), artifact(["bin"], "tool", True)]
    with tempfile.TemporaryDirectory() as work:
        bin_directory = os.path.join(work, "bin")
        os.makedirs(bin_directory)
        fake = os.path.join(bin_directory, "cargo")
        with open(fake, "w", encoding="utf-8") as handle:
            handle.write(FAKE_CARGO)
        os.chmod(fake, 0o755)
        paths = {name: os.path.join(work, name) for name in ("metadata.json", "artifacts.jsonl", "calls.txt", "durations.tsv")}
        with open(paths["metadata.json"], "w", encoding="utf-8") as handle:
            json.dump(metadata, handle)
        with open(paths["artifacts.jsonl"], "w", encoding="utf-8") as handle:
            handle.write("\n".join(artifacts) + "\n")
        if history is not None:
            write_durations(paths["durations.tsv"], history)
        environment = dict(os.environ, PATH=bin_directory + os.pathsep + os.environ.get("PATH", ""), SELFTEST_METADATA=paths["metadata.json"],
                           SELFTEST_ARTIFACTS=paths["artifacts.jsonl"], SELFTEST_CALLS=paths["calls.txt"], **{DURATIONS_VARIABLE: paths["durations.tsv"]})
        environment.pop(DECLARATIONS_VARIABLE, None)
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--jobs", str(jobs), "--", "--offline", "--no-fail-fast", "-p", "demo"]
                                   + (["--", *test_arguments] if test_arguments else []),
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment, cwd=work, timeout=300)
        calls = open(paths["calls.txt"], encoding="utf-8").read().splitlines() if os.path.exists(paths["calls.txt"]) else []
        after = read_durations(paths["durations.tsv"]) if history is not None else None
    return completed, calls, after


def selftest():
    failures = []
    completed, calls, _ = selftest_run(4)
    starts = {line.split()[1]: (float(line.split()[2]), line.split()[3]) for line in calls if line.startswith("start ")}
    ends = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    if sorted(starts) != ["alpha", "beta", "lib", "tool"] or sorted(ends) != sorted(starts) \
            or max(moment for moment, _ in starts.values()) >= min(ends.values()) or {threads for _, threads in starts.values()} != {"1"}:
        failures.append(f"四个目标同时在跑：四条的起止区间应当两两重叠（最晚的起点早于最早的终点）、每条 RUST_TEST_THREADS=1，实际起点 {starts}、终点 {ends}")
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    result_lines = [line for line in completed.stdout.splitlines() if line.startswith("test result: ")]
    if not summary or "dispatched=4 collected=4" not in summary[-1] or len(result_lines) != 5:
        failures.append(f"收回的条数等于派出的：末行应当是 dispatched=4 collected=4、四个目标加文档测试共 5 行 test result，实际末行 {summary}、test result {len(result_lines)} 行")
    if completed.returncode != 101 or "test beta_case ... FAILED" not in completed.stdout or "-p demo --test beta" not in completed.stdout:
        failures.append(f"红的目标照报、退 101：实际退 {completed.returncode}，输出尾部 {completed.stdout[-400:]!r}")
    if "Compiling demo v0.1.0" not in completed.stderr or calls.count("doc") != 1:
        failures.append(f"编译与文档测试：Compiling 那一行应当转到标准错误、文档测试跑恰好一次，实际标准错误 {completed.stderr[-300:]!r}、文档测试 {calls.count('doc')} 次")
    # 用时表：tool 最长、beta 次之（文件头 ③「次序」「线程」）
    history = {"demo --bin tool": 30.0, "demo --test beta": 20.0, "demo --lib": 5.0, "demo --test alpha": 1.0}
    _, calls, after = selftest_run(2, history)
    starts = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("start ")}
    ends = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    first_end = min((ends[name] for name in ("tool", "beta") if name in ends), default=None)
    if sorted(starts) != ["alpha", "beta", "lib", "tool"] or first_end is None \
            or not all(starts[name] >= first_end for name in ("lib", "alpha")) or sorted(after or {}) != sorted(history) \
            or any(abs((after or {}).get(key, 0.0) - history[key]) < 0.5 for key in history):
        failures.append(f"长的先起：--jobs 2 时用时表里最长的 tool 与 beta 应当先起、lib 与 alpha 等前两条有一条跑完才起，跑完用时表写回这一趟四个目标的用时，"
                        f"实际起点 {starts}、终点 {ends}、跑完的用时表 {after}")
    _, calls, _ = selftest_run(5, history)
    threads = {line.split()[1]: line.split()[3] for line in calls if line.startswith("start ")}
    if threads != {"tool": "2", "beta": "1", "lib": "1", "alpha": "1"}:
        failures.append(f"空出来的线程分给后起的、长的多分：--jobs 5、四个目标时最长的 tool 应当拿 2 条线程、另三条各 1 条（合起来 5 条），实际 {threads}")
    # 测试二进制的参数（文件头「测试二进制的参数」）与数慢档（文件头 ⑤）：同一趟里核两格
    passed = ["--include-ignored", "--nocapture", "--exact", "--skip", "alpha_case", "--skip", "tool_case"]
    completed, calls, _ = selftest_run(4, test_arguments=passed)
    def shown_calls(recorded):
        return "；".join(name + "：" + " ".join(words) for name, words in sorted(recorded.items()))
    received = {line.split()[1]: line.split()[2:] for line in calls if line.startswith("args ")}
    wanted_tail = ["--", *passed]
    if sorted(received) != ["alpha", "beta", "doc", "lib", "tool"] or any(words[-len(wanted_tail):] != wanted_tail for words in received.values()):
        failures.append("测试二进制的参数原样交给每个目标：四个目标与文档测试那一条收到的参数都应当以「" + " ".join(wanted_tail) + "」收尾，"
                        "实际 " + shown_calls(received))
    listed = {line.split()[1]: line.split()[2:] for line in calls if line.startswith("list ")}
    listing_tail = ["--", "--exact", "--skip", "alpha_case", "--skip", "tool_case", "--list", "--ignored"]
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    if sorted(listed) != ["alpha", "beta", "doc", "lib", "tool"] or any(words[-len(listing_tail):] != listing_tail for words in listed.values()) \
            or not summary or not summary[-1].endswith(" slow=5"):
        failures.append("带 --include-ignored 时逐个目标数出慢档条数：四个目标与文档测试应当各列一次（`--` 之后是「" + " ".join(listing_tail[1:]) + "」）、末行 slow=5，"
                        "实际列了 " + shown_calls(listed) + f"，末行 {summary}")
    if failures:
        for failure in failures:
            print(f"  ✗ test-targets-parallel 自检：{failure}")  # gate-lint:detail
        print(f"  → 看 run() 的派活、线程记账、用时表、回读与条数核对；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ test-targets-parallel 自检通过（{len(SELFTEST_CELLS)} 格：{'、'.join(SELFTEST_CELLS)}）")
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    if len(arguments) < 3 or arguments[0] != "--jobs" or not arguments[1].isdigit() or int(arguments[1]) < 1 or arguments[2] != "--":
        print("  ✗ 用法：test-targets-parallel.py --jobs <正整数> -- <cargo test 的参数…>", file=sys.stderr)
        print("  → 例：test-targets-parallel.py --jobs 31 -- --offline --no-fail-fast -p singlefs-core；线程数照调用方给的登记值（capped.sh），自检：--selftest", file=sys.stderr)
        sys.exit(2)
    sys.exit(run(int(arguments[1]), arguments[3:]))


if __name__ == '__main__':
    preflight(__file__)
    main()
