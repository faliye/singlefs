# 实审 A4b 报告：准入 ckpt_cost 按最坏情况计、重钉被 A4 带红的用例、查「推满抬 F 之后式子放行、落点仍取不到」

时刻：2026-09-26 UTC 22:07–23:3x（JST 2026-09-27 07:07–08:3x）。规格 `/tmp/claude-1000/impl-rev-a4b/spec.md`；A4 报告 `research/prompts/m2-rev-a4-implementer-report.md`。
途中收到主 agent 两条消息（22:4x UTC）：① 给 `crates/mutations.tsv` 实审 A4 那几行新锚点、写进替换文件并在副本证红，主表不直接改；② 随机历史 `random_history.rs:496` 那条已派调查员、不用管；③（22:5x）核崩溃枚举那条历史第几次被落点拒。三件都在下面各节答了。

## 一、结论

1. **式子改了**（`crates/singlefs-core/src/admission.rs`）：ckpt_cost = 分配记录树 + 中央映射树 + 记账树节点数 + 树表 1，一律按最坏情况计。
   - 分配记录树：盘数 × 2 × (高 − 1) + 1（每块盘两条叶路径，罩 bump 游标跨叶那一次）；
   - 中央映射树：逐层 min(这一层的节点数, 删 + 插的条目数) + 这一层至多切出来的节点数，根切开再加长出来的新根；一次空发布删、插的条目各至多「分配记录树那一项 + 记账树节点数」。
   A4 量表的全部 7 格加新造的「产品容量、中央映射树 2 层」一格（两块 4 GiB 盘、文件顺序写到 300 个数据单元），每一次空发布 difference（实写 − ckpt_cost）都 ≤ 0，没有一次少扣。多扣：产品容量 0–2（1 GiB）/ 2–4（4、64 GiB）/ 4–6（1 TiB）/ 5–7（映射树 2 层）块；只供测试的压小容量三格 11–16 / 17–29 / 141–163 块（见第三节）。
2. **重钉**：被 A4 带红的 7 条里 6 条重钉（表见第四节），4 个测试二进制在副本与主工作区都绿；第 7 条（随机历史「空间准入判着」取样点）在新式子下**不用改就绿**：32 个种子的落点拒绝 0 次（A4 式子下 2 次），整条取样点用例过。
   - `an_overwrite_whose_new_slots_exceed…` 照 A4 说的失去判别力，换几何（覆盖写 9 次 → 5 次）重造：可用 11 槽、需求 14（普通 6 + 固定点 8），普通分配装得下、固定点装不下，变异「需求只算普通分配」照样红。
   - `after_a_file_is_truncated…` 的「界 3」读法我改了一处（删与写回之间夹的发布数 ≤ 3，照条款逐字「要再发生 3 次这样的发布才回可分配集合」），**交主 agent 确认**（第八节第 1 条）。
3. **第 3 步定位**：A4 式子下那 2 次（种子偏移 29，第 113、132 步）是**落点一侧**的问题，不是式子少算：两块盘各有 100 个空槽，全在这次挂载开过的 2 个聚簇段里，段外成对的空槽 0 个，用户数据按 D3（空间分配） 已定项 10 ② 不许落进聚簇段；式子按槽数算可用 14 槽、放行。这是 C545（空间准入罩不住分裂与聚簇段层）那一格，没修。
4. **崩溃枚举那条历史（主 agent 第 ③ 问）**：直接调发布路径首次被落点拒：用例里写死的第 51 次是 A4 之前（按树高）钉的，今天没重量；同一份树（22:31 UTC 快照）上 A4 式子第 35 次（之后 50、65、80）；A4b 式子第 62 次（之后 77），第 18、32、47 次是式子先拒。B3a-3 看到的「第 51 次做成、第 65 次被拒」是 A4 式子下的数（它照抄 50 次会话覆盖写，第 35、50 次在会话里已经推过抬 F）。我那份钉 62，`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs` 的新值只写在这里（第五节），没动那份文件。
5. **变异**：新加 10 行（`mutations-append.tsv`）、替换实审 A4 那 5 行（`mutations-replacements.tsv`，按名字，含主 agent 点的 974、975、977、978 与测试名跟着变的 976），在副本上 25 行全证过红，外加点名那 6 条用例的 12 行既有变异照旧红（第六节）。主表没改。
6. **交主 agent 的设计问题**（第八节）：「界 3」的读法；分配记录树「每块盘两条路径」不是全部最坏情况（开新段、回落、换下旧叶里的节点时一块盘可改多于两片叶）；写入口的只供测试压小容量准入读不到；窄盘上只崩了再挂从第 8 次起永远推满仍不够。

**什么现象会推翻结论**：量表任一格出现 difference > 0；或主工作区合并别的会话的改动之后，四个测试二进制里任一条红在本报告钉的数上；或有人造出一次空发布在一块盘上改了三片叶以上（那时第 1 条的「分配记录树按每块盘两条路径」不再是上界，见第八节第 2 条）。

## 二、第 1 步：式子改了什么

`crates/singlefs-core/src/admission.rs`（行号是交回时主工作区里的）：

- `checkpoint_cost_of_the_version_to_build_on`（第 499 行）照旧是产品路径（发布与可写挂载的准入）调的那一个，改成转调
  `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`（第 513 行，新，`pub`），节点容量取 `CodeTwoTreeNodeCapacities::FromTheNodeFormat`。
  带容量的那一个给只供测试的压小容量量同一个数用（写入口装着压小容量时它的空发布按压小的容量切分）。
- 分配记录树：`allocation_record_tree_nodes_on_two_leaf_paths_per_device`（第 584 行，改名自 A4 的 `…_on_one_leaf_path_per_device`）
  = 盘数 × `ALLOCATION_RECORD_TREE_LEAF_PATHS_PER_DEVICE_AT_MOST`（第 574 行，= 2）× (高 − 1) + 1。`Some` 与 `None` 两臂都走它。
- 中央映射树：`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`（第 657 行，新）。输入：每层节点数（`central_mapping_tree_nodes_at_each_level`，
  第 610 行，形状里按 (层级, 层内序号) 二分找每层边界，不逐个数；断言形状的高 = 根节点头里读的高）、这一版进映射的条目数（`mapped_units.len()`）、
  删与插各至多几条（`CentralMappingEntryChangesOfAnEmptyPublish`，第 600 行；= 分配记录树那一项 + 记账树节点数）、这棵树的节点容量。
  逐层：原有节点里在改动路径上的 ≤ min(这一层节点数, 删 + 插)；切出来的新节点 ≤ `splits_at_one_level_at_most`（第 638 行）
  = min(原有节点数, 加的条数) + 余下的条数 ÷ ⌈容量 ÷ 2⌉（原有节点第一次切至少加 1 条，切出来的两半再切至少加 ⌈容量 ÷ 2⌉ 条；先删后插，删只让节点更空）；
  叶层「全树条目 + 插的 ≤ 叶容量」、内部层「下一层至多的节点数 ≤ 内部容量」时这一层一次都切不了；根切开就长一层（新根 1 + 它自己的切分，新根起步两个孩子）。
- 记账树（节点数）、树表（1）、实例表链不进，这三样照旧。模块文档第 18–20 行跟着改。
- 单测（`admission::tests`）：`allocation_record_tree_term_is_two_leaf_paths_on_each_device_plus_the_shared_root`（第 1078 行，改名改值：(高, 盘数) = (2,2)(3,2)(4,2)(3,3) → 5、9、13、13）、
  `central_mapping_term_counts_changed_paths_and_splits_on_every_level`（第 1111 行，新：产品容量 294 / 143 下四种形状 → 1、3、5、41）、
  `central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities`（第 1147 行，新：叶 2 内部 2 下 → 6、11）。

条款没写、我取的读法（交主 agent 看）：一次空发布里在映射树里改的只有分配记录树与记账树的节点（数据单元、extent、inode 的条目照抄上一版 key），
这一条是读 `crates/singlefs-core/src/transaction.rs` 装 `mapped_units` 那一段（第 4109 行 `let mut mapped_units` 起到第 4259 行 `let mapping_keys` 止）得出的；将来空发布要重写别的进映射的单元，这里要跟着加。

### D28（挂载期承诺量） 已定项 4 的新句（我写不了 kb，交书记员）

`.claude/kb/decisions/28-挂载期承诺量.md` 已定项 4「形态」首句与其后两条子项，建议换成：

