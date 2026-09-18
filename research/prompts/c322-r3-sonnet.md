你是 singlefs 三方论证第三轮的**辩方腿**。立场：复核第二轮判决第 3 条的四条排除——世代号 G1（全池最大 + 1）、「每盘择到的那一份」读法、乙′（盘 0 超级块 → 屏障 → 盘 1 超级块 → 屏障）、第一轮甲的 I-7.7 改写——替每一条认真辩一次（判据 T5）；之后正推候选包 P 在 T6 上的数。辩护要实打实：找得出第二轮读错、推错、漏看的地方就报，找不出就如实写「辩不动」，不许为辩而辩。

## 先读

1. 背景材料：`research/prompts/_c322-r3-background.md`（正文 + 小节清单 + 附录）。
2. 前两轮：`research/prompts/c322-r2-main-verification.md`（第二轮判决在第四节）、`research/prompts/c322-r2-opus-output.md`（五那一节是世代号与读法的数）、`research/prompts/c322-r2-sonnet-output.md`、`research/prompts/c322-r1-main-verification.md`。
3. 自己去查（引用时写命令与行号）：`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 16、`.claude/kb/decisions/18-块里携带什么信息.md` 第 879 行、`.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 与 16、`.claude/kb/layout/01-first-txn.md` 八；源码 `crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`。

## 要做的

- T5 逐条：每条排除写出「第二轮的理由」「替它辩的最强论证」「现查之后站不站得住」。副本模型上的数（第二轮攻方腿的 `k-model.out`）只能当线索；要推翻一条排除，给条文、代码行或能在仓里重做的构造。
- T6：数出 P 改不改第一个事务的字节（首次挂载路径上按 G2 写出的世代号是不是仍为 2、3、4、5；段序列登记表变不变）、层 0 里 checker 判 I-7.7 违例的状态数从 2 变成几、要改几处代码与条款（列文件与行）。
- 若你发现材料里某条前提与仓里原文对不上，先报那一条，再继续。

## 交付

- **分段写**：报告写进 `research/prompts/c322-r3-sonnet-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就报错停下），后面用 `>>` 追加。
- 引条款**整行抄**；现查的写明命令与行号；数字带口径。
- 不许用「本条」「上文」这类指代，每一格自足。不许改 kb、不许改仓里任何已有文件、不许 git 的任何写操作。最后回复只写一句指向报告。
