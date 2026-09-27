# 实审 B3c-3 报告（implementation-writer）：树表 0 条的一版上写行那一版的对拍比实例表与分配代

时刻：JST（UTC+9）；底座与各文件的 sha256 见第二节。交补丁，主工作区一个字没改。

## 一、结论

- 做成了规格表第 1、3、4 条与第 2 条的「写行那一版」那一半：`transaction.rs` 的 `VersionWithoutFilePublishOutput` 带上 `units`（这次写出的单元）与 `allocation_records`（这一版的全部分配记录），胶水从这两个字段解出整张实例表与每个角色的分配记录，写行那一版不再走「比不了」的两臂，模型两个方向都比；模型另加两道闸：模型说这次重写了实例表 / 重写了角色的那一版交回这两臂，直接判对不上（`model.rs:2034`、`model.rs:2080`，行号是补丁打上之后的）。
- **没做到的一半，停下交主 agent（第七节第 1 条）**：零单元发布（暖机、`apply_publish_without_units`、mkfs 之后第一次可写挂载那次）一个字节都不写、不经分配器，`publish_without_units(pool, previous_root, plan)` 手里只有上一版的根，拿不到这一版的实例表字节与分配记录；要它带上，得改签名、让 `mount.rs` 把上一版整份交进来（`mount.rs` 不在我的文件单里），或者胶水按指针从上一版的输出往下带（规格没定这条路）。所以「比不了」两臂对树表 0 条的一版**只对写行那一版不再出现**，零单元发布那几版照旧计数（新用例把次数钉住：第二次挂载里写行 1 版全比、暖机 1 版照计数）。
- 发布路径一个字节不变：改动只在交回的结构体上加字段与一次 `clone`，`PublishWrites`、`persist_publish_writes`、点名项与记录都没动（第四节）。
- 推翻条件：主工作区上打这份补丁之后，(a) 写行那一版的实例表或分配记录错了而对拍不红（新用例四条改坏的用例会先红）；(b) 录制流或层 0 钉值变了（那就是我对「只加字段」的判断错了）；(c) 随机历史里写行那一版报「这一版的实例表 / 单元的分配代」对不上而实现没错（说明胶水解法与实现口径不一致）。

## 二、底座

- 第一份副本：主工作区 2026-09-27 01:50:28 UTC（10:50 JST）拷的，C554 乙补丁落在 01:48:31 UTC 之后。证红（第五节）在它上面做。
- 协调消息说 Z3-A 乙又改了 `history.rs` 等：02:28:16 UTC（11:28 JST）整份重拷主工作区为 `rebased/`，套上同一份补丁（`git apply` 过），变异表按名字合并；fmt / clippy / build / 动到的测试二进制与门禁阶段在 `rebased/` 上跑（第六节）。补丁本身不动 `history.rs`（胶水改动只落在 `model_comparison.rs` 就够，`history.rs` 调的是同名函数），两份底座生成的补丁逐字节相同（`cmp` 过）。
- 02:28 UTC 那份底座里的 sha256（交回时再对主工作区核一次，见第九节）：

```text
c943101565c90b5549996dfb73450371955064a56acf08b9cc0d49aee187cbf8  crates/singlefs-core/src/transaction.rs
4b736f81a42b5449c927122bec3c50bd0cf97bbbabd714e196a40efd5267f471  crates/singlefs-harness/src/model.rs
e2ce425bed3db95a9e244a7d2b60b1def298440a86fda9cc9ca2a36e30fd8501  crates/singlefs-harness/src/model_comparison.rs
e21ff5be84e1867dd9a8f69cfe287f95fe151a5f6cb4b0c3cfbedd014d93e83a  crates/singlefs-harness/src/history.rs
7bd1bfd0bce9044e7db0e98c00fd359e9406d951bea75172c7064e385e5a6630  crates/mutations.tsv
b17ccb76531667b13df5755ee44fee71e140755823e7b06275e3f209d9440b43  crates/singlefs-harness/tests/common_admission/mod.rs
```

