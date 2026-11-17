# c996-code-r3 辩方腿报告（Sonnet，2026-11-13）

轮 c996-code-r3；批次 records/2026-11-11-background-gpu审核四条修复.md#7；立场：替被判出局的一方找站得住的理由、复核两份已有判决够不够得着、改法今天有没有照写。

读法与范围：代码读冻结副本 `/home/fy5090/.local/share/singlefs/work/c996-code-r3/repo/`（下文 `crates/…:行` 都是副本里那份文件自己的行号，`grep -n` 现取）；kb 条款写文件名加那份文件自己的行号；任务日志只读 `task-server/task log <号> --all`，落在草稿目录 `/home/fy5090/.local/share/singlefs/work/c996-code-r3-defense/log-<号>.txt`。没写模型，没有模型目录；没读禁读清单里的两样。

量的与推的：凡写「量的」都是我这一趟现跑 `grep` / `awk` / `task-server/task log` 读到的；凡写「推的」是读码推的、没跑；凡写「没量过」是要跑才知道的，命令在末尾「要主 agent 跑的命令」。

## 一、两份判决「三、改法」逐条：照写了没有（关的是哪几条 1）

回「照写了」「没照写」「推不出」。「够得着」一栏回：判决打中的那一格，是只打中这份实现，还是同样打中所有替代方案。

| 判决 / 条 | 回 | 今天的代码（文件:行） | 够得着吗 / 备注 |
|---|---|---|---|
| c996-code-r2 改法 1 新一、新二：答开不成时核心层不记成生效、清掉「在开卡」那一格、补跑阈值另记一格 | 照写了 | `crates/singlefs-core/src/background_compute.rs:1819`–`:1824`（开不成那一支只存请求给的阈值、不动生效那一组）、`:1436` 字段、`:1646` 读阈值、`:1787` 换上时清它；回归用例 `:2830`、`:2850`（先切 cpu 一臂走 CPU、不先切一臂认同一组） | 够得着，且替代方案同样打中：让核心层记成生效、再叫实现照新那一组走，仍是两边对生效那一组各认各的，根子是「实现没开成而核心层认了」，改在记账一侧才断得掉 |
| 改法 2 新三：接住 panic 挪进挂载与切换共用的开一张卡那一步 | 照写了 | `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:845`（共用那一步）、`:866`（`opening_with_its_panic_caught` 包着 `open_the_gpu`）、`crates/singlefs-background-gpu/src/background_gpu.rs:1306`（接住的实现）、`:1451`（切换开卡线程走同一步）；`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:328` 的 `resume_unwind` 仍在，落在那层接住的里面；回归用例 `crates/singlefs-background-gpu/src/background_gpu.rs:7805`、`:6344` | 够得着。注意只接住 `open` 里面的 panic：开卡步骤之外（`crates/singlefs-background-gpu/src/background_gpu.rs:1435` 的 `debug_assert!` 在锁里、在接住那层外面）的 panic 仍会毒化锁并在两批之间 `resume_unwind`（见第五节第 1 格） |
| 改法 3 新四：开卡期间出错停下、报警、补跑，答换上了解除 | 照写了 | `crates/singlefs-core/src/background_compute.rs:1516`–`:1536`（先判请求了还没交给实现的 `:1527` 当场解除，再判交给了实现的 `:1531` 记 `LeftStoppedWhileTheSwitchOpensItsCards`）；答换上了那一刻整批换掉停着的 `:1809`–`:1816`；回归用例 `:2908`、`:2976` | 够得着。这一格是用户定案，替代方案无从比 |
| 改法 4 新七：一张卡试满 4 次之后再点到它不再开、当场答开不成、带单独原因 | 照写了，但读法与判决字面相反 | `crates/singlefs-background-gpu/src/background_gpu.rs:1327`–`:1380`（记数）、`:1383`（叫停不算）、`:2626`（交设置那一刻先查）、`:2851`、`:2881`（为在等的那一组起之前再查）、`:165`（原因成员）；回归用例 `:5987` | 判决改法 4 与「交用户的」写的是累计、卸载重挂清零（`research/prompts/c996-code-r2-main-verification.md:70`：「试满的次数按一次挂载里这张卡累计算，卸载重挂清零」）。今天是连着开不成、开成清零（`crates/singlefs-background-gpu/src/background_gpu.rs:1372` 的 `forget_the_failed_openings_of`）。这是主 agent 自己改的读法，不是判决条文；见第三节 |
| 改法 5 壬：「没回来」那张表挪到进程级、开卡器持有 | 照写了，产品开卡器一条路；实验装置没共用 | `crates/singlefs-background-gpu/src/background_gpu.rs:1229`（表）、`:1269`（记）、`:1175`（每次开卡进来先查）；`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:148`、`:271`、`:288`（开卡器持有、交给每一次开卡同一张）；e202 的四处调用各现建一张空表（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:4266`、`:4280`、`:7873`、`:7975`） | 够得着。「进程级」实际是「开卡器那一个实例」：一个进程里造两个开卡器就是两张表；产品里造开卡器的只有测试（`crates/singlefs-background-gpu-tests/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs:95`），量的 |
| 改法 6 癸：运行库收的那一份带上切换关卡泄漏的累计，产品路径上有读者 | 照写了读口；仓里没有非测试的读者 | `crates/singlefs-mount-runtime/src/lib.rs:153`、`:323`、`:940`–`:946`、`:1192`；开卡器换给运行库的那一份 `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs` 的 `opened_background_gpu_for_the_mount_runtime` | 量的：`grep -rn gpus_leaked_by_the_background_gpu --include=*.rs crates prune-server` 去掉运行库自己，只剩 `crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:1605`、`:1613`。读口是 `pub`，字面「有读者」成立；仓内产品代码里没人调 |
| 改法 7 设计问题 ①：五个原因成员、旧成员文档回到它只装的那一种、穷举跟着加 | 照写了 | `crates/singlefs-background-gpu/src/background_gpu.rs:157`–`:165`（五个成员）；穷举 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:871`–`:875`、`:895`–`:899`，`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:130`–`:134`、`:208`–`:212`，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:337`–`:356`、`:422`–`:426` | 够得着 |
| 改法 8 新五、新六 | 照写了 | `crates/singlefs-background-gpu/src/background_gpu.rs:2731`–`:2745`（到了间隔、最近号不同就照最近那一组起）、`:2830`–`:2845`（收下被盖掉的切换开成的卡、最近那一组没点它就关） | 够得着 |
| 改法 9 九处没有变异行罩着的分支补一行 | 照写了九处；另有三处新分支按子串找没有行锚在上面 | 九处对应 `crates/mutations.tsv` 第 7794、7790/7789、7791/7792、7793、7779、7780、7781、7773 行（第一列逐字在，量的）；没有行锚的三处：`crates/singlefs-core/src/background_compute.rs:1787`（换上时清开不成那一组留下的阈值）、`crates/singlefs-mount-runtime/src/lib.rs:946` 与 `:1192`（读口克隆、卸载时随 `UnmountedPool` 带走）、`crates/singlefs-background-gpu/src/background_gpu.rs:1432`–`:1434`（这一次至多开「剩下的次数」） | 三处是按原文、替换文两列子串找的（`awk -F'\t'` 找第三、四列），换写法锚在别处的没逐行读，推的 |
| 改法 10 文档与注释（`compaction_plan.rs` 两处、23 处来历） | 照写了；另有残留不在这 23 处里 | `crates/singlefs-core/src/compaction_plan.rs:631`–`:650`（切换生效时刻改成现状）；任务 14619 在这十五份里列的行，今天 `grep -nE '用户定案|欠账 C996 的定案'` 在 15 份里只剩 e202 的两处（在存量滞后表里） | 残留：`crates/singlefs-core/src/background_compute.rs:13`、`:745` 仍写「同日弹窗定」（谁定的）；不在 14619 的清单里，也不在改法 10 点名里 |
| bggpu-audit-fix-code-r1 改法 1 甲：切换量代价之前先等账上还在跑的提交、等不到就当量代价出错 | 照写了 | `crates/singlefs-background-gpu/src/background_gpu.rs:3132`–`:3150`（量之前先等、放账；失败记 `:3149`）、`:3172`（每批报 `CostNotMeasuredAtTheSwitch`）、`:359`（成员）；回归用例 `:8690`–`:8800` 一带一格一条（两段历史各 5 种用户动作） | 够得着。只在最先一张量出错的卡上记原因（`get_or_insert`，`:3137` 一带），别的卡也不参与但原因不带 |
| 改法 2 乙：选段打分先查重复的段，再断言整段清单 | 照写了 | `crates/singlefs-background-gpu/src/live_slot_kernels.rs:502`（先查）在 `:505` 之后的整段断言之前；到核心层 `crates/singlefs-background-gpu/src/background_gpu_compute.rs:613` | 够得着 |
| 改法 3 丙：冲掉待写的空提交等不到交回单独原因、算超时数、算没交到卡上 | 照写了 | `crates/singlefs-background-gpu/src/background_gpu.rs:168`（原因成员）；`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:128`（算进超时数）、`:204`（`the_attempt_reached_the_gpu` 判没交到）；转给核心层 `crates/singlefs-background-gpu/src/background_gpu_compute.rs:324` | 够得着；这一格报的数对了、停卡时刻没改（读 `timeouts`） |
| 改法 5 重复键到核心层只放弃这一批 | 照写了 | `crates/singlefs-core/src/background_compute.rs:1272`（`reach_of_the_error`）、`:1523`（不停、不报警）；落盘那一半 `crates/singlefs-core/src/mounted_session.rs:5187`–`:5203`（照旧走只删意图的发布）；回归用例 `crates/singlefs-core/src/background_compute.rs:3051`，验证包 `crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs:1060` | 够得着；`crates/singlefs-core/src/mounted_session.rs:5099` 的函数文档仍写「出错之前就挂着待生效的切换时当场解除」，与新四之后只对「请求了还没交给实现」那一种成立的现状不全合，该文件不在这轮快照里，只记 |
| 改法 6 字节数、条数换各自的类型 | 照写了 | 核心层 `crates/singlefs-core/src/background_compute.rs:72`–`:117`（`GpuMemoryInBytes` 加 `checked_add`、`checked_sub`、`plus`、`Sum`）；本包 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:44`（输入长度）、`:112`（份数）、`:145`（一段条数）、`:207`（`ChunkFootprint`）、`:628`–`:720`（记账） | 够得着；判决说的「两个份数常量的用处逐处换」在 `crates/singlefs-background-gpu/src/checksum_kernels.rs` 等三段里见 diff 的 754、803、852 行 |

