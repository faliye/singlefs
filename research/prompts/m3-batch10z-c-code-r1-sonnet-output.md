# 第 10z 批第三块（显存账）代码轮第一轮：正推腿报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 S3 S4 S5 S6 S9 S10 P2 N10 M01 M02 M03 M04 M05 M06 M07 M08 M09 M10 M11 M12 M13 M14 M15 M16 M17 M18 M19 M20 M21 M22 M23 M24 M25 M26 M27 M28 M29 M30 M31 M32 M33 M34 M35 -->

立场：正推。分到的格：正文第三节 K1（1–5）、K2（1、2）、K3。背景材料：`research/prompts/_m3-batch10z-c-code-r1-body.md`、小节清单、附录、改动全文。
被判对象：冻结副本 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/`。下面引的四份源文件（`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs`）与主工作区逐字节相同，行号两边一样。
模型与输出：`research/prompts/m3-batch10z-c-code-r1-sonnet-model/`（`README.md` 说怎么放、怎么跑）。草稿目录 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/`（树副本与编译目录交回前已删，见末尾「草稿目录」）。

## 0 核对与环境

- 冻结副本核对：`sha256sum -c research/prompts/m3-batch10z-c-code-r1-snapshot/crates-sha256.txt` 在副本根下，末五行全是 OK（`crates/singlefs-replay-prefix/src/lib.rs: OK` 等）。
- 副本基线：`cargo test --offline -p singlefs-harness --lib`（`run-with-memory-cap.sh 4G`、`capped.sh 8`）末行 `test result: ok. 154 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 28.55s`，与实现员报告同数（`logs/lib-full-baseline.log`）。
- 开跑前 `ps` 看到别的会话的 `cargo test -p singlefs-checker-tier …`、`prove-10z-c` 一族的 rustc/ld，以及 `cargo test -p singlefs-harness --test k2_arms_probe`；没有 qemu、vm-bench、e152、fio。我都加 `nice -n 19` 照跑；有一条撞上内存包装排队（头一条 `12G` 的排了约 3 分钟被我自己的 `timeout 230` 掐掉，之后改 `4G` 即进，等锁时间不计入任何数）。另有一条自造用例写错了锁（`show` 里持着物理卡的锁又去读账，同线程再取同一把锁），跑挂住，我用 `proc.py stop` 停了那个测试进程，改了再跑；挂住的那次没有产出。
- 没跑 checker 档的任何测试，没起 GPU 设备；`nvidia-smi` 只读（`--query-gpu`）看到两张卡 `2 MiB / 32607 MiB` 与 `2 MiB / 16311 MiB`，没有占卡。
- 所有「会红」的模型都在副本里改、跑、还原，每次还原后 `cmp mut/gpu_memory_ledger.rs.orig tree/crates/singlefs-harness/src/gpu_memory_ledger.rs` 报相同。

## K1-1　G2 甲：账给的量任何一段历史上都不大于此刻真能再借的？

判：**做到；另有一形漏：设备丢了、账还没被问到的那一段多给（不是这一批引进来的，窗口内最多多给驱动那一份）**。

1. 「见到的那一份」连同三处用法拿掉了。命令与输出（仓内全部 `.rs`）：
```
$ grep -rn 'seen_on_the_card\|leases_seen_on_the_card\|seen_at_least\|lease_card_waiting_for_room' --include=*.rs crates | wc -l
0
```
2. 往下压的实现：读数里这个进程占着的是压的上界，别的猜法没有。压的那一行「returned_possibly_still_on_the_card_mebibytes.min(mebibytes)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:222`）；算「实际占着的里属于许出去的」时先扣这一份「.saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:203`）；读数进账只有一处「books_of_the_card.take_in_a_reading(*reading);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:969`）。没列、或另有不走账的设备时不压、且占着的算 0（203–206 行与 224–226 行），只会少给。
3. 不变量（推的）：销账时记进这一份的是整笔借的量（`return_a_lease` 的 `saturating_add(mebibytes)`，878–888 行），不小于那笔借据在卡上真占的（借的量 ≥ 建上的，是 10x/10y 批已定的前提，我没重验）；压只压到读数里这个进程此刻占着的，而真残余是占着的一部分，所以「这一份 ≥ 真残余」在任何读数之后都成立；room = cap(F + min(占着 − 这一份, 许出去的)) − 借着的 ≤ F + 实占的驱动与借据建上的 − D − 借着的 = 真能再借的。什么现象会推翻它：某笔借据销账之后卡上还留着比它借的更多（分配器按块拿超过借的量），或销账之后另一条线程还在往那笔借据的缓冲里建。
4. 复跑上一轮两腿的模型（原样拷进副本 `crates/singlefs-harness/tests/`，`cargo test --offline --no-fail-fast -p singlefs-harness --test sonnet_probe_common --test sonnet_probe_new_only --test ledger_attack_10z -- --nocapture --test-threads 1`，输出 `logs/probes-old-legs.log`，sha256 b49fe05b…837ab30e）：
   - 攻方 `mode=` 四档：`overgive_at_reading_max=0 overgive_bookings=0` 四档全是 0；少给照实：`false_refusals=146`、`4969`、`5222`、`5067`（与实现员报告同数）。
   - 正推 FUZZ 12 组：`grep -c 'over_count: 0,'` 与 `grep -c 'over_count:'` 都是 12。S10：`#1 ledger 18550 truth 19550 … #10 ledger 9550 truth 19550`（甲的数）。
   - 攻方丙形/乙附：`restart-sweep class=restart-allowed-and-fits cells=1300`、`restart-refused-and-does-not-fit cells=60`，没有「放行而放不下」那一类。
   - `ledger_attack_10z` 有 7 条断言红是预期的（模型断言照 10z 的数写），只读 ATTACK 行。
