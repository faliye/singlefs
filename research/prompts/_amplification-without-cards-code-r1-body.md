# 崩溃放量任务里不领卡、读库总账按态与测试进程线程份额：代码轮

日期：2026-11-18。

## 范围

`crates/singlefs-verification/` 下 2026-11-17 合进主工作区的三批改动，条款是 `.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」「所有任务都要多机可用，不许绑定某一台」，做法与出口在 `records/2026-11-17-崩溃判器与测试流的效率优化.md` 第九节与 `records/2026-11-13-全量提交门禁前收尾调度.md` 批 67、72、79、82、89 到 93 各行。

压着的决策分项：D13（验证路线） 已定项 15（一个验证包、一个门禁、两种跑法）与 D13（验证路线） 已定项 17（判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成），正文在 `.claude/kb/decisions/13-验证路线.md`。

被判的文件：

- `crates/singlefs-verification/src/crash_amplification.rs`
- `crates/singlefs-verification/src/crash_library.rs`
- `crates/singlefs-verification/src/crash_state_enumeration.rs`
- `crates/singlefs-verification/src/verdict_store.rs`
- `crates/singlefs-verification/src/crash_verify_gpu.rs`
- `crates/singlefs-verification/src/crash_point_replay_provenance.rs`
- `crates/singlefs-verification/src/bin/crash_library_reader.rs`
- `crates/singlefs-verification/tests/crash_amplification_pipeline_records_checks_and_reuses.rs`
- `crates/singlefs-verification/tests/crash_amplification_placement_only_run_takes_no_card.rs`
- `crates/singlefs-verification/tests/crash_amplification_run_of_both_stages_records_the_flow_before_it_leases_a_card.rs`
- `crates/singlefs-verification/tests/crash_amplification_observer_images_judged_on_the_gpu_name_the_card.rs`
- `crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs`
- `crates/mutations.tsv`

删掉的一份：`crates/singlefs-verification/tests/crash_amplification_product_path_books_its_scratch_on_the_card_pool.rs`。

## 实现今天的样子

下面是主 agent 读 `crates/` 的观测（行号是主工作区此刻的），方案按 `crates/` 今天的实现来谈。

1. 崩溃放量一趟（`crash_amplification.rs` 的 `run_from_environment_with_observers`）不在任务进程里取卡，结果里卡的来路恒为 `NoCardNeeded`。只排派活计划那一趟只数这台要判的块与状态，打一行 `CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=… pending_states=… states_reused=… tables_mebibytes=… cards= …`，不借草稿区、不建判器、不试判。
2. 观察者交回的盘上的池级 checker：`pool_checker_of_the_images_after_the_observers`（`crash_amplification.rs:1630`）只收选路；GPU 那一路建惰性成员 `TakingTheGpuCardsAtTheFirstImageToJudge`（同文件第 31289 行），第一张盘要判时才取卡，取不到走 `exit_because_no_card_is_taken_for_the_images_after_the_observers`（第 31354 行）：打一行、退 78。挂了观察者而一张盘都不交回的那一趟不取卡。
3. 第 ② 段核对：`verify_my_blocks_through_the_facts_verifier_pending_rows`（`crash_amplification.rs:34307`）往判定库写核对待判行、约 30 秒问一次 GPU 服务在不在、不在退 78，等服务写回核对行再读回记账。库是一次性库时在写任何一行之前拒（常量 `PENDING_ROWS_IN_A_DISPOSABLE_LIBRARY_ARE_UNSUPPORTED`，第 33348 行）。交给服务的三条公开函数在第 34593、34643、34662 行。
4. `verdict_store.rs` 新加表标签 `0x1F`「核对待判行」（文件头第 109 行、标签第 1805 行），写法对齐待判行那一张：不在才写（`put_the_facts_verifier_pending_row_unless_present`，第 4694 行）、整表扫（`every_facts_verifier_pending_row`，第 4775 行）、带两个守卫的一帧写回。表在 prune-server 那一份库里，不是新的存放处。
5. GPU 服务那一侧还不扫 `0x1F`：`gpu-verification-server/` 里没有读它的代码（`grep -rn "facts_verifier_pending" gpu-verification-server/src` 零命中）。配置开着 GPU 的崩溃放量分项此刻写了核对待判行没有人判。
6. 测试进程里层线程份额：`crash_library.rs:561` 的 `FEWEST_INNER_THREADS_OF_THE_SHARE = 2`，`shares_of_the_test_process`（第 566 行）里层下限取它。改环境的锁与一次性库名额的次序：先取名额、后拿锁，反着拿在新占名额那一刻 panic（常量 `PLACE_TAKEN_WHILE_HOLDING_THE_ENVIRONMENT_LOCK`，第 676 行）。
7. 读库程序 `src/bin/crash_library_reader.rs` 的 `keys` 多打三种行：`CRASH_LIBRARY_STATES`、`CRASH_LIBRARY_STORED_VERSION`、`CRASH_LIBRARY_STORED_KIND`（文件头第 33 到 41 行的样例）。
8. 两种判器写的是同一种结论：`JudgedBy::conclusion_kind` 对 `GpuJudge` 与 `CpuJudge` 都交 `ConclusionKind::ImageJudgement`。流水线测试里只排计划那条用例照这一点改成三趟（开 GPU 只排计划、空库；单机 CPU 判完；再开 GPU 只排计划），不起卡池。

## 要腿判的

正推：代码做的是不是条款说的——任务进程里有没有还在开卡、借显存的路；核对待判行的键、值与守卫够不够服务原子地写回而不重判、不漏判；等服务的那一段在服务不在、服务中途退出、行被别的进程先写时各走哪一支。

反推：哪一格会错——同一块被两个任务进程同时写核对待判行；待判行写了而服务永远不判（第 5 条的现状）时这一趟的退出码与汇总行说的是什么、会不会被当成判过；只排计划那一趟的复用数在库里有别的键的结论时数没数错；里层线程份额下限 2 在份额小于 9 时两份合起来超过份额，会不会让并在一片里跑的几个测试目标互相等死；锁次序那一处 panic 有没有正常路径会撞上。

验证包挂 `gpu` 特性的测试在任务里有 29 条红（测试自己要卡、任务里拿不到卡，或一次性库不收待判行），名单与原话在调度记录第 1321 到 1326 行；这一轮不判它们该怎么修，只判这三批有没有让本来能判的东西变得判不了。

### 检查单：走不到的理由二次查实

六份实现员报告「走不到的理由」一节的原话逐份是：不领卡那一份「无。改的都在验证包 `crates/singlefs-verification/src/` 下，按定义不列；也没加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员。不带 gpu 特性那一份 `mebibytes_of_the_packed_judge_tables` 的 `match *tables {}` 是空枚举上的穷举匹配，不是断言。」；读库总账那一份「无。改动在验证包 `crates/singlefs-verification/src/bin/` 里，不在要列的范围内」；里层份额、两份修补与卡池名单那几份都是「无」。改动全在验证包，没有产品包 `src/` 的行。腿要核的是那一处空枚举上的 `match *tables {}`：不带 `gpu` 特性时这个类型是不是真的一个值都造不出来。

### 检查单：类型与名字

新加的类型与名字：`FactsVerifierPendingRow`、`FactsVerifierPendingStreamKey`、`FactsVerifierRowsForOneFrame`、`FactsVerifierPendingBlockMaterial`、`PLACEMENT_LINE_PREFIX`、`FACTS_VERIFIER_WAITING_LINE_PREFIX`、`CardsTakenAtTheFirstImageToJudge`、`TakingTheGpuCardsAtTheFirstImageToJudge`、`FEWEST_INNER_THREADS_OF_THE_SHARE`。主 agent 量的：这几份 `src/` 相对 HEAD 新加的行里 `_ =>` 零处（`git diff HEAD -- crates/singlefs-verification/src | grep "^+" | grep -c "_ =>"` 打 0）；命名检查在打了补丁的快照上过（任务 19323、19324）。块号、态序号、入池序号是不是各用各的类型、有没有共用裸 `u64`，主 agent 没逐处核，交腿。

### 检查单：不兼容旧代码与旧格式

`CRASH_AMPLIFICATION_PLACEMENT` 那一行去掉了 `host=` 字段、`cards=` 恒空，没有留旧写法；表 `0x1F` 是新加的，没有迁移路径、没有认旧布局的分支；删掉的函数与类型（`verify_my_blocks_on_the_first_card`、`print_the_placement`、`PlacementOfThisRun`、`BufferLeaseOnTheCardPool` 这几样）没有留壳。读 PLACEMENT 那一行的分项驱动 `scripts/crash/crash-amplification-item.sh` 认不认新样子，腿对着脚本核。

### 检查单：不新建判定库

核对待判行是 `verdict_store.rs` 里的一张表，住在 prune-server 那一份库，读写只经 `VerdictStore`；一次性库不收待判行。改动里没有新目录、TSV、sqlite 或另一份 RocksDB。读库程序只读。

### 检查单：所有任务多机可用

任务进程不读本机有没有卡、不照多机配置自己挑卡；等服务那一段问的是判定库里服务的在跑标记，两台同一份库。新测试 `tests/crash_amplification_placement_only_run_takes_no_card.rs` 与改过的流水线用例用用例自己写的多机配置（只写 `ENABLE_GPU=1`），不要本机有卡。主机名与地址：这几份文件里主 agent 没搜到写死的（`grep -n "192\\.168\\|jplife\\|faliye" crates/singlefs-verification/src/crash_amplification.rs crates/singlefs-verification/src/verdict_store.rs` 零命中）。

### 检查单：测试优先多线程

里层并行拿线程经 `InnerThreadsTaken::take`，份额下限 2 条；等服务的那一段是单线程轮询，等的是别的进程，不并行。新测试没有按参数循环重建池的。线程数没有写死的新处。合入后验证里验证包库测试与另外几个测试目标并在一片里跑（份额 6）时两条「一块的各核对线程共用一份枚举表」的单测红过，根子就是份额下限，改后只靠新单测的算术钉着，没在份额 6 下实跑。

### 检查单：带 --item 的命令与项

这三批没有新登记的崩溃枚举用例与放量分项。新写与改写的变异行 44 行在快照上带 `--item` 判过（任务 19325：「计数：抓到 44 条、没红 0 条、无效 0 条、锚点失效 0 行」）。放量分项带 `--item` 的那几条没交：第 5 条的现状下交了判不完。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放的流或用例；新测试用的流是已有的小流（新池新建一个文件、两次写同一扇区），只为有块可数。
