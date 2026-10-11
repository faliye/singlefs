# 根环实现第 10o–10s 批代码轮第一轮：攻方腿（opus）报告（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 T1 T2 S1 -->

立场：找反例，攻正文第三节 K1–K5「哪一格会错」。代码读冻结副本 `~/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/`（下文 `crates/…:行` 都是冻结副本里的路径与行号）。
副本上量的数都在我草稿目录里的仓拷贝上，是副本上的数，不进 kb。

## 复跑

模型目录 `research/prompts/m3-batch10o-code-r1-opus-model/`。原型从上一轮攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 改来（加带洞的环起点、回退之后再做几步与卸载、按准入那一格强制抬 F 到上限三样），`agg.py` 原样拷自那一份。

```
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh setup <空草稿目录>   # 拷冻结副本、port.py 加计数与 T2 臂、放原型与两份 harness 用例、release 编原型
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh k1 <草稿目录>
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh t2 <草稿目录>
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh hole <草稿目录>
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh k4 <草稿目录>
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh hb <草稿目录>
bash research/prompts/m3-batch10o-code-r1-opus-model/rerun.sh summary <草稿目录>   # 与 out/ 里的原件比 SUMMARY 行
```

每段经 `scripts/hosts/run-with-memory-cap.sh`（原型 3G、K4 用例 2G、H-b 用例 4G）与 `capped.sh 15`；原型 14 个工作线程。

| 文件 | sha256 |
|---|---|
| main.rs | d66f815631f44ff685b71967d39a7d1d20ab4ed0355abc78a347c163b757d56c |
| Cargo.toml | b56b09599e21383fe9d9b12efe6e37cebc80ef6a4370118072fa7f7e02adee47 |
| port.py | 9de430bc00203f2088c32e9cdd8e4ec43f3c1c8c517c848754b2f703793ebe28 |
| opus-o1-arms.patch | e848e112f480edab7cd8c36144733afc29080a3724ff81bd2ae661b30712e471 |
| run-stage.sh | 1b6bbe645e6e725898fcb8bb0d5ea66e67347a6e1de3e74e7e2b9abeaba71919 |
| summary.py | 7c435aafa3e019250733b453f1b85d1352ddb9a090e0864102a0345f120188ae |
| k2sum.sh | 4fbb316e459f1d454ad8d783e93e7e7602caa0eb8af696368114e6056d52bf47 |
| agg.py | 4585df11f61eb9e7f331a67cc3df2f810927aee9a276283df048cfefd59331dc |
| opus_o1_gpu_memory_ledger_attack.rs | 68ce93ba14dfcae9d66dd455b13b2cfb17448a83be3331204ed191d1c54b360f |
| hb-test-append.rs | 361618fe917238994a3f69ca7f46ffbbf9ade9eeb1a17fb0ef6edaa7c4fd935d |
| rerun.sh | 7adb42077e21dc9512bbd96576859edff05bd67139897aa1697ab0166c6f3ddb |

`out/` 下 41 份原样日志（本报告引的每个 SUMMARY 行都由 `summary.py` 从它们数出）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 回收门槛越过有效根 | 没打中 | K1 那一组 16 份日志 2,448 段（1,860 段抬 F 那一串真顶上了记录），连 T2 两臂共 6,804 段（3,936 段顶上过），checker 与丢盘整读 0 红；读码另见「故障模型之外」一条线索（错位读，推的） |
| K1-1 被抛弃根掉出影子账（10i K1-1 形） | 没打中（10i 打中的那一形已关） | 同一组历史 216 + 522 段，被抛弃那一半顶上 232 段，I-7.4 0、latent-abandoned-free 0；10i 判决那一份同组 54 段里 8 段红 |
| K1-1 准入上限卡在旧记录（10i K1-2 形） | 没打中 | narrow-264-6-4 132 段全在第 14 步 ENOSPC，与无故障底子（base-prod 同一步）相同；10i 那一份在第 10–14 步 |
| K1-2 写掉那一槽就换类 / 三处同一份 | 没打中（读码） | 记录只在重建那一处 `Box::new(*root)` 取一次；写那一槽经 `record_root_written_by_this_process` 换类；落盘途中失败冻结、抬 F 第一道拒。没造写故障 |
| K1-3 T2「回退不拒」臂 | 量了，交用户 | refuse 臂 11 份日志 2,034 段里回退被拒 1,122 段；push 臂 12 份 2,322 段 0 拒、0 红（含回退之后 6 次覆盖写与卸载）；「抬 F 顶上过之后再回退」那一组 push 臂 630 段 0 红 |
| K1-4 K1-d 留洞的环 | 没打中 | 一洞、两洞两个起点 288 段，180 段顶上过（洞那一槽本身也在内），0 红；年轻环 36 段一段都没走到顶上（新类槽在第一次抬 F 之前就被写掉） |
| K1-4 H-b 下的 I-5.1 | 核了，主 agent 的读法站得住（量过，副本） | I-5.1 原文点名盘 0 槽 50368 被「实例表单元」与「实例表第 1 片」两处引用，是镜像自己造的双引用；对照镜像 I-5.1 成立 |
| K2-1 四形各读法同口径 | **打中，口径差（推的，读码）** | 实现解链指针记录时另判位置指针头部第一版取值，checker 的 I-3.14 第四形与 kb I-3.14 行都不判：这一形实现读不出整张表，checker 照收行、照往下走、I-9.29 门开着 |
| K2-2 解不开之后三件事 | 打中一处边角（推的） | 第 1 片起解不开时 I-3.8 照前几片的行判，不报不适用；kb I-3.8 行字面只写第 0 片，与字面一致、与「行没收全」的理由不一致 |
| K2-3 H-b 键 | 没攻 | 见「没做什么」 |
| K2-4 I-3.8 理由 | 同 K2-2 | — |
| K3-1 七个函数次序 | 没打中（读码两处） | `judge_release_generation_and_tree_table_birth`、`judge_allocation_records_disjoint` 两处逐行对，次序、提前返回、`every` 的取值时点都同；另五处没读 |
| K3-2 对拍能不能说明 | 没跑（要上卡） | 列了对拍流没罩到、改法会分叉的状态类（推的） |
| K3-3 调用点单测拦不拦得住 | 没打中 | 调用点按 naga 语句数、展开数逐个钉死、余量用 `checked_sub` 判，超预算一定红；漏的是「不在七个里的函数多调一处」只靠展开钉值拦 |
| K4-1 容量大于真能用的 | **打中（量过，副本）** | 设了额度变量就不再经配置给这张卡的额度与 3/4 那一份：配置 8G、额度变量 20000 时账的容量 20000（不设变量时 5632） |
| K4-2 flock / 判活 / SIGKILL / 首次起池 | 没打中三格，量了一格 | flock 范围与改名写对；被杀而没收尸的僵尸判成活着（量过），收尸之后下一次读账就回收；同时首次起池时一份驱动开销算两次，少借 1980 MiB（量过，100 MiB 一笔借） |
| K4-3 账放得下而撞显存 / 放得下而等满 | **两向都打中（量过，副本）** | 账上「清场后空闲」只要进程链不断就一直传下去：外人后来占了 20000，后起的进程照 30000 给容量，超出真空闲 9940 MiB；反过来起池时外人占着，链上后来的进程 6000 MiB 判红、3000 MiB 等满报本次未跑，卡其实是空的 |
| K4-4 借的量与扫源码 | **打中（读码）** | 列在「只许经卡池」里的 `crash_amplification_pipeline_records_checks_and_reuses.rs` 那条只排计划的用例经产品路径另起每张卡的设备、不记账；扫源码另有几种写法绕得过（推的） |
| K5 #9 钉值 | **打中核法的射程（推的）** | 删两项 I-3.14 那一核只罩打印的 58 行；进度文件每个片行带逐条不变量的计数、首个违例原文与 findings，打印行里没有，新钉值 `1d92a7ad…` 没有对应的核 |

