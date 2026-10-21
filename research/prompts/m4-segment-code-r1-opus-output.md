# m4-segment-code-r1 攻方腿（Opus）报告（2026-10-16）

立场：正文 `research/prompts/_m4-segment-code-r1-body.md` 第四节攻方那一行。只在冻结副本的拷贝上造反例；拷贝里的数是副本上的数，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| T1 建快照那次发布 | 没打中 | 事务号 + 1、续码 14 / 15、配对判定在没出问题的 277 段随机历史里都没红 | 随机历史量的，撞号推的 |
| T2 释放分流与 deadlist | 没打中 | `birth == S.txg` 一格是空的；hi 取这一批 txg 在串行之下与写意图 txg 同一组 | 推的 + 随机历史 |
| T3 删快照意图、批与续做 | 没打中 | 共用批大小、删空之后删快照、三处销毁中都没找到翻面 | 推的 + 随机历史 |
| T4 回退拒与读快照 | **打中（判据一侧）** | 无快照时回退、之后建快照、快照下覆盖写：池级 checker 在合法镜像上红 I-3.5；S8「第一版拒回退，不出现」的前提不成立 | 量的（5 步历史）；checker 根因推的 |
| T5 一批搬迁的事务形态 | **打中** | 整理搬过之后回退：回退那一版映射指着它自己释放的目的落点，之后覆盖写一律 `ReleaseTargetAlreadyReleased`，I-3.9 / I-3.15 红 | 量的（16 格里 6 格中）；根因推的 |
| T6 整理意图上盘与挂载续做 | **打中（轻）** | R 被「本来就重写」的节点腾空的那一批不删意图，多发一次只删意图的发布，模型判不合 | 量的；根因推的 |
| T7 R 排他、签发与停机、合取 7 | 没打中新的 | 回退那一路坏在映射（归 T5）；I-3.16 每一步全扫比没红；合取 7 不锁死（推的） | 随机历史量的 + 推的 |
| T8 discard 调用者 | **打中（轻）** | 扣下（孤儿头、读失败）与报错的段之后不补发；没造出打到被引用段的 discard | 推的（按代码） |
| T9 扫描与计划、Verus 核 | 没打中新的 | 核与发布路径差「本来就重写的不另搬」一条，T6 是它的后果 | 推的 |
| T10 恢复读侧 | 没打中 | 批间崩溃再挂、带快照重挂与冷启动都没被拒 | 随机历史量的 |
| T11 RefFS 与模型 | **打中** | 变异 4346 给出会红的输入：快照个数相同、活头相同、快照抄的根不同的两版 | 量的（原样绿、改坏红） |

所有量的数都是冻结副本拷贝上的数，不是入库装置上的数。自己提的改法都只在我的模型上看过、被攻过零轮。

复跑：`bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-segment-code-r1/repo --work <新的空目录>`（末行 `name=rerun_total same=7 differ=0`；整串没在这台机器上从零跑完，见「没做什么」）。模型目录各文件 sha256（与目录里 `SHA256SUMS` 同一份）：

```
ff167937c34e85294f317a7c2f90200495438df309c59c09493f223dd57e2114  ./out/compaction-seed1055-trace.txt
80510f20c39eee6cc26270b8c1c99b32874bc517d9e610877faeeaa5d1a74c9f  ./out/i35-seed2022.txt
7c2e7f62eab4124be15fcb0232cc4456025cda1cecc288152ccab6c936e269f4  ./out/random-batches.txt
9fa87661e7e5f7ab19f1b591905da5da6b6cf13cf7e7bde145e1d70cf62e4dd7  ./out/remount-sweep.txt
73923686a290e3a7f86b6f01e668473d2019f7b4cd2f04f606d13a3b92ec9ca4  ./out/rollback-minimal-trace.txt
9bf6a98958169653316237427a8d5fb77f0ec8510d6e60d2f4c6205863e630d2  ./out/seed126-prefix-suffixes.txt
db6c7232d808b86f83e8eb7aea421a4358d40e1c8b8d9051964953d04384fc17  ./out/t11-mutated.txt
0e7fe550aaa670e16ddde92bd0ba122db102db3c29e7932f399b0defaf0f3ead  ./out/t11-original.txt
c4730cd2c97997fa9503bc4a8792b7f220ab15bb6a0e8f25a887ef9a49671c03  ./src/attack_common/weights.rs
fe8b8722a9bb29e8406c7b4b6745e0edc88f59f15c3ff68aa066e03f3305135c  ./src/attack_minimize.rs
d9972a77f44d082751791e30509a56b45b2874eb631c366670088301dc6f5b97  ./src/attack_rollback_after_compaction.rs
c4cfbe75ec4776f7d900e76affb1caba17c449b50d51071f4357658b418b38d7  ./src/attack_segment_random_histories.rs
50983e6111f225820111f49a75784e97f7c36ce36f8c6e2ca93eecd203a7aabe  ./src/model_attack_test.rs
641bc42ed37a7616a44f8afc21b1387d4a8101365b2b094136f870151bc589eb  ./src/mounted_session-trace.diff
cffb15da49e6c9d63f1037e0dd34f3df77d864c1ee893cedec8e00a3a354cdbd  ./tools/rerun.sh
```

## 开工核对

冻结副本根上 `sha256sum -c research/prompts/m4-segment-code-r1-snapshot/crates-sha256.txt`，原样输出：

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

退出码 0。工作树（`~/.local/share/singlefs/work/m4-predev`）里这 21 个文件与冻结副本逐个相同（同一条命令在工作树根上跑，一行不是 `OK` 的都没有），另外 `crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-checker/src/walk.rs` 三份 `cmp` 相同；下面的 `crates/...:行号` 两边通用。

## 装置与复跑