## 二、判为「站住」「修掉了」的格，今天还站不站得住（关的是哪几条 2）

回：站得住 / 站不住 / 推不出。「站不住」指那一格的证据读数或子句在今天的代码上已不是原样，不一定是缺陷回来了。

c996-code-r2 判决「二、判决」（`research/prompts/c996-code-r2-main-verification.md`）：

| 格 | 回 | 依据 |
|---|---|---|
| 第一轮甲、乙、丙、丁、戊 ①②、己、庚「修掉了」 | 大体站得住；其中「开卡期间出错照切换解除」那一读已被新四翻面，丁的「每次交设置各数 4 次」那一读已被新七与主 agent 的连着读法先后翻面 | 任务 14649 里 `arm=error_after_the_handing_in … stopped=false` 那一行的读数，今天的代码走 `crates/singlefs-core/src/background_compute.rs:1531` 记 `LeftStoppedWhileTheSwitchOpensItsCards`、停着；「停下不越过切换生效」这层语义仍在（答换上了当场解除，`:1809`–`:1816`，回归用例 `:2908`）。读数相反是用户定案的结果，不是缺陷回来；判决表里那一行证据今后不能再拿来当「现状读数」 |
| 新一、新二、新三打中 | 站得住（打中成立，今天已修） | 前述「改法 1、2」两行 |
| 新四「打中（用户已定）」 | 站得住 | 改法 3 的行 |
| 新七「与用户定案相反」 | 站得住（打中成立）；改法的读法见第三节「累计→连着」 | `research/prompts/c996-code-r2-main-verification.md:70` 与今天代码 `crates/singlefs-background-gpu/src/background_gpu.rs:1372` 相反 |
| 壬「还中（缩小了）」、癸「还中」 | 站得住；今天壬修到「开卡器那一个实例」、癸修到「读口」 | 改法 5、6 的行：实验装置不共用那张表；读口无非测试读者 |
| 设计问题 ③(a)「不成立」 | 站得住（已断） | `crates/singlefs-core/src/background_compute.rs:1819`–`:1824` |
| 设计问题 ③(b)「叫停之后正在开的那一次照旧开完、开成的卡在那条线程上丢」成立 | 站得住，今天代码没动它 | `crates/singlefs-background-gpu/src/background_gpu.rs:1446`（叫停只在每一次开卡之前看，开着的那一次不叫醒）、`:1451`、`:2667` 的 `call_off`；条款只写「取消，不再接着开」（kb 里 `.claude/kb/decisions/26-后台整理与放置回收.md:253`），没写这一次开完 |
| 设计问题 ①「几种没开成共用一个成员」打中 | 站得住，已修 | 改法 7 的行 |
| 「接住 panic 把本包自己的断言也记成开不成」判「照条款字面，不改」 | 站得住（字面）；有一处判决没提的后果 | 接住用的是 `AssertUnwindSafe`（`crates/singlefs-background-gpu/src/background_gpu.rs:1309`），接住之后被它包着的闭包里改了一半的状态（泄漏账、`opening_of_a_gpu_at_a_switch` 那个 `FnMut` 的状态）有没有留下不一致，没人判；读码推的：泄漏账靠 `Drop` 与 `PoisonError::into_inner` 兜底（`crates/singlefs-background-gpu/src/background_gpu.rs:1288`），没量过 |
| 第一轮没下结论的四样「两条腿各推一遍没打中」 | 站得住（处理对）：判决写了「照规矩不撑否定结论」 | `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` |
| 检查单「不兼容旧代码…」「不新建判定库」「所有任务多机可用」「测试优先多线程」「新崩溃流多罩了什么」成立 | 站得住，这一轮的 diff 上我重核过同一批，见第五节 | 第五节 |

