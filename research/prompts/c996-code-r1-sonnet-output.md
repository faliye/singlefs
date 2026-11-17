# c996-code-r1 正推腿报告（2026-11-12）

立场：从 D26（后台整理与放置回收）已定项 13 第 2 条末句与第 4 条推「代码应该做什么」，再与 `crates/` 今天的实现逐格比。一样写「对上」，不一样把两边整行并排抄。
范围：只判 `research/prompts/_c996-code-r1-diff.md` 里的块与它们和周围代码的交叉；行号是主工作区此刻的（`grep -n` 现取）；kb 行号是 kb 文件自己的。
没跑任何项目代码；下面「读出来」的都是读码推的，要主 agent 跑的在末尾「要主 agent 跑的命令」一节。
被判条款的两段（整句抄，行号去 kb 文件现取）：第 2 条末句在 `.claude/kb/decisions/26-后台整理与放置回收.md:251`，第 4 条在 `.claude/kb/decisions/26-后台整理与放置回收.md:253`。

## 一、攻击面 1：逐句对照（题面第一问）

判的写法：「对上」「对不上」「代码里没有」「条款没写」。

| # | 条款句（kb 行） | 实现（文件:行） | 判 |
|---|---|---|---|
| 1 | 「挂载时只开参与的卡（点名了是点名的那几张，没点名是本机全部独立显卡），不参与的卡不开」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`） | 参与与否按标识找：`participating_gpu` 没找到就记进没开的清单、`continue`，不开：「not_opened_for_not_taking_part.push(discrete_gpu);」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:875`）；产品挂载把参与的卡交进去：「let gpus_opened = gpus_opened_at_the_mount(」（`crates/singlefs-background-gpu/src/background_gpu.rs:991`） | 对上 |
| 2 | 同句括号「没点名是本机全部独立显卡」，与同条前文「集成显卡、虚拟显卡、软件 Vulkan 适配器与认不出类型的适配器不算「显卡」、不进「全部」」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`） | 没点名取清单全部：「None => discrete_gpus.to_vec(),」（`crates/singlefs-background-gpu/src/adapter_inventory.rs:296`）；清单里只有独显：「AdapterDeviceKind::DiscreteGpu => true,」（`crates/singlefs-background-gpu/src/adapter_inventory.rs:44`），另四类 45-48 行都是 false | 对上 |
| 3 | 「挂着时切换点到还没开的卡，那一刻再开那张卡（开法与开不成的处理在第 4 条）」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`） | 点到的没开的卡（按点的次序）：「let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(settings);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2223`），非空就记下这一组并起开卡线程（同文件 2229-2231）。「那一刻」落在交设置的那一次调用，线程里开，调用即返回 | 对上 |
| 4 | 「切换之后不再参与的卡，切换生效时关掉」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`） | gpu 到 gpu：换上一组时先关没点名的：「self.close_the_cards(|device| !devices_named_by_the_settings.contains(device));」（`crates/singlefs-background-gpu/src/background_gpu.rs:2445`）。gpu 到 cpu：核心层只在这一组带 gpu 设置时才把设置交给实现：「let Some(gpu_settings) = &self.gpu_settings else {」（`crates/singlefs-core/src/background_compute.rs:822`）后面就 `return`，在 `crates/singlefs-core/src`、`crates/singlefs-mount-runtime/src`、`crates/singlefs-background-gpu/src` 里 `grep` 实现的 `use_the_settings_from_the_next_batch(` 调用，非测试、非定义的核心层调用只有 `crates/singlefs-core/src/background_compute.rs:829` 这一处 | gpu 到 gpu：对上。gpu 到 cpu：开着的卡一张不关；条款「不再参与」按字面读 cpu 也是不再参与，条款没明说 cpu 算不算，**读法待主 agent 定**；按字面读是对不上 |
| 5 | 「挂载那一刻点名的卡开不成，或挂着时切换点到的卡开不成，都间隔 1 分钟重开，最多重开 3 次。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 常量：「pub const INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED: Duration = Duration::from_secs(60);」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:319`）、「pub const REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST: u32 = 3;」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:322`）；循环头一次不等，之后每次先等间隔再开：「for reopenings_so_far in 0..=reopenings.reopenings_at_most {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:359`）、「wait_before_a_reopening(reopenings.interval);」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:361`）。挂载与切换用同一个函数：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:880`、`crates/singlefs-background-gpu/src/background_gpu.rs:1119` | 对上。间隔从上一次失败结束起算，不是两次开始之间隔 1 分钟；条款没写哪一种 |
| 6 | 「挂着时切换点到还没开的卡：切换请求照收，在会话锁外开新卡；」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 开卡线程：「.name("singlefs-opening-gpus-at-a-switch".to_string())」（`crates/singlefs-background-gpu/src/background_gpu.rs:2354`）；线程只拿自己那一把锁：「let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(」（`crates/singlefs-background-gpu/src/background_gpu.rs:2356`），不碰核心层借出实现那一把（`crates/singlefs-core/src/background_compute.rs:467`） | 对上 |
| 7 | 「重开期间照旧用切换前那组卡，开成了，下一批起换成新设置；」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`），加第 1 条「一批之内不换」（`.claude/kb/decisions/26-后台整理与放置回收.md:250`） | 开着的这段时间 `cards_on_this_host` 与 `participating` 一个字都不动：交设置只记下这一组（`crates/singlefs-background-gpu/src/background_gpu.rs:2229` 「self.settings_waiting_for_the_opening = Some(settings.clone());」）；换上只在 `put_the_gpus_opened_since_the_switch_in_effect`（`crates/singlefs-background-gpu/src/background_gpu.rs:2243`）。核心层调它的只有三处，都在两批之间：「self.put_the_switch_requested_from_the_next_batch_in_effect();」（`crates/singlefs-core/src/background_compute.rs:1647`，一批开头）、`crates/singlefs-core/src/mounted_session.rs:3416`、`crates/singlefs-core/src/mounted_session.rs:4513`（整理两个入口）；一批里实现一直被借着：「借出实现：一批里三段都经这一次借出算。」（`crates/singlefs-core/src/background_compute.rs:463`） | gpu 到 gpu：对上，一批之内不换也守住。cpu 到 gpu：见攻击面 2 的 ⑤，「切换前那组卡」条款没写是哪一组 |
| 8 | 「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 开卡线程收齐 1+3 个原因交回，两批之间收下：「if settings_of_the_opening == settings_waiting {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2302`）、记下并只交这一次「self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened.clone());」（`crates/singlefs-background-gpu/src/background_gpu.rs:2304`）。核心层收到就停：「self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));」（`crates/singlefs-core/src/background_compute.rs:1620`）。报警点名卡：「| BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { device, .. }」（`crates/singlefs-core/src/background_compute.rs:231`）；每次的原因在 `failures_of_each_opening`（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:380`）。不自动再试：失败分支 `return Some(not_opened);`（`crates/singlefs-background-gpu/src/background_gpu.rs:2305`）之前已经 `take()` 掉在等的那一组（同文件 2297），没有再起线程的路 | 对上。两处小差：（甲）开卡超时的那一次报成「等这一批的提交超过上限」：「submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,」（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:295`），其实是开卡到点，不是一批的提交，原因的标签含糊，条款只要求报原因；（乙）旧的失败结局被新一轮切换当成这一轮的，见攻击面 2 的 ② |
| 9 | 「挂载那一刻点名的卡重开 3 次都不成，才拒绝挂载。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）；同条后文「没点名的卡不用，参与的卡一张不剩时挂载照样不拒、后台任务进出错停着（第 3 条）」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 任何一张参与的卡重开到上限都没开成，一律拒：「BackgroundGpuOpenRefused::GpuUnusable {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:890`）。没点名时参与的卡就是全部独显（上面第 2 行）。类型文档照这个读：「给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。」（`crates/singlefs-background-gpu/src/background_gpu.rs:198`）；挂载参数表同读：「挂载时都不成就拒绝挂载」（`.claude/kb/mount-parameters.md:23`） | 点名的卡：对上。没点名（默认，全部独显参与）：D26（后台整理与放置回收）已定项 13 第 4 条字面有两句互相拉扯——「点名的卡……才拒绝挂载」加「没点名的卡不用……挂载照样不拒」读成默认模式下开不成的卡不用、不拒；挂载参数表第 23 行与代码读成一律拒。代码取后一种，且用户定案删掉了「挂载那一刻没开成的卡」一路，正是前一种读法的载体。判：按 D26 字面**对不上**，条款文本要主 agent 定改哪一边 |
| 10 | 「挂载时开卡（含编管线）等的上限是 5 分钟，到点当那张卡这一次没开成」「挂载等开卡成功或到点才往下走，不做回调」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 常量「pub const WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT: Duration = Duration::from_secs(5 * 60);」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:276`），每一次开卡都套它（`crates/singlefs-background-gpu/src/background_gpu.rs:1092`）；调用线程一直收到结局或到点才返回：「Ok(ReportOfTheOpeningThread::Outcome(outcome)) => return outcome,」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:340`） | 对上 |
| 11 | 「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`） | 见攻击面 3 | 见攻击面 3 |
| 12 | 实现员设计问题 ①「最多重开 3 次」按头一次之后再开 3 次、共 4 次、报 4 个原因 | 条款里「重开 3 次都不成」「点名的卡间隔 1 分钟重开、最多重开 3 次，都不成才拒绝挂载」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）；「报出哪张卡、每次开不成的原因」里「每次」含头一次。实现：「for reopenings_so_far in 0..=reopenings.reopenings_at_most {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:359`）；测试钉了 4 次：「assert_eq!(openings, 4, "头一次加 3 次重开");」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:1094`） | 读对。「重开」是头一次开不成之后的再开，三次再开加头一次是 4 次；3 次总共的读法在条款字面里找不到支撑。推翻它的现象：用户说明「3 次」含头一次 |

### 攻击面 1 的三条补充

- 没点名时「本机全部独立显卡」的口径：挂载（上面第 2 行）与切换（核心层用卡清单校验，清单是开着的加没开的，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:343` 起的 `devices_with_total_memory` 两样合起来）一致；对上。
- 切换在会话锁外开、重开期间用切换前那组卡、开成了下一批起换上：见第 6、7 行。有一处读出来的偏差在 ⑤（cpu 到 gpu）。
- 「重开 3 次都不成 GC 停下」之后旧卡留不留：条款没写。代码在失败分支里不关旧卡（`crates/singlefs-background-gpu/src/background_gpu.rs:2301-2306` 只关这一次新开的），所以停着期间不在新设置里的旧卡仍开着；第 4 行「切换生效时关掉」在这里没有触发，因为切换没生效。

## 二、攻击面 2：实现员自己定的那几处（设计问题 ③④⑤⑥⑨）与交错

### ③ 开卡线程起不了

- 实现：起线程失败记成 `ThreadNotStarted`，原因只有一个，卡取点名列表里头一张：「Err(error) => OpeningAtASwitch::ThreadNotStarted {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2376`）、「causes_of_each_opening: vec![GpuFailureCause::DeviceOrPipelineDidNotStart {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2380`）；下一次两批之间当结局收下（同文件 2248-2257），同样走 2301-2306 停 GC。
- 条款：「切换点到的卡开不成，都间隔 1 分钟重开」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）说的是卡开不成；起不了线程是宿主资源的事，条款没写。
- 与自家类型文档不一致：「`causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。」（`crates/singlefs-background-gpu/src/background_gpu.rs:176`）而这一支只有 1 个，也没重开；报的那张卡可能一次都没试过。挂载一侧同样的起线程失败走的是每一次开卡里面那条线程：「.map_err(|error| GpuFailureCause::DeviceOrPipelineDidNotStart {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:300`），是进重开循环的，会重开 3 次；挂载与切换在这一格上口径不同。
- 判：条款没写；代码与自己的类型文档不一致（轻）。读出来的，没跑。

### ④ 开卡开到一半又来一次切换、再来一次

- 实现：开着时 `use_the_settings_from_the_next_batch` 只替换在等的那一组：「if self.opening_at_a_switch.is_none() {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2230`）不成立就不起新线程；收结局时按最近那一组再算要开哪些（`crates/singlefs-background-gpu/src/background_gpu.rs:2308-2314`）。
- 逐种交错读下来：（甲）旧一轮开成：新开的卡并进开着的卡，新一组要的都开着就当场换上（2310），还缺就为它再起一条（2313）；（乙）旧一轮开不成而最近一组与旧一轮的不同：旧的失败不报（2302 不成立，落到 2308），按最近那一组重新开，那张卡又是 1+3 次；（丙）第三次切换同理，每次覆盖在等的一组。没找到分错活、在关掉的卡上提交、停着的被放开的路：换上与关卡都只在两批之间、都要先过 `discrete_gpus_named_and_not_opened` 那一判（2308），换上之前 `participating` 不变。
- 条款没写重叠的切换怎么数「3 次」。实现的读法是「按最近一组数，每一轮重新 1+3」。推翻它的现象：用户说明同一张卡的重开次数跨切换累计。
- 判：条款没写；实现自洽。

### ⑤ 从 cpu 切回 gpu，点到没开的卡

- 开着的这段时间核心层 `in_effect` 已是新那一组（`crates/singlefs-core/src/background_compute.rs:1610`），路线按它定成走 GPU，实现照它上一次换上的那一组卡与上限、等的上限跑（`participating` 与各卡预算没动）。条款「重开期间照旧用切换前那组卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）：切换前一刻是 cpu，没有「那组卡」；实现取的是上一次 gpu 的那一组，条款没写。
- 读出来的两个后果：（A）新一组可能给了更小的显存上限，开着的那几分钟（至多 N×23 分钟）仍按旧上限分活，第 3 条「任何时刻都不许比它多」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`）说的是现行那一组的上限，现行组在核心层已是新的、实现在用旧的；（B）更要紧的一条见下面 ② 的交错。
- 判：条款没写。（A）是读法选择，不是实现 bug；推翻它的现象：用户说 cpu 切回 gpu 时开卡期间 GC 留在 cpu 上跑。

### ② 同一设置 gpu 到 cpu 到 gpu：旧失败被当成新切换的（读出来的，没跑）

按字面走一遍：
1. gpu 设置 S 点到没开的卡 X：`settings_waiting_for_the_opening = Some(S)`，起线程（`crates/singlefs-background-gpu/src/background_gpu.rs:2229`、2231）。
2. 切到 cpu：核心层不把 cpu 设置交给实现（「let Some(gpu_settings) = &self.gpu_settings else {」`crates/singlefs-core/src/background_compute.rs:822`），两批之间也不去收开卡结局：「self.gpu_settings.as_ref()?;」（`crates/singlefs-core/src/background_compute.rs:856`）。
3. 线程开完（比如 4 次都不成）结局留在通道里，没人收。
4. 修好卡，切回 gpu，设置等于 S：`use_the_settings_from_the_next_batch` 看到 `opening_at_a_switch` 还是 `Some`（通道里的旧结局），不起新线程（2230），只把在等的一组记成这一份。
5. 下一个两批之间收结局：旧失败的设置与在等的一组相等，「if settings_of_the_opening == settings_waiting {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2302`），旧原因当这一次切换的报出去，GC 停下，那张卡对这次切换一次都没试。
- 这不是「不自动再试」要的样子：条款「切换点到的卡开不成……报出哪张卡、每次开不成的原因」说的是这一次切换点到的卡。要再切一次才会重新开。
- 同一条交错的另一面：开卡成功的结局留在通道里时，卡已经开着，cpu 期间没人关（上面第 4 行的 gpu 到 cpu），占着显存，正是这一欠账要消灭的「占用没分给它的卡」。
- 判：对不上。推翻它的现象：在 `background_gpu.rs` 的测试里造这个序列，第 4 步之后实现起了第二条开卡线程、或第 5 步没交回旧原因。

### ⑥ 卸载时开卡线程还在开

- 实现：丢后台 GPU 只处理开着的卡：「impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {」（`crates/singlefs-background-gpu/src/background_gpu.rs:3072`）；开卡线程与通道在字段里随之丢（不等、不 join）。线程没有取消信号：`gpus_opened_at_a_switch`（`crates/singlefs-background-gpu/src/background_gpu.rs:1108`）每张卡每一次之间不看收的一头在不在，最后才发送，发不出去照常丢：「let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2368`）。
- 开卡器那道闸只管挂载时的工作线程：「if self.an_opening_is_hung.load(Ordering::SeqCst) {」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:281`）、置位只在「self.an_opening_is_hung.store(true, Ordering::SeqCst);」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:284`）。
- 对不上：卸载之后这条线程还会按 1+3 次、间隔 1 分钟继续碰那张卡，最长 N×23 分钟（`longest_wait_of_one_gpu`，23 = 4×5 + 3×1），期间闸是放开的，重新挂载可以同时在同一张卡上开设备。实现员报告自己写了这一条（「进程开卡器的「上一次开卡还挂着」那道闸管不到这条线程」）。
- 泄漏账那一边对得上：这条线程上开成的卡量代价的提交都做完了，丢掉不会等：「Ok(card)」前的量代价（`crates/singlefs-background-gpu/src/background_gpu.rs:1166-1179`），量代价出错而提交还在跑的卡在开卡那一步里就泄漏并记账（`crates/singlefs-background-gpu/src/background_gpu.rs:3105`）。
- 判：开卡线程与闸对不上；账对得上。推翻它的现象：卸载后用一个会睡很久的开卡替身，再挂载一次，第二次挂载被闸挡住。

### ⑨ 开不成、GC 停下之后实现里一直记着，直到下一次交设置

- 记着的地方：`not_opened_after_the_reopenings_at_the_last_switch`（`crates/singlefs-background-gpu/src/background_gpu.rs:456`），置位在 2304，清除只在交设置的第一行：「self.not_opened_after_the_reopenings_at_the_last_switch = None;」（`crates/singlefs-background-gpu/src/background_gpu.rs:2222`）。核心层这边：校验过的切换请求一收到就解除全部停下：「self.replace_the_tasks_stopped(Vec::new());」（`crates/singlefs-core/src/background_compute.rs:1595`），到两批之间才把设置交给实现（同文件 1611）。
- 条款依据：「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。「不自动再试」「等人处理」（同条）。实现员把「实现里也记着」当作同一事件的另一半，条款没逐字写。
- 两边一起解除，读下来成立：从请求到交设置之间核心层认为没停、实现还记着，但这段里不会有批开出去，每个入口先过 1647（`begin_a_batch`）或 `mounted_session.rs` 的 3416、4513，先交设置清掉记录再判停没停；被拒的切换两边都不动。
- 停着的被悄悄放开：核心层能改停着那张表的只有三处，请求（1595）、交设置（1614）、收到开不成（1620），实现只交一次开不成（通道里的结局 2247 `take()` 掉，之后 `opening_at_a_switch` 是 `None`），第二次进来是 `None` 不会改写。没找到放开的路。
- 一处旁证：收到开不成时核心层把全部有 GPU 路径的任务整表换掉（1620），「停下以来叫起的次数」一并清零（`crates/singlefs-core/src/background_compute.rs:1436`）；若同时还停着一条别的原因的停下，会被换成这一条。现在只有 GC 一个有 GPU 路径的任务，今天走不到。
- 判：成立；没有一种交错读出两边分叉，除了上面交错甲里旧结局被新切换认走。

## 三、攻击面 3：关卡与泄漏账（设计问题 ⑦）

- 和卸载是同一个规矩：卸载（`crates/singlefs-background-gpu/src/background_gpu.rs:3075`）与切换关卡（`crates/singlefs-background-gpu/src/background_gpu.rs:2415`）都调同一个函数 `gpus_dropped_leaking_those_still_running`（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:943`），判据同一个函数 `card_has_a_submission_running_past_its_wait`（`crates/singlefs-background-gpu/src/background_gpu.rs:3087`），泄漏的做法同一句：「std::mem::forget(gpu);」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:952`），账同一个入口 `note_the_gpus_leaked_when_dropped`（卸载 `crates/singlefs-background-gpu/src/background_gpu.rs:3080`，关卡 `crates/singlefs-background-gpu/src/background_gpu.rs:2420`）。条款「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）只写了卸载，关卡也这样做是实现员的类推，条款没写。
- 不一样的只有报告口径：卸载把泄漏的卡交给调用方读：「self.gpus_leaked_when_dropped.note_the_drop(gpus_leaked);」（`crates/singlefs-background-gpu/src/background_gpu.rs:3082`）；关卡只进进程账。类型文档说它报「丢掉它那一刻泄漏了哪几张卡」（`crates/singlefs-background-gpu/src/background_gpu.rs:2180`），所以一次挂载期间在切换里泄漏的卡，调用方读报告看不到，要读进程账 `bytes_leaked_on`。条款没写报告要不要含关卡泄漏。判：同一规矩，报告口径不全（条款没写）。
- 重复扣或漏扣（按字面算了一遍）：泄漏时账加的是占着的扣掉开卡时已算上的（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:697` 的 `bytes_leaked_by_this_mount`）；重开同一张卡时新账从进程账的数起：「bytes_held: bytes_leaked_by_earlier_mounts,」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:421`），装不装得下最小的一片也读同一个数（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:823`）。算下来：开卡时已算上 a、泄漏时占着 h，账加 h−a 成 h，重开从 h 起，再泄漏占着 h2 加 h2−h：不重复、不漏。测试钉了加的那一步：`crates/singlefs-background-gpu/src/background_gpu.rs:4913` 的用例断言「泄漏账加的恰好是这一次占着的 600 字节」；重开之后再扣的那一步没有专门的用例（只有变异行「切换关卡泄漏的卡不记进这个进程的泄漏账」罩加的那一步）。
- 下标错位：`participating` 存的是卡在 `cards_on_this_host` 里的位置。关卡后按标识重找：「self.participating = ParticipatingGpusOnThisHost::of_the_positions(」（`crates/singlefs-background-gpu/src/background_gpu.rs:2422`），次序照关卡之前的参与次序；换上一组的路上随后整个覆盖（2490）。新开的卡只接在 `cards_on_this_host` 末尾（2295），所以挪的只有关卡。`left_out_under_their_memory_limit` 按标识不按位置。没找到错位。`discrete_gpus_after_the_opening.opened` 与卡同序靠两处同做一件事（`note_the_gpu_opened_at_a_switch` 在 `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:302`，`note_the_gpu_closed_at_a_switch` 在 316），`of_the_cards_opened_at_the_mount` 只断言张数相等不断言标识相同（`crates/singlefs-background-gpu/src/background_gpu.rs:2134`），挂载时是同一个循环里一起推（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:895-896`）。
- 泄漏之后设备还在进程里（没析构），卡清单却把它列成没开的：同一进程仍占着那张卡的上下文，这是「整张泄漏」本身的代价，不是新缺陷，也是 C996 想消掉的「占用没分给它的卡」在这一支上消不掉的部分。
- 判：同一规矩，账对得上，下标没错位；报告口径不全。

## 四、攻击面 4：期限与挂载（设计问题 ②⑧）

| 期限 | 挂载开卡 | 挂着切换开卡 |
|---|---|---|
| 一次开卡至多等 | 5 分钟：「pub const WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT: Duration = Duration::from_secs(5 * 60);」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:276`），套在每一次开卡上（`crates/singlefs-background-gpu/src/background_gpu.rs:1092`） | 同一个函数同一个常量：切换开卡那一步就是挂载交进来的 `open_a_gpu`（`crates/singlefs-background-gpu/src/background_gpu.rs:1018`） |
| 等 wgpu future | 挂载那一组的 `background_gpu_wait_limit`（`crates/singlefs-background-gpu/src/background_gpu.rs:1002`） | 交来的那一组的：「let wait_limit = settings.wait_limit();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2351`） |
| 重开之间 | 在开卡线程里 `std::thread::sleep` 60 秒（`crates/singlefs-background-gpu/src/background_gpu.rs:997`） | 同（`crates/singlefs-background-gpu/src/background_gpu.rs:1119`） |
| 一张卡连同重开 | `longest_wait_of_one_gpu`：每次上限乘 (重开次数+1)，加间隔乘重开次数（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:341`），5 分钟下 23 分钟，测试钉了（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:1108`） | 同一个数，但没人等它 |
| 外层期限 | 枚举期限 5 分钟，开卡线程报回卡数后延成「枚举 + 卡数 × 23 分钟」：「each_gpu: ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:54`）、`after_the_enumeration`（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:60`），卡数是参与的卡数：「report_the_gpus_to_open(participating.len());」（`crates/singlefs-background-gpu/src/background_gpu.rs:989`） | 没有外层期限：交设置当场返回，结局在两批之间 `try_recv`（`crates/singlefs-background-gpu/src/background_gpu.rs:2264`） |

- 算式对真走的等待：`longest_wait_of_one_gpu` 的算式 = 循环真走的（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:359-366`：开 1+重开次数 次、等重开次数 个间隔）。外层期限从工作线程开始算起（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:331`），枚举用掉的时间也在里面，所以留给卡的是「5 分钟 + N×23 分钟 − 枚举实际用时」，不小于 N×23 分钟（枚举实际用时到 5 分钟就先到期了）。没有一处按单卡 5 分钟封顶整个重开序列：判，没有让重开做不满 3 次的期限。
- 一处极端（推的，没跑）：枚举用到差一点 5 分钟、再加每张卡每一次都超时到点，最后一张卡最后一次开卡的到点和外层期限贴在一起，调用线程可能先报 `TimedOut` 而不是带 4 个原因的拒绝。推翻它的现象：外层期限代入 `each_gpu` 后留的余量够 N×23 分钟以上（上面的算式），则只剩调度抖动。
- 设计问题 ②（挂载最多被一张开不成的卡挡 23 分钟）：条款没写挂载能被重开挡多久；今天只有「到点当那张卡这一次没开成」（5 分钟）与「重开 3 次」两样，23 分钟是两样相乘（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。
- 设计问题 ⑧（切换开卡每一次沿用挂载的 5 分钟，等 future 用这一组的 `background_gpu_wait_limit`）：条款只写了挂载时的 5 分钟（同条），切换没写，实现沿用，条款没写。

## 五、实现员设计问题 ①–⑨ 的落点与依据（题面第二问）

| 问题 | 在代码里的落点（文件:行） | 依据的条款句 |
|---|---|---|
| ① 共开 4 次、报 4 个原因 | 「pub const REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST: u32 = 3;」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:322`）；循环 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:359-366`；挂载 `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:880`；切换 `crates/singlefs-background-gpu/src/background_gpu.rs:1119` | 「都间隔 1 分钟重开，最多重开 3 次」「报警里报出哪张卡、每次开不成的原因」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。读法见第 12 行：读对 |
| ② 挂载被一张开不成的卡挡到 23 分钟，外层期限 5 分钟加卡数乘 23 分钟 | `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:52-56`、`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:60-64`、`crates/singlefs-background-gpu/src/background_gpu.rs:989` | 条款没写挂载能被挡多久。可推的只有「等的上限是 5 分钟，到点当这一次没开成」与「重开 3 次，都不成才拒绝挂载」两句（同 253），23 分钟是两者相乘，条款没写这个积 |
| ③ 起不了开卡线程当头一张卡开不成，1 个原因，不重开 | `crates/singlefs-background-gpu/src/background_gpu.rs:2376-2384`；收结局 2248-2257 | 条款没写；与 176 行的类型文档（1+重开次数 个原因）不一致 |
| ④ 开卡时又来切换，记最近一组，等这一次开完再看，更早一组开不成的不报 | `crates/singlefs-background-gpu/src/background_gpu.rs:2221-2233`、`crates/singlefs-background-gpu/src/background_gpu.rs:2297-2315` | 条款没写。能挂的只有「切换请求照收，在会话锁外开新卡」（同 253）与「一批之内不换」（`.claude/kb/decisions/26-后台整理与放置回收.md:250`） |
| ⑤ cpu 切回 gpu 点到没开的卡，核心层认新那一组、实现照上一次换上的那一组卡 | 核心层 `crates/singlefs-core/src/background_compute.rs:1610-1611`；实现不动 `participating`：2229-2231 | 「重开期间照旧用切换前那组卡」（同 253）；cpu 之前那一刻没有卡，条款没写取哪一组 |
| ⑥ 卸载时开卡线程不等，开成的卡在那条线程上析构 | `crates/singlefs-background-gpu/src/background_gpu.rs:3072-3084`、`crates/singlefs-background-gpu/src/background_gpu.rs:2368`；开卡器闸 `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:281-284` | 「卸载时不等还在跑的提交……整张泄漏、不析构」（同 253）只说提交，条款没写卸载时开卡线程怎么办 |
| ⑦ 切换关卡泄漏的卡只记进程账，不进卸载那一刻的报告 | `crates/singlefs-background-gpu/src/background_gpu.rs:2391-2432`（2415、2420）；报告 3082 | 卸载那一句类推；关卡泄漏条款没写，报告口径也没写 |
| ⑧ 切换开卡每一次上限沿用挂载的 5 分钟，等 future 用这一组的上限 | 「let wait_limit = settings.wait_limit();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2351`）；5 分钟经 `crates/singlefs-background-gpu/src/background_gpu.rs:1018` 的同一个 `open_a_gpu` | 条款只写了挂载时「等的上限是 5 分钟」（同 253）；切换没写 |
| ⑨ 开不成、GC 停下之后实现里一直记着，直到下一次交设置 | `crates/singlefs-background-gpu/src/background_gpu.rs:2222`、2304；核心层 `crates/singlefs-core/src/background_compute.rs:1595` | 「停着的时候收到任何一次通过校验的切换请求……当场解除停下」「不自动再试」（同 253）；实现里记着这件事条款没逐字写 |

## 六、检查单八格（题面第三问）

1. 走不到的理由二次查实：**成立**。
   - `assert!` 三种变两种：参与的卡一张都不剩时，开不成已在上面先交回（`crates/singlefs-background-gpu/src/background_gpu.rs:2505`），量代价出错已先交回（2512），余下只能是 `left_out_under_their_memory_limit` 非空。穷举 `participating` 变空的路：挂载（`leaving_out` 只在有卡被晾下时才变空，`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:933-934`；点不到卡核心层已拒）、换上一组（空的只来自 `positions_left_out`，里面不是显存上限就是量代价出错，2466-2491）、关卡（只关本轮新开的卡或不点名的卡，2298、2303、2445，不关参与的卡）。断言文案在 `crates/singlefs-background-gpu/src/background_gpu.rs:2523`。
   - `expect`：`start_the_opening_at_a_switch` 的「至少一张」，两个调用点都先判了非空（2224 与 2309 之后的 else 支，调用在 2231、2313）；开卡线程上锁（2356）：`std::sync::Mutex` 线程 panic 会毒化，之后任何一次再起线程都会在这里 panic；文案说「上一次开卡线程 panic 时那一次已经往上报过」成立，因为那次 panic 经 `resume_unwind`（2280）在核心层借着实现的那把锁里重新抛出，核心层那把锁跟着毒化，之后每次借出都先在 `expect("GPU 实现上一次借出时 panic 过……`（`crates/singlefs-core/src/background_compute.rs:470`）死掉，到不了 2356。直接用 `BackgroundGpuOverTheCards`、不经核心层把手的用法（测试）不受这条保护，也没人那样用。其余三处：`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:307`、`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:321`（外壳只对自己推进卡与清单的两处同做）、`crates/singlefs-core/src/background_compute.rs:859`（与 843 的姊妹函数同形，`gpu_settings.as_ref()?` 在前）、`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:240`（只有 2447 调它，调之前 2224 与 2309 都判过要开的卡为空）。
2. 类型与名字：**成立**（带两条旁注）。
   - 重开次数、开卡次数、原因个数没共用裸整数：重开次数是 `reopenings_at_most: u32`（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:329`），开卡次数是循环下标，原因个数是 `Vec` 的长度，三者没有互相用同一个裸数；代价是测试里把 4 写死（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:1402` 的 `vec![...; 4]`），常量改了要手改，变异行会红，不会静默。
   - `_ =>`：diff 新加行里没有；新成员 `NotOpenedAfterTheReopenings` 加进了所有穷举的 `match`：`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:134`、204，`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:638`，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:372`。
   - `expect` 文案都写了依赖的不变量（见第 1 格各处）。
   - 旁注一：`first_device_to_open`（`crates/singlefs-background-gpu/src/background_gpu.rs:2344`）是位置代称，共用约束写法纪律字面禁这种，门禁词表（`.claude/naming-ordinal-tokens`）只列了 `first_stream` 一类、不会红。旁注二：diff 新加的注释里「用户定案」「用户 2026-…」共 21 行（量的：对 `research/prompts/_c996-code-r1-diff.md` 用 `grep -cE '^\+.*(用户定案|用户 2026-)'`），共用约束写「代码注释只写现状、不写谁定的」，这些归门禁 comment-history 格，我没跑，所以只算推的。
3. 不兼容旧代码与旧格式：**成立**。`grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r1-diff.md` 零命中（量的，退出码 1）。`BackgroundGpuCompute` 新方法缺省 `None`（`crates/singlefs-core/src/background_compute.rs:389`）给的是不开卡的实现，不是旧格式：不开卡的实现有 `crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:177` 等四处；同类缺省方法早已有（`crates/singlefs-core/src/background_compute.rs:400`、406）。缺省静默吞掉的风险在转发包装上，`e202` 两个包装都显式转发了（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:3027`、`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:7792`）。
4. 不新建判定库：**成立**。对 diff 新加行 `grep -cE '^\+.*(File::create|OpenOptions|fs::write|fs::File|rocksdb|RocksDB|sqlite|std::fs::)'` 命中 0（量的）；产品代码里开卡、关卡、重开都不写文件。
5. 所有任务多机可用：**成立**。真卡测试的卡来自服务交的环境变量：「pub const VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER: &str = "GPU_CARDS_BY_PRIORITY";」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:38`），这一台是哪一项只由多机配置判法认；进程在哪几张卡上读 `nvidia-smi` 当场的输出（序号与 UUID 现读），文件里没有写死的序号、UUID、主机名、地址（`grep -nE` 对 UUID 形、点分地址零命中，量的）。读不成、或读到这个进程一张卡都不在，打 `NOT_RUN_NO_GPU_ADAPTER` 按未跑算：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:603`、611。第二台上读不读得到：没有在第二台跑过，复核不了。
6. 测试优先多线程：**成立**。新加的单测一条一个场景、各自造替身卡与状态；diff 新加的测试行里没有 `for ... in` 的参数循环（对 `_c996-code-r1-diff.md` 的 `grep -nE '^\+\s+for .* in '` 只命中产品代码 5 处：353、1670、1854、1967、3687 行，量的）；测试里没有 `thread::sleep`（命中的两处在产品代码，1641 与 1672 行）。等的写法是放开信号：开卡那一步等通道放开，上限 20 秒只是别一直挂着的保险：「.recv_timeout(Duration::from_secs(20))」（`crates/singlefs-background-gpu/src/background_gpu.rs:4651`），放开在 4686。缺口不在这一格：没有测试罩 ③④⑤⑥ 与交错甲（见攻击面 2）。
7. 带 --item 的命令与项：**成立**（26 个名字），另有没罩着的新行为。
   - 量的（只读命令）：两份实现员的变异补丁（`~/.local/share/singlefs/work/c996-reopen-and-close/patch/mutations-append.tsv` 21 行与 `mutations-replacements.tsv` 5 行）里的 26 个名字，对 `crates/mutations.tsv` 逐个 `grep -cF`：每个恰好命中 1 次，0 个缺（输出 `total=26 miss=0`）；删除清单 8 个名字在 `crates/mutations.tsv` 里 0 次（8 行都是 `deleted-still-present=0`）；26 行的锚点串各在对应源文件里恰好出现 1 次（脚本输出 `anchors not exactly once: 0`，对主工作区此刻的源文件）；每行最后一列点名的测试函数在 `crates/` 里都有同名 `fn`（输出 `bad 0`）。行在 `crates/mutations.tsv` 的 7655、7671-7691 与 5678-5684。
   - 没变异行罩着的新行为（读码列的，推的）：产品挂载传 `OF_THE_PRODUCT` 与 `std::thread::sleep`（`crates/singlefs-background-gpu/src/background_gpu.rs:996-997`，实现员自己写了）、两份核那一层用 `NotAtAll`（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:155`，同）；另有：起线程失败这一支（`crates/singlefs-background-gpu/src/background_gpu.rs:2376`）、失败时把本轮新开的卡关掉（2303）、没有在等的一组时把本轮开的卡关掉（2298）、按在等的一组分「报」与「不报」的相等判断（2302）、关卡后参与的卡重找位置（2422）、关掉的卡按总线地址插回没开的那几张（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:323-329`）。
8. 新崩溃流多罩了什么：**成立**。开卡、关卡、重开都不写盘；停下的路是已有的两条：一批开头判停：「let Ok(batch_under_way) = background_compute.begin_a_batch(」（`crates/singlefs-core/src/mounted_session.rs:5119`）失败时走 `drop_the_live_intent_on_a_stop`（同文件 5125，只删意图的发布，已有的路），两个入口开头在判停之前换上（3416、4513）。新的只有「在哪个时刻发现要停」：任一次叫起的两批之间也可能发现，但与「挂着切到一张卡都用不上的那一组」同一个函数同一个位置（`crates/singlefs-core/src/background_compute.rs:1608-1621`），落盘形态没有新的。切换期间 GC 停下与恢复没有让已有崩溃流罩不到的新落盘形态；判据是 `drop_the_live_intent_on_a_stop` 那一支今天已有的崩溃枚举，我没读那一支的用例，复核不了「已有的崩溃枚举罩住这个新发现时刻」，只能说没有新形态。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 攻击面 1，第 1 行 只开参与的卡 | 对上 | 不参与的记进没开清单不开（`gpus_of_this_host.rs:875`） |
| 第 2 行 没点名的口径 | 对上 | 全部独显，其余四类不算（`adapter_inventory.rs:296`、44） |
| 第 3 行 切换点到再开 | 对上 | 交设置当场返回，线程开（2223、2231） |
| 第 4 行 切换后不再参与的卡关掉 | gpu 到 gpu 对上；gpu 到 cpu 字面对不上 | cpu 切换不交给实现（`background_compute.rs:822`），卡不关；条款没明说 cpu 算不算，待主 agent 定 |
| 第 5 行 1 分钟 / 最多 3 次 | 对上 | 常量 60 秒、3；头一次不等 |
| 第 6 行 会话锁外 | 对上 | 线程只拿自己的锁 |
| 第 7 行 旧卡照用 / 下一批换上 | 对上（gpu 到 gpu） | cpu 到 gpu 取哪一组条款没写 |
| 第 8 行 停 GC、报卡与每次原因、不自动再试 | 对上 | 开卡超时报成「等这一批的提交」，标签含糊 |
| 第 9 行 挂载拒绝 | 点名对上；没点名对不上 D26 字面 | D26 第 4 条两句与挂载参数表第 23 行读法冲突，代码取一律拒 |
| 第 10 行 5 分钟 | 对上 | |
| ① 共开 4 次 | 读对 | 「重开」是头一次之后的再开 |
| ② 23 分钟 | 条款没写；算式对得上真走的 | 无一处单卡 5 分钟封顶整个序列 |
| ③ 起不了线程 | 条款没写；与类型文档不一致（轻） | 1 个原因，也没重开 |
| ④ 开卡时又切换 | 条款没写；自洽 | 没找到分错活、放开停着的交错 |
| ⑤ cpu 到 gpu | 条款没写 | 开卡期间实现照上一次 gpu 的那一组 |
| 交错甲 gpu 到 cpu 到 gpu 同设置 | **对不上（读出来的）** | 旧失败结局被当成新切换的，那张卡一次都没试 |
| ⑥ 卸载时开卡线程 | **对不上** | 开卡器闸管不到切换开卡线程，卸载后还开最长 N×23 分钟，可与重挂同时开同一张卡 |
| ⑦ 关卡泄漏 | 同一规矩；账对得上；报告口径不全 | 下标没错位；泄漏账不重复不漏 |
| ⑧ 切换开卡期限 | 条款没写 | 沿用挂载的 5 分钟与这一组的等的上限 |
| ⑨ 停着时实现记着 | 成立 | 核心层与实现被同一次校验过的切换一起解除 |
| 检查单 1–8 | 成立 ×8 | 见第六节；第 2 格带两条旁注，第 7 格带没罩着的新行为清单，第 8 格有一处复核不了 |

## 八、要主 agent 跑的命令

我没写模型，没有复跑脚本，也没有往统一任务服务交任务；下面三条是读出来的结论里能用已有用例核的，每条一项。

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 库测试里重开次数的两条用例; 范围=库测试目标内点名 2 个函数; 为什么=核实开卡共 1+3 次、4 个原因与等 3 个间隔' task-server/task submit -- cargo test -p singlefs-background-gpu --lib waiting_for_the_gpu::tests::the_gpu_never_opened_is_reopened_three_times_and_every_cause_is_handed_back_in_order waiting_for_the_gpu::tests::the_product_reopens_a_gpu_not_opened_three_times_one_minute_apart`；在哪个目录起：仓根；看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`，`task-server/task log <号>` 里 `test result: ok. 2 passed`。
2. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 库测试里切换开卡的三条用例; 范围=库测试目标内点名 3 个函数; 为什么=核实切换重开第三次开成、开着时不等、关卡泄漏记账' task-server/task submit -- cargo test -p singlefs-background-gpu --lib background_gpu::tests::the_card_not_opened_twice_at_the_switch_opens_the_third_time_and_the_switch_takes_effect background_gpu::tests::the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch background_gpu::tests::the_card_closed_at_a_switch_with_a_submission_still_running_is_leaked_and_noted_in_the_process`；在哪个目录起：仓根；看哪几行：同上，`test result: ok. 3 passed`。
3. 没有现成用例能核交错甲与 ⑥：要让实现员加两条用例（交错甲：gpu 点到没开的卡 → 切 cpu → 开卡线程开完 → 切回同一设置，应当起第二条开卡线程、不交回旧原因；⑥：卸载时开卡线程在睡，闸应当挡住下一次挂载，或线程应当在卸载后停止再碰卡），我没写，因为写进 `crates/` 不归这一腿。

## 九、没做什么

- 没判别的腿的格，没替主 agent 采纳、没出判决。
- 没跑任何项目代码（没编译、没跑测试、没跑门禁与变异）；交错甲、⑥、③ 是读码推的，没有用例实证；「读出来的」后面写了推翻它的现象。
- 没在第二台读真卡断言；没读 `drop_the_live_intent_on_a_stop` 那一支的崩溃用例（第 8 格复核不了）。
- 没读另一条腿的产物与调度记录；没动 `crates/`。
- 实现员报告「走不到的理由」只读了第二份原文，第一份（只开参与的卡）的原文没读，第 1 格的判断来自读码。
- 没核「合入后验证任务 13924」等主 agent 给的验证数（背景材料里的数，当「推的」）。

## 十、什么现象会推翻上面的结论

- 「对上」的各行（攻击面 1 第 1–3、5、6、10 行）：推翻它们的是一条用例或一次真卡运行里观测到相反的事，例如不参与的卡被开了、重开次数不是 1+3、交设置在会话锁里等开卡。已有的用例（第八节命令 1、2）绿只证这几条读法与测试一致，不证条款。
- 第 4 行（关卡）和 ⑤：推翻的是主 agent 或用户对「cpu 算不算不再参与」「cpu 切回 gpu 的『切换前那组卡』指哪组」的读法，不是代码里的现象。
- 第 9 行（没点名时拒绝挂载）：推翻它的是用户说明「默认模式下开不成的卡跳过、参与的卡一张不剩才照样挂」；那样代码要改回有「没开成的卡」这一路，用户定案已经删掉了它，所以先改 D26（后台整理与放置回收）第 4 条的字，再谈代码。
- 交错甲、⑥、③：都是读码推的。推翻交错甲：造 gpu 到 cpu 到 gpu 同设置的序列，实现起了第二条开卡线程。推翻 ⑥：卸载时线程在睡，再挂载被闸挡住（或线程在卸载后不再碰卡）。推翻 ③：起线程失败的那一支真的重开 3 次、报 4 个原因。
- 攻击面 3 的「不重复不漏」：推翻它的是一条用例，关卡泄漏后重开同一张卡，新账的起点不等于进程账里的数。
- 攻击面 4：推翻它的是一次真走，某一处用单卡 5 分钟封顶了整个重开序列，或外层期限小于 N×23 分钟。
- 检查单各格：每格的证据已写在格里；格 5 复核不了第二台，格 8 没读崩溃枚举那一支。

## 十一、cite-check

对本报告跑 `python3 scripts/agents/cite-check.py research/prompts/c996-code-r1-sonnet-output.md`，退出码 0，末行：「✓ 核了 75 处引文、93 处裸行号，对上 75 处，没判 93 处（其中裸行号没有引文可比 93 处；逐处列在上面）」。93 处裸行号没有引文可比，我逐处把行号对应的那一行读出来看过（内容与我写的话对得上）。
