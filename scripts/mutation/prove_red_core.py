"""prove-red.sh 拆出的一份。
证红的判法本体：读变异表与成形、重测试目标补 --release 与验证包补行、起 cargo 与编译目录、还原日志与断点还原、判定库的行键与复用、并行的路、prove 与 prove_on_restored_sources。名字在入口 prove-red.sh 的内联 python 里再导出；弄坏开关 PROVE_RED_BREAK 的取值在入口文件头的开关表里。
入口 prove-red.sh 的内联 python 按同目录 import 这一份；拆出的两份只向前 import（core → selftest）。
"""
import base64, collections, contextlib, hashlib, importlib.util, json, os, re, secrets, shutil, signal, subprocess, sys, tempfile, threading, time

sys.dont_write_bytecode = True   # 下面按路径载入的两份模块不往 __pycache__ 里写
SCRIPT_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
ROCKSDB_PREBUILT = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（prove_holding_the_lock）
BROKEN = os.environ.get("PROVE_RED_BREAK", "")
TARGET_OPTIONS = {"--lib", "--test", "--bin"}
EXIT_NOT_RUN = 77
RESTORE_JOURNAL_NAME = ".prove-red-pending"   # 还原日志（文件头「判法与次序」的「还原日志」一条）：放在副本根


def load_module(path, module_name):
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


# 判法（切表、成形、判结局）与门禁 crates-mutation-replay 同一份
ROWS = load_module(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"), "crates_mutation_rows_for_prove_red")
# 副本锁（文件头「判法与次序」⓪）与 apply-writer-patch.py 同一把、同一份写法
COPY_LOCK = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "apply-writer-patch.py"), "apply_writer_patch_for_prove_red")
# 被改的包这一趟重编了没有（文件头「判法与次序」⑤）：与 merge-and-verify.sh 同一份判法
REBUILT = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_rebuilt_packages.py"), "lib_rebuilt_packages_for_prove_red")
# 副本里起 cargo 的环境（多机配置显式交过去）、核副本缺不缺（entries_missing_in_copy）与建副本的出路：拷贝范围的共用库
REPOSITORY_COPY = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "lib_repository_copy.py"), "lib_repository_copy_for_prove_red")
# 重测试目标只许 release（文件头「判法与次序」的「重测试目标」一条）：判法只在这份共用库，与门禁 crates-mutation-replay 同用
RELEASE_ONLY_TESTS = load_module(os.path.join(SCRIPT_DIRECTORY, "lib_release_only_tests.py"), "lib_release_only_tests_for_prove_red")
# 各路目录的删法（文件头「判法与次序」的「并行」一条）：与起 cargo 的入口脚本删编译目录同一份
RUN_BUILD_DIRECTORY = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run_build_directory.py"), "run_build_directory_for_prove_red")
COPY_HOWTO = ("python3 scripts/hosts/minimal-copy.py <仓根> <草稿目录>/copy --with-test-inputs 拷一份（拷贝范围取 scripts/hosts/lib_repository_copy.py 的闭包，"
              "连测试读的仓根文件），再给 --copy")
# 核副本缺不缺时闭包照哪棵树算：平常是 prove 的 main_root（主工作区）；自证开头放一棵临时的假主工作区进来，各格的假副本照它核（文件头「判法与次序」①）
CLOSURE_ROOT_FOR_SELFTEST = []
# 判定库的库根（文件头「判定库与复用」）：平常是这份脚本所在的主工作区 MAIN_ROOT；自证开头放一棵自己 git init 的临时仓进来
LIBRARY_ROOT_FOR_SELFTEST = []
SCRIPT_KEY = "scripts/mutation/prove-red.sh"   # 复用三问的规范输出与复用哨兵认的脚本名（.claude/gate.d/verdict-library-scripts.tsv 里这一行的路径）
SENTINEL_LABEL = "证红"
# 每条判定那一行末尾的标（文件头「判定库与复用」）：现判（带栏）、复用、强制现判、拒算
ROW_STATE = re.compile(r"（(现判|复用|强制现判|拒算)(?:：[^（）]*)?）$")
HOW_TEXT = "prove-red.sh 现判（直接起 cargo，不设上限与限时）"   # 判定库记录里的跑法：只是一样属性，不进键
# 并行（文件头「判法与次序」的「并行」一条）
LANES_VARIABLE = "PROVE_RED_LANES"
LANES_ROOT_PREFIX = "singlefs-prove-red-lanes-"
LANE_COPY_IGNORED = shutil.ignore_patterns("target", "prove-red-logs", COPY_LOCK.COPY_LOCK_NAME, COPY_LOCK.COPY_OWNER_NAME,
                                           RESTORE_JOURNAL_NAME, RESTORE_JOURNAL_NAME + ".tmp", LANES_ROOT_PREFIX + "*")
Lane = collections.namedtuple("Lane", "number rows copy environment log_prefix")
RowResult = collections.namedtuple("RowResult", "lines failed caught not_run fresh reported")   # 一行现判的结局（fresh 是交复用哨兵的那一份，没出结局的是 None）
DROP_LAST_LANE_FOR_SELFTEST = []   # 自证「并行收回数核对」那一格放一个值进来：变异那一段丢掉最后派出的那一路的结果


def own_table_rows(path):
    """弄坏开关 own-table：六段就收，哪一段空着都不管。"""
    rows = {}
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        if line.startswith("#") or not line.strip():
            continue
        fields = line.rstrip("\n").split("\t")
        if len(fields) == 6:
            rows[fields[0]] = ROWS.Row(line_number, fields[0], fields[1], ROWS.unescape(fields[2]), ROWS.unescape(fields[3]), fields[4], fields[5], tuple(fields))
    return rows, {}


def table_rows(copy):
    """({变异名: 成形的那一行}, {变异名: 为什么不成形})：照 crates-mutation-rows.py 的 read_table 读；同名的取表里最后一行。"""
    path = os.path.join(copy, "crates", "mutations.tsv")
    if BROKEN == "own-table":
        return own_table_rows(path)
    rows, malformed = ROWS.read_table(path)
    return {row.name: row for row in rows}, {name: reason for _line_number, name, reason in malformed}


def refusal_for(row, crate):
    arguments = row.arguments.split()
    before_separator = arguments[: arguments.index("--")] if "--" in arguments else arguments
    if BROKEN != "any-target" and not TARGET_OPTIONS & set(before_separator):
        return f"参数「{row.arguments}」没挑目标（--lib / --test / --bin 都没有），会把这个包的全部测试目标都编、都跑"
    if "-p" not in before_separator or before_separator[before_separator.index("-p") + 1:before_separator.index("-p") + 2] != [crate]:
        return f"参数「{row.arguments}」不是 -p {crate}"
    return None


def default_features_off_rows(chosen, environment):
    """(行, [要报的行])：验证包只点名集成测试目标的行换成补了 --no-default-features 的（文件头「判法与次序」的「验证包的测试目标不编装置二进制」一条），
    判法是 crates-mutation-rows.py 的 default_features_off_row（门禁 plan_rows 同一个函数，在补 --release 与补 gpu 特性之前）；别的原样。
    弄坏开关 no-default-features-unadded 下一律原样。"""
    if BROKEN == "no-default-features-unadded":
        return chosen, []
    off, notes = [], []
    for row in chosen:
        replaced = ROWS.default_features_off_row(row, environment)
        if replaced.arguments != row.arguments:
            notes.append(f"- {row.name}：cargo 参数补上 --no-default-features 起（验证包的测试目标不编装置二进制）")
        off.append(replaced)
    return off, notes


def release_forced_rows(chosen, registry_root):
    """(行, [要报的行])：命中重测试目标而 cargo 参数没带 release 的行换成补了 --release 的（文件头「判法与次序」的「重测试目标」一条），
    别的原样；登记表读 registry_root 那一份，读不了抛共用库的 RegistryUnreadable。弄坏开关 release-unforced 下一律原样。"""
    if BROKEN == "release-unforced":
        return chosen, []
    registered = RELEASE_ONLY_TESTS.crash_case_targets(registry_root)
    forced, notes = [], []
    for row in chosen:
        arguments, requirement = RELEASE_ONLY_TESTS.release_forced(row.arguments, registered=registered)
        if arguments != row.arguments:
            notes.append(f"- {row.name}：cargo 参数补上 --release 起（{'；'.join(requirement.basis)}）")
            row = row._replace(arguments=arguments)
        forced.append(row)
    return forced, notes


def source_path_problem(copy, path):
    root = os.path.realpath(copy)
    resolved = os.path.realpath(os.path.join(root, path))
    if os.path.isabs(path) or os.path.commonpath([root, resolved]) != root or resolved == root:
        return f"文件 {path} 不在副本内，不能施加变异"
    try:
        if not os.path.isfile(resolved):
            return f"文件 {path} 不是普通文件"
        if os.stat(resolved).st_nlink != 1:
            return f"文件 {path} 有硬链接，不能保证变异只改副本"
    except OSError as error:
        return f"文件 {path} 读不了：{error}"
    return None


