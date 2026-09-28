# 实四甲交回：E156、E158 两个装置跟上实一到实三的形态

实现员（implementation-writer），2026-09-26 开工，在主工作区改，没有提交、没有任何 git 写操作。开工时把两个 bin 与 `crates/mutations.tsv` 拷成快照 `/tmp/claude-1000/impl-rbf-4a/start-snapshot/`，下文「开工时」都指它。

## 一、结论

- 做了三件：
  - E158 本地常量「系统配置字段表合计」没设环境变量时取 489（原 481），照 D22（单元原子性怎么合成） 已定项 9（原文整行见第三节）。`local_constants_match_crates_today`、`constants_and_anchors_all_pass` 转绿。
  - e156 那一处恒真断言（开工时 `e156_allocation_basis_counts.rs:4080` 的 `170 * 812 >= 812 * 169`）换成用 `e156_level1_position_of_leaf` 现算的两条 `assert_eq!`，放在同一条单测最前面当第三组的前提。clippy 全过，变异证过会红。
  - 变异表：第 476 行换成新锚点与新测试名；末尾追加 2 行（第 747、748 行）。
- 回退那一半（第 3 条）我停下了，没改语义，列在第七节：
  - 两个装置里用到本地 shim `mount_rollback`（实三加的：先 `mount_writable`，再 `roll_back_by_a_forward_publish`）的 7 处调用，记的都是挂载时回退才有的东西：回退抛弃目标之后那一段、回退写行、之后的暖机，还有由它们派生的登记数与基底。
  - 挂着回退不抛弃根、不取号、不写行、不暖机（D23（journal 的角色与格式） 已定项 14），这几处的问法都不再成立。
  - 我只把两个 shim 的文档注释改成写明这件事（原来写「归实现批次实四」）。
- 验收没全达到：
  - 两个 bin 的单测还各有红：e158 42 过 1 红，e156 9 过 2 红。红的三条都属第 3 条（第七节 E158 第 4 条、E156 第 1、2 条），开工时就红，我没硬改。
  - clippy（check.sh 那一套 lint、`--workspace --all-targets --all-features`）、fmt、build 全过。
  - 门禁 27、33、53、80、92、93、94 退 0，89 退 77。74 见第八节。
  - naming-lint 全仓红 230 处，与实三交回时同数。我这两份文件里只有 1 处，在 e156 第 2915 行，HEAD 上就有，不是这一轮加的。
- 推翻条件：
  - 主工作区现状下，第七节列的三条红测在不改代码的情况下转绿；
  - 59 号复跑时第 476、747、748 行有一行没红；
  - 主工作区跑同一套 clippy 报出 e156 的 always-true。

## 二、写过的文件（自己列；与开工快照逐文件比出来的）

- `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（与开工时比：+/− 共 29 行）
- `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（+/− 共 23 行）
- `crates/mutations.tsv`：
  - 换第 476 行：变异名换成「E158 root_choice_repair PC2 的本地系统配置字段表合计默认值退回 481（该在没设环境变量时取今天的 489，D22 已定项 9 字段表合计；实四甲改）」。
  - 追加第 747 行「E156 R-3c 第三组的前提（实四甲把恒真的字面量比较换成现算）：层级 1 位置的除数错用成叶宽，170 号叶退回第一个层级 1 节点」。
  - 追加第 748 行「E158 root_choice_repair PC2 的环境变量不按给的值解、一律解默认 489（实四甲把单测的输入从 489 换成 497，默认值改成 489 之后才分得出这一格）」。
  - 这一轮里第 103 行（E142 量 5）也变了（`twenty_one` → `twenty_nine`），不是我改的，是别的会话在我开工之后改的。

别的会话同时在改 `crates/`，所以下面这份 `git diff --stat -- crates litmus` 分不出谁改的，原样附上：

