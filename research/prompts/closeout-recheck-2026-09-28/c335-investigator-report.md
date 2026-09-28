# C335 复查：根槽持续读不出时的多次可写挂载轨迹（调查员，2026-09-28）

仓：HEAD e5253e8a 加工作区，`rsync -a --exclude target --exclude .git` 拷到 `/tmp/claude-1000/closeout-recheck-2026-09-28/c335/repo/`（交回前已删，见末节）。
只在副本里加用例、改坏过一次 `mount.rs`（已还原并与主工作区 diff 为空）。主工作区一个字没改。

## 一、结论

1. **题面那条吸收态链今天走不通，也没有被乙-配置续换成「第一次就拒可写」。** 只坏一个根槽（每一次读都失败）时，
   两种住法都量了：
   - 坏槽上住较旧的根：20 轮、372 轮可写挂载全做成，与不注入的对照臂逐轮相同（挂载、行数、预留、准入、写的根逐字段相同）。
   - 坏槽上住比下一次所选根新的根：那条根那次发布的末条 journal 记录读得出，择根落到它前一条、重放把它补回来，
     所选那一版就是它，系统配置见证的计数器不比它新，读阶段第一遍判完、不重读，可写挂载照常做成（12 轮全做成）。
   - 乙-配置续（`NewerStateStillUnreadableAfterOneReread`）只在**根槽加上那次发布的末条记录一起读不出**时才拒，
     拒就是从第一次起每一次都拒（10 轮全拒），只读挂载照常，盘面逐字节不动、行数停住。这超出了 C335 题面「一个根槽」的前提。
2. **行数确实只增不减，但与根槽坏不坏无关**：每轮可写挂载 +1 行，对照臂同样 +1。今天 `crates/` 里没有删行的代码
   （`mount.rs:1012` 只往后接），kb 的 I-3.8 那一行也写着「第一版行只增不删，删行的条件没人判」。
   所以「根槽持续读不出 ⇒ 回收条件永远不成立」这一步今天观测不到差别：两边都不删。
3. **「预留拿不到 ⇒ 只读挂载」这一步今天不成立**：D2 已定项 13 与 D16 已定项 1（用户 2026-09-26 定）改成推满仍不够时挂载照样做成可写，
   代码在 `mount.rs:3990`（`MountSpaceAdmission::StillShortAfterTheFloorRaises`），只有树表 0 条的一版在取号之前拒。
4. **写也失败的坏槽**（真坏扇区）：根槽写失败今天是「冻结、原样重发」（同一 txg、同一槽），不是题面说的「推进一格再发」——
   D23 第 397 行写明推进 txg「随失败处置的代码在后面的里程碑做」。实测每 5 轮左右有一轮失败（卸载那一串或可写挂载自己那一串报错），
   下一轮可写挂载照常做成、跳过失败的 txg，不吸收。
5. **第 2 问（跨片）：量到了，不是推的。** 三条臂都在第 367 轮跨片：rows0 = 367，链从 1 片到 2 片，切换预留每块盘 104 → 112 槽（+8），
   准入仍「取号之前就够」，372 轮全做成（单元区 384 槽的盘，每轮挂载、覆盖写一次、正常卸载）。

推翻条件与造它的那一次见第六节。

## 二、原样复现

题面（2026-09-16 第三轮辩方腿）是推出来的，没有跑过的命令、种子和产物行，所以没有「原样复现」可做。我照题面搭了一个装置：
每轮「可写挂载 → 覆盖写 2999 字节一次 → 正常卸载」，根槽读失败用只供测试的开关
`FaultSchedule::every_read_of_named_root_ring_slots_fails`（`crates/singlefs-harness/src/fault_injection.rs`）点名 (区域, 槽)，每一次读都失败。
写失败那一臂另包一层 `InjectedFault::WriteFails`，落点是同一个槽。末条记录那一臂在最外层自己加了一个按重叠判的读失败。
盘是 `HistoryDeviceWidth::UnitAreaOf384Slots`（两块，单元区 384 槽），起步是 `common_admission::PoolUnderTest::start_after_the_first_file`。

