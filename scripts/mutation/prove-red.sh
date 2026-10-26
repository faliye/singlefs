#!/usr/bin/env bash
# admission: always 实现员每写一条新测试都要现证它会红，判的是这一刻副本里的源码与变异行
# run-condition: command python3 cargo
# 实现员的证红：在仓副本里逐条施加 crates/mutations.tsv 里点名的变异，经内存包装跑那一行指定的测试目标，看点名的测试红没红，再还原。
#
#   prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>
#   prove-red.sh --selftest            # 用假的 cargo 走一遍；PROVE_RED_BREAK=<项> 时必须判红
#
# 判法只有一份，与门禁 checker-tier-crates-mutation-replay 同用（这里不另写）：
#   切表与成形照 scripts/mutation/crates-mutation-rows.py 的 read_table（malformed_reason）；一条跑完判哪一档照它的 verdict_of_run；
#   libtest 的结果行在那里经 scripts/gate/lib_libtest_result_lines.py 解析。一行算不算重型照 .claude/hooks/lib_heavy_tests.py 的 classify（重型测试闸同一份，只读调用）。
#
# 弄坏开关 PROVE_RED_BREAK（只给 --selftest 证明它会红用）。一项一行，下一行起写打开后判错的自证项：
#   any-target  放过不挑目标的参数
#       「不挑目标的变异行应当整次拒绝、退 2」
#   main-allowed  放过主工作区
#       「--copy 指主工作区应当以「主工作区」为由拒绝」（那一格的主工作区是临时目录里一棵假的、经 prove 的 main_root 顶替，
#       自证从哪棵树起都判得出；打开开关时假 cargo 在那棵假树上照跑，不再以主工作区为由退 2）
#   heavy-run  没设 SINGLEFS_HEAVY_TESTS（子 agent 起的）时照样跑重型的行
#       「子 agent 起（没设 SINGLEFS_HEAVY_TESTS）：checker 档包的行跳过、逐行报本次未跑、cargo 一次都不起、整次退 77」
#   crash-point-replay-by-name  回到按 --test 目标名里带不带 crash_point_replay 跳过（改前的口径，与重型测试闸的按包判不一致）
#       「--test 目标名带 crash_point_replay 而包不是 checker 档：照跑、判抓到」「子 agent 起（没设 SINGLEFS_HEAVY_TESTS）：checker 档包的行跳过……」
#   own-table  回到自己切表（六段就收、不看哪一段空着），不经 read_table
#       「点名的行不成形（必须红的测试名空着）：整次拒绝、写明哪一段空着」「切表与 read_table 逐行相同」
#   own-judge  回到自己判结局（任一行点名的 FAILED 就算抓到，不看退出码、不看对上几个用例、不认 NOT_RUN），不经 verdict_of_run
#       「判法与 verdict_of_run 对同一组样本逐行相同」
#   baseline-ignored  不看基线红没红
#       「基线（没改源码）就红：停下、退 2、一条变异都不记抓到」
#   never-caught  一条都不记抓到
#       「会红的变异应当抓到、退 0」等要抓到的七格（编译目录那两格与副本锁那一格也要抓到才退 0）
#   error-line-invalid  回到「有一行 error: 就算编不过」（cargo 测试红了也打 error: test failed，点名的没红会被判成无效）
#       「编过了、别的测试红而点名的没红……应当判没红、不判无效」
#   gpu-memory-as-caught  不认内存包装退 245（显存不够），照输出里的 FAILED 判抓到
#       「内存包装退 245（显存不够）的那一条应当报显存不够、不算抓到、退 1」（实际判成抓到、退 0）、「判法与 verdict_of_run 对同一组样本逐行相同」
#   no-copy-lock  不在副本根上拿副本锁（见「判法与次序」⓪），锁被占着也照跑
#       「证红跑的时候副本根上拿着副本锁、跑完放掉；锁被占着时拒绝、退 2、cargo 一次都不起」
#   no-rebuild-check  施加变异之后那一趟编译不核被改的包有没有 Compiling 行（见「判法与次序」⑤）
#       「编译目录里的产物比施加变异后的源码新（共用编译目录）：那一条判没重编、退 1，不判没红」（实际判成没红）；
#       scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 不让这一格红（那一趟一行 Compiling 都没有）
#   rocksdb-entry-unchecked  起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       「副本里的 checker 档包依赖 rocksdb 而进程环境里没有预编库的取值：应当在起 cargo 之前整次拒绝、退 2、假 cargo 一次都不起」
#       （entry 判的是 --copy 那棵树、--cargo 是要起的那一份 cargo；树里没有依赖 rocksdb 的清单、cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo 时不判）
#   should panic 那一段的解析在 lib_libtest_result_lines.py：LIBTEST_RESULT_LINES_BREAK=should-panic-in-name 时
#       「#[should_panic] 的用例红了……应当判抓到、退 0」判错
# 编译目录那三格的开关（开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=unbound-second-argument  开头照改前的写法拿 $2 拼 CARGO_TARGET_DIR 的默认值（set -u 下只给 --selftest 一个参数时当场退 1）；
#     只在自证起的那一层子进程（PROVE_RED_SELFTEST_PROLOGUE 设着）里生效，外层照常跑完才报得出是哪一格
#       「✗ 自检：不设 CARGO_TARGET_DIR、只给 --selftest：开头那几行」（子进程退 1，输出 $2: unbound variable）
#       连带红的格：「✗ 自检：不设 CARGO_BUILD_JOBS：开头那几行」、「✗ 自检：调用方设了 CARGO_BUILD_JOBS：开头那几行」
#   PROVE_RED_BREAK=second-argument-target  CARGO_TARGET_DIR 的默认值照改前 $2 的取法拿第二个参数拼
#       「✗ 自检：--copy 不在第一个（--memory 1G --copy <副本>）：编译目录」（取成 1G/target）
#   PROVE_RED_BREAK=caller-target-overridden  调用方设了的 CARGO_TARGET_DIR 也换成 <副本>/target
#       「✗ 自检：调用方设了 CARGO_TARGET_DIR」；连带「编译目录里的产物比施加变异后的源码新（共用编译目录）……」
#       （那一格的共用编译目录就是调用方设的 CARGO_TARGET_DIR，被换成副本自己的 target 之后照常重编、判抓到、退 0）
#   PROVE_RED_BREAK=build-jobs-six  没设 CARGO_BUILD_JOBS 时写死 6（改前的样子），不取登记的重活默认并行度；同样只在自证起的那层子进程里生效
#       「✗ 自检：不设 CARGO_BUILD_JOBS：开头那几行」（子进程打 CARGO_BUILD_JOBS=6）
#
# 判法与次序：
#   ① --copy 必须是一份副本：目录在、里面有 crates/mutations.tsv、不是这份脚本所在的主工作区（在主工作区上改坏源码会波及别的会话）；
#   ⓪ 过了 ① 就在副本根上拿副本锁 <副本>/.prove-red.lock（排他 flock、不等），一直拿到这一次返回；拿不到（别的证红在跑）整次拒绝、退 2、一条都不跑。
#      scripts/gate/apply-writer-patch.py --root <副本> 开头试拿同一把锁，拿不到就不打补丁（主 agent 2026-10-10 往正在证红的副本打过补丁）；
#      锁的写法只在 apply-writer-patch.py 的 hold_copy_lock / release_copy_lock（这里按路径载入它）。正常收尾删掉锁文件；被打断时内核随进程放锁、文件留着不碍下一次拿；
#   ② 逐个找变异名对应的行（read_table 读成形的行；同名的取表里最后一行）；找不到、那一行不成形、参数里没有 -p <crate>、
#      没有 --lib / --test / --bin 之一的，整次拒绝（一条都不跑，退 2）——不挑目标的 cargo test 会把这个包的全部测试目标都编、都跑；
#   ③ 这一行的 cargo test <参数> 照 lib_heavy_tests.classify 判（当前目录是副本的根）：重型的（checker 档包 singlefs-checker-tier 的每一行都是）
#      只在 SINGLEFS_HEAVY_TESTS 是 commit 或 user-request（主 agent 带前缀起的）时照跑；别的时候（子 agent 起的）不跑、逐行报「本次未跑」与判成什么；
#   ④ 每组不同的参数先跑一次基线（不改源码），基线红就停（退 2）；
#   ⑤ 每条变异：原文在文件里恰好命中一次才改，经 scripts/hosts/run-with-memory-cap.sh <上限> 跑 cargo test <参数>，
#      跑完照 verdict_of_run 判：抓到（退出码非 0、点名的测试恰好对上一个用例且有 FAILED；#[should_panic] 的结果行多一段「 - should panic」同样认）、
#      无效（点名的一行都没出现而有 error 行或 could not compile）、没红（其余；点名省掉模块路径而对上两个用例也算）、
#      本次未跑（点名的测试 ok 又打了 NOT_RUN_NO_GPU_ADAPTER）；跑完把原文件写回并 touch。
#      判之前先核这一趟真重编了被改的那个包（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行）：
#      变异行改的文件往上找到的包（scripts/gate/lib_rebuilt_packages.py 的 package_of）在这一趟的输出里要有「Compiling <包名> v…」行；
#      没有（编译目录里的产物比改过的源码新，cargo 认它已编、测试跑在旧产物上）记「✗ <名>\t没重编」、整次退 1，
#      出路是刷源码修改时刻或换干净编译目录（不设 CARGO_TARGET_DIR，用副本自己的 target）重跑。内存包装退 245、250–254 的那一条不核（结果本来就不算数）；
#      改的不是包里的 .rs、或编完它不在编译目录的任何 dep-info 里（这一趟编译不读它，例如没开的 feature 下才编的模块）的不核（判法 packages_to_check）。
#      内存包装退 245（显存不够：命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY，run-with-memory-cap.sh 文件头「退出码」）的那一条：
#      输出里的 FAILED 是被显存不够弄红的，不算抓到，记「✗ <名>：显存不够」、整次退 1；退 250–254 的同样不算，记内存包装的结局。
#      基线那一次退 245 或 250–254：停下，原样退那个码。
# 每条一行判定打到 stdout（名、结局、日志路径）；日志放 ${PROVE_RED_LOG_DIRECTORY:-<副本>/prove-red-logs}。
# 退出码：0 跑了的都抓到、至少抓到一条（本次未跑的照列，不算失败）；1 有没红、无效或显存不够、撞了内存包装的结局；2 用法错、被拒或基线红；
#   77 一条都没抓到、点名的都是本次未跑（重型的行交主 agent，或点名的测试打了 NOT_RUN）；245、250–254 基线那一次原样转内存包装的结局。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 副本路径与主仓不同、或 bindgen 的环境变了，librocksdb-sys 的 C++ 会在基线那一次重编；cc 按 cargo 的并行度开编译器，
# 调用方另给 CARGO_BUILD_JOBS 就照它的，没给取登记的重活默认并行度（.claude/heavy-thread-default，scripts/hosts/heavy-threads.sh）。
# 32 路一起编 RocksDB 撞过 12G 与 16G 的上限（退 250，内存包装报出来，不是悄悄编坏）：撞了就压小 CARGO_BUILD_JOBS 或加大 --memory 再来。
source "$SCRIPT_DIRECTORY/../hosts/heavy-threads.sh"
if [[ -z "${CARGO_BUILD_JOBS:-}" ]]; then
  if [[ "${PROVE_RED_BREAK:-}" == build-jobs-six && -n "${PROVE_RED_SELFTEST_PROLOGUE:-}" ]]; then
    CARGO_BUILD_JOBS=6   # 弄坏开关（文件头）：改前的写法
  else
    CARGO_BUILD_JOBS="$(heavy_thread_default)"
  fi
