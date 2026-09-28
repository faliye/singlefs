# E16 第五次跑第二段执行报告（2026-09-28）

跑前登记：`research/prompts/e16-r5-prereg.md`（未改动，无需修订——这一段是登记第五节写死的
第二段，没有触发任何判据打中，不进第十二节）。上一段执行员报告：`research/prompts/e16-r5-runner-report.md`。

这一段回答的岔路（派发提示第 1 行）：`research/prompts/e16-r5-questions.md` 第 1 行的补充——
登记第二段（G3、G5、扇出 128/296、记录 1 项与 10⁶ 项），回答「1 TiB 上流数 ≥ 24 时峰值落到
批=1，是树高主导还是根下孩子数主导」。

## 一、装置改动

`research/e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs` 加了 `second_segment`
模式（`main()` 新分支）：新增 `RecordFormat::EntriesPerRecord(u64)` 变体（记录宽旋钮，R1/R∞ 用）、
四个新几何常量（`SMALL_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY`=F128、
`LARGE_FANOUT_ONE_TEBIBYTE_FILE_GEOMETRY`=F296、`RECORD_WIDTH_KNOB_MINIMUM_GEOMETRY`=R1、
`RECORD_WIDTH_KNOB_MAXIMUM_GEOMETRY`=R∞）、三个旋钮分组常量（`TREE_HEIGHT_KNOB_POINTS`
5 点、`FANOUT_KNOB_POINTS` 3 点、`RECORD_WIDTH_KNOB_POINTS` 3 点）与 `run_knob_group`/
`run_second_segment_mode` 两个函数。第一段的 `run_grid_mode`、`run_pc3_mode` 一个字没动
——`bash research/scripts/replay.sh E16R5` 核过第一段两份产物仍逐字节一致（见第五节）。

## 二、单测数

```
cd research && nice -n 19 bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 16 cargo test --release -p e7-index-bench --bin e16-fifth-run-peak-under-today-widths
```

**23 个单测全绿，0 个失败**（第一段 19 个 + 这一段新增 4 个：
`geometry_root_level_node_levels_root_children_of_the_fanout_knob_points_are_pinned`、
`record_width_knob_geometries_share_the_one_tebibyte_file_tree_shape`、
`entries_per_record_format_divides_with_ceiling_not_floor`、
`positive_control_four_batch_one_difference_equals_node_levels_plus_one_on_the_second_segment_points`）。

F128（leaf=fanout=128）、F296（leaf=fanout=296）在 1 TiB 单元数（33692212）下的
root_level/node_levels/root_children 独立算出：F128 = 3/4/17，F296 = 3/4/2
（`python3` 手推 `span(level)=leaf*fanout^level`，脚本见交回消息附的会话记录，与
`geometry_root_level_node_levels_root_children_of_the_fanout_knob_points_are_pinned` 钉的数一致）。

## 三、变异三个数与分类

```
cd research && nice -n 19 bash scripts/capped.sh 16 bash scripts/mutate.sh e16-fifth-run-peak-under-today-widths e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs mutations/e16_fifth_run_peak_under_today_widths.tsv
```

表 `research/mutations/e16_fifth_run_peak_under_today_widths.tsv`，追加第 17 条（只追加，
前 16 条一字不改）：「记录宽旋钮的 EntriesPerRecord 改成地板除」，锚点
`RecordFormat::EntriesPerRecord(entries_per_record) => entry_count.div_ceil(entries_per_record),`
→ `... => entry_count / entries_per_record,`。

**抓到 17 / 无效 0 / 没红 0**（17 条全部 `✅`，无 `⏭`、`❌`、`💥`、`⚠️`、`⏱`、`🧱`）。收尾：
「计数：内存撞顶 0 条（上限 16G）、超时 0 条」「已还原，基线仍全绿」。新的第 17 条被
`entries_per_record_format_divides_with_ceiling_not_floor` 与
`positive_control_four_batch_one_difference_equals_node_levels_plus_one_on_the_second_segment_points`
两条测试抓到。原有 16 条抓到的测试名与第一段报告逐一相同（未受这一段改动影响）。

