# c996-code-r1：判决（2026-11-12）

正文 `research/prompts/_c996-code-r1-body.md`，开工快照 `research/prompts/c996-code-r1-snapshot/`。这是这一题代码轮的第一轮，前面没有判决。

快照期间别的会话改过快照里的两份 kb（事先问过主 agent）：`.claude/kb/checks-owed.md` 改了 C996 之外的行（清单 `~/.local/share/singlefs/work/checks-owed-paid-writeback/changed-rows.md`，C996 那一行没碰），`.claude/kb/decisions/26-后台整理与放置回收.md` 改了第 97、168、192 三行（行内去掉已还清的欠账号，没增删行）。两条腿引这两份文件的行都落在 D26 第 250–253 行，不在被改的那几处；`checks-owed.md` 两条腿都没引。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/c996-code-r1-sonnet-output.md` | 逐句对照大多对上；「同一设置 gpu 到 cpu 到 gpu：旧失败被当成新切换的」对不上；「开卡线程与闸对不上；账对得上」；切到 cpu 不关卡、没点名时拒挂载两处「条款文本要主 agent 定」；检查单八格全部成立 |
| 攻方（Opus） | `research/prompts/c996-code-r1-opus-output.md`，模型 `research/prompts/c996-code-r1-opus-model/` | 甲、乙、丙、丁、戊、庚、壬、癸打中，己、辛、子推不出；泄漏账重复扣或漏扣、下标错位、一批之内换设置、停着的 GC 被悄悄放开没打中；检查单「类型与名字」「带 --item」两格不成立 |

核查命令原样：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/c996-code-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py c996-code-r1 --out ~/.local/share/singlefs/work/c996-code-r1-verdict/c996-code-r1-tree --repository /home/fy5090/code/singlefs`（任务 14159，退 1）：清单点名 13 份，倒推出 11 份、sha256 对上 11 份；倒推不出 2 份，就是开头写的那两份被别的会话改过的 kb。
- `python3 scripts/agents/cite-check.py research/prompts/c996-code-r1-sonnet-output.md research/prompts/c996-code-r1-opus-output.md --root ~/.local/share/singlefs/work/c996-code-r1-verdict/c996-code-r1-tree --background research/prompts/_c996-code-r1-background.md`：末行「✓ 核了 132 处引文、95 处裸行号，对上 97 处，没判 130 处」。没判的是快照树里没有的路径与裸行号。
- 快照树里没有的那 35 处引文（D26 第 250–253 行、快照外的 `mounted_session.rs`、`fs-design.md` 等）对着此刻的主工作区再核一遍（同一条命令 `--root /home/fy5090/code/singlefs`）：末行「✓ 核了 132 处引文、95 处裸行号，对上 132 处，没判 95 处（其中裸行号没有引文可比 95 处」。D26 第 251、253 行的五句条款原话主 agent 另用 `grep -nF` 逐句核过在那两行。
- 攻方腿引的关键代码行主 agent 现读过：`crates/singlefs-core/src/background_compute.rs` 第 856、1394–1395、1595、1609–1620 行，`crates/singlefs-background-gpu/src/background_gpu.rs` 第 2222–2231、2297–2314、2376–2381 行，与腿引的逐字相同。
- 判决引的模型读数由主 agent 复跑：模型目录拷到 `~/.local/share/singlefs/work/c996-code-r1-verdict/opus-model`（两份源码 sha256 与腿的相同），冻结副本是腿的副本（主 agent 核过：副本里那两份文件去掉模型源码整段之后与主工作区逐字相同），任务 14158：「name=rerun_exit background_gpu_lib=0 core_lib=0」，两行「test result: ok. 5 passed」，十条用例都 ok。
- 核不动的：辛（开卡线程 panic）、壬（驱动里挂住）要真驱动里的 panic 与挂住，替身造不出，只照代码读；核心层五条模型用的是照接口文档写的替身，不是真外壳，乙、丙两格另有 GPU 包两条对着真外壳的读数，甲没有对着真外壳的那一半。

## 二、判决

根在一处：核心层在一批开头把设置交给实现的那一刻就当成切换生效（`background_compute.rs:1609–1611`），实现要等会话锁外开完卡、下一次两批之间那一问才真换上。两边差一个开卡期。

