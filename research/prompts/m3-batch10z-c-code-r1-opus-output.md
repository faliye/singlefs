# 第 10z 批第三块代码轮第一轮：攻方报告（m3-batch10z-c-code-r1，2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 S4 S6 S9 P2 -->

被判对象：冻结副本 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/`。下文引代码写 `crates/…` 相对路径，行号取冻结副本；这三份文件与主工作区此刻逐字节相同（`cmp` 过：`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`）。
所有量出的数都是我的模型或副本上的数，不是入库装置上的数；checker 档那一圈（几轮、卡线程）不在 harness 档，我的模型是照原文次序抄的一份，抄得对不对是推的。

## 复跑

模型目录 `research/prompts/m3-batch10z-c-code-r1-opus-model/`。仓根执行：

```
rsync -a --exclude target --exclude .git ~/.local/share/singlefs/work/m3-batch10z-c-code-r1/tree/ <拷贝>/
bash scripts/run-with-memory-cap.sh 4G bash scripts/capped.sh 6 \
  bash research/prompts/m3-batch10z-c-code-r1-opus-model/run.sh <拷贝> <编译目录>
bash scripts/run-with-memory-cap.sh 4G bash scripts/capped.sh 6 \
  python3 research/prompts/m3-batch10z-c-code-r1-opus-model/mutate_10zc.py <拷贝> <编译目录> <变异名…>      # 不给名字列出 15 个
bash scripts/run-with-memory-cap.sh 4G bash scripts/capped.sh 6 \
  python3 research/prompts/m3-batch10z-c-code-r1-opus-model/mutate_10zc.py <拷贝> <编译目录> --with-models endings-notify-one record-the-caught-after-waiting