## K1　T1 甲撑不撑得住

### 装置

- 原型在冻结副本的拷贝上跑今天的实现（T1 甲），只加计数，不改判定（`port.py`）：抬 F 重算那一遍新类仍坏、拿记录顶上的次数与其中被抛弃那一半，上限那一遍顶上的次数，T2 臂顶上的次数。
- 历史照上一轮攻方的 s1grid：一个起点上，触发整环重读的那块盘第一遍每个有效根槽读坏，目标槽按三种读法造成新类（a 第一遍读得出、重读起一直坏；b 第一遍坏、重读读得出、之后一直坏；c 第一遍读得出、重读坏、之后每三次读一次读得出），故障三种（读错、读回全零、读回乱码）。挂上之后用户做 N 步改动，每步核池级 checker（I-3.1 记账多的一边是用户认的代价，不算红）、丢盘整读（挂载之前读得回的、txg ≥ 当时 F 的根，丢一块盘之后读回的变了就红）、根环表里被抛弃根的落点在分配器里可发（latent-abandoned-free）。
- 用户动作放开的几样：改动步数（3–40）、改动种类（覆盖写、建 inode、顺序写 8 个单元把窄盘推到抬 F）、改动之后卸载、强制每两步照准入那一格抬一次 F 到上限（`raise_rollback_floor_to_the_admission_ceiling`，宽盘上准入自己不抬）。

### K1-1 三形

命令（模型目录 `out/` 里）：`python3 ../summary.py scan1-prod-u8-s6.log … ab384-owu-ab-384-3-5.log`，原样：

```
SUMMARY scan1-prod-u8-s6.log segments=54 red=0 raise_or_ceiling_pushed=54 abandoned_pushed=48 nospace_refused=0
SUMMARY scan1-prod-u8-s9.log segments=54 red=0 raise_or_ceiling_pushed=54 abandoned_pushed=44 nospace_refused=0
SUMMARY scan1-prod-u8-s12.log segments=54 red=0 raise_or_ceiling_pushed=54 abandoned_pushed=44 nospace_refused=0
SUMMARY scan1-prod-u8-s15.log segments=54 red=0 raise_or_ceiling_pushed=54 abandoned_pushed=48 nospace_refused=0
SUMMARY live-prod-ab384.log segments=522 red=0 raise_or_ceiling_pushed=462 abandoned_pushed=48 nospace_refused=0
SUMMARY live-prod-mat-384-30-8.log segments=288 red=0 raise_or_ceiling_pushed=144 abandoned_pushed=0 nospace_refused=0
SUMMARY live-prod-narrow-264-6-4.log segments=132 red=0 raise_or_ceiling_pushed=132 abandoned_pushed=0 nospace_refused=132
SUMMARY live-prod-narrow-384-20-2.log segments=288 red=0 raise_or_ceiling_pushed=264 abandoned_pushed=0 nospace_refused=0
SUMMARY base-prod.log segments=12 red=0 raise_or_ceiling_pushed=0 abandoned_pushed=0 nospace_refused=2
SUMMARY ab384-owu-ab-384-1-0.log segments=138 red=0 raise_or_ceiling_pushed=114 abandoned_pushed=16 nospace_refused=0
SUMMARY ab384-owu-ab-384-3-0.log segments=150 red=0 raise_or_ceiling_pushed=138 abandoned_pushed=16 nospace_refused=0
SUMMARY ab384-owu-ab-384-3-5.log segments=234 red=0 raise_or_ceiling_pushed=210 abandoned_pushed=16 nospace_refused=0
```

（上面每行末尾 `t2_pushed=0 rollback_ok=0 rollback_err=0` 三项我在贴的时候删了，K1 这几组不回退。）

