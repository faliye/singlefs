# 实现员 impl-m2-writepath 报告：收口表第 13 行（C394 N1、N3）、第 5 行（C515）、第 38 行（实例表第二片写路径）

时刻一律 UTC（本机时钟），东京 = UTC+9。这一轮由两个实现员接力做完：前任（子 agent aa5f8a536cc664e40）05:3x–10:44 写完全部功能代码、测试、变异行与报告草稿，
进程退出时有几批验证没跑完；接手者（我）10:48 起核现场、补跑，主工作区其间又变了两次（13:10、14:50），我在副本里按现状三方合并了两次，
合并时补了一处语义冲突（第三节末），并成这一份。活全在副本里做，主工作区一个字没动。

**交付的补丁是 `work5/crates` 对 `base5-crates/`（14:49 主工作区 `crates/` 原样，14:52 核过与主工作区逐文件相同）的差。** 下文行号除特别注明外都是 `work5/` 里现取的。

**读数归属：**

| 读数 | 谁跑的 | 在哪个树上 | 日志（都在 `logs/`） |
|---|---|---|---|
| 10 条变异全红（`mount.rs` 那 5 条重跑 + 新加 5 条） | 接手者，14:52–15:38 | `work5/`（交付的那一底） | `mutations-worker9/` |
| 同样 10 条全红 | 接手者，13:48–14:50 | `work4/`（13:32 主工作区 + 补丁） | `mutations-worker8/` |
| 前任交的 34 条全红 | 接手者，10:54–13:29 | `work2/`（10:0x 主工作区 + 补丁） | `mutations-worker{5a,5b,5c,6,7}/` |
| 同样 34 条在旧底上全红（其中 11 条是 Q2 停下之前的版本） | 前任，06:28–08:14、09:5x | `work/`（05:3x 底） | `mutations-worker{1..4}/` |
| fmt、clippy（check.sh 同一张 lint 表）、`cargo build --all-targets` 全绿 | 接手者，14:52 | `work5/` | `check-first-three-work5.log` |
| 动到的测试二进制全绿（不带层 0） | 接手者，14:52 起 | `work5/` | `work5-touched-binaries.log` |
| 动到的测试二进制 + 层 0 五个流的快档全绿 | 前任 10:07–10:43（7 个）、接手者 10:53–13:30（其余 21 个与 core / checker lib） | `work2/` | `rebased-touched-binaries.log`、`rebased-touched-binaries-part2.log` |
| fmt / clippy / build 全绿 | 接手者，13:09–13:22、13:38–13:40 | `work3/`（13:08 主工作区 + 补丁）、`work4/` | `check-first-three-work3.log`、`check-first-three-work4.log` |
| check.sh 两次（红在别人的 e158） | 前任，09:53、10:02 | `work/`、`work2/` | `check-sh.log`、`check-sh-rebased.log` |
| 探针（Q1）、故障注入打中 Q2 的全量测试、基线全量 | 前任，06:1x–09:36 | `probe/`、`work/`、`pristine/` | `probe-*.log`、`modified-test.log`、`baseline-test.log` |

没跑的（主 agent 定）：`cargo test --all`（check.sh 第四步，13:1x 定，留给全部代码落定之后统一跑）；层 0 快档（13:47 定，留给最后统一跑的层 0 全量，`work4` / `work5` 上都没跑）。
接手者 12:31 起按主 agent 的「线程上限：4」用 `research/scripts/capped.sh 4` 起每条编译、测试、变异命令；那之前已在跑的变异与测试批是按整机并行度跑的。
副本、日志、脚本都在 `/tmp/claude-1000/impl-m2-writepath/` 下；逐步经过在 `drafts/progress-successor.md`，整点答复在 `progress.md`。

## 一、结论

1. **N1 做了**：释放之前按位置项读盘核校验和，那一读读不出先重读一次（只一次），还读不出就按对不上处置
   （`crates/singlefs-core/src/transaction.rs` 第 1839 行 `copies_failing_the_release_checksum_check`，第 1898 行那一句就是「重读一次」）。
   `PublishError::ReleaseChecksumReadFailedWhoseHandlingIsUndecided` 删掉，`crates/singlefs-harness/src/history.rs`、`model_comparison.rs` 两处穷举 `match` 跟着改。
2. **N3 做了，一格停下**：核出对不上（读不出也算）的那一份，在它那块盘上的分配记录留在「已分配」、不改成已释放
   （`crates/singlefs-core/src/allocator.rs` 第 909 行 `release_leaving_the_record_allocated_on`，调用点 `transaction.rs` 第 3547 行）。
   落盘即跨重挂；准入读分配器的已分配槽数，照已分配算、不另进式子。实九那一套只住内存的隔离位（`DeviceFreeMap` 三个字段、两个方法、
   三处空闲判定里的那一项，`PoolAllocator::quarantine_after_release_checksum_mismatch`）全删。
   **停下的一格**：一个单元只有一部分副本对不上时，照做两块盘的分配记录就不对称，第一版的冷启动走读要各盘记录集合相同
   （`crates/singlefs-core/src/recovery.rs` 第 1981 行 `allocation_records_are_one_per_device`，不同就判整池走读失败）、落点要各盘一致，
   怎么容下没有条款——这一格是跑全量时故障注入快档打中的。于是在任何写之前返回新成员
   `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（`transaction.rs` 第 2105 行，判定在第 1916 行）。每一份都对不上 / 都读不出时照定案做（第六节 Q2）。
3. **N4（C515）做了**：`crates/` 里 `policy_mismatches` 11 处同步删（`grep -rn policy_mismatches --include=*.rs crates` 在 `work5/` 零命中）。
   门禁 55 号期望串、它的 10 份 fixture、kb 两行、E142 产物要跟着改什么，见第七节（我不改）。
4. **实例表第二片写路径做了**：行一片写满 369 行再开下一片（`crates/singlefs-core/src/instance_table.rs` 第 140 行），
   bump 次序里尾片先、出生序号同序（`transaction.rs` 第 2262 行 `instance_table_page_roles_in_bump_order`、第 890 行 `build_instance_table_chain`），
   「有无下一片」写 1 / 0。每次写行整条链 COW 重写，被换下的旧链逐片释放（第 1721 行 `instance_table_chain_to_release`；旧链各片的指针随计划带进来：
   `InstanceTablePlan::Rewrite(InstanceTableRewrite)`，第 2240 行）。带文件的一版、树表 0 条的一版两条写行路径都写多片。
   `MountError::InstanceTableChainLongerThanOnePageUndecided` 删掉；取号之前的准入改成按这次之后的片数数角色，并先逐片核那条旧链（`mount.rs` 第 1153、1166 行）。
   新角色 `TransactionUnit::InstanceTablePageAfterTheFirst(InstanceTablePageIndex)`（第 1224 行）表示第 1 片起，第 0 片仍是 `TransactionUnit::InstanceTable`。
   验收三条都有用例：**第 371 次可写挂载成功**（取号 371，369 + 1 行两片）、**冷启动沿链读回真写者写出的全部 370 行**、**两片上池级 checker 全绿**。
5. **O3 做了**：checker 的 I-3.9 引用集合沿实例表链认第 1 片起的落点（`crates/singlefs-checker/src/walk.rs` 第 1389 行），配一份坏镜像用例证它会红。
6. **没替条款定的**：记录留在已分配之后池级 checker 的 I-3.11 当场红、I-3.1 之后会红（探针实测，第六节 Q1）；只一部分副本对不上那一格（Q2）；
   位置项指的盘不在池里（Q3，另一个成员 `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`，第 2094 行）。

7. **合并新底时补了一处**（接手者）：13:10 打进主工作区的「取号之前在分配器拷贝上走一遍那一串、发完断言与真发相同」按一片写死，写多片时那条断言当场 panic；
   按多片改了（`mount.rs` 第 1258、1353 行，第三节末），配 4 条新变异（A15–A18）与 1 条替代行（R529），全红。
8. **验证**（交付那一底 `work5/`）：fmt、clippy、`cargo build --all-targets` 全绿；动到的测试二进制全绿（第五节）；`mount.rs` 那 5 条与新加 5 条变异全红，
   前任交的 34 条在 `work2/` 上全红（第四节）。`cargo test --all` 与层 0 快档按主 agent 定留给最后统一跑。主表要删的 4 行、替代的 12 行按变异名给（第四节末），
   不照做门禁 59 号会红。
9. **新看到、没处理的一格**（Q7）：树表 0 条的一版上写行那一路只写一条记录，这份补丁让它能写多片之后，片数 ≥ 67（两万四千多行）时点名项装不下、`ByteWriter` 越界 panic；
   带文件那一路实二十已做再跨记录，这一路没接。交主 agent 定。

推翻条件：主工作区打上补丁之后第五节那些测试二进制里有一条红；第四节的变异在主工作区打上补丁之后有一条不红；最后统一跑的 `cargo test --all` 里有别的二进制因这份补丁红；层 0 全量（归 crash-verifier）
里写行路径出现新违例；或者有人指出 D3 已定项 10 ⑤ / D18 已定项 11 的新句子另有读法（例如「尾片先」只管落点、不管出生序号）。

## 二、交付物与写过的文件

交付（都在 `/tmp/claude-1000/impl-m2-writepath/`）：
- `impl-m2-writepath.patch`（sha256 `bd66237a76eea9bff422e0409d243b23a55946221b83e1b047eb54c1b7c5a745`）：`work5/crates` 对 `base5-crates/` 的差，只含 `crates/`，24 个文件，
  `crates/mutations.tsv` 不在里面。16:10:06 UTC 对主工作区现状 `git apply --check` exit=0（第五节末）。
  前任 10:06 生成的旧底补丁改名留着：`impl-m2-writepath-predecessor-work2-1006.patch`（对今天的主工作区打不上，不要用）。
- 变异表三件，**全部按变异名认、不按行号**：
  - `mutations-replacements.tsv`（sha256 `8298d0e850b1b394e7ee317892fe29e8e2341ef26585f7a6c07fc1fe190e211f`）：12 行整行替代，第一段是主表里现在那一行的变异名，其后六段是替代它的整行；
  - `mutations-delete.txt`（sha256 `5d8bb336c41e17c1f5c999238f0d61872739bd7bae1fcd74d7688a4f193d77f8`）：要整行删掉的 4 行的变异名；
  - `mutations-append.tsv`（sha256 `735af9f9fc19410e96b6b6bcc14fec3517bc5e8cdf2c79c7343df52bc81841a4`）：21 行追加到主表末尾（前任的 17 行 A01–A14、U01–U03，接手者的 4 行 A15–A18）。
  前任那两份按行号给的改名留着（`mutations-replacements-by-line-predecessor.tsv`、`mutations-append-predecessor-17.tsv`），行号已过时，不要用。
- 这份报告 `report.md`；前任 09:52 那一版改名 `report-predecessor-0952.md`。

补丁里的 24 个文件（`git apply --stat` 原样）：

```
 crates/singlefs-checker/src/walk.rs                |   57 ++
 crates/singlefs-core/src/allocator.rs              |   98 +--
 crates/singlefs-core/src/instance_table.rs         |  135 +++-
 crates/singlefs-core/src/mount.rs                  |  213 +++---
 crates/singlefs-core/src/recovery.rs               |    2 
 crates/singlefs-core/src/transaction.rs            |  673 ++++++++++++++------
 crates/singlefs-core/src/write_accounting.rs       |    4 
 .../src/bin/first_transaction_on_device.rs         |    4 
 .../src/bin/first_transaction_region_bytes.rs      |    5 
 crates/singlefs-harness/src/history.rs             |   12 
 crates/singlefs-harness/src/model.rs               |   91 ++-
 crates/singlefs-harness/src/model_comparison.rs    |   17 -
 crates/singlefs-harness/src/scenario.rs            |    2 
 .../tests/checker_known_bad_images.rs              |  141 ++++
 .../tests/first_transaction_step_five_publish.rs   |   18 -
 .../tests/first_transaction_step_two_data_unit.rs  |    4 
 ...ransaction_parallel_line_one_multi_unit_file.rs |   38 +
 .../tests/second_transaction_step_one_overwrite.rs |    1 
 ...econd_transaction_step_three_second_instance.rs |   10 
 ...ion_supplement_two_commit_generated_fallback.rs |    1 
 ...nsaction_supplement_two_instance_table_chain.rs |   45 +
 ...tion_supplement_two_instance_table_page_full.rs |  278 ++++----
 ...plement_two_instance_table_second_page_write.rs |  435 +++++++++++++
 ...n_supplement_two_release_checksum_quarantine.rs |  551 ++++++++++++----
 24 files changed, 2042 insertions(+), 793 deletions(-)
