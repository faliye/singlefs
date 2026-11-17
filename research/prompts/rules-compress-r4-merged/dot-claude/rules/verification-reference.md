# 验证纪律的参考：按需读

判据在 `.claude/rules/verification.md` 与 `.claude/rules/implementation-workflow.md`；`verification-reference.md` 放那两份到那一步才用的做法与明细，节名与那两份里的节名相同，脚本文件头写全了的只指过去。

## 定义与名字

什么时候读它：要知道验证包里哪个模块装什么、哪几个挂特性 `gpu`、哪几道门禁连带判它时。

- 验证包的模块：录制器、内存池（`memory_pool`）、理想模型、随机历史、故障注入设备、场景；崩溃态枚举引擎与记录核对器（`crash`）；崩溃枚举用例经判定库的两段（流先入库、再从库里读回判，判在 GPU 或 CPU）与它的计数（`crash_state_enumeration`）；崩溃点重放发现日志与工具链身份（`crash_point_replay_progress`）；一份长期判定库与两段协议（`crash_library`；块键只由内容定，测试的类只当索引）；崩溃注入与坏盘输入两场战役（随机历史两项的给定状态表、它的判器 trait 与 CPU 就地判在 `crash_given_states`）；可写挂载每个写落在哪一段的划分与清扫段的写（`writable_mount_phases`）；真设备一侧的设备日志比对与运行模式；崩溃放量：崩溃点身份与复用键 `crash_identity`、先录后核的流水线 `crash_amplification`、判定存储 `verdict_store`、单元校验和上 GPU `gpu_unit_checks`、三段流第 ① 段的事实表与 CPU 参照核对 `crash_facts`、第 ② 段的 GPU 核对内核 `crash_verify_gpu`、GPU 判器的输入表 `crash_judge_tables`、GPU 判器的判法 `crash_judge_gpu` 与它的派活 `crash_judge_dispatch`。`gpu_unit_checks`、`crash_verify_gpu`、`crash_judge_gpu` 与 `crash_judge_dispatch` 挂特性 `gpu`。
- 验证包连同这几道门禁判的东西（都照 `.claude/rules/verification.md`「一个门禁：两种跑法」跑）：QEMU（门禁 qemu-device-streams）、herd7（lkmm）、crates 变异整表（crates-mutation-replay）、全部实验复跑（research-build-and-replay 的 experiment-replay 格）、形式化验证（formal-verification：Verus 验 `.claude/gate.d/verus-pins.tsv` 点名的各 crate，`crates/singlefs-replay-prefix` 与 `crates/singlefs-core` 的被验区域，E165 红例钉表两格）、广谱变异（驱动 `scripts/mutation/broad-mutation.py`，判什么在「门禁管哪一半」broad-mutation 那一行）。

## 一个门禁：两种跑法

### 门禁 crash-point-replay 怎么跑

什么时候读它：改这道门禁起 cargo 的那几趟、照它的写法单跑一趟、或要知道一趟有多少条要现判时。分片、按需领卡、复用与各格的判法以 `.claude/gate.d/crash-point-replay.sh` 文件头「逐项点名」「复用」「分片」「feature-gated-tests 格」「product-gpu-real-card-tests 格」「显卡与清场」为准。

- 经服务交的每一种跑法（不带参数、`--check`、`--item`）都照文件头 `# task-server-shards: *` 拆成计划、各片、合并：各片只现判 crash-case-replay 格里起 cargo 的两样（按条复用那一趟只现判归这一片的测试目标，登记的崩溃枚举用例只判这一片领的块），合并那一步照常判九格、crash-case-replay 格核账不现判（按条复用那一趟还有键对不上的、崩溃枚举用例核账那一趟现判了块的，都判红）。`gate.sh` 整轮与样本档没有分片阶段，这道门禁自己在一件里依次走计划（1 片）、单片（`SHARE=0/1`，键对不上的测试函数与库里没有的块在这一步现判）、合并（核账），复用同一套代码、不另有不分片的路，单片红整件退单片的码、不起合并。
- crash-case-replay 格起 `cargo test --release -p singlefs-verification --lib --tests`：库、集成测试与装置二进制 `src/bin/` 的内联单测（`--tests` 带上全部 `test = true` 的目标，装置二进制都是）。包里全部测试函数按单元键复用，登记的崩溃枚举用例与放量分项经 `--skip` 排掉、交按块那一路逐条跑。
- feature-gated-tests 格再起一趟 `cargo test --release -p singlefs-verification --features <tests/ 里 cfg 点到的特性> --lib --test <挂特性的测试文件>…`；`BINDGEN_EXTRA_CLANG_ARGS` 取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。特性里有 `gpu` 的那一趟照旧一条 `cargo test`；不带特性那一趟与特性里没有 `gpu` 的那一趟经 `python3 scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数> -- <cargo test 的参数>` 按测试目标并行起，`--jobs` 取 `.claude/heavy-thread-default`，第二个 `--` 之后的 `--nocapture`、`--exact --skip` 原样交给每个目标。
- 要上卡的三处（登记的崩溃枚举用例里带 `gpu` 特性的命令、feature-gated-tests 带 `gpu` 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）照文件头 `# task-server-card-lease: on-demand` 到上卡那一步才领卡，各自怎么领在文件头「显卡与清场」；领卡没成的各自怎么办在文件头同一段：崩溃枚举用例带 `gpu` 的，录好的那几条判红、一条都不判；product-gpu-real-card-tests 的对拍那一格判红、那一趟不起；feature-gated-tests 带 `gpu` 那一趟由测试进程自己领，领卡没成的测试目标记本次未跑、这一格判红，排在最后的那一趟不再起。
- 要知道一趟有多少条要现判：门禁 `crash-point-replay`、crates-mutation-replay、formal-verification 都用 `--list-items --with-keys`（`crash-point-replay` 逐条点名拒算的测试函数与理由）；崩溃点重放入口 `scripts/crash/crash-point-replay.sh --list-items --with-keys` 按项打 `ITEM_PLAN` 行。