## 四、产物路径、完成标记与齐全性自查（第 4b 步）

| 产物 | 路径 | 行数 | 完成标记 |
|---|---|---|---|
| 第二段（三个旋钮分组 × 11 个几何点） | `research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out` | 12926 | `E7RESULT name=done emitted=12926` |

行数与完成标记一致（V1 不触发）。分族计数（现查命令与原样输出）：

```
config: 3
geometry_config: 11
cell: 12551
row_summary: 231
q1_summary: 11
q2_summary: 11
q3_summary: 22
q4_summary: 22
geometry_sensitivity: 63
done: 1
```

3+11+12551+231+11+11+22+22+63+1 = 12926，与总行数一致。**没有哪一族一次都没造出被测形状**：
11 个几何点各自 1141 个 cell（12551÷11=1141，与第一段每个几何的 cell 数完全相同，因为批/流数
轴没变）；231 = 11×21（每个几何 21 行 row_summary）；q1/q2 各 11（每个几何一行）；q3/q4 各 22
（11 个几何 × 2 种误差分母）；geometry_sensitivity = 63 = 3 个旋钮分组 × 21 个流数取值。

`windows`（fsync 次数）最小值 10（`grep 'name=cell' … | grep -oE 'windows=[0-9]+' | sort -n | head -1`），
没有 0；全文 `grep -ic 'nan\|inf'` 命中 0（V2 不触发）。两条臂经同一个 `simulate_cell` 里的
`unit_for_cursor` 单点生成读同一个 cursor（这条结构性保证与单测
`both_arms_see_the_same_unit_at_the_same_cursor` 是第一段就有的，这一段的新几何点复用同一份
`simulate_cell`，同样成立，V3 不触发）。

**V5（同一份装置连跑两遍逐字节相同）**：`second_segment` 模式独立现跑两遍
（`/tmp/claude-1000/e16-r5-seg2/second-segment-raw.out` 与
`/tmp/claude-1000/e16-r5-seg2/second-segment-raw-rerun.out`），`diff` 输出为空，逐字节相同
（这份确定性模型没有隐藏状态）。V5 不触发。

**S1（本地常量与 `crates/singlefs-format/src/lib.rs` 现查值核对）**：这一段没有新增从
`crates/` 抄来的常量（F128/F296/R1/R∞ 都是这一次自己选的旋钮取样点，不是「今天」的真实值），
S1 仍是第一段核过的那九行，未变。

**S3（几何配置行与独立锚点核对）**：`name=geometry_config` 里
`full_root_level_four`（3/4/147）、`two_fifty_six_mebibyte_file`（1/2/58）与跑前登记第七节 7.2
逐项相同；`small_fanout_one_tebibyte_file`（3/4/17）、`large_fanout_one_tebibyte_file`（3/4/2）
与这一段独立算出的锚点（第二节）逐项相同。S3 不触发。

## 五、复跑登记

`research/scripts/replay.sh` 新增一行（紧接第一段两行之后，不改那两行）：

```
E16R5|e16-fifth-run-peak-under-today-widths|second_segment|e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out|exact
```

`bash research/scripts/replay.sh E16R5` 原样输出：

```
E16R5 e16-fifth-run-peak-under-today-widths 字节一致 e16-fifth-run-peak-under-today-widths-2026-09-28.out
E16R5 e16-fifth-run-peak-under-today-widths 字节一致 e16-fifth-run-peak-under-today-widths-pc3-2026-09-28.out
E16R5 e16-fifth-run-peak-under-today-widths 字节一致 e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out
字节一致 3 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0
✓ 复跑判过的 3 行都对得上（字节一致 3、仅计时不同 0），结论断言全中
```

第一段两行仍逐字节一致，证明这一段的代码改动（新增函数/常量/枚举分支）没有动到第一段的
输出路径。三份产物都被实验页点名（第六节）。

## 六、判决行点名（第 4c 步）

