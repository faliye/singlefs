"""crates-mutation-rows.py 拆出的一份。
标准库与仓内库的载入、全部模块级赋值（常量、表、正则、假 cargo 与假 llvm 工具的正文、模块级可变状态如 WORKER_STATE 与 RUN_BUILD_REMOVAL_FAILURES：别的份 import 同一个对象，不各存一份）、弄坏开关的读取（break_is）、say / note、变异表一行的成形与读表、拷贝范围的头一段，以及几份共用的小函数。模块级赋值都在这一份（自证用的两张模块表不在这一份：FIXTURE_MODULES 在 driver、WRAPPED_FUNCTION_MODULES 在 commands）。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""


import ast
import collections
import concurrent.futures
import contextlib
import fnmatch
import gzip
import hashlib
import heapq
import importlib.util
import io
import json
import multiprocessing
import os
import random
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import traceback
from concurrent.futures.process import BrokenProcessPool

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
STAGE_REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(STAGE_REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import HeavyThreadDefaultError, default_heavy_threads  # noqa: E402
import multi_host_addresses  # noqa: E402

LIBTEST_RESULT_LINES_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_libtest_result_lines.py")   # libtest 结果行的解析（全仓一份）
_LIBTEST_RESULT_LINES_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_crates_mutation_rows", LIBTEST_RESULT_LINES_PATH)
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_RESULT_LINES_SPECIFICATION)
_LIBTEST_RESULT_LINES_SPECIFICATION.loader.exec_module(libtest_result_lines)
MUTATION_UNITS_PATH = os.path.join(SCRIPT_DIRECTORY, "lib_mutation_units.py")   # 函数单元键与按条判定库（与 mutate.sh 共用）
_MUTATION_UNITS_SPECIFICATION = importlib.util.spec_from_file_location("lib_mutation_units_for_crates_mutation_rows", MUTATION_UNITS_PATH)
mutation_units = importlib.util.module_from_spec(_MUTATION_UNITS_SPECIFICATION)
sys.modules[_MUTATION_UNITS_SPECIFICATION.name] = mutation_units   # 按名字登记：算闭包的子进程靠这个名字找回要跑的函数
_MUTATION_UNITS_SPECIFICATION.loader.exec_module(mutation_units)
write_atomically = mutation_units.write_atomically   # 同目录排他建临时文件、写完 fsync、改名换上；data 是 str 或 bytes
REUSE_CHECKS_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_reuse_checks.py")   # 复用三问与复用哨兵（全仓一份）
_REUSE_CHECKS_SPECIFICATION = importlib.util.spec_from_file_location("lib_reuse_checks_for_crates_mutation_rows", REUSE_CHECKS_PATH)
reuse_checks = importlib.util.module_from_spec(_REUSE_CHECKS_SPECIFICATION)
_REUSE_CHECKS_SPECIFICATION.loader.exec_module(reuse_checks)
SCRIPT_KEY = "scripts/mutation/crates-mutation-rows.py"   # 规范输出与复用哨兵认的脚本名

STAGE = "crates-mutation-replay.sh"   # 登记表的键（拷贝范围）、阶段脚本的文件名
STAGE_SCRIPT = os.path.join(STAGE_REPOSITORY, ".claude", "gate.d", STAGE)
MODULE_PATH = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))   # 入口文件：自证起的子进程按这个路径载入本模块
# 入口与拆出的各份模块的源码：自证里扫源码的几格都按这一张表读（拆出的新模块加进来，自证核它列全了）
MODULE_SOURCE_PATHS = (
    MODULE_PATH,
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_base.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_judging.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_support.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_static.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_units.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_baseline.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_targets.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_memory.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_runs.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_pool.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_plan.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_stage.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_commands.py"),
    os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_driver.py"),
)
SELFTEST_FIXTURES_PATH = os.path.join(SCRIPT_DIRECTORY, "crates_mutation_rows_selftest_fixtures.py")   # 自证夹具（文件头同名一段），只在 run_selftest 里载入
# 自证起的子进程里跑的代码串：留在本文件、模块级只赋一次的字面串，函数单元的算键库（scripts/gate/lib_function_units.py 的 fold）才折得出
# [sys.executable, "-c", <名字>, …] 的 -c 串；放进夹具文件、经 load_selftest_fixtures 放进 globals() 的名字它折不出，起这个子进程的格会整行拒算 inline-script-not-literal。
# 「进程池还没起就被停」那一格：拿掉 concurrent.futures 的 process 子模块（进程池还没起时它没载入），把 run_pool 换成一进来就抛 StopRequested 的，再调 run_pool_reporting_dead_worker
POOL_ERROR_CHILD_PROGRAM = ("import concurrent.futures, importlib.util, os, sys\n"
                            "specification = importlib.util.spec_from_file_location('crates_mutation_rows_pool_error_child', sys.argv[1])\n"
                            "module = importlib.util.module_from_spec(specification)\n"
                            "sys.modules[specification.name] = module\n"
                            "specification.loader.exec_module(module)\n"
                            "sys.modules.pop('concurrent.futures.process', None)\n"
                            "concurrent.futures.__dict__.pop('process', None)\n"
                            "def stopped_before_the_pool(*arguments, **keywords):\n"
                            "    raise module.StopRequested(143)\n"
                            "sys.modules['crates_mutation_rows_pool'].run_pool = stopped_before_the_pool\n"
                            "module.run_pool_reporting_dead_worker(dict(environment=dict(os.environ)), 1, [], [], set(), None, None)\n")
# 「被 TERM 停」那几格在子进程里跑 run_stage（按路径载入本模块）。子进程的进程环境里有调用方的 SINGLEFS_MULTI_HOST_CONFIG（Selftest.pool_process_environment 交的，
#   进程池的工作进程要它认这一台的地址），传给 run_stage 的 environment 里去掉它：交给测试的多机配置照自证用的 GATE_MUTATION_MULTI_HOST_CONFIG，同这一份自证里别的格
STAGE_CHILD_PROGRAM = ("import importlib.util, os, sys\n"
                       "specification = importlib.util.spec_from_file_location('crates_mutation_rows_selftest_child', sys.argv[1])\n"
                       "module = importlib.util.module_from_spec(specification)\n"
                       "sys.modules[specification.name] = module\n"
                       "specification.loader.exec_module(module)\n"
                       "sys.exit(module.run_stage(sys.argv[2], {name: value for name, value in os.environ.items() if name != 'SINGLEFS_MULTI_HOST_CONFIG'}))\n")
ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（run_stage）
TABLE = "crates/mutations.tsv"
# 判定库（文件头「判定库」）：被判那棵树的 git common-dir 底下这一个目录，固定一处、不按版本号分目录
LIBRARY_DIRECTORY_NAME = "singlefs-crates-mutation-library"
ROW_KEY_FORM = "singlefs-crates-mutation-row-v1"
BASELINE_KEY_FORM = "singlefs-crates-mutation-baseline-v1"
COVERAGE_KEY_FORM = "singlefs-crates-mutation-coverage-v1"
LLVM_TOOLS = ("llvm-profdata", "llvm-cov")
LLVM_TOOLS_VARIABLE = "GATE_MUTATION_LLVM_TOOLS"
COVERAGE_TARGET_SUFFIX = "-coverage"
# 编译目录（文件头「编译目录」）：GATE_MUTATION_TARGET_DIR 设了用它，其次环境里的 CARGO_TARGET_DIR 底下那一份（都跑完不删）；都没设建在这一趟的 <shard_root>/build 里、跑完删
TARGET_DIRECTORY_NAME = "singlefs-crates-mutation-target"
RUN_BUILD_DIRECTORY_NAME = "build"
RUN_BUILD_REMOVAL_FAILURES = []   # run_pool 收尾没删成的这一趟编译目录（有进程开着里面的文件、或删不成）：run_stage 照它整道判红
# 工作进程活着的时候在自己的编译目录里拿着这把共享锁（flock）：别的一趟回收多出来的 -w<片号> 之前试拿排他锁，拿不到就是有工作进程在用
WORKER_TARGET_LOCK_NAME = ".singlefs-mutation-worker.lock"
# 片号跨调用经锁领：共用编译目录那一层里每个片号一把锁文件（第 1 片 <共用目录>.slot，其余片 <共用目录>-w<片号>.slot）；工作进程启动时从 1 往上试拿排他锁（不阻塞），
# 拿到第一个空闲的就是它的片号，锁拿到进程退出。两次并发调用因此拿到不相交的片号集合，不会两边的第 2 片都指向同一个 -w2 而被 cargo 的目录锁互相排队
WORKER_SLOT_LOCK_SUFFIX = ".slot"
WORKER_SLOT_SEARCH_LIMIT = 4096
COVERAGE_FLAG = "-Cinstrument-coverage"
BUILD_PROFILES_DIRECTORY_NAME = "build-profiles"   # 插桩编译那一步（构建脚本、过程宏）写出的剖析文件的去处，在那一格的 TMPDIR 下，随 TMPDIR 一起删
# 这一道起的 cargo 关调试信息、release 档开增量（文件头「调试信息与 release 增量」）：cargo test 用 test 档（承 dev），cargo test --release 用 release 档（bench 承 release），
# 调试信息四档都设。取值放在一格的列表里：自证「改编译档的 debug 之后行键不变」「改编译档的 incremental 之后行键变」两格就地换它
CARGO_DEBUG_PROFILES = ("DEV", "TEST", "RELEASE", "BENCH")
CARGO_DEBUG_SETTING = ["false"]
CARGO_RELEASE_INCREMENTAL_SETTING = ["true"]
# 先编一份依赖（文件头「先编一份依赖」）：<共用目录>-seed（插桩的再加 -coverage）、源码副本 <共用目录>-seed-source、锁 <共用目录>-seed.lock
SEED_TARGET_SUFFIX = "-seed"
SEED_SOURCE_SUFFIX = "-seed-source"
SEED_LOCK_SUFFIX = "-seed.lock"
# 组的键（seed_group_of）：选目标的开关与只管输出的开关不进键，选包、特性、档、目标三元组与别的开关进键
SEED_SELECTOR_SWITCHES = frozenset(("--lib", "--bins", "--tests", "--benches", "--examples", "--doc", "--all-targets"))
SEED_VALUE_SELECTOR_SWITCHES = frozenset(("--test", "--bin", "--example", "--bench"))
SEED_DROPPED_SWITCHES = frozenset(("-q", "--quiet", "--offline", "-v", "--verbose", "--no-run"))
SEED_VALUE_DROPPED_SWITCHES = frozenset(("-j", "--jobs", "--color", "--message-format"))
SEED_VALUE_KEPT_SWITCHES = frozenset(("-p", "--package", "--features", "-F", "--profile", "--target", "--manifest-path", "--config", "-Z"))
# 编译目录里一个单元的产物与目录：deps/ 下「[lib]<名字>-<16 位哈希>[.<后缀>]」，.fingerprint/ 与 build/ 下「<包名>-<16 位哈希>」
UNIT_ARTIFACT_NAME = re.compile(r"^.+-(?P<hash>[0-9a-f]{16})(?:\..+)?$")
UNIT_DIRECTORY_NAME = re.compile(r"^(?P<name>.+)-[0-9a-f]{16}$")
SEED_COPY_TEMPORARY_SUFFIX = ".singlefs-seed-copy"
# 现判的栏（row_decision 交的栏名，次序就是报数的次序；拒算另报，不在栏里）
REUSE_COLUMNS = (("row-new", "行新加"), ("mutated", "被改坏的单元变了"),
                 ("test", "点名测试的闭包变了"), ("text", "原文或替换文变了"),
                 ("not-caught", "上次没判成抓到"), ("unexplained", "键对不上"))
COLUMN_TITLES = dict(REUSE_COLUMNS, refused="拒算", sentinel="复用哨兵强制现判")
REASONS_SHOWN = 20   # stderr 逐行列现判与拒算原因的行数上限（只管列多少，全部的看 plan）
REUSABLE_VERDICT = {"mutation": "caught", "baseline": "ok"}
RECORD_HEADER = ("# 门禁 crates-mutation-replay 的判定库记录（scripts/mutation/crates-mutation-rows.py 写，一条一个文件，按键寻址）：只有「抓到」与基线 ok 的、"
                 "照复用判据判过的复用（scripts/mutation/crates-mutation-rows.py 文件头「复用判据」）。不进工作树，别手改。\n")
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由它与登记的路径一起定：prepare_shard 照它们拷，is_copied_into_each_shard 照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
# 拷贝范围里的 cargo 配置目录：.cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS 给 librocksdb-sys 的 bindgen 指头文件；副本在 /tmp 下、往上找不到仓里那一份，
# 验证包常编 librocksdb-sys（prune-server 是它常在的依赖），点名它的测试的行基线必然不绿（弄坏开关 cargoconfignotcopied 拷的时候跳过它）
CARGO_CONFIGURATION_DIRECTORY = ".cargo"
DEFAULT_LONG_ROW_SECONDS = "300"               # 上一次用时不短于它的行先派：GATE_MUTATION_LONG_ROW_SECONDS
STOP_GRACE_SECONDS = 30                        # 被停时对自己起的那一条命令的进程组发 TERM，过这么久还不退再发 KILL
PER_MUTATION_TEMPORARY_DIRECTORY_NAME = "per-mutation-tmp"
# 内存上限的取法、进程树的量法、峰值记录的名字、systemd 大小写法、多机配置与 admission.py 的读入都在 scripts/hosts/task_memory.py，这里同名函数转调它（函数体里 import）。
# 模块级的这几处只在 task_memory.py 里用到：import multi_host_addresses、SIZE_TEXT、SIZE_UNIT_BYTES、SLICE_MEMORY_CAP_KEY、_ADMISSION_MODULE；
# TASK_MEMORY_CAP_VARIABLE、CGROUP_LISTING、CGROUP_ROOT 这里仍用（自证、def 缺省值），task_memory.py 里另有同一个值的一份；同一个值由 selftest_memory_cells 核。
SIZE_TEXT = re.compile(r"^([1-9][0-9]*)([KMGT])$")             # 磁盘余量与编译目录上限的写法：systemd 的写法，1024 进制
SIZE_UNIT_BYTES = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 编译目录占盘（文件头「磁盘余量」「编译目录上限」）：每个工作进程最多占两份编译目录（普通一份、插桩一份），一份涨满按 TARGET_DIRECTORY_CAP 算。
# 默认 64G 是推的：第二台量到的 -w<片号> 普通 39–71G、插桩 30–68G（records/2026-09-16-subagent拆分提案.md 第七十八节追记），取中偏上；
# 本机一份普通的此刻 77G，其中 debug/incremental 13G、debug/deps 30G、release 30G（量的），所以清增量产物不一定清到线下，清完仍超线的只报、不动别的。
# 备用 30G 是推的：留给别的会话与系统，不让工作进程把盘写满（第二台 1.8T 的盘会被写满）。
TARGET_DIRECTORY_CAP_VARIABLE = "GATE_MUTATION_TARGET_DIR_CAP"
DEFAULT_TARGET_DIRECTORY_CAP = "64G"
DISK_FREE_VARIABLE = "GATE_MUTATION_DISK_FREE"          # 外层给的目标盘余量（systemd 写法）；不设就读编译目录所在那块盘
DISK_RESERVE_VARIABLE = "GATE_MUTATION_DISK_RESERVE"
DEFAULT_DISK_RESERVE = "30G"
# 内存上限与每个工作进程的峰值（文件头「内存上限」）：开几个工作进程取满足「n × 那个编译并行度下每个工作进程的峰值 ≤ 内存上限」的最大 n。
# 上限先读统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES（这一次尝试的上限，字节十进制整数），
# 没有读仓根多机配置 multi-host.env 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（用 scripts/gate/admission.py 的 configuration_value，这一台照 multi_host_addresses.resolve，
# 找文件同 test_runs_multi_host_configuration），
# 再没有读自己 scope 的 memory.max（/proc/self/cgroup 的 cgroup v2 那一行接 /sys/fs/cgroup 再接 /memory.max），值是 max 或都读不到就没有这一项（照线程数）。
# memory.max 是有限值时它是硬顶：前两级取到的值比它大就取它（两者取小），前两级都没有取它本身。
# 每个工作进程的峰值由工作进程自己量整棵进程树、按这一格点名的包与编译并行度记进运行记录目录下的 worker-peaks/（文件头「内存上限」的「每个工作进程的峰值」）；
# 只有更低编译并行度的实测或记录读不了取保守值（内存上限本身，只开 1 个）；从没量过的包派活时单独跑一格量出来（探针）；不用写死的默认值。
TASK_MEMORY_CAP_VARIABLE = "TASK_SERVER_MEMORY_CAP_BYTES"
WORKER_PEAKS_RECENT = 256                # 每个包、每个编译并行度只看最新的这么多份记录（一个工作进程一趟一份）：取得宽，几趟只跑几行的点名跑挤不掉整张表那几趟的重读数（推的）
WORKER_PEAK_SAMPLE_SECONDS = 0.2         # 工作进程里的采样线程隔多久量一次整棵进程树
WORKER_PEAK_WRITE_STEP_BYTES = 16 << 20  # 这一趟的最大涨了这么多才重写一次记录：记下的比真峰值至多少这么多
WORKER_PEAK_RECORD_NAME = re.compile(r"[0-9]{20}-[0-9]+-[0-9a-z]{8}")   # worker-peaks/<包>/<编译并行度>/ 下一份记录的名字：起采样的时刻纳秒、进程号、随机字符
WORKER_PEAK_WORKSPACE_SCOPE = "@workspace"   # 一格的 cargo 参数没点名包（-p）时，峰值记在这个名字下；与点名了的包分开记
WORKER_PEAK_PACKAGE_NAME = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_-]*")   # worker-peaks/ 下一级的目录名：cargo 包名（WORKER_PEAK_WORKSPACE_SCOPE 之外）
SLICE_MEMORY_CAP_KEY = "SLICE_MEMORY_CAP_BY_HOST"   # 仓根多机配置 multi-host.env 里各台 slice 的总上限（<地址>=<systemd 写法> 的若干项，取这一台那一项；变异的内存上限直接按根目录设置的 50G 走）
CGROUP_LISTING = "/proc/self/cgroup"
CGROUP_ROOT = "/sys/fs/cgroup"
TARGET_DIRECTORY_COPIES_PER_WORKER = 2                  # 普通一份、插桩一份
INCREMENTAL_DIRECTORY_NAME = "incremental"
# 算数的结论与不算数的结局（文件头「复用判据」记红撤绿那一段）：不算数的照行键写，但不盖同一把行键原有的算数结论
VERDICTS_COUNTED = ("caught", "failure", "invalid")
VERDICTS_UNCOUNTED = ("notrun", "linkfailed")   # linkfailed：替换之后失败在链接、不在编译（verdict_of_run），没判到被改坏的那一处，也不是替换文编不过
# 链接失败的两个记号（verdict_of_run）：rustc 报链接器退非 0 的那一行与 rust-lld 的报错行；输出里没有 error[E<码>]（编译错）才算链接失败
LINK_FAILURE_LINE = "error: linking with"
LINK_FAILURE_LINKER = "rust-lld"
TEST_NAME_FORM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*")
# 点名的测试在本机没跑到被改坏的那一处时打的记号（见文件头「本次未跑」）：`NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`，变异那一趟补 --show-output，libtest 才把成功用例的这一行交出来。
# 记号的认法（整词、一行里每一处各是一截、ANSI 颜色码先去掉）只在 lib_libtest_result_lines.py 的 marker_pieces 写一份：不要求在行首，用例自己没换行就打、接在别的输出后面的同样认；
# 认的名字只有 NOT_RUN_MARK 一个，库认的 NOT_RUN_<原因> 一族里别的名字只用来划截
NOT_RUN_MARK = "NOT_RUN_NO_GPU_ADAPTER"
NOT_RUN_WORD = re.compile(r"[A-Za-z0-9_]+", re.ASCII)   # 库交出的一截从记号起头：开头连着的标识符字符是记号的全名（NOT_RUN_NO_GPU_ADAPTER_X 与 NOT_RUN_NO_GPU_ADAPTER 是两个词）
NOT_RUN_TEST_FIELD = re.compile(r"(?<!\S)test=(\S+)", re.ASCII)   # 一截里点名用例的那一段：前面是空白
NOT_RUN_REASON_FIELD = re.compile(r"(?<!\S)reason=(.*)$", re.ASCII)   # 一截里 reason= 起到这一截末尾
# 点名的测试撞上 GPU 设备丢失（Xid 这一类）时打的一行：行首 `GPU_DEVICE_LOST `，后面是 test=<当前线程名> card=<卡名> reason=<一句>；卡名里可以有空格、test= 不一定是测试名，
# 所以只认行首的记号、后面的字段整段当原文留着、不拆（文件头「本次未跑」）
DEVICE_LOST_LINE = re.compile(r"^GPU_DEVICE_LOST (.*?)[ \t\r]*$", re.M)
# 段外那一行：`--test-threads 1` 时 libtest 先打「test <全名> ... 」不换行，没带输出捕获的线程打的标记行接在它后面
DEVICE_LOST_LINE_OUTSIDE_SECTIONS = re.compile(r"^(?:test \S+ \.\.\. )?GPU_DEVICE_LOST (.*?)[ \t\r]*$", re.M)
LIBTEST_SHOW_OUTPUT = "--show-output"
# 开了就要上卡的 cargo 特性：验证包的 gpu，与产品 GPU 包（singlefs-background-gpu）真显卡对拍的 tests-on-a-real-gpu（Cargo.toml 的 [features] 与 [[test]] required-features 里的那个名字）
GPU_TEST_FEATURES = frozenset({"gpu", "tests-on-a-real-gpu"})
# 带 gpu 特性的格里放开 --test-threads 1 的测试目标（文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：它的测试按显存额度排队，带每台默认重活线程数。
# 放开名单另加验证包登记的只经卡池取卡的测试目标，那几个照被判的根现读（gpu_released_test_targets），不写进这里
GPU_TEST_THREADS_RELEASED_TARGETS = ("crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state",)
# 配了 GPU 而二进制没编进 gpu 特性就判段不跑的那两处选路函数（包、所在文件、函数名）：点名测试的单元闭包够得着其中一个，多机配置开了 GPU 时这一行补 gpu 特性起
#   （文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）。崩溃枚举用例走 crash_state_enumeration 的选路，给定状态引擎走 crash_amplification 的那一处，前者也经它取本次未跑那一句。
#   一样的列表，自证那一格换成自己的小仓里的那一处（GPU_ROUTE_UNITS[0] 整个换掉再换回）
GPU_ROUTE_UNITS = [(("singlefs-verification", "crates/singlefs-verification/src/crash_state_enumeration/judge_route_and_worker_threads.rs", "crash_point_replay_judge_route"),
                    ("singlefs-verification", "crates/singlefs-verification/src/crash_amplification/run_modes_and_cards.rs", "judge_not_run_without_the_gpu_feature"))]
GPU_ROUTE_FEATURE = "gpu"
# 开 GPU 时 GPU 路上不走的 CPU 判器入口（文件头「点名测试够不着被改坏的函数」）：static-check 可达性照它截点名测试的单元闭包。
#   一项 = (来处, 到)，两样都写 (所在文件, 函数名)；来处是 None 的按单元停（那个函数不进截过的闭包），写了来处的只停从来处那个函数连到它的那一条边
#   （同一个函数被测试或别的函数直接调，照连）。一样的列表，自证那一格换成自己的小仓里的那几处（REACHABILITY_GPU_ROUTE_STOPS[0] 整个换掉再换回）
REACHABILITY_GPU_ROUTE_STOPS = [(
    # 崩溃枚举判段的 CPU 那一臂（judge_the_recorded_item 里 CrashPointReplayJudgeRoute::Cpu）
    (("crates/singlefs-verification/src/crash_state_enumeration/judging_stage.rs", "judge_the_recorded_item"),
     ("crates/singlefs-verification/src/crash_amplification/cpu_checking.rs", "check_blocks_where_until_with_threads")),
    # 崩溃放量判段的 CPU 那一臂（run_from_environment_with_observers 里 ComputedOn::Cpu）
    (("crates/singlefs-verification/src/crash_amplification/run_from_the_environment.rs", "run_from_environment_with_observers"),
     ("crates/singlefs-verification/src/crash_amplification/run_from_the_environment.rs", "check_selected_blocks")),
    # 给定状态引擎的 CPU 判器：配了 GPU 只在 GPU 建不出判表（豁免见 REACHABILITY_CPU_FALLBACK_ROOTS）或有 GPU 判不了的状态时才进
    (None, ("crates/singlefs-verification/src/crash_amplification/given_states_engine.rs", "judged_on_the_cpu_here")),
    # 排计划时对基镜像恢复一次，结局只核「不晚于最新根」：改坏恢复照样过，执行了也判不出
    (None, ("crates/singlefs-verification/src/crash_amplification/crash_point_plan.rs", "assert_journal_recovery_of_the_base_image_lands_no_later_than_its_newest_root")),
)]
# GPU 许建不出判表、回到 CPU 判器的镜像：截过的闭包里有这几个函数之一的行豁免（文件头「点名测试够不着被改坏的函数」豁免那一段）；一样的列表，自证那一格整个换掉再换回
REACHABILITY_CPU_FALLBACK_ROOTS = [(("crates/singlefs-verification/tests/common_images_for_the_given_states_engine/mod.rs", "hand_out_allowing_the_gpu_to_refuse_its_tables"),)]
# 测试自己写的那份多机配置关着 GPU：串字面量里有它（文件头「点名测试够不着被改坏的函数」豁免那一段）
REACHABILITY_CPU_CONFIGURATION_TEXT = "ENABLE_GPU=0"
REACHABILITY_HOWTO = ("换成调得到被改坏函数的用例，或删掉这一行（closure：点名测试的单元闭包里根本没有被改坏的单元，开不开 GPU 都调不到；"
                      "gpu-route：开 GPU 时点名测试走 GPU 判段，被改坏的单元只在 GPU 路上不走的 CPU 判器入口后面。换的用例照 UNREACHABLE 行末那一段 cargo 参数（门禁真给这一行用的）核："
                      "直接调 CPU 判器，或自己写一份 ENABLE_GPU=0 的多机配置经 SINGLEFS_MULTI_HOST_CONFIG 交给引擎；换完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check <根> 看那一行不再报，"
                      "再单跑那一行证明点名的测试红（scripts/mutation/prove-red.sh））")
COLLISION_REFUSAL_PREFIX = "行键撞键："   # 撞键拒算那一句的起头（refuse_colliding_row_keys 写、run_stage 与成功句照它认）
BYTES_PER_STAT_BLOCK = 512   # os.stat 的 st_blocks 按 512 字节一块数（POSIX）
EXIT_SELECTION_ONLY = 3      # 只判了选中的行、都抓到：不是整道结论，门禁汇总记失败，不会被当成通过
# run_pool 里接住这两个信号：先停自己起的工作进程、删 shard_root，再退 128 + 信号号（双机分片的驱动停这一份、门禁被打断时都走这一条）
STOP_SIGNALS = (signal.SIGTERM, signal.SIGHUP)
# run_pool 在 $TMPDIR 下建的 shard_root 的名字起头（mutation-shard-run.sh 的自证、本模块的自证都按这个起头找它）
SHARD_ROOT_PREFIX = "singlefs-mutation-replay-"
# 变异复跑的崩溃判定（文件头「变异复跑的崩溃判定」）：变异与记覆盖的插桩跑每次真跑测试各起一份临时 prune-server（基线不起），起法在客户端自证那一份（TestServer、selftest_root）
PRUNE_SERVER_TEMPORARY_SERVICE_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "..", "prune-server", "prune_server_client_selftest.py")
PRUNE_SERVER_BINARY_VARIABLE = "PRUNE_SERVER_BINARY"
PRUNE_SERVER_BUILD_DIRECTORY_NAME = "prune-server-build"   # 没给 PRUNE_SERVER_BINARY 时这一趟编服务的编译目录，放 <shard_root> 底下
PRUNE_SERVER_WRAPPER_NAME = "prune-server-dies-with-its-worker"   # mutation_run_prune_server_binary 写进 <shard_root> 的包装
# 那份包装：先设父进程死亡信号（起它的工作进程一死，内核给它发 SIGKILL），设之前起它的进程已经没了就退，再换成真二进制（REAL_BINARY_PATH 换成它的路径）
PRUNE_SERVER_PARENT_DEATH_WRAPPER = '''#!/usr/bin/env python3
# scripts/mutation/crates-mutation-rows.py 写的包装（它文件头「变异复跑的崩溃判定」）：设父进程死亡信号，再换成真的 prune-server
import ctypes, os, signal, sys
PARENT_DEATH_SIGNAL_OPTION = 1   # prctl 的 PR_SET_PDEATHSIG
parent = os.getppid()
if ctypes.CDLL(None, use_errno=True).prctl(PARENT_DEATH_SIGNAL_OPTION, int(signal.SIGKILL)) != 0:
    sys.exit(f"prctl(PR_SET_PDEATHSIG) 没成：errno {ctypes.get_errno()}")
if os.getppid() != parent:
    sys.exit("起这个服务的工作进程在设父进程死亡信号之前就没了")
os.execv(REAL_BINARY_PATH, [REAL_BINARY_PATH] + sys.argv[1:])
'''
# 源码里一处字符串字面量或注释的起头（找字面量时跳过注释用）
LITERAL_OR_COMMENT_START = re.compile(r'//|/\*|b?r#*"|b?"|\'')
PATH_ATTRIBUTE_BEFORE_LITERAL = re.compile(r"#\s*\[\s*path\s*=\s*$")
PATH_ATTRIBUTE_AFTER_LITERAL = re.compile(r"\s*\]")
# include_str!(…)、include_bytes!(…) 紧跟着的字面量：rustc 照源文件所在目录解析，不照 crate 根
INCLUDE_MACRO_BEFORE_LITERAL = re.compile(r"\binclude_(?:str|bytes)\s*!\s*\(\s*$")
# run_stage 读的环境变量，线程数那两个（工作进程数、编译并行度）除外，它们的变量名在 crates_mutation_rows_memory.py 的 THREAD_SETTING_VARIABLES：
# 读它们的地方都在那一份里，没给就取每台默认重活线程数（自证起每一格之前从调用方的环境里清掉，弄坏开关除外）
STAGE_VARIABLES = ("GATE_MUTATION_ROW_SELECTION", "GATE_MUTATION_LONG_ROW_SECONDS",
                   "GATE_MUTATION_STOP_AFTER_ROWS", "GATE_MUTATION_TARGET_DIR",
                   "GATE_MUTATION_NOT_RUN_RECORD", "GATE_MUTATION_LLVM_TOOLS",
                   "SINGLEFS_REUSE_HOURS", "GATE_MUTATION_KEY_WORKERS",
                   TARGET_DIRECTORY_CAP_VARIABLE, DISK_FREE_VARIABLE, DISK_RESERVE_VARIABLE, TASK_MEMORY_CAP_VARIABLE)

Row = collections.namedtuple("Row", "line_number name path old new arguments expected fields")

FIELD_NAMES = ("变异名", "文件", "原文", "替换文", "cargo test 参数", "必须红的测试名")
REPLACEMENT_FIELD = FIELD_NAMES.index("替换文")   # 六段里只有它可以空：替换文空着就是把原文删掉
# 派活之前（crates-mutation-replay）与静态（code-source-discipline 经 static-check）判的那几样，打印与输出都照这个次序
ROW_PROBLEM_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY")
_ADMISSION_MODULE = []
# 拷贝范围（copied_with_path_dependencies）用 scripts/hosts/lib_repository_copy.py 的 copy_closure，不另抄一份；多机配置只交给真跑的测试（test_runs_multi_host_configuration）
REPOSITORY_COPY_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "lib_repository_copy.py")
# 拷贝范围跟不全时 <根> 是仓副本的那一种出路（编译期、运行期读的仓根文件在副本里没有：副本拷不全）；建法与 lib_repository_copy.py 的 copy_rebuild_howto 同一句
COPY_REBUILD_HOWTO = ("<根> 是仓副本、报的是读的仓根文件「在仓里没有」的，是副本拷不全：仓副本只用 python3 scripts/hosts/minimal-copy.py <仓根> <新副本目录> --with-test-inputs 建"
                      "（拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算），副本里自己的改动搬进新副本再判；不手拷、不手补缺的条目")
REGISTERED_COPY_RANGE_REASON = "登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行的 copy-range"
_REPOSITORY_COPY_MODULE = []
_PRUNE_SERVER_TEMPORARY_SERVICE_MODULE = []
# 重测试目标只许 release（文件头同名一段）：判法只在 lib_release_only_tests.py，这里按路径载入调它
RELEASE_ONLY_TESTS_PATH = os.path.join(SCRIPT_DIRECTORY, "lib_release_only_tests.py")
_RELEASE_ONLY_TESTS_MODULE = []
# run_stage、plan、row-plan、--list-items、--item 与基线切块判重测试目标的登记表从哪一份仓根读：本模块所在那一份（判别力样本的目录里没有登记表）；
# 自证里要另一份登记表的那一格换成自己造的一份再换回。static-check 不读它，读被判那棵树的（static.py 的 judged_registry_root）
RELEASE_REGISTRY_ROOT = [STAGE_REPOSITORY]
RELEASE_HOWTO = ("第五段补上 --release（插在第一个 -- 之前）：穷举、长时间随机、真设备这类用例用优化构建跑（.claude/rules/test-script-discipline.md「编译与算法都要快」）；"
                 "命中哪一样写在每行的依据里，判法在 scripts/mutation/lib_release_only_tests.py；补完单跑 python3 scripts/mutation/crates-mutation-rows.py static-check . 看那一行不再报 RELEASE")
RELEASE_REGISTRY_HOWTO = ("照报错改 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行（写法在那张表的文件头与 scripts/gate/admission.py 的 parse_crash_case）；"
                          "登记表读不了就判不了哪几行是重测试目标，不当不命中")
STATIC_CHECK_REGISTRY_HOWTO = ("static-check 的拷贝范围与重测试目标都读被判那棵树（<根>）里的 .claude/gate.d/stage-inputs.tsv，不读本模块所在那一份仓的：<根> 写有这张表的那一份"
                               "（仓副本照 python3 scripts/hosts/minimal-copy.py <仓根> <新副本目录> --with-test-inputs --with-gate-tables 建）；套补丁的临时树要把这张表连同补丁一起放进去")


def admission_module():
    """scripts/gate/admission.py（git common-dir、登记表里本阶段那一行的拷贝范围、前提都用它的），第一次用到时才读进来；读进来的那一份在 scripts/hosts/task_memory.py，
    内存上限的取法用的也是它，全进程只读一次。"""
    import task_memory
    return task_memory.admission_module()


# 弄坏开关从这两个变量里认（逗号分隔可给几个）：GATE_MUTATION_BREAK 是阶段脚本一路传下来的那个，CRATES_MUTATION_ROWS_BREAK 是直接起本模块自证时用的同义名
BREAK_VARIABLES = ("GATE_MUTATION_BREAK", "CRATES_MUTATION_ROWS_BREAK")


def break_is(environment, switch_name):
    return any(switch_name in environment.get(variable, "").split(",") for variable in BREAK_VARIABLES)



def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


def cargo_debug_overrides(environment):
    """这一道起的 cargo 的构建档覆盖（文件头「调试信息与 release 增量」）：{CARGO_PROFILE_<档>_DEBUG: 取值, CARGO_PROFILE_RELEASE_INCREMENTAL: 取值}，
    起 cargo 的环境照它顶掉调用方的同名变量。弄坏开关 debuginfo-kept：不交 DEBUG 那四项（调试信息照 Cargo.toml 与调用方的环境开着）；
    release-incremental-unset：不交 CARGO_PROFILE_RELEASE_INCREMENTAL（release 档照 Cargo.toml 与调用方的环境）。"""
    overrides = {} if break_is(environment, "debuginfo-kept") else {
        f"CARGO_PROFILE_{profile}_DEBUG": CARGO_DEBUG_SETTING[0] for profile in CARGO_DEBUG_PROFILES}
    if not break_is(environment, "release-incremental-unset"):
        overrides["CARGO_PROFILE_RELEASE_INCREMENTAL"] = CARGO_RELEASE_INCREMENTAL_SETTING[0]
    return overrides


def baseline_cargo_environment(environment, release_incremental_of_the_caller):
    """基线起的 cargo 的环境（文件头「基线连调用方的判定库服务」）：不带门禁加的 CARGO_PROFILE_RELEASE_INCREMENTAL 覆盖，release 档的增量编译照调用方的环境与仓根 Cargo.toml
    （release_incremental_of_the_caller：调用方自己设的值，没设交 None，没设就不放）。codegen-units 没写时它的生效值跟增量编译走（开着 256、关着 16），进崩溃判定的版本，
    基线的版本因此要与门禁 crash-point-replay 的 cargo test --release 同一把、才读得到真库里那一趟判过的块。调试信息那四项照旧（调试信息不进崩溃判定的版本）。
    弄坏开关 baseline-release-incremental-kept：基线也带这一项覆盖。"""
    if break_is(environment, "baseline-release-incremental-kept"):
        return dict(environment)
    baseline = {name: value for name, value in environment.items() if name != "CARGO_PROFILE_RELEASE_INCREMENTAL"}
    if release_incremental_of_the_caller is not None:
        baseline["CARGO_PROFILE_RELEASE_INCREMENTAL"] = release_incremental_of_the_caller
    return baseline


BUILD_ENVIRONMENT_BREAKS = ("build-environment-keyed", "profile-keyed", "build-jobs-keyed", "toolchain-version-keyed")


def build_environment_lines_for_break(root, environment):
    """构建环境的行（只给弄坏开关用，文件头「行键」）：BUILD_ENVIRONMENT_BREAKS 里任一个打开时是 lib_mutation_units.py 的 toolchain_lines
    （编译器与链接器相关的环境变量、cargo 配置、清单的 lints；profile-keyed、build-jobs-keyed、toolchain-version-keyed 各让它另收编译档、线程数、rustc 与 cargo 的版本），
    平常交空表：构建环境不进行键。cargo 配置与清单读不懂抛 MutationUnitsError。"""
    if not any(break_is(environment, name) for name in BUILD_ENVIRONMENT_BREAKS):
        return []
    return mutation_units.toolchain_lines(root, environment)


# ── 变异表 ────────────────────────────────────────────────────────────────────

def unescape(text):
    return text.replace("\\n", "\n")


def malformed_reason(fields, environment):
    """一行拆出来的几段不成形的原因，成形回 None。成形 = 六段；替换文可以空（删掉原文：改坏时原文换成空串），其余五段一段都不许空。
    弄坏开关 emptyreplacementrefused（只给证红用）让替换文空着也算不成形。"""
    if len(fields) != len(FIELD_NAMES):
        return f"不是六段（{len(fields)} 段）"
    empty = [FIELD_NAMES[index] for index, field in enumerate(fields)
             if field == "" and (index != REPLACEMENT_FIELD or break_is(environment, "emptyreplacementrefused"))]
    return "、".join(f"「{name}」" for name in empty) + "空着" if empty else None


def read_table(path, environment=None):
    """(成形的行, [(行号, 变异名, 为什么不成形)])。成形照 malformed_reason：
    crates-mutation-replay 与 static-check（门禁 code-source-discipline）都照这一份读，没有第二份判法。environment 不给就读本进程的环境（只看弄坏开关）。"""
    environment = os.environ if environment is None else environment
    rows, malformed = [], []
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            reason = malformed_reason(fields, environment)
            if reason:
                malformed.append((line_number, fields[0] or "（变异名空着）", reason))
                continue
            rows.append(Row(line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5], tuple(fields)))
    return rows, malformed


# ── 拷贝范围（crates-mutation-replay 拷进每片的、code-source-discipline 静态判的，同一份判法）──────────────────────

def copied_paths(stage_repository=STAGE_REPOSITORY):
    """stage_repository 那一份仓根里登记表 crates-mutation-replay 那一行登记的路径（目录去掉末尾的斜杠）；不给取本模块所在那一份仓（run_stage 一路），
    static-check 给被判的那棵树。登记表不在、读不了、没有那一行抛 admission 的 RegistrationError。"""
    admission = admission_module()
    input_paths, _rows = admission.resolve_input_paths(admission.read_registration_rows(stage_repository), STAGE)
    return [input_path.strip().rstrip("/") for input_path in input_paths if input_path.strip()]


def under_temporary_prune_server(environment):
    """environment 连的是 --with-temporary-service 那样起的临时 prune-server：判法同客户端的 selftest_service_problem（临时服务的标记设了、与 socket 相同、
    TCP 地址设了、socket 在），判的是给的这一份环境（客户端那一份只看本进程的 os.environ）。"""
    client = mutation_units.Library.client()
    marked = environment.get(client.TEMPORARY_SERVICE_VARIABLE, "")
    return (bool(marked) and marked == environment.get(client.SOCKET_VARIABLE, "") and bool(environment.get(client.TCP_ADDRESS_VARIABLE, ""))
            and os.path.exists(marked))


Components = collections.namedtuple("Components", "row_key refusal mutated test_unit test_full_name test_closure switches baseline_key coverage_id")


Decision = collections.namedtuple("Decision", "reuse column detail")


# ── 基线的批、派活的次序、libtest 的结果行 ────────────────────────────────────

def split_at_double_dash(arguments):
    words = arguments.split()
    if "--" in words:
        index = words.index("--")
        return words[:index], words[index + 1:]
    return words, []


# ── 工作进程 ──────────────────────────────────────────────────────────────────

WORKER_STATE = {}


def format_size(byte_count):
    """字节数写成 GiB / MiB / KiB / B 的一位小数（scripts/hosts/task_memory.py 的 format_size）。"""
    import task_memory
    return task_memory.format_size(byte_count)


def task_memory_cap_bytes(environment, cgroup_listing=CGROUP_LISTING, cgroup_root=CGROUP_ROOT, configuration_path=None):
    """这一条命令拿到的内存上限（文件头「内存上限」）：(字节或 None, 说明或没有这一项的原因)。取法、次序与说明的写法在 scripts/hosts/task_memory.py 的 task_memory_cap_bytes，
    这里只把弄坏开关交进去：memory-max-not-hard-cap（①② 取到了就不拿 memory.max 当硬顶）；memory-cap-ignored 在 memory_bound_for。
    cgroup_listing、cgroup_root 的缺省值是本文件的 CGROUP_LISTING、CGROUP_ROOT，task_memory.py 里另有同一个值的一份。"""
    import task_memory
    return task_memory.task_memory_cap_bytes(environment, cgroup_listing, cgroup_root, configuration_path,
                                             hard_cap_ignored=break_is(environment, "memory-max-not-hard-cap"))


def process_usage_bytes(pid):
    """一个进程此刻的用量：/proc/<pid>/status 的 RssAnon + RssShmem + VmPTE（scripts/hosts/task_memory.py 的 process_usage_bytes）。读不到（进程刚退出）交 None。"""
    import task_memory
    return task_memory.process_usage_bytes(pid)


def output_progress_seconds(environment):
    """边跑边报的间隔（文件头「边跑边收、边跑边报」）：GATE_MUTATION_PROGRESS_SECONDS，没设 60 秒。写法由 run_stage 开头核过。"""
    text = environment.get("GATE_MUTATION_PROGRESS_SECONDS", "").strip()
    return int(text) if text else 60


# ── 整道 ──────────────────────────────────────────────────────────────────────

# gate-overlap:copy-kept scripts/mutation/mutate.sh 那边的 worker_count_and_head_line 照这里的「各项取最小、至少 1、头一项是每台默认重活线程数」写了一份 bash：两边的项不同（这里 GATE_MUTATION_WORKERS 顶掉登记值那一项、第二项是要现跑的条数、第三项是磁盘余量；那边没有第三项，MUTATE_JOBS 只能压小、第二项是表里的条数、另算每个 cargo 的编译并行度），派活一边是本进程的进程池、一边是 bash 子 shell 按目录认领，共用的只剩一个取最小值，抽成共用要让 mutate.sh 每趟多起一个 python 只为算一个数、两边头一行的字也不同
def bytes_of_size(text):
    """systemd 写法的大小（正整数加 K / M / G / T，1024 进制）→ 字节；别的写法交 None（scripts/hosts/task_memory.py 的 bytes_of_size）。"""
    import task_memory
    return task_memory.bytes_of_size(text)


CLOSURE_REFUSED_HOWTO = [
    "上面每处点名了测试和变异表的行：测试带 #[cfg(feature = \"…\")] 的，给那几行的 cargo 参数补上编它要的特性（例 --features gpu，写法照同一张表里别的带 gpu 的行），",
    "或点名一条在现有特性下编得到的测试；「没有叫 … 的包」这一类就改那一行的 -p。改完单跑 python3 scripts/mutation/crates-mutation-rows.py plan <根> 看还有没有别的行（键算不出来，整张表一行都判不了）。"]


def print_rejection(headline, details, howto_lines):
    say(f"  ✗ {headline}")
    for detail in details:
        say(f"      {detail}")
    for index, howto_line in enumerate(howto_lines):
        say(("     → 怎么办：" if index == 0 else "       ") + howto_line)


def merge_progress_variable():
    """合并那一步分步进度的范围名放在环境里的哪个变量：row-plan --progress 设「合并」，合并那一步里的 run_stage 设「合并·核账」，不手设；没设就是关着的。
    变量名写在函数体里、不做模块级常量。"""
    return "GATE_MUTATION_MERGE_PROGRESS_SCOPE"


def merge_step_open(environment, step, total):
    """合并那一步分步打进度（scripts/mutation/mutation-shard-run.sh 文件头「三步」merge），打 stderr。开一步：打「  · <范围>·<步名>：开始[，共 N 行]」，交这一步的账（dict）；
    环境里没开分步进度交 None，merge_step_part、merge_step_row_done、merge_step_close 认 None、什么都不打。step 是 None 时开的是整个范围（行首只有范围名）。
    total 是这一步要逐行做的行数，不逐行做的写 None。
    GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS 只给自证：逐行的步每过一行停这么多秒（假的慢读库），只在临时 prune-server 底下认。"""
    scope = environment.get(merge_progress_variable(), "")
    if not scope:
        return None
    try:
        interval = output_progress_seconds(environment)
    except ValueError:   # 写法由 run_stage 开头核（不是正整数它拒），这里取没设时的间隔
        interval = output_progress_seconds({})
    pause = environment.get("GATE_MUTATION_PROGRESS_ROW_PAUSE_SECONDS", "").strip()
    row_pause = float(pause) if re.fullmatch(r"[0-9]+(\.[0-9]+)?", pause) and under_temporary_prune_server(environment) else 0.0
    label = scope if step is None else f"{scope}·{step}"
    now = time.monotonic()
    account = dict(label=label, total=total, started=now, printed=now, interval=interval, row_pause=row_pause)
    note(f"  · {label}：开始" + ("" if total is None else f"，共 {total} 行"))
    return account


def merge_step_part(account, text):
    """这一步里的一段算完（点名测试的闭包这类不逐行的段）：打一行，带这一步已用几秒。"""
    if account is not None:
        note(f"  · {account['label']}：{text}，这一步已用 {int(time.monotonic() - account['started'])} 秒")


def merge_step_row_done(account, done):
    """逐行的步里第 done 行做完了：离上一行打出来过了固定间隔（output_progress_seconds：GATE_MUTATION_PROGRESS_SECONDS，没设 60 秒）就打一行「在第 N 行、共 M 行」，
    行首是统一任务服务认的心跳前缀（task_server_lib.py 的 HEARTBEAT_LOG_MARK：这种行不算日志在涨，命令真卡死照样判得出）；开始与结束行不带它。
    心跳前缀是文本字面量，只写在这一处。"""
    if account is None:
        return
    if account["row_pause"]:
        time.sleep(account["row_pause"])
    now = time.monotonic()
    if now - account["printed"] >= account["interval"]:
        account["printed"] = now
        note(f"TASK_SERVER_HEARTBEAT {account['label']}：在第 {done} 行、共 {account['total']} 行，这一步已用 {int(now - account['started'])} 秒")


def merge_step_close(account, detail):
    """收一步：打「  · <范围>·<步名>：完，用了 N 秒，<细节>」。"""
    if account is not None:
        note(f"  · {account['label']}：完，用了 {int(time.monotonic() - account['started'])} 秒，{detail}")


def inserted_before_double_dash(arguments, new_words):
    """cargo 参数里补几个词：插在第一个 `--` 之前（libtest 的参数不动），没有 `--` 接在末尾。"""
    words = arguments.split()
    index = words.index("--") if "--" in words else len(words)
    return " ".join(words[:index] + list(new_words) + words[index:])


_REACHABILITY_PARALLEL = {}


ITEM_PLAN_PREFIX = "ITEM_PLAN"
ITEM_DONE_PREFIX = "ITEM_DONE"
ITEM_EXIT_BY_VERDICT = {"caught": 0, "notrun": 77}   # 别的档（没红 failure、无效 invalid、基线不绿 baseline、锚点失效 anchor、没判到 unrun）都是 1


# ── 自证 ──────────────────────────────────────────────────────────────────────

FAKE_CARGO = r'''#!/usr/bin/env python3
# 自证用的假 cargo：-V 打一行版本（FAKE_CARGO_VERSION 设了打它）；test 读 crates/tiny/src/lib.rs 里「// TEST <名字> NEEDS <原文>」「// TEST <名字> READS <相对 crate 根的路径>」，
# 原文在、文件在就 ok，不在就 FAILED；NEEDS 那一句末尾可以跟「 RUNS <函数名>[,<函数名>…]」，说这条用例执行哪几个函数（不跟就是定义行里有那段原文的函数）；
# 「// TEST <名字> FILES <相对 crate 根的路径>[,<路径>…] <原文> RUNS …」：原文在列出的每一份文件里都在才 ok（撞键拒算那一格改的是 lib.rs 以外的两份文件）；
# 过滤词照 libtest：`--` 之后带 --exact 时要与全名 tests::<名字> 逐字相同，不带是全名的子串；名字里带 :: 的就是全名（样本里两个模块各有一条同名用例）；
# 「// COMPILE-ERROR-IF <原文>」：源码里（这一行自己除外）有这段原文，test 就照 rustc 编不过的样子打 error[E0425] 与 could not compile、退 101（样本里替换文编不过的变异）；
# 「// TEST <名字> NOTRUN <一句>」照 GPU 对拍用例在没卡的机器上：一律 ok，带 --show-output 时照 libtest 的样子
# 在末尾交出它打的 NOT_RUN_NO_GPU_ADAPTER 那一行；
# 「// TEST <名字> LOST <原文> RUNS …」：NEEDS 的判法（原文在 ok、不在 FAILED），FAILED 时照撞上 GPU 设备丢失的用例在 failures 一段里交出自己那一段输出，
# 里面有一行 GPU_DEVICE_LOST test=main card=样本 GPU reason=样本：设备丢失（test= 不是全名）；
# 「// TEST <名字> LOSTINANOTHERCASE <原文> RUNS …」同上判红绿，FAILED 时它自己那一段只有断言的话、那一行 GPU_DEVICE_LOST 在 tests::some_other_gpu_case 的那一段里；
# 「// TEST <名字> LOSTOUTSIDE <原文> RUNS …」同上判红绿，FAILED 时它自己那一段只有断言，那一行 GPU_DEVICE_LOST（test=wgpu-callback）在实时打印的部分、结果行之前，不在任何一段里；
# $FAKE_CARGO_EDIT_SOURCE（「<原文>\t<改成>」）设了，test 跑完用例之后把当前目录那份 lib.rs 里恰好一次的原文改掉；
# 每起一次往 $FAKE_CARGO_LOG 追加一行「<参数>\t<lib.rs 的 sha256 前 12 位>」；
# $FAKE_CARGO_ENVIRONMENT_LOG 设了，每起一次 test 再往它追加一行 JSON：收到的 *_THREADS、SINGLEFS_THREAD_CAP、CARGO_BUILD_JOBS、CARGO_TARGET_DIR、
# 构建档覆盖（CARGO_PROFILE_*），外加键 ARGUMENTS（这一次的参数，认得出是不是插桩编译那一趟）与 RUN_NAME（TMPDIR 的名字：baseline-<行号>-… 是基线、
# line-<行号>-… 是变异，认得出是哪一趟）；调试信息、release 增量那几格读它。
# $FAKE_CARGO_SERVICE_LOG 设了，不带 --no-run 的 test 照收到的连哪一处往 prune-server 写一条（键 $FAKE_CARGO_CRASH_VERDICT_PREFIX 接 TMPDIR 的名字，
# 客户端从 $FAKE_CARGO_CLIENT_DIRECTORY 载入），再往它追加一行「<socket><TAB><临时服务的标记><TAB><TMPDIR 的名字>」：变异复跑的崩溃判定那一格读它。
# $FAKE_CARGO_BARRIER_DIR 设了，不带 --no-run 且源码不是 $FAKE_CARGO_BARRIER_PRISTINE（lib.rs 的 sha256 前 12 位）的那一次（改坏了的变异那一趟）在里面按
# CARGO_TARGET_DIR 记一个文件，等到里面有两个（两个工作进程都到了）或 20 秒：造「两个工作进程同时在跑」。
# $FAKE_CARGO_MUTATION_SLEEP 设了，不带 --no-run 且源码不是 $FAKE_CARGO_BARRIER_PRISTINE 的那一次（改坏了的变异那一趟，基线与覆盖不睡）睡这么多秒：
# 造「每一项自己的挂钟」（按项机读行 ITEM_DONE 的 seconds 那一格读它）。
# $FAKE_CARGO_SPAN_LOG 设了，test 每起一次往里记「start<TAB><TMPDIR 的名字><TAB><时刻>」，睡 $FAKE_CARGO_SPAN_SLEEP 秒（没设睡 0.6），退出时记一行 end（O_APPEND，一次写一行）：
# 派活那一段那几格按 TMPDIR 的名字（baseline-<行号>-…、line-<行号>-…）认是哪一格、几格同时在跑。
# $FAKE_CARGO_FORCED_OUTCOME 是 notrun 时每条用例都 ok，带 --show-output 时照没卡的机器交出一行 NOT_RUN_NO_GPU_ADAPTER（不算数的结局那几格造本次未跑）。
# 测试那一趟的 `--` 之后 --exact 出现两次，照 libtest 报 error: Option 'exact' given more than once、退 101。
# --test-threads（=N 或分开写）出现两次同样报 Option 'test-threads' given more than once、退 101；分开写时后面的 N 是它的值，不当过滤词。
# 开头的 --config <值> 跳过。test --no-run（插桩编译、先编一份依赖）：每个 -p 点名的包（不点名是 tiny）在 $CARGO_TARGET_DIR/debug/deps/ 下写一个假的测试可执行文件
# （内容是 JSON：源码副本的根与这个包 src/ 下的 .rs）与它的 dep-info，打一行 compiler-artifact 消息；
# 不带 --no-run 的 test（基线与变异直接跑测试，真 cargo 也先编进编译目录）：$CARGO_TARGET_DIR 设了，同样往里写 debug/deps/ 下每个 -p 点名的包一份 lib<包>-<16 个 0>.rlib，
# $FAKE_CARGO_INCREMENTAL_BYTES 设了再写一份增量产物（编译目录上限、停派、占盘记录那几格量的是这份）；
# 设了 LLVM_PROFILE_FILE 的 test 把这一趟跑到的用例执行的函数写进它指的那份 .profraw（%p 换成进程号、%m 换成 0，内容是 JSON）。
# test --no-run 还照真插桩编译里构建脚本的样子写一份剖析文件：有 LLVM_PROFILE_FILE 写它指的，没有写当前目录的 default_%m_%p.profraw；
# 写了的路径连同当前目录追加进 $FAKE_CARGO_PROFILE_LOG（设了才记）。
# `--` 之前的 --features / -F 里有 gpu 的（自证拿它当编 librocksdb-sys 的那几行；真仓里 验证包常编它），照真 cargo 编 librocksdb-sys 那样要 BINDGEN_EXTRA_CLANG_ARGS：环境里没有、
# 从当前目录往上每一层的 .cargo/config.toml（或 .cargo/config）的 [env] 里也没有，就照它的构建脚本报 stdbool.h 找不到、退 101，一个用例都不跑
import hashlib, json, os, re, sys, tomllib
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:
    arguments = arguments[2:]
if arguments[:1] in (["-V"], ["--version"]):
    print(os.environ.get("FAKE_CARGO_VERSION") or "cargo 0.0.0 (crates-mutation-rows selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
if os.environ.get("FAKE_CARGO_SPAN_LOG"):
    import atexit, time
    span_name = os.path.basename(os.environ.get("TMPDIR", ""))

    def write_span(kind):
        with open(os.environ["FAKE_CARGO_SPAN_LOG"], "a", encoding="utf-8") as span_log:
            span_log.write(f"{kind}\t{span_name}\t{time.time():.6f}\n")
    write_span("start")
    atexit.register(write_span, "end")
    time.sleep(float(os.environ.get("FAKE_CARGO_SPAN_SLEEP") or "0.6"))
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(arguments[1:]) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
if os.environ.get("FAKE_CARGO_ENVIRONMENT_LOG"):
    # 设了才记：每起一次 test 往里追加一行 JSON，收到的线程数变量、编译并行度与编译目录
    names = ("RUST_TEST_THREADS", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR")
    with open(os.environ["FAKE_CARGO_ENVIRONMENT_LOG"], "a", encoding="utf-8") as environment_log:
        recorded = {name: value for name, value in os.environ.items()
                    if name in names or name.endswith("_THREADS") or name == "SINGLEFS_THREAD_CAP" or name.startswith("CARGO_PROFILE_")}
        recorded["ARGUMENTS"] = " ".join(arguments[1:])
        recorded["RUN_NAME"] = os.path.basename(os.environ.get("TMPDIR", ""))
        environment_log.write(json.dumps(recorded, sort_keys=True) + "\n")
if os.environ.get("FAKE_CARGO_SERVICE_LOG") and "--no-run" not in arguments:
    # 设了才做：真跑测试的这一次照收到的连哪一处往服务里写一条（演用例里的引擎写崩溃判定：键 $FAKE_CARGO_CRASH_VERDICT_PREFIX 接 TMPDIR 的名字），
    # 写成之后往 $FAKE_CARGO_SERVICE_LOG 追加一行「<SINGLEFS_PRUNE_SERVER_SOCKET><TAB><PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET><TAB><TMPDIR 的名字>」；
    # 客户端从 $FAKE_CARGO_CLIENT_DIRECTORY 载入，连不上照客户端退 75
    sys.path.insert(0, os.environ["FAKE_CARGO_CLIENT_DIRECTORY"])
    import prune_server_client
    run_name = os.path.basename(os.environ.get("TMPDIR", ""))
    prune_server_client.process_client().write_many([((os.environ["FAKE_CARGO_CRASH_VERDICT_PREFIX"] + run_name).encode(), b"red")])
    with open(os.environ["FAKE_CARGO_SERVICE_LOG"], "a", encoding="utf-8") as service_log:
        service_log.write(f"{os.environ.get('SINGLEFS_PRUNE_SERVER_SOCKET', '')}\t{os.environ.get('PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET', '')}\t{run_name}\n")
if os.path.isfile("Cargo.lock"):
    # 外部依赖（Cargo.lock 里带 source 的包）：这份编译目录里缺它们的产物就照真 cargo 的样子「编」一遍（<档>/deps 下的 rlib 与 .d、.fingerprint、build），
    # 往 $FAKE_CARGO_DEPENDENCY_LOG 记一行编译目录（设了才记）；先编一份依赖那几格数它
    profile_directory = os.path.join(os.environ.get("CARGO_TARGET_DIR", "target"), "debug")
    missing = []
    for package in tomllib.load(open("Cargo.lock", "rb")).get("package", []):
        if "source" in package:
            unit = hashlib.sha256((package["name"] + package["version"]).encode()).hexdigest()[:16]
            rlib = os.path.join(profile_directory, "deps", f"lib{package['name'].replace('-', '_')}-{unit}.rlib")
            if not os.path.isfile(rlib):
                missing.append((package, unit, rlib))
    for package, unit, rlib in missing:
        crate = package["name"].replace("-", "_")
        registry_source = f"/crates-mutation-rows-selftest-registry/{package['name']}-{package['version']}/src/lib.rs"
        os.makedirs(os.path.dirname(rlib), exist_ok=True)
        with open(rlib, "w", encoding="utf-8") as handle:
            handle.write(package["name"])
        with open(os.path.join(profile_directory, "deps", f"{crate}-{unit}.d"), "w", encoding="utf-8") as handle:
            handle.write(f"{rlib}: {registry_source}\n\n{registry_source}:\n")
        for kind, name, file_name in ((".fingerprint", f"{package['name']}-{unit}", f"lib-{crate}"), ("build", f"{package['name']}-{unit[::-1]}", os.path.join("out", "generated.rs"))):
            os.makedirs(os.path.dirname(os.path.join(profile_directory, kind, name, file_name)), exist_ok=True)
            with open(os.path.join(profile_directory, kind, name, file_name), "w", encoding="utf-8") as handle:
                handle.write(unit + "\n")
    if missing and os.environ.get("FAKE_CARGO_DEPENDENCY_LOG"):
        with open(os.environ["FAKE_CARGO_DEPENDENCY_LOG"], "a", encoding="utf-8") as log:
            log.write(os.environ.get("CARGO_TARGET_DIR", "target") + "\n")
if os.environ.get("FAKE_CARGO_BARRIER_DIR") and "--no-run" not in arguments and hashlib.sha256(source.encode()).hexdigest()[:12] != os.environ.get("FAKE_CARGO_BARRIER_PRISTINE"):
    import time
    barrier = os.environ["FAKE_CARGO_BARRIER_DIR"]
    os.makedirs(barrier, exist_ok=True)
    open(os.path.join(barrier, hashlib.sha256(os.environ["CARGO_TARGET_DIR"].encode()).hexdigest()[:12]), "w").close()
    barrier_deadline = time.monotonic() + 20
    while len(os.listdir(barrier)) < 2 and time.monotonic() < barrier_deadline:
        time.sleep(0.05)
if os.environ.get("FAKE_CARGO_MUTATION_SLEEP") and "--no-run" not in arguments and hashlib.sha256(source.encode()).hexdigest()[:12] != os.environ.get("FAKE_CARGO_BARRIER_PRISTINE"):
    import time
    time.sleep(float(os.environ["FAKE_CARGO_MUTATION_SLEEP"]))
if os.environ.get("FAKE_CARGO_HANG_FILE"):
    # 自证「被 TERM 停」那一格：起来记一笔（先写成临时文件再改名，读的一头看到文件就是写全的进程号），等到放行、父进程（工作进程）没了或 120 秒，记一笔「退了」再退 101
    import time
    parent = os.getppid()
    with open(os.environ["FAKE_CARGO_HANG_FILE"] + ".part", "w", encoding="utf-8") as handle:
        handle.write(str(parent))
    os.rename(os.environ["FAKE_CARGO_HANG_FILE"] + ".part", os.environ["FAKE_CARGO_HANG_FILE"])
    deadline = time.monotonic() + 120
    while time.monotonic() < deadline and os.getppid() == parent and not os.path.exists(os.environ["FAKE_CARGO_RELEASE_FILE"]):
        time.sleep(0.05)
    open(os.environ["FAKE_CARGO_DONE_FILE"], "w").close()
    sys.exit(101)
before = arguments[1:arguments.index("--")] if "--" in arguments else arguments[1:]
features = set()
for index, word in enumerate(before):
    if word in ("--features", "-F") and index + 1 < len(before):
        word = "--features=" + before[index + 1]
    if word.startswith("--features="):
        features |= {feature.split("/")[-1] for feature in word[len("--features="):].replace(",", " ").split()}


def bindgen_include_given():
    if os.environ.get("BINDGEN_EXTRA_CLANG_ARGS"):
        return True
    directory = os.getcwd()
    while True:
        for name in ("config.toml", "config"):
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path):
                with open(path, "rb") as handle:
                    if "BINDGEN_EXTRA_CLANG_ARGS" in tomllib.load(handle).get("env", {}):
                        return True
        parent = os.path.dirname(directory)
        if parent == directory:
            return False
        directory = parent


if "gpu" in features and not bindgen_include_given():
    print("error: failed to run custom build command for `librocksdb-sys v0.0.0 (crates-mutation-rows selftest)`", file=sys.stderr)
    print("  rocksdb/include/rocksdb/c.h:65:10: fatal error: 'stdbool.h' file not found", file=sys.stderr)
    sys.exit(101)
if "--no-run" not in before and os.environ.get("CARGO_TARGET_DIR"):
    # 真 cargo test 不带 --no-run 也先把要跑的包编进 $CARGO_TARGET_DIR 再跑（基线与变异那一趟不另起 --no-run 先编）：照样往这份编译目录写增量产物与 deps 里的一份产物，
    # 编译目录上限、停派、占盘记录那几格才有东西可量；CARGO_TARGET_DIR 没设（真 cargo 会写当前目录的 target）的调用不写，免得往源码副本里落东西
    run_target = os.environ["CARGO_TARGET_DIR"]
    os.makedirs(os.path.join(run_target, "debug", "deps"), exist_ok=True)
    if os.environ.get("FAKE_CARGO_INCREMENTAL_BYTES"):
        run_session = os.path.join(run_target, "debug", "incremental", "session-" + str(os.getpid()))
        os.makedirs(run_session, exist_ok=True)
        with open(os.path.join(run_session, "dep-graph.bin"), "wb") as handle:
            handle.write(os.urandom(int(os.environ["FAKE_CARGO_INCREMENTAL_BYTES"])))
    for run_package in [before[index + 1] for index, word in enumerate(before) if word == "-p" and index + 1 < len(before)] or ["tiny"]:
        with open(os.path.join(run_target, "debug", "deps", f"lib{run_package}-{'0' * 16}.rlib"), "w", encoding="utf-8") as handle:
            handle.write(run_package)
if "--no-run" in before:
    packages = [before[index + 1] for index, word in enumerate(before) if word == "-p" and index + 1 < len(before)] or ["tiny"]
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    # 增量产物（rustc 每编一次改坏的副本往 incremental 里加的会话目录）：FAKE_CARGO_INCREMENTAL_BYTES 设了，每次编往里写这么多字节（编译目录上限那一格读；不带 --no-run 的 test 在上面同样写一份）
    if os.environ.get("FAKE_CARGO_INCREMENTAL_BYTES"):
        session = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "incremental", "session-" + str(os.getpid()))
        os.makedirs(session, exist_ok=True)
        with open(os.path.join(session, "dep-graph.bin"), "wb") as handle:
            handle.write(os.urandom(int(os.environ["FAKE_CARGO_INCREMENTAL_BYTES"])))
    for package in packages:
        crate = os.path.join(os.getcwd(), "crates", package)
        sources = sorted(os.path.relpath(os.path.join(directory, name), os.getcwd()) for directory, _names, names in os.walk(os.path.join(crate, "src"))
                         for name in names if name.endswith(".rs"))
        executable = os.path.join(deps, f"{package}-0000000000000000")
        # fresh 照 cargo 按修改时刻判的样子：产物在、而且不比这个包底下最新的文件旧（先编一份依赖那一格核摸过的包有没有重编读它）
        newest_source = max((os.stat(os.path.join(directory, name)).st_mtime_ns for directory, _names, names in os.walk(crate) for name in names), default=0)
        fresh = os.path.isfile(executable) and os.stat(executable).st_mtime_ns >= newest_source
        with open(executable, "w", encoding="utf-8") as handle:
            json.dump({"root": os.getcwd(), "sources": sources}, handle)
        with open(executable + ".d", "w", encoding="utf-8") as handle:
            handle.write(f"{executable}: {' '.join(sources)}\n\n")
        print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(crate, "Cargo.toml"), "target": {"kind": ["lib"], "name": package},
                          "profile": {"test": True}, "filenames": [executable], "executable": executable, "fresh": fresh}))
    # 剖析文件只在插桩编译时写（参数或 RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS 里带 -Cinstrument-coverage）：先编一份依赖那一趟不插桩，不写
    instrumented = any("instrument-coverage" in word for word in sys.argv[1:]) or any(
        "instrument-coverage" in os.environ.get(name, "") for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"))
    if instrumented:
        stray = os.path.abspath((os.environ.get("LLVM_PROFILE_FILE") or "default_%m_%p.profraw").replace("%p", str(os.getpid())).replace("%m", "0"))
        os.makedirs(os.path.dirname(stray), exist_ok=True)
        with open(stray, "w", encoding="utf-8") as handle:
            handle.write("{}")
        if os.environ.get("FAKE_CARGO_PROFILE_LOG"):
            with open(os.environ["FAKE_CARGO_PROFILE_LOG"], "a", encoding="utf-8") as log:
                log.write(os.getcwd() + "\t" + stray + "\n")
    sys.exit(0)
for compile_line in source.splitlines():
    if compile_line.startswith("// COMPILE-ERROR-IF ") and compile_line[len("// COMPILE-ERROR-IF "):] in source.replace(compile_line, ""):
        print("error[E0425]: cannot find value in this scope", file=sys.stderr)
        print("error: could not compile `tiny` (lib test) due to 1 previous error", file=sys.stderr)
        sys.exit(101)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
if after.count("--exact") > 1:
    print("error: Option 'exact' given more than once", file=sys.stderr)
    sys.exit(101)
# --test-threads 照 libtest：只能出现一次（--test-threads=N 或分开写 --test-threads N），分开写时 N 是它的值、不是过滤词
if sum(1 for word in after if word == "--test-threads" or word.startswith("--test-threads=")) > 1:
    print("error: Option 'test-threads' given more than once", file=sys.stderr)
    sys.exit(101)
if "--test-threads" in after:
    index = after.index("--test-threads")
    after = after[:index] + after[index + 2:]
filters = [word for word in after if not word.startswith("-")]
exact = "--exact" in after
failed = False
shown, executed, lost_sections = [], set(), []
definitions = {match.group(1): match.group(0) for match in re.finditer(r"(?m)^\s*(?:pub\s+)?(?:const\s+)?fn\s+(\w+).*$", source)}
for line in source.splitlines():
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, how, what = words[2], words[3], words[4]
    full_name = name if "::" in name else "tests::" + name
    if filters and not any((word == full_name) if exact else (word in full_name) for word in filters):
        continue
    if how == "NOTRUN" or os.environ.get("FAKE_CARGO_FORCED_OUTCOME") == "notrun":
        print(f"test {full_name} ... ok")
        if "--show-output" in after:
            shown.append(f"---- {full_name} stdout ----\nNOT_RUN_NO_GPU_ADAPTER test={name} reason={what if how == 'NOTRUN' else '自证：强令本次未跑'}\n")
        continue
    what, _runs, runs = what.partition(" RUNS ")
    executed |= set(runs.split(",")) if runs else {function for function, definition in definitions.items() if what in definition}
    if how == "FILES":
        paths, _space, text = what.partition(" ")
        ok = all(text in open(os.path.join("crates", "tiny", path), encoding="utf-8").read() for path in paths.split(","))
    else:
        ok = (what in source.replace(line, "")) if how in ("NEEDS", "LOST", "LOSTINANOTHERCASE", "LOSTOUTSIDE") else os.path.isfile(os.path.join("crates", "tiny", what))
    if os.environ.get("FAKE_CARGO_FORCED_OUTCOME") in ("ok", "FAILED"):
        ok = os.environ["FAKE_CARGO_FORCED_OUTCOME"] == "ok"   # 记红撤绿那几格：同一把键、不改源码，让这一趟每条用例都 ok 或都 FAILED
    failed = failed or not ok
    if not ok and how == "LOSTOUTSIDE":
        print("GPU_DEVICE_LOST test=wgpu-callback card=样本 GPU reason=样本：设备丢失（没带点名用例的输出捕获的线程打的）")   # 实时打印的部分，在结果行之前、不在任何一段里
    print(f"test {full_name} ... {'ok' if ok else 'FAILED'}")
    if not ok and how == "LOST":
        lost_sections.append(f"---- {full_name} stdout ----\nGPU_DEVICE_LOST test=main card=样本 GPU reason=样本：设备丢失\n")
    if not ok and how == "LOSTOUTSIDE":
        lost_sections.append(f"---- {full_name} stdout ----\nassertion failed\n")
    if not ok and how == "LOSTINANOTHERCASE":
        lost_sections.append(f"---- {full_name} stdout ----\nassertion failed\n")
        lost_sections.append("---- tests::some_other_gpu_case stdout ----\nGPU_DEVICE_LOST test=tests::some_other_gpu_case card=样本 GPU reason=样本：设备丢失\n")
if lost_sections:
    print("\nfailures:\n\n" + "\n".join(lost_sections))
if shown:
    print("\nsuccesses:\n\n" + "\n".join(shown))
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
if os.environ.get("FAKE_CARGO_EDIT_SOURCE") and source.count(os.environ["FAKE_CARGO_EDIT_SOURCE"].split("\t")[0]) == 1:
    # 派活之后锚点失效那一格：跑完用例之后把这一份副本里的那段原文改掉，造「开跑之后别的会话改了源码」
    with open("crates/tiny/src/lib.rs", "w", encoding="utf-8") as handle:
        handle.write(source.replace(*os.environ["FAKE_CARGO_EDIT_SOURCE"].split("\t")))
sys.exit(101 if failed else 0)
'''
# 自证用的假 llvm 工具（同一份写成 llvm-profdata 与 llvm-cov 两个名字，按名字分工）：--version 打 rustc -vV 的 LLVM 版本（FAKE_LLVM_VERSION 设了打它）；
# llvm-profdata merge 把几份 .profraw 的函数名并起来；llvm-cov export 读合并的那一份与每个假可执行文件，照源码里每个 fn 的定义行交函数（区域起在 fn 那一列），
# 计数是执行过的 1、没执行的 0
FAKE_LLVM_TOOL = r'''#!/usr/bin/env python3
import json, os, re, subprocess, sys
arguments = sys.argv[1:]
if arguments[:1] == ["--version"]:
    version = os.environ.get("FAKE_LLVM_VERSION") or re.search(r"LLVM version: (\S+)", subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout).group(1)
    print(f"LLVM (http://llvm.org/):\n  LLVM version {version}-rust-selftest\n  Optimized build.")
    sys.exit(0)
if os.path.basename(sys.argv[0]) == "llvm-profdata":
    if arguments[:1] != ["merge"]:
        sys.exit(2)
    executed = set()
    for path in arguments[1:]:
        if path.endswith(".profraw"):
            with open(path, encoding="utf-8") as handle:
                executed |= set(json.load(handle)["executed"])
    with open(arguments[arguments.index("-o") + 1], "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
    sys.exit(0)
if arguments[:1] != ["export"]:
    sys.exit(2)
profile, objects, index = None, [], 1
while index < len(arguments):
    word = arguments[index]
    if word == "-instr-profile":
        profile, index = arguments[index + 1], index + 2
    elif word == "--object":
        objects.append(arguments[index + 1])
        index += 2
    elif word.startswith("-"):
        index += 1
    else:
        objects.append(word)
        index += 1
with open(profile, encoding="utf-8") as handle:
    executed = set(json.load(handle)["executed"])
functions, seen = [], set()
for path in objects:
    with open(path, encoding="utf-8") as handle:
        built = json.load(handle)
    for source in built["sources"]:
        full = os.path.join(built["root"], source)
        if full in seen:
            continue
        seen.add(full)
        with open(full, encoding="utf-8") as handle:
            for number, line in enumerate(handle.read().splitlines(), 1):
                match = re.match(r"^(\s*)(?:pub\s+)?(?:const\s+)?fn\s+(\w+)", line)
                if match:
                    count = 1 if match.group(2) in executed else 0
                    functions.append({"name": match.group(2), "count": count, "filenames": [full],
                                      "regions": [[number, len(match.group(1)) + 1, number, len(line) + 1, count, 0, 0, 0]]})
print(json.dumps({"data": [{"functions": functions}], "type": "llvm.coverage.json.export", "version": "2.0.1"}))
'''


PARALLEL_REASON_FORM = re.compile(r"^同一次 crates-mutation-replay（本模块进程 \d+）的第 (\d+) 个工作进程，各用各的源码副本与编译目录（这一轮共 4 个）$")


ITEM_PLAN_LINE = re.compile(r"^ITEM_PLAN item=(?P<item>.*) to_judge=(?P<to_judge>[0-9]+) reusable=(?P<reusable>[0-9]+)$")
ITEM_DONE_LINE = re.compile(r"^ITEM_DONE item=(?P<item>.*) seconds=(?P<seconds>[0-9]+\.[0-9]+) judged=(?P<judged>[0-9]+) reused=(?P<reused>[0-9]+) exit=(?P<exit>[0-9]+)$")
