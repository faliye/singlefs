# E142 第 15 次跑第五段：Q142.1v 变体臂、组合臂、P1-P3/P5/P6、G4 —— 执行员报告

开工读跑前登记 `research/prompts/e142-r15-prereg.md`（第五节 5.1/5.2 臂与变体、第六节 Q142.1v 判据、第十一节 S3/S4 停机、第八节 8.2 G4）与上一段报告
`research/prompts/e142-r15-step234-runner-report.md`（14 equal / 15 unequal 的手工诊断，未经变体臂验证）。这一段做的是上一段
「岔路表」里点名还差的：Q142.1v（8 条注册变体 + 组合臂 N[C]）、阳性对照 P1–P3、P6、几何敏感性 G4。

## 一、单测数（命令数出来）

```
$ cd research && cargo test -p e7-index-bench --bin e142-first-txn-dry-run 2>&1 | tail -1
test result: ok. 69 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 12.51s
$ cargo test -p singlefs-harness --bin e142_first_transaction_write_dump_one_device 2>&1 | tail -1
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

模型 69 通过（上一段 61 + 这一段新增 8 条：`gap_readings_default_is_the_frozen_arm_n_main_reading`、
`assemble_allocation_cells_dense_empty_zero_keeps_every_position_and_zero_fills_the_empty_one`、
`assemble_allocation_cells_sparse_by_key_drops_empty_positions`、
`assemble_allocation_cells_dense_with_key_writes_the_position_start_key_into_empty_cells`、
`allocation_root_key_end_reading_variants_change_only_the_root_largest_key`、
`extent_upper_leaf_positional_key_range_covers_the_four_registered_combinations`、
`emit_variant_comparison_reports_matched_and_equal_flags_correctly`、
`run_variant_window_writes_with_default_readings_has_twenty_nine_writes_matching_frozen_region_names`）、
1 个 `#[ignore]`（层 0 整轮，Q142.9 附带够判后不跑，未变）。新建的 G4 导出 bin（重命名后的
`e142_first_transaction_write_dump_one_device`）5 通过（沿用旧导出 bin 的既有 4 条测试 + 新增
`one_device_parameters_puts_every_region_on_device_zero`）。`cargo build`/`cargo check` 两个二进制零警告；
`cargo clippy -p e7-index-bench --bin e142-first-txn-dry-run --profile test -- -D warnings` 这一段新加的代码
过了（修了 2 条：`derivable_impls` 改用 `#[derive(Default)]` + `#[default]`、`publish_first_file` 加
`#[allow(clippy::too_many_arguments, reason = "...")]`）；同一条命令仍报 21 处**这一段之前就存在**的 clippy 违规
（`unnecessary_cast`、`doc_lazy_continuation`、`wrong_self_convention`、`manual_is_multiple_of`、
`collapsible_match`、`type_complexity`、`assertions_on_constants`、`chunks_exact_to_as_chunks`），逐条核过行号
（40、102、121、443、493、591、1295、1696、1781、1860、1898、3259、4162、4199、4404、4423、5347、5899、5971、7075）
都落在这一段没碰过的代码里，不是这一段引入的，按「顺手的整理单独提交」不在这一段修，如实记（详见「六、没做什么」）。
新建的 G4 bin 过 `cargo clippy --profile test -- -D warnings` 零告警。

## 二、变异三个数与分类

变异表 `research/mutations/e142_first_transaction_dry_run.tsv`：109 → **123** 行（这一段新增 **M113–M126**，共 14 条，
覆盖 `assemble_allocation_cells` 三个读法分支、β 四种读法、`extent_upper_leaf_positional_key_range` 的 γ/η 四个分支、
`emit_variant_comparison` 的未配对分支、`GapReadings` 的 `#[default]` 选点、`run_variant_window_writes` 的窗口切点）。
另外修了 **2 条上一段遗留、被这一段的 α/β 重构改坏锚点的变异**（不算新增，锚点腐化，`mutation-sampling.md` 第七类）：
`M101_internal_cell_index_off_by_one`（原锚点 `cells[cell_within_level_one as usize] = ...` 那一行在把层级 1 节点的格改成
`assemble_allocation_cells` 之后不再存在，改锚到新的 `let child = if position == cell_within_level_one { ... }` 那一行，
风险原样保留：哪一格拿到孩子指针的下标错一位）；`M68_run_full_pipeline_ignores_requested_change_count`（原锚点是
`run_full_pipeline` 里调 `publish_first_file` 那一行，这一段给 `publish_first_file` 加了 `gap_readings` 参数、call site
文本跟着变，改锚只加了 `, GapReadings::default())`，不改判据本身）。