```

写这些文件的是谁：功能代码与用例全是前任写的（`work/` 里）；接手者在合并里改的是 `mount.rs`（第三节末那一处与导入行、`MountError` 成员）、`walk.rs` 导入行、
`first_transaction_on_device.rs` / `history.rs` / `model_comparison.rs` 的穷举臂——都在 `work4/` 里改、`work5/` 合并带过去。新文件只有一个：
`crates/singlefs-harness/tests/second_transaction_supplement_two_instance_table_second_page_write.rs`（集成测试，`tests/` 下自动进编译，不靠 `Cargo.toml` 的 `[[test]]`）。

主工作区 `git diff --stat -- crates litmus`（16:1x UTC 原样；**这些都不是这一轮的**，是别的会话打进主工作区还没提交的改动，这份补丁没打进主工作区）：

```
 crates/mutations.tsv                               |   64 +-
 crates/singlefs-checker/src/image.rs               |    8 +-
 crates/singlefs-checker/src/lib.rs                 |   10 +-
 crates/singlefs-checker/src/walk.rs                |  554 ++++++-
 crates/singlefs-core/src/journal.rs                |  131 +-
 crates/singlefs-core/src/mount.rs                  |  315 +++-
 crates/singlefs-core/src/mounted_read.rs           |    3 +-
 crates/singlefs-core/src/recovery.rs               |  403 ++++-
 crates/singlefs-core/src/transaction.rs            |  232 ++-
 .../src/bin/e158_root_choice_repair.rs             | 1611 ++++++++++++++++++--
 .../src/bin/first_transaction_on_device.rs         |   93 +-
 crates/singlefs-harness/src/fault_injection.rs     |    4 +-
 crates/singlefs-harness/src/history.rs             |   42 +-
 crates/singlefs-harness/src/model_comparison.rs    |   18 +-
 .../tests/checker_known_bad_images.rs              |  586 ++++++-
 .../tests/first_transaction_step_five_publish.rs   |   17 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |    7 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  223 ++-
 .../tests/second_transaction_step_four_rollback.rs |    8 +-
 ...second_transaction_step_three_formatted_pool.rs |   45 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ...transaction_supplement_three_fault_injection.rs |   35 +
 ..._transaction_supplement_three_random_history.rs |  182 ++-
 ...two_c533_row_publish_record_without_its_root.rs |    3 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   72 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  276 +++-
 28 files changed, 4546 insertions(+), 443 deletions(-)
```

别的会话在改的四个文件里补丁碰到的（相对 `base5-crates/`）：
- `crates/singlefs-checker/src/walk.rs` 3 处：导入行多导 `packed_unit_view`（第 20 行附近）、`references_of_root` 里 I-3.9 那一段的注释换成调用（第 1325 行起、调用在第 1331 行）、
  新函数 `note_instance_table_pages_after_the_first`（第 1389 行起，O3）。没碰 I-7.9、I-9。
- `crates/singlefs-core/src/recovery.rs` 1 处：第 2100 行一行注释（「今天写者只写一片」改成「行数不超过 369」）。没碰 `rebuild_version`。
- `crates/singlefs-core/src/mount.rs` 19 处 hunk：导入行、`MountError` 删一个成员与改一段文档、`PreviousVersion` 两个方法换成 `instance_table_chain` 与新函数
  `instance_table_rewrite_of_the_row_publish`（第 274 行）、写行两臂、`refuse_publishes_before_acquisition_that_do_not_pass_admission`（第 1153 行起）、
  取号之前在拷贝上走一遍那一段（第 1258、1353 行起，第三节末）、`establish_instance` 里拼 `InstanceTableRewrite` 与两处注释。没碰影子账与根环表那几处。
- `admission.rs` 没碰。

## 三、实现（按件）

### N1（D19 已定项 5 定案段，用户 2026-09-24；规格第一节第 1 条）

- `copies_failing_the_release_checksum_check`（`transaction.rs` 第 1839 行）：先按角色分掉豁免映射的四类（映射树、树表、实例表第 0 片与
  第 1 片起），再判「上一版有没有这个落点」，再对映射条目的两条位置项逐份读。读用一个闭包 `read_the_copy`，
  `read_the_copy().or_else(read_the_copy)` 就是「读不出先重读一次」；还读不出就把那一份按对不上记进交回的清单，不返回错误、发布照成。
- 位置项指的盘不在池里（`reader.device_identities()` 里没有它）：在读之前返回 `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`。
  改之前这一格落在旧成员里（`PoolReader::read` 对不在池里的盘也交 `None`）；按新条款它会变成「读不出 → 按对不上 → 那一份在它那块盘上的
  记录留在已分配」，而池里没有那块盘、没有那条记录，条款三句都没有对象，所以停在这一格（第六节 Q3）。
- 判定与后面的写读同一份输入：读盘核只做一次，交回的清单原样传进 `publish_admitted`，两者之间没有别的读、也没有写。

### N3（同上定案段；规格第一节第 1 条后半）

- 分配器（`allocator.rs`）：删掉实九那一套只住内存的隔离位（字段、计数、两个方法、三处空闲判定里的那一项）；`release`（第 901 行）改成
  调新方法 `release_leaving_the_record_allocated_on(placement, 释放代, 留在已分配的那几块盘)`（第 909 行），名单里的盘上那条记录一个字节都不动，
  别的盘照旧改写成已释放 + 释放代、进 defer。`crates/mutations.tsv` 里压着 `release` 与分配器释放那几行（10:0x UTC 那一版的第 24、237、238 行）原样留着。
- 发布（`transaction.rs` 第 3541–3552 行）：对每个要释放的落点，取读盘核交回的清单里同一个落点的那几块盘，交给上面那个方法。
  `released` 仍列逻辑上释放了的全部落点（映射条目这次不再写它们），`quarantined_after_release_checksum_mismatch` 列记录留在已分配的那几份。
- 跨重挂：不另写一位，盘上那棵分配记录树里那条就是已分配；重挂时 `rebuild_from_records` 照已分配标位，回收谓词只看已释放的记录，它永远不被回收。
- 准入：`AdmissionReading::of_allocator` 读 `allocated_slots()`，它就在里面；式子一项都没加（`abandoned_root_exclusive` 读的是影子账那一套，不含它）。
- 停下的一格：读盘核在每个单元两条位置项都核完之后看对不上的是不是池里每一块盘；只一部分对不上就返回
  `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported { unit, devices_whose_copy_failed }`（`transaction.rs` 成员定义紧跟在
  `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided` 后面），在准入、拷分配器与任何一个写之前。
  于是 `release_leaving_the_record_allocated_on` 在产品路径上拿到的名单要么空、要么是池里全部的盘；它仍按盘写，留给主 agent 定了不对称怎么办之后接着用。

### N4（C515，判决 N4）

`allocator.rs` 字段声明、构造、单测断言三处；`scenario.rs` 字段与赋值两处；`first_transaction_on_device.rs` 与
`first_transaction_region_bytes.rs` 两个二进制的结果行（格式串与参数一起删）；四份用例的断言（`first_transaction_step_two_data_unit.rs`、
`first_transaction_step_five_publish.rs` 拆掉元组只留 `key_order_mismatches`、`second_transaction_step_one_overwrite.rs`、
`second_transaction_supplement_two_commit_generated_fallback.rs`）。`first_transaction_step_five_publish.rs` 的 `BuiltPool::allocator` 字段只被那条断言读，
一起删了（留着编译器报 `never read`）。新底上 `first_transaction_on_device.rs` 多了一处对 `MountError` 的穷举 `match`（别的补丁加的），被删的 `InstanceTableChainLongerThanOnePageUndecided` 那一臂一并删掉。

### 实例表第二片写路径（规格第四节；D3 已定项 10 ⑤、D18 已定项 11）

- **角色**：第 0 片仍是 `TransactionUnit::InstanceTable`（根记录持有它），第 1 片起是新成员 `InstanceTablePageAfterTheFirst(片序号)`；
  `TransactionUnit::of_instance_table_page` 按片序号分。没有把 `InstanceTable` 改成带片序号的元组成员，理由：实验装置
  `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 第 160 行 `matches!(unit.identity, TransactionUnit::InstanceTable)` 在那种改法下编译不过，
  而 e156 是实验装置、这一轮不该动它；另外 `crates/mutations.tsv` 里有几行（10:0x UTC 那一版的第 67、76、383、469 行）的替换文写着 `TransactionUnit::InstanceTable` 这个值。
  代价：「一张表的一片」这一个概念在枚举里分成两个成员（第 0 片由根记录持有、第 k 片由上一片的链指针记录持有，释放取指针的来路确实不同）。
- **分片**：`instance_table_rows_of_each_page`（`instance_table.rs` 第 140 行）按 369 行一片 `chunks`，0 行也是一片（空的那一片）；
  片数与 `instance_table_pages_for_rows` 同一个数（单测钉住）。一片的记录 = 这一片的行 + 末尾链指针记录（第 152 行 `instance_table_page_records`）；
  链指针记录的写者 `InstanceTableChainRecord::to_bytes`（第 181 行）：最后一片与 mkfs 写的那 88 字节逐字节相同，别的片写 kind 1、有无下一片 1、下一片的指针。
- **次序与装法**：`PublishPlan::resolve` 与 `PublishShape::rewritten_roles` 在用户数据之后、别的提交内生块之前放
  `instance_table_page_roles_in_bump_order(片数)`，即尾片先、第 0 片最后；落点按这个次序在 `publish_admitted` 里取。
  `build_instance_table_chain`（第 890 行）从尾片装起：每装一片发一个树 0 的出生序号、算它的指针，交给前一片的链指针记录；
  树表（同在树 0）接在各片之后发号。点名项、单元写都按 `rewritten` 的次序，所以也是尾片先。树表 0 条的一版上写行
  （`publish_instance_table_on_version_without_file`）用同一个装链函数，落点次序是各片（尾片先）、分配记录树那一片最后。