```
 crates/mutations.tsv                               |  183 +-
 crates/singlefs-checker/src/image.rs               |   77 +-
 crates/singlefs-checker/src/lib.rs                 |  106 +-
 crates/singlefs-checker/src/walk.rs                |  441 ++--
 crates/singlefs-core/src/allocator.rs              |  118 +
 crates/singlefs-core/src/instance_table.rs         |   46 +-
 crates/singlefs-core/src/lib.rs                    |    1 -
 crates/singlefs-core/src/make_filesystem.rs        |   11 +-
 crates/singlefs-core/src/mount.rs                  | 1502 +++++++++----
 crates/singlefs-core/src/mounted_read.rs           |    5 +-
 crates/singlefs-core/src/recovery.rs               |  217 +-
 crates/singlefs-core/src/rollback_witness.rs       |  314 ---
 crates/singlefs-core/src/root_record.rs            |  124 +-
 crates/singlefs-core/src/system_configuration.rs   |  224 +-
 crates/singlefs-core/src/transaction.rs            |  371 +++-
 crates/singlefs-format/src/lib.rs                  |   56 +-
 .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
 ...e142_first_transaction_write_dump_one_device.rs |  121 +-
 .../src/bin/e156_allocation_basis_counts.rs        |  103 +-
 .../src/bin/e158_root_choice_repair.rs             |   60 +-
 .../src/bin/first_transaction_on_device.rs         |  157 +-
 .../src/bin/first_transaction_region_bytes.rs      |    4 +-
 crates/singlefs-harness/src/crash.rs               |  113 +-
 crates/singlefs-harness/src/fault_injection.rs     |   12 +-
 .../src/first_transaction_regions.rs               |   92 +-
 crates/singlefs-harness/src/history.rs             |  279 ++-
 crates/singlefs-harness/src/lib.rs                 |   41 +
 crates/singlefs-harness/src/model.rs               |  294 ++-
 crates/singlefs-harness/src/model_comparison.rs    |   71 +-
 .../tests/checker_known_bad_images.rs              |  753 ++++---
 crates/singlefs-harness/tests/common/mod.rs        |  108 +-
 .../tests/first_transaction_region_bytes.rs        |   78 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |    3 +-
 .../tests/second_transaction_step_five_reuse.rs    |  406 ++--
 .../tests/second_transaction_step_four_rollback.rs | 2244 ++++++++++----------
 ...second_transaction_step_three_formatted_pool.rs |  322 ++-
 ...econd_transaction_step_three_second_instance.rs |    5 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   97 +-
 ...transaction_supplement_three_crash_injection.rs |   34 +-
 ...transaction_supplement_three_fault_injection.rs |   12 +-
 ..._transaction_supplement_three_random_history.rs |  199 +-
 ...transaction_supplement_two_admission_formula.rs |  117 +-
 ...two_c533_row_publish_record_without_its_root.rs |    7 +-
 ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
 ...rop_and_devices_without_the_selected_version.rs |  139 +-
 ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
 ...tion_supplement_two_instance_table_page_full.rs |   77 +-
 ..._two_multi_record_transaction_zero_publishes.rs |    1 -
 ...action_supplement_two_presumed_clause_checks.rs |  105 +-
 ..._transaction_supplement_two_rollback_witness.rs | 1079 ----------
 ...ction_supplement_two_rollback_witness_layer0.rs |  457 ----
 ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |  310 +--
 .../system_configuration_mutability_classes.rs     |   35 +-
 54 files changed, 5867 insertions(+), 6135 deletions(-)
```

## 三、改了什么（文件:行号，行号是交回时的）

E158（`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`）：
- `:172` `parse_local_system_configuration_bytes` 没设环境变量时取 489（原 481）；`:153` 起 `local_system_configuration_bytes` 的文档注释跟着改：
  - 489 出自 D22（单元原子性怎么合成） 已定项 9，`.claude/kb/decisions/22-单元原子性怎么合成.md:189` 整行：

    > **定案**：字段表 = [layout/01-first-txn.md](../layout/01-first-txn.md)「一、mkfs 已经种下的」那一节里的「系统配置字段表」，46 行，合计 **489 字节**，住 4096 字节的系统配置槽（已定项 2），槽内余 3607。字段按**可改性**重新分段：

  - 跑前登记写的 481 是登记那天的今天；
  - 乙族各臂这时要传几，归 E158 的重跑登记定。
- `:6276` 单测改名 `parse_local_system_configuration_bytes_defaults_to_481_when_unset` → `…_defaults_to_489_when_unset`，钉 489。
- `:6284` 单测 `parse_local_system_configuration_bytes_parses_the_given_decimal` 的输入 489 → 497（理由见第四节）。
- `:38`–`:46` shim `mount_rollback` 的文档注释：删掉「这个装置的取样点与判定按新形态重做归实现批次实四」，改成列出用它的几处、为什么问法不再成立、交主 agent 随 E158 重跑登记定。shim 的代码一字没动。

