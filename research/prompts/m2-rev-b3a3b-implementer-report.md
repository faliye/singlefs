# 实审 B3a-3b 报告（implementation-writer）：层 0 七份测试文件跟上按设备切段 / 第三态，映射 key 新不变量归类

时刻都是 UTC（本机时钟），JST = UTC + 9。开工 2026-09-26T22:44Z。规格 `/tmp/claude-1000/impl-rev-b3a3b/spec.md`。

## 一、结论

- 规格表里 7 份文件都改了，只动测试代码，被测代码没动：
  - 6 份层 0 文件原来拿 `closed_form_state_count(&expanded)` 或写死的两态数去比 `tally.states`，现在改成比 `layer0_state_count_with_torn_in_place_overwrites(base, writes, segments, 那条用例的展开)`。写死的数照闭式换成三态口径的新值。原来那个两态的数留成「每次写只取两态时的闭式」一条断言，不再去比 `tally.states`。
  - `first_transaction_step_seven_layer0.rs` 把 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 挪进 `only_under_the_new_root` 那一格。挪之前核过两样：一是代码路径，二是探针在 5 个镜像上逐个取的判定（第四节），都对得上「只在最新根有文件时有对象」。
- 每个钉值的改前数、改后数、算式都在第三节。探针在今天的流上现算，只 `prepare` 再调状态数函数，不枚举。
- **要主 agent 先知道的两件事**（第八节 F1、F2）。这两件都不是这一轮带出来的，改法要做设计判断，我没动：
  - F1：有 4 份层 0 文件，今天在 `prepare` 那几条形状断言上就红，走不到状态数：`formatted_pool`、`parallel_line_one`、`spill_over`、`tree_split`。原因是第一个文件版本那次发布从 16 个单元写涨到 24 个（第一条流那一段 18 → 26），各条流写的单元也跟着变多，树分裂那几条流的映射树起点样子也变了。所以这几份的快档改完状态数之后照样红，要等流重新布置。
  - F2：按位置寻址的树那份，快用例在 `println!` 里对每一段算 `closed_form_state_count(&prepared.segments)`。第三条流 `ExtentLowerSegmentGrowsToTwoLevels` 的段是 `[322, 290, 1, 2]`，`1u64 << 322` 在 debug 和 release 下都会 panic，因为工作区 `Cargo.toml` 的 release 也开着 `overflow-checks = true`。这份文件没跑过：文件头写着「一条都没跑」。
- 变异：
  - 追加 7 行、替换 3 行，文件是 `/tmp/claude-1000/impl-rev-b3a3b/mutations-append.tsv` 和 `mutations-replacements.tsv`。
  - 10 行点的都是 layer0 目标，我证不了红，全部留给 59 号。
  - 另外在副本里逐条施加变异、只跑探针，看到每条变异都改动了用例钉的那个量（第六节）。这只是旁证，不是证红。
- 什么现象会推翻这些结论：
  - 提交时跑 layer0 快档，`acquisition_barrier` 不是 262152，或者 `step_seven` 快档里 `I-MAPPING-KEY` 评估过的状态数不是 `ROOT_PERSISTED_STATES`（9）；
  - F1 那 4 份把 `prepare` 修好之后，快档状态数不是第三节写的 37 / 117 / 25 / 13；
  - `position_addressed` 修好 F2 之后，五条流快档的 `tally.states` 不等于三态函数，按第三节应当是 25 / 13 / 10 / 13 / 33；
  - 追加的 7 行在 59 号上有一行不红。

## 二、写过的文件

只写了这 7 份，都在 `crates/singlefs-harness/tests/`。改动行数是对开工时的快照 `/tmp/claude-1000/impl-rev-b3a3b/originals/` 用 `git diff --no-index --numstat` 数的：

| 文件 | +/− |
|---|---|
| `first_transaction_step_seven_layer0.rs` | +24/−4 |
| `second_transaction_step_three_formatted_pool_layer0.rs` | +29/−7 |
| `second_transaction_step_three_acquisition_barrier_layer0.rs` | +27/−5 |
| `second_transaction_position_addressed_trees_layer0.rs` | +18/−12 |
| `second_transaction_parallel_line_one_layer0.rs` | +52/−12 |
| `second_transaction_parallel_line_three_spill_over_layer0.rs` | +28/−8 |
| `second_transaction_supplement_two_tree_split_layer0.rs` | +23/−17 |

全份 diff 在 `/tmp/claude-1000/impl-rev-b3a3b/logs/my-changes-vs-originals.diff`，7 段 511 行。

`crates/mutations.tsv` 主表没改。要追加的 7 行写在 `/tmp/claude-1000/impl-rev-b3a3b/mutations-append.tsv`，变异名如下：
1. `实审 B3a-3b：原地覆写的写也只取两态（只做过 mkfs 的池那条流快档退回 22）`
2. `实审 B3a-3b：原地覆写的写也只取两态（取号屏障那条流展开的两段退回 1 + 3 + 262143）`
3. `实审 B3a-3b：层 0 状态数函数不认原地覆写（按位置寻址的树那几条流枚举出来的比它多）`
4. `实审 B3a-3b：原地覆写的写也只取两态（并行线一快档退回 102）`
5. `实审 B3a-3b：原地覆写的写也只取两态（L8 退回 20 个状态、根槽已落的 4 个）`
6. `实审 B3a-3b：原地覆写的写也只取两态（树分裂快档每条流退回 8）`
7. `实审 B3a-3b：checker 在没有文件的一版上也判一格映射 key 与单元头相符（第一条流快档里它不再只在 txg 3 的根已持久时评估）`

要整行替换的 3 行写在 `/tmp/claude-1000/impl-rev-b3a3b/mutations-replacements.tsv`，是主表第 23、289、474 行，名字不变（第六节）。

这两份都是六段、制表符分隔，`awk -F'\t' '{print NF}'` 数出来 10 行全是 6 段。追加的 7 行用脚本核过：名字在主表里都不存在，原文在各自的源文件里都恰好命中 1 次。

## 三、每份文件：今天比的是什么、改成什么、钉值（改前、改后、算式）