- **形态**：ckpt_cost = 一次空发布至多写出的固定点单元数，一律按最坏情况计（用户 2026-09-27 定：游标跨叶、中央映射树多层都不许少扣），按当时的结构现算、每次发布重算：
  分配记录树 + 中央映射树 + 记账树每发布的节点数 + 1（树表）。每棵码 2 树当前的高 = 发布那一刻从它的根节点码 2 头里现读的层级 + 1（原句其余照旧）。
  - 分配记录树按盘分路：盘数 × 2 ×（当前的高 − 1）+ 1——每块盘至多两条从叶到根之下那一层的路径（bump 游标跨过一片叶的末槽那一次：这次取的落点在新叶、换下的上一版落点在旧叶），根一个；式子里的「盘数」是池里的盘数（第一版每个单元落池里每一块盘）。
  - 中央映射树逐层计：一次空发布在映射树里删、插的条目各至多「分配记录树那一项 + 记账树节点数」条；每层至多 min(这一层的节点数, 删 + 插) 个原有节点在改动的路径上，加这一层至多切出来的节点数（叶层按插的条数、内部层按下一层切出来的孩子数，切开的一半再切至少要加 ⌈节点容量 ÷ 2⌉ 条；全树条目加插的装得进一片叶、或下一层全部节点装得进一个内部节点时那一层一次都切不了），根切开再加长出来的新根；节点容量取节点格式的（叶 294、内部 143）。
  - 记账树不进 Σ，单列，按每发布的节点数；树表单列一项，每次发布 1 个单元；实例表链不进 ckpt_cost。
- **射程**补一句：「每块盘两条路径」罩的是 bump 游标在一个开放段里跨一片叶的末槽；一次空发布里开放段装不下而开新段、回落到最低空槽，或者换下的是很久以前落在别的叶里的节点时，一块盘上改的叶可以多于两片，这一格没定（建议新开欠账，名字由主 agent 起）。实测多扣（`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs` 的量表）：两块盘产品容量 0–7 块。
- **依据**补一条：用户定案 2026-09-27「一律按最坏情况计（游标跨叶、中央映射树多层都不许少扣）」「窄盘更早报 ENOSPC 认」；量表同上。
- 第一版两块 4 GiB 盘实现取值：ckpt_cost = 9 + 1 + 1 + 1 = 12（A4 那一版 8、按树高 6）；D16（发布语义） 已定项 1 第 57 行「第一版池规模 9 块 ⇒ 保留池 73 块」是 E148 的模型数，要写实现取值的话是 10 + 7 × 12 = 94 块——改不改由书记员照新句定。
- 同一个量也是切换预留里的 c_max：窄盘（单元区 240 / 256 / 384 槽，分配记录树高 2）ckpt_cost = 5 + 1 + 1 + 1 = 8（A4 6、按树高 5），
  每块盘切换预留 4 × (2 + 3 × 8) = 104 槽（A4 80、按树高 68），加保留池 8 槽。
- `.claude/kb/decisions/05-快照-空间记账机制.md` 第 107 行对照表「按当时各记录树的高之和 + 记账节点现算」同样过时，建议换成「按一次空发布至多写出的固定点现算（分配记录树每块盘两条路径、中央映射树逐层、记账节点、树表）」。
- D16（发布语义） 已定项 1「准入」那一行（第 42 行）没有 ckpt_cost 的式子，不用改；D2（RAID 条带策略） 已定项 13 C565 那一格（`02-RAID条带策略.md` 第 224 行「挂载准入」那一行的例外）行为没变（推满仍不够挂载照样做成），窄盘上走到它更早、更常（第四节第 6 条）。

## 三、量表（验收：A4 量表全部格 + 产品容量映射树 2 层一格，difference 全部 ≤ 0）

装置：`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（改写）。每格两块同宽稀疏内存盘：mkfs → 取号 → 暖机 → 第一个文件 → 覆盖写一次（映射树 2 层那一格再顺序写到 300 个数据单元）→ 空发布 60 次（1 TiB 30 次）。
每次发布之前按那一版现算 ckpt_cost（产品容量那几格同时核产品路径那个函数给同一个数；压小容量那几格按写入口装的同一个容量算），发完按 `rewritten` 逐棵、分配记录树逐盘数。
列：`ckpt_by_height` = 按树高那一版；`ckpt_a4` = A4 那一版（每块盘一条叶路径、映射树按高）；`ckpt` = 现在；`diff` = 每块盘实写固定点槽数 − ckpt（正 = 少扣）；`diff_a4` 同一次发布对 A4 那一版。
原样行 460 行在 `/tmp/claude-1000/impl-rev-a4b/a4b-rows-final.log`（`cargo test -p singlefs-harness --test admission_checkpoint_cost_per_device_paths -- --nocapture --test-threads 1`，副本 copy-prove，经内存包装），按形状归并（`aggregate.py`）的全文：

```text
  1x two-4GiB-mapping-4-8 overwrite h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=24 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=-13 diff_a4=2 | overwrite@txg4
  7x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=24 alloc=d0:2,d1:2+root1 map=3 acc=1 tt=1 fixed=10 diff=-14 diff_a4=1 | empty#0@txg5 .. empty#6@txg11
  1x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=24 alloc=d0:3,d1:3+root1 map=4 acc=1 tt=1 fixed=13 diff=-11 diff_a4=4 | empty#7@txg12
  1x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=26 alloc=d0:3,d1:3+root1 map=4 acc=1 tt=1 fixed=13 diff=-13 diff_a4=4 | empty#8@txg13
  1x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=26 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=-15 diff_a4=2 | empty#9@txg14
 50x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=26 alloc=d0:2,d1:2+root1 map=3 acc=1 tt=1 fixed=10 diff=-16 diff_a4=1 | empty#10@txg15 .. empty#59@txg64
  1x two-4GiB-mapping-3-3 overwrite h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=33 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=-18 diff_a4=5 | overwrite@txg4
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=33 alloc=d0:2,d1:2+root1 map=7 acc=1 tt=1 fixed=14 diff=-19 diff_a4=4 | empty#0@txg5
  4x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=33 alloc=d0:2,d1:2+root1 map=5 acc=1 tt=1 fixed=12 diff=-21 diff_a4=2 | empty#1@txg6 .. empty#4@txg9
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=33 alloc=d0:3,d1:3+root1 map=7 acc=1 tt=1 fixed=16 diff=-17 diff_a4=6 | empty#5@txg10
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=40 alloc=d0:3,d1:3+root1 map=7 acc=1 tt=1 fixed=16 diff=-24 diff_a4=6 | empty#6@txg11
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=40 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=-25 diff_a4=5 | empty#7@txg12
 52x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3->3 acc_before=1 ckpt_by_height=8 ckpt_a4=10 ckpt=41 alloc=d0:2,d1:2+root1 map=5 acc=1 tt=1 fixed=12 diff=-29 diff_a4=2 | empty#8@txg13 .. empty#59@txg64
  1x two-4GiB-mapping-2-2-accounting-8-3 overwrite h_alloc=3 h_map=4->4 acc_before=4 ckpt_by_height=12 ckpt_a4=14 ckpt=171 alloc=d0:2,d1:2+root1 map=14 acc=4 tt=1 fixed=24 diff=-147 diff_a4=10 | overwrite@txg4
  2x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4->4 acc_before=4 ckpt_by_height=12 ckpt_a4=14 ckpt=171 alloc=d0:2,d1:2+root1 map=13 acc=4 tt=1 fixed=23 diff=-148 diff_a4=9 | empty#0@txg5 .. empty#1@txg6
  1x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4->5 acc_before=4 ckpt_by_height=12 ckpt_a4=14 ckpt=171 alloc=d0:3,d1:3+root1 map=18 acc=4 tt=1 fixed=30 diff=-141 diff_a4=16 | empty#2@txg7
 57x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=5->5 acc_before=4 ckpt_by_height=13 ckpt_a4=15 ckpt=189 alloc=d0:2,d1:2+root1 map=16 acc=4 tt=1 fixed=26 diff=-163 diff_a4=11 | empty#3@txg8 .. empty#59@txg64
  1x two-1GiB overwrite h_alloc=2 h_map=1->1 acc_before=1 ckpt_by_height=5 ckpt_a4=6 ckpt=8 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=-2 diff_a4=0 | overwrite@txg4
 58x two-1GiB empty h_alloc=2 h_map=1->1 acc_before=1 ckpt_by_height=5 ckpt_a4=6 ckpt=8 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=-2 diff_a4=0 | empty#0@txg5 .. empty#59@txg64
  2x two-1GiB empty h_alloc=2 h_map=1->1 acc_before=1 ckpt_by_height=5 ckpt_a4=6 ckpt=8 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=0 diff_a4=2 | empty#13@txg18 .. empty#14@txg19
  1x two-4GiB overwrite h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | overwrite@txg4
 58x two-4GiB empty h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | empty#0@txg5 .. empty#59@txg64
  2x two-4GiB empty h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=-2 diff_a4=2 | empty#9@txg14 .. empty#10@txg15
  1x two-64GiB overwrite h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | overwrite@txg4
 58x two-64GiB empty h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | empty#0@txg5 .. empty#59@txg64
  2x two-64GiB empty h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=-2 diff_a4=2 | empty#9@txg14 .. empty#10@txg15
  1x two-1TiB overwrite h_alloc=4 h_map=1->1 acc_before=1 ckpt_by_height=7 ckpt_a4=10 ckpt=16 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=-6 diff_a4=0 | overwrite@txg4
 28x two-1TiB empty h_alloc=4 h_map=1->1 acc_before=1 ckpt_by_height=7 ckpt_a4=10 ckpt=16 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=-6 diff_a4=0 | empty#0@txg5 .. empty#29@txg34
  2x two-1TiB empty h_alloc=4 h_map=1->1 acc_before=1 ckpt_by_height=7 ckpt_a4=10 ckpt=16 alloc=d0:4,d1:4+root1 map=1 acc=1 tt=1 fixed=12 diff=-4 diff_a4=2 | empty#7@txg12 .. empty#8@txg13
  1x two-4GiB-file-300-units-mapping-two-levels overwrite h_alloc=3 h_map=1->1 acc_before=1 ckpt_by_height=6 ckpt_a4=8 ckpt=12 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=-4 diff_a4=0 | overwrite@txg4
 55x two-4GiB-file-300-units-mapping-two-levels empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=16 alloc=d0:2,d1:2+root1 map=2 acc=1 tt=1 fixed=9 diff=-7 diff_a4=0 | empty#0@txg6 .. empty#59@txg65
  5x two-4GiB-file-300-units-mapping-two-levels empty h_alloc=3 h_map=2->2 acc_before=1 ckpt_by_height=7 ckpt_a4=9 ckpt=16 alloc=d0:3,d1:3+root1 map=2 acc=1 tt=1 fixed=11 diff=-5 diff_a4=2 | empty#2@txg8 .. empty#45@txg51