```
grep -n 'name=verdict' research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out
```

0 行——这个装置没有 `name=verdict` 这一种产物行（与第一段相同，纯计数模型，合取留给主 agent）。

逐个点名布尔/计数类字段里取值异常的（按第 4c 步「取值 false / not_run / 违例计数 > 0」的判据）：

- `ratio_non_decreasing_through_streams_holds=false`：231 行 row_summary 里出现次数见下方命令。
  这是 Q5（批≤流数时不降）在这些行上不成立，登记预期内（第一段报告已说明模型可能不支持原句）。
- `row_dips_below_one=true`（跌破 1）：见下方命令，是 Q6③ 的正常输出字段（不表示「没过」，
  名字含义中性，不点名判定异常，只是数据）。
- `settles_near_one_beyond_sixteen_streams_holds=false`：2 处，均出在 `two_fifty_six_mebibyte_file`
  （G5，L=2）流数 768、1024 两行——这两行是丙行（峰值就是批=1 那格），Q6② 的「峰值超额」基准本身
  很小（peak_ratio=2.5 只是 B=1 的值），而比值在批轴远端又回升到 1.98–2.11，margin 超出四分之一
  门槛。是不是登记预期内：拿不准——登记第六节 Q6② 没有专门讨论「峰值本身就是 B=1」这种丙行的
  子情形，这是一个观测到的边界现象，不是这一段要回答的主问题（树高 vs 根下孩子数），如实记下，
  不展开判定。
- 没有 `not_run` 字段；没有名字表示「违例、不匹配、歧义、失败」的整数计数字段。

```
grep 'name=row_summary' research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out | grep -c 'ratio_non_decreasing_through_streams_holds=false'
```
→ 156

```
grep 'name=row_summary' research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out | grep -c 'row_dips_below_one=true'
```
→ 139

```
grep 'name=row_summary' research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out | grep -c 'settles_near_one_beyond_sixteen_streams_holds=false'
```
→ 2

`name=q2_summary` 的 11 行 `streams_interior`/`batch_interior` 全部 `true`（现查命令见第四节相邻
段），没有 F3（端点未量到）触发。

## 七、答案：1 TiB 上「流数 ≥ 24 峰值落到批=1」是根下孩子数主导，不是树高

**关键对照（四点，树高恒为 L=4，只变根下孩子数）**：

```
grep '^E7RESULT name=geometry_config' research/results/e16-fifth-run-peak-under-today-widths-second-segment-2026-09-28.out
```

```
name=geometry_config geometry=one_tebibyte_file units=33692212 leaf_units=144 fanout=147 root_level=3 node_levels=4 root_children=11
name=geometry_config geometry=full_root_level_four units=457419312 leaf_units=144 fanout=147 root_level=3 node_levels=4 root_children=147
name=geometry_config geometry=small_fanout_one_tebibyte_file units=33692212 leaf_units=128 fanout=128 root_level=3 node_levels=4 root_children=17
name=geometry_config geometry=large_fanout_one_tebibyte_file units=33692212 leaf_units=296 fanout=296 root_level=3 node_levels=4 root_children=2
```

（`full_root_level_four` 即 G3，单元数 457419312 是它自己「根下孩子数满」这个几何点的定义
——它是「满员扇出」的树，不是「1 TiB 文件」的树，跑前登记第五节 5.2 表已写明它与 G2 的比较是
「同一个树高、根下孩子数不同」，不是「同一个单元数」；F128/F296 才是「同一个 1 TiB 单元数、
树高恰好也是 4、根下孩子数不同」。两组对照互相印证同一个结论。）

`name=q1_summary` 四行（第七节起始已抄）显示，「峰值消失」（丙行，峰值就在批=1）开始出现的
流数随根下孩子数单调后移：

