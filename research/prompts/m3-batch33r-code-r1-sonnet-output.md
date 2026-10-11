# m3-batch33r-code-r1 云端正推腿报告（Sonnet）

轮 m3-batch33r-code-r1（代码轮第一轮），日期 2026-10-07。分到的格：K1（R1–R4）与 K3；立场照正文第四节「云端正推」：从条款与主 agent 定的读法推到代码行、测试与变异行。
被判对象是冻结副本 `/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/` 里那 4 份；实现员报告（`~/.local/share/singlefs/work/m3-batch33/r/report.md`）的结论句只当线索，下面每一句都是自己在冻结副本里现读的。

## 0　读了什么、复跑了什么

- 读：`.claude/kb/decisions/08-核心索引结构.md` 已定项 5（第 106–151 行）与已定项 16（第 461–473 行）、`.claude/kb/milestone/03-third-txn.md` 三·4（第 86 行）、`records/2026-10-01-删除与并发事务.md` 的 Q 交回那一条（第 1127 行，主 agent 定的 ①②）、改动全文、冻结副本里 4 份被判文件的相关段。
- 复跑 1（4 份 sha256 与快照目录里的逐份相同）：

```
$ cd ~/.local/share/singlefs/work/m3-batch33r-code-r1/tree && sha256sum <4 份> | diff - ~/code/singlefs/research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt && echo SAME_4_OF_4
SAME_4_OF_4
```

- 复跑 2（harness 档探针，模型目录 `research/prompts/m3-batch33r-code-r1-sonnet-model/m33_stream_probe.rs`）：在冻结副本的拷贝里照新流的 `record_the_stream` / `prepare` 录同一条流，只经 core 与 harness，不碰 checker 档。输出摘要见 R3 一节。
- 复跑 3（Python 小模型 `oracle_model.py`）：把新、旧判法逐分支抄成 Python，再按 D8 已定项 16 第 4 条的读法另写一份「规约」，在小范围内穷举比对（R1 一节）。这份模型的新判法与规约都是我一个人读同一批条款写出来的，不共用 crash.rs 的代码，但共用「读法」这个前提，见 R1 的推翻条件。
- 复跑 4（变异行锚点、8 行加第 16 行，`anchors.py`）：R4 一节。

## R1　oracle 认「这一版起没有文件」

### R1.1　新判法在文件从不消失的表上与旧判法等价吗

**判定：一致（等价成立，且不靠取样，靠集合恒等式）。**

旧判法在「根不在表里、报没有文件」一支问的是「版本表里有没有一项比这个根旧」（改动全文里被删的那段是 `versions.iter().find(...)` 加 `<` 比较，只看有没有）。新判法在同一支问「比它旧的带文件的项里最新那一项存在、且比比它旧的没有文件的最新那一项更新」：

- 「let newest_older_with_a_file = self」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1051-1057`） 加 「.max_by_key(|version| (version.checkpoint_txg, version.instance));」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1057`）：比根旧的带文件项取最新。
- 「match (newest_older_with_a_file, newest_older_without_a_file) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1064-1070`），其中 「(Some(with_a_file), None) => Some(with_a_file),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1069`） 与 「(None, Some(_) | None) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1070`）：表里没有「没有文件」那一组时（`with_a_file_only` 的 「versions_without_a_file: &[],」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:996`）），结果是 Some 当且仅当存在比根旧的带文件项，与「任一更旧版有文件」只差一个「取哪一项点名」，违例类相同。

