# 崩溃放量写待判行之前先问 GPU 服务、只排计划那一行与观察者盘切段：代码轮第二轮

日期：2026-11-18。

## 范围

第一轮判决 `research/prompts/amplification-without-cards-code-r1-main-verification.md`「三、改法」第 1 条（服务没接核对那一族之前的那一半）、第 2、3、5 条与第 6 条前一句，2026-11-18 合进主工作区的修补。条款是 `.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」「所有任务都要多机可用，不许绑定某一台」，做法与出口在 `records/2026-11-13-全量提交门禁前收尾调度.md` 写着「批 95」「批 96」「批 97」的各行。

压着的决策分项：D13（验证路线） 已定项 15（一个验证包、一个门禁、两种跑法）与 D13（验证路线） 已定项 17（判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成），正文在 `.claude/kb/decisions/13-验证路线.md`。

被判的文件：

- `crates/singlefs-verification/src/crash_amplification.rs`
- `crates/singlefs-verification/src/crash_point_replay_provenance.rs`
- `crates/singlefs-verification/src/crash_state_enumeration.rs`（这一轮没改；判的是它调 `judge_my_blocks_on_gpu` 的那一处在新行为下的结局）
- `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs`
- `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs`
- `crates/mutations.tsv`

第一轮改法里这一轮没做的：第 1 条的根（GPU 服务接核对待判行这一族，在写，没合）、第 4 条（块号、态序号、入池序号立 newtype，等服务那一侧合入之后两张表一起换）、第 6 条后一句（观察者盘那一族交服务之后删惰性取卡那一支）。这三样不在这一轮的射程里。

## 实现今天的样子

下面是主 agent 读 `crates/` 的观测（行号是主工作区此刻的），方案按 `crates/` 今天的实现来谈。

1. 第 ① 段：`ask_the_server_then_judge_my_blocks_through_the_pending_rows`（`crash_amplification.rs:33994`）先把每一块的待判行排好（GPU 判器组键全零、结论版本照判不入库的在这里交错），排出至少一行才问服务收不收镜像判定那一族；不收交回 `WaitingForTheGpuVerificationServerEnded::ServerDoesNotServe { line }`，一行不写。退出只在最外层：`judge_my_blocks_on_gpu`（带 `gpu` 特性那一份在第 34348 行）收到这一支调 `exit_because_the_gpu_verification_server_does_not_serve`（第 34316 行）：把那一行打到标准错误、`std::process::exit(78)`。
2. 第 ② 段：`verify_my_blocks_through_the_facts_verifier_pending_rows`（第 34933 行）在核对键下还没核过的块至少一块时才问服务收不收核对那一族，不收交回 `WaitingForTheFactsVerifierEnded::ServerDoesNotServe { line }`，最外层 `verify_my_blocks_on_gpu`（第 35141 行）同样退 78。
3. 问的那一下：`gpu_verification_server_does_not_serve_line_from_its_control_command`（第 33754 行）起 `python3 <控制命令> serves <族> <版本键 64 个十六进制字符>`；退 0 当收，退 78、别的退出码、被信号杀、起不来都当不收；族是枚举 `PendingRowFamilyOfTheGpuVerificationServer`（第 33670 行，`image-judgement` 与 `facts-verifier`）；那一行由纯函数 `gpu_verification_server_does_not_serve_line`（第 33705 行）拼，带 `family=`、`expected=`、`control=`、`exit_code=`、`said=`。问只在写第一行之前问一次；等的那一圈照旧每隔一段问服务在不在跑，不再问收不收。
4. 只在单测构建里编的一层 `judge_my_blocks_through_the_pending_rows`（第 33941 行，`#[cfg(test)]`）：问的那一下恒答收，再调第 1 条那个函数；`crash_state_enumeration.rs` 的一条单测照九个参数调它。产品路径上崩溃枚举用例那一侧（`crash_state_enumeration.rs:2735`）调的是 `judge_my_blocks_on_gpu`，走第 1 条的问。
5. 只排派活计划那一行：常量 `PLACEMENT_LINE_PREFIX`（第 34687 行）、纯函数 `placement_line_of_the_run`（第 34707 行），七栏逐字是 `judge=gpu pending_blocks= pending_states= states_reused= tables_mebibytes= verifier_pending_blocks= verifier_pending_states=`。后两栏照 `blocks_not_yet_verified_under_the_verifier_row_key`（第 34871 行）数，第 ② 段真跑决定写哪些核对待判行用的是同一个函数。
6. 观察者交回的盘在 CPU 上判：`images_after_the_observers_cut_into_pieces_for_the_cpu`（第 31742 行）从里层份额拿 `checking_threads().min(image_count).max(1)` 条，纯函数 `images_after_the_observers_cut_by_the_threads_taken`（第 31754 行）照拿到的条数切连着的几段，`judge_the_images_after_the_observers_on_the_cpu`（第 31767 行）一段一条线程。
7. 第 ① 段按卡池借草稿区那一支删了，剩 `judge_my_blocks_on_the_cards_started_for_this_run`（第 4957 行）；连带删两条单测与三行变异。
8. 来历表多一行 `placement-facts-verifier`（`crash_point_replay_provenance.rs:723`），是流水线测试里新用例 `a_placement_only_run_counts_every_state_for_the_facts_verifier_when_the_judge_conclusions_are_all_stored` 的流名（`tests/crash_amplification_pipeline_records_checks_and_reuses.rs:1483` 起）。
9. GPU 服务那一侧此刻只登记镜像判定一族：`grep -rn "facts_verifier_pending" gpu-verification-server/src` 零命中。主 agent 量的真环境一趟（任务 19423，崩溃放量入口点名 `fixed-script-through-rollback`、缩域到展开深度 8，缩过域、不算门禁通过）：计划行「CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=32 pending_states=141 states_reused=0 tables_mebibytes=12 verifier_pending_blocks=32 verifier_pending_states=141」，第 ① 段交服务判完 32 块 141 态，第 ② 段打 `CRASH_AMPLIFICATION_GPU_VERIFICATION_SERVER_DOES_NOT_SERVE family=facts-verifier …`、退 78。

