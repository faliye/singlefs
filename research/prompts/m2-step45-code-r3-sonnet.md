你是里程碑「第二个事务」步 4 / 步 5 代码三方对抗**第三轮**（轮名 m2-step45-code-r3）的**云端正推腿**。你的立场是核「代码做的是不是条款说的」：第二轮与 alloc-basis 第二轮打中之后落地的三处改法（T1 G5 影子账、T2 读不出账的被抛弃根跳过并计数、T3 抬 F 回收的槽扣住到生效）与它们压着的条款逐格对；另核 F_生效 回落那一格（第二轮判决第六节 2）三个改法方向各修哪一格，以及 X7 写回。

## 先读

1. 背景材料 `research/prompts/_m2-step45-code-r3-background.md`（正文 T1 / T2 / T3 改法表、X1–X8 判据、分工、禁读清单；第二轮判决全文；diff；附录一条款整段）。引条款从附录抄整行，行号写 kb 文件自己的（去 kb 文件里现查，不从背景材料里数——核查员抓到过第一轮正推腿 12 处 kb 行号写成标题行）。
2. 第二轮判决 `research/prompts/m2-step45-code-r2-main-verification.md`、第二轮云端攻方报告 `m2-step45-code-r2-opus-output.md`（X3 那一节的「三个改法各修哪一格」那张表是你要逐格核的）、alloc-basis 第二轮云端攻方报告 `research/prompts/alloc-basis-r2-opus-output.md` 的 2.1 / 2.2 节与第七节第 1 条。
3. 自己去读代码（原仓只读）：`crates/singlefs-core/src/mount.rs`（`isolate_slots_referenced_only_by_abandoned_roots`、`rebuilt_allocator`、`raise_rollback_floor`、`mount_rollback`、`establish_instance`、`MountOutput`、`MountError`）、`allocator.rs`（`DeviceFreeMap` 全部、`ReclaimedReuse`、`reclaim_released_up_to`、`release_reclaim_holds`、`allocate_commit_generated`）、`recovery.rs`（`allocation_records_under_root`、`effective_rollback_floor`）、`crates/singlefs-checker/src/walk.rs`（I-3.1 的并集怎么取候选集）、用例 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`second_transaction_step_five_reuse.rs`。
4. 草稿放你自己的目录 `/tmp/claude-1000/m2-step45-code-r3-sonnet/`；要跑东西把仓拷到那里（`rsync -a --exclude target --exclude .git`，副本里 `nice -n 19`），副本上的数注明是副本。第二轮判决第五节与正文第一节说每条新变异各红、用例各绿——在副本上把 `crates/mutations.tsv` 里第 50 行之后的每条（步 4 / 步 5 的新行）各跑一遍，贴原样结果。

## 要核的格

- **X6 代码与条款**：T1——G5 与用户 2026-09-16 窄读法措辞（D23（journal 的角色与格式） 已定项 14 那句「仍被有效根引用的槽不在其内」）的差别是不是只有「txg ≥ F」这一条；D23 主句「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配」在 G5 下逐格成立吗（固定脚本每一步：回退那次、四次覆盖写、抬 F、E）；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的定义与 `isolated_slots` 在每一步上是不是同一个数。T2——D23 主句对「读不出账的被抛弃根」说了什么、没说什么。T3——D16（发布语义） 已定项 1 生效行与可再分配行：扣住到「带新 F 的根落满每块盘」是不是条款说的「生效」；记账在第一条带新 F 的根之前动，与 I-3.1（已分配统计对得上） 的并集口径（checker 按最新根自己的 F）在崩在两条根之间的那个镜像上对不对得上（副本上崩了跑 checker）。
- **方向**：第二轮攻方腿 X3 那张表——(a) 新根写 max(上一条根的 F, F_生效)、(b) 候选集也用 max、(c) 只改 checker——按今天的代码逐格核：每个改法在 X3-A / X3-B / X3-C / X3-D 四格上各中不中，它说的对不对；有没有第四个改法能把四格都修掉而不改用户定过的条款字面。
- **X7 写回**：`02-second-txn.md` 步 4 / 步 5 现状、`invariants.md` I-3.1 那一行、`checks-owed.md` C340（回退之后记录链从哪条之后接没有定义） 第三列、第二轮判决第五节的每一句能核的都对着代码核。

## 规矩

- 不替攻方找新反例（那是攻方腿的）；只判代码与条款对不对得上、判决与处置合不合条款。
- 每一格指到代码行号与附录原文；说「一致」也要写是哪一句对上哪一句。
- 禁读：`research/prompts/m2-step45-code-r3-*-output*.md` 里别的腿的输出、`m2-step45-code-r3-opus.md`、`-local-*.md`。

## 交付

- **分段写**：报告写进 `research/prompts/m2-step45-code-r3-sonnet-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 报告开头一张「各格一览」（T1 / T2 / T3 各一行、方向一行、X7 一行：一致 / 不一致 / 够不着），然后按格分节，末尾「这条腿自己的限度」。
- 不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告。最后回复只写一句指向报告。
