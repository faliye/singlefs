# m4-close-fixes-code-r1：里程碑四收口，代码轮 r3 判决之后那两批修补，代码轮第一轮正文（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

代码轮 m4-close-code 跑满 r1–r3 三轮（判决 `research/prompts/m4-close-code-r3-main-verification.md`），r3 判决第三节交实现员的八条与之后的五条修补打进主工作区之后，被攻过零轮。用户 2026-11-08 定「再开一轮攻修补」，这是新一题的第一轮，只攻这两批修补，不重攻 r1–r3 已判过的改动。

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-close-fixes-code-r1-diff.md`（第一批补丁里产品代码的块原样抄录，6 个文件块）。第二批只改验证包（`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer-2/patch/crates.patch`），不走产品代码那一轮，读来当上下文。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `compaction_plan.rs`、`mount.rs`、`mounted_read.rs`、`mounted_session.rs`、`recovery.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-fixes-code-r1-snapshot/crates-sha256.txt`）。这几份里不是这一批的块（别的会话的改动）当上下文读、不攻；腿只攻 diff 里有的块。
- 实现员报告：`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer/result.md`（八条，「结论一览」与「停下交主 agent」）；`~/.local/share/singlefs/work/m4-close-code-r3-fix-writer-2/result.md`（五条）。
- 验证到的数（量的）：内核钉值任务 9510 `test result: ok. 1 passed`；变异 6822、6823、6824、6921 任务 9518「计数：抓到 4 条、没红 0 条、无效 0 条」；变异 6922 任务 9632 抓到 1 条；GPU 对拍四条缩域到 17 任务 9535 `test result: ok. 4 passed`；`data_unit_read_with_one_copy_failing_is_counted` 目标任务 8981 30 条过。全量域 GPU 对拍与变异 6923、6924 没跑过。
- 压着的条款：
  - `.claude/kb/decisions/04-校验和位置.md` D4（校验和位置） 已定项 4（第 71、73、75 行含用户两条读法）。
  - `.claude/kb/checks-owed.md` C946（只坏一份的读没有计数也没有搬迁修复）（第 589 行）。
  - `crates/singlefs-checker/invariants/` 的 I-9.4、I-9.12。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. r3 判决第五节写明「T3、T4 两格「没打中」只抽一次，不拿它支撑结论；下一批改完之后的代码轮里照攻」：这一轮照攻。
2. `crates/singlefs-core/src/transaction.rs` 第 11909 行一带 `hints_are_the_mapping_position` 那一判：`MappingLookup::NodeMalformedOrNoEntryWithThisKey`、`EntryNarrowerThanItsFieldTable` 记 false 走得到走不到，走到时计数少一次对不对。
3. D4（校验和位置） 已定项 4「可写挂载期间别的读」那一句的写法与实现对不对得上。
4. 第一批第 7 条「下一次用户改动发成之后搬」：只试一次、结局报在名字只说快照的成员里、三类豁免认不出映射 key。
5. 第一批第 5 条 `read_at` 报错也并进累计、第 6 条一次改动同一把 key 只计一次，两种计数口径有没有互相打架。

## 四、检查单

### 检查单：走不到的理由二次查实

两份实现员报告都没有「走不到的理由」一节（`grep -c 走不到的理由` 两份各 0）。第一批报告「改法逐处」里写成走不到的分支由腿逐处追；另加攻击面第 2 条那一处。

### 检查单：类型与名字

新加：`recovery::inode_records_of_the_leaf_ascend_after_the_leaves_before` 多的分隔 key 参数；`recovery::PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`；`PoolReaderCountingUnitReadsWithOneCopyFailing` 的去重；`OldDataUnitReadsBackWithOneCopyFailingInThisChange::keep_the_mapping_keys_not_read_back_before_in_this_change`；`read_the_data_units_of_the_range`。`expect` 依赖的不变量由腿逐处看 diff。第一批报告「停下交主 agent」第 1 条已知 `UserChangePublished::relocation_of_the_old_data_units_kept_for_the_snapshots` 名字只说了快照那一半。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r1-diff.md` 由腿现跑。

### 检查单：不新建判定库

本轮改动没有这一类：diff 全在 `crates/singlefs-core/src/`，产品代码不写判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；第二批测试读不读本机状态由腿看 `crates/singlefs-verification/tests/data_unit_read_with_one_copy_failing_is_counted.rs` 与 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。

### 检查单：测试优先多线程

第二批 GPU 对拍按显存额度排队、块那一层按块判（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 第 3876–3918 行一带）；`data_unit_read_with_one_copy_failing_is_counted` 各条单池单场景。

### 检查单：带 --item 的命令与项

两份报告的「要主 agent 跑的命令」都带 `SINGLEFS_RUN_SCOPE`；crates-mutation-replay 的项是第 6822–6824、6921–6924 行，6923、6924 在改成按块切分（实现员在做）。

### 检查单：新崩溃流多罩了什么

第二批加两条 GPU 判退坏叶流（打开那一样不判叶序与分隔 key、建上一版那一样不判）：比已有流多罩「并集只剩字节丢了，GPU 退到 V」这一形，缩域到 17 时 `states_staying_on_r=9`（任务 9535）。其余三条流多罩的由腿核。

## 五、格

1. 代码做的是不是条款说的：D4（校验和位置） 已定项 4 与 C946（只坏一份的读没有计数也没有搬迁修复）。
2. 第三节五条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
