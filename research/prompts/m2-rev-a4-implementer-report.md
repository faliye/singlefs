# 实审 A4 报告：准入 ckpt_cost 按盘分路计（代码审阅第 20 条）

时刻：2026-09-26 UTC 17:0x–18:xx（JST 2026-09-27 02:0x–03:xx）。规格 `/tmp/claude-1000/impl-rev-a4/spec.md`。

## 结论

1. **量出来了**：今天的 ckpt_cost（按树高）在每一格、每一次空发布上都少扣。两块盘、产品节点容量、每块盘改的记录落在一片叶里时，每块盘少扣 = 盘数 × (高 − 1) + 1 − 高：1 GiB（分配记录树高 2）1 块、4 GiB 与 64 GiB（高 3）2 块、1 TiB（高 4）3 块；审阅员推的「每盘约 2 块」在 4 GiB 上对。
2. **改了实现**：`crates/singlefs-core/src/admission.rs` 的 `checkpoint_cost_of_the_version_to_build_on`（第 493 行起）把分配记录树那一项从「树高」改成「盘数 × (高 − 1) + 1」（新的私有函数 `allocation_record_tree_nodes_on_one_leaf_path_per_device`，第 529 行），`Some` 与 `None` 两臂都改；中央映射树（树高）、记账树（节点数）、树表（1）三项照实写口径核过，没改。改后产品容量下每块盘落在一片叶里的空发布差全部 = 0（不少扣也不多扣）。
3. **规格的验收「差全部 ≤ 0」没达到**，还剩两种空发布少扣，条款没写怎么计，停下交主 agent（见「设计问题」第 1、2 条）：
   - 改的记录在一块盘上跨两片叶（bump 游标走过叶 61 的末槽 50343 那两次）：每块盘多一条路径，改后仍少扣 2 块（四档盘宽各 2 次）；
   - 中央映射树高于 1 层（这里只在只供测试的压小容量下量到）：一次空发布改的映射节点多于树高，每块盘一片叶的那些改后仍少扣 1–11 块（与跨叶叠在一起的 4–16 块）。
4. **改法让别的会话的 7 条既有用例红**（副本里对着同一快照的基线量过，基线绿、改后红），它们钉的是旧 ckpt_cost 下的绝对值，文件不在我的「要动的 crates 文件」里，没改，交主 agent 重钉或重造（见「改法波及的既有用例」一节）。其中一条是随机历史「空间准入判着」取样点：改后出现 2 次「推满抬 F 之后式子放行、落点仍取不到」，原先 0 次，这一条不只是重钉数（设计问题第 3 条）。
5. 三块盘走不到发布路径（mkfs 断言两块盘，`crates/singlefs-core/src/make_filesystem.rs` 第 344–348 行；映射条目只装两条位置项，`transaction.rs` 第 2771 行 `refuse_mapping_entries_that_do_not_name_two_pool_devices`），三块盘那一格只在分配记录树这一层用真分配器与真树函数量：每块盘 2 个 + 根 = 7，与新式子相等。

**什么现象会推翻结论**：量表里产品容量、每块盘一片叶的空发布出现 difference > 0（新式子少扣）；或 `ALLOCATION_RECORD_TREE_NODES_REWRITTEN`（写账用例第 126 行钉的 5）在两块 4 GiB 盘上不再是 2 × 2 + 1；或第一版改成三块盘以上时每个单元不再落每一块盘（那时「盘数」该换成副本数或落点盘数）。

## 量表（第 1 步）

