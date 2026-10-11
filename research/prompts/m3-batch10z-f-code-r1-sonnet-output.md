# 正推腿报告：根环实现第 10z 批第六块代码轮第一轮（m3-batch10z-f-code-r1，2026-10-08）

立场：正推。从用户原话、E174 读数、第五块判决第三节三条与 `crates/` 今天的实现推「应该是什么」，再与冻结副本逐格比。
冻结副本 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree/`（下文路径都是它下面的路径，行号都是现查的）。
没读禁读清单里的任何东西。第二节「已知、在修」的两处（`next_card_to_start` 每起一张卡多问一次叫停；库单测打 2 行 `NOT_RUN_NO_GPU_ADAPTER`）不报。

## 0 开工核对与我用的工具

- 快照：我拷出来的 `copy-a`、`copy-w0..3` 对 `research/prompts/m3-batch10z-f-code-r1-snapshot/crates-sha256.txt` 逐份核（跑变异之后也核了一遍，还原无误）：

```
copy-w0 not-OK lines: 0
copy-w1 not-OK lines: 0
copy-w2 not-OK lines: 0
copy-w3 not-OK lines: 0
copy-a not-OK lines: 0
```

- 基线（拷贝上，harness 档，`cargo test --offline -p singlefs-harness --lib -- gpu_memory_ledger`）：`test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out`。
  新 harness 档用例（`gpu_card_rounds_remember_a_stop_request_removed_mid_run`）基线：`STOP_FILE_REMOVED runs=6000 ends={"EveryBlockStored": 3324, "NotRun": 1165, "StoppedOnRequest": 1511} asserts_panicked=0 cards_panicked=0 stopped_on_request_but_run_not_marked_stopped_early=0 stop_seen_and_the_file_gone_at_the_end=374`。
- 工具（全部在拷贝上、harness 档、经 `run-with-memory-cap.sh 1G` 与 `capped.sh`，线程 ≤ 4，没起 GPU 设备）：
  1. 变异注入：这一批追加的 27 行与整行替换的 9 行里，harness 档那 31 行逐行注入、跑那一行点名的测试（`run_mut.py`）；另补 9 条探针（`probes.json`）。
  2. 自己写的模型 `research/prompts/m3-batch10z-f-code-r1-sonnet-model/sonnet_g2_model.rs`：真 `GpuMemoryLedger` 加一张假的物理卡，只用公开接口。
  3. checker 档两条扫源码用例的 Python 复刻（`emulate_scans.py`，复刻不是原用例）。
  4. 一个 rustc 小程序（`resume_in_drop.rs`）。
- 重型测试没跑：checker 档测试、带 gpu 特性的测试、碰卡用例都没跑（要卡、归主 agent）；真卡上的 Wait 超时、Wait 时设备出错、真丢设备，都复核不了（没造）。

## K1　G2 乙做到没有

### K1-1　每一条「缓冲交回驱动再销账」的路都先等了吗

应该是：账上销一笔借据之前，这张卡上这笔借据建过缓冲的路径都已经等过设备（E174 的等待臂在两张卡、两段上每一栏第一份读数都不含那笔缓冲，见 `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:14` 的「t0 之后的第一份」）；没建过缓冲的路径（设备没起来）没有东西可等。

今天的实现，逐路：

| 路 | 代码 | 等了吗 |
|---|---|---|
| 正常丢 `GpuCardLease` | 「device.poll(wgpu::PollType::Wait {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1264`），超时「std::time::Duration::from_secs(30);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:27`） | 等；结局交「self.booking.take_in_the_wait_before_returning(wait);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1280`），之后字段析构才销账（`card` 在 `booking` 之前析构，没改次序） |
| panic 展开中丢借据 | 同一个 `drop`；poll 自己 panic 时被 `catch_unwind` 接住，记成没等成，再「std::panic::resume_unwind(panic);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1282`） | 等；poll 没 panic 时展开照常往下走 |
| 起设备失败（`card_lent_for_the_booking`） | 「return Err(reason_for_running_out_of_gpu_memory(&format!(」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1516`），`booking` 随参数丢掉 | 不等；这一笔借到之后设备没起来，没有缓冲，一致 |
| 重起 | 老一代借据的 `GpuCard` 克隆拿着旧设备，`drop` 里等的是旧设备；新一代借据等新设备 | 等，等的是各自的设备 |
| 不走卡池的设备（`GpuCardsSource::DevicesStartedForThisRun`） | 不进账、没有 `GpuMemoryBooking` | 没有账可销 |

