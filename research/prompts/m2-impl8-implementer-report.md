# 实八报告（implementation-writer，接手 a710b10cf8d3e4f74；2026-09-26 写）

## 结论
- 三件都做完、各带用例、证过会红，`crates/mutations.tsv` 末尾追加 13 行（第 813–825 行），4 行证过、9 行留给门禁 59 号。
- 动到的、名字不带 layer0 的测试二进制：崩溃注入那个二进制里只剩快档那一条红，红的正是规格第 4 条点名的 C554 那一形；其余 5 个二进制全绿。
- 出口里「build 过」**这一刻不成立，但不是这一轮的改动造成的**：另一个会话在改 `crates/singlefs-harness/src/layer0_progress.rs`（未跟踪文件，加了 `shard` 字段与 `RunOneShardKeepingProgressFile` / `MergeShardLedgers` 两个成员），`crash.rs` 第 1884、2220 行（层 0 续跑那一段，不是我改的地方）跟着编不过。门禁 74 号与 33 号也红在这里。以前（那个会话改之前）我这边 clippy、release 编译都过，6 个二进制也在那时跑过。
- 推翻条件：崩溃注入快档种子基 + 2 或故障注入大档种子基 + 116 在当前构建上复跑又出新发现；或者 13 行变异里有一条被 59 号判「没红」。

## 前任做到哪、我接着做了什么
前任已经做完 (b)(d) 的模型、胶水、执行器接线、按注入点认的判定、`inject_one_fault` 里的计算，还有一条 lib 单元测试。我核过这些，一处没改。它的最后一句「Now the computation in inject_one_fault」说的那段已经在文件里、编得过。
没做的是：(c) 整件、三件的固定用例、变异行、收尾那几样。这些我补齐了。

## 三件各怎么做的
- **(b)**（前任写）：模型的 `ObservedOutcome::Refused` 带上实现点名的读坏槽（`model.rs` 第 1610 行：只有拒之前一个写都没发、一次发布都没做时才带）；胶水 `root_ring_slot_still_bad_after_one_reread_of_mount_error`（`model_comparison.rs`）；按注入点认多两格：「模型说该成、实现拒了」和「拒绝的理由」，实现点名的槽要在被吞根槽写的槽集合里（`fault_injection.rs` `left_by_the_lying_device`）。
  固定用例（我写）：`second_transaction_supplement_three_fault_injection.rs:984` `floor_raise_in_the_same_process_after_a_swallowed_root_slot_write_is_refused_for_that_slot_and_recognised_as_left_by_the_lying_device`。前半截不走按注入点认，钉住实现报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名的正是 txg 7 那一槽（区域 1 槽 2）；后半截走 `inject_one_fault_into_the_segment`，新发现 0，这一格按说谎设备认下 1 次。
- **(d)**（前任写）：`ModelRoot` 加两个字段 `highest_transaction_number_of_its_publish` / `highest_transaction_number_in_its_instance`（`model.rs:210`；写文件内容的发布取实例内下一个号，换实例从 0 数，空发布、写行、暖机、抬 F、回退一律取 0，与 core `transaction.rs:4310`、`:4363`、`mount.rs:1884` 对得上）；新加 `answer_mount_writable_with_the_roots_rebuilt_from_their_records`（`model.rs:1189`），W 取施加的那几次发布的最大事务号。
  固定用例（我写）：同一个文件第 1194 行 `the_rows_written_after_a_swallowed_root_slot_write_are_recognised_by_the_rows_the_model_recomputes_with_the_record_applied`。原来那条 `mount_after_…_while_the_model_expects_none_applied` 留着、只改了文档注释：它钉的是不按注入点认时对拍看到的样子。