```

内存上限：派发提示给 12G；收尾那一次 12G 在内存包装的准入里排队（slice 已占 33.6 GiB、总上限 40.1 GiB，`排队（已等 180 秒…）` 之后被我外面的 `timeout 235` 收掉，那一次的输出不算），之后改用 4G（更紧，测试本身用不到 1 GiB），复跑命令照 4G 写。

| 文件 | sha256 |
|---|---|
| `attack_10zc_endings.rs` | `d746dcfdd7e4b68dcdef6cb1e2f1c37d13de15c7a51efe66b632647c73dd7f1d` |
| `attack_10zc_rounds.rs` | `ff0cc1b18534dc21cc6c0511c7efa37ace5f20134a0deaee1ab3c2cfc4b19ffd` |
| `attack_10zc_ledger.rs` | `3512150be52e20668f65b5f2ca8895b0efd2283840560a365978177be0510fd9` |
| `mutate_10zc.py` | `c9019b5f8b6c04ad0d734bed16249e57b0d04166a75f980a2ac73c6eff048b2c` |
| `run.sh` | `e35f655e15dab24181d7de98ba72817047c59e2388e260ec70d662b4c42c3a62` |
| `output-models.txt` | `65961dcc478f76657a2b809ecbe3092a802b9267aa6945df15a973731cbac5a8` |
| `output-mutants-lib.txt` | `0599621747a880d22cbe9035ae7fffe878c5df66cc1908ec426fcbbad1db2b20` |
| `output-mutants-models.txt` | `5edbf8e44f5d7d6b8c9d530154173e6852bebf665f2949639a49bde359948dbf` |
| `output-mutants-models-earlier.txt` | `2d46c483a1c36b412cc442c19533a55438573c07924c95a0e7ec0392e45506cc` |

三份模型只 `use singlefs_harness::gpu_memory_ledger::…` 的公开项，不改被判代码。k1_5 随机一格与 k1_4 同组重跑的计数靠真线程的先后，两次跑会差；定形那几格每次相同。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| K1-4 挂住 | 没打中 | 1–32 段一组、随机 panic、组里套单段组、起线程中途 panic，280 组都收齐 | 量过（模型） |
| K1-4 真 panic 说成显存不够 | **打中（窄）** | A 的活自己 panic、那一刻设备没丢；同块 B 之后接到致命的 Out of Memory、设备丢了。等齐再认把 A 说成显存不够（`told_after_waiting=true`），A 收完那一刻认是 `false`；那一刻 `is_lost()` 是 `false`，系统看得到 | 量过（模型）；wgpu 先后推的 |
| K1-4 真的显存不够判红 | **打中** | 同一设备上不在同一组的两段活（同一测试进程里共用卡池设备的两条用例）：X 范围里接到、设备因它丢了，Y 先因丢设备失败先认，`y_told_out_of_memory=false`（判红）；同组对照 `true`。改法只罩同块几段，碰不到这一格；wgpu 的丢失原因里本来就写着 `Out of memory`，Y 认的那一刻看得到 | 量过（模型）；可达性推的 |
| K1-5 判两遍 / 漏判 / 无限再起 / 轮数超过卡数 | 没打中 | 6000 段随机：`judged_twice=0 every_stored_but_missing=0 more_rounds_than_cards=0` | 量过（照原文抄的模型） |
| K1-5 判完不入库 + K2-2 判红 | **打中** | 收尾再起的那一轮里叫停文件出现：上一轮判完、等着交回那一块的块还在 `BlocksStoredInOrder` 里，第 3081–3085 行 `assert_eq!(blocks_waiting(), 0)` 判不过，原文 panic，这一趟判红，4 块判完不入库。叫停文件任意一刻都可能出现（第 1383 行） | 定形格量过（模型）；checker 档原文那几行推的 |
| K1-5 / K2-2 账面 | 打中（小） | 一张卡在两轮里各收工一次，`outcome.cards` 记两条，`cards_used` 比用到的卡多 | 推的（读码） |
| K2-1 甲之下多给 | 没打中 | 带丢设备、重起、死借据的随机 8000 段：账放行而真放不下的借 0 次、重起 0 次；只在「设备丢了、账还没被告知」那一段读数上账给得比真能借的多（最多 2319），而任何建缓冲的路都先经 `prepare` 重判 | 量过（模型）；「都经 prepare」推的 |
| K2-2 其余（甲之外的少给、弃卡） | 没另打中 | 重起被拒而放得下的 960 / 928 次都是甲认的 S4、S9 那一形；只一张卡报过就本次未跑是实现员钉的设计 | 量过（模型） |
| K3 改回去照绿 | **打中** | 15 处改坏里 harness 档 `gpu_memory_ledger::tests` 照绿 7 处；其中两处是真毛病：`notify_all` 改 `notify_one`（三段以上一块挂住，我的挂住探针 60 秒没收齐），「范围里接到的」挪到等完之后再记（把第 10z 批第一块打中的那一形放回来，同组 2000 次里只有 294 / 1035 次认得出，改前 2000 次） | 量过（副本） |

## 一、K1-4：同块几段都报到再认

被判的走法：一块切成 `checking_threads()` 段，报到在起线程之前都领出来，`crates/singlefs-checker-tier/src/crash_amplification.rs:1500`「pieces.iter().map(|_| ending_together.join()).collect();」；每段收完先把范围里接到的记进设备、报到、等齐、再认，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1678`「self.record_an_out_of_memory(caught);」、`crates/singlefs-harness/src/gpu_memory_ledger.rs:1680`「ending.end_and_wait_for_the_others();」。认的规则没改：这段活自己的 Out of Memory，或「设备丢了、这之后记过一次」，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1706`「(Some(lost), Some(report)) => {」；「这之后」从这段活开始算，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1697`「let reports_since = reports.get(reports_before..).unwrap_or(&[]);」。卡线程的单段活与第 ② 段核对各自一个人一组，`crates/singlefs-checker-tier/src/crash_amplification.rs:2167`「let alone = WorksEndingTogetherOnADevice::new();」。

模型：`attack_10zc_endings.rs`，照 `on_the_card_catching_out_of_memory_ending_together`（`crash_amplification.rs` 第 2176 行起）的次序调 harness 档原样的 `DeviceConditionOfACard`、`WorksEndingTogetherOnADevice`、`WorkOnTheCardOfThisThread`；wgpu 那一半（哪一刻丢设备、哪一刻范围交回）由各格的次序给定。

### 甲：不在同一组的两段活（打中：真的显存不够判红）

