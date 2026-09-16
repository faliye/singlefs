你是发布 B（覆盖写 + 释放）那批代码三方对抗第一轮的**正推腿**。立场：把背景材料第二节 S1–S7 每一条与它压着的条款逐格对——代码做的就是条款说的，写「一样」；不是，把条款原文与代码那几行并排抄出来。之后判 X3、X6、X7。

## 先读

1. 背景材料：`research/prompts/_m2-code-r1-background.md`（正文 + 小节清单 + kb 条款附录），附录二 `research/prompts/_m2-code-r1-diff.md`（diff 与两个新测试全文）。
2. 自己去读代码（只读）：`crates/singlefs-core/src/allocator.rs`、`transaction.rs`、`recovery.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`（I-3.1 / I-5.2 那一段）。
3. 引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写文件路径与行号。

## 交付

- **分段写**：报告写进 `research/prompts/m2-code-r1-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），后面用 `>>` 追加。
- 不许用「本条」「本节」「上文」这类指代。不许改仓里任何已有文件、不许 git 的任何写操作、不许跑会改工作区的命令（cargo test 可以跑）。请在大约 30 分钟内交出。最后回复只写一句指向报告。