| 号 | 判 | 依据 |
|---|---|---|
| 甲 新卡还在开时旧卡出错，GC 停下，新卡换上之后不解除 | 打中 | 任务 14158「name=attack_stop_outlives_the_switch arm=error_after_the_handing_in batch_while_opening_begun=true opening_under_way=true after_the_error=LeftStopped card_opened=true batch_once_opened_begun=false stopped=true」，对照臂「name=attack_stop_outlives_the_switch arm=error_before_the_handing_in after_the_error=StopLiftedByTheSwitchRequestedBeforeTheError stopped=false」；违反 D26（后台整理与放置回收） 已定项 13 第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」。核心层一侧用替身量的 |
| 乙 交设置那一刻读到切换之前那一组的「用不上」，重新停下，换上之后不解除 | 打中 | 「name=attack_stale_stop stopped_at_the_mount=true stopped_after_the_request=false batch_at_the_handing_in_begun=false batch_once_opened_begun=false stopped=true implementation_usable_now=true」；真外壳一侧交设置那一刻交回的是切换之前那张卡的「一张都不剩」（任务 14158 的 `attack_failure_of_the_settings_before_the_switch` 那一行）；违反同条「停着的时候收到任何一次通过校验的切换请求（校验同挂载那一刻），当场解除停下，从下一批起用新设置」 |
| 丙 开完换上的那一组一张卡都用不上，不按条款的成员停、照开一批 | 打中 | 「name=attack_left_out_after_the_opening not_split_when_handed_in=None reported_by_the_poll=None not_split_after_the_poll=true batch_failed=true devices_given_work=[]」与核心层「name=attack_left_out_after_the_opening arm=card_not_open_at_the_switch batch_while_opening_begun=true batch_once_opened_begun=true batch_once_opened_on_the_gpu=true stopped=false」，对照臂「name=attack_left_out_after_the_opening arm=card_already_open batch_at_the_handing_in_begun=false stopped_by_gpu_unusable_under_the_settings_handed_in=true」；违反同条「报的是挂载那一刻同一个成员」 |
| 丁 同一组设置在那一问之前又交一次（或中间切过 cpu），拿到上一次开卡的陈旧失败、零次重开 | 打中，两条腿各自报出 | 「name=attack_stale_failure arm=handed_in_again_before_the_poll openings_when_handed_in_again=4 causes_reported=Some(4) openings_after_the_poll=4」，对照臂「name=attack_stale_failure arm=handed_in_again_after_the_poll reported_before=true reported_after=true openings=8」；`background_gpu.rs:2302` 按内容比 |
| 戊 不再参与的卡不关：① 切到 cpu；② 为被盖掉的切换开成的卡在最近那一组开不成之后；③ 切换时有提交还在跑而泄漏的卡 | ①② 打中；③ 不算打中 | ① 两条腿都读出，用户定案见「交用户的」；② 任务 14158 的 `attack_superseded_card` 那一行（最近那一组开不成之后，为被盖掉的切换开成的那张卡开着、不参与）；③ 是照卸载同一个规矩泄漏（同函数、同账），条款对切换没写，判成读法、写回 kb 时写明同卸载 |
| 己 cpu 切回 gpu 点到没开的卡，开卡期间在切 cpu 之前那组卡上算 | 随戊 ① 的定案消掉 | 切到 cpu 时关掉全部卡之后，切回时切换前那组卡是空的，开卡期间在 CPU 上算 |
| 庚 起不了开卡线程：1 个原因、不重开 | 打中 | `background_gpu.rs:2376–2381`；同一份代码里每一次开卡内层起线程失败是进重开循环的，两处口径不同；类型文档写「1 + 重开次数 个」 |
| 辛 开卡线程 panic 在会话锁里原样再 panic，整个运行库报坏 | 推不出，按打中办 | `background_gpu.rs:2280`；没跑，照代码读；挂载那一刻同一种 panic 只拒挂载 |
| 壬 开卡器「上一次开卡还挂着」的闸管不到切换开卡线程与重开之前没回来的那一次；卸载后开卡线程没有取消信号 | 打中，两条腿各自报出 | 照代码读，没跑；开卡器文件头自己的理由与重开写法在同一个包里矛盾 |
| 癸 切换关卡泄漏只进进程账，运行时看不出 | 打中（轻） | `.claude/rules/fs-design.md`「分支必须可观测」 |
| 子 第 4 条「没点名的卡不用，参与的卡一张不剩时挂载照样不拒」读不出所指 | 条款问题，用户已定 | 见「交用户的」 |
| 设计问题 ①（一共开 4 次、报 4 个原因）、②、⑧ | 成立 | 两条腿一致：条款写「重开」「最多重开 3 次」，头一次不算重开 |
| 泄漏账重复扣或漏扣、参与的卡下标错位、一批之内换设置、停着的 GC 被悄悄放开 | 这一轮不下结论 | 两条腿各自推过、都没造出；每条腿只抽了一次样，照规矩一次不撑否定结论，第二轮再攻 |

检查单八格：走不到的理由成立；类型与名字不成立（庚那一处）；不兼容旧代码成立；不新建判定库成立；多机可用成立（第二台上没跑过，推的）；测试优先多线程成立；带 --item 不成立（26 个名字逐字相同，但有 6 处新行为没有变异行罩着：`background_gpu.rs:2225`、`:2230`、`:2302`、`:2298` 与 `:2303`、`:2313`、`ThreadNotStarted` 两处，加实现员自己写的两处）；新崩溃流成立（没有新的落盘形态）。

这一轮打中的都落在同一批文件上，改完开第二轮；第二轮攻这一轮的修补本身。

## 三、改法

