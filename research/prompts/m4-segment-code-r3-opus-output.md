# m4-segment-code-r3 攻方腿（Opus）报告（2026-10-16）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 H1 H2 H3 H4 H5 H6 F1 F2 F3 F4 F5 F6 F7 F8 F9 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 R1 R2 R3 R4 R5 R6 R7 R8 G1 G3 G4 G5 U2 C554 C685 -->

立场：正文 `research/prompts/_m4-segment-code-r3-body.md` 第四节攻方那一行。只在冻结副本的拷贝上造反例，拷贝上量出的数是副本上的数，不是入库装置上的数。自己提的改法都只在我的模型上看过、被攻过零轮。

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| Q1 前两轮站住的（新攻击面：一次发布写到一半崩溃，段内整写子集与原地覆写的撕裂态，恢复后可写挂载、池级 checker、续推整理） | **没打中**（一次抽样）；新攻击面上撞到一处红，属已知、用户认过的那一形 | 回退（F1）、带活意图的回退、整理各次发布、快照下整理、整理之后删快照，记录 / 根 / 系统配置那几段的崩溃状态全过；删快照那一条流 8215 个状态全量全过。日志重建出来、根槽没落盘的一版之后第 23 次发布那一刻 I-3.1 红在记账多的一边，是 `05-空间记账.md` 第 8 行「钉最旧 txg 的代价（用户认的）」那一形，不带整理的覆盖写历史上同样红，不分辨 m4 的修补 | 量的 |
| Q2 H5（RefFS 照 U2 取快照集合） | **打中（判据一侧，RefFS 单元层；翻译今天喂不到，推的）** | 一版在核心层不止一条根（可写挂载的写行与暖机、整理各批都映到现行那一版），被抛弃的是这一版后一条根时，恢复落到的仍是这一版；H5 取「之前那一版」，建快照之后挂一次就抛弃最新根，快照集合判成没有快照、读快照读出 A 判红。删快照（意图 + 批，多条根）之后抛弃最后一批，H5 只留「快照还在」，核心层下一次走空把它删完 | 量的（RefFS 上）+ 推的（核心层那一侧） |
| Q2 H4（计划补满） | 没打中（一次抽样） | 2,014 次整理发布（4 GiB 1407、384 槽 607）里照抄意图的一批 0 次一个没搬、0 次多于 K；带打印再跑 2,088 次，跳过本来就重写的节点那一支只走到 1 次，「只剩本来就重写的节点、全带上」那一支 0 次 | 量的 |
| Q2 H3 / H2 / H1 | 没打中 | H3：崩溃状态与随机历史里停在 `NoYieldFromTheLastIntent` 的都是一条意图腾一段、签发那次开新段那一形，属用户定「取签发之前」的后果；H2：只改注释；H1：带回退的随机历史 144 段（每步过 checker）与带活意图的回退崩溃状态里，模型与实现没有不合 | 量的 |
| Q2 实现员交下来的四处 | 一处打中（即 H5 一版多条根）；一处扩了后果的边（退路那一版与模型：故障注入的比重不抽整理，没有任何对拍历史会撞上）；两处没打中 | 见 Q2 一节的表 | 量的 + 推的 |
| Q3 清单全不全 | 四个形状两腿碰到过而没进任何一张表或没有去向 | `batches_published` 只数搬了单元的、用户改动之后「至多一批」那一趟发三次；强制进 deadlist 的纪元 0 条目；`publish_relocation_batch` 文档注释的准入句与行为不符；整理不进故障注入的比重。另记一处读窄：判决 T10 依据的「坏条目当损坏」在崩溃状态上没有反例，但前两轮都没枚举过发布写到一半的状态 | 推的（读码）+ 量的（一处） |

拷贝上量出的数都不是入库装置上的数；Q2 H5 那一处的改法写在 Q2 一节，被攻过零轮。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r3-opus-model/tools/rerun.sh \
  --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-segment-code-r3/repo --work <新的空草稿目录>
