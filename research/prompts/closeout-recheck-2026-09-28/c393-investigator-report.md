# C393 调查报告：被抛弃根的账读不出时，可写挂载的结局与复用

日期：2026-09-28。仓：HEAD e5253e8a 加工作区（`rsync -a --exclude target --exclude .git` 拷到草稿目录的仓副本；副本的 `crates/` 与主工作区逐文件相同，只多了一份新用例，`diff -rq` 为证，见第二节）。
只查不修；没判该怎么改。

## 一、结论

历史：A、B（实例 1）→ 重开取号 2 → C (2, 8) → 崩溃恢复抛弃 C（C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽坏掉，恢复落到 (2, 7)，实例 3 写行 txg 9、暖机 txg 10）→ C 写回 → **再重开可写挂载（实例 4）**，这一次让 C 的一份账两份都读不出（块设备错）。
被抛弃根只由崩溃恢复造出，用的是 `common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`（与 `unreadable_abandoned_root_slot.rs`、C554 用例同一段）。

问题 1 与 2 的结局一览（每格都有一条用例，第二节有原样输出）：

| 读不出的账 | 读不出的时长 | 可写挂载结局 | `abandoned_roots_unreadable` | 影子账隔离（盘 0 / 盘 1） | 产品的「重读一次」钩子 | 之后复用 C 的单元 | 池级 checker I-7.4 |
|---|---|---|---|---|---|---|---|
| （对照）都读得出 | — | 照常可写挂载 | 0 | 14 / 14 | 0 次 | 到 txg 31 一个都没复用 | Holds |
| 分配记录树（根节点，槽 50339） | 一直读不出 | 照常可写挂载，**没被乙-配置续拒** | 1 | **0 / 0** | 0 次 | **txg 14 就复用**（C 的数据单元，盘 0 槽 50184） | **红，只红 I-7.4** |
| 分配记录树 | 一直读不出，产品若重读则钩子里撤掉 | 同上 | 1 | 0 / 0 | **0 次** | txg 14 复用 | 红，只红 I-7.4 |
| 分配记录树 | 两份各只坏第一次读 | 照常可写挂载 | **0** | **14 / 14** | 0 次 | 到 txg 31 没复用 | Holds |
| 树表（槽 50342） | 一直读不出 | 照常可写挂载，没被拒 | 1 | 0 / 0 | 0 次 | txg 14 复用 | 红，只红 I-7.4 |
| 树表 | 一直读不出，产品若重读则钩子里撤掉 | 同上 | 1 | 0 / 0 | 0 次 | txg 14 复用 | 红，只红 I-7.4 |
| 树表 | 两份各只坏第一次读 | 照常可写挂载 | **1** | **0 / 0** | 0 次 | txg 14 复用 | 红，只红 I-7.4 |

读法：

1. **乙-配置续罩不到这一格。** 所有臂的重开挂载都报 `read_stage: OnTheFirstRead`（所选那一版 (3, 10) 就是系统配置见证到的那次，jsn 10）与 `instance_table_of_the_newest_root: OnTheFirstRead`：C (2, 8) 比所选那一版旧，判据 N-配置 为假；最新根 (3, 10) 的实例表读得出。乙的两处重读（读阶段、影子账读最新根的实例表）都不碰被抛弃根自己的账。钩子在每一臂都是 0 次；同一段历史上让 (3, 10) 的实例表读不出的阳性对照里钩子是 1 次，见第二节。
2. **一直读不出**：两种账结局相同。照常可写挂载，计数 1，C 独占的 14 槽一个都不隔离（重开之后在分配器里是空闲的，见 `free_units_of_c_after_remount`），重开之后第一次覆盖写（txg 14）就把 C 的数据单元写掉，池级 checker 只红 I-7.4。
3. **第一次读不出、重读读得出**，这件事在两种账上不一样，差别在于有没有第二次读：
   - 产品层的「重读一次」（C554 乙，R = 1）对这一读不起作用：钩子 0 次，结局与一直读不出相同（两种账都一样）。
   - 设备层只坏第一次读时，**分配记录树**的根节点是进映射的节点：两份提示都读不出，就经中央映射回退，再读一次同一个槽（`reads_of_each_copy=(2, 1)`），读得出，于是隔离 14、计数 0、不复用。这第二次读是 D19 的映射回退，不是 C554 的重读。
   - **树表**豁免映射，只按位置条目读一遍（`(1, 1)`），没有第二次读，结局与一直读不出相同。
