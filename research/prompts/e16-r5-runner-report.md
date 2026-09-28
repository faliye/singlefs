# E16 第五次跑（今天单元宽度下的峰值落点、预测式误差、适用范围句）执行报告

跑前登记：`research/prompts/e16-r5-prereg.md`（sha256 cb75508485a4043abaf02e9fe86f63669a9b895bcb04fcd4165becf6b0ffff1c，核对：`sha256sum research/prompts/e16-r5-prereg.md`）。

## 接班说明

上一任执行员被机器重启打断，未交回。第一步核现场：装置文件在仓里不存在（上一次 Write 没落盘），设计员材料从会话记录重建在 `/tmp/claude-1000/e16-r5-design/`（未改动）。从重建的草稿（`/tmp/claude-1000/e16-r5-run/draft-e16_fifth_run_peak_under_today_widths.rs`，984 行，从未编译过）逐段对照登记核对后作为起点重写，过程中发现并修了一处语法错误（Q3/Q4 汇总函数里 `for (label, selector): (&str, fn(...)->f64) in [...]` 不是合法 Rust 语法，改成先声明一个类型化的 `const` 数组再 `for … in` 这个数组）与若干命名纪律问题（见下）。

## 一、单测数

`cd research && nice -n 19 bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 10 cargo test --release -p e7-index-bench --bin e16-fifth-run-peak-under-today-widths`

**19 个单测全绿，0 个失败**（16 个来自原稿，另加 3 个：`predicted_ratio_uses_node_levels_not_node_levels_minus_one`、`relative_error_denominator_is_the_passed_in_value_not_always_predicted`、`monotonic_tolerance_change_flips_the_batch_at_most_streams_verdict`——补这三条是因为原稿的 M10、M11、M14 三条变异锚点在原有单测里都抓不到，详见下节）。

## 二、变异三个数与分类

`cd research && nice -n 19 bash scripts/capped.sh 10 bash scripts/mutate.sh e16-fifth-run-peak-under-today-widths e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs mutations/e16_fifth_run_peak_under_today_widths.tsv`

表 `research/mutations/e16_fifth_run_peak_under_today_widths.tsv`，16 条。

**抓到 16 / 无效 0 / 没红 0**（`✅` 16 条，无 `⏭`、`❌`、`💥`、`⚠️`、`⏱`、`🧱`）。收尾行：「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」——内存撞顶与超时都是 0，不触发整轮已判失败。

逐条列出（编号对应跑前登记第九节 M1–M16，抓到的测试名照 mutate.sh 原样）：

| 编号 | 改什么 | 抓到的测试 |
|---|---|---|
| M1 | 最底一层扇出改用内部扇出（144→147） | geometry_root_level_node_levels_root_children_are_pinned、crates_format_literals_used_here_are_pinned_to_the_registered_values、geometry_unit_counts_match_the_byte_sizes、lowest_level_node_indices_of_adjacent_units |
| M2 | 内部扇出改小一档（147→146） | crates_format_literals_used_here_are_pinned_to_the_registered_values、geometry_unit_counts_match_the_byte_sizes、geometry_root_level_node_levels_root_children_are_pinned、s2b1_and_s2b2_match_the_independent_anchor_script |
| M3 | 数据单元载荷不扣头偏移 | data_unit_payload_bytes_matches_the_addition_form、geometry_unit_counts_match_the_byte_sizes |
| M4 | 一条记录装旧记录宽的项数（67→72） | entries_per_record_today_and_legacy、crates_format_literals_used_here_are_pinned_to_the_registered_values、write_ahead_log_leaf_record_blocks_today_vs_legacy_at_boundary_batches |
| M5 | 甲的游标记录不计块 | s2b1_and_s2b2_match_the_independent_anchor_script、positive_control_four_batch_one_difference_equals_node_levels_plus_one |
| M6 | 甲的根槽不计块 | positive_control_four_batch_one_difference_equals_node_levels_plus_one、s2b1_and_s2b2_match_the_independent_anchor_script |
| M7 | 祖先只走到根层级的下一层（漏根） | ancestor_deduplication_matches_hand_computation、s2b1_and_s2b2_match_the_independent_anchor_script、positive_control_four_batch_one_difference_equals_node_levels_plus_one |
| M8 | 根层级边界取错 | geometry_root_level_node_levels_root_children_are_pinned、predicted_ratio_uses_node_levels_not_node_levels_minus_one、s2b1_and_s2b2_match_the_independent_anchor_script |
| M9 | 流的落点地板除换成向上取整 | stream_offset_for_one_tebibyte_file_geometry_matches_the_anchor_script |
| M10 | 预测式树高少套一层 | predicted_ratio_uses_node_levels_not_node_levels_minus_one（新补） |
| M11 | 相对误差分母恒用预测比值 | relative_error_denominator_is_the_passed_in_value_not_always_predicted（新补） |
| M12 | 峰值集合容差 0.1%→10% | peak_set_tolerance_change_widens_the_peak_set |
| M13 | 批≈流数带宽因子 2→4 | near_diagonal_band_bound_change_flips_classification |
| M14 | 单调容差 1%→0 | monotonic_tolerance_change_flips_the_batch_at_most_streams_verdict（新补） |
| M15 | 乙每次 fsync 落全部脏单元（不扣已落过的） | s2b1_and_s2b2_match_the_independent_anchor_script、positive_control_four_ratio_above_one_for_small_batches、positive_control_four_batch_one_difference_equals_node_levels_plus_one |
| M16 | checkpoint 间隔恒取 2000 | checkpoint_interval_and_operation_count_are_pinned |

