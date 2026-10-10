# 删除批第 20 批代码轮第一轮：云端正推腿（Sonnet）报告（2026-10-05）

本腿只判 K1（N1–N9 条款对实现）与 K3（12 份被判文件每份一格）；K2 归攻方腿，不攻。方向：从条款、设计轮判决格、第 18 批代码轮判决与用户定案，推到冻结副本里的行，再推到钉它的测试与变异行。代码行号都是冻结副本 `/tmp/claude-1000/m3-batch20-code-r1/tree/` 里的全路径:行号；kb 与判决的行号是仓里主工作区那份文件自己的行号，开写前逐句 `grep -nF` 过。禁读清单里的攻方报告与模型一个字没读。

## 0 读过什么与复核的事实

读过：`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`「各条腿必须互不重复」、`.claude/rules/implementation-first.md`「规矩」、`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 开头三步表、背景正文与改动全文（四份补丁与变异追加）、`.claude/kb/invariants.md` 第 6 行与 I-9.5 / I-4.7 / I-4.6 / I-9.6 四行、`.claude/kb/decisions/08-核心索引结构.md` 已定项 16、`.claude/kb/checks-owed.md` C708 / C712 两行、设计轮判决与第 18 批代码轮判决、`research/prompts/e172-preregistration.md` 5.3 与 5.5、实现员 a 的报告（线索）。

命令 1（冻结副本与快照 sha256 对表）：
`cd /tmp/claude-1000/m3-batch20-code-r1/tree && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-batch20-code-r1-snapshot/crates-sha256.txt | grep -c ': OK$'` 输出 `12`（12 份全 OK，没有 FAILED）。

命令 2（变异表：新 35 行的锚点与测试名）：脚本逐行数「原文在目标文件里命中几次」与测试名在 crates 里是否存在，输出 `bad 0`，35 行各命中 1 次；全表 2728 行里只有第 1354 行（`litmus/publish-returns-after-the-rotation-barrier.litmus`，不在冻结副本的 `crates/` 里、不是这一批的行）找不到文件，其余 2727 行各命中 1 次。x2 要删的两行（「三·6 删除导出：写清零的种类名打成 write」「三·6 删除导出：屏障也重建出（空的）字节」）在冻结副本 mutations.tsv 里 `grep -c` 为 0。改动全文里三十五行追加与冻结副本里第 2682–2716 行逐字相同（脚本比，`all in tree: True`）。

命令 3（harness 档基线，副本拷在草稿目录 `/tmp/claude-1000/m3-batch20-code-r1-sonnet/work/tree/`，目标目录 `/tmp/claude-1000/m3-batch20-code-r1-sonnet/target/`，每条经 `run-with-memory-cap.sh` 6G、`capped.sh 6`）：
- `cargo test --offline -p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version`：`test result: ok. 28 passed; 0 failed`。
- `--test deletion_scenario_writes`：`test result: ok. 5 passed; 0 failed`。
- `--test checker_known_bad_images`：`test result: ok. 89 passed; 0 failed`。
- `--lib`：`test result: ok. 84 passed; 0 failed; 1 ignored`。
- `--test unlink_single_file_publishes`：`test result: ok. 9 passed; 0 failed`。
内存包装第一次拒了一条（退 252，同会话已有日常在跑），之后按它给的出路设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（理由写「正推腿与攻方腿是各自派出的子 agent 各跑各的」）；有几条命令被峰值表挡在 2 倍以内，改了上限（600M、1138M）或设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`，无一次 250–254 的结局被当结果用。开跑前 `ps -o pid,args -u "$(id -u)"` 看到的只有别的会话的 `cargo test -p singlefs-harness --test model_compares_the_third_and_fourth_inode_leaf_containers`，没有 qemu、fio、vm-bench、e152；本腿的 cargo 用自己的目标目录，没有等别人的文件锁。

命令 4（变异按点改坏复跑，harness 档的 23 行）：`python3 research/prompts/m3-batch20-code-r1-sonnet-model/mutation-rerun.py <行号…>` 在草稿目录的副本上逐行把原文换成替换文、跑那一行点名的测试、还原（还原后副本的 `crates/` 与冻结副本逐文件 sha256 相同）。第 2682–2698 行（a 的 17 行）、第 2703–2708 行（x 的 6 行）共 23 行，每行点名的测试都红（`test result: FAILED. 0 passed; 1 failed`，没有编译错），汇总在 `research/prompts/m3-batch20-code-r1-sonnet-model/mutation-results.txt`（23 行，`grep -c 'named_test_FAILED=1'` 输出 `23`）。

## K1-N1　L1 改法 A：反方向也配

条款与判决（逐句）：
- 设计轮判决：「| F4-b 配对 | 推到经树表条目偏移 116 的头 ID 配，配不上记判不了 |」（`research/prompts/m3-delete-checker-r1-main-verification.md:25`）。
- kb：I-9.5 行共用判法「配不上的版记判不了」（`.claude/kb/invariants.md:292`）与「树表、`deleted_inodes` 或 inode 树读不全的版报 `part_of_the_range_not_judged`」（`.claude/kb/invariants.md:292`）。
- 第 18 批代码轮判决 L1 一格：「改法照攻方的改法 A（反方向也配：每棵 inode 树都要恰好一条 `deleted_inodes` 指着它，配不上记判不了），进下一批」（`research/prompts/m3-batch18-code-r1-main-verification.md:26`）。