fi
export CARGO_BUILD_JOBS
# CARGO_TARGET_DIR 的默认值（副本自己的 target）在下面的 python 里解析完 --copy 之后才定、只设给 cargo 那几次（cargo_environment）：
# 这里不从位置参数取——只给 --selftest 时没有 $2（set -u 下当场退 1），--copy 不在第一个时 $2 也不是副本。
if [[ "${PROVE_RED_BREAK:-}" == unbound-second-argument && -n "${PROVE_RED_SELFTEST_PROLOGUE:-}" ]]; then
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$2/target}"   # 弄坏开关（文件头）：改前的写法
fi
python3 /dev/fd/3 "$SCRIPT_DIRECTORY" "$@" 3<<'PY'
import importlib.util, os, re, shlex, shutil, subprocess, sys, tempfile, time

sys.dont_write_bytecode = True   # 下面按路径载入的两份模块不往 __pycache__ 里写
SCRIPT_DIRECTORY = sys.argv[1]
MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
MEMORY_WRAPPER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
ROCKSDB_PREBUILT = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（prove_holding_the_lock）
BROKEN = os.environ.get("PROVE_RED_BREAK", "")
TARGET_OPTIONS = {"--lib", "--test", "--bin"}
GPU_MEMORY_HIT_EXIT = 245   # run-with-memory-cap.sh：显存不够（命令退了非 0、起止之间内核报了 NV_ERR_NO_MEMORY），结果不算数
EXIT_NOT_RUN = 77
HEAVY_TESTS_VARIABLE = "SINGLEFS_HEAVY_TESTS"
HEAVY_TESTS_OCCASIONS = ("commit", "user-request")   # .claude/hooks/heavy-test-guard.sh 认的那两个值：主 agent 提交时、用户要求时
NO_TIME_LIMIT = "不限时"


