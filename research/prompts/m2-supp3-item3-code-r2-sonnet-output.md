# 云端正推（Sonnet）：K2、K6

轮名 `m2-supp3-item3-code-r2`。判的是工作区里今天的 `crates/singlefs-harness/src/crash_injection.rs`（1415 行）与
`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`（603 行）。
凡引代码行号，均为本轮 `grep -n` 现查所得，不从背景材料或 diff 里数。
凡引产物读数，均为本轮在工作区当前代码上现跑所得，命令与原样输出照贴。

## K2：起点段放开之后，那 627 次写真的被抽到了吗

### 逐条核

**问题 1：`operation_kind` 何时为 `None`。**

`crates/singlefs-harness/src/crash_injection.rs:848-850`：

```rust
let operation_kind = match step {
    StepPosition::StartingPoint => None,
    StepPosition::Operation(step_index) => step_kinds.get(step_index).copied(),
};
```

`operation_kind == None` 当且仅当 `step == StepPosition::StartingPoint`。这条等价关系是代码本身的写法给出的，不是我的转述。

**问题 2：哪些段进了候选域。**

`crash_injection.rs:792-798`（`draw_crash_points` 里 `candidate_segments` 的定义）：

```rust
let candidate_segments: Vec<usize> = (0..segments.len())
    .filter(|segment_index| {
        segments[*segment_index].iter().all(|write| {
            (marks.after_make_filesystem..marks.after_the_last_finished_step)
                .contains(&stream_indexes[*write])
        })
    })
    .collect();
```

候选区间是 `after_make_filesystem..after_the_last_finished_step`，起点从 mkfs 之后就算数，不再从
`after_the_starting_point` 起（那个字段本轮已删）。diff 里能看到这处替换：
`research/prompts/_m2-supp3-item3-code-r2-diff.md:729-735`（`-` 行是 `marks.after_the_starting_point`
起算，`+` 行是 `marks.after_make_filesystem` 起算）。

`crates/mutations.tsv:184` 钉着这处改动本身：把 `after_make_filesystem..after_the_last_finished_step`
改回 `marks.after_each_step.first().map_or(0, |(_, after)| *after)..after_the_last_finished_step`
（也就是退回「从第一步跑完之后起算」的旧读法），点名必须红的测试是
`crash_points_fall_inside_the_starting_point_too`。

**问题 3：报告里有没有一条断言钉住 `operation_kind == None` 的崩溃点数 > 0。**

有，而且是两条，分别在两个文件里：

1. 外部测试文件 `second_transaction_supplement_three_crash_injection.rs:197-200`（在
   `assert_every_crash_injection_path_was_exercised` 里，被 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`
   这条快档用例调用）：
   ```rust
   assert!(
       tally.crash_points_inside_the_starting_point >= 1,
       "起点那一段里一个崩溃状态都没摆到（2026-09-20 才放开）；{replay}"
   );
   ```
2. `crash_injection.rs` 自己的单元测试模块里，`crash_injection.rs:1377-1380`：
   ```rust
   assert!(
       tally.crash_points_inside_the_starting_point >= 1,
       "起点那一段里一个崩溃状态都没摆到（起点段 2026-09-20 才放开）：{tally:?}"
   );
   ```
   紧跟着还有一条写死历史的单元测试 `crash_points_fall_inside_the_starting_point_too`
   （`crash_injection.rs:1385-1414`），其文档注释原话（`crash_injection.rs:1383-1384`）：
   「起点那一段也在候选里：从第一个文件起的历史，崩溃状态落得到起点那一步（代码三方
   m2-supp3-item3-code-r1 判决 K1-d：快档 7219 次写里 627 次此前永远抽不到）。」——这条测试点名引用了
   round 1 的 K1-d 编号，直接对应本轮 K2 要核的那个修补。

`tally.crash_points_inside_the_starting_point` 的定义与递增点：`crash_injection.rs:315`（字段定义，
注释「崩溃状态落在起点那一段里（起点段 2026-09-20 才放开，这个数证明真的摆到了）」）、
`crash_injection.rs:651`（`StepPosition::StartingPoint => tally.crash_points_inside_the_starting_point += 1`）。
因为 `operation_kind == None ⟺ step == StartingPoint`（问题 1），这个计数器就是
`operation_kind == None` 的崩溃点数，两者是同一个量，不是两个凑巧相等的量。

### 实测：快档一轮里 `operation_kind == None` 的崩溃点有几个

命令（本轮现跑，工作区当前代码，24 线程）：

```
cd crates && cargo test -p singlefs-harness --test second_transaction_supplement_three_crash_injection \
  crash_injection_fast_tier_recovers_only_into_versions_the_model_committed -- --nocapture
