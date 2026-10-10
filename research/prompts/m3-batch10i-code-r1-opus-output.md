# 根环实现第 10i 批代码轮第一轮：云端攻方（Opus）报告（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 S1 -->

攻击面：正文第三节 K1、K2，重点 K1。立场：假设第 10i 批的新类 `RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses` 是错的，在冻结副本的拷贝上造反例。只跑 harness 档；副本上的数不是入库装置上的数。代码行号一律引冻结副本 `/tmp/claude-1000/m3-batch10i-code-r1/tree/`。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 S1 被抛弃根掉出影子账 | **打中（安全性）** | 抬 F 那一遍读坏、重读仍坏的 S1 被抛弃根不进影子账重算，它引用的单元被回收、发出去，I-7.4 红；三个起点 54 段里 4–8 段红（第 10i 批之前 0 红但全拒，零轮改法 ledger 0 红） |
| K1-2 S1 被抛弃根卡住准入上限 | **打中（代价比必要的大）** | 被抛弃根不进上限，S1 被抛弃根读坏却把上限钉在它自己的 txg，54 段里 42 段 ENOSPC 停摆（exact 臂 0） |
| K1-3 S1 有效根卡住准入上限 | **代价量出来** | 384 槽盘上 S1-V 一直读坏的段 35%–56% 仍停摆（第 10i 批之前更多）；照记着的根算（asread）0 停摆 |
| K1 F 越过 S1 有效根、回收门槛越过有效根 | 没打中（一次抽样） | 五个起点约 1176 段 0 红；能住的最大 txg 大于 V 的两种盘面没造 |
| K1 解开路径 | 没打中（读码） | 写掉那一槽就换类；S1 不经 E2 重读，读得出就照读出来的算 |
| K2-1 回退与抬 F 对同一槽处置不同 | **打中（前后不一致）** | 同一次挂载里抬 F 照钉住推过某一 S1 槽，随后回退因同一槽拒（ab-384-3-5 第 6 步 132 段、mat-384-30-8 第 12 步 79 段）；「本可以安全回退」是推的 |
| K2-2 F_生效 | 没打中（读码） | 抬 F 那一串的 F_生效 读坏当没有根、不拒；拒的只有回退那一份 |

## 复跑

文中 `out/…` 都指模型目录 `research/prompts/m3-batch10i-code-r1-opus-model/out/`。数都是冻结副本拷贝上的数。

```
bash research/prompts/m3-batch10i-code-r1-opus-model/rerun.sh setup <空的草稿目录>   # 拷冻结副本、打 mount-arms-i1.patch 与 workspace-member.patch、release 编译
bash research/prompts/m3-batch10i-code-r1-opus-model/rerun.sh list                 # 33 个阶段与各自的环境变量
bash research/prompts/m3-batch10i-code-r1-opus-model/rerun.sh run <草稿目录> <阶段>  # 经 run-with-memory-cap.sh 3G 与 capped.sh 15
bash research/prompts/m3-batch10i-code-r1-opus-model/rerun.sh compare <草稿目录>    # 逐份比 OPUSI1-RUN / OPUSI1-SUM 行
python3 research/prompts/m3-batch10i-code-r1-opus-model/agg.py <日志…> --by=arm,pattern,red,classes [--show-checker=N]
python3 research/prompts/m3-batch10i-code-r1-opus-model/k2agg.py <k2 日志…>
```

模型目录每个文件的 sha256：

