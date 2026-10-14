# m3-batch10x-code-r1 攻方腿报告（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G1 R1 R2 R3 R4 -->

立场：找反例。攻正文第三节 K1 到 K4，重点 K1-3（竞态兜底那条路漏没漏）与 K2（单进程白等、白红、多借、压一张卡）。
代码全部读冻结副本 `~/.local/share/singlefs/work/m3-batch10x-code-r1/tree/`；下面引的仓内路径在冻结副本与仓根逐字节相同（`cmp` 过，命令见「量的数」）。
模型只在攻方自己的副本上跑，副本上的数不算入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-batch10x-code-r1-opus-model/`：

| 文件 | sha256 |
|---|---|
| attack_m3_10x_gpu_ledger.rs | fd7cf4446cc05d07f4eb594f8060af45af2cefa2fb22af34d7d8aa1c75e5c534 |
| run-output.txt | 3ac027512c1c27404934e0fb990ff4d7f30ea32b5ab0886c62e548f66cb7f0f7 |
| nvidia-smi-during-main-agent-gpu-run.txt | e06f9c55a099f4448e55a004c31a7178fe54faf5539a1a53898166e95a856487 |
| nvidia-smi-query-timing.txt | fcb4526f6ad4e3178ca54e003d212f5a7d26a24702d40b8004fab9826a45239f |

复跑（harness 档，不上卡）：

```bash
S=<草稿目录>; rsync -a --exclude /target --exclude /.git --exclude /scripts/experiments/target ~/.local/share/singlefs/work/m3-batch10x-code-r1/tree/ $S/repo/
cp research/prompts/m3-batch10x-code-r1-opus-model/attack_m3_10x_gpu_ledger.rs $S/repo/crates/singlefs-harness/tests/
cd $S/repo && CARGO_TARGET_DIR=$S/target nice -n 19 bash <仓根>/scripts/hosts/run-with-memory-cap.sh 8G \
  bash <仓根>/scripts/hosts/capped.sh 10 cargo test --offline -p singlefs-harness --test attack_m3_10x_gpu_ledger -- --nocapture --test-threads 1
