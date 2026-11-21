# m4-gpu-wait-outside-the-lock-design-r2：判决（2026-11-18）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 -->

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-design-r2-snapshot/`。这是这一题的第二轮（设计推论轮），第一轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端辩方（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-output.md` | 第一轮「会丢已答成功的发布不采」「其后 30 段前台写被拒」「缺口 2 撤回」三句都够不着；活锁存在性站得住、量够不着；「只核 R」翻回挂起；Q1 成立、Q3 永远发不出推不出 |
| 云端攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md` | 两个下限把卡挂住的那一批带回锁里；出错与作废谁先没排；让路把连着作废清零时后台发不出；一拍空转到 64 次；候选乙读复用槽没打中；找到「回锁照现行那一版重排计划、不作废」一个候选 |

核查：

- 引文：任务 19757 `python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-output.md research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-background.md`，末行「✓ 核了 74 处引文、35 处裸行号，对上 74 处，没判 35 处（其中裸行号没有引文可比 35 处；逐处列在上面）」。设计轮没有冻结的树，根取主工作区。裸行号没逐处核。
- 辩方探针：任务 19745 退 0，输出 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet-run/out/`。
  - `stale-base-kinds.out`，三种前台改动（覆盖写第一个文件、顺序写第二个 inode、建 inode）各 14 段历史，k 取 1、2、3、8、24、32、48。弱判据那一臂三行：`RESULT name=stale_base_kinds kind=overwrite_unit_of_the_first_file arm=stale_weak histories=14 lost=7 stale_publish_made=0 refused_by_the_release_check=12 panicked=0 setup_failed=0`；`RESULT name=stale_base_kinds kind=sequential_write_of_the_second_inode arm=stale_weak histories=14 lost=7 stale_publish_made=0 refused_by_the_release_check=14 panicked=0 setup_failed=0`；`RESULT name=stale_base_kinds kind=new_inodes arm=stale_weak histories=14 lost=7 stale_publish_made=0 refused_by_the_release_check=14 panicked=0 setup_failed=0`。整理被拒之后把现行那一版换回去的那一臂三种都是 `lost=0`（例：`RESULT name=stale_base_kinds kind=new_inodes arm=stale_weak_restored histories=14 lost=0 stale_publish_made=0 refused_by_the_release_check=14 panicked=0 setup_failed=0`）；阳性对照三种都是 `lost=14`。
  - 今天代码上的忙窗口基线（替身卡每次调用睡一段，四个读线程紧循环，有的加一个写线程），七行 `name=busy_window_foreground_wait`，判决用到的两行整行：
    - `name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=50 readers=4 writers=0 gap_us=0 samples=104512640 wait_micros_mean=0 wait_micros_p99=1 wait_micros_max=1104695 maintenance_publishes_total=60 blocked_samples=1076 blocked_wait_micros_p50=1995 blocked_wait_micros_p99=551723 blocked_wait_micros_max=1104695 blocked_publishes_none=1003 blocked_publishes_one=28 blocked_publishes_two_to_three=34 blocked_publishes_four_or_more=11 writer_calls=0 writer_call_micros_p99=0 writer_call_micros_max=0`
    - `name=busy_window_foreground_wait route=gpu_double sleep_ms_per_gpu_call=50 readers=4 writers=1 gap_us=0 samples=521701 wait_micros_mean=114 wait_micros_p99=0 wait_micros_max=1154895 maintenance_publishes_total=113 blocked_samples=229 blocked_wait_micros_p50=181417 blocked_wait_micros_p99=1154892 blocked_wait_micros_max=1154895 blocked_publishes_none=76 blocked_publishes_one=61 blocked_publishes_two_to_three=64 blocked_publishes_four_or_more=28 writer_calls=48 writer_call_micros_p99=1169025 writer_call_micros_max=1169025`
    - 睡眠 0 与 5 毫秒两档、四个读线程：被挡的取样最坏等 130610 与 165487 微秒（`busy-double_sleep0_readers4_tight.out`、`busy-double_sleep5_readers4_tight.out`）；睡眠 5 毫秒加一个写线程：被挡的中位 49312、最坏 401011 微秒，写线程一次调用 p99 是 247099 微秒（`busy-double_sleep5_readers4_writer1.out`）。各只跑了一趟。