```
8958d810690a02270d0fb481279759dab6400e6aab79ef8879a1dc96c20bf75d  Cargo.toml
3c3709f1c712ef02d92e68eb914c726affd20a95944eccee242cf60e37a1e75a  main.rs
7ae6325fb44d71191f26aa01104a8bb960992938e1cd28618e1d05ee210c055a  mount-arms-i1.patch
8f96b8f27b81e8c2bdd798cb049612aa64178553819f8419ac5ae0d7a22ba7fb  workspace-member.patch
2bc9beca432a92d852f64c6efa44fceedf0c1245cbc44c297544dfd060b7870a  rerun.sh
4585df11f61eb9e7f331a67cc3df2f810927aee9a276283df048cfefd59331dc  agg.py
518ce700dafa9c5fdd11123af4dc182eb9cf5f56c12dccf9531dd48ce52a6c98  k2agg.py
07b8847ac3fcd6c2dd298c61603981618a3432a89087e093b89901d2d5ad2166  agg-output.txt
8b03227f6be1dd6cf28b87162118320e62fad7b885d079346bd60f78edfb9264  out/ab384-owu.log
1398e6470369b7ff905f7c17bb6be3e6e9b37cf49fc956701cc5b5d03c4eb1f8  out/base-prod.log
520768b3cfcf458274d9ec622ddfc5b3f8c7d916c90e4b83a95b9c900034e732  out/k2-prod-3.log
55f82277b7f9ca28030f80971aa41ad2600ea9308972177bc8205e0a63e1ef79  out/k2-prod-s12.log
095f23e6d5e367f6f1493eef768ac7c383e9a15472383118b314048b9986641f  out/k2-prod-s15.log
07cbaa635cc136aedd077329c3433ba7b4051fe4e2769922bed8e14bce2f4b86  out/k2-prod-s6.log
382d93f3c9cc92db716734dceb3ca93f3f540c0839931d6b8c2ce268bd3abc48  out/k2-prod-s9.log
a0fb33d2e8de37765c828cb9c61bbdfba2ea3d7376b923511d8774e3bc1fc4bb  out/k2-refuse-3.log
388f0bf6d0f88bd25d435f31ad1eeb229ba8e6455a7f8e6e76022eba94d60365  out/live-asread-ab384.log
217310e06ff2b3240ec94a9b1b7d0eaef7f526852b492df61c5f9511bb88bb4f  out/live-asread-mat-384-30-8.log
dd10c4f4c3658d9f21de63b3790ff12977a89d38da2137605128896feff38cda  out/live-exact-ab384-A.log
0e84d0ffd3d3a54ca616520144962ad127e66df8a8b9d7a5def31a6af676046a  out/live-exact-ab384.log
a6d9a0b0e65248661f2b161ba376ee09dbe0509581a304d688315aed22ec6d2e  out/live-exact-mat-384-30-8.log
33d1eb5f78cf9f7b575af98dfbba1b06e533e3209bcfbd6457649414e30cf70c  out/live-exact-narrow-264-6-4.log
14923acab02972232f2441d77f1e757d22834324aeb27b0c600d711c437060f9  out/live-ledger-ab384.log
e9e4183ea72d1f30e83b79ca99c50275c57d59538e302fbee94244533910a9f4  out/live-prod-ab384.log
bd8f52e4c831ac2a6ee57c0904e5bc0b45efb14a965fc070eafffcb212e12a33  out/live-prod-mat-384-30-8.log
cf016ab75c978e43e8515a2eac86cc24652b4bbfe0ac4575e6da26cab6a9a0ba  out/live-prod-narrow-264-6-4.log
0c96468c273dba7ce8c6132f687bb32a1dfae9c8239ca8a8d603698c9d8511d3  out/live-prod-narrow-384-20-2.log
b08bee9ecabf74c3f614ebbe020df4400b568bb0907fc6ab4f781f613f75f930  out/live-refuse-ab384.log
d5f490c52e20db8b8562cd1709de9be3663bcc5bc7106b290c1581bb0d043753  out/live-refuse-mat-384-30-8.log
03b71e86ed5d76709492896bbf2ce91c619305b3f9dfb8d70b96136caf0b4ab9  out/live-refuse-narrow-264-6-4.log
9c278f3f754d9960114d636b38aed6dc66974802965059f86494571a91b63132  out/scan1-asread-u8-s6.log
81c34a191a569dc612b9623b70e9275b4ff8a5a3514eb88c6d8293af0ba09c5f  out/scan1-exact-u8-s6.log
31a65b7d79ad2c08fc1e7b5e31db4205c86d95ce51d53902ab3e7e18b3dd9d1e  out/scan1-ledger-u8-s12.log
2a94e8504b1ecaa4ce05f8c6a20939f2898777f07ab4fbb85f416765c90921f0  out/scan1-ledger-u8-s15.log
aaafeb935befc6125118ed20f1cefd395e91f3e25554a92d28286fd630411ded  out/scan1-ledger-u8-s6.log
fc9c3d219a8781b199705c7867dbbb03cce31f4f155a05bb30533420ea049ab0  out/scan1-ledger-u8-s9.log
95b901583a5ecd1da59579e0c946231f96bf86df16e0ac90a72b774443005075  out/scan1-prod-u8-s12.log
7e05a6d89b46385c54e4db2ff6174e8849acc88cfcaaa407e807fc9fade2fc5e  out/scan1-prod-u8-s15.log
c09f60520947d5cc6b9f66ab79ef2094be41a186a86cfbabeac84986f3721b39  out/scan1-prod-u8-s6.log
94cec47eaffca057a48304938dd8970ce6e5a0704c314e52e10d9208a83fc6e9  out/scan1-prod-u8-s9.log
7e78b5459615ca7361f33d7d4358dadd6936d779d716c4a530d0a6402b748ecb  out/scan1-refuse-u8-s6.log
```

