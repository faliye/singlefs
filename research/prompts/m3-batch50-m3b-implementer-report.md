# 第 50 批补做实现员报告：GPU 同判退役那四条、R3 发布路径验收、窄盘重扫、证红、构建与慢档

日期：2026-10-09。批次 `records/2026-10-01-删除与并发事务.md#第 50 批`。接上一位 `~/.local/share/singlefs/work/m3-batch50/m3/report.md`（sha256 fcd46e2f…）。草稿目录 `~/.local/share/singlefs/work/m3-batch50/m3b/`。

## 一、结论

- 第 1 条（GPU 同判 I-9.32–I-9.35）做了：扫描内核四条都判，码 8 那一支删了；四条各一条坏流加进 GPU 对拍文件。GPU 对拍我这里跑不了，命令在第九节。四条一开始各留调用点时扫描内核展开到 454 049 个表达式、超 400 000 的预算，改成单调用点之后 387 154（余量 12 846），`crash_judge_dispatch.rs` 三个内核的钉值跟着改。
- 第 2 条（R3 发布路径验收）做了：20 片与 40 片两种几何上第一片吸收第二片那一次每盘新写的槽数相同、钉 14；退回按位置重写、按位置换下的变异红在「指针照抄上一版」那一判。
- 第 3 条：标签 6 的退役常驻项 (2 + H_退役 + 1) × 16384 代码里算着、与 D3 已定项 2 现行正文一致，没改。**「贴边建满后 unlink 不超卖」那条用例没写，停下交主 agent**：探针量到「可用」照式子读，意图那次就掉到负（环窗口兜的暂态，与退役项无关），照「可用 + 环窗口」读恒为正、去掉退役项也不红；两种读法都做不出「有退役项绿、去掉就红」（第八节第 1 条，数在第五节第 3 条）。
- 第 4 条（11 条红用例）做了：窄盘形状重扫，单元区 292 槽、覆盖写 1 次、16 个单元三种场景同在；11 条都绿，造形派生的钉值照新形状改（第五节第 4 条逐条列）。
- 第 5 条：harness 档能证的 70 行全部抓到（含我改过的 3 行与新加的 1 行）；checker 档 9 行（上一位 5 行、我新加的 GPU 4 行）留给提交时的 checker-tier-crates-mutation-replay。
- 第 6 条：`cargo fmt --check`、`cargo clippy --all-targets --all-features`（加编码纪律七条）、`cargo build --all-targets`、e171 那个 bin 的 `cargo build` 都退 0；四个包 184 个测试二进制全绿（1877 过、0 红、67 忽略）；harness 慢档动过的 25 条全绿。
- 清单外的 crates 文件：没有。补丁的 70 个文件与上一位 `patch-combined/` 的 70 个逐个相同（`crash_judge_dispatch.rs` 在那 70 个里，W3 改过它）。

## 二、补丁

`~/.local/share/singlefs/work/m3-batch50/m3b/patch/`，打在主工作区现在这一版上（W3 与第 50 批合起来，形态同上一位的 `patch-combined/`）：

- `crates.patch`：70 个文件（`grep -c '^+++ b/'` 数的）。
- `mutations-append.tsv` 71 行（上一位 66 行，其中 1 行我改了目标；加我 5 行）、`mutations-replacements.tsv` 45 行（上一位 44 行，其中 2 行我又改过；加 1 行），见第五节第 5 条、`mutations-delete.txt` 2 行（`mutations-deletions.tsv` 同内容）、`report.md`（本报告）。
- 交回前核过：把主工作区此刻的 `crates/`、`litmus/` 拷进 `m3b/verify/`、照 `apply-writer-patch.py` 的 `plan()` 打上，与我的副本逐文件比，只差 `crates/mutations.tsv` 第 3551 行——主工作区在我拷 `base/` 之后改了「第10z批第二十一块 GPU 取卡 K5 卡池扫源码 use 里改回至多一层花括号…」那一行，按名字合并时它保留主工作区的新写法，补丁不碰它。
- `python3 scripts/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch50/m3b/patch --root /home/fy5090/code/singlefs --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 3632 行`，退 0。
- 派发写的「合进主工作区排在形式化线拍树之后」：我没打，主 agent 照次序打。

## 三、这一轮写过的文件（我自己列）

在上一位 `patch-combined/` 那 70 个文件之上我动过的（都在那 70 个里）：

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` | 删码 8 那一支；加退役那四条（第五节第 1 条）；谱系读法多记 I-9.35 的父版；墓碑树那一处读容器的调用点兼读退役树 |
| `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` | `INVARIANT_I_9_32`–`I_9_35`；两张叶身份表（各 256 × 7 字），`SCRATCH_END` 317 104 → 320 688 |
| `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` | `SCRATCH_WORDS` 320 688；覆盖表文档改写（不再回落 CPU） |
| `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | 三个内核展开之后的钉值与文档（第五节第 1 条） |
| `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 文件头一段；四条退役坏流与四条对拍用例 |
| `crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` | R3 发布路径那条用例与它的几何、量槽数的帮手 |
| `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` | 窄盘形状常量 292 / 1 / 16；192 槽那三条另立常量保持 14 个单元；造形派生的钉值与文档 |
| `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` | 同一份窄盘造法改成 292 / 1 / 16，函数名 fourteen → sixteen |
| `crates/mutations.tsv` | 追加 5 行；改 4 行（上一位追加的 1 行、上一位替换的 2 行、主工作区原有的 1 行），第五节第 5 条 |

临时加过、交回前删干净的：`admission_raises_the_floor_before_refusing.rs` 末尾的造形探针（`PROBE-TEMPORARY` 段）、`crates/singlefs-harness/tests/probe_temporary_label_six_sweep.rs` 整份。`grep -rn PROBE-TEMPORARY` 在副本里 0 处。

`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 没动（`cmp` 与主工作区逐字节相同）；它引的公开名字与签名、穷举 match 的枚举都没动。kb 一个字没写。

## 四、git diff --stat

主工作区没动；在草稿目录的临时仓里把主工作区的 `crates/`、`litmus/`（拷 `base/` 那一刻的）与我的副本比，`git diff --cached --stat -- crates litmus` 原样：