```

在这台机器上从零跑过一遍（release 从零编、跑三步，前台 235 秒内跑完），末行 `name=rerun_total same=3 differ=0`（第一遍是 `same=2 differ=1`：`out/known-pinned-threshold-steps.txt` 当时是原型改「已知形」标注之前那一版打的，换成定稿原型的输出之后三份逐字节相同，见「没做什么」）。随机批、删快照全量与小段批跑的其余几份不在复跑脚本里（各一批 1–5 分钟），起法写在各节与 `src/run_*.sh`。

模型目录各文件 sha256（与目录里 `SHA256SUMS` 同一份）：

```
c2e1790d0e2ad0d9401e5f0986ad2ae74a88e4caae0286c5cd01b602716017ad  ./out/compaction-batches.txt
055ad81ffd3bc45e8a34740e16b0d93d3b0b48d4b4053ab24c6cab9e4fd08c85  ./out/crash-destroy-snapshot-full.txt
ee2c3f540f5c26b9180fa7b0466263139896e77fa450ceb20529c2584a394b61  ./out/crash-small-rollback.txt
c8557a421b23de0922980e0bb75510d7983b8ef7137240ec705dcdf80442e468  ./out/crash-small-segments.txt
be0b582e8e69d77f43ed26cdf4a28efd2696b73f1ea5370f7a72a3f598af111b  ./out/known-pinned-threshold-steps.txt
900534a33e8ceef3457e6127c906c4387d5c2c627859eb10ceeeec24c8b7614e  ./out/reffs-abandoned-root.txt
54f4b514a24a061874b870f36e242ed1230d8e69f5bc38b6658ebb5bd7e0a4e4  ./src/attack_r3_compaction_batches.rs
6223927f69f4c323eecdbaebc9c784d783e56b5107a31c774f81dabf4ada0428  ./src/attack_r3_crash_states.rs
384731ac87267b6de3c9654381a5d27d5c6fb50a84655430d459e065fbd51bbb  ./src/attack_r3_reffs_abandoned_root.rs
688148cd4ee312c5b4f89c80a6e5d7cfbf28bf3afe4f84923d4771905f03cd3e  ./src/h4-trace.patch
c939aef145b5c7151fa2f96965a3767d6caf9033de0d46cd1ed062f98256fa20  ./src/run_batches_parallel.sh
de690b7c0c16006749092bc392d5924e5e089db53b290aaa7ed2ab3163cefb6d  ./src/run_small_segments.sh
7aa3fe84a1b281f312cb710449c135e422407b9dbadefb109f11848b79186ac2  ./tools/rerun.sh
```

## 开工核对

冻结副本根上 `sha256sum -c <仓根>/research/prompts/m4-segment-code-r3-snapshot/crates-sha256.txt`，原样输出（退出码 0）：

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

下面引代码写冻结副本里的路径与行号（冻结副本根 `~/.local/share/singlefs/work/m4-segment-code-r3/repo/`；引到的几份与工作树里逐字节相同，`cmp` 核过）。

## 装置

- 拷贝：`rsync -a --exclude target --exclude .git` 冻结副本到草稿目录 `repo/`，另拷仓根的 `.cargo/config.toml` 与 `.claude/heavy-thread-default`。release（工作区 profile 开着 `debug-assertions` 与 `overflow-checks`），每条 cargo 经 `capped.sh 31`，每次跑经 `run-with-memory-cap.sh`（8G，并行批每件 2G）。
- 原型（原样在模型目录 `src/`）：
  - `attack_r3_crash_states.rs`（harness 档测试目标）：自己造流——先跑前缀（录制流保留内容），再跑前缀 + 被攻的那一步，后一条流多出的那一截按屏障与 FUA 切段（`memory_pool::writes_and_segments`），每段展开整写子集，原地覆写（不是单元写、长于一个扇区、罩住旧内容）另取撕裂第三态（新旧不同的那一截前一半新、后一半旧，三条认法与取法照崩溃点重放那一组，在原型里自己写，不依赖 checker 档）。每个崩溃状态：冷读恢复（根要是模型认过的一版、内容与模型那一版相同、不早于被攻那一步之前最新的根）→ 可写挂载 → 池级 checker → 挂载之后还有活意图的接着整理推到停 → 再过池级 checker、再冷读一次内容不变。状态按（段、段内编码）寻址，可按序号分片。
  - `attack_r3_compaction_batches.rs`：随机历史（种子 = 本轮基数 + 9_000_000 + 下标；前两轮攻方用 + 7_000_000、+ 8_000_000），跑到底取最后一张镜像重挂，按 K = 1、2、3 与候选段强制整理、一次只放一次发布，逐次核 H4 的承诺与池级 checker；另一用法只跑历史、每一步过池级 checker、记结局。
  - `attack_r3_reffs_abandoned_root.rs`：RefFS 公开接口上的两格（Q2 H5）。
  - `h4-trace.patch`：拷贝里 `mounted_session.rs`、`compaction_plan.rs` 加的只打印的两行（`ATTACK_TRACE` 开着才打），不改逻辑。
- 崩溃枚举一共判了约 1.9 万个状态（各次 `ATTACK_TOTAL` 的 `judged=` 加起来 11,978，另有估时抽样约 7 千），远在约 10⁷ 的额度之内；没有一段历史全量超过约 10⁶ 而照跑的（超过的只跑了不含单元写的段，单元段交主 agent，见「要主 agent 跑的命令」）。

## Q1　前两轮判「站得住」的撑不撑得住——换的攻击面：一次发布写到一半崩溃

**结论：没打中（一次抽样）。** 新攻击面上撞到的唯一一处红是已知、用户认过的那一形，且不分辨 m4 的修补。

### 为什么这是前两轮都没用过的面

前两轮攻方都没调过崩溃枚举（两份报告「装置」一节各写着全量状态数 0），只在批与批之间整步崩：执行器那一种是 `crates/singlefs-harness/src/history.rs:579`：「/// 推到半路崩掉就是批间崩溃：重建出来的那一版带着意图、按映射条目的位置覆盖提示；执行器的可写挂载装产品开关，挂载里不续做」。执行器里唯一的写中途掉电截在记录之前：`crates/singlefs-harness/src/history.rs:534`：「/// 掉电截在第一条 journal 记录之前，那次改动没有任何一样能让恢复读到它的内容：交给模型与 RefFS 时与可写挂载相同。」——「记录落了、根槽没落、恢复由记录重建」这一形，随机历史一次都走不到。checker 档的回退崩溃用例只逐个取前缀、不在整理之后（它的文件头，没读正文）。这一节的状态是段内整写子集加撕裂态，接在整理之后的回退、带活意图的回退、快照下整理之后，崩完再可写挂载并续推整理。

### 跑了哪些形状（拷贝上，量过）

每个形状只判「不含单元写的段」（记录、根、系统配置那几段）与全落那一个；单元段那一大片（每次发布 2^14–2^20 个子集）没跑，交主 agent（见「要主 agent 跑的命令」）。例外是删快照那一条流，全量 8215 个状态全跑了。

| 形状（盘宽、前缀两次顺序写的单元数、强制选段、回退目标） | 全量状态数 | 这一轮判的 | 问题 |
|---|---|---|---|
| 整理推到停之后回退（384 槽，100，段 0，回退第 2、3、5 新） | 1,048,588 × 3 | 13 × 3 | 0 |
| 同上（4 GiB，150，段 1，回退第 3 新） | 1,048,588 | 13 | 0 |
| 整理推一批后崩、挂载，再回退（384 槽第 2、4 新；4 GiB 第 4 新） | 65,548 / 1,048,588 / 1,048,588 | 13 × 3 | 0 |
| 整理一趟推到停（384 槽 K = 1、2；4 GiB K = 1） | 4,997,341 / 16,793,788 / 13,893,721 | 241 / 205 / 97 | 9（都是下面那一形） |
| 推一批后崩、挂载，续推那一趟（384 槽） | 5,243,101 | 241 | 12（同上） |
| 快照下整理（384 槽） | 4,997,341 | 241 | 9（同上） |
| 快照下整理之后删快照（384 槽） | 8,215 | **8,215（全量）** | 0 |

原样摘录（模型目录 `out/crash-small-rollback.txt` 第一格、`out/crash-destroy-snapshot-full.txt`）：

```
ORDINALS from=0 to=1048588 judged_here=13
SUMMARY seg1 root=1:24 live=true resumed batches=10 noyield(15,15) count=1
SUMMARY seg1 root=1:25 live=true resumed batches=11 noyield(13,13) count=2
ATTACK_TOTAL shape=rollback-after-compaction states=1048588 judged=13 findings=0 first=[] every=1
ATTACK_TOTAL shape=destroy-snapshot-after-compaction states=8215 judged=4200 findings=0 first=[] every=1 seconds=159
ATTACK_TOTAL shape=destroy-snapshot-after-compaction states=8215 judged=4015 findings=0 first=[] every=1 seconds=151
```

`seg1 root=1:25 count=2` 是回退那次发布两份记录落了一份、根槽没落：恢复由记录重建出回退那一版（txg 25），可写挂载、续推整理 11 批、checker 都不红。回退目标带活意图（384 槽那一批里 prefix 的整理停在发布预算、意图活着）时，回退那一版重建出来也带着意图，续推照常收。

### 撞到的那一处：日志重建的一版之后第 23 次发布那一刻 I-3.1 红（已知形）

整理一趟（384 槽、K = 1）里第 2、5、8 批（txg 7、10、13）的记录落了、根槽没落（或两份记录只落一份）时，恢复由记录重建出那一版、可写挂载、续推 20 批，最后那一张镜像 I-3.1 红：

```
STATE ordinal=540696 seg=10 effective=Some((InstanceGeneration(1), CheckpointTxg(7))) journal=JournalScanReport { valid_records: 106, above_water: 1, prefix_applied: 1, verification_passed: 1, verification_failed: 0, maximum_applied_transaction: 104 }
STEP 19 batches=1 versions=1 txg=Some(CheckpointTxg(30)) stop=PublishBudgetSpent { budget: 1 }
CHECK after-step-19 reds=1
  RED 已知形（钉最旧 txg，记账多）I-3.1: 盘 0：记账的已分配 Some(7176192)，遍历全部有效根得到 7012352（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 30、环里自证过的根槽 23 个、最老的自证过的根 txg 8、遍历的候选根槽 23 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个