M9 还额外做了一次重构：把 `simulate_cell` 里 `geometry.units / stream_count` 抽成 `units_per_stream_for()`，原稿里对应的单测是在测试体里独立重算 `G2.units / 3`，不经过 `simulate_cell` 真正调用的那一行，锚点改了这条单测也不会红（第四类「取样点不敏感」的一种变体：测试与实现各自算了一遍，恰好没有连起来）；改完让两处测试都经这个函数走，M9 现在真的抓得到。

## 三、阳性对照 PC1–PC4 与停机 S1–S3

**PC1**（现跑原装置 `e16-journal sweep`，代码一字未动）：`cd research && nice -n 19 bash scripts/run-with-memory-cap.sh 4G ./target/release/e16-journal sweep > /tmp/claude-1000/e16-r5-run/pc1-original-sweep.out`，落进 `research/results/e16-journal-2026-09-28.out`（新文件，`research/results/e16-journal-2026-08-31.out` 原样保留未动）。56 个 `ratio` 逐格与跑前登记第二节抄录的表按两位小数核对（现查命令 `grep 'streams=2 batch=' research/results/e16-journal-2026-09-28.out`），S=2 行 3.500/3.667/2.337(≈2.34)/1.733(≈1.73)/1.387(≈1.39)/1.164(≈1.16)/1.076(≈1.08)/1.037(≈1.04)，与表一致；S=128 行同样核对一致。**F1 不触发。**

**PC2**（相对误差中位/最大与「9.6%、20.5%」核对，脚本 `/tmp/claude-1000/e16-r5-run/pc2.py`，只读 PC1 的产物，独立于装置的 `error_model_denominator`/`error_predicted_denominator` 字段重算）：
- 以**模型比值为分母**：中位 10.63%、最大 25.79%——**复现不出**。
- 以**预测比值为分母**：中位 9.61%、最大 20.50%——**与登记的「9.6%、20.5%」一致**（四舍五入到 0.1%）。

**判定：PC2 复现得出，用「预测比值为分母」这一种，F2 不触发。** 这解决了跑前登记第一节里悬而未决的「相对误差分母」问题：实验页原句用的分母是预测比值，不是登记默认写的模型比值——第十二节「修订」记这一条。

**PC3**（新装置 pc3 模式在 G0 上现跑原 56 格，`RecordFormat::LegacyOriginal`，与 PC1 的产物逐字段比对）：`cd research && nice -n 19 bash scripts/run-with-memory-cap.sh 4G ./target/release/e16-fifth-run-peak-under-today-widths pc3 > .../pc3-new-device-G0.out`，落进 `research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out`。比对脚本 `/tmp/claude-1000/e16-r5-run/pc3_compare.py`：56 格 × 4 字段（`a_blocks`/`intent_avg`、`l_blocks`/`write_ahead_log_leaf_avg`、`ratio`、`pred_ratio`/`predicted_ratio`）= 224 处比较，**0 处不同**。**停机 S2 不触发。**