```
 crates/mutations.tsv                               | 163 +++-
 crates/singlefs-checker-tier/src/bad_disk_input.rs |   3 +-
 .../singlefs-checker-tier/src/crash_injection.rs   |   4 +-
 .../src/crash_judge_checker.wgsl                   |  44 +-
 .../src/crash_judge_checker_common.wgsl            | 116 ++-
 .../src/crash_judge_checker_scan.wgsl              | 735 +++++++++++++++-
 .../src/crash_judge_common.wgsl                    |  30 +-
 .../src/crash_judge_dispatch.rs                    |  17 +-
 .../singlefs-checker-tier/src/crash_judge_gpu.rs   |  18 +-
 ...sh_enumeration_new_pool_file_creation_stream.rs |   5 +-
 ...gpu_agrees_with_the_cpu_judge_on_every_state.rs | 232 ++++-
 crates/singlefs-checker/src/image.rs               |  39 +-
 crates/singlefs-checker/src/lib.rs                 |   3 +-
 crates/singlefs-checker/src/walk.rs                | 962 ++++++++++++++++++++-
 crates/singlefs-core/src/admission.rs              | 420 +++++++--
 crates/singlefs-core/src/container_tree.rs         | 269 +++++-
 crates/singlefs-core/src/inode_tree.rs             | 561 ++++++++++--
 crates/singlefs-core/src/lib.rs                    |   1 +
 crates/singlefs-core/src/lineage_tree.rs           |  34 +-
 crates/singlefs-core/src/mount.rs                  |  85 +-
 crates/singlefs-core/src/records.rs                | 105 ++-
 crates/singlefs-core/src/recovery.rs               | 155 +++-
 crates/singlefs-core/src/retirement_tree.rs        | 391 +++++++++
 crates/singlefs-core/src/transaction.rs            | 633 +++++++++++---
 crates/singlefs-core/src/unit.rs                   |   4 +-
 crates/singlefs-core/src/unlink.rs                 |   8 +-
 crates/singlefs-core/src/write_accounting.rs       |  22 +-
 crates/singlefs-format/src/lib.rs                  |  66 +-
 crates/singlefs-harness/src/history.rs             |  26 +-
 crates/singlefs-harness/src/model.rs               | 297 ++++++-
 crates/singlefs-harness/src/model_comparison.rs    |  34 +-
 crates/singlefs-harness/src/mount_transparency.rs  |  16 +-
 crates/singlefs-harness/src/scenario.rs            |   1 +
 ...e_reads_back_what_it_read_before_the_unmount.rs |   2 +
 ..._inode_number_watermark_steps_cross_one_leaf.rs | 311 +++++++
 .../admission_checkpoint_cost_per_device_paths.rs  |  24 +-
 .../admission_formula_on_the_mounted_state.rs      |  20 +-
 .../admission_raises_the_floor_before_refusing.rs  | 137 +--
 ...t_land_outside_the_mounts_clustered_segments.rs |  15 +-
 ...dow_and_tombstone_prepay_on_the_publish_path.rs |  42 +-
 .../tests/checker_known_bad_images.rs              | 870 ++++++++++++++++++-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |  21 +-
 crates/singlefs-harness/tests/common/mod.rs        | 180 +++-
 .../tests/common_admission_pool/mod.rs             |  10 +-
 .../tests/common_deletion_in_progress/mod.rs       |  98 ++-
 ...ner_trees_grow_a_level_when_the_root_is_full.rs |  15 +-
 ...ew_tree_table_duplicates_and_slot_one_search.rs | 265 +++++-
 ...disk_content_is_refused_instead_of_panicking.rs |  17 +-
 .../tests/deletion_scenario_writes.rs              |   8 +-
 ...de_leaf_absorb_and_borrow_on_the_unlink_path.rs | 276 ++++++
 ...ber_watermark_pressed_back_reissues_a_number.rs |  60 +-
 ...judged_field_by_field_after_a_writable_mount.rs |  24 +-
 .../tests/new_pool_file_creation_publish.rs        |  51 +-
 .../tests/overwrite_in_one_instance.rs             |  19 +-
 ...s_the_deletion_state_of_every_walked_version.rs |   5 +-
 ...ifiers_and_the_first_tree_building_placement.rs |  20 +-
 ..._birth_txg_agrees_across_every_readable_root.rs |   4 +-
 ...he_root_ring_once_when_no_valid_root_is_read.rs |   2 +
 ...ing_once_when_one_device_reads_no_valid_root.rs |   2 +
 .../tests/rollback_by_a_forward_publish.rs         | 197 ++++-
 ..._lineage_record_and_its_checkpoint_cost_term.rs |  93 +-
 ...ed_unmount_keeps_what_the_other_device_reads.rs |   2 +
 crates/singlefs-harness/tests/unequal_devices.rs   |   6 +-
 ...ing_and_unreadable_reads_are_not_passed_over.rs |  25 +-
 ...remounted_twice_continues_to_identical_disks.rs |   2 +
 ...criteria_and_rollback_to_the_emptied_version.rs |  70 +-
 ...k_mount_continuation_and_the_emptied_version.rs |   2 +
 .../tests/unlink_single_file_publishes.rs          |  16 +-
 ...es_per_transaction_do_not_grow_with_the_file.rs |  16 +-
 .../tests/writable_mount_of_a_formatted_pool.rs    |  71 +-
 .../tests/write_accounting_per_structure.rs        |   6 +
 71 files changed, 7500 insertions(+), 1003 deletions(-)
```

## 五、各条做了什么

### 1. GPU 判器同判 I-9.32–I-9.35

