# c996-code-r2：C996 代码轮第一轮打中之后的修补，代码轮第二轮正文（2026-11-12）

## 一、题面

第一轮判决 `research/prompts/c996-code-r1-main-verification.md` 打中甲、乙、丙、丁、戊（两种）、庚、壬、癸，辛按打中办；「三、改法」九条都被攻过零轮。实现员照九条做了修补，已打进主工作区。这一轮攻的是修补本身：它修没修掉第一轮打中的那几格、有没有带出新的毛病，并复核第一轮「这一轮不下结论」的那四样（泄漏账重复扣或漏扣、参与的卡下标错位、一批之内换设置、停着的 GC 被悄悄放开）。

压着的条款是 D26（后台整理与放置回收） 已定项 13 第 1、2、4 条的现行文本（第一轮之后写回过：第 2 条加了切到 cpu 时关掉全部开着的卡，第 4 条改成参与的卡一律照重开、写明头一次不算重开、加了起不了线程与 panic 与前一次没回来与卸载取消与切换关卡泄漏五句）。行号去 `.claude/kb/decisions/26-后台整理与放置回收.md` 现查。

改动全文：`research/prompts/_c996-code-r2-diff.md`（修补补丁里 `crates/*/src/` 下 7 份文件的块）。实现员报告：`~/.local/share/singlefs/work/c996-code-r1-fixes/result.md`。测试改动在补丁 `~/.local/share/singlefs/work/c996-code-r1-fixes/patch/crates.patch` 里。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 被判的产品代码（开工快照 `research/prompts/c996-code-r2-snapshot/`，十份，与第一轮同一份清单）：`crates/singlefs-background-gpu/src/` 下 `background_gpu.rs`、`background_gpu_compute.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`、`gpu_opener_of_this_process.rs`、`gpus_of_this_host.rs`、`lib.rs`、`memory_limit_and_chunks.rs`、`waiting_for_the_gpu.rs`；`crates/singlefs-core/src/background_compute.rs`；`crates/singlefs-mount-runtime/src/lib.rs`。这一轮的修补动了其中七份（diff 文件列的那七份），`lib.rs`、`memory_limit_and_chunks.rs`、`waiting_for_the_gpu.rs` 没动。同一批文件上别的会话另开着一轮（审核四条里另三条的落点），两轮快照同一棵树；腿只攻这一轮 diff 里的块与它们和周围代码的交叉。
- 修补照实现员报告「逐条改了什么」的说法（主 agent 没逐行复核，腿要核）：两批之间那一问改成交回三种（还在开卡、换上了并带这一组用不用得上、开不成）；核心层多记一格交给了实现而实现还在开卡的设置，这期间照旧按切换之前那一组算，一批出错照这次切换解除停下，实现答换上了才生效；开卡结局按交设置的次序号认；新方法 `use_no_gpu_from_the_next_batch`，切到 cpu 时关掉全部开着的卡、叫停在开的切换开卡，切回 gpu 的开卡期间生效的仍是 cpu；最近那一组开不成时关掉此刻生效那一组之外开着的全部卡；起不了开卡线程也算一次没开成、间隔之后再起；开卡线程里接住 panic 记成那一次没开成；同一张卡前一次开卡没回来就不再起下一次；切换开卡线程带叫停信号，卸载和切到 cpu 时置上；切换关卡时的泄漏另累计进 `GpusLeakedWhenClosedAtASwitch`，`OpenedBackgroundGpuCompute` 多一个字段。
- 验证到的数（量的，都在实现员的副本上，副本与打进主工作区的补丁同源）：clippy 四个包任务 14268、build 任务 14269 退 0；核心层 lib 里 `background_compute` 模块「12 passed; 0 failed」（任务 14270）；产品 GPU 包 lib 里名字带 `background_gpu` 的「74 passed; 0 failed」（任务 14271）；替身对拍「11 passed」（任务 14265）；开卡器「11 passed」（任务 14266）；e202 单测「87 passed」（任务 14267）；证红核心层「点名 7 条……抓到 7 条」（任务 14294）、产品 GPU 包「点名 18 条……抓到 18 条」（任务 14331）。主工作区上的合入后验证任务 14424、41 行变异点名复跑任务 14425 交了、还没出结局。
- 实现员停下交主 agent 的三件（报告「停下交主 agent 的设计问题」原文）：① 新的三种「这一次没开成」都报成 `GpuFailureCause::DeviceOrPipelineDidNotStart { reason }`，没各起成员；② `crates/singlefs-core/src/compaction_plan.rs` 第 637、642 行文档还写旧的切换生效时刻，没动；③ 两处照条款自己读的：(a) 实现答开不成时核心层把那一组记成生效的、GC 停着，(b) 叫停之后开卡线程不再睡重开的间隔，正在开的那一次照旧开完，开成的卡在那条线程上照常丢。
- 第一轮攻方腿的十条模型用例在 `research/prompts/c996-code-r1-opus-model/src/`，第一轮没有改法那一臂的读数在任务 14158（判决第二节逐条引了原行）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 第一轮的修补本身：甲、乙、丙、丁、戊 ①②、庚、辛、壬、癸逐条，修补在那一格上还中不中。第一轮十条模型用例的历史在修补之后的代码上各走一遍，读数翻没翻面；实现员把它们收成的回归用例断言的是不是条款要的结局、还是照新实现现写的。
2. 修补带出的新状态：核心层现在有三组设置（生效的、请求了没交的、交了而实现还在开卡的），实现有三种答法。逐个交错判：开卡期间连着来两次、三次切换；开卡期间切到 cpu 再切回；开卡期间出错停下再收到切换；叫停与开卡结局同时到；实现答换上了而核心层那一格已被新请求盖掉；开不成时核心层把那一组记成生效（设计问题 ③(a)）之后补跑阈值、`in_effect()` 的读者读到的对不对。有没有一种交错让两边对生效的那一组认得不一样、让一批之内换了设置、或让停着的 GC 被放开。
3. 从未被看过的地方与第一轮没下结论的四样：切到 cpu 关掉全部卡时有提交还在跑的卡、泄漏账与新加的 `GpusLeakedWhenClosedAtASwitch` 会不会同一张卡记两次或漏记；关掉之后 `participating` 与各卡预算的下标；叫停之后开卡线程上开成的卡丢掉时记不记账；卸载与切到 cpu 同时发生；`use_no_gpu_from_the_next_batch` 在选 cpu 挂载、从没选过 gpu 的会话上被调。
4. 产生结论的方法：新的三种没开成共用一个成员（设计问题 ①）调用方分不分得开该做的决定；前一次开卡没回来就不再起，靠什么知道它回来了没有、它永远不回来时这张卡是不是永远开不成、重开次数怎么数；接住 panic 用的办法会不会把不该吞的 panic（本包自己的不变量断言）也吞掉。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」原文：「产品代码 `crates/<包>/src/` 里这一轮没有新加 `todo!`、`unreachable!`、`assert!`，也没有新加以 `Undecided`、`Unsupported`、`OutsideTheFirstVersion` 收尾的错误成员。改了口径、可能被问到的两处已有断言与 panic（不是新加的，照判决「检查单」那一格说明）：`crates/singlefs-background-gpu/src/background_gpu.rs` 的 `batch_not_split_over_the_gpus` 里已有的 `assert!`：告知一张卡都不参与之后、再交来一组换上之前调它会走到；核心层那时生效的选 `cpu`，`take_the_answer_on_the_settings_handed_in` 只在生效的或在等的那一组选 `gpu` 时问实现，三段也只在走 GPU 的批里调，产品里调不到；文档 `# Panics` 写明了这一种。同一份文件 `put_the_gpus_opened_since_the_switch_in_effect` 里已有的 `resume_unwind`：开一次卡里的 panic 已在开卡线程上接住，断开只剩本包自己的代码 panic。」腿照调用点逐个追这两处「调不到」成不成立。