5. 我自造的一份（第三条互不共用代码的路径：物理卡上加「设备丢了、旧设备那一份留在卡上等收回、重起、重起被拒」，`probes/lost_restart_model.rs`）：5 组参数 × 4000 种子 × 120 步，丢设备概率 20–40%：`LOSTFUZZ xd=2560 collect%=50 lose%=20: samples=135459 over_noticed=0 (max 0) …`，五组 `over_noticed` 全是 0（`logs/lostfuzz4.log`）。
   - 漏的那一形（`over_before_first_prepare_after_loss`，五组 559 / 305 / 1255 / 1265 / 1367 次，最大 2560 / 600 / 2560 / 1500 / 2560）：设备在卡上物理丢了、账还没被 `prepare_the_driver_share_before_starting_the_device` 问过，账还把旧设备当活的驱动算。定向复现（`logs/directed2.log`，`directed_loss_window_and_k_restarts`）：
```
DIRECTED healthy: lease 10000 built, device started: ledger_room=19550 truth=19550
DIRECTED device lost, ledger not asked yet (window): ledger_room=19550 truth=16990
DIRECTED prepare after loss -> Ok(())
DIRECTED after the first prepare (new device not started): ledger_room=16990 truth=16990
```
   - 为什么不算 OOM 风险（读码）：借到之后要起设备必走 `started_card`，丢了的设备在那里先过一遍重起判（「.prepare_the_driver_share_before_starting_the_device(card_position)」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1270`），用的是 `|started: &GpuCard| started.device_condition().is_lost(),`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1263`）），那一判按丢了之后的口径、连这一笔一起算，所以窗口里多给的数不会变成建上的缓冲；代价见 K2-2 第 1 条。
   - 什么现象会推翻「窗口内不会真建上」：存在一条不经 `started_card` 就用卡上丢了的设备建缓冲的路径（我读到的取卡入口只有 `lease_card` / `lease_the_first_card_with_room_now` / `book_the_scratch_of_the_cards_with_room_now`，都经 `card_lent_for_the_booking`）。

## K1-2　S6：被拒之后再来的每一次都判了吗？新设备起好之后账回到正常没有？

判：**每一次都判，做到；新设备起好之后账没有回到正常（甲之下的代价，且每丢一次、重起一次多扣一份驱动，直到封顶），属「做到但有一形漏」**。