def load_module(path, module_name):
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


# 判法（切表、成形、判结局）与门禁 checker-tier-crates-mutation-replay 同一份；重型不重型与重型测试闸同一份
ROWS = load_module(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"), "crates_mutation_rows_for_prove_red")
HEAVY_TESTS = load_module(os.path.join(MAIN_ROOT, ".claude", "hooks", "lib_heavy_tests.py"), "lib_heavy_tests_for_prove_red")
# 副本锁（文件头「判法与次序」⓪）与 apply-writer-patch.py 同一把、同一份写法
COPY_LOCK = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "apply-writer-patch.py"), "apply_writer_patch_for_prove_red")
# 被改的包这一趟重编了没有（文件头「判法与次序」⑤）：与 merge-and-verify.sh 同一份判法
REBUILT = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_rebuilt_packages.py"), "lib_rebuilt_packages_for_prove_red")


def default_memory_cap():
    try:
        match = re.search(r'REPLAY_MEMORY_CAP="\$\{REPLAY_MEMORY_CAP:-([0-9]+[KMGT])\}"', open(os.path.join(SCRIPT_DIRECTORY, "..", "experiment-apparatus", "replay.sh"), encoding="utf-8").read())
        return match.group(1) if match else "8G"
    except OSError:
        return "8G"


def own_table_rows(path):
    """弄坏开关 own-table：改前的切法（六段就收，哪一段空着都不管）。"""
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
    rows, malformed, _memory_from_table, _timeout_from_table = ROWS.read_table(path)
    return {row.name: row for row in rows}, {name: reason for _line_number, name, reason in malformed}


def refusal_for(row, crate):
    arguments = row.arguments.split()
    before_separator = arguments[: arguments.index("--")] if "--" in arguments else arguments
    if BROKEN != "any-target" and not TARGET_OPTIONS & set(before_separator):
        return f"参数「{row.arguments}」没挑目标（--lib / --test / --bin 都没有），会把这个包的全部测试目标都编、都跑"
    if "-p" not in before_separator or before_separator[before_separator.index("-p") + 1:before_separator.index("-p") + 2] != [crate]:
        return f"参数「{row.arguments}」不是 -p {crate}"
    return None


def heavy_reason(row, copy):
    """这一行的 cargo test 照 lib_heavy_tests.classify 判（当前目录是副本的根）：重型的交那一句，不是交 None。"""
    if BROKEN == "crash-point-replay-by-name":
        arguments = row.arguments.split()
        named_crash_point_replay = any(argument == "--test" and index + 1 < len(arguments) and "crash_point_replay" in arguments[index + 1] for index, argument in enumerate(arguments))
        return "目标名字带 crash_point_replay" if named_crash_point_replay else None
    found = HEAVY_TESTS.classify(["cargo", "test", *row.arguments.split()], copy)
    return f"{found.category}（{found.detail}）" if found else None


def heavy_tests_allowed():
    return os.environ.get(HEAVY_TESTS_VARIABLE, "") in HEAVY_TESTS_OCCASIONS and BROKEN != "crash-point-replay-by-name"


def own_verdict(row, code, text):
    """弄坏开关 own-judge：改前 prove-red 自己那一份判法（任一行点名的 FAILED 就算抓到，不看退出码、不看对上几个用例、不认 NOT_RUN）。"""
    if code == GPU_MEMORY_HIT_EXIT:
        return "gpumemory", ""
    if 250 <= code <= 254:
        return "memory", ""
    if re.search(r"^test (?:\S*::)?" + re.escape(row.expected) + r"(?: - should panic)? \.\.\. FAILED", text, re.M):
        return "caught", ""
    if re.search(r"^error\[E\d+\]:", text, re.M) or "could not compile" in text:
        return "invalid", ""
    if re.search(r"^error:", text, re.M) and not re.search(r"^test result: ", text, re.M):
        return "invalid", ""
    return "failure", ""


def verdict_of(row, code, text, memory):
    """(档, 那一句)：照 crates-mutation-rows.py 的 verdict_of_run（门禁同一份）。"""
    if BROKEN == "own-judge":
        return own_verdict(row, code, text)
    returncode = 1 if code == GPU_MEMORY_HIT_EXIT and BROKEN == "gpu-memory-as-caught" else code
    return ROWS.verdict_of_run(row, returncode, text, memory, NO_TIME_LIMIT, environment=os.environ)


def first_sentence(text):
    """verdict_of_run 那一句的头一行，去掉「crates/mutations.tsv:<行号> <名>：」。"""
    head = (text.splitlines() or [""])[0]
    return head.split("：", 1)[1] if "：" in head else head


def judged_line(row, code, text, memory, log_path):
    """(caught / failed / not-run, 要打的那一行)。"""
    verdict, sentence = verdict_of(row, code, text, memory)
    if verdict == "caught" and BROKEN != "never-caught":
        return "caught", f"{row.name}\t抓到\t{row.expected} 红了（日志 {log_path}）"
    if verdict == "invalid" or (verdict == "failure" and BROKEN == "error-line-invalid" and re.search(r"^error(?:\[E\d+\])?:", text, re.M)):
        return "failed", f"✗ {row.name}\t无效\t替换之后编不过（日志 {log_path}）"
    if verdict in ("failure", "caught"):
        detail = first_sentence(sentence) if verdict == "failure" and sentence else f"{row.expected} 没判红"
        return "failed", f"✗ {row.name}\t没红\t{detail}（日志 {log_path}）"
    if verdict == "notrun":
        return "not-run", f"- 本次未跑 {row.name}：{first_sentence(sentence)}（日志 {log_path}）"
    if verdict == "gpumemory":
        return "failed", f"✗ {row.name}：显存不够（内存包装退 {code}：起止之间内核报了 NV_ERR_NO_MEMORY），这一条的结果不算数，FAILED 不算抓到（日志 {log_path}）"
    return "failed", f"✗ {row.name}：内存包装的结局，退出码 {code}（日志 {log_path}）"


def rebuild_problem(copy, row, code, text, log_path, target_directory=None):
    """施加变异之后那一趟真重编了被改的包没有（文件头「判法与次序」⑤）：没有交要打的那一行，有（或这一趟不核）交 None。"""
    if BROKEN == "no-rebuild-check" or code == GPU_MEMORY_HIT_EXIT or 250 <= code <= 254:
        return None
    if target_directory and not os.path.isabs(target_directory):   # 相对路径是 cargo 照它的当前目录（副本根）解的
        target_directory = os.path.join(copy, target_directory)
    packages, _note = REBUILT.packages_to_check(copy, [row.path], target_directory)
    if not packages:   # 改的不是包里的 .rs、或这一趟编译不读它（不在编译目录的任何 dep-info 里）：cargo 本来就不因它重编，不核
        return None
    package = packages[0]
    if REBUILT.missing_rebuilds([package], [text]):
        return (f"✗ {row.name}\t没重编\t改了 {row.path}，这一趟的输出里却没有「Compiling {package} v…」行：编译目录里的产物比改过的源码新，"
                f"测试跑的是旧产物，判定不算数；刷源码修改时刻或换干净编译目录（不设 CARGO_TARGET_DIR，用副本自己的 target）重跑（日志 {log_path}）")
    return None


