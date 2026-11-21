# gpu-tests-placement-design-r1：判决（2026-11-19）

正文 `research/prompts/_gpu-tests-placement-design-r1-body.md`，开工快照 `research/prompts/gpu-tests-placement-design-r1-snapshot/`。这一题的第一轮，前面没有判决。调度记录 `records/2026-11-19-产品GPU包测试与E202装置归位.md`。

快照的记法：`kb-sha256.txt` 由 `three-way-materials.py gpu-tests-placement-design-r1` 那一趟写（结局 `type=succeeded exit=0`）；被判文件的清单 `judged-sha256.txt`（测试包 24 份加 `.claude/gate.d/checker-independence-and-sync.sh`，共 25 份）是两条腿交回之后才补记的，派腿之前漏了。补记时 `find … -newer research/prompts/_gpu-tests-placement-design-r1-background.md` 对这 25 份零命中，即材料生成之后（派腿之前）这些文件都没改过，补记的清单就是派腿那一刻的样子。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet） | `research/prompts/gpu-tests-placement-design-r1-sonnet-output.md`（sha256 e56ca896…） | F1 甲成立（按条目拆，判器与崩溃放量 0 行被拖出；按文件搬不成立），乙不成立；F2 甲成立；F3 丙推荐；F4 甲成立；第 5 格成立（要一个支撑包）；第 6 格成立，出口三「0 命中」推不出；第 7 格给了五条检查的判据 |
| 攻方（opus） | `research/prompts/gpu-tests-placement-design-r1-opus-output.md`（sha256 6a8a0444…） | F1 甲打中（照今天的代码拆不出不含判器的包），乙打中（就是 11-11 撤过的形状）；F2 三个候选各撞一条；F3 判据本身可能写错；第 5 格 E202 的两个共用模块不管怎么选都得跨包；「实现今天的样子」6 处对不上 |

