# c996-code-r3：C996 开卡与切换两轮判决的修补、产品 GPU 包审核三条的修补与切批记账换类型，代码轮第三轮正文（2026-11-13）

## 一、题面

这一轮是 C996（开卡打开本机全部独显，服务记 wrong-card）这一题的第三轮、最后一轮，同时是产品 GPU 包审核修补（`bggpu-audit-fix-code-r1`）那一题的第二轮：两题动的是同一批文件，合用一份开工快照、一轮攻。

前两份判决：

- `research/prompts/c996-code-r2-main-verification.md`「三、改法」第 1 到 10 条（新一、新二、新三、新四、新七、壬、癸、设计问题 ①、新五、新六、没有变异行罩着的分支、文档与注释），都被攻过零轮。
- `research/prompts/bggpu-audit-fix-code-r1-main-verification.md`「三、改法」第 1、2、3、5、6 条（甲：切换时量代价撞上还在跑的提交；乙：选段打分那一段重复的段；丙：冲掉待写的空提交等不到；重复键到核心层只放弃这一批；切批与记账的字节数、条数换成各自的类型），都被攻过零轮。

这些改法都已写进产品代码、合进主工作区。这一轮照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮的分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。攻的是前两轮站住的形态与这两批修补本身；这一轮里新冒出来的零轮形态不再开一轮，写进判决的交用户表。

压着的条款是 D26（后台整理与放置回收） 已定项 13 第 1、2、3、4、8 条的现行文本（2026-11-13 写回过：第 4 条补了开不成之后生效的照旧是切换之前那一组、开卡期间旧卡出错停下并报警而新卡开成解除、同一张卡一次挂载里连着 4 次开不成不再试、前一次没回来的闸管到进程、切换关卡的泄漏运行库读得到；第 8 条补了一批里同一个条目键出现两次只放弃这一批）。行号去 `.claude/kb/decisions/26-后台整理与放置回收.md` 现查。

改动全文：`research/prompts/_c996-code-r3-diff.md`（15 份产品文件相对上一轮开工快照的 `diff -u`，`lib.rs` 没有改动）。实现员报告五份：`~/.local/share/singlefs/work/bggpu-c996-batch3/writer/result.md`、`~/.local/share/singlefs/work/bggpu-c996-batch4/result.md`、`~/.local/share/singlefs/work/bggpu-types-batch5/result.md`、`result-2.md`、`result-3.md`（同目录）。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

- 被判的产品代码（开工快照 `research/prompts/c996-code-r3-snapshot/crates-sha256.txt`，15 份）：`crates/singlefs-background-gpu/src/` 下 `background_gpu.rs`、`background_gpu_compute.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`、`checksum_kernels.rs`、`gpu_opener_of_this_process.rs`、`gpus_of_this_host.rs`、`lib.rs`、`live_slot_kernels.rs`、`memory_limit_and_chunks.rs`、`region_record_kernels.rs`、`two_kernel_reconciliation.rs`、`waiting_for_the_gpu.rs`；`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/compaction_plan.rs`；`crates/singlefs-mount-runtime/src/lib.rs`。冻结副本在 `~/.local/share/singlefs/work/c996-code-r3/repo/`（15 份与快照 `sha256sum -c` 全对），腿读代码读冻结副本或主工作区都行，引行号以快照那一刻为准。`compaction_plan.rs` 这一轮只判文档那两处（第 637、642 行一带，切换生效时刻的说法），它别的部分别的会话在改、不在射程。
- 两批修补照实现员报告的说法（主 agent 没逐行复核，腿要核）：
  - 开不成时核心层不把那一组记成生效：`crates/singlefs-core/src/background_compute.rs` 的 `take_the_answer_on_the_settings_handed_in`；补跑阈值另记一格取请求给的。
  - 交给了实现、实现还在开卡的期间一批出错：停下、每次叫起报警、照阈值补跑，实现答换上了那一刻解除（`stop_the_task_after_a_compute_error` 一带；请求了、还没交给实现的切换照旧在停下那一刻算数、当场解除，两种分开）。
  - 同一张卡连着开不成满 4 次之后不再开：`FailedOpeningsOfEachGpuInThisMount`（只记切换开卡，开成清零），原因成员 `GpuFailureCause::OpeningsInThisMountUsedUp`。「开成清零」是主 agent 照「一张卡至多开 4 次」读的，用户没单独定。
  - 「前一次开卡没回来」那张表挪到进程级：`OpeningsNotReturnedInThisProcess`（`gpu_opener_of_this_process.rs`）。
  - 接住 panic 挪进挂载与切换共用的开一张卡那一步；起不了开卡线程、开卡 panic、前一次没回来、叫停、试满不再试各起一个原因成员。
  - 运行库读切换关卡泄漏的累计：`crates/singlefs-mount-runtime/src/lib.rs` 的 `gpus_leaked_by_the_background_gpu`（`MountedRuntime` 与 `UnmountedPool` 两处）。
  - 切换时量代价之前先等账上还在跑的那次提交（甲）；等不到报 `BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch`。
  - 一批里同一个条目键出现两次：三段都交回 `SameEntryKeyTwiceInOneBatch`（选段打分那一段在整段清单断言之前先查重复，乙），到核心层是 `BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`，只放弃这一批、任务不停、不报警；那一批的意图照放弃一批的规矩删（落点在 `crates/singlefs-core/src/mounted_session.rs`，那份文件不在这一轮快照里，腿只读、不判它别的部分）。
  - 冲掉待写的空提交等不到：单独的原因，算进超时数（丙）。
  - 类型：字节数用核心层的 `GpuMemoryInBytes`（加了 `checked_add`、`checked_sub`、`plus`、`Sum`）；份数 `CopiesOnTheGpu`、一段里几条 `EntryCountOfAChunk`、记录表条数 `AllocationRecordCountOfATable`、一张卡分到几个段 `ClusterSegmentCountOfAGpu`、主机上一条输入的字节长度 `InputLengthOnTheHostInBytes`；核心层报告类型里的字节字段换成 `GpuMemoryInBytes`，`InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice` 的字段改名成 `smallest_piece_bytes`。实现员说这一批不改行为；外壳三段「真建的缓冲与切批按的占用一致」那句断言的左边换了算法（原来把各缓冲内容的长度直接相加，现在每份按建缓冲时的 `input_buffer_bytes` 算再加），它推的是数值不变。