bggpu-audit-fix-code-r1 判决「二、判决」（`research/prompts/bggpu-audit-fix-code-r1-main-verification.md`）：

| 格 | 回 | 依据 |
|---|---|---|
| T1「开卡量代价出错的泄漏（G01 本身）」站住 | 站得住 | 泄漏处置还是同一个判别子：`crates/singlefs-background-gpu/src/background_gpu.rs:1512` 的 `started_gpu_with_its_cost_measured` 的量代价出错那一支没动；接住 panic 没绕开它（panic 时卡随 unwind 照常 `Drop`，今天与以前都不进泄漏账，推的）。新增的「切换时量代价」（甲）走的是同一个「还在跑」标记（`:3137`），不另起一个 |
| T2「重复条目键，校验和与区域扫描两段」站住 | 站得住，其中「到核心层与别的出错同一种停法」一句已被改法 5 取代 | `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:55`–`:69`（先整批查、后动计数）今天仍在；核心层今天按 `crates/singlefs-core/src/background_compute.rs:1272` 分两种停法 |
| T3「三段三处口径、改小上限的推导」站住 | 站得住，但原来的数不是在今天的代码上量的 | 原数是任务 14625 上攻方模型 `fit_cases=200000 fit_disagreements=0` 那一行，对的是换类型之前的 `u64` 实现；换类型之后实现员说数值不变（推的），今天没有对拍；见第五节第 1 格「换了算法的断言」 |
| T4「只改名的 25 份」站住 | 站得住（这一轮快照里不含它们） | 15 份产品文件里没有这 25 份；没重核 |
| 检查单「类型与名字」不成立（存量写法的延续） | 站得住，且这一批已修 | 改法 6 的行 |
| 检查单「测试优先多线程」成立 | 站得住 | 见第五节第 6 格 |

「够不够得着」的单独回答（两份判决打中的格里，哪一格同样打中所有替代方案）：

- 甲（切换量代价撞 `hold` 断言）：只打中这份实现，不是所有替代方案。它的前提是「切换时给请回来的卡再量一次提交的代价」，而条款第 3 条只写「挂着切换给了它装得下的上限，从下一批起照新上限按卡的顺序重新参与分活」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`），没写重新量代价；代码注释 `crates/singlefs-background-gpu/src/background_gpu.rs:3130` 把「照新上限重新量代价」当成第 3 条的原话引，现行条款里没有这一句（量的：`grep -cF` 该句在 `.claude/kb/decisions/26-后台整理与放置回收.md` 里零命中；这是旧条款的字，改成「重新参与分活」的记录在 `.claude/kb/decisions-history.md:15310`，即代码这一步是已被条款删掉的那一句留下的）。再量一次是为了 `participating_gpus()` 里每张参与的卡都有代价可报（`crates/singlefs-background-gpu/src/background_gpu.rs:2611` 的 `expect`），所以替代方案「请回来的卡沿用挂载时的旧值、不再量」要连 `expect` 一起改，另一种实现，不被甲打中；改法 1（量之前先等）是在现有实现上堵洞，够得着，但「同样打中所有替代方案」不成立。
- 乙（死成员）：只打中这份实现（先断言后查重复的次序），换成「先查再断言」之外的写法都不会中；够得着，改法恰好是挪次序。
- 丙（重交数多算 1）：只打中「把等超过上限的冲掉空提交也算交到了卡上」这一种分类写法；改法是改分类，够得着。
- 新一、新二：打中的是「实现没开成而核心层认了」，改在核心层记账一侧；今天实现一侧另有 `batch_not_split_over_the_gpus` 的断言（`crates/singlefs-background-gpu/src/background_gpu.rs:3181`）靠核心层不在「实现没换上」时走 GPU 一路，这是一条只靠注释维系的跨层约定（`crates/singlefs-background-gpu/src/background_gpu.rs:3157`–`:3162` 那段 `# Panics` 文档），改法没给它加会红的检查；见第五节第 1 格。

