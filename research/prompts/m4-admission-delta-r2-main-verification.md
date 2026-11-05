# m4-admission-delta-r2 判决：准入之前的步骤怎么做到不随总数涨（主 agent，2026-11-05，第二轮）

<!-- doc-lint:not-numbers B1 B2 B3 B4 B5 B6 B7 A3 A4 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 K1 L1 -->

正文 `research/prompts/_m4-admission-delta-r2-body.md`，背景材料 `research/prompts/_m4-admission-delta-r2-background.md`。腿：辩方（Sonnet）`research/prompts/m4-admission-delta-r2-sonnet-output.md`（复核第一轮判决 `research/prompts/m4-admission-delta-r1-main-verification.md`），攻方（Opus）`research/prompts/m4-admission-delta-r2-opus-output.md`（模型 `m4-admission-delta-r2-opus-model/`）。代码照旧读第一轮冻结副本 `~/.local/share/singlefs/work/m4-admission-delta-r1/repo/`。

## 一、核查

- 开工快照：第二轮自己一份 `research/prompts/m4-admission-delta-r2-snapshot/kb-sha256.txt`（材料任务 6516 记的，3 份 kb 文件）；腿交齐之后 `sha256sum -c research/prompts/m4-admission-delta-r2-snapshot/kb-sha256.txt` 三份都「OK」，腿跑着时没人改它们。代码读冻结副本，主工作区之后的改动没碰准入那一段（第二轮正文第二节，主 agent `diff` 现查）。
- cite-check：`python3 scripts/agents/cite-check.py research/prompts/m4-admission-delta-r2-sonnet-output.md research/prompts/m4-admission-delta-r2-opus-output.md --root /home/fy5090/.local/share/singlefs/work/m4-admission-delta-r1/repo --background research/prompts/_m4-admission-delta-r2-background.md`：「✓ 核了 110 处引文、42 处裸行号，对上 78 处，没判 74 处」；不带 `--root`（仓根）：「对上 110 处，没判 42 处（其中裸行号没有引文可比 42 处）」。两次都没有对不上的。
- 攻方模型复跑（任务 6553，在草稿目录里拷冻结副本、套 `apply_settle_trace.py` 改那份拷贝）：「name=rerun_summary scans_rc=0 patch_rc=0 settle_rc=0」「name=rerun_verdict all_ran=yes」。读数整行：
  - 「settle name=settle_rounds change=sequential_write_of_64_data_units settling_calls=1 allocation_rounds_per_call=[2] release_only_rounds=1 roles_compared=74 roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2」（覆盖写与建 inode 那几次都是 `roles_shifted_between_rounds=0`、一轮）；
  - 「scans name=lowest_free_pair_search data_units_kept_below=1200 allocation_records=2598 candidates_walked_before_the_free_pair=1267 cluster_segments_opened_in_the_mount=2 segment_comparisons_at_most=2534 one_user_data_placement_ns_median=393066」（kept_below 取 0、128、400 时 candidates 为 3、163、435）；
  - 「scans name=allocator_copy_by_unit_area unit_area_slots=16777216 unit_area_gib_per_device=256 allocation_records=116 allocator_clone_ns_median=34949849 one_unit_overwrite_publish_ns_median=71945489」（16384 槽时 clone 2870 ns）；
  - 「scans name=root_heads_behind_the_data_units data_units_written=400 data_units_in_the_version=400 units=417 extent_root_at=404 allocation_tree_at=411 accounting_tree_at=412 mapping_tree_at=415」；
  - 「scans name=tombstones_every_publish_copies unlinked=96 tombstone_records_before_an_overwrite=96 tombstone_containers_before_the_overwrite=1 tombstone_records_carried_by_the_overwrite=96」。
