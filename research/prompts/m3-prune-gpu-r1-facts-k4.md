# E161 K4-walk / K4-units 同键异判调查（2026-09-27）

只查不修。主仓一个字没改；所有改动在 `/tmp/claude-1000/investigate-e161-k4/` 的副本里（补丁留在 `probe-harness.patch`、`probe-walk.patch`）。

## 结论

1. 这 12 个不一致都出在 journal 环：同键的两边，单元区里读到的内容与所选根完全相同，差别只在 journal 环里多持久了一两条记录（`JournalRecord` 写）。池级 checker 有四条判定读这些记录，K4 的键却只收单元区里的读（装置 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:1846` `.filter(|read| read.offset >= UNIT_AREA_START_BYTES)`），环里的读（偏移 16777216 起）进不了键。
   - A 组（first_small 序号 9、10、11，K4-walk 与 K4-units 各算 3 个）：I-8.6、I-8.9 从「不适用」变「成立」。环里多了第一条自证过的记录（w2 / w3），读它的是 `scanned_journal_records_of_device`。
   - B 组（first_small 序号 150994968、150994969、150994970，只在 K4-units 里算 3 个）：I-3.10 从「不适用」变「成立」。多出的记录（w36 / w37）让 `tree_table_pointer_of_the_version_the_next_mount_applies_first` 从 none 变成 some，I-3.10 于是多读一棵分配记录树。
   - C 组（second_quick 序号 18、19、20，K4-units 3 个）：I-8.7 从「不适用」变「成立」。多出的记录是 w65 / w66。
2. 记录核对器不参与 K4 的判定：K4 两条臂的结局指纹只取 `check_pool_image` 的判定表（装置 `:2249`、`:2268`、`:2273`），`check_records`（`:2055`）不进这个指纹。
3. 最小补键：在 K4 键里加上「checker 正常那一遍在 journal 环里的读（位置 + 内容）」，四个格上的不一致都归 0。只加「journal 候选槽的答案」也能归 0，键数完全一样。补键后的不同键数：first_small 两条臂都从 5 / 4 涨到 13（状态 37 个）；second_quick K4-walk 从 7 涨到 16、K4-units 从 8 涨到 20（状态 84 个）。键数没有涨到接近状态数，但这只是这两个小格上量到的数。
4. 推翻条件：让 checker 看不见 journal 环以后，原键仍有不一致 ≥ 1；或者补了环读的键在这两格上仍有不一致 ≥ 1。这两条都在副本里量过，都是 0。另外在副本里故意造了一个环外的判定依赖，补了环读的键确实报出不一致（3 个、6 个），说明「归 0」不是计数失灵。

## 一、复现

### 1.1 现工作区复现不出：流的形状变了

先在现工作区的副本里跑（仓 HEAD `e5253e8a`，另带工作区里没提交的 crates 改动；装置文件与 HEAD 相同，`diff` 无输出）。形状行：

```
E7RESULT name=investigate_shape cell=first_small writes=41 segments=11 segment_lengths=[2,2,1,2,2,1,2,24,2,1,2] counts=[8,3,1,8,3,1,8,16777215,3,1,8] ordinals=16777260
E7RESULT name=investigate_shape cell=second_quick writes=477 segments=78 segment_lengths=[...] ... ordinals=48
```

产物里第一条流是 10 段（`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 第 16 行 `sampled_segment_states=[8, 3, 1, 8, 3, 1, 150994943, 3, 1, 8]`），第 3 行 `stream=first writes=41 segments=10`，第 4 行 `stream=second writes=477 segments=64`。现在的树把 26 写那段拆成了 `2,24`，所以 `small_domain_ordinals`（写数少于 26 的段全取）一下子变成 16777260 个状态。这次运行跑到 `progress cell=first_small slices=512/65537` 时被我停了（`proc.py stop`）。
在 `c3540c02`（第一个带 e161 装置的提交）上结果一样（11 段、16777260、48）。

### 1.2 在产物时刻之前最近的 crates 提交上复现出来了

