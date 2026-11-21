"""按尾巴读的登记（TAIL_READ_SCOPES）、不拒算的程序词（UNKEYED_PROGRAM_WORDS）、起仓内程序的登记（LAUNCHED_REPOSITORY_PROGRAMS）、
不走的调用边与子进程边（EXCLUDED_CALL_EDGES、EXCLUDED_PROGRAM_EDGES）、按角色排除的文件（NOT_JUDGING_FILES）、测试服务代码（is_test_service_code）与取这些登记的函数。
（函数单元算键库 scripts/gate/lib_function_units.py 的一部分：用法、判法与弄坏开关表在那一份文件头，不单独起。）"""
import ast
import fnmatch
import os
import lib_lag_split_family as split_family
from lib_function_units_lexer import active_breaks
from lib_function_units_base import FunctionUnitsError, LIBRARY_ROOT


# import 时就执行的模块级语句（文件头「Python 闭包：import 时执行的模块级语句」）：类头只当类头收进来时走的规则名（不算被点名、不连方法）
IMPORT_TIME_RULE = "import-time"
IMPORT_TIME_CLASS_RULE = "import-time-class"
# 判法单元闭包不走的子进程边（文件头「不走的子进程边」）：(起它的文件, 起它的单元的限定名, 被起的程序) → 为什么。按实测排：这一条边只打提示、不改结局，
# 自证 EXCLUDED_EDGE_PYTHON_CELL 与 EXCLUDED_EDGE_SHELL_CELL 在夹具上证这套机制（自证期间临时登记夹具里的边）。与 scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES
# （按角色排、整份文件）是两种排除。登记一条边之前先现查起点单元与被起的程序都在；每条登记的边配三格自证（准入边那三格是 ADMISSION_EDGE_*_CELL）。
# scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 的抄录（文件头「按角色排除的文件」）：admission.py 经这份库取它（admission_unit_key.py 的 not_judging_files_of_entry）
# 给 Options.excluded_files，不载入那份库
NOT_JUDGING_FILES = (
    "scripts/crash/lib_verdict_library.py", "scripts/gate/lib_function_units.py", "scripts/gate/lib_function_units_lexer.py",
    "scripts/gate/lib_function_units_base.py", "scripts/gate/lib_function_units_registry.py", "scripts/gate/lib_function_units_python_facts.py",
    "scripts/gate/lib_function_units_shell_text.py", "scripts/gate/lib_function_units_shell_index.py",
    "scripts/gate/lib_function_units_python_data_reads.py", "scripts/gate/lib_function_units_python_launches.py",
    "scripts/gate/lib_function_units_python_module.py", "scripts/gate/lib_function_units_rust_file.py",
    "scripts/gate/lib_function_units_rust_verus.py", "scripts/gate/lib_function_units_unit_graph.py",
    "scripts/gate/lib_function_units_rust_workspace.py", "scripts/gate/lib_function_units_wgsl.py",
    "scripts/gate/lib_function_units_rust_closure.py", "scripts/gate/lib_function_units_script_closure.py",
    "scripts/gate/lib_function_units_closure_keys.py", "scripts/gate/lib_function_units_selftest_fixture_trees.py",
    "scripts/gate/lib_function_units_selftest_rust_cells.py", "scripts/gate/lib_function_units_selftest_program_registries.py",
    "scripts/gate/lib_function_units_selftest_closure_rules.py", "scripts/gate/lib_function_units_selftest_real_repo.py",
    "scripts/gate/lib_function_units_selftest_run.py",
)
# 算键库运行期从 LIBRARY_ROOT 底下读、算键必须有的数据文件（每项是从仓根起的路径，只写字符串字面量）：给算键库拷出样本仓的那一侧只用 ast 读这个元组、不 import，
# 样本仓里缺了这些文件就算不了键。只给 cross-run-caches 子命令、--judging 命令行与产品行为视图读的（RUNTIME_LIBRARY_ROOT_READS_UNLISTED）不登在这里；
# 自证 RUNTIME_DATA_FILES_CELL 现扫算键库各份里以 LIBRARY_ROOT 拼出的读法，与这个元组逐项对
RUNTIME_DATA_FILES = ("scripts/gate/test-service-code-paths.tsv",)
# 测试服务代码的登记表（文件头「测试服务代码不进键」）：相对这份库所在的仓根；一行「<仓内路径><制表符><理由>」，路径以 / 收尾的是前缀
TEST_SERVICE_CODE_TABLE = RUNTIME_DATA_FILES[0]
# 登的文件连同拆分族一起认的后缀（scripts/gate/lib_lag_split_family.py 认族的那几种入口）；.wgsl 这类别的后缀只认登的那一份
TEST_SERVICE_CODE_FAMILY_SUFFIXES = (".py", ".sh", ".rs")
# 读进来的登记表：路径 → 理由（test_service_code_rows 头一次用到时读）；自证期间临时并进夹具里的行
TEST_SERVICE_CODE_ROWS = {}
# 不拒算的程序词（文件头「不拒算的程序词」）：(文件, 起它的单元的限定名（registered_function_name）, 词) → 为什么；登记之前现查起它的单元与词都在（REGISTERED_WORDS_REAL_CELL）
UNKEYED_PROGRAM_WORDS = {
    ("scripts/crash/gpu-quiesce-run.sh", "gpu_quiesce_card_health", "$GPU_QUIESCE_CARD_HEALTH_PROGRAM"):
        "只给自证换假程序：平常不设，走 else 那一支的 python3 gpu-card-health.py（那一支照连）",
    ("scripts/hosts/peer-host-lib.sh", "peer_host_run", "$2"):
        "在第二台上跑调用方给的一句话：那句话写在调用方的单元里（进调用方的键）；清场、探查这类一句话不判结局，折不出的 $(printf %q …) 不往里认",
    ("scripts/hosts/gpu-card-health.py", "probe_on_this_machine", "probe_command_text(request)"):
        "本文件 collect_card_readings 的源码（inspect.getsource，整份原文进键）经 base64 送进 python3 -c，没有别的代码",
    ("scripts/gate/lib_reuse_checks.py", "run_command", "command"):
        "reuse-selftests 格起登记的自证命令：命令是这一条判定的请求（line: 与 judge: 由那一格写），不在这个判法闭包里",
    ("scripts/mutation/crates_mutation_rows_runs.py", "run_capturing_output", "command"):
        "crates 变异起 cargo test、cargo build 与预编的测试二进制（被测的 crates 由变异行那一侧的键收：被改坏的那一处与点名的测试闭包），"
        "自证里起的是 -c 交的假 producer（正文是自证单元里的字面串）；调用方（crates_mutation_rows_judging.py 的 run_cargo_command 经 WORKER_STATE 的 launch、"
        "crates_mutation_rows_pool.py 的 build_seed_group、自证的子进程正文）交的都不是这一条判定要收的仓里代码",
    ("scripts/hosts/lib_repository_copy.py", "run_probe", "command"):
        "判副本拷全没有的探针（RUNTIME_PROBES）：退出码与键名只给副本闸用，不改判定结局",
    ("scripts/experiment-apparatus/device-code-key.py", "call_unit_graph", "unit_graph"):
        "算键的工具 unit-graph（编出来的二进制）：管怎么算键、不管判对错，与 NOT_JUDGING_FILES 同一个角色",
    # admission_run 写在 UNIT_VERDICT_ROUNDS_LIBRARY 那段正文里（exec 进模块对象，文件头「exec 进模块对象」：正文算在这份文件上）
    (".claude/gate.d/lib/code-tooling-shared-libraries.sh", "admission_run", "command or ['python3', admission, *arguments]"):
        "按条复用的共用段起 admission.py 的判定库子命令（verdict-plan、verdict-record、plan-failure、library-files：算键、查库、记库）。"
        "这一处登记只为命令写成 command or [...] 折不出：起的那份 admission.py 连同 admission 一族已经经别的边在闭包里"
        "（模块级 import 与 shell 格直接起的几条），这一行不把哪份文件、哪个单元排出键；"
        "command 只在样本档造卡住时给（**stall 交的 sleep，系统工具）",
    ("scripts/experiment-apparatus/device-code-key.py", "LibraryRelay.run_device", "command"):
        "起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）",
    ("scripts/experiment-apparatus/device-code-key.py", "run_device", "device_binary_path(root, options)"):
        "起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）",
    ("scripts/experiment-apparatus/e181-landing-check-long-run.sh", "run_segment", "$LAUNCH_BINARY"):
        "编出来的装置的私有副本：Rust 入口由复跑表的行键收（driver_e181 点名 --bin）",
    ("scripts/experiment-apparatus/e181-landing-check-long-run.sh", "run_selftest", "{tree}/scripts/experiment-apparatus/e181-landing-check-long-run.sh"):
        "reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己",
    ("scripts/experiment-apparatus/long_run_item_verdicts.py", "run_in_tree", "argv"):
        "reuse-cells 起调用方交进来的命令模板（-- 后面的词）",
    ("scripts/experiment-apparatus/e184-segment-one-long-run.sh", "run_segment", "$binary"):
        "--build-here 编出来的两份装置（MAIN_BINARY_NAME、DUMP_BINARY_NAME）：Rust 入口由复跑表的行键收（driver_e184_segment_one 正文里字面点名 -p 与 --bin）",
    ("scripts/experiment-apparatus/replay.sh", "replay_one", "${bin#@}"):
        "复跑表第二列的 @<驱动函数>：驱动由表的变量单元按名字连",
    ("scripts/experiment-apparatus/replay.sh", "replay_one", "$REPLAY_RELEASE_DIRECTORY/$bin"):
        "研究工作区编出来的 [[bin]]：Rust 入口由复跑表的行键收",
    ("scripts/hosts/two_logs.py", "run_wrapped", "command"):
        "two-logs 包装起调用方交进来的命令（-- 后面的词）",
    ("scripts/hosts/two_logs.py", "relay_cells.<locals>.wrapped", "shell_text"):
        "自证里起的假命令（echo、exit 这类一句话），不起仓里的代码",
    ("scripts/hosts/two_logs.py", "relay_cells.<locals>.signal_goes_to_command", "f\"echo $$ > '{pid_file}'; exec sleep 60\""):
        "自证里起的假命令，不起仓里的代码",
    ("scripts/hosts/run-build-directory.sh", "run_build_directory_selftest", "$code$([[ $name == normal ]] && echo 'exit 0' || echo 'exit 3')"):
        "自证里起的假编译命令（写 CARGO_TARGET_DIR、照结局退），不起仓里的代码",
    ("scripts/crash/gpu-quiesce-run.sh", "gpu_quiesce_run_configured", "${gpu_quiesce_commands[$1]}"):
        "多机配置 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 的一句话（停私有服务、回读），不起仓里的代码；清场或回读不过这一趟不跑，不改跑出来的判定",
    ("scripts/crash/gpu-quiesce-run.sh", "run_selftest",
     "touch '$work/b-started'; for ((round = 0; round < 200; round++)); do [[ -e '$work/a-done' ]] && break; sleep 0.05; done; echo command-b-end >> '$calls'"):
        "自证格 overlapping-runs 里后进来那一趟的假命令（touch、sleep、echo 写调用记录），串里的 $work、$calls 是自证的临时目录，不起仓里的代码",
    ("scripts/gate/baseline-diff.sh", "run_gates", "$gate"):
        "调用方交进来的门禁脚本（第一个参数，自证里是它写在临时仓里的假门禁）：基准树与主工作区各跑一趟、只比红行，那道门禁由点名它的那一条判定自己的键收",
    ("scripts/crash/crash-amplification-item.sh", "crash_amplification_run_on_this_host", "$CRASH_AMPLIFICATION_ITEM_LOCAL_RUN"):
        "只供测试的变量（文件头「只供测试」）：不是自证设了就退 2，这个函数只在设了时才定义；真的那一趟走 crash-amplification-lib.sh 里同名函数（那一份照连）；"
        "自证指到的假脚本由本文件的自证写出，代码就在本文件里",
    ("scripts/crash/crash-amplification-item.sh", "<script-body>", "$CRASH_AMPLIFICATION_ITEM_LOCAL_RUN"):
        "同上那一行：函数定义写在 if 里，不是顶层函数，脚本体的正文里也有这一句，脚本体那一份单元同样扫到它",
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py", "run_captured", "command"):
        "冻结根（--root，replay.sh 的 replay_frozen_root）副本里的 cargo 与编出来的装置、--measure 的内存包装：冻结根的内容摘要由复跑表的行键收"
        "（admission.py 的 replay_row_request，行键里 line:frozen-root=<名> <摘要>），判这几行的就是 e165-replay-mutation-kill-bounds.sh 与它载入的这份判法模块，已进键；内存包装只设上限",
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py", "state_lines", "['nice', '-n', '19'] + command"):
        "跑冻结根副本里编出来的装置列状态、重放状态：同 run_captured 那一行",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_tree_lines_revision_8.py", "process_arguments", "command"):
        "只给自证换假命令（SELFTEST_VARIABLE 设了才读）：平常走字面的 ps，那一支照连",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py", "load_hosts", "context.hosts_command"):
        "读主机清单：只定分几片、起在哪台，判定不随机器变（所有任务多机可用），不改判定结局",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py", "planned_shards", "hosts_command_of(environment)"):
        "读主机清单定片数：同 load_hosts 那一行，只定分几片，不改判定结局",
    # E171 复跑行（复跑表 driver_e171_r1_*、driver_e171_second_rerun_*）的闭包里，自证与单测替身起的子进程。复跑行只跑单测（mutation_driver.py 起 python3 -m unittest）、
    # 比对器与名单核对，不起 --selftest、不真跑编排器；下面每一处不是只有 --selftest 走到，就是单测把 subprocess.Popen / run 换成录制器、只比命令不起进程。
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py", "world_run", "world.command"):
        "自证端到端格（--selftest）的假世界：命令是 fake_world 拼的、起本编排器自己，带假驱动 fake_row_driver.py 与假任务服务 fake_task_server.py；复跑行与单测都不起它，不改被判那一行的结局",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py", "resume_cell", "world.command"):
        "同 world_run 那一行：自证端到端格的假世界",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py", "launch_local", "command"):
        "编排器真跑时起逐行驱动（context.driver_path：缺省是同目录 row_driver.py，ROWS_ORCHESTRATOR_DRIVER 只给自证换假驱动）；复跑行不真跑编排器，单测也不起它；row_driver.py 由单测 import，已在闭包里",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_8.py", "run_world",
     "one.served_command(world.root, wrapper_command(world, config_path, stop_file))"):
        "自证端到端格（--selftest）：在假世界里起修订 8 这一层的一遍，复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "launch_local_two", "command"):
        "第二段编排器真跑时起逐行驱动（context.driver_path）；单测 test_thread_shares 把 subprocess.Popen 换成录制器、只比命令里的 --lanes 与 --threads，不起进程",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "launch_peer_two", "command"):
        "第二段编排器真跑时经 shard_relay.py 转交第二台的 peer_side.py 起驱动；单测 test_thread_shares 把 subprocess.Popen 换成录制器，不起进程",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "check_rows_two", "context.check"):
        "编排器真跑时核产物的 check_product.py（context.check 取 --check 参数，真跑与自证都指同目录那一份）；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "relay_to_exited_server",
     "one.shard_relay.__file__"):
        "自证端到端格（--selftest）起 shard_relay.py（第一段编排器 import 的那个模块自己的路径）；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "world_run_two", "world.command"):
        "自证端到端格（--selftest）的假世界，同第一段编排器 world_run 那一行",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "resume_cell_two", "world.command"):
        "自证端到端格（--selftest）的假世界，同第一段编排器 world_run 那一行",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py", "regrow_cell",
     "one.served_command(world.root, ['python3', '-B', wrapper] + world.direct[3:])"):
        "自证端到端格（--selftest）：加槽那一格在假世界里起第二段编排器，复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py", "cell_e2e_local_gate_opens",
     "world.base.command"):
        "自证端到端格（--selftest）的假世界；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py", "cell_e2e_yield",
     "world.base.command"):
        "自证端到端格（--selftest）的假世界；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_11.py", "run_pass", "command"):
        "自证端到端格（--selftest）：在假世界里把这一层自己（os.path.abspath(__file__) run）起一遍；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "check_prerequisites", "command"):
        "前提核对起的三条命令是本目录的 revision_8_placeholders.py、frozen_tree_lines_revision_8.py 与 -c 载入 frozen_o3：这几份都由单测 import，已在闭包里",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "launch", "script"):
        "起单元脚本：缺省是同目录 segment_one_unit_revision_8.sh，UNITS_R8_UNIT_SCRIPT 只给自证换假脚本；单测 test_thread_shares 把 subprocess.Popen 换成录制器，"
        "真脚本由 test_revision_8 自己的 bash <脚本> 起法进闭包",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "plan",
     "override('UNITS_R8_UNIT_SCRIPT', UNIT_SCRIPT)"):
        "plan 打各单元的命令（单元脚本 PRINT_ONLY=1）：缺省是同目录 segment_one_unit_revision_8.sh，UNITS_R8_UNIT_SCRIPT 只给自证换假脚本；真脚本由 test_revision_8 自己的 bash <脚本> 起法进闭包",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "plan", "command"):
        "plan 打第 11–13 步逐行编排的 plan（rows_command 拼的缺省是同目录 rows_orchestrator_revision_8.py，UNITS_R8_ROWS_ORCHESTRATOR 只给自证换假的）：那份由单测 import，已在闭包里",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "run",
     "rows_command(settings, done, 'run', budget)"):
        "单元编排真跑时起第 11–13 步的逐行编排（rows_command 拼的缺省是同目录 rows_orchestrator_revision_8.py，UNITS_R8_ROWS_ORCHESTRATOR 只给自证换假的）；复跑行不真跑单元编排，单测不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "world_run", "world.command(mode)"):
        "自证端到端格（--selftest）的假世界；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_units_revision_8.py", "cell_e2e_resume", "world.command()"):
        "自证端到端格（--selftest）的假世界；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/shard_relay.py", "start_request", "request['argv']"):
        "代跑的那一片按请求文件里记的 argv 起（请求是编排器经任务服务写的，命令内容进编排器那一侧的键）；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/peer_side.py", "run_driver", "command + driver_arguments"):
        "第二台上起逐行驱动：驱动路径来自 expected.json（编排器真跑时写的 kit），驱动参数由编排器交；复跑行与单测都不起它",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/peer_side.py", "run_driver", "expected['driver']"):
        "同上一行：驱动路径来自 expected.json",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py", "selfproof_run", "binary"):
        "编出来的装置（stream-constants，Rust 入口由复跑表的行键收）：单测 test_thread_shares 把 subprocess.run 换成录制器、只比命令与环境里的线程数变量，不起进程",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py", "selfproof_run", "command"):
        "同上一行：命令是 nice 起编出来的装置，词在 binary 上",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/revision_8_selftest_harness.py", "run_selftest", "script"):
        "各层自证的共用跑法：script 是调用它的那一层自己的路径（os.path.abspath(__file__)），起的是那一层的 --selftest 子进程；复跑行与单测都不跑 --selftest",
    ("scripts/prune-server/prune_server_client_selftest_service.py", "TestServer.__init__", "command"):
        "起临时 prune-server：它只存判定、管怎么存，不判对错，起它的命令不改被判那一条的结局（同 NOT_JUDGING_FILES 的角色划分）",
    ("scripts/experiment-apparatus/e165-pin-red-examples.py", "run_reporting_progress", "command"):
        "起 cargo 或 --binary 给的程序（编出来的装置），只有 --replay 走到；脚本文件头说明这一下不在判法闭包里，装置的身份由行键里的装置那几样进键；"
        "冻结根驱动起这份脚本只带追加与 --check，不走到它",
    (".claude/gate.d/lib/crash-point-replay-selection-and-plan.sh", "crash_point_replay_pass_output", "$program_file"):
        "弄坏开关 pass-output-program-in-scratch 才走：那份文件由本脚本把 $crash_point_replay_pass_output_program 写进草稿目录，同一段正文经 python3 -B -c 那一支进键",
    (".claude/gate.d/lib/stage-cells.sh", "stage_cells_run", "$function_name"):
        "stage_cell 登记那一行第二列的函数：名字写在阶段脚本 stage_cell 那一行上，函数按词连（ScriptClosure.shell_names）；第二列都是字面名字由 STAGE_CELL_FUNCTIONS_REAL_CELL 判",
    (".claude/hooks/lib_selftest_scratch.py", "child_path_problems", "command"):
        "拉起调用方自证的子进程：调用方给的命令都是 [解释器, 调用方自己, --selftest]（session-start.sh、bash-command-detector.sh、agent-watch.py、quote-kb.py），代码就是调用方，已进键",
    ("scripts/gate/admission_selftest_cells.py", "run_thread_source_cells.<locals>.seen_by_the_case", "launched"):
        "自证：样本用例的命令把 cargo 起往后换成打变量的 sh，不起仓里的代码",
    ("scripts/gate/admission_selftest_cells.py", "stage_variables_seen_by_the_case", "launched"):
        "自证：样本用例的命令把 cargo 起往后换成打变量的 sh，不起仓里的代码",
    # 完工清理闸与交回闸当入口（开关复跑格逐条开关起它们的自证）时闭包里折不出的几处
    (".claude/hooks/completion_cleanup_guard_selftest.py", "Scenario.creator", "arguments"):
        "自证造运行目录的假入口：起的是系统的 sleep，或照 Bash 工具的起法起 bash -c 包住自证用例给的一句内建 read（launcher_command 都是自证里的字面串），不起仓里的代码",
    (".claude/hooks/completion_cleanup_guard_selftest.py", "Scenario.hold", "arguments"):
        "自证留一个占着目录的进程：调用处给的都是系统的 sleep 这类占位命令（自证用例里的字面列表），不起仓里的代码",
    (".claude/singlefs-ai-sop/scripts/claude-hook-lib.sh", "claude_hook_fields", "$CLAUDE_HOOK_FIELDS_PROGRAM"):
        "python3 -c 的串是同一份文件里 here-doc 字面量赋给 CLAUDE_HOOK_FIELDS_PROGRAM 的那一段（读钩子输入打四个字段）：正文在这份文件里、已进键",
    ("prune-server/control", "SelftestWorld.run", "self.command(arguments)"):
        "自证的 World 起的是这一份文件自己（command 交回 [sys.executable, os.path.realpath(__file__), *arguments]）：代码就是 prune-server/control，已进键",
    ("prune-server/control", "cell_watch_forwarded_streams_lines", "world.command(['watch'])"):
        "同 run 那一行：自证起这一份文件自己的 watch 子命令",
    ("prune-server/control", "cell_forwarded_watch_exits_on_hangup", "world.command(['watch'])"):
        "同 run 那一行：自证起这一份文件自己的 watch 子命令",
    ("prune-server/control", "cell_watch_accepts_transitional_states", "world.command(['watch'])"):
        "同 run 那一行：自证起这一份文件自己的 watch 子命令",
    # task-server 自证（task_server_lib.py --selftest）的闭包：下面几处起的是系统工具，或自证里换成假程序的那一处；假程序的正文写在 task_server_selftest.py 的单元里（已进键）
    ("prune-server/control", "run_systemctl", "paths.systemctl"):
        "系统工具 systemctl（prune-server 的用户级单元），不起仓里的代码；task-server 自证里退 75 的那一格把 check 换成假程序（TASK_SERVER_PRUNE_SERVER_CHECK），走不到这里",
    ("prune-server/control", "admin", "paths.binary"):
        "编出来的 prune-server 的 admin 子命令（问健康、管理动作）：它只存判定、管怎么存，不判对错（同 NOT_JUDGING_FILES 的角色划分）；task-server 自证里走不到，同 run_systemctl 那一行",
    ("prune-server/control", "forward", "paths.ssh"):
        "系统工具 ssh：服务在另一台时在同一绝对路径上起同一条 control，两台跑的就是这一份文件（已进键）；task-server 自证里走不到，同 run_systemctl 那一行",
    ("task-server/control", "run_systemctl", "systemctl()"):
        "系统工具 systemctl（服务自己的用户级单元）；TASK_SERVER_CONTROL_SYSTEMCTL 只给自证换假的（fake-systemctl），不起仓里的代码",
    ("task-server/task", "submit", "command"):
        "弄坏开关 fallback-run-when-unreachable 才走：连不上服务时就地起调用方交进来的命令；自证里那条命令是格里写的假命令，正文在 task_server_selftest.py 的单元里",
    ("task-server/attempt_runner.py", "main", "specification['argv']"):
        "起这一次尝试的命令：调用方交给服务的那一条，记在服务写的说明里；自证里都是格里写的假命令，正文在 task_server_selftest.py 的单元里",
    ("task-server/task_server_constants.py", "own_unit_state", "systemctl"):
        "系统工具 systemctl（服务自己的用户级单元），只读 is-active；TASK_SERVER_CONTROL_SYSTEMCTL 只给自证换假的",
    ("task-server/task_server_scheduler_lifecycle.py", "SchedulerLifecycle.check_prune_server.<locals>.run", "command"):
        "check_prune_server 里套的 run：只给自证换假程序（TASK_SERVER_PRUNE_SERVER_CHECK 那一种取法），平常走另一种取法 python3 prune-server/control check（那一支照连）",
    ("task-server/task_server_scheduler_notifying.py", "SchedulerNotifying.read_gpu_cards_failed", "command"):
        "gpu_card_failures 另起的读卡坏线程：只给自证换假程序（TASK_SERVER_SELFTEST_GPU_CARDS_FAILED 那一种取法），平常走另一种取法 python3 gpu-verification-server/control cards-failed（那一支照连）",
    ("task-server/task_server_scheduler_starting.py", "SchedulerStarting.start_unit", "command"):
        "系统工具 systemd-run（TASK_SERVER_SYSTEMD_RUN 只给自证换假的）起尝试的单元；被起的 attempt_runner.py 由同一个函数里 Popen(runner) 那一处连着",
    ("task-server/task_server_lib_configuration.py", "peer_shell_timed", "ssh"):
        "系统工具 ssh（TASK_SERVER_PEER_SSH 只给自证换假的）：在第二台跑探查与清理的一句话，那句话写在调用方的单元里（进调用方的键）",
    ("task-server/task_server_lib_configuration.py", "close_ssh_masters", "ssh"):
        "系统工具 ssh（TASK_SERVER_PEER_SSH 只给自证换假的）：停服务时对复用连接的每个 socket 起一次 -O exit 关主连接，命令的词都写在这个函数里，不起仓里的代码",
    ("task-server/control", "run_gpu_service_control", "fake"):
        "只给自证换的假 gpu-verification-server/control（TASK_SERVER_GPU_SERVICE_CONTROL，假程序的正文在 task_server_selftest.py 的单元里）；"
        "平常走另一种取法 python3 gpu-verification-server/control（那一支照连）",
    ("task-server/task_server_selftest_stalls.py", "cell_attempt_progress_readings", "lib.attempt_progress_probe_text(request)"):
        "自证在本机 bash 里起送到第二台的那一条探查命令：正文由 task_server_lib.py 的 attempt_progress_probe_text 拼出（attempt_progress_readings 的源码 base64 一份），"
        "随那份文件进键",
    # gpu-verification-server/control 经上面 run_gpu_service_control 平常那一支连进 task-server 自证的闭包：下面几处起系统工具或清场脚本，
    # 那份文件自己的自证都换成假的（Paths 照 GPU_VERIFICATION_SERVER_CONTROL_SELFTEST_ROOT 取），task-server 自证起的是假 control、走不到这里
    ("gpu-verification-server/control", "run_systemctl", "paths.systemctl"):
        "系统工具 systemctl（GPU 批处理验证服务的用户级单元），不起仓里的代码",
    ("gpu-verification-server/control", "peer_unit_state", "paths.ssh"):
        "系统工具 ssh：在第二台同一绝对路径的仓里起同一份 control 的 unit-state，两台跑的就是这一份文件（已进键）",
    ("gpu-verification-server/control", "forward", "paths.ssh"):
        "系统工具 ssh：在第二台同一绝对路径的仓里起同一条 control，同 peer_unit_state 那一行",
    ("gpu-verification-server/control", "quiesce_the_cards", "[*paths.quiesce_command, '--', 'true']"):
        "开卡之前的清场 scripts/crash/gpu-quiesce-run.sh -- true：只清场回读、跑 true，不改判定结局；真的那份由它自己的 --selftest 判",
    ("gpu-verification-server/control", "card_failed_lines", "paths.journalctl"):
        "只问环境的系统工具 journalctl：读单元此刻主进程打过的卡坏行（control check 的「卡坏」那一判），不起仓里的代码",
    ("task-server/task_server_machines.py", "PeerMachine.remove_build_directory", "self.scheduler.peer_runner"):
        "multi-host-run.sh --remove-build-directory；自证的世界一律把 TASK_SERVER_PEER_RUNNER 换成假脚本（FAKE_PEER_RUNNER），不起真的；真的那份由它自己的 --selftest 判",
    ("task-server/task_server_scheduler_notifying.py", "SchedulerNotifying.peer_cleanup_confirmed", "self.peer_runner"):
        "multi-host-run.sh --recover-cleanup；同 remove_build_directory 那一行",
    ("task-server/task_server_scheduler_lifecycle.py", "SchedulerLifecycle.workspace_write_judged", "read_back"):
        "起任务工作目录那棵树里的 apply-writer-patch.py --read-outcome，只读结局文件；自证里工作目录是假仓根、起的是格里写的假脚本（FAKE_APPLY_WRITER_PATCH），不起真的",
    (".claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh", "cell_prune_server_selftest", "$ROOT/.prune-server-selftest-program"):
        "只给自证换的假程序：被判的根是样本（judged_root_is_sample）、根里放了这份才起它，正文在样本里（stage-break-switch-replay 的 line:samples 收样本目录的内容）；"
        "真仓里走 else 那一支编出来的 prune-server",
    # 下面四行起的是表行或调用方给的命令：被起的那一份由各条判定自己的请求进键，同 scripts/gate/lib_reuse_checks.py 的 run_command 那一行
    (".claude/gate.d/lib/code-tooling-selftest-cells.sh", "start_selftest", "$inner"):
        "research-script-selftests 起 runner 表的一行（真仓是 SELFTEST_RUNNERS，样本根是 .selftest-runners）：被起的那一份由这一格每条判定的请求进键"
        "（同一份文件里往 $scratch/requests 写请求行的那段 printf：入口是那一行的脚本，带 line:command=<那一行> 与 bash 起的 program-argument:）；"
        "样本根里的表行由 stage-break-switch-replay 请求的 line:samples= 收",
    # TripLauncher.run 写在 SWITCH_REPLAY_PARALLELISM_LIBRARY 那段正文里（exec 进模块对象，文件头「exec 进模块对象」：正文算在这份文件上）
    (".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", "TripLauncher.run", "command"):
        "开关复跑的起法（SWITCH_REPLAY_PARALLELISM_LIBRARY 的 TripLauncher.run）起调用方交的命令，经它起的只有两格："
        "break-switch-replay（code-tooling-break-switch-replay-cells.sh 的 run_selftest_once）起登记表 break-switch-replay-files.tsv 一行的 --selftest，"
        "被起的那份脚本就是那一格每条判定的入口（同一份文件 break-switch-replay 程序的 request_line：带 program-argument:--selftest、line:command=、line:switch=）；"
        "selftest-cells-alone（code-tooling-copy-closure-and-selftest-shape-cells.sh 的 run_trip）带选格变量起 .claude/hooks/selftest-registry.tsv 登记的一份，"
        "同样是那一格每条判定的入口（selftest-cells-alone 程序的 request_line：带 line:cell=）；命令头是包装写法的前缀（SELFTEST_TEMPORARY_SERVICE_PREFIX）时，"
        "包装由同一个 request_line 的 file:scripts/prune-server/prune_server_client_selftest.py 与 line:temporary-service=wrapped 进键；"
        "样本根里的登记表由 stage-break-switch-replay 请求的 line:samples= 收",
    ("scripts/gate/selftest-process-guard.py", "main", "command[0]"):
        "守卫本次不盯时原样 exec 调用方给的那条命令（-- 之后）：被起的那条命令由起守卫的那一侧进键（code-tooling.sh 的 research-script-selftests："
        "入口是 runner 那一行的脚本、line:command=<那一行>）；这一份的 --selftest 起的是自证自己写的样本命令（本文件里的字面量）",
    ("scripts/gate/selftest-process-guard.py", "guard", "argv"):
        "守卫起调用方给的那条命令、盯它的进程树（经 strace 起时前面多 strace 与它的参数）：被起的那条命令同 main 那一行，由起守卫的那一侧进键",
    ("scripts/gate/selftest-process-guard.py", "find_strace", "path"):
        "系统工具 strace（PATH 上的，或自证给的替身路径 override）：只起一次 strace … -- true 问这台机器能不能跟踪 execve，不跑仓里的代码",
    ("scripts/mutation/prove_red_core.py", "run_cargo", "cargo"):
        "证红起 cargo test：平常是 PATH 上的 cargo，编的是 crates，由每条变异行自己的行键管（crates-mutation-rows.py 的 KeyContext：点名测试的闭包、"
        "工具链行、依赖版本行）；自证里是这份脚本自证写出的假 cargo 桩（os.path.join(work, \"bin\", …)），正文在自证的单元里、已进这份脚本的键",
    ('task-server/task_server_scheduler_notifying.py', 'SchedulerNotifying.kill_log_lines', "[self.journalctl, '--no-pager', '-q', '-o', 'cat', f'--since=@{int(since)}', f'--until=@{int(until) + 1}', *selector]"):
        '只问环境的系统工具 journalctl：读本机日志里杀进程的那几行（self.journalctl 只给自证换假的）',
    ('task-server/task_server_selftest_stalls.py', 'cell_runner_threads_file_counts', '[\'bash\', \'-c\', \'"$1" -c "$2" "$3" "$4" "$5" "$6" & wait\', \'runner-threads\', sys.executable, RUNNER_THREADS_PROGRAM, ready, str(RUNNER_THREADS_SPINNING), str(RUNNER_THREADS_SLEEPING), RUNNER_THREADS_NAME]'):
        '自证里起的假程序：bash 只把 sys.executable -c RUNNER_THREADS_PROGRAM 放到后台再 wait；RUNNER_THREADS_PROGRAM 是这份文件里的字面量，随这份文件进键',
    ('scripts/hosts/queue-health.py', 'read_compute_processes', 'command'):
        '只问环境的系统工具 nvidia-smi（QUEUE_HEALTH_NVIDIA_SMI 只给自证换假的）',
    ('scripts/hosts/queue-health.py', 'FakePeer.run', "['bash', '-c', text]"):
        '自证的假第二台（FakePeer）：把这份文件拼给第二台的那段 shell 文本放在这一台的 bash 里跑；文本由这份文件的函数拼出，随这份文件进键',
    ('scripts/agents/agent-watch.py', 'selftest_in', '[\'bash\', \'-c\', f"exec 3>>\'{held_output}\'; exec sleep 60"]'):
        '自证里起的假命令：拿着一个文件句柄睡 60 秒，给看门狗当被盯的进程',
    ('scripts/agents/agent-watch.py', 'selftest_in', '[sys.executable, \'-c\', f"import time\\nhandle = open({writing_file!r}, \'a\')\\ndeadline = time.time() + 60\\nwhile time.time() < deadline:\\n    handle.write(\'一行\\\\n\')\\n    handle.flush()\\n    time.sleep(0.3)\\n"]'):
        '自证里起的假命令：往临时文件里隔一会儿写一行，给看门狗当还在写输出的进程',
    ('scripts/agents/agent-watch.py', 'selftest_in', '[\'bash\', \'-c\', f"exec 3>>\'{silent_file}\'; touch -d \'1 hour ago\' \'{silent_file}\'; exec sleep 60"]'):
        '自证里起的假命令：拿着一个一小时没动的文件句柄睡 60 秒，给看门狗当不出声的进程',
    ('scripts/kb/ledger-append-section.py', 'problems_with_sample_leftover_check', '[sys.executable, \'-c\', "import subprocess; print(subprocess.Popen([\'sleep\', \'%d\'], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).pid, flush=True)" % LEFTOVER_SAMPLE_SECONDS]'):
        '自证里起的假命令：起一个睡几秒的 sleep、打出它的进程号就退出',
    ('scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py', 'subprocess_runner', 'command'):
        '起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）',
    ('scripts/experiment-apparatus/e197_scan_journal_glue_memory_by_device_count/drive.py', 'fake_runner.<locals>.runner', 'command'):
        '自证的假装置（fake_runner）：这份文件自己写出来的假二进制，退出码由 E197_FAKE_BINARY_EXIT 定',
    ('scripts/experiment-apparatus/e209_spike_cause_counters.py', 'mutation_check', "[sys.executable, '-I', str(mutated), '--selftest']"):
        '变异核对：起的是这份文件改坏一处的拷贝的 --selftest；代码就是这份文件，已是这条判定的入口',
    ('scripts/gate/apply-writer-patch.py', 'after_landing', 'gate33'):
        '调用方交进来的 mutation-tables 那一格的命令，只在弄坏开关 mutation-tables-in-process 下在这个进程里起；自证交的是这份文件里的假格',
    ('scripts/hosts/compile-then-swap.py', 'build', 'command'):
        '系统工具 cargo（nice cargo build 与 cargo test --no-run 两条字面命令）：编的是 crates 的装置，Rust 入口由调用方那一侧的请求进键',
    ('scripts/mutation/broad_mutation_library.py', 'raw_mutants_of_package', '[fake_driver, package]'):
        '只给自证换的假程序（BROAD_MUTATION_LIST_DRIVER）；平常走 cargo mutants --list 那一支',
    ('scripts/mutation/broad_mutation_runner.py', 'Runner.run_step', 'argv'):
        '步表里的 cargo 命令（check、build、nextest、test：crates 的测试，由每条变异的单元键进键）；自证的步表是假驱动',
    ('scripts/mutation/broad_mutation_run.py', 'coverage_step.<locals>.listed', "[executable, '--list', '--format', 'terse']"):
        '预编的测试二进制只列测试名（crates 的测试，由每条变异的单元键进键）',
    ('scripts/mutation/broad_mutation_run.py', 'coverage_step.<locals>.cover', 'argv'):
        '带覆盖插桩起预编的测试二进制（crates 的测试，由每条变异的单元键进键）',
    ('scripts/mutation/prove_red_core.py', 'run_cargo_to_log', 'cargo'):
        '同 run_cargo 那一行：系统工具 cargo（自证换假的），crates 的测试由变异行的单元键进键',
    ('.claude/gate.d/lib/crash-point-replay-package-pass-verdicts.sh', 'crash_point_replay_prebuild_case_binaries', '$word'):
        '准入模块交的登记用例命令去掉测试名的那一截（cargo test … --no-run）：只预先编测试二进制，用例由各自的单元键进键',
    ('scripts/mutation/mutate.sh', 'run_test_executable', '$executable'):
        '预编的测试二进制（crates 的测试，由变异条目的单元键进键）',
    ('scripts/mutation/mutation-shard-run.sh', 'mutation_shard_task_server_shard', '$(mutation_shard_llvm_check_command)'):
        '那个函数打出的一段字面 shell：只问这一台的 llvm 工具与 rustc 版本（系统工具），正文随这份文件进键',
    ('scripts/gate/merge-and-verify-steps-lib.sh', 'verification_call', '$VERIFICATION_STAGE'):
        '快照里的 crash-point-replay 门禁（自证换成假的）：它的判定按它自己的键入库，合入后验证只转它的退出码与输出',
    ('scripts/gate/merge-and-verify-steps-lib.sh', 'verification_step', '$VERIFICATION_STAGE'):
        '同 verification_call 那一行',
    ('scripts/gate/merge-and-verify-selftest-lib.sh', 'selftest', '$merge_rows_line'):
        '自证照这份脚本自己打出的那条核变异行的命令（LC_ALL=C grep -xF，系统工具）原样起一遍',
    ('scripts/gate/merge-and-verify-selftest-lib.sh', 'selftest', '$merge_check_line'):
        '自证照这份脚本自己打出的那条核对命令（find 加 sha256sum，系统工具）原样起一遍',
    ('.claude/gate.d/lkmm.sh', 'cell_litmus_verdicts', '.lkmm-key-samples/change.sh'):
        '样本根里的脚本（只在样本档起）：由这一格请求里样本目录的摘要进键',
    ('.claude/gate.d/qemu-device-streams.sh', 'cell_device_streams', '$qemu_path'):
        '外部程序 QEMU（vm-bench.sh 交的路径），不是仓里的代码',
    ('.claude/gate.d/qemu-device-streams.sh', 'run_key_samples', '.qemu-key-samples/change.sh'):
        '样本根里的脚本（只在样本档起）：由这一格请求里样本目录的摘要进键',
    ('.claude/gate.d/qemu-device-streams.sh', 'run_key_samples_three_questions', '.qemu-key-samples/change.sh'):
        '样本根里的脚本（只在样本档起）：由这一格请求里样本目录的摘要进键',
    ('.claude/gate.d/qemu-device-streams.sh', 'run_key_samples_three_questions', '.qemu-key-samples/unrelated.sh'):
        '样本根里的脚本（只在样本档起）：由这一格请求里样本目录的摘要进键',
    ('scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py', 'run_with_time_limit', 'command'):
        '起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）',
    ('scripts/hosts/test-environment-check.py', 'make_sudo_runner.<locals>.run_with_sudo', "['sudo', '-S', '-p', '', *command]"):
        '只问环境的系统工具：sudo 起 smartctl、nvme 读盘的健康信息',
    ('scripts/hosts/test-environment-check.py', 'start_holder_process', 'command'):
        '自证里起的假进程：解释器 -c SLEEPER_CODE（这份文件里的字面量）拿着一个文件睡着',
    ('scripts/crash/crash-amplification-two-hosts.sh', '<script-body>', '$CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN'):
        '只供自证的变量：自证指到这份文件自己写出的假脚本，代码就在这份文件里；真的那一趟不设它',
    ('scripts/crash/crash-amplification-two-hosts.sh', 'crash_amplification_run_on_this_host', '$CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN'):
        '只供自证的变量：自证指到这份文件自己写出的假脚本，代码就在这份文件里；真的那一趟不设它',
    ('scripts/experiment-apparatus/e10-aging-compaction-run.sh', '<script-body>', '$BINARY'):
        '起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）',
    ('scripts/experiment-apparatus/e10-aging-compaction-run.sh', 'judge_shard', '$LAUNCH_BINARY'):
        '起编出来的装置：Rust 入口由复跑表的行键收（admission.py 的 replay_row_request）',
    ('scripts/experiment-apparatus/e10-aging-compaction-run.sh', 'run_selftest', '{tree}/scripts/experiment-apparatus/e10-aging-compaction-run.sh'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_three.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e185-r2-segment-run.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e185-r3-segment-run.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ('scripts/experiment-apparatus/e185-r4-segment-run.sh', 'selftest', '{tree}/$SCRIPT_KEY'):
        'reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己',
    ("scripts/gate/test-targets-parallel.py", "run.<locals>.list_names", "command"):
        "列测试函数名（--list --format terse）：起 cargo test 或编好的 Rust 测试二进制（entry.binary，--prebuild-into 放进清单的），不是仓里要收进闭包的 python 或 shell；"
        "被测的 crates 代码由调用方那一侧的请求进键（门禁 crash-point-replay 按条复用的键、合入后验证补丁改到的测试目标），这一份只定怎么分片，不改每条测试的结局（文件头「结局」）",
    ("scripts/gate/test-targets-parallel.py", "run.<locals>.run_piece", "command"):
        "跑一件测试目标：起 cargo test 或编好的 Rust 测试二进制（同 list_names 那一行）；一件的结局由被测的 crates 与调用方的键管，这一份只定哪几件一起起、各分几条线程，不改每条测试的结局",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/device_card_lease.py", "take_card", "client"):
        "按需领卡的客户端（调用方给的 <树根>/task-server/task，折不出）：领卡没成抛 CardLeaseRefused、装置不起，领到了只定装置用哪张卡，"
        "客户端的标准输出收成卡串；不改读数。CARD_LEASE_CLIENT_WORDS_ANOMALY_CELL、CARD_LEASE_CLIENT_WORDS_STRIP_CELL、CARD_LEASE_CLIENT_WORDS_REACH_CELL 量这两行",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/device_card_lease.py", "return_card", "client"):
        "按需领卡的客户端（同 take_card 那一行）：还卡没成只打一行「!」、不改装置那一步的结局，客户端的标准输出转到标准错误；不改读数。"
        "三格同 take_card 那一行",
}
# 编工具程序的函数（文件头「运行期读的数据文件」编译产物那一条）：(文件, 函数名) → 它编的程序的源码目录（相对根）。函数是 python 函数的限定名
# 或 shell 函数的名字；这个函数进了闭包，那个目录整棵收成数据目录（它读的 target 下编出来的二进制不进键，由产生它的源码代表）
# 登记的被起程序（文件头「登记的被起程序」）：(文件, 起它的单元的限定名（registered_function_name）, 词) → 根里被起的那几份，整份连上
LAUNCHED_REPOSITORY_PROGRAMS = {
    # in_executor <执行器目录> <脚本> <参数…>：脚本体里各步点名的、与这份驱动同目录的那几份 python 脚本（执行器目录就是这份驱动所在的目录）
    ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/run_segment_two.sh', 'in_executor', '${@:2}'): ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/comparator_v1_against_stored.py', 'scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/comparator_two_versions.py', 'scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates/comparator_witness_classes.py'),
    ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_segment_two_q5.sh', 'in_executor', '${@:2}'): ('scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/run_q5_record_path_counts.py', 'scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487/comparator_q5_format_constants.py'),
    # preflight.py 起这两份时把自己的路径放进 PREFLIGHT_PROGRAM：source 的是它旁边那份 lib.sh（规范副本里的）
    ('.claude/scripts/lkmm.sh', '<script-body>', '${PREFLIGHT_PROGRAM%/*}/lib.sh'): ('.claude/singlefs-ai-sop/scripts/lib.sh',),
    ('.claude/scripts/fetch-deps.sh', '<script-body>', '${PREFLIGHT_PROGRAM%/*}/lib.sh'): ('.claude/singlefs-ai-sop/scripts/lib.sh',),
    # bash -c 的字面正文 source "$1" 再调它里面的函数：$1 是 settings.peer_host_lib、$2 是 settings.configuration_check（两处取法都是同仓那两份，自证另交假的）
    ('scripts/hosts/host-dispatch.py', 'Peer.__init__', "['bash', '-c', loader, 'host-dispatch', settings.peer_host_lib, settings.configuration_check, settings.repository_root]"): ('scripts/hosts/peer-host-lib.sh', 'scripts/crash/crash-point-replay-shard-configuration-check.sh'),
    ('scripts/hosts/host-dispatch.py', 'Peer.run', "['bash', '-c', runner, 'host-dispatch', self.settings.peer_host_lib, self.declarations, text]"): ('scripts/hosts/peer-host-lib.sh',),
    # 同一种写法：git-hooks-install.py 的 check_peer 起 bash -c 的字面正文 PEER_RUNNER，source "$1" 再调里面的函数；$1、$2 是 library、configuration_check（取法都是同仓那两份，自证另交假的）
    ('scripts/gate/git-hooks-install.py', 'check_peer', "['bash', '-c', PEER_RUNNER, 'peer', library, configuration_check, top, command]"): ('scripts/hosts/peer-host-lib.sh', 'scripts/crash/crash-point-replay-shard-configuration-check.sh'),
    # 自证的驱动世界起这份文件自己（command 交回 [sys.executable, SELF_PATH, …]），参数折不出：整份连上
    ('scripts/mutation/broad_mutation_selftest_a.py', 'Selftest.run', 'self.command(home, *arguments, source=source)'): ('scripts/mutation/broad-mutation.py',),
    # 同 run 那一行
    ('scripts/mutation/broad_mutation_selftest_a.py', 'selftest_basic_cells', "selftest.command('first', '--list-items')"): ('scripts/mutation/broad-mutation.py',),
    # 同 run 那一行
    ('scripts/mutation/broad_mutation_selftest_a.py', 'selftest_basic_cells', "selftest.command('second')"): ('scripts/mutation/broad-mutation.py',),
    # 同 run 那一行：经一段字面的转起脚本
    ('scripts/mutation/broad_mutation_selftest_a.py', 'selftest_test_variable_cells', "[sys.executable, '-c', relay, *selftest.command('first', '--item', '2')]"): ('scripts/mutation/broad-mutation.py',),
    # 同 run 那一行
    ('scripts/mutation/broad_mutation_selftest_a.py', 'selftest_stop_cells', "selftest.command('stop', '--item', '1')"): ('scripts/mutation/broad-mutation.py',),
    # 同 run 那一行
    ('scripts/mutation/broad_mutation_selftest_a.py', 'selftest_stop_cells', "selftest.command('stop', '--item', '1', '--lock-wait', '0')"): ('scripts/mutation/broad-mutation.py',),
    # 两处调用起的都是 [python3, admission, test-functions-request / test-plan, …]：admission 是这份脚本所在仓里的 scripts/gate/admission.py
    ('scripts/gate/refused-test-registry.py', 'run_command', 'command'): ('scripts/gate/admission.py',),
    # admission 是钩子所在仓里的 scripts/gate/admission.py（RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT 只给自证换假的）
    ('.claude/hooks/runner_dispatch_guard_prompt_checks.py', 'designer_admission_verdict', "[sys.executable, admission, 'experiment', project_root, f'E{int(match.group(1))}']"): ('scripts/gate/admission.py',),
    # 变异锚点静态核：static_check_command 交的是 [python, STATIC_CHECK_SCRIPT, static-check, <树>]，STATIC_CHECK_SCRIPT 是 crates-mutation-rows.py（自证另交假的）
    ('scripts/gate/apply_writer_patch_plan.py', 'run_static_check', 'command'): ('scripts/mutation/crates-mutation-rows.py',),
    # reuse-cells 的命令模板：{tree} 是它拷出来的树，起的就是这份脚本自己的 items 子命令，整份连上
    ('scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py', 'run_reuse_cells', "[sys.executable, '-B', os.path.join(os.path.dirname(HERE), 'long_run_item_verdicts.py'), 'reuse-cells', '--script', script, '--template', world.repo, '--items', ','.join((item_name(unit) for unit in units)), '--work', os.path.join(scratch, 'cells'), '--fake-judge', '--', *command]"): ('scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py',),
}
BUILT_TOOL_SOURCES = {
    ("scripts/gate/lib_function_units_unit_graph.py", "build_unit_graph"): "crates/singlefs-unit-graph/src",
    (".claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh", "function_units_build_unit_graph"): "crates/singlefs-unit-graph/src",
}
# 起编出来的工具的那一处（文件头「运行期读的数据文件」编译产物那一条）：(文件, 起它的单元的限定名（registered_function_name）, 折不出的那个词的 ast.unparse) → 编它的函数
# （BUILT_TOOL_SOURCES 的键）。这一处不拒算，编它的那份源码目录收成数据目录（规则名 tool-source），编它的函数进不进闭包都收
BUILT_TOOL_LAUNCHES = {
    # function-units-two-implementations-agree 的程序里 rust() 起这一趟现编的 unit-graph（binary 取 function_units_build_unit_graph 打的末行）
    (".claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh", "rust", "binary"): (".claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh", "function_units_build_unit_graph"),
}
# 调用方登记（文件头「调用方登记」）：(调用处的文件, 调用处单元的限定名（registered_function_name）, 交命令的那个实参的 ast.unparse) → 为什么。被调函数拿这个实参起子进程、
# 又折不出的，只从这一处起的那一条边不拒算；别的调用处照旧往上认
CALLER_SUPPLIED_COMMANDS = {
    (".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", "replay", "cmd"):
        "stage-break-switch-replay 经 lib_process_group.run_in_own_session 逐条开关起 stage-selftest.sh 喂那一道的样本：被起的阶段由这一格每条判定的请求进键"
        "（同一份文件 stage-break-switch-replay 程序的 request_line：入口 .claude/gate.d/<阶段>、line:samples= 样本目录的摘要、"
        "line:feeder= stage-selftest.sh 与 lib.sh 与包装的摘要）",
    (".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", "probe", "['env', f'{name}={value}', *interpreter, path, *PROBE_ARGUMENTS]"):
        "break-switch-outside-selftest 经 run_in_own_session 设上开关起被判那棵树里读开关的每一份脚本当探针：这一格不复用、每趟现判；"
        "整份 code-tooling.sh 当入口的 stage-break-switch-replay 喂的是样本根里的脚本，由它请求的 line:samples= 收（同一份文件 request_line）",
    ("scripts/gate/gate-structure-check.py", "start_stage_reporting", "['bash', stage_path, root, *arguments]"):
        "经 run_in_own_session 起 <根>/.claude/gate.d/ 下各道阶段的 --list 与 --check：自证起的是自证写进临时根的样本阶段"
        "（正文是本文件 SELFTEST_*_STAGE 那几段字面量，进这一份自证自己的键）；真仓上 code-tooling.sh 的 gate-structure、stage-owners 两格起它，每趟现判、不复用",
}
# 登记表里的命令（文件头「登记表里的命令」）：(文件, 函数的限定名（registered_function_name）) → (登记表, 命令那一列的下标)
REGISTRY_COMMAND_TEXTS = {(".claude/gate.d/lib/doc-registries-reference-and-invariant-cells.sh", "counting_command_result"): (".claude/gate.d/counted-claims.tsv", 2)}
# 载入路径的登记（文件头「载入路径的登记」）：(文件, 路径表达式的 ast.unparse) → 根里实际载入的那一份
SPEC_PATH_ARGUMENTS = {
    # 交回闸：两处路径由 decide 照 cite-check.py 所在目录拼好交进来（自证另交不在的路径量「读不到」那一支）
    (".claude/hooks/handback-guard.sh", "agent_watch_path"): "scripts/agents/agent-watch.py",
    (".claude/hooks/handback-guard.sh", "os.path.join(scripts_directory, 'kb', 'kb-spec-check.py')"): "scripts/kb/kb-spec-check.py",
    ("scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py", "anchors_path"):
        "scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py",
    ("scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/sweep_history_anchor_check.py", "anchors_path"):
        "scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py", "path"): ".claude/gate.d/lib-lag-baseline.py",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py", "path"): ".claude/gate.d/lib-lag-baseline.py",
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/cross_host_o3_check.py", "path"):
        "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py",
    # reuse_library.load_repinned 载入的是冻结层之一（调用方当参数交的）：第一段编排器、第二段编排器、修订 8 的逐行编排、修订 10 的第二段编排
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py", "path"): (
        "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator.py",
        "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two.py",
        "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_revision_8.py",
        "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/rows_orchestrator_segment_two_revision_10.py"),
    # patch-of-the-current-tree.py 的 load_sibling 载入同仓的两份（它 main 里两处调用的字面参数）
    ("scripts/gate/patch-of-the-current-tree.py", "path"): ("scripts/gate/lib_rebuilt_packages.py", "scripts/agents/snapshot-tree.py"),
    # mutation-shard-run.sh 算每片行数的内联 python：sys.argv[1] 是调用处交的 crates-mutation-rows.py
    ("scripts/mutation/mutation-shard-run.sh", "rows_module_path"): "scripts/mutation/crates-mutation-rows.py",
    # merge-and-verify.sh 与它 source 的 merge-and-verify-steps-lib.sh 的几段内联 python（按正文写在哪份文件里登记）：路径是 $REBUILT、$RERUN_FAILED_TESTS
    # 与快照里的 lib_crash_point_replay_items.py 当参数交的
    ("scripts/gate/merge-and-verify-steps-lib.sh", "library"): "scripts/gate/lib_rebuilt_packages.py",
    ("scripts/gate/merge-and-verify.sh", "library_path"): "scripts/crash/lib_crash_point_replay_items.py",
    ("scripts/gate/merge-and-verify.sh", "path"): ("scripts/gate/rerun-failed-tests.py", "scripts/gate/lib_rebuilt_packages.py", "scripts/gate/admission.py"),
    # broad-mutation-samples.py 的 build_units 载入广谱变异驱动（门禁 broad-mutation 造样本时当参数交的那一份）
    (".claude/gate.d/lib/broad-mutation-samples.py", "driver_path"): "scripts/mutation/broad-mutation.py",
    # write-guard.sh 的 load_notations_library 载入共用判据库：平常是 notations_library_path(hook_dir)（钩子所在仓的这一份）；
    # 自证「读不到判据就拒绝」那两格交的是临时目录里不在的一份（什么都没载入）
    (".claude/hooks/write_guard_decisions.py", "library_path"): ".claude/gate.d/lib-forbidden-notations.py",
    # bash-command-detector.sh 的 load_shell_words 载入共用切词模块：平常是 shell_words_library_path(<钩子目录>)（钩子所在仓的 lib_shell_words.py）；
    # 自证拷出来的两个临时目录各载入一份（都是同一份库的拷贝，其中一份的 shell_tokens 只交一个记号）
    (".claude/hooks/bash_command_detector_scope_and_index.py", "library_path"): ".claude/hooks/lib_shell_words.py",
}


