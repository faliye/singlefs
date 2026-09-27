# 调查：E158 第 4 次跑第二段，-配置 各臂实七-乙 `crash0`–`crash2` 的 `content_differs_from_the_ledger`（2026-09-27 JST）

## 一、结论

1. **是装置记账的毛病，不是被测挂载读错了内容。** 对 `crash0`–`crash2` 三格，崩溃分支可写挂载之后实际读到的是「没有文件」（`actual_read_after_mount=no_file`），与崩溃镜像读回的内容号 0（`0=no_file`）一致。报错是因为账里键 `2:4` 记的是内容号 1（`1=len3000:h0a49e06612569303`）。这一版是 -配置 主路径在第 8 步挂载时发布的根 `2:4`，崩溃分支自己的挂载也发布了一个根 `2:4`，两版用的是同一个键 `(实例, txg)`。崩溃分支查的是主路径的账（臂副本 bin 第 20460 行 `&main.ledger,`），按键查到了主路径那一版（第 18432 行），于是走进第 18440 行的 `content_differs`。
2. **最小复现**：臂 yi-cfg，族 seventh-crash，种子 `base+16`，格 `crash0`–`crash2`（崩溃序 0，分段 7、8、9）。一条命令约 5 秒，见第三节。**推翻条件**：让崩溃分支用自己的账（主路径账的拷贝，把分支挂载新写出的根改记成崩溃镜像读回的内容号），这 3 格应当变成 `last_confirmed`、`after_mount_read_back_content=0`、`lost_write_cell=false`，实七-乙 丢写 3/6 → 0/6。在副本里改了、跑了，结果正是如此（第四节）。
3. **今天那一臂与 -槽 三臂同样按主路径的账查同一个键 `2:4`，只是碰巧没出事。** 它们的主路径第 4 步崩溃恢复成功（`applied:mounted:K0`），退回到内容号 0 那一版，于是主路径的 `2:4` 记的也是内容号 0（`(2, 4): 0`），与分支实际读到的「没有文件」相同，所以判成 `last_confirmed`。-配置 八臂第 4 步都拒了（乙、甲、乙-窄读拒 `R3NewerRootWitnessedByTheSystemConfiguration`，丙拒 `R3RebuildFromRecordsStoppedBelowTheWitness`），第 8 步挂载时承接的是内容号 1，`2:4` 就记成了 1。
4. **别的族**（逐族依据见第六节）：
   - H-随 6/37：不受这处毛病影响。H-随 没有崩溃分支（分支行 0 条）。
   - 实七-甲 0/10：不受影响。注入分支用的是自己那个执行器的账（第 20310 行 `&executor.ledger,`）。
   - 实八 0/2（-配置 各臂）：不受影响。4 个崩溃状态都在第 20387 行找段那一步就 `not_constructible=segment_missing` 返回了，没走到查账。
   - 同一处毛病在**今天那一臂（-槽 三臂按同一产物推）的实八** `crash1:crash@5` 上也出现了（键 `3:10`），不过那一格改用自己的账后是 `older:1`，仍然算丢写，实八 3/6 不变。
   - 在副本里改用自己的账、整条臂重跑，yi-cfg 与今天两臂里除了上面这些格，别的行一行没变（第四节）。
5. **什么现象会推翻这个定位**：分支挂载之后实际读到的字节不是「没有文件」；或者改用自己的账之后这 3 格仍然是 `content_differs`；或者把自己的账故意记错之后，这 3 格不回到 `content_differs`（那样说明这条判定是瞎的）。前两样都量过，没有出现。第三样在副本里造了一次，确实回到了 `content_differs`（第五节）。

## 二、原样复现