E156（`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`）：
- `:4065`–`:4074` `pc_closed_form_matches_the_registered_anchor_r3c` 最前面加两条前提：
  - `assert_eq!(super::e156_level1_position_of_leaf(61), 0, …)`
  - `assert_eq!(super::e156_level1_position_of_leaf(170), 1, "R-3c：170 号叶应落在第二个层级 1 节点里")`
  - 删掉原来末尾那条恒真的 `assert!(170 * 812 >= 812 * 169, …)`；
  - 文档注释加一句「第三组的前提先用 `e156_level1_position_of_leaf` 现算钉住」。
- `:37`–`:44` shim `mount_rollback` 的文档注释，同 E158 那一处的改法。shim 的代码一字没动。

## 四、改了的单测：改前钉什么、为什么变、新值哪来

| 单测 | 改前钉的 | 为什么变 | 新值哪来 |
|---|---|---|---|
| e158 `parse_local_system_configuration_bytes_defaults_to_489_when_unset`（原 `…_481_…`） | 没设环境变量时取 481 | 默认值照 kb 改成 489 | D22（单元原子性怎么合成） 已定项 9（`.claude/kb/decisions/22-单元原子性怎么合成.md:189`，整行见第三节）；`crates/singlefs-format/src/lib.rs:224` 的 `pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;` |
| e158 `parse_local_system_configuration_bytes_parses_the_given_decimal` | 输入 "489" 解出 489 | 默认值成了 489 之后，输入 489 分不出「解出来的」与「退回默认值」。第五节 M-b 对照量过：输入留 489 时，「一律解默认」那处变异这条单测抓不到 | 497，任取一个与默认不同的数。不代表乙族臂该传几 |
| e156 `pc_closed_form_matches_the_registered_anchor_r3c` | 末尾 `170 * 812 >= 812 * 169`，两边都是字面量，恒真（clippy `assertions_on_constants`） | 派发第 4 条：换成真能失败的断言 | 前提照原断言消息「170 号叶应落在第二个层级 1 节点里」写成 `e156_level1_position_of_leaf(170) == 1`；另加 61 号叶落在第 0 个。这两个数就是本地常量 F = 169 下的 ⌊170 ÷ 169⌋、⌊61 ÷ 169⌋ |

`local_constants_match_crates_today`、`constants_and_anchors_all_pass` 两条没改，默认值改完就转绿了。

## 五、证红记录