## 三、写过的文件

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-core/src/transaction.rs` | `VersionWithoutFilePublishOutput` 加 `units: Vec<PublishedUnit>`、`allocation_records: AllocationRecordsOfTheVersionWithoutFile`；新枚举 `AllocationRecordsOfTheVersionWithoutFile`（写行：`WrittenIntoTheAllocationRecordTreeByThisPublish(Vec<AllocationRecord>)`；零单元：`SameAsThePreviousVersionNotHandedInByAZeroUnitPublish`）；写行那一路交回 `writes.units.clone()` 与装进分配记录树的那一份记录；零单元那一路交回空 `units` 与第二个成员；`publish_without_units` 解构重组时带上两个新字段 |
| `crates/singlefs-harness/src/model_comparison.rs` | 解实例表抽成 `instance_table_rows_of_units`（带文件的一版与写行那一版共用）；新 `observed_instance_table_written_by_a_row_publish`（写行那一版解不出就判 `Undecodable`，不当比不了）；分配记录抽成 `allocation_records_of_every_role(units, allocation_records, root)`，根记录直接指着而 `units` 里没有的角色按指针位置条目找：实例表（原有）、树表（新加，树表 0 条的一版从不重写 mkfs 那片树表）；`observed_root_of_version_without_file` 按 `allocation_records` 的成员分流 |
| `crates/singlefs-harness/src/model.rs` | `judge_instance_table`：`NotInTheOutput` 落在这次重写了实例表的那一版上判「这一版的实例表」；`judge_allocation_generations`：`RewrittenRolesOnly` 落在模型说这次重写了角色的那一版上判「单元的分配代」；两个臂与计数字段的文档改成「零单元发布」；单测 `a_version_without_file_compares_the_rewritten_roles_both_ways` 改写并改名为 `only_a_zero_unit_publish_on_a_version_without_file_may_hand_in_the_rewritten_roles_only`，`each_version_compares_its_instance_table` 末段改成「写行那一版交比不了判红、暖机那一版照计数」 |
| `crates/singlefs-harness/tests/model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file.rs` | 新测试二进制，5 条用例（第五节） |
| `crates/mutations.tsv`（经 `patch/mutations-append.tsv` 与 `patch/mutations-delete.txt`） | 追加 13 行（名字都以「实审 B3c-3（树表 0 条的一版上写行那一版比实例表与分配代）：」起头，全文见 `patch/mutations-append.tsv`）；删 1 行「实审 B3b 第 12 条（分配代对拍两个方向）：树表 0 条的一版只查实现重写的角色模型都有（少重写一个角色不报）」——改后它是等价变异（第五节末） |
| `crates/singlefs-harness/src/history.rs` | 没改 |

`git diff --stat -- crates litmus` 在补丁模式下分不出我的改动（副本不带 `.git`，主工作区里别的会话的改动很多），贴补丁自己的统计（`git apply --stat patch/crates.patch`，在主工作区跑）：

```text
 crates/singlefs-core/src/transaction.rs            |   38 +++
 crates/singlefs-harness/src/model.rs               |  179 ++++++++++---
 crates/singlefs-harness/src/model_comparison.rs    |  225 ++++++++++++-----
 ...tion_generations_of_a_version_without_a_file.rs |  268 ++++++++++++++++++++
 4 files changed, 601 insertions(+), 109 deletions(-)
