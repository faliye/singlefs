# 根环实现第 10y 批代码轮第一轮：正推腿报告（2026-10-07）

分到这条腿的：正文第三节 K1 至 K4 全部格，立场正推（从 `crates/` 今天的实现与已定条款推「应该是什么」，再与被判对象比）。
被判对象的代码读冻结副本 `~/.local/share/singlefs/work/m3-batch10y-code-r1/tree/`；下面引代码都是冻结副本里的路径与行号，引 kb 是仓里 `.claude/kb/` 的路径与行号（行号都是现查的）。

## 零、开工核对与跑法

冻结副本对快照清单：

```
$ cd ~/.local/share/singlefs/work/m3-batch10y-code-r1/tree; sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch10y-code-r1-snapshot/crates-sha256.txt 2>&1 | grep -vc ': OK'
0
```

跑法：草稿副本 `~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/repo`（从冻结副本 rsync，排除 /target、/.git、/research/target），编译目录 `~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/target`（交回前删）。
所有 cargo test 经 `nice -n 19 bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh <4G 或 6G> bash <脚本>`；只跑 harness 档（`singlefs-harness` 的 `--lib` 与我自己加在草稿副本 `tests/` 下的探针），没碰 checker 档包的测试。
开跑前 `python3 research/scripts/host-resources.py` 看到（原样）：

```
  线程：32 核，此刻在用约 28.7 核（loadavg 19.03 9.66 14.04），可用 3 线程
  线程：32 核，此刻在用约 24.3 核（loadavg 135.12 89.1 51.03），可用 8 线程
```

（第一行是第一次看，第二行是起 K4 扫描之前看；没有 qemu-system、vm-bench、e152、fio 在跑。）
模型源码、原始输出、变异脚本都在 `research/prompts/m3-batch10y-code-r1-sonnet-model/`（`src/` 与 `out/`，README 写了重跑的办法）。

## K1　判决第三节各条做到没有

### K1 第 1 条（K1-1）　四形还在不在、是不是同一个量、认不得的输出怎么办

**① 同一份报告。** 每借一次只取一份 `nvidia-smi -q -d MEMORY,PIDS` 报告，空闲与本进程占用都从这一份读：
`crates/singlefs-harness/src/gpu_memory_ledger.rs:636`：「pub const NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS: [&str; 3] = ["-q", "-d", "MEMORY,PIDS"];」；读卡的闭包里只有一处取报告，`crates/singlefs-harness/src/gpu_memory_ledger.rs:792`：「let report = run_the_report()?;」。
旧法（`--query-gpu` 与 `--query-compute-apps` 分两次）在产品代码里没有了（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:3127` 的测试文件里还留一处 `--query-compute-apps`，那份文件在删除线 U 手里）：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:242` 那处 `nvidia-smi` 调用只剩发现卡用的 `--query-gpu=pci.bus_id,index,memory.free`，不进账。

**② 本机真跑，新解析与 `--query-gpu=memory.free` 是不是同一个量。** 先看这台机器的原样输出（只读，没占卡；`nvidia-smi -q -d MEMORY,PIDS` 与 `--query-gpu` 各一遍，连跑两遍 `-q` 逐字节相同）：

```
$ nvidia-smi -q -d MEMORY,PIDS > smi-q-1.txt ; nvidia-smi --query-gpu=pci.bus_id,index,memory.free,memory.total,memory.reserved,memory.used --format=csv,noheader,nounits > smi-query-1.txt
$ cat smi-q-1.txt   （节选两张卡的 FB 与 Processes 段，全文在 model/out/smi-q-1.txt）
GPU 00000000:01:00.0
    FB Memory Usage
        Total                                          : 32607 MiB
        Reserved                                       : 497 MiB
        Used                                           : 2 MiB
        Free                                           : 32110 MiB
    ...
    Processes                                          : None
GPU 00000000:10:00.0
    FB Memory Usage
        Total                                          : 16311 MiB
        Reserved                                       : 462 MiB
        Used                                           : 2 MiB
        Free                                           : 15849 MiB
    ...
    Processes                                          : None
$ cat smi-query-1.txt
00000000:01:00.0, 0, 32110, 32607, 497, 2
00000000:10:00.0, 1, 15849, 16311, 462, 2
```

再把这一次真跑出来的报告现喂给新解析 `card_memory_readings_of_one_nvidia_smi_report`（探针 `src/sonnet_probe_nvidia_smi_report.rs` 里的 `live_report_free_equals_query_gpu_memory_free`：在测试里前后各跑一遍 `--query-gpu`、中间跑 `-q`，三次相同才认；命令经内存包装，退出码 0）：

```
test live_report_free_equals_query_gpu_memory_free ... PARSED [CardMemoryReading { free_mebibytes: 32110, total_less_reserved_mebibytes: 32110, occupied_by_this_process: NotListed }, CardMemoryReading { free_mebibytes: 15849, total_less_reserved_mebibytes: 15849, occupied_by_this_process: NotListed }]
QUERY  [("01:00.0", 0, 32110, 32607, 497), ("10:00.0", 1, 15849, 16311, 462)]
LIVE_MATCH attempt=1 cards=2
```

测试里的断言：每张卡 `free_mebibytes == memory.free`、`total_less_reserved_mebibytes == memory.total − memory.reserved`、`occupied_by_this_process == NotListed`，三条都过（`test result: ok. 2 passed`）。
所以解析出的空闲与 `--query-gpu=memory.free` 是同一个量（这台机器两张卡，逐 MiB 相等）；空卡上能用的 = 总量 − 保留，在这两张空卡上也等于报的空闲（32607 − 497 = 32110，16311 − 462 = 15849）。
推翻它的现象：驱动版本换了、`FB Memory Usage` 下的 `Free` 与 `memory.free` 取法不同（本机驱动 595.84；另一台机器与别的驱动**没读**）。

**③ 解析认不得的输出：报错，不悄悄当 0，也不报成本次未跑。** 同一个探针里把真报告改坏几种、再喂给解析（`unrecognised_outputs_are_errors_not_silent_zeroes`，原样）：

```
CASE 空串: Err("nvidia-smi 的报告里没有总线 01:00.0 那张卡")
CASE 乱码: Err("nvidia-smi 的报告里没有总线 01:00.0 那张卡")
CASE 缺 Reserved（老驱动的样子）: Err("nvidia-smi 的报告里总线 01:00.0 那张卡的 Reserved 读不出")
CASE Free 写 N/A: Err("nvidia-smi 的报告里总线 01:00.0 那张卡的 Free 读不出")
CASE Free 缺单位: Err("nvidia-smi 的报告里总线 01:00.0 那张卡的 Free 读不出")
CASE 没有 FB 小节: Err("nvidia-smi 的报告里总线 01:00.0 那张卡的 Total 读不出")
CASE 点名的卡不在报告里: Err("nvidia-smi 的报告里没有总线 99:00.0 那张卡")
CASE 进程号认不出: Err("nvidia-smi 的报告里一行进程号认不出：\"        Process ID                                     : abc\"")
CASE 本进程占的量 N/A: Ok([CardMemoryReading { free_mebibytes: 32110, total_less_reserved_mebibytes: 32110, occupied_by_this_process: NotListed }])
```

