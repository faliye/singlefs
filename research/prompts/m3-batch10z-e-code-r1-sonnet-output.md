# 第 10z 批第五块：代码轮第一轮，正推腿报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 M01 M05 M10 M13 M14 M17 M22 M23 M33 -->

轮名 m3-batch10z-e-code-r1，立场正推。分到的格：正文第三节 K1（1–3）、K2（1、2）、K3。
模型目录 `research/prompts/m3-batch10z-e-code-r1-sonnet-model/`（说明在它的 `README.md`），草稿目录 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet/`。

## 〇、前提、读过什么、怎么量的

判的是上一轮判决第三节三条（原话，行号去判决文件里现查）：
- 「K1-5：叫停时第 3081 行那一句不判红，判完的块照块序入库、没判的留给下一趟；`cards_used` 同一张卡只算一次。」（`research/prompts/m3-batch10z-c-code-r1-main-verification.md:36`）
- 「K1-4：同一设备上不同组的一段因设备丢了失败时，按丢失原因（含显存不够）认；真 panic 发生时设备没丢的，不因之后别段的 Out of Memory 说成显存不够。」（`research/prompts/m3-batch10z-c-code-r1-main-verification.md:37`）
- 「K3：两腿列的照绿项各补会红的用例与变异行」（`research/prompts/m3-batch10z-c-code-r1-main-verification.md:38`）

| 项 | 内容 |
|---|---|
| 冻结副本 | `~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/`；在它里面 `sha256sum -c research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt`：`OK` 334 行、不是 `OK` 的 0 行（`grep -c ': OK'` 与 `grep -vc ': OK'` 的输出）。下面引的行号都是它的；仓根的三份源文件此刻同一行号（引文对仓根也核过） |
| 我的副本 | 草稿目录 `tree/`（跑 sim）、`tree-mut/`（跑变异）是它的拷贝，`diff -rq` 只多我加的三个测试文件；编译目录 `target/`、`target-mut/`、`target-strings/` |
| 开跑前负载 | `ps` 看到别的会话的 `cargo test --release -p opus-probe`（pid 3038659 一族），没有 qemu、vm-bench、e152、fio；`nvidia-smi --query-gpu` 只读：5090 用着 239 MiB、5060 Ti 2 MiB；我没起任何 GPU 设备，所有跑的都是 harness 档纯函数与 `wgpu` 的 `to_string()` |
| 命令 | `CARGO_TARGET_DIR=<草稿目录>/target nice -n 19 bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test <名> -- --nocapture --test-threads 1`；变异那批 `--lib -- gpu_memory_ledger` |
| 禁读 | 没读攻方这一轮的输出、模型和提示 |
| 基线 | 未变异的 `cargo test --offline -p singlefs-harness --lib` 原样末行：`test result: ok. 169 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 27.88s` |

上一轮四份模型改到新接口，放 `probes/`：`rounds_new.rs`（攻方 `attack_10zc_rounds.rs` 的叫停四格与随机、正推 `rounds_sim4.rs` 的 STOPSIM，用 `RoundsOfTheCards` 与 `StopOfTheCards` 搭，外层循环照 `crash_amplification.rs` 第 2892–3168 行、卡线程照第 2235–2405 行的次序）、`endings_new.rs`（攻方 `attack_10zc_endings.rs` 的甲乙丙形与正推 `explain_demo.rs`）。装卡与账是模型（每张卡每轮一个「此刻带得了几个状态」），抄得对不对是读码推的，checker 档那几行没在这里跑。

## 一、K1-1　K1-5 叫停落在任一轮任一时点

**判：做到。有三处小漏（不判红、不漏判，是账与观测），列在最后。**

### 机制（读码）
卡线程每领一块之前问 `stop_of_the_cards`（`crates/singlefs-checker-tier/src/crash_amplification.rs:2121`），调用方叫停时 `may_take` 只放比判完的最大块号小的队头：「self.largest_judged.is_some_and(|largest| first < largest)」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1906`）。
最大块号取 max：「.map_or(pending_index, |largest| largest.max(pending_index))」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1959`）。
库按块序写：「while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2104`）。

「在等的」判法是精确的，推法：已写进库的是块号连着的前缀 0..k；判完而没写的块（在等的）只能在缺口 k 之后；缺口块与最大判完块之间的每一个块号，要么已领走（在别的卡手里，会判完或交回），要么在队里。队里比最大判完块号小的块因此恰是「写最大那一块还缺的块」。最大判完块已写进库时前缀盖到它，队里没有比它小的。比 `largest` 小与不大于 `largest` 在队里等价（最大判完块不会同时在队里，`<=` 的变异 M01 是等价变异，见 K3）。
所以「把不在等的块当在等」与「在等的块不领」两个方向都推不出反例；实现员报告里「两张卡都在块上报了时，比最大块号小的那块可能没人在等」那一句我推不出对应的形（上面的前缀论证没有用到谁在等）。