- **释放旧链**：旧链的各片指针不在上一版的内存态（`TransactionOutput`）里——根记录只持有第 0 片，第 1 片起在上一片的链指针记录里，
  从盘上重建的上一版、第一个文件版本那一版都不带它们。所以 `InstanceTableRewrite` 带着 `replaced_chain`（挂载时
  `instance_table_chain_of_root` 沿链读出来的 `page_pointers`，与接行读的是同一份），`instance_table_chain_to_release` 对每一片走
  `placement_to_release_after_checking_every_device`（两条位置条目同槽、每块盘在册、没释放过、跨度对得上）。`placements_to_release_via_mapping`
  从此跳过实例表的各片，`crates/mutations.tsv` 里压着那一行调用的两行（10:0x UTC 那一版的第 11、19 行）原样留着；旧链排在释放清单最前（与 bump 次序同序）。
  计划带着的旧链第 0 片必须就是上一版根记录指着的那一片（两处 `assert_eq!`，为什么走不到见第六节末）。
- **取号之前的准入**（`mount.rs` 第 1153 行起）：删掉「这一版已多于一片」「这次之后多于一片」两道拒绝；写行那次的形状改成
  `PublishShape::row_publish_rewriting_instance_table_pages(这次之后的片数)`（`PublishShape::ROW_PUBLISH` 留作一片时的常量，用例在用）；
  另加一道：写行那次要重写实例表时（带文件的一版恒要；树表 0 条的一版要写的行不为空时），先按释放判定路径逐片核旧链，核不过报
  `MountError::RowPublishAdmissionRefusedBeforeAcquisition`（调用在第 1166 行）。这一道是为手搭或外来的多片链：旧链上一片不在这一版的分配记录里时，
  不加它就是取号写完才在发布路径里撞上、号烧掉（`…_instance_table_chain.rs` 第 381 行那条用例就是这个形状）。
- **理想模型**（`crates/singlefs-harness/src/model.rs`）：删掉「多于一片必须拒」那条理由；`next_root` 对写行那两种发布按这次之后的行数
  现算片数（模型自己的 `instance_table_pages_for_rows`，第 116 行，不调实现的），多出来的几片进分配记录与占槽的上界。


### 合并新底时补的一处（接手者，13:3x UTC；不是前任的代码）

主工作区 13:10 打进的补丁在 `mount.rs` 里加了「取号之前在分配器的一份拷贝上把取号之后那一串（写行一次、暖机几次）逐次取一遍落点」
（`placements_of_the_publishes_after_acquisition_on_a_copy`，第 1258 行），发完之后 `establish_instance` 断言拷贝上取的与真发的逐次相同（第 1644 行起那条注释下的 `assert_eq!`）。
它按一片写死：带文件的一版上写行按 `PublishShape::ROW_PUBLISH` 取角色、树表 0 条的一版上写行按「实例表、分配记录树节点」两个角色，真取到的也只从根记录读这两条指针。
这份补丁让写行能写多片之后，不改它的话：370 行起那条断言当场 panic（变异 A15–A17 就是把我的改动退回去，三条都红在它上面）。改了三处，都照发布路径本身：
- 带文件的一版：角色按 `PublishShape::row_publish_rewriting_instance_table_pages(这次之后的片数)`；换下的落点先是整条实例表旧链（`instance_table_chain_to_release`，
  第 1275 行），再接经映射查到的那几个——与 `publish_version` 同序（这份补丁里经映射那一路跳过实例表各片，不补上旧链的话拷贝上少释放那几片，
  回退到环里最旧的根时写行当场回收的那一片拷贝上看不见；变异 A18 红在新底自带的 `rolling_back_to_the_oldest_ring_root_…` 上）。
- 树表 0 条的一版：角色按 `instance_table_page_roles_in_bump_order(这次之后的片数)` 再接分配记录树节点（尾片先，与 `publish_instance_table_on_version_without_file` 同序）。
- 真取到的（`placements_taken_by`，第 1353 行）：树表 0 条那一臂改读这次那条记录的点名项（第 1361 行；第 1 片起的指针不在根记录里，点名项按取落点的次序列全部重写角色），
  零单元发布一项都不点名、交空。
- 函数多一个参数 `&InstanceTableRewrite`（取号之前的准入与写行那次发布读的同一份，第 274 行 `instance_table_rewrite_of_the_row_publish`）。
- 两处注释按 N3 的新语义改：拷贝上不做读盘核、对不上的那一份照常释放，真发时那一份的分配记录留在已分配——两边只在那一槽这一串里被回收时分叉，
  所以「这一串里有一份核出对不上时不比」那条豁免照旧成立。没加新的 `expect` / `assert`；`placements_taken_by` 里那条 `expect` 是新底原有的、文字没动。
- 新底同一处还有一个变异行（主表里名为「增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点…」那一行）
  的原文被我改掉了，给了整行替代（`mutations-replacements.tsv` 最后一行，变异 R529，红在它原来点名的用例上）。

14:50 打进的实二十与这份补丁三方合并零冲突；合并之后我核过的交叉点：树表 0 条那一路我改写过的那条 `JournalRecord` 字面量带上了实二十加的
`place_in_publish: LastRecordOfThePublish`（`transaction.rs` 第 1031 行）；实二十删掉的 `MoreNamedUnitsThanOneJournalRecordHolds` 在合并后的 `crates/` 里零命中。
没接上的一格见第六节 Q7。

## 四、新测试与「改坏哪一行 → 哪条断言红」

新 / 改写的用例（行号是那条 `fn` 在 `work5/` 里的行）：

| 用例 | 文件:行 | 钉住什么 |
|---|---|---|
| Q1 `a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool`（名不变、断言改） | `crates/singlefs-harness/tests/second_transaction_supplement_two_release_checksum_quarantine.rs`:131 | 两份都坏：映射条目去掉、两盘记录 (未释放, 3)、B 写进分配记录树的账里也是、隔离清单两份、槽不回空闲池（复用窗口置 0 下 extent 根那一槽当场回去作对照）、下一个用户数据落点 50182 |
| Q2 `a_checksum_failure_on_only_one_copy_returns_the_unsupported_member_before_anything_is_written`（取代实九的用例 2） | 同文件:241 | 只盘 0 坏：返回 `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported{Data, [盘 0]}`，`DiskSnapshot` 前后相等、分配器记录不变（只钉成员与盘上不变） |
| Q3 `after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy`（名不变，改成挂载之后改坏两份） | 同文件:286 | 重开可写挂载之后两份都改坏再覆盖写：两份都进隔离、两盘记录 (未释放, 3) |
| Q4 `a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual`（新） | 同文件:453 | 盘 0 那一读拒一次：两盘各读 [2, 1] 次、不隔离、两盘照常 (已释放, 4) |
| Q5 `a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch`（新，替掉实九的用例 4） | 同文件:483 | 两盘那一份都一直读不出：各恰读两次 [2, 2]、两份按对不上隔离、两盘记录留在已分配 |
| Q6 `a_quarantined_copy_stays_allocated_across_a_remount_and_is_never_handed_out_again`（新） | 同文件:540 | 两份坏、覆盖写之后重开可写挂载、回收到顶：最新根那棵账里两盘都是 (未释放, 3)、不在空闲池、接连 8 个用户数据落点都不是它且越过了它 |
| Q7 `a_quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_term_of_its_own`（新） | 同文件:606 | 同一段历史两个池（一个改坏两份）、重挂回收到顶：改坏的那个每块盘已分配多两槽、可用少两槽，被抛弃根独占量 0 |
| Q8 `a_mapping_location_on_a_device_outside_the_pool_returns_the_undecided_clause_member_before_anything_is_written`（新） | 同文件:711 | 映射条目位置项指盘 7：返回 `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`，盘上不变、记录不变 |
| S1 `shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it`（名不变，改成两份都改坏） | `…_parallel_line_one_multi_unit_file.rs`:316 | 文件变短换下的尾巴两份都坏：两份都进隔离 |
| W1 `a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it`（新） | `…_instance_table_second_page_write.rs`:155 | 370 行：角色次序 [第 1 片, 第 0 片, t5, t6, t7, t8]、落点递增、出生序号 0 / 1 / 2（第 1 片 / 第 0 片 / 树表）、369 + 1 行、第 0 片链指针指第 1 片、点名项同序、两片分配记录 (未释放, 这次 txg, 跨 2) |
| W2 `cold_start_reads_back_every_row_the_writer_put_on_two_pages_and_the_pool_checker_stays_green`（新） | 同文件:265 | 冷启动沿链读回 1..=370 全部行、`recover` 读回文件、checker 零违例且 I-1.1 / I-2.1 / I-3.1 / I-3.8 判成立 |
| W3 `the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages`（新） | 同文件:298 | 再挂：两片旧链都在释放清单、两盘记录改成已释放、新链 369 + 2、checker 零违例、I-3.9 判成立 |
| W4 `a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both`（新） | 同文件:358 | 树表 0 条的一版上写两片：落点与出生序号 第 1 片 < 第 0 片 < 分配记录树节点、两片在那一版的账里；再挂释放两片与旧节点、checker 零违例 |
| P1 `writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page`（改写，原名 `…_and_the_next_one_is_refused_before_acquisition`） | `…_instance_table_page_full.rs`:96 | 验收那一条：360 次取号之后崩溃 + 10 次挂载，第 10 次取号 371 成立、369 + 1 两片、checker 零违例 |
| P2 `mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page`（改写，原名 `…_may_fill_the_page_but_not_overflow_it`） | 同文件:134 | 369 行一片；370 行两片 |
| P3 `rollback_counts_the_rows_of_the_table_it_rolls_back_to`（名不变、后半改写） | 同文件:170 | 回退写 369 行一片；回退写 370 行两片、首行是回退行、checker 零违例 |
| C1 `writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition`（改写，原名 `writable_mount_counts_the_rows_of_every_page_and_refuses_a_chain_longer_than_one_page_before_acquisition`） | `…_instance_table_chain.rs`:381 | 手搭两片链（第 2 片不在账里）：取号之前 `RowPublishAdmissionRefusedBeforeAcquisition{ReleaseTargetNotAllocated{第 1 片, 盘 0, 60000}}`，盘上不变、号不烧 |
| K1 `a_release_generation_outside_its_interval_on_the_second_page_of_a_replaced_instance_table_chain_reddens_the_release_generation_invariant`（新） | `checker_known_bad_images.rs`:4228 | O3：旧链第 1 片那两条已释放记录的释放代写成那次写行的 txg − 1，只 I-3.9 红；干净镜像上 I-3.9 判成立 |
| M1 `model::tests::a_mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain`（新） | `crates/singlefs-harness/src/model.rs`:1974 | 理想模型：370 行两片，写行那次加 12 条分配记录、占 8 槽 |
| U1 `instance_table::chain_record_tests::rows_fill_a_page_of_three_hundred_sixty_nine_before_the_next_page_opens`（新） | `crates/singlefs-core/src/instance_table.rs`:407 | 0/1/369/370/738/739 行的分片、片数与 `instance_table_pages_for_rows` 相同、链指针记录写了读回、最后一片与 mkfs 那一条逐字节同 |


