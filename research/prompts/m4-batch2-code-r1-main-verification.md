# m4-batch2-code-r1 判决：里程碑四第 2 批代码轮第一轮（主 agent，2026-10-13）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 T7 T8 T9 S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 G1 G2 G3 G5 G9 K-b K-c K-n I-1.13 I-4.4 I-3.1 I-3.11 I-1.11 I-1.12 I-4.2 I-5.1 I-9.16 I-9.25 I-9.27 -->

被判的产品代码（这一批四份补丁，冻结副本 `~/.local/share/singlefs/work/m4-batch2-code-r1/repo/`，sha256 在 `m4-batch2-code-r1-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/block_device.rs`、`crates/singlefs-core/src/lib.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/logged_operations.rs`、`crates/singlefs-core/src/region_scan.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-format/src/lib.rs`；连带判的池级 checker 与 checker 档：`crates/singlefs-checker/src/rebuild.rs`、`walk.rs`、`image.rs`、`lib.rs`，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`device_log.rs`。正文 `_m4-batch2-code-r1-body.md`，腿报告 `m4-batch2-code-r1-sonnet-output.md`（正推，275 行）与 `m4-batch2-code-r1-opus-output.md`（攻方，298 行）。

## 一、核查

- 快照：两条腿各自 `sha256sum -c` 冻结副本 16 行全 OK（两份报告开头原样）；腿跑着的时候主 agent 没改被判文件（轻型三条在工作树上跑，只读源码）。
- cite-check：正推 `✓ 核了 55 处引文，对上 48 处，没判 7 处`（没判的 7 处是 kb 与判决文件路径在冻结副本根下认不出，换工作树根再核：正推腿报告里写它用两个根各跑一次结果相同）；攻方 `✓ 核了 5 处引文，对上 4 处，没判 1 处`（`checks-owed.md:594` 同一原因）。
- 主 agent 在工作树（与冻结副本同内容）现查的行：
  - `recovery.rs` `key_width_for_kind` 的 `_ => None`（第 4016 行）、`user_visible_tree_root_pointers` 的 `_ => { … "树的种类没登记" }`（第 3473–3476 行）——T2 ① 成立。
  - `transaction.rs:8649` `let tree_table_entries = vec![`（写者每次发布照十二个种类现装树表）、`recovery.rs:2649` 文档「这一个丢掉它们」——T2 ② 成立。
  - `recovery.rs:2434` `inode_root_entry.tree.0 == extent_root_entry.head_identifier`——T1 攻方变异的锚点在。
  - `compaction_plan.rs:63` 按记录跨度逐槽计数、`region_scan.rs:189` `if reaches_in_from_below || reaches_out_above {` 整次拒——T5 成立。
  - `rebuild.rs:595` `.split_once("#[cfg(test)]")`——T8 ① 成立。
  - `crash.rs:2154–2166` `ZeroFill` 不当 COW、长于一个扇区且基镜像有非零字节就进第三态——T7 成立。
  - `crash_judge_conclusions.rs:420` `Holds => (violated, judged | bit)`、对拍用例 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2607–2608` 比判到位——T9 成立。
- 复跑：正推腿的 `tools/rerun.sh` 在 `~/.local/share/singlefs/work/m4-batch2-code-r1-main-rerun-forward/` 跑完退 0，`summary.txt` 与腿的 `out/expected.txt` sha256 相同（`6cefc30d262b81f55a93c8f6c2b7bcdd461b2ac61e1fa18af49c6b0013773da2`），`result=same`。攻方腿点名的两步（`s2-i113-random-histories`、`s5-region-scan-unpinned`）的复跑见第四节。
- 崩溃轻型（合入后在工作树跑，`CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=16 collected=16 green=12 red=2 not_run=2`，日志 scratchpad `light-crash.log`）：I-1.13 在各流的 CHECKER 行里评估计数 4 到 144 不等、违例一律 0，`I-1.13 Violated` 零命中——两条腿都没答的「I-1.13 在崩溃状态上会不会误红」在甲二与展开 6 的域上答了：不误红。两项红（crash-injection-fast-tier、position-addressed-trees）都是基线双机那条线的（那条用例在工作树单跑绿；按段登记规则全复用时误红，3c 归工具员改），不是这一批的。

## 二、判定与去向

| 格 | 判定 | 依据（腿 + 主 agent 现查） | 去向 |
|---|---|---|---|
| T1 配对判定 | 四样与 S1 一致；**打中**：用例缺「码 14 比码 15 多一条」，`recovery.rs:2422` 的 `!=` 改 `<` 六条全绿（正推探针 P2）；`:2434` 的 `==` 改 `<=` 六条全绿、两条码 15 指同一个码 14 的坏表被收下（攻方，量过） | 正推 T1、攻方 T1；主 agent 现查 `:2434` | S1 |
| T2 读侧 | match 一致；**打中 ①**：`walk_to_file`、`user_visible_tree_root_pointers` 对合法快照表报「树的种类没登记」（两腿探针一致，`:4016`、`:3473`）；**打中 ②**：`mount_writable` 收下带快照的池后，挂载自己的发布把码 14 / 15 全丢（攻方 `ATTACK_MOUNT … 码 14 / 15 共 0 条`，根在 `transaction.rs:8649` 现装十二条与 `recovery.rs:2649`「这一个丢掉它们」）；`previous_snapshot_txg` 读出即丢、写者恒写 0、`flags` 整字拒——与已知清单一致 | 两腿 T2；主 agent 现查 | S2（丙的核心） |
| T3 logged_ops | 偏移、44、369、与 D26 已定项 2「零格式位」相容——一致；round-trip 用例手写期望字节；读侧没接线（下一批） | 正推 T3，攻方没打中 | 无 |
| T4 区域扫描 | 五类与 G9 一致；**打中**：自定读法「隔离槽仍有映射条目指着时按普通单元判」翻成相反、`all→any`、位置条目不比盘、钉住只看起点槽四处各翻一次 12 条用例全绿（攻方，量过）——优先级的交叉没有用例钉住；主 agent 判读法本身：隔离（释放前核出校验和对不上）应优先于映射，隔离槽不许判 Movable | 攻方 T4、正推 T4（两个优先级变异存活） | S3 |
| T5 选段 | 与 G2 一致；**打中**（前提：盘上有一条跨段的坏记录）：扫描整次拒的段被 greedy 反复选中、整理卡死；改成 K-b 之后仍在——资格要另写「扫描拒了算不合格」；另：资格「至少一个活槽」已知（第 3 批改 K-b）、`plan_batch` 交空表分不出「搬空」与「搬不动」（正推） | 攻方 T5、正推 T5 | S4 |
| T6 discard 后端 | 默认实现与写零同一判据、文件后端三支一致；长度 0 在四个后端结局不同（规则没说）；没证红的 5 行里 harness 2 行正推证了红（`:129 / 217 / 273`），checker 档 3 行归提交时；真 `fallocate` 报不支持那一支没有用例（只有测试开关） | 两腿 T6 | S5（长度 0 统一）、C（真后端用例记欠账） |
| T7 两态 | harness 层两态与 G5 一致；**冲突**：`crash.rs:2154–2166` 对长于一个扇区、罩旧内容的 discard 给第三态（两腿一致，主 agent 现查）；`zero_fill` 同名分不出 discard 与 mkfs 清零；「一次 discard 段内部分落地」没枚举（攻方）；宿主与虚机结局不一致（已知） | 两腿 T7 | S6、S7 |
| T8 扫描重建 | 必红世界 ① ③ 在常量表之下仍红、I-1.13 的违例不是 I-3.1 / I-3.11 连带（正推）；**打中 ①**：白名单单测切点是源码第一处 `#[cfg(test)]` 字样，在 `use` 清单后加一行含这字样的注释再 `use crate::walk::check_pool_image` 两条单测仍绿（攻方 W1 对照，量过）；**打中 ②**：C827 只写两格，实测 C615（谱系）与 C768（退役）两格也走得到且占不适用的大头——随机历史 1736 态：违例 0、成立 960、不适用 776（C615 529、C768 243、C827 变短 4）；删过一次文件之后的 241 态 0 个成立（攻方 s2，复跑见第四节）；unlink 到底后报不适用·退役树（正推） | 两腿 T8；主 agent 现查 `:595` | S8、S9、交用户表第 1 行 |
| T9 覆盖表 | 收下标 92 与定案一致；编译期断言只证恒等列表；**打中**（两腿一致，推的，主 agent 现查 `:420`、`:2607–2608` 坐实）：I-4.4 在 CPU 恒不适用、I-1.13 在 CPU 真判，对拍比判到位，CPU 判成立的每个状态都红——第 3 批甲的接法（两个构造函数都不置位）把 CPU 一侧也屏蔽了，不合口径 | 两腿 T9 | S10 |

