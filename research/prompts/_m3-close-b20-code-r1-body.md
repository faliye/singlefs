# m3-close-b20-code-r1 正文：C971 乙入判不出与 C973 第 0 代根判据的代码轮（2026-11-07）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 20 批。方案按 `crates/` 今天的实现来谈。

## 被判的产品代码

| 文件 | 范围 | 来历 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 第 517–560 行（比法枚举加成员 `SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots` 与它在 `SelectedVersionAgainstTheWitness` 里的处置）、第 6603–6620 行、第 9133–9260 行（`newer_publish_witness` 与新函数 `some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots`，第 9233 行起） | C971 乙入判不出，实现员补丁 `~/.local/share/singlefs/work/m3-close-b20-c971-yi/patch/`，2026-11-07 打进主工作区（任务 10221） |
| `crates/singlefs-core/src/instance_table_sweep.rs` | 第 467–530 行（清扫读根环的自证判法）与第 601 行起 `genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached` | C973 第 0 代根判据，判决 `m3-close-b19-switch-sweep-code-r1` 第五节定的改法，已随 2026-11-07 那次提交进 HEAD；被攻过零轮。同一份文件这一批另有 20 处只改名（`ARootRing…` 等枚举成员去掉开头单字母、测试函数去掉 `a_`），行为不变 |

## 实现今天的样子（主 agent 的观测）

- 读过：`crates/singlefs-core/src/mount.rs` 第 517–560、9133–9260 行；`crates/singlefs-core/src/instance_table_sweep.rs` 第 460–640 行；`crates/singlefs-verification/src/model.rs`、`history.rs` 里补丁加的 `answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots` 与新步 `WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros`。
- C971 改法在读阶段的见证读：系统配置槽读得出而自证不过（撕裂、读回全零），且没有一块盘两槽都自证过，就给新的比法成员，按判据为真处置；用读阶段已有的那一次重读从盘上重读系统配置槽，仍是这一形就拒可写，拒因沿用已有的 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`。取号、F2 退回、逐盘核照 R2 本身判。
- C973 改法在清扫读根环：除调读者同一个落点核外，区域 r 的槽 0 读到第 0 代根、而环里已有落点公式会写进这一槽且 txg 更大的根时，算这一槽自证不过，走原有「重读一次仍不过这一轮不删」。

## 压着的条款

- C971：用户 2026-11-07 定「乙入判不出」，定义在 `research/prompts/e205-preregistration.md` 第 46 行：「乙入判不出：重读之后仍是这两种之一的槽，并进 R2 的「有槽设备报读错」那一半当读错计，R2 的另一半（没有一块盘两槽都自证过）照旧；判不出就拒可写。」（「这两种」是读得出而自证不过与读回全零，见同一份登记第 40–46 行）。决策落点：`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8 射程（第 185 行）；D23 已定项 14 第 415 行。代价数：E205 页「交用户的岔路表」乙入判不出那一行（撕裂轮换 513 态多拒 57 态，57 / 57 落在 C596（见证判不出的系统配置盘面只剩只读挂载） 已定的冲突格）。
- C973：`.claude/kb/checks-owed.md` C973 那一行的「怎么拦」与去向（第 0 代根判据的原文在那一行）；D16（发布语义） 已定项 6 第 195 行「恢复、挂载与实例表清扫读根环时都调」。

## 要两条腿回答的问题

1. 正推腿：C971 那几处代码做的是不是第 46 行那一句说的——认的是不是恰好「读得出而自证不过」与「读回全零」两种；重读是不是恰好一次、读的是系统配置槽不是根；「没有一块盘两槽都自证过」是不是照 R2 原有的那一半判；拒是不是在第一个写之前；拒因沿用已有成员有没有让别处的判断（只读挂载、C596 那一格）走错。C973 第 0 代根判据做的是不是 C973 那一行写的改法。
2. 攻方腿：哪一格会错。至少攻这几面：一块盘两槽都自证过、另一块盘一槽读回全零时会不会误拒；三块盘以上（若格式允许）判法是不是还对；重读那一次本身读故障时的走向；C973 判据在第 0 代根读自别的区域、环里还没有任何根写进那一槽、或落点公式算出的 txg 恰好相等时的走向；错位读回的是别槽的非第 0 代根时两条判据会不会叠加误判。
3. 两条腿都对下面八格逐格回「成立 / 打中（文件:行）」。

### 检查单：走不到的理由二次查实

本轮改动没有这一类：四份实现员报告（`~/.local/share/singlefs/work/m3-close-b20-c971-yi/result.md` 至 `result-4.md`）「走不到的理由」一节都写产品代码里没有新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员；`mount.rs` 新加的 `SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots` 是比法枚举成员、条款已定。C973 那一段的实现员报告同样没有。腿照调用点核这句话成不成立。

### 检查单：类型与名字

新名：比法成员 `WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots`（`mount.rs:560`）、函数 `some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots`（`mount.rs:9233`）、`genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached`（`instance_table_sweep.rs:601`）；改名的五个枚举成员见 `instance_table_sweep.rs` 第 283–308 行。主 agent 现查：`git diff HEAD -- crates/singlefs-core/src/mount.rs` 新加行里没有 `_ =>`、`expect(`、`unwrap()`。腿核 txg、槽号、区域号是不是用 newtype，有没有裸整数混用。

### 检查单：不兼容旧代码与旧格式

主 agent 现查新加行没有认旧版本的分支、旧布局号或迁移路径；腿复核。

### 检查单：不新建判定库

产品代码改动不存判定；本轮改动没有这一类（查过 `mount.rs`、`instance_table_sweep.rs` 的改动行）。

### 检查单：所有任务多机可用

产品代码改动不读本机状态、不写主机名；新测试在 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs`，用内存盘。腿复核。

### 检查单：测试优先多线程

新测试是 11 条单场景池级用例（每条一个盘面一次挂载），没有切片；随机历史与崩溃注入沿用已有的多线程跑法。腿判该不该并行。

### 检查单：带 --item 的命令与项

实现员报告列的 `--item`：crates-mutation-replay 22 行（1948、2284、2285、7045–7063，主 agent 任务 10223「点名的 22 条变异都抓到了」）；崩溃点重放两项 `read-faulted-states-never-hold-the-witness-criterion`、`random-histories-and-fault-injection-given-states`（合入后验证 10222 的验证包那一步按块跑过，现判 627、复用 18）。腿核新项能被 `--list-items` 列出。

### 检查单：新崩溃流多罩了什么

本轮没有新加崩溃点重放流；随机历史加了一步 `WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros`（只藏一块盘最新那一槽，C696 那一组比重），原来那一步（每块盘都藏）以 1 份留在同一组。取样点实测（任务 10211）：每块盘都藏那一步 Ok 0、Err 6，只藏一块盘那一步 Ok 23、Err 0。腿判新步比已有的多罩了什么。

## 已知的验证读数（主 agent 量的）

- 合入后验证 10222：测试 625 条全绿（只跑补丁改到的 9 项）、fmt、clippy、build 过；验证包崩溃枚举用例过；挂特性那一段红 1 条 `crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`，在 HEAD 干净工作树上同样红（任务 10227），不是这一批的。
- E158 装置单测在补丁上 95 passed（任务 10210），修前红的 8 条经调查判为用例造的盘面正是乙入判不出要拒的那一形（`~/.local/share/singlefs/work/m3-close-b20-c971-e158/result.md`），推翻条件 10207、10208 都绿。