## 要腿判的

辩方腿（复核第一轮判决）：第一轮坐实的七条各自够不够得着——每一条主 agent 现查的那几行是不是真撑得起「打中」；记下不当打中的第 8、9、10 条该不该升成打中；「没打中」那一段里只有攻方腿看过的那一条（惰性取卡退出时不留半截行）该不该记成没打中；改法第 1 到 5 条标零轮而不做探针 A/B 的理由站不站得住。再判这一轮的修补是不是真改到了被打中的那一格：第 1 条的现状（写了行空等到被停、库里留行）在修补之后还有没有路走得到。

攻方腿攻修补本身与第一轮写明没看全的地方，下面每一问都要给出代码里走得到的路或写明构造不出：

1. 问与写之间：问到收之后、写第一行之前或写到一半，服务停了、换成另一把键重起了，这一趟走哪一支、库里留不留没人判的行、退出码与末行说的是什么。等的那一圈只问在不在跑、不问收不收，服务在跑而不再收这一把键时会不会回到第一轮第 1 条的空等。
2. 只在有行要写时才问：库里已经有上一趟留下、没人判的同一把流键的行时，这一趟排出的行是空的还是非空的，空的那一支不问就等，等的是谁。
3. 当不收的几种（退 78、别的退出码、被信号杀、起不来、控制命令不认 `serves`）都归成退 78「运行条件不满足」：有没有一种其实是这一侧的缺陷（控制命令路径取错、`python3` 不在）被归成了环境问题而不红；`said=` 取标准输出或标准错误里的一行原样拼进一行日志，里面带换行、等号、空格时那一行还认不认得出。
4. 崩溃枚举用例那一侧：`std::process::exit(78)` 发生在一个 `cargo test` 测试二进制的某一条用例里，同一个二进制里别的用例与已经打出的 `ok` 行各是什么结局，门禁 `crash-point-replay` 读到的是什么，会不会把没跑完的记成过。
5. 只排计划那一行的后两栏：只排计划那一趟发生在录入之前还是之后；事实表还没写进库时「读回的事实表摘要」取到的是什么，后两栏数出来的是全部块、零块还是报错；第 ① 段部分复用、第 ② 段部分核过时两对栏之间该满足的关系（哪一栏不该大于哪一栏）代码保不保证。
6. 切段纯函数：`image_count` 是 0、拿到的条数是 0 或大于盘数、向上取整之后段数少于拿到的条数时，拿了而没用的线程占不占着别人的份额；结论按盘的次序交回这一句在段长不整除时成不成立。
7. 只在单测构建里编的那一层恒答收：崩溃枚举用例那一侧「问到不收」有没有任何测试罩着；那一层会不会让某条本该红的单测照过。
8. 删掉借草稿区那一支之后：带与不带 `gpu` 特性两种编法下，有没有哪条调用还指着被删的函数或字段的语义（比如 `GpuJudgingOfMyBlocks::NotRunTheGpuMemoryWasShort` 在产品路径上还造不造得出来，造不出来的成员是不是该删）。
9. 第一轮写明只抽查过的三处：惰性取卡那一成员取不到卡退出时库里留不留半截行（上一轮只有一条腿看过）；等的那一圈自己读库的开销会不会让任务服务的卡住判法不触发（上一轮只量了控制命令那一半）；问服务的子进程在第二台上起时走的是哪一支（控制命令在第二台上问本机还是两台都问）。