```

每格一行汇总与三块盘那一格（原样）：

```text
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-4-8 empty_publishes=60 crossings=2 taller_central_mapping=60 central_mapping_grew=0 difference_min=-16 difference_max=-11
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-3-3 empty_publishes=60 crossings=2 taller_central_mapping=60 central_mapping_grew=0 difference_min=-29 difference_max=-17
name=a4b-checkpoint-cost-cell cell=two-4GiB-mapping-2-2-accounting-8-3 empty_publishes=60 crossings=1 taller_central_mapping=60 central_mapping_grew=1 difference_min=-163 difference_max=-141
name=a4b-checkpoint-cost-cell cell=two-1GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-2 difference_max=0
name=a4b-checkpoint-cost-cell cell=two-4GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-4 difference_max=-2
name=a4b-checkpoint-cost-cell cell=two-64GiB empty_publishes=60 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-4 difference_max=-2
name=a4b-checkpoint-cost-cell cell=two-1TiB empty_publishes=30 crossings=2 taller_central_mapping=0 central_mapping_grew=0 difference_min=-6 difference_max=-4
name=a4b-checkpoint-cost-cell cell=two-4GiB-file-300-units-mapping-two-levels empty_publishes=60 crossings=5 taller_central_mapping=60 central_mapping_grew=0 difference_min=-7 difference_max=-5
name=a4-checkpoint-cost cell=three-4GiB-allocation-record-tree-only-one-leaf height_allocation_record_tree=3 allocation_record_tree_changed=d0:2,d1:2,d2:2+root1 checkpoint_cost_of_the_version_without_file=14
name=a4-checkpoint-cost cell=three-4GiB-allocation-record-tree-only-crossing height_allocation_record_tree=3 allocation_record_tree_changed=d0:3,d1:3,d2:3+root1 checkpoint_cost_of_the_version_without_file=14
```

读法：产品容量五格每块盘一片叶的空发布多扣 2（1 GiB）/ 4（4、64 GiB）/ 6（1 TiB）/ 7（映射树 2 层）块，跨叶那几次多扣 0 / 2 / 4 / 5 块——1 GiB 跨叶那两次正好相等（实写 8 = 2 × 2 × 1 + 1 + 1 + 1 + 1）。
产品容量下映射树 2 层那一格每次空发布只改 2 个映射节点（= 树高），A4 那一版在这一格只在跨叶那 5 次少扣（diff_a4 = 2）；映射树逐层那一项的判别力在压小容量三格（A4 那一版 55–60 次全少扣 1–16 块）。
三块盘那一格（发布路径走不到，只在分配记录树这一层量）：跨叶那一次每块盘改 3 个节点（两片叶 + 共用的层级 1 节点），3 × 3 + 1 + 树表 1 = 11 ≤ 14。
这份量表产物没进 `research/results/`：我的写范围只有 `crates/`、`litmus/` 与草稿目录；要留由主 agent 拷。

## 四、第 2 步：重钉（改前 = 按树高那一版钉的；A4 = 按每块盘一条叶路径；A4b = 现钉）

A4 与 A4b 两列都在同一份树上量（主工作区 2026-09-26 22:31 UTC 快照，副本 `copy-base` 用 A4 的 `admission.rs`、`copy-work` / `copy-scan` 用 A4b 的）。窄盘上 ckpt_cost（= c_max）按树高 5、A4 6、A4b 8。

| # | 用例（文件:行） | 断言 | 改前 | A4 | A4b（现钉） | 算式 / 来源 |
|---|---|---|---|---|---|---|
| 1 | `writable_mount_short_of_its_instance_switch_reserve_…`（`second_transaction_supplement_two_admission_formula.rs:475`） | 第 506 行取号之前每块盘可用 | −8 槽 | −21 | **−47** | 256 − 191 − 4 × (2 + 3c) − c，c = 5 / 6 / 8 → −8 / −21 / −47（三版一致：已分配 191 槽不随式子变）；F 上限 14、推一串 3 次、之后够，三样不变 |
| 2 | `an_overwrite_whose_new_slots_exceed_…`（同文件:591） | 盘面、需求、可用 | 覆盖写 9 次；(12, 6)；可用 8 | 可用 −5（判别力没了） | 覆盖写 **5** 次（第 594 行）；**(14, 6)**（第 676 行）；可用 **11**（第 686 行） | 探针扫 240 / 256 / 384 槽 × 0..30 次（`probe-geometry-a4b-1.log`）：9 次那一格挂载要先推抬 F、推完可用 44；「挂载取号之前就够且 6 ≤ 可用 < 需求」只有 240 × 5（可用 11、需求 14）与 384 × 16（7、14）两格，取前者。可用 11 = 240 − 117 − 4 × (2 + 24) − 8（已分配 117 由可用反推）；需求 14 = 普通 6 + 固定点 8（从同一次覆盖写关掉准入真发出去的角色现数） |
| 3 | `an_admitted_overwrite_whose_data_unit_finds_no_slot_…`（`second_transaction_admission_raises_the_floor_before_refusing.rs:57`） | 第 82 行第一次落点被拒、推过再做成是第几次 | 51（60 次内） | 35（[35, 50]） | **62**（覆盖写次数 60 → **70**，第 63 行） | 探针（`probe-direct-copy-scan.log`）：会话里第 18、32、47 次是式子先拒、推过再发成，第 62、77 次是落点被拒、推过再发成；70 次内只有 62 |
| 4 | `after_a_file_is_truncated_the_same_size_…`（同文件:101） | 第 153 行 (写回被拒次数, 删之后第几次用户可见发布做成) | (2, 3) | (1, 2) | **(3, 4)** | 实跑；第 159 行「界 3」改成数删与写回之间夹的发布（3 ≤ 3），读法交主 agent（第八节第 1 条） |
| 5 | `mount_still_short_after_the_publishes_of_one_admission_…`（同文件:221） | 第 254 / 262 / 267 / 280 行 | 短 7；推 (8,3)(11,3)；仍短 5；F 11 | 短 14（后面没走到） | 短 **36**；推 **(20,3)(23,3)**；仍短 **34**；F **23** | 探针（`probe-mount-a4b-1.log`）：第 8、9 次挂载已经推满仍不够、各推两串（F 到 5、14），第 10 次再推两串到 23；停因照旧「再推一串超过 8 次」、会话写报空间不够、卸载之后再挂取号之前就够、checker 0 违例，这几样不变 |
| 6 | `crash_only_remounts_on_a_narrow_pool_…_never_get_stuck`（同文件:177） | 第 206 行 15 次挂载的准入结局 | 够 ×9、推满不够 ×2、推了够、够 ×3 | 够 ×8、推满不够 ×7 | **够 ×7、推满不够 ×8** | 实跑；第 8 次起这块盘再也回不到「够」（A4 报告设计问题第 4 条，用户认了窄盘更早报 ENOSPC）；挂载全做成、checker 0 违例、F 只升不降照旧 |
| 7 | `unit_area_wall_sampling_…_with_the_space_admission_judged_…`（`second_transaction_supplement_three_random_history.rs:445`） | 第 496 行落点拒绝次数 = 0 | 0 | 2 | **0（没改）** | A4b 下整条用例绿（`impact-a4b-1/random-history-release.log`：22 过、2 红，红的两条是基线就红的，见第七节）；这条归调查员，我没动文件 |

改了用例头注释的：1、2、3、4、5、6 各自的文档注释照新数改写（2 另写明为什么换几何；4 写明两种算法；6 把「第 12 次推了就够」那句删掉，写成第 8 次起推满仍不够）。
点名这 7 条的 16 行既有变异（`crates/mutations.tsv` 第 638–644、760–766、804–805 行）锚点都在 core / harness 源码里、我没动，名字里的用例名也没改，**主表一行都没改**；
前 6 条的 12 行在副本上单跑照样红（第六节），第 7 条的 4 行（643、765、804、805）没重证：那条用例归调查员，而且它们按参数是 debug 下整批取样，单条跑很久。

## 五、第 3 步：「推满抬 F 之后式子放行、落点仍取不到」那 2 次；崩溃枚举那条历史第几次被拒

### 复现与定位（A4 式子）

副本 `copy-probe`（主工作区 22:31 UTC 快照、`admission.rs` 换回 A4 那一版）里加了两样只在副本里的探针：`PoolAllocator::probe_space_summary`（每块盘的空槽在不在聚簇段里、段外成对空槽几对、扣住与隔离几槽），
`history.rs` 的 `settle_user_change` 在会话报 `NoSpaceAfterRaisingTheFloor(PlacementRefused)` 时打一行；再用一条探针用例把「空间准入判着」取样点那 32 个种子（种子基 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`、150 步、`TOWARD_THE_UNIT_AREA_WALL`、单元区 256 槽）逐个串行跑（release）。原样（`probe-a4-1.keep`，行过长处原样保留）：

