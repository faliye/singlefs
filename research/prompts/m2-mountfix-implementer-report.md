# 实现员接手报告：impl-m2-mountfix（收口表第 27 行 C503、第 52 行 C502、第 5 行 C516 / C517 / C518）

时刻一律 UTC。接手自子 agent ad8f30e7660714e3d（交接摘要 `/tmp/claude-1000/handover/ad8f30e7660714e3d.md`）。

## 一、交付物

| 样 | 路径 | 说明 |
|---|---|---|
| 补丁 | `/tmp/claude-1000/impl-m2-mountfix/impl-m2-mountfix.patch` | 只含 `crates/`（14 个文件），不含 `crates/mutations.tsv`、不含 `litmus/`；04:25:34 在主工作区 `git apply --check` 退出 0 |
| 新变异行 | `/tmp/claude-1000/impl-m2-mountfix/mutations-append.tsv` | 16 行整行；补丁打上之后每行原文恰好命中一次（对 repo3 逐行数过） |
| 报告 | `/tmp/claude-1000/impl-m2-mountfix/report.md` | 本文件 |

补丁的 `git apply --stat`（主工作区上跑，04:25）：

```
 crates/singlefs-core/src/allocator.rs              |  355 ++++++++++++++++++++
 crates/singlefs-core/src/mount.rs                  |  200 ++++++++---
 crates/singlefs-core/src/recovery.rs               |   45 ++-
 crates/singlefs-core/src/root_ring.rs              |    2 
 crates/singlefs-core/src/transaction.rs            |    8 
 crates/singlefs-harness/src/history.rs             |   18 +
 crates/singlefs-harness/src/model_comparison.rs    |    6 
 crates/singlefs-harness/src/model.rs               |   34 --
 .../tests/checker_known_bad_images.rs              |   26 +
 .../tests/second_transaction_step_five_reuse.rs    |   97 +++++
 ..._transaction_supplement_three_random_history.rs |  243 ++++++++++----
 ...ion_supplement_two_commit_generated_fallback.rs |   93 +++++
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |  282 ++++++++++++++++
 ...upplement_two_unreadable_abandoned_root_slot.rs |  233 ++++++++++---
 14 files changed, 1380 insertions(+), 262 deletions(-)
```

主工作区一个字没动。`git diff --stat -- crates litmus` 在主工作区上反映的是别的会话已落地的改动、分不出谁的，所以给的是补丁自己的 stat。

## 二、每件做成了什么（行号是补丁打上之后 repo3 副本里的）