只有卡池建 `GpuMemoryBooking`：`grep -rn 'GpuMemoryLedger\|book_on_the_first_card_with_room\|book_on_the_card\|book_states_on_the_cards' --include=*.rs crates`，在 `crates/singlefs-harness/src/gpu_memory_ledger.rs` 之外只命中 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`（第 1165、1205、1385、1424、1471、1720 行，都在卡池里）。

借据先于缓冲丢的顺序前提：产品路径里判器都在卡线程的作用域里丢掉，「scratch.leases.push(lease);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2873`）攒下的借据在一轮收尾时才丢；对拍与核对用例里判器是借据之后声明的局部变量（反序析构）。我读了 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:941` 起的一处和 `crates/singlefs-checker-tier/tests/` 下五处借卡的用例，没找到借据先于缓冲丢的。这个前提是靠局部变量次序守的，没有任何测试钉它（见 K4）。

展开中丢借据时 poll 自己 panic：我用 rustc 直接编 `resume_in_drop.rs`（Drop 里 `catch_unwind` 之后 `resume_unwind`）：

```
--- normal      → normal-drop: caught outer? true      rc=0
--- unwinding   → thread 'main' panicked ... panic in a destructor during cleanup
                  thread caused non-unwinding panic. aborting.      rc=134
```

即展开中第二次 panic 会 abort。这不是这一批新增：改前的 `poll(Poll)` 同样走 `handle_error_fatal`（转述：wgpu 30.0.1 `src/backend/wgpu_core.rs` 第 1924 行 `Device::poll` 里非 Timeout 的错一律 panic），同一类设备出错在展开中丢借据，新旧都 abort。

判定：**一致**（每条建过缓冲的销账路都先等）。附：展开中遇设备致命错会 abort（新旧同），顺序前提无测试（归 K4）。
什么会推翻：找到一处 `GpuMemoryBooking` 在池外被建，或一处借据先于它的缓冲丢掉。

### K1-2　等失败那一支

应该是：等失败时账给的量不大于此刻真能再借的；「一直少借」要有界，不致让后面的借都报本次未跑（用户原话：「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:7`）；不压并发是同一行的另一句）。

今天的实现：等失败的那一笔「LeaseWhenTheBookingIsDropped::KeptOnTheBooksAfterAFailedWait => {}」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1434`），`Drop` 什么都不做，借据留在 `leases` 里；唯一销它的地方是重起记账那一步「self.leases.remove(&kept_lease);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:848`）。

量（模型 `sonnet_g2_model.rs`，真账加假卡）：
1. 多给：随机操作（借、建、缩、等成功还、等失败还且缓冲「下一次别人 poll 才还」或「设备销毁才还」、丢设备、重起被拒、起设备物理 OOM）20 万个种子，每个 60 步，账给的 `room` 超过物理上真能再借的 0 次：`K3-1 random: seeds=200000 violations=0`。判官有牙：把「等失败照样销账」注入同一个模型，`k3_1` 红（`control-w0.log` 的 ROW 0：`RED(caught)`，`test k3_1_random_ops_never_let_the_ledger_give_more_than_is_physically_lendable ... FAILED`）。
2. 少借的界：没有设备丢失时，等失败的借据永不销。每笔 4000 MiB 等超时、事后缓冲还回驱动（卡上借据缓冲为 0、`free_true=29400`）：

```
K1-2 round 0: room_by_ledger=23440 free_true=29400
K1-2 round 1: room_by_ledger=19440 free_true=29400
K1-2 round 2: room_by_ledger=15440 free_true=29400
K1-2 round 3: room_by_ledger=11440 free_true=29400
K1-2 round 4: room_by_ledger=7440 free_true=29400
K1-2 round 5: room_by_ledger=3440 free_true=29400
K1-2 round 6: refused
K1-2 round 7: refused
```

即最坏少到整张卡：每一笔等失败的借据永久扣掉它的全量，累计到再借不出一笔 4000，后面每一次借都是 `NoCardHasRoomNow`，调用方报本次未跑。
触发条件是 Wait 超时（30 秒）、`PollError` 或 panic。E174 里没有出现过：「wait_always_returns=true」，等一次最长 0.69 秒（`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:15`：「等一次要 0.3 到 0.69 秒」）。真卡上会不会超时：复核不了（没造）。

对照两边整行：
- 实现员（`~/.local/share/singlefs/work/m3-r1/batch10z-f/patch/report.md` 第 53 行）：「代价只是这一笔一直少借」
- 模型：上表，最坏是整张卡借不出、报本次未跑；这与第三节 K3 问的「卡放得下却报本次未跑」同形。

判定：不多给 **一致**；「一直少借」的界 **冲突**（不是一笔，是累加到整张卡；没有设备丢失就没有出口）。
什么会推翻：销账有一条不经设备丢失的出口（例如事后再 poll 一次、成了就销）；或证明等失败在这台机器上不可能发生（要真卡长时间观测，我复核不了）。

### K1-3　丢了的设备两种状态

应该是：两种状态下照读不到这个进程占着的算，只会少给，不会多给；少给的那一段在「设备丢了那一刻活着的借据」都还了之后出来。条款没写这一格（实现员自定第 2 条）。