开跑前看了负载：`ps` 里没有 qemu、vm-bench、e152、fio，也没有别的 cargo，所以没等锁。
每条 cargo 都经 `research/scripts/run-with-memory-cap.sh 12G` 与 `research/scripts/capped.sh 10` 起，全部退 0，没有 250–254。

## 三、逐轮轨迹（问 1）

装置文件：`/tmp/claude-1000/closeout-recheck-2026-09-28/c335/kept/c335_probe_trajectory.rs`（探查，只打印）。命令（在副本根目录）：

```
C335_CYCLES=372 CARGO_TARGET_DIR=/tmp/claude-1000/closeout-recheck-2026-09-28/c335/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test -p singlefs-harness --test c335_probe_trajectory -- --ignored --nocapture --test-threads 3 probe_no_bad_slot_control probe_older
```

输出末行（`run4-372.log`）与行数（每臂 372 行，三臂 1116 行）：

```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 80.67s
1116
```

每臂汇总（命令对 `run4-lines.txt` 数的，只数 ` mount=` 字段，不含 `unmount=`）：

| 臂 | 可写挂载做成 / 被拒 | 准入 | 预留（每盘槽数，片数）| rows0 ≠ 轮次的做成轮 |
|---|---|---|---|---|
| control-none | 372 / 0 | 372 admission=取号之前就够 | 366 104 槽 1 片;6 112 槽 2 片 | 0 |
| older-read-fail-write-land | 372 / 0 | 372 admission=取号之前就够 | 366 104 槽 1 片;6 112 槽 2 片 | 0 |
| older-read-write-fail | 299 / 73 | 299 admission=取号之前就够 | 294 104 槽 1 片;5 112 槽 2 片 | 0 |

被拒的那一臂（读写都失败）逐个是哪一种（整跑数）：

```
     73  mount=Err:Publish
      1 unmount=Err:RaiseFloorSequencePublishFailed
```

原样行（整行，`run1.log`，20 轮那一跑；第 4、8、12… 轮坏槽 (0,0) 被这一轮的根盖写过，下一轮照常）：

```
name=c335-probe arm=older-read-fail-write-land cycle=3 mount=Ok instance=4 rows_written=1 rows0=3 table_rows_on_disk=Some(3) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=17 bad_slot_written=false read_only=- roots=[14@(2,4),15@(0,5),16@(1,5),17@(2,5),18@(0,6),19@(1,6)]
name=c335-probe arm=older-read-fail-write-land cycle=4 mount=Ok instance=5 rows_written=1 rows0=4 table_rows_on_disk=Some(4) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=23 bad_slot_written=true read_only=- roots=[20@(2,6),21@(0,7),22@(1,7),23@(2,7),24@(0,0),25@(1,0)]
name=c335-probe arm=older-read-fail-write-land cycle=5 mount=Ok instance=6 rows_written=1 rows0=5 table_rows_on_disk=Some(5) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=29 bad_slot_written=false read_only=- roots=[26@(2,0),27@(0,1),28@(1,1),29@(2,1),30@(0,2),31@(1,2)]
name=c335-probe arm=control-none cycle=4 mount=Ok instance=5 rows_written=1 rows0=4 table_rows_on_disk=Some(4) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=23 bad_slot_written=false read_only=- roots=[20@(2,6),21@(0,7),22@(1,7),23@(2,7),24@(0,0),25@(1,0)]
name=c335-probe arm=older-read-write-fail cycle=4 mount=Ok instance=5 rows_written=1 rows0=4 table_rows_on_disk=Some(4) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Err:RaiseFloorSequencePublishFailed bad_slot_written=false read_only=- roots=[20@(2,6),21@(0,7),22@(1,7),23@(2,7)]
name=c335-probe arm=older-read-write-fail cycle=5 mount=Ok instance=6 rows_written=1 rows0=5 table_rows_on_disk=Some(5) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=27 bad_slot_written=false read_only=- roots=[25@(1,0),26@(2,0),27@(0,1),28@(1,1),29@(2,1)]
name=c335-probe arm=older-read-write-fail cycle=9 mount=Err:Publish instance=0 rows_written=0 rows0=0 table_rows_on_disk=None reserve_slots_per_device=0 chain_pages=0 admission=- overwrite=- unmount=- bad_slot_written=false read_only=Ok:(10,48) roots=[]
name=c335-probe arm=older-read-write-fail cycle=10 mount=Ok instance=11 rows_written=1 rows0=10 table_rows_on_disk=Some(10) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=51 bad_slot_written=false read_only=- roots=[49@(1,0),50@(2,0),51@(0,1),52@(1,1),53@(2,1)]
```

