# m4-exit5-relocation-proof-r1：出口 5 的证明罩到 core 搬迁执行（2026-10-26）

<!-- doc-lint:not-numbers G7 N6 P4 -->

## 一、题面

里程碑四出口 5（`.claude/kb/milestone/04-fourth-txn.md` 出口表第 5 行）：「「GC 在抽象文件系统状态上是恒等变换」在里程碑三七·1 选定的工具上证出；带 GC 的模型对拍与不跑 GC 逐项相同」。七·1 选的工具是 Verus。用户 2026-10-26 定这一出口「要罩到 core 搬迁执行」（`records/2026-10-13-里程碑四预开发.md` 里那一天「② 出口 5「要罩到 core 搬迁执行」」那一行）。这推翻了判决 `research/prompts/m4-gc-r1-main-verification.md` G7 行原文「执行层靠 N6、模型对拍与必出反例的世界」，所以开这一轮。

这一轮要判的是：「抽象文件系统状态」取哪个定义；「罩到 core 搬迁执行」罩到哪一层；用哪一种办法（或哪几种叠起来）把 `publish_relocation_batch` 这一路接进 Verus 证明；G7 那一句改成什么；各候选的核内占比与义务数，以及各自罩不到的清单。

被判的条款：出口 5 原文；判决 G7；`.claude/kb/decisions/13-验证路线.md` 已定项 1（M1–M4 与射程）、5、13、14；`.claude/rules/verification.md`「定义与名字」checker 档那一行里「Verus 验 `crates/singlefs-replay-prefix`」那一句与「门禁的结构」里 checker-tier-formal-verification 按条那一句。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

背景事实来自调查员报告 `~/.local/share/singlefs/work/m4-close-exit5-exit10/result.md`「一　出口 5」一节（闭包数是调查员在仓副本上用 `unit-graph closure` 量的，命令在该节 1.6），下面几条主 agent 现查过：

- 现有证明：`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`（两个可执行核 `choose_compaction_segment`、`plan_compaction_batch` 与 P4 抽象恒等引理，抽象状态是映射 key→落点加落点字节），`crates/singlefs-replay-prefix/src/gc_crash_proofs.rs`（七步协议模型、任意有限历史归纳、崩溃幂等、旧读者保护，产品只调 `released_record_can_be_reclaimed`）。
- core 搬迁执行入口：`crates/singlefs-core/src/transaction.rs:9680` `publish_relocation_batch`、`:10142` `relocation_checked_against_the_version`；`crates/singlefs-core/src/mounted_session.rs` 的 `run_compaction_on`。
- 核内占比（量的，调查员）：`publish_relocation_batch` 闭包 834 个 fn、18027 行，在核里 2 个 fn、12 行；`run_compaction_on` 356 / 20517 行；对照 `recovery::replay_journal` 3171 / 4616 行。
- 判法：`scripts/gate/verus-proof-check.py` 今天只认一行 crate（多于一行判红），禁 `external_body`、`assume_specification`、`admit`。
- 产品批大小 `CompactionBatchSize::NotDecidedByTheClauses`（`crates/singlefs-core/src/compaction_plan.rs`），产品路径签意图之前就停，K 只能当参数。
- 记录写明执行层不在证明里：`crates/singlefs-replay-prefix/src/README.md`「……完整组合精化仍未完成」那一句；`compaction_plan_proofs.rs` 文件头「设备屏障、字节解码与实际发布路径到该模型的精化仍是证明边界」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### X1　「抽象文件系统状态」取哪一个
候选：P4 的「每个 key 经映射读到的字节」；`gc_crash_proofs` 的 `logical`（所有视图同一个逻辑版本）；O1 模型的 POSIX 可见内容。格里要问：三个定义下「恒等变换」各自说的是什么、哪一个对得上出口 5 后半句「带 GC 的模型对拍与不跑 GC 逐项相同」、哪一个能在 Verus 里与 core 的数据结构接上。

### X2　「罩到 core 搬迁执行」罩到哪一层
候选：(a) core 里做判定的纯函数；(b) core 的可执行代码本身；(c) core 写出的写流与盘面（崩溃恢复之后读到的东西）。格里要问：用户「要罩到 core 搬迁执行」的最弱读法与最强读法各是哪一层；按 D13 已定项 1 射程「refinement 的边际价值只在崩溃点重放枚举取不到的那几维」，证明要罩的是哪几维（批大小、搬几批、槽数、盘数）。

### X3　办法与叠法
候选（调查员报告 1.4 那张表，名字是描述性的）：核外提（把搬迁执行里能写成纯函数的判定搬进 `singlefs-replay-prefix`、由 core 调，四样：对上一版核、新一版映射、目的地约束、一次发布的写序列及其持久前缀对应协议历史）；core 直验（Verus 直接验 `singlefs-core`，下界约 834 条义务、判法要改）；执行模型对拍（Verus 里另写一份执行模型证它精化协议，再拿 core 跑同一批输入经池级 checker 投影逐项比）；前提运行时核（把 `gc_crash_proofs` 两条前提写成判器，崩溃点重放每个态上核）。格里要问：哪一种或哪几种叠起来满足 X2 定的那一层；每种在副本上的核内占比与义务数（照 1.6 的命令量，要写出原型的写原型）；各自罩不到的清单；与用户 2026-10-01「Verus 跑真实代码」、D13 已定项 5（checker 不许调 core）、13（同源性计数）、14（哈希只能当未解释函数）冲不冲。

### X4　G7 那一句与条款怎么改
格里要问：G7「执行层靠 N6、模型对拍与必出反例的世界」改成什么；D13 要不要为 GC 恒等性质新立一项或补进已定项 1；`.claude/rules/verification.md` 两句要不要改；批大小 K 未定、`gc_crash_proofs` 自己列的未连接前提（floor 不超过受保护读者与根的版本、多槽 span、双盘副本、隔离、共享快照、不同逻辑版本）这一次各罩不罩。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | X1–X4 | 从出口 5 原文、用户定案、D13 各分项与今天的两份证明推：X1、X2 的推荐与理由；X3 每个候选在副本上量核内占比与义务数（能写原型的写一个最小原型、在副本里直接起 `verus` 读 verified 数），给推荐叠法；X4 给改写句；每条结论写文件:行号 |
| 云端攻方（Opus） | X1–X4 | 攻推荐之外与推荐本身：某个候选声称罩到的那一层，在 core 里找一条它罩不到、却会让抽象状态改变的路径（例：目的地与活读者重叠、写序列某个持久前缀对不上协议历史）；某个抽象状态定义下「恒等」成立而模型对拍不同；义务数与核内占比在副本上重量核不上。每个打中给可复现的命令与原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；冻结副本路径在派发提示里给。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；拷冻结副本，不在冻结副本里改；每棵树各用一个编译目录，跑完删。直接起的 `verus` 与 `cargo verus` 可以在副本里跑；门禁 checker-tier-formal-verification 与 `scripts/gate/verus-proof-check.py` 不起。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo 或 verus；线程上限照派发提示。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