4. **问题 2**：会让之后的发布复用被抛弃根还引用着的单元的，是上表里「隔离 0 / 0」的五格：分配记录树一直读不出、分配记录树钩子臂，以及树表的三格。每一格在池级 checker 上都红 I-7.4，而且只红它。另外两格（对照、分配记录树只坏第一次读）到 txg 31 都没复用（txg 32 的根会盖掉 C 的根槽），I-7.4 Holds。
5. 附带看到的（不在题面里，照实写）：账**在盘上真坏**（两份抹零），又没有任何复用时，池级 checker 对 I-7.4 同样判红，理由是「被抛弃的根……引用的单元已被重新分配或抹头」。所以在「盘上真坏」的镜像上，I-7.4 红分不出「复用了」和「账坏了」两件事。上表的 checker 列用的是盘上逐字节完好的镜像（读故障只注入在挂载与发布那一层），那里红就是复用造成的。

**什么现象会推翻这个定位**：
- (a) 在上面「一直读不出」的任一臂上，钩子次数 ≥ 1，或挂载报 `NewerStateStillUnreadableAfterOneReread`：说明乙管到了这一读。
- (b) 树表「只坏第一次读」那一臂出现隔离 14、计数 0：说明挂载里有第二次读。
- (c) 把 `mount.rs:1252` 那一支改成重读一次之后，(b) 那一臂的结局不变：说明定位错了。

(c) 的反面我在副本里造过一次：改完之后 (b) 那一臂翻成隔离 14、计数 0、不复用、I-7.4 Holds，见第四节。

## 二、复现命令与原样输出

### 2.1 已有三条用例先原样跑一次（副本，改动之前）

命令（在副本根）：
```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test -p singlefs-harness --test unreadable_abandoned_root_slot --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable --test checker_narrow_invariants_and_abandoned_roots > run-baseline.log 2>&1; echo exit=$?
```
输出（`grep -E '^test result|Running|FAILED|panicked'`）：
```
exit=0
     Running tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs (target/debug/deps/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable-e1e70f0fa9242c2e)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.89s
     Running tests/checker_narrow_invariants_and_abandoned_roots.rs (target/debug/deps/checker_narrow_invariants_and_abandoned_roots-51e0944e4e10d5f3)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.37s
     Running tests/unreadable_abandoned_root_slot.rs (target/debug/deps/unreadable_abandoned_root_slot-a18a78bb2603b3c9)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.69s
```

### 2.2 新用例（最小复现，harness 档）