```text
PROBE placement-refused-after-raising unit=Data(DataUnitIndexInFile(0)) refusal=NoFreeSlotOnAnyDevice stop=PublishesPerAdmissionWouldBeExceeded { publishes_pushed: 6, publishes_of_the_next_raise: 3 } floor_raises=2 refusals_that_pushed=["SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_", "SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_"] content_bytes=0 ckpt=6 available=[(0, 14), (1, 14)] txg=CheckpointTxg(199) floor=CheckpointTxg(173) space: dev0[unit_area=256 allocated=156 deferred=140 free_counter=100 free_in_cluster_segments=100 free_outside=0 free_pairs_outside=0 held=0 isolated=0 empty_segments=0] dev1[unit_area=256 allocated=156 deferred=140 free_counter=100 free_in_cluster_segments=100 free_outside=0 free_pairs_outside=0 held=0 isolated=0 empty_segments=0] cluster_segments=2 open_segment=None bump_cursor=50368
STEP seed_offset=29 position=Operation(113) member=UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))
PROBE placement-refused-after-raising unit=Data(DataUnitIndexInFile(0)) refusal=NoFreeSlotOnAnyDevice stop=PublishesPerAdmissionWouldBeExceeded { publishes_pushed: 6, publishes_of_the_next_raise: 3 } floor_raises=2 refusals_that_pushed=["SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_", "PlacementRefused { unit: Data(DataUnitIndexInFile(0)), refus"] content_bytes=23242 ckpt=6 available=[(0, 14), (1, 14)] txg=CheckpointTxg(286) floor=CheckpointTxg(260) space: dev0[unit_area=256 allocated=156 deferred=140 free_counter=100 free_in_cluster_segments=100 free_outside=0 free_pairs_outside=0 held=0 isolated=0 empty_segments=0] dev1[unit_area=256 allocated=156 deferred=140 free_counter=100 free_in_cluster_segments=100 free_outside=0 free_pairs_outside=0 held=0 isolated=0 empty_segments=0] cluster_segments=2 open_segment=None bump_cursor=50368
STEP seed_offset=29 position=Operation(132) member=UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))
DONE 29 ending=Completed placement_refusals=2
TOTAL placement_refusals=2
```

判定：两次都在种子偏移 29（种子 7463871032432355142）的第 113、132 步，被拒的是数据单元 `Data(0)`；两块盘各 100 个空槽**全在这次挂载开过的 2 个聚簇段里**，段外空槽 0、段外成对空槽 0，扣住 0、隔离 0；
式子按槽数算每块盘可用 14 槽、放行；推抬 F 停在「再推一串超过一次准入的 8 次发布」（已推 2 串 6 次）。用户数据不许落进聚簇段（D3（空间分配） 已定项 10 ②），所以是**落点一侧**：
式子不看空槽在不在聚簇段里、成不成对（C545（空间准入罩不住分裂与聚簇段层） 那一格），不是 ckpt_cost 少算了哪一项。没修（规格：是别的问题写进报告不修）。

### A4b 式子下

同一个探针换上 A4b 的 `admission.rs` 重跑 32 个种子（`probe-a4b-1.log`）末行原样：

```text
TOTAL placement_refusals=0
```

整条取样点用例在 A4b 下也绿（第七节的随机历史汇总）。这不说明 C545 那一格合上了：保留池与切换预留每块盘多扣 26 槽，这一批种子上式子先拒、落点那一道走不到；换种子或换盘宽照样可能走到。

### 崩溃枚举那条历史（主 agent 第 ③ 问）

`second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（`#[ignore]` 的崩溃枚举）照抄 `an_admitted_overwrite_…` 的前缀：单元区 384 槽、第一个文件之后崩了再挂、会话里覆盖写 50 次、第 51 次直接调发布路径要求 `PlacementRefused`。
探针 `probe_direct_publish_after_session_overwrites`（副本里）每一次会话覆盖写之前，在盘面与分配器的拷贝上直接调发布路径覆盖写一次、记结局，再照常经会话覆盖写。同一份树上两版式子，直接发布被拒的那几次原样：

```text
== A4（copy-base）
OVERWRITE 20 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 35 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
OVERWRITE 50 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
OVERWRITE 65 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
OVERWRITE 80 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
== A4b（copy-scan）
OVERWRITE 18 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 32 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 47 direct_before=SpaceAdmissionRefused session=Ok pushed_by=["admission"]
OVERWRITE 62 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
OVERWRITE 77 direct_before=PlacementRefused(Data(DataUnitIndexInFile(0)), NoFreeSlotOnAnyDevice) session=Ok pushed_by=["placement"]
```

- 改前数：用例写死的第 51 次是 A4 之前钉的（今天没重量）。A4 那一版在这份树上第一次直接发布落点被拒是第 35 次，会话在第 35、50 次各推过抬 F，所以照抄 50 次之后第 51 次直接发布做成、到第 65 次才被拒——B3a-3 的探针（副本取于 22:13 UTC，那时 `admission.rs` 还是 A4 那一版）看到的就是这个。
- 改后数（A4b）：第一次直接发布落点被拒是**第 62 次**（之后 77），第 18、32、47 次是式子先拒（会话推过再发成）。
- 原因：ckpt_cost 变大 ⇒ 保留池与切换预留每块盘多扣（窄盘 c_max 5 → 6 → 8），式子先拒的次数与位置变了，会话推抬 F 的时机跟着变，第一次落到「式子放行、落点取不到」的那一次跟着挪。
- 那份崩溃枚举用例的新值（只写在这里，没动那份文件）：`for overwrite_index in 1..=61`，第 62 次直接调发布路径断言 `PlacementRefused`；注释里「第 51 次」换成「第 62 次」；它引的 `an_admitted_overwrite_…` 那条我已钉 62（第四节第 3 条）。它是崩溃枚举（名字不带 layer0、标 ignore），我没跑。

## 六、变异行与证红

写在草稿目录（主表没改，交回之后主 agent 用 `research/scripts/apply-writer-patch.py` 的格式打）：