- 拷贝：`rsync -a --exclude target --exclude .git` 冻结副本到草稿目录 `repo/`，另拷 `.cargo/config.toml` 与 `.claude/heavy-thread-default`（`crates/singlefs-harness/src/heavy_threads.rs` 编译期 `include_str!` 它）。编译目录是草稿目录的 `target/`，release（工作区 profile 开着 `debug-assertions` 与 `overflow-checks`）。
- 原型全是 harness 档的测试目标（不进 `singlefs-checker-tier`），放在拷贝的 `crates/singlefs-harness/tests/` 下：
  - `attack_segment_random_histories.rs`：攻方混合比重（快照三操作 + 整理 + 回退 + 抬 F + 崩溃恢复抛弃根，另一组去掉回退与抬 F）生成随机历史，经 `singlefs_harness::history::execute_history_with` 跑（理想模型逐步对拍），**每一步之后的镜像都过一遍池级 checker `singlefs_checker::walk::check_pool_image`**（harness 档执行器自己不跑它：`PerStepChecker::Skipped`）。种子取 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7_000_000 + 下标`，别的用例没用过这一段。
  - `attack_minimize.rs`：把打中的一段历史逐块删操作缩短（删后仍打中就留）。
  - `attack_rollback_after_compaction.rs`：回退那一格的写死前缀与放开扫。
  - `attack_common/weights.rs`：两组比重。
- 这一条腿没有调崩溃枚举函数（`enumerate_crash_states*`）：打中的都是不崩或「崩在批之间、再可写挂载」那种整步崩，执行器自己做；全量状态数 0，不占约 10⁷ 的额度。
- 复跑：模型目录 `research/prompts/m4-segment-code-r1-opus-model/tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`），各文件 sha256 在模型目录的 `SHA256SUMS`。

## T5　一批搬迁的事务形态（G3）——打中：整理搬过单元之后回退，回退那一版的映射指着它自己释放掉的目的落点

**结论：打中。** 整理把单元 K 从源落点 S 搬到目的落点 D 之后，管理员回退到搬之前（或意图半路）的一版：回退那次发布释放 D、复活 S，而它写下的中央映射照抄现行那一版，把 K 指向 D。之后这个文件的任何覆盖写都被拒（`PublishError::ReleaseTargetAlreadyReleased`，理想模型答「该成」）；池级 checker 在回退那一版上红 I-3.9（被每一条有效根引用的槽带已释放标志）与 I-3.15（S 上未释放的记录没有映射条目指着）。崩不崩、回退之前重不重挂都打中。

### 复现（拷贝上，量过）

前缀（`attack_rollback_after_compaction.rs` 的 `compaction_then_remount_then_rollback_sweep`，起点 `HistoryStartingPoint::AfterFirstFile`）：顺序写 152 个数据单元 → 顺序写 290 个数据单元 → 挂着时整理推到停（一批 3 个，`region_selector` 16033315046779662336）→〔可写挂载一次，或不挂〕→ 回退到根环里第 k 新的根 → 覆盖写一个数据单元以内的内容。放开扫的是「回退之前挂不挂」两种 × k = 0…7 八种（由用户决定的两步，只固定前缀与整理那一步）。

命令（草稿目录里）：

```
B=$(ls -t target/release/deps/attack_rollback_after_compaction-* | grep -v '\.d$' | head -1)
nice -n 19 bash <仓根>/scripts/hosts/run-with-memory-cap.sh 2G $B compaction_then_remount_then_rollback_sweep --nocapture
```

16 格里 6 格打中，逐格（`ending=` 后面整段 `NewFinding` 原样存在模型目录 `out/remount-sweep.txt`，这里只抄每格开头与实现的答复）：

| 回退之前可写挂载 | k = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| 不挂 | Completed | 拒 | 拒 | 拒 | Completed | Completed | Completed | Completed |
| 挂一次 | Completed | Completed | Completed | 拒 | 拒 | 拒 | Completed | Completed |

「拒」那 6 格的实现答复都是 `implementation_answer: "拒了：PublishError::ReleaseTargetAlreadyReleased"`，模型答 `PublishOverwrite 该成`。不打中的格是回退目标落在整理之后（k = 0）或整理之前没有被搬的单元那几版（目标早于前两次顺序写、或回退到它们之外），推的，没逐格对根环。

不挂、k = 1 那一格加 `ATTACK_TRACE=1`（只在拷贝的 `mounted_session.rs` 里加了三处 `eprintln!`，不改逻辑）重跑一次，原样摘录：

```
TRACE compaction issue txg=CheckpointTxg(6) intent=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50496) }, allocation_generation_at_issue: CheckpointTxg(5), empty_cluster_segments_at_issue: 3300 }
TRACE compaction publish txg=CheckpointTxg(7) kind=Batch intent_on_disk_before=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50496) }, allocation_generation_at_issue: CheckpointTxg(5), empty_cluster_segments_at_issue: 3300 } logged_after=1 relocated=3
TRACE compaction publish txg=CheckpointTxg(8) kind=BatchCompletingTheIntent intent_on_disk_before=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50496) }, allocation_generation_at_issue: CheckpointTxg(5), empty_cluster_segments_at_issue: 3300 } logged_after=0 relocated=2
STEP Operation(3) op=Some(RollBackWhileMounted(RingRoot { index_from_newest: 1 })) outcome=Applied(RolledBack { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(7) }, released_user_visible_units: 2, resurrected_user_visible_copies: 4 })
RED Operation(3) I-3.9: 盘 1 槽 51200 的分配记录带已释放标志（释放代 9），而候选集里每一条有效根都还引用它
```

同一步的 I-3.15 那一行（整行，日志里截到 420 字）：`RED Operation(3) I-3.15: 根（实例 1、txg 9）：盘 1 槽 50556 的分配记录（未释放、跨 2 槽）罩住的单元（码 1、诞生代号 5、写序实例 1、事务号 157）在这一版的中央映射里没有一把相符的 key 指着这一槽（指着这一槽的 key：没有；与它相符的 key 落在：（码 1、出生树 65536、出生 txg 5、实例 1、事务号 157） 在 [(0, 51200), (1, 512`。

读法：txg 8 那一批把数据单元 K（诞生 txg 5）从 S = 50556 搬到 D = 51200；回退到 txg 7（意图半路，K 还在 S）的那次发布 txg 9 释放了 D（释放代 9）、复活了 S，映射却说 K 在 D。

后果另扫了回退之后接的几步（`seed_126_prefix_then_suffixes`，前缀是攻方混合比重种子下标 126 缩出来的四步：两次顺序写 → 整理推两批后崩、可写挂载 → 回退到第 3 新的根）。原样摘录（`SUFFIX` 行开头与实现答复）：

- `SUFFIX nothing ending=Completed reds=["Operation(3) I-3.9", "Operation(3) I-3.15"]`
- `SUFFIX overwrite-one-unit`、`SUFFIX overwrite-290`：`implementation_answer: "拒了：PublishError::ReleaseTargetAlreadyReleased"`
- `SUFFIX mount-then-overwrite`：模型 `的 Data：每块盘一条、仍分配、分配代 5（写它的那次发布）`，实现 `generation: ModelCheckpointTxg(11), is_released: true`——重挂之后读回的账里，K 的记录是已释放的。
- `SUFFIX unlink`、`SUFFIX compact-again`、`SUFFIX cold-start`：`ending=Completed`，I-3.9 / I-3.15 每一步都红。
- `SUFFIX raise-floor-then-rollback ending=Completed reds=[… "Operation(5) I-3.11", "Operation(5) I-3.15"]`：抬 F 之后再回退一次，又多红 I-3.11。

最后这一格说明 D 可以被回收：F 越过回退那一版的 txg 之后 D 进可分配集合，下一次分配落到 D 上就盖掉 K 在映射里的那一份（读侧先走提示 S，提示读得到时读不出错；走映射的读者——区域扫描、checker、`read_*_via_hint_then_central_mapping` 的回退那一支——读到别的单元）。「D 被盖掉之后读文件读错」这一步没有造出来（推的）。

### 根因（按代码推的）

- 回退那次发布接在 `version_to_publish_the_rollback_after` 造的「上一版」后面：用户可见单元与树根取目标版，中央映射树照抄现行那一版——`crates/singlefs-core/src/mount.rs:8187`：「central_mapping_tree: current.central_mapping_tree.clone(),」。
- 装映射条目时照抄的单元按上一版的映射查位置（判决 G3 的写路径重构），查的就是这一棵树的叶：`crates/singlefs-core/src/transaction.rs:3773`：「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」。
- 回退的释放集与复活集却按目标版自己的映射算：`crates/singlefs-core/src/mount.rs:8354`：「let target_units = user_visible_units_through_the_mapping(&target_version).map_err(|_| {」，释放的是「落点不在 R_old 引用的那几个里的」（`crates/singlefs-core/src/mount.rs:8381`：「释放集：cur 引用的用户可见单元里，落点不在 R_old 引用的那几个里的」）。两处取的映射不是同一版，搬过的 key 一边按 S 复活、一边按 D 写映射。
- 用户定的重构射程没列回退：`research/prompts/m4-gc-r1-main-verification.md:66`：「重构的射程：`transaction.rs` 装映射条目那一段、`recovery.rs` 重建 `mapped_units` 与落点、`walk.rs` / `image.rs` 的走读与 I-2.1、两份 WGSL 的镜像」。
- 随机历史里一直没撞到它：整理那一组比重不抽回退（`crates/singlefs-harness/src/history.rs:1176`：「不抽回退、抬 F 与崩溃恢复抛弃根」），快照那一组不抽整理。

### 四句

1. 分不分辨臂：不分辨 G3 的甲乙——甲（提示为准）下回退取目标版的树就带着目标版的提示，不会出这一格；乙下要回退一并改。打中的是乙落地时漏掉的一条写路径，不是乙本身。
2. 被判的系统看不看得到：看得到。回退已经把目标版整个从盘上重建（含它的中央映射），复活与释放就是按它算的。
3. 满足的是哪一句：正文 T5「搬迁后映射与快照读到旧落点」那一类（这里是映射指着已释放的落点）；也落在 T7「R 排他在……回退……四条路上各有没有漏口」那一格的回退路上（不是 R 排他本身，是回退与搬迁的交错）。checker 一侧满足 I-3.9 与 I-3.15 的字面。
4. 改法（只在我的模型上看过、被攻过零轮，没实现、没跑）：回退造「上一版」时，取目标版那几个用户可见单元的映射位置——或照 `take_the_mapping_positions_over_the_location_hints` 那一路把现行映射里这几把 key 换成目标版的位置，或从目标版带上它的中央映射叶。改法在打中的那几格上：推的，修 T5 与 T7 回退那一格；不碰 1032 那一格的 I-3.5（见 T10）。

什么现象会推翻它：入库装置上同一段历史回退之后覆盖写做成、I-3.9 / I-3.15 不红；或者回退那次发布的映射在 txg 9 上指着 S（50556）。

## T11　RefFS 与模型——打中：变异 4346（「非空」不看快照条目）有一条会红的输入

**结论：打中（判别力缺口，后果在 harness 模型一侧）。** `crates/mutations.tsv` 第 4346 行那条改法（`.filter(|_| false)` 让 `snapshot_roots` 恒空）在点名的用例上不红，原因是模型的用户可见状态另有一样 `root_relocations`，它的长度随快照个数变（`crates/singlefs-harness/src/model.rs:830`：「root_relocations: self」那一段按 `user_visible_root_units()` 逐个取，快照每个贡献两格），点名用例比的「建快照前后」「删快照前后」快照个数都变了，长度一变就不相等。会红的输入要快照个数相同、活头相同、快照抄的根不同：

- 甲：建快照 S1（抄活头 A）→ 覆盖写（活头换成 B）；
- 乙：再删 S1（意图 + 最后一批）→ 建快照 S2（抄活头 B）。

甲乙都只有一个快照、活头都是 B、都没搬过根，只差快照抄的是 A 还是 B；实现一侧 `crates/singlefs-core/src/mount.rs:3414`：「|| root_pointers.snapshot_tree_roots != previous_valid_root_pointers.snapshot_tree_roots」逐字节比，分得出。

用例加在拷贝的 `model.rs` 测试模块末尾（模型目录 `src/model_attack_test.rs` 原样），名为 `model::tests::attack_two_versions_with_one_snapshot_each_differ_when_the_snapshots_copy_different_roots`。

原样（不改代码）：拷贝里 `cargo test --release --offline -p singlefs-harness --lib --no-run` 编出单测二进制，经 `run-with-memory-cap.sh 2G` 直接跑这条攻方用例与表里点名的用例（`--nocapture --test-threads 1`），退出码 0：

```
test model::tests::a_snapshot_creation_and_the_final_destruction_batch_change_the_user_visible_state_and_the_intent_does_not ... ok
test model::tests::attack_two_versions_with_one_snapshot_each_differ_when_the_snapshots_copy_different_roots ... ATTACK_T11 before_the_destruction=ModelUserVisibleState { trees_written_by: Some(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(11), instance: ModelInstanceGeneration(2) }), snapshot_roots: [(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(9), instance: ModelInstanceGeneration(2) }, ModelCheckpointTxg(9))], root_relocations: [None, None, None, None] }
ATTACK_T11 after_the_recreation=ModelUserVisibleState { trees_written_by: Some(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(11), instance: ModelInstanceGeneration(2) }), snapshot_roots: [(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(11), instance: ModelInstanceGeneration(2) }, ModelCheckpointTxg(11))], root_relocations: [None, None, None, None] }
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 230 filtered out; finished in 0.10s
```

照第 4346 行的替换文把 `crates/singlefs-harness/src/model.rs:820`（「snapshot_roots: self」）那一段改坏，重编、同样跑两条，退出码 101，原样：

```
test model::tests::a_snapshot_creation_and_the_final_destruction_batch_change_the_user_visible_state_and_the_intent_does_not ... ok
test model::tests::attack_two_versions_with_one_snapshot_each_differ_when_the_snapshots_copy_different_roots ... ATTACK_T11 before_the_destruction=ModelUserVisibleState { trees_written_by: Some(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(11), instance: ModelInstanceGeneration(2) }), snapshot_roots: [], root_relocations: [None, None, None, None] }
ATTACK_T11 after_the_recreation=ModelUserVisibleState { trees_written_by: Some(ModelRootKey { checkpoint_txg: ModelCheckpointTxg(11), instance: ModelInstanceGeneration(2) }), snapshot_roots: [], root_relocations: [None, None, None, None] }
thread 'model::tests::attack_two_versions_with_one_snapshot_each_differ_when_the_snapshots_copy_different_roots' (378280) panicked at 〔拷贝里加了用例的 model.rs，第 7008 行第 9 列〕:
assertion `left != right` failed: 快照抄的根不同：非空
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 230 filtered out; finished in 0.16s
```

（panic 那一行原样是拷贝里的路径加 `:7008:9`，这里把路径换成了〔〕里那句：原样写会被引文核对当成冻结副本的行号；`7008` 是拷贝里加了用例之后的行号。点名用例在改坏之后照样 `ok`，即这条变异今天存活的原样。用例里两版的变量名起初写成序数，按命名纪律改名之后两种都重跑了一次，上面是改名之后这一次的输出。）改完已换回，拷贝里 `grep -c 'filter(|_| false)'` 为 0。

后果（推的）：「非空」把甲乙当成同一个状态，「4 个不同状态」的去重把一个真实可退的状态吞掉，抬 F 的上限按模型比实现高一档；实现那一侧不受影响，坏的是对拍的判别力。

四句：不分辨臂（只是一条用例补口）；系统看得到（快照条目在树表里）；满足正文 T11 字面「攻方可以给一条会红的输入」；改法 = 把这条用例（或同形的一条）进模型单测，变异表第 4346 行点名它。被攻过零轮。

RefFS 的允许集合（建快照、删快照、读快照）与核心层崩溃语义的对照：没打。这一条腿的随机历史只经 `execute_history_with` 与理想模型逐步对拍，没接 RefFS 的翻译（`ReferenceFileSystemTranslation`）；S7 那一半写进「没做什么」。

## T6　整理意图上盘与挂载续做——打中：R 被同一次发布「本来就重写」的节点腾空时，那一批不删意图

**结论：打中（后果轻：多发一次只删意图的发布，模型对拍判不合）。** 去掉回退与抬 F 的攻方混合比重，种子下标 1055（40 步）在第 16 步整理时理想模型报 `这一次整理发布对意图做的事` 不合；缩到 13 步之后在拷贝的 `mounted_session.rs` 加三处只打印的 `eprintln!` 重跑，原样摘录：

```
TRACE compaction issue txg=CheckpointTxg(26) intent=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50432) }, allocation_generation_at_issue: CheckpointTxg(25), empty_cluster_segments_at_issue: 3309 }
TRACE compaction publish txg=CheckpointTxg(29) kind=Batch intent_on_disk_before=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50432) }, allocation_generation_at_issue: CheckpointTxg(25), empty_cluster_segments_at_issue: 3309 } logged_after=1 relocated=1
TRACE compaction publish txg=CheckpointTxg(30) kind=Batch intent_on_disk_before=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50432) }, allocation_generation_at_issue: CheckpointTxg(25), empty_cluster_segments_at_issue: 3309 } logged_after=1 relocated=0
TRACE compaction publish txg=CheckpointTxg(31) kind=IntentCompletedWithoutABatch intent_on_disk_before=CompactionIntent { region: CompactionRegion { device: DeviceIdentity(0), start_slot: SlotNumber(50432) }, allocation_generation_at_issue: CheckpointTxg(25), empty_cluster_segments_at_issue: 3309 } logged_after=0 relocated=0
```

模型的那一句（`ENDING` 行原样摘录）：`活意图 ModelCompactionIntent { issued_txg: ModelCheckpointTxg(26), region: ModelCompactionRegion { device: ModelDeviceIdentity(0), start_slot: 50432, slot_count: 64 }, allocation_generation_at_issue: ModelCheckpointTxg(25) } 在：一次整理发布要么搬一批、要么删掉意图`；实现那一侧：`搬了 0 个单元（其中用户可见树的根 []），发布之后 logged_ops 里有活意图，活意图的 R 按记录全空`。

缩出来的 13 步（起点第一个文件）：可写挂载 ×2 → 建快照 ×4 → 删快照 ×2 → 整理（一批 1 个、推到停）→ 建快照 → 删快照 → 整理（一批 2 个、推两批后崩、可写挂载）→ 整理（第 16 步那一趟）。复跑：`ATTACK_WEIGHTS=without-rollback ATTACK_SEED_INDEX=1055 ATTACK_OPS=17 ATTACK_INVARIANTS=NONE ATTACK_KEEP=0,1,2,3,4,5,6,7,9,10,12,13,16 ATTACK_VERBOSE=1 <attack_minimize 二进制> --nocapture`。

读法（按代码推的）：txg 30 那一批计划里的单元是这次发布本来就要重写的池级节点（`crates/singlefs-core/src/transaction.rs:8853`：「这次重写本来就换下的角色不另搬：它的旧落点经 COW 释放、新落点不在活 R 里」），所以 `relocated=0`；R 里剩下的另一条记录也是这一类节点，同一次发布 COW 走了，发布之后 R 按记录全空。会话判「是不是最后一批」只看计划条数等不等于扫描条数（`crates/singlefs-core/src/mounted_session.rs:2431`：「if planned.len() == scan.allocations.len() {」），没算「本来就重写」的那几条也会离开 R，于是这一批照抄意图，下一次发布才单独删。

- 与条款：判决 G4「意图的收：目标态到达与最后一批同发布删（D8 已定项 5）」——腾空 R 的那一次发布没删意图。停机谓词本身（删意图那一刻 R 按记录全空）仍成立，txg 31 那次删得对。
- 后果：多一次整理发布，吃掉一个根槽与一格 `B_gc`；txg 30 与 31 之间崩，挂载续做要再发这一次；`batches_published` 把一次没搬单元的发布记成一批（`crates/singlefs-core/src/mounted_session.rs:2012`：「batches_published += 1;」），用户改动之后「至多一批」那一支会在一次空批之后就停（`crates/singlefs-core/src/mounted_session.rs:1986`：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」），推的，没造。
- 四句：不分辨 G1 / G4 的臂；系统看得到（这次重写哪些角色在发布路径里先算出来了，`settle_the_allocation_record_tree` 那一串）；满足正文 T6「签发、每一批、达成、丢意图各写什么、删什么」与模型的 `CompactionIntentOfThePublish` 字面；改法（零轮、没实现）：判最后一批时按「这次发布之后 R 里还剩哪几条记录」判（发布路径已经在算好的账上核 R 按记录全空，`region_is_empty_by_the_records` 那一判），而不是按条数——或者先按 `DeleteOnceTheRegionIsEmptyByTheRecords` 发、核不过再退回照抄意图（现成的重发一次那一支就是这个形状）。

T6 的零轮读法，攻了没打中的：

- 意图在批间崩溃之后的可写挂载续做：攻方混合比重里 `CrashesAfterBatchesThenMountsWritable` 那一种在三批随机历史里出现过（`ATTACK_KIND` 计数见模型目录 `out/`），之后的整理接着推盘上那一条，池级 checker 每一步都过，模型逐版对得上（除上面那一格）。
- `B_gc` 数每一次发布：没打中。签发、批、删意图各吃一格，预算公式 `3S − 4 − 已经过去的发布数` 在 S = 8 的默认几何上至少 1；没找出「一次运行把回退候选集挤成 1 个状态」的取样点（整理发布不改用户可见状态，挤的是同一个状态的根，不减少不同状态的个数）。
- 丢意图也发一次发布：没走到。丢意图要 R 里有隔离槽或区域扫描拒，随机历史不注入释放前校验和对不上，三批里一次都没触发（没量到触发次数，推的）。
- `segs0` 记录口径与「记账第 11 项占用口径」：判决 G1 已写它「只作观测量，停机不靠它」，停机与签发只读 `empty_cluster_segments_at_issue` 与现算的记录口径数（上面 TRACE 里 `empty_cluster_segments_at_issue: 3309`），两个口径只在 defer 里有槽的段上差；能翻面的判定只有「全局差值为假就不签下一条」（`the_completed_intent_yielded`），它两边都用记录口径，自洽。没打中。
- logged_ops 根恒为零的旧镜像：没单独造。随机历史从 mkfs 起的那几段（起点 `AfterMakeFilesystem`）每一步都过了可写挂载、冷启动与池级 checker；只读挂载与回退两条路没对旧镜像单独测。

## T4　回退拒与读快照——打中：「第一版有快照就拒回退」挡不住「回退之后再建快照」，池级 checker 在这段合法历史上红 I-3.5

**结论：打中（判据一侧；实现的回退与建快照本身没看出错）。** 判决 S8 说 I-3.5 出现规则 (b) 款在有快照时怎么读「第一版拒回退，不出现」（`research/prompts/m4-snapshot-r1-main-verification.md:37`：「I-3.5 出现规则的 (b) 款在有快照时怎么读」）。拒的只是「回退那一刻现行版或目标版有快照」；回退发生在没有快照的时候、之后再建快照，回退复活的那一对相邻候选版本照样留在根环里，快照下的覆盖写把复活的单元杀进 deadlist 之后，池级 checker 的 I-3.5 在这一对上红。

### 复现（拷贝上，量过）

快照那一组比重（`GenerationWeights::SNAPSHOT_OPERATIONS`，原样不改）跑种子下标 2000–2059 共 60 段、每段 40 步、每一步过池级 checker：4 段红 I-3.5（2022、2035、2041、2052），别的不变量一条没红，模型每一步对得上。种子下标 2022 缩到 5 步（`ATTACK_WEIGHTS=snapshots ATTACK_SEED_INDEX=2022 ATTACK_OPS=28 ATTACK_INVARIANTS=I-3.5 ATTACK_KEEP=5,21,22,26,27 ATTACK_VERBOSE=1`），原样摘录（`STEP` 行截到 420 字）：

```
START starting_point=AfterFirstFile
STEP Operation(0) op=Some(CloseAndMountWritable) outcome=Applied(Mounted { instance: InstanceGeneration(2), publishes: 2, allocation_records_compared: Compared { publishes: 2 }, reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 0, released_records_covered: 0 }, space_admission: AdmittedBeforeAcquisition })
STEP Operation(1) op=Some(PublishOverwrite(ContentChoice { length: InsideOneDataUnit { selector: 7879751949923199194 }, fill_seed: 17840163520990143491 })) outcome=Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 0, released_records_covered: 0 }, floor_raises_pushed_by_the_session: 0 })
STEP Operation(2) op=Some(RollBackWhileMounted(RingRoot { index_from_newest: 2 })) outcome=Applied(RolledBack { target: RollbackTarget { instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(4) }, released_user_visible_units: 4, resurrected_user_visible_copies: 8 })
STEP Operation(3) op=Some(CreateSnapshot) outcome=Applied(SnapshotPublished { change: Created, snapshot: 65549, reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 0, released_records_covered: 0 }, floor_raises_pushed_by_the_session: 0 })
STEP Operation(4) op=Some(PublishOverwrite(ContentChoice { length: SpanningDataUnitsThroughTheSequentialWrite { data_units: 291, selector: 7664500909322504686 }, fill_seed: 4676315349287084229 })) outcome=Applied(Published { reuse: RecordReuse { rewritten_from_released: 0, rewritten_with_changed_span: 0, released_records_removed: 0, released_records_covered: 0 }, floor_raises_pushed_by_the_session: 0 })
RED Operation(4) I-3.5: 相邻两版 (实例 {2}、txg 6) → (实例 2、txg 7)（NoHole）：后一版引用、前一版不引用的单元盘 0 槽 50180（用户可见树 None）诞生代号 3 不大于前一版的 txg，也不在一棵整棵继承下来的用户可见树里（根指针出生 txg 小于前一版的 txg、子树里读得出的诞生代号都不大于它）
```

红的那一对是 (txg 6 覆盖写) → (txg 7 回退那次发布)：回退复活了第一个文件的数据单元（槽 50180，诞生 3），它在 txg 7 的活头 extent 树里，(b) 款本来罩得住（树根出生 txg ≤ 4 < 6）；checker 却给它标「用户可见树 None」。同一对在第 2、3 步之后的镜像上不红，第 4 步快照下覆盖写把它杀进 deadlist（之后只有快照那两棵树还走得到它）才红。

去掉回退的攻方混合比重里另一格（种子下标 1032，缩到 16 步还没缩完；红的单元标的是 `Some(UserVisibleTreeOfTheWalk { kind: 1, tree: 65536 })`，不是 None，落空的是 (b) 款「整棵继承」那一半，形状与上面不同、是不是同一个根因没核）：没有回退、没有搬迁（两趟整理都停在 `compaction_stopped_holes_outside_the_region_too_few`），有建快照、unlink、多次可写挂载、删快照、再建快照；红在第 15 步，红的那一对是 (实例 2、txg 8) → (实例 2、txg 9)，而这一对在第 5 步（unlink）之后就在根环里、第 5–14 步的镜像上都不红。同一对相邻版本的判定随后来加进来的版本翻面，说明判定读的不是这两版自己的东西。

### 根因（按代码推的，没改 checker 验）

I-3.5 每一版的单元带的「在哪棵用户可见树里」取自一张跨全部版本的表、按落点查（`crates/singlefs-checker/src/walk.rs:1525`：「self.user_visible_tree_of_each_placement.get(placement).copied(),」），这张表只在走用户可见树时记（`crates/singlefs-checker/src/walk.rs:1703`：「if let Some(tree) = self.user_visible_tree_being_walked {」），折叠时「标签取落点序最大的那个落点的」（`crates/singlefs-checker/src/walk.rs:1507`：「同一个单元由几个落点折来时，标签取落点序最大的那个落点的」）。一个单元在较新的版本里只剩快照那两棵树走得到、又经共享子树不再重走时，较旧那一版里它的标签就取不到，(b) 款落空。是不是这一处，要在 checker 上加一行打印核（没做）。

### 四句

1. 分不分辨臂：不分辨 S8 的拒 / 抹掉两臂——两臂都允许「无快照时回退、之后建快照」。打中的是 S8 判决「不出现」那一句的前提。
2. 系统看得到吗：checker 手里有这一对版本各自的树与根指针出生 txg，看得到；它取标签取的是跨版本那张表。
3. 满足哪一句：正文 T4「第 2 批代码轮 T2 打中的五个读树表的读者……今天对码 14 / 15……各放行还是拒」之外，按正文第四节攻方立场「合法镜像被拒」那一类——合法历史的镜像被池级 checker 判红。核心层的五个读者对码 14 / 15 今天都放行（`crates/singlefs-core/src/recovery.rs` 的 `user_visible_tree_root_pointers` 把它们记进 `snapshot_tree_roots`，冷走读跳过它们），随机历史里冷启动与读快照没有一次读失败。
4. 改法（零轮、没实现）：I-3.5 的标签按版本记（每一版走读时把自己走到的落点连同标签记下），或在判 (b) 款时按 R 自己的用户可见树根重算子树成员；S8 的 (b) 款读法要另写一句「回退之后建快照」这一形。崩溃点重放与崩溃注入里用到 I-3.5 的流，只要前缀里有「回退 → 建快照 → 快照下覆盖写」就会误红（推的，没跑 checker 档）。

什么现象会推翻它：入库装置上同一段 5 步历史第 4 步之后池级 checker I-3.5 不红；或者 txg 7 那一版真的不引用槽 50180（那就是实现的回退少复活了一个单元，改记到实现一侧）。

T4 其余几样，攻了没打中：回退拒（`crates/singlefs-core/src/mount.rs:7653`：「.filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)」数的是全部码 14，销毁中的也算，所以「删到一半崩溃、回退」也拒）——随机历史里回退撞到快照全部拒成 `RollbackError::SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`；读快照在搬过快照单元之后（攻方混合比重里整理搬了 deadlist 留存的单元）全部读出建快照那一刻的内容，模型对得上；读快照走的是快照两条根指针往下、单元提示读不出再经现行那一版的中央映射（`read_the_first_file_of_a_snapshot`），留存的映射条目被整理搬走之后跟着换位置（`transaction.rs` 里 `mapping_entries_kept_for_the_snapshots` 随 `destination_of_key` 换）。

## T8　discard 调用者（G5）——打中（轻）：被扣下或报错的段之后不补发；没造出 discard 打到还被引用的段

**结论：一处轻的打中（推的，按代码），主问题没打中。**

- 打中（轻）：「装着还没清扫的孤儿头的段不发」（零轮）之后不补发。要发的段由分配器交出时就从「回收过、还没发」的集合里摘掉（`crates/singlefs-core/src/allocator.rs:830`：「segments_to_discard.push(self.segment_start_slot(segment_index));」，紧接着那一行摘掉），会话随后才判孤儿头、扣下（`crates/singlefs-core/src/mounted_session.rs:1769`：「.withheld_holding_unswept_orphan_heads」）；那个集合只在回收时加（`crates/singlefs-core/src/allocator.rs:791`：「self.segments_reclaimed_not_yet_discarded.insert(segment);」），全仓别处不碰它（`grep -n segments_reclaimed_not_yet_discarded crates/singlefs-core/src/*.rs` 命中全在 `allocator.rs`）。所以清扫抹过孤儿头之后，这一段要等段里再有槽被回收才会再交出来；读失败扣下的、块设备报错的（`crates/singlefs-core/src/mounted_session.rs:1796`：「Err(_cause) => discards.failed.push((device_identity, segment_start)),」）同样一次就丢。后果只是少发 discard（盘不知道这一段可以回收），不碰正确性。正文 T8「清扫之后补不补发」答：不补。改法（零轮）：扣下与报错的段放回集合，或清扫发布之后把它清过的段记回去。
- 没打中：discard 打到还被某一版根引用的段。整段可发要求段里 `used_per_segment` 为 0（仍分配与 defer 里的都算占着）、没有隔离、没有抬 F 扣住，回收又要求释放代 ≤ max(F_生效, 环里最旧有效根)；我找不出一版在环里、F 之上、又引用已回收槽的形状（推的）。攻方混合比重的 290 段随机历史里抬 F、整理、回退交错，discard 每次都在发布返回之后发；没出问题的 277 段每一步池级 checker 全绿，出问题的 13 段只记了每段第一处红（都是 I-3.5 或 I-3.9，见文末「随机历史一览」），没有一段的第一处红是读不出单元的那几条（I-2.x、I-1.11）。
- 经 T5 那一格走得到的形状（推的，没造出来）：回退之后映射指着已释放的 D，F 越过回退那一版、D 那一段整段可再分配时，discard 会发到 D 上——发的那一刻段里确实没有已分配记录，谓词照字面为真，坏在映射不在 discard。我接了「抬 F 到上限 → 整理一趟 → 冷启动」与「可写挂载 → 抬 F → 整理 → 抬 F → 整理 → 冷启动」两种后缀（`seed_126_prefix_then_suffixes` 第 8、9 格），没推到 D 被回收，红的仍只有 I-3.9 / I-3.15（第二格另在挂载那一步模型报分配代不合，同 T5）。
- discard 交 Unsupported、交错误、长度 0：调用方都只记下、不让发布算失败、不重试；长度恒为一段，不发长度 0。与判决 G5「报错不让发布算没发成」一致，没打中。
- 变异表第 4453 行（「不论整段可不可再分配都发 discard」）不超时的反例：没造（它点名的是 checker 档的崩溃枚举用例，这一条腿不碰 checker 档；harness 档里能红它的要一段「回收记下、整段不可再分配」的历史再读那一段，没写）。

## T7　R 排他、签发与停机、合取 7——没打中新的；回退那一路见 T5

- R 排他四条路：分配三种落点函数都带 `excluded_region`（`allocator.rs` 的开段、提交内生块回落、用户数据落点），抬 F 的空发布与整理批走同一个分配器；unlink 与删快照的释放进 defer，不分配。回退那一路：回退可以发生在活意图期间（`roll_back_by_a_forward_publish` 只拒快照，不拒活意图），logged_ops 照抄现行那一版，复活的记录落进 R 里时分配代 ≤ `c0`，之后接着搬——R 排他本身没漏口；回退与搬迁交错坏在映射，见 T5。
- K-c（一个槽既钉住又在隔离集里）：会话在扫描之前先按常驻隔离集判 R（`region_holds_a_quarantined_slot`），钉住与隔离同时成立也丢意图；选段时钉住 ∪ 隔离一起让段不合格。与「钉住与隔离同时成立也算」一致，没打中。
- `empty_segments` 增量维护：池级 checker I-3.16 每一步把最新根记账里的全空聚簇段数与全扫重算比（`crates/singlefs-checker/src/walk.rs` 的 I-3.16 那一节），没出问题的 277 段随机历史（分配、释放、整理搬迁、回退、可写挂载重建五处都走到）每一张镜像都没红它；出问题的 13 段只看了第一处红，I-3.16 没判到的那几张不算。没打中。
- 合取 7 被拒不推抬 F 会不会锁死：整理的签发、批、删意图三种发布都免合取 7（`crates/singlefs-core/src/transaction.rs:8330`：「let writes_in_the_foreground =」那一判只收写文件内容、建 inode、建快照），删除类也免，所以「全空段少于 `N₀` → 前台写被拒」时整理与删除都发得出；会停在「R 外空洞不够」（K-g）而前台写又被合取 7 拒的盘，用户还能删文件、删快照腾段。没造出锁死（推的，没在小盘上量）。
- `run_compaction` 一次调用至多签几条、发布后至多一批（G8 甲）：显式入口按 `B_gc` 停，用户改动之后那一趟数到一批就停——T6 那一格里一次「没搬单元的批」也被数成一批，见 T6。

## T1　建快照那次发布——没打中

- 独立发布、两个树号、码 14 / 15 一对、活头前驱换成 S.txg、第 11 个写前拒：随机历史里建快照都经会话（先走空再发），之后每一步池级 checker 的树表各格（I-9.16 ③、I-9.25、I-9.27 加的 14 / 15 那一格）与配对判定没红；快照数到上限的拒没走到（随机历史里同时活着的快照没到 10 个，没量最大值）。
- 事务号取上一版 + 1（零轮，`crates/singlefs-core/src/transaction.rs:7252`：「let transaction_of_the_snapshot = previous.highest_transaction_number_in_this_instance + 1;」）：覆盖写、建 inode、删除各步、整理的各次发布都是同一个取法，挂载的写行、暖机、抬 F 的空发布照抄上一版的「最高事务号」不占号，回退那次发布写 `transaction: 0`（`crates/singlefs-core/src/mount.rs:4813` 一带三处空发布计划同写 0）。想撞号的形状是「两次发布取到同一个号」：要一次不占号的发布之后接一次占号的、再接一次取同一个上一版的——会话每一步都把 `current` 换成新的一版，没有两次发布接同一个上一版，造不出（推的）。回退、根环、准入与 D16 的判定都不按事务号判，按 txg 与实例。
- 各类发布续没续码 14 / 15：续的那一段在发布路径的公共一段（`publish_admitted`），每一种发布都走它；回退那一次另造上一版，树表条目照现行那一版（回退要求两版都没快照，续的是空集）。随机历史里建快照之后接覆盖写、unlink、整理、抬 F、可写挂载、冷启动，每一步之后读快照都读出建快照那一刻的内容。
- S1 配对判定在建快照之后的每一版上：同上，checker 的配对那一格在没出问题的 277 段里没红（出问题的 13 段只看了第一处红）。

## T2　释放分流与 deadlist——没打中

- 切口与「只有六个用户可见角色问寿命判定」：照码读（`asks_the_snapshot_lifetime_judgement` 六类为真），池级元数据角色直接进 defer。随机历史里快照下覆盖写、unlink、整理搬留存单元、删快照交错，I-3.6、I-3.7、I-3.9、I-3.5 ③ 与逆向那几条在没出问题的 277 段里都没红。
- `birth == S.txg` 那一格：判定是 `birth > p` 才放（`crates/singlefs-core/src/transaction.rs:5006`：「Some(birth) if birth > previous_snapshot_txg_of_the_live_head => {」），等于 S.txg 的进 deadlist。建快照那次发布不写用户可见单元（`file: None`、不建 inode），出生 txg 等于 S.txg 的用户可见单元造不出；整理搬迁不改 birth。这一格是空的（推的）。
- deadlist 节点空间按普通分配（零轮）：deadlist 树的节点是 `DeadlistTree` / `DeadlistTreeNodeBelowTheRoot` 角色，不问寿命判定、换下直接进 defer；它们不在任何快照的子树里，「被下一次释放判成快照留着」的路走不到。没打中。
- 下一侧是活头时 hi 取这一批 txg（零轮）：意图那次把 P 传给活头（`crates/singlefs-core/src/transaction.rs:10830`：「Some((NextSideOfASnapshot::LiveHead, predecessor)) => predecessor,」），之后被杀的单元纪元都是 P < lo；意图与最后一批之间只有删快照的批、挂载的写行与暖机、抬 F，都不杀用户可见单元，所以 [lo, 意图 txg) 与 [lo, 这一批 txg) 放的是同一组。想打穿要「意图之后、最后一批之前有一次杀用户可见单元的发布」：会话的每一种改动与整理都先走空队列（`publish_the_change_raising_the_floor_for_space` 开头、`run_compaction_on` 开头），挂载续做先走空。崩在意图与批之间、只读挂载读快照也不写。没造出（推的）；随机历史里删快照中途崩的形状没有专门造（执行器的崩只在整理那一步）。

## T3　删快照意图、批与续做——没打中

- 意图在码 14 flags 位 0、最后一批删一对：随机历史里删快照全部走完，树表配对没红，删完之后快照个数对得上模型。
- 批大小与 unlink 共用开关（零轮，`crates/singlefs-core/src/unlink.rs:112`）：同一个 `UnlinkBatchSize` 换算成每批 deadlist 条目数，两者不交错（会话串行），共用只是同一个数，没找到让它翻面的判定。
- 有树而文件集合为空的一版上删快照放行（丁 2 改，`crates/singlefs-core/src/mounted_session.rs:1384`：「UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {」）：快照那一组与攻方混合两组比重在 unlink 之后都抽删快照，走完、模型对得上、checker 没红。删快照与 unlink、整理在同一次挂载里交错的先后：会话串行，队列先走空；整理先走空删除续做再推意图。没打中。
- 销毁中的快照在三处：回退拒数全部码 14（销毁中的也拒）；建新快照在有销毁中的快照时整次拒（`crates/singlefs-core/src/transaction.rs:7231`：「if let Some(being_destroyed) = previous.snapshots_being_destroyed.first().copied() {」），上限那一判轮不到它；读快照不看「销毁中」记号，崩在两批之间、下一次可写挂载之前用 `read_the_first_file_of_a_snapshot` 读那个快照，已放掉的单元在 defer 里、没被重用，读得出（推的，没造）。

## T9　区域扫描与计划的修订、Verus 核——没打中新的；与 T6 那一格有一处交叉

- 第 2 批代码轮 T4、T5 打中的反例：判类次序钉住 → 隔离 → 读头 → 类标签 → key 与位置照码读过（`classify_allocation`），隔离槽不看映射一律 `Quarantined`；扫描拒过的段进不合格段表、选段跳过它。上一轮那几处翻转今天各有用例（没逐条重打变异，只读了代码）。没找到新开的口子。
- `TwoRecordsMovableUnderOneMappingKey` 整次拒（零轮，`crates/singlefs-core/src/region_scan.rs:261`）：要两条起点不同的记录都判可搬、头里同一把 key，等于一条映射条目在同一块盘上指两个槽；合法历史里留存条目从上一版的映射条目照抄、整理搬它时跟着换，两条位置项各在一块盘上。造不出合法镜像让含快照的段被它拒（推的）；攻方混合比重里整理搬了快照留存的单元，没有一趟停在「扫描拒」。
- Verus 核与核心层：P1 只证选段资格、P2 证计划健全，对拍单测的输入域是一段至多 4 条记录、5 种形状、批大小 1–3（`crates/singlefs-core/src/compaction_plan.rs:808`、`:809`），共 5⁰ + … + 5⁴ = 781 种扫描结果 × 3 个批大小。核与代码之间还差的一条：P2 / `plan_batch` 不知道发布路径对本来就要重写的角色不另搬这一条（`crates/singlefs-core/src/transaction.rs:8341`：「这次重写要换下的角色不另搬」），所以「计划的那几个 = 这一批搬的那几个」「计划搬完扫描里每一条 = 这一批是最后一批」两句在核里成立、在发布路径上不成立——T6 那一格就是它的后果。核的前提「R 内每条未释放记录都有映射 key 且不在隔离集里」在 T5 那一格（回退之后 S 上的记录没有映射条目指着）之后不成立，扫描把那条记录判成 `Dead`（推的，没跑）。

## T10　恢复读侧——没打中

- I-2.5 挪到盖映射位置之前（判位置项设备升序）：合法搬迁镜像在随机历史里经过「整理推几批就崩、再可写挂载」（攻方混合两组比重都有 `CrashesAfterBatchesThenMountsWritable`）与整理之后的普通重挂、冷启动，没有一次挂载或冷读被拒；合法快照镜像（留存映射条目）同样。
- 映射条目值逆序的拒、logged_ops 条目不升序与多于一条整理意图的拒：合法历史造不出这样的条目（发布路径一次至多签一条、logged_ops 按 key 写）；没造坏镜像去测「拒的是这一版还是整个挂载」，只读了代码：`logged_operations.rs` 第 317 行起那几个拒绝在重建那一版时报，挂载按所选那一版走，选不出就整次挂载失败（推的，没逐条对 `mount.rs` 的退路）。
- 池级 checker 一侧的 I-3.5 误红见 T4；那是判据的读侧，不是核心层恢复的读侧。

## 零轮读法逐条（正文第二节）

| 读法 | 攻在哪一格 | 结果 |
|---|---|---|
| 码 14 的 `previous_snapshot_txg` 写前驱链 | T2、T3（删较新 / 较旧的快照、续上前驱） | 没打中 |
| 建快照那次事务号取上一版 + 1 | T1 | 没打中（推的：造不出两次发布接同一个上一版） |
| `mount_writable` 不换入口 | T4、T10（带快照的池重挂、冷启动、读快照） | 没打中 |
| deadlist 节点空间按普通分配 | T2 | 没打中 |
| 删快照时下一侧是活头 hi 取这一批 txg | T2 | 没打中（推的：串行之下两种取法同一组） |
| 删快照批大小与 unlink 共用开关 | T3 | 没打中 |
| C300 写前拒 | T2、T3 | 没走到（留存映射条目缺失要坏镜像，没造） |
| 有树而文件集合为空的一版上删快照放行 | T3 | 没打中 |
| `segs0` 沿用记录口径 | T6 | 没打中 |
| `B_gc` 数整理的每一次发布 | T6、T7 | 没打中；T6 那一格让一次空批也吃一格 |
| 丢意图也发一次发布 | T6 | 没走到（随机历史不注入隔离） |
| 意图、不合格段表、常驻隔离集归位到会话 | T6、T7（批间崩溃再挂、会话续推盘上那一条） | 没打中 |
| 装着还没清扫的孤儿头的段不发 discard | T8 | **打中（轻）**：扣下之后不补发 |
| 两条能搬的记录映射 key 相同时整次拒 | T9 | 没打中（推的：合法镜像造不出） |
| `empty_segments` 记录口径增量维护 | T7 | 没打中（I-3.16 每一步全扫比） |
| 合取 7 被拒不推抬 F | T7 | 没打中（推的：整理与删除都免合取 7） |
| 调查员修法（按 key 进出之外，搬走的 key 所在叶连同到根重写） | T5 | 没打中直接的；随机历史与写死前缀里整理搬了几十个单元，映射树形状对得上 checker 的 I-2.x / I-1.11；`ckpt_cost` 罩不罩得住没量（准入那一判在整理批上照判，没找到越过它的取样点，推的） |
| 丁 3 报告「交主 agent 的读法」另 12 条 | — | 没逐条攻（没读那份报告，写进「没做什么」） |

## 随机历史一览（拷贝上，量过）

四批，每批 12 件（第一批 16 件）并行，每件经 `run-with-memory-cap.sh 2G` 直接跑攻方随机历史的测试二进制，每段 40 步，每一步之后过池级 checker。批的段数按先跑一段估：40 步一段在这台负载（loadavg 80–190）下 10–20 秒。

| 批 | 比重 | 种子下标 | 跑完的段 | 过 checker 的镜像 | 出问题的段 |
|---|---|---|---|---|---|
| 1 | 攻方混合（含回退、抬 F） | 100–259（16 件 × 10 段） | 110（5 件被前台时限截断，没算） | 4507 | 6 |
| 2 | 攻方混合去掉回退与抬 F | 1000–1059 | 60 | 2436 | 2 |
| 3 | `SNAPSHOT_OPERATIONS` 原样 | 2000–2059 | 60 | 2460 | 4 |
| 4 | 攻方混合 | 300–359 | 60 | 2460 | 2 |

数是 `grep ATTACK_SUMMARY` 各件日志加出来的；出问题的段逐行在模型目录 `out/random-batches.txt`（第一批那几件跑时环境变量还叫 `ATTACK_FIRST_SEED`、摘要行写 `first=`，之后按命名纪律改成 `ATTACK_SEED_START` / `seed_start=`，逻辑没动）。出问题的 14 行归类：I-3.5 红 11 段（T4 那一类与 1032 那一形）、I-3.9 红 1 段（种子下标 126，T5）、模型不合 2 处（126 的回退被拒，T5；1055，T6）。崩溃枚举一次没调，状态数 0。

## 没打中的形状

- 回退之前重挂与否、回退目标 k = 0…7 两样放开扫过（T5 那张表）；整理批大小 1、3 与选段 0–2 三段在「一个数据单元 / 20 个数据单元」的小文件上 96 格全不中（`rollback_after_a_compaction_sweep`）：小文件的单元不落在被选的段里，打中要文件大到数据单元铺满几段（152 + 290 个单元）。
- 回退之后接的后缀 10 种（`seed_126_prefix_then_suffixes`），没有一种推到 D 被回收、被重用。
- 快照那一组原样比重 60 段里没有一次 I-3.x 以外的红；删快照中途崩、只读挂载读快照、快照数到上限没专门造。
- discard 打到被引用的段：抬 F、整理、回退交错的 290 段里没出现读不出单元的红。

## 这条腿自己的限度

- 判据是池级 checker 与理想模型：它们自己的误红会被我当成线索报出来（T4 就是判据一侧的），它们一起漏的我看不见。
- 每段只记第一处红；出问题的 13 段里第一处之后还红了什么没看。
- 根因都是按代码推的，只在拷贝里加了只打印的 TRACE，没改实现去验证改法。
- 机器负载很高，前台 4 分钟时限截断了第一批 5 件与几次缩短；缩短只做到「删任一块都不再打中」的块粒度，1032 那一段没缩完。
- 自己提的改法都没实现、被攻过零轮。

## 没做什么

- RefFS 那一半（S7 的允许集合与崩溃语义）没打；harness 模型只核了「非空」口径（T11）。
- 丁 3 报告与各补丁的 `report.md` 没读；它们自报的 12 条读法没逐条攻。
- 没跑 checker 档、没起门禁、没起 GPU；变异表第 4453、4346 行点名的原用例没跑（4453 点的是 checker 档）。
- T4 的 checker 根因没在 checker 上加打印核；T8 的「不补发」没造历史，只读代码。
- 复跑脚本 `tools/rerun.sh` 的整串没有在这台机器上从零跑完（release 从零编一遍超过前台 4 分钟时限）：在草稿目录里核过它的前几步（补丁打上之后与我的拷贝逐字节相同、原型与比重文件与拷贝里的相同、模型用例接上之后只差一个空行），之后各条命令与产出 `out/` 那几份的命令相同。整串交主 agent 跑，命令在下一节。

## 要主 agent 跑的命令

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r1-opus-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev \
  --frozen ~/.local/share/singlefs/work/m4-segment-code-r1/repo \
  --work <新的空草稿目录>
```

工作目录取仓根；线程数取环境变量 `RERUN_THREADS`（默认 31）；跑完看末尾 8 行：七行 `name=rerun_compare file=<名>.txt identical=true` 与一行 `name=rerun_total same=7 differ=0`。编译两遍 release（第二遍是打了变异 4346 之后只重编 harness），负载低时预计 10 分钟内（推的）。

## 草稿目录清理

交回前删掉了草稿目录 `~/.local/share/singlefs/work/m4-segment-code-r1-attack/` 里的仓拷贝 `repo/`（删前 `du -sh` 28M）与编译目录 `target/`（删前 2.4G）。留下的是日志 `logs/`（612K）、原型与批跑脚本 `attack/`（792K）与编译日志，共 1.5M；原型已原样进模型目录。

