# 实二七报告：可写挂载之前的逐盘核（空盘、停在旧状态的盘拒可写）

## 结论

- 可写挂载（`mount_writable`）与回退（`mount_rollback`）共用的 `establish_instance` 在取号之前逐盘核：交进来的每块盘带不带所选那一版（可写挂载是施加前缀之后那一版，回退是 R_old 那一版）。有一块不带就在取号之前返回新成员 `MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version, selected_version_journal_position, devices }`，`devices` 里每块盘写明缺什么（`SelectedVersionLackingOnDevice`），盘上逐字节不变。
- 判「不带」的口径（`crates/singlefs-core/src/mount.rs` 第 1962 行 `devices_without_the_selected_version`）：
  - 空盘：两个系统配置槽一份自证过的（校验和过、fsid 相同）都没有 ⇒ `NoSelfVerifiedSystemConfiguration`（D18 已定项 11「可见」= 独占打开成功且系统配置读得通）。
  - 停在旧状态：世代号最大的那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn（第 1936 行 `journal_position_of_the_selected_version`），**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同 ⇒ `BehindTheSelectedVersionAndMissingItsUnits { newest_system_configuration_journal_tail, units_missing }`。
  - 只落后、单元都在（根落盘之后、轮换之前崩了；取号失败回卷写回 tail 0）不拒；系统配置跟得上、只缺几份单元（单份坏）不拒。
- 所选那一版的单元清单：带文件的一版是重建出来的 `units`（借用，不拷）；树表 0 条的一版是实例表第 0 片、树表、分配记录树每个节点（第 377 行 `PreviousVersion::units`）。字节是空的单元（N2）跳过。
- 只读挂载不改代码：`read_unit_via_locations` 逐条试位置条目，空盘 / 旧盘上照样读回所选那一版；盘 0 是空盘那一格读到的是盘 1 上那一份（用例钉住）。
- 两条验收用例在今天的代码上红、改完绿；同文件 4 条今天绿的照绿。重同步没做，留给 C120（分叉盘回归的判定与重同步）。

推翻条件：某个健康池或正常崩溃状态（层 0、随机历史、坏盘普查、E158）在可写挂载上报出新成员；或者停在旧状态、系统配置却不落后的盘（见「交主 agent 的设计问题」第 1 条）挂成了可写。

## 写过的文件

