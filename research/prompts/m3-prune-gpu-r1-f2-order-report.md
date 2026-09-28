# 调查报告：先根后记录（order 世界）为什么没有一样判定报红

日期：2026-09-28。主仓 HEAD `e5253e8a`，仓副本是工作区（含别的会话未提交的改动）`rsync -a --exclude target --exclude .git` 拷出的；副本里的 `crates/singlefs-checker-tier/src/crash.rs` 与 `crates/singlefs-harness/src/memory_pool.rs` 与主仓逐字节相同（sha256 `ddf64360…` / `dc63d54d…`，两边各算一次，结果一致）。

## 结论

1. **是真的，该红而不红。** 同一份崩溃镜像（段 0..7 全持久、txg 3 的根持久、txg 3 的两条 journal 记录都没持久），写表按健康次序（记录在根前）交给记录核对器时 `root_without_record: true`；写表按攻方次序（根在记录前）交时 `false`。两张写表下 41 个写位置上镜像逐字节相同（`positions_with_differing_bytes=0`）。两遍恢复都读出 txg 3 的正确内容，两遍 oracle 不红，池级 checker 不红，记录核对器也不红：`red_all_judgments=[]`。
2. **按 D16 应当判红。** 纠正派发里的一处出处：发布的持久顺序是 D16（发布语义） **已定项 7**，不是已定项 4。已定项 7 定案（`16-发布语义.md:179`）规定记录与根之间有一道屏障，所以「根在、它的记录一条都不在」在这个顺序下走不到；它的射程第三条（`16-发布语义.md:185`）点名这一类状态是第二道屏障要挡的「根在案而记录缺席」，写明「记录核对器与反向链的输入有洞」。记录核对器这条判据自己的定义（`memory_pool.rs:776`）说的就是这个状态。已定项 4 的第 ④ 行（`16-发布语义.md:118`）把「根在而记录缺」定为只伤记录核对器、oracle 看不到的那一类，这一点和实测对得上：oracle 不红。
3. **今天没有哪一样逐状态判定抓得到。** 层 0 在每个状态上跑五样：看 journal 与不看 journal 两遍恢复各接一遍 oracle、池级 checker、记录核对器（`crash.rs` 的 `evaluate_state_recording_findings`）。攻方构造下五样全不红；我在副本里把真实现（`transaction.rs` 的 `persist_publish_writes`）也改成先根后记录，45 个枚举状态里有 27 个按记录字节认出「根在、它自己的记录全缺」，一个都没红。这个次序错能不能在别处被抓到：`new_pool_file_creation_publish.rs:373` 等处钉死了段序列的种类串，改坏后的真流段序列已经不一样（见「真实现改坏」一节），所以那几条 harness 用例多半会红——这是**推的，没跑**。那些用例钉的是整条流的形状，不是逐状态判定，也只罩被钉的那几条路径。
4. **机理有两层，缺一层都不会哑。** 第一层，`publishes_of_one_recording`（`memory_pool.rs:993`）按写表次序归发布：记录只归给排在它**后面**的第一条根。所以根在前的时候，这次发布分到的记录表是空的，它自己的记录要么归给下一次发布（后面还有根时），要么谁都不归（后面没有根时：`memory_pool.rs:1019` 直接返回，循环结束时手里剩下的记录被丢掉）。第二层，`crash.rs:139` 的 `!publish.records.is_empty()` 把记录表为空的发布整个跳过，于是攻方那一格一定不红。这道守卫原本的用意（`crash.rs:83`）是「一次发布一条记录都没写过」时不判；但它只看得到归进来的记录表，分不清「这次发布没写记录」和「这次发布的记录排在根后面」。
5. **攻方原话要收窄一处。** 「根之后的记录归到下一次发布」只在后面还有根时成立（`order_next` 世界：txg 3 的两条记录归进了 txg 4，`records=[37, 38, 65, 66]`）。攻方的 order 世界里新建文件就是最后一次发布，那两条记录谁都不归（`journal_records_in_no_publish=[37, 38]`）。两种情形都不红。

## 复现：攻方 order 世界原样复跑

照 `research/prompts/m3-prune-gpu-r1-opus-model/rerun.sh` 的挂法（先核 `SHA256SUMS`，13 项全部 OK），只编 E161 装置、只跑 order 这一个世界：

