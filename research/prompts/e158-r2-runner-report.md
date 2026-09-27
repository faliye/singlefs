# E158（择根与修复四岔路） 第 2 次跑第一段：执行员报告（2026-09-26，UTC 15:4x 写）

## 一、结论

- 问题单第一节前提 1、2、3 在今天的代码上都成立（S0 逐条重读 `crates/singlefs-core` 行号没漂；装置读数见第四节）。
- H1d（崩溃恢复把最新的根当读不出、抛弃它、取号，这次可写挂载在写行根落盘之前断；故障撤掉、再挂载）在八份副本上各跑 800 格：今天那一臂与岔路单第 2 行六臂（甲-txg、乙F-留环、乙-留环、乙-只配置、丁-留环、丁-只配置）三格逐格相同——① 128 格恢复落到的根引用的单元被覆盖（全在 n1 = 0，即被藏的就是首个文件那条根）、② 再挂载 K0 680 / K3 120（`MountError::Recovery`，冷启动读回 `MappingStillUnreadable`）、③ 读回最后确认那一版 664 格；「挂载做完」那 16 格被藏那一版在每一臂上都丢（12 格读回前一版内容、4 格读回没有文件）。A1（岔路单第 1 行 (a)）只在 2 格上不同：那 2 格的再挂载被拒（`AbandonedRootLedgerUnreadable`），数据在那之前已经丢了。(b) 原形、(b) 含链、(c) × 两个指称：除那 2 格同今天，那 2 格上 (b) 重建不重建得出没算。
- 推翻这句的观测：换几何（S=4 / 16、默认环长）、藏 k ≥ 2 条根、关闭 U、或枚举层 0 的非前缀崩溃状态之后，某条岔路单第 2 行的臂在 H1d 上三格与今天不同。
- PC1-a / b / c（今天）与 A1 的 PC1-a 都按登记结局出来；Q1-0 在今天那一臂上 87 次抛弃步挂载里 56 次造出被抛弃根，最少 2 个故障。
- 岔路单第 1 行、第 2 行都没够判（第九节岔路表）。

## 二、现场核对（接手）

- 上一任建的 `snapshot/`（14:03 UTC）与 `probe/` 已删；它们之后实八改了 harness 四个文件：

```
$ diff <(grep -v '/target/' snapshot-crates-sha256.txt) now-crates-sha256.txt   # 摘 4 个不同文件名
crates/singlefs-harness/src/fault_injection.rs、history.rs、model.rs、model_comparison.rs（core、format、checker 无差）
```

  按 S4「全部副本从同一次快照派生」重取 `snapshot2`：时刻 `2026-09-26 23:28:50 JST`，`HEAD` = `73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321`，`git status --porcelain crates/` 91 行（`snapshot2-status.txt`），`crates/` 全部文件 sha256 汇总的 sha256 = `574f31c7a55d4b9495a767e20444085030c269a682c6eb67db51f054290ad98e`，取时与工作树逐文件相同（`same-as-worktree`）。
- 开工时装置单测：`test result: FAILED. 42 passed; 1 failed`（红的就是 `mount_writable_trajectory_distinguishes_persistent_from_transient_faults`，与问题单第四节一致）。
- 负载：开工与各次编译前 `ps` 只见别的会话的 `claude` 进程与一次别人的 `gate.sh --staged`（pid 1821532，14:22 UTC），没有 qemu / vm-bench / e152 / fio；没等锁。线程上限 10 全程经 `research/scripts/capped.sh 10`，跑编译出的代码全程经 `run-with-memory-cap.sh 8G`（取 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认），没有 250–254 的结局。

## 三、停机条款

