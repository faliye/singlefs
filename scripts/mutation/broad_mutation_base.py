"""broad-mutation.py 拆出的一份。
标准库与仓内库的载入、全部模块级赋值（常量、表、正则、列名）、入口与拆出的各份模块的路径表。模块级可变状态（TEST_MODE、PRUNE_MODULE、REUSE_CHECKS_MODULE、selftest_fixtures）不在这一份：各在读写它的函数所在的那一份。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 broad-mutation.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → library → runner → gpu → run → selftest_a → selftest_b），没有环。
"""


import argparse
import datetime
import difflib
import fcntl
import hashlib
import importlib.util
import json
import os
import re
import secrets
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import zoneinfo
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import HeavyThreadDefaultError, default_heavy_threads  # noqa: E402
import run_build_directory  # noqa: E402  scripts/hosts/run_build_directory.py：这一趟的运行目录跑完删、删之前核开着的文件（文件头「库目录」runs/）

SELF_PATH = os.path.join(SCRIPT_DIRECTORY, "broad-mutation.py")
# libtest 的结果行（`test <全名>[ - should panic] ... <结局>`）全仓只在 lib_libtest_result_lines.py 解析一份：should panic 那一段不进名字
_LIBTEST_RESULT_LINES_SPECIFICATION = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_broad_mutation",
                                                                             os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_libtest_result_lines.py"))
libtest_result_lines = importlib.util.module_from_spec(_LIBTEST_RESULT_LINES_SPECIFICATION)
_LIBTEST_RESULT_LINES_SPECIFICATION.loader.exec_module(libtest_result_lines)
SCRIPT_KEY = "scripts/mutation/broad-mutation.py"   # 规范输出与复用哨兵认的脚本名
REAL_SELF_PATH = os.path.realpath(SELF_PATH)
# 入口与拆出的各份模块的源码：自证里扫源码的几格都按这两张表读（拆出的新模块加进来）
MODULE_SOURCE_PATHS = (
    SELF_PATH,
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_base.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_judging.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_library.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_runner.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_gpu.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_run.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_selftest_a.py"),
    os.path.join(SCRIPT_DIRECTORY, "broad_mutation_selftest_b.py"),
)
MODULE_SOURCE_FILES = tuple(os.path.relpath(path, REPOSITORY) for path in MODULE_SOURCE_PATHS)   # 仓根起的相对路径
DEFAULT_HOME_ENTRY = "broad-mutation-units"   # 库目录不给时取数据根下的这一项（.claude/hooks/lib_work_root.py 的 shared_data_path，两台读同一份）
DEFAULT_LOCK_WAIT_SECONDS = 120
DEFAULT_STOP_GRACE_SECONDS = 10
DEFAULT_SUSPECT_STREAK = 8           # 依据见文件头 --suspect-streak
DEFAULT_SUSPECT_TEST_SECONDS = 0.2
PROGRESS_INTERVAL_SECONDS = 300      # 一步跑着时隔多久报一行进度（文件头「输出」）
LOCK_POLL_SECONDS = 2
MUTATED_PACKAGES = ("singlefs-core", "singlefs-checker")    # 清单的次序：先 core 后 checker，序号从 1 起
STAGE_ONE_PACKAGES = ("singlefs-checker", "singlefs-core", "singlefs-format")   # 第一段：验证包之外的三个包
VERIFICATION_PACKAGE = "singlefs-verification"   # 第二段：验证包 crates/singlefs-verification
REGISTERED_REASON = "登记的崩溃枚举用例或放量分项（门禁 crash-point-replay 按块那一路判，这里经 --skip 排掉）"
ADMISSION = []                                  # scripts/gate/admission.py（第一次用到时才载入，registered_functions 用）
SUBMIT_PREFIX = "task-server/task submit --"   # 出路里起重活那条命令的前缀（文件头「谁起、什么时候起」）
VOLATILE_PREFIXES = ("/tmp", "/run", "/dev/shm", "/var/run")
VOLATILE_FILESYSTEMS = ("tmpfs", "ramfs")
IGNORED_DIRECTORY_NAMES = ("target", "mutants.out", "mutants.out.old")
# gate-overlap:copy-kept lib_verdict_library.py 广谱变异的判定是按写入次序只追加的行（同一把单元键几行、状态取最后一个定论），判定库共用部分按项名与两列键一项存一份 JSON；形态不同，读法、记红撤绿都不抽
VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "stage", "caught_by", "test_key", "coverage_key", "build_seconds", "test_seconds",
                   "judged_on", "method", "locator", "refused", "note")
