# 攻方腿（Opus）m2-supp3-item1-code-r2：Y1、Y2、Y5（2026-09-18，时刻均 UTC）

副本上的数，不是入库装置上的数。所有变异、探针、改法都只施加在 `/tmp/claude-1000/m2-supp3-item1-r2-opus/` 下的仓副本（`rsync -a --exclude target --exclude .git`，每份副本自己的 `target`）；主工作区 `crates/`、`.claude/gate.d/` 一字未改。

## 各格判定一览

| 格 | 判定 | 满足的字面分句 | 一句话 |
|---|---|---|---|
| Y1 | **字面打中第二分句（1 条变异、4 段历史），但方向对实现有利** | 「F 在空档里、机理是登记的那一类，却报新发现」 | core 读实例表的函数一改坏（Y1e：丢最后一行），基线里以第 1 条收尾的 4 段（种子 174、435、616、693）原样走到同一步、同一个 F、同样多算 6 × 16384 字节，却报成新发现；而这 4 段恰是 Y1e 在快档、专门取样点、1000 × 40 上唯一被判出的地方。第一分句（接走 F 不在空档里的）在基线与 13 条变异下共 21016 段历史（13 × 3 份日志加基线 4 份）里 0 次 |
| Y1 附带 | 字面不中，记一笔 | — | 抬 F 一个落点都不回收（A9）时，F 恰落在空档里的 11 段仍被第 1 条接走，多算 114688–606208 字节，不是登记的 6 槽：第 1 条只看「F 在空档里」，不看多算的量，空档里的别的机理照样被遮 |
| Y2 | **打中第二分句** | 「一条只改挂载路径里记录改写的变异，快档与专门取样点都不红」 | 挂载里（写行、暖机）复用改写时不改分配代（模式 a）、挂载里每次分配的分配代记成 txg − 1（模式 b），入库的快档与专门取样点两条用例输出与基线逐字相同（`diff` 空），`test result: ok` |
| Y2 第一分句 | 没打中 | — | 基线 3000 × 40（broad）、3000 × 30（reuse）、快档、专门取样点：`HarnessJudgement` 0 次 |
| Y5 | 没打中实现；测试的判别力缺一格 | — | 差分 60 趟、17654 个状态（线程 1–32、片长 1–7 / 按线程数 / 整条一片、有无观察者、乱切段）计数结构与观察者所见逐项相等。并片时「不看 journal 那一遍的第一处」取后一片（Y5m2）：入库测试 10 次 0 次红——`first_ignored_violation` 只存原因文字、不带持久集合，那条对拍用例里各状态文字相同 |
| Y5 附带 | 字面不中 | — | 工作线程 panic 时，调用方拿到的 panic 载荷从原来的消息变成 `a scoped thread panicked`（原消息仍由钩子打在输出里，5 / 5 次是序号最小的那一处） |

## 复跑与文件

模型目录 `research/prompts/m2-supp3-item1-code-r2-opus-model/`，原样日志在其 `logs/`。文件的 sha256（`SHA256SUMS` 同文）：

```
61ce32a838916988dfc24b07a00bc4b90dc8444ac259e2a38d8d42b0d017c8c5  opus_r2_gap.rs
dab05aa490cc674452cda9b39b772376cc5cad6067637aa53fa1899790caa58f  opus_r2_y2.rs
802440563e03ccae75e362922553d8440bd8980cb986e1eec2840f58acb9d6bc  y5-differential.rs
b8e52c83f12d2d739106a4d9333d0478d129bef8735019af6887665186b981a4  run-mutants.py
5fabe99e6960135671fa0822776853c72794dd7cfb60d0fe989c9552b27e2f96  y2e-patch.py
67fcf33fb9ced251010402a2de947a0670fc2550d40cccffd2f535263940a68f  y2-patch.py
abc9776faf6a252db673c2c68ef900a16d705df40f7263bfa9a758256ea0232c  y2-proposal-patch.py
7252f284d0a11d730f827531912809f094c67822043afa241caaa14d8cdfce43  run-y5-diff-mutants.sh
295b9d9d6fe0416442bbc9e38aec4a15eff628423deadac6c66c5c836ac1d973  run-y5-mutants.sh
706360812201c25c8d5b83f6a2106f595769d4f999662912d216cab634e0bb84  mutants.tsv
62f6c14416b4300f9ccbbebe2012495bfac3467f8160d8a7238162ee197714eb  y5-mutants.tsv
```

