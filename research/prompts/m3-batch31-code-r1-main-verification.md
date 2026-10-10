# 删除批第 31 批合入的代码：代码轮第一轮判决（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 N1 N2 N3 N4 N5 N6 F-L0 F-W F-IT F-KEY F-ID-a F-ID-b O3 A2 A3 M1 M2 M3 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch31-code-r1-body.md`、`_m3-batch31-code-r1-diff.md`、`_m3-batch31-code-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-batch31-code-r1-snapshot/`（crates 5 份、kb 2 份：`invariants.md`、D8（核心索引结构） 正文）；冻结副本 `~/.local/share/singlefs/work/m3-batch31-code-r1/tree/` | 冻结副本取在第 31 批合入验证绿之后，crates 5 份 `sha256sum -c` 全 OK；腿跑着的时候主 agent 没改这 7 份 |
| 云端正推（Sonnet） | `research/prompts/m3-batch31-code-r1-sonnet-output.md`（335 行，sha256 27fc26a3…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch31-code-r1-opus-output.md`（298 行，sha256 dcdf5a6e…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 117 处引文，对上 117 处，没判 0 处」；攻方「核了 40 处引文，对上 40 处，没判 0 处」 |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b31-sonnet-model/`（与原目录 `diff -r` 相同），`SINGLEFS_REPO=<冻结副本> rerun.sh ~/.local/share/singlefs/work/m3b31-sonnet-rerun all` 再 `compare.sh` | 与腿的 `expected-compare.txt` 逐行相同「SAME-AS-EXPECTED 92」，汇总 sha256 bcf344ad…；追加变异原样：`RESULT extra line=3 named_test=a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants verdict=SURVIVES cargo_rc=0 ran=5 failed=0`（正推 M3：F-L0 只在层级恰为 1 时调共用函数） |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b31-opus-model/`（逐文件 sha256 相同），`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> THREADS=10 rerun.sh ~/.local/share/singlefs/work/m3b31-opus-rerun base mut-fl0 fix-key` 再 `compare.sh` | 「SAME 1349 行」；`base.probe` 里 `collide=loc0` 9 行全是 `i1_7=Holds`、`collide=loc1` 9 行全是 `i1_7=Violated`；`fix-key.probe` 21 行 `collide=` 里 `Holds` 0 行；`fix-key.kbi` 末行「test result: ok. 187 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out」 |
| 读代码 | 冻结副本 `crates/singlefs-checker/src/walk.rs` 第 2369–2385、2588–2600、2812–2822 行 | 见第二节 O3、A3 两行 |

被判的 5 份文件（K3），每份一格判到。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 N1 F-L0（CPU、GPU 同一支，长度读法） | 一致：CPU 第 2500 行与 GPU 第 3183 行同一支，比长度照 I-2.1 那一行写的读法；不登记引用、两份都对不上时走读失败记两次，CPU 与 GPU 同 | — | **采** |
| K1 N1 第三支没列进 I-2.1 那一行 | 规则没说 | A2：条款缺字，F-L0 按头里类标签的长度比与用户 2026-10-06 定案同向 | **采**：写回 I-2.1 那一行，把「墓碑、谱系层级 ≤ 1 节点里类型段 0 的」列进不往下走的几支（第三节第 1 条） |
| K1 N1 F-W 之后、层级 ≤ 1、类型段 0：另一份按哪种长度比 | **冲突**：I-2.1 那一行（按头里类标签）不红；2026-10-07 写进 I-9.17 / I-9.23 的读法（另一份按类型段那一类的长度比）会红 I-2.1；代码取前者 | — | **代码对，kb 那一句写宽了**：那一句说的是「照类型段那一支往下走那一份」，而类型段那一支在层级 ≤ 1 的类型段 0 上自己就不往下走，这一形属于「不往下走的几支」，用户 2026-10-06 定的是按头里类标签的长度比。改 I-9.2、I-9.17、I-9.23 那一句：类型段那一支自己不往下走的照 I-2.1 那一行的读法（第三节第 1 条） |
| K1 N1 层级 0 的根下类型段 0 | 一致（读码与探针），但没有测试也没有变异钉层级 0：正推 M3 只在层级恰为 1 时调共用函数，存活 | 墓碑、谱系层级 0 的根各 25 形与期望模型逐行相同，只 A3 那一对换盘翻面 | **采**；进第 32 批补层级 0 的根那一形坏镜像与 M3 那行变异 |
| K1 N2 GPU extent 上段根豁免 | 一致（读码）：`pending_level == NONE` 只在读根那一次成立，层级 0 非空与层级 ≥ 1 的根照旧判分隔 key | 没打中：CPU 量 6 形，GPU 读码同判 | **采**；记「没打中（一次抽样）」。提交时由根环线那三条对拍流定 |
| K1 N3 站点 37 / 38 兜底 | 流的造法与期望一致（读码）；第 3065 行一行同改两个常量，两站点各自没被钉；与删句只在「进句时已经判不了」时有差 | 没打中（读码）：最终状态里走读内核不记 61，扫描内核走得到 37 / 38 | **采**；提交时先看这条流基线绿不绿，红就是读码错、第 3065 行没有判别力。两站点各自钉不钉不再加（实现员报告第 4 条：单删一句另一句接住） |
| K1 N4 测试尖度 | 一致：X3、X4、X5 三形与 F-L0 两形复跑 5 条过，变异 3054–3057 全红，正推 M1、M2 也红；GPU 四处调用与四条流一一对应（读码） | — | **采** |
| K1 N5 第 30 批留下的两处 | 一致（读码）；inode 类型段 0 那条「回落 CPU 后同判」在判不了的状态上按构造恒真；`undecidable = UNDECIDABLE_TOO_DEEP` 那一句没有变异行 | — | **采**；那一句的变异进第 32 批 |
| K1 N6 变异行 | 12 行成形、锚点各命中一次、筛得到；CPU 4 行复跑全红；GPU 8 行只核到静态 | — | **采**；GPU 8 行红不红归提交时 |
| K2 观察 O3：「走过」的键 | — | **打中**：容器树容器、inode 叶「走过」按第 0 条位置条目认；第 0 条指到这一遍先走过的单元、读回的是盘 1 那份带 I-1.7 违例的容器时，I-1.7 报成立；换成第 1 条指过去时红。三棵树、三种造法 9 对都翻面，不走 F-W 的两种也中；CPU 与 GPU（读码）一起漏 | **坐实**：冻结副本 `walk.rs` 第 2597 行「let child_placement = (child.locations[0].device, child.locations[0].slot);」后接第 2598 行 `visited_units.insert` 失败即合并、`return`，读回那一份的 I-1.3、出生身份、I-1.7 都不判；inode 叶第 2821 行同形；实例表链早改成按读回那一份认。主 agent 复跑同上。违反 I-1.7 的「任一」，口径同 m3-batch22-code-r1 正文第五节漏判。进第 32 批：照攻方 F-KEY（零轮）三处「走过」按读回那一份的落点认，GPU 同改；`read_index_node` 第 1473 行、extent 数据单元指针那一处、inode 往下走的内部孩子一并核；F-KEY 臂上谱系两形多红 I-4.9 的原因先查 |
| K2 N1 盘序翻面 A3 | — | **弱打中**：类型段 0 那一支「孩子是码 2」只看第一份按 16 KiB 对得上的副本，墓碑、谱系（层级 1 与层级 0 的根）各两对换盘一红一绿；第 30 批就在 | **坐实**（复跑同上：`PROBE matrix tree=tomb level=1 type=t0 c0=c16 c1=node red=[I-2.1,I-4.8,I-7.2,I-7.4,I-9.23]` 对 `c0=node c1=c16 red=[I-2.1,I-4.8,I-7.2,I-7.4]`）。冻结副本 `walk.rs` 第 2374 行「let child_class = read_unit_without_judging(self.reader, &child, node_bytes())」。判据字面两读，**交用户，用户 2026-10-07 定 F-ID-b「只认自证过的副本」**：认孩子的类只看按自己那一类长度整份自证过的副本，CPU、GPU 同改，进第 32 批；改完再写进 I-9.17、I-9.23 |
| K2 N1 主形、一份读不出、CPU 与 GPU | — | 没打中：CPU 量 275 形，257 形与期望模型逐行相同；mut-fl0 臂上比对出 36 处不同（比对会红） | **记「没打中（一次抽样）」** |
| K2 第二次抽样：m3-batch30-code-r1 F-W CPU、F-W 与 F-IT GPU；m3-batch29-code-r1 L1 一份读不出、L2 | — | 都没打中 | **第二次也没打中，记「没打中」**。m3-batch29-code-r1 L1 合法状态多红那一格要层 0，归提交时，仍空着 |
| K2 附带：inode 类型段 7 两份各自证成码 2 与码 3 | — | 换盘 I-3.1 / I-3.11 / I-5.1 跟着翻 | **记**：就是 C739（认孩子的类时两份副本各自证成不同类，取哪一类、按哪一类登记槽数没有条款） 还开着的那一处，不算新打中 |
| K3 | 5 份各一格：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/mutations.tsv`，都一致 | — | **采** |