产物文件的时间是 2026-09-27；在它之前最近一次动过 crates 的提交是 `9e56db41`。复现的做法：`git archive 9e56db41` 导出这棵树，再换进 `c3540c02` 的装置文件与 `tests/common/mod.rs`（装置第一次入库就是这一版），在装置里加一个只跑两格的模式 `investigate-k4`。这个模式调的仍是 feasibility 用的 `run_cell` + `report_cell`，外加逐状态探查。

```
E161_THREADS=4 nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 /tmp/claude-1000/investigate-e161-k4/target-9e56db41/release/e161_crash_state_dedup_and_time_split investigate-k4
```

跑出的形状：first_small 10 段、`ordinals=37`；second_quick 64 段、`ordinals=84`，都与产物一致。拿两格的 `reuse_arm`、`g1`、`determinism`、`g5_contents` 行（每格 9 行）与产物逐字比：

```
first_small identical 9 lines
second_quick identical 9 lines
```

其中与这次问题有关的三行，复现结果与产物逐字相同：

```
E7RESULT name=reuse_arm cell=first_small arm=k4_walk states=37 inconsistent=3 hits=32 distinct_keys=5 ...
E7RESULT name=reuse_arm cell=first_small arm=k4_units states=37 inconsistent=6 hits=33 distinct_keys=4 ...
E7RESULT name=reuse_arm cell=second_quick arm=k4_units states=84 inconsistent=3 hits=76 distinct_keys=8 ...
```

说明：
- 产物时刻的工作区没有留下快照，`9e56db41` 加 `c3540c02` 装置这套组合不能证明就是当时那一份代码。能说的只是：这两格的 18 行非计时产物逐字对得上。
- 这套模型是确定性的。run2、run3、run4 三次跑，84 行 `reuse_arm` / `investigate_key` / `investigate_group` 完全相同。这只能说明没有隐藏状态，不能当统计上稳定来用。
- 实验页「复跑」一节的命令写的是 `-p singlefs-harness`，装置现在已搬到 `singlefs-checker-tier`（只记下，没核那条命令现在能不能跑）。
## 二、机理：逐个状态

探查输出在 `run3.out` / `run3.err`（下面贴的行里，`ring_reads=[…]` 与 `candidate_journal=[…]` 两个字段已删掉，别的字段原样）。参照态指的是同键里序号最小的那个状态（`count_arm` 的定义）。

### 2.1 不一致的状态、段号、段内序号，以及和参照态差在哪条判定

```
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=9 segment=1 within=1 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=10 segment=1 within=2 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_walk key=e2f5ea4a ordinal=11 segment=2 within=0 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=9 segment=1 within=1 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=10 segment=1 within=2 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=ce431fc6 ordinal=11 segment=2 within=0 same_as_reference=false unit_elements=14 walk_elements=4 records=both_false verdicts_differing_from_reference=[I-8.6=Holds | I-8.9=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994968 segment=7 within=1 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994969 segment=7 within=2 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=first_small arm=k4_units key=3008d99a ordinal=150994970 segment=8 within=0 same_as_reference=false unit_elements=62 walk_elements=36 records=both_false verdicts_differing_from_reference=[I-3.10=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=18 segment=10 within=1 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=19 segment=10 within=2 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
cell=second_quick arm=k4_units key=fe1356dc ordinal=20 segment=11 within=0 same_as_reference=false unit_elements=114 walk_elements=84 records=both_false verdicts_differing_from_reference=[I-8.7=Holds]
```

参照态上这几条判定的原文：

```
cell=first_small arm=k4_walk ordinal=0:
  I-3.10=NotApplicable("候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）") 
  I-8.6=NotApplicable("环里没有一条自证过的记录找得到本实例内逻辑前一条：链判不了") 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=NotApplicable("环里一条自证过的记录都没有：没有哪一次发布的序号与末条标志可判") 
cell=first_small arm=k4_units ordinal=150994967:
  I-3.10=NotApplicable("候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）") 
  I-8.6=Holds 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=Holds 
cell=second_quick arm=k4_units ordinal=1:
  I-3.10=Holds 
  I-8.6=Holds 
  I-8.7=NotApplicable("环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来") 
  I-8.9=Holds 
```

### 2.2 和参照态比，持久集合差在哪几次写