- **(c)**（我写）：checker 加 `InstanceTableOfRootRecord`（`crates/singlefs-checker/src/walk.rs:2956`，`read` 沿用 I-7.9 那份只读不判的链读法，`abandons` 在 `:2970`，判据是有行 (i, Ti) 且 T > Ti）。记录核对器 `check_records_against`（`crash.rs`）第二条判据加一个条件：一次发布被恢复落到那一版的实例表判为被抛弃时，不要求它的单元还在。实例表从哪来：取记录流里写出 `effective_root` 那个根身份的最后一次根槽写的字节，读它的实例表指针，再在崩溃后镜像上沿链读（`instance_table_of_the_effective_root`）；读不出就一次发布都不豁免。签名改成泛型 `Reader: PoolReader + ImageReader`，两个调用方传的都是 `CrashImage`，调用处一个字没改。
  诊断：在副本里打印过，种子基 + 2 那 3 个红状态豁免的都是 (1, 4)（历史第 3 步崩溃恢复抛弃的那次覆盖写），落到的是 (3, 11) / (5, 15) / (5, 20)，实例表行 (1, 3, 0)，缺席的单元在偏移 822181888。
  固定用例：`second_transaction_supplement_three_crash_injection.rs:472` `units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`。前半截是整条流都落盘的状态：(1, 4) 的单元确实被盖掉了，记录核对器一条不红。判别力那一半：把 (1, 3)（正卡在 T = Ti 的边上）和最后一次覆盖写各自的数据单元两份都扣下，各判一次红。

## 复现种子复跑（主工作区，release）
- (b) 故障注入大档种子基 + 116（`SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355229 SINGLEFS_FAULT_INJECTION_SEEDS=1`），日志 `/tmp/claude-1000/impl-rbf-8/logs/repro-b-after.log`，原样：
  `说谎的设备丢掉一份内容之后盘面不一致：1 次（设备丢的，不算新发现）` / `    ModelDisagreement { aspect: "拒绝的理由" }：1 次` / `以「已知红」收尾 {}、新发现 0` / `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 2.86s`
- (c) 崩溃注入种子基 + 2（`…115`，24 步，4 个崩溃状态），日志 `/tmp/claude-1000/impl-rbf-8/logs/repro-c-after.log`，原样：
  `崩溃状态上 checker 跑了 4 次；记录核对器跑了 4 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次` / `崩溃状态：以已知红收尾 {}、新发现 0` / `test result: ok. 1 passed; …`
  （改之前前任的日志 `repro-c-seed115.log` 里是「恢复自称新态而单元缺席 3 次」「新发现 3」。）

## 第 4 条：崩溃注入快档还红的那一形
`crash_injection_fast_tier_recovers_only_into_versions_the_model_committed` 还红，清单外只剩 1 个新发现（`/tmp/claude-1000/impl-rbf-8/logs2/touched/crash-injection.log` 第 162、209–210 行，原样）：
```
崩溃状态：以已知红收尾 {}、新发现 1
崩溃状态上的新发现 ModelDisagreement { aspect: "冷启动读回" }：种子 7463871032432355129，第 17 段（1 个写，持久 0 个，扣下段内第 [0] 个），落在 Operation(3)／Some(CrashRecoveryAbandoningTheNewestRoot)
  恢复读回：Failed { what: "MappingStillUnreadable { slot: SlotNumber(50240) }（实际走的根 ModelRootKey { checkpoint_txg: ModelCheckpointTxg(3), instance: ModelInstanceGeneration(1) }）" }
```
这就是实七报告第 1 条的那一形（崩溃注入快档种子基 + 16，崩在抛弃根那一步里，走读失败 MappingStillUnreadable 50240），也就是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）。同一个状态上 checker 还判红 I-2.1、I-3.1、I-3.11、I-4.8、I-7.2、I-7.4，记录核对器判红 claimed_state_missing_unit，全是这一形连带出来的。没有 ignore，判据没放宽。