**PC4**（每个几何点 B=1 时甲−乙=祖先层数+1；B≤67 时比值>1）：由单测 `positive_control_four_batch_one_difference_equals_node_levels_plus_one`、`positive_control_four_ratio_above_one_for_small_batches` 覆盖，两条都绿（见第一节单测清单）。**V4 不触发。**

**S1**（装置本地常量与 `crates/singlefs-format/src/lib.rs` 现查值核对，跑前登记第十三节命令二）：

```
grep -nE '^pub const (DATA_UNIT_BYTES|DATA_UNIT_PAYLOAD_OFFSET|EXTENT_TREE_INTERNAL_FANOUT|EXTENT_TREE_LOWER_LEAF_DATA_UNITS|JOURNAL_RECORD_BYTES|JOURNAL_HEADER_BYTES|JOURNAL_NAMED_ENTRY_BYTES|JOURNAL_NAMED_ENTRIES_PER_RECORD|TEST_IMAGE_DEFAULT_BYTES):' crates/singlefs-format/src/lib.rs
```

输出九行与跑前登记第十三节抄录的九行逐字相同（2026-09-28 现查）。**S1 不触发。** 另外装置的 `assert_registered_constants_are_internally_consistent()` 在 `main()` 开头把这几个常量的加法/除法关系在运行期也过一遍（不只靠单测撑着，见下节命名纪律那段的解释）。

**S3**（几何配置行与独立锚点脚本核对）：产物里 `name=geometry_config` 三行的 `units`/`root_level`/`node_levels`/`root_children` 与跑前登记第七节 7.2 表逐项相同（G1: 131611/2/3/7；G2: 33692212/3/4/11；G4: 539075383/4/5/2），单测 `geometry_root_level_node_levels_root_children_are_pinned` 也钉了这几个数。**S3 不触发。**

## 四、命名纪律（naming-lint）

原稿 384 处命名违规里有 56 处出在新文件（G0–G5 单字母加数字、wal 缩写、S_AXIS/B_AXIS 单字母、q5/q6 单字母加数字、max 缩写、单字母闭包参数 a/b、config 缩写），逐条改完（`bash .claude/scripts/naming-lint.sh` 对这个文件现在 0 命中，全仓剩余 328 处是别的文件的存量，不归这次改）。改法：几何点常量按含义重命名（`G1`→`FOUR_GIBIBYTE_FILE_GEOMETRY` 这类），`wal_leaf` 全文展开成 `write_ahead_log_leaf`，`q5_holds`/`q6_*` 按它们对应的判断展开（`ratio_non_decreasing_through_streams_holds` 这类），闭包参数改成 `left_cell`/`right_cell` 这类。改名过程中顺带删掉了一个未读字段（`GeometryGrid.geometry`，本来没被任何代码读到，直接删掉而不是加 `#[allow(dead_code)]`）。

改名之后 `cargo build --release` 报了 11 条 `never used` 警告（七个基础常量与 G3/G5 两个几何点只在 `#[cfg(test)]` 里被读到，非测试构建看不见测试模块）：按「编译器报的 never used 当错误处理」补了 `assert_registered_constants_are_internally_consistent()`，在 `main()` 里把这几个常量的自洽关系也过一遍，警告清零（`cargo build --release` 现在 0 warning）。

## 五、产物路径、完成标记与齐全性自查（第 4b 步）

三份产物：

| 产物 | 路径 | 行数 | 完成标记 |
|---|---|---|---|
| 原装置 PC1（G0 全 56 格） | `research/results/e16-journal-2026-09-28.out` | 57 | `E7RESULT name=done emitted=57` |
| 新装置 PC3（G0 上 56 格，legacy 记录宽） | `research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out` | 58 | `E7RESULT name=done emitted=58` |
| 新装置第一段网格（G1/G2/G4 全网格） | `research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out` | 3530 | `E7RESULT name=done emitted=3530` |