核查命令（原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py gpu-tests-placement-design-r1 --out /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-verdict/gpu-tests-placement-design-r1-tree --repository /home/fy5090/code/singlefs`：结局 `type=succeeded exit=0`，「清单点名 3 份，倒推出 3 份，sha256 对上 3 份」。那一趟跑在补记 `judged-sha256.txt` 之前，树里只有 kb 三份，取的都是工作区此刻那一份。
- `python3 scripts/agents/cite-check.py research/prompts/gpu-tests-placement-design-r1-opus-output.md research/prompts/gpu-tests-placement-design-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-verdict/gpu-tests-placement-design-r1-tree --background research/prompts/_gpu-tests-placement-design-r1-background.md`：「核了 73 处引文、101 处裸行号，对上 1 处，没判 173 处」。没判的那些引的是 `crates/` 与门禁文件，倒推树里只有 kb。被引的 25 份自材料生成之后都没改过，主 agent 在此刻的树上逐处现核了判决要用的几处：
  - `crates/singlefs-verification/src/crash_library.rs:948` 开库结构体的字段类型是 `crate::crash_amplification::RecordedFlowAndItsPlan`（攻方第 1 格），坐实。
  - `crates/singlefs-verification/src/verdict_store.rs:1372` 引 `singlefs_checker::image::IMPLEMENTED_INVARIANTS`，坐实。
  - 直接 `use singlefs_verification` 的测试目标是 8 个（`grep -l singlefs_verification tests/*.rs | wc -l`），正文写 9 个是错的。
  - `common_discrete_gpus_of_this_host` 被 11 个测试目标 `mod` 进来，正文写 10 个是错的。
  - 那个模块第 542 行读的是 `scripts/crash/crash-point-replay-shard-configuration-check.sh` 的路径，正文写「读本包测试的文件头」是错的。
  - `scripts/experiment-apparatus/Cargo.toml:254` 经路径依赖引 `crates/singlefs-device-library`，正文写「有一个经路径依赖引 crates/ 的包」不全。
  - 两处同名函数：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:206-209` 与 `crates/singlefs-verification/src/gpu_memory_ledger.rs:780-783` 的 `pci_bus_identifier_without_the_domain`，函数体三行相同、形参名不同（`pci_bus_address` 与 `text`），坐实；另有第三份在 `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:488`（第二轮攻方腿报，主 agent 现核）。
  - `crates/singlefs-unit-graph/src/build_toolchain.rs:544` 读 `"/../singlefs-background-gpu-tests/build.rs"`（正推第 6 格），坐实。
  - 门禁样本档 `.claude/gate.d/fixtures/crash-point-replay.sh/` 下 `product-gpu` 起头的目录 11 个，坐实。门禁常量 `PRODUCT_GPU_PACKAGE_NAME` 此刻在 `.claude/gate.d/crash-point-replay.sh:781`，正推写的 `:794` 行号对不上，常量本身在。
- 正推的闭包数（23/43 个模块、92356 行；26 个模块、100868 行；3 组判器键会变，现判在 10⁹ 量级）与攻方的模块级闭包都是腿自己跑的，主 agent 没有复跑。用户的 F1 定案不选拆包，这几个数不进任何结论。

## 二、判决

四条岔路由用户定案（原话在调度记录「用户定案（四条岔路，原话）」与「用户定案（弹窗）」两条），这一轮不再为它们开第二、第三轮；腿的结论按定案收：

1. **F1 判定库存取与算键：各写各的，不公用。** 产品 GPU 包的测试自己写存取代码，不依赖验证包。两条腿对甲的分歧（正推「按条目拆成立」、攻方「照今天的代码拆不出」）都落在甲上，用户没选甲，不用分辨。攻方说乙「就是 11-11 撤过的形状」不成立：`records/2026-11-11-GPU两段与按需领卡核查.md:128` 撤的理由是两处特例键（「键取「测试可执行文件的摘要加用例名」、512 MiB 那一条只录一段共用字节，两样都是特例」），不是因为代码各写一份。约束：产品一侧的录入键照同一套算法（录入函数单元指纹加录入输入摘要，`crash_library.rs` 的 `recording_key_of_the_inputs`），不许特例；同一份输入两侧写进库的字节怎么证逐字节相同，进第二轮。正推说乙「撞前提 3」，照定案作废：前提 3 在跨包这一处让位。
2. **F2 装置：新开实验包**，在 `crates/` 下，依赖产品 GPU 包与验证包都正常。E202 与验证包 `src/bin/` 下的装置一起搬进去（用户弹窗定）。正推第 2 格「甲成立」与攻方「甲等于换个名字的测试包」的分歧，按装的是什么判：实验包只装装置二进制，不装产品包的测试，所以不是测试包，攻方这一条不采；攻方报的 `verification.md`「定义与名字」冲突照定案改那一行。
3. **F3 CPU 替身：产品 GPU 包在自己的 `tests/` 下另建一份**，照验证包 `tests/common_background_gpu_compute/mod.rs` 的做法，不跨包引。正推推荐的替身包与 `#[path]` 共用都不取。攻方说「运行时与 checker 不同算法」拿来判替身是外推，这一条采：判据格第 3 格那句判据写错了，第二轮不再拿它判替身。
4. **F4 单元键：产品 GPU 包的 `build.rs` 经 `singlefs-unit-graph` 算**（正推甲）。依据是用户原话「singlefs-unit-graph应该是共用的」。⑧ 的允许共用清单加 `singlefs-unit-graph` 与 `prune-server`（用户弹窗定「两个都加」），⑧ 另判这两个不依赖 wgpu、不含判器。
5. **第 5 格 共用模块**：正推的支撑包是在 F1 甲、F3 丙的前提下推的，定案之后前提不在了。攻方说 E202 的两个共用模块（取卡、对拍）不管怎么选都得跨包，这一条在新定案下仍然成立：实验包里的 E202 照 F1 定案可以直接用验证包的 `crash_library` 与 `gpu_card_configuration`，取卡与对拍那一层从哪来、产品测试那一份与实验包那一份是不是各写各的，进第二轮。
6. **第 6 格 引用**：正推的分类采用（此刻 191 个文件 1274 处）。出口三的「再搜 0 命中」与 `.claude/rules/path-moves.md`「怎么做」第 5 步冲突，正推说得对：出口三改成「除第 5 步留旧路径并登进 `.claude/gate.d/dead-path-kept.tsv` 的之外 0 命中」。`scripts/mutation/crates-mutation-rows.py:391` 的文档串进判法单元指纹，改它要与别的换键改动放在同一窗口（正推第 6 格），采。
7. **第 7 格 防再发**：正推第 7 节五条检查的判据采用，角色改成产品、验证、测试支撑、实验装置四种。正推缺口 1 第 ④ 条「测试支撑包依赖者不到 2 个判红」与 F3 定案（各写各的）相容，保留。五条都没跑过样本，由工具员各配会红的样本与弄坏开关。
8. **正文 6 处事实错**：照「一、腿与核查」现核的更正，第二轮正文改用现值。

## 三、改法

- 第二轮正文要判的（零轮，没有一条被攻过）：两侧录入键的逐字节对法；实验包的名字、依赖与 `Cargo.toml` 接法；26 个装置里 5 个不带实验号的二进制（`crash_library_reader`、`deletion_write_dump`、`new_pool_file_creation_device_log_check`、`new_pool_file_creation_on_device`、`new_pool_file_creation_region_bytes`）哪份留验证包、哪份进实验包（门禁 qemu-device-streams、`scripts/crash/crash-library-totals.sh`、`crash-amplification.sh`、`crash-point-replay.sh` 点名它们）；6 份经 `#[path]` 引验证包 `tests/common*` 的装置搬过去之后共用模块从哪来；E202 的取卡与对拍那一层从哪来；`pci_bus_identifier_without_the_domain` 两份怎么办；`tests-on-a-real-gpu` 特性在产品包里怎么挂。
- 措施 M1–M5 交工具员，判据照正推第 7 节加上「二、判决」第 7 条的改动；派在第二轮判决之后，角色与位置标准以那一轮为准。
- 这一轮没有要写进产品代码的零轮改法，不做 A/B。

## 交用户的

四条岔路与两处弹窗用户已在这一轮定了（调度记录）。交用户的是第二轮判完之后的那几条。

## 这一轮被判过的文件

- .claude/gate.d/checker-independence-and-sync.sh：判了 ⑧ 的允许共用清单（第 39–40 行）加 `singlefs-unit-graph` 与 `prune-server`，第 45 行写明的 `#[path]`、`include!` 盲区由新的 ⑨ 判
- crates/singlefs-background-gpu-tests/build.rs：判了它算对拍两侧单元键的那一段搬进产品 GPU 包的 `build.rs`
- crates/singlefs-background-gpu-tests/Cargo.toml：判了它依赖验证包的那一行在拆包后由产品包自己的存取代码顶替，第 32 行引的规则节名已不在 `verification.md` 里
- crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs：判了它搬进新开的实验包
- crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs：判了它随单元键那一段进产品 GPU 包，入口路径头一段改名
- crates/singlefs-background-gpu-tests/src/lib.rs：判了它随撤包删掉（库是空的）
- crates/singlefs-background-gpu-tests/src/reference_units_entries.rs：判了它随单元键那一段进产品 GPU 包
- crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs：判了它进产品 GPU 包 `tests/`，第 11 行跨包 `#[path]` 换成产品包自己的 CPU 替身
- crates/singlefs-background-gpu-tests/tests/checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs：判了它随三个校验和目标进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs：判了它被 11 个目标与 E202 共用、第 542 行读的是多机配置判法脚本、第 206 行起的函数与验证包 `gpu_memory_ledger.rs:780` 逐字相同；E202 那一份从哪来进第二轮
- crates/singlefs-background-gpu-tests/tests/common_history_pool_through_the_library/mod.rs：判了它随三个用它的目标进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs：判了它是产品一侧自写存取代码要顶替的那一份，录入键照同一套算法、不许特例；E202 那一份从哪来进第二轮
- crates/singlefs-background-gpu-tests/tests/common_pool_on_memory_devices/mod.rs：判了它只依赖核心层，随四个目标进产品 GPU 包 `tests/`
- crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs：判了它测的是产品开卡器接在运行库上，进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs：判了它经两个共用模块依赖验证包（攻方坐实），进产品 GPU 包 `tests/`后改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs：判了它测的是取卡模块，随那个模块的去处走
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs：判了它测的是取卡模块，随那个模块的去处走
- crates/singlefs-background-gpu-tests/tests/reference_units_entries_name_functions_that_exist.rs：判了它随单元键那一段进产品 GPU 包；第 173 行 `include_str!` 读 E202 源码，E202 搬走之后成了跨包引，进第二轮
- crates/singlefs-background-gpu-tests/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：判了它进产品 GPU 包 `tests/`，判定库那一段改用产品包自己的存取代码

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-19 不受影响：单元键的组成不变，包名与路径不进键；改的是算键的代码住哪个包 |
| D26（后台整理与放置回收） 已定项 13 | 不影响 | 2026-11-19 不受影响：产品 GPU 路径的计算放在哪不变，搬的是它的测试与装置 |