装置：`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（新建）。每格两块同宽稀疏内存盘：mkfs（分配器用 `allocator_after_make_filesystem`，装着根环表，根环转过之后照常回收）→ 取号 → 暖机 → 第一个文件版本 → 覆盖写一次 → 空发布 60 次（1 TiB 30 次）。每次发布之前按那一版现算 ckpt_cost，发完按 `TransactionOutput::rewritten` 逐棵数，分配记录树按根之下节点的 `position.device` 分盘数。每次发布一行 `name=a4-checkpoint-cost …`。
列：`checkpoint_cost_by_tree_height` = 改前字面（Σ 高 + 记账节点 + 1）；`checkpoint_cost` = 产品函数此刻的值；`fixed_point_slots_per_device` = 这次写出的固定点单元在每块盘上的槽数（单元两盘各一份，都是单槽）；`difference` = 后者 − ckpt_cost（正 = 保留池少扣）。

原样行全文（各 397 行）：改前 `/tmp/claude-1000/impl-rev-a4/rows-before-fix.txt`，改后 `/tmp/claude-1000/impl-rev-a4/rows-after-fix.txt`；三块盘那一行 `/tmp/claude-1000/impl-rev-a4/row-three-devices-after-fix.txt`（改前那一次是 `checkpoint_cost_of_the_version_without_file=4`，见 `red-before-fix.log`）。下面按形状归并（`aggregate.py` 归并，次数 ×、同形状的首末发布），改后那一份：

```text
  1x two-1GiB overwrite h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=6 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=0 | overwrite@txg4
 58x two-1GiB empty h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=6 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=0 | empty#0@txg5 .. empty#59@txg64
  2x two-1GiB empty h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=6 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=2 | empty#13@txg18 .. empty#14@txg19
  1x two-4GiB overwrite h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=0 | overwrite@txg4
 58x two-4GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=0 | empty#0@txg5 .. empty#59@txg64
  2x two-4GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=2 | empty#9@txg14 .. empty#10@txg15
  1x two-64GiB overwrite h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=0 | overwrite@txg4
 58x two-64GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=0 | empty#0@txg5 .. empty#59@txg64
  2x two-64GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=8 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=2 | empty#9@txg14 .. empty#10@txg15
  1x two-1TiB overwrite h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=10 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=0 | overwrite@txg4
 28x two-1TiB empty h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=10 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=0 | empty#0@txg5 .. empty#29@txg34
  2x two-1TiB empty h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=10 alloc=d0:4,d1:4+root1 map=1 acc=1 tt=1 fixed=12 diff=2 | empty#7@txg12 .. empty#8@txg13
  1x two-4GiB-mapping-4-8 overwrite h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=9 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=2 | overwrite@txg4
 57x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=9 alloc=d0:2,d1:2+root1 map=3 acc=1 tt=1 fixed=10 diff=1 | empty#0@txg5 .. empty#59@txg64
  2x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=9 alloc=d0:3,d1:3+root1 map=4 acc=1 tt=1 fixed=13 diff=4 | empty#7@txg12 .. empty#8@txg13
  1x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=9 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=2 | empty#9@txg14
  1x two-4GiB-mapping-3-3 overwrite h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=10 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=5 | overwrite@txg4
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=10 alloc=d0:2,d1:2+root1 map=7 acc=1 tt=1 fixed=14 diff=4 | empty#0@txg5
 56x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=10 alloc=d0:2,d1:2+root1 map=5 acc=1 tt=1 fixed=12 diff=2 | empty#1@txg6 .. empty#59@txg64
  2x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=10 alloc=d0:3,d1:3+root1 map=7 acc=1 tt=1 fixed=16 diff=6 | empty#5@txg10 .. empty#6@txg11
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=10 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=5 | empty#7@txg12
  1x two-4GiB-mapping-2-2-accounting-8-3 overwrite h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=14 alloc=d0:2,d1:2+root1 map=14 acc=4 tt=1 fixed=24 diff=10 | overwrite@txg4
  2x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=14 alloc=d0:2,d1:2+root1 map=13 acc=4 tt=1 fixed=23 diff=9 | empty#0@txg5 .. empty#1@txg6
  1x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=14 alloc=d0:3,d1:3+root1 map=18 acc=4 tt=1 fixed=30 diff=16 | empty#2@txg7
 57x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=5 acc_before=4 ckpt_by_height=13 ckpt=15 alloc=d0:2,d1:2+root1 map=16 acc=4 tt=1 fixed=26 diff=11 | empty#3@txg8 .. empty#59@txg64
```

改前同一归并里 ckpt 一列等于 ckpt_by_height（4 GiB 6、64 GiB 6、1 TiB 7、1 GiB 5、映射 4-8 为 7、3-3 为 8、2-2 为 12/13），difference 相应多 1–3（按盘数 × (高 − 1) + 1 − 高）。改前原样归并：

```text
  1x two-1GiB overwrite h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=5 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=1 | overwrite@txg4
 58x two-1GiB empty h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=5 alloc=d0:1,d1:1+root1 map=1 acc=1 tt=1 fixed=6 diff=1 | empty#0@txg5 .. empty#59@txg64
  2x two-1GiB empty h_alloc=2 h_map=1 acc_before=1 ckpt_by_height=5 ckpt=5 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=3 | empty#13@txg18 .. empty#14@txg19
  1x two-4GiB overwrite h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=2 | overwrite@txg4
 58x two-4GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=2 | empty#0@txg5 .. empty#59@txg64
  2x two-4GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=4 | empty#9@txg14 .. empty#10@txg15
  1x two-64GiB overwrite h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=2 | overwrite@txg4
 58x two-64GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:2,d1:2+root1 map=1 acc=1 tt=1 fixed=8 diff=2 | empty#0@txg5 .. empty#59@txg64
  2x two-64GiB empty h_alloc=3 h_map=1 acc_before=1 ckpt_by_height=6 ckpt=6 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=4 | empty#9@txg14 .. empty#10@txg15
  1x two-1TiB overwrite h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=3 | overwrite@txg4
 28x two-1TiB empty h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:3,d1:3+root1 map=1 acc=1 tt=1 fixed=10 diff=3 | empty#0@txg5 .. empty#29@txg34
  2x two-1TiB empty h_alloc=4 h_map=1 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:4,d1:4+root1 map=1 acc=1 tt=1 fixed=12 diff=5 | empty#7@txg12 .. empty#8@txg13
  1x two-4GiB-mapping-4-8 overwrite h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=4 | overwrite@txg4
 57x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:2,d1:2+root1 map=3 acc=1 tt=1 fixed=10 diff=3 | empty#0@txg5 .. empty#59@txg64
  2x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:3,d1:3+root1 map=4 acc=1 tt=1 fixed=13 diff=6 | empty#7@txg12 .. empty#8@txg13
  1x two-4GiB-mapping-4-8 empty h_alloc=3 h_map=2 acc_before=1 ckpt_by_height=7 ckpt=7 alloc=d0:2,d1:2+root1 map=4 acc=1 tt=1 fixed=11 diff=4 | empty#9@txg14
  1x two-4GiB-mapping-3-3 overwrite h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=8 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=7 | overwrite@txg4
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=8 alloc=d0:2,d1:2+root1 map=7 acc=1 tt=1 fixed=14 diff=6 | empty#0@txg5
 56x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=8 alloc=d0:2,d1:2+root1 map=5 acc=1 tt=1 fixed=12 diff=4 | empty#1@txg6 .. empty#59@txg64
  2x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=8 alloc=d0:3,d1:3+root1 map=7 acc=1 tt=1 fixed=16 diff=8 | empty#5@txg10 .. empty#6@txg11
  1x two-4GiB-mapping-3-3 empty h_alloc=3 h_map=3 acc_before=1 ckpt_by_height=8 ckpt=8 alloc=d0:2,d1:2+root1 map=8 acc=1 tt=1 fixed=15 diff=7 | empty#7@txg12
  1x two-4GiB-mapping-2-2-accounting-8-3 overwrite h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=12 alloc=d0:2,d1:2+root1 map=14 acc=4 tt=1 fixed=24 diff=12 | overwrite@txg4
  2x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=12 alloc=d0:2,d1:2+root1 map=13 acc=4 tt=1 fixed=23 diff=11 | empty#0@txg5 .. empty#1@txg6
  1x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=4 acc_before=4 ckpt_by_height=12 ckpt=12 alloc=d0:3,d1:3+root1 map=18 acc=4 tt=1 fixed=30 diff=18 | empty#2@txg7
 57x two-4GiB-mapping-2-2-accounting-8-3 empty h_alloc=3 h_map=5 acc_before=4 ckpt_by_height=13 ckpt=13 alloc=d0:2,d1:2+root1 map=16 acc=4 tt=1 fixed=26 diff=13 | empty#3@txg8 .. empty#59@txg64
