# 第 25 批代码轮第一轮（m3-formal-code-b25-r1）· 云端正推腿报告

腿：云端正推（Sonnet）。格：K1、K2、K3。方案按 `crates/` 今天的实现来谈：代码引文读冻结副本 `scripts/experiments/target/e171-r1-frozen-tree/`（只读，没在里面编、没往里写），行号是那棵树里的；kb 与登记引文写它们自己的行号。
模型与脚本：`research/prompts/m3-formal-code-b25-r1-sonnet-model/`（`tools/rerun.sh`；`outputs/` 是这一轮真跑出来的原样输出；`probes/` 是我自造的探针源码与对装置的补丁）。草稿副本在 `/tmp/claude-1000/m3-formal-code-b25-r1-sonnet/`（见末尾「草稿副本」一节）。

## 0. 怎么做的（短）

- 读：背景材料第 1–182 行（正文、小节清单、E171 页开篇引文）与它的小节标题表（附录里其余 kb 抄录没读，需要的条款我直接去 kb 文件里读：D8 已定项 6、D13 已定项 3 与 6），`.claude/agent-common.md`、`three-way-inference.md`、`implementation-first.md`、`evidence-discipline.md`、`mutation-sampling.md`、`verification.md` 相关节，登记 `research/prompts/e171-r1-prereg.md` 第 681–869 行、diff 全文的 K1 与 K3 相关段、装置 7700–8700 与 10040–10345 行、core 的 `inode_tree.rs`、harness 的 `model.rs` 与 `model_comparison.rs` 相关段。没读 `research/results/e171-r1-*2026-10-05*`（禁读清单）；没读别轮腿报告。
- 跑：harness 档测试（新用例、我自造的探针、`--lib`、`random_histories`、`unlink_in_histories_judged_by_the_model`）、装置二进制的小规模模式（`stream-constants`、`confirmed-loss-form`，冻结二进制与我加了探针的副本二进制）。全部经 `capped.sh 6` 与 `run-with-memory-cap.sh 8G`。
- 没跑：checker 档内联单测、`crates/mutations.tsv` 整表、门禁阶段。涉及它们的结论写「推的」。
- 过程里的两件事：①开跑前 `ps` 看到别的会话的 `cargo clippy`、`cargo check` 与 librocksdb 的 C++ 编译在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，所以照常跑（加了 `nice -n 19`）。②第一次起 `run-with-memory-cap.sh` 退 251（「这个会话已经有一条日常的在跑」），之后每条命令设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（理由：同会话里正推腿与攻方腿各跑各的）；峰值表超 2 倍那一道设了 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`。没有 250–254 的结局。
- 本会话没有 Write 与 Edit 工具，报告与脚本都用 Bash 排他新建后追加。

## K1　25-1：O1 跨叶容器

### K1.a 模型「第 k 条落第 ⌊k ÷ 233⌋ 片」与实现切叶容器的规则是不是同一个规则

判定：**一致**（在模型罩的那一种历史上：删完第一个文件之后，树空，只经建 inode 一次一个往右追加）。

依据（实现一侧，`inode_tree.rs`）：追加只看最右那片有没有空位——「let rightmost = rightmost_container_with_room(&containers);」（`crates/singlefs-core/src/inode_tree.rs:232`）；有空位就往里推：「containers[index.position()].records.push(*record);」（`crates/singlefs-core/src/inode_tree.rs:235`）；没空位（树空、或最右那片满 233）就新建一片：空位判据是「if last.free_record_slots() == 0 {」（`crates/singlefs-core/src/inode_tree.rs:347`）；片数上限是「if containers.len() > capacity {」（`crates/singlefs-core/src/inode_tree.rs:259`），容量取「index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))」（`crates/singlefs-core/src/inode_tree.rs:110`）。

依据（模型一侧，`model.rs`）：「let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;」（`crates/singlefs-harness/src/model.rs:197`）；上限是「position_from_the_leftmost < leaf_containers_one_root_holds,」（`crates/singlefs-harness/src/model.rs:201`）。

条款（D8 已定项 6）：「⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**」（`.claude/kb/decisions/08-核心索引结构.md:171`）。两边都是这句的直接推论：k 条记录时满 233 的左片不动、第 k 条落最右那片。

我量的（原样输出在 `outputs/k1-boundary-core.txt`、`outputs/k1-boundary-model.txt`）：
- 实现的纯函数 `write_records_into_leaf_containers` 一次追加一条，逐条断言「重写清单恰是 [⌊k ÷ 233⌋]」，跑了 31455 条全对，第 31456 条（`records_before = 31455`）拒：
  `PROBE core refusal at k=31455 (records_before) -> MoreLeafContainersThanOneRootNodeHolds { containers: 136, capacity: 135 }`
  `PROBE core checked=31455 containers=135`
- 模型的式子：`PROBE model capacity = 135`、`PROBE model at records_before=233*135-1 -> Ok(134)`、`PROBE model at records_before=233*135 panics=true`。两边的边界在同一个 `records_before = 31455`。

什么现象会推翻：实现把某一次追加落在不是最右的那片、或满 233 之前就分裂（任一都会让「重写的角色集合」与模型差一个角色，新用例会红）；或条款 D8 已定项 6 改成别的分裂点（取末尾以外）。

### K1.b 删除、续做、回退、多实例几种历史上还对不对得上

**删除之后条数变少**——判定：**规则没说**（在被罩范围之外，不是对不上）。实现对删除是另一条规则：删空一片就不落盘、右边每一片位置左移一格并全部重写——「containers.remove(index.position());」（`crates/singlefs-core/src/inode_tree.rs:308`）。这与模型的「第 k 条落第 ⌊k ÷ 233⌋ 片」不是同一个规则。历史生成器不产生这种历史：建 inode 只在第一个文件已删完的版本上调——「只在第一个文件已删完的一版上调（有树而文件集合为空的一版、或在它上面建过 inode 的一版）」（`crates/singlefs-harness/src/history.rs:496`）；之后再 Unlink 是「不适用」而不是做成：探针 `unlink_after_split` 的 `by_kind` 里 `Unlink: OperationTally { applied: 1, refused: 0, not_applicable: 1 }`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k1-probe-remount-sequences.txt` 里现取）。模型把条数记成 `inode_records_in_the_tree`，只在建 inode 时加一（「let inode_records_before = current」（`crates/singlefs-harness/src/model.rs:2159`））。
推翻条件：执行器哪天允许在建过 inode 的版本上 unlink 或删单个 inode——那时模型的条数与片位置都要跟着重写，否则会误红。