```
cd /tmp/claude-1000/investigate-f2-order/repo
CARGO_TARGET_DIR=/tmp/claude-1000/investigate-f2-order/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 cargo build --release --offline -p singlefs-checker-tier --bin e161_crash_state_dedup_and_time_split
nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 $B attack order 2>order.err | grep -v '^LAYER0' > order.out
```

输出：

```
exit=0
order 与存档逐行相同（去掉带 seconds= 的行）
E7RESULT name=attack_materialized_summary world=healthy_first_stream enumerated_states=45 red_names_full_local_localimage_fullrecords={} judgment_differs_materialized_image_with_full_record_stream_total=0 must_be_nonzero=0
E7RESULT name=attack_materialized_summary world=root_before_records_mutant enumerated_states=45 red_names_full_local_localimage_fullrecords={} judgment_differs_materialized_image_with_full_record_stream_total=0 must_be_nonzero=0
```

`sha256sum order.out` = `d52a62b16c951b15012643bb6ab9e5d6ca710470e48e10f974269632f079597f`，与存档 `SHA256SUMS` 里 `outputs/order.out` 那一行逐字相同，所以整份输出逐字节一致。

## 最小复现（世界 `order_min`，副本里在攻方 attack.rs 后面加的）

加的代码全文在 `/tmp/claude-1000/investigate-f2-order/attack-additions.diff`（这份 diff 留着，副本删了）。做法：录一遍今天的真实现（mkfs → 取号 → 暖机两次 → 新池新建文件），写表的改法与攻方 order 世界相同（段 8 的两条记录和段 9 的根对调）；然后手摆一个状态：段 0..7 全持久，txg 3 的根持久，两条记录不持久，系统配置分不持久（A）和持久（B）两种。同一个持久集合分别按健康写表和攻方写表交进去，逐样判。两张写表各按自己的下标摆这个集合。

```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 4 $B attack order_min
```

原样输出（exit=0；同一份二进制跑两次，两次逐行相同）：

```
E7RESULT name=investigate_order_segments healthy_seg8=["journal_record", "journal_record"] seg9=["root_record_fua"] seg10=["system_configuration_slot", "system_configuration_slot"] write_count=41
E7RESULT name=investigate_order_publishes stream=healthy publishes=[(txg=1 root=4 units=0 records=[2, 3]),(txg=2 root=9 units=0 records=[7, 8]),(txg=3 root=38 units=24 records=[36, 37])] journal_records_in_no_publish=[]
E7RESULT name=investigate_order_publishes stream=root_before_records publishes=[(txg=1 root=4 units=0 records=[2, 3]),(txg=2 root=9 units=0 records=[7, 8]),(txg=3 root=36 units=24 records=[])] journal_records_in_no_publish=[37, 38]
E7RESULT name=investigate_order_state label=A_root_persisted_records_absent_sysconf_absent/healthy_write_order recovery_consult=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) recovery_ignore=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) consult_journal=JournalScanReport { valid_records: 2, above_water: 0, prefix_applied: 0, verification_passed: 0, verification_failed: 0, maximum_applied_transaction: 0 } record_check=RecordCheck { root_without_record: true, claimed_state_missing_unit: false } red_all_judgments=["records:root_without_record"] tail_write_kinds_persisted=["unit_write+", "unit_write+", "unit_write+", "journal_record-", "journal_record-", "root_record_fua+", "system_configuration_slot-", "system_configuration_slot-"]
E7RESULT name=investigate_order_state label=A_root_persisted_records_absent_sysconf_absent/root_before_records_write_order recovery_consult=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) recovery_ignore=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) consult_journal=JournalScanReport { valid_records: 2, above_water: 0, prefix_applied: 0, verification_passed: 0, verification_failed: 0, maximum_applied_transaction: 0 } record_check=RecordCheck { root_without_record: false, claimed_state_missing_unit: false } red_all_judgments=[] tail_write_kinds_persisted=["unit_write+", "unit_write+", "unit_write+", "root_record_fua+", "journal_record-", "journal_record-", "system_configuration_slot-", "system_configuration_slot-"]
E7RESULT name=investigate_order_state label=B_only_records_absent/healthy_write_order recovery_consult=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) recovery_ignore=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) consult_journal=JournalScanReport { valid_records: 2, above_water: 0, prefix_applied: 0, verification_passed: 0, verification_failed: 0, maximum_applied_transaction: 0 } record_check=RecordCheck { root_without_record: true, claimed_state_missing_unit: false } red_all_judgments=["records:root_without_record"] tail_write_kinds_persisted=["unit_write+", "unit_write+", "unit_write+", "journal_record-", "journal_record-", "root_record_fua+", "system_configuration_slot+", "system_configuration_slot+"]
E7RESULT name=investigate_order_state label=B_only_records_absent/root_before_records_write_order recovery_consult=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) recovery_ignore=FileRead(root=(InstanceGeneration(1), CheckpointTxg(3)),content_matches_version=true) consult_journal=JournalScanReport { valid_records: 2, above_water: 0, prefix_applied: 0, verification_passed: 0, verification_failed: 0, maximum_applied_transaction: 0 } record_check=RecordCheck { root_without_record: false, claimed_state_missing_unit: false } red_all_judgments=[] tail_write_kinds_persisted=["unit_write+", "unit_write+", "unit_write+", "root_record_fua+", "journal_record-", "journal_record-", "system_configuration_slot+", "system_configuration_slot+"]
E7RESULT name=investigate_order_same_image write_positions_compared=41 positions_with_differing_bytes=0
E7RESULT name=done emitted=9
```