- 被抛弃根掉出影子账：`scan1-*` 是上一轮打中 I-7.4 的那一组（上一轮判决第一节：`scan1-prod-u8-s6` 54 段里 8 段 I-7.4 + latent-abandoned-free）。今天同组 4 × 54 段、`live-prod-ab384` 522 段，被抛弃那一半在抬 F 重算那一遍真顶上了的 232 段，0 红。机理在代码里对得上：顶上的记录进 `roots`（「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」（`crates/singlefs-core/src/mount.rs:3335`）），影子账重算吃的就是这一份（「let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;」（`crates/singlefs-core/src/mount.rs:3498`））。卸载那一串（`ab384-owu-*` 522 段，卸载 522 次全 Ok）同样 0 红，上一轮判决第五节那条「正常卸载那一串」的旁及随之关掉。
- 准入上限卡在旧记录：`narrow-264-6-4` 132 段全在第 14 步 `NoSpaceAfterRaisingTheFloor`，与无故障底子（`base-prod` 里同一起点 2 段，同样第 14 步）逐步相同；上一轮 prod 臂同组在第 10–14 步（上一轮 `out/live-prod-narrow-264-6-4.log` 经 `agg.py --by=sp,arm,red,refused`：@10 30 段、@11 20、@12 10、@13 40、@14 32）。上限那一遍顶上：「roots.push(*root_record_self_verified_on_one_rebuild_pass);」（`crates/singlefs-core/src/mount.rs:2500`），顶上的再照同一张表判有效（「.filter(|root| !abandoned_by_table(root, table))」（`crates/singlefs-core/src/mount.rs:2618`））。
- 回收门槛越过有效根：上面全部段 0 个 `broken>=F`。读码上，顶上的那条根只会让「环里最旧有效根」更小（门槛更低），越不过；门槛越过一条有效根要那条根本身不在 `roots` 里，而新类槽每一遍都在（读得出用读出的，读不出用记录）。

**四句**：不打中，不适用。**推翻条件**：同一组历史上任一段出现 I-7.4、latent-abandoned-free 或 `broken>=F`，或窄盘 ENOSPC 的步数早于无故障底子。

### K1-2 写掉那一槽就换类、三处是不是同一份

读码，没造写故障：

- 记录只在重建分配器那一处取一次：两遍按槽并好之后，新类的那一槽记「root_record_self_verified_on_one_rebuild_pass: Box::new(*root),」（`crates/singlefs-core/src/mount.rs:2276`；第 2273–2277 行那一臂，`root` 就是并好之后那一份读数里的根）。「多盘」与「整环重读」都在这一份并好的读数之前，没有第二份；「记根时重读」（E2）只管挂载时读坏与读回全零两类（`oldest_txg_in_the_ring_rereading_slots_bad_at_mount` 的入参 `bad_at_mount`），不碰新类。
- 写那一槽：这个进程每写一条根都经 `record_root_written_by_this_process`，在那一槽上直接换成有效根（「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`crates/singlefs-core/src/allocator.rs:2010`）），跳号就 panic（「"装了根环表的进程写的根 txg 逐个加一」（`crates/singlefs-core/src/allocator.rs:2002`））。带单元的发布在取完落点之后、根落盘之前就记了：落盘之前失败整份分配器换回发布之前那一份（槽回到新类，盘上那一槽也没写）；落盘途中失败冻结，抬 F 那一串第一道就拒（「if let Some(frozen) = allocator.frozen_publish() {」（`crates/singlefs-core/src/mount.rs:3404`）），回退同样（第 6693 行同一句）。冻结期间盘上那一槽可能已是新根、表里还是旧记录，而这期间没有一处会拿记录去顶。
- 没写就被清掉：只有换回整份分配器这一种「退回」，退回的是写之前那一份，记录跟着回来。

**推翻条件**：找到一条写根环槽的路径不经 `record_root_written_by_this_process`、又不冻结（例如有人在挂着的会话里直接调 `publish_version` 而不经分配器）。

### K1-3 T2「回退不拒」那一臂（量的数，交用户，不算打中）