```
cell=first_small arm=k4_units ordinal=8 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096]
cell=first_small arm=k4_units ordinal=9 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w2:+:JournalRecord:dev0:off16777216:len4096]
cell=first_small arm=k4_units ordinal=10 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w3:+:JournalRecord:dev1:off16777216:len4096]
cell=first_small arm=k4_units ordinal=11 vs_reference=0 writes=[w0:+:SystemConfigurationSlot:dev0:off0:len4096,w1:+:SystemConfigurationSlot:dev1:off0:len4096,w2:+:JournalRecord:dev0:off16777216:len4096,w3:+:JournalRecord:dev1:off16777216:len4096]
cell=first_small arm=k4_units ordinal=150994968 vs_reference=150994967 writes=[w36:+:JournalRecord:dev0:off16785408:len4096]
cell=first_small arm=k4_units ordinal=150994969 vs_reference=150994967 writes=[w37:+:JournalRecord:dev1:off16785408:len4096]
cell=first_small arm=k4_units ordinal=150994970 vs_reference=150994967 writes=[w36:+:JournalRecord:dev0:off16785408:len4096,w37:+:JournalRecord:dev1:off16785408:len4096]
cell=second_quick arm=k4_units ordinal=17 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096]
cell=second_quick arm=k4_units ordinal=18 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w65:+:JournalRecord:dev0:off16789504:len4096]
cell=second_quick arm=k4_units ordinal=19 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w66:+:JournalRecord:dev1:off16789504:len4096]
cell=second_quick arm=k4_units ordinal=20 vs_reference=1 writes=[w39:+:SystemConfigurationSlot:dev0:off4096:len4096,w40:+:SystemConfigurationSlot:dev1:off4096:len4096,w65:+:JournalRecord:dev0:off16789504:len4096,w66:+:JournalRecord:dev1:off16789504:len4096]
```

### 2.3 checker 里面：journal 环读出来的记录、下一次挂载先施加的那一版、分配记录树指针数

这一节靠副本里 `walk.rs` 加的打印（`probe-walk.patch`，开关只在探查那一次 `check_pool_image` 调用上打开），输出在 `run3.err`。journal_records 每项是 (jsn 计数器, 实例, txg, 提交标记)：

```
INVESTIGATE_STATE cell=first_small ordinal=0 || INVESTIGATE journal_records=0:[];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=8 || INVESTIGATE journal_records=0:[];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=9 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present")];1:[] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=10 || INVESTIGATE journal_records=0:[];1:[(1, 1, 1, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=11 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present")];1:[(1, 1, 1, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=150994967 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=0
INVESTIGATE_STATE cell=first_small ordinal=150994968 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
INVESTIGATE_STATE cell=first_small ordinal=150994969 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
INVESTIGATE_STATE cell=first_small ordinal=150994970 || INVESTIGATE journal_records=0:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")];1:[(1, 1, 1, "Present"), (2, 1, 2, "Present"), (3, 1, 3, "Present")] candidate_roots=3 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=1
```

second_quick 的几行（journal_records 字段太长，这里删了；计数器 4 的记录在 second_quick 里出现在序号 18–83 这 66 个状态上（`grep -c '(4, '` 数出 66）；同键 fe1356dc 那一组的成员是 1、3、5、7、9、11、13、15、17、18、19、20，其中只有 18、19、20 带它。原文在 `run3.err`）：

```
INVESTIGATE_STATE cell=second_quick ordinal=1 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=1
INVESTIGATE_STATE cell=second_quick ordinal=17 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=none allocation_node_pointers=1
INVESTIGATE_STATE cell=second_quick ordinal=18 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
INVESTIGATE_STATE cell=second_quick ordinal=19 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
INVESTIGATE_STATE cell=second_quick ordinal=20 || INVESTIGATE journal_records=… candidate_roots=4 versions_applied_only_by_records=[] next_mount_applies_first_tree_table=some(all_zero=false) allocation_node_pointers=2
```

### 2.4 机理，以及读了键外输入的那几行

三组的共同点：同键的两边，单元区里读到的「位置 + 内容」集合与所选根逐项相同（`unit_elements` 相同，键也相同），持久集合只差 `JournalRecord` 写。只差系统配置槽写的状态（first_small 1–8、second_quick 17）判定都与参照态相同，所以系统配置槽不是这次的原因。