- 验证到的数（量的；任务号是统一任务服务的）：
  - 第一批（两份判决的改法）合入后验证任务 15315 退 0：「测试全绿（9 个测试二进制，共 131 条测试；跑了 9 项（4 个包）；按条复用：现判 130、复用 753、拒算 1）」，fmt、clippy、build 过，产品 GPU 包真卡对拍「按条现判 28、复用 0、拒算 1」；同一批补丁前一趟任务 15185「共 309 条测试……现判 308、复用 570」。变异行点名复跑任务 15341「计数：抓到 51 条、没红 0 条、无效 0 条、锚点失效 0 行（共 51 条）」；C996 第一轮修补的变异行里剩下的 31 行任务 15382「计数：抓到 31 条、没红 0 条、无效 0 条、锚点失效 0 行（共 31 条）」。
  - 类型那一批合入后验证任务 15444 退 0：「测试全绿（11 个测试二进制，共 308 条测试；跑了 11 项（4 个包）；按条复用：现判 307、复用 480、拒算 1）」，真卡对拍「按条现判 28、复用 0、拒算 1」；变异行点名复跑任务 15450「计数：抓到 55 条、没红 0 条、无效 0 条、锚点失效 0 行（共 55 条）」；文档测试任务 15441「6 passed」（4 段 compile fail、2 段对照）。
  - 真卡对拍那一格有 2 处报「本次未跑」（任务 15185、15315、15444 的同一格），哪两处主 agent 没逐条读。
