# m2-supp3-item1-code-r1 · 云端攻方腿（Opus）报告：Z2 遮蔽、Z3 漏判、Z4 判别力余量

时刻按 UTC 记（东京 = UTC+9）。15:33Z 开工，16:25Z 前后撞周限额停，19:00Z 按主 agent 消息续做（第 1 节末「中途收到的消息」）。
所有变异、历史与改法都只施加在仓副本上（`/tmp/claude-1000/m2-supp3-item1-r1-opus/slot-*/`，各用自己的 target）；副本上量出的数不算入库装置上的数。主工作区 `crates/` 一个字没改：开工快照里 8 个 `crates/` 文件 19:0xZ 复核仍 `OK`（快照里另两份 kb `02-second-txn.md`、`16-发布语义.md` 此刻 `FAILED`，是别人改的，不是本腿；下文引这两份的行号是 19:0xZ 现查的）。

## 〇、各格判定一览

| 格 | 判定 | 最硬的一格（下文小节） |
|---|---|---|
| Z2 遮蔽（第 1 条） | **打中** | 一段没有任何回退的历史（挂载、覆盖写 3 次、抬 F 到 3），基线跑完；回收门槛差一（A1、A2、A8）或 checker 候选集差一（A5）的变异下，失败全部被已知红第 1 条接走（一-1）。快档 96 段：A1 下第 1 条从 1 段涨到 18 段，其中 17 段 F 那个 txg 上是有效根、不在任何空档里；新发现 0。大档两档（1000×60、3000×40）A1、A2、A5、A8 全绿 |
| Z2 遮蔽（第 0 条） | **打中** | 同一个 A1 下，种子 3 基线「跑完」，变异下在一次抬 F 之后以已知红第 0 条收尾（一-2）；根环槽号取模多 1（A7：txg 24–26 的根写到环外）快档新发现 0、第 0 条 50 → 54 段 |
| Z3 漏判 | **打中** | 复用改写已回收记录时不改分配代（N2）：快档、大档两档全绿，逐字相同于基线；探针证实快档 96 段里 18 段、大档 3000×40 里 812 段走到那一格（二-1）。另有四类（抬 F 上限、冷启动读回失败、合法回退被拒、装得下的内容被拒）两档都绿（二-2 至二-5） |
| Z3（「没发写不跑 checker」那一半） | 没打中 | 每一步都跑 checker 与跳过，1000 段 × 30 步在基线与 N1 下逐段收尾相同（二-7） |
| Z4 判别力余量（同类变异） | **打中** | N2 与第 41 行同一处（复用时改写记录那三行），快档不红（二-1、三-2）；N5（与第 121 行同类：让位漏掉起点在落点之前、跨进落点的记录）快档与大档都不红，但探针 4096 段一次都没走到那一格（三-3，不分辨） |
| Z4（几何取样点上第 130 行不红） | 字面没打中；余量量到 | 18 个几何点上第 130 行每点都「红」，但其中 3 个点（每段 20 步；种子 [96, 192)；种子 [2496, 2592)）红的是基线自己的路径断言（「复用时一条被罩住的已释放记录都没删过」），变异那一类失败 0 段——那三个点上快档在基线上就是红的。3000 个种子里第 130 行每种子判出率 101 / 3000；按 96 一窗切 31 窗，0 次的 2 窗、1 次的 4 窗（三-1） |

四句（分不分辨臂、判别子看不看得到、满足哪一分句、改法在打中的格上还中不中）逐条写在各小节末。

## 复跑

模型目录 `research/prompts/m2-supp3-item1-code-r1-opus-model/`（全部文件与 `logs/` 下每份日志的 sha256 在同目录 `SHA256SUMS`，172 行；`SHA256SUMS` 自己的 sha256 `f0a261ee1370010bfafb548366b9c94e8c74708af1c7dc2af12295b3f9178a5a`）。起副本、跑的命令：

```
D=/tmp/claude-1000/m2-supp3-item1-r1-opus; M=research/prompts/m2-supp3-item1-code-r1-opus-model
rsync -a --exclude target --exclude .git ./ $D/pristine/            # 开工快照的副本
mkdir -p $D/slot-A && rsync -a $D/pristine/{Cargo.toml,Cargo.lock,crates} $D/slot-A/
python3 $M/run-mutants.py --slot $D/slot-A --target $D/target-A --logs $D/logs-A --ids BASE,A1,N2 --large 1000x60,3000x40   # 快档 + 两档大档，release（根 Cargo.toml 的 release 开着 overflow-checks；core / checker 里没有 debug_assert）
python3 $M/run-mutants.py ... --table $M/probes.tsv --ids PN2,PB2b,PN5,PA3,PA4                                            # 够达探针
cp $M/opus_gap_probe.rs $M/opus_probe.rs $D/slot-D/crates/singlefs-harness/tests/ && bash $M/run-gap.sh $D/slot-D $D/target-D $D/gap 96 30 BASE A1   # 第 1 条的机理核对
cp $M/opus_z2_history.rs $D/slot-E/crates/singlefs-harness/tests/ && bash $M/run-z2.sh $D/slot-E $D/target-E $D/z2.out BASE A1 A2 A5 A8
cp $M/opus_z3_history.rs $D/slot-E/crates/singlefs-harness/tests/ && bash $M/run-z3.sh $D/slot-E $D/target-E $D/z3.out BASE B3 B5 B6 B4
python3 $M/geometry-env-patch.py $D/slot-C && python3 $M/run-geometry.py --slot $D/slot-C --target $D/target-C --out $D/geometry.tsv --ids BASE,R121 --points "seeds=96,ops=20;first=96,seeds=96,ops=30"
python3 $M/proposal-patch.py $D/slot-G && python3 $M/run-mutants.py --slot $D/slot-G --target $D/target-G --logs $D/logs-G --ids BASE,A1,B6,B1 --large 1000x60,3000x40
python3 $M/make-diffs.py $D/pristine $M/mutants.tsv $M/rows-41-121.tsv $M/probes.tsv > mutant-diffs.patch                     # 每条变异、探针在副本上的 diff
```

主要文件的 sha256：

