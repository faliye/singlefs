# 产品 GPU 包的测试与 E202 装置归位（第一轮）

方案按 `crates/` 今天的实现来谈。标「量的」的是主 agent 跑命令得到的，标「读码」的是主 agent 读源码得到的，标「推的」的没量过。调度记录在 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。

## 问题

`crates/singlefs-background-gpu-tests` 是为产品的后台 GPU 包（`crates/singlefs-background-gpu`）另开的测试包，里面还住着实验装置 E202（GPU与CPU两种模式GC对照组）的二进制。用户要求照标准拆干净：测产品 GPU 包的测试住进 `crates/singlefs-background-gpu/tests/`，E202 的装置住进装置的位置，这个测试包整个撤掉。

要推的是怎么拆才干净，不是要不要拆。四条岔路（F1–F4）各给出能落地的做法与代价，交用户定。

## 已定的前提（用户原话，不在这一轮里重议）

1. 「按照标准的要求严格的来。 测试就该在测试的地方   装置就该在装置的地方。 干净的拆开解决问题。」（这一轮开轮那天，调度记录「课题与出口」一节）
2. 产品 GPU 包与验证包互不依赖：「1 可以新开包 互不依赖 没有问题」，「互不依赖 代码可以不同，但是输出的键值要一样。」（`records/2026-11-11-GPU两段与按需领卡核查.md:134`、`:141`）
3. 录制入库只有一套：「我们在录制入库部分的逻辑应该是一致的。 不要做第二套或者第三套东西。 你要检查其他的地方是否有这种问题。」（同文件第 125 行起的「用户定案（键与录制入库）」）

## 压着这件事的条款

- 门禁 checker-independence-and-sync 的 ⑧（`.claude/gate.d/checker-independence-and-sync.sh:38`）：「验证包 singlefs-verification 与产品 GPU 包 singlefs-background-gpu：两边的传递闭包（dev-dependencies 也算）里都没有对方」；它的理由在同文件第 50 行：「⑧ 同一个理由落在 GPU 上：验证包的 GPU 判器与产品 GPU 包共用一段代码时，那段写错会让判器与被判的产品路径一起错，GPU 对拍照样全绿。」；它认不出的在第 45 行（`#[path = …]`、`include!`）。
- D13（验证路线） 已定项 15（判器整套上 GPU）、已定项 17（单元键由什么组成）；D26（后台整理与放置回收） 已定项 13（后台任务的计算放在哪、怎么开、什么节奏）。
- `.claude/rules/verification.md`「定义与名字」（验证包装真设备装置二进制 `src/bin/`）、「判定库只有一份」、「卡只归 GPU 服务，任务不拿卡」。
- `.claude/rules/implementation-first.md` 第 4 条（`scripts/experiment-apparatus/` 下的装置只用来独立验证、不与实现共用代码）。
- `.claude/rules/directory-layout.md`（`scripts/` 与 `research/` 各放什么）。
- `.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」。

## 实现今天的样子

主 agent 的观测（量的，命令都在仓根跑）：

- 测试包有 13 个测试目标、5 个共用模块（`tests/common_*/mod.rs`）、E202 的 bin（`src/bin/e202_gpu_cpu_gc_comparison.rs`，14096 行）、`build.rs`（经 `singlefs-unit-graph` 照 `src/reference_units_entries.rs`、`src/gpu_side_units_entries.rs` 两张登记表算对拍两侧的判法单元键），库 `src/lib.rs` 是空的（`wc -l`）。
- 直接 `use singlefs_verification` 的有 9 个测试目标与 E202 的 bin（逐文件 `grep -c singlefs_verification`）。用到的只有：`crash_library` 的 `PairingJudged`、`DisposableLibrary`、`TheOneLibraryKeptForThisProcess`、`RunStage`、`pairing_input_through_the_pairing_library`、`pairing_input_and_reference_through_the_pairing_library`、`pairing_conclusion_reused_or_judged`；`verdict_store::PairingAgreement`；`crash_identity::JudgingUnitsKey`；`sha256::DIGEST_BYTES`；`gpu_card_configuration::cards_handed_over_or_leased_on_demand`（`grep -rhoE` 计数）。这几份文件的行数：`crash_library.rs` 6101、`verdict_store.rs` 10904、`crash_identity.rs` 3715、`gpu_card_configuration.rs` 1959、`sha256.rs` 234。它们与验证包其余模块（GPU 判器、崩溃放量）之间怎么缠，主 agent 没读，推不出。
- 不依赖验证包的目标：`background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs`（第 11 行 `#[path]` 引验证包 `tests/common_background_gpu_compute/mod.rs`，那份只依赖核心层，验证包另有 8 个测试目标用它，例 `crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs:12`）、`real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs`、`real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs`、`mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs`（后者 `mod common_pairing_input_through_the_library`，那个模块用验证包，所以经模块依赖它）、`reference_units_entries_name_functions_that_exist.rs`。
- `tests/common_discrete_gpus_of_this_host/mod.rs` 被 10 个测试目标与 E202 的 bin 共用，第 542 行按 `CARGO_MANIFEST_DIR` 读本包测试的文件头；`tests/common_pool_on_memory_devices/mod.rs` 被 4 个共用、只依赖核心层。
- 判定库服务的 Rust 客户端在 `prune-server/src/lib.rs`，验证包经 `prune-server = { path = "../../prune-server" }` 依赖它（`crates/singlefs-verification/Cargo.toml:33`）。
- 实现驱动的装置二进制今天住验证包 `src/bin/`（`crates/singlefs-verification/Cargo.toml` 26 条 `[[bin]]`，例 `e185_gc_batch_wall_clock_split`）；`scripts/experiment-apparatus/` 下有一个经路径依赖引 `crates/` 的包 `e176_crates_oracle`（它的 `Cargo.toml` 第 3 行：「只有它以路径依赖引 crates/ 的 singlefs-core、singlefs-format（只读，拿 inode_tree.rs 的两个纯函数当 M1 的参照，不改 crates/）」）。
- 产品 GPU 包自己的 `tests/` 今天有 3 个目标，`Cargo.toml` 的 `[dev-dependencies]` 只有 noop 后端的 wgpu。
- 真卡测试取卡读 `GPU_CARDS_BY_PRIORITY` 当它是任务服务交的卡（`tests/common_discrete_gpus_of_this_host/mod.rs:42`），任务服务把它列为选卡变量、任务带了就拒（`task-server/task_server_lib.py:2029`）；`.claude/rules/verification.md` 写这批真卡对拍交 GPU 服务代跑（接口待定）。取卡怎么改不是这一轮的题，只在 F1–F4 的做法撞上它时写明。

