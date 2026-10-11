# m3-batch10x-code-r1　正推腿（Sonnet）报告：K1–K4

轮名 m3-batch10x-code-r1，立场正推。日期 2026-10-07。方案按 `crates/` 今天的实现来谈；代码读冻结副本，冻结副本与仓里此刻同 sha（下面命令原样输出）。

## 0　做法与前提

- 被判对象三份源码的 sha256（冻结副本在前、仓里在后，各取前 16 位）：

```text
$ for f in crates/singlefs-harness/src/gpu_memory_ledger.rs crates/singlefs-checker-tier/src/gpu_unit_checks.rs crates/singlefs-checker-tier/src/crash_amplification.rs; do sha256sum $f | cut -c1-16; sha256sum ~/.local/share/singlefs/work/m3-batch10x-code-r1/tree/$f | cut -c1-16; done
9f6fc8fd74f54b87
9f6fc8fd74f54b87
ff2ad2d40b35a9d4
ff2ad2d40b35a9d4
f20719ecf7d3cdef
f20719ecf7d3cdef
```

- 只跑 harness 档：`cargo test -p singlefs-harness`，经 `bash scripts/hosts/capped.sh 10 bash scripts/hosts/run-with-memory-cap.sh 4G …`，仓副本与编译目录在 `~/.local/share/singlefs/work/m3-batch10x-code-r1-sonnet/`（交回前删）。起跑前 `host-resources.py` 看过：本机可用线程 0–27 间摆动（主 agent 的合后验证在跑）；没有性能测量在跑。钩子有几次对默认预测 8G 报「本机放不下」，我给的上限是 4G，包装没排队、没退 250–254，测试二进制实际峰值 26 MiB（host 历史表记的）。
- 基线（冻结副本、不改一行）：`cargo test -p singlefs-harness --lib --offline gpu_memory_ledger` → `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out`。
- 模型与日志在 `research/prompts/m3-batch10x-code-r1-sonnet-model/`：`forward_probe.rs`（七条探针，假读数，不要显卡）、`mutate-probe.py` + `mutants1.json`（harness 档变异探针）、两份原样输出日志。
- checker 档（含 GPU 用例、`crash_amplification` 的单测）按规定一律不跑。凡写「复核不了」的，都是要真卡、或要 checker 档才能坐实的。
- G1 二的定义（岔路单原文）：「不做跨进程账，每次借现读空闲、只记本进程借的；两个进程同时读到同一份空闲的竞态，靠运行时认出显存不够（内存包装退 245）报本次未跑兜底」（`research/prompts/m3-batch10t-code-r1-forks.md:9`）。
- 用户已定、不算打中的（竞态一形）照背景材料不报。本报告里的冲突都是**单进程内**的形，或「兜底那条路本身漏了」的形。

## K1　G1 二做得对不对、全不全

### K1-1　能借的量那条式子

**逐项对上了，读数一致时一格不差。** 代码取的量与文件头口径对应：

- 现读的 F、总量、本进程实际占的：`CardMemoryReading`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:95-100`）；
- 本进程此刻能占的 M = F + 实际占着里属于借出去那部分：「Some(occupied) => occupied.min(promised_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:138`）；
- 能借的 = 容量(M) − 借着的：「room_now_mebibytes: capacity_of_memory_this_process_may_hold(may_hold_mebibytes)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:172`）、「.saturating_sub(booked.leases_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:173`）；
- 读不到实际占着的：文件头「实际占着的读不到（`nvidia-smi` 没列这个进程）时当借着的一点都没建、起了设备的驱动那一份已经在卡上，只会少借。」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:20`）。

我另写了一份**独立的定义式**（`definition_room`，i128 算、不用 saturating，在 `forward_probe.rs` 里）和一个**安全上界**（卡总量 − 别人占的 − 一份驱动 − 借着的），喂假读数逐格比：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:5: test p1a_room_equals_the_definition_on_a_grid ... P1a cells=3402 mismatches=0
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:7: test p1b_room_equals_the_safe_bound_when_the_reading_is_coherent ... P1b cells=1626 over_the_safe_bound=0 not_equal_when_unquoted=0
```

- P1a：3402 格（空闲 × 借着的 × 驱动三态 × 实际占着的六种读法 × 额度变量三档 × 配置额度三档，含读数自相矛盾的格），与定义式 0 格不同。
- P1b：1626 格，读数取自同一个假世界（别人占 0–15000、起没起设备、借的建了 0 / 一半 / 全部）：没有额度约束的格恰等于安全上界，有额度的格不超过它。即「已实际占的不会再算一遍，没建的不会漏算」在**读数一致**时成立。
- 判定：**一致**（读数一致时）。

**文字上的一处含糊（规则没说清）。** 文件头第 19 行写的是「F − 这个进程借了还没实际占的 − D（这个进程还没在这张卡上起设备时）」，第 19 行同一句又把「还没实际占的」定成「借着的连驱动那一份减实际占着的」。照字面，登记了没起设备时 D 被扣两次；代码只扣一次（`M − D − B`，promised 里含 D）。P1a 的定义式取「D 只扣一次」，与代码全等。这是文件头措辞与代码的差，不是代码错；建议主 agent 把文件头改成「D 在登记前另扣一份、登记后算在借着的里」。

**冲突 A：读数次序不一致，同一笔会被算两遍（单进程内）。**

