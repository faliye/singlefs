# 已结束会话留下的改动：代码轮第一轮判决（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 M1 M2 M3 Z2 Z4 R2 H4 LR1 LR2 LR3 LR4 LR5 LR6 LR7 LR8 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-leftover-code-r1-body.md`、`_m3-leftover-code-r1-background.md`、`_m3-leftover-code-r1-diff.md` | — |
| 开工快照 | `research/prompts/m3-leftover-code-r1-snapshot/`（crates 32 份、kb 4 份；冻结副本 `/tmp/claude-1000/m3-leftover-code-r1/tree/`） | `bash scripts/devices/snapshot-check.sh m3-leftover-code-r1`：「✓ 快照 m3-leftover-code-r1：36 个文件都与开工时一致」 |
| 云端正推（Sonnet） | `research/prompts/m3-leftover-code-r1-sonnet-output.md`（247 行，sha256 69e44687…），模型 `…-sonnet-model/`（k1、k3、k4、k5） | — |
| 云端攻方（Opus） | `research/prompts/m3-leftover-code-r1-opus-output.md`（sha256 d3fa52e2…），模型 `…-opus-model/` | 主 agent 现查 `logs/` 下被判「放过」的四份：`mut-slot1-only-both.log.lines` 首行「name=slot1-only-both cases=98 failed=0」，`mut-raise-no-restore`、`mut-map-first-instance-page-only`、`mut-defer-includes-abandoned` 三份都是 `failed=0` |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root . --background <背景材料>`：「✓ 核了 157 处引文，对上 157 处，没判 0 处」（快照 36 份与仓里一致，仓根与冻结副本同值） |

正推腿头一条现查坐实：`.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行写「判不出取 R2……用户定「改取 R2」；这里的实现还是 2026-09-28 取甲的那一版，欠 C331（择根倒挂压过已确认的写）」。M1 ① 的条款今天是 R2，实现欠着；R2 的实现归「里程碑三开工第一轮接手」那条线（那条线的会话 2026-10-03 回信：它下一批根环实现那一项含 R2，C331、C662 与 D23 已定项 14 的欠都归它）。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 ① C331 取甲 | 对 09-28 定案一致；对今天 kb 的 R2 冲突；`NoSelfVerifiedSystemConfiguration` 那一支今天 0 命中 | — | **条款已改，实现归别的线**：不在这一条线改；正推那两句并排与 K3 的结局交开工第一轮接手那条线 |
| K1 ② C393 (b) | 一致；最后一支「仍不成就拒」有用例、没有变异行 | Z2 ② 没打中：152 条根逐条比两臂，带文件的 107 条逐个相同 | **站住**（一次抽样）；补变异行 LR4 |
| K1 ③ F 算进 F_生效 | 一致；kb 内部出入：D16 已定项 1「生效」表格行没有被抛弃根，依据那一段有，代码照后者 | Z2 ⑤ 没打中（推的） | **站住**；kb 出入进写回批 LR8 |
| K1 ④ 丁-defer | 一致；条款自留的「没拿变异试过」仍开 | Z2 ④ 没打中：抬 F 一槽不回收的变异下 I-3.1 照红（56 条红 7 条） | **站住** |
| K2 Z2 ① 缺一槽 | — | 没打中「永远只读之外的新形」；永远只读多一个成因：吞掉 mkfs 写系统配置槽的那次写（C600），不靠种子的造法在模型目录 | **坐实 C600 的成因**；C600 随 R2 落地，交开工第一轮接手那条线 |
| K2 Z2 ③ 影子账被拒换回分配器 | — | 次序对，没有用例走到这一支（量过） | **打中覆盖缺口**：LR3 |
| K2 Z4 ① 错实现过用例 | 正推另列：第二块盘那一格、读错形、逐盘核 `Err(device)` 出口没有变异行 | 4 种放过：见证读与取号只看槽 1（98 + 188 条全过：写坏槽的用例都坏世代 5、落在槽 1）；现算只认实例表第 0 片（69 条全过）；影子账被拒不换回分配器（69 条全过）；defer 并进被抛弃根引用的（56 条全过，没造出判得不同的合法状态，可能等价） | **打中**：LR1、LR2、LR3；第四种照 `.claude/rules/mutation-sampling.md` 先判等价还是取样点不敏感（LR5） |
| K2 Z4 ② 造盘面 | — | 孤记录那一格（`writable_mount_of_a_formatted_pool.rs:222` 自注「真崩溃留不下这一形」）只有说谎设备留得下 | **打中**：LR6 |
| K2 Z4 ④ 变异表新 6 行 | — | 各自红点名的用例 | **站住** |
| K3 C600 两条出路 | 甲的拒绝集合包含 R2 的；按 R2 字面 C600 不属「判不出」 | 按 R2 起草：C600 8 段全挂上；H4 一格挂上并丢已确认的发布（R2 自认的纯形丢写），其余五格拒；钉甲的 98 条翻 11 条（3 条是草图取号一侧少一条可见性核造成） | **不是待选的岔路**：用户 2026-10-03 已定改取 R2，C600 在 R2 下挂上；结局表交开工第一轮接手那条线的实现员参考 |
| K4 写零折叠 | 切法与 `write_zeroes_in_chunks` 一致；缺一块不折（对拍 5954 万 + 4000 万条序列不一致 0）；旧规则折的新规则全折；长度 0 守卫、接续写越过段尾、两个 `assert` 三处没有用例钉 | 没打中：抽成单文件模型随机 20 万轮 | **站住**；补用例 LR7 |
| K5 窄一断言 | 一致，偏严一处（「最新根」按魔数认、不验自证，多拒不漏拒）；登记流误不误拒要跑 checker 档 | 没打中 | **站住**；登记流那一半归提交时 checker 档快档 |
| K5 白名单 | 能绕过：`use singlefs_harness as h; h::history::…` 这类别名写法下 `history.rs` 的引入者看不见；今天仓里 0 处 | — | **打中潜伏缺口**：记欠账（LR8 写回批），不改代码 |
| K5 口径与池几何 | 数值去空白、写 0 判红一致；选项型变量不去空白（拒的一侧）；9 个使用者池几何一致，checker 档那 1 个抄了三个数，与原件逐字相同 | 一处口径不一，落在拒的一侧 | **站住** |