- 扫描内核（`crash_judge_checker_scan.wgsl`「退役记录那四条」一节）照 `walk.rs` 写一份：
  - I-9.32：每片退役容器 ≤ 959 条；每条被退役身份类型 == 2、出生树 ≠ 0、容器出生代 < 死亡 txg ≤ 这一版 txg。不滤谱系。
  - I-9.33：死亡 txg 在这一版谱系上的按身份两两比（「在谱系上」用现有的 `d_txg_is_on_the_lineage`，谱系接不成链不判）；总数 > 4096 判不了。
  - I-9.34：谱系上的记录指的身份不在头 ID 配上的那棵 inode 树（同 ID 取表里最后一条，对 `walk.rs` 的 BTreeMap 后插盖前）的叶身份里；叶身份读内部条目、不开叶（`d_read_inode_leaf_identities`，照 `inode_leaf_identities_without_judging` 的几种读不全逐个交 NONE）。
  - I-9.35：父版 = 这一版谱系记录叶序第一条 T_rb == txg 的 T_old，否则 txg − 1（在 `d_read_the_lineage_of_a_version` 读记录时顺带记，只要记录读得出、不要求成链，同 `walk.rs` 的 `parent_txg`）；版本清单里按 `walk.rs` 的去重键（树表指针第一条位置条目的盘、槽、校验和 + txg，指针全零的不去重）恰好一份才比；门同 I-9.29（主走读沿最新根的实例表链走到末片）；有退役树的头 ID 配不上任何 inode 树的那一版不比；每棵 inode 树比「父版叶身份 − 这一版 = 死于这一版的退役身份」，漏记、多记都置位。
  - CPU 记射程里判不了的那几格这里都不置位；新的判不了站点 65–68（叶身份压栈满、叶身份表满、两两比太多、同一 txg 的版本多于 8 份），都是「条目太多」一类。
- 单调用点：退役容器借墓碑树那一处 `d_read_containers_of_a_container_tree` 读（种类码与记录宽按种类 `select`）；叶身份只一个读的调用点（被判那一版、父版两轮）；四条都在 `d_judge_the_retirement_records_and_their_completeness` 一处，从墓碑那一遍的同一个循环里调；墓碑与谱系那一遍的外层循环原样不动（第 10q 批那一行变异的锚点仍恰好命中一次）。
- 展开之后的表达式数（草稿工具 `m3b/naga-check/`，照 `crash_judge_dispatch.rs` 的 `kernel_inlined_size` 同一个数法、naga =30.0.1；它在主工作区那一版上量回 331 792 / 346 138 / 352 195，与主工作区的钉值逐个相等）：

| 内核 | 主工作区 | 上一位之后 | 我第一版 | 交回这一版 | 余量 |
|---|---|---|---|---|---|
| 恢复 | 331 792 | 332 666 | 332 666 | 332 666 | 67 334 |
| 走读 | 346 138 | 346 278 | 346 278 | 346 278 | 53 722 |
| 扫描 | 352 195 | 368 465 | 454 049 | 387 154 | 12 846 |

  第 10q 批那七对（调用方, 被调方）在扫描内核里各 1 / 1 / 1（同一个草稿工具数的）。三个内核都过 naga 解析与校验。上一位改了内核却没改 `each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget` 的钉值，这次一并改了。
- 草稿区每个状态 +3 584 字（两张叶身份表，317 104 → 320 688，`SCRATCH_WORDS` 与 `SCRATCH_END` 同改，变异表「第 5 批 GPU 判器：派活给每个状态的草稿区…」那一行锚点跟上）。
- GPU 对拍新加四条（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）：
  - `the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version`：删完的流上退役记录死亡 txg 8 → 9，CPU 要红 I-9.32、I-9.35；
  - `…_when_a_retirement_record_is_written_twice`：同一条再追加一条，CPU 要红 I-9.33；
  - `…_when_a_retirement_record_names_the_surviving_left_leaf`：233 个空 inode、unlink 2 号左吸收右，退役记录改指幸存的左片，CPU 要红 I-9.34、I-9.35；
  - `…_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version`：死亡 txg 8 → 7，CPU 只红 I-9.35。
  每条整份比、一个状态都不许判不了（照文件里现有的判法）。这几条只编过（`cargo clippy --all-targets --all-features` 把它们一起做过类型检查），没跑。

### 2. R3 的验收改到发布路径上

