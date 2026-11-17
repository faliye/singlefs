# m4-close-batch2-code-r1：里程碑四收口第二批合入，代码轮第一轮正文（2026-11-10）

<!-- doc-lint:not-numbers P2a K-k R -->

## 一、题面

代码轮 m4-close-fixes-code 跑满 r1–r3（判决 `research/prompts/m4-close-fixes-code-r3-main-verification.md`），之后这一批又合入八份补丁，被攻过零轮，这是新一题的第一轮。照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-close-fixes-code-r1-diff.md` 之后的这一批，产品代码部分在 `research/prompts/_m4-close-batch2-code-r1-diff.md`（5 份文件）。八份补丁与各自的实现员报告：

| 补丁 | 报告 | 改什么 |
|---|---|---|
| r3 收尾 | `~/.local/share/singlefs/work/m4-close-fixes-r3-writer/result.md`、`result-compile-and-rest.md` | r3 判决第三节：读接全、变异行、注释写实、坏盘 × 部分落盘接可写挂载 |
| 回退门用例 | `~/.local/share/singlefs/work/m4-close-r3-gate-read-writer/result.md` | 只改测试 |
| 主动修复两红 | `~/.local/share/singlefs/work/m4-close-r3-unit-read-writer/result.md` | `mount.rs` 取号之前取树表 0 条那一版的单元清单经计数层读 |
| 故障注入战役 | `~/.local/share/singlefs/work/m4-close-r3-fault-campaign-writer/result.md` | 只改验证包：执行器与模型认改动之后另发的那一批 |
| 同实例点名窗口放行 | `~/.local/share/singlefs/work/m4-one-copy-behind-writer/result.md` | `mount.rs` 落后支缺单元只算 (tail, 所选] 窗口里点名的单元；窗口之外缺的按读到一份坏处理（判决 `research/prompts/m4-one-copy-behind-refusal-r1-main-verification.md`，用户 2026-11-09 定 `c1-same`） |
| 整理暂停与出错停下删意图 | `~/.local/share/singlefs/work/m4-compaction-pause-deletes-intent-writer-2/result.md` | 用户 2026-11-10 定：整理暂停删意图、一批出错停下也删意图、删意图那一次归删除类不判式子 |
| 窗口用例取样点 | `~/.local/share/singlefs/work/m4-window-instance-sample-writer/result.md` | 只改测试 |
| 出错停下两红 | `~/.local/share/singlefs/work/m4-stop-drop-intent-reds-writer/result.md` | 只改测试 |

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 被判的产品代码：`crates/singlefs-core/src/` 的 `allocator.rs`、`compaction_plan.rs`、`mount.rs`、`mounted_session.rs`、`transaction.rs`（开工快照 `research/prompts/m4-close-batch2-code-r1-snapshot/`）。这几份里不是这一批的块（别的会话的改动）当上下文读、不攻；腿只攻 diff 里有的块。
- 验证到的数（量的）：合入后验证任务 11954 的测试段：两个包按条判绿 26 + 2566 条、红 2 条，红的两条随后由「出错停下两红」修补改了用例（任务 11981 `18 passed`，证红任务 11982 两条都抓到）；fmt、clippy、build 都过。放行的三个 GPU 目标任务 11763：`DISKS_AFTER_THE_UNMOUNT states=153 refused=0 judged_on_this_host=153`、同实例分叉 30 种取值。崩溃枚举用例全量没跑（95 条判定块因核心层改动几乎全作废，四小时内跑不完），缩到展开深度 4 的 11 项任务 11992 在跑；全量留给提交时的整轮门禁。
- 压着的条款：`.claude/kb/decisions/04-校验和位置.md` D4（校验和位置） 已定项 4；`.claude/kb/decisions/18-块里携带什么信息.md` D18（块里携带什么信息） 已定项 11；`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 1、2、13；`.claude/kb/decisions/28-挂载期承诺量.md` D28（挂载期承诺量） 已定项 1；`.claude/kb/decisions/21-权威态与派生态的分界.md` D21（权威态与派生态的分界） 已定项 10；I-5.6、I-1.11（`crates/singlefs-checker/invariants/`）。用户 2026-11-10 三条定案还没写回 D26、D28、D23（journal 的角色与格式） 第 405 行，腿以定案为准。
- 方案按 `crates/` 今天的实现来谈。

## 三、攻击面

