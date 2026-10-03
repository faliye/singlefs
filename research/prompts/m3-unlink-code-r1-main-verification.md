# 删除批（三·2 unlink）合入的代码：代码轮第一轮判决（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 L1 L2 L3 L4 L5 L6 L7 L8 A1 A2 A3 A4 A5 A6 A7 N1 N2 N3 O1 O2 O3 O4 U1 U2 U3 U4 U5 U6 U7 U8 U9 U10 U11 U12 U13 U14 F1 X1 W S K -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-unlink-code-r1-body.md`、`_m3-unlink-code-r1-background.md`、`_m3-unlink-code-r1-diff.md` | — |
| 开工快照 | `research/prompts/m3-unlink-code-r1-snapshot/`（crates 60 份、kb；冻结副本 `/tmp/claude-1000/m3-unlink-code-r1/tree/`） | `snapshot-check.sh m3-unlink-code-r1`：「70 个文件里 1 个对不上」，是 `.claude/kb/decisions/13-验证路线.md`（快照之后别的线改过）；腿只有正推引了它的第 138、144 行，两处在今天的文件里逐字对得上（下一行 cite-check） |
| 云端正推（Sonnet） | `research/prompts/m3-unlink-code-r1-sonnet-output.md`（451 行，sha256 8229fc7f…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-unlink-code-r1-opus-output.md`（sha256 531f6fe0…），模型 `…-opus-model/` | 主 agent 在冻结副本现查：`crates/singlefs-core/src/mount.rs` 第 4370–4378 行 `walk_failed` 里「writes_of_persisted_publishes: Vec::new(),」（A1）；第 4316–4319 行文档末句「三种都是已经发成的那几步与推成的那几串留在盘上，下一次挂载接着走。」（A6）；`crates/singlefs-harness/src/history.rs` 第 1333–1335 行「模型比不了已落盘的那几版——执行器不调。」（A3）；`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 在冻结副本 `crates/singlefs-harness/tests/` 下 0 处、`mount.rs` 里 6 处（A5） |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root . --background <背景材料>`：「✓ 核了 286 处引文，对上 286 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 只改尾叶与幂等 | 计划与 K-R3-1 一致；探针把 145–431 个单元的文件按产品批大小 144 删到底逐版 checker 无红；`is_entirely_kept` 的 `<` 改 `<=` 单测 7 条全绿、探针红；入库删除用例全是 3 个单元、批大小 1；幂等 ①、③ 只部分真判 | A7：模型只罩一个数据单元的文件，随机历史里从没出现多片叶的删除批 | **打中覆盖缺口**：U2、U13 |
| K2 续做、走空与推抬 F | 走空共用、外层循环各一份；「每次被拒各算一次」两处清零改成永不清零 5 条新用例全绿、无变异行；「不归空间不够」的分类无用例；战役用例在 Continuation 段 `continue` 跳过；挂载卡住时整个挂载报 `MountError::Publish` 规则没说，与 D16（发布语义） 已定项 1「准入」行「挂载处推满仍不够时挂载照样做成可写」方向相反 | A1 续做交回的错里已落盘发布的写账恒空（副本上落盘 13 次、错里记 0）；A2 回退到意图那一版之后再改，会话先走空落盘 4 次、再拒成 `NoFileVersionToChange`，拒绝不带那 4 版；A4 没崩过的那次挂载在续做里交回错时，崩溃注入把续做的写归进暖机段（推的）；A6 「下一次挂载接着走」在入库用例的窄盘上不成立，换「挂载照样做成可写」再卸载也一样卡；N1 续做推抬 F 那一段 69671 个崩溃状态恢复后 0 违例 | **打中**：U1、U3、U4、U9、U10、U12；A6 与「挂载照样做成可写」的方向交用户（第四节） |
| K3 准入 | 环窗口手算 368 / 9200 槽与代码相等；墓碑预付数值等价、H 取内存形状（字面弱）；落点那一判对删除类发布要的槽数规则没说（量到 544–549 对真写 12–17） | N2、N3 没打中：准入开着时删除类发布一次都没被空间拒；O1 落点需求约为实写的 27 倍，没造出误拒；O2 unlink 之后可用量反降 20–24 槽/盘 | **站住**（一次抽样）；H 改按条款字面读 U8；O1、O2 记欠账，交 E168（删除借保留量的环窗口） 第二段在真实现上量 |
| K4 回退、删空与读者 | 一致、真判；冷走读的 nlink 那一支用例判得住但变异表无对应行 | 没专门攻 | **站住**；补变异行 U6 |
| K5 新树与 checker | 一致；inode 树长高本批没做（条款没说含不含）；新加五个宽没有 `format-const` 标记 | 没专门攻 | **站住**；U7 |
| K6 模型与随机历史 | 新种子 1760 段含 unlink 全 Completed；第三个成员上的用户回退 / 抬 F 被跳过；换表后变异行 147 不红（与 478、481 同类）、319 归属没量、1327 没跑完 | A3 执行器碰到删除中的版本不调用户改动，F1「会话里每次先走空」在随机历史里从没被模型比过（512 段里 4 次全跳过）；A5 Q-T 改钉之后取号前那一拒没有用例钉了（副本上照样走得到、一个写都不发） | **打中**：U5、U11、U14 |

## 三、打中的改法（零轮，并进下一批 crates）

| # | 改法 | 验收 |
|---|---|---|
| U1 | 「每次被拒的删除类发布各算一次准入」两处计数各补一条用例与变异行 | 清零改成永不清零的变异红 |
| U2 | 真发布路径上多片下段叶的删除用例（产品批大小 144、文件 145 个单元以上） | `is_entirely_kept` `<` 改 `<=` 的变异红；逐版 checker 全绿 |
| U3 | 「至多一条 key」多出那一支的拒绝归「不是空间不够」那一组，补用例 | 改归空间不够的变异红 |
| U4 | 崩溃注入战役不再在 Continuation 段 `continue` 跳过 | Continuation 段至少一个写被判 |
| U5 | 窄盘、不补足环窗口的用例撞挂载取号前那一处落点拒（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`），一个写都不发；让 `crates/mutations.tsv` 第 147、478、481 行点名它；第 319 行量归属；第 1327 行 cargo 参数补用例名 | 这几行证红 |
| U6 | 冷走读 nlink 那一支补变异行 | 证红 |
| U7 | 新加的五个宽挂 `format-const` 标记（照 `.claude/rules/format-evolution.md`「改一个格式常量」） | 门禁 code-source-discipline 的 format-constants 格认得它们 |
| U8 | 墓碑预付的 H 从墓碑树根节点码 2 头现读（D3（空间分配） 已定项 2 字面） | 与内存形状不一致时用例红 |
| U9 | 续做交回错时带上已落盘各次发布的写账（`mount.rs` 第 727 行契约） | 副本那一形错里的写账等于落盘次数 |
| U10 | 会话那种拒绝（`NoFileVersionToChange`）带上走空发成的那几版与推的抬 F | 拒绝里的版本数等于走空落盘数 |
| U11 | 执行器不再跳过删除中的版本，模型比走空那几版 | 512 段那 4 次被比到 |
| U12 | 崩溃注入按 U9 交回的写账划续做段，不并进暖机 | 续做里交回错的那次挂载，续做的写落在 Continuation 段 |
| U13 | 模型罩多个数据单元的文件，随机历史里出现多片叶的删除批 | 至少一段历史里删除批 ≥ 2 片叶 |
| U14 | `mount.rs` 第 4316–4319 行文档「下一次挂载接着走」按第四节的定案改写 | — |

