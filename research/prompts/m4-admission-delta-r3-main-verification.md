# m4-admission-delta-r3 判决：准入之前的步骤怎么做到不随总数涨（主 agent，2026-11-05，第三轮，最终轮）

<!-- doc-lint:not-numbers B1 B2 B3 B4 B5 B6 B7 B8 A3 A4 J1 J2 J3 J4 J5 J6 J7 J8 J9 J10 J11 G1 K1 -->

正文 `research/prompts/_m4-admission-delta-r3-body.md`，背景材料 `research/prompts/_m4-admission-delta-r3-background.md`。腿：正推（Sonnet）`research/prompts/m4-admission-delta-r3-sonnet-output.md`，攻方（Opus）`research/prompts/m4-admission-delta-r3-opus-output.md`（模型 `m4-admission-delta-r3-opus-model/`）。前两轮判决 `research/prompts/m4-admission-delta-r1-main-verification.md`、`research/prompts/m4-admission-delta-r2-main-verification.md`。代码照旧读第一轮冻结副本 `~/.local/share/singlefs/work/m4-admission-delta-r1/repo/`。

## 一、核查

- 开工快照 `research/prompts/m4-admission-delta-r3-snapshot/kb-sha256.txt`（材料任务 6617，2 份 kb 文件）；cite-check 腿报告自报：正推「核了 39 处引文、124 处裸行号，对上 39 处，没判 124 处」，攻方「核了 44 处引文、27 处裸行号，对上 44 处，没判 27 处」，两份都没有对不上的。
- 攻方引的任务 6394 那一行，主 agent 现查坐实：「today name=fixed_point_vs_checkpoint_cost capacities=capped_mapping_4_8 data_units=85 checkpoint_cost_of_the_previous_version=29 fixed_point_slots_written=43 excess_slots=14 mapping_tree_nodes_after=38」（压小容量下写入类发布的固定点真写比上一版 ckpt_cost 多 14 槽；前两轮判决都没引它）。
- 攻方模型复跑（任务 6719，草稿目录里拷冻结副本、套 `apply_copy_counters.py` 只插计数）：「name=rerun_summary patch_rc=0 counts_rc=0」「name=rerun_verdict all_ran=yes」。读数整行：
  - 「counts name=release_judgement_indexes data_units_in_the_version_before=400 units_in_the_version_before=417 mapped_units_in_the_version_before=413 rounds_entering_the_release_judgement=2 index_builds=4 largest_index_looked_up=417 judgements=1 whole_allocator_copies_before_the_judgement=1 whole_allocator_copies_in_settling_rounds=2」（data_units 取 1、16、128 时 largest_index_looked_up 为 10、26、140）；
  - 「counts name=allocator_copies change=overwrite_of_one_unit unit_area_slots=16384 devices=2 allocation_records_after_the_history=126 judgements=1 whole_allocator_copies_before_the_judgement=1 whole_allocator_copies_in_settling_rounds=2 slot_bitmap_bytes_per_copy_at_least=98304」（unlink 那一行 `judgements=2 … before=2 … settling_rounds=4`）；
  - 「counts name=committed_reservation_of_an_inode_creating_publish inodes_created=32 recorded_before_bytes=198128 recorded_after_bytes=202992 short_if_the_previous_one_is_read_bytes=4864 expected_replicas_times_76_times_k=4864 equals_the_expectation=yes」（1、8 个时 152、1216，都 `equals_the_expectation=yes`）。
- 正推没写模型；它的代价量级都是推的。
- 核不动的：无。

## 二、逐格判定（最终轮）

