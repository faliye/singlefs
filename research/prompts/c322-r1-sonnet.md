你是 singlefs 三方论证的**正推腿**。立场：从前提 P1–P8 与跑前写死的判据 J1–J6 出发，推出取号那一步该取甲（只补条款、不加屏障）、乙（加屏障、两盘串行）还是丙（参照臂：号只认一次原子写）。

## 先读

1. 背景材料：`research/prompts/_c322-r1-background.md`（正文 + 小节清单 + 附录，附录是 kb 条款的整段抄录）。
2. 自己去查（引用时写命令与行号）：
   - `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 16 那一行（与它指向的「正文」到底在不在）
   - `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 16
   - `.claude/kb/decisions/16-发布语义.md` 已定项 7 / 已定项 8
   - `.claude/kb/invariants.md` I-7.7 那一行
   - `.claude/kb/checks-owed.md` C322 那一行
   - 源码：`crates/singlefs-core/src/transaction.rs`（`acquire_instance`、`warm_up`）、`crates/singlefs-core/src/recovery.rs`（`choose_superblock`）、`crates/singlefs-checker/src/walk.rs`（I-7.7 那两处判定）、`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`

## 要做的

- J1–J6 逐条：给数、给出处、判哪条臂过。
- J1 要真构造：按切段规则逐个写出「各盘实例代号不等」的崩溃状态（持久写集合）；C322 原句那个状态可不可达，给推理。
- J4 要真判：已定写法下两盘的世代号是不是恒同步；若是，C322 那条自证与原式等价，给一个换过的、会红的自证形态。
- 若你发现材料里某条前提与仓里原文对不上，先报那一条，再继续。

## 交付

- **分段写**：报告写进 `research/prompts/c322-r1-sonnet-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（文件已存在就报错停下，不许覆盖），后面追加。
- 引条款**整行抄**；现查的写明命令与行号；数字带口径。
- 不许用「本条」「上文」这类指代，每一格自足。不许改 kb、不许改仓里任何文件。最后回复只写一句指向报告。
