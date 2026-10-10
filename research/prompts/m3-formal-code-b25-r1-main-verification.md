# 第 25 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers K1 K2 K3 V4 V9 V11 V17 V18 V19 S1 S13 Q1 Q4 Q5 KX1 KX2 KX4 KM1 KM2 KM7 T01 T02 T13 -->

被判的文件：`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs`（25-1）；`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`（整份，25-3a 与 25-3 的改动连同之前那一版一起点名）、`crates/singlefs-checker-tier/Cargo.toml`、`Cargo.lock`（25-3a）；`crates/mutations.tsv` 里「O1 跨叶容器」3 行与「E171R1 装置」28 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b25-r1-body.md`、`_m3-formal-code-b25-r1-diff.md`、`_m3-formal-code-b25-r1-background.md` | — |
| 冻结副本与快照 | E171 第一段冻结树 `research/target/e171-r1-frozen-tree/`（摘要 4d071c12…）；crates 清单 `/tmp/claude-1000/m3-formal-code-b25-r1/crates-sha256.txt`（291 行）；kb 快照 `research/prompts/m3-formal-code-b25-r1-snapshot/kb-sha256.txt`（2 份） | 写判决前 `sha256sum -c`：冻结树对清单 0 处不符，kb 两份 OK |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b25-r1-sonnet-output.md`（sha256 e74815ed…），模型 `…-sonnet-model/` | 主 agent 照它的 `tools/rerun.sh setup k1 k2` 在会话草稿复跑，见第四节 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b25-r1-opus-output.md`（sha256 3b71b123…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh` 七段全跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结树> --root . --background <背景材料>`：122 处引文对上 122 处，没判 0 处 |

## 二、逐格判决

### K1　25-1：O1 跨叶容器

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 模型 ⌊k ÷ 233⌋ 与 core 切叶是不是同一规则 | 一致：core 纯函数逐条追加 31455 条，每条重写的片都是 ⌊k ÷ 233⌋，第 31456 条拒，模型式子在同一处 panic | 没打中：只固定前缀、用户动作放开扫，1296 段加定向 26 段全 `Completed` | **站住**，两次独立抽样都没打中（复跑 `rows=1296 not_completed=0`） |
| 第二片之后管理员回退那一支 | 「规则没说，可达历史上不执行」：读码以为建过 inode 的版本模型拒回退 | 打中：删回退那一支两行中的任一行，25-1 新用例照绿，T01、T02、T03、T13 转 `NewFinding` | **攻方坐实**（复跑 `drop-retain` 红 T01、T03、T13，`drop-extend` 红 T02、T13，`none` 四段全 `Completed`）。分辨两腿的事实：`answer_rollback_while_mounted` 的前提 `is_a_version_with_a_file_or_a_file_being_deleted` 把 `ModelVersionContents::InodesWithoutTheFirstFile` 也算进可回退（冻结树 `crates/singlefs-harness/src/model.rs:476-485`），正推读漏了这一个成员；正推自己的 KX4（删 `retain`）在新用例上不红与此一致。改法：加一条 T13 形状的 harness 档用例（`U`、建 468 个、回退、再建 3 个）与两行变异，零轮 |
| 新用例只到第 2 片 | 打中：KX1（胶水第 3 片起都当第 2 片）、KX2（模型落点封顶在第 2 片）在新用例上绿，把常量换成 2 × 233 + 4（470 步）后两条都红 | 没量这一格 | **坐实**（复跑同）。照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」第三行补敏感取样点：新用例常量改成 2 × 233 + 4，KX1、KX2 进变异表，零轮 |
| 超过 135 片：模型 panic、实现拒 | 不让模型答拒、保留 `assert!`、加纯函数两侧的边界用例（0.65 秒）、记账 | 那一支走不到：任何时间线约第 1277 次发布时模型在 `model.rs:1562` 加法溢出 panic（C682 同一个病，上界每次发布约乘 1.029）；建议记账、并进 C682 | **两腿都对，取并集**：保留 `assert!`、不答拒（条款没规定拒，D8（核心索引结构） 已定项 6 写的是根装满时长一层）；加正推的边界用例钉「两边的 135 是同一个 135」；C682 补攻方量到的后果（复跑 `k1-long` 四段都在 `Operation(1276)` / `Operation(1277)` panic，`k1-bound` 上界逐行同）。模型实际罩到约 1276 次建 inode，不是三万多次 |
| 胶水认错叶容器编号 | 被罩历史上一致；占槽跨度那一维这条用例整个不敏感（KX7、KX8 都绿，不是 25-1 新出的洞） | 位置序号没打中（推的）；容器号写错 O1 与新用例都放过，`many_inodes_write_and_read` 红 2 条 | 站住：容器号归 `many_inodes_write_and_read` 罩（复跑 `FAILED. 2 passed; 2 failed`），不归 O1 |
| 3 行变异 | 各红点名那条（第 235 步分配代分歧），另 110 条 harness 档测试零红 | 推的 | 站住：三行各红点名那条复跑同；「另 110 条零红」那一趟复跑在负载下超了脚本自己的 230 秒限时、没跑完，按「没复跑出」记 |

### K2　E171 装置：Q5 候选二

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 同类而判定不同的一对 | 没找到：真流上 4896 个附加态与类推出的 0 处不同，抹掉类键报 936 | 没找到：对拍截到 k = 6，4608 态每类四样取值唯一 | **没打中，两次独立抽样**（复跑两边都同） |
| 两个崩溃子列只吃四样输入 | 「规则没说」：登记只主张 O1 两列、RefFS、恢复 | 打中：判态那条路上 O2、不看 journal 的那遍恢复、`check_records` 任一 panic，两列都改写成恢复 panic（第五样输入），类核对 ③ 只记看 journal 那遍的读；注入「第 43 号写落、第 44 号没落时 O2 panic」，k = 2 的对拍与 ①–④ 全 0、q5 行与不注入时逐字同，k = 6 的对拍判出 1152 态不同 | **攻方坐实**（复跑 `fault-k2 rc=0` 全 0、`fault-k6 rc=3 mismatching_states=1152`）。这是方法上的洞：今天真实代码上两腿 9504 个态里 O2 都是 `Some([])`、没 panic，不是「已知某态判错」。改法取甲：判态路上把 O2、`check_records`、不看 journal 那遍恢复的 panic 各自接住、单列，不改写 O1 两列（四样输入的说法这才成立）；加丙：对拍截到 k = 6（复跑 41–55 秒量级）。零轮，要登记修订 |
| 对拍罩到哪几个写 | 只罩 C 的前 2 个写 | 前 2 个写是同一个单元在两块盘上的镜像（第 41、42 号写同偏移） | 坐实（复跑 `OPUSWRITE` 那几行同），并进上一格的丙 |
| 类核对的告警会不会红 | 打中：①②④、落地串核对、S13 拼法、journal 提示整环七处（KM1–KM7）关掉之后，装置跑法与内联用例口径的复刻输出不变；V17、V18 在装置跑法里读不出、只靠 checker 档内联单测 | 没量 | **坐实**（复跑 KM2–KM7 `stdout_same_as_unmutated=True`；KM1 在正推另造的 D2 流上才有 2 行不同；V17、V18 在主取样点 `stdout_same_as_unmutated=True`）。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」：每一处告警各配一条会红的内联用例与一行变异 |
| `q5_not_run=no_root_write` | 一致（没验，没看到对拍排在前面） | 打中：对拍先跑、不查根写，没有根写时逐态越界 panic，走 `s13=equivalence_pairing`，认定第 6 条要的出口到不了 | **攻方坐实**（复跑 `noroot rc=3 … s13=equivalence_pairing q5_not_run=equivalence_pairing_failed`）。改法：对拍之前先看主取样点那一形的 `judged_root_index`。今天的流上有根写（第 38 号写），产物不受影响 |
| S13 与 `class_landing_mismatches` 计法 | 计法一致；两处与登记字面不同：登记把它释成「恢复落点」，代码数的是落地串；`q5_equivalence` 行仍带 `refinements` 等字段，与认定第 3 条「另起 `q5_partition` 行」不同 | 没打中 | 站住计法；两处字面差改：登记修订里把释义改成落地串，`refinements`、`moved_last_segment_positions`、`stopped_at_the_state_limit` 挪进 `q5_partition` 行（认定第 3 条在先） |

### K3　E171 装置：其余改动

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| RefFS 改正式依赖的方向 | 一致：RefFS 直接依赖为空，实现与池级 checker 的闭包里没有它 | — | 站住 |
| V4、V11 | 照登记修订 2 第 2.4 节 | 推敏感 | 站住 |
| 第 1618、8729 行一带注释 | 一致 | — | 站住 |
| 28 行装置变异 | 锚点各恰命中 1 次；V9 用例文档写「K−1 次」而变异停在第 K 次 | 没找到等价或取样点不敏感的（推的） | 站住；V9 那句文档改成与变异一致 |

## 三、去向

1. **对正在跑的 E171 第一段**：Q5 主取样点是脚本第 10 步，跑的是这一版装置（带第五样输入的洞、对拍只罩一对镜像）。第 10 步照跑，它的 q5 行记成「方法待修，结论不作数」；登记修订 5（下一条）之后在新装置上单独重跑 Q5。其余各步不走候选二，不受这一格影响。
2. **登记修订 5（experiment-designer）**：甲（第五样输入的 panic 单列，O1 两列只随看 journal 那遍恢复）、丙（对拍 k = 6）、`no_root_write` 先于对拍判、KM1–KM7 每一处告警的会红用例与变异行、`class_landing_mismatches` 释义与 `q5_partition` 行的字段归属；写明哪几样改了判态路径、阳性对照要不要重跑。
3. **第 27 批实现员**：25-1 那一侧——T13 形状的回退用例与两行变异（`drop-retain`、`drop-extend`）、新用例常量改 2 × 233 + 4 与 KX1、KX2 两行变异、135 边界的纯函数用例；E171 装置那一侧照修订 5 改（含 V9 文档那一句）。
4. **写回**：C682 补「模型约 1277 次发布溢出 panic」与复跑命令；新立一笔欠账记本判决的零轮形态。
5. **E171 要不要在根环线第 10b 批之后的树上重跑**：10b 改了 `allocator.rs`、`mount.rs`、`walk.rs`，第一段的结论只属于冻结树那一版；等第一段跑完、修订 5 定下再一起判。

## 四、复跑核对

攻方：`bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 6 bash research/prompts/m3-formal-code-b25-r1-opus-model/tools/rerun.sh <会话草稿>/b25-opus-rerun k1-targeted k1-long k1-bound k1-rollback-mutations k1-container-number k2-device k1-sweep`，退 0。逐段与报告比：`k1-long` 四行、`k1-bound` 打点行（报告摘的五行逐行同）、`k1-rollback-mutations` 三组、`k1-container-number` 的 checker 开关两行与 `many_inodes_write_and_read` 2 红、`k2-device` 六种跑法（base、dump-k2、dump-k6、fault-k2、fault-k6、noroot）的 E7RESULT 行与报告第 150–153、175–181、223–224 行对上（复跑脚本把 `q5_equivalence_pairing`、`q5` 两种长行截在固定宽度，截断之前逐字同；`q5_equivalence`、`verdict` 两种行整行逐字同）、`k1-sweep` `rows=1296 not_completed=0`。

正推：`SCRATCH=<会话草稿>/b25-sonnet-rerun bash research/prompts/m3-formal-code-b25-r1-sonnet-model/tools/rerun.sh setup|k1|k2`。`setup`、`k2` 退 0；`k1` 退 1，红在「3 行变异另 110 条零红」那一趟（脚本自己的 `timeout=230` 在负载下超时），别的读数都出了：3 行变异各 `outcome=caught failed=1`，KX1、KX2 在新用例上 `not_caught`、在 470 步上 `caught`，KX4、KX6、KX7、KX8 `not_caught`，KX3、KX5 `caught`；K2 附加态 `mismatching_vs_class_columns=0`、抹键 `=936`；V17、V18 `stdout_same_as_unmutated=True`、V19 `exit=3`；KM2–KM7 `stdout_same_as_unmutated=True`、KM1 `differing_lines=2`。

核不动：checker 档内联单测与 28 行装置变异（两腿都没跑，归提交时）；攻方 K1 扫描的数是副本上的，按「副本上的数」引，不进 kb。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | T13 回退用例与两行变异 | 攻方 | 副本上量过变异红 | 第 27 批 |
| 2 | 新用例常量 2 × 233 + 4 与 KX1、KX2 两行 | 正推 | 副本上量过 | 第 27 批 |
| 3 | 135 边界纯函数用例 | 正推 | 副本上量过 | 第 27 批 |
| 4 | 甲（第五样输入单列）与丙（对拍 k = 6） | 攻方 | 丙量过、甲推的 | 登记修订 5、第 27 批 |
| 5 | `no_root_write` 先于对拍判 | 攻方 | 推的 | 同上 |
| 6 | KM1–KM7 会红用例与变异 | 正推 | 关掉不变量过；会红那一半没写 | 同上 |

Q5 候选二是用户 2026-10-04 定的；甲、丙是在候选二之内补它自己的前提，不换候选，不用用户定。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 6 | 备料 | 2026-10-05 不受影响：判的是 E171 装置判得对不对，E171 结论没出，三个 oracle 独立的条款不动 |
| D13（验证路线） 已定项 3 | 不影响 | 2026-10-05 不受影响：RefFS 改正式依赖只改依赖登记，射程不变 |
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-05 不受影响：模型与 core 的切叶规则都照它，135 片之后长一层仍是条款，实现的拒是缺口 |