1. 每一次都判（读码）：卡池留着的设备丢了就重起，起不来不留着：「if !is_lost(device) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1430`）、「let device = start()?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1434`），`?` 在写回之前返回，所以被拒之后那一格还是丢了的设备，下一次来照样进 `start`。`start` 里先过 `prepare`：第一次进「DriverShareOnTheCard::DeviceLostNotStartedAgain => {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1280`）之前的 `DeviceStarted` 臂把驱动那一份记回「自己此刻不占卡」（「self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:862`）），之后每次进 1280 行那一臂都再过一次「self.driver_share_of_a_new_device_fits_now(&mut books, card_position)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1281`）。起好了才记回：「self.books_locked()[card_position].driver_share = DriverShareOnTheCard::DeviceStarted;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1322`）。
2. 实测（上一轮正推 S6，在冻结副本上重跑，`logs/probes-old-legs.log`）：`S6: room before second lease 19550; restart attempts: first Err(NoCardHasRoomNow { need_mebibytes: 2560, card_rooms_mebibytes: [0] }); second Err(…); third Err(…)`。
3. 我另造的三个会红用例（`probes/killers.rs`，真代码上绿）把 S6 的两个侧面钉住：被拒几次、读数没变，能借的不变（`k_a_refused_restart_asked_again_does_not_take_another_driver_share_off_the_room`）；被拒之后又借了一笔、卡上再被占，再来起设备仍拒（`k_a_lease_booked_after_a_refused_restart_keeps_the_restart_judged`，借一笔的那一步在 `book_a_lease` 里「| DriverShareOnTheCard::DeviceLostNotStartedAgain => {}」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:903`）不动状态，是这条用例钉住的东西）。实现员自己的 S6 用例钉不住这两侧，见 K3 的 M05、M13。
4. 新设备起好之后账回不到正常（定向，`logs/directed2.log`，一笔 10000 建满、驱动实占 2560，丢一次、旧设备收回、新设备起好，重复）：
```
DIRECTED restarted, dead share freed, new device started (k=1): ledger_room=16990 truth=19550
DIRECTED restarted again (k=2): ledger_room=14430 truth=19550
DIRECTED restarted again (k=3): ledger_room=11870 truth=19550
DIRECTED restarted again (k=4): ledger_room=9310 truth=19550
DIRECTED restarted again (k=5): ledger_room=6990 truth=19550
```
   原因（读码）：丢了重起时旧设备那一份当销了账的记进去：「self.returned_possibly_still_on_the_card_mebibytes = self」之后接 `saturating_add(mebibytes_left_for_the_driver)`（`take_back_the_driver_share_of_a_lost_device`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:861`–866），新设备起好、旧的收回之后，这个进程占着的永远不低于新设备的驱动那一份加借据建上的，压（222 行）压不掉它，于是每次重起多扣 D，直到封顶在占着的（k=5 时 12560）。这是主 agent 在背景里列的「重起之后少给 D、压不掉」（正推 S4、S9），在甲之下照旧；我只补它累加的数。多给方向没有（K1-1 第 5 点）。
   什么现象会推翻：新设备起好之后账读到的占着的小于记着的那一份（那一份会被压低），或有路径在新设备起好时把那一份清掉（`record_the_device_started_on_the_card` 只改驱动那一份的状态，1320–1323 行，不清）。

## K1-3　对拍用例：Some 时本次未跑、None 时钉总状态数，各有没有一形照绿？

判：**读码做到；GPU 与 checker 档我跑不了（复核不了），下面是读码与纯函数能说的**。

1. Some 臂：「if let Some(reason) = &tally.stopped_because_the_gpu_memory_was_short {」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:282`）下面「println!("{}", not_run_line(TEST_NAME, reason));」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:283`）后 `return`，不走后面的断言。这一行是 `NOT_RUN_NO_GPU_ADAPTER test=… reason=显存不够：…`（`GPU_NOT_RUN_MARKER`、`GPU_MEMORY_SHORT_REASON_PREFIX` 在 `gpu_memory_ledger.rs` 里定义），用例退出码仍是 0——「本次未跑」靠标记行，不靠退出码，与同文件别的 GPU 用例同一个约定，不是这一批新引进来的。
2. 这个字段什么时候是 Some：单元校验和对拍时一块的一段在范围里接到 Out of Memory 或设备丢了且说得成，才交 `gpu_memory_was_short: Some`（`judge_one_piece_catching_out_of_memory_on_the_card`），收齐后「tally.stopped_because_the_gpu_memory_was_short = Some(reason);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1536`）并 `break`。所以 Some 时之前判完的块在库里、这一块与之后的没判；用例不再核这些，只报本次未跑，没有「部分判完照绿」的形。
3. None 臂：「"没报显存不够：这条流的每个状态都判了"」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:294`）前面的断言是 `tally.states_checked` 等于 `layer0_plan_state_count(…)`。`states_checked` 只在一块写进库之后加：「tally.states_checked += local.end - local.start;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1570`），所以 None 且没叫停时少判一块就红。两个崩溃点都 `ignored: false`，`plan_crash_points` 里的 `ignored_points` 全假，`expansion` 与 `layer0_plan_state_count` 的是同一份（读码），等式的右边就是计划里的总状态数。
4. 一形照绿（读码推的，没跑）：库里已有这一版判定的块（`pending_blocks` 滤掉）时 `states_checked` 少于总数——用例每次建空库（`VerdictStore::create_empty`），不会遇到。叫停（`SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE`）这条用例不设，也不会遇到；真有人在跑时建这个文件，None 臂会因少判而红，不会照绿。
5. 什么现象会推翻：`reason_the_gpu_memory_was_short_in_one_of_the_pieces` 在一段没报时也交 Some，或 Some 臂之前已写库的块里有判错的（对拍用例此时不核）。

## K1-4　第 D 条：同块几段都报到再认

判：**做到；等不齐没找到形；真显存不够不判红；真 panic 被说成显存不够只在「设备丢了且这一块里有人报过」时，是原设计，等齐把窗口拉宽了**。

1. 等不齐：每一段的报到在起线程之前一次领齐：「let ending_together = WorksEndingTogetherOnADevice::new();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1498`）、「pieces.iter().map(|_| ending_together.join()).collect();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1500`）；段 panic 或没走到报到就丢掉，丢的时候补报：「if !self.has_ended {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1803`）。CPU 判的段 `drop(ending)` 即报（「drop(ending);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1235`））。等的循环「while *works_not_yet_ended > 0 {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1791`），最后一段报完 `notify_all`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1770`）。几段之间没有别的同步原语（`grep -n 'Barrier\|Condvar' crates/singlefs-checker-tier/src/crash_amplification.rs crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 只命中 `StepKind::Barrier` 一处，4720 行），一段在等时不攥别的段要的锁（等在 `pollster::block_on(out_of_memory_scope.pop())` 之后、错误范围已弹出）。所以只有「某一段本来就永远判不完」才会拖住别的段，而那种情形整块本来就收不齐。
2. 真显存不够判红：一段在范围里接到就记进设备再报到：「.end_the_work_and_explain_its_failure(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2192`）里先记后等后认（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1670`–1682），`decide_the_work_on_the_card` 第一臂把范围里接到的算显存不够，与段活成没成无关。真 panic 与别的段的范围 OOM 并发（设备没丢）时，别的段那一条的 `works_it_explains` 是它自己，认不到这一段上，这一段的 panic 照样往外抛：
```
EXPLAIN device_lost=false: piece A (real panic) explained as None; piece B (caught OOM) explained as Some("序号 0 的卡单元校验和对拍时报了 Out of Memory")
```
3. 真 panic 被说成显存不够的那一形（`probes/explain_demo.rs`，`logs/explain1.log`）：
```
EXPLAIN device_lost=true:  piece A (real panic) explained as Some("设备丢了（Unknown：提交时显存不够），丢之前报过 序号 0 的卡单元校验和对拍时报了 Out of Memory"); piece B (caught OOM) explained as Some("序号 0 的卡单元校验和对拍时报了 Out of Memory")
```
   即设备在这一块里丢了、同块另一段报过 OOM，这一段不管什么 panic 都说成「设备丢了，丢之前报过…」：「match (self.lost_reason(), reports_since.last()) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1705`）。这是 `out_of_memory_explaining_a_failure_of_the_work` 文档里写的「丢了设备谁都做不成」，不是这一批引进来的；等齐之后这一段看得到更晚才记进来的 OOM，窗口变宽。要同时满足「设备丢了」与「同块有人报过 OOM」，我没有办法在没有卡的机器上造出 wgpu 的真丢设备，只能说是读码加纯函数。
4. 变异证（harness 档，见 K3）：M15（不等其余几段）、M25（范围里接到的不记进设备）、M26（先认后等）红；M17（没报到就丢掉时不补报）一条红、其余挂住；M16（最后一段不叫醒）挂住，不红。

## K1-5　第 E 条：再起一轮有没有一形判两遍、漏判、判完不入库、或无限再起

判：**判两遍、漏判、无限再起：没找到；判完不入库：有两形（叫停与本次未跑收尾）；叫停落在第 1、2 轮之间会撞到第 3081 行的断言 panic（产品代码读码加模型，checker 档与真卡复核不了）；另有一处汇报数算两遍**。

方法：产品循环在 checker 档、要卡；我把它的外层循环和卡线程的走法逐句搬成 harness 档模型（`probes/rounds_sim4.rs`，只用 `judge_blocks_on_one_card`、`BlocksForTheCards`、`BlocksStoredInOrder`、`after_a_round_of_the_cards` 四个真函数加假卡），假卡可设「建判器报 OOM」「判了 n 块之后下一块报 OOM」「每块耗时」。这是模型，不是产品循环，保真度的限定见「没做什么」。

1. 随机 3000 格（1–4 张卡、1–12 块、每张卡 1/6 概率装不下、1/6 建判器报 OOM、1/3 判了 0–7 块后报 OOM、每块 50–450 微秒；每格断言：没有一块成功判两遍；写库的是 `0..k` 的前缀按块序；`EveryBlockStored` 时每块判一遍且全写进；有一张从不报 OOM 且装得下的卡时必须 `EveryBlockStored`；轮数不超过卡数加一）。原样：
```
ROUNDS_SIM cells=3000 results={"ERR-no-card-holds-a-state": 157, "EveryBlockStored": 2353, "NotRun": 490} dup_finished_cells=47 violations=0 notrun_cells=490 notrun_cells_with_judged_but_unstored=10 max_wasted_blocks=3
```
   `ERR-no-card-holds-a-state` 是头一轮没有卡装得下，产品里在进循环之前的 `decide_scratch_before_booking` 分流，不是这条循环的结局。`violations=0`：判两遍、漏判、无限再起、放得下却未跑，在模型里一形没有。读码也是这个结论：交回的块不重判，因为只有「判块时报了 OOM」才 `give_back`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1917`），那一块没发去写库；下一轮的队只取队里剩的「blocks_of_the_round = work.blocks.blocks_not_taken();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3105`），判完等着前面那一块的块不在队里；轮数有界：下一轮的卡取「.filter(|card| !cards_short_of_memory.contains(&card.index))」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2937`）、报了 OOM 的才进集合，再起一轮要求这一轮至少一张新卡报 OOM（「if cards_short_of_memory_this_round == 0 {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2024`））。
2. **判完不入库（一）：本次未跑收尾**。`notrun_cells_with_judged_but_unstored=10`、`max_wasted_blocks=3`：所有卡都报过 OOM、前面那一块没人判时，后面已判完的块在内存里等不到前面的块，一块都不入库，下趟重判。不是错（本次未跑，库里是 `0..k` 前缀），是白判。实现员报告里「上一轮判完、等着前面那一块的那几块在内存里等到下一轮，不重判」说的是还有下一轮的情形，没写这个收尾。
3. **判完不入库（二）+ 一处会 panic：叫停落在第 1、2 轮之间**。第 1 轮有卡报了 OOM 且别的卡先于交回收工，库里写到 `k`，`k+1…` 在 `stored_in_order` 里等着，队里只剩被交回的那块。第 2 轮开始后（借草稿区、建判器可能是几分钟）有人建了叫停文件：`must_stop` 是「self.stop.load(std::sync::atomic::Ordering::SeqCst) || (self.stop_is_requested)()」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2118`），卡线程的「while !must_stop() {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1907`）一块都不领，第 2 轮没有 OOM、没有拒判、写库没错，于是走进这一段：「if stored_in_order.storing_error().is_none()」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3076`）里的断言，等着的块数不为 0，断言的消息是「"领走的块每一块都写进了库"」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3084`），判的量是「stored_in_order.blocks_waiting(),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3082`）。断言的前提（等着的块一定是这一轮自己领走的）在有上一轮遗留的等着的块之后不成立。模型原样（`probes/rounds_sim4.rs` 的 `stop_requested_between_round_one_and_two_with_blocks_waiting_behind_a_given_back_block`；2 张卡 10 块，第 0 张慢、第一块报 OOM，第 1 张快、别的块都判完先收工）：
```
STOPSIM stop_from_round=None result=EveryBlockStored stored=10 rounds=2 judged_ok=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1] assertion_failures=[]
STOPSIM stop_from_round=Some(2) result=StoppedOnRequest stored=0 rounds=2 judged_ok=[0, 1, 1, 1, 1, 1, 1, 1, 1, 1] assertion_failures=["round 2: blocks_waiting()=9 != 0（产品代码 3081 行断言会 panic）"]
```
   叫停时 9 块判完不入库；产品代码里这一刻不是返回 `StoppedOnRequest` 而是断言 panic（模型把断言的条件改成记一条，没有真 panic）。什么现象会推翻：第 2 轮即使叫停也先领交回的那一块（卡线程的 `must_stop` 在第 2 轮之前被清）。产品里叫停靠文件，没有被清的路径（读码）。
   窗口：第 1 轮的 `after_a_round_of_the_cards` 在叫停已成立时交 `NotRunTheGpuMemoryWasShort` 返回（不再起），所以只有叫停文件在「第 1 轮收尾判过之后、第 2 轮第一块被领走之前」出现才撞到；这一段含借草稿区与（冷编）建判器，可以是分钟级。
4. **汇报数算两遍**：每一轮收工的卡都 `extend` 进去「outcome.cards.extend(received.finished_cards);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3094`），最后「outcome.cards_used = outcome.cards.len();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3154`）。头一轮判完收工、第二轮又被用上的那张卡进两次，`cards_used` 比真用的卡数多、`CRASH_AMPLIFICATION_GPU_TOTAL cards_used=` 这一行报错的数。模型里 `dup_finished_cells=47`（3000 格里 47 格某张卡 `Finished` 出现两次）。只影响汇总行，不影响判定：crates 里 `cards_used` 的使用处是打印与取最大（`grep -rn cards_used crates` 命中的没有断言）；`scripts` 里我只看了三份崩溃放量脚本，命中的是显卡配置变量名，没看到解析输出行里 `cards=` 的，别的脚本没查。
5. 什么现象会推翻第 1 点的「没有判两遍、漏判、无限再起」：产品循环与模型有一处不一样而我没读出来。已知的差异：真循环在借草稿区时按账的此刻能借压每张卡的状态数（模型用固定的 `holds_states`），这会改变哪些卡「装了状态」，但不改变上面几条依赖的队、写库序、轮数的约束。

## K2-1　有没有一形让账给的量大于此刻真能用的（新代码引进来的）

判：**新代码没有引进多给；原有的丢失窗口（K1-1 第 5 点）还在**。

1. 这一批改动对账的数的方向：拿掉「见到的那一份」只会让压得更少（多扣），不会多给；`DeviceLostNotStartedAgain` 把被拒之后的每次再来判成照此刻再判（只会比放行更严）。K1-1 第 4、5 点三条互不共用代码的路径（两腿真账模型、我的丢/重起模型、定向）都是 0 次多给（丢失窗口外）。
2. 其余新代码不碰账的数：第 D、E 条与对拍用例动的是错误归属、块队、轮次与测试，不改 `rooms_of_the_cards`。第 E 条再起一轮会重新借草稿区：上一轮的借据在轮末丢掉（`scratch` 随循环体丢，丢时先让设备收一次再销账），销账的整笔进「卡上可能还没还的」，下一次读数只压到占着的为止（K1-1 第 3 点），所以只会少给不会多给；在同一张卡上相继借的两笔，账是按 `rooms_of_the_cards` 在锁里现读的，多给的形没有（模型里 `book_on_the_card` 连着借的路径被 5 组 fuzz 覆盖）。
3. 窗口里的多给：见 K1-1 第 5 点，`ledger_room=19550 truth=16990`，多给 2560。窗口外 0。

## K2-2　除甲认的少给之外，有没有一形卡放得下却报本次未跑、判红、或整趟弃卡

判：**有两形（一形读码推的、一形在模型里造出来；都比甲认的窄），另有一形是设计**。

1. **丢失窗口里多给的数，被重起判纠正时整张卡这一趟不用**（读码推的，要真丢设备才触发，我造不出）。产品路径第 ① 段借草稿区在窗口里按多给的量压状态数并记账，之后每张卡交 `card_lent_for_the_booking`，起设备时重起判按丢了之后的口径放不下就拒；拒了的卡「first_card_short_of_memory.get_or_insert(reason);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1429`）这一趟不用，没有按纠正后的量再借一次；每张都被拒就「return Err(CardPoolLeaseRefusal::OutOfGpuMemoryStartingTheDevice {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1436`），产品路径照这个报本次未跑。单卡机器上，卡放得下一个更小的草稿区，却整趟未跑。多给的量最大是驱动那一份（K1-1 第 5 点定向：多给 2560）。不是这一批引进来的，但 S6 让「丢了之后的每次判」更严，这一形是它的反面。什么现象会推翻：第 ① 段借草稿区之前有一步先问过 `prepare`（读码没有）。
2. **叫停落在第 1、2 轮之间的断言 panic**（K1-5 第 3 点）：这是「判红」那一类的新路径，由第 E 条引进来；窗口窄，后果是整个进程 panic 而不是返回 `StoppedOnRequest`。
3. **甲的少给在第 E 条再起一轮时被碰到**：上一轮的借据在轮末丢掉，销账的整笔进「卡上可能还没还的」，卡真还了之后这一份只压到占着的，驱动那一份被多扣。我在 harness 档定向造了一遍（`probes/killers.rs` 的 `k_pressing_lets_a_later_lease_count_once_the_card_took_the_returned_one_back`，真代码上绿）：一笔 10000 建满又还了、卡收回、再借一笔 5000 建满：账给 `21990`，真空闲 `24550`，少 2560。多扣的量不超过占着的，卡仍装得下一个状态时只是状态数变少、不报本次未跑；本次未跑要「此刻连一个状态都带不了」（`decide_scratch_before_booking`，`mebibytes_a_judge_needs(&tables, 1)` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:2960`），表本身很大时这 2560 可能决定能不能装下一个状态——这是甲认的代价在第 E 条上的一次，不单列为打中。
4. **一张卡报过 OOM，这一趟不再用它**（设计，不是漏）：「for (index, reason) in &received.cards_out_of_memory {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3101`）里把卡记进集合，之后的轮用 2937 行的过滤排除。单卡机器上一次瞬时 OOM（别的进程占了一阵）就整趟报本次未跑；这与改前相同。要不要给报过的卡一次重试，是 E174 乙的题。
5. 没找到的：`NotRun` 臂的 `expect`（「.expect("这一轮有卡报了 Out of Memory 才会走到本次未跑这一支");」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3140`））在 `cards_short_of_memory_this_round > 0` 时才可达（`after_a_round_of_the_cards` 的 2024 行把 0 的情形分走），而同一轮里 3101–3103 行已把 `first_out_of_memory` 置上，所以不会 panic；`after_a_round_of_the_cards` 的有限性见 K1-5 第 1 点。

## K3　测试判别力

方法：副本里对 `gpu_memory_ledger.rs` 非测试段，每条自造变异改一处（改前串在测试段之前恰好命中一次）、跑 `cargo test --offline -p singlefs-harness --lib gpu_memory_ledger`、还原；存活的再用不带过滤的 `--lib` 跑一次确认不是别的模块拦住（`logs/mut-full-*.log`）。基线 `--lib` 154 通过（见 0）。驱动 `mut/mutate_k3b.py`（`mutate_k3c.py` 是第二批），原样输出 `logs/mut-*.log`。

结果（35 条，M 号按两份脚本里的次序；M01–M28 在 `mutate_k3b.py`，M29–M35 在 `mutate_k3c.py`）：

| M | 改回去 / 改坏的 | 结果 |
|---|---|---|
| M01 | 读数进账那一步去掉（不压） | **照绿**（`--lib` 154 条也绿） |
| M02 | 销账把那一份换成这一笔（不累加） | 红：`returned_short_leases_while_a_long_lease_stays_built_each_stay_booked_as_possibly_on_the_card` |
| M03 | 销账不记进卡上可能还没还的 | 红 4 条 |
| M04 | 丢了重起：旧设备那一份不记 | 红 3 条 |
| M05 | 丢了重起：驱动那一份仍记成 `DeviceStarted` | **照绿**（`--lib` 也绿） |
| M06 | 实占先扣的那一份去掉 | 红多条 |
| M07 | 实占里属于许出去的不封顶 | 红 2 条 |
| M08 | 压法永不往下压（只改 `OnlyThroughTheBooks` 臂） | 红：`a_reading_that_cannot_tell_this_process_apart_leaves_the_returned_share_unpressed` |
| M09 | 没列或另有不走账的设备时压到 0 | 红：同上 |
| M10 | 重起判差 1 MiB 也放行（`capacity + 1 < leases`） | **照绿**（`--lib` 也绿） |
| M11 | `DeviceLostNotStartedAgain` 臂直接放行 | 红：`a_lost_device_refused_a_restart_is_judged_again_on_every_later_attempt` |
| M12 | `DeviceStarted` 臂丢了第一次也不判 | 红 2 条 |
| M13 | 被拒之后又借一笔，驱动那一份被记回 `RegisteredDeviceNotStarted` | **照绿**（`--lib` 也绿） |
| M14 | `is_this_process_registered_on_the_card`：Lost 算没登记 | 红 2 条 |
| M15 | 报到：不等其余几段（`while false && …`） | 红：`a_piece_failing_on_a_lost_device_is_told_after_another_piece_records_its_out_of_memory` |
| M16 | 报到：最后一段不 `notify_all` | **挂住**（等的段不被叫醒，45 秒超时前没有一条红；用例里的 `join` 无超时） |
| M17 | 报到：没报到就丢掉时不补报 | 一条红，另有挂住 |
| M18 | 卡线程那一圈不问 `must_stop`（`while true || !must_stop()`） | **照绿**（`--lib` 也绿） |
| M19 | 块队领序号最大的 | 红 2 条 |
| M20 | 块队交回两次不断言 | **照绿**（守卫） |
| M21 | 写库失败之后到的照写（去掉 `continue`） | 照绿（**等价**：写库失败后 `blocks_stored` 不再前进，`waiting.remove(&self.blocks_stored)` 取不到，那条 `continue` 够不着——死码） |
| M22 | 几轮：叫停也再起一轮 | 红：`after_a_round_another_round_starts_only_when_a_card_ran_out_and_another_card_is_left` |
| M23 | 几轮：没有没报过的卡也再起一轮 | 红 2 条 |
| M24 | 几轮：这一轮没卡报 OOM 也不当叫停 | 红 |
| M25 | 范围里接到的 OOM 不记进设备 | 红 |
| M26 | 先认后等 | 红 |
| M27 | 重起判读数绕开 `take_in_a_reading` | 照绿（**等价**：压的只影响之后的读数，这一次判的读数不变） |
| M28 | Lost 臂的「许出去的」不含驱动那一份 | 照绿（少给方向的精度，不是安全方向） |
| M29 | 块队 `any_left` 取反 | 红 |
| M30 | 卡线程那一圈：判不了（`Refused`）也接着领 | **照绿**（`--lib` 也绿） |
| M31 | 卡线程那一圈：写库那头走了也接着领 | **照绿**（`--lib` 也绿） |
| M32 | 几轮：还有 1 块没写也算判完（`<= 1`） | 红 |
| M33 | 按块序写库：写一块算两块 | 红 4 条 |
| M34 | 报到：报多了不 panic | 照绿（守卫） |
| M35 | 报了 OOM 的卡交回之后返回 `NothingLeftOrStopped` | 红 3 条 |

实现员报告里自己那几行变异证红（`prove-red.sh` 三次汇总）我没重跑，不评。上表里照绿的实质项是 M01、M05、M10、M13、M18、M30、M31（七条）；M16 是挂住不是红；M20、M34 是守卫；M21、M27 等价；M28 精度。

### K3 补充 1：对七条照绿的实质项，我写了会红的用例草稿

`probes/killers.rs`（放 `crates/singlefs-harness/tests/`，只用公开函数）五条用例，真代码上全绿（`logs/killers1.log`：`test result: ok. 5 passed; 0 failed`），对变异：
```
M01 读数不压（take_in_a_reading 的调用去掉）: 红：k_pressing_lets_a_later_lease_count_once_the_card_took_the_returned_one_back (6s)
M05 设备丢了重起：驱动那一份仍记成 DeviceStarted: 红：k_a_refused_restart_asked_again_does_not_take_another_driver_share_off_the_room (4s)
M10 重起判：恰差 1 MiB 放不下也放行: 红：k_the_restart_is_refused_when_one_mebibyte_short_and_allowed_when_exactly_enough (3s)
M13 借了一笔在 DeviceLostNotStartedAgain 时把驱动那一份记回登记了、没起: 红：k_a_lease_booked_after_a_refused_restart_keeps_the_restart_judged (6s)
M18 卡线程那一圈：不问叫停: 红：k_the_card_loop_stops_on_request_on_refusal_and_when_the_store_is_gone (2s)
M30 卡线程那一圈：判不了（Refused）也当没事接着领: 红：k_the_card_loop_stops_on_request_on_refusal_and_when_the_store_is_gone (1s)
M31 卡线程那一圈：写库那头走了也接着领: 红：k_the_card_loop_stops_on_request_on_refusal_and_when_the_store_is_gone (1s)
```
（`logs/killers-vs-mutants.log`）。这是草稿，不进库；是否并进实现员的用例由主 agent 定。

### K3 补充 2：checker 档的变异行与没有变异行的地方

- 变异行锚点：我对 `crates/mutations.tsv` 里落在四份被判文件上的 179 行逐行核了「原文恰好命中一次、必须红的测试名在 `crates/` 下的 `.rs` 里找得到」，原样输出 `rows on the four touched files 179 bad 0`，与实现员报告的 `rows 179 bad 0` 相同。这只核锚点，没跑（checker 档变异不是子 agent 的活）。
- 这一批新加的 17 行里 3 行在 checker 档（`崩溃放量 K3 单元校验和对拍时报了显存不够的那一块照样写库、接着领`、`GPU 取卡 K3 设备丢失的回调不记丢失`、`GPU 取卡 K3 按优先级取卡的入口照归卡池起设备`），用例都是不要卡的（`crash_amplification::tests::a_block_with_a_piece_short_of_gpu_memory_is_not_stored_and_no_later_block_is_taken` 读来是真会红：假段 `local.start == 5` 报显存不够，`stored_block_starts` 断言 `vec![0, 2]`；删掉 1535–1538 行的 `break` 后第 3 块会写，红）。
- **没有任何变异行也没有不要卡用例的地方（改回去照绿，读码推的，没跑）**：
  1. 第 ① 段外层循环（`crates/singlefs-checker-tier/src/crash_amplification.rs:2927` 起到 3148 行）：把 2937 行的「报过的卡不再用」过滤去掉；把 3105 行的「blocks_of_the_round = work.blocks.blocks_not_taken();」改成全部块（每轮重判全部）；3081 行那条断言；3094 行 `extend`。实现员报告「没做什么」一节自己写了这一条。
  2. 第 D 条在 checker 档的接线：1498–1500 行「每段领报到」；改成每段各自 `WorksEndingTogetherOnADevice::new()`（等于不等）也没有用例红。
  3. 对拍用例本身（C）：删掉 282 行那一整段 `if let Some` 无法被变异表拦（变异表要必须红的测试名，而这条用例只在真卡 OOM 时才分得出）。
  4. `outcome.cards.extend` 的重复（K1-5 第 4 点）没有用例。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 做到；一形漏（窗口） | 三处「见到的」用法零命中；两腿模型、我的丢/重起模型 0 次多给；设备丢了账没被问到的窗口多给最多一份驱动（19550 对 16990），起设备前被重起判纠正 |
| K1-2 | 每次都判：做到；回到正常：没有 | S6 三次都拒；重起一次多扣一份 D，k=5 时 6990 对 19550（甲之下，封顶在占着的） |
| K1-3 | 做到（读码，复核不了真卡） | Some 打未跑行返回；None 钉 `layer0_plan_state_count`；没找到一形照绿 |
| K1-4 | 做到；一形设计内 | 报到领齐、丢掉补报、无跨段同步；设备丢了且同块有人报过 OOM 时真 panic 会被说成显存不够（原设计，等齐把窗口拉宽） |
| K1-5 | 判两遍/漏判/无限再起：没有；判完不入库：有；一处断言 panic | 模型 3000 格 `violations=0`；叫停落在第 1、2 轮之间撞 3081 行断言（`blocks_waiting()=9`）；`cards_used` 算两遍；本次未跑收尾白判最多 3 块 |
| K2-1 | 新代码没有多给 | 多给只在原有的丢失窗口里 |
| K2-2 | 有两形 | 窗口里多给被纠正后整张卡这一趟不用（单卡整趟未跑，读码推）；叫停断言 panic（新）；甲的代价在第 E 条上再碰一次 |
| K3 | 七条实质项照绿，补了会红的草稿 | M01、M05、M10、M13、M18、M30、M31 照绿；M16 挂住不红；checker 档循环与接线没有用例也没有变异行 |

## 没做什么

- 没跑 checker 档任何测试，没起 GPU 设备，没跑 `check.sh`、门禁、`cite-check` 之外的 lint；没有占卡。`nvidia-smi` 只读一次。
- 第 E 条的循环是我照产品代码逐句搬的模型（`probes/rounds_sim4.rs`），不是产品循环本身；与产品代码不同之处至少有：卡上借草稿区的压状态数（模型用固定的 `holds_states`）、判器缓存、真线程调度与 `stop_is_requested` 的取值时机。第 3081 行断言 panic 是读码加模型的推断，没有在产品代码上跑到；要证实得在 checker 档造一条不要卡的用例（把循环抽成纯函数，或给 `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 一个假卡入口）。
- 「设备在卡上真丢了」「真卡上的 Out of Memory」我造不出，K1-1 的窗口、K1-4 的第 3 点、K2-2 的第 1 点都是读码加纯函数，不是真卡上的结果。
- 没验「借的量 ≥ 建上的」这条 10x/10y 批已定的前提（K1-1 第 3 点依赖它）。
- 没重跑实现员的 `prove-red.sh` 三次证红；没跑 checker 档变异行（`crates/mutations.tsv` 里 17 行新增、4 行替换，只核了锚点）。
- 不判别的腿的格（K1–K3 之外）、不替主 agent采纳或出判决；上面「判」字都是对着代码与条款的正推判定，不是判决。
- 乙（E174）的量不在这一轮；`cards_used` 算两遍与 `blocks_waiting` 断言是否要改、怎么改，是主 agent 的事。
- 重启类的压力只造了一块卡（`book_on_the_card(0, …)`）；多卡并发的账（`book_states_on_the_cards_with_room`）读码没发现新问题，没另造模型。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/`：留 `probes/`、`mut/`（含 `gpu_memory_ledger.rs.orig`，即冻结副本里那份的拷贝）、`logs/`，都已拷进模型目录。交回前删了自己建的仓副本 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/tree/`（21M，变异都已还原后才删）与编译目录 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-sonnet/target/`（2.5G）；复跑照模型目录 `README.md` 从冻结副本重拷、探针放进 `crates/singlefs-harness/tests/` 即可（编译约十几秒）。
