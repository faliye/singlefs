# m4-segment-code-r2 攻方腿（Opus）报告（2026-10-16）

<!-- doc-lint:not-numbers R1 R2 R3 R4 R5 R6 R7 R8 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 S2 S3 S7 S8 G1 G3 G4 G5 G8 G9 -->

立场：正文 `research/prompts/_m4-segment-code-r2-body.md` 第四节攻方那一行。只在冻结副本的拷贝上造反例；拷贝里量出的数是副本上的数，不是入库装置上的数。自己提的改法都只在我的模型上看过、被攻过零轮。

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| R1 回退之后的映射（F1） | **打中（模型一侧，后果超出清单）**；F1 本身没打中 | 整理搬过根单元之后回退，理想模型把现行那一版「最后一次搬它」并进回退那一版，抬 F 的上限比实现低，执行器报 `NewFinding`：4 GiB 310 段里 3 段、小盘 496 段里 12 段；F1 那一侧 240 格写死形状与 2924 段随机历史里映射、释放、复活都没红 | 量的（副本上改模型那一处之后同一批 0 段） |
| R2 腾空 R 的那一批与删意图（F2） | **打中（后果轻）** | 一份副本连读两次读不出、下一遍又读得出时，两条「核不过就照抄意图」的退路在产品代码里都走得到，发出去的那一版 R 按记录全空而意图还在，再多一次只删意图的发布 | 量的（注入瞬时读错） |
| R3 签发那次的 `segs0`（F3） | 没打中 | 定义句与全局差值两边都是记录口径；换值重算之间分配器整份换回；后一遍读盘不同就会 panic 的那一形没造出 | 量的 16 格 + 推的 |
| R4 强制开关（F4） | 没打中 | 产品路径碰不到；两臂只换 `verdict_of` 一处；强制进 deadlist 那一臂造得出纪元 0、以后没有快照放得掉的条目（只在测试里） | 推的（读码） |
| R5 空文件集合上的快照（F5） | 没打中；RefFS 口径不同、不分辨 | 核心层与模型 115 格对得上（含第 11 个被上限拒、回退拒）；RefFS 改的是空池那一形，核心层拒、翻译又收不到删完之后那一形 | 量的 + 推的 |
| R6 RefFS 快照允许集合（F6） | **打中（RefFS 单元层，今天翻译走不到）** | 恢复读出的内容对不上任何状态时快照集合退回 S_0：之后建的快照被记成没有，读它判红 | 量的（RefFS 上）+ 推的（走不到） |
| R7 开着的四个形状 | 两个造出取样点、两个造不出 | 「一个没搬而 R 还不空」两个取样点，模型判不合；ckpt_cost 余量 528–534 槽；零个全空段与两条同 key 可搬记录造不出 | 量的 + 推的 |
| R8 第一轮六格再抽样 | 没打中（一次抽样） | 换小盘、新比重、瞬时读错三个攻击面，T1、T2、T3、T7、T9、T10 都没出新的红 | 量的 |

所有量的数都是冻结副本拷贝上的数，不是入库装置上的数。自己提的改法都只在我的模型上看过、被攻过零轮。

复跑（整串没从零跑过；各步在已编好的拷贝上分开跑过，15 份产出逐字节相同，见「没做什么」）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r2-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-segment-code-r2/repo --work <新的空草稿目录>
```

末行 `name=rerun_total same=15 differ=0`。模型目录各文件 sha256（与目录里 `SHA256SUMS` 同一份）：

```
d25707e3a42cbbf8e3aedefc9d1334b495b755c876bed74c4a86dcf787d43d68  ./out/batch6-compaction-publish-kinds.txt
d1d90b1d8ccabc670e137e271a622916ed392a4419785b19f92c4e98274427af  ./out/checkpoint-cost-margins.txt
f0dfba933d4507aa62c1652e800674f78338fbe87381270c7c78fc95fddf3ebb  ./out/dump3395.txt
27afcd9355cfd56e84bb7abd51954fbc12c3ad36b8ed55f5d9a8b870f763066f  ./out/dump535.txt
ac34df1d9dc2ace598c9701ab35e665bf2c4c79dbb31f19a45ea94daf97142b6  ./out/dump640.txt
ae0f23d1811037fb5c1aca804888ed50aaf8972370a2652a2b7901d9e8e4b764  ./out/empty-file-set-snapshots.txt
72b1474f4fc7ff65a15d7e816a48664a69a73f9bf213e3828a031b9c8d81723c  ./out/model-rollback-relocations-before-and-after.txt
4a858ff181bd606af5d8df2e2de75305d08667a50794b17be3d50cd1a4774e58  ./out/mount-transient-reads.txt
771a831a0203544c35f171fd22eea1a5baba96d03081522c4a39a5dba4dede5e  ./out/random-problems.txt
b9125d92d96b9b938da7e306ca632e542a7de24e64bb14b39e2851de8ce101fb  ./out/random-summaries-small-devices-and-later.txt
5195acf691efa8694abe6171abe27da243b1a2991da7e064e561a2855bff7797  ./out/random-summaries.txt
8d05c31a66ce0bbfd2fcf6332d314782fd5a27a00fd46125560429662bd3694e  ./out/reference-filesystem-probe.txt
a10be3096ba091fb6a8e84cbffdf701d465ca854b8a5e12b1dd44da4563dfcf8  ./out/rollback-shapes-samples.txt
ad06fb4ba823224a0569d1b4eb5a4281c4b541bd9a17495a7a2d7034da55ade8  ./out/rollback-shapes-sweep.txt
2c28b7bf7b927563fefa36eda3e3490cbc4140a48ea3f94461b9c618f7cf648e  ./out/seed6318-small-device.txt
f55722e287b54ee7acbf9cbebda6d5d372c2d104ac9226af789efdfe936f0f85  ./out/trace297.txt
56fbaaa829f1d519bac6fba9cb5e67ce3177443aedd4e2b77af963047e09ec1d  ./out/trace476.txt
014f14667c83f08e95cdd6abdaa7c375013d431522d234b339566b7d7fea0ac9  ./out/transient-1055.txt
834efb71bdc838d3c1d2bebd4018948049d31360c5dc645ccaec48b89e1e4963  ./out/transient-completing.txt
220af7583cc6a05f6976e86aaed6b6588cce63de097cf3b1ab9279d82c56ab41  ./out/transient-issue-pairs.txt
2df602ac221460b3508f4032f56ff9913f1a53fd31cfeaceb84cbff52314150e  ./src/attack_common/weights_r2.rs
f6c8d5764c6043fb041313690501bc0156d107b91c6465517a054328157350aa  ./src/attack_r2_ceiling_dump.rs
a1c5c379d786633a8fc9444a05435618ae546862b4a3df1a18071cd90392965b  ./src/attack_r2_empty_file_set_snapshots.rs
7fed7e250352a4b49b7e80efd1b4c81f71e696ba177d27b2d7dd197641a4d726  ./src/attack_r2_minimize.rs
e4483abfc5f3ec4a63a27b3ee0c38ecf4ec34ae191eeb0a155b7e5e54a34c4e7  ./src/attack_r2_random_histories.rs
c687622162859b70cf4d91fa0e858ce9391f46e272b1a4976a4fd0f939b192af  ./src/attack_r2_reference_filesystem_probe.rs
67eab6c8d33452b5e9e488fa60c0aaf9a23233f89fca9d83d43253ca5b6cad25  ./src/attack_r2_rollback_shapes.rs
43361818c29ea21be3abb0006a30d3f40612bd59cf6539ec0fb529ab29cb334f  ./src/attack_r2_transient_read.rs
bc5e0df4d9451c2c1499c7e9fa3f97f221317eaa9cb8d78ae957ac74e95586b0  ./src/model-rollback-relocations.patch
6f4025b9fa8708aaa7fb15e13b34a4d88e4cdb452a0c8b96c0cfa6607f3f6e46  ./src/run_random_batch.sh
8b3dcd6c1bec4f4352ed04decff6e335ba195b214be2a8396c330c27f66ba3ac  ./src/trace-prints.patch
52481229259ff5ed24555ac920cfe41b163d57c098336bed521d4fd53b642c9c  ./tools/rerun.sh
```

## 开工核对

冻结副本根上 `sha256sum -c <仓根>/research/prompts/m4-segment-code-r2-snapshot/crates-sha256.txt`，原样输出：

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

退出码 0。下面引代码一律写冻结副本里的路径与行号（`crates/...:行号`，冻结副本根是 `~/.local/share/singlefs/work/m4-segment-code-r2/repo/`）。

## 装置

- 拷贝：`rsync -a --exclude target --exclude .git` 冻结副本到草稿目录 `~/.local/share/singlefs/work/m4-segment-code-r2-attack/repo/`，另拷仓根的 `.cargo/config.toml` 与 `.claude/heavy-thread-default`。编译目录是拷贝里的 `target/`，release（工作区 profile 开着 `debug-assertions` 与 `overflow-checks`）。每条 cargo 与每个测试二进制经 `capped.sh 31` 与 `run-with-memory-cap.sh`（≤ 8G）起。
- 原型全是 harness 档的测试目标（放在拷贝的 `crates/singlefs-harness/tests/` 下，不进 `singlefs-checker-tier`），原样存进模型目录 `src/`：
  - `attack_r2_random_histories.rs`：随机历史，执行器逐步与理想模型对拍，另外每一步之后的镜像过池级 checker `singlefs_checker::walk::check_pool_image`（执行器自己不跑它）。比重在 `attack_common/weights_r2.rs`：修复批新加的 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK` 原样（`with-rollback`），与两组新写的（`r2-mix`：整理 + 回退 + 删完之后的快照；`empty-file-set`：删完之后的快照）。种子 = `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 8_000_000 + 下标`（第一轮攻方是 `+ 7_000_000`，修复批的 F1 用例是 `+ 60_000` 起 4 个）。
  - `attack_r2_minimize.rs`：把打中的一段逐块删操作缩短。
  - `attack_r2_ceiling_dump.rs`：每一步之后打根环里每条根的 inode 树、extent 树根指针字节的指纹。
  - 只打印的 TRACE：拷贝的 `crates/singlefs-core/src/` 下 `mounted_session.rs`、`transaction.rs`、`region_scan.rs`、`compaction_plan.rs` 里加了 `ATTACK_TRACE` 开着才打印的几行（整理每次发布的种类、计划条数、扫描条数、搬了几个、发布之后 R 空不空、意图在不在，F2 那两条退路走没走到，签发之前的全空段数与换值重算，区域扫描拒），不改逻辑；补丁存模型目录 `src/trace-prints.patch`。
  - `attack_r2_rollback_shapes.rs`、`attack_r2_transient_read.rs`、`attack_r2_empty_file_set_snapshots.rs`、`attack_r2_reference_filesystem_probe.rs`：R1、R2 / R3 / R7 / R8、R5、R6 各节用的写死形状与注入。
