# c996-code-r1：产品 GPU 包只开参与的卡、开不成间隔重开、切换关掉不再点名的卡，代码轮第一轮正文（2026-11-12）

## 一、题面

C996（开卡打开本机全部独显，服务记 wrong-card）的改法由用户分几次弹窗定案，现行文本是 D26（后台整理与放置回收） 已定项 13 第 2 条末句与第 4 条（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 251、253 行）：挂载只开参与的卡；切换之后不再参与的卡切换生效时关掉；挂载那一刻点名的卡或挂着时切换点到的卡开不成，间隔 1 分钟重开、最多重开 3 次；切换在会话锁外开新卡，重开期间照旧用切换前那组卡，开成了下一批起换上，重开 3 次都不成 GC 停下、报警报出卡与每次开不成的原因、不自动再试；挂载那一刻点名的卡重开 3 次都不成才拒绝挂载。另一条用户定案：核心层「挂载那一刻没开成的卡」一路（`devices_not_opened_at_the_mount` 与对应拒绝成员）删掉。这一轮判的是代码做的是不是这几句话。

改动全文：`research/prompts/_c996-code-r1-diff.md`（两份补丁里 `crates/*/src/` 下的块，14 份文件块）。实现员报告两份：`~/.local/share/singlefs/work/c996-open-only-participating/result.md`（只开参与的卡）、`~/.local/share/singlefs/work/c996-reopen-and-close/result.md`（重开、关卡、删没开成一路）。测试改动在两份补丁目录的 `crates.patch` 里。

## 二、实现今天的样子（主 agent 的观测，2026-11-12）