- `/tmp/claude-1000/impl-rev-a4b/mutations-append.tsv`：新加 10 行（六段，名字表里没有；原文在今天主工作区的 `admission.rs` 里各恰好命中一次，第六节末尾的核对原样）。
- `/tmp/claude-1000/impl-rev-a4b/mutations-replacements.tsv`：按名字整行替换实审 A4 那 5 行（表里第 974–978 行，名字不变）：主 agent 点的 974、975、977、978 原文命中 0 次；976 原文还命中，但它点名的用例 `every_empty_publish_on_one_leaf_path_per_device_…` 已换成新名字，一并替换。
  替换后各盯的判定点：974 盘数不乘进去 → 产品容量那条（跨叶那几次少扣）；975 同一处 → 纯函数单测；976 漏掉共用的根 → 产品容量那条；977 `None` 臂退回几何的高 → 三块盘那条；978 分配记录树退回树高 → 产品容量那条（每块盘一片叶的空发布也少扣）。
- 没有要删的行（`mutations-delete.txt` 不建）。

追加的 10 行（名字 → 改坏哪一处 → 点名的用例）：

| 名字 | 改坏（`admission.rs`） | 点名的用例 |
|---|---|---|
| 实审 A4b（D28 已定项 4 按最坏情况计，用户 2026-09-27 定）：每块盘至多两条叶路径退回一条（bump 游标跨叶那一次少扣） | 第 574 行常量 2 → 1 | `every_empty_publish_under_the_node_format_capacities_…`（harness） |
| 实审 A4b：每块盘至多两条叶路径退回一条（按盘分路那条纯函数的单测） | 同上 | `allocation_record_tree_term_is_two_leaf_paths_…`（core --lib） |
| 实审 A4b：中央映射树那一项退回每层一个节点（…压小容量那几格少扣） | 逐层累加改成每层加 1 | `every_empty_publish_under_capped_code_two_tree_capacities_…`（harness） |
| 实审 A4b：中央映射树那一项退回每层一个节点（逐层计那条纯函数的单测） | 同上 | `central_mapping_term_counts_changed_paths_and_splits_on_every_level` |
| 实审 A4b：中央映射树每层切出来的节点不算 | 去掉 `+ splits_at_this_level` | 同上 |
| 实审 A4b：中央映射树根切开长出来的新根不算 | `1 + splits_of_the_new_level` → `splits_of_the_new_level` | 同上 |
| 实审 A4b：新根装不下时不再切、不再长高 | 新层切分次数改成 0 | `central_mapping_term_keeps_growing_new_roots_…` |
| 实审 A4b：中央映射树叶层一次都不按会切算 | 叶层判「切不了」恒真 | `central_mapping_term_counts_…` |
| 实审 A4b：中央映射树内部层一次都不按会切算 | 内部层判「切不了」恒真 | `central_mapping_term_keeps_growing_new_roots_…` |
| 实审 A4b：压小容量下量 ckpt_cost 照旧按节点格式的容量算 | 带容量那一个忽略传进来的容量 | `every_empty_publish_under_capped_code_two_tree_capacities_…`（harness） |

证红：`bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a4b/copy-prove --memory 8G <crate> <名字…>`，副本 = 主工作区 22:47 UTC 快照（`crates/mutations.tsv` 1011 行）换上我的四份文件、表里打上 5 行替换与 10 行追加；
依次证 core 8 行、harness 新 7 行、点名那 6 条用例的既有 12 行（`run-prove.sh`，外面套 `capped.sh 4`）。每组参数脚本先跑基线、绿了才改。原样输出（`prove-red-1.out`）：

```text
实审 A4b：每块盘至多两条叶路径退回一条（按盘分路那条纯函数的单测）	抓到	allocation_record_tree_term_is_two_leaf_paths_on_each_device_plus_the_shared_root 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/001.log）
实审 A4b：中央映射树那一项退回每层一个节点（逐层计那条纯函数的单测）	抓到	central_mapping_term_counts_changed_paths_and_splits_on_every_level 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/002.log）
实审 A4b：中央映射树每层切出来的节点不算	抓到	central_mapping_term_counts_changed_paths_and_splits_on_every_level 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/003.log）
实审 A4b：中央映射树根切开长出来的新根不算	抓到	central_mapping_term_counts_changed_paths_and_splits_on_every_level 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/004.log）
实审 A4b：新根装不下时不再切、不再长高	抓到	central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/005.log）
实审 A4b：中央映射树叶层一次都不按会切算	抓到	central_mapping_term_counts_changed_paths_and_splits_on_every_level 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/006.log）
实审 A4b：中央映射树内部层一次都不按会切算	抓到	central_mapping_term_keeps_growing_new_roots_while_they_overflow_under_capped_capacities 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/007.log）
实审 A4：分配记录树那一项退回树高（按盘分路那条纯函数的单测）	抓到	allocation_record_tree_term_is_two_leaf_paths_on_each_device_plus_the_shared_root 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-core/008.log）
✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了
core exit=0
实审 A4b（D28 已定项 4 按最坏情况计，用户 2026-09-27 定）：每块盘至多两条叶路径退回一条（bump 游标跨叶那一次少扣）	抓到	every_empty_publish_under_the_node_format_capacities_fits_in_the_worst_case_checkpoint_cost_before_it 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/001.log）
实审 A4b：中央映射树那一项退回每层一个节点（不按可能改的路径数与切出来的节点计，压小容量那几格少扣）	抓到	every_empty_publish_under_capped_code_two_tree_capacities_fits_in_the_worst_case_checkpoint_cost_counted_with_them 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/002.log）
实审 A4b：压小容量下量 ckpt_cost 照旧按节点格式的容量算	抓到	every_empty_publish_under_capped_code_two_tree_capacities_fits_in_the_worst_case_checkpoint_cost_counted_with_them 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/003.log）
实审 A4（D28 已定项 4 按盘分路计，用户 2026-09-27 定）：分配记录树那一项退回树高（盘数不乘进去，两块盘少扣）	抓到	every_empty_publish_under_the_node_format_capacities_fits_in_the_worst_case_checkpoint_cost_before_it 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/004.log）
实审 A4：按盘分路漏掉共用的根（每块盘一条路径、根不算）	抓到	every_empty_publish_under_the_node_format_capacities_fits_in_the_worst_case_checkpoint_cost_before_it 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/005.log）
实审 A4：树表 0 条、写过行的那一版上分配记录树那一项退回几何的高（None 臂不按盘分路）	抓到	three_devices_rewrite_at_most_two_leaf_paths_on_each_device_and_the_shared_root 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/006.log）
实审 A4：分配记录树那一项退回树高时，每块盘一片叶、中央映射树 1 层的空发布也少扣（按盘分路之后仍少扣的只该剩两格）	抓到	every_empty_publish_under_the_node_format_capacities_fits_in_the_worst_case_checkpoint_cost_before_it 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-new/007.log）
✓ 点名 7 条：跑了 7 条，跳过 0 条，跑的都抓到了
harness-new exit=0
实二五 准入（C363 (b) 判决第四节第 2 条）：发布路径不判空间准入（式子判拒的覆盖写照样发出去）	抓到	an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/001.log）
实二五 准入：可写挂载取号之前不判空间准入（实例切换的预留拿不到照样取号）	抓到	writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/002.log）
实五 准入（D28 已定项 1 接线，用户 2026-09-25 定 A1n）：需求退回只算普通分配、不算这次写出的固定点	抓到	an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/003.log）
实二五 准入（D28 已定项 4 Σ 名单）：ckpt_cost 漏掉树表那一项	抓到	writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/004.log）
实二五 准入（D28 已定项 3）：读数里的挂载期承诺量不扣实例切换的预留	抓到	writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/005.log）
实二五 准入：只供测试的开关关不掉发布路径的准入（关掉准入的那一档照样被式子拒）	抓到	an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/006.log）
实五 P3pl（D16 已定项 1「准入」那一行：准入放行而落点取不到也推）：会话只在准入拒时推抬 F，落点被拒原样交回	抓到	an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/007.log）
实五 P3pl（D3 已定项 9 第 2 条的界）：会话只在准入拒时推抬 F，删掉之后同样大小的写回落点被拒原样交回	抓到	after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/008.log）
实五 C283（D16 已定项 1「准入」那一行）：会话被拒一律不推抬 F（准入拒、落点被拒都原样交回）	抓到	mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/009.log）
实五 C283：一次准入最多 8 次发布改成 10 次（推满的判据松了，推满仍不够那一格多推一串）	抓到	mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/010.log）
实五 挂载处推（D16 已定项 1「准入」那一行）：上一版带文件的可写挂载准入不够照旧在取号之前拒，写行之后不推（只崩了再挂的窄池挂死）	抓到	crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/011.log）
实五 C565（挂载处推满仍不够怎么收尾）：换成另一种收尾——推满仍不够就报可写挂载被拒	抓到	mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount 红了（日志 /tmp/claude-1000/impl-rev-a4b/prove-red-logs-harness-existing/012.log）
✓ 点名 12 条：跑了 12 条，跳过 0 条，跑的都抓到了
harness-existing exit=0
exit=0
```