草稿副本 `/tmp/claude-1000/impl-rbf-4a/proof/`（`rsync -a --exclude target --exclude .git`，拷的时候这两份 bin 与主工作区逐字节相同），用它自己的 `target-proof`。每次改坏一处，跑那个 bin 的整个单测二进制（`cargo test -p singlefs-harness --bin <bin>`，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`）。还原时从 `originals/` 拷回并 `touch`。脚本 `proof-run.sh`，改坏用 `mutate-once.py`，要求原文恰好命中一次。

基线（不改动的副本）的红集：
- e158：`mount_writable_trajectory_distinguishes_persistent_from_transient_faults`，42 过 1 红；
- e156：`rollback_isolation_scenario_matches_the_new_layout`、`hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`，9 过 2 红。

下表只看基线红集之外的：

| 变异 | 改坏哪一行 | 哪条断言红（基线之外） | 同时红的（基线之外） |
|---|---|---|---|
| M-a（变异表第 476 行） | e158 `:172` `Err(env::VarError::NotPresent) => 489,` → `=> 481,` | `parse_local_system_configuration_bytes_defaults_to_489_when_unset` 在 `:6278` 的 `assert_eq!(value, 489, "没设环境变量应当取今天的 489")`：left 481 / right 489 | `local_constants_match_crates_today`（`:5656`）、`constants_and_anchors_all_pass`（`:5674`）；39 过 4 红 |
| M-b（第 748 行） | e158 `:169` `Ok(value) => value` → `Ok(value) => "489"` | `parse_local_system_configuration_bytes_parses_the_given_decimal` 在 `:6286` 的 `assert_eq!(value, 497, "环境变量给的十进制数要原样解出来")`：left 489 / right 497 | `parse_local_system_configuration_bytes_panics_on_non_decimal_value`（该 panic 不 panic）；40 过 3 红 |
| M-b 对照（不进变异表） | 同 M-b，另把那条单测的输入与期望退回 489 | `parses_the_given_decimal` **不红** | 只有 `panics_on_non_decimal_value` 红；41 过 2 红。说明输入不换，默认值改成 489 之后这一格就抓不到 |
| M-c（第 747 行） | e156 `:168` `leaf / E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT` → `leaf / E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS` | `pc_closed_form_matches_the_registered_anchor_r3c` 在 `:4070` 的 `assert_eq!(…level1_position_of_leaf(170), 1, "R-3c：170 号叶应落在第二个层级 1 节点里")`：left 0 / right 1 | `pc_closed_form_third_group_counts_released_and_allocated_records`（`:4130`，「第 61 与第 170 片叶应给出 9」：left 7 / right 9）；7 过 4 红 |

- 三条变异各证一次，都在这一轮证过，没有留给 59 号头一次跑的。
- 没有 `debug_assert` 挡在前面：红的都是单测自己的 `assert_eq!`。

clippy 那一处另证了一次：同一份副本放回开工时的 e156，跑同一套 clippy，报的是

```
error: this assertion is always `true`
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4080:9
```

lint 是 `clippy::assertions_on_constants`，退出码 101。换回改后的 e156 再跑，退出码 0。

## 六、变异行（`crates/mutations.tsv`）

| 行 | 动作 | 原因 |
|---|---|---|
| 476 | 换（原文、替换文对调；测试名换成改名后的） | 原行钉「默认该取 481」，原文 `=> 481,` 在改后的源码里零命中，锚点腐化。改成「默认退回 481 就红」，就是这一轮改法的「改回去它就红」 |
| 747 | 追加 | e156 恒真断言换成现算前提之后的「改坏就红」（M-c） |
| 748 | 追加 | e158 解数单测换输入之后的「改坏就红」（M-b） |

- 指着删掉的函数的旧行：`grep -n 'mount_rollback\|rollback_witness\|RollbackWitness' crates/mutations.tsv` 零命中，没有要删的。
- 点名的测试今天属第 3 条、开工时就红的，有第 551、656、660、661 行。它们在 59 号复跑时恒红，什么也证明不了。我没删也没改，归第七节的决定，见第九节第 3 条。
- 第 293、294、478 行点名 `constants_and_anchors_all_pass`。这条测试开工时是红的，默认值改成 489 之后转绿，这三行重新有判别力。这三行这一轮没单证，留给 59 号。

## 七、停下来的（派发第 3 条）：模拟的是挂载时回退、换形态之后问法不再成立的几处

两份 bin 里直接调本地 shim `mount_rollback` 的地方共 7 处，全列在下面：`grep -n '[^_]mount_rollback('`，去掉定义与 `format!` 那一行之后，e158 3 处（`:611`、`:1923`、`:4969`）、e156 4 处（`:1281`、`:1533`、`:3230`、`:4291`）。e158 `:611` 在 `apply_mount_rollback` 里，它再被 3 个枚举调用。shim 是实三加的：先 `mount_writable`，这一步取新号、写行、暖机；再 `roll_back_by_a_forward_publish`。每一处在 shim 下编得过、跑得动，但装置记下的东西仍是挂载时回退的样子。条款那一边，`.claude/kb/decisions/23-journal的角色与格式.md:374` 整行：

> **管理员回退是挂着时的一次向前发布**：管理员在一次可写挂载里从回退候选集选一个旧根 R_old，文件系统发一次普通发布（D16（发布语义） 已定项 7 的持久顺序），新根的用户可见树（inode 树、extent 树；`deleted_inodes` 树落地之后也算，C118（`deleted_inodes` 树的形态无落点））的根指针取 R_old 那一版的；树表单元、分配记录树、中央映射树与记账统计量（D5（快照 / 空间记账机制） 已定项 4）按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入（inode 号与树 ID 水位见表里「水位」一格）；实例表照 cur 的；checkpoint_txg 照常加一，**不新增实例、不取号、不写实例表行、不施加也不删除任何记录**。记号：cur = 现行那一版，new = 回退发出的新根，用户可见单元 = 数据单元与 extent 树、inode 树的单元。

同文件 `:378` 整行：

> | 候选集 | 根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效（D16（发布语义） 已定项 1）的根：(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti——被崩溃恢复抛弃的时间线上的根一条都选不中 |

### E158（`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`）

1. `apply_mount_rollback`（`:605`），即 R(j) 一步。
   - 它记 `RollbackEvent { abandoned: 时间线上目标之后那一段 }`；把装置自记的时间线截到目标，再接上 shim 那次可写挂载的写行与暖机；回退那次发布 D 不进时间线。
   - 我在草稿副本里加了一段只打印的探针（没进仓），从 `bootstrap(G0, 1)` 起逐个 R(j)，量到：
     ```
     PROBE1 base_timeline=[(0, 0), (1, 1), (1, 2), (1, 3), (1, 4)]
     PROBE1 target=(1, 3) harness_timeline=[(0, 0), (1, 1), (1, 2), (1, 3), (2, 5), (2, 6), (2, 7)] session_current=Some((2, 8)) abandoned=[(1, 4)] ring_after={(0, 0), (1, 1), (1, 2), (1, 3), (1, 4), (2, 5), (2, 6), (2, 7), (2, 8)} next_M=Err("S5: mount_writable 的 chosen_root (2, 8) 与装置自记的时间线尾 (2, 7) 不一致（不注入故障时应当相等）")
     PROBE1 target=(1, 2) rollback_err=mount_rollback: "挂着回退：TargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) }, exclusion: VersionWithoutFile }"
     ```
     装置记 (1, 4) 被抛弃，而它仍是有效根；回退之后接一步 M 就撞 S5。
   - 三个调用方：
     - H3 的 `enumerate`（`:828`）：把 R 之后的 M 记成「前提不满足（记为前提不满足，不算错）」剪掉（`:820`）；N_w、「活回退事件」轨迹、Q3-2 数的都是 `abandoned`。
     - Q3-1 的 `enumerate_fault_set_violations`（`:1520`）：R 之后的 M 在 `if let Ok` 里不声不响跳过；V1 判的「被抛弃」也来自 `abandoned`。
     - H1 的 `ledger_fault_history_family`（`:2218`）：Q1-1 至 Q1-4、PC1-a/b、op1 第三种变体都挑「被抛弃的可读根」。
   - 回退到没有文件的根（genesis、暖机根）今天在任何写之前拒（`VersionWithoutFile`），这几支到此为止。
2. op1 `FaultedOperationKind::MountRollbackTo`（`attempt_faulted_operation`，`:1923`）：Q1 在故障下做「挂载时回退」这个操作，今天没有这个操作。
3. Q2-2a 固定脚本（`run_fixed_publication_byte_script`，`:4969`）：报 `rollback_row_publish`、`rollback_warm_up_publish_{i}` 的写字节。在 shim 下，这两项报的是那次普通可写挂载的写行与暖机；D 自己的写一次都不报。
4. 红的单测 `mount_writable_trajectory_distinguishes_persistent_from_transient_faults`（`:6425`，停在 `:6494` 的「PC1-a 已经证明这一格会触发，第一步的计数必须 > 0」）：它挑的「被抛弃根」今天按实例表判仍然有效，影子账不读它的分配记录树，第一步计数不 > 0。开工时就红。第 551 行变异点名它。
5. 绿着、前提已经变了的单测 `mount_writable_then_raise_floor_reaches_the_injected_fault`（`:5557`）：
   - 文档说它证明故障走的是影子账那条管道 `isolate_slots_referenced_only_by_abandoned_roots`。
   - 探针量到它绿是走了断言里 `|| with_fault.error_member.is_some()` 那一半：
     ```
     PROBE2 nodes=27 rejected=9
     PROBE2 path=[Overwrite, RollbackTo(1, 3), Overwrite, Overwrite] before=1 target=(1, 3) after=2 abandoned_root=(1, 4) floor=CheckpointTxg(1) without=(Some(0), None) with=(None, Some("RollbackFloorCeilingNeedsUnreadableValidRootTreeTable"))
     ```
   - 故障打的是一条**有效**根的树表，拦下它的是抬 F 算上限那一步，不是影子账。第 659 行变异点名它（改坏的是抬 F 的目标，与回退无关，照样有效）。

### E156（`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`）

1. S1(c) `run_s1c_rollback_isolation_scenario`（`:1435`；`main` 在 `:3402` 调，单测 U11 `rollback_isolation_scenario_matches_the_new_layout` 在 `:4220` 调）。
   - 它钉的是：R-6 每盘隔离 54、D 的 jsn 9、回退后普通重开取到实例 4、记录 (3, 9)、(3, 10) 在环里。这几样全是「回退抛弃 B 到 C、回退写行、暖机」的产物。R-6 那一行，`research/prompts/e156-r3-prereg.md:473` 整行：

     > | **R-6** | S1(c) 场景（A → B → 重开、写行、两次暖机 → C → 回退到 A → 普通重开） | 重开后新开的段 [50304, 50368)；被抛弃时间线独有的槽 B 14 + 写行 10 + 两次暖机各 8 + C 14 = 每盘 **54**（C 的末槽 50342，仍在第 61 片叶）；同一算法按旧布局复算得 34，与 r2 那条断言一致 |
   - 今天普通重开 `isolated=[(0,0),(1,0)]`，断言停在 `:1590`，开工时就红。第 656 行（M32）点名它。
   - `main` 跑到这里同样会停（推的：同一个函数；`main` 没跑）。
2. HR 家族的回退一步（`main` `:3230` 起，`.expect("HR 管理员回退")` 在 `:3236`），第 10 个基底 `beta_hr_rollback_row`（`:3253`），单测 U13 `hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`（`:4232`）。
   - 第 3 次重跑登记「十二」修订 4（`research/prompts/e156-r3-prereg.md:585`，整段很长，这里不抄）把 HR 家族管理员回退那一步的写行结果、第 5 项恰为 0 的那个状态，当成岔路 7 Q7c① 判别力自证第一次有的对象。挂着回退不写行，这个对象不存在了。
   - 在 shim 下，候选按 shim 那次可写挂载之前的环算；可写挂载多发的写行、暖机把最旧的 (1, 52) 挤出了环，U13 停在 `:4292`：
     ```
     U13：HR 管理员回退: "挂着回退：TargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(52) }, exclusion: NotInRing }"
     ```
     开工时就红。第 660、661 行（M33、M34）点名 U13。
   - `main` 的 HR 走同一段前缀、同一个选目标的式子，推的是会在 `:3236` panic，而且在 S1(c) 之前；没跑。
3. HK 家族的回退一步（`run_hk_family`，`.expect("HK 回退")` 在 `:1282`）。
   - 它记「rollback_row」「warmup」两类合法状态（`:1290` 起）与 βK-r。在 shim 下，这两类记的是普通可写挂载的写行与暖机，D 不进合法状态。
   - 岔路 7 的 Q7b、Q7d 与 `q7d2_min_item5` 都在这批状态上数。
   - 没跑，不知道它在 shim 下撞不撞 NotInRing。

我为什么停而不译：
- 能照新形态写的只有「在当前这次挂载里调一次 `roll_back_by_a_forward_publish`」。但上面每一处要么记的是新形态里不存在的东西（被抛弃的段、写行、暖机、第 10 个基底），要么钉的是由它们推出的登记数（R-6 的 54、S1(c) 的 jsn、实例号与五条记录）。
- 照新形态改，要重定这些对象与数，这是重跑登记的事。
- 这些分支今天在 shim 下量出来的数也不可信：E158 的 R 之后 M 被当成前提不满足剪掉；E156 的 `main` 推的是跑不完。

## 八、第 4 步那几样的末尾原样输出与门禁判定行

最后一遍在起跑，主工作区，自己的 `target-main`。测试经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`；fmt、clippy、build 不经内存包装。

