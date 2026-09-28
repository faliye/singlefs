# 增补 2 收口表第 43 行复查：已知红今天还走不走得到、机理、C379

调查员，2026-09-28。只查不修；重型测试一条没跑。

## 〇、现场与口径

- 仓副本：`/tmp/claude-1000/closeout-recheck-2026-09-28/row43/repo`，从主工作区 `rsync -a --exclude target --exclude .git` 拷来。主工作区 HEAD `e5253e8a2afd26d3352a6f02fe015d67858eab89`，另有未提交（已暂存）的改动：`git diff HEAD --stat -- crates | tail -1` 报 `50 files changed, 4774 insertions(+), 868 deletions(-)`，`git diff HEAD -- crates | sha256sum` = `18c16f0c0754ea4c7e9340f2a37ec69b2c136d46212973a0ed18de0f95632a66`。副本就是这个工作区状态，报告里引的行号都是主工作区（工作区状态）现取的。
- 编译目录 `CARGO_TARGET_DIR=/tmp/claude-1000/closeout-recheck-2026-09-28/row43/target`，release。每条命令经 `research/scripts/capped.sh 16` 与 `research/scripts/run-with-memory-cap.sh 16G`，`nice -n 19`。
- 开跑前 `ps` 看到的：一条别的会话的 `cargo build --release -p singlefs-checker-tier …e161…`（pid 28331 起的那一支）与一批 rocksdb 的 `c++` 编译；没有 `qemu-system`、`vm-bench.sh`、`e152-*`、`fio`。没有等锁。
- 收口表第 43 行原文（`.claude/kb/milestone/02-second-txn.md` 第 382 行）整行照录：

| 43 | 回退之后把 F 抬进回退留下的空档（F 落在被抛弃实例那几条根的 txg 上），checker 在合法状态上判 I-3.1（已分配统计对得上） 红：回退目标根的 txg 在 F 之下、出了 checker 取并集的候选集，它独占的槽还在记账的已分配里。最短复现 11 步：可写挂载、覆盖写两次、回退到 (2, 7) 两次（实例 3 的三条根被抛弃）、覆盖写、可写挂载、覆盖写三次、抬 F 到 8，盘 0 记账的已分配 1441792、遍历全部有效根 1343488，差 6 × 16384；同一段只换 F，7、11、12 不红，8、9、10 红，红的正好是被抛弃实例的三个 txg。增补 3 第 1 件的历史生成器第一次跑撞出（快档种子 80；大档 3000 段里 30 段） | 实现与 checker 读法对不上，第 ② 行那一族（checker 候选集的下界用哪个 F、被抛弃的根算不算），增补 3 撞出 | 等 | 登记成生成器的已知红第 1 条（撞到照记、那段历史到此为止），用例 `raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form` 钉着复现；修法与第 ② 行一起判：抬 F 不许落进空档、checker 候选集改读法、还是抬 F 时记账放掉这些槽，要三方。第 1 条的匹配 2026-09-18 收窄成「新 F 那个 txg 上的根全属于被抛弃的实例」，但不比多算的量：代码三方第二轮攻方腿在抬 F 一个都不回收的变异下看到它在空档里接走 11 段、多算 7–37 个槽而不是这一行登记的 6 个——空档里别的机理会被它遮住（`research/prompts/m2-supp3-item1-code-r2-main-verification.md`第一节「收窄之后的第 1 条」那一格的旁注，没改） | 本行；第 ② 行；`m2-supp3-item1-implementer-report.md`第四节 |

## 一、问题 1：随机历史今天走不走得到已知红

### 1.1 随机历史原样跑（副本未改时）

命令（副本根目录下）：

```
nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 16 cargo test --offline --release -p singlefs-harness --test random_histories
nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 16 cargo test --offline --release -p singlefs-harness --test random_histories -- --ignored --nocapture random_histories_fast_tier reuse_heavy_random_histories rollback_heavy_random_histories allocation_record_sampling unit_area_wall_sampling_on_small_devices_passes
```