口径：「两态」是每次写只取没持久 / 持久；「三态」是层 0 今天的枚举域，原地覆写多一态「新旧都读不出」（`crash::TearableInPlaceOverwrites`）。这 6 份流里被认成原地覆写的全是系统配置槽写，每段两次（两块盘各一次），探针逐段打出来的 `tearable=` 里没有别的种类。所以只含两次系统配置槽写的段是 3² − 1 = 8（两态时 3）；带两次系统配置槽写加 k 次单元写的段，全量是 3² · 2^k − 1（两态时 2^(k+2) − 1）。记录段、根槽段、纯单元写段照旧两态。

| 文件 / 用例 | 改前比什么、钉多少 | 改后比什么、钉多少 | 算式（改后；括号里是改前） | 探针在今天的流上 |
|---|---|---|---|---|
| `formatted_pool` 快用例 `every_crash_state_outside_the_unit_segment_…` | `tally.states == closed_form_state_count(&expanded)`，`== 22` | `closed_form(expanded) == 22`（两态，保留）；`tally.states ==` 三态函数（第 161 行起）；`== 37`（第 177 行） | 1 + 三个系统配置槽 2 写段各 (3² − 1) + 三个记录 2 写段各 3 + 三个根槽段各 1 = 1 + 24 + 9 + 3 = 37（1 + 6·3 + 3 = 22） | two 22 → three 37（单元写那一段不展开，18 还是 26 都不影响这个数） |
| `formatted_pool` `the_formatted_pool_mount_…_segment_sequence` | 两态闭式 `== 262_165`，不比 `tally.states` | 没改（只把文档改成「每次写只取两态时的闭式」） | — | 今天是 67108885，这是 F1 的一部分 |
| `acquisition_barrier` | `tally.states == closed_form(expanded)`，`== 1 + 3 + 262_143` | `closed_form(expanded) == 1 + 3 + 262_143`（两态，保留）；`tally.states ==` 三态函数（第 204 行起）；`== 1 + (3 * 3 - 1) + 262_143`（第 218 行） | 取号那一段 2 次系统配置槽写 3² − 1 = 8，写行那一段 18 个单元写 2^18 − 1 = 262143，加全部持久 1，合计 262152（262147） | two 262147 → three 262152；段序列逐段见下表 |
| `position_addressed` 的 `enumerate()`（快用例与全量都经它） | `tally.states == closed_form_state_count(&expanded)`，没有写死的数 | `tally.states ==` 三态函数（第 415 行起），删掉 `expanded` | 快档（写数 < 10 的段展开）：Inline `[28,4,1,2]` 1+15+1+8 = 25（20）；BackToInline `[24,2,1,2]` 1+3+1+8 = 13（8）；GrowsToTwoLevels `[322,290,1,2]` 1+1+8 = 10（5）；AllocationRecordTree `[28,2,1,2]` 13（8）；VersionWithoutFile `[2,14,2,1,2,2,1,2]` 1+8+3+1+8+3+1+8 = 33（18） | 五条都与算式同；全量（能枚举的 4 条）：Inline 268435475 → 268435480，BackToInline 16777223 → 16777228，AllocationRecordTree 268435463 → 268435468，VersionWithoutFile 16401 → 16416 |
| `parallel_line_one` 快用例 | `tally.states == closed_form(expanded)`，`== 102` | `closed_form(expanded) == 102`（两态，保留）；`tally.states ==` 三态函数（第 291 行起）；`== 1 + 3 * (3 * 3 - 1) + 3 * 3 + 5 + 15 + 63`（第 307 行） | 1 + 三个系统配置槽 2 写段各 8 + 三个记录 2 写段各 3 + 五个根槽段 + 4 写段 15 + 6 写段 63 = 117（102） | two 102 → three 117（展开的小段在今天的流上逐段相同） |
| `parallel_line_one` 全量（ignore） | `closed_form = closed_form_state_count(segments)`，钉 `5_505_123`，`tally.states == closed_form` | 两态闭式改名 `closed_form_with_two_states_per_write`，两条钉值不变；`closed_form` 换成三态函数（`full_expansion`，第 340 行），钉算式与 `12_386_418`（第 358 行）；`tally.states == closed_form`，`#[ignore]` 的说明与文件头的数跟着改 | 1 + 3·8 + 3·3 + 5 + (9·2^16 − 1) + (9·2^18 − 1) + 15 + (9·2^20 − 1) + 63 = 12386418（5505123）。按文件里写着的段 `[2,2,1,2,2,1,18,2,1,20,4,1,22,6,1,2]` 算 | 今天的流是 `[…,26,…,30,…,32,…]`：两态 5435818083、三态 12230590578（F1） |
| `spill_over` | `tally.states == 1 + 15 + 1 + 3`；`counts.root_persisted == 4` | `tally.states ==` 三态函数（第 276 行起）；`== 1 + ((1 << 4) - 1) + ((1 << 1) - 1) + (3 * 3 - 1)`（第 292 行）；`root_persisted: 1 + (3 * 3 - 1)`（第 301 行）；其余三格不变 | 记录段 15 + 根槽 1 + 系统配置槽 3² − 1 = 8 + 全部持久 1 = 25（20）；根槽已落 = 轮换那一段 8 + 全部持久 1 = 9（4）；记录段那 15 个与根槽段那 1 个都是两态，`no_record 1 / only_one 6 / both 9` 不变 | two 20 → three 25；根槽之后的段 4 → 9（三态函数只展开根槽之后那一段算的） |
| `tree_split` 的 `enumerate()` | `tally.states == closed_form(expanded)` | `tally.states ==` 三态函数（第 311 行起），删掉 `expanded` | 快档与全量都从这里过 | — |
| `tree_split` 快用例 | `== 1 + 3 + 1 + 3` | `== 1 + 3 + 1 + (3 * 3 - 1)`（第 347 行） | 13（8） | 七条流都是 8 → 13 |
| `tree_split` 全量（ignore） | `== 1 + (2^(2u) − 1) + 3 + 1 + 3` | `… + 3 + 1 + (3 * 3 - 1)`（第 363 行） | 按文件里的 u：u=6 → 4108（4103），u=7 → 16396（16391），u=8 → 65548（65543）；七条合计 225364（225329），「约 22 万」仍对 | 今天的 u 是 11/14/21/12/12/10/11（F1） |
| `step_seven` | 状态数 B3a-2 已改成三态口径 | 没改数 | 探针复算：全量两态 67108885、三态 150994980，快档两态 29、三态 54，与文件里的常量相同 | 同左 |

