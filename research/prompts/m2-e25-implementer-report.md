# 实二五 交回：空间准入接线、抬 F 退回、核出对不上先重读、回退见证删与补、真设备读回 F、三处红、空节点坏镜像

实现员（implementation-writer），2026-09-25（时刻均为 UTC；JST = UTC + 9）。规格 `/tmp/claude-1000/impl-m2-e25/spec.md`（sha256 c35bde98…7a26，开工时核过）。
改动直接写在主工作区 `crates/` 里，没有另开补丁，所以没有 `git apply --check` 那一步可做：主工作区现状就是结果。
开工时 `crates/` 没有别人在改；收工时 `diff -rq` 对开工那一刻的副本差出 23 个文件：22 个是这一轮自己改的（清单见第二节），另一个 `e156_allocation_basis_counts.rs` 是别的会话在 00:32Z 之后改的，我没碰。

## 一、八件各到哪

| # | 件 | 结局 | 用例（新加或改过的） |
|---|---|---|---|
| 1 | 空间准入接进发布与可写挂载 | 做了，另加只供测试的开关 `SpaceAdmission`（见第六节 Q1、Q2） | `second_transaction_supplement_two_admission_formula.rs` 两条新用例；随机历史一条新取样；四条小盘旧用例改走开关 |
| 2 | 抬 F 第一次空发布被拒时退回分配器 | 做了 | `second_transaction_supplement_two_commit_generated_fallback.rs` 那条改写并改名 |
| 3 | 核出对不上先重读一次 | 做了；故障注入 `fast_tier`、`one_fixed_history` 转绿 | release_checksum 两条新用例；fault_injection 快档注释登记「每一读都坏」那一形 |
| 4 | 见证表被罩住的删 | 做了；A1 不再写满、A2 仍第 24 次写满 | witness 文件两条新用例 |
| 5 | 可写挂载按回退行补见证 | 做了；m = 1..3 落 R_old、m = 0 钉现状 | witness 文件两条新用例（N 的读法与规格字面不同，第六节 Q4） |
| 6 | 真设备二进制打盘上读回的 F | 做了：新行 `name=recover_cold_rollback_floor chosen_root=I:T rollback_floor_on_disk=F`，只在 raise-rollback-floor 模式 | bin 里一条宿主用例 |
| 7 | 实二二三留下的三处红 | 三处都转绿（原因见第四节） | formatted_pool 一条改注入点；checker_known_bad_images 补 I-7.10 / I-7.11 两份坏镜像；step_four 两条改写并改名 |
| 8 | checker 判「根之下没有空节点」 | **没改 walk.rs**：开工那一刻的 checker 已经判（只红 I-1.1），规格的验收前提「今天不红」不成立；加了一份账补齐的坏镜像钉住它（第六节 Q6） | checker_known_bad_images 一条新用例 |

关键落点（行号是现在的文件）：
- `crates/singlefs-core/src/admission.rs`：`SpaceAdmission` 460、`checkpoint_cost_of_the_version_to_build_on` 487、`SpaceBudgetOfARole` 513、`space_budget_of_role` 527、`demand_of_the_roles_on_each_device` 556、`admission_reading_before_a_publish` 581。
- `crates/singlefs-core/src/transaction.rs`：`PublishError::SpaceAdmissionRefused` 2892；发布路径的准入在 `prepare_the_version_publish` 里、算定形状之后、读盘核与动分配器之前（`if demand_is_judged {` 4570）；重读一次在 `copies_failing_the_release_checksum_check` 2700 起，`CopyCheck` 2775。
- `crates/singlefs-core/src/mount.rs`：`MountError::SpaceAdmissionRefusedBeforeAcquisition` 115；挂载准入在 `establish_instance` 取号与预演之前（`match start.space_admission {` 1712）；抬 F 快照 979、失败时换回 1080；`rollback_witness_entries_not_covered_by_another` 1560、`rollback_witness_entries_recovered_from_the_instance_table` 1588、`rollback_witness_tables_of_this_mount` 1625；`mount_writable_with_space_admission` 1967、`mount_rollback_with_space_admission` 2085（`mount_writable` / `mount_rollback` 签名不变，委托过去、恒判准入）。
- `crates/singlefs-core/src/allocator.rs`：挂载期常量 rows0 768、开关 771（`new` 起步 0 与「判」）。
- `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`：`rollback_floor_read_back_from_the_chosen_root_line` 1066；宿主用例 2644。