def cargo_environment(copy, target_directory=None):
    """cargo 那几次的环境：调用方设了（非空）CARGO_TARGET_DIR 照它的，没设就是副本自己的 target（target_directory 只给弄坏开关用）。
    证红要自己专用的编译目录，不与别的编译共用：每条变异改坏源码再还原，还原后的文件 mtime 可能比变异那次编出的产物旧，
    cargo 就把变异产物当成新的接着用——2026-09-28 在共用目录里另一份副本的测试就跑在了「步数不计数」那条变异的库上。"""
    environment = dict(os.environ)
    if BROKEN == "caller-target-overridden" or not environment.get("CARGO_TARGET_DIR"):
        environment["CARGO_TARGET_DIR"] = target_directory or os.path.join(copy, "target")
    return environment


def run_cargo(copy, arguments, memory, log_path, cargo, environment):
    with open(log_path, "w", encoding="utf-8") as log:
        result = subprocess.run(["bash", MEMORY_WRAPPER, memory, cargo, "test", *arguments], cwd=copy, stdout=log, stderr=subprocess.STDOUT,
                                env=environment)
    return result.returncode, open(log_path, encoding="utf-8", errors="replace").read()


def prove(copy, memory, crate, names, cargo="cargo", target_directory=None, main_root=None):
    """→ (退出码, [输出行])。main_root 是判「--copy 指的是不是主工作区」时的主工作区，不给就是这份脚本所在的仓根 MAIN_ROOT；
    自证给一棵临时的假主工作区，那一格判得出来不靠自证从哪棵树起（从符号链接拼的草稿树起时 MAIN_ROOT 里没有 crates/mutations.tsv）。"""
    copy = os.path.realpath(copy)
    main_root = MAIN_ROOT if main_root is None else os.path.realpath(main_root)
    environment = cargo_environment(copy, target_directory)
    if not os.path.isfile(os.path.join(copy, "crates", "mutations.tsv")):
        return 2, [f"✗ {copy} 里没有 crates/mutations.tsv，不像仓副本", "→ 怎么办：rsync -a --exclude target --exclude .git <仓根>/ <草稿目录>/copy/ 拷一份再给 --copy"]
    if copy == main_root and BROKEN != "main-allowed":
        return 2, [f"✗ --copy 指的是主工作区 {copy}", "→ 怎么办：在草稿目录里拷一份仓副本（rsync -a --exclude target --exclude .git），证红只在副本上改坏源码"]
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
        return prove_holding_the_lock(copy, memory, crate, names, cargo, environment)
    finally:
        if lock is not None:
            COPY_LOCK.release_copy_lock(copy, lock)


def prove_holding_the_lock(copy, memory, crate, names, cargo, environment):
    """拿着副本锁（文件头「判法与次序」⓪）往下做 ②–⑤：→ (退出码, [输出行])。"""
    rows, malformed = table_rows(copy)
    refusals, chosen, skipped = [], [], []
    for name in names:
        row = rows.get(name)
        if row is None:
            refusals.append(f"{name}：crates/mutations.tsv 里这一行不成形，{malformed[name]}" if name in malformed
                            else f"{name}：crates/mutations.tsv 里没有这一行")
            continue
        why = refusal_for(row, crate)
        heavy = None if why else heavy_reason(row, copy)
        if why:
            refusals.append(f"{name}：{why}")
        elif heavy and not (heavy_tests_allowed() or BROKEN == "heavy-run"):
            skipped.append(f"{name}：重型，{heavy}；没带 {HEAVY_TESTS_VARIABLE}=commit（或 =user-request），这一行一条都没跑")
        else:
            chosen.append(row)
    if refusals:
        return 2, [f"✗ {line}" for line in refusals] + ["→ 怎么办：照 crates/mutations.tsv 表头把那一行写成六段（替换文之外不许空），cargo test 参数加上 -p <crate> 与 --lib / --test <目标> / --bin <名> 之一再证；一条都没跑"]
    # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：有要跑的行就在起任何 cargo 之前判一次（判的是副本那棵树，起的是 cargo 那一份），
    #   判红整次拒绝、退 2、一条都不跑。树里没有依赖 rocksdb 的清单（自证的副本）、cargo 是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判
    if chosen and BROKEN != "rocksdb-entry-unchecked":
        entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT, "entry", "--root", copy, "--cargo", cargo], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if entry.returncode != 0:
            return 2, [line for line in (entry.stdout + entry.stderr).splitlines() if line.strip()] + [
                "✗ 起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条都没跑",
                '→ 怎么办：照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再证这一批']
    logs = os.environ.get("PROVE_RED_LOG_DIRECTORY") or os.path.join(copy, "prove-red-logs")
    os.makedirs(logs, exist_ok=True)
    output = [f"- 本次未跑 {line}" for line in skipped]
    for arguments in sorted({row.arguments for row in chosen}):
        code, text = run_cargo(copy, arguments.split(), memory, os.path.join(logs, "baseline.log"), cargo, environment)
        if code == GPU_MEMORY_HIT_EXIT and BROKEN != "gpu-memory-as-caught":
            return code, output + [f"✗ 基线那一次显存不够（内存包装退 {code}：起止之间内核报了 NV_ERR_NO_MEMORY），这一次不算结果（日志 {logs}/baseline.log）",
                                   "→ 怎么办：nvidia-smi 看谁在用卡，等它跑完再证这一批；不绕开包装重跑"]
        if 250 <= code <= 254:
            return code, output + [f"✗ 基线那一次撞了内存包装的结局（退出码 {code}，含义见 run-with-memory-cap.sh 文件头）", "→ 怎么办：照那一行的含义处理，不绕开包装重跑"]
        if code != 0 and BROKEN != "baseline-ignored":
            return 2, output + [f"✗ 基线（没改源码）就红：cargo test {arguments}（日志 {logs}/baseline.log）", "→ 怎么办：先让基线绿；基线里本来就红的测试不能拿来证红"]
    failed, caught, not_run = False, 0, len(skipped)
    for index, row in enumerate(chosen, 1):
        path = os.path.join(copy, row.path)
        original_text = open(path, encoding="utf-8").read()
        if original_text.count(row.old) != 1:
            output.append(f"✗ {row.name}：原文在 {row.path} 里命中 {original_text.count(row.old)} 次，要恰好 1 次")
            failed = True
            continue
        log_path = os.path.join(logs, f"{index:03d}.log")
        try:
            open(path, "w", encoding="utf-8").write(original_text.replace(row.old, row.new, 1))
            code, text = run_cargo(copy, row.arguments.split(), memory, log_path, cargo, environment)
        finally:
            open(path, "w", encoding="utf-8").write(original_text)
            os.utime(path)
        rebuilt_problem = rebuild_problem(copy, row, code, text, log_path, environment.get("CARGO_TARGET_DIR"))
        if rebuilt_problem:
            output.append(rebuilt_problem)
            failed = True
            continue
        outcome, line = judged_line(row, code, text, memory, log_path)
        output.append(line)
        failed = failed or outcome == "failed"
        caught += outcome == "caught"
        not_run += outcome == "not-run"
    summary = f"点名 {len(names)} 条：跑了 {len(chosen)} 条，抓到 {caught} 条，本次未跑 {not_run} 条"
    if failed:
        return 1, output + [f"✗ {summary}，有没红、无效或没重编的（逐条列在上面）",
                            "→ 怎么办：没红的换一条真会改行为的变异或补断言；无效的改替换文；照 .claude/rules/mutation-sampling.md 分类；没重编的按那一行刷修改时刻或换编译目录重跑"]
    if caught == 0:
        return EXIT_NOT_RUN, output + [f"- {summary}：一条都没抓到，点名的都是本次未跑（逐条列在上面）",
                                       f"→ 怎么办：重型的那几行交回主 agent，由它在提交时带 {HEAVY_TESTS_VARIABLE}=commit 跑；打了 NOT_RUN 的换一台有卡的机器再证"]
    return 0, output + [f"✓ {summary}，跑了的都抓到了"]