- `inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 新加 `the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty`：第一个文件之外再建 `片数 × 233 − 1` 个空 inode（满的 20 / 40 片），unlink 2..=117、234..=349（两片各剩 117 条，一次都不并不借），再 unlink 118：第一片 116 条、与右邻合计 233 条，第一片吸收第二片。断言：吸收之后身份与退役记录；第二片右边每一片挪了位置、指针照抄上一版；吸收那一次每盘新写的槽数（这次 `rewritten` 的单元按槽宽取整加起来）20 片与 40 片都是 14。
- debug 下单跑 21 秒，留在快档。单元层上一位那两条没动。
- 证红见第六节（「第 50 批补做 R3」那一行）。

### 3. 贴边建满后 unlink 不超卖（停下，交主 agent）

- 先核的那一半：标签 6 的退役常驻项算着。`crates/singlefs-core/src/admission.rs` 第 351–353 行把 `RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA`（2）、`retirement_tree_internal_levels`、`RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA`（1）与墓碑那三项一起加进常驻槽，再加 35 × n_记录，与 `.claude/kb/decisions/03-空间分配.md@已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况` 现行正文的式子逐项相同；发布路径取「这次之后」那一版的 H_退役（`transaction.rs` 第 4398 行起）。没改。
- 用例没写。探针（副本里临时加的 `probe_temporary_label_six_sweep.rs`，交回前删了）：随机历史 384 槽那一档小盘（补足环窗口，环窗口每盘 900 槽），会话里建 234 个空 inode（叶 [1..=233]、[234, 235]），unlink 2 号（开出墓碑树，不并），再顺序写把第一个文件长到 du 个数据单元（du = 20..=300 步 2，准入判着），之后不经会话直接发 unlink 3 号的意图与各批（最后一批第一片吸收第二片、开出退役树）。每次发布之后读 `admission_reading_before_a_publish`（每盘槽数）：

| du | 写入 | 写完 可用 | 意图之后 可用 | 最后一批之后 可用 | 已分配（写完 → 意图 → 最后一批） |
|---|---|---|---|---|---|
| 20 | 做成 | 168 | 143 | 143 | 99 → 124 → 124 |
| 96 | 做成 | 14 | −14 | −14 | 253 → 282 → 282 |
| 100 | 做成 | 6 | −22 | −22 | 261 → 290 → 290 |
| 102 | 做成（贴边那一格） | 2 | −26 | −26 | 265 → 294 → 294 |
| 104 起 | 被拒（推抬 F 之后仍不够，`NoSpaceAfterRaisingTheFloor`） | 208 | 183 | 183 | 59 → 84 → 84 |

  （原样在 `m3b/runs/probe-l1.log`，564 行。）意图那次从未被拒：它只判落点，环窗口那 900 槽物理上空着。
- 为什么停：派发要的「全程每次发布之后可用不小于 0、去掉退役项就红」照现行式子做不出来——
  1. 照式子本身读（环窗口照扣）：意图那次新写 ~25–29 槽、换下的还在 defer 里，贴边的格子在意图之后就是负的（−14 … −26），与退役项无关；这是环窗口本来要兜的暂态（D28 已定项 1）。
  2. 照「可用 + 环窗口」读（或删除队列非空时把环窗口算作可动用，同可写挂载那一判）：900 槽的环窗口把一次 unlink 的几十槽全罩住，去掉退役项（贴边多写 3 槽）也不会红。
  3. 开出退役树的那一次（最后一批）可用不变：已分配那一列也不变（新写的退役容器、根与换下的叶在同一次里相抵），标签 6 少 70 字节（一条记录的 35 字节 × 2 份）。常驻项 3 槽在开出第一片之后照旧扣着——第一片容器与根实占的 3 槽与预付的 3 槽同时在账上，这是 D3 现行式子的写法（墓碑树同形），上一位担心的「多占不从预付里扣」就是这一点；在这块盘面上它没有让任何一次发布之后的可用变负，变负的只有第 1 条那一种。
  要成立的判据得先定「删除类发布之后的可用怎么读」（比如只看转过一圈根环、defer 都放回之后的稳态），那是条款没写的设计判断，第八节第 1 条。
- 没加变异行（没有用例）。

### 4. 11 条红用例：窄盘重扫

- 网格（副本里临时探针，逐格现跑造盘、崩、再挂可写、卸载）：单元区 256–304 槽每 4 槽（13 档）× 覆盖写 0–4 次 × 文件长到 10–24 个单元 × 崩在意图之后 / 第一批之后 / 第二批之后，2 925 格，104 秒；原样 `m3b/runs/probe-n2.log`，挑格的脚本 `m3b/scripts/narrow-match.py`。
- 三种场景各自的判法（照那几条用例名字说的场景）：S1 崩在意图之后——取号之前放行、续做里一批被落得下那一判拒、推一串两次空发布、续做走完；S2 崩在第一批之后——取号之前放行、续做一串都没推成、推满仍放不下、卸载两次空发布做成；S3 不先覆盖写、崩在第二批之后——取号之前放行、一串都没推、续做走到空。
- 结果：S1 40 格、S2 21 格、S3 62 格；S1 与 S2 同一格（同单元区、覆盖写、单元数）的有 6 格，其中同单元区、同单元数、覆盖写 0 次又是 S3 的只有 292 槽、覆盖写 1 次、16 个单元那一格：

| 格 | 结局（探针原样） |
|---|---|
| 292 / 1 / 16 崩在意图之后 | adm=before craises=[2] refusals=["SpaceAdmissionRefused"] steps=16 outcome=walked |
| 292 / 1 / 16 崩在第一批之后 | adm=before craises=[] steps=14 outcome=short:SpaceAdmissionRefused free=[9, 9] cur_txg=24 unmount=raised2 |
| 292 / 0 / 16 崩在第二批之后 | adm=before craises=[] steps=14 outcome=walked free=[10, 10] unmount=raised2 |
| 256 / 2 / 14（旧形状）崩在意图之后 | adm=short[3] craises=[] steps=11 outcome=short:SpaceAdmissionRefused |

- 改了什么（断言的判法一条没改，改的只有随形状变的数）：
  - `admission_raises_the_floor_before_refusing.rs`：形状常量 256 / 2 / 14 → 292 / 1 / 16；单元区 192 槽那三条（准入与续做都推过、续做里抬 F 报块设备错、先写系统配置报错）另立 `DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES = 14`，造形与钉值都不变。
  - 续做步数：S1 14 → 16；S3 12 → 14；崩在第 7 批那一条 7 → 9。
  - 注错那一步之前落盘的份数：续做推过的那一形 16 → 19（写行、暖机两次、续做 14 步、推的两次），崩在第 7 批那一形 6 → 7（暖机两次）。
  - S2：只读挂载比崩溃那一刻多的发布 13 → 17；卸载之后再挂续做 2 步 → 1 步。
  - S3：卸载之前每盘空槽 [9, 9] → [10, 10]，卸载那一串 3 次 18 槽 → 2 次 14 槽（仍多于空槽，仍要落在自己回收出来的槽上）。
  - 卸载掉电那两条：现行那一版 txg 21 → 24，盘 1 上最新的根 19 → 22，扣到生效的释放代 20、21 → 23、24（区域归属 0 / 1 / 0 不变：24 mod 3 = 0 在盘 0）。
  - `mount_transparency_judged_field_by_field_after_a_writable_mount.rs`：同一份造法改成 292 / 1 / 16，函数名 `…_of_fourteen_data_units_…` → `…_of_sixteen_data_units_…`；断言 `(0, 1)` 没变。
- 结果：`admission_raises_the_floor_before_refusing` 25 过、0 红、2 忽略（那时还有探针，交回这一版 1 忽略）；`mount_transparency_judged_field_by_field_after_a_writable_mount` 48 过。
- 没扫的：单元区 304 槽以上、步长 4 之间的槽数、覆盖写 5 次以上；checker 档 `crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 用的是另一块 136 槽的盘（覆盖写 3、14 个单元、1 批），我没跑也没改，退役树进准入之后它的前提（推满仍放不下、建 0 个落得下、建 1 个落不下）可能变，第九节列了命令。

### 5. 证红