## 三、未定项清单的完整性，与 D26 已定项 13 第 1、2、3、4、8 条现行文本对代码（关的是哪几条 3）

### 3a 清单：有没有漏掉该交用户的

我拿到的清单：c996-code-r2 判决「交用户的」四条（`research/prompts/c996-code-r2-main-verification.md:67`–`:72`）、bggpu-audit-fix-code-r1 判决「交用户的」四条（`research/prompts/bggpu-audit-fix-code-r1-main-verification.md:60`–`:65`）、五份实现员报告「停下交主 agent 的设计问题」（读过原文，量的）、主 agent 自己定的四条。逐条核下来，下面这几样不在任何一份清单里，或者在清单里但与判决的原话相反没单写：

| 编号 | 没交的东西 | 为什么该交用户 | 在哪看得到（文件:行） | 量的 / 推的 |
|---|---|---|---|---|
| 未交一 | 新七的读法：判决写的是累计、卸载重挂清零，今天条款与代码是连着、开成清零 | 它把用户「一张卡至多开 4 次」的上限放松了：只要每次开成就不封顶；主 agent 清单第一条「开成清零」说了「用户没单独定」，但前一份判决「交用户的」第二条（`research/prompts/c996-code-r2-main-verification.md:70`）还写着相反的读法，两份都在记录里，用户读哪一份都对不上另一份 | 条款 `.claude/kb/decisions/26-后台整理与放置回收.md:253`（「同一张卡在一次挂载里切换开卡连着开不成满 4 次（跨切换接着数，中间开成一次就清零）」）；代码 `crates/singlefs-background-gpu/src/background_gpu.rs:1372` | 量的（两处原文并排） |
| 未交二 | 试满之后的出路自相矛盾：前一次开卡挂在驱动里没回来时，也一次次记进「连着开不成」，满 4 次后答 `OpeningsInThisMountUsedUp`，条款对这个成员说「要卸载重挂」，对「没回来」说「出路只有重启进程」 | 用户照报警卸载重挂，挂在驱动里的那一次还在、进程级表还卡着，重挂后又要失败；报警里这一回只带「试满」一个原因，更早几次「没回来」的原因不在里面 | 记数规则 `crates/singlefs-background-gpu/src/background_gpu.rs:1383`（`EarlierOpeningOfTheDeviceNotReturned` 也算开不成）；答开不成时只带一个原因 `:2896`；条款 `.claude/kb/decisions/26-后台整理与放置回收.md:253` 两句出路 | 推的（读码；没造「没回来」连着 4 次的模型） |
| 未交三 | 重复键放弃一批、任务不停、不报警之后，下一批若照旧组出同一对重复键，会怎样 | 条款只说「下一批照常算」；若重复是确定性的组批错误，每趟叫起都会：签意图（一次发布）→ 算到重复 → 单发一次只删意图的发布，既不前进也不报警；主 agent 自己定的「照样发一次只删意图的发布」正是这一圈里的写盘；用户定的是「只放弃这一批」，没看到这一圈 | 条款 `.claude/kb/decisions/26-后台整理与放置回收.md:257`（「后台任务不停、不报警，下一批照常算」）；落盘 `crates/singlefs-core/src/mounted_session.rs:5187`–`:5203`；核心层 `crates/singlefs-core/src/background_compute.rs:1523` | 推的（没造模型；重复键是不是确定性的取决于组批代码，我没读） |
| 未交四 | 重复键作废一批时，同一批里别的段已经算完、已改了的连续不确定计数留不留 | 条款第 8 条两句并排：「一批放弃（GPU 出错、等超过上限）时，那一批里已经交回的段照算一轮，放弃的那一段计数不动」与「这一批作废，重复的那一段连续不确定的计数一条不动、也不交 CPU 那一份判」，没写重复键这一支属于哪一句；代码是各段各自对账、没有回滚，即前面已对完的段照算一轮 | 条款行 `.claude/kb/decisions/26-后台整理与放置回收.md:257`；代码 `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:55`–`:69`、`crates/singlefs-background-gpu/src/live_slot_kernels.rs:502`（段内先查再动计数，段与段之间不回滚，推的） | 推的 |
| 未交五 | 切换时量代价（甲）的失败结局：这张卡不参与、之后每一批都报 GPU 出错，直到下一次换设置 | 用户看到的是「后台任务停下、报警」，这一格条款没有写；甲的改法没有 kb 写回条（r1 判决改法 7 只写第 8 条） | 代码 `crates/singlefs-background-gpu/src/background_gpu.rs:3132`–`:3150`、`:3172`；条款第 3 条 `.claude/kb/decisions/26-后台整理与放置回收.md:252` 只写「重新参与分活」 | 量的（代码行）；这一步是旧条款「重新量代价」留下的，现行条款已删它（`.claude/kb/decisions-history.md:15310`）：该问的是「这一步还要不要」，不是怎么写它的失败结局；是否算该交用户是我的判断 |
| 未交六 | 实现员报告里的两条小项没进任何清单：「开成」只算开一张卡那一步交回 `Ok`，开成之后换上那一组时量代价出错或因上限不用的，不记回连着开不成（批次 4 报告第 3 条）；起不了开卡线程那一回记在头一张要开的卡上（批次 3 报告第 1 条只写了哪几种开不成算进次数，没写记在哪张卡） | 都影响「一张卡至多开 4 次」怎么数 | `crates/singlefs-background-gpu/src/background_gpu.rs:3004`–`:3008`（记在头一张卡上） | 量的（读了两份报告原文） |

没漏的（核过）：主 agent 清单的四条与五份报告的设计问题，除上面两条小项外都在清单里或已被后一份报告关掉（批次 5 的第二份、第三份报告已关掉输入长度、核心层字节字段、两个计数、字段改名）。批次 3 报告第 5 条（e202 不共用表）实验装置，不该交用户，但条款第 4 条「各次挂载与切换开卡共用」没有限定「产品开卡器」，量的：`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:4266`。

### 3b 现行条款对代码，一行一处

kb 行号是 `.claude/kb/decisions/26-后台整理与放置回收.md` 自己的行号（第 1 条 250、第 2 条 251、第 3 条 252、第 4 条 253、第 8 条 257，量的，`awk 'NR==…'`）。