文件：副本里的 `crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs`。仓副本交回前删了，同一份另存在 `/tmp/claude-1000/closeout-recheck-2026-09-28/c393/c393_abandoned_root_account_unreadable_at_remount.rs`（454 行，sha256 `5268690faa767b80a6254ab6995e7f82261e0bef332a26485ee89a28b77300ed`）。放进 `crates/singlefs-harness/tests/` 就能跑，只用 `common` 里已有的 `SharedUnreadableRanges` 与 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`，没改任何 `src/`。
与主工作区的差（命令 `diff -rq crates /home/fy5090/code/singlefs/crates | grep -v target`）：
```
Only in crates/singlefs-harness/tests: c393_abandoned_root_account_unreadable_at_remount.rs
```
每个结局一条用例：

| 用例 | 钉的结局 |
|---|---|
| `c393_readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31` | 对照（两种账各一遍）：隔离 14、计数 0、到 txg 31 不复用、I-7.4 Holds |
| `c393_allocation_record_tree_unreadable_on_every_read` | 分配记录树一直读不出 |
| `c393_allocation_record_tree_unreadable_until_the_reread_hook` | 分配记录树，产品若重读就撤掉（钩子 0 次） |
| `c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy` | 分配记录树只坏第一次：映射回退吸收 |
| `c393_tree_table_unreadable_on_every_read` | 树表一直读不出 |
| `c393_tree_table_unreadable_until_the_reread_hook` | 树表，产品若重读就撤掉（钩子 0 次） |
| `c393_tree_table_unreadable_only_on_the_first_read_of_each_copy` | 树表只坏第一次：与一直读不出相同 |
| `c393_positive_control_the_hook_is_called_when_the_newest_instance_table_is_unreadable` | 钩子的阳性对照：(3, 10) 实例表读不出时钩子 1 次、隔离 14 |
| `c393_checker_on_a_persistently_damaged_account_without_reuse` | 盘上抹零、没有复用时 checker 对 I-7.4 的判定（只断言抹之前 Holds，抹之后的判定打印出来） |

命令：
```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test -p singlefs-harness --test c393_abandoned_root_account_unreadable_at_remount -- --nocapture > run-observe2.log 2>&1; echo exit=$?
```
输出：`exit=0`；`grep 'test result' run-observe2.log`：
```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.40s
```
各臂的产物行（命令 `grep -E '^name=c393 account=' run-observe2.log | sed -E 's/ rereads=.*space_admission=[A-Za-z]*//; s/I-7.4=Violated\("([^"]{0,60})[^)]*"\)/I-7.4=Violated("\1…")/g'`，只删了每行里各臂都一样的 `rereads=…`、`space_admission=AdmittedBeforeAcquisition` 两段，并截短第一个 Violated 的说明；完整的行在 `run-observe2.log`）：
```
name=c393 account=TreeTable unreadability=OnlyTheFirstReadOfEachCopy account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(1, 1) failed_reads=2 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1")] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 501…") red=["I-7.4"] I-7.4_after_zeroing_the_account=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：树表单元 在盘 0 槽 50342 的那一份与位置条目里的校验和对不上；树表单元 两份都读不到对得上的")
name=c393 account=AllocationRecordTreeRoot unreadability=EveryReadUntilTheRereadHook account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(2, 2) failed_reads=4 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1")] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 501…") red=["I-7.4"] I-7.4_after_zeroing_the_account=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；分配记录树（树 13）的根 两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的")
name=c393 account=TreeTable unreadability=EveryRead account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(1, 1) failed_reads=2 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1")] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 501…") red=["I-7.4"] I-7.4_after_zeroing_the_account=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：树表单元 在盘 0 槽 50342 的那一份与位置条目里的校验和对不上；树表单元 两份都读不到对得上的")
name=c393 account=AllocationRecordTreeRoot unreadability=Readable account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(1, 0) failed_reads=0 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
name=c393 account=AllocationRecordTreeRoot unreadability=EveryRead account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(2, 2) failed_reads=4 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1")] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 501…") red=["I-7.4"] I-7.4_after_zeroing_the_account=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50184 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；分配记录树（树 13）的根 两份都读不到对得上的；映射条目指的单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的")
name=c393 account=TreeTable unreadability=EveryReadUntilTheRereadHook account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(1, 1) failed_reads=2 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1")] I-7.4=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 501…") red=["I-7.4"] I-7.4_after_zeroing_the_account=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：树表单元 在盘 0 槽 50342 的那一份与位置条目里的校验和对不上；树表单元 两份都读不到对得上的")
name=c393 account=AllocationRecordTreeRoot unreadability=OnlyTheFirstReadOfEachCopy account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(2, 1) failed_reads=2 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
name=c393 account=TreeTable unreadability=Readable account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(2, 0) failed_reads=0 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
```
被删掉的那一段，在每一臂上都是这一串（命令 `grep -E '^name=c393 account=' run-observe2.log | grep -o 'rereads=.*space_admission=[A-Za-z]*' | sort | uniq -c`）：
```
      8 rereads=RereadsOfThisMount { read_stage: OnTheFirstRead { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(3), checkpoint_txg: CheckpointTxg(10) }, witness: NewerPublishWitness { witnessed_journal_counter: 10, comparison: AgainstTheSelectedVersionsLastRecord { selected_version_last_record_counter: 10 } } } }, instance_table_of_the_newest_root: OnTheFirstRead } space_admission=AdmittedBeforeAcquisition