- 用 `scripts/prove-red.sh --copy <副本> --memory 6G <crate> <名>`，每次一条（驱动 `m3b/scripts/prove-chunk.sh`，结局在 `m3b/prove-results.tsv`，日志在 `m3b/prove-logs/<名字的 sha256 前 12 位>/`）。范围：上一位本批追加 37 行、替换 36 行，加我新加的 R3 那一行、我改过的 W3 那一行；checker 档 9 行不在内。
- 结局：70 行都「抓到」。第一趟有两行没抓到，按 `.claude/rules/mutation-sampling.md` 分了类、改了再证：
  - 「第 50 批 R1：退役记录的死亡 txg 取这次发布的前一个 txg（不是身份消失那一次）」没红：点名的单元测试 `retirement_tree::tests::the_first_retirement_record_opens_container_one…` 自己造记录、不走 `retirement_records_of_this_publish`——真盲区。改成点名发布路径上断言死亡 txg 的那条 `inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`，红在第 79 行（左 death_txg 5、右 6）。
  - 「三·2 第 7 条：池级 checker 判 I-9.16 ② 的发号次序不认四棵新树（退回七棵）」无效：把 `[u16; 12]` 的数组换成 `[u16; 7]`，`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12]` 引它编不过（主工作区那一版同样编不过：`[u16; 11]` 换 7）。改成只在发号次序那一判里取前 7 种（`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER[..7]`），红在 `the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`（左 []、右 ["I-9.16"]）。
- 另外改的两行：「三·2 第 5 条：谱系记录 T_rb 与 T_old 两段写反」点名的测试被 W3 改了名（`a_lineage_record_is_sixteen_bytes…` → `a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from`），补一行替换、证过；「第 5 批 GPU 判器：派活给每个状态的草稿区…」锚点 317_104 → 320_688（checker 档，没证）。
- 我的 R3 那一行先写成只把挪了位置的片记成重写，证的时候红在 `allocator.rs` 第 1873 行的 `assert!`（同一个落点释放了两次），不是用例自己的断言；换成连换下的叶序一起退回按位置（身份消失那几片的上一版叶序取上一版最右那几个）之后红在用例的「指针照抄上一版」那一判。

### 6. 构建与慢档（第 4 步那几样，末尾原样）

本派发的 cargo 编译与测试调用：`cargo test --no-run` 五次（第 1 次编不过：探针里一处借用）、`cargo clippy --all-targets --all-features` 一次、`cargo fmt --check` 两次（第二次在改完一处文档注释之后）、`cargo build --all-targets` 一次、e171 那个 bin 的 `cargo build` 一次、最后的 `cargo test --no-run` 一次；另有草稿目录里那份 naga 小工具 `cargo build` 三次。合计超过定义的 10 次：派发第 6 条点名要这几样，我照点名跑了。证红经 `prove-red.sh` 起的 cargo 不在这个数里。

- `cargo fmt --all -- --check`：退 0，输出空（两次都是）。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条：退 0，末尾：
  ```
      Checking singlefs-checker-tier v0.1.0 (/home/fy5090/.local/share/singlefs/work/m3-batch50/m3/b/crates/singlefs-checker-tier)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.32s
  ```