## 判据格

每格答「成立 / 不成立 / 推不出」，并给依据（`crates/` 的文件:行，或上面的读数）。写做法的，写成对实现的改动：哪个文件搬到哪、哪个 `Cargo.toml` 加什么、哪一处引用怎么改。

1. **F1 判定库存取与算键放哪**：产品 GPU 包的测试要用上一节列的那几样而不依赖验证包。候选甲：把这几样（连同它们的闭包）从验证包拆成一个不含任何判器、不依赖 wgpu 的包，验证包与产品 GPU 包（dev 依赖）都依赖它；候选乙：产品 GPU 包测试自写一份存取与算键代码，键值与验证包逐字节对拍。逐个答：拆出来的闭包实际有多大、要不要把判器或崩溃放量一起拖出来（照 `use` 链读）；⑧ 的理由（第 50 行那一句）在甲下还成不成立、⑧ 的允许共用清单要加什么、要不要另判「新包不含判器」；乙撞不撞前提第 3 条；有没有第三条路。
2. **F2 E202 的装置放哪**：E202 量的是产品整理在 GPU 与 CPU 两种模式下的对照，要跑产品 GPU 包。候选甲：新开一个只放「要产品 GPU 包的实现驱动装置」的装置包；乙：`scripts/experiment-apparatus/` 下的一个包（照 `e176_crates_oracle` 的接法）；丙：验证包 `src/bin/`（要验证包依赖产品 GPU 包）。逐个答撞不撞 ⑧、`.claude/rules/implementation-first.md` 第 4 条、`.claude/rules/directory-layout.md` 的层数与「不放结果」；装置读判定库那一段（`crash_library` 两段）在各候选下从哪来（与 F1 联动）；E202 的 kb 页、登记表与变异行里的复跑命令要改哪些。
3. **F3 核心层特质的 CPU 替身放哪**：`common_background_gpu_compute` 只依赖核心层，验证包 8 个测试目标与产品 GPU 包这一个测试目标都用。候选甲：核心层带一个只给测试用的特性导出它；乙：并进 F1 甲那个包；丙：单开一个只依赖核心层的测试支撑包；丁：产品包与验证包各 `#[path]` 引同一份（就是今天的样子）。逐个答撞不撞 `.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」、⑧ 的盲区、核心层的依赖方向。
4. **F4 对拍两侧的单元键在哪算**：今天测试包 `build.rs` 经函数单元图照两张登记表算键，登记表里的入口路径写 `singlefs_background_gpu_tests::…`。搬进产品 GPU 包之后：甲，产品包 `build.rs` 接过这一段（产品包多一个构建依赖 `singlefs-unit-graph`，两张登记表放哪）；乙，别处（F1 甲那个包、测试运行时）。答各自对单元键组成（D13（验证路线） 已定项 17）的影响：键会不会因为包名、路径变而变（路径与名字不进键），哪几组判定要现判一次。
5. **共用模块怎么分**：`common_discrete_gpus_of_this_host`、`common_pool_on_memory_devices`、`common_pairing_input_through_the_library`、`common_history_pool_through_the_library`、`common_checksum_batches_over_the_handed_over_gpus` 在 F1–F3 定下之后各住哪；有没有一处要跨包 `#[path]` 或抄成两份（两样都不许，见前提与 `.claude/singlefs-ai-sop/rules/code-discipline.md`「注释、断言、常量、重复」）。
6. **撤包之后的引用**：全仓引包名的地方（`git grep -l "singlefs-background-gpu-tests\|singlefs_background_gpu_tests"` 量的 187 个文件 1261 处）分几类，哪几类随 `.claude/rules/path-moves.md`「怎么做」改、哪几类是门禁的常量与样本档（门禁 crash-point-replay 的 product-gpu-real-card-tests 格按包名认，`.claude/gate.d/crash-point-replay.sh:794`）、哪几类是冻结证据不改。
7. **防再发**：调度记录「没能避免的原因」五条缺口，各给一条会红的检查（判什么、拦哪一种写法、放哪道门禁哪一格），只写判据，不改门禁。

## 分工

- 正推腿：逐格给出能落地的做法，每一步写到文件与函数；第 1 格照 `use` 链实读拆出来的闭包，给出拆出来的模块清单与行数。
- 攻方腿：攻第 1、2、3、5 格——拆出来的包是不是把判器或别的会让 ⑧ 失去意义的代码一起带了出来；各候选里有没有藏着的跨包 `#[path]`、抄两份或第二套录制入库；E202 各候选撞不撞规则。再攻「实现今天的样子」里主 agent 写下的每一句「只有」「没有」：照行号去读，对不上的逐条列。