- A 组，I-8.6 / I-8.9：参照态（序号 0、8）上，环里读不出一条自证过的记录，两条都是「不适用」。序号 9、10、11 在某一块盘的环槽 0 上持久了一条记录（w2 / w3，偏移 16777216），checker 读出它，这两条就判成了「成立」。
- B 组，I-3.10：参照态 150994967 上环里有计数器 1、2 两条记录，`next_mount_applies_first_tree_table=none`，分配记录树指针数 0，I-3.10「不适用」。150994968–70 在环槽 2 上多了 txg 3 的记录（w36 / w37，偏移 16785408），`tree_table_pointer_of_the_version_the_next_mount_applies_first` 于是交回 some，指针数变成 1，I-3.10 读到了未释放的记录，判成「成立」。这里走的不是 `versions_applied_only_by_records`：这几个状态上它都是 `[]`。
- C 组，I-8.7：同键那一组里，只有 18、19、20 在环槽 3 上多了计数器 4 的记录（w65 / w66，偏移 16789504）。这条记录让「同一实例的两条非 0 事务号」凑成了对，I-8.7 从「不适用」变成「成立」。I-3.10 在参照态上已经「成立」，所以不在差异里。

读了键外输入的行。K4 的键只收单元区里的读，下面这几行读的都是 journal 环，或者读的东西由 journal 环决定：

| 作用 | 复现树（9e56db41 的 `crates/singlefs-checker/src/walk.rs`） | 现工作区 `crates/singlefs-checker/src/walk.rs` |
|---|---|---|
| 扫 journal 候选槽（答案随环里持久了哪几条写变） | 4634 `fn scanned_journal_records_of_device(`，4642 `.candidate_journal_slots(device)` | 4848，4856 |
| 读环里整条 4096 字节的记录 | 4647 `let Some(bytes) = reader.read(device, offset, record_bytes) else {` | 4861 |
| check_pool_image 里取环记录、交给 I-8.x | 5529 `let journal_records_by_device =`，5531 `judge_journal_back_chain(...)`，5536 `judge_transaction_numbers_per_instance(...)` | 5769，5771，5776 |
| I-8.6 判定 | 4837 `judgements.judge("I-8.6", record.back_chain == expected_back_chain, \|\| {` | 5051 |
| I-8.7 判定 | 4924 `judgements.judge("I-8.7", record.transaction > previous_transaction, \|\| {` | 5138 |
| I-8.9 判定 | 5232 `judgements.judge("I-8.9", true, String::new);` | 5446 |
| 下一次挂载先施加的那一版：由环记录定 | 3335 `fn tree_table_pointer_of_the_version_the_next_mount_applies_first(` | 3549 |
| 把那一版的树表并进 I-3.10 要读的分配记录树 | 3391 `tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);`，调用点 5784 `let allocation_node_pointers = allocation_record_node_pointers_of_the_candidate_versions(` | 3605，调用点 6024 |
| I-3.10 判定 | 3460 `judgements.judge("I-3.10", record.generation == birth_txg, \|\| {` | 3674 |

装置这一边（现工作区的 `crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`，与 HEAD 相同）：
- 1846 `.filter(|read| read.offset >= UNIT_AREA_START_BYTES)`：K4-walk、K4-units 的集合只收单元区的读，环里的读（`RING_START_BYTES` ≤ 偏移 < `UNIT_AREA_START_BYTES`）与 journal 候选槽的答案都被丢掉。
- 2249 `let verdict_outcome = fingerprint_of_debug_text(&verdicts);`、2268 `set_key(root, &walk_elements),`、2273 `set_key(root, &unit_elements),`：结局只取池级 checker 的判定。2055 `let records = check_records(image, consulted.effective_root);` 的结果不进 K4 的结局，所以记录核对器里没有哪一行造成这 12 个不一致。