不带 `#[ignore]` 的那几条：`test result: ok. 9 passed; 0 failed; 17 ignored; 0 measured; 0 filtered out; finished in 1.05s`，其中 `the_history_that_raised_the_floor_into_a_rollback_gap_completes_under_the_forward_rollback ... ok`（原第 43 行那 11 步在向前回退下跑完、不红）。
不带 `#[ignore]` 的几条里没有一条跑随机种子区间；种子区间在五条 harness 耗时用例里（`#[ignore]`，harness 档，照文件头「随时跑」跑了），原样输出的汇总行：

| 取样段 | 原样输出（种子区间行 + 历史行 + 已知红行） |
|---|---|
| 快档 | `种子 [7463871032432355113, 7463871032432355209)，每段 30 步，比重：各类操作都抽（快档、大档）`；`历史 96 段：跑完 96、以已知红收尾 {}、新发现 0；根环转过一圈的 61 段；最高 txg 44；一版里最多 746 条分配记录`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` |
| 偏向抬 F 之后复用 | `种子 [7463871032432355113, 7463871032432355161)，每段 30 步`；`历史 48 段：跑完 48、以已知红收尾 {}、新发现 0`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` |
| 偏向抬 F 之后回退 | `种子 [7463871032432355113, 7463871032432355161)，每段 30 步`；`历史 48 段：跑完 48、以已知红收尾 {}、新发现 0`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` |
| 越过原分配记录墙 | `种子 [7463871032432355113, 7463871032432355145)，每段 150 步`；`历史 32 段：跑完 32、以已知红收尾 {}、新发现 0`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` |
| 小盘逼近单元区墙 | `种子 [7463871032432355113, 7463871032432355145)，每段 150 步`；`历史 32 段：跑完 32、以已知红收尾 {}、新发现 0`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` |

五段合计 256 段，以已知红收尾 0 段，新发现 0。`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 75.06s`。

### 1.2 为什么走不到：生成器里造不出被抛弃的根

生成器能造被抛弃根的只有「崩溃恢复抛弃根」一步（`crates/singlefs-harness/src/history.rs` 第 370 行 `CrashRecoveryAbandoningTheNewestRoot`），管理员回退是向前发布、不抛弃根。那一步藏的是系统配置见证过的最新根，C554 乙在取号之前拒可写（同文件第 3383 行注释「这一步因此造不出被抛弃的根」）。快档的计数原样：

```
  操作 CrashRecoveryAbandoningTheNewestRoot：Ok 0、Err 53、前提不满足没调 15
  Err 成员 MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)：53 次
  操作 RaiseRollbackFloor：Ok 131、Err 210、前提不满足没调 90
  Err 成员 MountError::RollbackFloorAboveCeiling：210 次