问题 1 要的四样判定，在状态 A 上：

| 判定 | 攻方写表（先根后记录） | 健康写表（同一份镜像） |
|---|---|---|
| 恢复（看 journal） | `FileRead`，根 (1, 3)，内容与版本相同 | 相同 |
| 恢复（不看 journal） | `FileRead`，根 (1, 3)，内容与版本相同 | 相同 |
| oracle 两遍 | 不红 | 不红 |
| 池级 checker | 不红 | 不红 |
| 记录核对器 `root_without_record` | **false** | **true** |
| 记录核对器 `claimed_state_missing_unit` | false | false |

这里用健康写表去判攻方的那份镜像，只是对照：健康次序本身走不到这个状态（中间有屏障）。它说明的是，同一份镜像的判定只取决于写表里根和记录谁在前。

## 机理（文件:行号现取自主仓）

- `crates/singlefs-checker-tier/src/crash.rs:138` 在 `publishes_in(writes, persisted, continuity)` 返回的每次发布上判两条判据。
- `crates/singlefs-harness/src/memory_pool.rs:967`：层 0 用的是 `OneRecording`，交给 `publishes_of_one_recording(writes, 0)`（`memory_pool.rs:993`）。
- `memory_pool.rs:1000–1009`：按写表次序往下走，单元写（1003）和记录写（1004）先攒着，遇到根槽写（1005）才把攒下的整批用 `std::mem::take` 交给这条根（1007–1009）。所以一条记录归哪次发布，只看它后面第一条根是谁，不看记录自己写的 (实例, txg)。
- `memory_pool.rs:1019`：循环结束就返回，最后一条根之后剩下的单元写与记录写不归任何一次发布（攻方世界里的 `journal_records_in_no_publish=[37, 38]`）。
- `crash.rs:139–143`：

```
        if !publish.records.is_empty()
            && in_place(publish.root)
            && !publish.records.iter().any(|record| in_place(*record))
        {
            check.root_without_record = true;
```

  记录表为空，第一个合取项就是假，整条跳过。守卫的用意写在 `crash.rs:83`：「一次发布一条 journal 记录都没写过时不判「根在而记录一条都不在」（记录流本来就是空的，不是有洞）」。可它读到的只是按写表次序归进来的记录表，而先根后记录时，「没写过记录」和「记录排在根后面」这两种情形归出来的都是空表，守卫分不开。
- 第二条判据 `claimed_state_missing_unit`（`crash.rs:145`）判的是单元缺席，与记录在不在无关：这 24 个单元写仍排在根前、仍归 txg 3，状态 A 下全持久，所以它不红，这是对的。

## 攻方原话「归到下一次发布」的核对（世界 `order_next`）

