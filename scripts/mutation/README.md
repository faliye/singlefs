# scripts/mutation

变异：实验装置的变异驱动、crates 变异表的判法、实现员证红、广谱变异。变异表在 `tables/`（一个实验一张，`e<编号>_<简称>.tsv`）；crates 的变异表是 `crates/mutations.tsv`。

| 文件 | 做什么 |
|---|---|
| `mutate.sh` | 在 `scripts/experiment-apparatus/` 下逐条注入变异、跑测试、记红了哪几个；变异只改副本，副本根下照层级链回仓里的是 `scripts/hosts/lib_repository_copy.py` 的闭包里副本还没有的那几样（装置编译期 `include_str!` 读的 `.claude/heavy-thread-default` 这类、运行期读的与它们 source、起的；`research/results/` 另链）；每条判定按函数单元键写进 prune-server（mutation-units 族；库名 `<库根的 git common-dir>/singlefs-mutate-library` 只当名字与命名空间，本地不建、不读、不写；取不到 git common-dir 就退 2、不退到别的键前缀），键对得上的复用，`--full` 只写不读 |
| `prove-red.sh` | 实现员在仓副本里逐条证 `crates/mutations.tsv` 的行会红；命中重测试目标的行照 `lib_release_only_tests.py` 补 `--release` 起 |
| `prove_red_core.py` | `prove-red.sh` 内联 python 的本体：读变异表与成形、重测试目标补 `--release`、起 cargo 与编译目录、还原日志、判定库的行键与复用、并行的路、`prove`；入口的内联 python 同目录 import、名字再导出（`prove-red.sh` 文件头写着开关的代码在哪一份） |
| `prove_red_selftest.py` | `prove-red.sh` 的自证：样本表与样本结局、假 cargo、各组格函数与 `selftest`；只被入口的 `--selftest` 起，import `prove_red_core.py` |
| `crates-mutation-rows.py`、`mutation-shard-run.sh` | 门禁 crates-mutation-replay 的判法、分片那一套（门禁经 `task-server/task submit` 交一律分片，服务拆成计划、各片、合并三步，门禁照 `TASK_SERVER_SHARD_PHASE` 转给 `mutation-shard-run.sh` 的同一步，合并那一步接着核账）：每行一条判定，行键是被改坏的函数单元、原文、替换文、点名测试的单元闭包、开关与特性（构建环境：编译器与链接器相关的环境变量、cargo 配置、清单的 lints；rustc 与 cargo 的版本、Cargo.lock 里的依赖版本、编译档与线程数、驱动与算键库的代码、测试服务代码（`scripts/gate/test-service-code-paths.tsv` 登的）都不进）；一份库（prune-server 里 `mutation-units/singlefs-crates-mutation-library/`，库根照被判那棵树的 git common-dir 取、只当名字），两片判的都写进这一份，合并那一步只按此刻的行键核两片判的行在服务里读得到；变异复跑真跑测试的每一次各对自己起的一份临时 prune-server 跑（不读、不写真库），`crates-mutation-rows.py` 文件头「变异复跑的崩溃判定」 |
| `crates_mutation_rows_base.py` | `crates-mutation-rows.py` 拆出的底层：标准库与仓内库的载入、全部模块级赋值（常量、表、正则、假 cargo 与假 llvm 工具的正文、`WORKER_STATE` 这类共用的可变状态）、`break_is`、读变异表、拷贝范围的头一段；入口文件头列着拆出的各份与弄坏开关的代码各在哪一份 |
| `crates_mutation_rows_judging.py` | 判「抓没抓到」的函数：结局分类 `verdict_of_run`、基线参数与结局、起 cargo 的命令、拷每片源码、判一行与一批、覆盖复用判据；只 import base；驱动的代码不进行键 |
| `crates_mutation_rows_support.py` | 自证的脚手架：样本仓与样本库的造法、改文件与读调用日志的小函数 |
| `crates_mutation_rows_static.py` | 变异表的静态核：拷贝范围、锚点失效、替换文里没绑定的标识符、转义字面量、release 与验证包补行，连同样本仓上的自证格 |
| `crates_mutation_rows_units.py` | 几个子进程按内存配速同时跑的个数、判定库与运行记录目录、行键 `KeyContext`，连同自证格 |
| `crates_mutation_rows_baseline.py` | 基线的切法（按批、按块、长测试单列、派变异的次序），连同它与判结局的自证格 |
| `crates_mutation_rows_targets.py` | 工作进程的停法、片号与编译目录（共用目录、片号锁、占盘、磁盘余量、上限清理），连同自证格 |
| `crates_mutation_rows_memory.py` | 任务内存上限、进程树用量、每个工作进程的峰值、内存准入、工作进程数与跑着加开，连同自证格 |
| `crates_mutation_rows_runs.py` | 边跑边收与边跑边报、卡池目标、临时 prune-server、记覆盖、写用时，连同自证格 |
| `crates_mutation_rows_pool.py` | 先编一份依赖、工作进程初始化、三种任务与进程池 `run_pool`，连同自证格 |
| `crates_mutation_rows_plan.py` | 派活之前：读选行、`prechecks`、`plan_rows`、开 GPU 时补特性、static-check 可达性、判复用、牵连面，连同自证格 |
| `crates_mutation_rows_stage.py` | 一道的整个跑法：按项机读行、`run_stage`、复用哨兵、`CoveragePlanner`、llvm 工具、汇总报告，连同自证格 |
| `crates_mutation_rows_commands.py` | 命令行的各个子命令与直接调子命令的自证格 |
| `crates_mutation_rows_driver.py` | 自证的驱动：`Selftest` 类、格函数表、弄坏开关与格的对应表、挑格与并行、`run_selftest` |
| `lib_mutation_units.py` | `crates-mutation-rows.py`、`mutate.sh` 共用的函数单元键与按条判定库：被改坏的单元、点名测试是哪个单元、闭包键（切分与闭包调 `scripts/gate/lib_function_units.py`）、工具链行（编译器与链接器相关的环境变量、cargo 配置、清单的 lints）、编译器版本行与依赖版本行（闭包真点名的外部 crate 与传递依赖，规则与 `crates/singlefs-unit-graph` 的 `--lock-file` 相同；三张变异表的键平常都不收这三样，只在各自的弄坏开关下进键，`verus-proof-check.py` 与 `mutation-prune.py` 的弄坏开关也调依赖版本行那一份）、库的存放与多久没被引用就删；`mutation-prune.py` 另取它的 cargo 配置文件发现 |
| `crates_mutation_rows_selftest_fixtures.py` | `crates-mutation-rows.py` 自证的样本常量（样本 crate 的源码、样本变异表的行）：只在它的 `--selftest` 开跑时按路径载入（`crates-mutation-rows.py` 文件头「自证夹具」） |
| `lib_release_only_tests.py` | 重测试目标只许 release 的判法：给一组 cargo test 参数与仓根，答是不是必须带 release、依据（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行点到的测试目标，或特性里带 gpu）、已带没带；登记表读不了抛错、不当不命中。`crates-mutation-rows.py`（门禁补 `--release`、static-check 的 `RELEASE` 行）与 `prove-red.sh`（证红补 `--release`）都调它，按路径载入的调法写在它文件头；`--selftest` 只调判法函数 |
| `lib_broad_mutation_selftest_fixtures.py` | `broad-mutation.py` 的 `--selftest` 用的夹具：假的一步与假清单驱动的脚本正文、假源码树、各序号怎么答的计划表、期望的汇总。只在自证开头载入，驱动运行期的函数一个都不读。没有命令行入口 |
| `broad-mutation.py`、`mutation-prune.py` | 广谱变异的驱动、剪枝规则（第二段 验证包的测试里，登记的崩溃枚举用例与放量分项经 `--skip` 排掉，排除单读 `scripts/gate/admission.py` 的 `registered_test_functions`）：清单与结论都按单元键存在判定库服务里（`scripts/prune-server/prune_server_client.py` 的族 broad-mutation，没有「一版」与版本目录；库目录 `<数据根>/broad-mutation-units/` 只放运行目录、日志与锁）；每条变异的键是被变异的函数单元与它调到的单元、原文、替换文、抓它的测试的单元闭包（切分与闭包调 `scripts/gate/lib_function_units.py`；构建环境、rustc 与 cargo 的版本、Cargo.lock 里的依赖版本、编译档与线程数、驱动与算键库的代码都不进），不带参数就是剪枝（沿用得了沿用、沿用不了现判）；不按提交、不按 HEAD 选条 |
| `broad_mutation_base.py` | `broad-mutation.py` 拆出的底层：标准库与仓内库的载入、全部模块级赋值（常量、列名）、入口与各份模块的路径表；入口文件头列着拆出的各份与弄坏开关的代码各在哪一份 |
| `broad_mutation_judging.py` | 选条、沿用、结论分类（`Judgement`）、GPU 一趟的判法（`GpuJudging`）、`process_item`、`refuse` / `say` / `note` / `test_mode` / `break_is` 这类小函数；只 import base；驱动的代码不进键 |
| `broad_mutation_library.py` | 判定库 `Library`、读表、库目录与源码树、导出范围、清单的生成与缓存、规矩库与复用三问库的载入、变异补丁的施加与切分 |
| `broad_mutation_runner.py` | 步的跑法 `Runner`、libtest 与 nextest 结果的认法、工具版本、改坏与还原源码 |
| `broad_mutation_gpu.py` | GPU 那一趟判法之外的部分：门控测试文件、编译参数、判定行的读写、`GpuPass`、GPU 一趟的编译与跑 |
| `broad_mutation_run.py` | 一次跑：选条、算键与读库的提速、汇总、参数、`run_items`、复用哨兵、分片的计划与计数 |
| `broad_mutation_selftest_a.py` | 自证（一）：`Selftest` 类与基础、状态、表、环境、同步、剪枝、复用、算键提速几组格、自证夹具只被自证读那一组 |
| `broad_mutation_selftest_b.py` | 自证（二）：基线、服务、编译目录、登记、GPU 几组格，格函数表、串并行、`run_selftest` |

审核回归：`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/mutation/tests/test_audit_regressions.py --selftest`（等于在临时 prune-server 底下跑 `python3 -m unittest discover -s scripts/mutation/tests -v`；判定库那几条只连临时服务，裸跑 `--selftest` 退 2；门禁 code-tooling 的 research-script-selftests 格照这一句跑）。覆盖配置取值、缓存键、并发索引合并、过期回收、信号退出、参数取值，以及证红的副本边界、基线和源码恢复；使用临时目录与假命令，不跑真实变异。各驱动原有的 `--selftest` 仍需单独运行。