坏槽上住比所选根新的根（第 2 轮卸载之后点名这一轮最后一条根 txg 13 的槽 (1,4)；`run3.log`，12 轮全做成）。整行：

```
name=c335-probe arm=newest-read-fail-write-land armed_after_cycle=2 bad_slot=(1,4) with_last_record=false last_record_counter=Some(13)
name=c335-probe arm=newest-read-fail-write-land cycle=3 chosen=(3, 12) effective=(3, 13) read_stage=OnTheFirstRead last_record_counter=Some(19) mount=Ok instance=4 rows_written=1 rows0=3 table_rows_on_disk=Some(3) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=17 bad_slot_written=false read_only=- roots=[14@(2,4),15@(0,5),16@(1,5),17@(2,5),18@(0,6),19@(1,6)]
name=c335-probe arm=newest-read-fail-write-land cycle=6 chosen=(6, 31) effective=(6, 31) read_stage=OnTheFirstRead last_record_counter=Some(37) mount=Ok instance=7 rows_written=1 rows0=6 table_rows_on_disk=Some(6) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=35 bad_slot_written=true read_only=- roots=[32@(2,2),33@(0,3),34@(1,3),35@(2,3),36@(0,4),37@(1,4)]
name=c335-probe arm=newest-read-fail-write-land cycle=7 chosen=(7, 36) effective=(7, 37) read_stage=OnTheFirstRead last_record_counter=Some(43) mount=Ok instance=8 rows_written=1 rows0=7 table_rows_on_disk=Some(7) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=41 bad_slot_written=false read_only=- roots=[38@(2,4),39@(0,5),40@(1,5),41@(2,5),42@(0,6),43@(1,6)]
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 3.66s
```

同上，写也失败（`run7-newest-wfail.log`，12 轮全做成；第 6、10 轮卸载那一串失败，下一轮择到 36 / 60、重放到 37 / 61）：

```
name=c335-probe arm=newest-read-write-fail cycle=6 chosen=(6, 31) effective=(6, 31) read_stage=OnTheFirstRead last_record_counter=None mount=Ok instance=7 rows_written=1 rows0=6 table_rows_on_disk=Some(6) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Err:RaiseFloorSequencePublishFailed bad_slot_written=false read_only=- roots=[32@(2,2),33@(0,3),34@(1,3),35@(2,3)]
name=c335-probe arm=newest-read-write-fail cycle=7 chosen=(7, 36) effective=(7, 37) read_stage=OnTheFirstRead last_record_counter=Some(43) mount=Ok instance=8 rows_written=1 rows0=7 table_rows_on_disk=Some(7) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=41 bad_slot_written=false read_only=- roots=[38@(2,4),39@(0,5),40@(1,5),41@(2,5),42@(0,6),43@(1,6)]
```

边界臂：根槽 (1,4) 加 txg 13 那次发布的末条记录（计数器 13，两块盘各一份）一起读不出（`run3.log`，第 3–12 轮全拒）。整行（取头尾）：