历史（每一步指到许可它的那一句）：
1. 同一测试进程里两条 GPU 用例借到同一张卡，拿的是卡池留着的同一个设备（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1418`「起的时候拿着这张卡的锁：同一张卡上同时要的等这一次起完（别的卡不等）。」，留着的没丢就交它的克隆）。`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 一份测试二进制里取卡池 54 处（`grep -c gpu_card_pool_or_report_not_run`），cargo 默认几条线程一起跑用例。两段活各经 `on_the_card_catching_out_of_memory`，各自一个人一组（第 2167 行）。
2. X 的活里一次提交撞上 Out of Memory：wgpu-core 把它当致命，`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-core-30.0.1/src/device/resource.rs:706`「self.lose(&error.to_string());」，先置设备无效、再调丢失回调（同文件第 5420、5424 行），回调记进设备，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:128`「condition.record_the_device_lost(format!("{reason:?}：{message}"));」。那一句 Out of Memory 留在 X 这条线程的错误范围里，要等 X 的活收完、范围交回才记进设备。
3. Y 的下一次调用撞上无效的设备而失败，先收完：范围里没接到，等的只是它自己，`reports_since` 里一条都没有，第 1706 行那一支不中，认成 `None`；活 panic 了，`crates/singlefs-checker-tier/src/crash_amplification.rs:2209`「WorkOnTheCard::Panicked(panic) => std::panic::resume_unwind(panic),」，判红。
4. 之后 X 收完，自己那一句认得出，报本次未跑。

模型原样（`output-models.txt`）：

```
ATTACK k1_4=cross-group same_group=false x_told_out_of_memory=true y_told_out_of_memory=false y_reason=None
ATTACK k1_4=cross-group same_group=true x_told_out_of_memory=true y_told_out_of_memory=true y_reason=Some("设备丢了（Unknown：提交时显存不够），丢之前报过 序号 0 的卡判一块时报了 Out of Memory")
```

四句：
- 分不分辨臂：分辨「等同块几段」与「等这个设备上此刻活着的全部活」两种分组；对第 10z 批第一块判决给的另一个改法「范围里接到的当场记进设备」不分辨：范围只在活收完时交回，当场也记不进去，两个改法在这一格上都中（共用前提：Out of Memory 只经范围交回才知道）。
- 被判的系统当时看不看得到：看得到。丢失回调是 X 撞上的那一刻同步调的，原因字是 hal 的 `/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-hal-30.0.1/src/lib.rs:382`「#[error("Out of memory")]」，Y 认的那一刻 `lost_reason()` 里已经是 `Unknown：Out of memory`；只剩第 5420 行置无效到第 5424 行调回调之间那一小段（推的，没量）。
- 满足判据哪一分句：K1-4「真的显存不够判红」。
- 改法在这一格上还中不中：这一批的改法（同块报到）中——罩不到组外；正文没给别的跑前改法。我提的（被攻过零轮，只在模型上看过形）：丢失原因认得出是显存不够（hal 的 `Out of memory`）时，设备丢了就说成显存不够，不等别的活交回范围；或把一组扩成「这个设备上此刻在做的全部活」（`WorkOnTheCardOfThisThread::begin` 时登记进设备）。两者都没实现、没跑。

可达性：一个崩溃放量进程里同一个设备上并发的只有同块几段（卡线程一张卡一个设备），这一格要靠一个进程里几条 GPU 用例共用卡池设备才走得到（推的：commit 档那几份测试二进制用不用 `--test-threads 1` 我没查）。

### 乙：同组里一段真 panic、之后另一段致命的 Out of Memory（打中，窄：真 panic 说成显存不够）

历史：一块两段 A、B，同一组。A 的活自己 panic 了（真毛病，与显存无关），那一刻设备没丢、设备上一条 Out of Memory 都没有；A 收完、报到、等 B。之后 B 的范围里接到一次致命的 Out of Memory、设备丢了，B 收完把它记进设备。A 等齐再认：从 A 开始起「设备丢了、记过一次」，第 1706 行那一支中，`crates/singlefs-checker-tier/src/crash_amplification.rs:2151`「(Err(_), None, Some(detail)) => WorkOnTheCard::OutOfMemory { detail },」，A 的真 panic 报成显存不够，这一块不入库、这一趟本次未跑。

模型原样：

```
ATTACK k1_4=genuine-panic-widened told_after_waiting=true told_at_the_failure=false lost_when_a_failed=false reason=Some("设备丢了（Unknown：提交时显存不够），丢之前报过 序号 0 的卡单元校验和对拍时报了 Out of Memory")
```

`told_at_the_failure` 是 A 收完那一刻就认（改前的次序）：`false`，照判红。`lost_when_a_failed=false` 是 A 的活收完那一刻 `is_lost()` 的值。

四句：
- 分不分辨臂：分辨「先等再认」与改前「先认」：只有等齐的那一臂把它说成显存不够。第 10z 批第一块打中的那一形（A 因设备丢了先失败）A 失败那一刻设备已经丢了，所以下面的改法两形分得开。
- 系统当时看不看得到：看得到，A 的活交回那一刻 `is_lost()` 读得到（模型里是 `false`）。
- 分句：K1-4「真 panic 被说成显存不够」。代价是那一趟真毛病不判红、报本次未跑；下一趟不撞 Out of Memory 时照样判红（推的）。
- 改法（零轮，没实现）：活交回那一刻记下设备丢没丢；等齐之后「设备丢了」那一支只给失败那一刻设备已经丢了的活用，这一刻还没丢的只认算在它自己头上的 Out of Memory。

### 丙：挂不挂住（没打中）

试过的形（`k1_4_groups_of_pieces_never_wait_forever`）：每组 1、2、3、4、8、16、32 段各 40 组，每段随机三种收法之一（照常收、范围里有没有随机；活 panic 报到随 unwind 丢掉；活里先套一段单独一组的活再收），收之前随机让出 0–49 次；每 4 组里 1 组在起到第 k 段时让起线程那一步 panic（剩下的报到随迭代器丢掉，同第 1503 行 `zip(endings)` 的走法）。原样：

```
ATTACK k1_4=hang-probe outcome=Ok((280, 210, 70)) (Err 就是 60 秒内没收齐：挂住)
```

280 组全收齐（210 组正常、70 组带起线程失败）。没试的：一段的活本身卡在卡上不返回（改前同样挂，那时收的一方也要等全部段）。

## 二、K1-5：收尾再起一轮

被判的走法（`crash_amplification.rs` 第 2919–3148 行）：几轮共用一份按块序写库的 `crates/singlefs-checker-tier/src/crash_amplification.rs:2919`「let mut stored_in_order: BlocksStoredInOrder<GpuJudgedBlock> = BlocksStoredInOrder::new();」；一轮收完，下一轮只判这一轮没领走的，`crates/singlefs-checker-tier/src/crash_amplification.rs:3105`「blocks_of_the_round = work.blocks.blocks_not_taken();」；报过的卡不再用（第 2937、2943、3108 行同一句过滤）。每轮收完、这一轮没有卡报 Out of Memory 时有两条 assert（第 3076–3091 行）：

- `crates/singlefs-checker-tier/src/crash_amplification.rs:3084`「"领走的块每一块都写进了库"」（`assert_eq!(stored_in_order.blocks_waiting(), 0, …)`，不看叫停）；
- `crates/singlefs-checker-tier/src/crash_amplification.rs:3087`「!work.blocks.any_left() || stop_is_requested(),」（看叫停）。

模型：`attack_10zc_rounds.rs` 把几轮（第 2919–3148 行）、卡线程（第 2221–2391 行）、起下一张（第 2444–2460 行）、写库那一头（第 2050–2093 行）照原文次序抄成不碰卡的一份，用 harness 档原样的 `judge_blocks_on_one_card`、`BlocksForTheCards`、`BlocksStoredInOrder`、`after_a_round_of_the_cards`；两条 assert 照原文判，判不过记成 `AssertPanicked`。抄得对不对是推的（checker 档测试不跑）。

### 甲：收尾那一轮里叫停（打中：判完不入库，这一趟判红）

历史：
1. 两张卡、5 块，第 0 张装 1 个状态、第 1 张装 4 个（两张都装了，头一轮都起）。第 0 张领到块 0，判到末尾报 Out of Memory、交回块 0、收工；在它报之前第 1 张已经判完块 1–4、领不到块收工（交回之前队里只剩第 0 张手里那一块）。块 1–4 判完交到写库那一头，等着块 0（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1960`「while let Some(next_block) = self.waiting.remove(&self.blocks_stored) {」）。
2. 头一轮收完：有卡报了、没叫停、还有没报过的卡，再起一轮（第 3110 行读叫停是这一刻，`crates/singlefs-checker-tier/src/crash_amplification.rs:3110`「let stop = if stop_is_requested() {」）。
3. 用户（或分工脚本）在这之后建了叫停文件：`crates/singlefs-checker-tier/src/crash_amplification.rs:1383`「std::env::var_os("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE")」，文件在就要停，什么时候建由用户定。
4. 收尾那一轮第 1 张卡借草稿区、建判器之后，领块之前问叫停（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1907`「while !must_stop() {」），不领、收工。这一轮没有卡报 Out of Memory，第 3081 行那条 `assert_eq!` 读到等着的 4 块，panic。

模型原样（`output-models.txt`；事件按「建判器、判一块各算一个」从 1 数，头一轮 7 个）：

```
ATTACK k1_5=stop-in-the-closing-round cell=no-stop ended=EveryBlockStored rounds=2 stored=5 judged_once_or_more=5 judged_not_stored=0 stop_requested=false
ATTACK k1_5=stop-in-the-closing-round cell=stop-at-round-1-last-block ended=NotRun rounds=1 stored=0 judged_once_or_more=4 judged_not_stored=4 stop_requested=true
ATTACK k1_5=stop-in-the-closing-round cell=stop-at-round-2-build ended=AssertPanicked("领走的块每一块都写进了库（等着的 4 块）") rounds=2 stored=0 judged_once_or_more=4 judged_not_stored=4 stop_requested=true
ATTACK k1_5=stop-in-the-closing-round cell=stop-at-round-2-first-block ended=EveryBlockStored rounds=2 stored=5 judged_once_or_more=5 judged_not_stored=0 stop_requested=true
```

把叫停那一刻放开扫（定形格四个点；随机格里叫停在第 1–20 个事件中随机、或不叫停，6000 段）：叫停落在头一轮收完之前（第 3110 行读到）走本次未跑，落在收尾那一轮领到交回那一块之后照常写完；只有落在两者之间、而上一轮有判完等着的块时判不过。随机格原样：

```
ATTACK k1_5=random-rounds ended=AssertPanicked(领走的块每一块都写进了库) stop_requested=true runs=9
ATTACK k1_5=random-rounds ended=EveryBlockStored stop_requested=false runs=4045
ATTACK k1_5=random-rounds ended=EveryBlockStored stop_requested=true runs=85
ATTACK k1_5=random-rounds ended=NotRun stop_requested=false runs=1408
ATTACK k1_5=random-rounds ended=NotRun stop_requested=true runs=120
ATTACK k1_5=random-rounds ended=StoppedOnRequest stop_requested=true runs=333
ATTACK k1_5=random-rounds runs=6000 judged_twice=0 every_stored_but_missing=0 assert_without_stop=0 stopped_without_stop=0 more_rounds_than_cards=0 judged_not_stored_at_assert=28 judged_not_stored_elsewhere=248
```

9 段判不过、全在叫停之后（`assert_without_stop=0`），这 9 段里判完不入库的合计 28 块。随机格靠真线程的先后，计数每次跑会差（草稿目录里改前同一份模型的另一次跑也是 9 段，不入库）。

四句：
- 分不分辨臂：分辨。第 10z 批第一块判决给的两个改法里，这一格只在「收尾那一步判交回的块」这一臂上有（等着的块跨轮留着）；「写库不等交回的那一块」那一臂没有等着的块，这条 assert 不会读到（推的）。改前（第 10z 批第一块）没有第二轮，头一轮里领走的块连着，这条 assert 成立。
- 系统当时看不看得到：看得到，同一处第 3087 行那条已经在读 `stop_is_requested()`。
- 分句：K1-5「判完不入库」（4 块判完丢掉），与 K2-2「卡放得下却判红」（panic；卡都放得下，只是被叫停）。文档要的是「已经领的块判完、写进库再收尾」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2622`「每领一块之前先问 `stop_is_requested`：要停就不再领，已经领的块判完、写进库再收尾（领的块按块序连着，库里不留空洞）。」）。
- 改法（零轮，没实现、没在原文上跑）：一、第 3081 行那条也让叫停豁免，叫停时等着的块丢掉、照叫停收（这一趟不判红，判完的 4 块下一趟重判）；二、收尾那一轮在叫停之后照判交回的那几块（只判到等着的块都能写为止），守住第 2622 行那一句。二在模型上能改出来，没改。

### 乙：一张卡在两轮里各算一次（打中，小；推的）

`crates/singlefs-checker-tier/src/crash_amplification.rs:3094`「outcome.cards.extend(received.finished_cards);」每轮追加，收工过的卡下一轮再收工一次又追加一条；之后 `crates/singlefs-checker-tier/src/crash_amplification.rs:3154`「outcome.cards_used = outcome.cards.len();」。三张卡、第 0 张报了、第 1、2 张两轮都收工时 `cards_used` 是 4，`CRASH_AMPLIFICATION_GPU_TOTAL cards_used=` 那一行（第 3169 行）报的卡比本机多；`compile_seconds`（第 3155 行）是两轮的和（那是真花的时间）。只读码推的，没跑。属于判据哪一格：不在 K1-5 列的几种里，是账面上的数错，记一笔。

### 丙：判两遍、漏判、无限再起、轮数超过卡数（没打中）

随机格见上（1–4 张卡、1–8 块、每轮每张卡带得了 0/1/2/8 个状态、每轮建判器时 1/6 机会报、每张卡第 1–4 块随机报、判块前让出 0–199 次、报的那一块报之前再让出 0/2000/20000 次、叫停在第 1–20 个事件或不叫停）：`judged_twice=0 every_stored_but_missing=0 stopped_without_stop=0 more_rounds_than_cards=0`。判完不入库在别的结局里合计 248 块（本次未跑、叫停时上一轮判完的块丢掉），是设计认的（实现员报告「上一轮判完、等着前面那一块的那几块在内存里等到下一轮」），不另记。

## 三、K2-1：甲之下账给的会不会大于此刻真能用的（没打中）

攻的路：重起与 S6 新状态。丢了的设备重起时记成新状态，`crates/singlefs-harness/src/gpu_memory_ledger.rs:862`「self.driver_share = DriverShareOnTheCard::DeviceLostNotStartedAgain;」；此后每一次要起都现读再判，`crates/singlefs-harness/src/gpu_memory_ledger.rs:1281`「self.driver_share_of_a_new_device_fits_now(&mut books, card_position)」；新状态在「许出去的」里照样算驱动那一份，`crates/singlefs-harness/src/gpu_memory_ledger.rs:197`「| DriverShareOnTheCard::DeviceLostNotStartedAgain => booked」。卡池每一条借的路都经 `card_lent_for_the_booking`（`gpu_unit_checks.rs` 第 1348、1369、1426 行），它先起设备，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1456`「let mut card = match self.started_card(&booking) {」，留着的丢了就重判，`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1263`「|started: &GpuCard| started.device_condition().is_lost(),」。

模型 `attack_10zc_ledger.rs`：显存账原样接一张假卡（第 10z 批第一块攻方那张卡加丢设备）。丢设备时旧设备的驱动实占、建在旧设备上的缓冲、销了账还没还的一起留在卡上（「旧设备那一堆」），丢之前借的借据成「死借据」（再也建不上、不再长）；旧设备的借据都还了才可能收走那一堆。借照 `lease_card` 的次序：先记账，设备没起着就 `prepare`，放行才起新设备（驱动实占每次随机 200–2560）、记起好了，被拒就把这一笔还掉。别的进程的占用随机涨落。真能再借的 = 空闲 − 活借据（不算死借据）还没建的 − 驱动还要长的。每读一次比账给的与真能再借的；每次账放行（借到、重起放行）时看放行之前真放不放得下。

原样（`output-models.txt`，两颗种子各 4000 段、每段 120 步）：

```
ATTACK k2_1=random-restarts seed=0x9e3779b97f4a7c15 histories=4000 steps_each=120 Sweep { readings: 53514, overgive_at_reading_max: 2275, overgive_readings: 101, new_overcommit_after_a_booking: 0, new_overcommit_after_a_restart: 0, restarts_allowed: 15909, restarts_refused: 960, restarts_refused_but_fit: 960, losses: 13109, first_overgive: Some("room=25244 truth=25166 free=27726 others=2344 driver_now=0 started=false old_pile=2040 booked={0: 1352, 1: 610} built={} dead={0, 1} returned_still=0") }
ATTACK k2_1=random-restarts seed=0x2545f4914f6cdd1d histories=4000 steps_each=120 Sweep { readings: 53549, overgive_at_reading_max: 2319, overgive_readings: 114, new_overcommit_after_a_booking: 0, new_overcommit_after_a_restart: 0, restarts_allowed: 15847, restarts_refused: 928, restarts_refused_but_fit: 928, losses: 13016, first_overgive: Some("room=15906 truth=15232 free=17792 others=12545 driver_now=0 started=false old_pile=1773 booked={0: 1099} built={} dead={0} returned_still=0") }
```

读法：
- 账放行而真放不下：借 0 次、重起 0 次（`new_overcommit_after_a_booking: 0, new_overcommit_after_a_restart: 0`）。
- 读数上账给得比真能借的多：101 / 53514 与 114 / 53549 次，最多 2319 MiB。两颗种子头一次都是同一形：设备刚丢、账还没被告知（`started=false`、旧设备那一堆在卡上、账上还记着「设备起好了」），账把旧设备的驱动实占当成新设备的驱动那一份已经占上。账在这一段放行的借据接着就进 `card_lent_for_the_booking` → `started_card` → `prepare`，在那里照新状态重判（重起放行 0 次放不下），所以没有一笔真按这个读数建上卡。「每条建缓冲的路都先经 prepare」是读码推的（上面三处调用点），不是量的。
- 重起被拒而真放得下：960 / 928 次，全部（`restarts_refused == restarts_refused_but_fit`）：旧设备的驱动那一份记进「销了账、卡上可能还没还的」，甲压不到占着的以下。这是正文写明的「重起之后少给 D、压不掉（正推 S4、S9）」那一形，不算打中。

没打中的取样范围：一张卡、别的进程 0–16000 MiB 涨落、每笔借 1–6000 MiB、驱动实占 200–2560 MiB；没扫的：两张卡、几条线程真并发（模型里一步一步走，读数与借在同一刻）、`AlsoOutsideTheBooks`（不走账的设备在同一张卡上）与丢设备叠在一起。

## 四、K2-2：甲认的少给之外，卡放得下却本次未跑、判红、弃卡

- 判红：第二节甲那一形（收尾那一轮叫停，assert panic）。卡都放得下。
- 本次未跑而放得下：只有一张卡、它报过一次 Out of Memory 就本次未跑（收尾那一轮只用没报过的卡），哪怕那张卡之后又空出来。这是实现员用 `with_one_card_only_a_block_it_gives_back_leaves_the_run_not_run` 钉的设计，报了 Out of Memory 本来就说明账没防住，不算打中。
- 收尾那一轮在报过的卡之外没有一张装得下时走本次未跑（`crates/singlefs-checker-tier/src/crash_amplification.rs:3009`「if let Some(reason) = first_out_of_memory {」），不判红：模型随机格里 `Red` 一次都没有（结局表里没有那一行）。
- 收尾那一轮重借草稿区：头一轮在同一张卡上借过的那一笔还了之后记进「销了账、卡上可能还没还的」，甲压不到这个进程占着的以下，收尾那一轮在这张卡上少借（推的，没量）。这是甲认的代价，不算打中，记一笔：收尾那一轮正好是「刚还了一大笔、马上又要借」的时候，这一形在这里比别处常见。

## 五、K3：改回去照绿

副本（冻结副本的拷贝 `~/.local/share/singlefs/work/m3-batch10z-c-code-r1-opus/copy/`）上逐条改坏 `gpu_memory_ledger.rs` 的一处（锚点恰好命中一次），跑 `cargo test --offline -p singlefs-harness --lib -- gpu_memory_ledger::tests`（改前 68 条全绿：`test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out`），跑完还原。15 处，原样（`grep -oE 'name=[a-z-]+ exit=[0-9]+ result=[a-z]+' output-mutants-lib.txt`）：

```
name=lost-state-reverted-to-registered exit=101 result=caught
name=lost-state-promises-no-driver-share exit=0 result=survived
name=lost-state-not-registered exit=101 result=caught
name=endings-notify-one exit=0 result=survived
name=endings-do-not-wait exit=101 result=caught
name=record-the-caught-after-waiting exit=0 result=survived
name=blocks-taken-largest-first exit=101 result=caught
name=store-after-a-storing-error exit=0 result=survived
name=a-failed-block-counts-as-stored exit=101 result=caught
name=stop-ignored-after-an-out-of-memory exit=101 result=caught
name=card-keeps-taking-after-the-store-is-gone exit=0 result=survived
name=a-refused-block-is-given-back exit=0 result=survived
name=stop-checked-after-taking-a-block exit=0 result=survived
name=an-out-of-memory-card-says-nothing-left exit=101 result=caught
name=blocks-left-counted-from-not-taken exit=101 result=caught
```

照绿的 7 处，逐处看是不是真毛病（「量过」是副本或模型上贴了原样输出，「推的」是读码）：

| 变异 | 改了哪一处 | 是不是真毛病 | 依据 |
|---|---|---|---|
| `endings-notify-one` | 第 1770 行 `notify_all` → `notify_one` | **是**：一块三段以上时，最后报到的只叫醒一段，其余的一直等，这一块挂住；一块的段数是 `checking_threads()`（没设变量取本机并行度，本机 32） | 量过：我的挂住探针 `ATTACK k1_4=hang-probe outcome=Err(Timeout) (Err 就是 60 秒内没收齐：挂住)`、`model_exit=101`；改前 `outcome=Ok((280, 210, 70))`。库单测只有两段一组，叫醒一段就够 |
| `record-the-caught-after-waiting` | 第 1678 行那一句挪到第 1680 行等完之后 | **是**：放回第 10z 批第一块打中的那一形（先失败的那一段等完马上认，另一段还没把范围里接到的记进设备） | 量过：同组 2000 次 `y_told_out_of_memory=294`（`output-mutants-models-earlier.txt` 里 mut-b4 那一段是 1035），改前 `runs=2000 y_told_out_of_memory=2000`；库单测那一条要靠先后才红，这两次都照绿 |
| `stop-checked-after-taking-a-block` | 第 1907 行起先领一块、再问叫停 | **是**：领了不判也不交回，这一块没人判、它后面判完的等着，叫停那一趟照第二节甲在第 3081 行 panic | harness 档照绿是量过；checker 档 `the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest`（有卡）会不会红是推的。库单测的假卡 `must_stop` 恒为假 |
| `card-keeps-taking-after-the-store-is-gone` | 第 1914 行交不到写库那一头后接着领 | 小：写库那条线程走了（它 panic 了）这一趟本来就判红，多判几块是白判 | 推的 |
| `a-refused-block-is-given-back` | 拒判的那一块也交回 | 不影响结局：拒判整趟判红 | 推的 |
| `store-after-a-storing-error` | 第 1961 行那一支恒假 | 等价：写失败的那一块已经从 `waiting` 里拿走，`blocks_stored` 不再往前，之后的块永远取不到，那一支走不到 | 推的（读第 1960–1967 行） |
| `lost-state-promises-no-driver-share` | 新状态在「许出去的」里不算驱动那一份 | 在我扫的范围里看不出：K2-1 模型两颗种子的数与改前逐字相同（`output-mutants-models-earlier.txt` 里 mut-b3 那一段）；旧设备那一堆比驱动那一份大时才会不同，模型里驱动实占不超过 D | 推的 |

正文第三节写明不判的：第 D 条在 checker 档的接线、第 E 条按轮跑的循环本身没有不要卡的用例。第二节甲的 assert 在那个循环里，我没把「循环没用例」当打中，打中的是那一处的行为。

## 没打中的形状

| 攻击面 | 试过的形 | 取样范围 |
|---|---|---|
| K1-4 挂住 | 一段 panic 丢掉报到、组里套单段组、起线程中途 panic、几段同时等 | 1–32 段一组共 280 组（第一节丙） |
| K1-4 一段永远不报到 | 只能靠活本身卡在卡上不返回；改前收的一方同样要等全部段，不是这一批带进来的 | 没造（要卡） |
| K1-5 判两遍、漏判、无限再起、轮数超过卡数、没叫停却当叫停收 | 随机 6000 段，见第二节丙 | 1–4 张卡、1–8 块、5 轮内每轮每卡容量与建判器报不报随机、叫停时刻随机 |
| K2-1 多给 | 丢设备、重起被拒再来、死借据、旧设备那一堆、新设备驱动实占随机 | 一张卡，两颗种子各 4000 段 × 120 步 |
| K2-2 弃卡 | 收尾那一轮报过的卡之外没卡、没卡装得下 | 随机格里走本次未跑，没有判红 |

## 这条腿自己的限度

- checker 档的几轮与卡线程是我照原文次序抄的模型，不是原文在跑；抄错一步，第二节的数就不作数。最该核的一步：收尾那一轮第 1 张卡建好判器之后、领块之前那一次 `must_stop()`（原文第 2221–2391 行经 harness 档 `judge_blocks_on_one_card`，第 1907 行）。推翻条件：在原文上（有卡、checker 档、第 ① 段起过卡池）照第二节甲的次序造出来，第 3081 行不 panic。
- 第一节甲的可达性靠「一个进程里几条 GPU 用例共用卡池设备并发」；commit 档那几份测试二进制是不是 `--test-threads 1` 跑的我没查。推翻条件：那几份都单线程跑、产品路径里同一设备上并发的只有同块几段。
- 第一节甲、乙里 wgpu 的先后（丢失回调与范围交回谁先谁后、丢失原因里有没有 `Out of memory`）是读 wgpu-core 30.0.1 与 wgpu-hal 30.0.1 源码推的，没在卡上造过 Out of Memory。
- 第五节的变异只改 `gpu_memory_ledger.rs`；checker 档的三份 src 与对拍用例没改坏、没跑（checker 档测试不跑）。
- 自己提的改法（第一节甲乙、第二节甲各两条）都只在读码上推过，没在模型上实现，被攻过零轮。
- 这是同一份提示的一次抽样；「没打中」那几格照规则一次不算数。

## 没做什么

- 没跑 checker 档任何测试、没起 GPU 设备、没跑 nvidia-smi 以外读卡的东西（钩子自动跑的资源提示读了 nvidia-smi）。
- 没判 K1-1、K1-2、K1-3 的对拍用例那几格（不在分给我的攻击面里）；没判正推那几格；没替主 agent 采纳。
- 副本上的数不进 kb；复跑要在入库装置上重做。
- cargo 调用（都经内存包装）：模型目标 9 次（其中 1 次在准入里排队、被我外面的 `timeout 235` 收掉）、库单测基线 1 次、变异探针里库单测 36 次与模型目标 7 次；开跑前 `ps` 看到别的会话的 `cargo test`（`singlefs-checker-tier`、`singlefs-harness --test k2_arms_probe` 等，没有 qemu、vm-bench、e152、fio），我都 `nice -n 19` 照跑；收尾那一次 12G 在准入里排队 180 秒以上，改 4G 跑（见开头）。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-c-code-r1-opus/`：`copy/`（冻结副本的拷贝，模型测试拷在它的 `crates/singlefs-harness/tests/` 里）、`target/`（编译目录）、`*.log`（各次跑的原文）。交回前删 `copy/` 与 `target/`。
