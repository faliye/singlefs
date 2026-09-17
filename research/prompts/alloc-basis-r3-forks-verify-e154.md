<!-- 核查员原样报告：2026-09-17 主 agent 为岔路表派的只读核查（岔路单第 3、4 行），报告原件在 /tmp/claude-1000/verify-forks/e154/report.md；这里原样转存，不改内容 -->
<!-- sha256 1ceed8cb9a96894072bb570b295fa10a226340d6c5bed8c54b4b1e5f1acc2b1b -->

# E154 核查：「发出了候选根还引用的槽」从哪来，装置与实现的六项对应今天还成不成立

核查员（只读），2026-09-17 UTC 16:02–16:35。仓里一个文件都没改、没跑门禁、没提交。草稿、仓副本、改过的装置副本与全部产物都在 `/tmp/claude-1000/verify-forks/e154/`。

## 零、结论一览

| # | 结论 | 什么现象会推翻它 |
|---|---|---|
| 1 | H5-1-首 / H5-1-覆 / H5′-1-首 / H5′-1-覆 / H4a / H4b 上的全部「发出了候选根还引用的槽」都是**模型 bug**：`perform_instance_switch` 给旧实例写的行是 `(旧实例, 0, 0)`（E154 装置 1540–1547 行）。按 D23（journal 的角色与格式） 已定项 14 的 (i, T) 判法，这一行把旧实例留在环里的每一条根都判成被抛弃，「环里最旧有效根」当场跳到写行根自己的 txg，回收门槛越过还在候选集里的根。条款要写的是 (旧实例, 最后发布的 txg, W)。按条款逐项算那一刻的门槛是 234，被复用槽的释放代是 237，不该回收 | 副本里把那一行的 T 改成「该实例在环里最新根的 txg」之后，这六类历史里还有任何一格 unsafe > 0；或者按条款逐项重算那一刻门槛 ≥ 237 |
| 2 | 洞（根槽写失败留下的旧内容）不是原因：H4a 一次写失败都不注入，同一格（C=600、n=16、c=4、S=4、pad=0）照样 44 个命中，与 H5-1-首 相同；追踪里洞上那条旧根（txg 232）在条款口径下反而把门槛往下拉（偏保守），与 E153 看到的「甲-T1 在洞上多扣」同向 | 在不写 (i, 0) 行的前提下，有一格只因为洞而命中 |
| 3 | 同一个 bug 也改了这几类历史的 Q1 / Q5：H5-1-首 全部 24 臂 × 48 格里「达标」697 → 484、「卡死」114 → 316；真实基线 串重判-G7-止 在 H5-1-首 上 13 格里 11 格从「达标 3」变「越界 4」。岔路单第 3、4 行若用了 H4 / H5 / H5′ 的 k = 1 这几族的达标、越界、卡死数，要按修过的装置重跑 | 修过行之后这几族的 Q1 表与原产物逐格相同 |
| 4 | k ≥ 2 一个都不中是**另一个模型 bug 恰好抵消**：写行发布自己失败后递归切换，行只写 [当前实例, 新实例)，实例 1 那一行随失败的写行发布一起丢了。追踪 H5-2-首：唯一持久的行是 `(2, 0)`，实例 1 没有行，它的根按 (i, T) 判法全部有效 | 追踪里 k = 2 持久的行里有实例 1 |
| 5 | H6 的命中与切换行无关（修了行仍是 962），分两块：① 60 格回退到**已经被环转掉**的根（装置 2395–2399 行只查 F 与被抛弃，没查还在不在环里），占 348 个命中；实现在这一步报 `RollbackTargetNotInRing`（mount.rs 1044–1050 行）。② 余下 614 个是「每次发布持久之后按环转回收」与「影子账只在回退那一刻算一次」对不上：每次抬 F 之前重算降到 282（剩下的机理：抬 F 那次推空自己盖掉了 R_old 的根槽，重算在它之前），每次回收之前重算降到 0 | 加了「目标在环里」与「每次回收前重算影子账」两处之后 H6 仍有命中 |
| 6 | D16（发布语义） 已定项 1 的谓词式子本身在这批历史上没有被打中：H4 / H5 族的命中在条款口径下不回收；H6 剩下的命中在谓词之外——D23（journal 的角色与格式） 已定项 14 的主句不许复用被抛弃根引用的单元，而它的机制句只说影子账「每次挂载与每次抬 F 按当时的候选集重算」。实现今天回收只在挂载与抬 F 两处、抬 F 先重算再回收，同一段历史上不复用；若采用 E154 登记里 G7 / F-扣 那种「每次发布持久之后按环转回收」的读法，机制句的时点罩不住，要改成「每次回收之前重算」 | 实现或条款写明的某种回收时点下，影子账在挂载与抬 F 时重算就够，而副本上量到反例 |
| 7 | 问题二：①②③⑤⑥ 成立；④ 式子成立、「非空」换成了按树表根指针比（R9 本来就排除），树表读不出时整次抬 F 拒绝。六项之外，登记没列的两样对不上：切换写行的 T（上面第 1 条），以及切换写行发布带的 F（装置带 F_生效，登记 R8 写「带失败那次发布本来要带的 F」） | 见第二节各行 |

## 一、问题一

### 1.1 副本与产物一致

副本：`rsync -a --exclude target /home/fy5090/code/singlefs/research/ /tmp/claude-1000/verify-forks/e154/research/`，装置源码 sha256 与仓里相同（`0c673b1e…`）。只加打印与由环境变量打开的反事实开关（`tools/patch_trace.py`），不开任何开关时输出与产物逐字节相同：

```
$ cmp full-nofix.out /home/fy5090/code/singlefs/research/results/e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out && echo "nofix output byte-identical to stage4 product"
nofix output byte-identical to stage4 product
d510ff5d288d9bc40e0838e4a81e19089b72f303a2ccd019c85d9f5e2fa3c978  full-nofix.out
d510ff5d288d9bc40e0838e4a81e19089b72f303a2ccd019c85d9f5e2fa3c978  /home/fy5090/code/singlefs/research/results/e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-17-stage4.out
```

所以下面副本上追到的每一步，就是产物里那一格发生的事。

### 1.2 选哪一格

产物里 `history=H5-1-首`、`gate=串重判-G7-止`、unsafe > 0 的 13 格（命令 `tools/diffruns.py` 的尾段列了全部 13 格）；取 C=600、n=16、c_max=4、S=4、pad=0，产物原行：

```
E7RESULT RUN gate=串重判-G7-止 baseline=true reclaim=G7 stop=止 history=H5-1-首 c=600 n=16 c_max=4 s=4 pad=0 delete_then_rewrite_outcome_label=达标 delete_then_rewrite_publish_count=3 df_reported_enough_but_write_failed_count=0 admission_passed_but_allocation_failed_count=0 budget_was_binding=false maximum_publish_attempts_seen_field=4 allocation_failure_hit_field=false wedged=false unsafe_candidate_reuse_count_field=44 retained_states_dropped_below_minimum=0 filler_objects_written_so_far=234
```

追踪命令：`E154_TRACE_ONE='H5-1-首,串重判,G7,止,600,16,4,4,0,3' nice -n 19 target/release/e154-two-gates-serial-rejudge-and-reclaim-timing > trace-h5-1-first-600-16-4-4-0.log`，末行 unsafe 仍是 44。

### 1.3 追到的第一次复用（追踪原行，行号是 `trace-h5-1-first-600-16-4-4-0.log` 的行号）

事件次序：txg 238、239 两次推空把 F 抬到 234 并在两块盘上生效；240–243 是 H5 前缀的删 X 与三次触动；244 是这次准入的第一次推空（要带 F = 240），被注入根槽写失败；随后实例切换，245 是写行发布，246 是暖机空发布，**246 的固定点落在槽 0–3 上**，第一次命中。