| kb 行 | 条款说的 | 代码（文件:行） | 对上 / 代码做了条款没写 / 条款写了代码没做 |
|---|---|---|---|
| 250 | 切换从下一批起生效，一批之内不换 | `crates/singlefs-core/src/background_compute.rs:1795`（两批之间问）；`crates/singlefs-core/src/mounted_session.rs:5092` 一带「一批之内不再读」 | 对上 |
| 251 | 切到 cpu：关掉全部开着的卡、丢掉还在开的那一组；切回当没开的卡重开，开卡期间在 CPU 上算 | `crates/singlefs-background-gpu/src/background_gpu.rs:2661`–`:2682`；核心层 `crates/singlefs-core/src/background_compute.rs:1775` | 对上；回归用例 `:2830` |
| 251 | 挂载时量的代价不再用来分活 | 切换时对「开卡时因上限没量」「晾下后请回」的卡再量一次，量出的代价不用来分活：`crates/singlefs-background-gpu/src/background_gpu.rs:3125`–`:3140` | 代码做了条款没写：再量一次与量不成这张卡不参与、之后每批报 GPU 出错（见未交五） |
| 252 | 连最小的一片都装不下的卡不用、挂着切换给了它装得下的上限从下一批起重新参与 | `crates/singlefs-background-gpu/src/background_gpu.rs:3090`–`:3150` | 对上 |
| 253 | 起不了开卡线程、开卡线程 panic 都算这张卡这一次没开成，照重开 | `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:317`、`crates/singlefs-background-gpu/src/background_gpu.rs:1306`、`:2991`–`:3012` | 对上；做了没写：起不了切换开卡线程那一回记在「头一张要开的卡」上（`:3008`） |
| 253 | 前一次开卡没回来的那张表在进程级（开卡器持有），各次挂载与切换开卡共用 | `crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:148`、`:288`；`crates/singlefs-background-gpu/src/background_gpu.rs:1229` | 对上（产品开卡器）；条款写了、e202 没做（四处各现建空表，`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:4266`、`:4280`、`:7873`、`:7975`） |
| 253 | 开不成之后核心层不把那一组记成生效、补跑阈值取那一次切换请求给的 | `crates/singlefs-core/src/background_compute.rs:1819`–`:1824`、`:1646` | 对上 |
| 253 | 连着 4 次开不成不再开、当场答开不成、单独成员；中间开成清零 | `crates/singlefs-background-gpu/src/background_gpu.rs:2626`–`:2650`、`:2881`、`:1372` | 对上；做了没写：叫停没开的不记（`:1383`）、挂载那一刻的开不成不记（条款写的是「切换开卡」，字面对上） |
| 253 | 报警里报出哪张卡、每次开不成的原因 | 一次切换开卡线程里的每一次（`crates/singlefs-background-gpu/src/background_gpu.rs:1417` 一带）；跨切换累计试满只一个原因（`:2896`） | 条款写了、代码在跨切换接着数时没做全（报警里只有最后那一轮的原因） |
| 253 | 卸载时还在开的切换开卡取消 | `crates/singlefs-background-gpu/src/background_gpu.rs:3739`–`:3742` | 对上；条款没写「正在开的那一次开完、结局没人收」（设计问题 ③(b)，成立） |
| 253 | 开卡期间旧卡出错：停下、每次叫起报警、补跑；换上了解除，开不成照开不成停着 | `crates/singlefs-core/src/background_compute.rs:1531`、`:1809`–`:1824` | 对上 |
| 253 | 出错之前已收到、还没交给实现的切换在停下那一刻算数 | `crates/singlefs-core/src/background_compute.rs:1527` | 对上 |
| 253 | 切换关卡泄漏的卡累计、运行库读得到、卸载之后同一份 | `crates/singlefs-mount-runtime/src/lib.rs:940`–`:946`、`:323`、`:1192` | 对上（读口）；仓内非测试没有调用方 |
| 253 | 累计到第 4 次等超过上限报用不了；交活之前先等前一次超时提交那一等也算 | `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:128`、`:204` | 对上；做了没写：冲掉待写的空提交等不到也算一次超时、不算重交（r1 判决字面读法，没进条款） |
| 257 | 重复键三段交回、核心层不停不报警、下一批照常算、意图照放弃一批的规矩删 | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:550`、`:587`、`:613`；`crates/singlefs-core/src/background_compute.rs:1272`、`:1523`；`crates/singlefs-core/src/mounted_session.rs:5187`–`:5203` | 对上 |
| 257 | 重复的那一段连续不确定计数不动、不交 CPU 判 | `crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:55`–`:69` | 对上；「这一批作废」对同一批别的段的计数没写（见未交四） |

## 四、正文第二节「验证到的数」逐条能不能核，与没被跑到的测试目标（关的是哪几条 4）

读法：任务日志整份取进草稿目录（`task-server/task log <号> --all > …/log-<号>.txt`，只读，量的）；变异行名字从日志里每一行「✓ <名字>：<测试> 红了」取出，与 `crates/mutations.tsv` 第一列逐个 `grep -cxF`（量的，在主工作区与冻结副本两棵树上各做一遍，结果相同）。

| 正文里的数 | 能核吗 | 日志里读到的（草稿目录 log 文件的行） | 备注 |
|---|---|---|---|
| 15315 退 0；「共 131 条测试…现判 130、复用 753、拒算 1」；fmt、clippy、build 过；真卡对拍「按条现判 28、复用 0、拒算 1」 | 核得出，逐字对上 | log-15315.txt 第 1197、1210、1301、1603、1847 行 | 「共 131 条」是这一趟现跑的，「复用 753」是键相同、没重跑、沿用旧判；第 1848 行「product-gpu-real-card-tests：有 2 处本次未跑」 |
| 15185 退 0；「共 309 条测试…现判 308、复用 570」 | 核得出 | log-15185.txt 第 4326 行 | 这个任务三次尝试：前两次都退 250（`memory-cap-hit`），第三次才退 0（`task-server/task status 15185` 量的）；与 15315 命令里是同一个补丁目录，两趟总数一个 884、一个 879（现判+复用+拒算，算术），差 5 条的来源日志里没有：推不出 |
| 15341「抓到 51 条、没红 0、无效 0、锚点失效 0」 | 核得出，51 个名字在 `crates/mutations.tsv` 第一列各恰好 1 次 | log-15341.txt 第 273 行；名字 51 个（40 个以「C996」起头） | 该日志第 274、275 行：基线 51 行不绿 0；「复用：现判 1 条、复用 50 条」——50 条是沿用旧判，不是这一趟在今天树上重新改坏重跑 |
| 15382「抓到 31 条…」，正文称「C996 第一轮修补的变异行里剩下的 31 行」 | 数核得出；称呼不准 | log-15382.txt 第 120 行；名字 31 个，25 个「C996」起头、6 个「七·3」起头 | 「复用：现判 1 条、复用 30 条」；31 个名字都在 `crates/mutations.tsv` 第一列恰好 1 次 |
| 15450「抓到 55 条…」 | 核得出，55 个名字都在表里各 1 次 | log-15450.txt 第 168 行；0 个「C996」起头 | 「复用：现判 1 条、复用 54 条」 |
| 15444 退 0；「共 308 条测试…现判 307、复用 480、拒算 1」；真卡对拍 28/0/1 | 核得出 | log-15444.txt 第 100、108 行 | 11 项里含两个多卡对拍目标，各「判 0 条」：第 100 行之前的 `ITEM_DONE` 行里 `checksum_batches_over_three_real_gpus…`、`checksum_batches_over_two_real_gpus…` 都是 `judged=0 reused=0`（量的） |
| 15441「6 passed」（4 段 compile fail、2 段对照） | 核得出 | log-15441.txt 第 104–111 行 | 工作目录是实现员的副本 `…/bggpu-types-batch5/copy`，不是合入后的树；合入后验证 15444 的测试第 1–4 条没有文档测试（日志第 100 行前的「测试第 n 条」只有 `--lib`、`--test`）：这四段「换反编不过」的钉子合入后没再跑，量的 |
| 「真卡对拍那一格有 2 处报『本次未跑』，哪两处没逐条读」 | 核出来了 | `/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-13-3012059/verification-product-gpu.log`、`…-4039652/…`、`…-1190749/…` 各第 10、11 行 | 两处都是多卡目标：`checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`「声明 3 张、拿到 1 张」，`checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`「声明 2 张、拿到 1 张」，「提交时单交」。三趟都同样未跑 |
| 「合入后验证 Verus 没跑」 | 正文自己说了没验 | — | 不判 |

核不出的点名：（一）15185 与 15315 总数差 5 条的来源；（二）变异行「复用」的键怎么算、复用的旧判是在哪棵树上判的（日志只给「现判的按栏」那一行，没给复用来源）；（三）正文里「变异行点名复跑」51 + 31 + 55 = 137 条在今天的树上是不是每一行都被重新改坏过：日志说现判只有 3 条（量的：50+30+54 = 134 条是复用）。

### 没被合入后验证跑到的测试目标：有没有哪一支只被它们罩着

合入后验证两趟（15315 在先、15444 在后）点到的目标，来自两份日志「编译并跑挑出的 9 项」「…11 项」（量的）：第一趟是核心层 `--lib`、验证包三个、本包 `--lib`、`e202` 二进制、本包一个集成目标、运行库 `--lib` 与一个集成目标；第二趟是核心层 `--lib`、验证包三个（与第一趟只重一个）、本包 `--lib` 与 `gpu_gc_capacity_contracts_through_the_host`、测试包五个。下列引到被改产品代码、而这五趟（15315、15185、15444 与三趟变异）都没点到的目标，按 `grep -rlE` 在 `crates/` 下找（量的）：

| 没点到的目标 | 它引到的被改代码 | 只被它罩着吗 |
|---|---|---|
| `crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` | 核心层停下与补跑一路（`crates/singlefs-core/src/background_compute.rs:1516`、`:1646`）；该文件有三处 `LeftStopped` 断言（`crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:334` 一带） | 不是只被它罩：核心层 `--lib` 里 `crates/singlefs-core/src/background_compute.rs:2908`、`:3022`（无切换同错照 `LeftStopped`）也罩（两趟日志里核心层 `--lib` 都跑了）；验证包这一份是端到端的，推的 |
| `crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs`、`…_switching_backends_between_batches.rs` | 一批出错放弃、只删意图的发布的落盘（改法 5 的落盘那一半） | 重复键放弃一批那一次只删意图的发布的**落盘形态**，我没找到别的已跑目标逐状态枚举它；验证包 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs:1060` 只断言结局（推的）。这两个目标归提交时的崩溃枚举门禁，不归合入后验证 |
| `crates/singlefs-background-gpu/tests/gpu_gc_formal_contracts_through_the_host.rs` | 类型换型改了对被验函数（`singlefs_replay_prefix::gpu_gc_proofs` 一族）的全部调用点，取 `.0` 交进去 | 推不出：我没读它引哪几个被验函数；第二趟只点了 `gpu_gc_capacity_contracts_through_the_host` |
| 运行库另外四个集成目标 | `crates/singlefs-mount-runtime/src/lib.rs` 的读口 | 不引 GPU 路径；不是 |
| 两个多卡真卡对拍 | 多卡分活（`plans_of_a_batch_filling_the_gpus_in_order`）与换类型之后的占用 | 不是只被它们罩：本包 `--lib` 里有替身多卡的同类用例（`one_gpu_short_of_the_smallest_piece_of_the_batch_is_left_out_and_the_other_gpu_takes_the_whole_batch` 在 15444 日志第 64 行作为变异行的筛选词）；真卡上的多卡行为是没验 |
| 四段 compile fail 文档测试 | `ChunkFootprint::bytes_held_by_a_chunk` 的参数换反编不过 | **只被没在合入后跑的那一趟罩着**：只在 15441（副本）跑过，合入后验证不跑文档测试；换型这一批的主要防混用钉子就是它们 |