- `crates/singlefs-core/src/mount.rs`：新成员与两个类型（第 179、189、199 行）、`PreviousVersion::units`（第 377 行）、`InstanceStart::selected_version_journal_position`、两个函数（第 1936、1962 行）、`establish_instance` 里取单元清单（第 2035 行）与取号之前的核（第 2142 行）、`mount_writable` / `mount_rollback` 各算一次 jsn（第 2367、2546 行）、两处 `# Errors` 文档。
- `crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`（两处）、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`（两处）：穷举 `match` 补新成员一臂（不写通配臂）；`model_comparison` 里归「健康内存盘上不该出现」那一组。
- 新文件 `crates/singlefs-harness/tests/second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs`（14 条用例：调查文件搬来 6 条 + 新写 8 条）。
- `crates/mutations.tsv` 末尾追加 15 行（第 709–723 行），变异名都以「实二七（fsync 失败掉盘的核查…）：」开头，后半依次是：拒可写那一句关掉；带文件的一版单元清单成了空的；可写挂载那条路不把所选根那条记录交给判落后；没有自证过的系统配置的盘不记下；jsn 计数器取 0；不看系统配置落不落后；只落后也记成不带；空字节单元也拿去读盘比；树表 0 条那一版分配记录树节点角色记错；回退那条路不交 R_old 那条记录；读一个单元只试第一条位置条目；发布冻结着也放行另发一次（×3，三条矩阵用例各一行）；只交盘 0 时分配记录树那一判 panic。
- 改动的 patch（我这六个文件）：`/tmp/claude-1000/impl-m2-e27/mine.patch`，对主工作区现状 `git apply --check --reverse` 退出码 0（主工作区就是改完的样子）。

## `git diff --stat -- crates litmus` 原样（主工作区里别的会话没提交的改动都在内，分不出谁改的；我的只有上一节那六个文件，新测试文件是未跟踪文件、不在这张表里）

```text
 crates/mutations.tsv                               |  356 +-
 crates/singlefs-checker/src/image.rs               |   79 +-
 crates/singlefs-checker/src/lib.rs                 |  100 +-
 crates/singlefs-checker/src/walk.rs                | 2170 ++++++++-
 crates/singlefs-core/src/admission.rs              |  174 +-
 crates/singlefs-core/src/allocator.rs              |  226 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  139 +-
 crates/singlefs-core/src/lib.rs                    |    4 +
 crates/singlefs-core/src/make_filesystem.rs        |    2 +
 crates/singlefs-core/src/mount.rs                  | 1488 ++++--
 crates/singlefs-core/src/mounted_read.rs           |  285 +-
 crates/singlefs-core/src/recovery.rs               | 1190 +++--
 crates/singlefs-core/src/system_configuration.rs   |   76 +-
 crates/singlefs-core/src/transaction.rs            | 4934 +++++++++++++++-----
 crates/singlefs-core/src/write_accounting.rs       |   20 +-
 crates/singlefs-format/src/lib.rs                  |  131 +
 crates/singlefs-harness/src/bad_disk_input.rs      |  335 +-
 .../src/bin/e156_allocation_basis_counts.rs        |  744 ++-
 .../src/bin/e158_root_choice_repair.rs             | 2564 +++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  253 +-
 .../src/bin/first_transaction_on_device.rs         |  961 +++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    5 +-
 crates/singlefs-harness/src/history.rs             |  227 +-
 crates/singlefs-harness/src/model.rs               |  467 +-
 crates/singlefs-harness/src/model_comparison.rs    |   98 +-
 crates/singlefs-harness/src/on_device_modes.rs     |  132 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              | 1298 ++++-
 crates/singlefs-harness/tests/common/mod.rs        |   29 +
 .../tests/first_transaction_step_five_publish.rs   |  313 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_six_recovery.rs   |    5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../tests/parallel_line_one_sequential_write.rs    |   55 +-
 .../second_transaction_mapping_node_admission.rs   |  230 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |  167 +-
 ...ansaction_parallel_line_one_sequential_write.rs |   52 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  256 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  343 +-
 .../tests/second_transaction_step_five_reuse.rs    |   29 +-
 .../tests/second_transaction_step_four_rollback.rs |  241 +-
 .../tests/second_transaction_step_one_overwrite.rs |  218 +-
 ...second_transaction_step_three_formatted_pool.rs |  192 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |  103 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ..._transaction_supplement_one_write_accounting.rs |   97 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   37 +-
 ...transaction_supplement_three_crash_injection.rs |    3 +
 ...transaction_supplement_three_fault_injection.rs |  131 +-
 ..._transaction_supplement_three_random_history.rs |  590 ++-
 ...nsaction_supplement_two_accounting_node_full.rs |  134 +-
 ...transaction_supplement_two_admission_formula.rs |  318 +-
 ...two_c533_row_publish_record_without_its_root.rs |   87 +-
 ...ion_supplement_two_commit_generated_fallback.rs |  496 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   53 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +-
 ...n_supplement_two_release_checksum_quarantine.rs |  906 +++-
 ...saction_supplement_two_reused_record_overlap.rs |   19 +-
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    6 +-
 ...saction_supplement_two_row_publish_admission.rs |  358 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  792 +++-
 ...upplement_two_unreadable_abandoned_root_slot.rs |   44 +-
 .../system_configuration_mutability_classes.rs     |    5 +
 66 files changed, 20028 insertions(+), 5210 deletions(-)