复跑（`M` = 模型目录的绝对路径；每步先 `rsync -a --exclude target --exclude .git <仓>/ <副本>/`，副本各自的 target）：

```bash
# Y1：探针放进副本，逐条变异 × 三组规模（比重:第一个种子:种子数:步数）
cp $M/opus_r2_gap.rs SLOT1/crates/singlefs-harness/tests/
python3 $M/run-mutants.py --slot SLOT1 --logs LOGS/Y1 --ids BASE,Y1a,Y1b,Y1c,Y1d,Y1e,A1,A2,A5,A6,A8,A9,A10,Y2d \
  --scales broad:0:96:30,reuse:0:48:30,broad:0:1000:40 --threads 12
# 基线大档：同上 --ids BASE --scales broad:0:3000:40,reuse:0:3000:30
# Y2：挂载里的变异（环境变量 OPUS_Y2 = off / a / b / c / e 选），跑入库的两条用例
python3 $M/y2-patch.py SLOT2 && python3 $M/y2e-patch.py SLOT2
#   模式 c（只在挂载外生效的 N2，对照用）是在 SLOT2 上再做一次定点替换：
#   旧 `            if !(in_mount && opus_mode() == "a") {` → 新 `            if !(in_mount && opus_mode() == "a") && !(!in_mount && opus_mode() == "c") {`
cd SLOT2 && for m in off a b e; do OPUS_Y2=$m cargo test --release -p singlefs-harness \
  --test second_transaction_supplement_three_random_history -- --exact \
  random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation \
  reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms --nocapture; done
# Y2 改法：SLOT2 拷一份为 SLOT2P，python3 $M/y2-proposal-patch.py SLOT2P，放进 opus_r2_y2.rs / opus_r2_gap.rs 再跑
# Y5：差分探针追加到副本的 tests/second_transaction_step_zero_layer0.rs 末尾
cat $M/y5-differential.rs >> SLOT5/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
cd SLOT5 && OPUS_TRIALS=60 cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 \
  -- --ignored --exact opus_r2_parallel_matches_the_single_threaded_walk --nocapture
bash $M/run-y5-mutants.sh SLOT5M LOGS/Y5m 10 Y5m1 Y5m2 Y5m3         # 入库测试判不判得出
bash $M/run-y5-diff-mutants.sh SLOT5 LOGS/Y5d 20 Y5m1 Y5m2          # 差分探针自证会红
```

开工时核开工快照：12 个文件里 11 个 `OK`，`.claude/kb/milestone/02-second-txn.md: FAILED`（mtime `2026-09-18 20:40:21 UTC`，晚于快照记的 20:23Z）。这条腿没有引那份文件的任何一行，只报事实。
开跑前 `ps` 看过：没有 `qemu-system`、`vm-bench.sh`、`fio`、`e152-*`、别的 `cargo` 在跑；load average 约 6.5（32 核）。

## Y1 收窄之后的第 1 条

被判的那一句：`crates/singlefs-harness/src/history.rs` 第 934 行 `        && observation.raised_floor_lands_only_on_abandoned_roots == Some(true)`；它的值由第 2030 行起的 `raised_floor_lands_only_on_abandoned_roots` 经 `singlefs_core::recovery` 的 `readable_roots` / `choose_root` / `instance_table_of_root` 算。

**独立的空档读法（探针 `opus_r2_gap.rs`）**：不经 core 的实例表解析。每一步之后用 checker 的 `chosen_superblocks` / `valid_roots` 记根环；每次成功的回退，按执行器同一条选法（`index_from_newest % 根数`）算出目标 (实例, T)，空档 = 开区间 (T, 回退之前环里最新根的 txg + 1)。F 落在任一空档里记 `in_gap=true`。以第 1 条收尾的记一行 `GAP`，以新发现收尾而发生在抬 F 那一步的记 `NEWRAISE`（带 `lands=` 即实现算出的值）。

### 基线（副本，未施加变异）

```
SUMMARY first=0 seeds=3000 ops=40 weights=各类操作都抽（快档、大档） {"completed": 1041, "form1_in_gap=true": 30, "known_red_0": 1929, "known_red_1": 30}
SUMMARY first=0 seeds=3000 ops=30 weights=偏向抬 F 之后的复用（第 121 行那一类的取样点） {"completed": 967, "form1_in_gap=true": 12, "known_red_0": 2021, "known_red_1": 12}
```

第 1 条接走的 42 段 F 全在空档里，多算全是 `overcount_bytes=Some(98304)`（6 × 16384，登记的那 6 槽）。基线上没有一段以「抬 F 之后 I-3.1 多算」的新发现收尾。

### 变异（`model/mutants.tsv`；每条 × 快档 96 × 30、专门取样点 48 × 30、broad 1000 × 40）

| 编号 | 改了什么 | 第 1 条接走的（F 在空档里 / 不在） | 抬 F 那一步的新发现里 F 在空档里的 |
|---|---|---|---|
| Y1a | core 实例表行的所选根 txg 读小 1 | 0 / 0、0 / 0、4 / 0 | 0 |
| Y1b | 同上读大 1 | 0 / 0、0 / 0、4 / 0 | 3（种子 693 第 8 步、854、936；多算都是 98304） |
| Y1c | core `abandoned_by_table` 把 txg = T 也算被抛弃 | 1 / 0、1 / 0、20 / 0 | 0 |
| Y1d | 回退不查目标是不是被抛弃时间线的根 | 1 / 0、1 / 0、11 / 0 | 0 |
| Y1e | core `instance_table_of_root` 丢掉最后一行 | 1 / 0、1 / 0、6 / 0 | **4**（种子 174、435、616、693；多算都是 98304） |
| A1、A2、A5、A8 | 第一轮的回收门槛与 checker 候选集差一 | 1 / 0、1 / 0、11 / 0 | 0 |
| A6 | checker 把 txg = T 也当被抛弃 | 0 / 0 三档 | 0 |
| A9 | 抬 F 一个落点都不回收 | 1 / 0、1 / 0、11 / 0 | 0 |
| A10 | 抬 F 回收多一代 | 1 / 0、0 / 0、6 / 0 | 0 |
| Y2d | 仍分配的记录落盘时分配代写小 1 | 1 / 0、1 / 0、12 / 0 | 0 |

第一分句（接走而 F 不在空档里）：所有日志合计 `GAP` 181 行、`in_gap=false` 0 行（`grep -c` 原样见 `logs/`）。**没打中**。

### 打中：第二分句，Y1e（与 Y1b 的 3 段同形）

同一个种子，基线与 Y1e 各一行（原样，截到前 80 字）：

```
GAP seed=174 step=10 F=6 in_gap=true gaps=[step3:(2,4)->(4,7)] ring_before=Some(
NEWRAISE seed=174 step=10 F=6 in_gap=true gaps=[step3:(2,4)->(4,7)] lands=Some(f
```

Y1e 那一行全文：`NEWRAISE seed=174 step=10 F=6 in_gap=true gaps=[step3:(2,4)->(4,7)] lands=Some(false) turned=false violations=[("I-3.1", "盘 0：记账的已分配 Some(1376256)，遍历全部有效根得到 1277952")]`，差 98304。种子 435、616、693 同形（基线同一步、同一个 F，Y1e 下 `lands=Some(false)`，差都是 98304）。历史每一步由 `generate_history_with_weights(HistorySeed(s), 40, &GenerationWeights::BROAD)` 给出；那一步被第 1 条放过，是因为第 934 行要 `Some(true)`，而 `raised_floor_lands_only_on_abandoned_roots` 最后一步 `newest_table.rows.iter().any(...)` 读的是被 Y1e 改坏的那份行表。

**四句**：

1. 分不分辨臂：分辨。可比的另一臂是「按 checker 的独立解析判空档」（第一轮实现员报 checker 的实例表解析不公开）。那一臂在这 4 格上读到的是没改坏的行，会说 `true`，第 1 条接走（推的：我的独立空档读法在这 4 格上都说 `in_gap=true`，量过；「checker 臂会接走」按第 934 行与分类次序推，没实现）。
2. 系统当时看不看得到判别它的东西：看不到。匹配函数只经 core 读，core 读错时它没有第二份读法可比。
3. 满足字面哪一分句：Y1 触发观测第二分句「F 在空档里、机理是登记的那一类，却报新发现」——F 在空档里（独立读法），多算的量与登记那 6 槽相等，报成 `CheckerViolations { invariants: ["I-3.1"] }` 新发现。
4. 跑前条款的改法（「改代码、补会红的用例与变异」）在这 4 格上：**这 4 格是 Y1e 被判出的全部**。Y1e 在快档（1 / 1 段，与基线同）、专门取样点（同基线）、1000 × 40 上唯一的新发现就是这 4 段。换成独立读法，这 4 段会回到第 1 条，Y1e 在三档上都判不出（推的）。

**判定**：字面打中，判据方向写反了一格——core 的读根 / 读实例表错了时，第 1 条「跟着错」把登记的那一类报成新发现，这对捶打是好事（它是 Y1e 唯一的报警），不是缺陷。按「判据自己也会写错」，这一格属于「打中归错了判据」一类：它该进「判别力」一侧，不该进「错判」一侧。我不建议据此改代码；如果主 agent 要改，改之前先在这 4 格上核「改完之后 Y1e 还红不红」。

### 附带：空档里的别的机理照样被遮（字面不中）

A9（抬 F 一个落点都不回收，`reclaim_floor(CheckpointTxg(0), None)`）下 1000 × 40 第 1 条接走 11 段，F 全在空档里，多算（原样摘自 `GAP` 行）：

```
seed=80 step=21 F=8 overcount_bytes=Some(606208)
seed=174 step=10 F=6 overcount_bytes=Some(212992)
seed=254 step=15 F=4 overcount_bytes=Some(114688)
seed=693 step=14 F=5 overcount_bytes=Some(278528)
seed=832 step=33 F=9 overcount_bytes=Some(442368)
```

（另 6 段同形，全文在 `logs/Y1x/A9-broad-0-1000x40.log`。）登记那一类是 6 槽 98304；这些是 7–37 槽。第 1 条只问「F 落在空档里」「只有 I-3.1 且多算」，不问多算的是哪些槽，所以抬 F 回收整个坏掉时，F 恰落在空档里的那些段被当成第 43 行接走。A9 在同一档里另有 196 段新发现、快档 17 段，变异本身判得出；被遮的是那 11 段的机理归类。Y1 触发观测只写了「F 不在空档里」这一分句，这一格不满足字面。

候选改法（只在我的模型上想过、没实现、被攻过零轮）：第 1 条再加「多算的字节 = 回退目标根独占、且释放代 > 新 F 的那几个槽」这一类量的相等。今天做不到：要按 checker 读法走回退目标根的树，matching 函数手上只有违例文字里的两个数。

## Y2 执行器的两条判定

被判的那一句：`crates/singlefs-harness/src/history.rs` 第 1775 行 `    // 挂载写出的写行与暖机那几次发布、零单元发布，这里不比分配记录（挂载交回的东西里没有挂载之前那一版的记录）。`，下接可写挂载与回退都走 `AppliedStep::judged_by_outcome_only`。

### 第一分句（合法历史上误红）：没打中

基线副本上（`opus_r2_gap.rs` 把执行器判出的逐条记 `HJ` 行）：快档 96 × 30、专门取样点 48 × 30、broad 3000 × 40、reuse 3000 × 30，`HJ` 行 0 行，收尾计数见 Y1 节基线两行。冷启动判定同：入库两条用例的冷启动结局 `FileRead：78 次`、`NoFile：19 次`（快档），`FileRead：8 次`、`NoFile：1 次`（专门取样点），没有一次 `Failed`。

### 第二分句：打中

装置（`y2-patch.py`，只在副本上）：在 `mount.rs` 的 `establish_instance` 取号之后立一个线程局部标志（写行、暖机两段都在它的作用域里），`allocator.rs` 按环境变量 `OPUS_Y2` 在标志立着时改记录：

- `a`：`make_room_for_record_on_device` 复用改写已回收记录时不改分配代（第一轮 N2 那一处，只在挂载里生效）；
- `b`：挂载里每一次 `record` 把分配代记成 txg − 1；
- `c`（对照）：N2 只在挂载外生效；`off`：不变，只计数。

计数（`off`，入库两档同种子、同比重，`opus_r2_y2.rs` 原样行）：

```
Y2 mode=off tier=fast new_findings=0 mount_rewrites=154 other_rewrites=242 mount_records=4129
Y2 mode=off tier=reuse new_findings=0 mount_rewrites=368 other_rewrites=450 mount_records=2568
```

即快档里复用改写已回收记录共 396 次，154 次（39%）落在写行与暖机里；专门取样点 818 次里 368 次（45%）。

跑入库的两条用例（`random_histories_fast_tier_…` 与 `reuse_heavy_random_histories_…`），四种模式下去掉用时行之后与 `off` 逐字相同（`diff` 输出为空，脚本打印 `IDENTICAL-a`、`IDENTICAL-b`、`IDENTICAL-e`），`a` 的原样行：

```
历史 48 段：跑完 20、以已知红收尾 {0: 27, 1: 1}、新发现 0；根环转过一圈的 45 段；最高 txg 41
test reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms ... ok
历史 96 段：跑完 45、以已知红收尾 {0: 50, 1: 1}、新发现 0；根环转过一圈的 62 段；最高 txg 37
test random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 10.93s
```

对照 `c`（同一处、只在挂载外生效）快档 17 段、专门取样点 28 段新发现——判定本身抓得到这一类，只是挂载里的不看。

**四句**：

1. 分不分辨臂：分辨。今天的臂（挂载不判）在 `a`、`b` 上两档都绿；改法臂（下）在 `a` 上快档 7 段、专门取样点 12 段红，`b` 上 72 / 47 段红。
2. 看不看得到：看得到。第 1775 行的理由「挂载交回的东西里没有挂载之前那一版的记录」与公开接口不符：`MountOutput` 交回 `    pub effective_root: RootRecord,`（`crates/singlefs-core/src/mount.rs` 第 158 行）与 `    pub row_publish: PoolVersion,`（第 163 行）、`    pub warm_up_publishes: Vec<PoolVersion>,`（第 165 行），`crates/singlefs-core/src/recovery.rs` 第 378 行 `pub fn allocation_records_under_root(` 是公开的，挂载之前的镜像执行器手上也有（`HistoryPool::image`）。
3. 字面分句：「一条只改挂载路径里记录改写的变异，快档与专门取样点都不红」——`a` 正是只改挂载路径里的记录改写。
4. 改法在打中的格上：见下表。

### 改法（只在我的模型上量过、被攻过零轮）

`y2-proposal-patch.py`：可写挂载与回退成功之后，取挂载之前的镜像上 `effective_root` 那棵分配记录树当「上一版」，逐次比写行、每一次暖机（带文件的那几版）的 `allocation_records`，改写或新增的记录分配代要等于那一次的 txg，不等就 `AllocationGenerationIsNotThePublishTxg`；零单元的那几版跳过。

| 格 | 今天 | 改法 | 标 |
|---|---|---|---|
| `a` 快档 / 专门取样点 | 0 / 0 段新发现 | 7 / 12 段，签名都是「改写或新增的分配记录的代不是这次发布的 txg」 | 量过（`logs/Y2P-mode-a.log`、`Y2P-a-probe-fast.log`） |
| `b` 快档 / 专门取样点 | 0 / 0 | 72 / 47 | 量过（`Y2P-mode-b.log`） |
| `c`（挂载外，对照） | 17 / 28 | 17 / 28（不变） | 量过 |
| 基线误红：快档、专门取样点、broad 3000 × 40、reuse 3000 × 30 | 0 | 0（`HJ` 行 0，收尾计数与基线逐字同） | 量过（`Y2P-off-*.log`） |
| 挂载在写行之后半路失败（`Err`）那几次 | 不判 | 仍不判（改法只看 `Ok`） | 推的 |
| 回退之后 `effective_root` 是恢复施加 journal 重建出来、根槽从没落盘的那一版 | — | 读它的分配记录树可能读不出，读不出就不判（`.ok()?`） | 推的，没造出这种历史 |

改法之下 `a` 快档报出的第一段（原样截取）：`HJ seed=7 pos=Operation(23) AllocationGenerationIsNotThePublishTxg { records: [AllocationRecord { device: DeviceIdentity(0), slot: SlotNumber(50240), span_slots: 2, generation: CheckpointTxg(4), is_released: false }, …`。

### 不归这一格、顺带量到的

- Y2d（记录**落盘**时仍分配的分配代写小 1，内存里的记录不变）：三档与基线计数相同、`HJ` 0 行。执行器判的是写入进程内存里的记录，盘上那一格归收口表第 44 行（checker 缺判据），与第一轮判决第二节第 3 行写的一致，不算打中。
- 模式 `e`（B6 只在可写挂载与回退自己的恢复里生效）：入库两条用例与基线逐字相同。我没有核挂载路径走不走到补齐字节那一句（没加计数器），所以这一格**不能**当「没打中」的证据，只记「试过、判别力未知」。

## Y5 并行等价

### 差分：没打中

`y5-differential.rs` 追加进副本的 `tests/second_transaction_step_zero_layer0.rs`：单线程原写法照 `00c9d4f` 的 `enumerate_layer0_selecting_versions_observing_each_state` 逐行抄（只调公开的 `evaluate_state_for_versions`），与 `enumerate_layer0_in_state_slices` 在同一组输入上各跑一次，比整个 `Layer0Tally`（含 `first_violation`、`first_ignored_violation`、每条不变量的第一处）与观察者看到的（持久集合 + 看 journal 那一遍恢复报告的 `Debug` 文字）逐个相等。输入按趟号取（SplitMix64）：到 E 的固定脚本那条流，段按写的次序重切，段长 0–5、每个写 1/12 漏掉（不在任何段里）、1/12 同时进下一段（同一个写在两段里）、1/6 段内倒序；每段 1/4 展开；版本表原样或把 B 的内容换成 C 的（造违例）；线程 1、2、3、5、8、32；片长按线程数定、整条一片、每片 1–7 个；一半带观察者。

```
OPUS_Y5_SUMMARY trials=60 first_trial=0 states=17654 trials_with_violations=39 trials_with_checker_violations=60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 262.82s
```

60 趟 `equal_tally=true equal_observed=true`；线程数分布 1:8、2:12、3:12、5:9、8:12、32:7 趟，带观察者 32 趟，片长按线程数定 11 趟、固定片长 49 趟。

探针自证会红（`run-y5-diff-mutants.sh`，20 趟）：

```
Y5m1 diff exit 101 first_unequal: OPUS_Y5 trial=4 segments=115 expanded=28 states=243 threads=3 slice=StatesPerSlice(2) observer=true violations=65 ignored=65 checker_violated=805 equal_tally=true equal_observed=false | …
Y5m2 diff exit 101 first_unequal: OPUS_Y5 trial=1 segments=131 expanded=36 states=251 threads=5 slice=StatesPerSlice(1) observer=false violations=30 ignored=30 checker_violated=810 equal_tally=false equal_observed=true | …
```

（Y5m1：收到哪片就把手上等着的按片号并掉、不等空缺；Y5m2：并片时 `first_ignored_violation` 取后面那一片的。）

### 入库测试的判别力缺一格（字面不中）

`run-y5-mutants.sh`，每条变异跑 `crates/mutations.tsv` 第 131–138 行点名的两条集成用例各 10 次、`crash::tests` 一次：

```
Y5m1 lib exit 0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.04s
Y5m1 integration red 10 / 10 ; last: test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 1.73s
Y5m2 lib exit 0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.02s
Y5m2 integration red 0 / 10 ; last: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 3.97s
Y5m3 lib exit 0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.03s
Y5m3 integration red 0 / 10 ; last: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 2.61s
```

（原样，`logs/Y5m-run.out`。）

Y5m2 两边都判不出：单测 `absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation` 里前一片的 `first_ignored_violation` 是 `None`（`crates/singlefs-harness/src/crash.rs` 第 1662 行 `            Some("后面那一片的 Ignore"),` 是期望「取后面的」那一格），「前一片有、后一片也有」这格没有；集成用例 `one_state_slices_on_eight_threads_…` 比的是整个 `Layer0Tally`，而 `first_ignored_violation` 只存原因文字（第 833 行 `            tally.first_ignored_violation = Some(reason);`，不像 `first_violation` 带持久的写），那条流上各个违例状态的原因文字多半相同（推的：没逐状态打出来），取哪一片都一样。实现本身在差分里对（60 趟），缺的是会红的用例与变异表一行。Y5 的触发观测写的是「漏片、重片而断言不红」，这一格是「并片次序错而断言不红」，字面不中，交主 agent 判归不归反向接受条款。

Y5m3（不管设多少只起 1 个线程）入库测试不红是应当的，那一支归门禁 54 号（Y6，不归我）。

### 工作线程 panic（字面不中）

`y5-panic-probe.diff`（只在副本上）：`OPUS_PANIC_AT=28,30` 时持久写数为 28 或 30 的状态 panic（到 E 的平时那 108 个状态里序号 14 与 17）；单线程原写法与并行版（每片 1 个状态）各包一层 `catch_unwind`。线程 1 与 8 各 5 次，10 / 10 次都是（原样，8 线程第 1 次）：

```
OPUS_PANIC reference payload=Some("OPUS_PANIC persisted_count=28")
thread '<unnamed>' (3597452) panicked at crates/singlefs-harness/src/crash.rs:782:13:
OPUS_PANIC persisted_count=28
OPUS_PANIC parallel threads=8 payload=Some("a scoped thread panicked")
```

（行号是副本里注入之后的，782 在注入的那段里，调用方那一处 1259 是副本的 `std::thread::scope`，原文件里是第 1252 行。）不卡死、10 次都先打出序号最小那一处的消息；变的是调用方拿到的载荷。今天没有用例拿层 0 枚举的 panic 载荷做 `#[should_panic(expected = …)]`（`grep -rn should_panic crates/singlefs-harness/` 只命中 1 行，是 `crash.rs` 第 1577 行一条注释，不是属性），所以不影响判红，只影响读日志的人：门禁输出的最后一句 panic 从真消息变成 `a scoped thread panicked`，真消息在它上面几行。

## 改法各修哪一格

| 改法 | Y1 第二分句那 4 格（Y1e） | Y1 附带（A9 空档里被遮的 11 段） | Y2 `a` / `b` | Y5m2 判别力 | 标 |
|---|---|---|---|---|---|
| 不改（今天） | 报新发现（是 Y1e 唯一的报警） | 被第 1 条接走 | 两档不红 | 入库测试不红 | 量过 |
| 第 1 条改用 checker 的独立解析判空档 | 回到第 1 条，Y1e 三档都判不出 | 不变 | — | — | 推的 |
| 第 1 条再按多算的槽比 | 不变 | 转成新发现 | — | — | 推的，没实现 |
| Y2 改法（挂载的写行、暖机按生效根的盘上记录比） | — | — | 7 / 12、72 / 47 段红，基线 0 误红 | — | 量过（副本） |
| 给并片单测补「前后两片都有 `first_ignored_violation`」一格；`first_ignored_violation` 也带持久的写 | — | — | — | 推的：这两处任一处都让 Y5m2 红 | 推的 |

所有改法只在我的模型上量过或推过，被攻过零轮。

## 没打中的形状与取样范围

- Y1 第一分句：13 条变异（core 实例表解析 ±1、core 与 checker 判被抛弃的边界、回退不查被抛弃、读实例表丢行、回收门槛 5 种、落盘分配代），每条三档；加基线 6144 段。21016 段历史里 `GAP` 181 行，`in_gap=false` 0 行。没试：改 `readable_roots` / `choose_root` 本身（读根环、择新序）的变异；checker 的 `valid_roots` 改坏时我的独立读法跟着坏（它就用 checker 的读法）。
- Y2 第一分句：基线 6144 段历史（快档、专门取样点、两份 3000 段）0 次 `HarnessJudgement`。没试：抬 F 半路报错（`Err`）那一支的判定在合法历史上会不会误红——入库快档与专门取样点两档（144 段）里抬 F 的 `Err` 只见到 `RollbackFloorAboveCeiling` 与 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`（两份 3000 段的没数），按名字都在发布之前，`records_on_disk_before` 与现行版本相同，按代码推不会误红，没造出发布之后半路失败的历史。
- Y5：60 趟差分（17654 个状态）；panic 注入 10 次；没试：观察者自己 panic 时不卡死（按代码读：接收端随闭包丢掉、工作线程下一次 `send` 失败就退，没跑）；带观察者时前面那片最慢、后面各片的报告全压在内存里（到 E 的流上两百多万个状态，今天带观察者的只有几十个状态的两条流，没量内存）；`SINGLEFS_LAYER0_THREADS` 的读法只读了代码与单测，没另造环境。

## 这条腿自己的限度

- 副本上的数，不进 kb；主 agent 要引须在入库装置上重做。
- 独立空档读法只认「回退目标 T 到回退前环里最新根」这一段开区间；多次回退叠在一起时，我没有核它与实例表按行判的被抛弃集合在每一格上逐一相等，只在 181 行里对上了「第 1 条接走的都 `in_gap=true`」。
- Y2 的挂载内标志装在 `establish_instance` 取号之后，它罩住写行与暖机两段；取号之前的重建分配器不在其内（那一段不写记录）。
- `run-mutants.py` 的 `BASE` 我实际是直接跑探针二进制（命令同上文），不是经脚本；日志同名在 `logs/`。
- Y5 差分用的单线程原写法是我按 `git show 00c9d4f:crates/singlefs-harness/src/crash.rs` 抄的，抄错会让两边一起错；探针的自证（Y5m1、Y5m2 都红）只证它对这两处敏感。

## 没做什么

- 不判 Y3、Y4、Y6，没读这一轮别的腿与主 agent 核实。Y5m3（只起 1 个线程）怎么判红归门禁 54 号，没跑门禁。
- 没跑门禁任何阶段；`stage-owners.tsv` 里登记给 three-way-attack 的阶段：按共用约束那条 `awk` 数出 0 个。
- 没改主工作区任何文件；写的只有这份报告与 `research/prompts/m2-supp3-item1-code-r2-opus-model/`。副本与完整日志留在 `/tmp/claude-1000/m2-supp3-item1-r2-opus/`（`base`、`slotY1`、`slotY2`、`slotY2p`、`slotY5`、`slotY5m`、`slotY5d`、`slotY5p` 各带 target，约 10 GB 量级），模型目录里的 `logs/` 是拷过来的原样日志，没拷的只有 `.pristine` 备份。