```
name=c335-probe arm=newest-and-record-read-fail armed_after_cycle=2 bad_slot=(1,4) with_last_record=true last_record_counter=Some(13)
name=c335-probe arm=newest-and-record-read-fail cycle=3 chosen=(0, 0) effective=(0, 0) read_stage=- last_record_counter=None mount=Err:NewerStateStillUnreadableAfterOneReread instance=0 rows_written=0 rows0=0 table_rows_on_disk=None reserve_slots_per_device=0 chain_pages=0 admission=- overwrite=- unmount=- bad_slot_written=false read_only=Ok:(3,12) roots=[]
name=c335-probe arm=newest-and-record-read-fail cycle=12 chosen=(0, 0) effective=(0, 0) read_stage=- last_record_counter=None mount=Err:NewerStateStillUnreadableAfterOneReread instance=0 rows_written=0 rows0=0 table_rows_on_disk=None reserve_slots_per_device=0 chain_pages=0 admission=- overwrite=- unmount=- bad_slot_written=false read_only=Ok:(3,12) roots=[]
```

边界臂 10 轮里被拒的轮数（命令数的）：10

窄盘（单元区 240 槽，每轮也正常卸载）20 轮：对照臂与旧根槽读失败臂都是 20 轮全做成、准入全是「取号之前就够」（`run5-narrow.log`）。
推满仍不够那一支只有「只崩了再挂、不卸载」才走得到；已有用例 `crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck`
（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:280`）钉的是第 8 次起每次推满仍不够、15 次挂载全做成。今天在副本里现跑一次：

```
test crash_only_remounts_on_a_narrow_pool_push_floor_raises_after_the_row_publish_and_never_get_stuck ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 9.93s
```

## 四、机理（文件:行，行号在主工作区现取）

| 轨迹 | 走到哪 | 文件:行 |
|---|---|---|
| 每轮 +1 行、一行不删（两臂相同） | 写行那次的表 = 所选那一版的表后面接上这次的行，没有删行分支；这次写哪几行按 [max(上一个实例, 1), 新实例) 列 | `crates/singlefs-core/src/mount.rs:1012`（`rows.extend_from_slice(rows_written)`）、`crates/singlefs-core/src/mount.rs:2968`（`instance_rows_to_write`）；kb `.claude/kb/invariants.md:141` I-3.8「第一版行只增不删，删行的条件没人判」 |
| 切换预留随行数跨片 | 片数 = ⌈(rows0 + N_switch) ÷ 369⌉，乘 (N_switch + 1) | `crates/singlefs-core/src/admission.rs:203`、`crates/singlefs-core/src/admission.rs:219` |
| 预留拿不到也做成可写挂载 | 写行、暖机之后推满仍不够，交回 `StillShortAfterTheFloorRaises`，不报错 | `crates/singlefs-core/src/mount.rs:3990`；kb `.claude/kb/decisions/02-RAID条带策略.md:224` |
| 较旧根的槽读不出：不影响 | 择根只看读得出的根；它不比所选那一版新，见证判据为假 | `crates/singlefs-core/src/mount.rs:4245`（第一遍判假就往下走） |
| 更新的根的槽读不出：重放补回 | 读阶段先择根、扫环、重放，再拿**重放之后**的根去比见证的计数器；那条根的末条记录读得出，重放施加到它，所选那一版就是它 | `crates/singlefs-core/src/mount.rs:4214`（`newer_publish_witness(devices, system_configuration, &effective_root, &records)`）、`crates/singlefs-core/src/mount.rs:4166` |
| 根槽加末条记录都读不出：每次都拒 | 重放补不回，所选那一版比见证的旧，重读一遍仍旧，拒可写；拒在取号之前，一个字节不写，两处坏落点也就永远没有写去盖它们 | `crates/singlefs-core/src/mount.rs:4252` |
| 写也失败：一轮失败、下一轮跳过 | 根槽写失败把这次发布冻结（原样重发，同 txg 同槽），本次挂载或卸载报错；失败那次的 journal 记录已落盘，下一次挂载的第一个 txg 取「环里根与记录的最大 txg + 1」，跳过坏槽那一格 | `crates/singlefs-core/src/transaction.rs:1146`、`crates/singlefs-core/src/transaction.rs:1165`、`crates/singlefs-core/src/mount.rs:1107`；kb `.claude/kb/decisions/23-journal的角色与格式.md:397`（推进 txg 留到后面的里程碑） |

## 五、跨片（问 2）：量的

三条臂都在第 367 轮第一次出现 2 片（命令数的）：

```
control-none cycle=367
older-read-fail-write-land cycle=367
older-read-write-fail cycle=367
```

旧根槽读失败那一臂跨片前后原样行（去掉 roots 字段以外整行保留）：

```
name=c335-probe arm=older-read-fail-write-land cycle=366 chosen=(366, 2191) effective=(366, 2191) read_stage=OnTheFirstRead last_record_counter=Some(2197) mount=Ok instance=367 rows_written=1 rows0=366 table_rows_on_disk=Some(366) reserve_slots_per_device=104 chain_pages=1 admission=取号之前就够 overwrite=Ok unmount=Ok:F=2195 bad_slot_written=false read_only=-
name=c335-probe arm=older-read-fail-write-land cycle=367 chosen=(367, 2197) effective=(367, 2197) read_stage=OnTheFirstRead last_record_counter=Some(2203) mount=Ok instance=368 rows_written=1 rows0=367 table_rows_on_disk=Some(367) reserve_slots_per_device=112 chain_pages=2 admission=取号之前就够 overwrite=Ok unmount=Ok:F=2201 bad_slot_written=false read_only=-
```

式子对得上：⌈(366 + 3) ÷ 369⌉ = 1、⌈(367 + 3) ÷ 369⌉ = 2（`admission.rs:203`）；预留 = 4 × (2 × 片数 + 3 × c_max) 槽，104 槽反解 c_max = 8 槽，2 片时 4 × (4 + 24) = 112，多 8 槽 = (N_switch + 1) × 2，
与 C335 那一行写的「每块盘多 8 块」相同。盘上的表（`instance_table_chain_of_root` 从盘上读）逐轮等于分配器里的 rows0，两条路一致。
要多少次挂载：这个装置每轮写一行（mkfs 同一个进程的实例 1 不写行），第 n 轮 rows0 = n，所以第 367 次可写挂载跨第一片；再往后每 369 次跨一片（推的，只量了到 372 轮）。
写也失败那一臂里失败的可写挂载同样写了一行（取了号、写行那次发布落盘之后才在暖机里失败），做成的轮 rows0 仍等于轮次。
跨片之后在单元区 384 槽的盘上准入没变，一次都没推抬 F；这是这一个盘宽、这一种每轮都正常卸载的节奏下的数，没扫别的盘宽与节奏。

## 六、最小复现（harness 档用例）

文件（副本已删，留在草稿目录）：`/tmp/claude-1000/closeout-recheck-2026-09-28/c335/kept/a_persistently_unreadable_root_slot_leaves_writable_mounts_and_one_row_per_mount_as_without_it.rs`，
放回仓里要进 `crates/singlefs-harness/tests/`，它用 `mod common_admission;`。sha256：

```
061a9dc49ae2a1e6ec12ecd388058f3dd704ed8ca86f5384547ce35092216fb9  kept/a_persistently_unreadable_root_slot_leaves_writable_mounts_and_one_row_per_mount_as_without_it.rs
```

五条用例，一条一个场景：

| 用例 | 场景 | 钉的数 |
|---|---|---|
| `an_older_root_slot_unreadable_on_every_read_leaves_twenty_writable_mounts_as_in_the_control_arm_one_row_each` | 较旧根的槽 (0,0) 读失败、写照落，20 轮 | 每轮做成、rows0 = 轮次、预留 104 槽 1 片、取号之前就够；坏槽在第 4、8、12、16、20 轮被盖写；与对照臂逐轮相同；注入打中过 |
| `an_older_root_slot_whose_reads_and_writes_fail_fails_one_mount_in_five_and_the_next_mount_skips_the_failed_txg` | 同一槽读写都失败，20 轮 | 第 4 轮卸载、第 9 / 14 / 19 轮挂载失败；那几轮只读挂载读回 (10,48)、(15,72)、(20,96)；下一轮第一个 txg 是 25、49、73、97 |
| `a_newest_root_slot_unreadable_on_every_read_is_replayed_from_its_last_record_and_every_writable_mount_is_made` | 最新根 txg 13 的槽读失败，12 轮 | 全做成；第 3、7、11 轮择根与施加后各为 (3,12)→(3,13)、(7,36)→(7,37)、(11,60)→(11,61)，都在第一遍判完 |
| `a_newest_root_slot_and_its_last_record_unreadable_refuse_every_writable_mount_from_the_first` | 同上再坏掉它的末条记录 | 第 3–12 轮全拒 `NewerStateStillUnreadableAfterOneReread`，只读读回 (3,12)，盘面逐字节不变，行数 2 |
| `rows_cross_one_instance_table_page_at_the_367th_mount_and_the_reserve_grows_by_eight_slots_while_every_mount_is_admitted_before_acquisition`（`#[ignore]`，耗时） | 较旧根的槽读失败，372 轮 | 第 367 轮起 2 片、112 槽，之前 1 片、104 槽，全做成、全取号之前就够 |