- 被判的产品代码（开工快照 `research/prompts/c996-code-r1-snapshot/`）：`crates/singlefs-background-gpu/src/` 下 `background_gpu.rs`、`background_gpu_compute.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`、`gpu_opener_of_this_process.rs`、`gpus_of_this_host.rs`、`lib.rs`、`memory_limit_and_chunks.rs`、`waiting_for_the_gpu.rs`；`crates/singlefs-core/src/background_compute.rs`；`crates/singlefs-mount-runtime/src/lib.rs`。主工作区的 `background_gpu.rs` 在这两份补丁之后又被别的会话改过（一个测试辅助函数的返回类型，clippy 修正），那些块不归这一轮；腿只攻 diff 里的块与它们和周围代码的交叉。
- 重开规矩在 `waiting_for_the_gpu.rs`：`INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED` 60 秒、`REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST` 3，`gpu_opened_with_the_reopenings` 开一次、开不成等间隔再开，至多开 1 + 3 次，都不成交回每一次的原因；`longest_wait_of_one_gpu` 是一张卡连同重开最多等多久（5 分钟上限下 23 分钟），产品开卡器 `gpu_opener_of_this_process.rs` 的每卡期限改成它。
- 挂载：`gpus_of_this_host.rs` 的 `gpus_opened_at_the_mount` 只对参与的卡开（经 `gpu_opened_under_its_memory_limit` 与重开），不参与的卡不开、照列进卡清单；参与的卡都开不成拒成 `BackgroundGpuOpenRefused::GpuUnusable { device, causes_of_each_opening }`。
- 切换：`background_gpu.rs` 的 `use_the_settings_from_the_next_batch` 点到的卡都开着就当场换上（`put_the_settings_in_effect`，一开头关掉这一组没点名的开着的卡，`close_the_cards`）；点到没开的卡就记下这一组、`start_the_opening_at_a_switch` 起一条线程开（`gpus_opened_at_a_switch`），调用当场返回。`put_the_gpus_opened_since_the_switch_in_effect` 在两批之间取结局：还在开就不动；开完了按实现员报告第 1 条那四种情形处理。关卡时上面还有等超过上限、没做完的提交的卡整张泄漏，记进进程泄漏账（`memory_leaked_in_this_process`）。
- 核心层：`BackgroundGpuCompute` 新方法 `put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`（缺省 `None`）；`put_the_switch_requested_from_the_next_batch_in_effect` 末尾问一次，交回开不成就照「切到一张卡都用不上的设置」同一个停法停（`replace_the_tasks_stopped`），报警成员 `GpuUnusableUnderTheSettingsHandedIn`，新出错成员 `BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`。`devices_not_opened_at_the_mount`、`GpuDeviceNotOpenedAtTheMount`、`ParticipatingGpuDeviceNotOpenedAtTheMount` 删了，实现员报告第 3 条的 grep 在 `crates/` 与 `litmus/` 下零命中。
- 验证到的数（量的）：合入后验证任务 13924，三个包按条复用「现判 2、复用 746、拒算 1」测试全绿；产品 GPU 包真卡对拍跑到 29 条过，另有两个要 2 张、3 张卡的目标本次未跑（这一趟只拿到 1 张）；同一趟 clippy 全特性组合红在验证包 `crash_amplification.rs`（不在这一轮的文件里，快照撞上别的会话改到一半），照此刻的树单交验证包带 gpu 特性的 clippy 任务 13992 退 0。26 行变异点名复跑任务 14014、变异表格任务 14018 交了、等结局。
- 实现员停下交主 agent的设计问题（第二份报告「停下交主 agent 的设计问题」一节，原文照抄进这一轮的攻击面）：①「最多重开 3 次」按头一次之后再开 3 次做，一共开 4 次、报 4 个原因；② 挂载最多被一张开不成的卡挡 23 分钟，开卡器外层期限是「枚举 5 分钟 + 卡数 × 23 分钟」；③ 起不了开卡线程当成头一张卡开不成，1 个原因、不重开；④ 开卡还在开时又来一次切换，记下最近那一组、等这一次开完再看，为更早一组开的、开不成的不报；⑤ 从 `cpu` 切回 `gpu`、点到没开的卡，开着的这段时间核心层认为生效的是新那一组、实现照它上一次换上的那一组卡；⑥ 卸载时开卡线程还在开不等它，开成的卡在那条线程上析构；⑦ 切换关卡时泄漏的卡只记进进程泄漏账，不进丢后台 GPU 那一刻的泄漏报告 `gpus_leaked_when_dropped`；⑧ 切换开卡每一次的上限沿用挂载的 5 分钟、等 wgpu future 用这一组的 `background_gpu_wait_limit`；⑨ 开不成、GC 停下之后实现里一直记着「开不成」，直到下一次 `use_the_settings_from_the_next_batch`。
- 压着的条款：D26（后台整理与放置回收） 已定项 13 第 2 条、第 4 条；同一项第 1 条「一批之内不换」；`.claude/kb/mount-parameters.md` 第 23 行（`background_gpu_devices` 那一行）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 过度外推与越位：代码做的是不是第一节那几句话。逐句对：只开参与的卡（没点名时「本机全部独立显卡」的口径与第 2 条一致没有）；切换在会话锁外开、重开期间用切换前那组卡、开成了下一批起换上（是不是一批之内不换，第 1 条）；重开 3 次都不成 GC 停下、报出卡与每次的原因、不自动再试；挂载重开 3 次都不成才拒；切换生效时关掉不再参与的卡。实现员设计问题 ①（一共开 4 次）读得对不对。
2. 内部矛盾与并发：实现员自己定的那几处（设计问题 ③④⑤⑥⑨）。腿判：开卡线程起不来；开卡开到一半又来一次切换、再来一次；从 `cpu` 切回 `gpu` 又切回 `cpu`；卸载时开卡线程还在开、开成的卡在那条线程上析构时进程泄漏账与开卡器的「上一次开卡还挂着」那道闸是不是对得上；开不成之后核心层停着、实现记着，下一次通过校验的切换是不是两边一起解除。有没有一种交错让核心层认为生效的那一组与实现在用的卡不一致到分错活、在关掉的卡上提交、或停着的 GC 被悄悄放开。
3. 哪一条最脆弱：关卡与泄漏账（设计问题 ⑦）。关掉的卡上有等超过上限的提交时整张泄漏、记进进程泄漏账，之后挂载这张卡先扣掉它；不进 `gpus_leaked_when_dropped`。腿判：这和卸载那一套（第 4 条「卸载时不等还在跑的提交……整张泄漏、不析构」）是不是同一个规矩；关掉之后再点它重开，泄漏账有没有重复扣或漏扣；参与的卡按标识重找位置时下标有没有错位。
4. 期限与挂载（设计问题 ②⑧）：开卡器每卡期限改成 23 分钟之后，挂着时切换开卡与挂载开卡用的期限各是什么、有没有一处还是按单卡 5 分钟算而让重开做不满 3 次；`longest_wait_of_one_gpu` 的算式和真走的等待对不对得上。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」两份原文：第一份见 `~/.local/share/singlefs/work/c996-open-only-participating/result.md`「走不到的理由」一节；第二份原文「产品代码 `crates/<包>/src/` 里这一次没有新加 `todo!`、`unreachable!`、`assert!`，也没有新加条款没定的错误成员（名字以 `Undecided`、`Unsupported`、`OutsideTheFirstVersion` 收尾的）。`batch_not_split_over_the_gpus` 里原有的那条 `assert!` 只改了文案（三种原因改成两种：开不成那一种在上面先交回，量代价出错也在上面先交回）。新加的 `.expect` 不在本节判的种类里，各自的不变量写在文案里：`start_the_opening_at_a_switch` 的「至少一张」与开卡线程上锁那一处、`note_the_gpu_closed_at_a_switch`、`failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in`；`of_the_settings` 的 `expect` 改了文案。」腿核那条 `assert!` 改文案之后判的三种变两种成不成立，几处 `expect` 依赖的不变量成不成立（开卡线程上锁那一处在线程 panic 之后会不会毒化）。