`cargo test --offline -p singlefs-harness --bin e158_root_choice_repair`（退出码 101）：
```
failures:
    tests::mount_writable_trajectory_distinguishes_persistent_from_transient_faults

test result: FAILED. 42 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.53s

error: test failed, to rerun pass `-p singlefs-harness --bin e158_root_choice_repair`
```
`cargo test --offline -p singlefs-harness --bin e156_allocation_basis_counts`（退出码 101）：
```
    tests::hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test
    tests::rollback_isolation_scenario_matches_the_new_layout

test result: FAILED. 9 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.44s

error: test failed, to rerun pass `-p singlefs-harness --bin e156_allocation_basis_counts`
```
这三条红就是第七节 E158 第 4 条、E156 第 1、2 条，开工时就红（第五节基线）。其余 51 条全过。

`cargo fmt --all -- --check`：退出码 0，无输出。

`cargo clippy --offline --workspace --all-targets --all-features -- -D warnings`（加 check.sh 那七条 `-D clippy::…`，退出码 0）：
```
    Checking singlefs-harness v0.1.0 (crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```
`cargo build --offline --all-targets`（退出码 0）：
```
   Compiling singlefs-harness v0.1.0 (crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
```

门禁（单跑，`SINGLEFS_GATE_FULL=1`，判定行原样）：
```
== 27-format-constants.sh exit=0
  ✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）
== 33-mutation-tables.sh exit=0
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 743 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
== 53-format-const-placeholders.sh exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
== 80-absolute-assertions.sh exit=0
  ✓ 152 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 148 份，别处 src/bin 下以 e<数字>_ 开头的 4 份）
== 92-layout-checker-sync.sh exit=0
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，105 个格式常量里变了 7 个（checker 在同一次改动里跟了 7 个，按滞后表放行 0 个），都不欠 checker 跟进
== 93-feature-bits.sh exit=0
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 54 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
== 94-checker-implementation-disjoint.sh exit=0
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`；共享模块 1 份源码的正文 283 行里没有分支与循环
== 89-closeout-row27-preconditions.sh exit=77
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
== 74-model-differential.sh（经 run-with-memory-cap.sh 8G、capped.sh 10，release，CARGO_TARGET_DIR 指 target-main） exit=0
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
== naming-lint（.claude/scripts/naming-lint.sh） exit=1
  ✗ 230 处名字不合命名纪律（查了 261 个 .rs 文件、62491 个声明的名字）