### 崩溃放量入口

什么时候读它：起或改 `scripts/crash/crash-amplification.sh` 与它的分项时。用法、退出码表、按需领卡、分片与展开深度以入口与分项驱动 `scripts/crash/crash-amplification-item.sh` 的文件头为准。

- `bash scripts/crash/crash-amplification.sh` 不带参数跑登记表里全部分项（`--list-items` 列、`--item <分项>` 点名），每项交 `crash-amplification-item.sh`，机器与卡由服务分；多机配置 `ENABLE_GPU=1` 时由 GPU 判。
- 用不上配置开了的 GPU（配置对不上、点名的卡用不了、第二台连不上、清场没过）退 78（`.claude/rules/test-script-discipline.md`「用上现有的硬件加速」），算法上判不了（有状态 GPU 判不了、内核展开超预算、建不出表）退 248。
- 判器整套上了 GPU，罩全部池级 checker 不变量（D13（验证路线） 已定项 15，`crates/singlefs-verification/src/crash_judge_gpu.rs`）。

### 崩溃放量的块键

什么时候读它：改崩溃放量的复用、或判一块为什么现判时。键的算法以 `crates/singlefs-verification/src/crash_identity.rs` 为准，存放布局在 `crates/singlefs-verification/src/verdict_store.rs`。

复用与续跑照 `.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」。每块的键是块键（这一步的写表与起步镜像的摘要、段展开、判器另读的输入与段内序号）加判这一块的判器组的单元键（判器入口的函数单元闭包，WGSL 按 fn 切、去注释；闭包真引到的 cfg 原子各带这一次编译的真值；工具链与 cargo 解析后的构建档；`Cargo.lock` 里闭包真引到的外部包）；闭包没引到的 cfg 原子不进键，带不带 `gpu` 特性编的两趟里闭包不提 `gpu` 的判器组键相同、判定共用这一份库。判器组的单元键由 `crates/singlefs-verification/build.rs` 经 `crates/singlefs-unit-graph` 按 `crates/singlefs-verification/src/judging_units_entries.rs` 算好编进二进制。崩溃放量、崩溃枚举用例、给定状态表与两种对拍的判定都在 prune-server 那一份库里，不随趟、不随判器换，测试的类只当索引。

### 各道的判定单位

什么时候读它：要知道某道门禁按什么单位入库、判红怎么记时。各道的键与键前缀以它的文件头为准。

- 按判定入库的单位：qemu-device-streams 每档、lkmm 每份 litmus 两条（herd7 与绑定各一条，绑定那一条的键另含锚点指到的函数单元指纹）、crates-mutation-replay 每行、formal-verification 每条、broad-mutation 每条变异、research-build-and-replay 的 experiment-replay 格每行复跑、mutation-tables 格每条变异；门禁 `crash-point-replay` 按条复用那几趟每条测试函数一条、登记的崩溃枚举用例每个崩溃状态块一条。键对得上的复用、对不上的现判；整道、整格没有复用判定。
- 判红照同一把键记红、撤掉原有的绿、下一趟现判的：`crash-point-replay` 按条复用那几趟、model-differential-and-scenarios 各模型对拍格、research-build-and-replay 的 research-unit-tests 格、experiment-replay 格（对不上的）与 mutation-tables 格（没被抓到或编不过的）、code-tooling 按条复用的各格（research-script-selftests、reuse-selftests、function-units-two-implementations-agree、mutation-list-items、entry-scripts 与两格开关复跑）、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification。
- 按判定入库的例外里，prune-server 自己的自证是门禁 code-tooling 的 prune-server-selftest 格这一类。

### 缩域与崩溃域上限

什么时候读它：要缩崩溃域、或指定范围的那一趟被估算拒了时。

- 认 `--crash-expand-up-to <n>` 的是崩溃点重放入口、崩溃放量入口与 `scripts/gate/merge-and-verify.sh`；环境变量是 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`、`SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`，命令里怎么写、闸照哪个目标认哪一个，照 `.claude/agent-common.md`「不做」一节「门禁只有一个，跑法两种」那一条。
- 合入后验证（`scripts/gate/merge-and-verify.sh`）没给 `--crash-expand-up-to` 时走同一道估算：不超上限照旧，超上限或有项估不出而算得出 n 用这个 n 缩域接着跑，算不出 n 缩到 5 个展开深度（`CRASH_DOMAIN_FALLBACK_DEPTH`）接着跑，这两种汇总都写缩过域；给了值照给的；判法在 `scripts/gate/merge-and-verify.sh` 文件头「崩溃域上限」，估法与上限在 `scripts/crash/lib_crash_point_replay_items.py` 文件头「指定范围的崩溃域上限」。不指定范围那一趟不受影响。