命令（副本根目录）与原样输出（`final-run1.log`）：

```
CARGO_TARGET_DIR=/tmp/claude-1000/closeout-recheck-2026-09-28/c335/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test -p singlefs-harness --test a_persistently_unreadable_root_slot_leaves_writable_mounts_and_one_row_per_mount_as_without_it -- --include-ignored --test-threads 5
test a_newest_root_slot_and_its_last_record_unreadable_refuse_every_writable_mount_from_the_first ... ok
test a_newest_root_slot_unreadable_on_every_read_is_replayed_from_its_last_record_and_every_writable_mount_is_made ... ok
test an_older_root_slot_whose_reads_and_writes_fail_fails_one_mount_in_five_and_the_next_mount_skips_the_failed_txg ... ok
test an_older_root_slot_unreadable_on_every_read_leaves_twenty_writable_mounts_as_in_the_control_arm_one_row_each ... ok
test rows_cross_one_instance_table_page_at_the_367th_mount_and_the_reserve_grows_by_eight_slots_while_every_mount_is_admitted_before_acquisition has been running for over 60 seconds
test rows_cross_one_instance_table_page_at_the_367th_mount_and_the_reserve_grows_by_eight_slots_while_every_mount_is_admitted_before_acquisition ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 65.81s
```

编译零警告（`build1.log` 与这一次的 `final-run1.log` 里 `^warning` / `^error` 行数：0 / 0）。clippy 没跑。