| 文件 | sha256 |
|---|---|
| `mutants.tsv`（18 条变异） | `1690eb4a3fb059399d9444ceb677163fdd3b1d7078ff8572d6125c37cb1c610a` |
| `probes.tsv`（6 条够达探针） | `fe93cdfe972245615f9840fcede2b3ffe75253046870a38374e4da43527be8d8` |
| `rows-41-121.tsv`（第 41、121 行原样取） | `3b608ed17de7718c14dc4bdee5b141d6480edba41c3893b1f0c74b859889d22f` |
| `mutant-diffs.patch`（上面三张表 27 条在副本上的 diff） | `1c14dcd1f21d4ebd629159ac301525c2a36fc35f058bef39efa3ec5775ff63bf` |
| `run-mutants.py` | `8487191e5106081be143a658cab8582368dbe2771b3b0817f422349fe226b125` |
| `run-probe.py` | `5e0bc3ce95be2f224bf7bcfe8b380f8c6e93415caf476a9e3d79a10168667437` |
| `run-geometry.py` | `a978ae95475ebb052ba4a666f207c570f0a3a0edd3a2f329350bd5c257be69d0` |
| `geometry-env-patch.py` / `geometry-env.diff` | `c3100e115526b9491ff652a7633b3c0b8a0acda17cfb6b9bdbcb5c7ca98bfa98` / `d9650e0d37c5e4cb0392391cafa2be4a32f6249f0bf8554a04a46f7291c78eca` |
| `always-check.diff`（每一步都跑 checker 的开关） | `1fa1d03f1c5c576ccf3ce613524d33220e8d9cffd2937817fb9f397f5825c3e6` |
| `opus_probe.rs` / `opus_gap_probe.rs` | `0033ebe6b1e223700b349592407a56d6ebbcd185c45580be435558f4f63fba0c` / `4be64993e80288b14f76c35069221cd59feb85f47f1142dae469406ea66562e0` |
| `opus_z2_history.rs` / `opus_z3_history.rs` | `f9ae6e803339e0f1545351bb8c673b35916edbad026dea4911f9c1d94d4573d4` / `547902c9a04e8924a10fc1d581bbad0aaac64eafc035ab751ea2ecb505971dd6` |
| `run-gap.sh` / `run-z2.sh` / `run-z3.sh` | `4b8de086f66a2e737cf8957a723caa492ef1e0d4a5b833887ec531beecd557b7` / `ff4936376cd0808a6802c4522ec62b4e3c87f39578a4ee7b48c3e5e1860abcf6` / `7c737fcbbf9d6628e8019f9a50258e7febeb9f5e243c21426a7e6c4dffcef365` |
| `proposal-patch.py` / `proposal.diff` | `2e50590e92808907f14dd7206da3cc82fbb91ffa6fea90cfaa0b738faaeb49c2` / `ed110d014cc9e9c9d960a6ba1dc22d156411658013cad533d280beee48737f55` |
| `make-diffs.py` | `330340312c2bf558472f17a674ed5c60e6ba6a0ff351d08f94b0c9931602616a` |

原样日志在 `logs/`（汇总表 `summary-*.tsv`、`geometry.tsv`、`gap-*.log`、`probe*-*.log`、`z2-history.out`、`z3-history.out`）与 `logs/raw/`（每条变异每一档的整份 cargo 输出）。下文贴的行都从这些文件里原样取。

**档与 profile**：扫描用 release（`--profile release`），因为根 `Cargo.toml` 的 `[profile.release]` 写着 `overflow-checks = true`，`grep -rn debug_assert crates/singlefs-core/src crates/singlefs-checker/src` 0 条；N2、B3、B6 三条另在 dev profile（门禁跑的那一档）上复跑快档，三条都 `test result: ok`（`logs/raw/logs-dev/`），与 release 同。

**作废过的数**：`run-gap.sh`、`run-z2.sh` 第一版还原文件用 `mv` 没 `touch`，连跑跨 crate 的变异时 cargo 没重编被还原的 crate：gap 探针的 A5（96×30）、A8（96×30）、A2（1000×30）第一轮与 z2 历史第一轮的 A5、A8 行是两条变异叠在一起跑的，已改名 `stale-*` / `*-first-run-A8-stale.out` 留着，脚本加了 `touch` 之后重跑，下文只用重跑的数（`logs/gap-run.out` 里有这两句作废说明）。

## 一、Z2 遮蔽

判据原文（`research/prompts/_m2-supp3-item1-code-r1-background.md` 第 44 行，Z2 一格的「触发的观测」）：「构造一段历史或一条对 `crates/singlefs-core`、`crates/singlefs-checker` 的变异，它的失败被第 0 或第 1 条匹配、而根因不是登记的那一类（例：不经回退的抬 F 之后 I-3.1 记账多算，被第 1 条接走）」。

被判对象里许可它的那几句（`crates/singlefs-harness/src/history.rs`，开工快照 `5e715bbe…`）：

- 第 751 行：`/// 只看「抬 F 那一步之后、根环没转圈、只有 I-3.1 红且记账多于遍历」，不看 F 是否落在回退留下的空档里：观察里没有回退历史。`
- 第 755、756 行：`    observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)` / `        && !observation.root_ring_has_turned()`——第 1 条的全部条件，外加 `only_allocated_statistic_above_walked`。
- 第 748 行：`    observation.root_ring_has_turned() && only_allocated_statistic_above_walked(observation)`——第 0 条，哪一步之后都算。
- 测试文件 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 第 305 行：大档只断言 `        report.new_findings.is_empty(),`，没有快档那组「路径跑到了」。

### 一-1　第 1 条接走「不经回退的抬 F 之后 I-3.1 记账多算」

**历史**（`opus_z2_history.rs`，起点「第一个文件之后」）：可写挂载（实例 2，写行 txg 4、暖机 txg 5）、覆盖写三次（txg 6、7、8）、抬 F（选择子 3 ⇒ 0 + 3 mod (8 − 0 + 3) = 3）。一次回退都没有，txg 3 那条是第一个文件的有效根，不在任何空档里。每一步指到的许可：挂载与覆盖写走 `history.rs` 第 1357、1250 行的 `apply_mount_writable` / `apply_publish_overwrite`，抬 F 的目标取法在第 1420、1421 行（`[今天的 F, 现行 txg + 2]`），生成器自己会抽到这一段（比重表第 286–294 行「开着、有文件」那张里有覆盖写与抬 F）。

**变异**（副本上的 diff，全文在 `mutant-diffs.patch`）：

```
### A1：回收门槛 ≤ 写成 <（重建与抬 F 共用的 reclaim_released_up_to）
-            .filter(|record| record.is_released && record.generation <= floor)
+            .filter(|record| record.is_released && record.generation < floor)
### A2：抬 F 回收少一代（只改 raise_rollback_floor 的调用处）
-        reclaim_floor(new_floor, oldest_valid_root),
+        reclaim_floor(CheckpointTxg(new_floor.0.saturating_sub(1)), oldest_valid_root),
### A5：checker 候选集把 txg = F 的根也挡在外面
-            let below_floor = root.checkpoint_txg < newest_rollback_floor;
+            let below_floor = root.checkpoint_txg <= newest_rollback_floor;
### A8：回收门槛取 min（F 与环里最旧有效根取小的）
-    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
+    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.min(oldest))
```