九种里八种是 `Err`（报告里点名的卡缺、`Total`、`Reserved`、`Free` 读不出，一行进程号认不出）。只有「本进程那一行 `Used GPU Memory` 读不出」落进 `NotListed`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:764`：「None | Some(Err(())) => OccupiedByThisProcess::NotListed,」），那一支只会少借（借着的当一点都没建）。
`Err` 之后各出口：卡池借卡一路 panic 判红（`BookingRefusal::CardMemoryUnreadable` → `lease_card` 里 panic），产品路径经 `scratch_booking_refusal_outcome` 落 `Red`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:609-611`）。就是说：认不得 = **判红**，不是本次未跑，也没有悄悄当 0。
缺 `Reserved` 行的老驱动（我把本机报告里那一行删掉造的）同样落 `Err("…Reserved 读不出")`，即那种驱动上卡放得下也判红：这一条是推的（没有老驱动可验），旧法 `--query-gpu=memory.free` 不要这一行。
推翻它的现象：在真有的驱动上 `Reserved`/`Free` 的行名不是这两个字。

**④ 这几种「认不得」的 Err/当 0 在测试里没有钉（K3 同一处）：** 我把它们改回「悄悄当 0」，harness 档整个 `gpu_memory_ledger::tests` 照绿（见 K3）。

**⑤ 四形逐个看（读码，数与方向）：**

| 形（第 10x 批判决 K1-1） | 新代码 | 判 |
|---|---|---|
| 空闲与占用分两次读，两次之间别的线程建上的一笔算两遍 | 一份报告，见 ① | 不在了 |
| 本进程没走卡池的设备被当成借据已建上的 | `AlsoOutsideTheBooks` 分不出时借着的当一点都没建、只会少借；登记在 `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:383`：「BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL」 | 在，**有一个小窗口**，见下 |
| 「空着时能借的」拿总量当上界 | `total_less_reserved_mebibytes`，本机两张空卡逐 MiB 等于空闲，见 ② | 不在了 |
| 已销账、卡上还没还的缓冲当成别的借据已建好的 | 记 `returned_possibly_still_on_the_card_mebibytes`（上界），读数照 `occupied − 驱动` 往下压 | 这一形不在了，但**记法本身有偏**，见 K2-2 的 P1 |