`acquisition_barrier` 的段序列逐段是这样。按设备记屏障之后，这条流的段序列与改之前逐字相同，展开的还是第 13、14 段：每道池屏障两块盘都发，每块有写的盘都被自己那道屏障放行，所以段照旧在池屏障处关。取号写之前、取号之后那两道都是池屏障。`kinds` 与 `tearable` 取自探针原样行（`logs/probe-run-probe_b3a3b_acquisition_barrier.round1.log` 的 `PROBE_SEGMENT`）：

| 段 | 写数 | 种类 | 原地覆写（写表下标） | 两态 | 三态 | 展开 |
|---|---|---|---|---|---|---|
| 0 | 2 | 系统配置槽 ×2 | 0、1 | 3 | 8 | 否 |
| 1 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 2 | 1 | 根槽 | — | 1 | 1 | 否 |
| 3 | 2 | 系统配置槽 ×2 | 5、6 | 3 | 8 | 否 |
| 4 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 5 | 1 | 根槽 | — | 1 | 1 | 否 |
| 6 | 26 | 系统配置槽 ×2 + 单元 ×24 | 10、11 | 67108863 | 150994943 | 否 |
| 7 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 8 | 1 | 根槽 | — | 1 | 1 | 否 |
| 9 | 26 | 系统配置槽 ×2 + 单元 ×24 | 39、40 | 67108863 | 150994943 | 否 |
| 10 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 11 | 1 | 根槽 | — | 1 | 1 | 否 |
| 12 | 2 | 系统配置槽 ×2（B 的轮换） | 68、69 | 3 | 8 | 否 |
| 13 | 2 | 系统配置槽 ×2（重开取号） | 70、71 | 3 | 8 | **是** |
| 14 | 18 | 单元 ×18（写行） | — | 262143 | 262143 | **是** |
| 15 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 16 | 1 | 根槽 | — | 1 | 1 | 否 |
| 17 | 18 | 系统配置槽 ×2 + 单元 ×16 | 93、94 | 262143 | 589823 | 否 |
| 18 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 19 | 1 | 根槽 | — | 1 | 1 | 否 |
| 20 | 18 | 系统配置槽 ×2 + 单元 ×16 | 114、115 | 262143 | 589823 | 否 |
| 21 | 2 | 记录 ×2 | — | 3 | 3 | 否 |
| 22 | 1 | 根槽 | — | 1 | 1 | 否 |
| 23 | 2 | 系统配置槽 ×2 | 135、136 | 3 | 8 | 否 |

整条流共 137 写、原地覆写 18 次。全量两态 135004199、三态 303431744，甲二快档两态 73、三态 138。这几个数只是记一下，文件里没钉。

不改的：`formatted_pool` 的 `the_formatted_pool_mount_…_segment_sequence` 与 `formatted_pool_mount_stream_has_the_same_base_writes_and_segments_…`，`tree_split` 的 `the_accounting_streams_read_the_tree_height_…`，`step_seven` 的阳性对照与记录核对器那几条。这些用例都不比 `tally.states`，按设备切段与第三态碰不到它们。它们会不会因为 F1 而红，要看 `prepare`。

## 四、`MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 归进 `only_under_the_new_root`：怎么核的

**代码路径**（`crates/singlefs-checker/src/walk.rs`，行号现查）：
- 判这条的 5 处（第 872、897、924、938、954 行）全在 `walk_central_mapping_entries`（第 859 行）里，逐条判中央映射树叶里的条目。没有条目就一格都不判，整条报「不适用」。
- `walk_central_mapping_entries` 只在第 842 行被调，调它的是 `walk_tree_table_and_central_mapping_root`（第 800 行）。条目来自 `walk_code_two_subtree` 收的叶条目；映射根指针全零时，第 991 行直接返回，不收条目。
- `walk_tree_table_and_central_mapping_root` 只有两个调用方：`walk_root`（第 763 行）与 `walk_version_applied_only_by_records`（第 795 行）。
- `walk_root` 的调用点有 4 处：
  - 第 5568 行（最新根）、第 5663 行（回退候选根）：判定进报告；
  - 第 4006 行（收位置项的 `harvesting_walk`）、第 5401 行（被抛弃根）：各用自己的 `Judgements::default()`，判定丢掉、不进报告。
- `walk_version_applied_only_by_records`（第 5694 行）只走 `versions_applied_only_by_records` 认出来的版本。它的条件 ①（第 5312 行 `applied_by_a_recovery`）要最新根指着的实例表里有一行，记着恢复施加到过这一版的 txg。
- I-3.10 读「下一次挂载会先施加的那一版」走的是 `tree_table_pointer_of_the_version_the_next_mount_applies_first`（第 3335 行），只读树表指针去收分配记录，不走映射树。

**在第一条流上**：
- 种子根与两次暖机的根都是没有文件的一版，映射根指针全零，走不到条目。
- txg 3 的根有文件，映射树里有条目。
- txg 3 的记录已落、根槽没落时，由记录施加出来的那一版不走。实例表的行是挂载写行时写的，这条流上唯一一次挂载在 txg 3 之前，没有哪一行能记着施加到 txg 3，条件 ① 不成立。
- 所以这条只在 txg 3 的根已持久的状态上有对象，与同一格里那 15 条同一个集合。

