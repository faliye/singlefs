"""crates-mutation-rows.py 拆出的一份。
自证的驱动：Selftest 类、格函数表、弄坏开关与格的对应表、挑格与并行、run_selftest 与自带的几格。
入口文件头写着这一份在的位置、弄坏开关表与用法；入口 crates-mutation-rows.py 从各份 import 全部名字再导出，别处照旧按入口取名字。
拆出的各份只向前 import（base → judging → support → static → units → baseline → targets → memory → runs → pool → plan → stage → commands → driver），没有环。
"""
import contextlib
import gzip
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import traceback
from crates_mutation_rows_base import (BREAK_VARIABLES, DISK_FREE_VARIABLE, EXIT_SELECTION_ONLY, FAKE_CARGO, FAKE_LLVM_TOOL, HeavyThreadDefaultError,
    LLVM_TOOLS, MODULE_PATH, MODULE_SOURCE_PATHS, RECORD_HEADER, ROW_PROBLEM_KINDS, SCRIPT_DIRECTORY,
    SELFTEST_FIXTURES_PATH, STAGE_VARIABLES, TABLE, TASK_MEMORY_CAP_VARIABLE, break_is, default_heavy_threads, format_size,
    mutation_units, note, read_table, reuse_checks, task_memory_cap_bytes, write_atomically)
from crates_mutation_rows_judging import (prepare_shard)
from crates_mutation_rows_support import (build_selftest_repository, coverage_invocations, mutation_invocations, per_row_how, plain_invocations,
    replace_in_file, run_command_capturing, selftest_new_library_copy, selftest_verdict_view, selftest_write, source_hash)
from crates_mutation_rows_static import (escaping_literals, is_copied_into_each_shard, selftest_include_macro_literal_cell,
    selftest_run_overrides_outside_service_cell, selftest_static_check_parallel_repository)
from crates_mutation_rows_units import (library_of, memory_bounded_jobs, row_content_hash, run_records_directory, selftest_key_pool_fake_cells,
    selftest_key_release_fake_cell, selftest_library, selftest_multiple_packages_cell, selftest_reuse_cells, selftest_seed_short_timings,
    selftest_specification_line_cell, selftest_mutate_plan_test_service_code_cell)
from crates_mutation_rows_baseline import (selftest_baseline_timing_write_cell, selftest_coverage_exact_cell, selftest_device_lost_cell,
    selftest_heavy_chunk_cells, selftest_heavy_chunk_resume_cell, selftest_libtest_result_cell, selftest_not_run_item_name_cell,
    selftest_not_run_mid_line_cell, selftest_split_result_line_cell)
from crates_mutation_rows_targets import (selftest_disk_peak_cells, selftest_target_cap_cells, selftest_target_directory_cells,
    selftest_target_measure_cadence_cell, selftest_target_prune_cells, selftest_worker_slot_cells)
from crates_mutation_rows_memory import (THREAD_SETTING_VARIABLES, host_memory_capacity, selftest_disk_cells, selftest_dispatch_order_cell,
    selftest_jobs_after_memory_cut_cell, selftest_memory_cells, selftest_peak_target_cells, selftest_worker_count_cells, selftest_worker_growth_cell,
    selftest_worker_peak_cells)
from crates_mutation_rows_runs import (selftest_gpu_cell_dispatch_cells, selftest_gpu_test_threads_pool_targets_cell,
    selftest_output_collection_cells, selftest_output_progress_cells, selftest_output_stop_cells, selftest_service_build_directory_cell)
from crates_mutation_rows_pool import (selftest_heavy_chunk_verdict_cell, selftest_key_pool_repository_cells, selftest_long_baseline_chunk_cells,
    selftest_long_baseline_chunk_run_cell, selftest_mutation_run_service_cells, selftest_output_verdict_cell, selftest_pool_parallel_cells,
    selftest_seed_cells, selftest_seed_jobs_cell, selftest_stop_before_pool_cell, selftest_worker_failure_report_cell)
from crates_mutation_rows_plan import (selftest_debuginfo_cells, selftest_default_features_off_cell, selftest_gpu_route_feature_cell,
    selftest_gpu_test_threads_cell, selftest_driver_and_library_not_in_key_cell, selftest_key_release_cells, selftest_release_forced_cell,
    selftest_reuse_fix_cells, selftest_build_environment_and_version_cells, selftest_test_service_code_key_cell)
from crates_mutation_rows_stage import (run_stage, selftest_build_profile_cell, selftest_cargo_configuration_cell, selftest_gpu_configuration_cells,
    selftest_killed_run_resume_cell, selftest_link_failure_cell, selftest_product_gpu_feature_cells, selftest_sentinel_uncounted_cell,
    selftest_shard_root_window_cell, selftest_signal_and_pipe_cells)
from crates_mutation_rows_commands import (command_static_check, selftest_anchor_and_impact_cells, selftest_closure_refused_cell,
    selftest_crash_domain_narrowing_cells, selftest_item_lines_cells, selftest_manifest_path_cells,
    selftest_parallel_key_cell, selftest_path_dependency_copy_cells, selftest_reachability_cells, selftest_reachability_refused_cells,
    selftest_reachability_static_check, selftest_release_judged_tree_registry_cell, selftest_release_static_check_cell, selftest_static_check_parallel_cell,
    selftest_undefined_identifier_cell,
    selftest_unfiltered_static_check_cell, selftest_unit_key_cells, selftest_wgsl_unit_key_cell)
import crates_mutation_rows_support
import crates_mutation_rows_static
import crates_mutation_rows_units
import crates_mutation_rows_baseline
import crates_mutation_rows_targets
import crates_mutation_rows_memory
import crates_mutation_rows_runs
import crates_mutation_rows_pool
import crates_mutation_rows_plan
import crates_mutation_rows_stage
import crates_mutation_rows_commands