### 实测
`rounds_new.rs` 随机格：1–4 张卡、1–10 块、每张卡每轮带得了 0/1/2/8 个状态、建判器时 OOM 概率 1/6、判第 n 块 OOM、报得早晚不一、叫停文件在第 1–24 个事件之一出现（一半的格）。20000 格原样（`logs/rounds_new3.log`）：
```
ATTACK_NEW k1_5=random-rounds ended=EveryBlockStored stop_requested=false runs=12624
ATTACK_NEW k1_5=random-rounds ended=EveryBlockStored stop_requested=true runs=391
ATTACK_NEW k1_5=random-rounds ended=NotRun stop_requested=false runs=4731
ATTACK_NEW k1_5=random-rounds ended=NotRun stop_requested=true runs=36
ATTACK_NEW k1_5=random-rounds ended=StoppedOnRequest stop_requested=true runs=2218
ATTACK_NEW k1_5=random-rounds runs=20000 flags={"cards_used_gt_cards_that_judged": 1076, "cards_used_lt_cards_that_judged": 3487, "notrun_judged_unstored": 326} overbooked_states_total=315 blocks_judged_after_the_stop_was_read_total=147
```
`flags` 里没有出现的项，在代码里都有计数，值是 0：轮末断言 panic（`assert_panicked`）、一块判两遍（`judged_twice`）、写库不是前缀（`store_not_prefix`）、判红（`red`）、`EveryBlockStored` 却缺块（`every_stored_but_missing`）、`StoppedOnRequest` 而判完的有没入库的或还有等着的（`stopped_with_judged_unstored`）、没叫停却 `StoppedOnRequest`（`stopped_without_stop`）、轮数超过卡数加 1（`more_rounds_than_cards`）、叫停读到之后才领走的块不比判完的最大块号小（`judged_block_taken_under_stop_not_below_the_max_judged`，用线程里最近一次读到的叫停给每块打标签，不靠事件快照）。
叫停之后又判的块（147 块，20000 格合计）全是必要的那种。叠加「叫停文件又消失」（`UNSTOP_AFTER=1`、`=3`，各 10000 格，`logs/rounds_unstop_1.log`、`logs/rounds_unstop_3.log`）同样 0 项。

上一轮定形格重跑（`logs/rounds_new.log`、`logs/rounds_new4.log`）。攻方 `stop-in-the-closing-round`，上一轮 `stop-at-round-1-last-block` 是 `NotRun`（判了 4 块没入库）、`stop-at-round-2-build` 是 `AssertPanicked`（等着 4 块）、`stop-at-round-2-first-block` 是 `EveryBlockStored`（出处：上一轮攻方模型目录 `m3-batch10z-c-code-r1-opus-model/output-models.txt` 第 11–13 行），现在三格都是：
```
ATTACK_NEW k1_5=stop-in-the-closing-round cell=stop-at-round-1-last-block ended=EveryBlockStored rounds=2 stored=5 judged_once_or_more=5 judged_not_stored=0 stop_requested=true only_for_waiting_rounds=1 cards_used=1
ATTACK_NEW k1_5=stop-in-the-closing-round cell=stop-at-round-2-build ended=EveryBlockStored rounds=2 stored=5 judged_once_or_more=5 judged_not_stored=0 stop_requested=true only_for_waiting_rounds=0 cards_used=1
ATTACK_NEW k1_5=stop-in-the-closing-round cell=stop-at-round-2-first-block ended=EveryBlockStored rounds=2 stored=5 judged_once_or_more=5 judged_not_stored=0 stop_requested=true only_for_waiting_rounds=0 cards_used=1
```
正推 STOPSIM（两卡 10 块，第 0 张慢、判第 1 块就 OOM；上一轮是第 2 轮起叫停，等着 9 块，这里用事件近似）：
```
STOPSIM_NEW stop_at_event=Some(4) ended=StoppedOnRequest rounds=2 stored=2 judged=2 judged_not_stored=0 waiting_at_end=0 only_for_waiting_rounds=1 overbooked_states=8 cards_used=1 cards_that_judged={1}
STOPSIM_NEW stop_at_event=Some(8) ended=StoppedOnRequest rounds=2 stored=6 judged=6 judged_not_stored=0 waiting_at_end=0 only_for_waiting_rounds=1 overbooked_states=4 cards_used=1 cards_that_judged={1}
```

### 做到但有一形漏（三处，都不判红、不漏判）

