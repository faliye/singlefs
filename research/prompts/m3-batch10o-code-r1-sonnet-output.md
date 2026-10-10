# m3-batch10o-code-r1 云端正推腿（Sonnet）报告：K1–K5（2026-10-06）

立场：正推。从条款、定案与冻结副本里今天的实现推「应该是什么」，再逐格与代码比；一样写一样，不一样把两边整行并排抄。
引代码写冻结副本（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/`）里的绝对路径加行号，引 kb 写相对仓根的路径加那份文件自己的行号；每一处行号都是写进来之前在被引文件里 `grep -nF` 现查的。
判定三种：一致 / 冲突 / 规则没说。每条结论后面写「什么现象会推翻它」。

## 0 前提与复跑环境

- 快照核对：`cd /home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2; sha256sum -c ~/code/singlefs/research/prompts/m3-batch10o-code-r1-snapshot/crates-sha256.txt 2>&1 | awk '{print $NF}' | sort | uniq -c`，输出原样：`316 OK`（清单 316 行）。
- 复跑只在草稿副本 `~/.local/share/singlefs/work/m3-batch10o-code-r1-sonnet/tree/`：冻结副本的 `crates/`、`Cargo.*`，加原型包 `crates/t2-arm-probe`，再加 T2 臂的核心补丁。补丁只在环境变量 `SONNET_T2_ARM=norefuse` 时改回退那一处，不设就是冻结副本今天的码。
- 可复现物在 `research/prompts/m3-batch10o-code-r1-sonnet-model/`：`t2-arm-core.patch`、`main.rs`、`probe-main-vs-10i-opus-main.diff`、`Cargo.toml`、`rerun.sh`、`rerun-stage.sh`、`agg.py`、`agg2.py`、`golden-progress-file-strip-check.py`、`golden-printed-lines-from-main-agent-golden9.txt`、`golden-progress-file-from-main-agent-run2.txt`（K5 的两份材料，主 agent 草稿里取的副本）、`out/`（各阶段日志里的 OPUSI1-RUN、OPUSI1-SP、OPUSI1-TIME 行）。
- 原型 `main.rs` 取自 10i 轮攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs`（子命令 `s1grid`：造「两遍里只有一遍自证过」的槽，挂上后按用户动作推进，每步核 checker 与丢盘整读）；我加的改动在 `probe-main-vs-10i-opus-main.diff`：回退那一步加无故障盘面上的 F_生效 真值、加 `OPUS_ROLLBACK_BELOW`（挑 F_生效 之下的有效根当目标）、加 `SONNET_AFTER_ROLLBACK_STEPS`（回退后接着覆盖写）、加 `SONNET_PICK_LAST`（挑 F_生效 之上最低的那条）。
- 每条跑编译产物的命令都经 `nice -n 19 bash research/scripts/run-with-memory-cap.sh <3G 或 6G> bash research/scripts/capped.sh 15 …`。开跑前 `ps` 看到别的会话在跑 `cargo test`，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，照常排队跑。所有阶段退 0，没有 245、250–254。
- harness 档小测（草稿副本上、T2 臂不开）：`cargo test --release --offline -p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root`，末段原样：`test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.64s`。
- 没跑：checker 档（`singlefs-checker-tier` 的测试：K3 的库单测、K4 的卡池与扫源码用例、K5 的 golden）与一切要显卡的东西，归主 agent；要它们才能坐实的格写「复核不了」。

## K1　T1 甲撑不撑得住（A）

### 应然（定案与条款原文）

- D16 已定项 1 第 37 行对新类的处置：「这个进程写掉那一槽就换类，不留跨抬 F 的钉：抬 F 那一遍读坏时顶上的那条根只算在那一次抬 F 里」（`.claude/kb/decisions/16-发布语义.md:37`）；同一行：「有效根照它本身进上限与候选集，被抛弃根照它记着的落点进影子账、不进上限」；回退那一半同一行：「读坏重读仍坏照旧拒」（括注「不拒」那一臂没量）。
- 被抛弃根不许丢影子账：D23 已定项 14「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）。
- 发布失败：「写失败的槽按旧内容算」（`.claude/kb/decisions/16-发布语义.md:34`）。

### K1-1　抬 F 那一串拿记着的根顶上，有没有一形越线：一致（没找到）

代码路径（推到的行）：

| 步 | 行 |
|---|---|
| 重建分配器那一遍记整条根记录 | 「root_record_self_verified_on_one_rebuild_pass: Box::new(*root),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2276`） |
| 抬 F 读根环：读坏重读仍坏拿记着的根顶上 | 「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:3335`），入口「let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:3498`） |
| 算上限那一遍同样顶上 | 「roots.push(*root_record_self_verified_on_one_rebuild_pass);」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2500`） |
| 回退判候选那一遍不顶上，S1 槽算「知道住着根的槽」 | 「RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => true,」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1102`），回退读根环照旧「return Err(RootRingSlotKnownToHoldARootStillBadAfterOneReread {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/recovery.rs:1430`） |

