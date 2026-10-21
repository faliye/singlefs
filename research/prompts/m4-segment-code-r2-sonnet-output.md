# m4-segment-code-r2 辩方腿报告（Sonnet，2026-10-16）

立场：辩方，复核第一轮判决 `research/prompts/m4-segment-code-r1-main-verification.md`：打中的 T4、T5、T6、T11 是不是真的错、F1–F6 够不够得着、F1 的零轮读法是否仍守住 G3；不改代码的 T7、T8、T10 站不站得住；每条修补同不同样打中别的替代改法。
冻结副本：`~/.local/share/singlefs/work/m4-segment-code-r2/repo/`，下文 `crates/…:行号` 都是它里面的行号；kb 引文写 kb 文件名加那份文件自己的行号。
草稿目录拷贝：`~/.local/share/singlefs/work/m4-segment-code-r2-defense/repo`（与冻结副本 sha256 同、touch 过 mtime；探针要的几处只打印的改动与变异都是临时打的、用完换回，补丁在模型目录），编译目录 `…/m4-segment-code-r2-defense/target`（种子来自 `m4-segment-code-r1-main-rerun-forward/target`，只读拷）。

## 0　开工核对

命令（在冻结副本根上）：`sha256sum -c ~/.local/share/singlefs/work/m4-predev/research/prompts/m4-segment-code-r2-snapshot/crates-sha256.txt`，原样输出：

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

## 进度（报告分段落盘，重起先读这里已写完的段）

- 段 0：开工核对。
- 段 1：§1 R1 / F1 / T5。
- 段 2：§2 T4 / F9，§3 R2 / F2 / T6 后一半。
- 段 3：§4 R3 / F3 / T6 前一半，§5 R4 / F4 / T2，§6 R5 / F5 / T1。
- 段 4：§7 R6 / F6 / T11。
- 段 5：§8 R7 与顺带撞到的一处，§9 T7 / T8 / T10。
- 段 6：§10 判决条目表，§11 没做什么，§12 要主 agent 跑的命令。
- 段 7：§13 模型目录，§14 草稿目录清理。

## 1　R1 回退之后的映射（F1）与第一轮 T5

结论：**站得住（部分）**。T5 是实现错，F1 改到了打中的那一形；F1 不取「整棵目标版」的读法与 D23 的字面一致，仍守住 G3。F1 多出来的重写在默认几何上罩得住（算术），在更大的池上没有上界论证。