（原文分别在 `allocator.rs` 第 896 行、`mount.rs` 第 659 行、`walk.rs` 第 1376 行、`mount.rs` 第 278 行，各命中一次。）

**命令**：`bash $M/run-z2.sh $D/slot-E $D/target-E $D/z2-history.out BASE A1 A2 A5 A8`。**原样输出**（`logs/z2-history.out` 的收尾行）：

```
== BASE
OPUS-Z2 ending Completed
== A1
OPUS-Z2 ending KnownRed form 1 at Operation(4) after Some(RaiseRollbackFloor) newest_txg Some(10) violations [("I-3.1", "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040")]
== A2
OPUS-Z2 ending KnownRed form 1 at Operation(4) after Some(RaiseRollbackFloor) newest_txg Some(10) violations [("I-3.1", "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040")]
== A5
OPUS-Z2 ending KnownRed form 1 at Operation(4) after Some(RaiseRollbackFloor) newest_txg Some(10) violations [("I-3.1", "盘 0：记账的已分配 Some(983040)，遍历全部有效根得到 884736")]
== A8
OPUS-Z2 ending KnownRed form 1 at Operation(4) after Some(RaiseRollbackFloor) newest_txg Some(10) violations [("I-3.1", "盘 0：记账的已分配 Some(999424)，遍历全部有效根得到 983040")]
```

基线那一步 `reclaimed_placements: 1`（mkfs 那片第 0 版树表，释放代 3 = F），A1 下 `reclaimed_placements: 0`，差的正是那 1 槽 16384 字节；登记的第 43 行机理（F 落在被抛弃实例的 txg 上、回退目标根独占的 6 槽）一格都不在这段历史里。

**在生成器自己的种子上**（快档与大档，命令 `run-mutants.py --ids BASE,A1,A2,A5,A8 --large 1000x60,3000x40`，`logs/summary-fast-A.tsv`、`summary-large-A.tsv`）：

```
BASE fast exit 0 历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
A1 fast exit 101 历史 96 段：跑完 33、以已知红收尾 {0: 45, 1: 18}、新发现 0；根环转过一圈的 46 段；最高 txg 28
A1 large 1000x60 exit 0 历史 1000 段：跑完 277、以已知红收尾 {0: 513, 1: 210}、新发现 0；根环转过一圈的 514 段；最高 txg 28
A1 large 3000x40 exit 0 历史 3000 段：跑完 876、以已知红收尾 {0: 1485, 1: 639}、新发现 0；根环转过一圈的 1498 段；最高 txg 28
A5 large 3000x40 exit 0 历史 3000 段：跑完 875、以已知红收尾 {0: 1370, 1: 755}、新发现 0；根环转过一圈的 1383 段；最高 txg 28
A8 large 3000x40 exit 0 历史 3000 段：跑完 876、以已知红收尾 {0: 1485, 1: 639}、新发现 0；根环转过一圈的 1498 段；最高 txg 28
```

A2 与 A1 两档逐字相同。快档 exit 101 **不是**分类判出来的：新发现 0，红在测试文件第 67 行那条路径断言（`logs/raw/logs-fast/A1-fast.log` 原样）：

```
thread 'random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation' (695934) panicked at crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:67:5:
一条已释放的记录都没被复用
```

（A8 红在第 66 行「抬 F 一次都没回收到落点」，A5 同 A1 红在第 67 行。）大档没有这组断言，四条变异两档都 `exit 0`。

**机理核对**：`opus_gap_probe.rs` 对每段以第 1 条收尾的历史，取抬 F 之前最后一份镜像，按实现的读法（`singlefs_core::recovery` 的 `readable_roots` / `choose_root` / `instance_table_of_root`）看新 F 那个 txg 上的根是被抛弃的还是有效的。`logs/gap-run.out` 原样（作废两行见第〇节）：

```
A1 GAP-SUMMARY first 0 seeds 96 ops 30: form1 18 in_gap 1 not_in_gap 17
A2 GAP-SUMMARY first 0 seeds 96 ops 30: form1 18 in_gap 1 not_in_gap 17
A8 GAP-SUMMARY first 0 seeds 96 ops 30: form1 18 in_gap 1 not_in_gap 17
A2 GAP-SUMMARY first 0 seeds 1000 ops 30: form1 196 in_gap 9 not_in_gap 187
A1 GAP-SUMMARY first 0 seeds 1000 ops 30: form1 196 in_gap 9 not_in_gap 187
A5 GAP-SUMMARY first 0 seeds 96 ops 30: form1 20 in_gap 1 not_in_gap 19
```

基线 3000 × 40（`logs/gap-BASE-3000x40.log`）：`GAP-SUMMARY first 0 seeds 3000 ops 40: form1 30 in_gap 30 not_in_gap 0`——基线上第 1 条接的 30 段全在空档里，登记的机理在基线上成立；变异下多出来的全是空档外的。A2 下 196 段按回退次数分：0 次回退 98 段（全不在空档）、1 次 70 段（4 段在空档）、2 次 27 段（5 段在空档）、3 次 1 段（`logs/gap-A2-1000x30.log` 按 `rollbacks_ok` 与 `root_at_F` 两列数）。A1 快档第 1 条的 18 段里一行原样：`GAP seed 7 step 15 F 10 rollbacks_ok 0 root_at_F valid(instance 2) violations [("I-3.1", "盘 0：记账的已分配 Some(1540096)，遍历全部有效根得到 1474560")]`。

### 一-2　第 0 条接走不是「根环转圈」的记账多算

同一个 A1，逐种子探针（`opus_probe.rs`，`run-probe.py --id BASE / A1 --env OPUS_SEEDS=96 --env OPUS_OPS=30`，`logs/probe96-BASE.log`、`logs/probe96-A1.log`）原样：

```
/tmp/claude-1000/m2-supp3-item1-r1-opus/probe96-BASE.log:OPUS seed 3 rollbacks_ok 0 raises_ok 2 :: Completed
/tmp/claude-1000/m2-supp3-item1-r1-opus/probe96-A1.log:OPUS seed 3 rollbacks_ok 0 raises_ok 1 :: KnownRed0 at Operation(24) after Some(RaiseRollbackFloor) newest_txg Some(28) violations [("I-3.1", "盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2097152")]
/tmp/claude-1000/m2-supp3-item1-r1-opus/probe96-A1.log:OPUS seed 37 rollbacks_ok 1 raises_ok 2 :: KnownRed0 at Operation(15) after Some(RaiseRollbackFloor) newest_txg Some(25) violations [("I-3.1", "盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2031616")]
/tmp/claude-1000/m2-supp3-item1-r1-opus/probe96-A1.log:OPUS seed 65 rollbacks_ok 1 raises_ok 2 :: KnownRed0 at Operation(20) after Some(RaiseRollbackFloor) newest_txg Some(25) violations [("I-3.1", "盘 0：记账的已分配 Some(2670592)，遍历全部有效根得到 2654208")]
```