**探针旁证**（不枚举，只在 5 个手摆的镜像上各跑一次 `check_pool_image`）：源码 `draft/probe/generated/probe_b3a3b_step_seven.rs`，日志 `logs/probe-run-probe_b3a3b_step_seven.log`。原样行：
```text
PROBE_STEP_SEVEN write_count=41 root_index=38 first_unit=12 record_writes=[36, 37]
PROBE_MAPPING_KEY case=nothing_persisted mapping_key=Some(NotApplicable("这个镜像上没有它判的对象"))
PROBE_NOT_APPLICABLE case=nothing_persisted count=23 ["I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15", "I-MAPPING-KEY"]
PROBE_MAPPING_KEY case=everything_before_the_first_unit_write mapping_key=Some(NotApplicable("这个镜像上没有它判的对象"))
PROBE_NOT_APPLICABLE case=everything_before_the_first_unit_write count=21 ["I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.7", "I-8.8", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15", "I-MAPPING-KEY"]
PROBE_MAPPING_KEY case=units_and_records_of_txg_three_without_its_root mapping_key=Some(NotApplicable("这个镜像上没有它判的对象"))
PROBE_NOT_APPLICABLE case=units_and_records_of_txg_three_without_its_root count=20 ["I-1.10", "I-3.1", "I-3.9", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.7", "I-8.8", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15", "I-MAPPING-KEY"]
PROBE_MAPPING_KEY case=txg_three_root_persisted_rotation_not mapping_key=Some(Holds)
PROBE_NOT_APPLICABLE case=txg_three_root_persisted_rotation_not count=4 ["I-7.9", "I-8.7", "I-8.8", "I-9.14"]
PROBE_MAPPING_KEY case=every_write_persisted mapping_key=Some(Holds)
PROBE_NOT_APPLICABLE case=every_write_persisted count=4 ["I-7.9", "I-8.7", "I-8.8", "I-9.14"]
```
5 个镜像上违例都是空的（`PROBE_VIOLATED … []`）。

对着用例里的分类逐格对：
- 「txg 3 的单元与记录已落、根没落」那个镜像：`I-MAPPING-KEY` 与 `only_under_the_new_root` 那 15 条一起报不适用，I-3.10 已经评估。它们在这里分开了，所以映射 key 不能归进 I-3.10 那一格，只能归「只在新根下」。
- 根已持久的两个镜像：只剩 `never_comparable_on_this_stream` 那 4 条不适用。
- 什么都没持久的镜像：不适用的 23 条正好是 16 + 1 + 4 + 2 这几格，其余 23 条都评估了。所以文档里「其余 22 条」改成了「其余 23 条」：46 − 16 − 1 − 4 − 2 = 23，清单是 `image.rs` 里 `IMPLEMENTED_INVARIANTS: [&str; 46]`。

5 个镜像不是全部状态。快档 54 个状态里评估过的状态数要在提交时跑层 0 才核得到；按上面的代码路径推，应当等于 `ROOT_PERSISTED_STATES` = 9。

`step_seven` 的改动：
- `use singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;`
- 数组末尾加一格（第 135 行），rustfmt 把数组排成一行一条；
- 文档注释第 83–101 行：「15 条」改「16 条」，补一段写映射 key 的理由，「其余 22 条」改「其余 23 条」。

## 五、探针（怎么算的钉值）

- 副本：`/tmp/claude-1000/impl-rev-b3a3b/copy`，22:44:20Z 用 `rsync -a --exclude target --exclude .git` 从主工作区取，用它自己的 target。
  - 取之前记了 13 份文件的 sha256（`logs/sha-at-start.txt`），包括 A2b / A2c / A4b 在改的 core 文件与 `crash.rs`、`walk.rs`、`image.rs`；副本里逐份相同。
  - 副本的 core 编得过，没有拿 `impl-rev-a2b/orig`、`impl-rev-a2c/orig` 顶任何文件。
- 探针的做法：
  - 7 份层 0 文件各整份拷成一个名字不含 layer0 的目标（`tests/probe_b3a3b_*.rs`），摘掉原有的 `#[test]` / `#[ignore]`，末尾加一条探针 `#[test]`：只调 `prepare`，再调状态数函数，不枚举。
  - `acquisition_barrier` 的建流写在用例里面，所以探针在 `enumerate_layer0_selecting_versions` 那一行之前截断，接上算数。
  - 共用的打印件在 `tests/probe_b3a3b_common/mod.rs`：逐段打写种类、原地覆写下标，以及这一段单独展开时的两态 / 三态数；再打整条流在这份展开下、全量、甲二快档下的两态 / 三态数。
- 生成器 `draft/probe/make_probes.py`，生成出来的源码在 `draft/probe/generated/`，共用件 `draft/probe/probe_b3a3b_common.rs`，驱动 `draft/run-probes.sh`。
- 探针跑了三趟：
  - 第一趟（`logs/probe-run-*.round1.log`）有 4 份在 `prepare` 的形状断言上 panic，这就是 F1。
  - 第二趟 `research/scripts/capped.sh` 被别的会话改坏，7 条都退 2，没跑（F4）。
  - 第三趟（`logs/probe-run-*.log`）把探针里原用例的 `assert_eq!` 换成「不等就打一行 `PROBE_MISMATCH`」，在今天的流上照样算出数，7 条都 `test result: ok. 1 passed`。
- 每条都经 `run-with-memory-cap.sh 8G`，线程上限：前两趟 `capped.sh 5`；第三趟主树那份坏着，用的是副本里的 `capped.sh`，它与 HEAD 逐字相同。