| 件 | 改法 | 钉它的用例 |
|---|---|---|
| C518（一次挂载之内环转过一圈之后不回收） | 分配器新带一张只住内存的根环表 `RootRingOccupancy`（`allocator.rs:725`），挂载时从盘上读（`mount.rs:610`，读根带槽位的 `recovery::readable_roots_with_ring_slots`），这个进程每写一条根记一条 `record_root_written_by_this_process`（`allocator.rs:1290`；调用点 `transaction.rs:868`、`3210`，零单元发布 `mount.rs:1100`、`1322`）：盖掉之后按 D16 已定项 1 的谓词 max(F_生效, 环里最旧有效根) 当场回收（与挂载同一个 `mount::reclaim_floor`）。带单元的发布在落点取完、装记账行之前记，记账行按回收之后的数写 | `second_transaction_supplement_two_root_ring_turn_in_one_mount.rs:70`；随机历史 `…three_random_history.rs:892`（原「一次挂载里转环停在已知红第 0 条」那一形现在跑完、每步 checker 绿）；`checker_known_bad_images.rs:3076` 改成一条都不红 |
| C517（固定点分配被拒之后同一进程里写不出去） | 同 C518 那一处 | `…root_ring_turn_in_one_mount.rs:241`：384 槽小盘上挂载、37 次覆盖写、抬 F，重挂那一路与不重挂那一路都发得出去，不重挂再连发 11 次都 Applied |
| C503（隔离位清零的时机条文与实现说反话） | `DeviceFreeMap::clear_isolation_of_slot`（`allocator.rs:373`）；根环表里被抛弃的根带着它引用的全部落点（影子账算的时候交回），它的槽被这个进程的根盖掉的那一次发布里，清掉环里别的被抛弃根都不再引用的那几个槽的隔离位。挂载内回收出来、仍被环里某条被抛弃根引用的槽当场补隔离（D23 已定项 14 主句：被抛弃根离开根环之前它引用的单元不许重新分配） | `…unreadable_abandoned_root_slot.rs:299`（盖掉 B 的根槽那一次：两盘隔离数 36 → 30，B 独占的四个固定点回到空闲、没有新记录）、`:400`（txg 27 回收的 mkfs 实例表两槽因 B 还在环里补隔离，txg 28 盖掉 B 之后 txg 29 的数据单元落回 50176） |
| C502（抬 F 时现行版本里没有实例表单元） | `raise_rollback_floor` 从现行那一版的根指针沿链读实例表（`mount.rs:878`，`instance_table_chain_of_root`，接实四的链读法），不看 `TransactionOutput::units`；成员 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 删掉，模型里对应的 `RaiseWithFormatTimeInstanceTableUnsupported` 与 `instance_table_unit_is_in_this_processs_memory` 一并删 | `second_transaction_step_five_reuse.rs:633`（mkfs 同一进程覆盖写三次之后抬 F 到 3 做成、checker 全绿）、`:686`（现行根的实例表两份都改坏：`InstanceTableMalformed`、盘上逐字节不变） |
| C516（抬 F 那一串发布被拒时前面几次已落盘） | 新成员 `MountError::RaiseFloorSequencePublishFailed { publishes_persisted, cause }`（`mount.rs:56`），抬 F 那一串里任何一次 `publish_version` 失败都包成它（`mount.rs:968`） | `…commit_generated_fallback.rs:356`：第二次空发布拿不到固定点，报 `publishes_persisted = 1`，盘上正好多出 txg 14、F = 8 那一条根；原有那条「第一次就拿不到」的用例改成 `publishes_persisted: 0` |

推翻条件：
- C518 / C517：一次可写挂载之内连发多于 24 次，谓词放行的槽要重挂才拿得到（`…root_ring_turn_in_one_mount.rs:70` 在 txg 26 之后 50178 不空闲），或者 checker 在做过挂载的会话里转环之后判 I-3.1 红，这两件就没做成。
- C503：被抛弃根的根槽被盖掉之后隔离数不降，或者降到把环里别的被抛弃根还引用的槽也放出去（36 → 30 那条断言两边都钉着）。
- C502：mkfs 同一进程里覆盖写之后抬 F 报错。
- C516：抬 F 那一串中途被拒时 `publishes_persisted` 与盘上新多出的根条数对不上。

## 三、每条新测试「改坏哪一行 → 哪条断言红」

两轮：① 03:32–04:05 在 repo2（补丁 + 03:27 的主工作区）上改坏一处、跑那条测试所在的**整个测试二进制**，记同时红了哪些；② 04:18–04:24 在 repo3（补丁 + 04:11 的主工作区）上按 `mutations-append.tsv` 的每一行（带过滤的参数，与门禁 59 号同一跑法）再跑一遍，16 行全红。基线红集：两份不改坏的副本上把动到的二进制整份跑过，一条都没红（repo3：`singlefs-core --lib` 91 过、`checker_known_bad_images` 25 过、`step_five_reuse` 13 过、`three_random_history` 20 过 2 忽略、`commit_generated_fallback` 5 过、`root_ring_turn_in_one_mount` 3 过、`unreadable_abandoned_root_slot` 3 过）。被改坏的代码里没有 `debug_assert`，每一格红在测试断言上。行号是 repo3 里的。

