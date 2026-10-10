# 第 34 批代码轮第一轮判决（2026-10-06）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 Q1 Q2 Q3 Q4 RV B4 FL FS M1 M2 M3 M4 ALT1 -->

被判的文件（第 34 批三份补丁、34-4 与主 agent 改的钉值，逐份见改动全文 `research/prompts/_m3-formal-code-b34-r1-diff.md`）：
- 34-1：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs`；
- 34-2：`crates/singlefs-core/src/root_record.rs`、`crates/singlefs-core/src/pointer.rs`、`crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs`、`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`；
- 34-3：`crates/singlefs-harness/src/model.rs`；
- 34-4 与各份追加：`crates/mutations.tsv`；
- 主 agent：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs`（#9 三个钉值）。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b34-r1-body.md`、`_m3-formal-code-b34-r1-diff.md`、`_m3-formal-code-b34-r1-background.md`（第一版 1.45 MB 挪到会话草稿 `b34-materials-v1/`，裁成 111 KB） | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b34-r1/tree/`（第 34 批全部合入后），清单同目录 315 行；kb 快照 `research/prompts/m3-formal-code-b34-r1-snapshot/kb-sha256.txt` | 腿交回后核：冻结副本 315 行全 OK，kb 快照全 OK；之后根环线对 `.claude/kb/invariants.md` 的 I-3.1 那一行做过一次定点替换（行数不变），两腿没引那一行 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b34-r1-sonnet-output.md`（sha256 b31b7b79…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b34-r1-opus-output.md`（sha256 a9b621cf…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 复跑（第四节） |
| 引文 | 两份腿报告 | 主 agent 合跑 cite-check（`--root` 冻结副本与仓）：48 / 48 |

## 二、逐格判决

### P1　RV

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| P1-a 合法历史多报 | 没打中：106 个前缀状态 I-5.4 全成立，其中 12 个走到下一次挂载先施加的那一版、0 个走到由记录施加出来的那一版 | 没打中：随机历史 430 段 35 989 张、层 0 六段 221 354 个状态，I-5.4 违例 0、部分没判 0；新旧之差只有「不适用 → 成立」 | **站住**。两腿立场不同、各抽一次，按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不记成「没打中」的结论；站住靠攻方的正推（两类版本被认出时单元还没到回收，`crates/singlefs-checker/src/walk.rs:7741`）与提交时的层 0 快档 |
| P1-b 新根段取自哪条记录 | 一致：偏移 `0..86`、`274..360` 与 D23（journal 的角色与格式） 新根段对上；M1–M4 四种取错字段都被新用例抓 | **打中**：checker 取同一次发布里先到的那条带提交标记的记录，core 施加带末条标志的那条；第一条换成上一版新根段（s3）、再叠两条记录罩住同一个槽，新 checker 0 红、两种挂载都拒 | **打中坐实**。两腿分歧点是「坏写者让同一次发布的几条记录新根段不同」这一格，正推按已定项 15「每条记录都带同一份新根段」没造这一形。主 agent 现查：checker 取先到的那条（`crates/singlefs-checker/src/walk.rs:7749`「.or_insert_with(\|\| record.new_root_segment.clone());」、`walk.rs:5519` 起的 `.find`），core 取末条（`crates/singlefs-core/src/recovery.rs:3658`「每条记录都带整次发布的新根段（已定项 15），施加末条那一份」）；攻方 crafted 段复跑 195 行与腿逐字相同。改法 FL（两类版本都取带末条标志那条的新根段）进第 35 批，CPU 与 GPU 同改，被攻过零轮；FS（同 (实例, txg) 每条记录新根段逐字节相同）没有判它的不变量，记欠账 |
| P1-c GPU 同组同判法 | 一致（读码，推的） | 没打中（读码，推的）；GPU 取「数组里第一条」，环绕之后可能不是键序第一条 | 站住（推的）；FL 同改 GPU 两处（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3768` 起的同键取先到、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1071` 那个函数），攻方那句环绕顺带消掉 |
| P1-d 条款与实现 | 条款没写：由记录施加出来的那一版还要不低于下界、单元未到回收、进得了重放前缀三个条件 | — | 写回批：I-5.4 那一行补这三条 |