两边整行并排：

- 定义要的（转述文件头口径，`crates/singlefs-harness/src/gpu_memory_ledger.rs:13-20`）：F 与实际占着的是同一刻的一对，M 才等于卡此刻对这个进程真能占的。
- 代码做的：先一次 `nvidia-smi` 读空闲与总量，再另起一次 `nvidia-smi` 读本进程占的：「let reported_by_bus = reported_cards_by_pci_bus();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1482`）、「let used_by_this_process = memory_used_by_this_process_on_each_bus();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1483`）。账的锁只锁借的人，不锁已经借到的那条线程去建缓冲。

两次 `nvidia-smi` 本机各约 30–40 毫秒（原样）：

```text
$ time nvidia-smi --query-gpu=pci.bus_id,index,memory.free,memory.total --format=csv,noheader,nounits >/dev/null
real	0m0.031s
$ time nvidia-smi --query-compute-apps=pid,gpu_bus_id,used_memory --format=csv,noheader,nounits > /dev/null
real	0m0.039s
```

空闲读在前、占的读在后，两次之间同一进程里别的线程把借的缓冲建起来：空闲读到的是建之前的、占的读到的是建之后的，那一份既在 F 里又在 occ 里，M 偏大。逆过来（先读占的、再读空闲）只会偏小。

模型量出的超出（`forward_probe.rs` 的 P2：同一个状态、两种读数次序，其余相同）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:13: P2 3000 6000 3000 | free-then-occupied (code order) | 7751 | 4751 | 3000
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:22: P2 worst_overshoot_of_the_code_order=3000
```

字段：别人占 3000、借着 6000、两次读之间建了 3000；代码次序给 7751，两次读之后真能借的是 4751，多借 3000 MiB；先读占的再读空闲的次序是 1751（偏小），一致读是 4751。

P3 起 8 条线程在同一本账上借、建、还（假的卡把借的量真占上，两次读之间隔 300 微秒，真机上隔的是上面的几十毫秒）。数「账自己的承诺被破」（借着的合计 + 一份驱动 + 别人占的 > 总量）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:56: P3 3000 FreeThenOccupied FreeThenDrop 2 | 384 816 | 0 0 | 10 2108
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:61: P3 coherent_and_freed_before_refund violations=0; code_order(free-then-occupied) promise_overflow_events_total=47
```

- 代码次序的 12 次线程整跑里，别人占 3000 的 6 次全有承诺被破（最大一次超 2108 MiB，47 次事件合计）；别人占 0 的 6 次是 0；读数一致、先还内存再销账的 6 次整跑 0 次。两次整个探针跑，代码次序格的事件合计是 38 与 47（线程调度不同）；方向一致。
- 别人占 0 的格里代码次序也是 0：因为配置额度取卡的总量时，容量被 `min(额度, M)` 封在总量上，把多算的盖住了。真实配置（`gpu-cards.env.example:13`「GPU_CARDS_BY_PRIORITY='local:0:8G peer:0:8G'」）里额度是 8G：大空闲卡上额度那一项先束住，F 那一项不是最小项，这一形不出现；**只在卡已经挤到 F 那一项成最小项时出现**。
- 现实里的触发条件：同一进程里有一条线程已借到、正在建缓冲，另一条线程同时来借。崩溃放量产品路径借的次序是串行的（第 ① 段草稿区借完、建判器，再借第 ② 段、单元校验和），GPU 对拍用例在 cargo test 默认并行线程下同进程并行借（`gpu_unit_checks.rs:1175` 一路）；那一条路的 OOM 兜底没接（已知项），撞了就是红。
- 判定：**冲突 A（单进程内、窄）**。
- 推翻它的现象：（a）卡池读数改成一次 `nvidia-smi` 同时读出空闲与按进程占的，或次序换成先占后空闲，P3 的代码次序格 0；（b）真机上两次读之间借到的缓冲在毫秒内不可能建起来（我只量了两次 `nvidia-smi` 的耗时，没量真机上建缓冲的时间）。

**冲突 B：已销账、卡上还没还的缓冲，被当成别的借据已建好的那一部分。**

- 两边：代码把「实际占着的」整个夹进 promised（借着的 + 驱动）再当成借着的已建好的那一部分：「Some(occupied) => occupied.min(promised_mebibytes),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:138`）。`GpuCardLease` 丢掉时只 Poll 一次再销账：「if let Err(error) = self.card.device().poll(wgpu::PollType::Poll) {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1071`），不等；wgpu 要等设备收拾才还给驱动。
- 确定性算例（P5）：别人 0，另一笔借着 4000 还没建，一笔已销账的 4000 还占着卡：账给 9751，卡上真能借的 5751，多借 4000：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:67: test p5_a_dropped_lease_still_occupying_the_card_is_counted_as_the_realization_of_a_live_lease_that_has_not_built_yet ... P5 room=9751 safe_while_the_stale_buffers_are_still_on_the_card=5751 overshoot=4000
```

- P3 里「读数一致、先销账后还内存」的 6 次整跑，3 次卡真被占超过总量（505、60、935 MiB）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:30: P3 0 Coherent DropThenFree 3 | 1172 28 | 1 505 | 0 0
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:46: P3 3000 Coherent DropThenFree 1 | 360 840 | 1 60 | 0 0
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:48: P3 3000 Coherent DropThenFree 3 | 397 803 | 1 935 | 0 0
```