种子 3 在基线上跑完（转过根环也没红），A1 下在一次抬 F 之后以第 0 条收尾：失败的根因是回收门槛差一，第 0 条只看「转过圈 ∧ 只有 I-3.1 多算」就接走（第 748 行）。种子 37、65 在基线上本来也以第 0 条收尾，但红在更晚的一步、在挂载之后；A1 下提前到抬 F 那一步——同样被第 0 条接走。

**A7（根环槽号取模多 1，`root_ring.rs` 第 36 行）**：

```
-        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,
+        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % (ROOT_RING_SLOTS_PER_REGION + 1),
```

按取模推（没逐槽核盘面）：txg 24–26 写到每区第 9 个槽、环外，txg 27 起盖 txg 0 那一槽。原样（`logs/summary-F.tsv`）：

```
A7 fast exit 101 历史 96 段：跑完 41、以已知红收尾 {0: 54, 1: 1}、新发现 0；根环转过一圈的 55 段；最高 txg 40
A7 large 1000x60 exit 101 历史 1000 段：跑完 278、以已知红收尾 {0: 676, 1: 15}、新发现 31；根环转过一圈的 683 段；最高 txg 44
A7 large 3000x40 exit 101 历史 3000 段：跑完 941、以已知红收尾 {0: 1957, 1: 36}、新发现 66；根环转过一圈的 1992 段；最高 txg 52
```

快档新发现 0、第 0 条 50 → 54 段，红在测试文件第 75 行的路径断言（`复用时一条被罩住的已释放记录都没删过`）；大档判得出（新发现是 `["I-2.1", "I-4.8", "I-7.4"]` 一类，在可写挂载之后）。快档里多出来的 4 段第 0 条是 A7 的失败被当成转圈接走。

### 一-3　四句

1. **分不分辨臂**：跑前条款给 Z2 的改法是「收窄匹配条件或补判定」（背景第 50 行）。打中的格分辨这两个改法里的形态：按「新 F 那个 txg 上的根是被抛弃的」收窄第 1 条，一-1 的格大多转成新发现，一-2 的格一格都碰不到（见下表）。
2. **判别子看不看得到**：看得到。第 1 条要分的「F 在空档里 / 不在」在抬 F 之前的镜像上逐条可读（实例表行与根环），探针就是这么读的；实现员报告第 231 行说「从 `FailureObservation` 里拿不到回退历史」，但不需要回退历史，镜像够。第 0 条要分的「转圈留下的槽 / 别的原因多算」在单一镜像上不一定分得开（两者都是已释放、没回收、没有候选根引用），这一半判别子是否看得到没核。
3. **满足哪一分句**：Z2 触发观测的「一条对 `crates/singlefs-core`、`crates/singlefs-checker` 的变异，它的失败被第 0 或第 1 条匹配、而根因不是登记的那一类」，以及括号里的例子逐字（一-1 那段历史）。
4. **改法在打中的格上还中不中**：

| 改法 | 一-1 的格（第 1 条接走） | 一-2 的格（第 0 条接走） | 基线 |
|---|---|---|---|
| P1 收窄第 1 条：抬 F 之前的镜像上新 F 那个 txg 的根全是被抛弃的才算（`proposal-patch.py` 的 P1） | **量过**：A1 快档第 1 条 18 → 1 段、新发现 0 → 17；大档 1000×60 第 1 条 210 → 12、新发现 198；3000×40 第 1 条 639 → 28、新发现 611（`logs/summary-G-proposal.tsv`）。剩下的 1 / 12 / 28 段是 F 恰好落在空档里、变异多算的槽与登记机理叠在一起的格：对这几格不起作用 | **量过**：不起作用（A1 快档第 0 条 45 段不变） | **量过**：三档与原代码逐字相同（`{0: 50, 1: 1}` / `{0: 705, 1: 13}` / `{0: 1929, 1: 30}`，新发现 0） |
| 补判定：第 0、1 条按「多算的量 = 登记机理算出来的那几个落点的跨度和」匹配（例：第 1 条 = 回退目标根独占、释放代 > F 的落点） | 推的：会接住 P1 剩下的叠加格（多算量不等） | 推的：会接住一-2（种子 3 多算 98304 字节，转圈机理在那一刻应为 0）；没实现、没跑 | 推的 |
| 大档加快档那组路径断言 | 推的：只让大档在 A1 这类「每次回收都红」的变异上红，不改分类 | 推的：同左 | 没跑 |

P1 只在本腿的副本上量过、被攻过零轮；副本 diff `proposal.diff`（139 行，含 P2、P3）。

**推翻条件**：在入库装置上重做，A1 下以第 1 条收尾的种子里 `root_at_F` 是被抛弃根的占多数；或者基线上出现 `not_in_gap` 的第 1 条收尾（那说明第 1 条在基线上就接了空档外的东西，本节的「变异才造出空档外的格」要改写）。

## 二、Z3 漏判

判据原文（背景第 45 行，Z3 的触发观测）：「一条落在生成器走得到的路径上的变异（`crates/singlefs-core` 或 `crates/singlefs-checker`），快档与大档都不红」。每一格先用够达探针（`probes.tsv`：在变异会改变行为的那一刻 panic）证明生成器走到了那一格，再报变异下两档的结局。

### 二-1　N2：复用改写已回收记录时不改分配代（**打中**）

```
### N2：复用改写已回收记录时不改分配代
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -539,7 +539,6 @@
-            existing.generation = generation;
```

（`allocator.rs` 第 542 行。）盘上那条分配记录复用之后仍写着旧的释放代，而 D3（空间分配）第 116 行：`**独立 keyspace 的 btree，key = 落点（设备身份 + 设备内偏移），value = 分配代。**`

**走到了**（探针 PN2：旧代与这次分配代不同就 panic；`logs/summary-probe-P.tsv`、`logs/raw/logs-probe/PN2-fast.log` 原样）：