```
482: TRACE PUB_OK txg=243 inst=1 F=234 non_empty=true F_eff=234 oldest_valid(model)=232 oldest_valid(clause_rows)=232 free=58 released=74
483: TRACE PUB_TRY txg=244 inst=1 non_empty=false floor=240 release_obj=None alloc_units=None rewrite_table=false fail=true
484: TRACE PUB_FAIL_INJECTED txg=244 (ring slot not written, keeps old content)
485: TRACE AFTER_FAIL RING next_txg=245 instance=1 F_eff=234 F_top=234 oldest_valid(model)=232 oldest_valid(clause_rows)=232
486: TRACE AFTER_FAIL instance_table(model)=[]
500: TRACE SWITCH prev_inst=1 new_inst=2 row_instance=1 model_selected_root_txg=0 fix_mode=false fixed_selected=0
517: TRACE ROWS_PERSISTED txg=245 rows=[InstanceRow { instance: 1, selected_root_txg: 0, applied_transaction_high_water: 0, is_rollback: false }]
518: TRACE PUB_OK txg=245 inst=2 F=234 non_empty=false F_eff=234 oldest_valid(model)=245 oldest_valid(clause_rows)=232 free=132 released=0
519: TRACE PUB_TRY txg=246 inst=2 non_empty=false floor=234 release_obj=None alloc_units=None rewrite_table=false fail=false
520: TRACE UNSAFE_HIT slot=0 this_publish_txg=246 first_matching_candidate=(txg=236 inst=1) all_ring_roots_referencing_slot=["(txg=236 inst=1 F=0 abandoned(model)=true abandoned(clause)=false ge_F_eff=true)"] reclaim_log[slot]=(released_at,reclaimed_after_publish_txg,threshold,F_eff,oldest_valid(model),oldest_valid(clause_rows))=Some((237, 245, 245, 234, 245, 232))
```

命中那一刻环的样子（追踪 521–535 行，`abandoned(model)` 是装置判的，`abandoned(clause)` 是按条款写行 (1, 243) 判的）：

```
521: TRACE AT_HIT RING next_txg=246 instance=2 F_eff=234 F_top=234 oldest_valid(model)=245 oldest_valid(clause_rows)=232
522: TRACE AT_HIT instance_table(model)=[InstanceRow { instance: 1, selected_root_txg: 0, applied_transaction_high_water: 0, is_rollback: false }]
523: TRACE AT_HIT instance_table(clause)=[InstanceRow { instance: 1, selected_root_txg: 243, applied_transaction_high_water: 0, is_rollback: false }]
524: TRACE AT_HIT ring[0] region=0 dev=0 txg=240 inst=1 F=234 non_empty=true abandoned(model)=true abandoned(clause)=false
525: TRACE AT_HIT ring[1] region=0 dev=0 txg=243 inst=1 F=234 non_empty=true abandoned(model)=true abandoned(clause)=false
526: TRACE AT_HIT ring[2] region=0 dev=0 txg=234 inst=1 F=0 non_empty=true abandoned(model)=true abandoned(clause)=false
527: TRACE AT_HIT ring[3] region=0 dev=0 txg=237 inst=1 F=0 non_empty=true abandoned(model)=true abandoned(clause)=false
528: TRACE AT_HIT ring[4] region=1 dev=1 txg=241 inst=1 F=234 non_empty=true abandoned(model)=true abandoned(clause)=false
529: TRACE AT_HIT ring[5] region=1 dev=1 txg=232 inst=1 F=0 non_empty=true abandoned(model)=true abandoned(clause)=false
530: TRACE AT_HIT ring[6] region=1 dev=1 txg=235 inst=1 F=0 non_empty=true abandoned(model)=true abandoned(clause)=false
531: TRACE AT_HIT ring[7] region=1 dev=1 txg=238 inst=1 F=234 non_empty=false abandoned(model)=true abandoned(clause)=false
532: TRACE AT_HIT ring[8] region=2 dev=0 txg=242 inst=1 F=234 non_empty=true abandoned(model)=true abandoned(clause)=false
533: TRACE AT_HIT ring[9] region=2 dev=0 txg=245 inst=2 F=234 non_empty=false abandoned(model)=false abandoned(clause)=false
534: TRACE AT_HIT ring[10] region=2 dev=0 txg=236 inst=1 F=0 non_empty=true abandoned(model)=true abandoned(clause)=false
535: TRACE AT_HIT ring[11] region=2 dev=0 txg=239 inst=1 F=234 non_empty=false abandoned(model)=true abandoned(clause)=false
```

逐项读出来：

| 量 | 这一刻的值 | 出处 |
|---|---|---|
| 被复用的槽 | 槽 0（同一次还有 1、2、3），这次发布是 txg 246 的暖机空发布，作固定点 | 追踪 520 行 `slot=0 this_publish_txg=246` |
| 引用它的根 | txg 236、实例 1、F = 0；环里只有这一条根引用它 | 追踪 520 行 `all_ring_roots_referencing_slot` |
| 根环每个槽的 txg | 区域 0：240、243、234、237；区域 1：241、**232**、235、238；区域 2：242、245、236、239 | 追踪 524–535 行 |
| 洞在哪 | ring[5]（区域 1 槽 1）：失败的 txg 244 该写这里（244 mod 3 = 1，⌊244/3⌋ mod 4 = 1），没写成，留着上一圈的 txg 232 | 追踪 484 行 `PUB_FAIL_INJECTED txg=244`，493 / 509 / 529 行 `ring[5] … txg=232` |
| 实例表 | 装置：`(1, 0, 0)`；条款：`(1, 243, W)` | 追踪 522 / 523 行；条款见 1.4 |
| F_生效 | 234 | 追踪 521 行 |
| 环里最旧有效根 | 装置 245；条款口径 232 | 追踪 521 行 |
| 槽 0 的释放代 | 237（txg 237 那次发布换下 236 的固定点） | 追踪 520 行 `reclaim_log[slot]=… Some((237, 245, 245, 234, 245, 232))` |
| 回收发生在哪次发布之后、用的门槛 | txg 245（写行发布）持久之后，门槛 = max(F_生效 234, 最旧有效根 245) = 245 | 同上：(释放代, 回收于, 门槛, F_生效, 最旧有效根(装置), 最旧有效根(条款行)) |

装置在 245 持久那一刻：`ROWS_PERSISTED txg=245 rows=[(1, 0, …)]`（517 行）→ `PUB_OK txg=245 … oldest_valid(model)=245 oldest_valid(clause_rows)=232 free=132 released=0`（518 行）：74 个已释放槽一次全部回收。写行之前（501 行）装置自己算的最旧有效根还是 232。

装置里造成这一跳的三行（仓里文件 `research/e7-index-bench/src/bin/e154_two_gates_serial_rejudge_and_reclaim_timing.rs`）：

```
1538:    let first_row_instance = 1u64.max(previous_instance);
1540:    for row_instance in first_row_instance..new_instance {
1541:        rows.push(InstanceRow {
1542:            instance: row_instance,
1543:            selected_root_txg: 0,
364:    fn is_abandoned(&self, root: &RootRecord) -> bool {
1147:            let threshold = effective.max(oldest);
```

`is_abandoned` 的判法（364–368 行）是 `row.instance == root.instance && root.txg > row.selected_root_txg`，与条款的 (i, T) 判法同形；错的是行里的 T。回收在每次发布持久之后做（1395 行 `reclaim_after_publish(pool, reclaim);`），写行发布一持久门槛就跳。

### 1.4 按条款原文逐项算这一刻

D16（发布语义） 已定项 1 的形态表（`.claude/kb/decisions/16-发布语义.md`，行号现取，文件 blob `f30865818c730113125863000c8a9bceb0144032`，工作区 ` M`）：