- 要同时有：一笔已销账的缓冲还占着卡、另一笔借着而没建。产品路径一次只一笔借据，碰不到；GPU 用例并行线程下碰得到。`Poll` 之后卡上是否立刻还，**复核不了**（要真卡）。
- 判定：**冲突 B（窄；产品路径碰不到，GPU 用例并行时碰得到）**。
- 推翻它的现象：量到 `GpuCardLease` 丢掉之后的 `Poll` 总能在销账前把缓冲还给驱动；或账改成销账前读一次卡、等占着的降到这个进程借着的以下。

### K1-2　跨进程那一套删干净了吗

- 对账、取卡、判器派活、核对五份源码（`gpu_memory_ledger.rs`、`gpu_unit_checks.rs`、`crash_judge_dispatch.rs`、`crash_verify_gpu.rs`、`crash_amplification.rs`）grep `flock|thread::sleep|Condvar|wait_timeout|thread::park|/proc/|is_process_alive|process_alive|LONGEST_WAIT|ledger_directory|gpu-memory-ledgers`：5 行命中，全在 `crash_judge_dispatch.rs`（`:32`、`:878`、`:882` 是一次派活等卡算完的 120 秒上限，`:1090`、`:1092` 读 `/proc/self/status` 的常驻峰值），与借卡无关；`scripts/`、`.claude/gate.d/`、两份 env 样板里没有 `gpu-memory-ledgers`、`GPU_MEMORY_LEDGER`。账里只剩进程内的 `Mutex`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:694`、`:758`）。
- 借的路径：`book_on_one_of_these_cards` 与 `book_states_on_the_cards_with_room` 都是「读、判、记」在一次锁里，借不到当场交 `Err`，不等（`crates/singlefs-harness/src/gpu_memory_ledger.rs:695-720`）。
- 剩下的：**注释里还写着旧的「等满了」**，共 10 行（命令原样输出）：

```text
$ grep -rn "等满了\|等别的用例与别的 GPU 测试进程还" crates --include=*.rs | cut -d: -f1,2
crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1061
crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs:9
crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs:11
crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:6
crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:40
crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs:7
crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs:13
crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs:291
crates/singlefs-checker-tier/src/gpu_unit_checks.rs:811
crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:928
```

  还有函数名 `lease_card_waiting_for_room` 本身（已不等，文档自己认了：「/// （第 10x 批：只记这个进程借的，借不到不等）。」，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1153`）。
- 判定：**一致**（行为上删干净了）；**注释过时 10 处，函数名过时 1 处**，不影响行为。
- 推翻它的现象：在产品代码里 grep 出读写账本文件、`flock`、等待循环的任何一处（我的 grep 没有）。

### K1-3　竞态兜底：撞了显存之后每一条路走到哪里

逐路（读源码；这一格要真卡才能撞，**撞满显存的实机一遍我复核不了**）：