## 证红（第 3 步；副本 `/tmp/claude-1000/impl-rbf-8/copies/proof`，自己的 target，release，经内存包装；已删）
基线（不改动的副本）：故障注入二进制 13 绿；harness `--lib` 78 绿；崩溃注入二进制红 1 条 `crash_injection_fast_tier_…`（C554 那一形，记进基线红集）。每条变异跑的是那条测试所在的整个二进制；还原的办法是从原件拷回再 `touch`，四份被改的文件都和主工作区逐字节比过 `same`（`proof-logs/restored.txt`）。

| 变异（表里的行） | 改坏哪一行 | 哪条断言红 | 基线之外同时红的 |
|---|---|---|---|
| 第 813 行 M1 | `history.rs` 抬 F 那条路：`root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),` 改成 `None,` | `floor_raise_in_the_same_process_…`（fault_injection 测试文件第 1064 行）`实现点名的读坏的槽是 txg 7 那一次被吞的根槽写的槽`，left None / right `Some(ModelRingPosition { region: 1, slot_in_region: 2 })` | 无（12 过 1 红） |
| 第 815 行 M2b | `fault_injection.rs` `.contains(&ring_slot)` 后接 `\|\| true` | lib `fault_injection::tests::a_refused_floor_raise_…`（fault_injection.rs:3314）`RefusedWhenModelRequiresSuccess：读坏的槽不是被吞那次写的` | 无（77 过 1 红） |
| 第 819 行 M5 | `model.rs` `Some(_) => highest_transaction_number_before_this_publish + 1,` 改成 `Some(_) => 0,` | `the_rows_written_after_…`（fault_injection 测试文件第 1214 行）`实现写的行等于模型重算的行，不许成新发现`，新发现签名 `ModelDisagreement { aspect: "写的实例表行" }` | 无（12 过 1 红） |
| 第 824 行 M9 | `crash.rs` 删掉 `&& !abandoned_by_the_landed_version(&publish)` | `units_of_a_publish_the_landed_version_…`（crash_injection 测试文件第 534 行）`被抛弃的 (1, 4) 的单元不算「该在」，记录核对器一条都不判红`，left `claimed_state_missing_unit: true` | 无（快档那条在基线红集里） |

留给门禁 59 号、没单独证的 9 行：814、816、817、818（b），820、821、822、823（d），825（c，`abandons` 的 `>` 改 `>=`，点名 (c) 用例里 (1, 3) 那一半）。变异名以 `实八 (b)` / `实八 (d)` / `实八 (c)` 开头。13 行原文都用 `rows.py` 在主工作区核过恰好命中一次，门禁 33 号也没点它们。

## 第 4 步那几样（主工作区；末行原样）
- 负载：两次 `ps` 都没有 cargo、qemu、fio，没等锁。
- 动到的测试二进制（release，`bash research/scripts/run-with-memory-cap.sh 16G bash research/scripts/capped.sh 10 bash /tmp/claude-1000/impl-rbf-8/run-touched-2.sh`，日志在 `/tmp/claude-1000/impl-rbf-8/logs2/touched/`）：
  - `-p singlefs-harness --lib`：`test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.32s`
  - `-p singlefs-checker --lib`：`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - `--test second_transaction_supplement_three_fault_injection`：`test result: ok. 13 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.46s`
  - `--test second_transaction_supplement_three_crash_injection`：`test result: FAILED. 7 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.26s`（红的是快档，C554 那一形，见「第 4 条」一节）
  - `--test record_checker_judges_absence_by_the_persisted_set`：`test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.70s`
  - `--test second_transaction_supplement_two_record_checker_reuse_legality`：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s`
  - 随机历史那个二进制（模型改了，也是门禁 74 号跑的那一个）：主工作区 cargo 编不过（另一个会话的原因），所以直接跑了编出来的 `target/release/deps/second_transaction_supplement_three_random_history-4ae23017673bae18`。它编译时我的语义改动已经全在里面，之后只动过 rustfmt 排版和测试文件。经 `run-with-memory-cap.sh 16G` 跑：`test result: ok. 24 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 39.34s`，六段「模型对拍 N 步」N 都大于 0。