- 没验的：建缓冲显存不足、冲掉待写那条路在真卡上的行为（丙的改法只在替身卡上钉过）；这两批合入之后 Verus 没跑（归提交时的门禁）。
- 前两轮攻方腿的模型：`research/prompts/c996-code-r2-opus-model/`、`research/prompts/bggpu-audit-fix-code-r1-opus-model/`，没有改法那一臂的读数在任务 14649 与 14625（原行存在 `research/prompts/c996-code-r2-main-checks/attack-model-lines-task-14649.txt` 与 `research/prompts/bggpu-audit-fix-code-r1-main-checks/attack-model-lines-task-14625.txt`）。两份判决都要求改完之后同一批输入上读数翻面才算换到东西：这一轮攻方腿把那些模型用例在今天的代码上各走一遍（命令交主 agent 跑）。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 「已定」撑不撑得住：两份判决判为站住的那几格（c996-code-r2 判决第二节里没打中的行；bggpu-audit-fix-code-r1 判决里 G01、G03 的校验和与区域记录两段、G04 的算术）在这两批修补与换类型之后还站不站得住。换类型有没有哪一处把两个量换反了、把原来会 panic 的溢出换成了不 panic 或反过来、把 `checked_` 的失败支改了去向；那句换了算法的断言在哪种输入上左边会与原来不同。
2. 改法换没换到东西：前两轮的模型用例逐条在今天的代码上重跑，读数翻没翻面；实现员收成的回归用例断言的是条款要的结局、还是照新实现现写的。逐条回：c996-code-r2 的新一、新二、新三、新四、新七、壬、癸、新五、新六；bggpu-audit-fix-code-r1 的甲、乙、丙与重复键只放弃这一批。
3. 修补带出的新状态与交错：连着开不成的计数（开成清零、只记切换开卡）与进程级「没回来」那张表、叫停、卸载、切到 cpu 再切回之间的交错——计数会不会漏清或多清、挂载那一刻的开卡算不算进去、同一进程里先后两次挂载之间表里留下的项会不会让后一次挂载永远开不成那张卡；开卡期间出错停着时再来一次切换（盖掉在开的那一组）、实现先答开不成再来切换、答换上了而核心层那一格已被盖掉，各走到哪；重复键放弃的那一批与同一批里别的段已交回的结果、连续不确定的计数、意图的删除各是什么结局，有没有一种交错让重复键被反复触发而任务既不停也不前进。
4. 未定项清单的完整性：两份判决「交用户的」与五份实现员报告「停下交主 agent 的设计问题」里列的，加上主 agent 自己定的那几条（开成清零；重复键照样发一次只删意图的发布；输入长度另立类型不与显存字节同型；`EntryLargerThanAChunkOnTheDevice` 的字段改名），有没有漏掉该交用户而没交的；条款现行文本与代码逐句对一遍，有没有代码做了而条款没写、或条款写了而代码没做的。
5. 报出的数字能不能核：第二节「验证到的数」逐条，腿从任务日志（`task-server/task log <号>`，只读）与变异表里核得出的核，核不出的点名；合入后验证只跑「补丁改到的测试目标与变异行点名的测试目标」，这两批改到的产品文件里有没有哪一支只被没跑到的测试目标罩着。

## 四、检查单

### 检查单：走不到的理由二次查实

五份实现员报告「走不到的理由」一节原样：

- `~/.local/share/singlefs/work/bggpu-c996-batch4/result.md`：「这一次在产品代码里没有新加 `todo!`、`unreachable!`、`assert!` 或条款没定的错误成员。补丁对段树是整份，批次 3 列过的那一处断言照新行号再列一次：- crates/singlefs-background-gpu/src/background_gpu.rs:3148 `batch_not_split_over_the_gpus` 的「参与的卡一张都不剩只有两种」断言走不到。参与的卡为空，又没有开不成、量代价出错、因显存上限不用这三样的，只有告知一张卡都不参与（`use_no_gpu_from_the_next_batch`，核心层交 `cpu` 那一组时调）之后、下一组换上之前这一段。这期间核心层 `BackgroundComputeOfThisMount::in_effect` 是 `cpu`，只在实现答 `PutInEffect` 那一刻（`take_the_answer_on_the_settings_handed_in`）换成 `gpu` 那一组，答 `NotOpened` 不换。调用点：`latest_settings_handed_in` 在交来的一组还在开卡时先答 `StillOpeningItsCards`，开不成时先答 `NotOpened`，换上之后参与的卡由 `put_the_settings_in_effect` 重排；`plans_of_a_batch` 一路只在核心层走 GPU 时调；`failure_under_the_settings_handed_in` 只由 e202 在挂载那一刻读，那时挂载那一组参与的卡不为空，或有因上限不用的卡。」（行号是那一批合入时的，类型那一批之后移了，腿现查；批次 3 那一份另列了一处新加的 `debug_assert!`：`gpus_opened_at_a_switch` 里「累计试满的卡不交到切换开卡线程」。）
- `~/.local/share/singlefs/work/bggpu-types-batch5/result.md`：「没有要列的：这一批在产品代码 `crates/<包>/src/` 里没新加 `todo!`、`unreachable!`、`assert!`，也没新加条款没定的错误成员。新加的是三个 `const fn` 里超出 `u64` 时的 `panic!`（`GpuMemoryInBytes::plus`、`CopiesOnTheGpu::bytes_of_the_copies`、`EntryCountOfAChunk::bytes_of_the_entries`），都在文档的 `# Panics` 写明、各有一条 `#[should_panic]` 用例钉着；它们顶替的是原来裸 `u64` 的 `*` / `+`（仓根 `Cargo.toml` 的 `[profile.release]` 开着 `overflow-checks = true`，原来溢出同样 panic，现查过）。」
- `result-2.md`：「没有要列的：这一次在产品代码 `crates/<包>/src/` 里没新加 `todo!`、`unreachable!`、`assert!`，也没新加条款没定的错误成员。新加的是 `const fn` 里的 `panic!`：`InputLengthOnTheHostInBytes::plus`（和超出 `u64`）、`InputLengthOnTheHostInBytes::minus`（扣得比剩下的多）、`EntryCountOfAChunk::input_bytes_of_the_entries`（乘出来超出 `u64`），各在文档 `# Panics` 写明、各有一条 `#[should_panic]` 用例；它们顶替的是原来裸 `u64` 的 `+`、`-`、`*`（仓根 `[profile.release]` 开着 `overflow-checks = true`，前一份报告现查过），原来溢出同样 panic。`of_this_many_bytes`、`of_this_many_records`、`of_this_many_segments` 里的 `expect` 顶替原来各处的 `u64::try_from(…).expect(…)`。」