## 二、这一轮写过的文件（自己列；与开工那一刻的副本 `diff -rq` 逐个对过，22 个；差出来的第 23 个 e156 不是我的）

- core：`crates/singlefs-core/src/admission.rs`、`allocator.rs`、`mount.rs`、`transaction.rs`
- harness src：`crates/singlefs-harness/src/history.rs`、`model_comparison.rs`、`fault_injection.rs`、`bin/first_transaction_on_device.rs`
- harness tests：`tests/common/mod.rs`（`replace_the_witness_on_one_device` 从见证用例文件挪进来，两个二进制共用）、`checker_known_bad_images.rs`、
  `second_transaction_step_four_rollback.rs`、`second_transaction_step_three_formatted_pool.rs`、`second_transaction_supplement_two_rollback_witness.rs`、
  `second_transaction_supplement_two_release_checksum_quarantine.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`、
  `second_transaction_supplement_two_admission_formula.rs`、`second_transaction_supplement_two_root_ring_turn_in_one_mount.rs`、
  `second_transaction_supplement_three_random_history.rs`、`second_transaction_supplement_three_fault_injection.rs`、
  `second_transaction_supplement_three_crash_injection.rs`、`second_transaction_supplement_three_bad_disk_input.rs`
- `crates/mutations.tsv`（见第三节）
- 没碰：`e158_root_choice_repair.rs`、`first_transaction_regions.rs`、`tests/first_transaction_region_bytes.rs`、`crates/singlefs-checker/`（walk.rs 试改过、又改回，与开工时逐字节相同）、`litmus/`、名字含 layer0 的文件。

手工编辑之外的写法（照实列）：`history.rs`、`second_transaction_supplement_three_{crash_injection,bad_disk_input,fault_injection}.rs` 四个文件里给
`HistoryExecution` 加字段那几处是用 Bash 里的 python 做的逐处恰好一次替换；见证用例文件末尾那四条 Z9 用例是 `cat >>` 追加的；
格式化用 `rustfmt --edition 2021` 逐个跑在自己改过的文件上（没跑 `cargo fmt --all`，免得动 e158）。这几处本该走 Edit，内容与 Edit 等价，列在这里。
`crates/mutations.tsv` 的改动都经 `research/scripts/replace-once.py`（整行恰好一次替换），脚本在草稿目录 `scripts/`。

## 四、第 7 件：三处红各自为什么红、怎么转绿

1. `second_transaction_step_three_formatted_pool` 的 `transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write`：
   注入点是「每块盘第 N 次读系统配置槽 0」。回退见证进来之后，可写挂载在判定之前每块盘读槽 0 的次数从 1 次变成 6 次
   （`mount_writable` 择系统配置、择根读见证；`replay_journal` 择系统配置、读见证；`rebuilt_allocator` 择根读见证、影子账再读一次见证），
   原来的「第 2 次」落到了择根读见证那一读上，判定那一读没被打到（报 `None`、挂载做成）。读法不改，注入点改成判定那一读：第 7 次
   （常量 `DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT`，注释写了每一读是谁的）。数法：草稿副本里给读槽 0 的地方各打一行、
   照一次挂载的次序数（副本上 4、5、6 都不成，7 成）。行 66 的变异（取号不核判定的号）复验见第五节。
