# 删除批第 11 批合入的代码：代码轮第二轮判决（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 K1 K2 K3 H1 H2 H3 H4 G1 G2 G3 G4 MU1 MU2 MU3 V1 V2 V3 V4 V5 V6 U1 U4 U8 U9 U12 U13 U14 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-unlink-code-r2-body.md`、`_m3-unlink-code-r2-background.md`、`_m3-unlink-code-r2-diff.md` | — |
| 开工快照 | `research/prompts/m3-unlink-code-r2-snapshot/`（crates 21 份、kb 3 份；冻结副本 `/tmp/claude-1000/m3-unlink-code-r2/tree/`） | `bash research/scripts/snapshot-check.sh m3-unlink-code-r2`：「✓ 快照 m3-unlink-code-r2：24 个文件都与开工时一致」 |
| 云端正推（Sonnet） | `research/prompts/m3-unlink-code-r2-sonnet-output.md`（226 行，sha256 7308ab2f…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-unlink-code-r2-opus-output.md`（253 行，sha256 c2a1410b…），模型 `…-opus-model/`（`logs/` 18 份） | 主 agent 逐字核 `logs/sweep2.log` 第 937 行、`sweep3.log` 第 557 行、`mu1-tests.log` 第 22 行、`mu2-tests.log` 第 39 行、`mu1-probe.log` 第 897 行、`mu2-probe.log` 第 398 行，都与报告抄的相同；另把 `sweep2.log` 里续做阶段 `Publish` 注错点的「落下的写 − 取号 − 写账」重算：差 0 的 590 个点、差 2 的 242 个点 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root /tmp/claude-1000/m3-unlink-code-r2/tree --root . --background <背景材料>`：「✓ 核了 96 处引文，对上 96 处，没判 0 处」 |

两条腿的数都在冻结副本的拷贝上、只跑 harness 档量；checker 档没跑（`crash_injection.rs`、`device_log.rs`、`tests/crash_injection_campaign.rs` 与它们的变异行归提交时）。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| N1 结局与文档 | 结局、会话先走空一致；U14 文档改写一致；「正常卸载照常」冲突：`mount.rs` 第 913、4373 行（第 11 批写的）与第 4508 行（挂载准入那一处原有的）写它成立，同批用例 `admission_raises_the_floor_before_refusing.rs` 第 505 行断言卸载 `Err` | 打不到：执行器的历史里空间不够那几支与 `StillShortAfterTheFloorRaises` 一次都没出现，`history.rs` 第 3795 行那一支没被模型比过（模型只罩 1 个单元的文件） | **文档冲突坐实**，改法 V5 等设计轮 `m3-unmount-after-short-continuation-r1` 定了再写；模型比不到那一支与 U13 同一个病根，随 U13 进下一批 |
| N2 写账条数与次序 | 一致（75 = 73 + 取号 2）；按 txg 先后、准入推的串进账一致但没有入库用例钉（探针删掉对应代码入库用例全绿） | H3 打中：MU1 删准入推的、MU2 删续做推的，`unlink_mount_continuation_and_the_emptied_version` 与 `admission_raises_the_floor_before_refusing` 两份全绿（日志原行见第一节） | **打中**：V3 |
| N2 抬 F 先写系统配置的写 | A 自报缺口①：字面不违反 `mount.rs` 第 728 行，违反第 292 行宣布的账对得上设备数，差 2 次写 | H1 打中：每推过一串抬 F 少 2 个写，准入推的那几串也少，A 的自报只罩续做推的一半 | **打中**：V2 |
| N2 / N5 抬 F 自己报错那一支 | A 自报缺口②：规则没说，与写行阶段 `FloorRaiseFailedAfterTheMountsPublishes` 口径不齐 | H2 打中：错原样交回（`mount.rs` 第 4474–4476 行），`crash_injection.rs` 落进 `Err(_) => None`（第 1653 行），续做写全归暖机段，一次最多 334 个写；战役用不注错的盘摆不出，判的是 U12 验收字面 | **打中**，主 agent 现查代码坐实：V1 |
| N2 U9 用例注释那句全称 | — | H4：注错点在记录写之后时份数比只读挂载少 1，写账合计照样对得上；老口径，不分辨本批 | **打中注释**：V4 |
| N3 | 覆盖写、unlink、建 inode 三支一致（变异行 2135–2138 复跑红）；走空被空间拒、走空多步没有用例 | 没打中（一次抽样）：854 段历史全 Completed，三支都走到 | **站住**（一次抽样）；走空多步随 U13 |
| N4 | 复跑 14 条变异行全部红；第 2144 行（LR2）两次超时复核不了 | 只攻了 U9（H3） | **站住**，第 2144 行由 A 证过红（A 报告第三节），主 agent 不另证 |
| N5 取界 | 按位置取、没退回按内容认，一致；准入与续做都推那一形读代码一致、没钉；U12 五种结构取法一致，`DataUnit` 与 `DeletedInodesTreeNode` 空转 | 没打中（一次抽样）：2456 个起点里 48 个续做推过、38 个两边都推，界与真值逐个相等 | **站住**（一次抽样）；「两边都推」补一条单测进 V6 |
| N5 挂载透明的 F 判 | 规则没说：只认准入推的（探针 3 `RollbackFloor` 判 `Violated`） | — | 用户已定「续做算挂载的一部分」，归下一批（与 U4 同一处） |
| N6 | 本批 40 行锚点全中；删的 4 行守的代码今天不存在 | — | **站住** |
| K3 | 21 份每份一格：20 份一致，`mount.rs` 部分一致 | — | 见第五节 |

