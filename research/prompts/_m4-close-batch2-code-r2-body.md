# m4-close-batch2-code-r2：里程碑四收口第二批，代码轮第二轮正文（2026-11-10）

<!-- doc-lint:not-numbers R -->

## 一、题面

代码轮第一轮判决 `research/prompts/m4-close-batch2-code-r1-main-verification.md` 打中三处（A1 后台入口停着不删意图、A2 删意图那一次的落点受 R 排他、A3 落后支两处经裸读者调），用户 2026-11-10 定了 A1、A2 的改法，修补已合入主工作区。这一轮是同一题的第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。另把 `crates/singlefs-core/src/mounted_read.rs` 相对 HEAD 的全部改动并进来攻（早先几批合入，代码轮 m4-close-fixes-code r1、r2 正文提到它、点名一节没列）。

改动全文：`research/prompts/_m4-close-batch2-code-r2-diff.md`（第一节是修补的产品代码，五份；第二节是 `mounted_read.rs` 相对 HEAD）。实现员报告：`~/.local/share/singlefs/work/m4-batch2-r1-fixes-writer/result.md`、`~/.local/share/singlefs/work/m4-batch2-r1-fixes-writer-2/result.md`（承接，合成补丁）。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `allocator.rs`、`compaction_plan.rs`、`mount.rs`、`mounted_read.rs`、`mounted_session.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-batch2-code-r2-snapshot/`）。这几份里不是 diff 里的块当上下文读、不攻。`mount.rs` 里另有别的会话刚打进的三份补丁（读取段、实例表读取段、影子账拒因字段），不在这一轮的 diff 里。
- 用户 2026-11-10 定案：A1 后台入口判出停着、现行那一版有活意图时起跑一趟只删意图，之后照旧不写；那一趟吃发布预算，预算扣完就等下一次用户改动之后那一次调用再删。A2 只删意图、不搬单元的那一次（`CompactionIntentStep::Drop` 与不带一批的 `DeleteOnceTheRegionIsEmptyByTheRecords`）取落点不排除 R，撤分配器侧意图的时机不变；带一批的最后一批照旧排除 R。A3 两处落后支改经计数层的读者。
- 验证到的数（量的）：落后支测试目标任务 12093 `13 passed`；停着删意图测试目标任务 12094 `22 passed`；两个整理崩溃目标里不枚举的用例任务 12096 `3 passed`、`2 passed`；clippy 两包全部目标任务 12095 退 0；证红任务 12132「点名 15 条：现判 15 条…抓到 15 条」；变异行静态核任务 12098 退 0；命名检查四格（仓根）任务 12168 退 0；崩溃用例登记核任务 12170 退 0；拒算登记任务 12169「拒算 12 条、登记表 12 行」。崩溃点重放 4 项缩到展开深度 4（任务 12167）：删意图两项绿（`compaction-paused-short-of-space-dropping-the-intent` 判了 2 条崩溃枚举用例，含新加的删意图落在 R 里那一条），`compaction-relocation`、`lifecycle-closure` 两项 `red=0`、只因缩域 `exhaustive=false` 记红；全量留给提交时的整轮门禁。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 1、2、13；`.claude/kb/decisions/04-校验和位置.md` D4（校验和位置） 已定项 4；`.claude/kb/decisions/28-挂载期承诺量.md` D28（挂载期承诺量） 已定项 1；I-5.6（`crates/singlefs-checker/invariants/07-分配器.md`）。用户 2026-11-10 的定案还没写回 kb，腿以定案为准。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. A2 的修补本身：删意图那一次放开 R、发布返回后把排他放回——发布途中、落盘前、落盘后各一刻崩溃，恢复后 I-5.6 与续做成不成立；放开 R 那一窗口里有没有别的分配（同一次发布里的别的单元、另发那一批、前台）能落进 R；不带一批的删意图那一核不把自己落在 R 里的新落点算成 R 不空，会不会因此放过 R 里真有别的记录的那一形。
2. A1 的修补本身：后台入口新触发 `LiveIntentWhileStoppedAfterAComputeError` 与预算、节奏、让路的次序；攒批会话里先排空待发（一次用户改动）再删意图那一形（实现员报告「要你定」第 7 条，主 agent 照起跑通则收）；切到一张卡都不剩那一刻有活意图时怎么判。
3. A3 的修补本身：`caller_inputs_agreeing_with_the_disk` 与它的五个调用方、推环、读文件入口改经计数层之后，账并进分配器与拒了随错丢掉是否都对；新取样点用例（回退目标取根环里没有的 txg）是不是只靠那一核记账。
4. 从未被看过的：`mounted_read.rs` 相对 HEAD 的全部改动。

## 四、检查单

### 检查单：走不到的理由二次查实

两份实现员报告「走不到的理由」都写「无」。主 agent 另列：A2 放开 R 之后「发布返回后把排他放回」那一步，若发布出错返回，排他是否一样放回；腿逐处追调用点判。

### 检查单：类型与名字

新加：后台入口触发 `LiveIntentWhileStoppedAfterAComputeError`；`compaction_plan.rs` 两个 `pub(crate)` 方法（R 排他的放开与放回）；新崩溃流 `CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion`；验证包新用例名（腿看 diff）。`expect` 依赖的不变量由腿逐处看 diff。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r2-diff.md` 由腿现跑。

### 检查单：不新建判定库

本轮改动没有这一类：diff 全在 `crates/singlefs-core/src/`，产品代码不写判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；新测试读不读本机状态由腿看实现员报告「拒算的测试」（都写无）与测试源码，拒算登记任务 12169 没有未登记。

### 检查单：测试优先多线程

新用例与新崩溃流的线程取法由腿核（`crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs`、`crash_enumeration_compaction_paused_short_of_space_dropping_the_intent_stream.rs`、`writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs`、`common_compaction_prefix/mod.rs`）。

### 检查单：带 --item 的命令与项

新崩溃枚举用例 `crash-case:compaction-paused-with-only-the-region-free-dropping-the-intent-in-the-region-stream` 登记在 `.claude/gate.d/stage-inputs.tsv` 第 162 行，挂在合表 `.claude/gate.d/crash-point-replay-items.tsv` 第 78 行的项 `compaction-paused-short-of-space-dropping-the-intent` 下；任务 12167 点这一项跑到了 2 条崩溃枚举用例。

### 检查单：新崩溃流多罩了什么

新流比暂停删意图流多罩「第一批之后隔离 R 之外的空槽、暂停删意图那一次的新落点落在 R 里」及其途中每个崩溃点；原暂停流不改（改了会让「出错停下流与暂停流逐步相同」那条用例红）。腿核这一句与新流的终点是否够（流到删意图落盘就结束，之后的重新签发不在里面）。

## 五、格

1. 代码做的是不是条款与用户 2026-11-10 定案说的（第二节那三条）。
2. 第三节四条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