def own_verdict(row, code, text):
    """弄坏开关 own-judge：prove-red 自己那一份判法（任一行点名的 FAILED 就算抓到，不看退出码、不看对上几个用例、不认 NOT_RUN）。"""
    if re.search(r"^test (?:\S*::)?" + re.escape(row.expected) + r"(?: - should panic)? \.\.\. FAILED", text, re.M):
        return "caught", ""
    if re.search(r"^error\[E\d+\]:", text, re.M) or "could not compile" in text:
        return "invalid", ""
    if re.search(r"^error:", text, re.M) and not re.search(r"^test result: ", text, re.M):
        return "invalid", ""
    return "failure", ""


def verdict_of(row, code, text):
    """(档, 那一句)：照 crates-mutation-rows.py 的 verdict_of_run（门禁同一份）。"""
    if BROKEN == "own-judge":
        return own_verdict(row, code, text)
    return ROWS.verdict_of_run(row, code, text, environment=os.environ)


def first_sentence(text):
    """verdict_of_run 那一句的头一行，去掉「crates/mutations.tsv:<行号> <名>：」。"""
    head = (text.splitlines() or [""])[0]
    return head.split("：", 1)[1] if "：" in head else head


def judged_line(row, code, text, log_path):
    """(caught / failed / not-run, 要打的那一行)。"""
    verdict, sentence = verdict_of(row, code, text)
    if verdict == "caught" and BROKEN != "never-caught":
        return "caught", f"{row.name}\t抓到\t{row.expected} 红了（日志 {log_path}）"
    if verdict == "linkfailed" and BROKEN != "link-failure-listed-as-invalid":
        return "failed", (f"✗ {row.name}\t链接失败\t替换之后编过了、失败在链接，不是替换文编不过：删掉这一趟用的编译目录（调用方给的 CARGO_TARGET_DIR；"
                          f"没给的这一趟自己建的已经删了），重跑这一行（日志 {log_path}）")
    if verdict in ("invalid", "linkfailed") or (verdict == "failure" and BROKEN == "error-line-invalid" and re.search(r"^error(?:\[E\d+\])?:", text, re.M)):
        return "failed", f"✗ {row.name}\t无效\t替换之后编不过（日志 {log_path}）"
    if verdict in ("failure", "caught"):
        detail = first_sentence(sentence) if verdict == "failure" and sentence else f"{row.expected} 没判红"
        return "failed", f"✗ {row.name}\t没红\t{detail}（日志 {log_path}）"
    if verdict == "notrun":
        return "not-run", f"- 本次未跑 {row.name}：{first_sentence(sentence)}（日志 {log_path}）"
    return "failed", f"✗ {row.name}：结局认不出（verdict_of_run 判成 {verdict}，退出码 {code}）（日志 {log_path}）"


def named_packages(row):
    """cargo test 参数里 -p / --package 点名的包（-- 之前）。"""
    words = row.arguments.split()
    words = words[:words.index("--")] if "--" in words else words
    packages = []
    for index, word in enumerate(words):
        if word in ("-p", "--package") and index + 1 < len(words):
            packages.append(words[index + 1])
        elif word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
    return packages


def packages_reading_the_file(copy, relative, target_directory):
    """编译目录里 dep-info 列着被改文件的那几个包：一份 dep-info 列的头一个源文件是那个编译单元的根文件，它所在的包就是读这份文件的包。
    被改文件经 #[path]（或 include_str! 一类）编进别的包时，重编的是那个包，不是被改文件自己所在的包。"""
    wanted = {os.path.normpath(os.path.join(os.path.realpath(copy), relative)), os.path.normpath(os.path.join(os.path.abspath(copy), relative))}
    roots = {os.path.abspath(copy), os.path.realpath(copy), REBUILT.workspace_root_of(copy, relative)}   # dep-info 里的相对路径以工作区根解

    def resolved(word):
        return [os.path.normpath(word)] if os.path.isabs(word) else [os.path.normpath(os.path.join(root, word)) for root in roots]

    readers = set()
    for path in REBUILT.dep_info_files(target_directory):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        if os.path.basename(relative) not in text:
            continue
        words = [token.rstrip(":") for token in text.split()]
        if len(words) < 2 or not wanted & {candidate for word in words[1:] for candidate in resolved(word)}:
            continue
        for candidate in resolved(words[1]):
            inside = os.path.relpath(os.path.realpath(candidate), os.path.realpath(copy))
            package = None if inside.startswith("..") else REBUILT.package_of(copy, inside)
            if package is not None:
                readers.add(package)
                break
    return readers


def included_package_rebuilt(copy, row, text, target_directory):
    """被改文件经 #[path]（或 include_str!）编进别的包时的重编核对（文件头「判法与次序」⑤）：-p 点名的包里 dep-info 列着被改文件的，
    这一趟输出里有它的 Compiling 行，就算被改文件重编过。"""
    if not target_directory:
        return False
    readers = set(named_packages(row)) & packages_reading_the_file(copy, row.path, target_directory)
    return bool(readers & REBUILT.compiled_packages(text))


def rebuild_problem(copy, row, code, text, log_path, target_directory=None):
    """施加变异之后那一趟真重编了被改的包没有（文件头「判法与次序」⑤）：没有交要打的那一行，有（或这一趟不核）交 None。"""
    if BROKEN == "no-rebuild-check":
        return None
    if target_directory and not os.path.isabs(target_directory):   # 相对路径是 cargo 照它的当前目录（副本根）解的
        target_directory = os.path.join(copy, target_directory)
    packages, _note = REBUILT.packages_to_check(copy, [row.path], target_directory)
    if not packages:   # 改的不是包里的 .rs、或这一趟编译不读它（不在编译目录的任何 dep-info 里）：cargo 本来就不因它重编，不核
        return None
    package = packages[0]
    if REBUILT.missing_rebuilds([package], [text]):
        if BROKEN != "path-included-package-ignored" and included_package_rebuilt(copy, row, text, target_directory):
            return None
        return (f"✗ {row.name}\t没重编\t改了 {row.path}，这一趟的输出里却没有「Compiling {package} v…」行：编译目录里的产物比改过的源码新，"
                f"测试跑的是旧产物，判定不算数；刷源码修改时刻或换干净编译目录（不设 CARGO_TARGET_DIR，这一趟自己建一份新的）重跑（日志 {log_path}）")
    return None


def cargo_environment(copy, target_directory=None):
    """cargo 那几次的环境：调用方设了（非空）CARGO_TARGET_DIR 照它的（归调用方、这里不删）；没设就不放，prove 拿到副本锁之后给这一趟自己建一份（new_trip_build_directory，跑完删）。
    证红要这一趟自己专用的编译目录，不与别的编译共用、不沿用副本里别的命令留下的 target（.claude/rules/test-script-discipline.md「编译与算法都要快」）：
    每条变异改坏源码再还原，还原后的文件 mtime 可能比变异那次编出的产物旧，cargo 就把变异产物当成新的接着用——共用目录里另一份副本的测试就会跑在上一条变异的库上；
    别的命令编出的产物与这一趟的编译参数对不上时，头一条变异链接不过（链接器报未定义的符号）。
    弄坏开关 caller-target-overridden：调用方设了的也换成 <副本>/target（target_directory 给的取代它；调用方没设的不动、照旧自己建一份）；copy-target-reused：没设时用 <副本>/target、不另建。
    副本里没有多机配置（它是 minimal-copy.py git init 的一棵新仓，不带 multi-host.env）：多机配置照 lib_repository_copy.py 的 environment_for_commands_in_the_copy 显式交给 SINGLEFS_MULTI_HOST_CONFIG（取这份脚本所在的主工作区那一份）。
    构建档照门禁关调试信息（crates-mutation-rows.py 的 cargo_debug_overrides，文件头「判定库与复用」）：构建档与线程数不进行键，这里只定起的 cargo 与门禁起的一样。"""
    environment = REPOSITORY_COPY.environment_for_commands_in_the_copy(MAIN_ROOT)
    if (BROKEN == "caller-target-overridden" and environment.get("CARGO_TARGET_DIR")) or (BROKEN == "copy-target-reused" and not environment.get("CARGO_TARGET_DIR")):
        environment["CARGO_TARGET_DIR"] = target_directory or os.path.join(copy, "target")
    environment.update(ROWS.cargo_debug_overrides(environment))
    return environment


def baseline_environment(environment):
    """基线那几次起 cargo 的环境（文件头「判定库与复用」的基线一条）：不带门禁加的 CARGO_PROFILE_RELEASE_INCREMENTAL，调用方自己设了的值照放回、没设就不放
    （crates-mutation-rows.py 的 baseline_cargo_environment，门禁的基线同用这一份）。增量编译开着时 codegen-units 的生效值是 256，崩溃判定的版本与门禁 crash-point-replay 的
    cargo test --release 不是同一把，基线读不到真库里那一趟判过的块。弄坏开关 baseline-incremental-kept：基线也带这一项覆盖。"""
    if BROKEN == "baseline-incremental-kept":
        return environment
    return ROWS.baseline_cargo_environment(environment, os.environ.get("CARGO_PROFILE_RELEASE_INCREMENTAL"))