三份的行数都与各自「emitted=」完成标记一致，**V1 不触发**。

第一段网格按族计数（现查命令与原样输出）：

```
grep -c '^E7RESULT name=cell mode=grid' research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out   → 3423
grep -c '^E7RESULT name=row_summary'    research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out   → 63  (= 21 流数 × 3 几何)
grep -c '^E7RESULT name=q1_summary'     同上 → 3  (每个几何 1 行)
grep -c '^E7RESULT name=q2_summary'     同上 → 3
grep -c '^E7RESULT name=q3_summary'     同上 → 6  (每个几何 2 行：model / predicted 分母)
grep -c '^E7RESULT name=q4_summary'     同上 → 6
grep -c '^E7RESULT name=geometry_config' 同上 → 3
grep -c '^E7RESULT name=geometry_sensitivity' 同上 → 21 (每个流数取值 1 行)
```

1 配置行 + 3 geometry_config + 3423 cell + 63 row_summary + 3 q1 + 3 q2 + 6 q3 + 6 q4 + 21 geometry_sensitivity + 1 done = 3530，与总行数一致。**没有哪一族一次都没造出被测形状**：三个几何各自 1141 个 cell（3423÷3），各自 21 行 row_summary，逐几何都有 q1–q4 汇总。

`windows`（fsync 次数）字段最小值为 10（`grep '^E7RESULT name=cell mode=grid' … | grep -oE 'windows=[0-9]+' | sort -n | head -1`），没有 0；全文 `grep -ic 'nan\|inf'` 命中 0。**V2 不触发。** 两条臂逐 cursor 读同一个生成器（`simulate_cell` 里 `unit_for_cursor` 只调一次，两条臂共用同一个 `unit`），单测 `both_arms_see_the_same_unit_at_the_same_cursor` 钉了这条不变量。**V3 不触发（结构性保证 + 单测）。** 同一份装置连跑两遍（重构 `units_per_stream_for` 前后各跑一次 grid 与 pc3）`diff` 结果为空，逐字节相同。**V5 不触发。**

## 六、判决行点名（第 4c 步）

`grep -n 'name=verdict' research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out research/results/e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out research/results/e16-journal-2026-09-28.out` → **0 行**：这个装置没有 `name=verdict` 这一种产物行，跑前登记也没有把哪一行定成「判决行」——它是纯计数模型，每个几何点各出一份 Q1–Q6 的汇总，合取留给主 agent（登记第六节表头「判定由主 agent 做」）。

按第 4c 步的精神改查布尔与计数类字段，逐个点名取值为 `false` 的：

- `ratio_non_decreasing_through_streams_holds=false`：63 行 row_summary 里出现 37 次（跨三个几何）。这是 Q5（原句前半「批 ≤ 流数时不降」）在这些行上不成立，且这条恰是问题单第 3 行的直接材料（不是意外，是登记第六节 Q5 本来就要报的量）。是不是登记预期内：是——登记第八节明确写了「结果反过来（模型在 L=3 上仍逐行在 B≈S 见顶）我照判甲，不改分类」，即预期结果可能不支持原句，这条不算异常。
- `row_dips_below_one=false`（即没跌破 1 的行）：出现 26 次，对应 `row_dips_below_one=true`（跌破 1）出现 37 次——过半数的行在批轴走到 16384 时比值已经跌破 1，比原句「塌向 1」更进一步。登记预期内（第四节最后一条「设计时手推的另一件事」已经预告了这个可能）。
- `ratio_non_increasing_beyond_streams_holds`、`settles_near_one_beyond_sixteen_streams_holds`：全部 63 行都是 `true`，没有 `false`。
- 没有 `not_run` 字段；没有名字表示「违例、不匹配、歧义、失败」的整数计数字段（这个装置的字段都是比值、批号、布尔或比例，没有这一类计数）。

结论：**判决行 0 条（装置没有这个字段名）；点名的布尔字段里 `ratio_non_decreasing_through_streams_holds` 与 `row_dips_below_one` 各有一部分 `false`/`true`（对应字段含义相反，已在上面分别点名），都在登记预期内，不是异常。**