跑了一轮（`bash research/scripts/mutate.sh e142-first-txn-dry-run e7-index-bench/src/bin/e142_first_transaction_dry_run.rs mutations/e142_first_transaction_dry_run.tsv`）：

```
$ grep -c '^✅' <日志>
123
$ grep -c '^💥\|^⚠️\|^⏱' <日志>
0
$ tail -1 <日志>
已还原，基线仍全绿
```

**123/123 抓到、0 无效、0 没红**，收尾「已还原，基线仍全绿」；日志
`research/results/e142_first_transaction_dry_run-mutate-2026-09-25-q142-1v-full-table.log`
（sha256 `0077f68cec40ced96f6cc7db63504f785597649243407cf16e34bede4fc6d055`）。
跑前用脚本核对全部 123 条锚点在当前源码里逐条唯一命中一次（`src.count(original) == 1`），跑后 `sha256sum` 确认工作区
源文件与冻结值一致，没有被变异过程遗留改动。

crates 侧新导出 bin（`e142_first_transaction_write_dump_one_device`）不带独立变异表——它是只驱动/只观测装置，
判定逻辑全在模型那一侧；`crates/mutations.tsv` 不追加这一条（这个 bin 除了固定几何参数与既有导出 bin 的重复逻辑，
没有自己的判定分支）。

## 三、产物路径与完成标记

| 文件 | 行数 | 完成标记 |
|---|---|---|
| `research/results/e142-first-txn-dry-run-2026-09-25-q142-1v-combined.out`（模型输出 922 行 + crates 导出 35 行，`replay.sh:157` 改指它） | 957 | 两段各自的 `E7RESULT name=done` 都在（`emitted=922`、`emitted=35`） |
| `crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs`（G4 新建入库装置，只驱动只观测） | 267 | 跑起来在 `crates/singlefs-core/src/make_filesystem.rs:194` panic（预期内，见「五、G4」） |

`research/results/e142-r15-crates-write-dump-2026-09-25.out`（crates 导出，02:55:56 UTC 采集，上一段落盘）**不重采、原样复用**：现场重跑
`cargo run --release -p singlefs-harness --bin e142_first_transaction_write_dump` 与它逐字节相同（`diff` 命中 0 行），确认 `crates/` 这一侧
对第一个事务窗口仍是同一份字节，重采不会改变任何结论。

冻结 sha256：模型源码 `32f77ab5d9d4cb0751666124fca828a30730a24dda5719dfb0af69afeeb499cc`（比上一段的
`8e7b77b1afb2cb3a994d5c994793e3466dffc22741205a5b015e1b4834cdb544` 多了 α/β 变体开关、Q142.1v 驱动、P2/P5 阳性对照、
extent_root 的 γ/η 独立校验，共 8 条新单测）；变异表 `c049a31cc75c6ff9d369634a4d227c2f3e407d98f443d6f49cfb095e7ac1c9bf`
（109 → 123 行）；G4 装置源码 `aafc980580f3d5612868d60012b84097615d2400cdd2827854f57c8783d10299`。

## 四、核心结果：Q142.1v 变体归因（问题单第 1 行的归因，不改「全等/不等」本身）

主读法 P 判定不变：**仍是「不等」**（14 equal / 15 unequal，与上一段一致，`name=impl_bytes_equal_summary` 逐字节相同）。
这一段做的是上一段没做的归因（够判点：Q142.1v 判完）。

**α（分配记录树内部/根节点一格怎么编）**：`name=gap_reading_matches_crates gap=alpha readings_whose_direct_fields_equal_crates=SparseByKey`——
crates 用的是 5.2 乙（稀疏，只写有孩子的格）。判据：`allocation_internal_of_device_0` 整段字节在 α=SparseByKey 变体下与 crates 完全相等
（`name=impl_bytes_equal_variant variant=alpha_SparseByKey step=12 region=allocation_internal_of_device_0 device=0 ... equal=true`），
在 α=DenseWithKey 下仍不等。