### 谁跑得了与范围声明

什么时候读它：命令被 `.claude/hooks/project-code-guard.sh` 或 `.claude/hooks/run-scope-guard.sh` 拒了、或要写范围声明时。

- 谁跑得了由 `.claude/hooks/project-code-guard.sh` 判（判定与逐类写法在 `.claude/hooks/lib_project_code_runs.py` 文件头）：会编的 cargo、编出来的代码、本仓的代码与 `qemu-system-*`、`herd7`、`e152-file-system-benchmark` 这类要经服务的外部程序，不经 `task-server/task submit` 直接起的执行前拒；套在包装（`bash -c`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里的与命令位置上执行的脚本里的照判；子 agent 起 `task-server/task` 的 submit、wait、cancel、clean 执行前拒。要核弄坏开关（`CRASH_POINT_REPLAY_BREAK`、`FORMAL_VERIFICATION_BREAK`、`GATE_MUTATION_BREAK` 这类 `*_BREAK`）下的判红，命令写进报告交主 agent。派发提示里叫子 agent 跑项目代码的句子由 `.claude/hooks/runner-dispatch-guard.sh` 拒。
- 范围声明四段的写法、种类与范围数的判法、拒什么以 `.claude/hooks/run-scope-guard.sh` 文件头为准；拒绝说明附上次红的清单（取自 `~/.local/share/singlefs/last-runs/`，每道门禁与入口跑完写，写法在 `scripts/gate/lib_reuse_checks.py` 文件头「三」）。

### 复用自证

什么时候读它：给带判定库的门禁或入口写、改自证时。

带判定库的门禁与入口（登记在 `.claude/gate.d/verdict-library-scripts.tsv`），自证里各有一格 `reuse-three-questions`：三问照 `.claude/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」，判法在 `scripts/gate/lib_reuse_checks.py` 文件头「一」；门禁 code-tooling 的 reuse-selftests 格按登记表逐份核那一格的规范输出。哨兵与强制现判的判据在 `.claude/rules/verification.md`「一个门禁：两种跑法」，抽法、种子与条数在 `lib_reuse_checks.py` 文件头「二」。

### 凡用 GPU 先清场，跑完不复原

什么时候读它：跑或改用 GPU 的测试、入口与门禁时。判据（清场与分卡由服务做、调用方不套、跑完不复原、卡坏当场弹窗）在 `.claude/rules/verification.md` 同名一条。

- 算在内的：崩溃放量入口与分项驱动、带 `gpu` 特性的 cargo（GPU 对拍、门禁带 `gpu` 特性那一趟）、手跑的 GPU 测试。手跑的 GPU 测试编与跑写成一条交，不先另交 `--no-run` 只编的那一条；门禁与入口脚本照各自的起法。
- 起跑就分卡的（手跑、经服务直接交的带 `gpu` 特性的 cargo）清场回读没过不跑、退 78：卡上有不归清场白名单的计算进程时就是这样，等那个进程跑完再交，不停它。
- 文件头声明了 `# task-server-card-lease: on-demand` 的门禁与入口（崩溃放量入口 `scripts/crash/crash-amplification.sh`、门禁 `crash-point-replay`）自己不清场、不挑卡：服务起跑不分卡、不清场，跑到上卡那一步经 `task-server/task lease take` 领一张，服务对那张卡清场回读过了才交；领卡时清场没过由服务隔一会儿换卡再挑，领卡那一方照等。
- 卡坏：清场之前查卡或清场回读判出卡坏，清场那一行写 `result=card-fault`，服务把那一件的结局记成 `card-fault`。
- 要复原，手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0。
- 单个 GPU 测试二进制只用分到的卡（哪几张卡由服务分）；派活（哪件上哪张卡）与判法拆开。