改坏哪一行 → 哪条断言红（每条日志在 `prove-red-logs-core/`、`prove-red-logs-harness-new/`、`prove-red-logs-harness-existing/`）：

| 日志 | 行 | 改坏之后红在（证红那一刻副本里的行号，之后只做了 fmt 与 clippy 的格式改动） |
|---|---|---|
| core/001 | 追加：两条路径退回一条（纯函数单测） | `admission.rs:1072` 「盘数 × 2 × (高 − 1) + 1」 |
| core/002 | 追加：映射树每层一个节点（纯函数单测） | 两个映射单测都红：`admission.rs:1147`「叶层、内部层、长出来的每一层新根都按会切算」与 1111「每层 min(节点数, 删 + 插) + 切出来的…」 |
| core/003 | 追加：切出来的节点不算 | 同上两条都红 |
| core/004 | 追加：新根不算 | 同上两条都红 |
| core/005 | 追加：新根不再切 | 只红 1147（`…keeps_growing_new_roots…`） |
| core/006 | 追加：叶层不按会切算 | 两条都红（1111 先报） |
| core/007 | 追加：内部层不按会切算 | 只红 1147 |
| core/008 | 替换 975（盘数不乘进去，纯函数单测） | 1072 |
| harness-new/001 | 追加：两条路径退回一条 | `admission_checkpoint_cost_per_device_paths.rs:594` 「…保留池少扣」（产品容量那条，跨叶那几次） |
| harness-new/002 | 追加：映射树每层一个节点 | 同文件 635（压小容量那条） |
| harness-new/003 | 追加：压小容量照旧按节点格式容量算 | 635 |
| harness-new/004 | 替换 974（盘数不乘进去） | 594 |
| harness-new/005 | 替换 976（漏掉共用的根） | 594 |
| harness-new/006 | 替换 977（`None` 臂退回几何的高） | 同文件 817 「one-leaf：…每块盘两条路径（3 × 2 × 2 + 1 = 13）加树表 1」 |
| harness-new/007 | 替换 978（分配记录树退回树高） | 594 |
| existing/001–006 | 表里 638、639、640、641、642、644 | `second_transaction_supplement_two_admission_formula.rs` 638（式子判拒：None）、494（NotJudgedByTheTestOnlySwitch）、638、499（可用 −47 那一格）、494（AdmittedBeforeAcquisition）、661（关掉准入照样被拒） |
| existing/007–012 | 表里 760、761、762、763、764、766 | `second_transaction_admission_raises_the_floor_before_refusing.rs` 67（第 62 次覆盖写被 `Publish{PlacementRefused}` 拒）、119（长大被拒不是推满仍不够）、282（会话写报 `Publish` 不是推满仍不够）、252（取号之前短 36 那一格）、185（第 8 次崩了再挂在取号之前被拒）、226（第 8 次挂载被拒） |

同时红了哪些：prove-red 的参数都带 `-- <用例名前缀>` 过滤，同一二进制里别的用例没跑；core 那几条过滤到 `admission::tests`，11 条里只红点名那一两条（日志）。被测代码里没有 `debug_assert`。
第 7 条随机历史那 4 行（643、765、804、805）没证（第四节末尾）。

追加行与替换行的原文在今天主工作区 `admission.rs` 里的命中次数（交回前现核：逐行取第二段文件、第三段原文把 `\n` 换成换行后数，`anchor-check-final.txt` 原样）：

```text
mutations-append.tsv 1 实审 A4b（D28 已定项 4 按最坏情况计，用户 2026-09-27 定）
mutations-append.tsv 1 实审 A4b：每块盘至多两条叶路径退回一条（按盘分路那条纯函数的单测）
mutations-append.tsv 1 实审 A4b：中央映射树那一项退回每层一个节点（不按可能改的路径数与切出来的节点
mutations-append.tsv 1 实审 A4b：中央映射树那一项退回每层一个节点（逐层计那条纯函数的单测）
mutations-append.tsv 1 实审 A4b：中央映射树每层切出来的节点不算
mutations-append.tsv 1 实审 A4b：中央映射树根切开长出来的新根不算
mutations-append.tsv 1 实审 A4b：新根装不下时不再切、不再长高
mutations-append.tsv 1 实审 A4b：中央映射树叶层一次都不按会切算
mutations-append.tsv 1 实审 A4b：中央映射树内部层一次都不按会切算
mutations-append.tsv 1 实审 A4b：压小容量下量 ckpt_cost 照旧按节点格式的容量算
mutations-replacements.tsv 1 实审 A4（D28 已定项 4 按盘分路计，用户 2026-09-27 定）：分
mutations-replacements.tsv 1 实审 A4：分配记录树那一项退回树高（按盘分路那条纯函数的单测）
mutations-replacements.tsv 1 实审 A4：按盘分路漏掉共用的根（每块盘一条路径、根不算）
mutations-replacements.tsv 1 实审 A4：树表 0 条、写过行的那一版上分配记录树那一项退回几何的高（None
mutations-replacements.tsv 1 实审 A4：分配记录树那一项退回树高时，每块盘一片叶、中央映射树 1 层的空发布
```

## 七、第 4 步那几样（主工作区，交回前；`research/scripts/capped.sh` 22:52 UTC 被别的会话改坏——第 22 行语法错，preflight 两行插进了数组里——之后的命令手设同一组线程变量 `CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 SINGLEFS_THREAD_CAP=4 SINGLEFS_RANDOM_HISTORY_THREADS=4` 代替）

动到的测试二进制（`run-main-verification.sh`，整条经 `run-with-memory-cap.sh 8G`），`main-verification-summary.txt` 原样：

```text
core-lib exit=0 test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s 
admission_checkpoint_cost_per_device_paths exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.09s 
second_transaction_supplement_two_admission_formula exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.59s 
second_transaction_admission_raises_the_floor_before_refusing exit=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 89.22s 
exit=0
```

（末行 `exit=0` 是脚本的退出码。随机历史那个二进制见下面 74 号。）

`cargo fmt --all -- --check`（退出 1），按文件归并的 Diff 原样（我的四份文件 0 处；列出来的三份是别的会话在改的）：

```text
      1 Diff in /home/fy5090/code/singlefs/crates/singlefs-core/src/allocator.rs
     67 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
      2 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs
```

`cargo clippy -p singlefs-core --all-targets --all-features -- -D warnings`（加 check.sh 那 7 条 `-D clippy::…`），退出 101，报错只剩别的会话在改的 `allocator.rs:216`（`admission.rs` 0 处；改之前报过的 `admission.rs` 两处手写 div_ceil 与两处文档列表缩进已改），`clippy-core-2.log` 的报错行原样：

```text
error: manual implementation of `.is_multiple_of()`
   --> crates/singlefs-core/src/allocator.rs:216:12
error: could not compile `singlefs-core` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
error: could not compile `singlefs-core` (lib test) due to 1 previous error
```

harness 那几个测试目标带上 `-D` 编不过 core（同一处 `allocator.rs`），照 A4 的做法用同一套 lint 的 `-W` 跑（`cargo clippy -p singlefs-harness --test admission_checkpoint_cost_per_device_paths --test second_transaction_supplement_two_admission_formula --test second_transaction_admission_raises_the_floor_before_refusing -- -W …`），退出 0，告警只在 `allocator.rs`，`clippy-harness-2.log` 的定位行与末行原样：

```text
   --> crates/singlefs-core/src/allocator.rs:216:12
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
```

`cargo build --offline --all-targets`（退出 0），`build-all-targets-1.log` 末三行原样：

```text
   Compiling singlefs-core v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-core)
   Compiling singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.57s
```

