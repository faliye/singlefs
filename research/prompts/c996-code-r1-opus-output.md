# c996-code-r1 攻方腿报告（Opus，2026-11-12）

立场：假设实现员的读法与主 agent 的倾向是错的，造交错打穿它。只攻正文第三节四条攻击面与第四节检查单八格。方案按 `crates/` 今天的实现谈，代码行号取主工作区此刻的行号。

## 复跑命令与模型文件

模型是两份接进测试模块末尾的 lib 测试（不起卡、不写盘），没有崩溃状态枚举：这一轮的对象是开卡、关卡与核心层停下的内存状态机，不写盘，第 3b 步的崩溃状态枚举不适用。我一条都没跑（子 agent 不交任务），读数全部「推的，待主 agent 跑」。

复跑（交给服务；`<运行目录>` 是工作根下名字以 `-run` 收尾的新目录）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/c996-code-r1-opus-model
```

`rerun.sh` 把 `src/` 两份接进冻结副本拷出的树（已接过的跳过），各跑一次 `cargo test -p <包> --lib attack_ -- --nocapture`，末尾打 `name=attack_…` 行、`test result:` 行与 `name=rerun_exit` 行。冻结副本就是我的仓副本（主工作区此刻的树经 `minimal-copy.py` 拷出，已接好两份测试并 `cargo fmt` 过）。

| 文件 | sha256 |
|---|---|
| `research/prompts/c996-code-r1-opus-model/src/attack_on_the_switch_opening.rs`（接在 `background_gpu.rs` 测试模块末尾，5 条用例） | `5d930e40ed1aabe724b51455b03f78e15de9b3525f9c71763f4da6df5d965fb5` |
| `research/prompts/c996-code-r1-opus-model/src/attack_on_the_core_stop.rs`（接在 `background_compute.rs` 测试模块末尾，5 条用例） | `d59af6289da48c5a3b68279783ca54900ae596bb002e655b53087a7cd50f15f9` |
| `research/prompts/c996-code-r1-opus-model/tools/rerun.sh` | `35b4e5863b8e9f0cd72fdf0154df973dbb51ae634e1680d772e2bfbfb5d61551` |
| `research/prompts/c996-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

每条用例的断言写的是本报告说的那种行为：用例过，就是那一格照本报告说的走；用例红，那一格的打中撤回。两份各带对照臂（同一段历史只差一步），见各节。

## 各格判定一览

| 号 | 格 | 判 | 违反的句子（kb 文件自己的行号） | 模型用例 |
|---|---|---|---|---|
| 甲 | 攻击面 2 / 设计问题 ⑤：切换开卡期间旧卡出错，GC 停下，新卡换上之后照旧停着 | 打中 | 第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」（26 号 kb:253） | 核心层 `attack_a_stop_on_the_card_before_the_switch_…` 与对照臂 |
| 乙 | 攻击面 2 / 设计问题 ⑤⑨：交设置那一刻读到的是切换之前那一组的「用不上」，重新停下，新卡换上之后不解除 | 打中 | 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」（:253） | GPU 包 `attack_the_failure_read_when_the_settings_are_handed_in_…`、核心层 `attack_the_stop_of_the_settings_before_the_switch_…` |
| 丙 | 攻击面 1：开完换上的那一组一张卡都用不上，两批之间那一问不交回，下一批照 GPU 开、出错停在别的成员上 | 打中 | 第 4 条「报的是挂载那一刻同一个成员（`GpuUnusableUnderTheSettingsHandedIn`）」（:253） | GPU 包 `attack_the_switch_put_in_effect_by_the_opening_…`、核心层两条（含对照臂） |
| 丁 | 攻击面 2 / 设计问题 ④⑨：同一组设置在那一问之前又交一次，拿到上一次开卡的陈旧失败、一次都不重开 | 打中 | 第 4 条「重开 3 次都不成，GC 停下」（:253）；设计问题 ④ 自己说的「为更早的一组开的、开不成的，不报」 | GPU 包 `attack_the_same_settings_handed_in_again_before_the_poll_…` 与对照臂 |
| 戊 | 攻击面 1 / 3：三种情形下不再参与的卡不关——切到 `cpu`；为被盖掉的切换开成的卡在最近那一组开不成之后；切换时有提交还在跑而泄漏的卡 | 打中 | 第 2 条「切换之后不再参与的卡，切换生效时关掉。」（:251） | GPU 包 `attack_the_card_opened_for_a_superseded_switch_…`（第二种）；另两种按代码推 |
| 己 | 设计问题 ⑤：`cpu` 切回 `gpu` 点到没开的卡，开着期间在切 `cpu` 之前那组卡上算 | 推不出（算打中） | 第 4 条「重开期间照旧用切换前那组卡」（:253） | 推的 |
| 庚 | 设计问题 ③ / 检查单「类型与名字」：起不了开卡线程报 1 个原因、不重开 | 打中 | 第 4 条「都间隔 1 分钟重开，最多重开 3 次」（:253）；类型自己的不变量「1 + 重开次数 个」 | 推的 |
| 辛 | 攻击面 2：开卡线程 panic 在会话锁里原样再 panic，整个运行库报坏 | 推不出（算打中） | 条款没写；挂载那一刻同一种 panic 只拒挂载 | 推的 |
| 壬 | 设计问题 ⑥ / 攻击面 3：开卡器的「上一次开卡还挂着」闸管不到切换开卡线程与重开之前没回来的那一次 | 打中（内部矛盾） | 开卡器文件头自己写的「同一个驱动加载器里再起一条会不会也挂没量过」 | 推的 |
| 癸 | 设计问题 ⑦：切换关卡泄漏只进进程账，运行时看不出走了泄漏那一支 | 打中（轻） | `.claude/rules/fs-design.md` 五条硬要求第 4 条 | 推的 |
| 子 | 攻击面 1：kb 第 4 条「没点名的卡不用，参与的卡一张不剩时挂载照样不拒」在只开参与的卡之后读不出所指 | 推不出（交 kb 措辞） | :253 那一句 | 推的 |
| — | 设计问题 ①（一共开 4 次） | 成立 | | |
| — | 设计问题 ②⑧（期限） | 成立（⑧ 条款没写，无害） | | |
| — | 泄漏账重复扣、漏扣，按标识重找位置错位 | 没打中 | | |
| — | 停着的 GC 被悄悄放开 | 没打中 | | |
| — | 检查单八格 | 见第五节：走不到 成立；类型与名字 不成立（庚）；不兼容 成立；判定库 成立；多机 成立；多线程 成立；带 --item 不成立（漏罩）；崩溃流 成立 | | |

