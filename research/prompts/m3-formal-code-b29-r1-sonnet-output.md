# m3-formal-code-b29-r1 云端正推腿报告（2026-10-05）

腿：云端正推（Sonnet）。批次：`records/2026-10-01-里程碑三形式化接入.md#29`。立场：从 b25、b26、c655-c687 三份判决、E171 登记修订 5 与 kb 条款推「改动该是什么」，与冻结副本逐格比。
冻结副本：`/tmp/claude-1000/m3-formal-code-b29-r1/tree/`。开工与收尾各核一次 `cd tree && sha256sum -c ../crates-sha256.txt | grep -vc ': OK'`，输出都是 `0`（不符 0 处）；我的变异只施加在草稿副本里，施加后都还原，收尾那次核的是草稿副本，同样 `0`。

## 0 做法与复跑入口

- 只跑 harness 档与池级 checker 库（经 harness 档测试二进制）：没跑 checker 档包的测试、GPU、变异整表。涉及这三样的结论写「推的」。
- 在草稿副本 `/tmp/claude-1000/m3-formal-code-b29-r1-sonnet/tree`（冻结副本 rsync 去掉 target，只多了 `crates/singlefs-harness/tests/b29_*.rs` 探针）里编、跑；每条 cargo 命令经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`（线程上限 10）。
- 模型目录 `research/prompts/m3-formal-code-b29-r1-sonnet-model/`：`tools/rerun.sh`（复跑入口，段名见文件头）、`tools/mutants.py`、`probes/`（5 份探针）、`data/mutants.tsv`（从改动全文抽出的 61 行变异行，第 1 列是序号，照各补丁追加块出现的次序从 0 数）、`data/extra.tsv`（我加的变体，零轮）、`out/`（每一段的原样输出）。
- 开跑前看了负载：别的会话的 `cargo test`、E171 驱动在跑，没有 `qemu-system`、`fio`、`vm-bench.sh`；包装排队等过内存（最长 85 秒），并行理由写「派出去的几个 agent 各跑各的」。
- 探针 `b29_l1_rollback_grid` 不带断言，`mutants.py` 对它报的 `SURVIVED` 没有意义，读格子里的 `CELL` 行（`NewFinding` = 抓到，`Completed` = 没抓到）。

## L1　第 27 批：b25 判决的改法与 E171 修订 5 装置

### L1-a　改的是不是判决与登记说的：一致

判决去向第 3 条：「第 27 批实现员：25-1 那一侧——T13 形状的回退用例与两行变异（drop-retain、drop-extend）、新用例常量改 2 × 233 + 4 与 KX1、KX2 两行变异、135 边界的纯函数用例」（`research/prompts/m3-formal-code-b25-r1-main-verification.md:54`） 逐件对到 diff：
- T13 用例 `a_rollback_past_an_inode_leaf_container_split_then_creating_inodes_is_judged_by_the_model_container_by_container`：unlink、建 468 个、回退到根环第 3 新（`index_from_newest: 2`）、再建 3 个，与判决（「加一条 T13 形状的 harness 档用例（U、建 468 个、回退、再建 3 个）与两行变异」（`research/prompts/m3-formal-code-b25-r1-main-verification.md:24`））同形；drop-retain、drop-extend 是 27-2 追加块第 0、1 行。
- 新用例常量 `2 * INODE_LEAF_RECORDS + 4`（470 步），改名 `model_compares_inode_creations_into_the_third_inode_leaf_container.rs`（27-2b 的 rename）；KX1、KX2 在 27-2b 替换块第 23、24 行（旧名那两行 2、3 被替换）。
- 135 边界：纯函数用例 `appending_inode_records_one_at_a_time_rewrites_container_floor_k_over_233_and_refuses_the_136th_container`（0.78 秒）与 model.rs 的 `the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond`；保留模型 `assert!`、不答拒，与判决（「保留 assert!、不答拒」（`research/prompts/m3-formal-code-b25-r1-main-verification.md:26`））同向。
- 装置照修订 5：十件逐件重做（`JudgingPiece::ALL` 十项）、k = 6（「const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8493`））、S1 新加三条、F9、no_root_write 在对拍之前判（「if let Some(shape_name) = shape_without_a_root_write(&shapes) {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8028`），次序：造主取样点那一形、造对拍那一形、先看两形的根写、再对拍），U1–U9 与 V20–V31 十二行变异都在（27-2 追加块第 6–17 行）。`unattributed_panic_states=` 照「主 agent 认定：27-2 装置交回」第 2 条，见 L1-e。
- 命令与输出：`python3 research/prompts/m3-formal-code-b29-r1-sonnet-model/tools/mutants.py anchors <data/mutants.tsv> <冻结树根> | tail -1` 输出 `NOT_ONE 0`（61 行原文在各自文件里都恰命中 1 次）；`python3 research/scripts/crates-mutation-rows.py static-check <冻结树根>` 末行 `CHECKED	2677	292	9`，只报 1 行 ANCHOR（变异表第 1354 行，`litmus/…litmus` 文件不在冻结副本里，与本轮无关）。
- 推翻条件：diff 里某件与上面对不上，或某行锚点命中数不是 1。

### L1-b　T13 回退用例与 drop-retain / drop-extend：罩得住点名的行为；换取样点各只罩一半

基线：27 批新增的三份用例（纯函数 135 边界、T13、第三片）在冻结副本上各 1 过（0.66 秒、12.64 秒、12.25 秒），同一趟里 lib 82 过 1 忽略、`checker_known_bad_images` 86 过、`mount_transparency_judged_field_by_field_after_a_writable_mount` 48 过、两份 `writable_mount_refusal_*` 1 与 5 过（`out/baseline-harness.txt`）。
点名的两行施加一次：行 0（drop-retain）、行 1（drop-extend）都 `CAUGHT failed=a_rollback_past_an_inode_leaf_container_split_then_creating_…`（`out/l1-mutant-results.tsv`）。
换取样点（探针 `probes/b29_l1_rollback_grid.rs`：建几个、回退到根环第几新、回退之后再建几个，每格一个线程跑一段历史）。基线 11 格全 `Completed`（`out/l1-grid-baseline.txt`）。施加变异之后：

| 格（建 : 回退到第几新 : 回退后再建） | drop-retain（行 100） | drop-extend（行 101） |
|---|---|---|
| 468:2:3（入库用例） | 抓到，红在回退那一步 `Operation(469)` | 抓到，同 |
| 468:1:3 | 没抓到（目标与现行同有 3 片，没有要丢的） | 抓到 |
| 468:3:3、468:6:3、468:2:0、468:2:1 | 抓到（回退那一步已红，回退之后不再建也红） | 抓到 |
| 236:3:3（目标没有右侧片） | 抓到 | 没抓到（没有要取回的） |
| 236:2:3、470:2:3 | 没抓到（目标与现行同片数） | 抓到 |
| 703:6:3（4 片的树） | 抓到 | 抓到 |

读法：两行变异各管一半（丢掉多余的片 / 取回目标的片），入库用例 468:2:3 恰好两半都罩；单看一行，要「现行比目标多片」（drop-retain）或「目标有右侧片」（drop-extend）的取样点。与判决 K1 第二行（「删回退那一支两行中的任一行」（`research/prompts/m3-formal-code-b25-r1-main-verification.md:24`））同义，没有冲突。原样输出 `out/l1-grid-baseline.txt`、`out/l1-grid-mutants.txt`（每行变异跑的格子清单略有不同，文件里 `## row` 之后的格就是那一行跑的）。
- 推翻条件：复跑里基线某格不 `Completed`，或变异行在 468:2:3 上不再红。

### L1-c　KX1 / KX2 与「第三片」：罩得住第 3 片，罩不住第 4 片及以后

行 23（KX1，`.min(1)`）、行 24（KX2，`.min(1)`）都 `CAUGHT failed=creating_inodes_one_at_a_time_past_the_first_leaf_container_…`；行 20–22 同样抓到。
我加的变体（`data/extra.tsv`，零轮）：行 104 把第 4 片及以后的胶水当第 3 片（`.min(2)`）、行 105 把模型落点封顶在第 3 片、行 102 retain 只丢左起第 1、2 片、行 103 extend 只取回左起第 1、2 片。在入库用例上四行都 `SURVIVED passed=1`。换到 4 片的取样点（705 个 inode）：行 106、107 在第 701 次建 inode 那一步红（`NewFinding(单元的分配代 @Operation(700))`），行 108（703:4:3、703:5:3）、行 109（703:3:3）在回退之后那一步红（`@Operation(704)`）。
结论：判决 K1 第三行要的「第 3 片」罩到了；「第 k 片都对」没有罩到——入库的 470 步与 T13 的 468 步最多触到第 3 片，封顶在第 3 片的错法活下来。这不违反判决字面（只要求 2 × 233 + 4），但「换取样点还红不红」的答案是：到第 4 片就不红。补法（零轮）：把新用例常量再往上一档（4 × 233 + 4 = 936 步），或另加一条建到 703 个的用例（探针里 703 步约 60 秒）。
- 推翻条件：入库用例里加到 4 片之后变体仍活下来，或变体在 703 步上也不红。

### L1-d　135 边界两条用例：罩得住，两个方向

行 4（放宽一片，第 136 片照收）、行 5（提前一条分裂）、行 18（model 的 `assert!` 放宽一片）都 `CAUGHT`。纯函数用例逐条追加 31 455 条、每条断言重写的是 ⌊k ÷ 233⌋ 片、第 31 456 条断言拒，任一方向挪一位都红；与模型那条合起来，两边的 135 是同一个 135。没有取样点问题（k 逐条全扫）。
- 推翻条件：某个把 `capacity` 或分裂点挪一位的变异不红。

### L1-e　逐件 catch_unwind、piece_panics、unattributed：没有静默漏算；七处语句在逐件重做之外；一处同因两记

读码（冻结副本）：正常那一路的 panic 只在 「let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:4341`） 一处接；接到之后进 `observation_of_a_panicking_state`（第 4066 行），调它的循环在第 4390 行，在 catch 块之外。
- 逐件重做里没包 `catch_unwind` 的语句：第 4075 行 `persisted_writes_of_state`、第 4081 行 `landing_vector_of`、第 4092 行 `segment_of_ordinal`、第 4164 行 `reference_filesystem_moment_of`、第 4170 行 `crash_judge_is_applicable`、第 4263 行 `sha256_text(crash_image_digest)`、第 4274 行起的 `RecoverySummary::of`。这几处自己 panic，异常从 `observe_sampled_states` 出去：dump 模式下被第 7503 行的 catch 接住、那条流记成 `pipeline_panicked:…` 的 `not_exported`、进 `streams_not_exported=`；Q5 的 `confirmed-loss-form` 两处直接调 `export_prepared_stream`（第 8174、8235 行），没有这层 catch，进程 panic。两种都是响的，不会有既不进逐件计数、也不进 unattributed 的静默态。
- 正常那一路 panic、逐件十件都没复现：`judging_path_panic` 有值而 `piece_panics` 空，进 `unattributed_panic_states`（「fn unattributed_panic_states_of<'observation>(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3675`））；`report_batch` 按批累加（第 7066 行），三批互不重叠（`observe_sampled_states` 里第 4313 行的 `assert!`），一个态只计一次。
- 同因两记：`newest_persisted_root` 在 `layer0_oracles` 件的闭包里（第 4123 行）与 `crash_judge_of_the_state` 里（「fn crash_judge_of_the_state(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3811`），函数体第 3817 行）各调一次；它若 panic，`layer0_oracles` 与 `crash_judge` 两列各记一次、`root_persisted` 不记。`piece_panic_states=` 是按件数的数、不是按态，读它的人要知道；不改判定。
- 判别力：行 19（`unattributed_panic_states_of` 恒 0）点名的用例 `a_judging_path_panic_that_no_piece_repeats_stops_the_q5_as_unattributed` 红（checker 档，没跑，推的）。
- 推翻条件：造一个正常那一路 panic 的态，逐件十件都没复现而 `unattributed_panic_states=` 不数它；或 `piece_panics` 非空的态被数进 unattributed。

### L1-f　`judge_the_confirmed_loss_shapes` 的四个调用点（重点）：改坏哪一个，都没有测试会红

主 agent 已知差别「27-2b 新用例罩不到四个调用点」，这里坐实、补形。四个调用点是 `stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed` 在 `judge_the_confirmed_loss_shapes`（第 8021 行起）里的四处：
- 对拍之后：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8080`）；阳性对照之后：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8113`）；反向取样点导出之后：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8187`）；主取样点导出之后：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8248`）。
- 命令：`grep -nF 'judge_the_confirmed_loss_shapes' <装置>` 的调用处只有两处：生产入口 `run_confirmed_loss_form` 与测试 「let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11837`）（U8）。`run_confirmed_loss_form` 自己只在 `main` 的分派里被调，没有测试调它。U8 两次都喂「没有根写」的形，在第 8028 行 `return 3`，走不到那四处。
- 另一个直接调 `stop_the_q5…` 的是单元用例 「let exit_code = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11948`）：它测的是函数本身，不测「哪个调用点调了它、调在什么位置」。
- 结论：删掉、挪位或改条件这四处中的任一处，没有一条测试会红（checker 档，没跑，读码加 grep 推的）。生产路径上它们只在真 panic 时才触发：`piece_panic_injection` 只在阳性对照那遍设值，主与反向取样点的 `context` 里是 `None`。
- 补法（零轮）：让 `judge_the_confirmed_loss_shapes` 的主、反向取样点吃一个注入入口，在 U8 同形的用例里喂 `piece: None` 的规则，四个出口各断言 `q5_not_run=unattributed_panic` 与退出码 3；变异行是删这四行 `if let` 各一行。
- 推翻条件：找到一条会调到这四处中任一处的测试。

## L2　第 28 批 CPU：b26 判决第 1 条

### L2-a　I-9.16 ④ 十一种：合法历史上不误红；坏盘上点名的两行抓得到；条款今天只有 ①②

- 判决：「④ 改成树表非空时十一种各恰好一条」（`research/prompts/m3-formal-code-b26-r1-main-verification.md:29`）。代码 「const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =」（`crates/singlefs-checker/src/walk.rs:3553`） 下一行取 `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`（种类 1、2、3、4、6、7、8、9、10、11、12 共十一种，中央映射树不进树表）。
- kb 今天没有 ③④：「两道任一道不成立即违例」（`.claude/kb/invariants.md:303`）（写回批没做；这两格的依据是判决，不是 kb 条款）。
- 合法历史：探针 `probes/b29_l2_checker_sweep.rs`（8 组命名比重 × 种子 1–12 与 30–53 × 6 步，共 288 段历史；mkfs 之后每段整段持久的前缀每隔 3 段、加每段内全枚举取样的每 150 个态跑一次 `check_pool_image`），共 8140 次：任何不变量都没有 `Violated`；I-9.16 8140 次 `Holds`；I-2.4 8140 次 `Holds`；I-5.4 6395 次 `Holds`、1745 次 `NotApplicable`、0 次 `PartiallyJudged`。种子 30–53 那 192 段带了操作种类计数：Unlink 46 次、CreateInode 36 次、RollBackWhileMounted 102 次、RaiseRollbackFloor 121 次、`OverwriteLosingPower…` 18 次、`CrashRecoveryAbandoningTheNewestRoot` 4 次（`out/l2-final-sweep.txt`，每个进程一段 `##`）。另三份合法大树镜像（探针 `b29_l2_big_tree.rs`：470 个 inode 三片叶、T13 回退之后、705 个 inode 四片叶）I-9.16、I-5.4、I-2.4 都 `Holds`，`violated=[]`（`out/l2-bigtree.txt`）。
- 「哪一版合法地少一种」：没找到。判法的依据是 core 重建一版要十一种各一条（判决 J1 C1 的分辨事实），所以读得出文件的版本都有十一种；0 条的树表（mkfs 第 0 版、零单元发布）④ 不判（代码：「if tree_table_entries.is_empty() {」（`crates/singlefs-checker/src/walk.rs:3564`））。
- 坏盘：行 25（④ 退回 1–4，缺 deadlist 放过）、行 26（缺 logged_ops 放过）都 `CAUGHT`；L4 里另有重复种类 ≥ 32 那一格的探针。
- GPU 一侧同一谓词：③ 两两比、④ 十一个位置各要一条（「// ④：树表非空时进树表的十一种（发号次序 1,2,3,4,6,7,8,9,10,11,12）各至少一条，与 ③ 合起来各恰好一条」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1696`）），与 CPU 逐位同形，见 L4。
- 推翻条件：任何合法历史上 I-9.16 判红；或有读得出文件的版本树表少于十一种。

### L2-b　I-5.4「成立」要至少一棵树走到叶：合法历史上不翻，坏盘上不放过，但「成立」这个标签比逐版判得宽

- 代码：「let some_allocation_record_tree_walked_to_its_leaves = judged_allocation_nodes > 0;」（`crates/singlefs-checker/src/walk.rs:4962`）；全候选走完没有一棵走到叶时 `not_applicable` 加 `partially_judged`（第 4973 行），违例当场照报。
- 合法历史：上面 8140 次与三份大树镜像里 I-5.4 从未 `PartiallyJudged`；`NotApplicable` 1745 次都没有伴随 `PartiallyJudged`：新代码只在「树表非空、全零格成立、没有一棵树走到叶」时才加 `partially_judged`，这样的态一个没出现，所以这 1745 次走的是原来就有的分支（推的，没拿旧代码逐次对）。
- 坏盘：探针 `b29_l2_i54_after_zeroing_each_unit_of_the_second_version`（`probes/b29_l4_append.rs`）：覆盖写之后的镜像上逐个把 26 个单元的两份副本清零，I-5.4 26 次都 `Holds`，其中清零发布 B 的单元（槽 50257–50264）时 B 的分配记录树读不出，别的不变量红（`["I-2.1","I-4.8","I-7.2","I-7.4"]`），I-5.4 仍 `Holds`（`out/l4-dup-kind-and-i54-units.txt` 里 `slot=50257` 到 `slot=50264` 各一行）。原因：别的候选根（发布 A）的分配记录树走到了叶，B 那版的「根记录里分配记录树根全零」那一格被计成「成立」。整体 checker 仍红（别的不变量），所以没有放过坏盘；但 I-5.4 自己对 B 版没判到而报成立，比判决 C2 想要的没判到不报成立宽一档。判决原话 「改法：带树一版分配记录树根全零那一判只在至少一棵分配记录树走到叶时计进 I-5.4 的评估」（`research/prompts/m3-formal-code-b26-r1-main-verification.md:30`），字面是「至少一棵」，所以这是实现符合判决，宽出来的那一档在判决自己的射程里。零轮补法：按版本记「这一版的树走到叶没有」，没走到的那一版的全零格进 `partially_judged` 清单。
- 推翻条件：合法历史上出现 I-5.4 `PartiallyJudged`；或判决 C2 的「至少一棵」改成逐版后我的探针在 B 版上仍报 `Holds`。

### L2-c　G1 运行配置字节取自条款：对、全

- 条款 「5 秒」（`.claude/kb/layout/01-first-txn.md:149`）、「0 = 内置默认」（`.claude/kb/layout/01-first-txn.md:151`），T_dirty 一行 「2 GiB」（`.claude/kb/layout/01-first-txn.md:150`）；harness 里三个常量 「const BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES: u32 = 5;」（`crates/singlefs-harness/src/mount_transparency.rs:1357`）、「const BUILT_IN_DIRTY_THRESHOLD_BYTES_OF_THE_CLAUSES: u64 = 2 * 1024 * 1024 * 1024;」（`crates/singlefs-harness/src/mount_transparency.rs:1360`）、「const BUILT_IN_COMPACTION_WATERMARKS_OF_THE_CLAUSES: [u64; 3] = [0, 0, 0];」（`crates/singlefs-harness/src/mount_transparency.rs:1363`），次序 4 + 8 + 8 × 3 = 36 字节，与 `433..469` 一致。
- 判别力：行 28（T_time 5 改 4）`CAUGHT`；我加的变体行 110（T_dirty 2 GiB 改 1 GiB）、行 111（水位默认 0 改 1）都 `CAUGHT`，新用例 `the_runtime_configuration_after_a_mount_alone_is_the_built_in_default_written_in_the_clauses` 在两次里都在失败名单里（`mut-110.log`、`mut-111.log` 各数到 1 次；这两种改坏各让该文件 48 条里 41 条红，所以不是只有新用例在守）。
- 推翻条件：任一常量与条款不同；或改其中一项而新用例不红。

### L2-d　journal_tail 两半：判得住；两半各有一条会红的用例

- 判法 「&& (!mount_completed || tail_after == highest_record_counter),」（`crates/singlefs-harness/src/mount_transparency.rs:2127`）（上一行 `tail_before <= tail_after && tail_after <= highest_record_counter`，第 2125 行），与判决 「R1 ≤ R2 ≤ R2 环里自证记录的最大计数器，挂载做完取等」（`research/prompts/m3-formal-c655-c687-r1-main-verification.md:32`） 同式。
- 行 29（去掉 R1 ≤ R2 那一半）、行 30（挂载做完不判取等）各 `CAUGHT`，红的是两半各自点名的用例（`out/l1-mutant-results.tsv`）；二次崩溃镜像上 `mount_completed` 为假、不判取等，与判决「挂载做完取等」只管做完的态一致。
- 范围限制（推的，没量）：`tail_of` 吃的是 R1、R2 各自择到的那一份系统配置（第 2097 行起，读 `chosen_system_configuration_bytes`），不是逐盘；另一块盘上落后的 tail 这一行判不到。判决 H6 取的是攻方的式子，没写逐盘。
- 推翻条件：合法历史上这一行红（判决量过 28 974 + 152 019 个态 0 次，我没复量）；或两半各去掉之后点名用例不红。

### L2-e　G3 判别子：判不出「这一拒是不是因为注入的读故障」，靠调用方声明；两处崩溃注入调用点的声明站得住

- 判官只吃入参 `ReadFaultsUnderTheRefusedMount`（第 2561 行起的 `can_hide_a_transient_read_error_from_the_crash_image`），崩溃注入第二、三截都交常量 「const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =」（`crates/singlefs-checker-tier/src/crash_injection.rs:2566`）（`TheRefusedMountReadTheCrashImageWithoutFaults`）。声明成立的依据：被拒的那次挂载在 「let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool」（`crates/singlefs-checker-tier/src/crash_injection.rs:1920`） 起的设备是不带故障层的 `SparseBlockDevice`，读到的就是镜像字节，瞬时读错不会发生。
- `TheRefusedMountReadThroughInjectedFaults` 没有生产调用点：`grep -rn ThroughInjectedFaults crates` 只命中 `mount_transparency.rs` 自己与一份 harness 用例；也就是说「注入态上合法」那一支今天只有用例走。
- 与 b26 判决 J2-a 同判（「两腿同判，坐实」（`research/prompts/m3-formal-code-b26-r1-main-verification.md:40`））。G3 把注入态上的新拒因从「复算为假也合法」收成「复算为假不合法」，会不会在合法历史上误红：探针（L3-a）在读故障一类历史上 14 640 + 6 558 个第一次崩溃态、42 619 + 900 个二次崩溃态里没有一次「某块盘一份自证过的槽都没有」，抽出的挂载 0 次被拒（`out/l3-readfault-8ops.txt`、`out/l3-final-6ops.txt`）。
- 行 31、32 各 `CAUGHT`。推翻条件：读故障一类历史上某个崩溃镜像上挂载以「读阶段判完之后系统配置又读不出」被拒而复算为假。

### L2-f　G5 不判注入态上的二次崩溃透明：与条款一致；放过的只有条款本来不管的那一类

- 代码 「violations.extend(match read_faults_in_the_first_crash_state {」（`crates/singlefs-checker-tier/src/crash_injection.rs:3435`）：注入过读故障的第一次崩溃态，它上面那次挂载途中的二次崩溃镜像不判挂载透明、记一次没判；别的判（`checker_violations_of`，第 3430 行；RefFS 票）照旧。
- 条款：「挂载透明的射程：只管不带故障、可写挂载做成的崩溃状态…带故障的状态不在射程：取号那一刻有一槽读不出与 N-配置今天报成同一个拒因，接进带读故障的档之前要分开」（`.claude/kb/decisions/13-验证路线.md:57`）。所以「一个只在注入态上露的坏实现」按条款不在挂载透明的射程；这样的坏实现仍会被池级 checker 与 RefFS 一票在那些镜像上判（它们照判），只有「挂载前后字段相等」这一类关系不判。
- 行 33（二次崩溃那一截不看读故障标记）是 checker 档 lib 用例，没跑，推的。
- 推翻条件：条款把挂载透明接进带读故障的档（C664 还清）之后，这一处要撤；或造出一个只在注入态二次崩溃上、且池级 checker 也看不见的挂载透明违例。

### L2-g　G6 的 C3 / B4 取样：只罩到「0 与 2 的幂」「一个写、1024 长」两个点，更长历史上罩不罩得住是「实现 OK、用例没罩」

行 34、35 在 checker 档 lib 里，没跑。读码：二次崩溃种子 = 历史种子 ^ 盐 ^ 段号 ^ 子集指纹（「^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),」（`crates/singlefs-checker-tier/src/crash_injection.rs:3181`） 上下四行，函数 `seed_of_the_second_crash_points` 第 3173 行）。
- 指纹对每个位逐位 `(f ^ bit) * 素数`，对长度没有上限，所以「更长段」这一轮实现不漏；用例 B4 钉的是 1024 长、扣一个写，`take(1024)` 这类只在更长段上才错的变体罩不到（推的，没跑）。
- 异或合成让不同的 (历史种子, 段号) 对撞：同一子集指纹下，(种子 4, 段 0) 与 (种子 0, 段 4) 的种子相同（`4 ^ 0 == 0 ^ 4`），两段历史的二次崩溃抽同一串随机数；「全落」这一个子集在每段都有，所以这样的对撞在实际历史上随处可能。C3 用例固定段号 16、只变历史种子，看不见种子与段号互相抵消这一类。零轮：种子与段号分开哈希而不是异或，或把「种子 ^ 段号」当成要核的撞对数写进用例。
- 推翻条件：给同一子集指纹的两个 (种子, 段号) 对，算出的种子不同。

## L3　第 29 批 G4

### L3-a　合法历史会不会让某块盘两个系统配置槽都读不出：没找到（约 130 万个态里 0 个）；探针对改坏的核心会红

- 机制：每次发布以根槽 FUA、轮换、屏障收尾（「writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {」（`crates/singlefs-core/src/transaction.rs:1386`） 前后各一步，第 1390 行是屏障）；轮换写 `slot_generation % 2` 那一槽（「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:542`）），世代号是同盘两槽自证过的最大者加 1，所以覆写的总是较旧那一槽；撕裂它，较新那一槽仍自证。崩溃注入的基线在 mkfs 之后：「/// mkfs 写完时录制流里有几步（崩溃注入的基线从这里起）。」（`crates/singlefs-harness/src/history.rs:1429`），mkfs 写到一半的盘面不进枚举。
- 探针 `probes/b29_l3_zero_slot_probe.rs`（harness 档）：随机历史 × 8 组命名比重，mkfs 之后每段「前面的段整段持久、本段每个写 落 / 不落，系统配置槽的写再加一个撕裂态（前半取新、后半取旧）」全枚举（段内超过 800 个态按种子抽 800），每态数每块盘自证过的系统配置槽；第二层：在抽出的第一次崩溃镜像上起一次可写挂载（录制），对挂载那几段同样枚举（二次崩溃）。
- 结果（`out/l3-final-6ops.txt`、`out/l3-readfault-8ops.txt`）：种子 0–23 × 8 组 × 6 步共 192 段历史，第一次崩溃态 992 567（历史里没有读故障步）+ 6 558（有）、二次崩溃态 161 208 + 900；另「读回全 0」那一组 16 段 8 步历史，第一次崩溃态 36 311 + 14 640、二次崩溃态 139 445 + 42 619。「某块盘 0 份自证过」的态数全是 0；只剩一份自证过的态有（第一次崩溃 9 565 + 60，二次崩溃 9 262 + 50），说明撕裂路径真走到了；抽出的挂载 0 次被拒。
- 阳性对照（探针要证明它会红）：在草稿副本 `tree-mut` 里把核心的系统配置轮换改成 MUT-B1 同形（新槽写完紧跟着把同盘另一槽清 0，同在一段里），同一探针数到第一次崩溃态 4 120 / 44 203、二次崩溃态 1 264 / 4 088 个「某块盘 0 份」（`out/l3-positive-control-mutant.txt`）。
- 题面点名的三类：撕裂写（只会撕较旧那一槽，见机制）、mkfs 清零（基线在 mkfs 之后）、回退（同一个生成器，L2 那批 288 段历史里 `RollBackWhileMounted` 102 次、`RaiseRollbackFloor` 121 次，抬 F 先写系统配置的那一步也在枚举里）。
- 没覆盖：两块盘之外的池、别的盘宽（只用了 `UnitAreaOf384Slots`）、超过 8 步的历史、段内超过 800 个态的段（抽样）。
- 判定：没找到。读法上「没有读故障的历史留不下这份盘面」是实现的性质，这一格的结论是「在这些取样点上成立」。推翻条件：任何一段历史的任何一个态上某块盘 0 份自证过，且那段历史里没有读故障步。

### L3-b　第二、三截判法与 C670 同不同形：同接法，三处不同形

- 同：第二截与第三截都经同一个判官（第二截 「match judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount(」（`crates/singlefs-checker-tier/src/crash_injection.rs:2615`），第三截 `reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults`，两条件各记一次判过、一次不合法，`criterion_true_without_read_faults.then_some(…)` 只在判据复算为真时才报红）；都只在 `NoneInjected` 判（G4 在 「if system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)」（`crates/singlefs-harness/src/mount_transparency.rs:2693`） 的分支，`Injected` 在第 2702 行的臂里不判）。
- 不同形 ①：择不到系统配置时 C670 的复算交 `None`（不报）——第 2478 行起；G4 的复算按真——第 2350 行起。镜像上一份系统配置都择不到时读阶段本身就拒了，不会走到这一拒，所以这一处不影响判定，但两个复算在同一个空集上答案相反。
- 不同形 ②：C670 的条件带「哪一支为真」（`WitnessedCounterBranchThatHolds`），G4 的是无载荷成员（第 2579 行）。
- 不同形 ③：G4 的复算是「某块盘一份自证过的都没有」或「见证判不出」，只读两槽字节；C670 要走 journal 记录（`scan_journal`）。读故障没有时「见证判不出」恒假（`PoolReader::read` 在内存盘上不返回 `None`），所以 G4 在无故障态上只剩前一条。
- 行 44–46 各 `CAUGHT`（44、45 是同一个变异写了两行，点名的两条用例都在失败名单里）。

### L3-c　手工两槽损坏翻成不合法、注入口径下合法的一条，是不是都从条款推得出：一半推得出

- 注入口径下合法：条款 「带故障的状态不在射程」（`.claude/kb/decisions/13-验证路线.md:57`） 推得出（G4 在 `Injected` 不判，只剩 U6 四条，四条都成立 ⇒ 合法）。
- 手工两槽损坏翻成不合法：条款本身推不出。同一行（「拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里」（`.claude/kb/decisions/13-验证路线.md:57`））写的合法条件是这四条，手工两槽损坏上四条都成立，按这句它合法。翻成不合法靠的是 C670 的论证类推——「没有任何读故障的崩溃态，不许让 D23（journal 的角色与格式） 已定项 14 可写挂载那条见证判据为真」（`.claude/kb/checks-owed.md:528`）（只点名「见证判据」）——与判决 「改法 G4：没注入读故障的崩溃态上新拒因复算为真」（`research/prompts/m3-formal-code-b26-r1-main-verification.md:41`）（零轮）。
- 条款层的缺口：不变量清单里只有池级的 「池里至少有一块盘的两个系统配置槽里有一个槽自证过」（`.claude/kb/invariants.md:94`），没有「每块盘在每个没有读故障的崩溃态上至少一份自证过」这一条；G4 实际是在 harness 里加了一个没有 kb 行的性质判官。L3-a 的探针支持这个性质，但写回批要给它一行条款（或把它并进 C670），否则判官的依据只有判决。
- 推翻条件：D23 已定项 14 或 D18 已定项 11 里有一句允许无读故障的崩溃态留下「某块盘一份自证过的都没有」；或 L3-a 的探针找到这样的态。

### L3-d　E171 装置补的那一臂：编得过、没有用例点它

`refusal_condition_text` 里加的一臂（「"system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3439`））只是 `match` 要穷举；它的唯一调用处是装置的探针（`judge_writable_mount_refusal_of_a_crash_state`，不带故障层的封装），装置里只有一条用例读这类词（`witness_criterion_true_without_read_faults`），没有用例让新臂出现。影响只在 `second_crop` 一列的文字。推翻条件：有用例要新臂的词。

## L4　GPU：26-5 与 29-2

没跑 GPU（重型）。GPU 一侧都是读码，写「推的」；CPU 一侧在 harness 档实测。

### L4-a　逐格同形：读码上没有发现新的不同形，除 L4-b 那一处

| 格 | CPU（池级 checker / core） | GPU | 读码结论 |
|---|---|---|---|
| I-9.16 ③ ④ ② | `judge_tree_table_entries_ordering` 第 3491 行、`judge_tree_table_entry_kind_counts` 第 3559 行 | `w_judge_tree_table_entries_ordering` 第 1673 行 | 条目宽 < 12 只判 ①、0 条不判 ③④、③ 任一种类码重复、④ 十一种各一条、再 ②，违例位同谓词；种类位置 `tree_kind_in_the_issuing_order` 给 1–4 与 6–12，与 CPU 十一种常量一致 |
| I-5.4 根记录里的分配记录树根 | 第 4913 行：树表读得出且条目非空、根记录偏移 342 不全零 ⇒ 违例 | 扫描内核第 918 行，同条件 | 同；GPU 只有违例位，CPU 的「成立 / 不适用 / 部分没判」三态 GPU 不分，对拍比的是违例位 |
| I-2.4 根槽 428..457 | 第 8789 行，对 `valid_roots` 每条 | 第 4244 行，对 `checker_collect_roots`（第 269 行）每条 | 两边的「根」同口径：magic、整槽校验和、fsid、flags，都不按候选集滤；GPU 多一个 `MAX_ROOTS = 48` 的容量界（越过判不了） |
| I-2.4 extent 偏移，池级走读 | 四处：第 1271、2228、2569、2730 行 | 四处：第 1482、1584、1769、2595 行 | 逐处对应；CPU 只判 MAC 的那一处（`walk.rs` 容器树内部条目子指针，第 2064 行）GPU 同样只判头（第 2928 行） |
| I-2.4 extent 偏移，恢复 | core 十处读指针都判：根记录四条（`root_record.rs` 第 146–153 行）、树表条目、inode 内部条目与 extent 记录（`records.rs` 第 449、503、523 行）、extent 树三处（`extent_tree.rs` 第 363、370、735 行）、分配记录树（`allocation_record_tree.rs` 第 777 行）、码 2 树（`code_two_tree.rs` 第 154 行）、实例表链（`recovery.rs` 第 2200 行经 `parse_with_chain_pointer_head_fields_judgement`） | GPU 恢复内核对应：择根（941）、树表条目（`walk_failed(63u)`）、inode 内部条目（`walk_failed(64u)`）、extent 上段叶两处、下段叶、extent 内部条目、分配记录树内部条目、码 2 树内部条目、实例表链、容器树内部条目（2860） | 站点逐一对上（`pointer_extent_offset_is_zero` 在 `crash_judge_common.wgsl` 定义之外 7 处调用，另有四处读 `Ptr.extent_offset_zero`）；加密 / 压缩几段的差别是已知的 |

- 附带（CPU 与 GPU 同形，不是 GPU 独有）：容器树（墓碑、谱系）层级 1 条目的子指针，core 恢复会判 extent 偏移而拒（`recovery.rs` 第 3946 行起），池级 checker 与 GPU 走读内核都只判 MAC / nonce（`walk.rs` 第 2064 行、`crash_judge_checker.wgsl` 第 2928 行）；这一类指针上 extent 偏移非 0 的盘面 checker 不红、core 拒。读码推的，没造镜像验。

### L4-b　找到一个 CPU 红、GPU 恢复内核绿的盘面：树表里两条同一种类，种类码 ≥ 32

- core：「let mut kinds_seen: BTreeSet<u16> = BTreeSet::new();」（`crates/singlefs-core/src/recovery.rs:2330`）、「if !kinds_seen.insert(entry.kind) {」（`crates/singlefs-core/src/recovery.rs:2335`），任何 `u16` 种类重复都拒这张树表。
- GPU 恢复内核：「if (kind < 32u) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2947`） 只记 32 个以内的种类，注释自己写明 「// 与前一条比种类：种类 ≥ 32 的两条同种拦不到，走读同样拦不到（kinds 是 u16，但一条流写不出这种树）」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2953`）；这一段在 26-5、29-2 里没动，对拍新加的 ③ 用例只造种类 7 的重复（< 32）。池级 checker 的 ③ 两边都比整个 `u16`，所以走读那一半 CPU 与 GPU 同红，不同的只有恢复那一半。
- CPU 一侧实测（探针 `probes/b29_l4_append.rs` 的 `b29_l4_duplicate_kind_probe`，追加在 `checker_known_bad_images.rs` 的拷贝后面）：把新池新建文件那一版树表里 deadlist、logged_ops 两条的种类都改成 K、重封到根：

```
L4PROBE kind7 violated=["I-9.16"] i916=Some("Violated(\"根（实例 1、txg 3）的树表：种类 7 有 3 条（③ 每种至多一条）\")") root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: UnitMalformed { what: "树表里同一种树有两条" } }
L4PROBE kind32 violated=["I-9.16"] i916=Some("Violated(\"根（实例 1、txg 3）的树表：种类 32 有 2 条（③ 每种至多一条）\")") root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: UnitMalformed { what: "树表里同一种树有两条" } }
L4PROBE kind40 violated=["I-9.16"] i916=Some("Violated(\"根（实例 1、txg 3）的树表：种类 40 有 2 条（③ 每种至多一条）\")") root=Some((InstanceGeneration(1), CheckpointTxg(3))) outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(3))), failure: UnitMalformed { what: "树表里同一种树有两条" } }
```

  K = 7、13、31、32、40 五个值同样（`out/l4-dup-kind-and-i54-units.txt`，干净镜像上 `outcome=FileRead`）：池级 checker 红 I-9.16，core 只读恢复在这个根上 `UnitMalformed`；而 GPU 恢复内核对 K ≥ 32 不拦，会照常读到文件（读码推的）。K ≥ 32 的重复不是任何正确实现写得出的盘面（注释也这么说），所以这是造出来的坏盘才露的差别，不是合法历史上的误判。
- 零轮补法：恢复内核对种类 ≥ 32 的条目逐条与前面各条比（树表至多几十条，量级与 ③ 在走读内核里的两两比相同）；对拍加一条 K = 40 的重复用例（照 `record_new_pool_with_a_tree_table_entry_kind_rewritten` 的造法，deadlist 与 sparse side table 两条都改成 40；这两条的根指针本来全零，与那个助手函数里的断言兼容）。
- 推翻条件：GPU 恢复内核在 K = 40 的盘面上与 CPU 一样拒（对拍这一条绿）。

### L4-c　「只违反 ③ 的表超额整态交 CPU」：交不回去，整趟停下

- 主 agent 登记的是 「只违反 ③ 的表超额整态交 CPU」（`records/2026-10-01-里程碑三形式化接入.md:1031`）。崩溃放量里 GPU 判不了时的实际处置是 「配置开了 GPU，不退回 CPU 判器：照这两个字查是哪张表满了、改内核的容量再跑」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2009`）：整趟停下、报错。同一文件第 2021 行的 `expect` 说明走到解码时不会再有判不了的态。`crash_judge_gpu.rs` 文件头那句 「//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:6`） 只是注释，放量路径不照它做。
- 扫描内核一张树表只记 11 棵（「if (tree_count >= C_MAX_TREES_PER_TABLE) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:526`） 与 「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:527`），上限 「const C_MAX_TREES_PER_TABLE: u32 = 11u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:382`））。只违反 ③ 不违反 ④ 的表至少 12 条（十一种各一条再多一条重复），所以第 12 棵起这一态判不了。
- 对拍用例那一侧也不交回 CPU：文件头写明 「一个状态都不许判不了」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5`）。
- 结论：「去掉 ③ 是等价变异」成立的前提是「GPU 判得了的态上」；对只违反 ③ 的表，今天的结局是放量停下报错、对拍用例红，不是交回 CPU 判。对「GPU 一个态都不许判不了」的对拍来说，这是响的而不是静默的，但登记里的整态交 CPU 不对，该改成整趟停下。
- 推翻条件：放量路径在判不了的态上改成回落 CPU（`crash_amplification.rs` 里 `不退回 CPU 判器` 那几处被改）。

### L4-d　新加六条与改的那一条：各自坏掉 GPU 哪一处会红（读码，没跑）

| 用例（后缀） | 点名它的变异行 | 坏掉的 GPU 处 | 备注 |
|---|---|---|---|
| `…tree_table_carries_two_entries_of_one_kind` | 无（扫描内核容量，等价变异，见 L4-c） | 走读内核 ③ | 种类 7 的重复 |
| `…tree_table_lacks_the_deadlist_kind` | 行 40 | 走读内核 ④ 循环退回 4 | 需 28-1 的 CPU ④ 才绿 |
| `…root_record_carries_a_non_zero_mac_byte` | 行 41 | 根槽字节数 29 → 28 | |
| `…also_holds_an_allocation_record_tree_in_its_root_record` | 行 42 | 扫描内核全零格 | |
| `…code_two_node_pointer_carries_an_extent_offset`（29-2 改成两半都比） | 行 43（后被 60 替换）、51 | 走读内核读码 2 节点；恢复内核树表条目 | 两行共用一条用例，各自坏一半，用例都红 |
| `…instance_table_pointer_of_a_root_record_carries_an_extent_offset` | 行 50、57 | 恢复内核择根；走读内核实例表链 | 同上，两行共用 |
| inline / inode 内部条目 / 分配记录树 / 下段根 / 下段叶 五条 | 行 52–56、58、59 | 各自一处 | 一行一处；行 54 与 58（上段叶内联）、行 56 与 59（inode 内部条目）各点同一条用例 |

- 没有变异行的三处（主 agent 已记）：码 2 树内部子指针（`crash_judge_common.wgsl` 第 1939 行）、extent 内部子指针（第 2271 行 `extent_internal_child_pointers_carry_no_extent_offset`）、实例表链（第 2706 行）。现有流里造不出多层树，与行 52–56 的取样点同一个问题；零轮：让一条流长出多层 extent 树（单文件写满 144 个数据单元以上）或多层分配记录树再加三条。
- 缺的一条：L4-b 的 K ≥ 32 重复。
- 推翻条件：GPU 对拍里某条用例在对应变异下仍绿。

## L5　主 agent 的两处

### L5-a　#9 两个钉值的出处：值与日志逐字相同、出自「第 28 批 + 10e」那棵树；冻结副本上仍成立是读码推的

- 命令与输出（日志在 `/tmp/claude-1000/-home-fy5090-code-singlefs/8adc3bc8-1d0b-43bd-9a3f-9e63dae04aaf/scratchpad/`，`n9-step1.log`、`n9-step2.log`）：

```
$ grep -n 'left: (58' n9-step1.log ; grep -n 'left: ("layer0-progress' n9-step2.log
75:  left: (58, "e1ab3f797f369a2e9f3e45195331d252306b41346bef10922c7e04e1336a8cc4")
84:  left: ("layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt", "2a77c2d767d5dadc12e55e06860327826fb9c95c04870fe15ab5cb1241bf26bc")
$ grep -n 'Compiling singlefs-checker-tier' n9-step1.log n9-step2.log
n9-step1.log:3:   Compiling singlefs-checker-tier v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-checker-tier)
n9-step2.log:23:   Compiling singlefs-checker-tier v0.1.0 (/tmp/claude-1000/m3-r1/n9-copy/crates/singlefs-checker-tier)
$ ls /tmp/claude-1000/m3-r1/n9-copy
ls: cannot access '/tmp/claude-1000/m3-r1/n9-copy': No such file or directory
```

- 冻结副本里的钉：「"e1ab3f797f369a2e9f3e45195331d252306b41346bef10922c7e04e1336a8cc4";」（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:845`）、「"2a77c2d767d5dadc12e55e06860327826fb9c95c04870fe15ab5cb1241bf26bc";」（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:852`）、「const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt";」（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:850`） 的名字里计划哈希 `0370ad26…`，三处与两份日志的 `left` 逐字相同；旧值 `7e64a883…`、`15867e36…` 只剩两行注释里的出处说明（第 840、841 行），没有残留的断言值。
- 取自哪棵树：第一趟编的是主工作区路径，第二趟编的是副本 `n9-copy`（已删，`ls` 报 `No such file or directory`，我复核不了）。「同一棵」只有记录里的指纹为证：「树（指纹 bdf78215…）上取」（`records/2026-10-01-里程碑三形式化接入.md:1048`）、「树，指纹 bdf78215…f8cb」（`records/2026-09-30-里程碑三开工第一轮.md:640`）；主工作区之后又并进了 29、第 19 批与主 agent 的两处，所以这两趟的树不能再从现在的主工作区去对。
- 与冻结副本的差（冻结副本取在第 19 批之后、10f 之前：「删除线第 19 批之后、根环 10f 之前取」（`records/2026-10-01-里程碑三形式化接入.md:1059`））：29-1 动 `mount_transparency.rs`、`crash_injection.rs`、E171 装置、一份 harness 用例；29-2 动两份 `.wgsl` 与对拍用例；主 agent 两处动 `crash_amplification.rs` 的 import 与两个 cfg。这些都不在那条用例的输入里（它打的行来自层 0 对新池新建文件流的枚举：池级 checker 判定集合、核心发布写出的字节、`crash.rs`、`layer0_progress.rs`，这几处 29 与主 agent 两处一个字没动）。删除线第 19 批动 `transaction.rs` 的 C479 只供测试的水位开关（「只供测试压回去的 inode 号水位」（`crates/singlefs-core/src/transaction.rs:209`），读开关处在第 5886、8007 行，建 inode 与记账水位），新池新建文件的发布不走那两处、开关默认不装。提交前还会并进 10f（`mount.rs`），新池新建文件流是 mkfs、取号、暖机、发布，没有重挂。
- 判定：值与出处一致；冻结副本上仍成立，是读码推的，没复跑（checker 档）。提交时那一趟 `checker-tier-layer0-replay` 才是判官。推翻条件：那一趟断言的 `left` 不是 `e1ab3f79…`（58 行）或 `2a77c2d7…`。

### L5-b　cfg 改动：两个特性都开时，被改的两处照样编进来；单特性组合我没复核

- 改动后的两处：「#[cfg(feature = "verdict-store")]」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1025`）（第 1025 行，`sigma_publish_writes`，原来是 `any(verdict-store, gpu)`）与 「#[cfg(all(feature = "gpu", feature = "verdict-store"))]」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1060`）（第 1060 行，GPU 的 σ 对拍，原来是 `gpu`）。两个特性都开时：`any(verdict-store, gpu)` 与 `verdict-store` 都为真，`gpu` 与 `all(gpu, verdict-store)` 都为真，编进来的集合没变；`sigma_publish_writes` 的两个调用处（第 1072、1239 行，一个在 GPU 用例、一个在 verdict-store 用例）都在作用域里。
- 门禁 feature-gated-tests 格的特性清单从源码 cfg 现读：「特性从 checker 档 tests/ 顶层每个 .rs 的 cfg 现读，不另抄名单」（`.claude/gate.d/checker-tier-layer0-replay.sh:49`）；这份文件里别处早就有 `cfg(feature = "gpu")` 与 `cfg(feature = "verdict-store")` 两种单特性写法，新加的 `all(…)` 不改并集 `verdict-store,gpu`。
- 「照样跑」要分两种机器：有 NVIDIA 卡才真对拍；没有卡时这条用例按文档（「本机没有 NVIDIA 卡时打一行」（`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs:1058`），下一行接 `NOT_RUN_NO_GPU_ADAPTER` 报本次未跑、返回成功）编进来、报本次未跑，不是通过。这与改之前相同。
- 没复核：单开 gpu、单开 verdict-store 编不编得过、clippy 是否 0（要现编 wgpu、RocksDB）；`BlockVerifier::Never` 的 `expect(dead_code)`（「#[expect(dead_code, reason = "没编进 gpu 时只替生命周期参数占位，从不建出")]」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2387`））在哪些组合下会兑现，也靠主 agent 报的四种组合 clippy 都退 0。
- 推翻条件：提交时两个特性都开的那一趟里，`record_checker_judges_absence_by_the_persisted_set` 这份文件的用例数比改之前少。

## 判定一览

判定词：一致 = 改动做的与判决 / 条款 / 登记说的相同；冲突 = 两边不同，并排写在对应小节；规则没说 = 判决与条款都没定。

| 格 | 判定 | 一句话 |
|---|---|---|
| L1-a 改法对判决与登记 | 一致 | 判决去向第 3 条各件、修订 5 第 5.1–5.3 节各件都在；61 行变异锚点各命中 1 次 |
| L1-b T13 用例与两行变异 | 一致 | 两行各 `CAUGHT`；换取样点各罩一半（468:1:3 只有 drop-extend 抓，236:3:3 只有 drop-retain 抓），入库用例两半都罩 |
| L1-c KX1 / KX2 / 第三片 | 一致，有取样点缺口 | 第 3 片罩到；`.min(2)` 类变体与 retain / extend 只处理前两片在入库用例上活下来，703 步上抓到 |
| L1-d 135 边界两条用例 | 一致 | 三行 `CAUGHT`，k 逐条全扫 |
| L1-e 逐件 / unattributed 记账 | 一致 | 没有静默漏算（逐件之外 7 处语句 panic 时 dump 模式记 `not_exported`、Q5 模式进程 panic）；同一次 panic 在 `layer0_oracles` 与 `crash_judge` 两列各记一次 |
| L1-f 四个调用点 | 冲突（坐实已知差别） | 删 / 挪 / 改四个 `if let` 任一个，没有测试会红；生产路径上只有真 panic 才触发 |
| L2-a I-9.16 ④ 十一种 | 一致；kb 没有 ③④ | 288 段历史 8140 次 `check_pool_image` 0 误红；条款今天只写 ①②，依据是判决 |
| L2-b I-5.4 要至少一棵树走到叶 | 一致，标签宽一档 | 合法历史上不翻；B 版分配记录树读不出而 A 版读得出时 I-5.4 仍 `Holds`（别的不变量红） |
| L2-c G1 字节取自条款 | 一致 | 三个常量与 kb 三行相同，三类改坏（T_time、T_dirty、水位）各被抓 |
| L2-d journal_tail 两半 | 一致 | 两行变异各抓；判的是择到的那一份，不是逐盘 |
| L2-e G3 判别子 | 一致 | 判官靠调用方声明，崩溃注入两处声明站得住；读故障类历史 0 次被拒 |
| L2-f G5 | 一致 | 注入态上二次崩溃不判挂载透明，与 D13 射程句同向；别的判照旧 |
| L2-g G6 的 C3 / B4 | 规则没说（推的） | 用例只罩两个点；种子与段号异或可对撞；checker 档没跑 |
| L3-a 合法历史上误红 | 一致（没找到） | 约 130 万态里「某盘 0 份自证过的槽」0 个；改坏的核心上同一探针数到 4 120 / 44 203 |
| L3-b 与 C670 同不同形 | 一致，三处不同形 | 同一接法；择不到系统配置时答案相反、条件无载荷、复算只读两槽 |
| L3-c 两条用例推不推得出 | 冲突（一半） | 注入口径下合法推得出；手工两槽损坏翻成不合法靠 C670 类推与判决 J2-b，kb 没有条款 |
| L3-d E171 装置一臂 | 一致 | 只是 `match` 穷举；没有用例点它 |
| L4-a CPU / GPU 逐格同形 | 一致（读码） | 站点逐处对上；唯一例外见 L4-b |
| L4-b CPU 红 GPU 绿的盘面 | 冲突 | 树表里两条同种类、种类码 ≥ 32：CPU 红且 core 恢复拒，GPU 恢复内核 `kind < 32` 才记；GPU 半边读码推的 |
| L4-c ③ 交回 CPU | 冲突 | 放量路径判不了就整趟停下报错，不退回 CPU；登记里的整态交 CPU 不对 |
| L4-d 新加用例各抓哪一处 | 一致（读码，没跑） | 点名对应成立；三处无变异行已知，缺一条 K ≥ 32 用例 |
| L5-a #9 钉值出处 | 一致（读码推的） | 值与日志逐字相同、出自「28 + 10e」树；副本 `n9-copy` 已删，同一棵靠记录指纹；冻结树上仍成立没复跑 |
| L5-b cfg 改动 | 一致（两特性都开） | 编进来的集合不变；单特性组合没复核；没卡时是「本次未跑」不是通过 |

## 没做什么

- 没跑 checker 档任何测试（E171 装置内联用例 U1–U9 与一条 unattributed 用例、变异行 6–17、19、33–35、37–39、47、49、`crash_enumeration_sharded_across_processes`）、GPU 对拍（变异行 40–43、50–60，L4 全部 GPU 半边）、`crates/mutations.tsv` 整表：都归提交时。上面涉及它们的结论都标了推的。
- 没复编单开 gpu、单开 verdict-store 两种组合（要现编 wgpu、RocksDB）。
- 没复核 `n9-copy` 与 pin 树是同一棵（副本已删，指纹算法我没拿到）；没复跑 #9 那一趟。
- 我加的变体行（`data/extra.tsv`）、各探针与它们量到的数都是副本上的数，零轮；不进 kb，要引得由主 agent 在入库装置上重做一次。
- 没替主 agent 采纳或出判决；没改仓内任何文件，只写了本报告与模型目录。
- 草稿处置：`/tmp/claude-1000/m3-formal-code-b29-r1-sonnet/` 下我建的工作树与编译目录（`tree`、`tree-mut`、`rerun-smoke`、`target`、`target-mut`）已删；留下 `out/`（探针与变异原样输出）、`report/`（本报告的模板与渲染脚本）、`model/`（探针与脚本的原件），都是小文件，入库那份在模型目录。

## 复跑

```
REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b29-r1/tree WORK=<会话草稿>/b29-sonnet-rerun \
  bash research/prompts/m3-formal-code-b29-r1-sonnet-model/tools/rerun.sh all
```

脚本里每条 cargo 自己经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`；整条脚本若要由外面一层包装起，设 `RERUN_ALREADY_WRAPPED=1`。可单跑的段：`setup anchors baseline l3 l2 l2-bigtree l1-mutants l1-extra l1-grid l4 l3-positive`；推的挂钟（我这台上）：setup 约 1 分钟、baseline 1.5、l3 4.5、l2 3、l2-bigtree 1.2、l1-mutants 12、l1-extra 3、l1-grid 6、l4 1、l3-positive 3。`anchors` 退 1 是因为 `static-check` 报那 1 行 litmus 锚点，不是红。

引文核对（本报告）：

```
$ python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b29-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b29-r1/tree --root . --background research/prompts/_m3-formal-code-b29-r1-background.md
  ✓ 核了 70 处引文，对上 70 处，没判 0 处（逐处列在上面）
```