```

三块盘（只量分配记录树这一层）改后原样行：

```text
name=a4-checkpoint-cost cell=three-4GiB-allocation-record-tree-only height_allocation_record_tree=3 allocation_record_tree_changed=d0:2,d1:2,d2:2+root1 checkpoint_cost_of_the_version_without_file=8
```

记账树与树表照实写口径核过（第 2 步后半）：产品容量下每一次空发布重写的记账树节点数都等于之前那一版的节点数、树表都是 1 个（用例 `every_empty_publish_on_one_leaf_path_per_device_fits_in_the_checkpoint_cost_before_it_on_each_device_width` 逐次断言）；压小容量那几格（记账树 4 个节点）同样相等（上表 `acc_before` 与 `acc` 两列）。这两项不改。中央映射树那一项今天按树高：产品容量下这几个规模它恒 1 层、重写 1 个，对得上；高于 1 层时对不上（设计问题第 2 条）。

## 第 2 步：改了什么

- `crates/singlefs-core/src/admission.rs`
  - 模块文档第 18–20 行：「条款给的量」那一句改成按一次空发布实写计、分配记录树按盘分路。
  - `checkpoint_cost_of_the_version_to_build_on`（第 493 行起）：`Some` 臂分配记录树那一项 = `allocation_record_tree_nodes_on_one_leaf_path_per_device(从根节点头现读的高, 池里的盘数)`；`None` 臂（树表 0 条、写过行）同一个函数、高取几何的高；中央映射树、记账树、树表三项不动。文档注释写了新口径与射程（跨叶、映射树高于 1 层两格不罩）。
  - 新私有函数 `allocation_record_tree_nodes_on_one_leaf_path_per_device`（第 529 行）：`盘数 × (高 − 1) + 1`，`checked_*`，`expect` 写明依赖「根层级至少 1」。盘数取 `allocator.devices.len()`（第一版每个单元落每一块盘，与 `ReplicaCount::of_every_device_in_the_pool` 同一条理由）。
  - 新单测 `allocation_record_tree_term_is_one_leaf_path_on_each_device_plus_the_shared_root`（第 884 行）：(高, 盘数) = (2,2)(3,2)(4,2)(3,3) → 3、5、7、7。
- `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（新建，三条用例）：
  - `every_empty_publish_on_one_leaf_path_per_device_fits_in_the_checkpoint_cost_before_it_on_each_device_width`（第 411 行）：四档盘宽、产品容量，每块盘一片叶的每次空发布 difference ≤ 0；每档至少量到一次；记账树节点数与树表 1 逐次核；中央映射树 1 层。
  - `three_devices_rewrite_one_leaf_path_on_each_device_and_the_shared_root`（第 459 行）：三块 4 GiB 盘的真分配器上取 8 个单槽、下一代全释放再取 8 个，`nodes_whose_contents_changed` 数出每盘 2 个 + 根；`None` 臂的 ckpt_cost = 7 + 1 = 8。
  - `per_device_leaf_paths_still_under_reserve_only_at_leaf_boundary_crossings_and_under_a_taller_central_mapping_tree`（第 539 行）：七格（四档盘宽 + 三档压小容量）里少扣的空发布只许是「跨叶」或「映射树高于 1 层」两种，两种各至少一次（把设计问题第 1、2 条钉成可复跑的现状；条款定了怎么计、改了实现，这条跟着改）。
  - 这三条都不是崩溃枚举（不调 `enumerate_layer0` 一族），测试二进制名字不带 layer0；不标 `#[ignore]`。整条二进制 debug 下 32 秒（主工作区，`main-new-test.log`）。

## 设计问题（停下，交主 agent）