### 测试线程

什么时候读它：起 `cargo test`、改变异那一路或 GPU 对拍的并发时。

- 控制测试线程的只有变异那一路。别的 `cargo test`（门禁 `crash-point-replay` 各趟、`scripts/crash/crash-point-replay.sh`、`scripts/gate/admission.py` 拼的 crash-case 命令、崩溃放量、手跑的 GPU 测试）参数里不带 `--test-threads`，测试线程取登记的每台默认重活线程数（`.claude/heavy-thread-default`，服务经 `RUST_TEST_THREADS` 交下来）。
- 变异那一路：`scripts/mutation/crates-mutation-rows.py` 起的 `cargo test`（基线的批、变异、记覆盖的插桩跑）特性里有 `gpu` 的带 `--test-threads 1`，点名的测试目标全在放开名单里的带登记值（放开名单是它的 `GPU_TEST_THREADS_RELEASED_TARGETS`，加验证包 `tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs` 的 `FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL` 登记的只经卡池取卡的测试目标，照被判那棵树现读），`--` 之后行里自己已写 `--test-threads` 的不重复加；`scripts/mutation/broad-mutation.py` 的 GPU 那一趟带 `--test-threads 1`。
- 同一个测试二进制里并行的 GPU 用例经卡池按显存排队（`crates/singlefs-verification` 的 `gpu_test_start`）；测试二进制之间一个接一个上卡：卡池的显存账按进程记，几个进程同时上卡互相不排队。
- CPU 一侧的并行写在用例里，用例自己起多线程。GPU 对拍与内存对拍不压并发：用例内部的 CPU 线程数由用例自己读的变量定，GPU 对拍测试的 CPU 一侧只读 `SINGLEFS_THREAD_CAP`（`.claude/heavy-thread-default` 那个参数的覆盖名），没设取登记的每台默认重活线程数；崩溃放量与对拍驱动的机器、卡与内存显存额度由服务分。

## 判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB

### 判定库里的表

什么时候读它：要核判定库有哪几类表、某类判定该落哪张表时。表的布局与键的字节写法以 `crates/singlefs-verification/src/verdict_store.rs` 文件头为准，Python 一侧的族与种类以 `scripts/prune-server/prune_server_client.py` 文件头「键空间」为准。

- 库是 prune-server 服务状态目录下的 `library`，布局只一种：全部表在默认列族、键原样、开库选项照 `verdict_store.rs`。
- Rust 一侧 `verdict_store.rs` 文件头登记的 29 张表：判定块、判定向量两张、违例、录入块、路径索引、事实表、核对表、判法单元组成清单、布局行、结论块与结论违例、恢复字、计数向量、块属性、录下的流程、一趟的清单、判表、给定状态表、片进度、观察者四张、被抛弃根、配对结论、对拍输入、读集见证两张。
- Python 一侧的五族：verdict-library、unit-verdicts、mutation-units、broad-mutation、apparatus-items，各自种类下的条目。

### 库名与命名空间

什么时候读它：脚本取不到库名、或要核主仓与仓副本读写的是不是同一份时。

各道的库名是被判那棵树的 git common-dir 下它自己的固定目录名（`scripts/crash/lib_verdict_library.py` 的 `library_directory`，文件头「存放」），只当名字，本地不建、不读、不写；键里只取固定目录名，哪棵仓不进键，主仓与 git init 过的仓副本读写同一份（广谱变异的种类固定是 `broad-mutation-units`，不随库名）；被判那棵树取不到 git common-dir 的整趟拒、一项都不判，不换别的库名。只在临时服务底下另带一份按库目录分的命名空间（`prune_server_client.py` 文件头「键空间」）。

### 判定库登记表的现状

什么时候读它：改 `.claude/gate.d/verdict-library-scripts.tsv`、或判某份脚本该怎么登记时。各列的写法以那张表的表头为准。