```

另外四段的比重表里没有这一步（四段计数都是 `Ok 0、Err 0、前提不满足没调 0`）。已知红的匹配函数第一条就要「抬 F 之前的镜像上 F 那个 txg 上的根全属于被抛弃的实例」（`history.rs` 第 1616 行 `raised_floor_lands_only_on_abandoned_roots == Some(true)`），盘上一条被抛弃的根都没有，这一条恒不成立。

### 1.3 不经管理员回退、只靠崩溃恢复：写得出确定的复现

副本里新加 harness 档用例文件 `crates/singlefs-harness/tests/row43_probe.rs`（只在副本里），直接调入口、不经生成器。共同前缀：建池（第一个文件 A，txg 3）→ 同一进程覆盖写 B（txg 4）、C（txg 5）→ 崩溃恢复抛弃 C，两种造法：

- 甲（双故障形，即 C583 那一形）：`tests/common/mod.rs` 第 433 行起的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元在挂载期间读回全 0，另把每盘见证 C 的系统配置槽清零。
- 乙（单次崩溃加一时读不出，副本里新写的造法）：C 那次发布的系统配置轮换没落盘就崩（把两盘两槽系统配置退回 C 之前的字节），下一次可写挂载时 C 的根槽与数据单元读回全 0（重读一次仍读不出），挂载交回后原样写回。系统配置一槽都不坏。

乙是不是合法崩溃状态，拿 C 那次发布的录制流核（`ROW43_STREAM=1`，原样节选）：

```
ROW43-STREAM C 那次发布 #141: device=0 kind=write offset=16793600 length=4096 hash=f9fc02fc8a2ff6ac
ROW43-STREAM C 那次发布 #142: device=1 kind=write offset=16793600 length=4096 hash=f9fc02fc8a2ff6ac
ROW43-STREAM C 那次发布 #143: device=0 kind=barrier offset=0 length=0 hash=0000000000000000
ROW43-STREAM C 那次发布 #144: device=1 kind=barrier offset=0 length=0 hash=0000000000000000
ROW43-STREAM C 那次发布 #145: device=0 kind=write_fua offset=7344128 length=512 hash=83117cf4899dbb78
ROW43-STREAM C 那次发布 #146: device=0 kind=write offset=4096 length=4096 hash=f60faea1704641c0 <- 系统配置槽
ROW43-STREAM C 那次发布 #147: device=1 kind=write offset=4096 length=4096 hash=578db02ee14fcb08 <- 系统配置槽
ROW43-STREAM C 那次发布 #148: device=0 kind=barrier offset=0 length=0 hash=0000000000000000
ROW43-STREAM C 那次发布 #149: device=1 kind=barrier offset=0 length=0 hash=0000000000000000
ROW43-STREAM C 的根槽：区域 2 偏移 7344128
```

C 那次发布只有 #146、#147 两次写落在系统配置槽上，都在根槽 FUA（#145）之后；乙的盘面就是「录制流前缀到 #145 为止」这一个崩溃状态。两种造法的恢复结局相同：`ROW43 CrashBeforeRotation: C=(inst 1, txg 5); recovery lands on txg 4, new instance 2, row txg 6, current txg 7`（甲那一行只把变体名换成 `DoubleFault`）。

**不再挂载就接着覆盖写（扫描一，两种造法 × 恢复后覆盖写 0–4 次 × F = 3–7，共 50 格）**：
- 恢复后没覆盖写时 checker 全绿，但抬 F 的上限是 0，抬到 5 被拒（`RollbackFloorAboveCeiling { requested: CheckpointTxg(5), ceiling: CheckpointTxg(0) }`）。
- 覆盖写 1 次起，两种造法抬 F 之前都已经 I-7.4 红（恢复看不见 C，C 的单元被这次会话再发出去）；乙没坏任何系统配置槽，也红在 I-7.4。
- 覆盖写 4 次时上限到 8，抬 F 到 5 做成：`抬后红['I-3.1', 'I-7.4']`，按清单判成 `NewFinding(CheckerViolations { invariants: ["I-3.1", "I-7.4"] })`，不是已知红（第 0 条要「只有 I-3.1 红」）。F = 4、6、7 抬后只有原来那条 I-7.4。

**恢复之后先再可写挂载一次、再覆盖写（扫描二，两种造法 × 覆盖写 0–6 次 × F = 3–9，共 98 格）**：再挂载时 C 读得出，按实例 2 的表判被抛弃，影子账把它的单元隔离，后面的覆盖写不再复用它们。98 格里 50 格抬 F 被上限拒、盘面不变且全绿；48 格抬 F 做成，其中 42 格跑完全绿，**6 格以已知红第 0 条收尾**：两种造法各 3 格，都是覆盖写 4、5、6 次、F = 5（C 那个 txg，空档）。同一段只换 F：3、4、6、7、8、9 全绿，只有 5 红。

最小复现（乙：单次崩溃加一时读不出，不坏系统配置），用例 `row43_minimal_crash_before_rotation_remount_four_overwrites_raise_into_the_gap` 断言结局是 `HistoryEnding::KnownRed { form: 0, .. }`：

```
cd /tmp/claude-1000/closeout-recheck-2026-09-28/row43/repo
nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 16 cargo test --offline --release -p singlefs-harness --test row43_probe -- --nocapture --exact row43_minimal_crash_before_rotation_remount_four_overwrites_raise_into_the_gap
```

原样输出（`ROW43_DUMP=1` 那一遍，副本 checker 另打印明细）：

```
ROW43 CrashBeforeRotation: C=(inst 1, txg 5); recovery lands on txg 4, new instance 2, row txg 6, current txg 7
ROW43 remount: instance 3, row txg 8, current txg 10
ROW43-RESULT variant=CrashBeforeRotation remount=true overwrites_after=4 current_txg_before_raise=14 floor=5 abandoned_present_before=Some(true) lands_only_on_abandoned=Some(true)
  violations_before=[]
  raise=Ok(publishes 2, reclaimed 13, ceiling 11, abandoned_roots_unreadable 0)
  violations_after=[("I-3.1", "盘 0：记账的已分配 Some(2326528)，遍历全部有效根得到 2162688（其中隔离豁免 0）；机理：根环槽数 24、最新根 txg 16、环里自证过的根槽 17 个、最老的自证过的根 txg 0、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 1 个、回退下界 F 5、低于 F 的根槽 5 个")]
  i31_not_applicable_after=None
  ending=KnownRed(form 0)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.96s