CHECK after-step-20 reds=0
```

（`out/known-pinned-threshold-steps.txt`，逐步过 checker：只在 txg 30 = 7 + 23 那一张红，前后都绿。）同一形在不带整理的历史上照样出：4 GiB、覆盖写一次、记录落了根槽没落，之后再覆盖写 30 次，txg 28 = 5 + 23 那一张红、记账多 163,840 字节（同一份文件后半）。

判它是已知形：红在记账多的一边、环里自证过的根槽 23 个少于环长 24（从没落盘的那一版留下的洞），正是 `crates/singlefs-checker/invariants/05-空间记账.md:8`：「⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1」那一句往下说的「这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止」（那一行很长、里面套着引号，只抄到第一个内层引号之前）。欠账 `.claude/kb/checks-owed.md:459`：「① 由记录施加出来、根槽从没落盘的那一版（r 是这种版本、之后 F 抬过它时，它既不在根环里也不在候选集里，原式不并它引用的槽；那时被抛弃根还引不引用得到这一槽没推完）」是相邻的一格（那一格说的是 F 抬过它时）。

四句：
1. 分不分辨臂：不分辨 m4 的任何一处修补（不带整理的覆盖写历史上同形）。
2. 被判的系统看不看得到：看得到——实现按 txg 窗口留着那一版的槽，checker 只走盘上读得出的根。
3. 满足的字面：I-3.1「已分配空间统计 == 实际遍历所有引用得到的和」；被同一行的「钉最旧 txg 的代价（用户认的）」那一句收下。
4. 跑前条款给的改法：这一轮没有给这一形的改法；不算打中，只记「新攻击面在合法崩溃之后接 23 次发布就撞得到这一形，随机历史与崩溃点重放都走不到（前者截在记录之前，后者不续推 23 次）」。

### 各格逐个（Q1 点名的）

| 前两轮判「站得住」的 | 这一轮的面 | 结果 |
|---|---|---|
| F1（回退只重写位置变了的路径） | 回退那次发布记录 / 根 / 系统配置三段的崩溃状态 × 7 个形状，崩完续推整理 | 没打中；回退那次发布的单元段没跑 |
| F4（强制开关） | 没攻：开关挂在测试自己造的写入口上，会话与崩溃流都够不着 | 没攻 |
| F5（空文件集合上建快照） | 没攻（前缀里没有删完文件那一步） | 没攻 |
| F6（RefFS 恢复后的快照集合） | 读码：崩溃落在删快照那一步里时 Q 里两个下标都收，读出的 A 两边都对，「意图 + 批」两条根的那一形不缺；同一个「一版多条根」的问题落在 H5 那一支（Q2） | F6 那一支没打中（推的） |
| T7（R 排他） | 崩溃状态上挂载按 logged_ops 装回活意图（`live=true`），续推每一趟都收 | 没打中 |
| T8（discard 少发不补） | 录到的流里没有 discard 写（种类只有单元、记录、根、系统配置） | 没攻到 |
| T10（坏 logged_ops 条目拒可写） | 约 1.2 万个崩溃状态，可写挂载一次没拒 | 没打中（单元段没跑：坏条目只会出在单元写里） |
| R8 六格（T1、T2、T3、T7、T9、T10） | 删快照全量 8215 个状态（T3）、快照下整理（T2）、续推整理（T7、T9） | 没打中；T1 建快照那次发布没枚举 |

什么现象会推翻「没打中」：单元段那一大片（交主 agent 的分片）里有一个状态冷读走到模型没认过的根、可写挂载被拒、checker 红在已知形之外，或续推整理之后内容变了。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这一格的「没打中」还要再抽一次才算。

## Q2　修复批三的修补本身

### H5（RefFS 抛弃最新根之后的快照集合）——打中（判据一侧，RefFS 单元层）

H5 的取法写在 `crates/singlefs-reference-filesystem/src/lib.rs:709`：「/// 主 agent 照 U2 定的读法）：允许集合里每一个状态都可能是被抛弃的最新根那一版，各取它之前那一版、并起来。」，「之前那一版」按 `crates/singlefs-reference-filesystem/src/lib.rs:735`：「/// S_j 那一版之前那一版的下标：上一次恢复之后、j 之前最后一次已确认发布（返回成功的发布调用、fsync、回退、建快照、删快照）的末尾；」取。这把「被抛弃的根」等同于「写出 S_j 那一版的那条根」。核心层一版常有几条根：翻译层自己把可写挂载与整理写的根映到现行那一版——`crates/singlefs-harness/src/reference_filesystem_translation.rs:408`：「// 可写挂载与抬 F 不改内容：不进 RefFS 的操作序列（H1 只让发布调用、fsync、回退抬 lo），它们写的根映到现行版本。」、`crates/singlefs-harness/src/reference_filesystem_translation.rs:414`：「// 先走空的删除续做只放已删的文件与已删快照的块；批间崩溃之后的可写挂载照可写挂载。不进操作序列，写的根映到现行版本。」；删快照在 RefFS 里是一个发布末尾（`crates/singlefs-reference-filesystem/src/lib.rs:198`：「EnteredKind::CreateSnapshot | EnteredKind::DestroySnapshot => true,」），核心层是意图那次加各批、至少两条根（本轮删快照那条流录到两次发布，`out/crash-destroy-snapshot-full.txt` 的 `segments=8 sizes=[12, 2, 1, 2, 12, 2, 1, 2]`）。

复现（RefFS 公开接口，`src/attack_r3_reffs_abandoned_root.rs`；原样 `out/reffs-abandoned-root.txt`）：

```
CASE mount-roots allowed_snapshot_sets=[{}]
CASE mount-roots allowed_reads_of_7=[None]
CASE mount-roots read_7_as_A=Err(SnapshotReadOutsideTheAllowedReads { name: SnapshotName(7), read_out: OneFile(FileContent([65])), allowed_reads: [None] })
CASE destroy-batches allowed_snapshot_sets=[{SnapshotName(7): OneFile(FileContent([65]))}]
CASE destroy-batches allowed_reads_of_7=[Some(OneFile(FileContent([65])))]
CASE destroy-batches recreate_7_after_the_walk=Err(SnapshotNameAlreadyTaken { name: SnapshotName(7) })
```

- 第一格：写 A、发布、建快照 7（RefFS 序列到此为止）；核心层随后可写挂载一次，写行与暖机的根都映到建快照那一版；抛弃最新根（暖机那一条）的那次恢复落到它前一条根，仍是建快照那一版、快照 7 在、读出 A（推的：写行与暖机照抄树表）。RefFS 的输入与它自己的单测 `a_recovery_that_abandoned_the_newest_root_written_by_a_snapshot_creation_restarts_without_that_snapshot` 逐字相同，交回「没有快照」，读快照 7 读出 A 判红——合法状态上误红。整理推过几批再抛弃最新根、或为空间推过抬 F 的那几条根，同形。
- 第二格：建快照 7、删快照 7（RefFS 一个操作），抛弃最后一批那条根，恢复落到意图那条根（快照 7 带销毁中记号、读照读），核心层下一次用户改动先走空、把它删完；RefFS 只留「快照 7 在」，之后不再变。今天执行器读快照只取不在销毁中的（`crates/singlefs-harness/src/history.rs:488`：「/// 读、删哪一个快照：执行时按现行那一版树表里不在销毁中的码 14 条目（按（诞生 txg, 树号）升序，即建的先后）取第」），核心层的树号不复用，这一格多半只留一个读不到的幽灵快照，没造出判红的后续（推的）。
- 今天的翻译喂不到：执行器那一步在 C554 乙之后不再造出被抛弃的根，`crates/singlefs-harness/src/history.rs:517`：「/// C554 乙之后读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），」；翻译里调 H5 那一支的只有 `crates/singlefs-harness/src/reference_filesystem_translation.rs:636`：「.restart_after_a_recovery_that_abandoned_the_newest_root(crash, read_out.clone())」。所以后果停在 RefFS 单元层，与第二轮 R6 同一层（推的）。

四句：
1. 分不分辨臂：分辨「之前那一版」（H5）与「被抛弃的根可能是这一版自己的后一条根」两种读法；第一格与 RefFS 自己那条单测输入逐字相同、答案必须不同。
2. 被判的系统看不看得到：**看不到**——RefFS 只收操作序列与读出的内容，哪条根被抛弃不在输入里（`evidence-discipline.md` 判据表「判别子观测不到」那一形）。翻译看得到（它有根到版本的表）。
3. 满足的字面：正文第二节 H5「一版不止一条根时『之前那一版』照字面取」与实现员交下来的「H5 那两处读法」第二处。
4. 改法（只在我的模型上看过、没实现没跑、被攻过零轮；下面各格都是推的）：甲，每个 j 取「S_j 自己那一刻的」与「之前那一版的」两份并起来——两格都不再误红（推的），代价是第二轮 R6 那一形（被抛弃的确是建快照那条根）判别力变弱：核心层留着快照也不红；乙，翻译把「被抛弃的根是不是写出这一版的那一条」交进来（翻译有 `NewVersionWrittenBy`），RefFS 按它取一份，删快照这种一个操作多条根的另取前后两份——两格都修（推的），要改 `restart_after_a_recovery_that_abandoned_the_newest_root` 的签名。

什么现象会推翻它：入库装置上第一格的序列里 RefFS 交回的快照集合含快照 7；或者核心层在可写挂载之后抛弃最新根，落到的那条根的树表里没有快照 7。

### 实现员交下来的四处

| 第几处 | 攻法 | 结果 |
|---|---|---|
| 两条退路照抄出来的那一版模型判不合 | 找对拍历史里谁会撞上它：故障注入的比重不抽整理（`crates/singlefs-harness/src/fault_injection.rs:1943`：「// 挂着时整理只进它自己那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`），故障注入用的几组比重不抽它；」），修复批的 H2 用例只核会话、不过模型 | 不越过已知清单；记一处边：瞬时读错 × 整理在任何带模型对拍的历史里都没有，退路那一版与模型的冲突今天是哑的（推的） |
| 只剩本来就重写的节点那一批搬的个数可能超过 K | 带打印跑 2,088 次整理发布（下表） | 没造出；那一支 0 次 |
| H5 没有已确认发布那一支（取 S_0 的） | 读码：上一次恢复之后可写挂载至少写一条写行的根，都映到 S_0 那一版；被抛弃的若是它们之一，落到的仍是 S_0 那一版 | 没打中（推的） |
| H5 一版多条根 | 见上 | **打中**（RefFS 单元层） |