代码：
- 正方向（旧）：「.filter(|entry| kind_of(entry) == TREE_KIND_DELETED_INODES)」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8360`）；`deleted_inodes` 头 ID 配不上或配到不止一条 inode 树：「Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)」（同文件 `:8383`）；extent 树同理：「[] | [_, _, ..] => Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),」（同文件 `:8414`）。
- 反方向（新）：「for inode_tree_entry in tree_table_entries」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8446`）逐棵 inode 树数「种类 9 且头 ID 等于这棵树 ID」的 `deleted_inodes` 条目数，「if deleted_inodes_entries_naming_the_inode_tree != 1 {」（同文件 `:8458`）则对三条各记 `DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry`（枚举变体在 `:7942`，理由串在 `:7964`）。
- 三条各报什么：`DELETION_QUEUE_SCAN_INVARIANTS` 三个元素（`walk.rs:7989` 起）都走 `part_of_the_range_not_judged`；`into_report` 的次序是违例、部分没判、射程里判不了、评估过成立、不适用——「} else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/image.rs:311`）后面接 `InvariantVerdict::NotApplicable(reason)`，所以配不上的版没有违例时三条报「不适用并带理由」，别的版判出违例照报违例。
- 与 `invariants.md` 文件头「判定有四种」对：文件头写「部分没判是判到的那一部分都对得上、射程里另有一部分这张镜像上判不了」且「今天只有 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）报它。」（`.claude/kb/invariants.md:6`），I-9.5 行又写明读不全的版报 `part_of_the_range_not_judged` 而不是第四种判定、是否改报随 C701 那一问（`.claude/kb/invariants.md:292`）。新增的配不上一形报「不适用并带理由」，落在四种之内的第三种，没有越到第四种；与 I-9.5 行、F4-c 同一报法。一致。

实跑（探针 `research/prompts/m3-batch20-code-r1-sonnet-model/probe_n1.rs`，输出 `research/prompts/m3-batch20-code-r1-sonnet-model/probe_n1.out`）：意图之后那一版干净时 `violated=[]`，I-9.5、I-4.7、I-4.6 都 `Holds`；L1a（种类码 9 → 13）与 L1c（只改中间版）都是 `violated=["I-9.16"]`，三条都是 `NotApplicable("走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的 deleted_inodes 条目不是恰好一条：那一版这棵树的队列与记录判不了")`。

钉它的测试与变异：L1a、L1b、L1c 三个用例（`pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 第 428、455、481 行起）各断言红集恰好 `["I-9.16"]`（「vec!["I-9.16"],」在 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:441`、`:466`、`:503` 三处各一）和三条的理由含「头 ID 指着它的 deleted_inodes 条目不是恰好一条」；变异第 2682–2684 行把 `!= 1` 改成 `false &&`，三条点名的测试复跑都红。

遗留的次要形（不算打中，给攻方腿与主 agent 参考）：两条 `deleted_inodes` 都指同一棵 inode 树时，正方向对每一条各判一遍（队列各读各的），反方向再记判不了；若其中一条队列与树不符，I-9.5 会先判红、压过判不了。那一形的树表本身违反 I-9.16 ③（种类各恰好一条），不是从入口走得到的已发布状态，按第五节判据不算误红。推翻这条的现象：入口走得到一个已发布状态，树表里 `deleted_inodes` 出现两条。

判定：一致。什么现象会推翻：任一合法删除中间态上三条之一报「不适用」带这句理由，或造得出两个方向都配上而 I-9.5 / I-4.7 / I-4.6 漏判的树表形态（归 K2）。

## K1-N2　L2：I-4.6 不适用的理由
条款：第 18 批代码轮判决第三节第 2 条「2. L2：不适用的理由串与镜像对上。」（`research/prompts/m3-batch18-code-r1-main-verification.md:39`），L2 一格写「改法 A 之后那一版走判不了、带对的理由；理由串与镜像对不上那一处并进下一批」（`research/prompts/m3-batch18-code-r1-main-verification.md:28`）。

代码：这一批的改动里**没有改**那句不适用理由串，它还在原处：「"走读到的每一版 inode 树里都没有 nlink = 0 的记录",」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8288`）。N2 是经 N1 达成的：L2 那一版（有 nlink = 0 的记录而没有 `deleted_inodes` 条目指着）现在走 `part_of_the_range_not_judged`，`into_report` 里它排在「评估过成立」与「不适用」之前（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/image.rs:311`），所以报出来的理由是配不上那一句，旧句只在「评估次数为 0、也没有记过判不了」时出现——那时每一版的 inode 树里确实没有 nlink = 0 的记录，旧句是真话。背景表格写的「I-4.6 不适用的理由串换成配不上那一句」按效果读一致，按字面读（换串）不是这样做的；主 agent 若要字面换串，是另一个改法。

实跑：探针里 L2 镜像（L1a 之上 1 号 size 多一个净荷）红集是 `["I-9.16"]`，I-4.6 不红——size 超前的违例在配不上的版上被判不了盖住，不是漏判（F4-b「配不上记判不了」）。钉它的用例是 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:518` 起的那条（断言理由含配不上那句、且不含「没有 nlink = 0 的记录」），变异第 2685 行复跑红。

判定：一致（经 N1）。推翻它的现象：某一形里 I-4.6 报不适用而理由说「没有 nlink = 0 的记录」、镜像上却有 nlink = 0 的记录。

## K1-N3　未钉格

逐项（每项：条款 → 代码 → 用例 → 变异，变异都是本腿按点改坏复跑的结果）：

| 项 | 条款或判决 | 用例（pool_checker… 文件里的行） | 变异（mutations.tsv 行） | 复跑 |
|---|---|---|---|---|
| I-4.6 size = 0 那一支 | 「size 为 0 时 0 或 1 个单元」（`.claude/kb/invariants.md:182`），代码 `data_units <= 1`（walk.rs:8569） | 748 起，搭建 `pool_with_a_first_file_of_size_zero_and_one_data_unit`（common_deletion_in_progress/mod.rs:211）走发布路径写，断言一个数据指针、红集 `Vec::<&str>::new()`、三条 `Holds` | 2687（`<= 1` → `== 0`） | 红 |
| 中间版写坏 I-9.5 | F4-a「定候选集每一版加记录施加出来的几版」 | 772 起：只改中间版 2 号 nlink → 0，红集恰好 `[I-9.5]`，说明含中间版 txg | 2689（只扫最新根） | 红 |
| 中间版写坏 I-4.7 | 同上 | 807 起：只拿掉中间版 1 号记录，红集恰好 `[I-4.7]` | 2690（只扫最新根） | 红 |
| 墓碑「未发布」一支 | I-9.6 行「按实例表判」 | 730 起：容器头写序实例 1 → 2、对象 ID 抬到水位，I-9.6 `Holds` | 2688（去掉 `if !is_published` 的过滤） | 红 |
| C708 还差的一形 | `.claude/kb/checks-owed.md` C708 行「**还差一形**」 | 847 起：队列 {1} 不动，1 号 nlink 0 → 1，红集恰好 `[I-9.5]`，说明含「deleted_inodes 里的号 1」「nlink = 1，不是 0」；也登记进 mod.rs:814 的 `known_bad_images_after_the_intent_of_a_three_unit_file`，`checker_known_bad_images` 89 条全绿 | 2686（正方向不核 nlink） | 红 |
| `DeletionScanGap` 七个零用例变体 | 判决第三节第 3 条 | 见下表 | 2692–2698 | 7 行全红 |

七个变体与新变体各自的用例（行为 `pool_checker…rs` 的行，理由子串都是用例里断言的）：