1. 整理暂停与出错停下删意图：删意图之前、途中、之后崩溃，恢复后 I-5.6 与续做是否都成立；删意图那一次「归删除类不判式子」的判法（`deletes_only_the_compaction_intent`）有没有放过别的发布；一趟开头停着时删与不删的分界（见检查单第一格）。
2. 同实例点名窗口放行：窗口取法（tail 与所选同实例、记录只按计数器认）在跨实例、记录槽自己坏、两块盘都落后时判得对不对；窗口之外缺的进计数与搬迁路径之后，卸载与下一次挂载对不对。
3. r3 收尾：读接全那几处（`transaction.rs` 写入口轮换那一读、`mount.rs` 一族）是不是都接上了、读别的版本的只计数不搬。
4. 三处改动之间的交叉：删意图那一次发布与可写挂载落后支、与读到只坏一份另发的那一批，是否会在同一次挂载里互相踩。

## 四、检查单

### 检查单：走不到的理由二次查实

各份实现员报告「走不到的理由」都写「无」（产品代码没有新加 `todo!`、`unreachable!`、`assert!` 与条款没定的错误成员）。主 agent 另列一处：`crates/singlefs-core/src/mounted_session.rs` 一趟开头判出停着时 `budget == 0` 那一支（同文件 `compaction_stays_stopped` 之后、`drop_the_live_intent_on_a_stop` 之前那一判）：预算为 0 时不删意图、意图与 R 排他照留。实现员推它走不到（后台入口判出停着就不起跑；显式入口预算 `.max(1)`；攒批会话排空发成用户改动后预算归零重算），主 agent 判它留作根环预算护栏、不删。腿逐处追调用点判「走不到」成不成立，并判留着它与用户定案「停下都删意图」冲不冲突。

### 检查单：类型与名字

新加：`drop_the_live_intent_on_a_stop`、`deletes_only_the_compaction_intent`、`CompactionIntentStep::Drop`、步种 `CompactionIntentDroppedOnAStop`、入口 `publish_compaction_intent_drop_on_a_stop`、计数 `compaction_intent_dropped_on_a_stop`；`mount.rs` 窗口那几个函数（腿看 diff）；r3 收尾里计数层读者的改名。删掉：`pause_the_live_intent`、`resume_the_live_intent`、`IntentDropOnThePauseRefusedShortOfSpaceUndecided`。`expect` 依赖的不变量由腿逐处看 diff。

### 检查单：不兼容旧代码与旧格式

主 agent 没看到认旧版本的分支；`grep -nE 'legacy|old_format|migrat' research/prompts/_m4-close-batch2-code-r1-diff.md` 由腿现跑。

### 检查单：不新建判定库

本轮改动没有这一类：diff 全在 `crates/singlefs-core/src/`，产品代码不写判定。

### 检查单：所有任务多机可用

本轮改动的产品代码没有这一类；新测试读不读本机状态由腿看各份实现员报告「拒算的测试」（都写无）与测试源码。

### 检查单：测试优先多线程

同实例分叉扫描、坏盘 × 部分落盘枚举在用例里按线程切片（`writable_mount_of_a_device_behind_the_version_judges_the_units_named_after_its_tail.rs`、`crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs`），由腿核线程数取法。

### 检查单：带 --item 的命令与项

新崩溃流 `compaction-paused-short-of-space-dropping-the-intent` 已登记 `.claude/gate.d/stage-inputs.tsv` 与 `.claude/gate.d/crash-point-replay-items.tsv`；整理一族 9 项与带读坏的 2 项的 `--item` 命令在任务 11992。腿核新项能被 `--list-items` 列出。

### 检查单：新崩溃流多罩了什么

整理暂停删意图流：比整理搬迁流多罩「最后一批被准入拒、暂停删意图、重新签发」这一段；出错停下删意图那一条录制流与它逐步相同，只加小用例钉住相同（`the_flow_stopped_after_a_compute_error_writes_exactly_what_the_paused_flow_writes`）。腿核这两句。

## 五、格

1. 代码做的是不是条款与用户定案说的：D4 已定项 4、D18 已定项 11（`c1-same`）、D26 已定项 1/2/13 与用户 2026-11-10 三条定案、D28 已定项 1 的不判名单。
2. 第三节四条攻击面。
3. 第四节八格逐格回「成立」「不成立」或「推不出」，推不出的算打中。
