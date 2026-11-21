# 正推腿报告：崩溃放量任务里不领卡、读库总账按态与测试进程线程份额（代码轮第一轮）

日期：2026-11-18。读的是主工作区此刻的树（HEAD 4319aa2b035b 加未提交改动）；行号都是这一轮里 `grep -n` 现取的，不是从背景材料里数的。
没写模型（五个问题都是读代码对条款，不需要建模），没起任何命令；要主 agent 跑的见末尾「要主 agent 跑的命令」。
背景材料正文里的句子我只当线索，下面每一条都回到代码里核过。

## 问题 1：任务进程里还有没有开卡、借显存、照多机配置自己挑卡的路

结论：**不成立**。四处里三处（崩溃放量一趟的两段判与核对、只排派活计划那一趟）没有取卡的路；「观察者交回的盘」那一处有；另外三条不在这四处里的路在任务进程里同样取卡。

条款原句：`.claude/rules/verification.md:50` 「交的任务只跑录入、往判定库写待判行、读结论，不领卡、不分卡、不清场」。

取卡、起卡池、借显存的函数在 `src/` 与 `tests/` 里的全部调用点（`grep -rnE` 现跑，去掉注释行，输出整行）：

```
src/crash_state_enumeration.rs:2704:                crate::crash_amplification::gpu_cards_of_this_run_from_the_multi_host_configuration,
src/crash_amplification.rs:1652:                gpu_cards_of_this_run_from_the_multi_host_configuration,
src/crash_amplification.rs:5100:                .book_the_scratch_of_the_cards_with_room_now(
src/crash_amplification.rs:14592:            match gpu_cards_of_this_run_from_the_multi_host_configuration() {
src/crash_amplification.rs:14621:                    gpu_cards_of_this_run_from_the_multi_host_configuration,
src/crash_amplification.rs:25467:                    gpu_cards_in_use(cards_by_priority.as_ref())?;
src/crash_amplification.rs:25966:                match pool.lease_the_first_card_with_room_waiting_for_the_other_threads(
src/crash_amplification.rs:32202:                .lease_the_first_card_with_room_now(
src/crash_amplification.rs:33259:    gpu_cards_in_use(cards_by_priority.as_ref())
```

（另有定义行与字符串行，不是调用。14592、14621 在单测里。）下面逐处对这几个调用点。