| 改坏哪一行 | 红的断言（原样） | 同一二进制里同时红的 |
|---|---|---|
| `allocator.rs:1325–1326` 挂载内回收换成空（`let reclaimed: Vec<AllocationRecord> = Vec::new();`） | `…root_ring_turn_in_one_mount.rs` `turning_the_root_ring_in_one_writable_mount_…`：「盘 DeviceIdentity(0)：txg 26 盖掉 txg 2 之后 mkfs 那片树表按谓词回收，同一次挂载里就回到空闲」 | 同一处跑四个二进制：C517 那条（left 全是 `Refused(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))`，right 11 个 `Applied`）；随机历史 `turning_the_root_ring_with_overwrites_in_one_mount_runs_to_the_end_…`（left `KnownRed { form: 0 … Operation(21) …}`，right `Completed`）；`checker_known_bad_images` 那条（left `["I-3.1"]`，right `[]`）；C503 两条（`[34, 34]` 对 `[36, 36]`；defer 187 对 181） |
| `allocator.rs:1300` 跳号判定 `if false && …` | `allocator::tests::a_root_ring_occupancy_that_missed_a_root_stops_reclaiming_as_the_ring_turns`：「txg 24 那条根没经分配器：记到 25 时表停下」（left `Some(FollowsEveryRootWrittenByThisProcess)`） | 无（`singlefs-core --lib` 整份） |
| `transaction.rs:2278` 删掉 `record_zero_unit_roots_leading_to` | `a_zero_unit_publish_the_caller_issued_…`：「txg 3 那条根补记了：表照旧跟得上」（left `Some(StoppedAtAnUnrecordedRoot { expected_txg: CheckpointTxg(3), recorded_txg: CheckpointTxg(4) })`） | 无 |
| `allocator.rs:375` 清位 `if false && …` | `the_isolation_bits_only_the_abandoned_root_holds_…`：「B 的根槽被盖掉那一次发布里，只有 B 撑着的六个隔离位（四个固定点、mkfs 实例表两槽）两块盘各清掉」（left `[36, 36]`，right `[30, 30]`） | `a_slot_reclaimed_while_…`（txg 29 数据单元 left `SlotNumber(50220)`，right `SlotNumber(50176)`） |
| `allocator.rs:1369` `if true \|\| !is_still_referenced`（多清） | 同一条断言，left `[24, 24]`，right `[30, 30]` | 无 |
| `recovery.rs:499` 根一律记成本区域第 0 槽 | C503 那条：「盖掉 B 的根槽之前：34 个，加 txg 27 那次回收时补隔离的 mkfs 实例表两槽」（left `[8, 8]`） | `turning_the_root_ring_in_one_writable_mount_…`（defer 215 对 221）、`a_slot_reclaimed_while_…`（换下代 12 对 9） |
| `allocator.rs:1327–1330` 不补隔离 | `a_slot_reclaimed_while_…`：「盘 DeviceIdentity(0)：50176 回收了，但 B 还在环里引用它，补了隔离」 | C503 那条（`[34, 34]` 对 `[36, 36]`） |
| `mount.rs:878–880` 退回读 `current.units` 里的实例表单元 | `raising_the_floor_in_the_make_filesystem_process_…`：`抬 F 到 3: InstanceTableMalformed`（`expect` 那一行） | 无 |
| `mount.rs:878–880` 读不出当成空表 | `raising_the_floor_when_the_current_roots_instance_table_is_unreadable_…`：「现行那一版的实例表两份都读不出：Some(RollbackFloorAboveCeiling { requested: CheckpointTxg(3), ceiling: CheckpointTxg(0) })」——成员对不上就红；盘面那条断言在这个变异下不红（上限那一关也在写之前拒） | 无 |
| `mount.rs:969` `publishes_persisted: 0` | `a_raise_whose_second_empty_publish_is_refused_…`：「这一串在被拒之前已经落盘了一次（txg 14）」（left 0，right 1） | 无 |
| `history.rs:3115` `if true \|\| !continues_past_the_ring_turn_form`（门禁 59 号原第 171 行那一处） | 新用例 `the_continuing_checker_notes_the_first_known_red_form_in_the_make_filesystem_process_and_runs_to_the_end`（left `KnownRed { form: 0 … Operation(22) …}`，right `Completed`） | `unit_area_wall_sampling_on_small_devices_…` |

