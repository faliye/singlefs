# 根环实现第 10z 批第五块代码轮第一轮：攻方报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 H1 H2 H3 H4 H5 H6 H11 H12 H19 C1 C2 C3 C4 F1 F2 F3 F4 F5 -->

轮名 m3-batch10z-e-code-r1，攻方腿。被判对象是冻结副本 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/`（开工时 `sha256sum -c research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt` 334 行 OK、0 行不对）。下文引代码写冻结副本的绝对路径与它自己的行号；模型、副本上的数都不是入库装置上的数。

## 复跑

```
# 仓根。先拷一份冻结副本（冻结副本本身只读），编译目录单独一个，不与别的拷贝共用
rsync -a --exclude target --exclude .git ~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/ <拷贝>/
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 \
  bash research/prompts/m3-batch10z-e-code-r1-opus-model/run.sh <拷贝> <编译目录>
# 改回去的探针（每条改完、跑完即还原）
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 python3 -I \
  research/prompts/m3-batch10z-e-code-r1-opus-model/mutate_10ze.py <拷贝> <编译目录> H1-largest-is-the-last-judged H19-stop-with-one-waiting-block-stops …
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 env PYTHONDONTWRITEBYTECODE=1 python3 -I \
  research/prompts/m3-batch10z-e-code-r1-opus-model/show_mutant_10ze.py <拷贝> <编译目录> H1-largest-is-the-last-judged
bash scripts/hosts/run-with-memory-cap.sh 12G bash scripts/hosts/capped.sh 6 python3 -I \
  research/prompts/m3-batch10z-e-code-r1-opus-model/rows_10ze.py <拷贝> <编译目录> 3329 3333 3335 3340 3341 3347 3348
# 窗口宽度另按 RUST_BACKTRACE 不设、=1、=full 各跑一次 attack_10ze_window（output-window.txt）
```

`attack_10ze_rounds` 的调度按种子定，同一 `ATTACK_10ZE_SCRIPTS` 两次跑计数逐字相同（变异之前那一次与最后那一次 `diff` 只差窗口测试的名字行）；`attack_10ze_window` 量时长，每次不同。

模型目录 `research/prompts/m3-batch10z-e-code-r1-opus-model/` 每个文件的 sha256：

```
e9e08ed2d38f6500257de18275a0fff4ca5e466d32b0104b6b03a1cb67eb4083  research/prompts/m3-batch10z-e-code-r1-opus-model/attack_10ze_rounds.rs
16e1ac1e577da394a5885dc6b75e3b374cb097a29cde5b97dfb5e47284ada688  research/prompts/m3-batch10z-e-code-r1-opus-model/attack_10ze_window.rs
2b73ef7d809395775091a282c1ee08574b8f9eebf650371c68f2a162782d4bb9  research/prompts/m3-batch10z-e-code-r1-opus-model/mutate_10ze.py
eb1beddf2d89bda829d62de071cac604c70d0804c8f4dfed9099defe19d6d4ad  research/prompts/m3-batch10z-e-code-r1-opus-model/output-models.txt
36b214f364e7aa8df65a01877fa0fd5305cb9dc41be134b9ce455c325c777bcb  research/prompts/m3-batch10z-e-code-r1-opus-model/output-mutant-details.txt
112edb4cb21c9dcd460a67964947d22bcfd849ba021eae4de8b26f39e65e9a7c  research/prompts/m3-batch10z-e-code-r1-opus-model/output-mutants.txt
8c1f55b70264c16f1b7ffeb4d9e1f8d738128f3c91f988a9eb72d1d71fc25ffc  research/prompts/m3-batch10z-e-code-r1-opus-model/output-rows.txt
0e58aae56a1122ae7537f31f49513e0f16fb3874412d1705e58d9d7fb57ee36a  research/prompts/m3-batch10z-e-code-r1-opus-model/output-window.txt
84084537820956e4ab5bf94ea5c2c621088c3035387df81de02d75271c83005f  research/prompts/m3-batch10z-e-code-r1-opus-model/rows_10ze.py
f9f2785cd13a865a0b444b9113773c8651159f96ffcf0763b74b703badfe8020  research/prompts/m3-batch10z-e-code-r1-opus-model/run.sh
01929f514889c7041ef1481891a90c0922179294621bfd9321e5f00008bbdc66  research/prompts/m3-batch10z-e-code-r1-opus-model/show_mutant_10ze.py
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节） |
|---|---|---|
| K1-1「在等的」按比最大块号小判 | 没打中 | 推的证明：没判完、比最大块号小的块一定是判完的最大那块在等的，反过来也成立，判法与「在等的」逐块相同；模型叫停只出现一臂 200000 段历史：不 panic、不判两遍、照叫停收尾时判完的块全入库（量过，第一节） |
| K1-1 叫停加有卡报 OOM 加判完全入库改成照叫停收尾（实现员第 2 处） | 没打中（K1-3 那一问） | 同一批 200000 段：照叫停收尾 55771 段，库里只有块序前缀、没判的都留在队里，判完没入库的 0 段（量过）；本次未跑那一行不再打，见第二节 |
| K1-2 (a) 比对 wgpu 原文 | 没打中（这一批新的） | 措辞变：wgpu 钉死 `=30.0.1`，回落判红（推的）；同文：主机内存不够、显存碎片、gpu-allocator 的 OOM 都给同一句，说成显存不够——错误范围那一路改前就这样合并，不是这一批放宽的（读码，第三节） |
| K1-2 (b) 交回那一刻的两段窄窗口 | 没打中新形状；两段宽度量了 | 第二段：panic 到记下，默认钩子首次 49–90 µs、之后中位 6–9 µs，`RUST_BACKTRACE=1` 首次约 7.8 ms；第一段由丢设备那条线程在 5420、5424 两行之间被调走多久定，此刻本机两次相邻读钟之间最长停过约 1–2 ms（量过，harness 档进程，不是卡上；第四节） |
| K2-1 账给的大于真能用的 | 没打中 | 这一批 harness 库的改动从第 1560 行起，账的算术一行没动，丢失缘由只进解释、卡池不读它（读码，第五节） |
| K2-2 放得下却本次未跑、判红、弃卡 | 打中（窄），一半是这一批带进来的 | 叫停文件在一轮收尾附近被删：断言 panic「没人叫停，却有 4 块没判」（模型 1369/200000，改前就有这一形）；照叫停收尾、整趟却没记成提早收尾，进对账判红（2799/200000，其中最后一轮有卡报 OOM 的 1352 段是第 2 处改动带进来的：改前这些段报本次未跑、恒记成提早收尾）。真机上要在微秒级的缝里删文件才中（推的，第六节） |
| K3 改回去照绿 | 打中 | harness 档两处改回去 lib 全绿、模型红：判完最大块号改成记最后判完的（叫停只出现也 panic，11/20000）；在等 1 块也照叫停收尾（判完的块没入库，71/20000）（量过）。checker 档交回那一刻的记法改成恒记「丢了」、`cards_used` 改回 `outcome.cards.len()` 都没有用例拦（推的，第七节） |