```

## 四、为什么发布路径一个字节不变、层 0 钉值不动

- 录制流只来自 `persist_publish_writes(writer, &writes)` 里的 `CommitStep`（`transaction.rs` 同名函数），入参是 `PublishWrites`。补丁没动 `PublishWrites` 的类型与三处装它的代码、没动 `persist_publish_writes` / `persist_the_publish_or_freeze_it` / `resend_the_frozen_publish`，也没动记录与根的装法：写行那一路新加的只是 `let units_handed_in_with_the_version = writes.units.clone();`（`writes` 装完之后）与 `records: allocation_records.clone()`（交给分配器的那一份内容不变，原本是 move），零单元那一路只多两个常量字段。
- 下一次发布读这一版的地方（`mount.rs` 的 `empty_publish_plan_after_version_without_file` 读 `root`、`record`、`record_bytes`；`placements_taken_by` 读 `rewritten` 与点名项；`unmount` 读 `root`）一个都不读新字段。
- 所以段序列、状态数、层 0 各流的钉值不受影响。没跑层 0 与 54 号去实测这一句（按定义归提交时）；证据只到 `cargo build --all-targets` 与动到的测试二进制全过（第六节）。什么现象推翻它：提交时 54 号的任何一条钉值变了。

## 五、先红与证红

**先红**：新测试二进制先跑在「只做了 `transaction.rs` 的形状适配、胶水与模型都没改」那一版上（第一份副本），5 条全红（日志 `/tmp/claude-1000/impl-rev-b3c3/red-first-shape-only-2.log`：`test result: FAILED. 0 passed; 5 failed`）：

| 用例 | 改之前红在哪 |
|---|---|
| `the_row_publish_on_a_version_without_a_file_is_compared_by_its_whole_instance_table_and_every_role_both_ways` | `:153` 实例表比过 `left: 0 right: 1`（计数里写行那一版落进 `instance_tables_not_in_the_output: 2`、`rewritten_role_sets_compared: 2`） |
| `a_wrong_row_in_the_instance_table_written_by_the_row_publish_is_reported`（写行写出的实例表里上一个实例那一行的所选根 txg 加 1，单元照样解得开） | `:200` `left: Ok(()) right: Err(InstanceTableOfTheVersion)` |
| `a_row_publish_that_hands_in_no_page_of_the_instance_table_is_reported`（交回的 `units` 里去掉实例表那一片） | `:214` 同上 |
| `a_wrong_allocation_generation_of_the_instance_table_written_by_the_row_publish_is_reported`（新写那一片在盘 0 上的分配代减 1） | `:236` `left: Ok(()) right: Err(AllocationGeneration)` |
| `a_wrong_allocation_generation_of_the_tree_table_the_row_publish_did_not_rewrite_is_reported`（mkfs 树表在盘 1 上的分配代改成 1） | `:262` 同上 |

（行号是那一刻的文件；改完之后 5 条全绿：`green-glue.log`、`new-final.log`。第一次先红时暖机次数钉成 2 次红在 `:147`，实测两块盘只暖机 1 次，改了钉值再跑的就是上表。）

**证红**：`bash research/scripts/prove-red.sh --copy <第一份副本> --memory 8G singlefs-harness <13 个名字>`，经 `capped.sh 4`，debug（被测代码里没有 `debug_assert`）。基线（不改源码）那一次各组都绿。13 条全抓到；日志随第一份副本删了，下表是删之前从每份日志里摘的 panic 行（`prove-red.log` 留着，逐条的判定原样在里面）。prove-red 按行里的过滤名跑，所以「同时红了哪些」只看得到过滤进来的那几条：每条都只有点名的那一条红（`0 passed; 1 failed`）。

| # | 改坏哪里（变异名去掉共同前缀） | 哪条断言红 |
|---|---|---|
| 1 | `transaction.rs` 写行那次不交回写出的单元（`units_handed_in_with_the_version` 换成空） | 新二进制 `:131`「交回的单元就是这次写出的那几个，与重写的角色逐项同序」 |
| 2 | `transaction.rs` 写行那次把分配记录报成零单元那一臂 | 新二进制 `:153:60` `expect("照实交回的这一版与模型对得上")` → `InstanceTableOfTheVersion`（胶水走零单元那一臂报比不了，模型的新闸判红） |
| 3 | `model_comparison.rs` 写行那一版的实例表照旧报比不了 | 同 `:153:60` → `InstanceTableOfTheVersion` |
| 4 | `model_comparison.rs` 写行那一版的分配记录照旧只交重写的角色 | 同 `:153:60` → `AllocationGeneration` |
| 5 | `model_comparison.rs` 不按根记录里的树表指针交回 mkfs 树表的分配记录 | 同 `:153:60` → `AllocationGeneration`（模型这一版有树表、实现没交） |
| 6 | `model.rs:2034` `if rewritten_by_this_publish {` → `if false {` | `--lib` `model::tests::each_version_compares_its_instance_table`，`model.rs:2911` `expect_err("写行那一版这次重写了实例表链，交回「比不了」")` |
| 7 | `model.rs` `== Some(&expected.key.checkpoint_txg);` → `!= None;`（凡有实例表角色都当这次重写） | 同一条，`model.rs:2919` 暖机那一版照计数的 `assert_eq!(…, Ok(()))` |
| 8 | `model.rs:2080` `if !rewritten_in_the_model.is_empty() {` → `if false {` | `--lib` `model::tests::only_a_zero_unit_publish_on_a_version_without_file_may_hand_in_the_rewritten_roles_only`，`model.rs:2819` `expect_err("写行那一版只交回重写的角色（角色集合与模型的相同）")` |
| 9 | `model.rs` `rewritten_in_the_model == *rewritten` → `.is_subset(rewritten)` | 同一条，`model.rs:2789` `expect_err("零单元发布多报一个重写的角色")` |
| 10 | `model.rs` 实例表 `as_slice() ==` → `len() >= len()`（与 B3b 那条同一处原文，换了点名的用例） | 新二进制 `:202` `left: Ok(()) right: Err(InstanceTableOfTheVersion)` |
| 11 | `model.rs` `Undecodable { what: _ } => Ok(())` 插在原臂前面 | 新二进制 `:216` 同上形 |
| 12 | `model.rs` 分配代 `==` → `<=` | 新二进制 `:238` `right: Err(AllocationGeneration)` |
| 13 | `model.rs` 分配代 `==` → `>=`（与 B3b 前的第 136 行同替换文，换了点名的用例） | 新二进制 `:264` 同上 |

13 条都证过，没有留给 59 号只追加不证的行。第 2 至 5 条红在同一句 `expect` 上、分得开是哪一格（报的 aspect 不同，见上表）。

**删掉的 B3b 那一行是等价变异**：新闸让「模型说这次重写了角色」的那一版在 `==` 之前就判红，走到 `rewritten_in_the_model == *rewritten` 时模型那一侧恒为空集，而空集 ⊇ S 与空集 == S 同真同假，`is_superset` 那条替换在任何输入上都与原式同值。实测：临时加一行同原文同替换文、点名新单测，prove-red 判「没红」（`prove-red.log` 第 14 行 `✗ 等价核（临时）：原 B3b 904 那条	没红`，`test result: ok. 1 passed`），证完删掉了那一行。原来那一行点名的用例 `a_version_without_file_compares_the_rewritten_roles_both_ways` 也改名了，不删它 59 号必红。换上的是第 9 条（同一处原文、会改行为的 `is_subset`）。

**等价、没单独证的一处**：写行那一版胶水把「`units` 里一片实例表都没有 / 链接不上」判成 `Undecodable` 而不是 `NotInTheOutput`——换成后者，模型的新闸照样判「这一版的实例表」，两种写法对拍结果相同，所以没给它挂变异。

## 六、交回前的验证（都在 02:28 UTC 那份底座 `rebased/` 上，自己的 target，`capped.sh 4`，跑编出来的代码经 `run-with-memory-cap.sh 8G`）

负载：开跑前 `ps` 只看到别的实现员（`impl-rev-a4e`）的 `cargo build`，没有 qemu / fio / vm-bench / e152；没等锁。

| 项 | 原样末尾 |
|---|---|
| `cargo fmt --all -- --check` | `exit=0`（无输出） |
| `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 那七条 `-D` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.33s` / `exit=0` |
| `cargo build --offline --all-targets` | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 36.59s` / `exit=0` |
| 新二进制 `--test model_comparison_judges_…_of_a_version_without_a_file` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.42s` |
| `-p singlefs-core --lib` | `test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.45s` |
| `-p singlefs-harness --lib` | `test result: FAILED. 89 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 426.95s`——红的那一条是基线红（见下） |

**基线红集**（同一份底座、不打补丁的副本上跑的）：`--lib` 里只有 `model_comparison::tests::every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways`，基线 `89 passed; 1 failed`，打补丁之后同一条、同一处红：`种子 0：NewFinding { signature: ModelDisagreement { aspect: "模型说该成、实现拒了" }, observation: FailureObservation { position: Operation(15), operation_kind: Some(CrashRecoveryAbandoningTheNewestRoot)`，实现拒的是 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`——C554 乙（挂载重读一次）带进来的，不是这一件。第一份副本（01:50 UTC）上基线也是这一条红。

**登记给我的门禁阶段**（`stage-owners.tsv` 里 implementation-writer 那几道，`SINGLEFS_GATE_FULL=1`，在 `rebased/` 上跑）：

| 阶段 | 退出码 | 原样末行 |
|---|---|---|
| 33 | 1 | `    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。`——点名的只有第 659 行「E158 root_choice_repair session s10…原文在 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs 里命中 2 次」；主工作区不打补丁同样只红这一行（`gate-33-main.log`）。我追加的 13 行原文都恰好命中一次（合并时脚本核过）。 |
| 53 | 0 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 74 | 1 | `  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）`，3 条红：`crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、`rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`。同一个二进制 release 在不打补丁的副本上跑，红的是同三条、原因同上（快档 96 段里新发现 46 段，全是 `NewerStateStillUnreadableAfterOneReread` 那 46 次）。打补丁前后每段的步数、比过的根条数逐段相同，只有比过的分配记录多了写行那几版的：偏向复用 29268→29310、快档 31194→31356、偏向回退 22266→22302，其余三段不变；两边的对不上都只有「模型说该成、实现拒了」，没有一条「这一版的实例表 / 单元的分配代」。 |
| 92 | 77 | `  ! /tmp/claude-1000/impl-rev-b3c3/rebased 不是 git 仓，本阶段跳过`（副本不带 `.git`；补丁不碰 layout 与 checker） |
| 93 | 0 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，…）` |
| 94 | 0 | `  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（…）`，末行是它的「判不了的」说明 |
| 89 | 77 | `  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）` |

74 与 33 的红都不在这一件的改动里，照写，没修。

## 七、停下交主 agent 的

1. **零单元发布那几版还是「比不了」**（规格表第 2 条的另一半）。`publish_without_units` / `prepare_the_publish_without_units` 只收上一版的 `RootRecord`；`mount.rs` 的三处调用（`publish_empty_after`、取号之后第一次发布、预演）都只交根。这一版的实例表与分配记录就是上一版的（指针照抄），可这次发布手里没有那一份。三条路，我一条都没走：
   - 甲：改签名，交上一版整份 `VersionWithoutFilePublishOutput`，输出里照抄它的 `units` / 记录（同带文件那一版「没重写的从上一版照抄」）。要改 `mount.rs`（C554 乙、Z3-A 乙刚改过）、`transaction.rs` 里 `warm_up`，以及 `transaction.rs` 之外直接调 `publish_without_units` / `prepare_the_publish_without_units` 的另 6 份（`history.rs`、`e158_root_choice_repair.rs`、4 份测试，`grep -rlE '\b(publish_without_units|prepare_the_publish_without_units)\(' crates --include=*.rs` 数的）；而且取号之后第一次发布接的是从盘上恢复出来的上一版（`PreviousVersion::WithoutFile { root, table }`），它也没有单元字节，还得 `mount.rs` 从盘上读回来装进去。
   - 乙：胶水按指针往下带：一次挂载交回的那一串里，零单元那一版的实例表指针、分配记录树根指针与前一版相同，就用前一版交回的 `units` / 记录比；会话里单调的零单元发布要在 `history.rs` 的会话上记住上一份。比的是实现交出的东西，不另读盘；但这是规格没定的比法。
   - 丙：维持现状，零单元那几版只比重写的角色集合（空集），实例表照计数。
   无论哪条，mkfs 之后第一次重写之前的那几版（`AfterMakeFilesystem` 起点那段历史的第一次挂载与之后的零单元发布）都没有分配记录可比：mkfs 不建分配记录树，只有分配器在内存里种的两条，没有哪份输出带它们。
2. **模型那两道新闸是我加的判法**（`model.rs:2034`、`model.rs:2080`）：规格第 2 条说「两臂对这一版不再出现」，我让模型在「这次重写了实例表 / 模型说这次重写了角色」的那一版上遇到这两臂就判对不上，不只靠胶水不交它们。它顺带把带文件的一版的写行那一版也罩进去（那一版的 `units` 本来就带整条链，真历史上走不到）。不要这两道闸就删第 6、7、8 条变异，第 9 条还要换一条（原 B3b 那条在闸在时是等价变异，闸撤了它又不等价了）。
3. **基线红，不是这一件**：C554 乙之后，`CrashRecoveryAbandoningTheNewestRoot` 那一步的可写挂载被 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)` 拒、模型说该成：`--lib` 1 条、74 号 3 条。变异表里点名 `every_published_version_is_compared_…` 的第 907–909 行（B3b 的）在这之前 59 号跑不红。
4. 门禁 33 号红在第 659 行（E158 装置那一行原文命中 2 次），主工作区不打补丁同红，归 E158 执行员。

## 八、B3b 第二节第 2 条第 2 小条（这一件不做）：现状与要改哪里

行号是主工作区 02:28 UTC 那一版（不打这份补丁）的。

- **带文件的一版、mkfs 之后第一次重写实例表之前**：`crates/singlefs-core/src/transaction.rs:6444` `(InstanceTablePlan::Carry(_), None) => {}`——第一个文件版本（`publish_first_file`，`:4437`）没有上一版的 `TransactionOutput`，照抄实例表时 `units` 里一片都不放；胶水 `crates/singlefs-harness/src/model_comparison.rs:197` 交回 `NotInTheOutput`，模型照计数。要比，得让第一个文件版本拿到它下面那一版实例表的字节：那一版是树表 0 条的 mkfs 那一片（`MakeFilesystemOutput::instance_table_unit`）或写行那次写出的链（这份补丁之后在 `VersionWithoutFilePublishOutput::units` 里）。改法落在 `publish_first_file` 的入参（交上一版树表 0 条的输出或 mkfs 那一片的字节）与 `:6444` 那一臂；调用方（`grep -rlE '\bpublish_first_file\(' crates --include=*.rs`，`transaction.rs` 之外）：`history.rs`、`scenario.rs`、`crash_injection.rs`、三个装置（`e156`、`e158`、`first_transaction_on_device`）与 22 份测试。与第七节第 1 条的甲同一类改动，可以一起定。
- **从盘上重建、实例表多于一片的一版**：`crates/singlefs-core/src/recovery.rs:1816` 重建带文件的一版时 `units` 里只放第 0 片（`TransactionUnit::InstanceTable`，字节取根记录指着的那一片）；胶水 `model_comparison.rs:205` 链指着第 1 片而 `units` 里没有，交回 `NotInTheOutput`。要比，重建时沿链读完（`recovery.rs:1298` 的 `instance_table_chain_of_root` 已经会沿链读）、把第 1 片起各片按 `TransactionUnit::InstanceTablePageAfterTheFirst(k)` 放进 `units`。`recovery.rs` 这一轮归 C554 乙，不在我的文件单里。

## 九、补丁与交回那一刻的核对

补丁目录 `/tmp/claude-1000/impl-rev-b3c3/patch/`：`crates.patch`、`mutations-append.tsv`（13 行）、`mutations-delete.txt`（1 行）、`report.md`（本报告的拷贝）。没有 `mutations-replacements.tsv`。`crates.patch` 由 `make-patch.sh` 从底座原件与副本逐文件 `diff -u` 生成（新文件带 `new file mode 100644`），不含 `crates/mutations.tsv`。

交回前 02:40:25 UTC（11:40 JST）在主工作区核的（原样）：

```text
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-harness/src/model.rs: OK
crates/singlefs-harness/src/model_comparison.rs: OK
crates/singlefs-harness/src/history.rs: OK
crates/mutations.tsv: OK
crates/singlefs-harness/tests/common_admission/mod.rs: OK
git-apply-check-exit=0
delete-hits [1]
append-name-clashes 0 six-fields True
d93b873149397edaee2241b96f6cf048ea364f4dac74cc1ef53124a89054511e  /tmp/claude-1000/impl-rev-b3c3/patch/crates.patch
1facfccf78bfb73a64bf071a9fe4f7f5bc9ece2150c423a43c7170209295a9dd  /tmp/claude-1000/impl-rev-b3c3/patch/mutations-append.tsv
f0c879e2adfd5a4c5cf7a937be8ff6890a0973999980774e0d3710acf7f0d237  /tmp/claude-1000/impl-rev-b3c3/patch/mutations-delete.txt
```

（前六行是主工作区对 02:28 UTC 底座 sha256 的核对，也就是交回那一刻主工作区的 `history.rs`、`model_comparison.rs` 与我验证用的底座逐字节相同；`delete-hits` 是要删的名字在今天表里命中几行。）

## 十、草稿目录删了什么、留了什么

- 删了：第一份副本 `copy/` 262M（证红在它上面做，prove-red 的逐条日志随它删了，摘要在第五节与 `prove-red.log`）、`baseline-copy/` 262M、`target-baseline/` 511M、`baseline2/` 263M、`target-baseline2/` 1.1G、`rebased/` 263M、`target-copy/` 18G。编译目录与仓副本一个不留。
- 留着：`patch/`（交付）、`report.md`、`progress.md`、`base/` 与 `base2/`（两份底座里那几个文件的原件与 `SHA256SUMS`，打补丁对不上时拿它们三方合并）、`make-patch.sh`、`verify.sh`、各份 `*.log`（先红、证红、基线、验证与门禁的原样输出）、`spec.md`（主 agent 给的）。

## 十一、没做什么

- 零单元发布那几版的实例表与分配代没接上（第七节第 1 条，交主 agent 定路）；B3b 第二节第 2 条第 2 小条那一格按规格不做（第八节）。
- 没走三方对抗；没提交；层 0（名字带 layer0 的测试二进制）、54 号、55 号、57 号、59 号、QEMU、herd7、`cargo test --all`、`check.sh`、`gate.sh` 整轮都没跑（重型，归提交时）。层 0 钉值不变是推的（第四节），没实测。
- 74 号与 `--lib` 的红、33 号的红都是基线红，没修（第六节、第七节第 3、4 条）。
- 92 号在副本上退 77（不是 git 仓），没在主工作区单跑它判我的补丁（补丁不碰 layout 与 checker）。
- 证红在 debug 下跑，没在 release 下另跑一遍（被测的模型、胶水与 `transaction.rs` 那几处没有 `debug_assert`）。
- 没改 kb、没写 `research/`。