1. **跨叶那一两次怎么计**。bump 游标走过一片叶的末槽时，这次取的落点在新叶、换下的上一版落点在旧叶，每块盘改两条叶路径（高 3 时 3 个节点：两片叶 + 同一个层级 1 节点；若同时跨层级 1 的边界还会更多，这批量没走到）。量到的：1 GiB txg 18–19、4 GiB 与 64 GiB txg 14–15、1 TiB txg 12–13，各 2 次，改后每块盘仍少扣 2 块。可选的计法（推的，没量）：每块盘按两条路径计（盘数 × 2 × (高 − 1) + 1，两盘高 3 时 9，平时多扣 1 块）；或按上一版实写的分配记录树节点数取大；或承认这一格靠推空发布的余量（B = 8 次发布的预算）兜。条款没写，我没定。
2. **中央映射树高于 1 层怎么计**。换下的那几个单元的映射条目在旧叶、这次新写的条目按出生 txg 落最右的叶，一次空发布改不止一条路径；只供测试的压小容量下量到改后仍少扣 1–11 块（每块盘一片叶的）。产品容量下映射树叶装 294 条，要一版里进映射的单元多到几百个（多单元文件）才长到 2 层——产品路径走得到，这批量没在产品容量下造出来。条款写「中央映射树按树高」，与实写不符，怎么计没定。
3. **改法之后「空间准入判着」的随机历史出现落点拒绝**：`second_transaction_supplement_three_random_history` 的 `unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval`（release）改后红：`UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))` 2 次（用例要 0 次），另有 `NoSpaceAfterRaisingTheFloor(SpaceAdmissionRefused)` 90 次；同一快照的基线两者都是 0（基线日志 `baseline-logs/random-history-release.log` 第 370–397 行一段，改后 `impact-logs/random-history-release.log` 第 370–401 行一段）。即式子放行（推过抬 F 之后）而落点取不到——保留池变大之后这批小盘历史走到了新的状态。原因没查：`AdmissionReading::of_allocator` 文档注释里记着的「抬 F 回收的槽扣住到 F 生效之前，计数上已回到空闲、分配器却不发它们，式子里没有一项装它们」可能是其一（推测，没量）。
4. **窄盘上挂载从某次起一直「推满仍不够」**：`second_transaction_admission_raises_the_floor_before_refusing` 的 `crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck`（两块单元区 240 槽的盘、只崩了再挂 15 次）改后第 9 次起 7 次都是「推满仍不够」（原来是第 10、11 次推满仍不够、第 12 次推了够、之后取号之前就够）。挂载照样做成（C565 那一种收尾），但 240 槽的盘在新的切换预留下再也回不到「够」：每块盘的切换预留 (N_switch + 1) × R × c_max 随 c_max 5 → 6 涨 12 槽、保留池涨 1 槽。这是准入不少扣的直接后果；窄盘上「永远推满仍不够」要不要接受、用例名里的 never get stuck 怎么改，归主 agent。

## 改法波及的既有用例（文件不在我的清单里，没改；交主 agent 重钉或重造）

做法：主工作区快照拷进副本 `copy-fix`（17:20 UTC），副本里施加改法跑下面这些测试二进制（`run-impact.sh`，汇总 `impact-summary.txt`）；再把副本里 `admission.rs` 换回改前、`touch` 之后对红了的那几个二进制跑基线（`run-baseline.sh`，汇总 `baseline-summary.txt`）。基线同样红的不算我的（别的会话在改 `mount.rs` / `transaction.rs` 等，见「没做什么」）。

改后红、基线绿（我的改法造成，7 条）：

| 测试二进制 | 用例 | 断言处 | 改前钉的 | 改后量到的 |
|---|---|---|---|---|
| second_transaction_supplement_two_admission_formula | an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write | 第 675 行 | 每块盘可用 8 槽（131072）< 需求 12 槽 | 可用 −5 槽（−81920）：普通分配那 6 槽也装不下了，「普通分配装得下、固定点装不下」这一格的判别力没了，要换几何重造 |
| 同上 | writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_the_row_publish_and_is_admitted | 第 498 行 | 取号之前每块盘 −8 槽（−131072） | −21 槽（−344064）；后面的断言没走到 |
| second_transaction_admission_raises_the_floor_before_refusing | an_admitted_overwrite_whose_data_unit_finds_no_slot_raises_the_floor_and_is_published_in_the_session | 第 79 行 | 第一次被落点拒、推过抬 F 再做成的是第 51 次 | 第 35 次（列表 [35, 50]） |
| 同上 | after_a_file_is_truncated_the_same_size_is_written_back_within_three_user_visible_publishes | 第 145 行 | (写回被拒 2 次, 第 3 次发布做成) | (1, 2) |
| 同上 | mount_still_short_after_the_publishes_of_one_admission_is_made_and_its_writes_report_no_space_until_a_normal_unmount | 第 250 行 | 第 10 次取号之前每块盘短 7 槽 | 短 14 槽；后面的断言没走到 |
| 同上 | crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck | 第 198 行 | 9 次够、推满不够 ×2、推了够、够 ×3 | 8 次够、之后 7 次都推满仍不够（设计问题第 4 条） |
| second_transaction_supplement_three_random_history（release） | unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval | 第 496 行 | 落点拒绝 0 次 | 2 次（设计问题第 3 条） |

⚠️ 这 7 条此刻在主工作区里是红的（我的改法已经落进主工作区的 `admission.rs`）。同时在改 core 的 A1b 会话要是跑到这几个二进制，会看到这几条红——不是它的改动造成的。