## 三、进下一批的改法与写回

1. 写回（这一轮判决时主 agent 写）：I-2.1 那一行把 F-L0 这一支列进不往下走的几支；I-9.2、I-9.17、I-9.23 那一句加「类型段那一支自己不往下走的照 I-2.1 那一行的读法」。
2. O3：F-KEY 三处按读回那一份认「走过」，CPU、GPU 同改；造攻方 loc0 那几形坏镜像与对拍流，加变异。
3. A3：F-ID-b，CPU、GPU 同改；钉攻方那 4 对（两种盘序同判）；改完写进 I-9.17、I-9.23。
4. 层级 0 的根下类型段 0 那一形坏镜像与正推 M3 那行变异；inode 类型段 0 那条流的 `UNDECIDABLE_TOO_DEEP` 那一句加变异。

F-KEY、F-ID-b 被攻过零轮。

## 四、交用户

| 岔路 | 用户定案 |
|---|---|
| A3：类型段 0 那一支认「孩子是码 2」时两份副本怎么看：只认自证过的副本 / 任一份不是码 2 就红 / 保持按读者读到的那一份 | 2026-10-07「只认自证过的副本」（F-ID-b） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 不影响 | 2026-10-07 不受影响：内部条目形态不改，打中的是 checker 认「走过」用哪条位置条目、认孩子的类看哪一份副本 |
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-07 不受影响：extent 上段根删空之后可以零条目照旧，这一轮只判 GPU 补上 CPU 已有的那条豁免 |