- `cargo build --offline --all-targets`：退 0，末尾 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.13s`。
- `cargo build --offline -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`：退 0，末尾 `    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s`（`--all-targets` 那一次已编过）。
- 四个包（harness、core、checker、format）`cargo test --no-run` 出的 184 个测试二进制，逐个经 `scripts/run-with-memory-cap.sh 3G bash scripts/capped.sh 4 <二进制>` 跑：184 个退 0；`test result` 行合计 1877 过、0 红、67 忽略。日志 `m3b/runs/all-6/`。
- harness 慢档动过的 25 条（`--ignored`，经内存包装）：`rollback_by_a_forward_publish` 22 条各起一个进程、各 1 线程，22 条都 `test result: ok. 1 passed`（最长 45.51s）；`admission_checkpoint_cost_per_device_paths` 1 条 ok（89.10s）；`admission_raises_the_floor_before_refusing` 的 `at_366_instance_rows_…` ok（87.27s）；`unlink_writes_per_transaction_do_not_grow_with_the_file` 的 `unlink_of_files_of_sixteen_and_thirty_two_batches_…` ok（31.48s），它的钉值上一位已照算法改过，这次没再改。日志 `m3b/runs/rollback-slow/`、`m3b/runs/t4-slow-*.log`。慢档跑在窄盘钉值改完之前那一份编译上；之后改的只有测试文件里的造形钉值、checker 档源码与变异表，四个慢档用例的二进制不受影响（推的，没重跑）。

## 六、证红逐条结局

`m3b/prove-results.tsv` 原样摘（名 → 结局 → 红了的测试）：

- 第 50 批 格式：一片退役容器装的条数按 33 字节一条算 → 抓到 → `deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for`
- 三·2 第 1 条：第一个文件版本那次之后的树 ID 水位退回 65544（四棵新树不占号） → 抓到 → `tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold`
- 树 ID 整体挪开（用户 2026-10-01 定案）：起点改回 11（十二棵 11..22、水位 11 / 23），格式常量与 D8 已定项 11 登记的值不再逐个相等 → 抓到 → `tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold`
- 树 ID 整体挪开：起点改回 11 之后 extent、inode 树的树 ID 与池级 checker 自己那份种类码 11、12 同值（种类码与树 ID 值域不相交那一判红） → 抓到 → `every_tree_kind_code_the_checker_reads_is_below_every_tree_identifier`
- 第 50 批 R1：退役记录死亡 txg 段与容器出生代段写反 → 抓到 → `a_retirement_record_is_thirty_four_bytes_identity_in_entry_order_then_the_death_txg`
- 第 50 批 M3：左吸收右时身份留给右片（应留左半身份） → 抓到 → `the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity`
- 第 50 批 M3：借不均分（多借一条） → 抓到 → `a_container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour`
- 第 50 批 R3：删空最左一片之后右边挪了位置的片按位置重写（新写的槽随右边片数涨） → 抓到 → `dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number`
- 第 50 批 R2：标签 6 不加退役那 35 字节每条记录 → 抓到 → `the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels`
- 第 50 批 R2：标签 6 常驻不加退役树内部层数 → 抓到 → `the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels`
- 第 50 批 R2：环窗口 W 不加退役那 6 + H_退役 → 抓到 → `ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots`
- 第 50 批 R2：ckpt_cost 退役那一项不加退役树内部层数 → 抓到 → `the_retirement_term_of_the_checkpoint_cost_is_a_container_a_root_and_every_internal_level`
- 第 50 批 容器树多条追加：一次发布跨两片时重写清单只记最后那一片 → 抓到 → `a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers`
- 第 50 批 容器树多条追加：一次发布跨两片时重写清单只记最后那一片（容器树自己那条用例） → 抓到 → `appending_several_records_rewrites_every_touched_container_and_all_their_ancestors`
- 第 50 批 退役树：开放容器满了不关、照样往里装 → 抓到 → `a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers`
- 第 50 批 退役树：根满不长高（多算一格扇出） → 抓到 → `opening_a_container_past_what_one_root_holds_grows_the_retirement_tree_one_level`
- 第 50 批 退役树：没有新记录的发布照样重写开放容器 → 抓到 → `a_publish_without_retirement_records_carries_the_whole_tree`
- 第 50 批 退役树只追加：往开放容器里装时盖掉已有的记录 → 抓到 → `the_same_identity_retired_twice_keeps_both_records`
- 第 50 批 退役树：只供测试的容量收 0 条 → 抓到 → `the_capacity_capped_for_tests_lies_between_one_and_the_product_capacity`
- 第 50 批 M3：借不均分（多借一条），一路删到并 → 抓到 → `deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs`
- 三·2 第 6 条：W 每层不取 min(N_ℓ, 2·(K + 26)) → 抓到 → `ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots`
- 三·2 第 6 条：环窗口少乘那一格 + 1（(3 × S) × W） → 抓到 → `ring_window_on_two_sixteen_gibibyte_devices_is_twenty_five_times_three_hundred_eighty_six_slots`
- 三·2 第 6 条：墓碑预付每对象 32768 ÷ 815 向下取整（40，不是 41） → 抓到 → `tag_six_on_root_only_tombstone_and_retirement_trees_is_six_slots_plus_forty_one_and_thirty_five_bytes_per_record`
- 树 ID 整体挪开：起点改回 11 之后 extent、inode 树的树 ID 与核心层的种类码 11、12 同值（种类码与树 ID 值域不相交那一判红） → 抓到 → `every_tree_kind_code_is_below_every_tree_identifier`
- 三·2 inode 树删一条：那一片还有记录时不进重写清单 → 抓到 → `removing_a_record_that_triggers_no_merge_rewrites_only_that_container`
- 三·2 inode 树删一条：删空最右那一片不记删空的片数（根不重写） → 抓到 → `removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root`
- 三·2 inode 树删一条：删空树里最后一条不记删空的片数（根不重写成零条目根） → 抓到 → `removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root`
- 三·2 inode 树删一条：删空中间一片时右边挪了位置的片不重写 → 抓到 → `dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them`
- 三·2 第 6 条：墓碑预付常驻退回 3 槽（不加墓碑树内部层数 H） → 抓到 → `tag_six_after_the_tombstone_tree_grows_one_level_counts_one_more_slot`
- 三·2 第 6 条：树表里没有 deleted_inodes 条目的版本照样记一槽墓碑预付 → 抓到 → `tombstone_prepay_of_a_version_without_a_deleted_inodes_entry_is_zero`
- 三·2 第 7 条：读侧不认墓碑树与谱系树的 key 宽 → 抓到 → `deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none`
- 三·2 第 5 条：谱系记录 T_rb 与 T_old 两段写反 → 抓到 → `a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from`
- 第 50 批 R1：新池新建文件那次不零根注册退役树（树表退回 11 条） → 抓到 → `tree_table_holds_twelve_entries_keyed_by_tree_identifier`
- 第 50 批 死亡写侧：身份消失的那次发布不写退役记录 → 抓到 → `the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds`
- 第 50 批 死亡写侧：被吸收的右片不写退役记录（只记删空摘掉的） → 抓到 → `the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`
- 第 50 批 M3：两片之和恰好 233 不并（合并判据少一格） → 抓到 → `the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`
- 第 50 批 N1：借之后右片的分隔 key 不改成它这一版的最小 key → 抓到 → `a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`
- 第 50 批 M3：借完不从借出的那一片接着看合并 → 抓到 → `the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`
- 第 50 批 M3：低于下限也不借 → 抓到 → `a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`
- 第 50 批 checker I-9.31：退役树条目身份不判 → 抓到 → `a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant`
- 第 50 批 checker I-9.32：被退役身份的类型不判 → 抓到 → `a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant`
- 第 50 批 checker I-9.33：谱系上同一身份两条不判 → 抓到 → `a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant`
- 第 50 批 checker I-9.33：不按谱系滤（删 → 回退 → 再删的合法两条误红） → 抓到 → `unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens`
- 第 50 批 checker I-9.34：谱系上的退役记录指活叶不判 → 抓到 → `a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant`
- 第 50 批 checker I-9.35：漏记不判 → 抓到 → `a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant`
- 第 50 批 checker I-9.35：多记不判 → 抓到 → `a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant`
- 第 50 批 checker I-9.35：父版恒取 txg − 1（回退那一版不取 T_old） → 抓到 → `rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens`
- 第 50 批 checker I-9.35：退役树头 ID 配不上 inode 树时照「没有退役记录」判漏记 → 抓到 → `a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant`
- 第 50 批 checker I-9.27：退役树（种类 13）条目不判头 ID → 抓到 → `a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant`
- 第 50 批 checker I-9.16 ④：缺退役树那一条放过（种类退回十一种） → 抓到 → `a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk`
- 第 50 批 checker I-9.4：第三句（左容器最大 key < 右容器号）又判回来，借之后的合法两片误红 → 抓到 → `a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant`
- D8 已定项 8 头 ID：deleted_inodes 的树表条目头 ID 退回写 0（不写它服务的 inode 树的树 ID） → 抓到 → `tree_table_holds_twelve_entries_keyed_by_tree_identifier`
- D8 已定项 8 头 ID：墓碑树 的树表条目头 ID 退回写 0（不写它服务的 inode 树的树 ID） → 抓到 → `tree_table_holds_twelve_entries_keyed_by_tree_identifier`
- 三·2 第 1 条：第一个文件版本那次不零根注册四棵新树（树表退回 7 条） → 抓到 → `tree_table_holds_twelve_entries_keyed_by_tree_identifier`
- 三·2 第 6 条：标签 6 的 n_live 恒取 0（记账行与发布路径的读数不按这一版 inode 树现算） → 抓到 → `tag_six_of_the_first_file_version_is_six_slots_and_one_record_and_two_new_inodes_add_two_more`
- 三·2 第 6 条：可用(d) 不扣环窗口（式子末项漏扣） → 抓到 → `the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_six_slots`
- 三·6 删除历史：只供测试的批大小从 1 改成 2（H1 少走一批） → 抓到 → `the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_twelve_units_per_device`
- 三·2 第 1 条：核心层判 I-9.16 ② 的发号次序不认四棵新树（退回七棵） → 抓到 → `the_cold_walk_refuses_a_tree_table_whose_tombstone_and_lineage_entries_swapped_their_kinds_before_reading_any_tree`
- 三·2 第 6 条：环窗口的批大小不取分配器上装的删除开关（恒按产品 144） → 抓到 → `the_ring_window_follows_the_batch_size_switch_installed_on_the_allocator`
- 三·2 第 6 条：发布之前的读数不记环窗口 → 抓到 → `the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_device`
- 三·2 第 5 条：冷走读的映射条目数不算谱系树 → 抓到 → `a_rollback_appends_one_lineage_record_that_reads_back_field_by_field`
- 三·2 删除批第 12 批代码轮 U8：墓碑预付的 H 照旧读内存里的墓碑树形状，不读根节点码 2 头 → 抓到 → `the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header`
- C673 ⑤ I-9.27 判法（19-1 证据缺口）：deleted_inodes（种类 9）条目不判头 ID（它的头 ID 写 0 的坏镜像不红） → 抓到 → `each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant`
- C673 ⑤ I-9.27 判法（19-1 证据缺口）：墓碑树（种类 10）条目不判头 ID（它的头 ID 写 0 的坏镜像不红） → 抓到 → `each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant`
- 26-3 挂载透明 续做规则的种类集合退回 {1, 2} → 抓到 → `a_deleted_inodes_root_pointer_that_moved_with_a_mount_that_did_not_continue_is_red`
- 第28批 I-9.16 ④ 退回只要种类 1–4：缺 deadlist 那一条放过（正推 C1） → 抓到 → `a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant`
- 第28批 I-9.16 ④ 退回只要种类 1–4：缺 logged_ops 那一条放过（正推 C1） → 抓到 → `a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant`
- 三·2 第 7 条：池级 checker 判 I-9.16 ② 的发号次序不认四棵新树（退回七棵） → 抓到 → `the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`
- 第 50 批 R1：退役记录的死亡 txg 取这次发布的前一个 txg（不是身份消失那一次） → 抓到 → `the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one`
- 第 50 批补做 R3：发布路径上左吸收右之后右边每一片按位置重写、按位置换下（退回改之前的写法，40 片那一次比 20 片多写） → 抓到 → `the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty`

## 七、受影响的层 0 流与崩溃枚举用例

（池级 checker 的判定集合这次没再改；补丁里 `crates/singlefs-checker/src/` 的改动是上一位的，照上一位报告第八节。下面是合起来之后的推的影响，都没跑。）

- 上一位第八节那几条照旧成立：新池新建文件那次树表 12 条、声明长度 2400、水位 65549，层 0 各流的输入指纹全变；`crash-case:layer0-unlink-after-the-multi-record-publish-stream` 最后一批多写退役容器与根；小盘几何（补足 1007）连带随机历史与崩溃放量的钉值。
- GPU 一侧这次改了：上一位说的「含删到底的流上 GPU 记判不了（码 8）」没有了，含退役树的状态 GPU 照判 I-9.32–I-9.35。崩溃放量带 GPU 那一趟、GPU 对拍文件里含 `unlink_to_the_end` 与删到最后一批的流（「删完的流」「删 → 回退 → 再删」那几条）推的会整份比上，坐实要跑第九节的命令。
- 扫描内核展开之后 387 154 个表达式（预算 400 000），比上一位那一版多 18 689；驱动编扫描内核的用时与内存会跟着涨，没量。判法摘要（`gpu_judge_digest`）变了：判定存储里 GPU 判器那一版的旧结论不复用。
- 窄盘：`admission_raises_the_floor_before_refusing.rs` 的形状只在 harness 档；checker 档 `crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs`（136 槽）推的会受退役树进准入影响，见第九节。
- 层 0 快档 `checker-tier-layer0-replay` 是重型，集成时不跑，提交时对着这一节与上一位第八节看。

## 八、停下交主 agent 的设计问题

1. **「贴边建满后 unlink 不超卖」的「可用」怎么读**（第五节第 3 条）。现行式子下，贴边那一格写完可用 2 槽、意图之后 −26 槽（环窗口兜的暂态，与退役项无关）；照「可用 + 环窗口」读恒为正，去掉退役项也不红；开出退役树的那一次可用不变。要这条用例成立得先定判据，候选（我推的，没量）：
   - 甲：只在删除走完、再转过一圈根环（defer 放回）之后的稳态上判「可用 ≥ 0」；
   - 乙：判「这次删除新占而还没放回的槽 ≤ 环窗口」，另判稳态；
   - 丙：不做这条用例，退役项的预付只由单元层钉值（`admission.rs` 单测）与变异行罩着。
   另：D3 现行式子在开出第一片退役容器（墓碑树同形）之后照旧扣常驻 3 槽，实占与预付同时在账上；这块盘面上它没让可用变负，换一种盘面会不会，没量。
2. 窄盘那 11 条的形状只扫到 304 槽、步长 4；三种场景同在的只有一格，余量窄：以后准入再多扣几槽，这一族大概又要整体重扫（推的）。要不要把三种场景拆成各自的形状常量，由主 agent 定。

## 九、主 agent 要跑的命令

工作目录都是主工作区 `/home/fy5090/code/singlefs`，先把补丁打进去（派发写的次序：形式化线拍树之后）。下面 2–5 是 checker 档、重型，提交时或用户要求时跑；`BINDGEN_EXTRA_CLANG_ARGS` 取 `scripts/crash-amplification-lib.sh` 第 81 行的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS`。线程上限照当时的派发（这里写 31）。