基线就红、改后同样红的（不归我，照列）：`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount` 3 条（屏障次序，`Barrier` 多了一道）、`second_transaction_supplement_three_crash_injection` 的 `crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed`、`second_transaction_supplement_three_fault_injection` 的 `fault_injection_fast_tier_returns_errors_instead_of_panicking`、`second_transaction_supplement_three_random_history` 的 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`。

改后绿（副本里跑过）：core `--lib` 124 条（那时还没加新单测）、新测试 3 条、`a_floor_raise_refused_for_space_counts_as_short_of_space`、`a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is`、`entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write`、`second_transaction_supplement_two_root_ring_turn_in_one_mount`、`second_transaction_supplement_two_release_checksum_quarantine`、`second_transaction_step_four_rollback`、`record_checker_judges_absence_by_the_persisted_set`、`crash_injection_writable_mount_after_the_crash`、`second_transaction_supplement_one_write_accounting`（钉 5 个分配记录树节点的那一份，与新式子一致）、`second_transaction_supplement_three_bad_disk_input`。

改后汇总原样（`impact-summary.txt`）：

```text
core-lib exit=0 test result: ok. 124 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s 
admission_checkpoint_cost_per_device_paths exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.76s 
second_transaction_supplement_two_admission_formula exit=101 test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.58s 
second_transaction_admission_raises_the_floor_before_refusing exit=101 test result: FAILED. 6 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 88.99s 
a_floor_raise_refused_for_space_counts_as_short_of_space exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s 
a_block_device_error_while_raising_the_floor_for_space_is_handed_up_as_is exit=0 test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s 
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=101 test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.22s 
entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write exit=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.76s 
second_transaction_supplement_two_root_ring_turn_in_one_mount exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.59s 
second_transaction_supplement_two_release_checksum_quarantine exit=0 test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.87s 
second_transaction_step_four_rollback exit=0 test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 108.73s 
record_checker_judges_absence_by_the_persisted_set exit=0 test result: ok. 6 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 11.15s 
crash_injection_writable_mount_after_the_crash exit=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.67s 
second_transaction_supplement_one_write_accounting exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s 
second_transaction_supplement_three_bad_disk_input exit=0 test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 41.31s 
second_transaction_supplement_three_crash_injection exit=101 test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 180.20s 
second_transaction_supplement_three_fault_injection exit=101 test result: FAILED. 12 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 99.90s 
random-history-release exit=101 test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 35.57s 
```

基线汇总原样（`baseline-summary.txt`）：

```text
second_transaction_supplement_two_admission_formula exit=0 test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.69s 
second_transaction_admission_raises_the_floor_before_refusing exit=0 test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 85.55s 
rollback_floor_written_into_the_system_configuration_first_and_normal_unmount exit=101 test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.84s 
test failing_to_write_the_new_floor_into_the_second_system_configuration_publishes_no_root_and_keeps_the_first_device_carrying_it ... FAILED
test normal_unmount_raises_the_floor_to_the_current_txg_with_marked_empty_publishes_on_every_device_after_writing_the_system_configuration ... FAILED
test raising_the_floor_writes_the_new_floor_into_every_system_configuration_behind_a_barrier_before_the_first_root ... FAILED
second_transaction_supplement_three_crash_injection exit=101 test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 180.12s 
test crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed ... FAILED
second_transaction_supplement_three_fault_injection exit=101 test result: FAILED. 12 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 102.53s 
test fault_injection_fast_tier_returns_errors_instead_of_panicking ... FAILED
random-history-release exit=101 test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 34.93s 
test crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... FAILED
```

⚠️ 门禁 59 号：`crates/mutations.tsv` 里点名上面 7 条用例为「必须红」的变异行有 16 行（第 638–644、760–766、804–805 行，`awk -F'\t' 'NF==6 && $6 ~ /…/'` 现数），这 7 条改后基线就红，59 号复跑这 16 行之前要先重钉那几条用例。

## 第 3 步：D28 已定项 4 的新句（我写不了 kb，交书记员）

`.claude/kb/decisions/28-挂载期承诺量.md` 第 106–107 行现文：「**形态**：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算；……」与「  - 分配记录树、中央映射树进 Σ，按树高；」。建议改成：

- **形态**（第 106 行首句换成）：ckpt_cost = 一次空发布实写的固定点单元数，按当时的结构现算：Σ（每棵记录树一次发布改写的节点数）+ 记账树每发布的节点数 + 1（树表），每次发布按当时的树高重算。分配记录树按盘分路计：盘数 × （当前的高 − 1）+ 1——根之下每块盘一条从叶到根之下那一层的路径，根一个（第一版每个单元落池里每一块盘，一次发布在每块盘上都改记录；式子里的「盘数」是池里的盘数）；中央映射树按一条从叶到根的路径计：当前的高。（第 106 行其余照旧：每棵树当前的高 = 根节点码 2 头里现读的层级 + 1……）
- 第 107 行换成：「  - 分配记录树进 Σ，按盘分路：盘数 × (高 − 1) + 1；中央映射树进 Σ，按树高；」
- **射程**（第 114 行后补一句）：按盘分路那一项罩的是「这次改的记录在每块盘上落在同一片叶里、中央映射树 1 层」的空发布：第一版两块 4 GiB 盘实写 8 块、ckpt_cost 8。改的记录在一块盘上跨两片叶（bump 游标走过叶的末槽那一两次，每块盘多一条路径），或中央映射树高于 1 层（一次改不止一条路径）时，实写多于 ckpt_cost（量表：`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`，跨叶每块盘少 2 块），这两格怎么计没定。三块盘以上两份落哪两块盘定下之后（C126），「盘数」跟着换成每个单元落的盘数。
- **依据**补一条：用户定案 2026-09-27「改条款按盘分路计并改实现」（代码审阅第 20 条，`records/2026-09-27-代码审阅38条去向.md`「用户定案」一节第 20 行）；量表同上，按树高那一版在 1 GiB / 4 GiB 与 64 GiB / 1 TiB 两盘上每块盘少扣 1 / 2 / 3 块。
- 这一格还欠的（建议新开欠账号，名字由主 agent 起）：跨叶与中央映射树高于 1 层两格怎么计（设计问题第 1、2 条）。

D16（发布语义） 已定项 1「准入」那一行（`.claude/kb/decisions/16-发布语义.md` 第 42 行）：行里只有「可分配 = min(可再分配 + 活元数据 − 保留池, `df`)」与推抬 F 的规则，没有 ckpt_cost 的式子，**这一行不用改**。同一节第 43 行「`df`」那一格的「保留池 = 10 + 7 c_max」按 D28 已定项 4 现算的 c_max 取，式子也不用改；第 57 行「c_max 按现算取，第一版池规模 9 块 ⇒ 保留池 73 块」是 E148 的模型数（D28 已定项 4 射程写着「它的 9 块是模型数」），按盘分路之后第一版两块 4 GiB 盘的 ckpt_cost 实测 8（量表），要写实现取值的话是 10 + 7 × 8 = 66 块——改不改由书记员照 D28 的新句定。

门禁 51 号（`.claude/gate.d/51-admission-terms-covered.sh`）只读 `.claude/kb/decisions/`，只判式子的项与对照表逐一对得上、右列是「第 N 项」或「**例外**：」加理由（文件头第 6–14 行），我没动 kb，它不会因这次改动红。对照表那一行的措辞过时了，建议一并改：`.claude/kb/decisions/05-快照-空间记账机制.md` 第 107 行「| checkpoint 保留池 | **例外**：挂载期承诺量的池级成员，按当时各记录树的高之和 + 记账节点现算（D28（挂载期承诺量） 已定项 4），……」里的「按当时各记录树的高之和 + 记账节点现算」换成「按一次空发布实写的固定点现算（分配记录树按盘分路、中央映射树按树高、记账节点、树表）」。

## 变异行（`/tmp/claude-1000/impl-rev-a4/mutations-append.tsv`，5 行，六段，名字表里没有，原文在今天的 `admission.rs` 里各恰好命中一次；主 agent 追加）

1. `实审 A4（D28 已定项 4 按盘分路计，用户 2026-09-27 定）：分配记录树那一项退回树高（盘数不乘进去，两块盘少扣）` → `every_empty_publish_on_one_leaf_path_per_device_fits_in_the_checkpoint_cost_before_it_on_each_device_width`
2. `实审 A4：分配记录树那一项退回树高（按盘分路那条纯函数的单测）` → `allocation_record_tree_term_is_one_leaf_path_on_each_device_plus_the_shared_root`（`-p singlefs-core --lib -- admission::tests`）
3. `实审 A4：按盘分路漏掉共用的根（每块盘一条路径、根不算）` → 同第 1 条那一用例
4. `实审 A4：树表 0 条、写过行的那一版上分配记录树那一项退回几何的高（None 臂不按盘分路）` → `three_devices_rewrite_one_leaf_path_on_each_device_and_the_shared_root`
5. `实审 A4：分配记录树那一项退回树高时，每块盘一片叶、中央映射树 1 层的空发布也少扣（按盘分路之后仍少扣的只该剩两格）` → `per_device_leaf_paths_still_under_reserve_only_at_leaf_boundary_crossings_and_under_a_taller_central_mapping_tree`

五行全证过（每条新测试至少一行），没有留给 59 号的。

## 证红（第 3 步）

- **改前整条二进制**：主工作区在施加改法之前跑整个 `admission_checkpoint_cost_per_device_paths` 二进制，三条全红（`red-before-fix.log`，`test result: FAILED. 0 passed; 3 failed`）：第一条红在第 446 行（`under_reserved` 非空，首行 two-1GiB empty#0 difference=1），第三条红在第 563 行（「每块盘一片叶、中央映射树 1 层的空发布不少扣」；那一次之后 fmt 合了一行，今天是第 562 行），三块盘那条红在第 526 行（left `MetadataBlocks(4)` right `MetadataBlocks(8)`）。改前没有新单测（被测函数是这一轮新加的），它由第 2 行变异证。
- **变异**：`bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a4/copy-fix --memory 8G <crate> <变异名…>`（副本 17:20 UTC 快照，`crates/mutations.tsv` 末尾接上那 5 行；基线由脚本先跑、绿）。原样输出：

```text
实审 A4（D28 已定项 4 按盘分路计，用户 2026-09-27 定）：分配记录树那一项退回树高（盘数不乘进去，两块盘少扣）	抓到	every_empty_publish_on_one_leaf_path_per_device_fits_in_the_checkpoint_cost_before_it_on_each_device_width 红了（日志 /tmp/claude-1000/impl-rev-a4/prove-red-logs/001.log）
实审 A4：按盘分路漏掉共用的根（每块盘一条路径、根不算）	抓到	every_empty_publish_on_one_leaf_path_per_device_fits_in_the_checkpoint_cost_before_it_on_each_device_width 红了（日志 /tmp/claude-1000/impl-rev-a4/prove-red-logs/002.log）
实审 A4：树表 0 条、写过行的那一版上分配记录树那一项退回几何的高（None 臂不按盘分路）	抓到	three_devices_rewrite_one_leaf_path_on_each_device_and_the_shared_root 红了（日志 /tmp/claude-1000/impl-rev-a4/prove-red-logs/003.log）
实审 A4：分配记录树那一项退回树高时，每块盘一片叶、中央映射树 1 层的空发布也少扣（按盘分路之后仍少扣的只该剩两格）	抓到	per_device_leaf_paths_still_under_reserve_only_at_leaf_boundary_crossings_and_under_a_taller_central_mapping_tree 红了（日志 /tmp/claude-1000/impl-rev-a4/prove-red-logs/004.log）
✓ 点名 4 条：跑了 4 条，跳过 0 条，跑的都抓到了
实审 A4：分配记录树那一项退回树高（按盘分路那条纯函数的单测）	抓到	allocation_record_tree_term_is_one_leaf_path_on_each_device_plus_the_shared_root 红了（日志 /tmp/claude-1000/impl-rev-a4/prove-red-logs/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
```

  改坏哪一行 → 哪条断言红：第 1、5 行（`admission.rs` 第 536 行 `devices_in_the_pool` 换成 `1u64`）→ 新测试第 446 行 / 第 562 行；第 2 行（同一处）→ `admission.rs` 第 885 行，left `[2, 3, 4, 3]`；第 3 行（第 538 行 `checked_add(1)` 换成 `checked_add(0)`）→ 新测试第 446 行；第 4 行（`None` 臂第 512–515 行换回几何的高）→ 新测试第 526 行。证红的参数都带 `-- <用例名>` 过滤，同一二进制里别的用例没跑（被过滤掉）；核心 lib 那一行过滤到 `admission::tests`，9 条里只红这一条（`prove-red-logs/001.log`）。日志注意：核心那次跑把 `prove-red-logs/001.log` 覆盖了，第 1 行变异那次的日志没留；它红在哪一行与改前整条二进制那次相同（第 446 行，改前的代码就是这条变异的样子）。被测代码里没有 `debug_assert`。

## 第 4 步那几样的原样输出

动到的测试二进制，主工作区（改法落进去之后，`run-main-verification.sh` 经 `run-with-memory-cap.sh 8G`、`capped.sh 4`；另外受影响的那一批在副本里跑，见上一节）：

```text
-p singlefs-core --lib:                                  test result: ok. 125 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
-p singlefs-harness --test admission_checkpoint_cost_per_device_paths:  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.92s
```

`cargo fmt --all -- --check`（退出 1；Diff 全在别的会话的 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`，我的两份文件按包 `cargo fmt -p … -- --check` 各 0 处）：

