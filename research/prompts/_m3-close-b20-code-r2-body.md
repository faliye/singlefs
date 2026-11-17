# m3-close-b20-code-r2 正文：C971 与 C973 代码轮第二轮（2026-11-08）

批次：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 20 批。方案按 `crates/` 今天的实现来谈。上一轮判决 `research/prompts/m3-close-b20-code-r1-main-verification.md`。这一轮只攻第一轮站住的形态与第一轮打中之后的改法，不重复第一轮已攻的角度。

## 被判的产品代码

| 文件 | 范围 | 来历 |
|---|---|---|
| `crates/singlefs-core/src/recovery.rs` | 第 5129 行字段 `slots_of_another_pool_whose_checksum_verifies`、第 5154–5190 行它的计数 | r1 打中「别池的槽」的改法，补丁 `~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/patch/`，任务 10242 打进主工作区 |
| `crates/singlefs-core/src/mount.rs` | 第 9236–9260 行 `some_system_configuration_slot_fails_self_verification_and_no_device_self_verified_both_slots` 新那一支；第 399–401 行拒因注释 | 同上 |
| `crates/singlefs-core/src/instance_table_sweep.rs` | 第 601 行起第 0 代根判据 | r1 判成立、只攻过一轮；这一轮算第二份样本 |

## 实现今天的样子（主 agent 的观测）

- 读过：`crates/singlefs-core/src/recovery.rs` 第 5125–5195 行；`crates/singlefs-core/src/mount.rs` 第 9326–9340 行与第 9236–9260 行；`crates/singlefs-core/src/instance_table_sweep.rs` 第 605–614 行。
- 别池的槽：校验和过而 fsid 不是本池的槽记进新字段；R2 本身、读坏账、只读挂载与恢复不读它；乙那一判多一支：有盘带别池的槽，且没有一块盘两槽都自证过，判真。

## 压着的条款

- `research/prompts/e205-preregistration.md` 第 46、47 行；`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8 射程（第 185 行，r1 写回之后是「第一遍判据为真时整个读阶段重读一次，只按重读那一遍判」与别池槽算自证不过）；`.claude/kb/checks-owed.md` C707、C971、C973 三行。

## 要两条腿回答的问题

1. 辩方腿：复核 r1 判决。三处打中够不够得着、改法修没修到打中的格；r1 判「成立」的几格（尤其 C973 判据只有一次抽样的「没打中」）站不站得住；「重读粒度改条款不改代码」这一判是不是同样打中别的读法。
2. 攻方腿，换攻击面，不重复 r1 的五面：别池的槽与撕裂、全零、读错同时出现的组合；只读挂载与离线修复的路径会不会因为新字段或乙这一拒走错；取号、F2 退回、逐盘核确实不读新字段；随机历史新步与理想模型 `answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots` 判的是不是产品判的同一件事（模型与实现共用错误前提的可能）；透明判官择系统配置不按 fsid 筛、可写挂载按 fsid 筛（`crates/singlefs-verification/src/mount_transparency.rs`，实现员报告 `~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/result.md`）能不能让判官判错；C973 判据在 S 不是 2 的幂、环绕一圈以上时的走向。
3. 两条腿都对下面八格逐格回「成立 / 不成立（文件:行）/ 推不出」。

### 检查单：走不到的理由二次查实

本轮改动没有这一类：实现员报告 `~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/result.md`「走不到的理由」一节写「无。产品代码（`crates/singlefs-core/src/`）里没有新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员。」腿照调用点核这句话。

### 检查单：类型与名字

新字段 `SystemConfigurationSlotsOfOneDevice::slots_of_another_pool_whose_checksum_verifies: u64`（`recovery.rs:5129`，是计数）。新加行没有 `expect`。腿核有没有 `_ =>`、缩写、裸整数混用。

### 检查单：不兼容旧代码与旧格式

主 agent 看过改动行，没有认旧版本的分支、旧布局号或迁移路径；腿复核。

### 检查单：不新建判定库

本轮改动没有这一类（查过 `recovery.rs`、`mount.rs` 改动行与三条新用例）。

### 检查单：所有任务多机可用

新用例在 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs`，用内存盘，不读本机状态。腿复核。

### 检查单：测试优先多线程

三条新用例各是单场景一个盘面一次挂载，没有切片。腿判该不该并行。

### 检查单：带 --item 的命令与项

变异行：6 行追加在 `crates/mutations.tsv` 第 7064–7069 行，主 agent 任务 10312 带 `--item` 复判（等结局）；证红 10241「抓到 3 条」。没有新崩溃枚举项。

### 检查单：新崩溃流多罩了什么

本轮没有新加崩溃点重放流或崩溃枚举用例；三条用例是单盘面用例。

## 已知的验证读数（主 agent 量的）

合入后验证 10243：测试 218 条全绿（只跑补丁改到的 3 项）、fmt、clippy、build 过；挂特性那一段只红 `crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`（HEAD 上同样红，任务 10227；钉值已改，重跑 10316 等结局）。r1 模型全量（任务 10233）S = 4、5、8、16 的 `hidden`、`equality`、`other-mismatch` 都是 0，模型是攻方照代码手抄的。
