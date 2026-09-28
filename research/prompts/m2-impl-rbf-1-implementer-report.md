# 实一（回退改形态：格式与系统配置）实现员报告

时刻：2026-09-26 交回。工作在主工作区；编译一律用自己的 `CARGO_TARGET_DIR`（`/tmp/claude-1000/impl-rbf-1/target-*` 与各副本自己的 `target`），没碰项目的 `target/`。

## 结论

- `SYSTEM_CONFIGURATION_BYTES` 481 → 489，F 8 字节落在槽内 [481, 489)；系统运行量那一档 52 → 60，四档 389 / 4 / 36 / 60；回退见证表跟着常量后移到 [489, 1242)，不与 F 重叠（格式测试钉住 489 与 1242）。
- mkfs 写 F = 0（新常量 `ROLLBACK_FLOOR_AT_MKFS`，第 0 代根与四个系统配置槽共用）；`SystemConfiguration::parse_slot` 读出每个槽的 F，`verified_system_configuration_slots` 按槽序各交一份、`choose_system_configuration` 交回择到那一槽的 F。怎么用不在这一批。
- incompat：mkfs 只打位 1（0x02）；位 0 退役，读者见到位 0（单独或与位 1 同带）走「不认识的 incompat 位」，挂载在任何写之前拒成 `NoValidSystemConfiguration`；checker 同样判 `UnknownIncompatBit`。
- 根记录 flags：核心层读者与 checker 都收位 0（卸载记号），其余位非 0 照旧拒；写出的根 flags 仍恒 0。
- 第一个事务的 21 个区域里只有两份系统配置槽变了，每份只变 5 个字节（偏移 6 与 [155, 159)），见「钉死值」一节。
- 15 条新变异全部证红；基线红集为空。门禁 27 号 crates 这一侧转绿，只剩 E142 装置那一行红。

推翻条件：主工作区现状下重跑这一批动到的 6 个测试二进制有任一条红；或 59 号复跑时这 15 行里有一行没红；或把第一个事务的区域字节与 HEAD 比，系统配置槽之外有区域变了、或槽里变的不止偏移 6 与 [155, 159)。

## 这一轮写过的文件

- `crates/singlefs-format/src/lib.rs`
- `crates/singlefs-core/src/system_configuration.rs`
- `crates/singlefs-core/src/root_record.rs`
- `crates/singlefs-core/src/make_filesystem.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/rollback_witness.rs`（只改模块注释里的偏移）
- `crates/singlefs-checker/src/lib.rs`
- `crates/singlefs-harness/tests/system_configuration_mutability_classes.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_two_rollback_witness.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_two_rollback_witness_layer0.rs`（改了，没跑：名字带 layer0）
- `crates/singlefs-harness/tests/system_configuration_rollback_floor_and_layout_identity.rs`（新建）
- `crates/mutations.tsv`：第 167、170 行改了点名的测试名（测试改名），第 730–744 行追加 15 行（变异名见「变异行」一节）
- `mount.rs` 没动。`e142_first_transaction_write_dump*.rs`、`e156_allocation_basis_counts.rs` 的未提交改动不是这一轮我写的（主 agent 说的 rustfmt 排版），我没碰。

`git diff --stat -- crates litmus` 原样（含那三份不是我改的 bin；新建的测试文件未跟踪，另列）：

```
 crates/mutations.tsv                               |  19 ++-
 crates/singlefs-checker/src/lib.rs                 |  18 ++-
 crates/singlefs-core/src/make_filesystem.rs        |   8 +-
 crates/singlefs-core/src/rollback_witness.rs       |   2 +-
 crates/singlefs-core/src/root_record.rs            |  51 +++++-
 crates/singlefs-core/src/system_configuration.rs   | 180 ++++++++++++++++-----
 crates/singlefs-core/src/transaction.rs            |  11 ++
 crates/singlefs-format/src/lib.rs                  |  26 +--
 .../src/bin/e142_first_transaction_write_dump.rs   |  73 ++++++---
 ...e142_first_transaction_write_dump_one_device.rs | 121 ++++++++++----
 .../src/bin/e156_allocation_basis_counts.rs        |  55 ++++---
 ..._transaction_supplement_two_rollback_witness.rs |   6 +-
 ...ction_supplement_two_rollback_witness_layer0.rs |   8 +-
 .../system_configuration_mutability_classes.rs     |  34 ++--
 14 files changed, 456 insertions(+), 156 deletions(-)
```
新文件：`git diff --stat --no-index /dev/null crates/singlefs-harness/tests/system_configuration_rollback_floor_and_layout_identity.rs` → ` 1 file changed, 354 insertions(+)`。

补丁（只含我的 11 份改动 + 新文件）：`/tmp/claude-1000/impl-rbf-1/impl-rbf-1.patch`，sha256 `5dfdffcbf7d2526dc228b9f1bc20189da4382c8cd464f2cec5696654ae69025b`；`git apply --check -R` 对主工作区现状过、`git apply --check` 对 HEAD 导出（`git archive HEAD`）过。

## diff 摘要（文件:行号，行号是改后文件的现行行号）