## 五、正文「四、检查单」八格逐格回（关的是哪几条 5）

回「成立」「不成立」「推不出」。同一格下先写判，再写依据。

1. 走不到的理由二次查实：**成立**（读码推的；一处正文的前提说错了，下面更正）。
   - `batch_not_split_over_the_gpus` 里那句「参与的卡一张都不剩只有两种」的断言（`crates/singlefs-background-gpu/src/background_gpu.rs:3181`）走不到：参与的卡为空、又没有开不成、量代价出错、上限不用三样，只有告知一张卡都不参与之后、下一组换上之前（`:2661`–`:2682`）。这期间核心层生效的是 `cpu`（`crates/singlefs-core/src/background_compute.rs:1775`），答开不成那一支不换生效的那一组（`:1819`–`:1824`），所以核心层不走 GPU 一路问它。这条约定只靠两边的文档维系（`crates/singlefs-background-gpu/src/background_gpu.rs:3157`–`:3162`），没有会红的检查；今天它被 `crates/mutations.tsv` 第 7773 行（「C996 实现答开不成时核心层把那一组记成生效」）钉着，该行在 15341 里被抓到（log-15341.txt 的名字列表，量的）。
   - `gpus_opened_at_a_switch` 里的 `debug_assert!`（`crates/singlefs-background-gpu/src/background_gpu.rs:1435`）：**正文说「release 里不判」，不对**：仓根 `Cargo.toml:27` 写 `debug-assertions = true`（量的，`grep -nF 'debug-assertions = true'`），release 同样判。前提被破坏时的后果（推的）：这句在切换开卡线程里、在接住 panic 的那一层之外、持着开卡那一步的锁（`crates/singlefs-background-gpu/src/background_gpu.rs:2970`），panic 会毒化那把锁，线程不送结局，两批之间那一问走到 `:2772` 的 `resume_unwind`、在会话锁里 panic。走得到吗：三处起线程的地方都先查过累计试满——交设置那一刻 `:2636`、为在等的那一组起之前 `:2862`、起不了线程到了间隔再起那一支 `:2735`（那一支没再查，但没有线程在跑、计数没动，且头一张的计数起不了线程那一刻已验过小于上限，`:3008`–`:3012`）；上一次切换的开卡还在跑时新交来的一组只记下、不起线程（`:2647`）。走不到，没有造模型。
   - 新 `panic!`：`GpuMemoryInBytes::plus`（`crates/singlefs-core/src/background_compute.rs:102`）、`CopiesOnTheGpu::bytes_of_the_copies`（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:132`）、`EntryCountOfAChunk::bytes_of_the_entries`（`:162`）、`input_bytes_of_the_entries`（`:177`）、输入长度 `plus`（`:70`）、`minus`（`:82`）。顶替的原式是裸 `u64` 的 `+`、`*`、`-`；仓根 `[profile.release]` 开着 `overflow-checks = true`（`Cargo.toml:25`，量的），所以原式在 debug 与 release 里溢出都 panic，同样。`minus` 顶替的是一片的「终点减起点」（diff 里 `self.input_bytes.end - self.input_bytes.start`，今天只在 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:479` 一处调，量的），下溢原来同样 panic；一片的终点不小于起点由切片函数建出来（推的，没造反例）。`of_this_many_*` 里的 `expect` 顶替的是各处原来的 `u64::try_from(…).expect(…)`，只在 `usize` 比 `u64` 宽的平台上走得到。
