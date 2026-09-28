# E158 第 4 次跑第二段：产物（执行员，2026-09-27）

## 一、结论

- 第二段 12 条臂的产物与 `r4-compare` 都跑完，13 份都以 `E7RESULT name=done` 收尾，第一行都是 `E7INPUT name=crates_snapshot key=E158 sha256=ad9f544263a5a61bf0bed305ea6e98701042e63ff2ad5e96f2b66552f360f9dc`。开跑检查（5.6 回比、S9 的种子与写死历史、执行器逐步对拍）全过，**没有 `name=stop`**；S5：实七-甲、实八复现，**实七-乙两边都复现不出（S5b，不停，标「底座上复现不出」）**。
- **-配置 各臂（甲、乙、丙、乙-窄读，配置与配置续共 8 条臂）在这一段四族上的丢写格数逐个相同**：H-随 6/37，实七-甲 0/10，实七-乙 3/6，实八 0/2。**乙-配置 = 乙-配置续 = 同一组数。** -槽 各臂（甲-槽、乙-槽、乙-窄读-槽）与今天逐族相同：43/43、10/10、1/5、3/6。
- 照登记的字面算，乙-配置、乙-配置续在 H-随 与实七-乙 上丢写都大于 0。两处要主 agent 先看：
  1. H-随 的 6 格**全部来自 ③c 尾巴**（`q3c_lost_cells=6`）；不算 ③c 是 2 格，Q1 只算环内、也不算 ③c 是 1 格；Q2 回滚 0、Q3 读回丢写 0（修订第 23 条的分解栏）。
  2. 实七-乙 的 3 格是 `crash0`–`crash2` 三个崩溃状态格，读回写的是 `failed:content_differs_from_the_ledger`。我读代码的推断（**推的，没量过**）：崩溃分支的读回拿的是这一臂主路径的账（`main.ledger`，副本 bin 第 20458 行 `let after_mount_read_back = classify_random_read_back(`、第 20460 行传 `&main.ledger,`），按根的键 (2:4) 查内容号。-配置 各臂主路径上实例 2 是在别的时点生的，2:4 在账里对应内容号 1，所以键撞上了。这样的话，这 3 格是装置记账的毛病，不是这几条臂丢了写。今天那一臂同一格读回 `last_confirmed`，被测挂载的起始镜像指纹、选到的根、`last_confirmed`、`e_content` 两臂都一样（见第四节）。
- `crates/mutations.tsv` 第 659 行的替换行写进了草稿 `/tmp/claude-1000/e158-r4-seg2-run/mutations-replacements.tsv`（sha256 `9d24ef37…cdcd`），锚点在主工作区 bin 与臂副本 bin 上都只命中 1 次。**不过点名的用例在今天主工作区上替换之前就已经红了**（见第七节），在主工作区上「仍红」证明不了什么；它只在快照那一版上证过：基线绿、替换之后红。
- 推翻条件：任一份产物的 `r4_random_summary` / compare 的 `r4_loss` 行与下面抄的不一样；有人在 `research/results/` 下重写了这 13 份（sha256 在草稿 `products.sha256`）；在崩溃分支里改用分支自己的账之后，-配置 各臂在实七-乙 上的 `crash0`–`crash2` 仍然是 `content_differs`（那样第 2 条推断就错了）。

## 二、怎么跑的