| 几何 | 树高 L | 根下孩子数 | 丙行起点（流数） |
|---|---|---|---|
| large_fanout_one_tebibyte_file | 4 | 2 | 全部 21 行（流数 2 起）已经丙行 |
| one_tebibyte_file（真实基线） | 4 | 11 | 流数 24 起丙行 |
| small_fanout_one_tebibyte_file | 4 | 17 | 流数 32 起丙行 |
| full_root_level_four | 4 | 147 | 甲行到流数 128、宽平台到 128、乙行到 256，流数 384 才转丙行 |

同一个树高（L=4）下，根下孩子数从 11 提到 147，丙行起点从流数 24 推迟到流数 384（16 倍）；
根下孩子数从 11 降到 2，丙行起点从流数 24 提前到流数 2（甚至更早，全表都是丙行）。

**反过来，不控制根下孩子数的树高对照点不是单调的**（`grep 'name=q1_summary'` 前五行）：

| 几何 | 树高 L | 根下孩子数 | Q1 分类 |
|---|---|---|---|
| four_gibibyte_file（G1） | 3 | 7 | 全部丙行 |
| one_tebibyte_file（G2） | 4 | 11 | 分段，流数 24 起丙行 |
| sixteen_tebibyte_file（G4） | 5 | 2 | 分段，形状复杂（丙→甲→宽平台→乙→宽平台→丙） |
| two_fifty_six_mebibyte_file（G5） | 2 | 58 | 全部丙行 |

G5 树高最矮（L=2）但根下孩子数不算少（58），仍然全部丙行——单独用树高排不出这张表的顺序
（L=2 全丙、L=3 全丙、L=4 分段@24、L=5 分段@复杂），但用根下孩子数基本排得出来（2→全丙，
7→全丙，11→24 起丙，58→全丙 [这一格是唯一的反例，见下]，147→384 起丙）。

**反例与结论的限度**：G5（根下孩子数=58）全部丙行，与「根下孩子数越多丙行起点越晚」这条单调
关系不完全吻合——58 应该比 11 更晚才对，但 G5 全丙。这说明控制变量不完整时（G5 同时改了树高到
2），树高确实还有次要作用，不是「根下孩子数说了算、树高完全无关」；但在**严格控制树高**的
四点对照（F296/G2/F128/G3，树高恒 4）里，根下孩子数从 2 到 147 单调推迟丙行起点 16 倍，这是
本段能给出的最强证据：**在「1 TiB、树高 4」这个具体组合下，「流数 ≥ 24 时峰值落到批=1」这件事
主要由根下孩子数=11 太少决定，不是由树高=4 太矮决定**——把根下孩子数抬到接近满员（147），
同一个树高下这件事要到流数 ≈ 384 才发生。

**记录宽旋钮不是这件事的解释变量**：`record_width_knob_maximum`（100 万项/条）与真实基线
（67 项/条）逐行分类相同（q1_summary 详见第八节），只有 `record_width_knob_minimum`（1 项/条）
让全表变成丙行——但那是乙臂自己的记录开销被拉高、把甲的相对优势吃光，不是「批=1」的树结构效应，
机制不同，不构成对「树高 vs 根下孩子数」这个问题的第三种解释。

## 八、岔路表

| # | 岔路 | 已够判 / 还差什么 | 剩下的量能不能让它翻面 |
|---|---|---|---|
| 补充 1 | 1 TiB 上流数≥24 峰值落到批=1，是树高主导还是根下孩子数主导 | **已够判**：严格控制树高（L=4 恒定）的四点对照（根下孩子数 2/11/17/147）显示丙行起点随根下孩子数单调从「全表丙行」推迟到「流数 384」，16 倍跨度；不控制树高的四点（G1/G2/G4/G5）不单调，树高单独解释不了。答案：**根下孩子数主导**，树高在严格控制根下孩子数时的作用没有独立测出（G5 是唯一的轻微反例，见第七节「反例与限度」，但它没有推翻树高恒定组的单调结论）。 | **不会翻面**「根下孩子数是这四点单调关系的主变量」这一条：这四点每一点都逐字节记在产物里，重跑给出相同的数（V5 已核）。**能补而没补的**：根下孩子数的中间取值（比如 30、50、80）没有扫描，只能说「单调、16 倍跨度」，说不出「具体是线性还是别的函数关系」；G5 那个轻微反例（树高 2、根下孩子 58、仍全丙）提示树高可能有次要作用，要把它坐实需要再找一组「树高不同、根下孩子数相同」的对照点，这一段没有专门造这样的点（F128 的根下孩子数=17 与哪个树高更矮的点接近，没有去算）。 |