- 负载（JST 12:05 前后）：`ps` 看到别的会话有 `cargo test --offline -p singlefs-harness --test second_transaction_supplement_three_fault_injection`、`--test crash_enumeration_sharded_across_processes`、`--lib`，没有 qemu、vm-bench、e152、fio。没有等锁。
- 臂副本 bin 的 sha256：yi-cfg 与今天两份都是 `12b82f491453e4f598f8262632dc949967508a8a1cae745a1b4db078499fb148`，与执行员报告一致。
- 做法：`cp -a` 两份臂副本（连 target）到草稿目录的 `yi-cfg-repro`、`today-repro`，用草稿脚本 `/tmp/claude-1000/investigate-e158-seg2-ledger/run_one.sh` 跑。环境变量照 `run_arm.sh`，外加 `capped.sh 1` 与 `run-with-memory-cap.sh 10G`，两条并行，合计 2 个线程。
- 结果（`progress.md` 原样）：

```
12:10:12 JST repro-today rc=0 用时 193 秒 行数 1834
12:12:13 JST repro-yi-cfg rc=0 用时 314 秒 行数 1752
```

- 与产物逐字节比：`cmp $S/repro-yi-cfg.out $R-yi-cfg.out && echo yi-cfg_identical` 输出 `yi-cfg_identical`，今天那一臂输出 `today_identical`。**原现象复现出来了，而且确定性地一样。**

## 三、定位：诊断读数与最小复现

诊断只加在副本里（`yi-cfg-diag`、`today-diag`，补丁脚本 `patch_diag.py`，最终的 diff 在 `diag-final.diff`，100 行）。改动如下：
- 在 `run_a_crash_branch` 的 after-mount 读回之后往 stderr 打一行 `INVESTIGATE name=crash_branch_ledger`，内容包括：着陆的键、主路径账在这个键上的内容号与字节摘要、挂载后实际读到的字节摘要、崩溃镜像与挂载后各自可读的根、分支挂载新写出的根、「自己的账」下的读回类别。
- 两个开关，默认都关：`E158_INVESTIGATE_OWN_LEDGER=1` 时改用自己的账判；`E158_INVESTIGATE_ONLY_FAMILY=<族名>` 时只跑那一族。
- 第五节另加了 `E158_INVESTIGATE_OWN_NUMBER`，只在造推翻现象时用。

**「不改判定」的证据**：诊断版不设开关、整条臂跑，stdout 与产物逐字节相同。命令 `cmp $S/diag-yi-cfg.out $R-yi-cfg.out && echo diag_yi_cfg_stdout_identical` 输出 `diag_yi_cfg_stdout_identical`，今天那一臂输出 `diag_today_stdout_identical`。编译：`capped.sh 2 cargo build --release --offline --bin e158_root_choice_repair`，两份 rc=0，0 条 warning、0 条 error。

yi-cfg 那 3 格的诊断行原样（`diag-yi-cfg.err`，已切掉末尾的整本账，整本账见下）：