A4、A6 都是攻方推的或只在副本上量过；U9–U14 被攻过零轮，改完进代码轮第二轮。

## 四、交用户

挂载续做推满抬 F 仍不够时：今天整个挂载报 `MountError::Publish`、已发成的留在盘上；D16（发布语义） 已定项 1「准入」行写的是「挂载处推满仍不够时挂载照样做成可写」，管的是写行之后那一判，没点名续做。攻方量到（A6）在入库用例的窄盘上两种做法之后都卡在下一次挂载的取号前；N3 在准入开着的 126 + 22 + 512 段上删除类发布一次都没被空间拒（一次抽样）。交用户：续做卡住时挂载失败，还是照 D16 那一行做成可写。

## 五、被判的文件

这一轮判的 `crates/` 文件（冻结副本）：`crates/singlefs-checker/src/lib.rs`�`crates/singlefs-checker/src/walk.rs`�`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`�`crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs`�`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs`�`crates/singlefs-checker-tier/src/crash_injection.rs`�`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs`�`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`�`crates/singlefs-core/src/admission.rs`�`crates/singlefs-core/src/allocator.rs`�`crates/singlefs-core/src/container_tree.rs`�`crates/singlefs-core/src/deleted_inodes.rs`�`crates/singlefs-core/src/extent_tree.rs`�`crates/singlefs-core/src/inode_tree.rs`�`crates/singlefs-core/src/lib.rs`�`crates/singlefs-core/src/lineage_tree.rs`�`crates/singlefs-core/src/mounted_read.rs`�`crates/singlefs-core/src/mounted_session.rs`�`crates/singlefs-core/src/mount.rs`�`crates/singlefs-core/src/records.rs`�`crates/singlefs-core/src/recovery.rs`�`crates/singlefs-core/src/tombstone_tree.rs`�`crates/singlefs-core/src/transaction.rs`�`crates/singlefs-core/src/unit.rs`�`crates/singlefs-core/src/unlink.rs`�`crates/singlefs-core/src/write_accounting.rs`�`crates/singlefs-format/src/lib.rs`�`crates/singlefs-harness/src/fault_injection.rs`�`crates/singlefs-harness/src/history.rs`�`crates/singlefs-harness/src/model_comparison.rs`�`crates/singlefs-harness/src/model.rs`�`crates/singlefs-harness/src/reference_filesystem_translation.rs`�`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`�`crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs`�`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`�`crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`�`crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs`�`crates/singlefs-harness/tests/a_floor_raise_refused_for_space_counts_as_short_of_space.rs`�`crates/singlefs-harness/tests/common_admission/mod.rs`�`crates/singlefs-harness/tests/common_admission_pool/mod.rs`�`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs`�`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`�`crates/singlefs-harness/tests/model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file.rs`�`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`�`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`�`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`�`crates/singlefs-harness/tests/random_histories.rs`�`crates/singlefs-harness/tests/release_checksum_quarantine.rs`�`crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs`�`crates/singlefs-harness/tests/unequal_devices.rs`�`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`�`crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs`�`crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs`�`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs`�`crates/singlefs-harness/tests/unlink_single_file_publishes.rs`�`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`�`crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs`�`crates/singlefs-harness/tests/write_accounting_per_structure.rs`�`crates/singlefs-harness/tests/write_entries_refuse_an_inode_being_deleted.rs`�`crates/singlefs-harness/tests/zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in.rs`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 16 | 不影响 | 2026-10-03 不受影响：七条照实现；补用例、补变异与第 6 条续做的错型进下一批 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-10-03 不受影响：环窗口式子与代码相等；落点那一判的口径与 unlink 后可用量下降记欠账 |
| D3（空间分配） 已定项 2 | 不影响 | 2026-10-03 不受影响：墓碑预付数值等价，H 改字面读法进下一批 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-03 不受影响：续做卡住时挂载结局交用户 |

用户 2026-10-03 弹窗定（原样）：「照 D16 做成可写（推荐）」。U14 与挂载续做的结局照这一句改：续做推满抬 F 仍放不下时挂载照样交回可写、队列留着，之后会话里每次用户改动先走空、走不动报空间不够，正常卸载照常；条款补句随删除批写回。