今天的实现：两状态交 0：「| DriverShareOnTheCard::DeviceStartedAgainWhileLeasesOfTheLostDeviceRemain => return 0,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:206`）；回到正常的出口只有借据归还时那一个条件「if self.leases_alive_when_the_device_was_lost.is_empty() {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:871`）。「设备丢了那一刻活着的」集合在重起记账时填：「.extend(self.leases.keys().copied());」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:853`），触发重起的那一笔借据已经在 `leases` 里，所以它也在集合里。

多给：没有。同一个 20 万种子模型（含丢设备、重起被拒、起设备物理 OOM）0 次；判官有牙：下面三条注入都让 `k3_1` 红：重起好了就当旧设备已经不在（追加表第 8 行，`control-w1.log` ROW 1）、旧设备那一份当成新设备的驱动那一份（追加表第 7 行，`control-w2.log` 的 ROW 2）、第二次丢设备（Remain 状态）不重新记账（探针 P5，144 个种子红：`K3-1 random: seeds=20000 violations=144`，`p5-trace.log`）。

少给，量出来两段（模型 `k3_2`、`k1_3`，同一张 30000 MiB 的卡，A 借 10000 建满，驱动实占 600、额度 2560）：

```
K3-2 A alive (Remain): free_true=13800 room_by_ledger=0 safe_room=11840 short_by=11840
K3-2 A returned ok  : free_true=24400 room_by_ledger=16840 safe_room=22440 short_by=5600
K3-2 B returned ok  : free_true=29400 room_by_ledger=27440 safe_room=27440
K1-3 after: free_true=29400 room_by_ledger=16840 (safe_room=27440) lease A on books=true
K1-3 later: room_by_ledger=16840 (free_true=29400)
K1-3b stuck(before 2nd loss): room=16840 safe=27440 | after 2nd restart and C returned: room=27440 safe=27440
```

1. 暂时的一段：触发重起的那一笔 B 也在集合里，所以 B 活着时（它拿的其实是新设备）整段都照读不到算，少给等于所有已建缓冲加驱动那一份（上表第一二行）。A、B 都还了回到 `room == safe_room`（第三行）。
2. 永远的一段：集合里的借据只在「归还」时出集合；设备丢了那一刻活着的借据 A，丢掉时 Wait 失败 → 走「不销账」，`return_a_lease` 不被调，A 永远在集合里 → 永远 Remain；room 一直少 10600 MiB（= A 的 10000 加驱动已占的 600），再借再还若干笔也不变（`K1-3 later`）。出口只有下一次丢设备：重起记账时把没等成的借据从 `leases` 和集合里一起去掉（`K1-3b`）。实现员报告第 53 行自认「要等下一次丢设备才销（推的，没上卡造过）」，上面是量出来的数。

判定：多给 **一致**（没有）；少给的暂时一段 **一致**（设计内，只会少借）；永远一段 **规则没说**（条款没写这一格，与「只会少借」同方向，但没有出口）。
什么会推翻：（暂时一段）借据归还后 `room` 没回到 `safe_room`；（永远一段）找到一条路让旧设备那一代的等失败借据在没有第二次丢设备时出集合。真卡上旧设备被 OOM 丢掉之后，旧一代借据的 Wait 会不会失败：复核不了（转述：wgpu-core 30.0.1 `src/device/resource.rs` 第 5436 行 `release_gpu_resources` 是设备失效且队列空时释放缓冲，OOM 丢的设备 `raw` 还在，Wait 多半成功；硬件丢的设备 Wait 走 `handle_hal_error` 后 `handle_error_fatal` 会 panic）。

### K1-4　「卡上可能还没还」那一份拿干净没有

应该是：字段、往下压那一步、重起时往上加驱动那一份，三处都没有了。

命令与输出（冻结副本 `crates/` 下）：

```
$ cd ~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree
$ grep -rn 'returned_possibly\|take_in_a_reading' --include=*.rs crates | wc -l
0
$ grep -n 'returned_possibly\|take_in_a_reading' crates/mutations.tsv | wc -l
0
```

另有一处名字里带「卡上可能还没还」：`crates/mutations.tsv` 第 3201 行（`grep -n '卡上可能还没还' crates/mutations.tsv` 命中的变异名），它的原文已换成 `…RemainDriver` 那一支，不引用旧字段。「销了账」的其余命中都是注释与断言文字（`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 25、157、190 行），不是口径。重起时往上加那一句（`saturating_add(mebibytes_left_for_the_driver)` 加到「销了账而卡上可能还没还」）在 `take_back_the_driver_share_of_a_lost_device` 里已删（diff 第 828–831 行），现在的函数是「fn take_back_the_driver_share_of_a_lost_device(&mut self) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:845`），不带参数。别处仍有的 `saturating_add(mebibytes_left_for_the_driver)` 只在「许出去的 = 借据 + 驱动那一份」这一处（第 199 行起），是现行口径。
判定：**一致**。什么会推翻：全仓 grep 找到旧字段或旧函数名的一处使用。

## K2　第五块判决第三节三条做到没有

### K2-1　叫停记住

应该是：判决第三节第 1 条「一趟里见过一次叫停就记住」（`research/prompts/m3-batch10z-e-code-r1-main-verification.md:37`），「两样结局都要为 0」（同一行）：外层循环、卡线程、一轮收尾、整趟收尾问的是同一个。

今天的实现：读文件只有一处，harness 档 `StopRequestRememberedInThisRun`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2221`）；checker 档每个读叫停的地方：
- 外层 `run_from_environment` 建一个：「let stop_request = StopRequestRememberedInThisRun::new(&stop_is_requested_by_the_environment);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5594`），整趟收尾问它：「let stop_was_requested = gpu_memory_was_short || stop_is_requested();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5677`）；GPU 判的调用（第 5611 行）与 CPU 判的调用（第 5646 行）传的都是同一个闭包 `&stop_is_requested`。
- 第 ① 段函数里再包一层：「let stop_request = StopRequestRememberedInThisRun::new(stop_is_requested);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2947`）。卡线程（`stop_of_the_cards`）、起下一张卡（第 2476 行）、一轮收尾「let stop = stop_request.stop_of_the_run();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3116`）问的是这一层。
- 两层的关系：内层 true 只可能来自外层 true（内层只在自己没见过时才调外层），外层 true 之后恒 true；所以一轮收尾读到 true 时整趟收尾必然读到 true，反过来整趟收尾先读到 true 不影响一轮收尾（它也会读到）。同一条线程读 true 与存标记之间别的线程读到 false 的缝存在，但一轮收尾在 `thread::scope` 收齐之后才读，缝里的那条线程已经存完。
- 全仓读叫停文件的函数 `stop_is_requested_by_the_environment` 只有这一个调用点（`grep -c 'stop_is_requested_by_the_environment' crates/singlefs-checker-tier/src/crash_amplification.rs` 输出 4：第 1389 行定义、第 5594 行建记住的那一个、第 5081 与 5122 行是 `///` 注释；`grep -rn ... --include=*.rs crates` 在别的文件零命中）。