- 第三列：写了判定库的行都写 prune-server（一行脚本跨几族的按族各写）；登记表射程外带判定库的脚本照样逐个登记一行，第三列写 prune-server。第三列不是 prune-server 的只许是「不算：」行（只提到别人的复用）与「无判定库：<欠账号或待立>」行。task-server 的拒法：命令点到第三列不是 prune-server 的带判定库那一行的，提交时当场拒（`task-server/task_server.py` 文件头 ②「提交时拒」③），不按开关。
- `scripts/experiment-apparatus/replay.sh` 与 `.claude/gate.d/research-build-and-replay.sh` 两行是 unit-verdicts 族的 replay-row 种类，读写只经 `scripts/gate/admission.py` 的 `library-files` 走服务（replay.sh 文件头「判定库」）。
- 第三列写「无判定库：待立」的是 `.claude/gate.d/fixtures`（门禁样本档）一行。
- 登记表射程外带判定库、照样逐个登记的脚本，例：E171（三oracle合谋测试）的 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py`、E203（准入之前扫类的步代价）的 `scripts/experiment-apparatus/e203-admission-scan-cost-run.sh`。
- 中转库的例子：`scripts/gate/admission.py library-files` 给 replay.sh 与 E182（被抛弃根读不出的三条岔路）两份长跑脚本搬判定用的中转目录。

## 门禁的结构

什么时候读它：写或改一道门禁、读它的参数、汇总与退出码时。参数解析、样本标记、逐格汇总与整道退出码的判法以共用库 `.claude/gate.d/lib/stage-cells.sh` 文件头为准，样板是 `.claude/gate.d/doc-decisions.sh`。

- 门禁 source 共用库（住在 `lib/` 下，`gate.sh` 不把它当阶段跑），用 `stage_cell <格名> <函数> <判什么的一句> <判红时的出路>` 逐格登记；参数解析、样本标记、逐格汇总都经它，不各写一份。门禁 `crash-point-replay` 不 source 它跑格，参数照同一张表自己解析。
- 文件头写格名表：每格一行 `# gate-cell: <格名> <判什么的一句>`，与登记的逐格相同、次序相同。

| 参数 | 做什么 |
|---|---|
| 不带 | 全部格按登记次序跑 |
| `--list` | 逐行打「格名、制表符、判什么」，不跑任何格，也不起任何重活 |
| `--check <格名>[,<格名>…]` | 只跑点名的格 |
| 格名写错、`--check` 后面缺格名 | 列出可用的格名，退 2 |

其余参数（项目根、`--force`、门禁自己的选项如 `--write`）原样交给门禁。

- 汇总：每格在自己的子 shell 里跑，汇总逐格列「格名：绿 / 红 / 本次未跑」；整道的退出码（红、全退 77、有格退 75 各怎么退）与退 77、75 的格往 `$GATE_NOT_RUN_FILE` 报，照共用库文件头 `stage_cells_run` 一段。红格后面接它自己登记的出路与只重跑红格的 `--check` 命令。
- 样本要证某一格时，在样本根放 `.gate-cells`（一行一个格名），门禁只跑点名的格；被判的根是门禁所在的仓、或是 `gate.sh --staged` 的临时树时，这个文件不认。
- `--list` / `--check` 只管格，与文件头 `# gate-cell:` 表逐格相同。按项判的门禁逐项点名另用两个参数：`--list-items` 逐行打可点名的项，不编、不跑；`--item <项>` 只跑点名的项，给几次取并集，项写错退 2 并列出可点的项。可点的项：`crash-point-replay` 按崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）与挂特性的测试文件（它文件头「逐项点名」）、qemu-device-streams 按模式、research-build-and-replay 的 experiment-replay 格按实验、crates-mutation-replay 按变异行、formal-verification 按条（「formal-verification 的项」一节）。
- `scripts/gate/gate-structure-check.py` 判 `.claude/gate.d/` 顶层每道门禁（`lib/` 下的不判）：文件头有格名表；`--list` 退 0、输出与格名表逐格相同、跑的时候没起 cargo、QEMU、herd7 这类重活；`--check` 不存在的格退 2。
- 阶段归属表 `.claude/gate.d/stage-owners.tsv` 一行一个门禁与格名：门禁、格名、判红时派给谁修、为什么。

### formal-verification 的项

什么时候读它：要点名 formal-verification 的某一项、或核它的项怎么切时。判法以 `scripts/gate/verus-proof-check.py` 文件头「逐项点名与按条复用」为准。

`.claude/gate.d/verus-pins.tsv` 登记的各 crate 逐个判。证明的项怎么切（哪些单元成项、哪些不成项、项名的写法）在 `scripts/gate/verus-proof-check.py` 文件头「Verus 逐项验」那一段（`proof_items`），整格跑时每个 crate 各项验过的义务之和等于登记表里它那一行登记的数（`.claude/gate.d/formal-verification.sh` 文件头 verus-proof 那一行 `# gate-cell:`）。`toolchain`、`proof`（整格）、`table` 各一项；必红世界每行、空真每处 `requires`、红例钉表每行各一项。点名单跑一项与它在整格里跑的判定相同。每条的单元键：证明项是被验函数单元连同它 `requires`、`ensures`、`decreases` 与函数体用到的 spec fn、proof fn、类型、常量的闭包（`verus-proof-check.py` 文件头「被验函数单元闭包」那一段）；钉表每行是那一行档与参数重放真走到的装置函数单元闭包，连同状态输入与期望逐态行。键与判定库写在 `formal-verification.sh` 文件头「复用」一段。