```
- 74 号：六段的「模型对拍 N 步」行都在（`/tmp/claude-1000/impl-rbf-4a/logs/gate-74-model-differential.log`）。
- 89 号退 77，是本次未跑，不算通过。
- naming-lint：
  - 全仓 230 处，实三交回时也是 230。
  - 我的两份文件里只有 1 处：
    ```
            crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:2915  变量 k1_1_matches_registered_item1_of_17_and_item5_of_1 —— 「k1」是单字母加数字
    ```
    `git show HEAD:` 那份里同名出现 3 次，不是这一轮加的。
  - 这一轮新声明的名字只有改名后的单测 `parse_local_system_configuration_bytes_defaults_to_489_when_unset`，没被判。
- 派发点名的 27、33、80 全过；naming-lint 没过，原因同上。

## 九、交主 agent 的设计问题

1. 第七节那三条红测怎么处置：e158 `mount_writable_trajectory_distinguishes_persistent_from_transient_faults`、e156 U11 `rollback_isolation_scenario_matches_the_new_layout`、U13 `hr_rollback_row_basis_has_zero_deferred_and_flips_the_q7c1_self_test`。
   - 我看到的三种做法，我没选：
     - (a) 标 `#[ignore = "…"]`，写明等重跑登记；
     - (b) 连同点名它们的变异行一起删；
     - (c) 等重跑登记重定对象与数，再照新数改。
   - 不处置的话，提交时的全量 `cargo test`（check.sh）在这三条上红。