## 三、打中的改法（零轮，并进下一批 crates）

| # | 改法 | 验收 |
|---|---|---|
| V1 | 续做里推的抬 F 自己报了不是空间不够的错时，照 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 的样子包一层，带上这次挂载写行、暖机、准入推的抬 F、续做已落盘各步与续做推成的那几串的写账（攻方 G1）；`crash_injection.rs` 的划段认这个成员 | 注错落在续做推的那一串里时，错里写账合计 = 设备落下的写 − 取号两次；划段不再交 None；改回原样交回的变异红 |
| V2 | 续做那一格交 `Publish` 时，准入推的与续做推的每一串先写系统配置那一步的写进错里（合进 `writes_before_the_first_publish` 或另立一格，实现员按契约文字选一种并改契约文档）（攻方 G2） | 「落下的写 − 取号两次 = 写账合计」在准入推过、续做推过、两边都推过三种起点上各一条用例；删掉那一段的变异红 |
| V3 | U9 补两条用例：准入推过抬 F 的起点与续做里推过抬 F 的起点，注错点放在抬 F 之后的续做一步（攻方 G3） | MU1、MU2 两刀各红一条点名用例，进变异表 |
| V4 | `unlink_mount_continuation_and_the_emptied_version.rs` 第 548 行注释改成只说用例挑的那个注错点上成立（攻方 G4） | — |
| V5 | `mount.rs` 第 913、4373、4508 行「正常卸载照常可走」照设计轮 `m3-unmount-after-short-continuation-r1` 的定案改写 | 等那一轮 |
| V6 | `crash_injection.rs` 取界补「准入与续做都推」一条单测；U12 划段的五种结构里空转的两种写明为什么留 | 单测点名那一形；checker 档，提交时跑 |

## 四、复跑核对

主 agent 没复跑腿的模型；判决用到的数是攻方日志原行（第一节逐字核过）与对 `sweep2.log` 的重算。第 2144 行正推腿两次超时，以 A 报告的证红为准。

## 五、被判的文件

这一轮判的 `crates/` 文件（冻结副本与仓里同值）：`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/device_log.rs`、`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`、`crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs`、`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`crates/singlefs-harness/tests/common_admission_pool/mod.rs`、`crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs`、`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`、`crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs`、`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs`、`crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs`、`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`。

门禁 `54-layer0-replay` 改名为 `checker-tier-layer0-replay` 带进 crates 的注释与 `#[ignore]` 理由（规格 `/tmp/claude-1000/m3-batch11/tooling/rename/outside-live-spec.json` 里 crates 的 45 条替换，主 agent 把新旧串里的门禁名、「54 号」、「门禁」去掉后逐条比对，45 条全相同，不交腿判）：`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`、`crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/layer0_progress.rs`、`crates/singlefs-checker-tier/src/verdict_store.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_floor_raise_pushed_by_admission.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_position_addressed_trees.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_a_formatted_pool.rs`、`crates/singlefs-checker-tier/tests/crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs`、`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-harness/tests/mounted_read_by_offset.rs`（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` 已在上一段）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-03 不受影响：打中的都是写账与用例，第 6 条续做的结局照用户定案；卸载那一形归设计轮 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：「正常卸载」那一行在续做推满的盘上做不到，归设计轮 `m3-unmount-after-short-continuation-r1`，这一轮只记代码文档与之冲突 |