## 七、推翻条件，与在副本里造出来的那一次

| 定位 | 什么现象会推翻它 | 造没造、结果 |
|---|---|---|
| 行数每轮 +1、与根槽坏不坏无关，因为今天没有删行（`mount.rs:1012`） | 某一轮 rows0 < 轮次，或坏槽臂与对照臂行数不同 | 造了：副本里在 `mount.rs:1012` 之后加「只留最近 5 行」，用例 1 在第 6 轮红（下面原样输出），证明这套逐轮断言看得见删行；之后 `mount.rs` 还原，与主工作区 `diff` 为空 |
| 只坏根槽时更新的根靠它的末条记录重放补回，所以乙-配置续不拒（`mount.rs:4214`） | 只坏根槽、记录读得出，可写挂载却被拒；或择根与施加后相同却照样做成 | 造了反面：再坏掉那条记录，第 3 轮起每轮被拒（边界臂，第三节原样行）。只坏根槽时择根 (3,12) ≠ 施加后 (3,13)，说明那条根确实没被读到、是重放补的 |
| 跨片在单元区 384 槽、每轮卸载的节奏下不改变准入 | 第 367 轮起出现「取号之前就够」以外的判定或挂载失败 | 没另造（换更窄的盘或不卸载的节奏才可能出现）；逐轮断言会不会红由上面那次删行变异证过 |
| 写也失败时一轮失败、下一轮跳过失败的 txg（`mount.rs:1107`） | 失败之后下一轮在同一个 txg 上再失败，或连续两轮失败 | 没另造；20 轮与 372 轮里失败轮之间都隔着做成的轮（用例 2 钉了 20 轮的失败轮次与下一轮的第一个 txg） |