```
371: | 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
372: | 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算（C282（环里最旧根没有定义） 要的定义） |
375: | 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |
376: | 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |
```

「按实例表判仍然有效」与切换写哪一行，在 D23（journal 的角色与格式） 已定项 14：索引行（`.claude/kb/decisions/23-journal的角色与格式.md` 691 行）写切换的行「行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)……旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行」；正文（同文件 1209 行）写判法「(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）」。两行都很长，整行原文在附录 A，这里只指路。

逐项代进 txg 245 持久之后、回收之前的环（追踪 521–535 行，回收只改槽状态不改环，所以与那一刻相同）：

| 项 | 条款怎么算 | 值 |
|---|---|---|
| 切换写的行 | 旧实例 1 发布过根，最后一条持久的是 txg 243（244 写失败）⇒ (1, 243, W) | (1, 243, W) |
| 按实例表判有效 | 实例 1 的根 T ≤ 243 全部有效；实例 2 没有行 ⇒ 有效 | 环里 12 条根全部有效 |
| 环里最旧有效根 | 盘上全部根槽里自证合法、有效的最小 txg；写失败的槽按旧内容算 ⇒ ring[5] 的 232 算数 | 232 |
| F_生效 | 各盘所带 F 最大值的最小值：盘 0（区域 0、2）max(234, 234, 0, 0, 234, 234, 0, 234) = 234；盘 1（区域 1）max(234, 0, 0, 234) = 234 | 234 |
| 可再分配门槛 | max(F_生效, 环里最旧有效根) = max(234, 232) | 234 |
| 槽 0 可再分配？ | 已释放 ∧ 释放代 237 ≤ 234 | **否** |
| 回退候选集 | 有效 ∧ txg ≥ 234 ⇒ 234、235、236、237、238、239、240、241、242、243、245 | txg 236 在候选集里，它引用槽 0 |

所以按条款原文，这一刻槽 0 不回收；装置回收了，然后在 246 把它发给固定点，候选根 236 被破坏。条款在这个洞上没有不安全：洞上的旧根 232 让门槛更低（多扣），不是更高。

### 1.5 今天的实现在同一个环上会不会回收

`crates/singlefs-core/src/` 里没有实例切换：

```
$ grep -rn 'instance_switch\|InstanceSwitch\|probe_write\|切换' crates/singlefs-core/src/ | wc -l
0
```

H5 这段历史在实现里走不到；能走到的近邻是 H4a（崩溃后重开，`mount_writable`）。那条路上三处决定会不会回收（`crates/singlefs-core/src/mount.rs`，blob `9a492f2739a1780e9c80211a1a9f62e8ed0c7d60`，未跟踪；另一个会话仍在改，照现取的这一版读）：

```
1000:        selected_root_txg: effective_root.checkpoint_txg,
258:        .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
249:    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
372:     let oldest_valid_root = roots
373:         .iter()
374:         .filter(|root| !is_abandoned(root))
375:         .map(|root| root.checkpoint_txg)
376:         .min();
377:     allocator.reclaim_released_up_to(
378:         reclaim_floor(effective_floor, oldest_valid_root),
379:         ReclaimedReuse::Immediately,
380:     );
606:     .filter(|root| !abandoned_by_table(root, &table))
607:     .map(|root| root.checkpoint_txg)
608:     .min();
629:     let reclaimed = allocator.reclaim_released_up_to(
630:         reclaim_floor(new_floor, oldest_valid_root),
631:         ReclaimedReuse::HeldUntilFloorTakesEffect,
```

- 写行：普通挂载给上一个实例写 `(实例, 施加前缀之后那条根的 txg, W)`（mount.rs 998–1003 行），不是 0；中间实例才写 `(i, 0, 0)`（mount.rs 856–862 行）。所以重开之后旧实例留在环里的根按 `abandoned_by_table`（252–259 行）一条都不被判抛弃。
- 回收只在两处：挂载重建（mount.rs 372–380 行，用所选根那一版的实例表判有效，写行之前）与抬 F（mount.rs 599–608、629–632 行，用现行版本的实例表判有效，回收之前先按新 F 重算影子账 611–628 行）。发布持久之后不回收。
- 同一个环上：挂载那一刻有效根的最小 txg 是环里最旧的那条（H4a 追踪 486 行 `oldest_valid(model)=233`，没有 (1, 0) 行时装置与条款同值），门槛 max(234, 233) = 234 < 237，槽 0 不回收；之后要等一次抬 F 才再回收，而抬 F 的新 F 一旦 ≥ 237，txg 236 自己就不在 txg ≥ F 的候选集里，回收的槽还要扣住到新 F 落满两块盘（`ReclaimedReuse::HeldUntilFloorTakesEffect`，mount.rs 631 行；放开在 665 行）。

结论：今天的实现在这个环上不会回收槽 0。

附带：`allocator.rs` 587–589 行的文档注释仍写「第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效」，而调用方 mount.rs 249 行已经取 max(F_生效, 最旧有效根)；注释过时，不影响行为。

### 1.6 判定

**模型 bug**，装置 1540–1547 行：切换给旧实例写 `selected_root_txg: 0`，与 D23（journal 的角色与格式） 已定项 14 的切换行规则、实现 mount.rs 1000 行、E153 装置 1925 / 1932 行都不一致。条款与实现在这个环上都不回收。

### 1.7 反事实：只改那一行，全网格重跑

开关 `E154_FIX_SWITCH_ROW=1`：切换写行的 T 取「该实例在环里最新那条根的 txg，没有取 0」（`tools/patch_trace.py` 第 7 段），别的一个字不动。同一个二进制跑两遍全量 `main()`：

```
$ nice -n 19 python3 tools/tabulate.py full-nofix.out full-fixrow.out full-fixrow-shadow1.out full-fixrow-shadow2.out
full-nofix.out: RUN rows=18432 DONE_line=True
full-fixrow.out: RUN rows=18432 DONE_line=True
full-fixrow-shadow1.out: RUN rows=18432 DONE_line=True
full-fixrow-shadow2.out: RUN rows=18432 DONE_line=True
history | full-nofix.out: runs_with_unsafe>0 / total_unsafe | full-fixrow.out: runs_with_unsafe>0 / total_unsafe | full-fixrow-shadow1.out: runs_with_unsafe>0 / total_unsafe | full-fixrow-shadow2.out: runs_with_unsafe>0 / total_unsafe
H1 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H4a | 185/1152 runs, sum=7292 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H4b | 185/1152 runs, sum=7292 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-1-覆 | 864/1152 runs, sum=51408 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-1-首 | 864/1152 runs, sum=45964 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-2-覆 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-2-首 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-3-覆 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5'-3-首 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-1-覆 | 631/1152 runs, sum=25191 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-1-首 | 697/1152 runs, sum=26658 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-2-覆 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-2-首 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-3-覆 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H5-3-首 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0 | 0/1152 runs, sum=0
H6 | 270/1152 runs, sum=962 | 270/1152 runs, sum=962 | 201/1152 runs, sum=630 | 0/1152 runs, sum=0
```

（`full-fixrow-shadow1/2` 两列另加了影子账重算开关，见 1.9。）改行之后 H4a / H4b / H5-1-* / H5′-1-* 的命中全部为 0；H1、H5-2/3、H5′-2/3、H6 的 RUN 行与原产物逐格相同（下表第二列为 0）。阳性对照没有被这一改关掉：`E7RESULT DONE` 行里 20 个 `*_hit_any` 两边逐个相同，PC4 各行里命中为 0 的都是 30 行。

同一次改动对 Q1 的影响（`tools/diffruns.py full-nofix.out full-fixrow.out 'H5-1-首' '串重判-G7-止'`，原样输出）：

```
history | rows differing (any field) | rows whose Q1 label/count differ | label counts A -> B
H1 | 0/1152 | 0 | {'卡死': 180, '未成': 48, '越界': 553, '达标': 371} -> {'卡死': 180, '未成': 48, '越界': 553, '达标': 371}
H4a | 185/1152 | 50 | {'卡死': 116, '达标': 185, '退化': 851} -> {'卡死': 154, '越界': 12, '达标': 135, '退化': 851}
H4b | 185/1152 | 50 | {'卡死': 116, '达标': 185, '退化': 851} -> {'卡死': 154, '越界': 12, '达标': 135, '退化': 851}
H5'-1-覆 | 864/1152 | 256 | {'卡死': 202, '越界': 84, '达标': 578, '退化': 288} -> {'卡死': 290, '越界': 12, '达标': 562, '退化': 288}
H5'-1-首 | 864/1152 | 223 | {'卡死': 264, '越界': 97, '达标': 503, '退化': 288} -> {'卡死': 184, '越界': 14, '达标': 666, '退化': 288}
H5'-2-覆 | 0/1152 | 0 | {'卡死': 238, '越界': 20, '达标': 606, '退化': 288} -> {'卡死': 238, '越界': 20, '达标': 606, '退化': 288}
H5'-2-首 | 0/1152 | 0 | {'卡死': 190, '越界': 12, '达标': 662, '退化': 288} -> {'卡死': 190, '越界': 12, '达标': 662, '退化': 288}
H5'-3-覆 | 0/1152 | 0 | {'卡死': 132, '越界': 36, '达标': 696, '退化': 288} -> {'卡死': 132, '越界': 36, '达标': 696, '退化': 288}
H5'-3-首 | 0/1152 | 0 | {'卡死': 138, '越界': 38, '达标': 688, '退化': 288} -> {'卡死': 138, '越界': 38, '达标': 688, '退化': 288}
H5-1-覆 | 631/1152 | 370 | {'卡死': 180, '越界': 89, '达标': 542, '退化': 341} -> {'卡死': 504, '越界': 48, '达标': 259, '退化': 341}
H5-1-首 | 697/1152 | 213 | {'卡死': 114, '达标': 697, '退化': 341} -> {'卡死': 316, '越界': 11, '达标': 484, '退化': 341}
H5-2-覆 | 0/1152 | 0 | {'卡死': 412, '越界': 108, '达标': 291, '退化': 341} -> {'卡死': 412, '越界': 108, '达标': 291, '退化': 341}
H5-2-首 | 0/1152 | 0 | {'卡死': 334, '越界': 20, '达标': 457, '退化': 341} -> {'卡死': 334, '越界': 20, '达标': 457, '退化': 341}
H5-3-覆 | 0/1152 | 0 | {'卡死': 318, '越界': 160, '达标': 333, '退化': 341} -> {'卡死': 318, '越界': 160, '达标': 333, '退化': 341}
H5-3-首 | 0/1152 | 0 | {'卡死': 332, '越界': 135, '达标': 344, '退化': 341} -> {'卡死': 332, '越界': 135, '达标': 344, '退化': 341}
H6 | 0/1152 | 0 | {'卡死': 442, '未成': 90, '越界': 43, '达标': 69, '退化': 508} -> {'卡死': 442, '未成': 90, '越界': 43, '达标': 69, '退化': 508}
--- H5-1-首 gate=串重判-G7-止: Q1 label/count A -> B per geometry (only differing)
('4000', '1', '9', '4', '2') 达标 3 unsafe 20 -> 达标 3 unsafe 0
('4000', '16', '4', '4', '0') 达标 3 unsafe 40 -> 越界 4 unsafe 0
('4000', '16', '4', '4', '1') 达标 3 unsafe 44 -> 越界 4 unsafe 0
('4000', '16', '4', '4', '2') 达标 3 unsafe 40 -> 越界 4 unsafe 0
('4000', '16', '9', '4', '0') 达标 3 unsafe 50 -> 越界 4 unsafe 0
('4000', '16', '9', '4', '1') 达标 3 unsafe 59 -> 越界 4 unsafe 0
('4000', '16', '9', '4', '2') 达标 3 unsafe 50 -> 越界 4 unsafe 0
('600', '16', '4', '4', '0') 达标 3 unsafe 44 -> 越界 4 unsafe 0
('600', '16', '4', '4', '1') 达标 3 unsafe 40 -> 越界 4 unsafe 0
('600', '16', '4', '4', '2') 达标 3 unsafe 40 -> 越界 4 unsafe 0
('600', '16', '9', '4', '0') 达标 3 unsafe 50 -> 达标 3 unsafe 0
('600', '16', '9', '4', '1') 达标 3 unsafe 59 -> 越界 4 unsafe 0
('600', '16', '9', '4', '2') 达标 3 unsafe 50 -> 越界 4 unsafe 0
```

所以 stage4 产物里 k = 1 的 H5 / H5′ 与 H4a / H4b 的达标、越界、卡死计数，以及它们的 PC1 / PC2 / PC4 行（`diff` 126 行不同，例 `pc4-root_write_failure arm=串重判-G7-止` 70 → 34），都带着这个 bug。H5-1-首 上真实基线 13 格里 11 格由「达标 3」变「越界 4」，这正是岔路单第 4 行「删后写回要几次改用户可见状态的发布越过界 3」要读的量。

### 1.8 k ≥ 2 为什么一个都不中

装置 1550–1554 行：写行发布失败时递归 `perform_instance_switch`，而 `pool.instance_table` 只在发布成功时才换成新行（1364–1365 行），递归那一层 `previous_instance` 已经是 2，`first_row_instance = 1u64.max(previous_instance)`（1538 行）= 2，只写 (2, 0)。追踪 H5-2-首 同一格（`trace-h5-2-first-600-16-4-4-0.log`，用多认一个历史名的副本 `research3/` 跑）：

```
484: TRACE PUB_FAIL_INJECTED txg=244 (ring slot not written, keeps old content)
500: TRACE SWITCH prev_inst=1 new_inst=2 row_instance=1 model_selected_root_txg=0 fix_mode=false fixed_selected=0
517: TRACE PUB_FAIL_INJECTED txg=245 (ring slot not written, keeps old content)
533: TRACE SWITCH prev_inst=2 new_inst=3 row_instance=2 model_selected_root_txg=0 fix_mode=false fixed_selected=0
550: TRACE ROWS_PERSISTED txg=246 rows=[InstanceRow { instance: 2, selected_root_txg: 0, applied_transaction_high_water: 0, is_rollback: false }]
delete_then_rewrite_outcome_label=越界
delete_then_rewrite_publish_count=4
unsafe_candidate_reuse_count_field=0
```

持久的只有 `(2, 0)`；实例 1 没有行，它的根按判法全部有效，门槛不跳，所以不中。按条款，切换的行落在 [所选根的实例, 新实例) = [1, 3)：(1, 243, W) 与 (2, 0, W)——实例 1 的根同样全部有效，结果碰巧同向，但装置丢了一行。E153 在同形历史 H3² 上写的是 `(outgoing, basis_txg)` 再加 `(i, 0)`（E153 装置 1941–1946 行），两行都在。

### 1.9 H6 的命中是另外两件事

改了切换行之后 H6 仍是 270 格、962 个命中（1.7 表最后一行）。副本 `research4/` 另加两个开关、只跑 H6（`E154_ONLY_H6=1`，不开开关时 RUN 行与全量产物的 H6 行逐行相同）：`E154_FIX_ROLLBACK_IN_RING=1` 回退目标已不在环里就记退化；`E154_FIX_SHADOW=1` 每次抬 F 的推空之前按新 F 的候选集重算影子账（照实现 mount.rs 611–628 行），`=2` 每次回收之前按当时的候选集重算。

```
$ nice -n 19 python3 tools/tabulate.py h6-nofix.out h6-inring.out h6-inring-shadow1.out h6-inring-shadow2.out
h6-nofix.out: RUN rows=1152 DONE_line=True
h6-inring.out: RUN rows=1152 DONE_line=True
h6-inring-shadow1.out: RUN rows=1152 DONE_line=True
h6-inring-shadow2.out: RUN rows=1152 DONE_line=True
history | h6-nofix.out: runs_with_unsafe>0 / total_unsafe | h6-inring.out: runs_with_unsafe>0 / total_unsafe | h6-inring-shadow1.out: runs_with_unsafe>0 / total_unsafe | h6-inring-shadow2.out: runs_with_unsafe>0 / total_unsafe
H6 | 270/1152 runs, sum=962 | 210/1152 runs, sum=614 | 141/1152 runs, sum=282 | 0/1152 runs, sum=0
$ （同一份 h6-nofix 与 h6-inring 比）
H6 runs that become 退化 once rollback target must still be in ring: 60
  of which had unsafe>0 before: 60 unsafe sum 348
  by S: {'4': 36, '8': 24}
```

**① 回退到已被环转掉的根（60 格、348 个命中，模型 bug）**。装置 2395–2399 行：

```
2395:     let effective_floor_now = pool.effective_rollback_floor();
2396:     if rollback_target_root.txg < effective_floor_now || pool.is_abandoned(&rollback_target_root) {
2397:         // 回退够不着（不在候选集里）：登记明令这一格记「回退够不着」，退化。
2398:         return (degenerate_summary(&pool), 0);
2399:     }
```

`R_old` 是删后写回之前克隆出来的 `RootRecord`，之后又触动两次，S = 4 时环一圈 12 条，这两次发布可以把 R_old 的根槽盖掉；这里不查它还在不在环里。条款的候选集是「根环里按实例表判仍然有效、且 txg ≥ F_生效的根」（D16（发布语义） 已定项 1 形态表 376 行、D23（journal 的角色与格式） 已定项 14 正文 1209 行），实现报 `MountError::RollbackTargetNotInRing`（mount.rs 1044–1050 行）。追踪 `trace-h6-serial-shadow1-4000-1-4-4-2.log`（`串-G7-止`，C=4000、n=1、c=4、S=4、pad=2）：回退目标 1955（3934 行 `ROWS_PERSISTED txg=1969 rows=[… selected_root_txg: 1955 … is_rollback: true]`）的根槽 ring[11] 已被 1967 盖掉（3952 行 `ring[11] … txg=1967`），回退之后环里除写行根 1969 外全是被抛弃根，最旧有效根 = 1969，回退那次发布换下的实例表槽 4、5 当场回收，1970 把它们发给固定点，被 1958–1968 那几条被抛弃根引用着。

**② 回收时点与影子账重算时点对不上（修了 ① 之后剩 614 → 每次抬 F 前重算剩 282 → 每次回收前重算 0）**。追踪 `trace-h6-rejudge-inring-shadow1-4000-1-4-4-2.log`（`串重判-G7-止`，C=4000、n=1、c=4、S=4、pad=2，三个开关 row / inring / shadow=1 都开，命中仍是 2）：

```
3917: TRACE ROLLBACK_TARGET txg=1957 in_ring=true F_eff=1949
3919: TRACE ROWS_PERSISTED txg=1961 rows=[InstanceRow { instance: 1, selected_root_txg: 1957, applied_transaction_high_water: 0, is_rollback: true }]
  3934:TRACE PUB_OK txg=1968 inst=2 F=1956 non_empty=false F_eff=1955 oldest_valid(model)=1957 oldest_valid(clause_rows)=1957 free=66 released=36
  3935:TRACE PUB_TRY txg=1969 inst=2 non_empty=false floor=1957 release_obj=None alloc_units=None rewrite_table=false fail=false
  3936:TRACE PUB_OK txg=1969 inst=2 F=1957 non_empty=false F_eff=1956 oldest_valid(model)=1961 oldest_valid(clause_rows)=1961 free=68 released=34
  3937:TRACE PUB_TRY txg=1970 inst=2 non_empty=true floor=1957 release_obj=None alloc_units=Some(1) rewrite_table=false fail=false
3938: TRACE UNSAFE_HIT slot=4 this_publish_txg=1970 first_matching_candidate=(txg=1959 inst=1) all_ring_roots_referencing_slot=["(txg=1959 inst=1 F=1949 abandoned(model)=true abandoned(clause)=true ge_F_eff=true)", "(txg=1960 inst=1 F=1949 abandoned(model)=true abandoned(clause)=true ge_F_eff=true)", "(txg=1958 inst=1 F=1949 abandoned(model)=true abandoned(clause)=true ge_F_eff=true)"] reclaim_log[slot]=(released_at,reclaimed_after_publish_txg,threshold,F_eff,oldest_valid(model),oldest_valid(clause_rows))=Some((1961, 1969, 1961, 1956, 1961, 1961))
```

回退目标 1957（在环里），回退那次发布 1961 把旧实例表槽 4、5 放成已释放（释放代 1961）；被抛弃根 1958、1959、1960 还引用它们，R_old 1957 也引用它们，所以回退时按窄读法不隔离，这一步与条款一致。txg 1969 是一次抬 F（F 1956 → 1957）的推空：按实现的时点在它之前重算影子账，那时 1957 还在环里、仍豁免槽 4、5；1969 持久时恰好盖掉 1957 的根槽（1957 与 1969 都落 ring[4]），环里最旧有效根变成 1961，装置在这次发布持久之后立刻按环转回收，槽 4、5 空闲；1970 是一次用户写，不抬 F、不重算，把槽 4、5 发给数据，三条被抛弃根还在环里。

按条款逐句看这一格：
- D16（发布语义） 已定项 1 的谓词：释放代 1961 ≤ max(F_生效 1956, 环里最旧有效根 1961) ⇒ **可再分配**，谓词本身允许。
- D23（journal 的角色与格式） 已定项 14 的主句「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」⇒ **不许**。
- 同一句的机制「只隔离其中只被被抛弃根引用的槽……每次挂载与每次抬 F 按当时的候选集重算」⇒ 在「每次发布持久之后按环转回收」下罩不到：候选集是被 1969 这次抬 F 发布自己的根槽写缩小的，重算在它之前。

实现在这一格不复用：它发布之后不回收；抬 F 时先重算影子账、再按抬之前的环算最旧有效根回收（mount.rs 599–632 行），1957 还在环里、门槛 max(1957, 1957) = 1957 < 1961，不回收；下一次挂载或抬 F 重算时 1957 已不在候选集里，槽 4、5 会先被隔离。

判定：② 这一块**不是 D16 谓词的式子错**，是 E154 登记里 G7 / F-扣 两种回收时点（「每次发布持久之后……已释放 → 空闲」「环转过去的照常直接 → 空闲」，登记 635–636 行）与 D23 已定项 14 影子账的重算时点合在一起的缺格。岔路单第 3 行（两个 F 的口径）与第 5 行（影子账读法）若采用按环转每次发布回收的读法，机制句要写成「每次回收之前按当时的候选集重算」；副本上这样做 H6 命中为 0，代价是 H6 的 1152 格里 102 格 Q1 变了（`full-fixrow.out` 对 `full-fixrow-shadow2.out`，例：`串-G7-止` C=4000、n=1、c=9、S=4、pad=2 由达标变卡死）。实现今天的时点（只在挂载与抬 F 回收、抬 F 先重算）在这批历史上没有打中。

### 1.10 E153 与 E154 在「环里最旧有效根 / 候选集 / 洞」上差在哪几行

判法本身两边同形，差在喂给判法的实例表行、以及回收在写行之前还是之后：

| 项 | E153（`research/e7-index-bench/src/bin/e153_ledger_shape_and_ring_holes.rs`，blob `9233d139…`） | E154（blob `e563a692…`） | 效果 |
|---|---|---|---|
| 被抛弃判法 | 198 行 `self.instance_table_rows.iter().any(\|(row_instance, valid_up_to)\| *row_instance == instance && txg > *valid_up_to)` | 364–368 行 `row.instance == root.instance && root.txg > row.selected_root_txg` | 相同 |
| 环里最旧有效根 | 208 行，有效根 txg 的最小值 | 370–376 行，同 | 相同 |
| 候选集 | 232 行 有效 ∧ txg ≥ F_生效 | 1224 行 `(root.txg >= effective && !abandoned) \|\| abandoned`，把被抛弃根也算进「不许复用」 | E154 的 unsafe 同时数了 E153 的 `violations_candidate` 与 `violations_abandoned` 两种（E153 1397–1401 行分开数）；H6 那些命中全是被抛弃根的引用 |
| 洞 | 1354–1356 行写失败不 `commit_root`，1422 行「根槽保留旧内容」 | 1326–1341 行写失败不写环 | 相同：洞上留上一圈的旧根，最旧有效根被它拉低 |
| **切换写的行** | **1925 行** `push((self.pool.current_instance, outgoing_row_txg))`，1932 行传 `basis_txg`（失败之前最后一次成功发布的 txg）；H3² 第二次切换 1945 行对没发成过的实例传 0，第一次切换的行保留 | **1540–1547 行** `selected_root_txg: 0`；k ≥ 2 时丢掉旧实例那一行 | **分叉点**。E153 旧实例的根全有效，洞上旧根把门槛拉低 ⇒ 多扣；E154 旧实例的根全判抛弃，门槛跳到写行根 ⇒ 提前回收 |
| 回收与写行的先后 | 切换在 1918–1922 行先按旧表算门槛重建，1925 行才写行；测量臂每次发布之前用「这次发布之前的环」回收（1349、1364–1366 行），基线只在挂载 / 切换 / 回退 / 抬 F 回收（1016、1362–1363 行） | 写行发布持久（1364–1365 行换表）之后同一次 `attempt_publish` 里立刻回收（1395 行） | 即使 E153 也写 (i, 0)，切换那一步的回收仍按旧表；E154 下一刻就用新表 |

E153 那边原行（现取）：

```
198:         self.instance_table_rows.iter().any(|(row_instance, valid_up_to)| *row_instance == instance && txg > *valid_up_to)
208:         self.live_roots().iter().filter(|entry| self.is_valid(entry.instance, entry.txg)).map(|entry| entry.txg).min().expect("环里至少有 mkfs 的根")
226:         self.oldest_valid_root_txg().max(self.effective_floor())
1016:     /// 基线的回收：只在挂载 / 切换 / 回退重建 / 抬 F 时调用；门槛为 max(floor, oldest_valid)。
1349:         let old_ring_threshold = self.pool.reclaim_threshold();
1362:             // 物理回收步（测量臂：每次发布之前一步，门槛用「环」；基线：不在这里回收，回收只在
1363:             // 挂载/切换/回退/抬F）。
1364:             if arm.arm.is_measured() {
1366:                 arm.reclaim_measured(old_ring_threshold, blocking);
1422:             // 写失败：根槽保留旧内容（不进环），但这个 txg 已经「用掉」——切换写行发布的 txg 是它 + 1。
1918:         let threshold = self.pool.reclaim_threshold();
1922:             arm.rebuild_allocator(&snapshot, threshold, Some(&blocking), &abandoned_txgs, &candidate_txgs);
1925:         self.pool.instance_table_rows.push((self.pool.current_instance, outgoing_row_txg));
1932:         self.switch_rebuild_and_take_instance(basis_txg, basis_txg);
1944:         // 第一次切换取的新实例一次都没发布成功：它的行是 (i, 0)，不是 (i, basis_txg)。
1945:         self.switch_rebuild_and_take_instance(basis_txg, 0);
```

所以「E153 在洞上偏保守、E154 提前回收」的原因是切换行的 T，不是洞，也不是两边最旧有效根的算法。

### 1.11 什么现象会推翻问题一的判定

- 副本上改了切换行之后，H4a / H4b / H5-1-* / H5′-1-* 仍有 unsafe > 0 的格（实测 0，见 1.7）。
- D23（journal 的角色与格式） 已定项 14 的切换行规则被读成「旧实例一律写 T = 0」（附录 A 原文里写的是「k 最后发布的 txg（没发布过根时 0）」）。
- 在同一个环上按条款逐项重算，门槛 ≥ 237（1.4 表算出 234）。
- 实现里出现了发布之后按环转回收、或挂载写行写 T = 0 的代码（现查 mount.rs 1000 行写所选根的 txg、回收只有 377 与 629 两处调用）。
- H6 那一格：在「只在挂载与抬 F 回收、抬 F 先重算」的时点下，副本上量出复用（本轮没有把装置改成这种时点跑，只读实现推的，见「没做什么」）。

## 二、问题二：「装置与实现的对应」①–⑥ 今天还成不成立

登记原文（`research/prompts/e154-preregistration.md` 523 行，整行）：

```
523: ① 空发布写 c 槽、在 c = 4 那一格与 `rewritten_roles` 的 4 个角色同数；② 区域 = txg mod 3、区域设备 [0, 1, 0]；③ F_生效 = 各盘所带 F 最大值的最小值；④ 抬 F 上限的式子（「非空」除外，见 R9）；⑤ F-扣 的扣住：记账算空闲、分配器不发、带新 F 的根落满两块盘之后放开；⑥ 新实例写行那次发布带恢复后生效的 F，暖机空发布带当前根的 F。
```

六个文件现取的 blob（UTC 16:09 与 16:31 两次取，六个值两次相同）与登记、第二段基线对照：

```
$ for f in …; do printf '%s %s %s\n' "$(git hash-object $f)" "$(git status --porcelain -- $f | cut -c1-2)" "$f"; done
9a492f2739a1780e9c80211a1a9f62e8ed0c7d60 ?? crates/singlefs-core/src/mount.rs
66c9eb1dc842a5147b700c33b5b18a1deba2a964  M crates/singlefs-core/src/allocator.rs
bc8737acf2b37fa12c93041823549913cfd91559  M crates/singlefs-core/src/transaction.rs
ad1ea35c6d344e45a1bb347a2955cdacf42dd864  M crates/singlefs-core/src/recovery.rs
2ecb59a840676f316fb43339171fdb2cae5312ec  crates/singlefs-core/src/root_ring.rs
417b92240034e5f0a6c57ae5ff2b2a25c69e7985  M crates/singlefs-format/src/lib.rs
```

| 文件 | 登记（「三」表） | 第二段基线（「十二」第 3 条） | 第四段现查（「十二」第 11 条） | 今天 |
|---|---|---|---|---|
| mount.rs | b228b7f | fe1acd7 | a358a7b | **9a492f2**（又变了） |
| transaction.rs | 6dd2ef9 | 0d4db4d | bc8737a | bc8737a（同第四段） |
| recovery.rs | ef4da61 | ad1ea35 | ad1ea35 | ad1ea35 |
| allocator.rs | 66c9eb1 | 66c9eb1 | 66c9eb1 | 66c9eb1 |
| root_ring.rs | 2ecb59a | 2ecb59a | 2ecb59a | 2ecb59a |
| singlefs-format/src/lib.rs | 7816f5b | 7816f5b | 7816f5b | **417b922**（变了） |

对比用的旧版：对象库里只有登记那一版（b228b7f、6dd2ef9、ef4da61、7816f5b 在，`git cat-file -e` 通过；fe1acd7、a358a7b、0d4db4d 不在），所以函数级 diff 都是「登记那一版 → 今天」，抽出来放在 `blobs/`。

| 项 | 判定 | 依据（今天的文件:行，与登记那一版的函数级 diff） |
|---|---|---|
| ① 空发布写 c 槽，c = 4 时与 `rewritten_roles` 的 4 个角色同数 | **成立** | transaction.rs 968–990 行 `rewritten_roles`：没有文件、实例表 Carry 时只剩 AllocationTree / AccountingTree / MappingTree / TreeTable 4 个；四者落点都是 `UnitFootprint::OneSlot`（632–637 行）、跨 1 槽（567–572 行）。函数体与 6dd2ef9 逐字相同（`diff` 退出码 0）。抬 F 的推空走 `publish_version` + `InstanceTablePlan::Carry`（mount.rs 641–657 行）。新增的零单元分支 `publish_without_units`（transaction.rs 434 行；mount.rs 740–752、889–902 行）只管树表 0 条的一版（第一个文件版本之前），模型起点之后不经过 |
| ② 区域 = txg mod 3，区域设备 [0, 1, 0] | **成立** | root_ring.rs 35–36 行 `region: checkpoint_txg.0 % ROOT_RING_REGIONS`、`slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION`（文件未变）；singlefs-format/src/lib.rs 185 行 `ROOT_RING_REGIONS: u64 = 3`、192 行 `ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0]`。lib.rs 的改动只有第 3 行文档注释的路径（`git diff` 1 增 1 删：`first-txn-layout.md` → `layout/01-first-txn.md`） |
| ③ F_生效 = 各盘所带 F 最大值的最小值 | **成立** | recovery.rs 351–371 行 `effective_rollback_floor`，与 ef4da61 的函数体逐字相同（`diff` 退出码 0） |
| ④ 抬 F 上限的式子（「非空」除外） | **式子成立；「非空」与失败处理变了** | mount.rs 463–533 行 `rollback_floor_ceiling` + 536–547 行 `ceiling_from_newest_and_non_empty_roots`：有效 = txg ≥ current_floor ∧ 不被 `abandoned_by_table` 判抛弃（475–480 行）；每块盘最新有效根取最小（481–495 行）；第 4 新的非空有效根、不足 4 个取最旧有效根、两者取小（541–546 行）——与 b228b7f 同式。变的：「非空」从「环里有它自己那条记录且事务号非 0」换成「树表里 inode / extent 树根指针与前一条有效根不同」（`user_visible_trees_changed`，444–450、509–528 行，2026-09-17 用户定案）；要比的树表读不出时整次抬 F 返回 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（496–508 行），登记那一版没有这条失败。登记 R9 本来就把「非空」排除在外，式子这一半不触发 S2。另记一处登记没写的小差：实现传进来的 `current_floor` 是现行版本根自己带的 F（mount.rs 583 行 `current.root.rollback_floor`），装置 `rollback_floor_ceiling` 取未被抛弃的可读根里 F 的最大值（装置 390–395 行）；登记那一版实现同样传 `current.root.rollback_floor`，不是今天才变的 |
| ⑤ F-扣 的扣住：记账算空闲、分配器不发、落满两块盘之后放开 | **成立** | allocator.rs 未变（66c9eb1）：221–232 行扣住位、237–240 行开段绕开、243–246 行放开、268 行回收时空闲计数照加。mount.rs `raise_rollback_floor` 与 b228b7f 的函数级 diff 只有三处：删掉 `scan_journal`、错误名改成 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`、`rollback_floor_ceiling` 的调用签名；影子账重算（611–628 行）→ 回收并扣住（629–632 行）→ 推空循环（636–664 行）→ 放开（665 行）的次序不变 |
| ⑥ 写行发布带恢复后生效的 F，暖机空发布带当前根的 F | **成立** | 写行：带文件的一版 mount.rs 878 行、树表 0 条的一版 898 行，都是 `rollback_floor: start.effective_floor`；`effective_floor` 来自 `rebuilt_allocator` 里的 `effective_rollback_floor`（352–357 行）经 995、1016 行传入。暖机：`publish_empty_after` 735 / 748 行 `rollback_floor: current_*.root.rollback_floor`。登记那一版这两处写在 `establish_instance` 里，今天拆成 `publish_rows_on_file_version`（684–711 行）、`publish_without_units`、`publish_empty_after`（715–754 行），带的 F 不变 |

六项都成立（④ 的「非空」除外），第四段那次没核的 S2 按今天的实现补核通过；mount.rs 在第四段之后又变过一次（a358a7b → 9a492f2），我手里没有 a358a7b，只能说「登记那一版 → 今天」这六项没有变。

**六项之外、登记没列而与问题一直接相关的两样：**

| 项 | 实现 / 条款 | 装置 | 登记 |
|---|---|---|---|
| 新实例给上一个实例写的行里的 T | 实现 mount.rs 998–1003 行 `selected_root_txg: effective_root.checkpoint_txg`；中间实例 856–862 行写 0。条款见附录 A | 1540–1547 行一律 0 | R8（48 行）与 5.1「崩溃与重开」（588 行）都没写 T 取什么 |
| 切换写行发布带的 F | 实现没有切换 | 1537、1550 行带 `pool.effective_rollback_floor()`；追踪 `trace-h5-1-first-600-16-4-4-0.log` 483 行失败那次推空 `floor=240`，写行发布 txg 245 持久时 `F=234`（518 行） | R8（48 行）写「**带失败那次发布本来要带的 F**」；登记变异表 M13（811 行）把「写行发布带 F_生效」列为该被抓的改坏，而仓里变异表 `research/mutations/e154_two_gates_serial_rejudge_and_reclaim_timing.tsv` 的 M13 是另一条（H5′ 窗口起点） |

## 三、没做什么

- 仓里一个文件都没改，没跑门禁，没做任何 git 写操作；`blobs/` 里的旧版实现是用 `git cat-file -p` 读出来的只读副本。
- 没在仓里的装置上跑单测、变异表或复跑；修过行、加过开关的装置只在草稿目录的副本里跑，**没跑它们的单测**。改了切换行之后，装置里至少 `fill_phase_injection_eliminates_degeneration_that_original_injection_point_could_not_reach`（断言「达标」）、`run_root_write_failure_history_with_one_failure_and_first_position_injects_exactly_one_failure_and_switches` 这类钉了 H5 结果的单测可能要跟着改，这一点没验。
- 反事实开关是我按条款与实现写的最小改法（切换行 T 取该实例环里最新根的 txg；回退目标须在环里；影子账重算两种时点），**不是被审过的修法**；影子账 `=2` 那种时点只量了 unsafe 与 Q1 标签，别的量（隔离槽数、卡死的原因）没看。
- 「实现在同一个环上不回收」是读代码推的：实现里没有实例切换、没有按准入触发的抬 F，H5 与 H6 这两段历史在 `crates/` 上跑不起来，没有在实现上构造这两个环去量。
- 切换写行发布带的 F（装置带 F_生效、登记 R8 说带失败那次的 F）只指出了不一致，没量它对 Q1 的影响。
- 没读 E153、E154 的实验页，没读 `records/2026-09-17-已分配口径三方与两个实验.md`；E153 的判断只来自它的源码。
- mount.rs 第二段基线 fe1acd7 与第四段 a358a7b 不在对象库里，「对应六项」只核了「登记那一版 → 今天」，中间两次改动各改了什么不知道。
- D23（journal 的角色与格式） 文件在我读的过程中被别的会话改过一次（16:1x 取的 blob 0865da8 → 16:31 取的 0ac8139），附录 A 是 16:3x 现抽的；691、1209 两行里我用到的句子两次都在（`grep -n` 命中行号不变），改了别的哪几行没查。
- 岔路单第 3、4 行要不要按修过的装置重跑、怎么修装置，不归核查员定。

## 附录 A：D23（journal 的角色与格式） 已定项 14 两行原文（`.claude/kb/decisions/23-journal的角色与格式.md`，blob `0ac8139286390dba438d01150473e952aa062376`，UTC 16:3x 现抽）

691 行（索引行，含切换写行规则）：

> 691: 14. **重放的下界（2026-09-02，用户定案）：由所选根给出——只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化。** C113（扫描重建时多版单元的现行版本判定无输入） 定案（2026-09-05）加一条显式例外——管理员回退：从回退候选集（根环里按实例表判仍有效、且 txg ≥ F_生效的根）选 R_old，不施加它之后的任何记录、取新实例代号、在 R_old 指着的那一版实例表上写回退行，第一个新根的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1（2026-09-16 随 C143（inode 号水位在回退后会退回去重发） 定案取 CJ2：回退本来就全环扫描、逐条验证，额外读 0；被抛弃时间线里根槽写失败留下的「只有记录、没有根」的 txg 因此也被跳过；新实例的第一次发布同样从这个最大值 + 1 起，见注 3），回退与第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号，以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退候选集还要 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）：平时 F 不动时深度是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。同一次定案把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表，2026-09-05）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写得进去 ⇒ 瞬时失败，走**实例切换**；写不进去 ⇒ 持续失败，走**转只读到下次挂载**。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份超级块、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。此前写的「只读支不需要分配也不写单元所以必然收敛」说的是只读支自己，不是切换支怎么终止（第八轮反推腿 2.1）。切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据，2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）；**这个量的口径与式子 2026-09-13 由用户定案，权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。**预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡（第八轮反推腿 4.1–4.3）。**「分配失败」按它发生在哪里拆三支**（此前把三种事搅成一格，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」：池一满就整卷只读，而释放空间要写盘，重挂又被预留合取挡住，成了不可逆）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。**实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根取被重发的那个在飞 checkpoint 所基于的根（旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old），行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)，其余按 D18（块里携带什么信息） 已定项 11（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方；旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行），号 ≤ W 的事务照旧（开放 checkpoint 里已完成的事务全部 > W，C287（切换收养开放 checkpoint 的事务后再崩） 按这个 W 封死，2026-09-13 D23（journal 的角色与格式） 已定项 14 的注 4，2026-09-14 用户复核确认）、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。**重放下界遇回退行截断**：所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止——否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销，盘上没有任何痕迹（C124（回退行与重放下界没有会红的检查））。 两份镜像何时算「记录在」：**任一份自证校验和过即在**（2026-09-14 用户收尾弹窗定案，E142（第一个事务的干跑） 第七次跑记的 G9 收口；崩溃点重放里的检查 2026-09-14 由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件，C328（journal 两份镜像何时算「记录在」没有条款） 同日还清）。 **状态：已定。** ⚠️ **第六条口径（2026-09-13，D16（发布语义） 已定项 4）**：一次发布整体施加或整体不施加——前缀不得停在一次发布的中间；正文见已定项 14 那一节的第六条（2026-09-13 已写进正文）；引用前缀口径时六条一起引。

1209 行（正文，含 (i, T) 判法与回退候选集）：

> 1209: **显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限；表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；第一个新根的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1（2026-09-16 随 C143（inode 号水位在回退后会退回去重发） 定案取 CJ2：回退本来就全环扫描、逐条验证，额外读 0；被抛弃时间线里根槽写失败留下的「只有记录、没有根」的 txg 因此也被跳过；新实例的第一次发布同样从这个最大值 + 1 起，见注 3）；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合 2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」2026-09-17 用户定案指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算（里程碑「第二个事务」步 4 / 步 5 代码三方三轮，`research/prompts/m2-step45-code-r3-main-verification.md` 第六节 5）；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西，里程碑「第二个事务」三方第一轮攻方腿指出两种读法（`research/prompts/m2-r1-main-verification.md` 第三节）（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。

## 附录 B：草稿目录里的东西（`/tmp/claude-1000/verify-forks/e154/`）

| 路径 | 是什么 |
|---|---|
| `research/` | 仓里 `research/` 的副本 + `tools/patch_trace.py`（打印与切换行开关） |
| `research2/`、`research3/`、`research4/` | 再加 `tools/patch_shadow.py`（影子账重算开关）；`research3` 多认 `H5-2-首`；`research4` 再加 `tools/patch_inring.py`（回退目标须在环里、只跑 H6） |
| `full-nofix.out` | 不开开关的全量输出，与 stage4 产物逐字节相同 |
| `full-fixrow.out`、`full-fixrow-shadow1.out`、`full-fixrow-shadow2.out` | 全量：改切换行；再加抬 F 前重算 / 每次回收前重算 |
| `h6-*.out` | 只跑 H6 的四个变体 |
| `trace-*.log` | 五份单格追踪（文件名写了历史、臂与几何） |
| `blobs/` | 登记那一版与今天的 mount.rs / transaction.rs / recovery.rs |
| `tools/tabulate.py`、`tools/diffruns.py` | 数 unsafe、比两份输出 |

```
d510ff5d288d9bc40e0838e4a81e19089b72f303a2ccd019c85d9f5e2fa3c978  full-nofix.out
b984f920c86f6834023946a172f0addd808fef9ffaa428d489b2399f24ac7c81  full-fixrow.out
4355f8d780f6fbaf997059f0170a7a13d0da94cc410bea263e7ef88c45bea2a0  full-fixrow-shadow1.out
9246bee2285a5bd966b04a6afffbee63400dd5adbcdafed08c6f41cb2a69e8f4  full-fixrow-shadow2.out
cb9ba14602e4bde203b2c4401b5b794b4eda8ccdd2fe4c0284b48ff089fea141  h6-nofix.out
19521ecfb736686417ec3ec73a27036d3d420c623c9f2da6f2788c1b70caa264  h6-inring.out
298515937aa4ab94a2a83a449305f37520975193498cd3c0a828f50fddcb4da2  h6-inring-shadow1.out
f879319543170f1e4a8377fddd4cf5b8eb00c282e92ea3fb3d8130d035dd003b  h6-inring-shadow2.out
d470c88ec51e490ad1c9fdfb6757c0a7f7563ee7e0abc53fcac0c97c13f7bb95  trace-h4a-600-16-4-4-0.log
ea8e3302fe3eb59440e1fbd0286aa5829a22142d5acabcf3cbbf854ed0d29b09  trace-h5-1-first-600-16-4-4-0.log
ef4183a05ed4dc65c5926b58de15c4ebc13790545b4b652f4d017914a3d67c0c  trace-h5-2-first-600-16-4-4-0.log
42a1df1d1991a732384e8425d302c219995484941f2b49bf4569eaf714ccb7a0  trace-h6-4000-1-4-4-1.log
1bcfbb94ce22ad8d243d06d5f0ecad8a7a6768d0caf6a74b400c3da86cfd3a7f  trace-h6-rejudge-inring-shadow1-4000-1-4-4-2.log
530fbd0da68e640d6b0dcff616515198f4149ca6088469c0e7c405182c06ba10  trace-h6-serial-shadow1-4000-1-4-4-2.log
bf06b7eebf1066442e082c810db6b8ca7f58a56785e777d7f1dc265cb29e0b3e  tools/patch_trace.py
854201c9dfc8daa235e7a4d651f5e682842b65ed8fdf2c8249c94280e7eb5158  tools/patch_shadow.py
b2c2c68572310a4dd10c31279d776da81f9c384d246fc4597ecd43de9c35c78f  tools/patch_inring.py
2aeb36f188665ffa5818839c8dd0efdb9c7f15a968fa11986684bcc3445ecf25  tools/tabulate.py
0065bc5114578c5e8cbfc57dc7e517beeaa466fd83b0128e63224920f3d3ba9d  tools/diffruns.py
```