| 条款 | 结果 |
|---|---|
| S0 | `first_txg_of_new_instance`（`mount.rs:599`–`617`）仍不读 F；可写挂载仍按最新根的实例表判抛弃（`rebuilt_allocator` `:815`）、账读不出只计数（`:714` `unreadable += 1`）；`MountSpaceAdmission` 仍四成员（`:370`）、写行之后仍推（`:2744`）。与登记第一节一致，不停 |
| S1 | 第三节表逐行重读：`choose_root` `recovery.rs:776` 键 `(checkpoint_txg, instance)`；`next_counter` `mount.rs:2864`；`write_acquired_instance` `transaction.rs:654` 写 tail 0、F 整池生效值；`persist_the_root_then_rotate_the_system_configuration` `:981` 先根 FUA 后轮换；`SystemRuntimeQuantities` 四项、`FIELD_TABLE_BYTES = 60`；`unmount` `mount.rs:1349` F = 现行 txg。与登记一致，不停 |
| S2 | 七臂的臂表行在快照副本上逐行恰好命中一次（A1 6 行、甲-txg 1、乙F-留环 1、乙 / 丁各 18） |
| S3 | 七份副本 `cargo build --release -p singlefs-harness --lib` 与 bin 都编过（乙两臂各 1 条 `CarryOverAcrossThePool` never constructed 的警告，按定义乙不用它）；今天的 489 与 D22 已定项 9 一致（`r2_decoded_offsets` pass） |
| S4 | 七份副本 `crates/` 与快照逐文件比：差的文件只有各自臂表点名的文件与装置 bin（`today changed 1 unexpected []`、`A1 changed 4 unexpected []`、`jia-txg changed 2`、`yiF-keep-ring changed 2`、乙 / 丁四臂各 `changed 6`，都 `unexpected [] missing []`） |
| S5 | 装置择根（checker 解码）与 `crates::choose_root`、装置判抛弃与 `root_is_abandoned_by_the_instance_table` 在每个用到的镜像上逐条比；九份产物 `name=stop` 0 行、`stop:` 0 处 |
| S6 | 新写的第 2 次跑那一节没有用 `FixedGeometry`；几何全用本地 `GEOMETRY_PRIMARY` 等 |

## 四、问题单第一节前提 1–3

| 前提 | 答 | 依据（`crates/` 文件:行号 2026-09-26 现查；产物行原样） |
|---|---|---|
| 1 被抛弃根今天只由崩溃恢复造出 | 成立 | 管理员回退不抛弃（`mount.rs:3381` `roll_back_by_a_forward_publish`）；H1c 抛弃步：`E7RESULT name=r2_q1_0_summary arm=today geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=56 abandoned_roots_total=202 fewest_faults_that_produced_one=2 first_txg_rule_of_today_mismatches=0 row_publish_txg_at_or_below_the_floor=0`。最少 2 个故障（登记第四节推 3）⇒ F6 的后半「最小故障数与推的 3 不同」按条款只记「推的数不成立」；原因：重放验点名单元要全部份都过、第一份读错就停（`recovery.rs` `replay_journal` 里 `named.locations.iter().all(...)`），一份就断得了链 |
| 2 C331 今天仍打得中、乙要重定义 | 成立 | `first_txg_of_new_instance` 不读 F；今天那一臂 87 次写行发布 txg 与装置按读得出的根与记录算的 max + 1 逐次相等（上一行 `first_txg_rule_of_today_mismatches=0`）；乙族照登记 5.3 分五臂（加 8 字节的乙 / 丁四臂、零字节的乙F-留环）。乙 / 丁副本在 [489, 497) 写出刚落盘那条根的 txg：`E7RESULT name=r2_unmount_check arm=乙-留环 context="q1-0 n1=1" verdict=pass layout_verdict=pass txg_before_unmount=4 floor_in_newest_system_configuration=[4, 4] unmount_publishes=3 expected_publishes=3 last_nonzero_byte=[489, 489] bytes_489_to_497_as_txg=[7, 7] newest_root_txg=7`。这一段没跑 H2 / PC2，「打不打得中」本身没在副本上量 |
| 3 准入改了形态 | 成立 | `MountSpaceAdmission` 四成员（`mount.rs:370`–`390`）、写行之后推（`:2744`）照旧；装置 K0–K4 分类（K3 / K4 按这次调用设备一层写次数，单测 `mount_outcome_class_splits_refusals_by_the_writes_of_the_call`）。G0 上 H1d 再挂载只见 K0 680、K3 120，注入写失败那几次是 K4；K1、K2 没出现——G_小盘没跑，「新准入在 H1c 上走不走得到」还没量 |

## 五、H1d 在各臂上的三格

每臂 800 格（n1 ∈ {0,1,2,3} × 两种造法 × b ∈ {records, unit_first_copy} × 断法；b = unit 那 8 个基准按 V2 作废、不进 800）。三格只取「① 覆盖了没有、② 再挂载 K 类与错误成员、③ 读回标签」逐格比，命令（原样）：

```
cells() { grep "name=r2_h1d_cell " "$1" | sed -E 's/.* (n1=[0-9]+) tip=\([0-9, ]+\) (form=[a-z_]+) (break=[a-z_]+) (cut=[a-z_]+) (point=[a-z0-9]+) .*(cell1_overwritten=[a-z]+) .*(cell2_remount_class=K[0-9]) remount_error=([A-Za-z]+) .*(cell3_read_back_after_remount=[^ ]+) .*/\1 \2 \3 \4 \5 \6 \7 \8 \9/'; }
（今天取快照上那一份 …-r2-all-today-snapshot.out，逐臂 diff）
a1 three_cells_differing=2
jia-txg three_cells_differing=0
yif-keep-ring three_cells_differing=0
yi-keep-ring three_cells_differing=0
yi-config-only three_cells_differing=0
ding-keep-ring three_cells_differing=0
ding-config-only three_cells_differing=0
```