```

步骤 13 步：A（mkfs 进程 txg 3）→ 覆盖写 B（4）→ 覆盖写 C（5）→ 崩在 C 的轮换之前 → 可写挂载，C 一时读不出（实例 2，写行 6、暖机 7）→ 关掉、再可写挂载（实例 3，写行 8、暖机 9、10）→ 覆盖写四次（11–14）→ 抬 F 到 5（空发布 15、16）。差 2326528 − 2162688 = 163840 = 10 × 16384，两盘相同。

## 二、问题 2：机理

### 2.1 差的是哪几个槽、被谁引用

副本 checker 里加了只在 `ROW43_DUMP` 下打印的明细（`walk.rs` 那一份：每条根单独走一遍记下引用，再列出最新根分配记录树里没被候选集遍历覆盖的记录）。最小复现抬 F 之后那一次（原样，逐盘各一组，这里只贴盘 0，盘 1 逐项相同）：

```
ROW43-DUMP F_eff=5 newest=(inst 3, txg 16, F 5)
ROW43-DUMP root region=1 inst=1 txg=4 F=0 abandoned=false below_floor=true
ROW43-DUMP root region=2 inst=1 txg=5 F=0 abandoned=true below_floor=false
ROW43-DUMP root region=0 inst=2 txg=6 F=0 abandoned=false below_floor=false
ROW43-DUMP unwalked record dev=0 slot=50176 span=2 gen=6 released=true referenced_by_ring_roots=[(0, 0), (1, 3), (1, 1), (1, 4), (1, 2), (1, 5)]
ROW43-DUMP unwalked record dev=0 slot=50257 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50258 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50259 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50260 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50261 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50262 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50263 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP unwalked record dev=0 slot=50264 span=1 gen=6 released=true referenced_by_ring_roots=[(1, 4)]
ROW43-DUMP device=0 allocated=Some(2326528) walked=2162688 quarantined_exempted=0 defer=Some(2031616)
```

（明细里另有 15 槽释放代 3、4 的记录也没被遍历覆盖；它们释放代 ≤ 5，抬 F 时已回收，不在已分配里，与差无关。上面只摘了与差有关的 9 条记录与三条根；全文在副本日志 `minimal-dump.log`、`minimal-dump2.log`。）

释放代 6 的记录合计 2 + 8 = 10 槽，正好是差 163840 = 10 × 16384。

- **是哪几个槽**：每盘 50176–50177（跨 2）与 50257–50264（8 个 1 槽单元），释放代都是 6。
- **被谁引用**：50257–50264 只被 B（实例 1、txg 4，恢复落到的那条根）引用；50176 被 txg 0–5 的根都引用，其中 txg 5 是被抛弃的 C。候选集里的根（txg ≥ 5、没被抛弃的：6–16）一条都不引用它们。实例 2 第一次发布（写行，txg 6，在 B 上面接）把它们换下、记成释放代 6。
- **为什么记账算已分配**：已释放、还在 defer 里的仍算已分配（`crates/singlefs-core/src/transaction.rs` 第 6298 行注释「第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。」）。抬 F 只回收释放代 ≤ 回收下界的（`crates/singlefs-core/src/mount.rs` 第 2367–2368 行 `allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest_valid_root), …)`；`allocator.rs` 第 655 行注释写的谓词），释放代 6 > 5，不回收。
- **为什么 checker 遍历不到**：候选集的判法在 `crates/singlefs-checker/src/walk.rs` 第 5884–5885 行：`let below_floor = root.checkpoint_txg < effective_rollback_floor;`、`let walked = *index == newest_index || (!abandoned && !below_floor);`。B 在 F 之下，C 被抛弃，唯二引用这几槽的根都出了候选集。

这与收口表第 43 行登记的机理同形：「回退目标根」换成「崩溃恢复落到的根 B」，空档由崩溃恢复抛弃的 C 留下。

### 2.2 「被抛弃根带的 F 算不算进 F_生效」今天怎么取

- checker：`walk.rs` 第 5845 行注释「（被抛弃时间线上的根带的 F 算不算，D16 写着仍开着，这里照「有效根」的字面不算；…）」，第 5850 行 `if abandoned_by_the_newest_roots_table(root) {` 之后 `continue`，被抛弃的根不参与「各盘最新有效根所带 F 的最大值」；第 5871 行起再与系统配置槽带的 F 取大。
- 实现：`crates/singlefs-core/src/recovery.rs` 第 1412 行文档注释同一句「被抛弃时间线上的根带的 F 算不算进 F_生效，D16（发布语义） 已定项 1「「生效」取 SysPre」那一段写着仍开着；这里照「有效根」的字面，不算。」，第 1699 行 `.is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))` 之后跳过。
- 两边取法一致：都不算。

### 2.3 换一种取法差会不会消失（副本里改 checker 试，扫描二 98 格加最小复现各跑一遍）

| 副本 checker 的读法（环境变量 `ROW43_EXPERIMENT`） | 最小复现结局 | 扫描二 98 格里不是全绿的 |
|---|---|---|
| 今天的读法（`none`） | `KnownRed(form 0)` | 6 格，都是已知红第 0 条（F = 5）；另 92 格全绿 |
| 被抛弃根带的 F 也算进 F_生效（`abandoned_floor_counts`，第 5850 行那一跳不跳） | `KnownRed(form 0)` | 同样 6 格，逐格相同 |
| 候选集不滤被抛弃的根（`walk_abandoned`） | 用例断言红：抬 F 之前就 `I-3.1`、`I-3.9`、`I-4.2` 红，抬后 `I-3.1`、`I-4.2` | 99 条结局（98 格加最小复现）里 75 条不绿：抬 F 被上限拒的 50 条全部不绿（抬之前就红），抬 F 做成的 49 条里 25 条不绿（包括 F = 3、4 这类不在空档的）；最小复现盘 0 抬后「遍历 2424832 > 记账 2326528」，差的方向反过来 |
| 下界取「≤ F 的最大一条没被抛弃的根的 txg」（`walk_nearest_root_below_floor`，F 落在空档时把 B 放回候选集） | `violations_after=[]`，用例断言红（不再是已知红） | 0 格；99 条结局（98 格加最小复现）全是 `Completed` |

- 「被抛弃根带的 F 算不算」换了取法差不消失：这段历史里被抛弃的 C 带的 F 是 0（明细 `root region=2 inst=1 txg=5 F=0 abandoned=true`），系统配置里已是 5，算进去 max 不变，F_生效 仍是 5。这一行的结论只在「被抛弃根带的 F 不高于别处」的历史上成立；被抛弃根带着更高 F 的历史我没造出来，没量。
- 差只在把 B（F 之下、恢复落到的那条根）放回遍历时消失，与 2.1 的「这几槽只被 B 与 C 引用」相互印证。这几种读法只是副本里的探针，不是改法建议。

## 三、问题 3：C379（记账的全空聚簇段数与分配器能开的段两个口径）今天还在

代码上两个口径还是分开的：`crates/singlefs-core/src/allocator.rs` 第 683 行 `empty_segments` 只数 `used_per_segment == 0`；第 756 行 `lowest_empty_segment` 另要第 762 行 `&& self.isolated_per_segment[*segment] == 0` 与扣住为 0。记账行照前者写：`crates/singlefs-core/src/transaction.rs` 第 6301 行 `PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),`。

镜像怎么造（用例 `row43_c379_whole_segment_isolated`，副本里）：最小复现那条池走到「再可写挂载」为止（影子账已按实例 2 的表隔离了 C 的单元，自然发生的），然后照攻方探针的造法，用公开入口 `PoolAllocator::isolate_abandoned` 把每块盘能开的最低全空段整段（64 槽）交给影子账隔离，再覆盖写一次让记账行落盘，副本 checker 从镜像上读记账行统计 11（全空聚簇段数）。**整段隔离这一步是注入的**：自然历史里一整段只被被抛弃根独占，我没造出来（崩溃恢复一次只抛弃一条根，那一版独占的单元十来个槽、跟别的单元混在同一段里）。

原样输出（左：整段隔离；右：不隔离的对照，`ROW43_C379_NO_ISOLATION=1`）：

```
ROW43-C379 再挂载之后（影子账只隔离了 C 的单元）: 盘 0 empty_segments=3308 lowest_empty_segment=Some(50432) isolated_slots=14 allocated_slots=77
ROW43-C379 整段隔离之后: 盘 0 empty_segments=3308 lowest_empty_segment=Some(50496) isolated_slots=78 allocated_slots=77
ROW43-C379 覆盖写一次之后（记账行落盘）: 盘 0 empty_segments=3308 lowest_empty_segment=Some(50496) isolated_slots=78 allocated_slots=93
ROW43-DUMP device=0 allocated=Some(1523712) walked=1523712 quarantined_exempted=0 defer=Some(1228800) empty_cluster_segments_row=Some(3308)
```

```
ROW43-C379 再挂载之后（影子账只隔离了 C 的单元）: 盘 0 empty_segments=3308 lowest_empty_segment=Some(50432) isolated_slots=14 allocated_slots=77
ROW43-C379 覆盖写一次之后（记账行落盘）: 盘 0 empty_segments=3308 lowest_empty_segment=Some(50432) isolated_slots=14 allocated_slots=93
ROW43-DUMP device=0 allocated=Some(1523712) walked=1523712 quarantined_exempted=0 defer=Some(1228800) empty_cluster_segments_row=Some(3308)
```

（盘 1 逐项相同。）整段隔离之后：镜像上记账行「全空聚簇段数」3308（与不隔离时相同），分配器能开的最低段从 50432 跳到 50496。两个口径对同一件事给的数不一致，C379 今天还在；数与欠账行登记的攻方数（3310、50304 → 50368）不同，是另一个池面，形态相同。

推翻条件与它造出来的那一次：把副本 `empty_segments` 改成也要隔离与扣住为 0（跑完已改回，`diff` 与原件无差），同一用例整段隔离之后记账行变成 `empty_segments=3307`、`empty_cluster_segments_row=Some(3307)`，分配器能开的最低段照样是 50496。也就是说，如果今天的记账行已经把隔离算进去，整段隔离后它会少 1；实际没少，所以用例分得出两种写法。

## 一（续）：加长的种子区间与探针（放在三之后，是因为这一段最后跑完）

副本 `history.rs` 里加了两条探针（只打印，不改判定）：每次做成的抬 F 都算一次 `raised_floor_lands_only_on_abandoned_roots`（今天只在有违例时才算），每一步之后看环里有没有按最新根实例表判被抛弃的根（`row43_probe_any_ring_root_abandoned`，与匹配函数同一句判法）。探针会亮：1.3 的确定复现里它在抬 F 之前报 `abandoned_present_before=Some(true)`、`lands_only_on_abandoned=Some(true)`。

大档用例 `random_histories_large_tier_seeds_and_length_from_the_environment`，种子从快档之后接着取（`SINGLEFS_RANDOM_HISTORY_FIRST_SEED=7463871032432355209`），每段 60 步，`SINGLEFS_RANDOM_HISTORY_SHRINK=none`。原样汇总行：

| 比重 | 原样输出 | 探针 |
|---|---|---|
| broad，1000 段 | `历史 1000 段：跑完 1000、以已知红收尾 {}、新发现 0；根环转过一圈的 980 段；最高 txg 86`；`操作 CrashRecoveryAbandoningTheNewestRoot：Ok 0、Err 1396、前提不满足没调 338`；`Err 成员 MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)：1396 次`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` | 抬 F 做成 3563 次：`Some(false)` 3164、`None` 399、`Some(true)` 0；环里出现被抛弃的根 0 次 |
| rollback，500 段 | `历史 500 段：跑完 500、以已知红收尾 {}、新发现 0；根环转过一圈的 498 段；最高 txg 78`；`操作 RollBackWhileMounted：Ok 4860、Err 2434、前提不满足没调 86`；`已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []` | 抬 F 做成 3184 次：`Some(false)` 2960、`None` 224、`Some(true)` 0；环里出现被抛弃的根 0 次 |