- `crates/singlefs-format/src/lib.rs:224` `SYSTEM_CONFIGURATION_BYTES` 481 → 489（注释 222–223 写明 2026-09-26 加 F、偏移 481 起）；`:260-263` 见证表注释改成从 489 起、实三删；`:278` 见证表占位 [489, 1242)；测试 `:522` 起那一条见证表偏移断言 489（`:528`）、末端 1242（`:536`）；`:547` 起那一条槽余量 3615 → 3607（`:552`）。
- `crates/singlefs-core/src/system_configuration.rs`
  - `:29` 新常量 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`；`:32` `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT = 0x01`；`:33` `SUPPORTED_INCOMPAT_BITS` 只含位 1。旧名 `INCOMPAT_FIRST_SSD_LINE_BIT` 从代码里删了。
  - `:53` `ROLLBACK_FLOOR_OFFSET = 481`；`:204` `SystemRuntimeQuantities::rollback_floor: CheckpointTxg`；`:209` 这一档 `FIELD_TABLE_BYTES` 52 → 60。
  - `:330` 写 incompat 用新位；`:409-410` 在实例代号之后断言位置 481、写 F；`:483-485` 读 F；`:535` `incompat_bits_are_mountable` 按新位判布局身份。
  - 测试：`:577` 辅助 `slot_with_first_incompat_byte`；`:642` 改名 `system_configuration_is_489_bytes_in_a_4096_slot_and_round_trips`；`:680` 未知位样本从 0x02 换成 0x04；`:687` 新 `slot_carrying_the_retired_layout_bit_zero_is_refused_like_an_unknown_incompat_bit`；`:725` 新 `the_rollback_floor_is_written_little_endian_at_481_and_read_back_from_the_slot`；`:816` 改名 `the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total`；见证表那条测试的偏移跟着到 489。
- `crates/singlefs-core/src/root_record.rs:17` `ROOT_RECORD_FLAG_UNMOUNT_MARKER = 1 << 0`；`:90` 读者只拒记号之外的位；`:50` 写者注释；测试 `:160` 辅助、`:171` 新 `root_carrying_the_unmount_marker_is_accepted_and_any_other_flag_bit_is_refused`。
- `crates/singlefs-core/src/make_filesystem.rs:45` `ROLLBACK_FLOOR_AT_MKFS = CheckpointTxg(0)`；`:311` 第 0 代根用它；`:354` 系统配置槽用它。
- `crates/singlefs-core/src/transaction.rs:376-385` 系统配置写带 F：本盘两槽自证过的里取最大、两槽都读不出写 0（设计问题 Q1）；`:400` 填进 `SystemRuntimeQuantities`。
- `crates/singlefs-checker/src/lib.rs:219` checker 自己的 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（替掉裸字面量 0x01，93 号从此认得出它）；`:222`/`:224` `ROOT_FLAGS_OFFSET`、`ROOT_FLAG_UNMOUNT_MARKER`；`:256` incompat 判法；`:322` 根 flags 判法；`:150`/`:161` 见证表偏移注释 481 → 489。
- `crates/singlefs-harness/tests/system_configuration_mutability_classes.rs`：两份样本加 `rollback_floor: CheckpointTxg(0)`；两个 sha256 字面量换新（`:85-92`）；481 → 489、52 → 60；测试改名 `each_mutability_class_writes_its_own_field_table_budget_389_4_36_60`。
- `crates/singlefs-harness/tests/second_transaction_supplement_two_rollback_witness.rs:596-600` 改坏见证表条数那一字节从 `bytes[481]` 改成按 `ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT`（489）取。
- `crates/singlefs-harness/tests/second_transaction_supplement_two_rollback_witness_layer0.rs:213-217` `481 + 1 + 2 * 16` 改成按同一个常量算（514 → 522，断言照旧成立）；没跑。
- 新文件 `crates/singlefs-harness/tests/system_configuration_rollback_floor_and_layout_identity.rs`：`:92` mkfs 四槽只带位 1、F = 0、核心层与 checker 都收；`:137` 每个自证过的槽各交自己的 F；`:183` mkfs 之后的系统配置写（取号）照带盘上已有的 F；`:221` 系统配置带位 0（单独 / 与位 1 同带）挂载在任何写之前拒、流不多一步、四槽逐字节不变、checker 判 `UnknownIncompatBit`；`:298` checker 收卸载记号、拒其余位。
- `crates/mutations.tsv:167`、`:170` 点名的测试名跟着改名换（锚点没变）；`:730-744` 追加 15 行。

## 每条测试的证红记录

做法：每条变异一份副本（`rsync -a --exclude target --exclude .git` 拷 `Cargo.toml`、`Cargo.lock`、`crates`），各用各的 `target`，改坏一处，跑那条测试所在的**整个**测试二进制（不按名字挑），记红了哪些；跑完删副本的 `target`。脚本 `/tmp/claude-1000/impl-rbf-1/prove.sh`（第二轮改名后用 `prove2.sh`），判定逐行在 `/tmp/claude-1000/impl-rbf-1/mut/summary.txt` 与 `mut-rerun/summary.txt`，每条的完整日志 `mut/row<N>.log`。debug 构建；被测代码里这几处都是 `assert!`/`assert_eq!`，没有 `debug_assert`，release 下同样红，没另跑 release。

基线（不改的副本，整二进制）原样：
```
baseline [-p singlefs-core --lib] red_count=0 red=[] exit=0
baseline [-p singlefs-format --lib] red_count=0 red=[] exit=0
baseline [-p singlefs-harness --test system_configuration_rollback_floor_and_layout_identity] red_count=0 red=[] exit=0
```
改名之后第二轮基线同样 `red_count=0`（core lib、新测试二进制两项）。基线红集为空。

表中「行」是 `crates/mutations.tsv` 的行号；15 行每行都证过（没有留给 59 号只追加不证的）。

| 行 | 改坏哪一行 | 哪条断言红（原样消息） | 同一二进制里同时红的 |
|---|---|---|---|
| 730 | `singlefs-format/src/lib.rs:224` 489 → 481 | `system_configuration_fits_in_the_slot_with_room_for_one_more_pointer`：`4096 槽余 3607`，left 4096 right 4088 | `the_rollback_witness_table_follows_the_field_table_crosses_512_and_fits_in_the_slot`（共 2 条） |
| 731 | `singlefs-format/src/lib.rs:264-265` 见证表偏移写成 `SYSTEM_CONFIGURATION_BYTES - 8` | `the_rollback_witness_table_follows_…`：`见证表从字段表末尾起：F 占 [481, 489)，见证表从 489 起`，left 481 right 489 | 无（1 条） |
| 732 | `system_configuration.rs:209` 60 → 52 | `the_four_mutability_classes_budget_389_4_36_60_…`：`系统运行量：槽世代号 8 + 整槽校验和 32 + journal tail 8 + 实例代号 4 + 回退下界 F 8`，left 52 right 60 | 另 7 条（凡是写槽的都在 `finish` 的 `系统运行量写出的字节数与字段表不符` 上断言失败）：`a_self_describing_slot_declaring_an_out_of_range_…`、`slot_carrying_the_retired_layout_bit_zero_…`（当时名 `a_slot_…`）、`corrupted_slot_and_unknown_incompat_bit_…`、`system_configuration_is_489_bytes_…`、`the_rollback_floor_is_written_…`、`the_rollback_witness_rides_after_…`、`to_slot_writes_exactly_the_budgeted_bytes_…`（共 8 条） |
| 733 | `system_configuration.rs:410` 写 F 改成 `put_u64(0)` | `the_rollback_floor_is_written_little_endian_at_481_…`：`F 住字段表末尾 [481, 489)、小端`，left `[0; 8]` right `[1, 2, 3, 4, 5, 6, 7, 8]` | 无（1 条） |
| 734 | `system_configuration.rs:485` 读 F 改成 `CheckpointTxg(0)` | `every_self_verified_system_configuration_slot_hands_out_its_own_rollback_floor`：`盘 0 两槽都自证过：按槽序交回两份，各带各的 F`，left `[CheckpointTxg(0), CheckpointTxg(0)]` right `[CheckpointTxg(5), CheckpointTxg(9)]` | `system_configuration_write_after_make_filesystem_carries_…`：`盘 0：取号那一写照带系统配置里已有的 F，不回卷成 0`，left `CheckpointTxg(0)` right `CheckpointTxg(7)`（共 2 条） |
| 735 | `system_configuration.rs:330` 写 incompat 改成退役的位 0 | `make_filesystem_writes_rollback_floor_zero_and_only_the_new_layout_identity_bit_…`：`盘 0 偏移 0：incompat 第一个字节只有位 1`，left 1 | 另 2 条在 `mkfs 写的槽自证得过: NotSelfDescribing` 上红（共 3 条） |
| 736 | `system_configuration.rs:537-540` 读者把位 0 也当受支持的布局身份 | `mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_…`：`只带退役的位 0：挂载拒成池里没有一份可用的系统配置，实际 None` | 无（1 条） |
| 737 | `root_record.rs:90` 改成 `get_u32() != 0` | `root_carrying_the_unmount_marker_…`：`带卸载记号的根照收，别的字段一个不差`，left `None` | 无（1 条；改名前后两轮都证过） |
| 738 | `root_record.rs:90` 改成 `get_u32() & 0 != 0` | 同一条：`flags 0x00000002：卸载记号之外的位非 0 拒收`，left `Some(…)` right `None` | 无（1 条；两轮都证过） |
| 739 | `make_filesystem.rs:354` 系统配置里的 F 写 `CheckpointTxg(1)` | `make_filesystem_writes_rollback_floor_zero_…`：`盘 0 偏移 0：F 那 8 字节是 0` | 无（1 条） |
| 740 | `transaction.rs:381-385` 照带改成 `CheckpointTxg(0)` | `system_configuration_write_after_make_filesystem_carries_the_rollback_floor_already_on_the_device`：`盘 0：取号那一写照带系统配置里已有的 F，不回卷成 0`，left `CheckpointTxg(0)` right `CheckpointTxg(7)` | 无（1 条；两轮都证过） |
| 741 | `singlefs-checker/src/lib.rs:219` 0x02 → 0x01 | `make_filesystem_writes_rollback_floor_zero_…`：`checker 也认这一槽的 incompat 位: UnknownIncompatBit` | `mounting_a_pool_…`：`只带退役的位 0：checker 判不认识的 incompat 位`，left `Ok(SystemConfigurationView{…})`（共 2 条） |
| 742 | `singlefs-checker/src/lib.rs:322` 改成 `!= 0` | `the_checker_accepts_the_unmount_marker_on_a_root_…`：`带卸载记号的根照收: NonZeroFlags` | 无（1 条） |
| 743 | `singlefs-checker/src/lib.rs:322` 改成 `& 0 != 0` | 同一条：`flags 0x00000002：卸载记号之外的位非 0`，left `Ok(RootView{…})` right `Err(NonZeroFlags)` | 无（1 条） |
| 744 | 同 736 那一处 | `slot_carrying_the_retired_layout_bit_zero_is_refused_like_an_unknown_incompat_bit`（core lib）：`只带退役的位 0：带回退见证、系统配置不带 F 的旧镜像：挂不上` | 无（1 条；两轮都证过） |

证红时 `grep -qx` 按裸名比、lib 单测带模块前缀，第一轮 summary 里 lib 那几行印成 NOT-RED；改按 59 号的认法（`^test (\S+::)?名字 ... FAILED$`）重判，15 行全是抓到。第一轮之后把三条以 `a_` 开头的新测试改名（命名检查判「a」是单字母），涉及 737、738、740、744 四行，第二轮对这四行重证（`mut-rerun/summary.txt`），结果同上。

改名的旧测试不算新测试、没另证：`system_configuration_is_481…` → `_489_`、`the_four_mutability_classes_budget_389_4_36_52_…` → `_60_`、`each_mutability_class_writes_its_own_field_table_budget_389_4_36_52` → `_60`（后两条的变异是既有的第 167、170 行，只换了点名的测试名，没跑 59 号复证）。

## 变异行

追加的 15 行（`crates/mutations.tsv` 第 730–744 行；变异名原样，第二段起的文件、原文、替换文、参数、点名测试见表本身）：

- 730：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：字段表合计退回 481（系统配置不带 F 的那一版）（`crates/singlefs-format/src/lib.rs` → `system_configuration_fits_in_the_slot_with_room_for_one_more_pointer`）
- 731：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：回退见证表没跟着字段表后移、压在 F 那 8 字节上（`crates/singlefs-format/src/lib.rs` → `the_rollback_witness_table_follows_the_field_table_crosses_512_and_fits_in_the_slot`）
- 732：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：系统运行量那一档的字节预算退回 52（不含 F）（`crates/singlefs-core/src/system_configuration.rs` → `the_four_mutability_classes_budget_389_4_36_60_and_add_up_to_the_field_table_total`）
- 733：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：写系统配置槽时 F 那 8 字节写成 0、不写字段里的值（`crates/singlefs-core/src/system_configuration.rs` → `the_rollback_floor_is_written_little_endian_at_481_and_read_back_from_the_slot`）
- 734：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：读系统配置槽不读 F、一律交回 0（`crates/singlefs-core/src/system_configuration.rs` → `every_self_verified_system_configuration_slot_hands_out_its_own_rollback_floor`）
- 735：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：写系统配置槽时 incompat 还打退役的位 0（mkfs 起就写旧布局身份）（`crates/singlefs-core/src/system_configuration.rs` → `make_filesystem_writes_rollback_floor_zero_and_only_the_new_layout_identity_bit_into_every_system_configuration_slot`）
- 736：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：读者仍把退役的位 0 当受支持的布局身份（旧镜像挂得上）（`crates/singlefs-core/src/system_configuration.rs` → `mounting_a_pool_whose_system_configuration_carries_the_retired_layout_bit_is_refused_before_any_write`）
- 737：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：根记录读者不收卸载记号（flags 位 0 非 0 也拒）（`crates/singlefs-core/src/root_record.rs` → `root_carrying_the_unmount_marker_is_accepted_and_any_other_flag_bit_is_refused`）
- 738：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：根记录读者对 flags 卸载记号之外的位也收（`crates/singlefs-core/src/root_record.rs` → `root_carrying_the_unmount_marker_is_accepted_and_any_other_flag_bit_is_refused`）
- 739：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：mkfs 往系统配置里写的 F 不是 0（`crates/singlefs-core/src/make_filesystem.rs` → `make_filesystem_writes_rollback_floor_zero_and_only_the_new_layout_identity_bit_into_every_system_configuration_slot`）
- 740：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：mkfs 之后的系统配置写不带盘上已有的 F、回卷成 0（`crates/singlefs-core/src/transaction.rs` → `system_configuration_write_after_make_filesystem_carries_the_rollback_floor_already_on_the_device`）
- 741：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：checker 的 incompat 布局身份还按位 0 判（`crates/singlefs-checker/src/lib.rs` → `make_filesystem_writes_rollback_floor_zero_and_only_the_new_layout_identity_bit_into_every_system_configuration_slot`）
- 742：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：checker 判根槽不收卸载记号（flags 位 0 非 0 也判 flags 非 0）（`crates/singlefs-checker/src/lib.rs` → `the_checker_accepts_the_unmount_marker_on_a_root_and_refuses_every_other_flag_bit`）
- 743：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：checker 判根槽对 flags 卸载记号之外的位也收（`crates/singlefs-checker/src/lib.rs` → `the_checker_accepts_the_unmount_marker_on_a_root_and_refuses_every_other_flag_bit`）
- 744：实一（回退改形态：格式与系统配置，D22 已定项 9 / 15、D15 已定项 4、D22 已定项 7）：读者仍把退役的位 0 当受支持的布局身份（核心层单测那一格）（`crates/singlefs-core/src/system_configuration.rs` → `slot_carrying_the_retired_layout_bit_zero_is_refused_like_an_unknown_incompat_bit`）

改了的旧行：第 167 行（运行配置 36 → 35 那条）点名测试 `the_four_mutability_classes_budget_389_4_36_52_…` → `…_389_4_36_60_…`、变异名里「389 / 4 / 36 / 52」→「389 / 4 / 36 / 60」；第 170 行（节点大小分错档那条）点名测试 `each_mutability_class_writes_its_own_field_table_budget_389_4_36_52` → `…_60`。两行的原文锚点没变。第 168 行变异名里还写着「四档的和仍是 481」，锚点没变，没改（只是名字里的旧数）。

门禁 33 号对整张表：`crates/mutations.tsv 739 条的原文各命中源码一次`（原样见门禁一节）。

## 钉死值的改前、改后

全部只因这一批的格式改动而变：F 8 字节进字段表（481 → 489、52 → 60）、见证表跟着后移、incompat 布局身份从位 0 换位 1。

| 位置 | 钉的是什么 | 改前 | 改后 | 为什么变 |
|---|---|---|---|---|
| `singlefs-format/src/lib.rs:224` | `SYSTEM_CONFIGURATION_BYTES` | 481 | 489 | D22 已定项 9：末尾加 F 8 |
| `singlefs-format` 测试 `:528`、`:536` | 见证表偏移 / 末端 | 481 / 1234 | 489 / 1242 | 见证表起点就是字段表合计 |
| `singlefs-format` 测试 `:552` | 4096 槽余量 | 3615 | 3607 | 489 之后的余量（D22 已定项 9「槽内余 3607」） |
| `system_configuration.rs:209` 与两处测试、harness `system_configuration_mutability_classes.rs` | 系统运行量字节预算 | 52 | 60 | D22 已定项 9 分段表 60 = 8 + 32 + 8 + 4 + 8 |
| 同上测试消息 | 字段表行数 / 合计 | 45 行 / 481 | 46 行 / 489 | 已定项 9「46 行，合计 489」 |
| `system_configuration.rs` 测试 `:650` | 槽里 incompat 第一个字节 | 0x01（位 0） | 0x02（位 1） | D15 已定项 4：位 0 退役、位 1 是布局身份 |
| 同上 | 补齐从哪起全 0 | 481 | 489 | 字段表合计 |
| `system_configuration.rs` 测试 `:680` | 「不认识的 incompat 位」样本 | `|= 0x02` | `|= 0x04` | 0x02 成了受支持的位，改用未分配的位 2 |
| `system_configuration.rs` 见证表测试 | 条数字节偏移 / 第二条 txg 字节 / 改坏的字节 | 481 / [506, 514) / 481 | 489 / [514, 522) / 489 | 见证表后移 8 |
| harness `system_configuration_mutability_classes.rs:89-92` | mkfs 样本槽 sha256 | `666e95617f8902f6b69320000fa87d359e33e37bd9cfed5c5b82b234ee2c134c` | `a065c12485a9ff72117d549852461f5cb4529d664ac2384271f6e0f22d0f167f` | 与旧字节比只差偏移 6（0x01 → 0x02）与 [155, 159)（CRC-32C 4 字节 b4 f2 cc 6f → b3 07 ac 40） |
| 同上 | 第一个事务之后样本槽 sha256 | `5d3e773bf0a429fc4e48f58790f79b305a23beaaeeed69700d73d90835c623f9` | `c67464f93c9021cf354d649f37044041e4b5eec252f23d103a7e8c1328796260` | 同上（CRC d4 8a a2 b1 → d3 7f c2 9e） |
| 同上 | 补齐起点 | 481 | 489 | 字段表合计 |
| harness `second_transaction_supplement_two_rollback_witness.rs:597-600` | 改坏见证表条数的那一字节 | `bytes[481]` | 按常量取，489 | 见证表后移（改前若不改，这一格改的是 F 的第一个字节，I-7.10 那条断言会红） |
| harness `…_rollback_witness_layer0.rs:213-216` | 第二条见证条目末端 | 481 + 1 + 32 = 514 | 489 + 1 + 32 = 522（按常量算） | 见证表后移；`> 512` 仍成立。名字带 layer0，没跑 |

两个旧 sha256 是在 HEAD 的导出（`git archive HEAD`）上用同一份样本现打的，与改前字面量逐字相同；新值在主工作区现状的副本上打；逐字节比对的数据在 `/tmp/claude-1000/impl-rbf-1/evidence/sample-slot-hex-head/*.hex`、`evidence/sample-slot-hex-rbf1/*.hex`（打样本用的测试文件留在 `evidence/zz_dump_system_configuration_samples-{head,rbf1}.rs`）。

**第一个事务的字节**（`first_transaction_region_bytes` 在 HEAD 导出与现状副本上各跑一次，release，产物 `/tmp/claude-1000/impl-rbf-1/evidence/first-transaction-regions-head.out` 与 `…-rbf1.out`）：21 个区域里只有两份系统配置槽（第一个事务那次发布写的槽 1，盘内偏移 4096）变了，其余 19 个区域（16 个单元区域、根槽、两份 journal 记录）sha256 逐字相同；变的两份各只差 5 个字节：

```
device 0 len 4096 sha old 028a34024b10 new e305c8810ee7 diff [(6, '0x1', '0x2'), (155, '0x6b', '0x6c'), (156, '0xf4', '0x1'), (157, '0x7b', '0x1b'), (158, '0x77', '0x58')]
device 1 len 4096 sha old 1ccff8d7abc9 new 5dc1f792b1d2 diff [(6, '0x1', '0x2'), (155, '0x9c', '0x9b'), (156, '0xd5', '0x20'), (157, '0x99', '0xf9'), (158, '0xca', '0xe5')]
```
全值：盘 0 `028a34024b10678733bc2d839862967dcada90ebf6c8554ad2c32f989e093242` → `e305c8810ee77e4ab706c42297d958ba21d2e225e11b4f0b147394767ab0d39e`；盘 1 `1ccff8d7abc99375937c491203fdd01f33305091e0d61cc4c53f34bbcdf7e1fe` → `5dc1f792b1d2c39a0291ebbe9a7a39045423787534e33eabbf7a138f1852dadc`。F = 0 落在旧补齐 0 的 [481, 489)，空见证表整段全 0，所以这两处不添字节差；根记录 flags 仍写 0，根槽不变。mkfs、取号、暖机写的每一个系统配置槽同理都只在偏移 6 与 CRC 上变（由同一个 `to_slot` 写，未逐槽比对）。

注：两边 `first_transaction_region_bytes` 都退 1（`name=impl_region_table_against_writes … writes_outside_the_table=…`），HEAD 上就是这样，与这一批无关（区域表落后于按位置寻址的那几棵树，调度记录把 `first_transaction_regions.rs` 排在实四）。

## 第 4 步那几样的末尾原样输出

`cargo build --offline --all-targets`（`build-main-final.log` 末 1 行，退出码 0）：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.93s
```

`cargo fmt --all -- --check`：退出码 0，无输出（`fmt-check-2.log` 为空）。

check.sh 那一套 lint 的 `cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings -D clippy::wildcard_enum_match_arm …`（退出码 101，`clippy-final.log` 里的全部 error / 位置行）：
```
error: this assertion is always `true`
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4059:9
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 1 previous error
```
唯一一处在 `e156_allocation_basis_counts.rs:4059`，HEAD 上就在（`git show HEAD:…e156_allocation_basis_counts.rs` 第 4058 行 `170 * 812 >= 812 * 169,`），不在这一批的改动里、不是我改的文件。把它之外的目标单独跑同一套 lint：
```
clippy -p singlefs-format -p singlefs-core -p singlefs-checker --all-targets: exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
clippy -p singlefs-harness --lib --bins --test '*': exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```
（`Finished … 0.00s` 是因为上一条 `--keep-going` 已把这些目标检查过、全缓存；那一条对它们一个告警都没报。）

动到的测试二进制，整个二进制跑（`touched-main-final/`，脚本 `run-touched.sh`）：
```
format-lib: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
core-lib: test result: ok. 122 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
checker-lib: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
harness-mutability: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
harness-rbf-new: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.93s
harness-witness: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.39s
```
`second_transaction_supplement_two_rollback_witness_layer0.rs` 改了、按规矩没跑（名字带 layer0）；它只经 `cargo build --all-targets` 编过。

## 门禁判定行

全部在主工作区现状上跑（`nice -n 19 bash .claude/gate.d/<阶段>.sh`；74 号另设 `CARGO_TARGET_DIR=/tmp/claude-1000/impl-rbf-1/target-gate74`、经内存包装与 capped 10）。派发点名的 27、33、80、命名检查，加 stage-owners 登记给我的 33、53、74、89、92、93、94。末行或判定行原样：

- **27-format-constants**（退出码 1）：
```
  ✗ 格式常量在 kb 与实验源码之间对不上：
     research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:111  const SYSTEM_CONFIGURATION_BYTES = 481，而 .claude/kb/layout/01-first-txn.md 定的现行值是 489
```
- **33-mutation-tables**（退出码 0）：
```
  ✓ 148 个实验二进制都有成形的变异表，1672 条变异的原文各命中源码一次；crates/mutations.tsv 739 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
```
- **53-format-const-placeholders**（退出码 0）：
```
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
```
- **74-model-differential**（退出码 0）：
```
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）：
```
- **80-absolute-assertions**（退出码 0）：
```
  ✓ 152 个实验二进制各自至少有一条绝对值断言（research/e7-index-bench/src/bin 下 148 份，别处 src/bin 下以 e<数字>_ 开头的 4 份）
```
- **89-closeout-row27-preconditions**（退出码 77）：
```
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）
```
- **92-layout-checker-sync**（退出码 0）：
```
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 73ba4a4c019b9e3fc9c92f3122bfbbdaee93c321，110 个格式常量里变了 2 个（checker 在同一次改动里跟了 2 个，按滞后表放行 0 个），都不欠 checker 跟进
```
- **93-feature-bits**（退出码 0）：
```
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 55 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:33，常量声明没写在一行里，解不出值））
```
- **94-checker-implementation-disjoint**（退出码 0）：
```
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`；共享模块 1 份源码的正文 304 行里没有分支与循环
```
- **命名检查** `bash .claude/singlefs-ai-sop/scripts/naming-lint.sh <仓根>`（退出码 1）：
```
  ✗ 242 处名字不合命名纪律（查了 263 个 .rs 文件、62045 个声明的名字）
```

门禁读法：
- 27 号：派发提示说改前红在 `crates/singlefs-format/src/lib.rs:223` 的 481（我没在改前跑它）；现在报出的只剩 E142 装置 `:111` 那一行，crates 这一侧转绿。E142 那一侧不归我。
- 命名检查：第一次跑是 245 处，其中 3 处是我新起的以 `a_` 开头的测试名（判「a」是单字母），改名后 242 处，我这一轮写的名字零处。我动过的文件里还剩的是原有的：`system_configuration.rs:751` `a_self_describing_slot_declaring_an_out_of_range_slots_per_region_is_refused_by_name`、`second_transaction_supplement_two_rollback_witness.rs` 的五个 `a_…` 测试与 `:945` 变量 `newer_than_the_abandoned_c`，没动。
- 92 号绿，但 `.claude/gate.d/layouts.tsv` 那一行仍写位 0「第一条纯 SSD 布局线」：它绿是因为 D15 登记表里位 0 那一行的含义列仍是这几个字（退役的位）。见「交主 agent」第 1 条。
- 93 号绿：代码里认出 4 处 feature bit 常量（core 的位 1、退役位 0，checker 的位 1），`SUPPORTED_INCOMPAT_BITS` 被 rustfmt 排成两行、解不出位号，列进「没判位号的」（改前它的值是一个常量名，同样解不出）。
- 74 号是 stage-owners 登记给我的；它跑的是 release 的随机历史测试二进制，我给了自己的 `CARGO_TARGET_DIR`，没碰项目的 `target/`。
- 按主 agent 的通知：preflight-lint（「准入与运行条件」）与 doc-lint 报 `CLAUDE.md` 缺规则引用的红不归实一，照认，我没跑也没改。

我跑之前 `ps` 看过负载：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，也没有别的 `cargo`、`gate.sh`；派发提示说的两个 `e158_root_choice_repair` 进程我开工时已经不在进程表里。没等锁。

## 交主 agent 的

### 写范围之外、这一批牵连到的（要改成什么）

1. `.claude/gate.d/layouts.tsv` 那一行：① 布局名改成 D15 已定项 4 登记表位 1 那一行含义列的原文「第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）」；② 位号 0 → 1；③④ 两列路径不变（格式定义仍是 `crates/singlefs-format/src/lib.rs` 与两份 layout，checker 判定仍是 checker 的三份源码；incompat 位与根 flags 的判法在 `crates/singlefs-checker/src/lib.rs:219`、`:256`、`:322`）。
2. E142 装置 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`（行号现查）：
   - `:111` `SYSTEM_CONFIGURATION_BYTES` 481 → 489；
   - `:277`/`:282` 布局身份常量 0x01 → 位 1（0x02），`SUPPORTED_INCOMPAT_BITS` 只含位 1；`:1386` mkfs 写的 incompat 字节跟着；`:1481-1485` 读者判法跟着（位 0 按不认识的位拒）；`:6742-6743` 那两个自测样本与 `:7106` 的「位 0 置 1」断言跟着；
   - 模型的系统配置写：在实例代号之后（`:1373` `SYSTEM_CONFIGURATION_TAIL_OFFSET` = 469，tail 8 + 实例代号 4 之后，即偏移 481）写 F 8 字节，第一个事务那条流里每一次都是 0；`:6906`、`:6908`（3615 → 3607）、`:6910`（469 + 8 + 4 + 8 = 489）、`:7089`（「481 之后的 3615 字节」→ 489 / 3607）、`:5972` 那一行宽度表跟着；
   - 装置不建模回退见证表（grep `rollback_witness` 零命中），见证表后移对它没有影响。
   - 复跑后期望：与 crates 的 21 个区域比对只在两份系统配置槽上与旧产物不同，而且只在偏移 6 与 [155, 159)（上面「钉死值」一节有 crates 这一侧的新旧 sha256）。
3. kb（书记员）：
   - `.claude/kb/feature-bits.md` 记账表：位 1「名称」`—` → `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT`；位 0 的名称列要不要从 `INCOMPAT_FIRST_SSD_LINE_BIT` 换成代码里现在的 `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT` 由你定（旧名已从代码删）；「代码侧引用了哪几位」那一段过时：core 两个常量、`SUPPORTED_INCOMPAT_BITS` 只含位 1，checker 不再是裸字面量 `0x01`，改成自己的具名常量（位 1），93 号现在罩得到它。
   - `.claude/kb/layout/01-first-txn.md` 七节末行「系统配置槽轮换写 | 481 / 4096」仍写 481。
   - `.claude/kb/invariants.md` I-7.9 状态格「带卸载记号那一支与根记录 flags 位 0 未实现」：读者（core 与 checker）已收位 0，打记号的写者与 I-7.9 按记号分支仍没有。
4. E158 装置（`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`，我没动）：它的本地常量 `SYSTEM_CONFIGURATION_BYTES` 默认取 481（`:146`），与 crates 的 489 比对（`:205-207`）→ bin 单测 `local_constants_match_crates_today`（`:5629`）与 `constants_and_anchors_all_pass`（`:5648`）会红（读代码推的，按规矩没跑这个没动过的二进制）；59 号的第 299、300 行点名的正是 `constants_and_anchors_all_pass`，基线就红。装置跑整轮时 5.7 常量回比（V3）会停。它的「今天」臂 481 现在变成了乙族的 489，改不改、怎么改是 E158 那边的判断（调度记录把 E158 装置排在实四）。
5. `crates/singlefs-harness/tests/checker_known_bad_images.rs:2280` 的 `.expect("481")` 是见证表偏移那一行的消息文字，值取自常量、已跟到 489；我没动这份文件（动了就得跑整个二进制），只是消息过时。
6. `crates/mutations.tsv` 第 168 行变异名里「四档的和仍是 481」过时，锚点没变，没改。
7. 门禁 69 号可能因为 `crates/mutations.tsv` 变了而要一份不比它旧的产物；不归我，没跑。

### 条款没写清、我取了一个的（设计问题）

- **Q1 非抬 F 的系统配置写带哪一份 F**（`crates/singlefs-core/src/transaction.rs:376-385`）。取号、发布末尾的轮换、取号失败的回卷，这几种写都要往 [481, 489) 填一个 F。D16 已定项 1 只写了「抬 F 那一串」先写系统配置、已写进的不回卷，没写别的写带什么。我取「这块盘两槽里自证过的那几份的最大值；两槽都读不出写 0」，理由是它保住 I-7.12 判的「本盘两槽的最大值」不降。可选的另外两种：照抄择到的那一槽（与见证表同一取法）、整池的 F_生效。实一里系统配置的 F 只可能是 mkfs 写的 0，三种取法写出的字节相同。⚠️ 两槽都读不出时写 0 在实二之后可能不对：那块盘的新槽自证得过、F = 0，而那块盘上的根可能带着更大的 F，I-7.12 会红；整池 F_生效 那一种没有这个问题。测试 `system_configuration_write_after_make_filesystem_carries_…` 只钉「照带、不回卷成 0」（四槽种同一个 F），不钉取哪一种。
- **Q2 根记录里的卸载记号不进 `RootRecord`**。读者收位 0 之后把它丢掉，`to_slot` 恒写 0。实一没有谁打记号、挂载与恢复也不按它分支，所以没加字段；实二的卸载操作要写它时，是给 `RootRecord` 加字段（18 处构造要跟）还是走写者参数，要定。
- **Q3 checker 也收了 flags 位 0**（`singlefs-checker/src/lib.rs:322`）。派发第 4 条没点名 checker；我按同一条 D22 已定项 7 让两个读者一致（否则带记号的根 core 收、checker 判 `NonZeroFlags`）。checker 不按记号给 I-7.9 分支（实二）。如果你要 checker 这一格留到实二，撤掉 `:322` 那一改与第 742、743 行变异、新测试 `the_checker_accepts_the_unmount_marker_…` 即可。
- **Q4 旧镜像被拒时报的是「池里没有一份可用的系统配置」**（`RecoveryFailure::NoValidSystemConfiguration`），不是一个说出「不认识的布局位」的成员。这与今天所有不认识的 incompat 位同一条路（`parse_slot` 返回 `NotSelfDescribing`、这一槽不可择），我没改分流；要不要给「布局位不认识」单开一个错误成员，条款没写。
- **Q5 实一到实三之间写出的镜像带位 1、却仍有回退见证表与回退行**。D15 位 1 的含义写着「没有回退见证与回退行」；在实三删掉它们之前，这一段代码写出的盘与位 1 的含义不符。格式第一个外部用户出现之前是软的，只提醒：这段时间若有产物把盘上字节留进 `research/results/`，它们是「位 1 + 见证表」的过渡形态。
- **Q6 常量名**。「常量名跟着位号改」：新位叫 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT`（0x02），退役位叫 `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT`（0x01，只在测试与文档里用，留着是为了「用过的位不回收」写在代码里）；旧名 `INCOMPAT_FIRST_SSD_LINE_BIT` 删了。名字按含义起，没把位号写进名字。

条款没写、不是分支的，没加：`RootRecord` 上的卸载记号访问器（Q2）；checker `SystemConfigurationView` 上的 F（C556，实二）；`SystemConfiguration` 上「本盘两槽 F 取大」之类的辅助函数（Q1 定了再说）。

## 草稿与产物（都在 `/tmp/claude-1000/impl-rbf-1/`，没进仓）

写范围只放行 `crates/`、`litmus/` 与这个目录，`research/results/` 不在里面，所以下面这些没落进仓；要留的由你拷：
- `impl-rbf-1.patch`：这一批的补丁（sha256 见上）。
- `mutations-append.tsv`、`make_rows.py`：追加的 15 行与生成它们的脚本；`prove.sh`、`prove2.sh`：证红脚本；`mut/summary.txt`、`mut-rerun/summary.txt` 与 `mut/row<N>.log`、`mut-rerun/row<N>.log`：证红判定与原始日志。
- `evidence/`：第一个事务 21 个区域的新旧导出（`first-transaction-regions-{head,rbf1}.out`）、两份样本槽的新旧字节（`sample-slot-hex-{head,rbf1}/`）、打样本用的草稿测试（`zz_dump_system_configuration_samples-{head,rbf1}.rs`）。
- `touched-main-final/*.log`、`clippy-final*.log`、`build-main-final.log`、`fmt-check-2.log`、`naming-lint-2.log`、`gate-final-*.log`：第 4 步各项的完整输出。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表（59 号）归 `crash-verifier`；没提交，没做任何 git 写操作。
- 没跑名字带 layer0 的测试二进制（改过的 `second_transaction_supplement_two_rollback_witness_layer0.rs` 只编过）；没跑全量 `cargo test`、`check.sh`、`gate.sh` 整轮、54 / 55 / 57 / 59 / 87 号。
- 没跑没动过的测试二进制：E158 bin 单测会红是读代码推的（上面第 4 条）；其余二进制里有没有别处钉着系统配置槽的字节，我按 `grep` 查了（`481`、`[6]`、`incompat`、`SYSTEM_CONFIGURATION_CHECKSUM_OFFSET` 的用处），除上面列的之外没找到，但没跑它们证实。
- 没删回退见证表（实三）；`mount_rollback*`、择根、生效值取 max、抬 F 先写系统配置、卸载操作、checker 读系统配置里的 F（C556）、I-7.12、I-7.9 两支都没碰（实二、实三）。
- 没改 `.claude/gate.d/layouts.tsv`、E142 装置、kb（不在写范围，要改成什么写在上面）。
- 没给 crates 里的 bin 加 `preflight`（主 agent 通知：另排）；preflight-lint 与 doc-lint 那两处红照认，没跑。
- 收到主 agent 的一条通知（在读代码的阶段、动手改代码之前收到，没记下时刻）：规范副本同步到 0.0.59、`preflight` 改造另排、那两道红不归实一；实一范围不变，我照原范围做，没因此改任何东西。

## 交回前删掉的编译目录与仓副本

先 `du -sh` 记了大小再 `rm -rf`（要留的样本字节、区域导出与草稿测试先拷进了 `evidence/`）：
- `/tmp/claude-1000/impl-rbf-1/target-main`：11G（主工作区用的编译目录）
- `/tmp/claude-1000/impl-rbf-1/target-gate74`：782M（跑 74 号用的编译目录）
- `/tmp/claude-1000/impl-rbf-1/digest-new`：1.4G（现状副本，含它的 target）
- `/tmp/claude-1000/impl-rbf-1/digest-old`：1.4G（HEAD 导出，含它的 target）
- `/tmp/claude-1000/impl-rbf-1/apply-check`：5.7M（`git apply --check` 用的 HEAD 导出）
- `/tmp/claude-1000/impl-rbf-1/mut/baseline` 与 `mut/row1`–`mut/row15`：各 5.7M（证红副本，target 跑完已删，只剩源码）
- `/tmp/claude-1000/impl-rbf-1/mut-rerun/baseline` 与 `mut-rerun/row1`–`row4`：各 5.8M（第二轮证红副本）

留着的只有日志、判定、补丁、脚本、`evidence/` 与本报告，整个目录现在 844K。
