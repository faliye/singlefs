# m4-close-batch2-code-r3：里程碑四收口第二批，代码轮第三轮正文（2026-11-10）

<!-- doc-lint:not-numbers R -->

## 一、题面

同一题第三轮，也是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」：第三轮之后不开第四轮）。第二轮判决 `research/prompts/m4-close-batch2-code-r2-main-verification.md` 打中 B2（停着删意图不过回退门）与 H3（停着那一判排在挂载预算读根环之前），改法经 A/B（任务 12435）换得到东西后写进产品代码，另修 H2、补两处用例、改 `mounted_read.rs` 一处注释。第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。这一轮里新冒出的零轮形态不再开轮，写进判决的交用户表。

改动全文：`research/prompts/_m4-close-batch2-code-r3-diff.md`（产品代码两份）。实现员报告：`~/.local/share/singlefs/work/m4-batch2-r2-fixes-writer/result.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的产品代码：`crates/singlefs-core/src/mounted_session.rs`、`crates/singlefs-core/src/mounted_read.rs`（开工快照 `research/prompts/m4-close-batch2-code-r3-snapshot/`）。前两轮已判的块当上下文读；腿只攻这一轮 diff 里的块与它们和前两轮修补的交叉。
- 验证到的数（量的）：证红任务 12476「点名 8 条：现判 8 条…抓到 8 条」；新测试目标 `compaction_stopped_intent_drop_judges_the_rollback_gate` 任务 12477 退 0；`compaction_intent_deletion_without_a_batch_is_refused_while_the_region_keeps_records_of_earlier_versions` 任务 12478 退 0；`compaction_on_the_cpu_while_stopped_after_a_gpu_failure` 任务 12479 退 0；clippy 任务 12480、build 任务 12481 退 0；变异行静态核任务 12482 退 0；拒算登记任务 12483「拒算 12 条、登记表 12 行」；命名四格（仓根）任务 12523 退 0。A/B：任务 12435 `name=m4_close_batch2_r2_ab_verdict without_the_patch_reproduces=yes with_the_patch_holds_at_a_closed_gate=yes with_the_patch_drops_once_the_gate_opens=yes arms_failed=0`。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13 第 4、5 条（2026-11-10 写回后的现行字面）、已定项 1 第 3 条；I-5.6（`crates/singlefs-checker/invariants/07-分配器.md`）。
- H2 实现员取了「解除停下时重判触发」那一条路（排空被拒时停着删意图那一趟不再记成没完），没改名。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 已定项撑不撑得住：D26 已定项 13 第 4、5 条的现行字面与这一版代码逐句对得上吗（停着删意图吃预算、扣完等；门关就不删、意图与 R 排他留到门开）。门一直不开时意图与 R 排他留多久、是否与第 4 条「停下都删意图」的意思冲突。
2. 未定项完整性：H2 取的「解除时重判」与改名那一条路，是否漏掉别的入口（显式入口、可写挂载续做）；B2 改法与前台写不进的破例那一档、攒批会话、放锁入口的交叉（A/B 腿报告「没做的」三项）。
3. 数字能不能核：上面第二节每个任务号与引的行，腿逐个能不能在日志里找到；A/B 两臂读数的口径（H3 那一行 2 对 16）对不对。

## 四、检查单

### 检查单：走不到的理由二次查实

实现员报告「走不到的理由」写「无」；主 agent 没有另列。腿核这一轮 diff 里新加的分支有没有走不到的。

### 检查单：类型与名字

新加的函数与分支名由腿看 diff；H2 那一支没改名，触发名 `LiveIntentWhileStoppedAfterAComputeError` 照旧，腿判名字与做的事现在对不对得上。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r3-diff.md` 由腿现跑。

### 检查单：不新建判定库

本轮改动没有这一类：diff 只在 `crates/singlefs-core/src/`，产品代码不写判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；新测试读不读本机状态由腿看实现员报告「拒算的测试」（写无）与测试源码，拒算登记任务 12483 没有未登记。

### 检查单：测试优先多线程

新测试文件 `compaction_stopped_intent_drop_judges_the_rollback_gate.rs`、`compaction_intent_deletion_without_a_batch_is_refused_while_the_region_keeps_records_of_earlier_versions.rs` 是不是按条拆开、该并行的有没有并行，由腿核。

### 检查单：带 --item 的命令与项

本轮没有新崩溃流；崩溃点重放点名 2 项（`compaction-paused-short-of-space-dropping-the-intent` 与活意图期间写那一项），任务 12525 在跑。crates 变异表点名 9 行，任务 12522 在跑。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：停着删意图、删除队列不空那一形只补了两条小用例（显式入口、后台入口各一），没开崩溃枚举流；腿判这一形要不要崩溃枚举。

## 五、格

1. 第三节三条攻击面。
2. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