改过的四条随机历史写死用例与已知红第 0 条的复现，靠的是表里已有的行：W1（主工作区第 155、156 行）、m1c（第 172 行）、m1d（第 173 行）在 repo3 上照样红；第 182 行的变异换到 mkfs 那条会话的复现上（追加的第 14 行）照样红（「要以已知红收尾：NewFinding { signature: CheckerViolations { invariants: ["I-3.1"] } …」）。

`mutations-append.tsv` 的 16 行：第 1–13 行对应上表前十格（第一格拆成四行各点一条测试）；第 14 行是主工作区第 182 行换了必须红的测试；第 15 行是第 288 行（原第 27 行 ④「隔离位不挡分配」）换到改名后的 C503 用例；第 16 行是第 171 行换到新用例。

## 四、`crates/mutations.tsv` 里要主 agent 处置的已有行（行号是主工作区 04:25 那一版的）

补丁打上之后这四行不再成立，只追加、不改别人的行，所以列在这里：

| 行 | 为什么 | 处置 |
|---|---|---|
| 56「步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错」 | 原文 `.ok_or(MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)?;` 随 C502 删掉（锚点命中 0 次），点名的测试也删了 | 删；C502 的两条变异是追加的第 11、12 行 |
| 171「「已知红第 0 条那一形只记不停」退回成判红就停；逼近分配记录墙那一段走不到墙」 | 在 repo2 上复跑：退出码 0、没红。C518 之后那一段 32 段历史全在挂载里转环，日志原样：「已知红第 0 条那一形只记不停 0 步（0 段历史）」 | 删；追加的第 16 行把同一处变异换到新用例上（红） |
| 182「checker 在 I-3.1 的说明文字里少写一项机理标识」 | 点名的 `turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form` 改名拆成两条（一次挂载那一形现在跑完） | 删；追加的第 14 行点 mkfs 那条会话的复现（红） |
| 288「增补 2 收口第 27 行 ④：隔离位不挡分配（is_free 不看隔离位）」 | 点名的 `the_isolation_bits_of_an_abandoned_root_survive_…` 随 C503 改写成「清掉」那一条 | 删；追加的第 15 行点改名后的用例（红） |

第 45 行（「步 4：普通重开不隔离被抛弃根引用的槽」，原文 `&|_| false,\n        ShadowLedger::On,\n    );`）：前任加的 `first_txg` 参数排在最后会把这个锚点改掉，我把参数挪到 `previous` 后面，锚点照旧恰好命中一次。
主工作区 04:25 那一版的表共 468 行（含注释），补丁打上之后逐行数锚点：只有第 56 行命中 0 次，其余每行恰好 1 次；追加的 16 行也都恰好 1 次。

## 五、`check.sh`

在 repo3 上跑（补丁 + 04:11 的主工作区；`e156_allocation_basis_counts.rs`、`e158_root_choice_repair.rs` 两个文件用 04:11 那一版，因为主工作区 04:25 那一版 e158 编不过、两份都不合 rustfmt，那是别的会话正在改的文件、不在补丁里）。`nice -n 19 bash .claude/scripts/check.sh` 末尾原样：

```
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
  ✗ clippy 有告警（按 -D warnings 视为错误），或者踩了编码纪律的某一条
     → 怎么办： 上面每条告警都指着文件和行号，逐条改。编码纪律那几条的写法见 rules/code-discipline.md。
                确有必要保留的，在那一处写 #[allow(<lint>, reason = "为什么")]，理由写进 reason——
                不要整仓关掉 -D warnings（rules/command-safety.md：警告是最便宜的信号）。
exit=1
```

- fmt 那一段过了（「✓ 格式通过」）；clippy 唯一一处红是 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:1672` 的 `items_after_test_module`，不在补丁里：同一条在不带补丁的主工作区副本上照样红（`/tmp/claude-1000/impl-m2-mountfix/mainclean-clippy.log`，03:5x 拷的主工作区，`--keep-going` 之下全仓只有这一处）。带补丁的 repo3 上 `--keep-going` 跑 clippy，报出的也只有 e158 那几处（`repo3-clippy.log`）。
- clippy 红了 `check.sh` 就停，后面的 `cargo build --all-targets`、`cargo test --all` 没跑到。另跑的：repo3 上 `cargo build --all-targets` 退出 0；动到的七个测试二进制全过（数见第三节开头）。`cargo test --all` 整份没跑（按主 agent「只跑动到的测试二进制」）。
- 登记给我的门禁阶段：按主 agent 要求留到最后统一跑，这一次没跑。

## 六、停下交主 agent 的设计问题

1. **mkfs 同一个进程那条会话不装根环表，已知红第 0 条在那里照旧走得到。** C518 的条文是「一次挂载之内」，mkfs 加第一个事务的那条会话没做过挂载：分配器是调用方 `PoolAllocator::new` 建的，这张表从一开始就不知道 txg 1、2 那两条零单元根，前任选了不装（`allocator.rs:820` 字段注释）。结果：这条会话里连发多于 24 次，checker 照旧在合法状态上判 I-3.1 红（`…three_random_history.rs:919` 钉着，第 22 步、差 16384 字节）。我没删清单第 0 条、也没收窄它的判据，只在 `history.rs:1476` 上面与那一条的 `shape` 里写明「C518 修好之后只有 mkfs 同一个进程那条会话还走得到」。要定的：(a) mkfs 那条会话也装表（mkfs 之后从盘上读一次，或 `make_filesystem` 交出一张）——那样第 0 条可以整条删；代价是三条逼近分配记录墙的写死用例（`…three_random_history.rs:503`、`:604`、`:662`）现在正是靠「这条会话里每次覆盖写恰好加 16 条、每次空发布恰好加 8 条」凑到 812，要重新设计；(b) 维持现状，但把第 0 条的判据收窄到那条会话，免得做过挂载的会话里 C518 退化时被随机历史当成已知红接走（今天只有专门的用例拦得住）。
2. **根环表跟不上时停下（`RootRingOccupancyTracking::StoppedAtAnUnrecordedRoot`，`allocator.rs:708`）没有条款。** 记到的根 txg 不是上一条加一（有根没经分配器写出去）时，前任选的是从那一刻起挂载内回收与轮转清隔离位都不做、到下一次挂载为止——少回收、多隔离，不报错、不写盘。经公开入口我找到的唯一一条缺口是调用方在树表 0 条那一版上直接调 `publish_without_units`，那一条由 `publish_first_file` 在发之前补记（`transaction.rs:2278`），所以现在只有单元测试走得到它（`allocator.rs:1947`）。要不要改成报错、或者留着当防御，交主 agent。
3. **C516：没改成「整串先准入再发」。** 派发写明那是设计判断；现在只有错误成员报出走到第几次。另外 `RaiseFloorSequencePublishFailed` 在 `publishes_persisted = 0`（第一次就被拒、一个字节没写）时也用这个成员，调用方此前拿到的是 `MountError::Publish(…)`；`history.rs` 与 `model_comparison.rs` 的胶水已跟着改（按 `cause` 分类）。
4. **两处改坏了没有用例会红（等价变异，没做成变异行）：** ① `mount.rs:1100`、`:1322` 两处零单元发布之后的 `record_root_written_by_this_process`：删掉之后下一次带单元的发布只可能是 `publish_first_file`，它的补记（`transaction.rs:2278`）会把同样几条根记进去，结果一样；② 前任加的 `note_rollback_floor_took_effect`（抬 F 生效之后把表里的 F 抬高）我删了：释放代 ≤ 新 F 的抬 F 自己已经回收过，之后释放的释放代都高于新 F，挂载内谓词里 F 那一半再抬也多收不回一个槽，理由写在 `allocator.rs` 那个字段的注释里。
5. **kb 里还写着旧名字（不归我改）：** `.claude/kb/checks-owed.md` 第 445 行（C502）引 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`、第 446 行（C503）引 `the_isolation_bits_of_an_abandoned_root_survive_the_ring_rotation_that_overwrites_its_root_slot`；`.claude/kb/milestone/02-second-txn.md` 第 197、362、392 行同样引着旧名字。C502 / C503 / C516–C518 的状态由主 agent 或 kb-scribe 改。

## 七、已有变异行复跑

- repo3 上（补丁 + 04:11 的主工作区）：点名这几个动到的二进制的已有行（`three_random_history`、`unreadable_abandoned_root_slot`、`commit_generated_fallback`、`step_five_reuse`、`root_ring_turn_in_one_mount`）加第 342 行（checker 第 ④ 条，点名改写过的那条 `checker_known_bad_images` 用例），除去第四节要删的 56、171、182、288，共 54 行，**54 行全红**。行号：39 40 41 42 43 46 48 52 53 54 55 57 62 63 70 90 91 92 93 109 128 129 138 139 140 142 143 144 145 146 147 148 155 156 159 160 161 165 167 168 169 172 173 187 216 220 222 286 287 296 342 353 413 468。结果在 `/tmp/claude-1000/impl-m2-mountfix/mutation-proofs/existing-rows-replay-repo3.out`，每行的 cargo 原样输出在同目录 `logs-existing-repo3/`。
- 更早在 repo2（补丁 + 03:27 的主工作区）上复跑 55 行：54 红、第 171 行没红（第四节）。
- 别的二进制上的已有行（崩溃注入、坏盘输入、故障注入等里也有历史转过根环的）没复跑：C518 改了一次挂载里转环之后的分配，按取样判红的那些行可能跟着变，门禁 59 号整表复跑归 crash-verifier。

## 八、写过的文件

补丁里 14 个（`crates/` 下，路径见第一节 stat）：`singlefs-core/src/{allocator,mount,recovery,root_ring,transaction}.rs`；`singlefs-harness/src/{history,model,model_comparison}.rs`；`singlefs-harness/tests/` 下 `checker_known_bad_images.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_supplement_three_random_history.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`、`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs`，新建 `second_transaction_supplement_two_root_ring_turn_in_one_mount.rs`。`crates/mutations.tsv` 没碰，追加的 16 行在 `mutations-append.tsv`（变异名见那份文件第一列）。

接手之后我在前任的基础上改的：随机历史里三条逼近分配记录墙的写死用例重新设计（前任走到一半，四条在带补丁的副本上红）；已知红第 0 条那条复现拆成两条（一次挂载里跑完、mkfs 那条会话里照旧第 0 条）；加 `the_continuing_checker_notes_…`；`history.rs` 第 0 条的说明；删 `note_rollback_floor_took_effect`、挂载内回收门槛改用 `mount::reclaim_floor`；`rebuilt_allocator` 的 `first_txg` 参数挪位（保第 45 行锚点）；`clear_isolation_of_slot` 的注释改成只管影子账那一套隔离（主工作区新加了释放核校验和的第二套隔离）；按实七 / 实四之后主工作区的现状把补丁重放两次（03:27、04:18），第二次 `transaction.rs` 一处补记调用换到 `publish_version_of_trees_holding_one_data_unit` 前面、两条测试的 `TransactionUnit::Data` 改成 `TransactionUnit::Data(DataUnitIndexInFile::FIRST)`。

## 九、负载与经过里要记的

- 03:09 核现场：前任起的后台任务都已结束（`ps` 里没有 cargo）；副本 repo2 与日志都在。
- 03:29 `ps` 看到性能测量在跑：pid 4066033 `bash research/scripts/vm-bench.sh …e159-fsync-wait-group-commit…`、pid 4066354 `qemu-system-x86_64`；新的编译停下等 pid 4066033 退出，03:31:38 退出，等了 1 分 45 秒。别的会话的 cargo（pid 3996240、4042469）同时在跑，没等锁。
- 04:26 起过一轮复跑时主工作区的 e158 编不过（别的会话在改），那一轮 47 行判「编不过」作废（`existing-rows-replay-repo3-broken-e158.out`），换回 04:11 那一版 e158 之后重跑。

## 十、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 crash-verifier；没提交。
- 没新加层 0 流或崩溃点重放用例。
- `cargo test --all` 整份没跑（主 agent 要求只跑动到的二进制）；`check.sh` 停在别人文件的 clippy 上，build 与 test 两段没走到。
- 登记给我的门禁阶段没跑，按主 agent 要求留到最后统一跑。
- 第六节五条没替主 agent 定；kb 没改。
- 草稿里没入库的：`/tmp/claude-1000/impl-m2-mountfix/probes/`（探针：逼近墙的历史网格搜索、已知红第 0 条在挂载里与 mkfs 会话里各走不走得到），`probe-grid.log`（432 段历史的网格，回退那一条的设计从这里挑出来）。它们只用来找写死用例的历史，结论已经写成用例钉进补丁，不是实验产物，没进 `research/results/`。
