# 第 10z 批 a 块（显存账）代码轮第一轮：正推腿报告（K1、K2、K3）

轮名 m3-batch10z-a-code-r1，立场正推。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch10z-a-code-r1/tree/` 里的 `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/mutations.tsv`；下文引代码一律写这份副本的路径与行号（我的草稿副本与它逐字节相同：四份 `cmp` 都无输出）。
对照的是上一轮判决 `research/prompts/m3-batch10y-code-r1-main-verification.md` 第四节第 1 条那四项：
「K1-1 不走卡池设备的登记窗口、D − x 多给；」（`research/prompts/m3-batch10y-code-r1-main-verification.md:48`）
「K1-3 单元校验和对拍几条线程派活包住；Out of Memory 计数按线程认，不按整台设备；设备丢了重起时重新登记驱动那一份；」（`research/prompts/m3-batch10y-code-r1-main-verification.md:49`）
「K2-2 P1（已销账的短借据多扣）、P2（一张卡 OOM 不停整趟）；」（`research/prompts/m3-batch10y-code-r1-main-verification.md:50`）
「K3 补用例与变异：正推那 6 条照绿的、7 处没覆盖的，攻方那两处接线。」（`research/prompts/m3-batch10y-code-r1-main-verification.md:51`）
GPU 口径的总纲在 D24：三条硬约束的第 3 条是「3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。**」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`）；下面 K1-2 里「单元校验和对拍用例不读新字段」一条与它相关。用户原话（只防 OOM、不压并发）取自正文第一节，不在 kb 里，这里不当 kb 条款引。

## 〇、结论先行

| 项 | 判 |
|---|---|
| K1-1 先记后起 | 做到（两个入口都经 `start_the_card`，先记后起，起不来照样记着） |
| K1-1 乙附、丙形不再多给 D − x | 做到（模型上改前多给 1960、改后 0）；但这一批换进来的「见到的那一份」引进一形新的多给（S3），和一批新的少借（S1、S5、S7） |
| K1-3 每条核对线程各自接 OOM | 做到（读码），但单元校验和对拍的用例（`the_pipeline_with_gpu_unit_checks_…`）不读新字段：OOM 时不报本次未跑，部分判完也照绿 |
| K1-3 按线程认 OOM | 做到；残差两处（编内核按「此刻在编的活」认、设备丢了那一支宽松）与实现员自认的一致，没找到第三种 |
| K1-3 设备丢了重起 | 做到一半：头一次被拒是对的；**被拒之后账已改成「驱动那一份许出去没占上卡」，之后再进 `prepare` 的（被拒之前已借到的并发用例）不再判**（S6：第 2、3 次直接 Ok，运行时 `OutOfGpuMemory` 兜底）；重起之后账少给 2560 MiB 且压不下去（S4、S9） |
| K2-2 P1 | 实现员那组数做到（S10：改前第 10 笔 9550，改后 19550）；但压法靠「见到的那一份」，它只升不降，有三种形状压偏（S1 少借、S5 少借、S3 多借） |
| K2-2 P2 | 做到「一张卡 OOM 只停那一张、块交回队里」；队尾一形漏：别的卡先收工、OOM 的卡后交回，那一块没人领，前面判完的块也因按块序写库而不入库（小模型 `p2probe`） |
| K2-1 五条取法 | 乙附、重起 +D、P2 残差：没有多给，只有少借；P1 假设（见到的不变小）：有多给的形状；OOM 归属残差：与账无关，是认错原因的问题 |
| K3 | 6 条 + 7 处里能不要卡的都补了；实现员 19 行 harness 档追加变异我逐行重放，全红；我另造 12 条，9 条照绿（含「销账时见到的那一份不减」，与 S3 同向）；没补行的几处里至少 5 处不用真卡也能补 |

## 一、怎么量的