```

原样输出（节选，完整原始输出已核对，此处贴与本问题直接相关的两行 + 结果行）：

```
崩溃状态 93 个：截在发布中间（根还没落盘）的 75 个、截回到最后一版之前的 88 个、段内有洞（后发的写先持久）的 40 个、落在起点那一段里的 3 个
  落在 PublishFirstFile 里：1 个
  落在 PublishOverwrite 里：21 个
  落在 PublishWithoutUnits 里：2 个
  落在 CloseAndMountWritable 里：53 个
  落在 CloseAndMountRollback 里：9 个
  落在 RaiseRollbackFloor 里：4 个
  落在 ColdStartRecover 里：0 个
test crash_injection_fast_tier_recovers_only_into_versions_the_model_committed ... ok
```

「落在起点那一段里的 3 个」就是 `tally.crash_points_inside_the_starting_point`，也就是
`operation_kind == None` 的崩溃点数：这一轮跑出来是 **3**。核对内部一致性：
`crash_points_by_operation_kind` 七类之和 = 1+21+2+53+9+4+0 = 90，总崩溃点数 93，
93 − 90 = 3，与「落在起点那一段里的 3 个」逐字对上（`operation_kind` 为 `None` 的崩溃点不进
`crash_points_by_operation_kind` 这张按 `HistoryOperationKind` 分类的表，`crash_injection.rs:654-659`
的 `if let Some(kind) = crash_point.operation_kind` 只在 `Some` 时才计入）。
这条用例本身判 `ok`，即 `assert_every_crash_injection_path_was_exercised` 里
`tally.crash_points_inside_the_starting_point >= 1` 这条断言在本次跑上为真（3 ≥ 1）。

### 变异复核：断言是不是摆设

`crates/mutations.tsv:184` 那条变异，本轮手动施加复核（未走 `mutate.sh`/`replay.sh`，直接用
Python 定点替换施加同一处代码改动，跑完立即用同一份替换反向复原，`git status` 确认复原后与
提交前逐字节相同，工作区只留下这份报告与草稿目录里的备份）：

把 `crash_injection.rs:795` 的
`(marks.after_make_filesystem..marks.after_the_last_finished_step)` 改回
`(marks.after_each_step.first().map_or(0, |(_, after)| *after)..marks.after_the_last_finished_step)`
（即撤回本轮对起点段候选域的放开），跑：

```
cd crates && cargo test -p singlefs-harness --lib -- crash_injection::tests::crash_points_fall_inside_the_starting_point
```

原样输出（施加变异之后）：

```
test crash_injection::tests::crash_points_fall_inside_the_starting_point_too ... FAILED
thread '...' panicked at crates/singlefs-harness/src/crash_injection.rs:1406:9:
起点那一段里一个崩溃状态都没摆到：[Operation(1), ... Operation(9)]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out
```

复原之后重跑同一条命令：

```
test crash_injection::tests::crash_points_fall_inside_the_starting_point_too ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out
```

`git status --porcelain crates/singlefs-harness/src/crash_injection.rs` 复原后只显示
`A  crates/singlefs-harness/src/crash_injection.rs`（与本轮开工时一致，无残留改动）。

### K2 结论

这项修补（起点段进候选域）与「没修」在报告上**分得出来**：
`assert_every_crash_injection_path_was_exercised` 里 `crash_points_inside_the_starting_point >= 1`
这条断言直接钉住 `operation_kind == None` 的崩溃点数 > 0，快档一轮实测该值为 3；
撤回这处放开之后，同一条断言所在的独立单元测试 `crash_points_fall_inside_the_starting_point_too`
立即判红（本轮亲测，非转述 `mutations.tsv` 的声明）。`crates/mutations.tsv:184` 那一行不是摆设。

**什么现象会推翻这条结论**：若某一天 `crash_points_inside_the_starting_point >= 1` 这条断言被删除、
或改成存在性弱于「> 0」的形式（比如只判 `Some`/`None` 分支被编译到），或者
`crash_points_fall_inside_the_starting_point_too` 被标记 `#[ignore]`／从门禁跑的范围移出，
K2 的结论就不再成立——报告将退回round 1 的状态：起点段候选域打开与没打开在报告上无法分辨。