第三趟 `PROBE_MISMATCH` 原样（F1 的依据）：
```text
formatted_pool: PROBE_MISMATCH line=74 left=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2] right=[2, 2, 1, 2, 2, 1, 18, 2, 1, 2] message=取号两写 | 写行发布的记录两写 | 根 | 系统配置两写 | 暖机的记录两写 | 根 | 系统配置两写与第一个文件版本的十六个单元写 | 记录两写 | 根 | 系统配置两写
formatted_pool: PROBE_MISMATCH line=79 left=41 right=33 message=取号 2 + 两次零单元发布各 5 + 文件版本 21
parallel_line_one: PROBE_MISMATCH line=155 left=14 right=9 message=B：9 个单元（≤ 10）
parallel_line_one: PROBE_MISMATCH line=163 left=15 right=10 message=C：10 个单元（≤ 10）
parallel_line_one: PROBE_MISMATCH line=170 left=[2, 2, 1, 2, 2, 1, 26, 2, 1, 30, 4, 1, 32, 6, 1, 2] right=[2, 2, 1, 2, 2, 1, 18, 2, 1, 20, 4, 1, 22, 6, 1, 2] message=…
spill_over: PROBE_MISMATCH line=103 left=74 right=68 message=63 片叶容器 + 5 个角色
spill_over: PROBE_MISMATCH line=104 left=(1, 67, 7) right=(1, 67, 1) message=两条记录：装满 67 项的一条 + 跨出去的 1 项
spill_over: PROBE_MISMATCH line=118 left=[148, 4, 1, 2] right=[136, 4, 1, 2] message=只录这一次发布：68 个单元 × 2 盘、两条记录 × 2 盘、根槽 FUA、系统配置槽 × 2 盘
tree_split: CentralMappingRootSplit 之前 "L10"（钉 "L6"）、之后 "L3 L3 L4 I3"（钉 "L3 L3 I2"）、写出 11 个单元（钉 6）、段 [22, 2, 1, 2]（钉 [12, 2, 1, 2]）
tree_split: CentralMappingLeafSplit 之前 "L3 L3 L4 I3"、之后 "L3 L2 L2 L2 L1 I2 I3 I2"、14 个单元、段 [28, 2, 1, 2]
tree_split: CentralMappingTwoLevelsSplitInARow 之前 "L3 L3 L4 I2 I1 I2"、之后 "L3 L2 L2 L2 L1 I1 I2 I1 I1 I2 I1 I1 I2 I1 I2"、21 个单元、段 [42, 2, 1, 2]
tree_split: CentralMappingEmptyLeafDropped 之前 "L2 L2 L2 L2 L2 L2 I6"、之后 "L2 L6 L4 I3"、12 个单元、段 [24, 2, 1, 2]
tree_split: CentralMappingRootLowered 之前 "L3 L3 L4 I3"、之后 "L10"、12 个单元、段 [24, 2, 1, 2]
tree_split: AccountingRootSplit 形状对得上，10 个单元（钉 6）、段 [20, 2, 1, 2]；AccountingLeafSplit 形状对得上，11 个单元（钉 7）、段 [22, 2, 1, 2]
```
（`tree_split` 那几行是从 `logs/probe-run-probe_b3a3b_tree_split.log` 的 `PROBE_MISMATCH` 行缩写的，原行带整份 `rewritten` 清单，太长；原行在日志里。`acquisition_barrier`、`position_addressed`、`step_seven` 三份的 `PROBE_MISMATCH` 计数都是 0。）

第三趟状态数原样（摘 `PROBE` / `PROBE_FULL` / `PROBE_SPILL` 行）：
```text
PROBE formatted_pool_fast writes=41 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2] tearable_total=8 two_state_selected=22 three_state_selected=37
PROBE acquisition_barrier writes=137 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2, 2, 18, 2, 1, 18, 2, 1, 18, 2, 1, 2] tearable_total=18 two_state_selected=262147 three_state_selected=262152
PROBE ExtentInlineToLowerSegment writes=35 sizes=[28, 4, 1, 2] tearable_total=2 two_state_selected=20 three_state_selected=25
PROBE ExtentLowerSegmentBackToInline writes=29 sizes=[24, 2, 1, 2] tearable_total=2 two_state_selected=8 three_state_selected=13
PROBE ExtentLowerSegmentGrowsToTwoLevels writes=615 sizes=[322, 290, 1, 2] tearable_total=2 two_state_selected=5 three_state_selected=10
PROBE AllocationRecordTreeTwoLeavesPerDevice writes=33 sizes=[28, 2, 1, 2] tearable_total=2 two_state_selected=8 three_state_selected=13
PROBE VersionWithoutFileRowPublishWithItsOwnTree writes=26 sizes=[2, 14, 2, 1, 2, 2, 1, 2] tearable_total=6 two_state_selected=18 three_state_selected=33
PROBE_FULL ExtentInlineToLowerSegment two_state_full=268435475 three_state_full=268435480 two_state_quick=21 three_state_quick=26
PROBE_FULL ExtentLowerSegmentBackToInline two_state_full=16777223 three_state_full=16777228 two_state_quick=9 three_state_quick=14
PROBE_FULL AllocationRecordTreeTwoLeavesPerDevice two_state_full=268435463 three_state_full=268435468 two_state_quick=9 three_state_quick=14
PROBE_FULL VersionWithoutFileRowPublishWithItsOwnTree two_state_full=16401 three_state_full=16416 two_state_quick=19 three_state_quick=34
PROBE parallel_line_one_fast writes=115 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 30, 4, 1, 32, 6, 1, 2] tearable_total=12 two_state_selected=102 three_state_selected=117
PROBE_FULL parallel_line_one_fast two_state_full=5435818083 three_state_full=12230590578 two_state_quick=123 three_state_quick=168
PROBE spill_over writes=155 sizes=[148, 4, 1, 2] tearable_total=2 two_state_selected=20 three_state_selected=25
PROBE_SPILL root_segment=2 root_persisted_two_state=4 root_persisted_three_state=9 record_writes=[[148, 149], [150, 151]]
PROBE step_seven writes=41 sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2] tearable_total=8 two_state_selected=67108885 three_state_selected=150994980
PROBE_FULL step_seven two_state_full=67108885 three_state_full=150994980 two_state_quick=29 three_state_quick=54
```
`tree_split` 七条的 `PROBE` 行都是 `two_state_selected=8 three_state_selected=13`。
（`PROBE_MISMATCH` 那一块里，每行前面的「文件名:」是我加的，好分清是哪份的；`parallel_line_one` 第 170 行的 `message=` 太长，缩成了「…」，原文在 `logs/probe-run-probe_b3a3b_parallel_line_one.log`。其余行原样。）

## 六、变异

**主表里点名这 7 个目标的行**。用脚本逐行核了两样：原文在源文件里命中几次；`--` 之后的过滤串对得上文件里几条 `#[test]`。
- 第 58、59、66、298、414、415、518、519、575、576、967 行：原文各命中 1 次，过滤串各对上 1 条用例，点名的用例都在。这一轮只改测试里的比较与钉值，它们的锚点都在 `src/` 里，没断。
- **第 23、289、474 行**：原文各命中 1 次，但点名的用例 `layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape` 在 `step_seven` 里已经没有了（HEAD 里有，第 482 行；工作区里改名成 `layer0_quick_tier_matches_the_full_tally_shape`）。过滤串 `layer0_partial_enumeration` 对上 0 条，59 号跑到会报「点名的测试 … 没跑到」。
  - 不是这一轮改的名。
  - 替换行把第五段改成 `-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_quick_tier_matches`，第六段改成 `layer0_quick_tier_matches_the_full_tally_shape`，其余四段原样。
  - 快档展开的状态包含原来那条部分枚举的全部状态（原来只跳过 18 写那一段，快档对它只少了「单元写只落一部分」的组合），所以推它们在快档上照样红。这是推的，要 59 号证。