- 真账：`GpuMemoryLedger::new` 加公开函数 `book_on_the_card`、`record_the_device_started_on_the_card`、`card_rooms_mebibytes_now`、`prepare_the_driver_share_before_starting_the_device`；读数函数由我的「物理卡」给：这个进程占的 x = 驱动实占 + 活着借据已建上的 + 销了账卡上还没还的；空闲 F = 空卡能用 32 110 − 外人 − x。**真能再借（保证值）** = F − 活着借据还没建的 − 驱动还要长的；账多给 = 账给的 − 真能再借。
- 对照：改前 = 冻结副本 `patch -R` 掉实现员的 `crates.patch`（`tree-old`），同一份探针在两份树上各跑一遍，编译目录各用各的（头一次两树共用一个目录，cargo 把改前的当新鲜、跑出两份一样的数，我发现后弃掉，重编重跑）。
- 命令（工作目录各树根）：`CARGO_TARGET_DIR=<草稿目录>/target[-old] nice -n 19 bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 15 cargo test --offline -p singlefs-harness --test sonnet_probe_common --test sonnet_probe_new_only -- --nocapture --test-threads 1`；退出码都是 0，包装没有 250–254 的结局。开跑前 `ps` 看到别的会话的 `cargo test -p singlefs-harness -- --ignored …`（pid 800410 一族），没有 qemu、vm-bench、e152、fio，我加 `nice -n 19` 照跑。
- 输出原样在 `research/prompts/m3-batch10z-a-code-r1-sonnet-model/logs/final-new.log`（sha256 030eac62…fa4f）、`final-old.log`（sha256 efe43ac2…cb26），探针源码在 `…/probes/`。
- 没有占卡、没有起 GPU 设备；`nvidia-smi -q -d MEMORY,PIDS` 只读跑过一次（`…/logs/nvidia-smi-q-memory-pids.txt`：驱动 595.84，5090 总 32607、保留 497、空闲 32110，进程表 0 行）。
- 不碰重型：没有跑 checker 档的任何测试；checker 档里的几处只读码，标「读码」。

## 二、K1：第 10y 批判决第四节第 1 条各项做到没有

### K1-1（问题 1）：不走卡池的设备先记后起；乙附、丙形下账给的量

**先记后起：做到。**
- 两个不走卡池的入口（`usable_gpu_cards`、`gpu_cards_by_priority`）起设备都传 `OutsideTheCardPool`：「start_the_card(discovered, DeviceOwnerOnTheCard::OutsideTheCardPool).ok()」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:379`），408 行同一句。`started_with_its_owner_recorded` 对卡池的设备直接起：「DeviceOwnerOnTheCard::TheCardPool => start(),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:175`），不归卡池的先插总线号再起：「.insert(bus.to_string());」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1492`）。
- 读与记的先后对：读数先取回报告「let report = run_the_report()?;」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:812`），解析时才问有没有不走账的设备「has_devices_outside_the_books_on_the_bus(bus)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:786`）。报告里若已经有那个设备的占用，它必在记号插上之后才起，所以解析时必看得见记号：乙形的窗口关了（读码）。
- 起不来照样记着：「起不来的照样记着（分不出，只会少借）。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1479`）；记号只增不减。