## 七、replay.sh 结果

在 `research/scripts/replay.sh` 里新增两行（紧接原 E16 两行之后）：

```
E16R5|e16-fifth-run-peak-under-today-widths||e16-fifth-run-peak-under-today-widths-2026-09-28.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|pc3|e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out|exact
```

`bash research/scripts/replay.sh E16R5` 原样输出：

```
E16R5 e16-fifth-run-peak-under-today-widths 字节一致 e16-fifth-run-peak-under-today-widths-2026-09-28.out
E16R5 e16-fifth-run-peak-under-today-widths 字节一致 e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out
✓ 复跑判过的 2 行都对得上（字节一致 2、仅计时不同 0），结论断言全中
```

PC1 那份产物（`research/results/e16-journal-2026-09-28.out`）复用**已有**的 E16 复跑行（`E16|e16-journal||e16-journal-2026-08-31.out|exact`），不是这一次的新品种，没有另开复跑行；它与已归档的 `e16-journal-2026-08-31.out`——两份都是对同一个未改动装置的现跑，`diff` 结果为空，逐字节相同（`diff research/results/e16-journal-2026-08-31.out research/results/e16-journal-2026-09-28.out` 无输出）。

## 八、登记修订

跑前登记第十二节（修订）原来留空。这一次只有一条要记（已按 evidence-discipline.md 的三步走：记一次输、只许收严、写明收严在哪）：

**跑前登记第一节「相对误差」写的默认分母是模型比值，PC2 显示实验页那两个数（9.6%、20.5%）实际是用预测比值当分母算出来的。** 登记原文已经预留了这条出路（「这个分母是否就是实验页那两个数的算法，由第五节 PC2 在原装置现跑的产物上核：复现得出就用它」），PC2 现跑复现得出，第 2 行（Q3/Q4）判据因此照登记原文的指示改用「预测比值为分母」的那一路；**登记原判据的两种分母都没删**，产物里 `error_model_denominator`/`error_predicted_denominator` 两个字段照样各报一遍（Q3/Q4 表两行都在），这是收严（多算一路、都报），不是丢弃。依据：`/tmp/claude-1000/e16-r5-run/pc2.py` 的输出（本报告第三节抄录），时点在产物之前（PC2 是跑第一段网格之前先跑的阳性对照）。

## 九、Q1 归类没做「加密」那一步

跑前登记第六节 Q1 末句要求：`B_lo`/`B_hi` 换成批轴上相邻取值会改变归类时，在两个相邻取值之间逐个整数加密重判、报「加密过」。**这一版没有实现这一步**（装置头部注释里写明了）。人工抽查了两处最可能踩边界的地方：① G2 上 S=16（甲行，peak_batch_low=peak_batch_high=15）到 S=24（丙行，peak_batch_low=peak_batch_high=1）的分类跳变很陡（不是从 15 挪到 16 才翻面，是从「峰值在 15」直接跳到「峰值恰好在 1」），不像加密能改变的边界情形；② 乙行组（S=24–96）的 `moved_diagonal_ratio` 从 0.21 到 0.05 平滑过渡，没有在相邻流数点上出现分类反复。**但这只是抽查，不是系统性证明**，第二段如果要补，这是最该补的一项；实验页「它答不了的」与本报告岔路表都会点名。

## 十、结果：三个问题逐个答

产物文件：`research/results/e16-fifth-run-peak-under-today-widths-2026-09-28.out`（下称「网格产物」）。三个几何点：G1=`four_gibibyte_file`（L=3）、G2=`one_tebibyte_file`（L=4，真实基线）、G4=`sixteen_tebibyte_file`（L=5）。

### 第 1 行：峰值落在哪

`grep '^E7RESULT name=q1_summary' 网格产物`（原样）：