| 格 | 判定 | 依据 |
|---|---|---|
| B6 上界快路加回落 | 打中（这一轮，攻方 J3 与正推格一同向） | 攻方 J3：辩方给的 B6 定义把删除类与删快照整类排除在快路外，unlink、截断、删快照每一批都走今天整条路，被判的条款本身就是 unlink 那一条（D8（核心索引结构） 已定项 16）。正推格一：式子判保留精确什么都没省，换成断言要改 D28（挂载期承诺量） 已定项 1 两句（`28-挂载期承诺量.md:35`、`:38`）；分配记录树只能取 K1；快路放行后的全量规划不在 D8 已定项 16 免责原文「装单元、拷字节」之内。攻方 J4（现量）：建 inode 的发布若取上一版记下的标签 6，每建一个少扣 2 份副本 × 76 字节（任务 6719 三档都相等）。被攻过一轮，打中 |
| B7 差量规划连差量预演加五件 | 打中（这一轮，攻方 J2 现量） | 五件做完之后固定点每轮释放判定仍对上一版整张 `units`、`mapped_units` 各建一份索引（`transaction.rs:4613`、`:4984`、`:5079`），任务 6719「largest_index_looked_up=417」与 units 同涨；全记录分叶（`allocation_record_tree.rs:372`）与分配记录树全节点集合（`:13045`）不在五件里。正推格二：改完还剩 8 类随总数涨的步（inode 叶容器整份拷、聚簇段求和 `allocator.rs:578`、开新段与回落扫 `:1004`、`:1041`、extent 与其余文件、版本侧两张全索引、分配记录树自己的计划与节点集、「分配器记录等于上一版」前提没断言、实例表链），另有抬 F 预演乘数（至多 48 次整条发布）；节点身份换名触及 14 份源文件 98 行，容器按引用读要重写 `container_tree.rs` 的 Verus 规格。被攻过一轮，打中 |
| 清单完整性 | 不完整，三轮补到这一份 | 攻方 J1（现量）：每次发布两道准入之前在调用方整份拷一次分配器（`transaction.rs:12314`），前三轮清单从 `prepare_the_version_publish` 数起漏了；任务 6719「whole_allocator_copies_before_the_judgement=1 … whole_allocator_copies_in_settling_rounds=2」（每次覆盖写共 3 份，unlink 6 份）。正推格三补三条：回落扫 `allocator.rs:1041`、实例表链 `transaction.rs:8768`、`:12945`、抬 F 预演乘数 `mounted_session.rs:2914–2921`。攻方 J6：照射程后半句读成穷举豁免单，准入之后的真取落点（`transaction.rs:14819`，走第二轮 G2 那条线性扫）也归运行时决策路径一格 |
| 第二轮已判的格 | 站得住 | 攻方 J7：第二轮判 B2、B5 打中的依据站得住；J6：B4 是条款变更站得住，「要改两处」的读法见上一格 |
| 前两轮的数 | 存在性够，频率与带宽不够 | 正推格四与攻方 J8、J10、J11 对上：任务 6265 的「literal_bound_exceeded=1139」是模型自己生成器的频率；`k1_band` 是 python 照公式复算的，起点写死、实写数是假定的，只有 2×4GiB 的 529 有旁证；任务 6553 的候选数、根头下标、墓碑条数三列量的是答案在哪不是走了多少步，撑「线性扫」的是同一行的挂钟列（`lowest_free_pair_search` 那一族 `one_user_data_placement_ns_median` 2830 → 393066、`allocator_copy_by_unit_area` 那一族 `allocator_clone_ns_median` 2870 → 34949849）。前两轮判决里引这几样的句子，只能读成「存在这种情形」，不能读成频率或带宽 |
| 代价量级 | 照抄字节有数，扫类没数 | 正推格五（推的，E199（发布照抄上一版的代价） G0、s=4）：B1 准入前照抄 404072244 字节（E199 产物行 5、13 的算术）；B6 放行时剩 323725868（80.1%，s=2 时 66.8%），仍随墓碑记录数线性涨；B7 上限 2340364。三条的扫类代价（线性找、建索引、整份拷分配器的步数与挂钟）E199 没数 |

## 三、三轮合起来的结局

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：

| 候选 | 被攻几轮 | 打穿几轮 | 结局 |
|---|---|---|---|
| A3 原样（准入判挪到差量上界、没有回落） | 2（第一、二轮） | 2 | 打穿 |
| A4 只换表示 | 2 | 2 | 打穿 |
| B2 差量规划连固定点 | 2（第二、三轮） | 2 | 打穿 |
| B4 只修落点判前置并重述射程 | 2 | 2（是条款变更、要改的不止一处） | 打穿（作为「不改条款」的候选）；作为「改条款」交用户 |
| B5 差量上界放行加精确复核断言 | 2 | 2 | 打穿 |
| B6 上界快路加回落 | 1（第三轮） | 1 | 打中，零轮之后没有再攻 |
| B7 差量规划连差量预演加五件 | 1（第三轮） | 1 | 打中，零轮之后没有再攻 |

没有一条候选在三轮里站住。打中 B7 的都是「还剩哪几步随总数涨」——打中的是它的覆盖面不够，不是它的做法会让准入判错；打中 B6 的有一条是拒绝判对错（J4，建 inode 少扣）。

## 四、交用户

| 事 | 出处 | 被攻过几轮 |
|---|---|---|
| 「准入之前不随总数涨」要做到，随总数涨的步是一张长清单（第一轮判决第二节、第二轮判决第二节、本判决第二节「清单完整性」与 B7 那一格合起来），不是三处；没有一条候选罩全 | 本判决第二、三节 | 三轮 |
| 往哪个方向做，要先有扫类代价的数：现在只有照抄字节的数（E199（发布照抄上一版的代价）），线性找、建索引、整份拷分配器每次发布的步数与挂钟随盘容量怎么涨没有计数模型 | 本判决第二节「代价量级」 | 零轮 |
| B8（零轮，这一轮由三条腿的改法合成，没被攻过）：B7 再加 J1（调用方那一份分配器拷贝改成撤回日志或差量视图）、J2（释放判定的索引随一版增量维护）、正推格二那 8 类与三处补的清单逐项改法 | 正推格二、攻方 J1、J2 | 零轮 |
| B4 那一支：改 D8（核心索引结构） 已定项 16 射程与 `.claude/rules/fs-design.md` 运行时决策路径一格，把两道准入之间那一段（连同准入之后的真取落点）读成不在射程内 | 第二轮判决格三、本判决攻方 J6 | 两轮 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-11-05 不受影响：三轮没有候选站住，条款不改；改不改射程交用户 |
| D8（核心索引结构） 已定项 1 | 不影响 | 2026-11-05 不受影响：记账树台阶行有不改它的做法（第二轮判决格五）|
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-05 不受影响：B5、B6 降精确判为断言要改它两句，两条都打中，不改 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-05 不受影响：准入语义不改 |