改法（都零轮，写进第 3 批派发提示；条款没写的由主 agent 定读法，标「主 agent 2026-10-13 定」）：

- **S1**（`recovery.rs` 用例）：补「码 14 比码 15 多一条」用例；变异两行：`:2422` `!=`→`<`、`:2434` `==`→`<=`，各自红。
- **S2**（S1 原文「读树表的每一处各加 14、15 一格」落地）：`key_width_for_kind` 与 `user_visible_tree_root_pointers` 给码 14 / 15 各一臂（码 14 / 15 的根不走读（它们由快照读路径解），`walk_to_file` 对它们 `continue`、`user_visible_tree_root_pointers` 把它们计进用户可见根——主 agent 定：码 14 / 15 是用户可见角色，D5 已定项 19 判定）；写路径发布时树表从重建版本带出的快照条目原样续上（`transaction.rs:8649` 现装十二条之后追加 `RebuiltVersionWithItsSnapshotEntries` 交出的码 14 / 15，`TreeTableEntry` 加 `previous_snapshot_txg` 字段、四处构造点同改）；`mount_writable` 用带快照条目的重建入口；池级 checker I-9.16 ③、I-9.25、I-9.27 各加 14 / 15 一格；用例：合法两对快照的池 `mount_writable` 后发一次布，盘上最新根仍两对、字段逐个相同。
- **S3**（`region_scan.rs`）：判类优先级钉成钉住 → 隔离 → 读头 → 类标签 → key 与位置（主 agent 认实现员的读法），**隔离槽一律判 Quarantined，不看映射**（主 agent 2026-10-13 定：隔离 = 释放前核出校验和对不上，搬它等于把坏副本搬走）；攻方四处翻转各钉一条用例与一行变异。
- **S4**（`compaction_plan.rs`）：资格改成判决 G2 的「R 里至少一个能搬的单元」，按 `scan_region` 的分类取；扫描被拒的段记进本次挂载的「不合格段」集合、不再选（主 agent 定：拒整次扫描的记录是盘上的坏记录，归 C411 的域，整理不替它修）；`plan_batch` 交空表时返回值分「搬空」与「搬不动」两种（枚举，不是空 Vec）；用例：跨段坏记录的池上 `choose_region` 不选那一段、选别的或报没有合格段。
- **S5**（`block_device.rs`）：长度 0 的 discard 四个后端统一交 `Applied` 且不计入 `applied_calls`（主 agent 定：长度 0 没有动作）；用例逐后端钉。
- **S6**（`crash.rs` 与 D13 口径，主 agent 2026-10-13 定）：discard 取态照 D13（验证路线） 已定项 4 原地覆写的字面——长于一个扇区、罩住的范围里原来有东西的取第三态；`memory_pool.rs:612` 那句注释与判决 G5「两态」改成「照 D13 已定项 4 取态」；第 3 批丁发 discard 时在崩溃流上验第三态枚举得到；「一次 discard 段内部分落地」是不是第三态的另一种形态，写进 D26 未定项 9 由丁的崩溃流答。
- **S7**：`zero_fill` 同名——记欠账 C829（discard 与 mkfs 清零在段种类串里同名；出路：`StepKind` 加 `Discard` 成员，连着 `crash_facts.rs`、`crash_judge_tables.rs` 的种类表改，等 3c 的崩溃放量改动提交后再做）。
- **S8**（`rebuild.rs` 白名单单测）：切点改成「行首恰是 `#[cfg(test)]` 的那一行」且先剥掉 `//` 注释与字符串字面量再找 `walk::`、`check_pool_image`；攻方 W1 那种注释绕过要红。
- **S9**（kb）：C827 行补 C615（谱系记录）、C768（退役记录）两格走得到与攻方的计数（1736 态里不适用 776，C615 529、C768 243）；交用户表第 1 行定谱系与退役记录在扫描重建里怎么读。
- **S10**（`crash_judge_gpu.rs`、`crash_judge_conclusions.rs`）：对拍「不比」改成只在比较处去掉下标表里的位，CPU 两个构造函数照旧置位；`crash_judge_gpu.rs` 文档「对拍逐位比、不屏蔽」那句同改；第 3 批甲补丁里五个辅助函数统一取位那一半沿用（`~/.local/share/singlefs/work/m4-judge-mask-writer/patch/`）。