```
INVESTIGATE name=crash_branch_ledger arm=yi-cfg attempt=crash0 crash_order=0 literal_last=0 rollback_aware_last=0 crash_read_back_number=0 crash_read_back_bytes=0=no_file landing_key=2:4 main_ledger_number_at_key=1 main_ledger_bytes_at_key=1=len3000:h0a49e06612569303 actual_read_after_mount=no_file crash_image_roots={(0, 0)} after_mount_roots={(0, 0), (2, 2), (2, 3), (2, 4)} new_roots_by_the_branch_mount=[(2, 2), (2, 3), (2, 4)] main_ledger_numbers_of_new_roots=[((2, 2), None), ((2, 3), None), ((2, 4), Some(1))] own_ledger_number_of_new_roots=0 own_ledger_after_mount_read_back=last_confirmed own_ledger_after_mount_read_back_rollback_aware=last_confirmed own_ledger_after_mount_read_back_root=2:4 own_ledger_after_mount_read_back_content=0
INVESTIGATE name=crash_branch_ledger arm=yi-cfg attempt=crash1 crash_order=0 literal_last=0 rollback_aware_last=0 crash_read_back_number=0 crash_read_back_bytes=0=no_file landing_key=2:4 main_ledger_number_at_key=1 main_ledger_bytes_at_key=1=len3000:h0a49e06612569303 actual_read_after_mount=no_file crash_image_roots={(0, 0), (1, 1)} after_mount_roots={(0, 0), (1, 1), (2, 2), (2, 3), (2, 4)} new_roots_by_the_branch_mount=[(2, 2), (2, 3), (2, 4)] main_ledger_numbers_of_new_roots=[((2, 2), None), ((2, 3), None), ((2, 4), Some(1))] own_ledger_number_of_new_roots=0 own_ledger_after_mount_read_back=last_confirmed own_ledger_after_mount_read_back_rollback_aware=last_confirmed own_ledger_after_mount_read_back_root=2:4 own_ledger_after_mount_read_back_content=0
INVESTIGATE name=crash_branch_ledger arm=yi-cfg attempt=crash2 crash_order=0 literal_last=0 rollback_aware_last=0 crash_read_back_number=0 crash_read_back_bytes=0=no_file landing_key=2:4 main_ledger_number_at_key=1 main_ledger_bytes_at_key=1=len3000:h0a49e06612569303 actual_read_after_mount=no_file crash_image_roots={(0, 0), (1, 1)} after_mount_roots={(0, 0), (1, 1), (2, 3), (2, 4)} new_roots_by_the_branch_mount=[(2, 3), (2, 4)] main_ledger_numbers_of_new_roots=[((2, 3), None), ((2, 4), Some(1))] own_ledger_number_of_new_roots=0 own_ledger_after_mount_read_back=last_confirmed own_ledger_after_mount_read_back_rollback_aware=last_confirmed own_ledger_after_mount_read_back_root=2:4 own_ledger_after_mount_read_back_content=0

同一格今天那一臂（`diag-today.err`）：

```
INVESTIGATE name=crash_branch_ledger arm=today attempt=crash0 crash_order=0 literal_last=0 rollback_aware_last=0 crash_read_back_number=0 crash_read_back_bytes=0=no_file landing_key=2:4 main_ledger_number_at_key=0 main_ledger_bytes_at_key=0=no_file actual_read_after_mount=no_file crash_image_roots={(0, 0)} after_mount_roots={(0, 0), (2, 2), (2, 3), (2, 4)} new_roots_by_the_branch_mount=[(2, 2), (2, 3), (2, 4)] main_ledger_numbers_of_new_roots=[((2, 2), None), ((2, 3), None), ((2, 4), Some(0))] own_ledger_number_of_new_roots=0 own_ledger_after_mount_read_back=last_confirmed own_ledger_after_mount_read_back_rollback_aware=last_confirmed own_ledger_after_mount_read_back_root=2:4 own_ledger_after_mount_read_back_content=0
```

两臂实七-乙主路径账的全部键与内容号（同一诊断行末尾原样，`attempt=crash0 crash_order=0` 那一行）：

```
diag-yi-cfg: main_ledger_all_keys={(0, 0): 0, (1, 1): 0, (1, 2): 0, (1, 3): 1, (2, 4): 1, (2, 5): 1, (2, 6): 3, (3, 7): 3, (3, 8): 3, (4, 9): 3, (4, 10): 3, (5, 11): 3, (5, 12): 3, (5, 13): 3} main_ledger_contents=["0=no_file", "1=len3000:h0a49e06612569303", "2=len32635:h82b162f6d9102c56", "3=len15379:h3d85f6097da3dfb9"]
diag-today: main_ledger_all_keys={(0, 0): 0, (1, 1): 0, (1, 2): 0, (1, 3): 1, (2, 4): 0, (2, 5): 0, (3, 6): 0, (3, 7): 0, (3, 8): 4, (3, 9): 5, (4, 10): 5, (4, 11): 5, (5, 12): 5, (5, 13): 5, (6, 14): 5, (6, 15): 5, (6, 16): 5} main_ledger_contents=["0=no_file", "1=len3000:h0a49e06612569303", "2=len32635:h82b162f6d9102c56", "3=len32768:he1766e1a7cd69b64", "4=len26010:h48f8d286f86181ad", "5=len12782:h14147ab631bd6885"]
```

**读数怎么读（正推）**：
- 崩溃镜像只有 mkfs 那一截的根（`{(0, 0)}` 或加上 `(1, 1)`）。分支的可写挂载写出 `(2, 2)`、`(2, 3)`、`(2, 4)`，挂载后读回的文件是「没有文件」，与崩溃镜像读回的内容号 0 字节相同。**被测挂载没有读错。**
- yi-cfg 主路径账里 `(2, 4): 1`。它来自主路径第 8 步的挂载：`name=r4_random_mount family=seventh-crash … step=8 … operation=mount … a_chosen=1:3`，承接的根 `1:3` 是内容号 1（`(1, 3): 1`）；在它之前，第 4 步崩溃恢复拒了（`a_error=R3NewerRootWitnessedByTheSystemConfiguration`）。
- 今天那一臂主路径第 4 步崩溃恢复选了 `a_chosen=1:2`（内容号 0，抛弃 `1:3`），它写出的 `2:4` 记成 `(2, 4): 0`。
- 两条时间线（主路径、崩溃分支）在各自镜像上的下一个挂载都拿到实例号 2、写到 txg 4，所以键 `(实例, txg)` 撞上了。账的键类型是 `type TimelineRoot = (u32, u64);`（第 342 行）、`content_number_by_root: BTreeMap<TimelineRoot, u64>,`（第 18283 行），键里没有时间线这一维。

代码路径（臂副本 bin，sha256 `12b82f49…`，行号用 `awk 'NR==n'` 现取）：
- 第 20458 行 `let after_mount_read_back = classify_random_read_back(`，第 20460 行 `&main.ledger,`：分支挂载之后的读回拿主路径的账判。
- 第 18432 行 `let by_root = landing.and_then(|root| ledger.content_number_of(root));`：按着陆根的键查内容号，查到就不再按字节找。
- 第 18440 行 `let member = "failed:content_differs_from_the_ledger".to_string();`：查到的那一版字节与实际读到的不等，就判这一类。
- 同一个函数里第 20423 行（崩溃镜像读回）也传 `&main.ledger,`。崩溃镜像只含主路径写序列的前缀，那里的键与主路径同源，这 3 格上判的是 `last_confirmed`，没有撞键。第 20447 行传给 `evaluate_a_random_mount` 的也是主路径的账，这一处没展开查（见「没做什么」）。

**最小复现**（一条命令，约 5 秒；`S=/tmp/claude-1000/investigate-e158-seg2-ledger`，副本 `yi-cfg-diag` 已经编好）：

```
bash $S/run_one.sh yi-cfg-diag yi-cfg min-yi-cfg E158_INVESTIGATE_ONLY_FAMILY=seventh-crash
```

`progress.md` 原样 `12:24:53 JST min-yi-cfg rc=0 用时 5 秒 行数 105`。3 格读数（`grep -o` 抽字段，`paste` 拼行；`cell=false` 是 `overwrite_cell=false` 被截出的尾巴）：

```
cell=crash0:crash@0	cell=false	after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_root=2:4	after_mount_read_back_content=1
lost_write_cell=true	cell=crash1:crash@0	cell=false	after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_root=2:4
after_mount_read_back_content=1	lost_write_cell=true	cell=crash2:crash@0	cell=false	after_mount_read_back=failed:content_differs_from_the_ledger
after_mount_read_back_root=2:4	after_mount_read_back_content=1	lost_write_cell=true		
cells=6 lost_write_cells_q1_q2_q3=3
q3_lost_cells=3
```

只跑一族不改变这一族的行：换了覆写开关重编之后又跑一次，`cmp $S/min2-yi-cfg.out $S/min-yi-cfg.out` 输出 `rebuilt_default_identical`。只跑一族时，产物里 H-随、实七-甲、实八 的行 0 条（`grep -c` 输出 `0`）。

## 四、推翻条件：改用分支自己的账

「自己的账」的做法：拷一份主路径的账，把分支挂载新写出的根（挂载后可读、崩溃镜像里不可读的那几条）改记成崩溃镜像读回的内容号。这与主路径 `carry_new_ring_roots` 给挂载新根记「承接的那一版」同一个意思（第 18867–18869 行）。这只是用来验定位的办法，**不是修法建议**。

最小复现加开关：`bash $S/run_one.sh yi-cfg-diag yi-cfg min-own-yi-cfg E158_INVESTIGATE_ONLY_FAMILY=seventh-crash E158_INVESTIGATE_OWN_LEDGER=1`，rc=0，5 秒。读数：

```
cell=crash0:crash@0	cell=false	after_mount_read_back=last_confirmed	after_mount_read_back_root=2:4	after_mount_read_back_content=0
lost_write_cell=false	cell=crash1:crash@0	cell=false	after_mount_read_back=last_confirmed	after_mount_read_back_root=2:4
after_mount_read_back_content=0	lost_write_cell=false	cell=crash2:crash@0	cell=false	after_mount_read_back=last_confirmed
after_mount_read_back_root=2:4	after_mount_read_back_content=0	lost_write_cell=false		
cells=6 lost_write_cells_q1_q2_q3=0
q3_lost_cells=0
```

**这 3 格变成 `last_confirmed`，内容号 0，不丢写；yi-cfg 实七-乙 3/6 → 0/6。**

两条臂整条改用自己的账重跑（`E158_INVESTIGATE_OWN_LEDGER=1`，不限族），拿产物逐字段比（草稿脚本 `fielddiff.py`：按 `key=value` 逐行比，只打出变了的行与字段）。`progress.md` 原样：`12:21:33 JST own-today rc=0 用时 200 秒 行数 1834`、`12:23:46 JST own-yi-cfg rc=0 用时 333 秒 行数 1752`。`python3 $S/fielddiff.py $R-<臂>.out $S/own-<臂>.out` 原样输出：

```
== yi-cfg
line1699 name=r4_random_branch family=seventh-crash attempt=crash0 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0
line1701 name=r4_random_cell family=seventh-crash cell=crash0:crash@0 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0 q3_lost:true->false q3_lost_rollback_aware:true->false q3_last_confirmed:false->true lost_write_cell:true->false lost_write_cell_rollback_aware:true->false
line1702 name=r4_random_branch family=seventh-crash attempt=crash1 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0
line1704 name=r4_random_cell family=seventh-crash cell=crash1:crash@0 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0 q3_lost:true->false q3_lost_rollback_aware:true->false q3_last_confirmed:false->true lost_write_cell:true->false lost_write_cell_rollback_aware:true->false
line1705 name=r4_random_branch family=seventh-crash attempt=crash2 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0
line1707 name=r4_random_cell family=seventh-crash cell=crash2:crash@0 | after_mount_read_back:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->last_confirmed after_mount_read_back_content:1->0 q3_lost:true->false q3_lost_rollback_aware:true->false q3_last_confirmed:false->true lost_write_cell:true->false lost_write_cell_rollback_aware:true->false
line1711 name=r4_random_summary family=seventh-crash | lost_write_cells_q1_q2_q3:3->0 lost_write_cells_rollback_aware:3->0 lost_write_cells_without_q3c:3->0 lost_write_cells_with_q1_in_ring_only_and_without_q3c:3->0 q3_lost_cells:3->0 q3_lost_cells_rollback_aware:3->0 q3_last_confirmed_cells:3->6 lost_write_cells_by_destination:{"crash":->{}
== today
line1820 name=r4_random_branch family=eighth-crash attempt=crash0 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
line1822 name=r4_random_cell family=eighth-crash cell=crash0:crash@4 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
line1823 name=r4_random_branch family=eighth-crash attempt=crash1 | after_mount_read_back:failed:content_differs_from_the_ledger->older:1 after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->older:1 after_mount_read_back_content:5->1
line1825 name=r4_random_cell family=eighth-crash cell=crash1:crash@5 | after_mount_read_back:failed:content_differs_from_the_ledger->older:1 after_mount_read_back_rollback_aware:failed:content_differs_from_the_ledger->older:1 after_mount_read_back_content:5->1
line1826 name=r4_random_branch family=eighth-crash attempt=crash2 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
line1828 name=r4_random_cell family=eighth-crash cell=crash2:crash@15 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
line1829 name=r4_random_branch family=eighth-crash attempt=crash3 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
line1831 name=r4_random_cell family=eighth-crash cell=crash3:crash@15 | after_mount_read_back:last_confirmed:by_bytes->last_confirmed after_mount_read_back_rollback_aware:last_confirmed:by_bytes->last_confirmed
```

- yi-cfg：变的只有实七-乙那 3 格、它们的分支行和实七-乙的汇总行；H-随、H-随全、实七-甲、实八 一行没变。
- 今天那一臂：实七-乙一行没变；实八 4 个崩溃分支变了。其中 3 格只是 `last_confirmed:by_bytes` → `last_confirmed`（原来主路径账里没有这个键，退到按字节找；丢写判定不变）。`crash1:crash@5` 从 `content_differs` 变成 `older:1`，仍然算丢写。实八汇总行没变（3/6）。

## 五、造一次推翻现象：自己的账故意记错，判定会不会回红

目的是说明第四节变绿不是因为这条判定变瞎了。给副本加覆写开关 `E158_INVESTIGATE_OWN_NUMBER`，只在第四节那种自己的账里把分支新根的内容号改成给定的值；改完重编，rc=0。然后故意记成内容号 1：

```
bash $S/run_one.sh yi-cfg-diag yi-cfg falsify-yi-cfg E158_INVESTIGATE_ONLY_FAMILY=seventh-crash E158_INVESTIGATE_OWN_LEDGER=1 E158_INVESTIGATE_OWN_NUMBER=1
```

原样输出：

```
 cell=crash0:crash@0	 after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_content=1	lost_write_cell=true
 cell=crash1:crash@0	 after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_content=1	lost_write_cell=true
 cell=crash2:crash@0	 after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_content=1	lost_write_cell=true
 cell=crash3:crash@24	 after_mount_read_back=failed:content_differs_from_the_ledger	after_mount_read_back_content=1	lost_write_cell=true
lost_write_cells_q1_q2_q3=4
```

记错之后 4 个崩溃格全部回到 `content_differs`（`crash3` 原本内容号 3，也被改成 1），丢写 4。**判定会红**，第四节的绿是账对了的结果。另一个现成的旁证：今天那一臂实八 `crash1:crash@5` 在自己的账下判成 `older:1`，分支挂载之后真读到了更旧的那一版（实际读到 `len32633:h31c1cb8a5fec00b4`，等于内容号 1 的字节），这条判定照样报丢写。

## 六、各臂、各族

**为什么只有 -配置 各臂中招**：比的是实七-乙主路径各步的 `order` 与 `fingerprint_after` 串起来的 sha256 前 12 位，另取第 4 步结局与 3 格读回；产物原样：

```
today step4 outcome="applied:mounted:K0" step8 fingerprint_after=8828d8c9dbd8ed35 main_steps_fp=1aecbc785101 |  3 after_mount_read_back_content=0  3 after_mount_read_back=last_confirmed
jia-slot step4 outcome="applied:mounted:K0" step8 fingerprint_after=8828d8c9dbd8ed35 main_steps_fp=1aecbc785101 |  3 after_mount_read_back_content=0  3 after_mount_read_back=last_confirmed
yi-slot step4 outcome="applied:mounted:K0" step8 fingerprint_after=8828d8c9dbd8ed35 main_steps_fp=1aecbc785101 |  3 after_mount_read_back_content=0  3 after_mount_read_back=last_confirmed
yi-narrow-slot step4 outcome="applied:mounted:K0" step8 fingerprint_after=8828d8c9dbd8ed35 main_steps_fp=1aecbc785101 |  3 after_mount_read_back_content=0  3 after_mount_read_back=last_confirmed
jia-cfg step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
jia-cfg-carry step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
yi-cfg step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
yi-cfg-carry step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
bing-cfg step4 outcome="refused:R3RebuildFromRecordsStoppedBelowTheWitness" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
bing-cfg-carry step4 outcome="refused:R3RebuildFromRecordsStoppedBelowTheWitness" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
yi-narrow-cfg step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
yi-narrow-cfg-carry step4 outcome="refused:R3NewerRootWitnessedByTheSystemConfiguration" step8 fingerprint_after=711b43689b08fa51 main_steps_fp=2926d504fcc2 |  3 after_mount_read_back_content=1  3 after_mount_read_back=failed:content_differs_from_the_ledger
```

- -槽 三臂的主路径与今天那一臂逐步同指纹（`1aecbc785101`），第 4 步崩溃恢复成功，`2:4` 记成内容号 0，与分支实际读到的「没有文件」相同。所以撞键了却判成 `last_confirmed`：**毛病在，被同内容掩盖了。**
- -配置 八臂逐步同指纹（`2926d504fcc2`），第 4 步都拒，第 8 步挂载承接内容号 1，`2:4` 记成 1，于是撞出 `content_differs`。
- -槽 三臂没有加诊断跑；上面「记 0」这一条对今天那一臂是诊断量出来的，对 -槽 三臂是凭主路径同指纹、同读回推的。

**第二段别的族**（每一族按「走不走 `run_a_crash_branch` 的 after-mount 读回」判。各族分支行数从产物现数，命令 `grep -c "name=r4_random_branch family=$fam "` 等，原样输出）：

```
yi-cfg h-random branch_lines=0 crash_attempt_lines=0 not_constructible=0 content_differs=0
yi-cfg h-random-all branch_lines=0 crash_attempt_lines=0 not_constructible=0 content_differs=0
yi-cfg seventh-fault branch_lines=6 crash_attempt_lines=0 not_constructible=0 content_differs=0
yi-cfg seventh-crash branch_lines=4 crash_attempt_lines=4 not_constructible=0 content_differs=6
yi-cfg eighth-crash branch_lines=4 crash_attempt_lines=4 not_constructible=4 content_differs=0
today h-random branch_lines=0 crash_attempt_lines=0 not_constructible=0 content_differs=0
today h-random-all branch_lines=0 crash_attempt_lines=0 not_constructible=0 content_differs=0
today seventh-fault branch_lines=6 crash_attempt_lines=0 not_constructible=0 content_differs=0
today seventh-crash branch_lines=4 crash_attempt_lines=4 not_constructible=0 content_differs=0
today eighth-crash branch_lines=4 crash_attempt_lines=4 not_constructible=0 content_differs=2
```

- **H-随 6/37（-配置）**：不受影响。H-随走 `run_the_random_families` → `run_the_main_branch`，只有主路径一条时间线，读回用的是它自己的执行器的账，没有分支行（`branch_lines=0`）。那 6 格是 ③c 尾巴 `failed:NoValidRoot` / `failed:UnitUnreadable`，是尾巴那次 `recover` 本身报的错：`third_tail_class` 第 19637–19638 行 `RecoveryOutcome::Failed` 那一支直接给 `failed:…`，不经过按键查账（按键查账的是第 19646 行那一支，走不到）。整臂改用自己的账重跑，H-随 一行没变（第四节）。
- **实七-甲 0/10（-配置）**：不受影响。注入分支 `run_a_fault_branch` 从起点重跑一遍，用的是分支自己那个执行器的账（第 20310 行 `&executor.ledger,`），不借主路径的账。整臂重跑一行没变。
- **实八 0/2（-配置）**：不受影响。4 个崩溃状态都在第 20387 行 `let Some(segment_index) = stream.find(` 那一步以 `not_constructible=segment_missing` 返回（`not_constructible=4`），没走到查账。剩下的 2 格是主路径的格。
- **实八 在今天那一臂（-槽 同指纹，推的）**：同一条路径真跑到了。`crash1:crash@5` 撞键 `3:10`：主路径记 5（`len32634`），分支实际读到 `len32633`，等于内容号 1。改用自己的账后是 `older:1`，这一格 `overwrite_cell=true`、`q2_rollback=true`，本来就丢写，汇总 3/6 不变。另外 3 格原来就不在主路径账里，按字节找到了对的内容号，丢写判定不受影响。
- **实七-乙 在今天、-槽 各臂 1/5**：那 1 格是主路径 `main:crash_recovery@4`，不走分支读回。`crash0`–`crash2` 撞键但被同内容掩盖（见上），数不变。

## 七、排除掉的解释

| 解释 | 排除它的观测 |
|---|---|
| 被测挂载真把内容读错了 | 诊断的 `actual_read_after_mount=no_file`，与崩溃镜像读回的内容号 0（`0=no_file`）字节相同。这条读数直接取 `recover` 的结果做字节摘要，不经过账 |
| 两臂起始镜像或崩溃镜像不同 | 两臂诊断行的 `crash_image_roots`、`after_mount_roots`、`new_roots_by_the_branch_mount` 逐项相同，`actual_read_after_mount` 都是 `no_file`；执行员报告里的起始指纹 `97f3fd5d54dd1462`、`a_chosen=0:0` 两臂也一样 |
| 复现依赖环境或是偶发的 | 原样重跑两臂，与产物逐字节相同（第二节） |
| 诊断改了判定 | 诊断版不设开关跑，stdout 与产物逐字节相同（第三节） |
| 改用自己的账之后判定变瞎了 | 故意记错，判定回到 `content_differs`（第五节） |
| 被测挂载在这 3 格上拒了、没写根 | 挂载后新出现了根 `(2, 2)`–`(2, 4)`，读回落在 `2:4` |

## 八、没做什么

- 没修，没判该怎么修。「自己的账」只是为验定位写的，不是修法建议。没碰主工作区、`research/results/`、那 12 份臂副本（只读、只从里面拷出）。
- 没跑重型测试。没跑门禁。
- -槽 三臂没加诊断跑，结论凭主路径同指纹推（第六节注明了）。
- 第 20447 行传给 `evaluate_a_random_mount` 的主路径账（驱动 `e_content`、`last_confirmed` 这类挂载行字段），这次没查它会不会同样撞键。这 3 格上两臂挂载行的这些字段相同，没有表现出来。
- 单一时间线内部（主路径自己、注入分支自己）会不会复用同一个 `(实例, txg)` 键，没查。
- 执行员报告第一节第 2 条说「实例 2 是在别的时点生的」：实测两条时间线拿到的实例号都是 2、txg 都写到 4；撞的是 txg 相同，不是实例生成时点不同。那句措辞不准，机理结论不变。

## 九、草稿清理与怎么重建

- 删了四份副本（先 `du -sh` 记下大小）：`yi-cfg-repro` 368M、`today-repro` 490M、`yi-cfg-diag` 369M、`today-diag` 491M，都在 `/tmp/claude-1000/investigate-e158-seg2-ledger/` 下。删之前核过没有进程在里面跑（`ps … | grep` 输出 `0`）。
- 留着的是报告、各次输出（`*.out`、`*.err`，sha256 在 `outputs.sha256`）、`run_one.sh`、`patch_diag.py`、`fielddiff.py`、`diag-final.diff`、`progress.md` 与编译日志，都是几 MB 以内的文本。
- 要重建最小复现：先 `cp -a /tmp/claude-1000/e158-r4-seg2/arms/yi-cfg $S/yi-cfg-diag`，再 `patch $S/yi-cfg-diag/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs $S/diag-final.diff`，然后在副本里 `capped.sh 2 cargo build --release --offline --bin e158_root_choice_repair`，最后跑第三节那条命令。这个 diff 在臂副本 bin（sha256 `12b82f49…`）上用 `patch -o` 试打过：rc=0，打出来的文件里 3 处 `E158_INVESTIGATE`。