A1 那 2 格（diff 原样）：

```
< n1=0 form=read_fails break=unit_first_copy cut=none point=none cell1_overwritten=false cell2_remount_class=K0 none cell3_read_back_after_remount=no_file
> n1=0 form=read_fails break=unit_first_copy cut=none point=none cell1_overwritten=false cell2_remount_class=K3 AbandonedRootLedgerUnreadable cell3_read_back_after_remount=no_file
< n1=0 form=reads_zeros break=unit_first_copy cut=none point=none cell1_overwritten=false cell2_remount_class=K0 none cell3_read_back_after_remount=no_file
> n1=0 form=reads_zeros break=unit_first_copy cut=none point=none cell1_overwritten=false cell2_remount_class=K3 AbandonedRootLedgerUnreadable cell3_read_back_after_remount=no_file
```

| 臂 | ① 覆盖 | ② 再挂载 | ③ 读回最后确认那一版 |
|---|---|---|---|
| 今天（= 岔路单第 2 行的丙） | 128 / 800 | K0 680、K3 120 | 664 / 800 |
| A1（(a)，树表 / 分配记录树两个指称是同一处拒） | 128 | K0 678、K3 122 | 664 |
| (c) × 两个指称 | 同今天（定义：今天的实现加一笔落点未定的记录） | 同今天 | 同今天 |
| (b) 原形 × 两个指称、(b) 含链 × 两个指称 | 同今天，除 2 格 | 同今天，除 2 格 | 同今天，除 2 格；那 2 格 (b) 没算 |
| 甲-txg / 乙F-留环 / 乙-留环 / 乙-只配置 / 丁-留环 / 丁-只配置 | 128 | K0 680、K3 120 | 664 |

按 n1 与断法（今天，`awk` 汇总 `cells-today.txt`，原样摘）：`n1=0 cut=crash cases=68 cell1_overwritten=56 remount_K3=52 read_back_last_confirmed=16`、`n1=0 cut=write_fails cases=68 cell1_overwritten=56 remount_K3=52 read_back_last_confirmed=16`、`n1=0 cut=barrier_fails cases=24 cell1_overwritten=16 remount_K3=16 read_back_last_confirmed=8`、`n1=0 cut=none cases=4 cell1_overwritten=0 remount_K3=0 read_back_last_confirmed=0`；n1 = 1、2、3 各 212 格 `cell1_overwritten=0 remount_K3=0`，「不断」那 4 格读回 `other_content`、其余全是最后确认那一版。

(b)(c) 由计数推的依据行：`E7RESULT name=r2_h1d_fork_one_derived arm=today fork_one_arm=(b)原形 referent=tree_table cases_where_it_differs_from_this_copy=2 cells=same_as_this_copy_except_those`（六行同形，原形 / 含链 / (c) × 两个指称）。

对用户同日定的那一形意味着什么（只陈述读数）：岔路单第 2 行各臂的定义都只改新实例第一次发布的 txg，不改择根、不改拒不拒；H1d 里覆盖发生在写行根落盘之前、由分配器从所选根的账里发出被藏那条根的单元造成，与第一次发布的 txg 无关——所以这六臂在 H1d 上与今天相同是臂定义的直接后果。重跑登记里没有「系统配置（或别处）知道有更新的根时拒可写挂载」这种形状的臂，这一段量不到那种修法（交主 agent，第十一节第 1 条）。

## 六、Q1-0 在各臂、PC1、实七两段历史

Q1-0（`name=r2_q1_0_summary`，各臂原样）：

```
E7RESULT name=r2_q1_0_summary arm=甲-txg geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=0 abandoned_roots_total=0 fewest_faults_that_produced_one=none first_txg_rule_of_today_mismatches=81 row_publish_txg_at_or_below_the_floor=33
E7RESULT name=r2_q1_0_summary arm=乙F-留环 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=24 abandoned_roots_total=65 fewest_faults_that_produced_one=4 first_txg_rule_of_today_mismatches=81 row_publish_txg_at_or_below_the_floor=0
E7RESULT name=r2_q1_0_summary arm=乙-留环 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=62 abandoned_roots_total=208 fewest_faults_that_produced_one=2 first_txg_rule_of_today_mismatches=6 row_publish_txg_at_or_below_the_floor=0
E7RESULT name=r2_q1_0_summary arm=A1 geometry=GEOMETRY_PRIMARY(S=8,ring=3MiB) attempted=111 not_applicable=24 void_attempts=25 mounted=87 producing_at_least_one_abandoned_root=56 abandoned_roots_total=202 fewest_faults_that_produced_one=2 first_txg_rule_of_today_mismatches=0 row_publish_txg_at_or_below_the_floor=0
```