只看了扫描方向、持久集合、oracle 版本号三样，排除的根据如下（每样各凭一条观测）：
- 扫描方向（`candidate_unit_slots`）：把它的答案加进键，不一致数不变（first_small 3 / 6，second_quick 3），见 3.1。
- 持久集合：它确实能区分这些状态，但它是经 journal 环读传进 checker 的。只加环读就已经归 0，所以没必要再往键里放持久集合；放进去键数等于状态数（37 / 84）。
- oracle 版本号：K4 的结局里没有 oracle（装置 2249 只取 `verdicts`）。

### 2.5 为什么 K4-walk 是 3 个、K4-units 是 6 个

B 组在 K4-walk 里不同键：参照态的 `walk_elements=4`，150994968–70 是 36（2.1 那几行）。多出来的读，来自 checker 顺着「下一次挂载先施加的那一版」去读分配记录树与单元头，走树那一遍记下了它们。正常那一遍对每个候选槽本来就都读一遍（`unit_elements=62`，两边相同），集合去重之后看不出这几次读是判 I-3.10 读的，所以 K4-units 把 B 组并成了一个键。
C 组在 K4-walk 上是 0，也只是凑巧：计数器 4 的记录同时打开了「下一次挂载先施加的那一版」（指针数 1→2），走树的读集跟着变了。K4-walk 本身并没有收进 I-8.7 的输入。
旁证：让 checker 看不见环以后（第四节），first_small 上 K4-walk 的键数从 5 降到 4，B 组的三个又并回了参照态那个键。

## 三、最小补键（副本里改键，只跑 first_small 与 second_quick）

### 3.1 各种补法的不一致数与不同键数（`run3.out`，run2 / run4 逐字相同）

`added=` 是往 K4 集合里另外并进去的元素（取自 checker 正常那一遍的整份调用，也按「位置 + 答案」算）：`ring_reads` 是环里的读；`candidate_journal_slots` / `candidate_unit_slots` 是两种候选槽调用的答案；`fixed_structure` 是环起点之前的读，加上盘列表与盘大小；`all_normal_calls_as_set` 是整份调用当成集合；`persisted_set` 是把持久集合掩码与原键配对。

```
cell=first_small arm=k4_walk added=none states=37 inconsistent=3 distinct_keys=5
cell=first_small arm=k4_walk added=ring_reads states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=ring_reads_and_candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_walk added=candidate_unit_slots states=37 inconsistent=3 distinct_keys=5
cell=first_small arm=k4_walk added=fixed_structure states=37 inconsistent=3 distinct_keys=29
cell=first_small arm=k4_walk added=all_normal_calls_as_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_walk added=persisted_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_units added=none states=37 inconsistent=6 distinct_keys=4
cell=first_small arm=k4_units added=ring_reads states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=ring_reads_and_candidate_journal_slots states=37 inconsistent=0 distinct_keys=13
cell=first_small arm=k4_units added=candidate_unit_slots states=37 inconsistent=6 distinct_keys=4
cell=first_small arm=k4_units added=fixed_structure states=37 inconsistent=3 distinct_keys=29
cell=first_small arm=k4_units added=all_normal_calls_as_set states=37 inconsistent=0 distinct_keys=37
cell=first_small arm=k4_units added=persisted_set states=37 inconsistent=0 distinct_keys=37
cell=second_quick arm=k4_walk added=none states=84 inconsistent=0 distinct_keys=7
cell=second_quick arm=k4_walk added=ring_reads states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=candidate_journal_slots states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=ring_reads_and_candidate_journal_slots states=84 inconsistent=0 distinct_keys=16
cell=second_quick arm=k4_walk added=candidate_unit_slots states=84 inconsistent=0 distinct_keys=11
cell=second_quick arm=k4_walk added=fixed_structure states=84 inconsistent=0 distinct_keys=47
cell=second_quick arm=k4_walk added=all_normal_calls_as_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_walk added=persisted_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_units added=none states=84 inconsistent=3 distinct_keys=8
cell=second_quick arm=k4_units added=ring_reads states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=candidate_journal_slots states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=ring_reads_and_candidate_journal_slots states=84 inconsistent=0 distinct_keys=20
cell=second_quick arm=k4_units added=candidate_unit_slots states=84 inconsistent=3 distinct_keys=8
cell=second_quick arm=k4_units added=fixed_structure states=84 inconsistent=3 distinct_keys=72
cell=second_quick arm=k4_units added=all_normal_calls_as_set states=84 inconsistent=0 distinct_keys=84
cell=second_quick arm=k4_units added=persisted_set states=84 inconsistent=0 distinct_keys=84
```

