# m4-gpu-wait-outside-the-lock-code-r1：后台整理一批等 GPU 拆到会话锁外、前台等锁让路，代码轮第一轮正文（2026-11-19）

## 一、题面

攻这一整批合进主工作区的 `crates/` 改动：后台整理一批拆成锁里取输入、锁外算、回锁发布三段，前台入口等会话锁时后台让路，GPU 包里等卡那一处在调用方停的时候放弃那一等，加它们的测试、崩溃枚举用例与变异行。

- 条款：设计三轮的判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md`、`research/prompts/m4-gpu-wait-outside-the-lock-design-r2-main-verification.md`、`research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md`（第三轮「三、改法」十四条是这一批的实现规格）；写回的决策条款是 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 5、6 条与 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md` 已定项 2 的射程。用户另定的两句（原话）：让路下限「时间和次数都要考虑」（N 次或 T 毫秒，先到哪个算哪个）；判定「测试分两种，有GPU和没有GPU。没有GPU的话 CPU也是两段。无论怎么样都是两端。」
- 这一批合进主工作区的完整改动（主工作区合入之前那一刻 → 合入之后）：`~/.local/share/singlefs/work/merge-and-verify/2026-11-19-1029961/merge-patch/crates.patch`（25 份文件），加 `crates/mutations.tsv` 的追加与换行（同目录 `mutations-append.tsv`、`mutations-replacements.tsv`）；合入之后另一份补丁 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-mutation-rows-fix/patch/`（`crates/mutations.tsv` 换第 5750 行替换文、删一行三样全同的重复行）。八份实现员补丁各自的报告在 `merge-patch/result.md`（拼在一起）。
- 合入后验证量到的（主 agent 的观测）：第 1 步 28 个测试目标验证红 0 项、fmt 与 clippy 过；第 2 步 crates-mutation-replay 点 126 行抓到 125、无效 1（已改）、基线不绿 0；22 行点全量崩溃枚举用例的变异在缩域（展开深度 4）下证红 22 行抓到 22 行，全域留给提交时（用户定）；登记的 26 条崩溃枚举用例缩到深度 5 判绿、块全复用；`cargo build --offline -p gpu-verification-server` 退 0。
- 方案按 `crates/` 今天的实现来谈；读码引主工作区今天的路径与行号。

## 二、要攻的地方

1. 三段的拆法与条款对不对得上：锁里取输入（`crates/singlefs-core/src/mounted_session.rs` 的 `take_the_input_of_a_background_compaction_batch` 一族）、锁外算（`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/region_scan.rs`）、回锁发布（`publish_the_batch_computed_out_of_the_lock`）；回锁照现行那一版重排计划、剔掉记录变了的单元、重取隔离集，签发的量取现行那一版的，有冻结的发布就丢这一批。
2. 让路：`crates/singlefs-mount-runtime/src/lib.rs` 里前台入口 `try_lock` 失败才登记、后台取输入之前问一次、N 次或 T 毫秒下限、一批发成清零；锁外那一段期间维护线程自己的 `publish_when_due` 与重发冻结照跑。
3. 停：GPU 包 `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs` 等那一处读停标记，`GaveUpWaitingForTheCallerIsStopping`；卸载与 Drop 不等。
4. 测试：`crates/singlefs-mount-runtime/tests/` 新加的两份、`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs`、新崩溃流 `crates/singlefs-verification/tests/crash_enumeration_compaction_batch_computed_before_a_foreground_change_and_pruned_at_the_relock_stream.rs`、九条整理崩溃流各补的 CPU 两段版；它们测的是不是条款说的，变异行改回去会不会红。

## 三、检查单

### 检查单：走不到的理由二次查实

实现员报告列的两处，行号是主工作区今天的：
- `crates/singlefs-core/src/compaction_plan.rs:1190` `hand_a_batch_out_of_the_lock` 的断言「至多一批在锁外」：只有 `MountedSession::take_the_background_batch_that_starts` 一处调它，那一处只经 `MountedSessionWithPendingChanges::take_the_input_of_a_background_compaction_batch` 走到，而那个入口第一判就是 `has_a_batch_out_of_the_lock()` 为真即拒；判过之后到调它之间持着同一个 `&mut self`。
- `crates/singlefs-core/src/mounted_session.rs:9311` `publish_the_batch_computed_out_of_the_lock` 里的 `unreachable!`：票只在至多一次发布的那一种趟里、每批之前那几判放行之后交出（放行时这一趟发布数是 0），回锁发成一次之后发布数是 1，再走 `judge_before_the_next_compaction_batch` 时「至多一次发布」那一判（或更早的预算、至多一批两判）必然停下，交回 `Err`。
没有新加条款没定的错误成员；`GaveUpWaitingForTheCallerIsStopping` 是条款定了的「卸载时不等还在跑的提交」那一支（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:23`）。