登记给实现员的门禁阶段（`stage-owners.tsv` 现列 7 个；74 号设 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`、阶段内经包装），`owned-gates-summary.txt` 原样：

```text
== 33-mutation-tables.sh exit=1
      crates/mutations.tsv:436 增补 2 收口第 27 行 ④：隔离位不挡分配（is_free 不看隔离位）；C503 清隔离位之后改名的那条用例判出（盖掉之前那几槽发不出去）：原文在 crates/singlefs-core/src/allocator.rs 里命中 0 次
      crates/mutations.tsv:974 实审 A4（D28 已定项 4 按盘分路计，用户 2026-09-27 定）：分配记录树那一项退回树高（盘数不乘进去，两块盘少扣）：原文在 crates/singlefs-core/src/admission.rs 里命中 0 次
      crates/mutations.tsv:975 实审 A4：分配记录树那一项退回树高（按盘分路那条纯函数的单测）：原文在 crates/singlefs-core/src/admission.rs 里命中 0 次
      crates/mutations.tsv:977 实审 A4：树表 0 条、写过行的那一版上分配记录树那一项退回几何的高（None 臂不按盘分路）：原文在 crates/singlefs-core/src/admission.rs 里命中 0 次
      crates/mutations.tsv:978 实审 A4：分配记录树那一项退回树高时，每块盘一片叶、中央映射树 1 层的空发布也少扣（按盘分路之后仍少扣的只该剩两格）：原文在 crates/singlefs-core/src/admission.rs 里命中 0 次
    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。
== 53-format-const-placeholders.sh exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
== 74-model-differential.sh exit=1

error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
== 92-layout-checker-sync.sh exit=0
      跟上了：删掉 ROLLBACK_WITNESS_ENTRY_BYTES（原值 16，crates/singlefs-format/src/lib.rs）
      跟上了：删掉 ROLLBACK_WITNESS_TABLE_BYTES（原值 ROLLBACK_WITNESS_COUNT_BYTES + ROLLBACK_WITNESS_ENTRIES_MAXIMUM * ROLLBACK_WITNESS_ENTRY_BYTES，crates/singlefs-format/src/lib.rs）
      跟上了：删掉 ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT（原值 SYSTEM_CONFIGURATION_BYTES，crates/singlefs-format/src/lib.rs）
      跟上了：改值 SYSTEM_CONFIGURATION_BYTES：481 → 489（crates/singlefs-format/src/lib.rs）
    没抽到常量的格式定义路径 1 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：
      第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md
== 94-checker-implementation-disjoint.sh exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 283 行里没有分支与循环（`#[cfg(test)]` 标着的项 263 行不扫）
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑
== 93-feature-bits.sh exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
== 89-closeout-row27-preconditions.sh exit=77
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
    没做成探针的 2 笔：
      alloc-basis 第三轮 ②：根槽写失败推进一格重发：这条路径今天不存在，没有一处逐字文本代表「它进来了」
      alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点，没有可扫的对象
    收口表第 27 行现算 5 笔；探针与没做成探针的清单两边都没有的 1 笔：被抛弃根的根槽读不出时既不隔离也不计数
    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
exit=0
```

读法：33 号红的 6 行里 974、975、977、978 是这一轮改动带出来的，替换行在 `mutations-replacements.tsv`（主 agent 打上之后这 4 行消掉）；430、436 原文在别的会话在改的 `allocator.rs`，不归我。
74 号：随机历史二进制 22 过、2 红，红的是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，两条在同一份树的 A4 副本上也红（`impact-a4-base-1-summary.txt` 里 A4 那一版 21 过、3 红，多红的那条就是第 7 条）；第 7 条 `unit_area_wall_sampling_…_judged_…` 在 A4b 下过。89 号退 77（本次未跑，不是通过）；53、92、93、94 号绿。

受式子影响可能变的别的测试二进制（不在我的文件清单里，只跑不改），A4b 副本（copy-work，22:31 快照）`impact-a4b-1-summary.txt` 原样：

```text
admission_checkpoint_cost_per_device_paths exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.51s 
second_transaction_supplement_two_admission_formula exit=101 test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.75s 
test an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write ... FAILED
test writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted ... FAILED
second_transaction_admission_raises_the_floor_before_refusing exit=101 test result: FAILED. 6 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 94.83s 
test an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session ... FAILED
test after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes ... FAILED
test mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount ... FAILED
test crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck ... FAILED
a_floor_raise_refused_for_space_counts_as_short_of_space exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s 
a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s 
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=0 test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.58s 
entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.72s 
second_transaction_supplement_two_root_ring_turn_in_one_mount exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s 
second_transaction_supplement_two_release_checksum_quarantine exit=0 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.80s 
second_transaction_step_four_rollback exit=0 test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 131.72s 
record_checker_judges_absence_by_the_persisted_set exit=0 test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 46.30s 
crash_injection_writable_mount_after_the_crash exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.84s 
second_transaction_supplement_one_write_accounting exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s 
second_transaction_supplement_three_bad_disk_input exit=0 test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 42.00s 
second_transaction_supplement_three_crash_injection exit=101 test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 246.95s 
test crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed ... FAILED
second_transaction_supplement_three_fault_injection exit=101 test result: FAILED. 12 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 123.44s 
test fault_injection_fast_tier_returns_errors_instead_of_panicking ... FAILED
core-lib exit=0 test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s 
random-history-release exit=101 test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 48.10s 
test crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... FAILED
exit=0
```

同一份树换回 A4 的 `admission.rs`（copy-base），`impact-a4-base-1-summary.txt` 与 `impact-a4-base-2-summary.txt` 原样：

```text
admission_checkpoint_cost_per_device_paths exit=101 
second_transaction_supplement_two_admission_formula exit=101 test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.38s 
test an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write ... FAILED
test writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted ... FAILED
second_transaction_admission_raises_the_floor_before_refusing exit=101 test result: FAILED. 6 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 95.23s 
test after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes ... FAILED
test an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session ... FAILED
test mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount ... FAILED
test crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck ... FAILED
random-history-release exit=101 test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 40.00s 
test crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... FAILED
test unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval ... FAILED
exit=0
second_transaction_supplement_three_crash_injection exit=101 test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 218.12s 
test crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed ... FAILED
second_transaction_supplement_three_fault_injection exit=101 test result: FAILED. 12 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 105.91s 
test fault_injection_fast_tier_returns_errors_instead_of_panicking ... FAILED
exit=0
```

读法：A4b 副本里红的只有我四个二进制那 6 条（当时还没重钉；重钉之后主工作区那四个二进制全绿，见上）与三条基线就红的（崩溃注入 `crash_injection_small_fast_tier_…`、故障注入 `fault_injection_fast_tier_…`、随机历史那两条），三条在 A4 副本上同样红、红在同一条断言（两边日志 panicked 的文件行号与提示句相同，只差线程号与临时目录里的进程号，见 `impact-a4b-1/` 与 `impact-a4-base-2/`）。
A4 副本上 `admission_checkpoint_cost_per_device_paths exit=101` 是编不过（新用例调 A4 没有的 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`），不是断言红。

## 八、交主 agent 的设计问题（停在那一处，没自己定）

1. **「界 3」怎么数**（`after_a_file_is_truncated…`，D3（空间分配） 已定项 9 第 2 条）。A4b 下 256 槽的盘上写回被拒 3 次、建 3 次 inode、删之后第 4 次改变用户可见状态的发布（写回本身）做成。
   条款逐字「删掉的块被最近 4 个状态里更旧的 3 个钉着，要再发生 3 次这样的发布才回可分配集合」，D16（发布语义） 已定项 1 第 57 行同一句「要再发生 3 次改变用户可见状态的发布才回可分配集合」——按这两句，删与写回之间夹 3 次、写回接在后面，正好在界上；
   用例原来的读法把写回本身也数进 3 次（「删之后第 3 次改变用户可见状态的发布做成」），按那个读法 A4b 在这块盘上越界。我把第 159 行的断言改成数夹在中间的发布（≤ 3），钉 (3, 4)。
   推翻条件 / 另一种处置：主 agent 认用例原来的读法，就把第 159 行改回 `user_visible_publishes_after_the_truncation <= 3`，这条用例在 A4b 下红，那时是「按最坏情况计让窄盘越过 D3 已定项 9 第 2 条的界」，要交用户定（换几何只会把它藏起来）。
   推测（没量）：写回要等被删的块本身回来是因为保留池与切换预留每块盘多扣 26 槽，别处回收的槽不够了；A4 之前与 A4 那一版写回用得上别处回收的槽，所以夹 2 次、1 次就做成。