**β（分配记录树根 largest_key 取哪个末端）**：`name=gap_reading_matches_crates gap=beta readings_whose_direct_fields_equal_crates=WholeAddressSpace`——
crates 用的是 5.2 丁（`[(0,0),(0xFFFFFFFF, 2^48-1)]`，字面「整个 key 空间」）。判据：根 largest_key 那 10 个字节在 β=WholeAddressSpace 下
与 crates 完全相等，在其余三种读法（甲/乙/丙）下都不等。

**γ/ε/η（extent 上段叶 key 区间）：阻断，S3，不是 5.2 已登记的任何一种读法**——见下一节详述，这是这一段最重要的发现。

**组合臂 N[C]**（α=SparseByKey + β=WholeAddressSpace，γ/ε/η 因阻断照主读法不变）：
`name=impl_bytes_equal_variant_summary variant=combination_n_c equal=20 unequal=9 alpha=SparseByKey beta=WholeAddressSpace`——
从 14/29 升到 20/29，6 个分配记录树相关区域（`allocation_internal_of_device_0` ×2、`allocation_internal_of_device_1` ×2、
`allocation_root` ×2）**全部变为相等**，验证了 α、β 的归因就是这两格差异的全部来源，没有第三个未登记的差异残留在这 6 个区域里。

剩余 9 个不等区域，`mismatch_bytes` 相比主读法 P 全部下降（`extent_root` 不变——本就与 α/β 无关；其余全降），
但仍不等：

| 区域 | 主读法 P 下 mismatch_bytes | N[C] 下 mismatch_bytes | 归因 |
|---|---|---|---|
| extent_root（2 份） | 14 | 14（不变） | γ/η（S3，见下） |
| mapping_root（2 份） | 40 | 16 | 部分是 α/β 驱动的分配记录树校验和已修复，剩下 16 字节是 extent_root 校验和经映射树位置条目传递过来的（「传过去的字段」，取决于 γ/η 阻断） |
| tree_table（2 份） | 24 | 16 | 同上（树表条目里 extent 树根指针的位置条目校验和） |
| journal_record（2 份） | 72 | 48 | 同上（journal 记录点名项里 extent_root 那一项的位置条目校验和 + 记录头自己的校验和） |
| root_record（1 份） | 20 | 19 | 几乎全部（19/20 字节）仍不等——root_record 经 journal 记录的校验和链，对 extent_root 的依赖比对分配记录树的依赖更重，这与 root_record 直接嵌的是「最新 journal 记录」的校验和、journal 记录点名项里恰好排着 extent_root 这一项一致 |

这张表**不是变体臂算出来的**（γ/η 阻断，没有变体可跑）——是拿 N[C]（已确认 α/β 归因）与主读法 P 的 mismatch_bytes 做前后对比，
推断残差的来源；「几乎全部」「部分」这类词是这段对比给出的强度描述，不是精确到字节的分解（要精确分解还得给 mapping_root/tree_table/
journal_record/root_record 也做类似 extent_root 的字段级 offset 归因，这一段没做，见「六、没做什么」）。

### extent_root 的 γ/η：条款定了的字段，冻结的臂 N 两个都没实现（S3，交主 agent）

派发提示第 3 条要求「对着 D18（块里携带什么信息） 已定项 2 原文判」。已定项 2 射程段原文
（`.claude/kb/decisions/18-块里携带什么信息.md:166`，整行抄）：

> 按位置寻址的树（分配记录树、extent 树，D8（核心索引结构） 已定项 14），节点头的 key 区间写这个节点按位置规定罩的那一段，
> 与「子树覆盖区间」在这类树上是同一段；checker 按位置独立算出每个节点该罩的那一段逐节点核。

这句话点名 extent 树（含上段叶）跟分配记录树受同一条规则：key 区间＝**这个节点按位置规定罩的那一段**，不是「首末两条实际条目的 key」。
分配记录树的叶、层级 1、根三层的代码都遵守这条（`build_allocation_tree` 里 smallest_key/largest_key 都是从「这一段的起点/终点」算出来的，
不是从记录本身的 key 算的——`allocation_record_tree_five_nodes_match_the_position_addressed_layout` 那条既有单测已经钉住）。

但 extent 上段叶（`build_index_node` 调用处，冻结代码原样不动）：