```
PN2	fast	101	历史 96 段：跑完 33、以已知红收尾 {0: 44, 1: 1}、新发现 18；根环转过一圈的 47 段；最高 txg 28
新发现 Panic { location: "crates/singlefs-core/src/allocator.rs:542" }：第一个种子 3（同签名的种子 [3, 7, 15, 20, 26, 27, 38, 46, 51, 54, 56, 57, 62, 66, 68, 69, 71, 94]），在 Operation(25)（Some(PublishOverwrite)）之后
  panic 在 crates/singlefs-core/src/allocator.rs:542：opus-probe-N2 old CheckpointTxg(5) new CheckpointTxg(29)
```

大档 3000×40 上 812 段走到（`PN2	large 3000x40	101	历史 3000 段：跑完 885、以已知红收尾 {0: 1275, 1: 28}、新发现 812；…`）。

**两档都不红**（命令 `run-mutants.py --ids N2 --large 1000x60,3000x40`；`logs/summary-large-B.tsv` 原样；dev profile 复跑 `logs/raw/logs-dev/N2-fast.log` 末行 `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 35.54s`）：

```
N2 fast exit 0 历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
N2 large 1000x60 exit 0 历史 1000 段：跑完 282、以已知红收尾 {0: 705, 1: 13}、新发现 0；根环转过一圈的 711 段；最高 txg 61
N2 large 3000x40 exit 0 历史 3000 段：跑完 1041、以已知红收尾 {0: 1929, 1: 30}、新发现 0；根环转过一圈的 2102 段；最高 txg 50
```

三行与基线逐字相同。仓里别的用例判得出（副本上跑 `cargo test --release -p singlefs-harness --test second_transaction_step_five_reuse`，`logs/n2-step-five.log`）：`test raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it ... FAILED`，红在那个文件第 198 行的逐字段断言——随机历史这一路判不出，是因为 29 条不变量里没有一条看已分配记录的分配代（I-3.9 只看带已释放标志的记录）。

四句：分辨臂——跑前条款「收窄匹配条件或补判定」，收窄碰不到（没有任何已知红被触发），只有补判定碰得到；判别子看得到——分配代在镜像的分配记录树里，与那个落点上单元头的诞生 txg 可比（推的，没实现）；满足 Z3 触发观测的「落在生成器走得到的路径上」「快档与大档都不红」两个分句；它同时是 Z4 的「与第 41 行同类的变异快档不红」（第 41 行改的就是这三行，三-2）。

### 二-2　B3 / B5：抬 F 的上限算错，F 可以抬过「第 4 新的非空持久有效根」（**打中**）

```
### B3：抬 F 上限不取第 4 新的非空根（只取每盘最新有效根）
-    Some(newest_on_every_device.min(fourth_newest))
+    Some(newest_on_every_device.max(fourth_newest))
### B5：非空判定恒真（空发布也算非空）
-    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
+    true || root_pointers.inode_tree != previous_valid_root_pointers.inode_tree
```

（`mount.rs` 第 575、477 行。）条款：`.claude/kb/decisions/16-发布语义.md` 第 374 行「抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)…」（整行太长，行号 `grep -nF '| 抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)'` 现取）。

手写历史（`opus_z3_history.rs` 的 ceiling 段：挂载、覆盖写三次、抬 F 到 7、再回退到 txg 6 那条根），`bash run-z3.sh … BASE B3 B5 B6 B4`，`logs/z3-history.out` 原样：

```
== BASE
OPUS-Z3 ceiling step 4 RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 7 }) -> Refused { member: "MountError::RollbackFloorAboveCeiling" }
OPUS-Z3 ceiling step 5 CloseAndMountRollback(RingRoot { index_from_newest: 4 }) -> Applied(Mounted { instance: InstanceGeneration(3), publishes: 2 })
OPUS-Z3 ceiling ending Completed
== B3
OPUS-Z3 ceiling step 4 RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 7 }) -> Applied(RaisedFloor { new_floor: CheckpointTxg(7), publishes: 2, reclaimed_placements: 26, reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 0 } })
OPUS-Z3 ceiling step 5 CloseAndMountRollback(RingRoot { index_from_newest: 4 }) -> Refused { member: "MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）" }
OPUS-Z3 ceiling ending Completed
```

基线拒掉抬到 7、之后回退到 txg 6 成功；B3 下抬 F 成功、回收 26 个落点，本该还能回退的 txg 6 被拒——最少保留 4 个状态的承诺丢了，两段都以「跑完」收尾。（B5 在这一段上与基线同，B5 的格在种子上：快档 `Err 成员 MountError::RollbackFloorAboveCeiling` 122 → 60 次。）

两档（`logs/summary-fast-B.tsv`、`summary-large-B.tsv`）：

```
B3 fast exit 0 历史 96 段：跑完 58、以已知红收尾 {0: 36, 1: 2}、新发现 0；根环转过一圈的 64 段；最高 txg 52
B3 large 1000x60 exit 0 历史 1000 段：跑完 342、以已知红收尾 {0: 600, 1: 58}、新发现 0；根环转过一圈的 667 段；最高 txg 86
B3 large 3000x40 exit 0 历史 3000 段：跑完 1424、以已知红收尾 {0: 1408, 1: 168}、新发现 0；根环转过一圈的 1983 段；最高 txg 67
B5 fast exit 0 历史 96 段：跑完 50、以已知红收尾 {0: 44, 1: 2}、新发现 0；根环转过一圈的 64 段；最高 txg 52
B5 large 3000x40 exit 0 历史 3000 段：跑完 1281、以已知红收尾 {0: 1592, 1: 127}、新发现 0；根环转过一圈的 2016 段；最高 txg 67
```

B3 快档里 `RollbackFloorAboveCeiling` 从 122 次降到 39 次、抬 F 回收落点从 852 涨到 3332（`logs/raw/logs-fast-B/B3-fast.log`），生成器大量走到那一格。仓里别的用例判得出：同一副本 `second_transaction_step_five_reuse` 四条红（`logs/b3-step-five.log`：`test result: FAILED. 7 passed; 4 failed; …`，含 `raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused`）。

四句：分辨臂——只有补判定碰得到；判别子：`F 抬过上限` 在镜像上看得到（上限的定义全从根环与树表来），而 checker 没有这条判定，是收口表已登记的欠账：`.claude/kb/milestone/02-second-txn.md` 第 338 行（第 26 行「checker：认「非空持久有效根」与 F 抬过上限要红的检查…」）；另一半「这次 `Ok` 该不该是 `Err`」是第 2 件模型的事。满足 Z3 两个分句。

### 二-3　B6：冷启动读回失败被记成合法结局（**打中**；B4 同形、是背景已声明的射程外）