## K1　新类撑不撑得住

### K1-1　打中：抬 F 那一遍把「两遍里只有一遍自证过」的被抛弃根读坏时，它掉出影子账，它引用的单元被回收、重新分配（I-7.4 红）

**历史**（起点 `ab-384-3-0`，模型 `main.rs` 的 `abandoned_start(UnitAreaOf384Slots, 3, 0)`）：实例 1 写第一个文件、覆盖写 3 次；崩了再挂（实例 2）覆盖写 2500 字节得根 C（实例 2、txg 9，槽 区域 0 / 槽 3，在盘 0）；藏起 C 的根槽与数据单元、系统配置轮换停在 C 之前，可写挂载（实例 3）择根落到 C 之前那一条；把藏的字节写回，进程退出。这时 C 在环里、按最新根的实例表被抛弃。

**故障与挂载**（实例 4）：重建分配器读根环那一遍是每个槽的第 k = 6 次读。第 k 次读盘 0 上每个住着有效根的槽都报读错（盘 0 一条不被抛弃的自证有效根都没读出 ⇒ 整环重读）；C 那一槽第 k 次读得出、自证过，第 k + 1 次起读回全零（或乱字节），只有 (n − k) 是 3 的倍数的那几次读得出；进程写过的槽不再注故障。挂载结局 `Ok(reread)`，C 那一槽并成 `RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { root: AbandonedRoot { .. } }`（挂载时 C 读得出，影子账照常隔离它独占的单元）。

**用户动作**：覆盖写，第 s 步起每步改成顺序写 8 个数据单元（s 取 6、9、12、15 扫了四档，见下表）。第一次准入抬 F（0 → 17 或 21）时，算上限那一遍第一次读 C 那一槽读得出（被抛弃根不进上限，上限不被 C 那一槽卡），抬 F 那一遍读它两次都读坏。

