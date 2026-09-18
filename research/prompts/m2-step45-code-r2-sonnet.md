你是里程碑「第二个事务」步 4 / 步 5 代码三方对抗**第二轮**（轮名 m2-step45-code-r2）的**云端辩方腿**。你的立场是复核第一轮的判决 `research/prompts/m2-step45-code-r1-main-verification.md`：每一格判得够不够得着、打中的是不是真打中（引的条款与代码对不对）、改法是不是同样修好所有替代方案、没打中的是不是真没打中、处置合不合跑前条款——并替被判出局的一方（P1、窄读法、最新根自己的 F、只用 F_生效 做回收门槛）辩护：它们有没有一个读法能站住。另核 X7（写回）。

## 先读

1. 第一轮判决 `research/prompts/m2-step45-code-r1-main-verification.md`、第一轮正文 `_m2-step45-code-r1-body.md`（跑前条款在它第三节）、第一轮四条腿的报告（`m2-step45-code-r1-opus-output.md`、`-sonnet-output.md`、`-local-attack-output-s1.md` / `-s2.md`、`-local-defense-output-s1.md` / `-s2.md`）、核查员报告 `m2-step45-code-r1-verifier-output.md`。
2. 这一轮的背景材料 `research/prompts/_m2-step45-code-r2-background.md`（正文 S1–S6 改法表、X1–X7 判据；附录一条款整段）与附录二 `_m2-step45-code-r2-diff.md`。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`、`allocator.rs`、`recovery.rs`、`crates/singlefs-harness/src/crash.rs`、用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`；行号去原文件现查、写原文件的（核查员抓到第一轮正推腿 12 处 kb 行号写成标题行）。
4. 草稿放你自己的目录 `/tmp/claude-1000/m2-step45-code-r2-sonnet/`；要跑东西把仓拷到那里（`rsync -a --exclude target --exclude .git`），副本上的数注明是副本；第一轮判决第五节说每条变异各红、用例各绿——在副本上把那几条变异各跑一遍，贴原样结果。

## 要复核的格

- 第一轮判决第四节 X1–X4、X8 每一格：报告的历史在今天的代码上还成不成立（改法之后哪些消失了、哪些还在）；「X4 第五条是死条款」够不够得着（有没有一段历史让 `rollback_high_water_of_root` 取到真）。
- 第五节五处改法：每处的会红用例是不是真钉住那条改法（把改法撤回它红、别的改法撤回它不红）；P2 与 P1 是岔路——判决说「代价数没有」，你替 P1 辩：P1 在哪些格上比 P2 好，那些格今天有没有用例。
- 第六节决策点：每一条是不是真没有条款（全仓 grep 一遍），有条款的就不是决策点。
- X7：`02-second-txn.md` 步 4 / 步 5 现状、`second-txn-layout.md`、第一轮判决第五节的每一句能核的都对着代码核。

## 规矩

- 不替攻方找新反例（那是攻方腿的）；只判第一轮判决对不对、处置合不合条款。
- 每一格指到代码行号与附录原文；说「判对」也要写是哪一句对上哪一句。
- 禁读：`research/prompts/m2-step45-code-r2-*-output*.md` 里别的腿的输出、`m2-step45-code-r2-opus.md`、`-local-*.md`。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step45-code-r2-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 报告开头一张「各格复核一览」（第一轮判决第四节每格、第五节每处改法、第六节每条决策点各一行：判对 / 判错 / 够不着），然后按格分节，末尾「这条腿自己的限度」。
- 不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告。最后回复只写一句指向报告。