```
    let extent_unit = build_index_node(
        TreeIdentifier(TREE_IDENTIFIER_EXTENT), 0, 24,
        &extent_key, &extent_key,
        ...
```

`smallest_key`、`largest_key` **都直接传的是 `extent_key`**——那一条记录自己的 key `(0, 1, 0)`，不是「这片叶按位置规定罩的
`[0, 142]` 那一段」。这不是 5.2 登记的 γ甲（闭区间，`inode` 分量 = 142）也不是 γ乙（半开，143）：两者都假设用了「positionally 罩的
那一段」作为基础，冻结代码根本没走这条路——它复用的是 D18 已定项 2 主体（未按位置寻址的树）那条更早的规则「叶层码 2 节点上它就是
首末两条条目的 key」，忘了 extent 树被已定项 2 的射程段单独点名覆盖了。

**独立校验**（不接进 `publish_first_file`，只用来核对哪个 (γ,η) 组合能解释 crates 的字节；见 `extent_upper_leaf_positional_key_range`）：

```
E7RESULT name=gap_reading_reference_check region=extent_root gamma=Closed eta=Zero matches_crates=false matches_frozen_arm_n=false
E7RESULT name=gap_reading_reference_check region=extent_root gamma=Closed eta=Maximum matches_crates=true matches_frozen_arm_n=false
E7RESULT name=gap_reading_reference_check region=extent_root gamma=HalfOpen eta=Zero matches_crates=false matches_frozen_arm_n=false
E7RESULT name=gap_reading_reference_check region=extent_root gamma=HalfOpen eta=Maximum matches_crates=false matches_frozen_arm_n=false
```

crates 的字节恰好等于「γ=Closed（5.2 登记的主读法甲）+ η=Maximum（5.2 登记的变体乙）」这个组合算出的 key 区间
（smallest=`(0,0,0)`、largest=`(0,142,2^64-1)`）；**冻结的臂 N 在全部四种组合下都对不上自己**（`matches_frozen_arm_n` 全 false），
证实它走的是第三种、完全独立于 5.2 表的写法。

**归类**：这不是「5.2 登记的读法之间选哪个」的空白格问题（F1），是「条款定了的字段，冻结实现没有实现条款」——按第十一节 S3
「模型与 crates 在条款定了的地方对不上……不许把 crates/ 的代码或字节抄进模型去凑相等，不许改 5.2 的主读法……报：哪几个区域、
哪几个字段、两边各是什么、条款原文哪一句管这个字段」办：**这一段没有改 `build_index_node` 调用处那两行**（那是冻结主产物的字节，
改了就要重跑步④、重采 crates 快照，超出这一段的派发范围），只把发现记在这里，交主 agent 定——是现在就改（照 γ甲+η乙 或径直按
positionally 罩的一段实现，需要另一轮跑）、还是先把这条已定项 2 的射程段再确认一遍适用范围。extent_root 的另外两处差异
（偏移 [60] 与 [84] 各 1 字节，smallest/largest key 的 inode 分量）与上面同一个成因（用记录 key 代替位置区间），不再单独归因。

## 五、阳性对照 P1–P3、P6 与几何敏感性 G4

**P1（新布局读到了）**：读现有主产物（不用重算）。`name=old_new_region` 逐行核过：
`data_unit`/`inode_leaf`/`inode_root` 三类各两份 `changed=false`（预言「不变」的都不变）；
`extent_root`/`allocation_root`/`accounting_root`/`mapping_root`/`tree_table`/`root_record`/`journal_record` 全 `changed=true`
（预言「变」的都变了）；`name=old_new_region_summary old_roles=21 new_roles=29 only_in_o=2 only_in_n=10`
（只在 O 的 2 个＝旧分配记录树单节点根兼叶两份；只在 N 的 10 个＝新分配记录树 5 节点 × 2 盘）——与第七节 B12 逐条相符。**P1 过。**

**P2（比对器分得出一个字节）**：在四条臂（冻结 arm N、`alpha_SparseByKey`、`beta_WholeAddressSpace`、`combination_n_c`）各自的镜像上，
四个点各翻一位，喂给与真比 crates 同一条比对路（`find_matching_impl_write` + `compare_paired_write`）：

```
$ grep 'name=positive_control_p2' <主产物> | grep -c 'matches_injection=true'
16
$ grep 'name=positive_control_p2' <主产物> | grep -c 'matches_injection=false'
0
```