问题 1 的结论：**随机历史今天走不到已知红第 0 条**。试过的历史：五段写死取样 256 段（30 / 150 步），加长的 broad 1000 段 × 60 步、rollback 500 段 × 60 步，合计 1756 段，以已知红收尾 0 段、新发现 0。原因是生成器造不出被抛弃的根：加长的 1500 段里（带探针的只有这两段）做成的抬 F 共 6747 次，没有一次落在空档，每步之后的镜像上被抛弃的根一次都没出现；五段写死取样加 broad 大档里，崩溃恢复那一步共 53 + 1396 = 1449 次，全被 C554 乙拒。**只靠崩溃恢复造被抛弃实例、再把 F 抬进空档，写得出确定的复现**（1.3），而且不必坏系统配置：C 的轮换没落盘就崩、C 的根槽与数据单元一时读不出、再可写挂载一次、覆盖写四次、抬 F 到 5。恢复之后不再挂载就接着覆盖写时，抬 F 之前已经 I-7.4 红，结局是新发现 `["I-3.1", "I-7.4"]`，不归已知红。

## 四、每个结局的最小复现（都在副本 `crates/singlefs-harness/tests/row43_probe.rs`）

| 结局 | 用例（`--exact`） | 期望 / 实测 |
|---|---|---|
| 已知红第 0 条（崩溃恢复造空档，单次崩溃加一时读不出） | `row43_minimal_crash_before_rotation_remount_four_overwrites_raise_into_the_gap` | 断言 `KnownRed { form: 0 }`；今天的代码 1 passed（1.3 的原样输出） |
| 同一空档、不再挂载：I-7.4 先红，新发现 | `row43_sweep_crash_before_rotation` / `row43_sweep_double_fault`（覆盖写 4、F = 5 那一格） | 打印 `NewFinding(CheckerViolations { invariants: ["I-3.1", "I-7.4"] })`，没有断言 |
| 两种造法 × 再挂载扫描 | `row43_sweep_remount_before_overwrites` | 98 格：6 格已知红（F = 5），42 格全绿，50 格被上限拒且全绿 |
| C379 两个口径 | `row43_c379_whole_segment_isolated` | 打印记账行 3308、能开的最低段 50432 → 50496，没有断言 |