# 判定库在 prune-server 里（文件头「判定库」）：族与键空间照 scripts/prune-server/prune_server_client.py 文件头「键空间」，种类固定，表是相对路径的头一段
PRUNE_SERVER_CLIENT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "prune-server", "prune_server_client.py")
PRUNE_SERVER_CLIENT_MODULE = "singlefs_prune_server_client"     # 与 lib_mutation_units.py 同一个模块名：一个进程只载入一份客户端
LIBRARY_KIND = "broad-mutation-units"
VERDICTS_TABLE, GPU_VERDICTS_TABLE, BASELINES_TABLE, BASELINE_ENTRIES_TABLE, COVERAGE_TABLE, LISTS_TABLE, COMPOSITIONS_TABLE = (
    "verdicts", "gpu-verdicts", "baselines", "baseline-entries", "coverage", "lists", "compositions")
ORDER_STATE = {"last": 0, "count": 0}    # order_stamp 在这个进程里用到的最后一个纳秒数与序号
BASELINE_COLUMNS = ("baseline_key", "step", "outcome", "finished_at", "threads")
# 按条记基线（文件头「基线」）：一行一份在没改过的树上跑绿的记录；kind 是 test（一条测试）、rest（一个二进制里宏调用展开的那几条）、binary（验证包一个二进制，
# 有拒算的是去掉那几条的那一份），binary 只当索引（rest 另按它找）
BASELINE_ENTRY_COLUMNS = ("record_key", "step", "threads", "kind", "binary", "finished_at")
COVERAGE_COLUMNS = ("record_key", "stage", "binary", "test", "covered", "profiles", "judged_on")
# 按覆盖逐条（验证包逐个二进制）挑测试、基线键拒算时按条记基线的那三个测试步；文档测试那一步每条都整步跑，不在里面
PER_ENTRY_TEST_STEPS = ("one-test", "two-test-verification")
EXIT_GATE_RED = 8
DECISIVE_OUTCOMES = ("caught", "missed", "unviable")          # 这条变异自己的结局
STAGE_ONE_ONLY = "stage-one-only"
DRIVER_ERROR = "driver-error"
NOT_RUN = "none"
ROW_OUTCOMES = (*DECISIVE_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR)
SUMMARY_COLUMNS = (*DECISIVE_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR, NOT_RUN)
SUMMARY_KEYS = {"caught": "caught", "missed": "missed", "unviable": "unviable",
                STAGE_ONE_ONLY: "stage_one_only", DRIVER_ERROR: "driver_error", NOT_RUN: "not_run"}
OUTCOME_TEXT = {"caught": "抓到", "missed": "没红", "unviable": "无效",
                STAGE_ONE_ONLY: "只过了第一段", DRIVER_ERROR: "驱动错", NOT_RUN: "沿用不了、还没现判"}