都是腿提的或主 agent 推的，零轮。A/B 里没有改法的那一臂就是任务 14158 的十条读数；实现员把这十条模型用例收成回归用例，改完之后在同一批输入上读数翻面才算换到东西，读数的任务号由合入后验证补在各条里。

1. 甲、乙、丙（一起做，改核心层与实现之间那一问的形状）：那一问的交回改成三种——没动、换上了（连同换上之后这一组用不用得上的答案）、开不成。核心层把已交给实现、实现还在开卡记成还没生效：这期间一批出错，照「出错之前已经收到、还没生效的切换请求」解除停下；交设置那一刻实现还没换上时不读用不上；收到换上了时照挂载那一刻的停法重新判（用得上就不停，用不上按 `GpuUnusableUnderTheSettingsHandedIn` 停）。零轮。
2. 丁：开卡的结局按交设置的次序编号认，不按内容比；为更早一次交设置开的、开不成的不报，照最近那一组重开。零轮。
3. 戊 ①（用户定案）：切到 cpu 时核心层告诉实现一张卡都不参与，实现关掉全部开着的卡、丢掉在等的那一组与还没取走的开卡结局；切回 gpu 当成点到没开的卡照重开规矩开，开卡期间在 CPU 上算。零轮。
4. 戊 ②：最近那一组开不成时，关掉此刻生效那一组之外开着的全部卡，不只关这一次开成的。零轮。
5. 庚：起不了开卡线程也进重开循环（间隔之后再起，至多重开 3 次，每次一个原因）。零轮。
6. 辛：开卡线程里接住 panic，记成这一次开卡没开成的一个原因、照重开；不在会话锁里再 panic。零轮。
7. 壬：同一张卡前一次开卡没回来就不再起下一次，算这一次又没开成、照间隔往下数；切换开卡线程带一个取消信号，卸载时置上，线程在每一次开卡之间看它、置上就不再开。零轮。
8. 癸与戊 ③：切换关卡时泄漏照卸载同一个规矩，另报出来（累计进一份对外读得到的泄漏报告）。零轮。
9. 没有变异行罩着的 8 处新行为各补一行变异与用例。

## 交用户的

用户 2026-11-12 弹窗已定两条：

- 切到 cpu 时开着的卡：选「全部关掉 (Recommended)」。
- 没给 `background_gpu_devices` 时挂载那一刻一张卡重开 3 次都开不成：选「拒绝挂载 (Recommended)」。代码不动，D26（后台整理与放置回收） 已定项 13 第 4 条里「没点名的卡不用，参与的卡一张不剩时挂载照样不拒」那半句随写回删改。

主 agent 照用户的重开规矩推的、没单独问的四条（都被攻过零轮，用户可以推翻）：起不了开卡线程照重开（改法 5）；开卡线程 panic 当成一次开不成、照重开（改法 6）；前一次开卡没回来就不再起、卸载时取消开卡线程（改法 7）；切换关卡时有提交还在跑的卡照卸载同一个规矩泄漏并报出来（改法 8）。

挂载最多被一张开不成的卡挡 23 分钟（4 次开卡各至多 5 分钟加 3 个 1 分钟间隔），外层期限是枚举 5 分钟加卡数乘 23 分钟：这是重开 3 次的直接算术，报给用户知道。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/src/background_gpu.rs：判了切换开卡、换上、关卡、重开到上限的报法与泄漏账；丁、戊、庚、辛、癸打中在这一份。
- crates/singlefs-background-gpu/src/background_gpu_compute.rs：判了两批之间那一问转给核心层的形状与开不成的报法；丙的实现那一半在这一份。
- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs：判了删掉挂载那一刻没开成的转发、加新方法转发，对得上。
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了每卡期限改成连同重开的最长等待（成立）与上一次开卡还挂着那道闸的射程（壬）。
- crates/singlefs-background-gpu/src/gpus_of_this_host.rs：判了挂载只开参与的卡、重开后拒挂载、关卡挪回尚未开的那几张与泄漏判法，对得上；不给卡列表时拒挂载由用户定案认下。
- crates/singlefs-background-gpu/src/lib.rs：判了文档改成现行规则，对得上。
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：判了穷举 match 里新成员那一臂，对得上。
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了重开规矩的常量、次数与最长等待的算式（设计问题 ①② 成立），到点之后前一次开卡线程照旧跑着（壬）。
- crates/singlefs-core/src/background_compute.rs：判了切换生效的时刻、停下与解除、切到 cpu 不通知实现；甲、乙、丙与戊 ① 打中在这一份。
- crates/singlefs-mount-runtime/src/lib.rs：判了 `switch_background_compute` 的文档改动，对得上；辛的后果（维护线程 panic 记成运行库坏了）落在这一份的已有代码上。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-12 改了：第 2 条加切到 cpu 时关掉全部开着的卡，第 4 条删改「没点名的卡不用」半句、写明切换关卡的泄漏同卸载，随这一轮写回 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-11-12 不受影响：这一轮只判开卡、关卡与切换生效的时刻，不碰卸不卸给 GPU 的判据 |