窗口（推的，没造出来）：`start_the_card` 先 `request_device`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:341`：「let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {」），编管线，**之后**才登记「这个进程在这条总线上起过不走卡池的设备」（383）。设备已占上卡、登记还没落的这一段里，同一个进程另一条线程借卡池读报告，会把那个设备的占用当成借据已建好的（`OnlyThroughTheBooks`，封顶到许出去的量），多借最多一份驱动那一份。只有一个进程里既有 `usable_gpu_cards`/`gpu_cards_by_priority` 起的设备、又有卡池借卡并行时才碰到；窗口宽度（编管线那几十毫秒到几秒）没量。
推翻它的现象：造出这一形，两线程交错，卡池那一笔借到的比 `空闲 − 未建的许出去的` 多。

### K1 第 2 条（K1-3）　Out of Memory 都接住了吗

逐处读码（没有卡可跑 checker 档，这一格全是读码加 wgpu 源码核对；跑不动的写在「没做什么」）：

| 处 | 代码 | 判 |
|---|---|---|
| 起设备 `request_device` | `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:203`：「if error.to_string() == wgpu::wgc::device::DeviceError::OutOfMemory.to_string() {」→ `CardStartFailure::OutOfGpuMemory`；卡池调用方报本次未跑（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1475`：「Err(CardStartFailure::OutOfGpuMemory { reason }) => {」） | 接住 |
| 编单元校验和的管线 | 同一条线程上推 OOM 范围（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:362`：「let out_of_memory_scope = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);」），弹出时有错就是 `OutOfGpuMemory`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:375`：「if let Some(error) = pollster::block_on(out_of_memory_scope.pop()) {」） | 接住 |
| 判器内核在另起线程上编 | 每个设备装没接住错误的回调，先记 OOM 再 panic（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:178`：「wgpu::Error::OutOfMemory { .. } => {」，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:183`：「panic!("wgpu error: {error}\n")」）；卡线程上的活 panic 后按设备上记的认（`crates/singlefs-checker-tier/src/crash_amplification.rs:2016`：「(Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },」） | 接住（经「记在设备上 + 活 panic」） |
| 只排派活计划那一趟 | 建判器与试判整段包进 `on_the_card_catching_out_of_memory`（`crates/singlefs-checker-tier/src/crash_amplification.rs:5776`：「let trial = on_the_card_catching_out_of_memory(」），出 `Err` 打本次未跑行、交回 `NotRunTheGpuMemoryWasShort` | 接住 |
| 设备丢失 | 每个设备装丢失回调（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:358`：「device.set_device_lost_callback(move |reason, message| {」）；下一次借这张卡先看留着的设备丢没丢（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1282`：「|started: &GpuCard| started.device_condition().lost_reason().is_some(),」），丢了就重起，不缓存 `Err`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1322`：「*started = Some(device.clone());」只在 `start()` 成了才写） | 接住 |
| **单元校验和对拍（第 ① 段由 CPU 判时，几条线程一起派活）** | 只借缓冲，对拍本身不包：`crates/singlefs-checker-tier/src/crash_amplification.rs:1612`：「check_unit_checksums_on_gpu(card, &borrowed)」不在 `on_the_card_catching_out_of_memory` 里（该函数全文件只有 4 处调用，行 2078、2140、5577、5776，没有这一处）；文件头第 45 行自己写了：「单元校验和对拍几条线程一起派活，那一段没包进接 Out of Memory 的那一层」 | **没接住**：那里的 OOM 还是 panic 判红（文件头认的残差，不在判决 K1-3 点名的几处里） |

`request_device` 那一支的字符串比较我对了 wgpu 源码（`Cargo.toml` 锁 `wgpu =30.0.1`），原样：

```
$ grep -n 'Not enough memory left' wgpu-core-30.0.1/src/device/mod.rs
347:    #[error("Not enough memory left.")]
$ grep -n 'RequestDeviceErrorKind::Core(error) => error.fmt(_f)' wgpu-30.0.1/src/api/device.rs
828:            RequestDeviceErrorKind::Core(error) => error.fmt(_f),
$ grep -n 'map_host_device_oom_err' wgpu-hal-30.0.1/src/vulkan/adapter.rs | head -2
2902:                .map_err(super::map_host_device_oom_err)?
```

`wgpu` 的 `RequestDeviceError` 的 Display 就是 `wgpu_core` 那个错的 Display，`DeviceError::OutOfMemory` 的 Display 是「Not enough memory left.」，两边相等的比较成立。名字叫 `map_host_device_oom_err`：主机内存不够也被当成这个错，起设备时主机 OOM 会被说成「显存不够」（推的，没造）。

**回调接住之后这一块的结局：** 报 OOM 的那条卡线程置 `stop`、发 `CardMessage::OutOfMemory` 就 `return`，这一块不交去写库（`crates/singlefs-checker-tier/src/crash_amplification.rs:2090`、`2146`：「let _ = sender.send(CardMessage::OutOfMemory(reason));」）；写库线程按块序把已经到的块写进库，收完后整趟报本次未跑（`crates/singlefs-checker-tier/src/crash_amplification.rs:2830`：「if let Some(reason) = received.out_of_memory {」之后打本次未跑行并交 `NotRunTheGpuMemoryWasShort { tally, reason }`）。结局是**本次未跑**，判完且按块序到齐的块照写进库；判完而前面还有块没到的（`waiting` 里的）那几块不写（只丢工，不丢正确性）。

**设备丢了重起时，丢之前借的账会不会算错：** 账（`books`）不动：借据还记在原卡上，重起只做 `record_the_device_started_on_the_card`，把 `DeviceStarted` 再置一遍。此刻能借的 = 现读空闲 + min(占着 − 已销账未还, 许出去的) − 驱动 − 借着的；设备重起后同一个进程可能同时占着旧设备（借据持有的克隆没丢干净）与新设备两份驱动开销，占着被封顶到许出去的，现读的空闲是真的，所以不会多借（K2-1 的算式与随机验证）。会算偏的是「丢之前销账的缓冲」混进 `returned` 的那一份，与 K2-2 的 P1 同源，方向是少借。

两个只对读码成立的小毛病（方向都是把真故障说成显存不够，不是放过）：
- `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:165`：「match (self.lost_reason(), last_out_of_memory) {」：设备丢了、而这个设备**此前任何时候**报过 OOM，就说成显存不够；`last_out_of_memory` 只在记新的时覆盖、从不清。早先一次已处理的 OOM + 之后一次无关的设备丢失（驱动重置）→ 报本次未跑而不是判红。测试 `a_failure_is_put_down_to_gpu_memory_only_after_an_out_of_memory_on_the_device` 的最后一条断言钉的正是这一形。
- 同一设备上别的线程在这段活期间记了 OOM，而这段活因别的原因 panic，也会被说成显存不够（`out_of_memory_explaining_a_failure_since` 只比次数）。
推翻它们的现象：读码之外有哪一处在丢失之前清掉 `last_out_of_memory`、或按线程认。我没有清它们的代码证据，grep `last_out_of_memory` 只命中记录与读取。

### K1 第 3 条　K2-2 拆出的两种结局，各自的出口

借草稿区之前先分三种（`crates/singlefs-harness/src/gpu_memory_ledger.rs:447`：「pub fn decide_scratch_before_booking(」），调用点在 `crates/singlefs-checker-tier/src/crash_amplification.rs:2685`：「match decide_scratch_before_booking(&capacities, &capacities_when_the_cards_are_empty) {」：

| 结局 | 出口 | 判 |
|---|---|---|
| 至少一张卡此刻带得了一个状态 | 去借；借时账里再压一次，一张都没借到仍是此刻借不到（`decide_after_attempting_every_card`） | 对 |
| 每张卡此刻带不了、至少一张空着时带得了（此刻借不到） | 打本次未跑行、交 `NotRunTheGpuMemoryWasShort`（`crash_amplification.rs` 的 `ScratchBeforeBooking::NoCardHoldsAStateNow` 一支，不去借） | 对：不判红、不整趟报错 |
| 每张卡空着时也带不了（放得下是假的） | `None`，往下走到「没有一张卡装得下这条流的表…」红（`crates/singlefs-checker-tier/src/crash_amplification.rs:2746`：「let Some(scratch) = scratch.filter(|scratch| !scratch.cards_for_the_judges.is_empty()) else {」） | 对：等多久都一样，判红 |

「放不下一个状态」与「此刻借不到」的分界是 `crates/singlefs-harness/src/gpu_memory_ledger.rs:380`：「if need_of_one_state_mebibytes > room.ceiling_mebibytes {」。没起卡池的进程（`DevicesStartedForThisRun`）此刻与空着时两张表同值（`states_in_flight_capacity_when_the_card_is_empty: states_in_flight_capacity`），走不到「此刻借不到」那支，行为与改之前相同。
借草稿区里 `NothingLeftToPlace` 拆成「前面的卡已装完」「此刻带不了（`NoRoomNow`）」「空着时也带不了（`CannotHoldOneState`）」三种，`place_states_on_the_cards_that_book_now` 里 `*states_the_card_holds == 0` 的卡交 `classify_a_card_holding_no_state` 判，不再并进「没有要装的」。
harness 档变异（`crates/mutations.tsv` 第 10y 批两行）都红，原样见 K3。这一格唯一没钉的是边界：`>` 改 `>=`（需要的恰好等于空卡上能借的）整个 `gpu_memory_ledger::tests` 照绿，见 K3 的 X4。
推翻它的现象：造出一张卡「此刻带不了、空着时带得了」而产品路径判红，或「空着时也带不了」而报了本次未跑。

## K2　只防 OOM、不压并发两头（新代码引进来的）

### K2-1　新上界、重起、回调有没有一形让账给的量大于此刻真能用的

**账给的量不会超过现读的空闲（算式 + 随机验证）。** 此刻能借的 = `capacity(M) − B`，`M = F + min(占着 − 已销账未还, 许出去的)`，`capacity(M) ≤ M − 驱动`（额度、3/4、配置额度都只会更小），许出去的 ≤ B + 驱动，所以 `room ≤ F`（没登记时 `≤ F − 驱动`）。我拿真函数 `card_room_for_this_process` 与真容量式 `card_ledger_capacity_mebibytes` 随机验了 300 万组（读数、占着的三种、驱动那一份三种、已销账未还、环境额度与配置额度有无都随机；`src/sonnet_probe_k2_property.rs`），原样：

```
SAMPLES 3000000 room_now>free: 0 max_over=0
```

这只证「不超过现读的空闲」，不证「不超过真空闲减还没建的」：后者要看账把占着的拆成借据建上的与别的占用对不对。读码找到的偏差，方向都是**多借**一点，量级取决于非借据占用 X（管线、暂存、分配器按块多拿）：占着的里不是借据建的那部分 X 被当成借据已建上，未建的许出去的被少算 X，多借 X；驱动固定开销是 2560 的拍数（判决正文自己列为不在这一轮范围的第三条）。X 我没量（要真在卡上跑 GPU 用例，是重型）。
新的上界（`total_less_reserved`）只用在 `NeverFits` / `CannotHoldOneState` 的判红分界，不进 `room_now`，不会让账多借。重起和回调不碰账。我没找到一形让账给的量大于此刻现读的空闲。
推翻它的现象：造出 `room_now > F` 的一组读数与账（随机 300 万组 0 次），或真卡上 X 大到使「借到的比真能用的多」的实例。

### K2-2　有没有一形：卡放得下却报本次未跑、判红，或整趟弃卡

**P1（打中，读码 + 在真账上造出）：已销账、卡上其实已经还了的借据，在有一笔长借据建着的时候让后面的借少借，且一直累积。**
机理：销一笔借据把它的量加进 `returned_possibly_still_on_the_card_mebibytes`（`crates/singlefs-harness/src/gpu_memory_ledger.rs:829`：「self.returned_possibly_still_on_the_card_mebibytes = self」，加法 `saturating_add`）；每读一次只往下压到占着减驱动那么多（`crates/singlefs-harness/src/gpu_memory_ledger.rs:197`：「.min(mebibytes.saturating_sub(driver_on_the_card_mebibytes))」，应用处 `crates/singlefs-harness/src/gpu_memory_ledger.rs:926`：「books_of_the_card.returned_possibly_still_on_the_card_mebibytes =」）。
只要有一笔长借据把「占着 − 驱动」撑得比累计的 `returned` 大，这个数就一直不降，而此刻能借的里 `占着 − returned` 被当成借据已建上的量（`crates/singlefs-harness/src/gpu_memory_ledger.rs:163`：「.saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)」）：真已经还给驱动的短借据，每还一笔，长借据已占的量就被再扣一遍（占着的已经不含短借据，空闲里也不含）。
造法（`src/sonnet_probe_k2_returned.rs`，真账 `GpuMemoryLedger`，读数用假的；假设短借据丢了之后卡上马上还了，这是最好的情形）：一笔 10000 的长借据建满，再借、建、丢十笔 1000 的短借据，每步读此刻能借的，原样：

```
A 建满之后此刻能借的 = Ok([19550])（真空闲 19550）
第 1 笔短借据还了、卡上已经还给驱动：此刻能借的 = 18550（真空闲 19550，少借 1000)
第 2 笔短借据还了、卡上已经还给驱动：此刻能借的 = 17550（真空闲 19550，少借 2000)
第 3 笔短借据还了、卡上已经还给驱动：此刻能借的 = 16550（真空闲 19550，少借 3000)
第 4 笔短借据还了、卡上已经还给驱动：此刻能借的 = 15550（真空闲 19550，少借 4000)
第 5 笔短借据还了、卡上已经还给驱动：此刻能借的 = 14550（真空闲 19550，少借 5000)
第 6 笔短借据还了、卡上已经还给驱动：此刻能借的 = 13550（真空闲 19550，少借 6000)
第 7 笔短借据还了、卡上已经还给驱动：此刻能借的 = 12550（真空闲 19550，少借 7000)
第 8 笔短借据还了、卡上已经还给驱动：此刻能借的 = 11550（真空闲 19550，少借 8000)
第 9 笔短借据还了、卡上已经还给驱动：此刻能借的 = 10550（真空闲 19550，少借 9000)
第 10 笔短借据还了、卡上已经还给驱动：此刻能借的 = 9550（真空闲 19550，少借 10000)
要 18550 MiB（真空闲 19550，放得下）：
  借不到（NoCardHasRoomNow，调用方报本次未跑）：要 18550，此刻能借 [9550]