复现命令一律是：副本根目录下 `nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 16 cargo test --offline --release -p singlefs-harness --test row43_probe -- --nocapture --exact <用例>`；要明细加 `ROW43_DUMP=1`（依赖副本 checker 里的打印）。机理所在的文件:行见二。

## 五、推翻条件，以及造出来的那一次

- 「已知红那一格的差是 B 独占、释放代 6 的 10 槽」：把 B 放回遍历，差应当消失。副本里用 `walk_nearest_root_below_floor` 造过：最小复现 `violations_after=[]`，断言 `KnownRed` 的用例因此红（`panicked at crates/singlefs-harness/tests/row43_probe.rs:243:5`）。所以这条用例分得出「差在」与「差不在」。
- 「最小复现不靠坏系统配置」：如果乙那一形被 C554 乙拒，恢复那一步会 `expect("变体乙：可写挂载")` 失败。实测挂载做成、落到 txg 4（1.3 的原样行）。
- 「生成器造不出被抛弃的根」：只要加长的两段里出现一次 `ROW43-PROBE abandoned_present`，或崩溃恢复那一步出现 `Ok` 大于 0，这条就被推翻。1756 段里两者都是 0，探针在 1.3 的盘面上会亮。
- 「F_生效 算不算被抛弃根的 F，这段历史上不影响差」：被抛弃的 C 带的 F 高于 5 时这句会失效。本轮没造出这种历史。
- C379：见三末段。