4 臂 × 4 点 = 16 条，**全部 `matches_injection=true`**（恰好命中注入的那一个区域、那一个偏移、改动 1 字节）。**P2 过。**

**P3（独立比对脚本）**：写在 `/tmp/claude-1000/e142-r15-step5/e142_r15_independent_compare.py`（**没有**放进
`research/scripts/`——那个目录不在 `experiment-runner` 的写范围表里，`.claude/hooks/agent-write-scope.tsv` 只登记了
`research/scripts/replay.sh` 这一行，见「六、没做什么」）。带 `--selftest`（三条：自己比自己 → 0 处不同；改一个十六进制字符 →
恰报那一个区域；删一行 → 报那个区域配不上），三条都过：

```
$ python3 e142_r15_independent_compare.py --selftest
P3 自测：3 条全过
```

拿它独立重算主产物与 crates 导出的逐区域比对，与模型内置的 `impl_bytes_equal` 结果**完全一致**（14 equal / 15 unequal，
不等的 15 个 `(device, offset, length)` 键逐个 diff 为 0 行）：

```
$ python3 e142_r15_independent_compare.py <主产物> <crates导出> | tail -1
summary model_regions=29 crates_regions=29 equal=14 unequal=15 unmatched_model=0 unmatched_crates=0
```

**P3 过**（脚本本体没有落进入库的 `research/scripts/`，是这一段的限制，不是判定本身的问题）。

**P5（两边的窗口起点是同一处）**：给模型加了 `name=before_window_summary side=model`（原来只有 crates 一侧发这一行）：

```
E7RESULT name=before_window_summary side=model operations=37 writes=31 last_five=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096
E7RESULT name=before_window_summary operations=37 writes=31 last_five=device0@16781312+4096,device1@16781312+4096,device0@7340032+512,device0@0+4096,device1@0+4096
```

两行 `operations`、`writes`、`last_five` 逐字相同。**P5 过**（两边窗口切点是同一处，Q142.1 比的确实是同一段）。

**P6（变体开关读到了）**：每条变体只应该让「这一格的直接字段与传过去的字段」跟着变。用 `emit_p6_diff_against_frozen`
（变体自己 vs 冻结臂 N 自己，不是变体 vs crates）核：

```
$ grep 'variant_vs_frozen_arm_n variant=alpha_SparseByKey' <主产物> | cut -d' ' -f3 | sort -u
region=allocation_internal_of_device_0 region=allocation_internal_of_device_1 region=allocation_root
region=journal_record region=mapping_root region=root_record region=tree_table
$ grep 'variant_vs_frozen_arm_n variant=beta_WholeAddressSpace' <主产物> | cut -d' ' -f3 | sort -u
region=allocation_root region=journal_record region=mapping_root region=root_record region=tree_table
```

α 只碰分配记录树的两类节点（叶不碰——叶的内容与 α 读法无关）与它们的下游校验和（`journal_record`、`mapping_root`、
`root_record`、`tree_table`），**不碰 `extent_root`**；β 只碰 `allocation_root` 与同一批下游，**不碰
`allocation_internal_of_device_{0,1}`**（β 是根专属的字段）也不碰 `extent_root`。两条变体各自的「读到了」范围与 5.2 对
「直接字段」「传过去的字段」的定义一致，没有越界碰到别的格。**P6 过**（两条测试过的变体都过；其余 3 条变体
`alpha_DenseWithKey`、`beta_LastSlotOnDevice`、`beta_MaximumSlotNumber` 这一段没跑 P6，只跑了 Q142.1v 本体的比对，见「六」）。

**G4（一盘，走整条写路，与 crates 比）**：`crates/` 侧**造不出这个几何**。新建的只读导出
`crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs` 跑起来在
`crates/singlefs-core/src/make_filesystem.rs:194` 断言失败退出（`assert_eq!` 不是 `Result`，走不到错误处理）：

```
$ ./target/release/e142_first_transaction_write_dump_one_device
thread 'main' panicked at crates/singlefs-core/src/make_filesystem.rs:194:5:
assertion `left == right` failed: 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）
  left: 1
 right: 2
```

