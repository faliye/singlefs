# m4-gpu-wait-outside-the-lock-code-r3：后台整理一批等 GPU 拆到会话锁外、前台等锁让路，代码轮第三轮正文（2026-11-20）

## 一、题面

攻代码轮第二轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-main-verification.md`「三、改法」的修补（第 1、2、3、5、6 条；第 4 条改的是 kb，一并核对得上），这一轮是最后一轮。

- 条款：`.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 4、5、6 条。第 5 条在第二轮判决之后补了一句（用户弹窗定「不跨重挂」）：回锁里出错那一支要删的意图被门压住的，留到门开，之后不论走哪条路，取下一批的输入之前都先照这道门与破例判它；「待删」只记在这次挂载里，卸载重挂之后照盘上那条活意图续做。第 6 条清零那一句照用户 2026-11-19 定的原话改回「取了输入而没让路的那一批也把两个计数清零」。
- 修补的补丁：合入后验证合成的那一份 `~/.local/share/singlefs/work/merge-and-verify/2026-11-20-4031890/merge-patch/`（`crates.patch` 改 8 份文件，`mutations-append.tsv` 9 行、`mutations-replacements.tsv` 1 行）；三个实现员先后在同一个副本上改，报告是同一份 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-fix/result.md`。
- 修补在今天的树里落在哪（主 agent 现查的行号）：
  - 改法第 1 条（被门压住的删意图）：会话上记的那一条 `crates/singlefs-core/src/compaction_plan.rs:615`、`:622` `IntentOfAnAbandonedBatchStillToDrop`，记下、取、忘三个方法在同一份文件第 825–856 行；回锁出错那一支记下它在 `crates/singlefs-core/src/mounted_session/compaction_batch.rs:696` 一带；取下一批输入之前先判、先删在同一份文件第 23–51 行 `take_the_input_of_one_compaction_batch` 与第 1491 行 `stop_of_the_abandoned_batch_whose_intent_is_still_to_drop`；回退门与破例在 `crates/singlefs-core/src/mounted_session/compaction_run.rs:505-512`。`take_the_input_of_one_compaction_batch` 的调用点：`crates/singlefs-core/src/mounted_session/compaction_publish.rs:234`、`crates/singlefs-core/src/mounted_session/compaction_run.rs:728`。
  - 改法第 2 条（回锁那一刻判破例）：`crates/singlefs-core/src/mounted_session/compaction_batch.rs:798`，判法 `crates/singlefs-core/src/mounted_session.rs:521` `foreground_cannot_write_under_the_switches`。
  - 改法第 3 条（停着时开头删意图发成之后清让路计数）：`crates/singlefs-core/src/mounted_session/compaction_run.rs` 停着那一支。
  - 改法第 5 条的只供测试用的开关：`crates/singlefs-core/src/allocator.rs:258` `note_a_cluster_segment_opened_in_this_mount_for_tests`。
  - 新用例：`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/` 下 `compute_errors_and_the_rollback_gate_closed_in_the_window.rs`、`issues_at_the_relock.rs`、`yields_to_a_foreground_entry_waiting_for_the_session_lock.rs`；变异行名单在开工快照目录。
- 合入后验证量到的（主 agent 的观测）：第 1 步点 2 项「共 54 条测试」全绿，fmt、clippy、build 过；第 2 步 crates-mutation-replay 点这一批 10 行「抓到 10 条、没红 0 条、无效 0 条、锚点失效 0 行」「基线不绿 0 行」，命名检查与拒算清单都过。
- 方案按 `crates/` 今天的实现来谈；读码引主工作区今天的路径与行号。

## 二、要攻的地方

1. 改法第 1 条：「待删」那一条记下、判、删、忘四步。记下之后哪条路不经 `take_the_input_of_one_compaction_batch` 就能接着在这条意图下搬（锁里一口气推完的入口、可写挂载续做、删除续做、停着时一趟开头那一支）；删成、冻结、门关、破例、现行那一版里的活意图已换成别的几种情形下忘得对不对；删它那一次用放弃那一批时的停因，这一趟停与不停、预算与让路计数记得对不对。
2. 改法第 2 条：回锁那一刻照 `foreground_cannot_write_under_the_switches` 照发，与第 5 条「破例的条件……成立时，这一次也照发」逐字对得上吗；破例照发那一次与回退门之后的那几判（冻结、写带宽）次序。
3. 改法第 3 条与第 6 条清零口径合起来：「一批 = 一次整理发布」之后，还有哪一处整理发布发成之后不清。
4. 改法第 5 条：只供测试用的开关在产品路径上走不到；新用例测的是条款说的量（空洞数、要搬数取现行那一版，资格的两个合取项），还是这一版实现碰巧的计数；改坏对应那一判用例会不会照样绿。
5. kb 两处改动（第 5 条补的那一句、第 6 条去掉「下限到了、」）与代码今天做的逐句对得上吗。

第一、二轮已攻过而没打中的角度不重复：锁外与维护线程发布交错、CPU 路径卸载等整批、锁里旧入口仍是 pub、取输入先排空后问让路、回锁有冻结就丢批、GPU 包建缓冲那一等读停标记、fsync 试锁失败到登记与放锁到减计数两个窗口、注释口径与不切片理由。

## 三、检查单

### 检查单：走不到的理由二次查实

三个实现员的报告「走不到的理由」一节都写「没新加 `todo!`、`unreachable!`、`assert!` 或条款没定的错误成员」。腿核这句在今天的树里成立，修补碰的几处函数里有没有新加的走不到的断言与 `expect`。

### 检查单：类型与名字

新名字：`IntentOfAnAbandonedBatchStillToDrop`、`stop_of_the_abandoned_batch_whose_intent_is_still_to_drop`、`forget_the_intent_of_an_abandoned_batch_still_to_drop`、`has_an_intent_of_an_abandoned_batch_still_to_drop`、`note_a_cluster_segment_opened_in_this_mount_for_tests`。腿核：同一个量不同语义有没有共用裸整数、有没有缩写与 `_ =>`、`expect` 写没写清依赖哪条不变量；新用例里的 `other =>` 已改成 `let … else`。

### 检查单：不兼容旧代码与旧格式

修补不改盘上格式、不加版本号与迁移路径；「待删」只住内存（用户定不跨重挂）。腿核有没有为旧路径留的分支。

### 检查单：不新建判定库

修补只加单元与集成测试，不碰崩溃枚举用例与判定存放。腿核。

### 检查单：所有任务多机可用

新用例不读本机独有状态、不写死主机名与地址。腿核。

### 检查单：测试优先多线程

新用例各自一个池、一个场景，住在同一个测试目标里由 libtest 并行跑。腿核该并行而没并行的。

### 检查单：带 --item 的命令与项

这一批 10 行变异点名的测试都写带模块前缀的全名，合入后验证第 2 步按这 10 行点名判过（「基线不绿 0 行」）。腿核报告里的命令带不带范围声明、点没点全。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：修补没有新加崩溃点重放流或崩溃枚举用例，查过的范围是合成补丁的 `crates.patch` 与追加、替换的变异行。

### 检查单：放的位置

新用例住在验证包 `crates/singlefs-verification/tests/` 下被测那个目标的子模块里；只供测试用的开关在被测的产品包 `crates/singlefs-core/src/allocator.rs`，照仓里已有的 `_for_tests` 写法。没有新开包、没有跨包 `#[path]` 与 `include!`。腿核。

## 四、分工

| 腿 | 立场 |
|---|---|
| 云端正推（Sonnet） | 逐条核修补做的是不是第二轮判决「三、改法」与 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 5、6 条今天的原文说的；九格逐格答成立 / 不成立 / 推不出 |
| 云端攻方（Opus） | 攻第二节五条：造修补之后才写得出、与条款不符的历史；测试改回去会不会照样绿；由用户决定的动作放开扫，只固定前缀与故障 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查。
- 引代码写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；读码推出来、没量过的写「推的，没量过」。候选变异点名的测试写带模块前缀的全名。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