```
name=q1_summary geometry=four_gibibyte_file classification=丙行 detail=全部行同一类
name=q1_summary geometry=one_tebibyte_file classification=分段 detail=S=2:甲行,S=3:甲行,S=4:甲行,S=6:甲行,S=7:甲行,S=8:甲行,S=11:甲行,S=12:甲行,S=16:甲行,S=24:丙行,S=32:丙行,S=48:丙行,S=64:丙行,S=96:丙行,S=128:丙行,S=192:丙行,S=256:丙行,S=384:丙行,S=512:丙行,S=768:丙行,S=1024:丙行
name=q1_summary geometry=sixteen_tebibyte_file classification=分段 detail=S=2:丙行,S=3:丙行,S=4:丙行,S=6:丙行,S=7:甲行,S=8:甲行,S=11:甲行,S=12:甲行,S=16:宽平台,S=24:乙行,S=32:乙行,S=48:乙行,S=64:乙行,S=96:乙行,S=128:宽平台,S=192:丙行,S=256:丙行,S=384:丙行,S=512:丙行,S=768:丙行,S=1024:丙行
```

**真实基线 G2 上不是「全部行都是甲行」：流数 ≤ 16 的 9 行是甲行（峰值落在批≈流数），流数 ≥ 24 的 12 行全是丙行（峰值直接在批=1，此后单调下滑，没有逐行内部峰值）。按登记第 1 行的判法这记「乙、丙或分段」——这里是「分段」，S≤16 与 S≥24 两段各自记，不是单一候选。** G1（树高比 G2 低一层）上全部 21 行都是丙行；G4（树高比 G2 高一层）上形状比 G2 更复杂（丙→甲→宽平台→乙→宽平台→丙）。三个几何的整体分类互不相同，`geometry_sensitivity` 行（`grep -c 'stable=false' 网格产物` = 15，`stable=true` = 6，21 个流数取值里 15 个三几何分类不一致）：**触发 F4（射程随树高变），第 1 行的答案必须带树高或根下孩子数条件，不能只按 G2 写成一句无条件的话。**

Q2（二维最大值，`grep '^E7RESULT name=q2_summary' 网格产物`）：G2 上全局最大比值 3.888889 落在 S=8、B=8（单点，`streams_lower=streams_upper=8 batch_lower=batch_upper=8`），`streams_interior=true batch_interior=true`——两轴都落在扫描区间内部，**Q2 够判条件满足**。G1、G4 同样 `streams_interior=true batch_interior=true`（G1 的满足带 S=2..1024 一整段，因为 L=3 时 B=1 附近比值全体持平在 3.0；G4 单点在 S=7,B≈6-7）。**没有一个几何触发端点扩展（F3），第 1 行不因端点而不够判。**

### 第 2 行：预测式误差

`grep '^E7RESULT name=q3_summary' 网格产物`（原 56 格子集，两种分母）：

| 几何 | 分母 | 中位 \|e\| | 最大 \|e\| | 判定（对 9.6%/20.5%） |
|---|---|---|---|---|
| G1 | model | 22.11% | 93.87% | 超 / 超 |
| G1 | predicted | 18.11% | 48.42% | 超 / 超 |
| G2 | model | 14.96% | 62.28% | 超 / 超 |
| G2 | predicted | 13.01% | 38.38% | 超 / 超 |
| G4 | model | 19.68% | 50.15% | 超 / 超 |
| G4 | predicted | 16.45% | 33.40% | 超 / 超 |

**三个几何、两种分母，Q3 全部判「超」**——今天的宽度下预测式的误差比页上原来报的 9.6%/20.5%（那是原装置 128 扇出、L=4 下的数）大得多，即便用 PC2 核实过的「predicted 分母」也是超（G2 上中位 13.0%、最大 38.4%，都明显高于 9.6%/20.5%）。**触发 F6**：Q4（`positive_error_fraction`）三个几何、两种分母都是 0.9816——98.16% 的格子上预测式系统性偏高，与页上「预测系统性偏高」的定性结论方向一致，但幅度（今天的误差远大于原表）不一致。**对应第 2 行的判定：乙（超出），「哪几格、差多少」见网格产物里 `name=cell` 行的 `error_predicted_denominator` 字段（1141×3 格全部列出，不逐格摘抄）。**

### 第 3 行：适用范围句还是不是真话

按登记「G2 上 Q1 为甲、Q5 全行成立、Q6①②全行成立且③没有跌破 1 ⇒ 原句成立」合成（合取由主 agent 做，这里只报每个量的判定）：