证法：每条变异在一份独立副本里改一处，跑那条测试所在的**整个**测试二进制，跑完从原件拷回并 `touch`（`run-mutations.py` / `run-mutations-shared-baseline.py`）；
每份副本用自己的 `target/`（`CARGO_TARGET_DIR` 指到副本里），开跑前都用 `rsync -rlpc`（按内容比、不带时间戳，换过的文件拿新的 mtime）从原件同步、`diff -rq` 核过相同。
每条的 cargo 原样输出在 `logs/mutations-worker<k>/mutation-<号>.log`，红的断言原文由 `extract-red.py` 从日志里取。
被测代码（`transaction.rs`、`allocator.rs`、`mount.rs`、`instance_table.rs`、`walk.rs`、`model.rs`）里 `grep -c debug_assert` 都是 0，没另跑 `--release`。

主工作区在这一轮里变了三次，变异分三批证（都是接手者跑的；前任在旧底 `work/` 上跑过同样 34 条也全红，读数不沿用）：

| 批 | 树 | 条数 | 基线 | 结果 |
|---|---|---|---|---|
| worker5a/5b/5c/6/7 | `work2/`（10:0x 主工作区 + 补丁） | 34 条：前任交的全部行 | 各二进制先跑不改动的，17 条 `BASELINE` 行全是 `failed=[]` | 34 红 |
| worker8 | `work4/`（13:32 主工作区 + 补丁，合并时补了 mount.rs 那一处） | 10 条：`mount.rs` 里的 5 条重跑（R25、R68、R105、R109、R421）+ 新加 5 条（R529、A15–A18） | 与同一树上不改动的测试批同时跑，先按空基线判；那一批被我在第 10 个二进制停掉（第五节），前 9 个全绿，这 10 条点名的二进制有 5 个在那 9 个之外 | 10 红 |
| worker9 | `work5/`（14:49 主工作区 + 补丁，交付的就是它） | 同 worker8 那 10 条（主 agent 定：只跑 mount.rs 那几条与新加的几条） | 与第五节 `work5` 那一批同时跑，先按空基线判，那一批零 FAILED 之后成立 | 见下 |

`-p singlefs-core --lib` 那一行 BASELINE 脚本数出 `passed=94`，cargo 自己的 `test result` 是 95 passed——脚本的正则不认带 `- should panic` 的那一行，不是少跑。
worker5 那 34 条逐条与交付的变异行逐字段相同，worker8 / worker9 那 10 条也是（脚本核过）。编号：R / E 后面的数是前任起名时主表的行号，只作编号用；交付物按变异名认。
没在 work5 上重跑的 24 条为什么还算数：补丁的 24 个文件里有 20 个，「我的差」在 work2（对 base2）与 work5（对 base5）上逐行相同（两边 `diff` 的增删行排序后相同）；
另 4 个不同的是合并带来的：`mount.rs`（补那一处，里面的变异行全在 worker8 / worker9 重跑）、`walk.rs`（只差导入行，O3 那段函数与 A13 改的那一处相同）、
`model_comparison.rs` 与 `first_transaction_on_device.rs`（穷举臂，没有变异行）。那 24 条改的那一处、点名的用例都没变；变的是周围合并进来的新底，
第五节 work5 那一批测试二进制（它们点名的用例都在里面）不改动时全绿。

### 最后那一底（`work5/`，交付的就是它）上的 10 条：全红

`grep -cE '^[RAEU][0-9]+\s+RED' logs/mutations-worker9/summary.txt` 数出 10，SURVIVED 0（14:52–15:38 UTC；`work4/` 上 worker8 同样 10 条也全红，`logs/mutations-worker8/`）：