2. 类型与名字：**不成立**。
   - 新类型内值都是 `pub`：`crates/singlefs-core/src/background_compute.rs:76` 的 `GpuMemoryInBytes(pub u64)` 与 `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:44` 一族。签名处交反编不过（15441 里四段 compile fail 钉着，只在合入前跑过，见第四节），但取 `.0` 再包成另一个类型是编得过的；非测试代码里这种取包在边界处很多（如 `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:449` 一带的 `memory_limit.0.saturating_sub(…0)`），混用靠约定，不是靠类型。
   - 名字与语义对不上：`FailedOpeningsOfEachGpuInThisMount`（`crates/singlefs-background-gpu/src/background_gpu.rs:1327`）与 `OpeningsInThisMountUsedUp { failed_openings }`（`:165`）读起来是「一次挂载里的开卡次数预算用完」，而计数是连着开不成、开成清零、只记切换开卡、叫停不记（`:1372`、`:1383`）。名字说预算，语义是连败数；建议名字带上「连着」与「切换」（命名归实现员，我只判对不上）。
   - 实现员报告「留着的裸整数」十条（`~/.local/share/singlefs/work/bggpu-types-batch5/result-2.md` 第 108–120 行）：被验包的接口、CRC 算术、切片下标、核里的字、位置、局部计数、测试替身、测试数据、真卡测试局部量、不归显存的计数；按那份清单读，两个语义不同的量并排是裸整数的，我没找到（没逐处读代码，推的）。
   - `_ =>`：diff 里新加的行没有（`grep -nE '^\+.*[^a-z_]_ =>' research/prompts/_c996-code-r3-diff.md` 零命中，量的）。
   - 新 `expect`：`GpuMemoryAccount::hold` 与 `release` 的 `expect` 写明了依赖「切批与记账必须一致」（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:699`，原有的）；`of_this_many_*` 的 `expect` 写「装得进 u64」，没写依赖「平台 `usize` 不比 `u64` 宽」这条不变量，推的，轻。
3. 不兼容旧代码与旧格式：**成立**。`grep -nE 'legacy|old_format|migrat|兼容|deprecated' research/prompts/_c996-code-r3-diff.md` 零命中（量的）；diff 里新加的行没有 `From<u64>`、`from(`、`new(` 这类收裸整数的转接（`grep -nE '^\+.*(impl From<u64>|impl From<usize>|fn from\(|pub const fn new\(|pub fn from_)'` 零命中，量的）。内值 `pub` 本身是一个收裸整数的入口，但那是类型的构造，不是为旧代码留的转接。
4. 不新建判定库：**成立**。diff 里新加的行没有 `File::`、`OpenOptions`、`create_dir`、`write_all`、`rocksdb`、`sqlite`（`grep -nE` 的命中只剩四行 `gpu_gc_*_proofs::…` 函数名，是我的正则里 `fs::` 撞上了 `proofs::`，量的，不是写文件）。
5. 所有任务多机可用：**成立**。新加的行没有读本机状态、写死地址与主机名（`grep -nE '^\+.*(std::env|env::var|127\.0|localhost|hostname)'` 零命中，量的）。真卡对拍在没有卡的机器上：对拍目标自己打「本次未跑」那一类（`research/prompts/` 日志里 `product-gpu-real-card-tests` 一格把它与通过分开数，三趟都报「没卡没跑的对拍 0 条」，即有卡；没卡时的分支我没跑，推的）。15315 在第二台、15444 在本机，两台各领到一张卡都过（log-15315.txt 与 log-15444.txt 「按需领卡」那一行，量的）。
6. 测试优先多线程：**成立**，记两处。
   - 新用例里有两处按 10 毫秒轮询等一个条件，至多 2000 次（diff 里 `std::thread::sleep(Duration::from_millis(10))` 两处，`research/prompts/_c996-code-r3-diff.md:2362`、`:2645`）：有上限的条件轮询，不是写死一个时长；仍是 sleep 轮询，不是通道或 `join`。
   - 进程级那张表在并行用例之间不互相看见：表是开卡器实例的成员（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:148`），不是 `static`（`grep -rn 'static .*OpeningsNotReturned'` 零命中，量的）；用例各自 `with_none_not_returned()` 现建（`crates/singlefs-background-gpu/src/background_gpu.rs:1236` 的构造，用例里调它至少五处）。一格一条用例：甲的十条各占一个函数（`:8690`–`:8800`）。
7. 带 --item 的命令与项：**不成立**。三批点名的名字（51、31、55，共 137 个，两两不重）逐个在 `crates/mutations.tsv` 第一列恰好 1 次，主工作区与冻结副本两棵树相同（量的，第四节）；c996-code-r2 判决改法 9 点名的九处各有一行（第一节）。不成立的是：新加的分支里还有三处按原文、替换文子串找不到行锚（推的）：`crates/singlefs-core/src/background_compute.rs:1787`（换上时清开不成留下的阈值；不清则开不成之后的下一组换上，阈值仍被旧的盖住）、`crates/singlefs-mount-runtime/src/lib.rs:946` 与 `:1193`（读口克隆、卸载时带走）、`crates/singlefs-background-gpu/src/background_gpu.rs:1432`–`:1434`（一次切换开卡至多开「剩下的次数」，别的行只锚了清零与不记两头）。
8. 新崩溃流多罩了什么：**成立**（推的，没跑）。重复键放弃一批那一次、开卡期间出错停下时的落盘，都是同一个只删意图的发布：`crates/singlefs-core/src/mounted_session.rs:5192` 的 `drop_the_live_intent_on_a_stop`，与别的出错放弃一批同一条；变的是发生的次数，不是落盘的形态。没点到崩溃枚举目标（`crash_enumeration_compaction_relocation_stream_on_the_gpu_path.rs`、`…_switching_backends_between_batches.rs` 两个都不在合入后验证的项里），所以「已有崩溃流罩得到」是读码结论，归提交时的崩溃门禁。

## 要主 agent 跑的命令

没往统一任务服务交任何任务；下面三条判决要引的读数我都没有。每条：在哪个目录起、跑完看哪几行。

1. 没被合入后验证点到、罩着核心层停下与补跑的验证包目标。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 compaction_on_the_cpu_while_stopped_after_a_gpu_failure; 范围=1 个测试目标; 为什么=合入后验证五趟都没点它，它端到端罩着停下之后在 CPU 上补跑那一路，这一轮改了停下与补跑阈值的取法' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-verification --release --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure`
   - 目录：仓根 `/home/fy5090/code/singlefs`（补丁已合进主工作区）。
   - 看：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`，日志里 `test result: ok`；被范围声明闸拒（点到崩溃枚举）就照拒绝说明缩域或写全域理由，不换写法。
2. 对被验函数的全部调用点取 `.0` 的那一族，换类型之后没点到的对拍目标。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的测试目标 gpu_gc_formal_contracts_through_the_host; 范围=1 个测试目标; 为什么=类型换型改了对被验函数的调用点，合入后验证第二趟只点了 gpu_gc_capacity_contracts_through_the_host' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-background-gpu --test gpu_gc_formal_contracts_through_the_host`
   - 目录：仓根；看 `exit=0` 与 `test result: ok`。
3. 「换反编不过」的四段 compile fail 文档测试在合入后的树上跑一次。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的文档测试 memory_limit_and_chunks; 范围=1 个测试目标; 为什么=这四段只在实现员副本里跑过，合入后验证不跑文档测试' task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo test -p singlefs-background-gpu --doc memory_limit_and_chunks`
   - 目录：仓根；看 `exit=0`，日志里 `test result: ok. 6 passed`（4 段 compile fail、2 段对照，与 15441 同形）。

## 没做什么

- 没往统一任务服务交任何任务，上面三条与判决要引的数都没复跑；本报告里没标「量的」的数一律是推的。
- 没造模型、没写模型目录；第三节「未交二、三、四」与第五节第 1、8 格的后果都是读码推的，没造反例。
- 没读 `mounted_session.rs` 的组批顺序与 `crates/singlefs-core/src/region_scan.rs`，所以重复键到底是不是确定性的、同一批各段谁先谁后，我没判。
- 没逐处读 15 份产品文件的 diff 全文：核心层与开卡一路、甲乙丙、癸与换类型的几处读了；`region_record_kernels.rs`、`checksum_kernels.rs` 的换类型只看了几处代表（份数乘法、断言左边）。外壳三段「真建的缓冲」那句断言左边换了算法，我看到描述符一份原来按裸长度、今天按「补到整字、空的占 4 字节」算（`research/prompts/_c996-code-r3-diff.md:754`–`:757`），数值不变只在描述符长度是 4 的倍数且非空时成立（推的，没量）。
- 没核对 c996-code-r2 攻方模型的 18 条用例在今天的代码上翻没翻面（那是攻方腿的活）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c996-code-r3-defense` 里只有任务日志与名字列表，没有仓副本与编译目录，没有要删的东西。