三形逐形：
1. 回收门槛越过盘上真住着的有效根：回收门槛的「环里最旧有效根」把新类里有效的那一半按记着的 txg 算进来（「fn oldest_valid_root_in_the_ring(&self) -> Option<CheckpointTxg> {」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1136`，里面 `RootReadOnOneRebuildPass::ValidRoot` 一支），抬 F 那一遍顶上的根进 `roots`、再进 `oldest_valid_root_in_the_ring_for_raising_the_floor`。读码一致；实测见下表，没有一段红。
2. 被抛弃的根掉出影子账：被抛弃的那一半进「fn placements_referenced_by_abandoned_roots(&self) -> Vec<PlacementOnDevice> {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1164`），抬 F 那一遍顶上的根进 `roots` 重算影子账。10i 判决的那一形原样重造：`research/prompts/m3-batch10i-code-r1-main-verification.md:13` 记「arm=prod pattern=c red=true classes=I-7.4, latent-abandoned-free n=8」，`research/prompts/m3-batch10i-code-r1-main-verification.md:28` 记「副本上 54 段里 42 段 ENOSPC」；同一组 54 段（`scan1`：起点 `ab-384-1-0`、`ab-384-3-0`、`ab-384-3-5`，S1 槽住被抛弃根，从第 6、9、12、15 步起每步写 8 个单元把 F 推过去）在今天的码上：4 个阶段共 216 段，红 0，被拒 0。
3. 准入上限被一条已经不在盘上的旧记录卡住：记录只在重建那一遍记一次，那一槽的盘上内容只有这个进程写它才会变（K1-2），「记录不在盘上」只有一种来路：发布在根落盘之后失败、分配器换回发布之前的那一份，记录退回而那一槽已被写（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1983`、`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1985` 的注释写的就是这一支）；条款已定（D16:34「写失败的槽按旧内容算」）。读得出的槽照盘上读，只有那一槽又读坏才拿记录顶上，顶上的旧根只会压低上限与回收门槛、被抛弃的旧根只会多隔离，没有越线的方向。**推的**，没造这一形（要在 FUA 之后注入发布失败，harness 档没有现成的开关）。

实测（草稿副本，T1 甲就是冻结副本今天的码，臂不设）：

| 阶段 | 起点 | 段数 | 红 | 抬 F 被拒 |
|---|---|---|---|---|
| `scan1`（S1 被抛弃根；第 6、9、12、15 步起推 F） | ab-384-1-0、ab-384-3-0、ab-384-3-5 | 4 × 54 = 216 | 0 | 0 |
| `live`（40 步覆盖写） | ab-384-1-0、ab-384-3-0、ab-384-3-5 | 522 | 0 | 0 |
| `live` | mat-384-30-8 | 288 | 0 | 0 |
| `live` | narrow-384-20-2 | 288 | 0 | 0 |
| `live` | narrow-264-6-4 | 132 | 0 | 132 段在第 14 步 `NoSpaceAfterRaisingTheFloor`；无故障基线（`base-today`）同一起点同一步被拒（2 / 2），不是新类造成的 |
| `live`（20 步；4 GiB 盘，环里有洞） | ab-4g-1-0、ab-4g-3-5 | 138 + 234 = 372 | 0 | 0 |
| 合计 | | 1818 | 0 | 0（narrow-264 那 132 段与无故障同） |

每段：第一遍读坏的盘上每个有效根的槽、槽 s 住有效根或被抛弃根、s 的三种读法（a 第一遍读得出重读起一直坏，b 第一遍坏重读读得出之后一直坏，c 第一遍读得出重读坏之后每三次读得出一次）、三种故障（读错、读回全零、读回垃圾）。探针能不能红另做了对照：把抬 F 那一遍「顶上」拿掉（`SONNET_T2_ARM=positive-control-drop-substitute`，只在草稿副本加的开关），同一组 `scan1` 第 6 步起 54 段里 44 段红，类 `I-7.4` 与 `latent-abandoned-free`（`out/poscontrol-scan1-s6.log`，`classes={"I-7.4", "latent-abandoned-free"}`）。探针造得出 10i 那一形，今天的码 0 / 54。

推翻它的现象：在今天的码上造出 S1 槽里「第一遍读得出、重读起一直坏」之外的一形（例：发布在 FUA 之后失败、重发之前先抬 F）而 I-7.4 红、抬 F 被拒或回收门槛越过盘上一条有效根。

### K1-2　「写掉那一槽就换类」每条写路径都成立吗；「记整条根记录」三处是同一份吗：一致

- 写根环槽的代码只有一处是用户发布路径：执行器 `CommitStep::WriteRootRecordForceUnitAccess {`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/transaction.rs:446`，写的偏移 `slot_offset(`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/transaction.rs:462`）），另有 mkfs 写第 0 代根（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/make_filesystem.rs:337`、`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/make_filesystem.rs:626`，新池，没有这一类）。`slot_offset(` 在 `crates/singlefs-core/src` 里别处出现的都是读（`recovery.rs`、`instance_table_sweep.rs`、`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:1760`）。
- 记账那一头：根环表里换掉某一槽的入口只有「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2010`，在 `record_root_written_by_this_process`，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:1993`）；带重读的 `record_root_written_by_this_process_rereading_slots_bad_at_mount`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2063`）末尾调它（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2128`），补记零单元发布的 `record_zero_unit_roots_leading_to`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2136`）逐个 txg 调它。换下的占位是新类时，被抛弃那一半按它记着的落点清隔离（`displaced` 那一段，同一函数里）。
- 「没写就被清掉」：`occupants.remove` 全文件只一处（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2121`），对象是「挂载那一刻读坏、读回全零」那一张表（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/allocator.rs:2101` 取的 `ring_slots_pinned_since_the_mount_…`），新类不在里面。没有没写就清掉新类的路径。
- 「写了、记录还留着」：发布在根落盘之后失败、分配器整个换回发布之前（K1-1 第 3 形）。这一支条款已定「写失败的槽按旧内容算」，不是新类特有。
- 「同一份」：新类只在整环重读那一支里算（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2092` 的赋值在 `None => {` 重读分支里，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2090` 先调钩子）；记的整条根是 `roots_with_ring_slots` 里那一槽的根（两遍按槽并起来的那一份，多块盘各自区域的槽各有自己的一份，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2276`）；记根时重读只重读「挂载那一刻读坏、读回全零」的槽（同上 2101 行取的表），不重读新类的槽，新类里有效的那一半的 txg 仍进它算「最小的那个」用的集合。三处没有分歧。

推翻它的现象：在 `record_root_written_by_this_process` 之外找到一处写根环槽而不记；或 `occupants.remove` 作用到新类的槽；或新类与「挂载读坏」两类同时占同一个槽（表是 `BTreeMap<RootRingSlot, _>`，一个槽一项，不可能）。

### K1-3　T2「回退不拒」那一臂：量的结果（交用户，不算打中）

臂：在冻结副本拷贝的 `recovery.rs` 里让 `readable_roots_with_ring_slots_rereading_ring_slots_known_to_hold_a_root_once` 在 `SONNET_T2_ARM=norefuse` 时对 S1 槽「重读仍坏」拿根环表里记着的那条根顶上（回退判候选与算 F_生效 两处都走这个函数），其余不动（`t2-arm-core.patch`）。历史：同 K1-1 那一组（S1 槽三种读法 × 三种故障 × 第一遍读坏的盘），先按用户动作推 3 或 9 步覆盖写（F 照钉住推过），再回退一次，之后接着覆盖写并核 checker。回退目标四种：`p0` 每块盘上比现行旧的有效根里最新的那条、`p1` 次新、`b0` 低于无故障盘面上 F_生效 的有效根里最高的那条（应被拒）、`lowest`（`SONNET_PICK_LAST=1`）F_生效 之上最低的那条。每段的真值 `expect`（cand 或 below）取自无故障盘面上的 F_生效（`effective_rollback_floor_rereading_the_newest_instance_table_once`）。起点 mat-384-30-8（288 段）与 ab-384-3-5（234 段，其中 18 段 S1 槽住被抛弃根），共 522 段一组：

| 臂 | 推几步 | 目标 | 回退 Ok | 回退被拒 `CandidateJudgementStillUnreadableAfterOneReread` | 被拒 `TargetNotACandidate` | 判错（Ok 而 expect=below） | checker 红 |
|---|---|---|---|---|---|---|---|
| 今天（拒） | 3 | p0 | 60 | 462 | 0 | 0 | 0 |
| 不拒 | 3 | p0 | 522 | 0 | 0 | 0 | 0 |
| 今天 | 3 | p1 | 60 | 462 | 0 | 0 | 0 |
| 不拒 | 3 | p1 | 522 | 0 | 0 | 0 | 0 |
| 今天 | 9 | p0 | 240 | 282 | 0 | 0 | 0 |
| 不拒 | 9 | p0 | 522 | 0 | 0 | 0 | 0 |
| 今天 | 3 | b0（目标低于 F_生效） | 0 | 228 | 60 | 0（288 段 below 没有一段 Ok；另 234 段没有低于 F 的有效根） | 0 |
| 不拒 | 3 | b0 | 0 | 0 | 288 | 0 | 0 |
| 今天 | 9 | b0 | 0 | 282 | 240 | 0 | 0 |
| 不拒 | 9 | b0 | 0 | 0 | 522 | 0 | 0 |

回退之后接着覆盖写（`SONNET_AFTER_ROLLBACK_STEPS`，推 3 步、目标 p0）：不拒臂 522 次回退 Ok、之后 12 步每步 checker 都不红（`out/after-norefuse-s3-p0.log`，`after_rollback_ok=12`，红 0）；今天的臂 462 次被拒、60 次 Ok 后 12 步不红（`out/after-today-s3-p0.log`）。4 GiB 盘（环里有洞；mat-4g-30-8 288 段 + ab-4g-3-5 234 段，推 3 步、p0、回退后 6 步）：今天 Ok 72 / 被拒 450，不拒 Ok 522，两臂红 0（`out/t2-4g-today-s3-p0.log`、`out/t2-4g-norefuse-s3-p0.log`）。边界：`lowest`（推 3 步）在 mat-384-30-8 上不拒臂 288 / 288 Ok（今天 60 Ok、228 被拒）；ab-384-3-5 上最低一条是第 0 代根（无文件版），两臂都按 `VersionWithoutFile` 拒（探针的挑法造成，不是臂）。

对照（探针能不能分辨「判错」）：在草稿副本里把回退判候选里 `BelowEffectiveFloor` 那一拒拿掉（`SONNET_T2_ARM=positive-control-no-floor`），`b0` 推 3 步 mat-384-30-8 的 288 段里 60 段 Ok 而 expect=below；checker 在回退后 6 步里仍不红（`out/poscontrol-s3-b0.log`）。所以「判错」这一格靠 `expect` 分辨，checker 红是更弱的信号；两臂在 `expect` 上都是 0 次判错。

读法：在这组历史上，「不拒」那一臂回退成功的段数多（推 3 步 462 → 0 被拒），成功的回退目标都落在 F_生效 之上、之下的都被拒，checker 红 0；今天的拒在这一类槽读坏时拒 462 / 522（推 3 步）、282 / 522（推 9 步）。量到的就这些：没有一段让「不拒」那一臂红或判错。没量的：不拒臂下回退目标恰是 S1 槽自己那条根时回退写出的新版（`rebuild_version` 读的是目标那一版的树，不读它的根槽，读码如此，没单造）；多次回退；回退与抬 F 交错多轮。

推翻它的现象：在这组历史之外，不拒臂回退之后 checker 红，或回退目标低于 F_生效 被接受。

### K1-4　下一轮的攻击面两格

**K1-d 留洞的环**：一致（没找到越线）。环上有洞（年轻的环里还没轮到的槽）时，新类的槽与洞那一槽里留着的上一圈的根是两个槽、两条各自算的根：上限取「每块盘上最新的有效根、第 4 新的非空有效根、不足 4 条时取最旧有效根」按 txg 排，不按槽位置（`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-core/src/mount.rs:2593` 起）。T1 甲之后新类里有效的那条按它自己的 txg 参加排序，不再按「能住的最大 txg」并进来。10i 判决（`research/prompts/m3-batch10i-code-r1-main-verification.md`，K1-d 那一行，转述）说这一形随 T1 改成照 R 本身算上限、不再按窗口取最大；今天的码就是这样：读得出的非空根不足 4 条时取最旧有效根，洞里的旧根 R 本来就是读得出的有效根，按它自己的 txg 参加。实测：4 GiB 盘上的起点（`truth` 里 ab-4g-1-0 的 d0 有 9 个、d1 有 4 个，ab-4g-3-5 的 d0 有 3 个、d1 有 2 个读回全零（没轮到）的槽）ab-4g-1-0、ab-4g-3-5 共 372 段，20 步覆盖写，红 0、抬 F 被拒 0；同两个起点加 mat-4g-30-8 上推 3 步再回退（K1-3 的 4 GiB 行）红 0。

推翻它的现象：造出读得出的非空有效根不足 4 条、洞里留着的上一圈根与新类的槽里记着的根并存而上限被卡在已不在盘上的那条根的环。

**H-b 下的 I-5.1**：一致（主 agent 的读法站得住）。判据是条款原文：I-5.1 一行写「不存在被两个不同引用同时占用的物理范围」、展开写「同一个 (设备, 槽) 被两条指针引用判红」（`.claude/kb/invariants.md:196`）。H-b 那张镜像（第 1 片的指针盘 0 那条位置条目指回第 0 片）里，第 1 片的指针与根记录里的实例表指针占着同一个 (设备, 槽)，两条不同的指针引用同一落点，按条款本来就该红 I-5.1；harness 用例钉的正是：判红的违例集合 = 对照（第 1 片盘 0 那一份坏）的违例集合加 I-5.1（`a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back`，在 K2 引的 13 条里，通过）。所以多出来的 I-5.1 是镜像自己的毛病，不是 10k 的修法没修净。

推翻它的现象：一份第 1 片两条位置条目都不与别的指针共落点、盘 1 完好的 H-b 镜像，改后仍红 I-5.1。

## K2　I-3.14 的口径在各读法之间是不是同一个（B）

### 应然（条款原文）

- D18 已定项 11：「`kind` 不是 0 或 1 的记录：这一片解不开（表不可读）」（`.claude/kb/decisions/18-块里携带什么信息.md:308`），同一行接着写「链指针记录不在一片的最后一条、或最后一条不是合法的链指针记录，同样解不开」。行 flags 的一句在同一份的 303 行：「flags 1（第一版恒 0、非 0 拒收」。
- I-3.14 行：「走到的每一片都判；不成立的一片解不开：行不收、链不往下走、候选集滤不全（I-9.29 的门关）；只读不判的读法与引用集合那一遍同一个读法；GPU 走读内核同判，扫描内核的引用集合那一段与只读读法、恢复内核读实例表同判 flags」（`.claude/kb/invariants.md:156`）。
- I-3.8 行：「最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全」（`.claude/kb/invariants.md:150`）。

### K2-1　四形在六处读法里是不是同一个：一致

四形 = ① kind 不是 0 或 1　② 链指针记录在片中间　③ 行 flags 非 0　④ 最后一条不是合法的链指针记录。读码逐格对（每格都是「这一片解不开」）：

| 读法 | ① kind 未登记 | ② 链记录在中间 | ③ flags 非 0 | ④ 末条不合法 |
|---|---|---|---|---|
| 主走读（CPU） | `judge_instance_table_page_record_forms`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:791`）第一格，返回 `IllFormedSoThePageDoesNotDecode` | 同函数第二格 | 同函数第三格（`INSTANCE_TABLE_ROW_FLAGS_OFFSET`） | 链指针记录那一格（`chain_record_view`，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:734`）判 Malformed，走读在 3248 行停 |
| 只读读法（CPU） | `instance_table_rows_of_root_without_judging`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:6170`）里 `row[0] != 0` 交 None | 同一格（kind 1 不是 0） | 同一格加 flags 一句（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:6204`） | `chain_record_view` 交 Malformed ⇒ None |
| 引用集合那一遍（CPU） | `note_instance_table_pages_after_the_first`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:4686`）末条之前每条要是 `Row` | 同一格 | 同一格（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:4719`） | 末条走同一个 `chain_record_view`，数不全 |
| GPU 走读内核 | 三个布尔 `kinds_are_registered` 等，任一假就 `break`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1682`） | 同 | 同 | `w_judge(INVARIANT_I_3_14, chain != 2u)` 与随后的 `chain == 2u` 停（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1680`） |
| 扫描内核（引用集合） | 末条之前每条 kind 与偏移 21 都要是 0（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:512`） | 同一格 | 同一格 | 末条 `w_chain_record_kind == 2` 交 `r_complete = false` |
| 恢复内核（读实例表） | `/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:255`（kind 与 flags 一句同判） | 同 | 同 | 末条 `rd8(base, last) != 1u` 与有无下一片两支不合返回 false |

另一份扫描内核里的「读一条根自己的实例表」（`r_instance_rows`）与它同形（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1153`）。

核到的差别只有两处，都不改四形的口径：
- 链指针记录头部（MAC、nonce、算法类型）：实现 `InstanceTableChainRecord::parse` 把它也算「解不开」，checker 的 `chain_record_view` 不看它（由 I-2.4 一族另判）。I-3.14 条款只写「有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零」，checker 与条款一致，实现更严；这一处第 10o 批之前就如此，不归本轮。
- 第 k > 0 片解不开时：主走读保留前 k 片的行去判 I-3.8（只判得更少，不会误红），只读读法交 None（整张表没有行）。两边的「行不收」颗粒度不同，条款说「行不收」，没分颗粒度。

harness 档小测坐实四形各自一条与 H-b 的对照：`cargo test --release --offline -p singlefs-harness --test checker_known_bad_images -- instance_table`，末行原样：`test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 161 filtered out; finished in 4.98s`（含 `a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set`、`a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set`、`a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set`、`a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged` 等）。GPU 一侧的同判只读到码，没跑（checker 档）。

推翻它的现象：造一份四形之一的实例表镜像，六处读法里有一处照收它的行、或链照往下走。

### K2-2　「行不收、链不往下走、I-9.29 门关」三件事每处都做到了吗：一致（一处窄冲突在 K2-4）

- 行不收：主走读在 `pages.push` 之前停（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:3227` 起的分支）；GPU 走读在记行的循环之前 `break`；只读读法交 None；引用集合那一遍交「数不全」。
- 链不往下走：上面每一处都在换 `pointer` 之前退出。
- I-9.29 门：CPU 的 `abandoned_roots_filtering` 只在 `ChainRecordView::LastPage` 分支里置「滤全」，I-3.14 不成立走的分支碰不到它；GPU 的 `w_newest_instance_table_chain_reached_its_last_page` 只在 `chain == 0u` 的分支里置真，也碰不到。harness 里四形的 `…leaves_the_abandoned_root_in_the_candidate_set` 与 `…leaves_the_one_birth_invariant_partially_judged` 各一条钉门关（上面那 13 条里）。
- 没有一处照收、照走、门没关的读法。

推翻它的现象：第 k 片（k ≥ 1）四形之一、之后还有第 k+1 片带抛弃根的行，某一处读法仍把 k+1 片的行收进候选集的滤。

### K2-3　H-b「走过」的键按读回、自证过的那一份：一致

- 键：主走读 `/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:3174` 按 `placement_it_was_read_from`（`read_referenced_unit_with_the_location_it_was_read_from` 交回的第一份与校验和对得上的位置条目）认；GPU 走读用 `read_from_the_first_location`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1617`）重读第一条位置条目、对 `sum0`，对得上取盘 0、否则取盘 1（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1624`）。与 CPU 的「第一份对得上的」同一个选法（`w_read_referenced_unit` 同一个次序：先 `matches0`，才看 `matches1`）。
- 「读回同一份而键不同」：同一单元的两份副本分别被前后两条链读回，键各是自己读到的那一份；后一条链会把这一片再走一遍，只多走、不少判。
- 「读回不同而键相同」：键是一次对得上校验和的读的物理落点；两条链用同一落点必须同时对得上各自指针里的校验和，内容只能相同。身份四元组（片序号）判在 visited 之后，真成环（两条位置条目都指回走过的片）走「走到已经走过的单元」那条出口，不记 I-1.1，这一处第 10o 批之前就如此，不归本轮。
- 没有一形让链被当成走过而少判一片；harness 钉 `a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back`（上面 13 条里），修前红 I-3.9 的一支由变异行钉住（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/mutations.tsv` 里 H-b 那行，没跑）。

推翻它的现象：构造「盘 0 位置条目指回走过的片、盘 1 完好」之外的一形，使第 k 片读回而被当成走过。

### K2-4　I-3.8 报不适用的理由与 I-3.14 那一片对不对得上

两侧原文并排：

| 条款 | 代码 |
|---|---|
| 「最新根的实例表第 0 片解不开时报不适用，理由点名这一片解不开、行没收全」（`.claude/kb/invariants.md:150`） | 「最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:777`），只在 I-3.14 不成立的分支里置（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/walk.rs:3257` 的 `else if walk_stopped_at_a_page_that_does_not_decode`）；第 0 片读不出、身份不对、容器记录不合法这三种退出不置，I-3.8 落到缺省理由「这个镜像上没有它判的对象」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker/src/image.rs:368`） |

判定：I-3.14 四形落在第 0 片时，理由与 I-3.14 对得上（一致）；条款那一句写的是「第 0 片解不开」一般地说，代码只对 I-3.14 那四形点名理由，另三种第 0 片失败走缺省理由——条款没分，所以这一处记「规则没说」（窄冲突，行为上都是不适用，只差理由文字）。第 k > 0 片解不开：I-3.8 照前 k 片的行判、不报不适用，条款没写这种情形（规则没说）。GPU 一侧只出违例位，不出不适用理由，对拍比的是违例位（没有对照可比，不是回归）。

推翻它的现象：条款作者认为「第 0 片读不出」也要点名理由，则那三处退出要补；认为只指 I-3.14，则条款那一句加「（I-3.14 不成立）」即对上。

## K3　扫描内核单调用点改法有没有改了判定（C）

### 应然

题面是「判定次序与副作用不变」，只是把 7 个函数里的多处调用并成一处（实现员报告，转述）。预算在 `crash_judge_dispatch.rs`：「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`）。旧码取自 `research/prompts/_m3-batch10o-code-r1-diff-2.md` 第 10q 批那一段的 `-` 行（下面「并之前」列都是转述，不写成引文）。

### K3-1　7 个函数的次序、副作用、提前返回与并之前逐个对：一致

逐函数读的结论，新码行都在 `/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`：

| 函数（行） | 并之前（转述） | 并之后 | 判 |
|---|---|---|---|
| `judge_release_generation_and_tree_table_birth`（664） | 先 `r_noting = false`、清记录数、走最新根一遍（`walks_every_node` 真）；建已释放表（行满返回 `UNDECIDABLE_TOO_MANY_ENTRIES`）；再对每条候选 `r_noting = every`、`r_candidate = c`、走一遍 | 一个循环 `walk_index` 从 0 到候选数（678 行起）：第 0 圈走最新根、`r_noting` 假、不设 `r_candidate`；之后每圈走一条候选，`walks_every_node = every \|\| index == c_newest_index`、`r_noting = every`、`r_candidate = candidate`；唯一的调用点是「r_references_of_root(record_base, walks_every_node, candidate_slot);」（694）；建已释放表只在第 0 圈（`continue` 挡掉别的圈）、行满同样 `return`；`every` 在第 0 圈末尾赋值（717）后才被候选圈读 | 次序、`r_noting`、`r_candidate`（最新根那一遍两边都不设它）、`c_alloc_record_count` 清零、早退都同 |
| `judge_allocation_records_disjoint`（975） | 候选每条根（树表 36、分配记录 342）→ 记录施加的每一版（99、274）→ 找「下一次挂载先施加的那一版」再判 | 一个循环到「候选数 + 版数」，末圈才找下一次挂载那一版、没有就 `break`；唯一调用点「judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);」（997）；偏移默认值就是版那一支的两个常量 | 同；`JOURNAL_NEW_ROOT_SEGMENT_OFFSET` 等于 99（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:95`），与旧的字面 99 一致 |
| `judge_allocation_generations_against_unit_births`（1079） | 每条候选：先判它持有的分配记录树（342，`c_seen_node_insert` 去重），再判树表（36）；每一版判树表（99）；下一次挂载那一版判树表 | 同一个循环，候选支里先做持有的分配记录树那一段、再落到循环尾部唯一的「judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));」（1108）；版支、下一次挂载支只设 `base` | 同 |
| `judge_accounting`（1721） | `c_harvested = false`；`c_quarantine_exempted(…, w_reference_count[0], …)`；存 `exempted_all`；`c_deferred_below_the_floor`；`c_quarantine_exempted(…, c_newest_reference_count, …)` | 两圈循环（「for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {」，1731）：第 0 圈 `reference_limit = w_reference_count[0]`；第 1 圈先存 `exempted_all`、`c_deferred_below_the_floor`、再换 `reference_limit = c_newest_reference_count`；唯一调用点（1740）在圈尾 | 次序同：先第一次豁免、后存拷贝与推迟、后第二次豁免 |
| `judge_first_data_unit_of_the_versions_…`（2123） | 候选每条（36，`root.txg`）→ 记录施加的每一版（99，`record.txg`） | 一个循环到「候选数 + 版数」（2125），唯一调用点（2139），三个量按支设 | 同 |
| `judge_deletion_state_of_the_versions`（2567） | 同上两段 | 同形，调用点 2578 | 同 |
| `judge_tombstones_and_the_lineage_of_the_versions`（3152） | 同上两段，`d_object_birth_count`、`d_object_births_are_compared` 在循环之前设（没动） | 同形，调用点 3170 | 同 |

两处不属于控制流的细节也核了：`record_row_of_the_version_the_next_mount_applies_first()` 只读头里一个字（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1118`），没有副作用，所以「末圈才找」与「前两段判完才找」等价；`r_candidate` 只有 `r_references_of_root` 一支读（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:52`），第 0 圈不设与旧码最新根那一遍不设一致。

推翻它的现象：在某个状态上，7 个函数里有一个的违例位或 `undecidable` 在并前并后不同；或有一支原先「提前返回」的路径现在继续往下判。

### K3-2　上卡对拍 128 条只红删除那 3 条，能不能说明 10q 没改判定：不能单独说明，读码补上

对拍比的是同一条固定脚本流上每个状态的 GPU 与 CPU 违例位。V1（新编 10q 与 10r）128 条里只红删除那 3 条、V3（不打 10q）同形（背景材料所述，转述；我没跑，要显卡且是 checker 档）：两次在同一批状态上给出同一组红，说明这批状态上 10q 没改判定。它够不着的状态类，也是改法与 GPU / CPU 分歧最可能相关的几类：
1. 同时有「由记录施加出来的版」与「下一次挂载先施加的那一版」的状态（新池新建文件那次发布的状态）：旧码两个函数（`judge_allocation_records_disjoint`、`judge_allocation_generations_against_unit_births`）里这一支是循环之后单独写的一段，新码是末圈。对拍的流是删除线那条流（背景材料所述，转述），不含新池新建文件那次发布的状态（我没核，推的）。这一支由 `the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5024`）与 10q 替换进变异表的 34-1 行钉（我没跑）。
2. 候选集 ≥ 2 条、已释放记录只出现在非最新候选里的状态：`every` 在第 0 圈末尾才定，候选圈读它；这与旧码「先建已释放表再 `every = c_released_count > 0u`」同一时点，读码一致，我没核对拍那条流有没有这一类。
3. 已释放表、`C_MAX_RELEASED`、`c_seen_node_insert` 命中（两条候选持同一棵分配记录树）这几类边界：两边的早退与去重都在同一个位置，读码一致，对拍没撞到。
4. 候选数为 0 的状态：旧码跳过候选循环；新码的循环条件 `walk_index <= c_candidate_count` 仍跑第 0 圈（最新根那一遍），旧码里最新根那一遍本来就在循环外无条件跑，同。

读码 7 个函数（K3-1）已经把控制流逐点对过，所以 K3-2 的结论是：对拍 + 读码合起来，没找到改判定的一形；单靠对拍的 128 条不够，缺的是 1–3 那几类，由读码与那条 34-1 变异行补。

推翻它的现象：在 1–3 里任一类状态上造出一个 GPU 与 CPU 不同的违例位，且在 10q 之前同一状态上相同。

### K3-3　10s 的调用点单测钉今天的数，能不能拦住「换一种写法又把展开推回超预算」：拦得住大多数，两处限度

- `each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1306`）：naga 解析源码后数每个函数里的 `Statement::Call`（「Statement::Call { function, .. } => callees.push(*function),」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1253`），七对（调用方，被调方）各要：调用方里调被调方一处、整个内核里调被调方一处、整个内核里调调用方一处。`If`、`Switch`、`Loop`、嵌套块都会下钻，所以换成 `switch` 分几支各调一次、或在别的函数里再加一处调用，被调方的全内核调用点数都会变 2，红。
- `each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1344`）：三个内核各钉展开后的表达式数与余量，扫描内核钉 378_232（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1350`）、余量 21_768。任何改动内核源码使展开变化的写法，都先红在这里（精确相等，不是不超过）。10s 的变异行 `judge_accounting` 加回一处「展开 392 383 还在预算里，钉调用点数的单测红」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/mutations.tsv:3051`）说明了这两条的分工：总数还在预算里的回涨由前一条抓，预算之内的任何别的涨由后一条抓。
- 限度一：这两条是钉值，不是阻止；红了以后维护者可以改钉值，设计上就是「量过新数再改」（用例注释自己这么写）。真正的硬线是 `every_kernel_stays_within_the_inlined_size_budget`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1188`）不超 400_000。扫描内核余量 21_768（预算的约 5.4%），一个新增的多调用点函数可以一步越过。
- 限度二：调用点单测只盯那七对。一个第八个函数在多处被调、总数涨得不多时只由展开大小的钉值抓。

我没跑这两条（checker 档的库单测，只在提交时跑）；读码如上。推翻它的现象：造一个改写（例：把 `judge_accounting` 的调用包进新函数、该新函数在两处被调）展开大小变了而这两条都绿。

## K4　显存账会不会让一台机器 OOM，或者白白压并发（D、E）

### 应然

用户原话（背景材料所述，转述）：全局的任务只管会不会对指定的电脑造成 cpu 或 gpu 的 OOM，不终止别的任务，不压并发。代码里同一句的落点：「账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:6`）。所以两个方向各一条判据：账放得下而实际撞显存 ⇒ 违「只防 OOM」；实际放得下而账让它等满、报未跑或判红 ⇒ 违「不压并发」。

### K4-1　账上的容量会不会大于卡上真能用的

容量照 `card_ledger_capacity_mebibytes`：设了额度变量取「额度」与「清场后空闲减驱动开销」里小的，没设取 `budget` 减驱动开销（「Some(quota) => quota.min(」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:46`）。清场后空闲是头一个起池的进程在建设备之前读的：`reported_by_bus` 在 `request_device` 之前读（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:164` 与 `/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:181`），所以 2560 MiB 不会被基线先扣一次再扣一次（第一个进程）。

| 情形 | 判 | 依据 |
|---|---|---|
| 同一张卡几个设备：池内 | 一致 | 池里每张卡一个设备，借出去的是 `GpuCard` 克隆，共用设备与编好的管线（「compiled_pipelines_by_source_digest: std::sync::Arc::default(),」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:214`） |
| 同一张卡几个设备：池外 | 冲突（窄） | 见下面两行并排；三处不进账的设备：① 起池时 `gpu_cards_by_priority` 先 `usable_gpu_cards()`，对每一张够空闲的卡都建设备，配置没点名的卡建完即丢；② 崩溃放量产品路径自己起设备；③ `#[ignore]` 的量速度与规划显卡两条取整张卡不记账（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:215`、`…:2890`、`…:2971`） |
| 种假违例位那条另编一套管线 | 一致（缓冲）／规则没说（管线占多少） | 它在 `lease_card_waiting_for_room` 借的卡上建（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2602`），缓冲仍被借来那张卡的额度（需要量加驱动开销）封顶；另编的三个内核管线占的显存没量（要显卡），不在账里 |
| 驱动开销比 2560 大 | 规则没说（10r 报告 Q6 已知没做，不算打中） | 常数写着「5090 上量到的是 2326 MiB（草稿区从 241 MiB 到 12283 MiB 五档都是这个数），留到 2560」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:26`）；5080 与 5060 Ti 上没量；代码里没有运行时拿真实空闲回写账的一步 |

「同一张卡几个设备：池外」两侧原文并排：

| 账的口径 | 池外的设备 |
|---|---|
| 「驱动的上下文与编好的内核每个进程在每张卡上付一次，同一台机器上别的进程起的池各再记一份。」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:916`，`GpuCardPool` 的文档），账本只记一个进程起一次池 | 「let cards = crate::gpu_unit_checks::gpu_cards_by_priority(&mine);」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_amplification.rs:4233`）与「None => crate::gpu_unit_checks::usable_gpu_cards(),」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_amplification.rs:4245`）：开 GPU 的放量每次再起一套设备；L1 用例（`a_placement_only_run_reports_the_states_judged_at_recording_as_reused`）先起池、再跑开 GPU 的放量，同一个进程在同一张卡上两个设备、账只记一份 |

后果量级：一个设备的额外开销没量（要显卡），一个进程在同一张卡上两套上下文时可能多占约一份驱动开销（推的，5090 上一份是 2326 MiB，上面那条注释）。只有在这张卡已被账订满、又有这两条路径同时跑时才会越过卡。

推翻它的现象：在订满的卡上起 L1 用例或开 GPU 的放量，`nvidia-smi` 报的进程占用比账上记的多出一份驱动开销以上，且别的进程撞 `NV_ERR_NO_MEMORY`。

### K4-2　跨进程账

| 问 | 判 | 依据 |
|---|---|---|
| `flock` 的范围与读写次序 | 一致 | `with_the_card_ledger_locked`：打开稳定的 `.lock` 文件、`lock.lock()`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:373`）后才读账本、去掉死进程的记录、改、写临时文件再 `rename`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:393`），最后才放锁；一次只锁一张卡的账，不嵌套，没有死锁；借的时候先不持锁读一遍选卡，再持锁重判一遍才记（`book_on_the_card_if_it_still_fits`），不会因为读与记之间别的进程记了一笔而超订 |
| 判活：进程号复用会不会误判 | 一致 | 记的是（进程号，`/proc/<pid>/stat` 第 22 项启动时刻）两样，对不上就是不在了（「start_time == process.start_time_in_clock_ticks),」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:183`）；同一个进程号被另一个进程拿去，启动时刻必然不同。读文件出别的错当还在（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:184`，宁多等不放超）。限度：不同 pid 命名空间（容器）共用同一个工作根时会把别的命名空间的记录判成不在，本机没有这种用法 |
| SIGKILL 的进程它那份要等多久回收 | 一致 | 死进程的记录在下一次任何进程读写这张卡的账本时就去掉（「is_alive(lease.process));」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:276`）；等着借的进程每 1 秒读一次账（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:106`），所以约 1 秒加被杀进程被回收（`/proc` 项消失）所需的时间。进程成了僵尸而没被收时，`/proc/<pid>/stat` 还在、启动时刻没变，记录照算（推的；父进程一般立即收，被 `run-with-memory-cap` 的 scope 杀掉的也是） |
| 两个进程同时首次起池谁记清场后空闲 | 一致（偏保守） | 在这张卡的锁里，先拿到锁的记它自己读到的数，后到的采用账本里的数（「if records.registered_processes.is_empty()」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:430`）。两个进程的读数在各自建设备之前，但拿锁的先后不等于读数的先后：后读（因而读数里已经少了先建的那个进程的上下文）的先拿到锁，账本里的基线就少一份驱动开销，容量偏小约一份，只会多等不会超订（推的） |

推翻它的现象：账本里留下一条已经死了的进程的记录而 1 秒之后还没被回收；或进程号复用后旧记录被当成活的。

### K4-3　账放得下而实际撞显存／实际放得下而账让它等满

**（甲）实际放得下而账让它等或判红（「不压并发」一侧）：冲突，窄。** 两侧原文并排：

| 设计句 | 代码 |
|---|---|
| 「账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:6`） | 后起的进程起池时，`usable_gpu_cards` 把空闲显存低于 2048 MiB 的卡直接滤掉（「if free < MINIMUM_FREE_MEMORY_MEBIBYTES {」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:178`，常数「pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:13`）；配置点名的卡因此「起不来」（「named_but_not_started: configured_cards_not_usable(&mine, &started),」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1113`），分支「if named.named_but_not_started.is_empty() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1194`）的 else 一支交 `CardsPresentButNotUsable`，第一条来要卡的用例 panic（`gpu_card_pool_or_report_not_run`） |

这个读数是后起进程自己用 `nvidia-smi` 读的实时空闲，不是账本里的基线。先起的进程把账订满之后，真实空闲约等于「基线 − 驱动开销 2326 MiB − 已借」，容量是「基线 − 2560 MiB」，订满时剩的只有 2560 − 2326 = 234 MiB 上下，远低于 2048。能不能订满取决于额度：模板的清场额度写成「GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='local:0:24G peer:0:16G'」（`gpu-cards.env.example:17`），local 的 5090 空闲 32110 MiB，额度 24 G，订满后剩约 7768 MiB，够；peer 的 5080 总显存 16303 MiB（本轮开跑前资源提醒钩子读到第二台的 5080：总 16303 MiB、空闲 15839 MiB），额度 16 G 大于空闲，容量取 15839 − 2560 = 13279 MiB，订满后剩约 234 MiB：后起的进程在 peer 上起池就判红，不是等。默认额度（平时 8 G、没配额度时空闲的 3/4）订不满到这一步。**推的，没造**（要显卡与两个进程；checker 档）。

另两条同方向的、读码得到的（也是推的）：
- 没有排队次序：`book_waiting_for_room` 每一圈重新按此刻账判（「if waited >= self.settings.longest_wait {」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:546`），一个大借据在许多小借据此起彼伏时可以一直抢不到，等满 30 分钟报「本次未跑」，尽管总需求按次序是放得下的。
- 起池之后判器开草稿区按借来的额度与这个进程起池时读到的空闲里小的那个（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:226` 的 `memory_budget_mebibytes_of`，借来的卡额度在 `/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1038` 设成「借的量加 2560」）：后起进程读到的空闲若低于「借的量加 2560」，开出来的草稿区比订的小，订了没用满，别的进程白等一部分。

**（乙）账放得下而实际撞显存（「只防 OOM」一侧）：规则没说，三条推的。**
- 基线只在没有别的进程活着时重读：「if records.registered_processes.is_empty()」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-harness/src/gpu_memory_ledger.rs:430`）。进程接力（A 起、B 起、A 退、C 起、……）时基线永远是 A 当年读的；其间别的程序（本地模型服务等）把空闲占走一块，账不知道。清场约定之下没有这种别的程序，不清场时有。
- 起池里「先建设备、后登记」：设备在 `usable_gpu_cards` 里建（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:181`），登记在之后的 `open_the_ledger_of_the_started_cards`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1158` 所在函数）。登记之前这个进程的上下文已经占着显存而别的进程的账看不到它：别的进程在这段窗口里把账订到容量，加上两个上下文，合计可超过卡。窗口是毫秒到秒级，没造。
- 池外设备（K4-1 表）不在账里。

推翻它的现象：（甲）在 peer 的 5080 上第一个进程订满、第二个进程起池，没有判红；（乙）按账订满之后 `nvidia-smi` 的空闲仍大于 0 且没有 `NV_ERR_NO_MEMORY`。

### K4-4　7 份改走卡池的测试：借的量与它真开的缓冲对得上吗；扫源码检查能不能被换个写法绕过

借的量与真开的（读码对）：

| 文件 | 借 | 真开 | 判 |
|---|---|---|---|
| `crash_amplification_pipeline_records_checks_and_reuses.rs`（L1，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs:961`） | 不借（只排计划不判状态） | 起池一次；第二趟开 GPU 的放量另起设备（K4-1） | 借的量对；设备在池外（K4-1 窄冲突） |
| `crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs` | 3 ×（表 16 倍事实表字节 + 每状态结论与回读各 4 字节 + 1 MiB）（「const FACTS_VERIFIERS_ALIVE_AT_ONCE: u64 = 3;」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs:114`；「const LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD: u64 = 16;」，同文件 110 行） | 可撕标志一字节（「u8::from(*tearable)));」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_facts.rs:361`）上卡是 `RECORDED_WORDS` = 4 个字 = 16 字节（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_verify_gpu.rs:19`），正好等于上界 16；别的表每项展开得比它窄；用例里同时活着的核对器至多 2 个（健康流一个，加改坏的或半改坏的一个，`crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs` 162、241、246 行），借 3 份偏大不偏小。每个核对器在 `GpuFactsVerifier::new` 里自建管线，不走池里的管线缓存，管线占的显存不在账里 | 一致（偏大），管线没记 |
| `gpu_judge_root_choice_tie_…`、`gpu_judge_tree_identifier_watermark_…`、`gpu_recovery_root_choice_refuses_…`、`record_checker_judges_absence_…` | `judge_every_state_on_a_card_leased_from_the_pool`：借 `mebibytes_a_judge_needs` = 表 + 状态数 ×（草稿 `SCRATCH_WORDS` × 4 + 结论与回读 2 × `VERDICT_WORDS` × 4）（「tables.mebibytes() + (states * bytes_per_state).div_ceil(1024 * 1024)」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:157`） | 判器开表缓冲、草稿缓冲（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:561`，条数照 `states_per_dispatch`）、每条草稿缓冲一条结论缓冲（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:787`）与一条回读缓冲（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:800`）；借来的卡额度是借的量加 2560，`states_per_dispatch` 取三者小的，所以草稿区不会比订的大；每条草稿缓冲一个几十字节的派活缓冲没算进去 | 一致 |
| `gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` | `checks_at_once * mebibytes_crc32c_of_ranges_needs(2 * units_per_check)`（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs:49`），那个函数是「let bytes = range_count * (SLOT_BYTES + 4 + 2 * RESULT_ENTRY_BYTES) + PARAMETER_BYTES;」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/src/gpu_unit_checks.rs:245`） | 输入槽 `count × 32768`、长度字 4、结果 8、回读 8、参数 16 | 一致（逐项相等） |