## K6：那几条断言够不够

### 逐条清点 `assert_every_crash_injection_path_was_exercised` 查的是什么

函数体在 `second_transaction_supplement_three_crash_injection.rs:140-236`。逐条列出（行号为
`assert!`/`assert_eq!`/`for` 关键行，本轮 `grep -n` 现查）：

| 行 | 断言 | 形态 |
|---|---|---|
| 143-147 | `tally.crash_points >= FAST_TIER_SEEDS`（即 `>= 24`） | **聚合下界**（不是逐段存在性，是全体崩溃点总数的下界） |
| 148-151 | `tally.model_judgements == tally.crash_points` | 等式（内部一致性交叉核） |
| 152-155 | `tally.checker_runs == tally.crash_points` | 等式（内部一致性交叉核） |
| 156-159 | `tally.record_checks == tally.crash_points` | 等式（内部一致性交叉核） |
| 160-177 | 对 `[UnitWrite, JournalRecord, RootRecordFua, SystemConfigurationSlot]` 四种写种类各 `>= 1` | 存在性（4 项，写种类全枚举） |
| 178-192 | 对 `[PublishOverwrite, CloseAndMountWritable]` 两种操作各 `>= 1` | 存在性（**只 2 项，`HistoryOperationKind` 共 7 种**） |
| 193-196 | `crash_points_withholding_a_write_before_a_persisted_one >= 1` | 存在性 |
| 197-200 | `crash_points_inside_the_starting_point >= 1` | 存在性（K2 那条） |
| 201-204 | `crash_points_inside_an_unfinished_publish >= 1` | 存在性 |
| 205-208 | `crash_points_that_land_before_the_last_committed_version >= 1` | 存在性 |
| 209 | `recoveries_failed == 0` | **绝对值**（钉死为零，不是下界） |
| 210-213 | `recoveries_reading_a_file >= 1` | 存在性 |
| 214-217 | `recoveries_without_a_file >= 1` | 存在性 |
| 218-221 | `recoveries_that_applied_journal_records >= 1` | 存在性 |
| 222-225 | `model_contents_compared >= 1` | 存在性 |
| 226-229 | `model_versions_without_a_file_matched >= 1` | 存在性 |
| 230-235 | 对 `["I-3.1", "I-5.4"]` 两条不变量各 `>= 1` | 存在性（**只 2 项，checker 实测判过的不变量本轮跑出 28 条**，见下） |

统计：**16 条判据里，13 条是纯存在性断言（`>= 1`）；1 条是聚合下界（第 143-147 行，形式与
round 1 K1-g 打中的 `crash_points >= 24` 完全相同，见下）；3 条是等式（含 1 条钉零）。
没有一条是「每一段/每一类都必须 ≥ 某个数」的逐段判据。**

（另有 `report.new_findings.is_empty()` 在调用方 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`
自己那条测试里，`second_transaction_supplement_three_crash_injection.rs:132-136`，不在
`assert_every_crash_injection_path_was_exercised` 函数体内，此处不计入上表但一并说明存在。）

### 第 143-147 行与 round 1 K1-g 打中的那条是不是同一种断言

背景材料 `research/prompts/_m2-supp3-item3-code-r2-background.md:37`（K6 的问题原文，本轮
`grep -n` 现查命中）：「第一轮 K1-g 打中「全是存在性断言，`crash_points >= 24` 允许 23 段全 0
（实际就有 1 段 0 个）」。」

今天 `second_transaction_supplement_three_crash_injection.rs:143-147`：

```rust
assert!(
    tally.crash_points >= FAST_TIER_SEEDS,
    "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
    tally.crash_points
);
```

`FAST_TIER_SEEDS: u64 = 24`（`second_transaction_supplement_three_crash_injection.rs:25`）。
判据字面就是「总数 ≥ 24」，与 round 1 K1-g 引的 `crash_points >= 24` 是**同一个常量、同一种聚合形式**：
它只约束 24 段历史里崩溃点数的**总和**，不约束逐段分布，因此数学上仍然允许「23 段各 0、1 段扛下全部
24+ 个」这种极端分布通过。断言自己的失败信息「平均每段连一个崩溃状态都没摆出来」也印证了它检验的是
均值，不是逐段下界。

`histories_without_any_crash_point`（`crash_injection.rs:308`，注释「一个崩溃状态都摆不出的历史段数」）
在整份 `second_transaction_supplement_three_crash_injection.rs` 里被 grep 零命中（本轮现查：
`grep -n histories_without_any_crash_point crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`
无输出）——**没有任何一条断言在读这个字段**，K1-g 打中的那个洞今天原样还在。

### 实测：七类操作里今天还有几类是 0；一段 0 个崩溃点今天还可不可能

命令一（大档，SEEDS=500，与快档同规模每段 24 步、每段抽 4 个崩溃点，本轮现跑）：

```
cd crates && SINGLEFS_CRASH_INJECTION_SEEDS=500 SINGLEFS_CRASH_INJECTION_OPERATIONS=24 \
  SINGLEFS_CRASH_INJECTION_POINTS=4 cargo test -p singlefs-harness \
  --test second_transaction_supplement_three_crash_injection \
  crash_injection_large_tier_from_the_environment -- --ignored --nocapture