（乙-只配置、丁两臂与乙-留环的 summary 同数。）甲-txg 一条被抛弃根都造不出：例 `n1=1 c1=C k=2 b=records` 所选根 (1, 2)、写行发布 txg = 3，正是被藏的 (1, 3) 的 txg，落进同一个根槽把它盖掉；乙F-留环造得出的 24 次全在关闭 U 的历史上（`grep … | grep -v 'abandoned_count=0' | grep -o 'c1=[CU]'` 得 `33 c1=U`，含作废的）。这几行是岔路单第 2 行的旁证，不是登记给第 2 行的量。

PC1（原样）：

```
E7RESULT name=r2_positive_control arm=today control=PC1-a node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=allocation_record_tree_root shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-b node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=1 expected_count_by_the_device=1 isolated_slots=0 unit=tree_table shared=false expectation=ok_count_equals_the_device_count verdict=pass
E7RESULT name=r2_positive_control arm=today control=PC1-c node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=1 unintercepted=[] class=K0 error=none writes=71 abandoned_roots_unreadable=0 expected_count_by_the_device=0 isolated_slots=0 exclusive_placements=24 isolated_slots_without_the_fault=28 expectation=ok_count_zero_isolated_zero verdict=pass
E7RESULT name=r2_positive_control arm=A1 control=PC1-a node="n1=1 c1=C k=1 b=unit_first_copy" abandoned_root=(1, 4) faults=2 unintercepted=[] class=K3 error=AbandonedRootLedgerUnreadable writes=0 abandoned_roots_unreadable=none expected_count_by_the_device=1 isolated_slots=none unit=allocation_record_tree_root shared=false expectation=AbandonedRootLedgerUnreadable_and_zero_writes verdict=pass
```

F1、F2 都没触发（PC1-a 计数 = 1 > 0；PC1-c 计数 0、隔离 0，不注入时同一次挂载隔离 28 槽）。V1 对今天那一臂与 A1 没触发。

实七两段（快照上的今天那一臂，原样）：

```
E7RESULT name=r2_seventh_batch_fault_summary arm=today seed=7463871032432355223 drawn=6 new_findings=2 findings_reopening_to_mapping_still_unreadable=2 known_red_hits=0
E7RESULT name=r2_seventh_batch_crash_summary arm=today seed=7463871032432355129 crash_points=4 new_findings=2 findings_reading_back_mapping_still_unreadable=1 known_red_hits=0
```

两段都复现（大档那一段在第 1 步注 `barrier_fails`、重开走到 (1, 3) 报 `MappingStillUnreadable { slot: SlotNumber(50240) }`；快档第 3 步的崩溃状态同一形）。工作树那一份这一段是 `new_findings=1`（快照之后 harness 的记录核对器被改过，`claimed_state_missing_unit` 不再报），其余行与快照那一份逐字节相同（`cmp`：`differ: byte 598107, line 1022`，diff 只有末 3 行）。按臂：乙-留环、乙-只配置与快照今天那一份这一段逐行相同（`seventh_batch_lines_differing_from_today_snapshot=0`），丁两臂 4 行（两个读序号）不同、新发现相同；A1（27 行不同）、甲-txg、乙F-留环（各 29 行不同）历史一开头就走岔，抽到的注入点与崩溃点不再落在那一步，这两段历史在这三臂上没罩住。

## 七、单测、变异

- 单测：`grep -c '#\[test\]' crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` → `57`（43 条沿用 + 14 条第 2 次跑那一节）；工作树 `cargo test --release -p singlefs-harness --bin e158_root_choice_repair`（经 capped.sh 10 与 8G 内存包装）→ `test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s`；`cargo clippy --release -p singlefs-harness --bin e158_root_choice_repair --tests` 0 条警告；`bash .claude/scripts/naming-lint.sh` 对这份文件 0 条。开工就红的那条照第一节处理表改了取被抛弃根的地方（改从 H1c 抛弃步），断言形状不变、转绿；`mount_writable_then_raise_floor_reaches_the_injected_fault` 改打分配记录树节点、断言 `RaisedFloor::abandoned_roots_unreadable > 0`、删掉「报了错也算」。
- 变异：入库装置的变异行进 `crates/mutations.tsv` 末尾（`grep -c 'E158 root_choice_repair 第 2 次跑' crates/mutations.tsv` → `17`，用 `research/scripts/insert-row.py` 逐行插，每行带 `--absent`），归门禁 59 号；59 号整表是重型，我没跑。我在草稿副本上逐条「改坏 → 跑点名的单测 → 还原」（草稿脚本 `tools/run_mutations.py`，每条 cargo test 经 capped.sh 10 与 8G 包装）：新 17 行 + 沿用的第 551、659 行共 19 条，第一遍 **抓到 18、没红 1、无效 0**，还原之后基线 `restored_baseline_exit=0`。没红的那条是「根槽写之前的屏障把紧挨在它之后的也算进去」（`if writes < write_count` → `<=`）：分类为取样点不敏感（第三类）——两种写法只在 `write_count = 0` 且流以屏障开头时不同；单测补了这一格（`barriers_before_write(&barrier_first, 0) == 0`、`… 1) == 1`）之后单独重跑这一条：`caught`。最终 **19 条全抓、0 没红、0 无效**。门禁 33 号：`crates/mutations.tsv 859 条的原文各命中源码一次`。