# TAIL_READ_SCOPES 前缀一格的「被判那棵树根上的那一份」（文件头「按尾巴读的登记」）
TAIL_READ_ROOT_COPY = "."


def in_tail_read_scope(relative, scope, tails, breaks):
    """相对根的文件 relative 落不落在登记的前缀 scope 里：目录前缀按整段目录比（crates 不认 crates-old/）；TAIL_READ_ROOT_COPY 只认相对根的路径
    就是那一处字面尾巴（tails 里的一截，不带首尾的 /）的那一份。弄坏开关 tail-read-scopes-root-ignored 下 TAIL_READ_ROOT_COPY 也当目录前缀比（一份都不认）。"""
    for prefix in scope:
        if prefix == TAIL_READ_ROOT_COPY and "tail-read-scopes-root-ignored" not in breaks:
            if relative in tails:
                return True
        elif relative.startswith(prefix + "/"):
            return True
    return False


# 按尾巴读的登记（文件头「按尾巴读的登记」）：(文件, 函数的限定名（registered_function_name）, 被读路径表达式的 ast.unparse) → (仓里那一组只留的前缀（相对根）, 为什么)；
# 前缀是空元组的：那一处读的是调用方交进来的输入，一份都不收；前缀写 TAIL_READ_ROOT_COPY（"."）的只留被判那棵树根上的那一份。登记之前现查那一处与前缀都在（REGISTERED_WORDS_REAL_CELL）；
# 登记的是算键时真问到登记的那一处（照尾巴收的，或 tree_copy_file 那一种）：参数照调用处的实参认的，登的是实参那一处的函数与表达式（TAIL_READ_SCOPES_REAL_CELL 逐行核）
TAIL_READ_SCOPES = {
    ("scripts/agents/snapshot-tree.py", "selftest", "os.path.join(head_snapshot, DEFAULT_LIST)"): ((),
        "自证读的是它自己在临时仓里建的那一份快照清单（head_snapshot 在自证的临时目录下），不读真仓的 research/prompts/<轮>-snapshot/；"
        "真仓里每开一轮三方就多一份同名清单，不是这一处的输入"),
    ("task-server/task_server_lib_commands.py", "cargo_manifest_scope", "os.path.join(current, 'Cargo.toml')"): ((),
        "从提交任务的起点目录往上找第一份清单：起点是请求那一侧的 cwd（task_server.py 拒整工作区编译那一处交 run.directory），"
        "读到哪一份由请求定，不是判法自己的数据；自证里起点都在每格自己的临时目录下"),
    ("scripts/gate/apply_writer_patch_plan.py", "plan_with_three_way", "os.path.join(patch_directory, 'result.md')"): ((),
        "patch_directory 是命令行给的补丁目录（apply 到 check_then_apply 一路原样传下来；实现员交回的补丁目录，在工作根下）：读哪一份报告由调用方给的目录定，"
        "不是判法自己的数据；records/、research/ 里同名的 result.md 读不到"),
    ("scripts/kb/replace-batch.py", "selftest", "pathlib.Path(directory, 'first.md')"): ((),
        "自证读的是它自己在 tempfile.TemporaryDirectory() 建的临时目录里写下的 first.md（directory 都是 with 绑的那个临时目录），不读真仓；"
        "research/ 里同名的 first.md 读不到（同一处表达式在 selftest 里出现几次，一行登记管全部）"),
    (".claude/singlefs-ai-sop/scripts/claude-hooks/discarded-output-count-check.sh", "run_selftest", "os.path.join(scratch_directory, 'records.tsv')"): ((),
        "--selftest 那一支 python3 -c \"$JUDGE_PROGRAM\" selftest 起的内联正文（JUDGE_PROGRAM 是这份钩子里 IFS= read -r -d '' 读进的 here-doc）："
        "scratch_directory 是 with tempfile.TemporaryDirectory() 绑的临时目录，records.tsv 是自证经 DISCARDED_OUTPUT_COUNT_CHECK_RECORD 交给它起的钩子、钩子刚写下的检出记录，"
        "不读真仓；records/、research/ 里以 records.tsv 收尾的文件读不到；"
        "读法：with open(record, encoding=\"utf-8\") as record_file:（record = os.path.join(scratch_directory, \"records.tsv\")）"),
    ("scripts/gate/rerun-failed-tests.py", "workspace_packages", "os.path.join(workspace_root, 'Cargo.toml')"): ((),
        "workspace_root 是 --root 给的工作区（不给取这份脚本所在的仓根）：读哪一棵树的根清单由调用方定"
        "（merge-and-verify.sh 给集成副本，自证给临时工作区），不是判法自己的数据"),
    ("scripts/gate/rerun-failed-tests.py", "workspace_packages", "os.path.join(member, 'Cargo.toml')"): ((),
        "member 是 workspace_root 那份根清单 members 展开的目录，都在调用方给的工作区底下：读哪几份由调用方给的根定，不是判法自己的数据"),
    ("scripts/gate/verus_proof_check_base.py", "crate_identifier", "os.path.join(registry.root, registry.crate_directory, 'Cargo.toml')"): (("crates",),
        "registry.root 是被判那棵树，registry.crate_directory 取那棵树 .claude/gate.d/verus-pins.tsv 的 crate 行：判真仓时读的是登记的那几个 crate 的清单，"
        "都在 crates/ 下（往 crates/ 外登 crate 行要照改这一行）；records/、research/ 里同名的读不到"),
    ("scripts/gate/verus_proof_check_base.py", "crate_package_name", "os.path.join(registry.root, registry.crate_directory, 'Cargo.toml')"): (("crates",),
        "同 crate_identifier 那一行：被判那棵树里 verus-pins.tsv 登记的 crate 的清单，都在 crates/ 下（往 crates/ 外登 crate 行要照改这一行）"),
    ("scripts/hosts/rocksdb-prebuilt.py", "build_once", "os.path.join(root, 'Cargo.lock')"): ((TAIL_READ_ROOT_COPY,),
        "build_once 把 root 那棵树根上的 Cargo.lock 拷进试编工程（shutil.copyfile 的源）：root 经 command_build（main 与 command_install 调它）从 main 交下来，main 里只有两处赋值，"
        "缺省的 REPOSITORY_ROOT（这份脚本所在的仓根）与 --root 给的树根（os.path.abspath(argv[index + 1])），读的都是那棵树根上的那一份；"
        "records/、research/ 里同名的 Cargo.lock 读不到；读法：shutil.copyfile(os.path.join(root, \"Cargo.lock\"), os.path.join(project, \"Cargo.lock\"))"),
    ("scripts/hosts/compile-then-swap.py", "locate_bin", "os.path.join(crate_directory, 'Cargo.toml')"): (("crates",),
        "从目标往上找 crate 的清单：目标与往上的每一层都要在 <root>/crates 里（check_crates，出了 crates/ 就拒；只有自证的弄坏开关 outside-crates 不核），"
        "crates/ 外的 Cargo.toml 读不到"),
    ("scripts/hosts/lib_repository_copy.py", "coarse_repository_root_references", "os.path.join(package_root, 'Cargo.toml')"): (("crates",),
        "读清单的那一轮走的是开头那份 list(package_roots) 的拷贝，只有 crates/<名>/Cargo.toml；path 依赖接进来的目录只扫 .rs、不读清单"),
    # 只走固定几个目录的：被判那棵树里登记、成员或包目录都在这几处下（根级的清单登记写不出「只留根上那一份」，那一份不收）
    ("scripts/gate/lib_feature_gated_targets.py", "product_gpu_rows", "package_directory / 'Cargo.toml'"): (("crates",),
        "product-gpu 子命令：package_directory 是 main 里的 pathlib.Path(words[1])（命令行给的包目录）；起它的是门禁 crash-point-replay 的 "
        "crash_point_replay_list_targets（product-gpu-real-card-tests 格交 \"crates/$PRODUCT_GPU_PACKAGE_NAME\"，相对被判那棵树），"
        "读的只有 crates/ 下那个包的清单（起法换到 crates/ 外的包目录要照改这一行）；读法：manifest = package_directory / \"Cargo.toml\""),
    ("scripts/experiment-apparatus/device-code-key.py", "read_manifest", "os.path.join(directory, 'Cargo.toml')"): (("crates", "prune-server", "scripts/experiment-apparatus", TAIL_READ_ROOT_COPY),
        "directory 只有三种来处：package_layout 认的包目录（只认 scripts/experiment-apparatus 与 crates/<名>，别的退 2）"
        "、构建根（装置包目录或仓根）、闭包里 Rust 单元往上最近的包目录（crates/<名>；"
        "包目录给 crates/singlefs-verification 时经 path 依赖到 prune-server/；"
        "装置包下的 e176_crates_oracle 在 scripts/experiment-apparatus 里）；构建根是仓根的那一份是根级清单，留根上那一份（TAIL_READ_ROOT_COPY）：这份脚本自己的 toolchain_lines 只把它的 [profile.release] 摊进工具链行，别的表进不进键没核到底；"
        "读法：with open(os.path.join(directory, \"Cargo.toml\"), \"rb\") as handle:"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py", "path_dependency_entries", "os.path.join(crates_directory, crate, 'Cargo.toml')"): (("crates",),
        "crates_directory = os.path.join(source_root, \"crates\")，crate 取 os.listdir(crates_directory)："
        "只读源树 crates/<名>/Cargo.toml；"
        "读法：manifest = os.path.join(crates_directory, crate, \"Cargo.toml\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py", "main", "os.path.join(repository, COMPATIBILITY_RECORD)"): (("records",),
        "repository 是 --repository 给的仓根，COMPATIBILITY_RECORD 是写死的 records/2026-10-31-删兼容旧版本代码.md："
        "只读 records/ 下这一份（常量改到 records/ 外要照改这一行）；"
        "读法：record_text = read_text(os.path.join(repository, COMPATIBILITY_RECORD))"),
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_driver.py", "source_hashes", "os.path.join(setup.root, path)"): (("crates",),
        "脚本体 exec python3 - 起的内联正文：setup.root 是 --root 给的冻结根（不给取脚本所在的仓根），path 取 SOURCE_FILES 那两份"
        "（crates/singlefs-core/src/recovery.rs、crates/singlefs-replay-prefix/src/lib.rs）：读的只有根里 crates/ 下这两份源码、算 sha256 比登记值，"
        "research/ 里快照下的同名源码读不到；"
        "读法：{path: sha256_of_file(os.path.join(setup.root, path)) for path in SOURCE_FILES}"),
    ("scripts/gate/admission_unit_key.py", "manifest_dependency_names", "os.path.join(root, directory or '.', 'Cargo.toml')"): (("crates", "prune-server", TAIL_READ_ROOT_COPY),
        "唯一的调用处交的 directory 是闭包里 Rust 单元所在 crate 的目录（workspace.crate_of_path：根清单 members 的 crates/<名>，"
        "加经 path 依赖收进来的 prune-server/）；directory 为空时（仓根是包）读的是根级清单，留根上那一份（TAIL_READ_ROOT_COPY）：读到的依赖表只有落到 Cargo.lock 的包经依赖行进键，表里的 features 这类不进；"
        "读法：sections = manifest_sections(os.path.join(root, directory or \".\", \"Cargo.toml\")) or {}"),
    ("scripts/gate/admission_unit_key.py", "workspace_package_directories", "os.path.join(root, directory, 'Cargo.toml')"): (("crates", TAIL_READ_ROOT_COPY),
        "directory 取根清单 [workspace] members 按 glob 展开的目录（仓里的 members 都在 crates/ 下，往 crates/ 外加成员要照改这一行）；"
        "根清单写了 [package] 时另有 \".\"（根级清单，留根上那一份 TAIL_READ_ROOT_COPY：读到的包名到目录的表不由别处写进键；仓根现在不是包）；"
        "读法：name = ((manifest_sections(os.path.join(root, directory, \"Cargo.toml\")) or {}).get(\"package\") or {}).get(\"name\")"),
    ("scripts/gate/admission_unit_key.py", "workspace_binary_entry", "os.path.join(root, directory, 'Cargo.toml')"): (("crates",),
        "directory 取 workspace_package_directories(root) 的值，同那一行：都是 crates/ 下的工作区成员；"
        "读法：sections = manifest_sections(os.path.join(root, directory, \"Cargo.toml\")) or {}"),
    ("scripts/gate/admission_selftest_cells.py", "run_real_crash_case_features_cells", "os.path.join(repository_root, directory, 'Cargo.toml')"): (("crates",),
        "directory 取 workspace_package_directories(repository_root) 的值，同那一行：都是 crates/ 下的工作区成员；"
        "读法：declared = (manifest_sections(os.path.join(repository_root, directory, \"Cargo.toml\")) or {}).get(\"features\") or {}"),
    ("scripts/gate/lib_function_units_rust_workspace.py", "load_manifest", "os.path.join(root, path)"): (("crates", "prune-server", TAIL_READ_ROOT_COPY),
        "调用处只有 discover_rust_crates 两处：path 是 \"Cargo.toml\"（根级清单，留根上那一份 TAIL_READ_ROOT_COPY：members、exclude、[workspace.dependencies] 定工作区的 crate 与依赖，不由别处写进键）"
        "或 posixpath.join(directory, \"Cargo.toml\")，"
        "directory 是根清单 members 展开的 crates/<名>（没有 [workspace] 时取 crates/ 下各目录）"
        "与经 path 依赖收进来的仓内 crate（prune-server/）；"
        "读法：with open(os.path.join(root, path), \"rb\") as handle:"),
    ("scripts/gate/verus_proof_check_base.py", "kernel_files", "os.path.join(crate, 'Cargo.toml')"): (("crates",),
        "crate = os.path.join(root, registry.crate_directory)，"
        "crate_directory 取被判那棵树 verus-pins.tsv 的 crate 行，都在 crates/ 下（同 crate_identifier 那一行）；"
        "读法：manifest = os.path.join(crate, \"Cargo.toml\")"),
    ("scripts/gate/verus_proof_check_base.py", "crate_manifest", "os.path.join(root, registry.crate_directory, 'Cargo.toml')"): (("crates",),
        "同 crate_identifier 那一行：被判那棵树里 verus-pins.tsv 登记的 crate 的清单，都在 crates/ 下；"
        "读法：path = os.path.join(root, registry.crate_directory, \"Cargo.toml\")"),
    ("scripts/gate/verus_proof_check_base.py", "path_dependencies", "os.path.join(root, crate_directory, 'Cargo.toml')"): (("crates",),
        "从 verus-pins.tsv 登记的 crate 起顺 dependencies、build-dependencies 的 path 依赖往下走（出了仓根判红）："
        "登记的两个 crate 往下连到的都在 crates/ 下（往 crates/ 外接 path 依赖要照改这一行）；"
        "读法：manifest_path = os.path.join(root, crate_directory, \"Cargo.toml\")"),
    # 读调用方给的根上的根级文件（Cargo.toml、Cargo.lock）：读到的值没有别处写进键的登 TAIL_READ_ROOT_COPY（只留根上那一份）；
    # 读到的值由调用方解析之后写进键的（依赖行、工具链行、cfg、crate 起法）登空元组，为什么一格写进键的那一处
    ("scripts/crash/lib_verdict_library.py", "locked_dependency_lines", "os.path.join(root, 'Cargo.lock')"): ((),
        "root 是调用方给的根（verus-proof-check.py 交被判那棵树，e165-pin-red-examples.py 交 --root），读那棵树根上的 Cargo.lock，"
        "读到的包由调用方照依赖行写进键（verus-proof-check.py 的 closure_dependency_lines 进「依赖版本」；lib_verdict_library.py 命令行 --dependency-of 进 index.key 的 dependencies；"
        "e165-pin-red-examples.py 进 FunctionUnitIndex.key 的 dependencies），文件不再收；"
        "读法：path = os.path.join(root, \"Cargo.lock\")"),
    ("scripts/gate/crate-cross-references-check.py", "package_key", "os.path.join(root, name)"): ((),
        "root 是 check <根> 给的被判那棵树，name 取 ROOT_INPUTS（Cargo.toml、Cargo.lock、rust-toolchain、rust-toolchain.toml、.cargo/config、.cargo/config.toml）："
        "读的是那棵树根上的这几份，内容 sha256 照 manifest 行写进这一包的单元键（文件头「复用」），文件不再收；"
        "读法：lines += [f\"manifest {name} {sha256_of_file(os.path.join(root, name))}\" for name in ROOT_INPUTS]"),
    ("scripts/gate/crate-cross-references-check.py", "unkeyed_toolchain_and_configuration_lines", "os.path.join(root, name)"): ((),
        "root 是 check <根> 给的被判那棵树，name 取 UNKEYED_ROOT_FILES（Cargo.lock、rust-toolchain、rust-toolchain.toml、.cargo/config、.cargo/config.toml）："
        "只在弄坏开关 toolchain-and-cargo-configuration-keyed 下把那棵树根上这几份的内容 sha256 收回单元键（平常这几样不进键，文件头「复用」），"
        "读的是被判那棵树的输入、不是判法自己的数据，文件不收：收根上的 Cargo.lock，仓里一更新依赖判法键就变、所有包的复用作废，"
        "而文件头「复用」要的是只换了 Cargo.lock 的包照复用；records/、research/ 里同名的读不到；"
        "读法：+ [f\"toolchain-and-configuration {name} {sha256_of_file(os.path.join(root, name))}\" for name in UNKEYED_ROOT_FILES]"),
    ("scripts/gate/admission_unit_key.py", "KeyContext.lock", "os.path.join(closure_root, 'Cargo.lock')"): ((),
        "closure_root 是调用方交的闭包根（被判那棵树），读它根上的 Cargo.lock，读到的包只经 dependency_lines 进调用方的键（scripts/experiment-apparatus/device-code-key.py 的代码键；compute_unit_key 不读它）；"
        "读法：path = os.path.join(closure_root, \"Cargo.lock\")"),
    ("scripts/gate/admission_unit_key.py", "manifest_dependency_names", "os.path.join(root, 'Cargo.toml')"): ((TAIL_READ_ROOT_COPY,),
        "读调用方交的根上那份根清单的 [workspace.dependencies]：只有落到 Cargo.lock 的包名、版本经依赖行进键（dependency_lines 拿它定 allowed），"
        "表里的 features、default-features 不进键，留根上那一份；"
        "读法：shared = ((manifest_sections(os.path.join(root, \"Cargo.toml\")) or {}).get(\"workspace\") or {}).get(\"dependencies\") or {}"),
    ("scripts/gate/admission_unit_key.py", "rust_cfg_for_profile", "os.path.join(closure_root, 'Cargo.toml')"): ((TAIL_READ_ROOT_COPY,),
        "closure_root 是调用方交的闭包根，读它根上的清单定 debug_assertions 留不留（closure_of_entry 交给 Rust 闭包的 cfg，closure_key 收 cfg 行）；"
        "第二处调用 test_functions_of_package（admission.py:3528）拿 cfg 判测试编不编，那份清单按不按含 cfg 的键复用没追到底，宁多收：留根上那一份；"
        "读法：sections = manifest_sections(os.path.join(closure_root, \"Cargo.toml\")) or {}"),
    ("scripts/gate/admission_unit_key.py", "workspace_package_directories", "os.path.join(root, 'Cargo.toml')"): ((TAIL_READ_ROOT_COPY,),
        "root 是调用方交的根，读它根上的根清单取 members：包名到目录的表（workspace_binary_entry、run_real_crash_case_features_cells 等几处用）不由别处写进键，留根上那一份；"
        "读法：root_sections = manifest_sections(os.path.join(root, \"Cargo.toml\")) or {}"),
    ("scripts/gate/lib_function_units_rust_workspace.py", "discover_rust_crates", "os.path.join(root, 'Cargo.toml')"): ((TAIL_READ_ROOT_COPY,),
        "root 是调用方给的根，读它根上的根清单原文，只为查 [workspace] members 里有没有含反斜杠的多行基础串（members_in_multiline_basic_strings_with_a_backslash）；"
        "同一份文件 load_manifest 已按 TAIL_READ_ROOT_COPY 登记收进键，这一处读的是同一份：留根上那一份，records/、research/ 里同名的清单读不到；"
        "读法：with open(os.path.join(root, \"Cargo.toml\"), encoding=\"utf-8\", errors=\"replace\") as handle:"),
    ("scripts/gate/lib_function_units_closure_keys.py", "run_controls", "os.path.join(root, name)"): ((TAIL_READ_ROOT_COPY,),
        "root 是命令行 controls <根> 给的根，name 只取 \"Cargo.toml\"、\"Cargo.lock\"：拷那棵树根上的两份，副本上算的对照键不由别处收进这两份，留根上那一份；"
        "读法：if os.path.isfile(os.path.join(root, name)):（下一行 shutil.copy2 拷它）"),
    ("scripts/gate/verus_proof_check_base.py", "copy_crate", "os.path.join(root, 'Cargo.lock')"): ((TAIL_READ_ROOT_COPY,),
        "root 是被判那棵树（命令行给），把它根上的 Cargo.lock 整份拷进副本、定副本里 cargo 解析出的版本；键里「依赖版本」只收闭包点名的包（closure_dependency_lines），"
        "整份锁对判定的影响没核到底，留根上那一份；"
        "读法：shutil.copy2(os.path.join(root, \"Cargo.lock\"), os.path.join(work, \"Cargo.lock\"))"),
    ("scripts/gate/verus_proof_check_proofs.py", "check_toolchain", "os.path.join(root, 'Cargo.lock')"): ((TAIL_READ_ROOT_COPY,),
        "root 是被判那棵树，读它根上的 Cargo.lock 核登记的版本（toolchain 一项每趟现核、不复用；它所在的判法闭包在别处复不复用没核到底，留根上那一份）；"
        "读法：lock_path = os.path.join(root, \"Cargo.lock\")"),
    ("scripts/gate/verus_proof_check_proofs.py", "compare_mir", "os.path.join(root, 'Cargo.lock')"): ((TAIL_READ_ROOT_COPY,),
        "root 是被判那棵树，把它根上的 Cargo.lock 整份拷进 MIR 比对的副本，同 copy_crate 那一行，留根上那一份；"
        "读法：shutil.copy2(os.path.join(root, \"Cargo.lock\"), os.path.join(copy, \"Cargo.lock\"))"),
    ("scripts/gate/verus_proof_check_listing.py", "crate_build_text", "os.path.join(root, 'Cargo.toml')"): ((),
        "root 是被判那棵树，读它根清单的 [workspace.package] edition，读到的值照「crate 起法」那一样进键（shared_components 把 crate_build_text 的摘要放进复用键）；"
        "读法：with open(os.path.join(root, \"Cargo.toml\"), \"rb\") as handle:"),
    ("scripts/mutation/crates_mutation_rows_pool.py", "read_lock_file_packages", "os.path.join(root, 'Cargo.lock')"): ((TAIL_READ_ROOT_COPY,),
        "root 是调用方给的树根（path_packages_reaching_external 交的根、种子源树 seed_source），读那棵树根上的 Cargo.lock："
        "读到的 path 包与它们够不够得着外部包不由别处写进键，留根上那一份；"
        "读法：with open(os.path.join(root, \"Cargo.lock\"), \"rb\") as handle:"),
    ("scripts/mutation/lib_mutation_units.py", "toolchain_lines", "os.path.join(root, 'Cargo.toml')"): ((),
        "root 是调用方给的工作区根，读它根清单的 lints、workspace.lints（弄坏开关 profile-keyed 下另加 profile）照 manifest 行进工具链行；"
        "工具链行只在弄坏开关下进键（crates-mutation-rows.py 的 build-environment-keyed 一类经 crates_mutation_rows_base.py 的 build_environment_lines_for_break，"
        "mutate.sh 的 toolchainkeyed 经 mutate_plan），平常没人调它，三张变异表的键都不收；"
        "读法：manifest_path = os.path.join(root, \"Cargo.toml\")"),
    ("scripts/mutation/lib_mutation_units.py", "read_lock_packages", "os.path.join(root, 'Cargo.lock')"): ((),
        "root 是 UnitKeys 的 self.root（调用方给的工作区），读它根上的 Cargo.lock；读到的包只经 dependency_lines 进依赖版本行再进键"
        "（UnitKeys.dependencies_of、mutation-prune.py 的 dependency_lines、verus-proof-check.py 的 closure_dependency_lines，三处都是），文件不再收；"
        "读法：path = os.path.join(root, \"Cargo.lock\")"),
    ("scripts/mutation/mutation-prune.py", "build_environment_lines", "os.path.join(root, 'Cargo.toml')"): ((),
        "root 是调用方给的工作区根；只在弄坏开关下读（global_lines 在 BUILD_ENVIRONMENT_BREAKS 里任一个打开时调它：profile-keyed 收解析后的 [profile]、"
        "manifest-text-in-global-key 收原文的 sha256，照 build 行进 closure_key 的工具链行），平常不调、不进键；"
        "读法：manifest = os.path.join(root, \"Cargo.toml\")"),
    # 从调用方给的目录往上找清单：读到哪一份由给的目录定
    ("scripts/gate/lib_rebuilt_packages.py", "package_of", "os.path.join(directory, 'Cargo.toml')"): ((),
        "从调用方给的树里一份文件所在目录往上找第一份带 [package] 的清单（不出树根）：读到哪一份由给的树与文件定；"
        "读法：manifest = os.path.join(directory, \"Cargo.toml\")"),
    ("scripts/gate/lib_rebuilt_packages.py", "workspace_root_of", "os.path.join(directory, 'Cargo.toml')"): ((),
        "同 package_of 那一行：从调用方给的树里一份文件所在目录往上找包与工作区的清单；"
        "读法：manifest = os.path.join(directory, \"Cargo.toml\")"),
    ("scripts/hosts/target-prune.py", "workspace_root_of", "os.path.join(current, 'Cargo.toml')"): ((),
        "从调用方给的目录往上找写了 [workspace] 的清单（不出给的根）：读到哪一份由调用方给的目录定；"
        "读法：manifest = os.path.join(current, \"Cargo.toml\")"),
    ("scripts/gate/crate-cross-references-check.py", "Layout.package_by_manifest", "os.path.join(directory, 'Cargo.toml')"): ((),
        "Layout.classify 调它：从被判那棵树的编译单元 dep-info 里一份不在本地包目录里的文件所在目录往上找第一份带 [package] 的清单（被判那棵树里的找到 <根> 为止）；"
        "读到哪几份由被判那棵树与 cargo check 写的 dep-info 定（此刻仓里那几份是根级、.claude/、litmus/ 下的文件，crates/ 下没有 include 到 records/、research/ 的），"
        "读到的只定这一趟的分类，分类每趟照事实现判、不入库；"
        "读法：manifest = os.path.join(directory, \"Cargo.toml\")"),
    # 调用方给的目录或参数（行目录、集成副本、命令行给的路径）
    ("scripts/agents/verdict-skeleton.py", "write_skeleton", "os.path.join(repository, 'research', 'prompts', f'{round_name}-main-verification.md')"): ((),
        "这一处是写出的目标，不读：open 用 'x'（弄坏开关 overwrite-existing 下 'w'）新建判决骨架，路径由 --repository 与轮名定；"
        "读法：target = out or os.path.join(repository, 'research', 'prompts', f'{round_name}-main-verification.md')"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/cross_host_o3_check.py", "compare", "os.path.join(directory, 'o3.out')"): ((),
        "directory 取命令行给的 --step-12-row-0、--cross-host-row-0 两个行目录；"
        "读法：o3_hashes = [sha256_of_file(os.path.join(directory, \"o3.out\")) if os.path.exists(os.path.join(directory, \"o3.out\")) else \"missing\""),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py", "row_differences", "os.path.join(first, name)"): ((),
        "first、second 是调用方交的两个行目录；"
        "读法：left, right = os.path.join(first, name), os.path.join(second, name)"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py", "row_differences", "os.path.join(second, name)"): ((),
        "同 os.path.join(first, name) 那一行；"
        "读法：left, right = os.path.join(first, name), os.path.join(second, name)"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py", "run_o3_step", "os.path.join(row_directory, 'o3.out')"): ((),
        "row_directory 是调用方交的这一行的输出目录，o3.out 是冻结执行器刚写出的；"
        "读法：output = os.path.join(row_directory, \"o3.out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py", "q1_lines", "os.path.join(row_directory, 'o3.out')"): ((),
        "row_directory = os.path.join(rows_root, f\"row-{number}\")，rows_root 是调用方交的行目录根；"
        "读法：o3 = read_o3(os.path.join(row_directory, \"o3.out\"))"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py", "read_row_o3", "os.path.join(directory, FROZEN_EXECUTOR_OUTPUT)"): ((),
        "directory 是调用方交的行目录；"
        "读法：path = os.path.join(directory, FROZEN_EXECUTOR_OUTPUT)"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py", "run_o3_step", "os.path.join(row_directory, 'o3.out')"): ((),
        "row_directory 是调用方交的这一行的输出目录，o3.out 是冻结执行器刚写出的；"
        "读法：output = os.path.join(row_directory, \"o3.out\")"),
    ("scripts/gate/merge-and-verify-steps-lib.sh", "<module-body>", "os.path.join(tree, 'Cargo.toml')"): ((),
        "verification_member_present 起的内联 python：tree = sys.argv[1]，"
        "是调用处交的集成副本或自证的树（\"$tree_root\"、\"$tree\"）；"
        "读法：with open(os.path.join(tree, \"Cargo.toml\"), \"rb\") as handle:"),
    ("scripts/gate/merge-and-verify-steps-lib.sh", "<module-body>", "os.path.join(tree, member, 'Cargo.toml')"): ((),
        "同 os.path.join(tree, 'Cargo.toml') 那一行：member 是那棵树根清单的 members；"
        "读法：with open(os.path.join(tree, member, \"Cargo.toml\"), \"rb\") as handle:"),
    ("scripts/gate/merge-and-verify.sh", "manifest", "os.path.join(directory, 'Cargo.toml')"): ((),
        "开跑那一刻的指纹用的内联 python（MAIN_TREE_FINGERPRINT_PROGRAM）：directory 取 tree（命令行给的要验的树）、"
        "tree 根清单 members 展开的目录、依赖往下走到的包目录、补丁改到的路径往上的各层目录（都不出 tree）：读哪几份由调用方给的树与补丁定，"
        "不是判法自己的数据；records/、research/ 里同名的清单读不到；"
        "读法：path = os.path.join(directory, \"Cargo.toml\")"),
    ("scripts/mutation/mutate.sh", "manifest_of", "os.path.join(directory, 'Cargo.toml')"): ((),
        "list_test_targets 起的内联 python：root = sys.argv[1]（--test-targets 给的工作区目录，不给取当前目录），directory 取 root 与 root 根清单 members 展开的目录："
        "读哪一个工作区由调用方给的目录定，不是判法自己的数据；records/、research/ 里同名的清单读不到；"
        "读法：with open(os.path.join(directory, \"Cargo.toml\"), \"rb\") as handle:"),
    ("scripts/gate/merge-and-verify.sh", "mutation_rows", "os.path.join(directory, file_name)"): ((),
        "脚本里 select_targets 所在的内联 python：directory 取 patch_directories（命令行给的补丁目录，实现员交回的补丁目录在工作根下），file_name 只取 mutations-append.tsv、mutations-replacements.tsv：读哪几份由调用方给的目录定，"
        "不是判法自己的数据；research/ 里快照目录下的同名文件读不到；"
        "读法：path = os.path.join(directory, file_name)"),
    ("scripts/gate/apply_writer_patch_plan.py", "snapshot_touched_paths", "os.path.join(patch_directory, name)"): ((),
        "patch_directory 同 plan_with_three_way 那几行，是命令行给的补丁目录：name 取 MUTATION_FILES 的三份变异表补丁文件，读哪几份由调用方给的目录定，"
        "不是判法自己的数据；research/ 里快照目录下的同名文件读不到；"
        "读法：path = os.path.join(patch_directory, name)"),
    ("scripts/gate/apply_writer_patch_plan.py", "plan_with_three_way", "os.path.join(patch_directory, 'mutations-append.tsv')"): ((),
        "patch_directory 同 os.path.join(patch_directory, 'result.md') 那一行，是命令行给的补丁目录：读它里面的追加行，由调用方给的目录定，"
        "不是判法自己的数据；research/ 里快照目录下的同名文件读不到；"
        "读法：appended = table_rows(os.path.join(patch_directory, \"mutations-append.tsv\"))"),
    ("scripts/gate/apply_writer_patch_plan.py", "plan_with_three_way", "os.path.join(patch_directory, 'mutations-replacements.tsv')"): ((),
        "同 os.path.join(patch_directory, 'mutations-append.tsv') 那一行：读补丁目录里的替换行；"
        "读法：replaced = table_rows(os.path.join(patch_directory, \"mutations-replacements.tsv\"))"),
    ("scripts/gate/apply_writer_patch_plan.py", "plan_with_three_way", "os.path.join(patch_directory, 'mutations-delete.txt')"): ((),
        "同 os.path.join(patch_directory, 'mutations-append.tsv') 那一行：读补丁目录里的删除行；"
        "读法：deletion_path = os.path.join(patch_directory, \"mutations-delete.txt\")"),
    ("scripts/gate/crate-roles-check.py", "cargo_projects", "os.path.join(root, 'Cargo.toml')"): ((TAIL_READ_ROOT_COPY,),
        "入口是 .claude/hooks/selftest-registry.tsv 登记的 --selftest：root 是 judge 的参数，自证里是 tempfile.TemporaryDirectory 下每格自己写的样本树，"
        "平常跑（门禁 code-tooling 的 crate-roles 格）是 $ROOT 给的被判那棵树；读调用方给的根上的根清单（[workspace] members 与 exclude），"
        "读到的值不由别处写进键，留根上那一份；"
        "读法：root_manifest, _ = read_manifest(os.path.join(root, \"Cargo.toml\"))"),
    ("scripts/gate/crate-roles-check.py", "judge", "os.path.join(root, directory, 'Cargo.toml')"): (("gpu-verification-server", "prune-server", "scripts/experiment-apparatus"),
        "同 cargo_projects 那一行：root 由调用方给，directory 取那棵树根清单 [workspace] members 与 exclude 点名的 crates/ 之外的目录，"
        "仓里现在是 gpu-verification-server、prune-server、scripts/experiment-apparatus（往 crates/ 外加成员或排除项要照改这一行）；"
        "读法：manifest, _ = read_manifest(os.path.join(root, directory, \"Cargo.toml\"))"),
    # 自证与测试自己建的临时目录
    ("scripts/agents/checklist-specs.py", "selftest", "os.path.join(directory, 'checklist.md')"): ((),
        "directory 是 tempfile.TemporaryDirectory()；"
        "读法：checklist = os.path.join(directory, 'checklist.md')"),
    ("scripts/agents/dispatch-prompt.py", "selftest", "os.path.join(work, 'attack.md')"): ((),
        "work = tempfile.mkdtemp(prefix=\"dispatch-prompt-selftest-\")；"
        "读法：attack = os.path.join(work, \"attack.md\")"),
    ("scripts/agents/snapshot-tree.py", "selftest", "os.path.join(head_snapshot, HEAD_DIFF)"): ((),
        "head_snapshot 是自证在 work = tempfile.mkdtemp(…) 下的样本仓里建的快照目录；"
        "读法：shutil.copy(os.path.join(head_snapshot, HEAD_DIFF), os.path.join(repository, \"research\", \"prompts\", \"r4-snapshot\", HEAD_DIFF))"),
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_selftest.py", "selftest", "os.path.join(temporary, 'r2.out')"): ((),
        "脚本体 exec python3 - 起的内联正文里的 selftest："
        "temporary 是 tempfile.TemporaryDirectory(prefix=\"e165-kill-bounds-selftest-\")；"
        "读法：output = read(os.path.join(temporary, \"r2.out\"))"),
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_selftest.py", "selftest", "os.path.join(temporary, 'r3.out')"): ((),
        "同 r2.out 那一行；"
        "读法：output = read(os.path.join(temporary, \"r3.out\"))"),
    ("scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_selftest.py", "selftest", "os.path.join(temporary, 'r4.out')"): ((),
        "同 r2.out 那一行；"
        "读法：output = read(os.path.join(temporary, \"r4.out\"))"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_tree_lines_revision_8.py", "cell_before_snapshot_end_to_end", "os.path.join(repository, path)"): ((),
        "repository = fake_repository(os.path.join(scratch, \"repo\"))，scratch 是自证格的临时目录；"
        "读法：\"\".join(f\"{sha256_of_file(os.path.join(repository, path))}  {path}\\n\" for path in (\"Cargo.toml\", \"crates/fake/src/row_15.rs\")))"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_list_check.py", "AfterLists.run_after_check", "os.path.join(root, 'out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：output = os.path.join(root, \"out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_segment_one.py", "CounterBranchStateList.test_only_counter_branch_states_are_listed_with_their_fields", "os.path.join(scratch, 'out')"): ((),
        "scratch 是 tempfile.TemporaryDirectory()；"
        "读法：output = os.path.join(scratch, \"out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_segment_one.py", "WitnessTrajectories.test_the_trajectory_lines_by_segment_and_by_inode_creation", "os.path.join(scratch, 'out')"): ((),
        "scratch 是 tempfile.TemporaryDirectory()；"
        "读法：output = os.path.join(scratch, \"out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/test_segment_one.py", "ConfirmedLossTrajectory.lines_of", "os.path.join(scratch, 'out')"): ((),
        "scratch 是 tempfile.TemporaryDirectory()；"
        "读法：output = os.path.join(scratch, \"out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_collusion_pipeline.py", "RunDirectoryTests.test_result_lines_count_the_done_line_and_refuse_an_existing_file", "os.path.join(root, 'out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：path = os.path.join(root, \"out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_collusion_pipeline.py", "ComparatorSectionTests.test_a_positive_control_counts_red_only_when_the_matrix_carries_the_red", "os.path.join(output, 'out')"): ((),
        "output 是 tempfile.TemporaryDirectory()；"
        "读法：with open(os.path.join(output, \"out\"), encoding=\"utf-8\") as handle:"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_columns.py", "ImageJudgeTests.test_a_patch_that_runs_past_the_end_of_the_blob_is_refused", "os.path.join(root, 's.tsv')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：write(os.path.join(root, \"s.tsv\"), …)"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_columns.py", "ExecutorOutputTests.test_states_unrecoverable_streams_errors_digest_mismatches_and_the_freeze_lines_are_read", "os.path.join(root, 'o3.out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：path = os.path.join(root, \"o3.out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_columns.py", "ExecutorOutputTests.test_an_output_without_the_complete_marker_is_not_complete", "os.path.join(root, 'o3.out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：path = os.path.join(root, \"o3.out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_comparison.py", "ComparatorO3Tests.test_a_control_for_o3_is_counted_and_the_unknown_oracle_name_is_refused", "os.path.join(root, 'out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：with open(os.path.join(root, \"out\"), encoding=\"utf-8\") as handle:"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_comparison.py", "ComparatorO3Tests.test_the_executor_state_lines_are_sorted_and_lose_the_time_and_the_process_number", "os.path.join(root, 'o3.out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：path = os.path.join(root, \"o3.out\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_comparison.py", "render", "os.path.join(root, 'out')"): ((),
        "root 是 tempfile.TemporaryDirectory()；"
        "读法：with open(os.path.join(root, \"out\"), encoding=\"utf-8\") as handle:"),
    ("scripts/gate/admission_selftest_support.py", "run_unit_key_cells", "os.path.join(work, 'Cargo.lock')"): ((),
        "work = tempfile.mkdtemp(prefix=\"admission-selftest-units-\")，样本仓建在里面；"
        "读法：lock = os.path.join(work, \"Cargo.lock\")"),
    ("scripts/gate/admission_selftest_support.py", "run_unit_key_cells", "os.path.join(work, 'Cargo.toml')"): ((),
        "work 同 Cargo.lock 那一行，两处都是交给 cell(…) 改的样本清单；"
        "读法：cell(\"仓根 Cargo.toml 加一行注释（N07）\", os.path.join(work, \"Cargo.toml\"), …)"),
    ("scripts/gate/stage-mine.py", "selftest_keep_foreign", "os.path.join(repo, 'record.md')"): ((),
        "repo 是 tempfile.TemporaryDirectory()；"
        "读法：with open(os.path.join(repo, 'record.md'), encoding='utf-8') as handle:"),
    ("scripts/hosts/test-environment-check.py", "check_host_disk", "os.path.join(case_directory, 'report.md')"): ((),
        "case_directory 是 host_environment 在自证的 workspace（tempfile.mkdtemp）下 mkdtemp 的；"
        "读法：report_path = os.path.join(case_directory, 'report.md')"),
    ("scripts/kb/quote-kb.py", "selftest_in", "os.path.join(os.path.dirname(src), 'checklist.md')"): ((),
        "src 在 work 下，work 是共用库建的自证临时目录（selftest 交进来）；"
        "读法：ck = os.path.join(os.path.dirname(src), 'checklist.md')"),
    ("scripts/kb/stale-candidates.py", "experiment_page_selftest_failures", "os.path.join(directory, 'facts.tsv')"): ((),
        "directory 是 tempfile.TemporaryDirectory() 里建的样本仓；"
        "读法：locations = {row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts.tsv')))}"),
    ("scripts/kb/stale-candidates.py", "carrier_lines_selftest_failures", "os.path.join(directory, 'facts.tsv')"): ((),
        "directory 是 tempfile.TemporaryDirectory() 里建的样本仓；"
        "读法：locations = {row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts.tsv')))}"),
    ("scripts/kb/stale-candidates.py", "selftest", "os.path.join(directory, 'candidates.tsv')"): ((),
        "directory 是 tempfile.TemporaryDirectory() 里建的样本仓；"
        "读法：table_path = os.path.join(directory, 'candidates.tsv')"),
    (".claude/singlefs-ai-sop/scripts/session-checks.py", "run_selftest_cases", "os.path.join(scratch, 'records.tsv')"): ((),
        "scratch 是 tempfile.TemporaryDirectory()；"
        "读法：record_file = os.path.join(scratch, \"records.tsv\")"),
    (".claude/singlefs-ai-sop/scripts/session-checks.py", "run_hook_cases", "os.path.join(scratch, 'records.tsv')"): ((),
        "scratch 是 tempfile.TemporaryDirectory()；"
        "读法：record_file = os.path.join(scratch, \"records.tsv\")"),
    ("scripts/experiment-apparatus/e171_three_oracle_collusion/expansion_table.py", "expansion_rows", "os.path.join(scratch, 'mutations.tsv')"): ((),
        "scratch 是 tempfile.TemporaryDirectory(prefix=\"e171-expansion-\")，里面写的是这一次调用交进来的表文本；"
        "读法：table_path = os.path.join(scratch, \"mutations.tsv\")"),
    ("scripts/mutation/mutation-shard-run.sh", "<module-body>", "f'{directory}/mutations.tsv'"): ((),
        "mutation_shard_selftest 里分片那一格的判定用的内联 python（heredoc PY_ASSIGN_JUDGE）：directory = sys.argv[1]，"
        "是那一格在 $work 下自己建的 assign-cell 目录，里面的 mutations.tsv 是同一格前一段 python（PY_ASSIGN_DATA）刚写出的；"
        "读法：with open(f\"{directory}/mutations.tsv\", encoding=\"utf-8\") as handle:"),
}


# 按尾巴读而没登记 TAIL_READ_SCOPES 的那几处收进来的、落在 records/、research/ 下的文件（文件头「按尾巴读的登记」末段，TAIL_READS_RECORDS_REAL_CELL 读）：
# {(根的绝对路径, 文件, 行, 函数的限定名（registered_function_name）, 表达式, 是不是内联脚本): {相对根的文件…}}；只记不改判定，键与闭包照旧
UNSCOPED_TAIL_READS = {}
TAIL_READ_RECORD_PREFIXES = ("records/", "research/")


def record_unscoped_tail_read(module, node, unit, origins):
    """按尾巴读、没登记 TAIL_READ_SCOPES 的这一处收进来的文件里，落在 records/、research/ 下的记进 UNSCOPED_TAIL_READS。
    弄坏开关 tail-reads-under-records-unrecorded 下不记。"""
    if "tail-reads-under-records-unrecorded" in module.index.breaks:
        return
    under = {origin[1] for origin in origins if origin[0] == "file" and origin[1].startswith(TAIL_READ_RECORD_PREFIXES)}
    if under:
        key = (module.index.root, module.source_file(), getattr(node, "lineno", 0), registered_function_name(unit), ast.unparse(node), module.inline is not None)
        UNSCOPED_TAIL_READS.setdefault(key, set()).update(under)


def shell_unquoted_word(word):
    """登记表里比 shell 的词用的写法：去掉包着它的那一对双引号（里面的引号照留，同原文）。"""
    return word[1:-1] if len(word) >= 2 and word[0] == word[-1] == '"' else word


def registered_function_name(unit):
    """按（文件, 函数）登记的几张表里函数一格的写法（文件头「不拒算的程序词」）：python 单元取限定名 unit.name（<类>.<方法>、<外层函数>.<locals>.<内层函数>，
    模块级的就是名字，<module-body>、<main-guard> 照写），shell 单元取 unit.short（函数名，套着的函数是内层的名字；脚本体 <script-body>）；没有单元交空串。
    同一份文件里别的类的同名方法、别的函数里套着的同名函数各是一行登记，不共用。弄坏开关 registered-functions-short-names 下 python 也只取最后一段。"""
    if unit is None:
        return ""
    if getattr(unit, "language", None) != "python":
        return getattr(unit, "short", None) or unit.name
    if "registered-functions-short-names" in active_breaks():
        return (getattr(unit, "short", None) or unit.name).rsplit(".", 1)[-1]
    return unit.name


def program_word_registered(file, function, words):
    """words 里有一个登在 UNKEYED_PROGRAM_WORDS 的 (file, function, 词) 上（文件头「不拒算的程序词」）。弄坏开关 program-words-refused 下一律不算。"""
    return "program-words-refused" not in active_breaks() and any((file, function, word) in UNKEYED_PROGRAM_WORDS for word in words)


def launched_programs_registered(file, function, words, root):
    """words 里有一个登在 LAUNCHED_REPOSITORY_PROGRAMS 的 (file, function, 词) 上（文件头「登记的被起程序」）→ 被起的那几份（相对根）；
    没登记、登记的有一份不在根里 → None。弄坏开关 launched-programs-refused 下一律 None。"""
    if "launched-programs-refused" in active_breaks():
        return None
    for word in words:
        targets = LAUNCHED_REPOSITORY_PROGRAMS.get((file, function, word))
        if targets is not None:
            return list(targets) if all(os.path.isfile(os.path.join(root, target)) for target in targets) else None
    return None


def built_tool_launch_source(file, function, words):
    """words 里有一个登在 BUILT_TOOL_LAUNCHES 的 (file, function, 词) 上、编它的函数登在 BUILT_TOOL_SOURCES 里（文件头「运行期读的数据文件」编译产物那一条）
    → 那份源码目录（相对根）；没登记 → None。弄坏开关 built-tool-launches-refused 下一律 None。"""
    if "built-tool-launches-refused" in active_breaks():
        return None
    for word in words:
        builder = BUILT_TOOL_LAUNCHES.get((file, function, word))
        if builder is not None and builder in BUILT_TOOL_SOURCES:
            return BUILT_TOOL_SOURCES[builder]
    return None


def test_service_code_rows():
    """测试服务代码登记表（TEST_SERVICE_CODE_TABLE）读成 {路径: 理由}，一个进程读一次（记进 TEST_SERVICE_CODE_ROWS）；# 起头的行与空行不算。
    读不了、一行不是两列、路径或理由空着、路径重复的抛 FunctionUnitsError：表写错了，键算不下去（调用方按「算不出」处置）。
    弄坏开关 test-service-code-table-row-dropped 下少表里头一行。"""
    if TEST_SERVICE_CODE_ROWS:
        return TEST_SERVICE_CODE_ROWS
    rows = {}
    try:
        with open(os.path.join(LIBRARY_ROOT, TEST_SERVICE_CODE_TABLE), encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise FunctionUnitsError(f"读不了测试服务代码登记表 {TEST_SERVICE_CODE_TABLE}（{error}）：这份库算键要它") from error
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 2 or not fields[0].strip() or not fields[1].strip() or fields[0] in rows:
            raise FunctionUnitsError(f"{TEST_SERVICE_CODE_TABLE}:{number} 写错了（一行「<仓内路径><制表符><理由>」，路径不重复）：{line}")
        rows[fields[0]] = fields[1]
    if "test-service-code-table-row-dropped" in active_breaks() and rows:
        rows.pop(next(iter(rows)))
    TEST_SERVICE_CODE_ROWS.update(rows)
    return TEST_SERVICE_CODE_ROWS


def is_test_service_code(path):
    """path（仓内相对路径）是不是测试服务代码（文件头「测试服务代码不进键」）：落在登记表里以 / 收尾的一行底下，就是登的那份文件，
    或是登的 .py、.sh、.rs 那份文件的拆分族里的部分文件（lib_lag_split_family.in_family）。弄坏开关 test-service-code-split-families-unrecognized 下不认拆分族。"""
    families = "test-service-code-split-families-unrecognized" not in active_breaks()
    for row in test_service_code_rows():
        if row.endswith("/"):
            if path.startswith(row):
                return True
        elif path == row or (families and row.endswith(TEST_SERVICE_CODE_FAMILY_SUFFIXES) and split_family.in_family(row, path)):
            return True
    return False


def root_gitignore_rules(root):
    """仓根那一份 .gitignore 里落得到仓根这一层文件的那几条（文件头「仓根被忽略的私有文件」）→ [(是不是 ! 取反, 模式)]，照原次序；文件不在或读不出交 []。
    空行与 # 起头的跳过；\\# 与 \\! 起头的去掉反斜杠照字面认；以 / 收尾的只挡目录，跳过；开头的 / 与 **/ 去掉；去掉之后还带 / 的点的是更深一层的路径，跳过。"""
    try:
        with open(os.path.join(root, ".gitignore"), encoding="utf-8", errors="replace") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return []
    rules = []
    for line in lines:
        pattern = line.rstrip()
        if not pattern or pattern.startswith("#"):
            continue
        negated = pattern.startswith("!")
        if negated:
            pattern = pattern[1:]
        elif pattern.startswith(("\\#", "\\!")):
            pattern = pattern[1:]
        if pattern.endswith("/"):
            continue
        while pattern.startswith("**/"):
            pattern = pattern[3:]
        if pattern.startswith("/"):
            pattern = pattern[1:]
        if pattern and "/" not in pattern:
            rules.append((negated, pattern))
    return rules


def root_file_ignored(rules, name):
    """仓根这一层的文件名 name 照 root_gitignore_rules 交的几条判挡没挡着：最后一条对得上的定，那一条是 ! 取反的就不算挡着。"""
    ignored = False
    for negated, pattern in rules:
        if fnmatch.fnmatchcase(name, pattern):
            ignored = not negated
    return ignored


ADMISSION_EDGE = ("scripts/experiment-apparatus/e183-capacity-cells.sh", "plan_step", "scripts/gate/admission.py")
EXCLUDED_PROGRAM_EDGES = {
    ADMISSION_EDGE: "准入是闸：退出码只定跑不跑（77 不跑、3 退 78、别的退 2，都在编译之前），标准输出只有 E7INPUT 行，"
                    "读产物的判法先 sed '/^E7INPUT /d'；不改读数",
}
# 不走的调用边（文件头「不走的调用边」）：(调用处的文件, 调用处单元的限定名, 被调的名字) → 为什么。登记了的那一条按名字连的边不连，被调的那个单元
# 只经这条边才到得了的，连同它读的数据、起的程序都不进键；别的单元调它照连
EXCLUDED_CALL_EDGES = {
    ("scripts/gate/lib_function_units_unit_graph.py", "unit_graph_directory", "built_unit_graph"):
        "找或编产品行为视图用的 unit-graph 程序（编它读的 Cargo.lock、.claude/heavy-thread-default、crates/singlefs-unit-graph/src 只定缓存目录名与编法）："
        "程序交出的单元行逐个进各条判定的键（文件头「判定种类」），程序本身管怎么算键、不管判对错，同 NOT_JUDGING_FILES 的角色",
}
