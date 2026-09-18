你是里程碑「第二个事务」空池可写挂载与「非空」按树表认的代码三方对抗**第一轮**（轮名 m2-emptypool-nonempty-r1）的**云端正推腿**。你的立场是核「代码做的是不是条款说的」：三处改法 V1（「非空」按树表里 inode 树与 extent 树的根指针与前一条有效根比，树表读不出就拒绝抬 F）、V2（只做过 mkfs 的池可写挂载）、V3（三处拒绝都在落盘之前）与它们压着的条款逐格对；另核写回。

## 先读

1. 背景材料 `research/prompts/_m2-emptypool-nonempty-r1-background.md`（正文 V1–V3、Z1–Z7 判据、分工、禁读清单；diff；附录条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查，不从背景材料里数）。
2. 实现员报告 `research/prompts/m2-emptypool-nonempty-r1-implementer-report.md`。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`、`recovery.rs`、`transaction.rs`，用例 `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_three_formatted_pool_layer0.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_step_four_rollback.rs`。
4. 草稿放你自己的目录 `/tmp/claude-1000/m2-emptypool-nonempty-r1-sonnet/`；要跑东西把仓拷到那里（`rsync -a --exclude target --exclude .git`，副本里 `nice -n 19`，不跑 release 全量枚举），副本上的数注明是副本。在副本上把 `crates/mutations.tsv` 第 62–71 行十条变异各跑一遍（改坏 → 点名的用例红、还原 → 绿），贴原样结果。

## 要核的格

- **Z5 代码与条款**：D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」的射程罩不罩得住写行那一次（它是新实例的第一次发布，按 D16 已定项 8 与 D18（块里携带什么信息） 已定项 11 要写行，行为空时写不写实例表）——逐字读已定项 9 的原文与 D18 已定项 11，写出代码做的与条款字面的对应；D16 已定项 1 表格后「非空」那段 ⚠️ 与代码的「前一条有效根」「按 (txg, 实例) 排」「最旧那条与没有两棵树比」「树表读不出就拒绝」是不是同一件事，哪一句是实现员自己补的；`FileVersionWithoutAnyJournalRecord`、`InstanceRowsOnVersionWithoutFileUnsupported`、`RollbackToVersionWithoutFileUnsupported`、`VersionWithoutFileNotWrittenByMakeFilesystem` 四个拒绝各自有没有条款，没有的写「没有条款」并写 grep 命令与零命中。
- **Z7 写回**：`.claude/kb/milestone/02-second-txn.md` 步 3、步 5 现状里写空池挂载与「非空」的那两句，`.claude/kb/layout/01-first-txn.md` 八「只做过 mkfs 的池的可写挂载」一行，每一句能核的都对着代码核。

## 规矩

- 不替攻方找新反例（那是攻方腿的）；只判代码与条款对不对得上。
- 每一格指到代码行号与附录原文；说「一致」也要写是哪一句对上哪一句。
- 禁读：`research/prompts/m2-emptypool-nonempty-r1-*-output*.md` 里别的腿的输出、`m2-emptypool-nonempty-r1-opus.md`、`-local-*.md`。

## 交付

- **分段写**：报告写进 `research/prompts/m2-emptypool-nonempty-r1-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 报告开头一张「各格一览」（Z5 每一小问一行、Z7 一行：一致 / 不一致 / 够不着），然后按格分节，末尾「这条腿自己的限度」。
- 不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告。最后回复只写一句指向报告。