已攻过而没打中的角度不重复：核对待判行四种时序（同一块两个进程同时写、写了之后任务崩、写回与读回之间行被删、流键两边算法不同）；只排计划那一趟的复用数在三种库况下的数法；里层线程份额下限 2 与锁次序会不会互等；「拿着改环境的锁去新占名额就 panic」有没有正常路径撞上；空枚举上的 `match *tables {}`。

### 检查单：走不到的理由二次查实

三份实现员报告「走不到的理由」一节的原话：主体那一份「无：这一批的改动都在验证包 `crates/singlefs-verification/src/` 与 `tests/`，没有在产品代码（`crates/<包>/src/`，验证包、池级 checker、函数单元图除外）里新加 `todo!`、`unreachable!`、`assert!` 或条款没定的错误成员。」；两份修补同样是「无」。同一份报告里另有一处实现员自己判的走不到：借草稿区那一支「`judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from` 只有 `judge_my_blocks_on_gpu_cards` 一个调用方、卡的来路恒为 `GpuCardsSource::DevicesStartedForThisRun`」，照这个理由删了 `TheCardPoolOfThisProcess` 那一臂。腿要核：删之前那一臂是不是真的没有第二个调用方走得到（对着这一轮开工快照之前的版本 `git show HEAD:crates/singlefs-verification/src/crash_amplification.rs` 追）。

### 检查单：类型与名字

新加的名字：`PendingRowFamilyOfTheGpuVerificationServer`、`GPU_VERIFICATION_SERVER_DOES_NOT_SERVE_LINE_PREFIX`、`GPU_VERIFICATION_SERVER_DOES_NOT_SERVE_EXIT_CODE`、`gpu_verification_server_does_not_serve_line`、`gpu_verification_server_does_not_serve_line_from_its_control_command`、`ask_the_server_then_judge_my_blocks_through_the_pending_rows`、`exit_because_the_gpu_verification_server_does_not_serve`、`PlacementOfTheRun`、`placement_line_of_the_run`、`images_after_the_observers_cut_into_pieces_for_the_cpu`、`images_after_the_observers_cut_by_the_threads_taken`、`StatesPlacedOnTheCards`、`judge_my_blocks_on_the_cards_started_for_this_run`，两个结局枚举各多一个成员 `ServerDoesNotServe { line }`。命名检查在打了补丁的快照上过（任务 19407 命名四格、19408）。`PlacementOfTheRun` 的块数是 `usize`、态数是 `u64`，都是裸整数；第一轮第 4 条的 newtype 没做，这一格在那一条上照旧不成立，腿判新加的这几处有没有另外的。

### 检查单：不兼容旧代码与旧格式

`CRASH_AMPLIFICATION_PLACEMENT` 那一行删了 `cards=` 与四个恒为 0 的栏，没留旧写法；读它的分项驱动与入口同一次换上（任务 19415）。没有 `families=` 一栏的旧「在跑」行在服务的控制命令那一侧算一族都不收，那是服务那一侧的事，不在这一轮被判的文件里。只在单测构建里编的那一层 `judge_my_blocks_through_the_pending_rows` 是为不改 `crash_state_enumeration.rs` 的一条单测留的旧签名：算不算「兼容旧代码」，腿判；主 agent 已把删它登在 `records/2026-11-17-subagent账本自上而下清欠.md` 第二百四十六节。

### 检查单：不新建判定库

本轮改动没有新的存放处：问服务收不收读的是控制命令的退出码，不落盘；只排计划那一行只打印。

### 检查单：所有任务多机可用

问的那一下起的是控制命令子进程，控制命令照多机配置取（`gpu_verification_server_control_command`）；它问哪几台是控制命令自己的事（主 agent 量的一次，任务 19369：两台都答、至少一台在跑且登记了这一族这把键退 0）。任务派到第二台时这条子进程在第二台上起，那一支主 agent 没量过，交腿（攻方第 9 问）。新单测用用例自己目录里现造的替身控制命令，不读本机的服务状态。

### 检查单：测试优先多线程

新单测没有按参数循环重建池的；切段那一处改成照拿到的条数起线程。问服务与等服务是单线程，等的是别的进程。五条新用例曾因同一条语句里两次取库而自己等自己（任务 19384），修补拆成两条 `let`；同一条线程重复取库改成当场 panic 那一件延后，登在同一份账本第二百四十七节。

### 检查单：带 --item 的命令与项

这一轮没有新登记的崩溃枚举用例与放量分项。新写与改写的 17 行变异在快照上带 `--item` 判过（任务 19409：「计数：抓到 17 条、没红 0 条、无效 0 条、锚点失效 0 行」）。真环境里带 `--item` 的放量分项跑过一趟（任务 19423，缩过域、不算门禁通过），停在第 ② 段问到不收。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放的流。新用例的流名 `placement-facts-verifier` 录的是已有形态的小流，只为有块可数；它多罩的不是崩溃状态，是一种库况：第 ① 段的结论都在库里、第 ② 段一块都没核过时，那一行的后两栏等于整条流、前两栏是 0。