```

## 新测试证红：改坏哪一行 → 哪条断言红

做法：仓副本 `/tmp/claude-1000/impl-m2-e27/{baseline,clean,mutant}`（只拷 `Cargo.toml`、`Cargo.lock`、`crates`、`litmus`，各用自己的 `target`），每条变异从 `clean` 拷回原件并 `touch` 之后再改下一条，每次跑整个测试二进制 `second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version`。汇总在 `mutation-results*.txt`，逐次日志 `mutant-*.log`。

基线：
- 今天的代码（`baseline` 副本，核心没改，只放搬来的 6 条）：`test result: FAILED. 4 passed; 2 failed`，红的正是两条验收用例，断言在 `assert_refused_or_both_copies`：「盘 1 是空盘：可写挂载做成了，而这一版有单元只剩一份 [Data@50184, ExtentRoot@50265, InodeLeafContainer@50266, InodeRoot@50268]，写两次之后池级 checker 判红 ["I-2.1", "I-4.8", "I-7.4"]」，停在旧状态那条同样。
- 改完的代码、未改坏的副本：`test result: ok. 14 passed; 0 failed`（跑了两次，第二次在 N2 那条换后端之后），基线红集为空。

| 变异（`crates/mutations.tsv` 行） | 改坏哪一行 | 点名的用例红在哪条断言 | 同一次还红了 |
|---|---|---|---|
| 709 | `mount.rs` 取号前 `if !devices_without.is_empty()` → `if false && …` | `mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies`：`assert_refused_or_both_copies`「可写挂载做成了，而这一版有单元只剩一份」 | 停在旧状态的验收用例、`blank_device_refuses…`、`stale_device_one_refuses…`、`device_one_stale_on_the_version_without_file…`、`rollback_onto…`（`expect_err`「…要拒」） |
| 710 | `PreviousVersion::units` 带文件那臂 `&output.units` → `&output.units[..0]` | `mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies`：同上 | `stale_device_one_refuses…`、`rollback_onto…` |
| 711 | `mount_writable` 里 `own_record.as_ref()` → `None` | 同上一行那条验收用例 | `blank_device_refuses…`（jsn 断言，left `(1, 0)`）、`stale_device_one_refuses…` |
| 712 | 没有自证过的系统配置那臂不 `push`、只 `continue` | `blank_device_refuses_the_writable_mount_by_name_before_any_write`：`expect_err("有一块空盘，可写挂载要拒")` | 空盘验收用例、`rollback_onto…` |
| 713 | `journal_position_of_the_selected_version` 里 `counter: record.counter` → `0` | `stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write`：`expect_err("盘 1 停在旧状态，可写挂载要拒")` | `blank_device_refuses…`（jsn 断言）、`rollback_onto…`、停在旧状态的验收用例 |
| 714 | 落后判定 `if tail >= position { continue; }` → `if false { continue; }` | `current_device_one_with_one_corrupted_copy_still_mounts_writable`：「跟得上这一版、只坏了一份的盘，可写挂载照样做成」 | 无 |
| 715 | `if !units_missing.is_empty()` → `if units_missing.len() < usize::MAX` | `device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies`：「只落后、单元都在的盘不许拒可写」 | N2 那条、`fsync_drop_persists_across_reopen` 与 `fsync_drop_persists_until_reopen`（「故障消失之后可写挂载仍报错：WritableMountRefusedByDevicesWithoutTheSelectedVersion」——矩阵里真有只落后的状态） |
| 716 | `.filter(\|unit\| !unit.bytes.is_empty())` → `.filter(\|_unit\| true)` | `device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable`：「数据单元哪块盘上都读不出，不是哪一块盘缺它：可写挂载照常做成」 | 无 |
| 717 | 树表 0 条那一版分配记录树节点 `identity` → `TreeTable` | `device_one_stale_on_the_version_without_file_…`：「缺的就是第二次挂载写行那次重写的那几个单元」，left `[TreeTable ×5, InstanceTable]` | 无 |
| 718 | `mount_rollback` 里 `own_record.as_ref()` → `None` | `rollback_onto_the_blank_or_stale_device_one_is_refused_the_same_way_before_any_write`：jsn 断言，left `(1, 0)` | 无 |
| 719 | `recovery.rs` `read_unit_via_locations` 只试 `&locations[..1]` | `read_only_mount_reads_the_selected_version_from_the_device_that_carries_it`：「盘 0 是空盘：只读挂载要做成，实际 Open(Walk(UnitUnreadable { slot: 50263 }))」 | `blank_device_refuses…`（盘 0 空盘那格变成 `Recovery(UnitUnreadable)`） |
| 720–722 | `transaction.rs` 冻结判定 `None => Ok(())` → `None \| Some(_) => Ok(())` | 三条矩阵用例各自的 `assert!(all_problems.is_empty())`（「冻结没拦住另一次发布」） | 三条同一次一起红 |
| 723 | `allocation_record_tree.rs` 那句 `.ok_or(violated(…))?` → `.expect(…)` | `mount_writable_with_only_device_zero_is_refused`：「盘 1 缺席时可写挂载必须拒绝，不许 panic」 | 无 |

- 15 行都在副本里实跑过、点名的用例都红了，没有留给门禁 59 号的。714、716 两行先按「删掉那一行」跑红，因为门禁 59 号不收空的替换文，又换成上表的替换文重跑了一次，红的仍是同一条（`mutation-results-m3m5.txt`）。
- 716 那一条最初是等价变异：内存盘上零长度读回的是空字节，与空字节相等，删掉这个跳过也照样放行。之后 N2 那条用例改在不接受零长度读的盘（`ZeroLengthReadRefusingDevice`，报 `Unaligned`）上跑，这个跳过就承重了，重跑证红（`mutation-results-m5.txt`）。
- 被测代码里没有 `debug_assert`，没另跑 `--release`。

## 交回前的验证（主工作区，末尾原样输出）

动到的测试二进制（整个二进制）：

```text
$ cargo test --offline -p singlefs-harness --test second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version
test current_device_one_with_one_corrupted_copy_still_mounts_writable ... ok
test device_one_behind_by_the_rotation_with_the_data_unit_unreadable_everywhere_still_mounts_writable ... ok
test blank_device_refuses_the_writable_mount_by_name_before_any_write ... ok
test mount_writable_with_a_blank_device_one_refuses_or_keeps_two_copies ... ok
test mount_writable_with_only_device_zero_is_refused ... ok
test mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies ... ok
test device_one_stale_on_the_version_without_file_refuses_the_writable_mount_naming_what_the_row_publish_rewrote ... ok
test read_only_mount_reads_the_selected_version_from_the_device_that_carries_it ... ok
test stale_device_one_refuses_the_writable_mount_naming_every_unit_it_misses_before_any_write ... ok
test rollback_onto_the_blank_or_stale_device_one_is_refused_the_same_way_before_any_write ... ok
test device_one_behind_only_by_the_rotation_it_missed_still_mounts_writable_with_two_copies ... ok
test fsync_drop_persists_until_reopen ... ok
test fsync_drop_persists_across_reopen ... ok
test fsync_drop_one_shot ... ok
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.75s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.75s

