# m4-segment-code-r1 正推腿报告（Sonnet，2026-10-16）

立场：正推。核「代码做的是不是判决与用户定案说的那件事」，逐格对快照轮 S1–S9、GC 轮 G1–G9 的判决原文与第七节用户定案；零轮读法逐条判与判决原文相容不相容。
冻结副本：`~/.local/share/singlefs/work/m4-segment-code-r1/repo/`。下文代码行号都是它里面的行号（`crates/singlefs-core/src/…`）；kb 行号是 kb 文件自己的行号（工作树 `.claude/kb/decisions/…`）。
探针放在 `research/prompts/m4-segment-code-r1-sonnet-model/`，在草稿目录的拷贝上编、跑（拷贝与冻结副本 sha256 同；每条命令经 `capped.sh 31` 与 `run-with-memory-cap.sh 8G`）。

## 0　开工核对（冻结副本的 sha256）

命令（在冻结副本根上）：`sha256sum -c ~/.local/share/singlefs/work/m4-predev/research/prompts/m4-segment-code-r1-snapshot/crates-sha256.txt`，原样输出：

```
crates/mutations.tsv: OK
crates/singlefs-core/src/admission.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/block_device.rs: OK
crates/singlefs-core/src/code_two_tree.rs: OK
crates/singlefs-core/src/compaction_plan.rs: OK
crates/singlefs-core/src/lib.rs: OK
crates/singlefs-core/src/logged_operations.rs: OK
crates/singlefs-core/src/mounted_read.rs: OK
crates/singlefs-core/src/mounted_session.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/records.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/region_scan.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/unlink.rs: OK
crates/singlefs-core/src/write_accounting.rs: OK
crates/singlefs-format/src/lib.rs: OK
crates/singlefs-reference-filesystem/src/lib.rs: OK
crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs: OK
crates/singlefs-replay-prefix/src/lib.rs: OK
```

（21 行，全 OK。）

## 读法说明

- 「一致 / 不一致 / 规则没说」三种判定；不一致的两句并排抄原文。
- 每条结论后给「什么现象会推翻它」。
- 判据只取两份判决的第三节、第七节、第 2 批代码轮判决第二节与用户在调度记录第三节定的读法；kb 条款引 kb 文件自己的行号。

## 探针与复跑

- 探针源与复跑脚本：`research/prompts/m4-segment-code-r1-sonnet-model/`（`probes/` 里九个 `probe_forward_*.rs` 与两个接在原用例后面的 `append_to_*.rs`、`tools/rerun.sh`、`out/expected.txt`）。步名 s00a–s15b 在下面各格里点名。
- 冻结副本的拷贝在草稿目录（`repo-copy/`，编译目录 `target/`），只在拷贝上加探针、做变异；每条编译与测试命令都经 `scripts/hosts/capped.sh 31` 与 `run-with-memory-cap.sh 8G`，同一时刻只起一条 cargo。
- 本机此刻负载很高（loadavg 数十到一百多），冷编译与带 4 GiB 稀疏盘的用例慢；整份 `rerun.sh` 没有在前台一次跑完（见末尾「要主 agent 起的长活」）。

## T1　建快照那次发布

结论：与快照轮 S1、S2 和用户定案「建快照算非空」一致；零轮读法「事务号取上一版 + 1」相容；一处规则没说：文件集合为空的一版上建快照被拒。