- `cargo fmt --all -- --check`：退出 1，`Diff in` 只出现在 `crates/singlefs-harness/src/layer0_progress.rs`（6 处，另一个会话的文件）；我改的文件都用 `rustfmt --edition 2021` 单独排过。末行原样：`         merged.ledger_paths.push(path);`
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 check.sh 那 7 条 lint（另一个会话动手之前）：退出 0，末行原样 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.75s`
- `cargo build --offline --all-targets`：退出 101，末行原样 `error: could not compile \`singlefs-harness\` (lib test) due to 5 previous errors`。报错全在 `layer0_progress.rs:1723/1733/1751` 和 `crash.rs:1884/2220`：`toolchain_never_asked` 找不到、`Layer0ProgressPlan` 缺 `shard`、`Layer0Resume` 多了两个成员。`layer0_progress.rs` 是未跟踪文件被改过；`crash.rs` 里那几处是层 0 分片续跑，快照对比显示是两次快照之间别人加的，不在我改的第 15 行和第 700–782 行里。
- 登记给我的门禁阶段（`nice -n 19 bash research/scripts/capped.sh 10 bash .claude/gate.d/<文件>`，74 号设了 `GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=16G`）：
  - 33 号，退出 1，末行 `    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。`。点名的只有 `crates/mutations.tsv:794`、`:799` 两行（实六续跑，原文在 `layer0_progress.rs` 里命中 0 次），都是那个会话正在改的文件，不是我追加的行。
  - 53 号，退出 0，末行 `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
  - 74 号，退出 1，末行 `                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。`。红因是日志第 2、8 行的编译错误（同上），不是对拍对不上；同一个二进制的预编版本我跑过，全绿（见上）。
  - 92 号，退出 0；93 号，退出 0；94 号，退出 0（末行见日志 `/tmp/claude-1000/impl-rbf-8/logs2/gates/`）
  - 89 号，退出 77（本次未跑，不算通过）

## 写过的文件（我自己列的；前任写的另外标出）
- `crates/singlefs-harness/src/model.rs`：(b)(d) 由前任写，我只跑了 rustfmt
- `crates/singlefs-harness/src/model_comparison.rs`：前任写，我跑了 rustfmt
- `crates/singlefs-harness/src/history.rs`：前任写
- `crates/singlefs-harness/src/fault_injection.rs`：前任写，我跑了 rustfmt
- `crates/singlefs-harness/src/crash.rs`：我改的只有第 15 行 import 与第 700–782 行（记录核对器）；这份文件里别的改动是另一个会话的
- `crates/singlefs-checker/src/walk.rs`：`InstanceTableOfRootRecord`，加了 25 行，没删
- `crates/singlefs-harness/tests/second_transaction_supplement_three_fault_injection.rs`：(b)(d) 两条新用例，改了两段文档注释，import 加了 `FailureSignature`
- `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`：(c) 用例，外加两个辅助函数与 import
- `crates/mutations.tsv`：末尾追加 13 行（第 813–825 行），变异名是 `实八 (b) 执行器：…`、`实八 (b) 按注入点认：读坏的槽正是被吞那次根槽写的也不认`、`…（任何槽都认）`、`实八 (b) 模型：…`、`实八 (b) 胶水：…`、`实八 (b) 按注入点认：被吞的根槽写落在哪个槽一个都不收`、`实八 (d) 模型：写文件内容的发布记事务号 0…`、`实八 (d) 模型：由记录重建…`、`实八 (d) 按注入点认：实现写的行与重算的行相同反而不认`、`实八 (d) 模型：「写的实例表行」对不上时不带实现写的行`、`实八 (d) 按注入点认：有被吞的根槽写也不重算…`、`实八 (c) 记录核对器：…`、`实八 (c) checker：…T = Ti 也算被抛弃…`。用 `research/scripts/insert-row.py` 逐行插，别人的行一行没动。

逐文件和前任开工快照 `before/` 比出来的增删行数（我和前任合计；crash.rs 里含别人的改动）：model.rs +113/−7、model_comparison.rs +47/−2、history.rs +11/−3、fault_injection.rs +191/−9、crash.rs +63/−21、walk.rs +25/−0、fault_injection 测试 +175/−8、crash_injection 测试 +153/−3、mutations.tsv +13/−0。

`git diff --stat -- crates litmus` 原样（这些是多个会话自上次提交以来的全部改动，分不出谁改的）：`82 files changed, 11814 insertions(+), 7164 deletions(-)`

## 交主 agent 的
1. **主工作区现在编不过**，原因是另一个会话在改 `crates/singlefs-harness/src/layer0_progress.rs`（未跟踪），`crash.rs` 里配套的改动（第 1884、2220 行）也没改完。不在交给我的「别的会话在改」清单里，我没碰。门禁 33、74 号也红在这里。那边收工之后要重跑 build 与 74 号。
2. **(c) 的实现取法是我定的**，条款只写了「按恢复落到的那一版的实例表判为被抛弃」：
   - 那一版的实例表，取记录流里写出这个根身份的最后一次根槽写所带的指针，在崩溃后镜像上沿链读，解析用的是 checker 那一份，不用 core 的。前提是同一实例里实例表不重写（core 里只有 `mount.rs:1890` 写行那次发布是 `InstanceTablePlan::Rewrite`），并且由记录重建的根的实例表指针照所选根（D23（journal 的角色与格式） 已定项 15）。
   - 读不出、或者根身份在基镜像里时，一律不豁免（偏严）。
   - 记录核对器因此多依赖了一样东西：checker 的实例表链读法。D13（验证路线） 已定项 7 列的入参里有崩溃后镜像，实例表是从它上面读的，没加入参。
3. 诊断里看到的一件事，没核：种子基 + 2 里 (1, 4) 的单元在它的根还在根环里时就被复用了（第 3 步抛弃，到 txg 11 就缺席）。D23 已定项 14 写的是「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配」，而这里 (1, 4) 被抛弃时读不出、影子账看不到它。这一格像是 C554 的同一个根源（影子账算不到暂时读不出的被抛弃根），只是恢复落在一个实例表已经判它被抛弃的版本上，所以读回没出错。(c) 的豁免把这一格放过了，这是条款要的结局，不是判据放宽；但它说明 C554 的修法要连这一格一起看。

## 没做什么
- 没走三方对抗；层 0、QEMU、herd7 和 crates 变异整表归 `crash-verifier`；没提交。
- 13 行变异里 9 行没单独证红，留给门禁 59 号。
- 主工作区的 `cargo build --all-targets`、门禁 74 号没有在我的改动上跑绿，被另一个会话的编译错误挡住；74 号那个二进制用的预编版本跑过，全绿。
- 崩溃注入快档还红在 C554 那一形（规格第 4 条），没修，也没 ignore。
- 随机历史与故障注入的大档没整档跑，只复跑了两个复现种子。

## 草稿与清理
- 删了：`/tmp/claude-1000/impl-rbf-8/copies/diag`（1.1G，诊断副本，含它自己的 target）；`/tmp/claude-1000/impl-rbf-8/copies/proof`（1.2G，证红副本，含它自己的 target）。
- 留着的是报告和复跑材料，都在 `/tmp/claude-1000/impl-rbf-8/` 下：`progress-2.md`、`logs/`（复现日志、编译日志）、`logs2/`（动到的二进制、门禁、随机历史、fmt 的日志）、`proof-logs/`（证红日志）、`rows.py`、`rows.tsv`（13 行变异原文）、`mutate-one.py`、`proof-run.sh`、`run-touched-2.sh`。前任的 `before/`、`progress.md`、`logs/` 里它那几份，我没动。
- `/tmp/singlefs-crash-injection-1155558-seedbase-7463871032432355115` 是前任跑大档留下的判红镜像，不是我建的，没动。