exit=0

$ cargo test --offline -p singlefs-core --lib
test result: ok. 119 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

exit=0

$ cargo test --offline -p singlefs-harness --lib
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.51s

exit=0
```

`cargo fmt --check`（红，红的三个文件都不是我改的：两个 E142 装置二进制与 e156；我的六个文件没有差异）：

```text
     16 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs
      8 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs
     10 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
exit=1
```

`cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings` 加 check.sh 那七条（红，只红 e156 一处；同一处在改之前的 baseline 副本上同样红，不是我改的）：

```text
error: this assertion is always `true`
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 1 previous error
exit=101
```

`cargo build --offline --all-targets`：

```text
   Compiling singlefs-harness v0.1.0 (/home/fy5090/code/singlefs/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.05s
exit=0
```

`git apply --check --reverse /tmp/claude-1000/impl-m2-e27/mine.patch`（主工作区）：退出码 0。

负载：开跑前 `ps` 看到别的会话的 `cargo test -q -p singlefs-harness --test s4_z19b_rerun`（pid 3995569）与 `cargo test --release --bin e142-first-txn-dry-run`（pid 4066304），没有性能测量在跑；没遇到等锁。线程上限 8，每条 cargo 都经 `research/scripts/capped.sh 8`。时刻（UTC）：开工约 04:25，交回约 05:10。

clippy 红的位置原样：

```text
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4057:9
```

## 交主 agent 的设计问题（条款没写到、我按下面的读法写了，没有自己定）

1. **系统配置跟得上、只缺所选那一版几份单元的盘，放不放可写**：今天放（用例 `current_device_one_with_one_corrupted_copy_still_mounts_writable` 钉住）。理由：它不是空盘、没停在旧状态，D18 已定项 11 作废的两类都不含它；单份坏在释放时由读盘核隔离（D19 已定项 5 硬规则 1）。代价：用改之前的代码在空盘上挂过可写的池，盘 1 的系统配置已经被写成最新的，缺的那几份照抄单元这道核认不出来——它与单份坏在盘上分不开（C120 说「回归盘不是祖先」今天没有可判定的形式）。要改成「缺单元就拒」，删掉 `mount.rs` 第 1962 行函数里的落后判定即可，那条用例随之改判。
2. **「落后」用哪个量判**：我用每块盘世代号最大的那份系统配置的 (journal_instance, journal_tail) 与所选那一版那次发布末条记录的 jsn 比（每次发布之后各盘轮换写的就是这一对）。派发提示里还写了「根环落后」，没做成判据：盘 1 只住区域 1，健康时它上面最新的根也可以比所选根落后两个 txg，判不出落后多少才算。只落后、单元都在的状态真出现：改坏第 715 行那一条，矩阵里 `fsync_drop_persists_*` 两条红，24 个持续档格子里红了 20 格（`BarrierAfterUnits` 只红持续到重开之后那两格、`UnitWrite` 只红持续到重开之后那两格，其余四个注入点两档四格全红：施加记录追上来的那一版、系统配置轮换失败、取号失败回卷写回 tail 0，都是只落后、单元都在）。
3. **回退到 R_old、停在旧状态的盘带着 R_old 的全部单元时**：放可写（R_old 的 jsn 不比它的系统配置新，逐盘核不看单元）。例：盘 1 停在 txg 3 之后，回退到 (1, 3) 会挂成可写，被抛弃的 (1, 4)、(1, 5) 的单元只在盘 0、由影子账隔离。D18 已定项 11「缺号期间池里发布过的盘整盘作废」按字面会拒这一格。没钉用例。
4. **盘少交一块（只交盘 0）**：今天被拒是因为 `Recovery(InvariantViolated { I-1.1, "分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点" })`（分配记录树的几何按交进来的盘算），不是按「可写设备数 ≥ w 的下限」拒的。新的逐盘核只看交进来的盘；那处碰巧的拒绝一旦没了，单盘会过这道核。显式的「交进来、带着这一版的盘数 < 2 就拒」今天走不到（重建先失败），写了也证不了红，所以没写。
5. **成本**：只对落后的盘逐个读所选那一版的单元（大文件按文件大小读一遍）；跟得上的盘一个单元都不读。健康池挂载不多读。

## 可能受影响、没跑的（按定义不归我跑，留给提交时那一次）

- `.claude/gate.d/` 各阶段都没跑（定义「门禁阶段都不跑」）。`crates/mutations.tsv` 第 585 行（不是我加的）第四段替换文是空的，门禁 59 号的形状检查会整张表拒掉。
- E158 装置（`e158_root_choice_repair`）读系统配置的那几条臂：一块盘两个系统配置槽都被注入读错的故障集，可写挂载 / 回退从 `Ok` 变成新成员（那块盘不「可见」）。推断，没跑 E158；那批产物若要复跑会与旧数不同。
- 调查文件第 7 条 `mount_writable_while_every_read_of_device_one_fails_is_recorded`（只记不判，没有断言）没搬：它那一格（盘 1 每次读都报错）改完之后应当被拒成 `NoSelfVerifiedSystemConfiguration`。推断，没跑。
- 坏盘输入普查（`bad_disk_input`）盲坏法里坏到某块盘系统配置两槽、或落后盘上的单元的镜像，可写挂载的结局文字会变；层 0、随机历史、`checker_known_bad_images` 等其余调 `mount_writable` 的测试二进制都没跑。我按「正常崩溃状态里所选那一版的单元两块盘都在、落后的盘单元不缺」推断它们不受影响。
- `first_transaction_on_device` 只补了穷举臂，没在真设备上跑。
- C120 那句「第一版 2 盘掉盘只能只读挂载 ⇒ 不可达」的改写归书记员，没碰 kb。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交；没做 git 写操作。
- 没做重同步（C120）。
- 没改 checker。