### P2　全零树表指针（用户定甲）

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| P2-a 合法历史误红 | 没打中：core 里写全零树表指针的四处都在测试模块里 | 没打中：6 702 张合法镜像上全零的根与记录 0 条 | 站住 |
| P2-b 整版放过 | 下一次挂载先施加的那一版全零：checker 0 红、core 拒；由记录施加出来的那一版全零：红 I-3.1、I-3.5，不红「损坏」 | 同（z5、z6） | 两腿同判，归 Q1 交用户；不是第 34 批引入 |
| P2-c 「自证过的根」同一集合 | 不是（表） | 不是（表） | 两腿同判。条款 `.claude/kb/invariants.md` I-9.28、I-9.5 只说「根」，实现里五处另收由记录造出来的版本；写回批写明各处的集合，处置归 Q1 |
| P2-d 与 core 同结局 | 同方向、不同结局：最新根全零 core 拒挂载，不退回上一条完好的根 | 同：core 读到那条根的每条路上两边同向（z2 回退到 txg 4 时 core 拒） | 不矛盾：正推说的是 core 不退回，那是用户 2026-10-05 没选乙（core 当自证不过）的后果，照用户定案不改 |
| P2-e I-9.28 部分没判的说明 | — | 打中（文字）：前缀写「树表指针非零而条目没读全」，单子里却有「树表指针全零」的根 | **坐实**：`crates/singlefs-checker/src/walk.rs:5178`。第 35 批只改说明。攻方另报被抛弃的根全零时 I-7.4 的说明写的是「单元已被重新分配或抹头」（读码，推的）；第 35 批同一实现员核实后一并改说明 |

### P3　34-2 根记录四条指针