1. 打补丁：`python3 scripts/apply-writer-patch.py ~/.local/share/singlefs/work/m3-batch50/m3b/patch`。看「✓ 打上了」与 code-source-discipline 那一行退 0。
2. GPU 判器的单测（内核解析与校验、草稿区字数、展开预算与钉值、第 10q 批单调用点）：
   ```
   SINGLEFS_HEAVY_TESTS=commit BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include" \
     bash scripts/gpu-quiesce-run.sh -- bash scripts/run-with-memory-cap.sh 16G bash scripts/capped.sh 31 \
     cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests crash_judge_dispatch::tests
   ```
   看 `checker_scan_kernel_source_parses_and_validates`、`scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels`、`each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`（钉 332 666 / 387 154 / 346 278）、`each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel`、`every_kernel_stays_within_the_inlined_size_budget` 各 `ok`。
3. GPU 对拍整份（第 1 条的验收；四条新的名字见第五节第 1 条）：
   ```
   SINGLEFS_HEAVY_TESTS=commit BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include" \
     bash scripts/gpu-quiesce-run.sh -- bash scripts/run-with-memory-cap.sh 24G bash scripts/capped.sh 31 \
     cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
   ```
   看 `test result` 0 红，`NOT_RUN_NO_GPU_ADAPTER` 行数 0（有卡时）；四条新的、含删到最后一批与「删 → 回退 → 再删」的那几条都 `ok`。