## 一、K1-1「在等的」按比判完的最大块号小来判

被判的那一句：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1906`：「self.largest_judged.is_some_and(|largest| first < largest)」；最大块号这样记：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1959`：「.map_or(pending_index, |largest| largest.max(pending_index)),」，判完一块、交写库之前记：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:2052`：「blocks.record_a_judged_block(pending_index);」。

**推的证明（两个方向都成立，判法与「在等的」逐块相同）。** 设 m 是此刻判完的块里块号最大的，k 是队里一块。

- 在等的 ⇒ k < m：k 被某个判完、没入库的块 j 等着，按块序写库 j > k，而 m ≥ j。
- k < m ⇒ 在等的：k 没判完（在队里），写库按块序，m 要写进库就得先写 k，所以 m 判完了却还没入库、在等 k。

所以实现员报告里那句「比最大块号小的那块可能没人在等，也照领（多判一块）」是空的：叫停之后领到的每一块都有判完的块在等。在等的块只会来自交回（不交回时队里的块都比领过的大，见下一段），交回发生的那一轮有卡报 OOM、不进轮末断言，下一轮带着 m（`left_for_another_round`）。

没交回时叫停也不会留下在等的：卡总是领队里最小的，所以队里每一块都比此刻领走的任何一块大，也就比 m 大；m 之后才变大的情形要一块比队头更大的块正在判，而那块是在队头之前领走的，与「领最小的」矛盾。

**量过（模型，不是入库装置）。** `attack_10ze_rounds.rs` 把 checker 档外层循环、卡线程、起下一张卡、`stop_of_the_cards` 照次序抄成不碰卡的一份，harness 档的 `BlocksForTheCards`、`judge_blocks_on_one_card`、`RoundsOfTheCards` 原样调用；卡线程每一步（建判器、问叫停、判一块、交写库）之前停下，调度器按种子挑一条放行。随机取 2–3 张卡、3–7 块；每张卡第 0 轮装 1 个状态、四分之一的卡第 1 轮起装 0 个；建判器在第 0 轮或第 1 轮报 OOM 各 10%；一半的卡在它判的第 1–3 块报 OOM；叫停文件四分之三的段在第 1–40 个事件出现、只出现不消失；每份取法跑 4 个调度种子。用户决定的那一步（什么时候叫停）是放开扫的，只固定卡数、块数与 OOM 的落点。

输出原样（`research/prompts/m3-batch10z-e-code-r1-opus-model/output-models.txt`）：

```
test k1_5_monotonic_stop_random_rounds ... ATTACK arm=monotonic-stop scripts=50000 runs=200000 outcomes={"EveryBlockStored": 109985, "NotRun": 34244, "StoppedOnRequest": 55771} counts={"not_run_with_judged_blocks_not_stored": 3992, "stopped_on_request_with_an_out_of_memory_in_the_last_round": 27661} scripts_with_both_stopped_and_not_run=530
```

`panic_or_loop_or_err`、`judged_twice`、`stopped_with_judged_blocks_not_stored`、`every_block_stored_but_short` 都没出现（这一臂的测试照这四条断言为 0，跑过）。`not_run_with_judged_blocks_not_stored` 3992 段是本次未跑时判完、在等交回块的那几块不入库（交回的块没有卡能判），下一趟重判，是少判不是判错；`scripts_with_both_stopped_and_not_run=530` 是同一份取法、不同的线程先后，一次照叫停收尾、一次本次未跑：差在交回的块落在叫停之前还是之后，两种结局都提早收尾。

`cards_used`：`RoundsOfTheCards::cards_used` 是收过工的卡按序号去重，几轮里同一张卡只算一张，对；报了 OOM 的卡发的是 `OutOfMemory` 不是 `Finished`，它判过的块不进这个数（改前 `outcome.cards` 也只收 `Finished`，同义），只影响汇总行。读码；改回去没有用例拦，见第七节 C2。

推翻条件：在入库装置上照这几步次序（叫停只出现、不消失）造出轮末断言 panic、同一块判两遍、或照叫停收尾时有判完的块没入库。

## 二、实现员第 2 处：叫停、这一轮有卡报 OOM、判完的都入库时照叫停收尾（K1-3 那一问）

被判的那一句：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:2178`：「StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,」；checker 档照它收尾：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3128`：「AfterARoundOfTheCards::StoppedOnRequest => break,」。

- **没判的是不是都留给下一趟、库里有没有缺块**：写库只照块序写前缀（`BlocksStoredInOrder::arrive`），下一趟的 `pending` 是库里没有的块，所以库里不会有洞。模型那一臂 200000 段里照叫停收尾 55771 段，其中最后一轮有卡报 OOM 的 27661 段，判完没入库的一段都没有（第一节那一行的 `stopped_with_judged_blocks_not_stored` 不出现）。量过，模型。
- **结局随不随先后变**：实现员说的那一形（交回的块同一轮被别的卡领走，还是落到收尾那一轮）由他的用例 8 与用例 1 第 2 格钉着。模型里同一份取法换线程先后，既有照叫停收尾也有本次未跑的有 530 份；按决策表推，差在叫停落在一轮收尾那一读之前还是之后，是叫停时点的差，不是交回先后的差（推的：这 530 份没逐段拆开看）。
- **丢了的东西（读码，推的）**：改前这一形报本次未跑，打一行 `gpu_not_run_line`；改后走 `Judged` 那一支（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:5529`：「(tally, Some(outcome), None, None, false)」），那一行不打，`gpu_memory_was_short` 是 false，这一趟有卡显存不够只剩 `rounds.first_out_of_memory()` 里，外面读不到。两种结局都提早收尾、分项驱动都记成本次未跑，只是缘由那一行没了，不算打中；但它让「这一趟记不记成提早收尾」改成全靠第 5591 行再读一次叫停文件，见第六节。