1. 独立发布、不带文件、新树号从水位连发两个、第 11 个写前拒：`crates/singlefs-core/src/transaction.rs:7269`：「file: None,」；`crates/singlefs-core/src/transaction.rs:7244`：「let watermark_after_this_publish = watermark_before_this_publish.checked_add(2).ok_or(」；`crates/singlefs-core/src/transaction.rs:7236`：「if snapshots_before_this_publish >= SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {」。会话入口 `crates/singlefs-core/src/mounted_session.rs:1125`：「publish_create_snapshot(&mut writer, allocator, previous, instance)」，先走空队列再发（`crates/singlefs-core/src/mounted_session.rs:1359` 起）。
2. S.txg 同一个值写进活头 inode 条目的前驱与码 14 的诞生 txg：`crates/singlefs-core/src/transaction.rs:10824`：「SnapshotStepOfThePublish::CreateSnapshot { .. } => txg,」；码 14 的前驱是捕获那一刻活头的值 `crates/singlefs-core/src/transaction.rs:11040`：「previous_snapshot_txg_of_the_live_inode_tree_before_this_publish,」；码 15 的头 ID 是配对码 14 的树号 `crates/singlefs-core/src/transaction.rs:11050`：「head_identifier: snapshot_inode_root_tree.0,」。对 S1 判决原文 `research/prompts/m4-snapshot-r1-main-verification.md:30` 的字段逐个对上（根指针抄活头两个根、头 ID、前驱 0、flags 位 0 留给销毁中 `crates/singlefs-core/src/records.rs:448`：「pub const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u16 = 1;」）。
3. 之后每一版原样续上：`crates/singlefs-core/src/transaction.rs:10964`：「let snapshot_entries_carried_from_the_previous_version: Vec<TreeTableEntry> = previous」。
4. S1 配对判定（第 2 批代码轮 T1 打中后补的两条用例与变异）还在：`crates/singlefs-core/src/recovery.rs:2428`：「if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {」；`crates/singlefs-core/src/recovery.rs:2440`：「inode_root_entry.tree.0 == extent_root_entry.head_identifier」；变异行 `crates/mutations.tsv:4179`、`:4180` 在表里。
5. 「非空」：建快照那次的快照根指针进字节比较 `crates/singlefs-core/src/mount.rs:3414`：「|| root_pointers.snapshot_tree_roots != previous_valid_root_pointers.snapshot_tree_roots」，对 `.claude/kb/decisions/16-发布语义.md:46` 的「建快照那次发布算非空」。
6. 零轮读法「建快照那次事务号取上一版 + 1」：`crates/singlefs-core/src/transaction.rs:7252`：「let transaction_of_the_snapshot = previous.highest_transaction_number_in_this_instance + 1;」。覆盖写、整理一批、删除类三个入口写的是同一个式子（`:7053`、`:7546`、`:7940`），所以「撞号」只可能出现在两个入口在同一个上一版上各发一次；会话串行发布，直接调发布函数的调用方才造得出。回退那次事务号写 0（`crates/singlefs-core/src/mount.rs:8237` 的文档），不与它们共号。相容。
7. 规则没说：现行那一版有树而文件集合为空时会话拒建快照 `crates/singlefs-core/src/mounted_session.rs:1390`：「| UserChange::CreateSnapshot => {」（拒成 `NoFileVersionToChange`）。S2 判决原文没写这一格；RefFS 同取拒（`crates/singlefs-reference-filesystem/src/lib.rs:441`：「CreatingASnapshotWithoutAFileIsNotInTheFirstVersion,」）。删快照在同一格放行（T3）。

探针：
- s09（`probe_forward_snapshot_cells.rs`）：建快照之后建 inode、unlink 意图与各批、再 unlink 两个新 inode，每个发布出来的版本 `pairs_equal=true`、活头前驱不变；删空文件集合之后 `creation_on_an_emptied_file_set result=Err(NoFileVersionToChange)`。原样输出：

```
name=probe_forward_carry kind=new_inodes txg=7 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_intent_and_batches txg=8 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_intent_and_batches txg=9 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_of_a_new_inode txg=10 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_of_a_new_inode txg=11 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_of_a_new_inode txg=12 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_carry kind=unlink_of_a_new_inode txg=13 pairs_equal=true live_head_previous_snapshot_txg_equal=true
name=probe_forward_creation_on_an_emptied_file_set result=Err(NoFileVersionToChange)
```
- s00b：已入库的 `snapshot_creation_publish_and_the_entries_every_later_publish_carries` 在拷贝上 `test result: ok. 5 passed; 0 failed`（覆盖写、挂载写行与暖机、十个快照、第 11 个拒）。
- 没走到：推环的空发布（`push_the_root_ring…` 报 `refused=NothingHeldUntilThisProcessWritesARingSlot`，该池上没有宽护要解）；抬 F 的空发布这一类靠 s00b 里挂载写行与暖机那条用例代。

推翻条件：任一类发布（抬 F 的空发布、整理一批）发出来的版本 `snapshot_entries()` 与建快照那一版不等，或活头前驱变了；或同一个实例里两个发布入口在同一个上一版上发出同一个事务号。

## T2　释放分流与 deadlist

结论：与 S3 一致，一处缺：判决写的「只供测试的开关：强制立即释放 / 进 deadlist」代码里没有。零轮读法（deadlist 节点空间按普通分配；下一侧是活头时 hi 取这一批 txg）相容。

1. 切口：`crates/singlefs-core/src/transaction.rs:3511`：「pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(」，在 `placements_released_by_this_publish`（`:8673`）出清单时把进 deadlist 的几个拿掉，`publish_admitted` 落释放（`:10136`）读的是拿掉之后的清单。对 `research/prompts/m4-snapshot-r1-main-verification.md:32` 的「切口在 `placements_released_by_this_publish` 出清单之后、`publish_admitted` 落释放之前」。
2. 只有六个用户可见角色问寿命判定：`crates/singlefs-core/src/transaction.rs:4958`：「pub const fn asks_the_snapshot_lifetime_judgement(identity: TransactionUnit) -> bool {」，别的角色（含 deadlist 树自己的两种角色）一律 `false`。
3. 判定：`crates/singlefs-core/src/transaction.rs:5006`：「Some(birth) if birth > previous_snapshot_txg_of_the_live_head => {」；`birth == S.txg` 那一格归 deadlist（钉住），取不到 birth 也进 deadlist。对 `.claude/kb/decisions/05-快照-空间记账机制.md:422`：「elif birth > p: 立即释放」。实际上 `birth == S.txg` 的用户可见单元不会出现：建快照那次发布不写用户可见单元（T1 第 1 条 `file: None`）。
4. deadlist 条目：纪元取活头此刻的前驱 `crates/singlefs-core/src/transaction.rs:4863`：「epoch: previous_snapshot_txg_of_the_live_head,」；key 43 = 8 + 8 + 27 `crates/singlefs-core/src/code_two_tree.rs:44`：「pub const DEADLIST: Self = Self(&[8, 8, 1, 8, 8, 4, 6]);」，叶 376 `crates/singlefs-core/src/code_two_tree.rs:1255`：「leaf_entries: 376,」（内部 125 在下一行），种类 8 `crates/singlefs-core/src/records.rs:424`：「pub const TREE_KIND_DEADLIST: u16 = 8;」。映射条目留存（α）：`crates/singlefs-core/src/transaction.rs:4819`：「fn deadlist_changes_of_this_publish(」把被钉单元的两条位置项放进 `kept_mapping_entries_after_this_publish`。
5. 缺的那一项：判决 S3 末句要一个强制开关（`research/prompts/m4-snapshot-r1-main-verification.md:32`：「只供测试的开关：强制…立即释放 / 进 deadlist」）。`grep -rn 'SnapshotLifetime' crates/singlefs-core/src` 只命中判定函数与它的单测（`crates/singlefs-core/src/transaction.rs:5001`：「pub fn snapshot_lifetime_verdict(」的调用点与 `:11877` 起的单测），没有开关；再查一遍：在冻结副本的 `crates/` 下跑 `grep -rniE 'force[a-z_]*(deadlist|release)|SnapshotLifetimeSwitch|lifetime_switch' --include=*.rs .`，只命中整理的「强制释放前核出对不上」开关与一条无关的 FUA 用例名，没有 deadlist / 寿命判定的强制入口。两个分支在自然历史里都走得到（用例钉了，s00b 与 `snapshot_release_and_destroy_through_the_deadlist`），缺的是 `.claude/rules/fs-design.md` 五条硬要求第 2 条要的强制入口。
6. 零轮「deadlist 节点空间按普通分配」：`space_budget_of_role` 把 deadlist 两种角色归 `OrdinaryAllocation`（`crates/singlefs-core/src/admission.rs:1502`：「TransactionUnit::DeadlistTreeNodeBelowTheRoot(_) | TransactionUnit::DeadlistTree => {」），换下它们时 `asks_the_snapshot_lifetime_judgement` 为 `false`，直接进 defer：deadlist 节点不会被判成「快照留着」，用户可见单元也不会被当成 deadlist 节点。相容。
7. 两个快照之间出生的单元在中间那个快照删掉时：下一侧按（诞生 txg，树号）排 `crates/singlefs-core/src/transaction.rs:4735`：「chain.sort_by_key(|entry| (entry.birth_txg, entry.tree));」；放的是纪元落在 [lo, hi) 且出生 > P 的 `crates/singlefs-core/src/transaction.rs:4783`：「&& (lo..hi).contains(&entry.epoch)」与 `:4785`：「.is_some_and(|birth| birth > predecessor)」；下一侧是活头时 hi 取这一批 txg `crates/singlefs-core/src/transaction.rs:4775`：「Some(NextSideOfASnapshot::LiveHead) | None => hi_when_the_next_side_is_the_live_head,」。我手推了三种删序（只有一个快照；先删新的；先删旧的），放的与留的与 `snapshot_release_and_destroy_through_the_deadlist` 里钉的相同。
8. 零轮「hi 取这一批 txg」：意图到最后一批之间会话不会插进用户改动（先走空，T3 第 3 条），所以纪元 ≥ 写意图 txg 的条目一条都没有，两种取法放同一组；直接调发布函数的调用方才能在意图之后覆盖写，产品路径不走。相容。
9. kb 内部的一处张力（不是代码的错，写回时核）：`.claude/kb/decisions/05-快照-空间记账机制.md:277`：「| 哪一份 | 快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg) 的条目」与 `.claude/kb/decisions/05-快照-空间记账机制.md:278` 的「销毁 S」行（lo = S.txg）区间写法不同；判决 S4 和代码取后者。

探针：s00b；s11 的第一个用例（`probe_forward_destroying_a_snapshot_after_its_pinned_unit_was_relocated_releases_the_destination`）走过「被钉单元进 deadlist、留存映射条目、删快照放掉」整条：见 T5。

推翻条件：造出一个 `birth` 恰等于某快照 S.txg 的用户可见单元而它被立即释放；或六角色之外的角色（deadlist 节点）被判进了 deadlist。

## T3　删快照意图、批与续做

结论：与 S4 和岔路 2、4 一致；零轮读法（批大小与 unlink 共用开关；有树而文件集合为空的一版上删快照放行）相容。

1. 意图只占码 14 条目 flags 位 0 `crates/singlefs-core/src/records.rs:448`：「pub const TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED: u16 = 1;」；写意图那次把被删快照的前驱传给那一刻的下一侧，下一侧是活头就换活头 inode 条目的前驱 `crates/singlefs-core/src/transaction.rs:10830`：「Some((NextSideOfASnapshot::LiveHead, predecessor)) => predecessor,」，是下一条码 14 就换它的前驱（`:10984` 起的 map）。
2. 一批放多少条：`crates/singlefs-core/src/unlink.rs:112`：「pub fn deadlist_entries_per_snapshot_destruction_batch(batch_size: UnlinkBatchSize) -> usize {」取 unlink 同一个开关，产品 144（`crates/singlefs-format/src/lib.rs:179`：「pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;」）。最后一批：`crates/singlefs-core/src/transaction.rs:7363`：「let step = if releasable <= entries_per_batch {」，同一次发布删掉那一对码 14、15（`:10955` 起把被删那对从续上的条目里滤掉）。
3. 次序：会话任何一次改动先把队列走空，销毁中的快照与 `deleted_inodes` 一起走（`crates/singlefs-core/src/mounted_session.rs:1359` 起）；走空时 unlink 先、快照后 `crates/singlefs-core/src/transaction.rs:7906`：「(None, Some(snapshot)) => publish_snapshot_destruction_step(」；整理先走空 `crates/singlefs-core/src/mounted_session.rs:1937` 起；挂载续做是删除队列、抬 F、再整理（`crates/singlefs-core/src/mount.rs:6443`：「CompactionRunReach::ContinueTheLiveIntentAtTheMount,」之前先 `continue_the_deletion_queue_until_it_is_empty`）。所以删快照、unlink、整理一批在同一次挂载里不会交错成「半个」：谁先谁后由队列定，unlink 先于快照，两者都先于整理。
4. 有树而文件集合为空的一版上删快照放行：`crates/singlefs-core/src/mounted_session.rs:1384`：「UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {」。
5. 销毁中的快照被当成什么：建新快照——拒，在数上限之前 `crates/singlefs-core/src/transaction.rs:7231`：「if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {」，所以「上限 10 数不数它」不会出现；读快照——照读，读出建它那一刻的内容；回退——拒（T4）。
6. 崩在两批之间：可写挂载续做到空。

探针：s09 第二个用例（`probe_forward_a_snapshot_being_destroyed_…`），原样输出：

```
name=probe_forward_creation_while_destroying result=Err("SnapshotCreationRefused(SnapshotBeingDestroyed { being_destroyed: TreeIdentifier(65549) })")
name=probe_forward_read_while_destroying result=File(3000 bytes, equals_the_content_at_creation=true)
name=probe_forward_rollback_while_destroying result=Err("SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses { version: Current, snapshots: 1 }")
name=probe_forward_user_visible_tree_root_pointers snapshot_tree_roots=Ok(2)
name=probe_forward_mount_after_the_intent snapshot_entries_left=0 snapshots_being_destroyed_left=0 deadlist_entries_left=0 kept_mapping_entries_left=0
name=probe_forward_destruction_on_an_emptied_file_set versions_published=2 snapshot_entries_left=0 live_head_previous_snapshot_txg_after=0
```
最后一行是文件集合为空的一版上删快照：发了意图加最后一批共 2 版，活头前驱传回 0。

推翻条件：销毁中的快照还能建新快照；崩在两批之间的镜像可写挂载之后快照条目、deadlist 条目或留存映射条目有剩；一批放的不是按 key 升序的前几条（`snapshot_release_and_destroy_through_the_deadlist` 里钉了）。

## T4　回退拒与读快照

结论：与 S8 一致（只数码 14，在有效镜像上等价于「码 14 / 15」）；读快照走提示再走映射；第 2 批代码轮 T2 打中的读者今天对码 14 / 15、码 8、码 12 都放行。

1. 回退拒：现行那一版在任何读之前判 `crates/singlefs-core/src/mount.rs:8259`：「let snapshots_in_the_current_version = snapshots_in_the_tree_table(current);」，目标那一版重建出来之后判 `crates/singlefs-core/src/mount.rs:8338`：「let snapshots_in_the_target_version = snapshots_in_the_tree_table(&target_version);」；只数码 14 `crates/singlefs-core/src/mount.rs:7649`：「fn snapshots_in_the_tree_table(version: &TransactionOutput) -> usize {」。有效镜像上码 14 与码 15 条数相等是读树表时判过的（T1 第 4 条），所以只数一种不漏。销毁中的算（flags 不看）。
2. 读快照：`crates/singlefs-core/src/recovery.rs:5931`：「pub fn read_the_first_file_of_a_snapshot(」从码 14 / 15 的两条根指针往下读，节点按根指针头部的出生树核，位置提示读不出再查**这条根**的映射（留存的映射条目在里面）。整理把被钉单元搬走之后读到目的地：已入库用例 `a_unit_kept_only_for_a_snapshot_is_relocated_and_the_snapshot_still_reads_it`（s00a 里 8 条全绿）把源落点清零后读，仍读出建快照那一刻的内容。
3. 读树表的读者：`key_width_for_kind` 对码 14 / 15 交 `None` `crates/singlefs-core/src/recovery.rs:4197`：「TREE_KIND_SNAPSHOT_INODE_ROOT | TREE_KIND_SNAPSHOT_EXTENT_ROOT => None,」，调用方在走到它之前先 `continue`（冷走读 `:5619` 起，`user_visible_tree_root_pointers` 把它们记进 `snapshot_tree_roots`）。
4. 探针 s10（一张同时带码 14 / 15、非零 deadlist 根、非零 logged_ops 根的树表）：

```
name=probe_forward_readers_table kinds_present: snapshot_inode=true snapshot_extent=true deadlist_root_nonzero=true logged_ops_root_nonzero=true
name=probe_forward_readers walk_to_file=Ok(Some(2000))
name=probe_forward_readers user_visible_tree_root_pointers=Ok(2)
name=probe_forward_readers rebuild_version=Ok("Ok")
name=probe_forward_readers rebuild_version_with_its_snapshot_entries=Ok(1)
name=probe_forward_readers allocation_records_under_root=Ok(88)
name=probe_forward_readers open_pool_for_read=Ok("Ok")
name=probe_forward_readers mount_read_only=Ok("Ok")
```

推翻条件：任一读者对上面那张树表报「树的种类没登记」；或带销毁中快照的版本回退被收下。

## T5　一批搬迁的事务形态（G3）

结论：与 G3 取乙一致；被快照钉住的单元搬走后快照仍读得到（含「再删快照」）；调查员修法把装着搬走 key 的叶连同到根整条标重写，多叶、单叶两种形状下重写集合对；多出来的重写**不过准入**，在量到的取样点上没有越过 ckpt_cost，但没造出越过的点。

1. 一批 = 一个发布入口、一个事务：`crates/singlefs-core/src/transaction.rs:7447`：「pub fn publish_relocation_batch<Device: BlockDevice>(」交给 `publish_compaction_step`（`:7494`）；目的落点由这次发布的分配器现取 `crates/singlefs-core/src/transaction.rs:10165`：「let destination = allocate_relocation_destination(allocator, relocation)?;」，分配代写诞生代（`:8626` 起，`try_allocate_user_data(relocation.birth_txg)`）；映射位置换到目的地 `crates/singlefs-core/src/transaction.rs:10224`：「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」；源落点进同一张释放清单。崩在这次发布前后：根没落盘读旧映射与未释放的源，落盘之后读映射的目的地，没有中间态；「源落点先于映射释放」的必红世界在 `a_batch_that_releases_the_source_while_the_mapping_still_names_it_is_judged_red` 里钉住（s00a）。崩溃点枚举归 checker 档，我没跑。
2. 调查员修法：`crates/singlefs-core/src/transaction.rs:6855`：「fn plan_rewriting_the_paths_to_the_moved_entries(」，对每把搬走的 key 从根往下每个节点标 `plan.origins[index] = CodeTwoTreeNodeOrigin::RewrittenThisPublish;`（`:6873`），再重算上一版被换下的节点。已入库用例 `consecutive_batches_rewrite_the_mapping_leaf_holding_each_moved_entry_and_a_writable_mount_keeps_both` 钉了两叶那一形。
3. 探针 s11 第二个用例：映射树压成叶 3 条、内部 3 个孩子（只供测试的容量，5 片叶），一批搬四个落在不同叶里的单元（数据单元、extent 根、inode 叶容器、inode 根）：

```
name=probe_forward_batch_vs_ckpt_cost mapping_leaves_before=5 mapping_leaves_after=5 units_moved=4 ckpt_cost_of_the_version_before_the_batch=1083 reserve_class_units_written_by_the_batch=15 of_which_mapping_nodes=8 ordinary_allocation_units_written=0
name=probe_forward_empty_publish_for_comparison reserve_class_units_written=14 of_which_mapping_nodes=7
```
这一批写了 15 个保留池一类的固定点单元（8 个是映射节点），同一版上的一次空发布写 14 个（7 个映射节点）：多出 1 个映射节点；该版的 ckpt_cost 是 1083，远不到。
4. 为什么不过准入：普通批没有 `OrdinaryAllocation` 角色，`crates/singlefs-core/src/admission.rs:1526`：「pub fn publish_has_an_ordinary_allocation(rewritten_roles: &[TransactionUnit]) -> bool {」为假，式子那一判与落点先判都不判（`crates/singlefs-core/src/transaction.rs:8453`：「let demand_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {」）；只有签发与删意图那两次发布（重写 logged_ops 根，它是普通分配）才进式子。这一批多写的节点只受「落点取得到」约束。所以「罩不罩得住」问的 ckpt_cost 其实不是它的闸：产品容量下一批多写的映射节点 ≤ 搬的 key 数（至多一段 64 槽里的单元数）× 树高，而 ckpt_cost 的分配记录树那一项取整棵树（`crates/singlefs-core/src/admission.rs:1091`：「代价随单元区线性涨（两块 4 GiB 盘 529 个节点）。」），数量级上罩得住；这是推的，我没造越过的取样点。
5. 文档与行为不符：`crates/singlefs-core/src/transaction.rs:7440`：「/// 照删除的三种发布同一张计划：不写文件内容、不建 inode、实例表照抄、不发树号；准入同删除类（式子那一判照判，合取 7 免）。」而删除类的式子那一判是不判的（`:8453` 起），普通批同样不判。注释该改，或行为该改——判决 G4 把整理批排在合取 7 之外、没写式子那一判。
6. 被快照钉住的单元搬走再删快照（s11 第一个用例）：

```
name=probe_forward_after_relocation violated=[] holds=78 not_applicable=21 partially=0
name=probe_forward_after_destroy destination_slot=50184 source_slot=50180 destination_records_released=2 destination_records_unreleased=0 source_records_released=2 released_placements=[50282, 50283, 50284, 50285, 50286, 50287, 50271, 50288, 50289, 50184, 50240, 50244, 50242]
name=probe_forward_after_destroy_checker violated=[] holds=78 not_applicable=21 partially=0
```
删快照放的是目的地（50184 在放掉的清单里）、源落点（50180）没有被放第二次；池级 checker 搬完和删完两个镜像上都没有违例。

推翻条件：搬走被钉单元之后读快照读到旧落点的字节或读不出；删快照放了源落点；批写出的保留池一类单元多于该版 ckpt_cost（多叶、大盘上的取样点）。

## T6　整理意图上盘与挂载续做

结论：G1 字段表与 G8 的整理发布写什么删什么一致；**`segs0` 与判决 G1 的定义句不一致**（差在签发那次发布自己开新段时，探针坐实，差一）；零轮读法 `B_gc` 数每一次发布、丢意图也发一次发布相容；`segs0` 沿用记录口径相容但另有差一；旧镜像、新镜像三条路和坏条目的结局见下。

1. 字段表：`crates/singlefs-core/src/logged_operations.rs:197`：「条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | c0 22 | segs0 30 | 补零 38，小端。」与判决 `research/prompts/m4-gc-r1-main-verification.md:32` 的「key 10 = 种类 2 + 签发 txg 8」「value 34 定宽」逐段相同；读者拒保留种类码、起始槽不对齐、槽数不是 64、补零非零、不升序、多于一条整理意图。探针 s14 原样输出：

```
name=probe_forward_logged_ops_decode entry_bytes=44
name=probe_forward_logged_ops_decode one=Ok(1)
name=probe_forward_logged_ops_decode padding_not_zero=Err(EntryRefused(CompactionPaddingNotZero))
name=probe_forward_logged_ops_decode reserved_kind=Err(EntryRefused(UnknownKind { kind_code: 2 }))
name=probe_forward_logged_ops_decode slot_count_not_64=Err(EntryRefused(CompactionRegionSlotCountIsNotOneClusterSegment { slot_count: 32 }))
name=probe_forward_logged_ops_decode two_ascending=Err(MoreThanOneCompactionIntent { compaction_intents: 2 })
name=probe_forward_logged_ops_decode two_descending=Err(EntriesNotInStrictlyAscendingKeyOrder)
```
2. 签发、每一批、达成、丢意图各发一次发布：签发 `crates/singlefs-core/src/mounted_session.rs:2366`：「CompactionIntentStep::Issue(issued),」；丢意图 `crates/singlefs-core/src/mounted_session.rs:2426`：「CompactionIntentStep::Drop,」（隔离槽 / 扫描被拒 / 一个能搬的都没有三种）；最后一批同发布删 `:2433`：「CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,」。每一次发布都用掉一个根槽 `crates/singlefs-core/src/mounted_session.rs:2006`：「publishes_of_the_run += 1;」，对 `research/prompts/m4-gc-r1-main-verification.md:39` 的「一次运行的发布预算 B_gc 现算 = 3S − 4 − 已经过去的发布数、至少 1」，预算式在 `crates/singlefs-core/src/compaction_plan.rs:760`：「.saturating_sub(4)」。零轮「`B_gc` 数每一次发布」「丢意图也发一次发布」：相容。「已经过去的发布数」代码只数整理自己的发布、用户改动成功后清零（`crates/singlefs-core/src/mounted_session.rs:1256`：「compaction.publishes_since_the_last_user_change = 0;」），挂载自己的写行与暖机不数——判决 G8 没写它数不数，规则没说。
3. 挂载恢复：重建时读回活意图、在写行之前装上 R 排他 `crates/singlefs-core/src/mount.rs:7512`：「allocator.compaction_mut().issue(intent);」。
4. **`segs0` 不一致**：判决 G1 `.claude/kb/decisions/26-后台整理与放置回收.md:63`：「| value | `segs0` | 8 | 30 | 签发那次发布写下的记账第 11 项（全空聚簇段数，D5（快照 / 空间记账机制） 已定项 4）里 `R` 所在盘的值，只作观测量 |」，`crates/singlefs-core/src/logged_operations.rs:128` 同句。代码写进意图的是签发之前那一版的分配记录按记录口径数出来的首盘全空段数：`crates/singlefs-core/src/mounted_session.rs:2353`：「empty_cluster_segments_at_issue: empty_cluster_segments_by_the_records(」。签发那次发布自己开了一个新聚簇段时，它写下的第 11 项比这个数少一。探针 s12（把开放段填到只剩两槽，让签发那次发布开新段）：

```
name=probe_forward_segs0 case=issue_inside_the_open_segment segs0_written_in_the_intent=3310 accounting_row_11_written_by_the_issue_publish=3310 issue_publish_opened_a_new_segment=false
name=probe_forward_segs0 case=issue_publish_opens_a_new_segment segs0_written_in_the_intent=3310 accounting_row_11_written_by_the_issue_publish=3309 issue_publish_opened_a_new_segment=true
```
会让哪一个判定翻面：「一条意图完成而全局差值为假就不签下一条」`crates/singlefs-core/src/compaction_plan.rs:736`：「empty_cluster_segments_now > empty_cluster_segments_at_issue」。意图开了一段新段又腾空了 R，净增为 0 时，按判决定义（第 11 项）3310 − 3309 = 1 为真，继续签；代码拿 3310 比，为假，这一趟因无产出而停（`crates/singlefs-core/src/mounted_session.rs:2253`：「if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {」）。`last_completed_intent` 只活在这一趟里，下一趟照常重签，所以是少跑一趟、不是卡死。零轮「`segs0` 沿用记录口径」：口径相容（C372 之后第 11 项本身就是记录口径，s11 里搬完一批三个数相等：`allocator_incremental=3310 full_scan_of_the_records=3310 accounting_row_11=3310`），差一另算。
5. 带活意图的新镜像（探针 s13 的第一个用例，R 取第一个文件那次发布的那一段，原样输出）：

```
name=probe_forward_read_only_mount_with_a_live_intent result=Ok
name=probe_forward_product_mount_with_a_live_intent continuation=Some("Stopped(BatchSizeNotDecidedByTheClauses)")
name=probe_forward_product_mount_with_a_live_intent region_excluded=Some(CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50240) }) intent_region=CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50240) }
name=probe_forward_rollback_with_a_live_intent target=RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) } result=Ok intent_still_in_the_new_version=true region_still_excluded=Some(CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50240) })
```
只读挂载收下；产品开关下可写挂载在任何写之前停在批大小条款没写这一条（`crates/singlefs-core/src/mounted_session.rs:1926`：「CompactionBatchSize::NotDecidedByTheClauses => {」），意图留在盘上、R 排他恢复，之后每次挂载都这样——产品路径上盘上那条意图永远推不完（只有测试开关签过它）；回退收下，logged_ops 随现行那一版（`crates/singlefs-core/src/mount.rs:8201`：「logged_operations: current.logged_operations.clone(),」），意图和 R 排他跨回退留着。logged_ops 根恒为零的旧镜像：`crates/singlefs-core/src/recovery.rs:2971`：「if logged_operations_root_pointer == NodePointer::empty_root() {」直接得空表，三条路都等于没有意图。
6. 坏条目（T10 第 3 条）：可写挂载整个拒、只读与冷走读收下。
7. 判决 G4 的三条水位（V-a）没有实现：`crates/singlefs-core/src/compaction_plan.rs:257`：「第一版三条水位恒 0、0 = 关（判决 G4 V-a；内置默认改非零不在这一批）」，所以 G8 丁（用户改动之后越线再跑一批）在产品路径上不起跑（`crates/singlefs-core/src/compaction_plan.rs:767`：「pub const fn compaction_starts_after_a_user_change(forcing: CompactionRunForcing) -> bool {」）。是自报的缺，不是漏报。

推翻条件：`segs0` 与第 11 项在签发开新段时仍相等；或签发那次发布开新段而 `NoYieldFromTheLastIntent` 没有提前停。

## T7　R 排他、签发与停机、合取 7

结论：R 排他在分配三路、抬 F、回退、unlink 释放上没有漏口（读码，加探针没发现）；K-c 一致；`empty_segments` 五处增量维护与全扫重算逐步相等；合取 7 的射程与「被拒不推抬 F」与用户定案一致，**后果**：产品路径上整理起不来，一块盘一旦没有全空聚簇段，前台写入类发布全拒、只有删除类能解开。

1. R 排他：分配三路都有「在 R 外」合取：用户数据 `crates/singlefs-core/src/allocator.rs:908`：「pub fn lowest_user_data_slot_outside_the_region(」，开新段 `:942`：「pub fn lowest_empty_segment_outside_the_region(」，提交内生块回落 `:978`：「pub fn lowest_commit_generated_fallback_slot_outside_the_region(」，两个分配入口都在查询前取排他区（`:2209`、`:2272`：「let excluded_region = self.compaction.region_excluded_from_placement();」）。开放段 bump 不会落进 R：选段排开这次挂载开过的段（`crates/singlefs-core/src/compaction_plan.rs:161` 一带的 `opened_in_this_mount`），强制选段同样判（`crates/singlefs-core/src/mounted_session.rs:2270` 起）。抬 F 的空发布、回退发布的固定点单元与整理批的目的地都走这两个入口；unlink 释放只释放不分配；回退复活不分配，复活进 R 的记录由每批之前对 R 重扫接住（判决 G1 F4）。挂载续做在写行之前就装上排他（T6 第 3 条）。
2. K-c：R 里有隔离槽（常驻隔离集）就发一次只删意图的发布 `crates/singlefs-core/src/mounted_session.rs:2397`：「let scan = if region_holds_a_quarantined_slot(region, &quarantined_slots) {」；一个槽既钉住又在隔离集里时选段按钉住排除（`slots_that_disqualify_a_segment` 是两者的并），续做按隔离丢意图：已入库用例 `a_slot_both_pinned_and_quarantined_is_scanned_as_pinned_and_still_drops_the_intent` 钉了。常驻隔离集只活一次挂载（下一次挂载同一段会被重新选、重新签、再丢），是少跑几趟的代价，规则没说跨挂载记不记。
3. `empty_segments` 增量维护：`crates/singlefs-core/src/allocator.rs:850`：「pub fn empty_segments(&self) -> u64 {」；分配、释放、复活三处调同一个加减函数 `:580`：「fn note_slots_under_an_unreleased_record(」，回收不动。探针 s01（两盘各 1 GiB，随机走分配、释放、回收、回退复活，每步两盘各与「全扫记录」比）：

```
name=probe_forward_empty_segments checked_comparisons=32000 rebuilds_checked=64 max_segments_empty_by_records_but_occupied_by_deferred_slots=2
```
32000 次比较全等，64 次挂载重建（`rebuild_from_records`）后同样全等；记录口径与占用口径在 defer 槽上最多差过 2 段，说明序列真的走到了两口径分开的状态。整理搬迁（分配目的地、释放源）在 s11 里搬完一批：`allocator_incremental=3310 full_scan_of_the_records=3310 accounting_row_11=3310`。判别力：往拷贝里塞变异 M1（复活不加回计数），探针红（s02，`test result: FAILED. 0 passed; 1 failed`）。
4. 合取 7 管哪些发布：`crates/singlefs-core/src/transaction.rs:8314`：「fn reach_of_the_empty_cluster_segments_conjunct(」前台写入类 = 写文件内容、建 inode、建快照，回退、删除类、整理批免；建快照没有普通分配角色，实际不判。N₀ 第一版取 1 `crates/singlefs-core/src/compaction_plan.rs:302`：「pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;」，对 `.claude/kb/decisions/03-空间分配.md:146` 的「`N₀` 第一版取 1」。被拒不推抬 F：`crates/singlefs-core/src/mounted_session.rs:1026`：「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」在 `refusal_is_short_of_space` 里归「不是空间不够」，已入库用例 `a_foreground_write_refused_by_the_empty_cluster_segments_floor_in_a_session_pushes_no_floor_raise` 与 `an_unlink_intent_is_exempt_from_the_empty_cluster_segments_floor` 在拷贝上绿（s00c，17 条）。
5. 「被拒不推抬 F 会不会让一块盘永远凑不够 N₀ 而整理又起不来」：会，在产品路径上。整理起不来：`crates/singlefs-core/src/mounted_session.rs:1286`：「pub fn run_compaction<Device: BlockDevice>(」的显式入口先看批大小，产品值是 `NotDecidedByTheClauses`，一个意图都不签（s13 同一现象）；水位触发关着（T6 第 7 条）。所以一块盘上若每个聚簇段都有未释放记录，前台写入类发布全拒成 `EmptyClusterSegmentsBelowTheFloor`，抬 F 不推、整理不起；出路只有删除类：unlink 把槽放进 defer，按记录口径某一段一旦整段没有未释放记录就立刻算全空，合取 7 当场放开（删掉的东西没占满一整段时放不开）。这与用户 2026-10-13 的定案相容——调度记录 `records/2026-10-13-里程碑四预开发.md:163`：「弹窗问用户合取 7 现在接还是等整理能自己跑」，用户选现在就接，后果如上。我没造 4 GiB 盘上凑出零个全空段的取样点（几千个聚簇段），靠开关抬 N₀ 的既有用例代；这一格是推的。
6. `run_compaction` 一次至多签几条、发布后至多一批：一趟的发布预算 `crates/singlefs-core/src/mounted_session.rs:1978`：「if publishes_of_the_run >= budget {」，用户改动之后那一批 `crates/singlefs-core/src/mounted_session.rs:1986`：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」；一次调用可以签几条意图，受预算和「无产出就停」约束（判决 G8 甲）。一致。

推翻条件：造出一个落点落进活 R 的发布；`empty_segments` 与全扫记录不等的序列（探针没找到）；产品路径上整理自己起来了。

## T8　discard 调用者（G5）

结论：与 G5 一致；两处规则没说、代码取了一种读法：discard 交 Unsupported / 报错 / 孤儿头被扣下之后该段不补发。mutations.tsv 第 4453 行那条超时的变异，有不超时的 harness 层用例杀它。

1. 时机与粒度：发布都返回之后、按 64 槽对齐的整段发。`crates/singlefs-core/src/mounted_session.rs:1268`：「published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(」在 `publish_user_change` 的最后（整理批、抬 F 的发布都在它前面返回了）；显式入口同样在整趟之后（`:1320`）。发 discard 与下一次分配之间没有别的调用：同一个调用栈里依次执行，谓词不会在中间变假。谓词：`crates/singlefs-core/src/allocator.rs:809`：「pub fn segment_is_reusable_as_a_whole(&self, segment_index: usize) -> bool {」要求没有分配位置着的槽（defer 窗口里的也算占着）、没有隔离、没有抬 F 扣住（逐项对判决 G5「已回收、没被抬 F 扣住、没被隔离、F 已生效」）。
2. 待发集只在回收那一刻记：`crates/singlefs-core/src/allocator.rs:791`：「self.segments_reclaimed_not_yet_discarded.insert(segment);」；`crates/singlefs-core/src/allocator.rs:821`：「pub fn take_the_segments_to_discard(&mut self) -> Vec<SlotNumber> {」整段可再分配的交出去并摘掉，段里又有槽被分配的也摘掉，只差扣住或隔离的留着等放开。
3. 孤儿头：`crates/singlefs-core/src/mounted_session.rs:1814`：「fn segment_holding_unswept_orphan_heads<Reader: PoolReader + ?Sized>(」读段里每个槽的头，认出本池的单元头、写序实例在现行实例表里还有一行并按那一行判未发布的，整段不发（记进 `withheld_holding_unswept_orphan_heads`）；读不出实例表或读不出段里的槽也不发。零轮「装着还没清扫的孤儿头的段不发」：相容（discard 取态是三态之一，D13 已定项 4，不能当擦除用）。「清扫之后补不补发」：该段已经从待发集摘掉，之后只有再回收一个槽才会重新记上——不补发；判决没写，规则没说。已入库用例 `a_segment_holding_an_unswept_orphan_head_is_withheld_from_the_discard`（s00c 绿）。
4. 三种结局：`crates/singlefs-core/src/mounted_session.rs:1786`：「match device.discard_at(」后三臂——`Applied` 记进 `applied`、`Unsupported` 记进 `unsupported`、`Err` 记进 `failed`（`:1796`：「Err(_cause) => discards.failed.push((device_identity, segment_start)),」），都不让发布算没发成，也都不重试；长度恒为一段，长度 0 不会发（默认实现、文件后端、`O_DIRECT` 后端对长度 0 一律交 `Applied` 不计数，`crates/singlefs-core/src/block_device.rs:197`：「fn discard_at(」的文档）。判决 G5 写「默认实现返回不支持并计数」：默认实现没有状态记不了数（文档自己写明），计数在两个覆盖它的后端里；相容。
5. 与回退、整理续做交错时有没有可能被 discard 的段还被某一版根引用：回收的门槛是释放代 ≤ max(F_生效, 环里最旧有效根)，进待发集的槽已经不被任何有效根引用；回退复活只复活 defer 里或「已回收没被复用」的记录，复活之前对每份副本读盘核校验和（`every_copy_of_the_resurrected_units_reads_back`），读不回就拒回退，不会悄悄用上被 discard 过的段。我没造这样的交错序列，属读码推。
6. `crates/mutations.tsv:4453`：「m4 闭环流：回收记下的段不论整段可不可再分配都发 discard」那一行的目标测试是 checker 档的崩溃枚举（慢，超时）。同一个变异（把 `segment_is_reusable_as_a_whole` 那一判换成 `if true`）放进拷贝，不超时的 harness 层三条用例当场红（s03，已入库的用例，不是我写的）：

```
test a_segment_holding_an_isolated_slot_is_not_discarded ... FAILED
test a_segment_held_until_the_floor_takes_effect_is_discarded_only_after_the_hold_is_released ... FAILED
test a_segment_with_a_slot_still_allocated_is_not_discarded ... FAILED
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
```
所以 4453 这一行可以把「必须红的测试」换成这三条里的一条（0.00 秒），超时问题就消了。

推翻条件：造出一个 discard 打到仍被有效根引用的段；或扣住 / 隔离 / 仍分配的槽所在的段被发了 discard。

## T9　区域扫描与计划的修订、Verus 核

结论：第 2 批代码轮 T4、T5 打中的反例关掉了，没新开口子；`TwoRecordsMovableUnderOneMappingKey` 只会在坏盘上触发；核与核心层之间对拍的输入域有缺口：**记录跨度全是 1**，把选段打分改成数记录条数而不数槽数（变异 M4）对拍与 harness 层选段用例全绿；本机没有 verus，78 条 verified 复核不了。

1. 判类次序 `crates/singlefs-core/src/region_scan.rs:266`：「一条未释放记录的分类，次序：钉住 → 隔离 → 读头 → 类标签 → key 与位置（判决 m4-batch2-code-r1 S3）。」，隔离槽一律 `Quarantined`（钉住与隔离都看记录罩住的每一个槽）。`RegionChoice` 三成员、`PlannedBatch` 三成员见 `crates/singlefs-core/src/compaction_plan.rs:88`：「pub fn choose_region(」与 `:221`：「pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> PlannedBatch {」。扫描拒过的段不再选：`crates/singlefs-core/src/compaction_plan.rs:147`：「segments_refused_by_the_region_scan.contains(*start_slot);」。已入库的 `compaction_plan_chooses_regions_and_batches_by_the_clauses` 在拷贝上 21 条绿（s15b 的未变异对照即此）。
2. `TwoRecordsMovableUnderOneMappingKey`：`crates/singlefs-core/src/region_scan.rs:261`：「return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });」。触发条件是一条映射条目的两条位置项落在 R 那块盘上，合法镜像的两条位置项各在一块盘（D19 已定项 5），留存给快照的映射条目拷贝自被杀单元上一版的条目、搬迁时整条换位置，也各在两盘。所以「含快照的段永远整理不了」只在坏盘上成立；代码自己的文档也这么说（`:183` 起）；推的，没造这样的盘。`Dead` 比判决 G9 的字面窄一点：映射里没人指向它**且**头里的 key 在映射里也没有才报已死，有 key 在别处则报不一致——两者都不可搬，相容。
3. 核与核心层还差哪几条：
   - P1 只证资格，不证最优；最优（活槽最少、同分取起点最前）由核心层对拍保证：`crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs:64`：「pub open spec fn region_choice_is_sound(」。
   - 对拍域大小（用 python 按用例里的常量数出）：选段 1–3 段、每段 36 种形状（活记录 0–2 条 × 4 种不合资格之一 × 3 种扫描结果）、活意图在哪一段或没有，共 190584 个输入；计划 0–4 项、每项 5 类、K 取 1–3，共 2343 个输入。两条对拍用例在拷贝上绿（`test result: ok. 3 passed; 0 failed`，含那条 `should_panic`）。
   - 缺口：记录跨度全是 1 `crates/singlefs-core/src/compaction_plan.rs:902`：「span_slots: 1,」，段内活记录至多 2 条 `:807`：「const MOST_LIVE_RECORDS_IN_A_SEGMENT: u64 = 2;」，区域至多 4 项 `:808`：「const MOST_ALLOCATIONS_IN_A_REGION: usize = 4;」。真实的活槽数是 Σ 记录跨度（数据单元两槽、节点一槽，`crates/singlefs-core/src/compaction_plan.rs:120`：「for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {」）。变异 M4：把这一行改成只数记录起点那一槽，s15a（对拍 3 条）与 s15b（harness 选段 21 条）全绿：`test result: ok. 3 passed; 0 failed`、`test result: ok. 21 passed; 0 failed`；mutations.tsv 里文件列是 `compaction_plan.rs` 的 25 行（`grep -c` 数的）没有哪行的原文含 `record.slot.0..record.slot.0` 这一句。按判决 G2「活槽最少者先」，两个数据单元（4 槽）的段与三个节点（3 槽）的段，代码选后者，M4 选前者，没有任何用例分得出。
   - 核入参（记录 → 每段活槽数、钉住槽用 `range` 查、被拒段）的推导不在 Verus 里，只被上面的对拍盖到；P2 的前提「能搬的 key 互异」靠扫描先拒 + 计划开头的断言 `crates/singlefs-core/src/compaction_plan.rs:223`：「region_scan.movable_mapping_keys_are_pairwise_distinct(),」。
   - 复核不了：`which verus cargo-verus` 空，Verus 的 78 条 verified 与必红世界表我没跑。

推翻条件：造出让 M4 变红的输入（段内放 span 2 的记录）而对拍没覆盖——那就是缺口成立；或在合法镜像上触发 `TwoRecordsMovableUnderOneMappingKey`。

## T10　恢复读侧

结论：I-2.5 提前之后合法的搬迁镜像、合法的快照镜像都不被它拒；logged_ops 读回坏条目拒的是**整个可写挂载**，不是只拒这一版，只读与冷走读照收；和 D8 已定项 5 的「提示类不挡挂载」字面有张力，规则没说。

1. I-2.5 先判盘上读来的提示、再盖映射位置、盖完再判一次：`crates/singlefs-core/src/recovery.rs:3573`：「pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;」与 `:3578`（同一句）。合法搬迁镜像：映射条目指目的落点、父节点里的提示是源落点，两处各自位置项都按设备升序，已入库的 8 条搬迁用例（冷走读、重建、可写挂载）在拷贝上绿（s00a）；合法快照镜像：建快照 5 条（s00b）与 s09 / s10 的读回绿。映射条目值逆序：经指针覆盖的会被第二次判拒；留存给快照的映射条目不经任何指针，没有人核它的设备序（T9 第 2 条的同一个缺口），推的，没造盘。
2. logged_ops 读回：`crates/singlefs-core/src/recovery.rs:2969`：「let logged_operations_root_pointer = pointer_of(TREE_KIND_LOGGED_OPERATIONS)?;」读根、`:2999`：「logged_operation_entries_of_the_root_entries(&node.entries).map_err(|_refusal| {」解条目，解不过报 `UnitMalformed`（`crates/singlefs-core/src/recovery.rs:4176`：「pub const LOGGED_OPERATIONS_ROOT_ENTRIES_REFUSED_BY_THE_FIELD_TABLE: &str =」）。不升序 `crates/singlefs-core/src/logged_operations.rs:327`：「return Err(LoggedOperationsRootRefusal::EntriesNotInStrictlyAscendingKeyOrder);」、多于一条 `:333`：「if compaction_intents > 1 {」，探针输出见 T6 第 1 条。
3. 拒的是哪一个：把盘上那条意图的 R 起始槽改成 50241（不对齐，节点重封、校验和沿引用链补到根槽），s13 第二个用例原样输出：

```
name=probe_forward_malformed_intent writable_mount=Recovery(UnitMalformed { what: "logged_ops 根里有条目过不了字段表" })
name=probe_forward_malformed_intent read_only_mount=Ok
name=probe_forward_malformed_intent cold_walk_of_the_chosen_root=Ok(Some(3000))
```
可写挂载整个报错、不退到前一版；只读挂载与冷走读收下同一张镜像。代码注释自己写了这一取法（`crates/singlefs-core/src/recovery.rs:2968` 一带：「盘上读来的坏条目当损坏处置，可写挂载拒」），而 `crates/singlefs-core/src/mount.rs:6427` 一带写「整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5）」，指的是续做；`.claude/kb/decisions/08-核心索引结构.md:108`：「**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载」没有说**损坏的**提示类条目挡不挡可写挂载。判决 G1 只说「R 的解码拒不对齐的值」。这一格规则没说，实现取最严的一种；需要主 agent 定。

推翻条件：合法搬迁镜像或合法快照镜像在可写挂载被 I-2.5 拒；或坏 logged_ops 条目让只读挂载也报错 / 让可写挂载退到前一版接着挂。

## T11　RefFS 与模型

结论：RefFS 的三个操作与允许集合、模型「非空」的判法与核心层同口径（含搬根、含快照抄的根）；崩溃语义**不同口径**（规则没说）；mutations.tsv 第 4346 行那条没红的变异不是等价变异，给出了会红的输入。

1. RefFS 加了 `CreateSnapshot` / `DestroySnapshot` 与读快照：`crates/singlefs-reference-filesystem/src/lib.rs:434`：「Operation::CreateSnapshot(name) => {」，状态加快照集合 `crates/singlefs-reference-filesystem/src/lib.rs:38`：「pub type Snapshots = BTreeMap<SnapshotName, VisibleState>;」；建快照与删快照各是发布末尾、返回成功时抬 lo；没有文件时建快照、快照名已活着、删不活着的、有活快照时回退这四种不收。对 S7 判决原文 `research/prompts/m4-snapshot-r1-main-verification.md:36` 一致。
2. 「非空」口径：模型 `crates/singlefs-harness/src/model.rs:817`：「fn user_visible_state(&self) -> ModelUserVisibleState {」由活头树根是谁写的、每个快照抄的两个根、整理搬过的根单元三样组成；实现按字节比 `crates/singlefs-core/src/mount.rs:3414`（T1 第 5 条）。建快照算非空、删快照的意图那次不算、最后一批算、搬根算——两边相同；搬的是只被快照引用的根单元时，实现里码 14 / 15 条目的根指针跟着换成映射位置、「非空」那一判为真，模型的 `user_visible_root_units`（`crates/singlefs-harness/src/model.rs:778`：「pub fn user_visible_root_units(&self) -> Vec<ModelUserVisibleRootUnit> {」）把快照抄的根也算进搬根的名单——同口径。探针 s07 原样输出：

```
name=probe_forward_snapshot_root_relocation unit=snapshot_extent_root source_slot=50240 kept_entry_destination_slot=50283 tree_table_entry_root_hint_slot_before=50240 after=50283 entry_root_pointer_changed=true user_visible_trees_changed=true
name=probe_forward_snapshot_root_relocation unit=snapshot_inode_root source_slot=50244 kept_entry_destination_slot=50274 tree_table_entry_root_hint_slot_before=50244 after=50274 entry_root_pointer_changed=true user_visible_trees_changed=true
```
3. 崩溃语义不同口径：RefFS 在读出的文件定不了落在哪一个末尾时取下标最大的那个状态的快照集合 `crates/singlefs-reference-filesystem/src/lib.rs:594`：「.find(|allowed_state| allowed_state.state == recovered)」（`:584` 的注释自己写了「快照集合不比」）。建快照那次发布崩在中途，核心层的结局是二选一（发布落盘与否，快照有与没有）；删快照崩在两批之间，核心层的结局是：快照条目还在、销毁中，读快照入口照读，可写挂载续做到删完（T3）。RefFS 把前者一律答「有」、后者一律答「没有」。探针 s08 原样输出（建快照与删快照各是「写了之后报错」、之后崩溃，恢复读出 A）：

```
name=probe_forward_reffs creation_failed_after_writing allowed_states=2 snapshots_after_the_recovery=[SnapshotName(7)]
name=probe_forward_reffs creation_returned snapshots_after_the_recovery=[SnapshotName(7)]
name=probe_forward_reffs destruction_failed_after_writing snapshots_after_the_recovery=[]
```
允许集合里确有两个状态（`allowed_states=2`），恢复之后模型却只认一种快照集合；之后再提交「删这个快照」（建快照那次其实没落盘时核心层拒），模型会多收一个。第一版崩溃判定只比文件，这个差不会立刻变红；一旦随机历史把「崩溃 → 恢复 → 删快照」连起来就会露出。规则没说（判决 S7 只写「RefFS 加三操作」）。
4. mutations.tsv 第 4346 行没红的原因与会红的输入：`crates/mutations.tsv:4346`：「m4 快照模型对拍：「非空」不看快照条目」把 `snapshot_roots` 去掉；但 `root_relocations` 逐个根单元记一项，而名单里含快照的两个根（`crates/singlefs-harness/src/model.rs:830`：「root_relocations: self」），快照个数变了它的长度跟着变，状态照样不等——30 条原有 `model::` 单测在变异下全绿（s05b：`test result: ok. 30 passed; 0 failed`）。会红的输入：状态只差「快照抄的是哪个根」、快照个数相同。序列：四次覆盖写之后 建 S1（txg 10）→ 覆盖写（11）→ 删 S1（12、13）→ 建 S2（14）。活头都是 txg 11、各带一个快照，S1 抄 txg 9 的根、S2 抄 txg 11 的根；非空根从新到旧 14、13、11、10、9……，状态各不相同，抬 F 的上限取第 4 新的非空根 = 10；变异下 14 与 11 的状态相等被去重，上限变 9。三条原样输出：

```
s04（干净）：name=probe_forward_model_ceiling ceiling=Some(ModelCheckpointTxg(10))
s05a（变异 M3，探针红）：name=probe_forward_model_ceiling ceiling=Some(ModelCheckpointTxg(9))   test result: FAILED. 0 passed; 1 failed
s06（真实现，同一段历史）：name=probe_forward_real_ceiling ceiling=CheckpointTxg(10) txg_of_the_version_that_created_the_first_snapshot=CheckpointTxg(10) txg_of_the_newest=CheckpointTxg(14)
```
真实现与干净的模型都是 10，变异的模型是 9。这条输入进 `model::tests` 就杀这行变异；模型对拍的随机历史是否会走到这个序列（要在 `rollback_floor_ceiling` 被比的事件上），我没量。
5. 判别力：探针 s04 / s05a 同一个文件，变异前绿、变异后红；s06 是独立的一条路（真实现 + 池级读盘），与模型不共用代码。

推翻条件：RefFS 与实现对同一段「写了之后报错再崩溃」的快照操作答出相同的快照集合（目前不同）；或在 `model::tests` 里加上面那条输入之后变异 4346 仍不红。

## 零轮读法逐条判（正文第二节）

| 读法 | 判 | 依据 |
|---|---|---|
| 码 14 的 `previous_snapshot_txg` 写前驱链 | 相容 | T1 第 2 条；T3 第 1 条前驱传递；`research/prompts/m4-snapshot-r1-main-verification.md:30` 的「显式 S.txg」与前驱链第二来源 |
| 建快照那次事务号取上一版 + 1 | 相容 | T1 第 6 条：与覆盖写、整理、删除三个入口同式，回退写 0 |
| `mount_writable` 不换入口 | 相容 | s00b 里挂载写行与暖机之后快照条目原样续上；s09 第二个用例挂载续做删快照 |
| deadlist 节点空间按普通分配 | 相容 | T2 第 6 条；但判决 S3 要的强制开关缺（T2 第 5 条） |
| 下一侧是活头时 hi 取这一批 txg | 相容 | T2 第 7、8 条；会话先走空，意图之后不会插进杀用户单元的发布 |
| 删快照批大小与 unlink 共用开关 | 相容 | T3 第 2 条，144 |
| C300 写前拒 | 相容 | 留存映射条目缺 ⇒ `DeadlistEntryNotReleasable`，在任何落盘动作之前（`crates/singlefs-core/src/transaction.rs:4894`：「.ok_or_else(|| PublishError::DeadlistEntryNotReleasable {」） |
| 有树而文件集合为空的一版上删快照放行 | 相容 | T3 第 4 条与 s09 |
| `segs0` 沿用记录口径 | 口径相容，另有差一 | T6 第 4 条，s12 |
| `B_gc` 数整理的每一次发布 | 相容 | T6 第 2 条 |
| 丢意图也发一次发布 | 相容 | T6 第 2 条 |
| 意图、不合格段表、常驻隔离集归位到 `MountedSession::compaction` | 相容 | 意图在 logged_ops（盘上权威），后两样只住内存、一次挂载；跨挂载不记（T7 第 2 条，规则没说） |
| 装着还没清扫的孤儿头的段不发 discard | 相容 | T8 第 3 条 |
| 同一份扫描里两条能搬的记录映射 key 相同整次拒 | 相容 | T9 第 2 条；`plan_batch` 开头断言 |
| `empty_segments` 改记录口径增量维护 | 相容 | T7 第 3 条，32000 次比较全等 |
| 合取 7 被拒不推抬 F | 相容，后果见 T7 第 5 条 | 与用户 2026-10-13 定案一致 |
| 调查员修法（装着搬走 key 的叶连同到根整条标重写） | 相容 | T5 第 2、3 条；多出来的重写不过准入（T5 第 4 条） |

## 丁 3 报告「交主 agent 的读法」逐条判（补丁目录 `~/.local/share/singlefs/work/m4-gc-intent-writer/patch/report.md` 第 119–135 行，正文写 12 条、报告里是 15 条）

| 条 | 读法 | 判 | 依据 |
|---|---|---|---|
| 1 | 签发单独一次发布 | 相容 | D8 已定项 5 三态表第二行要「意图已提交、工作做了一半」的崩溃点；多用一个根槽，记进 `B_gc`（T6 第 2 条） |
| 2 | 丢意图 = 一次只删那一条的发布，四种情形都发 | 相容 | G2 K-c「丢掉这条意图连同 R 排他」；意图删在盘上才不会下次挂载接着推（T6 第 2 条、s13） |
| 3 | 最后一批的认法与同发布删意图 | 相容 | G4 停机谓词与 D8 已定项 5（T6 第 2 条；`crates/singlefs-core/src/transaction.rs:7585` 起核账） |
| 4 | `B_gc` 数每一次发布 | 相容 | T6 第 2 条 |
| 5 | `segs0` 取签发之前那一版的记录口径 | 口径相容，另差一 | T6 第 4 条，s12 |
| 6 | 事务号取下一个号 | 相容 | T1 第 6 条 |
| 7 | 挂载续做只在删除续做走到空时推、只推盘上那条 | 相容 | G8 次序；`crates/singlefs-core/src/mount.rs:6443` 起；产品开关下不续做（T6 第 5 条、s13） |
| 8 | R 排他在写行之前装上 | 相容 | `crates/singlefs-core/src/mount.rs:7512`（T6 第 3 条） |
| 9 | logged_ops 随现行那一版，回退留着意图 | 相容 | `crates/singlefs-core/src/mount.rs:8201`、s13 回退输出 |
| 10 | 孤儿头判法，扣下的段不追着发 | 相容 | T8 第 3 条；「清扫之后补不补发」规则没说 |
| 11 | 常驻隔离集经分配器上的过路名单进会话 | 相容 | T7 第 2 条 |
| 12 | 分配器上那一份仍叫 `issue` | 相容 | 命名，不影响行为 |
| 13 | 合取 7 新成员之后会话照旧为它推抬 F | 被后一次改动取代 | 现在 `EmptyClusterSegmentsBelowTheFloor` 归不是空间不够那一类（`crates/singlefs-core/src/mounted_session.rs:1026`：「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」），被拒不推抬 F，对应用户 2026-10-13 的定案与零轮读法「合取 7 被拒不推抬 F」（T7 第 4 条） |
| 14 | 写账新种类 `LoggedOperationsTreeNode` | 相容 | `write_accounting.rs` 里加了一格，值 0 |
| 15 | 签发那次发布落盘途中失败冻结时撤了 R 排他，冻结的发布原样重发之后盘上有意图，会话下一步的断言会红 | 读码上是潜在口子，今天会话层走不到 | 重发入口 `crates/singlefs-core/src/transaction.rs:1384`：「pub fn resend_the_frozen_publish<Device: BlockDevice>(」是公开函数，会话不调它；调用方自己重发成功再用同一个会话推整理，会撞 `advance_compaction_by_one_publish` 开头那条分配器 / logged_ops 一致的断言（`crates/singlefs-core/src/mounted_session.rs:2213`：「assert_eq!(」）。没写探针 |

## 给主 agent 的发现清单（按重要性，每条在上面哪一格有依据）

1. T6：写进意图的 `segs0` 与判决 G1 字段表的定义句差一（签发那次发布自己开新聚簇段时），会让「一条意图完成而全局差值为假就不签下一条」提前判假；探针坐实（s12）。改法二选一：`segs0` 取签发那次发布写下的第 11 项，或把定义句改成签发之前那一版的值。
2. T7：产品路径上整理起不来（批大小条款没写 + 水位全关），合取 7 的 N₀ = 1 却开着；一块盘没有全空聚簇段时前台写入类发布全拒、不推抬 F，只有删除类能解。与用户 2026-10-13「现在就接」的定案相容，但后果要在判决里写明；这一格我没造出真盘取样点。
3. T10：坏的 logged_ops 条目让可写挂载整个报错（只读与冷走读收下），与 D8 已定项 5 的「提示类……不挡挂载」字面有张力，规则没说，要定。
4. T11：RefFS 与核心层在「写了之后报错再崩溃」的快照操作上口径不同（RefFS 取下标最大的状态：建快照一律有、删快照一律没有）；mutations.tsv 第 4346 行没红的变异不是等价变异，`probes/append_to_model_tests.rs` 那条输入杀它（干净 10、变异 9）。
5. T9：选段同证过的核的小输入对拍，输入域里记录跨度全是 1；把「活槽」改成数记录条数的变异 M4 对拍与 harness 21 条全绿。要么对拍加 span 2 的记录，要么加一行变异。
6. T2：判决 S3 要的「强制立即释放 / 进 deadlist」只供测试的开关没有。
7. T5：`publish_relocation_batch` 的文档注释写「式子那一判照判」与行为（普通批不判）不符；一批多写的映射节点只受落点约束，不过 ckpt_cost，量到的取样点没越过。
8. T8：mutations.tsv 第 4453 行（超时）可以把必须红的测试换成已入库的三条 harness 用例（0.00 秒）。
9. T1：文件集合为空的一版上建快照被拒，规则没说（RefFS 同取）；T8：discard 交 Unsupported / 报错 / 孤儿头被扣下之后该段不补发，规则没说；T6：「已经过去的发布数」不数挂载自己的写行与暖机，规则没说。
10. T2：kb 05 号决策两行（第 277、278 行）对「哪一份」的区间写法不同，判决 S4 与代码取后者。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 建快照那次发布 | 一致 | S1、S2、「建快照算非空」逐字段对上；每类发布原样续快照条目（探针）；文件集合为空的一版上建快照被拒，规则没说 |
| T2 释放分流与 deadlist | 一致，缺一项 | 切口、六角色、`birth == S.txg` 进 deadlist、key 43 / 叶 376 对上；判决 S3 要的强制开关代码里没有 |
| T3 删快照意图、批与续做 | 一致 | 意图在 flags 位 0、前驱传递、批 144 共用开关、最后一批删一对；销毁中的快照建拒、读照读、回退拒；挂载续做到空 |
| T4 回退拒与读快照 | 一致 | 回退只数码 14（等价于 14 / 15）；读快照提示不通走映射；五个读树表的读者对码 14 / 15、8、12 全放行（探针） |
| T5 一批搬迁（G3） | 一致 | 一批一个事务、映射为准；被钉单元搬走再删快照放的是目的地；多出来的重写不过准入，取样点没越过 ckpt_cost；文档注释与行为不符 |
| T6 整理意图上盘与续做 | 不一致（一处） | 字段表与各发布对上；`segs0` 与定义句差一（探针）；水位 V-a 自报未实现；三条路与坏条目的结局已量；零轮读法相容 |
| T7 R 排他、签发与停机、合取 7 | 一致，有后果 | R 排他无漏口；`empty_segments` 与全扫重算 32000 次全等；合取 7 与用户定案一致，产品路径上整理起不来 |
| T8 discard 调用者（G5） | 一致 | 时机、整段谓词、三种结局对上；4453 行有不超时的杀手；清扫之后不补发规则没说 |
| T9 区域扫描与计划、Verus 核 | 一致，对拍有缺口 | 反例关掉了；对拍域 190584 / 2343 个输入，记录跨度全是 1，变异 M4 不红；verus 本机没有，复核不了 |
| T10 恢复读侧 | 一致，一处规则没说 | I-2.5 提前不拒合法搬迁 / 快照镜像；坏 logged_ops 条目可写挂载整个拒、只读收下 |
| T11 RefFS 与模型 | 一致，口径差一处 | 「非空」同口径（含搬根、快照抄的根）；崩溃语义不同口径；4346 变异非等价，给了会红的输入 |
| 零轮读法 17 条 | 全部相容 | `segs0` 口径相容但另差一；合取 7 的后果见 T7 |

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决；不写攻方的反例。
- 没跑任何门禁阶段、没跑 checker 档测试（崩溃点枚举、GPU 对拍、QEMU、herd7、整表变异）、没起 GPU、没碰 `crates/singlefs-checker-tier/`、没读 checker 与 `crates/singlefs-checker` 的文件（不在这一轮的被判文件里）。
- T5 的崩溃前后「各读到哪一份」只靠读码与已入库用例，没跑崩溃点枚举。
- Verus：本机 `which verus cargo-verus` 为空，78 条 verified 与必红世界表没复核。
- 没造 4 GiB 盘上零个全空聚簇段的取样点（T7 第 5 条）；没造合法镜像上触发 `TwoRecordsMovableUnderOneMappingKey` 的盘（T9 第 2 条）；没造 `ckpt_cost` 被一批越过的取样点（T5 第 4 条）。
- 没量模型对拍的随机历史会不会走到 T11 第 4 条的序列。
- 没读被禁读清单里的文件。第 2 批正推腿的模型目录 `m4-batch2-code-r1-sonnet-model/` 的 `rerun.sh` 与探针文件名我看过一眼学格式（它不在禁读清单里），没读它的报告。
- 整份 `rerun.sh` 没在前台一次跑完：冷编译加四个 4 GiB 盘的用例远超 240 秒前台上限。`out/expected.txt` 是按同一条摘要管线从我在拷贝上逐步跑出的日志里抽出来的（`step=… exit=…`、排序后的 `name=probe_forward…` 行、`test result` 行去掉用时、失败条数）；`rerun.sh` 的准备段与全部 `replace_once` / `restore`（四处变异的原文必须恰好命中一次）我用 `RERUN_ONLY=none` 在真仓与冻结副本上空跑过两遍，通过。

## 要主 agent 起的长活

- 整份复跑（冷编译 + 21 步，本机负载高时预计一个多小时）：`bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r1-sonnet-model/tools/rerun.sh --repo ~/.local/share/singlefs/work/m4-predev --frozen ~/.local/share/singlefs/work/m4-segment-code-r1/repo --work <新的草稿目录>`；末尾打 `result=same` 或 `differs`，`differs` 时先看 `<work>/summary.txt` 与 `out/expected.txt` 的 diff。要只跑几步：环境变量 `RERUN_ONLY="s01-allocator-empty-segments s12-segs0"`（这时不比 expected）。
- 没有别的要等的。

## 草稿目录清理

交回前删了：`~/.local/share/singlefs/work/m4-segment-code-r1-forward/repo-copy/`（冻结副本的拷贝，28M）、`~/.local/share/singlefs/work/m4-segment-code-r1-forward/target/`（编译目录，17G）；`rerun.sh` 空跑时建的三个临时工作目录（各约 28M）当场删了。留着的只有文本：`logs/`（116K，各探针与变异的原始输出）、`probes/`（104K）、`cite/`（20K）、`progress-notes.md`；目录本身仍归本轮认领（`.owner`）。