**会不会让卡池在那条总线上一直少借或多借：只会少借，不会多借。**
- 记号一插，这条总线上的读数里这个进程一律是 `AlsoOutsideTheBooks`，进这一句：「OccupiedByThisProcess::NotListed | OccupiedByThisProcess::AlsoOutsideTheBooks { .. } => 0,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:199`）。占着的当 0：借着的 B 与驱动那一份 D 整份从空闲里扣，而空闲本来就不含它们已经建上的，等于数了两遍。同一个 0 臂的数（S7，设备起好、报告没列这个进程，借 10 000 建满）：改后账给 6 990、真能再借 19 550，少 12 560（= D + 已建上的）；改前账给 9 550，少 10 000。S7 造的是 `NotListed`，`AlsoOutsideTheBooks` 走同一个 0 臂（读码）。
- 多借的形一个都没有：同一台物理卡模型上，驱动实占 600 < D 的 400 条种子 × 120 步，改前 5 661 次多借、最多多 1 960（= D − x，正是攻方乙附、丙形那一形），改后 0 次。
```
final-old.log  FUZZ monotone xd=600 collect%=50: Stats { samples: 14185, over_count: 5661, over_max: 1960, under_count: 4067, …
final-new.log  FUZZ monotone xd=600 collect%=50: Stats { samples: 13899, over_count: 0, over_max: 0, under_count: 11271, under_max: 11195, …
```
- 永久性：记号只增不减，起不来也记。它只在「同一个进程里先走不归卡池的路径、后起了卡池」时碰得到：`gpu_cards_in_use` 里起过卡池才照池里的卡用，没起过才自己起（`crates/singlefs-checker-tier/src/crash_amplification.rs:5682`，自己起的两处在 5707 与 5719 行）；GPU 用例不许直接调那两个入口，有扫源码用例管。是否真有一个测试二进制先走产品路径、后起卡池，我没逐个二进制查。
- 推翻它的现象：在改后树上造出任何一个读数序列，使账给的量大于物理卡模型的「真能再借」且没有设备丢失；或查到一个调用方在同一进程里让不走卡池的设备与卡池设备同一张卡共存且卡池还在借。

**乙附、丙形两种取法下账给的量是不是都不大于真能用：这两形本身是，但这一批换进来的「见到的那一份」引进了新的多给，见 K1-3（问题 3）与 K2-1。**

### K1-3（问题 2）：单元校验和对拍几条线程里任一条报 OOM

**每条核对线程各自接：做到（读码；真卡上的 OOM 造不出，没有注入点）。**
- 每条线程的一段都经 `judge_one_piece_catching_out_of_memory_on_the_card`，里面 `on_the_card_catching_out_of_memory` 开头起一段活、起一个错误范围、`catch_unwind` 接住工作：「std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2158`）。没报 OOM 的 panic 照抛：「WorkOnTheCard::Panicked(panic) => std::panic::resume_unwind(panic),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2182`）。
- 一段报了：「if let Some(reason) = reason_the_gpu_memory_was_short_in_one_of_the_pieces(&judged) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1445`）→ 记原因、`break`，在「store.store_verdict_block(…)」之前，这一块不入库、不再领下一块，之前判完的块在库里。产品路径 `run_from_environment` 读这个字段打本次未跑：「let gpu_memory_was_short = match &checking.stopped_because_the_gpu_memory_was_short」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5375`）。

**一形漏：对拍用例不读这个字段。**
- `check_blocks_where_until` 现在对「卡上 OOM」不再 panic，而是在 `CheckingTally` 里带一个新字段交回。仓里调用 `check_recorded_blocks` 而给卡的，除产品路径外只有一处：`the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_healthy_flow`：「let tally = check_recorded_blocks(」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:269`），之后只断言「assert!(tally.states_checked > 0);」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:281`）与不对拍、不红。`grep -c stopped_because` 这份测试文件 0 次。
- 后果（读码，没造出真 OOM）：一段 OOM 时，这一块不入库、之后的块不领，`tally.states_checked` 只是少；**第一块之后才 OOM 时 `> 0`、没有红、没有不对拍，用例照绿，没有「本次未跑」那一行，也不是红**；第一块就 OOM 时断言 `> 0` 红，没有显存不够的原因。改前这一形是 panic 红。D24 那一条要的是 GPU 路径的结果不能静默当作复核过了：「3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。**」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`）。
- 推翻它的现象：该用例在别处另读 `stopped_because_the_gpu_memory_was_short`（grep 零命中），或 `states_checked` 被另一条断言钉成总状态数（文件里没有）。

**按线程认之后，别的线程的 OOM 会不会仍把这条线程真正的 panic 说成显存不够；或反过来：有两形，都是实现员自认的残差，我没找到第三形。**
- 记在哪段活上：这条线程上有活就算那一段：「Some(work) => vec![work],」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1600`）；没有活的线程（判器内核另起的编译线程）就算此刻这个设备上所有在编内核的活：`None => self` 一支里的 `.keys()` 取全部（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1601-1607`）。认的时候只看算在这段活上的：「.find(|report| report.works_it_explains.contains(&work))」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1681`）。
- 形一（多给显存不够的名）：同一个设备上两段活各在编不同摘要的内核，其中一段编内核另起的线程报 OOM，两段活都被算上，另一段活自己的真 panic 因此被说成显存不够，判本次未跑而不是红。编内核按源码摘要分格（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:537`），产品路径只有一个摘要，所以多段同时编的只可能是共用一张卡的不同 GPU 用例；这一形没有用例（单测只造了一段活在编）。
- 形二：设备丢了那一支，这段活开始之后记过任何一次 OOM（别的活的也算）就说成显存不够：「(Some(lost), Some(report)) => {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1686`）。用例 `a_lost_device_explains_a_failure_only_with_an_out_of_memory_since_the_work_began` 钉的就是这个取法，不是漏。
- 反过来（真 OOM 被当 panic 判红）：要有一条既没有活、又不在编内核的线程记 OOM。GPU 相关文件里另起线程只有判器内核那三条编译线程（`crash_judge_dispatch.rs` 里一处 `std::thread::scope`），核对线程与卡线程都在活里。我没造出、读码也没找到第三形。
- `WorkOnTheCardOfThisThread` 丢掉时换回原来那一段：「WORK_ON_THE_CARD_OF_THIS_THREAD.with(|work| work.set(self.work_of_this_thread_before));」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1549`）；嵌套活没有用例，改成 `set(None)` 照绿（K3 的 N1）。
- 推翻它的现象：在真卡上让别的活的 OOM 与这段活的 assert 失败同时发生，并看到这段活的失败被说成显存不够，且两段活不在「同一个设备同时编内核」「设备丢了」这两形里。

**设备丢了重起，账算得对吗：被拒的那一次对，之后不对，两处。**
- 代码：丢了的设备要重起，先把账改成「驱动那一份许出去、没占上卡」、旧设备那一份记进销了账没还的、见到的作废，再读一次、放不放得下：「books[card_position].take_back_the_driver_share_of_a_lost_device(」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1297`），放不下判据「if capacity_mebibytes < booked.leases_mebibytes {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1310`）；被拒之后账已经改成 `RegisteredDeviceNotStarted`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:879`）。而这个状态下再来：「DriverShareOnTheCard::RegisteredDeviceNotStarted => Ok(()),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1295`），不再判。
- S6（改后树，原样）：借 10 000、起设备、建满，再把剩下的 19 550 全借走（还没建），设备丢了重起，连续三次 `prepare_the_driver_share_before_starting_the_device`：
```
S6: room before second lease 19550; restart attempts: first Err(NoCardHasRoomNow { need_mebibytes: 2560, card_rooms_mebibytes: [0] }); second Ok(()); third Ok(()); room now 0
```
  调用方先借后起：`lease_card` 先 `book_on_the_first_card_with_room`，借到了才 `card_lent_for_the_booking` 进 `started_card`；`started_card` 里 `started_or_start` 对丢了的设备每次来都重起：「|started: &GpuCard| started.device_condition().lost_reason().is_some(),」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1253`）。被拒之后**新来的借**用的是改过的账（驱动那一份记成许出去而没占上卡），借得到就说明同一个不等式放得下，再进 `prepare` 也过，不算漏。漏的是**被拒之前已经借到、借据还在手上的并发用例**：它们按丢设备之前的账（驱动已在卡上）借的，轮到它们进 `started_card` 时账已被头一个改掉，`prepare` 不再判，直接 `request_device`，而这时「借着的 > 容量」。兜底还在：起设备时显存不够判成 `OutOfGpuMemory`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:138`）报本次未跑，所以是「账少拦一道，运行时兜底」，不是无人兜底（读码，真起设备造不出）。
- S4、S9（改后树）：重起之后旧设备那一份还了、新设备起好、借据重新建上 10 000，账给的比真能再借的少 D，且读几次都不降：
```
S4 old_device_still_on_the_card_when_prepared=false: before_loss 19550 prepared Ok(()) after_restart reads 16990 16990 truth 19550 (under 2560)
S4 old_device_still_on_the_card_when_prepared=true: before_loss 19550 prepared Ok(()) after_restart reads 16990 16990 truth 19550 (under 2560)
S9: truth 19550; book 17000 -> NoCardHasRoomNow { need_mebibytes: 17000, card_rooms_mebibytes: [16990] }
```
  原因：重起时旧设备那一份 D 加进“销了账没还的”（`crates/singlefs-harness/src/gpu_memory_ledger.rs:882`）、见到的清零（`crates/singlefs-harness/src/gpu_memory_ledger.rs:883`），之后每读一次先按「实际占着 − 见到的」压它、再用压过的值去算这一次见到的（`crates/singlefs-harness/src/gpu_memory_ledger.rs:865`），见到的于是总比真占着的少那一份 D，压它的差永远是 D，压不下去。实现员报告第二节第 3 条写的「旧设备的显存真还了时，下一次读数照占着的把它压掉」在这个模型上不成立（对一次读数序列看，两次读数同值）。方向是少借，不是多借。
- 推翻它的现象：真卡上设备丢了重起之后，卡池读到的这个进程占着的里还留着旧设备那一份，使「见到的」从一开始就包含它（那样 R 会被压到 0）；我的模型假定旧设备的那一份在新设备起好时已经还了。

## 三、K1 问题 3 / K2-2：销了账的那一份压得对不对；P2 交回的块

**P1 那组数：做到。** 实现员那组（空卡 32 110、长借据 10 000、十笔 1 000 短借据各借、建、丢、马上还给驱动）我在两份树上各跑：
```
改后 S10 long built: ledger 19550 truth 19550; #1 ledger 19550 truth 19550; … #10 ledger 19550 truth 19550;
改前 S10 long built: ledger 19550 truth 19550; #1 ledger 18550 truth 19550; #2 ledger 17550 …; #10 ledger 9550 truth 19550;
```
**P1 那组数之外：压偏的三形，一形多给、两形少给。** 机制：销了账的那一份 R 每次读数压到“实际占着 − 见到的”（`crates/singlefs-harness/src/gpu_memory_ledger.rs:217`），见到的只在读数比它大时往上换（`crates/singlefs-harness/src/gpu_memory_ledger.rs:870`），换的时候把此刻全部借据号当成“那一刻活着的”（`crates/singlefs-harness/src/gpu_memory_ledger.rs:872`），借据销账时从见到的里减它整笔借的量（`crates/singlefs-harness/src/gpu_memory_ledger.rs:903`）。
- S1（少给，改前对、改后不对）：一笔借据借了、起设备、建了、丢了，卡上在第一次读数之前就还给驱动。第一次读数时 x − 见到的(0) 不小于它，R 保持原值，之后「见到的」按压偏的算，R 就是一个不动点：
```
S1 b1=2560: after drop, no leftover: ledger 26990 truth 29550     (改前 29550)
S1 b1=2560: L2 built 10000: ledger 16990 truth 19550              (改前 19550)
S8: truth 29550; book 27000 -> NoCardHasRoomNow { need_mebibytes: 27000, card_rooms_mebibytes: [26990] }   (改前 Ok)
S1b: 同一段但第一次读数时那一笔还在卡上：leftover still on card: ledger 19550 truth 19550；freed: 29550 truth 29550（对）
```
  即「销了账的那笔在被读数见到之前就还了」这一形；b1 取 1 000 少 1 000，取 ≥ 2 560 少 2 560。
- S5（少给，改前后一样）：一笔借据借了、没建、在「见到的」更新时已经在账上，后来没建就丢了：见到的被减去它整笔借的 5 000（实际卡上什么都没有），R 压不掉：`S5: U returned unbuilt: ledger 11550 truth 16550`，再读一次同值。改前同值（见 final-old.log），不是这一批引进的，但这一批的改法没管它。
- S3（多给，改前对、改后不对）：长借据建满被读到（见到的 = D + 10 000），之后它的每趟缓冲还给驱动（借据还活着，建上的变 0），同时一笔短借据销了账、卡上还留着 1 000：压 R 用的「x − 见到的」成了 0，R 被压成 0，账把那 1 000 当成长借据建上的：
```
S3: L1 idle (live, a=0), S leftover 1000 on card: ledger 19550 truth 18550 (> truth is an overlend)     (改前 18550)
```
  多给 = min(缓冲缩掉的量, 销了账没还的)。随机序列上（缓冲可缩）400 种子 × 120 步：改后多给 66 次、最多 4 437（collect 50%），34 次、最多 3 952（collect 90%）；改前 0 次、0 次。缓冲不缩（只增）时改后改前都是 0 次。缓冲会不会缩：「一次建的五个缓冲」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:440`）每调一次建五个、函数返回时丢，单元校验和对拍与 GPU 用例借的就是这一类，借据活着、缓冲反复建丢：缩的方向存在（卡上什么时候真还给驱动我没量）；判器的表与草稿区建在判器里（`crash_judge_dispatch.rs` 的 `tables_buffer`、`scratch_lanes`），判器活着时不丢，按读码不缩。
```
FUZZ shrink  xd=2560 collect%=50  新：over_count 66 over_max 4437   旧：over_count 0
FUZZ shrink  xd=2560 collect%=90  新：over_count 34 over_max 3952   旧：over_count 0
FUZZ 缓冲只增的各组（monotone、full-build、built-only-returns 共 10 组） 新：over_count 全 0   旧：全 0，只有 xd=600 一组 5661
```
- 少给的频度（模型相关，不当结论只当量级）：借据只在建过之后才丢、建满（`built-only-returns`），卡上随即还给驱动的概率 0 / 50 / 100%：少给的读数占比，改后 3525/9769、6775/11445、8619/12553，改前 1594/9878、3303/11586、4328/12677；卡上完全没有销了账没还的那些读数里仍少给的：改后 70/1075、731/1882、3506/4819，改前 94/1075、492/1896、1929/4808。最多少给的量都在 10 560–15 060 MiB（≈ 整个进程在这张卡上的占用）。这个模型的取值（借多少、建多少、什么时候还）是我定的，不是量出来的。
- 多张卡：每张卡的账各记各的，读数一张一张压：「for (books_of_the_card, reading) in books.iter_mut().zip(&readings) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:992`），一张卡上的压法不看别的卡，所以多张卡就是几份单卡序列，没有跨卡的压偏（读码）。
- 推翻它的现象：真卡上的 `nvidia-smi -q` 序列里，销了账的缓冲在借据丢掉到下一次读数之间总是还没还给驱动（那样 S1 一形不出现）；或判器与对拍的借据在活着时缓冲从不缩（那样 S3 一形不出现，但我的 S3 要的是 `crc32c_of_ranges` 那一类借据在借据活着时每次调用建丢缓冲）。

**P2（一张卡 OOM 只停那一张、块交回队里）：做到，队尾一形漏。**
- 判一块时报 OOM：交回再退出，不再置整趟的停：「work.blocks.give_back(pending_index);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2263`）；交回的先给下一张来领的卡：「fn take_a_block(&self) -> Option<usize> {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2065`）里先取交回里最小的。建判器时报 OOM：下一张卡照起（`crates/singlefs-checker-tier/src/crash_amplification.rs:2217`）。收工后别的卡判完了照常交回：「if blocks_stored >= pending_blocks {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2434`）。
- 会不会判两遍：不会。交回的块只在队里一份，`take_a_block` 取走就删（小模型 `a_given_back_block_is_taken_once`：`taken: Some(1) None`）。
- 漏判、判完没入库（队尾一形）：别的卡先领不到块（队空）退出，OOM 那张卡之后才交回，没人再来领；而写库按块序「while let Some(block) = waiting.remove(&received.blocks_stored) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2003`），后面的块先到就等前面那一块，前面那块没了，后面判完的块也留在 `waiting` 里不入库。`BlocksForTheCards` 是从冻结副本逐字抄出来的（`p2probe/src/blocks.rs`，std 里跑，不起卡）：
```
test tests::card_b_has_left_before_card_a_gives_the_block_back ... B asks again: None
any_left after give_back = true
blocks_stored = 0 of 2 (block 1 was judged but waits for block 0)
test tests::card_a_gives_the_block_back_before_card_b_asks_again ... B asks again after give_back: Some(0)
```
  结局是本次未跑（块没判完），判完的块下一趟重判；窗口只在队尾（别的卡还有块在手时，交回的会被它们领走）。不是判错，是少判。实现员第二节第 5 条自认的就是这个，这里补上「判完的后面的块也因按块序写库而不入库」。
- 推翻它的现象：真卡上队尾 OOM 时别的卡领了交回的块（比如交回比别的卡的最后一次领取早）。

## 四、K2：只防 OOM、不压并发两头

### K2-1（问题 1）：实现员自定的五条取法，有没有一形让账给的量大于此刻真能用的

| 取法（实现员报告第二节） | 有没有多给的形 | 依据 |
|---|---|---|
| 1 乙附：没列、分不出时驱动实占按 0 算 | 没有；只多扣 | S7：设备起好、报告没列这个进程，借 10 000 建满，改后账给 6 990 / 真 19 550（少 12 560），改前 9 550（少 10 000）；`xd=600` 的 400 条随机序列多给 0 次（改前 5 661 次、最多 1 960）。代价比报告写的「多扣一份 D」看起来的更具体：改后 = D + 已建上的借据，多出改前的正是 D |
| 2 P1 的假设：见到的那一份在那几笔借据销账之前不变小 | **有**：见到的那一份在借据活着、缓冲反复建丢时会变小 | S3：账给 19 550、真 18 550，多 1 000；随机序列（缓冲可缩）多给 66 次、最多 4 437，改前 0 次。同一条假设还让 S1、S5 少给（见 K1 问题 3）。多给上界 = min(缓冲缩掉的量, 销了账没还的)，没有越过现读的空闲（`seen` 只用来压 R，许出去的封顶不变）。 |
| 3 重起时旧设备那一份记进销了账没还的（+D） | 没有；少借 D 且不降 | S4、S9（见 K1 问题 2） |
| 4 OOM 归属的残差 | 与账无关，不是账给的量；是把哪段活的失败说成显存不够 | 两形见 K1 问题 2；我没找到第三形 |
| 5 P2 交回的块在别的卡都收工后没人领 | 与账无关；少判不是多判 | `p2probe` |

另外两个我顺带量到的、不在实现员自列的五条里：
- S6 重起被拒之后，已经借到的并发用例不再过 `prepare` 的闸（K1 问题 2）：账少拦一道，运行时 `OutOfGpuMemory` 兜底。
- 乙附 0 臂对「读数没列这个进程」一律成立，包括起设备那一刻（驱动已占、报告还没列）：只会少借。

**推翻**：在改后树的真账上造出一条不靠缓冲缩小的序列，使账给的量大于「真能再借」。我在缓冲只增的 10 组随机序列（每组 400 种子 × 120 步，共约 13 万个读数）上没造出。

### K2-2（问题 2）：有没有一形卡放得下却报本次未跑、判红，或者整趟弃卡

- **少借形造成的本次未跑（有）**：同一台卡、真空闲放得下，账不给：S8 真能再借 29 550，要 27 000 的一笔借不到：`NoCardHasRoomNow { need_mebibytes: 27000, card_rooms_mebibytes: [26990] }`（改前 Ok）；S9 真能再借 19 550，要 17 000 的一笔借不到（16 990）。S1、S4、S5、S7 是这样的形。借不到的结局是 `NoRoomNow`，产品路径与 GPU 用例都报「本次未跑」，不判红、不整趟弃卡。
- **判红（没有）**：红只出在「要的比空着时能借的都大」：`need_mebibytes > largest_ceiling_mebibytes` 才 `NeverFits`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:276`），上界取 `reading.total_less_reserved_mebibytes`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:249`），不看借着的、不看销了账没还的，少借形碰不到它。
- **整趟弃卡（改后没有这一批引进的）**：一张卡 OOM 只停那一张；起判器时 OOM 照起下一张卡；整趟停只在卡拒判（「判不了」）与写库失败时置 `stop`，这一批没动。
- **队尾 P2 一形**（K1 问题 3）：别的卡有地方、却因先收工而报本次未跑。
- 推翻：真卡上这些形下账给的量与真能再借吻合。

## 五、K3：测试判别力

### 逐项：新加用例与变异行能不能拦住这一批每一处被改回去
- **实现员的 harness 档 19 行追加变异，我在副本上逐行重放（`mut/replay_tsv.py`，行号 3202–3218、3227、3228，每行只跑点名的那一条用例）：19 行全红**，每行红 1 条。基线：改后树 `cargo test -p singlefs-harness --lib` 140 passed（我用 N1、N5、N9 三次整跑 `--lib` 都是 `140 passed; 0 failed; 1 ignored`）。
- 6 条正推照绿的（X16、X17、X18、X19、X4、X2）：行 3213–3218 各一行，重放全红。
- 7 处没覆盖的：
  1. 登记不走卡池设备那一段：harness 行 + checker 行 3219 拦「起之前不记」；两处入口的实参 `OutsideTheCardPool`（`gpu_unit_checks.rs:379`、`408`）改成 `TheCardPool` 没有行，照绿。
  2. 读数器接线：checker 行 3220。
  3. 编管线 OOM 范围：要真卡，没补，认。
  4. 设备丢失回调、`lost_reason().is_some()`：没补。
  5. `print_the_not_run_line_or_refuse` 起设备显存不够：行 3226；`card_lent_for_the_booking` 那一支没补。
  6. 借之前三臂：行 3225。
  7. 空着时带几个状态：行 3224。
- 另有 checker 档 8 行我没跑（重型，归主 agent）。

### 我另造的 12 条 harness 档变异：9 条照绿
对照 K0（实现员行 004 的改法，`.min(mebibytes)`）红，说明我的跑法会红。过滤 `gpu_memory_ledger` 跑；N1、N5、N9 另整跑 `--lib` 仍 `140 passed`。

| 变异（改回去的那一形） | 结果 |
|---|---|
| N1 `WorkOnTheCardOfThisThread` 丢掉时换成 `None`（不回原来那一段） | **照绿** |
| N2 编内核计数 `still_compiling <= 1` 改 `<= 2` | **照绿** |
| N3 编内核计数 `+= 1` 改 `= 1` | **照绿** |
| N4 见到的更新条件 `>` 改 `!=` | **照绿** |
| N5 那一刻活着的借据号不记（`BTreeSet::new()`） | **照绿** |
| N6 那一刻活着的借据号累加不替换 | **照绿** |
| N7 重起不作废早先见到的数 | 红（2 条） |
| N8 重起不清那一刻活着的借据号 | **照绿** |
| N9 借据销账时见到的那一份不减（`saturating_sub(0)`） | **照绿** |
| N10 重起放得下判据 `<` 改 `<=` | **照绿** |
| N11 占着的里属于许出去的不封顶 | 红（2 条） |
| N12 见到的那一份不扣销了账的 | 红（3 条） |

等价性：N4 在读数变小时与原文不同；N10 在刚好放得下时不同；N1 在嵌套活时不同；N2、N3 在同一段活同时两处编内核时不同；N5、N9 在「那一刻活着的借据销账」时与原文不同，方向是多给（与 S3 同向）。都不是等价变异，只是没有用例走到。
其中 N5、N9 最要紧：行 3206 只钉「那一刻之后借的销账不减」，**没有一条用例钉「那一刻活着的借据销账要减」**，S3 一形的机制就在这条边上。

### 没补行的几处里，不用真卡也能补的
1. 两个入口的实参：仓里已有扫源码用例（`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs`）的同一类，数 `start_the_card(` 调用与第二实参，不要卡。
2. 对拍用例不读 `stopped_because_the_gpu_memory_was_short`：同样可以扫源码（凡调 `check_recorded_blocks`、`check_selected_blocks` 且给卡的都要读那个字段），或把 `assert!(tally.states_checked > 0)` 改成与总状态数相等。
3. `lost_reason().is_some()` 与设备丢失回调：回调体可抽成取 `wgpu::DeviceLostReason`（公开枚举，`wgpu-types-30.0.1/src/lib.rs:554`）与消息的具名函数，用例造 `DeviceConditionOfACard`；`is_some()` 那一句可抽成 `DeviceConditionOfACard` 的方法放 harness 档。
4. 没接住的错误 `handle_an_uncaptured_wgpu_error`：`wgpu::Error::OutOfMemory { source }` 成员公开（`wgpu-30.0.1/src/api/device.rs:902-907`），checker 档不要卡就能喂。`failure_of_a_device_request` 的 `RequestDeviceError` 成员 `pub(crate)`（`device.rs:790-792`），造不了，要把字符串比较抽成取字符串的函数。
5. `check_blocks_where_until` 的「一段报了就 break、不入库」：它只对 `UnitChecksumsOn` 分派，给一个判一段的闭包参数就能用假段驱动，不要卡；今天只有取第一个原因的纯函数有用例，`break` 与不 `store` 没有。
6. 卡线程判一块报 OOM 的 `give_back` + `return`：抽成对 `BlocksForTheCards` 的一个函数即可；今天 `give_back` 那一行换成空语句，无用例红。
7. 重起被拒之后再来：harness 档用例（我的 S6 三次 `prepare`）可直接补，改前 / 改后都不要卡。
这几条是读码与取 wgpu 源码得的可行性，我没有在副本里把它们写出来跑。

推翻 K3 的现象：这 9 条在主 agent 的整 `--lib` 跑里有红的；或 N5、N9 被某条我没跑到的用例（例如别的包）钉住。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1（问题 1）先记后起 | 做到 | 两入口先记总线号再起，起不来照样记；只会少借，多借 0 次（xd=600：改前 5 661 次、改后 0） |
| K1-1 乙附、丙形账给的量 ≤ 真能用 | 做到但有一形漏 | 这两形本身不再多给 D − x；换进来的「见到的那一份」在借据缓冲会缩时多给（S3，最多 4 437），同时少借变多（S1、S5、S7） |
| K1-3（问题 2）几条线程各接 OOM、之后的块结局 | 做到但有一形漏 | 产品路径对；对拍用例 `the_pipeline_with_gpu_unit_checks_…` 不读新字段，OOM 时可部分判完照绿（读码） |
| K1-3 按线程认 | 做到，残差两处 | 编内核按「此刻在编的活」认、设备丢了那一支宽松，都是自认的，没找到第三形 |
| K1-3 设备丢了重起，账算得对吗 | 做到但有一形漏 | 头一次被拒对；之后再进的并发用例不判（S6）；重起后少给 D 且压不下去（S4、S9） |
| K2-2（问题 3）P1 那组数 | 做到 | S10：改前第 10 笔 9 550、改后 19 550 |
| K2-2 P1 之外（长短交替、多卡） | 做到但有一形漏 | S1、S5 少给，S3 多给；多卡各卡独立（读码） |
| K2-2 P2 交回的块 | 做到但有一形漏 | 不判两遍；队尾别的卡先收工则交回的没人领、后面判完的块也不入库，报本次未跑（p2probe） |
| K2-1（问题 1）五条取法 | 一条有多给 | 乙附、重起 +D、P2 残差只少不多；P1 假设有多给（S3）；OOM 归属与账无关 |
| K2-1（问题 2）放得下却未跑、判红、弃卡 | 有未跑，无判红，无新的弃卡 | S8、S9 放得下被拒；红只出在 `NeverFits`，不受影响；整趟停只在卡拒判与写库失败 |
| K3 新用例与变异 | 做到但有一形漏 | 19 行重放全红；我造的 12 条里 9 条照绿（N1–N6、N8–N10），N5/N9 对应 S3 的边没人钉；至少 5 处没补行的不要真卡也能补 |

## 七、复核不了与没做什么

- 复核不了：真卡上的 OOM、真丢设备造不出（注入开关要加在 `crash_judge_dispatch.rs`，归删除线 V）；所以 K1-3 的 OOM 结局、对拍用例的部分判完照绿、S6 在真卡上起设备的后果，都是读码与账层面的数，没有真卡观测。「卡上缓冲什么时候还给驱动」没量，S1、S3 的发生频度取决于它。
- 模型的数是我定的取值（借多少、建多少、什么时候还），只当量级；S1、S3、S4、S5、S6、S7、S10 的数是确定序列的原样输出，不依赖随机。
- checker 档的 8 行变异、checker 档的全部测试没跑（重型，归主 agent）；除实现员的 19 行外 K3 只在 harness 档量。N4、N6、N8、N10、N2、N3 只跑了过滤 `gpu_memory_ledger` 的 54 条，没整跑 `--lib`；N1、N5、N9 整跑了。
- 没占卡、没起 GPU 设备；`nvidia-smi -q -d MEMORY,PIDS` 只读一次。
- 收到主 agent 的消息：线程上限由 15 改成 10。我在 K3 的 19 行重放跑完、写 K2 与 K3 两节时收到，之后没有再起编译或测试。
- 不替主 agent 采纳或出判决；不判别的腿的格（K4、G1、P1、P2 之外的）没碰。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-a-code-r1-sonnet/` 里的编译目录 `target`、`target-old`、`target-mut`、`target-p2` 与仓副本 `tree`、`tree-old`、`tree-mut`、`p2probe` 没删：判决与复核还要在副本上重跑，主 agent 复核完可整个删掉；入库的是 `research/prompts/m3-batch10z-a-code-r1-sonnet-model/`（探针源码、变异脚本、输出）。