### H4（计划补满）——没打中（一次抽样）

本来就重写的角色在代码里是 `crates/singlefs-core/src/transaction.rs:7503`：「/// 这次重写本来就换下的角色（记账树、分配记录树里内容变了的节点）不另搬：它们的新落点本来就不在活 R 里。」；只剩它们的那一支 `crates/singlefs-core/src/compaction_plan.rs:249`：「/// 删意图，它们不受批大小限，因为本来就重写的节点不是搬迁。剩下的只有钉住、隔离、已死、不一致的交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。」。会话的再计划循环靠 `crates/singlefs-core/src/mounted_session.rs:2555`：「"被拒时点名的 key 都在这一次的计划里，计划不取已跳过的：跳过的集合每被拒一次至少多一把 key"」那一条断言收敛。

随机历史（比重 `COMPACTION_WHILE_MOUNTED_WITH_ROLLBACK`，种子 + 9_000_000 起）跑到底、重挂，K = 1、2、3 × 候选段，一次放一次发布，每次之后过 checker。原样（`out/compaction-batches.txt`，每批 24 件、24 个 `.rc` 都是 0）：

```
### batch 2 rc_files=24 rc_nonzero=0
summed completing=168 reds=0 carry_zero=0 over_k=0 publishes=1407 carry_batches=1083
### batch 3 rc_files=24 rc_nonzero=0
summed completing=180 reds=0 carry_zero=0 over_k=0 publishes=607 carry_batches=253
### batch 4 rc_files=24 rc_nonzero=0
summed completing=168 reds=0 carry_zero=0 over_k=0 publishes=1407 carry_batches=1083
      1 TRACE h4 skip keys=1
### batch 5 rc_files=24 rc_nonzero=0
summed completing=210 reds=0 carry_zero=0 over_k=0 publishes=681 carry_batches=270
```