量：实现员的 harness 档用例我换一组种子再跑：`sonnet_stop_wide.rs`（`SCRIPTS` 6000、`FIRST_SEED=0x5EED_F00D`，24000 段）：

```
STOP_FILE_REMOVED runs=24000 ends={"EveryBlockStored": 13463, "NotRun": 3882, "StoppedOnRequest": 6655} asserts_panicked=0 cards_panicked=0 stopped_on_request_but_run_not_marked_stopped_early=0 stop_seen_and_the_file_gone_at_the_end=1602
```

两样结局 0；窗口（见过叫停、整趟收尾时文件已不在）走到 1602 段。局限：这个用例里的外层循环、卡线程是实现员「照次序抄成不碰卡的一份」，不是 checker 档的真代码；真代码的接线靠上面的读码与扫源码用例，不是被跑出来的（见 K4）。
另一条老路：文件在整趟收尾之前才出现（所有块都已判完、没人见过叫停），整趟收尾读到 true → `stopped_early=true` 而 `unjudged=0`。这在改前就是这样（改前整趟收尾也现读一次），不是这一批带来的。

判定：**一致**（做到同一个；两样结局在 2.4 万段和实现员的 6000 段上都是 0）。
什么会推翻：一处读叫停文件的新调用点；或一轮收尾问到的和卡线程问到的不是同一个对象（例如有人把 `CardsAtWork` 的字段改回闭包）。

### K2-2　头一句 OOM 那一行

应该是：判决第三节第 2 条「叫停加有卡报 OOM 而照叫停收尾时，把头一句 OOM 照本次未跑那一行的形态打出来」（`research/prompts/m3-batch10z-e-code-r1-main-verification.md:38`）；不在不该打时打。