| 路 | 代码 | 结局 |
|---|---|---|
| ① 内存包装退 245 | 包装文件头：「245    显存不够：命令退了非 0，而它起止之间的内核日志里有显卡驱动报的显存不够」（`scripts/hosts/run-with-memory-cap.sh:26`）；「RUN_WITH_MEMORY_CAP_BREAK=gpuoomonsuccess  命令退 0 也查内核日志、查到显存不够就退 245」（`scripts/hosts/run-with-memory-cap.sh:237`）是弄坏开关 | 只在**命令退非 0**时才看内核日志。被 wgpu 接住、退 0 的那一路（下面 ②③）不会变 245，而且本来就报了本次未跑，不矛盾 |
| ② 第 ① 段建判器、判一块 | 范围推在这条线程上：「.push_error_scope(wgpu::ErrorFilter::OutOfMemory);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1989`）；工作 panic 也接住：「let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1990`）；两处调用 `:2018`、`:2078` | 出错发 `CardMessage::OutOfMemory`：「received.out_of_memory.get_or_insert(reason);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1932`）；写库线程把前面按序到的块照写进库；最后「if let Some(reason) = received.out_of_memory {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2718`）打本次未跑、交 `NotRunTheGpuMemoryWasShort`。一致 |
| ③ 第 ② 段核对 | 同一个函数包住 `verify_blocks_from_these_facts`：「let tally = match on_the_card_catching_out_of_memory(&card, "第 ② 段核对时", || {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5414`）。`crash_verify_gpu.rs` 里没有线程（grep `thread::|spawn|rayon` 0 命中），错误范围按线程分在 wgpu 30.0.1 里确是线程本地（`wgpu-30.0.1/src/backend/wgpu_core.rs:634` 的 `scopes: HashMap<thread_id::ThreadId, …>`） | 本次未跑。一致 |
| ④ 单元校验和对拍 | 文件头自己认没接：「单元校验和对拍几条线程一起派活，wgpu 的错误范围按线程分，一个范围接不住几条线程，这一段没接：」（`crates/singlefs-checker-tier/src/crash_amplification.rs:39`） | 默认错误处理函数 panic，整趟红。已知、不算打中 |
| ⑤ 起设备 | `started_card` 里 `start_the_card(...).ok_or_else(...)` 起不来交 `Err`：「let mut started = start_the_card(&card.discovered).ok_or_else(|| {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1114`），调用处直接 panic：「.unwrap_or_else(|reason| panic!("{test_name}：{reason}"))」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1288`）；`OnceLock` 把这个 `Err` 存下来，之后这个进程在这张卡上每次借都 panic | **冲突 C**：别的进程在登记之后、起设备之前占满卡，这一形（上下文建不起来）判成红，且这个进程之后这张卡一直红。错误信息自己写的就是「卡被别的程序占满」（`:1116`），却没走本次未跑 |
| ⑥ 每张卡此刻都借不出一个状态 | 见下 | **冲突 D** |
| ⑦ GPU 用例（对拍文件）| `lease_card_waiting_for_room` 借不到当场 panic：「panic!("{test_name}：{reason}（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）")」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1165`），5 处调用都在 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（`:2070`、`:2516`、`:2630`、`:2822`、`:3501`） | 已知（归删除线下一批）。**注意**：以前是等，现在是借不到当场红，不只是 OOM 那一形；并行线程同进程时，一条用例占着显存，另一条当场红 |

**冲突 D：每张卡此刻都借不出一个状态，产品路径判红，不报本次未跑。**

两边整行并排：

- 定义（G1 二、文件头与产品文件头）：「比此刻能借的大而空着时放得下，是借不到：不等，调用方按借不到走下一条」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:23`）；「不用（不装状态，它那一份留给后面的卡），一张都借不到就照对拍用例那一路打「本次未跑」那一行（同一个记号，原因写显存不够），」（`crates/singlefs-checker-tier/src/crash_amplification.rs:23`）。
- 代码：第 ① 段在借之前先算每张卡「按此刻能借的」装几个状态，容量是 0 的卡装不了：「let rooms_mebibytes = pool.card_rooms_mebibytes_now_of(&card_indices)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2230`）→「let states_in_flight_capacity =」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2242`）；全是 0 就不借：「let scratch = if capacities.iter().all(|capacity| *capacity == 0) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2596`），后面落到判红：「"没有一张卡装得下这条流的表（{} MiB）与一个状态的草稿区：看配置里每张卡的显存额度与卡上此刻的空闲显存",」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2636`）。
- 这里的容量 0 把「空着时也放不下」（该红）与「此刻别人占着放不下」（该本次未跑）并成一类。账的 `book_states_on_the_cards_with_room` 本来会正确分开这两种（`NoCardHasRoomNow` 对 `NeedLargerThanEveryCard`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:809-826`），但全 0 时账根本没被调到。
- 模型（P4，两张卡，别人占着，每张此刻借得出 2000、1000，一个状态连表要 3040）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:63: test p4_every_card_without_room_now_is_capacity_zero_before_the_booking_is_ever_tried ... P4 rooms_now=[2000, 1000] states_capacity_by_room_now=[0, 0] (空着时每张卡能借 13751 MiB)
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:64: P4 ledger_outcome=NoCardHasRoomNow need=3040
```

  账说「借不到」（本次未跑那一路），预检的容量却是 `[0, 0]`。预检那一行本身在 checker 档，我没跑：这一格的「预检会走到判红」是**读代码得出**，模型只证了「两个量不同」。
- 范围：只有起过卡池的进程（GPU 用例进程里跑产品路径的那两条：`crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs`）。分项驱动没起卡池，不碰。别的进程占着卡时，那两条用例判红而不是报本次未跑。
- 判定：**冲突 D（真漏）**。
- 推翻它的现象：在别的进程占满卡的状态下，跑 `the_product_path_judges_on_scratch_it_booked_on_the_card_pool`，输出是 `NOT_RUN_NO_GPU_ADAPTER … reason=显存不够` 而不是 `没有一张卡装得下这条流的表`（要 checker 档 + 真卡，**我复核不了**，交主 agent）。

**没找到「撞了显存判成通过」的形。** 接住 OOM 的三个落点（`(Ok(_) | Err(_), Some(error))` 一律 `Err`）都转本次未跑或红，没有一条走 `Ok`；`(Err(panic), None)` 照常往外抛（`crates/singlefs-checker-tier/src/crash_amplification.rs:1997`）。找不到不等于没有；推翻它的现象：真卡撞满后某一路返回绿。

## K2　只防 OOM、不压并发两头

### K2-1　单进程里账给的量会不会大于此刻真能用的

- **「现读之后、借之前卡被别人占了」算竞态兜底那一类。** 读（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1482`）与记（`crates/singlefs-harness/src/gpu_memory_ledger.rs:695-706`）在这个进程的一把锁里，别的进程不受这把锁管；这正是用户定的「两个进程同时读到同一份空闲」。判定：**一致，不算打中**。
- **读数一致时账给的量不大于真能用的**（K1-1 的 P1b：1626 格 0 格超出）。
- 单进程里确实给大了的两形，都是前面量到的：**冲突 A**（空闲与占的分两次读，P2 超出 3000 MiB，P3 事件 47）、**冲突 B**（已销账未还被当成别的借据已建好，P5 超出 4000 MiB）。它们不是用户让漏的那一形（同一个进程里的线程竞态，不是两个进程）；兜底路径：产品路径上有 OOM 接住（K1-3 ②③），GPU 用例上没接（K1-3 ⑦）。
- 起设备时驱动实际占的比账留的少，是**偏保守**的方向（不会给大）：账每张卡留 `MEBIBYTES_LEFT_FOR_THE_DRIVER` = 2560 MiB（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:27`），本机一次读到起了设备的 wgpu 测试进程在 `nvidia-smi` 里占 211 MiB（原样）：

```text
$ nvidia-smi --query-compute-apps=pid,gpu_bus_id,used_memory,name --format=csv
pid, gpu_bus_id, used_gpu_memory [MiB], process_name
3870820, <PCI 总线号，原样输出里是 00000000 起头那一串，这里省去免被门禁当成时刻>, 211 MiB, /home/fy5090/.local/share/singlefs/work/m3-r1/prove-10x/target/debug/deps/crash_amplification_product_path_books_its_scratch_on_the_card_pool-998e6918fe5d874d
```

  这条同时坐实一件事：**Vulkan（wgpu）进程会被 `--query-compute-apps` 列出**，账读「本进程实际占的」那条路（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:137-168`）读得到数，不是永远 `None`。读不到才会退回「借着的当一点都没建」的偏保守算法。211 是那一刻的一个读数（该进程当时建了多少缓冲不知道），不当 2560 的量过值；「2560 没上卡量」是主 agent 已列的已知项。
- 判定：**一致（读数一致时）；冲突 A、B（单进程、窄）**。
- 推翻它的现象：P1b 换成真卡读数后某格超出；或 A、B 在真机上被前面写的两个「推翻现象」推翻。

### K2-2　放得下却借不到、白等、白红；借不到的卡重排之后会不会压到一张

- **重排不压到一张**（P6，两张卡，每张最多装 10，一共 10 个状态）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/forward_probe-run.log:69: P6 both rooms ample: states_booked_per_card=[Some(10), None] expected=[Some(10), None]
```

  另三格同一份输出里（`forward_probe-run.log:70-72`）：卡 0 此刻只放得下 4 个→`[Some(4), Some(6)]`；卡 0 没空→`[None, Some(10)]`；两张都只放得下 4 个→`[Some(4), Some(4)]`。「前一张放满了才用下一张」（用户 2026-09-29 定）与「借不到的卡不装状态、留给后面的卡」都成立。两张卡都有空时全压第一张是**设计**（`crates/singlefs-harness/src/gpu_memory_ledger.rs:4-5` 文件头写明）。
- **借不到的卡这一趟不再试**：`book_states_on_the_cards_with_room` 在锁里每张卡只试一次（`crates/singlefs-harness/src/gpu_memory_ledger.rs:762`）。别人此刻占着、下一秒还回的卡，这一趟不用；岔路单 G1 一写的是「借不到的卡下一笔再试」（`research/prompts/m3-batch10t-code-r1-forks.md:9`），一趟之内不重试，一致。
- **放得下却借不到的形**：
  1. 冲突 D：每张卡此刻都借不出一个状态时判红（K1-3）。
  2. 本进程里别的借据占着、此刻借不到的 GPU 用例：当场 panic（K1-3 ⑦）。以前是等，现在当场红；**不压并发这一头，对并行同进程的 GPU 用例是倒退**，但那几处在对拍文件、已转删除线（主 agent 已列）。
  3. 销账后卡上还没还（冲突 B 的另一面）：空闲读低，账少给，借不到（偏保守）；以前会等，现在当场本次未跑。`Poll` 只一次，不等。
  4. 没配显卡优先级时 `M` 的 3/4 做上界：一张空闲 15849 MiB 的 5060 Ti，没配额度，3/4 再减驱动那一份是 `15849/4*3 − 2560 = 9326`，要 12000 MiB 的用例判成「比空着时能借的都大」（红），而卡上物理放得下。这是 10v 批主 agent 定的口径（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:312` 的 `free_memory_mebibytes / denominator * numerator`），一致，不算打中。
- 判定：**重排一致；放得下借不到：冲突 D（真漏）、⑦（已知）、3、4（偏保守，设计）**。
- 推翻它的现象：P6 在真卡读数下某格把状态压到一张；或 D 在真卡上走了本次未跑。

## K3　测试判别力

### K3-1　有卡用例新判据能不能分出「借了」与「借了又提前还」

- 判据的取法：①段在判器建好那一刻问账：「work.scratch_lease_identifiers[position].map(|lease_identifier| {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2035`），随后调池里按借据号查账里那张表（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1326-1331`，`GpuMemoryLedger::is_the_lease_on_the_books`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:872`）；单元校验和与第 ② 段在**做完**那一刻问：「lease_on_the_books_after_its_work(&buffers_held_until_checked),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4811`）。用例断言 `Some(true)`。
- 它**不是**上一轮判决点名的「产品自己填的字段」（上一轮：`research/prompts/m3-batch10t-code-r1-main-verification.md:33`）原样：现在查的是账里按借据号的那张表，与产品手里那个 `lease` 变量是两份记录；借据号来自原子计数器，一个号只发一次（`crates/singlefs-harness/src/gpu_memory_ledger.rs:518-523`），借了又还、再借一笔也对不上同一个号。
- 分得出：变异行「借了又在判之前还掉」把 `scratch.leases.push(lease);` 换成 `drop(lease);`（`crates/mutations.tsv:3141`），判器建好时账上那一笔没了，字段是 `Some(false)`，断言红；单元校验和与核对两行把 `lease` 换成 `None`，字段是 `None`，也不等于 `Some(true)`。**这三行我没法跑（checker 档），只读出判别链**。
- 分不出的：①段只在**判器建好**那一刻采样一次，之后到判完之间借据被还掉，不红（代码里借据放在 `scratch.leases`、活到函数收尾，今天没有这样的写法，但判据不拦它）；单元校验和与核对是采在做完之后，没这个洞。
- 判定：**一致（窄缺口：①段只采建好那一刻）**。
- 推翻它的现象：跑那三行变异，任一行绿；或写出一个在建好之后、判完之前还借据的改法，那条用例仍绿。

### K3-2　扫源码检查堵了六种绕法之后，还有哪种写法绕得过

读 `attribute_marks_test_code`（`crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:377-386`）、起卡名字表与 `scan_the_library_sources`。仍绕得过的：

1. **`#[cfg_attr(test, test)]`、`#[cfg_attr(test, cfg(...))]` 这类**：`attribute_marks_test_code` 里 `trimmed.strip_prefix("cfg")` 之后剩下 `_attr(test, test)`，`predicate.starts_with('(')` 为假，不当测试代码，里面的起卡调用不扫。罕见写法，读代码得出，没跑（checker 档）。
2. **文件头自己认的**：`include!`、`#[path = "…"]`、`benches/`、`examples/`、产品代码里包一层再从测试调（「不扫的（交 review）：」，`crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:14`）。
3. **拿到的卡在借据销账后继续用**：`GpuCard` 是 `Clone`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:77`），`lease.card().clone()` 可以活得比借据久；文本扫描看不见（产品路径自己就是这样写的：`crates/singlefs-checker-tier/src/crash_amplification.rs:5396` 的 `Arc::new(lease.card().clone())`，靠借据另放在 `leases` 里活到收尾保证）。本检查只扫「怎么拿到卡」，不扫「用多久」。
4. **不在七份「只经卡池」名单里的测试文件**：`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 仍经 `usable_cards_or_report_not_run`（`grep -rln` 命中它与 `src/crash_judge_dispatch.rs`、`src/gpu_unit_checks.rs`）拿不入账的卡；它归删除线下一批，主 agent 已列。
5. wgpu 的 `util::initialize_adapter_from_env_or_default`（`wgpu-30.0.1/src/util/init.rs`）不带三个名字就能拿到适配器，**但要起设备仍得写 `request_device`**，名字表里有，所以这一条**堵住了**，不算绕过。

- 判定：**一致，仍有 1–4 的洞；1 是名字上的真漏洞（罕见），2、4 已知，3 要另一类检查**。
- 推翻它的现象：对 1 写一个 `#[cfg_attr(test, test)] fn x() { usable_gpu_cards(); }` 放进 `src/`，扫描照绿（要 checker 档跑，我没跑）。

### K3-3　变异行能不能拦住这一批任何一处被改回去；删掉的 20 行有没有该留的

**我在 harness 档自己补了 14 处变异**（冻结副本逐个换一处、跑 `cargo test -p singlefs-harness --lib --offline gpu_memory_ledger`、还原；脚本 `mutate-probe.py`，输出在 `mutate-probe-run.log`，基线 27 条全绿）：

```text
research/prompts/m3-batch10x-code-r1-sonnet-model/mutate-probe-run.log:6: C12 configured quota also gets the 3/4 default | exit=0 | ['test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s'] | failed_tests=0
research/prompts/m3-batch10x-code-r1-sonnet-model/mutate-probe-run.log:8: C14 OOM reason wording not 显存不够 | exit=0 | ['test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s'] | failed_tests=0
research/prompts/m3-batch10x-code-r1-sonnet-model/mutate-probe-run.log:13: C19 same-card-twice assertion removed | exit=0 | ['test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s'] | failed_tests=0
research/prompts/m3-batch10x-code-r1-sonnet-model/mutate-probe-run.log:14: C20 card count mismatch only checks less-than | exit=0 | ['test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s'] | failed_tests=0
```

- 14 处里 10 处被 1–12 条用例抓到（C1、C2、C5、C7、C8、C13、C15、C16、C17、C18），4 处没抓到：
  - C12（配置额度也套 3/4 默认）：**等价变异**，`min(额度, M) − D` 已是另两项的最小，不算漏；
  - C14（OOM 那一句把「显存不够」改成「内存不够」）：`reason_for_running_out_of_gpu_memory`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:437`）整个 workspace 里没有一条用例读它，任何人改这句话都不红。下游（分项驱动、门禁）是否按「显存不够」认这一行，我没查全；
  - C19（同一张卡给两次的断言去掉）、C20（读到的卡数比池里多不拒，把 `!=` 换成 `<`）：守卫类，没有用例。第 10x 批已有的行 `readings.len() != books.len()` → `false &&`（`crates/mutations.tsv` 里那一行，主 agent 报 23 行 harness 变异都抓到，我没复跑）与这一处的 `<` 版不同，`<` 版抓不到。
- C1（`occupied` 不加回、M 只取 F，对应删掉的「估计只取现读的空闲」今天的样子）被 1 条用例抓到，但**变异表里没有对应的行**：删掉的 20 行里「第10t批 显存账 K4-3 估计只取现读的空闲（卡被订满时后起的进程判成放不下）」今天还有对应的东西（`crates/singlefs-harness/src/gpu_memory_ledger.rs:138` 那一行的另一个方向），该有一行。

**删掉的 20 行逐行对今天的代码**（`research/prompts/_m3-batch10x-code-r1-diff.md` 的 `mutations-delete.txt` 一节，我按行读）：

| 去向 | 行 | 数 |
|---|---|---|
| 概念今天没有了（跨进程账、等满最长等待、判活、默认工作根、僵尸、开账上界登记驱动那一份、取卡时每张都登记） | 还了不叫醒等着借的；第10s批 4 行；第10t批 K4-2 2 行；第10w批 登记驱动那一份要每张都登记上才交回 | 9 |
| 同一件事在 10x 新行里重加了 | 挑卡从优先级最低的那张起；比每张卡都大的也去等；第10v批 产品路径借草稿区等满了判红；第10w批 一张卡借到了也等借不到的那张；真账本上借草稿区一张借到了也等被订满的那张；一张都没借到而空着时放得下的也判红（不等）；估计在压与借之间变小也不在锁里重压 | 7 |
| 「估计」那一套换成每次现读，由新行「借的时候不照现读」等盖住 | 估计没有上界；估计没有下界；估计照抄旧的、不夹进现读的区间 | 3 |
| **今天还在、该留的** | 第10t批 K4-3 估计只取现读的空闲（卡被订满时后起的进程判成放不下）：今天的样子是 `Some(occupied) => occupied.min(promised_mebibytes),` 换成 `Some(_) => 0,`（上面 C1），有用例抓、没有行 | 1 |

**10x 改的产品路径上，改回去不红的位置**（读代码，没有用例让这些行走到；要真卡撞 OOM，或要卡被别的进程占着；checker 档我没跑）：

1. `on_the_card_catching_out_of_memory` 本身与它在 `crash_amplification.rs:2018`、`:2078`、`:5414` 三处的调用：变异行 `crates/mutations.tsv:3152` 只改「写库线程收到 `OutOfMemory` 当成拒判」那一处，对应的 checker 档单测只喂假消息（`crates/singlefs-checker-tier/src/crash_amplification.rs:3745`）。把三处调用换成直接 `Ok(work())`，没有用例变红。主 agent 已列（「`on_the_card_catching_out_of_memory` 没有用例」），不算打中。
2. 收到 OOM 之后转本次未跑那几行（`crates/singlefs-checker-tier/src/crash_amplification.rs:2718-2723`、`:2328`）：改成别的结局，没有用例红。同上。
3. `cards_of_the_pool_with_what_each_lends_now`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2230-2262`）把「此刻能借的」换回「起池那一刻的空闲」：空闲的卡上两者相同，两条产品路径用例仍绿；这正是上一轮攻方 S5 那一形，没有变异行。**这一条是新报的**：要一条「别的进程占着卡」的 checker 档用例或一行变异才分得出。
4. 冲突 D 那一处预检 `capacities.iter().all(...)`：无用例。

- 判定：**harness 档：10 行以上能拦、3 处洞（C14、C19、C20）、1 行该留没留（C1）、1 处等价；checker 档：OOM 兜底整路与 S5 改回去都不红**。
- 推翻它的现象：把上面 1–3 任一处改回去后跑对应 checker 档用例，红了。

## K4　与删除线 S 的交界

- 10x 在 `crash_amplification.rs` 里的 hunk（`_m3-batch10x-code-r1-diff.md` 的 `@@` 行）按旧文件行号：`-14`、`-33`、`-53`、`-1746` 起往后。S 的落点：我没有 S 的 diff，按 `versions_without_a_file` 在这份文件里出现的位置判断（`grep -n` 输出 `168 177 186 451 670 2570 5516`）：`CrashFlow`（`crates/singlefs-checker-tier/src/crash_amplification.rs:177` 的 `pub versions_without_a_file`）、判器另读的流程输入 `JudgeInputsOfTheFlow` 的文档（`:451`）与进计划哈希的那段文本（`:670`）、两处建表（`:2570`、`:5516`）。**10x 没有任何一个 hunk 落在旧行 `-53…-1746` 之间，也没落在建表那两处的行上**，即没动 `CrashFlow`、复用键、块键、计划哈希文本。
- 10x 与 S 在同一函数里挨着的两处：
  1. 判器建表：S 的 `build_judge_tables_against_the_version_table(&judge_flow, flow.versions_without_a_file)`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2568`）10x 没碰；10x 在它之后拿同一份 `tables` 算每张卡的容量：「let run_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2590`）。借的量、容量、判器建的草稿区都用这份 `tables.mebibytes()`，S 加进版本表的行已在里面。一致。
  2. 只排计划那一趟：S 的 `flow.versions_without_a_file` 在 `print_the_placement`（建表那一处）10x 没碰；10x 的「let local_cards = card_capacities_of_this_run(cards_taken_from, cards, &tables)?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:5526`）用同一份表。一致。
- 复用键 / 块键：10x 的写库路径没动 `store_judged_block` 的键。中途 OOM 停下，之前按块序写进库的块下一趟按同一把键（含 S 的版本表上界）复用；这一条是读代码，**中途 OOM 后续跑**要真卡，我复核不了。
- 「没有文件的那几版」让表变大 → `tables.mebibytes()` 变大 → 每张卡容量里装得下的状态数变小、更容易落到容量 0：见冲突 D（S 的流更可能撞到那一处判红）。这是 10x 的 D 与 S 的叠加，不是互相踩。
- 判定：**一致，没有互相踩到**；叠加只放大冲突 D。
- 推翻它的现象：把 10x 的 diff 对着 S 合入前的文件 `git diff` 看到有 hunk 落在 S 的行上（我读的是合后文件与 10x diff 的 `@@` 行号，没有 S 合入前的文件）。

## 要主 agent 跑的命令（checker 档 + 真卡；子 agent 不跑）

- 坐实冲突 D：先让另一个进程占住被测那几张卡的几乎全部空闲显存（让每张卡此刻借不出一个状态），再跑（工作目录仓根，`SINGLEFS_HEAVY_TESTS=user-request` 前缀，内存上限 16G，线程 10）：
  `bash scripts/hosts/capped.sh 10 bash scripts/hosts/run-with-memory-cap.sh 16G cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_amplification_product_path_books_its_scratch_on_the_card_pool -- the_product_path_judges_on_scratch_it_booked_on_the_card_pool`
  看输出里是 `NOT_RUN_NO_GPU_ADAPTER … reason=显存不够…`（定义要的）还是 panic 带「没有一张卡装得下这条流的表」（我读出的）。
- 坐实冲突 C：同上，让占卡的进程在登记之后、起设备之前才把卡占满，看是不是 panic 带「上起不了设备」。要时序控制，不好做；可以只读代码判。
- 坐实 K3-2 的 `cfg_attr`：在 `crates/singlefs-checker-tier/src/` 临时放一个 `#[cfg_attr(test, test)] fn x() { let _ = usable_gpu_cards(); }`，跑 `--test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start`，看是否仍绿；跑完删掉。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 式子取的量 | 一致（读数一致时）；冲突 A、冲突 B（单进程内、窄） | P1a 3402 格对定义式 0 差，P1b 1626 格不超安全上界；空闲与占的分两次读（`gpu_unit_checks.rs:1482-1483`）会多借，已销账未还会被当成别的借据已建好 |
| K1-1 文件头措辞 | 规则没说清 | 文件头第 19 行照字面 D 扣两次，代码只扣一次 |
| K1-2 跨进程删干净 | 一致 | 产品代码里没有账本文件、flock、判活、等待；注释里还有 10 处「等满了」、函数名 `lease_card_waiting_for_room` 过时 |
| K1-3 兜底各路 | ②③ 一致；④⑦ 已知；冲突 C；冲突 D | 第 ① 段、第 ② 段接住 OOM 转本次未跑；起设备失败判红；每张卡此刻都借不出一个状态时预检判红而不是本次未跑 |
| K1-3 判成通过 | 没找到 | 接住 OOM 的三处都不走 `Ok`；找不到不等于没有 |
| K2-1 单进程给大 | 竞态那一类一致；冲突 A、B | 现读到借之间被别的进程占属兜底那一类；A、B 是进程内线程的形 |
| K2-2 放得下借不到、重排 | 重排一致；冲突 D；⑦ 已知；3/4 与驱动 2560 是设计、偏保守 | P6 四格没有压到一张；GPU 用例借不到由「等」变「当场红」 |
| K3-1 判据分得出 | 一致（窄缺口） | 借了又还掉分得出；①段只采判器建好那一刻，建好之后到判完之间还掉不红 |
| K3-2 六种绕法之后 | 一致，仍有洞 | `#[cfg_attr(test, test)]` 绕得过（读代码）；文件头自己认的几种；`GpuCard` 克隆出借据之外；七份名单之外的文件 |
| K3-3 变异行 | harness 档 3 处洞 + 1 行该留没留；checker 档 OOM 兜底整路与 S5 改回去不红 | 14 处自补变异抓到 10 处；C14、C19、C20 没抓、C12 等价；删掉的 20 行里 1 行（C1）该留 |
| K4 与 S 的交界 | 一致，没有互相踩到 | 10x 的 hunk 不在 `CrashFlow`、计划哈希文本、建表那几处的行上；S 的流让表变大，更容易撞到冲突 D |

## 没做什么

- checker 档的任何测试（含 GPU 用例、`crash_amplification` 的单测、扫源码检查本身）没跑；凡标「读代码」的格都没经运行证明。
- 真卡撞满显存的一遍没做：wgpu 报 OOM 后各路的实际结局、`Poll` 后缓冲是否立刻还给驱动、两次 `nvidia-smi` 之间真机上缓冲建多快，都没测；只量了两次 `nvidia-smi` 各约 30–40 毫秒。
- P3 的线程模型用 `sleep` 把读之间的间隔缩成 300 微秒、把建缓冲拖慢，是模型的取样点，不是真机比例；同一份跑两次事件数 38 与 47，不稳定，只当方向，不当数。
- 没复跑主 agent 报的 23 行 harness 变异与 14 行 checker 档变异；没判已知项（单元校验和对拍几条线程、GPU 用例自己的 OOM、extent 偏移对拍、`started_card` 侧变异、`on_the_card_catching_out_of_memory` 用例、驱动固定开销 2560）；上面提到它们的地方只为分清「已知」与「新报」。
- 没替主 agent 采纳或出判决；没碰仓里的任何文件（只写了本报告与模型目录）。