- 辩方没写模型，读的是第一轮两份复跑的日志。
- 对第一轮判决的更正（辩方第一节，主 agent 现查任务 6394 日志坐实）：第一轮判决第二节 A3 那一格写「A3 下先报空间不够、推抬 F 写盘」，现量只坐实了前一半——今天那一行「outcome=refused_publish cause=journal_records_exceed_the_limit(86>85) floor_raises=0 images_unchanged=true」，A3 模拟下那一行「outcome=refused_no_space_after_raising_the_floor last_refusal=space_admission_refused floor_raises=0 images_unchanged=true」；拒成哪一种变了，推抬 F 写盘在这条历史上没发生（只在 defer 里有可回收槽的池上才会，推的）。第一轮判决原文不改。
- 核不动的：无。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| 第一轮 A3 出局够不够得着 | 只够得着原样的 A3 | 辩方第一节：拒绝次序变了、字面上界不是上界、K1 一带三条都只打中「没有回落」的 A3；回落变体（辩方 B6）次序与种类与今天相同。攻方 G11 同向：第一轮两处打中是按构造的反例，不靠取样 |
| 第一轮 A4 | 站得住，不打中 B2、B3 | 辩方第二节：对「不改规划算法」站得住，B2、B3 换算法；新规划器交出什么、下游吃不吃得下第一轮没人看过 |
| 格一 B2 的固定点 | 打中（模型现量） | 攻方 G1：B2 两种读法，一种就是 B3，另一种「接着上一轮只补新进来的」与今天不逐个相同——任务 6553「sequential_write_of_64_data_units … roles_shifted_between_rounds=5 new_roles_placed_before_a_role_of_the_earlier_round=2」。辩方格一：收敛到同一个固定点要三个前提，今天一条都没断言；L1：K1 无条件，W 的 2·(K+26) 隐含一条没写的落点局部性假设 |
| 格二 B3 与 B2 差在哪 | B3 不是另一档，是 B2「不克隆」缺的机制；两者都没罩到分配器本体的线性找 | 辩方格二与攻方 G2 对上；攻方 G2 现量：找空槽线性扫（`candidates_walked_before_the_free_pair` 3、163、435、1267 随 `data_units_kept_below` 涨，`allocator.rs:971`、`:974`），记一条分配过滤全部记录（`:1152`），释放按 (盘, 槽) 线性找（`:2265`、`:2211`）；克隆分配器随单元区涨（16384 槽 2870 ns，16777216 槽 34949849 ns）|
| 格三 B4 改射程 | 打中：是条款变更，且要改两处 | 辩方格三与攻方 G4 对上：射程后半句只排出「装单元、拷字节」，两道之间是分配、生命周期判定、会交回拒绝，`.claude/rules/fs-design.md:23` 把它们点名成运行时决策路径；用户「只管准入之前」够不着两道之间 |
| 格四 B5 | 打中 | 辩方格四：读法一什么都不省，读法二就是 B6（要改 D28（挂载期承诺量） 已定项 1 两句）；断言按 `.claude/singlefs-ai-sop/rules/code-discipline.md` 是 panic，降级成空间拒会触发推抬 F 写盘。攻方 G5：上界放不下那一带至少宽 K1 − 实写数，随单元区线性涨 |
| 格五 记账树台阶行 | 有做法，不用改 D8（核心索引结构） 已定项 1 | 辩方格五与攻方 G10 对上：ckpt_cost 的记账树一项可只走删插的那十几条路（差量规划），或取闭式上界（要改 D28（挂载期承诺量） 已定项 4 的读法）；要台阶行条数自己有上界才要改 D5（快照 / 空间记账机制） 已定项 2 射程 |
| 格六 第一轮的方法 | 撑得住各自的存在性，撑不住「A3 一族出局」与「修正式子是真上界」 | 辩方格六；攻方 G11 没打中 |
| 第一轮正文没点名的 | 打中（模型现量） | 攻方 G7：ckpt_cost 每次求值经 `unit()` 线性找根头（`transaction.rs:4351`），根头排在全部数据单元之后（任务 6553「data_units_written=400 … allocation_tree_at=411」）；G8：墓碑、退役、谱系容器落点判之前拷一次、每轮 resolve 里再拷一次（`:8601`、`:8646`、`:8729`、`:12511`），覆盖写带着全部墓碑（「unlinked=96 … tombstone_records_carried_by_the_overwrite=96」）；G9：聚簇段集合在取落点时每个候选走一遍。B2、B3、B5 的定义都没罩到这几处 |

## 三、第三轮（最终轮）

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，第三轮只攻前两轮站住的形态：

- **B6 上界快路加回落**（辩方提）：上界够用才走快路，放行之后固定点、记录条数、合取 7 一字不改；放不下整条走今天的路。它只救放行路径。
- **B7 差量规划连差量预演加五件**（攻方提）：B3（差量规划、分配器差量视图）之外，取空槽与记录按索引找、根头按下标取、墓碑退役谱系容器按引用读、记账树按差量算。
- 第三轮另攻：随总数涨的步的清单完整不完整（第一轮正推表一表二、第二轮 G7–G9 之后还有没有）；这两轮报出的数能不能核。
- 第三轮里新冒出来的形态不再开轮，写进判决的交用户表、标「零轮」。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| 这一轮不交：B2、B4、B5 打中，B6、B7 被攻过零轮，进第三轮 | 本判决第二、三节 | 一轮（B2、B4、B5）、零轮（B6、B7） |
| B4 是条款变更（D8（核心索引结构） 已定项 16 射程与 `.claude/rules/fs-design.md` 运行时决策路径一格两处），第三轮之后若交用户，交的是「改不改条款」 | 第二节格三 | 两轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-11-05 不受影响：这一轮只判候选；射程读法照第一轮，B4 要改它，挂起 |
| D8（核心索引结构） 已定项 1 | 不影响 | 2026-11-05 不受影响：记账树台阶行有不改它的做法 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-05 不受影响：B5、B6 降精确判为断言要改它两句，挂起 |
| D28（挂载期承诺量） 已定项 4 | 不影响 | 2026-11-05 不受影响：记账树一项取闭式上界要改它的读法，挂起 |
| D5（快照 / 空间记账机制） 已定项 2 | 不影响 | 2026-11-05 不受影响：只有要台阶行条数自己有上界才动它 |