删行变异那一次（`mutant-rowdrop.log`），命令同第六节、只跑用例 1：退出码 101，原样：

```
thread 'an_older_root_slot_unreadable_on_every_read_leaves_twenty_writable_mounts_as_in_the_control_arm_one_row_each' (2928487) panicked at crates/singlefs-harness/tests/a_persistently_unreadable_root_slot_leaves_writable_mounts_and_one_row_per_mount_as_without_it.rs:426:9:
  left: ("Ok", 1, 5, Some(5), 104, 1, "取号之前就够", "Ok", true)
 right: ("Ok", 1, 6, Some(6), 104, 1, "取号之前就够", "Ok", true)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 2.43s
```

## 八、排除掉的解释

- 注入没打中、坏槽没真坏：用例 1、3 断言读失败注入打中过（`fired_count > 0`），用例 2 断言写失败注入打中过；较旧根那一臂坏槽在第 4、8、12… 轮被这一轮的根盖写过，下一轮照常，说明那条盖写过的根同样读不出也不挡。
- 更新的根没被拒是因为系统配置没见证到它：边界臂只多坏一条记录就每轮被拒，说明见证在，差别只在记录读不读得出。
- 行数增长由坏槽引起：对照臂（不点名任何槽）逐轮相同，20 轮逐字段比过，372 轮行数、预留、准入分布相同。
- 准入没变是因为预留没跟着行数算：第 367 轮预留从 104 变 112，式子对得上（第五节）。
- 结果不稳定：装置是确定性的（内存稀疏盘，没有随机源），372 轮那一跑的前 20 轮与单跑 20 轮那一次三臂逐行相同。这只说明没有隐藏状态，不是统计上稳定。

## 九、没做什么

- 没修，没判该怎么改，没判这笔账该不该收口。
- 没跑重型测试与 checker 档（`singlefs-checker-tier`）；每轮盘面没跑池级 checker。
- 新用例没跑 clippy / fmt；没放回主工作区，只留在草稿目录 `kept/`。
- 盘宽只量了单元区 384 槽（372 轮）与 240 槽（20 轮），4 GiB 没跑；节奏只量了「挂载、覆盖写一次、正常卸载」，「只崩了再挂」只现跑了已有的那一条用例；树表 0 条的池（预留不够在取号之前拒）上行数随挂载涨会不会走到拒，没量。
- 坏槽只点了 (0,0) 与 (1,4) 两个，没扫根环全部 24 个槽；每个根环槽的轨迹是否相同是推的。
- 边界臂坏掉末条记录用的是我在最外层自写的按重叠判读失败，不是 `fault_injection` 的开关（它一份计划只挂一条）。
- 附带看到的一处对不上：`.claude/kb/checks-owed.md` C565 那一行说变异在 `crates/mutations.tsv` 第 766 行，工作区里那条（推满仍不够换成报拒绝）在第 763 行；第 766 行是另一条（实五 P3r）。没改。

## 十、删了什么、留了什么

- 删了：仓副本 `/tmp/claude-1000/closeout-recheck-2026-09-28/c335/repo/`（344M）、编译目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c335/target/`（1.1G）、`mount.rs` 的备份 `mount.rs.orig`。
- 留了（草稿目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/c335/`）：`kept/` 下两份用例文件（最小复现与探查装置）；各次运行的日志 `build1.log`、`run1.log`–`run7-newest-wfail.log`、`run4-lines.txt`、`final-run1.log`、`mutant-rowdrop.log`。
  这些是草稿产物，没进 `research/results/`：本轮只查不修，写范围只有报告与草稿目录；要不要入库由主 agent 定。