今天的实现：打印在照叫停收尾那一支里：「AfterARoundOfTheCards::StoppedOnRequest => {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3135`），行由 `RoundsOfTheCards::not_run_line_of_a_stop_after_an_out_of_memory`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2394`）给，没有卡报过交 `None`。何时走到这一支（`after_a_round_of_the_cards`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:2283` 起）：
1. 「StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2297`）：这一轮有卡报 OOM、叫停、没有判完在等的块。判决字面的情形。
2. 「if cards_short_of_memory_this_round == 0 {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2293`）之下的 `StoppedOnRequest`：这一轮没有卡报，但早几轮有卡报过（`first_out_of_memory` 是 `Some`）。这一支比判决字面宽，实现员自定第 5 条；判决第 38 行没有写「这一轮」，写的是「叫停加有卡报 OOM」，所以没有与字面冲突，背景材料写的「判决写的这一轮」我在判决第 38 行里没找到。
3. 一个不会打的情形：别的卡把全部块判完（`EveryBlockStored`）走 `break` 不经这一支，只打一行 `CRASH_AMPLIFICATION_GPU_OUT_OF_MEMORY_ON_A_CARD`（第 3127 行起），不是本次未跑那一行。
会让门禁多数一条本次未跑的情形：OOM 的那张卡没有让任何一块丢掉（它交回的块被别的卡判了），之后因为叫停而收尾。模型 `k2_2`（真 `RoundsOfTheCards`：卡 0 领一块报 OOM 交回，卡 1 判完 3 块写库，叫停）：

```
K2-2 after=StoppedOnRequest blocks_stored=3 blocks_waiting=0 line=Some("NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=卡 0 报了 Out of Memory")
```

即打了一行本次未跑，库里的 3 块是完整的块序前缀，没有块因 OOM 而缺。这一形按判决第 38 行字面是该打的（OOM 缘由要外面读得到），按「本次未跑 = 这一趟没跑完」读是多一条。我把这一形记为 **规则没说**，交主 agent 定。
同一趟不会打两行：该支打完就 `break`，后面叫停分支（`crates/singlefs-checker-tier/src/crash_amplification.rs:5677` 起）不再打本次未跑那一行。
判定：该打的时候打了（**一致**）；不该打的边界一格 **规则没说**。
什么会推翻：一次叫停加 OOM 的收尾没有这一行（harness 档用例 `a_stop_after_an_out_of_memory_prints_the_first_out_of_memory_in_the_not_run_line` 钉了整行与 `None` 一支）；或同一趟出现两行。

### K2-3　七条会红用例与抽到 harness 档的两处：改回去是不是都红

应该是：判决第三节第 3 条「checker 档接线能抽成 harness 档纯函数的抽出来测」「抽不出的写明为什么」（`research/prompts/m3-batch10z-e-code-r1-main-verification.md:39`），七条各一行变异。

harness 档那一半我逐行注入、跑那一行点名的测试（拷贝上，`run_mut.py`；还原后 `sha256sum -c` 全 OK，见第 0 节）：

```
$ cat mut-w*.log | grep '^ROW' | awk '{print $3}' | sort | uniq -c
     31 RED(caught)
      5 SKIP(not
```

31 行全红：追加表 22 行（G2 乙 10 行、叫停 3 行、F4 与起下一张卡 2 行、M05 M10 M13 M14 M17 M22 M23 七行）加整行替换 9 行。每一行红的都是它点名的那一条测试（日志里 `FAILED` 的测试名与变异表第 6 列相同；例：M05 红在 `the_largest_judged_block_not_the_last_judged_lets_a_stopped_card_take_the_block_it_waits_for`，F4 红在 `a_work_notes_the_device_at_the_moment_it_returns_not_before_and_not_after`，起下一张卡红在 `the_next_card_starts_only_when_it_holds_states_and_a_block_is_left_to_take_under_the_stop`）。`SKIP` 的 5 行是 checker 档变异（下表）。

checker 档那 5 行，我不能跑（重型），逐行能做到的：

| 变异行（`crates/mutations.tsv`） | 我能做的 | 结果 |
|---|---|---|
| 3370 借据丢掉时改回只收一次（Poll） | 读码 + E174 | 只有碰卡用例盯它；**我推它改回 Poll 照绿**（K4） |
| 3371 没等成也当等完了 | 读码：用例对 `Poll` / `Timeout` / panic 三种结局都断言「没等成」 | 红（读码，没跑） |
| 3372 扫 wgpu-core 找的不是 `self.lose(&error.to_string())` | Python 复刻该用例的扫描 | 基线 `(True, 1)`，变异 `(True, 0)` → 断言 `count == 1` 红 |
| 3373 整趟收尾改回现读叫停文件 | `emulate_scans.py` | 基线 `scan1_pass=True`，变异 `scan1_pass=False` → 红 |
| 3374 照叫停收尾那一支不打头一句 OOM | `emulate_scans.py` | 基线 `scan2_pass=True`，变异 `scan2_pass=False` → 红 |

丢失缘由用例：期望句写死「let out_of_memory = "Out of memory";」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2109`）；wgpu-hal 30.0.1 的 `DeviceError::OutOfMemory` 的 `#[error("Out of memory")]` 在 `src/lib.rs` 第 382 行（转述）。这条防的是 wgpu 以后换句子，仓里没有会模拟它的变异，实现员自己写明（报告第 98 行）。
判定：harness 档 **一致**（31/31 红）；checker 档三条复刻红、一条读码红、一条（Poll）见 K4。
什么会推翻：任何一行变异注入后点名的测试照绿。

## K3　只防 OOM、不压并发两头（新代码引进来的）

### K3-1　有没有一形账给的量大于此刻真能用的

应该是：任何时刻 `room` 不大于「空闲 − 还没建的借据 − 驱动还没长到的那一份」。

量（模型 `k3_1`：1 张 30000 MiB 的卡，借 500–8500 MiB、建、缩、等成功还、等失败还、丢设备、重起被拒、起设备物理 OOM，每个种子 60 步，每步比一次 `room` 与物理上的 `safe_room`；设备丢了而账还不知道的那一段不比，因为账要到下一笔借据触发重起才知道，重起那一步 `prepare` 会再判一次）：

```
K3-1 random: seeds=200000 violations=0
```

判官有牙（同一模型，注入后重跑）：

| 注入 | `k3_1` |
|---|---|
| 等失败照样销账（这一批的变异行，追加表第 5 行） | 红 |
| 重起好了就当旧设备已经不在（追加表第 8 行） | 红 |
| 旧设备那一份当成新设备的驱动那一份（追加表第 7 行） | 红 |
| 第二次丢设备（Remain 状态）不重新记账（探针 P5） | 红，144/20000 个种子 |
| `DeviceLostNotStartedAgain` 也记占着（探针 P1）、重起没等成的那几笔不同时出集合（探针 P3） | 照绿（这两条不是多给，P1 的状态我的模型停在「设备丢了」那一段不比；P1 被 lib 用例抓住，P3 见 K4） |

前提（模型保不了的）：一、「等成功之后，下一次 `nvidia-smi` 读数里这笔缓冲不在」——这是 E174 的读数：等待臂每一栏第一份读数都不含（`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:14`），只测了一笔缓冲、每张 1 GiB 以内、两张卡、驱动 595.84、`MemoryHints::Performance`、Vulkan；多笔小缓冲、别的驱动不保证；二、借据先于它的缓冲丢的顺序前提（K1-1）；三、别的进程与两进程同读一份空闲（用户已认）。
判定：**一致**（模型里没有多给；前提一条是 E174 的读数，一条是读码）。
什么会推翻：真卡上等完之后第一份读数仍含那笔缓冲（E174 推翻条件同一句，`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:379` 那一条）；或模型再找到一个 `room > safe_room` 的种子。

### K3-2　卡放得下却报本次未跑、判红、整趟弃卡、或因等设备压并发

1. 等设备压并发：不压。`Wait` 只阻塞丢借据的那一条线程；它在 `poll` 返回之后才拿账的锁（`take_in_the_wait_before_returning` 在第 1280 行，在 `poll` 之后），等的时候不持账的锁。等的范围是这张卡的设备上到调用那一刻为止所有线程已提交的活（转述：wgpu-core 30.0.1 `src/device/resource.rs` 第 856–862 行 `Wait { submission_index: None }` 等 `last_successful_submission_index`），E174 第二段同一设备另一条线程不断提交 400 毫秒的活时等 0.3 到 0.69 秒（`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:15`）。产品路径里借据是判器都丢掉之后才还，设备上没有剩下的活。**一致**。
2. 等超时之后的永久少借：见 K1-2，累加到整张卡借不出时每一笔借都是本次未跑。**冲突**（不在用户已认之列）。
3. 设备丢了之后的少给窗口：见 K1-3 的 `k3_2` 第一行 —— 一张空闲 13800 MiB 的卡，账给 0，物理上还能再借 11840 MiB；这段从重起好持续到设备丢了那一刻活着的借据（含触发重起的那一笔）都还了。借 ≤ 11840 MiB 的新借据在这段里被报本次未跑。条款没写这一格，方向是少借；窗口长度 = 触发重起那一笔借据的寿命（产品路径里大概是第 ① 段那一轮，推的）。**规则没说**。
4. 展开中遇设备致命错 abort：新旧同（K1-1）。

判定：1 **一致**；2 **冲突**；3 **规则没说**。
什么会推翻：2 —— 真卡上 Wait 在这个仓的任何用法下都不超时、不出错（要长时间观测）；3 —— 把触发重起那一笔不放进「设备丢了那一刻活着的」集合后，`k3_2` 第一行的 `room` 不再是 0。（这一条是我的建议方向，没试。）

## K4　测试判别力

应该是：这一批每一处改动「改回去」至少有一条测试红；没有的要写明为什么抽不出。

1. harness 档：31/31 红（K2-3）。
2. 我补的 9 条探针（在整个 `gpu_memory_ledger` 模块上跑，`probes.json`）：

```
$ cat probe-w*.log | grep '^ROW' | awk '{print $3}' | sort | uniq -c
      2 GREEN(survived)
      7 RED(caught)
```

活下来的两条：
- **P5**：第二次丢设备时（已在 Remain 状态）不重新记账（`prepare_the_driver_share_before_starting_the_device` 里把 Remain 移出「DeviceStarted」那一支）。lib 全模块 94 个测试照绿；我的模型红（144/20000）。含义：Remain 状态下再丢一次设备，旧一代没出集合、新一代的借据不进集合，之后集合清空时回到「照读数算」而卡上还有第二代旧设备 —— 多给。真代码上 0 个种子红（K3-1），是测试缺这一格，不是代码错。
- **P3**：重起时没等成的借据从 `leases` 里去掉了，却不同时从「设备丢了那一刻活着的」集合里去掉。lib 全模块照绿；我的 `k1_3b` 红：`room=26840` 对 `safe=27440`（少 600 = 驱动已占的那一份，一直回不到「照读数算」）。
这两条的缺口各配一个用例就补上（模型里 `k3_1` 与 `k1_3b` 就是）。

3. checker 档接线的「改回去」（这一批第三节要求「抽不出的写明为什么」；实现员报告 §六 (b) 3、4 写了其余参数与 `cards_used` 抽不出）。`emulate_scans.py` 在冻结副本 `crash_amplification.rs` 上复刻两条扫源码用例，注入下列「改回去」，两条扫描都照绿：

```
baseline (frozen): scan1_pass=True scan2_pass=True
REVERT-A 一轮收尾不问记住的叫停（常量 NotRequested）: scan1_pass=True scan2_pass=True
REVERT-B 起下一张卡不看叫停（常量 NotRequested）: scan1_pass=True scan2_pass=True
REVERT-C 卡线程判块那一圈不看叫停: scan1_pass=True scan2_pass=True
REVERT-D 整趟收尾 stop_is_requested 闭包改成永远 false: scan1_pass=True scan2_pass=True
REVERT-E 打头一句 OOM 的那一支把 println 换成 eprintln: scan1_pass=True scan2_pass=True
```

对应的实际位置：A 是「let stop = stop_request.stop_of_the_run();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3116`），B 是「work.stop_of_the_cards(),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2476`），C 是「&|| work.stop_of_the_cards(),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2303`），E 是扫描里这一句：「&& arm_body.contains("println!(\"{not_run_line}\");"),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4659`），它也是 `eprintln!("{not_run_line}");` 的子串。除了 `crates/singlefs-harness/tests/gpu_card_rounds_remember_a_stop_request_removed_mid_run.rs`（那是「照次序抄成不碰卡的一份」，不碰 checker 档这几行），仓里没有不要卡的测试引用这几处：`grep -rn 'next_card_to_start\|work_caught_noting\|stop_of_the_cards\|take_in_the_wait' --include=*.rs crates/singlefs-checker-tier` 只命中 `src/`，没命中 `tests/`。能让 A、B、C 改回去变红的只会是要卡的用例（例如 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里按叫停领五块的那一条；推的，没跑）；E 改成 `eprintln!` 之后那一行改走 stderr（门禁读不读 stderr 我没查）。
4. `GpuCardLease::drop` 本身（等不等、等多久、结局记没记）：「self.booking.take_in_the_wait_before_returning(wait);」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1280`）、超时常量（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:27`）、`catch_unwind` 与 `resume_unwind` 都没有不要卡的测试；去掉第 1280 行那句，所有等失败的借据都被当成等完销账 —— 照绿（只有 K1-2 那个多给方向会红，没有测试造得出等失败）。
5. 碰卡用例能不能抓住「改回 Poll」（变异表 3370 行）：用例先「lease.card().queue().submit([encoder.finish()]);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:83`），紧接着读一次 `nvidia-smi`（第 84 行，起一个子进程），然后才「drop(buffers);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card.rs:89`）。读一次 `nvidia-smi` 约 0.05 秒（E174 修订 1 自己的数，`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:25`：「各约 0.05 秒」），2 GiB 的清零要多久我没量，推的是早做完了、丢借据时活已经做完。E174 量的不等待臂在这一档：「活已经做完的档（已做完、逐趟）下一次读数不含」（`.claude/kb/experiments/174-借据丢掉时等待与显存回还.md:130`）。所以改回 `Poll` 时这条用例的最后一个断言很可能照绿 —— 这是推的，要卡才能定（见末尾「要主 agent 跑的命令」）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 一致 | 建过缓冲的销账路都先等（唯一建 `GpuMemoryBooking` 的是卡池）；起设备失败那一路没缓冲不等；展开中设备致命错会 abort（新旧同）；借据先于缓冲丢的顺序前提无测试 |
| K1-2 | 不多给 一致 / 「一直少借」的界 冲突 | 等失败的借据没有设备丢失就永不销，累加到整张卡借不出（模型：第 7 笔 4000 起 refused，卡上 `free_true=29400`）；20 万种子 0 次多给 |
| K1-3 | 多给 一致 / 少给 规则没说 | 暂时一段：触发重起那一笔活着时少给全部已建缓冲加驱动（空闲 13800 的卡账给 0、物理还有 11840）；永远一段：设备丢了那一刻活着的借据等失败 → 永远 Remain（少 10600），出口只有下一次丢设备 |
| K1-4 | 一致 | 旧字段、往下压、重起往上加三处都没了，全仓 grep 零命中 |
| K2-1 | 一致 | 读叫停文件只有一处；两层包装 true 只增；24000 段两样结局都 0（实现员自己的 6000 段也 0）；用例里的接线是抄的 |
| K2-2 | 一致 / 边界一格 规则没说 | 该打的时候打；OOM 没让任何块丢掉而因叫停收尾时也打一行本次未跑（模型 `k2_2`），按判决字面该打、按「没跑完」读是多一条 |
| K2-3 | 一致 | harness 档 31/31 红且红在点名的测试；checker 档 3 条复刻红、1 条读码红、1 条（Poll）见 K4 |
| K3-1 | 一致 | 模型 200000 种子 0 次 `room > safe_room`，判官 4/6 注入红；前提是 E174 的读数与借据先于缓冲丢 |
| K3-2 | 压并发 一致 / 等超时累加 冲突 / 重起窗口少给 规则没说 | 等设备只阻塞丢借据的线程、不持账的锁 |
| K4 | 冲突 | harness 档 31/31 红；探针 9 条 2 条活（P3、P5）；checker 档接线改回去（A–E 五形）两条扫描都照绿；`drop` 里等不等、记没记、超时无测试；碰卡用例很可能抓不住改回 Poll（推的） |

## 没做什么

- 没跑 checker 档任何测试、没跑带 gpu 特性的测试、没起 GPU 设备；没编 checker 档。checker 档那 5 行变异、整行替换里 checker 档那一部分（替换 9 行全在 harness 档，已跑）里要卡的一行（Poll）我没能判。
- 没造真卡上的 Wait 超时、Wait 时设备出错、真丢设备；K1-2、K1-3 的触发频率因此不知道，只量了「触发之后账怎么走」。
- 模型是公开接口加假卡：不含 wgpu 的分配器按块拿（`min(occ, promised)` 之外的多占）、活着的借据缓冲缩回驱动的惰性释放、nvidia-smi 读数与建缓冲的同刻缝；这几样是前面几批的口径，不是这一批新增，我没有再攻。
- 没替主 agent采纳或出判决；「规则没说」的几格只写了现象与数。
- 没核第二节「实现员自定」第 4 条的叫停标记「只在 `run_from_environment` 建一个」之外的调用方（`grep` 只有这一个调用点，别的 crate 里没有）；第 6 条（`after_the_round` 其余参数与 `cards_used` 抽不出）我只读了实现员报告 §六 (b) 3、4，没独立判「抽不出」是否成立，K4 第 3 点只写了抽不出的后果。

## 要主 agent 跑的命令（我复核不了的）

1. 变异表 3370 行（改回 Poll）在真卡上有没有红：把 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 的「device.poll(wgpu::PollType::Wait {」（第 1264 行）那一段换成 `device.poll(wgpu::PollType::Poll)`，在仓根跑 `SINGLEFS_HEAVY_TESTS=commit bash scripts/crash/gpu-quiesce-run.sh -- bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 8 cargo test --offline -p singlefs-checker-tier --features gpu --test gpu_unit_checks_a_returned_card_lease_leaves_no_buffer_on_the_card -- --nocapture`，看 `GPU_CARD_LEASE_RETURNED … occupied_after_the_lease_was_returned=…` 一行与 `test result`；照绿就是 K4 第 5 点坐实（绿了说明这条用例分不出 Wait 与 Poll，要把清零换成在飞的活再提交才分得出）。
2. 模型复跑（在冻结副本的拷贝上，harness 档，不起卡）：把 `research/prompts/m3-batch10z-f-code-r1-sonnet-model/sonnet_g2_model.rs` 拷到拷贝的 `crates/singlefs-harness/tests/`，`SEEDS=200000 CARGO_TARGET_DIR=<目录> bash scripts/hosts/run-with-memory-cap.sh 1G bash scripts/hosts/capped.sh 4 cargo test --offline -p singlefs-harness --test sonnet_g2_model -- --nocapture`，看 `K3-1 random: seeds=200000 violations=0` 与 K1-2、K1-3、K3-2、K2-2 那几行。

## 草稿产物与删除

- 模型、脚本、探针、各条原始输出：`research/prompts/m3-batch10z-f-code-r1-sonnet-model/`（`README.txt` 写了每个文件做什么；`logs/` 21 份）。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1-sonnet/` 里保留脚本、日志、`rows.json`、`probes.json`（小文件，不删：报告引的输出出自它们）。
- 已删（`du -sh` 在删之前记的）：仓副本 `copy-a`、`copy-w0`、`copy-w1`、`copy-w2`、`copy-w3` 各 21M；编译目录 `target-a` 1.6G、`target-w0` 1.5G、`target-w1` 1.5G、`target-w2` 1.4G、`target-w3` 1.6G；`tiny`（rustc 小程序目录）4.4M。都在 `~/.local/share/singlefs/work/m3-batch10z-f-code-r1-sonnet/` 下。

## 引文核对

（交回前跑 `python3 scripts/agents/cite-check.py … --root <冻结副本> --root .`，末行贴在下面）

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-batch10z-f-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree
  ✓ 核了 34 处引文，对上 28 处，没判 6 处（逐处列在上面）
$ python3 scripts/agents/cite-check.py research/prompts/m3-batch10z-f-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-batch10z-f-code-r1/tree --root .
  ✓ 核了 34 处引文，对上 34 处，没判 0 处（逐处列在上面）
```

只带冻结副本为根时，没判的 6 处引的是仓根下的文件（判决 `research/prompts/m3-batch10z-e-code-r1-main-verification.md` 第 37、38、39 行与 E174 页第 25、130 行等，不在冻结副本里）；再加仓根为第二个根后全部对上。