腿照调用点逐个追：那处断言「调不到」成不成立；那处 `debug_assert!` 在 release 里不判，前提被破坏时会发生什么；每个新 `panic!` 顶替的原式在 debug 与 release 两种构建里是不是同样 panic（`minus` 顶替的减法原来在哪几处、原来下溢是不是 panic）、产品路径上喂得到它的输入有没有被上游挡住。

### 检查单：类型与名字

新类型六个（第二节列的）与新原因成员五个（起不了开卡线程、开卡 panic、前一次没回来、叫停、试满不再试，名字在 `GpuFailureCause` 里现查）、`SameEntryKeyTwiceInOneBatch` 一族、`CostNotMeasuredAtTheSwitch`、`OpeningsNotReturnedInThisProcess`、`FailedOpeningsOfEachGpuInThisMount`。腿判：新类型的内值是 `pub`（`pub struct AllocationRecordCountOfATable(pub u64)` 这类），混用靠不靠得住；实现员报告「留着的裸整数」各条（`result-2.md` 十条）留得对不对，有没有两个语义不同的量仍并排是裸整数；标识符 `OpeningsInThisMountUsedUp`、`failed_openings` 说的是「一次挂载里用完」而计数是连着的、开成清零，名字与语义对不对得上；有没有 `_ =>`；新 `expect` 写没写清依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支。腿现跑 `grep -nE 'legacy|old_format|migrat|兼容|deprecated' research/prompts/_c996-code-r3-diff.md`，并核换类型时有没有留着收裸整数的旧入口当转接。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在产品代码 `src/` 下。腿核 diff 里有没有写文件、开库的新路径。

### 检查单：所有任务多机可用

产品代码没有这一类。进程级那张表是进程内的状态，不落盘。腿核新测试（五份报告列的）有没有读本机状态、写死地址；真卡对拍在没有显卡的机器上是不是报「本次未跑」。

### 检查单：测试优先多线程

新用例一条一个场景。腿核涉及开卡线程、等间隔的用例有没有写死 sleep 的等法、有没有该拆而没拆的参数循环；进程级那张表在同一个测试二进制里并行的用例之间会不会互相看见（用例之间共用进程级状态算不算读了别的用例留下的状态）。

### 检查单：带 --item 的命令与项

本轮没有新的崩溃点重放项。变异行：第一批点名 51 行（任务 15341）加 31 行（任务 15382），类型那一批 55 行（任务 15450）。腿核这些名字与 `crates/mutations.tsv` 逐字相同、新加的分支还有没有没被变异行罩着的（c996-code-r2 判决改法 9 点名的九处补全了没有）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没加崩溃流。腿判重复键放弃一批时那一次只删意图的发布、开卡期间出错停下时的落盘时机，有没有已有崩溃流罩不到的新形态。

## 五、分工

- 攻方腿（`three-way-attack`）：第三节五条攻击面，重点第 1、2、3 条；把前两轮的模型用例在今天的代码上重跑的命令写进报告「要主 agent 跑的命令」，另可自己写小模型。报告 `research/prompts/c996-code-r3-opus-output.md`，模型目录 `research/prompts/c996-code-r3-opus-model/`。
- 辩方腿（`three-way-defense`）：复核两份已有判决——判它们打中的每一格够不够得着（是不是同样打中所有替代方案）、判为站住的格站不站得住、两份判决的改法在今天的代码上是不是照写了；第三节第 4、5 条与第四节八格。报告 `research/prompts/c996-code-r3-sonnet-output.md`。

## 六、格

1. 第三节五条攻击面逐条回；前两轮打中的每一格回「修掉了」「还中」或「推不出」。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