2. 回退一步照新形态怎么译，是重跑登记要定的：
   - 两种译法：
     - 「在当前这次挂载里调一次 `roll_back_by_a_forward_publish`」；
     - shim 的「先可写挂载（取号、写行、暖机），再回退」。
   - 旧形态的回退自带一次重开，新形态不带。E158 的枚举里 M 本来就是单独一步，照前一种译不丢历史；E156 的 HR 前缀从没重开过，两种译法的状态集不同。
   - E158：
     - 被抛弃的根今天只由崩溃恢复造出（实三报告结论第二条）；H1、H3 两族与 Q3-1 都靠「回退抛弃」造对象。
     - 岔路 3 的候选 2（回退见证）随 D23 已定项 14 删了，Q3-2 量的 N_w 没有对象。
   - E156：第 10 个基底、S1(c) 的 R-6 与它的 jsn、实例号、五条记录，以及 HK、HR 两族的合法状态集（岔路 7 的 `q7d2_min_item5` 就在这上面取最小）。
3. 变异第 551、656、660、661 行点名的测试今天恒红，59 号复跑会把它们记成「红了」，什么也证明不了。随第 1 条一起定。
4. E158 跑前登记里按 481 写的锚点、算术与条款，我没改。它们是登记写死的数，改它们等于改实验的问法：
   - 第七节 7.1 锚点（`e158_root_choice_repair.rs:5422` 算、`:5425` 打）：
     - `4096 == 481 + 3615` 两边都是字面量，照旧打 `verdict=pass`，文字是「D22 已定项 9：系统配置字段表合计 481 字节、槽内余 3615、今天没跨 512」；
     - kb 今天是 489、槽内余 3607，这一行在假装通过。
   - 第七节 7.2 算术（`:5330` 起 19 行）：按 481 算的那几行（4096-481、512-481、481+8、389+4+36+52、469+8+4、甲/丙同放 512-481-W）算术本身没错，但「今天 = 481」这个前提不在了。
   - 停机条款 S3，`research/prompts/e158-preregistration.md:910` 整行：
     > - **S3**　某一臂的副本编不过（第八类）⇒ 那一臂停；今天的 `crates/` 与 D22 已定项 9 的 481 对不上 ⇒ 全停。

     按字面今天触发。
   - 乙族各臂，同文件 `:731` 整行：
     > 装置里一律写本地常量，开跑时逐个与 `crates/` 那一份比，不等就停（第十一节 V3）。今天那一臂比 `crates/` 的值；乙族各臂的系统配置字段表比「481 + 8」（本地按臂的定义现算）。

     - 今天这多出来的 8 字节是回退下界 F，与乙族加的 `published_txg` 不是一个字段；
     - 乙族臂这时比 489 + 8 还是比别的数，由重跑登记定。
     - 装置现在的做法：没设 `SINGLEFS_E158_LOCAL_SYSTEM_CONFIGURATION_BYTES` 取 489，设了就用设的值。