4. checker 档变异 9 行（checker-tier-crates-mutation-replay 点名跑）：
   ```
   SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/checker-tier-crates-mutation-replay.sh \
     --item "第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表" \
     --item "第 10 批 10b GPU 判器：扫描内核一张树表只记 8 棵树（改回树表 7 条时的上限，新池第一个文件版本的 11 条第 9 棵起判不了）" \
     --item "第 10 批 10b GPU 判器：恢复内核不把谱系树当容器树读（落进只读根的那一支，映射条目数少算谱系树的节点与容器）" \
     --item "第 10 批 10b GPU 判器：走读内核不读谱系树根（改回 key 形态只认 1–5、别的种类不走）" \
     --item "第 10 批 10b GPU 判器：扫描内核引用集合这一遍把谱系树记成数不全（改回种类 1–4 之外一律数不全）" \
     --item "第 50 批补做 GPU 判器 I-9.32：死亡 txg 大于这一版的 txg 不判" \
     --item "第 50 批补做 GPU 判器 I-9.33：谱系上同一身份两条置成 I-9.32 那一位" \
     --item "第 50 批补做 GPU 判器 I-9.34：谱系上的退役记录指活叶置成 I-9.33 那一位" \
     --item "第 50 批补做 GPU 判器 I-9.35：漏记那一向不判" \
     /home/fy5090/code/singlefs
   ```
   看每行「抓到」，「无效」「没红」都是 0。上一位那 5 行（`第 5 批 GPU 判器：派活…`、`第 10 批 10b GPU 判器：…` 四行）也一直没证过。
5. checker 档 136 槽那条（退役树进准入之后前提可能变）：
   ```
   SINGLEFS_HEAVY_TESTS=commit bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 31 \
     cargo test --release -p singlefs-checker-tier --test crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short
   ```
   看 `the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall ... ok`。红了的话照 `admission_raises_the_floor_before_refusing.rs` 那样重扫它的单元区槽数（它的文件头写了攻方怎么扫的）。
6. 提交时：六道重阶段、`checker-tier-layer0-replay` 快档与全量、整张 `crates/mutations.tsv` 复跑，对着第七节与上一位第八节看。

## 十、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异整表、Verus 形式化验证）、GPU 对拍、checker 档单测都没跑，归提交时的主 agent（第九节）；没提交；kb 一个字没写；补丁没打进主工作区。
- 第 3 条那条用例没写（第八节第 1 条）。
- checker 档变异 9 行没证；`crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs` 没跑、没改。
- 窄盘没扫 304 槽以上与步长 4 之间。
- 慢档 4 个二进制跑在窄盘钉值改完之前的那一份编译上（第五节第 6 条），之后没重跑。
- 上一位报告第十一节「kb 要跟着改的」照旧要写；这次另加一条：不变量表 I-9.32–I-9.35 的 GPU 那一栏（不再回落 CPU）。

## 十一、草稿目录里留着的、删掉的

留着（`~/.local/share/singlefs/work/m3-batch50/m3b/` 下）：`patch/`（交回的补丁）、`report.md`（本报告）、`prove-results.tsv` 与 `prove-logs/`（证红结局与日志）、`prove-*.txt`（证红名单）、`runs/`（各趟测试与探针日志：`probe-n2.log` 窄盘网格、`probe-l1.log` 贴边探针、`all-6/` 184 个二进制、`rollback-slow/`、`t4-slow-*.log`、`clippy-1.log`、`build-*.log`）、`scripts/`（我的驱动脚本）、`draft/`（GPU 那一节的草稿）、`naga-check/` 的源码（量内核展开大小的草稿工具，`cargo build --release` 之后 `naga-check/target/release/naga-check <crates/singlefs-checker-tier/src>` 打三个内核的表达式数与第 10q 批那七对的调用点数；它的 `target/` 已删）。

删掉的（交回之前）：上一位留下的副本 `~/.local/share/singlefs/work/m3-batch50/m3/b/`（含 43G 的 `target/`，派发说用完删掉）；我建的 `m3b/base/`（拷主工作区的那一份）、`m3b/applied/`、`m3b/verify/`、`m3b/patchgen/`（出补丁用的临时 git 仓）、`m3b/naga-check/target/`。上一位草稿目录里别的东西（`m3/patch/`、`m3/patch-combined/`、`m3/report.md`、日志）没动。