## 六、排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| 差来自 checker 漏走由记录施加出来的版本 | 机理段 `并进遍历的由记录施加出来的版本 0 个`，而且把 B 放回遍历差就归零 |
| 差来自隔离豁免 | `quarantined_exempted=0`，差的 10 槽都是 `released=true`，豁免只看未释放的记录 |
| 差来自 F 的取法（被抛弃根的 F） | `abandoned_floor_counts` 下 99 条结局与今天逐条相同（`diff` 无差），C 带的 F 是 0 |
| 差来自根环转圈 | 最小复现最新根 txg 16 < 根环槽数 24 |
| 乙这一形不是合法崩溃状态 | C 的录制流：系统配置写只有 #146、#147，都在根槽 FUA #145 之后 |
| 生成器里的已知红是因为种子不够多才 0 段 | 探针 `Some(true)` 0 次、被抛弃的根 0 次，这是没造出前提，跟种子多少无关 |

## 七、没做什么

- 没修，也没判该怎么改；二、2.3 表里的几种读法只是副本里的探针。
- 没跑任何重型测试（checker 档 `singlefs-checker-tier`、54 / 55 / 57 / 59 / 87 号、全量 cargo test）；崩溃注入二进制在 checker 档，所以乙那一形没有拿崩溃注入的全部截断点核，只核了 C 那次发布的录制流次序。
- 没造「被抛弃根带着高于别处的 F」的历史，所以「被抛弃根的 F 算进 F_生效」只在 F = 0 的被抛弃根上试过。
- C379 的整段隔离是用 `isolate_abandoned` 注入的；自然历史里整段只被被抛弃根独占的状态我没造出来。
- 没改生成器（崩溃恢复那一步仍藏系统配置见证过的根），也没把副本里的用例与探针带回主工作区。