### 1a 崩溃放量一趟：成立（没有取卡的路，观察者那一支除外，见 1d）
入口 `crates/singlefs-verification/src/crash_amplification.rs:23950` 「pub fn run_from_environment_with_observers(」。
- 结果里卡的来路写死：`crates/singlefs-verification/src/crash_amplification.rs:24562` 「let gpu_cards_taken_from = GpuCardsSource::NoCardNeeded;」；只录、空区间那两个提早交回的分支也写 `NoCardNeeded`（第 24426、24500 行）。
- 第 ① 段判（走 GPU 时）`crates/singlefs-verification/src/crash_amplification.rs:24680` 「let gpu_judging = judge_my_blocks_on_gpu(」，进 `crates/singlefs-verification/src/crash_amplification.rs:33850` 「pub(crate) fn judge_my_blocks_on_gpu(」。走到写待判行之前只有两个判断：一次性库（`crates/singlefs-verification/src/crash_amplification.rs:33859` 「if library.take().is_a_disposable_library() {」）与服务在不在（`crates/singlefs-verification/src/crash_amplification.rs:33862` 「let control_command = exit_unless_the_gpu_verification_server_runs()?;」）。这一支里没有上面清单里任何一个取卡调用，交回的账 `cards_used: 0`。
- 走到取卡之前的判断：没有（这一支不取卡）。

### 1b 只排派活计划那一趟：成立
入口同上，开关 `crates/singlefs-verification/src/crash_amplification.rs:23990` 「let placement_only = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY")」；分支 `crates/singlefs-verification/src/crash_amplification.rs:24564` 「if placement_only {」。
- 到这个分支前的判断：读环境选路（`crates/singlefs-verification/src/crash_amplification.rs:23986` 起的 `computation_routes`）、本趟要判（`crates/singlefs-verification/src/crash_amplification.rs:24394` 「if !judges {」之后）、空区间的那一支另有一个出口（`crates/singlefs-verification/src/crash_amplification.rs:24451` 「if block_ranges.is_some() && my_blocks.is_empty() && !placement_only {」，只排计划那一趟不走它）。
- 分支里：走 GPU 才排（`crates/singlefs-verification/src/crash_amplification.rs:24565` 「let placement_line = match routes.crash_judge {」），走 CPU 报错（`crates/singlefs-verification/src/crash_amplification.rs:24579` 「"只排派活计划排的是 GPU 判器的活：要配置开着 GPU（ENABLE_GPU=1）".to_string(),」），随后交回；不借草稿区、不建判器、不试判。观察者的录入段在这一趟也跳过：`crates/singlefs-verification/src/crash_amplification.rs:24364` 「let observer_recording = if records && !placement_only && !observers.is_empty() {」。
- 新测试 `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:129` 「card_pool_started_in_this_process().is_none(),」钉「这个进程的卡池没起」（我没跑它，见末尾命令）。

### 1c 第 ② 段核对：成立
调用点 `crates/singlefs-verification/src/crash_amplification.rs:24759` 「tally: verify_my_blocks_on_gpu(」，进 `crates/singlefs-verification/src/crash_amplification.rs:34512` 「fn verify_my_blocks_on_gpu(」（`verify_my_blocks_on_gpu(` 在 `src/`、`tests/` 里只有这一个调用点，grep 现跑）。
- 之前的判断：本趟没叫停（`crates/singlefs-verification/src/crash_amplification.rs:24754` 「let facts_verifier_route = (!stop_was_requested).then_some(routes.facts_verifier);」）、核对走 GPU（`crates/singlefs-verification/src/crash_amplification.rs:24757` 「Some(ComputedOn::Gpu) => Some(VerificationOutcome {」）。进函数后：一次性库（`crates/singlefs-verification/src/crash_amplification.rs:34521` 「if library.take().is_a_disposable_library() {」）、服务在不在（`crates/singlefs-verification/src/crash_amplification.rs:34524` 「let control_command = exit_unless_the_gpu_verification_server_runs()?;」），然后写核对待判行、等。整条路里没有取卡调用。走 CPU 的核对（第 24770 行起的 `FactsVerifierOn::Cpu`）也不碰卡。

### 1d 观察者交回的盘：不成立（任务进程里有取卡、起卡池、借显存的路）
调用链（逐跳 文件:行）：
1. 入口 `crates/singlefs-verification/src/crash_amplification.rs:24869` 「let pool_checker = if observers.is_empty() {」，没挂观察者交 `None`；挂了的 `crates/singlefs-verification/src/crash_amplification.rs:24872` 「Some(pool_checker_of_the_images_after_the_observers(」。崩溃枚举用例那一路同一个选路在 `crates/singlefs-verification/src/crash_state_enumeration.rs:2350` 「let pool_checker = pool_checker_of_the_images_on_the_route(route)?;」。
2. `crates/singlefs-verification/src/crash_amplification.rs:1630` 「pub(crate) fn pool_checker_of_the_images_after_the_observers(」：走 CPU 交 `OnTheCpu`；走 GPU `crates/singlefs-verification/src/crash_amplification.rs:1635` 「ComputedOn::Gpu => pool_checker_of_the_images_on_the_gpu(),」，建的是惰性成员 `crates/singlefs-verification/src/crash_amplification.rs:31289` 「TakingTheGpuCardsAtTheFirstImageToJudge(CardsTakenAtTheFirstImageToJudge),」，建的时候不取卡。
3. 头一张要判的盘：`crates/singlefs-verification/src/crash_amplification.rs:32432` 「pub(crate) fn judge_the_observers_of_the_flow(」里 `crates/singlefs-verification/src/crash_amplification.rs:32688` 「if !to_judge.is_empty() {」之后 `crates/singlefs-verification/src/crash_amplification.rs:32705` 「let judged = pool_checker.judge(library, &images)?;」，进 `crates/singlefs-verification/src/crash_amplification.rs:31404` 「let cards_of_this_run = cards_taken_at_the_first_image.cards_taken_now_or_before();」，取卡在 `crates/singlefs-verification/src/crash_amplification.rs:31322` 「.get_or_init(|| match (self.take_the_cards)() {」。
4. 取卡函数 `crates/singlefs-verification/src/crash_amplification.rs:33252` 「pub(crate) fn gpu_cards_of_this_run_from_the_multi_host_configuration(」：读这个进程的多机配置（`crates/singlefs-verification/src/crash_amplification.rs:33254` 「let configuration = multi_host_configuration_of_this_process()?;」），再 `crates/singlefs-verification/src/crash_amplification.rs:33256` 「crate::gpu_card_configuration::cards_by_priority_from_configuration_or_environment(Some(」，再 `crates/singlefs-verification/src/crash_amplification.rs:33259` 「gpu_cards_in_use(cards_by_priority.as_ref())」，再 `crates/singlefs-verification/src/crash_amplification.rs:33229` 「crate::gpu_unit_checks::gpu_test_start::card_pool_of_this_process_started_now()?」（起这个进程的卡池）。
5. 卡池起的时候读本机硬件：`crates/singlefs-verification/src/gpu_unit_checks.rs:3105` 「pub(in crate::gpu_unit_checks) fn start_the_gpu_card_pool() -> GpuCardPoolStart {」里 `crates/singlefs-verification/src/gpu_unit_checks.rs:3119` 「&enumerate_vulkan_adapters(),」枚举 Vulkan 适配器、读 PCI 总线，`crates/singlefs-verification/src/gpu_unit_checks.rs:3132` 「GpuCardPoolStartOutcome::Started(Box::new(open_the_card_pool(named.cards)))」开池。
6. 借显存与起设备：判之前 `crates/singlefs-verification/src/crash_amplification.rs:32198` 「let pool = the_card_pool_this_run_takes_its_cards_from();」，`crates/singlefs-verification/src/crash_amplification.rs:32202` 「.lease_the_first_card_with_room_now(」，借到后在卡上起设备（`crates/singlefs-verification/src/gpu_unit_checks.rs:2849` 「let mut card = match self.started_card(&booking) {」）。

走到取卡之前的判断（依次）：不是只排计划（24564 分支已交回）→ 判段要跑（24394）→ 没叫停（`crates/singlefs-verification/src/crash_amplification.rs:24787` 「if stop_was_requested || stop_is_requested() {」）→ 挂了观察者（24869）→ 选路走 GPU（1635）→ 有要现判的盘（32688）→ 取卡函数里：环境里有服务交的卡串（`crates/singlefs-verification/src/gpu_card_configuration.rs:608` 「std::env::var(CARDS_BY_PRIORITY_KEY).ok(),」）或有领卡通道（`crates/singlefs-verification/src/gpu_card_configuration.rs:587` 「(None, Some(channel)) => lease(channel).map(Some),」），两样都没有且配置开了 GPU 就报错（`crates/singlefs-verification/src/gpu_card_configuration.rs:644` 「if configured_gpu_is_enabled()」）→ 池起得成（有 NVIDIA 卡）。
取不到时的出口：`crates/singlefs-verification/src/crash_amplification.rs:31356` 「std::process::exit(IMAGES_AFTER_THE_OBSERVERS_WITHOUT_A_CARD_EXIT_CODE);」，码 `crates/singlefs-verification/src/crash_amplification.rs:31336` 「pub const IMAGES_AFTER_THE_OBSERVERS_WITHOUT_A_CARD_EXIT_CODE: i32 = 78;」。

对条款的判断：统一任务服务提交时拒带领卡声明的命令（`task-server/task_server_lib.py:2018` 「CARD_DECLARATION_MARKS = ("# task-server-card-lease:", "# task-server-gpu-cards:", "# task-server-gpu-arguments:")」），所以经服务交的任务里环境没有卡串与领卡通道，这一支多半在第 4 步报错、退 78，不会真开卡。但代码本身没有「任务里不取卡」的判断：只要环境里有卡串或通道（别的起法、手跑、测试守卫设回），它就起卡池、借显存、判盘。条款说任务「不领卡、不分卡」，这条路是条款之外的取卡路，所以写不成立。
（对照：本仓自己的测试 `crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs:5` 「要本机一张 NVIDIA 独显」就是按这条路写的，它在任务里要卡，属背景材料说的 29 条红之列，这一轮不判。）

### 1e 不在四处里的另外三条取卡路（同样不成立）
- 崩溃枚举用例的观察者盘：`crates/singlefs-verification/src/crash_state_enumeration.rs:2704` 「crate::crash_amplification::gpu_cards_of_this_run_from_the_multi_host_configuration,」，与 1d 同一条链，只是入口换成 `crates/singlefs-verification/src/crash_state_enumeration.rs:2350`。
- 给定状态表引擎（随机历史两项，崩溃注入默认批等调）：入口 `crates/singlefs-verification/src/crash_amplification.rs:25349` 「pub fn start_from_environment(」，判表走 GPU 的分支 `crates/singlefs-verification/src/crash_amplification.rs:26404` 「JudgedBy::GpuJudge => match self.judge_the_table_on_the_gpu(table_with_its_columns)? {」，第一张表上卡时取卡 `crates/singlefs-verification/src/crash_amplification.rs:25945` 「let taken = self.cards_taken_at_the_first_table_judged_on_the_gpu()?;」→ `crates/singlefs-verification/src/crash_amplification.rs:25467` 「gpu_cards_in_use(cards_by_priority.as_ref())?;」→ 借卡 `crates/singlefs-verification/src/crash_amplification.rs:25966` 「match pool.lease_the_first_card_with_room_waiting_for_the_other_threads(」。这一段不在这三批的 diff 里（`git diff HEAD -U0` 里没有改到这几行），是原有代码，但仍在被判的文件里、条款之外。
- 测试直接调的对拍入口：`crates/singlefs-verification/src/crash_amplification.rs:4826` 「pub fn judge_my_blocks_on_gpu_cards(」（卡由调用方起好交进来），调用者是 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（5793、5909、5943 行）。属测试自己起卡，不是任务进程的产品路径，归背景材料说的那 29 条。
- 附带：测试共用层 `crates/singlefs-verification/tests/common_crash_points/mod.rs:555` 「fn cards_of_the_caller_for_a_test_configuration_with_the_gpu() -> Option<EnvironmentRestoredOnDrop>」仍把调用方的卡串（`crates/singlefs-verification/tests/common_crash_points/mod.rs:580` 「Some(common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run())」）设回给每一趟放量，第 ①② 段不用它，只有 1d 那条路会读到。

### 1f 留下的死分支（推的，没跑）
第 ① 段按卡池借草稿区的整条分支还在：`crates/singlefs-verification/src/crash_amplification.rs:4761` 「GpuCardsSource::TheCardPoolOfThisProcess => cards_of_the_pool_with_what_each_lends_now(」与借草稿区的调用点 5100。唯一的调用者 `crates/singlefs-verification/src/crash_amplification.rs:4826` 「pub fn judge_my_blocks_on_gpu_cards(」只传 `GpuCardsSource::DevicesStartedForThisRun`（第 4848 行），`TheCardPoolOfThisProcess` 只在 `gpu_cards_in_use` 里造（第 33242 行），而 `gpu_cards_in_use` 的两个调用点（25467、33259）都不走进第 ① 段，所以这条分支在产品路径上走不到；是删「在任务里借草稿区」时没删净的壳，建议主 agent 交实现员核。
推翻 1 的现象：在带 `gpu` 特性的二进制里，让一趟挂了观察者的放量走到头一张要判的盘，同时环境里设 `GPU_CARDS_BY_PRIORITY`：若此时进程没有起卡池（`card_pool_started_in_this_process()` 仍是 `None`）、也没有 `GPU_CARD_LEASE` 行，1d 就推翻（待主 agent 跑，见末尾）。

## 问题 2：核对待判行（表标签 0x1F）的键、值与一帧写回的两个守卫

结论：键、值、两个守卫的**形状**够让 GPU 服务在一帧里原子地写核对行、删核对待判行，且写错块、写错版本、盖掉别人刚写的行都会被守卫或键拒掉；任务一侧「不在才写」与读回记账对着库里同一把键。但「服务真的这么写」**推不出**：`gpu-verification-server/` 与 `prune-server/` 里没有任何读写 0x1F 的代码（见下），服务一侧的写帧要带的认领键与持有者在这一帧的类型里没有。两处缺口写在「缺什么」。

### 2a 键（逐段）
- 表标签：`crates/singlefs-verification/src/verdict_store.rs:1805` 「Self::FactsVerifierPendingRows => 0x1F,」，读回 `crates/singlefs-verification/src/verdict_store.rs:1883` 「0x1F => Some(Self::FactsVerifierPendingRows),」；文件头登记 `crates/singlefs-verification/src/verdict_store.rs:109` 「核对待判行 | `1F` ‖ 流键（32 字节，[`FactsVerifierPendingStreamKey`]）‖ 块号（8 字节，计划里全部点的块按次序编号）」。头一个字节 0x1F 小于 prune-server 留给自己记账的段：`prune-server/src/protocol.rs:202` 「pub const SERVICE_KEY_LEADING_BYTE_MINIMUM: u8 = 0xF0;」，按这一处的判据客户端写它不被拒（别的拒绝条件我没逐个读；`prune-server/src/storage.rs` 与 `gpu_judging_rows.rs` 的文件头还写着「0x00–0x1B」，是没跟上的旧说法，代码按 0xF0 判）（`prune-server/src/protocol.rs:205` 「pub fn is_service_key(key: &[u8]) -> bool {」按 0xF0 判）。
- 流键 32 字节：`crates/singlefs-verification/src/verdict_store.rs:346` 「pub struct FactsVerifierPendingStreamKey(pub [u8; SHA256_DIGEST_BYTES]);」，算法 `crates/singlefs-verification/src/crash_amplification.rs:34231` 「pub fn facts_verifier_pending_stream_key(」，内容 `crates/singlefs-verification/src/crash_amplification.rs:34236` 「let mut message = b"singlefs facts-verifier-pending stream key\n".to_vec();」之后接录下的流程摘要、事实表摘要、期待的核对器组键，取 SHA-256。
- 块号 8 字节大端：键的拼法 `crates/singlefs-verification/src/verdict_store.rs:2246` 「Self::FactsVerifierPendingRow(stream, block_number) => {」那一支先放流键字节（第 2247 行）再放块号大端字节（第 2248 行）。块号是计划里全部点的块按次序的序号：`crates/singlefs-verification/src/crash_amplification.rs:34343` 「let mut block_number = 0u64;」起逐块加一。
- 键里没有的：块键、态区间、版本都在值里；键只认「哪条流的第几块」。

### 2b 值（逐栏）
结构 `crates/singlefs-verification/src/verdict_store.rs:350` 起，编码在 `crates/singlefs-verification/src/verdict_store.rs:382` 起：
- 格式号 1 字节：`crates/singlefs-verification/src/verdict_store.rs:366` 「const FACTS_VERIFIER_PENDING_ROW_FORMAT_NUMBER: u8 = 1;」；读的一侧只认这一个（`crates/singlefs-verification/src/verdict_store.rs:398` 「if bytes.len() != FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES」后接格式号比较），没有认别的号的分支。
- 块键 72 字节（`crates/singlefs-verification/src/verdict_store.rs:352` 「pub recorded_block: VerdictBlockKey,」）：服务照它写核对行的键。
- 录下的流程摘要 32 字节（`crates/singlefs-verification/src/verdict_store.rs:354` 「pub recorded_flow_digest: [u8; SHA256_DIGEST_BYTES],」）：服务读回流、排计划、取段展开。
- 事实表摘要 32 字节（`crates/singlefs-verification/src/verdict_store.rs:356` 「pub facts_digest: [u8; SHA256_DIGEST_BYTES],」）：服务读回事实表，也进核对行的值与核对键。
- 态序号区间两端各 8 字节（`crates/singlefs-verification/src/verdict_store.rs:358` 「pub state_ordinals: std::ops::Range<u64>,」）：这一块要核的状态；服务读材料时核它就是计划里块键相同的那一块（`crates/singlefs-verification/src/crash_amplification.rs:34612` 「if !is_a_block_of_the_plan {」）。
- 期待的 GPU 核对器组键 32 字节（`crates/singlefs-verification/src/verdict_store.rs:360` 「pub expected_facts_verifier_units: JudgingUnitsFingerprint,」）：服务自己编进去的那一把与它不同就不认领；同时进核对键。
- 入池序号 8 字节（`crates/singlefs-verification/src/verdict_store.rs:362` 「pub pool_entry_number: u64,」）：写行那一刻的纳秒数（`crates/singlefs-verification/src/crash_amplification.rs:33419` 「fn pool_entry_number_now() -> u64 {」），服务按它排先后。
- 总长与读回：`crates/singlefs-verification/src/verdict_store.rs:369` 「const FACTS_VERIFIER_PENDING_ROW_VALUE_BYTES: usize = 1」，读完要正好到末尾（`crates/singlefs-verification/src/verdict_store.rs:418` 「cursor.is_at_the_end().then_some(Self {」）。
- 与待判行（0x1D）比：少了「判表摘要」「结论版本键」「强制现判」三栏。判表、结论版本是判器那一边的东西，核对没有；强制现判是复用哨兵对判器结论抽的；核对这一段（`crash_amplification.rs` 第 34218 到 34718 行）里没有强制现判的写法（按这个行区间找「forced」「强制」零命中，现跑），所以没有这一栏。缺的不是漏项。

### 2c 一帧写回的两个守卫（各守什么）
帧的结构：`crates/singlefs-verification/src/verdict_store.rs:453` 「pub struct FactsVerifierRowsForOneFrame {」。
- 守卫一（核对行守卫）：`crates/singlefs-verification/src/verdict_store.rs:457` 「pub verifier_row_key: Vec<u8>,」与 `crates/singlefs-verification/src/verdict_store.rs:458` 「pub verifier_row_as_read: Option<Vec<u8>>,」。编的时候先读这一块在这把核对键下的行：`crates/singlefs-verification/src/verdict_store.rs:4833` 「.read_values_kept_in_or_out(&[verifier_row_key.as_slice()])?」，键是 `crates/singlefs-verification/src/verdict_store.rs:4829` 「VerdictStoreKey::VerifierBlock(&pending_row_as_read.recorded_block, verifier_digest)」。写的那一刻库里那一行与读到的不同就整帧拒：防盖掉别人刚写的核对行（文件头 `crates/singlefs-verification/src/verdict_store.rs:453` 起的文档注释）。读到的与要写的逐字节相同时不写它（`crates/singlefs-verification/src/verdict_store.rs:4841` 「(!is_stored_identically).then(|| (verifier_row_key.clone(), verifier_row_value));」，对应 `crates/singlefs-verification/src/verdict_store.rs:455` 「pub verifier_row_to_put: Option<(Vec<u8>, Vec<u8>)>,」），只删待判行。
- 守卫二（核对待判行守卫）：`crates/singlefs-verification/src/verdict_store.rs:460` 「pub pending_row_key_to_delete: Vec<u8>,」与 `crates/singlefs-verification/src/verdict_store.rs:461` 「pub pending_row_as_read: Vec<u8>,」，值取自服务取这一行时读到的那一份（`crates/singlefs-verification/src/verdict_store.rs:4852` 「pending_row_as_read: pending_row_as_read.to_value_bytes(),」）。删的时候库里那一行与读到的不同就整帧拒：防删掉判段在服务读完之后补写的那一行。
- 核对行的值与任务一侧存核对行同一种编法：`crates/singlefs-verification/src/verdict_store.rs:734` 「fn verifier_block_value_bytes(facts_digest: &[u8; 32], bits_of_each_state: &[u32]) -> Vec<u8> {」，`crates/singlefs-verification/src/verdict_store.rs:5240` 「pub fn read_verifier_block(」按它读回。
- 帧编之前服务照调的核对三样：材料 `crates/singlefs-verification/src/crash_amplification.rs:34593` 「pub fn material_of_a_facts_verifier_pending_block(」、现算核没核完 `crates/singlefs-verification/src/crash_amplification.rs:34643` 「pub fn facts_verifier_pending_block_is_verified_in_the_library(」、编帧 `crates/singlefs-verification/src/crash_amplification.rs:34662` 「pub fn facts_verifier_frame_of_a_pending_block(」；编帧时核流键是照这一行算出的那一把（`crates/singlefs-verification/src/crash_amplification.rs:34692` 「!= facts_verifier_pending_stream_key(」）与结论位条数等于这一块状态数（`crates/singlefs-verification/src/crash_amplification.rs:34703` 「if u64::try_from(bits_of_each_state.len()).ok() != Some(states_of_the_block) {」）。

### 2d 任务一侧两处对着库的同一把键（逐把对）
任务一侧写：
- 「不在才写」：`crates/singlefs-verification/src/verdict_store.rs:4694` 「pub fn put_the_facts_verifier_pending_row_unless_present(」，键 `crates/singlefs-verification/src/verdict_store.rs:4700` 「let key = VerdictStoreKey::FactsVerifierPendingRow(stream, block_number).to_key_bytes();」，先认领这把键 `crates/singlefs-verification/src/verdict_store.rs:4703` 「match self.service.claim(&key.0, "认领核对待判行")? {」；认到（从没写过）就写（`crates/singlefs-verification/src/verdict_store.rs:4704` 「ClaimOutcome::Claimed => {」）；服务答已有结论（键写过，含服务核完删掉留下的删除记号）就读回，在的交回库里那一份，不在的照写（`crates/singlefs-verification/src/verdict_store.rs:4708` 「ClaimOutcome::AlreadyConcluded => {」）；别的客户端认领着就隔一会再问、等满上限报错（`crates/singlefs-verification/src/verdict_store.rs:4722` 「ClaimOutcome::HeldByOther { .. } => {」）。
- 调用处：流键 `crates/singlefs-verification/src/crash_amplification.rs:34336` 「let stream = facts_verifier_pending_stream_key(」，块号逐块编（34343 起），只写这台机器领的块（`crates/singlefs-verification/src/crash_amplification.rs:34348` 「if !is_mine(number_of_this_block) {」），写之前先看核对行在不在（`crates/singlefs-verification/src/crash_amplification.rs:34354` 「.read_verifier_block(&key, &verifier_row_key)」，在就记 `blocks_already_verified` 不写，`crates/singlefs-verification/src/crash_amplification.rs:34358` 「tally.blocks_already_verified += 1;」），写的结局 `Written` 用自己的行，`AlreadyPresent` 用库里那一份（`crates/singlefs-verification/src/crash_amplification.rs:34380` 「crate::verdict_store::FactsVerifierPendingRowPut::AlreadyPresent(present) => {」）。

任务一侧读回记账：
- 等的那一圈读待判行：`crates/singlefs-verification/src/crash_amplification.rs:34402` 「.read_facts_verifier_pending_rows(」，键的拼法 `crates/singlefs-verification/src/verdict_store.rs:4749` 「VerdictStoreKey::FactsVerifierPendingRow(stream, *block_number).to_key_bytes()」，与「不在才写」的 4700 是同一个构造函数、同一个流键变量（34336）、同一个块号。
- 待判行不在了就看核对行在不在：`crates/singlefs-verification/src/crash_amplification.rs:34417` 「.read_verifier_block(&waited.row.recorded_block, &verifier_row_key)」；在就算核完；不在补写（`crates/singlefs-verification/src/crash_amplification.rs:34424` 「// 核对待判行不在了却还没核过：照不在才写补写，接着等」）。
- 记账：`crates/singlefs-verification/src/crash_amplification.rs:34476` 「.read_verifier_block(&waited.row.recorded_block, &verifier_row_key)」读核对行，结论位条数要等于这一块状态数（`crates/singlefs-verification/src/crash_amplification.rs:34486` 「if u64::try_from(verifier_row.bits.len()).ok() != Some(states_of_the_block) {」），再累加 `crates/singlefs-verification/src/crash_amplification.rs:34494` 「tally.blocks_verified += 1;」。

与服务一侧对键（服务一侧的代码这个仓里还没有，下面是帧编法里已定的键）：
- 待判行键：服务扫表 `crates/singlefs-verification/src/verdict_store.rs:4775` 「pub fn every_facts_verifier_pending_row(」按 `crates/singlefs-verification/src/verdict_store.rs:4781` 「VerdictStoreKey::WholeTablePrefix(VerdictStoreTable::FactsVerifierPendingRows)」整表取（键里切出流键与块号），编帧时删的键 `pending_row_key_to_delete` 用同一个构造函数（`crates/singlefs-verification/src/verdict_store.rs:4846` 起）；流键在编帧时核过是照这一行算出的（34692）。
- 核对行键：任务一侧 `crates/singlefs-verification/src/crash_amplification.rs:34324` 「let verifier_row_key = facts_verifier_row_key(verifier_units, facts_digest);」；服务一侧 `crates/singlefs-verification/src/crash_amplification.rs:34630` 「fn facts_verifier_row_key_of_the_pending_row(」用 `crates/singlefs-verification/src/crash_amplification.rs:34634` 「&JudgingUnitsKey::of_the_parts(row.expected_facts_verifier_units.0, Vec::new()),」重造；`facts_verifier_row_key` 只读键里的指纹（`crates/singlefs-verification/src/crash_amplification.rs:5560` 「let mut message = b"singlefs facts verifier row\n".to_vec();」之后只接 `fingerprint()` 与事实表摘要），`of_the_parts`（`crates/singlefs-verification/src/crash_identity.rs:423` 「pub fn of_the_parts(」）原样存下指纹，行里的期待指纹正是任务一侧写下的 `crates/singlefs-verification/src/crash_amplification.rs:34317` 「let expected_facts_verifier_units = JudgingUnitsFingerprint(*verifier_units.fingerprint());」。两边键相同。
- 第 ③ 段比对用的核对键也是同一个变量：`crates/singlefs-verification/src/crash_amplification.rs:24759` 起传进去的 `&gpu_facts_verifier_units` 与 `facts_digest_read_back`，和 24856–24857 行算 `verifier_digest_to_compare_against` 用的同一对。

### 2e 能不能「原子地写回而不重判、不漏判」逐项推
- 不漏判（核对行没写成而待判行被删了）：删待判行与写核对行在同一帧，帧里的两个守卫在写的那一刻核；服务取行之后判段补写了新行（入池序号是纳秒数，补写出来的行与读到的字节不同）会被守卫二拒掉（行字节里带纳秒数的入池序号，两次写同一纳秒的可能我没排除，推的）；别的服务线程先写了不同的核对行会被守卫一拒掉。读到的核对行与要写的相同时只删待判行，也不会丢：核对行本来就在。形状够。
- 不重判（同一块被核两遍）：任务一侧「不在才写」+ 认领键，同一块的行只写一份；服务一侧核之前先在库上现算核没核完（`crates/singlefs-verification/src/crash_amplification.rs:34643` 那一个函数），核完的只删行。这两层都在，但**两个服务线程同时取到同一行时都会去核**：帧里没有认领键与持有者（`crates/singlefs-verification/src/verdict_store.rs:453` 「pub struct FactsVerifierRowsForOneFrame {」这个类型里没有这两样），先写的成功，后写的被守卫拒掉——结果对，白核一遍。判器那一族的写帧有认领键：`prune-server/src/gpu_judging_rows.rs:406` 「claim_key: gpu_claim_key(&self.block.stream_key, self.block.block_number),」，而 prune-server 的写结论帧要求认领键非空：`prune-server/src/protocol.rs:1170` 「if view.claim_key.is_empty() {」。所以服务一侧要么在这一帧外另加认领键，要么这一族接受重核；这一点这个仓里推不出。
- 缺什么（写成「推不出」的两样）：
  1. 服务一侧读写 0x1F 的代码不在：`grep -rniE 'facts_verifier_pending|FactsVerifierPending|facts-verifier-pending|verifier_rows_for_one_frame' gpu-verification-server prune-server`（`.rs`、`.py`、`.md`、`control`）零命中（命令现跑，只有 `prune-server/src/sha256.rs` 因为别的缘故命中了 `0x1F` 字面，与此无关）。两个守卫「在写的那一刻核」这句话，今天只是文档与帧类型里的约定，没有代码在核。
  2. 这一帧要通过 prune-server 的 `GuardedConclusionWrite`（带认领键、持有者身份、守卫键与守卫期待、条目、带期待的删键，见 `prune-server/src/gpu_judging_rows.rs:410` 「deleted_keys: vec![GuardedDeletion {」），帧类型里缺认领键与持有者；认领键归哪一族（`gpu-claim/` 那一族照流键与块号拼，0x1F 与 0x1D 的流键用不同的标签算，不会撞）要服务一侧定。
- 没推出毛病的：两个守卫的期待值与删键的期待值（待判行读到的字节）都带在帧里，守卫键与删键不相同（协议要求「同一帧里不又写又删同一把键」，核对行键与待判行键表标签不同，满足）。

推翻 2 的现象：服务一侧补上 0x1F 之后，用两个线程同时取同一行跑一次，若出现一块核对行被写两遍且内容不同、或待判行被删而核对行不在，则本条「够」被推翻；若两个守卫之一在写帧时没被核，同样推翻。

## 问题 3：等服务的那一段，四种情形各走哪一支

结论：四种情形都有确定的一支；服务不在与服务中途退出退 78 并打一行，核对行被别的进程先写不退出、记账照走，一次性库不退出、交回错误串（用例里 `.expect` 成 panic，测试进程退 101）。另有一种背景材料没单列、但会碰上的：服务在跑而不扫 0x1F（今天的现状），这一段不会返回（见 3e）。
等的间隔与问服务的间隔：每圈睡 2 秒（`crates/singlefs-verification/src/crash_amplification.rs:33304` 「std::time::Duration::from_secs(2);」），问服务的间隔 30 秒（`crates/singlefs-verification/src/crash_amplification.rs:33311` 「std::time::Duration::from_secs(30);」）折成圈数向上取整（`crates/singlefs-verification/src/crash_amplification.rs:33319` 「.div_ceil(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL.as_secs());」，= 15 圈）。

四种情形（一行一种）：
1. 服务不在（进核对段时）：`crates/singlefs-verification/src/crash_amplification.rs:34524` 「let control_command = exit_unless_the_gpu_verification_server_runs()?;」→ `crates/singlefs-verification/src/crash_amplification.rs:33838` 「exit_because_the_gpu_verification_server_is_not_running(&line);」→ `crates/singlefs-verification/src/crash_amplification.rs:33822` 「std::process::exit(GPU_VERIFICATION_SERVER_NOT_RUNNING_EXIT_CODE);」，退 78（`crates/singlefs-verification/src/crash_amplification.rs:33344` 「pub const GPU_VERIFICATION_SERVER_NOT_RUNNING_EXIT_CODE: i32 = 78;」），标准错误打一行以 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_NOT_RUNNING` 起头（`crates/singlefs-verification/src/crash_amplification.rs:33341` 「"CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_NOT_RUNNING";」），后面是 `control=<控制命令路径> exit_code=<码> said=<控制命令的标准错误>` 或 `not_started=<原因>`（`crates/singlefs-verification/src/crash_amplification.rs:33775` 「"exit_code={} said={}",」、`crates/singlefs-verification/src/crash_amplification.rs:33779` 「GpuVerificationServerCheck::NotStarted(reason) => format!("not_started={reason}"),」）。**产品路径上第 ① 段先碰到**（同一个函数 `crates/singlefs-verification/src/crash_amplification.rs:33862` 「let control_command = exit_unless_the_gpu_verification_server_runs()?;」，在 ① 的写待判行之前），② 的这一次只在 ① 过了之后才走到。
2. 服务中途退出（等的时候）：等的圈里每走满 15 圈问一次（`crates/singlefs-verification/src/crash_amplification.rs:34438` 「if reckonings_since_the_server_was_last_asked」、`crates/singlefs-verification/src/crash_amplification.rs:34471` 「reckonings_since_the_server_was_last_asked += 1;」），问的结果 `crates/singlefs-verification/src/crash_amplification.rs:34441` 「if let Some(server_not_running) = (waiting.server_not_running_line)() {」，不在跑交回 `crates/singlefs-verification/src/crash_amplification.rs:34442` 「return Ok(WaitingForTheFactsVerifierEnded::ServerNotRunning {」，`verify_my_blocks_on_gpu` 里 `crates/singlefs-verification/src/crash_amplification.rs:34546` 「WaitingForTheFactsVerifierEnded::ServerNotRunning { line } => {」→ 34547 `exit_because_the_gpu_verification_server_is_not_running(&line)`，退 78、打的行同第 1 行。发现要等到下一次问（至多约 30 秒加问一次的耗时）；已核完的核对行留在库里，未核完的核对待判行也留着（下一趟照「不在才写」接着等）。问的办法是起 `python3 <控制命令> check`（`crates/singlefs-verification/src/crash_amplification.rs:33804` 「Command::new("python3")」），控制命令退非 0（不只是服务不在：版本对不上退 1、单元环境缺 CARGO 退 78 都算）一律走这一行、退 78。
3. 核对行被别的进程先写：三种时点。（a）写待判行之前已在：`crates/singlefs-verification/src/crash_amplification.rs:34354` 「.read_verifier_block(&key, &verifier_row_key)」读到就记 `blocks_already_verified`（34358）、不写待判行、不等，不打行、不退出。（b）写待判行时别的进程已写或正在写：认领与读回那一支（`crates/singlefs-verification/src/verdict_store.rs:4708` 「ClaimOutcome::AlreadyConcluded => {」读到在的交回库里那一份，`crates/singlefs-verification/src/crash_amplification.rs:34380` 「crate::verdict_store::FactsVerifierPendingRowPut::AlreadyPresent(present) => {」用它），被别人认领着就等（`crates/singlefs-verification/src/verdict_store.rs:4722` 「ClaimOutcome::HeldByOther { .. } => {」，等满 `PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT` 报错：`crates/singlefs-verification/src/verdict_store.rs:4723` 「if waiting_started.elapsed() >= PRUNE_SERVER_NUMBER_CLAIM_WAIT_LIMIT {」，错误串以 `Err` 交回）。（c）写之后别的进程写了核对行：待判行已不在而核对行在，记成核完（`crates/singlefs-verification/src/crash_amplification.rs:34417` 「.read_verifier_block(&waited.row.recorded_block, &verifier_row_key)」，在则 `crates/singlefs-verification/src/crash_amplification.rs:34421` 「verified.push(waited);」）；待判行还在而核对行已在，继续等服务删行（34411–34414 行那一支），服务不在就走第 2 行。读回时结论位条数与状态数不等交 `Err`（34486）。除第 2 行与等不到认领的错误外，不退出、不打行（进度行只在还剩的块数变了、且离上一行满 5 分钟才打，`crates/singlefs-verification/src/crash_amplification.rs:33332` 起的间隔常量）。
4. 库是一次性库：`crates/singlefs-verification/src/crash_amplification.rs:34521` 「if library.take().is_a_disposable_library() {」→ `crates/singlefs-verification/src/crash_amplification.rs:34522` 「return Err(PENDING_ROWS_IN_A_DISPOSABLE_LIBRARY_ARE_UNSUPPORTED.to_string());」，在写任何一行之前，不打行、不调 `process::exit`；错误串（常量 `crates/singlefs-verification/src/crash_amplification.rs:33348` 「pub const PENDING_ROWS_IN_A_DISPOSABLE_LIBRARY_ARE_UNSUPPORTED: &str =」）经 `?` 从 `run_from_environment_with_observers` 交回 `Err(String)`；测试共用层 `crates/singlefs-verification/tests/common_crash_points/mod.rs:743` 「.expect("按环境跑得完");」把它变成 panic（测试进程退 101）。**这一处在产品路径上走不到**：`verify_my_blocks_on_gpu(` 只有一个调用点（24759），它之前第 ① 段 `judge_my_blocks_on_gpu` 在同一个库上有同一个判断（`crates/singlefs-verification/src/crash_amplification.rs:33859` 「if library.take().is_a_disposable_library() {」，`?` 在 `crates/singlefs-verification/src/crash_amplification.rs:24690` 「.map_err(|reason| gpu_judging_refusal_naming_the_flow(flow_name, reason))?」），一次性库总是先死在 ①。用例钉的也是这一句：`crates/singlefs-verification/tests/crash_amplification_run_of_both_stages_records_the_flow_before_it_leases_a_card.rs:97` 「refusal, PENDING_ROWS_IN_A_DISPOSABLE_LIBRARY_ARE_UNSUPPORTED,」。

### 3e 服务在跑而不扫 0x1F（现状）
等的圈（`crates/singlefs-verification/src/crash_amplification.rs:34435` 「if blocks_not_verified_yet.is_empty() || stop_is_requested() {」）只有三个出口：块都核完、被叫停、问到服务不在。服务在跑而不认 0x1F（2e 的缺口一，零命中）时三个都不发生：问服务的 `control check` 退 0（它只看单元在跑与版本对不上行，版本对不上行由服务打、服务不认识 0x1F 就不会打），进度行因还剩块数不变不打，这个进程一直睡、一直不退，交给统一任务服务的卡住判定或四小时上限去停。它不会被报成「核完」，也不会退 0；但背景材料「反推」要问的「待判行写了而服务永远不判时汇总行说什么」——答案是没有汇总行、没有退出码，只有挂着。这一条是代码读出来的推，没跑过。
推翻 3 的现象：任一情形实际退出码或打的行与上面不同（例如第 2 行实际不退 78、第 4 行在 ② 里真被走到）。

## 问题 4：正文「实现今天的样子」第 6 条与第 8 条，逐句核

### 第 6 条（里层线程份额下限 2、锁与名额的次序）
- 句一「测试进程里层线程份额：`crash_library.rs:561` 的 `FEWEST_INNER_THREADS_OF_THE_SHARE = 2`」：对得上。`crates/singlefs-verification/src/crash_library.rs:561` 「pub(crate) const FEWEST_INNER_THREADS_OF_THE_SHARE: usize = 2;」。
- 句二「`shares_of_the_test_process`（第 566 行）里层下限取它」：对得上。`crates/singlefs-verification/src/crash_library.rs:566` 「pub(crate) fn shares_of_the_test_process(heavy_threads: usize) -> SharesOfTheTestProcess {」，下限 `crates/singlefs-verification/src/crash_library.rs:572` 「.max(FEWEST_INNER_THREADS_OF_THE_SHARE),」；临时服务那一份 `crates/singlefs-verification/src/crash_library.rs:567` 「let temporary_servers = (heavy_threads / 2 / THREADS_OF_A_TEMPORARY_SERVER).max(1);」，一份临时服务记的线程数 `crates/singlefs-verification/src/crash_library.rs:555` 「3 + prune_server::server::WORKER_THREADS_OF_A_SERVICE_IN_THIS_PROCESS + 2;」（= 3 + 2 + 2 = 7，常量在 `prune-server/src/server.rs` 第 1038 行）。
- 句三「先取名额、后拿锁，反着拿在新占名额那一刻 panic（常量 `PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK`，第 676 行）」：对得上。常量 `crates/singlefs-verification/src/crash_library.rs:676` 「const PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK: &str =」；panic 在 `crates/singlefs-verification/src/crash_library.rs:703` 「panic!("{PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK}");」，条件 `crates/singlefs-verification/src/crash_library.rs:700` 「if crate::crash_amplification::this_thread_holds_the_environment_lock() {」，取名额的入口 `crates/singlefs-verification/src/crash_library.rs:756` 「let place_in_the_test_process = TemporaryServerPlaceInTheTestProcess::take();」。
- 正文没写、代码里有的三处限定（不是不一致，是正文说得比代码宽）：
  1. 三样都在 `#[cfg(test)]` 里（单测构建），产品构建里 `InnerThreadsTaken::take` 照要的数给（`crates/singlefs-verification/src/crash_library.rs:628` 「pub(crate) fn take(wanted: usize) -> Self {」那一份没有额度）。
  2. panic 只在「这条线程还没起着一份、要新占名额」时发；这条线程已经起着一份的，拿着锁再开不查、不等（`crates/singlefs-verification/src/crash_library.rs:686` 「fn take() -> Self {」里第 692–699 行那一支）。锁一侧（先拿锁、后取名额以外的方向）没有检查。
  3. 份额小于 9（= 一份临时服务 7 条加下限 2）时两份合起来超过份额：代码自己的文档注释写了（`crates/singlefs-verification/src/crash_library.rs:542`–543）。按公式算（推的，源码公式，没跑）：每台默认重活线程数 31 时临时服务 2 份、里层 17 条；份额 6 时临时服务 1 份（7 条）、里层 2 条，合 9 > 6。

### 第 8 条（两种判器同一种结论、只排计划那条用例三趟）
- 句一「`JudgedBy::conclusion_kind` 对 `GpuJudge` 与 `CpuJudge` 都交 `ConclusionKind::ImageJudgement`」：对得上。`crates/singlefs-verification/src/crash_amplification.rs:26444` 「pub fn conclusion_kind(self) -> ConclusionKind {」，`crates/singlefs-verification/src/crash_amplification.rs:26446` 「Self::GpuJudge | Self::CpuJudge => ConclusionKind::ImageJudgement,」。
- 句二「流水线测试里只排计划那条用例照这一点改成三趟（开 GPU 只排计划、空库；单机 CPU 判完；再开 GPU 只排计划）」：对得上。用例 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1386` 「fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {」，第一趟 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1415` 「placement_only_on_the_gpu_configuration();」，判之前那趟的断言在 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1427` 「placement_run_before_judging.states_reused, 0,」，第二趟 CPU `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1418` 「let (run_on_the_cpu, _) = amplify(&recorded, "placement-reuse");」，第三趟 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1420` 「let (placement_run, placement_line) = placement_only_on_the_gpu_configuration();」，其后断言复用数等于整条流的态数 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1450` 「placement_run.states_reused, state_count,」。「单机 CPU」靠用例目录里写的空配置（`EnvironmentRunDirectory::new` 写 `std::fs::write(&configuration, "")`）。用例带 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1384` 「#[cfg(feature = "gpu")]」，不带特性编的二进制里不跑它。
- 句三尾巴「不起卡池」：**对不上这一条用例**。这条用例只断言卡的来路是不要卡：`crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1457` 「"只排计划的那一趟只数要判的块与状态，不取卡"」；整份文件里没有 `card_pool_started_in_this_process` 的任何调用（`grep -n card_pool_started_in_this_process` 这份文件零命中，现跑）。「这个进程的卡池没起」是另一个新用例钉的：`crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:129` 「card_pool_started_in_this_process().is_none(),」。代码这一侧（只排计划那一趟不取卡）成立（问题 1 的 1b），只是正文把「不起卡池」记在了流水线那条用例上。
推翻 4 的现象：照 1b 的命令跑那条新用例，`card_pool_started_in_this_process()` 若不是 `None`，则「不起卡池」在代码一侧也不成立。

## 问题 5：正文末尾八格检查单

### 格一「走不到的理由二次查实」：成立
被判的是 `match *tables {}`。`crates/singlefs-verification/src/crash_amplification.rs:34215` 「match *tables {}」在 `crates/singlefs-verification/src/crash_amplification.rs:34213` 「#[cfg(not(feature = "gpu"))]」下，类型是 `crates/singlefs-verification/src/crash_amplification.rs:7310` 「pub enum PackedJudgeTablesOfTheRun {}」（没有成员的枚举，上一行 7308 也是 `not(feature = "gpu")`）。不带 `gpu` 特性时造这个值的路：
- 安全代码造不出空枚举的值；`crates/singlefs-verification/src/lib.rs:18` 「#![deny(unsafe_code)]」，`grep -rn 'allow(unsafe_code)' src tests` 零命中，`grep -c unsafe src/crash_amplification.rs` 打 0，所以没有 `unsafe` 的口子。
- 这个类型只出现在 `crash_amplification.rs` 里（`grep -rn PackedJudgeTablesOfTheRun src tests` 去掉这份文件之后零命中）；唯一给它值的函数不带特性时只会报错：`crates/singlefs-verification/src/crash_amplification.rs:7361` 「Err("这个二进制没编进 gpu 特性，读不了 GPU 判表".to_string())」；选路在要 GPU 而没编进时先报错：`crates/singlefs-verification/src/crash_amplification.rs:1470` 「if !gpu_feature_is_compiled_in {」。所以 `Option<PackedJudgeTablesOfTheRun>` 在不带特性时只能是 `None`，`match *tables {}` 走不到；它不是断言，是对空类型的穷举。
限定：我核的是这个 crate 的 `src/` 与 `tests/`；其他 crate 不能造它（类型在这个 crate 里，造它要 `unsafe`）。推翻它的现象：`grep -rn unsafe` 在这个类型的使用范围里出现命中。

### 格二「类型与名字」我这一半（块号、态序号、入池序号各用各的类型）：不成立（三样共用裸 `u64`）
- 块号：`crates/singlefs-verification/src/verdict_store.rs:4697` 「block_number: u64,」（`put_the_facts_verifier_pending_row_unless_present` 的参数）、`crates/singlefs-verification/src/verdict_store.rs:4823` 「block_number: u64,」（编帧函数）、`crates/singlefs-verification/src/crash_amplification.rs:34343` 「let mut block_number = 0u64;」（任务一侧编号）。
- 入池序号：`crates/singlefs-verification/src/verdict_store.rs:362` 「pub pool_entry_number: u64,」，产生处 `crates/singlefs-verification/src/crash_amplification.rs:33419` 「fn pool_entry_number_now() -> u64 {」。
- 态序号：`crates/singlefs-verification/src/verdict_store.rs:358` 「pub state_ordinals: std::ops::Range<u64>,」，造它的地方 `crates/singlefs-verification/src/crash_amplification.rs:34365` 「state_ordinals: point.ordinals.start + local.start」。
三个量在签名里都是裸 `u64`（态序号是 `Range<u64>`），互换位置编译器不报。判器那一族（0x1D、0x1E）原来就是这样，这一批照写。对照：同一个文件里块起点有专门的类型 `crates/singlefs-verification/src/verdict_store.rs:747` 「pub block_start: BlockStartStateIndex,」，所以这个文件里有「给序号起类型」的先例，新写的三样没起。`_ =>` 零处我自己跑了：`git diff HEAD -- crates/singlefs-verification/src | grep '^+' | grep -c '_ =>'` 打 `0`。命名检查（任务 19323、19324）我没读，不核。
推翻它的现象：这三处签名里出现各自的类型（块号、态序号、入池序号各一个），或实现里它们各有包装；读到那样的签名就推翻。

### 格三「不兼容旧代码与旧格式」：不成立（Rust 侧成立，脚本自证桩里还留着旧样子）
- Rust 侧：PLACEMENT 那一行没有 `host=`、`cards=` 恒空：`crates/singlefs-verification/src/crash_amplification.rs:34200` 「"{PLACEMENT_LINE_PREFIX} judge=gpu pending_blocks={} pending_states={pending_states} states_reused={states_reused} tables_mebibytes={} cards= states_placed_on_this_host=0 states_placed_on_other_hosts=0 states_per_second_on_this_host=0 estimated_seconds_on_this_host=0",」。删掉的四个名字 `verify_my_blocks_on_the_first_card`、`print_the_placement`、`PlacementOfThisRun`、`BufferLeaseOnTheCardPool`：`grep -rn` 在 `crates/`、`scripts/`、`.claude/gate.d/`、`.claude/hooks/` 里零命中（现跑）。表 0x1F 只认一个格式号（`crates/singlefs-verification/src/verdict_store.rs:366` 「const FACTS_VERIFIER_PENDING_ROW_FORMAT_NUMBER: u8 = 1;」，读的一侧 398 起只比这一个），没有认旧布局的分支。
- 分项驱动认不认新样子：认。它按字段名取：`scripts/crash/crash-amplification-lib.sh:690` 「fields = dict(re.findall(r"(\w+)=([0-9]+)(?= |$)", line.strip()))」，估时合计为 0 时速度打 0（`scripts/crash/crash-amplification-lib.sh:694` 「print(pending // seconds if seconds else 0)」），不读 `host=`、`cards=`；驱动里 `scripts/crash/crash-amplification-item.sh:244` 「pending="$(crash_amplification_placement_total "$placement_log" pending_states)"」、`scripts/crash/crash-amplification-item.sh:274` 「local_speed="$(crash_amplification_placement_rate "$placement_log")"」只用 `pending_states` 与估时。
- 残留：驱动自证里的假二进制仍打旧样子的行：`scripts/crash/crash-amplification-item.sh:332` 「CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=$1」、`scripts/crash/crash-amplification-item.sh:446` 「CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=${STUB_PLACEMENT_PENDING:-1000}」（439、441 行同），入口自证里 `scripts/crash/crash-amplification.sh:997` 「CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=6 pending_states=60」（1000、1001 行同）。它们带 `host=`、非空 `cards=`、非 0 的装卡态数，Rust 今天不会打这样的行；这些是造旧形态输入的测试数据（共用约束「里程碑 17 收尾之前不兼容任何旧东西」那一条），不在这三批 13 份文件里，写进报告交主 agent。第 1f 条的死分支（`crates/singlefs-verification/src/crash_amplification.rs:4761`）也是删 `verify_my_blocks_on_the_first_card` 时留下的壳，同样交主 agent。
推翻它的现象：那几处假行被主 agent 判为不是造旧形态输入的测试数据（只是借来演示字段名的占位），而 Rust 今天打的样子里确有 host= 或非空 cards=。

### 格四「不新建判定库」：成立
- 核对待判行是 `verdict_store.rs` 里的一张表，读写经 `VerdictStore`：写 `crates/singlefs-verification/src/verdict_store.rs:4703` 「match self.service.claim(&key.0, "认领核对待判行")? {」（经服务认领、写帧），读 `crates/singlefs-verification/src/verdict_store.rs:4742` 「pub fn read_facts_verifier_pending_rows(」；一次性库不收：`crates/singlefs-verification/src/crash_amplification.rs:34521` 「if library.take().is_a_disposable_library() {」。
- 三批的 `src/` diff 里新加的行里找新开库、新目录、sqlite：`git diff HEAD -- crates/singlefs-verification/src | grep '^+' | grep -nE 'DB::open|rocksdb::|sqlite|create_dir|File::create|fs::write|OpenOptions'` 只命中一行：测试里写多机配置文件的 `std::fs::write(&configuration, "ENABLE_GPU=1\n")`，不是判定库。
- 读库程序只读：唯一开库 `crates/singlefs-verification/src/bin/crash_library_reader.rs:706` 「let library = match LibraryOpenInThisProcess::open_or_share() {」；`grep -nE '\.store_|\.put_|\.delete_|write_batch|\.claim\(' src/bin/crash_library_reader.rs` 零命中（现跑）。
- 新测试开的是 `DisposableLibrary`（临时 prune-server 里的一段名字空间）：`crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:50` 「let _the_library_of_this_process = directory.make_it_the_library_of_this_process();」。
限定：我 grep 的是这几份 `src/` 文件的 diff 与读库程序；没扫 `scripts/` 与别的包。

### 格五「所有任务多机可用」：不成立（两处与正文说的不一样）
- 「任务进程不读本机有没有卡、不照多机配置自己挑卡」：第 ①②段与只排计划成立；**观察者交回的盘那一支不成立**：它读多机配置（`crates/singlefs-verification/src/crash_amplification.rs:33254` 「let configuration = multi_host_configuration_of_this_process()?;」）、起卡池时枚举本机 Vulkan 适配器与 PCI 总线（`crates/singlefs-verification/src/gpu_unit_checks.rs:3119` 「&enumerate_vulkan_adapters(),」）、照显卡配置给这台机器点名挑卡（卡池那一步），两台上这一支的结局取决于本机有没有卡。链见 1d。
- 「等服务那一段问的是判定库里服务的在跑标记，两台同一份库」：**对不上**。代码问的是控制命令：`crates/singlefs-verification/src/crash_amplification.rs:33804` 「Command::new("python3")」加 `crates/singlefs-verification/src/crash_amplification.rs:33806` 「.arg("check")」，控制命令路径取自多机配置所在目录（`crates/singlefs-verification/src/crash_amplification.rs:33791` 「fn gpu_verification_server_control_command(configuration_file: &Path) -> PathBuf {」）。`gpu-verification-server/control` 的 `check` 读的是本机的 `systemctl --user`、状态目录里的 serving 文件与 `journalctl`，点名了卡的第二台经 ssh 转去查（该文件头「用法」一节：「`check`」与「在第二台做的，经 ssh 直连第二台的地址」两段），不读判定库里的任何标记。两台都能跑（命令走共享根同一绝对路径、第二台经 ssh），所以多机上不是缺陷，只是正文这句说错了机制。
- 「新测试 `tests/crash_amplification_placement_only_run_takes_no_card.rs` 与改过的流水线用例用用例自己写的多机配置（只写 `ENABLE_GPU=1`），不要本机有卡」：成立。新测试 `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:49` 「std::fs::write(&configuration, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");」，流水线用例 `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1399` 「std::fs::write(&configuration_with_the_gpu, "ENABLE_GPU=1\n").expect("写得了开 GPU 的配置");」；两处的第 ①②段、只排计划都不进 1d 那条路，所以不要卡。
- 主机名与地址：`grep -n "192\.168\|jplife\|faliye" crash_amplification.rs verdict_store.rs` 零命中（现跑，退出码 1）。
推翻它的现象：把观察者那一支改成不取卡后，在没有 NVIDIA 卡的那台机器上这条链的用例结局与有卡的一台相同。

### 格六「测试优先多线程」：成立，一处推不出
- 里层并行拿线程经 `InnerThreadsTaken::take`：`crates/singlefs-verification/src/crash_amplification.rs:31397` 「let _threads_taken = crate::crash_library::InnerThreadsTaken::take(」（观察者交回的盘走 CPU 判那一支，`#[cfg(test)]` 下）。
- 等服务的那一段是单线程轮询：`crates/singlefs-verification/src/crash_amplification.rs:34399` 「while !blocks_not_verified_yet.is_empty() {」里睡 `crates/singlefs-verification/src/crash_amplification.rs:34540` 「std::thread::sleep(GPU_VERIFICATION_SERVER_RECKONING_INTERVAL);」，等的是别的进程，没有开并行。
- 新测试没有按参数循环重建池：新测试整份 132 行只有一个 `#[test]`，没有循环；流水线用例里唯一的 `for` 只遍历两个结果做断言。
- 线程数没有写死的新处：`git diff HEAD -- crates/singlefs-verification | grep -nE '^\+.*(spawn|thread::scope|\.threads\(|num_threads|available_parallelism|RUST_TEST_THREADS|test-threads|rayon|ThreadPool|with_threads)'` 零命中（现跑）。
- 推不出：份额 6 下实跑。正文说「改后只靠新单测的算术钉着，没在份额 6 下实跑」，我同样没跑，算术见问题 4（份额 6 时里层 2 条、一份临时服务 7 条）。待主 agent 跑（命令见末尾）。
推翻它的现象：份额 6 的一片里那两条「一块的各核对线程共用一份枚举表」单测仍红。

### 格七「带 --item 的命令与项」：成立（没有新登记）；变异行条数推不出
- 这三批没有新登记的崩溃枚举用例与放量分项：本轮新加与改写的 5 份测试文件名（`crash_amplification_placement_only_run_takes_no_card`、`crash_amplification_pipeline_records_checks_and_reuses`、`crash_amplification_run_of_both_stages_records_the_flow_before_it_leases_a_card`、`crash_amplification_observer_images_judged_on_the_gpu_name_the_card`、`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start`）在 `.claude/gate.d/crash-point-replay-items.tsv` 与 `.claude/gate.d/stage-inputs.tsv` 里 `grep -n` 零命中（现跑）。限定：只查了这两张表。
- 44 行变异行带 `--item` 判过（任务 19325）：我没读那个任务，也没法在不跑的情况下核；我只量到 `git diff HEAD -- crates/mutations.tsv` 里新增 61 行、删 7 行（含别的批改的行，不能当那 44 的对应）。**推不出**，待主 agent 对着任务 19325 的计数行核（命令里的计数行就是它报的那一句）。
推翻它的现象：门禁登记表里查到这五份文件之一的 `crash-case:` 行或放量分项。

### 格八「新崩溃流多罩了什么」：成立
新测试 `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:40` 「let flow = common_crash_points::new_pool_file_creation_as_one_crash_point(」用的是共用层已有的小流 `crates/singlefs-verification/tests/common_crash_points/mod.rs:865` 「pub fn new_pool_file_creation_as_one_crash_point(」（`git status` 里 `tests/common_crash_points/` 没有改动）；流名 `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs:22` 「const FLOW_NAME: &str = "line-fields::placement-only-takes-no-card";」落在来历表已有的 `line-fields` 条上（`crates/singlefs-verification/src/crash_point_replay_provenance.rs:688` 「"line-fields",」）。来历表的 diff 只删了一条（对应被删的测试那条流），没有加：`git diff HEAD -- crates/singlefs-verification/src/crash_point_replay_provenance.rs` 只有 5 行删、零行加（现跑）。
推翻它的现象：来历表或崩溃点登记里出现新流名。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 问题 1 条款「卡只归 GPU 服务，任务不拿卡」：崩溃放量一趟的两段 | 成立 | 第 ① 段写待判行、第 ② 段写核对待判行，进函数前只判一次性库与服务在不在，没有取卡调用 |
| 问题 1 只排派活计划那一趟 | 成立 | 24564 分支只数块与状态、交回，卡的来路写死不要卡 |
| 问题 1 第 ② 段核对 | 成立 | 调用点只有一个，函数里没有取卡 |
| 问题 1 观察者交回的盘 | 不成立 | 头一张要判的盘时读多机配置、起卡池、借显存、在卡上判；任务里多半在取卡那一步退 78，但环境有卡串或通道就真开卡 |
| 问题 1 条款之外另三条路 | 不成立 | 崩溃枚举用例的观察者盘、给定状态表引擎、测试直接调对拍入口；另有第 ① 段按卡池借草稿区的死分支 |
| 问题 2 键 | 成立 | 0x1F、流键 32 字节、块号 8 字节大端；与任务一侧「不在才写」与读回记账是同一把键 |
| 问题 2 值 | 成立 | 七栏齐全，读的一侧只认一个格式号，少的三栏是判器那一边独有的 |
| 问题 2 两个守卫 | 成立（形状） | 核对行守卫防盖掉别人刚写的、待判行守卫防删掉补写的；服务一侧没有代码在核 |
| 问题 2 原子写回、不重判不漏判 | 推不出 | 缺：服务一侧读写 0x1F 的代码（零命中）；帧里无认领键与持有者，同一行可被两个服务线程各核一遍（结果对、白核） |
| 问题 3 服务不在 | 成立 | 33862 / 34524 先问，退 78，打 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_NOT_RUNNING` 行 |
| 问题 3 服务中途退出 | 成立 | 每 15 圈（约 30 秒）问一次，退 78，打同一行；控制命令退任何非 0 都走这里 |
| 问题 3 核对行被别的进程先写 | 成立 | 写前在就记已核；写后在就记核完；不退出，除非等不到认领或服务不在 |
| 问题 3 一次性库 | 成立 | 写任何行之前交回错误串，不退出；② 里这一支产品路径走不到（① 先死） |
| 问题 3 服务在跑而不扫 0x1F | 推的 | 这一段不返回、没有汇总行与退出码，挂到被任务服务判卡住 |
| 问题 4 第 6 条 | 成立 | 三句都对得上；正文没写三处限定（仅单测构建、只对新占名额的线程、份额小于 9 两份合起来超份额） |
| 问题 4 第 8 条 | 对得上两句，不对一句 | 「不起卡池」这个断言在另一个新用例里，不在流水线那条用例里 |
| 问题 5 格一 走不到的理由 | 成立 | 空枚举，无 `unsafe`，不带特性时只能是 `None` |
| 问题 5 格二 类型与名字（我这一半） | 不成立 | 块号、入池序号、态序号共用裸 `u64`；`_ =>` 零处 |
| 问题 5 格三 不兼容旧 | 不成立 | Rust 与驱动解析成立；驱动与入口自证里的假行仍是旧样子；另有死分支 |
| 问题 5 格四 不新建判定库 | 成立 | 经 `VerdictStore`，一次性库不收，读库程序只读 |
| 问题 5 格五 多机可用 | 不成立 | 观察者那一支读本机卡；「问判定库里在跑标记」说错了，实际是控制命令 `check` |
| 问题 5 格六 测试优先多线程 | 成立，一处推不出 | 份额 6 下实跑待主 agent 读 |
| 问题 5 格七 带 --item | 成立，一处推不出 | 没有新登记；44 行变异行的计数我没核 |
| 问题 5 格八 新崩溃流 | 成立 | 用共用层已有的小流与来历表已有的条 |

## 要主 agent 跑的命令

我一条都没起。下面的命令（推的：测试名从源码里读出，没跑过）交主 agent 经任务服务跑；`--cwd` 写仓根是因为这些是 `cargo test`，需要整仓，我没有建副本。

1. 只排派活计划不取卡（问题 1、4；这个目标走崩溃放量引擎，先照不会走以外的写法写缩域）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_amplification_placement_only_run_takes_no_card 里的 1 条测试; 范围=1 个测试; 为什么=核只排派活计划那一趟不取卡、不起这个进程的卡池' task-server/task submit --cwd /home/fy5090/code/singlefs -- env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=2 cargo test -p singlefs-verification --release --features gpu --test crash_amplification_placement_only_run_takes_no_card -- a_placement_only_run_with_the_gpu_configured_and_no_card_handed_out_prints_its_line_without_a_card --exact`
   在仓根起；看 `TASK_SERVER_RESULT` 行 `type=` 与 `exit=0`，`task-server/task log <号>` 里 `test result: ok. 1 passed`。被范围声明闸拒了就照它的说明在缩域与全域理由里二选一改。
2. 核对待判行的键、值、守卫、等服务那一圈（问题 2、3）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的 --lib 里的 6 条测试; 范围=6 个测试; 为什么=核核对待判行的不在才写、帧的两个守卫、替身服务写回读回、等的时候叫停、组键全零与态区间对不上的拒绝' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --lib -- verdict_store::tests::a_facts_verifier_pending_row_is_put_only_when_it_is_not_there verdict_store::tests::a_facts_verifier_frame_writes_no_verifier_row_already_stored_identically crash_amplification::tests::the_facts_verifier_pending_rows_verified_by_a_stand_in_service_read_back_as_verified_in_this_process crash_amplification::tests::a_stop_while_waiting_for_the_facts_verifier_leaves_the_facts_verifier_pending_rows_and_returns crash_amplification::tests::facts_verifier_pending_rows_are_not_written_under_a_verifier_key_of_zeros crash_amplification::tests::the_material_of_a_facts_verifier_pending_row_whose_states_are_not_its_block_is_refused --exact`
   在仓根起；看 `TASK_SERVER_RESULT` 的 `exit=0` 与 `6 passed`。
3. 1d 的现象核（推翻 1 的那一条）我没有现成的命令：需要主 agent 另派实现员写一条在带 `gpu` 特性、环境设 `GPU_CARDS_BY_PRIORITY`、挂观察者的用例里看 `card_pool_started_in_this_process()`；本轮不写（写它要卡）。标「复核不了」。
4. 份额 6 下实跑（问题 5 格六）：线程份额由服务按片分，调用方不能写 `SINGLEFS_THREAD_CAP`，我写不出单条命令；标「复核不了」，等主 agent 在合入后验证里把验证包库测试与别的目标并在一片里跑到份额 6 时读那两条「一块的各核对线程共用一份枚举表」单测的结局。
5. 命名检查与变异行计数（问题 5 格二、格七的任务 19323、19324、19325）：主 agent 已交，我没读它们的输出；不在我这一腿。

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 没写模型、没建仓副本、没起任何样本进程；所以没有模型目录、没有 `rerun.sh`、没有 sha256 清单。草稿目录只有只读的找行号脚本与清单（`/home/fy5090/.local/share/singlefs/work/amplification-without-cards-code-r1-forward/`），不进仓。
- 反推的格（同一块被两个任务进程同时写核对行、复用数数没数错、里层份额下限在份额小于 9 时会不会互相等死、锁次序 panic 有没有正常路径会撞上）是攻方腿的分工，我只在问题 2、4 里顺带读到的写了，没有为它们另查。
- 读库程序的第 7 条（`keys` 多打三种行）、29 条红名单、观察者结论拼块的版本口径，不在这一腿的五个问题里，没读。
- 问题 1 只查了任务进程里取卡的路，没查 GPU 服务一侧该怎么接 0x1F 与观察者盘；2e 的缺口一、二只写了缺什么，没替服务写。
- 引文核对：每一处引文写进报告前我在被引文件里按行找过原文（`find_lines2.py` 逐条对，命中 0 次的没写成引文）；交回前跑 `scripts/agents/cite-check.py`，末行见交回正文。
- 另有一行记给主 agent：`.claude/gate.d/path-moves.tsv` 第 89 行点名了被删掉的测试文件作新前缀，我没查那一行在门禁里有没有后果，请核。