推翻条件：照叫停收尾之后下一趟的 `pending` 漏了某一块（库里有洞），或照叫停收尾时有判完、在等的块被丢掉而又没记成本次未跑。

## 三、K1-2 (a) 丢失缘由比对 wgpu 原文

被判的那一句：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:138`：「if message == wgpu::hal::DeviceError::OutOfMemory.to_string() {」。wgpu-core 丢设备时给的那一句：`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:706`：「self.lose(&error.to_string());」，hal 那一句：`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/lib.rs:382`：「#[error("Out of memory")]」。

- **措辞换了**：wgpu 钉死在 `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/Cargo.toml:22`：「wgpu = { version = "=30.0.1"」，只有手动升版才变。升版之后两种变法：hal 那一句的措辞变——比较两边取自同一个 `to_string()`，照认；wgpu-core 不再拿 `error.to_string()` 当丢失那一句（比如加前缀或换 Debug）——认不出，回落成 `Other`，退回改前的样子：这段活之后没记过 OOM 的判红。不会多认。推的，读码。这一种变法用例拦不住，见第七节 K3-3。
- **别的错误恰好同文**：hal 里给「Out of memory」的不止显存。`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/mod.rs:1581`：「vk::Result::ERROR_OUT_OF_HOST_MEMORY | vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => {」（主机内存不够同一句），`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/vulkan/mod.rs:1607`：「vk::Result::ERROR_FRAGMENTATION => get_oom_err(err),」，`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/lib.rs:394`：「gpu_allocator::AllocationError::OutOfMemory => Self::OutOfMemory,」。这几样丢设备都认成 `GpuMemoryShort`，同设备上之后失败的活都说成「显存不够」→ 本次未跑，那张卡这一趟弃掉。主机内存不够说成显存不够是误称；但改前错误范围那一路已经这样合并（wgpu 把这几样都归 `Error::OutOfMemory`，范围接到的照样说成显存不够），这一批只是把同一种合并带到丢设备那一路，没有新放宽一类错误。卡显存放得下却因主机内存不够被弃，属 K2-2「弃卡」的字面，但改前就有、不是这一批带进来的。推的，读码；主机内存不够让驱动给 `ERROR_OUT_OF_HOST_MEMORY` 而不是被内核杀掉，本机没造过。
- 显存预算那一路（`lose_if_oom`）只有设了 `memory_budget_thresholds.for_device_loss` 才会因预算丢设备，`crates/` 里零命中（`grep -rn memory_budget_thresholds` 没有输出），不在射程里。

推翻条件：wgpu 30.0.1 里有一条丢设备的路给的那一句恰是「Out of memory」，而它的缘由与内存（主机或显存）无关。

## 四、K1-2 (b) 交回那一刻的两段窄窗口

记法：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2198`：「let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));」之后才 `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2200`：「let device_when_the_work_returned = card.device_condition().device_when_the_work_returns();」。设备丢了那一支只给交回那一刻已丢的：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1757`：「DeviceWhenTheWorkReturned::NotLost => None,」。

**第二段（真 panic 说成显存不够）**：活真 panic 的那一刻到第 2200 行之间，是 panic 钩子加展开的时长。这段时间里同组别的段撞上致命的 OOM、设备丢了，这段活就记成「交回那一刻丢了」。量过（harness 档测试进程里同一种写法，`attack_10ze_window.rs`，不是卡上的活），`output-window.txt` 原样：

```
WINDOW rust_backtrace=None panics=50 first_micros=49 rest_median_micros=9 rest_max_micros=23
WINDOW rust_backtrace=Some("1") panics=50 first_micros=7782 rest_median_micros=220 rest_max_micros=283
WINDOW rust_backtrace=Some("full") panics=50 first_micros=8809 rest_median_micros=1134 rest_max_micros=1214
```

默认钩子首次几十微秒、之后个位数微秒；设了 `RUST_BACKTRACE` 的进程首次 panic 要取、解回溯，约 8–9 毫秒。仓里的脚本、门禁阶段都不设它（`grep -rln RUST_BACKTRACE scripts .claude/gate.d crates` 没有输出），此刻环境里也没设。checker 档二进制更大，解回溯更慢是推的。活在展开时丢的局部量（缓冲、映射）没有等卡的 Drop，读码，推的。

**第一段（真的显存不够判红）**：`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:5420`：「self.valid.store(false, Ordering::Release);」与 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:5424`：「device_lost_closure(DeviceLostReason::Unknown, message.to_string());」之间，设备已经无效、丢失还没记。另一段在这当中因设备无效失败：它的调用拿到的是 `DeviceError::Lost`（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/mod.rs:357`：「Self::Lost => ErrorType::DeviceLost,」），这一类 wgpu 不交错误范围也不交没接住的处理（`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-30.0.1/src/backend/wgpu_core.rs:304`：「ErrorType::DeviceLost => return, // will be surfaced via callback」），所以它自己的范围里什么都没有；要是它连展开带记一起在这段里做完，就记成没丢、判红，与实现员写的一样。两行之间只有取一把锁，正常是微秒以下；能拉宽它的是丢设备那条线程恰在这两行之间被调走。量过此刻本机（6 条线程各量 1 秒，`nice -n 19`）两次相邻读钟之间被停的最长时长，两次分别约 2 ms、1 ms（`output-window.txt` 与 `output-models.txt` 的 GAP 行）。真 panic 那一段自己从 panic 到记下也要几微秒以上（上一段的数），所以要中，得丢设备那条线程被调走、同时另一段恰在那几毫秒里撞上无效设备、panic、展开完。推的，造不出：要卡。

这两段之外没找到新形状。另一种被说成显存不够的：设备在这段活开始之前就因显存不够丢了（同一测试进程里别的用例共用卡池的设备），这段活失败也说成显存不够、本次未跑（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1770`：「Some(format!("设备因显存不够丢了（{}）", loss.why))」不看 `reports_before`）。这是设计：设备已经丢了，活本来就做不成；它会把一段同时有真 bug 的活这一趟盖成本次未跑，下一趟设备好的时候照红。不算打中。