| # | 交付物里 | 结果 | 变异（改坏哪一处） | 必须红的用例：红在哪条断言（原文，截断） | 同一二进制里一起红的 |
|---|---|---|---|---|---|
| R529 | 替代（主表里原名那一行） | RED | 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出） | `a_formatted_pool_mounted_twice_writes_th…` 红在 core/mount.rs:1659:9：assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串 / left: [[(InstanceTable, SlotNumber(50240)), (AllocationTree, SlotNumber(50242))], []] / right: [[(InstanceTable, | 7 条：a_third_writable_mount_on_a_version_whos…、rolling_back_before_any_file_version_iso…、rolling_back_to_a_warm_up_root_before_an…等 |
| A15 | 追加第 18 行 | RED | 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出） | `a_row_publish_past_one_page_writes_the_t…` 红在 core/mount.rs:1657:9：assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串 / left: [[(InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)), SlotNumber(50304)), (InstanceTable, SlotNumb | 2 条：cold_start_reads_back_every_row_the_writ…、the_next_mount_rewrites_the_whole_two_pa… |
| A16 | 追加第 19 行 | RED | 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出） | `a_version_without_file_past_one_page_wri…` 红在 core/mount.rs:1658:9：assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串 / left: [[(InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)), SlotNumber(50240)), (InstanceTable, SlotNumb | 0 条 |
| A17 | 追加第 20 行 | RED | 实例表第二片 × 增补 2 收口表第 39 行那一族：发完之后比对时，树表 0 条那一版上写行真取到的落点只认第 0 片（第 1 片起的点名项被错配到别的角色上，挂载自己的断言判出） | `a_version_without_file_past_one_page_wri…` 红在 core/mount.rs:1660:9：assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串 / left: [[(InstanceTable, SlotNumber(50240)), (AllocationTree, SlotNumber(50242))], []] / right: [[(InstanceTableP | 0 条 |
| A18 | 追加第 21 行 | RED | 实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见） | `rolling_back_to_the_oldest_ring_root_reu…` 红在 second_transaction_supplement_three_random_history.rs:633:5：assertion `left == right` failed: NewFinding { signature: Panic { location: "crates/singlefs-core/src/mount.rs:1656" }, observation: FailureObservation { position: Operation(22), operation_kind: Some(CloseAndMountRollback), violations: [],  | 0 条 |
| R25 | 替代 | RED | 步 3：不写行（实例表照抄） | `remount_takes_instance_two_writes_the_ro…` 红在 core/mount.rs:1660:9：assertion `left == right` failed: 取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串 / left: [[(AllocationTree, SlotNumber(50304)), (AccountingTree, SlotNumber(50305)), (MappingTree, SlotNumber(50306 | 4 条：checker_rejects_an_instance_table_row_wh…、damaging_every_instance_two_root_on_one_…、stray_record_of_the_previous_instance_is…等 |
| R68 | 替代 | RED | 步 3：写行时这次要写的行没接进重写出去的那条实例表链（带文件的一版与树表 0 条的一版共用这一处拼法） | `a_formatted_pool_mounted_twice_writes_th…` 红在 second_transaction_step_three_formatted_pool.rs:1282:5：assertion `left == right` failed: 写行那一条还在：第一个文件版本照抄的是写行之后那一版的指针 / left: [] / right: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(2), applied_transaction_high_water: 0, is_rollback: false }] | 2 条：rolling_back_to_a_warm_up_root_before_an…、writable_mount_after_a_crash_right_after… |
| R421 | 替代 | RED | 实例表第二片：取号之前不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才在发布路径里撞上，号烧掉） | `writable_mount_follows_the_chain_and_ref…` 红在 second_transaction_supplement_two_instance_table_chain.rs:403:18：第二片不在账里：要在取号之前拒绝：Err(Publish(PublishSequenceFailed { cause: ReleaseTargetNotAllocated { unit: InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)), device: DeviceIdentity(0), slot: SlotNumber(60000) }, writes_of_persisted_publishes: [] | 0 条 |
| R105 | 替代 | RED | 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉） | `writable_mount_that_cannot_publish_the_r…` 红在 second_transaction_supplement_two_row_publish_admission.rs:217:18：要在取号之前按写行那次发布的准入拒绝：Err(Publish(PublishSequenceFailed { cause: AllocationRecordsExceedOneNode { records: 822, capacity: 812 }, writes_of_persisted_publishes: [], writes_of_failed_publishes: [] })) / note: run with `RUST_BACKTRACE=1` environm | 2 条：writable_mount_whose_first_warm_up_publi…、writable_mount_whose_second_warm_up_publ… |
| R109 | 替代 | RED | 增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉） | `writable_mount_whose_first_warm_up_publi…` 红在 second_transaction_supplement_two_row_publish_admission.rs:317:18：要在取号之前按暖机那几次空发布的准入拒绝：Ok(Mounted { output: MountOutput { instance: InstanceGeneration(2), chosen_root: RootRecord { filesystem_identifier: [95, 83, 70, 83, 45, 69, 49, 52, 50, 45, 48, 48, 48, 49, 45, 0], instance: InstanceGeneration(1), chec | 1 条：writable_mount_whose_second_warm_up_publ… |

不是红在用例自己断言上的：R529、A15、A16、A17 与 R25 红在 `establish_instance` 里「取号之前在拷贝上取的与真发的逐次相同」那条 `assert_eq!`
（`mount.rs` 第 1660 行起，日志里行号因变异改了上面几行各差一两行）——A15–A17 改的就是拷贝那一侧，那条断言正是它们要钉的；
R25（写行那次实例表照抄）在 work2 上红在用例自己第 138 行的断言，合并新底之后先被那条断言截下（拷贝上按重写实例表取了落点，真发照抄），照样红在点名的用例上。
A18 红在新底自带的用例 `rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition`：
拷贝上不释放旧链时，回退那次挂载拷贝上取的落点与真发的不同，`establish_instance` 那条 `assert_eq!` panic（日志里 `Panic { location: "crates/singlefs-core/src/mount.rs:1656" }`，
行号因变异删了上面几行而前移），用例把它记成一条新发现、在自己第 633 行的断言上红。

### 前 34 条（`work2/` 上，worker5a/5b/5c/6/7）：全红

`grep -chE '^[RAEU][0-9]+\s+RED' logs/mutations-worker{5a,5b,5c,6,7}/summary.txt` 五个数加起来 4 + 4 + 3 + 14 + 9 = 34，SURVIVED 0：

| # | 10:0x 那一版主表的行号（只作参照，交付物按名字） | worker | 结果 | 变异（改坏哪一处） | 必须红的用例：红在哪条断言（原文，截断） | 同一二进制里一起红的 |
|---|---|---|---|---|---|---|
| R25 | 第 25 行（替代） | 6 | RED | 步 3：不写行（实例表照抄） | `remount_takes_instance_two_writes_the_ro…` 红在 second_transaction_step_three_second_instance.rs:138:5：assertion `left == right` failed: 写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9） / left: [AllocationTree, AccountingTree, MappingTree, TreeTable] / right: [InstanceTable, AllocationTree, AccountingTree, MappingTree, TreeTable] | 0 条 |
| R68 | 第 68 行（替代） | 6 | RED | 步 3：写行时这次要写的行没接进重写出去的那条实例表链（带文件的一版与树表 0 条的一版共用这一处拼法） | `a_formatted_pool_mounted_twice_writes_th…` 红在 second_transaction_step_three_formatted_pool.rs:1271:5：assertion `left == right` failed: 写行那一条还在：第一个文件版本照抄的是写行之后那一版的指针 / left: [] / right: [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(2), applied_transaction_high_water: 0, is_rollback: false }] | 2 条：rolling_back_to_a_warm_up_root_before_an…、writable_mount_after_a_crash_right_after… |
| R105 | 第 105 行（替代） | 7 | RED | 增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉） | `writable_mount_that_cannot_publish_the_r…` 红在 second_transaction_supplement_two_row_publish_admission.rs:217:18：要在取号之前按写行那次发布的准入拒绝：Err(Publish(PublishAfterAcquisitionFailed { cause: AllocationRecordsExceedOneNode { records: 822, capacity: 812 }, writes_of_persisted_publishes: [], writes_of_failed_publishes: [] })) / note: run with `RUST_BACKTRACE=1` environment variable | 2 条：writable_mount_whose_first_warm_up_publi…、writable_mount_whose_second_warm_up_publ… |
| R109 | 第 109 行（替代） | 7 | RED | 增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉） | `writable_mount_whose_first_warm_up_publi…` 红在 second_transaction_supplement_two_row_publish_admission.rs:317:18：要在取号之前按暖机那几次空发布的准入拒绝：Ok(Mounted { output: MountOutput { instance: InstanceGeneration(2), chosen_root: RootRecord { filesystem_identifier: [95, 83, 70, 83, 45, 69, 49, 52, 50, 45, 48, 48, 48, 49, 45, 0], instance: InstanceGeneration(1), checkpoint_txg: Checkpoi | 1 条：writable_mount_whose_second_warm_up_publ… |
| R388 | 第 383 行（替代） | 6 | RED | 增补 2 第 9 行 P1（C497）：带文件的一版上重写实例表的发布把实例表挪到树表单元之后取落点 | `c497_every_publish_that_rewrites_the_ins…` 红在 second_transaction_supplement_two_presumed_clause_checks.rs:77:9：带文件的一版上写行：实例表（槽 50308）要排在 AllocationTree（槽 50304）之前 bump / note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace | 0 条 |
| R389 | 第 384 行（替代） | 6 | RED | 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录节点先于实例表取落点 | `c497_every_publish_that_rewrites_the_ins…` 红在 second_transaction_supplement_two_presumed_clause_checks.rs:162:5：树表 0 条的一版上写行：实例表（槽 50242）排在分配记录节点（槽 50240）之前 bump / note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace | 0 条 |
| R421 | 第 416 行（替代） | 6 | RED | 实例表第二片：取号之前不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才在发布路径里撞上，号烧掉） | `writable_mount_follows_the_chain_and_ref…` 红在 second_transaction_supplement_two_instance_table_chain.rs:403:18：第二片不在账里：要在取号之前拒绝：Err(Publish(PublishAfterAcquisitionFailed { cause: ReleaseTargetNotAllocated { unit: InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)), device: DeviceIdentity(0), slot: SlotNumber(60000) }, writes_of_persisted_publishes: [], writes_of_ | 0 条 |
| R433 | 第 428 行（替代） | 6 | RED | 实例表第二片：片数向下取整（370 行算一片） | `writable_mounts_fill_the_instance_table_…` 红在 core/transaction.rs:3531:52：no entry found for key | 2 条：mount_after_crashes_right_after_acquisit…、rollback_counts_the_rows_of_the_table_it… |
| R435 | 第 430 行（替代） | 5a | RED | C394 N3：分配器释放时不看「留在已分配」那几块盘（对不上那一份的记录照样改写成已释放） | `a_released_unit_whose_copies_fail_the_ch…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:173:5：assertion `left == right` failed: 两盘的分配记录都留在已分配、分配代照旧 / left: [(DeviceIdentity(0), true, CheckpointTxg(4)), (DeviceIdentity(1), true, CheckpointTxg(4))] / right: [(DeviceIdentity(0), false, CheckpointTxg(3)), (DeviceIdentity(1), false, CheckpointTxg(3))] | 4 条：a_quarantined_copy_counts_as_allocated_i…、a_quarantined_copy_stays_allocated_acros…、a_release_checksum_read_that_keeps_faili…等 |
| R436 | 第 431 行（替代） | 5a | RED | C394 N3（用户 2026-09-24 定：对不上那一份的分配记录留在已分配）：核出对不上之后照常释放那一份（记录改写成已释放） | `after_rebuilding_the_previous_version_fr…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:334:5：assertion `left == right` failed: 两盘那条都留在已分配 / left: [(DeviceIdentity(0), true, CheckpointTxg(6)), (DeviceIdentity(1), true, CheckpointTxg(6))] / right: [(DeviceIdentity(0), false, CheckpointTxg(3)), (DeviceIdentity(1), false, CheckpointTxg(3))] | 4 条：a_quarantined_copy_counts_as_allocated_i…、a_quarantined_copy_stays_allocated_acros…、a_release_checksum_read_that_keeps_faili…等 |
| R437 | 第 432 行（替代） | 5a | RED | C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池） | `a_release_checksum_read_that_keeps_faili…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:498:5：assertion `left == right` failed: 读不出的那两份按对不上隔离 / left: [] / right: [CopyQuarantinedAfterReleaseChecksumMismatch { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(0), placement: Placement { slot: SlotNumber(50180), span: 2 } }, CopyQuarantinedAfterR | 0 条 |
| A01 | 追加第 1 行 | 5a | RED | C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉） | `a_release_checksum_read_that_fails_once_…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:459:25：重读读得出、对得上：发布照成: ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported { unit: Data(DataUnitIndexInFile(0)), devices_whose_copy_failed: [DeviceIdentity(0)] } | 1 条：a_release_checksum_read_that_keeps_faili… |
| A02 | 追加第 2 行 | 5b | RED | C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试） | `a_release_checksum_read_that_keeps_faili…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:489:5：assertion `left == right` failed: 每块盘那一份读两次（第一次、重读一次），之后不再读 / left: [3, 3] / right: [2, 2] | 0 条 |
| A03 | 追加第 3 行 | 5b | RED | C394 N3：核出对不上之后照常释放那一份，重挂回收之后那一槽被再发出去（隔离不跨重挂） | `a_quarantined_copy_stays_allocated_acros…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:569:5：assertion `left == right` failed: 重挂之后最新根那棵账里，那一槽两盘的记录仍是已分配、分配代 3 / left: [(DeviceIdentity(0), true, CheckpointTxg(4)), (DeviceIdentity(1), true, CheckpointTxg(4))] / right: [(DeviceIdentity(0), false, CheckpointTxg(3)), (DeviceIdentity(1), false, CheckpointTx | 4 条：a_quarantined_copy_counts_as_allocated_i…、a_release_checksum_read_that_keeps_faili…、a_released_unit_whose_copies_fail_the_ch…等 |
| A04 | 追加第 4 行 | 5b | RED | C394 N3：核出对不上之后照常释放那一份，准入里它不再算已分配（两块盘的可用一样多） | `a_quarantined_copy_counts_as_allocated_i…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:638:9：assertion `left == right` failed: 盘 DeviceIdentity(0)：已分配恰好多那一份的两槽：DeviceAdmissionTerms { device: DeviceIdentity(0), capacity: BytesOnOneDevice(3472883712), allocated: BytesOnOneDevice(196608), unreclaimable: BytesOnOneDevice(0), deferred: BytesOnOneDevice(0), | 4 条：a_quarantined_copy_stays_allocated_acros…、a_release_checksum_read_that_keeps_faili…、a_released_unit_whose_copies_fail_the_ch…等 |
| A05 | 追加第 5 行 | 5b | RED | C394 N3（2026-09-24 故障注入快档打中的那一格）：只一部分副本对不上时照做、不在写之前停（两块盘的分配记录从此不对称，冷启动走读判整池失败） | `a_checksum_failure_on_only_one_copy_retu…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:260:5：只有盘 0 那一份对不上 ⇒ 点名「不对称的账第一版不支持」的成员：Ok(TransactionOutput { root: RootRecord { filesystem_identifier: [95, 83, 70, 83, 45, 69, 49, 52, 50, 45, 48, 48, 48, 49, 45, 0], instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4), tree_table: NodePointer { hea | 0 条 |
| A06 | 追加第 6 行 | 5c | RED | C394：位置项指的盘不在池里时不在写之前交回点名条款没写的成员（当成读不出、按对不上处置，却没有那块盘的记录可留） | `a_mapping_location_on_a_device_outside_t…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:726:5：位置项指的盘不在池里 ⇒ 点名「条款没写这一格」的成员：Err(ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported { unit: Data(DataUnitIndexInFile(0)), devices_whose_copy_failed: [DeviceIdentity(7)] }) / note: run with `RUST_BACKTRACE=1` environment variable to display a back | 0 条 |
| A07 | 追加第 7 行 | 6 | RED | 实例表第二片写路径（D3 已定项 10 ⑤，用户 2026-09-24 定尾片先）：各片在 bump 次序里第 0 片先（头片先） | `a_row_publish_past_one_page_writes_the_t…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:163:5：assertion `left == right` failed: 尾片先：第 1 片、第 0 片，之后才是四个固定点单元 / left: [InstanceTable, InstanceTablePageAfterTheFirst(InstanceTablePageIndex(1)), AllocationTree, AccountingTree, MappingTree, TreeTable] / right: [InstanceTablePageAfterTheFirst(InstanceTablePageI | 1 条：a_version_without_file_past_one_page_wri… |
| A08 | 追加第 8 行 | 6 | RED | 实例表第二片写路径（D18 已定项 11，用户 2026-09-24 定一片写满 369 行再开下一片）：一片只写 368 行就开下一片 | `a_row_publish_past_one_page_writes_the_t…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:216:5：assertion `left == right` failed: 第 0 片写满 369 行 / left: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53,  | 1 条：the_next_mount_rewrites_the_whole_two_pa… |
| A09 | 追加第 9 行 | 6 | RED | 实例表第二片写路径：链指针记录「有下一片」写成 0（读者按「无下一片而指针不全零」拒收，两片的表读不出） | `a_row_publish_past_one_page_writes_the_t…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:123:44：那一片按第几片解得开 | 3 条：a_version_without_file_past_one_page_wri…、cold_start_reads_back_every_row_the_writ…、the_next_mount_rewrites_the_whole_two_pa… |
| A10 | 追加第 10 行 | 6 | RED | 实例表第二片写路径（每次写行 COW 重写整条链）：被换下的旧链只释放第 0 片（第 1 片起永远占着） | `the_next_mount_rewrites_the_whole_two_pa…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:319:9：旧链那一片 Placement { slot: SlotNumber(50304), span: 2 } 在这次释放的清单里：[Placement { slot: SlotNumber(50306), span: 2 }, Placement { slot: SlotNumber(50312), span: 1 }, Placement { slot: SlotNumber(50313), span: 1 }, Placement { slot: SlotNumber(50314), span: 1 }, Plac | 1 条：a_version_without_file_past_one_page_wri… |
| A11 | 追加第 11 行 | 6 | RED | 实例表第二片写路径：带文件的一版上写行时不释放被换下的那条旧链 | `the_next_mount_rewrites_the_whole_two_pa…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:319:9：旧链那一片 Placement { slot: SlotNumber(50306), span: 2 } 在这次释放的清单里：[Placement { slot: SlotNumber(50312), span: 1 }, Placement { slot: SlotNumber(50313), span: 1 }, Placement { slot: SlotNumber(50314), span: 1 }, Placement { slot: SlotNumber(50315), span: 1 }] | 1 条：cold_start_reads_back_every_row_the_writ… |
| A12 | 追加第 12 行 | 6 | RED | 实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片 | `a_version_without_file_past_one_page_wri…` 红在 second_transaction_supplement_two_instance_table_second_page_write.rs:414:13：assertion `left == right` failed: 盘 DeviceIdentity(0) 上被换下的那一片改写成已释放 / left: (false, CheckpointTxg(1)) / right: (true, CheckpointTxg(3)) | 0 条 |
| A13 | 追加第 13 行 | 7 | RED | 实例表第二片（实四 O3）：I-3.9 的引用集合不认实例表链第 1 片起的落点（旧链第 1 片的已释放记录被跳过，释放代写错不红） | `a_release_generation_outside_its_interva…` 红在 checker_known_bad_images.rs:3680:5：assertion `left == right` failed: 只有 I-3.9 红：[("I-1.1", Holds), ("I-1.2", Holds), ("I-1.3", Holds), ("I-1.4", Holds), ("I-1.6", Holds), ("I-1.7", Holds), ("I-1.8", Holds), ("I-1.10", Holds), ("I-2.1", Holds), ("I-2.3", Holds), ("I-2.4", Holds), ("I-2.5", Holds | 0 条 |
| A14 | 追加第 14 行 | 7 | RED | 实例表第二片（理想模型）：写行那次发布只按一片数实例表（多出来的几片不进分配记录与占槽的上界） | `a_mount_that_writes_more_rows_than_one_p…` 红在 crates/singlefs-harness/src/model.rs:1993:9：assertion `left == right` failed: 两片实例表加四个固定点单元，每盘一条 / left: 10 / right: 12 | 0 条 |
| U01 | 追加第 15 行 | 7 | RED | 实例表第二片写路径（D18 已定项 11：0 行也是一片）：0 行时切出 0 片（写者按片数取落点、按切片装片，两者不等） | `rows_fill_a_page_of_three_hundred_sixty_…` 红在 core/instance_table.rs:428:13：assertion `left == right` failed: 0 行 / left: [] / right: [0] | 0 条 |
| U02 | 追加第 16 行 | 7 | RED | 实例表第二片写路径：写行那次只给第 0 片取落点（多于一片的链装不出来，一次挂载写 370 行就失败） | `mount_after_crashes_right_after_acquisit…` 红在 core/transaction.rs:3531:52：no entry found for key / note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace | 2 条：rollback_counts_the_rows_of_the_table_it…、writable_mounts_fill_the_instance_table_… |
| U03 | 追加第 17 行 | 7 | RED | 实例表第二片写路径：写行那次只给第 0 片取落点（回退要写 370 行时失败） | `rollback_counts_the_rows_of_the_table_it…` 红在 core/transaction.rs:3531:52：no entry found for key /  / ---- writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page stdout ---- | 2 条：mount_after_crashes_right_after_acquisit…、writable_mounts_fill_the_instance_table_… |
| E11 | 第 11 行（原样） | 7 | RED | 释放退回按提示 | `release_goes_through_the_previous_mappin…` 红在 core/transaction.rs:1858:13：Data(DataUnitIndexInFile(0)) 在上一版的映射里查不到：placements_to_release_via_mapping 刚按同一个上一版查过，查不到时它已经报错返回 | 2 条：release_reports_a_mapping_entry_narrower…、release_reports_a_mapping_entry_whose_sl… |
| E19 | 第 19 行（原样） | 7 | RED | 步 1 变异：不释放旧落点 | `release_rewrites_the_first_versions_reco…` 红在 second_transaction_step_one_overwrite.rs:213:13：A 的落点 50180 改写成已释放 /  / ---- release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking stdout ---- | 7 条：cold_start_reads_the_second_content_and_…、overwrite_publishes_the_second_version_t…、release_goes_through_the_previous_mappin…等 |
| E76 | 第 76 行（原样） | 6 | RED | 增补 1：发布路径漏计实例表单元的写（写行发布按种类的合计对不上录制器） | `writable_remount_row_publish_and_each_wa…` 红在 second_transaction_supplement_one_write_accounting.rs:396:5：assertion `left == right` failed: 取号两盘各一次系统配置槽 + 写行发布 + 各次暖机按种类的合计 == 录制器在挂载这段里记下的写 / left: WriteCallsAndBytes { write_calls: 41, written_bytes: 452096 } / right: WriteCallsAndBytes { write_calls: 43, written_bytes: 517632 } | 0 条 |
| E289 | 第 286 行（原样） | 6 | RED | 步 3（R1）：树表 0 条的一版上写行时，被换下的那片实例表不释放（defer 队列里少 2 槽） | `a_formatted_pool_mounted_twice_writes_th…` 红在 second_transaction_step_three_formatted_pool.rs:1181:9：assertion `left == right` failed: mkfs 的实例表 2 槽 + 树表 1 槽 + 新写的实例表 2 槽 + 这一版自己那片分配记录树 1 槽（C512）；换下的 2 槽在 defer 队列里 / left: (6, 0) / right: (6, 2) | 1 条：the_third_writable_mount_keeps_the_insta… |
| E434 | 第 429 行（原样） | 5c | RED | C394（释放判定不核映射条目位置项里的单元校验和）：释放之前读盘核校验和那一核拿掉（位置项的校验和比不比都当对得上），被改坏的那一份照常回到空闲池 | `a_released_unit_whose_copies_fail_the_ch…` 红在 second_transaction_supplement_two_release_checksum_quarantine.rs:173:5：assertion `left == right` failed: 两盘的分配记录都留在已分配、分配代照旧 / left: [(DeviceIdentity(0), true, CheckpointTxg(4)), (DeviceIdentity(1), true, CheckpointTxg(4))] / right: [(DeviceIdentity(0), false, CheckpointTxg(3)), (DeviceIdentity(1), false, CheckpointTxg(3))] | 4 条：a_checksum_failure_on_only_one_copy_retu…、a_quarantined_copy_counts_as_allocated_i…、a_quarantined_copy_stays_allocated_acros…等 |
| E458 | 第 453 行（原样） | 5c | RED | 并行线一：C394：释放之前读盘核校验和只走这次重写的角色（文件变短换下的尾巴不核，被改坏的那一份照常回到空闲池） | `shrinking_a_multi_unit_file_checks_the_r…` 红在 second_transaction_parallel_line_one_multi_unit_file.rs:334:5：assertion `left == right` failed: 文件变短换下的第 2 个数据单元：两盘那一份都核出对不上，都隔离 / left: [] / right: [CopyQuarantinedAfterReleaseChecksumMismatch { unit: Data(DataUnitIndexInFile(2)), device: DeviceIdentity(0), placement: Placement { slot: SlotNumber(50186), span: 2 } }, Co | 0 条 |

几处不是红在用例自己的断言上的（work2 的行号）：R433、U02、U03 红在 `transaction.rs` 的 `slot_of`（`BTreeMap` 取值；work5 第 3563 行）——变异让片数与取落点的角色数对不上，第 1 片没有落点；
E11 红在 `copies_failing_the_release_checksum_check` 里的 `panic!`（work5 第 1865 行；那条变异把前面的释放判定换掉了，`panic!` 的前提就不在了）；
A09 红在用例自己第 123 行的 `expect("那一片按第几片解得开")`。

**主表要跟着改的**（`crates/mutations.tsv`，我们不改，交主 agent；**全部按变异名认，不按行号**）：
- `mutations-replacements.tsv`：12 行整行替代，每行第一段是主表里现在那一行的变异名、其后六段是替代它的整行。11 行是前任的（其中 5 行变异名也换了），
  最后 1 行是接手者加的（主表里名为「增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点…」那一行，
  它的原文被合并时的那一处改掉了）。
- `mutations-delete.txt`：4 行整行删掉，一行一个变异名（「Z1-a：取号之前不算实例表」「Z1-a：实例表准入漏算链指针」「Z1-a：实例表准入把正好写满一片也拒掉」
  「Z1-a：实例表准入按每次挂载只写一行算」）。它们钉的是「一片装不下就在取号之前拒」，这一轮那道拒绝删了：其中 3 行的原文在 `mount.rs` 里命中 0 次，
  4 行点名的两条用例都已改名（第四节表里 P1、P2）。不删门禁 59 号会红。
- `mutations-append.tsv`：21 行追加到主表末尾（前任的 17 行 A01–A14、U01–U03，接手者的 4 行 A15–A18）。
- 核法：脚本 `check-anchors-by-name.py` 把主表（14:49 那一版，564 行变异）按名字打上 12 行替代、删掉 4 行、接上 21 行之后逐行核「原文在 `work5/` 里恰好命中一次、
  替换文不同于原文、点名的测试 `fn` 在 `crates/` 里有」：581 行零问题；12 个替代名与 4 个删除名在主表里都恰好找得到（主表 564 个变异名没有重名）。

## 五、测试结果

### 交付那一底 `work5/`（接手者，14:52–16:09 UTC，上限 4，`logs/work5-touched-binaries.log`）

fmt / clippy / build（`logs/check-first-three-work5.log`，check.sh 前三步同一条命令与 lint 表，末尾原样）：

```
== cargo fmt --check 14:52:09
fmt exit=0
== cargo clippy 14:52:10
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.99s
clippy exit=0
== cargo build --all-targets 14:52:11
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.30s
build exit=0
== done 14:52:19
script exit=0
```

（这一遍几秒就完是因为 14:51 那一遍已经编过，那一遍红在别的会话正在改的 `e158_root_choice_repair.rs` 上——我拷的是 14:49:49 那一版、别人 14:50:04 又改了；
重拷主工作区那一版之后这一遍全绿。）

测试：26 个 harness 集成测试二进制 + harness lib + 两个 bin 的单测 + core lib + checker lib，全部 `ok`、0 failed、日志里 `FAILED` / `panicked` 零命中
（`grep -E 'Running|test result|exit='` 的输出，只去掉了每个 `Running` 行的前缀与 `target/debug/deps/…` 路径；`harness exit` 之后那个 `src/lib.rs` 是 core，再往后是 checker）：

```
src/lib.rs
test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.04s
src/bin/first_transaction_on_device.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s
src/bin/first_transaction_region_bytes.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
tests/checker_known_bad_images.rs
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1001.00s
tests/first_transaction_step_five_publish.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 262.20s
tests/first_transaction_step_two_data_unit.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 85.07s
tests/second_transaction_mapping_node_admission.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 216.61s
tests/second_transaction_parallel_line_one_last_record_flag.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 310.08s
tests/second_transaction_parallel_line_one_multi_unit_file.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 229.68s
tests/second_transaction_parallel_line_three_many_inodes.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 164.12s
tests/second_transaction_step_four_rollback.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 422.71s
tests/second_transaction_step_one_overwrite.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 291.12s
tests/second_transaction_step_three_formatted_pool.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 265.41s
tests/second_transaction_step_three_second_instance.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 203.77s
tests/second_transaction_supplement_one_write_accounting.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
tests/second_transaction_supplement_three_bad_disk_input.rs
test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 28.71s
tests/second_transaction_supplement_three_crash_injection.rs
test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 18.65s
tests/second_transaction_supplement_three_fault_injection.rs
test result: ok. 9 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 79.91s
tests/second_transaction_supplement_three_random_history.rs
test result: ok. 21 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 297.44s
tests/second_transaction_supplement_two_c533_row_publish_record_without_its_root.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.47s
tests/second_transaction_supplement_two_commit_generated_fallback.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 83.37s
tests/second_transaction_supplement_two_instance_table_chain.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s
tests/second_transaction_supplement_two_instance_table_page_full.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.02s
tests/second_transaction_supplement_two_instance_table_second_page_write.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
tests/second_transaction_supplement_two_presumed_clause_checks.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 215.93s
tests/second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.02s
tests/second_transaction_supplement_two_release_checksum_quarantine.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 140.46s
tests/second_transaction_supplement_two_row_publish_admission.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.03s
tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 172.80s
harness exit=0
src/lib.rs
test result: ok. 99 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s
core exit=0
src/lib.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
checker exit=0
```

点名跑的 26 个集成测试二进制（逐个 `--test`）：这份补丁动到的 11 个用例文件各自的二进制，加上被补丁改过的 core / checker 代码经过、且在这一轮之前就有的
`second_transaction_mapping_node_admission`、`second_transaction_step_three_formatted_pool`、`second_transaction_supplement_one_write_accounting`、
`second_transaction_supplement_three_{bad_disk_input,crash_injection,fault_injection,random_history}`、`second_transaction_supplement_two_{presumed_clause_checks,row_publish_admission}`，
以及两次合并进来的新底改过或新加的 `second_transaction_step_four_rollback`、`second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit`、
`second_transaction_supplement_two_tree_nodes_and_the_central_mapping`、`second_transaction_parallel_line_one_last_record_flag`、`second_transaction_parallel_line_three_many_inodes`、
`second_transaction_supplement_two_c533_row_publish_record_without_its_root`。名字带 layer0 的一个都没点（第一次起这一批时写的是不挑目标的写法，
项目 hook `heavy-test-guard.sh` 当场拒了，改成逐个点名）。

### 之前那几底上的读数（留作「合并之前也绿」的证据）

- `work2/`（10:0x 主工作区 + 补丁）：前任 10:07–10:43 跑完 7 个（harness lib 68、两个 bin 12 / 0、`checker_known_bad_images` 28、`first_transaction_step_five_publish` 8、
  `first_transaction_step_two_data_unit` 3、`second_transaction_mapping_node_admission` 4，全 ok，`logs/rebased-touched-binaries.log`）；接手者 10:53–13:30 跑完其余 16 个与
  **层 0 五个流的快档**（`first_transaction_step_seven_layer0` 5 passed / 1 ignored、`second_transaction_parallel_line_one_layer0` 1 / 1、
  `second_transaction_step_three_acquisition_barrier_layer0` 1、`second_transaction_step_three_formatted_pool_layer0` 3、`second_transaction_step_zero_layer0` 8 / 1），
  core lib 95、checker lib 3，全 ok（`logs/rebased-touched-binaries-part2.log`）。那时主 agent 还没定层 0 快档不跑。
- `work3/`（13:08 主工作区 + 补丁）：fmt / clippy / build 全绿（`logs/check-first-three-work3.log`）。
- `work4/`（13:32 主工作区 + 补丁）：fmt / clippy / build 全绿；动到的测试二进制跑到第 10 个（前 9 个全 ok）时被我停掉——主工作区 14:50 又变了，
  这一底的读数不再服务于交付（`logs/work4-touched-binaries-stopped-on-rebase.log`）。更早一次带层 0 的那一批在主 agent 定「层 0 快档不跑」之后停掉
  （`logs/work4-touched-binaries-aborted-with-layer0.log`）。两次都是连子 shell、cargo 与正在跑的测试进程按写死的 pid 一起停，事后核过没有留下的进程。

### 补丁对主工作区现状

`git apply --check`（16:10:06 UTC，`e980a21` 之上的主工作区）exit=0。那一刻主工作区 `crates/` 与 `base5-crates/` 逐文件比（sha256），只差别的会话在改的
`crates/mutations.tsv`（又追加了几行）与 `e158_root_choice_repair.rs`，补丁都不碰。拿那一刻的主表与 e158 核变异交付物：583 行里只有 1 行问题，
是 E158 那一族一行（原文在 16:06 被别的会话改过的 `e158_root_choice_repair.rs` 里命中 0 次），不在这份补丁碰的文件里、不是这一轮的。

## 六、停下交主 agent 的设计问题

**Q1（N3 × checker 的 I-3.11 / I-3.1）记录留在已分配之后，池级 checker 在健康的历史上判红。**
- 现象（探针 `probe/crates/singlefs-harness/tests/zz_probe_quarantine_checker.rs`，不进补丁；日志 `logs/probe-quarantine-checker.log`、对照 `logs/probe-quarantine-checker-control.log`）：
  改坏 A 的数据单元、覆盖写一次之后，I-3.11 当场红：「盘 0：记账的已分配 Some(376832) 减 defer 待释放 Some(147456)，不等于从最新根（txg 4）走读到的 196608」——
  差的正是留在已分配的那 2 槽（没被改坏的对照镜像上这一格不红）。I-3.1（已分配 = 候选根引用的并集）要等那一份的最后一条引用根离开候选集之后才红；
  探针里它被一个既有的红盖住了——对照（不改坏、同样 30 次覆盖写）也在 I-3.1 上红（mkfs 同一个进程那条会话没装根环表，环转过一圈不回收，不是这一轮的事），所以 I-3.1 那一半这一轮没量出独立的数。
  同一份镜像上 I-2.1 / I-4.8 / I-7.4 也红，那是故意改坏的那一份本身（候选根 txg 3 还引用它），与 N3 无关。
- 条款：规格第一节第 1 条只说「落盘即跨重挂，准入里照已分配算、不另进式子」，没说 I-3.1、I-3.11 怎么认这种「已分配、却没有任何根引用」的记录；
  checker 分不开它与泄漏（能想到的判据——那一份读不出或自证不过就豁免——本身是个设计判断，而且读不出可能是瞬时的）。
- 我没动 checker 的 I-3.1 / I-3.11，也没在新用例里跑 checker（Q1–Q8 都不跑）。要定的：这两条不变量的读法（豁免怎么认、要不要另记一处见证），以及 C394 的检查那一半（会红的镜像）怎么写。

**Q2（N3 × 两块盘的账对称）只一块盘那一份对不上时，按份留在已分配会让两块盘的分配记录不对称，第一版有两处容不下；我停在写之前。**
- 证据一（全量测试，`logs/modified-test.log`，停下之前那一版）：`second_transaction_supplement_three_fault_injection` 的
  `fault_injection_fast_tier_returns_errors_instead_of_panicking` 与 `one_fixed_history_injects_twelve_faults_and_none_of_them_panics` 两条判红，
  原文：「种子 7463871032432355113 read_returns_corrupted_bytes：整池第 786965 次read，落在第 7 步（PublishOverwrite）：CheckerViolations { invariants: ["I-3.11"] }」，
  「重开：OutsideEverythingTheModelAllows（走读失败：InvariantViolated { invariant: "E142 走读同款", detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点" }」。
  也就是：覆盖写的读盘核读到一份坏字节 → 只那块盘的记录留在已分配 → 之后冷启动走读（`recovery::allocation_records_are_one_per_device`，`recovery.rs` 第 1981 行）判整池走读失败。
  不对照的基线（`logs/baseline-test.log`）里这两条是绿的。
- 证据二（停下之前那一版的用例，已删）：只盘 0 那一份隔离、复用窗口置 0 下发布之后，下一次覆盖写
  `PlacementRefused{Data, UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported{[(0, 50182), (1, 50180)]}}`：盘 0 那条记录永远不释放、
  盘 1 那一槽永远不会被两盘一致地发出去，只要它是两盘里最低的空槽对，用户数据写就一直被拒。
- 条款：判决 N3 写「选记录留在已分配时 E1 照留」、规格写「在它那块盘上的分配记录留在已分配」，按份留下是定了的；没写的是之后两块盘的账不对称时，
  冷启动走读的那道判与「各盘一致才分配」怎么容下它（放宽走读那道判要说清哪种不对称合法、checker 怎么认；或者两块盘一起留；或者等 D2 已定项 10 每副本各落点）。
- 我的处置：只一部分副本对不上 ⇒ 在任何写之前返回 `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（用例 Q2 钉住成员与盘上不变）。
  代价：这时那个文件覆盖写不成（判决 N1 表里「乙」那一格的后果），直到那一份修好或两份都坏；两份都对不上、都读不出时照定案做。
  N1 的「先重读一次」仍然挡掉瞬时读错（用例 Q4）。这一处请主 agent 定了之后改掉这个成员。

**Q3（N1 的边角）位置项指的盘不在池里。** `PoolReader::read` 对不在池里的盘也交 `None`，按新条款它会被当成「读不出 → 按对不上 → 那一份在它那块盘上的记录留在已分配」，
可池里没有那块盘、没有那条记录；池里那块没被任何位置项点到的盘上的那一份也从来没核过。我在任何写之前返回
`ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`（用例 Q8）。只有坏镜像、外来镜像走得到。
同类的另一格我没做：两条位置项点了同一块池里的盘（I-2.5 那一类），另一块盘那一份照样没核就释放。

**Q4 从盘上重建的上一版只带实例表第 0 片。** `recovery::rebuild_version`（实十四在改，我没碰）把实例表只作为一个角色放进 `units`；多片的表重建出来之后
`TransactionOutput::units` 缺第 1 片起的那几片。释放不依赖它（旧链从计划带进来），产品路径里重建出来的一版只当写行那次发布的上一版、不被照抄，所以今天不出错；
但「这一版全部角色的单元」那句文档对它不成立。要不要在 `rebuild_version` 里沿链读进各片，交实十四那边一起定。

**Q5 取号之前的准入没罩到的一格（多片之后才走得到，前任没做）。** 树表 0 条的一版上写行，分配记录树节点那一条的释放核与分配记录条数准入
仍在取号之后（`publish_instance_table_on_version_without_file` 里），多片只让条数多几条，是旧缺口。
前任原写的另一格「写行那次末条要点名的项多于 67 就返回 `MoreNamedUnitsThanOneJournalRecordHolds`、号会烧」已不成立：实二十 14:50 UTC 打进主工作区，
那个成员删了，带文件的一版上最后一个事务装不下一条记录时再跨记录（`roles_named_by_each_record_of_the_publish`，`transaction.rs` 第 3130 行）。

**Q7（接手者合并实二十时看到的；走得到、会 panic、我没处理）树表 0 条的一版上写行只写一条记录，点名项 = 片数 + 1，多于 67 项时这一条写不下。**
- `publish_instance_table_on_version_without_file` 那一路（`transaction.rs` 第 1022 行那条 `JournalRecord`）一次发布只写一条记录、点名全部重写角色。
  这份补丁让它能写多片之后，片数 ≥ 67（这一版的行数 + 这次写的行数 > 66 × 369 = 24354）时点名项 ≥ 68，`JournalRecord::to_bytes`
  （`journal.rs` 第 187 行起）往 4096 字节的定长缓冲里写第 68 项时越界，`ByteWriter::put`（`bytes.rs` 第 18 行）切片 panic。
- 走得到：树表 0 条的一版上反复「取号之后崩溃」，下一次挂载给中间那些实例各补一行，约两万四千次取号就到。补丁之前这一版多于一片就在取号之前拒，
  所以这一格是这份补丁打开的；带文件那一路没有这个问题（实二十做了再跨记录）。
- 条款有（D23 已定项 17「末条再跨记录」，用户 2026-09-24 定），这一路没接上。两种改法都要设计判断，我没替它定：
  在取号之前按片数拒（新成员，点名「这一路没实现再跨记录」）；或把这一路接到 `roles_named_by_each_record_of_the_publish` 与实二十的多条记录写法上
  （再跨的那几条的事务号、发布边界怎么认照带文件那一路）。用例与变异都没有。

**Q6（小）** `InstanceTablePlan::Rewrite` 带 `replaced_chain` 是我定的形状：旧链第 1 片起的指针不在上一版的内存态里，释放要从拼行的那一份读取来。
另一种做法是发布路径自己再沿链读一遍盘（多一次读、判定与写读两份输入），或把整条链记进 `TransactionOutput`（第一个文件版本那一版拿不到链，要多读盘）。

### 断言、`expect` 与 `unreachable!`：为什么走不到

- `transaction.rs` 两处 `assert_eq!(…replaced_chain.first(), Some(&…root.instance_table))`（第 2909 行、第 722 行）：
  产品路径只有 `mount::establish_instance` 拼 `InstanceTableRewrite`，旧链取自 `rebuild_previous_version` 里 `instance_table_chain_of_root(devices, root)`，
  上一版取自同一个 `root`（带文件的是 `rebuild_version(devices, root, …)`，它交出的 `output.root` 就是 `*root`；树表 0 条的直接是 `root`）。用例里手拼的一处（`…_second_instance.rs` 的坏行用例）照写第 0 片。
- `build_instance_table_chain` 里 `slot_of(role)`（`BTreeMap` 取值）与两处 `.expect("一条链至少一片…")`：落点按 `instance_table_page_roles_in_bump_order(instance_table_pages_for_rows(行数))` 取，
  装片按 `instance_table_rows_of_each_page(行)` 切，两者片数相等（单测 U1 钉住，变异 R433 / U01–U03 就是把它们弄不等之后红在这里）；0 行也切出一片。
- `instance_table_page_records` 的 `assert!`（行数 ≤ 369）：唯一调用方按 `chunks(369)` 切的片。
- `key_tail_of` 与树表 0 条那一路的两处 `.expect(…实例表…)`：点名的实例表角色都是这次按同一张行表取了落点、装了片的。
- 这一轮删掉了 `publish_admitted` 里原有的 `unreachable!("重写时上面一定装了单元")`，没新加 `unreachable!` 或 `todo!`。

### 加了的非分支项（逐项）

- `InstanceTablePageIndex` 的 `Debug, PartialEq, Eq, PartialOrd, Ord, Hash`：它进了 `TransactionUnit`，那个枚举 derive 了这几样，被迫加。
- `TransactionUnit::of_instance_table_page`、`is_a_page_of_the_instance_table`（分片序号 → 角色、判是不是实例表的一片，调用点在发布与释放路径里）。
- `InstanceTableRewrite::pages_after_this_publish`（片数，准入与角色清单两处用）、`InstanceTableChainRecord::to_bytes`（链指针记录的写者，读者已有）。
- `PublishShape::row_publish_rewriting_instance_table_pages`（`ROW_PUBLISH` 改成它的一片取值）、模型的 `ModelPublishKind::rewrites_the_instance_table`。
- 没有新的 trait 实现；`history.rs`、`model_comparison.rs`、`write_accounting.rs`、两份用例里补的臂是穷举 `match` 逼出来的。


## 七、N4（C515）要跟着改、我们不改的几处（行号前任开工时现查，接手者 14:5x UTC 在主工作区重查过，没变）

- `.claude/gate.d/55-qemu-first-transaction.sh` 第 183 行：期望串 `name=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN`
  删成 `name=transaction key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN`——真设备二进制那一行现在打的就是
  `name=transaction key_order_mismatches={} root_txg={} back_chain={}`（`first_transaction_on_device.rs`）。
- 同一道门禁的 10 份 fixture（`.claude/gate.d/fixtures/55-qemu-first-transaction.sh/{red,green}/{direct,second-instance,skip-first-transaction-barrier,second-transaction,page-cache}/out.txt`，
  `grep -rln policy_mismatches .claude/gate.d/fixtures` 现数 10 份）第 18 行 `E7RESULT name=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=628216162`
  删成 `E7RESULT name=transaction key_order_mismatches=0 root_txg=3 back_chain=628216162`，期望串与 fixture 一起改，门禁 55 号的判别力 fixture 才照旧成立。
- `.claude/kb/verification-build.md` 第 76 行（「用户数据的落点与政策函数不一致的次数」那一行）：判决写「那一行」，按 C515 删掉，或按门禁 39 号的口径改写成「已删，见 C515」；
  第 69 行那一句里「落点政策复算 `allocator::PoolAllocator::policy_mismatches`」也要跟着改（它说「四行里两行有实现」）。
- `.claude/kb/checks-owed.md` 第 454 行 C515：还账，还清方式写「`crates/` 里 11 处同步删、门禁 55 号期望串与 fixture 同步改」。
- 产物：E142 的复跑（`research/scripts/replay.sh` 第 157 行 `E142|@driver_e142||…decisions-catchup.out|exact`）跑的是 `first_transaction_region_bytes`，
  它 `name=impl_config` 那一行末尾的 ` policy_mismatches=0` 没了，逐字节比对会差这一行，E142 要重跑一次、换产物。
  E152 的实验页（`.claude/kb/experiments/152-…md` 第 246 行）贴的是旧结果行，E152 按要求才跑，不用动。


## 八、主工作区中途变了三次：怎么同步的

1. **10:0x（前任）**：开工时的底 `base-crates/`（05:3x）到 10:0x 主工作区 `crates/` 变了 24 个文件，旧底补丁在 `allocator.rs`、`mount.rs`、`transaction.rs` 打不上。
   前任把主工作区当时的样子拷成 `work2/`、`crates/` 另存 `base2-crates/`，改过的每个文件用 `git merge-file <新底> <旧底> <我的>` 三方合并，冲突 5 处
   （`mount.rs` 3 处、`allocator.rs` 1 处、`transaction.rs` 1 处，都是导入行、文档注释与写行两臂的新形状），`first_transaction_on_device.rs` 多删一臂。
2. **13:10–13:11（接手者处理）**：主工作区又打进两份补丁（13:32 时主工作区 `crates/` 相对 HEAD 有 20 个文件改动，含 `mount.rs`、`recovery.rs`、`walk.rs`），13:32 核 `git apply --check` 打不上
   （`walk.rs`、`mount.rs`、`first_transaction_on_device.rs`、`history.rs`、`model_comparison.rs`）。拷成 `work4/`、`base3-crates/`，按 `<base3> <base2> <work2>` 三方合并：
   冲突 5 个文件 6 处——`walk.rs` 导入行（取并集：新底的 `check_unit` 与我的 `packed_unit_view`）、`mount.rs` 导入行（取并集）与 `MountError` 成员
   （留新底新加的 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`、删这份补丁删的 `InstanceTableChainLongerThanOnePageUndecided`）、
   `first_transaction_on_device.rs` / `history.rs` / `model_comparison.rs` 的穷举臂（同样留新成员、删旧成员）。另有一处不是文本冲突、是语义冲突：
   新底取号之前在分配器拷贝上走一遍那一串的代码按一片写死，按多片改了（第三节末「合并新底时补的一处」）。
3. **14:49–14:50（接手者处理）**：实二十打进主工作区（21 个文件，含 `transaction.rs`、`journal.rs`、`mount.rs`、`recovery.rs`、`walk.rs`，`crates/mutations.tsv` 删 3 行、追加 28 行）。
   拷成 `work5/`、`base5-crates/`，按 `<base5> <base3> <work4>` 三方合并：零冲突。别的会话 14:50:04 又改了 `e158_root_choice_repair.rs`（我拷的是 14:49:49 那一版，编不过），
   重拷之后 `base5-crates/` 与主工作区逐文件相同（14:52:09 UTC，sha256 全表比对）。交付的补丁就是 `work5/crates` 对 `base5-crates/` 的差。
- 核对：补丁碰的仍是同样 24 个文件（`work5` 对 `base5` 的逐文件 sha256 差集与 `work4` 对 `base3` 的相同）；「我的差」在其中 20 个文件上三个底逐行相同，另 4 个见第四节。
- 变异表：交付物改成按变异名认（主 agent 14:5x 要求），不再给行号。

## 九、没做什么

- 没走三方对抗；层 0 全量、崩溃注入与故障注入的大档、QEMU、herd7、`crates/mutations.tsv` 整表（门禁 59 号）归 `crash-verifier`，都没跑。
  写行路径多片之后的层 0 流一条都没加（现有的流都到不了 370 行，要写多片的流得先把表推过一片，哪几步、哪几个崩溃点值得枚举交 crash-verifier 定）。
- **层 0 快档留给全部代码落定之后统一跑**（主 agent 13:47 定）。按这条定之前，接手者 10:53 在 `work2/` 上那一批里跑过层 0 五个流的快档（全绿，第五节），
  `work4` / `work5` 上没跑；`work5` 上起测试时项目 hook（`heavy-test-guard.sh`）拒了一次不挑目标的写法，改成逐个 `--test` 点名之后才起。
- **`cargo test --all`（check.sh 第四步）没跑**，留给全部代码落定之后统一跑（主 agent 13:1x 定）；check.sh 的前三步照跑（第五节）。
- 登记给我的门禁阶段按原派发没跑。
- 没动主工作区、没提交；`crates/mutations.tsv` 没改（要替代 / 删 / 追加的在三个交付文件里，按变异名给）。
- kb、`research/`、门禁 55 号与它的 fixture 一个字没写（第七节写了要改成什么）。
- 没碰 `recovery.rs` 的 `rebuild_version`（Q4）、没改 checker 的 I-3.1 / I-3.11（Q1）、没放宽冷启动走读那道对称判（Q2）、没接树表 0 条那一路的再跨记录（Q7）。
- 没替 Q5 那一格补取号之前的准入；没做「两条位置项点了同一块池里的盘」那一格（Q3 末句）。
- 探针 `probe/…/zz_probe_quarantine_checker.rs` 与它的日志只在草稿目录，不进补丁；全量日志、变异日志、测试批日志都在 `logs/` 下，没入库
  （实现员的写范围不含 `research/results/`；要留的话主 agent 拷，都在 `/tmp/claude-1000/impl-m2-writepath/logs/`）。
- 接手者停掉过自己起的两批测试（第五节），都按写死的 pid；别的进程一个没碰。