```text
     67 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```

`cargo clippy`（check.sh 那一套 -D）：`-p singlefs-core --all-targets --all-features` 退出 0；带上 `-p singlefs-harness` 时退出 101，红在别的会话在改的 `crates/singlefs-checker/src/walk.rs`（第 231、1453、4633、4641、4668 行，4 个 error），harness 编不到。新测试目标改用同一套 lint 的 `-W` 跑（`cargo clippy -p singlefs-harness --test admission_checkpoint_cost_per_device_paths`），我的文件 0 处告警：

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
error: could not compile `singlefs-checker` (lib) due to 4 previous errors
0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.10s
```

`cargo build --offline --all-targets`（退出 0）：

```text
   Compiling singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.20s
```

登记给实现员的门禁阶段（`stage-owners.tsv` 现列 7 个；74 号设 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`、阶段内经包装），各自末几行与退出码原样（`owned-gates-summary.txt`）：

```text
== 33-mutation-tables.sh exit=1
      crates/mutations.tsv:874 实审 B1 第 8 条：checker 映射 key 的类标签不在登记表里也判成立：原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次
      crates/mutations.tsv:877 实审 B1 第 8 条：checker 不拿映射 key 的类标签比被指单元头里的：原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次
      crates/mutations.tsv:878 实审 B1 第 8 条：checker 不拿映射 key 的出生身份比被指单元头里的：原文在 crates/singlefs-checker/src/walk.rs 里命中 0 次
    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。
== 53-format-const-placeholders.sh exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1�
== 74-model-differential.sh exit=1
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     → 怎么办：单跑看细节（经内存包装，上限同这一道）：
                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture
                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。
== 92-layout-checker-sync.sh exit=0
      跟上了：删掉 ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT（原值 SYSTEM_CONFIGURATION_BYTES，crates/singlefs-format/src/lib.rs）
      跟上了：改值 SYSTEM_CONFIGURATION_BYTES：481 → 489（crates/singlefs-format/src/lib.rs）
    没抽到常量的格式定义路径 1 条（第 ④ 条对它们没有对象可判，只受第 ②③ 条管）：
      第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）：.claude/kb/layout/02-second-txn.md
== 94-checker-implementation-disjoint.sh exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）�
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/
== 93-feature-bits.sh exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INC
== 89-closeout-row27-preconditions.sh exit=77
      alloc-basis 第三轮 ②：根槽写失败推进一格重发：这条路径今天不存在，没有一处逐字文本代表「它进来了」
      alloc-basis 第三轮 ③：扣住配今天的 checker 放过「扣住位被撤掉」那份坏镜像：扣住位只住内存、盘上没有落点，没有可扫的对象
    收口表第 27 行现算 5 笔；探针与没做成探针的清单两边都没有的 1 笔：被抛弃根的根槽读不出时既不隔离也不计数
    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
```

