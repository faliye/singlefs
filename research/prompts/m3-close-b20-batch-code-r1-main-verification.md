# m3-close-b20-batch-code-r1：判决（2026-11-10）

正文 `research/prompts/_m3-close-b20-batch-code-r1-body.md`，开工快照 `research/prompts/m3-close-b20-batch-code-r1-snapshot/`。这是第 20 批修补之后七份产品补丁的第一轮代码轮；上一轮判决 `research/prompts/m3-close-b20-repairs-code-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m3-close-b20-batch-code-r1-sonnet-output.md` | 六件「都对得上，没有冲突」；检查单「带 --item」不成立（轻）；走不到的理由附一处不实 |
| 攻方（Opus） | `research/prompts/m3-close-b20-batch-code-r1-opus-output.md`，模型 `research/prompts/m3-close-b20-batch-code-r1-opus-model/` | 打中 A1、A2（读码推的，待主 agent 跑）；其余面没打中 |

- 快照：腿跑着的时候与写判决之前，主 agent 两次 `cat research/prompts/m3-close-b20-batch-code-r1-snapshot/*.txt | sha256sum -c`，七份都 OK，树就是开工那一刻的树，没跑 `snapshot-tree.py`，`cite-check.py` 的 `--root` 直接给仓根。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b20-batch-code-r1-sonnet-output.md research/prompts/m3-close-b20-batch-code-r1-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b20-batch-code-r1-background.md`，「✓ 核了 24 处引文、106 处裸行号，对上 23 处，没判 107 处」。没判的两处是正推腿第 174、175 行只写 `mount.rs:1496`、`mount.rs:9922`（仓里三份同名），主 agent 现读 `crates/singlefs-core/src/mount.rs` 第 1496 行 `.max(chosen_root.checkpoint_txg);`、第 9922 行 `let root_chosen_by_the_read_stage = chosen_root;`，对得上。
- 材料缺口：`research/prompts/_m3-close-b20-batch-code-r1-diff.md` 里读阶段根全集那一份是空围栏（那份 `crates.patch` 没有 `diff --git` 行，主 agent 摘 diff 只认这一行）。两条腿都报了、都改读原补丁 `~/.local/share/singlefs/work/m3-close-b20-read-stage-roots/patch/crates.patch`。材料在这一轮里不改。
- 攻方用例复跑：交接副本按 `scripts/hosts/minimal-copy.py` 从主工作区重建（快照对上，内容与腿开工时相同），任务 12392 退 0，日志 `task-b0f6b921-12392.attempt-3.log`：`ATTACK_UNMOUNT_CELL` 17 行、`ATTACK_ABANDONED_SETUP` 1 行、`ATTACK_ABANDONED_CELL` 13 行，末行 `name=rerun_exits cases=0`。

## 二、判决

| 格 | 判 | 依据 |
|---|---|---|
| A1 判退之后 R 的根槽在重建那一遍读坏一次，正常卸载被拒 | **打中（量的）**。违 `.claude/kb/decisions/23-journal的角色与格式.md` 第 393 行「被抛弃根的账（树表或分配记录树）读不出时不跳过」那一段管的正常卸载 | 12392 对照行 `ATTACK_UNMOUNT_CELL control … valid_root_recorded_on_the_slot_of_r=Some(CheckpointTxg(9)) … borrow_of_c_for_r=Some(150) unmount=raised`；`k=7` 行 `valid_root_recorded_on_the_slot_of_r=None slot_of_r_recorded_bad_or_all_zero=true c_holds_the_slot_of_r=true borrow_of_c_for_r=None unmount=refused:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot …` |
| A2 被抛弃根读阶段读到、重建那一遍读坏一次，影子账不护它 | **打中（量的），比腿报的重**：腿报「这条根独占的槽不隔离」；量到的是那一格隔离 0、这一次挂载在 R 还在根环里时写掉 R 的 14 个单元。C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一机理的子形 | 12392 对照行 `ATTACK_ABANDONED_CELL control … slot_of_r_recorded_as_abandoned=true isolated=24 … units_of_r_overwritten_by_the_mount=0`；`k=5` 行 `slot_of_r_recorded_as_abandoned=false isolated=0 abandoned_roots_unreadable=0 units_of_r_overwritten_by_the_mount=14 overwrites_done=1 first_overwrite_writing_r_while_r_in_the_ring=Some(1)` |
| 六件「代码做的是不是条款说的」（正推第一到六节） | **成立**（读码，两条腿互不冲突）；判退之后下界收读阶段的根、判退用 R 两件的条款原文没点名判退那一形，是推出来的 | 正推报告；主 agent 现读第 1496、9922 行 |
| 去重键、删现择、拒因字段、H6 别的盘面、H3 新旧入口（攻方面一之二到面五） | **没打中**，一次抽样，不支撑「没有问题」；试过的形逐条在攻方报告「没打中的形状」 | 攻方报告 |
| `recovery.rs` 第 2063 行文档「之后那几遍读坏的根不因此漏掉」 | **不实**（A2 那一形漏了），跟 A2 一起改 | 攻方报告检查单第 2 条 |
| H6 借的拒绝成员 | **轻**：「第 0 代之后读得到更新的东西」报 `TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`，这个枚举按管理员下一步分成员（`system_configuration_repair.rs:142`），这一形管理员该做的是换断言 | 攻方报告第四节 |
| 检查单：带 --item | **不成立（轻）**：三份实现员报告写 `--item <项>` 占位；八行变异行锚点各命中一次（两腿都量了） | 两份报告 |
| 检查单：走不到的理由 | **成立**，附一处不实：`m3-close-b20-repairs-fixes-2/result.md` 第 69 行「只有单元读不出才到得了」，零单元发布也到得了 | 正推报告 |
| 检查单：多机可用 | **成立**；正文「新用例都用内存盘」不实，实例表那条新用例用 temp_dir 下的镜像文件 | 攻方报告 |
| 检查单：不兼容旧代码 | **推不出**：`recover`、`mount_read_only` 旧入口产品里 0 个调用方，验证包 390 处在用；D22 已定项 8 射程把它们写成现存入口 | 两份报告 |
| 检查单其余四格 | 成立 | 两份报告 |

## 三、改法

| 改法 | 修哪一格 | 状态 |
|---|---|---|
| G1：读阶段择根那一遍连槽一起留；重建那一遍某槽读坏或全零、读阶段在那一槽读出过自证根的，根环表照 T1 甲那一类记（`RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`，带读阶段那一份整条根记录），影子账与回收门槛照「并出来的照读出来的算」 | A1、A2 | **零轮**（攻方提的，只在它的模型上想过）。写进产品代码之前先在探针里做 A/B：攻方两条用例在有 G1 与没有 G1 两臂上跑，量 A1 的 `k=7` 卸载做不做得成、A2 的 `k=5` 隔离数与写掉的单元数；A/B 读数：12487（不带 G1，与 12392 逐格相同）、12488（带 G1：A1 `k=7` `unmount=raised`、`borrow_of_c_for_r=Some(150)`；A2 `k=5` `isolated=24`、`units_of_r_overwritten_by_the_mount=0`；其余 28 格两臂逐格相同）；探针 12489（不带 G1）、12490（带 G1）：管理员回退 `from_k=2`–`7` 六格由 `rolled_back` 变 `refused:CandidateJudgementStillUnreadableAfterOneReread`，与 `from_k=8` 起两臂同一结局，P1 `k=5` 可写挂载由做成变 `AccountOfAnAbandonedRoot` 拒；带 G1 副本核心包 mount、recovery 单测 12495 `75 passed; 0 failed`。用户 2026-11-10 弹窗定读阶段读出过自证根的槽算「挂载那一刻读得出」（选「算，照 G1 写」），改法照写 |
| G2：只给影子账加读阶段那一份 | A2 | 零轮；攻方推它修不了 A1，不选 |
| G3：H2 借 C 那一臂放宽到「那一槽记成读坏或全零而 C 在那一槽有条目」 | A1 | 零轮；放宽 H2 字面，攻方不推荐，不选 |
| `recovery.rs` 第 2063 行文档改实 | 文档 | 随 G1 |
| H6 借的拒绝成员：这一形换成按「换一个断言的版本」报的成员 | 轻 | 并进同一批实现员 |
| 测试注释第 367 行过期、三份实现员报告的 `--item` 占位 | 轻 | 测试注释并进同一批；`--item` 由主 agent 照 12397 列出的项名补交 |

G1 没写进主工作区：用户 2026-11-10 叫停「那就停下 G1 不再继续」，A1、A2 与 G1 补丁冻结进 `.claude/kb/checks-owed.md` C1012（读阶段读出、重建那一遍读坏的根槽不并进根环表与影子账），去向「未排」；这一轮被判的四份产品代码在主工作区仍是开工快照那一份。

A1、A2 都不分辨「这一批改没改」（攻方报告末节），分辨的是「读阶段那一份只给 C」与「也给根环表、影子账」；改法归第 20 批。

## 交用户的

无：G1 是零轮改法，按规矩先做 A/B，不交岔路；H2 的字面不改（G3 不选）。

## 这一轮被判过的文件

- crates/singlefs-core/src/recovery.rs：`recover_of_the_pool` 收本池 fsid（H3）成立；`choose_root_keeping_every_self_verified_root_read` 交回全部自证根成立，第 2063 行文档对被抛弃根不实（A2）
- crates/singlefs-core/src/system_configuration_repair.rs：断言第 0 代时 tail 取 mkfs 写的值、环里有更新的根或记录就拒（H6），七种盘面成立；借的拒绝成员名不贴这一形（轻）
- crates/singlefs-core/src/mount.rs：H1 取 max 收 `chosen_root`、判退之后下界、并读阶段全部根按 (实例代号, checkpoint_txg) 去重、实例表照读阶段择到的根取、拒因字段不可为空，与条款对得上；读阶段那一份只给 C、不给根环表与影子账，A1、A2 打中
- crates/singlefs-core/src/mounted_read.rs：`mount_read_only_of_the_pool` 收本池 fsid（H3）成立，旧入口去留推不出

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-11-10 不受影响：第 393 行条款不动，A1、A2 是代码没做到它，改代码 |
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-11-10 不受影响：H3、H6 射程与代码对得上 |