**续做（挂载、冷启动恢复插进建 inode 的序列）**——判定：**一致**。我自造了 5 段序列（`probes/k1_probe.rs`），每段跑完 `ending=Completed`（原样在 `outputs/k1-probe-remount-sequences.txt`）：①第 233 条之后挂载、再建 3 条（挂载那次写行把两片叶都比一遍）；②第 235 条之后挂载再建；③第 235 条之后冷启动恢复、再挂载、再建；④第 232 条之后挂载、建 2 条、再挂载、再建 1 条（两次挂载夹着 233→234 的边界）；⑤第 234 条之后再 unlink（不适用）。

**回退**——判定：**规则没说**（这一支在模型罩的历史里不可达）。模型的回退答先看现行版有没有文件——「.is_a_version_with_a_file_or_a_file_being_deleted()」（`crates/singlefs-harness/src/model.rs:2380`），没有就返回「模型里现行版本没有文件」的 `NotModeled` 分歧；而建过 inode 的版本都是「有树而文件集合为空」，第一个文件删完之后再没有路径回到有文件的版本（`answer_publish_first_file` 要求树还没建起来，覆盖写与 unlink 对这种版本拒或 `SessionState` 分歧）。所以「建到第二片之后再回退」的目标版本只可能是第一个文件还活着时的版本，那一版没有右侧叶容器；25-1 加的那段「现行有、目标没有的右侧叶容器这一版也没有」——「role_written_at.retain(|role, _| {」（`crates/singlefs-harness/src/model.rs:1746`）——在可达历史上从不执行。同一处还有「contents.clone_from(&target_root.contents);」（`crates/singlefs-harness/src/model.rs:1725`）把条数一并换成目标版的，这一步对可达历史是对的（目标版条数为 0）。
我量的：自造变异 KX4（把那段 `retain` 删掉）在新用例上不红（`outputs/k1-mutations-extra.txt`：`RESULT name=KX4 … outcome=not_caught`）。这与实现员自报的「有代码没测试」一致，而且是「加不出会红的用例」（模型 API 在无文件现行版上就拒），不是「没想到加」。
推翻条件：执行器或模型改成允许在无文件的现行版上回退（`answer_rollback_while_mounted` 的前提放开）——那一支马上变成可达，需要立刻补用例。

**多实例**——判定：**一致**。上面 ①④ 里每次挂载换一个实例代号，写行那次发布把两片叶容器各比一遍；实例表行也在比（`instance_tables_compared` 非 0，见 `research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k1-probe-remount-model-counts.txt`）。没有跨实例回退到旧实例根的历史（那要 `CrashRecoveryAbandoningTheNewestRoot` 接在有文件的版本上），不在这条规则的射程里。

### K1.c 胶水映射：放过 / 误红

判定：**被罩的历史上一致；另有三个没被新用例罩到的取样点**。

- 胶水一行：「ModelUnitRole::inode_leaf_container_at(u64::from(index.0))」（`crates/singlefs-harness/src/model_comparison.rs:80`）。对比是两个方向、逐角色：实现交回的每个角色这一版模型要有（「let Some(written_at) = expected.role_written_at.get(role) else {」，`crates/singlefs-harness/src/model.rs:3240`），模型这一版的每个角色实现要交回（「let handed_in: BTreeSet<ModelUnitRole> =」，`crates/singlefs-harness/src/model.rs:3222`），再逐片比分配代。所以实现把某次追加落错片、或多重写一片、或边界差一条，都会在「角色集合」或「分配代」上红（我的 KX3、KX5 与新增三行变异都这么红，见 K1.e）。没找到合法历史误红：K1.b 的五段序列、新用例、470 步的三片变体（`outputs/k1-k1_three_leaves.txt`）全绿。
- **没罩到 ①：第 3 片及以后。** 新用例只建到第 237 条——「const INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER: u64 = INODE_LEAF_RECORDS + 4;」（`crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs:16`），位置只到 1。KX1（胶水把位置封顶在 1）与 KX2（模型把落点封顶在 1）在新用例上都绿，把常量换成 `2 * INODE_LEAF_RECORDS + 4`（`probes/k1_three_leaves.rs`，470 步，约 11 秒）后两条都红（`outputs/k1-mutations-extra.txt` 后半）。这是 `.claude/rules/mutation-sampling.md` 那张表第三行说的：「补一个敏感的取样点，不是留档」（`.claude/rules/mutation-sampling.md:17`）——处置是补一个敏感取样点（把常量加到 2 × 233 + 4）。
- **没罩到 ②：占槽跨度。** KX7（右侧叶容器占槽记 16 KiB）绿；对照 KX8（最左叶容器占槽记 16 KiB）同样绿——所以这条用例对「占槽上界」这一维整个不敏感，不是 25-1 新出的洞；那一维归墙区间那几条用例（推的，没逐条跑）。
- **没罩到 ③：135 片边界。** KX6（assert 放宽一片）绿，要 31455 次建造才碰得到。

### K1.d 实现员自报两处，该怎么处置

**超过 135 片（建满 31455 个）：模型 `assert!` panic、实现拒。** 我的判断：**不让模型答「拒」；保留 `assert!`；加一条便宜的边界用例；记账。**
依据：①条款没有「拒」这个行为——D8 已定项 6 写的是「码 2 根装满时长一层」，并写明「根长高之前今天的代码在 `crates/singlefs-core/src/inode_tree.rs` 的 `MoreLeafContainersThanOneRootNodeHolds` 拒」（`.claude/kb/decisions/08-核心索引结构.md:155`）；实现的拒是还没做长高的缺口，不是被条款规定的行为。模型答「拒」等于把缺口钉成规范，将来实现做了长高，这条模型答反而误红；`assert!` 按「模型今天只罩……」的写法与 `model_comparison.rs` 里别的不罩的几种单元同风格（例如「TransactionUnit::ExtentUpperNodeBelowTheRoot(position) =>」，`crates/singlefs-harness/src/model_comparison.rs:73`）。②实现的拒在对拍里会被映成「说不出理由的拒」——「| PublishError::InodeTreeWriteRefused(_)」（`crates/singlefs-harness/src/model_comparison.rs:685`）归 `Unexplained`；历史执行器先调实现、后取模型答——「&IdealModel::answer_create_inode,」（`crates/singlefs-harness/src/history.rs:3834`），所以第 31456 次建 inode 时实现返回拒、模型随后在 `assert!` 处 panic，是 harness 级的失败，不会被记成一次对得上的拒（推的，没跑到那一步）。③便宜的边界用例已经量过：纯函数两侧各一格，0.65 秒（`outputs/k1-boundary-core.txt`、`outputs/k1-boundary-model.txt`），能钉住「两边的 135 是同一个 135、边界在同一个 `records_before`」。整段 31455 步历史我没跑（单步约 60 毫秒，debug 下约半小时，推的）。
记账：条款没有长高之后的叶容器位置规则（新根的 key 区间与出生序号），模型没法罩；记进 checks-owed 并指到 D8 已定项 6 那句。
推翻条件：D8 已定项 6 把「根装满的拒」改成条款行为；或实现做了长高。

**建到第二片之后再回退那一支有代码没测试。** 我的判断：**不加整历史用例（加不出）；记账；另加一条钉前提的小用例，或删掉那段代码。**
依据：见 K1.b「回退」——模型 API 在无文件现行版上拒绝回退，那一支只能经一次改了前提的改动变活，而今天它不会被执行；给它写用例得先改执行器，那是另一件事。钉前提的做法：用例断言 `answer_rollback_while_mounted` 在建过 inode 的现行版上返回 `NotModeled`——前提一变，用例先红，提醒补那一支。不留死代码是另一条路：删掉那段（`crates/singlefs-harness/src/model.rs:1745` 一带），等回退放开再按用例写回。两条路二选一由主 agent 定；我倾向钉前提，因为 KX4 证明删了也没人发现，而钉前提的用例才是会红的东西。

### K1.e 3 行变异各只红点名那条吗

判定：**是**。三行（`crates/mutations.tsv` 的「O1 跨叶容器（理想模型自身）：第二片晚一条才分出」在第 2478 行、「O1 跨叶容器（胶水）：实现交回的每片叶容器都当成最左那片」在第 2479 行、「O1 跨叶容器（理想模型自身）：新记录落在第二片时最左那片也记成这次重写」在第 2480 行——引文：「O1 跨叶容器（胶水）：实现交回的每片叶容器都当成最左那片」（`crates/mutations.tsv:2479`））在我的草稿副本上逐行施加、跑点名的测试（`outputs/k1-mutations-3-rows.txt`）：三行都 `outcome=caught failed=1`，且红在第 235 步（`Operation(234)`，`AllocationGeneration` 分歧；第一行的原样失败消息在 `research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k1-mutation-row1-failure.log`，片段：`position: Operation(234), operation_kind: Some(CreateInode)`）。
「只红点名那条」我另核了一遍别的 harness 档测试：`--lib`、`random_histories`、`unlink_in_histories_judged_by_the_model` 共 110 条（`--no-fail-fast`）下，第二、三行零红；第一行只红我自己追加的边界探针（它断言 233 条落第 1 片），仓里原有的 109 条全绿（`outputs/k1-mutations-collateral.txt`）。别的 140 多个 harness 测试二进制我没逐个跑（推的：这三处改动只在 `records_before ≥ 233` 或位置 ≥ 1 时才与原式不同，随机历史建不到那么多个 inode）。
推翻条件：有一条别的 harness 用例在这三行下红了（说明点名之外有人盯着）；或点名的用例在原样代码下不绿（我在原样副本上跑它是绿的：`outputs/k1-model_compares_inode_creations_across_two_inode_leaf_containers.txt`）。

## K2　E171 装置：Q5 候选二

### K2.a 144 个等价类的分法站不站得住（同类态上 O1 两个崩溃子列、RefFS 一票、恢复报告相同吗）

判定：**一致**（没找到同类而判定不同的一对）。结构论证、读数与自证三样：

- 结构：登记的论据是「只要恢复在代表上读到的字节区间与 C 的 24 个写的字节区间不相交」（`research/prompts/e171-r1-prereg.md:713`）恢复在同一 K 取法下读到的每个字节都相同。装置的实现：分法按「写属于第几次调用」分——「fn before_any_refinement(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7848`）；类核对 ③ 在 72 个代表上经只转发、只记的读者跑恢复（「impl PoolReader for ReadRecordingPoolReader」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7967`）；四个 `PoolReader` 方法它全转发，trait 只有这四个），读段与类里每个写求交：「if recorded_reads_overlap_the_write(&read_ranges, write) {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8145`）；journal 位置提示记成整个环：「end_in_bytes: ring_start.0 + ring_bytes,」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7999`）。C 的 24 个写是第 3 次调用的数据与树单元（没有哪条根指着它们），所以代表上的恢复一个读都碰不到它们：`refinements=0`，`read_overlaps_left=0`。恢复是读到的字节的确定函数，读不到 C 的字节，C 落没落恢复就走同一条读轨迹。
- 读数（冻结二进制 `confirmed-loss-form` 主取样点，原样在 `outputs/k2-main-point-frozen-bin.stdout`）：类核对一行全 0、`s13=false`，对拍 288 态 `mismatching_states=0`。我另在主取样点的真流（C 24 个写）上，每种 K 取法各取「C 单个落」24 个、「C 缺一个」24 个、随机子集 20 个，共 4896 个附加态，照原路径判，与类里推出的恢复、O1 两个崩溃子列、RefFS 一票逐态比：`mismatching_vs_class_columns=0`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-main-stream-class-probe.stderr:5`，原样：`PROBE main_stream_extra_states=4896 mismatching_vs_class_columns=0 (o1_or_ref=0) classes_seen=144 classes_with_different_o2=0 classes_with_different_layer0=0 o2_values={"Some([])"}`）。层 0 判定在同类里也全相同。
- 自证会红（我的探针能分辨）：把探针里的类键抹掉一位（`PROBE_BREAK_KEY=1`），同一批态立刻报出不同：`PROBE main_stream_extra_states=3600 mismatching_vs_class_columns=936 …`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-main-stream-class-probe-break-key.stderr:5`）。
- 这一格的 Q5 读数本身（不是我判的）：`lost_the_confirmed_version=150994944 reference_filesystem_red=150994944 model_crash_judge_red=0`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-main-point-frozen-bin.stdout:203`）。

什么现象会推翻：任一类里有两个态，O1 两列或 RefFS 或恢复落点不同（我的探针 5040 个态里没有；装置自己对拍 288 态里也没有）；或 ③ 在某个代表上漏记了一次读（例如恢复走了不经 `read` 与 `journal_record_offsets_hint` 的读法——今天 trait 只有这两个读法）。我只量了 5040 个态，不是 2²⁴ 里的全部，所以这是「没打中」，不是证明；结构论证是它的依据。

### K2.b 「三条 oracle 的判定是不是真的相同」

判定：**规则没说**（登记没主张 O2、O3 在同类里相同）。类等价只主张四样：O1 两个崩溃子列、RefFS 一票、恢复报告、层 0 判定的取值被代表推出；登记自己写明 O2、O3 只在判过的态上算：「O2、O3 附带只在判过的态上」（`research/prompts/e171-r1-prereg.md:747`）。`confirmed-loss-form` 里每个判过的态才有 O2 红名单，没判的态没有。
我量的：这条流上 O2（池级 checker）在我看过的全部 5040 个态上都是 `Some([])`（没红任何不变量；`outputs/k2-main-stream-class-probe.stderr:3`：`PROBE o2_distinct_values_over_all_states={"Some([])"}`）。所以「同类里 O2 相同」在这条流上成立，但**它对 C 的取值不敏感**——O2 在这里一个态都没红，看不出有没有差别。O3 在 `confirmed-loss-form` 里不跑（只对导出的 144 个态跑），没量。
推翻条件：同类里找到 O2 红名单不同的两个态（要先有一个让 O2 红的 C 取法，这条流上我没找到）。

### K2.c 288 态交叉核对与类核对 ①–④ 分不分得出一个分错的类；V17–V19 之外还有没有放过的

判定：**有放过**（告警没有被证明会红）。这是规则缺口，不是我造出了一个具体的分错类——**我没造出**：把根槽写移进 C 会被 ③ 的读段重叠自动拉回键里（自愈），把 C 的种类改成系统配置槽写会被 ② 报（探针 D2：`PROBE_UNIT D2 forbidden_kind=1 refinements=1 mismatching_states=0 stops_at_s13=true`，`outputs/k2-unit-logic-replica-baseline.txt:4`）。
事实并排：规则：「往被测对象里塞一个已知的、本该被第二条路子抓到的故障，看它是不是真抓到了。」（`.claude/singlefs-ai-sop/rules/evidence-discipline.md:15`，在「校验路径本身也要证明它会红」那一节）。登记给的变异只有三行：「V17：键去掉」（`research/prompts/e171-r1-prereg.md:724`，后接「C 至少落一个」那一位）、「V18：类核对 ③ 恒判不相交。」（`research/prompts/e171-r1-prereg.md:725`）、「那一类的态数写成 2²⁴。」（`research/prompts/e171-r1-prereg.md:726`，V19）。①、②、④ 两半、`class_landing_mismatches`、S13 的拼法、journal 位置提示的整环记法，没有一条变异行盯着。
我量的（装置二进制副本里把每一处告警改成不计数，跑「内联用例口径的复刻」——`probes/e171_device_probe.patch` 里的 `probe_unit_logic`，复刻 288 态那一形与手造 journal 写那一形，**不是内联单测本身**）：KM1（② 不计数）、KM2（① 不计数）、KM3（④ 后一半不计数）、KM4（④ 前一半不计数）、KM5（代表落地串核对恒过）、KM6（S13 不含 `class_landing_mismatches`）、KM7（提示不记整环）七条。对复刻内联用例的那几段（A、B、D、C 四段读数），七条的输出与未变异逐行相同；唯一的差是 KM1 在我另加的 D2 流上（`forbidden_kind=0 … stops_at_s13=false`）。原样：`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-km1-km4-unit-logic.txt`、`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-km5-km7-unit-logic.txt`、`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-km6-km7-unit-logic-retry.txt`、`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-km6-unit-logic-retry2.txt`（KM6、KM7 在先前几次跑里撞了包装自己的结局 252「内存排不上」，那一行不算结果，在小一点的上限下重跑的读数是 `stdout_same_as_unmutated=True differing_lines=0`）。含义：没有任何一条现有用例或变异行会因为这七处之一被关掉而红。KM1 与 D2 合起来说明 ② 这条告警本身是活的，只是没人让它红过。
另一件：V17、V18 在装置自己的跑法里读不出来——对主取样点，施加 V17 或 V18 后标准输出与未变异逐行相同（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-v17-v19-runtime-main-point.txt:1`：`stdout_same_as_unmutated=True differing_lines=0`；V18 同——那份文件里 V18 一行是包装自己的结局 `exit=252`，不算结果，重跑见 `research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-v18-runtime-main-point-retry.txt:1`，`exit=0 stdout_same_as_unmutated=True`），V19 则红（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-v17-v19-runtime-main-point.txt:4`：`s1 … confirmed_loss_states_of_a_class_with_in_flight_writes_landed local=16777215 crates=16777216 holds=false`，`exit=3`）。原因：真跑里模型从不停在第 3 次调用（对拍行 `crash_judge_not_applicable_states=0`，见 `outputs/k2-main-point-frozen-bin.stdout:16`），「C 落没落」这一位在所有态上同值；V18 要 C 的某个写落在恢复读到的位置，真流里没有。这两条只靠 checker 档内联单测盯着（装置的「fn equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10073`）带「模型停在第 3 次调用」第二遍，「fn an_in_flight_write_over_a_journal_record_of_the_confirmed_call_is_moved_into_the_key() {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10146`）手造重叠）；那是重型测试，我没跑，所以 V17、V18 会不会红是**推的**。我的复刻里同样的两段取样点读数对得上内联用例的断言口径：`not_applicable_states=216 (unit test expects 216)`、`moved_positions=[5] (expects [5])`（`outputs/k2-unit-logic-replica-baseline.txt:2`、`:5`）。
一处措辞：V17 的变异文本改的是 ④ 里推导触达调用时用的那一位（「let in_flight_writes_landed_in_the_class = InFlightWritesLanded::AtLeastOneLanded;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8408`）），不是 `class_of_ordinal` 里的键；名字写「键去掉那一位」略宽。真正去掉键里那一位会被 `class_landing_mismatches` 在任何跑法下抓（「checks.class_landing_mismatches += 1;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8439`），推的，没量）。
288 态对拍只罩 C 的前 2 个写：对拍流是「截到第 3 次调用发出的前 2 个写之后」——「ConfirmedLossTruncation::AfterTheFirstInFlightWrites(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7334`）。主取样点上 ④ 后一半只核「C 全落」一态——「if ClassColumns::of_the_observation(fully_landed).as_ref() != Some(&derived) {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8441`）。C 的后 22 个写靠 ③ 的结构论证，不靠对拍。
推翻条件：给 KM1–KM7 里任一条写出会红的用例或变异行后，真有一个分错的类被它抓到而 V17–V19 与现有用例全放过。

### K2.d S13 与 `class_landing_mismatches` 的计法

判定：**计法一致；登记对 `class_landing_mismatches` 的释义与代码不同（小冲突）；q5_equivalence 行的字段与「主 agent 认定」第 3 条不同（小冲突）。**

计法（代码）：S13 = 停在态数上限、或细分后仍有读段相交、或 ①、②、④ 前后两半、落地串五个计数任一非 0——「|| checks.class_landing_mismatches > 0」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8477`），函数头「fn equivalence_stops_at_s13(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8467`）。`class_landing_mismatches` 每个代表与每个「C 全落」态各至多加一次（两处，各一句「checks.class_landing_mismatches += 1;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8382`），另一处在 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8439`），判的是「那一态的落地串不是分法说的那样」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8276`，注释接着列：代表上类里的写不全是 0、C 全落上不全是持久、或代表不在它那一类）。对拍的 S13 另含「各类态数之和不等于组合数」——「|| self.class_states != self.product_of_the_choices」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8540`）与「对拍有一态不同」；主取样点那一支只用 `equivalence_stops_at_s13`——「let stops = equivalence_stops_at_s13(&classification, &evaluation.checks);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7531`），各类态数之和由 `q5` 行的 `a10_states_equal_the_product` 报、比对器再核（登记 721 行），不进 S13。登记对 S13 的定义没把态数之和写进去：「停机 S13（新设）：类核对 ①–④ 任一不过」（`research/prompts/e171-r1-prereg.md:727`），所以主取样点不含它与登记一致。对拍不过退 3 且不跑主取样点：「if pairing.stops_at_s13() {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7354`），与「主 agent 认定」第 5 条一致。

小冲突 ①（释义）：两边并排——登记：「实现员自加的 `class_landing_mismatches`（每类的恢复落点与代表不同的态数）算进 S13」（`research/prompts/e171-r1-prereg.md:859`）；代码：「那一态的落地串不是分法说的那样」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8276`）。登记说的是「恢复落点」，代码数的是「落地串」（哪些写落了），不是恢复落到哪一版。恢复落点与代表不同由 ④ 后一半（`derived_class_mismatches`，`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8441`）盯。两个都进 S13，所以判定不受影响，但登记的释义会让读的人以为这一项在比恢复结果。
小冲突 ②（行的分工）：登记：「细分次数等分法类的量另起一行 `name=q5_partition`，不进 S13」（`research/prompts/e171-r1-prereg.md:859`）。代码：`name=q5_equivalence` 行的字段仍含 `refinements`、`moved_last_segment_positions`、`stopped_at_the_state_limit`——「refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={} s13={}」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8487`），而 `name=q5_partition` 行的字段是写数、种类、键与类的位置、要判的态数、上限：「in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={}」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8513`）。原样输出：`E7RESULT name=q5_equivalence sampling_point=main_before_the_first_barrier_of_call_3 … refinements=0 moved_last_segment_positions=[] stopped_at_the_state_limit=false s13=false`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-main-point-frozen-bin.stdout:57`）。登记 727 行的触发观测写「`name=q5_equivalence` 那一行任一计数不是 0」：读的人若字面照做，细分一次（`refinements=1`，登记允许）会被误读成 S13；装置另给了 `s13=` 字段直接出判定，所以按那个字段读就不会误判。
什么现象会推翻：`s13=` 字段与 `equivalence_stops_at_s13` 的拼法对不上（我读代码是同一函数出的）；或哪一个版本的比对器读 `refinements` 当 S13 触发。

### K2.e `q5_not_run=no_root_write` 那一支退 3，是不是只在登记写的那种情形下走到

判定：**一致**（但没有用例或变异盯着这一支，**未验**）。
- 触发条件：主取样点分支里 `export.not_exported` 有值——「if let Some(why) = &export.not_exported {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7471`）。`export_prepared_stream` 里把它设成 `Some` 的只有一处：「export.not_exported = Some("no_root_write_to_judge".to_string());」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:4478`），条件是「if stream.judged_root_index == usize::MAX {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:4474`），也就是截断后的写表里一条根槽区的写都没有（`judged_root_index` 取最后一个 `RootRecordFua` 种类的写：「.rposition(|write| write.kind == StepKind::RootRecordFua)」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7158-7161`））。Q5 这条流里已确认的第 2 次调用的根槽写落在截断之前：`q5_shape` 一行 `root_writes_made_plain=[52] truncation=81 … confirmed_root=1,3`（`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-main-point-frozen-bin.stdout:17`），所以走不到——与登记一致：「6. 「没有根写可判」那一支在这条流上走不到」（`research/prompts/e171-r1-prereg.md:862`）。走到只会发生在被测实现这一趟不再写根槽（实现回退了），那时退 3 并记 `q5_not_run=no_root_write`，不进 S13、不进判定：「"name=q5 sampling_point={sampling_point} not_run=no_root_write not_exported_why={}",」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7475`）。
- 退码：文件头写「3 停机条款 S1、S3、S5、S9、S13 没过，或 `confirmed-loss-form` 那一趟 Q5 没跑」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:26`），这一支返回 3（判决行从「states=0 s13=false q5_not_run=no_root_write」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7479`）起）。
- 未验：这一支没有内联用例（它在 `run_confirmed_loss_form` 里，不好单测），也没有变异行。方向相反的取样点分支（从「// 方向相反的取样点照读法 7 全枚举」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7396`）那一段起）不看 `not_exported`，没有根槽写时会得到 `states=0` 的判决行而不是 `q5_not_run`，那一支我没造出来验（推的）。
推翻条件：有一条别的路径把 `not_exported` 设成 `Some`（今天 grep：Q5 分支走到的 `export_prepared_stream` 里只有 4478 一处；4376 是构造函数，6913 一带在 `run_dump` 里，不经 Q5 分支）；或这条流上真出现没有根槽写的截断。

## K3　E171 装置：其余改动

### K3.a RefFS 改成正式依赖之后依赖方向对不对

判定：**一致**。
- 改动：`Cargo.toml` 加一行「singlefs-reference-filesystem = { path = "../singlefs-reference-filesystem" }」（`crates/singlefs-checker-tier/Cargo.toml:14`），装置里换成「use singlefs_reference_filesystem as reference_filesystem;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:35`）。`Cargo.lock` 只多一行（`git diff HEAD --stat -- Cargo.lock crates/singlefs-checker-tier/Cargo.toml`：`Cargo.lock | 1 +`、`Cargo.toml | 2 ++`），与登记 2.3 第 2 点要求相符。
- 规则：RefFS 的行写「依赖只有 `std`，不引实现、池级 checker、harness 档、checker 档，实现与池级 checker 也不依赖它」与「它是库，被 harness 档与 checker 档调用」（`.claude/rules/verification.md:15`）；D13 已定项 3 的三条：「`singlefs-core` 与 `singlefs-checker` 的依赖闭包里没有它」（`.claude/kb/decisions/13-验证路线.md:97`）。门禁 ⑤⑥⑦ 判的是 RefFS 自己的闭包、源码零引、实现与池级 checker 的闭包里没有它；checker 档依赖它不在三格射程里（登记 2.3）。
- 我用 `cargo metadata --offline` 在冻结树副本上复算（不是门禁）：RefFS 的直接依赖是空集（闭包只有它自己，⑤ 的内容）；`singlefs-core`、`singlefs-checker`、`singlefs-format`、`singlefs-replay-prefix` 的闭包里都没有 RefFS（⑦ 的内容）；`singlefs-harness` 的闭包里没有 `singlefs-checker-tier`。依赖边：`singlefs-checker-tier` → `singlefs-reference-filesystem`、`singlefs-harness` → `singlefs-reference-filesystem`，没有反向的边（原样在 `research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k3-anchors-and-closure.txt:33`–`:39`）。⑥ 的真名那一半：RefFS 的 `src` 与 `tests` 里 `grep -rEn 'singlefs_core|singlefs_checker|singlefs_harness|singlefs_checker_tier'` 零命中（目录里只有 `Cargo.toml` 与 `src`）；别名那一半我没查。
- 没跑门禁 checker-independence-and-sync（归主 agent）。
推翻条件：门禁 ⑤⑥⑦ 任一红；或 RefFS 的 `Cargo.toml` 里出现 dev-dependencies 经中间包够到实现。

### K3.b V4、V11 两支改得是不是照登记修订 2 第 2.4 节

判定：**一致**。
- V4：登记要求新用例「新用例 `a_stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged`」（`research/prompts/e171-r1-prereg.md:780`）。装置里：函数在「fn a_stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged() {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9943`），断言四条都在：`StoppedByAPanic { call_index: 3, .. }`、「assert_eq!(export.not_exported, None, "panic 截断的流照常导出");」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9971`）、「assert!(!export.observations.is_empty(), "取样态照判");」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9972`）、「"panic 之前对拍判过的调用恰 1 次（第 2 次调用）"」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9979`）。变异行锚点与登记写的一样（`export.ending = Some(stream.ending.clone());` 全文恰 1 处，我的 `tools/check_e171r1_anchors.py` 判 28 行锚点各命中 1 次、替换文与原文不同、点名的测试是 `#[test]` 函数：`rows=28 bad=0`，`outputs/k3-anchors-and-closure.txt:29`）。
- V11：登记要求新用例「新用例 `a_torn_slot_beside_raised_tails_is_only_the_undecidable_branch`」（`research/prompts/e171-r1-prereg.md:786`）三条断言，装置里在「fn a_torn_slot_beside_raised_tails_is_only_the_undecidable_branch() {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9910`）：撕后 `some_device_has_fewer_than_two_verified_slots` 为真、`counter_sentence` 是 `None`、`witnessed_counter` 等于没撕之前的——「"另三槽抬过的 tail 还在，c_见证 与撕之前那份相同（取样点敏感）"」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9936`）。变异锚点是「let counter_sentence = if some_device_has_fewer_than_two_verified_slots」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3161`）。登记还要求「它注释里「V11 … 这里红」那半句挪到新用例上」（`research/prompts/e171-r1-prereg.md:785`）：旧用例的文档注释（从「/// 见证判据的两支：干净的池上两支都假」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9876`）起）已无 V11 字样。
- 内联用例条数：`grep -c '#\[test\]'` 现数 44（登记写今天 40，加 V4、V11 各一条与候选二的对拍、V18 那条，共 44）。
推翻条件：这两条用例在冻结树上编不过或不绿（我没跑，checker 档，推的）。

### K3.c 第 1618、8729 行一带注释与代码是否一致

判定：**一致**（一处旧措辞残留，见 K3.d 末）。背景材料的 1618、8729 是暂存区那一版的行号（`git show :crates/…` 现取：1618 行是「胶水在这一版跨出一片 inode 叶容器时断言失败」，8729 行是流三用例文档注释里「（模型今天只罩一片叶容器）」）；冻结树里对应处是：
- 「// 胶水造观测时 panic（`model_comparison::model_unit_role` 里模型还不罩的几种单元」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1632`），后面列「extent 上段根之下的节点、记账树根之下的节点、墓碑树第二片起的容器与根之下的节点」。对着胶水：「TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {」（`crates/singlefs-harness/src/model_comparison.rs:73`）、「TransactionUnit::AccountingTreeNodeBelowTheRoot(position) =>」（`crates/singlefs-harness/src/model_comparison.rs:87`）、「TransactionUnit::TombstoneLeafContainer(index) => {」（`crates/singlefs-harness/src/model_comparison.rs:101`，断言只许第 0 片）、「TransactionUnit::TombstoneNodeBelowTheRoot(position) =>」（`crates/singlefs-harness/src/model_comparison.rs:110`）：四处 panic，与注释列的一一对上；`InodeLeafContainer` 不在里面，注释说的「跨出一片 inode 叶容器不再走到这里」属实（`crates/singlefs-harness/src/model_comparison.rs:80` 按片造角色）。
- 流三用例文档注释：「没有一次「模型接不了这份观测」，模型一次都不停。」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10204`）；代码断言「assert_eq!(live.driver.model_stopped_at_call, None);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10233`）与「assert!(!live.driver.calls.iter().any(|call| {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10227`，里面匹配 `ModelCannotTakeTheObservation`）、所有调用 `Agrees` 或取号暖机。装置自己的跑法也对得上：流三 240 次调用的模型判定直方图是 `Agrees: 238, NotModelled: 2`、`model_stopped_at_call=None`（原样 `outputs/k3-stream3-model-verdicts.stderr:4`，是副本二进制的探针行，不是产物行）。
  一处没有被注释提的限制：模型自己的 135 片 `assert!` 在 `answer_create_inode` 里，不在 `catch_unwind` 里；流三只建 235 个 inode，走不到。

### K3.d 28 行装置变异：各自是不是只让点名的那条红、有没有等价或取样点不敏感的

判定：**锚点一致；灵敏度是推的（checker 档重型，没跑）；有一处文字小冲突（V9）；有一批告警没有变异行（见 K2.c）。**

- 构成：28 行 = 暂存区已有的 23 行（`git show :crates/mutations.tsv | grep -c '^E171R1 装置'` 为 23，与冻结树里的逐行比较 `diff … | grep -c '^<'` 为 0，即一行没变）+ 这一轮新追加的 5 行（V4、V11、V17、V18、V19，冻结树 `crates/mutations.tsv` 第 2481–2485 行；HEAD 里这类行为 0）。
- 锚点：28 行各恰命中 1 次，替换文与原文不同，点名的测试都是 `#[test]` 函数：`rows=28 bad=0`（`outputs/k3-anchors-and-closure.txt:29`）；与「25-3 只核了 28 行锚点各恰命中 1 次」一致。
- 新 5 行逐条（静态读，推的）：V4 红在「`export.not_exported` 应为 `None`」（9971 行）；V11 红在「`counter_sentence` 应为 `None`」，原用例对它取样点不敏感、新用例补了（登记 2.4 的说法属实）；V19 红在「各类态数之和等于组合数」与主取样点之和，**运行期也红**（S1 两条 `holds=false`，退 3；`research/prompts/m3-formal-code-b25-r1-sonnet-model/outputs/k2-v17-v19-runtime-main-point.txt:3`–`:6`）；V17 靠「模型停在第 3 次调用」第二遍的 216 个不适用态，V18 靠手造的 journal 重叠——两条的取样点是**专门补的敏感点**，运行期同值（K2.c），符合 `.claude/rules/mutation-sampling.md` 的处置：「补一个敏感的取样点，不是留档」（`.claude/rules/mutation-sampling.md:17`）。我没发现等价变异。
- 暂存区那 23 行：25-3 只核了锚点，我也没重核灵敏度（没动、重型）。25-1 与 25-3 改到的用例只有流三那一条（V9），它的断言现在多了「模型一次都不停、都对得上」，只会更敏感。
- **V9 文字小冲突**（三处并排）：登记：「| V9 | 装置流三的驱动 | 建 inode 停在 K − 1 次（叶容器还是一片）」（`research/prompts/e171-r1-prereg.md:513`）；用例文档：「/// （V9：停在 K − 1 次，叶容器还是一片，这里红）」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:10203`）；变异行：「E171R1 装置 V9 流三建 inode 停在叶容器变成两片的那一次」（`crates/mutations.tsv:2457`），替换文把停止条件从 `split + 1` 改成 `split`——停在第 K 次、叶容器**已经**两片，不是 K − 1、一片。两种形态都会让 `assert_eq!(live.inode_creations, split + 1)`（或前面的 `expect("叶容器变成了两片")`）红，抓得到；但用例文档描述的不是那一行变异的样子。这是 25-3 之前就有的措辞（暂存区那一版的文档同样写「K − 1 次」），不是这一轮引进的，列出来供主 agent 决定要不要改。
- 「只让点名的那条红」：我没有 checker 档的测试输出，不能证明别的用例不红；规则也没要求别的不红，只要求点名的那条红。

推翻条件：任一新行在冻结树上跑点名的测试不红（V17、V18 尤其）；或旧 23 行里有一行在 25-3 之后因为装置文本变动锚点虽命中而替换文已不改行为（重型整表复跑才看得到，归主 agent 提交时跑）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1.a 同一规则 | 一致 | 两边都是「只追加最右、满 233 末尾分裂」；实现纯函数 31455 条逐条落 ⌊k÷233⌋ 片、第 31456 条拒，模型式子在同一点 panic |
| K1.b 删除 | 规则没说 | 实现删空一片会左移全部重写，模型不罩；生成器不产生（建 inode 后再 unlink 是不适用） |
| K1.b 续做 | 一致 | 5 段挂载、冷启动恢复插进建 inode 的序列，全 `Completed` |
| K1.b 回退 | 规则没说 | 回退要现行版有文件，而建过 inode 后再没有；那段新代码在可达历史上不执行 |
| K1.b 多实例 | 一致 | 两次挂载夹着 233→234 边界，全对得上 |
| K1.c 胶水放过 / 误红 | 一致（被罩范围）；三个没罩到 | 第 3 片以后、占槽跨度、135 边界；新用例封顶在第 2 片，KX1、KX2 全绿，470 步变体红 |
| K1.d 超过 135 片 | 建议：不答拒；保留 assert；加边界用例；记账 | 条款写「根装满时长一层」，实现的拒是缺口，不是规范 |
| K1.d 第二片之后回退 | 建议：记账 + 钉前提的小用例（或删段） | 整历史用例加不出（模型 API 在无文件版上拒回退） |
| K1.e 3 行变异 | 一致 | 三行各红点名那条（第 235 步，分配代分歧）；别的 110 条 harness 档测试零红（仅我的探针红在第一行） |
| K2.a 同类判定相同 | 一致（没找到反例） | 主取样点真流 5040 个态，恢复、O1 两列、RefFS 与类推出的 0 处不同；抹键自证红 936 |
| K2.b 三条 oracle | 规则没说 | 类只主张 O1 两列、RefFS、恢复；O2 在这条流上恒空红名单，看不出差别；O3 不在这一趟 |
| K2.c ①–④ 与 V17–V19 之外的放过 | 有放过（告警没证明会红） | ①②④、落地串、S13 拼法、提示整环七处告警改成不计数，复刻口径上一行不变；我没造出具体的分错类；V17、V18 运行期读不出 |
| K2.d S13 与 class_landing | 计法一致；两处小冲突 | 登记把 class_landing 释成「恢复落点」，代码数的是「落地串」；q5_equivalence 行仍带细分字段 |
| K2.e no_root_write 退 3 | 一致（未验） | 只在没有根槽写时走到，这条流上走不到；没有用例、方向相反那支不看它 |
| K3.a 依赖方向 | 一致 | RefFS 直接依赖空，实现与池级 checker 闭包里没有它，两档指向它 |
| K3.b V4、V11 | 一致 | 两条用例与变异行照登记 2.4 |
| K3.c 注释与代码 | 一致 | 1632 行四处 panic 与胶水逐一对上；流三 238 个 `Agrees` + 2 个取号暖机 |
| K3.d 28 行变异 | 锚点一致；灵敏度推的；V9 文字小冲突 | 23 旧 + 5 新；V17、V18 取样点专补；告警无变异行见 K2.c |

## 要主 agent 现查 / 我复核不了的

- 重型（checker 档内联单测 44 条、`crates/mutations.tsv` 的 28 行装置变异的红绿）：没跑，上面凡涉及的都标了「推的」。V17、V18 会不会红尤其要提交时那一趟看。
- K2.c 的 KM1–KM7 与 K2.a 的 5040 态探针，是我在副本二进制上加的探针（`probes/e171_device_probe.patch`），不是入库装置的产物。按 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 的写法，要引进 kb 得由主 agent 把它做成入库装置的新一次跑，逐字段坐实。
- K1.d 里「31455 步整历史约半小时」是推的，我没跑那一段。
- `outputs/` 里有几行 `exit=252`（K2.c 提到的 V18、KM6、KM7 各一次）是 `run-with-memory-cap.sh` 自己的结局「内存排不上」（别的会话占着 slice 的总上限），不是装置的结果；我没绕开包装，换小一点的上限（`MEMORY_CAP=3G`、`2G`、`1500M`，脚本读这个环境变量）重跑，读数在 `…retry*.txt`。
- 复跑：`bash research/prompts/m3-formal-code-b25-r1-sonnet-model/tools/rerun.sh setup|k1|k2|k3`（工作目录不限；`SCRATCH` 默认 `/tmp/claude-1000/m3-formal-code-b25-r1-sonnet-rerun`；`k1` 约 8 分钟、`k2` 约 4 分钟，都经 `capped.sh 6` 与 `run-with-memory-cap.sh 8G`）。`setup` 我在另一个草稿目录验过能跑通，`k2`、`k3` 同；`k1` 的各条命令我分开跑过、输出在 `outputs/`，没把整条 `k1` 一口气在脚本里跑过一遍。

## 没做什么

- 没跑 checker 档测试（含装置内联单测）、`crates/mutations.tsv` 整表、门禁阶段（checker-independence-and-sync、doc 与 lint 一类）。
- 没替主 agent 采纳或出判决：K1.d 的两条处置是「建议」，选哪条路归主 agent。
- 没读 `research/results/e171-r1-*2026-10-05*` 与别轮腿的报告；没改仓内任何文件，只写了本报告与模型目录。
- K2.b 的 O3 没量（`confirmed-loss-form` 不跑 O3）；K2.a 只抽了 5040 态（不是 2²⁴ 全部）；K1 只量了 `harness` 档里与模型对拍有关的 110 条，没逐个跑 151 个二进制。
- K1.d 的「钉前提的小用例」「135 边界用例」我没写进仓里（不在我的写范围），只在副本里量了它们的读数。

## 草稿副本（交回闸要求逐个写全路径与为什么不删）

我建过的编译目录与仓副本都已删掉：`/tmp/claude-1000/m3-formal-code-b25-r1-sonnet/tree`（冻结树副本加 debug/release 编译目录）、`/tmp/claude-1000/m3-formal-code-b25-r1-sonnet/rerun-check`（`rerun.sh setup` 的验证副本）、`/tmp/claude-1000/m3-formal-code-b25-r1-sonnet/` 下的 `e171-frozen-bin`、`e171-probe-bin*`、`out/`、`logs*/`、`meta.json`、`staged-device.rs`。要复现用 `tools/rerun.sh setup`。留下的只有这份报告、`research/prompts/m3-formal-code-b25-r1-sonnet-model/` 和草稿目录里的两个小脚本（`lines.py`、`fix_report.py`，只是我核行号与改报告用的）。

## cite-check 末行

`python3 scripts/cite-check.py research/prompts/m3-formal-code-b25-r1-sonnet-output.md --root scripts/experiments/target/e171-r1-frozen-tree --root . --background research/prompts/_m3-formal-code-b25-r1-background.md`：

    ✓ 核了 83 处引文，对上 83 处，没判 0 处（逐处列在上面）