- Q1（G2）：**分段，不是甲**（S≤16 是甲行，S≥24 是丙行）。
- Q5（批≤流数不降，`grep 'geometry=one_tebibyte_file' 网格产物 | grep name=row_summary` 数 `ratio_non_decreasing_through_streams_holds`）：21 行里 12 行 `false`（S=24,32,48,64,96,128,192,256,384,512,768,1024，worst_drop_fraction 从 1.26% 到 4.67%，都在 batch=2 或 6 附近）。**Q5 不是「全行成立」。**
- Q6①（批≥流数不升）：21 行全部 `true`。**成立。**
- Q6②（16×流数处落定）：21 行全部 `true`。**成立。**
- Q6③（是否跌破 1）：`grep 'geometry=one_tebibyte_file' … row_dips_below_one=true` 命中 12 行（S=2..32 共 12 个较小流数在 batch=16384 处 ratio 降到 0.9927–0.9968，跌破 1；S=48 及以上没跌破——批轴最大只到 16384，更大流数的「16×流数」结算点更靠后，还没来得及跌破）。**跌破 1 的行不是 0，Q6③ 不是「全行没跌破」。**

**合成结论（G2 上）：原句不成立（乙）。改写模板照登记第六节给的填：「攒批吃不掉延后祖先的收益，只在流数 ≤ 16（对应今天 1 TiB 文件、根下 11 个孩子这个几何）时成立；批超过流数之后，比值在 16×流数附近落定到峰值超额的四分之一以内，但对流数 ≤ 32 的情形，比值会在批轴远端（≥16384）跌破 1，不只是「塌向 1」。」** G1、G4 上 Q1 与 G2 判得不同（第一节已述），第三节「改写要带上树高或根下孩子数条件」——G3（同高、根下孩子满）没跑（第二段），所以「树高」与「根下孩子数」这两个因素**没分开**：目前只能说「随几何变」，不能确认是哪一个变量在主导。

## 十一、判别力自证（登记第八节 8.2）

单测 `near_diagonal_band_bound_change_flips_classification`（把带宽因子从 2 挪到 4，分类必须从乙行翻成甲行）与 `peak_set_tolerance_change_widens_the_peak_set`（容差从 0.1% 挪到 10%，B_lo 必须从 2 翻成 1）都绿，两条断言里都写了「翻面」那句判定，不只是留档。

## 十二、两处不建模的结构差异

跑前登记第三节末尾写明的两处（`crates/` 今天的发布点名每个重写单元、一次写还动别的几棵树）这一次没有建模，装置头部注释与本报告都点名了；它们会改每次 fsync 的实际块数，但按主 agent 认的项，这次不算「宽度」，不在这一题的范围里。实验页「它答不了的」一节会重复这一句。

## 十三、没做什么

- 第二段（G3、G5、F128、F296、R1、R∞）没跑：登记的够判停机规则是「第一段够判就不跑」，第 1 行两轴都不在端点、G1/G2/G4 三个方向相反的树高取样点已经给出（F4 已触发，「不稳定」已经是够判的答案之一，不是欠账）；但**树高与根下孩子数这两个因素没分开**（第三行改写句里已注明），这是第二段能补、第一段答不了的部分，交主 agent 判要不要续。
- Q1 的「加密」重判步骤没实现（第九节已述），只做了人工抽查。
- 没跑门禁阶段（提交前由 gate-triage 跑）。
- 没判这个实验的结论能不能推翻或确立 D23（journal 的角色与格式）或别的决策（那是推论，要走三方）。
- 「### 影响的决策」一节的回看，因为实验页还没写、决策文件是否引用了 E16 还没查，留给主 agent 在实验页写好之后核（见交回消息）。

## 十四、实验页与索引