class Selftest:
    def __init__(self, work):
        self.work = work
        self.seeds_short_timings = True   # run 在 run_stage 起之前给没有用时的行记 10 秒（selftest_seed_short_timings）
        self.cells = 0
        self.failures = []
        self.not_run = []   # 本次未跑的格（skip）
        self.bin_directory = os.path.join(work, "bin")
        selftest_write(os.path.join(self.bin_directory, "cargo"), FAKE_CARGO, executable=True)
        self.llvm_directory = os.path.join(work, "llvm")
        for tool in LLVM_TOOLS:
            selftest_write(os.path.join(self.llvm_directory, tool), FAKE_LLVM_TOOL, executable=True)
        self.log = os.path.join(work, "cargo.log")
        # BINDGEN_EXTRA_CLANG_ARGS 也不带进来：调用方 export 了它，副本里没有 .cargo/ 照样编得过，「拷贝范围含 .cargo/」那一格就红不出来；
        # RUSTFLAGS 一类也不带：假 cargo 不看它，带进来只会让弄坏开关下的构建环境行随调用方变
        self.base_environment = {key: value for key, value in os.environ.items()
                                 if key not in STAGE_VARIABLES and key not in THREAD_SETTING_VARIABLES and key not in mutation_units.BUILD_VARIABLES and not key.startswith("CARGO_PROFILE_")
                                 and key not in ("BINDGEN_EXTRA_CLANG_ARGS", "FAKE_CARGO_VERSION", "FAKE_LLVM_VERSION",
                                                 # 本台的多机配置：交多机配置那几格自己设
                                                 "SINGLEFS_MULTI_HOST_CONFIG",
                                                 # 切块大小、进度间隔与在算行的门槛：切块、进度那几格自己设
                                                 "GATE_MUTATION_BASELINE_CHUNK_FILTERS", "GATE_MUTATION_PROGRESS_SECONDS", "GATE_MUTATION_COMPUTING_LINE_SECONDS")}
        self.base_environment.update(PATH=self.bin_directory + os.pathsep + os.environ.get("PATH", ""), FAKE_CARGO_LOG=self.log,
                                     GATE_MUTATION_TARGET_DIR=os.path.join(work, "target"),
                                     GATE_MUTATION_WORKERS="1", GATE_MUTATION_LLVM_TOOLS=self.llvm_directory)
        # 目标盘余量：别的格不比它，给一个用不完的假值；磁盘余量那几格自己设
        self.base_environment[DISK_FREE_VARIABLE] = "100T"
        # 内存上限：别的格不比它，这一项没有——服务交的变量不设（调用方设的已随 STAGE_VARIABLES 清掉），下面假 memory.max 写 max，交给本模块的多机配置不写 SLICE_MEMORY_CAP_BY_HOST；
        #   有了上限就要读每个工作进程的峰值记录，各格的运行记录目录随各自的小仓变（run_records_directory 的命名空间），从没量过的包的格单独跑（探针）。内存上限那几格自己设
        # 自己 scope 的 memory.max：它的有限值是硬顶（task_memory_cap_bytes），自证跑在任务服务给的 scope 里，真 scope 的上限会把别的格要的工作进程数收窄；
        # 换成临时目录里写着 max（不限）的一份（GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT），内存上限那几格自己改
        cgroup_listing = os.path.join(work, "selftest-cgroup-listing")
        cgroup_root = os.path.join(work, "selftest-cgroup")
        selftest_write(cgroup_listing, "0::/selftest.slice\n")
        selftest_write(os.path.join(cgroup_root, "selftest.slice", "memory.max"), "max\n")
        self.base_environment["GATE_MUTATION_CGROUP_LISTING"] = cgroup_listing
        self.base_environment["GATE_MUTATION_CGROUP_ROOT"] = cgroup_root
        # 复用哨兵：别的格比的是现判与复用的条数，键相同的全当哨兵、不抽强制现判；reuse-three-questions 与 reuse-sentinel 两格自己设
        self.base_environment[reuse_checks.SENTINEL_COUNT_VARIABLE] = "1000"
        self.base_environment.pop(reuse_checks.SENTINEL_SEED_VARIABLE, None)
        # 交给真跑测试那几格的多机配置（文件头「真跑测试的格把多机配置交给测试、带 gpu 特性的格跑完核真用了卡」）：调用方设的不带进来，
        # 本模块所在仓那一份也不读（有没有、写了什么随机器变），换成临时目录里的一份；交多机配置那几格自己设
        self.base_environment.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        # 调用方设的那一份另记下来（只读 SINGLEFS_MULTI_HOST_CONFIG 这一个变量，不补缺省）：变异格在进程池的工作进程里起临时 prune-server，scripts/prune-server 的 TestServer 照工作进程
        #   自己的进程环境认这一台在多机配置 HOSTS 里的地址，进程环境里没有它就退回本模块所在仓根的 multi-host.env（源码副本里没有）。它不进上面的 base_environment：
        #   run_stage 的 environment 里有它，交给测试的多机配置就照它、顶掉下面 GATE_MUTATION_MULTI_HOST_CONFIG 那份自证用的（pool_process_variables 交进进程环境的是工作进程与它起的子进程）
        self.caller_multi_host_configuration = os.environ.get("SINGLEFS_MULTI_HOST_CONFIG", "").strip()
        self.multi_host_configuration = os.path.join(work, "selftest-multi-host", "multi-host.env")
        selftest_write(self.multi_host_configuration, "# 自证用的多机配置：假 cargo 不读它，交多机配置那几格只核它的路径到没到测试的环境里\nENABLE_GPU=1\n")
        self.base_environment["GATE_MUTATION_MULTI_HOST_CONFIG"] = self.multi_host_configuration
        # 用时表与占盘记录（运行记录，文件头「判定库」末段）放这一趟自证自己的目录，不读真仓的多机配置、不碰真共享根
        self.base_environment["GATE_MUTATION_RUN_RECORDS_DIRECTORY"] = os.path.join(work, "selftest-run-records")
        # lib_function_units.py 的 Rust 词法单元缓存（它文件头「Rust 词法单元的缓存」）放进这一趟自证自己的目录，不碰真仓那一份
        self.base_environment["FUNCTION_UNITS_LEX_CACHE_DIRECTORY"] = os.path.join(work, "rust-lex-cache")

    def run(self, root, **changes):
        """在 root 上跑一趟 run_stage：(退出码, stdout, stderr, 这一趟假 cargo 起的每一行)。"""
        with contextlib.suppress(FileNotFoundError):
            os.unlink(self.log)
        environment = dict(self.base_environment, **changes)
        if self.seeds_short_timings:
            selftest_seed_short_timings(root, environment)
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr), self.pool_workers_process_environment(environment):
            exit_code = run_stage(root, environment)
        invocations = []
        if os.path.exists(self.log):
            with open(self.log, encoding="utf-8") as handle:
                invocations = handle.read().splitlines()
        return exit_code, stdout.getvalue(), stderr.getvalue(), invocations

    def pool_process_variables(self, environment):
        """进程池的工作进程要在自己的进程环境里有的变量（{名字: 值}）：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 就是它那一项，没设交空（不补缺省）。
        environment 是这一趟 run_stage 要用的环境（弄坏开关 pool-worker-caller-configuration-dropped 在里面读）。
        工作进程从起进程池的那个进程继承进程环境：进程内跑 run_stage 的经 pool_workers_process_environment、自证起子进程跑 run_stage 的经 pool_process_environment 交。
        run_stage 的 environment 参数不带它（__init__ 里 base_environment 的说明）。
        弄坏开关 pool-worker-caller-configuration-dropped：不交，工作进程认不到调用方指的多机配置、退回本模块所在仓根那一份。"""
        if not self.caller_multi_host_configuration or break_is(environment, "pool-worker-caller-configuration-dropped"):
            return {}
        return {"SINGLEFS_MULTI_HOST_CONFIG": self.caller_multi_host_configuration}

    def pool_process_environment(self, environment):
        """自证起子进程（子进程里跑 run_stage、起进程池）用的环境：environment 再加 pool_process_variables 那一项，environment 里已有的照它。"""
        return dict(self.pool_process_variables(environment), **environment)

    @contextlib.contextmanager
    def pool_workers_process_environment(self, environment):
        """进程内跑 run_stage 那一段：本进程的环境里临时有 pool_process_variables 那一项（进程池 fork 出的工作进程继承它），出来原样还回去。
        run_stage 用的 environment 参数不变，交给测试的多机配置还是 GATE_MUTATION_MULTI_HOST_CONFIG 那份自证用的。"""
        extra = self.pool_process_variables(environment)
        saved = {name: os.environ.get(name) for name in extra}
        os.environ.update(extra)
        try:
            yield
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value

    def check(self, label, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(f"{label}{('：' + detail) if detail else ''}")

    def skip(self, label, reason):
        """这一格的前提没造出来：记本次未跑（不算对、也不算错，不进格数），run_selftest 收尾逐格列出，别的格都对时整趟退 77。"""
        self.not_run.append(f"{label}：{reason}")


def selftest_loaded_by_path_parallel_cell(selftest, work):
    """本模块被别的脚本按文件路径装成别的模块名、不登记进 sys.modules（scripts/mutation/prove-red.sh 的 load_module 那种装法）时，并行的几处照样能用（forked_parts 不 pickle 函数）：
    起一个 python 子进程照那种装法装本模块，static-check 在 static-check 切片并行那份样本仓上跑（GATE_MUTATION_KEY_WORKERS=3）；子进程退 0，static-check 输出同本进程单线程那一趟逐行相同。
    弄坏开关 forked-parts-by-pool（换回进程池、按名字 pickle 函数）下这一格判错（子进程抛 PicklingError、退非 0）。子进程自己一个进程组，超时只停它自己。"""
    root, result_path = (os.path.join(work, name) for name in ("loaded-by-path-sample", "loaded-by-path-result.json"))
    selftest_static_check_parallel_repository(root)
    child_code = ("import importlib.util, json, os, sys\n"
                  "specification = importlib.util.spec_from_file_location('crates_mutation_rows_loaded_by_path', sys.argv[1])\n"
                  "module = importlib.util.module_from_spec(specification)\n"
                  "specification.loader.exec_module(module)\n"
                  "module.load_selftest_fixtures()\n"
                  "registered = specification.name in sys.modules\n"
                  "stops = module.selftest_fixture_values()['SELFTEST_REACHABILITY_STOPS']\n"
                  "code, output, _routes, _exempted = module.selftest_reachability_static_check(sys.argv[2], stops, {'GATE_MUTATION_KEY_WORKERS': '3'})\n"
                  "with open(sys.argv[3], 'w', encoding='utf-8') as handle:\n"
                  "    json.dump(dict(registered=registered, code=code, output=output), handle)\n")
    child = subprocess.Popen([sys.executable, "-c", child_code, MODULE_PATH, root, result_path],
                             stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        child_output, _ = child.communicate(timeout=selftest_cell_limits()["function_seconds"] - 60)
    except subprocess.TimeoutExpired:
        child.kill()
        child_output, _ = child.communicate()
    try:
        with open(result_path, encoding="utf-8") as handle:
            result = json.load(handle)
    except (OSError, ValueError) as error:
        selftest.check("按文件路径装成别的模块名时 static-check 的并行照样能用：子进程退 0", False,
                       f"子进程退 {child.returncode}、结果文件读不了（{type(error).__name__}）；子进程输出尾部：{child_output[-1500:]}")
        return
    single_code, single_output, _routes, _exempted = selftest_reachability_static_check(root, SELFTEST_REACHABILITY_STOPS, {"GATE_MUTATION_KEY_WORKERS": "1"})
    selftest.check("按文件路径装成别的模块名（不登记进 sys.modules）装本模块：子进程退 0、本模块没登记进 sys.modules",
                   child.returncode == 0 and not result["registered"], f"子进程退 {child.returncode}、登记进 sys.modules {result['registered']}")
    selftest.check("按文件路径装成别的模块名时 static-check 切片并行（GATE_MUTATION_KEY_WORKERS=3）的输出同本进程单线程那一趟逐行相同",
                   result["code"] == single_code and result["output"].splitlines() == single_output.splitlines(),
                   f"退 {result['code']} 与 {single_code}；子进程输出尾部：{result['output'][-600:]}")


def selftest_cell_functions():
    """自证的格函数，照跑的次序（函数体里的表；selftest_cell_call 照名字从这张表里取，不用 globals() 查）。
    一个格函数里的格互相有次序（头一趟、第二趟复用这一类），不拆开；按开关挑格与并行都以格函数为单位。"""
    return (
        run_selftest_cells, selftest_unit_key_cells, selftest_reuse_cells,
        selftest_sentinel_uncounted_cell, selftest_parallel_key_cell, selftest_wgsl_unit_key_cell,
        selftest_multiple_packages_cell, selftest_specification_line_cell, selftest_driver_and_library_not_in_key_cell,
        selftest_test_service_code_key_cell, selftest_mutate_plan_test_service_code_cell,
        selftest_build_environment_and_version_cells, selftest_loaded_by_path_parallel_cell,
        selftest_reuse_fix_cells, selftest_item_lines_cells, selftest_crash_domain_narrowing_cells,
        selftest_build_profile_cell, selftest_worker_count_cells, selftest_memory_cells,
        selftest_worker_peak_cells, selftest_peak_target_cells, selftest_jobs_after_memory_cut_cell, selftest_worker_growth_cell, selftest_dispatch_order_cell,
        selftest_disk_cells, selftest_target_cap_cells,
        selftest_target_prune_cells, selftest_target_measure_cadence_cell, selftest_disk_peak_cells, selftest_libtest_result_cell,
        selftest_split_result_line_cell, selftest_cargo_configuration_cell, selftest_product_gpu_feature_cells,
        selftest_gpu_configuration_cells, selftest_target_directory_cells, selftest_worker_slot_cells,
        selftest_signal_and_pipe_cells, selftest_killed_run_resume_cell, selftest_shard_root_window_cell,
        selftest_stop_before_pool_cell, selftest_worker_failure_report_cell, selftest_closure_refused_cell, selftest_coverage_exact_cell,
        selftest_not_run_item_name_cell, selftest_not_run_mid_line_cell, selftest_device_lost_cell, selftest_link_failure_cell, selftest_gpu_route_feature_cell, selftest_gpu_test_threads_cell,
        selftest_gpu_test_threads_pool_targets_cell,
        selftest_gpu_cell_dispatch_cells, selftest_service_build_directory_cell,
        selftest_release_forced_cell, selftest_default_features_off_cell, selftest_release_static_check_cell, selftest_release_judged_tree_registry_cell, selftest_unfiltered_static_check_cell,
        selftest_anchor_and_impact_cells,
        selftest_undefined_identifier_cell, selftest_reachability_cells, selftest_reachability_refused_cells, selftest_static_check_parallel_cell,
        selftest_debuginfo_cells, selftest_seed_cells, selftest_seed_jobs_cell, selftest_pool_parallel_cells,
        selftest_mutation_run_service_cells, selftest_run_overrides_outside_service_cell, selftest_path_dependency_copy_cells,
        selftest_include_macro_literal_cell, selftest_manifest_path_cells, selftest_output_collection_cells, selftest_output_stop_cells,
        selftest_output_progress_cells, selftest_output_verdict_cell, selftest_heavy_chunk_cells,
        selftest_heavy_chunk_verdict_cell, selftest_long_baseline_chunk_cells, selftest_long_baseline_chunk_run_cell, selftest_baseline_timing_write_cell,
        selftest_heavy_chunk_resume_cell,
        selftest_key_pool_fake_cells,
        selftest_key_pool_repository_cells, selftest_key_release_fake_cell, selftest_key_release_cells,
        selftest_cell_scheduling_cells, selftest_break_cell_table_cell,
    )


def selftest_cell_function_names():
    """自证的格函数的名字，次序同 selftest_cell_functions。"""
    return tuple(function.__name__ for function in selftest_cell_functions())


def selftest_cell_functions_without_work():
    """只收 selftest、不收 work 的格函数（不在临时目录里搭东西）。"""
    return ("selftest_sentinel_uncounted_cell", "selftest_heavy_chunk_cells", "selftest_long_baseline_chunk_cells", "selftest_key_pool_fake_cells", "selftest_break_cell_table_cell", "selftest_seed_jobs_cell")


def selftest_timing_sensitive_cell_functions():
    """跑的时候要等对方起来、机器忙时会把格记成本次未跑的格函数（selftest.skip）：并行时排在派活池最前，与别的格函数同时跑。"""
    return ("selftest_killed_run_resume_cell", "selftest_worker_slot_cells", "selftest_heavy_chunk_resume_cell")


def selftest_break_cell_table():
    """弄坏开关 → 打开后必须判错的格所在的格函数（文件头「弄坏开关」那一节点名的格）。值是元组：只走这些格函数就够看到开关红；
    None：没有哪一格点名它，走全部格函数；空元组：本模块没有格点名它（由别的自证证红），只走核表那一格。
    弄坏开关 selftest-break-table-entry-dropped 下少一行 nobaseline。"""
    table = {
        "anchor-mid-run-asserts": ("selftest_anchor_and_impact_cells",),
        "anchor-rows-planned": ("selftest_anchor_and_impact_cells",),
        "anchor-stops-table": ("selftest_anchor_and_impact_cells",),
        "baseline-batch-composition": ("selftest_reuse_fix_cells",),
        "baseline-by-tree-snapshot": ("selftest_unit_key_cells",),
        "baseline-key-without-target": ("selftest_reuse_fix_cells",),
        "baseline-on-temporary-service": ("selftest_mutation_run_service_cells",),
        "baseline-one-at-a-time": ("selftest_pool_parallel_cells",),
        "baseline-red-keeps-caught": ("selftest_reuse_fix_cells",),
        "baseline-release-incremental-kept": ("selftest_debuginfo_cells",),
        "broken-pool-looked-up-late": ("selftest_stop_before_pool_cell",),
        "buildprofilesunset": ("selftest_build_profile_cell",),
        "cargoconfignotcopied": ("selftest_cargo_configuration_cell", "selftest_seed_cells"),
        "cell-child-inherits-parent-process-state": ("selftest_cell_scheduling_cells",),
        "cell-heaviest-first-ignored": ("selftest_cell_scheduling_cells",),
        "cell-jobs-by-task-cap": ("selftest_cell_scheduling_cells",),
        "cell-jobs-by-threads-only": ("selftest_cell_scheduling_cells",),
        "cell-leftover-processes-kept": ("selftest_cell_scheduling_cells",),
        "cell-limit-ignored": ("selftest_cell_scheduling_cells",),
        "cell-outcome-dropped": ("selftest_cell_scheduling_cells",),
        "closure-error-masked": ("selftest_closure_refused_cell",),
        "compile-per-worker": ("selftest_seed_cells", "selftest_pool_parallel_cells"),
        "coveragenotrecorded": ("run_selftest_cells", "selftest_build_profile_cell", "selftest_unit_key_cells"),
        "dead-worker-cause-unprinted": ("selftest_worker_failure_report_cell",),
        "debuginfo-kept": ("selftest_debuginfo_cells",),
        "depsnotpruned": ("selftest_target_prune_cells",),
        "device-lost-ignored": ("selftest_device_lost_cell",),
        "link-failure-as-invalid": ("selftest_link_failure_cell",),
        "link-failure-not-red": ("selftest_link_failure_cell",),
        "dispatch-by-crate-only": ("selftest_dispatch_order_cell",),
        "diskbudgetbycap": ("selftest_disk_peak_cells",),
        "disknotchecked": ("selftest_disk_cells",),
        "emptyreplacementrefused": ("selftest_cargo_configuration_cell",),
        "exactduplicated": ("selftest_coverage_exact_cell",),
        "extratargetskept": ("selftest_target_directory_cells",),
        "forked-parts-by-pool": ("selftest_loaded_by_path_parallel_cell",),
        "given-states-count-ignored": ("selftest_gpu_configuration_cells",),
        "gpu-cells-dispatched-in-parallel": ("selftest_gpu_cell_dispatch_cells",),
        "gpu-route-feature-unadded": ("selftest_gpu_route_feature_cell",),
        "gpu-test-threads-unadded": ("selftest_gpu_test_threads_cell",),
        "gpu-test-threads-all-one": ("selftest_gpu_test_threads_cell", "selftest_gpu_test_threads_pool_targets_cell"),
        "gpu-unused-as-missed": ("selftest_gpu_configuration_cells",),
        "heavy-batch-row-dropped": ("selftest_heavy_chunk_cells", "selftest_heavy_chunk_verdict_cell", "selftest_heavy_chunk_resume_cell",
                                    "selftest_long_baseline_chunk_cells", "selftest_long_baseline_chunk_run_cell"),
        "heavy-batch-unsplit": ("selftest_heavy_chunk_cells", "selftest_heavy_chunk_verdict_cell", "selftest_heavy_chunk_resume_cell",
                                "selftest_long_baseline_chunk_cells", "selftest_long_baseline_chunk_run_cell", "selftest_baseline_timing_write_cell"),
        "impact-row-dropped": ("selftest_anchor_and_impact_cells",),
        "include-literals-from-crate-root": ("selftest_include_macro_literal_cell",),
        "inusetargetremoved": ("selftest_target_directory_cells",),
        "item-done-reused-silent": ("selftest_item_lines_cells",),
        "item-done-seconds-whole-run": ("selftest_item_lines_cells",),
        "item-plan-reusable-zero": ("selftest_closure_refused_cell", "selftest_item_lines_cells"),
        "item-plan-selection-ignored": ("selftest_closure_refused_cell", "selftest_item_lines_cells"),
        "jobs-feedback-ignored": ("selftest_worker_peak_cells",),
        "keepmutationtmp": None,
        "key-context-kept-for-workers": ("selftest_key_release_fake_cell", "selftest_key_release_cells"),
        "key-parent-not-prewarmed": ("selftest_key_pool_fake_cells", "selftest_key_pool_repository_cells"),
        "key-pool-not-frozen": ("selftest_key_pool_fake_cells", "selftest_key_pool_repository_cells", "selftest_key_release_cells"),
        "key-token-values-unshared": ("selftest_key_pool_fake_cells", "selftest_key_pool_repository_cells"),
        "library-outside-git-tolerated": ("selftest_reuse_fix_cells",),
        "long-baseline-not-isolated": ("selftest_long_baseline_chunk_cells", "selftest_long_baseline_chunk_run_cell", "selftest_baseline_timing_write_cell"),
        "unknown-timing-merged-as-short": ("selftest_long_baseline_chunk_cells", "selftest_long_baseline_chunk_run_cell", "selftest_baseline_timing_write_cell"),
        "baseline-timing-unwritten": ("selftest_baseline_timing_write_cell",),
        "lifecycle-filter-any-stream": ("selftest_output_collection_cells",),
        "lifecycle-lines-kept": ("selftest_output_collection_cells", "selftest_output_stop_cells", "selftest_output_progress_cells", "selftest_output_verdict_cell"),
        "lifecycle-prefix-loose": ("selftest_output_verdict_cell",),
        "lifecycle-prefix-renamed": ("selftest_output_stop_cells",),
        "lifecycle-tail-dropped-on-stop": ("selftest_output_stop_cells",),
        "listitemspipeunhandled": ("selftest_signal_and_pipe_cells",),
        "llvmversionunchecked": ("selftest_unit_key_cells",),
        "local-shard-numbering": ("selftest_worker_slot_cells",),
        "manifest-path-ignored": ("selftest_manifest_path_cells",),
        "memory-cap-ignored": ("selftest_memory_cells", "selftest_worker_peak_cells"),
        "memory-cap-exits-before-fewer-workers": ("selftest_memory_cells",),
        "narrowed-verdicts-recorded": ("selftest_crash_domain_narrowing_cells",),
        "narrowed-sentinel-drawn": ("selftest_crash_domain_narrowing_cells",),
        "narrowed-sharding-allowed": ("selftest_crash_domain_narrowing_cells",),
        "memory-cap-short-self-exit": ("selftest_memory_cells",),
        "memory-max-not-hard-cap": ("selftest_memory_cells",),
        "multiple-packages-last-only": ("selftest_multiple_packages_cell",),
        "mutated-source-reread": ("selftest_static_check_parallel_cell",),
        "mutated-test-service-code-keyed": ("selftest_test_service_code_key_cell", "selftest_mutate_plan_test_service_code_cell"),
        "mutated-unit-ignored": ("selftest_unit_key_cells",),
        "mutation-run-on-real-service": ("selftest_mutation_run_service_cells",),
        "new-package-run-single-worker": ("selftest_worker_peak_cells",),
        "no-default-features-unadded": ("selftest_default_features_off_cell",),
        "nobaseline": ("run_selftest_cells", "selftest_reuse_fix_cells"),
        "not-run-test-name-only": ("selftest_not_run_item_name_cell",),
        "not-run-line-start-only": ("selftest_not_run_mid_line_cell",),
        "notrunasmissed": ("run_selftest_cells", "selftest_not_run_mid_line_cell"),
        "overcapdispatched": ("selftest_target_prune_cells",),
        "overlapunchecked": ("selftest_worker_slot_cells",),
        "path-dependencies-not-copied": ("selftest_path_dependency_copy_cells",),
        "plain-cells-cards-visible": ("selftest_gpu_cell_dispatch_cells",),
        "pool-worker-caller-configuration-dropped": ("selftest_worker_failure_report_cell",),
        "product-gpu-feature-unquiesced": ("selftest_product_gpu_feature_cells",),
        "build-jobs-keyed": ("selftest_debuginfo_cells",),
        "build-environment-keyed": ("selftest_unit_key_cells", "selftest_build_environment_and_version_cells"),
        "dependency-versions-keyed": ("selftest_build_environment_and_version_cells",),
        "driver-and-library-keyed": ("selftest_driver_and_library_not_in_key_cell",),
        "profile-keyed": ("selftest_debuginfo_cells",),
        "toolchain-version-keyed": ("selftest_unit_key_cells", "selftest_build_environment_and_version_cells"),
        "progress-line-unwritten": ("selftest_output_stop_cells",),
        "heartbeat-prefix-dropped": ("selftest_output_progress_cells",),
        "progress-line-only-on-output": ("selftest_output_progress_cells",),
        "computing-cell-baseline-unrecognized": ("selftest_output_progress_cells",),
        "computing-cell-coverage-unrecognized": ("selftest_output_progress_cells",),
        "computing-cell-prefix-renamed": ("selftest_output_progress_cells",),
        "computing-line-unwritten": ("selftest_output_progress_cells",),
        "computing-line-without-output": ("selftest_output_progress_cells",),
        "reachability-closure-unchecked": ("selftest_reachability_cells",),
        "reachability-cpu-configuration-ignored": ("selftest_reachability_cells",),
        "reachability-cpu-configuration-test-code-only": ("selftest_reachability_cells",),
        "reachability-edges-as-units": ("selftest_reachability_cells",),
        "reachability-fallback-roots-ignored": ("selftest_reachability_cells",),
        "reachability-refused-unjudged": ("selftest_gpu_test_threads_cell", "selftest_reachability_refused_cells"),
        "reachability-rows-unsliced": ("selftest_static_check_parallel_cell",),
        "workspace-files-unsliced": ("selftest_static_check_parallel_cell",),
        "rust-file-replaced-on-entry-module": ("selftest_static_check_parallel_cell",),
        "reachability-source-text-ignored": ("selftest_reachability_cells",),
        "reachability-stops-ignored": ("selftest_reachability_cells",),
        "reachability-stops-unverified": ("selftest_reachability_cells",),
        "records-written-at-end": ("selftest_killed_run_resume_cell",),
        "red-not-recorded": ("selftest_reuse_fix_cells",),
        "release-incremental-unset": ("selftest_debuginfo_cells",),
        "release-unchecked": ("selftest_release_static_check_cell",),
        "release-unforced": ("selftest_release_forced_cell",),
        "reuse-lookup-skipped": ("selftest_reuse_cells",),
        "rocksdb-entry-unchecked": ("run_selftest_cells",),
        "row-key-collision-unrefused": ("selftest_reuse_fix_cells",),
        "rowplanselectionignored": (),
        "unsharded-rows-judged": (),
        "run-overrides-honored-outside-service": ("selftest_run_overrides_outside_service_cell",),
        "runtargetkept": ("selftest_signal_and_pipe_cells", "selftest_target_directory_cells"),
        "runtargetopenfilesunchecked": ("selftest_target_directory_cells",),
        "sample-root-peaks-recorded": ("selftest_worker_peak_cells",),
        "seed-copy-serial": ("selftest_pool_parallel_cells", "selftest_seed_cells"),
        "seed-copy-threads-unregistered": ("selftest_seed_jobs_cell",),
        "seedfreshacrossgroups": ("selftest_seed_cells",),
        "seedfreshunchecked": ("selftest_seed_cells",),
        "seedmtimekept": ("selftest_seed_cells",),
        "selftest-break-cells-ignored": ("selftest_break_cell_table_cell",),
        "service-build-directory-ignored": ("selftest_service_build_directory_cell",),
        "selftest-break-table-entry-dropped": ("selftest_break_cell_table_cell",),
        "shardrootkeptonsignal": ("selftest_signal_and_pipe_cells", "selftest_stop_before_pool_cell"),
        "shardrootnamedlate": ("selftest_shard_root_window_cell",),
        "specification-lines-product-view": ("selftest_specification_line_cell",),
        "split-result-line-unseen": ("selftest_split_result_line_cell",),
        "static-check-registry-from-script-repository": ("selftest_release_static_check_cell", "selftest_release_judged_tree_registry_cell"),
        "static-check-slices-misordered": ("selftest_static_check_parallel_cell",),
        "staticusagebare": ("run_selftest_cells",),
        "tablemissingok": ("run_selftest_cells",),
        "target-measured-every-cell": ("selftest_target_measure_cadence_cell",),
        "targetcapignored": ("selftest_target_cap_cells", "selftest_target_prune_cells"),
        "targetdefaulttmp": ("selftest_signal_and_pipe_cells", "selftest_target_directory_cells"),
        "task-memory-constants-skewed": ("selftest_memory_cells",),
        "term-sent-unwaited": ("selftest_signal_and_pipe_cells", "selftest_stop_before_pool_cell"),
        "test-closure-ignored": ("selftest_unit_key_cells",),
        "test-runs-configuration-unpassed": ("selftest_gpu_configuration_cells",),
        "timingsignored": ("run_selftest_cells",),
        "uncounted-forced-observed": ("selftest_sentinel_uncounted_cell",),
        "uncounted-overwrites-counted": ("selftest_reuse_fix_cells",),
        "undefinedidentifiersunchecked": ("selftest_undefined_identifier_cell",),
        "unfiltered-rows-unchecked": ("selftest_unfiltered_static_check_cell",),
        "unmeasured-package-not-probed-alone": ("selftest_worker_peak_cells",),
        "unmeasured-target-borrows-package-peak": ("selftest_peak_target_cells",),
        "unmeasured-worker-peak-guessed": ("selftest_worker_peak_cells",),
        "unreadable-worker-peaks-guessed": ("selftest_worker_peak_cells",),
        "wgsl-whole-file": ("selftest_wgsl_unit_key_cell",),
        "worker-failure-unconverted": ("selftest_worker_failure_report_cell",),
        "worker-jobs-held-after-memory-cut": ("selftest_jobs_after_memory_cut_cell",),
        "worker-peaks-unkeyed-by-jobs": ("selftest_worker_peak_cells",),
        "worker-peaks-unkeyed-by-package": ("selftest_worker_peak_cells",),
        "worker-peaks-unkeyed-by-target": ("selftest_peak_target_cells",),
        "worker-peaks-unrecorded": ("selftest_worker_peak_cells",),
        "worker-peaks-window-unbounded": ("selftest_worker_peak_cells",),
        "worker-tree-self-only": ("selftest_worker_peak_cells",),
        "workers-beyond-rows": ("selftest_worker_count_cells",),
        "workers-not-grown-after-measuring": ("selftest_worker_growth_cell",),
        "workers-by-processor-half": ("selftest_worker_count_cells", "selftest_seed_jobs_cell"),
        "workspaceunitsshared": ("selftest_seed_cells",),
    }
    if break_is(os.environ, "selftest-break-table-entry-dropped"):
        table.pop("nobaseline", None)
    return table


def selftest_break_switches(environment):
    """环境里打开的弄坏开关（两个变量 GATE_MUTATION_BREAK、CRATES_MUTATION_ROWS_BREAK，逗号隔开），照出现的次序。"""
    return [name for variable in BREAK_VARIABLES for name in environment.get(variable, "").split(",") if name]


def selftest_selected_cell_functions(environment):
    """这一趟自证要走的格函数，照 selftest_cell_function_names 的次序：没有弄坏开关，或有一条开关表里没有、值是 None，走全部；
    开关都在表里就走它们点名的格函数的并集，加上核表那一格。弄坏开关 selftest-break-cells-ignored 下一律走全部。"""
    names = selftest_cell_function_names()
    switches = selftest_break_switches(environment)
    if not switches or break_is(os.environ, "selftest-break-cells-ignored"):
        return names
    table = selftest_break_cell_table()
    named = {"selftest_break_cell_table_cell"}
    for switch in switches:
        cells = table.get(switch)
        if cells is None:
            return names
        named.update(cells)
    return tuple(name for name in names if name in named)


def selftest_break_cell_table_cell(selftest):
    """弄坏开关表与按开关挑格：表点名的格函数都存在；入口与拆出的各份模块里读的每一条弄坏开关表里都有一行；表里有的开关只走点名的格函数的并集，
    表里没有的开关与没有开关走全部。"""
    names = selftest_cell_function_names()
    table = selftest_break_cell_table()
    unknown = sorted({cell for cells in table.values() if cells for cell in cells} - set(names))
    selftest.check("弄坏开关表点名的格函数都是自证的格函数", not unknown and all(callable(function) for function in selftest_cell_functions()),
                   f"表里点名了不存在的格函数：{unknown}")
    listed = {os.path.basename(path) for path in MODULE_SOURCE_PATHS}
    on_disk = {name for name in os.listdir(SCRIPT_DIRECTORY) if name.startswith("crates_mutation_rows_") and name.endswith(".py") and not name.endswith("_selftest_fixtures.py")}
    selftest.check("MODULE_SOURCE_PATHS 列全了本目录里拆出的模块（夹具除外）", on_disk <= listed, f"没列的：{sorted(on_disk - listed)}")
    source = ""
    for path in MODULE_SOURCE_PATHS:
        with open(path, encoding="utf-8") as handle:
            source += handle.read()
    found = (set(re.findall(r'break_is\((?:[^"()]|\([^()]*\)|"[^"]*")*?,\s*"([a-z0-9-]+)"\)', source))
             | set(re.findall(r'break_is\("([a-z0-9-]+)"\)', source)) | set(re.findall(r'with_break\(selftest, "([a-z0-9-]+)"\)', source)))
    missing = sorted(found - set(table))
    selftest.check("本文件代码里读的每一条弄坏开关在弄坏开关表里都有一行", bool(found) and not missing, f"表里缺：{missing}")
    probe = "nobaseline"
    expected = set(table.get(probe) or ()) | {"selftest_break_cell_table_cell"}
    only_probe = selftest_selected_cell_functions({BREAK_VARIABLES[0]: probe})
    selftest.check("表里有的弄坏开关只走它点名的格函数（加核表那一格）、次序照格函数的次序",
                   table.get(probe) is not None and set(only_probe) == expected and len(only_probe) < len(names)
                   and list(only_probe) == [name for name in names if name in expected],
                   f"{probe} 点名 {table.get(probe)}，走的是 {only_probe}")
    selftest.check("表里没有的弄坏开关、没有开关都走全部格函数",
                   selftest_selected_cell_functions({BREAK_VARIABLES[0]: "no-such-break-switch"}) == names and selftest_selected_cell_functions({}) == names,
                   "不认得的开关或没有开关没走全部")
    selftest.check("几条开关走各自点名的格函数的并集",
                   set(selftest_selected_cell_functions({BREAK_VARIABLES[1]: probe + ",timingsignored"}))
                   == expected | set(table.get("timingsignored") or ()) | {"selftest_break_cell_table_cell"}, "两条开关没走并集")


def selftest_process_alive(pid):
    """进程 pid 还活着（读 /proc；僵尸不算，不在了也不算）。"""
    try:
        with open(f"/proc/{pid}/stat", "rb") as handle:
            text = handle.read().decode("utf-8", "replace")
    except OSError:
        return False
    return text[text.rindex(")") + 2:].split()[0] not in ("Z", "X")


def selftest_probe_sleeper(work):
    """调度探针用：起一个睡着的孙进程（不接标准输入输出），进程号记进 work/sleeper-pid。"""
    sleeper = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    selftest_write(os.path.join(work, "sleeper-pid"), f"{sleeper.pid}\n")


def selftest_probe_sleeper_pid(base, name):
    """调度探针 name 记下的孙进程号（base 是起它时给 selftest_fork_cell_functions 的目录）；没记下是 None。"""
    try:
        with open(os.path.join(base, "cell-" + name, "sleeper-pid"), encoding="utf-8") as handle:
            return int(handle.read().strip())
    except (OSError, ValueError):
        return None


def selftest_probe_process_state(selftest, work):
    """调度探针（不在格函数名单里，由 selftest_cell_scheduling_cells 经 selftest_fork_cell_functions 起）：子进程里 SIGTERM 取默认处理、自己是新进程组的头。"""
    selftest.check("探针：格函数子进程的 SIGTERM 取默认处理、自己是新进程组的头",
                   signal.getsignal(signal.SIGTERM) == signal.SIG_DFL and os.getpgrp() == os.getpid(),
                   f"SIGTERM 的处理 {signal.getsignal(signal.SIGTERM)}、进程组 {os.getpgrp()}、进程号 {os.getpid()}")


def selftest_probe_leftover_process(selftest, work):
    """调度探针：起一个睡着的孙进程就返回，不管它。"""
    selftest_probe_sleeper(work)
    selftest.check("探针：起了一个睡着的孙进程", True)


def selftest_probe_overtime(selftest, work):
    """调度探针：起一个睡着的孙进程，自己睡 40 秒才返回（用时上限给 2 秒）。"""
    selftest_probe_sleeper(work)
    time.sleep(40)
    selftest.check("探针：睡过了用时上限", True)


def selftest_probe_interval(selftest, work, companion):
    """调度探针：起来先记一个标记，等同时派出的另一个探针 companion 的标记也有了（至多 20 秒，不看机器忙闲两个起得差多久），再停 0.2 秒；起止时刻记进 work/interval。
    两个探针真同时在跑，两段时刻就重叠；一个一个跑的话先起的那个等不到对方，两段不重叠。"""
    started = time.time()
    selftest_write(os.path.join(work, "started"), "\n")
    companion_started = os.path.join(os.path.dirname(work), "cell-" + companion, "started")
    waiting_until = time.monotonic() + 20
    while not os.path.exists(companion_started) and time.monotonic() < waiting_until:
        time.sleep(0.02)
    time.sleep(0.2)
    selftest_write(os.path.join(work, "interval"), f"{started} {time.time()}\n")
    selftest.check("探针：跑了一段", True)


def selftest_probe_concurrent_interval(selftest, work):
    selftest_probe_interval(selftest, work, "selftest_probe_concurrent_interval_companion")


def selftest_probe_concurrent_interval_companion(selftest, work):
    selftest_probe_interval(selftest, work, "selftest_probe_concurrent_interval")


def selftest_probe_functions():
    """调度探针里由 selftest_fork_cell_functions 按名字起的那几个（函数体里的表；不算格函数：selftest_cell_function_names 不含它们，只有 selftest_cell_call 查）。"""
    return (selftest_probe_process_state, selftest_probe_leftover_process, selftest_probe_overtime,
            selftest_probe_concurrent_interval, selftest_probe_concurrent_interval_companion)


def selftest_probe_intervals_overlap(base, names):
    """调度探针 names 两个的 interval 有没有重叠：(重叠与否, 说明)；读不到记 (None, 原因)。"""
    spans = []
    for name in names:
        try:
            with open(os.path.join(base, "cell-" + name, "interval"), encoding="utf-8") as handle:
                begin, end = (float(part) for part in handle.read().split())
        except (OSError, ValueError) as error:
            return None, f"{name} 没留下起止时刻（{type(error).__name__}：{error}）"
        spans.append((begin, end))
    (left_begin, left_end), (right_begin, right_end) = spans
    return left_begin < right_end and right_begin < left_end, f"{names[0]} {left_begin:.2f}～{left_end:.2f}、{names[1]} {right_begin:.2f}～{right_end:.2f}"


def selftest_cell_scheduling_cells(selftest, work):
    """格函数并行的调度（文件头「--selftest 的挑格与并行」）：默认并行度（线程数与内存上限两者的小者）、要等对方起来的与久的先派、两个探针 jobs 给 2 时同时跑、派几个收几个；
    格函数子进程自己是进程组的头、SIGTERM 取默认处理、跑完核组里没留下进程（留下的停掉写成一格判错）、超过用时上限停整组并写明组里每个进程在等什么。
    探针（selftest_probe_*）经 selftest_fork_cell_functions 真起子进程跑，用时一共几秒；用时不随机器忙闲变：超时那一格的探针睡 40 秒、上限 2 秒、只核没等满 30 秒，同时跑那一格的两个探针各等对方起来再收尾、不比起得差多久。
    弄坏开关 cell-jobs-by-threads-only、cell-heaviest-first-ignored、cell-outcome-dropped、cell-child-inherits-parent-process-state、
    cell-leftover-processes-kept、cell-limit-ignored 各红其中一格（格名在文件头开关表）。"""
    environment = dict(os.environ)
    mib = 1024 * 1024
    cases = (((31, None), 31), ((31, 11420 * mib), 19), ((31, 4000 * mib), 5), ((31, 100 * mib), 1), ((4, None), 4))
    wrong = [f"线程 {arguments[0]}、上限 {arguments[1]} 得到 {selftest_cell_jobs_for(*arguments, environment)}（要 {expected}）"
             for arguments, expected in cases if selftest_cell_jobs_for(*arguments, environment) != expected]
    selftest.check("默认并行度：线程数与内存上限容得下的个数两者的小者（31 线程无上限 31、上限 11420M 19、上限 4000M 5、上限 100M 1、4 线程 4）",
                   not wrong, "；".join(wrong))
    # 默认并行度的内存取这一台能给的内存，不取服务交的小上限（弄坏开关 cell-jobs-by-task-cap）：这一台那一项 50G、服务交 4000M、线程 31：开 31 个（50G 容得下 97 个，卡在线程数）；
    # 多机配置没写这一项时退回服务交的 4000M：开 5 个（假 /proc/self/cgroup 读不到，memory.max 不收窄）
    memory_work = os.path.join(work, "scheduling-memory")
    cell_configuration = {name: os.path.join(memory_work, name, "multi-host.env") for name in ("with-slice", "without-slice")}
    for name, slice_line in (("with-slice", "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=50G 192.0.2.70=8G\n"), ("without-slice", "")):
        selftest_write(cell_configuration[name], "HOSTS=127.0.0.1 192.0.2.70\nCARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
                       "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n" + slice_line
                       + f"QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\n"
                       f"NFS_SHARED_ROOTS={os.path.join(memory_work, name)}\n")
    jobs_of = {}
    for name in cell_configuration:
        jobs_environment = dict(environment, SINGLEFS_THREAD_CAP="31", SINGLEFS_MULTI_HOST_CONFIG="", GATE_MUTATION_MULTI_HOST_CONFIG=cell_configuration[name],
                                GATE_MUTATION_CGROUP_LISTING=os.path.join(memory_work, "cgroup-listing-absent"), **{TASK_MEMORY_CAP_VARIABLE: str(4000 * mib)})
        jobs_of[name] = selftest_cell_jobs(jobs_environment)
    selftest.check("默认并行度按这一台能给的内存除，不按服务交的小上限除：多机配置这一台那一项 50G、服务交 4000M、31 线程开 31 个；多机配置没写这一项时退回服务交的 4000M 开 5 个",
                   jobs_of == {"with-slice": 31, "without-slice": 5}, f"得到 {jobs_of}")
    names = selftest_cell_function_names()
    named = set(selftest_heaviest_cell_functions()) | set(selftest_timing_sensitive_cell_functions())
    selftest.check("先派表与要等对方起来那张表点名的格函数都是自证的格函数", not named - set(names), f"不存在：{sorted(named - set(names))}")
    ordered = selftest_cell_dispatch_order(["selftest_probe_filler", "selftest_seed_cells", "selftest_anchor_and_impact_cells", "selftest_worker_slot_cells",
                                            "selftest_probe_other_filler"], environment)
    selftest.check("派出次序：要等对方起来的先，再久的（照先派表的次序），别的保持原次序",
                   ordered == ["selftest_worker_slot_cells", "selftest_anchor_and_impact_cells", "selftest_seed_cells", "selftest_probe_filler", "selftest_probe_other_filler"],
                   f"得到 {ordered}")
    base = os.path.join(work, "scheduling")
    probes = ["selftest_probe_process_state", "selftest_probe_leftover_process"]
    outcomes = selftest_fork_cell_functions(probes, base, selftest, environment, 2)
    selftest.check("派几个收几个：两个探针派出去、收回两个结果", sorted(outcomes) == sorted(probes), f"收回 {sorted(outcomes)}")
    selftest.check("格函数子进程：SIGTERM 取默认处理、自己是新进程组的头（进程池 terminate 它的工作进程时不卡死）",
                   outcomes.get("selftest_probe_process_state") == ("ok", (1, [], [])), repr(outcomes.get("selftest_probe_process_state")))
    leftover = outcomes.get("selftest_probe_leftover_process", ("error", "没有收回结果"))
    sleeper = selftest_probe_sleeper_pid(base, "selftest_probe_leftover_process")
    selftest.check("格函数跑完它那一组里留下的进程被停掉，写成一格判错",
                   leftover[0] == "ok" and any("还留着 1 个进程（已停）" in failure for failure in leftover[1][1]) and sleeper is not None and not selftest_process_alive(sleeper),
                   f"结局 {leftover}、孙进程 {sleeper} {'还活着' if sleeper is not None and selftest_process_alive(sleeper) else '已不在'}")
    started = time.monotonic()
    timed = selftest_fork_cell_functions(["selftest_probe_overtime"], os.path.join(work, "scheduling-overtime"), selftest, environment, 2, limit_seconds=2)
    elapsed = time.monotonic() - started
    outcome = timed.get("selftest_probe_overtime", ("error", "没有收回结果"))
    sleeper = selftest_probe_sleeper_pid(os.path.join(work, "scheduling-overtime"), "selftest_probe_overtime")
    selftest.check("格函数超过用时上限：停整组、写明停时组里每个进程在等什么，不无限等",
                   outcome[0] == "error" and "超过 2 秒" in outcome[1] and "time.sleep(60)" in outcome[1] and sleeper is not None and not selftest_process_alive(sleeper) and elapsed < 30,
                   f"结局 {outcome}、用了 {elapsed:.1f} 秒")
    pair = ["selftest_probe_concurrent_interval", "selftest_probe_concurrent_interval_companion"]
    selftest_fork_cell_functions(pair, os.path.join(work, "scheduling-concurrent"), selftest, environment, 2)
    together, together_text = selftest_probe_intervals_overlap(os.path.join(work, "scheduling-concurrent"), pair)
    selftest.check("两个格函数在 jobs 给 2 时同时跑（探针测得出重叠）", together is True, together_text)


def selftest_cell_call(name, selftest, work):
    """走一个格函数：名字对着 selftest_cell_functions 与 selftest_probe_functions 两张表取，都不在抛 KeyError。"""
    function = {function.__name__: function for function in selftest_cell_functions() + selftest_probe_functions()}[name]
    if name in selftest_cell_functions_without_work():
        function(selftest)
    else:
        function(selftest, work)


def selftest_cell_limits():
    """格函数的几个调度数：单个格函数用时上限秒（比任务服务判卡住的时限短）、主进程的占用、每多开一个子进程再占的内存。"""
    return {"function_seconds": 600, "base_bytes": 1280 * 1024 * 1024, "extra_bytes": 512 * 1024 * 1024}


def selftest_cell_jobs_for(threads, cap_bytes, environment):
    """格函数同时跑几个（文件头「--selftest 的挑格与并行」）：线程数与内存上限容得下的个数两者的小者，至少 1。
    内存上限容得下的个数 = (上限 − base_bytes) ÷ extra_bytes，向下取整；cap_bytes 是 None（没有内存上限这一项）不收。
    弄坏开关 cell-jobs-by-threads-only：只看线程数。"""
    if environment is not None and break_is(environment, "cell-jobs-by-threads-only"):
        return max(1, threads)
    limits = selftest_cell_limits()
    return memory_bounded_jobs(threads, cap_bytes, limits["base_bytes"], limits["extra_bytes"])


def selftest_cell_jobs(environment):
    """--jobs 没给时格函数同时跑几个：线程数取每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，取不到打一行说明、一个一个跑），
    内存取这一台能给的内存（host_memory_capacity），取不到时取这一条拿到的上限 task_memory_cap_bytes（读不了这一项不收，说明里点名），再交 selftest_cell_jobs_for；打一行依据。
    弄坏开关 cell-jobs-by-task-cap：只取这一条拿到的上限。"""
    try:
        threads = max(1, int(default_heavy_threads(environment)))
    except HeavyThreadDefaultError as error:
        note(f"  ! 取不到每台默认重活线程数（{error}），自证的格函数一个一个跑")
        return 1
    cap_bytes, cap_text, cap_label = None, "", "内存上限"
    try:
        if not break_is(environment, "cell-jobs-by-task-cap"):
            cap_bytes, cap_text = host_memory_capacity(environment)
            cap_label = "这一台能给的内存"
        if cap_bytes is None:
            cap_bytes, cap_text = task_memory_cap_bytes(environment)
            cap_label = "内存上限"
    except Exception as error:   # 多机配置读不了一类：这一项不收，不让并行度的取法拖垮整趟自证
        cap_bytes, cap_text = None, f"{type(error).__name__}：{error}"
    jobs = selftest_cell_jobs_for(threads, cap_bytes, environment)
    basis = (f"{cap_label} {format_size(cap_bytes)} 容得下 {max(1, (cap_bytes - selftest_cell_limits()["base_bytes"]) // selftest_cell_limits()["extra_bytes"])} 个"
             if cap_bytes is not None else f"没有内存上限这一项（{cap_text}）")
    note(f"  · 自证并行度 {jobs}：线程 {threads}、{basis}")
    return jobs


def selftest_heaviest_cell_functions():
    """并行时先派的格函数，按单个最久的在前；别的照 selftest_cell_function_names 的次序跟在后面（先派久的，整趟的尾巴最短）。
    次序照整份自证每个格函数那一行打的用时（要算键的 CPU 秒数多，起假 cargo 多的挂钟长）。"""
    return ("selftest_unit_key_cells", "selftest_reuse_fix_cells", "run_selftest_cells", "selftest_gpu_cell_dispatch_cells", "selftest_anchor_and_impact_cells",
            "selftest_memory_cells", "selftest_seed_cells", "selftest_signal_and_pipe_cells", "selftest_loaded_by_path_parallel_cell")


def selftest_cell_dispatch_order(names, environment):
    """names 派出去的次序：要等对方起来的（selftest_timing_sensitive_cell_functions）最先，再是 selftest_heaviest_cell_functions 里有的（照那张表的次序），
    别的保持原次序。弄坏开关 cell-heaviest-first-ignored：原次序。"""
    if break_is(environment, "cell-heaviest-first-ignored"):
        return list(names)
    sensitive, heaviest = selftest_timing_sensitive_cell_functions(), selftest_heaviest_cell_functions()
    return sorted(names, key=lambda name: (name not in sensitive, heaviest.index(name) if name in heaviest else len(heaviest), list(names).index(name)))


def selftest_group_members(group):
    """进程组 group 里此刻还活着的进程（读 /proc；僵尸不算）：[(进程号, 状态, 内核里等的地方, 命令行前 200 个字符)]。"""
    members = []
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat", "rb") as handle:
                text = handle.read().decode("utf-8", "replace")
            fields = text[text.rindex(")") + 2:].split()   # 状态、父进程、进程组……（进程名在括号里，可能带空格）
            if fields[0] in ("Z", "X") or int(fields[2]) != group:
                continue
            with open(f"/proc/{entry}/cmdline", "rb") as handle:
                command = handle.read().replace(b"\0", b" ").decode("utf-8", "replace").strip()[:200]
            try:
                with open(f"/proc/{entry}/wchan", "rb") as handle:
                    waiting = handle.read().decode("utf-8", "replace").strip() or "0"
            except OSError:
                waiting = "读不到"
        except (OSError, ValueError, IndexError):
            continue   # 读的时候进程退了
        members.append((int(entry), fields[0], waiting, command))
    return members


def selftest_stop_group(group):
    """停进程组 group（只给自证自己起的格函数子进程那一组）：先 TERM，过 3 秒还在再 KILL；交停之前组里的成员（selftest_group_members），空表是组里本来就没有。
    group 是自证自己的进程组时一个信号都不发。"""
    members = selftest_group_members(group)
    if not members or group == os.getpgrp():
        return members
    for signal_number, wait_seconds in ((signal.SIGTERM, 3.0), (signal.SIGKILL, 5.0)):
        with contextlib.suppress(ProcessLookupError, PermissionError):
            os.killpg(group, signal_number)
        deadline = time.monotonic() + wait_seconds
        while selftest_group_members(group) and time.monotonic() < deadline:
            time.sleep(0.05)
        if not selftest_group_members(group):
            break
    return members


def selftest_describe_members(members):
    return "；".join(f"进程 {pid}（状态 {state}、等在 {waiting}）{command}" for pid, state, waiting, command in members) or "（组里已经没有进程）"


def selftest_cell_child(name, work, parent, saved_environment):
    """fork 出来的子进程：自己当一个新会话与进程组的头（父进程跑完或超时对整组收尾，格函数起的孙进程一个不漏），SIGTERM 取默认处理
    （父进程为收尾装的处理器带进来，格函数里的进程池 terminate 它的工作进程时，工作进程在处理器里退不出来，进程池收尾卡死）；
    用自己的 Selftest 与目录 work/cell-<格函数名>/（假 cargo 记录、编译目录、运行记录各用各的）走一个格函数，
    结果 (格数, 判错的格, 本次未跑的格) 存进那个目录的 outcome.pickle 就 os._exit；什么都不往外抛。
    弄坏开关 cell-child-inherits-parent-process-state：不换进程组、不还原 SIGTERM。"""
    import pickle
    status = 1
    try:
        if not break_is(saved_environment, "cell-child-inherits-parent-process-state"):
            signal.signal(signal.SIGTERM, signal.SIG_DFL)
            os.setsid()
        child_work = os.path.join(work, "cell-" + name)
        os.makedirs(child_work)
        os.environ.clear()
        os.environ.update(saved_environment)
        child = Selftest(child_work)
        # Rust 词法单元的磁盘缓存：父进程已经算热的那一份拷过来，子进程不各算一遍
        for variable in ("FUNCTION_UNITS_LEX_CACHE_DIRECTORY",):
            if os.path.isdir(parent.base_environment[variable]):
                shutil.copytree(parent.base_environment[variable], child.base_environment[variable])
        os.environ.clear()
        os.environ.update(child.base_environment)
        try:
            selftest_cell_call(name, child, child_work)
            outcome = ("ok", (child.cells, child.failures, child.not_run))
        except BaseException:
            outcome = ("error", traceback.format_exc()[-1500:])
        path = os.path.join(child_work, "outcome.pickle")
        with open(path + ".tmp", "wb") as handle:
            pickle.dump(outcome, handle)
        os.replace(path + ".tmp", path)
        status = 0
    finally:
        with contextlib.suppress(Exception):
            sys.stdout.flush()
            sys.stderr.flush()
        os._exit(status)


def selftest_fork_cell_functions(names, work, parent, saved_environment, jobs, **options):
    """每个格函数一个 fork 出来的子进程（各自一个进程组，selftest_cell_child），至多 jobs 个同时跑；子进程退出由它的 pidfd 报（子进程起的孙进程拿着文件描述符也不影响）。
    交 {格函数名: ("ok", (格数, 判错的格, 本次未跑的格)) 或 ("error", 说明)}；收回的与派出的对不对得上由调用方核（selftest_merge_cell_outcomes）。
    · 派出的次序：selftest_cell_dispatch_order（要等对方起来的先，再久的）。
    · 每个格函数跑完核它那一组里没有留下进程：留下的停掉（selftest_stop_group），把它们写成这个格函数的一格判错。
    · 每个格函数至多跑 limit_seconds（不给取 selftest_cell_limits 的 function_seconds）：超了先记下组里每个进程在等什么，再停整组，记 error；父进程不无限等。
    · 收尾时还没退的格函数同样停整组再等。
    每个格函数跑完往 stderr 打一行：用时、CPU 秒数、峰值内存（子进程连它等过的孙进程的 ru_maxrss）。
    弄坏开关 cell-leftover-processes-kept：跑完不核组里有没有留下进程；cell-limit-ignored：不看用时上限；
    cell-outcome-dropped：names 里排在最后的那一个的结果交回之前丢掉。"""
    import pickle
    import selectors
    limit_seconds = options.get("limit_seconds")   # 只收这一个选项
    limit = selftest_cell_limits()["function_seconds"] if limit_seconds is None else limit_seconds
    if break_is(saved_environment, "cell-limit-ignored"):
        limit = None
    pending, running, outcomes = selftest_cell_dispatch_order(names, saved_environment), {}, {}
    selector = selectors.DefaultSelector()
    previous_handler = signal.signal(signal.SIGTERM, lambda _number, _frame: sys.exit(143))

    def finish(descriptor, overtime):
        name, pid, started = running.pop(descriptor)
        selector.unregister(descriptor)
        os.close(descriptor)
        problem = None
        if overtime:
            members = selftest_group_members(pid)
            problem = f"超过 {limit} 秒没跑完，已停它整组；停时组里的进程：{selftest_describe_members(members)}"
            selftest_stop_group(pid)
            with contextlib.suppress(ProcessLookupError):
                os.kill(pid, signal.SIGKILL)   # 子进程没来得及换进程组（组不存在）时只停它自己
        _pid, status, usage = os.wait4(pid, 0)
        leftovers = [] if overtime or break_is(saved_environment, "cell-leftover-processes-kept") else selftest_stop_group(pid)
        path = os.path.join(work, "cell-" + name, "outcome.pickle")
        if problem is not None:
            outcomes[name] = ("error", problem)
        else:
            try:
                with open(path, "rb") as handle:
                    outcomes[name] = pickle.load(handle)
            except (OSError, EOFError, pickle.UnpicklingError) as error:
                outcomes[name] = ("error", f"子进程退出状态 {status}，没留下结果（{type(error).__name__}：{error}）")
        if leftovers and outcomes[name][0] == "ok":
            cells, failures, not_run = outcomes[name][1]
            outcomes[name] = ("ok", (cells + 1, failures + [f"格函数 {name} 跑完，它那一组里还留着 {len(leftovers)} 个进程（已停）：{selftest_describe_members(leftovers)}"], not_run))
        elif leftovers:
            outcomes[name] = ("error", f"{outcomes[name][1]}；它那一组里还留着 {len(leftovers)} 个进程（已停）：{selftest_describe_members(leftovers)}")
        cells_text = f"{outcomes[name][1][0]} 格" if outcomes[name][0] == "ok" else "出错"
        note(f"  · 自证格函数 {name}：{time.monotonic() - started:.0f} 秒（{cells_text}），CPU {usage.ru_utime + usage.ru_stime:.0f} 秒、峰值内存 {usage.ru_maxrss // 1024} MiB")

    try:
        while pending or running:
            while len(running) < jobs and pending:
                name = pending.pop(0)
                sys.stdout.flush()
                sys.stderr.flush()
                pid = os.fork()
                if pid == 0:
                    selftest_cell_child(name, work, parent, saved_environment)   # 不返回
                descriptor = os.pidfd_open(pid)
                running[descriptor] = (name, pid, time.monotonic())
                selector.register(descriptor, selectors.EVENT_READ)
            deadlines = [started + limit for _name, _pid, started in running.values()] if limit is not None else []
            wait = max(0.0, min(deadlines) - time.monotonic()) if deadlines else None
            for key, _events in selector.select(timeout=wait):
                finish(key.fd, False)
            if limit is not None:
                for descriptor, (_name, _pid, started) in list(running.items()):
                    if time.monotonic() - started >= limit:
                        finish(descriptor, True)
    finally:
        for descriptor, (name, pid, _started) in list(running.items()):
            selftest_stop_group(pid)
            with contextlib.suppress(ProcessLookupError):
                os.kill(pid, signal.SIGKILL)
            with contextlib.suppress(ChildProcessError):
                os.waitpid(pid, 0)
            os.close(descriptor)
        selector.close()
        signal.signal(signal.SIGTERM, previous_handler)
    if break_is(saved_environment, "cell-outcome-dropped") and names:
        outcomes.pop(names[-1], None)
    return outcomes


def selftest_run_cell_functions(selftest, work, names, jobs, saved_environment):
    """走这些格函数，结果并进 selftest（格数、判错的格、本次未跑的格）；交这一趟同时跑了几路。jobs 是 1 或只有一个格函数：在本进程里按次序走，同一个 selftest、同一个 work；
    不然全部格函数进同一个派活池（要等对方起来的排在最前，selftest_cell_dispatch_order），至多 jobs 个同时跑，结果按 names 的次序并进来（selftest_merge_cell_outcomes）。"""
    if jobs <= 1 or len(names) <= 1:
        for name in names:
            selftest_cell_call(name, selftest, work)
        return 1
    selftest_merge_cell_outcomes(selftest, names, selftest_fork_cell_functions(list(names), work, selftest, saved_environment, jobs))
    return min(jobs, len(names))


def selftest_merge_cell_outcomes(selftest, names, outcomes):
    """收回的结果按 names（登记次序）并进 selftest：格数、判错的格、本次未跑的格逐个加，没跑成的记一格判错。
    派出去几个就要收回几个：收回的名字与派出的对不上（缺的、多的），另记一格判错、逐个点名。"""
    missing = [name for name in names if name not in outcomes]
    extra = sorted(set(outcomes) - set(names))
    selftest.check("派出去的格函数都收回了结果（个数与名字都对得上）", not missing and not extra,
                   f"派出去 {len(names)} 个、收回 {len(outcomes)} 个" + (f"；缺 {'、'.join(missing)}" if missing else "") + (f"；多 {'、'.join(extra)}" if extra else ""))
    for name in names:
        kind, value = outcomes.get(name, ("error", "没有收回结果"))
        if kind == "ok":
            cells, failures, not_run = value
            selftest.cells += cells
            selftest.failures.extend(failures)
            selftest.not_run.extend(not_run)
        else:
            selftest.cells += 1
            selftest.failures.append(f"格函数 {name} 没跑成：{value}")


def selftest_options(arguments):
    """--selftest 后面的选项：--cells <格函数名>[,…]（只走点名的）、--jobs <正整数>（同时跑几个格函数，1 是在本进程里一个一个跑）。
    交 (点名的格函数或 None, 并行数或 None, 拒绝的原因)。"""
    names, jobs, rest = None, None, list(arguments)
    while rest:
        option = rest.pop(0)
        value = rest.pop(0) if rest else ""
        if option == "--cells":
            names = value.split(",")
            unknown = [name for name in names if name not in selftest_cell_function_names()]
            if not value or "" in names or unknown:
                return None, None, f"--cells 后面要写 <格函数名>[,…]（给的是「{value}」，认不出 {unknown}）；可用的：{'、'.join(selftest_cell_function_names())}"
        elif option == "--jobs":
            if not value.isdigit() or int(value) < 1:
                return None, None, f"--jobs 后面要写正整数（给的是「{value}」）"
            jobs = int(value)
        else:
            return None, None, f"--selftest 后面只认 --cells、--jobs（给的是「{option}」）"
    return names, jobs, ""


def selftest_fixture_values():
    """自证夹具（文件头同名一段）：按路径载入 SELFTEST_FIXTURES_PATH，交它的 FIXTURE_VALUES（名字 → 值）。
    载入的那一份不登记进 sys.modules（夹具全是常量，没有函数要按名字找回）。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_selftest_fixtures", SELFTEST_FIXTURES_PATH)
    fixtures = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(fixtures)
    return fixtures.FIXTURE_VALUES


FIXTURE_MODULES = (crates_mutation_rows_support, crates_mutation_rows_static, crates_mutation_rows_units, crates_mutation_rows_baseline, crates_mutation_rows_targets, crates_mutation_rows_memory, crates_mutation_rows_runs, crates_mutation_rows_pool, crates_mutation_rows_plan, crates_mutation_rows_stage, crates_mutation_rows_commands,)


def load_selftest_fixtures():
    """自证夹具（文件头同名一段）：把 selftest_fixture_values 交的名字放进用到它们的每一份模块（本份与 FIXTURE_MODULES 里的各份）。只在自证里调。"""
    values = selftest_fixture_values()
    globals().update(values)
    for module in FIXTURE_MODULES:
        vars(module).update(values)


def run_selftest(arguments):
    load_selftest_fixtures()
    named, jobs, refusal = selftest_options(arguments)
    if refusal:
        print(f"  ✗ crates-mutation-rows.py --selftest 的用法不对：{refusal}")
        print("     → 怎么办：--selftest [--cells <格函数名>[,…]] [--jobs <N>]；格函数名取上面列的，要走全部就不给 --cells")
        return 2
    # 自证不用服务给这一次尝试的编译目录、不用调用方给的卡串：编译目录（CARGO_TARGET_DIR）与卡串从本进程的环境里去掉
    #   （Selftest 与 fork 出来的格函数子进程都照本进程的环境搭；上卡的格与服务给的编译目录那两个格函数自己设假的）
    for name in ("CARGO_TARGET_DIR", "GPU_CARDS_BY_PRIORITY"):
        os.environ.pop(name, None)
    work = tempfile.mkdtemp(prefix="crates-mutation-rows-selftest-")
    saved_environment = dict(os.environ)
    parallel_ways = 1
    try:
        selftest = Selftest(work)
        # Rust 词法单元的磁盘缓存先指到这一趟自证自己的目录，不碰真仓那一份
        os.environ["FUNCTION_UNITS_LEX_CACHE_DIRECTORY"] = selftest.base_environment["FUNCTION_UNITS_LEX_CACHE_DIRECTORY"]
        # 进程内直接调的几条命令（row-plan、import 这类，拿 os.environ）同样不读真仓的多机配置、不碰真共享根（文件头「判定库」末段）
        os.environ["GATE_MUTATION_RUN_RECORDS_DIRECTORY"] = selftest.base_environment["GATE_MUTATION_RUN_RECORDS_DIRECTORY"]
        # plan 与 library-directory 读本进程的环境：与 run_stage 那一份对齐
        os.environ.clear()
        os.environ.update(selftest.base_environment)
        # 格函数的挑法与并行（文件头「--selftest 的挑格与并行」）：--cells 点名的只走点名的；不然有弄坏开关就只走开关点名的格函数，
        #   点名的格函数都没红再把其余的走完（表点名错了的出路写在第一行）；没有开关走全部。--jobs 没给取每台默认重活线程数
        every_name = selftest_cell_function_names()
        names = tuple(name for name in every_name if name in named) if named else selftest_selected_cell_functions(saved_environment)
        jobs = jobs or selftest_cell_jobs(saved_environment)
        if jobs > 1 and len(names) > 1:
            note(f"  · 自证并行：{len(names)} 个格函数，每个一个子进程，至多 {jobs} 个同时跑")
        parallel_ways = selftest_run_cell_functions(selftest, work, names, jobs, saved_environment)
        if not named and len(names) < len(every_name) and not selftest.failures:
            rest = tuple(name for name in every_name if name not in names)
            parallel_ways = max(parallel_ways, selftest_run_cell_functions(selftest, work, rest, jobs, saved_environment))
            if selftest.failures:
                selftest.failures.insert(0, f"弄坏开关 {'、'.join(selftest_break_switches(saved_environment))} 点名的格函数 {'、'.join(names)} 一格都没红，红在点名之外的格函数里："
                                            "改 selftest_break_cell_table 里这几条开关的行，补上下面判错的格所在的格函数")
    finally:
        os.environ.clear()
        os.environ.update(saved_environment)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ crates-mutation-rows.py 自证有 {len(selftest.failures)} 格判错（共 {selftest.cells} 格）：")
        for failure in selftest.failures:
            print(f"      {failure}")
        print("     → 怎么办：设了 GATE_MUTATION_BREAK 的，这就是要的结果（那个开关把对应的判法弄坏了）；没设的，照上面那几格去看 run_stage、row_decision、")
        print("       KeyContext.components、baseline_reuse_problem、judge_coverage、dispatch_order、command_plan、escaping_literals、read_table 与 row_problems；")
        print("       单元切分与闭包看 scripts/mutation/lib_mutation_units.py 与 scripts/gate/lib_function_units.py；起 cargo 那一段看 run_cargo_command。")
        for item in selftest.not_run:
            print(f"      本次未跑：{item}")
        return 1
    if selftest.not_run:
        # 有格的前提没造出来（skip）：不判绿，退 77（本次未跑）
        print(f"  · crates-mutation-rows.py 自证：判了的 {selftest.cells} 格都对，本次未跑 {len(selftest.not_run)} 格（前提没造出来，不算通过）：")
        for item in selftest.not_run:
            print(f"      {item}")
        print("     → 怎么办：机器闲一点（/proc/pressure/io 的 some 低下来）再跑一遍 --selftest；还是本次未跑就照那一格写的原因看造前提的那几步")
        return 77
    if named:
        print(f"  ✓ crates-mutation-rows.py 自证（只走 --cells 点名的 {len(named)} 个格函数：{'、'.join(named)}，不是整份）{selftest.cells} 格都对，并行 {parallel_ways} 路")
        return 0
    print(f"  ✓ crates-mutation-rows.py 自证 {selftest.cells} 格都对，并行 {parallel_ways} 路（复用、改一行只重跑那一行、跑到一半续跑、基线不绿、"
          "久的先派、选行只判一部分退 3、多久没被引用才删、记录里没有主机名、拷贝范围两样（include_str!、include_bytes! 的实参照源文件所在目录解析）、"
          "替换文空着的删行变异照常判、派活之前那几样 crates-mutation-replay 与 static-check 同一份判法、"
          "点名的测试打了 NOT_RUN 记本次未跑且变异那一趟补了 --show-output、只有本次未跑的退 77、should panic 的用例红了判抓到、"
          "单元键：被改坏函数的别一行变只现判原文在它里面的行（S34）、改名一行都不现判（N34）、"
          "同文件加没人调的函数一行都不现判（N45）、WGSL 按 fn 取键（改另一个 fn 键不变、改被改坏的 fn 键变、原文只在注释里的拒算）、改点名测试调到的函数与被改坏函数用的常量现判点名它的行、基线按执行过的单元复用而不按整树快照、"
          "驱动与算键库的代码、编译档、线程数都不进键（改了行键都不变）、拒算的照跑、"
          "撞键拒算（内容相同的两个函数各改一处、点名同一条测试的两行同键，两趟都拒算照跑、选行时照整张表判）、基线键与覆盖号带包与测试目标、一行的基线结局不随这一批拼进了哪几行而变、"
          "记红撤绿（同一把键改判没红或基线不绿之后下一趟现判）、不算数的结局不盖同一把键的算数结论、被判的树不是 git 工作树一条都不跑、"
          "按项机读行（--list-items --with-keys 每项一行 ITEM_PLAN 且条数对得上合计、不起 cargo；真跑每项恰好一行 ITEM_DONE、全复用的一趟也是、复用的 0 秒、现判的各项各记各的挂钟）、"
          "工作进程数取每台默认重活线程数（或 GATE_MUTATION_WORKERS）、要现跑的条数与内存上限 ÷ 那个编译并行度下每个工作进程的实测峰值的小者、取不到登记值就拒、没有内存上限这一项时照线程数、"
          "工作进程量自己整棵进程树的峰值、按编译并行度记下读得回来、只看最新的几份，编译并行度跟着进程数走时算到自洽，从没量过的包不把整趟收成 1 个而是头一格单独跑、量到之后按实测放开，记录读不了取保守值只开 1 个、"
          "拷贝范围含 .cargo/（点名的测试要编 gpu 特性的那一行在副本里读得到 [env]）、"
          "产品 GPU 包的 tests-on-a-real-gpu 与 gpu 同样当上卡的特性认（真跑测试的那几次收尾带 --test-threads 1、没卡打 NOT_RUN 记本次未跑）、不另套清场、"
          "真跑测试的每一次在副本里收到本模块所在仓的多机配置（只编不跑的收不到）、多机配置不在起 cargo 之前就拒，带 gpu 特性而给定状态全交 CPU 判的没红记本次未跑而 GPU 判过的照常判、"
          "被 TERM 停时删掉 $TMPDIR 下的 singlefs-mutation-replay-*（TERM 落在建目录那一瞬也不漏；进程池还没起就被停照样退 143、不抛 AttributeError）、"
          "片号大于这一趟工作进程数的 -w<片号> 开跑时回收而有进程在用的不删、两个变量都没设时编译目录不落 /tmp、"
          "同一个共用编译目录上并发的两次调用领到的片号集合不相交（片号锁）而片号锁被占着的 -w<片号> 回收时留着、"
          "--list-items 接 | head 时安静退、"
          "点名的测试在这一组特性下不编时 run_stage 与 --list-items 打真错并点名变异表的行（并行算与就地算一样）、行里已带 --exact 的变异记覆盖时 --exact 只出现一次、"
          "本次未跑认项名（test=crash_state_enumeration::<项名> 对不上测试名也记本次未跑）、开 GPU 时点名测试走崩溃判段的行补 gpu 特性起且键照补了之后的特性算、"
          "命中重测试目标的行（crash-case 目标、gpu 特性）补 --release 起而不命中的不补、static-check 报命中而第五段没写 --release 的行、"
          "改 rustc、cargo 版本行键不变而改编译器的环境变量全现判、不按时限过期、llvm 工具不对判红、plan 只读且逐栏报数、"
          "锚点失效按行判（那一行记锚点失效、不入库、整道判红，别的行照跑照入库）而 row-plan、plan、--list-items 把锚点失效的行单列（数据文件那一行也不算键）、"
          "牵连面在样本 crate 上报对牵连最大的单元与行数、"
          "static-check 可达性判点名测试够不着被改坏的函数（开 GPU 时只在 CPU 判器入口后面的报 gpu-route、GPU 关时不报，闭包里根本没有的报 closure，"
          "测试自己写 CPU 配置的与 GPU 许建不出判表的豁免，测试直接调 CPU 判器入口的放行，截断表里的函数找不到报配置错）、"
          "磁盘余量放不下时工作进程数按余量收（已有的编译目录只算还能涨的、余量写法不认整道拒、没给余量读编译目录所在的盘）、"
          "编译目录超过上限时判完一格清掉增量产物、"
          "起的每条 cargo 都关了调试信息、开了 release 增量而这两项不进行键（别的构建档取值照进）、几个工作进程时外部依赖只在先编那一份里编一次且只拷外部依赖的产物、先编那一份的源码按内容同步而摸过的包没重编就判红、"
          "拷依赖产物分到几条线程上同时拷、"
          "几个工作进程时基线的批同时在跑的不少于工作进程数、磁盘余量压到 1 个时同时在跑的基线至多 1 批、"
          "变异与插桩跑各连自己的一份临时 prune-server 而真服务一条没多、基线连调用方的那一份且起的 cargo 不带 release 增量覆盖、不在临时服务底下设了只给自证换的变量拒跑、"
          "变异行带 --manifest-path 点工作区之外的小工程时拷贝范围补进那个目录、基线与变异照原样带 --manifest-path 起、单元键照那个包的源码算而指到仓外的拒、"
          "GPU 阶段现场行边跑边滤、工作进程内存有界而判定与不滤时逐字相同、被停时留下的最后 32 行进日志、照固定间隔打进度行（不出输出也打、不丢）、"
          "重基线批按过滤词切块而每行判定与不切时逐字相同、被整组硬杀之后重跑只补没判的块、"
          "算键的子进程不各抄父进程的工作区（fork 之前共用词法单元、先算一条、冻进永久代，闭包的键不变）、派工作进程之前放掉算键的上下文而判定与覆盖不变、"
          "上卡的格一格接一格（不领卡、不另编，环境里给的卡串原样交，不上卡的格看不到卡）、服务给的编译目录照用不删）")
    return 0


def run_selftest_cells(selftest, work):
    repository = os.path.join(work, "repository")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    # 一：头一趟全跑；第二趟输入不变，假 cargo 一次都不起、判定逐字相同
    first = selftest.run(repository)
    selftest.check("头一趟判绿", first[0] == 0, first[1] + first[2])
    selftest.check("头一趟起 1 批基线 + 4 条变异，另记 4 份覆盖（各一次插桩编译、一次插桩跑）",
                   len(plain_invocations(first[3])) == 5 and len(coverage_invocations(first[3])) == 8, "\n".join(first[3]))
    second = selftest.run(repository)
    selftest.check("第二趟输入不变：cargo 一次都不起", second[0] == 0 and second[3] == [], "\n".join(second[3]))
    selftest.check("第二趟判定与头一趟逐字相同（只差复用的报数）", selftest_verdict_view(second[1]) == selftest_verdict_view(first[1])
                   and "现判 0 条、复用 4 条、拒算 0 条、本次未跑 0 条" in second[1], second[1])
    selftest.check("第二趟 stderr 头一行报复用 4 条", "复用抓到 4 条" in (second[2].splitlines() or [""])[0], second[2])
    # 起 cargo 之前判 RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：同一份小仓另放一份依赖 rocksdb 的 验证包清单、
    #   进程环境里四个取值是空的，PATH 上的假 cargo 不带 entry 认的标记 ⇒ 退 1、假 cargo 一次都不起、报出 entry 的红句
    rocksdb_repository = os.path.join(work, "rocksdb-repository")
    shutil.copytree(repository, rocksdb_repository, symlinks=True)
    selftest_write(os.path.join(rocksdb_repository, "crates", "singlefs-verification", "Cargo.toml"),
                   '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
    refused_by_rocksdb = selftest.run(rocksdb_repository, ROCKSDB_LIB_DIR="", ROCKSDB_STATIC="", SNAPPY_LIB_DIR="", SNAPPY_STATIC="")
    selftest.check("RocksDB 预编库的取值缺了：起 cargo 之前判红、退 1、假 cargo 一次都不起",
                   refused_by_rocksdb[0] == 1 and refused_by_rocksdb[3] == [] and "rocksdb-prebuilt entry：" in refused_by_rocksdb[1] + refused_by_rocksdb[2],
                   refused_by_rocksdb[1] + refused_by_rocksdb[2])
    shutil.rmtree(rocksdb_repository)
    # 二：进程数不同，换一棵树（库是新的一份，全部现判），判定仍逐字相同
    parallel = selftest.run(selftest_new_library_copy(work, repository, "parallel-repository"), GATE_MUTATION_WORKERS="4")
    selftest.check("GATE_MUTATION_WORKERS=4 与 =1 的判定逐字相同", selftest_verdict_view(parallel[1]) == selftest_verdict_view(first[1]), parallel[1])
    selftest.check("换一棵树（库是新的一份）：1 批基线 + 4 条变异全部现判，报「行新加」那一栏", len(plain_invocations(parallel[3])) == 5
                   and "行新加 4" in parallel[1], parallel[1] + parallel[2])
    unlimited = selftest.run(repository, SINGLEFS_REUSE_HOURS="0")
    selftest.check("SINGLEFS_REUSE_HOURS=0 照样复用（记录不按时限过期）", unlimited[0] == 0 and unlimited[3] == [], "\n".join(unlimited[3]))
    # 三：只改一行的替换文：只那一行现判（栏是原文或替换文变了），它点名测试的基线复用（基线键与点名测试都没变），只起那一条变异
    table_path = os.path.join(repository, TABLE)
    replace_in_file(table_path, "{ value * 3 }", "{ value * 4 }")
    one_row = selftest.run(repository)
    changed = mutation_invocations(plain_invocations(one_row[3]), pristine)
    selftest.check("只改一行的替换文：只那一行现判，栏是原文或替换文变了，它的基线复用", one_row[0] == 0 and len(plain_invocations(one_row[3])) == 1 and len(changed) == 1
                   and "doubles" in changed[0] and per_row_how(one_row[1]).get("double-becomes-triple") == "现判：原文或替换文变了",
                   one_row[1] + "\n".join(one_row[3]))
    # 四：改一行注释（锚点照旧命中）：单元指纹不变，全部复用
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "pub fn add(", "// 改了一行注释\npub fn add(")
    pristine = source_hash(repository)
    comment_changed = selftest.run(repository)
    selftest.check("改一行注释：单元指纹不变，全部复用、cargo 一次都不起", comment_changed[0] == 0 and comment_changed[3] == []
                   and "现判 0 条、复用 4 条" in comment_changed[1], comment_changed[1] + "\n".join(comment_changed[3]))
    # 五：跑到一半停掉（只供测试的开关），下一趟只跑剩下的；基线已判过 ok 的不再跑。从一份空库起：prune-server 没有删的操作，
    # 换一棵树：仓整份拷一份（连同它自己的 .git，git common-dir 不同，库的命名空间是新的一份）
    halfway_repository = os.path.join(work, "halfway-repository")
    shutil.copytree(repository, halfway_repository, symlinks=True)
    halfway = selftest.run(halfway_repository, GATE_MUTATION_STOP_AFTER_ROWS="2")
    selftest.check("跑完 2 条就退：判红、说明是只供测试的开关", halfway[0] == 1 and "GATE_MUTATION_STOP_AFTER_ROWS=2" in halfway[1]
                   and len(mutation_invocations(halfway[3], pristine)) == 2, halfway[1] + "\n".join(halfway[3]))
    resumed = selftest.run(halfway_repository)
    selftest.check("下一趟只跑剩下的 2 条、基线不再跑，判完的 2 条复用；给这一趟现判的 2 条测试补记覆盖",
                   resumed[0] == 0 and len(plain_invocations(resumed[3])) == 2 and len(mutation_invocations(resumed[3], pristine)) == 2
                   and len(coverage_invocations(resumed[3])) == 4, "\n".join(resumed[3]))
    # 六：上一次跑得久的先派，按用时从长到短；stderr 头一行报前移几条（用时表在运行记录目录里，文件头「判定库」末段）
    ordered_repository = selftest_new_library_copy(work, repository, "ordered-repository")   # 换一棵树：库是新的一份，6 条变异都要现判才有派活次序
    timings_directory = os.path.join(run_records_directory(ordered_repository, dict(selftest.base_environment)), "timings")
    table_rows, _malformed = read_table(table_path)
    long_rows = {"negates": "400", "halves": "1800"}
    for row in table_rows:
        if row.expected in long_rows:
            write_atomically(os.path.join(timings_directory, row_content_hash(row)),
                             mutation_units.record_text(RECORD_HEADER, dict(seconds_running=long_rows[row.expected])))
    ordered = selftest.run(ordered_repository)
    order = [line.split("\t")[0].split()[-1] for line in mutation_invocations(ordered[3], pristine)]
    ordered_head_line = (ordered[2].splitlines() or [""])[0]
    selftest.check("上一次跑得久的先派（按用时从长到短）", order[:2] == ["halves", "negates"] and "前移 2 条" in ordered_head_line,
                   f"派活次序 {order}；{ordered_head_line}")
    # 七：选行只判一部分：都抓到退 3、末行写明不是整道结论
    selection_file = os.path.join(work, "selection.txt")
    selftest_write(selection_file, "".join(f"{row.line_number}\n" for row in table_rows[:2]))
    selected_repository = selftest_new_library_copy(work, repository, "selected-repository")   # 换一棵树：库是新的一份，选中的 2 行现判
    selected = selftest.run(selected_repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行：都抓到退 3，末行写明只判了选中的 2 行", selected[0] == EXIT_SELECTION_ONLY
                   and "只判了选中的 2 行，不是整道结论" in selected[1].splitlines()[-1] and len(mutation_invocations(selected[3], pristine)) == 2,
                   selected[1])
    reused_selected = selftest.run(selected_repository, GATE_MUTATION_ROW_SELECTION=selection_file)
    selftest.check("选 2 行再跑一趟：照判定库复用，与头一趟现判出来的那两行的判定相同", reused_selected[0] == EXIT_SELECTION_ONLY
                   and reused_selected[3] == [] and "复用 2 条" in reused_selected[1], reused_selected[1])
    selftest_write(os.path.join(work, "bad-selection.txt"), "99999\n")
    bad_selection = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=os.path.join(work, "bad-selection.txt"))
    selftest.check("选行文件里的行号不是一条变异：判红、一格都没跑", bad_selection[0] == 1 and bad_selection[3] == [], bad_selection[1])
    # 判定库那一份（九 扫它）
    source_library = library_of(repository)
    # 九：记录里不写主机名（判定库记录与解开的覆盖记录、组成清单；整份库经临时服务按前缀扫出来），判定库目录本地一个文件都没有
    host_name = os.uname().nodename
    record_texts = [gzip.decompress(data).decode("utf-8", "replace") if relative.endswith(".json.gz") else data.decode("utf-8", "replace")
                    for relative, data in selftest_library(source_library).service.scan("")]
    local_files = [name for _walked, _directories, names in os.walk(source_library) for name in names]
    selftest.check("记录里不写主机名", len(host_name) >= 3 and record_texts and not any(host_name in text for text in record_texts) and not local_files,
                   f"{host_name}；库里 {len(record_texts)} 条；库目录本地的文件 {local_files[:3]}")
    # 十一：基线不绿（点名的测试读一个副本里没有的文件）：不算抓到、整道判红；弄坏开关 nobaseline 下误判成抓到
    outside = os.path.join(work, "outside")
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS + [SELFTEST_READS_OUTSIDE_ROW], SELFTEST_READS_OUTSIDE_TEST)
    baseline_red = selftest.run(outside)
    selftest.check("点名的测试读副本里没有的文件：记基线不绿、不算抓到、整道判红", baseline_red[0] == 1 and "基线不绿 1 行" in baseline_red[1]
                   and "reads-outside：基线不绿——点名的测试 reads_answer 在没改坏的副本上的结果是 FAILED" in baseline_red[1]
                   and "计数：抓到 4 条" in baseline_red[1], baseline_red[1])
    # 十二：每片的副本不留原来的修改时刻（留着的话，跨轮复用的编译目录里上一轮改坏的二进制会被 cargo 当成新的，基线跑的是改坏的代码）
    stale_source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    os.utime(stale_source, (1000, 1000))
    shard_root = os.path.join(work, "shard-root")
    os.makedirs(shard_root)
    shard_work, _target = prepare_shard(1, shard_root, dict(copied=["crates", "Cargo.toml"], root=repository, shared_target_directory=os.path.join(work, "t")))
    copied_mtime = os.stat(os.path.join(shard_work, "crates", "tiny", "src", "lib.rs")).st_mtime
    selftest.check("每片的副本不留原来的修改时刻（取拷的那一刻）", copied_mtime > 1000 + 3600, f"副本的修改时刻 {copied_mtime}")
    # 十三：拷贝范围两样（code-source-discipline 调的那一份判法）
    copied = ["crates", "Cargo.toml", "Cargo.lock", "litmus"]
    selftest.check("拷贝范围：litmus/ 下的文件在、docs/ 下的不在、target 底下的不在",
                   is_copied_into_each_shard("litmus/a.litmus", copied) and not is_copied_into_each_shard("docs/a.rs", copied)
                   and not is_copied_into_each_shard("crates/x/target/debug/a", copied) and not is_copied_into_each_shard("../crates/a", copied))
    scope_root = os.path.join(work, "scope")
    selftest_write(os.path.join(scope_root, "crates", "demo", "tests", "paths.rs"),
                   '#[path = "../../outside/mod.rs"]\nmod outside;\n// "../../in-a-comment/x"\n/* "../../in-a-block/x" */\n'
                   'const A: &str = "../../litmus";\nconst B: &str = r#"../../fixture-data/x"#;\nconst C: &str = "../other-crate/src";\n'
                   'const D: &str = "../../";\nlet e = b"../../bytes/x";\n')
    literals, scanned = escaping_literals(scope_root, copied, 1, os.environ)
    outside_literals = sorted((line, resolved) for _file, line, _literal, resolved, inside in literals if not inside)
    inside_literals = [resolved for _file, _line, _literal, resolved, inside in literals if inside]
    selftest.check("逃出 crates/ 的字面量：#[path]、注释、crates/ 之内的不算，litmus 在范围里，原始字符串、字节串、指到仓根的判不在",
                   scanned == 1 and inside_literals == ["litmus"] and outside_literals == [(6, "fixture-data/x"), (8, "."), (9, "bytes/x")],
                   f"在范围里 {inside_literals}；不在 {outside_literals}")
    # 十四：替换文空着的删行变异：成形、照常改坏源码（原文换成空串）、判抓到；弄坏开关 emptyreplacementrefused 下判不成形、这一格判错
    deletion = os.path.join(work, "deletion")
    build_selftest_repository(deletion, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW])
    deleted = selftest.run(deletion)
    deleted_hash = hashlib.sha256(SELFTEST_SOURCE.replace("{ value / 2 }", "", 1).encode()).hexdigest()[:12]
    selftest.check("替换文空着的删行变异：成形、原文换成空串去跑、判抓到", deleted[0] == 0 and "✓ halve-deleted：halves 红了" in deleted[1]
                   and any(line.endswith("\t" + deleted_hash) for line in deleted[3]), deleted[1] + "\n".join(deleted[3]))
    # 十五：派活之前那几样只有一份判法：static-check（门禁 code-source-discipline 调它）与 run_stage 的 prechecks 对同一张表报同样几行，删行变异（第 6 行）两边都不报
    static_root = os.path.join(work, "static")
    build_selftest_repository(static_root, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW] + [row for _kind, row in SELFTEST_STATIC_PROBLEM_ROWS])
    selftest_write(os.path.join(static_root, "docs", "outside.rs"), "pub fn outside() {}\n")
    expected_problems = [(kind, 7 + index) for index, (kind, _row) in enumerate(SELFTEST_STATIC_PROBLEM_ROWS)]
    static = run_command_capturing(command_static_check, [static_root])
    reported = [(line.split("\t")[0], int(line.split("\t")[1])) for line in static[1].splitlines() if line.split("\t")[0] in ROW_PROBLEM_KINDS]
    selftest.check("static-check：不成形、锚点、测试名、拷贝范围、转义五样各报一行，删行变异不报", static[0] == 1 and reported == expected_problems,
                   static[1])
    # static-check 的失败那一路：缺 <根> 时 ERROR 行写原因、下一行给照抄的「→ 怎么办」；--table 指的文件不在时 ERROR 退 2，不报「CHECKED 0 …」退 0
    bare = run_command_capturing(command_static_check, [])
    selftest.check("static-check 缺 <根>：ERROR 行写原因、下一行给 → 怎么办、退 2",
                   bare[0] == 2 and "ERROR\t缺 <根>" in bare[1] and "→ 怎么办：python3 scripts/mutation/crates-mutation-rows.py static-check \"$PWD\"" in bare[1],
                   bare[1])
    missing_table = run_command_capturing(command_static_check, [static_root, "--table", os.path.join(work, "no-such-table.tsv")])
    selftest.check("static-check --table 指的文件不在：ERROR 退 2，不报 CHECKED", missing_table[0] == 2
                   and "ERROR\t--table 指的变异表不在" in missing_table[1] and "CHECKED" not in missing_table[1], missing_table[1])
    staged = selftest.run(static_root)
    selftest.check("crates-mutation-replay 派活之前判的是同一份：同样几行判红、删行变异不报、假 cargo 一次都没起",
                   staged[0] == 1 and staged[3] == [] and all(f"{TABLE}:{line_number} " in staged[1] for _kind, line_number in expected_problems)
                   and f"{TABLE}:6 " not in staged[1], staged[1])
    # 十六：点名的测试打了 NOT_RUN_NO_GPU_ADAPTER 后返回成功（第 6 行）：记本次未跑、逐条打出来，不算抓到也不算没红，整道不判红、成功句不说全抓到，
    #       往 GATE_MUTATION_NOT_RUN_RECORD 报一行；变异那一趟补了 --show-output（假 cargo 只在带它时交出那一行）。弄坏开关 notrunasmissed 下当没红记，这一格判错
    not_run_root = os.path.join(work, "not-run")
    build_selftest_repository(not_run_root, SELFTEST_TABLE_ROWS + [SELFTEST_NOT_RUN_ROW], SELFTEST_NOT_RUN_TEST)
    not_run_pristine = source_hash(not_run_root)
    not_run_record = os.path.join(work, "not-run-record.txt")
    not_run = selftest.run(not_run_root, GATE_MUTATION_NOT_RUN_RECORD=not_run_record)
    recorded_lines = open(not_run_record, encoding="utf-8").read().splitlines() if os.path.exists(not_run_record) else []
    success_lines = [line for line in not_run[1].splitlines() if line.startswith("  ✓ crates 变异表复跑")]
    selftest.check("点名的测试打了 NOT_RUN：记本次未跑、逐条列出、计数里抓到 4 条没红 0 条、整道不判红、成功句不说全抓到、成功句报本次未跑 1 条、记录里一行",
                   not_run[0] == 0 and "本次未跑：1 条" in not_run[1]
                   and f"{TABLE}:6 card-path-changed：本次未跑——点名的测试 gpu_parity 打了 NOT_RUN_NO_GPU_ADAPTER（样本：本机没有 GPU 适配器）" in not_run[1]
                   and "计数：抓到 4 条、没红 0 条" in not_run[1] and len(success_lines) == 1 and "抓到 4 条、本次未跑 1 条" in success_lines[0]
                   and "本次未跑 1 条（现判的按栏" in success_lines[0]
                   and "各自红在点名的测试上" not in not_run[1] and len(recorded_lines) == 1 and "card-path-changed" in recorded_lines[0],
                   not_run[1] + "\n".join(recorded_lines))
    shown_runs = [line for line in mutation_invocations(not_run[3], not_run_pristine) if "-- --show-output gpu_parity\t" in line]
    selftest.check("变异那一趟的 libtest 参数里补了 --show-output（插在 -- 之后），基线那一批不补",
                   len(shown_runs) == 1 and not any("--show-output" in line for line in not_run[3] if line.endswith("\t" + not_run_pristine)),
                   "\n".join(not_run[3]))
    # 十七：一条抓到的都没有、只有本次未跑的：退 77（不是通过，也不是红）
    only_not_run_root = os.path.join(work, "not-run-only")
    build_selftest_repository(only_not_run_root, [SELFTEST_NOT_RUN_ROW], SELFTEST_NOT_RUN_TEST)
    only_not_run = selftest.run(only_not_run_root)
    selftest.check("只有本次未跑、一条都没抓到：退 77", only_not_run[0] == 77 and "本次未跑：1 条" in only_not_run[1]
                   and "这一道记本次未跑，不是通过" in only_not_run[1], only_not_run[1])