```

原样输出（节选，与本问题直接相关的行）：

```
历史 500 段：跑完 500、以已知红收尾 {}、新发现 0；一个崩溃状态都摆不出的 4 段；模型目录最多 41 版
崩溃状态 1926 个：截在发布中间（根还没落盘）的 1490 个、截回到最后一版之前的 1843 个、段内有洞（后发的写先持久）的 769 个、落在起点那一段里的 123 个
  落在 PublishFirstFile 里：17 个
  落在 PublishOverwrite 里：423 个
  落在 PublishWithoutUnits 里：59 个
  落在 CloseAndMountWritable 里：985 个
  落在 CloseAndMountRollback 里：223 个
  落在 RaiseRollbackFloor 里：96 个
  落在 ColdStartRecover 里：0 个
test crash_injection_large_tier_from_the_environment ... ok
```

**七类操作里今天实测有 1 类是 0**：`ColdStartRecover`（快档 24 段与这次 500 段两次独立跑都是 0，
七类里其余六类在这两次跑上均 ≥ 1）。

**一段 0 个崩溃点今天仍然可能，而且这次跑上真的发生了**：「一个崩溃状态都摆不出的 4 段」，
4/500 = 0.8%，与 round 1 K1-g 「实际就有 1 段 0 个」是同一种失败模式的复现，只是样本更大、
计数从 1 变成 4。快档那 24 段这次跑上是 0（`crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`
的输出「一个崩溃状态都摆不出的 0 段」），但这只是 24 段的小样本里没摊上；聚合断言
`crash_points >= 24` 完全不排除它，500 段的样本已经实测复现。

### `ColdStartRecover` 的 0 是结构性的，不是抽样运气

`crash_point.operation_kind` 的取值来自「哪一步含有被扣下的那次写」（`crash_injection.rs:845-847`
`marks.step_containing(stream_indexes[segments[segment_index][first_withheld]])`）——只有**写**才能
被归到某个操作步下。`ColdStartRecover` 这一步本身在实现里不写盘：

`crates/singlefs-harness/src/history.rs:2615-2639`（`apply_cold_start_recover`）：调用
`recover(&pool.devices, JournalPolicy::Consult)`（`history.rs:2619`），而
`crates/singlefs-core/src/recovery.rs:1355`：

```rust
pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {
```

`reader` 的类型是 `&dyn PoolReader`（只读接口），它内部调用的 `replay_journal`
（`crates/singlefs-core/src/recovery.rs:840`）签名同样是 `reader: &dyn PoolReader`——
整条 `recover` 调用链没有写设备的能力。因此 `ColdStartRecover` 这一步在录制流里**不产生任何写**，
`step_containing` 也就永远不会把某次被扣下的写归到这一步：`crash_points_by_operation_kind` 里
`ColdStartRecover` 这一项**在这份实现下恒为 0**，两次独立跑（24 段、500 段）都是 0 与这条结构性
论证一致，不是巧合。

### K6 结论

`assert_every_crash_injection_path_was_exercised` 今天由 13 条存在性断言、1 条聚合下界、3 条等式
（其中 1 条钉零）组成。round 1 K1-g 打中的那条聚合下界（`crash_points >= 24`）字面未改：常量、
形式都与背景材料引的原句一致，而且这轮实测（500 段样本）复现了同一种失败模式——4 段一个崩溃状态
都没摆出来，`histories_without_any_crash_point` 这个字段在整份用例文件里零引用。
七类操作里，`ColdStartRecover` 结构性恒为 0（该操作不写盘），且断言表只覆盖了 7 类中的 2 类
（`PublishOverwrite`、`CloseAndMountWritable`），另外 5 类（含结构性为 0 的 `ColdStartRecover`）
一条断言都没有；checker 实测判过 28 条不变量（本轮快档跑出的 `I-1.1`…`I-9.14` 逐行都在渲染输出里），
断言只钉了其中 2 条（`I-3.1`、`I-5.4`）。**一段 0 个崩溃点在今天的代码上仍然可能，而且本轮的
500 段复跑已经实测到 4 段。**

**什么现象会推翻这条结论**：若 `assert_every_crash_injection_path_was_exercised` 加入一条读
`tally.histories_without_any_crash_point`（要求它等于 0，或给每段设下界）的断言，且该断言在
快档与大档上都实测通过，则「一段 0 个崩溃点仍然可能」这一半结论作废；若某天证明
`ColdStartRecover` 这一步在某种尚未覆盖的执行路径下确实会写盘（推翻「结构性恒零」的论证），
则「结构性」这个定性需要改成「实测恒零，机理未定」。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2：`operation_kind == None` 的崩溃点数（快档一轮） | 观测值 | 本轮快档实测 3（`crash_points_inside_the_starting_point`），与 `crash_points_by_operation_kind` 七类之和（90）加起来正好等于总数 93 |
| K2：有没有断言钉住它 > 0 | 有 | `second_transaction_supplement_three_crash_injection.rs:197-200` 与 `crash_injection.rs:1377-1380` 两处都断言 `>= 1`，且外层的单元测试 `crash_points_fall_inside_the_starting_point_too` 专门写死历史复核；本轮手动施加 `crates/mutations.tsv:184` 那条变异，该测试如约判红 |
| K2：这项修补今天能否与「没修」区分 | 能区分 | 断言存在 + 变异测试证明会红，两者都实测过，不是纸面声明 |
| K6：`assert_every_crash_injection_path_was_exercised` 查的是什么 | 13 存在性 + 1 聚合下界 + 3 等式（1 钉零） | 没有逐段/逐类下界，行号见文中表格 |
| K6：round 1 K1-g 打中的 `crash_points >= 24` 今天变了没有 | 没变 | 常量、形式与背景材料引文逐字一致，字段 `histories_without_any_crash_point` 在测试文件里零引用 |
| K6：七类操作里今天有几类实测是 0 | 1 类（`ColdStartRecover`） | 两次独立跑（24 段、500 段）都是 0，且已证明结构性恒零（该操作不写盘，`recover` 全链路只读） |
| K6：一段 0 个崩溃点今天还可不可能 | 可能，且已复现 | 500 段大档实测跑出 4 段 `histories_without_any_crash_point`，与 round 1 描述的失败模式相同 |

## 没做什么

- K1、K3、K4、K5 不归这条腿判，未作答（分给云端攻方 Opus 与本地攻方）。
- `crates/mutations.tsv` 里其余 36 条本轮新增变异（除第 184 行外）未逐条复跑，只复核了与 K2 直接相关的第 184 行。
- 没有跑 `.claude/gate.d/` 下任何阶段（`stage-owners.tsv` 未登记这条腿要跑的阶段；未按定义要求主动去核）。
- 没有跑大档默认规模（500 段 × 40 步 × 8 崩溃点，`crash_injection_large_tier_from_the_environment` 的默认参数）；
  本轮为对齐快档规模只跑了「每段 24 步、每段抽 4 个」的 500 段变体，用于复现 K6 的「一段 0 个」；
  是否在默认更大规模（40 步、8 点）下同样出现 0 段，没有验证，写在这里避免被读成「已核过默认大档」。
- `crates/mutations.tsv` 第 184 行以外的行是否也各自对应一条能独立复跑的单测，没有逐条验证（超出 K2/K6 射程）。
- 没有判断这两处发现（round 1 K1-g 的洞仍在、`ColdStartRecover` 结构性为 0 且断言未覆盖）该不该记进 `.claude/kb/checks-owed.md`，那是主 agent 的判决范围，不是这条腿的职责。