```
钩子的阳性对照与「盘上真坏」两行（`grep -E '^name=c393-(hook|damaged)' run-observe2.log`，只截掉阳性对照行里 `read_stage` 那一段）：
```
name=c393-hook-positive-control hook_calls=1 isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 instance_table_of_the_newest_root: OnTheOneReread }
name=c393-damaged account=AllocationRecordTreeRoot I-7.4_before=Holds I-7.4_after=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：分配记录树（树 13）的根 在盘 0 槽 50339 的那一份与位置条目里的校验和对不上；分配记录树（树 13）的根 两份都读不到对得上的；映射条目指的单元两份都读不到对得上的") red_after=["I-7.4"]
name=c393-damaged account=TreeTable I-7.4_before=Holds I-7.4_after=Violated("被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：树表单元 在盘 0 槽 50342 的那一份与位置条目里的校验和对不上；树表单元 两份都读不到对得上的") red_after=["I-7.4"]
```
同一组臂一共跑过三次，结局逐臂相同：`run-observe1.log` 只有观测、没有断言，`run-observe2.log` 与 `run-final.log` 带断言、9 条全过。这是确定性装置，三次只说明没有隐藏状态，不是统计上稳定。

## 三、机理（行号在主工作区现取：`awk 'NR==n'`，副本与主工作区的这两份文件逐字节相同）

`crates/singlefs-core/src/mount.rs`：
```
1215: /// 树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载；候选根读不出就当它什么都不豁免（隔离只会多不会少）。
1251:     for root in roots.iter().filter(|root| is_abandoned(root)) {
1252:         let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) else {
1253:             unreadable += 1;
1254:             continue;
1296:     match tree_table_has_no_entries(devices, root) {
1354:             .ok()?
1360:         Err(_failure) => None,
```
- 影子账 `isolate_slots_referenced_only_by_abandoned_roots`（可写挂载经 `rebuilt_allocator` 第 1437–1438 行进来）对每条被抛弃根调一次 `placements_referenced_by_root`。交回 `None` 就在 1253 行加一计数，1254 行 `continue`：这条根引用的落点一个都不进 `allocator.isolate_abandoned`，也不进 `placements_referenced_by_abandoned_root`，于是根环表里这条根的 `referenced_placements` 为空。没有重读，也没有报错交回。
- `None` 从哪来：树表读不出，是 1296 行 `tree_table_has_no_entries` 报错，走 1360 行；分配记录树读不出，是 1348–1354 行 `allocation_records_under_root_in_the_unit_area_starting_at(...).ok()?`。
- 乙-配置续的两处重读都不在这条路上：读阶段只在 4245 / 4251 行判 N-配置（`witnesses_a_publish_newer_than_the_selected_version`）；影子账那一处只重读**最新那条根的实例表**（1179 行首读，1184 行 `before_the_one_reread.before_rereading()`）。被抛弃根 C 比所选那一版旧，它自己的账不在这两样里。

`crates/singlefs-core/src/recovery.rs`（为什么两种账在「只坏第一次」上分叉）：
```
404: pub(crate) fn read_mapped_tree_node_via_hint_then_central_mapping(
412:     if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {
422:     read_unit_via_locations(reader, &mapped_locations, unit_bytes).map_err(|_still_unreadable| {
1868:     let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
1896:             read_mapped_tree_node_via_hint_then_central_mapping(
2111:     let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
```
- 分配记录树的节点经 1896 行走 404 行这个函数：412 行按提示读两份，都读不出就查中央映射，再在 422 行照映射落点（与提示同槽）读一次。「只坏第一次」那一臂的 `reads_of_each_copy=(2, 1)` 就是这三次读：盘 0 提示、盘 1 提示、盘 0 映射。
- 树表在 2111 行（影子账先走的 `tree_table_entry_count`）与 1868 行都只走 `read_unit_via_locations`，两份各读一次，没有映射回退。对照臂树表 `reads_of_each_copy=(2, 0)`：读得出时 1296 行、1868 行各读盘 0 一次。「只坏第一次」那一臂是 `(1, 1)`：第一次就在 1296 行失败，1868 行走不到。

复用怎么发生：C 独占的 14 槽不在实例 4 那一版的账里，影子账又没隔离，重开之后分配器把它们当空闲槽（`free_units_of_c_after_remount` 列出 C 那次发布的 12 个起点槽，含数据单元 50184、分配记录树根 50339、树表 50342）。重开之后第一次覆盖写（txg 14）就把新数据单元落在 50184。C 的根槽要到 txg 32 才被盖掉（`unreadable_abandoned_root_slot.rs` 的 `the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot` 钉的是这一点），所以 txg 14 的时候 C 还在根环里，I-7.4 判红。

## 四、推翻条件，与在副本里造的那一次

推翻条件 (c)：`mount.rs:1252` 那一支读不出时要是重读一次，树表「只坏第一次」那一臂就该翻成隔离 14、计数 0、不复用；一直读不出的各臂结局不变，只有读的次数翻倍。
造法（副本里改，跑完换回原文件，换回之后与主工作区 `diff -q` 相同）：
```
--- mount.rs (工作区原样)
+++ mount.rs (推翻条件造法：影子账读不出时重读一次)
@@ -1252 +1252,3 @@
-        let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) else {
+        let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root)
+            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))
+        else {
```
命令同 2.2（输出落 `run-mutant-reread.log`），`exit=101`。`grep -E '^test |test result|panicked'`：
```
test c393_positive_control_the_hook_is_called_when_the_newest_instance_table_is_unreadable ... ok
thread 'c393_tree_table_unreadable_until_the_reread_hook' (1230564) panicked at crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs:302:5:
test c393_tree_table_unreadable_until_the_reread_hook ... FAILED
thread 'c393_tree_table_unreadable_on_every_read' (1230561) panicked at crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs:302:5:
test c393_tree_table_unreadable_on_every_read ... FAILED
thread 'c393_allocation_record_tree_unreadable_until_the_reread_hook' (1230557) panicked at crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs:302:5:
test c393_allocation_record_tree_unreadable_until_the_reread_hook ... FAILED
thread 'c393_allocation_record_tree_unreadable_on_every_read' (1230555) panicked at crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs:302:5:
test c393_allocation_record_tree_unreadable_on_every_read ... FAILED
thread 'c393_tree_table_unreadable_only_on_the_first_read_of_each_copy' (1230563) panicked at crates/singlefs-harness/tests/c393_abandoned_root_account_unreadable_at_remount.rs:300:5:
test c393_tree_table_unreadable_only_on_the_first_read_of_each_copy ... FAILED
test c393_allocation_record_tree_unreadable_only_on_the_first_read_of_each_copy ... ok
test c393_checker_on_a_persistently_damaged_account_without_reuse ... ok
test c393_readable_control_isolates_14_and_nothing_of_c_is_reused_through_txg_31 ... ok
test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.24s
```
各臂行（`grep -E '^name=c393 account=' run-mutant-reread.log | sed -E 's/rereads=.*space_admission=[A-Za-z]* //' | cut -c1-400`）：
```
name=c393 account=TreeTable unreadability=EveryReadUntilTheRereadHook account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(2, 2) failed_reads=4 hook_calls=0 reuse_txg=14 overwri
name=c393 account=TreeTable unreadability=EveryRead account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(2, 2) failed_reads=4 hook_calls=0 reuse_txg=14 overwritten=[(50184, "t1"
name=c393 account=AllocationRecordTreeRoot unreadability=EveryReadUntilTheRereadHook account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(4, 4) failed_reads=8 hook_calls=0 reuse
name=c393 account=AllocationRecordTreeRoot unreadability=EveryRead account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[50184, 50330, 50332, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341, 50342] isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] abandoned_roots_unreadable=1 reads_of_each_copy=(4, 4) failed_reads=8 hook_calls=0 reuse_txg=14 overwritte
name=c393 account=TreeTable unreadability=OnlyTheFirstReadOfEachCopy account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(3, 1) failed_reads=2 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
name=c393 account=AllocationRecordTreeRoot unreadability=OnlyTheFirstReadOfEachCopy account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(2, 1) failed_reads=2 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
name=c393 account=AllocationRecordTreeRoot unreadability=Readable account_slot=50339 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(1, 0) failed_reads=0 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
name=c393 account=TreeTable unreadability=Readable account_slot=50342 result=Ok(()) current_txg=Some(13) free_units_of_c_after_remount=[] isolated=[(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)] abandoned_roots_unreadable=0 reads_of_each_copy=(2, 0) failed_reads=0 hook_calls=0 no_reuse_through_txg=31 I-7.4=Holds red=[]
```
读法：
- 树表「只坏第一次」那一臂翻了：`isolated` 14 / 14，`abandoned_roots_unreadable=0`，`reads_of_each_copy=(3, 1)`，`no_reuse_through_txg=31 I-7.4=Holds`。它的用例红在第 300 行（计数那一句）。
- 四个一直读不出的臂：隔离仍 0 / 0、计数仍 1、txg 14 仍复用，只有读的次数翻倍（树表 (2, 2)，分配记录树 (4, 4)）。它们红在第 302 行（读次数那一句），没有红在结局那几句上。
- 分配记录树「只坏第一次」、对照、钩子阳性对照、盘上真坏那四条照绿。

所以结局由 1252 行那一支决定，这几条用例分得出「有重读」和「没重读」。
换回原文件之后重跑：`exit=0`，`grep 'test result' run-final.log`：
```
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.77s
```

推翻条件 (a)（钩子 ≥ 1 次或挂载被拒）没在副本里造，理由是要把钩子穿进 `isolate_slots_referenced_only_by_abandoned_roots`，改动面比 (c) 大。钩子这一路的判别力由阳性对照那条用例证：同一段历史上 (3, 10) 实例表读不出时，`hook_calls=1`、`instance_table_of_the_newest_root: OnTheOneReread`。可见各臂的「0 次」不是钩子没接上。

## 五、排除掉的解释与各自的观测

| 解释 | 排除依据 |
|---|---|
| 注入没打到影子账那一读（阴性结果只是代码没跑到） | 对照臂 `reads_of_each_copy` 分配记录树 (1, 0)、树表 (2, 0)：那一段确实被读到了；各读不出臂的 `failed_reads` 为 2 / 4 |
| 注入打到了别的单元，或打到挂载更早的一步 | 各读不出臂 `result=Ok(())`、`current_txg=13`，与对照臂相同：择根、重放、`rebuild_previous_version` 都没受影响；一直读不出的树表臂 `(1, 1)`，读次数与影子账那一次对得上 |
| 乙-配置续本来会拒，只是这次没触发 | 8 个臂的 `rereads` 逐字相同（`uniq -c` 为 8），读阶段在第一遍判完，所选 (3, 10) 就是见证到的那次（`witnessed_journal_counter: 10`，`selected_version_last_record_counter: 10`） |
| 钩子接错了，所以一直是 0 次 | 阳性对照 `hook_calls=1` |
| 复用来自别处，不是因为 C 没被隔离 | 对照臂同样写到 txg 31，一个都没复用；读不出臂被写掉的是 C 那次发布写出的数据单元 50184（`overwritten=[(50184, "t1")]`，逐字节比 C 写的内容），它也在 `free_units_of_c_after_remount` 里 |
| I-7.4 红是 checker 自己读坏了 | checker 读的是 `pool.memory_pool()` 那份镜像，注入只在挂载与发布的设备包装上；对照臂同一装置 Holds |
| 分配记录树「只坏第一次」绿，是因为注入没生效 | `failed_reads=2`，第一次两份都坏了；读次数 (2, 1) 与映射回退那一读对得上 |

## 六、三次推导

- **正推**：第三节的代码路径。`mount.rs` 1296 行、1360 行，或 1348–1354 行交出 `None`，走到 1252–1254 行：只计数、不隔离、不重读；乙的两处重读判的是 N-配置和最新根的实例表，不碰被抛弃根的账。
- **反推**：假如乙罩得住这一读，一直读不出的臂应该出现钩子 ≥ 1 次或挂载报错，实测钩子 0 次、挂载 `Ok`；假如挂载里有第二次读，树表「只坏第一次」那一臂应该隔离 14，实测 0 / 0，`reads_of_each_copy=(1, 1)`。
- **校验**（三条路不共用代码）：core 自己报的隔离数与计数；逐字节比 C 写出的单元（测试里的 `overwritten_units_of`）；`singlefs-checker` 的池级 I-7.4。三条在每一臂上一致。checker 这条路分得出差别：对照臂 Holds，读不出的臂红。

## 七、没做什么

- 没修，没判该怎么修、「修复」该是什么动作。
- 没跑重型测试（checker 档 `singlefs-checker-tier`、54 / 55 / 57 / 59 / 87 号、全量 `cargo test`）。只跑了 `singlefs-harness` 的 4 个测试二进制，都经 `run-with-memory-cap.sh 12G` 与 `capped.sh 10`。
- 没在副本里造推翻条件 (a)（把钩子穿进影子账，或读不出就拒），理由见第四节。
- 只造了一段固定历史（3 代、C 单层分配记录树、两盘）。以下都没试：
  - 分配记录树多层、坏的是根下面某个节点；
  - 账「解不开」而不是「读不出」；
  - 读回全 0 而不是块设备错（`UnreadableRangeReadBack::Zeros`）；
  - 被抛弃根不止一条；
  - 崩溃恢复那次挂载（实例 3）本身读 C 的账。那一次 C 的根槽读不出，C 不在影子账的输入里，这一形是 `unreadable_abandoned_root_slot.rs` 钉的，不在本题的「账」里。
- 没看挂着时抬 F（准入或卸载）重算影子账那一路（`mount.rs` 2353–2363 行）。它会再读一次被抛弃根的账，所以「后来读得出」的臂在抬 F 之后可能补上隔离。本段历史里 txg 14 的复用发生在任何抬 F 之前，挂载报 `space_admission=AdmittedBeforeAcquisition`，这次没有走到那一路。
- E158 第 708 行附近「798 对全 `Ok(count>0)`」没重跑，只读了原文作参考，本报告的结论不引它。
- 副本的改动没回主工作区；主工作区一个字没改。
- 删了什么：仓副本连同它的编译目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c393/repo`（1.5G，`du -sh` 量的），以及那里的临时备份 `/tmp/claude-1000/closeout-recheck-2026-09-28/c393/mount.rs.orig`。留着的：新用例的另存、推翻条件的改法 `mutation-reread-once.diff`、五份运行日志与一份编译日志，都在 `/tmp/claude-1000/closeout-recheck-2026-09-28/c393/` 下。
- `ps` 开跑前看到的：没有性能测量在跑；有别的会话的 python 与 `ask-local.sh`。本次没有等 cargo 的锁。