读法：
- 往键里至少再加一样：checker 正常那一遍在 journal 环里的读（位置 + 内容）。加上之后，两格两臂的不一致都是 0。不同键数：first_small 5→13（K4-walk）、4→13（K4-units），状态 37 个；second_quick 7→16（K4-walk）、8→20（K4-units），状态 84 个。键数与状态数之比从 0.083–0.135 涨到 0.190–0.351（`python3` 算的），离「每个状态一个键」还远，但这只是两个小格上的量。26 写大段上会怎样没量，推不出来。
- 只加 `candidate_journal_slots` 的答案，不一致与键数和加环读完全一样。不过这个答案来自枚举器（`crates/singlefs-harness/src/memory_pool.rs` 的 `candidate_journal_slots` 只看哪几次环写持久了），不带内容。这两格里恰好一样，不能拿来说明一般情况。
- 只加固定结构那一类的读，K4-units 在两格上还剩 3 个，K4-walk 在 first_small 上还剩 3 个，键数却涨到 29 / 47 / 72。补固定结构补错了地方。
- 加单元候选槽，两格两臂的数一样都不变。
- 「加环读就够」是在这两格上量出来的，不是构造上保证的。构造上保证同键同判的，只有整份调用当成集合这一种，而它在这两格上键数等于状态数（37 / 84），也就是说没有复用可言。

## 四、推翻条件，以及在副本里造出来的那两次

这个定位的说法是：同键异判全部来自池级 checker 读 journal 环（2.4 的表），而 K4 的键丢掉了环里的读。下面任何一条出现，这个说法就是错的：

1. 让 checker 看不见 journal 环以后，原 K4 键在这两格上还有不一致 ≥ 1。
2. 往键里补上环里的读以后，这两格上还有不一致 ≥ 1。
3. 某个不一致的状态，与它的参照态比，持久集合里没有任何一次 `JournalRecord` 写不同。

量到的：
- 条件 2：见 3.1，四行 `added=ring_reads` 的 `inconsistent` 都是 0。
- 条件 3：见 2.2，9 个不一致状态各自差的写里都有 `JournalRecord`。
- 条件 1：在副本的 `scanned_journal_records_of_device` 开头加了一个开关：设了 `E161_INVESTIGATE_BLIND_JOURNAL` 就交回空表（`probe-walk.patch`）。打开它跑（`run5-blind.out`）：

```
E161_INVESTIGATE_BLIND_JOURNAL=1 E161_THREADS=4 nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 <同一个二进制> investigate-k4
```
```
E7RESULT name=reuse_arm cell=first_small arm=k4_walk states=37 inconsistent=0 hits=33 distinct_keys=4 keys_over_states=0.108108 keys_at_or_a
E7RESULT name=reuse_arm cell=first_small arm=k4_units states=37 inconsistent=0 hits=33 distinct_keys=4 keys_over_states=0.108108 keys_at_or_
E7RESULT name=reuse_arm cell=first_small arm=k4_full states=37 inconsistent=0 hits=0 distinct_keys=37 keys_over_states=1.000000 keys_at_or_a
E7RESULT name=reuse_arm cell=second_quick arm=k4_walk states=84 inconsistent=0 hits=80 distinct_keys=4 keys_over_states=0.047619 keys_at_or_
E7RESULT name=reuse_arm cell=second_quick arm=k4_units states=84 inconsistent=0 hits=76 distinct_keys=8 keys_over_states=0.095238 keys_at_or
E7RESULT name=reuse_arm cell=second_quick arm=k4_full states=84 inconsistent=0 hits=0 distinct_keys=84 keys_over_states=1.000000 keys_at_or_
```

打开开关以后，两份输出里 I-8.6 / I-8.7 / I-8.9 判成 Holds 的行数（`grep -c`）：

```
run5-blind.out:0
run4.out:11
```