批 2、4 是 4 GiB 下标 10–57（批 4 是批 2 加打印重跑），批 3 是 384 槽下标 100–147 取前三个候选段，批 5 是 384 槽下标 200–247 取全部候选段。`carry_zero` 是照抄意图的一批一个没搬，`over_k` 是搬的多于 K。批 1（16 件、下标 10–57）有一件被前台时限截断、`.rc` 只有 15 个，整批作废，没算。

为什么那一支走不到（推的）：记账树每次发布整批重写，R 里的记账树节点永远是「本来就重写」、从不真搬；要搬的个数超过 K，得有一个单元在先前那一遍试算里是本来就重写、在最后那一遍（计划罩住 R 里全部）里变成真搬，而最后那一遍的计划包含先前每一遍的，改动的记录只多不少。没造出取样点，「超过 K」仍是推的。

### H3、H2、H1

- H3（`segs0` 取签发之前）：崩溃状态续推的每一趟都停在 `NoYieldFromTheLastIntent`，摘要里 `noyield(15,15)`、`noyield(13,13)` 是全空段数与 `segs0` 相等——一条意图腾一段、签发那次自己开新段时就是这样，是用户定「取签发之前」的后果（判决第二轮交用户表第 1 行），不算打中。
- H2：只改注释，见上表第一行。
- H1（模型回退只取目标版的搬根记录）：带回退的随机历史 144 段（4 GiB 下标 300–371、384 槽下标 400–471，每段 40 步、每一步过池级 checker）全部 `Completed`、checker 0 红（`out/compaction-batches.txt` 批 6、7）；带活意图的回退那几格崩溃状态也没有模型以外的红。没打中。