| 变体 | 用例行 | 造法 | 变异行 |
|---|---|---|---|
| `TreeTableUnreadable` | 662 | 中间版树表两份全零，不补引用链 | 2692 |
| `TreeTableEntryShorterThanTheFieldTable` | 688 | 中间版树表条目宽收窄到 8、重封补引用链 | 2693 |
| `DeletedInodesHeadIdentifierNamesNoSingleInodeTree` | 546 | `deleted_inodes` 头 ID 改 0，红集恰好 `["I-9.27"]` | 2694 |
| `DeletedInodesRootUnreadable` | 644 | `deleted_inodes` 根两份全零 | 2695 |
| `ExtentTreeOfTheInodeTreeNotSingle` | 576 | extent 条目头 ID 改 0，红集恰好 `["I-9.27"]`，I-9.5 `Holds` | 2696 |
| `ExtentTreeUnreadable` | 615 | extent 根两份全零，I-9.5 `Holds` | 2697 |
| `TombstoneTreeUnreadable` | 714 | 墓碑容器两份全零 | 2698 |
| `InodeTreeNamedByNoSingleDeletedInodesEntry`（新） | 428、455、481、518 | L1a / L1b / L1c / L2 | 2682–2685 |

照旧没有用例的变体：`AccountingTreeUnreadable`（`grep -c AccountingTreeUnreadable crates/mutations.tsv` 输出 0；`grep -rn '记账树读不全\|AccountingTreeUnreadable' --include=*.rs singlefs-harness singlefs-checker-tier` 零命中）。判决第三节第 3 条点名的是「九个变体里零用例的七个」，这七个没有它；`InodeTreeUnreadable` 是旧的一个（355 起，变异 mutations.tsv 第 2610 行）。所以这一项不算冲突，但十个变体里有一个仍没有按理由串断言的用例和变异行，写在这里。

「断言是不是恰好」：只红目标的断言（`violated_invariants` 恰好等于一个集合）只有 L1a / L1b / L1c（`["I-9.16"]`）、头 ID 改 0 与 extent 头 ID 改 0（`["I-9.27"]`）、C708 一形（`[I-9.5]`）、两条中间版（各一条）、size = 0 一支（空集）。读不全型的用例与 L2、墓碑未发布一支**没有**断言红集，只断言目标那条的判定。用探针把这几份镜像的红集量了一遍（`research/prompts/m3-batch20-code-r1-sonnet-model/probe_n3.out`，探针 `probe_n3.rs`）：
```
P L2 violated=["I-9.16"]
P extent-root-zero violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
P deleted-root-zero violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
P tree-table narrow=false violated=["I-2.1", "I-3.1", "I-4.8", "I-7.4"]
P tree-table narrow=true violated=["I-4.8", "I-7.4"]
P tombstone-zero violated=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
P tombstone-unpublished violated=["I-1.2", "I-1.11", "I-4.2", "I-7.7"]
```
这几份镜像本来就不是「红目标」的镜像（它们是「目标判不了而不判红」的镜像），别的条红的是读不全 / 写序不是已发布这些主走读条，用例的点名断言（目标那条报不适用带对的理由、不报违例）对得上；没有哪一份把目标条判红。

kb 状态列落后一处（写回批的事，不是代码错）：C708 行与 I-9.5 行都还写「还差「队列里的号在树里 nlink ≠ 0」那一形的坏镜像」（`.claude/kb/invariants.md:292`），而这一形今天有了。

判定：一致，一处小缺口（`AccountingTreeUnreadable` 无用例无变异，不在判决点名的七个里）。推翻它的现象：上面任一行的变异复跑没红，或某个用例过的不是点名的那一支（理由子串换了仍绿）。

## K1-N4　`image.rs` 过期注释

条款：判决第三节第 6 条点名「`crates/singlefs-checker/src/image.rs` 第 86–87 行过期注释（说 GPU 内核还没判，第 18 批 P2 已判）」（`research/prompts/m3-batch18-code-r1-main-verification.md:43`）。

代码：「I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器的扫描内核判前四条」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/image.rs:86`），「（下标 69–72，删除态扫描，第 18 批 P2），下标 73 的 I-4.4 第一版恒报不适用、内核不置位（`crash_judge_gpu` 的覆盖表）。」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/image.rs:87`）。新注释的两个事实各有旁证：覆盖表注释写「73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:69`），扫描内核写「// I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2064`）。只改注释，不改代码。

判定：一致。推翻它的现象：覆盖表里下标 69–72 有一个内核并不判，或下标 73 内核置位。

## K1-N5　GPU 恢复内核补 nlink

条款：D8 已定项 16 第 4 条「从意图提交起只读挂载、冷走读、模型读回答」没有文件（`.claude/kb/decisions/08-核心索引结构.md:467`）；C712 行的验收是「恢复内核读偏移 36 的 nlink，为 0 就答没有文件（`walk_no_file`）」（`.claude/kb/checks-owed.md:567`）。背景正文写「用户 2026-10-03 定 GPU 判器全覆盖」，覆盖表上方的代码注释写的是「用户 2026-10-04 定」改回要求全覆盖（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:71`）：日期差一天，内容同一件事，不影响判定。

逐支对（CPU 在 `singlefs-core/src/recovery.rs`，GPU 在 `crash_judge_common.wgsl`，都在冻结副本 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/` 下）：