### 检查单：类型与名字

新类型与成员：锁外那一批的票（`BatchOutOfTheLock`）、让路下限 `CompactionYieldFloor`、让路的问答与停因成员、`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`、测试里的 `JudgeOfTheEnumeration`。腿核：同一个量不同语义有没有共用裸整数、有没有缩写、`_ =>`、`expect` 写没写清依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

这一批不改盘上格式、不加版本号与迁移路径；腿核改动里有没有为旧路径留的分支（例：三段之外还留着整批锁内等的旧入口）。

### 检查单：不新建判定库

崩溃枚举用例的 CPU 两段版照先例经 `crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of` 写一份只开 CPU 的临时配置、照抄 prune-server 的地址，判定写进那一份库；腿核测试与装置有没有另开判定存放处。

### 检查单：所有任务多机可用

新测试不读本机独有状态、不写死主机名；CPU 两段版的临时配置在测试自己的临时目录里由调用方配置现生成。腿核。

### 检查单：测试优先多线程

崩溃枚举用例走 `enumerate_crash_states_*` 一族、线程数取登记值；运行库那几份测试一个池一条维护线程、注释写明不切片的理由（例：`mount_runtime_computes_the_batch_out_of_the_session_lock.rs` 第 603、668、717 行）。腿核该并行而没并行的。

### 检查单：带 --item 的命令与项

新登记的崩溃枚举用例 `crash-case:compaction-batch-computed-out-of-the-lock-around-*` 与九条整理流的 `*-on-the-cpu-judge` 都在 `.claude/gate.d/stage-inputs.tsv` 与 `.claude/gate.d/crash-point-replay-items.tsv`，合入后验证已按项点到（缩域）。腿核新项能不能被 `--list-items` 列出。

### 检查单：新崩溃流多罩了什么

新流「计划在前台改动之前算好、回锁剔掉再发」比已登记的「活意图期间写类改动」流多罩的是：锁外算那一段期间前台发布一次（覆盖写或删文件），回锁时照现行那一版剔掉记录变了的单元再发的写序列；前缀与那条流逐块相同由用例 `the_in_lock_reference_with_an_overwrite_writes_the_blocks_of_the_registered_flow_with_a_write_while_the_intent_is_live` 钉住。九条整理流的 CPU 两段版与 GPU 版录同一条流（共用 `enumerate_with_both_observers`），块键相同、多罩的是 CPU 判器那一侧。

### 检查单：放的位置

新测试都住在被测那个包自己的 `tests/`（运行库的在 `crates/singlefs-mount-runtime/tests/`，跨包与崩溃枚举的在验证包 `crates/singlefs-verification/tests/`）；没有新开包、没有跨包 `#[path]` 与 `include!`。腿核。

## 四、分工

| 腿 | 立场 |
|---|---|
| 云端正推（Sonnet） | 逐条核代码做的是不是第三轮「三、改法」十四条与 D26 已定项 13 第 5、6 条说的；九格逐格答成立 / 不成立 / 推不出 |
| 云端攻方（Opus） | 假设这一批是错的：造回锁剔不干净、隔离记号漏比、让路饿死或互等、停标记漏读、锁外与维护线程发布交错写出今天写不出的序列的历史；测试测的是不是实现而不是条款、变异改回去会不会照样绿；由用户决定的动作放开扫 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查。
- 引代码写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