TRIP_BUILD_DIRECTORY_PREFIX = "singlefs-prove-red-target-"
TRIP_BUILD_DIRECTORIES = []   # 这一趟自己建、还没删的编译目录：被 TERM、HUP 停时 stopped_by_signal 照它删


def new_trip_build_directory():
    """调用方没给 CARGO_TARGET_DIR 时这一趟自己的编译目录（文件头「判法与次序」的「编译目录归这一趟」）：建在这一台自己的临时目录下
    （副本在工作根的共享盘上，编译目录不放过去，也不进副本的 target），名字随机，每一趟一份新的。建不了抛 OSError。"""
    directory = os.path.realpath(tempfile.mkdtemp(prefix=TRIP_BUILD_DIRECTORY_PREFIX))
    TRIP_BUILD_DIRECTORIES.append(directory)
    return directory


def remove_trip_build_directory(directory):
    """收尾删这一趟自己建的编译目录（成功、红、被打断都走）：→ (删成了没有, [要打的行])。directory 是 None（调用方给了编译目录，归调用方删）原样交 (True, [])；
    删成了不打字（输出与调用方给了编译目录时一样多）；有进程开着里面的文件、删不成交 (False, 原因与出路)。弄坏开关 trip-target-kept：不删。"""
    if directory is None or BROKEN == "trip-target-kept":
        return True, []
    removed, lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(directory)
    if removed:
        TRIP_BUILD_DIRECTORIES.remove(directory)
    return removed, [] if removed else lines


def run_cargo_to_log(copy, arguments, log_path, cargo, environment, mode):
    """cargo test <参数> 起一次，输出写进日志（mode：w 覆盖、a 接着写）：→ (退出码, 日志全文)。"""
    with open(log_path, mode, encoding="utf-8") as log:
        result = subprocess.run([cargo, "test", *arguments], cwd=copy, stdout=log, stderr=subprocess.STDOUT,
                                env=environment)
    return result.returncode, open(log_path, encoding="utf-8", errors="replace").read()


def run_cargo(copy, arguments, log_path, cargo, environment):
    """cargo 是调用方交的：平常是 PATH 上的 cargo，自证给它写出的假 cargo 桩。这一处登在 scripts/gate/lib_function_units.py 的 UNKEYED_PROGRAM_WORDS：
    真 cargo 编的 crates 由每条变异行的行键管，假 cargo 的正文在自证里，单元闭包不往上认这个参数。
    不领卡（文件头「不领卡」）：带不带 gpu 特性都照原样起、不另编，环境里的 GPU_CARDS_BY_PRIORITY 有就原样交。
    弄坏开关 gpu-rows-compiled-apart：带 gpu 特性的先另起一条只编的（--no-run，输出写进同一份日志），编过了再起这一条；
    gpu-rows-cards-dropped：带 gpu 特性的起之前去掉环境里的 GPU_CARDS_BY_PRIORITY。"""
    gpu_row = BROKEN in ("gpu-rows-compiled-apart", "gpu-rows-cards-dropped") and ROWS.gpu_test_run(["cargo", "test", *arguments], environment)
    if gpu_row and BROKEN == "gpu-rows-cards-dropped":
        environment = {name: value for name, value in environment.items() if name != "GPU_CARDS_BY_PRIORITY"}
    mode = "w"
    if gpu_row and BROKEN == "gpu-rows-compiled-apart":
        separator = arguments.index("--") if "--" in arguments else len(arguments)
        code, text = run_cargo_to_log(copy, [*arguments[:separator], "--no-run"], log_path, cargo, environment, mode)
        if code != 0:
            return code, text
        mode = "a"
    return run_cargo_to_log(copy, arguments, log_path, cargo, environment, mode)


MUTATED_RUN_SERVICE_ROOT_PREFIX = "singlefs-prove-red-service-"
MUTATED_RUN_SERVICE_ROOTS = []   # 这一趟自己建、还没删的变异服务根（临时服务的目录与编出来的 prune-server 都在里面）：被 TERM、HUP 停时 stopped_by_signal 照它删


def prepare_mutated_run_service():
    """变异那几趟各起一份临时 prune-server 之前的准备（文件头「判定库与复用」的变异一条）：在这一台自己的临时目录下建这一趟的服务根，取服务二进制：
    → ({"root": 服务根, "binary": 二进制路径或 None}, 取不到的原因或 None)。二进制照 crates-mutation-rows.py 的 mutation_run_prune_server_binary 取：PRUNE_SERVER_BINARY 设了照它
    （绝对路径、可执行），没设照这份脚本所在仓的 prune-server/ 编一份进服务根；编在这份脚本所在的仓根里（cargo 从起它的目录往上找 .cargo/config.toml，librocksdb-sys 靠它），编完回到原来的目录。
    取不到时服务根照样交回，由调用方收尾删。弄坏开关 mutated-run-on-real-service：不建、不取，交 ({"root": None, "binary": None}, None)。"""
    if BROKEN == "mutated-run-on-real-service":
        return {"root": None, "binary": None}, None
    root = os.path.realpath(tempfile.mkdtemp(prefix=MUTATED_RUN_SERVICE_ROOT_PREFIX))
    MUTATED_RUN_SERVICE_ROOTS.append(root)
    previous_directory = os.getcwd()
    os.chdir(MAIN_ROOT)
    try:
        binary, problem = ROWS.mutation_run_prune_server_binary(dict(os.environ), root)
    finally:
        os.chdir(previous_directory)
    return {"root": root, "binary": binary}, (None if binary else problem)


def remove_mutated_run_service_root(service):
    """收尾删这一趟的服务根（成功、红、被打断都走）；没建的（root 是 None）不动。"""
    root = service["root"]
    if root is None:
        return
    shutil.rmtree(root, ignore_errors=True)
    if root in MUTATED_RUN_SERVICE_ROOTS:
        MUTATED_RUN_SERVICE_ROOTS.remove(root)


@contextlib.contextmanager
def mutated_run_prune_server(service, arguments):
    """改坏之后起的那一次 cargo test 自己的临时 prune-server（文件头「判定库与复用」的变异一条）：交要加进那一次 cargo 环境的三个变量（连哪一处与临时服务的标记，都指它），
    收尾停服务、删它的目录；起法与门禁的变异复跑同一个函数（crates-mutation-rows.py 的 mutation_run_prune_server）。service 里没有二进制（弄坏开关 mutated-run-on-real-service）交空，
    那一次照调用方的环境连那一份服务。服务起不来抛 RuntimeError，不吃掉（judge_row 的 finally 先把源码写回）。"""
    if service["binary"] is None:
        yield {}
        return
    with ROWS.mutation_run_prune_server(service["binary"], ["cargo", "test", *arguments], service["root"], {}) as settings:
        yield settings


def stopped_by_signal(signal_number, _frame):
    """被停（文件头「编译目录归这一趟」）：删这一趟自己建的编译目录与变异服务根（进程马上按默认处理停，起 cargo 的子进程开着的文件不再等，不核开着的进程；
    临时服务由包装设的父进程死亡信号随进程一起停），再按默认处理重发同一个信号。弄坏开关 trip-target-kept-on-stop：编译目录不删。"""
    for directory in list(TRIP_BUILD_DIRECTORIES) if BROKEN != "trip-target-kept-on-stop" else []:
        removed, removal_lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(directory, check_open=False)
        if not removed:
            print("\n".join(removal_lines), file=sys.stderr)
    for root in list(MUTATED_RUN_SERVICE_ROOTS):
        shutil.rmtree(root, ignore_errors=True)
    signal.signal(signal_number, signal.SIG_DFL)
    os.kill(os.getpid(), signal_number)


def remove_trip_build_directories_when_stopped():
    """TERM、HUP 都走 stopped_by_signal；只在主线程里设得了。"""
    for signal_number in (signal.SIGTERM, signal.SIGHUP):
        signal.signal(signal_number, stopped_by_signal)


def normalized_text(original_bytes):
    """锚点按文本模式匹配（通用换行）；写回用原始字节，保留 CRLF。"""
    return original_bytes.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")


def mutated_bytes_of(original_bytes, old, new):
    """施加变异之后那份文件的字节（⑤ 施加与还原日志认「施加之后的样子」同用这一份）。"""
    return normalized_text(original_bytes).replace(old, new, 1).encode("utf-8")


def write_restore_journal(copy, row, original_bytes):
    """施加之前在副本根落还原日志（文件头「判法与次序」的「还原日志」一条）：先写临时文件、落盘再换上，被杀在中途时要么没有、要么是整份。"""
    if BROKEN == "no-restore-journal":
        return
    record = {"name": row.name, "path": row.path, "old": row.old, "new": row.new,
              "original_sha256": hashlib.sha256(original_bytes).hexdigest(),
              "original_base64": base64.b64encode(original_bytes).decode("ascii")}
    journal = os.path.join(copy, RESTORE_JOURNAL_NAME)
    with open(journal + ".tmp", "w", encoding="utf-8") as handle:
        json.dump(record, handle, ensure_ascii=False)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(journal + ".tmp", journal)