## 八、产物、完整标记、自查

九份，均 `research/results/`：`e158-root-choice-repair-2026-09-26-r2-all-{today,today-snapshot,a1,jia-txg,yif-keep-ring,yi-keep-ring,yi-config-only,ding-keep-ring,ding-config-only}.out`。每份末行 `E7RESULT name=done emitted=N` 与行数相等（today 1023、today-snapshot 1024、a1 1017、jia-txg 1014、yif-keep-ring 1015、乙 / 丁四份各 1018）。today 由工作树 `cargo run` 出（`replay.sh` 登记那一份）；其余在各自副本上 `cargo build` 后直接跑二进制，环境变量 `SINGLEFS_E158_ARM=<臂名>`、`SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES`（今天 / A1 / 甲 / 乙F 489，乙 / 丁 497）。

自查（4b，一条命令逐份数各类行，原样）：

```
a1.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
ding-config-only.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
ding-keep-ring.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
jia-txg.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=2 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
today.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
today-snapshot.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
yi-config-only.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
yif-keep-ring.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
yi-keep-ring.out h1d_cells=800 h1d_refs=24 q1_0=111 q1_0_histories=6 pc1=3 unmount_checks=6 seeds_fault_drawn=6 seeds_crash_points=4 constants=26 anchors=16
```

H1d 800 格 = n1 = 0：4 个基准 × 41 格；n1 = 1、2、3：各 4 个基准 × 53 格（今天那一份 `grep -o 'n1=… form=… break=…' | sort | uniq -c` 逐组是 41 / 53）。Q1-0 111 = Σ(时间线根数 − 1) × 3 = (4 + 7 + 5 + 7 + 6 + 8) × 3。甲-txg 的 PC1 只有 2 行：PC1-a、PC1-c 两行都是「找不到历史」，PC1-b 跟在 PC1-a 那段历史后面、没有历史就没跑。登记的 H1c 全族（m、c2、op1 O1–O3、Φ1）、H2、H2c 这一段一行都没造——不是缺行，是没跑（第十二节）。

## 九、判决行（4c）

`grep -n 'name=verdict' <每份>`：九份都是 0 行（装置不打 `name=verdict` 行，判定写在各行的 `verdict=` 字段里）。按字段逐个点名：
- `verdict=fail`：只在 `e158-root-choice-repair-2026-09-26-r2-all-jia-txg.out` 第 999、1000 行，`name=r2_positive_control arm=甲-txg control=PC1-a / PC1-c`，`reason="H1c 里找不到…的历史（V1）"`。为什么：甲-txg 下抛弃步一条被抛弃根都造不出（新实例第一次发布的 txg 落回被藏根的 txg、盖掉它们，见第六节）。登记预期内吗：PC1 是岔路单第 1 行各臂的阳性对照，不是第 2 行的；这两行是「这一臂上摆不出」的照实报，不触发 V1（V1 的范围是 PC1 在第 1 行各臂上）。拿不准它该不该写成 `fail`——交主 agent。
- `verdict=not_applicable`：每份 24 行，全在 `name=r2_q1_0`（b = unit / unit_first_copy 而断链发布没有点名单元：暖机、写行那几次零单元发布）；登记 5.1「那次发布没有点名单元时 b = unit 这一支记不适用」，预期内。
- 名字表示作废的计数：每份 `void_cases=8`（H1d：b = unit 的 8 个基准，第二份拦不到，V2）与 `void_attempts=25`（Q1-0 同因），预期内（修订 12.1 第 1 条）。
- 名字表示不匹配的计数：`first_txg_rule_of_today_mismatches` 今天 / A1 两份为 0；甲-txg、乙F-留环为 81，乙 / 丁四份为 6——这几臂按定义就不照今天的规则取 txg，7.1 第六行（停机 S1）只对今天那一臂生效，预期内。`row_publish_txg_at_or_below_the_floor=33` 只在甲-txg（新实例第一次发布落在 F 及以下），拿不准算不算「失败类」，点名在这里。
- 没有别的 `false` 型判定字段；`layout_verdict` 九份全 `pass`，`verdict=pass` 其余全是。