NEXTEST_TEST_FAILED_EXIT = 100       # cargo nextest run：有测试没过
CARGO_FAILED_EXIT = 101              # cargo：编不过、或 cargo test 有测试没过
EXIT_INCONSISTENT, EXIT_USAGE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_STOPPED = 1, 2, 4, 5, 6, 143
EXIT_RUN_DIRECTORY_KEPT = 9          # 这一趟别的都成了，运行目录没删掉（有进程开着里面的文件，或回读还在）
NEXTEST_FAILURE_LINE = re.compile(r"^\s*(?:FAIL|TIMEOUT|ABORT|LEAK-FAIL|SIG[A-Z0-9]+)\s+\[[^\]]*\]\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$")
# libtest 那几步（文档测试、验证包）的日志里认出二进制的两种行：cargo test 起每个测试二进制前打的 Running、文档测试打的 Doc-tests
LIBTEST_RUNNING_LINE = re.compile(r"^\s*Running (?:unittests )?(?P<source>\S+) \((?P<executable>[^)]+)\)\s*$")
LIBTEST_DOCTESTS_LINE = re.compile(r"^\s*Doc-tests (?P<crate>\S+)\s*$")
# 进度行里数的几种日志行：编了一个 crate、nextest 过了一条、libtest 过了一条
COMPILING_LINE = re.compile(r"^\s*Compiling \S")
NEXTEST_PASS_LINE = re.compile(r"^\s*PASS \[")
# 编译那一步的失败才算无效：日志里有一条带源码位置的 rustc 错误（error 行后面两行之内有 `--> 文件:行:列`），又有 cargo 的「could not compile」
COMPILE_ERROR_LINE = re.compile(r"^error(?:\[E\d+\])?: ")
COMPILE_ERROR_LOCATION = re.compile(r"^\s*--> \S+:\d+:\d+")
COULD_NOT_COMPILE_LINE = re.compile(r"^error: could not compile ")
HUNK_HEADER = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
UNRECOGNISED_TEST = "（没从输出里认出用例名，看日志）"
# cargo-mutants 自己改文件时跟在替换文后面的那段注释：照样带上，树里改出来的文件与它自己改的逐字相同（generate_list 拿它给的 diff 核）
MUTATION_MARKER = " /* ~ changed by cargo-mutants ~ */"
BASELINE_SEQUENCE = 0
ENVIRONMENT_SEQUENCE = "environment"      # 在没改过的树上验环境的那几步，日志与假一步的调用记录里用这个名字
SELFTEST_WORK_VARIABLE = "BROAD_MUTATION_SELFTEST_WORK"
SELFTEST_CELLS_VARIABLE = "BROAD_MUTATION_SELFTEST_CELLS"     # 开发时分段跑自证：逗号隔开的几组格名（selftest_cell_functions）；不设就是全部，门禁与提交前跑的都不设
SELFTEST_STEP_DRIVER_NAME = "fake-step-driver"   # 自证写在自证工作目录里的假一步（文件头「只供测试」BROAD_MUTATION_STEP_DRIVER）
TEST_ONLY_VARIABLES = ("BROAD_MUTATION_STEP_DRIVER", "BROAD_MUTATION_LIST_DRIVER", "BROAD_MUTATION_EXPORT_EVERYTHING", "BROAD_MUTATION_STAGE_ONE_ONLY", "BROAD_MUTATION_REJUDGE_NAMED",
                       "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_ALLOW_VOLATILE_HOME", "BROAD_MUTATION_REGISTERED", "BROAD_MUTATION_BREAK",
                       "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS")
SELFTEST_ANCESTOR_DEPTH = 6


NAME_LOCATION_PREFIX = re.compile(r"^[^:]+:\d+:\d+: ")


# ── 判定：键、沿用、状态 ───────────────────────────────────────────────────────────────────────────────────

PRUNE_BREAKS = ("catching-test-own-function-only", "mutated-function-only", "container-fingerprint-dropped", "mutated-test-service-code-keyed", "defining-file-in-key", "relative-position-dropped",
                "relative-position-counts-comments", "dependency-versions-keyed", "toolchain-version-keyed", "build-environment-keyed", "profile-keyed", "build-jobs-keyed", "judging-units-in-global-key", "manifest-text-in-global-key",
                "exclusions-in-global-key", "catching-test-exclusion-dropped", "unviable-rejudged", "missed-coverers-unchecked", "missed-always-rejudged",
                "list-cache-context-dropped", "outside-mutation-selected", "const-context-runs-selected", "spawning-binary-selected",
                "coverage-paths-unnormalized", "nextest-filter-single-argument", "coverers-cfg-inactive-refused")