2. `checker_known_bad_images` 的 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`：checker 的
   `IMPLEMENTED_INVARIANTS` 多了 I-7.10 / I-7.11，坏镜像语料没跟上（`targets` 与 `listed` 两个集合差这两条）。补了一组
   `known_bad_images_of_the_rollback_witness`：I-7.10 那份把盘 0 两槽的见证条数改成 24（上限 23）并重封整槽校验和；I-7.11 那份在没回退过的池上
   往四个槽里塞一条 (2, 1, 3)。另加 `each_rollback_witness_bad_image_reddens_its_own_invariant` 钉各自只红的那几条（I-7.10 只红它；I-7.11 那份外加
   I-3.1——塞进去的条目把 B 判成被抛弃，与见证用例文件里同一个结论）。
3. `second_transaction_step_four_rollback` 两条（影子账判别力那两条）：它们让实例 3 的四条根读不出（一条用改字节、一条用读失败），期望恢复退到
   被抛弃的 C (2, 8) 再看 C 的数据单元被没被盖。回退见证（C332 的修法）之后恢复不再择 C：见证 (3, 1, 3) 抛弃 B、C，落到 R_old (1, 3)、读回 A——
   红的就是这一格。改法：两条都先断言恢复落 R_old 读回 A（见证起作用），影子账那一半改成沿 C 的根直接走读到文件（`recovery::walk_to_file`，
   恢复择到一条根之后走的同一段）：影子账关着读不回第三次的内容、开着原样读回。C 仍是影子账要护的对象——回退挂载崩在写行的根落了、轮换还没落的那一格，
   恢复择的就是它（Z9-B 那条用例 m = 0 那一格钉着）。两条改了名（名字里原来写「恢复退到被抛弃的根」，现在不成立了），`crates/mutations.tsv`
   第 30、228、251 行第 6 列跟着换成新名（名字前缀没变，第 5 列的过滤词照旧选得中）。

## 五、证红

做法：草稿目录一份仓副本（`/tmp/claude-1000/impl-m2-e25/mutants/`，自己的 target），逐条施加变异、跑那条变异点名的测试所在的**整个**测试二进制、
记下红了哪些、把原件拷回并 `touch`（`scripts/prove-red.sh`，日志 `logs/prove/`）。副本在 run4 之后从主工作区拷出，这一轮我改过的文件与主工作区逐字节相同（证完每一行都拷回原件、`touch`，收尾 `diff -rq` 只差别的会话后来改的 e156）；不改动时的红集取主工作区
这一轮最后一遍整批跑（`logs/run4/summary.txt`）：只有开工时就红、不归我的那几条（第七节），与下表点名的测试无一重合。
被测代码里没有 debug_assert 挡在前面（红的都是测试自己的断言，日志里逐条看过 panic 位置），没另跑 `--release`。

## 六、停下交主 agent 的设计问题与要知道的事

**Q1 需求怎么算（第 1 件，条款没写，实现员取的读法）。** D28 已定项 1 只写「可用(d) ≥ 需求(d)」，C370 只收窄到「按盘字节」。我取：
- 一次发布的需求 = 这次重写的角色里**普通分配**那几个的槽数（数据单元、extent 树节点、inode 树叶容器与根；`admission::space_budget_of_role`）；
  固定点（分配记录树、中央映射树、记账树、树表——ckpt_cost 的 Σ 名单）从式子第八项 checkpoint 保留池里出，实例表链从第五项的实例切换预留里出，
  都不再算需求（依据：D23 已定项 24「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」；D28 已定项 4「实例表链的开销归已定项 3 的切换预留」）。
- 一个普通分配都没有的发布（空发布、写行、暖机、抬 F 那一串）**不判**。判它们会让推空发布抬 F 被式子挡住（D3 已定项 17「释放空间这个操作本身不需要申请空间」），
  也会让挂载取号之后的暖机被挡（取号之后才拒 = C378 那一类）。
- 可写挂载的需求逐盘 0：判的是「扣完九项之后可用(d) ≥ 0」，即实例切换的预留拿得到（D2 已定项 13）。
- 读数取分配器此刻的计数（挂载时是回收与影子账隔离之后、写行之前；`AdmissionReading::of_allocator` 的文档早就写了「那两段里准入读哪一个，条款没写」）。
- ckpt_cost 对树表 0 条的一版：中央映射树 0、记账树 0；分配记录树只在那一版写过行时有，取几何的高，否则 0；树表恒 +1。c_max 取同一个数（已定项 4 末条）。
- rows0 = 挂载时读到的行数 + 写行那次要写的行数，记在分配器上当挂载期间常量；mkfs 同一个进程里 0。
要推翻它：主 agent 定「固定点也算需求」或「空发布也判」，改 `space_budget_of_role` / `prepare_the_version_publish` 那一判即可，第 3 行变异就是前一种。

**Q2 只供测试的开关 `SpaceAdmission`（第 1 件带出来的，新 API）。** 式子接上之后，小盘（240 / 256 / 384 槽）上墙由式子先拒，「准入放行而落点取不到、
在任何写之前拒绝」那条兜底（D3 已定项 5、C545）在健康的历史里走不到了——四条旧用例（单元区墙取样、挂载预演取不到落点、回退到环里最旧的根、C517 同一次挂载）
与它们挂着的第 153、434、497–500 行变异会因此失去对象。按 fs-design 五条硬要求第 2 条加了开关：`PoolAllocator::set_space_admission`、
`mount::mount_writable_with_space_admission` / `mount_rollback_with_space_admission`（旧入口签名不变、恒判）、`HistoryExecution::space_admission`。
那四条用例装「关」照旧测兜底，另加一条「判着」的单元区墙取样。开关要不要留、留在哪一层，交主 agent。
**小盘上的实际影响**（量过）：384 槽的盘上可写挂载之后第 11 次覆盖写就被式子拒，之后每一次都拒（根环 24 条根的 defer 装不下：式子扣切换预留
4 × (2 + 3 × c_max) ≈ 68 槽、保留池 ≈ 5 槽，defer 按读法甲扣两次）；256 槽的盘上第 5 次。4 GiB 的盘上一次都不拒（层 0、快档、各步用例都没变）。
D16 已定项 1「准入不够时先推空发布抬 F 再判」（C283）没实现，被拒之后用户只能自己推空发布。

**Q3 抬 F 第二次起被拒（第 2 件射程外）。** 只做了「第一次空发布在任何写之前被拒 ⇒ 分配器整个换回抬 F 之前那一份」（第一次在落盘途中失败的不换：
冻结着的那次要原样重发，它的字节按回收之后的账装）。第二次起被拒时前面几次已落盘、带着新 F 与回收之后的账，扣住位照旧留到 F 生效——
这时扣住的槽计数上是空闲、实际发不出去，D28 的式子照样把它们算成可用（C546 的病根在这一格还在）。那一格怎么办条款没写。

**Q4 补见证的 N（第 5 件，与规格字面不同）。** 规格「N 取实例表里回退行之后第一个有行的实例」。回退那一次挂载写的行是 [r_old, N)：回退行与中间实例的 (i, 0, 0)；
回退跨过实例时（固定脚本里实例 2 的世界回退到 (1, 3)，新实例 3 写 (1, 3, 0, 回退) 与 (2, 0, 0)）照字面取到中间实例 2，补出的 (2, 1, 3) 罩不住实例 2 的根——
落回它正是这一条要挡的；实例 2 若自己也做过回退，(2, …) 还会与盘上那一条说两种目标（I-7.10 红）。我取「回退行之后第一条 T ≠ 0 的行的实例；没有就是所选根自己的实例」
（N 自己那一行由下一次挂载写，T 是它择到的根的 txg，恒 > 0；中间实例恒 T = 0）。两种读法在没有中间实例时相同（攻方 z9_b 那段历史）。
用例 `the_witness_restored_after_a_rollback_across_instances_names_the_instance_that_rolled_back_not_the_intermediate_one` 分得开两种读法（第 15 行变异就是字面那一读）。
另：回退到 mkfs 的第 0 代根 (0, 0) 不写行（实例 0 不写行），那一次回退崩在同一个窗口时没有回退行可补，这一格补不到。

**Q5 核出对不上的重读（第 3 件）。** 两次结果不同类时（第一次读不出、重读读出来对不上，或反过来）按「两次都没对上」处置，报出来的样子取重读那一次的
（`QuarantinedCopyReading` 两个成员的文档改了措辞）。条款只写「两次都读不出或都对不上」，混合那一格是我取的。
「每一读都给坏字节」那一形：两次都对不上，照规则隔离一对好槽——登记在 fault_injection 快档那条用例的注释里，样子钉在 release_checksum 用例 13；
随机注入一次只坏一次调用，摆不出这一形。

**Q6 第 8 件的前提不成立。** 开工那一刻的 checker 已经判按位置寻址的树根之下的空节点：`crates/singlefs-checker/src/walk.rs` 504 行
`(KeyRangeReading::PrescribedByThePositionJudgedByTheCaller, _)` 那一臂调 `check_internal_node_separators`，`crates/singlefs-checker/src/lib.rs` 530 行
`if view.entries.is_empty()` 报 `KeyOutsideDeclaredRange`，记在 I-1.1 上。草稿副本里在开工时的代码上造同一份坏镜像（账、校验和、引用链补齐），
`check_pool_image` 只红 I-1.1：`分配记录树（树 13）层级 0 盘 0 第 62 格的节点：key 宽 / key 区间与条目不符（Err(KeyOutsideDeclaredRange)）`。
所以没改 walk.rs（试加过专门的一判，拿开工时的代码一比就撤了）；坏镜像入库当回归用例，另加第 18 行变异（拿掉那一判它必须红）。
红的说明文字是「key 区间与条目不符」，不是「空节点」——要不要改成专门的一句，交主 agent。

## 五（表）、证红结果（接第五节的做法；节的次序按写进文件的先后，内容以节名为准）

新追加的 18 行（`/tmp/claude-1000/impl-m2-e25/mutations-append.tsv`，已追加到 `crates/mutations.tsv` 末尾），逐行都在副本里证过（一行一个二进制整跑），不留给 59 号：

| # | 改坏哪一处（变异名开头） | 红在哪条断言（文件:行 与消息开头） | 同一二进制里另外红的（不含不改动时就红的） |
|---|---|---|---|
| 1 | transaction.rs `if demand_is_judged {` → `if false && …`（发布不判准入） | admission_formula.rs:559 「式子判拒：None」 | 无 |
| 2 | mount.rs `match start.space_admission {` → 恒走关掉那一臂（挂载不判准入） | admission_formula.rs:482 「式子判拒、在取号之前返回：None」 | 无 |
| 3 | admission.rs 需求过滤 `== Demand` → `!= InstanceSwitchReserve`（固定点也算需求） | admission_formula.rs:570 「两块盘都短一槽：可用 5 槽 < 需求 6 槽」 | 无 |
| 4 | admission.rs ckpt_cost 漏掉树表那一项 | admission_formula.rs:482（挂载那条：式子放行了） | 发布那一条 |
| 5 | admission.rs 读数里挂载期承诺量写成 0 | admission_formula.rs:482 | 发布那一条 |
| 6 | model_comparison.rs `SpaceAdmissionRefused(_)` 映射成 `Unexplained` | random_history.rs:449 「新发现……」（准入判着的单元区墙取样） | 无 |
| 7 | transaction.rs 开关关掉那一臂照样判需求 | admission_formula.rs:604 「同一次挂载里关掉准入，同一次覆盖写做成……」 | 无 |
| 8 | mount.rs `*allocator = allocator_before_the_raise.clone();` 删掉（C546） | fallback.rs:411 「回收的槽回到 defer……」left [55, 55] | 无 |
| 9 | transaction.rs 读得出而对不上不重读 | release_checksum.rs:631 「重读对得上的那一份不隔离……」 | 用例 13（重读次数变了） |
| 10 | 同上一处，点名故障注入快档 | fault_injection.rs:119 「注入之后「已知红」清单外的失败」 | `one_fixed_history`（也回红） |
| 11 | transaction.rs 重读那一次仍对不上时当成对得上 | release_checksum.rs:674 「盘 0 那一份两次都对不上……」 | 同文件 7 条持续改坏的用例 |
| 12 | mount.rs 罩住判定恒假（被罩住的不删） | witness.rs:852 「第 3 次回退写出的见证表 3 条……至多留两条」 | 无 |
| 13 | mount.rs 罩住判定不看新实例代号 | witness.rs:879 「第 2 次回退做成、写出 2 条」（实际 1 条） | 写满那条用例（它造的 23 条被第一条罩住、删得只剩一条） |
| 14 | mount.rs 回退行过滤恒假（不补见证） | witness.rs:982 「m = 1：可写挂载按回退行补回了 (2, 1, 3)」left [] | 跨实例那条 |
| 15 | mount.rs 补见证的 N 按字面（中间实例的 (i, 0, 0) 也算） | witness.rs:1044 「补回来的是做那次回退的实例 3，不是中间实例 2」 | 无 |
| 16 | bin 打出的 F 写成 0 | first_transaction_on_device.rs:2704 「所选根从盘上读回的 F 是抬到的上限 3」 | 无（另外 4 条是不改动时就红的） |
| 17 | walk.rs I-7.11 那一判恒真 | checker_known_bad_images.rs:2341 「I-7.11 那份坏镜像红的不变量」left ["I-3.1"] | 无 |
| 18 | checker lib.rs `if view.entries.is_empty()` → `if false`（空节点不判） | checker_known_bad_images.rs:2510 「只红 I-1.1……」left [] | 无 |

表里的行号是断言所在行（日志 `logs/prove/row-N.log` 的 panic 位置原样）。第 10 行那一格另红的两条 `a_write_error_after_the_acquisition…`、`a_swallowed_write…`
不改动时就红（第七节），不算。

这一轮改了锚点或改了点名测试名的既有行，也在同一份副本里逐行复验（同一脚本，第 19–28 个）：

| 表里的行 | 改了什么 | 复验 |
|---|---|---|
| 30、251（影子账两条） | 第 6 列换新名 | 红：step_four_rollback.rs:923 / :908 |
| 228（点名一组槽只留最后一个） | 第 6 列换新名 | 红：step_four_rollback.rs:1031 |
| 88、486（抬 F 回落） | 第 6 列换新名 | 红：fallback.rs:383（两行都是） |
| 66（取号不核判定的号） | 没改，注入点改了 | 红：`crates/singlefs-core/src/mount.rs:1809` 的 `assert_eq!`（判定与取号的号对不上），与改之前同一处 |
| 409（重读还读不出当成核得上） | 锚点换到 `CopyCheck::Unreadable => {…}` | 红：release_checksum.rs:588 |
| 495（抬 F 第二次被拒报的已落盘份数 0） | 锚点换缩进 | 红：fallback.rs:483 |
| 547（读不出不重读） | 锚点换到重读那一句 | 红：release_checksum.rs:550 |
| 548（重读两次） | 锚点换到重读那一句 | 红：release_checksum.rs:579 left [3, 3] |
| 307（取号之后报错时回卷实例代号） | 锚点多两个字段 | **没法证**：点名的 `a_write_error_after_the_acquisition…` 不改动时就红（第七节） |
| 494、496（抬 F 失败账） | 494 锚点换缩进；496 被我的替换脚本误改之后照开工副本原样换回 | **没法证**：点名的 bin 用例 `failed_raise_of_the_rollback_floor…` 不改动时就红 |

## 三、`crates/mutations.tsv` 改了哪些行

- **追加 18 行**（表末，名字都以「实二五」开头；逐行证过，见第五节表一）。
- **改锚点 6 行**（这一轮的改动让原文在源码里命中 0 次，门禁 33 号点名；照今天源码里的原文改、变异的意思不变）：
  「C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）」、
  「C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）」、
  「增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账……」、
  「增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串第二次空发布被落点拒绝时报出的已落盘份数成了 0……」、
  「C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置……」、「C394 N1：「先重读一次」写成重读两次……」。
  改锚点的脚本按名字前缀多匹配了一行「……抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）」、把它的原文 / 替换文改错了；
  已照开工那一刻的副本原样换回（`scripts/restore-row-496.py`），与开工时逐字节相同。
- **改第 6 列（点名的测试名）5 行**：第 30、228、251 行（step_four 两条改名）、第 88、486 行（fallback 那条改名）。第 5 列的过滤词是名字前缀，没变。
- 门禁 33 号收尾时原样末行（exit 0）：
  `  ✓ 147 个实验二进制都有成形的变异表，1614 条变异的原文各命中源码一次；crates/mutations.tsv 679 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）`
  （主 agent 在我干到一半时点名 33 号红，那时它报的正是改锚点那 6 行。）

## 七、交回前的验证（末尾原样输出）

跑前看负载：只有 E158 执行员的 `research/scripts/replay.sh E158`（capped 6）与主 agent 的看门狗，没有 qemu / vm-bench / fio / e152；都加了 `nice -n 19`、线程上限 16。

1. 整批测试二进制（名字含 layer0 的与 herd7 那一个不跑；`scripts/run-binaries.sh`，每个二进制整跑）：开工时一遍（`logs/baseline/summary.txt`）、收尾一遍（`logs/run4/summary.txt`）。
   run4：60 个里 57 个 exit=0；不是 0 的三个与开工时逐条相同、都不归我：
   - `first_transaction_region_bytes`：`test result: FAILED. 3 passed; 1 failed; …`（等 kb，规格「已知不归你的红」）；
   - `second_transaction_supplement_three_fault_injection`：`test result: FAILED. 7 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out; finished in 53.61s`，
     红的是 `a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it`（「取号之后第 18 次写报错」，布局变了注入序号错开）
     与 `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave`——开工时就是这两条红（开工时另两条 `fast_tier`、`one_fixed_history` 也红，现在绿了）；
   - `bin:first_transaction_on_device`：`test result: FAILED. 12 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s`，四条都是开工时就红的（「发布 C 与发布 B 同型」段数 28+2+1+2 对 16+2+1+2 一类，布局变了），我加的那条绿。
   动到的几个二进制的末尾：
   - admission_formula `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.15s`
   - rollback_witness `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.28s`
   - random_history `test result: ok. 20 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 376.14s`
   - checker_known_bad_images（run4 之后加了空叶那条，单独再跑一遍）`test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 57.49s`
2. `cargo fmt --all -- --check`：exit 1，`Diff in` 只落在两个不归我的文件——`e158_root_choice_repair.rs`（5 处，E158 执行员的）与 `e156_allocation_basis_counts.rs`
   （9 处；这个文件开工时与我无关，00:32Z / 00:42Z 被别的会话改过，我没碰）。我改过的文件全部干净。
3. clippy（check.sh 那一套 `-D warnings` 加七条编码纪律 lint）：`--all-targets` 整跑 exit 101，唯一的错在 `e156_allocation_basis_counts.rs:4037`（`assertions_on_constants`，别的会话刚改的）；
   绕开那一个 bin 分开跑：`clippy core/checker/format exit=0`、`clippy harness lib + every integration test + four bins exit=0`、`clippy harness bins in test profile exit=0`。
   （e156 被改之前，我这一轮的 `--all-targets` 整跑一次是干净的。）
4. `cargo build --offline --all-targets`：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.06s` / `build exit=0`。
5. 登记给我的门禁阶段：33 号 exit 0（末行见第三节）；53 号 exit 0；92 号 exit 0；94 号 exit 0；93 号 exit 0；89 号 exit 77（本次未跑）；
   **74 号 exit 1**：`✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：「随机历史：逼近分配记录墙的取样点」`——阶段找的段标题是「逼近分配记录墙」，
   测试打的是「── 随机历史：越过原分配记录墙的取样点 ──」（实二一拆墙时改的标题，开工那一刻的日志里就是这个）；那一段在 release 下照样打了「模型对拍 4452 步」。
   不是这一轮的改动带来的，没修（要改的是 74 号的 `SECTIONS`，在 `.claude/` 下，不归我）。我加的那一段标题「── 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） ──」
   不与 74 号认的五段重名。

## 八、要知道的事（不归我、但这一轮的改动碰得到）

- **门禁 55 号**：raise-rollback-floor 模式多打一行 `name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3`（宿主上照同一条路跑出来的值），
  排在 `name=recover_cold` 那一行之后、分段计时行之前；末行的条数随之多 1。55 号的判据与预录样本由主 agent / 崩溃验证员改，我没碰 `.claude/`。
  这一行的 F 取的是恢复择根那一套现读的所选根（`choose_system_configuration` + `choose_root`），读不出时两段都写 `none`。
- **层 0**：没加新流。已有流的录制流不因这一轮的改动变：4 GiB 固定脚本上式子一次都不拒；见证的补与删只在「见证缺了一条」「表里有互相罩住的两条」时才动写出的字节，
  固定脚本里回退一次、见证恒一条；核出对不上的重读只在读出坏字节时多一次读（读不进录制流）；抬 F 的退回只在第一次空发布被拒时。这是推的，提交时的层 0 全量为准。
- **E158 装置**：它调 `mount_writable` / `mount_rollback`（签名没变）。回退见证的补与罩住的删会改变「回退挂载崩在写行轮换之前」那一类历史之后的择根结局，
  E158 若造了这类历史，数会变。我没跑 E158。
- **E156 装置** 这一轮被别的会话改着（上面 fmt / clippy 那两处），与我无关。
- **小盘上的随机历史、C517 那条用例** 改走开关之后测的是「准入放行之后的兜底拒绝」，不再是产品路径；产品路径在小盘上的样子由新加的「准入判着」那一段取样钉着（第六节 Q2）。

## 九、干到一半收到的主 agent 消息

1. 2026-09-24T23:47Z 左右（写进度文件之前，那时八件都已写完第一遍、正在加测试开关）：整点询问 + 「门禁 33 号在主工作区红，点名 transaction.rs、mount.rs 的锚点」。进度写进 `progress.md`；33 号按第三节修绿。
2. 2026-09-25T00:38Z 左右（第一批证红刚跑完）：共用约束第 57 行加了第五种拒绝写法（整份覆盖 `research/results/` 下已存在且未跟踪的文件）。这一轮不写 `research/results/`，手上步骤不用改。

## 十、没做什么

- 没走三方对抗；层 0（没有新流、已有流一条没跑）、QEMU / 门禁 55、herd7、crates 变异表整表复跑（门禁 59）归 crash-verifier；没提交、没做任何 git 写操作。
- 没写 kb、没写 `research/`、没碰 `.claude/`：D28 已定项 1 / 4 的「接进来了」、D23 已定项 14「实现取法」①⑥的现状、D19 已定项 5「对不上也先重读」的实现状态、
  C545 / C546 / C547 各行的现状，交书记员；74 号的段名交主 agent。
- 没跑：全量 `cargo test --all`、`check.sh`、`gate.sh` 全量、名字含 layer0 的二进制、`publish_order_matches_litmus`（herd7）、E156 / E158 装置。
- 第 8 件没改 walk.rs（前提不成立，第六节 Q6）。
- 变异表第 307、494、496 行点名的测试不改动时就红，没法证（第五节表二）；第 499、500 行没复验（与 498、497 同一条用例，留给 59 号）。
- 第六节 Q1–Q6 的读法都没走三方、被攻过零轮。

## 五（续）、开关保住了原来挂在小盘旧用例上的变异

第 153、434、497、498 行（原来点名的四条小盘用例现在装「准入关掉」的开关）同一套脚本复验（`logs/prove-switch/`），都红：
153 → random_history.rs:394「新发现…」；434 → root_ring_turn_in_one_mount.rs:270「不重挂：同一次覆盖写发得出去…」；
497 → random_history.rs:603「NewFinding { … 拒绝之前写了盘 …」；498 → random_history.rs:728「NewFinding { CheckerViolations { I-3.1 } …」。
第 499、500 行与 498、497 点名同一条用例，留给 59 号。

## 附：`git diff --stat -- crates litmus` 原样（含别的会话此前留下的未提交改动与 e156，分不开谁改的；我改的是第二节那 22 个）

```text
 crates/mutations.tsv                               |  323 +-
 crates/singlefs-checker/src/image.rs               |   79 +-
 crates/singlefs-checker/src/lib.rs                 |  100 +-
 crates/singlefs-checker/src/walk.rs                | 2170 ++++++++-
 crates/singlefs-core/src/admission.rs              |  174 +-
 crates/singlefs-core/src/allocator.rs              |  204 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  139 +-
 crates/singlefs-core/src/lib.rs                    |    4 +
 crates/singlefs-core/src/make_filesystem.rs        |    2 +
 crates/singlefs-core/src/mount.rs                  | 1066 ++++-
 crates/singlefs-core/src/mounted_read.rs           |  285 +-
 crates/singlefs-core/src/recovery.rs               | 1190 +++--
 crates/singlefs-core/src/system_configuration.rs   |   76 +-
 crates/singlefs-core/src/transaction.rs            | 4934 +++++++++++++++-----
 crates/singlefs-core/src/write_accounting.rs       |   20 +-
 crates/singlefs-format/src/lib.rs                  |  131 +
 crates/singlefs-harness/src/bad_disk_input.rs      |  335 +-
 .../src/bin/e156_allocation_basis_counts.rs        |  597 ++-
 .../src/bin/e158_root_choice_repair.rs             | 2103 ++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  253 +-
 .../src/bin/first_transaction_on_device.rs         |  957 +++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    5 +-
 crates/singlefs-harness/src/history.rs             |  220 +-
 crates/singlefs-harness/src/model.rs               |  461 +-
 crates/singlefs-harness/src/model_comparison.rs    |   89 +-
 crates/singlefs-harness/src/on_device_modes.rs     |  131 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              | 1298 ++++-
 crates/singlefs-harness/tests/common/mod.rs        |   29 +
 .../tests/first_transaction_step_five_publish.rs   |  313 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_six_recovery.rs   |    5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../tests/parallel_line_one_sequential_write.rs    |   55 +-
 .../second_transaction_mapping_node_admission.rs   |  230 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |  167 +-
 ...ansaction_parallel_line_one_sequential_write.rs |   52 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  256 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  343 +-
 .../tests/second_transaction_step_five_reuse.rs    |   29 +-
 .../tests/second_transaction_step_four_rollback.rs |  241 +-
 .../tests/second_transaction_step_one_overwrite.rs |  218 +-
 ...second_transaction_step_three_formatted_pool.rs |  192 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |  103 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ..._transaction_supplement_one_write_accounting.rs |   97 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   37 +-
 ...transaction_supplement_three_crash_injection.rs |    3 +
 ...transaction_supplement_three_fault_injection.rs |  131 +-
 ..._transaction_supplement_three_random_history.rs |  501 +-
 ...nsaction_supplement_two_accounting_node_full.rs |  134 +-
 ...transaction_supplement_two_admission_formula.rs |  318 +-
 ...two_c533_row_publish_record_without_its_root.rs |   87 +-
 ...ion_supplement_two_commit_generated_fallback.rs |  197 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   53 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +-
 ...n_supplement_two_release_checksum_quarantine.rs |  906 +++-
 ...saction_supplement_two_reused_record_overlap.rs |   19 +-
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    6 +-
 ...saction_supplement_two_row_publish_admission.rs |  358 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  461 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |   44 +-
 .../system_configuration_mutability_classes.rs     |    5 +
 66 files changed, 18379 insertions(+), 5028 deletions(-)
```