### 1.1 打中的是实现，不是判据或测试
- 根因锚点还在、F1 没动它：「central_mapping_tree: current.central_mapping_tree.clone(),」（`crates/singlefs-core/src/mount.rs:8191`）。回退造的「上一版」里映射树的节点仍是现行那一版的；F1 是在它之后补一条：把「两版映射里位置项不同」的 key 交给发布路径，装着它们的叶连同到根的路这次重写。
- 比较：「central_mapping_locations_in_the_version(current, key)」（`crates/singlefs-core/src/mount.rs:8244`）与「!= central_mapping_locations_in_the_version(target_version, key)」（`crates/singlefs-core/src/mount.rs:8245`）；调用点「let previous = version_to_publish_the_rollback_after(current, &target_version);」（`crates/singlefs-core/src/mount.rs:8468`），交给发布路径在 `crates/singlefs-core/src/mount.rs:8531`。
- 发布路径只重写落在新一版 key 集合里的 key：「.filter(|key| mapping_keys.contains(key)),」（`crates/singlefs-core/src/transaction.rs:6896`），重写那一步是「fn plan_rewriting_the_paths_to_the_moved_entries(」（`crates/singlefs-core/src/transaction.rs:6912`），与整理的一批同一条路。
- 重写的叶里条目的值取新一版单元指针里的位置：「build_mapping_entry(key, *locations),」（`crates/singlefs-core/src/transaction.rs:11081`）；回退那一版的单元指针取目标版的，而目标版重建时映射位置已覆盖提示：「take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {」（`crates/singlefs-core/src/recovery.rs:3574`）。

### 1.2 F1 不取「整棵目标版」，仍守住 G3
- 判决字面：「中央映射树取目标那一版的」（`research/prompts/m4-segment-code-r1-main-verification.md:41`）。D23 已定项 14 的字面相反：「按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入」（`.claude/kb/decisions/23-journal的角色与格式.md:378`），中央映射树在那一串里。F1 的取法（节点现行版的、条目位置取目标版的）与这一句相容，判决字面与它冲突。
- 回退的复活 / 验盘 / 释放集只走用户可见单元：「pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {」（`crates/singlefs-core/src/mount.rs:7682`）里 `MappingTree` 与 `MappingTreeNodeBelowTheRoot` 两臂是 false；取目标版整棵映射树就要把目标版的映射树节点也复活、逐盘验、从释放集里扣掉，这一串今天没有。
- G3 的字面：「写路径改成以映射为准」（`research/prompts/m4-gc-r1-main-verification.md:34`）。F1 之后回退那一版里用户可见单元的映射条目值 = 目标版映射位置 = 回退复活的落点；探针里池级 checker 的 I-3.9、I-3.15（候选根引用的槽不带已释放标志、记录罩住的单元在映射里有 key 指着这一槽）每一步都不红（下面）。
- 归属：这一点在判决字面与 D23 之间，不是 F1 的错；建议主 agent 把判决 F1 一条的「整棵取目标版」改成 D23 的读法再入正文。

### 1.3 探针：F1 有判别力（r1 核心与「F1 变异」都红 24 格，r2 核心 0 格）
49 个场景（`model/probes/defense_probes.rs` 的 `f1_scenarios`）：152 + 290 个数据单元的文件，整理两趟中间覆盖写把搬过的单元的 key 换掉、整理停在意图半路再回退、两趟整理之间重挂再冷启动、连续两次回退；回退目标扫根环里的位置；每一步过池级 checker 与理想模型。三份二进制同一份探针：
```
$ bash model/tools/probe_group.sh bin/probes-r2-core f1 25 f1
exits:      25 0
scenarios=49 not_completed=0 reds=0
$ bash model/tools/probe_group.sh bin/probes-f1-neutered f1 25 f1mut      # r2 核心，只把 crates/singlefs-core/src/mount.rs 里 mapping_keys_whose_target_locations_differ_from_the_current_mapping 改成返回空表（model/patches/mutation-f1-no-mapping-keys.diff）
exits:      25 0
scenarios=49 not_completed=24 reds=108
$ bash model/tools/probe_group.sh bin/probes-r1core f1 25 f1r1             # 第一轮冻结副本的核心 + r2 的装置与 checker
exits:      25 0
scenarios=49 not_completed=24 reds=108
```
红的 24 格实现答复三种：`PublishError::ReleaseTargetAlreadyReleased` 12 格（回退之后的覆盖写被拒）、`RollbackError::CurrentVersionUnitNotReleasable` 9 格（第二次回退被拒）、`RollbackError::TargetVersionUnreadable` 3 格（以回退那一版为目标的回退读不出它的映射），与第一轮攻方 T5 同形（逐场景行在 `model/out/f1-r1-core.txt`、`f1-r2-core-with-f1-neutered.txt`、`f1-r2-core.txt`）。
随机历史：整理夹着回退那一组比重（`COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`）在新种子区间（偏移 940000 起 120 段、950000 起 93 段，每段 40 步，回退那几步之后的镜像过池级 checker）：
```
rb1: segments=57 (其余 63 段的进程被前台时限截掉，没算) not_completed=2 reds=0
rb2: segments=31 not_completed=0 reds=0
rb3-1: segments=31 not_completed=0 reds=0
rb3-2: segments=31 not_completed=1 reds=0
```
150 段里池级 checker 一条都不红（命令 `bash model/tools/sweep.sh bin/sweeps-r2-core-width compaction_rollback <起点> <每进程段数> <进程数> <标签> rollback 40`）；3 个 `not_completed` 都不是回退的映射，见 §3.3（两个是 F2 的同形另一支）与 §8.1（一个抬 F 上限）。

### 1.4 判决点名的几种形状
| 形状 | 探针 | 结果 |
|---|---|---|
| 目标版之后多批整理 | `f1-two-compactions-with-an-overwrite-between-rollback-{0..7}`、`f1-two-rollbacks-*` | r2 全走完、0 红 |
| 整理搬的单元又被覆盖写过 | 同上第一组（中间 `sequential(290)`） | r2 全走完、0 红；F1 变异下这一组 7 格红（回退目标 1–7），与第一轮同形 |
| 回退目标落在意图半路、现行版带活意图 | `f1-rollback-with-a-live-intent-then-compact-again-rollback-{0..7}`（整理崩在第 1 批后，回退，覆盖写，再整理到停） | r2 全走完；`advance_compaction_by_one_publish` 开头 `assert_eq!` 分配器上的活意图 = 现行版 logged_ops 里那条，8 格都过 |
| 两块盘不对称 | 不可造：搬迁的源落点是两盘同槽的一个 `Placement`（「计划里的一项：搬哪个单元（映射 key）、它现在在哪（源落点，两盘同槽）」，`crates/singlefs-core/src/compaction_plan.rs:194`），释放集 `any(...)` 跳过判（`crates/singlefs-core/src/mount.rs:8425`）在同槽下与 `all` 等价 | 造不出，写清楚 |
| 映射树只有一片叶 | `model/probes` 的 `f1_small_scenarios`：文件 0 / 20 / 60 个单元 × 强制选段 0–3 × 回退 0–4，60 格 | r2 与「F1 变异」**都**走完、0 红（`model/out/f1small-*.txt` 逐行相同）。一片叶时回退那次发布本来就整片重写（分配记录树节点换 key），F1 的那一条是空操作；T5 要映射树至少两片叶才露 |
| 回退不动 logged_ops、R 排他与意图 | 同「意图半路」一格；R 排他由 `advance_...` 的断言守，回退之后再整理 8 格都走完 | 守住；「暂停的意图之后再起」今天没有取样点（暂停要空间不够，不造） |

### 1.5 多出来的重写过不过准入（ckpt_cost）
- 回退不判式子，空间从 checkpoint 保留池出：「回退不判式子，空间从 checkpoint 保留池里出」（`crates/singlefs-core/src/admission.rs:1506`）。保留池里中央映射树那一项是「min(这一层的节点数, 删的 + 插的)」逐层累加（`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`，`crates/singlefs-core/src/admission.rs:1382`），「删的 + 插的」按分配记录树整棵几何与记账树节点算。
- 探针算术（`defense_probe_arithmetic`）：中央映射树叶容量 `leaf_entries=294`；默认两块 4 GiB 盘单元区 211968 槽，映射条目至多 211968 条 ⇒ 叶至多 ⌈211968 ÷ 294⌉ = 721 片；分配记录树整棵几何 529 个节点（「1 + 2 × 264 = 529；」，`crates/singlefs-core/src/admission.rs:1273`），删加插至少 2 × 529 = 1058 > 721。逐层 `min(节点数, 删加插)` 饱和，F1 把整棵映射树的叶都重写也在这一项之内。
- 限度：这是算术，不是量出来的；盘更大、映射密度更高时叶数涨得比分配记录树快（叶 ≈ 槽数 ÷ 294 对 节点 ≈ 槽数 ÷ 812 × 2），上界不再罩住，回退在几乎写满的池上可能被落点拒（`RollbackError::Publish`，盘上不动、分配器换回）而不是写坏。没有造出这一格（要几十万条映射）。

### 1.6 替代改法
| 替代 | 判断 |
|---|---|
| 判决字面：整棵取目标版映射树 | 与 D23 字面冲突，要扩复活 / 验盘 / 释放集到映射树节点（§1.2）；不更好 |
| 回退前先把所有用户可见 key 所在叶全部标重写（不比位置） | 一样对、多重写；比较是精确的，不如它省；若担心 `!=` 比较漏形，这是更稳的退路，代价是上面 §1.5 那一项 |
| 把回退做成一次反向整理（用 `publish_relocation_batch` 把 D 搬回 S） | 重新拷字节、占批预算，且回退的复活 / 释放已经把 S 记成已分配、D 记成已释放，两套账打架；更差 |
| 拒绝跨整理的回退 | 收窄候选集，和 D16 / D23 的候选集定义冲突；更差 |

## 2　第一轮 T4（回退拒与读快照）与 F9：坏在判据，F9 够得着

结论：**站得住**。T4 红的是池级 checker 的 I-3.5，不是核心层：同一份实现、同一段合法历史，第一轮冻结副本的 `walk.rs` 红，r2 的 `walk.rs` 绿；理想模型每一步都对得上（结局 `Completed`）。

- 根因（改前，转述）：I-3.5 的「在哪棵用户可见树里」标签取自一张跨全部版本、按落点查的表，后走的版本顶替先走的。F9 的改动：每版走读时自己记，每版走完把这一张存进该版：「fn note_the_user_visible_tree_of_a_placement_in_this_version(&mut self, placement: (u32, u64)) {」（`crates/singlefs-checker/src/walk.rs:1580`）、「user_visible_tree_of_each_placement: std::mem::take(」（`crates/singlefs-checker/src/walk.rs:1509`）。
- 探针一：攻方 T4 的 5 步历史（无快照回退 → 建快照 → 快照下覆盖写）及其两个延长。同一份探针二进制，只把 `walk.rs` 在 r2 与第一轮冻结副本两份之间换：
```
$ PROBE_GROUP=t4 bin/probes-r2-core --ignored --nocapture --exact defense_probe_group        # r2 的 walk.rs
PROBE group=t4 name=t4-attack-five-steps ending=Completed steps=5 rollbacks=["applied"] reds=0
PROBE group=t4 name=t4-five-steps-then-destroy-and-cold-start ending=Completed steps=7 rollbacks=["applied"] reds=0
PROBE group=t4 name=t4-five-steps-then-second-snapshot ending=Completed steps=9 rollbacks=["applied"] reds=0
$ 同上，bin/defense_probes-with-the-round-one-checker                                          # 第一轮的 walk.rs
PROBE group=t4 name=t4-attack-five-steps ending=Completed steps=5 rollbacks=["applied"] reds=1 first_reds=Operation(4) I-3.5：相邻两版 (实例 {2}、txg 6) → (实例 2、txg 7)（NoHole）：后一版引用、前一版不引用的单元盘 0 槽 50180（用户可见树 None）……
（另两行 reds=1 与 reds=5，同一条 I-3.5；逐行在 model/out/t4-r1-checker.txt）
```
- 探针二：攻方的快照那一组比重原样不改，种子基 + 7000000 + 下标（攻方报告 `research/prompts/m4-segment-code-r1-opus-output.md` 里的 2022、2035、2041、2052），每一步过池级 checker：
```
r2 的 walk.rs：   seed_offset=7002022 / 7002035 / 7002041 / 7002052  ending=Completed reds=0（四行）
第一轮的 walk.rs：seed_offset=7002022 reds=13；7002041 reds=8；7002052 reds=2；7002035 reds=0
```
  7002052 那一行红的单元带 `Some(UserVisibleTreeOfTheWalk { kind: 1, tree: 65536 })` 标签，是攻方说的另一形（标签非 None、「整棵继承」落空），F9 的另一处改（父单元下面挂它的每一个孩子，`walk.rs` 里 `note_references_of_a_pointer_followed_by_the_walk` 一节）一并收了。逐行 `model/out/t4-snapshot-seeds-r1-checker.txt`、`t4-snapshot-seeds-r2-checker.txt`。
- F9 没把必红世界钝掉：`checker_known_bad_images`（266 个）、`checker_snapshot_space_accounting_must_red_worlds`（11）、`checker_compaction_intent_and_relocation_must_red_worlds`（12）、`checker_judges_reference_intervals_and_every_new_root_segment_field`（15）、`pool_checker_judges_the_snapshot_entries_of_the_tree_table`（4）、`pool_checker_shared_subtree_below_a_snapshot_only_unit_keeps_the_children_it_shares_with_the_head`（1）在 r2 拷贝上全绿（`test result: ok` 六行，`logs/checker-tests.log`）。
- 推翻条件：入库装置上 r2 的 `walk.rs` 在这 5 步历史或四个种子上红 I-3.5；或第一轮的 `walk.rs` 在它们上不红（那是我没复现攻方的量）。
- 没做：S8 的 (b) 款读法补「回退之后建快照」一形归阶段同步，不在这里改；没跑 checker 档的崩溃枚举。
- 替代改法：攻方提的「判 (b) 款时按 R 自己的用户可见树根重算子树成员」——按版重算，不存表；对本题等价（同一组历史同判），代价是 I-3.5 每对版本多走一遍子树；F9 取的存表按版记在内存占用上多一张表 / 版，两者不分高下，F9 不更差。

## 3　R2 腾空 R 的那一批与删意图（F2），连同第一轮 T6 的后一半

结论：**部分**。F2 守住了判决点名的那一形（R 被本批腾空 → 同一批删意图），但同形的另一支（计划里的单元都被顺带重写、一个没搬、R 还不空）F2 没封，已造出取样点：模型照样判不合。核不过那一支在产品路径上走不到。

### 3.1 F2 怎么判「这一批之后 R 按记录全空」
- 计划条数等于扫描条数的是最后一批，直接带删意图：「if planned.len() == scan.allocations.len() {」（`crates/singlefs-core/src/mounted_session.rs:2529`）。F2 加的是其余的批：照抄意图而在算好的账上核 R：「CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,」（`crates/singlefs-core/src/mounted_session.rs:2540`）；发布路径在落盘之前核「CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord(region) => {」（`crates/singlefs-core/src/transaction.rs:7854`），R 已全空就拒成 `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`（`crates/singlefs-core/src/transaction.rs:7860`），会话在「}) if kind == CompactionPublishKind::Batch => {」（`crates/singlefs-core/src/mounted_session.rs:2595`）照同一批删意图再发。
- 条款：「目标态到达与最后一批同发布删」（`research/prompts/m4-gc-r1-main-verification.md:35`）。腾空 R 的那一批同发布删意图，F2 之后成立；既有用例两条绿（`tests/compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish.rs`，`logs/existing-fix-tests.log`）。
- 核不过那一支在产品路径走不到（读码推的）：再发那一次与头一次只差 logged_ops 一条（删 / 照抄），分配器换回进来时那一份、取落点不读它，R 排他开着时落点不进 R；只有只供测试的开关关掉排他、或「RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => false,」（`crates/singlefs-core/src/transaction.rs:7842`）强制才进。强制用例在、过（同一个测试文件里的 `the_intent_deletion_refused_after_a_batch_emptying_the_region_publishes_the_batch_carrying_the_intent_and_the_next_publish_deletes_it`）。
- 模型 `(false, true, …)` 臂：「(false, true, true | false) => {」（`crates/singlefs-harness/src/model.rs:3700`）报「一次整理发布要么搬一批、要么删掉意图」。**(false, true, true)**（没搬、意图照抄、R 按记录全空）今天只有强制开关会发出：模型判不合是对的，不是模型错——它守的是 G4 那一句；强制用例绕开模型直接调会话，所以不撞它。**(false, true, false)**（没搬、意图照抄、R 不空）见 3.3，两边都说不通。

### 3.2 `batches_published` 的计数
- 记批只记搬了单元的：「if !version.units_relocated_by_this_publish.is_empty() =>」（`crates/singlefs-core/src/mounted_session.rs:2068`）。用户改动之后「至多一批」的停判看这个数：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」（`crates/singlefs-core/src/mounted_session.rs:2039`）。没搬单元的照抄发布不计批，所以这一支的「至多一批」变成「至多一批搬了单元的」；一趟的总发布数仍被预算罩着（读码推的，没造）。

### 3.3 同形另一支：计划里的单元都是顺带重写的、R 还不空（F2 没封）
取样点（不改被判代码；拷贝上加了只打印的跟踪行，`model/patches/defense-trace-zero-move-batches.diff`）：
```
$ bash model/tools/seed_runs.sh bin/sweeps-r2-core-width
SEEDRUN tag=rollback-940015 exit=0
DEFENSE_SEED weights=compaction_rollback width=4g seed_offset=940015 ending=NewFinding { signature: ModelDisagreement { aspect: "这一次整理发布对意图做的事" }, observation: FailureObservation { position: Operation(35), operation_kind: Some(CompactWhileMounted), …
DEFENSE_TRACE kind=Batch planned=2 relocated=0 region_empty_after=false scan_allocations=4
SEEDRUN tag=rollback-940105 exit=0
DEFENSE_SEED weights=compaction_rollback width=4g seed_offset=940105 ending=NewFinding { signature: ModelDisagreement { aspect: "这一次整理发布对意图做的事" }, observation: FailureObservation { position: Operation(24), …
DEFENSE_TRACE kind=Batch planned=1 relocated=0 region_empty_after=false scan_allocations=3
SEEDRUN tag=plain-256-960004 exit=0
DEFENSE_SEED weights=compaction width=256 seed_offset=960004 ending=NewFinding { signature: ModelDisagreement { aspect: "这一次整理发布对意图做的事" }, observation: FailureObservation { position: Operation(18), …
DEFENSE_TRACE kind=Batch planned=1 relocated=0 region_empty_after=false scan_allocations=3
```
模型的话（960004 一行，原样）：`活意图 ModelCompactionIntent { issued_txg: ModelCheckpointTxg(29), region: … } 在：一次整理发布要么搬一批、要么删掉意图`，实现一侧：`搬了 0 个单元（其中用户可见树的根 []），发布之后 logged_ops 里有活意图，活意图的 R 按记录不空（或没有活意图）`。
- 计划里的单元是什么：940105 与 940015 的跟踪（`DEFENSE_TRACE_ROLES`）是 `AllocationTreeNodeBelowTheRoot { level: 0, device: 0 / 1, … }`——盘 0、盘 1 上各一片分配记录树的叶，在 R 里、槽号最低，计划取的是 R 里槽号最低的前 K 个，这一批本来就要重写它们（读码推的：R 里的记录释放弄脏了覆盖它们的叶），所以不另搬；R 里另外 2–3 条记录这一批没计划到。
- 不是回退的后果、不是 F2 引入的：960004 是整理那一组比重、盘宽 256 槽、没有回退；第一轮冻结副本的核心（`bin/sweeps-r1core`）在 940015、940105 上同一步同一句不合。4 GiB 盘上整理那一组比重 320 段、4500 多批（外加 108 段小枚举 281 批）一次都没出现，带回退 / unlink / 建 inode 的那一组 150 段里 2 段（§1.3 那四批）。
- 后果：多发一次没搬单元的发布（吃一个根槽和 `B_gc` 一格），下一轮计划就不再含那两片叶，进度不丢；harness 里是随机历史红。**这一形条款没写**（G1/G4 没说照抄意图的发布必须搬至少一个）；F2 自己的读法（「一批 = 搬了单元的那一次」，`crates/singlefs-core/src/mounted_session.rs:2068`）把它划成「不是一批」，于是它既不是签发、不是批、不是删意图——模型只认这三种，判不合是模型照 F2 的读法该红的；要么实现不发它、要么模型认它。
- 同形还有一支没有强制用例：最后一批删意图核不过、照抄再发（「Err(RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {」，`crates/singlefs-core/src/mounted_session.rs:2574`）。「每条分支必须能被测试强制进入。」（`.claude/rules/fs-design.md:122`）：`RegionCheckOfTheIntentDeletion` 只交进了再发那一次（`crates/singlefs-core/src/mounted_session.rs:2603`），头一次不读它；全仓 `grep -rn RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted crates/singlefs-harness/tests` 零命中（只在 `history.rs` 的失败映射里有）。把同一个开关也交给头一次就能强制。
- 替代改法（对 3.3）：
  1. 规划时去掉本次发布顺带重写的单元并补足到 K 个真要搬的——两遍规划，R 里剩下的全是顺带重写的就等于 R 腾空、同批删意图；封住 3.3 与 F2 的全部同形，实现改动大；
  2. 模型加一臂 `(false, true, false)`：认「只含顺带重写节点的批」，实现不动；最小，但与 F2 的 `batches_published` 读法要一致（也记一批或单独记）；
  3. F2 现状：只封 R 被腾空那一支。
  三条里 1 最干净，2 最省；F2 现状是最差（留着随机历史会红）。

## 4　R3 签发那次发布的 `segs0`（F3），连同第一轮 T6 的前一半

结论：**站得住（实现一侧）**，带一条条款张力要用户 / 主 agent 知道：F3 让停机谓词「全局差值」的基线挪低了一格。T6 前一半（`segs0` 写 3310、记账第 11 项 3309）是实现错，F3 改到了。

- 定义句有两处，F3 取的和它们逐字一致：「签发那次发布写下的记账第 11 项（全空聚簇段数，D5（快照 / 空间记账机制） 已定项 4）里 `R` 所在盘的值，只作观测量」（`.claude/kb/decisions/26-后台整理与放置回收.md:63`）；停机谓词也用它：「一条意图完成而全局差值 `全空聚簇段数(现在) − segs0 ≥ 1` 为假就不签下一条」（`.claude/kb/decisions/26-后台整理与放置回收.md:26`）。代码里这一判是「empty_cluster_segments_now > empty_cluster_segments_at_issue」（`crates/singlefs-core/src/compaction_plan.rs:753`），会话调它：「if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {」（`crates/singlefs-core/src/mounted_session.rs:2351`），`completed` 是从盘上 logged_ops 解出来的那一条，所以基线是 F3 写下的值。
- F3 的取法：核「if handed_in.empty_cluster_segments_at_issue == written_by_the_accounting {」（`crates/singlefs-core/src/transaction.rs:7875`），不等就用记账那个数重算再发；记账那个数从这一版自己取：「fn empty_cluster_segments_written_by_the_version(」（`crates/singlefs-core/src/transaction.rs:7767`）。
- 两次算账之间换得干净：头一次算账的结果在 `check_before_persisting` 报错时整个丢掉，分配器换回进来时那一份：「if let Err(refused) = check_before_persisting(&output) {」（`crates/singlefs-core/src/transaction.rs:8521`）后一行 `*allocator = allocator_before_this_publish;`；第二次从同一份分配器、同一个 txg 起算；成功后分配器上的活意图一并换成盘上那一条：「.take_the_intent_written_by_the_issue(handed_in, written);」（`crates/singlefs-core/src/transaction.rs:7727`，定义在 `crates/singlefs-core/src/compaction_plan.rs:503`）。会话每一步开头的 `assert_eq!` 核分配器上的活意图 = 现行版 logged_ops 里那条，拷贝上跑的 ~600 段随机历史（含每一次签发）没有一次断言失败。
- 准入、根槽、`B_gc` 不多算：准入与落点在头一次算账里只在内存里过、不写；根槽与发布计数只在发成之后记一次：「publishes_of_the_run += 1;」（`crates/singlefs-core/src/mounted_session.rs:2060`）。第二次算账不会再不等：产品代码里兜底的是一句 `panic!`（「签发那次照记账换过 segs0」，`crates/singlefs-core/src/transaction.rs:7720`），拷贝上跑的 ~600 段随机历史没有走到它；它是产品路径上的 panic，换成类型化的错会更稳（建议，不算打中）。
- 既有两条用例绿：`an_issue_inside_the_open_segment_writes_the_empty_segments_of_its_accounting`、`an_issue_opening_a_new_cluster_segment_writes_the_empty_segments_of_its_accounting_not_the_count_before_it`（`logs/existing-fix-tests.log`）。
- **条款张力（要知道，不是实现错）**：签发那次发布自己开了新聚簇段时，记账第 11 项比签发之前的记录数少一。F3 之前基线是签发之前的数（净产出 0 的一条意图完成后「现在 > 基线」为假，不再签下一条）；F3 之后基线是记账那个数（同一条净产出 0 的意图「现在 > 基线」为真，再签一条）。探针：
```
$ bin/probes-r2-core --ignored --nocapture --exact defense_probe_arithmetic
PROBE group=f3 name=k-h-boundary net_zero_cycle: yielded_with_the_count_before_the_issue(3310 vs 3310)=false yielded_with_the_accounting_value(3310 vs 3309)=true
```
  两种读法都有字面：D26 的表（63 行）说基线是记账那个数——F3 照它；D26 已定项 1（26 行）要的「产出」按直觉是相对签发之前——F3 之前的值。差一格只在「签发自己开新段」那一支，每趟仍被 `B_gc` 罩着；产品路径今天不起整理（`batch_size: CompactionBatchSize::NotDecidedByTheClauses,`，`crates/singlefs-core/src/compaction_plan.rs:346`），所以只影响测试开关下的整理。
- 推翻条件：随机历史里出现一次签发之后 `segs0` ≠ 记账第 11 项，或第二次算账 panic；或用户 / 主 agent 把基线定为签发之前的数（那时 F3 要退回，D26 的 63 行要改）。
- 替代改法：(a) F3 现状：照字面、两次算账、一句 panic 兜底；(b) 改条款：`segs0` 定义成签发之前的记录口径数，代码退回 F3 之前，不重算、无 panic，K-h 语义最直；与 D26 的 63 行相反，要用户定；(c) 事先算「这次签发会不会开新段」再减一：要先过一遍取落点，等于把算账做两遍还不如 F3 干净。F3 在「照条款字面」这个前提下最好；前提要不要，是 (b) 的问题。

## 5　R4 强制开关（F4），连同第一轮 T2 缺的那一项

结论：**站得住**。S3 末句要的「只供测试的开关」有了，两臂各一条用例强制进入，产品路径碰不到它；两臂与 S3 只差判定那一处。覆盖面有一处限度：开关挂在写入口上，会话 / 随机历史够不着。

- 条款：「只供测试的开关：强制」（`research/prompts/m4-snapshot-r1-main-verification.md:32`，后接「立即释放 / 进 deadlist」两臂）与「每条分支必须能被测试强制进入。」（`.claude/rules/fs-design.md:122`）。T2 缺这一项成立（第一轮 grep 零命中）。
- 平常路径碰不到：写入口新开时的值是「snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,」（`crates/singlefs-core/src/transaction.rs:368`）；`set_snapshot_lifetime_judgement` 全仓只有一处调用，在测试里（「writer.set_snapshot_lifetime_judgement(judgement);」，`crates/singlefs-harness/tests/snapshot_lifetime_judgement_forced_onto_either_arm_by_the_test_only_switch.rs:48`；定义「pub fn set_snapshot_lifetime_judgement(&mut self, judgement: SnapshotLifetimeJudgement) {」在 `crates/singlefs-core/src/transaction.rs:452`）。核心源码里有 10 处 `PoolWriter::new(`（含单测；`grep -rn 'PoolWriter::new(' crates/singlefs-core/src | wc -l` → 10），产品路径上的都不经这个开关。
- 两臂只差判定：`verdict_of` 三臂里 `FollowTheJudgement` 走 `snapshot_lifetime_verdict(...)`，另两臂直接给 `ReleasedIntoTheDeferQueue` / `KeptOnTheDeadlist`（`crates/singlefs-core/src/transaction.rs:5040-5050`）；问判定的仍只有那六类（「pub const fn asks_the_snapshot_lifetime_judgement(identity: TransactionUnit) -> bool {」，`crates/singlefs-core/src/transaction.rs:4976`），其余 14 类照旧进 defer；删快照的批放哪几条不读它（`deadlist_entries_the_destruction_of_a_snapshot_releases` 那一段不经 `verdict_of`）。后继副作用（映射条目留存、deadlist 条目、journal 点名）都从同一份 `DeadlistChangesOfThisPublish` 派生，没有另一条独立判定。
- 既有三条用例绿：`a_new_pool_writer_follows_the_snapshot_lifetime_judgement`、`forcing_the_release_arm_releases_every_replaced_user_visible_unit_under_a_snapshot`、`forcing_the_deadlist_arm_keeps_every_replaced_user_visible_unit_without_any_snapshot`（`logs/existing-fix-tests.log`）。
- 限度：写入口是每次发布现造的，开关只在测试自己造的写入口上有效；`MountedSession` / 执行器 / 随机历史强制不了（`HistoryExecution` 没有这一格）。判决 F4 要的「两臂各一条用例强制进入」满足；「在随机历史里强制」今天没有。
- 推翻条件：产品路径出现一处 `set_snapshot_lifetime_judgement` 调用，或 `verdict_of` 的非强制臂不再等于原判定。
- 替代改法：开关挂在分配器上（同 `CompactionSwitches`、`deletion_switches`，会话造写入口时抄过去）——会话 / 随机历史也能强制，代价是分配器多一个字段、拷贝一份；两种在 F4 要的点上一样好，分配器那一种覆盖面更大。先例两边都有（`inode_number_watermark_source` 在写入口上，`container_tree_capacities` 经分配器抄到写入口上），F4 取前者不是错。

## 6　R5 空文件集合上的快照（F5），连同第一轮 T1

结论：**站得住**。核心、RefFS、模型三处在「unlink 删空之后的那一版」上同口径放开；探针证实放开前后实现与模型的对拍从红到绿。注意：RefFS 与模型里「没有文件」比核心里「有树而文件集合为空」宽，宽的那一半（新池还没有第一个文件时建快照）核心拒、harness 翻译层碰不到。

- 核心：有树而文件集合为空的一版放行：「UserChange::CreateSnapshot => emptied_version,」（`crates/singlefs-core/src/mounted_session.rs:1412`）；树表 0 条的新池仍拒：`PoolVersion::WithoutFile(_)` 两处（`crates/singlefs-core/src/mounted_session.rs:1206`、`:1368`）一律 `NoFileVersionToChange`。
- 模型：建快照只要现行版本有树：「if !current.contents.has_trees() {」（`crates/singlefs-harness/src/model.rs:3445`），上限 10 照数；RefFS：不再按「没有文件」拒，快照抄 `state_so_far`：「captured: state_so_far.clone(),」（`crates/singlefs-reference-filesystem/src/lib.rs:607`）。
- 探针（`f5_scenarios`，6 格，起点第一个文件，先 unlink 再建快照；每一步过池级 checker、理想模型逐步比）：
```
$ bin/probes-r2-core 的 f5 组                      → scenarios=6 not_completed=0 reds=0
$ bin/probes-r1core 的 f5 组（第一轮核心）          → scenarios=6 not_completed=6 reds=0   # 六格都是 ModelDisagreement「模型说该成、实现拒了」，实现答复 UserChangeRefused::NoFileVersionToChange
```
  六格：建快照 → 读快照 → 冷启动 → 读 → 可写挂载 → 读 → 建 inode → 读；建快照后回退（执行器判 `NotApplicable(CurrentVersionWithoutFile)`，见下）；可写挂载后删快照、冷启动、再建；建快照后抛弃最新根的恢复、读、再建；连建 12 个（第 11、12 个被拒 `Refused { member: "PublishError::SnapshotCreationRefused" }`，上限 10 数它）；对照（有文件时建快照再 unlink，读快照读出建它那一刻的文件）。逐行 `model/out/f5-r2-core.txt`、`model/out/f5-r1-core.txt`。
- 回退拒：执行器在没有文件的现行版本上不调回退（`NotApplicable(CurrentVersionWithoutFile)`）；核心入口自己拒：「if snapshots_in_the_current_version > 0 {」（`crates/singlefs-core/src/mount.rs:8288`）与目标版同判「if snapshots_in_the_target_version > 0 {」（`crates/singlefs-core/src/mount.rs:8367`），数的是树表里的快照条目，空文件集合上的快照也算。
- 建快照那次发布崩在中途：harness 里只有「抛弃最新根的恢复」那一格（`f5-crash-abandoning-the-snapshot-creation-root`，绿）；RefFS 一侧见 §7 的翻译探针（崩在建快照那一步里读回同一份内容：允许集合 `{} ; {S}`）。真正的崩溃点枚举归 checker 档，没跑。
- 限度（要知道）：RefFS 的 `NoFile` 在新池第一个文件之前也成立，那时建快照 RefFS 收、核心拒（`WithoutFile`）；翻译层遇到被拒的操作记「没发生」，所以不矛盾，但 RefFS 新增的单测「建快照、再写第一个文件、读快照读出没有文件」（`a_snapshot_created_while_there_is_no_file_reads_no_file_after_the_first_file_is_written`）描述的序列核心做不出来；翻译层遇 `Unlink` 就停：「TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:404`），所以「空文件集合上的快照」从没进过 RefFS 的判定。F5 在 RefFS 一侧是文字上的放开，没有取样点。
- `history.rs` 里 `HistoryOperation::CreateSnapshot` 的文档注释还写着「现行那一版有树时调（有树而文件集合为空的一版也调：会话拒成 `NoFileVersionToChange`）。」（`crates/singlefs-harness/src/history.rs:543`），F5 之后过时（注释，不影响行为）。
- 推翻条件：入库装置上这 6 格任一格红；或空文件集合上建快照之后冷启动 / 可写挂载读不出快照；或 12 个里第 11 个被收。
- 替代改法：(a) F5 现状（三处放开）；(b) 只放开核心、RefFS 与模型维持拒——RefFS 拒「核心收的操作」会让翻译层停（弃权），更差；(c) 把 RefFS 的「没有文件」拆成「新池」「删空」两态——最准，但 RefFS 没有 unlink，拆了也没人用。F5 的取法在这三个里最小而够用。

## 7　R6 RefFS 的快照允许集合（F6）与第一轮 T11（连同 F7）

### 7.1 F6：站得住
结论：**站得住**。恢复之后允许的快照集合改成「允许集合里与读出内容相同的每个状态那一刻的快照集合」，翻译层探针上建快照 / 删快照两种结局都收、别的内容判红；没有找到核心层能落到而 RefFS 判不允许、也没有找到不合法被放进来的形状。实现员自补的排除读法在被判的范围内是对的。

- 代码：「pub fn allowed_snapshot_sets(&self) -> Vec<Snapshots> {」（`crates/singlefs-reference-filesystem/src/lib.rs:403`）；读快照的判与收窄「pub fn judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(」（`crates/singlefs-reference-filesystem/src/lib.rs:430`）；恢复之后取哪些下标：「let indices_the_snapshot_sets_are_taken_at =」（`crates/singlefs-reference-filesystem/src/lib.rs:738`）；自补的排除：「.filter(|(_, snapshots)| precondition.holds_in(snapshots))」（`crates/singlefs-reference-filesystem/src/lib.rs:556`）。
- 新公开接口在 harness 翻译里的用法一共两处：「let allowed_reads = model.allowed_reads_of_the_snapshot(*name);」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:645`）与「.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(*name, read_state)」（`crates/singlefs-harness/src/reference_filesystem_translation.rs:648`）；`allowed_snapshot_sets()` 在 RefFS 自己的单测之外没有调用方；旧接口 `read_snapshot` / `snapshots()` 在全仓已无引用（`grep -rn 'read_snapshot(\|\.snapshots()' crates | grep -v reference-filesystem/src/lib.rs` 只命中 `crates/singlefs-harness/src/history.rs` 里另一个函数 `apply_read_snapshot` 的两行）。
- RefFS 单测 30 个全绿（`cargo test -p singlefs-reference-filesystem --lib`：`test result: ok. 30 passed`，`logs/reffs-tests.log`）。
- 翻译层探针（`defense_probe_reference_filesystem_snapshot_sets`，两段历史、每一步里崩、读回第一个文件的内容或覆盖写的内容，判票与重启之后 RefFS 允许的快照集合；`S` 是快照，`file3000B` 是它抄的内容）：
```
create-overwrite-read-destroy（建快照 → 覆盖写 → 读快照 → 删快照）
  moment=inside-step-0 readback=first-file  green=true   sets_after=[{} ; {65549:file3000B}]      # 崩在建快照那一步里：有与没有都收
  moment=inside-step-0 readback=overwrite   green=false  # 别的内容仍判红
  moment=inside-step-1 readback=first-file/overwrite green=true  sets_after=[{65549:file3000B}]    # 崩在覆盖写里：快照确定在
  moment=inside-step-3 readback=overwrite   green=true   sets_after=[{65549:file3000B} ; {}]      # 崩在删快照那一步里：还在与已删都收
  moment=after-every-step readback=overwrite green=true  sets_after=[{}]                         # 都返回过之后：删了
two-snapshots-then-destroy-the-first
  moment=inside-step-2 readback=first-file  green=true   sets_after=[{65549:file3000B,65551:file3000B} ; {65551:file3000B}]
```
  逐行 `model/out/f6-reference-filesystem-snapshot-sets.txt`（20 行）。
- 自补的「实现做了的操作说明那一刻的快照集合满足什么」：它拿「前面的操作都做成了」的快照集合判前提（`submit` 里的 `snapshot_set_of_each_allowed_start_at`）。这对被判的范围是对的：写了之后报错（两可）的操作后面再有操作、再崩溃，`allowed_set_for_a_crash_in` 直接拒成「OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion」（`crates/singlefs-reference-filesystem/src/lib.rs:657`），翻译层记弃权，不判；所以「前面的操作都做成了」只在不会被判错的序列上用。推翻它要一个：操作 i 写了之后报错、操作 i+1 被实现接受、崩溃后翻译层仍判——今天翻译层在那一步停，造不出。
- 与 F6 同源但不是 F6 引入的：抛弃最新根的恢复读出的内容不在允许集合里时快照集合退回 S_0 那一刻的（「读出的内容一个都对不上（抛弃最新根的那次恢复不判读出的内容）时照取 S_0 那一刻的。」，`crates/singlefs-reference-filesystem/src/lib.rs:724`），可能比盘上真实的新或旧；后果只是之后的前提判不过时翻译层弃权（覆盖变少），不会误红。改动前也是这个读法。
- 推翻条件：入库装置上上面任一行变；或造出一段「实现在崩溃后真读到某个快照集合、RefFS 的 `allowed_snapshot_sets()` 里没有它」的历史（要崩溃点枚举，checker 档，没跑）。
- 替代改法：(a) F6 现状：按全部对上的状态取并集；(b) 取下标最小的那个——对称的错，会排除「建快照落了盘」；(c) 把快照集合和文件内容一起记进允许状态（每个 S_j 带自己的快照集合）——最准，改动面是 `AllowedState` 的公开形态，翻译层要改；F6 的并集在被判的范围内给出同一组判票，改动小；(c) 在「读快照之后收窄」那一步更直，不更正确。

### 7.2 第一轮 T11：坏在测试，F7 够得着
结论：**站得住**。T11 攻方那一格是判别力缺口（变异表第 4346 行「「非空」不看快照条目」原样绿），坏在 harness 模型的单测，不是实现；RefFS 与核心层口径不同那一半（正推）是 §7.1 的 F6。
- F7 的用例与变异行：「fn two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots(」（`crates/singlefs-harness/src/model.rs:7010`），变异表第 4346 行现在点名它：`crates/mutations.tsv` 第 4346 行末两列是 `-p singlefs-harness --lib -- model::tests::two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots --exact`。
- 探针：原样跑绿；按第 4346 行的替换文改坏 `model.rs`（`model/patches/mutation-4346-snapshot-roots-ignored.diff`）重编再跑红，断言是 `assertion left != right failed: 快照抄的根不同：用户可见状态不同`（「快照抄的根不同：用户可见状态不同」，`crates/singlefs-harness/src/model.rs:7044`）：
```
$ cargo test -p singlefs-harness --lib -- model::tests::two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots --exact     # 原样
test model::tests::two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 232 filtered out; finished in 0.00s
$ 同上，model.rs 按第 4346 行改坏
test model::tests::two_versions_with_one_snapshot_each_differ_in_the_user_visible_state_when_the_snapshots_copy_different_roots ... FAILED
assertion `left != right` failed: 快照抄的根不同：用户可见状态不同
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 232 filtered out; finished in 0.00s
```
  改完已换回（`cmp` 与冻结副本相同）。
- 推翻条件：入库装置上原样红，或按第 4346 行改坏仍绿。
- 替代改法：把去重的「4 个不同状态」改成按快照条目的内容去重——不是这一格的问题（缺口在测试）；F7 取的补用例是最小改法。

## 8　R7 第一轮交下来的开着的形状，和这条腿顺带撞到的一处

| 形状 | 取样点造出了吗 | 结果 |
|---|---|---|
| 计划里的单元都被这次发布顺带重写、一个没搬而 R 还不空 | **造出了** | §3.3：三个种子（回退那一组比重 940015、940105；整理那一组比重盘宽 256 槽 960004），模型判不合；F2 不覆盖 |
| 多出来的重写越过 `ckpt_cost` | 没造出（要几十万条映射） | §1.5：默认两块 4 GiB 盘上算术罩得住（叶至多 721 片 < 1058），更大的池上没有上界论证 |
| 零个全空段 | 没造出 | 小盘宽 240 / 256 槽整理那一组比重各 62 段（`DEFENSE_WIDTH=240`、`256`，`model/tools/sweep.sh`）：拒绝全是空间准入先拒（两档合计 `UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::SpaceAdmissionRefused)` 856 次，`PublishError::ContentExceedsDataUnit` 2 次），合取 7（`EmptyClusterSegmentsBelowTheFloor`）0 次；合取 7 的拒绝只在既有用例里用合成的分配器造出（`an_overwrite_on_a_pool_without_an_empty_cluster_segment_is_refused_by_the_empty_cluster_segments_floor_by_default`，探针里再跑绿）。零个全空段的池再叠整理，要分配器与盘上记录一致，合成的做不到 |
| 合法镜像触发 `TwoRecordsMovableUnderOneMappingKey` | 造不出，理由如下 | 扫描拒的条件是同一块盘上两条起点不同的记录都判可搬、头里是同一把 key（「return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });」，`crates/singlefs-core/src/region_scan.rs:261`）。一把 key 的映射条目两条位置项必须在两块不同的盘上：「if locations[0].device == locations[1].device {」（`crates/singlefs-core/src/transaction.rs:3899`）拒成 `MappingEntryLocationsOnTheSameDevice`；位置项是两盘同槽的一个落点。同一块盘上另一条判可搬的记录，除非映射指着它，会被判不一致 / 已死；回退留在已分配的坏校验和副本（`release_leaving_the_record_allocated_on`）不在映射里，也不会判可搬。F1 之后回退把条目换回的是同一个 key 的另一个落点，旧落点那条记录是已释放（探针 F1 组每一步过 checker 的 I-3.15） |

### 8.1 顺带撞到的一处：回退那一组比重上「抬 F 的上限」对拍不合（不是这一批修补引起的）
- 种子（回退那一组比重，偏移 950093，第 18 步 `RaiseRollbackFloor`）：`ModelDisagreement { aspect: "抬 F 的上限" }`，模型 `上限 Some(9)（D16（发布语义） 已定项 1）`，实现 `实现报上限 11`（`model/out/seeds.txt` 里 `rollback-950093` 那一段）。前 17 步含三次回退（第 2、9、14 步）、十趟整理（第 13 步一趟 16 批 32 个单元，第 14 步回退到 txg 10，释放 151 个用户可见单元、复活 8 个）。
- 不能归给修补批：第一轮冻结副本的核心（`bin/sweeps-r1core`，装置与 checker 是 r2 的）同种子同一步同一句（`logs/seed-r1core-950093.log`）。也没证明与修补批无关：装置那一侧（`model.rs` 的去重、F5/F7 改过）同是 r2 的。要分辨是模型错还是实现错，要在入库装置上用第一轮冻结副本整套（装置 + 核心）跑一遍这个种子；这条腿没做。
- 这一格不在 R1–R8 里；写在这里是因为它出现在回退那一组比重上，主 agent 判是否算第二轮打中。

## 9　不改代码的三处判定：T7、T8、T10

### 9.1 T7（R 排他与合取 7，用户定维持）：站得住
- 后果的描述都对：产品路径上整理不起跑，批大小产品值 `batch_size: CompactionBatchSize::NotDecidedByTheClauses,`（`crates/singlefs-core/src/compaction_plan.rs:346`），水位恒 0：「第一版三条水位恒 0、0 = 关」（`crates/singlefs-core/src/compaction_plan.rs:257`）；`N₀` 第一版取 1：「pub const EMPTY_CLUSTER_SEGMENTS_FLOOR_OF_THE_FIRST_VERSION: u64 = 1;」（`crates/singlefs-core/src/compaction_plan.rs:302`），条款原文「`N₀` 第一版取 1」与「只拦前台写入类发布」同在一行（`.claude/kb/decisions/03-空间分配.md:146`）。
- 前台写入类发布的范围：「let writes_in_the_foreground =」那一判含写文件内容、建 inode、建快照（`crates/singlefs-core/src/transaction.rs:8637`），所以空文件集合上建快照（F5）也受它管。拒绝不推抬 F：「PublishError::EmptyClusterSegmentsBelowTheFloor(_)」（`crates/singlefs-core/src/mounted_session.rs:1028`）在 `refusal_is_short_of_space` 里列在不是空间不够的那一组，上面两行注释写了理由（抬 F 回收 defer 里的槽不改全空段数）。
- 探针：既有用例在冻结副本拷贝上绿：`test an_overwrite_on_a_pool_without_an_empty_cluster_segment_is_refused_by_the_empty_cluster_segments_floor_by_default ... ok`（`logs/t7-conjunct7.log`），带着 `N₀` = 1 与两块盘各 0 个全空段。
- 出路：删除类发布免（unlink 与删快照；`Exempt` 臂），所以这一阶段「只有一个文件」的池上，用户的出路是删文件或删快照，不是写（删光文件之后写文件内容本来就拒，只放建 inode、建快照与删快照）；这一条用户定了「维持，记后果」，依据条款字面（`.claude/kb/decisions/03-空间分配.md:146`）成立。
- 推翻条件：条款把 `N₀` 改成 0 或让整理在产品路径起跑；或造出一个池，删除类发布也被合取 7 拒（不可能：`Exempt` 臂不读它）。
- 替代：改 `N₀` 为 0（关掉）——违 D3 登记表第 7 行；产品路径起整理——等 E10 的批大小与水位值；用户已选维持。

### 9.2 T8（discard 调用者，少发只记欠账）：站得住
- 少发的机理读码确认：分配器交出要发的段时就把它从「回收过、没发过」集合里摘掉：「segments_to_discard.push(self.segment_start_slot(segment_index));」（`crates/singlefs-core/src/allocator.rs:830`），会话随后才扣下装着孤儿头的段：「.withheld_holding_unswept_orphan_heads」（`crates/singlefs-core/src/mounted_session.rs:1794`）、块设备报错记失败：「Err(_cause) => discards.failed.push((device_identity, segment_start)),」（`crates/singlefs-core/src/mounted_session.rs:1821`）；集合只在回收时加：「self.segments_reclaimed_not_yet_discarded.insert(segment);」（`crates/singlefs-core/src/allocator.rs:791`）。所以扣下 / 报错的段要等段里再有槽被回收才会再交出来。
- 条款不要求补发：G5 的字面只管发的时机与粒度：「按 64 槽对齐的整段发」（`research/prompts/m4-gc-r1-main-verification.md:36`）；分配器的文档把集合的口径写明了：「只看回收时记下的段，不扫单元区」（`crates/singlefs-core/src/allocator.rs:820`）；盘不知道段可回收只是 FTL 少一个提示，不碰一致性。
- 没造出 discard 打到仍被引用的段：谓词要求段里没有未释放记录、没有隔离、没有抬 F 扣住（`segment_is_reusable_as_a_whole`，`crates/singlefs-core/src/allocator.rs:808`）。拷贝上整理那一组比重 27 段、40 步、每一步之后的镜像都过池级 checker（`model/tools/sweep.sh bin/sweeps-r2-core-final compaction 980000 1 31 allchk all 40`）：`segments=27 not_completed=0 reds=0`，728 批；被引用的单元若被 discard 抹了，I-2.1 / I-1.11 会红。
- 推翻条件：造出一个池，discard 已发给某段而该段里某一版根还引用着的单元（checker 档的设备日志判器，I-7.4）；或补发的代价比欠账小到该做（把扣下 / 报错的段放回集合是一行，但 `Unsupported` 的设备会每次重试）。
- 替代：扣下的段放回集合——一行，对「装着孤儿头」这一支有用，对 `Unsupported` 要区分；主 agent 记欠账是够用的。

### 9.3 T10（坏 logged_ops 条目当损坏，拒可写挂载）：站得住
- 依据条款的原文：D8 已定项 5 的「不挡挂载」说的是**提示类意图不必续做到底**：「**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载」（`.claude/kb/decisions/08-核心索引结构.md:108`）。它对盘上条目字节本身坏了没有话；G1 只定「R 的解码拒不对齐的值」（`research/prompts/m4-gc-r1-main-verification.md:32`），怎么拒写在恢复路径的注释里：「盘上读来的坏条目当损坏处置，可写挂载拒」（`crates/singlefs-core/src/recovery.rs:2968`）。判决 T10 一行引的就是这几处，引得对（`research/prompts/m4-segment-code-r1-main-verification.md:32`）。
- 探针（`defense_t10_a_bad_logged_operations_entry_through_each_read_path`，沿用既有用例的造坏法：把意图的 R 起始槽改成不对齐的 50241、重封节点、校验和补到根槽）：
```
PROBE group=t10 name=writable-mount refused=Recovery(UnitMalformed { what: "logged_ops 根里有条目过不了字段表" })
PROBE group=t10 name=cold-recover outcome="FileRead root=(InstanceGeneration(2), CheckpointTxg(6)) bytes=3000" effective_root=Some((InstanceGeneration(2), CheckpointTxg(6)))
PROBE group=t10 name=cold-recover-of-the-unmodified-pool outcome="FileRead root=(InstanceGeneration(2), CheckpointTxg(6)) bytes=3000"
```
  可写挂载在任何写之前拒（helper 断言盘逐字节不变），冷启动 `recover` 读同一条根、读出同样的 3000 字节：数据不丢，只是不能写。
- 读成「不挡挂载 = 丢掉坏条目照挂」的替代站不稳（我的论证，条款没写）：logged_ops 每次发布都照抄，丢坏条目要在树里改写它；整条解不出就不知道它是不是一条整理意图、R 排他靠什么装，整理可能把单元搬进它声明的 R 里。读成「当损坏」同处理别的坏结构，一致。
- 推翻条件：用户 / 条款把「提示类的坏条目」定成可丢；或冷启动路径也该拒（今天收）。
- 替代：把拒从「可写挂载拒」放宽到「只读挂载收、可写拒并报怎么修」——今天就是这样，缺的是修复入口，不是判定。

## 10　判决条目表（辩方判定）

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| T4 打中（判据：I-3.5 红） | 站得住：坏在池级 checker 的标签取法，实现与模型一致 | §2：同一实现，第一轮 `walk.rs` 红、r2 `walk.rs` 绿（5 步历史 + 四个种子） |
| F9 够得着 T4 | 站得住 | §2：两形（标签 None、标签非 None）都收；六个必红 / 已知坏镜像测试目标共 309 个全绿 |
| T5 打中（实现：回退的映射） | 站得住：坏在实现 | §1：F1 变异（函数改成返回空表）与第一轮核心上同一组探针 49 格红 24 格、108 条池级 checker 违例；r2 核心 0 格 |
| F1 够得着 T5、仍守 G3 | 站得住（部分） | §1.1–1.2：D23 字面「不从 R_old 那棵账载入」；判决 F1 字面与它冲突；成本罩得住只是算术（§1.5） |
| T6 前半（`segs0` 差一，实现） | 站得住 | §4：F3 之后 `segs0` = 记账第 11 项，既有用例两条绿 |
| F3 | 站得住，带条款张力 | §4：F3 把停机谓词「全局差值」的基线挪低一格（`3310 vs 3309`）；要用户 / 主 agent 知道 |
| T6 后半（腾空 R 的那一批不删意图，实现 + 模型） | F2 够得着判决点名的那一形，**够不着同形另一支** | §3.3：三个种子造出「没搬、照抄、R 不空」，模型判不合；回退那一组比重上与整理那一组比重（小盘）上都有；第一轮核心上同样 |
| T2 缺的强制开关 → F4 | 站得住 | §5：产品路径一处不调；两臂各一条用例；覆盖面限于直接造写入口的测试 |
| T1 → F5 | 站得住 | §6：6 格探针从红到绿；上限 10 数它；RefFS 一侧是文字放开，翻译层没有取样点 |
| T11 打中（测试：变异 4346 存活）→ F7 | 站得住：坏在 harness 模型的单测 | §7.2：原样绿、按变异行改坏红 |
| T11 口径（RefFS 取下标最大）→ F6 | 站得住 | §7.1：翻译层探针 20 行；自补的排除读法在被判范围内对 |
| T7 维持合取 7 | 站得住 | §9.1：条款字面 + 既有用例；出路是删除类发布 |
| T8 少发 discard 只记欠账 | 站得住 | §9.2：G5 字面不要求补发；27 段每步过 checker 0 红；补发是一行但要区分 `Unsupported` |
| T10 坏条目当损坏、拒可写挂载 | 站得住 | §9.3：可写挂载拒、冷启动读得出同一条根；引的条款原文对 |
| R7 四个开着的形状 | 一个造出（§3.3），一个算术（§1.5），两个没造出并写了理由（§8） | §8 |
| 顺带：回退那一组比重上「抬 F 的上限」对拍不合 | 报出，不归修补批 | §8.1：种子 950093；第一轮核心上同样 |

### 10.1 每条修补是不是同样打中别的替代改法（汇总）
| 修补 | 别的改法 | 比较 |
|---|---|---|
| F1 | 整棵取目标版 / 全标重写 / 反向整理 / 拒绝跨整理回退 | F1 最小；全标重写是最稳的退路，代价是多重写（§1.6） |
| F2 | 规划时去掉顺带重写的单元并补足 / 模型加一臂 | 两者都比 F2 现状好（现状留着随机历史红）；前者封得最干净（§3.3） |
| F3 | 改条款定义 / 事先算 | 取决于基线要不要相对签发之前；用户定（§4） |
| F4 | 开关挂分配器 | 覆盖面更大，同样满足 F4 要的点（§5） |
| F5 | 只放核心 / 拆 RefFS 两态 | F5 现状最小而够用（§6） |
| F6 | 取最小下标 / 允许状态带快照集合 | 取最小下标是对称的错；带快照集合更直但改公开形态（§7.1） |
| F9 | 按 R 自己的树根重算 | 等价（§2） |

### 10.2 替被判出局一方辩护
- 出局的一方一：判决 F1 的字面「中央映射树取目标那一版的」。D23 已定项 14 的字面支持的是现算、不载入（`.claude/kb/decisions/23-journal的角色与格式.md:378`）；它出局的理由是字面与条款冲突，不是想法本身错——目标版自己的映射条目在 F1 里一条不少地用上了（用户可见单元那部分）。
- 出局的一方二：G3 里不把映射当唯一入口、继续让指针（提示）承重的写法（判决 G3 一行「乙」之外的那一边，`research/prompts/m4-gc-r1-main-verification.md:34`）。T5 这一格只出在「映射是唯一权威」的写法上：回退取目标版的指针，映射却是现行版的，两份落点对不上；提示承重的写法回退时直接带着目标版的指针，这一格不会出（攻方自己也这么说：「甲（提示为准）下回退取目标版的树就带着目标版的提示，不会出这一格」，`research/prompts/m4-segment-code-r1-opus-output.md:142`）。F1 补上之后这一格关了，但补它的代价（回退发布路径多一条比较与重写）是出局的一方不用付的。它被判出局的理由是别的（搬走单元后映射与快照读到旧落点，G3 一行），这一条不翻案，只记：「映射唯一权威」每多一条写路径，就多一处要记得改的地方，T5 是第一处漏掉的。
- 出局的一方三：攻方 T8 的「少发要补」。§9.2：补发改动小，但对 `Unsupported` 的设备会每次重试，且条款不要求；记欠账够。

## 11　没做什么
- 不替主 agent 撤销或维持判决，只交复核结果。
- 没跑 checker 档（崩溃点重放、崩溃枚举、QEMU、GPU、形式化验证、变异整表）、没起门禁；所有数是冻结副本拷贝上 debug 档的数，不是入库装置上的数。
- F1 的 `ckpt_cost`：只做了算术，没造出几十万条映射的池。
- F2 同形另一支：给出种子与跟踪，没缩成最短历史。
- 暂停的意图再起、零个全空段上的整理、discard 打到仍被引用的段：没造出（理由在 §8、§9.2）。
- T4 的 S8 (b) 款读法 / 阶段同步写回不在这里；§8.1 的抬 F 上限对拍不合只报出、没查模型错还是实现错。
- 没复核 Verus 的 78 verified（本机无 verus）。
- 随机历史里 `rb1` 那一批 63 段被前台时限截掉，没算。
- 没读攻方第二轮报告（禁读）。

## 12　要主 agent 跑的命令
复跑（编三个探针目标 ×2 套代码、跑 T4 / F1 ×2 / F1 小池 / F5 / F6 / 算术 / T10 / 四个种子 / T11 两遍，预计 15 到 25 分钟，debug 档；工作目录仓根）：
```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r2-sonnet-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev \
  --frozen ~/.local/share/singlefs/work/m4-segment-code-r2/repo \
  --work <新的空草稿目录>
```
环境变量：`RERUN_THREADS`（默认 31，派发提示的线程上限）；`RERUN_TARGET_SEED=<一份已编过的 target 目录>` 给了就拷来当编译目录种子（省首次全量编译）；`RERUN_STOP_AFTER=build` 或 `t4` 只跑到那一步。跑完看末尾：十行 `name=rerun_compare file=<名>.txt identical=true` 与一行 `name=rerun_total same=10 differ=0`。模型目录里还有三份别的代码上量出来的对照（`out/f1-r1-core.txt`、`out/f5-r1-core.txt`、`out/t4-r1-checker.txt`，另有 `t4-snapshot-seeds-*.txt`、`f1small-r2-core-with-f1-neutered.txt`）：它们要第一轮冻结副本的核心或 `walk.rs`，不进复跑，当单次量的读。

## 13　模型目录

`research/prompts/m4-segment-code-r2-sonnet-model/`：`probes/` 三份测试源（放进拷贝的 `crates/singlefs-harness/tests/`）、`patches/` 三份补丁（只打印的跟踪行、F1 变异、变异行 4346）、`tools/`（`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来、只改 `rerun_steps` 与它用的几个函数；`probe_group.sh`、`sweep.sh`、`seed_runs.sh`）、`out/`（单次量与复跑要比的输出）。`out/f1-r1-core.txt` 与 `out/f1-r2-core-with-f1-neutered.txt` 逐字节相同（同一个 sha256），`out/f1small-r2-core.txt` 与 `out/f1small-r2-core-with-f1-neutered.txt` 同。各文件 sha256（同目录 `SHA256SUMS`）：

```
c53645443b1dad869d41b9b5b4052285b1fd2e3f9d664c80573b4c9fbb17b76f  ./out/arithmetic.txt
3955116cf1cd9fa839a692c883f9bc86468b2aec62fee8a15ea183d7a87ae787  ./out/f1-r1-core.txt
bd8d14018b4153bd286f7f0e3f745c4b401500e9197d1ba7d436a16d4cbff671  ./out/f1-r2-core.txt
3955116cf1cd9fa839a692c883f9bc86468b2aec62fee8a15ea183d7a87ae787  ./out/f1-r2-core-with-f1-neutered.txt
ee9bbe0d95e0f7221936cfc543c8fcbaff04916ec7d78a0e7572172e6b5dbdc0  ./out/f1small-r2-core.txt
ee9bbe0d95e0f7221936cfc543c8fcbaff04916ec7d78a0e7572172e6b5dbdc0  ./out/f1small-r2-core-with-f1-neutered.txt
447130b6826f0b21e37d1cf159fec72946b44b4fc5f40ca36d77922266a85c0e  ./out/f5-r1-core.txt
b8c9285b632b5159caae4a2dd94e61e13a06371b1ddf785afd81c37526628577  ./out/f5-r2-core.txt
7855694b349cb8a079e89ecfc518f000ce902a620e42ee3df3fa69f5085d5cd2  ./out/f6-reference-filesystem-snapshot-sets.txt
e110ca6cff93700bd59fb7f1b89efe6dd6b2f22dcd6f7ac6aa5dfbd0037893c0  ./out/seeds.txt
1a6b9327b5ed5eeca02d24660df5fa10b5de8199b6a86fe75d259394e6872245  ./out/t10.txt
209fe0f974a3407841a2fb527e4e770a736570ccb3c5b77ff893355a2ceae37b  ./out/t11.txt
3eeaefcaecbdffdc2885de67b6d8c7f1e3d6be06e8019cf5f312db35eeb1af6e  ./out/t4-r1-checker.txt
4713145ce4911e50ee5d49aee040fae368100bfa0adf8a1eb70eb6394a90285b  ./out/t4-r2-checker.txt
98bb78a1db14243ee77aca707b54e2e948eff9b19447dabd5e1c7d68cfcec36e  ./out/t4-snapshot-seeds-r1-checker.txt
4a6df2cf21d338ac4f85daa9c65766682dc5d46948113279ec04763ef58562ef  ./out/t4-snapshot-seeds-r2-checker.txt
2cce5fc5e2055a48c3f30f84b1c751976d68299a0fbf7997aeb6e93a1641d8c9  ./patches/defense-trace-zero-move-batches.diff
f2d7c3ec0b4f7cf9a579994ac16bd9b2cb4f4696094f91d51603bea00b64bd83  ./patches/mutation-4346-snapshot-roots-ignored.diff
c5480eff17fdd4b477750c03ae551f1950c25f8592a614578e2809513eaddb6f  ./patches/mutation-f1-no-mapping-keys.diff
195bbb2bc1932762b96b34ebe89866f6f7c869d6bc21cb6caa5ec7c09b404bc6  ./probes/defense_probes.rs
6dab0154014c6e2680efa07043558e5050099e7ae9b9a47e54e87b5da3c6c683  ./probes/defense_random_sweeps.rs
314474aefd6e5fbdb5f5130ae6c1b1ea10f89fac4431b39b0e3c143189ccc62c  ./probes/defense_t10_bad_logged_operations_entry.rs
ab1d7aa9a3e9c8af4aa480a264d4e55ee2f1758740e5523cec44770b480c8608  ./tools/probe_group.sh
70662180cd0230261495a49a9db8c380c2803daa732e5a9fe9e7c042da0f1c0f  ./tools/rerun.sh
bee498ee6d8ab185244d73b3ec1b22e0ca4b4c0f6c08d99c63c0992ea4d69db3  ./tools/seed_runs.sh
32a9cacd861b43017e8594485aef0fa5d0112bd444186971bd9f0ed4d2616515  ./tools/sweep.sh
```

## 14　草稿目录清理
交回前删掉的（删前 `du -sh`）：
- `~/.local/share/singlefs/work/m4-segment-code-r2-defense/target`：7.4G（编译目录，种子拷自 `m4-segment-code-r1-main-rerun-forward/target`，只读拷、原目录没动）；
- `~/.local/share/singlefs/work/m4-segment-code-r2-defense/repo`：29M（冻结副本的拷贝，探针与只打印的跟踪补丁加在它上面）；
- `~/.local/share/singlefs/work/m4-segment-code-r2-defense/repo-r1`：28M（第一轮冻结副本的拷贝 + r2 的装置 / checker，量 r1 核心的对照）；
- `~/.local/share/singlefs/work/m4-segment-code-r2-defense/bin`：1.9G（各份探针二进制：r2 核心、F1 变异、r1 核心、第一轮 `walk.rs` 各一）；
- `~/.local/share/singlefs/work/m4-segment-code-r2-defense-smoke`：8.5G（`rerun.sh` 冒烟用的工作目录，`RERUN_STOP_AFTER=t4` 那一趟：`name=rerun_compare file=t4-r2-checker.txt identical=true`）。

留下的：`~/.local/share/singlefs/work/m4-segment-code-r2-defense/logs`（12M，报告里引的各份日志）、`model`（草稿里的模型目录原本，已原样进 `research/prompts/m4-segment-code-r2-sonnet-model/`，补丁头的时间戳在入库那份里去掉了）、`sha-check.txt`、`build1.log`。冻结副本 `~/.local/share/singlefs/work/m4-segment-code-r2/repo` 没写过。