## Q3　开着的清单全不全

前两轮判决的交用户表、改法、欠账与「第三轮清单」里都没有、而前两轮的腿碰到过的形状（逐条在这一轮现查过代码还在）：

| 形状 | 谁碰到过 | 今天的代码 | 后果（量的还是推的） | 进没进表 |
|---|---|---|---|---|
| 一趟整理的批数只数搬了单元的那几次发布，用户改动之后「至多一批」那一趟因此可以发三次（腾空 R、一个没搬就删意图的那一批不计，接着签发、再搬一批才停） | 第二轮攻方 R2「推的、没造」；第二轮辩方 §3.2「读码推的，没造」 | `crates/singlefs-core/src/mounted_session.rs:2071`：「if !version.units_relocated_by_this_publish.is_empty() =>」；停判 `crates/singlefs-core/src/mounted_session.rs:2042`：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」 | 推的：H4 之后照抄意图的一批都至少搬一个，剩下不计批的只有「R 里只剩本来就重写的节点、一个没搬就删意图」那一批（本轮 2,088 次里那一支 0 次）；用户改动之后那一趟多吃两次发布与两个根槽，仍被 `B_gc` 罩着；产品路径不起跑 | 没进（第二轮判决 R2 一行只写退路与 H4） |
| 强制进 deadlist 那一臂造得出纪元 0 的条目，之后建的快照都放不掉它 | 第二轮攻方 R4（推的）；辩方 R4 只记「开关挂写入口、随机历史够不着」 | 开关只在测试自己造的写入口上（第二轮两腿同读） | 推的：只在测试里；用例若在强制臂之后再建快照、删快照，会留下删不掉的条目 | 没进（R4 一行只收辩方那一句） |
| `publish_relocation_batch` 文档注释的准入句与行为不符 | 第一轮正推 T5（「文档注释写『式子那一判照判』与行为（普通批不判）不符」） | 注释还在：`crates/singlefs-core/src/transaction.rs:7505`：「/// 照删除的三种发布同一张计划：不写文件内容、不建 inode、实例表照抄、不发树号；准入同删除类（式子那一判照判，合取 7 免）。」 | 推的：注释误导；行为没量 | 第一轮判决 T5 一行写了「注释与行为不符」，改法与欠账里没有去向，第二轮没提 |
| 瞬时读错 × 整理从没在带模型对拍的历史里出现过 | 第二轮攻方 R2 用自己的注入层量到退路；第二轮判决改成 H2 用例（只核会话） | `crates/singlefs-harness/src/fault_injection.rs:1943`：「// 挂着时整理只进它自己那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`），故障注入用的几组比重不抽它；」 | 推的：退路发出的那一版模型判不合（实现员已交），可没有一条对拍历史会走到，冲突是哑的；哪天把整理加进故障注入的比重，第一次就会报新发现 | 没进 |
| 「记录落了、根槽没落」之后接 23 次发布：I-3.1 红在记账多的一边 | 本轮新撞到（零轮）；欠账 C685 ① 是相邻一格 | 见 Q1 一节 | 量的；已由「钉最旧 txg 的代价（用户认的）」收下 | 已知形；只补一句：随机历史与崩溃点重放都走不到这一形（前者截在记录之前，后者不续推） |