**追加的 7 行**（`mutations-append.tsv`，都点 layer0 目标）：

| # | 改坏哪一行 | 点名的用例 | 该红在哪条断言 |
|---|---|---|---|
| 1 | `crash.rs` `            Self::NotPersistedTornOrPersisted => 3,` → `2` | `formatted_pool` `every_crash_state_outside_the_unit_segment_…` | `tally.states == 1 + 3 * (3 * 3 - 1) + 3 * 3 + 3`：变异后枚举与三态函数都退回两态（22），前一条比较相等，这一条红。今天 `prepare` 先红（F1），59 号会记「抓到」，但红在别处 |
| 2 | 同上 | `acquisition_barrier` `no_unit_of_the_new_instance_…` | `tally.states == 1 + (3 * 3 - 1) + 262_143`（变异后 262147） |
| 3 | `crash.rs` `layer0_state_count_with_torn_in_place_overwrites` 里 `&TearableInPlaceOverwrites::of(base, writes),` → `&TearableInPlaceOverwrites::none(writes.len()),` | `position_addressed` `every_position_addressed_tree_stream_recovers_cleanly_…` | `enumerate()` 里 `tally.states ==` 三态函数：枚举那边照旧三态（`crash.rs:2800` 另认一次原地覆写），函数退回两态。第一条流就是 25 对 20。这一份没有写死的数，第 1 类变异让两边一起变、红不了，所以换成这一条 |
| 4 | 同 1 | `parallel_line_one` 快用例 | `tally.states == 1 + 3 * (3 * 3 - 1) + 3 * 3 + 5 + 15 + 63`（变异后 102）。今天 `prepare` 先红（F1） |
| 5 | 同 1 | `spill_over` | `tally.states == … + (3 * 3 - 1)`（变异后 20），`counts.root_persisted` 变异后 4。今天 `prepare` 先红（F1） |
| 6 | 同 1 | `tree_split` 快用例 | `tally.states == 1 + 3 + 1 + (3 * 3 - 1)`（变异后 8）。今天 `prepare` 先红（F1） |
| 7 | `walk.rs` `        self.walk_central_mapping_entries(&leaf_entries);` 前面插一句 `self.judgements.judge(MAPPING_KEY_MATCHES_THE_UNIT_HEADER, true, String::new);` | `step_seven` `layer0_quick_tier_matches_the_full_tally_shape` | `assert_checker_counts` 里 `I-MAPPING-KEY 评估过的状态数`：变异后没有文件的版本也判一格，每个走得到树表的状态都评估；新归的那一格要 9，所以红。改之前那一格（`else` → 全部状态）在这条变异下会绿，这一行钉的就是这次归类 |

**证红**：
- 10 行都点名字带 layer0 的目标，`prove-red.sh` 会跳过，我也不许跑这类目标，所以一行都没证，**全部留给 59 号**：追加 7 行、替换 3 行。
- 旁证：在副本里逐条施加变异，只跑探针，看变异是不是改动了用例钉的那个量。
  - 脚本 `draft/mutation-effect-probes.sh`，日志 `logs/mutation-*.log`，都 `test result: ok. 1 passed`。
  - 跑之前把副本的 `crash.rs`、`walk.rs` 另存进 `draft/copy-originals/`，每条跑完从那里拷回并 `touch`。跑完 sha256 是 `baa9fe35…`（crash.rs）、`07ec2b47…`（walk.rs），与开工时相同。
- 旁证的原样：
```text
# 变异 7（walk.rs）下 step_seven 探针：5 个镜像全变成 Holds（改前前 3 个是 NotApplicable）
PROBE_MAPPING_KEY case=nothing_persisted mapping_key=Some(Holds)
PROBE_MAPPING_KEY case=everything_before_the_first_unit_write mapping_key=Some(Holds)
PROBE_MAPPING_KEY case=units_and_records_of_txg_three_without_its_root mapping_key=Some(Holds)
PROBE_MAPPING_KEY case=txg_three_root_persisted_rotation_not mapping_key=Some(Holds)
PROBE_MAPPING_KEY case=every_write_persisted mapping_key=Some(Holds)
# 变异 1 类（=> 2）下：三态函数退回两态
PROBE formatted_pool_fast … two_state_selected=22 three_state_selected=22
PROBE acquisition_barrier … two_state_selected=262147 three_state_selected=262147
PROBE parallel_line_one_fast … two_state_selected=102 three_state_selected=102
PROBE spill_over … two_state_selected=20 three_state_selected=20
PROBE_SPILL root_segment=2 root_persisted_two_state=4 root_persisted_three_state=4 record_writes=[[148, 149], [150, 151]]
PROBE CentralMappingRootSplit … two_state_selected=8 three_state_selected=8（其余六条同）
# 变异 3（none）下：函数退回两态
PROBE ExtentInlineToLowerSegment … two_state_selected=20 three_state_selected=20
PROBE VersionWithoutFileRowPublishWithItsOwnTree … two_state_selected=18 three_state_selected=18
```
（`…` 是我删掉的 `writes= sizes= tearable_total=` 三段，原行在 `logs/mutation-crash-two-states-*.log`、`logs/mutation-crash-count-function-none-*.log`。）

旁证能说明的只到这一步：变异改动了钉值那一侧的量，或者改动了三态函数那一侧的量。说明不了「层 0 用例真的红在那一条」，那一步要 59 号证。变异 3 那一侧要枚举照旧取三态才红：`crash.rs:2800` 的 `let tearable = TearableInPlaceOverwrites::of(base, writes);` 不在变异范围里，这是读代码推的。

## 七、交回前的验证（第 4 步那几样，末尾原样）