两腿同判站住：core 读根槽的入口都经 `RootRecord::parse_slot`（`crates/singlefs-core/src/root_record.rs:128`），`read_judging_the_extent_offset_from` 在冻结副本里 0 次；两份改了的测试仍判原来那件（正推在副本上施加 `mutations.tsv` 两条新变异，红在点名的断言上）。正推另记：`pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs` 那条单测现在比的是同一实现的两个入口，根记录那一路的判别力在同文件与新文件的用例上——照记，不改。GPU 恢复内核根记录那一判（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:928`）仍只判 extent 偏移，已知，进第 35 批。

### P4　条款没写的四问

两腿各给候选与代价，都判「不挡这一批合入」；用户 2026-10-06 定 Q1 甲、Q2 乙、Q3 甲、Q4 甲（第五节）。正推另报：今天没有任何测试钉住主走读对由记录施加出来的那一版的 `NoTreeTable` 那一支，候选之间换哪个都不会有现成用例变红——用户定了之后照定的那一支补用例与变异。

### P5　I-5.5 专门理由、B4、#9、变异行

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| I-5.5 专门理由 | 分得出「没有对象」与「一次都没判」；「认出来而没有数据单元」那一形分不出（Q4） | 同；6 702 张上缺省理由 0 次 | 站住；那一形归 Q4 |
| B4 | 补的点按判决做对了；片数落在 7..59、6..99 内部或 101..134 内部的窗口变体存活（`.claude/rules/mutation-sampling.md` 第三类） | 没攻 | 第 35 批再补取样点，让正推列的三个存活窗口都红（例 30、120），每个窗口变体在副本上证红 |
| #9 钉值 | 注释两处来历并存（829 行与 847 行）；钉值取自 34-1 之前（推的，旁证：RV 让新池新建文件 58 个前缀状态里 4 个 I-5.4 从不适用变成成立） | 没攻 | 钉值删除线第 24 批合入后已在主工作区重取（`crash_enumeration_sharded_across_processes.rs` 847–849 行），829 行那句第 35 批改成「重取经过见下」 |
| 变异行 | 36 行锚点唯一、harness 档 34 行全红、点名的测试都跑到；`mutations.tsv` 955、965 两行红在 `allocation_roots[0]` 越界 panic 上；不越界的替代 ALT1 红在断言上 | 61 行静态核 bad 0 | 站住；955、965 两行第 35 批换成 ALT1 形（不是 34-4 引入，替换文与 HEAD 一字相同） |

### 越格线索（不归这一轮）

攻方在合法随机历史上量到新旧两份 checker 同样红 I-3.1（种子 187957262、187957475、187957674）与 I-3.9（种子 187957506），都在带崩溃重挂、并进遍历的由记录施加出来的版本 ≥ 1 的镜像上；复现命令在攻方报告「越格线索」一节。像 C697（已分配统计在短环加大文件上误红）、C560（满环时根槽写坏的恢复前镜像上记账多于遍历） 那一族，没核是不是同一形（推的）。去向第三节第 4 条。

## 三、去向

1. **第 35 批实现员**（零轮）：
   - FL：`crates/singlefs-checker/src/walk.rs` 两类版本（由记录施加出来的那几版、下一次挂载先施加的那一版）取带末条标志那条记录的新根段，下一次挂载那一版要求末条已落；GPU `crash_judge_checker.wgsl` 拼 `R_DATA` 那一段与 `crash_judge_checker_scan.wgsl` 的 `record_index_of_the_version_the_next_mount_applies_first` 同改；攻方 s3 那一形进 `checker_known_bad_images.rs`，配变异。
   - `walk.rs:5178` I-9.28 部分没判的说明改写；被抛弃的根全零时 I-7.4 的说明核实后改写。
   - GPU 恢复内核 `crash_judge_common.wgsl:928` 根记录指针那一判改成与 core 同判（34-2 那四类字段）。
   - B4 再补取样点；`mutations.tsv` 955、965 两行换成 ALT1 形；`crash_enumeration_sharded_across_processes.rs` 829 行注释。
   - Q1 甲：主走读对由记录施加出来的那一版树表指针全零照 core 跟过去（`walk.rs` 的 `NoTreeTable` 那一支），下一次挂载先施加的那一版全零另判损坏；两形坏镜像进 `checker_known_bad_images.rs`，配变异。
   - Q3 甲：GPU 走读内核对候选根全零树表指针同判（`crash_judge_checker.wgsl` 的 `w_walk_tree_table_and_mapping`），加一条造这种根的对拍流与一行变异。
   - Q4 甲：I-5.5「认出了那一版但没有数据单元」另写一句不适用理由，配用例与变异。
   - 新池新建文件崩溃枚举用例里 I-5.4 计数那一类（第 34 批之前记下的）。
2. **欠账**：FS 那一格记进 C738（同一次发布几条记录的新根段一致没有不变量判）。
3. **写回批**：I-5.4 那一行补由记录施加出来的那一版的三个条件；I-9.28、I-9.5 与 I-5.4 / I-5.5 写明各处「自证过的根」的集合与 Q1 甲；I-9.28 那一行「CPU 红、GPU 不红」随第 35 批 Q3 甲合入改；I-5.5 那一行「认出来而没有数据单元那一形仍落缺省理由」随 Q4 甲改；Q2 乙不改条款。
4. **越格线索**：发给根环线（C697、C560 所在的那一线）带种子与复现命令，不在形式化线修。

## 四、复跑核对

正推：模型 `tools/rerun.sh setup build tests rows custom b4 probes patches` 在会话草稿 `b34-sonnet-rerun/` 跑，退 0（第一趟 34 行变异、B4、custom2、探针与补丁全退 251——内存包装拒了，8G 超过峰值表两倍没给理由，命令一行没跑——挪进同目录 `first-try/`，第二趟带理由重跑）。编译日志三个被测包各有 `Compiling`。34 行变异逐行比 `exit`、`named_test_failed`、`kind` 与首个 panic 前 80 字：0 行不同；custom1 / custom1b（M1–M4）与 custom2（ALT1）每条的退出码与红的用例集合相同；B4 八个窗口去掉耗时后逐行相同；探针与补丁的 `PROBE` 行 51 行，只差腿那一趟截断得更短；补丁 q1a、q2、q4 下的 `test result` 行与 `results/patches.out` 同（q2：119 过 2 败）。

攻方：模型 `tools/rerun.sh` 不给段名全跑，在会话草稿 `b34-opus-rerun/` 退 0。`crafted` 段 `B34X` 195 行排序后与 `out/crafted.txt` 逐字相同；随机历史五批（`B34R`）与层 0 各段（`B34L0T`）逐字段比，腿留存的每个字段都相同（seed 57 腿一趟 65 548，复跑分两段 33 000 + 32 548，两段 `new_red={}`、`differences={}`）；四行 `B34R-RED-FIRST` 与腿的种子、步、动作逐行同。

核不动：GPU 对拍与 GPU 变异行（`crates/mutations.tsv` 2855、2644 行）、层 0 快档、#9 钉值，归提交时。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | Q1 由记录造出来的两类版本里树表指针全零，要不要也判损坏 | 两腿 | 正推副本量过候选「由记录施加出来的那一版照 core 跟过去」（补丁 q1a，一行，坏镜像 121 条加探针 4 条全过） | **用户 2026-10-06 弹窗定甲**（选项原文「甲 两类都判损坏」）：第 35 批 |
| 2 | Q2 全零树表指针要不要同时红 I-2.5 | 两腿 | 正推副本量过（补丁 q2，3 行，两条全零用例期望多一个 I-2.5） | **用户 2026-10-06 弹窗定乙**（选项原文「乙 照今天，不红 I-2.5」）：不改 |
| 3 | Q3 GPU 判器要不要对全零树表指针同改 | 两腿 | 读码，推的 | **用户 2026-10-06 弹窗定甲**（选项原文「甲 同改」）：第 35 批 |
| 4 | Q4 I-5.5「认出了那一版但没有数据单元」要不要另写理由 | 两腿 | 正推副本量过（补丁 q4，3 行，没有测试依赖缺省理由）；攻方 6 702 张上这一形 0 次 | **用户 2026-10-06 弹窗定甲**（选项原文「甲 另写一句」）：第 35 批 |
| 5 | FL（P1-b 改法） | 攻方 | 攻方副本量过：s3 上红 I-5.4，4 492 张合法镜像上与今天逐条相同 | 第 35 批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 15 | 备料 | 2026-10-06 不受影响：FL 让 checker 与 core 同取末条，已定项 15「每条记录都带同一份新根段」不变；违反它的写者今天没有判据，记欠账 |
| D19（块指针的结构与宽度预算） 已定项 3 | 不影响 | 2026-10-06 不受影响：34-2 把头部判法铺到根记录四条指针，字段表不变 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-06 不受影响：RV 与全零树表指针那几处不碰回收门槛与回退候选 |