所以对任何带文件的一组、任何顺序、任何同 txg 两实例或回退造出的表，两种判法交回的违例类逐项相同；不同的只有违例原文里点名的那一版（旧的点名遍历里第一个比根旧的，新的点名最新的）。违例原文只进样本串，不进发现表的签名（「pub struct Layer0FindingSignature {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:553-557`），签名三段是哪一遍、哪一段、哪次发布），也没有别的测试按「下面有文件」字样比对（`grep -rn '下面有文件'` 在冻结副本里只命中 crash.rs 自己（含新单测里的一处 `violation.reason.contains`）与 core 里三处与 oracle 无关的文字）。

我的小模型把这一点穷举了一遍（实例 1、2 × txg 1–5 共 10 个根，每个根「不在表里 / 带文件」，共 1024 张表 × 10 个落到的根 × 读回结果（没有文件、走读失败、表里每一版的内容、表外内容）），不含「没有文件」组：

```
$ python3 oracle_model.py
MODEL equivalence_new_vs_old tables_with_a_file_only judged=81920 differing=0
```

**单测取的表够不够**：钉住等价的那条单测是 「fn a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any_older_version_with_a_file」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5043-5044`） 一带（第 5044 行起），取 「for versions in [」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5046-5048`） 两张表：多记录流（「fn the_multi_record_publish_stream_versions」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:4990`），实例 1 的第 3、4、5 代）与 「fn two_instances_at_txg_seven」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:4926`）（同 txg 7 的实例 1、2）；根取 「for instance in 0..=2 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5051-5052`） 与 「for checkpoint_txg in 0..=8 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5052`），最新根槽取 「for newest_persisted_root in [None, Some((root.1, root.0))] {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5063`），计数断言 「3 * 9 * (versions.len() + 2) * 2,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5087`）。
- 同 txg 两实例：有（第二张表）。
- 回退：没有。回退流的表（`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs` 里实例 1 的第 3、4 代加实例 2 的几代、第 8、9 代）没进这条单测。按上面的恒等式它们不会判得不同，所以只是取样偏窄、不是漏判。
- 对照用的旧判法是实现员在测试里按旧分支手抄的（「(RecoveryOutcome::NoFile { .. }, None) => versions」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5022-5028`） 起），不是旧代码本身（旧代码已被替换，冻结副本里只剩这份抄本）。我对着改动全文里被删的旧分支核过它：同一个 `<` 比较、同一个「有没有」语义，抄得对。
- 这条单测没有比 `RecoveryOutcome::Failed`（走读失败）一支；新旧两版在那一支都是 `(Failed, _) => ReadFailed`、不看版本表，我的模型里同样 0 差异（上面 81920 项里含走读失败）。

**什么现象会推翻它**：对一张只有带文件项的表，找到一个落到的根与读回结果，新旧判法交回的违例类不同（模型里那 81920 项之外的输入，或实现里 `newest_older_with_a_file` 的比较方向与 `with_a_file_only` 的构造不一致的地方）。

### R1.2　新流的版本表（各批不进表）与 D8 已定项 16 第 4 条对得上吗

**判定：一致。**

条款：「从意图提交起只读挂载、冷走读、模型读回答」（`.claude/kb/decisions/08-核心索引结构.md:468`） ；恢复代码照做：「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-core/src/recovery.rs:5077`） 之后 `return Ok(None)`（交回「没有文件」），最后一批删掉记录之后 「let Some(inode_record) = inode_record else {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-core/src/recovery.rs:5073`） 同样交回没有文件。所以意图那次起的每一版（意图、两批、最后一批）冷走读都答没有文件，条款要的 oracle 也就是：这四个根报没有文件不是违例、读出内容是违例。

新流的表（`prepare` 里）：A、B、C 在 `versions`（「let versions = vec![」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:390-406`）），意图那一版在没有文件组（「let intent = recorded.deletion_steps.first().expect("意图那一步");」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:407-411`）），两批与最后一批不进表。版本表里的 txg 对不对，我在冻结副本里用 harness 档探针录了同一条流（`m33_stream_probe`，经 core 的 `publish_unlink_intent` / `publish_unlink_step`，批大小取 `deletion_history_batch_size()`）：

```
PROBE version A instance=1 txg=3 data_units=1 nlink_record=Some(1)
PROBE version B instance=1 txg=4 data_units=2 nlink_record=Some(1)
PROBE version C instance=1 txg=5 data_units=3 nlink_record=Some(1)
PROBE deletion_step 0 instance=1 txg=6 data_units=3 record_present=true nlink=Some(0) deleted_inodes=[InodeNumber(1)]
PROBE deletion_step 1 instance=1 txg=7 data_units=2 record_present=true nlink=Some(0) deleted_inodes=[InodeNumber(1)]
PROBE deletion_step 2 instance=1 txg=8 data_units=1 record_present=true nlink=Some(0) deleted_inodes=[InodeNumber(1)]
PROBE deletion_step 3 instance=1 txg=9 data_units=0 record_present=false nlink=None deleted_inodes=[]
PROBE root_slot_fua_writes 9
```

即 A、B、C 是实例 1 第 3、4、5 代，意图第 6 代，各批第 7、8、9 代，流里一共 9 次发布（第 1、2 代是暖机），没有第二个实例。表与条款对得上。用模型把这张表（带文件：实例 1 第 3、4、5 代；没有文件：实例 1 第 6 代）对每个落到的根、每种读回结果判一遍，与规约逐项相同，输出：

```
MODEL unlink_stream_table txg=3 NoFile/A/B/C/foreign -> NoFileWhereThisRootHasOne | None | WrongContent | WrongContent | WrongContent
MODEL unlink_stream_table txg=5 NoFile/A/B/C/foreign -> NoFileWhereThisRootHasOne | WrongContent | WrongContent | None | WrongContent
MODEL unlink_stream_table txg=6 NoFile/A/B/C/foreign -> None | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone
MODEL unlink_stream_table txg=9 NoFile/A/B/C/foreign -> None | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone | FileReadWhereThisRootHasNone
MODEL new_vs_spec_all_tables judged=3739770 differing=0 first_diff=None
```

（第二行是对 3 739 770 项穷举的汇总：实例 1、2 × txg 1–5 的 10 个根，每个根「不在表里 / 带文件 / 没有文件」，共 59 049 张表。）

一处要点名的放宽：各批不进表，靠的是「根不在表里、报没有文件时看比它旧的最新那一项」（「(RecoveryOutcome::NoFile { .. }, Layer0VersionAtTheRoot::NotInTheTable) => versions」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1131-1132`）），所以第 7、8、9 代以及任何比第 6 代新的、不在表里的根报没有文件都放过。这条流里比第 6 代新的根只有第 7–9 代这三个（探针 `PROBE publish` 九行），放宽不漏；将来同一条流里再接第二个文件版本时，那一版必须进带文件组，不然读出内容会被判违例（偏严，不是偏松），这个方向安全。

**什么现象会推翻它**：探针在别的批大小或别的 `build_pool` 下录出第二个实例、或意图与某批的 (实例, txg) 与表里写的不同（表就错了）；或 recovery.rs 第 5077 行那一支被改成 nlink = 0 读出内容。

### R1.3　「没有文件」组里的版本被一次读出内容的恢复判成立吗

**判定：不成立（oracle 判违例）。**

「RecoveryOutcome::FileRead { .. },」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1150-1156`） 一支：落在 `WithoutAFile` 或 `NotInTheTable` 一律交回 「Layer0OracleViolationKind::FileReadWhereThisRootHasNone,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1154`） ，与读出什么内容无关。单测钉了第 6–9 代读出 C 的内容（「Some(Layer0OracleViolationKind::FileReadWhereThisRootHasNone),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5139`）），变异 3106 钉了落在没有文件那一版上报没有文件那一格（R4）。「没有文件」组里的版本被判成立的一形不存在：这一支没有任何比较内容的分支。
**什么现象会推翻它**：存在一个落到「没有文件」组里的根、读回内容、而 oracle 返回 None 的输入（模型里这类输入 0 个：`MODEL file_read_on_a_root_in_the_without_a_file_group judged=787320 accepted_as_ok=0`）。

### R1 小结

R1.1 一致、R1.2 一致、R1.3 不成立（判红）。没发现冲突。另有一条「条款有、测试没钉」的小格：`Layer0VersionTable::with_a_file_only` 构造不查带文件组内部的重复根（`new` 才查带文件与没有文件两组之间的重叠，「!versions_with_a_file.iter().any(|with_a_file| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1009-1012`）），重复根按先找到的那项判；旧代码同样用 `find`，不是新引入的。

## R2　续做走到空的全量每状态核：观察者挪进工作线程

### R2.1　工作线程上的挂载核与调用线程上的是不是同一个判法

**判定：一致（同一个函数、同一份输入，只是在哪个线程上调）。**

- 判定函数只有一份：测试里 「fn observe_every_state(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:611-620`） 起的 `observe_the_publish_boundary`、`observe_the_deletion_cut_in_the_middle`、`observe_the_mount_continuation` 三样；两种观察者都是这一个函数的闭包：「= &|image, report, counts| observe_every_state(prepared, image, report, counts);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:677`）（工作线程版）与 「= &mut |image, report, counts| observe_every_state(prepared, image, report, counts);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:682`）（调用线程版）。
- 层 0 枚举里两种放法评状态用的是同一个调用，参数逐项相同：「let consulted_report = evaluate_state_recording_findings(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2593-2601`）（调用线程版，报告与持久集合带回去）与 「let consulted_report = evaluate_state_recording_findings(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2605-2613`）（工作线程版，评完就地交 `observe`）。给观察者的三样都一样：崩溃镜像（基线、写表、这个状态的持久集合）、看 journal 那一遍恢复的报告、一份计数；差别只在计数记在哪里：工作线程版 「&mut tally.observer_counts,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2621`）（这一片自己的计数，评它的那个线程里），调用线程版 「&mut slice_tally.observer_counts,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3579`）（并片时按片号次序）。
- 状态数两边都记：工作线程版 「tally.observed_states += 1;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2623`），调用线程版 「slice_tally.observed_states += 1;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3581`）；并片时 「self.observed_states += observed_states;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:889`） 与 「self.observer_counts.absorb(observer_counts);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:890`） 相加，计数按名字求和（「for (name, count) in following.0 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:824-826`）），与次序无关，所以工作线程上各片先后完成不影响并出来的数。
- 观察者类型要 `Fn + Sync`（「&'observer (dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts) + Sync);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2511`） 一带），只读 `prepared`，各线程共用；挂载在每个线程自己造的稀疏盘拷贝上做（「let materialized = materialized_crash_image(image);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:544-557`）），没有跨线程共享的可写物。

**什么现象会推翻它**：同一个状态在两种放法下交给观察者的报告或持久集合不同（例如工作线程版的 `persisted` 在评之前被移走）；或观察者里读了线程局部状态。我在代码里没看到。

### R2.2　三种跑法逐项相等的对拍能不能分辨「工作线程上漏核了一部分状态」

**判定：能分辨，但分辨靠的是计数名与对调用线程那一趟的比较，不靠「每个状态都看过」那句断言。**

对拍是 「fn the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and_with_the_observer_on_the_calling_thread(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:801`）：一个工作线程、四个工作线程（观察者都在工作线程）、观察者在调用线程（四个工作线程），三份 `Layer0Tally` 用 `assert_eq!` 比（「on_several_worker_threads, on_one_worker_thread,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:819-826`） 一带）。`Layer0Tally` 派生了 `PartialEq`（「#[derive(Clone, Debug, Default, PartialEq, Eq)]」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:746`）），字段含状态数、违例、checker 逐条、发现表、`observed_states`、`observer_counts`（「pub observed_states: u64,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:778`）、「pub observer_counts: Layer0ObserverCounts,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:780`））。

- 工作线程漏核一部分状态（`observe` 没被调）：观察者计数里 `mount_continuation_steps_N` 每个状态加一（「&format!("mount_continuation_steps_{expected_continuation_steps}"),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:587-590`）），漏几个就少几个，与调用线程那一趟比不相等；快档用例里也有 「.map(|steps| counts.count_named(&format!("mount_continuation_steps_{steps}")))」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:777`） 起的求和等于状态数那一句（「"每个状态都可写挂载过一次"」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:780`））。所以漏核会红。
- 但「每个状态都看过」那一句 「tally.observed_states, tally.states,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:754-757`）（以及层 0 枚举里 「tally.observed_states, states_of_this_run,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3665-3668`））**分辨不出**这一种：工作线程版是在 `observe(...)` 调用之后无条件 「tally.observed_states += 1;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2623`），不管观察者做没做事；它只能抓「整片没评」「片丢了」。这一句在工作线程版上近似同义反复，不能当「漏核」的证据。
- 一个工作线程与四个工作线程两趟：状态 189 个，默认切法两趟都是 12 片、每片 16 个状态（探针外的小算：片数取 「const LAYER0_MINIMUM_SLICE_COUNT: u64 = 64;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1703`） 与线程数乘 「const LAYER0_SLICES_PER_WORKER_THREAD: u64 = 16;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1705`） 的较大者，每片至少 「const LAYER0_MINIMUM_STATES_PER_SLICE: u64 = 16;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1707`））。所以 1 对 4 只比调度、不比切法；续跑用的切法另有一套（「const LAYER0_RESUMABLE_SLICE_COUNT: u64 = 65_536;」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2446`）），这条对拍一个都没走到。
- 变异 3109（R4）是它的会红证据：把工作线程那一趟的计数换成空计数，对调用线程那一趟不相等。

**这条对拍没钉的格（条款或定的读法要求，但没有用例）**：
1. 工作线程观察者加断点续跑：新测试里 `Layer0Resume::NoProgressFile` 只在快档（「&Layer0Resume::NoProgressFile,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:700`）），全量那条用 「&Layer0Resume::from_environment(FULL_ENUMERATION_STREAM_NAME),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:892`） 但是 `#[ignore]`。冻结副本里 `OnTheWorkerThreads` 的调用方只有这一个测试文件（`grep -rn OnTheWorkerThreads` 在 `tests/` 与 `src/` 里除 crash.rs 外只命中它）。续跑读回的片（「SliceWaitingToBeMerged::RestoredFromTheProgressFile(restored_tally) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3560-3567`））不再给观察者看，靠进度文件里那一片的 `observed_states` 与 `observer_counts` 接上；工作线程版把计数写进片的计数、再由 「file.append_finished_slice(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:3585-3589`） 落盘，路径与调用线程版相同（两版都在并片时落盘），读码上对得上，没有用例跑过。这是「没人钉」，不是「错」。
2. 分片（`SINGLEFS_LAYER0_SHARD`）与工作线程观察者同样没有用例。计划哈希不含观察者在哪个线程上（「message.push(u8::from(has_observer));」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2839`），只含有没有观察者），两种放法的进度文件可互认，这是设计（改动全文里的文档注释写明）。

**什么现象会推翻它**：把工作线程版里的 `observe(...)` 调用改成只对偶数序号调，而快档对拍与求和断言都不红；我读码推它们会红，没有跑（checker 档不跑）。

## R3　新流：多记录流 A、B、C 之后 unlink 那个文件

### R3.1　快档与全量的期望写全没有

**判定：测试自己声明的范围（见 「版本表是 A、B、C 三版有文件，意图那一版起没有文件」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:5`） 起的文档注释）里写全了；声明范围之外，D8 已定项 5、16 里有 5 格这条流没有钉（逐格见 R3.3）。**

条款与检查对照（检查的位置都在新测试文件里；「复跑」列是我在 harness 档探针二里对真代码跑出来的，见 R3.2）：

| 条款 | 流里的检查 | 位置 | 复跑 |
|---|---|---|---|
| 「操作从未发生，用户那边也没返回成功」（`.claude/kb/decisions/08-核心索引结构.md:116`）：单元写落了、记录没落 ⇒ 走上一版，意图那次读回 C | 删除停在中间的观察者 | 「assert_eq!(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:477-491`） 起（走上一版的根）与 「(DeletionPublishKind::Intent, RecoveryOutcome::FileRead { content, .. }) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:484-491`）（读回 C 的字节） | 单元写落一个与全落，都落到 C、读回 C |
| 「恢复时续做，已做完的部分被幂等地重跑」（`.claude/kb/decisions/08-核心索引结构.md:117`） | 可写挂载一次、续做步数 = 落到的那一版还差几步 | 「mounted.output.deletion_queue_continuation.len(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:581-586`） | 落到意图 3 步、批一 2、批二 1、最后一批 0 |
| 「续做一遍发现无事可做，然后删掉意图」（`.claude/kb/decisions/08-核心索引结构.md:118`） 与 「删除意图必须与最后一批工作在同一个事务里提交」（`.claude/kb/decisions/08-核心索引结构.md:120`） | 本设计没有「做完而意图未删」这一态；检查是最后一批之后续做 0 步、挂完队列空 | 「assert!(current.deleted_inodes.is_empty(), "挂完队列是空的");」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:593`） 与变异 3102（R4） | 最后一批之后落到第 9 代、续做 0 步 |
| 「从意图提交起只读挂载、冷走读、模型读回答」（`.claude/kb/decisions/08-核心索引结构.md:468`） | oracle 的新判法（R1），每个状态都判 | `assert_clean` 里 「assert_eq!(tally.violations, 0, "oracle：{:?}", tally.first_violation);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:623`） | 62 个代表状态上 oracle 无违例 |
| 「「有树、文件集合为空」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile`」（`.claude/kb/decisions/08-核心索引结构.md:469`） | 落到删除那几版的挂完是第三个成员 | 「(PoolVersion::WithTreesAndNoFile(current), LandedVersion::OneOfTheDeletionPublishes(_)) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:592`） | 第 6–9 代挂完都是 `WithTreesAndNoFile` |
| 「`mount_writable` 在写行与暖机之后、交回之前把队列走到空」（`.claude/kb/decisions/08-核心索引结构.md:470`） | 挂载交回「走到空」 | 「matches!(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:566-573`） 起 | 62 个状态都 `walked_to_empty=true` |
| 「每批释放尾部 K 个单元」（`.claude/kb/decisions/08-核心索引结构.md:466`）（K 由只供测试的旋钮压小） | 录流后核形状：意图不动单元、每批少一个、最后一批删记录 | 「fn assert_the_shape_of_the_deletion_steps(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:290-324`） | 探针里 `data_units=3,2,1,0`、最后一批 `record_present=false` |

快档与全量的状态数，我只复核了闭式算术与段长（探针输出 `segment_sizes` 与测试里 「2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:337-342`） 起那串逐项相同，35 段，9 个 1 写段、17 个 2 写段）：每次写只取两态的全量闭式 1 448 083 588（探针 `closed_form_two_states_per_write 1448083588`，测试 「assert_eq!(closed_form_with_two_states_per_write, 1_448_083_588);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:857`）），快档展开的段（长度小于 10）的闭式 139（探针 `closed_form_of_segments_shorter_than_ten 139`），再加十个系统配置槽 2 写段每段由 3 态变 8 态的 +50 得 189、全量得 1 448 083 638（算术：`python3 -c` 核过 1448083588 + 50 = 1448083638，快档 139 + 50 = 189）。这个 +50 来自 `layer0_state_count_with_torn_in_place_overwrites`（checker 档的函数），我复核不了它自己算出的数（要编 checker 档测试）。

**什么现象会推翻它**：新流的检查里漏了一条上表某行条款的断言（我在文件里没找到），或探针里某个状态的落点与测试期望不同（62 个状态没有）。

### R3.2　崩在意图中间、某一批中间、最后一批之后，各读回哪一版、挂完是不是走到空

**判定：与 D8 已定项 5、16 对得上。** 下面是探针二（`m33_state_probe`，harness 档，core 的 `recover` 看 journal 那一遍 + 可写挂载，取 62 个在层 0 枚举域里的代表状态：「前 k 段全持久」与「前 k 段全持久再加第 k 段只落第一个写」）里与你问的三种崩点对应的行，原样：

```
PROBE2 state=prefix19+first_write_of_segment newest_persisted_root(txg,instance)=Some((5, 1)) effective_root(instance,txg)=Some((1, 5)) outcome=C mount=ok steps=0 walked_to_empty=true current=WithFile
PROBE2 state=prefix20 newest_persisted_root(txg,instance)=Some((5, 1)) effective_root(instance,txg)=Some((1, 5)) outcome=C mount=ok steps=0 walked_to_empty=true current=WithFile
PROBE2 state=prefix20+first_write_of_segment newest_persisted_root(txg,instance)=Some((5, 1)) effective_root(instance,txg)=Some((1, 6)) outcome=NoFile mount=ok steps=3 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=prefix24 newest_persisted_root(txg,instance)=Some((6, 1)) effective_root(instance,txg)=Some((1, 6)) outcome=NoFile mount=ok steps=3 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=prefix24+first_write_of_segment newest_persisted_root(txg,instance)=Some((6, 1)) effective_root(instance,txg)=Some((1, 7)) outcome=NoFile mount=ok steps=2 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=prefix28 newest_persisted_root(txg,instance)=Some((7, 1)) effective_root(instance,txg)=Some((1, 7)) outcome=NoFile mount=ok steps=2 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=prefix32 newest_persisted_root(txg,instance)=Some((8, 1)) effective_root(instance,txg)=Some((1, 8)) outcome=NoFile mount=ok steps=1 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=prefix32+first_write_of_segment newest_persisted_root(txg,instance)=Some((8, 1)) effective_root(instance,txg)=Some((1, 9)) outcome=NoFile mount=ok steps=0 walked_to_empty=true current=WithTreesAndNoFile
PROBE2 state=all_persisted newest_persisted_root(txg,instance)=Some((9, 1)) effective_root(instance,txg)=Some((1, 9)) outcome=NoFile mount=ok steps=0 walked_to_empty=true current=WithTreesAndNoFile
```

按段号：第 19 段是意图那次的 26 个单元写、第 20 段是它的两份记录、第 21 段根槽、第 22 段轮换；第 23–26、27–30、31–34 段依次是批一、批二、最后一批。读法：

- 崩在意图那次发布中间（单元写落一个，或全落、记录一份没落）：落到第 5 代（C）、读回 C、挂载续做 0 步、挂完 `WithFile`。对应 D8 已定项 5 三态表第一行「没有意图」，对应测试里 `Intent` 那一支。
- 意图那次的记录落了一份而根槽没落：恢复照记录走到第 6 代、答没有文件（探针里恢复看记录，一份记录落盘就走到新版本），续做 3 步。oracle 两边都认（落第 5 代读 C，或落第 6 代没有文件）；测试里的手写检查只管「记录一份没落」那一半（「&& !self.record_copies.iter().any(|copy| persisted[*copy])」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:226`）），记录落了的状态交给 oracle、续做步数与池级 checker。
- 某一批中间（单元全落、记录没落）：落到上一版（意图或上一批）、答没有文件、续做还差 3、2、1 步。记录落了一份：落到这一批的版本，续做差 2、1、0 步。
- 最后一批之后：第 9 代、没有文件、续做 0 步、挂完 `WithTreesAndNoFile`（第三个成员，D8 已定项 16 第 5 条）。

判器那一头：`MODEL` 里的 `judge_states.py` 把这 62 行逐行交给 oracle 的 Python 抄本与续做步数的期望，输出 `JUDGE states=62 bad=0`（见下），落到的根分布 `(1, 1):3 (1, 2):7 (1, 3):8 (1, 4):7 (1, 5):6 (1, 6):7 (1, 7):7 (1, 8):7 (1, 9):5` 另有 `(0, 0):5`（mkfs 之后、取号之前），每个版本都有落点。

```
$ python3 judge_states.py <探针二输出>
JUDGE states=62 bad=0
JUDGE landed_roots(instance,txg)->states (0, 0):5 (1, 1):3 (1, 2):7 (1, 3):8 (1, 4):7 (1, 5):6 (1, 6):7 (1, 7):7 (1, 8):7 (1, 9):5
```

这条判法路径自己会红吗：把探针二输出里一行改坏（第 7 代那行读回改成 C、续做步数改 9），`judge_states.py` 必须报 BAD；rerun.sh 里有这一步（自证），输出见末尾「复跑一览」。

**复核不了的**：①快档 189 个状态里的撕裂镜像（系统配置槽的「新旧都读不出」第三态）；探针二没造它们，写行与取号落在撕裂的系统配置上能不能可写挂载，测试 「.unwrap_or_else(|error| panic!("每个崩溃状态上可写挂载都做得成：{error:?}"));」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:565`） 要的就是这个，我复核不了（要编 checker 档的枚举器）；②池级 checker、记录核对器在每个状态上全绿（checker 档）；③全量 1.4e9 个状态。实现员报告写明 checker 档测试一条没跑（`~/.local/share/singlefs/work/m3-batch33/r/report.md` 第 7 行与第 196 行，线索），所以快档绿不绿到今天为止没有人跑过，我的探针替代的是「每个代表状态上期望成不成立」这一半，不是整条测试。

**什么现象会推翻它**：探针二的任一状态上期望与实测不同（0 个）；或快档在撕裂镜像上挂载 panic。

### R3.3　这条流没钉的格（读法或条款要求、测试没有）

这些不是「错」，是对着 D8 已定项 5、16 逐条读出来、这条流没有断言的格；属不属于这条流的范围由主 agent 定：

1. **全量那条 `#[ignore]` 用例没登记成 `crash-case:`。** 「#[test]」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:836-838`） 起的全量用例标了 「#[ignore = "全量 1448083638 个状态」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:837`）；`scripts/crash/crash-case-check.py` 的 placement 判法按 「ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)」（`scripts/crash/crash-case-check.py:95`） 认出调了 `enumerate_layer0` 一族而标了 `#[ignore]` 的用例，要在 `.claude/gate.d/stage-inputs.tsv` 里有 `test=singlefs-checker-tier:crash_enumeration_unlink_after_the_multi_record_publish_stream:<函数名>` 的登记行。今天主工作区的登记表里没有：

```
$ grep -c 'unlink_after' .claude/gate.d/stage-inputs.tsv
0
```

   测试文件头的文档注释（「登记成崩溃枚举用例，带断点续跑、可双机分片」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:19`））写的是「登记成」，实际没登。实现员报告写明这一行归主 agent 登记（线索，`~/.local/share/singlefs/work/m3-batch33/r/report.md` 第 197 行）。我没跑 `crash-case-check.py`（它是门禁的静态判据，归主 agent），所以「placement 红」是读码推的。
2. **续做各批的事务号**（D8 已定项 16 第 2 条末句 「意图那次、每一批与续做的每一批各占一个事务号，等于本实例的下一个」（`.claude/kb/decisions/08-核心索引结构.md:466`））：观察者只数 `deletion_queue_continuation.len()`，不核续做那几批落成第几代。录流时的第 6–9 代是核过的（`publishes` 与 `version` 逐一对上，「let publish = publishes」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:354-359`） 起），续做的不核。
3. **挂完的盘没过池级 checker、没与不崩溃的终态比**（D8 已定项 16 第 7 条 ③ 后半与 ④：「每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿」（`.claude/kb/decisions/08-核心索引结构.md:471`））：观察者只核池版本的成员与队列空。第 7 条那组判据另有 harness 层的 `unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs`（只见文件名，我没读它的断言，不判它钉没钉）。
4. **①b：同一份崩溃镜像各自重挂载续做，两份盘逐字节相同**（「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」（`.claude/kb/decisions/08-核心索引结构.md:471`））：这条流每个状态只挂一次。
5. **快档不展开单元段**：快档里「单元写落一半」的停在中间只有「全落」那一个状态（「[1, 1, 1, 1],」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:772-773`）），落一半的 2^n − 2 个只在全量（`#[ignore]`、没有登记、没人跑过）里。这是快档的设计（「快档只展开写数小于 10 的段」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:18-19`）），不是缺陷；但三·4 的验收句（「那一处，那条流的判器红」（`.claude/kb/milestone/03-third-txn.md:86`））靠的是快档里的续做断言与池级 checker（最后一批之后的状态在快档里都有），不靠单元写落一半。

读法一句：R3 里与 D8 已定项 5、16 对得上的是「三种崩点各落到哪一版、读回什么、挂完走到空」；第 1 项是门禁面上的欠登记，第 2–4 项是 D8 已定项 16 第 6、7 条里这条流没声明要钉的格。

## R4　变异 8 行（第 3102–3109 行）与第 16 行改后的锚点

### R4.1　锚点：筛得到吗

**判定：9 行（第 16 行加新 8 行）原文在冻结副本里各恰好命中一次，替换后 `cargo check` 都编得过。**

```
$ python3 anchors.py
row 16: hits=1 line=1055 file=crates/singlefs-checker-tier/src/crash.rs
row 3102: hits=1 line=4111 file=crates/singlefs-core/src/transaction.rs
row 3103: hits=1 line=5422 file=crates/singlefs-core/src/mount.rs
row 3104: hits=1 line=1066 file=crates/singlefs-checker-tier/src/crash.rs
row 3105: hits=1 line=1069 file=crates/singlefs-checker-tier/src/crash.rs
row 3106: hits=1 line=1145 file=crates/singlefs-checker-tier/src/crash.rs
row 3107: hits=1 line=1057 file=crates/singlefs-checker-tier/src/crash.rs
row 3108: hits=1 line=1009 file=crates/singlefs-checker-tier/src/crash.rs
row 3109: hits=1 line=2621 file=crates/singlefs-checker-tier/src/crash.rs
```

第 16 行现在是 「(version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/mutations.tsv:16`） 一列的原文，落在 「(version.checkpoint_txg, version.instance) < (checkpoint_txg, instance)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1055`），即 `newest_older_version_if_it_has_a_file` 里「比根旧」那个 filter；替换成 `false` 后带文件那一组永远没有「比它旧的最新一项」，根不在表里报没有文件一律放过，仍是这一行的名字「oracle 放过没版本的更新根」要的故障。

替换后的编译（`mutation_compile_check.py`，每行在拷贝里单独施加、`cargo check --offline -p singlefs-checker-tier --lib`、再还原；自证：给替换文加一段写坏的语法，同一条命令退 101）：

```
MUTCHECK row=16 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=newer_root_without_any_version_reporting_no_file_is_violation
MUTCHECK row=3102 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers
MUTCHECK row=3103 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers
MUTCHECK row=3104 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=every_crash_state_of_an_unlink_after_the_multi_record_publish_recovers
MUTCHECK row=3105 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=a_table_where_the_file_never_vanishes_judges_a_root_not_in_it_like_any
MUTCHECK row=3106 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_fi
MUTCHECK row=3107 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only
MUTCHECK row=3108 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=a_root_both_with_and_without_a_file_is_refused_when_the_table_is_built
MUTCHECK row=3109 anchor_hits=1 cargo_check_exit=0 error_lines=0 named_test=the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_f
```

**什么现象会推翻它**：门禁 59 号的施加方式与我这里「整份原文换整份替换文、`\n` 转换行」不同（例如多一层缩进规整）；或第 16 行在主工作区里被别的合入再改。

### R4.2　点名的测试会不会红（读码推；checker 档不跑）

| 行 | 改坏什么 | 点名的测试 | 红在哪一句（读码） | 函数一级核（模型） |
|---|---|---|---|---|
| 16 | 比根旧的 filter 换成 `false` | 库单测 `newer_root_without_any_version_reporting_no_file_is_violation` | 「assert!(oracle_violation_for_versions(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:4966-4972`） 那句 `.is_some()` 落空 | 与规约不同 83 653 / 3 739 770 项 |
| 3102 | 最后一批不删 `deleted_inodes` 那条 key | 新流快档 | 最后一批之后的状态队列不空：续做步数 ≠ 0（「mounted.output.deletion_queue_continuation.len(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:581-586`）），或挂载报错 panic，或池级 checker I-9.5 红（`assert_clean`） | 不适用（core 代码） |
| 3103 | 挂载里续做被跳过 | 新流快档 | 落到意图、批一、批二的状态续做步数 0 ≠ 3、2、1，同一条断言。注意 「outcome: DeletionQueueContinuationOutcome::WalkedToEmpty,」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-core/src/mount.rs:5416`）：续做结果的初值就是「走到空」，跳过续做时 「matches!(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:566-573`） 起的「走到空」那句**不红**，红的是步数与 「assert!(current.deleted_inodes.is_empty(), "挂完队列是空的");」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:593`） | 不适用 |
| 3104 | 根不在表里时不看最新一项是不是没有文件 | 新流快档 | 第 7–9 代报没有文件判 `NoFileWhereAnOlderRootHasOne`，`tally.violations > 0`，`assert_clean` 红；对着探针二的 62 个状态，落到第 7、8、9 代、答没有文件的有 7 + 7 + 5 个（这 62 个里「第 k 段只落第一个写」中，段长 ≥ 10 的七个单元段的那 7 个只在全量域里，不在快档域里；记录段、根槽段、轮换段的都在两个域里） | 在新流表上第 7、8、9 代报没有文件都判违例（`MODEL unlink_stream_batch_roots_NoFile mutant=row3104 txg7:NoFileWhereAnOlderRootHasOne txg8:… txg9:…`） |
| 3105 | 没有「没有文件」组时，更旧的带文件版不判违例 | 库单测 `a_table_where_the_file_never_vanishes_…` | 参照模型对第 6–8 代报没有文件给出违例、新判法给 None，「for newest_persisted_root in [None, Some((root.1, root.0))] {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5063`） 里的 `assert_eq!` 红；`newer_root_without_…` 也同时红。**新流表上看不出这一行**（模型：`mutant=row3105` 在第 7–9 代 None，与正确值相同），只有库单测抓得到 | 与规约不同 28 501 项 |
| 3106 | 落在没有文件那一版上报没有文件判违例 | 库单测 `no_file_from_the_version_…` | 「for checkpoint_txg in 6..=9 {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5123-5129`） 一带第 6 代 `assert_eq!(…, None, …)` 红（新流快档也红：探针二里落到第 6 代答没有文件的有 7 个） | 与规约不同 196 830 项 |
| 3107 | 不看最新一项、看最老一项 | 库单测 `a_root_not_in_the_table_is_judged_by_the_newest_older_entry_only` | 第 9 代：最老的带文件版是第 3 代，比第 6 代旧，判成不是违例，「.expect("比第 9 代旧的最新一项是第 8 代、有文件：报没有文件是违例")」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5182`） 那一句 panic。**新流表上也看不出这一行**（第 7–9 代 None，同正确值） | 与规约不同 30 748 项 |
| 3108 | 同一个根既有文件又没有文件不拒 | 库单测（`#[should_panic]`） | 「#[should_panic(expected = "既写成有文件、又写成文件从这一版起没有了")]」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5206`）：不 panic 即红 | 不适用 |
| 3109 | 工作线程上看的观察者计数不记进那一片 | 新流对拍 | 工作线程两趟的 `observer_counts` 空，对调用线程那一趟 `assert_eq!` 红；快档用例里 「counts.count_named("cut_in_multi_record_publish_0"),」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:759-766`） 起那组 [3, 12] 也红 | 不适用 |

模型里「与规约不同的项数」是把变异后的判法抄进 `oracle_model.py` 对 3 739 770 项算出来的，说明这行变异改变了函数的判定，不是说点名的测试一定红；点名的测试红不红是上面「红在哪一句」的读码推。

**没有变异行盖着的新代码**（读码找的，没有一行变异会改坏它们）：
1. `FileRead` 落在「没有文件」组那一支（「RecoveryOutcome::FileRead { .. },」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1150-1152`） 的 `WithoutAFile` 一半）：库单测 「assert_eq!(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:5130-5141`） 起钉了，但没有变异行证明这条断言会红（我的模型里这类改坏有 787 320 项不同，函数一级看得见）。
2. 计划哈希里「没有文件那几版」只在非空时拼（「if !versions.versions_without_a_file.is_empty() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:2822`））：没有用例钉「空时与加这一项之前逐字相同」（旧进度文件照样相认），也没有用例钉「非空时不同的 N 组算不同的计划」；没有变异行。
3. 全量那条（`#[ignore]`、没登记）没有变异行，实现员报告写明这一点（线索）。

**什么现象会推翻它**：任一行点名的测试在改坏之后仍绿（要跑 checker 档，归主 agent 提交时的变异整表）。

## K3　被判的文件名单：4 份各有一格

| # | 文件（冻结副本里的全路径，sha256 与快照相同） | 判到的格 | 判定 |
|---|---|---|---|
| 1 | `/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs`（5385 行） | R1（版本表、新判法、旧签名转调）、R2（观察者、新入口、计划哈希）、库单测 4 条 | 一致；两处「没人钉」见 R2.2 与 R4.2 |
| 2 | `/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs`（1092 行） | 只加一个类型 | 一致（见下） |
| 3 | `/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs`（933 行） | R3、R2 的对拍 | 声明范围内一致；全量没登记，5 格没钉（R3.3） |
| 4 | `/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/mutations.tsv`（3109 行） | R4 | 9 行锚点各中一次、替换后编得过、读码推点名的测试都红；3 处新代码没有行盖着 |

第 2 份的判法：「#[derive(Clone, Copy, Debug, PartialEq, Eq)]」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:795-799`） 起的 `PublishedVersionWithoutAFile` 只有实例代号与 txg 两个公开字段，`PublishedVersion`（「#[derive(Clone, Debug, PartialEq, Eq)]」（`/home/fy5090/.local/share/singlefs/work/m3-batch33r-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:785`） 起）原样没动，所以所有只认带文件版本的老调用方（GPU 输入表、放量库、`crash_amplification.rs`）的类型与行为不变；`PublishedVersionWithoutAFile` 在冻结副本里只被 `crash.rs` 与新测试文件引用（`grep -rn PublishedVersionWithoutAFile` 命中 3 个文件，没有第四个）。「没有文件」组没有 GPU 一侧的对应物：本轮不判（正文第一节），但这意味着新流今天只能由 CPU 的层 0 判，放量库与 GPU 输入表还没有这一类版本。
**什么现象会推翻它**：别的 crate 里有按位置构造 `PublishedVersion` 并假定「版本表里每一项都带文件」的代码被这次合入改了语义（我只在 harness 与 checker 档两个 crate 里 grep 过，两处的构造都没改）。

## 复跑一览（模型目录 `research/prompts/m3-batch33r-code-r1-sonnet-model/`）

文件：`rerun.sh`（入口，变量 `SINGLEFS_REPO`，另 `SINGLEFS_FROZEN_TREE`、`SINGLEFS_THREADS`、`SINGLEFS_STAGES`，文件头写明）、`compare.sh`（用法在文件头第一行，排序、去耗时字段后比两份汇总，自己不依赖 `SINGLEFS_REPO`）、`oracle_model.py`、`judge_states.py`、`anchors.py`、`mutation_compile_check.py`、两份 harness 档探针 `m33_stream_probe.rs`、`m33_state_probe.rs`、`summary-expected.txt`（第一次从头跑的汇总，131 行）。编译 6G、跑 4G，线程上限 6，都经 `scripts/hosts/run-with-memory-cap.sh` 与 `capped.sh`，同一时刻只起一条 cargo，编译目录在输出目录里；只跑 harness 档与 `cargo check`，没有跑 `singlefs-checker-tier` 的测试。

「在空目录里从头跑通」：因为前台命令有 240 秒时限，每次分两次调用同一个输出目录（`SINGLEFS_STAGES="1 2 3 4"` 与 `"5 6 7 8"`），做了两次，输出目录各自从空开始：

```
$ bash compare.sh rr1/summary.txt rr2/summary.txt
SAME 131 行
562d030b3261f97ac34bbeb6d122f06d4c38e17f64ecfc891709551e781269f5  rr1/summary.txt
562d030b3261f97ac34bbeb6d122f06d4c38e17f64ecfc891709551e781269f5  rr2/summary.txt
```

自证会红（都在汇总里，`SELFRED` 两行）：

```
SELFRED broken_replacement_does_not_compile=yes
SELFRED judge_reports_the_two_doctored_states=yes
--- 改坏探针二输出里两行之后判器的输出（rr1/doctored-judge.txt 原样）：
JUDGE BAD state=prefix24 root=(1,6) outcome=NoFile problems=['steps=9_expected=3']
JUDGE BAD state=prefix28 root=(1,7) outcome=C problems=['FileReadWhereThisRootHasNone']
JUDGE states=62 bad=2
JUDGE landed_roots(instance,txg)->states (0, 0):5 (1, 1):3 (1, 2):7 (1, 3):8 (1, 4):7 (1, 5):6 (1, 6):7 (1, 7):7 (1, 8):7 (1, 9):5
--- 替换文写坏之后同一条 cargo check（rr1/selfred-mutcheck.txt 原样）：
MUTCHECK row=3106 anchor_hits=1 cargo_check_exit=101 error_lines=2 recompiled_checker_tier=True named_test=no_file_from_the_version_without_a_file_on_is_not_a_violation_and_a_fi
MUTCHECK   crates/singlefs-checker-tier/src/crash.rs:1145:182: error: expected pattern, found `@`: expected pattern
MUTCHECK   error: could not compile `singlefs-checker-tier` (lib) due to 1 previous error
```

小模型里变异后的判法与规约的差（`MODEL mutant=` 五行原样，差 0 就说明这行变异在函数一级看不见）：

```
MODEL mutant=row16 differing_from_spec=83653 of=3739770
MODEL mutant=row3104 differing_from_spec=55152 of=3739770
MODEL mutant=row3105 differing_from_spec=28501 of=3739770
MODEL mutant=row3106 differing_from_spec=196830 of=3739770
MODEL mutant=row3107 differing_from_spec=30748 of=3739770
```

开跑前 `ps -o pid,args -u "$(id -u)"` 看到本机有别的 `cargo test --no-fail-fast --release -p singlefs-checker-tier …`（经 gpu-quiesce 的 release 整套，不在共用约束列的那几类性能测量里）在跑；没有 qemu-system、vm-bench、fio、e152 在跑。按共用约束「只有别的 cargo 在跑」，全部加 `nice -n 19` 照常跑，没有测等文件锁的时间。

## 判定一览

冲突格 0 个，没有需要把两句并排整行抄的地方。

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 R1.1 新旧判法在文件从不消失的表上等价吗 | 一致 | 靠集合恒等式成立，不靠取样；模型对 81 920 项逐项相同，单测的两张表偏窄（没有回退表）但不漏判，违例原文点名的版本换成了「最新的」 |
| K1 R1.2 新流版本表对 D8 已定项 16 第 4 条 | 一致 | 实例 1 第 3、4、5 代带文件、第 6 代没有文件、第 7–9 代不进表，探针录出的 txg 与表逐项相同；比第 6 代新的根一律放过「没有文件」，这条流里只有第 7–9 代三个 |
| K1 R1.3 「没有文件」组被读出内容的恢复判成立吗 | 不成立（判违例） | 读回内容的分支不比内容，一律 `FileReadWhereThisRootHasNone`；模型 787 320 项 0 项放过 |
| K1 R2.1 工作线程与调用线程的挂载核是不是同一个判法 | 一致 | 同一个观察者函数、同一个评状态调用，只差计数记在哪一片；并片按名字求和与次序无关 |
| K1 R2.2 对拍能不能分辨工作线程漏核 | 能（靠计数名与对调用线程那一趟的比较） | 「每个状态都看过」那句在工作线程版近似同义反复，分辨不了「没调观察者」；1 对 4 同切法（12 片）只比调度；工作线程观察者加续跑 / 分片没有用例（没人钉，不是错） |
| K1 R3.1 快档与全量的期望写全没有 | 声明范围内一致；范围外 5 格没钉 | 全量那条 `#[ignore]` 没登记成 `crash-case:`（今天 grep 0 命中）；续做各批事务号、挂完过 checker 与终态对比、①b 逐字节相同、快档单元写落一半，都没断言 |
| K1 R3.2 三种崩点各读回哪一版、挂完是不是走到空 | 一致 | 62 个代表状态在真代码上跑，期望逐项成立（`JUDGE states=62 bad=0`）；撕裂镜像、池级 checker、全量复核不了 |
| K1 R4.1 锚点 | 一致 | 第 16 行与新 8 行各命中 1 次，施加后 `cargo check` 都过 |
| K1 R4.2 点名的测试会不会红 | 读码推：9 行都红 | 3105、3107 两行只有库单测抓得到（新流表上看不出）；3103 的「走到空」那句不红，红在步数与队列空；3 处新代码没有变异行盖着 |
| K3 crash.rs | 一致 | R1、R2 与 4 条库单测各有一格，两处没人钉 |
| K3 memory_pool.rs | 一致 | 只加一个类型，老类型不动，只被 crash.rs 与新测试引用 |
| K3 新测试文件 | 声明范围内一致 | 全量没登记，5 格没钉 |
| K3 mutations.tsv | 一致 | 9 行锚点各中一次、编得过；3 处新代码没有行 |

## 给主 agent 现查的线索（不是判决）

1. 全量那条 `#[ignore]` 用例今天没有 `crash-case:` 登记行（`grep -c 'unlink_after' .claude/gate.d/stage-inputs.tsv` 为 0）；登记行的写法实现员报告第 91 行给了原文。
2. 新对拍与层 0 枚举里「每个状态都看过」那句，在工作线程版上不能证明观察者被调过（R2.2）；靠的是计数名。要不要加一个「观察者被调次数」的独立计数，由你定。
3. 变异 3105、3107 只在库单测上红，新流表上看不出；库单测与新流各抓一半，提交时变异整表要两边都跑到。
4. 变异 3103 改坏之后「续做走到空」那句断言不红（`outcome` 初值就是 `WalkedToEmpty`），红靠步数与队列空两句；这两句都在快档用例里，没问题，只是第 566 行那句对这一行是空的。
5. 新测试到今天没人跑过绿；我的探针二在真代码上对 62 个代表状态把期望逐项跑成立了，但不含撕裂镜像与池级 checker，提交时快档的头一次绿红仍要看。

## 没做什么

- 没有跑 `singlefs-checker-tier` 的任何测试：新流快档、对拍、库单测 4 条、全量、变异整表都没有跑。「红在哪一句」全是读码推；实现员报告也写明这些一条没跑（线索），所以这一批新测试到今天没有人见过绿。
- 没有判 K2（攻方格）：观察者续跑、分片、判定次序、删除流里 oracle 放过的反例，不归这条腿。
- 没有采纳或出判决，没有改 `crates/`、`.claude/`、`.claude/gate.d/stage-inputs.tsv`；`crash-case:` 登记行（R3.3 第 1 项）没有写，也没有跑 `crash-case-check.py` 等门禁静态判据。
- 没有读 `research/prompts/m3-batch33r-code-r1-opus-output.md`、`…-opus-model/`、`…-main-verification.md` 与攻方腿的提示；实现员报告只读了结论、验证与登记几节作线索（第 7、31–33、76、91、95–135、144、196–197 行附近）。
- 没有读 `unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` 等 harness 层 `unlink_` 用例的断言，不判 D8 已定项 16 第 7 条在 harness 层钉没钉。
- 探针二没造撕裂镜像（系统配置槽「新旧都读不出」第三态），也不是层 0 枚举器：62 个状态是枚举域里的代表，不是全体；GPU 输入表、放量库（`crash_amplification.rs`）、恢复内核一侧不判（正文第一节）。
- 没有量全量每状态挂载的挂钟（R 推两台 13–65 天的算术不判；探针二 62 个状态一趟 52 秒是 6 线程、本机忙时的调度数，不当单次耗时）。
- 引用的 kb 行号是 kb 文件自己的行号；冻结副本里的引文是全路径加该文件自己的行号。草稿目录 `~/.local/share/singlefs/work/m3-batch33r-code-r1-sonnet/` 里自己建的编译目录与副本（`target`、`target-check`、`copy`、`rr1`、`rr2`）在交回前删掉（都可由 `rerun.sh` 重建，汇总留在模型目录的 `summary-expected.txt`）。