打开开关以后，两格上 K4-walk、K4-units 用原键的不一致都是 0，环类判定一条都没有判成 Holds。说明除了环以外，没有别的键外输入在这两格上制造不一致。

「补了环读的键不一致为 0」这个计数会不会报红，要单独验：在探查里把判定指纹换成（原判定，第一个在本格里取值有变的系统配置槽写是否持久），也就是故意让结局依赖一样既不在单元区、也不在环里的输入，再用同一套计数去数（`run4.out`）：
```
name=investigate_injected cell=first_small arm=k4_walk added=none injected_outcome_input=persisted_w0 states=37 inconsistent=6 distinct_keys=5
name=investigate_injected cell=first_small arm=k4_walk added=ring_reads injected_outcome_input=persisted_w0 states=37 inconsistent=3 distinct_keys=13
name=investigate_injected cell=first_small arm=k4_units added=none injected_outcome_input=persisted_w0 states=37 inconsistent=9 distinct_keys=4
name=investigate_injected cell=first_small arm=k4_units added=ring_reads injected_outcome_input=persisted_w0 states=37 inconsistent=3 distinct_keys=13
name=investigate_injected cell=second_quick arm=k4_walk added=none injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=7
name=investigate_injected cell=second_quick arm=k4_walk added=ring_reads injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=16
name=investigate_injected cell=second_quick arm=k4_units added=none injected_outcome_input=persisted_w39 states=84 inconsistent=9 distinct_keys=8
name=investigate_injected cell=second_quick arm=k4_units added=ring_reads injected_outcome_input=persisted_w39 states=84 inconsistent=6 distinct_keys=20
```

补了环读的键在造出来的环外依赖上报出 3 / 3 / 6 / 6 个不一致，所以第三节里那几个 0 不是计数失灵造成的。

## 五、三次推导

- 正推：K4 的键只收单元区里的读（装置 1846）；checker 的 I-8.6 / I-8.7 / I-8.9 读环里的记录，I-3.10 的读法也由环里的记录决定（2.4 的表）。那么只差环写的两个状态可以同键，判定却不同。2.2 的持久集合差与 2.1 的判定差一一对上。
- 反推：如果这个说法是错的，应当看到下面三样中的一样：补了环读仍有不一致、看不见环仍有不一致、有不一致的一对不差环写。三样都量了，都没有出现（第四节）。
- 校验：换一条不经 K4 键计数的路，直接在 checker 内部打印环记录与分配记录树指针数（2.3，`walk.rs` 里的打印），再用持久集合的写差（2.2，从枚举计划算，不经读者）对一遍。两条路都指向同一批 `JournalRecord` 写。计数路子本身会不会报红，用 4 里造出来的依赖验过。

## 六、另外看到的（没展开）

- 在现工作区（`e5253e8a` 加没提交的 crates 改动）与 `c3540c02` 上，第一条流是 11 段 `[2,2,1,2,2,1,2,24,2,1,2]`，`small_domain_ordinals` 交回 16777260 个状态，second_quick 是 48 个状态。在现在的树上直接跑 feasibility，first_small 这一格已经不是 37 个状态的小格了。S2 那几条在现在的树上判什么，我没跑，所以没核。

## 七、没做什么

- 没修，也没判该怎么改键、该不该用复用臂。
- 没跑任何重型测试、门禁、层 0，也没跑 cargo test。只跑了 e161 装置二进制加进去的 `investigate-k4` 模式，范围是 first_small 与 second_quick 两格，外加一次只打印形状的运行。现工作区上那一次误跑了大域，跑到 first_small 的 512/65537 片时被我停掉。
- first_quick 与 segment_head 两格没跑。产物里 first_quick 上 K4-walk 3 个、K4-units 6 个，是不是同一机理，没验。
- 产物当时用的那份代码没有留快照，没法逐字确认就是 9e56db41 加 c3540c02 装置这一套。能确认的只是两格 18 行非计时产物逐字一致。
- 26 写大段上补了环读之后的键数没量，4.12 说的「键数接近状态数」这次没有碰到。
- 副本里的改动没回主工作区；副本与编译目录交回前删掉，补丁与输出留在草稿目录。