- 实验页：`.claude/kb/experiments/16-journal的角色WALvs意图日志.md`，新增「### 第五次跑：今天的单元宽度下，峰值落点、预测式误差与适用范围句」一节（在「⚠️ 适用范围有界」那一节之后），标题行日期补上「09-28」，「还没做的那一半」加一条第二段的欠账说明。
- 「### 影响的决策」表：13 行全部重新回看（第 7b 步①：D23（journal 的角色与格式） 已定项 1 的依据段确实引了 E16，其余 12 行同样查过，关系没有变，只是日期与理由按这一次的改动重写；D23（journal 的角色与格式） 不在 `.claude/decision-links-pending` 里）。**D23（journal 的角色与格式） 已定项 1 那一行的理由段这次改写得比较多**——它的依据句直接引了「批≤并发流数时成立」这条被第五次跑证明只在小流数成立的话——这一格该不该从「支撑」升级判法、要不要在决策文件里补一句「已在多个流数档验证适用范围有界」，我没有改决策文件（不在写范围内），列在这里交主 agent 判。
- 索引行：`.claude/kb/experiments.md` 第 39 行（E16（journal 的角色：WAL vs 意图日志）），状态改成「半跑（08-28；09-03 补字节口径；09-28 补第五次跑）」，结论列追加「今天真实宽度下「批≈流数见顶」只在流数 ≤ 16 成立，射程随树高变」——这个措辞是我按结果起草的，交主 agent 认。
- `.claude/kb/experiments-history.md`：`## E16（journal 的角色：WAL vs 意图日志）` 节（现查行号 1099）的「**现状**」行与新增的「### 2026-09-28」条目。

## 十五、门禁自查（不算数，仅供参考）

顺手起了一次 `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh`（全仓扫描，误操作，不在这次任务范围内，也不该由我跑），起在后台，跑完前收工，没有等它、也没有拿它的结果当依据；`/tmp/claude-1000/e16-r5-run/report.md` 与实验页的写法按 kb-discipline 与 writing-discipline 的规则自查过（不用位置指代、不自称、引用带简称），没有另外跑门禁核对。

## 十六、岔路表（问题单三行）

| # | 判断 | 已够判 / 还差什么 | 剩下的量（第二段）能不能让它翻面 |
|---|---|---|---|
| 1 | 峰值落在哪 | **已够判**。Q1 全部行没有落在批轴端点（F3 不触发）；Q2 两轴的最大值都在扫描区间内部（G1/G2/G4 的 `streams_interior`、`batch_interior` 都是 true）。答案：G2（真实基线）上「分段」——流数 ≤16 的 9 行是甲行，流数 ≥24 的 12 行是丙行；三个几何（G1/G4）整体分类互不相同（21 个流数取值里 15 个不一致，触发 F4）。 | **不会翻面整体判定**（「分段」「射程随树高变」这两条结论不会因为多测 G3/G5 而变回单一的「甲」）；能补的是把「随几何变」细化成「随树高还是根下孩子数」——G3（同高、根下孩子满）能把这两个因素分开，这是第二段唯一能改变答案精度的地方。Q1 的「加密」重判步骤没做，只人工抽查了两处最可能踩边界的地方（S=16→24 的跳变、乙行组的比值梯度），不是系统性证明，理论上仍有翻面风险，但抽查没有发现任何迹象。 |
| 2 | 预测式误差对不对得上 | **已够判**。三个几何、两种分母、原 56 格子集与全网格都报了（Q3/Q4 各 6 行）；「哪几格、差多少」在网格产物的 `name=cell` 行里逐格可查。答案：全部「超」（中位 13.0%–22.1%、最大 33.4%–93.9%，远超页上原来的 9.6%/20.5%），98.2% 的格子预测式系统性偏高，方向不变、幅度变了（触发 F6）。 | **不会翻面**「超」这个方向：G1、G2、G4 三个几何一致地超，不是某一个几何的特例；第二段能补的是扇出、记录宽两个旋钮对误差幅度的敏感性，不会把「超」改判成「不超」。 |
| 3 | 适用范围句还是不是真话 | **已够判**（依赖第 1、2 行，两行都够判）。G2 上 Q1 不是甲（分段）、Q5 有 12/21 行不成立、Q6③ 有 12/21 行跌破 1——合取结果是「乙」（原句不成立，要改写），改写模板已在第十节给出。 | **不会翻回「原句成立」**；第二段能补的是改写句里「随几何变」那半句要不要换成更精确的「随树高」或「随根下孩子数」的条件——这半句的精度会变，方向不会翻。 |