推翻条件：在卡上造出一段活在设备丢之前真 panic、却被说成显存不够，而那段活从 panic 到交回远长于上面量的几十微秒（例如展开里有等卡的 Drop）。

## 五、K2-1 账给的量大于此刻真能用的

没打中。读码：改动全文里 harness 库的第一个 hunk 头是 `@@ -1560,7 +1560,32 @@ pub struct DeviceConditionOfACard {`，账（`GpuMemoryLedger`，冻结副本第 910 行起）的借、销、读数与重起判一行没动；新加的 `CauseOfTheDeviceLoss` 只进 `out_of_memory_explaining_a_failure_of_the_work` 的解释，卡池照的仍是 `is_lost`（丢了就重起、重起照此刻判）。第 ① 段新加的一轮（只判在等的那几块）照旧走 `scratch_of_each_card_booked_by_where_the_cards_came_from` 现读、照此刻能借的再压一次再借，`pending_states` 算的是下一轮队里全部的块（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2942` 起），比只判在等的那几块要多，但每张卡装的状态数受此刻借得到的封顶，多算只让多一张卡上卡，不让账多给。用户已认的同读一份空闲、一份报告里约 1.3 ms 的窗口不算。

推翻条件：在这一批新加的路径上造出一段历史，账上放行的量大于那一刻卡上真能用的，而这段历史不经上面那两种已认的窗口。

## 六、K2-2 放得下却本次未跑、判红、弃卡

### K2-2-1 叫停文件在一轮收尾附近被删（打中，窄）

叫停是每次现读文件在不在：`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:1386`：「.is_some_and(|path| Path::new(&path).exists())」，卡线程每领一块问一次（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2124`：「} else if (self.stop_is_requested)() {」），一轮收尾再读一次（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3105`：「let stop = if stop_is_requested() {」），整趟收尾又读一次（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:5591`：「let stop_was_requested = gpu_memory_was_short || stop_is_requested_by_the_environment();」）。文件出现、消失都是用户的动作（文件头写它给「本机先判着、等别的机器准备好再重新分工」用），四处读到的可以不一样。模型「文件被删」一臂（叫停在第 s 个事件出现、在第 s+1 到 s+20 个事件之间消失，其余取法同第一节），`output-models.txt` 原样：

```
test k1_5_stop_file_removed_random_rounds ... ATTACK arm=stop-file-removed scripts=50000 runs=200000 outcomes={"AssertPanicked": 1369, "EveryBlockStored": 116162, "NotRun": 36375, "StoppedOnRequest": 46094} counts={"not_run_with_judged_blocks_not_stored": 3993, "panic_or_loop_or_err": 1369, "same_with_an_out_of_memory_in_the_last_round": 1352, "stopped_on_request_but_run_not_marked_stopped_early": 2799, "stopped_on_request_with_an_out_of_memory_in_the_last_round": 23546} scripts_with_both_stopped_and_not_run=654
```

两样结局：

1. **断言 panic（改前就有这一形）**：卡都照叫停收了工，文件在一轮收尾那一读之前被删，`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:2300`：「!blocks.any_left() || stop == StopOfTheRun::Requested,」不过，「没人叫停，却有 4 块没判」，这一趟判红。1369/200000。改前第 3081 行之后那一句（`research/prompts/m3-batch10z-c-code-r1-main-verification.md:15`：「下一句才写 `!work.blocks.any_left() || stop_is_requested()`」）就是同一个读法，不是这一批带进来的。
2. **照叫停收尾、整趟却没记成提早收尾（一半是这一批带进来的）**：一轮收尾读到叫停、照叫停收尾，第 5591 行再读时文件没了，`stop_was_requested` 是 false，第 ②③ 段照跑、`stopped_early: false`（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:5768`：「stopped_early: false,」），对账里没判的块不为 0，驱动照 `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:321`：「"整份自己跑、或导入了别的机器的库之后，对账为空"」判红。2799/200000，其中最后一轮有卡报 OOM 的 1352 段：改前这一形（叫停、这一轮有卡报 OOM）报本次未跑，`gpu_memory_was_short` 恒真，第 5591 行不看文件；改后走照叫停收尾，第 5591 行全看文件。没有 OOM 的那一半改前就这样。

模型里的事件是离散的，消失落在两读之间的频率远高于真机：真机上第 3105 行到第 5591 行之间只有打一行汇总，删文件要落在这几微秒里才中，第 1 样要落在最后一张卡收工到一轮收尾那一读之间（同样是微秒级）。真机频率推的，没量。

四句：
- 分不分辨臂：第 2 样里有 OOM 的那一半分辨改前、改后两条臂（同一段历史改前记成提早收尾、改后判红）；第 1 样与没有 OOM 的那一半两臂一起中，病根在共用的「叫停每次现读」，另立一笔，不拿它判第 2 处改动。
- 系统看不看得到：看得到。卡线程那一刻读到了叫停，外层循环可以把「这一趟见过叫停」记下来往后带；判据要分的两种情形（叫停过、没叫停过）在做决定的那一刻有可观测的差别。
- 满足判据字面的哪一分句：K2-2「判红」（正文第三节 K2 第 2 条「有没有一形卡放得下却报本次未跑、判红、或整趟弃卡」）。不是「本次未跑」也不是「弃卡」。
- 跑前条款给的改法：正文没给改法。我提的改法见第八节，量过 / 推的照那张表。

推翻条件：叫停文件的语义被定成「只出现不消失、要删得等这一趟退出之后」（那时这两样都属误用，不算打中）；或入库装置上第 5591 行改成不再读文件，同一段历史不判红。

### K2-2-2 其余

- 主机内存不够说成显存不够、弃卡：见第三节，改前就有。
- 叫停、这一轮有卡报 OOM、判完的块在等交回的块，而报过的卡之外没有卡：本次未跑，判完在等的那几块不入库、下一趟重判（模型 `not_run_with_judged_blocks_not_stored` 3992 段）。这是少判不是判错，没有卡能判那一块，不是「放得下却」。
- 叫停之后只判在等的那一轮，`pending_states` 按全部剩下的块算，第一张卡照装满算，可能多上一张卡、多编一次内核（推的）；不是本次未跑或判红，不算。

## 七、K3 改回去照绿

### K3-1 harness 档九处改回去（量过，副本）

`mutate_10ze.py` 在冻结副本的一份拷贝上逐条定点改 `gpu_memory_ledger.rs`（原文恰好命中一次），跑 `cargo test -p singlefs-harness --lib -- gpu_memory_ledger`（83 条）与本模型叫停只出现一臂（`ATTACK_10ZE_SCRIPTS=5000`，断言第一节那四条为 0），再还原。改之前 lib 那 83 条全绿：`test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out`。`output-mutants.txt` 的 MUTANT 行原样：

```
MUTANT H1-largest-is-the-last-judged lib_rc=0 model_rc=101
MUTANT H2-next-card-ignores-the-stop lib_rc=0 model_rc=0
MUTANT H3-stop-before-any-judged-takes-the-head lib_rc=101 model_rc=0
MUTANT H4-less-or-equal lib_rc=0 model_rc=0
MUTANT H5-short-cards-counted-as-left lib_rc=101 model_rc=0
MUTANT H6-record-on-taking-not-on-judging lib_rc=0 model_rc=0
MUTANT H11-device-never-lost-at-return lib_rc=101 model_rc=0
MUTANT H12-device-read-after-waiting lib_rc=101 model_rc=0
MUTANT H19-stop-with-one-waiting-block-stops lib_rc=0 model_rc=101
```

| 改法 | 改的是哪一句 | lib | 模型 | 判 |
|---|---|---|---|---|
| H1 最大块号改成最后判完的那一块 | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1959` | 绿 | 红 | **照绿，是真毛病**：叫停只出现时 11/20000 段轮末断言 panic（`output-mutant-details.txt`：「没有卡报 Out of Memory：判完的块每一块都写进了库」`left: 1`），K1-5 那一形回来了 |
| H19 在等 1 块也照叫停收尾 | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:2178` | 绿 | 红 | **照绿，是真毛病**：71/20000 段照叫停收尾时有判完的块没入库（`stopped_with_judged_blocks_not_stored`） |
| H2 起下一张卡不看叫停 | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1950` | 绿 | 绿 | 照绿，近等价：叫停之后照起一张领不到块的卡，多编一次内核、多借一笔草稿区，结局不变（推的） |
| H4 `<` 改 `<=` | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1906` | 绿 | 绿 | 等价：判完的块不在队里，`first == largest` 不会出现 |
| H6 领到就记（不等判完） | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:2052` | 绿 | 绿 | 近等价：只多判已交回、本来没人等的块，结局不变（推的） |
| H3 什么都没判完时叫停也领队头 | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-harness/src/gpu_memory_ledger.rs:1906` | 红 | 绿 | lib 拦住 |
| H5 报过的卡也算还有的卡 | 第 2313 行 filter | 红 | 绿 | lib 拦住 |
| H11 交回那一刻恒记没丢 | 第 1701 行 | 红 | 绿 | lib 拦住 |
| H12 等齐之后再读丢没丢（(b) 改回去） | 第 1728 行 | 红 | 绿 | lib 拦住 |

实现员没证红的七行 harness 档变异（`crates/mutations.tsv` 第 3329、3333、3335、3340、3341、3347、3348 行）在拷贝上逐行照那一行的参数跑，点名的测试都红（`output-rows.txt` 七行 `named_test_red=True`）。

H1、H19 两处的四句：
- 分不分辨臂：分辨。同一份 lib 用例在原文与改回去之间不分（都绿），模型在同一批取法上分（原文 0、改回去 11 段 / 71 段），差的就是这两句。
- 系统看不看得到：看得到。两处都在不碰卡的 harness 档函数里，单测用固定的几步次序就能造出（推的，没写成 lib 用例；模型里的例子是两张以上卡、一轮里先后两块交回再叫停），不靠真线程的先后。
- 满足判据字面的哪一分句：K3「有没有一形『改回去』照绿」（正文第三节 K3）。
- 改法：正文没给；我提的 F1 修这两格，见第八节。

### K3-2 checker 档接线（推的，checker 档不跑）

| 改回去 | 哪一句 | 有没有拦得住的用例 |
|---|---|---|
| C1 交回那一刻的记法改成恒记「丢了」（(b) 在接线处改回去） | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2200` | 没有：harness 档用例直接给参数，不经这一行；checker 档 `grep -rn device_when_the_work_returns` 只有这一行与 `gpu_unit_checks.rs` 一个用例里的调用，没有一条用例在卡上造「真 panic 之后别的段丢设备」 |
| C2 `cards_used` 改回 `outcome.cards.len()` | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:3163`：「outcome.cards_used = rounds.cards_used();」 | 没有：checker 档测试里 `cards_used` 只出现在打印行里（`crash_amplification_fixed_script_through_rollback.rs` 第 377 行一类），没有断言；而且要卡上真 OOM 才有第二轮 |
| C3 起下一张卡改回只看 `any_left()` | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2467`：「if work.blocks.any_left_to_take(work.stop_of_the_cards()) {」 | 没有；与 H2 同，近等价 |
| C4 `stop_of_the_cards` 两个分支调换次序 | `/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/crash_amplification.rs:2124` | 没有；判红之后再领在等的几块，写库已失败、不写，结局不变，近等价 |

### K3-3 丢失缘由那条用例拦不住 wgpu 改那一句

`/home/fy5090/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2061`：「let out_of_memory = wgpu::hal::DeviceError::OutOfMemory.to_string();」：用例的期望值与产品代码取自同一个 `to_string()`，钉不住「wgpu-core 丢设备时给的就是这一句」（第三节第 706 行那一句）。升 wgpu 时 wgpu-core 改了给法，用例照绿、产品回落判红。wgpu 钉死版本，所以只在升版那一刻要紧（推的）。

## 八、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| F1 把模型叫停只出现那一臂（固定种子、几千份取法）收成 harness 档 lib 用例，或补两条定形用例：两张卡先后判完一高一低两块、其间交回一块，叫停后断言交回的块被领（拦 H1）；叫停、在等 1 块、有别的卡时断言再起一轮而不是照叫停收尾（拦 H19） | K3-1 的 H1、H19 | 量过：模型那一臂在 H1、H19 下红（`output-mutants.txt`）；收成 lib 用例之后照样红是推的，没在副本上写成 lib 用例跑 |
| F2 叫停取一次就记住：外层循环与卡线程共用一个「这一趟见过叫停」的标记，一轮收尾、整趟收尾读标记，不再各自现读文件 | K2-2-1 两样都修 | 推的：等价于模型「叫停只出现」那一臂，那一臂两样都是 0（量过）；标记怎么接进 `run_from_environment` 没写、没跑 |
| F3 只修第 2 处带进来的那一半：叫停、这一轮有卡报 OOM 而判完的都入库时，照叫停收尾但把 `gpu_memory_was_short` 置真（或另带一个「这一趟叫停过」交回去） | K2-2-1 第 2 样里有 OOM 的 1352 段；第 1 样与没有 OOM 的那一半不起作用 | 推的 |
| F4 把「`catch_unwind` 之后立刻记交回那一刻设备丢没丢」抽成 harness 档一个接受闭包的函数，checker 档只调它，harness 档用例造「闭包 panic 之后、等齐之前别的段记丢失」 | K3-2 的 C1 | 推的 |
| F5 用例里不从 `to_string()` 取期望值，写死「Out of memory」，另加一条扫 wgpu-core 源码里 `self.lose(&error.to_string());` 的不要卡用例 | K3-3 | 推的 |

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 叫停落在任一轮任一步（只出现）：轮末断言 panic、判两遍、漏判、照叫停收尾丢判完的块、全入库块数不对、无限再起 | 50000 份取法 × 4 个调度种子 = 200000 段；2–3 张卡、3–7 块、建判器 OOM 两轮各 10%、判块 OOM 第 1–3 块各约 17%、四分之一的卡第 1 轮起装不下、叫停第 1–40 个事件或不叫停 | 全 0（量过，模型） |
| 「在等的」把不在等的当在等或反过来 | 同上，另有第一节的推的证明 | 没有 |
| 第 2 处改动后库里有洞、没判的没留给下一趟 | 同上 | 没有（量过） |
| 比对原文：wgpu 别的丢设备的路给同一句而缘由与内存无关 | wgpu-core 30.0.1 里 `handle_hal_error` 的三种 hal 错误、`lose_if_oom`、销毁那一路（给空串、`Destroyed`）；hal 的 vulkan 错误映射 | 没有；同文的都是内存（主机或显存），见第三节 |
| 第二段窗口被展开里等卡的 Drop 拉长 | 读 `judge_states` 一类活里丢的局部量 | 没找到等卡的 Drop（推的） |
| 账多给（K2-1） | 读这一批 harness 库全部 hunk | 没有 |
| 只判在等的那一轮因 `pending_states` 多算而借不到、报本次未跑 | 读借草稿区那一路 | 每张卡装的受此刻借得到的封顶，借不到的那一张不装，不会因多算而一张都借不到（推的） |

## 这条腿自己的限度

- 模型是 checker 档那一圈照次序抄的，抄得对不对是推的；卡、写库线程都是假的，写库在调度器两步之间做（真机上写库那条线程可能落后），交卡的消息顺序与真机同一条通道。
- 事件计数下的「文件被删」一臂把真机上微秒级的缝放成了一个事件，第六节的频率只说明形状可达，不是真机频率。
- 两段窗口的宽度是在 harness 档测试进程里量的（`nice -n 19`、此刻本机负载下），不是 checker 档二进制、不是卡上的活；第一段只量了线程被调走多久，没有造出真的丢设备。
- 改回去的探针只做了 harness 档九处；checker 档四处是读码推的。H2、H6 判「近等价」是推的，没另写会红的用例去证。
- 「没打中」各格只抽了这一次样。

## 没做什么

- checker 档测试、重型测试一条没跑；没起 GPU 设备、没占卡（本机卡在跑 E174），nvidia-smi 也没调。
- 没判正推那几格，没替主 agent 采纳；副本与模型上的数不是入库装置上的数。
- F1–F5 都没在副本上实现、跑过（F1 的模型部分除外）。
- 编译与跑的过程中有一次差错，已重做：两份冻结副本拷贝共用一个编译目录，cargo 照相对路径认成同一个包，第二份拷贝的头两次复跑用上了上一份拷贝里最后编的变异版（一次是第 3348 行那条变异、它碰不到模型走的路，计数没变；一次是 H19、红了）；改用另一个编译目录从头编、重跑，计数与变异之前那一次逐字相同（`diff` 只差窗口那一个测试名行），`output-models.txt` 取的是重跑那一份。变异探针每条都是改了源文件之后编的，不受影响。
- 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1-opus/` 里交回前删了两份仓副本（`tree/`、`tree2/`）与两个编译目录（`target/`、`target2/`），留着日志、草稿段与 `show_mutant.py`（模型目录里那份 `show_mutant_10ze.py` 的原件）。
