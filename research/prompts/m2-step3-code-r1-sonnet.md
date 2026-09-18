你是里程碑「第二个事务」步 0 / 步 3 那批代码三方对抗**第一轮**（轮名 m2-step3-code-r1）的**云端正推腿**。你的立场是核「代码做的是不是条款说的」：不找反例（那是攻方腿的），把每一条选择与它压着的条款原文并排放，逐句判「一致 / 不一致 / 条款没说」。

## 先读

1. 背景材料 `research/prompts/_m2-step3-code-r1-background.md`：正文（S1–S11 选择表、X1–X11 判据）、小节清单、附录一（条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查）。
2. 附录二 `research/prompts/_m2-step3-code-r1-diff.md`（从提交 5f9e449 起 `crates/` 的全部 diff、两个新文件全文、`crates/mutations.tsv`）。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`transaction.rs`、`allocator.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`，以及 E142 装置 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 里恢复的那一段。
4. 草稿放你自己的目录 `/tmp/claude-1000/m2-step3-code-r1-sonnet/`。要跑东西，把仓拷到那里（`rsync -a --exclude target --exclude .git`），副本上的数注明是副本。

## 分到的格

- **S1–S11 逐条**：每条一格，三列——代码怎么做（路径 + 行号）、条款原文怎么说（附录整行）、判定（一致 / 不一致 / 条款没说 ⇒ 这是实做时定下的取法，要标预想交用户）。「条款没说」的格单独列一张表。
- **X9 装置同步**：`e142_first_transaction_dry_run.rs` 的恢复与 `crates/singlefs-core/src/recovery.rs` 的 `replay_journal`，两条规则（候选只收同实例、链首接所选根自己那条）是不是同一条——逐分支并排，找出任何一处两边处理不同的输入（例：所选根自己那条记录读不出、候选里有别的实例的记录、jsn 断号在第一条）。
- **X10 写回**：`.claude/kb/milestone/02-second-txn.md` 步 0 / 步 3 的「现状」段、`.claude/kb/layout/01-first-txn.md` 八的「第一次之后的可写挂载（写行）」「空发布（暖机，后续可写挂载的实例…）」两行与「第二条流」那句、`.claude/kb/invariants.md` I-3.8 的状态列——每一句能核的都对着代码或用例核，列出不符的句子（逐字引原句）。

## 规矩

- 不判攻方腿的格（X1–X8、X11 不写打中 / 没打中）。
- 每一格都要指到代码行号与附录原文；说「一致」也要写是哪一句对上哪一句。
- 禁读：`research/prompts/m2-step3-code-r1-*-output*.md` 里别的腿的输出。前两轮判决可读：`research/prompts/m2-code-r1-main-verification.md`、`m2-code-r2-main-verification.md`。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step3-code-r1-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 报告开头一张「各格判定一览」（S1–S11、X9、X10 各一行），然后按格分节，末尾「条款没说的取法」一张表与「这条腿自己的限度」。
- 不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告。最后回复只写一句指向报告。