## 十、岔路表（问题单第一节与岔路单各行）

| 行 | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 | 算出它的命令 |
|---|---|---|---|
| 问题单前提 1 | 已够判：成立（56 / 87 造出被抛弃根，最少 2 个故障） | 不能：剩下的量都以它为前提 | `grep 'name=r2_q1_0_summary' research/results/e158-root-choice-repair-2026-09-26-r2-all-today.out` |
| 问题单前提 2 | 已够判：成立（今天的规则 0 处对不上、不读 F；乙族分五臂） | 能部分翻：H2 上 F10（关闭 U 时今天那一臂写行 txg 等于含 F 的算法）没跑，跑了若触发就要重答 | 同上一行的 `first_txg_rule_of_today_mismatches`；源码 `grep -n 'first_txg_of_new_instance' crates/singlefs-core/src/mount.rs` |
| 问题单前提 3 | 已够判：成立（K0–K4 分类落地）；但「新准入下 K1、K2 在 H1c 上走不走得到」没量 | G_小盘能让 Q1-1b 的 K1 / K2 两栏从「没出现」变成有数，不翻前提本身 | `grep -o 'cell2_remount_class=K[0-9]' …-today.out \| sort \| uniq -c` |
| 岔路单第 1 行（被抛弃根的账读不出时「修复」指什么） | 还差：Q1-1a/b/c（G0 全族、O1–O3、瞬时 / 持续）、Q1-2（(b) 两个指称的来源）、Q1-3（(b) 持久化开销）、Q1-4、Q1-5；(b) 两个指称的 PC；H1d 那 2 格上 (b) 的结局；G_S4 / G_S16 / G_小盘；够判条件 ①（与哪几条已定分项相容）归主 agent | 能：翻面观测是「三种形态下被拒比例不同」或「(b) 两个指称开销不同」，这一段一个都没量；H1d 已给出一条定性读数——A1 在 800 格里只多拒 2 格、且拒的时候数据已丢 | H1d：第五节 `cells()` 命令；PC1：`grep 'name=r2_positive_control' …-today.out …-a1.out` |
| 岔路单第 2 行（C331 怎么修） | 还差：Q2-1（H2 崩溃 0 档、H2c 崩溃 1 档，七臂 × 两种打中 × 两种关闭）、PC2 与它的开关、Q2-2a、第八节行 2 敏感性；这一段只有旁证（H1d 六臂与今天相同；Q1-0 上甲-txg 与关闭 C 的乙F-留环把新实例第一次发布落回被藏根的 txg） | 能：翻面观测是「打中它要几个故障」或「每次发布开销」不同，这一段都没量 | Q1-0 旁证：`grep 'name=r2_q1_0_summary' …-{jia-txg,yif-keep-ring,yi-keep-ring}.out` |
| 岔路单第 3 行 | 本登记不量（C332 作废） | — | — |
| 岔路单第 4 行 | 本登记不量（用户挪后） | — | — |

## 十一、交主 agent 的