```
### B6：补齐字节从最后一个载荷字节算起（冷启动读回：内容末字节非零就报 I-2.3「补齐字节非零」）
-    if bytes[payload_end..].iter().any(|byte| *byte != 0) {
+    if bytes[payload_end - 1..].iter().any(|byte| *byte != 0) {
```

（`unit.rs` 第 520 行；`data_unit_payload` 在 `singlefs-core` 里只有 `recovery.rs` 第 1325 行一处调用，只冷启动读回走它。）许可它的那几句：`history.rs` 第 1447–1453 行 `apply_cold_start_recover` 对 `recover` 的任何结局都返回 `StepOutcome::Applied(AppliedEffect::Recovered { … })`（第 1450 行 `    StepOutcome::Applied(AppliedEffect::Recovered {`）；冷启动不写盘，第 1565 行 `            if stream_length == checked_stream_length {` 让这一步连 checker 都不跑；快档只断言 `        count_of(&tally.recovery_outcomes, "FileRead") >= 1,`（测试文件第 99 行）。

手写历史（第一个文件之后冷启动一次）`logs/z3-history.out` 原样：

```
== BASE
OPUS-Z3 cold step 0 ColdStartRecover -> Applied(Recovered { outcome: "FileRead" })
OPUS-Z3 cold ending Completed
== B6
OPUS-Z3 cold step 0 ColdStartRecover -> Applied(Recovered { outcome: "Failed(RecoveryFailure::InvariantViolated（I-2.3）)" })
OPUS-Z3 cold ending Completed
```

两档（`logs/raw/logs-F/B6-*.log` 原样，dev profile 复跑快档 `test result: ok`）：

```
历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
  冷启动结局 Failed(RecoveryFailure::InvariantViolated（I-2.3）)：72 次
  冷启动结局 FileRead：6 次
  冷启动结局 NoFile：19 次
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 17.19s
历史 3000 段：跑完 1041、以已知红收尾 {0: 1929, 1: 30}、新发现 0；根环转过一圈的 2102 段；最高 txg 50
  冷启动结局 Failed(RecoveryFailure::InvariantViolated（I-2.3）)：2222 次
  冷启动结局 FileRead：273 次
  冷启动结局 NoFile：1509 次
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 555.26s
```

基线三档 `Failed(...)` 一次都没有（快档 `FileRead：78`、`NoFile：19`；3000×40 `FileRead：2495`、`NoFile：1509`）——这个结局在合法镜像上是干净的信号，生成器看见了、记下了、没判。B6 下 97 次冷启动里 72 次读不回已提交的文件，快档照绿，因为剩下 6 次（多半是 0 字节内容）满足了「至少一次 FileRead」。

四句：分辨臂——「补判定」碰得到，「收窄」碰不到；判别子看得到（`RecoveryOutcome::Failed` 就在结局里，不用模型）；满足 Z3 两个分句；与背景第 24 行「冷启动 `recovery::recover` 读回的内容不和任何东西比，那是第 2 件模型的事」不冲突——那句说的是**内容**不比（B4：读回字节倒序，三档与基线逐字相同，那一格才归第 2 件），读回**失败**不需要模型。改法 P2（读回 `Failed(…)` 算失败，签名「冷启动读回」），**量过**：B6 快档新发现 45、1000×60 新发现 445、3000×40 新发现 1333；基线三档与原代码逐字相同（`logs/summary-G-proposal.tsv`）。只在本腿副本上量过、被攻过零轮。

### 二-4　B2：回退到 txg = F_生效 的合法目标被拒（**打中**，大档走到、两档都绿）

```
### B2：回退到 txg = F_生效 的根也拒
-    if target.checkpoint_txg < effective_floor {
+    if target.checkpoint_txg <= effective_floor {
```

（`mount.rs` 第 1203 行；条款 `.claude/kb/decisions/16-发布语义.md` 第 376 行「回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效」。）够达探针 PB2b（目标 txg = F_生效 且 F_生效 > 0 就 panic）：快档 0 段走到，大档 3000×40 走到 31 段、1000×60 走到 10 段（`PB2b	large 3000x40	101	历史 3000 段：跑完 1033、以已知红收尾 {0: 1906, 1: 30}、新发现 31；…`；一行原样 `  panic 在 crates/singlefs-core/src/mount.rs:1203：opus-probe-B2b target RollbackTarget { instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(22) } floor CheckpointTxg(22)`）。变异下：

```
B2 fast exit 0 历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
B2 large 1000x60 exit 0 历史 1000 段：跑完 282、以已知红收尾 {0: 704, 1: 14}、新发现 0；根环转过一圈的 710 段；最高 txg 58
B2 large 3000x40 exit 0 历史 3000 段：跑完 1047、以已知红收尾 {0: 1922, 1: 31}、新发现 0；根环转过一圈的 2101 段；最高 txg 50
```

大档计数与基线不同（历史走了别的路）、两档都绿。四句：这一格是「入口返回 `Err` 算合法结局」（`history.rs` 第 964 行 `            StepOutcome::Applied(_) | StepOutcome::Refused { .. } => true,` 这一类）——生成器看不到「这次拒绝该不该」，判别子在第 2 件的模型里（背景第 24 行同一段的射程）；两个改法都碰不到；按 `evidence-discipline.md`「判据自己也会写错」记「判别子观测不到」，交主 agent 定它算不算 Z3 打中。

### 二-5　B1：装得下的内容被拒（打中，同 B2 一类；有一个不要模型的改法）

`transaction.rs` 第 1248 行 `>` 改 `>=`：正好装满载荷容量的内容报 `ContentExceedsDataUnit`。快档 `ContentExceedsDataUnit` 108 → 177 次、两档都绿：

```
B1 fast exit 0 历史 96 段：跑完 53、以已知红收尾 {0: 43}、新发现 0；根环转过一圈的 59 段；最高 txg 37
B1 large 3000x40 exit 0 历史 3000 段：跑完 1137、以已知红收尾 {0: 1832, 1: 31}、新发现 0；根环转过一圈的 2017 段；最高 txg 49
```

快档的「六种内容长度都进过入口」（测试文件第 94 行 `            "内容长度「{}」没进过入口",`）把 `Err` 也算「进过」（`history.rs` 第 964 行），所以「载荷容量」一次都没发成也过。改法 P3（装得下的四种长度各至少一次 `Ok`），**量过**：B1 快档红在 `装得下的内容长度「载荷容量」一次都没发成`；大档没有这组断言，照绿；基线快档照绿。被攻过零轮。

### 二-6　A3 / A4：回收门槛不看「环里最旧有效根」（两档都绿；「撞已知红就停」让这一格走不到）