## 门禁管哪一半

什么时候读它：查 `.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md` 的一条判据由哪道门禁、哪一格判，或一格红了要知道它判的是什么时。各格的格名表以门禁文件头的 `# gate-cell:` 为准。

| 判什么 | 谁判 |
|---|---|
| 崩溃点重放跑了；包里全部测试函数按单元键复用或现判都绿；每条登记的崩溃枚举用例（登记行第三列带 `separate-run=` 的没被 `--item` 点名时不跑、报本次未跑）的崩溃状态块，判器结论与挂在这个点上的观察者结论在判定库里都有单元键相同、不红的判定，缺的在各片里判，有红块判红；登记行第三列写了 `red-blocks=judged-by-test` 的用例按 `cargo test` 的退出码判、块那一行只核 `blocks = reused + fresh + refused`；合并核账那一趟还有键对不上的测试函数或现判了块的判红；登记了 `exhaustive=<前缀>` 的，那一行计数行要带 `exhaustive=true`（缩过域的那一趟 `exhaustive=false` 不判、成功句写缩过域、不算门禁通过；这一行没有 `exhaustive=` 字段照判红：`scripts/gate/admission.py` 文件头 `exhaustive=<前缀>` 那一条）；用例跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（`admission.py` 的 `judge_worker_threads`，它的 `--selftest` 拿合成日志核，工作线程照单机判）；挂特性的验证包测试文件：跑到加复用 0 条、每处 `NOT_RUN_NO_GPU_ADAPTER` 都报「本次未跑」、不判红，整道照样退 0，一个挂特性的文件都没有记无对象可判（`.claude/gate.d/crash-point-replay.sh` 文件头「feature-gated-tests 格」那一段；格名表那一行写「不为 0」，与这一段不一致，以这一段为准）；产品 GPU 包的测试包带 `tests-on-a-real-gpu` 的真卡对拍判红整格红 | 门禁 `crash-point-replay` 的 crash-case-replay、feature-gated-tests、product-gpu-real-card-tests 格；判的是合并核账那一趟的日志，各片只看 `cargo test` 的退出码；覆盖声明 `# gate-covers: 崩溃点重放`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃放量的分片驱动只经服务的分片协议起，计划、各片、合并三步都对 | `scripts/crash/crash-amplification-two-hosts.sh --selftest`（假的服务照分片协议逐步起它）；派到第二台的那一片在同一棵共享的树里、同一个绝对路径上跑，不送树、不取回（`task-server/task_server_lib.py` 文件头「挑机器」「分片」） |
| 形式化验证：恢复重放取前缀、择根、扫环三截的核与择根取前缀的合判推论经 Verus 验过；E165 的红例钉表自己成形、在被判那棵树的实现上逐行重放与原实现相同（红例都钉进钉表由 E165 第二段驱动写账时当场判，不归这道门禁）；不一致事件表里每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红 | 门禁 formal-verification（`.claude/gate.d/formal-verification.sh`，判法与弄坏开关在 `scripts/gate/verus-proof-check.py` 与 `scripts/experiment-apparatus/e165-pin-red-examples.py` 文件头；各格判什么在「formal-verification 各格」一节） |
| 广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红 | 门禁 broad-mutation 的 gate-verdict 格：判驱动 `--summary` 的 `BROAD_MUTATION_GATE` 行与汇总行；驱动错、只过了第一段算没结论，驱动 `--summary` 退 1（判定库或清单表对不上、结局认不出）照判红；经服务分片起的那一趟（不带 `--check` 经统一任务服务交）计划与各片先起驱动剪枝跑，合并那一步驱动核账（分片没判到的条判红、一条不现判），退 0 再读 `--summary`；不是服务分片起的（`gate.sh` 整轮、手跑）与 `--check gate-verdict` 指定范围的不起驱动、只读库，判库里此刻有没有全部条的结论（文件头「判什么」那一段）；判定与被停的那一条怎么记在「broad-mutation 的判定与被停」一节 |
| 模型对拍跑了、每段都判过 | 门禁 model-differential-and-scenarios 以 `model-` 起头的那几格；覆盖声明 `# gate-covers: 模型对拍`，键登记在项目根 `.claude/gate-not-implemented.tsv` |
| 崩溃枚举用例住在验证包的 `tests/`、不标 `#[ignore]`、登记进崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv`（点名 `crash-case:` 行或放量分项）；验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块（`scripts/crash/crash-case-check.py` 的 modules）| 门禁 `crash-point-replay` 开跑前在真仓上跑 `scripts/crash/crash-case-check.py --only placement,modules`（placement 那一样判写在别的包里、标了 `#[ignore]` 或没登记的），红了不起 cargo；门禁 code-tooling 的 research-script-selftests 格跑那份脚本的自证 |
| 函数名与类型名不只由空泛词拼成；测试文件不按里程碑起名 | 门禁 code-source-discipline：vague-names 格读 `.claude/naming-vague-words` 与 `.claude/naming-vague-exclude`；test-file-names 格在真仓上跑 `scripts/crash/crash-case-check.py --only file-names` |
| 池级 checker 库不依赖实现；RefFS 的依赖只有 `std`（门禁那一格还许 `singlefs-format`，收严记在欠账）、源码零处引实现、池级 checker 与验证包（真名与改名别名都算）、实现与池级 checker 的依赖闭包里没有它 | 门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格（⑤⑥⑦ 判 RefFS 那三样；RefFS 包不在时 ⑤⑥ 报本次未跑） |
| 判器调用点（池级 checker、记录核对器、GPU 判器）合规：判法文件头「一处调用点算合规，有一样成立就行」那一段列了落在判定段里、落在判器本体里、包住它的函数带 `// gpu-exempt: <截名>` 或 `// cpu-reference-for-gpu-comparison` 标记、文件在登记表 excluded 里、落在判器自测里；一样都不成立的判红 | 门禁 checker-independence-and-sync 的 cpu-judging-sites 格：判法 `scripts/crash/cpu-judging-sites-check.py`（合规的几种位置与标记写在哪，在它文件头「一处调用点算合规」那一段），判定段与判器本体登记在 `.claude/gate.d/cpu-judging-sites.tsv` |
| 验证包一条用例一个场景 | 门禁 model-differential-and-scenarios 的 one-scenario 格（在真仓上跑 `scripts/crash/crash-case-check.py --only one-scenario`，读 `crates/singlefs-verification/tests/` 的用例源码，不跑测试） |
| 变异行点名的测试跑得到：`--` 之后的筛选词筛得到点名的测试 | 门禁 code-source-discipline 的 mutation-tables 格 |
| `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 | 门禁 lkmm（`.claude/gate.d/lkmm.sh`，逻辑在 `.claude/scripts/lkmm.sh`） |
| 两块 virtio 盘上六个模式各档负载跑完，设备侧独立录制与程序录制流逐项相同，漏一道屏障与走页缓存两个对照红在该红的地方 | 门禁 qemu-device-streams（`.claude/gate.d/qemu-device-streams.sh`） |
| 每道阶段认第一个参数当项目根 | 门禁 code-tooling 的 stage-owners 格 |
| 跑项目代码的命令经没经任务服务、写法认不认得出 | `.claude/hooks/project-code-guard.sh`（判法在「谁跑得了与范围声明」一节） |
| 登记的自证与样本档不起真工具 | `scripts/gate/selftest-process-guard.py` 盯 `.claude/hooks/selftest-registry.tsv` 登记的自证与样本档跑的整棵进程树，起了真 cargo、rustc、`qemu-system-*`、herd7、verus 就判红 |

### formal-verification 各格

什么时候读它：要知道 formal-verification 哪一格判哪一样时。格名表（七格）以 `.claude/gate.d/formal-verification.sh` 文件头为准。

- verus-toolchain-pinned、verus-proof、verus-vacuity、verus-must-red-worlds 四格用 Verus 逐个 crate 验登记表 `.claude/gate.d/verus-pins.tsv` 的 crate 行点名的被验区域；verus-toolchain-pinned 另核 Verus 与产品的 rustc 版本与登记表的 `rustc` 行相同。
- verus-proof 起 Verus 之前照信任清单 `.claude/gate.d/verus-trust-list.tsv` 判被验区域里的可信声明与 cfg 原子（逐条登记；axiom、作弊标记与 `verus!` 外面被验类型的 trait impl 判红），整格跑时另比产品构建与 `verus --compile` 构建的 MIR 逐条目相同。
- 红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格：pinned-red-examples-table 只判钉表自己（每行 22 段、指纹、抄来的段、档的写法、点名的变异还在，不读红例账、产物与 `research/`），pinned-red-examples-replay 逐行重放。
- same-origin-counter 读不一致事件表 `divergence-events.tsv` 一张表，不看依赖：每个已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红，未立的对报未立、不计入。
- 钉全由 E165 第二段驱动 `scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh` 写完一对红例账与产物时当场判：追加、再 `--check` 同一对，没钉全那一趟判红（它文件头「钉红例」一段）。

### broad-mutation 的判定与被停

什么时候读它：读 broad-mutation 的判定、或它的一条被停了时。

- 判定在 prune-server 里：broad-mutation 族的 `verdicts/` 等七张表（驱动文件头「判定库」），判定行不带校验列。
- 驱动不给任何一步设内存上限、不限时，撞上限与卡住由统一任务服务管；被停的那一条不落行、不算抓到（`scripts/mutation/broad-mutation.py` 文件头「内存与限时」）。

## 复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判

### 单元键的组成与排除

什么时候读它：改算键的代码、判法新读文件、登记文件级或边级排除时。键的逐项组成以 `scripts/gate/admission.py` 文件头「单元键」为准；一句定义在 `.claude/rules/verification.md`「一个门禁：两种跑法」。

- 文件级排除：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。crates 变异判法不在这条排除的射程里：那里两份照进判法单元指纹，覆盖号带判法单元指纹。
- 边级排除：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`。每一条登记的边配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。
- python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头（`lib_function_units.py` 文件头「Python 闭包：import 时执行的模块级语句」）。
- 判法运行期按字面尾巴整组收的数据文件，收进来的不许落在 `records/`、`research/` 下：照那一处真走到的地方登记 `TAIL_READ_SCOPES`，读的是调用方交进来的输入的登空元组（`lib_function_units.py` 文件头「按尾巴读的登记」）；算键库自证的 `TAIL_READS_RECORDS_REAL_CELL` 在真仓判，点名读法的 文件:行 与收进来的文件。
- 作废的判定报得出是哪几个函数单元变了，靠键的组成清单随判定落盘。