1. **重跑登记里没有一条臂能量用户同日要改的那一形的修法**：岔路单第 2 行六臂（含乙F-留环）都只改新实例第一次发布的 txg，H1d 的覆盖发生在写行根落盘之前、是分配器从所选根的账里把被藏根的单元发出去，与 txg 无关，所以六臂与今天逐格相同是定义的直接后果。「系统配置（或别处）说有更新的根时拒可写挂载 / 不复用它的单元」这种形状的臂要不要补、补成什么，交主 agent（补臂要走修订，且在那一段产物之前）。
2. H1d 的覆盖只在 n1 = 0（被藏的是首个文件、它前一条根的树表 0 条）时发生；n1 ≥ 1 时写行那次发布开更高的新段、碰不到。实七两段历史都是这一形（第 1 步 / 第 3 步就抛弃根）。n1 ≥ 1 而中间夹过一次重挂载（开新段）的历史这一段没造，覆盖在那里会不会出现没量。
3. 实七那两段历史在 A1、甲-txg、乙F-留环上走岔（harness 的模型按今天的规则答第一次发布的 txg），没罩住；这三臂的 H1d 只能看装置那张表。
4. 工作树的 `crates/` 在快照之后又被改过两次：harness 的记录核对器（崩溃注入那一段在工作树上少一条 `claimed_state_missing_unit`），以及 15:28 / 15:32 UTC 的 `mount.rs`、`transaction.rs`（写行根之前多了屏障）。后者让登记在 `E158` 下的 `r2-all` 那一行复跑对不上（H1d 800 → 832 格，三格计数除「读回最后确认」664 → 696 外不变）。按新代码重出、改指登记行，还是等 `crates/` 落定，交主 agent。
5. 甲-txg 产物里两行 `verdict=fail`（PC1 在那一臂上摆不出）该不该改成别的判定词，交主 agent。
6. `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 改了（不是新 bin）：代码轮判决的点名清单若按「本轮改过的入库装置」算，要点它（门禁 56 号）。
7. 问题单 `research/prompts/e158-r2-questions.md:24` 引了 `/tmp/claude-1000/impl-rbf-4a/report.md`，门禁 69 号报它；不在我的写范围，没动。

## 十二、登记修订、写过的文件

- 重跑登记 `research/prompts/e158-r2-prereg.md` 第十二节新增 12.1（7 条，写于 2026-09-27 00:0x JST、任何产物之前；原判据与原臂一条没改，第六、七、九节的去留一条没动）：① b 补一支 unit_first_copy（依据单测 `replay_stops_at_the_first_copy_of_a_named_unit_so_the_second_copy_is_never_read`）；② H1d 的操作化（n1 从 0 起，依据单测 `the_row_publish_before_its_root_overwrites_the_hidden_newest_root_only_when_it_is_the_first_file`；崩溃只取前缀；(b)(c) 从计数推）；③ 读落点被这次调用写过之后不再拦；④ PC1-c 的历史收严到「只有一条被抛弃根」；⑤ 实七两段历史的参数；⑥ 臂表整张重写、PC 开关这一段没加；⑦ 第一节处理表的删与改这一段没做。
- 写过的文件：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（加第 2 次跑那一节、main 里一处分派、改两条沿用单测、加 14 条单测；sha256 `3f709e69b34bc094de9856b52875513384d0737e46ec69e005f1e18905bd98e2`）；`crates/mutations.tsv` 末尾 17 行；`research/mutations/e158_arms.tsv`（整张重写，31 行；用它在快照上重套七臂，与实际跑的副本 `crates/` 逐文件相同）；`research/prompts/e158-r2-prereg.md` 12.1；`research/scripts/replay.sh`（加 `E158R2` 一行与 `driver_e158_r2_all`，旧的 E158 各行没动）；`research/results/` 九份新产物；`.claude/kb/experiments/158-择根与修复四岔路.md`（标题括注、正文最前「2026-09-26：第 2 次跑第一段」一节、影响的决策表五行回看、历史版本 2026-09-26 一条）；`.claude/kb/experiments.md` E158 那一行（状态列与结论列）；`.claude/kb/experiments-history.md` 2026-09-26 一条。
- 实验页的「影响的决策」表：五行（D16 已定项 6、D23 已定项 14、D22 已定项 9、D8 已定项 8、D28 已定项 3）都重新 `grep -c "E158（"` 那条决策文件，都得 0，依据段都没引这个实验 ⇒ 都仍写备料、回看日期改成 2026-09-26；没有一格该升成支撑 / 推翻。都不在 `.claude/decision-links-pending` 里。
- `replay.sh`：新行先以编号 `E158R2` 登记，`bash research/scripts/replay.sh E158R2`（15:1x UTC）→ `字节一致 1 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0`（退出 0）。之后门禁 88 号（15:25 UTC 被别的会话改过）只认 `E<数字>|` 形状的登记行，改登记到 `E158` 下（`E158|@driver_e158_r2_all||e158-root-choice-repair-2026-09-26-r2-all-today.out|exact`）。`replay.sh E158`（含第一次跑的 14 行）：第一次跑那 14 行里跑完的 11 行全「对不上」（`local_value=481` → `489`、挂载时回退删除带来的结构性差异，与这一段无关），另 3 行（`q3-1-s4`、`q2-1-g0` 两行，各跑 15 分钟以上）被我停掉（`proc.py stop` 734920、735061、735152，链随之退出；判「跑不了 3」），汇总 `字节一致 0 ／ 仅计时不同 0 ／ 对不上 11 ／ 跑不了 3`。
- ⚠️ 产物之后 `crates/singlefs-core/src/mount.rs`（15:28 UTC）、`transaction.rs`（15:32 UTC）被别的会话改了：15:4x UTC 在工作树上重跑 `r2-all`，与登记产物 121 行不同——H1d 从 800 格变 832 格（写行根之前的屏障每个基准 6 → 8 次），`cell1_overwritten_cases=128`、K3 120 不变，读回最后确认那一版 664 → 696；Q1-0、PC1 行不变（草稿 `recheck-after-core-change.out`）。登记产物没重出，交主 agent 定；实验页复跑一段写明了。装置对新代码照样编得过。

## 十三、门禁（登记给 experiment-runner 的阶段，最后一次 15:5x UTC，末行原样摘、退出码）

| 阶段 | 退出码 | 末行 / 红在哪 |
|---|---|---|
| 27 | 0 | `✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）` |
| 33 | 1 | 红的是 `crates/mutations.tsv` 第 301、690、692、714 行（C378、实二八、实二），15:28 / 15:32 UTC 别的会话改 `mount.rs`、`transaction.rs` 之后锚点失配；我加的 17 行不在内。15:1x UTC 那一次是 `✓ … crates/mutations.tsv 859 条的原文各命中源码一次` |
| 34 | 0 | `✓ 实验索引行与正文标题一致（索引 159 行、正文 159 份）` |
| 40 | 1 | 红的是 E142 两份、E156 `…-2026-09-26-r2.out`、两份 gate69 日志；E158 只出现在「没判的」清单里（历史点名）。E158 开工时报的 4 份旧产物已在实验页历史节点名 |
| 52 | 0 | `✓ 比对了 4 处登记 …` |
| 69 | 1 | 红的是 E142 装置指纹、`research/prompts/_defs-m2-closeout-r2-appendix.md:1158` 与 `research/prompts/e158-r2-questions.md:24` 两处 /tmp 引用（后者是问题单，不在我的写范围） |
| 75 | 0 | `✓ 决策与实验双向登记对得上、回看不过期 …` |
| 80、84、85、86、96 | 0 | 84：`✓ 登记产物里判决字段是 false 的都在对应实验页里点了名（点名 3 个）`；85：`✓ 点了产物的实验都写了复跑命令` |
| 88 | 1 | 红的只剩 `.claude/kb/experiments/142-第一个事务的干跑.md:120` 一行；E158 页里抄的产物行都只取自登记的 `…-r2-all-today.out`（别的臂改成按字段写） |
| 99 | 0 | `✓ 登记了路径与共用项的实验页判过了（3 份、14 条路径）…` |
| 20（kb 形状 / doc-lint，另跑） | 0 | `✓ kb 形状检查通过` |

15 号、87 号不归我，没跑；59 号（crates 变异整表）是重型，没跑。

## 十四、没做什么

- 登记第一段的「第一节处理表里的删与改」（shim、H3、Q3-1、N_w、PC3、PC-Nw、权重上限 12 与它们的单测、变异行、`replay.sh` 里指向它们的行）没做（修订 12.1 第 7 条）；第一次跑的 14 行复跑全对不上，要随那件事一起处置。
- 登记第二段起的量都没跑：Q1-1 至 Q1-5、(b) 两个指称的 PC、G_S4 / G_S16 / G_小盘；Q2-1（H2、H2c）、PC2 与它的开关、Q2-2a、第八节行 2；岔路单第 3、4 行按登记不量。
- H1d 只在 G0、关闭 C、藏 1 条根、前缀崩溃状态上跑；(b) 在 H1d 那 2 格上的结局没算。
- 没判这些读数能不能推翻或确立任何决策（走三方）；没提交；没跑门禁全量、15、59、87 号。
- 没改 `crates/singlefs-core`、`crates/singlefs-checker`；臂的改动只在草稿副本上。

## 十五、草稿与清理

- 草稿目录 `/tmp/claude-1000/e158-r2-runner/`：接手时的旧快照 `snapshot/` 与上一任的 `probe/`（含它的 target）已删；我建的 `snapshot2/`（快照）、`arms/`（八份副本连各自 target，3.7G）、`mutation-copy/`（变异用副本）都已删。留着的只有文本：报告、`progress.md`、日志、`tools/`（`gen_arms.py`、`apply_arm.py`、`splice_device.py`、`patch_legacy_tests.py`、`run_mutations.py`、`run_products.sh`、`build_arm_libs.sh`）、`hunks/`、`legacy/`、`final/e158_root_choice_repair.rs`（与工作树那份同 sha256）、`products/`（今天那一臂在快照上的第一遍输出与日志）、`cells-*.txt`、`recheck-after-core-change.out`。没有编译目录、工作树或仓副本留在临时目录里。
- `replay.sh` 自己的输出目录 `/tmp/singlefs-replay-625660`（E158R2 那次）与 `/tmp/singlefs-replay-733923`（E158 那次）是脚本按惯例留的，没删。
- 没有留下在跑的进程（`ps` 查过 `e158_root_choice_repair` / `replay.sh` 为空）。