判决依据读没读窄：第二轮判决「第一轮判决复核」一行说 T10 站得住的依据是「可写挂载拒成 `UnitMalformed` 而冷启动读得出同一条根」；这条依据默认坏条目只来自损坏。前两轮都没枚举过一次发布写到一半的状态；本轮约 1.2 万个崩溃状态里可写挂载一次没拒，没造出「合法崩溃留下坏条目」——但 logged_ops 的节点只出现在单元写里，单元段那一大片没跑，这一句还没被检验到底。

## 没打中的形状

- 崩溃状态（记录、根、系统配置三段与全落）：回退在整理之后 4 个形状、带活意图的回退 3 个、整理一趟 3 个、续推那一趟 1 个、快照下整理 1 个，加删快照全量 8215 个状态；问题只有 Q1 那一处已知形（30 个状态，分在三个形状里）。
- H4：4 GiB 两批、384 槽两批，K = 1、2、3，照抄意图的一批一个没搬 0 次、多于 K 0 次。
- H1：带回退的随机历史 144 段每一步过 checker。
- H5 没有已确认发布那一支：读码，没造。
- 试过没跑起来的：回退那次发布的单元段（每个形状 2^20 个子集），按估时 25–44 个状态每秒（31 线程，见下），一个形状要 7–12 小时，交主 agent。

## 这条腿自己的限度