```
### A3：挂载重建不看环里最旧有效根（只按 F_生效 回收）
-        reclaim_floor(effective_floor, oldest_valid_root),
+        reclaim_floor(effective_floor, None),
### A4：抬 F 不看环里最旧有效根
-        reclaim_floor(new_floor, oldest_valid_root),
+        reclaim_floor(new_floor, None),
```

（`mount.rs` 第 407、659 行。）两条的快档、1000×60、3000×40 与基线逐字相同（`A3 large 3000x40 exit 0 历史 3000 段：跑完 1041、以已知红收尾 {0: 1929, 1: 30}、新发现 0；根环转过一圈的 2102 段；最高 txg 50`）。够达探针 PA3 / PA4（最旧有效根 > F、且有释放代落在 (F, 最旧有效根] 的已释放记录时 panic）三档都 `新发现 0`：4096 段历史一次都没走到变异会改变回收集合的那一刻。

推的（没实现）：走到那一刻需要一个「最旧有效根 > F、中间有已释放没回收的落点」的镜像，而那些落点只被已经盖掉的根引用、不在候选集里——同一份镜像在上一步 checker 就已经判 I-3.1 多算、根环已转过，历史在那一步以第 0 条收尾。所以这一格是「撞已知红就停」吃掉的，不是比重没抽到；实现员报告第 220 行已写「这一条修好之前，随机历史罩不到「根环转过之后」的那些路径（例如环里最旧有效根接管回收门槛）」。四句：跑前的两个改法都碰不到（收窄第 0 条，历史照样停在第一个红上）；要碰到得让历史撞第 0 条之后接着走、在下一次挂载之后看 I-3.1 回没回绿（基线上挂载重建按最旧有效根回收，推的会回绿；A3 下不会）——这个改法没实现、没量。记「打中，改法碰不到」，射程与收口表第 ② 行（`.claude/kb/milestone/02-second-txn.md` 第 311 行）同一件事。

### 二-7　「没发写就不跑 checker」：没打中

`always-check.diff` 给副本加一个开关（`OPUS_ALWAYS_CHECK=1` 时每一步都跑 checker），同一批 1000 段 × 30 步各跑一次，逐段收尾（含在哪一步、签名、违例文字）`diff` 为空：

```
always-BASE-skip.log:OPUS-SUMMARY first 0 seeds 1000 ops 30: completed 551 known0 439 known1 10 new 0
always-BASE-check.log:OPUS-SUMMARY first 0 seeds 1000 ops 30: completed 551 known0 439 known1 10 new 0
always-N1-skip.log:OPUS-SUMMARY first 0 seeds 1000 ops 30: completed 467 known0 363 known1 10 new 160
always-N1-check.log:OPUS-SUMMARY first 0 seeds 1000 ops 30: completed 467 known0 363 known1 10 new 160
```

`check_pool_image` 只读镜像，没发写的一步镜像逐字节不变，跳过不改结论。这一半只抽了两个臂（基线、N1），一次，按规则记「这一次没打中」。

## 三、Z4 判别力余量

判据原文（背景第 46 行）：「一个只改快档规模或比重的取样点上第 130 行不红，或一条与第 121 行同类的变异快档不红」。

### 三-1　几何取样点：第 130 行每点都红，其中 3 点红的是基线自己的断言

装置：`geometry-env-patch.py` 只把快档的种子起点、种子数、每段步数与两张比重表改成从环境变量取，判定一字不动（`geometry-env.diff`，59 行）。命令 `run-geometry.py --ids BASE,R121,R41 --points …`；`logs/geometry.tsv` 汇总（每格：退出码、第 130 行那一类（`I-5.4` 新发现）判出的种子数、测试红在哪）：

| 取样点 | 基线 | 第 130 行（R121） |
|---|---|---|
| 96 × 30（门禁那一档） | exit 0 | exit 101，新发现 2（种子 54、69） |
| 96 × 20 | **exit 101**，红在「复用时一条被罩住的已释放记录都没删过」 | exit 101，新发现 **0**，红在同一条基线断言 |
| 96 × 25 | exit 0 | exit 101，新发现 1（54） |
| 96 × 35 / 96 × 40 | exit 0 / exit 0 | 新发现 4 / 5 |
| 种子 [96, 192) × 30 | **exit 101**，同上那条断言 | 新发现 **0**，红在同一条基线断言 |
| 种子 [192, 288)、[288, 384)、[384, 480)、[480, 576) × 30 | 四点 exit 0 | 新发现 4、4、**1**、2 |
| 种子 [2496, 2592) × 30 | **exit 101**，同上那条断言 | 新发现 **0**，红在同一条基线断言 |
| 比重（有文件那张）56/12/12/10/4/4/2、48/20/…、52/16/8/14/…、52/16/16/6/… | 四点 exit 0 | 新发现 2、3、3、4 |
| 比重（关着那张）80/12/8、60/32/8 | 两点 exit 0 | 新发现 2、2 |
| 128 × 30 / 64 × 30 | exit 0 / exit 0 | 新发现 2 / **1** |

原样两行（`logs/geometry.tsv`）：

```
BASE	first=2496,seeds=96,ops=30	exit 101	历史 96 段：跑完 48、以已知红收尾 {0: 47, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 40		失败原因：复用时一条被罩住的已释放记录都没删过	抬 F 回收到落点 30 次、共回收 685 个落点；复用已释放的记录 88 条（其中跨度变了 34 条）；已释放的记录被删 0 条
R121	first=2496,seeds=96,ops=30	exit 101	历史 96 段：跑完 48、以已知红收尾 {0: 47, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 40		失败原因：复用时一条被罩住的已释放记录都没删过	抬 F 回收到落点 30 次、共回收 685 个落点；复用已释放的记录 88 条（其中跨度变了 34 条）；已释放的记录被删 0 条
```

每种子判出率（`opus_probe.rs` 在 R121 下跑 3000 个种子 × 30 步，`logs/probe-R121-3000x30.log`；基线同批 `logs/probe-BASE-3000x30.log`）：

```
R121 ['OPUS_SEEDS=3000', 'OPUS_OPS=30'] 0 ['OPUS-SUMMARY first 0 seeds 3000 ops 30: completed 1524 known0 1351 known1 24 new 101']
BASE ['OPUS_SEEDS=3000', 'OPUS_OPS=30'] 0 ['OPUS-SUMMARY first 0 seeds 3000 ops 30: completed 1601 known0 1375 known1 24 new 0']
```

