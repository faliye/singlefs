# m4-close-code-r3 判决：收口修补二批第二段的代码轮（主 agent，2026-11-05，第一轮）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T3.1 -->

正文 `research/prompts/_m4-close-code-r3-body.md`，改动全文 `research/prompts/_m4-close-code-r3-diff.md`，背景材料 `research/prompts/_m4-close-code-r3-background.md`。腿：正推（Sonnet）`research/prompts/m4-close-code-r3-sonnet-output.md`，攻方（Opus）`research/prompts/m4-close-code-r3-opus-output.md`（模型 `m4-close-code-r3-opus-model/`）。

被判的产品代码（冻结副本 `~/.local/share/singlefs/work/m4-close-code-r3/repo/`，开工快照 `research/prompts/m4-close-code-r3-snapshot/crates-sha256.txt`）：`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/compaction_plan.rs`。

## 一、核查

- 开工快照：六份与冻结副本 `sha256sum -c` 6 个 OK；腿跑着时主工作区这六份没被这一路改过。复跑前给冻结副本补了不在射程里的 `.claude/heavy-thread-default`（harness 包编译要读它），补完照旧 6 个 OK。
- cite-check 腿报告自报：正推「✓ 核了 11 处引文、71 处裸行号，对上 11 处，没判 71 处」；攻方报告 cite-check 通过。两份都没有对不上的。
- 正文更正：正文第一节写 C946（只坏一份的读没有计数也没有搬迁修复） 在 `.claude/kb/checks-owed.md` 第 595 行，主 agent 现查是第 593 行（第 595 行是 C949）。
- 攻方模型复跑：第一趟（任务 6839）两个目标都编不过「couldn't read `…/.claude/heavy-thread-default`」，是冻结副本缺这一份，补进后重交（任务 6841）：「name=attack_r3_separators_target rc=0 expected=0」「name=attack_r3_counts_target rc=0 expected=0」「name=attack_r3_summary both_targets_as_expected=yes」。观测行整行：
  - 「attack-r3: shape=separator_200 checker_i_9_12=Violated("第 1 条的分隔 key 200 不大于上一个孩子的最大 key 233")」，同一形四个读者 `writable_mount=Ok(())`、`cold_walk=Ok(())`、`read_only_open=Ok(())`、`snapshot_cold_read=Ok(())`；
  - 「attack-r3: shape=separator_235 checker_i_9_12=Violated("第 1 条的分隔 key 235 大于这个孩子的最小 key 234")」，可写挂载、冷走读、快照冷读都拒在 I-9.4「inode 记录号小于分隔 key 或容器号」，「attack-r3: shape=separator_235 read_only_open=Ok(())」；
  - 「attack-r3: damaged=instance_table_first_page_one_copy unit_reads_with_one_copy_failing_while_mounting=2」，对照「attack-r3: damaged=tree_table_one_copy unit_reads_with_one_copy_failing_while_mounting=1」；
  - 「attack-r3: relocated old_slot=one_copy_overwritten read_only_count=1 mount_count=1」（映射里那一份两盘都好），对照「attack-r3: relocated old_slot=untouched read_only_count=0 mount_count=0」；
  - 「attack-r3: failed_whole_read_is_err=true count_after_the_failed_read=0 device_reads_after_the_failed_read=0 unit_one_read_is_ok=true count_after_reading_unit_one=1」。