# ── GPU 那一趟（C489：第二段补一趟带 gpu 特性的 验证包；文件头「GPU 那一趟」） ───────────────────────────────────────
# GPU 判定行的 gpu_key 是这一趟的写法与编进来的特性（GpuJudging.pass_key）；驱动的代码不进。

GPU_VERDICT_COLUMNS = ("key", "sequence", "package", "name", "outcome", "caught_by", "test_key", "coverage_key", "gpu_key", "test_seconds",
                       "judged_on", "method", "note")
GPU_ROW_OUTCOMES = (*DECISIVE_OUTCOMES, DRIVER_ERROR)
GPU_FEATURE = "gpu"
GPU_TESTS_DIRECTORY = "crates/singlefs-verification/tests"
GPU_BUILD_STEP, GPU_TEST_STEP = "gpu-build-verification", "gpu-test-verification"
GPU_BASELINE_SEQUENCE = "gpu-baseline"
GPU_NO_ADAPTER_MARK = "NOT_RUN_NO_GPU_ADAPTER"      # GPU 对拍找不到 GPU 适配器时测试打的那一行（门禁 crash-point-replay 的 feature-gated-tests 格读同一个词）
GPU_NOT_RUN = "not-run"                              # 这一趟没跑成：不落行、不算没红
BINDGEN_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-amplification-lib.sh")
GPU_ATTRIBUTE_START = re.compile(r"#(!?)\[\s*cfg\s*\(")
GPU_FEATURE_NAME = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")
GPU_PASSED_COUNT = re.compile(r"^test result: \w+\. (\d+) passed", re.M)
ROCKSDB_PREBUILT_PATH = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")


REGISTERED_CHILD = r'''
import importlib.util, json, sys, types
specification = importlib.util.spec_from_file_location("broad_mutation_registered_child", sys.argv[1])
driver = importlib.util.module_from_spec(specification)
specification.loader.exec_module(driver)
sys.modules['broad_mutation_judging'].TEST_MODE = True     # 弄坏开关在这个进程里认（探针把驱动当模块载入；test_mode 在 broad_mutation_judging 里读它）
rules = driver.prune_rules()
tree = rules.UnitTree(sys.argv[2])
quick = [(entry["binary"], entry["test"]) for entry in rules.test_entries(tree, (driver.VERIFICATION_PACKAGE,)) if entry["test"].endswith("kept_in_a_test_target")]
exclusions = driver.registered_exclusions(rules, tree, {("quick", "kept_in_a_test_target")})
nothing = driver.registered_exclusions(rules, tree, {("quick", "no_such_case")})
stub = types.SimpleNamespace(rules=rules, tree=tree, exclusions=[])
runnable = sorted(entry["test"] for entry in driver.Judgement.runnable(stub, ("singlefs-format",)))
runner = types.SimpleNamespace(skip_names=["alpha_case", "tool_case"])
base = ["cargo", "test", "--locked", "--release", "-p", driver.VERIFICATION_PACKAGE, "--lib", "--tests", "--no-fail-fast"]
commands = {"plain": driver.Runner.command_with_skips(runner, "two-test-verification", base),
            "exact": driver.Runner.command_with_skips(runner, "two-test-verification", base + ["--", "--exact", "a_case"]),
            "other": driver.Runner.command_with_skips(runner, "one-test", ["cargo", "nextest"]),
            "gpu": driver.Runner.command_with_skips(runner, driver.GPU_TEST_STEP, driver.gpu_pass_command("test", ["gpu"], ["lib"])),
            "none": driver.Runner.command_with_skips(types.SimpleNamespace(skip_names=[]), "two-test-verification", base)}
print(json.dumps({"quick": quick, "exclusions": exclusions, "nothing": nothing, "runnable": runnable, "commands": commands, "base": base}))
'''