在新建文件之后再接一次覆盖写（txg 4），只把 txg 3 的根挪到它自己的两条记录前面；状态是 txg 3 的根及之前全持久、txg 3 的两条记录和之后的写全没持久。原样输出（exit=0）：

```
E7RESULT name=investigate_order_next roots=[4, 9, 38, 67] write_count=70
E7RESULT name=investigate_order_next stream=healthy publishes=[(txg=1 root=4 units=0 records=[2, 3]),(txg=2 root=9 units=0 records=[7, 8]),(txg=3 root=38 units=24 records=[36, 37]),(txg=4 root=67 units=24 records=[65, 66])] journal_records_in_no_publish=[]
E7RESULT name=investigate_order_next stream=txg3_root_before_its_records publishes=[(txg=1 root=4 units=0 records=[2, 3]),(txg=2 root=9 units=0 records=[7, 8]),(txg=3 root=36 units=24 records=[]),(txg=4 root=67 units=24 records=[37, 38, 65, 66])] journal_records_in_no_publish=[]
E7RESULT name=investigate_order_next state=txg3_root_persisted_records_and_later_absent effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) record_check=RecordCheck { root_without_record: false, claimed_state_missing_unit: false }
E7RESULT name=done emitted=5
```

后面还有根时，记录确实归到了下一次发布（txg 4 分到 `[37, 38, 65, 66]`）；后面没有根时（攻方的 order 世界、`order_min`），记录谁都不归。两种都不红。

## 真实现改坏：`persist_publish_writes` 改成先根后记录（世界 `order_impl`）

攻方只是在写表里对调了几个下标。为了排除「只是手摆写表才出现」这种解释，我在副本里把 `crates/singlefs-core/src/transaction.rs` 的 `persist_publish_writes`（主仓 1047 行起：单元 1051 → 屏障 1058 → 记录 1059 → 屏障 1065 → 根与系统配置 1066）改成「单元 → 屏障 → 根与系统配置（含其后的屏障）→ 记录 → 屏障」（diff 在 `/tmp/claude-1000/investigate-f2-order/transaction-root-before-records.diff`）。这个函数由三条发布路径共用，所以两次暖机和新建文件都跟着改了。然后录一遍流、按攻方的展开法枚举（段长 < 16 的段展开，共 45 个状态），每个状态跑那五样判定。判「该红」**不经** `publishes_of_one_recording`：按记录自己字节里的 (实例, txg)（`journal.rs` 的 `to_bytes`，偏移 16 与 26）去对根身份，数「根持久、按字节认出的它自己那几条记录全没持久」的状态。

健康实现（同一套计数，作阴性对照）：

```
E7RESULT name=investigate_order_impl publishes=[(txg=1 root=4 units=0 records=[2, 3]),(txg=2 root=9 units=0 records=[7, 8]),(txg=3 root=38 units=24 records=[36, 37])] journal_records_in_no_publish=[]
E7RESULT name=investigate_order_impl own_records_after_each_root=[(4, [2, 3]), (9, [7, 8]), (38, [36, 37])]
E7RESULT name=investigate_order_impl states=45 states_root_persisted_own_records_all_absent=0 of_those_red_by_any_judgment=0 red_by_name={} example_tail_from_write_30=None
```

（按字节认出的归属和按写表次序归的逐项相同；该红 0 个，判红 0 个。）

改坏的实现：

```
E7RESULT name=investigate_order_impl segments=["2xsystem_configuration_slot", "1xroot_record_fua", "2xsystem_configuration_slot", "2xjournal_record", "1xroot_record_fua", "2xsystem_configuration_slot", "2xjournal_record", "24xunit_write", "1xroot_record_fua", "2xsystem_configuration_slot", "2xjournal_record"] write_count=41
E7RESULT name=investigate_order_impl publishes=[(txg=1 root=2 units=0 records=[]),(txg=2 root=7 units=0 records=[5, 6]),(txg=3 root=36 units=24 records=[10, 11])] journal_records_in_no_publish=[39, 40]
E7RESULT name=investigate_order_impl own_records_after_each_root=[(2, [5, 6]), (7, [10, 11]), (36, [39, 40])]
E7RESULT name=attack_state_budget world=implementation_root_before_records closed_form_state_count=16777240 enumerated_with_torn=45 writes=41 segments=11
E7RESULT name=investigate_order_impl states=45 states_root_persisted_own_records_all_absent=27 of_those_red_by_any_judgment=0 red_by_name={} example_tail_from_write_30=Some("unit_write-,unit_write-,unit_write-,unit_write-,unit_write-,unit_write-,root_record_fua-,system_configuration_slot-,system_configuration_slot-,journal_record-,journal_record-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-,system_configuration_slot-")
```