def write_text(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


# 自证的样本表：checker 那一行是 checker 档包的（重型），noexpected 那一行必须红的测试名空着（不成形）
SELFTEST_TABLE = ("# 样本\n"
                  "caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "untargeted\tcrates/demo/src/lib.rs\t{ 42 }\t{ 43 }\t-p demo -- answer\tanswer_is_42\n"
                  "layer\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --test demo_crash_point_replay -- answer\tanswer_is_42\n"
                  "panics\tcrates/demo/src/lib.rs\t{ 42 }\t{ 45 }\t-p demo --lib -- answer\tanswer_panics\n"
                  "broken\tcrates/demo/src/lib.rs\t{ 42 }\t{ 46 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "elsewhere\tcrates/demo/src/lib.rs\t{ 42 }\t{ 47 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 48 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "checker\tcrates/singlefs-checker-tier/src/lib.rs\t{ 42 }\t{ 41 }\t-p singlefs-checker-tier --lib -- answer\tanswer_is_42\n"
                  "redbase\tcrates/redbase/src/lib.rs\t{ 50 }\t{ 51 }\t-p redbase --lib -- answer\tanswer_is_42\n"
                  "noexpected\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\t\n"
                  "five-fields\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\n")

# 判法逐行相同那一格的样本：(说明, 退出码, 输出, 点名)；几种改前 prove-red 与 verdict_of_run 判得不一样的都在里面
SELFTEST_VERDICT_SAMPLES = (
    ("点名的红了、退 101", 101, "test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("should panic 的红了", 101, "test tests::answer_panics - should panic ... FAILED\ntest result: FAILED. 0 passed; 1 failed\n", "answer_panics"),
    ("点名省掉模块路径、两个模块各红一个", 101, "test a::answer_is_42 ... FAILED\ntest b::answer_is_42 ... FAILED\ntest result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
    ("退出码 0 而输出里有 FAILED", 0, "test tests::answer_is_42 ... FAILED\n", "answer_is_42"),
    ("编不过", 101, "error[E0308]: mismatched types\nerror: could not compile `demo`\n", "answer_is_42"),
    ("点名的绿、别的红", 101, "test tests::answer_is_42 ... ok\ntest tests::other ... FAILED\ntest result: FAILED. 1 passed; 1 failed\nerror: test failed\n", "answer_is_42"),
    ("点名的绿、打了 NOT_RUN", 0, "test tests::answer_is_42 ... ok\n---- tests::answer_is_42 stdout ----\nNOT_RUN_NO_GPU_ADAPTER test=tests::answer_is_42 reason=没卡\n"
                                   "test result: ok. 1 passed; 0 failed\n", "answer_is_42"),
    ("包装退 245", 245, "test tests::answer_is_42 ... FAILED\n", "answer_is_42"),
)


def selftest_copy(work, name="copy"):
    copy = os.path.join(work, name)
    write_text(os.path.join(copy, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    for package, answer in (("demo", 42), ("singlefs-checker-tier", 42), ("redbase", 50)):
        write_text(os.path.join(copy, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\n')
        write_text(os.path.join(copy, "crates", package, "src", "lib.rs"), f"fn answer() -> u32 {{ {answer} }}\n")
    write_text(os.path.join(copy, "crates", "mutations.tsv"), SELFTEST_TABLE)
    return copy


def selftest():
    work = tempfile.mkdtemp(prefix="prove-red-selftest-")
    saved_heavy_tests = os.environ.pop(HEAVY_TESTS_VARIABLE, None)
    try:
        copy = selftest_copy(work)
        fake_cargo = os.path.join(work, "bin", "cargo")
        target_log = os.path.join(work, "target-directories.txt")
        cargo_log = os.path.join(work, "cargo-arguments.txt")
        lock_log = os.path.join(work, "copy-lock.txt")
        # 假 cargo 照源码里那个数打 libtest / cargo 的原样形态：42 基线全绿；45 是 #[should_panic] 的用例没 panic
        # （结果行在名字与「 ... 」之间多一段「 - should panic」，cargo 收尾还打一行 error: test failed）；46 替换之后编不过；
        # 47 点名的测试绿、别的测试红（同样带 error: test failed 与 test result: 行）；别的数照旧只打点名的那一行 FAILED。
        # 读哪份源码看 -p：-p <包> 读 crates/<包>/src/lib.rs（redbase 那个包的源码是 50，基线就红）。
        # 每起一次先往 target_log 记一行它看到的 CARGO_TARGET_DIR（编译目录那两格核它），往 cargo_log 记一行参数（重型那两格核它起没起）
        write_text(fake_cargo,
                   "#!/usr/bin/env bash\n"
                   f"printf '%s\\n' \"${{CARGO_TARGET_DIR-<未设>}}\" >> {shlex.quote(target_log)}\n"
                   f"printf '%s\\n' \"$*\" >> {shlex.quote(cargo_log)}\n"
                   # 副本锁那一格：起的时候副本根（当前目录）上的锁被人拿着记 held，没有锁文件或拿得到记 free
                   f"if [[ -e {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} ]] && ! flock -n {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} true 2>/dev/null; then echo held; else echo free; fi >> {shlex.quote(lock_log)}\n"
                   "package=demo; previous=''\n"
                   "for argument in \"$@\"; do [[ \"$previous\" == -p ]] && package=\"$argument\"; previous=\"$argument\"; done\n"
                   "source_file=\"crates/$package/src/lib.rs\"\n"
                   # 照 cargo 按 mtime 判新不新鲜：编译目录里这个包的产物不在、或源码比它新，才重编（拷一份当产物、照真 cargo 打 Compiling 行）；
                   # 测试跑的是产物，不是此刻的源码（编译目录里留着比源码新的旧产物时，测试跑在旧产物上）
                   "build_directory=\"${CARGO_TARGET_DIR:-target}/fake-build\"; mkdir -p \"$build_directory\"; artifact=\"$build_directory/$package.rs\"\n"
                   "if [[ ! -e \"$artifact\" || \"$source_file\" -nt \"$artifact\" ]]; then cp \"$source_file\" \"$artifact\"; echo \"   Compiling $package v0.1.0 ($PWD/crates/$package)\"\n"
                   # 照 rustc 写 dep-info（<编译目录>/debug/deps/<包>.d 列出读过的源文件）：没重编那一格靠它认出这个文件是这一趟编译读的
                   "  mkdir -p \"${CARGO_TARGET_DIR:-target}/debug/deps\"; echo \"$artifact: $PWD/$source_file\" > \"${CARGO_TARGET_DIR:-target}/debug/deps/$package.d\"; fi\n"
                   "case \"$(cat \"$artifact\")\" in\n"
                   "*'{ 42 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... ok\\n\\n"
                   "test result: ok. 2 passed; 0 failed; 0 ignored\\n'; exit 0 ;;\n"
                   "*'{ 45 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n\\n---- tests::answer_panics stdout ----\\nnote: test did not panic as expected at crates/demo/src/lib.rs:9:5\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   "*'{ 46 }'*) printf 'error[E0308]: mismatched types\\n --> crates/demo/src/lib.rs:1:22\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "*'{ 47 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   "*) echo 'test tests::answer_is_42 ... FAILED'; exit 101 ;;\n"
                   "esac\n", executable=True)
        fake_wrapper = os.path.join(work, "wrapper.sh")
        # 假内存包装：照跑命令；源码里是 48 时（那一条 cargo 照旧打点名的 FAILED）跑完改退 245，演「显存不够」
        write_text(fake_wrapper, "#!/usr/bin/env bash\nshift\n\"$@\"; status=$?\n"
                                 "if grep -q '{ 48 }' crates/demo/src/lib.rs 2>/dev/null; then echo 'run-with-memory-cap: 显存不够（假包装），退 245'; exit 245; fi\n"
                                 "exit \"$status\"\n")
        global MEMORY_WRAPPER
        MEMORY_WRAPPER = fake_wrapper
        os.environ["PROVE_RED_LOG_DIRECTORY"] = os.path.join(work, "logs")
        failures = []
        checked_cases = 0

        def cargo_calls():
            calls = open(cargo_log).read().splitlines() if os.path.exists(cargo_log) else []
            if os.path.exists(cargo_log):
                os.unlink(cargo_log)
            return calls

        code, lines = prove(copy, "1G", "demo", ["caught"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"会红的变异应当抓到、退 0，实际 {code} {lines}")
        checked_cases += 1
        if open(os.path.join(copy, "crates", "demo", "src", "lib.rs")).read() != "fn answer() -> u32 { 42 }\n":
            failures.append("跑完没把源码写回原样")
        code, lines = prove(copy, "1G", "demo", ["caught", "untargeted"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("没挑目标" in line for line in lines):
            failures.append(f"不挑目标的变异行应当整次拒绝、退 2，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["layer"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("layer\t抓到\t") for line in lines):
            failures.append(f"--test 目标名带 crash_point_replay 而包不是 checker 档：照跑、判抓到、退 0（重型只按包判），实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["panics"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("panics\t抓到\t") for line in lines):
            failures.append(f"#[should_panic] 的用例红了（结果行带「 - should panic」、收尾有 error: test failed）应当判抓到、退 0，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["broken"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ broken\t无效\t") for line in lines):
            failures.append(f"替换之后编不过（error[E…] 与 could not compile）应当判无效、退 1，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["elsewhere"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ elsewhere\t没红\t") for line in lines):
            failures.append(f"编过了、别的测试红而点名的没红（有 error: test failed 也有 test result:）应当判没红、不判无效，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["gpu"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ gpu：显存不够") for line in lines) or any(line.startswith("gpu\t抓到\t") for line in lines):
            failures.append(f"内存包装退 245（显存不够）的那一条应当报显存不够、不算抓到、退 1，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "redbase", ["redbase"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("基线（没改源码）就红" in line for line in lines) or any(line.startswith("redbase\t抓到\t") for line in lines):
            failures.append(f"基线（没改源码）就红：停下、退 2、一条变异都不记抓到，实际 {code} {lines}")
        # 起 cargo 之前判 RocksDB 预编库的取值：另一份副本里放一份依赖 rocksdb 的 checker 档包清单、进程环境里四个取值是空的，假 cargo 不带 entry 认的标记
        #   ⇒ 整次拒绝、退 2、假 cargo 一次都没起（别的格副本里没有这份清单，entry 报不判）
        rocksdb_copy = selftest_copy(work, "rocksdb")
        write_text(os.path.join(rocksdb_copy, "crates", "singlefs-checker-tier", "Cargo.toml"),
                   '[package]\nname = "singlefs-checker-tier"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
        saved_values = {name: os.environ.get(name) for name in ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")}
        os.environ.update({name: "" for name in saved_values})
        try:
            cargo_calls()
            code, lines = prove(rocksdb_copy, "1G", "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            for name, value in saved_values.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        checked_cases += 1
        if code != 2 or calls or not any("rocksdb-prebuilt entry：" in line for line in lines):
            failures.append(f"副本里的 checker 档包依赖 rocksdb 而进程环境里没有预编库的取值：应当在起 cargo 之前整次拒绝、退 2、假 cargo 一次都不起，实际退 {code}、假 cargo 起了 {len(calls)} 次 {lines}")
        # 主工作区拿临时目录里一棵同样形态的树（有 crates/mutations.tsv）顶替、经 main_root 交给 prove：自证从哪棵树起（仓里、仓副本、符号链接拼的草稿树）都判得出这一格
        fake_main = selftest_copy(work, "main")
        code, lines = prove(fake_main, "1G", "demo", ["caught"], cargo=fake_cargo, main_root=fake_main)
        checked_cases += 1
        if code != 2 or not any("主工作区" in line for line in lines):
            failures.append(f"--copy 指主工作区应当以「主工作区」为由拒绝，实际 {code} {lines}")
        # 重型那两格：照 lib_heavy_tests.classify 判，checker 档包的行子 agent 起时不跑、主 agent 带前缀起时照跑
        cargo_calls()
        code, lines = prove(copy, "1G", "singlefs-checker-tier", ["checker"], cargo=fake_cargo)
        calls = cargo_calls()
        checked_cases += 1
        if code != EXIT_NOT_RUN or calls or not any(line.startswith("- 本次未跑 checker：重型") for line in lines):
            failures.append(f"子 agent 起（没设 {HEAVY_TESTS_VARIABLE}）：checker 档包的行跳过、逐行报本次未跑、cargo 一次都不起、整次退 77，"
                            f"实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        os.environ[HEAVY_TESTS_VARIABLE] = "commit"
        try:
            code, lines = prove(copy, "1G", "singlefs-checker-tier", ["checker"], cargo=fake_cargo)
        finally:
            os.environ.pop(HEAVY_TESTS_VARIABLE, None)
        calls = cargo_calls()
        checked_cases += 1
        if code != 0 or not any(line.startswith("checker\t抓到\t") for line in lines) or not any("-p singlefs-checker-tier" in call for call in calls):
            failures.append(f"主 agent 带 {HEAVY_TESTS_VARIABLE}=commit 起：checker 档包的行照跑、判抓到、退 0，实际退 {code}、cargo 的参数 {calls} {lines}")
        # 切表两格：点名的行不成形整次拒绝；切出来的行与 read_table 逐行相同
        code, lines = prove(copy, "1G", "demo", ["noexpected"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("不成形" in line and "必须红的测试名" in line for line in lines):
            failures.append(f"点名的行不成形（必须红的测试名空着）：整次拒绝、写明哪一段空着、退 2，实际 {code} {lines}")
        ours = sorted(table_rows(copy)[0])
        theirs = sorted({row.name for row in ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))[0]})
        checked_cases += 1
        if ours != theirs:
            failures.append(f"切表与 read_table 逐行相同：这里切出 {ours}，read_table 切出 {theirs}")
        # 判法那一格：同一组样本，这里判的档与 verdict_of_run 判的逐行相同
        differing = []
        for description, sample_code, sample_output, expected in SELFTEST_VERDICT_SAMPLES:
            row = ROWS.Row(1, "sample", "crates/demo/src/lib.rs", "a", "b", "-p demo --lib", expected, ())
            ours = verdict_of(row, sample_code, sample_output, "1G")[0]
            theirs = ROWS.verdict_of_run(row, sample_code, sample_output, "1G", NO_TIME_LIMIT, environment=os.environ)[0]
            if ours != theirs:
                differing.append(f"{description}：这里判 {ours}，verdict_of_run 判 {theirs}")
        checked_cases += 1
        if differing:
            failures.append(f"判法与 verdict_of_run 对同一组样本逐行相同（{len(SELFTEST_VERDICT_SAMPLES)} 条），对不上的：{'；'.join(differing)}")
        # 副本锁那一格（文件头「判法与次序」⓪）：证红跑的时候副本根上拿着锁（假 cargo 每次起都试拿一次，记 held / free）、跑完放掉、锁文件不留；
        #   锁被别人占着（这里自己先拿着，演另一趟证红）时整次拒绝、退 2、cargo 一次都不起
        if os.path.exists(lock_log):
            os.unlink(lock_log)
        lock_wrong = []
        code, lines = prove(copy, "1G", "demo", ["caught"], cargo=fake_cargo)
        seen = open(lock_log).read().split() if os.path.exists(lock_log) else []
        if code != 0 or not seen or set(seen) != {"held"}:
            lock_wrong.append(f"跑的时候假 cargo 每次都应当看到锁被拿着（held），实际 {seen}、退 {code}")
        if os.path.exists(os.path.join(copy, COPY_LOCK.COPY_LOCK_NAME)):
            lock_wrong.append("跑完锁文件还在")
        cargo_calls()
        held = COPY_LOCK.hold_copy_lock(copy)
        try:
            code, lines = prove(copy, "1G", "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            if held is not None:
                COPY_LOCK.release_copy_lock(copy, held)
        if held is None or code != 2 or calls or not any("副本上有证红在跑" in line for line in lines):
            lock_wrong.append(f"锁被占着时应当拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}（自证{'拿到了' if held is not None else '没拿到'}锁）")
        checked_cases += 1
        if lock_wrong:
            failures.append("证红跑的时候副本根上拿着副本锁、跑完放掉；锁被占着时拒绝、退 2、cargo 一次都不起：" + "；".join(lock_wrong))
        # 没重编那一格（文件头「判法与次序」⑤）：调用方设的 CARGO_TARGET_DIR 是一份共用编译目录，里面 demo 的产物（没改过的源码）比施加变异后的源码新
        #   （另一份副本后编的那种，修改时刻取明天）：假 cargo 不重编、测试跑在旧产物上全绿；要判没重编、退 1，不判没红
        shared_target = os.path.join(work, "shared-target")
        stale_artifact = os.path.join(shared_target, "fake-build", "demo.rs")
        write_text(stale_artifact, "fn answer() -> u32 { 42 }\n")
        # 上一次在这份编译目录里编这个副本时留下的 dep-info：lib.rs 是编译读过的文件
        write_text(os.path.join(shared_target, "debug", "deps", "demo.d"), f"{stale_artifact}: {os.path.realpath(copy)}/crates/demo/src/lib.rs\n")
        tomorrow = time.time() + 86400
        os.utime(stale_artifact, (tomorrow, tomorrow))
        saved_target = os.environ.get("CARGO_TARGET_DIR")
        os.environ["CARGO_TARGET_DIR"] = shared_target
        try:
            code, lines = prove(copy, "1G", "demo", ["caught"], cargo=fake_cargo)
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if saved_target is not None:
                os.environ["CARGO_TARGET_DIR"] = saved_target
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ caught\t没重编\t") and "Compiling demo" in line for line in lines):
            failures.append(f"编译目录里的产物比施加变异后的源码新（共用编译目录）：那一条判没重编、退 1，不判没红，实际退 {code} {lines}")
        # 编译目录三格：开头那几行不碰位置参数（只给 --selftest 也走得过去）；默认值在解析完 --copy 之后定，次序不限；调用方设了的不覆盖
        caller_target = os.environ.pop("CARGO_TARGET_DIR", None)
        try:
            probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True,
                                   env=dict(os.environ, PROVE_RED_SELFTEST_PROLOGUE="1"))
            checked_cases += 1
            if probe.returncode != 0 or "PROVE_RED_PROLOGUE CARGO_TARGET_DIR=<未设>" not in probe.stdout.splitlines():
                failures.append(f"不设 CARGO_TARGET_DIR、只给 --selftest：开头那几行应当走得过去、不替调用方设编译目录（子进程打 CARGO_TARGET_DIR=<未设>、退 0），"
                                f"实际退 {probe.returncode}，输出 {(probe.stdout + probe.stderr).strip()!r}")
            # 编译并行度：没设 CARGO_BUILD_JOBS 取登记文件第一条非注释行的数（SINGLEFS_THREAD_CAP 也没设），设了的照它
            with open(os.path.join(MAIN_ROOT, ".claude", "heavy-thread-default"), encoding="utf-8") as registry:
                registered_jobs = next((line.split()[0] for line in registry if line.strip() and not line.lstrip().startswith("#")), "<登记里没有数>")
            for preset, want in ((None, registered_jobs), ("3", "3")):
                environment = {name: value for name, value in os.environ.items() if name not in ("CARGO_BUILD_JOBS", "SINGLEFS_THREAD_CAP")}
                environment["PROVE_RED_SELFTEST_PROLOGUE"] = "1"
                if preset is not None:
                    environment["CARGO_BUILD_JOBS"] = preset
                jobs_probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True, env=environment)
                checked_cases += 1
                if jobs_probe.returncode != 0 or f"PROVE_RED_PROLOGUE CARGO_BUILD_JOBS={want}" not in jobs_probe.stdout.splitlines():
                    failures.append(f"{'不设' if preset is None else '调用方设了'} CARGO_BUILD_JOBS：开头那几行应当{'取登记的重活默认并行度 ' if preset is None else '照它的 '}{want}（子进程打 CARGO_BUILD_JOBS={want}），"
                                    f"实际退 {jobs_probe.returncode}，输出 {(jobs_probe.stdout + jobs_probe.stderr).strip()!r}")
            if os.path.exists(target_log):
                os.unlink(target_log)
            code, lines = run(["--memory", "1G", "--copy", copy, "demo", "caught"], cargo=fake_cargo)
            seen = open(target_log).read().splitlines() if os.path.exists(target_log) else []
            checked_cases += 1
            if code != 0 or not seen or set(seen) != {os.path.join(os.path.realpath(copy), "target")}:
                failures.append(f"--copy 不在第一个（--memory 1G --copy <副本>）：编译目录应当是 <副本>/target、照常抓到退 0，实际退 {code}，cargo 看到的 {seen} {lines}")
            if os.path.exists(target_log):
                os.unlink(target_log)
            os.environ["CARGO_TARGET_DIR"] = os.path.join(work, "caller-target")
            code, lines = run(["--copy", copy, "demo", "caught"], cargo=fake_cargo)
            seen = open(target_log).read().splitlines() if os.path.exists(target_log) else []
            checked_cases += 1
            if code != 0 or not seen or set(seen) != {os.path.join(work, "caller-target")}:
                failures.append(f"调用方设了 CARGO_TARGET_DIR：应当照它的、不覆盖，实际退 {code}，cargo 看到的 {seen}")
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if caller_target is not None:
                os.environ["CARGO_TARGET_DIR"] = caller_target
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 prove() / table_rows() / refusal_for() / heavy_reason() / verdict_of() / judged_line() 的判法（判结局在 crates-mutation-rows.py 的 verdict_of_run，"
                  "结果行在 lib_libtest_result_lines.py），编译目录那三格看 run() / cargo_environment() 与开头 python3 之前那几行；"
                  "PROVE_RED_BREAK 或 LIBTEST_RESULT_LINES_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ prove-red 自检通过（判法样本 {len(SELFTEST_VERDICT_SAMPLES)} 条）：会红的抓到并写回源码，should_panic 红了抓到，编不过判无效，别的测试红判没红，包装退 245 报显存不够不算抓到，基线红停下退 2，"
              f"不挑目标的整次拒绝，目标名带 crash_point_replay 而包不是 checker 档的照跑，主工作区拒绝，"
              f"checker 档包的行子 agent 起时本次未跑退 77、带前缀起时照跑，不成形的行整次拒绝，切表与 read_table 相同，"
              f"判法与 verdict_of_run 对 {len(SELFTEST_VERDICT_SAMPLES)} 条样本逐行相同，跑的时候副本根上拿着副本锁、跑完放掉、锁被占着时拒绝，"
              f"共用编译目录里的产物比施加变异后的源码新时判没重编，"
              f"不设 CARGO_TARGET_DIR 只给 --selftest 走得过开头，--copy 不在第一个时编译目录照样是副本的 target，调用方设了的不覆盖（查了 {checked_cases} 种）")
        return 0
    finally:
        os.environ.pop(HEAVY_TESTS_VARIABLE, None)
        if saved_heavy_tests is not None:
            os.environ[HEAVY_TESTS_VARIABLE] = saved_heavy_tests
        shutil.rmtree(work, ignore_errors=True)


def run(arguments, cargo="cargo"):
    """→ (退出码, [输出行])：先解析 --copy / --memory（两个的次序不限），再证红；编译目录的默认值由 prove 照解析出的副本定。"""
    raw_arguments = list(arguments)
    copy, memory = None, default_memory_cap()
    while arguments and arguments[0] in ("--copy", "--memory"):
        if len(arguments) < 2:
            return 2, ["✗ 选项后面没给值", "→ 怎么办：prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>"]
        if arguments[0] == "--copy":
            copy = arguments[1]
        else:
            memory = arguments[1]
        arguments = arguments[2:]
    if copy is None or len(arguments) < 2:
        return 2, ["✗ 用法不对", "→ 怎么办：prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>"]
    # 弄坏开关 second-argument-target：改前 $2 的取法，--copy 不在第一个时取错
    target_directory = os.path.join(raw_arguments[1], "target") if BROKEN == "second-argument-target" and len(raw_arguments) > 1 else None
    return prove(copy, memory, arguments[0], arguments[1:], cargo=cargo, target_directory=target_directory)


def main():
    arguments = sys.argv[2:]
    if arguments == ["--selftest"]:
        if os.environ.get("PROVE_RED_SELFTEST_PROLOGUE"):
            # 自证起的那一层子进程：只证开头那几行走得过去，打它此刻看到的 CARGO_TARGET_DIR 就退（编译目录那一格核这一行）
            print(f"PROVE_RED_PROLOGUE CARGO_TARGET_DIR={os.environ.get('CARGO_TARGET_DIR', '<未设>')}")
            print(f"PROVE_RED_PROLOGUE CARGO_BUILD_JOBS={os.environ.get('CARGO_BUILD_JOBS', '<未设>')}")
            return 0
        return selftest()
    code, lines = run(arguments)
    for line in lines:
        print(line)
    return code


sys.exit(main())
PY