按 96 个种子一窗切成 31 窗（`awk` 按 `int(seed/96)` 数新发现），每窗判出数：`0:2 1:0 2:4 3:4 4:1 5:2 6:1 7:3 8:4 9:5 10:4 11:3 12:1 13:2 14:6 15:6 16:4 17:1 18:5 19:2 20:3 21:6 22:6 23:4 24:2 25:2 26:0 27:4 28:6 29:3 30:4`。0 次的 2 窗（1、26）正是上表基线自己就红的那两窗；1 次的 4 窗（4、6、12、17）。

四句：Z4 字面的「第 130 行不红」在 18 个点上一次都没出现，**字面没打中**。但「红」在 3 个点上来自基线自己的路径断言（`released_records_removed >= 1`，测试文件第 76 行），那几个点上第 130 行那一类失败 0 段；门禁 59 号只看「点名的测试 FAILED」，分不出这两种红。这是「判据写错」四形态里的「判别子观测不到」的邻居：59 号的判据看不到红的原因。改法（推的，没实现）：第 130 行的点名测试改成只看「新发现里有 `I-5.4`」的专用用例，或快档的判红与路径断言分开两条用例。余量的数：门禁那一窗 2 个种子；把种子起点挪一窗，31 窗里 6 窗在 0–1 之间。

### 三-2　与第 41 行同类的变异快档不红（**打中**）

第 41 / 129 行改的是 `make_room_for_record_on_device` 里复用时改写记录的那三行（`allocator.rs` 第 541–543 行）。同一处拆开来一次拿掉一行：

| 编号 | 拿掉的行 | 快档 | 大档 3000×40 |
|---|---|---|---|
| N1 | `existing.span_slots = …`（第 541 行） | exit 101，新发现 12（`allocator.rs:529` panic 与 `I-3.1`） | 没跑 |
| **N2** | `existing.generation = generation;`（第 542 行） | **exit 0**，与基线逐字相同 | **exit 0**，与基线逐字相同 |
| N3 | `existing.is_released = false;`（第 543 行） | exit 101，新发现 18（`I-3.9`） | 没跑 |

N2 的 diff、命令、原样输出、够达探针与四句见二-1。改法（推的）：补一条判定「已分配（不带已释放标志）的记录，它的代等于那个落点上单元头的诞生 txg」，或让快档对复用那一步逐字段核记录；都没实现、没量。

### 三-3　与第 121 行同类的变异（N4 红；N5 不红但走不到）

| 编号 | 改法（`allocator.rs` 第 520、521 行） | 快档 | 大档两档 | 够达 |
|---|---|---|---|---|
| N4 | `record.slot.0 < placement_end` → `record.slot.0 + 1 < placement_end` | exit 101，新发现 9（`allocator.rs:549` panic） | 没跑 | — |
| N5 | `placement.slot.0 < record.slot.0 + span` → `placement.slot.0 <= record.slot.0`（漏掉起点在落点之前、跨进落点的记录） | exit 0，与基线逐字相同 | 两档 exit 0，与基线逐字相同 | 探针 PN5（新落点碰到起点在它之前的记录就 panic）快档与两档大档新发现 0：4096 段一次都没走到 |

N5 字面满足「一条与第 121 行同类的变异快档不红」，但生成器走不到那一格，而这一格在整个系统里可不可达本腿没核（要一条跨两槽的已回收记录落在奇数槽、之后一个用户数据落点起在它的第二槽；推的：可写挂载之间聚簇段登记清空之后可能走到）。四句：不分辨（快档与大档都没有输入走到它）；判别子（I-5.4）看得到，缺的是历史；满足 Z4 第二分句字面；改法「给第 121 行那一类补一个专门的取样点」对 N5 起作用的前提是先有一段走得到它的历史——本腿没造出来。

## 没打中的形状

| 形状 | 取样 | 结果 |
|---|---|---|
| 「没发写不跑 checker」漏判 | 基线与 N1，1000 段 × 30 步，跳过 / 每步都跑各一次 | 逐段相同（二-7） |
| checker 判「被抛弃」差一（A6，`walk.rs` 第 1374 行 `>` → `>=`） | 快档 | exit 101，新发现 72，不是遮蔽 |
| 回收门槛差一在挂载重建一侧显形 | A1 / A3 在挂载之后 | A3 的格走不到（二-6）；A1 在挂载一侧没单独造出格 |
| 第 1 条在基线上接空档外的东西 | 基线 3000 × 40 | 30 段全在空档里（一-1），没打中 |
| 第 130 行在任一几何点上不红 | 18 个点 | 没有（三-1） |

## 这条腿自己的限度

- 扫描全在 release profile 上（overflow-checks 开、没有 `debug_assert`）；dev profile 只复跑了 N2、B3、B6 的快档。
- 变异是手挑的 18 条加 6 条探针，不是 cargo-mutants 全量（第 7 件的事）；「没打中」的形状每个只抽一次。
- 机理核对（gap 探针）用的是实现的读法（`singlefs_core::recovery`），不是 checker 的读法；两者对「被抛弃」的定义都是行 (i, T) 且 txg > T，没逐条对过。
- 一-3 的「补判定」、二-6 的「撞第 0 条之后接着走」、三-1 的专用用例、三-2 的判定都是推的；量过的改法只有 P1、P2、P3，只在本腿副本上量过、被攻过零轮。
- 报告里改自己文件的两处（B3 大档一行抄错、三-1 两行数字写错）是写完后逐行对日志核出来的，已按原样改正；二-2、三-1 的原样行此后都用脚本对过日志。

## 没做什么

- 不判 Z1、Z5、Z6；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑门禁阶段（`stage-owners.tsv` 里没登记给 `three-way-attack` 的）；没改主工作区任何文件，只写了本报告、模型目录与草稿目录。
- A1 以外的变异没逐种子对「第 0 条接走」做机理核对；N1、N3、N4 没跑大档；B4 只证明了与基线逐字相同，没核读回内容。
- 草稿与副本在 `/tmp/claude-1000/m2-supp3-item1-r1-opus/`（`slot-A`…`slot-G`、各自的 target、`progress.md`），要入库的日志已拷进模型目录 `logs/`；副本本身没入库（大、可由 `pristine` 与表重建）。

## 中途收到的消息

- 16:25Z 前后撞周限额停；19:00Z 收到主 agent「限额 18:40Z 已重置，接着做，不用重做」：按原计划续做，补跑了种子窗口 26、dev profile 复跑、N2 / B3 在别的用例上的交叉核，写报告。
- 续做中收到主 agent 的第二条消息：之前那个用 `pgrep -f` 的等待循环违反 command-safety.md、会匹配到自己。那个循环已结束；之后只用 `run_in_background` 等通知，没再用 `pgrep`。