- 攻方模型：任务 19741 退 0，`name=self_checks checked=55 failed=0`，输出 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-opus/model-run/out/`。它是攻方照正文第三节十条写的离散事件模型，不是产品代码；到达率与卡挂住的时刻是摆的。判决用到的格：`hang_brought_back_into_the_lock.out` 里 `arm=form k=1` 各行 `read_wait_max_ms` 在 39860.1 到 40001.8 之间（下限把卡挂住的那一批接回锁里，读等满约 40 秒，与模型里今天那一臂同量级）；`yield_and_floors.out` 里 `k=1 n_yield=1 yield_resets_k=True fsync_per_second=1000.0` 那一行 `background_publishes=0`、同参数 `yield_resets_k=False` 是 59。

## 二、判决

| 格 | 判决 |
|---|---|
| 第一轮「会丢已答成功的发布不采」 | **维持，依据换成第二轮的**。辩方说 54 段取样够不着是对的（前台改动只有一种、k 只到 8）；它补扫的三种改动、k 到 48 共 42 段历史里 `stale_publish_made=0`，旧底上的整理发布一次都没落盘，全被发布路径的释放目标核拒掉。有活意图那一支、建快照与删快照的旧底没扫，这一句的射程只到「没有活意图、三种前台改动、k 不超过 48」 |
| 第一轮「其后 30 段前台写被拒」 | **撤回**。辩方说它是探针把现行那一版留在旧底上造出来的：换回去的那一臂 `lost=0`，坐实。第一轮判决 Q2 弱判据那一格里这半句作废，「不比 txg 不行」的依据改成：旧底上的整理发布必被拒、整理白算一批，并不伤前台 |
| 第一轮「缺口 2 撤回」 | **改判：一半成立**。忙窗口基线里被挡的取样拿到锁之前后台发 4 次以上的每趟有 6 到 28 个，放锁后被维护线程连着越过是有的；「一拍 64 次都抢不到」没出现。前台被挡的时长跟着 GPU 一次调用的时长涨：睡 50 毫秒时读最坏等 1.10 秒、写线程一次调用 p99 是 1.17 秒。这是今天锁内等的代价的第一份量到的数（替身卡，睡眠是摆的） |
| 活锁「要下限」 | 存在性两腿一致，采；量（作废占多少）照旧是仿真 |
| Q1 两个下限 | **打中**（两腿一致：辩方判「成立」，攻方模型量到）。连着作废 k 批后回锁里算、破例批恒在锁里算，都把一批带回会话锁里等 GPU；卡挂住时那一批正好落在锁里，读等满约 40 秒（攻方模型）。正文第三节第 7 条出局 |
| Q1 出错与作废谁先 | 攻方打中，辩方没覆盖：只有一条腿，照「只有一份样本答到的那件事」办，记线索，第三轮形态里写死「出错先于复核」 |
| Q2 五样之外 | 两腿都没打中（辩方「严判据下五样之外不成立」，攻方读码没打中）。停 |
| Q2 一拍空转到 64 次 | 两腿一致：作废的停因在运行库 `keeps_going`（`crates/singlefs-mount-runtime/src/lib.rs` 第 671 行，反向匹配）里默认接着叫，与欠账 C1008（停着门关的那一趟没有备忘、一拍连叫 64 次） 同形。打中；形态不再作废之后这一格没有对象 |
| Q2 第 8 条计数 | 攻方打中（弱），辩方没覆盖：记线索 |
| Q3 让路把连着作废清零 | 两腿一致（辩方「缺两个计数的清零规则，让路清作废数会饿死后台」，攻方模型 `background_publishes=0`）。打中；第 7 条出局之后 k 没有了，剩让路自己的下限 N |
| Q3 互等、`try_lock` 窗 | 两腿都没打中。停 |
| Q4 读盘候选乙 | 读到复用槽两腿都没打中；块设备的线程安全两腿都推不出（`read_at(&self)`、`write_at(&mut self)`，真盘实现没读）。候选乙挂起 |
| Q4 候选甲、丙 | 甲「成立但不是圈数的函数」、丙「至少 1/5 批仍在锁里」：辩方读码推的，攻方「部分」。挂起 |
| Q6 | 攻方四条（维护线程自己的 `publish_when_due` 与重发冻结在锁外那一段里不跑、同步一拍进不了新分支、第二个后台调用者撞 `end_the_batch` 断言、回锁里算的批在观测里没有成员）只有一条腿：记线索，进第三轮形态 |
| 辩方替今天的做法辩 | 「零新状态」成立；「卡挂住一次事件只挂一次、之后任务停下」是推的。今天的做法输在哪有了数：忙窗口基线那两行 |
| 「回锁照现行那一版重排计划、不作废」 | 攻方一·5 与辩方翻回的「只核 R」是同一个方向：回锁之后不比 txg，改比 R 里的记录，在现行那一版上发。整份零轮，它拿掉第 7 条两个下限。进第三轮 |

第二轮照 `.claude/rules/three-way-inference.md`「开不开下一轮（设计推论轮）」照开第三轮（最终轮）。

## 三、改法

1. 回锁照现行那一版重排计划、不作废（有活意图那一支剔掉记录变了的单元，签发那一支在锁里重判空洞与资格）：零轮；最脆弱的前提「两版里 R 的记录逐条相同则单元字节与分类都没变」两腿都只读了一侧。
2. 出错先于复核：零轮。
3. 让路的下限 N 不与别的计数互相清零：零轮。
4. 维护线程在锁外那一段期间照跑 `publish_when_due` 与重发冻结：零轮。
5. 第一轮改法 1（比 txg 作废）量过，但它带来的下限出局；第一轮改法 3（回锁之前放 GPU 实现那把锁）量过，留。

标零轮的，写进产品代码之前先在探针里做 A/B。

## 交用户的

第三轮判决之后一起交。到这一轮为止能说的：

- 今天锁内等的代价量到了：GPU 一次调用 50 毫秒时，读最坏等 1.10 秒，写一次调用 p99 是 1.17 秒（替身卡，一趟）。
- 「比 txg、不等就作废、再加下限」这条路出局：下限把卡挂住的那一批带回锁里，缺口 3 原样回来。
- 剩下一条没被攻过的路：回锁照现行那一版重排计划。

## 这一轮被判过的文件

- crates/singlefs-mount-runtime/src/lib.rs：判了一拍的 `keeps_going` 对作废停因接着叫、维护线程自己的发布在锁外那一段里跑不跑；忙窗口基线量的是它今天的行为。
- crates/singlefs-core/src/mounted_session.rs：判了回锁复核的五样全不全、出错与复核的先后、破例批落在哪。
- crates/singlefs-core/src/background_compute.rs：判了在途的批被拒的位置与 `end_the_batch` 断言的先后。
- crates/singlefs-core/src/compaction_plan.rs：判了作废的停因进不进备忘与趟状态。
- crates/singlefs-core/src/transaction.rs：判了旧底上的整理发布被释放目标核拒掉（探针量到 42 段全拒或没起）。
- crates/singlefs-core/src/block_device.rs：判了读盘候选乙要的读写并发，两腿都推不出它经不经得起。
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了卡挂住时一段等多久进了下限那一格的模型。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-11-18 不受影响：形态挂起，第三轮之后才写回 |
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-18 不受影响：第 4、5、6 条的改句没写回 |