### 复用核查在快照树上怎么做

什么时候读它：要量复用对不对（同一份输入跑第二趟现判 0、哨兵与强制现判那一条）时。

- 快照树用 `python3 scripts/hosts/minimal-copy.py <仓根> <快照树> --with-gate-tables --with-test-inputs` 取（用法在 `scripts/hosts/minimal-copy.py` 文件头「用法」一段），要在树里起的脚本照它的 `--with <条目>=<为什么>` 逐条声明；快照树放在自己认领的草稿目录下，树里起命令要的多机配置照 `python3 scripts/hosts/lib_repository_copy.py configuration-path <源树>` 打的那一份显式交给 `SINGLEFS_MULTI_HOST_CONFIG`（多机配置决不拷进副本；`minimal-copy.py` 文件头「副本里起命令的」那一句）。
- 两趟都经服务在快照树里交：submit 的 `--cwd` 与门禁、入口的根都写快照树，范围声明照常写；两趟之间谁都不改快照树。
- 第二趟有现判的，按它成功句点名变了的函数单元查，查完再判复用对不对；量完删掉快照树。

## 测试与崩溃检测优先多线程

### 线程数怎么取

什么时候读它：写 Rust 装置或测试、要取线程数时。

Rust 装置取登记的每台默认重活线程数的写法在 `scripts/experiment-apparatus/heavy_threads_default/threads.rs`。测试线程的取法在「测试线程」一节。