- 这一条腿没有调崩溃枚举函数（`enumerate_crash_states*`），全量状态数 0，不占约 10⁷ 的额度。

## R1　回退之后的映射（F1）

**结论：F1 在产品代码一侧没打中；打中的是理想模型一侧（测试代码），后果超出已知清单：** 整理搬过用户可见树的根单元、之后回退到搬之前的一版，再抬 F 时理想模型算的上限比实现报的低（缩短后的三格：7 对 9、11 对 13、7 对 12），执行器判 `NewFinding { aspect: "抬 F 的上限" }`。修复批为 F1 新加的比重 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK` 在 310 段随机历史里撞到 3 次。根因按代码推、按实现那一侧的根指针字节量过：模型在回退那条根上把现行那一版「最后一次搬它」的记录并进去（`crates/singlefs-harness/src/model.rs:2748`：「// 搬过的根单元：照抄上一版的（回退那次再并进目标那一版的，用户可见树取的是它的），只留这一版还引用着的；这一批搬的由」，代码在 `crates/singlefs-harness/src/model.rs:2750`：「let mut relocations_carried = previous.user_visible_root_relocations.as_ref().clone();」），目标那一版之后才搬的根单元仍被目标那一版引用，`retain` 留下它（`crates/singlefs-harness/src/model.rs:2791`：「relocations_carried.retain(|unit, _| referenced.contains(unit));」），于是模型把回退那条根当成与它前一条（搬了根的那一批）同一个状态；实现那一侧回退那条根的树表根指针取目标那一版的字节（`crates/singlefs-core/src/mount.rs:8163`：「target_root_of_the_tree_of_kind(entry.kind)」），与目标版逐字节相同、与前一条不同。

### 复现（拷贝上，量过）

种子下标 476（比重 `with-rollback`）缩到 6 步（`ATTACK_KEEP=0,2,3,5,6,7`）：整理（一批 2 个、推一批后崩、可写挂载；这一步没搬成）→ 整理推到停（txg 7、8、9 三批，txg 6 签发）→ 回退到根环里第 2 新（目标 txg 8）→ 顺序写 149 个单元 → 回退（目标 txg 5）→ 抬 F。`attack_r2_ceiling_dump` 每一步之后打根指针指纹（`i` 后四位是 inode 树根指针字节的指纹，`e` 后四位是 extent 树的），最后一步之前那一行原样：

```
ROOTS 0@0:i无e无 1@1:i无e无 2@1:i无e无 3@1:iea3fe8e0b 4@2:iea3fe8e0b 5@2:iea3fe8e0b 6@2:iea3fe8e0b 7@2:iea3fe2c5f 8@2:iea3fe2c5f 9@2:i85ffe2c5f 10@2:iea3fe2c5f 11@2:ib838ee2d7 12@2:iea3fe8e0b
ENDING NewFinding { signature: ModelDisagreement { aspect: "抬 F 的上限" }, observation: FailureObservation { position: Operation(5), operation_kind: Some(RaiseRollbackFloor), violations: [], panic: None, newest_ring_root_txg: Some(14), root_ring_slot_count: Some(24), harness_judgement: None, model_disagreement: Some(ModelDi
```

（ENDING 一行原样截到 330 字；`model_answer` 那一段原样是 `上限 Some(7)（D16（发布语义） 已定项 1）`、`implementation_answer: "实现报上限 9"`，在模型目录 `out/trace476.txt`。）

读法：txg 7 那一批搬了 extent 树根（`e8e0b`→`e2c5f`），txg 9 那一批搬了 inode 树根（`ea3f`→`85ff`）；txg 10 是回退到 8 那次发布，字节与 8 相同（`iea3fe2c5f`），与 9 不同。实现按字节去重，从新到旧的不同状态是 12、11、10、9，第 4 新是 9，上限 9。模型给 10 这条根的「最后一次搬它」是 {inode 根: 9, extent 根: 7}（9 那一条从现行那一版并进来），与 9 的状态相等，10 不算「非空」，从新到旧是 12、11、9、7，上限 7。

同一个根因的另两格（缩短后同样打指纹，模型目录 `out/dump535.txt`、`out/dump640.txt`）：

| 种子下标 | 缩到几步 | 回退目标 | 目标之后搬了根的那几批 | 回退那条根的指纹 = 目标的？ | 模型 / 实现的上限（缩短后那一段） |
|---|---|---|---|---|---|
| 476 | 6 | txg 8 | txg 9（inode 根） | 是（`iea3fe2c5f`） | 7 / 9 |
| 535 | 8 | txg 12 | txg 13（inode 根） | 是（`ifd0ded7a8`） | 11 / 13 |
| 640 | 8 | txg 10 | txg 12（extent 根）、13（inode 根） | 是（`i7fc4e5548`） | 7 / 12 |

复跑：`ATTACK_ENDING_CONTAINS='抬 F 的上限' ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=476 ATTACK_KEEP=0,2,3,5,6,7 ATTACK_VERBOSE=1 <attack_r2_minimize 二进制> --nocapture`，与 `ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=476 ATTACK_KEEP=0,2,3,5,6,7 <attack_r2_ceiling_dump 二进制> --nocapture`（`tools/rerun.sh` 里各一步）。

### 四句

1. 分不分辨臂：不分辨 F1 的取法（取目标版的位置、照现行版 COW 那几片叶）；分辨的是模型回退那一步「搬过的根」取谁的：只取目标那一版的（与实现的字节一致）对，现行那一版并进来的错。
2. 被判的系统看不看得到：看得到。实现回退时树表根指针整条从目标那一版取；模型手里有目标那一版的 `user_visible_root_relocations`。
3. 满足的是哪一句：正文 R1「回退之后……映射与释放集、复活集是否一致」之外，按第四节攻方立场「后果超出清单」——是对拍判据（理想模型）在合法历史上报不合，不是实现错。修复批自己的 F1 取样组 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`（只给按模型逐版比的用例，`crates/singlefs-harness/tests/rollback_after_compaction_takes_the_mapping_positions_of_the_target.rs` 抽 4 个种子）多抽几个种子就会红。
4. 改法（只在我的模型上量过、被攻过零轮）：模型回退那一步「搬过的根」只取目标那一版的（`TakenFromTheRollbackTarget` 时 `relocations_carried` 换成目标那一版的那一份，不从 `previous` 起抄；拷贝上的改动原样在模型目录 `src/model-rollback-relocations.patch`），实现一侧不改。在打中的格上量过（副本上的数）：4 GiB 上打中过的 476、535、640 三个种子改后 `problems=0`，297 那一格照旧打中（它是 R7 那一形，不是这个根因）；小盘（单元区 384 槽那一档）上比重 `r2-mix` 种子下标 3000–3495 共 496 段，原样模型 12 段出问题（11 段「模型说该拒、实现做成了」——会话为空间推的抬 F 模型判超过上限，1 段「抬 F 的上限」），改后同一批 0 段（`out/model-rollback-relocations-before-and-after.txt`）。小盘上的那 11 段缩了一段（种子下标 3395 缩到 5 步：可写挂载 → 整理一批搬两个根 → 回退到整理之前 → 再整理 → 顺序写 150 个单元），指纹与上表同形（回退那条根 `i76c7e9023` 与目标 txg 4 相同、与它前一条 txg 7 不同，`out/dump3395.txt`）。

什么现象会推翻它：入库装置上同一段 6 步历史抬 F 时模型与实现报同一个上限；或者 txg 10 那条根的树表根指针与 txg 8 字节不同（那就是实现回退没取目标版的根指针，改记到实现一侧）。

### F1 本身：攻了没打中的形状（拷贝上，量过）

`attack_r2_rollback_shapes`，前缀是第一轮攻方种子下标 126 的两次顺序写（152、290 个数据单元），用户决定的两步放开扫（回退之前可写挂载与否 × 回退目标第 0–11 新），回退之后接「覆盖写一个单元 → 可写挂载 → 顺序写 290 → 冷启动」，每一步过池级 checker、与模型逐步比：

| 形状 | 格数 | 打中 |
|---|---|---|
| 目标版之后有过两趟、两条意图的整理（`two-runs`） | 24 | 0 |
| 整理搬过的单元又被覆盖写（`compaction-then-overwrite`，搬过的 key 在现行那一版里查不到） | 24 | 0 |
| 意图半路崩、可写挂载、回退、再整理把盘上那一条推完（`crash-mid-intent-then-resume`） | 24 | 0 |
| 同上，回退之后先覆盖写一个单元、再整理（`crash-mid-intent-overwrite-then-resume`） | 24 | 0 |
| 只有 152 个单元那一次顺序写（`one-write-*`）、起点那个小文件（`single-leaf-*`） | 144 | 0（整理一次都没搬成，不算取样点） |

抽查三格逐步打印（模型目录 `out/rollback-shapes-samples.txt`）：回退到意图半路（目标 txg 7，现行那一版多搬了一批）之后续推，扫描比回退之前多一条（复活回 R 的那个源落点），接着搬完、同一次发布删意图；`two-runs` 回退到第二条意图半路（目标 txg 21）；`compaction-then-overwrite` 回退到整理之前（目标 txg 4），释放 297 个、复活 316 份。都没有红。

另外两组随机历史（比重 `with-rollback` 310 段、`r2-mix` 155 段，每段 40 步，过 checker 的镜像 12636 + 6355 张；R8 那一节另有小盘上的几批）里，除本节与 R7 那两类，没有一处 I-3.x、I-2.x 红；回退那一格的 `ReleaseTargetAlreadyReleased` 一次没出现。

R1 点名的其余几样：
- 回退不动 logged_ops、目标早于签发、现行版带活意图（`crash-mid-intent-*` 回退目标 ≥ 签发之前那几格）：回退之后续推同一条意图，R 排他与盘上那一条对得上（会话每一步开头的 `assert_eq!` 比分配器上的活意图与盘上那一条，24 × 2 格没有一次 panic）；暂停的意图（被空间拒）没造出来，随机历史里一次 `PausedShortOfSpace` 都没有（推的：4 GiB 盘上没压满）。
- 两块盘不对称：落点两盘同槽，搬迁与回退都按落点整体走；不对称只在释放前核出对不上、一份留在已分配时出现，回退那一侧另有一段专管（`resurrection_against_the_current_account` 的隔离单元）。没造，推的。
- 映射树只有一片叶：4 GiB 几何上整理要用户数据铺满几段才搬得动（上表最后一行），这时映射树不止一片叶，没造出取样点；只有一片叶时 `plan_rewriting_the_paths_to_the_moved_entries` 第一圈就落到叶、`break`（读码）。
- 多出来的重写过不过准入：回退那次发布不判式子，固定点那一半照 ckpt_cost 的最坏数；ckpt_cost 里中央映射树那一项每层取 min(这一层的节点数, 删的 + 插的)，删插数至少是分配记录树几何上全部节点数的两倍（两块 4 GiB 盘 529 个节点 ⇒ ≥ 1058），而 4 GiB 上映射树叶层的节点数推不到这么多（数据单元两槽，条目至多约 13 万条，叶半满时约 900 片）。所以多重写的那几条路在这一几何上罩得住。推的，没量；见 R7 同一条。

## R2　腾空 R 的那一批与删意图（F2）

**结论：打中（后果轻，只在一次瞬时读错之下）。** 会话对一次整理发布先算一遍账、核不过换一个 `intent_step` 再算一遍；两遍都照发布路径「释放之前读盘核」把被换下与搬走的单元读一遍。读盘核对一份读不出先重读一次（引文见下），连着两次读不出就把那一份隔离、记录留在已分配（`copies_of_a_released_unit_failing_the_release_checksum_check`）；会话的退路是「核不过 ⇒ 照抄意图再算一遍、不再核」。所以一次只落在前一遍上的瞬时读错，让两条退路在产品路径上都走得到——代码注释说它们只在只供测试的开关下走得到：

- `crates/singlefs-core/src/mounted_session.rs:2573`：「// 最后一批在算好的账上 R 还不按记录全空（只供测试的开关关掉排他、这一批的新落点落进了 R）：这一批不删意图，照搬。」
- `crates/singlefs-core/src/mounted_session.rs:2591`：「// 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过只在只供测试的开关关掉排他时走得到，照这一批不删意图再发」
- 读盘核重读一次：`crates/singlefs-core/src/transaction.rs:3982`：「// 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5」

退路那一遍（`CompactionIntentStep::CarryTheIntent`，`crates/singlefs-core/src/mounted_session.rs:2583`、`:2617`）读盘这回读得出，R 在它算好的账上按记录全空，意图却照抄发出去——正是 F2 要去掉的那一形（判决 G4「目标态到达与最后一批同发布删」），下一次发布再单独删意图。

### 复现（拷贝上，量过）

`attack_r2_transient_read`：内存稀疏盘外面套一层，挂载与前缀都不碰它，从整理开跑那一刻起数读（从 0 起），`ATTACK_FAIL_FROM..ATTACK_FAIL_TO` 里每一个读号单独注入一格（`ATTACK_FAIL_PAIRS=1` 时连同下一个读号），那几次读交 `BlockDeviceError::InputOutput`（原型文件头那句还写着旧的变量名 `ATTACK_FAIL_READS`，以这里为准）。每一格从同一张镜像重开挂载。

**同形另一支（最后一批删意图核不过）。** 前缀：第一个文件 → 顺序写 152 → 顺序写 290；强制 R = 盘 0 槽 50496，一批 3 个（另把开放段占到只剩两槽，见 R3）。最后一批（txg 10）第一遍读盘核读搬走的源落点 50556 的盘 0 那一份时，第 216 次读与它的重读（第 217 次）交错。原样（模型目录 `out/transient-completing.txt`）：

```
TRACE compaction publish txg=9 kind=Batch planned=3 scan_allocations=5 relocated=3 region_start=50496 region_empty_after=false intent_live_after=true
TRACE fallback completing-batch delete refused -> carry
TRACE compaction publish txg=10 kind=Batch planned=2 scan_allocations=2 relocated=2 region_start=50496 region_empty_after=true intent_live_after=true
TRACE compaction publish txg=11 kind=IntentCompletedWithoutABatch planned=0 scan_allocations=0 relocated=0 region_start=50496 region_empty_after=true intent_live_after=false
INJECT 216 reads=383 failed=["216:dev0:slot50556", "217:dev0:slot50556"] outcome=Stopped(NoEligibleSegment) publishes=4 batches=2
```

不注入的那一格是 `txg=10 kind=BatchCompletingTheIntent … region_empty_after=true intent_live_after=false`、`publishes=3`。

**F2 新加的那一支（照抄核出腾空了 R、删意图那一遍核不过）。** 前缀照修复批 F2 用例那 12 步（第一轮攻方种子下标 1055），强制 R = 盘 0 槽 50432，一批 1 个。第四次发布（txg 33）计划的是这次本来就重写的节点；删意图那一遍读 R 里槽 50485 的盘 0 那一份时第 339、340 次读交错。原样（模型目录 `out/transient-1055.txt`）：

```
TRACE fallback delete-refused-after-emptying -> carry
TRACE compaction publish txg=33 kind=Batch planned=1 scan_allocations=2 relocated=0 region_start=50432 region_empty_after=true intent_live_after=true
TRACE compaction publish txg=34 kind=IntentCompletedWithoutABatch planned=0 scan_allocations=0 relocated=0 region_start=50432 region_empty_after=true intent_live_after=false
INJECT 339 reads=514 failed=["339:dev0:slot50485", "340:dev0:slot50485"] outcome=Stopped(NoYieldFromTheLastIntent { empty_cluster_segments_now: 3309, empty_cluster_segments_at_issue: 3309 }) publishes=6 batches=3
```

不注入的那一格：`TRACE carry-unless found the region emptied -> delete`、`txg=33 kind=BatchCompletingTheIntent … relocated=0 … region_empty_after=true intent_live_after=false`、`publishes=5`。txg 33 那一版就是第一轮 T6 的形状（没搬单元、R 按记录全空、意图照抄），F2 的用例经只供测试的开关强制出来的也是它。

### 正文 R2 的几问

- 「这一批之后 R 按记录全空」怎么判：在这一遍算好的账上 `region_is_empty_by_the_records`（读码）。读盘核把一份隔离时，那条记录留在已分配，R 就不空；这一遍算出的「空不空」取决于这一遍读盘的结果，不是这一批的计划。
- 核不过那一支在产品路径上真走不到吗：走得到，条件是一份副本连读两次（读与重读）都读不出、下一遍又读得出。注入的是瞬时读错，不是只供测试的开关；排他照常开着。今天产品路径上整理不起跑（批大小 `NotDecidedByTheClauses`，第一轮 T7），所以今天的产品二进制里碰不到。
- 模型 `(false, true, …)` 臂判不合是实现错还是模型错：R 按记录全空而意图照抄的那一版违反 G4，模型判它不合是对的（`crates/singlefs-harness/src/model.rs:3700`：「(false, true, true | false) => {」；搬了单元的那一版落在 `crates/singlefs-harness/src/model.rs:3664`：「(true, true, true) => {」，同样判不合，推的：本节两格没接模型，直接在会话上跑）。只供测试的开关造出来的那一版本身就违反 G4；退路照抄而不再核，是实现一侧的问题。R 还不空的那一形（`(false, true, false)`）条款没写，见 R7。
- 同形另一支：见上，量过。
- `batches_published`：没搬单元的发布不记成一批（`crates/singlefs-core/src/mounted_session.rs:2068`：「if !version.units_relocated_by_this_publish.is_empty() =>」），随机历史里每一趟整理报的批数都不多于搬的单元数（原型里逐趟比，随机历史 537 段 0 处）。推的、没造：用户改动之后那一趟（`AtMostOneBatchAfterAUserChange`）在一次没搬单元的删意图批之后不停，接着签发、再搬一批才停，一趟发三次；产品路径不起跑，没造。

随机历史里这两条退路一次都没走到：比重 `with-rollback` 带 TRACE 的 155 段（`out/batch6-compaction-publish-kinds.txt`），整理发布 2415 次（照抄的批 2032、删意图的最后一批 330、只删意图 53），F2 那一支与两条退路的 TRACE 行 0 条；小盘 256 槽 `with-rollback` 496 段（也带 TRACE）里 F2 那一支自然走到 2 次（txg 29、33 两趟，计划里一个或几个是本来就重写的节点，同一次发布删了意图，模型对得上），两条退路 0 次。

### 四句

1. 分不分辨臂：不分辨 G4 的读法；分辨的是退路「照抄、不再核」与「照抄、再核一次（核出全空就删）」两种实现。
2. 被判的系统看不看得到：看得到。退路那一遍算好的账就在手里，`region_is_empty_by_the_records` 一判就知道。
3. 满足的是哪一句：正文 R2「核不过那一支在产品路径上真走不到吗」与「同形另一支」字面；后果落在 G4「目标态到达与最后一批同发布删」与模型 `(true, true, true)`、`(false, true, true)` 两臂。
4. 改法（被攻过零轮；推的，没实现没跑）：两条退路都改成 `CarryTheIntentUnlessThisPublishEmptiesTheRegion`（退路那一遍核出全空就再换删意图，至多再算一遍），或者第一遍的读盘核结果（哪几份隔离）带进后一遍、不重读。在打中的两格上：推的，两种改法都修；前一种在「每一遍都读得出不同结果」的设备上仍可能来回，要给上界。

什么现象会推翻它：入库装置上同样注入第 216、217 次（或 339、340 次）读错之后，txg 10（或 33）那一版删了意图；或者读盘核对读不出的那一份在后一遍沿用前一遍的结果。

## R3　签发那次发布的 `segs0`（F3）

**结论：没打中（一次抽样）；有一处推的、没造出取样点的 panic 路。**

- `segs0` 与 G1 定义句：写下的是签发那次发布自己的记账第 11 项（`crates/singlefs-core/src/transaction.rs:10803`：「PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),」，记录口径的全空整段数），停机谓词「全局差值」两边都取记录口径（`crates/singlefs-core/src/mounted_session.rs:2351`：「if !the_completed_intent_yielded(empty_now, completed.empty_cluster_segments_at_issue) {」，`crates/singlefs-core/src/compaction_plan.rs:753`：「empty_cluster_segments_now > empty_cluster_segments_at_issue」）。F3 之前两边差的是「签发之前那一版」与「签发那一版」，现在都是发布之后的账，对得上。
- 两次算账之间换得干不干净：前一遍核不过时分配器整份换回进来时那一份（`crates/singlefs-core/src/transaction.rs:8522`：「*allocator = allocator_before_this_publish;」），只供测试的「签发之后强制一次释放前核出对不上」那个一次性开关也跟着分配器换回去（它的「用过了」记号住在分配器上），后一遍照样用得上；发成之后才把分配器上的活意图换成写下的那一条（`crates/singlefs-core/src/transaction.rs:7727`：「.take_the_intent_written_by_the_issue(handed_in, written);」）。根槽、`B_gc`、`intents_issued` 都在会话那一层按「发成一次」数，换值重算不多算（读码）。
- 量的：拷贝上把开放段占到只剩两槽（照 F3 用例的取法），签发那次自己开新段，TRACE 原样 `TRACE issue segs0 handed_in=3300 accounting=3299 -> recompute`；后一遍读盘核读的是 8 个被换下的固定点单元各两份（第 21–36 次读）。逐个把「一次读与它的重读」一起交错（16 格，`out/transient-issue-pairs.txt`）：16 格都 `outcome=Stopped(NoEligibleSegment) publishes=3 batches=2`，与不注入那一格相同，没有 panic。
- 推的、没造出来：后一遍若因读盘核隔离了一份，而那一份的释放本来会让某一段变成全空，后一遍的记账第 11 项就与前一遍不同，产品代码里那句 panic 会打出来（`crates/singlefs-core/src/transaction.rs:7721`：「"签发那次照记账换过 segs0（{written_by_the_accounting}）再算，记账第 11 项成了 {written_the_second_time}：\」）。文档注释的前提「两次只差 logged_ops 条目里那 8 个字节」（`crates/singlefs-core/src/transaction.rs:7570`：「/// 自己开了新聚簇段时它比记账第 11 项多一。算好账之后两者不同就照记账那个数换掉、再算一次再发（两次只差 logged_ops 条目里那 8 个字节，」）对读盘那一半不成立（R2 那两格就是这个前提在别处落空）。要「签发换下的单元正是某一段最后一条未释放记录」，这一条腿没造出来。改法（被攻过零轮）：后一遍沿用前一遍的读盘核结果，或把 panic 换成交回拒绝。

什么现象会推翻「没打中」：入库装置上签发那次换下的单元有一个是某段最后一条记录、它在后一遍被隔离时，发布照样成、`segs0` 写的是后一遍的记账第 11 项。

## R4　强制开关（F4）

**结论：没打中（读码与 F4 用例，一次抽样）。**

- 平常路径碰不碰得到：开关住在写入口上，新开的写入口照判定（`crates/singlefs-core/src/transaction.rs:368`：「snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,」）；装它的只有 `set_snapshot_lifetime_judgement`（`crates/singlefs-core/src/transaction.rs:452`：「pub fn set_snapshot_lifetime_judgement(&mut self, judgement: SnapshotLifetimeJudgement) {」），`grep -rn set_snapshot_lifetime_judgement crates/` 只命中它自己与 harness 的 F4 用例那一份。产品路径每次发布都新开写入口，碰不到。
- 两臂与 S3 的判定是不是只差判定那一处：是。换的只有 `verdict_of` 这一个结论（`crates/singlefs-core/src/transaction.rs:4859`：「.verdict_of(&mapping_key, previous_snapshot_txg_of_the_live_head)」），之后进 deadlist、留存映射条目、从释放集里排除、读盘核跳过钉住的单元，都按这一个结论走同一段代码。
- 别的副作用：强制进 deadlist 那一臂造得出判定本身造不出的条目——纪元照写活头此刻的前驱（`crates/singlefs-core/src/transaction.rs:4881`：「epoch: previous_snapshot_txg_of_the_live_head,」），没打过快照时是 0、birth 比纪元新。删快照只放纪元落在 [该快照诞生 txg, hi) 的条目（`crates/singlefs-core/src/transaction.rs:4798`：「&& (lo..hi).contains(&entry.epoch)」），纪元 0 的条目以后建的任何快照都放不掉，没快照时回退那一版又照抄现行那一版的 deadlist（`crates/singlefs-core/src/mount.rs:8192`：「// 两版都不带快照（回退之前判过，判决 S8）：deadlist 与留存的映射条目照现行那一版（删完全部快照之后是空的）。」，括注的前提在这一臂下不成立）。只在用了这一臂的用例里出现，推的、没造用例；F4 用例在这一臂之后没有接回退或建快照，不受影响。

什么现象会推翻它：产品路径上有一处调用方装了强制臂；或者强制两臂之下有一处副作用不经 `verdict_of` 的结论（另算一遍判定）。

## R5　空文件集合上的快照（F5）

**结论：核心层与理想模型没打中（量过）；RefFS 那一处的口径与另两处不同，不分辨（推的，翻译走不到）。**

量的（`attack_r2_empty_file_set_snapshots`，起点第一个文件 → unlink 走完 → 建 n 个快照，n = 1–11，之后放开扫一步再接「建快照 → 读快照 → 可写挂载 → 冷启动」，每一步过 checker、与模型逐步比）：99 格 0 打中；第 11 个快照实现报 `PublishError::SnapshotCreationRefused`、模型同拒；读快照读出 `NoFile`。另一组「unlink → 建快照（或建了再删）→ 建 inode → 回退第 0–7 新 → 可写挂载」16 格 0 打中：带快照时回退一律 `SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses(Current)`，删了快照之后回退到删文件之前的那几版做成，模型对得上（`out/empty-file-set-snapshots.txt`）。

RefFS：它的「文件集合为空」只有空池那一形，F5 的改动让空池上建快照照收（`crates/singlefs-reference-filesystem/src/lib.rs:600`：「// 文件集合为空时建快照照收，快照读出「没有文件」（用户 2026-10-16 定「放开」，判决」，单测 `crates/singlefs-reference-filesystem/src/lib.rs:856`：「fn a_snapshot_created_while_there_is_no_file_reads_no_file_after_the_first_file_is_written() {」从 `on_a_freshly_formatted_pool` 起）。核心层放行的只有「有树而文件集合为空」那一形（`crates/singlefs-core/src/mounted_session.rs:1412`：「UserChange::CreateSnapshot => emptied_version,」），树表 0 条的空池照拒（`crates/singlefs-core/src/mounted_session.rs:1419`：「PoolVersion::WithoutFile(_) => {」），执行器在那一形上根本不调（`crates/singlefs-harness/src/history.rs:4643`：「return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);」）；而「有树而文件集合为空」那一形是 unlink 之后才有，翻译到 unlink 就停（`crates/singlefs-harness/src/reference_filesystem_translation.rs:404`：「TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,」）。所以 RefFS 这次改的那一格与核心层放开的那一格没有一段历史能同时走到：RefFS 判的空池建快照，核心层拒、不进 RefFS 的序列；核心层放开的那一形，RefFS 收不到。不会误判，也不判它（推的）。
- 上限 10 数不数它：核心层与模型都数（量的，上面第 11 个拒）；RefFS 不设上限（读码）。
- 建快照那次发布崩在中途：没跑崩溃枚举（不碰 checker 档）；核心层那是一次独立发布，崩了就是那条根在不在；RefFS 收不到这一形（同上）。没造，推的。

## R6　RefFS 的快照允许集合（F6）

**结论：RefFS 单元层上打中一形（量的），今天的翻译喂不到它（推的）；排除读法本身与翻译里的用法没打中。**

### 打中：恢复读出的内容对不上任何状态时，快照集合退回 S_0 那一刻的

`crates/singlefs-reference-filesystem/src/lib.rs:724`：「// 读出的内容一个都对不上（抛弃最新根的那次恢复不判读出的内容）时照取 S_0 那一刻的。」，代码 `crates/singlefs-reference-filesystem/src/lib.rs:740`：「vec![StateIndex(0)]」。抛弃最新根的那次恢复落到最新根的前一条，读出的是「最后一次确认的发布之前」的内容，允许集合 Q 从那次发布起算，读出的内容不在 Q 里——S_0 之后建过、在被抛弃那条根之前就落了盘的快照，RefFS 记成没有。

量的（`attack_r2_reference_filesystem_probe` 第一条，直接在 RefFS 上走）：第一个文件 A + 发布 → 建快照 7 → 覆盖写 B + 发布 → 抛弃最新根的那次恢复读出 A（落到建快照那一版，快照 7 在它的树表里）。原样（`out/reference-filesystem-probe.txt`）：

```
test a_restart_after_an_abandoned_newest_root_forgets_a_snapshot_created_since_the_last_recovery ... BEFORE allowed_snapshot_sets=[{SnapshotName(7): OneFile(FileContent([65]))}]
AFTER allowed_snapshot_sets=[{}]
READ_SNAPSHOT_7_AS_A Err(SnapshotReadOutsideTheAllowedReads { name: SnapshotName(7), read_out: OneFile(FileContent([65])), allowed_reads: [None] })
CONTAINS_THE_SET_ON_DISK false
```

之后读快照 7 读出 A，翻译判红（`crates/singlefs-harness/src/reference_filesystem_translation.rs:644`：「ReferenceFileSystemEvent::ReadSnapshot { name, read_out } => {」那一臂交 `StoppedVerdict::Red`）；删快照 7 RefFS 拒成「没有这个快照」（弃权）。

走不走得到：同一形经历史与翻译走一遍（同一份原型第二条），抛弃根那一步实现拒成 `MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)`，翻译不产出那个事件，票 `Green`（原样在同一份 `out/` 文件里）。执行器的这一步今天造不出被抛弃的根（`crates/singlefs-harness/src/history.rs:517`：「/// C554 乙之后读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），」）；造得出的那一形（系统配置没见证到最新那条根）这一步不造，崩在带读故障的恢复里的 RefFS 不判。所以今天没有一段喂给 RefFS 的历史会走到这一臂（推的）。这一臂 F6 之前就退回 S_0（第一轮冻结副本同一处是 `snapshots_at(StateIndex(0))`），F6 照搬了。

四句：不分辨 F6 的「全部对上的状态取并集」；被判的系统（RefFS）看不到抛弃的是哪一条根，只看到读出的内容——判别子观测不到，要改判据（照「恢复落在最新根前一条」取那一刻的快照集合，或取 S_0 到 lo 之间每一个状态的集合之并）；满足正文 R6「会不会把一个合法的崩溃结局排除掉」字面；改法被攻过零轮、没实现。

什么现象会推翻它：入库装置上翻译能产出「抛弃最新根的那次恢复」事件、而那一形之后的读快照判绿；或者 D23 的崩溃恢复抛弃根只抛弃没确认过的根（那就落不到建快照那一版之前的状态，这一臂不会出现）。

### 攻了没打中的

- 排除读法（实现做了的操作、读快照读出的内容排除说不通的快照集合）：前提按「实现在任何写之前拒」写，拒掉的操作不进序列；进了序列的那一刻实现那一个快照集合满足前提，排除掉的都是实现不可能在的那几个。核心层的删快照在同一次调用里走完意图与全部批，回退要求现行与目标两版都没快照（比 RefFS 只看现行那一版的前提更严，`crates/singlefs-reference-filesystem/src/lib.rs:536`：「Operation::RollBackToVersion(_) => {」），RefFS 只会更宽、不会把实现做成的操作拒掉。读码，没造反例。
- 多个快照集合只在「崩在一次建快照或删快照里」或「写了之后报错」之后才出现；翻译里写了之后报错只给历史停下的最后一步，崩溃判定判完读出的内容就停（第二次崩溃也只比内容），之后没有读快照、删快照、回退交给同一个 RefFS。所以排除读法今天只在 RefFS 自己的单测里走到（推的）。
- 新公开接口在 harness 翻译里的每一处用法：`grep -rn 'allowed_snapshot_sets\|allowed_reads_of_the_snapshot\|judge_a_snapshot_read_and_keep' crates/` 在 RefFS 之外只命中翻译那两行（读快照那一臂：先取允许读出的那几样拼报告，再判、判过收窄）；旧的 `snapshots()`、`read_snapshot()` 没有别的调用方留着。
- 随机历史：比重 `r2-mix` 155 段里读快照 199 次（各件 `ATTACK_KIND` 行加出来的）（读出文件或 `NoFile`），模型逐步比都对得上；这一组随机历史没接 RefFS 翻译（翻译到 unlink 就停，见 R5）。

## R7　第一轮交下来的开着的形状

| 形状 | 结果 | 量的还是推的 |
|---|---|---|
| 计划里的单元都被顺带重写、一个没搬而 R 还不空 | **造出取样点，两个**；模型判不合，执行器报 `NewFinding` | 量的 |
| 多出来的重写越过 `ckpt_cost` | 造出取样点；余量 528–534 槽，不越过 | 量的；「推不到越过」是推的 |
| 零个全空段 | 造不出 | 量的（最小值）+ 推的（为什么） |
| 合法镜像触发 `TwoRecordsMovableUnderOneMappingKey` | 造不出 | 量的（0 次）+ 推的 |

### 计划里的单元都被顺带重写、一个没搬而 R 还不空

取样点甲：比重 `with-rollback`、种子下标 297，缩到 20 步（`ATTACK_KEEP=0,1,2,5,8,11,12,14,15,16,18,19,20,21,22,23,24,25,26,27`），最后那一趟整理（R = 盘 0 槽 51904，一批 1 个）原样（`out/trace297.txt`）：

```
TRACE compaction publish txg=72 kind=Batch planned=1 scan_allocations=3 relocated=1 region_start=51904 region_empty_after=false intent_live_after=true
TRACE compaction publish txg=73 kind=Batch planned=1 scan_allocations=2 relocated=0 region_start=51904 region_empty_after=false intent_live_after=true
TRACE compaction publish txg=74 kind=BatchCompletingTheIntent planned=1 scan_allocations=1 relocated=1 region_start=51904 region_empty_after=true intent_live_after=false
```

txg 73 计划的是槽 51965 上一个码 2 节点（`TRACE   planned=[PlannedRelocation { mapping_key: [2, 2, 0, 1, 0, 0, 0, 0, 0, 62, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0], source: Placement { slot: SlotNumber(51965), span: 1 } }]`），这次发布本来就重写它、不另搬（`crates/singlefs-core/src/transaction.rs:9172`：「// 这次重写本来就换下的角色不另搬：它的旧落点经 COW 释放、新落点不在活 R 里（判决 G3 的一批只搬「照抄」的单元）。」），R 里还剩槽 51967。模型判这一版不合（`crates/singlefs-harness/src/model.rs:3702`：「"活意图 {live:?} 在：一次整理发布要么搬一批、要么删掉意图"」），执行器 `NewFinding { signature: ModelDisagreement { aspect: "这一次整理发布对意图做的事" } …}`，实现那一侧原样 `搬了 0 个单元（其中用户可见树的根 []），发布之后 logged_ops 里有活意图，活意图的 R 按记录不空（或没有活意图）`（`out/random-problems.txt`）。

取样点乙：小盘（单元区 240 槽那一档）、比重 `r2-mix`、种子下标 6318，第 38 步那一趟整理 `TRACE compaction publish txg=40 kind=Batch planned=2 scan_allocations=4 relocated=0 region_start=1664 region_empty_after=false intent_live_after=true`，同一句判不合（`out/seed6318-small-device.txt`，改过模型回退那一步的副本上跑的，与 R1 那个改法无关）。

出现率（副本上的数）：比重 `with-rollback` 4 GiB 上 382 段里 1 段，小盘 240 槽 `r2-mix` 372 段里 1 段；带 TRACE 的 4 GiB 那几批（`with-rollback` 155 段、`r2-mix` 155 段）一次没碰到。修复批 F1 用例给这一组比重只抽 4 个种子，多抽就会撞上。

四句：分不分辨臂——分辨「本来就重写的那几个算不算搬」两种读法（G3「一批只搬照抄的单元」与模型「一次整理发布要么搬一批、要么删意图」在这一形上冲突），条款没写；系统看得到（发布路径先算出这次重写哪些角色）；满足的是正文 R7 第一形字面，后果是 harness 的随机历史红，超出「条款没写、没有取样点」这一句；改法（被攻过零轮、没实现）：计划一批时跳过本来就重写的那几个、照批大小补别的可搬单元，或者模型把「这一版 R 里的未释放记录比上一版少」也算一批，二选一要条款定。

### 多出来的重写越过 `ckpt_cost`

取样点（`attack_r2_transient_read` 的 `fixed_point_slots_of_the_compaction_publishes_and_the_rollback_against_the_checkpoint_cost`，前缀两次顺序写，强制 R = 槽 50496、一批 3 个或 1 个，之后回退到整理之前或签发那一版）：每次发布写出的固定点那一族（分配记录树、中央映射树、记账树、树表、谱系、退役）新槽数与它接在后面那一版的 `checkpoint_cost_of_the_version_to_build_on` 比。原样摘录（`out/checkpoint-cost-margins.txt`）：

```
MARGIN compaction txg=9 relocated=3 fixed_point_slots_written=14 ckpt_cost=545
MARGIN rollback txg=11 target_txg=7 fixed_point_slots_written=17 ckpt_cost=545
```

整理每一次发布 11–14 槽，回退那一次 17 槽，ckpt_cost 545。余量大是因为 ckpt_cost 里中央映射树那一项每层取 min(这一层的节点数, 删插数)（`crates/singlefs-core/src/admission.rs:1440`：「.checked_add(nodes_at_this_level.min(changed_paths) + splits_at_this_level)」），删插数至少是分配记录树几何上全部节点的两倍，在这一几何上等于整棵映射树。要越过，映射树某一层的节点数得多于约 1058（4 GiB 两块盘），推的：4 GiB 上一个文件的映射条目推不到这个数。

### 零个全空段

造不出。带 TRACE 的随机历史里记了每次签发之前那一版按记录口径的全空段数：小盘 240 槽 `r2-mix` 372 段、256 槽 `with-rollback` 496 段的最小值都是 11，384 槽 `empty-file-set` 372 段最小 14，4 GiB 上约 3300。这几档盘上准入在记录罩满每一段之前就拒了（256 槽那一批里 `UserChangeRefused::NoSpaceAfterRaising…` 3854 次、签发被空间拒 855 次），单元区里环窗口补足的那一段一直空着（推的：`HistoryDeviceWidth` 的注释写了补足之外才是名字里的槽数）。合法历史里要让每一段都有一条未释放记录，得绕过准入直接往分配器上占槽（F3 用例那种做法），那就不是合法镜像了。

### 合法镜像触发 `TwoRecordsMovableUnderOneMappingKey`

造不出。拷贝里在这一处拒（`crates/singlefs-core/src/region_scan.rs:261`：「return Err(RegionScanRefusal::TwoRecordsMovableUnderOneMappingKey { device, slots });」）与会话记扫描拒绝那一处各加了一行 TRACE：带 TRACE 的三批随机历史（4 GiB `r2-mix` 155 段、240 槽 372 段、384 槽 372 段，签发 915 次）扫描拒绝 0 次。推的：一把映射 key 只对一个单元，落点两盘同槽，搬迁的目的地排除活 R；回退复活源落点时释放目的落点，被释放的记录不算可搬；释放前核出对不上而留在已分配的那一份判「隔离」不判「可搬」。同一块盘上两条未释放记录同 key，只有映射本身坏了才会有。

## R8　第一轮没打中的格再抽一次样

换的攻击面（第一轮攻方的取样办法是两块 4 GiB 盘上、它自己的两组混合比重与 `SNAPSHOT_OPERATIONS` 原样、种子 `+ 7_000_000` 起的随机历史，加回退那一格的写死前缀）：

1. 小盘：单元区 384、256、240 槽那三档（`HistoryDeviceWidth`），空间紧，会话为空间推抬 F、签发被空间拒都走得到；
2. 新比重与新种子段：`r2-mix`、`empty-file-set`、修复批的 `with-rollback`，种子 `+ 8_000_000` 起；
3. 瞬时读错：可写挂载与整理的发布路径上逐个读号注入（R2、R3 那一份原型）；
4. 写死扫的：删完之后的快照（R5 那 115 格）、回退的几种形状（R1 那 240 格）。

随机历史一共 2924 段、约 12 万张镜像过池级 checker（`out/random-summaries.txt`、`out/random-summaries-small-devices-and-later.txt`）。其中 1033 段在原样模型上跑，1891 段在改了模型回退那一步（R1 的改法）的副本上跑——后者是为了让 R1 那个误判不把别的格盖住，写在每一格里。出问题的只有 R1 那一类（原样模型上 15 段）与 R7 那一形（2 段）。

| 第一轮的格 | 这一轮换的攻击面与样本 | 结果 |
|---|---|---|
| T1 建快照 | 384 槽 `empty-file-set` 372 段（建快照 4028 次、读 1163 次）；小盘 `r2-mix` 868 段（建快照 3610 次，空间紧时会话先推抬 F 再建）；删完之后建到第 11 个被上限拒，实现与模型同拒（R5） | 没打中 |
| T2 释放分流与 deadlist | 同上几批里快照下覆盖写、删文件、删快照交错；释放前读盘核遇上「一次读与重读都读不出」时把那一份隔离（挂载那一扫的 36 格，见 T10 一行），之后的发布与 checker 都没红 | 没打中 |
| T3 删快照 | 小盘上删快照 3337 次（384 槽 `r2-mix` 改模型那一批、240 槽、384 槽 `empty-file-set` 三批加起来），删快照之后回退到删文件之前（R5 那 16 格） | 没打中 |
| T7 R 排他与合取 7 | 小盘上签发被空间拒 855 次（256 槽那一批）、整理一次都没暂停（`PausedShortOfSpace` 0 次）；瞬时读错下两条退路走得到（R2），会话每一步开头核「分配器上的活意图 = 盘上那一条」的断言一次没响；零个全空段造不出（R7） | 没打中新的；与 R2 同一处的读法写在 R2 |
| T9 区域扫描与计划 | 带 TRACE 的三批签发 915 次，区域扫描拒 0 次；「本来就重写的不另搬」与模型冲突的那一形造出了取样点（R7 第一形） | 没打中新的；R7 那一形是第一轮 T9 自己报的「核与发布路径差的那一条」的后果 |
| T10 恢复读侧 | 意图半路崩、可写挂载之后的镜像再可写挂载一次，挂载期间 1814 次读逐个注入：单次读错 1824 格（含读号越界的 10 格）全部与不注入那一格相同；「一次读与它的重读」一起错 1824 格，1772 格相同、16 格挂载拒（系统配置读阶段之后仍读不出 8、单元读不出 5、选中那一版有盘读不到 2、实例表 1）、36 格挂载做成而分配记录指纹不同（挂载那几次发布释放旧节点时读盘核两次都没读出、照设计隔离那一份），选中的根、txg、活意图三样都没变；小盘上冷启动 2318 次、可写挂载 7438 次 | 没打中 |

第一轮攻方 T4、T5、T6、T8、T11 不在这一格里。

T10 那一扫的原样在 `out/mount-transient-reads.txt`（每一类各摘几格，与汇总行）。

什么现象会推翻这一格的「没打中」：同样的取样面（小盘、新比重、读错注入）再抽一次样出现了实现一侧的红；或者 T10 那一扫换一张镜像（带快照、带删除续做）出现「挂载做成而选中的根或活意图变了」。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这一格的「没打中」还要再抽一次才算。

## 没打中的形状

- F1：写死形状 240 格（两条意图之后回退、搬过又覆盖写、意图半路崩之后回退再续、回退之后先写再续、只有一次顺序写、起点小文件），回退目标第 0–11 新 × 回退之前挂不挂，全部走完、checker 不红、模型对得上；后两类整理一次都没搬成，不算取样点。
- F2 本身那一支（照抄核出腾空 R、换删意图）：带 TRACE 的随机历史里只在小盘 256 槽那一批走到 2 次，修复批 F2 用例那 12 步前缀上走到 1 次；不注入读错时三次都做对了。
- F3：签发自己开新段那一格换值重算，后一遍上 16 种「读与重读都错」都不改记账第 11 项。
- F5：删完之后建 1–11 个快照 × 9 种后续，及建 inode 之后回退第 0–7 新，115 格。
- F6：排除读法在翻译里只走得到单个快照集合的那一形；读快照 199 次（4 GiB `r2-mix`）对得上。
- 瞬时读错：可写挂载期间 1814 次读逐个注入单次错（全同），逐个注入「读与重读都错」（没有一格换了选中的根或活意图）。
- 随机历史：2924 段，比重 `with-rollback`、`r2-mix`、`empty-file-set`，盘宽 4 GiB、384、256、240 槽，种子 `+ 8_000_000` 起，每一件的起始下标与段数在 `out/random-summaries*.txt` 每行的 `seed_start=`、`seeds=`。

## 这条腿自己的限度

- 判据是池级 checker、理想模型与（R6 那一条）RefFS：它们一起漏的我看不见；R1 那一格本身就是判据一侧的误判，它在小盘上压着别的格，后半程的随机历史是在改了模型那一处的副本上跑的（每一格写了用的是哪个）。
- 瞬时读错的注入是我在内存盘外面套的一层，按读号注入；「读与重读都错、下一遍又读得出」在真盘上多常见没量。
- 根因都是按代码推的，只在拷贝里加了只打印的 TRACE；改法只有 R1 那一处在副本上改过、跑过，别的都没实现。
- 每段随机历史只记第一处问题；一段里第一处之后还有什么没看。
- 「没打中」都是一次抽样，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不能单独支撑结论。

## 没做什么

- 没调崩溃枚举函数、没跑 checker 档、没起门禁、没起 GPU；R5「建快照那次发布崩在中途」与 R6 崩在建快照、删快照里的那几形没有跑崩溃点（只读码）。
- R4 强制进 deadlist 那一臂之后接回退或建快照的那一形没造用例（推的）。
- R3 那一处「后一遍读盘不同就 panic」没造出取样点。
- R7「零个全空段」「两条同 key 可搬记录」没造出合法镜像。
- 复跑脚本 `tools/rerun.sh` 整串没从零跑过：prepare（拷冻结副本、打补丁）只做了 `patch --dry-run` 核两份补丁打得上；build 与之后各步在我已编好的拷贝上用 `RERUN_REUSE_TREE` 分四次跑过（前台 4 分钟时限），15 份产出与 `out/` 里的逐字节相同（`reference-filesystem-probe.txt` 两边去掉计时那一行）。整串交主 agent 跑，命令在下一节。
- `out/random-*.txt`、`out/batch6-*.txt`、`out/seed6318-*.txt` 是随机历史批跑的摘录，复跑脚本不重跑那几批（一批 1–3 分钟、几十批）；要重跑用 `src/run_random_batch.sh`，比重、盘宽、起始下标在各节写了。

## 要主 agent 跑的命令

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r2-opus-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev \
  --frozen ~/.local/share/singlefs/work/m4-segment-code-r2/repo \
  --work <新的空草稿目录>
```

工作目录取仓根；线程数取环境变量 `RERUN_THREADS`（默认 31）。release 从零编一遍再跑各步，推的约 10–15 分钟。跑完看末尾 16 行：15 行 `name=rerun_compare file=<名>.txt identical=true`（其中一行带「两边去掉计时那一行」）与一行 `name=rerun_total same=15 differ=0`。model-fix 那一步跑完，草稿目录里那棵拷贝的模型是改过的。

## 草稿目录清理

交回前删掉了草稿目录 `~/.local/share/singlefs/work/m4-segment-code-r2-attack/` 里的仓拷贝 `repo/`（删前 `du -sh`：编译目录 `repo/target/` 1.7G、`repo/crates/` 29M）、存原样模型二进制的 `bins/`（297M）与复跑试跑的 `rerun-test4/`（2.4M）。留下的是日志 `logs/`、原型与补丁的工作稿 `attack/` 与 `progress.md`，共 15M；原型、补丁与复跑脚本已原样进模型目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-segment-code-r2-opus-output.md` 末行：

```
  ✓ 核了 37 处引文、6 处裸行号，对上 37 处，没判 6 处（其中裸行号没有引文可比 6 处；逐处列在上面）
```

没判的 6 处是下面三个位置在正文与本节各出现一次：`crates/singlefs-core/src/mounted_session.rs:2583`（与 `:2617` 一起点名退路那两处，没抄引文）、`crates/singlefs-reference-filesystem/src/lib.rs:600`、`crates/singlefs-core/src/transaction.rs:9172`（后两处的引文里套着「」，脚本没认出来；我用 `grep -nF` 在冻结副本里核过各命中一次，行号就是这两个）。