开跑前 `ps` 看到的：别的会话在跑几条 `cargo test`（`second_transaction_supplement_three_crash_injection`、`probe_unit_area_wall_seeds`、`second_transaction_supplement_three_random_history`，后来又有 `--lib` 与几个非层 0 目标），没有 qemu、vm-bench、e152、fio。我的命令都加了 `nice -n 19`，线程上限走 `capped.sh`：开头是 6，收到主 agent 消息之后改成 5。跑编出来的代码都经 `run-with-memory-cap.sh 8G`，一次都没撞到包装的 250–254。等锁没单独计时。

- **动到的测试二进制**：7 个都是名字带 layer0 的，按定义一个都不跑。它们的比较与钉值交提交时的层 0 验证（第九节）。
- `cargo build --offline --all-targets`：
  - 主树退出 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s`。这 7 个层 0 二进制在主树 target 里的时间戳是 23:01:55–56Z，晚于我最后一次改文件的 23:00:33Z。
  - 副本（开工快照 + 我这 7 份 + 探针）退出 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.88s`，警告与错误 0 条（`grep -c '^warning\|^error'` 得 0）。
- `cargo fmt --all -- --check`（主树）：退出 1。`Diff in` 点名的是 `singlefs-core/src/allocator.rs`（A2c 的）、`singlefs-harness/src/bin/e158_root_choice_repair.rs`、`tests/core_review_unit_area_start_and_publish_limits.rs`（A4b 的），都不是我的。我这 7 份逐个 `rustfmt --edition 2021 --check`，退出 0。
- clippy（check.sh 那套：`-D warnings` 加 7 条）：
  - 整个工作区 `--all-targets --all-features --keep-going`：退出 101，卡在 `error: manual implementation of `.is_multiple_of()`` `--> crates/singlefs-core/src/allocator.rs:216:12`（A2c 在改的文件），core 编不过，harness 没查到。
  - 只查 harness（`-p singlefs-harness --no-deps --all-targets --all-features --keep-going`）：退出 101，`could not compile` 的是 `core_review_unit_area_start_and_publish_limits`、`checker_narrow_invariants_and_abandoned_roots`、`e156_allocation_basis_counts`、`e158_root_choice_repair`，都不是我的。日志里我这 7 个目标的名字出现 0 次。
  - 只查我这 7 个目标（同一套 lint，`--no-deps` 加 7 个 `--test`）：退出 0，末行 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.80s`。
- 登记给我的门禁阶段（主树），退出码与判定行原样：
  - 33 号：退出 1，`  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：`，点名第 430、436 行（`singlefs-core/src/allocator.rs`）与第 974、975、977、978 行（`singlefs-core/src/admission.rs`），都是别人在改的文件，我没碰主表。
  - 53 号：退出 0，`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
  - 92 号：退出 0，`  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进`
  - 94 号：退出 0，`  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 283 行里没有分支与循环（`#[cfg(test)]` 标着的项 263 行不扫）`
  - 93 号：退出 0，`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
  - 89 号：退出 77，`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`。本次未跑，不是通过。
  - 74 号（外面套了 `capped.sh 5`，阶段里面自己经内存包装）：退出 1，`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 48.79s`。红的是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，与 B3a-2、B3a-3 报的是同两条。那是随机历史的测试二进制，我只改了 7 个层 0 测试文件，它们是另外的测试二进制，编不进它，所以不是这一轮带出来的。

## 八、交主 agent 的问题与发现

**F1：4 份层 0 文件在今天的代码上先红在 `prepare`，与按设备切段 / 第三态无关**。依据是第五节的 `PROBE_MISMATCH`。`research/prompts/m2-layer0-scale-r1-opus-output.md` 第 209 行起那一段（第 215 行）早就推过「冻结副本上名字带 `layer0` 的这几份用例在 `prepare` 的段形状断言上就会红」；`step_seven` 后来跟上了（18 → 26），这几份没跟。根因看起来是每次发布写的单元多了：`rewritten` 清单开头是 `AllocationTreeNodeBelowTheRoot` 盘 0、盘 1 各一片叶 61，第一个文件版本那次发布的单元写从 16 变成 24。这是读探针输出推的，没去追是哪一批改的。

这 4 份都要做设计判断，我没动 `prepare`：
- `formatted_pool`：机械的一份。段 18 → 26、写数 33 → 41、「十六个单元写」「文件版本 21」这些文字，外加 `the_formatted_pool_mount_…_segment_sequence` 的两态闭式 262165 → 67108885。与 `step_seven` 已经改过的那几处同形。快档的 37 不受影响。
- `parallel_line_one`：文件头的前提是「一次发布的单元数不超过 10」，今天 B 是 14 个单元、C 是 15 个，单元段 30 写、32 写。快档展开的小段没变，117 不受影响；全量要 12230590578 个状态，今天的形状上跑不动。要定：重新挑内容大小让单元数回到 ≤ 10（可能做不到，固定开销就不止 10），还是改前提、全量另想办法。
- `spill_over`：`leaf_containers = JOURNAL_NAMED_ENTRIES_PER_RECORD - 5 + 1` 写死了 5 个非叶容器角色，今天是 11 个（74 = 63 + 11），第二条记录装 7 项、不是 1 项。改成 67 − 11 + 1 = 57 片叶容器就回到「68 项、末条 1 项」。记录段照旧 4 写，快档 25、根槽已落 9 不受影响。角色数是现数还是写死，要定。
- `tree_split`：7 条流被录那一次之前的树都不是用例钉的样子（映射树第一个文件版本就有 10 条，不是 6 条），节点容量的布置都要重算，才打得中「根分裂 / 叶分裂 / 两层连着分裂 / 摘空 / 降高」。快档 13 不受影响；全量里 `CentralMappingTwoLevelsSplitInARow` 今天是 2^42 量级。

我改的全量钉值（`parallel_line_one` 的 12386418、`tree_split` 按 u 的算式）是按文件里写着的形状算的，与它们的两态钉值同一个口径。重新布置流时要跟着一起重算。

**F2：`position_addressed` 的快用例在今天的代码上必红**。`enumerate()` 的 `println!` 算 `closed_form_state_count(&prepared.segments)`，第三条流的段是 `[322, 290, 1, 2]`。`1u64 << 322` 在 debug 与 release 下都 panic（`Cargo.toml` 第 17 行 release 也是 `overflow-checks = true`）。探针打的 `PROBE_LONGEST stream=ExtentLowerSegmentGrowsToTwoLevels longest_segment=322`。
- 红在我改的那条比较之后：前两条流过得去，第三条流先过比较，再在 `println!` 上 panic。
- 我没改：它不在规格的改法里，打印哪个数要定。可选的改法：只对 `ENUMERABLE_IN_FULL` 打全量闭式；或者改打三态函数在 `full_expansion` 下的数，这个也溢出，同样要限；或者这一格不打。

**F3：主表第 23、289、474 行点名的用例在 `step_seven` 里已经改名**。替换行在 `mutations-replacements.tsv`（第六节）。不是这一轮改的名，是这一轮按规格「找点名它的同形行」找出来的。

**F4：`research/scripts/capped.sh` 有一段时间是坏的**。22:52:25Z 被别的会话改过：第 22 行语法错，preflight 两行插进了 `variable_names` 数组。我第二趟探针 7 条都退 2，没跑；之后改用副本里与 HEAD 逐字相同的那份跑了第三趟和变异旁证。约 23:03Z 我复查时主树那份已经好了，之后的 build、clippy、74 号都经主树那份跑。主 agent 中途的消息说的就是这件事，在第三节写完之后收到，没改我做的任何东西。

**F5：没有停在「条款没写」的分支上**。这一轮只改测试里的比较与钉值，没加错误成员、`todo!`、`assert!`。

**F6：`step_seven` 里用的是常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，不是字面的 `"I-MAPPING-KEY"`**。主 agent 写回 kb、定了编号之后，改 `image.rs:38` 那个常量的值，这份用例跟着变，不用再改。

## 九、受影响的层 0 流与崩溃枚举用例

checker（`crates/singlefs-checker/src/`）没动，按定义这一节可以不写。列出来是给集成时排快档用：
- `crash-case:layer0-first-stream`（`step_seven` 全量）：`assert_checker_counts` 里 `I-MAPPING-KEY` 的期望从全部状态（150994980）改成 `ROOT_PERSISTED_STATES`（9）。这条用例能不能绿取决于这次归类，提交时要跑。
- `step_seven` 快档 `layer0_quick_tier_matches_the_full_tally_shape`：同一处，期望从 54 改成 9。
- 6 份 `second_transaction_*_layer0` 的快用例：状态数改成三态口径。其中 4 份先红在 `prepare`（F1），`position_addressed` 先红在 `println!`（F2），能直接判出这一轮改动对不对的只有 `acquisition_barrier`（262152）。这几份不在 54 号的两个二进制里，按 `m2-layer0-scale-r1-opus-output.md` 第 205 行，它们只在全量 `cargo test` 里跑。
- 带 `#[ignore]` 的全量：`parallel_line_one`、`position_addressed`、`tree_split` 各一条，钉值跟着改了，没有哪道门禁跑它们（同一份报告第 205 行）。
- 没有新加层 0 流，也没有新的崩溃枚举用例。

