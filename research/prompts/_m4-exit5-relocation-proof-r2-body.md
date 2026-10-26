# m4-exit5-relocation-proof-r2：出口 5 照「core 直验，连胶水一起证」的做法与代价（2026-10-26）

<!-- doc-lint:not-numbers X1 X2 X3 X4 Y1 Y2 Y3 Y4 Y5 Y6 G7 N6 P4 -->

## 一、题面

同一题的第二轮。第一轮判决 `research/prompts/m4-exit5-relocation-proof-r1-main-verification.md`：用户 2026-10-26 定出口 5 罩到的层是「core 直验，连胶水一起证」；第一轮攻方打中的三处挂起：判决 `research/prompts/m4-gc-r1-main-verification.md` G7「整理不改用户看得见的东西」在带回退与抬 F 的历史上不成立；`gc_crash_proofs` 协议里没有持久的回退目标视图；P4 的目的地前提只要求起点槽号互异。射程连回退路径（`crates/singlefs-core/src/mount.rs` 的回退）一起算。

这一轮照 `.claude/rules/three-way-inference.md`「三轮分工」第二轮：第一轮的修补本身、产生结论的方法、从未被看过的地方。这一轮要判的是：Verus 直接验 `singlefs-core` 搬迁执行与回退那两路的可执行代码，怎么接、射程到哪、判法与门禁怎么改、分几阶段、代价多少、罩不到什么；以及第一轮挂起的三处在这一形下怎么收。

被判的条款与材料：里程碑四出口 5；判决 G7；`.claude/kb/decisions/13-验证路线.md` 已定项 1、5、13、14；`.claude/rules/verification.md`「定义与名字」checker 档那一行里「Verus 验 `crates/singlefs-replay-prefix`」与「门禁的结构」里 checker-tier-formal-verification 按条那一句；第一轮两份腿报告 `research/prompts/m4-exit5-relocation-proof-r1-sonnet-output.md`、`research/prompts/m4-exit5-relocation-proof-r1-opus-output.md` 与它们的模型目录（正推的原型核 `src/relocation_execution_proofs.rs`，攻方的 Verus 反例 `verus/attack_gc_protocol_and_p4.rs`）。

## 二、实现今天的样子（主 agent 的观测，2026-10-26）

- 冻结副本：`~/.local/share/singlefs/work/m4-close-rounds-frozen/`（第一轮同一份；GPU 垃圾回收会话第 42 批之后的 `mount.rs`、`mounted_session.rs`、`compaction_plan.rs`、`background_compute.rs` 在主工作区有新改动，引这几份写主工作区的行号并注明）。
- 现有证明：`crates/singlefs-replay-prefix/` 下的核（`compaction_plan_proofs.rs`、`gc_crash_proofs.rs` 等），由 core 调其中几个可执行核；门禁 checker-tier-formal-verification 的判法 `scripts/gate/verus-proof-check.py` 只认一行 crate（多于一行判红），禁 `external_body`、`assume_specification`、`admit`（作弊标记）。
- 第一轮量的数（腿量的，复跑见第一轮判决第五节）：`publish_relocation_batch` 闭包 836 个 fn、18057 行，核内 12 行；正推原型 88 → 102 verified；正推探针 `Option::copied` 要 `assume_specification`。
- core 的依赖：`crates/singlefs-core/Cargo.toml`（今天不依赖 vstd）；core 用到的 std 集合（`BTreeMap`、`Vec`、`Option` 的组合子、闭包 `FnOnce`、trait `BlockDevice` 的泛型）以代码为准，腿现查。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### Y1　怎么接
格里要问：core 加 vstd 依赖、`verus!` 宏包住被验的函数，还是用 Verus 的注解跑在不改编译产物的那一种模式（例如只在 `cfg(verus_keep_ghost)` 下展开的规格）；对产品构建（不跑 Verus 的 `cargo build`）有什么影响；每一种在副本上能不能编过 core 今天的一个小函数（给命令与原样输出）。

### Y2　std 规格与「作弊标记」
`Option::copied` 这一类 std 函数要规格。格里要问：vstd 自带的规格罩到 core 用到的 std 集合的多少（现数）；罩不到的那几个，用 `assume_specification` 写规格算不算作弊——判法今天把它当作弊标记一概禁，改成「只许对 std 的已知函数、规格逐条登记、每条配一个必红世界」行不行，那一条门禁怎么判得出「写的规格与 std 的行为不符」。

### Y3　射程与分阶段
格里要问：发布闭包（`publish_relocation_batch`）与回退闭包（`mount.rs` 的回退那一路）里哪些函数单元必须进证明才算「罩到」，哪些可以当未解释（设备读写、校验和——D13 已定项 14）；按什么次序分阶段（先哪几个函数、每阶段的验收是什么），每阶段的义务数怎么现数（照第一轮 1.6 的闭包量法与直接起 verus 读 verified 数）；推一个总代价并写明是推的。

### Y4　判法与门禁
格里要问：`verus-proof-check.py` 只认一行 crate、`.claude/gate.d/verus-pins.tsv` 的登记数、必红世界每个带 ensures 的函数一行、空真筛查每处 requires，这几条在 core 进来之后怎么改；`.claude/rules/verification.md` 那两句怎么改；单元键与复用（D13 已定项 17）对 core 里被验的函数怎么算。

### Y5　第一轮挂起的三处
格里要问：G7 改写成什么（恒等性质的主语连「回退能退回的版本集合」与 F 一起说）；`gc_crash_proofs` 协议补持久回退目标视图与多逻辑版本之后，第一轮攻方那个 Verus 反例（回收覆写源槽、旧版映射读坏）是否被排除；P4 目的地前提改成按区间不重叠之后，那个两槽罩一槽的反例是否被排除。给出改后的协议与引理在副本上的 verified 数。

### Y6　罩不到的
格里要问：在 Y1–Y5 的形下，哪些仍罩不到（设备屏障与 FUA、字节编解码、校验和无碰撞、双盘副本、隔离、共享快照……），各由什么罩（崩溃点重放全量、N6、模型对拍），写成清单。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Y1–Y6 | 在副本上实测 Y1 的接法、Y2 的规格覆盖、Y3 第一阶段的一个最小原型（照第一轮正推原型往 core 里接），给分阶段计划与代价、Y4 的判法改法、Y5 的改后协议与 verified 数；每条结论写文件:行号 |
| 云端攻方（Opus） | Y1–Y6 | 攻正推会给的这一形：某种接法让产品构建变了或 Verus 与 rustc 看到的不是同一份代码；某条 `assume_specification` 写的规格与 std 实际行为不符而门禁判不出；分阶段里某一阶段的验收在 core 里有一条绕过它而改变抽象状态的路径；改后协议仍放过第一轮那两个反例的变体。不许重复第一轮已攻过的角度（第一轮判决第二节逐格写着）。每个打中给可复现的命令与原样输出 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本或主工作区里的路径与行号，写明是哪一份。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；拷冻结副本，不在冻结副本里改；每棵树各用一个编译目录，跑完删。直接起的 `verus` 与 `cargo verus` 可以在副本里跑；门禁 checker-tier-formal-verification 与 `scripts/gate/verus-proof-check.py` 不起。
- 每条编译与 Verus 命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo 或 verus；线程上限照派发提示。本机内存紧时经 `scripts/hosts/multi-host-run.sh --peer` 在第二台跑。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 模型与原型放 `research/prompts/m4-exit5-relocation-proof-r2-<模型>-model/`，带 `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh`）。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