扫源码检查（`gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs`）能绕开：它是逐行的文字匹配，三处可以绕：
1. 别名：`use …::usable_gpu_cards as cards_now;` 之后写 `cards_now()`，`use` 行没有「usable_gpu_cards(」，调用行也没有；经入口函数改名同样。
2. 注释剥法：每行按第一个 `//` 截断（「line.split_once("//").map_or(line,」，`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs:94`），字符串字面量里的 `//` 会把同一行后面的调用一起截掉。
3. 库源码里只认顶格恰为 `#[cfg(test)]` 的项（「if lines[position].1.trim_end() != "#[cfg(test)]" {」，同文件 109 行）；`#[cfg(all(test, feature = "gpu"))]` 这类写法下的测试代码不扫。
此外 7 份文件的「绕开卡池」清单只有三个字面（「"GpuCrashJudge::new(",」，同文件 44 行起），`GpuCrashJudge::with_states_per_dispatch(` 与 `on_card_with_packed_tables(` 不在里面；但这两个函数要一张 `GpuCard`，卡的来路只剩池借与清单里的另两个字面，所以这一支无害。上面三种绕法的后果是多一个设备不进账，即 K4-1 池外那一类。

推翻它的现象：上面三种写法之一写进一份 tests/ 文件，两条扫源码用例仍绿。（可由主 agent 在草稿副本里加一行别名 `use` 加一处调用跑这两条，checker 档。）

## K5　#9 钉值重取（F）

### 应然

钉值的来历写在测试文件的注释里：「2026-10-06 根环线第 10o 批合入（加 I-3.14，已实现的不变量 83 → 84 条）之后在主工作区（含删除线第 30 批）重取」（`/home/fy5090/.local/share/singlefs/work/m3-batch10o-code-r1/tree-r2/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:849`），新钉值打的行 `4b8b1e84…`、进度文件 `1d92a7ad…`，旧钉值 `4fc9cde7…`、`11977571…`。主 agent 的核法：把两趟 `GOLDEN_TALLY` 各一项 ` I-3.14=46/0/0` 删掉之后整段 SHA-256 等于旧钉值。问：这能不能证明「只多了 I-3.14」；进度文件那个值有没有把别的变化一起钉进去。

### K5-1　打的行那一半：一致（复核，原样输出）

材料是主 agent 草稿目录里留的那 58 行（`~/.local/share/singlefs/work/m3-r1/golden9/new-lines.txt`，已拷进模型目录 `golden-printed-lines-from-main-agent-golden9.txt`），命令在模型目录里跑：

```text
4b8b1e84e16b6a4ecd22a8bc28a15c75a5dbef3b99a1cfddc9e5b98703ed04b1  golden-printed-lines-from-main-agent-golden9.txt
行数 58  I-3.14=46/0/0 出现次数 2
删后 sha256 4fc9cde737d618c21d064f5cd8072b63130103b8533509c2cec93c09f5216ae8
```

读法：新 58 行的 SHA-256 等于新钉值；只删这两处子串（`GOLDEN_TALLY` 两行各一处，格式 `I-3.14=46/0/0` 与同行的 `I-3.8=46/0/0` 同形：评估 46 个状态、红 0、不适用 0）之后等于旧钉值，所以新旧 58 行逐字节只差这两处插入。它证的是「打的行里除了 I-3.14 那两项没有别的字节变」。

### K5-2　进度文件那一半：一致（复核，原样输出）；这一份「没有把别的变化一起钉进去」

进度文件值取自「只换了打的行钉值的副本」再跑，主 agent 的核法没有覆盖它（删除法只管打的行）。但那一趟的断言消息把整份进度文件（47 行：文件头加 46 个片行）原样打在 `run2.log` 里（`~/.local/share/singlefs/work/m3-r1/golden9/run2.log` 第 40–86 行），我把它拷出来，先核它自己的 SHA-256 等于新钉值，再把 46 个片行里 `checker_evaluated_states` 等三个字段里的 `I-3.14:…` 项删掉、按片行的格式（`<正文> checksum=<正文的 CRC-32C>`，`layer0_progress.rs` 的 `slice_line`）重算每行校验和、重算整份 SHA-256：

```text
1d92a7ad8c48c01f43dfe9d675d6256e110545b588d2bc5f6ad8601e63ed8cab  golden-progress-file-from-main-agent-run2.txt
片行 46 行，删掉 I-3.14 项 46 个
删后 SHA-256 119775719aa569efddcf00022a0ac5f1d70a2102af181e379a70aa6e4afd6cb5
旧钉值       119775719aa569efddcf00022a0ac5f1d70a2102af181e379a70aa6e4afd6cb5
相同：新进度文件与旧版只差 I-3.14 项
```

读法：文件值先对上新钉值（`1d92a7ad…`）；删掉 46 项（每个片行一项，一个片一个状态、各评估 I-3.14 一次）、重算每行校验和之后整份 SHA-256 等于旧钉值（`11977571…`）。所以这份进度文件与旧版的差别只有每个片行里多的那一项 I-3.14，没有别的变化（包括删除线第 25–30 批）被一起钉进去。脚本 `golden-progress-file-strip-check.py` 先核每行自己的校验和、再删项重算，换一份新取的进度文件也能直接跑（golden 子用例 `print_the_unsharded_enumerations_for_the_golden_comparison` 设 `SINGLEFS_SHARDED_TEST_GOLDEN_DIRECTORY=<目录>` 会把进度文件留在那个目录里）。

限度：这两份材料是主 agent 草稿目录里的副本，我没有在入库的树上重跑 golden 用例（checker 档，重型）；材料取自哪棵树是主 agent 报的（含删除线第 30 批）。

推翻它的现象：在入库的树上重跑 golden 用例，进度文件删 I-3.14 项后的 SHA-256 不等于 `11977571…`，或打的行删两处子串后不等于 `4fc9cde7…`。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 抬 F 顶上的根有没有越线的一形 | 一致 | 1818 段（S1 被抛弃根、有效根、4 GiB 有洞的环）红 0；10i 那一形（54 段红 8 / 被拒 42）今天 0 / 54；拿掉「顶上」的对照 44 / 54 红，探针能红 |
| K1-2 写掉那一槽就换类、记整条根记录三处同一份 | 一致 | 写根环槽的发布路径只有一处、记账替换只有 `record_root_written_by_this_process` 一处、`occupants.remove` 不碰新类；唯一留下旧记录的是发布失败退回，条款已定 |
| K1-3 T2「回退不拒」臂的量 | 量到（交用户） | 522 段一组：推 3 步今天拒 462、不拒 0；推 9 步 282 对 0；回退目标判错 0、checker 红 0（含回退后 12 步、4 GiB 盘），探针的判错对照 60 段能分辨 |
| K1-4 K1-d 留洞的环 | 一致 | 4 GiB 盘有洞的环 372 段 20 步红 0、抬 F 被拒 0；上限按 txg 排、不按槽位置 |
| K1-4 H-b 下的 I-5.1 | 一致 | 第 1 片指针与根里的实例表指针占同一个 (设备, 槽)，按 I-5.1 原文本来就红；harness 用例钉「对照加 I-5.1」，13 条通过 |
| K2-1 四形在六处读法里同口径 | 一致 | 24 格逐格对过，都是「这一片解不开」；另两处差别（链指针头部、第 k 片颗粒度）不改口径 |
| K2-2 行不收、链不往下走、I-9.29 门关 | 一致 | 每处都做到；门只在 `LastPage` / `chain == 0u` 分支里置，I-3.14 不成立的分支碰不到 |
| K2-3 H-b 键按读回那一份 | 一致 | CPU 与 GPU 同一个选法（先第一份、对不上才第二份），没有读回同一份而键不同、或读回不同而键相同的一形 |
| K2-4 I-3.8 不适用理由与 I-3.14 | 规则没说（窄冲突） | I-3.14 四形落在第 0 片时理由点名 I-3.14，对得上；条款写「第 0 片解不开」，读不出、身份不对、容器记录不合法三种走缺省理由；第 k > 0 片条款没写 |
| K3-1 7 个函数并之前并之后 | 一致 | 次序、`r_noting`、`r_candidate`、早退逐点对过，偏移 99 与 `JOURNAL_NEW_ROOT_SEGMENT_OFFSET` 一致 |
| K3-2 对拍 128 条能否说明 10q 没改判定 | 一致（读码补）；对拍单独不够 | 对拍够不着「版 + 下一次挂载那一版」「候选 ≥ 2 条带已释放」「早退边界」三类，由读码与 34-1 变异行补；我没跑（显卡） |
| K3-3 10s 调用点单测能否拦住再涨 | 一致，两处限度 | 调用点数 + 展开大小精确钉值拦得住大多数；它们是钉值（可改）、只盯七对；真硬线是 400_000 的预算用例；没跑（checker 档） |
| K4-1 容量会不会大于真能用的 | 冲突（窄）／规则没说 | 池内每卡一个设备；池外三处设备不进账（放量产品路径、起池时对未点名卡建设备、`#[ignore]` 两条）；驱动开销 2560 只在 5090 上量过（Q6 已知） |
| K4-2 跨进程账四问 | 一致 | flock 范围与次序对；判活（进程号加启动时刻）对；SIGKILL 约 1 秒回收；首次起池偏保守 |
| K4-3 甲 实际放得下而账让它等或红 | 冲突（窄，推的） | 后起进程起池时 `nvidia-smi` 实时空闲低于 2048 MiB 就把卡判不可用并 panic；额度写成整卡（模板里 peer 5080 的 `16G`）、第一个进程订满时可达；无排队次序、订了没用满另两条也是推的 |
| K4-3 乙 账放得下而实际撞显存 | 规则没说（推的） | 基线进程接力时不重读；先建设备后登记有窗口；池外设备不在账里 |
| K4-4 7 份测试借的量对不对 | 一致 | 借的量与真开的缓冲逐项对得上（核对器偏大，管线没记） |
| K4-4 扫源码检查能否被绕 | 冲突（窄） | 别名 `use … as`、字符串里的 `//`、非顶格 `#[cfg(test)]` 的写法可绕过，后果是一个设备不进账 |
| K5-1 打的行那一半 | 一致 | 新 58 行 SHA-256 等于新钉值；删两处 ` I-3.14=46/0/0` 等于旧钉值（复核，原样输出） |
| K5-2 进度文件那一半 | 一致 | 主 agent 草稿里的 47 行进度文件 SHA-256 等于新钉值；删 46 个 I-3.14 项重算校验和等于旧钉值，没有别的变化被钉进去 |

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决。
- 没跑 checker 档（`singlefs-checker-tier` 的测试）：K3 的两条库单测、K4 的卡池与扫源码用例、K5 的 golden 重跑（K5 用主 agent 草稿里已有的输出复核）；没有起任何要显卡的东西。K2 的 GPU 同判、K3-2 的对拍只读到码，写「复核不了」。
- 没造的形：K1-1 第 3 形（FUA 之后发布失败）、K4-3 甲（整卡额度、两个进程）、K4-3 乙的起池窗口、K4-1 池外设备（L1 用例加开 GPU 的放量同卡）。这些都标了「推的」。
- T2 臂的数是冻结副本拷贝上加补丁量的，按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」，判决要引时由主 agent 在入库装置上重做一次；`SONNET_T2_ARM` 里两个 `positive-control-*` 开关只为证探针能红，不是候选改法。
- 草稿目录交回前删：`~/.local/share/singlefs/work/m3-batch10o-code-r1-sonnet/`（仓的拷贝 `tree/`、编译目录 `tree/target/`、阶段日志 `out/`、报告草稿 `rep/`）；留下的证据在 `research/prompts/m3-batch10o-code-r1-sonnet-model/`。