门禁读法：33 号红的 11 行（第 308、309、310、319、494、759、865、870、874、877、878 行）原文都在 `crates/singlefs-checker/src/walk.rs`，是别的会话在改的文件，`admission.rs` 的行一条都没红（`grep -c admission` 该日志 0）；我的 5 行没进 `crates/mutations.tsv`，它没判到。74 号红的 3 条 = 基线就红的 2 条（`crash_recovery_abandoning_…closeout_row_43`、`random_histories_fast_tier_…`）+ 我的改法造成的 1 条（`unit_area_wall_sampling_…space_admission_judged…`，设计问题第 3 条）。89 号退 77（本次未跑，不是通过）。53、92、93、94 号绿。命名纪律（`.claude/scripts/naming-lint.sh`）全仓 204 处，我的两份文件 0 处（改完之后重跑，`naming-lint-final.log` 里 `grep -c admission` 为 0；末行「✗ 204 处名字不合命名纪律（查了 285 个 .rs 文件、70577 个声明的名字）」）。

## 这一轮写过的文件

- `crates/singlefs-core/src/admission.rs`（改）
- `crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`（新建）
- `crates/mutations.tsv`：没改（照派发提示，5 行在 `/tmp/claude-1000/impl-rev-a4/mutations-append.tsv`，名字见上）。
- 草稿目录 `/tmp/claude-1000/impl-rev-a4/`：本报告、`progress.md`、量表原样行、各日志与跑它们的脚本。