## 三、打中的改法（零轮，并进下一批 crates）

| # | 改法 | 验收 |
|---|---|---|
| LR1 | 补坏槽 0（偶数世代）的可写挂载与取号用例；第二块盘那一格、读错形、逐盘核 `Err(device)` 出口各补一条变异行 | 「只看槽 1」那一种错实现至少红一条点名用例 |
| LR2 | 补两片实例表上的 C393 现算用例 | 「现算只认实例表第 0 片」红 |
| LR3 | 补抬 F 一串在影子账上被拒的用例（被抛弃根的账与映射重读之后都读不出） | 「被拒不换回分配器」红；走到 `isolate_slots_referenced_only_by_abandoned_roots` 交 `Err` 那一支 |
| LR4 | C393 最后一支「仍不成就拒」补变异行 | 证红 |
| LR5 | 「defer 并进被抛弃根引用的」：先找让它与原式判得不同的合法状态；找到就补用例，找不到把等价性写成一条测试留档 | 照 `.claude/rules/mutation-sampling.md` 三分写清 |
| LR6 | 孤记录那条用例改用 `WriteIsSwallowed` 吞取号写造盘面，或在用例名与注释写明测的是说谎设备留下的盘面 | 两者取前者；做不到写明为什么 |
| LR7 | `device_log.rs` 长度 0 守卫、接续写越过段尾、两个 `assert` 各一条用例或变异行 | 各自证红 |
| LR8 | 写回批：D16（发布语义） 已定项 1「生效」表格行与依据段的出入；C662 / C331 不动；白名单别名绕过记一条欠账 | 规格检查绿 |

LR1 会动见证读与取号的用例：开工第一轮接手那条线的 R2 实现会改这两处的判法，LR1 派之前与那条线对文件，R2 落地之后用例按 R2 的结局写。M2、M3 其余各格站住；M1 ① 的改动不在这一条线。

## 四、复跑核对

主 agent 没复跑腿的模型；判决用到的放过计数是腿日志原行（第一节）。checker 档没跑，窄一断言对登记流误不误拒归提交时 `checker-tier-layer0-replay` 快档。

## 五、被判的文件

这一轮判的 `crates/` 文件（冻结副本与仓里同值）：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash_identity.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/device_log.rs`、`crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs`、`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs`、`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/numeric_environment_variable.rs`、`crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs`、`crates/singlefs-harness/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs`、`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`、`crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs`、`crates/singlefs-harness/tests/common_admission/mod.rs`、`crates/singlefs-harness/tests/common_admission_pool/mod.rs`、`crates/singlefs-harness/tests/common/mod.rs`、`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`、`crates/singlefs-harness/tests/fault_injection_fast_tier.rs`、`crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs`、`crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`、`crates/singlefs-harness/tests/random_histories.rs`、`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs`、`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-03 不受影响：条款已是 R2，实现欠着归 C331，不在这一轮改 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：代码照依据段做；表格行与依据段的出入进写回批 |