5. SysPre 与新生效值的影响（实二报告「写范围之外、这一批牵连到的」第 5、6 条）：
   - 单测里调 `raise_rollback_floor` 的（e158 `mount_writable_then_raise_floor_reaches_the_injected_fault`）是绿的，但它绿的理由已经变了（第七节 E158 第 5 条）。
   - e156 没有单测直接调 `raise_rollback_floor`、`effective_rollback_floor`。
   - 产物没重跑（派发不让跑整轮）。`research/scripts/replay.sh` 第 185、188–201 行 E156 / E158 共 15 行都是 `exact` 对拍，推的是会对不上。
6. 门禁 69 号：两个 bin 与变异表改了，`research/results/` 里没有不比它们旧的 E156 / E158 产物，推的是会红，没跑 69。
7. naming-lint 在 e156 第 2915 行那一处，HEAD 上就有。要改只是一个局部变量改名（同名 3 处）。不在派发范围，我没顺手改。

## 十、没做什么

- 没走三方对抗；层 0、QEMU、herd7、crates 变异整表（59 号）归 `crash-verifier`；没跑全量 `cargo test`、整轮门禁、54 / 55 / 57 / 59 / 69 / 87 号；没提交，没做任何 git 写操作。
- 没跑 E156 / E158 的整轮实验，`main()` 在 shim 下跑不跑得完都是推的。
- 第七节那几处的语义一处没改，三条红测没改、没 ignore、没删。
- 两个 bin 之外的 `crates/` 文件一个没碰（`crates/mutations.tsv` 只改了第 476 行、追加了 747、748 两行）。kb、`research/`、`.claude/` 一处没改。
- 第 293、294、478 行（点名 `constants_and_anchors_all_pass`）这一轮没单证，留给 59 号。

## 十一、草稿与清理

草稿都在 `/tmp/claude-1000/impl-rbf-4a/`。

交回前删掉的（先 `du -sh`）：
- `target-main` 9.4G：主工作区用的编译目录；
- `target-proof` 985M：证红副本的编译目录；
- 证红副本 `proof/` 152M。

删完目录 1.8M，留下的有：
- 这份报告、`progress.md`；
- `logs/`：基线、改后、证红、探针、clippy 核对、门禁、最后一遍的原样日志；
- `start-snapshot/`：开工时的两个 bin 与变异表，单个文件，不是仓副本；
- `originals/`：证红时还原用的两个 bin，单个文件；
- `my-e156.diff`、`my-e158.diff`、`my-mutations.diff`：与开工快照比的差异。`my-mutations.diff` 里含别的会话改的第 103 行；
- `diff-stat.txt`；
- `proof-run.sh`、`mutate-once.py`、`clippy-proof.sh`、`probe-run.sh`、`probe-module.rs`：探针只在副本里加过，没进仓。

没有要入库的产物：没跑实验，证红与探针的日志只是这份报告的底稿。