| 支 | CPU | GPU |
|---|---|---|
| 一条 1 号记录都没有 | 「let Some(inode_record) = inode_record else {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5073`）→ `Ok(None)` | 「if (!found_inode) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3272`）→ `walk_no_file()` |
| 1 号记录 nlink = 0 | 「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5077`）→ `Ok(None)` | 「if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3277`）→ `walk_no_file()` |
| 判的位置 | 取到 1 号记录之后、核数据单元个数「data_unit_count_matches_the_inode_size(roots.extent.data_pointers.len(), &inode_record)?;」之前（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5085`） | 同位置，在「if (!eq64(expected_units, u64_of(data_count))) {」之前（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3286`） |
| 同号多条记录 | 循环里后到的盖前面的：「inode_record = Some(record);」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:5069`） | 循环里每条 1 号记录都重写 `found_inode`、`inode_object_birth`、`inode_nlink`、`inode_size`，后到的盖前面的（同文件 3264–3268 行） |
| 多片 inode 叶 | 按 `roots.inode.entries` 的次序逐片读 | 外层循环 `for (var e = 0u; e < inode_root.entry_count; e = e + 1u)`，同一次序 |
| nlink 读的偏移 | 「let nlink = reader.get_u32();」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/records.rs:101`）前面读了 8 + 8 字节再跳 8 + 4 + 4 + 4：「reader.skip(8 + 4 + 4 + 4); // locality_id、mode、uid、gid」（同文件 `:100`），共 36 | 「const INODE_RECORD_NLINK_OFFSET: u32 = 36u;」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:135`）；读法「inode_nlink = rd32(leaf.base, ro + INODE_RECORD_NLINK_OFFSET);」（同文件 `:3267`），`rd32` 对非对齐偏移走逐字节拼（同文件 215–220 行） |
| 记录体坏（偏移 108 起非零、号小于分隔 key / 容器号） | `InodeRecord::parse` 返回 `None`；「if record.inode < separator_key || record.inode < identity.container {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-core/src/recovery.rs:4616`）→ 错 | 「if (lt64(inode, separator) || lt64(inode, identity_container)) {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3261`）→ `walk_failed(53u)`；偏移 108 起 32 字节非零 → `walk_failed(52u)` |

`inode_nlink` 初值 0 与 `NLINK_OF_AN_INODE_BEING_DELETED` 同值，但只在 `found_inode` 为真之后才读（3272 行已先返回），不会把「没找到」当成「nlink = 0」。两个常量与 `singlefs-core` 的 `NLINK_OF_AN_INODE_BEING_DELETED`（`records.rs:42`）同值 0，注释指向它。

GPU 判器的扫描内核（`crash_judge_checker_scan.wgsl`）不用跟 N1：它只从 `deleted_inodes` 条目一侧去配（「// I-9.5、I-4.7、I-4.6（walk.rs judge_deletion_queue_of_a_version）：每一条 deleted_inodes 条目」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2418`），配不上直接 `return`（「if (inode_tree_entry_count != 1u) {」，同文件 `:2317`），与 CPU 的旧一遍同形；CPU 新增的反方向一遍只记判不了、不置违例位，GPU 判器只出违例位，两边位相同。这回答了第 18 批判决 K2 N8 逐支一格那句「GPU 判器只出违例位，不用跟（推的，进下一批时由实现员核）」（`research/prompts/m3-batch18-code-r1-main-verification.md:30`）：读代码属实。

钉它的变异：4 行（第 2699–2702 行）原文在 wgsl 里各命中 1 次，必须红的测试是 checker 档 GPU 对拍，本腿**复核不了**（GPU 不跑、checker 档不许子 agent 跑）。读代码：第 2699–2701 行把 nlink 判整个关掉（`false &&`），则意图那一版起 GPU 照读文件、CPU 答没有文件，整份比的三条用例（删除中间态、只差一个写、墓碑坏流）必红；第 2702 行把 nlink 判挪到核单元数之后，size 超前那一流上 GPU 先 `walk_failed(54u)`、CPU 答没有文件，必红。

判定：一致。推翻它的现象：对拍里有一个状态 GPU 与 CPU 恢复结局不同且与 nlink 有关（本腿读不出这样的支）；或同号两条记录前一条 nlink = 0、后一条 nlink = 1 时 CPU 与 GPU 取的不是同一条（两边都取后一条，读代码一致）。

## K1-N6　对拍覆盖

条款：第 18 批判决 K2 N8 对拍覆盖一格「**坐实为覆盖缺口，不是判法分叉**：进下一批，与 C712（GPU 恢复内核不判 nlink = 0） 一起做」（`research/prompts/m3-batch18-code-r1-main-verification.md:31`）。
冻结副本里 6 条用例都换成整份比，旧函数一个不剩：
- `grep -c 'compare_the_checker_on_a_deletion_flow' <对拍文件>` 输出 `0`。
- 比较函数里「|| gpu_recovery != *cpu_recovery」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2047`）与 `undecidable`、红位、池级 checker 违例位一起进 `mismatches`；「fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(」（同文件 `:1755`）断言 `comparison.mismatches.is_empty()`。
- 用它的 5 条：5334（队列号没有记录）、5349、5364（待删占用行不为零 / 缺行）、5408（size 超前）、5424（墓碑到水位）各 `fn` 行；加上「删除中间态」那条（5163）直接断言 `comparison.mismatches.is_empty()`，一共 6 条，与 C712 行验收写的「六处」相同。另两条（5380、5395）本来就整份比。

新用例（5253 起）取的状态：
- 构造：「fn segments_missing_one_write_of_each_unexpanded_segment(」（同文件 `:5209`）对写数超过展开上限的每一段、段里每一次写各造一种切法——把那一段拆成「其余的写」与「只有这一次写」两段，只展开后一段。展开语义见 `crash.rs`：`NotExpanded` 是「只以整段持久进入后面的状态」，`EveryProperSubset` 对一个写的段只多出「那一个写没落」这一个状态（`2^1 - 1`），再加最后全落那一个，共 2 个。所以每种切法的状态是「之前的段全落、这一段只差这一次写」，与判决要的「只差一个写」同形。
- 守卫：拆段只对单元写成立（单元写是 COW、只取两态），「flow.writes[*write_index].kind,」（同文件 `:5220`）断言每一写是 `StepKind::UnitWrite`；每种切法恰好两个状态的断言在取卡之前跑（同文件 5285 行的消息「第 {} 段只差第 {} 次写的切法只有那一个状态与最后全落那一个」），没卡的机器上也判到。
- 个数：判决引的老数据 `research/prompts/m3-batch18-code-r1-opus-model/out/gpugap.out:1` 里「not_expanded_segments=[(7, 24), (11, 30), (15, 26), (19, 24)]」，24 + 30 + 26 + 24 = 104，与判决写的 104 个状态相同；新用例的个数是运行时由 `flow.segments` 里写数超过 `limit` 的段算出来的，会在 `CRASH_JUDGE_GPU_MISSING_ONE_WRITE ... states_missing_one_write=` 那一行打出（提交时 `= 104` 才算对上）。**今天的流是不是还是这四段，本腿复核不了**（录这条流要 checker 档包的代码）。
- 小提示：展开上限环境变量设到 ≥ 30（四段都不再算「不展开」）时，用例开头的 `!variants.is_empty()` 断言会 panic，而不是空过；设成这么大本来就等于全量枚举（约 1.17×10^9 个状态），不是平时用法。

钉它的变异：第 2699–2702 行里第 2700 行（第一批那一段只差一个写）专门点名这条新用例；其余三行点名整份比的旧用例与墓碑 / size 超前两条。GPU 对拍本腿不跑。

判定：一致（GPU 一侧复核不了）。推翻它的现象：提交时 `states_missing_one_write` 不是 104，或这条用例在有卡的机器上红。

## K1-N7　E142 导出 bin：mkfs 拒的 `--device-bytes`

条款：第 18 批判决第三节第 5 条「5. N9：`--device-bytes 0` 的退出码与文件头对齐。」（`research/prompts/m3-batch18-code-r1-main-verification.md:42`）；K2 N9 一格给的是二选一，实现取「解得开而 mkfs 拒的值也退 1 并打原因」（`research/prompts/m3-batch18-code-r1-main-verification.md:33`）。

代码：文件头改成「1 = 命令行参数不是上面两种之一，或每盘字节数解得开而」`make_filesystem` 不收……（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:19`）与「101 = mkfs 收了、后面某一步（取号、暖机、新池新建文件）在内存盘上没走通，panic 带那一步与原因。」（同文件 `:21`）。分流在 `ending_of_an_unfinished_path`：「NewPoolFileCreationPathStep::MakeFilesystem => {」（同文件 `:84`）走退 1，其余三步（取号、暖机、新池新建文件）仍 panic。

实跑（本腿在副本上 `cargo build -p singlefs-checker-tier --bin …` 后直接跑编出来的 bin，经 `run-with-memory-cap.sh 2G / 4G`，命令里 `--device-bytes <值>`，输出原样）：
```
[--device-bytes 0] exit=1 stdout_bytes=0
e142_new_pool_file_creation_write_dump 每盘 0 字节 mkfs 不收：mkfs：JournalRingTooLargeForDevice { ring_bytes: 805306368, device_bytes: 0 }
[--device-bytes abc] exit=1 stdout_bytes=0
e142_new_pool_file_creation_write_dump --device-bytes 后面要一个十进制字节数，收到 "abc"：invalid digit found in string
[--device-bytes 1048576] exit=1 stdout_bytes=0
e142_new_pool_file_creation_write_dump 每盘 1048576 字节 mkfs 不收：mkfs：JournalRingTooLargeForDevice { ring_bytes: 805306368, device_bytes: 1048576 }
[--bogus] exit=1 stdout_bytes=0
e142_new_pool_file_creation_write_dump 只收零个参数或 --device-bytes <每盘字节数>，收到 ["--bogus"]
```
另跑 `1`、`805306368`、`1073741824`：都 `exit=1` 带 `JournalRingTooLargeForDevice`；`1099511627776`（1 TiB）：`exit=0`、输出 1089093 字节；`18446744073709551616`：`exit=1`，「number too large to fit in target type」（解不开那一支）。不带参数：`exit=0`，957310 字节，sha256 `50a22a0326e64baa835a19ed9cc4daf143d8b6a31e59ab07d5213c43f1175c55`。

一个文件头没写的出口（规则没说，不算打中）：`--device-bytes 18446744073709551615`（解得开、mkfs 还没轮到）退出 134，stderr「memory allocation of 140737488349056 bytes failed」；`--device-bytes 576460752303423488` 同样 134；32 TiB（`35184372088832`）在 4G 上限下被包装结束（250，不算结果）。这些是内存盘 `SparseBlockDevice::new` 按盘大小分配结构，先于 mkfs 就撞内存；文件头列的是 0 / 1 / 101 三个出口，没有这一形。改动前的行为相同（建盘在 `run_new_pool_file_creation` 之前，这一批没碰），所以不是这一批引入的。

钉它的测试与变异：用例 `zero_device_bytes_parse_but_make_filesystem_refuses_them_and_the_bin_ends_as_a_refused_command_line`（同文件 `:332`）断言原因含 `JournalRingTooLargeForDevice`、录制流 0 个操作；`a_failure_after_make_filesystem_stays_an_implementation_failure`（同文件 `:362`）对另三步断言仍是实现的毛病。变异第 2709、2710 行读代码：第 2709 行把 `MakeFilesystem` 一支改成 `StepAfterMakeFilesystemFailed`，第一条用例里的 `match` 走到 panic 红；第 2710 行把 `WarmUp` 改成 `DeviceBytesRefusedByMakeFilesystem`，第二条用例红。这两条用例在 checker 档包里，本腿**复核不了**测试本身（上面是直接跑 bin 的行为）。

判定：一致。推翻它的现象：mkfs 拒的值退出码不是 1、或 stderr 不带 mkfs 的原因；或这两条用例在 `cargo test -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump` 里红。

## K1-N8　`kind_name` / `operation_bytes` 挪进 harness

条款：「重复要生成，不许手抄」（`.claude/singlefs-ai-sop/rules/code-discipline.md:179`）。

代码：`RecordedOperationKind::name`（「pub const fn name(self) -> &'static str {」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/lib.rs:46`）、`RetainedOperation::written_bytes_rebuilding_zero_fills`（「pub fn written_bytes_rebuilding_zero_fills(&self) -> Option<Vec<u8>> {」，同文件 `:129`）。`to_stream_line` 也改用它（「let kind = self.kind.name();」，同文件 `:72`）；两个 bin 里「operation.kind.name(),」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:250`、`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:268`）与「.written_bytes_rebuilding_zero_fills()」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:243`、`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:261`）取代各自手抄的函数。

等价：
- 读 diff：新函数四个种类名与旧 `kind_name` 逐字相同（`write`、`write_fua`、`write_zeroes`、`barrier`），字节重建三支（普通写与 FUA 写取 `contents.clone()`、写清零按长度铺 0、屏障 `None`）与旧 `operation_bytes` 逐支相同，只有 `expect` 的消息文字变了。
- 跑：e142 导出不带参数，改前（取主工作区 `git show HEAD:` 那一份，它有 `kind_name`，8 处命中，没有 `--device-bytes` 选项）与冻结副本的 bin：`50a22a0326e64baa835a19ed9cc4daf143d8b6a31e59ab07d5213c43f1175c55`，957310 字节，两份相同。删除导出不带参数：10118080 字节，`research/results/e172-deletion-write-dump-2026-10-05.out` 同为 10118080 字节，两份只在 `offset=0` 与 `offset=4096` 的 `device_region_bytes` 行上不同（每侧 30 行：h1 的 8 行、h2 的 22 行；把这两类行去掉后其余逐字相同，`diff` 无输出），背景正文说那是系统配置两槽因 T_time 改毫秒而不同，不算这一批的出入（本腿没有解码那些字节，只核了「不同的只是这两个偏移」）。
- 导出的窗口里没有写清零，所以导出对拍盖不到 `WriteZeroes` 一支；它由 harness 的两条新单测（`recorded_operation_kind_names_are_the_four_literals_the_dumps_print`、`written_bytes_are_the_retained_content_the_zero_fill_by_length_and_none_for_a_barrier`）和四行变异钉（第 2705–2708 行，本腿复跑 4 行全红；`--lib` 84 条全绿）。

变异表：x2 删掉的两行在冻结副本里 `grep -c` 为 0（命令 2），没有留下找不到锚点的旧行（全表只有第 1354 行 litmus 文件在副本外）。

一处文档句与全仓不符：`RecordedOperationKind::name` 的文档写「全仓这一处定义，不各抄一份」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/lib.rs:44`），而冻结副本里还有手抄的种类名映射：`e142_new_pool_file_creation_write_dump_one_device.rs`（`fn kind_name` 在第 70 行、`fn operation_bytes` 在第 81 行）、`e170_o3_spec_executor_export.rs`、`e171_three_oracle_collusion.rs`、`e171_three_oracle_collusion_after_the_oracle_fixes.rs` 各一份（`grep -rnF '"write_zeroes"' --include=*.rs crates` 命中这四个 bin 加 harness 自己）。背景正文只把 `e142_new_pool_file_creation_write_dump_one_device.rs` 记进了下一批清单，e170、e171 两个（含 `after_the_oracle_fixes`）三份没在清单里；它们是反向解析也在手抄的那种（既有 `=> "write_zeroes"` 又有 `"write_zeroes" => …`）。这是文档句说大了或清单少了，不是等价性问题。

判定：一致（等价），一处文档句与全仓不符、清单少三份。推翻等价的现象：任一导出的 `kind=` 或字节与改前不同（上面两个导出对照没出现）。

## K1-N9　H3 与删除导出的参数

条款（E172 跑前登记第五节 5.3 H3 一行，`research/prompts/e172-preregistration.md:404`）：「同 H1，只改两处——① 顺序写之前在同一个实例里用 `publish_new_inodes` 建一个 size 0 的 inode（号 2）」「它与 1 号在同一片 inode 叶容器里」「意图那次的调用方时间 = 顺序写的时间 + 60 秒」「导出的 `impl_config` 另打 `intent_write_time_seconds=`」（以上四句均在 `research/prompts/e172-preregistration.md:404`）；第三段「要 H3 的导出先合入」（`research/prompts/e172-preregistration.md:422`）；真实基线「合入后主工作区导出的原样输出（H1、H2；第三段起 H3）」（`research/prompts/e172-preregistration.md:409`）。

逐步对：
| 登记 | 代码 | 实测 |
|---|---|---|
| ① 同一个实例里 `publish_new_inodes` 建 size 0 的 2 号，顺序写之前 | 「publish_new_inodes(」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:634`），实例 `INSTANCE_OF_THE_NEW_POOL_FILE_CREATION`，顺序写接在它的输出上：「&version_before_the_sequential_write,」（同文件 `:654`） | 用例断言 2 号 `(nlink, size) == (1, 0)`、`object_birth.0 + 2 == intent_txg`（建 2 号、顺序写、意图三次发布连着）：「"2 号建在顺序写之前：建 2 号、顺序写、意图三次发布连着"」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/tests/deletion_scenario_writes.rs:367`） |
| 同片叶容器 | 同一个 `publish_new_inodes`，2 号落最左那片 | 「"2 号与 1 号在同一片 inode 叶容器里"」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/tests/deletion_scenario_writes.rs:357`） |
| ② 意图时间 = 顺序写的时间 + 60 秒 | 「+ SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:674`），基数是顺序写写进 1 号记录的时间 | `run.intent_write_time_seconds` 为 1788000120，1 号意图之后与各批的记录时间同为 1788000120（不是顺序写的 1788000060） |
| `impl_config` 另打 `intent_write_time_seconds=` | 「intent_write_time_seconds={}」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:177`），只在 `SecondsAfterTheSequentialWrite` 支打 | `--history h3` 的 `impl_config` 行末尾是 `intent_write_time_seconds=1788000120`；不带参数那两行 `impl_config` 没有这一格 |
| 不带参数照旧只导 H1、H2 | 「HISTORIES_DUMPED_WITHOUT_ARGUMENTS.to_vec()」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:66`）；`EVERY_HISTORY` 是 3 个（「pub const EVERY_HISTORY: [DeletionHistory; 3] = [」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:306`），仓里 `EVERY_HISTORY` 只有这个 bin 用 | 不带参数 `exit=0`、10118080 字节、`impl_config` 两行（h1、h2）；`--history h1` 与不带参数里 h1 的那些行 sha256 同为 `edb210cb0109133b…`（取前 16 位），`--history h2` 同为 `980b37888c86eb2f…`；`--history h3` 150 行（149 行历史 + 1 行 `done`，与背景正文说的 149 行相同），3568590 字节，sha256 `9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e` |
| 命令行不对退 1、一行都不导 | 「[first_argument, ..] => Err(CommandLineRefusal::UnknownArgument(first_argument.clone())),」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs:99`）、「std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE)」（同文件 `:281`） | `--history h4`、`--history`、`--history h3 h1`、`h3`、`--history=h3`、`--history H3`：六种都 `exit=1`、stdout 0 字节，stderr 打 `命令行不对：<原因>（收到 …）；用法：不带参数（导 h1、h2）或 --history <h1 | h2 | h3>` |

登记没给、实现自己取的两处（规则没说，不算冲突）：2 号 inode 建出来的写入时间取 `FIXED_WRITE_TIME_SECONDS`（「pub const WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE: u64 =」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/scenario.rs:237`；注释自己写了「E172 跑前登记第五节 5.3 没给这个时间」）；`intent_write_time_seconds=` 放在 `impl_config` 行末尾。登记的 PC-A 说「被删记录偏移 36 起 nlink」，H3 的叶容器里有 1 号、2 号两条记录，比对侧要按号找，不能按「第一条」——这是实验侧的事。

钉它的测试与变异：第 2703、2704 行（H3 意图时间不加 60 秒；顺序写之前建 0 个 inode）本腿复跑都红。第 2711–2716 行（bin 里 6 条参数用例）在 checker 档包里，本腿**复核不了**测试本身；读代码，6 行改法各打在 `parse_dumped_histories` 的一支上，对应的 6 条用例各断言那一支的返回值，改坏必红；上面的 6 种命令行实跑给的是行为证据。

判定：一致。推翻它的现象：`--history h3` 的 149 行与 X 交回的那 149 行不是逐字节相同（本腿没有 X 的文件，只给出自己这份的 sha256 供主 agent 对），或不带参数的导出在 T_time 那两处之外还与入库那份不同（本腿核了：不同只在 `offset=0`、`offset=4096` 的行）。

## K3　12 份被判文件，每份一格

| # | 文件（冻结副本里的路径，在 `/tmp/claude-1000/m3-batch20-code-r1/tree/` 下） | 本批改了什么 | 判定与依据 |
|---|---|---|---|
| 1 | `crates/mutations.tsv` | 追加 35 行（a 17、g 4、x 8、x2 6），删 2 行（x2） | 一致。35 行锚点各命中 1 次、测试名都在（命令 2）；删的两行 `grep -c` 为 0；与改动全文逐字相同。本腿按点改坏复跑 23 行（第 2682–2698、2703–2708 行）全红；其余 12 行（g 的第 2699–2702 行要 GPU、e142 与 x2 的第 2709–2716 行在 checker 档包）复核不了。`AccountingTreeUnreadable` 没有变异行（N3）。 |
| 2 | `crates/singlefs-checker/src/image.rs` | 只改 86–87 行的注释 | 一致（N4）。`Judgements::into_report` 的次序（`:311`）是 N2 达成的依据，没改动。 |
| 3 | `crates/singlefs-checker/src/walk.rs` | 新变体 `InodeTreeNamedByNoSingleDeletedInodesEntry` 与它的理由串、`judge_deletion_queue_of_a_version` 末尾的反方向一遍、文档注释 | 一致（N1、N2）。三处改动都在 `:7942`、`:7964`、`:8442–8466`；反方向一遍只记判不了、不判红，放在正方向之后（注释 `:8442` 写明先记更近的理由）。`or_insert` 保证先记的理由留着（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/image.rs:284` 起），正方向配不上的版报的是头 ID 那句，不被反方向盖掉；探针 L1a 报的是新句，因为 L1a 里正方向一遍根本没有 `deleted_inodes` 条目可进。 |
| 4 | `crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs` | 两个挪出去的函数改调 harness、`--history` 参数、H3 的 `intent_write_time_seconds=` 格、6 条参数单测 | 一致（N8、N9）。不带参数与入库那份只差 `offset=0/4096` 的 30 行；`--history` 六种坏写法退 1；单测在 checker 档，复核不了。 |
| 5 | `crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs` | 文件头退出码、`EndingOfAnUnfinishedPath` 分流、`two_recording_memory_devices`、挪出去的函数改调 harness、单测 | 一致（N7、N8）。`--device-bytes 0` 退 1 带 mkfs 的原因；不带参数输出与改前逐字节相同。一处文件头没写的出口（超大盘字节数建盘就 134）不是这一批引入的。 |
| 6 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | 两个常量与找第一个文件那一段补读、判 nlink | 一致（N5）。逐支与 `recovery.rs:5069–5077` 同判同位，GPU 不跑，4 行变异复核不了。 |
| 7 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 比较函数拆出 `compare_on_every_state_of_these_segments`、删 `compare_the_checker_on_a_deletion_flow`、6 条换整份比、加只差一个写的用例 | 一致（N6）。整份比含恢复结局与判不了；104 个状态与今天的流是否对得上要在有卡的机器上看 `states_missing_one_write=`，复核不了。 |
| 8 | `crates/singlefs-harness/src/lib.rs` | `RecordedOperationKind::name`、`RetainedOperation::written_bytes_rebuilding_zero_fills`、`to_stream_line` 改调、两条单测 | 一致（N8）。`--lib` 84 条绿，4 行变异红。文档句「全仓这一处定义，不各抄一份」与全仓不符（见下「冲突与小出入」）。 |
| 9 | `crates/singlefs-harness/src/scenario.rs` | H3、`InodesBesideTheDeletedFile`、`DeletionHistoryIntentWriteTime`、`intent_write_time_seconds`、`SizeZeroInodeCreation`、`EVERY_HISTORY` 变 3 个 | 一致（N9）。H1、H2 的取法分支与改前同值（`TheFileRecordsCurrentWriteTime` 取原来的 `write_time_seconds`）；`deletion_scenario_writes` 5 条绿，2 行变异红。 |
| 10 | `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs` | 新搭建与坏镜像的造法（树表改种类 / 拿条目 / 改头 ID / 收窄、两份全零、墓碑未发布、nlink 回 1、size 0 一单元的池、按叶改的变体） | 一致（N3）。改成「按叶槽改」的三个旧函数（nlink 改 0、拿掉 1 号记录、墓碑对象 ID 抬到水位）行为不变（断言文字同，`unlink_single_file_publishes` 9 条、`container_trees_grow_a_level_when_the_root_is_full` 4 条、`inode_number_watermark_pressed_back_reissues_a_number` 1 条、`checker_known_bad_images` 89 条都绿）。 |
| 11 | `crates/singlefs-harness/tests/deletion_scenario_writes.rs` | 加 H3 一条用例 | 一致（N9）。5 条绿；H3 的形状与登记逐步对上（N9 表）。 |
| 12 | `crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs` | 加 16 条新用例（原 12 条，现 28 条）：L1a / L1b / L1c / L2 四条，七个旧变体各一条，size 0、墓碑未发布、中间版两条、C708 一形各一条 | 一致（N3）。28 条绿，17 行变异红；断言「只红目标」的与不断言的分开列在 N3。 |

## 冲突与小出入（两边整行并排）

没有一处是「代码做的不是条款、判决与用户定案说的」的硬冲突；下面五处是并排抄出来的出入，供主 agent 判要不要处理。

1. N8 文档句与全仓不符。
   - 一边：「全仓这一处定义，不各抄一份」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/src/lib.rs:44`）。
   - 另一边：「RecordedOperationKind::WriteZeroes => "write_zeroes",」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:888`）；同一行原样还在 `e171_three_oracle_collusion.rs:1538`、`e171_three_oracle_collusion_after_the_oracle_fixes.rs:2432`（都在 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/` 下）与「RecordedOperationKind::WriteZeroes => "write_zeroes",」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs:74`）。背景正文只把最后一份记进了下一批清单。
2. `AccountingTreeUnreadable` 无用例无变异。
   - 一边：判决第三节第 3 条要「零用例的七个至少各一条」，十个变体在代码里都有名：「DeletionScanGap::AccountingTreeUnreadable => {」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:7978`）。
   - 另一边：`grep -c 'AccountingTreeUnreadable' crates/mutations.tsv` 输出 `0`；`grep -rn '记账树读不全\|AccountingTreeUnreadable' --include=*.rs singlefs-harness singlefs-checker-tier` 零命中。它不在判决点名的七个里（第 18 批本腿报告说它由旧的条目宽用例「顺带钉」），所以不算这一批漏做。
3. N2 字面。背景表格「I-4.6 不适用的理由串换成配不上那一句」对 `walk.rs:8288` 的旧串（「"走读到的每一版 inode 树里都没有 nlink = 0 的记录",」，`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker/src/walk.rs:8288`）一个字没改；效果经 N1 的 `part_of_the_range_not_judged` 达成。
4. kb 状态列落后（等写回）。一边：I-9.5 行仍写「C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差」（`.claude/kb/invariants.md:292`）；另一边：那一形的用例今天在 `/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:847`。C712 行里「`compare_the_checker_on_a_deletion_flow` 的六处」的写法同理（C712 行的验收列，该行在 `.claude/kb/checks-owed.md:567`），函数今天已不存在。
5. 背景正文写「用户 2026-10-03 定 GPU 判器全覆盖」，代码注释写「用户 2026-10-04 定」（`/tmp/claude-1000/m3-batch20-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:71`）；差一天，不影响判定。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 反方向配对 | 一致 | 反方向恰好一条的数法、记判不了、报「不适用并带理由」与 F4-b、I-9.5 行、文件头四种都对上；L1a / L1b / L1c 的 28 条测试全绿、变异红 |
| K1-N2 I-4.6 理由 | 一致（经 N1） | 旧理由串字面没改，效果由判不了的优先次序达成 |
| K1-N3 未钉格 | 一致，一处小缺口 | 七个变体、size 0、墓碑未发布、中间版两条、C708 一形都有用例且 17 行变异本腿复跑全红；`AccountingTreeUnreadable` 仍无用例无变异；读不全型用例不断言红集（量过，红的是主走读那几条） |
| K1-N4 注释 | 一致 | 两处事实有旁证 |
| K1-N5 GPU 恢复 nlink | 一致（GPU 不跑） | 与 `recovery.rs:5069–5077` 逐支同判同位，同号多条两边都取后到的；4 行变异读代码成立、复核不了 |
| K1-N6 对拍覆盖 | 一致（GPU 复核不了） | 6 条换整份比，新用例每种切法恰好两个状态、取的正是「之前的段全落、这一段只差一个写」；104 个要提交时对 |
| K1-N7 e142 退出码 | 一致 | mkfs 拒的值退 1 带原因，实跑 6 种输入；巨大盘字节数建盘就 134 是文件头没写的出口，不是这一批引入的 |
| K1-N8 挪函数 | 一致（等价），文档句说大了 | 不带参数输出与改前逐字节相同；全仓还有 4 份手抄，清单只记了 1 份 |
| K1-N9 H3 与参数 | 一致 | H3 形状与登记逐步对上；不带参数只导 H1、H2，与入库那份只差两处偏移的 30 行；`--history` 六种坏写法退 1 |
| K3-1 `mutations.tsv` | 一致 | 35 行锚点 / 测试名 / 与改动全文都对；23 行复跑红，12 行复核不了 |
| K3-2 `image.rs` | 一致 | 只改注释 |
| K3-3 `walk.rs` | 一致 | 三处改动，反方向只记判不了 |
| K3-4 `deletion_write_dump.rs` | 一致 | 单测在 checker 档，复核不了 |
| K3-5 `e142_…_write_dump.rs` | 一致 | 同上 |
| K3-6 `crash_judge_common.wgsl` | 一致 | GPU 不跑 |
| K3-7 GPU 对拍测试 | 一致 | 同上 |
| K3-8 `harness/src/lib.rs` | 一致 | 文档句与全仓不符 |
| K3-9 `scenario.rs` | 一致 | H1、H2 不变，H3 对上登记 |
| K3-10 `common_deletion_in_progress/mod.rs` | 一致 | 5 个用到它的测试二进制全绿 |
| K3-11 `deletion_scenario_writes.rs` | 一致 | 5 条绿 |
| K3-12 `pool_checker_…rs` | 一致 | 28 条绿，17 行变异红 |

## 要主 agent 跑的命令（checker 档，子 agent 不许跑）

工作目录 `/home/fy5090/code/singlefs`，命令前缀 `SINGLEFS_HEAVY_TESTS=commit`，经 `bash research/scripts/run-with-memory-cap.sh <上限> bash research/scripts/capped.sh <线程数>` 起，上限与线程数由主 agent 按提交时的峰值表定：
1. GPU 对拍新用例：`cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress --nocapture`；看 `CRASH_JUDGE_GPU_MISSING_ONE_WRITE` 那行 `states_missing_one_write=104 failed=0`。
2. 6 条换整份比的旧用例：同一个 `--test`，后面分别点名 `the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress`、`…when_the_deletion_queue_names_an_inode_without_a_record`、`…when_the_pending_deletion_occupancy_row_is_not_zero`、`…when_the_pending_deletion_occupancy_row_is_missing`、`…when_the_size_of_an_inode_being_deleted_asks_for_more_data_units`、`…when_a_published_tombstone_reaches_the_inode_number_watermark`；每条看 `CRASH_JUDGE_GPU` 行 `mismatches=0`。
3. 两个 bin 的单测：`cargo test -p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump`、`cargo test -p singlefs-checker-tier --bin deletion_write_dump`。
4. 12 行复核不了的变异（第 2699–2702、2709–2716 行）：走门禁 checker-tier-crates-mutation-replay，或 `mutate.sh` 点名这 12 行。
5. 对 H3：本腿的 `--history h3` 输出 sha256 是 `9571c90498286a6ddfd4ac8718f3aedb8380132fa22496303342d58c0c09c45e`，主 agent 拿 X 交回的那 149 行 + `done` 行比。

## 没做什么

- 不判 K2（攻方腿的格），不替主 agent 采纳或出判决；「打中」「没打中」我都没有下，上面的「次要形」与「出入」只是读到的现象。
- GPU 不跑（按定义），checker 档包的任何测试与二进制测试不跑；N5 的 GPU 一侧、N6 的 104 个状态、第 2699–2702 与 2709–2716 行变异、两个 bin 的单测都是读代码或直接跑 bin 行为，不是测试本身的结果。
- 没有解码 `offset=0` / `offset=4096` 那 30 行的字节，只核了不同的行集合；没有拿到 X 的 H3 那 149 行（只给了自己的 sha256）。
- 没有在冻结副本上写任何东西；变异与探针都在草稿目录的副本上做，探针与复跑脚本放在 `research/prompts/m3-batch20-code-r1-sonnet-model/`（`mutation-rerun.py`、`probe_n1.rs`、`probe_n3.rs`、`probe_n1.out`、`probe_n3.out`、`mutation-results.txt`）；没有写 `rerun.sh`（本腿的模型目录只放读到的现象，不是攻方那种造反例的模型）。
- 草稿目录 `/tmp/claude-1000/m3-batch20-code-r1-sonnet/` 里的 `work/tree/`（冻结副本拷贝）与 `target/`（编译目录）交回前已删，日志与输出留在该目录里（`mut/`、`dump-*.out`、`dwd-*.out`、`probe_*`），不入库：都是复跑产物，要点可从上面命令重出。
