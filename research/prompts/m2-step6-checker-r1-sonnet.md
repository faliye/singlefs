你是里程碑「第二个事务」步 6「checker 新接 I-7.4 与 I-4.8」代码三方对抗**第一轮**（轮名 m2-step6-checker-r1）的**云端正推腿**。你的立场是核「代码做的是不是条款说的」：改法 U1（池级 checker 按回退候选集里每条根各判一格 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），判据是走那条根时 I-2.1（校验和与内容匹配） 的违例数或走读失败数有没有变大；最新根单独判一格 I-4.8；没有更早候选根时 I-7.4 不适用）与 `invariants.md` 两行的定义、判别力句逐格对；另核写回。

## 先读

1. 背景材料 `research/prompts/_m2-step6-checker-r1-background.md`（正文 U1、Y1–Y6 判据、分工、禁读清单；diff；附录一条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查，不从背景材料里数）。
2. 自己去读代码（原仓只读）：`crates/singlefs-checker/src/walk.rs`（`check_pool_image`、`walk_root`、`read_referenced_unit`、`judge_unit_header`）、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/src/crash.rs`（层 0 怎么报每条不变量的评估次数）、用例 `second_transaction_step_five_reuse.rs`、`second_transaction_step_zero_layer0.rs`、`first_transaction_step_seven_layer0.rs`。
3. 草稿放你自己的目录 `/tmp/claude-1000/m2-step6-checker-r1-sonnet/`；要跑东西把仓拷到那里（`rsync -a --exclude target --exclude .git`，副本里 `nice -n 19`），副本上的数注明是副本。在副本上把 `crates/mutations.tsv` 里「步 6」两行各跑一遍（改坏 → 红、还原 → 绿），贴原样结果；再跑一遍 `second_transaction_step_zero_layer0` 的快用例，贴 I-7.4 与 I-4.8 的评估次数（`--nocapture` 看它打印的每条不变量计数）。

## 要核的格

- **Y4 代码与条款**：I-7.4 定义里「重新分配给其他对象」与「清扫抹头」两种坏法，「校验和对不上或头用不了」罩不罩得住每一种——逐种写出对应的镜像形态与 checker 走到哪一行判红；I-4.8 判别力句「抓『本事务内释放的块被重新分配并写入』」在固定脚本上哪一步真被评估过（复用窗口置 0 那条用例、E 那一步）；「近 K 代」按 D16（发布语义） 已定项 1 是回退候选集，代码用的候选集是「按最新根实例表有效 ∧ txg ≥ 最新根自己带的 F」——与条款的 F_生效 差在哪（第二轮判决第六节 2 已列，只核这一格对新两条的影响）；不适用那一格（候选集只剩最新根）条款允不允许。
- **Y6 写回**：`invariants.md` I-7.4 / I-4.8 两行的状态列、文件开头「判 26 条」那句、`verification-build.md` 26 条那句、里程碑 `02-second-txn.md` 步 6 现状，每一句能核的都对着代码核。

## 规矩

- 不替攻方找新反例（那是攻方腿的）；只判代码与条款对不对得上。
- 每一格指到代码行号与附录原文；说「一致」也要写是哪一句对上哪一句。
- 禁读：`research/prompts/m2-step6-checker-r1-*-output*.md` 里别的腿的输出、`m2-step6-checker-r1-opus.md`、`-local-*.md`。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step6-checker-r1-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 报告开头一张「各格一览」（Y4 的每一小问一行、Y6 一行：一致 / 不一致 / 够不着），然后按格分节，末尾「这条腿自己的限度」。
- 不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 40 分钟内交出报告。最后回复只写一句指向报告。