这正是第八节 G4 登记的允许结局：「`crates/` 侧造不出 ⇒ G4 记『只在一个几何上量过』，不停机」。模型这一侧的「该看到」
（分配记录树 3 个节点、单元 10 个、窗口 13 次写）已由既有单测钉住，不用重跑：`positive_control_without_barriers_has_4092_violations_out_of_8192`
断言 `writes.len() == 13`；`allocation_record_tree_shape_matches_the_five_geometry_points` 的 `one_device` 那一档断言
`first_txn_allocation_nodes == 3`。**G4 记「只在一个几何上量过」，不挡够判**（第六节「够判点」原文：「G4 做不出来（crates 侧造不出一盘的池）
时记『只在一个几何上量过』，不挡够判」）。

## 六、没做什么

- **P3 脚本没有落进 `research/scripts/`**：写范围表（`.claude/hooks/agent-write-scope.tsv`）只登记了
  `research/scripts/replay.sh` 这一行，没有整个目录；脚本落在草稿目录 `/tmp/claude-1000/e142-r15-step5/`，跑过了、
  自测过了、与主产物比对过了，但没有入库。这是不是要给 `experiment-runner` 补一行写范围（或者改派给别的角色去入库这一份脚本），
  交主 agent 定。
- **P6 只跑了两条变体**（`alpha_SparseByKey`、`beta_WholeAddressSpace`，即与 crates 匹配的那两条），没有对
  `alpha_DenseWithKey`、`beta_LastSlotOnDevice`、`beta_MaximumSlotNumber` 三条也做「与冻结臂 N 自己比、只碰该碰的字段」核对——
  这三条变体在 Q142.1v 主体（与 crates 比）里跑过、`impl_bytes_equal_variant` 里有它们的行，缺的只是 P6 那道额外的
  「自己 vs 自己」核对。
- **mapping_root / tree_table / journal_record / root_record 四类残留差异没有做到 extent_root 那种字段级归因**
  （逐字节标出是哪个位置条目的哪个校验和）——只做了「N[C] 前后 mismatch_bytes 数量对比」这种较粗的归因，见「四」表末尾那句。
  要做到字段级，需要给这四类结构也写一遍类似 `code2_field_rows`/`explain_offset` 的字段表，这一段没做。
- **G4 只跑了「crates/ 造不出」这一步**，没有反过来验证「如果 `crates/singlets-core` 那条断言放开会怎样」——那需要改
  `crates/singlefs-core`，不在只驱动只观测的写范围内，也不在这一段的派发范围内。
- **γ/η 的发现没有落成代码修复**：`build_index_node` 调用处那两行按第十一节 S3「不许…改主读法」原样留着，只记录发现，交主 agent 判断
  下一步（改代码需要重跑步④、重采 crates 快照，是另一轮的量）。
- **`.claude/kb/experiments/142-第一个事务的干跑.md` 没有更新**——派发提示没有要求这一段写实验页，「实验页与索引行要不要这一次写」
  按共用约束由主 agent 决定；这一段的新发现（α/β 归因、extent_root 的 S3 发现、P1-P6/G4 结果）目前只在这份报告与产物里。
- **这一段之前就存在的 21 处 clippy 违规没有修**（详见「一」），按「顺手的整理单独提交」不属于这一段。
- **`naming-lint.sh` 对这一段新加的两个函数名判违规，没有改**：`run_p2_positive_control`（「p2」单字母加数字）、`run_q142_1v`
  （「q142」单字母加数字）。`e142_first_transaction_dry_run.rs` 不是新文件，已有 `p6_*`/`g1_*`/`g2_*`/`s_star_*`/`t9_*` 等几十处
  同类未登记缩写（回扫欠账，不在这一段范围），新加的两个跟这些是同一种写法；没有单独修这两个是为了不在「一批未回扫的欠账里挑两个改」，
  是不是该借这一段一起清，交主 agent 定。新建的 G4 装置（`e142_first_transaction_write_dump_one_device.rs`）过
  `naming-lint.sh` 零违规——那一份按「新文件」的标准做了。
- **没有跑门禁 15 号、87 号**（不归我）；**没有提交**。
- **一处流程偏差，如实报告**：变异表新增 14 行（M113–M126）用了 Bash 里一段 Python 脚本做批量字符串替换（改 `Max`/`MaxSlotNumber` 等
  4 处命名为 `Maximum`/`MaximumSlotNumber` 那一步，24 处替换），没有用 Edit 工具逐条替换——按共用约束「手写的改动一律用 Edit」，
  这一步做法不对；改完用 `grep`/`cargo build`/`cargo test` 核过内容正确、写范围内没有越界（还是同一份已在改的文件），
  但流程上应该用 Edit。如实记录，供主 agent 判断要不要紧。