1. **`cards_used` 去重做到，但数的口径没动。** 收过工的卡按序号放进集合：「self.cards_that_finished.extend(cards_finished);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2292`），`cards_used` 取集合大小，外层循环交它：「outcome.cards_used = rounds.cards_used();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3163`）。同一张卡两轮各收工一次只算一张（20000 格里 `cards_used` 超过卡数的 0 格）。
   集合里放的是发了 `Finished` 的卡：「received.finished_cards.push(card_outcome);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2081`）；判了几块之后报 OOM 的卡发的是 `OutOfMemory`，不进集合；一块都没领到（叫停、队空）就收工的卡照样进。这一口径是改前 `outcome.cards.len()` 带下来的（改动全文里 `-    outcome.cards_used = outcome.cards.len();` 一行），判决只要求「同一张卡只算一次」。造出来的数：
   ```
   CARDS_USED_NEW ended=EveryBlockStored rounds=1 stored=10 cards_used=1 cards_that_judged_a_block={0, 1} judged_by_card=[3, 7]
   ```
   随机 20000 格里 `cards_used` 比真判过块的卡数少的 3487 格、多的 1076 格（`flags`）。只影响汇总行 `CRASH_AMPLIFICATION_GPU_TOTAL cards_used=`，不影响判定。
2. **叫停文件被删的窗口（读码推，造不出）。** 叫停的判据是文件在不在：「.is_some_and(|path| Path::new(&path).exists())」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1386`），不单调。卡线程读到叫停收工之后，外层循环另读一次：「let stop = if stop_is_requested() {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3105`）。两次读之间文件被删，这一次读到没叫停，而队里有没领走的块、没有卡报 OOM，`after_the_round` 里「!blocks.any_left() || stop == StopOfTheRun::Requested,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2300`）panic，这一趟判红。改前同一族断言也如此，不是这批引进的；模型里事件只来自卡线程，卡收工后没有事件，造不出（`UNSTOP_AFTER` 两组各 10000 格 0 项）。窗口是最后一张卡收工到第 3105 行之间的几微秒量级（推的，没量）。
3. **叫停轮多借。** 再起的那一轮（`AnotherRoundOnlyForTheBlocksTheJudgedBlocksWaitFor`）借草稿区时数的是队里所有没领走的块：「let pending_states: u64 = rounds」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2942`），而叫停下真能领的只有比最大判完块号小的。借的量是 `min(要判的状态数, 每张卡此刻带得了的)`，所以多借的上限是每张卡此刻的带得了量，只持到这一轮收尾。模型里（每块 1 个状态）STOPSIM 事件 4 的叫停轮多算 8 个状态（10 块里只有 2 块可领），随机 20000 格合计 315 个；真单位一块是 「pub const STATES_PER_BLOCK: u64 = 1 << 16;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:124`）个状态。这一条归 K2-2，方向是「占着不用」，不是「给多了」。

### 推翻条件
- 在打上补丁的树上造出一次：叫停落在某一时点，`after_the_round` 的两条断言 panic，或有判完的块没入库（`StoppedOnRequest` / `EveryBlockStored`）、一块判两遍、库里不是前缀。
- 或造出队里一个比最大判完块号小的块不是「写最大那一块还缺的」（推翻上面的前缀论证；论证的前提是块只从队头领、只由领走的卡判完或交回）。

## 二、K1-2　K1-4 丢失缘由与交回那一刻

### (a) 比对 wgpu 的原文：做到

回调把 wgpu 给的那句与 hal 的显存不够一句比：「if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:138`），回调装在设备上：`device.set_device_lost_callback(move |reason, message| {`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:354`）。
wgpu 一侧（`wgpu-core =30.0.1` 与 `wgpu-hal`，`Cargo.toml` 里钉死 `=30.0.1`：「wgpu = { version = "=30.0.1", default-features = false」（`crates/singlefs-checker-tier/Cargo.toml:22`））原样：
```
$ grep -nF 'self.lose(&error.to_string());' wgpu-core-30.0.1/src/device/resource.rs
706:                self.lose(&error.to_string());
$ grep -nF 'device_lost_closure(DeviceLostReason::Unknown, message.to_string());' wgpu-core-30.0.1/src/device/resource.rs
5424:            device_lost_closure(DeviceLostReason::Unknown, message.to_string());
$ grep -nF '#[error("Out of memory")]' wgpu-hal-30.0.1/src/lib.rs
382:    #[error("Out of memory")]
$ cd wgpu-core-30.0.1/src && grep -nE '\blose\(' device/*.rs
device/resource.rs:706:                self.lose(&error.to_string());
device/resource.rs:5415:    fn lose(&self, message: &str) {
```
调 `lose` 的只有第 706 行一处。另一条给回调的路是设备已无效且队列清空时补发 `Destroyed`（`device/resource.rs:986` 前后，注释写「reason DeviceLostReason::Destroyed and no message」），句子是空串（`message: String::new()`）。
这几句的运行时原文（`wgpu-strings/` 独立小工程，`wgpu =30.0.1`，只打 `to_string()`，没起设备，`logs/wgpu-strings.out`）：
```
hal OutOfMemory = "Out of memory"
hal Lost        = "Device is lost"
hal Unexpected  = "Unexpected error variant (driver implementation is at fault)"
wgc DeviceError::OutOfMemory = "Not enough memory left."
wgc DeviceError::Lost        = "Parent device is lost"
wgpu::Error::OutOfMemory     = "Out of Memory"
```
- **原文换了措辞**：比对的另一头取自同一个 crate 同一版本的 `Display`，wgpu 的措辞改了两头一起变；只有当回调拿到的句子与 hal 的 `Display` 不再相同（例如 wgpu 在 `lose` 前加前缀）才会认不出，这时缘由记成 `Other`，退回改前的样子：不在同一组的活判红，不多认。造出来：`ENDINGS_NEW k1_4=cross-group cause=Other same_group=false x_told_out_of_memory=true y_told_out_of_memory=false y=None`。这一档只是错报红（fail-closed）。
- **别的错误恰好同文**：回调的句子只来自 `lose(&hal 错误.to_string())`，hal 错误只有上面三种，三句互不相同；`wgpu::Error::OutOfMemory` 写作 `Out of Memory`（M 大写），与 hal 那句 `Out of memory` 不同，不会互相混。`handle_hal_error` 里 `OutOfMemory` 与 `Lost`、`Unexpected` 并列当致命（`wgpu-core device/resource.rs:701` 起），hal 报 OutOfMemory 的来路本身就是分配失败，没有「同文而不是显存不够」的来路。
- **没造出的**：真卡上因显存不够丢设备时回调给的句子到底是不是上面那句（本机卡在跑 E174，不起设备）。**复核不了**，原因是不能起 GPU 设备；现有证据是读 wgpu 源码两行加运行时 `to_string()`。
- 单测在 checker 档（`gpu_unit_checks::tests::a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short`），我没跑；它的断言我读过：认 `OutOfMemory` 一句、三句别的认 `Other`、回调记丢失之后一段没记过 OOM 的活认成显存不够，三层都有。

### (b) 交回那一刻的两段窄窗口：两形做到，窗口如实现员所说还在

读码：活交回的那一刻读设备丢没丢：「let device_when_the_work_returned = card.device_condition().device_when_the_work_returns();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2200`），在 `catch_unwind` 之后、弹错误范围与等同组之前；设备没丢的活不走丢失那一支：「DeviceWhenTheWorkReturned::NotLost => None,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1757`）。
造出来（`endings_new.rs`，只用 harness 档公开函数，次序由各格给定，`logs/endings_new.log` 原样）：
```
k1_4=cross-group cause=GpuMemoryShort same_group=false x_told_out_of_memory=true y_told_out_of_memory=true y=Some(设备因显存不够丢了（Unknown：Out of memory）)
k1_4=repeated runs=2000 same_group_y_told=2000 cross_group_y_told=2000
k1_4=genuine-panic cause=GpuMemoryShort b_loses_before_a_reads=false a_returned=NotLost a_told_out_of_memory=false a=None
k1_4=genuine-panic cause=GpuMemoryShort b_loses_before_a_reads=true a_returned=Lost a_told_out_of_memory=true a=Some(设备丢了（Unknown：Out of memory），丢之前报过 序号 0 的)
k1_4=real-out-of-memory cause=GpuMemoryShort window_one=false y_told_out_of_memory=true y=Some(设备因显存不够丢了（Unknown：Out of memory）)
k1_4=real-out-of-memory cause=GpuMemoryShort window_one=true y_told_out_of_memory=false y=None
k1_4=hang-probe outcome=Ok((280, 210, 70))
```
（每行前面的 `ENDINGS_NEW ` 省了。）
- 甲形（不同组、一段因设备丢了先失败）：Y 认成显存不够，2000 次 2000 次（上一轮攻方同一形是 `y_told_out_of_memory=false`，同组对照 true）。
- 乙形（真 panic，之后同块别段致命 OOM 丢设备）：A 先读到设备没丢，等齐之后仍是 `None`，照 panic 抛。
- 挂住（攻方丙形，280 组 × 1–32 段，含起线程失败 70 组）：60 秒内全收齐，`Ok((280, 210, 70))`。
- 窗口 2（真 panic 说成显存不够）：B 的致命 OOM 让设备丢了发生在 A 读「交回那一刻」之前（`b_loses_before_a_reads=true`），A 的真 panic 被说成显存不够，结局是「本次未跑」（不是通过）；要 B 的致命 OOM 恰落在 A 的 panic 到读之间。
- 窗口 1（真显存不够判红）：wgpu 先置设备无效、后调回调（「self.valid.store(false, Ordering::Release);」在 `wgpu-core-30.0.1/src/device/resource.rs` 第 5420 行，回调在第 5424 行），一段活恰在两行之间因设备无效失败并先走到读，读到没丢，回调之后才记（`window_one=true`）→ 认不出、判红。这一形是错红（fail-closed）。
- 两个窗口的宽度**没量**：窗口 1 是同一线程里两条语句之间，要另一条线程在其中失败并走完 panic 到读，约要调 `lose` 的那条线程在两句之间被抢占；窗口 2 含 panic 展开与 panic 钩子打印，比窗口 1 长。都是推的。

还有一形判决没点名、实现员报告也没列：设备因别的缘由丢了（驱动重置，`Other`），而这段活开始之后同设备别的活记过一次 OOM，这段活（自己没报过）失败时被说成显存不够，句子里写「丢之前报过」，实际没核先后（`out_of_memory_explaining_a_failure_of_the_work` 的或模式：「CauseOfTheDeviceLoss::GpuMemoryShort | CauseOfTheDeviceLoss::Other,」在 `crates/singlefs-harness/src/gpu_memory_ledger.rs:1763`）。真值表原样（节选，`logs/endings_new.log`）：
```
table returned=Lost cause=Other loss_recorded=true other_work_reported_oom=true => Some(设备丢了（Unknown：x），丢之前报过 别的段的 OOM)
table returned=Lost cause=Other loss_recorded=true other_work_reported_oom=false => None
table returned=NotLost cause=Other loss_recorded=true other_work_reported_oom=true => None
```
这是 doc 写明的设计（「为别的丢的而这之后记过 Out of Memory…交两样都说的一句」），后果同窗口 2：本次未跑、不是通过。记下。

### 推翻条件
- (a)：真卡上因显存不够丢设备时回调给的句子不等于 `Out of memory`，或 wgpu 另有一条给回调的路用同一句而不是显存不够。
- (b)：造出一次交回那一刻设备**没丢**、之后同块别段丢了设备，而这段活仍被说成显存不够（推翻 `NotLost => None` 那一支）；或一次设备已丢、没有别的原因的真显存不够被判红（窗口 1 之外）。

## 三、K1-3　「照叫停收尾」：没判的块都留给下一趟、库里没有缺块

**判：做到。** 实现员第 2 处推翻上一块钉的结局，结局表现在是（`after_a_round_of_the_cards`，`crates/singlefs-harness/src/gpu_memory_ledger.rs:2171` 起）：一块都没剩 → `EveryBlockStored`；这一轮没有卡报 OOM → `StoppedOnRequest`（`if cards_short_of_memory_this_round == 0 {`，第 2174 行）；叫停而有卡报了：「StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,」（第 2178 行）；有等着的块，还有没报过的卡 → 再起一轮，没有卡 → 本次未跑（第 2179 行起）。
- `StoppedOnRequest` 两个来路都保证没有判完而没写的块（没有卡报时轮末断言保证，有卡报时 `blocks_waiting == 0` 是前提），所以库里写进去的是块号 0..k 连着的前缀，判完的块全在里面；没判的块留在队里，下一趟 `pending_blocks` 按库里有没有判定块重新算（`crates/singlefs-checker-tier/src/crash_amplification.rs:1961`），自然成为下一趟的。外层循环 `StoppedOnRequest => break`（「AfterARoundOfTheCards::StoppedOnRequest => break,」，第 3128 行），交回 `Judged`，调用方走被叫停的收尾：「let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();」（第 5591 行）。
- 实测（K1-1 同一批 20000 格）：`StoppedOnRequest` 2218 格里，判完的块数不等于写进库的块数、或收尾时还有等着的，0 格；`stopped_without_stop` 0 格。
- 一形代价：叫停而有卡报 OOM、判完的都入库的情形，改前打「本次未跑」那一行并置 `gpu_memory_was_short`，现在 `StoppedOnRequest => break` 什么也不打，这一趟的 OOM 缘由没有地方留（第一句 OOM 的文字只在本次未跑的两处打出来：第 3030 行那一处与 `NotRunTheGpuMemoryWasShort` 那一支）。观测损失，不影响库与判定。
- 一形依赖：被叫停的收尾靠第 5591 行再读一次叫停文件；文件此时已被删，则这一趟不走被叫停的收尾，往下走第 ② 段核对，库里却有没判的块（与 K1-1 小漏 2 同一个窗口，改前叫停收尾同样依赖它，推的）。
- 「本次未跑」那一支（`NotRun`）里判完而没写的块白判：20000 格里 326 格有判完而没入库的块（`notrun_judged_unstored`；`NotRun` 共 4767 格，其中叫停的 36 格）；库里没有缺块，下一趟重判，不是漏判。

### 推翻条件
- 造出一次 `StoppedOnRequest` 之后库里的块不是 0..k 前缀、或判完的块没入库、或没判的块下一趟的 `pending` 里没有它。

## 四、K2　只防 OOM、不压并发两头（新代码引进来的）

### K2-1　有没有一形让账给的量大于此刻真能用的：没找到

- 账本代码本批没动。改动全文第一处 harness 档改动在旧文件第 1560 行：`grep -n '^@@' research/prompts/_m3-batch10z-e-code-r1-diff.md | head -2` 原样 `10:@@ -1560,7 +1560,32 @@ pub struct DeviceConditionOfACard {` 与 `44:@@ -1640,20 +1665,25 @@ impl DeviceConditionOfACard {`；账本在它前面：`gpu_memory_ledger.rs` 里 `pub struct GpuMemoryLedger {` 在第 910 行、`pub fn card_room_for_this_process(` 在第 242 行、`pub fn decide_booking_by_priority(` 第 272 行，`DeviceConditionOfACard` 在第 1559 行。
- 新代码对账的唯一接口是设备丢失：缘由 `CauseOfTheDeviceLoss` 只被 `out_of_memory_explaining_a_failure_of_the_work` 读来认一段活的失败（第 1760–1772 行），不进账；卡池判留着的设备还能不能拿用的是 `is_lost()`（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1277`），语义没变（只记不抹）。
- 叫停轮多借（K1-1 小漏 3）是借得比用得多，方向相反，归 K2-2。
- G2 甲认的少给、设备丢了账没被问到那一下读数多给：已知不在这一轮范围，我没碰。

### K2-2　除甲认的少给之外，有没有一形卡放得下却报本次未跑、判红、或整趟弃卡

- **判红的新来路**：只有轮末两条断言（K1-1 小漏 2 的窗口，叫停文件被外部删）与 `give_back` 的断言（第 1969 行起，一块只交回一次；M22 是它的变异，现有测试没钉，见 K3）。20000 格随机里 `red` 与 `assert_panicked` 都是 0。
- **本次未跑的新来路**：没有。`NotRun` 的来路仍是：每张没报过的卡此刻都带不了一个状态（「ScratchStepBeforeBooking::NotRunNoCardHoldsAStateNow { not_run_line } => {」，`crates/singlefs-checker-tier/src/crash_amplification.rs:2996`），或有卡报过 OOM 而剩下的卡带不了、或没有未报过的卡（`crates/singlefs-harness/src/gpu_memory_ledger.rs:2179` 起）。实现员第 2 处只是把「叫停 + 有卡报 + 判完的都入库」从 `NotRun` 挪到 `StoppedOnRequest`，方向是少报本次未跑。
- **整趟弃卡的新来路**：一个——设备因显存不够丢了，同设备上别组的段失败被认成显存不够（K1-2 甲形），那张卡这一趟标成报过、不再用，手里那一块交回。设备已丢，这一趟内不会重起（重起发生在卡池下一次借的时候），所以不是「放得下而弃」；多出来的只是原先判红的那一段现在报本次未跑。
- **叫停轮多借**（K1-1 小漏 3，模型数在那里）：占着不用，持到这一轮收尾，借量上限是每张卡此刻带得了的量。与「不压并发」同方向的小代价，我没量真账上一轮的实际多借 MiB（一个状态要多少 MiB 取决于表，`mebibytes_a_judge_needs`，没起卡不量）。
- 单卡机器：一张卡报 OOM、叫停、判完的块在等 → 本次未跑（没有别的卡判交回的那一块），与改前一样，判完的块白判（`notrun_judged_unstored`）；这是「卡放不下」的真情形。

### 推翻条件
- 造出一次账里 `card_room_for_this_process` 给的量大于卡此刻真能用的，且原因在本批新增的代码路径上（设备丢失的缘由、`device_when_the_work_returns`、`RoundsOfTheCards`）。
- 或造出一次某张卡此刻带得了至少一个状态、而新代码路径给出本次未跑 / 判红 / 弃卡。

## 五、K3　测试判别力

**判：没做到。** 实现员报的 22 条 harness 档追加行都会红（我全部重证），但自造变异里有 7 处「这一批改的」照绿，另有 checker 档胶水 4 处既没有不要卡的用例、也没有变异行。

### 5.1 已有变异行：22 条 harness 档追加行我自己重跑，全抓到

`prove-red.sh --copy <tree-mut> --memory 12G singlefs-harness <名…>`，三趟原样末行（`mut/prove-red-unproven.log`、`mut/prove-red-proven-a.log`、`mut/prove-red-proven-b.log`）：
```
✓ 点名 7 条：跑了 7 条，跳过 0 条，跑的都抓到了      （实现员报告里没证的 3、7、9、14、15、21、22 行）
✓ 点名 8 条：跑了 8 条，跳过 0 条，跑的都抓到了
✓ 点名 7 条：跑了 7 条，跳过 0 条，跑的都抓到了
```
追加行 24 条里 22 条是 harness 档，另 2 条（第 23、24 行，`gpu_unit_checks.rs` 与 `crash_amplification.rs`）是 checker 档，没跑；替换的 5 行也没跑。

### 5.2 自造变异 40 条，在副本上跑 harness 档

`mut/mutants.py`：40 条变异，原文在 `gpu_memory_ledger.rs` 测试段之前恰好命中一次（我先核了锚点，坏的 0 条）。每条改一处、跑 `cargo test --offline -p singlefs-harness --lib -- gpu_memory_ledger`（83 条）、还原；照绿的再各跑整个 `--lib`（170 条）。`mut/results.tsv`：
```
前 40 行（带过滤）：RED 31、GREEN 9      后 9 行（整个 --lib）：GREEN 9
GREEN：M01 M05 M10 M13 M14 M17 M22 M23 M33
```
照绿的 9 条里 2 条是等价变异：M01（`first < largest` 改 `<=`：最大判完块不会同时在队里）、M33（`arrive` 里「失败之后到的不再写」那个 `continue` 走不到：失败那一块在第 2104 行已从等着的里取走，写库计数不前进，后到的块取不到同一个块号；实现员同样判等价）。其余 7 条是真漏，我各补了一条会红的用例（`probes/killers_new.rs`，真代码上 7 passed，只红对应的那一条，`logs/killers-run.log` 原样）：

| 号 | 变异（改回去/改坏的是这一批的哪一处） | 为什么现有用例照绿 | 补的用例 | 变异上 |
|---|---|---|---|---|
| M05 | 判完块号记最后判完的、不取最大（K1-5 的 `largest_judged`，第 1959 行） | 现有叫停用例里块按块号升序判完，最后判完的就是最大的 | `m05_the_block_judged_last_is_not_the_largest_judged`：三张卡领 5、6、7，7 先判完、5 后判完、6 报 OOM 交回；叫停后下一张卡要领得到 6 | 红 |
| M10 | 叫停而有卡报、等着的块 `== 0` 改 `< 2`（第 2178 行，实现员第 2 处自定的结局表） | 现有用例等着 4 块、9 块、0 块，没有刚好 1 块 | `m10_one_waiting_judged_block_still_needs_another_round_or_not_run` | 红 |
| M13 | 去掉轮末第一条断言（第 2294–2298 行，判决点名的第 3081 行那一句挪进来的） | 没有用例让它触发 | `m13_a_round_without_out_of_memory_leaves_no_judged_block_waiting` | 红 |
| M14 | 轮末第二条断言改成 `true`（「!blocks.any_left() || stop == StopOfTheRun::Requested,」，第 2300 行） | 同上 | `m14_a_round_without_a_stop_leaves_no_block_untaken` | 红 |
| M17 | 头一句 OOM 取最后一句（「.get_or_insert_with(|| reason.clone());」，第 2308 行） | 没有用例看 `first_out_of_memory()` 的取值 | `m17_the_first_out_of_memory_stays_the_first` | 红 |
| M22 | `give_back` 的「只交回一次」断言去掉（第 1969 行起） | 没有用例让它触发 | `m22_giving_a_block_back_twice_panics` | 红 |
| M23 | 认 OOM 取最早一条而不是最后一条（第 1749 行起的 `.iter().rev().find(…)`） | 现有用例每段活最多记一条 | `m23_the_last_out_of_memory_of_a_work_is_the_one_told` | 红 |

分类：M05、M10 是新逻辑上的真缺口（改回去会让叫停时判完的块丢掉或报错结局，行为级）；M13、M14、M22 是安全网断言没被钉住（断言本身去掉照绿，行为不变，防的是以后别的回归）；M17、M23 只改报出来的文字，影响小。M05 的后果不只是文字：叫停后该领的块领不到，下一轮没有 OOM 却有等着的块，轮末断言 panic（由 K1-1 的论证推出，没单独造这条调度的整轮格）。

### 5.3 checker 档胶水：没有不要卡的用例，也没有变异行（读码判，没跑）
在冻结副本的 `crates/mutations.tsv` 里，取第 2 列是这两份文件的行，打出第 3、5、6 列，再 `grep -E` 带 `rounds`、`stop_of`、`device_when`、`cards_used`、`any_left_to_take`、`WorkEnding`、`ending_together`、`cause_of`、`record_the` 的，原样只命中第 1445、3303、3350 行三条（这两份文件在第 3300 行之后共 5 行：3302–3304、3349、3350；3349 是 `cause_of_the_device_loss` 本身，测它的是纯函数用例，不属胶水；1445 的测试名 `the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest` 在 `--test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 里，要卡；3303、3350 是不要卡的）。下面四处改回去，没有任何一条不要卡的用例会红：
1. 交回那一刻读设备丢没丢的位置与参数（`crates/singlefs-checker-tier/src/crash_amplification.rs:2200`）：挪到弹范围之后或等齐之后，或写成常量 `Lost` / `NotLost`。harness 档只钉了 `device_when_the_work_returns` 这个函数与参数怎么用（M29、M30 红），不钉 checker 档调它的位置。
2. 外层循环交给 `rounds.after_the_round` 的参数：`cards_of_the_round`（`run_card_indices`）、收工的卡 `finished_card_indices`、第 3105 行读的 `stop`、`blocks: rounds.take_the_blocks_of_the_round(),`（第 3045 行）。
3. `outcome.cards_used = rounds.cards_used();`（第 3163 行）：改成 `outcome.cards.len()` 照绿。
4. 起下一张卡之前的「看叫停」：「if work.blocks.any_left_to_take(work.stop_of_the_cards()) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2467`）改回 `any_left()`：harness 档只钉 `any_left_to_take` 函数本身（M07），调用点没钉。
这四处和实现员报告「抽不出、没有不要卡的用例的」那一段对得上，我只是核了变异表确实没配行。

### 推翻条件
- 把 `probes/killers_new.rs` 那 7 条放进 harness 档后，M05 / M10 / M13 / M14 / M17 / M22 / M23 任一条仍照绿，或这 7 条在真代码上红。
- 或找到 5.3 里某一处已有一条不要卡的用例会红（我只读了变异表与 `grep` 用例名，没逐个读 checker 档所有测试）。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 做到，有三处小漏 | 叫停落在任一轮任一时点：20000 格随机 + 叫停文件消失两组各 10000 格，轮末断言 panic、判两遍、库不是前缀、缺块、判红都是 0；上一轮 `stop-in-the-closing-round` 三格与 STOPSIM 现在都入库；「在等的」判法有前缀论证、两个方向都推不出反例。漏：`cards_used` 口径没动（不计判了块又 OOM 的卡、计一块没领的卡）、叫停文件被外部删的窗口会让轮末断言 panic（改前同族）、叫停轮按全部剩下的块借草稿区（多借） |
| K1-2 (a) | 做到 | 比对的两头取自 `wgpu =30.0.1` 同一个 `Display`，调回调的只有 `lose(&hal 错误.to_string())` 一处，三句互不相同；换措辞只会退回改前（判红，fail-closed），同文碰撞没有来路。真卡上回调给的句子复核不了 |
| K1-2 (b) | 做到，两窗口还在 | 甲形、乙形都按判决改对（Y 认成显存不够 2000/2000，真 panic 设备没丢时 `None`）；窗口 1 真显存不够判红、窗口 2 真 panic 说成显存不够（结局本次未跑，不是通过）；另一形：驱动重置丢的设备 + 别的段报过 OOM，被说成显存不够（设计里写了），都是推的宽度 |
| K1-3 | 做到 | `StoppedOnRequest` 两个来路都保证判完的块全入库、库里是 0..k 前缀、没判的下一趟 `pending` 重算；代价是叫停 + OOM 时不再打本次未跑那一行，OOM 缘由没有地方留 |
| K2-1 | 没找到 | 账本代码本批没动（第一处 harness 改动在旧文件第 1560 行，账本在第 910 行起）；新增的缘由不进账 |
| K2-2 | 没找到会判红 / 本次未跑 / 弃卡的新形；有一形多借 | 新来路只有「设备因显存不够丢了，同设备别组的段被认成显存不够」的弃卡，设备已丢本趟内不重起；叫停轮多借至每张卡此刻带得了的量，持到这一轮收尾 |
| K3 | 没做到 | 实现员 22 条 harness 档追加行我全部重证、全抓到；自造 40 条变异 9 条照绿（2 条等价），7 条真漏（M05、M10 行为级，M13、M14、M22 安全网，M17、M23 文字），补 7 条会红的用例；checker 档胶水 4 处没有不要卡的用例也没有变异行（读码判） |

## 七、要主 agent 跑的命令 / 复核不了的

- 把 `probes/killers_new.rs` 的 7 条并进 `gpu_memory_ledger.rs` 的测试段，并为 M05、M10、M13、M14、M17、M22、M23 各加一行变异（锚点与替换文在 `mut/mutants.py`）。验证：`CARGO_TARGET_DIR=<编译目录> bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test killers_new`，看 `test result: ok. 7 passed`；变异上各红一条（`python3 mut/run_killers.py M05 M10 M13 M14 M17 M22 M23`，路径要改成自己的）。
- 复核不了（要真卡 / checker 档）：真卡上因显存不够丢设备时回调给的句子；两个窗口的实际宽度；checker 档新增两条用例（`the_pieces_of_one_block_report_their_endings_in_one_group`、`a_device_lost_for_lack_of_memory_explains_a_failure_on_it_as_gpu_memory_short`）我只读了没跑；替换的 5 行与追加的第 23、24 行变异。
- 5.3 的四处胶水要加用例，需要把外层循环里「交 `after_the_round` 的参数」与「读设备丢没丢」抽成不要卡的函数再测；实现员报告把它们记作抽不出，我只确认了变异表里没有配行，没判抽不抽得出。

## 八、没做什么

- 正文第三节 K1–K3 之外的格、采纳与判决，没碰；没读攻方这一轮的任何产物。
- 没跑 checker 档任何测试，没起 GPU 设备、没占卡；`nvidia-smi` 只读。
- 没跑重型测试与全量门禁；没做任何 git 写操作；没写 `crates/` 与仓里除了报告与模型目录之外的文件（所有改源码的变异都在草稿目录的副本上，已还原：`cmp` 与冻结副本原文逐字节相同）。
- 装卡与账的数是模型（每张卡每轮一个带得了的状态数），外层循环与卡线程是按冻结副本的次序抄的，抄得对不对是推的；没有拿真 `CardsAtWork` 跑。
- 两个窗口的宽度、叫停文件被删的窗口宽度、叫停轮多借的真 MiB 都没量。
- 没逐个读 checker 档的全部测试来确认 5.3 四处「没有用例」，只核了变异表与用例名。

## 九、清理

交回前删了自己建的编译目录与仓副本（`du -sh` 记的大小）：`~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet/target`（1.1G）、`target-mut`（1.5G）、`target-strings`（606M）、`tree`（21M）、`tree-mut`（21M）。没删（小、入库材料的来源或日志）：`~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet/model`、`mut`、`logs`、`wgpu-strings`（40K）——模型源码、脚本、日志已拷进 `research/prompts/m3-batch10z-e-code-r1-sonnet-model/`，草稿目录里的是同一份。