### 检查单：类型与名字

新常量 `INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED`、`REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST`；新类型 `ReopeningsOfAGpuNotOpened`、`OpeningOfAGpuAtASwitch`、`WaitForTheOpeningAtASwitch`；新函数 `gpu_opened_with_the_reopenings`、`longest_wait_of_one_gpu`、`gpu_opened_under_its_memory_limit`、`gpus_opened_at_a_switch`、`start_the_opening_at_a_switch`、`put_the_gpus_opened_since_the_switch_in_effect`、`put_the_settings_in_effect`、`close_the_cards`、`note_the_gpu_closed_at_a_switch`、`put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`、`failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in`；新出错成员 `GpuNotOpenedAfterTheReopenings`、`GpuNotOpenedAfterTheReopeningsAtTheSwitch`、`GpuFailureCause::NotOpenedAfterTheReopenings`；`GpuUnusable` 的字段 `cause` 改成 `causes_of_each_opening`。腿判重开次数、开卡次数、原因个数有没有共用裸整数，有没有 `_ =>`，`expect` 写没写清依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支：核心层「挂载那一刻没开成」一路整条删了，没有开关；`BackgroundGpuCompute` 新方法缺省交回 `None` 是给不开卡的实现用的，腿判它是不是为旧实现留的兼容。腿现跑 `grep -nE 'legacy|old_format|migrat|兼容' research/prompts/_c996-code-r1-diff.md`。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在产品代码 `src/` 下，产品代码不写判定。腿核 diff 里有没有写文件、开库的新路径。

### 检查单：所有任务多机可用

产品代码没有这一类。第一份补丁的真卡测试新加了读「这个进程在哪张卡上」的断言（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs`，补丁之后测试目录被别的会话搬进了这个新包，补丁里的路径还是搬之前的；经 nvidia-smi 读），读不到时打 `NOT_RUN_NO_GPU_ADAPTER` 按本次未跑算；腿判它在第二台上读得到吗、有没有写死本机的卡序号或 UUID。

### 检查单：测试优先多线程

新单测都是一条用例一个场景，各自造替身卡与状态，由测试框架并行跑；两条切换用例自己起开卡线程，等的上限 20 秒。腿核有没有该拆而没拆的参数循环、有没有写死 sleep 而不是放开信号的等法。

### 检查单：带 --item 的命令与项

本轮没有新的崩溃点重放项或崩溃枚举用例。crates 变异表新写与改锚点的 26 行由任务 14014 带 `--item` 点名复跑（命令在第二份实现员报告「要主 agent 跑的命令」第 12 条）。腿核那 26 个名字与 `crates/mutations.tsv` 里的行逐字相同，有没有新行为没有变异行罩着（实现员报告写了两处没罩着：产品挂载传 `OF_THE_PRODUCT` 与 `std::thread::sleep`、两份核那一层用 `NotAtAll` 那一句）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没加崩溃流与崩溃枚举用例。开卡、关卡、重开都不写盘；腿判切换期间 GC 停下与恢复有没有让已有崩溃流罩不到的新落盘形态（停着时那一次只删意图的发布照旧过第 5 条那道门）。

## 五、格

1. 第三节四条攻击面。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