def drop_restore_journal(copy):
    for leftover in (RESTORE_JOURNAL_NAME, RESTORE_JOURNAL_NAME + ".tmp"):
        try:
            os.unlink(os.path.join(copy, leftover))
        except FileNotFoundError:
            pass


def restore_unrestored_mutation(copy):
    """拿到副本锁之后、做 ② 之前照还原日志把上一趟没还原的变异写回（文件头「判法与次序」的「还原日志」一条）。
    → (None, [要报的行])：没有日志，或写回了 / 本来就是原文；(2, [拒绝行])：日志读不了、文件两样都不是。"""
    journal = os.path.join(copy, RESTORE_JOURNAL_NAME)
    if BROKEN == "no-restore-journal" or not os.path.exists(journal):
        return None, []
    clean_copy_way = "→ 怎么办：照还原日志里的文件与原文核对那份文件（diff -rq 对一份干净副本），盖回原文、删掉日志再证；或换一份干净副本；一条都没跑"
    try:
        with open(journal, encoding="utf-8") as handle:
            record = json.load(handle)
        name, relative, old, new = record["name"], record["path"], record["old"], record["new"]
        if not all(isinstance(value, str) for value in (name, relative, old, new)):
            raise TypeError("日志里的变异名、文件、原文、替换文不全是字符串")
        original_bytes = base64.b64decode(record["original_base64"], validate=True)
        if hashlib.sha256(original_bytes).hexdigest() != record["original_sha256"]:
            raise ValueError("日志里的原字节与它记的 sha256 对不上")
    except (OSError, ValueError, KeyError, TypeError) as error:
        return 2, [f"✗ 副本里有上一趟证红留下的还原日志 {journal}，读不了：{error}", clean_copy_way]
    problem = source_path_problem(copy, relative)
    if problem:
        return 2, [f"✗ 还原日志 {journal} 点名的{problem}", clean_copy_way]
    path = os.path.join(copy, relative)
    with open(path, "rb") as handle:
        current = handle.read()
    if current == original_bytes:   # 写回之后、删日志之前被打断
        drop_restore_journal(copy)
        return None, [f"- 上一趟的还原日志还在、{relative} 已是施加 {name} 之前的原文：删了日志"]
    mutated = mutated_bytes_of(original_bytes, old, new)
    if not (mutated.startswith(current) or original_bytes.startswith(current)):   # 前缀：写到一半被杀
        return 2, [f"✗ 副本里 {relative} 既不是上一趟证红施加 {name} 之前、也不是施加之后的样子（还原日志 {journal}）：被打断之后又被改过", clean_copy_way]
    with open(path, "wb") as handle:
        handle.write(original_bytes)
    os.utime(path)
    drop_restore_journal(copy)
    return None, [f"- 上一趟没还原的变异已写回：{name}（{relative}，还原日志 {journal}）"]


def old_text_misses(copy, chosen):
    """基线之前核点名的每一行原文在副本源码里恰好命中一次（文件头「判法与次序」的「先核原文」一条）：→ [拒绝行]。"""
    if BROKEN == "old-text-checked-late":
        return []
    misses = []
    for row in chosen:
        with open(os.path.join(copy, row.path), "rb") as handle:
            hits = normalized_text(handle.read()).count(row.old)
        if hits == 0:
            misses.append(f"✗ {row.name}：原文在 {row.path} 里命中 0 次，要恰好 1 次——副本里可能留着别的变异，照还原日志或换一份干净副本"
                          f"（diff -rq 对一份干净副本看哪份文件不同）；副本是干净的，就是这一行的原文与源码对不上，照源码改原文")
        elif hits != 1:
            misses.append(f"✗ {row.name}：原文在 {row.path} 里命中 {hits} 次，要恰好 1 次——把 crates/mutations.tsv 里这一行的原文写长到只命中一处")
    return misses


def copy_closure_refusal(copy, closure_root):
    """①（文件头「判法与次序」）的后半：按 lib_repository_copy.py 的同一份闭包（closure_root 那棵树、带测试输入）核副本根下缺不缺闭包里的条目。
    缺的、闭包算不成的交 (2, [输出行])；不缺交 None。弄坏开关 copy-closure-unchecked 下不核。"""
    if BROKEN == "copy-closure-unchecked":
        return None
    try:
        missing, checked = REPOSITORY_COPY.entries_missing_in_copy(closure_root, copy, test_inputs=True)
    except (ValueError, OSError) as error:   # ClosureRefused 是 ValueError 的子类
        return 2, [f"✗ 算不出 {closure_root} 的副本闭包，判不了副本 {copy} 缺不缺：{error}",
                   f"→ 怎么办：python3 scripts/hosts/lib_repository_copy.py entries {closure_root} --test-inputs 看它报什么、照它的出路修；一条都没跑"]
    if not missing:
        return None
    return 2, [f"✗ 副本 {copy} 缺 {len(missing)} 条闭包里的仓根条目（闭包共 {checked} 条，照 {closure_root} 带测试输入算）："
               + "；".join(f"{entry}（{'；'.join(reasons)}）" for entry, reasons in missing),
               f"→ 怎么办：{REPOSITORY_COPY.copy_rebuild_howto(closure_root)}，再给 --copy；一条都没跑"]


class LibraryPlanRefused(Exception):
    """行键或判定库算不出（文件头「判定库与复用」末句）：lines 是整次拒绝要打的那几行。"""

    def __init__(self, lines):
        super().__init__(lines[0])
        self.lines = lines


def key_peers(copy, chosen, registry_root, environment):
    """表里没点名、而原文与替换文都与点名的某一行相同的行（行键撞得上的只有这几行；crates-mutation-rows.py 的 plan_rows 同一取法），
    照点名的行一样先补 --no-default-features（验证包只点名集成测试目标的）、再补 --release（命中重测试目标的）。"""
    planned = {row.line_number for row in chosen}
    texts = {(row.old, row.new) for row in chosen}
    rows, _malformed = ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))
    peers = default_features_off_rows([row for row in rows if row.line_number not in planned and (row.old, row.new) in texts], environment)[0]
    return release_forced_rows(peers, registry_root)[0]


def gpu_route_notes(key_context, before, after):
    """补了 gpu 特性的行与补了之后不编的行各打一行（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条）：before、after 是补之前与之后的点名的行（点名次序）。"""
    planned = {row.line_number: (row, after_row) for row, after_row in zip(before, after)}
    notes = []
    for line_number, feature in key_context.gpu_route_lines:
        if line_number in planned:
            row, featured = planned[line_number]
            released = "--release" in featured.arguments.split() and "--release" not in row.arguments.split()
            notes.append(f"- {row.name}：cargo 参数补上 --features {feature} 起（多机配置 {key_context.gpu_route_configuration} 开了 GPU（ENABLE_GPU=1），"
                         f"点名测试的单元闭包够得着崩溃判段的选路函数；同门禁 crates-mutation-replay 的 gpu_route_rows）"
                         + ("，补了 gpu 特性就命中重测试目标、另补 --release" if released else ""))
    for line_number, why in key_context.gpu_route_unbuilt:
        if line_number in planned:
            notes.append(f"- {planned[line_number][0].name}：点名的测试走崩溃判段，补 gpu 特性之后不编（{why}），照原样参数起")
    return notes


