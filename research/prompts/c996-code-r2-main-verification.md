# c996-code-r2：判决（2026-11-12）

正文 `research/prompts/_c996-code-r2-body.md`，开工快照 `research/prompts/c996-code-r2-snapshot/`。这是这一题代码轮的第二轮，第一轮判决 `research/prompts/c996-code-r1-main-verification.md`。

这一轮的正文、快照与两条腿是做门禁与判定入库的那个会话开的；用户 2026-11-12 定这块产品代码交给做 background-gpu 审核四条的会话收尾（交接单 `~/.local/share/singlefs/work/c996-handover/handover.md`，用户在接手方弹窗选「是，由这个会话接」），判决由接手方写。写判决之前拿快照 `sha256sum -c`：`crates-sha256.txt` 十份都对得上，与 `research/prompts/bggpu-audit-fix-code-r1-snapshot/crates-sha256.txt` 里同名的十份逐份相同。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/c996-code-r2-sonnet-output.md` | 第一轮甲、乙、丙、戊 ①、庚、壬「判得住」，丁「判得住，带一处条款没写」，戊 ②、辛「推不出」，癸「措辞过头」；九条改法都做了，辛只做在切换那一边、癸在产品路径读不到、新分支 6 处没有变异行；设计问题 ③(a)「条款只撑得住一半」；检查单「带 --item」不成立，其余成立 |
| 攻方（Opus） | `research/prompts/c996-code-r2-opus-output.md`，模型 `research/prompts/c996-code-r2-opus-model/` | 新一（切 cpu、切回开不成、再交：下一批在会话锁里 panic）打中、重；新二（不先切 cpu：两边对生效那一组认得不一样）、新三（挂载那一刻开卡 panic 不重开）打中；新四（开卡期间旧卡批批出错不停不报警）推不出；壬、癸还中；甲、乙、丙、丁、戊、己、庚修掉了；检查单「走不到的理由」「类型与名字」「带 --item」不成立 |

核查：

- 交出方做过的（交接单第五节，原话）：快照树任务 14650「清单点名 12 份，倒推出 12 份，sha256 对上 12 份」；攻方 18 条模型用例在模型目录拷贝上复跑任务 14649「name=rerun_exit background_gpu_lib=0 core_lib=0」，两行「test result: ok. 9 passed」。
- 接手方自己做的：快照十份此刻与工作区相同，所以引文直接对冻结副本核，命令原样：

```
python3 scripts/agents/cite-check.py research/prompts/c996-code-r2-opus-output.md research/prompts/c996-code-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1/repo --background research/prompts/_c996-code-r2-background.md
```

  末行「✓ 核了 92 处引文、47 处裸行号，对上 92 处，没判 47 处（其中裸行号没有引文可比 46 处；逐处列在上面）」。
- 任务 14649 的读数行接手方从日志里原样取出，留在 `research/prompts/c996-code-r2-main-checks/attack-model-lines-task-14649.txt`（25 行）；模型目录四份文件此刻的 sha256 与攻方报告开头那张表相同。这一趟是交出方交的，接手方没有重交：交出方那份带模型的仓副本已删，读数取自它的任务日志。
- 接手方在冻结副本里读过、判决要靠的行：`crates/singlefs-background-gpu/src/background_gpu.rs:2890`–`:2893`（`batch_not_split_over_the_gpus` 的断言）；`crates/singlefs-core/src/background_compute.rs:1696`–`:1698`（答开不成时 `self.in_effect = handed_in`）、`:1736`（一批的走法读 `in_effect`）；`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:311`（挂载那一路开卡线程 panic 原样再 panic）；`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:184`（开卡器只留 `gpu_compute`）。
- 核不动的：真驱动里的 panic 与挂住（模型用替身代替）；wgpu 的 `Queue` 析构不带上限地等（两条腿都只引代码注释与源码，没在真卡上量）。

## 二、判决

| 格 | 判决 | 依据 |
|---|---|---|
| 第一轮甲、乙、丙、丁、戊 ①②、己、庚 | 修掉了 | 任务 14649：`name=attack_repaired_stop_outlives_the_switch arm=error_after_the_handing_in … after_the_error=StopLiftedByTheSwitchRequestedBeforeTheError … stopped=false`；`name=attack_repaired_stale_stop … stopped=false`；`name=attack_repaired_left_out_after_the_opening arm=card_not_open_at_the_switch … stopped_by_gpu_unusable_under_the_settings_handed_in=true`；`name=attack_repaired_stale_failure arm=handed_in_again_before_the_poll openings_when_handed_in_again=4 causes_reported=Some(4) openings_after_the_poll=8`；`name=attack_repaired_superseded_card … drops_of_forty_one=1`。丁的「每次交设置各数 4 次」这一读被用户定案推翻，见新七 |
| 新一：切到 cpu、切回 gpu 点的卡试满都不成、人再切一次，下一批在会话锁里 panic | 打中（重） | `name=attack_retry_after_not_opened arm=switched_to_the_cpu_first not_opened=true gpus_opened_after_not_opened=[] still_opening_at_the_retry=true batch_panicked_with=Some("参与的卡一张都不剩只有两种：…")`，核心层那一半 `switched_to_the_cpu_first=true … devices_in_effect_once_not_opened=Some([…52, 49…]) route_at_the_retry=Some(Gpu)`。根子：实现答开不成时核心层把那一组记成生效（设计问题 ③(a)） |
| 新二：同一段历史不先切 cpu，批分给已切走的旧卡 | 打中 | `name=attack_retry_after_not_opened arm=stayed_on_the_gpu … batch_panicked_with=None devices_given_work=[…48, 49…]`；核心层认生效的是新那一组，实现照旧用切换之前那张卡 |
| 新三：挂载那一刻开卡线程 panic 不重开 | 打中 | `name=attack_opening_panic arm=mount panicked=true starts=1`，对照 `arm=switch panicked=false opened=true starts=2`；D26（后台整理与放置回收） 已定项 13 第 4 条「起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开」管挂载与切换两边；辩方腿独立报了同一处，接手方自己那一轮的攻方腿也报了 |
| 新四：新卡开卡期间旧卡批批出错，不停、不报警、不补跑 | 打中（用户已定该怎样） | `name=attack_errors_while_opening arm=switch_opening routes=[Some(Gpu), Some(Gpu), Some(Gpu)] after_each_error=[StopLifted…, StopLifted…, StopLifted…] stopped=false`，对照 `arm=no_switch … after_the_error=LeftStopped compensating=true`。条款两句叠出来、没写这一段；用户 2026-11-12 定「停下并报警，新卡开成自动解除」 |
| 新七：同一张卡试满 4 次之后人再切到它 | 与用户定案相反 | 现行代码每次交设置各数 4 次（`openings_after_the_poll=8`，回归用例钉的就是 8）；用户 2026-11-12 定「不再试，要卸载重挂」 |
| 壬：前一次开卡没回来就不再起，只管一次挂载之内 | 还中（缩小了） | `name=attack_hung_opening_across_mounts … starts_within_the_earlier_mount=1 … starts_after_the_later_mount=2` |
| 癸：切换关卡的泄漏在产品路径上读不到 | 还中 | 两条腿读码一致：开卡器把开成的结局换成运行库收的样子时只留把手与一段报告字符串，新累计的那一份在产品路径上没有读者 |
| 设计问题 ③(a)：开不成时核心层把那一组记成生效 | 不成立 | 新一、新二都从这里来 |
| 设计问题 ③(b)：叫停之后正在开的那一次照旧开完、开成的卡在那条线程上丢 | 成立 | 两条腿一致；已经睡着的那一觉叫不醒（至多一个间隔），这期间核心层生效的是 cpu，不出错 |
| 设计问题 ①：几种「这一次没开成」共用一个成员 | 打中 | 成员文档只说起不了设备或编不了管线，对起不了线程、开卡 panic、前一次没回来、叫停四种是假话；人照报警处理时出路不同（前一次还挂在驱动里只能重启进程） |
| 新五、新六：起不了线程之后被盖掉的那一次再起；为被盖掉的切换开成的卡在最近那一组开卡期间一直开着 | 打中（轻，读码，没有模型） | 攻方读码推的，辩方条 2 独立报了前一处；后一处让进程在谁都没点的卡上开着设备，正是 C996 的来由 |
| 接住 panic 把本包自己的断言也记成开不成 | 照条款字面，不改 | 条款不分驱动 panic 与本包断言；原话进了原因，没有吞掉。列进交用户的 |
| 第一轮没下结论的四样（泄漏账重复扣或漏扣、下标错位、一批之内换设置、停着的 GC 被悄悄放开） | 两条腿各推一遍都没打中 | 两条腿各一次读码，照规矩不撑否定结论；下一轮接着攻 |
| 检查单：走不到的理由二次查实 | 不成立 | 新一走到了 `batch_not_split_over_the_gpus` 的断言 |
| 检查单：类型与名字 | 不成立 | 设计问题 ① |
| 检查单：带 --item 的命令与项 | 不成立 | 41 个名字与变异表逐字相同；新分支九处没有变异行（攻方第五节表，辩方条 9 报了其中六处） |
| 检查单：不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、新崩溃流多罩了什么 | 成立 | 两条腿一致 |

## 三、改法

都是腿提的或接手方照用户定案推的，零轮。A/B 里没有改法的那一臂是任务 14649 的读数；实现员把模型用例收成回归用例，改完之后在同一批输入上读数翻面才算换到东西，任务号由合入后验证补在各条里。与 `research/prompts/bggpu-audit-fix-code-r1-main-verification.md`「三、改法」第 1、2、3、5 条合成一批，一个实现员一次改完。

1. 新一、新二：实现答开不成时，核心层不把那一组记成生效，清掉「交给了实现、还在开卡」那一格，生效的照旧是切换之前那一组，有 GPU 路径的后台任务按开不成那个错停着；补跑阈值另记一格取请求给的。回归用例取模型两臂：先切 cpu 那一臂下一批走 CPU、不 panic，不切那一臂核心层与实现认的是同一组。零轮。
2. 新三：接住 panic 挪进挂载与切换共用的开一张卡那一步，挂载那一刻开卡线程 panic 同样算这一次没开成、照重开。回归用例取模型两臂：`arm=mount` 翻成不 panic、`starts=2`。零轮。
3. 新四（用户 2026-11-12 定「停下并报警，新卡开成自动解除」）：交给了实现、实现还在开卡的期间，一批出错不当场解除，后台任务停下、每次叫起报警、照阈值在 CPU 上补跑；实现答换上了那一刻解除，答开不成照开不成停着。回归用例取模型两臂：`arm=switch_opening` 翻成停着、报警。零轮。
4. 新七（用户 2026-11-12 定「不再试，要卸载重挂」）：一张卡在一次挂载里开卡累计试满 4 次都不成之后，再有切换点到它不再开，当场答开不成（带一种单独的原因），GC 停下、报警；要卸载重挂。原来钉「再交一次重新试 4 次」的回归用例（`openings=8`）翻成不再开。零轮。
5. 壬：「前一次开卡没回来」那张表从每次挂载各一份挪到进程级（开卡器持有），各次挂载与切换共用。回归用例取模型：`starts_after_the_later_mount` 翻成 1。零轮。
6. 癸：运行库收的那一份带上切换关卡泄漏的累计（连同卸载那一份），产品路径上有读者。零轮。
7. 设计问题 ①：起不了开卡线程、开卡 panic、前一次没回来、叫停、试满不再试各起一个原因成员，`DeviceOrPipelineDidNotStart` 的文档回到它只装的那一种；穷举的 `match` 跟着加。零轮。
8. 新五、新六：起不了线程的那一次到了间隔而最近交来的号已经不同时，照最近那一组起；收下为被盖掉的切换开成的卡时，最近那一组没点它就当场关。零轮。
9. 没有变异行罩着的九处新分支各补一行变异与用例；`background_gpu.rs` 里告知不参与时清掉生效那一组点的卡那一句先判是不是等价变异。
10. 文档与注释：`crates/singlefs-core/src/compaction_plan.rs` 第 637、642 行还写旧的切换生效时刻，改成现状；这几份文件注释里写来历的 23 处（任务 14619 列的，「用户定案」「定案」）改成只写现状。
11. 写回 kb：D26（后台整理与放置回收） 已定项 13 第 4 条照改法 1、3、4、5 改写（开不成之后生效的照旧是切换之前那一组；开卡期间旧卡出错停下并报警、新卡开成解除；同一张卡一次挂载里累计试满 4 次不再试；前一次没回来的闸管到进程），变更史记用户 2026-11-12 两条定案的原话。

## 交用户的

- 用户 2026-11-12 已定、这一轮照办的两条：开卡期间旧卡出错「停下并报警，新卡开成自动解除」；同一张卡试满 4 次之后人再切到它「不再试，要卸载重挂」。
- 接手方照已有定案推的、没单独问的（可以推翻）：试满的次数按一次挂载里这张卡累计算，卸载重挂清零；前一次开卡没回来的闸管到整个进程（重挂也不再起，出路只有重启进程）。
- 知道一下：开卡线程里本包自己的断言 panic 也照条款记成「这一次没开成」、照重开，报警里看得到 panic 的原话，但看上去像卡坏了；条款没分这两种，没改。
- 知道一下：新一在改之前，用户「切到 cpu 让出卡、切回时卡开不成、处理完再切一次」会让整个挂载的后台维护线程坏掉。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/src/background_gpu.rs：判了切换开卡、收下结局、告知不参与、关卡与重开；新一的断言、新三切换那一边的接住、新五、新六、设计问题 ① 打中在这一份
- crates/singlefs-background-gpu/src/background_gpu_compute.rs：判了两批之间那一问三种答法转给核心层的形状，对得上
- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs：判了两个包装转发新方法、照旧直接读挂载那一刻的出错，对得上
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了开卡器留给运行库的东西与「前一次没回来」的射程；癸、壬打中在这一份
- crates/singlefs-background-gpu/src/gpus_of_this_host.rs：判了挂载那一刻的重开循环与切换关卡泄漏的累计；新三挂载那一边打中在这一份
- crates/singlefs-background-gpu/src/lib.rs：判了这一轮的修补没动它，腿没在它上面打中东西
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：判了这一轮的修补没动它；对出错原因穷举的那一处 `match` 在这一份，设计问题 ① 的改法要跟着动它
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了挂载那一路开卡线程 panic 原样往上传；新三打中在这一份
- crates/singlefs-core/src/background_compute.rs：判了核心层三组设置与三种答法、出错解除、答开不成时记成生效；新一、新二的根子与新四打中在这一份
- crates/singlefs-mount-runtime/src/lib.rs：判了运行库收的开卡结局只有把手与报告、维护线程 panic 记成运行库坏了；癸的读者缺在这一份

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-12 改了：第 4 条照这一轮改法 1、3、4、5 改写，随这一批写回 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-11-12 不受影响：这一轮只判开卡、关卡与切换生效的时刻，不碰卸不卸给 GPU 的判据 |
