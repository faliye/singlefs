# c996-code-r2 攻方腿报告（Opus，2026-11-12）

立场：假设修补与实现员的读法是错的，造交错打穿它。攻正文第三节四条攻击面与第四节检查单八格，不重复第一轮已攻过的角度（第一轮的十条历史只按「今天该读出什么」重放一遍）。方案按 `crates/` 今天的实现谈，代码行号取主工作区此刻的行号（与开工快照 `research/prompts/c996-code-r2-snapshot/crates-sha256.txt` 列的 sha256 逐份对上，我核过七份）。kb 行号取 `.claude/kb/decisions/26-后台整理与放置回收.md` 自己的行号。

## 复跑命令与模型文件

模型是两份接进测试模块末尾的 lib 测试（替身卡、不写盘），没有崩溃状态枚举：这一轮的对象是开卡、关卡与核心层生效时刻的内存状态机，不落盘，第 3b 步的崩溃状态枚举不适用。一条都没编、没跑（子 agent 不交任务），下面每个读数都是「推的，待主 agent 跑」；只做过 `cargo fmt`（语法过了）。

复跑（交给服务；运行目录是工作根下名字以 `-run` 收尾的新目录）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿模型复跑：singlefs-background-gpu 与 singlefs-core 两个 lib 测试目标里名字带 attack_ 的用例; 范围=2 个测试目标; 为什么=c996-code-r2 判决要引的攻方腿读数由主 agent 在模型目录拷贝上复跑' task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/c996-code-r2-opus-model
```

| 文件 | sha256 |
|---|---|
| `research/prompts/c996-code-r2-opus-model/src/attack_on_the_repaired_switch_opening.rs`（接在 `background_gpu.rs` 测试模块末尾，9 条用例） | `11acb8750542646e6f5f82d93d3a9efca172e959b87cab42b63137878ccd83ee` |
| `research/prompts/c996-code-r2-opus-model/src/attack_on_the_repaired_core_answers.rs`（接在 `background_compute.rs` 测试模块末尾，9 条用例） | `543ece64b71177ab09d5a853b8862703d0548acdae4be6a251e962c6346fc294` |
| `research/prompts/c996-code-r2-opus-model/tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps` 那一段） | `04eada9680a51d1835b3725d50fc0a405e20b5270c64bdbbc9dca4b0adaf9c1f` |
| `research/prompts/c996-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