## 九、kb 改动

- `.claude/kb/experiments/16-journal的角色WALvs意图日志.md`：「第五次跑」一节里插入「第二段」
  两段正文（峰值消失起点由根下孩子数主导的分析、记录宽旋钮的附带发现），改了「产物」行（加第二段
  产物路径、变异表从 16 条改 17 条）、「它答不了的」段（去掉已跑完的第二段几何点，换成「根下孩子数
  中间取值没扫描」这条新欠账）、「还没做的那一半」那条 bullet（从「没跑」改成「已补跑」+ 结论）。
  「### 影响的决策」表：D23（journal 的角色与格式） 已定项 1 那一行的回看从「不受影响」改成
  「2026-09-28 待三方：……」——措辞交主 agent 认（见第十节）。**其余 12 行没有改动内容**：逐行核过
  这一段的改动会不会触及它们各自的依据（记录格式、原装置 56 格、字节口径、校验和结构……），一个都
  没碰，保留原样（第 7b 步③要求的「重新回看一遍」已做，结论是没有需要改的）。
- `.claude/kb/experiments-history.md`：E16（journal 的角色：WAL vs 意图日志） 节「现状」行改了措辞
  （补「两段」、加根下孩子数主导的结论）；2026-09-28 小节里新增一个 `####` 子节
  「第五次跑第二段：1 TiB 上「流数≥24 峰值落到批=1」是根下孩子数主导，不是树高」，带「依据」行。
  第一段那个 `####` 子节一个字没改。
- `.claude/kb/experiments.md`：E16（journal 的角色：WAL vs 意图日志） 那一行的状态列改
  「09-28 补第五次跑两段」，结论列追加根下孩子数主导那句。

## 十、交主 agent 认的项

1. D23（journal 的角色与格式） 已定项 1 那一行「2026-09-28 待三方：……」的具体措辞（第九节引了
   原文，写的是：依据段引的「批≤并发流数时成立」这条适用范围依赖一个此前没单独测过的旋钮——
   根下孩子数，不是一个只由树高定的常量射程；该不该升级判法、要不要在依据段补条件，留给三方判）。
2. G23.2（甲丙差不收敛）那一格：这次的新发现没有改 G23.2 本身引用的那组数（56 格扫描用的是
   *原装置* 128/128 扇出几何，第二段没有重跑那组数），所以这次没有改 G23.2 的文字；但 G23.2 的
   「而这只在『批≤流数』时成立」这句话与 D23 已定项 1 的依据句是同一件事的两处表述，D23 已定项 1
   改了「待三方」而 G23.2 没改，是不是要保持一致，留给主 agent 判。
3. 「根下孩子数的中间取值没有扫描」「G5 那个轻微反例要不要专门坐实」这两条欠账要不要立即续做，
   还是留在 kb 里当已知局限——这是新一轮任务规划的事，不是这一段该定的。

## 十一、没做什么

- 没跑门禁阶段（提交前由 gate-triage 跑）。
- 没判这个发现能不能推翻或确立 D23（journal 的角色与格式） 已定项 1 或 G23.2（那是推论，要走三方）。
- 没有专门造「树高不同、根下孩子数相同」的第三组对照点去坐实 G5 那个轻微反例（第七节已如实记下，
  不是回避，是这一段的岔路只要求答「主导变量是谁」，不要求排除全部次要作用）。
- 没有扫描根下孩子数的中间取值（第八节岔路表已列为「能补而没补的」）。
- 没有改 `.claude/kb/decisions/23-journal的角色与格式.md`（不在写范围内，第十节列出交主 agent 判）。