### 分片三步

什么时候读它：改某道门禁或入口的计划、各片、合并时。服务一侧的协议以 `task-server/task_server_lib.py` 文件头「分片」为准。

各道三步怎么做在各自文件头：门禁 `crash-point-replay` 在它文件头「分片」（片数照这一趟要现判的量定，不超过上界：多机配置点名的卡数，没开 GPU 取 `HOSTS` 的台数，点名的用例登记了 `shards-at-least=` 的取它与卡数的大者）；crates-mutation-replay 在它文件头「分片」与 `scripts/mutation/mutation-shard-run.sh` 文件头（`mutation-shard-run.sh` 只由这道门禁在服务分片里起，单独交退 2；片数按内存定、各片之间按上一次现跑的用时均分）；formal-verification 在它文件头「分片」；broad-mutation 在门禁 `.claude/gate.d/broad-mutation.sh` 文件头「分片」与驱动 `scripts/mutation/broad-mutation.py` 文件头「分片」；崩溃放量的分片驱动是 `scripts/crash/crash-amplification-two-hosts.sh`，只经服务的分片协议起。

### 工作线程那一半归哪一格

什么时候读它：要知道门禁判得出的工作线程那一半判的是什么时。

门禁 `crash-point-replay` 判工作线程的那一条在 `.claude/rules/verification-reference.md`「门禁管哪一半」一节 `crash-point-replay` 那一行；没照登记值取线程数的登记在 `.claude/heavy-thread-exempt`，由门禁 code-tooling 的 heavy-thread-defaults 格判。