`git diff --stat -- crates litmus` 原样（主工作区里别的会话也在改 `crates/`，这一份分不出谁改的；我的只有上面两份，新建那份未跟踪、不在这里面）：

```text
 crates/mutations.tsv                               |  638 +-
 crates/singlefs-checker/src/image.rs               |  216 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/position_addressed.rs  |    2 +-
 crates/singlefs-checker/src/walk.rs                | 1911 +++--
 crates/singlefs-core/src/admission.rs              |  350 +-
 crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
 crates/singlefs-core/src/allocator.rs              |  132 +-
 crates/singlefs-core/src/code_two_tree.rs          |    4 +-
 crates/singlefs-core/src/extent_tree.rs            |    2 +-
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/journal.rs                |   24 +-
 crates/singlefs-core/src/lib.rs                    |    2 +-
 crates/singlefs-core/src/make_filesystem.rs        |  179 +-
 crates/singlefs-core/src/mount.rs                  | 2557 +++++--
 crates/singlefs-core/src/mounted_read.rs           |   22 +-
 crates/singlefs-core/src/recovery.rs               |  493 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 -
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  277 +-
 crates/singlefs-core/src/transaction.rs            |  433 +-
 crates/singlefs-core/src/write_request_split.rs    |   51 +-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
 .../src/bin/e158_root_choice_repair.rs             | 6818 +++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |  175 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               | 2498 ++++++-
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
 .../tests/first_transaction_step_seven_layer0.rs   |  233 +-
 .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
 .../tests/parallel_line_one_sequential_write.rs    |    4 +-
 ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
 ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
 ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
 .../tests/second_transaction_step_five_reuse.rs    |  532 +-
 .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
 ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
 ...second_transaction_step_three_formatted_pool.rs |  567 +-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
 ...transaction_supplement_three_crash_injection.rs |  658 +-
 ...transaction_supplement_three_fault_injection.rs |  412 +-
 ..._transaction_supplement_three_random_history.rs |  434 +-
 ...transaction_supplement_two_admission_formula.rs |  342 +-
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
 ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
 .../system_configuration_mutability_classes.rs     |   35 +-
 87 files changed, 29059 insertions(+), 11053 deletions(-)
```

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交；没改 `crates/mutations.tsv`（5 行在草稿目录）。
- 规格验收「差全部 ≤ 0」没达到：跨叶与中央映射树高于 1 层两格停下交回（设计问题第 1、2 条），没自己定计法。
- 被改法波及的 7 条既有用例（两个准入二进制 6 条、随机历史 1 条）没重钉：文件不在我的「要动的 crates 文件」里。它们此刻在主工作区是红的。
- 设计问题第 3 条（准入判着时推满之后落点仍取不到 2 次）原因没查。
- 三块盘没走发布路径（mkfs 与映射条目都只收两块盘），只在分配记录树这一层量。
- 中央映射树高于 1 层只在只供测试的压小容量下量到，产品容量下（要几百个进映射的单元）没造。跨层级 1 边界（一次同时跨叶与层级 1 节点）这批历史没走到。
- 量表产物没进 `research/results/`：我的写范围只有 `crates/`、`litmus/` 与草稿目录。原样行在 `/tmp/claude-1000/impl-rev-a4/rows-before-fix.txt`、`rows-after-fix.txt`、`row-three-devices-after-fix.txt`（跑法：`cargo test -p singlefs-harness --test admission_checkpoint_cost_per_device_paths -- --nocapture --test-threads 1`，经内存包装），要留就由主 agent 拷进去。
- 门禁 51 号没跑（只读 kb，我没动 kb）；登记给我之外的门禁阶段、全量 `cargo test`、层 0 都没跑。
- 别的会话同时在改 `crates/`：`rollback_floor_…`、崩溃注入、故障注入、随机历史快档在同一快照的基线上就红，没查、没修。
- 负载：开跑前 `ps` 没看到 qemu / vm-bench / e152 / fio；中途看到别的会话的 `cargo test`（崩溃注入、随机历史、e158）同时在跑，没等锁（我的副本用自己的 target）。

清理：删了仓副本 `/tmp/claude-1000/impl-rev-a4/copy-fix`（删前 `du -sh` 4.8G，含它自己的 target）。草稿目录里没有别的编译目录或副本。