- 崩溃枚举只跑了不含单元写的那几段与删快照那一条全量；单元段的子集状态（每次发布 2^14–2^20 个）这一轮一个形状都没跑完，没打中的结论只罩到跑了的那几段。
- 判据是冷读恢复对模型认过的版本、可写挂载、池级 checker、续推整理：它们一起漏的看不见；没比 RefFS，崩溃之后的快照集合没判。
- 原地覆写的撕裂态照崩溃点重放的三条认法在原型里另写了一份，没与 checker 档那一份逐状态对过数（这条腿不碰 checker 档）。
- H5 那一处的核心层一侧是推的：没在核心层上造「可写挂载之后抛弃最新根」的镜像。
- 每一格「没打中」都是一次抽样。

## 没做什么

- 没起门禁、没跑 checker 档、没起 GPU、没碰 `crates/singlefs-checker-tier/`（只读了三份崩溃用例的文件头，确认它们没罩这一轮的面）。
- F4、F5 没攻；T8 的 discard 在录到的流里没出现，没攻到；建快照那次发布没枚举。
- 估时的那几次抽样（每 1000 / 2000 个状态判一个）只用来估时，不算进判过的状态。
- `out/known-pinned-threshold-steps.txt` 最初那一份是原型加「已知形」标注之前打的，复跑第一遍对不上（`same=2 differ=1`，只差标注前缀与多一行 `SEGMENTS`），换成定稿原型复跑的输出后再从零复跑一遍 `same=3 differ=0`；数没变。
- 复跑脚本只罩三份；`out/compaction-batches.txt`、`out/crash-small-segments.txt`、`out/crash-destroy-snapshot-full.txt` 要重跑用 `src/run_batches_parallel.sh`、`src/run_small_segments.sh` 与下一节的命令（两份脚本里写死了我草稿目录里二进制的路径，用时换成复跑目录里编出来的那一份）。

## 要主 agent 跑的命令

单元段的全量（这一轮没跑的那一大片）。先跑一遍复跑脚本得到编好的拷贝，再在它的 `tree/` 下按序号分片跑；每片 31 线程，按这一轮估时约 25–44 个状态每秒（384 槽、带续推）：

```
cd <复跑草稿目录>/tree
B=$(ls target/release/deps/attack_r3_crash_states-* | grep -v '\.d$' | head -n 1)
ATTACK_ORDINAL_FROM=0 ATTACK_ORDINAL_TO=65548 ATTACK_STATE_LIMIT=1100000 ATTACK_THREADS=31 ATTACK_VERBOSE=1 \
ATTACK_WIDTH=384 ATTACK_UNITS=100 ATTACK_REGION=0 ATTACK_ROLLBACK=1 ATTACK_SHAPE=rollback-with-live-intent \
  bash <仓根>/scripts/hosts/run-with-memory-cap.sh 8G "$B" crash_states_of_one_shape --nocapture
```

- 带活意图的回退（上面那条，65,548 个状态，推的约 25–45 分钟）先跑；整理之后的回退（`ATTACK_SHAPE=rollback-after-compaction ATTACK_ROLLBACK=1`，1,048,588 个状态）略超「每段约 10⁶」，跑不跑由主 agent 定，跑就按 `ATTACK_ORDINAL_FROM` / `ATTACK_ORDINAL_TO` 切十片。
- 看哪几行：每片末行 `ATTACK_TOTAL shape=… states=… judged=… findings=…`；`findings` 非 0 时看 `PROBLEM` 行，开头写「已知形（钉最旧 txg，记账多）」的是 Q1 那一形，别的才算线索。
- 内存 8G、线程 31（`ATTACK_THREADS`），工作目录是复跑目录里的 `tree/`，不需要别的环境变量。

## 草稿目录清理

交回前删掉了草稿目录 `~/.local/share/singlefs/work/m4-segment-code-r3-attack/` 里的仓拷贝 `repo/`（删前 `du -sh` 981M：编译目录 `repo/target/` 953M、`repo/crates/` 29M）与两次复跑试跑的目录 `rerun-test/`（834M，其中编译目录 805M）、`rerun-test2/`（834M）。留下的是原型与补丁工作稿 `attack/`（328K）、日志 `logs/`（1.5M）、`progress.md` 与开工核对的 `sha-check.txt`，共 1.8M；原型、补丁与复跑脚本已原样进模型目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-segment-code-r3-opus-output.md`（以工作树为根；被引的几份与冻结副本逐字节相同）末行：

```
  ✓ 核了 20 处引文、0 处裸行号，对上 20 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```

以冻结副本为根（`--root ~/.local/share/singlefs/work/m4-segment-code-r3/repo`）时 19 处对上、1 处没判：C685 那一行所在的 `.claude/kb/checks-owed.md` 不在冻结副本里（冻结副本只拷了 `crates/`）。