```

真空闲是 19550，要 18550 放得下，账说此刻能借 9550，报借不到，调用方报本次未跑。少借的量 = min(累计销账量, 长借据已占的量)，长借据一退出占着降下来，`returned` 随之压回 0，自己恢复。
实际的触发取决于 wgpu 的丢弃缓冲多快还给驱动（代码注释说「要等设备收拾」，那一段里照实是占着，账对）；还给之后账不知道，只要有长借据在，就一直记着。并发的 GPU 用例（几条用例同一个进程、一条长借据加许多短借据）正是这种形状，违反「不压并发」。读码 + 真账构造，没有在真卡上复现。
推翻它的现象：`returned` 在读数上有第二条衰减路径（我读 `returned_possibly_still_on_the_card_after_a_reading`，只有 `min(…, 占着 − 驱动)` 一条）；或 wgpu 在借据销账之前就保证缓冲还给驱动且账有办法知道。

**P2（读码，不是 10y 新引进的，但 10y 把触发它的 OOM 来源加多了）：一张卡 OOM 全趟弃卡。** 建判器或判一块时某张卡报 OOM，那条卡线程置全趟的 `stop` 并发 OOM 消息（`crates/singlefs-checker-tier/src/crash_amplification.rs:2088`：「work.stop.store(true, Ordering::SeqCst);」，判块那一处 `crates/singlefs-checker-tier/src/crash_amplification.rs:2145` 同一句），收完整趟报本次未跑，别的卡剩下的块不判。起设备那一步的 OOM 则不同：只还掉那张卡的借据、别的卡照用，一张都用不了才报本次未跑（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1452`：「if leases.iter().all(Option::is_none) {」）。两处处理不一致：运行中一张卡 OOM 而另一张卡空着、放得下，整趟还是弃。
是「报本次未跑」不是判红，判完的块在库里，下一趟接着判；但违反「不压并发」的字面。10y 的改动让更多 OOM（另起线程上的、起设备的）走进这条路。
推翻它的现象：有一处在 OOM 之后让另一张卡接着判剩下的块（我读 `judge_on_one_card_and_start_the_next`，没有）。

**P3（K1-1 ③ 已写）：认不得就判红**，老驱动没有 `Reserved` 行时卡放得下也判红；推的。

没打中的：`decide_scratch_before_booking` 三出口见 K1 第 3 条；`lease_card` 此刻借不到报本次未跑；起设备 OOM 不缓存 `Err`（重起，变异行证红）；`lease_card_waiting_for_room` 在同进程兄弟借着时 panic 是判决正文列为已知没做的，不算打中。

## K3　测试判别力

**条款对象：** 扫源码、判完再核、新变异行，能不能拦住这一批每一处被改回去。

**① 第 10y 批新增的 15 条 harness 档变异行（`crates/mutations.tsv` 第 3180 至 3194 行）我在草稿副本上逐条重套并跑它们点名的用例**（脚本 `src/mutate-harness.py`，每条先核旧串在文件里恰好命中一次；跑完还原；下面摘每条的结论，每条的完整原样输出在 `out/mut-10y-1.log`、`out/mut-10y-2.log`）：

```
[1] 第10y批 显存账 K1-1 空闲与占着的分两次读（每借一次取两份报告） => 红（被抓）
[2] 第10y批 显存账 K1-1 报告里 FB 之外小节的 Free 也当空闲（BAR1 的盖掉 FB 的） => 红（被抓）
[3] 第10y批 显存账 K1-1 读报告时不认这个进程另有不走账的设备（攻方乙形） => 红（被抓）
[4] 第10y批 显存账 K1-1 另有不走账的设备也当都走这份账（占着的当成借据建上的，攻方乙形） => 红（被抓）
[5] 第10y批 显存账 K1-1 估计只取现读的空闲（实际占着的里借据建上的一份都不算，第10t批 K4-3 那一形） => 红（被抓）
[6] 第10y批 显存账 K1-1 销了账、卡上还没还的不记（当成别的借据已经建上的，正推冲突 B） => 红（被抓）
[7] 第10y批 显存账 K1-1 销了账、卡上还没还的那一份不照读数往下压（卡上还了照样少借） => 红（被抓）
[8] 第10y批 显存账 K1-1 空卡上能用的照总量算（不减驱动保留的，攻方丙形） => 红（被抓）
[9] 第10y批 显存账 K2-2 此刻带不了的卡记成没有要装的（一张都借不到时判红） => 红（被抓）
[10] 第10y批 显存账 K2-2 每张卡此刻都带不了一个状态而空着时带得了也判红（白红） => 红（被抓）
[11] 第10y批 显存账 K3 卡上报 Out of Memory 那一句不写显存不够 => 红（被抓）
[12] 第10y批 显存账 K3 按状态数借时同一张卡给了两次不断言 => 红（被抓）
[13] 第10y批 显存账 K3 读到的卡比池里多不拒 => 红（被抓）
[14] 第10y批 显存账 K1-3 卡上留着的设备丢了照旧拿它 => 红（被抓）
[15] 第10y批 显存账 K1-3 起设备失败当场判红（不交错、下一次不再起） => 红（被抓）
```

15 条全红，每条红在它点名的那个用例上（完整输出含失败用例名，在 `out/mut-10y-1.log`、`out/mut-10y-2.log`）。另 8 条 checker 档行（第 3195 至 3202 行）按规矩我不跑；逐条读它们点名的用例，其中 5 条（第 3195 至 3199 行）用例是纯函数、不要卡，读码上能红；第 3200 行（判完那一刻不核）靠 `crash_amplification_product_path_books_its_scratch_on_the_card_pool` 的有卡用例，**没有卡的机器上那条用例报本次未跑，这条变异在无卡机器上照绿**；第 3201、3202 行是扫源码用例，不要卡。

**② 我自己再造 13 条「改回去」的变异（harness 档，`src/rows-extra.tsv`，同一个脚本，跑的是 `gpu_memory_ledger::tests` 整个模块 38 条用例）**，看这一批改的地方还有哪些改回去照绿。摘每条结论（完整原样输出在 `out/mut-extra-0.log` 至 `out/mut-extra-3.log`）：

```
[1] X1 ret 往下压的结果不存回 => 红（被抓）
[2] X2 往下压时 AlsoOutside 当 Only => 绿（没抓）
[3] X3 NeverFits 边界 > 改 >= => 红（被抓）
[4] X4 放不下一个状态边界 > 改 >= => 绿（没抓）
[5] X5 总线号只去一段域 => 红（被抓）
[6] X6 进程号相等改不等 => 红（被抓）
[7] X7 小节判缩进 <=4 改 <4 => 红（被抓）
[8] X9 借草稿区前判有一张带得了：any 改 all => 红（被抓）
[9] X13 设备起好了不留着 => 红（被抓）
[10] X16 本进程占的量读不出当 u64::MAX（悄悄当大） => 绿（没抓）
[11] X17 进程号认不出悄悄当 0 => 绿（没抓）
[12] X18 读不出 Free 悄悄当 0 => 绿（没抓）
[13] X19 读不出 Reserved 悄悄当 0 => 绿（没抓）
```

**照绿的六条（K3 的打中）：**
- X16、X17、X18、X19：解析认不得的输出改回「悄悄当大 / 当 0」——本进程那一行占的量读不出当 `u64::MAX`、进程号认不出当 0、`Free` 读不出当 0、`Reserved` 读不出当 0——整个 `gpu_memory_ledger::tests` 38 条一条不红。现有用例只钉了「点名的卡在报告里没有」这一种读不出。K1-1 ③ 那一格问的「认不得时是报错还是悄悄当 0」，实现是报错，但没有测试钉住它，改回去不会被发现。后果的方向（读码推的，没造）：X17（进程号当 0，这个进程的占用读成没列）与 X18（`Free` 当 0）都是少借，卡放得下也报借不到、本次未跑；X19（`Reserved` 当 0）就是 10x 判决 K1-1 的第三形（空卡上能借的拿总量当上界，空卡读到的空闲比总量少一份保留，借的量落进「借不到」而不是判红）；X16（占着的当极大）封顶到许出去的之后，把还没建的许出去的也当已建，多借。
- X2：往下压 `returned` 那一步（`returned_possibly_still_on_the_card_after_a_reading`）里把「这个进程另有不走账的设备」并进「都走这份账」那一臂，照绿：这一臂没有用例。
- X4：「放不下一个状态」的边界 `>` 改 `>=`（恰好等于空卡上能借的量时从「此刻借不到」变成「放不下」判红）照绿：用例里没有恰好相等的一组。
- 另：10y 在 `crates/singlefs-checker-tier` 里改的几处没有变异行覆盖（读码列出，改回去之后我判断没有一条现有用例会红，**没跑**，因为那些用例在 checker 档或要卡）：
  1. `start_the_card` 里登记「不走卡池的设备在这条总线上」的 `BUSES_WITH_DEVICES_STARTED_OUTSIDE_THE_CARD_POOL` 那一段（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:383`），整段删掉、或 `usable_gpu_cards`/`gpu_cards_by_priority` 两处的 `OutsideTheCardPool` 改成 `TheCardPool`：只有读报告那个纯函数有变异行（第 3182、3183 行），登记与传参没有；
  2. `open_the_card_pool` 里传给读卡闭包的 `has_devices_started_outside_the_card_pool_on_the_bus` 改成 `|_| false`：同上；
  3. `start_the_card` 里编管线那一段的 OOM 范围（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:362`、`375`）去掉：没有不要卡的用例覆盖；
  4. `device.set_device_lost_callback`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:358`）去掉，或 `started_card` 里 `lost_reason().is_some()`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1282`）改成 `false`：变异行只改了 `DeviceStartedOnDemand` 里的通用判断，设备类型与回调没人测；
  5. `print_the_not_run_line_or_refuse` 里 `OutOfGpuMemoryStartingTheDevice` 一支改成判红、`card_lent_for_the_booking` 里 OOM 改成 panic：没有用例；
  6. `judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 里 `decide_scratch_before_booking` 三支的接线（`crates/singlefs-checker-tier/src/crash_amplification.rs:2685` 起）：变异行只改了纯函数 `decide_scratch_before_booking`，没改它的调用处，`NoCardHoldsAStateNow` 一支接成别的结局没人发现；
  7. `states_in_flight_capacity_when_the_card_is_empty` 取 `room.ceiling` 还是 `room.room_now` 没有变异行。
  （这些都需要要卡的有卡用例或 checker 档用例才可能红；有卡机器上用例 `the_product_path_judges_on_scratch_it_booked_on_the_card_pool` 等也许会碰到其中几条，我没有证据说会。）

**③ 扫源码（K3-2）**：读码，`#[cfg_attr(条件, 属性…)]` 的处理是顶层逗号切开、跳过第一段（条件）、其余每段递归判（`attribute_marks_test_code` 调自己），所以 `#[cfg_attr(a, cfg_attr(test, test))]` 这种嵌套也认；字符串里的逗号在判之前已被抹掉。名单之外的 `tests/` 文件另有一条扫描用例，要求扫到的文件数 ≥ 20 且与登记表逐个相同、登记了而不绕要红。没找到「改回去照绿」的形；我没有跑这两个文件的用例（checker 档），读码判断。
**④ 判完再核（K3-1）**：核账那一步放在 `while !work.must_stop()` 循环之后、判器丢掉之前，所以「建好之后、判完之前还掉」会让 `scratch_lease_on_the_books_when_the_judging_finished` 变 `Some(false)`；判完之后、函数返回之前还掉的抓不到（借据由调用方在判器丢掉之后才还，不是这一格要防的）。
推翻 ② 的现象：把我那几条变异接着在有卡机器上、或经 checker 档的完整变异表里跑，哪一条红了。

## K4　删除线第 33 批 T 与 T1 甲的交界

**条款（行号都是现查的）：** T1 甲在 `.claude/kb/decisions/16-发布语义.md:37`：「被抛弃根照它记着的落点进影子账、不进上限」（「重建分配器两遍里只有一遍自证过的槽」那一句）。C694 在 `.claude/kb/checks-owed.md:545` 那一行（修法已定：C、B 都做、F1 乙、F2 乙）。C 的条款两处：`.claude/kb/decisions/23-journal的角色与格式.md:392`：「把水位之上、没被施加的全部 journal 记录点名的落点隔离」，同一行写它护的是：「它护的是挂载那一刻根槽读不出、系统配置又没见证到的那条根新分配的单元」；`.claude/kb/decisions/28-挂载期承诺量.md:27`（第九项）：「加水位之上没施加的 journal 记录点名、不被回退候选集里的根引用的槽」。
判决：`research/prompts/m3-c694-r2-main-verification.md` 第三节第 1、2 条（C 的选法 `all`、解开时机 `ring`；B 的凭据只读现行账）。I-7.4 在 `.claude/kb/invariants.md` 的 I-7.4 那一行。

**代码（B、C、T1 甲各在哪）：**
- B：`crates/singlefs-core/src/mount.rs:1608`：「if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {」，成立就 `return Ok(Vec::new());`（1609），即这条被抛弃根什么都不引用；凭据是树表槽与映射根槽的每一条位置条目在现行账里都有分配代大于那条根 txg 的记录罩住（`mount.rs:1625` 起）。
- C：`crates/singlefs-core/src/mount.rs:1566`：「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」——水位之上没施加的记录点名的落点，**被候选根（或现行账）引用的不隔离**；候选 = `crates/singlefs-core/src/mount.rs:1518`：「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」。
- T1 甲：`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`（行 1018）与 `mount.rs` 里抬 F 那一串 `readable_roots_for_raising_the_floor` 顶上的那条根，再进同一个 `isolate_slots_referenced_only_by_abandoned_roots`。

### K4-1　有没有一形让被抛弃根的落点掉出影子账（I-7.4）

**打中（在入库装置的造法上造出来了，机理用补丁对照坐实）：取新实例的那次可写挂载 M1 里，R 只有一个单元读回全零、R 的根槽与账都读得出，C 不隔离 R 的其余单元，M1 自己的写把它们盖掉。**

造法沿用 `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs`（384 槽小盘，第一个文件之后覆盖写 5 次，再录一次发布 R，崩在 R 的根槽落盘之后、系统配置轮换之前）。与已有用例（`ur`：R 的根槽与全部单元读回全零）不同的只是读回全零的集合：R 的 10 个单元（每个单元两块盘同偏移）里，**每次只让一个单元读回全零**（只这一次挂载），其余照读。每一格取 M1 的写，在「写行那次发布的根落盘之前」断电的盘面上看 R 的单元被盖掉几份、恢复择哪条根、池级 checker 的 I-7.4 / I-4.8，原样（探针 `src/sonnet_probe_k4_units2.rs`，输出 `out/k4-units2.log`）：

| 读回全零的那个单元（偏移 / 角色） | 根落盘前断电：R 的单元被盖掉 | 恢复择 | checker |
|---|---|---|---|
| 23330816 / 其它（数据/映射条目/树节点…） | 0 份 | txg 9（R 是 txg 9，P 是 txg 8） | 无 |
| 25116672 / 其它（数据/映射条目/树节点…） | 14 份 | txg 8（R 是 txg 9，P 是 txg 8） | 红：I-4.8 与 I-7.4（最新根 txg 9 引用的单元已被复用或抹头） |
| 25133056 / 其它（数据/映射条目/树节点…） | 14 份 | txg 8（R 是 txg 9，P 是 txg 8） | 红：I-4.8 与 I-7.4（最新根 txg 9 引用的单元已被复用或抹头） |
| 25165824 / 其它（数据/映射条目/树节点…） | 14 份 | txg 8（R 是 txg 9，P 是 txg 8） | 红：I-4.8 与 I-7.4（最新根 txg 9 引用的单元已被复用或抹头） |
| 25182208 / 其它（数据/映射条目/树节点…） | 0 份 | txg 9（R 是 txg 9，P 是 txg 8） | 无 |
| 25198592 / 其它（数据/映射条目/树节点…） | 0 份 | txg 9（R 是 txg 9，P 是 txg 8） | 无 |
| 25214976 / 其它（数据/映射条目/树节点…） | 0 份 | txg 9（R 是 txg 9，P 是 txg 8） | 无 |
| 25231360 / 其它（数据/映射条目/树节点…） | 14 份 | txg 8（R 是 txg 9，P 是 txg 8） | 红：I-4.8 与 I-7.4（最新根 txg 9 引用的单元已被复用或抹头） |
| 25247744 / 映射根 | 14 份 | txg 8（R 是 txg 9，P 是 txg 8） | 红：I-4.8 与 I-7.4（最新根 txg 9 引用的单元已被复用或抹头） |
| 25264128 / 树表 | 0 份 | txg 9（R 是 txg 9，P 是 txg 8） | 无 |

十个单元里五个（偏移 25116672、25133056、25165824、25231360、25247744）读回全零时：M1 做成、选到 P（txg 8），而 M1 在写行那次发布的根落盘之前就盖掉了 R 的 14 份单元（7 个单元 × 两盘），断电之后恢复择 P、R 的内容丢；这个盘面上池级 checker 判 I-4.8 与 I-7.4 红。读回全零的是树表（25264128）、数据单元（23330816）、或另三个单元（25182208、25198592、25214976，我没有逐个核它们的角色，推的是分配记录树的节点）时，C 照常护住（盖掉 0 份）。
M1 做完、写行那条根落盘之后（R 变成被抛弃的根）同一批单元组再看（`out/k4-units1.log`）：「over」那一档 10 个单元组里 5 个、顺序写三个数据单元（`seq3`）那一档 13 个里 6 个，池级 checker 在 M1 之后的盘面上报：

```
I-7.4: 被抛弃的根（实例 1、txg 9）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：树表单元 在盘 0 槽 1542 的那一份…
```

**机理对照（补丁在草稿副本，`src/patch-c-without-candidate-exemption.diff`，只改一行：C 那一圈循环不看 `referenced_by_candidates`）：** 同一批十个单元组补丁后全部「被盖掉 0 份、恢复择 txg 9」（原样见 `out/k4-units2-patched.log`），checker 无红。汇总（原输出提取）：

```
零单元偏移  角色     原：被盖掉  原：恢复择  补丁：被盖掉  补丁：恢复择
  23330816  其它       0        txg 9        0          txg 9
  25116672  其它      14        txg 8        0          txg 9
  25133056  其它      14        txg 8        0          txg 9
  25165824  其它      14        txg 8        0          txg 9
  25182208  其它       0        txg 9        0          txg 9
  25198592  其它       0        txg 9        0          txg 9
  25214976  其它       0        txg 9        0          txg 9
  25231360  其它      14        txg 8        0          txg 9
  25247744  映射      14        txg 8        0          txg 9
  25264128  树表       0        txg 9        0          txg 9
```

所以原因是（补丁对照坐实的只是「C 那一圈被 R 自己豁免」这一句；「M1 重建时 R 按实例表是有效根、取新实例的行还没写」是推的，我没有直接读 `abandoned_by_table` 在那一刻用的是哪张表）：M1 重建分配器时 R 按实例表是有效根，账读得出，`isolate_slots_referenced_only_by_abandoned_roots` 把 R 当**候选根**，R 引用的槽全部进豁免集；C 点名的落点正是 R 的单元，被它自己豁免了；它们又不在 P 的现行账里（R 之后才分配的），是空闲槽，M1 写行那次发布取落点时就取到它们（上面的盘面是写行那次发布的根落盘之前，R 的单元已经被盖掉）。R 的单元只有「账读不出」（树表、分配记录树那几个单元读不出，候选根读不出就什么都不豁免）时才被 C 护住。
条款与实现一致：D28 第九项（`28-挂载期承诺量.md:27`）写的就是「不被回退候选集里的根引用的槽」，实现照做；而 D23 已定项 14 同一行写的是隔离「全部」记录点名的落点、护的是「读不出」的那条根。两句话在「根读得出、没被所选的版本用上（走读到某个单元失败退回上一版）」这一形上给的不一样，这一形上条款自己漏了：R 读得出（候选）却不是所选版本，它独占的单元既不在现行账里、也被豁免。
这一形是已登记故障模型的一个更弱的变体，已登记的一组不覆盖它（`crates/singlefs-harness/src/history.rs:489`：「根槽与它那次发布的记录点名的单元整次挂载读不出」，是 `CrashRecoveryAbandoningTheNewestRoot` 那一项，我只让其中一个单元读不出、根槽照读得出）；`m3-c694-r2` 的 K1-2 判的是全部读不出。
推翻它的现象：补丁之外还有别的办法让 R 的单元不被 M1 盖掉我没看见（我只试了这一个补丁）；或证明这一形走不到（我只看到 `recover` 在这些盘面上退到了 P、M1 做成可写，没有逐行读恢复路径证明它一定如此）。

### K4-2　有没有一形，本该放行的可写挂载被拒

没造出来。我做了三样：
1. 把 B 与 C 两份入库用例在草稿副本上跑一遍（`out/bc-tests.log`，原样）：

```
test an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write ... ok
test an_abandoned_root_whose_account_slots_were_taken_by_later_units_no_longer_refuses_unmount_floor_raise_or_remount ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
test the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device ... ok
test a_power_loss_before_the_row_root_of_the_mount_that_took_a_new_instance_still_reads_back_the_root_whose_units_read_back_zero_at_that_mount ... ok
test the_units_named_by_the_readable_records_of_a_publish_whose_last_record_reads_back_zero_are_isolated ... ok
test the_units_named_by_the_record_of_a_root_reread_as_abandoned_stay_isolated_until_this_process_overwrites_its_root_ring_slot ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
```

2. 在同一个造法上扫「M1 的读回全零集合 × M1 之后的会话里覆盖写 0 至 30 次」：读回全零集合取 5 种（`ur/over`：R 的根槽与全部单元；`root/over`：只根槽；`units/over`：只单元；`ur/seq3`、`root/seq3`：R 换成顺序写三个数据单元的那一种，同上两种集合），k = 覆盖写 R 之前的次数取 3、5、7，共 15 格；M1 之后每覆盖写一次看池级 checker 的 I-7.4，在 0、10、20、30 次那四个点各试一次卸载、一次抬 F 到准入上限、一次再可写挂载。原样计数（`src/sonnet_probe_k4_sweep.rs`，输出 `out/k4-sweep1.log`，经内存包装跑完，退出码 0）：

```
$ grep -c 'VARIANT' k4-sweep1.log      → 15
$ grep -c 'step ' k4-sweep1.log        → 60
$ grep -c 'unmount=ok raise=ok remount=ok' k4-sweep1.log   → 60
$ grep -c 'RED' k4-sweep1.log          → 0
$ grep -c 'M1 refused' k4-sweep1.log   → 0
```

即这 15 格里 M1 都做成、卸载、抬 F、再挂载各一次共 60 组（每组三样）全成、I-7.4 一次都没红（R 离开根环之后 R 的单元被盖掉，那时它已不在环里）。注意这一扫的读回全零集合都是**已登记的形**，K4-1 打中的那一形不在里面。
3. 记录（`ur` 那族）：R 为「顺序写三个数据单元」时，三条记录分别读回全零、全部读回全零、各配第一个数据单元读回全零，7 种，M1 都做成、选到 R、R 的单元被盖掉 0 份、I-7.4 无红（`src/sonnet_probe_k4_records.rs`，`out/k4-records.log`）。F1 乙（记录缺口且点名的单元验不过才拒）在这 7 种上没有拒。

读码上有一处没造出来：B 的凭据要求树表槽**与映射根槽**的每一条位置条目都被现行账里更晚的记录罩住（`crates/singlefs-core/src/mount.rs:1625` 起），而走到 B 这一支的前提是「从中央映射现算」也失败（`mount.rs:1595` 起先试现算）。映射树不止一个节点（大池）时，可能是树表槽被占了、映射根槽没被占而它下面某个节点被占：账读不出、现算失败、凭据不齐，仍拒。那条根的树表已经没了，是死根，按 B 的理由本可放行。384 槽小盘上映射树只有一个节点，造不出这一形；它也落在「已写坏的盘」（旧代码写坏的）上，是不是要放行归条款，我没判。
推翻它的现象：造出 R 的账读不出、凭据成立、而挂载或卸载被拒的盘面；或证明 B 这一支走到时映射树恒只有一个节点。

T1 甲这一边：没有为它造合成形（要照 `tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 的读次数钩子在 M2 的重建那一遍读坏 R 的根槽，再让抬 F 那一遍读坏）。读码：抬 F 那一串顶上的那条根走的是与挂载时同一个 `isolate_slots_referenced_only_by_abandoned_roots`，K4-1 那一圈豁免在那里的候选集（`floor` 换成新 F）按「被抛弃」重算，R 此时按实例表已被抛弃、不是候选，所以抬 F 那一遍不重现 K4-1；B 在那里回 `Ok(Vec::new())` 时只是不再加隔离（隔离位只增不减），根环表里记的 `referenced_placements` 不变。

## 冲突并排（整行抄，两边各一句）

- **K2-2 P1 少借。** 应该是（`crates/singlefs-harness/src/gpu_memory_ledger.rs:6`：「账只拦放不下的，不限同时借几份」）；今天的实现（`crates/singlefs-harness/src/gpu_memory_ledger.rs:163`：「.saturating_sub(booked.returned_possibly_still_on_the_card_mebibytes)」）；量到：真空闲 19550 放得下 18550，账给 9550（见 K2-2）。
- **K4-1 C 被自己豁免。** 应该是（`.claude/kb/decisions/23-journal的角色与格式.md:392`：「把水位之上、没被施加的全部 journal 记录点名的落点隔离」）；今天的实现（`crates/singlefs-core/src/mount.rs:1566`：「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」）；量到：R 的单元被 M1 盖掉 14 份（见 K4-1）。
- **K3 改回去照绿。** 应该是背景 K3 的话：能不能拦住这一批每一处被改回去；被改的那一行（`crates/singlefs-harness/src/gpu_memory_ledger.rs:762`：「let free_mebibytes = card.free_mebibytes.ok_or_else(|| unreadable("Free"))?;」），改回去是（转述）把 `.ok_or_else(…)?` 换成 `.unwrap_or(0)`，`gpu_memory_ledger::tests` 38 条照绿。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 第 1 条 四形与同一个量 | 一致 | 一次报告、本机两张卡空闲与总量减保留逐 MiB 等于 `--query-gpu`；一个同进程小窗口（登记在编管线之后）是推的 |
| K1 第 1 条 认不得的输出 | 一致 | 九种里八种报错判红、一种（本进程占的量读不出）少借；不悄悄当 0；老驱动缺 `Reserved` 行判红（推的） |
| K1 第 2 条 OOM 接住了吗 | 规则没说（残差） | 起设备、编管线、另起线程的内核、只排计划、设备丢失都接；单元校验和对拍几条线程派活没包，文件头自认，判决没点名；结局本次未跑、判完的块进库 |
| K1 第 3 条 K2-2 两种结局 | 一致 | 此刻借不到报本次未跑，空着时也放不下才判红；`>` 边界没有用例钉 |
| K2-1 账给的量大于真能用的 | 一致 | 算式 `room ≤ 现读空闲`，300 万组随机 0 次超；非借据占用 X 多借没量 |
| K2-2 放得下却报本次未跑 / 整趟弃卡 | 冲突 | P1：长借据在时每还一笔短借据多扣一份，最多扣到长借据已占的量；P2：运行中一张卡 OOM 全趟弃卡 |
| K3 改回去照绿 | 冲突 | 我造的 13 条里 6 条照绿（认不得当 0 ×4、`>=` 边界、往下压的 AlsoOutside 一臂）；10y 的 15 条 harness 行全红；checker 档行未跑 |
| K4-1 落点掉出影子账（I-7.4） | 冲突 | R 只有一个单元读不出、账读得出时 C 被 R 自己豁免，M1 盖掉 R 的 14 份单元，checker 红；补丁一行后全护住 |
| K4-2 本该放行被拒 | 一致 | 15 格扫 60 组（卸载、抬 F、再挂载）全成，记录 7 种全成；B 的凭据在大池映射树多节点时可能偏严，没造出 |

## 没做什么

- 没跑 checker 档的任何测试（含 10y 的 checker 档变异行 3195 至 3202 行、扫源码用例、`crash_amplification` 的有卡用例）；没有在真卡上跑 GPU 用例，所以非借据占用 X、wgpu 丢弃缓冲多快还给驱动、`nvidia-smi -q` 带进程那一段的真实格式（实现员自定的零轮）都没量。
- 没读别的驱动版本、没读第二台机器的 `nvidia-smi`，K1-1 ② 只在本机驱动 595.84 上成立。
- 没为 T1 甲造合成形（见 K4-2 末段）；没验 B 在多节点映射树上的偏严。
- 没判决、没替主 agent 采纳；补丁只在草稿副本，没动仓里任何文件。
- 草稿副本 `~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/repo` 与编译目录 `~/.local/share/singlefs/work/m3-batch10y-code-r1-sonnet/target` 已删（草稿目录里只留日志、脚本和一份补丁后的 `mount-patched.rs.keep`，补丁本身在 model 的 `src/` 里）。