## 八、留下的材料与删掉的东西

- 删了：仓副本 `/tmp/claude-1000/closeout-recheck-2026-09-28/row43/repo`（369M）与编译目录 `/tmp/claude-1000/closeout-recheck-2026-09-28/row43/target`（353M）。
- 留着（主 agent 核复现要用，没入库，因为写范围只到报告与草稿目录）：`/tmp/claude-1000/closeout-recheck-2026-09-28/row43/materials/`，共 1.4M。
  - `row43_probe.rs`：副本里的那份用例文件（sha256 `4abd67a6518a93310d8e4f6e4c34859a4aa3f2229221701bbcc230b56345728d`）。放进 `crates/singlefs-harness/tests/` 就能跑，但它要用到副本 `history.rs` 里的 `row43_probe_any_ring_root_abandoned`。
  - `copy-vs-worktree.diff`：副本相对主工作区在 `history.rs` 与 `walk.rs` 上的全部改动，都是探针、打印与 `ROW43_EXPERIMENT` 开关（sha256 `dd2cd080deb28b784a5a7d0f3f750ce59a1e80f513dcaab6cab451e6101b6689`）。`allocator.rs` 的反事实改动跑完就改回了，diff 里没有它。
  - 各次运行的原样日志：`run-nonignored.log`、`run-tiers.log`、`probe1.log`、`probe2.log`、`minimal1.log`、`minimal-dump.log`、`minimal-dump2.log`、`exp-*.log`、`c379-*.log`、`large-broad.log`、`large-rollback.log`，以及汇表用的 `tabulate.py`。报告里说的「副本日志」都在这里。
- 重新复现的做法：把主工作区照 〇 的方法拷一份，打上 `copy-vs-worktree.diff`，放进 `row43_probe.rs`，再按四的命令跑。