- 负载（`ps -o pid,args -u "$(id -u)"`）：没有 qemu、vm-bench、e152、fio。别的会话有两条 cargo 在跑（`impl-layer0-findings` 那边的 `cargo test --lib`，`impl-rev-b3c3` 那边的 `cargo build --all-targets`），没等锁。
- 今天那一臂先跑（`run_arm.sh 4 today`，经 `capped.sh 4`、`run-with-memory-cap.sh 10G`）跑完，rc=0，用时 166 秒。其余 11 条臂分 4 条 lane 并行，**每条命令用 `capped.sh 1`**，四条合起来正好 4 个线程（装置本身是单线程；只有今天那一臂会调 harness 的注入函数，那些函数按线程变量开工人线程）。11 条都 rc=0，最后一条跑完。这和派发写的每条 `capped.sh 4` 不一样，是为了四条并行时不超过 4 个线程。
- 环境变量照 `research/prompts/e158-r4-seg2-device-runner-report.md` 的 `run_lane.sh`：另设了 `SINGLEFS_E158_INVESTIGATOR_REPORT`（`research/prompts/m2-investigate-gate74-reds-report.md`，sha256 `a03da8b5…1bc1bf14`，与登记第 434 行一致），以及非今天臂要读的 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT`。脚本 `/tmp/claude-1000/e158-r4-seg2-run/run_arm.sh`。
- 12 份臂副本里的 bin sha256 都是 `12b82f491453e4f5…`。**主工作区的 bin 已经不是这一版**（此刻 sha256 `06d75b963e45a891…`，与副本差 5 行：第 19076 行的 match 多了 `UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }` 一支）。不是我改的。
- compare：在 `arms/today` 里跑 `r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg2`，rc=0，375 行。

产物（全在 `research/results/`）：`e158-root-choice-repair-2026-09-27-r4-seg2-{today,jia-cfg,jia-cfg-carry,jia-slot,yi-cfg,yi-cfg-carry,yi-slot,yi-narrow-cfg,yi-narrow-cfg-carry,yi-narrow-slot,bing-cfg,bing-cfg-carry}.out` 与 `…-r4-seg2-compare.out`。

## 三、产物齐不齐（4b）

命令（在 `research/results/` 下）：

```
for f in e158-root-choice-repair-2026-09-27-r4-seg2-*.out; do a=${f#e158-root-choice-repair-2026-09-27-r4-seg2-}; a=${a%.out}; [ $a = compare ] && continue; echo "$a summary=$(grep -c 'name=r4_random_summary' $f) all_summary=$(grep -c 'name=r4_random_all_summary' $f) pc_random=$(grep -c 'name=r4_positive_control.*control=PC-random' $f) segments_h_random=$(grep 'name=r4_random_summary family=h-random ' $f | grep -o 'segments=[0-9]*') cells=$(grep -c 'name=r4_random_cell' $f) mounts=$(grep -c 'name=r4_random_mount' $f)"; done
```

原样输出：

```
bing-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
bing-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
jia-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
jia-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
jia-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537
today summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537
yi-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
yi-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
yi-narrow-cfg-carry summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
yi-narrow-cfg summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=55 mounts=520
yi-narrow-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537
yi-slot summary=4 all_summary=1 pc_random=3 segments_h_random=segments=30 cells=64 mounts=537
```

- 每臂四族（H-随、实七-甲、实七-乙、实八）各一行汇总，H-随全一行，PC-随三行（a、b、c）；H-随 30 段、H-随全 68 段，每一臂都齐。compare 的 `r4_loss` 48 行 = 12 臂 × 4 族。
- **格没立齐的地方**（按修订第 19 条记 `fault_not_reached` 或 `not_constructible`，不立格）：-配置 8 条臂在实七-甲 上 6 个注入点有 3 个 `fault_not_reached=true`（每臂 3 行，`grep -c 'fault_not_reached=true'`），摘要行 `branches_not_constructible=3`；在实八 上 4 个崩溃状态全部 `not_constructible=segment_missing`（摘要行 `branches_not_constructible=4`）。所以 -配置 各臂的实八**只有主路径 2 格**，今天那一臂的实八是 6 格（主路径 2 格加崩溃状态 4 格）。成因是 -配置 各臂在崩溃恢复那一步拒了，之后的写序列与今天那一臂不同，今天那一臂上抽的崩溃状态在它们的写序列上找不到那一段。-槽 各臂两族都立齐了。

## 四、丢写（翻面观测第一样），按臂 × 族

命令：`grep 'name=r4_loss' research/results/e158-root-choice-repair-2026-09-27-r4-seg2-compare.out`。48 行里 -配置 8 臂逐臂相同、-槽 3 臂与今天逐臂相同，下面整行抄今天、乙-配置、乙-配置续、乙-槽各四行（其余 32 行在同一文件第 242–289 行）：

```
E7RESULT name=r4_loss arm=today family=h-random geometry=harness-4gib cells=43 lost_write_cells=43 q1_overwrite_cells=32 q2_rollback_cells=38 q3_lost_cells=6 q3c_lost_cells=40 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=today family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=10 q1_overwrite_cells=4 q2_rollback_cells=6 q3_lost_cells=4 q3c_lost_cells=10 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=today family=seventh-crash geometry=harness-4gib cells=5 lost_write_cells=1 q1_overwrite_cells=1 q2_rollback_cells=1 q3_lost_cells=0 q3c_lost_cells=1 q3_last_confirmed_cells=5
E7RESULT name=r4_loss arm=today family=eighth-crash geometry=harness-4gib cells=6 lost_write_cells=3 q1_overwrite_cells=2 q2_rollback_cells=2 q3_lost_cells=1 q3c_lost_cells=2 q3_last_confirmed_cells=5
E7RESULT name=r4_loss arm=yi-cfg family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=10
E7RESULT name=r4_loss arm=yi-cfg family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=3 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=3 q3c_lost_cells=0 q3_last_confirmed_cells=3
E7RESULT name=r4_loss arm=yi-cfg family=eighth-crash geometry=harness-4gib cells=2 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2
E7RESULT name=r4_loss arm=yi-cfg-carry family=h-random geometry=harness-4gib cells=37 lost_write_cells=6 q1_overwrite_cells=2 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=6 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=10
E7RESULT name=r4_loss arm=yi-cfg-carry family=seventh-crash geometry=harness-4gib cells=6 lost_write_cells=3 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=3 q3c_lost_cells=0 q3_last_confirmed_cells=3
E7RESULT name=r4_loss arm=yi-cfg-carry family=eighth-crash geometry=harness-4gib cells=2 lost_write_cells=0 q1_overwrite_cells=0 q2_rollback_cells=0 q3_lost_cells=0 q3c_lost_cells=0 q3_last_confirmed_cells=2
E7RESULT name=r4_loss arm=yi-slot family=h-random geometry=harness-4gib cells=43 lost_write_cells=43 q1_overwrite_cells=32 q2_rollback_cells=38 q3_lost_cells=6 q3c_lost_cells=40 q3_last_confirmed_cells=37
E7RESULT name=r4_loss arm=yi-slot family=seventh-fault geometry=harness-4gib cells=10 lost_write_cells=10 q1_overwrite_cells=4 q2_rollback_cells=6 q3_lost_cells=4 q3c_lost_cells=10 q3_last_confirmed_cells=6
E7RESULT name=r4_loss arm=yi-slot family=seventh-crash geometry=harness-4gib cells=5 lost_write_cells=1 q1_overwrite_cells=1 q2_rollback_cells=1 q3_lost_cells=0 q3c_lost_cells=1 q3_last_confirmed_cells=5
E7RESULT name=r4_loss arm=yi-slot family=eighth-crash geometry=harness-4gib cells=6 lost_write_cells=3 q1_overwrite_cells=2 q2_rollback_cells=2 q3_lost_cells=1 q3c_lost_cells=2 q3_last_confirmed_cells=5
```

**乙-配置 与 乙-配置续 单列**（这一段四族；「不算 ③c」「Q1 只算环内且不算 ③c」两栏取自各自产物的 `r4_random_summary` 行，只报数、不改判据）：

| 臂 | H-随 丢写 / 格 | 不算 ③c | Q1 只算环内且不算 ③c | 实七-甲 | 实七-乙 | 实八 |
|---|---|---|---|---|---|---|
| 乙-配置 | 6 / 37 | 2 | 1 | 0 / 10 | 3 / 6 | 0 / 2（4 个崩溃状态没立格） |
| 乙-配置续 | 6 / 37 | 2 | 1 | 0 / 10 | 3 / 6 | 0 / 2（同上） |
| 甲-配置、甲-配置续、丙-配置、丙-配置续、乙-窄读-配置、乙-窄读-配置续 | 各同上 | 2 | 1 | 0 / 10 | 3 / 6 | 0 / 2 |
| 今天、甲-槽、乙-槽、乙-窄读-槽 | 43 / 43 | 39 | 39 | 10 / 10 | 1 / 5 | 3 / 6 |

乙-配置 的两行汇总整行抄（`research/results/e158-root-choice-repair-2026-09-27-r4-seg2-yi-cfg.out` 第 1197、1711 行）：

```
E7RESULT name=r4_random_summary family=h-random geometry=harness-4gib arm=yi-cfg segments=30 segments_panicked=0 cells=37 lost_write_cells_q1_q2_q3=6 lost_write_cells_rollback_aware=6 lost_write_cells_without_q3c=2 lost_write_cells_with_q1_in_ring_only_and_without_q3c=1 q1_overwrite_cells=2 q1_in_ring_overwrite_cells=1 q1_out_of_ring_overwrite_cells=1 q1_overwrite_cells_first_changed_while_the_newest=0 q2_rollback_cells=0 q2_rollback_cells_rollback_aware=0 q3_lost_cells=0 q3_lost_cells_rollback_aware=0 q3c_lost_cells=6 q3_last_confirmed_cells=37 q0_destinations={"left_as_it_was": 37} lost_write_cells_by_destination={"left_as_it_was": 6} tested_mounts=158 tested_classes={"K0": 121, "K3": 37} s4_mismatches=0 f6_checked=37 f6_failures=0 branches_not_constructible=0 
E7RESULT name=r4_random_summary family=seventh-crash geometry=harness-4gib arm=yi-cfg segments=1 segments_panicked=0 cells=6 lost_write_cells_q1_q2_q3=3 lost_write_cells_rollback_aware=3 lost_write_cells_without_q3c=3 lost_write_cells_with_q1_in_ring_only_and_without_q3c=3 q1_overwrite_cells=0 q1_in_ring_overwrite_cells=0 q1_out_of_ring_overwrite_cells=0 q1_overwrite_cells_first_changed_while_the_newest=0 q2_rollback_cells=0 q2_rollback_cells_rollback_aware=0 q3_lost_cells=3 q3_lost_cells_rollback_aware=3 q3c_lost_cells=0 q3_last_confirmed_cells=3 q0_destinations={"crash": 4, "left_as_it_was": 2} lost_write_cells_by_destination={"crash": 3} tested_mounts=10 tested_classes={"K0": 8, "K3": 2} s4_mismatches=0 f6_checked=2 f6_failures=0 branches_not_constructible=0 device_reproduces=false
```

### 4.1　实七-乙 那 3 格（乙-配置），与今天那一臂同一格并排

乙-配置 `crash0` 格与被测挂载（`…-r4-seg2-yi-cfg.out` 第 1701、1700 行），今天那一臂同一格（`…-r4-seg2-today.out` 第 1767 行）：

```
E7RESULT name=r4_random_cell family=seventh-crash geometry=harness-4gib seed=base+16 arm=yi-cfg cell=crash0:crash@0 kind=crash order=0 protected=none protected_root=none protected_rollback_aware=0 q1_in_ring_overwritten=0 q1_out_of_ring_overwritten=0 q1_landing_overwritten=written_after_the_cell overwrite_cell=false q1_first_change=none q2_rollback=false q2_rollback_rollback_aware=false crash_read_back=last_confirmed crash_read_back_rollback_aware=last_confirmed crash_read_back_root=0:0 crash_read_back_content=0 after_mount_read_back=failed:content_differs_from_the_ledger after_mount_read_back_rollback_aware=failed:content_differs_from_the_ledger after_mount_read_back_root=2:4 after_mount_read_back_content=1 q3_lost=true q3_lost_rollback_aware=true q3_last_confirmed=false q3c_tail=na q3c_lost=false lost_write_cell=true lost_write_cell_rollback_aware=true q0_destination=na
E7RESULT name=r4_random_mount family=seventh-crash geometry=harness-4gib seed=base+16 step=crash0 attempt=crash0 arm=yi-cfg operation=after_crash phase=before_first_crash_recovery duration=W hidden_ranges=0 reads_over_hidden_ranges=0 base_fingerprint=97f3fd5d54dd1462 a_class=K0 a_error=none a_writes=29 a_reads=393705 a_read_bytes=1611841536 a_waits=0 a_chosen=0:0 a_effective=0:0 a_abandoned_roots_unreadable=0 a_isolated=0 a_arm_observation=judged:true,newer:false,tail:0,last:none,undecidable:false,unreadable_slots:0,rounds:0,rebuilt:false,table_unreadable:false,table_rereads:0 a_reads_before_first_wait=none a_arm_extras=1:false device_chosen=0:0 device_effective=0:0 device_n_cfg=false device_c_witness=0 device_c_e=none device_undecidable=false device_n_slot=false device_unreadable_root_slots=0 truth_newer=false truth_orders_disagree=false truth_by_records_only=false last_confirmed=0 last_confirmed_rollback_aware=0 e_content=0 device_e_content=0 q2_rollback=false q2_rollback_rollback_aware=false f6=na s4_findings=[]
E7RESULT name=r4_random_cell family=seventh-crash geometry=harness-4gib seed=base+16 arm=today cell=crash0:crash@0 kind=crash order=0 protected=none protected_root=none protected_rollback_aware=0 q1_in_ring_overwritten=0 q1_out_of_ring_overwritten=0 q1_landing_overwritten=written_after_the_cell overwrite_cell=false q1_first_change=none q2_rollback=false q2_rollback_rollback_aware=false crash_read_back=last_confirmed crash_read_back_rollback_aware=last_confirmed crash_read_back_root=0:0 crash_read_back_content=0 after_mount_read_back=last_confirmed after_mount_read_back_rollback_aware=last_confirmed after_mount_read_back_root=2:4 after_mount_read_back_content=0 q3_lost=false q3_lost_rollback_aware=false q3_last_confirmed=true q3c_tail=na q3c_lost=false lost_write_cell=false lost_write_cell_rollback_aware=false q0_destination=na
```

两臂落到的根都是 `2:4`；今天那一臂的账记它是内容号 0（`after_mount_read_back_content=0`、`last_confirmed`），乙-配置 的账记它是内容号 1（`content_differs_from_the_ledger`）。被测挂载起始镜像指纹 `97f3fd5d54dd1462`、选到的根 `0:0`、`last_confirmed=0`、`e_content=0` 两臂都一样（今天那一臂第 1766 行）。这一格在第一次崩溃恢复之前（`phase=before_first_crash_recovery`，`truth_newer=false`），C554 那一形还没出现。`crash1`、`crash2` 同形（第 1704、1707 行）。上面第一节第 2 条的推断就是从这里来的：崩溃分支按主路径的账查根的键（**推的**）。

### 4.2　H-随 上 -配置 各臂那 6 格

6 格的 `q3c_lost=true` 全部为真（③c 尾巴读回 `failed:NoValidRoot` 5 格、`failed:UnitUnreadable` 1 格），`tail_read_back` 与 `q2_rollback` 在 6 格上都是 `last_confirmed` 与 `false`。其中不算 ③c 也丢写的 2 格（`…-r4-seg2-yi-cfg.out` 第 414、932 行）：

```
E7RESULT name=r4_random_cell family=h-random geometry=harness-4gib seed=base+23 arm=yi-cfg cell=main:crash_recovery@3 kind=crash_recovery order=3 protected=2 protected_root=1:4 protected_rollback_aware=2 q1_in_ring_overwritten=0 q1_out_of_ring_overwritten=4 q1_landing_overwritten=written_after_the_cell overwrite_cell=true q1_first_change=order30:later_confirmations11:in_ringfalse q2_rollback=false q2_rollback_rollback_aware=false tail_read_back=last_confirmed tail_read_back_rollback_aware=last_confirmed tail_read_back_root=6:31 tail_read_back_content=15 q3_lost=false q3_lost_rollback_aware=false q3_last_confirmed=true q3c_tail=failed:NoValidRoot q3c_lost=true lost_write_cell=true lost_write_cell_rollback_aware=true q0_destination=left_as_it_was
E7RESULT name=r4_random_cell family=h-random geometry=harness-4gib seed=base+79 arm=yi-cfg cell=main:crash_recovery@6 kind=crash_recovery order=6 protected=4 protected_root=1:6 protected_rollback_aware=4 q1_in_ring_overwritten=4 q1_out_of_ring_overwritten=0 q1_landing_overwritten=written_after_the_cell overwrite_cell=true q1_first_change=order25:later_confirmations5:in_ringtrue q2_rollback=false q2_rollback_rollback_aware=false tail_read_back=last_confirmed tail_read_back_rollback_aware=last_confirmed tail_read_back_root=5:25 tail_read_back_content=14 q3_lost=false q3_lost_rollback_aware=false q3_last_confirmed=true q3c_tail=failed:UnitUnreadable q3c_lost=true lost_write_cell=true lost_write_cell_rollback_aware=true q0_destination=left_as_it_was
```

- 第 414 行（种子基 + 23）：Q1 那一版在历史末尾「已不在根环里」、被改 4 份，第一次被改在第 30 步，那时它之后已经有 11 笔确认。
- 第 932 行（种子基 + 79）：Q1「还在根环里」被改 4 份，第一次被改在第 25 步，那时它之后已经有 5 笔确认（`q1_first_change=order25:later_confirmations5:in_ringtrue`）。
- 按登记字面，这两格都算丢写。上一个执行员担心的正是这种情况：那一版早就被后来的确认取代了（修订第 23 条）。这里只报数，算不算交主 agent。

### 4.3　③c：今天那一臂的原样行（派发要求整行抄）

`…-r4-seg2-today.out` 第 1241 行：

```
E7RESULT name=r4_random_summary family=h-random geometry=harness-4gib arm=today segments=30 segments_panicked=0 cells=43 lost_write_cells_q1_q2_q3=43 lost_write_cells_rollback_aware=43 lost_write_cells_without_q3c=39 lost_write_cells_with_q1_in_ring_only_and_without_q3c=39 q1_overwrite_cells=32 q1_in_ring_overwrite_cells=17 q1_out_of_ring_overwrite_cells=15 q1_overwrite_cells_first_changed_while_the_newest=4 q2_rollback_cells=38 q2_rollback_cells_rollback_aware=38 q3_lost_cells=6 q3_lost_cells_rollback_aware=5 q3c_lost_cells=40 q3_last_confirmed_cells=37 q0_destinations={"chosen_or_applied": 9, "in_ring_and_abandoned": 34} lost_write_cells_by_destination={"chosen_or_applied": 9, "in_ring_and_abandoned": 34} tested_mounts=164 tested_classes={"K0": 164} s4_mismatches=0 f6_checked=43 f6_failures=0 branches_not_constructible=0 
```

今天那一臂 H-随 43 格里 ③c 读失败 40 格（`q3c_lost_cells=40`）。不算 ③c 是 39 格，Q1 只算环内且不算 ③c 也是 39 格，Q2 回滚 38 格。-槽 三臂与这一行逐栏相同。-配置 各臂 ③c 读失败 6 / 37。照登记判，丢写格的算法没改。

## 五、多拒（Q4）与多读（Q5），这一段五族

命令：`grep -c 'name=r4_q4 .*extra_refusals=[1-9]' …-r4-seg2-compare.out`、`grep -c 'refusals_with_truth_newer_false=[1-9]' …-r4-seg2-compare.out`，原样输出：
```
0
0
```

- **多拒 = 0**：55 行 `r4_q4` 没有一行 `extra_refusals` 大于 0；40 行 `r4_q4_after_divergence`（修订第 25 条①）没有一行 `refusals_with_truth_newer_false` 大于 0。H-随全（登记 5.5 认定第 9 项立的合法状态族）在每条臂上都是 0。
- 拦下（N_真 为真时拒）：-配置 各臂 H-随 `W=30`、H-随全 `W=18`、实七-甲 / 实七-乙 / 实八 各 `W=1`；-槽 各臂都是空。H-随全 上有 18 次拦下，说明那 68 段里也有「崩溃恢复抛弃根」、而且确实有更新的根（`r4_random_all_summary` 里乙-配置 是 `tested_classes={"K0": 306, "K3": 18}`）。

乙-配置 与 乙-配置续 的 Q4 整行（compare 文件里按臂名 grep）：

```
E7RESULT name=r4_q4 arm=yi-cfg family=eighth-crash@harness-4gib compared_tested_mounts=2 extra_refusals=0 intercepted="W=1"
E7RESULT name=r4_q4 arm=yi-cfg family=h-random-all@harness-4gib compared_tested_mounts=293 extra_refusals=0 intercepted="W=18"
E7RESULT name=r4_q4 arm=yi-cfg family=h-random@harness-4gib compared_tested_mounts=64 extra_refusals=0 intercepted="W=30"
E7RESULT name=r4_q4 arm=yi-cfg family=seventh-crash@harness-4gib compared_tested_mounts=4 extra_refusals=0 intercepted="W=1"
E7RESULT name=r4_q4 arm=yi-cfg family=seventh-fault@harness-4gib compared_tested_mounts=1 extra_refusals=0 intercepted="W=1"
E7RESULT name=r4_q4 arm=yi-cfg-carry family=eighth-crash@harness-4gib compared_tested_mounts=2 extra_refusals=0 intercepted="W=1"
E7RESULT name=r4_q4 arm=yi-cfg-carry family=h-random-all@harness-4gib compared_tested_mounts=293 extra_refusals=0 intercepted="W=18"
E7RESULT name=r4_q4 arm=yi-cfg-carry family=h-random@harness-4gib compared_tested_mounts=64 extra_refusals=0 intercepted="W=30"
E7RESULT name=r4_q4 arm=yi-cfg-carry family=seventh-crash@harness-4gib compared_tested_mounts=4 extra_refusals=0 intercepted="W=1"
E7RESULT name=r4_q4 arm=yi-cfg-carry family=seventh-fault@harness-4gib compared_tested_mounts=1 extra_refusals=0 intercepted="W=1"
```

**Q5-同成**（判「选哪个」用；臂减今天、每格读调用数）。命令 `grep 'name=r4_q5 .*column=both_mounted' …-compare.out`，按臂摘（整行在 compare 文件里）：

| 臂 | H-随（格，合计，每格） | H-随全 | 实七-乙 | 实八 | 实七-甲 |
|---|---|---|---|---|---|
| 甲-配置、乙-配置、乙-窄读-配置 | 34 格，+136，每格 +4 | 275 格，+1100，每格 +4 | 3 格：甲 +12（每格 4）；乙、乙-窄读 +11（最小 3、最大 4） | 1 格，+4 | 没有同成格 |
| 甲-配置续、乙-配置续、乙-窄读-配置续 | 34 格，+272，每格 +8 | 275 格，+2200，每格 +8 | 3 格：甲 +24；乙、乙-窄读 +23（最小 7、最大 8） | 1 格，+8 | 没有同成格 |
| 丙-配置 | 34 格，−714，每格 −21 | 275 格，−5775，每格 −21 | 3 格，−63 | 1 格，−21 | 没有同成格 |
| 丙-配置续 | 34 格，−578，每格 −17 | 275 格，−4675，每格 −17 | 3 格，−51 | 1 格，−17 | 没有同成格 |
| 甲-槽 | 164 格，0 | 326 格，0 | 9 格，0 | 10 格，0 | 23 格，0 |
| 乙-槽、乙-窄读-槽 | 164 格，−43（最小 −1、最大 0） | 326 格，−20 | 9 格，−2 | 10 格，−3 | 23 格，−5 |

- 甲 / 乙-配置 +4、甲 / 乙-配置续 +8 与 5.4 PC-多读 (ii) 的点值一样（那一句的判定在第一段 L0 上，不在这一段）。
- 实七-乙 上乙比甲少 1 次读的那一格，以及 -槽 乙 各臂每格 −1 或 0，都只报数、没查成因。第一段认定第 3 项里那个「差 1」是执行员推的「读缓存去重」，主 agent 定了不采信、不查。
- 丙 各臂在同成格上比今天**少**读（每格 −21 / −17），只报数。
- Q5-拒（附带）：`column=refused_minus_today` 乙-配置 在 H-随 30 格上合计 +11781473（每格 +392569 到 +392852），甲-配置 −14858，乙-窄读-配置 −14353，丙-配置 +11781682（整行在 compare 文件 `name=r4_q5 … column=refused_minus_today`）。

## 六、PC-随、S5a / S5b / S9 / S7、F19

PC-随：每臂 3 句（a、b、c），12 臂 36 行全 `verdict=pass`。compare 里 -槽 三臂「到那一步为止与今天逐步相同」9 句全 `pass`（compare 第 102–104、192–194、206–208 行）。今天那一臂三行（`…-r4-seg2-today.out` 第 92、1218、1235 行）：

```
E7RESULT name=r4_positive_control arm=today control=PC-random of=a seed=base+2 verdict=pass sentences=2 sentence_today_checker_first_abandoned_red_step_and_root=pass sentence_measure_ok_on_an_older_version_implies_q2_rollback=pass missed_routes=none crash_recovery_order=4 first_abandoned_red=order5:[(1, 4)] mount_class=K0 mount_member=none mount_writes=67 q1_in_ring=0 q2_rollback=true
E7RESULT name=r4_positive_control arm=today control=PC-random of=b seed=shortest verdict=pass sentences=3 sentence_today_checker_first_abandoned_red_step_and_root=pass sentence_measure_checker_names_the_protected_root_implies_q1_in_ring=pass sentence_measure_ok_on_an_older_version_implies_q2_rollback=pass missed_routes=none crash_recovery_order=3 first_abandoned_red=order3:[(1, 3)] mount_class=K0 mount_member=none mount_writes=24 q1_in_ring=10 q2_rollback=true
E7RESULT name=r4_positive_control arm=today control=PC-random of=c seed=row43 verdict=pass sentences=3 sentence_today_checker_first_abandoned_red_step_and_root=pass sentence_measure_checker_names_the_protected_root_implies_q1_in_ring=pass sentence_measure_ok_on_an_older_version_implies_q2_rollback=pass missed_routes=none crash_recovery_order=3 first_abandoned_red=order4:[(1, 5)] mount_class=K0 mount_member=none mount_writes=46 q1_in_ring=2 q2_rollback=true
```

乙-配置 一行（`…-r4-seg2-yi-cfg.out`，of=a）：

```
E7RESULT name=r4_positive_control arm=yi-cfg control=PC-random of=a seed=base+2 verdict=pass sentences=1 sentence_arm_refuses_at_the_crash_recovery_step_before_any_write=pass missed_routes=none crash_recovery_order=4 first_abandoned_red=none mount_class=K3 mount_member=R3NewerRootWitnessedByTheSystemConfiguration mount_writes=0 q1_in_ring=0 q2_rollback=false
```

停机与对拍（`…-r4-seg2-today.out` 第 50–55、57、1242、1639、1725、1726、1778、1779、1833、1239 行）：

```
E7RESULT name=r4_segment_two_check arm=today item="7.2 H-随 (a) 种子个数、最小、最大" verdict=pass stops_the_family=true detail="count=28 minimum=Some(7463871032432355115) maximum=Some(7463871032432355205)"
E7RESULT name=r4_segment_two_check arm=today item="7.2 H-随 段数 28 + 1 + 1" verdict=pass stops_the_family=true detail="segments=30"
E7RESULT name=r4_segment_two_check arm=today item="S9 (a) 种子与调查员报告第 61 行逐个相等" verdict=pass stops_the_family=true detail="listed=28 local=28"
E7RESULT name=r4_segment_two_check arm=today item="S9 (b) 最短复现 3 步与调查员报告第 298–301 行逐项相等" verdict=pass stops_the_family=true detail="header=true steps=true"
E7RESULT name=r4_segment_two_check arm=today item="S9 (c) 第 43 行那条用例 8 步与源码逐项相等" verdict=pass stops_the_family=true detail="matched=12/12 operation_lines=8"
E7RESULT name=r4_segment_two_check arm=today item="5.6 快档首种子、段数、每段步数与崩溃注入快档步数、抽法（源码文本回比；不等只报，照本地常量跑）" verdict=pass stops_the_family=false detail="random_history_source=true crash_injection_source=true seeds=96 operations=30 crash_operations=24 crash_points=4"
E7RESULT name=r4_segment_two_executor_agreement family=h-random compared_steps=881 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_segment_two_executor_agreement family=h-random-all compared_steps=2108 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_segment_two_executor_agreement family=seventh-fault compared_steps=31 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_stop_check clause=S5 family=seventh-fault harness_reproduces=true device_reproduces=true verdict=reproduced
E7RESULT name=r4_segment_two_executor_agreement family=seventh-crash compared_steps=25 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_stop_check clause=S5 family=seventh-crash harness_reproduces=false device_reproduces=false verdict=S5b_not_reproduced_on_the_base
E7RESULT name=r4_segment_two_executor_agreement family=eighth-crash compared_steps=25 mismatches=0 verdict=pass detail=[]
E7RESULT name=r4_stop_check clause=S5 family=eighth-crash harness_reproduces=true device_reproduces=true verdict=reproduced
E7RESULT name=r4_failure_clause clause=F19 family=h-random arm=today segments_without_an_abandoned_red=0 labels=[] verdict=not_triggered
```

- S9：种子、写死历史、源码回比 6 项全 `pass`；执行器逐步对拍五族共 3070 步，`mismatches=0`（881 + 2108 + 31 + 25 + 25），所以 S7、S9 都没停，H-随 与 H-随全 照跑。
- S5a 没碰上。实七-甲、实八两边都复现（`verdict=reproduced`）。**S5b 碰上了：实七-乙 harness 与装置两边都复现不出**，照登记不停、全部臂照跑，这一段标「底座上复现不出」。复现不出的只是那个签名（崩溃状态读回报 `MappingStillUnreadable`）：今天那一臂主路径第 4 步那一格（`main:crash_recovery@4`）被藏的根仍是 `in_ring_and_abandoned`，丢写 1 格；其余 4 格是崩溃状态格，都读回 `last_confirmed`。
- F19 没触发（`segments_without_an_abandoned_red=0`）。

## 七、判决行（4c）

`grep -n 'name=verdict' <产物>` 对 13 份各数一遍，都是 0（`grep -c 'name=verdict' research/results/e158-root-choice-repair-2026-09-27-r4-seg2-*.out` 13 行全是 `:0`）。这个装置的判决写在各结果行的 `verdict=` 字段里，所以另把全部 `verdict=` 取值数了一遍（`grep -o 'verdict=[A-Za-z0-9_]*' <产物> | sort | uniq -c`）：11 份臂产物全是 `pass`（每份 59 个）；今天那一臂是 `pass` 64、`reproduced` 2、`not_triggered` 1、`S5b_not_reproduced_on_the_base` 1；compare 是 `pass` 42、`inference_holds` 1、`not_constructible` 12。另数了 `=fail` 的次数，13 份都是 0。下面逐个点名没过的：

| 产物:行 | 字段 = 取值 | 为什么 | 登记预期内吗 |
|---|---|---|---|
| `…-r4-seg2-today.out:1778` | `verdict=S5b_not_reproduced_on_the_base` | 实七-乙 harness 与装置都复现不出 r3 登记第 604 行的签名 | 预期内的分支（11.2 S5b：不停，标「底座上复现不出」）；试跑时已经这样 |
| `…-r4-seg2-compare.out:209–219`（11 行） | `verdict=not_constructible`（PC-多拒 `today_ok_on_the_cell`，[今]） | 前提是第四段的 L 族；compare 这次只吃了第二段的前缀，n1 = 1、2、3 都找不到格 | 第五段认定第 3 项同形：按 V1 ⑤ 不作废，跨段以五段一起跑的 compare 为准 |
| `…-r4-seg2-compare.out:220` | `verdict=not_constructible`（PC-22 `an_m_with_ok_zero_count_zero_isolation`，[今]，route=F2） | 同上，L5 在第四段 | 同上 |
| 8 份 -配置 产物 `:1663`、`:1751` | `branches_not_constructible=3`、`=4` | 实七-甲 3 个注入点在臂的那一步里没到（`fault_not_reached`）；实八 4 个崩溃状态找不到那一段（`segment_missing`） | 修订第 19 条写了这两种记法、不立格；登记没预言个数。拿不准算不算「失败类计数」，所以点名 |

`r4_void` 84 行全是 `voided=false`（`grep 'name=r4_void' …-compare.out | grep -o 'voided=[a-z]*' | sort | uniq -c` 输出 `84 voided=false`）。`r4_missed_arm_and_today_sentences count=12` 就是上表 12 句。其余名字表示违例 / 不匹配 / 失败的计数（`mismatches`、`s4_mismatches`、`f6_failures`、`segments_panicked`、`extra_refusals`、`refusals_with_truth_newer_false`）在 13 份里都是 0（`grep -Eo '(mismatches|s4_mismatches|f6_failures|segments_panicked)=[1-9][0-9]*'` 零命中）。

## 八、岔路表（`research/prompts/c554-fix-forks.md` 第 1 行）

**这一段够判了什么、还差什么**（其他段的数取自派发提示与各段报告，我没有复核）：

| 岔路单第 1 行要的 | 这一段的读数（指到产物里算出它的命令） | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 |
|---|---|---|---|
| 丢写：乙-配置 | H-随 6 / 37，实七-甲 0 / 10，实七-乙 3 / 6，实八 0 / 2（`grep 'name=r4_loss arm=yi-cfg ' …-r4-seg2-compare.out`）。派发提示给的其它段：H1g 16 / 32 | 按登记字面，**乙-配置 已在三族上丢写 > 0，岔路单写的是出局**；实七-乙 那 3 格与 H-随 里只由 ③c 或很晚的 Q1 构成的格，怎么读交主 agent（第一节） | 这一段四族都跑完了，登记里没有剩下的量。能让它翻面的只有：主 agent 判实七-乙 那 3 格是装置记账的毛病（修了是新一次跑），或者改 ③c / Q1 的读法。两样都不是这次跑的量 |
| 丢写：乙-配置续 | 与乙-配置逐族相同：6 / 37、0 / 10、3 / 6、0 / 2（`grep 'name=r4_loss arm=yi-cfg-carry ' …`）。派发给的 H1g 是 0 | 按字面，H-随、实七-乙 上都 > 0。出局与否同上一行，交主 agent | 同上 |
| 丢写：甲-配置、甲-配置续、丙-配置、丙-配置续、乙-窄读 两条 -配置 | 与乙-配置 逐族相同 | 同上。乙-窄读 两臂在第一段已判「不可判（PC-N-盖 没过）」 | 同上 |
| 丢写：今天、甲-槽、乙-槽、乙-窄读-槽 | 43 / 43、10 / 10、1 / 5、3 / 6（`grep 'name=r4_loss arm=today ' …` 等） | 已够判：四族都 > 0，出局 | 不能：丢写格只会多，剩下的量不会让它变成 0 |
| 实八「根还在环里、单元被复用」那一格 | 今天那一臂主路径 `in_ring_and_abandoned` 1 格、丢写 3 / 6；-配置 各臂只有主路径 2 格（4 个崩溃状态 `segment_missing`） | -配置 各臂在实八上只量到了主路径，崩溃状态格造不出来（它们在崩溃恢复那一步就拒了，写序列和今天不一样）。这是登记的构造方式决定的 | 登记里没有别的办法补这 4 格 |
| 多拒 | 这一段五族（含 H-随全 68 段）每臂 0（第五节那两条 grep 各输出 0） | 这一段够判。L 族与第 22 条那几格在第四段 | 跨段的 V1 以五段一起跑的 compare 为准（第四段认定第 4 项）。这一段的 V1 没有作废任何量（84 行全 `voided=false`） |
| 多读（同成） | 第五节那张表 | 这一段够判。乙-配置 +4、乙-配置续 +8，丙 −21 / −17 | 同上 |
| 第 22 条那一格 | 不在这一段 | 第四段 | — |
| 判据那一样今天有没有 | 不在这一段 | 第一段 S0 | — |
| 阳性对照 PC-随 | 12 臂 36 句 + compare 9 句全 `pass` | 已够判 | — |
| 停机 S5a / S5b / S9 | S5a 没碰上；S5b 碰上了（实七-乙，不停）；S9 全过 | 已够判 | — |

## 九、门禁（登记给 experiment-runner 的 14 道；这次不写实验页，读实验页的那几道最后跑）

| 阶段 | 退出码 | 原样首行（`grep -m1 -E '✓|✗'`），或说明 |
|---|---|---|
| 33 | 1 | `✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次…`：只点名第 659 行（命中 2 次），就是派发提示说的那一行，替换行见第十节 |
| 52 | 1 | `✗ 段序列登记表与 E142 产物的 name=segments 行（或第二条流与钉它的用例）对不上，共 2 处`：E142 / `kb/layout/01-first-txn.md`，不是这一段动的文件，照写没修 |
| 80 | 0 | `✓ 152 个实验二进制各自至少有一条绝对值断言…` |
| 96 | 0 | `✓ 实验源码纪律：扫了 152 个文件（读不动的 0 个）；C59 查了 696 处…` |
| 27 | 1 | `✗ 格式常量在 kb 与实验源码之间对不上`：点名 `e142_…rs:106` 与 `e157_…rs:74`，不是这一段的文件 |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）` |
| 40 | 1 | `✗ 有实验产物没被 experiments.md 点名…`：列出的里面有这一段的 13 份（`grep -c r4-seg2` = 13），另有 r2 / r3 / r4-seg1 等别人的。这次不写实验页，是预期内的 |
| 69 | 1 | `✗ 这些地方把 /tmp 下的东西当依据引用了`：点名 `m2-kb-writeback-batch2/3-scribe-report.md`，不是这一段的文件 |
| 75 | 0 | `✓ 决策与实验双向登记对得上、回看不过期（查了实验页 159 个、决策 28 条；…` |
| 84 | 0 | `✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）` |
| 85 | 0 | `✓ 点了产物的实验都写了复跑命令（点了产物的 15 页里判了 15 页）` |
| 86 | 1 | `✗ research 里有这些实验号的东西，kb/experiments/ 里却没有正文：E161 research/prompts/e161-preregistration.md`：不是这一段的文件；出路里「删掉 research 下那些文件」那一句没照做 |
| 88 | 77 | `! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判`：没判 |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）；…` |

日志在草稿目录 `gate-<阶段>.log`。

## 十、`crates/mutations.tsv` 第 659 行的替换行

- 原锚点 `        new_floor,` 在主工作区 bin 上命中 2 次：第 2079 行（`attempt_mount_writable_then_raise_floor` 里传给 `raise_rollback_floor` 的那个），和第 19249 行（第二段装置里的 `raise_rollback_floor` 调用，缩进 16 格，里面含 8 格那一段）。
- 替换行：原文 `        &mut current,\n        new_floor,\n        ShadowLedger::On,`，替换文 `        &mut current,\n        CheckpointTxg(new_floor.0 + 1_000_000),\n        ShadowLedger::On,`。其余四段（变异名、文件、cargo 参数、必须红的用例 `mount_writable_then_raise_floor_reaches_the_injected_fault`）照原行。草稿 `/tmp/claude-1000/e158-r4-seg2-run/mutations-replacements.tsv`（第 1 行是注释，第 2 行是替换行），sha256 `9d24ef3769259b2e196d60a7c25bcfde91e69ae786db2452d0886b4b737ccdcd`。
- 命中次数（把 `\n` 换成换行后做子串计数，同门禁 33 号第 74 行）：主工作区 bin（`06d75b96…`）1 次，臂副本 bin（`12b82f49…`）1 次。
- 证红（草稿副本里跑 `cargo test --offline -p singlefs-harness --bin e158_root_choice_repair mount_writable_then_raise_floor_reaches_the_injected_fault`，经 `capped.sh 4` 与 `run-with-memory-cap.sh 10G`）：
  - **主工作区此刻的 crates 副本：替换之前基线就红了**。`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 93 filtered out; finished in 4.70s`，panic 在第 22696 行：`H1c 抛弃步造出的节点里应当至少有一个有 op1 第三种变体可跑（有余量、有可读被抛弃根）`。施加替换之后也是同一处红（4.31s）。所以在今天的主工作区上，这条变异「仍红」什么也证明不了。这条用例在主工作区上为什么红，没查。已知的只有一样：主工作区的 bin 已经和臂副本不同（第二节，不是我改的）；crates 的其余部分和快照差在哪，没比。
  - **快照副本（`arms/today`，bin `12b82f49…`）**：基线 `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out; finished in 4.72s`；替换之后 `FAILED`，panic 在第 22666 行：`探测出的 floor 目标不注入任何故障应当成功，得到 Some("RollbackFloorAboveCeiling")`。之后已还原（`cmp` 过）。
- 主表没动。点名的用例要先在主工作区上回绿，门禁 59 号跑这一行才算数。这件交主 agent。

## 十一、没做什么

- 没写实验页、索引行、`experiments-history.md`；没改 `replay.sh`（派发要求）。这 13 份新产物都还没有实验页点名（门禁 40 号红在它们身上）。`replay.sh` 以后要复跑这一段：今天那一臂先跑，其余臂要 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT` 与 `SINGLEFS_E158_INVESTIGATOR_REPORT` 两个环境变量（照 `run_arm.sh`）。
- 没改登记（产物之前没发现要补的；登记修订那一段一个字没动）、没改 bin、没改 crates、没动 `crates/mutations.tsv`。
- 实七-乙 那 3 格是不是装置记账的毛病，只读了代码、并排了两臂同一格的产物行，**没有改装置去量**（改装置是新一次跑，不归这一段）。
- 用的线程上限与派发写的不同：非今天臂 `capped.sh 1`，四条并行（第二节）。没有量过换线程数会不会改变产物；装置是单线程的，只有今天那一臂调 harness 的注入函数，它用的是 `capped.sh 4`。
- 没跑 `r4-compare` 的五段合跑（第四段认定第 4 项归写页的执行员）；没判结论能不能推翻或确立决策；没跑门禁全量、重型测试；没提交。
- 门禁 27、40、52、69、86 的红不在这一段的文件上（40 号里这一段那 13 份除外，原因见上），没修。

## 十二、草稿与产物

- 产物 13 份的 sha256：草稿 `products.sha256`（13 行）。
- 草稿目录 `/tmp/claude-1000/e158-r4-seg2-run/`：`run_arm.sh`、`anchor_check*.sh`、各臂 `.err`（都是空的）、`progress.md`、`summary-table.txt`、门禁日志、`anchor*-{baseline,mutated}.log`、`mutations-replacements.tsv`。为证锚点建的两份编译副本 `mutcopy`、`mutcopy2` 已删。
- `/tmp/claude-1000/e158-r4-seg2/arms/` 下那 12 份臂副本是上一个执行员建的，这一段只读、只执行，**没删**：写页的执行员跑五段合并的 `r4-compare`、以及以后复跑，都还要用。