- 核不动的：无。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| T1 四个读者判 inode 叶交叠 | 打中两处（现量） | ① 四个读者拒因写 I-9.12，实际只比记录号跨叶升序，不判 I-9.12 里「分隔 key 大于上一个孩子的上界」那一半：分隔 key 改成 200，池级 checker 判红，四个读者都收下（任务 6841）；正推旁注同报（「读者只判记录的叶间升序，没判内部条目分隔 key 自己递增」），r2 判决改法甲当时没要这一半。② 只读挂载丢了分隔 key（`mounted_read.rs:509` 写成 `_separator_key`）：分隔 key 改成 235，另三个读者拒在 I-9.4，只读挂载收下，与两处代码注释「四个读者一个结局」不符。正推判四处调同一个 `recovery.rs:6010`、核心层没有第五个读者，站得住 |
| T2 只坏一份的计数 | 打中三处（现量）、两处推的 | ① 实例表第 0 片坏一份，一次可写挂载经计数层读两遍、计 2（`recovery.rs:3598` 与 `mount.rs:1341` 起的链读），与 `mount.rs:9414`「不多数」那句注释矛盾。② 整理搬走单元后盘上的提示还指旧槽，旧槽一块盘上那一份被盖掉，映射位置上两份都好，只读 `read_at` 与可写挂载照样各计 1 次。③ `read_at` 一次调用中途报错，本次观测整份丢掉，已读到的只坏一份不进累计（`mounted_read.rs:1043`，正推旁注同报）。推的：用户改动被空间拒推抬 F 再试，每一轮重读旧单元再计一次（正推 (a)，`mounted_session.rs:259`「被拒的也算」），而预演不计，两处理由互相矛盾（攻方同向）；挂载退版先建退到的那一版、失败被 `.ok()` 吞掉、计数不清零（正推 (b)，`mount.rs:9442`）。正推判落点、两份都坏不计、不加盘上字节、重开从 0 起都对，站得住 |
| T2 的条款落差 | 交用户 | 正推 (c) 与攻方同报：D4（校验和位置） 已定项 4 定案（`.claude/kb/decisions/04-校验和位置.md` 第 71 行）「读到只坏一份时计数，并经已有的搬迁路径把这个单元整个搬到新槽」不分读路径；代码只在部分写、截断读回（与快照下另发那批）搬；只读挂载只计数是用户 2026-11-04 原话「只读挂载只计数，不修」；冷恢复读、挂载时建上一版、可写挂载时读到的只计数不搬，是主 agent 认下实现员的提案（`records/2026-10-13-里程碑四预开发.md` 那几天的行），不是用户原话，代码注释 `mount.rs:7658` 自己写了交主 agent |
| T3 另发一批搬迁与被拒推抬 F | 没打中（只抽一次） | 正推判成立：同一事务层、同一预算函数 `push_one_floor_raise_within_the_admission_budget`、`current` 换到推到的那一版、三个开关生产路径无调用者、排空被拒那一支没动。攻方只报一处诊断上的出入：推满时不报「被宽护卡住」那一项，与走空那一套（`mounted_session.rs:5032`）不一致。正推另提：用户原话「推抬 F 再试一遍」、代码是循环到推满，与 D8（核心索引结构） 已定项 16 第 6 条会话走空同一写法 |
| T4 `publish_the_change_once` 的挪动 | 没打中（只抽一次） | 正推：新旧 `match` 整段逐字相同；攻方：挪动的那段没有 hunk，`crates/mutations.tsv` 落在 `mounted_session.rs` 的锚点逐个恰好命中 1 次（它当场数的，提交时由门禁 code-source-discipline 的 mutation-tables 格判） |

## 三、交给下一批的实现员（与这一批原有的两处红合成一簇）

腿提的改法都只在腿自己的模型上推过，被攻过零轮。攻方的两份模型用例今天断言的是现状，修完把断言翻过来就是必红用例。

1. **T1①**：四个读者（`recovery.rs:3833`、`:7412`、`:7597` 与 `mounted_read.rs:575` 那一处）照 I-9.12 另判「每条内部条目的分隔 key 大于上一个孩子的最大 key」，与池级 checker 同一结局；验收：攻方 `separator_200` 那一形四个读者都拒。
2. **T1②**：只读挂载用上分隔 key（`mounted_read.rs:509`），与另三个读者同一判法；验收：`separator_235` 那一形只读挂载也拒在同一不变量。
3. **T2①**：同一次挂载里同一个单元的同一份坏副本只计一次（实例表链被读两遍那一形）；验收：实例表第 0 片坏一份计 1。
4. **T2②**：只坏一份按这个单元此刻的权威落点判，盘上旧提示指的旧槽读坏不计；验收：搬走之后旧槽被盖掉那一形计 0。
5. **T2③**：`read_at` 中途报错时已读到的只坏一份照样进累计（或写明不进、与写入口「被拒也算」统一成一种口径）；验收用攻方那一形。
6. **T2 推的两处**：推抬 F 重试时每轮重读旧单元重复计数、挂载退版计数不清零——先写会红的用例坐实，坐实就改成每次用户改动、每次挂载各只计一次。
7. **原有两处红**：GPU 恢复内核 `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` 读 inode 叶那一段补与 CPU 同样的叶序判法（连同第 1、2 条新加的分隔 key 判法，GPU 与 CPU 同一结局；对拍 `the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order` 转绿）；`crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs:167` 去掉可省的显式生命周期。

每处改法一条变异行。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| D4（校验和位置） 已定项 4 字面要求读到只坏一份就搬，不分读路径；现在只有部分写、截断读回与快照下另发那批会搬。只读挂载只计数是用户定的；冷恢复读（不挂载、没有写者）、挂载时建上一版那一读、可写挂载期间别的读到只坏一份，要不要搬 | 第二节「T2 的条款落差」 | 零轮 |

## 五、后续

- T3、T4 两格「没打中」只抽一次，不拿它支撑结论；下一批改完之后的代码轮里照攻。
- kb 写回（D4（校验和位置） 已定项 4 的两条用户读法、C946（只坏一份的读没有计数也没有搬迁修复） 改成现状、准入设计轮的结论）随这一批收尾一起写。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D4（校验和位置） 已定项 4 | 不影响 | 2026-11-05 不受影响：判的是代码对不对得上条款与用户定案；读路径之间搬不搬的落差交用户，条款这一轮不改 |