### 检查单：类型与名字

新方法 `use_no_gpu_from_the_next_batch`、`take_the_answer_on_the_settings_handed_in`；新类型 `GpusLeakedWhenClosedAtASwitch`；两批之间那一问的三种答法的类型与成员名（diff 里现查）；`OpenedBackgroundGpuCompute` 新字段；交设置的次序号用什么类型。腿判次序号、重开次数、原因个数有没有共用裸整数，有没有 `_ =>`，新 `expect` 写没写清依赖哪条不变量；三种没开成共用 `DeviceOrPipelineDidNotStart`（设计问题 ①）算不算一个成员装了调用方要分开的决定。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支：两批之间那一问的旧形状整条换掉。腿现跑 `grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r2-diff.md`，并核旧的两种答法（`Option` 形状）有没有留着的转接。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在产品代码 `src/` 下。腿核 diff 里有没有写文件、开库的新路径。

### 检查单：所有任务多机可用

产品代码没有这一类。新写 23 条测试都用替身卡、现造输入，实现员报告「拒算的测试」写无。腿核新测试有没有读本机状态、写死地址。

### 检查单：测试优先多线程

新用例一条一个场景；涉及开卡线程的用例靠放开信号等，实现员写了证红里有一行约 80 秒才红（「C996 产品实现两批之间那一问等开卡开完」）。腿核有没有写死 sleep 的等法、那条 80 秒的用例为什么要这么久、有没有该拆而没拆的参数循环。

### 检查单：带 --item 的命令与项

本轮没有新的崩溃点重放项。crates 变异表追加 25 行、换锚点 16 行，共 41 行由任务 14425 带 `--item` 点名复跑（命令在实现员报告「要主 agent 跑的命令」末条）。腿核 41 个名字与 `crates/mutations.tsv` 逐字相同；第一轮点名没有变异行罩着的 8 处补全了没有（实现员写一处测不到：产品挂载传 `OF_THE_PRODUCT` 与 sleep）；这一轮新加的分支还有没有没罩着的。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没加崩溃流。腿判切换生效时刻改了之后，停下、解除、只删意图的那一次发布的落盘时机有没有已有崩溃流罩不到的新形态。

## 五、格

1. 第三节四条攻击面；第一轮打中的每一格回「修掉了」「还中」或「推不出」。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