## 七、岔路表（问题单 `research/prompts/m2-keyspace-rerun-questions.md:11-14`）

| # | 问题 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 1 | 独立装置按 D8 已定项 14 改成按位置寻址之后，第一个事务写出的每个区域与 `crates/` 实装比，全等还是不等 | **够判、已判完**：第六节「够判点」要求的 Q142.1v（8 条注册变体全跑：α 乙丙、β 乙丙丁，γ乙 ε乙 η乙 阻断已如实记）与 P1–P3、P6 都做完，答案不变——**不等**（14/29，与上一段一致）；这一段新增的是归因：α=乙、β=丁 是登记内的空白格（F1），γ/η 是条款定了但冻结实现没实现的字段（S3，交主 agent） | 没有剩余的量能让「全等/不等」翻面——这是配对+sha256 直接算出的事实。剩下没做的（P6 补另外 3 条变体、mapping_root 等四类的字段级归因、γ/η 的代码修复）都只影响归因精细度，不影响这一行的判定 |
| 2 | 第一个事务新的写清单能不能从 E142 产物整行抄进 `layout/01-first-txn.md` | **够判、已判完**：答案不变——**不能**（15/29 行的取值与实装对不上，上一段已列）。这一段新增的信息：15 行里有 6 行（分配记录树的两类节点）现在能明确标「空白格取值」（R7 最后一句，α=乙、β=丁 已由变体臂验证过是唯一对得上 crates 的读法，交主 agent 决定要不要把这两格写进条款）；extent_root 那 2 行不能标「空白格取值」，要标「条款定了但两边有一边没实现，S3，交主 agent」；mapping_root/tree_table/journal_record/root_record 那 7 行仍是「不能」，原因待补字段级归因 | 没有剩余的量能让「能/不能」翻面——不等的行摆在那里，抄不了是事实。剩下的量只决定每一行该标哪种「不能」的理由 |

两行的判定这一段都没有变，变的是判定的归因精细度：从「手工诊断，非结论」升级为「变体臂逐条验证过」。

## 八、门禁（跑归属表登记给 experiment-runner 的 13 个阶段）

| 阶段 | 结果 |
|---|---|
| 27-format-constants | ✓ |
| 33-mutation-tables | ✗ **不是这一轮的**：点名的是 `crates/mutations.tsv:307/721/722`，全是「实二七」那批 fsync/mount 相关的行（重复行、锚点在 `mount.rs` 里命中 0 次）——这一段没碰 `crates/mutations.tsv`，也没碰 `mount.rs` |
| 34-experiment-index-sync | ✓ |
| 40-results-cited | ✗ **部分是这一轮的**：新产物 `e142-first-txn-dry-run-2026-09-25-q142-1v-combined.out` 还没被 `.claude/kb/experiments.md` 点名——这一段按共用约束没有更新实验页（派发提示没要求，见「六」），是不是要点名交主 agent 定；其余 7 份（`e156`/`e158` 相关）与这一段无关 |
| 52-segment-registry | ✗ **预期内、不变**：`layout/01-first-txn.md` 段序列表仍是旧布局（`16+2+1+2`/`...18...`），产物是新布局（`24+2+1+2`/`...26...`）——与上一段报告记录的同一处 F2（条款自己两处对不上）一致，等第 2 行最终判定出来后由书记员改表 |
| 80-absolute-assertions | ✓ |
| 85-repro-command | ✓ |
| 86-experiment-orphans | ✓ |
| 88-quoted-result-lines | ✓ |
| 69-evidence-in-repo | ✗ **不是这一轮的**：点名的三处 `/tmp` 引用都在别的提示文件（`m2-final-code-r3/r4`、`m2-safety-r1`），与 E142 无关 |
| 75-decision-experiment-links | ✓ |
| 96-experiment-source-discipline | ✓ |
| 99-multipath-registry | ✓ |

`bash research/scripts/replay.sh E142` 结果：

```
E142  @driver_e142             字节一致 e142-first-txn-dry-run-2026-09-25-q142-1v-combined.out
字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0
```