45 个状态里有 27 个是「根在、它自己的记录全缺」，五样判定一个都没红。归属也整体错开了一位：txg 2 分到的 `[5, 6]` 其实是 txg 1 的记录，txg 3 分到的 `[10, 11]` 其实是 txg 2 的，txg 3 自己的 `[39, 40]` 谁都不归。所以真实现这样改坏时，哑掉的不只是空表守卫这一处：txg 2、txg 3 分到的表不是空的，但里面是上一次发布的记录，而那几条记录前面有屏障，根持久时它们必然已持久，判据当然不会成立。

这条流的段序列 `2+1+2+2+1+2+2+24+1+2+2`，和 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:381` 钉死的 `2+2+1+2+2+1+2+24+2+1+2` 已经不一样（种类串钉在同一文件 385 行，所在用例是 302 行的 `recorded_paths_match_the_registered_segment_sequences`），所以那条 harness 用例多半会红——**推的，没跑**（派发只准跑 E161 装置）。

## 推翻条件，以及在副本里造的那一次

**会推翻这个定位的现象**：
- (a) 把 `crash.rs:139` 的 `!publish.records.is_empty() &&` 去掉，攻方的状态 A 在攻方写表下仍不红。那就说明跳过不在这道守卫上。
- (b) `publishes_of_one_recording` 打出来，攻方写表下 txg 3 的记录表不是空的。
- (c) 同一份镜像在两张写表下判定相同，或者镜像字节不同。那就说明差别不来自写表次序。

(b)、(c) 在最小复现里已经直接看过：txg 3 的记录表是 `records=[]`；两张写表下判定分别是 true 与 false；镜像 `positions_with_differing_bytes=0`。(a) 在副本里造了一次：去掉守卫（diff 在 `/tmp/claude-1000/investigate-f2-order/crash-guard-removed.diff`），重编后跑四个世界。原样输出：

```
E7RESULT name=investigate_order_state label=A_root_persisted_records_absent_sysconf_absent/root_before_records_write_order record_check=RecordCheck { root_without_record: true, claimed_state_missing_unit: false } red_all_judgments=["records:root_without_record"]
E7RESULT name=investigate_order_state label=B_only_records_absent/root_before_records_write_order record_check=RecordCheck { root_without_record: true, claimed_state_missing_unit: false } red_all_judgments=["records:root_without_record"]
E7RESULT name=investigate_order_next state=txg3_root_persisted_records_and_later_absent effective_root=Some((InstanceGeneration(1), CheckpointTxg(3))) record_check=RecordCheck { root_without_record: true, claimed_state_missing_unit: false }
E7RESULT name=attack_materialized_summary world=healthy_first_stream enumerated_states=45 red_names_full_local_localimage_fullrecords={} judgment_differs_materialized_image_with_full_record_stream_total=0 must_be_nonzero=0
E7RESULT name=attack_materialized_summary world=root_before_records_mutant enumerated_states=45 red_names_full_local_localimage_fullrecords={"records:root_without_record": (11, 0, 11)} judgment_differs_materialized_image_with_full_record_stream_total=0 must_be_nonzero=11
E7RESULT name=investigate_order_impl states=45 states_root_persisted_own_records_all_absent=0 of_those_red_by_any_judgment=0 red_by_name={} example_tail_from_write_30=None
```

（最后一行是健康实现；再把 transaction.rs 改坏、守卫也去掉：）

```
E7RESULT name=investigate_order_impl states=45 states_root_persisted_own_records_all_absent=27 of_those_red_by_any_judgment=27 red_by_name={"records:root_without_record": 36} example_tail_from_write_30=Some(…同上…)
```

去掉守卫后攻方那一格红了，所以 (a) 这个观测做得出来，而今天的代码上它没出现。跳过确实发生在 `crash.rs:139`。

同一次输出也说明，**光去掉守卫并不是一个分得清的判据**（这只是观测，怎么修归主 agent）：
- 攻方世界里红了 11 个状态。按 `order.out` 的分段，这 11 个是段 9 的 3 个加段 10 的 8 个，也就是 txg 3 的根持久的全部状态；其中只有段 9 的空子集是真的记录全缺，其余 10 个两条记录至少落了一条，照样红了。原因是空表上的 `!any(...)` 恒真。
- 真实现改坏时红了 36 个，按字节认出的真洞只有 27 个，多出的 9 个是假红。这 27 个全被抓到，是因为 txg 1 的记录表是空的，只要 txg 1 的根持久就红，不是 txg 2、txg 3 的错位被认了出来。
- 健康流两处都还是 0 红，这是因为健康流里每次发布都分到了非空的记录表。`crash.rs:83` 说随机历史上会有一条记录都不写的发布，那种流上去掉守卫会怎样，我没量。

## 排除掉的解释

| 解释 | 排除它的观测 |
|---|---|
| 状态 A 在攻方的枚举里没被走到（阴性结果其实是代码没跑到） | `order_min` 直接手摆了这个状态并跑了全部判定；攻方 order 世界 `root_before_records_mutant segment=9 states=3`，空子集在里面；判定代码跑到了，同一份镜像换健康写表就判红 |
| 镜像本身不一样（攻方对调下标时连字节也变了） | `positions_with_differing_bytes=0`（41 个写位置） |
| 恢复在这个状态上出错，只是 oracle 没看出来 | 两遍恢复都是 `FileRead`，根 (1, 3)，`content_matches_version=true`；看 journal 那一遍 `valid_records: 2, prefix_applied: 0` |
| 只是手摆写表才有的假象，真实现改坏不会这样 | `order_impl` 改坏实现：27 个该红的状态，0 个红 |
| 攻方说「归到下一次发布」而它的世界里没有下一次发布，说法不成立 | `order_next`：有下一次发布时确实归过去了；没有时 `journal_records_in_no_publish=[37, 38]`，谁都不归。两种都不红 |
| 池级 checker 本该抓 | 状态 A 两种写表下池级 checker 都不红（`red_all_judgments` 里没有 `checker:` 项）。我没打这个状态上每条不变量是 Holds 还是 NotApplicable，见「没做什么」 |

## 删了什么

- 已删：仓副本 `/tmp/claude-1000/investigate-f2-order/repo`（330M）、编译目录 `/tmp/claude-1000/investigate-f2-order/target`（386M）。
- 留下：本报告，三份 diff（`attack-additions.diff`、`transaction-root-before-records.diff`、`crash-guard-removed.diff`），各世界的 `.out` 与 `.err`，`build*.log`，`transaction.rs.orig` 与 `crash.rs.orig`（改前的原文件，和主仓逐字节相同）。都在 `/tmp/claude-1000/investigate-f2-order/`，没入库：派发只给了草稿目录写，要不要进 `research/results/` 由主 agent 定。

## 没做什么

- 没修，也没判该怎么修。「去掉守卫会假红」只是副本里的一次观测，不是改法建议。
- 没跑重型测试。没跑 `cargo test` 的任何目标，包括 harness 档的 `new_pool_file_creation_publish`：它会不会在真实现改坏时红是推的。没跑门禁，没跑层 0。
- 只跑了 E161 装置副本的 order、order_min、order_next、order_impl 四个世界，没跑攻方另外九个世界。
- 没打状态 A 上池级 checker 每条不变量的 Holds / NotApplicable 分布；只看了有没有 Violated。
- 在一条记录都不写的发布出现的随机历史上，守卫去掉会怎样，没量。
- 按字节认记录归属用的偏移（16、26）是照 `journal.rs` 的 `to_bytes` 读出来的；健康流上它和按写表次序归的结果逐项相同，别的格式没核。
- 攻方的「先根后记录」只在新建文件那一次发布上做；真实现改坏那一版动的是三条发布路径共用的函数。覆盖写、写行、抬 F 等别的路径上没单独跑。
- 副本里的改动没带回主工作区。