冻结副本就是我的仓副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy`（`minimal-copy.py` 从主工作区拷出，两份模型已接好并 `cargo fmt` 过；接进去之前那部分与主工作区逐字节相同，我用 `cmp` 核过），在 `.deliverables` 写成「交接」。每条用例的断言写的是本报告说的那种行为：用例过，那一格照本报告走；用例红，那一格撤回。

## 各格判定一览

| 号 | 格 | 判 | 违反的句子（kb 自己的行号）或依据 | 模型用例 |
|---|---|---|---|---|
| 新一 | 切到 `cpu` 之后切回 `gpu` 的卡开不成（设计问题 ③(a) 把那一组记成生效），用户再交同一组：下一批走 GPU，实现一张卡都没开，`batch_not_split_over_the_gpus` 的 `assert!` 在会话锁里 panic，运行库报坏 | 打中（重） | 检查单「走不到的理由」第一处的前提「核心层那时生效的选 `cpu`」不成立；第 4 条「重开 3 次都不成，GC 停下，报警……等人处理」（:253）之后用户照条款再切一次，得到的是整个运行库坏掉 | GPU 包 `attack_the_retry_after_a_switch_not_opened_following_the_cpu_panics_in_the_next_gpu_batch`；核心层 `attack_core_retry_after_not_opened_following_the_cpu_routes_the_batch_to_the_gpu` |
| 新二 | 同一段历史不先切 `cpu`：开不成之后核心层认生效的是新那一组，实现照旧开着、照旧用切换之前那张卡；再交同一组的开卡期间，批分给核心层生效那一组没点的卡 | 打中 | 第 2 条「切换之后不再参与的卡，切换生效时关掉；」（:251）；两边对「生效的那一组」认得不一样 | GPU 包 `attack_the_retry_after_a_switch_not_opened_runs_the_batch_on_the_card_switched_away_from`；核心层 `attack_core_retry_after_not_opened_on_the_gpu_routes_the_batch_to_the_gpu` |
| 新三 | 挂载那一刻开卡线程 panic：原样往上传、拒挂载，一次都不重开（切换开卡线程同一种 panic 照重开） | 打中 | 第 4 条「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开。」（:253），这一句跟在「挂载那一刻参与的卡开不成，或挂着时切换点到的卡开不成」之后，管挂载 | GPU 包 `attack_an_opening_panicking_at_the_mount_is_not_reopened_while_the_same_at_a_switch_is` |
| 新四 | 用户切走一张坏卡、新卡开卡期间：每一批照旧在坏卡上出错、每次都照「还没生效的切换」解除，至多 23 分钟里不停、不报警、过了补跑阈值也不在 CPU 上补跑 | 推不出（算打中） | 条款两句叠出来的：「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下。」与「重开期间照旧用切换前那组卡」（都在 :253）；条款没写开卡期间每批都出错怎么办 | 核心层 `attack_batches_failing_while_the_switched_card_opens_never_stop_nor_compensate` 与对照臂 |
| 壬 | 前一次开卡没回来就不再起：闸只管一次挂载之内，下一次挂载在同一张卡上照起 | 还中（缩小了） | 第 4 条「同一张卡前一次开卡没回来就不再起下一次，算这一次又没开成，照间隔往下数。」（:253）；开卡器文件头自己的理由「同一个驱动加载器里再起一条会不会也挂没量过」 | GPU 包 `attack_an_opening_hung_in_an_earlier_mount_does_not_keep_a_later_mount_from_starting_another` |
| 癸 / 戊 ③ | 切换关卡泄漏累计进 `GpusLeakedWhenClosedAtASwitch`，但产品挂载那条路（开卡器 → 运行库）把这份把手丢了，运行时照旧读不到 | 还中 | 第 4 条「切换关卡时有提交还在跑的卡照卸载同一个规矩，整张泄漏、不析构，并报出来。」（:253） | 推的（照代码读） |
| 甲 | 新卡开卡期间旧卡出错 | 修掉了（终局）；开卡期间的读法见新四 | | 核心层 `attack_repaired_stop_on_the_card_before_the_switch` 与对照臂 |
| 乙 | 交设置那一刻读到切换之前那一组的「用不上」 | 修掉了（终局）；实现员的回归用例把开卡期间开批钉成了对的，见第一节 | | GPU 包 `attack_repaired_failure_of_the_settings_before_the_switch`；核心层 `attack_repaired_stale_stop` |
| 丙 | 开完换上的那一组一张卡都用不上 | 修掉了 | | GPU 包 `attack_repaired_left_out_after_the_opening`；核心层两条 `attack_repaired_left_out_after_the_opening_*` |
| 丁 | 同一组在那一问之前又交一次 | 修掉了 | | GPU 包 `attack_repaired_stale_failure_*` 两臂 |
| 戊 ① | 切到 `cpu` 不关卡 | 修掉了 | | 推的（实现员用例钉了，我没另写） |
| 戊 ② | 为被盖掉的切换开成的卡 | 修掉了 | | GPU 包 `attack_repaired_superseded_card` |
| 己 | `cpu` 切回 `gpu` 开卡期间在切 `cpu` 之前那组卡上算 | 修掉了（随戊 ①）；但经设计问题 ③(a) 换个入口又出来了，见新二 | | 核心层 `attack_core_retry_*` 的开卡期间那一读 |
| 庚 | 起不了开卡线程 1 个原因、不重开 | 修掉了；另有一处小毛病（为被盖掉的那一次再起线程，第二节） | | 推的 |
| 辛 | 开卡线程 panic 在会话锁里再 panic | 切换那一路修掉了；挂载那一路见新三 | | 见新三 |
| 第一轮不下结论的四样 | 泄漏账重复扣或漏扣、下标错位、一批之内换设置、停着的 GC 被悄悄放开 | 没打中（单次抽样，不撑否定结论） | 第三节 | |
| 设计问题 ③(a) | 开不成时核心层把那一组记成生效 | 不成立（新一、新二都从这里来） | | 同新一、新二 |
| 设计问题 ③(b) | 叫停之后不睡间隔、正在开的那一次开完、开成的卡在线程上丢 | 成立（带一处小漏报，第三节） | | 推的 |
| 设计问题 ① | 五种「这一次没开成」共用 `DeviceOrPipelineDidNotStart` | 推不出（算打中）：成员自己的文档对其中四种是假话 | | 推的 |
| 检查单八格 | | 走不到 不成立；类型与名字 不成立；不兼容 成立；判定库 成立；多机 成立；多线程 成立；带 --item 不成立；新崩溃流 成立 | 第五节 | |

## 一、攻击面 1：第一轮打中的每一格，修补之后还中不中

第一轮十条模型用例的历史我在今天的接口上各写了一条（模型目录 `src/` 两份文件后半段，名字以 `attack_repaired_` 起头），断言写条款要的结局；核心层那五条的实现答法照产品外壳在同一段历史上答的摆（替身不按自己的规则答，答什么由用例逐批摆）。下表的「今天该读出」是推的，待主 agent 跑。

| 格 | 第一轮读数（任务 14158，判决原文） | 今天该读出（推的） | 判 | 依据 |
|---|---|---|---|---|
| 甲 | `after_the_error=LeftStopped … stopped=true` | `after_the_error=StopLiftedByTheSwitchRequestedBeforeTheError … route_once_opened=Some(Gpu) stopped=false devices_in_effect=Some([41])` | 修掉了（终局） | 「return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;」（`crates/singlefs-core/src/background_compute.rs:1418`），前面那一判多了交给实现而还在开卡的那一格（`crates/singlefs-core/src/background_compute.rs:1413-1417`） |
| 甲对照臂 | `StopLiftedByTheSwitchRequestedBeforeTheError stopped=false` | 同 | 没变 | 同上 |
| 乙 | `stopped_after_the_request=false batch_at_the_handing_in_begun=false … stopped=true` | GPU 包：交设置那一刻那一问答 `StillOpeningItsCards`，开完答 `PutInEffect { not_split_over_the_gpus: None }`、参与的是 41；核心层：`route_at_the_handing_in=Some(Gpu) route_once_opened=Some(Gpu) stopped=false` | 修掉了（终局） | 实现在等开卡就答还在开卡（`crates/singlefs-background-gpu/src/background_gpu.rs:2567` 那一判），核心层只在答换上了时照答案换停着的那几条 |
| 丙 | `reported_by_the_poll=None … batch_failed=true`；核心层 `batch_once_opened_begun=true stopped=false` | GPU 包：那一问答 `PutInEffect { Some(EveryParticipatingGpuLeftOut) }`；核心层：`route_once_opened=None`、停在 `GpuUnusableUnderTheSettingsHandedIn` | 修掉了 | 「not_split_over_the_gpus: self.batch_not_split_over_the_gpus(),」（`crates/singlefs-background-gpu/src/background_gpu.rs:2574`） |
| 丁 | `openings_when_handed_in_again=4 causes_reported=Some(4) openings_after_the_poll=4` | `openings_when_handed_in_again=4 causes_reported=Some(4) openings_after_the_poll=8` | 修掉了 | 「if settings_waiting.handing_in_number == handing_in_number {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2601`）：号不同的不报、照最近那一组重开 |
| 丁对照臂 | `openings=8` | 同 | 没变 | |
| 戊 ① | 切到 `cpu` 不告诉实现 | 核心层交 `cpu` 时调「(None, Some(gpu_compute)) => gpu_compute.lock().use_no_gpu_from_the_next_batch(),」（`crates/singlefs-core/src/background_compute.rs:856`），实现「self.close_the_cards(|_device| true);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2471`） | 修掉了 | 实现员用例 `no_gpu_taking_part_closes_every_open_card_and_the_next_settings_open_their_cards_again` 钉着，我没另写 |
| 戊 ② | `gpus_opened=[01, 41] … drops_of_forty_one=0` | 一问就答 `NotOpened(81)`，`gpus_opened=[01] participating=[01] drops_of_forty_one=1` | 修掉了 | 「self.close_the_cards_outside_the_settings_in_effect();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2603`） |
| 戊 ③ / 癸 | 切换关卡泄漏只进进程账 | 多了一份 `GpusLeakedWhenClosedAtASwitch`，但产品那条路读不到 | 还中 | 见下面「癸」 |
| 己 | 推不出 | 切到 `cpu` 时关掉全部，切回开卡期间走 CPU（核心层用例的 `route_while_opening` 那一读） | 修掉了；经设计问题 ③(a) 换个入口又出来（新二） | 第二节 |
| 庚 | 起不了线程 1 个原因、不重开 | 起不了线程记一个原因、隔间隔在之后的两批之间再起，合起来至多 4 次 | 修掉了（另有一处小毛病，第二节「新五」） | 「let interval_left = self」（`crates/singlefs-background-gpu/src/background_gpu.rs:2498`） |
| 辛 | 推不出 | 切换开卡线程上接住：「std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {」（`crates/singlefs-background-gpu/src/background_gpu.rs:1238`） | 切换那一路修掉了；挂载那一路没接（新三） | 第二节 |
| 壬 | 闸管不到切换开卡线程与重开之前没回来的那一次 | 一次挂载之内管到；跨挂载管不到 | 还中（缩小了） | 第二节「壬」 |

### 实现员收成的回归用例，断言的是条款要的结局还是照新实现现写的

- 甲、丙、丁、戊 ② 那几条断言的是条款的结局（解除、按同一个成员停、又交那一次照它重开、开成的那张关掉）。
- 乙的核心层用例 `a_mount_stopped_under_its_settings_is_not_stopped_again_while_the_switched_card_opens_nor_once_it_is_put_in_effect` 断言「交给实现那一刻实现还在开卡……不再停」时写的是 `batch_while_opening.is_ok()`：开卡期间这一批开了。它开在切换之前那一组上，那一组参与的卡一张都不剩，这一批交下去就报「一张都不剩」、照「还没生效的切换」解除、放弃、删意图，下一批再来一遍，直到新卡开完。条款「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」要的是「从下一批起用新设置」，新设置在开卡期间用不上；这一批开不开条款没写。这条用例把新实现的开卡期间那一读钉成了对的，不是条款推出来的（新四是同一格）。
- 核心层的替身 `GpuComputeOpeningCardsWhenTheTestSays` 只会答还在开卡与换上了，从不答开不成（`crates/singlefs-core/src/background_compute.rs:2065-2082` 的 `put_the_gpus_opened_since_the_settings_were_handed_in_in_effect` 只有这两支）。核心层「答开不成就把那一组记成生效」（设计问题 ③(a)）这一支在核心层用例里一次都没走到；新一、新二都落在这一支之后。
- GPU 包用例没有一条把「告知一张卡都不参与」「开不成」「再交」「开一批」四步连起来走。

### 癸 / 戊 ③：产品那条路读不到切换关卡的泄漏（还中）

`close_the_cards` 照卸载同一个规矩泄漏，另累计进对外读得到的那一份；交给调用方的把手只在 `OpenedBackgroundGpuCompute` 的字段里。产品挂载走开卡器：它把开成的那一份换成运行库收的样子时只拿走了把手，「gpu_compute: opened.gpu_compute,」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:184`），运行库收的结构只有两样，「pub report_of_the_opening: String,」（`crates/singlefs-mount-runtime/src/lib.rs:150`）连同 `gpu_compute`。`gpus_leaked_when_closed_at_a_switch` 在这一刻随 `opened` 丢掉，产品里没有第二个持有者。能读到它的只有直接调 `open_background_gpu_compute` 的那几处（e202 实验与测试）。条款「切换关卡时有提交还在跑的卡照卸载同一个规矩，整张泄漏、不析构，并报出来。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）的「报出来」在产品挂载上没做到，运行时照旧看不出切换关卡走了泄漏那一支（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。卸载那一份 `gpus_leaked_when_dropped` 在同一处同样丢掉，那是改前就有的，不归这一轮。

四句：分辨臂——分得开（直接调 `open_background_gpu_compute` 的读得到，走开卡器的读不到）；系统看得到——开卡器手里就拿着那份把手；满足的分句——「并报出来」；改法见第六节。

## 二、攻击面 2：修补带出的新状态

### 新一 切到 `cpu`、切回 `gpu` 开不成、再交同一组：下一批在会话锁里 panic（打中，重）

历史（每步指到许可它的那一句）：

1. 挂载 `gpu` 点 A（开着）。用户切到 `cpu`。下一批开头核心层交 `cpu`：「(None, Some(gpu_compute)) => gpu_compute.lock().use_no_gpu_from_the_next_batch(),」（`crates/singlefs-core/src/background_compute.rs:856`）；实现关掉全部开着的卡，并清掉两样：「self.devices_named_by_the_settings_in_effect.clear();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2469`）、「self.left_out_under_their_memory_limit.clear();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2470`）。实现此刻参与的卡为空、因上限不用的卡为空。条款许这一步：「从 gpu 切到 cpu 算全部卡都不再参与，切换生效时关掉全部开着的卡、丢掉还在开的那一组，之后切回 gpu 当成点到没开的卡，照第 4 条的重开规矩开，开卡期间在 CPU 上算。」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）。
2. 用户切回 `gpu` 点 B（没开；B 也可以就是 A，切 `cpu` 时都关了）。核心层交设置、实现开卡，开卡期间在 CPU 上算（修好的己）。
3. B 头一次与 3 次重开都开不成。实现收下结局、关掉生效那一组之外的卡（此刻那一组点的卡已清空，没有可关的），记下开不成：「self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2604`）。核心层收到开不成，照设计问题 ③(a) 把 B 那一组记成生效、停着：「self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));」（`crates/singlefs-core/src/background_compute.rs:1698`，上一行把 `in_effect` 换成 B）。条款：「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）。
4. 人处理：修好 B，再交同一组（或任何一组点到没开的卡；切 `cpu` 之后所有卡都没开）。请求那一刻解除停下：「self.replace_the_tasks_stopped(Vec::new());」（`crates/singlefs-core/src/background_compute.rs:1633`）；条款「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置；」（:253）。
5. 下一批开头核心层交 B、问实现：实现清掉开不成（`crates/singlefs-background-gpu/src/background_gpu.rs:2426`）、起开卡线程，答还在开卡；核心层记成「交给了实现、还在开卡」，生效的仍是第 3 步记下的 B（选 `gpu`）。这一批不停着，按生效的那一组定走法：「let requested_backend = self.in_effect.settings.background_compute;」（`crates/singlefs-core/src/background_compute.rs:1736`）→ 走 GPU。
6. 三段在实现上算：`plans_of_a_batch` 头一件事就问「match self.batch_not_split_over_the_gpus() {」（`crates/singlefs-background-gpu/src/background_gpu.rs:2920`）。开不成已在第 5 步清掉、量代价出错在第 1 步清掉，参与的卡为空、因上限不用的卡也为空，走到「"参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"」（`crates/singlefs-background-gpu/src/background_gpu.rs:2892`）那一句 `assert!`，panic。
7. panic 发生在维护线程持着会话锁、借着实现那把锁的时候：维护线程的 panic 记成运行库坏了，「shared_of_the_thread.note_the_failure(description.clone());」（`crates/singlefs-mount-runtime/src/lib.rs:749`），之后前台拿锁一律失败；实现那把锁毒化，核心层以后每一次借实现都 panic。

模型：GPU 包 `attack_the_retry_after_a_switch_not_opened_following_the_cpu_panics_in_the_next_gpu_batch` 对着真外壳走第 1、2、3、5、6 步（替身卡；开卡那一步头 4 次开不成、第 5 次起挂着等放开），读数该是 `not_opened=true gpus_opened_after_not_opened=[] still_opening_at_the_retry=true batch_panicked_with=Some("参与的卡一张都不剩只有两种：…")`。核心层 `attack_core_retry_after_not_opened_following_the_cpu_routes_the_batch_to_the_gpu` 照外壳那几步的答法（还在开卡 → 开不成 → 还在开卡）走核心层，读数该是 `route_while_opening=Some(Cpu(CpuRequestedByTheSettings)) route_once_not_opened=None devices_in_effect_once_not_opened=Some([41]) route_at_the_retry=Some(Gpu) stopped_at_the_retry=false handed_in_at_the_retry=true`。两半接在同一组答法上：核心层在第 5 步把这一批交给 GPU，外壳在同一刻一张卡都没开。

检查单「走不到的理由」第一处原文写的是「告知一张卡都不参与之后、再交来一组换上之前调它会走到；核心层那时生效的选 `cpu`」；文档同样写「之后、再交来一组换上之前调它（核心层这时生效的选 `cpu`，」（`crates/singlefs-background-gpu/src/background_gpu.rs:2870`）。第 3 步之后核心层生效的选 `gpu`、实现那一组没换上，前提不成立。

四句：分辨臂——分得开（对照臂不先切 `cpu`，同一段历史不 panic，见新二；把第 3 步改成开不成时生效的照旧是切换之前那一组，第 5 步走 CPU，也不 panic）；系统看得到——实现知道自己没换上任何一组（`devices_named_by_the_settings_in_effect` 为空），核心层看得到实现答的是开不成；满足的分句——条款没许运行库在「等人处理」之后的那一次切换上坏掉，最近的是第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」，这里的结局是整个挂载失效；改法见第六节。

用户最可能走到这条路：GPU 模式下临时切到 `cpu` 让出卡（别的任务要用），之后切回；切回点的那张卡这时正被占满或驱动出过错，4 次开不成、报警；用户处理完照报警再切一次。

### 新二 同一段历史不先切 `cpu`：开不成之后两边对「生效的那一组」认得不一样（打中）

第 1 步不切 `cpu`，其余同新一。第 3 步：实现关的是此刻生效那一组之外的卡：「self.close_the_cards_outside_the_settings_in_effect();」（`crates/singlefs-background-gpu/src/background_gpu.rs:2603`），生效那一组是它自己记的 `devices_named_by_the_settings_in_effect`，仍是 A；A 照旧开着、照旧参与；核心层却把 B 记成生效。第 5 步那一批走 GPU，实现分给 A。

模型：GPU 包 `attack_the_retry_after_a_switch_not_opened_runs_the_batch_on_the_card_switched_away_from` 读数该是 `gpus_opened_after_not_opened=[01] batch_panicked_with=None devices_given_work=[01]`；核心层 `attack_core_retry_after_not_opened_on_the_gpu_routes_the_batch_to_the_gpu` 读数该是 `route_at_the_retry=Some(Gpu)`、`devices_in_effect_once_not_opened=Some([41])`。

两种读法各打中一边：照设计问题 ③(a)，切到 B 在开不成那一刻生效，A 是「切换之后不再参与的卡」，「切换之后不再参与的卡，切换生效时关掉；」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）要关，实现没关、之后还给它分活（C996 的来由就是进程开着不参与的卡、服务记 wrong-card）；照另一读法（开不成时切换没生效），核心层不该把 B 记成生效，第 5 步那一批照「重开期间照旧用切换前那组卡」（:253）在 A 上算才对得上，可核心层这时以为跑的是 B。两边对生效的那一组认得不一样，只是停着那一段没有批，看不出来；用户一再切，就露出来。用户切走 A 多半正因为 A 坏了，这时第 5 步那一批在 A 上出错、照「还没生效的切换」解除，接上新四。

四句：分辨臂——分得开（新一与新二只差第 1 步；改成开不成时核心层照旧记切换之前那一组生效，两臂都与实现对上）；系统看得到——核心层收到的就是开不成；满足的分句——上面那两句；改法见第六节。

### 新四 新卡开卡期间每批在旧卡上出错：不停、不报警、不补跑（推不出，算打中）

历史：挂载点 A；A 开始出错（设备丢了、或次次等超过上限）；用户切到没开的 B 躲开它；B 开卡至多 23 分钟。这段时间每一批照「重开期间照旧用切换前那组卡，开成了，下一批起换成新设置；」（:253）在 A 上算、出错；每一次出错，交给了实现还在开卡的那一组都算「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下。」（:253），核心层答 `StopLiftedByTheSwitchRequestedBeforeTheError`、不记停着。结局：这至多 23 分钟里每一次叫起都开一批、在 A 上出错、放弃、删意图；一次都不停，所以不报警（报警只在停着时报，「那个后台任务停下，并在停着期间每次后台整理被叫起都报警」，:253），过了补跑阈值也不在 CPU 上补跑（补跑只看停着的，`whether_the_task_runs_the_next_batch` 头一判 `!self.is_stopped_after_a_compute_error(task)` 就照设置走）。

模型：核心层 `attack_batches_failing_while_the_switched_card_opens_never_stop_nor_compensate`（最满的那块盘 95%，过默认 85%），读数该是三批 `routes=[Some(Gpu), Some(Gpu), Some(Gpu)]`、`after_each_error` 三个都是 `StopLiftedByTheSwitchRequestedBeforeTheError`、`stopped=false`；对照臂 `attack_the_same_error_without_a_switch_stops_and_compensates_on_the_cpu`（同一个错、没有切换）读数该是 `after_the_error=LeftStopped compensating=true`。

条款没写开卡期间批批出错怎么办；第一轮之前（甲）的结局是停着到人再切，修补之后的结局是开卡期间空转。哪一种对，条款推不出；只差「请求那一刻」与「生效那一刻」之间这至多 23 分钟。乙那一格是它的特例：切换之前那一组一张卡都用不上，开卡期间每一批都报「一张都不剩」、解除、放弃（实现员的乙核心层用例把「这一批开了」钉成了对的，见第一节）。

四句：分辨臂——分得开（对照臂）；系统看得到——核心层知道这一组还在开卡、知道出错的是切换之前那组卡；满足的分句——两句条款各自满足，合起来推不出开卡期间该停还是该空转；交主 agent 定。

### 新一的另一个入口：开卡期间切到 `cpu` 再切回，前一次开卡还挂着

切 B 时 B 的头一次开卡挂在驱动里（到点 5 分钟算没开成，线程照旧挂着）；这期间用户切到 `cpu`：叫停，旧开卡线程不再开下一次，但它手里那一次停不下来。用户切回 `gpu` 点 B：新开卡线程先等旧线程放开 `opening_of_a_gpu_at_a_switch` 那把锁（「let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(」，`crates/singlefs-background-gpu/src/background_gpu.rs:2689`，旧线程开完手里那一次至多 5 分钟），然后 4 次都撞上「这张卡上前一次开卡到点之后还没回来」（`crates/singlefs-background-gpu/src/background_gpu.rs:1229`）→ 开不成，正好是新一第 3 步（切 `cpu` 时已清空），用户再切一次就 panic。这一条不用 B 真坏，只要一次挂住的开卡。推的，没写模型（要挂住的驱动）。

### 新五 起不了开卡线程之后被盖掉：为被盖掉的那一次再起线程（推不出，轻）

交 B（#1），起不了开卡线程，记成 `ThreadNotStarted { #1, [B], … }`；间隔到之前又交了 C（#2，点没开的卡）。实现只记下最近那一组、不另起（「if self.opening_at_a_switch.is_none() {」，`crates/singlefs-background-gpu/src/background_gpu.rs:2437`）。下一次那一问到了间隔，照 `ThreadNotStarted` 里记的 #1 与 [B] 再起线程（`crates/singlefs-background-gpu/src/background_gpu.rs:2491-2525`），先为没人要的 B 开卡（至多 4 次、23 分钟），开完才轮到 C。交来的是只点开着的卡的一组（当场换上）时同样照起，开出来的 B 收下之后关掉。`UnderWay` 那一种是改前就有的「等那一次开完再看」；`ThreadNotStarted` 那一种没有在开的线程，再起是为一个已作废的号新开一次。条款「重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」没管被盖掉的那一次；新卡因此晚开至多一整轮重开。推的，没写模型。

### 新六 为被盖掉的切换开成的卡，在最近那一组开卡期间一直开着（推不出，轻）

交 41（#1，在开），再交 81（#2，没开）。41 开成：「self.take_in_the_cards_opened_at_a_switch(outcome.opened);」（`crates/singlefs-background-gpu/src/background_gpu.rs:2596`）收下，号不同，接着为 81 开卡；81 这一轮至多 23 分钟里 41 开着、不在生效那一组里、不参与（81 开成时 `put_the_settings_in_effect` 关它，开不成时 `close_the_cards_outside_the_settings_in_effect` 关它，修好的戊 ②）。这段时间进程在一张谁都没点的卡上开着设备，服务那一侧会照 C996 的来由记 wrong-card。第 2 条「切换之后不再参与的卡，切换生效时关掉」只管生效那一刻；41 从没生效过，条款没写它。收下时只要最近那一组没点它就可以当场关。推的，没写模型。

### 逐个交错判过、没打中的

| 交错 | 走到哪 | 结局 |
|---|---|---|
| 开卡期间连着来两次、三次切换 | 每次交设置记新号、只留最近那一组（`crates/singlefs-background-gpu/src/background_gpu.rs:2433-2436`）；核心层的那一格同时被最近一次覆盖，问的也是最近那一组 | 两边认的都是最近那一组；被盖掉那几次开成的卡见新六 |
| 开卡期间切到 `cpu` 再切回 | 叫停、丢掉在等的那一组；切回当没开的卡开、开卡期间走 CPU | 对；旧线程手里那一次挂着时接上新一 |
| 开卡期间出错停下、再收到切换 | 出错那一刻照还在开卡的那一组解除（不停）；再请求只是把待交的那一组换掉 | 不停着；新四 |
| 叫停与开卡结局同时到 | 叫停先 `try_recv` 一次，取到就收下再关；没取到就丢掉收的那一端，开卡线程送不出去、在那条线程上丢（「let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);」，`crates/singlefs-background-gpu/src/background_gpu.rs:2703`） | 两种都关得上，没有漏关的卡 |
| 实现答换上了而核心层那一格已被新请求盖掉 | 新请求只进待交的那一格；一批开头先交、再问，问到的是刚交的那一组 | 答的与核心层记的同一组；没有错配 |
| 开不成之后补跑阈值 | 取用户最近那一次切换给的（设计问题 ③(a) 那一格的阈值） | 与第 4 条「挂着随切换请求一起换、从下一批起生效」相符 |
| 开不成之后 `in_effect()` 的读者 | 一批开头定走法（新一、新二）、一批结束断言一批之内没换、切换沿用实现那一处（取 `in_effect.gpu_compute`，开不成的那一组带的是同一份实现） | 只有定走法那一处读错 |

## 三、攻击面 3：从未被看过的地方，与第一轮没下结论的四样

### 新三 挂载那一刻开卡线程 panic：拒挂载、一次都不重开（打中）

修补只在切换开卡线程上接住 panic（`crates/singlefs-background-gpu/src/background_gpu.rs:1238`，调它的只有 `gpus_opened_at_a_switch`）。挂载那一刻开卡走 `gpus_opened_at_the_mount`，每一次开卡没包：「gpu_opened_with_the_reopenings(reopenings, &mut wait_before_a_reopening, || {」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:914`）。产品开一张卡那一步在另一条线程上起设备、编管线、量代价，那条线程 panic（wgpu 在起设备或编管线里 panic）时，等它的那一头照原样再 panic：「std::panic::resume_unwind(opening_thread.join().expect_err(」（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:311`），一路穿过重开循环到开卡器的工作线程，工作线程 panic，开卡器拒这一次挂载：「description: "开卡的工作线程 panic 了".to_string(),」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:350`）。一次都不重开。

条款：「挂载那一刻参与的卡开不成，或挂着时切换点到的卡开不成，都间隔 1 分钟重开，最多重开 3 次；」紧接着「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开。」（都在 `.claude/kb/decisions/26-后台整理与放置回收.md:253`）。这一句管挂载与切换两种开卡；挂载那一刻的「开卡线程 panic」没照重开。

模型：GPU 包 `attack_an_opening_panicking_at_the_mount_is_not_reopened_while_the_same_at_a_switch_is`：同一份产品开一张卡那一步（`opening_of_a_card_within_the_limit`，起卡那一件头一次 panic、之后开成），交挂载那一刻的开法读数该是 `name=attack_opening_panic arm=mount panicked=true starts=1`，交切换开卡线程的开法（每次包 `opening_with_its_panic_caught`、照重开）读数该是 `arm=switch … opened=true starts=2`。

四句：分辨臂——分得开（同一份开卡那一步，只差外面包没包）；系统看得到——panic 就在重开循环里冒出来；满足的分句——上面那一句「开卡线程 panic 都算这张卡这一次没开成，照重开」；改法见第六节。第一轮辛那一格把「挂载那一刻同一种 panic 只拒挂载」当成轻的那一边作对照，那时条款还没有这一句；第一轮之后写回的条款把挂载也管进来了，代码只改了切换那一边。

### 壬 前一次开卡没回来就不再起：一次挂载之内管到，跨挂载管不到（还中，缩小了）

闸是开一张卡那一步自己带的一份表：「let devices_with_an_opening_not_returned = Arc::new(Mutex::new(Vec::new()));」（`crates/singlefs-background-gpu/src/background_gpu.rs:1163`）。产品每一次挂载另造一份开一张卡那一步：「let mut open_a_gpu = opening_of_a_gpu_in_use(adapters, memory_leaked_in_this_process.clone());」（`crates/singlefs-background-gpu/src/background_gpu.rs:1068`），挂载那一刻用完交给切换（「Box::new(open_a_gpu),」，`crates/singlefs-background-gpu/src/background_gpu.rs:1096`）。表跟着挂载走，同一张卡上前一次挂载留下的那条挂在驱动里的线程，下一次挂载的表里没有。开卡器的进程级闸只盯工作线程，工作线程等完 4 次、拒了挂载就放开；开卡器文件头自己写明了射程「同一个理由管到一次挂载之内」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:10`），而同一行的理由是「同一个驱动加载器里再起一条会不会也挂没量过」（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:10`）——驱动加载器是进程级的，跨挂载同样是同一个。

历史：挂载点 X，X 头一次开卡挂住，后 3 次被闸住，约 8 分钟后拒挂载；用户重挂（或卸载一个开卡期间挂住的挂载再挂）：新一次挂载枚举、在 X 上再起一条，挂住的线程变两条。每重挂一次多一条。条款「同一张卡前一次开卡没回来就不再起下一次，算这一次又没开成，照间隔往下数。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）没把「前一次」限在一次挂载之内。

模型：GPU 包 `attack_an_opening_hung_in_an_earlier_mount_does_not_keep_a_later_mount_from_starting_another`：两份开一张卡那一步（各代表一次挂载），起卡那一件挂着等放开、每次至多等 50 毫秒。读数该是 `earlier_mount_first=Some(WaitForTheGpuExceededTheLimit…) earlier_mount_again=Some(DeviceOrPipelineDidNotStart{"这张卡上前一次开卡到点之后还没回来…"}) starts_within_the_earlier_mount=1 later_mount_first=Some(WaitForTheGpuExceededTheLimit…) starts_after_the_later_mount=2`。

四句：分辨臂——分得开（同一份里闸住、另一份里照起）；系统看得到——表是可以放进进程级那一份（开卡器已持有进程级的泄漏账与实例）；满足的分句——上面那一句的「前一次开卡」；改法见第六节。

### 别的从未被看过的地方

- `use_no_gpu_from_the_next_batch` 在选 `cpu` 挂载、从没选过 `gpu` 的会话上：运行库选 `cpu` 挂载不开卡、不带实现，核心层交 `cpu` 时 `(None, None)` 那一支什么都不做；切换时另递一份实现、选 `cpu` 的，那一份收到「一张卡都不参与」、关掉它开着的卡。没打中。
- 卸载与切到 `cpu` 同时发生：两件都要 `&mut` 实现，被会话锁排开；先切 `cpu` 再卸载，卸载那一刻没有在开的那一次。没打中。
- 叫停之后开卡线程上开成的卡丢掉时记不记账：开成的卡量代价的提交都已做完，照常析构，不用记；开卡途中量代价出错而提交还在跑的那张，在开卡线程上就泄漏、记进进程账（`started_gpu_with_its_cost_measured`），叫停之后这一次的原因没人收（「reason: "切换开卡叫停了（卸载或切到 cpu），不再开".to_string(),」所在那一族原因不报到哪里，`crates/singlefs-background-gpu/src/background_gpu.rs:1253`），这张泄漏的卡哪儿都不报。条款「并报出来」管的是切换关卡；这一张是开卡里泄漏的，条款没写，记一笔推不出、轻。
- 切到 `cpu` 关掉全部卡时有提交还在跑的卡：照 `close_the_cards` 泄漏、进进程账与切换关卡那一份（产品读不到，见癸）。同一张卡之后再点它重开，新卡的账从进程账起，再泄漏只记这一次的增量。没重复扣。

### 第一轮没下结论的四样（这一轮各推了一遍，单次抽样）

| 样 | 推了哪些形状 | 结论 |
|---|---|---|
| 泄漏账重复扣或漏扣 | 切 `cpu` 关卡泄漏后切回再开再泄漏；为被盖掉的切换开成的卡收下再关；叫停时取到的结局收下再关；开卡量代价出错泄漏之后被叫停 | 进程账不重不漏；漏「报」两处（癸、叫停那一张），不是账错 |
| 参与的卡下标错位 | `close_the_cards` 关完按标识重找位置；`use_no_gpu` 之后参与的为空；收下的卡接在末尾不动已有下标 | 下标没错位；错的是「参与的那一组」与核心层生效那一组对不上（新一、新二），不是下标 |
| 一批之内换设置 | 核心层生效那一组只在一批开头与两个入口那一问里动；实现参与的卡只在交设置、告知不参与与那一问里动；一批结束断言没换 | 没打中 |
| 停着的 GC 被悄悄放开 | 能动停着那几条的五处：请求切换解除、一批出错记停着、答换上了换成照答案判的、答开不成换成开不成、挂载那一刻。后两处只在有「交给了实现、还在开卡」那一格时走到；那一格有值时停着的一定是空的（请求那一刻解除、有那一格时出错不记停着），所以「换成照答案判的」不会把一条真停着的清掉 | 没打中 |

这四样都是一条腿抽一次样，照规矩不撑否定结论（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。

## 四、攻击面 4：产生结论的方法

### 设计问题 ①：五种「这一次没开成」共用一个成员（推不出，算打中）

`DeviceOrPipelineDidNotStart { reason }` 今天装着五种：真的起不了设备或编不了管线、起不了切换开卡线程（「reason: format!("起不了切换开卡的那条线程：{error}"),」，`crates/singlefs-background-gpu/src/background_gpu.rs:2719`）、开卡 panic（「reason: format!("开卡 panic 了：{said_by_the_panic}"),」，`crates/singlefs-background-gpu/src/background_gpu.rs:1245`）、前一次没回来不再起（「reason: "这张卡上前一次开卡到点之后还没回来（还挂在驱动里），这一次不再起".to_string(),」，`crates/singlefs-background-gpu/src/background_gpu.rs:1229`）、叫停（`crates/singlefs-background-gpu/src/background_gpu.rs:1253`）。成员自己的文档只说第一种：「/// 起不了设备或编不了管线（驱动不对、显存不够）。」（`crates/singlefs-background-gpu/src/background_gpu.rs:150`），对另外四种是假话。

程序里没有按这几种分开走的地方（我 `grep -rn DeviceOrPipelineDidNotStart crates/*/src` 看了全部命中：只有映射到核心层的同名成员与报出来，没有分支），所以「调用方分不分得开该做的决定」按程序答是：现在没有要分开的决定。按人答不是：报警里的四个原因是人「等人处理」时读的，「前一次还挂在驱动里」的出路只有重启进程（挂住的线程停不下来），「驱动不对、显存不够」的出路是修卡或换卡再切一次，「开卡 panic」多半是本包的 bug（见下一节）。人只能读 `reason` 的字。同一件事在开卡器那一层另有自己的成员：运行库的「AnEarlierOpeningIsStillHung,」（`crates/singlefs-mount-runtime/src/lib.rs:139`），调用方照它的出路办；挂载里每张卡那一层却把同一件事塞进「起不了设备」。推不出它算一个成员：检查单「类型与名字」那一格判不成立。

### 前一次开卡没回来就不再起：靠什么知道回没回来、永远不回来时怎样、重开怎么数

- 靠什么知道：开卡那一刻在表里记下这张卡，记号跟着交给那条线程的那一件走（「let _returned_when_this_ends = opening_not_returned;」，`crates/singlefs-background-gpu/src/background_gpu.rs:1181`），那一件跑完（含量代价）或起不了线程、或 panic 展开时丢掉记号、划掉这张卡（「.retain(|device| *device != self.device);」，`crates/singlefs-background-gpu/src/background_gpu.rs:1222`）。判得对；表跟着一次挂载走，跨挂载不认（壬）。
- 永远不回来：这一次挂载里这张卡永远开不成，之后每次切换点它都 4 次撞闸、约 3 分钟就报开不成（间隔照睡，开卡本身当场拒）；条款「等人处理」，人能做的只有重启进程。重挂能绕过闸（壬），绕过去就在驱动里多挂一条。
- 重开怎么数：撞闸的那一次算一次、记一个原因、照间隔往下数，与条款「算这一次又没开成，照间隔往下数」相符；起不了切换开卡线程的那几次也扣在头一张卡的重开次数里（`crates/singlefs-background-gpu/src/background_gpu.rs:1282`），合起来至多 4 次。没有共用裸整数（`HandingInNumber` 是包装类型，重开次数是 `u32`，原因个数是 `Vec` 的长度）。
- 划掉表里那一行的 `retain` 没有变异行罩着（第五节）：它改坏了，这张卡开过一次之后同一次挂载里再也开不成（切走再切回就撞闸）。

### 接住 panic 会不会把不该吞的 panic 也吞掉（推不出，记一笔）

切换开卡线程上包住的是整个「开一次卡」：照显存上限判装不装得下、产品开一张卡那一步（另一条线程上起设备、编管线、量代价，那条线程的 panic 在这里原样再 panic 出来）。里面不只是驱动：量代价那一段要在卡上建缓冲、记账（推的：`measured_cost_of_one_submission` 经这张卡的显存账建缓冲，我没逐行追到 `hold`），记账超过上限是本包自己的 bug，照断言 panic，「.expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致");」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:468`）；量代价出错泄漏那一支算账也有「.expect("账从此前泄漏的那一片起、这一次挂载里不放：占着的不比它少")」（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:700`）。这类不变量 panic 在切换开卡时被接住，记成「开卡 panic 了：本包在这张卡上占的缓冲超过了上限……」，照重开再撞 3 次，最后报成这张卡开不成、GC 停下、报警点名一张好卡。条款原话「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开。」（`.claude/kb/decisions/26-后台整理与放置回收.md:253`）不分驱动 panic 与本包断言，代码照字面做了，所以不算违反；但本包的 bug 被报成卡坏了，人照报警去修卡、换卡，修不好。条款推不出该不该分开，交主 agent。同一种断言在挂载那一刻照旧 panic、拒挂载（新三），两条路对同一个断言的处置不同。

接住之后的状态：`AssertUnwindSafe` 包着的那一次里，会被半改的只有进程泄漏账与「没回来的开卡」表两把锁，两处都用 `unwrap_or_else(PoisonError::into_inner)` 取锁，毒化不传开；替身卡与产品卡随展开丢掉。没找到接住之后留下坏状态的地方。

## 五、检查单八格

### 走不到的理由二次查实：不成立

- `batch_not_split_over_the_gpus` 的 `assert!`：实现员的理由是「告知一张卡都不参与之后、再交来一组换上之前调它会走到；核心层那时生效的选 `cpu`」。新一给出一段产品里调得到的历史：告知不参与 → 切回的那一组开不成（核心层照设计问题 ③(a) 把它记成生效，选 `gpu`）→ 再交一组、实现还在开卡 → 核心层按生效的那一组走 GPU → 三段的 `plans_of_a_batch` 头一件事就调它（`crates/singlefs-background-gpu/src/background_gpu.rs:2920`）→ 参与的为空、因上限不用的也为空 → `assert!` 走到。前提「核心层那时生效的选 `cpu`」被 ③(a) 打破。
- `put_the_gpus_opened_since_the_switch_in_effect` 里的 `resume_unwind`（`crates/singlefs-background-gpu/src/background_gpu.rs:2553`）：开一次卡已在开卡线程上接住；包着的范围之外，开卡线程上只剩本包自己几行（拼原因、扣重开次数、`opened.push`）与开卡线程开头那一处 `expect`（「"开卡那一步只在开卡线程上锁着用，开一次卡里的 panic 在锁里就接住了：锁没被毒化",」，`crates/singlefs-background-gpu/src/background_gpu.rs:2690`）。我没找到让这几行 panic 的输入。这一处成立。

### 类型与名字：不成立

- 新方法 `use_no_gpu_from_the_next_batch`（核心层特质缺省什么都不做、两份核那一层缺省同样）、`take_the_answer_on_the_settings_handed_in`；新类型 `GpusLeakedWhenClosedAtASwitch`、`LatestGpuSettingsHandedIn`（`StillOpeningTheirCards` / `PutInEffect { failure_under_the_settings }` / `NotOpened { failure }`）、外壳那一层 `LatestSettingsHandedIn`；`OpenedBackgroundGpuCompute` 新字段 `gpus_leaked_when_closed_at_a_switch`。次序号是包装类型 `HandingInNumber(u64)`（「Self(self.0 + 1)」，`crates/singlefs-background-gpu/src/background_gpu.rs:371`），不与重开次数（`u32`）、原因个数（`Vec` 长度）共用裸整数。
- `_ =>`：`grep -nE '^\+.*\b_ =>' research/prompts/_c996-code-r2-diff.md` 退 1、零命中（原样输出为空，`echo rc=$?` 打出 `rc=1`）。
- 新 `expect` 都写了依赖哪条不变量（`start_the_opening_at_a_switch` 的「只为点到没开的卡的那一组开卡：至少一张」，开卡线程那一处见上一格）。
- 不成立的一处：五种「这一次没开成」共用 `DeviceOrPipelineDidNotStart`，成员文档只写其中一种（第四节）；同一件事「前一次开卡还挂着」在开卡器那一层有自己的成员，在每张卡那一层没有。另：两个新的特质方法缺省都是「什么都不做 / 当场换上」，一个包装忘了转发，卡就静默不关、答错；今天三处包装（e202 两处、两份核那一层）都转了（我逐个看过 diff 里那三处），不算打中，记一笔。

### 不兼容旧代码与旧格式：成立

`grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r2-diff.md` 退 1、零命中。两批之间那一问的旧形状（交回 `Option<BackgroundGpuComputeFailed>`）在核心层特质、两份核那一层、e202 两个包装里都换成了三种答法，没有留着的转接。一处疑点：`failure_under_the_settings_handed_in` 核心层不再直接读，留着给特质那一问的缺省答法用，实现员报告写「验证包里那几份只覆写 `failure_under_the_settings_handed_in` 的替身不用改」；它同时还被 e202 直接调（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:7977`），不是只为旧替身留的，我判成立。

### 不新建判定库：成立

`grep -nE '^\+.*(File::create|fs::write|OpenOptions|rocksdb|create_dir|std::fs::)' research/prompts/_c996-code-r2-diff.md` 退 1、零命中。

### 所有任务多机可用：成立（推的，没在第二台上跑）

对实现员补丁 `~/.local/share/singlefs/work/c996-code-r1-fixes/patch/crates.patch` 的新增行跑 `grep -nE '^\+.*(env::var|nvidia-smi|std::fs::|File::create|fs::write|OpenOptions|rocksdb|create_dir|hostname|HOSTS|GPU_CARDS|127\.0\.0\.1|/tmp|/home)'`，退 1、零命中。新用例的卡都是现造的替身。

### 测试优先多线程：成立

补丁新增行里的 `thread::sleep` 只有两处，都在产品代码里（重开间隔、`UntilItEnds` 时睡到间隔）；`grep -cE '^\+.*thread::sleep'` 打出 `2`。新用例等开卡用通道的 `recv_timeout(20 秒)` 与开卡那一步的锁，不写死睡。那条约 80 秒才红的（「C996 产品实现两批之间那一问等开卡开完」）：变异把 `NotAtAll` 换成 `UntilItEnds`，那一问在用例线程上等开卡开完，开卡那一步每次等用例放开至多 20 秒、开 4 次，4 × 20 秒；绿的时候那一问当场交回，不等这 80 秒。是红的那一边的兜底超时，不是该拆的参数循环。推的，没跑。

### 带 --item 的命令与项：不成立（名字逐字相同；新分支有没罩着的）

41 个名字（实现员报告「要主 agent 跑的命令」末条里 `--item '…'` 的单引号串）在 `crates/mutations.tsv` 第一列逐个 `grep -cxF`：`cut -f1 names41-counts.txt | sort | uniq -c` 原样输出「     41 1」（草稿 `draft/names41-counts.txt`），41 个都恰好 1 次。第一轮点名的 8 处：实现员表里 7 处各有一行，「产品挂载传 `OF_THE_PRODUCT` 与 sleep」写测不到；我照读同意（替身走不到那一句）。

这一轮新加、我在 `crates/mutations.tsv` 第三列按原文子串（`awk -F'\t' -v k=… 'index($3,k)>0'`）找、零命中的分支：

| 行为 | 位置 | 改坏之后会怎样 |
|---|---|---|
| 「没回来的开卡」表里划掉这张卡 | 「.retain(|device| *device != self.device);」（`crates/singlefs-background-gpu/src/background_gpu.rs:1222`） | 开过一次的卡同一次挂载里再也开不成 |
| 起不了线程的次数扣在头一张卡的重开里 | 「.saturating_sub(u32::try_from(causes_before_this_card.len()).unwrap_or(u32::MAX)),」（`crates/singlefs-background-gpu/src/background_gpu.rs:1282`） | 一张卡开到 5 次以上；用例里头一次起不了、第二次就开成的那条测不出 |
| 叫停之后不再开、不再睡间隔 | `crates/singlefs-background-gpu/src/background_gpu.rs:1287`、`:1292` 两处 `called_off.load` | 不睡间隔那一处在用例里间隔为 0，改坏了也测不出 |
| 告知不参与时清掉生效那一组点的卡、因上限不用的卡 | `crates/singlefs-background-gpu/src/background_gpu.rs:2469`、`:2470` | 前一处改坏：切 `cpu` 时卡已全关，我没找到看得出的后果（可能是等价变异）；后一处改坏：切 `cpu` 之前因上限不用的卡留在账上，新一那一批不 panic、改报那几张早已关掉的卡「一张都不剩」 |
| 起不了线程到了间隔才再起 | 「let interval_left = self」所在那一判（`crates/singlefs-background-gpu/src/background_gpu.rs:2498-2502`） | 当场再起，间隔不是 1 分钟 |
| 核心层交 `cpu` 时作废还在开卡的那一格 | 「self.handed_in_while_the_implementation_opens_its_cards = None;」（`crates/singlefs-core/src/background_compute.rs:1660`） | 切到 `cpu` 之后照旧问那一格，实现答换上了、那一问里调 `batch_not_split_over_the_gpus`，走到它的 `assert!` |
| 补跑阈值取还在开卡的那一格 | 「.or(self」（`crates/singlefs-core/src/background_compute.rs:1528`） | 开卡期间读数报旧阈值 |
| 核心层在没有那一格时也问一次、答案不用 | 「let _answer_with_no_settings_waiting = self」（`crates/singlefs-core/src/background_compute.rs:1677`） | 为被盖掉的那一次开成的卡收不下、关不掉 |
| 答开不成时把那一组记成生效 | `crates/singlefs-core/src/background_compute.rs:1697`（那一行 `self.in_effect = handed_in;`；同样的一句在 `:1689` 有一行换锚点的旧变异罩着，`:1697` 没有） | 设计问题 ③(a) 的那一读没有用例钉着 |

锚点我是按原文子串数的，换一种写法锚在别处的行我没逐行读。

### 新崩溃流多罩了什么：成立（推的）

新四开卡期间每批出错解除、放弃、删意图：走的是改前就有的「出错之前挂着切换、当场解除」那一支（`BatchAbandonedAfterABackgroundComputeError` 带 `StopLiftedByTheSwitchRequestedBeforeTheError`），落盘的仍是那一次只删意图的发布，只是次数变多，没有新形态。新一的 panic 发生在一批算到一半、发布之前，盘上留的是那一批开头之前的状态（有活意图的话意图还在），重挂之后照可写挂载续做；这与在那一刻崩溃同形，崩溃点重放的枚举覆盖「意图发了、批没发」那一类状态。停下、解除、只删意图那一次发布的落盘时机这一轮没变（核心层改的只是「生效」记在哪一格，不碰发布）。都是照代码读，没核已有崩溃流具体罩了哪几格。

## 六、改法（我提的，只在我的模型上写过读数、没实现、没跑，被攻过零轮）

- 改法一（新一、新二，设计问题 ③(a)）：实现答开不成时，核心层不把那一组记成生效：清掉「交给了实现、还在开卡」那一格，生效的照旧是切换之前那一组，有 GPU 路径的后台任务按开不成那个错停着；补跑阈值照第 4 条「挂着随切换请求一起换」另记一格取请求给的。这样新一第 5 步生效的是 `cpu`、那一批走 CPU；新二第 5 步生效的是 A、那一批在 A 上算，与实现、与「重开期间照旧用切换前那组卡」都对得上。
- 改法二（新三）：把接住 panic 挪进产品开一张卡那一步（`opening_of_a_card_within_the_limit` 里包住等那条线程的那一下），挂载与切换共用；或在 `gpus_opened_at_the_mount` 的每一次开卡外面同样包 `opening_with_its_panic_caught`。
- 改法三（壬）：「没回来的开卡」表从每次挂载各一份挪到进程级的一份（开卡器已持有进程级的泄漏账与实例，同一处多持一份），挂载与切换、各次挂载共用。
- 改法四（癸 / 戊 ③）：运行库收的 `OpenedBackgroundGpu` 带上切换关卡的泄漏把手（连同卸载那一份），或随每批的记账报出这一批开头切换关卡泄漏了哪几张。
- 改法五（新四，交主 agent 定条款）：开卡期间一批出错不当场解除，记成「停着、等交给实现的那一组生效时解除」：这期间照停着报警、照阈值补跑，实现答换上了那一刻解除（答开不成照开不成停）。条款「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」要跟着改写成「……照它生效的那一刻解除」，是条款改动，不是代码改法能单独定的。
- 改法六（设计问题 ①）：起不了开卡线程、开卡 panic、前一次没回来、叫停各起一个原因成员（`memory_limit_and_chunks.rs` 的穷举 match 与核心层 `GpuFailureOnTheDevice` 跟着加），`DeviceOrPipelineDidNotStart` 的文档回到它只装的那一种；本包断言的 panic 与驱动 panic 分不分开交主 agent 定。
- 改法七（新五、新六）：`ThreadNotStarted` 到了间隔、而最近那一次交来的号已经不同时，不为旧号再起，照最近那一组起；收下为被盖掉的切换开成的卡时，最近那一组没点它就当场关。
- 改法八（检查单「带 --item」）：第五节表里九处各补一行变异与用例；`:2469` 那一行先判是不是等价变异。

| 改法 | 新一 | 新二 | 新三 | 新四 | 壬 | 癸 | ① | 新五 | 新六 | 带 --item |
|---|---|---|---|---|---|---|---|---|---|---|
| 一 | 推的 | 推的 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 二 | 不修 | 不修 | 推的 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 三 | 不修 | 不修 | 不修 | 不修 | 推的 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 四 | 不修 | 不修 | 不修 | 不修 | 不修 | 推的 | 不修 | 不修 | 不修 | 不修 |
| 五 | 不修 | 不修 | 不修 | 推的（条款先改） | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 |
| 六 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 推的 | 不修 | 不修 | 不修 |
| 七 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 推的 | 推的 | 不修 |
| 八 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 不修 | 推的 |

每一格「推的」都是按代码推、没实现没跑；表里没有「量过」。改法一只修新一、新二，新四在改法一之后照样中（开卡期间生效的仍是切换之前那一组，出错照样解除）。新一另有一条更窄的改法（`batch_not_split_over_the_gpus` 参与的为空时报错不断言），它只把 panic 换成一个错，新二照中，不推荐。

## 七、要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy` 已在 `/home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/.deliverables` 写成「交接」；里面只多了两份接在测试模块末尾的攻方腿用例（`cargo fmt` 过），产品代码一字没改，没编过。副本上量出的数注明是副本。

1. GPU 包 lib 测试目标里攻方腿那 9 条，在副本起（`--cwd` 是副本）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标里名字带 attack_ 的 9 条攻方腿用例（副本）; 范围=1 个测试目标; 为什么=c996-code-r2 攻方腿量修补之后切换开卡与挂载开卡的读数，判决要引' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy -- cargo test -p singlefs-background-gpu --lib attack_ -- --nocapture
```

   看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `test result: ok. 9 passed`，与 `name=attack_retry_after_not_opened`（两行）、`name=attack_opening_panic`（两行）、`name=attack_hung_opening_across_mounts`、`name=attack_repaired_stale_failure`（两行）、`name=attack_repaired_superseded_card`、`name=attack_repaired_left_out_after_the_opening`、`name=attack_repaired_failure_of_the_settings_before_the_switch`。哪一条 `FAILED`，就撤回那一条对应的格（新一 / 新二 / 新三 / 壬 / 丁 / 戊 ② / 丙 / 乙的「修掉了」）；编不过的，把报错交回承接的腿修。新一那条用例过的时候日志里会有一段 panic 的原话（`catch_unwind` 接住的那一下），不是失败。

2. 核心层 lib 测试目标里攻方腿那 9 条，在副本起：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试目标里名字带 attack_ 的 9 条攻方腿用例（副本）; 范围=1 个测试目标; 为什么=c996-code-r2 攻方腿量修补之后核心层照三种答法走的读数，判决要引' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy -- cargo test -p singlefs-core --lib attack_ -- --nocapture
```

   看：`type=succeeded exit=0`；`test result: ok. 9 passed`；`name=attack_core_retry_after_not_opened`（两行）、`name=attack_errors_while_opening`（两行）、`name=attack_repaired_stop_outlives_the_switch`（两行）、`name=attack_repaired_stale_stop`、`name=attack_repaired_left_out_after_the_opening`（两行）。红了撤回对应的新一 / 新二的核心层那一半、新四、甲 / 乙 / 丙的「修掉了」。

3. 判决要引的数由主 agent 复跑模型目录（新 shell 里起；包装先删掉那一个运行目录，只删工作根下以 `-run` 收尾的那一个）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿模型复跑：singlefs-background-gpu 与 singlefs-core 两个 lib 测试目标里名字带 attack_ 的用例; 范围=2 个测试目标; 为什么=c996-code-r2 判决要引的攻方腿读数由主 agent 在模型目录拷贝上复跑' task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/c996-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy /home/fy5090/.local/share/singlefs/work/c996-code-r2-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/c996-code-r2-opus-model
```

   看：`task wait` 末行退 0（退非 0 的件数为 0 算过）；日志里 `name=rerun_exit background_gpu_lib=0 core_lib=0`，两行 `test result: ok. 9 passed`，18 行 `name=attack_…`。照判决规矩，主 agent 先把模型目录拷进自己的草稿目录，把命令里的模型目录换成拷贝再交。范围声明种类若被 `run-scope-guard` 判成与命令形状不符，照它的出路改种类，不改命令。

探针的内存与时间：没跑，没有读数。GPU 包那几条用例里最长的等是开卡那一步的兜底超时（20 秒，只在用例红的时候等满）；绿的时候每条在 1 秒量级（推的）。

## 八、没打中的形状

- 停着的 GC 被悄悄放开：五处能动停着那几条的入口在第三节表里逐个走过；「交给了实现、还在开卡」那一格有值时停着的一定为空，答换上了那一刻的替换不会清掉一条真停着的。
- 两边对「最近交来的那一组」认得不一样：一批开头先交、再问，问到的是刚交的；被盖掉的几次只在实现里留下开成的卡（新六），不进核心层。
- 叫停与开卡结局同时到：取到就收下再关，没取到就丢掉收的那一端、卡在开卡线程上丢；没有漏关、没有重复关。
- 一批之内换设置：核心层生效那一组、实现参与的卡都只在两批之间动。
- 泄漏账重复扣或漏扣、参与的卡下标错位：第三节表里的形状。
- `use_no_gpu_from_the_next_batch` 在选 `cpu` 挂载、没带实现的会话上：那一支什么都不做。
- 起不了开卡线程的次数与线程里重开次数合起来超过 4：扣在头一张卡上，合起来至多 4（`crates/singlefs-background-gpu/src/background_gpu.rs:1282`）；头一张开成、后面的卡不扣，各自 4 次，对。

取样范围：交错按代码手推，模型只落了第二节、第三节点名的那几段历史（每段一对臂）；没有随机跑批，没有多张卡多次连环切换的组合扫描。

## 九、这条腿自己的限度

- 一条都没跑：18 条模型用例没编过，读数都是按代码推的；编不过或红了，对应的格照第七节撤回。
- 核心层那几条用的是照用例逐批摆答法的替身，不是真外壳；它摆的答法是我按真外壳代码推的同一段历史上的答，GPU 包那一半对着真外壳（替身卡）量同一段历史，两半接在同一组答法上，中间没有一条把核心层与真外壳接起来跑的用例（核心层不依赖 GPU 包，接不起来；要接得走验证包或运行库那一层，我没写）。
- 用户动作放开扫过的：新一、新二扫了「先切 `cpu` / 不切」；新四扫了「有切换在开卡 / 没有切换」；新三扫了「挂载那一刻 / 切换开卡线程」；壬扫了「同一次挂载 / 下一次挂载」。没扫：切回时点的是 A 本身还是别的卡、再交的是同一组还是别的组（按代码推结局相同：切 `cpu` 之后所有卡都没开，任何点到卡的一组都走同一条路）。
- 新三、壬、新一的另一个入口靠真驱动里的 panic 与挂住，模型用替身的 panic 与挂着等放开代替。
- 新四判「推不出」：两种结局（停着到人再切、开卡期间空转）哪种对，要主 agent 照条款定，不是我能判的。
- 实现员报告与补丁我读了（不在禁读清单里）；禁读清单里的另一条腿的报告与模型、别的会话那一轮的材料、`records/` 下的调度记录没读。

## 十、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在第七节）。
- 没改仓里任何代码；写的只有本报告、模型目录 `research/prompts/c996-code-r2-opus-model/`（两份用例、两份脚本）与草稿目录。
- 崩溃状态枚举：这一轮的模型不落盘，第 3b 步不适用，没调枚举函数，跑的状态数 0。
- 编译目录：没建。副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r2-opus/copy`（`du -sh` 54M）留给主 agent 跑第七节，已声明「交接」，用完由主 agent 删。草稿目录 `draft/` 里是普通文件（模型源码的格式化前后两份、`rerun-steps.sh`、`names41.txt`、`names41-counts.txt`、`mutation-names.txt`、`minimal-copy.txt`、`fmt.txt` 与两份格式化差异）。