def library_plan(copy, chosen, registry_root, environment):
    """(KeyContext, 这一趟的点名的行, {行号: Components}, 判定库, {行号: (Decision, 记录)}, [补 gpu 特性要报的行])：照 crates-mutation-rows.py 的 KeyContext、
    gpu_route_rows、refuse_colliding_row_keys、open_library 与 decide_rows（文件头「判定库与复用」）。库根平常是 MAIN_ROOT，自证换成 LIBRARY_ROOT_FOR_SELFTEST 里那一棵。
    交回的行：多机配置开了 GPU 时点名测试走崩溃判段、cargo 参数没开 gpu 的那几行换成补了 gpu 特性的（门禁 plan_rows 同一个函数），别的原样；键照换了之后的参数算。
    弄坏开关 library-unread 下键算得出的行一律判现判、不读 records/ 与索引；gpu-route-feature-unadded 下不补 gpu 特性。算不出抛 LibraryPlanRefused。"""
    library_root = LIBRARY_ROOT_FOR_SELFTEST[0] if LIBRARY_ROOT_FOR_SELFTEST else MAIN_ROOT
    function_units = ROWS.mutation_units.function_units()
    route_notes = []
    try:
        peers = key_peers(copy, chosen, registry_root, environment)
        key_context = ROWS.KeyContext(copy, environment)
        key_context.release_registered = RELEASE_ONLY_TESTS.crash_case_targets(registry_root)   # 补了 gpu 特性的行再照判一次重测试目标（gpu_route_rows 里）
        key_context.prepare(list(chosen) + peers)
        components = {row.line_number: key_context.components(row) for row in list(chosen) + peers}
        if BROKEN != "gpu-route-feature-unadded":
            every_row, components = ROWS.gpu_route_rows(key_context, list(chosen) + peers, components, environment)
            routed = {row.line_number: row for row in every_row}
            before, chosen = list(chosen), [routed[row.line_number] for row in chosen]
            route_notes = gpu_route_notes(key_context, before, chosen)
        key_context.collisions = ROWS.refuse_colliding_row_keys(components, environment)
        library = ROWS.open_library(library_root, environment)
    except ROWS.mutation_units.ClosureRefused as error:
        raise LibraryPlanRefused([f"✗ 点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），行键算不出来：一条都没跑"]
                                 + [f"   {line}" for line in error.detail_lines()]
                                 + [f"→ 怎么办：{''.join(ROWS.CLOSURE_REFUSED_HOWTO)}"]) from error
    except (ROWS.mutation_units.MutationUnitsError, ROWS.admission_module().InputManifestError, ROWS.HeavyThreadDefaultError,
            function_units.FunctionUnitsError, function_units.UsageError, RELEASE_ONLY_TESTS.RegistryUnreadable, OSError, UnicodeDecodeError) as error:
        raise LibraryPlanRefused([f"✗ 算不出行键或判定库的位置（{type(error).__name__}：{error}），判定库没处对：一条都没跑",
                                  f"→ 怎么办：库根 {library_root} 要是 git 工作树、副本的根 Cargo.toml 读得懂；"
                                  f"python3 scripts/mutation/crates-mutation-rows.py plan {MAIN_ROOT} 看它报什么"]) from error
    chosen_components = {row.line_number: components[row.line_number] for row in chosen}
    if BROKEN == "library-unread":
        decided = {line_number: (ROWS.Decision(False, "refused", comp.refusal) if comp.row_key is None
                                 else ROWS.Decision(False, "unexplained", "弄坏开关 library-unread：没读库"), None)
                   for line_number, comp in chosen_components.items()}
    else:
        decided = ROWS.decide_rows(library, chosen, chosen_components, key_context, environment)
    return key_context, chosen, chosen_components, library, decided, route_notes


def planned_sentinel(library, chosen, components, environment):
    """复用哨兵（文件头「判定库与复用」）：→ (哨兵, 要打的行)。候选是此刻行键在 records/ 里记着抓到的行（直接读记录，与 decide_rows 分开），
    项名照门禁的 sentinel_item。弄坏开关 sentinel-unplanned 下不抽。缩过域的一趟不抽（文件头「缩过域的一趟」；弄坏开关 narrowed-sentinel-drawn 下照抽）。"""
    sentinel = ROWS.reuse_checks.ReuseSentinel(SCRIPT_KEY, SENTINEL_LABEL, environment=os.environ)
    lines = []
    if BROKEN == "sentinel-unplanned":
        sentinel.plan({}, reuse=False, off_reason="弄坏开关 sentinel-unplanned", printer=lines.append)
        return sentinel, lines
    narrowing = ROWS.crash_domain_narrowing(environment)
    if narrowing and BROKEN != "narrowed-sentinel-drawn":
        sentinel.plan({}, reuse=False, off_reason=ROWS.narrowed_sentinel_off_reason(narrowing), printer=lines.append)
        return sentinel, lines
    candidates = {}
    for row in chosen:
        key = components[row.line_number].row_key
        record = library.read("records", key) if key else None
        if record is not None and record.get("row_key") == key and record.get("verdict") == ROWS.REUSABLE_VERDICT["mutation"]:
            candidates[ROWS.sentinel_item(row)] = "抓到"
    sentinel.plan(candidates, printer=lines.append)
    return sentinel, lines


def state_text(decision):
    """每条判定那一行末尾标的那几个字（文件头「判定库与复用」）：复用、强制现判、拒算，别的是「现判：<栏>」。"""
    if decision.reuse:
        return "复用"
    if decision.column == "sentinel":
        return "强制现判"
    if decision.column == "refused":
        return "拒算"
    return f"现判：{ROWS.COLUMN_TITLES.get(decision.column, decision.column)}"


def record_verdict(library, row, comp, verdict, text, code, seconds, test_full_name, environment):
    """照行键写一条判定（文件头「判定库与复用」；字段与 crates-mutation-rows.py 的 judge_row 同形，这里没有每条自己的 TMPDIR，删 TMPDIR 那几个数写 0）：
    → 写不进去时要打的那一行，写了或不写交 None。拒算的不写；本次未跑不盖同一把键原有的算数结论；弄坏开关 red-not-recorded 下只写抓到。
    缩过域的一趟一条都不写（文件头「缩过域的一趟」；弄坏开关 narrowed-recorded 下照写）。"""
    if not comp.row_key:
        return None
    if ROWS.crash_domain_narrowing(environment) and BROKEN != "narrowed-recorded":
        return None
    if verdict != ROWS.REUSABLE_VERDICT["mutation"] and BROKEN == "red-not-recorded":
        return None
    if verdict in ROWS.VERDICTS_UNCOUNTED and ROWS.uncounted_verdict_keeps_counted_record(
            library.read("records", comp.row_key), verdict, comp.row_key, environment):
        return None
    light = ROWS.light_components(comp)
    fields = collections.OrderedDict(
        kind="mutation", row_key=comp.row_key, verdict=verdict, text=text,
        removal_files=0, removal_allocated_bytes=0, removal_apparent_bytes=0, removal_left=0,
        exit_code=code, judged_date=ROWS.mutation_units.today_text(), how=HOW_TEXT, test_full_name=test_full_name,
        test_closure=light["test_closure"], mutated=light["mutated"],
        seconds_running=f"{seconds:.3f}")
    try:
        library.write("records", comp.row_key, fields)
    except OSError as error:
        return f"! {row.name}：判定没写进判定库（{library.path('records', comp.row_key)}：{error}），下一趟这一条照现判"
    return None


def record_index(library, rows, components):
    """复用与现判出结局的行照 index_entry 写索引（只用来让下一趟说清现判的原因）：→ 写不进去时要打的那几行。"""
    problems = []
    for row in rows:
        comp = components[row.line_number]
        if not comp.row_key:
            continue
        try:
            library.write("index", ROWS.row_identity(row), ROWS.index_entry(comp, row))
        except OSError as error:
            problems.append(f"! {row.name}：索引没写进判定库（{error}），下一趟键对不上时说不清变了什么")
    return problems


def build_jobs_total(environment):
    """这一趟的编译并行度总数（开头 python3 之前那几行定的 CARGO_BUILD_JOBS）：不是正整数抛 ValueError。"""
    jobs = environment.get("CARGO_BUILD_JOBS", "").strip()
    if not (jobs.isascii() and jobs.isdigit() and int(jobs) >= 1):
        raise ValueError(f"CARGO_BUILD_JOBS={jobs} 不是正整数")
    return int(jobs)


def lane_ceiling(environment):
    """同时至多开几路（文件头「判法与次序」的「并行」一条）：PROVE_RED_LANES 设了照它，没设是编译并行度总数；写错抛 ValueError。"""
    given = os.environ.get(LANES_VARIABLE, "").strip()
    total = build_jobs_total(environment)
    if not given:
        return total
    if given.isascii() and given.isdigit() and int(given) >= 1:
        return int(given)
    if BROKEN == "parallel-lanes-unchecked":
        return 1
    raise ValueError(f"{LANES_VARIABLE}={given} 不是正整数")


def row_needs_card(row):
    """这一行的 cargo 参数带 gpu 特性：判法只在 lib_release_only_tests.py（不给 crash-case 目标，依据就只剩 gpu 特性那一样）。"""
    return RELEASE_ONLY_TESTS.release_requirement(row.arguments, registered=()).required


def planned_lanes(rows, ceiling):
    """rows（点名次序）分成几路：→ [每路的行，点名次序]，几路按头一行的点名次序排（文件头「判法与次序」的「并行」一条）。
    改同一个文件的行、要卡的行并进同一组；组数比上限多时把大的组先分、每次放进此刻行数最少的那一路。弄坏开关 parallel-same-file-split、parallel-gpu-rows-split。"""
    leaders = list(range(len(rows)))

    def leader_of(index):
        while leaders[index] != index:
            index = leaders[index]
        return index

    group_holder = {}
    for index, row in enumerate(rows):
        keys = [(os.path.normpath(row.path), index if BROKEN == "parallel-same-file-split" else None)]
        if BROKEN != "parallel-gpu-rows-split" and row_needs_card(row):
            keys.append("要卡")
        for key in keys:
            leaders[leader_of(index)] = leader_of(group_holder.setdefault(key, index))
    groups = {}
    for index in range(len(rows)):
        groups.setdefault(leader_of(index), []).append(index)
    lanes = [[] for _lane in range(min(len(groups), ceiling))]
    for members in sorted(groups.values(), key=lambda members: (-len(members), members[0])):
        min(lanes, key=len).extend(members)
    return [[rows[index] for index in sorted(lane)] for lane in sorted(lanes, key=min)]