```

判结果看 `SUMMARY` 五行与 `test result: ok. 5 passed`。模型是纯算术、几毫秒跑完，没缩取样；不涉及崩溃状态（没有一段历史要恢复、要跑 checker），层 0 枚举那一条不适用，原型状态数 0。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 式子每一项 | **打中三形** | 甲：空闲与「这个进程占着的」分两次 `nvidia-smi` 读，中间这个进程建上卡的那一笔被算两遍（多借）；乙：这个进程不在账上的显存被当成「借了已经占上的」（多借）；丙：「空着时能借的」拿总量当 M，空卡读得到的空闲比总量少约 497 MiB，有一段要的量永远报本次未跑、到不了判红 |
| K1-2 跨进程删干净 | 没打中 | 账本文件、锁、判活、等满的代码都没了；只剩两处旧注释还写「等满」 |
| K1-3 竞态兜底 | **打中（第二条路漏四处）** | 起设备、判器内核在别的线程上编、只排派活计划那一趟试判、致命 Out of Memory 丢了设备之后同进程后面的活：四处撞了显存都是 panic 判红，wgpu Out of Memory 那条路接不住，只剩内存包装 245 接（接住时整条命令作废，不是这一趟本次未跑）。没找到判通过的形 |
| K2-1 单进程多借 | **打中两形（量过，模型）** | 甲形最坏多借 12288 MiB、乙形最坏 6783 MiB，都在一个进程里、不是两个进程的竞态 |
| K2-2 白等、白红、压一张卡 | **打中两形** | 白红：每张卡此刻放不下一个状态而空着时放得下，产品路径不去借、直接判红；`lease_card_waiting_for_room` 同进程兄弟借着就判红。白未跑：同进程兄弟借着当场报本次未跑（定义如此，记一笔）。压一张卡：没打中（先满一张是用户定的） |
| K3-1 核账判据 | 没打中 | 判器建好那一刻核账，分得出「借了」与「借了又在建判器之前还」；「建好之后、判完之前还」分不出（推的，没造出可达的改法） |
| K3-2 扫源码绕法 | 没攻 | 时间给了 K1-3、K2 |
| K3-3 变异行 | 部分 | 这一批的打中格里，产品路径「全是 0 就判红」那一处与 OOM 错误范围那几处没有变异行（`on_the_card_catching_out_of_memory` 没有用例已列在已知没做里）；删掉的 20 行没逐行核 |
| K4 与删除线 S | 没打中 | 只读了借卡那几处与 `versions_without_a_file` 的交界，没找到互相踩的；读得浅 |

## K1-1　式子每一项取的是不是定义说的那个量

### 甲形：空闲与占着的不是同一刻读的，这个进程刚建上卡的那一笔被算两遍

读法在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs`：先起一次 `nvidia-smi` 读空闲，「let reported_by_bus = reported_cards_by_pci_bus();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1482`），再起一次读这个进程占着的，「let used_by_this_process = memory_used_by_this_process_on_each_bus();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1483`）。
账的锁只锁这个进程的账，不锁这个进程别的线程建缓冲：借到借据的线程出了锁就建，下一个等锁的线程进锁就读。
两次读之间那条线程把它借到的那一笔 S 建上卡：空闲是建之前的（还把 S 当空着），占着的是建之后的（含 S），「Some(occupied) => occupied.min(promised_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:138`）把 S 算成「借了已经占上的」不再扣。
M 比真的大 S，这一笔能借的比卡上真空着的多，多出的量是 S 与「额度 − 驱动那一份 − 建之前的空闲」里小的那个。

历史（单进程，每一步许它的那一句）：
1. 同一个测试二进制里两条 GPU 用例并行（libtest 线程）；用例一照「借 `need_mebibytes`：照卡的优先级记在第一张此刻放得下的卡上」借到 S，出锁（`crates/singlefs-harness/src/gpu_memory_ledger.rs:653` 那一句之下的 `book_on_one_of_these_cards`，锁在第 705 行 `drop(books)` 放）。
2. 用例二进锁，`nvidia-smi` 第一次读空闲。
3. 用例一建 S 的缓冲（锁外）。
4. 用例二 `nvidia-smi` 第二次读占着的，含 S。
5. 用例二照混着的读数算能借的，借到比卡上真空着的多的量；两笔都建上卡就超。

量过（模型 `k2_a_…`，四种配置 × 别的进程占 0–20 GiB × 先借 1–12 GiB，88 行）：`SUMMARY k2a rows=88 rows_over_physical_free=42 worst_over=12288`。
几行原样（`research/prompts/m3-batch10x-code-r1-opus-model/run-output.txt`）：

```text
ROW k2a config=quiesced-32G other=12288 lease=12288 free_after=4847 room_consistent_before=4974 room_consistent_after=4847 room_mixed=17135 over=12288
ROW k2a config=quiesced-32G other=0 lease=8192 free_after=21231 room_consistent_before=21358 room_consistent_after=21231 room_mixed=22016 over=785
ROW k2a config=daily-15G other=20480 lease=8192 free_after=751 room_consistent_before=878 room_consistent_after=751 room_mixed=4608 over=3857
```

读法：清场那一份配置（`local:0:32G`，本机 `~/.config/singlefs/gpu-cards.env` 第 11 行）而卡上真的干净（other=0），多借封顶在 785 MiB（额度 32768 比空卡能读到的多出的那一截加驱动差）；卡上另有进程占着（主 agent 这一轮的合后验证与变异就是两个测试进程同时在 5090 上，见「量的数」②），多借到整笔 S。
一致读数的两列（建之前、建之后）与真空着的差 127 与 0，混着读才多出来，所以是读法造的，不是额度造的。

窗口多大：两次 `nvidia-smi` 各 0.04–0.07 秒（「量的数」③），第二次起进程的那几十毫秒就是窗口。

**四句**：
- 分不分辨臂：这一轮没有两条臂；它只打 G1 二的一项（「本进程借了还没实际占的」），两次读改成同一刻的读数、或先读占着的再读空闲就不中（改法见下），所以分得出「这一形的读法」与「同一刻的读法」。
- 系统当时看不看得到：看得到。两次读都在锁里、都是这个进程自己起的，它知道两次之间隔了一次进程启动。
- 满足的是哪一句：定义「F − 这个进程借了还没实际占的 − D」里「还没实际占的」取错了（`crates/singlefs-harness/src/gpu_memory_ledger.rs:19` 那一句），不是竞态那一句：只有一个进程。
- 跑前条款给的改法：这一轮没有跑前条款。

### 乙形：这个进程不在账上的显存被当成「借了已经占上的」

文件头自己写了这一种的来路：「实际占着的比许出去的多（分配器按块拿、别的入口起的设备）只算到许出去的那么多」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:121`）。封顶到许出去的那么多，挡的是「多算」，挡不住「把不在账上的那一份当成在账上的那一笔已经建了」。
「别的入口起的设备」是真的会同进程同时出现的：`crash_amplification.rs` 文件头第 5 条写，同一个测试二进制里经 `run_from_environment` 的用例跑在取卡池的用例起池之前的，这一趟自己在点名的卡上起设备、不进账（`crates/singlefs-checker-tier/src/crash_amplification.rs` 文件头那一段，第 41-45 行）。两边同一个进程号，`nvidia-smi` 按进程号报的占着的是两边合起来的。

历史：产品路径用例在卡上自己起设备占着 U（不在账上）；取卡池的用例甲第一次在这张卡上借 F1，登记了、设备还没起（`started_card` 起设备要几百毫秒到几秒）；用例乙此刻借：占着的 U 被封顶成 F1 + D 算进 M，能借的里没扣 F1 与驱动那一份。

量过（模型 `k2_b_…`，48 行）：`SUMMARY k2b rows=48 worst_over=6783`。原样一行：

```text
ROW k2b config=quiesced-32G unbooked=23167 first=4096 free=8943 second_room=8943 free_once_the_first_is_built=2160 over=6783
```

还了的借据留在卡上还没放的缓冲（`GpuCardLease` 丢掉时只 `poll(Poll)` 一次，不等）也落进同一形，推的、没量。

**四句**：分得出（把「不在账上的」从占着的里扣掉、或读不到归属时当 0 就不中，推的）；系统看不看得到：看不到。`nvidia-smi` 按进程号报，同一进程里两个入口的显存分不开；这一格属于「判别子观测不到」，改法只能是让读数分得开或改口径。满足的分句同甲形。没有跑前条款。

### 丙形：「空着时能借的」拿总量当 M，空卡读不到总量

「ceiling_mebibytes: capacity_of_memory_this_process_may_hold(reading.total_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:174`）。文件头的定义同样写「同一个算法拿卡的总量当 M」，代码照定义；错在定义的前提：空着的卡读得到的空闲不是总量。
本机此刻空着的 5090：`memory.free` 32110、`memory.total` 32607、没有进程（「量的数」①），差的 497 MiB 是驱动留着的，谁都借不走。

要的量落在 (空卡此刻能借的, 空着时能借的] 之间：账判「空着时放得下」，不判红；此刻永远借不到，报本次未跑。文件头要的是「等多久都放不下，调用方判红（配了 GPU 就一定用 GPU）」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:22`），这一段永远走不到那一支。
量过（模型 `k1_c_…`，四种配置）：

```text
ROW k1c config=quiesced-32G room_on_the_empty_card=29550 need=29551 outcome=not-run ceiling_seen_in_red=0 largest_need_still_not_run=30047 band_mebibytes=497
ROW k1c config=daily-15G room_on_the_empty_card=12800 need=12801 outcome=red ceiling_seen_in_red=12800 largest_need_still_not_run=12801 band_mebibytes=1
ROW k1c config=unconfigured-3/4 room_on_the_empty_card=21521 need=21522 outcome=not-run ceiling_seen_in_red=0 largest_need_still_not_run=21893 band_mebibytes=372
ROW k1c config=unconfigured+env30000 room_on_the_empty_card=21521 need=21522 outcome=not-run ceiling_seen_in_red=0 largest_need_still_not_run=21893 band_mebibytes=372
```

`daily-15G` 那一行的 `band_mebibytes=1` 是模型二分的起点造的：`need=12801` 已经判红，门槛就是 12800、带宽 0（额度 15G 比空卡的空闲小，额度先封顶）。另三种配置各有 497、372、372 MiB 的带。
带只罩固定量的用例（GPU 对拍按量借那一族）；产品路径按状态数压，一个状态的量远小于带宽，不受这一形影响。

**四句**：分得出（ceiling 改取「空卡读到的空闲 + 这个进程占着的」就不中，推的）；系统看得到（每次借都现读了空闲与总量）；满足的分句：「借的要比这张卡空着时能借的…还大，等多久都放不下，调用方判红」，`空着时能借的` 取错了量；没有跑前条款。

## K1-2　跨进程那一套删干净了吗

没打中。冻结副本里 `grep -rn 'gpu-memory-ledgers\|LONGEST_WAIT_FOR_GPU_MEMORY\|POLL_INTERVAL_FOR_MEMORY'`（`.rs`、`.sh`、`.py`、`.tsv`）零命中；三份被判文件里 `flock|账本|等满|1800|判活|/proc/` 只剩两处注释：
- 「这条用例报「本次未跑」时打的那一行」那一句在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 811 行，后半句还写「崩溃放量产品路径借草稿区等满了也打它」；
- `crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 9 行是新口径的说明，不算。
分项驱动 `scripts/crash/crash-amplification-item.sh` 文件头第 24 行还写「池里的卡等满了也登记不上」、自证桩里还造「等了 1800 秒」的行（第 299、307、333 行），是说法旧了，不是代码路径。

试过的形：账本目录变量、`SINGLEFS_WORK_ROOT`、等满常数、判活函数在 crates、scripts、`.claude/gate.d` 里的引用；都没有。

## K1-3　竞态兜底：两个进程一起借超了，会走到哪条路

前提（用户定的）：两个进程同一刻读到同一份空闲、一起借超了，不在账里防。这一节只核兜底路。
兜底有两条：① 内存包装：命令退非 0、起止之间内核日志有 `NVRM … NV_ERR_NO_MEMORY` 就退 245；② 产品路径在卡上接住 wgpu 的 Out of Memory，报本次未跑（「第 ① 段卡上建判器、判一块时 wgpu 报 Out of Memory」那一句在 `crates/singlefs-checker-tier/src/crash_amplification.rs:37`）。
② 只接 `on_the_card_catching_out_of_memory` 包住的那几段，而它的错误范围是线程的：「wgpu 的错误范围按线程分：`work` 要在调它的这条线程上建缓冲、派活。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1980`）。没报 Out of Memory 而 `work` panic 了就照样往外抛：「(Err(panic), None) => std::panic::resume_unwind(panic),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1997`）。
wgpu 没接住的错误默认 panic：`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs` 第 694 行 `panic!("wgpu error: {err}\n");`；范围按线程存：同一份文件第 634 行 `scopes: HashMap<thread_id::ThreadId, Vec<ErrorScope>>,`。这个仓里没有一处装 `on_uncaptured_error` 或 `set_device_lost_callback`（`grep -rn` crates 零命中）。

### 四处撞了显存走不到 ②，判红

| 号 | 在哪 | 撞显存时 | 引文 |
|---|---|---|---|
| R1 | 借到之后在卡上起设备、编 CRC 管线（卡池 `started_card`），不在任何错误范围里 | `request_device` 交错就 panic；编管线报 Out of Memory 走默认处理 panic | 「账上登记了这个进程，卡却起不来上下文（驱动坏了，或卡被别的程序占满）」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1116`）；「.unwrap_or_else(\|reason\| panic!("{test_name}：{reason}"))」见第 1288 行 |
| R2 | 第 ① 段建判器时三份内核里两份在另起的线程上编（默认，没设 `SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1`） | 那两条线程上的 Out of Memory 不进卡线程的范围，走默认处理 panic；卡线程 `join().expect` 再 panic；范围里没有 Out of Memory，`resume_unwind` | 「.spawn(\|\| compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery));」见 `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 第 454 行；「recovery.join().expect("恢复内核的编译线程不 panic"),」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:468`） |
| R3 | 只排派活计划那一趟借到草稿区之后试判：建判器、判两段都没包 | 直接 panic | 「let judge = GpuCrashJudge::on_card_with_packed_tables(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5612`）；「let _warm = judge.judge_states(first..warm_end);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5624`）；文件头只写了「只排派活计划那一趟试判前同样借。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:25`） |
| R4 | ② 接住了一次 Out of Memory，而那一次落在 wgpu-core 判成致命的调用上（建管线、建绑定组、建命令编码器、提交）：设备被判丢了。卡池每张卡一个设备、整个进程共用、起过就不再起 | 这一趟报本次未跑；同一进程后面借这张卡的用例拿到的是丢了的设备，wgpu 把「设备丢了」类的错误一律吞掉不报，回读映射失败时 `expect` panic，不是 Out of Memory，接不住 | wgpu-core `src/device/resource.rs` 第 706 行 `self.lose(&error.to_string());`（`handle_hal_error` 把 Out of Memory 当致命）；只有建缓冲、纹理、采样器、查询集走第 714 行的不致命那一支；wgpu `wgpu_core.rs` 第 304 行 `ErrorType::DeviceLost => return, // will be surfaced via callback`；「started: std::sync::OnceLock<Result<GpuCard, String>>,」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1007`） |

可达的历史（R1，两个进程，竞态之后只走这一个进程的那几步）：
1. 进程甲、乙同一刻读到 5090 空着，各借一笔、各登记了这张卡（用户认了的竞态）。
2. 甲先起设备、建缓冲，占满。
3. 乙 `card_lent_for_the_booking` → `started_card` → `start_the_card`：上下文要的显存（驱动在 `ctx_buf_pool` 里分配）分不出来，`request_device` 交错 → 「.ok()?;」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:229`）→ 乙 panic，GPU 用例红、产品路径红。
本机上一次开机里驱动确实这样报过：`journalctl -k -b -1` 里 39 行 `NVRM … NV_ERR_NO_MEMORY`，其中一行是 `rmMemPoolReserve(pCtxBufPool->pMemPool[i], totalSize[i], 0) @ ctx_buf_pool.c:315`（「量的数」④）。

R2 的历史：乙在第 ① 段 `GpuCrashJudge::on_card_with_packed_tables` 里（被「match on_the_card_catching_out_of_memory(card, "建判器（表与草稿区）时", || {」包着，`crates/singlefs-checker-tier/src/crash_amplification.rs:2018`），表与草稿区建成了，编内核时甲把卡占满；恢复内核或扫描内核在另起的线程上建管线，报 Out of Memory，那条线程上没有范围 → 默认处理 panic → `join().expect` → 卡线程上范围是空的 → 抛出 → 整趟红。

### 245 接得住多少

① 的判法在 `scripts/hosts/run-with-memory-cap.sh` 文件头：命令退非 0 才读内核日志，「驱动的这一行不带进程号，同一段时间里别的进程撞了显存也算进来」（`scripts/hosts/run-with-memory-cap.sh:100`）。R1–R4 都让命令退 101，驱动那一行在窗口里，就退 245；本机此刻 `journalctl -k` 读得到（「量的数」④）。所以经包装跑的那一条，R1–R4 最后都落进 245，不会留成红。
接住之后的代价不同：② 是「这一趟本次未跑、判过的块在库里」，245 是「这一条命令的输出全不算数」。门禁里一条 `cargo test` 罩一整个测试二进制甚至整个包，一处 R2 让这一整条的结果全作废（包括同一条里别的用例真红的结论）。
没经包装跑的（人手直接 `cargo test`、读不到内核日志的机器）R1–R4 就是红，原因写的是「起不来上下文」「编译线程 panic」，认不出是显存。

### 没找到判通过、没人认出的形

试过的形与结论：
- 回读：判器、单元校验和、第 ② 段核对三处回读都 `recv().expect(…).expect("回读映射成功")`、`get_mapped_range().expect(…)`，设备丢了、缓冲没建成都 panic，不会交空结果（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` 第 887、889 行，`gpu_unit_checks.rs`、`crash_verify_gpu.rs` 同形）。
- 叫停那一趟：`stopped_early` 时 `violations` 交空（`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 4911 行那一处），固定脚本与 `common_crash_points` 的调用方只断言判过的块里 0 红、打 `stopped_early=true` 行；门禁 feature-gated-tests 格带 `--nocapture` 数 `NOT_RUN_NO_GPU_ADAPTER` 行，入口 `crash-amplification.sh` 把 `stopped_early=true` 记 not-run。没找到哪个调用方把它当通过。
- 卡上真撞显存而 wgpu 不报：没找到，Linux 上 NVIDIA 的 Vulkan 设备内存分配失败照报（推的，没在卡上试）。
- 没找到挂死的形：设备丢了时映射回调照样以错误送达（推的，读 wgpu 源码，没跑）。

**四句**（R1–R4 一起）：
- 分不分辨臂：这一格不判臂；它打的是定义里「wgpu 报 Out of Memory 时崩溃放量产品路径把这一块报本次未跑」那一句的射程，R3、R2 在产品路径里，R1、R4 在产品路径与 GPU 用例共用的卡池里。
- 系统看不看得到：R1 看得到（错误说「卡被别的程序占满」，代码自己写出了这一种）；R2 看得到（是这个进程自己另起的线程）；R4 看得到（同一个进程刚报过一次 Out of Memory）；R3 看得到。四处都是没接，不是接不住。
- 满足的分句：用户定的「靠运行时认出显存不够（内存包装退 245）报本次未跑兜底」字面只要求 245 那一条，R1–R4 全落进 245，按这一句字面没漏；按 10x 自己文件头加的第二条（第 ① 段建判器、判一块时报本次未跑），R2、R3 漏了。属于哪一条判据交主 agent 定。
- 改法：没有跑前条款。

## K2-1　单进程里账给的量大于此刻真能用的

**打中两形**，就是 K1-1 的甲形（混着读，最坏多借 12288 MiB）与乙形（不在账上的显存，最坏多借 6783 MiB），都只要一个进程，不落在用户认了的竞态里。数与原样行见 K1-1。

题面问的「现读之后、借之前卡被别人占了」：这一形在代码里不存在，读、判、记在同一次锁里（「在锁里现读每张卡，算这几张卡此刻各能借给这个进程的」那一段，`rooms_of_the_cards`，`crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 585-615 行）。「记了之后、建上卡之前被别人占了」才是那一形，它是两个进程的竞态，归用户认了的那一类。

## K2-2　白等、白红、压一张卡

### 白红甲：每张卡此刻放不下一个状态、空着时放得下，产品路径判红

产品路径先照此刻能借的算每张卡带得了几个状态（`cards_of_the_pool_with_what_each_lends_now`，`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2224-2259 行），全是 0 就不去借：「let scratch = if capacities.iter().all(|capacity| *capacity == 0) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2596`），接着判红：「没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2636`）。
定义要的是本次未跑：「比此刻能借的大而空着时放得下，是借不到」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:23`）。
就算去借也一样：账把「这张卡此刻一个状态都带不了」记成「前面的卡已经装下了全部要判的状态（或这张卡一个状态都带不了）：没去借。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:249`），判的时候「ScratchAttemptOnACard::NothingToPlace => {}」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:327`）不计成「此刻借不到」，全是它就交判红那一支。
这一批的变异行「第10x批 显存账 此刻连一个状态都放不下的卡判成空着时也放不下（判红）」（`crates/mutations.tsv:3132`）钉的是另一处（`CannotHoldOneState`），这一处没有。

量过（模型 `k2_w1_…`，一张卡此刻空闲 3000、空着 32110，表 600 MiB 加一个状态 2 MiB）：

```text
ROW k2w1 room_now=440 room_when_empty=29550 one_state=602 holds_now=0 booking=red need=602 largest_ceiling=0 decision_on_nothing_to_place=NoCardHoldsOneState { largest_ceiling_mebibytes: 0 }
```

可达的历史：
1. 起过卡池的测试进程里，产品路径用例要判一条流。
2. 此刻池里每张卡都被占着：同一进程里 GPU 对拍用例借着大的一笔（这个二进制里两边并行，产品路径的环境锁只排产品路径之间），或卡上另一个测试进程占着（「量的数」②：主 agent 这一轮就是两个测试进程同时在 5090 上，一个到 11759 MiB）。
3. 产品路径算出每张卡带 0 个状态 → `Err` → 调用方「run_from_environment(&flow, &judging, &root, limit).expect("按环境跑得完");」（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:328`）panic，用例红。
4. 经分项驱动跑的，这一句被认成设备用不上（`scripts/crash/crash-amplification-lib.sh` 第 340 行的 `device_pattern` 里有「没有一张卡装得下这条流的表」），退 78，出路写「修好设备，或改配置」；该是 77 本次未跑。

**四句**：分得出（把全是 0 那一支改成去借、`NothingToPlace` 在「此刻带不了」时改记 `NoRoomNow`，就落进本次未跑，推的）；系统看得到（此刻能借的与空着时能借的它都算了）；满足的分句是定义里「比此刻能借的大而空着时放得下，是借不到…一张都借不到报本次未跑」；没有跑前条款。

### 白红乙：GPU 对拍那一份借不到就判红

「（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1165`）：10t 时它是等，10x 去掉等之后变成借不到当场 panic。
调用方 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 有 151 条 `#[test]`，五处调它（第 2070、2516、2630、2822、3501 行），其中第 2070 行在 `compare_on_every_state_of_these_segments_against_the_version_table` 里、被很多条用例共用；libtest 几条线程同时借同一张 5090。主 agent 这一轮那个二进制在 5090 上到过 11759 MiB（「量的数」②）。
同一进程里兄弟借着、几秒之后就还，这一条判红。模型 `k2_d_…` 量的是同一形的「借不到」那一步（没走 panic 那一层）：

```text
ROW k2d first=20480 second=12288 free_while_first_held=8943 second_outcome=not-run second_books_after_the_first_returned=true
ROW k2d first=24576 second=8192 free_while_first_held=4847 second_outcome=not-run second_books_after_the_first_returned=true
```

`not-run` 在 `lease_card_waiting_for_room` 那一层就是 panic。哪几条用例、各要多少、会不会真撞在一起，没上卡量，推的。
这一处背景写的「已知没做」只有「GPU 用例自己报 Out of Memory 还没改成本次未跑」，不是这一条（这一条是账交借不到，不是卡上报 Out of Memory）。

**四句**：分得出（改成报本次未跑或同进程里等自己的借据还回来就不中）；系统看得到（借着的都是这个进程自己账上的）；满足的分句：「卡上放得下却借不到、白红」；没有跑前条款。

### 白未跑：同进程兄弟借着，当场报本次未跑

`lease_card_or_report_not_run` 与产品路径同上一形，只是落进本次未跑。定义就是「不等」，按字面不算打中；记一笔：一个进程里几条 GPU 用例并行，借不到的那几条每一趟都报本次未跑，门禁 feature-gated-tests 格逐条记未跑，同一份代码两次跑哪几条未跑可以不同。要不要「同进程里等自己的借据」交主 agent。

### 压到一张卡：没打中

`place_states_on_the_cards_that_book_now` 照卡的先后先装满前一张，是用户定的「一个显卡 任务满了才去分另外一个」（`crates/singlefs-harness/src/gpu_memory_ledger.rs` 文件头第 5 行）。借不到的卡留给后面的卡，后面的卡照它剩下的装。按代码推了「第一张此刻满、第二张空」「两张都只装得下一半」两种形（推的，没进模型），没有把本该分开的状态压到一张上。

## K3　测试判别力

- K3-1 没打中。产品路径用例断言判器建好那一刻草稿区那一笔还在账上（`crates/singlefs-checker-tier/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs` 第 211-216 行），并断言判器用的额度等于借的加驱动那一份、草稿区不超过借的（第 199-210 行）。「借了又在建判器之前还」会红。分不出的是「建好之后、判完之前还」：借据在调用方手里活到函数收尾，卡线程拿不到它，没造出一个能在那一段里还掉的单行改法，推的。
- 另记一处判别力的缺口：这份用例在产品路径 `stopped_early` 时打本次未跑、当场返回（第 160-169 行）。改法让账永远借不到（例：此刻能借的恒为 0）时，两条用例都不红，只记两条本次未跑；这一类要靠 harness 档的账单测拦，产品路径那一层拦不住。
- K3-2 没攻。
- K3-3 部分：这一批新增 32 行变异（改动全文 `research/prompts/_m3-batch10x-code-r1-diff.md` 里 `mutations-append.tsv` 那一节，`sed -n 6506,6545p <那份文件> | grep -c '^第10x批'` 输出 `32`）里，没有一行罩白红甲那一处（`crash_amplification.rs` 第 2596 行全是 0 就判红）、也没有一行罩 K1-1 丙形（ceiling 取总量那一项被换成「空卡读到的」不会让哪一行红，因为没有用例在那段带里）。删掉的 20 行没逐行核今天还在不在。

## K4　与删除线 S 的交界

没打中，读得浅。只核了两处：第 ① 段借草稿区按 `mebibytes_a_judge_needs(tables, states)` 算，`tables` 是连同 `versions_without_a_file` 建的那一份（`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 2568-2577 行一带），借的与判器开的同一份表；只排派活计划那一趟用 `states_of_the_flow` 当封顶，判那一趟用 `tables.state_count`，两者不同时判器开的只会不超过借的（判器的额度是借的加驱动那一份），不会超借。复用键那一侧没读。

## 改法（全部是我自己提的、只按代码推、没实现没跑、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 | 在别的打中格上 |
|---|---|---|---|
| 先读这个进程占着的、再读空闲（或一次 NVML 调用同一刻读两样） | K1-1 甲、K2-1 甲 | 推的 | 换过来之后混着读只会少借（建上卡的那一笔在占着的里没有、在空闲里已经扣了）；对乙、丙不起作用 |
| 读得到占着的也不当成借据已经建上（照「读不到」那一支：借着的当一点没建、起好的驱动那一份当在卡上） | K1-1 乙、K2-1 乙；也盖住甲 | 推的 | 代价：这个进程已经建上卡的那一笔被扣两遍（空闲里扣过、借着的里又扣），同进程里后借的白借不到 |
| ceiling 的 M 取「此刻空闲 + 卡上已用」（`nvidia-smi memory.used`，空卡上是 32112） | K1-1 丙 | 推的 | 不碰别的格 |
| 起过卡池的产品路径不在「全是 0」时判红，交给借那一步判；账里「此刻带不了」记成 `NoRoomNow` 而不是 `NothingToPlace` | K2-2 白红甲 | 推的 | 不碰别的格 |
| `lease_card_waiting_for_room` 借不到报本次未跑，或同进程里等自己的借据还回来 | K2-2 白红乙 | 推的 | 等自己的借据会把 10t 的「等满」带回来一半（只等本进程，不等外人） |
| 起设备、编 CRC 管线那一步认出 Out of Memory（`request_device` 交错、错误范围里有 Out of Memory）报本次未跑 | K1-3 R1 | 推的 | — |
| 判器的三份内核都在卡线程上编，或在另起的线程上各推一个 Out of Memory 范围、把结果交回卡线程 | K1-3 R2 | 推的 | 都在卡线程上编要多几分钟冷编（文件头说驱动编译单线程） |
| 只排派活计划那一趟试判也包进 `on_the_card_catching_out_of_memory` | K1-3 R3 | 推的 | — |
| 卡池记住哪张卡的设备报过 Out of Memory，之后借这张卡的照「本次未跑」报，或重起设备 | K1-3 R4 | 推的 | 重起设备要清 `OnceLock`，`started` 那一格今天起过就不再变 |

## 量的数（命令与原样输出）

① 空卡读数（本机，2026-10-07，没有进程在卡上时）：

```text
$ nvidia-smi --query-gpu=index,pci.bus_id,name,memory.free,memory.total --format=csv,noheader
0, 00000000:01:00.0, NVIDIA GeForce RTX 5090, 32110 MiB, 32607 MiB
1, 00000000:10:00.0, NVIDIA GeForce RTX 5060 Ti, 15849 MiB, 16311 MiB
$ nvidia-smi | tail -15   （节选两行）
|  0%   30C    P8             12W /  450W |       2MiB /  32607MiB |      0%      Default |
|  No running processes found                                                             |
```

② 主 agent 这一轮的 GPU 测试在跑时（不是我起的；我只读 `nvidia-smi`，每 10 秒一次、18 次），原样在 `research/prompts/m3-batch10x-code-r1-opus-model/nvidia-smi-during-main-agent-gpu-run.txt`。命令：
`timeout 200 bash -c 'for i in $(seq 1 18); do echo "== $(date +%s)"; nvidia-smi --query-compute-apps=pid,gpu_bus_id,used_memory,process_name --format=csv,noheader; nvidia-smi --query-gpu=index,memory.free,memory.used --format=csv,noheader; nvidia-smi | sed -n "/Processes/,\$p" | grep -E "^\|\s+[0-9]" ; sleep 10; done'`
`grep -v '^==' … | sort | uniq -c | sort -rn | head` 的前几行：

```text
      8 3966260, 00000000:01:00.0, 2687 MiB, /home/fy5090/code/singlefs/target/release/deps/record_checker_judges_absence_by_the_persisted_set-f3c1f68122a9a243
      8 |    0   N/A  N/A         3966260    C+G   ...ersisted_set-f3c1f68122a9a243       2687MiB |
      3 3950266, 00000000:01:00.0, 11759 MiB, /home/fy5090/code/singlefs/target/release/deps/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state-d0da6b886678413b
      3 0, 20340 MiB, 11772 MiB
```

读法：wgpu 的 Vulkan 进程在 `--query-compute-apps` 里是列出来的（类型 `C+G`），所以 `occupied_by_this_process_mebibytes` 是 `Some`，K1-1 甲、乙走得到；`gpu_unit_checks.rs` 第 136 行注释担心的「驱动不按进程报 Vulkan 的上下文」在本机不成立。两个测试进程同时在 5090 上。起了设备、借据还完的进程占 2687 MiB（模型里的 `BASELINE`）。

③ 两次 `nvidia-smi` 各要多久（`research/prompts/m3-batch10x-code-r1-opus-model/nvidia-smi-query-timing.txt`）：

```text
$ for i in 1 2 3 4 5; do /usr/bin/time -f "%e s" nvidia-smi --query-gpu=pci.bus_id,index,memory.free,memory.total --format=csv,noheader,nounits >/dev/null; /usr/bin/time -f "%e s apps" nvidia-smi --query-compute-apps=pid,gpu_bus_id,used_memory --format=csv,noheader,nounits > /dev/null; done
0.04 s
0.05 s apps
0.05 s
0.05 s apps
0.04 s
0.06 s apps
0.04 s
0.07 s apps
0.07 s
0.06 s apps
```

④ 内核日志读不读得到、驱动撞显存时报不报：

```text
$ timeout 20 journalctl -k -n 1 -q --no-pager | cut -c1-200; echo "rc=$?"
Oct 07 〔本机主机名〕 kernel: Memory cgroup out of memory: Killed process 3611919 (python3) total-vm:280136kB, anon-rss:64000kB, file-rss:6856kB, shmem-rss:0kB, UID:1000 pgtables:192kB oom_scor（本机主机名已换掉）
rc=0
$ timeout 60 journalctl -k -q --no-pager --since "2026-09-20" | grep -cE 'NVRM.*NV_ERR_NO_MEMORY'
0
$ timeout 60 journalctl -k -q --no-pager -b -1 | grep -cE 'NVRM.*NV_ERR_NO_MEMORY'
39
$ timeout 60 journalctl -k -q --no-pager -b -1 | grep -E 'NVRM.*NV_ERR_NO_MEMORY' | head -3 | cut -c1-260   （第三行）
Oct 06 08:29:48 〔本机主机名〕 kernel: NVRM: GPU1 nvAssertOkFailedNoLog: Assertion failed: Out of memory [NV_ERR_NO_MEMORY] (0x00000051) returned from rmMemPoolReserve(pCtxBufPool->pMemPool[i], totalSize[i], 0) @ ctx_buf_pool.c:315（本机主机名已换掉）
```

（第一条 `--since` 只读到这次开机，这次开机以来 0 行；上一次开机 39 行。）

⑤ 冻结副本与仓根逐字节相同（`cmp -s` 七份：`gpu_memory_ledger.rs`、`gpu_unit_checks.rs`、`crash_amplification.rs`、`crash_judge_dispatch.rs`、产品路径卡池用例、`run-with-memory-cap.sh`、`crash-amplification-item.sh`，七行都是 `same …`）；冻结副本对快照 `sha256sum -c` 318 份 `OK`。

⑥ 模型（复跑命令见开头；第三次跑，只改过一句注释之后重跑，ROW 与 SUMMARY 行与第二次逐行相同，`diff` 输出 `rows-identical`）：

```text
SUMMARY k1c configurations_with_a_band=3
SUMMARY k2a rows=88 rows_over_physical_free=42 worst_over=12288
SUMMARY k2b rows=48 worst_over=6783
SUMMARY k2d not_run_rows=3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

第一次跑 `k2_w1` 红（我给的一个状态 62 MiB 比此刻能借的 440 小，造的形不对），改成 602 MiB 重跑才绿；这一次红是模型自己的取值错，不是被判的代码。

## 没打中的形状

| 格 | 试过的形 | 取样 |
|---|---|---|
| K1-1 | 「借了还没占的」在三种驱动那一份状态下（没登记、登记没起设备、起好）各展开一遍，读到 / 读不到占着的两支；分配器多拿块（占着的比许出去的多）| 读码，`gpu_memory_ledger.rs` 第 124-150 行逐支；一致读数下只差驱动实占比 2560 多的那 127 MiB（模型 k2a 的 `room_consistent_before − free_after`），那一项归已知没做「驱动固定开销 2560 MiB 没上卡量」 |
| K1-2 | 账本目录、锁、判活、等满的常数与变量在 crates、scripts、`.claude/gate.d` 的引用 | `grep -rn`，零命中 |
| K1-3 判通过 | 回读交空、映射失败吞掉、叫停那一趟被当通过、OOM 被吞成别的错 | 读码：三处回读、两份叫停调用方、门禁两格、入口一处；wgpu 0.30 源码里错误分派那一段 |
| K1-3 没人认出 | 挂死（映射回调不送达）、读不到内核日志 | 读 wgpu 源码，推的；本机日志读得到 |
| K2-2 压一张卡 | 第一张此刻满第二张空；两张都只装得下一半 | 按代码推，没进模型 |
| K3-1 | 借了又在建判器之前还；建好之后还 | 读码 |
| K4 | 草稿区的量与判器开的量是不是同一份表、同一个封顶 | 读码两处 |

## 这条腿自己的限度

- 没上卡：K1-3 的 R1–R4 都是读码加读 wgpu 源码推的，没在卡上造过两个进程一起借超；「哪一种调用先撞」由驱动定，我没量。R4 要 Out of Memory 落在致命那几种调用上，概率没量。
- K1-1 甲的窗口是两次 `nvidia-smi` 之间那几十毫秒，模型只造了「混着读」那一刻的读数，没造「另一条线程正好在这几十毫秒里建缓冲」的时序；能不能撞上、多常撞上没量。
- K1-1 乙要「同一进程里有不在账上的显存、同时卡池的一笔借了还没建」，同进程两种入口并行是照 `crash_amplification.rs` 文件头写的那几个二进制推的，没跑那几个二进制看它们真的重叠。
- K2-2 白红乙没数那几条对拍用例各借多少、并行时会不会合起来超；只有主 agent 那一趟的 11759 MiB 峰值作旁证。
- 模型里的卡池容量闭包是照 checker 档代码抄的（`budget` 抄 `memory_budget_mebibytes_of`），不是调它；抄错了模型就错。
- 只抽一次样：本格的「没打中」都按规则算一次观测，不支撑「没有」。
- K3-2、K3-3 删掉那 20 行、K4 复用键那一侧没攻。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量的数不算入库装置上的数。
- 没跑重型测试，没跑 checker 档任何测试二进制，没上卡；`nvidia-smi` 与 `journalctl` 只读。
- 编译与跑都在草稿目录 `~/.local/share/singlefs/work/m3-batch10x-code-r1-opus/` 下的仓副本 `repo/` 与编译目录 `target/` 里，交回前删掉（见交回）。草稿目录里留 `build1.log`、`run1.log`、`run2.log`、`run3.log`、`nvsmi-watch.txt`、`nvsmi-timing.txt`，要用的已拷进模型目录。