## 十、`git diff --stat -- crates litmus`（主树，原样；工作区里同时有别的会话与此前各批没提交的改动，分不出谁的，我写过的以第二节为准；全份在 `logs/git-diff-stat.txt`）

含 layer0 的几行与末行：
```text
 .../tests/first_transaction_step_seven_layer0.rs   |  261 +-
 .../second_transaction_parallel_line_one_layer0.rs |   64 +-
 ...action_parallel_line_three_spill_over_layer0.rs |   36 +-
 ..._transaction_position_addressed_trees_layer0.rs |   30 +-
 ...action_step_three_acquisition_barrier_layer0.rs |   55 +-
 ...transaction_step_three_formatted_pool_layer0.rs |   36 +-
 .../tests/second_transaction_step_zero_layer0.rs   |  663 +-
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
 ...transaction_supplement_two_tree_split_layer0.rs |   40 +-
 93 files changed, 30124 insertions(+), 11251 deletions(-)
```
这几行里 `first_transaction_step_seven_layer0.rs` 的 261 行、`acquisition_barrier` 的 55 行，大半是此前各批（B3a-2、A1）没提交的改动，我的只有第二节那 +24/−4、+27/−5。`step_zero_layer0` 与 `rollback_witness_layer0` 不是我的。

## 十一、草稿目录删了什么、留了什么

- **删了**：仓副本 `/tmp/claude-1000/impl-rev-b3a3b/copy`。删之前 `du -sh` 整份 14G，其中 `copy/target` 14G。它是我在 22:44:20Z 用 rsync 建的，自己的 target 也在里面。
- **留着**，都在 `/tmp/claude-1000/impl-rev-b3a3b/` 下，共 1.2M，没有仓副本与编译目录：
  - 报告 `report.md`；
  - 变异行 `mutations-append.tsv`、`mutations-replacements.tsv`，主 agent 用 `apply-writer-patch.py` 写回；
  - 开工快照 `originals/`（7 份）；
  - 探针源码 `draft/probe/`（生成器、共用件、生成出的 7 份），驱动脚本 `draft/run-probes.sh`、`draft/mutation-effect-probes.sh`，变异旁证用的副本原件 `draft/copy-originals/`；
  - 日志 `logs/`：探针三趟、变异旁证、build、clippy、fmt、各门禁阶段、sha、diff；
  - 进度 `progress.md`。
- 这些都不入库。探针只是用来算钉值、核归类的一次性工具，依据已经写进这份报告；要复算，照第五节在新副本里重建。

## 十二、没做什么

- 没跑任何名字带 layer0 的测试二进制（规格与定义都不许），包括 `--no-run`。这 7 份改后的比较与钉值都没在枚举上核过，交提交时的层 0 验证与 59 号。
- 变异 10 行一行都没证红（都点 layer0 目标），全部留给 59 号；第六节的探针旁证不算证红。
- 没修 F1（4 份的 `prepare`）、F2（`position_addressed` 的 `println!` 溢出）：要做设计判断，也不在规格的改法里。
- 没改 `crates/mutations.tsv` 主表，也没改 kb；`I-MAPPING-KEY` 的编号写回不归我。
- 没跑三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