## 一、攻击面 2：设计问题 ③④⑤⑥⑨ 的并发交错

根子是一处：核心层把「交设置给实现」当成切换生效（`in_effect` 在一批开头就换成新那一组），实现却要等会话锁外开完卡、下一次两批之间那一问才真换上。两边对「生效」的时刻差一个开卡期（每张卡至多 23 分钟），下面甲、乙、丙、丁都落在这一段里。

### 甲 切换开卡期间旧卡出错，GC 停下，新卡换上之后照旧停着（打中）

历史（每步指到许可它的那一句）：

1. 挂载点 A（开着）；请求切到只点 B（挂载时没开）。核心层照收：「self.replace_the_tasks_stopped(Vec::new());」（`crates/singlefs-core/src/background_compute.rs:1595`）。
2. 下一批开头核心层换上并交给实现：「if let Some(requested) = self.requested_from_the_next_batch.take() {」（`crates/singlefs-core/src/background_compute.rs:1609`）、「self.in_effect.hand_the_gpu_settings_to_the_implementation();」（`crates/singlefs-core/src/background_compute.rs:1611`）。实现点到没开的 B，起开卡线程、当场返回：「if self.opening_at_a_switch.is_none() {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2230`）。这一批照旧在 A 上算（实现员自己的用例 `the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch` 钉的就是这一格）。
3. 这一批在 A 上出错（设备丢了、等超过上限都行）。核心层停下：此刻待生效的已经被第 2 步取走，「if self.requested_from_the_next_batch.is_some() {」（`crates/singlefs-core/src/background_compute.rs:1394`）不成立，记成停着。
4. B 开成，下一次两批之间那一问换上它、交回 `None`；核心层只在实现交回「开不成」时才动停着的那几条：「self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));」（`crates/singlefs-core/src/background_compute.rs:1620`），交回 `None` 什么都不做。GC 停在一张已经关掉、设置里也没点的卡 A 上，报警点名 A，直到用户再切一次。

条款：用户请求切到 B 在出错之前，按第 4 条与用户定案「开成了，下一批起换成新设置」，B 开完之前这一次切换「还没生效」，应当照它解除停下。代码按交设置那一刻算生效，不解除。对照臂 `attack_the_same_error_before_the_handing_in_is_lifted_by_the_switch_request`：同一个错落在第 2 步之前，照那一次请求当场解除（「return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;」（`crates/singlefs-core/src/background_compute.rs:1395`））。两臂里实现都还在 A 上算，结局只差核心层那一下交设置落在错之前还是之后。

用户最可能走到这条路：A 出毛病，用户切到 B 躲开它；B 开卡要几秒到 23 分钟，这期间的批照旧落在出毛病的 A 上。若用户在 A 已经停下之后才切（停着时收到切换当场解除），下一批照旧落在 A 上、再出错、再停（第 3 步），同样停到 B 换上之后。

四句：分辨臂——分得开「核心层按交设置算生效」与「按实现换上算生效」两种写法（对照臂只差这一步）；系统看得到——两批之间那一问就是核心层看实现换没换上的地方，看得到；满足的分句——「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）；改法见第六节甲。

### 乙 交设置那一刻读到的是切换之前那一组的「用不上」，B 换上之后不解除（打中）

历史：挂载点 A、给 A 的显存上限装不下最小的一片，参与的卡一张都不剩，核心层挂载就停着（第 3 条那一支）。用户为此切到只点 B（没开、不给上限）。请求那一刻解除（`:1595`）。下一批开头交设置之后核心层读一次「这组设置下一批分不分得到卡上」：「tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);」（`crates/singlefs-core/src/background_compute.rs:1613`）。实现此刻还照切换之前那一组，`batch_not_split_over_the_gpus` 交回 A 的「一张都不剩」（参与的卡空、`left_out` 里是 A），核心层当场又停，报的成员是 `GpuUnusableUnderTheSettingsHandedIn`、点名 A。B 开完换上之后那一问交回 `None`，停着不解除（同甲第 4 步）。

GPU 包用例 `attack_the_failure_read_when_the_settings_are_handed_in_is_the_one_of_the_settings_before_the_switch` 对着真外壳量：交设置那一刻交回 A 的「一张都不剩」，开完之后交回 `None`、参与的只有 B。核心层用例 `attack_the_stop_of_the_settings_before_the_switch_is_put_back_at_the_handing_in_and_outlives_the_opening` 照这一对答法走核心层：挂载停着、请求之后不停、交设置那一批停、B 换上之后那一批照旧停。

四句：分辨臂——分得开（点开着的卡的同一次切换当场换上、当场按新那一组判，不会停）；系统看得到——实现知道自己还没换上（`settings_waiting_for_the_opening` 是 `Some`），核心层看不到；满足的分句——「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）；改法见第六节甲、乙。

### 丁 同一组设置在那一问之前又交一次：陈旧失败、零次重开（打中）

历史：切到只点 B；B 头一次与 3 次重开都开不成，开卡线程把结局放进通道。核心层下一次两批之间那一问之前（后台入口默认 30 秒才叫起一次；或者用户先切到 `cpu`——选 `cpu` 时这一问直接交回 `None`：「self.gpu_settings.as_ref()?;」（`crates/singlefs-core/src/background_compute.rs:856`），这一段没有上限），用户修好 B、再交同一组设置（或从 `cpu` 切回同一组）。实现：清掉「开不成」（「self.not_opened_after_the_reopenings_at_the_last_switch = None;」（`crates/singlefs-background-gpu/src/background_gpu.rs:2222`）），记下这一组；上一次开卡还没被取走，不再起（`:2230`）。那一问取到上一次的结局，按内容比：「if settings_of_the_opening == settings_waiting {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2302`）成立，交回旧的 4 个原因，核心层当场停 GC。用户这一次切换，B 一次都没重开。

GPU 包用例 `attack_the_same_settings_handed_in_again_before_the_poll_get_the_old_failure_without_any_reopening` 断言又交时已开 4 次、交回 4 个原因、问完仍是 4 次；对照臂 `attack_the_same_settings_handed_in_again_after_the_poll_reopen_the_card_four_more_times` 在又交之前先问一次，又交的那一组新开 4 次（共 8 次）。两臂只差核心层那一问的时机。

条款：设计问题 ④ 写的是「为更早的一组开的、开不成的，不报」；按内容比，等值的更早一组的陈旧结局照报。kb「重开 3 次都不成，GC 停下……等人处理，不自动再试」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）：人处理了、又切了一次，这一次切换没有开过卡就停下。四句：分辨臂——分得开（对照臂）；系统看得到——实现能分出结局属于哪一次交设置（按交的次序编号即可），只是没分；满足的分句——上面那一句的「重开 3 次都不成」前件不成立而后件发生了；改法见第六节丁。

### 己 设计问题 ⑤：`cpu` 切回 `gpu`、点到没开的卡，开着期间在切 `cpu` 之前那组卡上算（推不出，算打中）

历史：挂载点 A；切到 `cpu`（核心层不告诉实现：「/// 选 `gpu` 时把这一组交给它带着的实现，从下一批起照它跑；选 `cpu` 时什么都不做（实现这时不算活，切到 `gpu` 那一批再交）。」（`crates/singlefs-core/src/background_compute.rs:817`））；再切到 `gpu` 只点 B（没开）。开 B 的这段时间核心层认为生效的是「`gpu`、只有 B」，每批走 GPU 路，实现照它上一次换上的那一组在 A 上算，A 的显存上限与等的上限还是切 `cpu` 之前那一组给的。条款「重开期间照旧用切换前那组卡」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）里「切换前」是 `cpu`，那一组一张卡都没有；照字面这期间应当在 CPU 上算（或停着），不该用用户已经切走、新设置也没点的 A。实现员把「切换前那组卡」读成「实现上一次在用的那组卡」，条款推不出这个读法。

叠上甲：A 正是用户切走的卡时，这期间的批在 A 上出错，GC 停下、报警点名一张设置里没有的卡，B 换上之后照旧停着。四句：分辨臂——分得开（「切 `cpu` 时把卡交还」或「开着期间走 CPU」两种写法都不在 A 上算）；系统看得到——核心层知道上一组是 `cpu`；满足的分句——上面那一句；改法见第六节戊。

### 庚 设计问题 ③：起不了开卡线程，1 个原因、不重开（打中）

`start_the_opening_at_a_switch` 起线程失败就记成头一张卡开不成，原因只有一个：「causes_of_each_opening: vec![GpuFailureCause::DeviceOrPipelineDidNotStart {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2380`），下一次两批之间交回、核心层停 GC。三处对不上：

- 条款：「挂载那一刻点名的卡开不成，或挂着时切换点到的卡开不成，都间隔 1 分钟重开，最多重开 3 次。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。起不了线程就是这张卡这一次没开成，条款要求重开，代码一次都不重开就停 GC。
- 类型自己的不变量：「/// 哪张卡、每一次开不成的原因（按开的次序，头一次的在前，1 + 重开次数 个）。」（`crates/singlefs-background-gpu/src/background_gpu.rs:345`），这里是 1 个。
- 同一份代码里另一处起线程失败照重开：每一次开卡内层再起一条线程等期限，那一处起不了线程报成 `DeviceOrPipelineDidNotStart`，是重开循环里的一个原因（`waiting_for_the_gpu.rs` 的 `outcome_of_opening_a_gpu_within_the_limit`），会重开。两种起线程失败一种重开、一种不重开。

起线程失败多是进程线程数或内存一时用尽，正是「间隔 1 分钟再试」能等过去的那一类。改法见第六节庚。

### 辛 开卡线程 panic：在会话锁里原样再 panic，整个运行库报坏（推不出，算打中）

开卡线程 panic（wgpu 在起设备、编管线里 panic，或替身之外任何一处），通道断开；下一次两批之间那一问在核心层持着会话锁时原样再 panic：「std::panic::resume_unwind(opening_thread.join().expect_err(」（`crates/singlefs-background-gpu/src/background_gpu.rs:2280`）。维护线程 panic 记成运行库坏了（「shared_of_the_thread.note_the_failure(description.clone());」（`crates/singlefs-mount-runtime/src/lib.rs:749`）），之后前台拿锁一律报「description: "持着会话锁的线程 panic 过".to_string(),」（`crates/singlefs-mount-runtime/src/lib.rs:461`）。也就是：挂着时一次切换点到的新卡在开卡里 panic，前台读写跟着全部失败。挂载那一刻同一种 panic 只让开卡器报「开卡的工作线程 panic 了」、拒这一次挂载（`gpu_opener_of_this_process.rs` 断开那一支）。第 4 条对切换点到的卡开不成只给了「重开、都不成 GC 停下、报警」一条出路，没写开卡 panic；代码选了最重的那一种，条款推不出。开卡线程上锁那一处的 `expect`（「"开卡那一步只在开卡线程上锁着用：上一次开卡线程 panic 时那一次已经往上报过",」（`crates/singlefs-background-gpu/src/background_gpu.rs:2357`））在产品里不会毒化着被走到：再 panic 时核心层持着实现的锁，实现那把锁先毒化，核心层之后每一次借实现都 panic（`background_compute.rs` 的 `lock` 文档）。所以检查单那一问「会不会毒化」答：会毒化，但走不到那个 `expect`，代价是整个运行库。改法见第六节辛。

### 壬 设计问题 ⑥：开卡器的闸管不到切换开卡线程，也管不到重开之前没回来的那一次（打中，内部矛盾）

开卡器文件头说它在上一次开卡回来之前闸住新开卡，理由是「（[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后闸放开、开出来的东西没人要：」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:10`）。这一轮的改动在两处绕过了它：

1. 重开：每一次开卡在另一条线程上至多等 5 分钟，到点之后「/// 到点之后那条线程照旧跑着、不停它（编管线是驱动里的同步调用，停不下来）：它开出来的东西没人收，它跑完时就地丢掉。」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:280`）；隔 1 分钟重开就在同一张卡、同一份适配器上再起一条（「gpu_opened_with_the_reopenings(reopenings, std::thread::sleep, || {」（`crates/singlefs-background-gpu/src/background_gpu.rs:1119`））。一张挂在驱动里的卡，挂载或切换会在它上面先后叠起至多 4 条卡在驱动里的线程。挂载那一刻开卡器的闸只盯工作线程：工作线程等完 4 次、拒了挂载就放开闸（「drop(gate_opened_when_the_opening_ends);」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:319`）），里面那 4 条还挂着，下一次挂载照样起枚举。
2. 卸载时切换开卡还在开：线程不等、带着挂载时那一份适配器接着开（至多 23 分钟）；开卡器闸是放开的，新一次挂载可以同时在同一张卡上起设备。开成的卡在那条线程上析构；量代价出错而有提交还在跑的那张记进进程泄漏账，账是同一份（「let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2349`）），不重不漏，只是晚到：新一次挂载开这张卡那一刻读到的泄漏字节里没有它，那张孤儿设备占的显存也不在新一次挂载的上限里。

kb 第 4 条要求到点算没开成、隔 1 分钟重开，没说前一次还挂着时怎么办；开卡器的那句理由与重开的写法在同一个包里互相矛盾，交主 agent 定是收紧重开（前一次没回来就不再起，算这一次又没开成）还是删掉开卡器那句理由。

### 设计问题 ⑨：停着与解除两边是否一起放开

请求切换那一刻核心层解除（`:1595`），实现在下一批开头交设置时清掉记着的开不成那一样（`background_gpu.rs:2222`）；两刻之间没有批能走到实现，不会出现一边放开一边停着去分活。没打中，限两点：切到 `cpu` 时实现那一样留着（无害，`cpu` 不问实现）；丁那一格是 ⑨ 与 ④ 叠出来的。

## 二、攻击面 1：代码做的是不是那几句话

| 句子 | 代码 | 判 |
|---|---|---|
| 挂载只开参与的卡，不参与的不开 | `gpus_opened_at_the_mount` 不参与的进 `not_opened_for_not_taking_part`、`continue` | 成立 |
| 没点名时「本机全部独立显卡」 | 卡清单答开着与没开的合起来、按总线地址排；没点名的切换取清单全部 | 成立 |
| 切换在会话锁外开新卡 | `start_the_opening_at_a_switch` 起线程、当场返回 | 成立（关卡与量「请回来的卡」的代价仍在锁里，条款没管） |
| 重开期间照旧用切换前那组卡 | 用实现上一次换上的那一组 | 从 `gpu` 切来的成立；从 `cpu` 切来的推不出（己） |
| 开成了，下一批起换成新设置 | 每批开头与两个入口那一问里换上 | 成立；但核心层在交设置那一刻已当成生效（甲、乙、丙） |
| 重开 3 次都不成：GC 停下、报出卡与每次原因、不自动再试 | 那一问交回、核心层停 | 起不了线程不重开（庚）；陈旧结局零次重开（丁） |
| 挂载那一刻点名的卡重开 3 次都不成才拒绝挂载 | `gpus_opened_at_the_mount` 重开后拒 | 成立；没点名时的读法见子 |
| 切换之后不再参与的卡，切换生效时关掉 | `put_the_settings_in_effect` 开头关 | 三种情形不关（戊） |
| 一批之内不换（第 1 条） | 换上只在一批开头与入口；`end_the_batch` 断言核心层那一组不变 | 成立 |
| 设计问题 ①：一共开 4 次、报 4 个原因 | 「for reopenings_so_far in 0..=reopenings.reopenings_at_most {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:359`） | 成立：条款写「重开」「最多重开 3 次」「每次开不成的原因」，头一次不算重开；派发提示「三次原因」与条款差一次，以条款为准 |

### 丙 开完换上的那一组一张卡都用不上：那一问不交回，下一批照 GPU 开、出错停在别的成员上（打中）

历史：挂载点 A；切到只点 B（没开），给 B 的显存上限装不下最小的一片（核心层只判不超过显存总量，照收）。开卡线程按装不下照开、不量代价（「Ok((card, _left_out_judged_again_when_the_settings_are_put_in_effect)) => {」（`crates/singlefs-background-gpu/src/background_gpu.rs:1136`））；那一问里换上时再判一次，B 不参与，参与的卡一张都不剩，但那一问只在「开不成」时交回，此刻交回 `None`（`put_the_gpus_opened_since_the_switch_in_effect` 末尾）。核心层只在「换上待生效的那一组」那一刻读「用不上」（`:1613`），之后不再读；下一批照 GPU 开，三段交回「参与的卡一张都不剩」，核心层按一批算着出错停（`GpuSegmentFailed`），那一批放弃、删意图。

条款：「挂着切换到参与的卡一张都不剩的设置（第 3 条）：下一次整理入口与后台入口照旧报停着，报的是挂载那一刻同一个成员（`GpuUnusableUnderTheSettingsHandedIn`）」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。对照臂 `attack_the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in`：同样的上限点开着的卡，交设置那一刻就按这个成员停着、不开批。两臂只差点的卡开没开着。B 还可以是「换设置时量代价出错」（`CostNotMeasuredAtTheSwitch`）那一种，同一条路。四句：分辨臂——分得开（对照臂）；系统看得到——实现在那一问里刚算出 `batch_not_split_over_the_gpus`；满足的分句——上面那一句的「报的是挂载那一刻同一个成员」与「下一次整理入口与后台入口照旧报停着」（这里入口没停，开了一批）；改法见第六节乙。

### 戊 三种情形下不再参与的卡不关（打中）

条款：「切换之后不再参与的卡，切换生效时关掉。」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）。C996 的来由就是进程在不参与的卡上开着设备、服务记 wrong-card。

1. 切到 `cpu`：核心层不告诉实现（`background_compute.rs:817` 那一句），开着的卡全都留着，切换开卡线程照开、开成的卡留在通道里。选 `cpu` 时没有一张卡参与。
2. 为被盖掉的切换开成的卡：切到 X（没开、开着）；再切到只点 Y（没开）。X 开成，接到开着的卡末尾，接着为最近那一组开 Y（「self.start_the_opening_at_a_switch(settings_waiting, discrete_gpus_to_open);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2313`））；Y 头一次与 3 次重开都开不成，只关「这一次开卡开成的」（「self.close_the_cards(|device| devices_opened_by_this_opening.contains(device));」（`crates/singlefs-background-gpu/src/background_gpu.rs:2303`）），这一次什么都没开成，X 留着、不参与，GC 停着期间一直开着，直到下一次换设置。GPU 包用例 `attack_the_card_opened_for_a_superseded_switch_stays_open_without_taking_part_when_the_latest_switch_fails` 断言之后开着的是 A 与 X、参与的只有 A、X 那张替身卡一次都没析构。
3. 切换时有提交等超过上限还在跑的卡：整张泄漏、不析构（第三节），设备一直开到进程结束。卸载时条款明许泄漏（「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）），切换时条款只写了关掉。用户反复切走、切回一张偶尔慢的卡，每一次都永久留下一份设备。

四句：分辨臂——分得开（第 2 种模型断言只差最近那一组开不开得成）；系统看得到——三种情形实现都知道卡不在此刻那一组里；满足的分句——上面那一句；改法见第六节戊。

### 子 kb 第 4 条「没点名的卡不用」在只开参与的卡之后读不出所指（推不出，交 kb 措辞）

现行一句：「挂载时开卡（含编管线）等的上限是 5 分钟，到点当那张卡这一次没开成：点名的卡间隔 1 分钟重开、最多重开 3 次，都不成才拒绝挂载，没点名的卡不用，参与的卡一张不剩时挂载照样不拒、后台任务进出错停着（第 3 条）」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。没给 `background_gpu_devices` 时本机全部独显都参与、也都「没点名」：照字面读，开不成的那张「不用」、全都开不成挂载也不拒；代码在这时重开 3 次、都不成拒绝挂载（`gpus_of_this_host.rs` 的 `without_named_cards_any_card_that_did_not_open_refuses_the_mount`，「BackgroundGpuOpenRefused::GpuUnusable {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:890`））。另一种读法：「没点名的卡」指给了列表时没点的卡；我核过 `git show HEAD:` 里改前的同一句（「点名的卡照「点名的卡不存在」拒绝挂载，没点名的卡不用」）与改前代码（参与的卡没开成拒、不参与的跳过），改前项目一直按这一读法；但只开参与的卡之后没点的卡根本不开，这半句就没有对象了。两种读法一种让代码违反条款、一种让条款留下空话，交主 agent 定 kb 措辞（`mount-parameters.md` 第 23 行写的是「参与的卡开不成（挂载那一刻点名的卡、挂着时切换点到的新卡）……挂载时都不成就拒绝挂载」，同样只说点名的卡）。我核过：改前那一句是 `git show HEAD:.claude/kb/decisions/26-后台整理与放置回收.md` 里抓出来的原文。

## 三、攻击面 3：关卡与泄漏账（设计问题 ⑦）

### 和卸载那一套是不是同一个规矩

判法与记账是同一份：关卡交 `gpus_dropped_leaking_those_still_running`（有等超过上限的提交就「std::mem::forget(gpu);」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:952`）），记账走同一个函数（「.note_the_gpus_leaked_when_dropped(&gpus_leaked);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2421`），卸载那一处是 `:3081`）。差两样：

- 条款授权不同（戊第 3 种）：卸载许泄漏，切换只写关掉。
- 报出来（癸，打中，轻）：卸载那一刻泄漏的同时记进 `gpus_leaked_when_dropped`（「self.gpus_leaked_when_dropped.note_the_drop(gpus_leaked);」（`crates/singlefs-background-gpu/src/background_gpu.rs:3082`））；切换时泄漏只进进程账，进程账在产品路径上没有对外读的地方（`GpuOpenerOfThisProcess` 里是私有字段，`bytes_leaked_on` 只在开卡时读）。切换关卡走了「泄漏」还是「析构」那一支，运行时看不出来，碰的是「4. **分支必须可观测**：运行时能看出走了哪一条。」（`.claude/rules/fs-design.md:127`）。

### 关掉之后再点它重开：泄漏账重复扣、漏扣（没打中）

推了三段：

1. 挂载开 X 时账从此前泄漏的 L0 起（「let bytes_leaked_by_earlier_mounts =」（`crates/singlefs-background-gpu/src/background_gpu.rs:1082`）），这一次挂载里一次提交超时占 B，切换关 X：占着 L0 + B、开卡时已算 L0，记进进程账的是 B（「.checked_sub(self.bytes_leaked_by_earlier_mounts)」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:699`）），账成 L0 + B。
2. 同一次挂载再点 X：开卡那一步现读进程账，新卡的账从 L0 + B 起；判装不装得下最小的一片也扣它（「memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:823`））。再关、再泄漏 C，记 C，账成 L0 + B + C。
3. 切换开卡那几次量代价出错而有提交在跑的（实现员用例 4 × 600）：每次开卡现读账，后一次的起点含前一次，各记各的增量。

另一份挂载同时在 X 上泄漏的那一片，晚于本次开卡记进账时不在本次的起点里，这是改前就写明的（`GpuMemoryLeakedInThisProcess` 文档「两次挂载同时开着时各自开卡那一刻都没算上对方的那一片」），不归这一轮。壬第 2 条那张孤儿设备同属这一类。

### 参与的卡按标识重找位置（没打中）

`close_the_cards` 先记下参与的卡的标识，关完按标识在剩下的卡里找（「.position(|card| card.discrete_gpu().identifier == *device)」（`crates/singlefs-background-gpu/src/background_gpu.rs:2428`）），次序照旧；`left_out_under_their_memory_limit` 与 `cost_not_measured_at_the_last_switch` 记的是标识不是下标。那一问里接上的卡放在末尾，不动已有下标；两处「只关这一次开成的卡」关的卡此刻都不在参与的卡里（它们开的时候没开着，此刻生效的那一组换上时它们还没开，点不到它们）。`note_the_gpu_opened_at_a_switch` 与 `note_the_gpu_closed_at_a_switch` 的 `expect` 的前提（只开没开的、只关开着的）在我试的交错里都成立。

## 四、攻击面 4：期限与挂载（设计问题 ②⑧）

- 挂载：开卡器每张卡的期限是连同重开的最长等待（`OpeningDeadlinesOfTheOpener::OF_THE_PRODUCT`），外层延到「.saturating_add(self.each_gpu.saturating_mul(gpus_to_open_counted))」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:63`）；内层每一次开卡至多 5 分钟（「match outcome_receiver.recv_timeout(opening_limit) {」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:303`）），间隔是 `std::thread::sleep` 60 秒。算式「.saturating_mul(self.reopenings_at_most.saturating_add(1))」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:343`）加 3 个间隔 = 23 分钟，与真走的 4 次开卡加 3 次睡一致。真走的比算式多一点点（每次起线程、`recv_timeout` 醒来与 `sleep` 多睡的那几毫秒），余量来自枚举那 5 分钟没用完的部分；只有枚举本身用到将近 5 分钟、且 N 张卡都次次到点时才会在最后那一次开卡交回之前撞外层期限。推的，没量；不算打中。没有哪一处还按单卡 5 分钟算整张卡（`grep` 过 `WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT` 与 `longest_wait_of_one_gpu` 在 `crates/` 下非测试文件的全部命中：开卡器、外壳开一张卡那一步、`waiting_for_the_gpu.rs` 自己）。
- 切换：开卡线程外面没有期限，重开做得满 3 次；每一次开卡同样至多 5 分钟（「WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,」（`crates/singlefs-background-gpu/src/background_gpu.rs:1092`）），等 wgpu 的 future 用这一组的等的上限（「let wait_limit = settings.wait_limit();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2351`））。⑧ 条款没写，挂载那一刻的 5 分钟搬到切换无害，成立。一个边：等的上限给得大（量代价是 1 次空跑加 5 次，每次至多等一个上限），量代价本身可能超过 5 分钟，卡没坏也次次到点、4 次都不成、GC 停下；挂载那一刻同样如此，不是这一轮新出的。
- 设计问题 ②：一张开不成的卡挡挂载至多 23 分钟、外层「枚举 5 分钟 + 卡数 × 23 分钟」，是重开 3 次的直接算术，条款没写挂载能被挡多久，成立；报给用户时要带上这个数。

## 五、检查单八格

### 走不到的理由二次查实：成立

- `batch_not_split_over_the_gpus` 的 `assert!` 改成两种（「"参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"」（`crates/singlefs-background-gpu/src/background_gpu.rs:2523`））：参与的卡变空的路只有三条——挂载那一刻全因显存上限不用（`left_out` 不空）、换上一组时全因显存上限或量代价出错不用（前者 `left_out` 不空，后者上面先交回）、`close_the_cards` 关卡（它只关此刻那一组没点的或这一次开成而没参与的，参与的卡不在里面）。「开不成」那一种在上面先交回。成立。
- 「.expect("只为点到没开的卡的那一组开卡：至少一张");」（`crates/singlefs-background-gpu/src/background_gpu.rs:2347`）：两处调用之前都判过不空（`:2224`、`:2309`）。成立。
- 开卡线程上锁那一处：见辛，会毒化但走不到这个 `expect`，前提是核心层借实现那把锁先毒化；代价是整个运行库报坏。按「走不到」成立，按条款推不出（辛）。
- `note_the_gpu_closed_at_a_switch`、`of_the_settings`、`failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in` 的 `expect`：前提分别是「只关开着的」「换上时点的卡都开着」「选 `gpu` 的那一组带着实现」，我试的交错里都成立。
- 实现员没列、这一次新加的 `note_the_gpu_opened_at_a_switch` 的 `expect`（「外壳只开没开的卡」）：同一时刻至多一次开卡在开、卡只在那一问里接上，成立。

### 类型与名字：不成立（一处）

重开次数、开卡次数、原因个数没有共用裸整数：重开次数是 `reopenings_at_most`，开卡次数由 `0..=` 与 `saturating_add(1)` 现算，原因个数是 `Vec` 的长度。diff 新加的行里没有 `_ =>`（`grep -nE '^\+.*\b_ =>' research/prompts/_c996-code-r1-diff.md` 退 1、零命中）。`expect` 都写了依赖哪条不变量。不成立的那一处：`GpuNotOpenedAfterTheReopenings` 文档写「1 + 重开次数 个」，起不了开卡线程时只有 1 个（庚）；这条不变量没有 `assert`，靠文档。

### 不兼容旧代码与旧格式：成立

`grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r1-diff.md` 退 1、零命中。`BackgroundGpuCompute` 新方法缺省交回 `None`：缺省用在不开卡的替身与测试假实现（`region_scan.rs`、验证包与运行库测试里的几份），它们本来就不开卡，「没有在开的卡」是它们的真实答案，不是为旧实现留的分支。

### 不新建判定库：成立

`grep -nE '^\+.*(File::create|fs::write|OpenOptions|rocksdb|create_dir|std::fs::)' research/prompts/_c996-code-r1-diff.md` 退 1、零命中。

### 所有任务多机可用：成立（核的是 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs` 一份）

卡从环境变量 `GPU_CARDS_BY_PRIORITY`、`HOSTS` 取，`nvidia-smi` 序号经现跑的 `--query-gpu=index,pci.bus_id` 对到 wgpu 的总线地址，这个进程在哪几张卡上经现跑的 `--query-compute-apps=gpu_uuid,pid` 与 `--query-gpu=index,uuid` 读；文件里没有写死的序号、UUID 或总线地址（`grep -nE 'GPU-[0-9a-f]{8}|\b0000:[0-9a-f]{2}:'` 在这份文件里命中 1 行，是第 199 行文档注释里讲去掉域那一句的例子，不是判法里写死的值）。第二台上 `nvidia-smi` 不在或读不成打 `NOT_RUN_NO_GPU_ADAPTER`，算本次未跑。没在第二台上跑过，推的。

### 测试优先多线程：成立

新用例一条一个场景；两条切换用例等开卡用 `recv_timeout(20 秒)` 的放开信号，不写死 `sleep`（diff 新加行里的 `sleep` 只有产品的重开间隔两处与一行文档）；`waiting_for_the_gpu` 三条重开用例共用一个脚本辅助函数、各给一份脚本，不是参数循环。

### 带 --item 的命令与项：不成立（名字逐字相同；漏罩的不止实现员写的两处）

26 个名字（第二份实现员报告第 177 行证红命令里的单引号串，去掉范围声明那一串）在 `crates/mutations.tsv` 第一列逐个 `grep -cxF`，26 个都恰好 1 次（草稿 `names26-counts.txt`）。没有变异行罩着的新行为，除实现员写的两处，我在 `crates/mutations.tsv` 第三列（原文）里按锚点逐个找、零命中的有：

| 行为 | 位置 | 改坏之后会怎样 |
|---|---|---|
| 点的卡都开着时清掉在等的那一组 | 「self.settings_waiting_for_the_opening = None;」（`crates/singlefs-background-gpu/src/background_gpu.rs:2225`） | 之后那一次开卡开完，换上的是被盖掉的旧那一组 |
| 上一次切换开卡没完时不另起 | 「if self.opening_at_a_switch.is_none() {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2230`） | 两条开卡线程同时开、结局覆盖 |
| 结局是为最近那一组开的才报 | 「if settings_of_the_opening == settings_waiting {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2302`） | 为被盖掉的那一组开不成也停 GC |
| 关掉这一次开成而没人要的卡 | `:2298`、`:2303` 两处 `close_the_cards(...devices_opened_by_this_opening...)` | 不参与的卡开着（正是 C996 的毛病） |
| 为最近那一组再开一次 | 「self.start_the_opening_at_a_switch(settings_waiting, discrete_gpus_to_open);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2313`） | 最近那一组永远等不到开卡 |
| 起不了开卡线程 | `OpeningAtASwitch::ThreadNotStarted` 两处 | 不报、不停 |

锚点我是按原文子串数的（`awk -F'\t' 'index($3,k)>0'`），换一种写法锚在别处的行我没逐行读。

### 新崩溃流多罩了什么：成立

开卡、关卡、重开都不写盘。切换开卡开不成在一批开头停 GC，之后那一批不开、有活意图就单发一次只删意图（`mounted_session.rs` 那一支停在「CompactionRunStop::BackgroundTaskStoppedAfterAnEarlierComputeError,」（`crates/singlefs-core/src/mounted_session.rs:5132`））；这个落盘形态改前就有（切到一张卡都用不上的设置、补跑腾出空间之后那一批开头停着都走它），这一轮只多了一个触发它的原因，没有新形态。那条路改前有没有崩溃流罩着不是这一轮的问题，我没核。

## 六、改法（我提的，只在我的模型上写过、没跑过，被攻过零轮）

- 甲（切换生效时刻）：核心层把「交给实现、实现还在开卡」记成第三种状态（已交、未生效），不把它当成已生效：这期间一批出错，照「出错之前已经收到、还没生效的切换请求」解除；实现的那一问在换上那一刻交回「换上了」，核心层到这时才算生效。
- 乙（换上那一刻再判一次）：那一问的交回改成三种——没动、换上了（连同换上之后 `batch_not_split_over_the_gpus` 的答案）、开不成；核心层收到「换上了」时照挂载那一刻的停法重新判停着（用得上就清掉交设置那一刻按旧那一组记下的停着，用不上就按 `GpuUnusableUnderTheSettingsHandedIn` 停）。同时修丙。
- 丁（陈旧结局）：结局按交设置的次序编号认，不按内容比；或者交来新的一组时把还没取走的结局当成「为更早一组开的」处理（开成的照接上、开不成的不报，照新那一组重开）。
- 戊（不关的三种）：切到 `cpu` 时核心层交给实现一个「一张卡都不参与」的信号（关掉全部、丢掉在等的那一组）；最近那一组开不成时关掉此刻那一组之外开着的全部卡，不只关这一次开成的；切换时有提交还在跑的卡记进「待关」，之后每批开头等它做完再关，不当场泄漏。第 1 种交用户：选 `cpu` 时要不要连着开着的卡一起关、那次切回 `gpu` 是否要重开。己与第 1 种同一处：切到 `cpu` 时交还了卡，切回时「切换前那组卡」就是空的，开着期间走 CPU 或停着，交用户选。
- 庚：起不了开卡线程也进重开循环（间隔之后再起，至多重开 3 次，每次一个原因）。
- 辛：开卡线程里接住 panic（`catch_unwind`），记成这一次开卡没开成的一个原因、照重开；不在会话锁里再 panic。
- 壬：同一张卡前一次开卡没回来（到点了线程还在驱动里）就不再起下一次，算这一次又没开成、照间隔往下数；或删掉开卡器文件头那句理由，由主 agent 定。
- 癸：切换时泄漏的卡同样报出来（例：`gpus_leaked_at_the_switches` 累计一份，或进每批的记账）。

| 改法 | 甲 | 乙 | 丙 | 丁 | 戊 | 己 | 庚 | 辛 | 壬 | 癸 |
|---|---|---|---|---|---|---|---|---|---|---|
| 甲的改法 | 推的，待主 agent 跑 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 乙的改法 | 不修 | 推的，待主 agent 跑 | 推的，待主 agent 跑 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 丁的改法 | 不修 | 不修 | 不修 | 推的，待主 agent 跑 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 戊的改法 | 不修 | 不修 | 不修 | 不修 | 推的（三种各修各的） | 推的（切 `cpu` 交还卡那一种） | 不修 | 不修 | 不修 | 不修 |
| 庚、辛、壬、癸的改法 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 推的 | 推的 | 推的 | 推的 |

甲与乙的改法改的都是核心层与实现之间那一问的形状，宜一起做；只做乙不做甲，甲照中（甲的停是一批出错停的，不是交设置那一刻判的）。

## 七、要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy` 已在 `/home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/.deliverables` 写成「交接」；里面只多了两份接在测试模块末尾的攻方腿用例（`cargo fmt` 过），产品代码一字没改，没编过。副本上量出的数注明是副本。

1. GPU 包 lib 测试目标里攻方腿那五条，在副本起（`--cwd` 是副本）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标里名字带 attack_ 的五条攻方腿用例（副本）; 范围=1 个测试目标; 为什么=c996-code-r1 攻方腿量切换开卡交错的读数，判决要引' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy -- cargo test -p singlefs-background-gpu --lib attack_ -- --nocapture
```

   看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `test result: ok. 5 passed`，与五行 `name=attack_stale_failure`（两行）、`name=attack_superseded_card`、`name=attack_left_out_after_the_opening`、`name=attack_failure_of_the_settings_before_the_switch`。哪一条 `FAILED`，就撤回那一条对应的打中（丁、戊第 2 种、丙的实现那一半、乙的实现那一半）；编不过的，把报错交回承接的腿修。

2. 核心层 lib 测试目标里攻方腿那五条，在副本起：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试目标里名字带 attack_ 的五条攻方腿用例（副本）; 范围=1 个测试目标; 为什么=c996-code-r1 攻方腿量切换开卡期间核心层停下与解除的读数，判决要引' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy -- cargo test -p singlefs-core --lib attack_ -- --nocapture
```

   看：`type=succeeded exit=0`；`test result: ok. 5 passed`；`name=attack_stop_outlives_the_switch`（两臂）、`name=attack_stale_stop`、`name=attack_left_out_after_the_opening`（两臂）。红了撤回对应的甲、乙、丙。

3. 判决要引的数由主 agent 复跑模型目录（新 shell 里起；运行目录先被包装删掉，只删工作根下以 `-run` 收尾的那一个）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿模型复跑：singlefs-background-gpu 与 singlefs-core 两个 lib 测试目标里名字带 attack_ 的用例; 范围=2 个测试目标; 为什么=判决要引的攻方腿读数由主 agent 在模型目录拷贝上复跑' task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy /home/fy5090/.local/share/singlefs/work/c996-code-r1-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/c996-code-r1-opus-model
```

   看：`task wait` 末行退 0；日志里 `name=rerun_exit background_gpu_lib=0 core_lib=0`，十行 `name=attack_…`，两行 `test result: ok. 5 passed`。照判决规矩，主 agent 先把模型目录拷进自己的草稿目录，把命令里的模型目录换成拷贝再交。范围声明种类若被 `run-scope-guard` 判成与命令形状不符，照它的出路改种类，不改命令。

探针的内存与时间：没跑，没有读数。

## 八、没打中的形状

- 停着的 GC 被悄悄放开：试了核心层能动停着那几条的全部入口（请求切换 `:1595`、换上待生效 `:1613`、那一问交回开不成 `:1620`、一批出错 `:1394`），在开卡开着、开不成、开成、切 `cpu`、又切同一组、停着时切换这几种次序下逐个走；放开只发生在一次通过校验的请求上。我造不出没请求就放开的交错。
- 泄漏账重复扣、漏扣：三段推演见第三节，同一次挂载里关了再开、开卡量代价出错连着几次、两次挂载交叉三种。
- 参与的卡下标错位：关卡、接卡、换上三处，见第三节。
- 一批之内换设置：换上只在一批开头与两个入口；`end_the_batch` 断言核心层那一组一批之内不变，实现的参与的卡只在那一问里动。
- 两条开卡线程同时开、结局互相覆盖：`:2230` 那一判挡住，`opening_at_a_switch` 至多一个（这一行没有变异行罩着，见第五节）。
- `note_the_gpu_opened_at_a_switch` 的 `expect` 被走到：要一张卡在开卡线程开着时被别处挪进「开着的」，别处只有那一问会挪，同一时刻至多一次开卡。
- 期限做不满 3 次重开：挂载与切换各查一遍（第四节）。

## 九、这条腿自己的限度

- 一条都没跑：十条模型用例没编过、没跑过，断言里的每一个读数都是按代码推的；编不过或红了，对应的打中照第七节撤回。
- 核心层五条用的是照 `BackgroundGpuCompute` 文档写的替身，不是真外壳；替身在乙、丙两格的那一对答法另由 GPU 包两条对着真外壳量（交设置那一刻答旧那一组、换上之后答新那一组；那一问交回 `None`）。甲没有对着真外壳的那一半：「切换开卡期间批在旧卡上算」靠实现员自己的用例 `the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch`。
- 用户动作放开扫过的：甲扫了「错在交设置之前 / 之后」，丁扫了「又交在那一问之前 / 之后」，丙扫了「点的卡开着 / 没开」，各一对对照臂；没扫的是多张卡、多次连环切换的组合，那几格只按代码推。
- 辛、壬靠真驱动里的 panic 与挂住，替身造不出，只按代码推。
- 子那一格我按 `git show HEAD:` 的改前原文与改前代码判读法，没读调度记录（禁读）。
- 背景材料第二节的验证数（任务 13924、13992、14014、14018）我没核、没引。

## 十、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在第七节）。
- 没读禁读清单里的另一条腿的报告与模型、`records/` 下的调度记录。
- 编译目录：没建。副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r1-opus/copy`（`du -sh` 54M）留给主 agent 跑第七节，已声明「交接」，用完由主 agent 删。草稿目录里另有 `names26.txt`、`names26-counts.txt`、`mutation-names.txt`、`fmt-check.txt`、`minimal-copy.txt` 与两份模型源码的格式化前后拷贝，都是普通文件。