def prepared_lanes(copy, lane_rows, environment):
    """→ ([Lane…], 各路目录的根或 None, [要打的行])。只有一路就在 --copy 那份副本上跑，不建目录；两路以上建好各路的目录与环境，工作副本由这一路在基线那一段自己拷。
    弄坏开关 parallel-shared-target、parallel-jobs-undivided。建不了目录抛 OSError。"""
    if len(lane_rows) <= 1:
        return [Lane(1, rows, copy, environment, "") for rows in lane_rows], None, []
    base = os.path.join(copy, os.environ["CARGO_TARGET_DIR"]) if os.environ.get("CARGO_TARGET_DIR") else tempfile.gettempdir()
    os.makedirs(base, exist_ok=True)
    lanes_root = os.path.realpath(tempfile.mkdtemp(prefix=LANES_ROOT_PREFIX, dir=base))
    total = build_jobs_total(environment)
    jobs = total if BROKEN == "parallel-jobs-undivided" else max(1, total // len(lane_rows))
    lanes = []
    for number, rows in enumerate(lane_rows, 1):
        lane_environment = dict(environment)
        lane_environment["CARGO_TARGET_DIR"] = os.path.join(lanes_root, "target" if BROKEN == "parallel-shared-target" else f"lane-{number:02d}/target")
        lane_environment["CARGO_BUILD_JOBS"] = str(jobs)
        lanes.append(Lane(number, rows, os.path.join(lanes_root, f"lane-{number:02d}", "tree"), lane_environment, f"lane-{number:02d}-"))
    return lanes, lanes_root, [f"- 并行：开 {len(lanes)} 路（撞文件的行与要卡的行留在同一路），每路编译并行度 {jobs}；各路的工作副本与编译目录在 {lanes_root}"]


def run_in_lanes(lanes, work):
    """每路一条线程跑 work(lane)，全部收齐才回：→ ([每路的结果，按派活的次序；抛了异常的是 None], [每路抛的异常，一路一行])。
    只有一路就在本线程跑，异常照常往外抛。弄坏开关 parallel-one-at-a-time。"""
    if len(lanes) <= 1:
        return [work(lane) for lane in lanes], []
    results, problems = [None] * len(lanes), [None] * len(lanes)

    def run_lane(position, lane):
        try:
            results[position] = work(lane)
        except Exception as error:   # 收进这一路的结局，由收束那一步点名、整次判红；这里不往外抛，线程里抛了没人接
            problems[position] = f"✗ 第 {lane.number} 路抛了 {type(error).__name__}：{error}"

    threads = [threading.Thread(target=run_lane, args=(position, lane)) for position, lane in enumerate(lanes)]
    for thread in threads:
        thread.start()
        if BROKEN == "parallel-one-at-a-time":
            thread.join()
    for thread in threads:
        thread.join()
    return results, [problem for problem in problems if problem]


def lanes_collected_problem(lanes, results):
    """派出几路要收回几路（文件头「判法与次序」的「并行」一条）：对不上交要打的那一行，对得上交 None。弄坏开关 parallel-count-unchecked。"""
    collected = sum(1 for result in results if result is not None)
    if BROKEN == "parallel-count-unchecked" or collected == len(lanes):
        return None
    return f"✗ 派出 {len(lanes)} 路、只收回 {collected} 路，没收回的那几路的行没判定：整次不算数"


def remove_lanes_root(lanes_root):
    """收尾删各路的目录：→ (删成了没有, [要打的行])。删成了不打字（输出与串行一样多）。弄坏开关 parallel-lanes-kept。"""
    if lanes_root is None or BROKEN == "parallel-lanes-kept":
        return True, []
    removed, lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(lanes_root)
    return removed, [] if removed else lines


def prove(copy, crate, names, cargo="cargo", target_directory=None, main_root=None, closure_root=None, registry_root=None):
    """→ (退出码, [输出行])。main_root 是判「--copy 指的是不是主工作区」时的主工作区，不给就是这份脚本所在的仓根 MAIN_ROOT；
    自证给一棵临时的假主工作区，那一格判得出来不靠自证从哪棵树起（从符号链接拼的草稿树起时 MAIN_ROOT 里没有 crates/mutations.tsv）。
    closure_root 是核副本缺不缺时闭包照哪棵树算：不给时取自证放进 CLOSURE_ROOT_FOR_SELFTEST 的那一棵，没有就是 main_root。
    registry_root 是重测试目标的登记表读哪一份仓根的（文件头「判法与次序」的「重测试目标」一条）：不给就是 MAIN_ROOT，自证那一格给自己造的一份。"""
    copy = os.path.realpath(copy)
    main_root = MAIN_ROOT if main_root is None else os.path.realpath(main_root)
    environment = cargo_environment(copy, target_directory)
    if not os.path.isfile(os.path.join(copy, "crates", "mutations.tsv")):
        return 2, [f"✗ {copy} 里没有 crates/mutations.tsv，不像仓副本", f"→ 怎么办：{COPY_HOWTO}"]
    if copy == main_root and BROKEN != "main-allowed":
        return 2, [f"✗ --copy 指的是主工作区 {copy}", f"→ 怎么办：在草稿目录里拷一份仓副本：{COPY_HOWTO}；证红只在副本上改坏源码"]
    refused = copy_closure_refusal(copy, closure_root or (CLOSURE_ROOT_FOR_SELFTEST[0] if CLOSURE_ROOT_FOR_SELFTEST else main_root))
    if refused is not None:
        return refused
    lock = None
    if BROKEN != "no-copy-lock" and copy != main_root:   # 主工作区只在弄坏开关 main-allowed 下走到这里，不在它根上建锁文件
        try:
            lock = COPY_LOCK.hold_copy_lock(copy)
        except OSError as error:
            return 2, [f"✗ 在副本 {copy} 上建不了副本锁 {COPY_LOCK.COPY_LOCK_NAME}：{error}", "→ 怎么办：看副本根可不可写；一条都没跑"]
        if lock is None:
            return 2, [f"✗ 副本 {copy} 上已有一趟证红在跑（{COPY_LOCK.COPY_LOCK_NAME} 被进程 {COPY_LOCK.copy_lock_holder(copy)} 占着），这一次一条都没跑",
                       "→ 怎么办：副本上有证红在跑，等它跑完或换一份副本"]
    try:
        # 编译目录归这一趟（文件头「判法与次序」的「编译目录归这一趟」）：调用方给了 CARGO_TARGET_DIR 的照它的、不删；没给的这一趟自己建一份新的，成功、红、被打断都在这里删
        trip_directory = None
        if not environment.get("CARGO_TARGET_DIR"):
            try:
                trip_directory = new_trip_build_directory()
            except OSError as error:
                return 2, [f"✗ 建不了这一趟自己的编译目录（{tempfile.gettempdir()} 下）：{error}", "→ 怎么办：看临时目录可不可写、盘满没满；一条都没跑"]
            environment["CARGO_TARGET_DIR"] = trip_directory
        try:
            code, lines = prove_holding_the_lock(copy, crate, names, cargo, environment, registry_root)
        finally:
            removed, removal_lines = remove_trip_build_directory(trip_directory)
        return (code, lines) if removed else (code or 1, lines + removal_lines)
    finally:
        if lock is not None:
            COPY_LOCK.release_copy_lock(copy, lock)


def prove_holding_the_lock(copy, crate, names, cargo, environment, registry_root=None):
    """拿着副本锁（文件头「判法与次序」⓪）先照还原日志写回上一趟没还原的变异，再往下做 ②–⑤：→ (退出码, [输出行])，写回的那一行打在最前面。"""
    refused, restored = restore_unrestored_mutation(copy)
    if refused is not None:
        return refused, restored
    code, lines = prove_on_restored_sources(copy, crate, names, cargo, environment, registry_root)
    return code, restored + lines


def prove_on_restored_sources(copy, crate, names, cargo, environment, registry_root=None):
    """②–⑤（副本源码里没有上一趟没还原的变异之后）：→ (退出码, [输出行])。registry_root 见 prove。"""
    rows, malformed = table_rows(copy)
    refusals, chosen = [], []
    for name in names:
        row = rows.get(name)
        if row is None:
            refusals.append(f"{name}：crates/mutations.tsv 里这一行不成形，{malformed[name]}" if name in malformed
                            else f"{name}：crates/mutations.tsv 里没有这一行")
            continue
        why = refusal_for(row, crate) or source_path_problem(copy, row.path)
        if why:
            refusals.append(f"{name}：{why}")
        else:
            chosen.append(row)
    if refusals:
        return 2, [f"✗ {line}" for line in refusals] + ["→ 怎么办：照 crates/mutations.tsv 表头把那一行写成六段（替换文之外不许空），cargo test 参数加上 -p <crate> 与 --lib / --test <目标> / --bin <名> 之一再证；一条都没跑"]
    chosen, default_features_notes = default_features_off_rows(chosen, environment)
    try:
        chosen, release_notes = release_forced_rows(chosen, registry_root or MAIN_ROOT)
        release_notes = default_features_notes + release_notes
    except RELEASE_ONLY_TESTS.RegistryUnreadable as error:
        return 2, [f"✗ 判不了点名的行里哪几行是重测试目标：{error}", f"→ 怎么办：{RELEASE_ONLY_TESTS.REGISTRY_HOWTO}；一条都没跑"]
    try:
        ceiling = lane_ceiling(environment)
    except ValueError as error:
        return 2, [f"✗ 算不出同时开几路：{error}", f"→ 怎么办：{LANES_VARIABLE} 写正整数或不设（不设时取 CARGO_BUILD_JOBS）；CARGO_BUILD_JOBS 写正整数或不设（不设时取登记的重活默认并行度）；一条都没跑"]
    misses = old_text_misses(copy, chosen)
    if misses:
        return 2, misses + ["→ 怎么办：照上面每一行的出路办，基线与变异一条都没跑"]
    # 判定库（文件头「判定库与复用」）：算行键、读库、抽复用哨兵；键相同、库里记着抓到的复用（不起 cargo），别的照跑
    try:
        key_context, chosen, components, library, decided, route_notes = library_plan(copy, chosen, registry_root or MAIN_ROOT, environment)
    except LibraryPlanRefused as error:
        return 2, list(release_notes) + error.lines
    sentinel, sentinel_lines = planned_sentinel(library, chosen, components, environment)
    decisions, reused = {}, []
    for row in chosen:
        decision = decided[row.line_number][0]
        if decision.reuse and sentinel.forced(ROWS.sentinel_item(row)):
            decision = ROWS.Decision(False, "sentinel", "复用哨兵抽中：键相同，这一趟不读库")
        decisions[row.line_number] = decision
        if decision.reuse:
            reused.append(row)
    to_judge = [row for row in chosen if not decisions[row.line_number].reuse]
    refused_count = sum(1 for row in chosen if components[row.line_number].row_key is None)
    counts_text = f"现判 {len(to_judge) - refused_count} 条、复用 {len(reused)} 条、拒算 {refused_count} 条"
    # 缩过域的一趟（文件头「缩过域的一趟」）：判法与门禁同一个函数 crash_domain_narrowing；现判的每一行末尾的标之前接一句缩过域，不写判定库与索引
    narrowing = ROWS.crash_domain_narrowing(environment)
    narrowed_unrecorded = bool(narrowing) and BROKEN != "narrowed-recorded"
    narrowed_note = f"；缩过域（{narrowing}）、不算门禁通过、没入库" if narrowing else ""
    # 组成清单先进库（点名测试的闭包）：这一趟中途停了，门禁下一趟照样报得出变了哪几个单元
    key_context.persist(library, components)
    output = list(release_notes) + route_notes + sentinel_lines
    for row in chosen:
        if components[row.line_number].row_key is None:
            output.append(f"- 拒算 {row.name}：行键算不出（照跑、不复用、不写库）：{components[row.line_number].refusal}")
    for row in reused:
        output.append(f"{row.name}\t抓到\t{row.expected} 红了（{decisions[row.line_number].detail}）（复用）")
    # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：有要跑的行就在起任何 cargo 之前判一次（判的是副本那棵树，起的是 cargo 那一份），
    #   判红整次拒绝、退 2、一条都不跑。树里没有依赖 rocksdb 的清单（自证的副本）、cargo 是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判
    if to_judge and BROKEN != "rocksdb-entry-unchecked":
        entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT, "entry", "--root", copy, "--cargo", cargo], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if entry.returncode != 0:
            return 2, [line for line in (entry.stdout + entry.stderr).splitlines() if line.strip()] + [
                "✗ 起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条都没跑",
                '→ 怎么办：照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再证这一批']
    logs = os.environ.get("PROVE_RED_LOG_DIRECTORY") or os.path.join(copy, "prove-red-logs")
    if BROKEN != "shared-log-directory":   # 每一趟写自己的日志目录：同一份副本上别的机器上的证红不会把基线日志换成它的（文件头「判法与次序」的日志一条）
        logs = os.path.join(logs, f"run-{os.getpid()}-{secrets.token_hex(4)}")
    if to_judge:
        os.makedirs(logs, exist_ok=True)

    mutated_service = {"root": None, "binary": None}   # 变异那几趟各起临时 prune-server 用的服务根与二进制（prepare_mutated_run_service 在派路之前填，收尾删）

    def baseline_red(rows, code, problem_of):
        """基线红的那几行照行键记 baseline-not-ok（文件头「判定库与复用」记红撤绿）：→ 写不进去时要打的那几行。"""
        problems = []
        for row in rows:
            text = f"crates/mutations.tsv:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem_of(row)}"
            problem = record_verdict(library, row, components[row.line_number], "baseline-not-ok", text, code, 0.0, "", environment)
            if problem:
                problems.append(problem)
        return problems

    def lane_baselines(lane):
        """这一路（两路以上时先拷工作副本）跑自己那几组参数的基线：→ {参数: (退出码, 输出, 日志)}；一组红了或没证实点名的测试就停，这一路后面几组不跑（收齐之后的判法在那一组上就退了）。"""
        if lane.copy != copy:
            shutil.copytree(copy, lane.copy, ignore=LANE_COPY_IGNORED, copy_function=shutil.copy, symlinks=True)
        runs = {}
        for position, arguments in enumerate(sorted({row.arguments for row in lane.rows})):
            group_suffix = f"-{position + 1}" if position else ""
            log_path = os.path.join(logs, f"{lane.log_prefix}baseline{group_suffix}.log")
            baseline_arguments = ROWS.with_shown_output(arguments.split())
            code, text = run_cargo(lane.copy, baseline_arguments, log_path, cargo, baseline_environment(lane.environment))
            runs[arguments] = (code, text, log_path)
            if BROKEN != "baseline-ignored" and (code != 0 or any(ROWS.baseline_problem(row, code, text) or ROWS.not_run_reason(row.expected, text)
                                                                  for row in lane.rows if row.arguments == arguments)):
                break
        return runs

    def judge_row(lane, row, index, library_lock):
        """在这一路的工作副本上施加、跑、还原、判一行（文件头「判法与次序」⑤）：→ RowResult。写判定库拿着 library_lock：各路共用一个判定库客户端。"""
        path = os.path.join(lane.copy, row.path)
        log_path = os.path.join(logs, f"{index:03d}.log")
        with open(path, "rb") as handle:
            original_bytes = handle.read()
        hits = normalized_text(original_bytes).count(row.old)
        if hits != 1:
            return RowResult([f"✗ {row.name}：原文在 {row.path} 里命中 {hits} 次，要恰好 1 次"], True, 0, 0, None, False)
        try:
            write_restore_journal(lane.copy, row, original_bytes)
        except OSError as error:
            return RowResult([f"✗ {row.name}：施加之前落不了还原日志 {os.path.join(lane.copy, RESTORE_JOURNAL_NAME)}（{error}），没施加——看副本根可不可写"], True, 0, 0, None, False)
        state = state_text(decisions[row.line_number])
        started = time.monotonic()
        try:
            with open(path, "wb") as handle:
                handle.write(mutated_bytes_of(original_bytes, row.old, row.new))
            # 改坏之后这一次 cargo 对它自己的临时 prune-server 跑（文件头「判定库与复用」的变异一条）：判出的崩溃结论不进调用方的库；基线（lane_baselines）不走这里
            with mutated_run_prune_server(mutated_service, row.arguments.split()) as service_settings:
                code, text = run_cargo(lane.copy, row.arguments.split(), log_path, cargo, dict(lane.environment, **service_settings))
        finally:
            with open(path, "wb") as handle:
                handle.write(original_bytes)
            os.utime(path)
            drop_restore_journal(lane.copy)
        seconds = time.monotonic() - started
        rebuilt_problem = rebuild_problem(lane.copy, row, code, text, log_path, lane.environment.get("CARGO_TARGET_DIR"))
        if rebuilt_problem:   # 判定不算数：不写库、不交哨兵（文件头「判定库与复用」）
            return RowResult([f"{rebuilt_problem}（{state}）"], True, 0, 0, None, False)
        outcome, line = judged_line(row, code, text, log_path)
        # 写进判定库的档：抓到、本次未跑照 judged_line 的结局，别的照 verdict_of_run 的档（没红、无效；弄坏开关改出来的「没红」记没红）
        verdict, sentence = verdict_of(row, code, text)
        verdict = {"caught": "caught", "not-run": "notrun"}.get(outcome) or (verdict if verdict in ("failure", "invalid", "linkfailed") else "failure")
        with library_lock:
            record_problem = record_verdict(library, row, components[row.line_number], verdict, sentence or line, code, seconds,
                                            ROWS.single_full_name(row.expected, text) or "", environment)
        lines = [f"{line}{narrowed_note}（{state}）"] + ([record_problem] if record_problem else [])
        return RowResult(lines, outcome == "failed", int(outcome == "caught"), int(outcome == "not-run"), (row.line_number, verdict, sentence, None), True)

    def judged_on_lanes(lanes):
        """④⑤ 按路跑（文件头「判法与次序」的「并行」一条）：先各路的基线、收齐照串行的次序判，再各路施加变异、收齐按点名次序打：→ (退出码, [输出行])。"""
        baseline_results, baseline_exceptions = run_in_lanes(lanes, lane_baselines)
        collected_problem = lanes_collected_problem(lanes, baseline_results)
        if baseline_exceptions or collected_problem:
            return 2, output + baseline_exceptions + ([collected_problem] if collected_problem else []) + [
                f"→ 怎么办：基线那一段有一路没走完，一条变异都没施加；看 {logs} 下那一路的基线日志与各路目录所在的盘还有多少，修好再证这一批；要退回串行就设 {LANES_VARIABLE}=1"]
        lane_of = {row.line_number: lane.number for lane in lanes for row in lane.rows}
        run_of = {(lane.number, arguments): run for lane, runs in zip(lanes, baseline_results) for arguments, run in runs.items()}
        for arguments in sorted({row.arguments for row in to_judge}):
            group = [row for row in to_judge if row.arguments == arguments]
            red_rows = [row for row in group if run_of[lane_of[row.line_number], arguments][0] != 0]
            if red_rows and BROKEN != "baseline-ignored":
                written = []
                for row in red_rows:
                    code, text, _log_path = run_of[lane_of[row.line_number], arguments]
                    written += baseline_red([row], code, lambda row: ROWS.baseline_problem(row, code, text) or f"退 {code}（cargo test {arguments}）")
                red_log_path = run_of[lane_of[red_rows[0].line_number], arguments][2]
                return 2, output + written + [f"✗ 基线（没改源码）就红：cargo test {arguments}（日志 {red_log_path}）；{counts_text}",
                                              "→ 怎么办：先让基线绿；基线里本来就红的测试不能拿来证红"]
            if BROKEN != "baseline-ignored":
                for row in group:
                    code, text, log_path = run_of[lane_of[row.line_number], arguments]
                    problem = ROWS.baseline_problem(row, code, text)
                    written = baseline_red([row], code, lambda _row: problem) if problem else []
                    problem = problem or ROWS.not_run_reason(row.expected, text)
                    if problem:
                        return 2, output + written + [f"✗ 基线没有证实 {row.name} 的点名测试通过：{problem}（日志 {log_path}）；{counts_text}",
                                                      "→ 怎么办：确认基线实际执行了点名测试并通过，再证红"]
        positions = {row.line_number: position for position, row in enumerate(to_judge, 1)}
        library_lock = threading.Lock()
        finish_order = []

        def lane_rows(lane):
            results = {}
            for row in lane.rows:
                results[row.line_number] = judge_row(lane, row, positions[row.line_number], library_lock)
                finish_order.append(row.line_number)
            return results

        lane_results, lane_exceptions = run_in_lanes(lanes, lane_rows)
        if DROP_LAST_LANE_FOR_SELFTEST and len(lanes) > 1:
            lane_results[-1] = None
        collected_problem = lanes_collected_problem(lanes, lane_results)
        results_of = {line_number: result for results in lane_results if results for line_number, result in results.items()}
        lines = output + lane_exceptions + ([collected_problem] if collected_problem else [])
        failed, caught, not_run = bool(lane_exceptions or collected_problem), len(reused), 0
        fresh_outcomes, reported = [], list(reused)   # 交复用哨兵的现判结局（行号, 档, 那一句, 删临时目录）；写索引的行
        # 收齐才按点名次序回读；弄坏开关 parallel-output-finish-order 下按跑完的先后，parallel-lane-red-swallowed 下头一路之外的红不算
        in_finish_order = sorted(to_judge, key=lambda row: finish_order.index(row.line_number) if row.line_number in finish_order else len(finish_order))
        for row in in_finish_order if BROKEN == "parallel-output-finish-order" else to_judge:
            result = results_of.get(row.line_number)
            if result is None:
                continue
            lines += result.lines
            swallowed = BROKEN == "parallel-lane-red-swallowed" and len(lanes) > 1 and lane_of[row.line_number] != lanes[0].number
            failed = failed or (result.failed and not swallowed)
            caught += result.caught
            not_run += result.not_run
            if result.fresh is not None:
                fresh_outcomes.append(result.fresh)
            if result.reported:
                reported.append(row)
        return finished_summary(lines, failed, caught, not_run, fresh_outcomes, reported)

    def finished_summary(lines, failed, caught, not_run, fresh_outcomes, reported):
        """记索引、交复用哨兵、打汇总句：→ (退出码, [输出行])。缩过域的一趟索引也不写，汇总句写缩过域（文件头「缩过域的一趟」）。"""
        lines = lines + ([] if narrowed_unrecorded else record_index(library, reported, components))
        # 复用哨兵（文件头「判定库与复用」）：复用的交「复用」，现判出结局的照门禁的 observe_fresh_outcomes 交
        for row in reused:
            sentinel.observe(ROWS.sentinel_item(row), ROWS.reuse_checks.REUSED, "抓到")
        ROWS.observe_fresh_outcomes(sentinel, fresh_outcomes, {row.line_number: row for row in chosen}, environment)
        sentinel_problems = sentinel.finish(printer=lines.append)
        for message, howto in sentinel_problems:
            lines += [f"✗ {message}", f"→ 怎么办：{howto}"]
        summary = (f"点名 {len(names)} 条：{counts_text}；跑了 {len(to_judge)} 条，抓到 {caught} 条（其中复用 {len(reused)} 条），本次未跑 {not_run} 条"
                   + (f"；缩过域（{narrowing}）、不算门禁通过、现判的没入库（复用的是库里全域下判的）" if narrowing else ""))
        if failed:
            return 1, lines + [f"✗ {summary}，有没红、无效、链接失败或没重编的，或有一路没收回（逐条列在上面）",
                               "→ 怎么办：没红的换一条真会改行为的变异或补断言；无效的改替换文；照 .claude/rules/mutation-sampling.md 分类；"
                               "链接失败的删掉调用方给的编译目录重跑那一行（没给的这一趟自己建的编译目录跑完已删，再重跑一次同一条命令）；没重编的按那一行刷修改时刻或换编译目录重跑；"
                               f"有一路没收回的照那一行重跑这一批（判完的已在判定库里），要退回串行就设 {LANES_VARIABLE}=1"]
        if sentinel_problems:
            return 1, lines + [f"✗ {summary}，复用哨兵判红（逐条列在上面）",
                               "→ 怎么办：照上面复用哨兵那几行的出路办"]
        if caught == 0:
            return EXIT_NOT_RUN, lines + [f"- {summary}：一条都没抓到，点名的都是本次未跑（逐条列在上面）",
                                          "→ 怎么办：打了 NOT_RUN、GPU_DEVICE_LOST 的是测试自己要开卡，任务不拿卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）；"
                                          "交 GPU 服务代跑的接口还没定，这几条照记本次未跑，不算证过红"]
        return 0, lines + [f"✓ {summary}，跑了的都抓到了"]

    try:
        if to_judge:   # 都复用的一趟不起服务；有要现判的行，在起任何一次测试的 cargo 之前取服务二进制
            service, service_problem = prepare_mutated_run_service()
            mutated_service.update(service)
            if service_problem:
                return 2, output + [f"✗ 变异那几趟要各起一份临时 prune-server，服务的二进制取不到（{service_problem}），测试的 cargo 一次都没起",
                                    f"→ 怎么办：设 PRUNE_SERVER_BINARY=<编好的 prune-server 的绝对路径> 再跑；或修好 prune-server/ 的编译"
                                    "（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server，经 task-server/task submit 交）"]
        try:
            lanes, lanes_root, lane_plan = prepared_lanes(copy, planned_lanes(to_judge, ceiling) if to_judge else [], environment)
        except OSError as error:
            return 2, output + [f"✗ 建不了各路的目录：{error}",
                                f"→ 怎么办：看调用方设的 CARGO_TARGET_DIR 或临时目录可不可写、盘满没满；要退回串行就设 {LANES_VARIABLE}=1；一条都没跑"]
        output += lane_plan
        try:
            code, lines = judged_on_lanes(lanes)
        except BaseException:
            remove_lanes_root(lanes_root)
            raise
        removed, removal_lines = remove_lanes_root(lanes_root)
        return (code, lines) if removed else (code or 1, lines + removal_lines)
    finally:
        remove_mutated_run_service_root(mutated_service)


def write_text(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


def prove_from_arguments(arguments, cargo="cargo"):
    """→ (退出码, [输出行])：先解析 --copy，再证红；编译目录的默认值由 prove 照解析出的副本定（上限由 task-server 按记录给，不收 --memory 这类参数）。"""
    if len(arguments) < 4 or arguments[0] != "--copy":
        return 2, ["✗ 用法不对", "→ 怎么办：prove-red.sh --copy <仓副本> <crate> <变异名…>"]
    return prove(arguments[1], arguments[2], arguments[3:], cargo=cargo)