臂：`port.py` 在回退判候选与算 F_生效 那一遍（两处都经「return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {」（`crates/singlefs-core/src/recovery.rs:1430`）那一出口）给一个线程局部的表，`OPUS_T2_ARM=push` 时装进这个进程根环表里新类槽记着的根：新类槽重读仍坏时顶上记录、不拒。refuse 臂就是今天的实现。

历史：起点 `ab-384-3-5`、`ab-384-1-0`、`ab-384-3-0`（带一条被抛弃根）、`mat-384-30-8`；新类造在有效根槽（V）与被抛弃根槽（A）上；回退之前的改动 3 / 9 / 15 步；回退目标取比现行旧、txg ≥ F 的有效根里最新的那条或次新的那条（`OPUS_ROLLBACK_PICK` 0 / 1）；一组在回退之后再覆盖写 6 次并正常卸载（`k2after-*`）。每步与回退之后照 K1 同一套核。

命令：模型目录 `out/` 里对 refuse 臂 11 份、push 臂 12 份分别 `python3 ../summary.py …` 再按列加总，原样：

```
T2-refuse logs=11 segments=2034 red=0 pushed=966 t2_pushed=0 rollback_ok=912 rollback_err=1122
T2-push logs=12 segments=2322 red=0 pushed=1110 t2_pushed=1278 rollback_ok=2322 rollback_err=0
```

只看「抬 F 那一串先顶上过记录、再回退」那一组（K1-3 点名的那组历史），另一段 python 逐行数，原样：

```
T2-GROUP refuse-arm logs=8 segments=1494 raised_with_push=966 rollback_refused_among_them=486 red_among_them=0
T2-GROUP push-arm logs=9 segments=1782 raised_with_push_then_rollback_pushed=630 red_among_them=0
```

- refuse 臂被拒的理由全是 `CandidateJudgementStillUnreadableAfterOneReread`（`k2sum.sh` 逐份列出，见模型目录日志）。A 类那几组 refuse 臂回退全拒（每份 18 段 18 段拒），push 臂全过。
- push 臂 0 红，含回退之后 6 次覆盖写与卸载（`k2after-push-*` 522 段，卸载 522 次 Ok）。
- 两臂只差回退做没做成；这组历史上没量出「不拒」那一臂的安全代价。没量的：回退目标恰是新类槽本身那条根、且那一槽之后一直读不出（目标版本要从盘上整个重建，读不出就在 `TargetVersionUnreadable` 拒，没造）。

### K1-4 K1-d 留洞的环

起点（原型 `hole_start`，搭法照 `crates/singlefs-harness/tests/common_root_ring_holes/mod.rs`：S = 4 的 12 槽小环、两块 128 MiB 盘）：`hole-1-20` 覆盖写到 txg 20 再崩一次挂载（写行那次的根槽没落），洞那一槽留着上一圈的根；`hole-2-20` 再崩一次，两个洞；`young-hole` 只 mkfs 就崩一次挂载，第 0 代根副本留在写行那次本该写的槽。宽盘上准入自己不抬，所以每两步强制抬一次 F 到上限（`OPUS_FORCE_CEILING_RAISE_EVERY=2`），20 步。原样：

```
SUMMARY holeF-hole-1-20.log segments=144 red=0 raise_or_ceiling_pushed=96 abandoned_pushed=0 t2_pushed=0 rollback_ok=0 rollback_err=0 nospace_refused=0
SUMMARY holeF-hole-2-20.log segments=144 red=0 raise_or_ceiling_pushed=84 abandoned_pushed=0 t2_pushed=0 rollback_ok=0 rollback_err=0 nospace_refused=0
SUMMARY holeF-young-hole.log segments=36 red=0 raise_or_ceiling_pushed=0 abandoned_pushed=0 t2_pushed=0 rollback_ok=0 rollback_err=0 nospace_refused=0
SUMMARY hole-1-20.log segments=144 red=0 raise_or_ceiling_pushed=0 abandoned_pushed=0 t2_pushed=0 rollback_ok=0 rollback_err=0 nospace_refused=0
```

洞那一槽本身做新类的那几段（`hole-1-20` 的 `@0/3`，`hole-2-20` 的 `@0/3`、`@1/3`；槽号按写行 txg 21、22 照落点公式算的，推的，没逐槽打印 txg）各 12 段里 12 段走到了顶上（11 段抬 F 重算那一遍、1 段上限那一遍），0 红。不强制抬 F 的 `hole-1-20` 一段都没走到顶上（`raise_or_ceiling_pushed=0`），这一组不算数，只留着说明强制抬那一下是必要的。

读法上 K1-d 那一形随 T1 甲关掉：S1 时上限并的是「能住的最大 txg」，可以高于洞里那条旧根自己的 txg；今天顶上的是那条根本身，照它自己的 txg 进上限与候选集。没造的：环里非空的不同状态不足 4 个（上限取最旧有效根）而洞里那条是最旧的那一形，要连着几次空发布把环填成同一个状态，这一组起点没有。

`young-hole` 36 段：9 段挂载被拒（`WritableMountRefusedByDevicesWithoutTheSelectedVersion`；我读成触发盘上的读故障让一块盘看不到择到的那一版、与新类无关，推的，没逐段核），27 段新类槽在第一次抬 F 之前就被这个进程的发布写掉，一段都走不到顶上。第 0 代根副本住在落点公式算不到的槽这一形，今天的挂载次序下够不着顶上那一处（推的：写行那次的 txg 1、2 正落在 (1,0)、(2,0)）。

### K1-4 H-b 下的 I-5.1（核主 agent 的读法）

在拷贝里给 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 末尾加一条只打印的用例（模型目录 `hb-test-append.rs`），对 H-b 那张镜像与它的对照各跑一次池级 checker。命令：`cargo test --offline -p singlefs-harness --test checker_known_bad_images opus_o1_print_the_i51_text_on_the_h_b_image -- --nocapture`（经内存包装 4G），原样：

```
OPUSO1-HB shape=CopyOnDeviceZeroBroken I-2.1 Violated("实例表第 1 片 在盘 0 槽 200000 的那一份与位置条目里的校验和对不上")
OPUSO1-HB shape=CopyOnDeviceZeroBroken I-5.1 Holds
OPUSO1-HB shape=CopyOnDeviceZeroBroken I-9.29 Holds
OPUSO1-HB shape=CopyOnDeviceZeroBroken violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
OPUSO1-HB shape=FirstLocationOfItsPointerNamesTheFirstPage I-2.1 Violated("实例表第 1 片 在盘 0 槽 50368 的那一份与位置条目里的校验和对不上")
OPUSO1-HB shape=FirstLocationOfItsPointerNamesTheFirstPage I-5.1 Violated("盘 0 槽 50368：被同一版引用了两次——先是 实例表单元，又是 实例表第 1 片")
OPUSO1-HB shape=FirstLocationOfItsPointerNamesTheFirstPage I-9.29 Holds
OPUSO1-HB shape=FirstLocationOfItsPointerNamesTheFirstPage violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"]
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 174 filtered out; finished in 15.25s
```

I-5.1 红的那一条点名的就是镜像自己造的那一处：指着第 1 片的指针，盘 0 那条位置条目写的是第 0 片的槽 50368，根记录里的实例表指针也指着它。I-5.1 的引用按位置条目的 (设备, 槽) 登记，与那一份读不读得回无关；对照镜像那条位置条目指自己的槽（200000），I-5.1 成立。主 agent「是那张镜像自身的毛病」站得住（副本上量过，入库装置没重做）。**推翻条件**：把那条位置条目改指一个没人引用的空槽，I-5.1 照红。

### 故障模型之外的一条线索（推的，没跑）

读根环槽只验自证校验和与 fsid（「let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {」（`crates/singlefs-core/src/recovery.rs:1254`）之后只有 `RootRecord::parse_slot`），不核读出那条根的 txg 照落点公式该不该住这一槽。重建那两遍里有一遍是错位读（读回别的槽、自证得过的一条根），另一遍读坏，新类就记下一条不住在这一槽的根，T1 甲之后抬 F 每一遍拿它顶；S1 时同一形照「能住的 txg」钉住，钉的是这一槽而不是那条根。错位读不在 D16 已定项 1「根槽这一次读坏」的故障分类里，所以不算打中；checker 有 I-7.17（根槽位置等于 txg 算出的位置）判写错位，判不到读错位。零轮改法：新类记录时核 `target_for_publish(root.checkpoint_txg) == ring_slot`（第 0 代根照 `genesis_root_ring_slots` 另认），对不上照挂载那一刻读坏记。只在读码上推的，被攻过零轮。

## K2　I-3.14 的口径

### K2-1 打中：链指针记录的「合法」实现比 checker 多判一样（口径差，推的，读码）

- 实现：「NodePointer::read_with_head_fields_judgement_from(」（`crates/singlefs-core/src/instance_table.rs:224`）——有下一片时位置指针按走读那一种判头部，文档自己写着「有下一片而位置指针头部有一段不是第一版的取值（MAC / nonce、算法类型、压缩算法码与」（`crates/singlefs-core/src/instance_table.rs:192`）也是 `None`，整片解不开。
- checker 主走读：「0 if pointer.all_zero => ChainRecordView::LastPage,」（`crates/singlefs-checker/src/walk.rs:740`）、「1 if !pointer.all_zero => ChainRecordView::NextPage(pointer),」（`crates/singlefs-checker/src/walk.rs:741`），I-3.14 那一格的说明也只列四样（`crates/singlefs-checker/src/walk.rs:3218`）。kb 的定义同 checker：「且自身合法（有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零）」（`.claude/kb/invariants.md:156`）。
- 一形：最新根的实例表第 0 片末尾那条链指针记录「有下一片」、位置指针头部 MAC 不全零（或 extent 偏移非 0）。实现读不出整张表（`instance_table_chain_of_root` 交错；抬 F 那一串在 `InstanceTableMalformed` 拒，重建分配器那一处重读一次仍读不出就 `NewerStateStillUnreadableAfterOneReread`，推的，没跑）；checker 判这一片 I-3.14 成立、沿指针往下走、收全各片的行，I-3.8 照判、I-9.29 门开着。镜像照样被 I-2.4（指针头部第一版取值）判红，所以不是漏报一张坏镜像，是「这一片解不开之后行不收、链不往下走、门关」三件事在这一形上与实现不同。
- 四句：不分辨臂（这一轮没有臂）；系统看得到（字节就在那条记录里）；满足的是 I-3.14「片解不开」那一句与实现 `InstanceTablePage::parse` 的口径差，归 K2-1 第 1 条；改法：I-3.14 第四形加「有下一片时位置指针头部取第一版的值」，或明写这一形归 I-2.4、不算解不开，二选一要主 agent 定。没造镜像，没跑。
- **推翻条件**：checker 的走读在跟随实例表链指针之前另有一处把头部不合第一版的当解不开处理（我只读了 `walk_instance_table_chain` 与 `chain_record_view`，没读 GPU 走读内核这一格）。

### K2-2 / K2-4 边角：第 1 片起解不开时 I-3.8 照前几片判（推的）

「if !pages.is_empty() {」（`crates/singlefs-checker/src/walk.rs:3255`）之后照收到的片判 I-3.8，只有一片都没收到时才「} else if walk_stopped_at_a_page_that_does_not_decode {」（`crates/singlefs-checker/src/walk.rs:3257`）报不适用。第 1 片起某片解不开：实现读不出整张表，checker 拿第 0 片那几行判唯一与低于挂载根。与 kb 字面（「最新根的实例表第 0 片解不开时报不适用」（`.claude/kb/invariants.md:150`））一致，与不适用理由那句「行没收全：没有行可判」的道理不一致：行同样没收全。能红能绿：被漏掉那几片里的重复实例代号看不到。只算口径边角，不算打中；交主 agent 定要不要把「任一片解不开」都报不适用。

### K2-3 H-b 键

没攻（见「没做什么」）。

## K3　扫描内核单调用点

- K3-1：读了 10q 的头两处（`_m3-batch10o-code-r1-diff-2.md` 第 11–135 行：`r_references_of_root` 那一处与 `judge_allocation_records_disjoint`）。前一处：第 0 圈走最新根、收已释放记录（含 `C_MAX_RELEASED` 越界提前返回），`every` 在第 0 圈末尾才取，之后每圈 `r_candidate`、`r_noting`、`walks_every_node` 与并之前逐项同；后一处：候选根、由记录施加出来的那几版、下一次挂载先施加的那一版次序照旧，最后一版仍在前两段判完之后才找。没看出次序或副作用差。另五处没读。
- K3-2（推的，没上卡）：对拍的 128 条流要罩到这几类状态，才说得上「改法没改判定」：已释放记录数撞到 `C_MAX_RELEASED`（第 0 圈提前返回）；由记录施加出来的版本数 > 0 且下一次挂载先施加的那一版存在（三段合一的循环三段都走）；候选根里有一条就是最新根（`walks_every_node` 的第二个析取项）。对拍日志里有没有这几类，我没看（要读对拍产物，没读）。
- K3-3：调用点按 naga 语句数（「Statement::Call { function, .. } => callees.push(*function),」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1253`）），文本换写法躲不开；展开数逐个内核钉死、余量用「super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL.checked_sub(size.expressions),」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1366`）判，超预算是 `None`、对不上任何钉值，一定红。漏的一形：七个之外的函数多调一处、或把被调函数的身子手抄进第二个调用方，调用点那条不红，只靠展开钉值红；钉值是「变了就红」，改的人量过新数再改钉值，余量一点点耗掉没人拦（有人拦的只是越过预算那一下）。不算打中。

## K4　显存账

装置：拷贝里的 harness 档用例 `crates/singlefs-harness/tests/opus_o1_gpu_memory_ledger_attack.rs`（模型目录同名文件），只用 `singlefs_harness::gpu_memory_ledger` 的公开接口、假进程号、临时目录里的真账本文件（真 `flock`、真改名），不碰显卡。容量闭包照 `gpu_unit_checks.rs` 第 1164–1174 行那一段抄（没给额度取空闲的 3/4，给了取 min(额度, 空闲)，再经 `card_ledger_capacity_mebibytes` 减 2560）。命令：`cargo test --offline -p singlefs-harness --test opus_o1_gpu_memory_ledger_attack -- --nocapture --test-threads=1`（经内存包装 2G），原样：

```
test k4_a_killed_but_unreaped_process_is_judged_alive ... OPUSO1-K4 zombie state_after_kill=Some("Z") alive_while_zombie=true alive_after_reap=false
test k4_environment_quota_replaces_the_configured_card_quota ... OPUSO1-K4 env-quota free=30000 config8192_no_env=5632 config8192_env20000=20000 no_config_no_env=19940 no_config_env40000=27440
test k4_simultaneous_first_registration_counts_one_driver_share_twice ... OPUSO1-K4 simultaneous f1=27440 f2=27440 capacity=18020 booked_in_100s=15400 with_true_free=17380 short_by=1980
test k4_stale_free_memory_is_inherited_by_overlapping_processes ... OPUSO1-K4 stale-high p1_free_recorded=30000 p2_free_read=10000 p2_free_used=30000 p2_capacity=19940 p2_booked=17380 p2_driver=2560 p2_needs_on_card=19940 real_free_when_p2_started=10000 over_by=9940
OPUSO1-K4 chain p3_free_read=31000 p3_free_used=30000
test k4_stale_low_free_memory_makes_tests_red_or_not_run_while_the_card_is_empty ... OPUSO1-K4 stale-low p1_free_recorded=10000 p2_free_read=30000 p2_free_used=10000 p2_capacity=4940 need6000=Some(NeedLargerThanEveryCard(NeedLargerThanEveryCard { need_mebibytes: 6000, largest_capacity_mebibytes: 4940 })) need3000_while_p1_holds2000=Some(WaitedPastTheLongestWait { need_mebibytes: 3000, waited: 300.148377ms })
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.85s
```

（最长等待在用例里设成 300 毫秒；产品里是「pub const LONGEST_WAIT_FOR_GPU_MEMORY: Duration = Duration::from_secs(30 * 60);」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:104`）。）

### K4-3 打中两向：「清场后空闲」只要进程链不断就一直传下去（量过，副本）

- 机理：起池时账本上还有活着的起池进程，就不记自己读到的空闲、照账本里那一份算容量（「if records.registered_processes.is_empty()」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:430`））。那一份是头一个进程起池那一刻读的；之后只要始终有一个进程活着，几小时、几十个测试二进制接力下去都不换。账只认账本里的进程，卡上不经账的显存用量变了，账看不见。
- 撞显存一向：P1 起池时卡上空 30000；之后一个不经账的 GPU 用户占了 20000；P2 起池自己读到 10000，账照 30000 给它容量 19940，P2 借到 17380、加自己的驱动 2560，要 19940，真空闲 10000，超 9940 MiB（`over_by=9940`）。P1 退出、P2 还活着时 P3 读到 31000，照样用 30000（`chain`那一行）。
- 等满报本次未跑一向：P1 起池时卡被外人占着、读到 10000；外人走了，P2 读到 30000，账照 10000 给容量 4940：要 6000 的用例判红（`NeedLargerThanEveryCard`，`lease_card` 照它 panic）；P1 借着 2000 时 P2 要 3000 等满最长等待报本次未跑，卡上其实空着 27000 以上。
- 谁是「不经账的 GPU 用户」，仓里现成就有：崩溃放量取卡不经卡池（「let cards = crate::gpu_unit_checks::gpu_cards_by_priority(&mine);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4233`）、「None => crate::gpu_unit_checks::usable_gpu_cards(),」（`crates/singlefs-checker-tier/src/crash_amplification.rs:4245`）），对拍文件里量速度那条取整张卡（「/// 量速度的那条用例取一整张卡：空闲显存最大的那张，不经卡池记账」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:207`）），还有别的工作根下（`SINGLEFS_WORK_ROOT` 不同、账本目录就不同）或第 10s 批之前的旧二进制、本机别的服务。
- 四句：分辨臂——「后起的进程照账本记的清场后空闲算」（正文第二节，主 agent 认的那一处）对「每个进程照自己读到的」：前者在外人后来占卡那一形上撞显存，后者不撞（后者在两个记账进程同时起池时会多算，那一格正是这次改动要修的）；系统看得到判别它的东西（P2 起池时手里就有自己读到的 10000）；满足的是 K4-3 第一句「账放得下而实际撞显存」与第二句「放得下而等满报本次未跑」；正文没给改法。零轮改法（只在我的模型上推过、被攻过零轮）：后起的进程容量取 min(账本里那一份, 自己读到的 + 账上别的进程已借的 + 别的进程各 2560)，这一格量过撞显存那一向是「推的」——我只量了今天的账给出多少，改法没实现。

### K4-1 打中：设了额度变量，配置给这张卡的额度就不作数（量过，副本）

「Some(quota) => quota.min(」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:46`）这一支不用 `budget_mebibytes`：配置 `GPU_CARDS_BY_PRIORITY` 给这张卡 8G 时，不设额度变量容量 5632，设 20000 就是 20000；没配置时 3/4 那一份留给卡上别的进程（19940），设 40000 就到 27440。配置那一项的文档写的是「崩溃放量在这张卡上占的显存不超过它」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:83`，`memory_quota_mebibytes` 那一字段的文档），配置模板写「卡上别的服务照常跑」。正文第一节的设计 ④ 字面就是「取它与清场后空闲减驱动开销两者的较小值」，所以这是设计本身（零轮）漏了配置额度与 3/4 那一份，不是实现偏离设计。零轮改法：`Some(quota) => quota.min(budget_mebibytes.saturating_sub(driver)).min(free.saturating_sub(driver))`，推的。

### K4-2

- `flock`：每次改账都对同名锁文件 `lock()`（「lock.lock()」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:373`）），读、去死进程、改、写临时文件、改名都在锁里；同一进程里两条线程各开一次锁文件，`flock` 按打开的文件描述互斥，也互斥。读账与「此刻放得下就记」分两次加锁，第二次在锁里重判（`book_on_the_card_if_it_still_fits`），不会超记。没打中。
- 进程号复用：比启动时刻（「.is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:183`）），同号同刻要在同一个时钟滴答里复用，没打中。
- SIGKILL：`/proc/<pid>` 消失之后下一次任何进程读账就删它那几行，等着的进程每 1 秒重读。僵尸（被杀而父进程还没收尸）`/proc/<pid>/stat` 还在、判成活着（量过：`alive_while_zombie=true alive_after_reap=false`），驱动这时已经还了显存，账要等收尸。父进程卡住不收尸时这一份一直挂着，可以把别人推到等满报本次未跑；正常父进程（cargo、包装 shell）当场收，算边角。
- 两个进程同时首次起池：先拿到锁的记自己读到的。读空闲在起设备之前（「let reported_by_bus = reported_cards_by_pci_bus();」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:164`）在 `request_device` 之前），但 P2 的设备若已经起了、P1 才读，P1 记下的那份已经扣了 P2 的驱动开销，两边再各给对方记 2560，P2 那一份算两次：量过少借 1980 MiB（按 100 MiB 一笔借；3/4 份额下理论差 1920）。压并发，不撞显存。
- 驱动开销比 2560 大：账的每一处都按常数 2560（`MEBIBYTES_LEFT_FOR_THE_DRIVER`），真开销大于它时 n 个进程合起来差 n × (真开销 − 2560)；正文第二节说 Q6 没上卡量，这里不重复。
- 同一张卡几个设备：`usable_gpu_cards` 对本机每张 NVIDIA 独显都起设备，起池时没点名的卡上也起了一个、随后丢掉（短暂，不在账里）；点名的卡上每个起池进程一个设备、付一份 2560。K4-4 那一条另在同一进程里多起一份。

### K4-4 打中：列进「只许经卡池」的文件里，有一条用例经产品路径另起不记账的设备（读码）

`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs` 在扫源码那份的 `FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL` 里。它的 `a_placement_only_run_reports_the_states_judged_at_recording_as_reused` 先经卡池判有没有卡（取了池：这个进程在每张卡上起一个设备、在账本上登记），再开 GPU 判器那一类（「environment.set("GPU_COMPUTATION_CLASSES", OsStr::new("crash_judge_gpu"));」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:980`））跑一趟只排计划的崩溃放量，那一趟照路由要卡（「Some(gpu_cards_in_use(cards_by_priority.as_deref(), host)?)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3926`）），经 `usable_gpu_cards` 在本机每张 NVIDIA 独显上再起一个设备。用例文档说「/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存。要 `verdict-store,gpu` 两个特性。」（`crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:958`）——草稿区不开是对的，但设备上下文与编好的 CRC 管线是实开的，账上一份都没有，「每卡一个设备」在这个进程里不成立。扫源码看不见：它只扫测试文件里的字面调用，产品路径不扫。

扫源码另有几种写法绕得过（推的，没编）：

- `CALLS_THAT_BYPASS_THE_POOL` 只列「"GpuCrashJudge::new(",」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:44`），`GpuCrashJudge::with_states_per_dispatch(`、`on_card_with_packed_tables(`、`on_card_compiling_these_kernel_sources(` 都不在；借来的卡 `lease.card().clone()` 之后把公开字段 `memory_quota_mebibytes` 改成 `None`，判器就按空闲显存的 3/4 开草稿区，账上只记借的那一点。
- 判「代码部分」按行里第一个 `//` 切（「line.split_once("//").map_or(line, |(code, _comment)| code),」（`crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:94`）），同一行前面有个带 `//` 的字符串字面量，后面的调用就当注释不扫；`use … usable_gpu_cards as cards;` 之后调 `cards()` 也不含被扫的那串。

借的量对不对得上（K4-4 第一句）：判器借 `mebibytes_a_judge_needs`，借来的卡额度是它加 2560，判器开的草稿区照「额度 − 表 − 2560」算，状态数又被这条流的状态数封顶，读码对得上；单元校验和那几条按 `mebibytes_crc32c_of_ranges_needs` 借，`create_buffer_init` 上传时驱动另开的暂存区不在里面（推的，没上卡量）。

## K5　#9 钉值重取

- 删两项之后哈希等于旧钉值，对打印的 58 行是决定性的：除了那两处 ` I-3.14=46/0/0`，别的字节一个不差。它说明不了 `46/0/0` 这三个数本身对不对（填别的数照样过这一核），那是钉值本来就钉的。
- 进度文件那个值没有对应的核，打中射程：片行带逐条不变量的计数与首个违例原文（「"checker_evaluated_states",」（`crates/singlefs-checker-tier/src/layer0_progress.rs:521`）、「"checker_first_violation",」（`crates/singlefs-checker-tier/src/layer0_progress.rs:523`），连同 `checker_violated_states`、`checker_not_applicable_states`、`findings`），每行末尾带自己正文的 CRC（「format!("{body} checksum={:08x}", crc32_castagnoli(body.as_bytes()))」（`crates/singlefs-checker-tier/src/layer0_progress.rs:481`））。打印的 `LAYER0_PROGRESS` 行只有片号、状态区间、进度计数（`crates/singlefs-checker-tier/src/crash.rs:3337` 那一行格式），没有逐条不变量那几列，所以打印行那一核罩不到它们。删除线第 25–30 批若改了某条不变量逐片的分布（总数不变）、不适用的片数、或违例原文，新钉值 `1d92a7ad…` 会把它一起钉进去，没人看见。
- 补的核法（推的，没跑：要跑 checker 档那条用例，归主 agent）：取这次跑出的进度文件，逐个片行删掉三张逐条计数表里 `I-3.14:` 那一项（`checker_first_violation` 里有的话也删），按 `line_with_checksum` 重算每行 CRC，整份 SHA-256 应等于旧钉值 `11977571…`；不等就逐行 diff 看多了什么。
- **推翻条件**：照上面那样删完重算等于 `11977571…`。

## 没打中的形状

| 形状 | 取样 | 结局 |
|---|---|---|
| 10i K1-1 那一组（被抛弃根新类、抬 F 那一遍仍坏、之后顺序写把窄盘推到抬 F），改动 6/9/12/15 步起顺序写 | 3 个起点 × 18 计划 × 4 | 216 段 0 红 |
| 同上，只覆盖写 40 步 | 3 个起点，522 段 | 0 红 |
| 有效根新类，成熟环、两种窄盘 | 288 + 132 + 288 段 | 0 红；窄盘 ENOSPC 步数与无故障底子同 |
| 覆盖写 30 步再正常卸载 | 3 个起点，522 段 | 0 红，卸载全 Ok |
| 带一洞、两洞的环，强制每两步抬 F 到上限 | 288 段，180 段顶上过 | 0 红 |
| mkfs 之后崩一次挂载的年轻环 | 36 段 | 走不到顶上 |
| T2 两臂：步数 3/9/15、回退目标最新/次新、V/A 两类、回退之后 6 次覆盖写与卸载 | refuse 2,034 段、push 2,322 段 | 两臂 0 红 |
| K4 跨进程账的 flock、进程号复用、读与记分两次锁 | 读码 + 五条用例里用到的那几步 | 没打中 |
| K3-3 换写法绕过调用点单测 | 读码 | 绕得过调用点那一条，绕不过展开钉值 |

失败条款上我自己装置的一处错（照 `evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」办）：第一版「回退之后卸载」的核在卸载之后仍按卸载之前的 F 判丢盘整读，`mat-384-30-8` push 臂 288 段全报 `broken>=F`（txg 47 那条根，卸载已把 F 抬过它）。是判据写错（卸载之后不该再保护低于新 F 的根），不是被判对象的问题；改成卸载之后取现行那一版根上的 F，同一组 288 段 0 红，refuse 臂与另几份 push 臂用改过的二进制重跑过，模型目录里的日志都是改过之后的。

## 这条腿自己的限度

- 一次抽样。没打中的格照规则不记站住。
- 全部数在副本上；原型是上一轮攻方原型改的，计划网格、核法照旧，攻击角度新增的只有带洞的环、回退之后的用户动作、强制抬 F 到上限三样。
- 没跑任何崩溃状态枚举（原型里 `l0` 子命令在，没用；本腿与接手的腿合计用掉的层 0 状态数 0）。这一轮的格都在挂着的会话与回退、抬 F 那一串上，崩溃点那一面没攻。
- 挂载期间故障只造了读故障，没造写故障；K1-2 那几条写路径全是读码。
- K4 的「撞显存」是账给出的数与假设的真空闲相比，没上卡；外人占卡的情形是构造的，频率没量。
- K2-1、K2-2、K4-4 的绕过写法、K5 的核法都是读码推的，没造镜像、没跑。
- 时长（`/usr/bin/time` 打的挂钟，14 线程）：`scan1-*` 每份 52–58 秒，`live-prod-ab384` 312 秒，`live-prod-mat-384-30-8` 307 秒，`live-prod-narrow-264-6-4` 25 秒，`live-prod-narrow-384-20-2` 150 秒，`base-prod` 33 秒，`k2-*-s3` 两臂各 69 秒、`k2-*-s9` 148–156 秒，带洞的环每个起点 40–51 秒；先跑 `ab-384-3-0` 一个起点 20 秒估过全量，全部原型段合计约一个半小时，没超过 40 分钟一批的估时，没缩取样。

## 没做什么

- K2-3（H-b「走过」的键按读回、自证过的那一份）：没造「两片读回同一份而键不同 / 读回不同而键相同」的镜像。
- K3-1 七处里只读了两处；K3-2 没读对拍产物，没上卡。
- GPU 走读内核、扫描内核、恢复内核里 I-3.14 的四形没逐一对（K2-1 只对了 CPU 主走读与实现）。
- 没跑 checker 档任何测试（重型）；K5 的核要跑 `crash_enumeration_sharded_across_processes.rs` 那一条，归主 agent。
- 不判正推、辩方那几格；不替主 agent 采纳。

## 草稿与要删的目录

草稿目录 `~/.local/share/singlefs/work/m3-batch10o-code-r1-opus/`（仓拷贝 `tree/` 与它的 `target/`、上一轮原型拷贝 `i1/`、日志 `out/`）交回前删掉；日志与原型已拷进模型目录。