2. **分配记录树「每块盘两条路径」不是全部最坏情况**。规格与用户定案点名的是游标跨叶那一次，我照它写成每块盘两条。读 `crates/singlefs-core/src/allocator.rs` 的取落点与释放（`try_allocate_commit_generated`、`release_leaving_the_record_allocated_on`）推得出三种走得到的、一块盘改多于两片叶的情形（推的，这一轮的历史里一次都没量到）：
   一次空发布写的固定点单元多到开放段（64 槽）装不下、开下一个全空段，而那一段在别的叶（段不对齐叶，812 槽一叶）；开放段没有了、回落到最低空槽，每个单元各落一处；换下的上一版节点很久以前落在别的叶（例如游标回到回收空的旧段之后，旧叶的节点才被重写）。
   那时一块盘改的叶数上界是这次取的落点数加换下的落点数，而那两个数又依赖 ckpt_cost 本身（固定点）。怎么计、要不要现在计，条款没写；建议新开欠账。现在的量表与三块盘那一格都没走到三片叶。
3. **只供测试的压小容量下，产品路径的准入读不到压小的容量**。`crates/singlefs-core/src/transaction.rs` 第 4787 行（`admission_reading_before_a_publish(allocator, previous)`）与 `crates/singlefs-core/src/mount.rs` 第 2907、3230 行（`admission_reading_of_a_writable_mount`）调的是按节点格式容量算的那一个，写入口装着 `CodeTwoTreeNodeCapacities::CappedForTests` 时准入的中央映射树那一项按 294 / 143 算切分，比那一版真会写的少（行号交回前现查）。
   产品路径不受影响；压小容量的那几个测试二进制（树分裂那一族）在 4 GiB 盘上准入不紧，今天没红。要接上就在那三处把 `pool.code_two_tree_node_capacities()` 传进 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities`，那两份文件不在我的清单里。
4. **窄盘只崩了再挂从第 8 次起永远推满仍不够**（240 槽，第四节第 6 条）：用户已认「窄盘更早报 ENOSPC」，挂载照样做成（C565 那一种收尾）；我只把用例钉成现状，名字里的 never_get_stuck 指「挂载都做成」没改。要不要给这一格另起用例证「推了再判够了」还走得到（256 槽 12 次覆盖写那一格走得到，`second_transaction_supplement_two_admission_formula.rs` 第 1 条），归主 agent。
5. **大映射树上多扣得多**：逐层 min(节点数, 删 + 插) 没用「一次发布插进映射树的 key 在每棵树里是连着的一段」这一结构（新 key 带这次的 txg，在各自那棵树的 key 段里最大），叶 100、内部 10 的三层树算 41 块（单测里那一格），实写多半 3 块左右（推的）。
   用这一结构能收紧（插的只落在每棵树一段、至多两个原有节点），代价是式子要多写一条「插的 key 连成几段」的论证；这一轮没做，照最坏情况的字面先多扣。
6. **第 3 步那两次的根子（C545）还开着**：见第五节，A4b 下这一批种子走不到了，不等于合上。调查员那边要的数：种子偏移 29、第 113 / 132 步、两盘各 100 空槽全在 2 个聚簇段里。

## 九、这一轮写过的文件

- `crates/singlefs-core/src/admission.rs`（改：ckpt_cost 按最坏情况计、带容量的那一个、三个新私有函数与一个新结构、三条单测）
- `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（A4 新建、未跟踪，这一轮改写：五格产品容量含映射树 2 层、三格压小容量、三块盘两种走法；三条用例全换）
- `crates/singlefs-harness/tests/second_transaction_supplement_two_admission_formula.rs`（重钉两条）
- `crates/singlefs-harness/tests/second_transaction_admission_raises_the_floor_before_refusing.rs`（未跟踪，重钉四条）
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`：**没改**（第 7 条在 A4b 下不用改就绿；主 agent 说归调查员）
- `crates/mutations.tsv`：**没改**（追加 10 行、替换 5 行在草稿目录 `mutations-append.tsv`、`mutations-replacements.tsv`，名字见第六节）
- 草稿目录 `/tmp/claude-1000/impl-rev-a4b/`：本报告、`progress.md`、量表原样行、探针源码（`probes/`，含只在副本里加的 `probe-space-summary.diff`）、各日志与跑它们的脚本

`git diff --stat -- crates litmus` 原样（主工作区里别的会话也在改 `crates/`，分不出谁改的；我的只有上面四份，两份未跟踪、不在这里面）：

```text
 crates/mutations.tsv                               |  697 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1915 +++--
 crates/singlefs-core/src/admission.rs              |  629 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  299 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   37 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  199 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   43 +-
 crates/singlefs-core/src/recovery.rs               |  712 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  506 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2505 ++++++-
 crates/singlefs-harness/src/crash_injection.rs     |  821 ++-
 crates/singlefs-harness/src/device_log.rs          |   38 +-
 crates/singlefs-harness/src/fault_injection.rs     |  749 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             | 1122 ++-
 crates/singlefs-harness/src/lib.rs                 |   60 +-
 crates/singlefs-harness/src/model.rs               |  964 ++-
 crates/singlefs-harness/src/model_comparison.rs    |  338 +-
 crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
 crates/singlefs-harness/src/read_tally.rs          |    2 +-
 crates/singlefs-harness/src/segments.rs            |  106 +-
 .../tests/checker_known_bad_images.rs              |  986 ++-
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/common_tree_split/mod.rs                 |  138 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 .../tests/first_transaction_step_five_publish.rs   |   22 +-
 .../tests/first_transaction_step_one_mkfs.rs       |    8 +-
 .../tests/first_transaction_step_seven_layer0.rs   |  261 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 .../second_transaction_parallel_line_one_layer0.rs |   64 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...action_parallel_line_three_spill_over_layer0.rs |   36 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 ..._transaction_position_addressed_trees_layer0.rs |   30 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 .../tests/second_transaction_step_one_overwrite.rs |    3 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   55 +-
 ...second_transaction_step_three_formatted_pool.rs |  688 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   36 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  346 +-
 ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  258 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ...plement_two_instance_table_second_page_write.rs |    4 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
 ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
 ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
 ...second_transaction_supplement_two_tree_split.rs |  147 +-
 ...transaction_supplement_two_tree_split_layer0.rs |   40 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 93 files changed, 30111 insertions(+), 11251 deletions(-)
```

## 十、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交；没改 kb（D28（挂载期承诺量） 已定项 4 新句、D16（发布语义） 已定项 1 第 57 行、D5 对照表第 107 行交书记员，第二节）。
- 随机历史那条（第 7 条）没重钉、它的 4 行变异（643、765、804、805）没重证：主 agent 说归调查员；我只报 A4b 下它不用改就绿、32 个种子落点拒绝 0 次。
- 崩溃枚举 `second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs` 没改、没跑（新值在第五节末尾）。
- 分配记录树「每块盘多于两片叶」那三种情形（第八节第 2 条）是读代码推的，没造出来、没量；式子照规格只罩到每块盘两条路径。
- 产品路径的准入接压小容量（第八节第 3 条）没做：调用点在 `transaction.rs`、`mount.rs`，不在我的清单里。
- 映射树按「插的 key 连成几段」收紧（第八节第 5 条）没做。
- 登记给我之外的门禁阶段、全量 `cargo test`、层 0 都没跑；89 号退 77（本次未跑）。
- 量表产物、探针日志没进 `research/results/`：写范围只有 `crates/`、`litmus/` 与草稿目录，要留由主 agent 从草稿目录拷（`a4b-rows-final.log`、`a4b-aggregate-final.txt`、`probe-*.log`、`probe-a4-1.keep`）。
- 负载：开跑前 `ps` 看到别的会话的 `cargo test`（`crash_enumeration_sharded…`、`checker_narrow_invariants…`）与 `gate.sh`，没有 qemu / vm-bench / e152 / fio；副本各用自己的 target，没等锁。主工作区 22:29 UTC 一度编不过（别的会话在改 `allocator.rs`、`journal.rs`），22:30 编得过之后才拷的副本。
- `research/scripts/capped.sh` 22:52 UTC 被别的会话改坏（第 22 行语法错），之后手设线程变量代替，没碰那份脚本；主 agent 23:0x 说已修好。那段时间经它起的只有一条 core 的 clippy（退 2、一行没跑），已手设变量重跑（第七节那一条），不用再重跑。

## 十一、清理

删了 5 份仓副本（各含自己的 target），删前 `du -sh`：`copy-work` 4.6G、`copy-base` 3.2G、`copy-scan` 1.5G、`copy-probe` 1.2G、`copy-prove` 2.5G。草稿目录里没有别的编译目录或副本；探针源码留在 `probes/`（不是副本、不编）。