**每一步许可它的代码**（冻结副本行号）：
1. `crates/singlefs-core/src/mount.rs:3338`：「let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =」起，抬 F 那一遍「读坏重读仍坏就拒」的那一组把两遍里只有一遍自证过的槽拿掉；读坏的这一类只记进 `ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad`（`crates/singlefs-core/src/mount.rs:3361`：「let mut ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad =」），不进交回的根表。
2. `crates/singlefs-core/src/mount.rs:3563`：「} = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;」拿到的 `roots` 里没有 C；`crates/singlefs-core/src/mount.rs:3577`：「match isolate_slots_referenced_only_by_abandoned_roots(」只在这一份 `roots` 里找被抛弃根（`crates/singlefs-core/src/mount.rs:1515`：「for root in roots.iter().filter(|root| is_abandoned(root)) {」）。挂载那一刻 C 独占之外、又被当时的候选根或现行账引用的单元（这里是盘 0 / 盘 1 槽 1536–1537 的实例表单元）挂载时被豁免没隔离；新 F 把那条候选根压到 F 之下、现行账也已释放它，按新 F 重算时只剩 C 引用它，而 C 不在 `roots` 里，没补隔离。
3. 回收门槛 `reclaim_floor` 取 max(F, 环里最旧有效根)（`crates/singlefs-core/src/mount.rs:1410`：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」），释放代 ≤ 新 F 的照回收；抬 F 那一遍的回收 `crates/singlefs-core/src/mount.rs:3601`：「.reclaim_released_up_to(」不核被抛弃根的引用（进程内记根那一处核：`crates/singlefs-core/src/allocator.rs:2048`：「self.isolate_reclaimed_slots_an_abandoned_root_still_references(」）。
4. 带新 F 的根落满每块盘后扣住的放开，接下来的顺序写把槽 1536 发了出去，C 还在环里。

**checker 原文**（副本上的数；模型目录里 `python3 agg.py out/scan1-prod-u8-s6.log --by=sp --show-checker=1` 打出的那一行，checker 的违例串原样取自日志）：
```
OPUSI1-CHECKER ab-384-3-0 prod s1-A@0/3(d0)-trig0-c-Z step=9 F=17 txg=24 classes=["I-7.4"] broken>=F=[] broken_abandoned=[] v=["I-7.4: 被抛弃的根（实例 2、txg 9）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：实例表单元 在盘 0 槽 1536 的那一份与位置条目里的校验和对不上；实例表单元两份都读不到对得上的"]
```
违反 `.claude/kb/decisions/23-journal的角色与格式.md:390`：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。冻结副本自己的注释写着这一拒挡的就是它：`crates/singlefs-core/src/mount.rs:3556`：「这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按……往下走，那一槽里的被抛弃根引用的槽」。第 10i 批前这一类走这一拒，第 10i 批把被抛弃根那一半也一起放过了。

**三臂同一段历史**（三个起点 `ab-384-1-0`、`ab-384-3-0`、`ab-384-3-5` 上全部 S1-A 计划，每起点 18 段，共 54 段；用户动作只按步数切换、不看臂；副本上的数）：

| 第几步起顺序写 8 单元 | 今天（10i） I-7.4 红 | 今天只有潜在漏洞（单元可发、还没被发） | 第 10i 批之前（refuse 臂） | 零轮改法（ledger 臂） |
|---|---|---|---|---|
| 6 | 8 | 0 | 0 红，54 段全报 `FloorRaiseFailedWhilePushingForSpace` | 0 红 |
| 9 | 0 | 8 | — | 0 红 |
| 12 | 4 | 4 | — | 0 红 |
| 15 | 4 | 8 | — | 0 红 |

「潜在漏洞」是我自己的判官（`latent_abandoned_free`：根环表里记着的被抛弃根引用的落点此刻在分配器里可发），正确实现上恒 0；ledger 臂四档 216 段一次都没出。refuse 臂只跑了第 6 步那一档。

**只在读坏交替那一形（计划 c）出**：计划 a（重读起一直坏）、b（第一遍坏、重读读得出、之后一直坏）里算上限那一遍也读坏 C 那一槽，上限被钉在它能住的最大 txg = 9（C 自己的 txg），F 到不了 X 的释放代，碰巧挡住；交替读坏让算上限那一遍读得出、抬 F 那一遍读坏，挡不住。卸载那一串不判上限（`crates/singlefs-core/src/mount.rs:3532`：「卸载抬到现行那一版的 txg，不另判上限」），同一组起点上「覆盖写 30 次再卸载」的 a、b 两形 36 / 36 段卸载之后潜在漏洞 4 个槽（`out/ab384-owu.log`）；进程随即退出、下一次挂载重读环会重新隔离，这 36 段 checker 没红（卸载那一串自己的空发布没发到那几个槽）。

**四句**：
1. 分辨臂：分。今天的实现红、第 10i 批之前不红（但拒抬 F）、ledger 臂不红且拒的段数与今天相同（下表 K1-2 的停摆两臂一样）。
2. 被判的系统当时看得到：看得到。分配器根环表那一项带着 C 引用的全部落点（`RootReadOnOneRebuildPass::AbandonedRoot { referenced_placements }`），抬 F 那一遍也知道这一槽这一遍读坏了；ledger 臂用的就是这两样。
3. 满足的分句：正文 K1「有没有一形……让被抛弃的根掉出影子账」；D23 第 390 行那一句。
4. 跑前条款的改法：正文第五节只写「改法进下一批」，没给候选，无从逐个核。

**我提的改法（只在我的模型上量过、被攻过零轮）**：抬 F 那一遍读坏、重读仍坏的 S1 槽，若根环表记着的是被抛弃根，就按它记着的落点补隔离（现行账里还分配着的除外），再回收；或者干脆只放过有效根那一半，被抛弃根那一半照旧走「重读仍坏就拒」。前者是 ledger 臂（模型补丁 `mount-arms-i1.patch` 里 `OPUS_I1_ARM=ledger` 那一段），后者没实现、推的。

### K1-2　打中（代价比必要的大，不是安全性）：住被抛弃根的 S1 槽读坏时照样把准入上限钉在它能住的最大 txg，F 卡在那条被抛弃根自己的 txg，挂载里之后 ENOSPC

**机理**：`rollback_floor_ceiling_of_this_process`（`mount.rs` 里 `ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator)` 那一段，交 `highest_txg_it_can_hold()`）不分根环表那一项是有效根还是被抛弃根，一律按能住的最大 txg 并进上限；能住的最大 txg 又并进了读出来那条根的 txg（`txgs_it_can_hold_if_read_bad`，`mount.rs` 里 `.max(root.checkpoint_txg)`）。上限只在有效根上取（D16 已定项 1「抬 F 的上限」那一行「「有效根」「非空」在环里全部的根上取」），被抛弃根在任何一遍都不进上限；进程挂载时已经按最新根的实例表判出它被抛弃，这一槽读坏时「可能是一条非空有效根」不成立。

**量的**（同 K1-1 那三个起点、每起点 18 段 S1-A 计划，覆盖写 40 次，副本上的数，`out/live-*-ab384*.log`）：

| 臂 | 停在 `NoSpaceAfterRaisingTheFloor`（`FloorAlreadyAtTheCeiling`） | 停在 `FloorRaiseFailedWhilePushingForSpace` | 40 次都写完 | 红 |
|---|---|---|---|---|
| 今天（10i） | 42 / 54（F 停在 8 或 9 = C 自己的 txg，第 8、14、15 步起） | 0 | 12 | 12 段潜在漏洞（K1-1 那一形） |
| refuse（第 10i 批之前） | 0 | 54 / 54 | 0 | 0 |
| ledger（K1-1 的零轮改法） | 42 / 54 | 0 | 12 | 0 |
| exact（ledger 再加：S1 被抛弃根不并上限、S1 有效根按它自己的 txg 并与钉门槛） | 0 | 0 | 54 / 54（F 到 38–74） | 0 |

`FloorAlreadyAtTheCeiling` 原样（模型目录 `out/live-prod-ab384.log` 里计划名带 `s1-A@` 的行，`grep 'plan=s1-A@' | grep -o 'stop: FloorAlreadyAtTheCeiling …' | sort | uniq -c` 数出来）：`28 stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(9), ceiling: CheckpointTxg(9)`、`14 stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(8), ceiling: CheckpointTxg(8)`。卡住之后要写掉那一槽才解开，而写不进发布就到不了那一槽（384 槽盘上停在第 8–15 步），这次挂载里一直 ENOSPC。exact 臂在 K1-1 那一档（第 6 步起顺序写 8 单元）上也 0 红、0 拒（`out/scan1-exact-u8-s6.log` 54 段）。

**与用户已认的代价比**：D16「抬 F 的上限」那一行认的是「读坏槽并进上限、卡住就在这次挂载里报 ENOSPC」，读坏槽指挂载那一刻读坏、内容不知道的槽。S1 被抛弃根那一半内容是知道的，第 10i 批照「挂载那一刻读坏」钉住是用户的字面定案，所以这一格按字面不算越权；但它的代价不止「F 抬得少一点」：在这三个起点上 S1 那一类 54 段里 42 段从第 10i 批之前的「抬 F 被拒」换成了「F 卡在被抛弃根的 txg、ENOSPC」，可写挂载照样写不下去。第 10i 批要修的就是这一形（第三轮判决 S1 那一行），对被抛弃根那一半没修成。

**四句**：分辨臂（今天、refuse、ledger 停摆，exact 不停）；系统看得到（根环表那一项带着 `AbandonedRoot`）；满足正文 K1「或卡住上限多于必要」那一分句；跑前没给改法。exact 臂只在我的模型上量过、被攻过零轮。

### K1-3　代价量出来：住有效根的 S1 槽一直读坏时，准入上限被钉在那条根附近，384 槽盘上三到五成的段从「抬 F 被拒」换成「F 卡住、ENOSPC」

**没打中安全性**：S1-V（S1 槽住着有效根 V）在五个起点上 0 红（下表「红」一列，含 I-7.9、丢盘整读 txg ≥ F 的根）。

**代价**（覆盖写 40 次，计划 a / b / c × 读错 / 全零 / 乱字节 × 触发盘两种，副本上的数，`out/live-*-<起点>.log`、`out/base-prod.log`）：

| 起点 | S1-V 段数 | 今天：ENOSPC 停摆 | refuse（第 10i 批之前）：抬 F 被拒 | exact（S1 有效根按自己的 txg 并上限） | asread（S1 槽读坏时照挂载时读出的那条根算） | 无持续故障的对照 |
|---|---|---|---|---|---|---|
| `mat-384-30-8` | 288 | 100 | 144 | 100 | 0 | 写完 40 次 |
| `ab-384-3-5` | 216 | 120 | — | 120 | 0 | 写完 40 次 |
| 三个 `ab-384-*` 合计 | 468 | 250 | 408 | 250 | — | 写完 40 次 |
| `narrow-264-6-4` | 132 | 132 | 0 拒于写，132 段挂载本身报 `FloorRaiseFailedAfterTheMountsPublishes` | 132 | — | 第 14 步 ENOSPC（盘本来就满） |

（`narrow-264-6-4` 没故障也在第 14 步停，那一行分不出臂；`narrow-384-20-2` 288 段今天全写完。asread 只跑了表里两格。）

**读法**：exact 与今天逐格同数，停摆来自「F 不越过那一槽能住的 txg」这一条本身，不来自能住的最大 txg 取得比 V 大；asread（进程把挂载时读出的那条根记着，读坏时拿它当这一遍读出来的）一段都不停，也一段都不红（K1-1 那一档 54 段同样 0 红 0 拒，`out/scan1-asread-u8-s6.log`）。今天的实现在 S1-V 上比第 10i 批之前好（`mat-384-30-8` 144 → 100 段写不下去，`ab-384-*` 408 → 250），离「不拒」还差这么多。

**与用户已认的代价比**：用户认的是挂载那一刻读坏、内容不知道的槽「卡住就在这次挂载里报 ENOSPC」；S1 槽的内容进程知道，第 10i 批照字面把它也当作内容不知道。按字面不越权，记成「代价比『钉住不拒』读起来大」：384 槽盘上 S1-V 一直读坏的段里约 35%–56% 仍在 40 次写之内停摆、要等写掉那一槽才解开，而停摆时发布推不动、写不掉那一槽。

**四句**：分辨臂（今天与 exact 停、asread 不停）；系统看得到的只有 txg 与类别，看不到那条根的树表指针——asread 要根环表多记整条根记录（我的副本用一张线程局部表记，见补丁 `OPUS_I1_STASH`），这是 asread 要改的数据结构，不是现成就有的；满足正文 K1「卡住上限多于必要」；跑前没给改法。asread 只在我的模型上量过、被攻过零轮，层 0 没跑。

## K2　回退与 F_生效 那两处的读法

### K2-1　打中（前后不一致，不是安全性）：同一次挂载里抬 F 照「挂载那一刻读坏」钉住推过了某一槽，管理员回退随后因同一槽拒

**历史**：起点 `ab-384-3-5` 或 `mat-384-30-8`，S1-V 计划（同 K1-3），挂上之后覆盖写 s 次（s 扫 3、6、9、12、15），再管理员回退到环里比现行旧、txg ≥ F 的最新一条有效根（按无故障盘面选，`OPUS_ROLLBACK_AT_END=1`）。回退经 `crates/singlefs-core/src/mount.rs:6793`：「let target_root = rollback_candidate(」，读根环用 `crates/singlefs-core/src/mount.rs:6797`：「&ring_slots_known_to_hold_a_root_by(allocator),」——S1 槽在这一份里（`allocator.rs` 的 `ring_slots_known_to_hold_a_root` 第 10i 批加进 `RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`），读坏重读仍坏就拒。

**量的**（副本上的数，`model/k2agg.py` 汇总 `out/k2-prod-*.log`；「抬 F 读坏过」= 这次挂载里至少一次抬 F 那一遍读坏、重读仍坏而照钉住推过了那一槽）：

| 起点 | 覆盖写几次再回退 | 抬 F 读坏过、回退拒在同一个 S1 槽 | 抬 F 读坏过、回退做成 | 没抬过 F、回退拒在 S1 槽 |
|---|---|---|---|---|
| `ab-384-3-5` | 3 | 0 | 0 | 216 / 216 |
| `ab-384-3-5` | 6 | 132 | 44 | 0 |
| `mat-384-30-8` | 3 | 0 | 0 | 228 / 288 |
| `mat-384-30-8` | 12 | 79 | 26 | 0 |

拒的原样一例（`out/k2-prod-3.log`，计划 `s1-V@0/2(d0)-trig0-b-E`）：
```
rollback=Err(txg51:CandidateJudgementStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{ring_slot:RootRingSlot{region:0,slot:2},first_reading:Unreadable,reread:Unreadable}))
```
回退的目标（txg 51）不在那一槽：拒的理由是「根环里有没有目标、F_生效 多少」这一判读不全，而那一槽里是什么进程是知道的（txg 与有效 / 被抛弃都记着）。第 10i 批之前（refuse 臂）同一组 3 次写之后回退结局逐类同数（`out/k2-refuse-3.log`：ab-384-3-5 216 拒、mat-384-30-8 228 拒 60 成），所以「回退拒」本身不是第 10i 批带来的；第 10i 批带来的是同一槽在抬 F 那一处推得过、在回退那一处推不过。

**条文**：D16 第 37 行（`.claude/kb/decisions/16-发布语义.md:37`：「挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽（这三类是「这个进程知道住着根的槽」），这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退」）回退那一半把「挂载那一刻读得出」的槽列为「这个进程知道住着根的槽」、读坏重读仍坏就拒；S1 槽两遍里有一遍读得出，字面上套得上这一半，也套得上第 10i 批照抬 F 那一半改成的钉住——正文第二节第一条读法写的就是这个。条文对回退没写例外，实现照字面拒，不算拒的理由与条文对不上；对不上的是两处之间。

**「本可以安全回退却被拒」**：推的、没造盘面。回退判候选只要「目标在不在环里」与 F_生效；目标在别的槽、读得出，F_生效 的系统配置那一份不低于任何一条根带的 F（SysPre，D16 已定项 1「抬 F 那一串」），S1 槽里那条根的 F 改不了 F_生效 的上界——按这条推，回退到别的槽本可以做。这是推的，没在副本上实现一条「回退时 S1 槽读坏不拒」的臂去量它红不红。

**四句**：分不分辨臂——今天与第 10i 批之前都拒，不分辨「第 10i 批对不对」，分辨的是「回退那一处要不要跟着抬 F 那一处改」；系统看得到（根环表记着这一槽是 S1 类）；满足正文 K2「这一类在这两处与抬 F 那一处处置不同，会不会在同一次挂载里前后不一致」那一分句；跑前没给改法。

### K2-2　F_生效 那一处

抬 F 那一串自己的 F_生效 读法是 「pub fn effective_rollback_floor_rereading_the_newest_instance_table_once<」（`crates/singlefs-core/src/recovery.rs:1618`），根环读坏的槽当没有根、不按「知道住着根的槽」拒，S1 槽在这里读坏不拒；按「知道住着根的槽」拒的 F_生效 只在管理员回退那一份（`crates/singlefs-core/src/recovery.rs:1662`：「pub fn effective_rollback_floor_rereading_known_ring_slots_and_the_newest_instance_table_once<」），已含在 K2-1 里。上面这句是读码，没另造历史。

## 没打中的形状

| 形状 | 取样 | 结果 |
|---|---|---|
| S1-V 一直读坏时 F 越过那条有效根（能住的最大 txg 取得比 V 大，V 又是那块盘上最新的有效根） | 五个起点 S1-V 全部计划（ab-384 三个 468、mat-384-30-8 288、narrow-264-6-4 132、narrow-384-20-2 288 段），每段覆盖写 40 次，每步跑池级 checker（含 I-7.9） | 0 红。按代码推，能住的最大 txg 大于 V 的 txg 只在两种盘面上出现：第 0 代根在区域 1、2 槽 0 的副本，与「崩在记录之后、根槽之前」留下的洞；这两种都没造（见限度）。exact 臂（按 V 自己的 txg 并）与今天逐格同数 |
| 抬 F 那一遍把 S1-V 槽钉在「可能住过的最旧 txg」让回收门槛越过盘上有效根 | 同上 | 0 红、丢盘整读 txg ≥ F 的根 0 坏。门槛取 max(F, 环里最旧有效根)，钉低只会少回收 |
| S1 槽什么时候解开：这个进程写掉那一槽 | 读码 | `record_root_written_by_this_process` 把那一槽换成 `ValidRoot`（`crates/singlefs-core/src/allocator.rs:2010`：「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」），被抛弃根那一半照清隔离；进程内记根的 E2 重读不碰 S1 类（它用 `ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held`，不含 S1），S1 只在抬 F 那一遍读坏的那一次按钉住算，读得出就照读出来的算——所以「重读读出根才解开」对 S1 不适用，也没有漏解开的路径 |
| 写过的 S1 槽之后再读坏 | 原型默认进程写过的槽不再注故障；加这一条之前的探路跑过 9 段（`ab-384-3-5` 上 8 + 1） | 那一槽已是「这个进程写过且 FUA 返回过」，照知道住着根的槽拒（`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），是条文写明的处置，不算打中 |
| 正常卸载那一串丢掉 S1 被抛弃根 | ab-384 三个起点，覆盖写 30 次再卸载，522 段 | S1-A 的 a、b 两形 36 / 36 段卸载之后潜在漏洞 4 槽，checker 0 红（那一串自己的空发布没发到那几个槽、进程随即退出）；c 形 12 段在卸载前已有潜在漏洞 |

## 这条腿自己的限度

- 全部数在冻结副本的拷贝上；臂开关（`refuse`、`ledger`、`exact`、`asread`）是我在副本 `mount.rs` / `allocator.rs` 里加的，补丁 `mount-arms-i1.patch`。四个臂都只在我的模型上量过、被攻过零轮。
- 故障模型：读错、读回全零、乱字节，按槽、按「这个槽第几次读」注；计划 c 的「每三次读有一次读得出」是我挑的交替节奏，别的节奏没扫。进程写过的槽不再注故障。
- 起点只有 384 槽单元区（三个带被抛弃根的、一个成熟的、两个窄盘）；4 GiB 盘、带洞的环、第 0 代根副本住着 S1 的年轻环都没造。K1-1 在 4 GiB 盘上会不会因为不走「最低空闲槽」而更难发到那几个槽，没量。
- 层 0 没跑：这一轮的打中都在挂载之后的进程内，没造崩溃态；原型跑的层 0 状态数为 0（预算 10⁷ 一点没用）。
- 用户动作扫了：覆盖写、建 inode、两者交替、覆盖写后卸载、从第 6 / 9 / 12 / 15 步起顺序写 1 / 2 / 4 / 8 / 16 个单元；K2 回退前覆盖写 3 / 6 / 9 / 12 / 15 次。K1-1 的三臂对照只在「8 单元、四个起步点」上做（臂不影响用户动作：按步数切换）。顺序写 1 / 2 / 4 单元那几档是旧触发法（看到潜在漏洞才切），只用来找形，表里没用。
- 跑批：先跑 8 段估时（0.8 秒），之后每批一个子命令、几个起点；最后在同一个二进制上把报告引的 33 个阶段整套重跑一遍，与之前的日志按汇总键逐份比对，33 份全同，落进模型目录 `out/`（80 批合计 8652 段、`run_s` 合计 1966.8 秒）。
- 「本可以安全回退」（K2-1）是推的；「asread 不引入别的问题」只在上表那几格上看过。

## 没做什么

- 不判正推那几格（K3、K4）；不替主 agent 采纳。
- 没跑 checker 档、层 0 重型、变异表；没动仓里 `crates/`。
- 副本上的数不进 kb；要引的由主 agent 在入库装置上重做。
- 线程上限两次变更（15 → 31 → 15）照收到时改，之后起的命令按新值；已经跑完的不重跑。