两腿分歧：没有。T4 正推只报「规则没说」，攻方量出四处翻转存活，两边说的是同一件事的两面。

## 三、交给下一批的实现员（按文件聚簇）

| 簇 | 改法 | 文件 |
|---|---|---|
| 丙 快照发布路径 | S1、S2 加原计划的建快照发布、释放判定分流与 deadlist 树、映射条目留存、删快照意图与批、有快照拒回退 | `transaction.rs`、`recovery.rs`、`records.rs`、`mounted_read.rs`、`mount.rs`、`mounted_read_by_offset.rs`、`crates/singlefs-checker/src/walk.rs`（I-9.x、I-3.x）、harness 用例 |
| 丁 GC 发布路径与 discard 调用者 | S3、S4、S5、S6 加原计划的 `publish_relocation_batch`、分配器 R 排除区间与可发谓词、搬迁乙三处、常驻隔离集、停机与触发、discard 产品侧调用者与包装覆盖、G3 落地后扫描重建的搬迁条款 | `transaction.rs`（与丙撞文件——合成一个实现员或串行）、`allocator.rs`、`compaction_plan.rs`、`region_scan.rs`、`block_device.rs`、`mounted_session.rs`、`crates/singlefs-harness/src/memory_pool.rs`、`crates/singlefs-checker-tier/src/crash.rs`（只改注释与 discard 那一支的用例） |
| 己 checker 收尾 | S8、S10 | `crates/singlefs-checker/src/rebuild.rs`（白名单单测）、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crash_judge_conclusions.rs`、checker 档对拍用例 |
| 戊 kb 写回 | S7（C829）、S9、G1 字段表与 `format-const` 标记、greedy 同分与判类读法、discard 第一版取法、S6 的 D13 口径句 | `checks-owed.md`、D26、D8、D13、D5 |

丙与丁都要改 `transaction.rs`：丙先派（快照发布是丁搬共享块那一支的前提），丁在丙交回合入后派，或丁的实现员以丙的补丁为基线。

## 四、复跑与核不动的

- 正推：复跑 `result=same`（第一节）。
- 攻方：`RERUN_ONLY="s2-i113-random-histories s5-region-scan-unpinned"` 在 `~/.local/share/singlefs/work/m4-batch2-code-r1-main-rerun-attack/` 跑完退 0：`name=rerun_summary same=12 differ=0 missing_in_model_out=0`（s2 的随机历史计数与 s5 四处翻转的 core / harness 各一份，12 份输出与腿的 `out/` 逐份相同）。
- 核不动的：攻方 T7「一次 discard 段内部分落地」没有枚举——它要 checker 档的枚举器；正推 T7 的 checker 档探针 `probes/checker_tier_probe_forward_tearable.rs` 没编过——两条都由主 agent 现查 `crash.rs:2154–2166` 的判据代替，第三态的枚举留给丁的崩溃流验。T9 两腿都是读代码推的，主 agent 现查 `crash_judge_conclusions.rs:420` 与对拍用例 `:2607–2608` 坐实逻辑，GPU 对拍真跑归提交时。

## 五、交用户

| # | 题 | 数 | 候选 | 状态 |
|---|---|---|---|---|
| 1 | 扫描重建对谱系记录（C615）与退役记录（C768）怎么读：第一版这两格报不适用，随机历史 1736 态里不适用 776（C615 529、C768 243），删过一次文件之后的 241 态 0 个成立——I-1.13 在删过文件的池上几乎不判 | 攻方 s2（`m4-batch2-code-r1-opus-model/`，复跑见第四节） | 甲：第 4 批给谱系与退役记录各定一条扫描重建读法（回退留下的谱系记录按「不是任何现行版本的成员」认领、退役记录按退役树认领），I-1.13 在这两格真判；乙：第一版维持不适用，记进 C827 等里程碑五 | 开着，岔路单 `m4-batch2-code-r1-forks.md` |

其余改法（S1–S10）主 agent 自己定，不交用户；S6 把 discard 取第三态是按 D13 已定项 4 的字面读，不是新决策。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D5（快照 / 空间记账机制） 已定项 14 | 不影响 | 2026-10-13 不受影响：S1、S2 改的是实现与用例，条目形态不动 |
| D5（快照 / 空间记账机制） 已定项 19 | 支撑 | 2026-10-13 不受影响：S2 把码 14 / 15 计进用户可见角色，照已定项 19 判定 |
| D8（核心索引结构） 已定项 9 | 不影响 | 2026-10-13 不受影响：种类码登记不动 |
| D13（验证路线） 已定项 4 | 支撑 | 2026-10-13 不受影响：S6 按它的字面给 discard 取态，条款不改 |
| D17（实现分层与第三方管道） 已定项 2 | 不影响 | 2026-10-13 不受影响：S7 记欠账，`StepKind` 这一轮不加成员 |
| D26（后台整理与放置回收） 已定项 2 | 支撑 | 2026-10-13 改了：整理意图条目的字段表与四个 format-const 标记写进定案（照 `logged_operations.rs` 的 `to_bytes`） |
| D26（后台整理与放置回收） 已定项 5 | 支撑 | 2026-10-13 改了：S3、S4 的读法写进定案，射程补「另管区域扫描与选段的读法」 |
| D26（后台整理与放置回收） 已定项 7 | 不影响 | 2026-10-13 改了：射程里的现状句按现查改（整理命中 11 份、`discard_at` 产品路径上没有调用者），结论不动 |
| D26（后台整理与放置回收） 已定项 12 | 不影响 | 2026-10-13 改了：射程点名已定项 5 也管区域扫描与选段的读法，结论不动 |
| D26（后台整理与放置回收） 未定项 9 | 备料 | 2026-10-13 改了：第一版取法与 S6 的取态写进去，「段内部分落地」与宿主 / 虚机结局两问留开着 |
