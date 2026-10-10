# m3-batch10z-no-code-r1 攻方腿报告（第 10z 批第十四、十五块代码轮第一轮，2026-10-09）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 H1 H2 R P C F -->

## 复跑

工作目录仓根，`WORK` 给一个空的草稿目录（不在仓里）：

```
WORK=~/.local/share/singlefs/work/<自己的目录> bash research/prompts/m3-batch10z-no-code-r1-opus-model/rerun.sh
```

它把冻结副本 rsync 成三份（`copy` 入库原样加探针；`pre14` 把第十四块 core 两份文件的改动按改动全文第 893–1402 行倒推回去再加同样的探针；`k4copy` 打 K4 的遮蔽写法），release 编，按 `jobs/` 里的件表分批跑（每件经内存包装 1500M、单线程，一批至多 8 件），汇总进 `$WORK/summary/`，最后与 `runs-summary/` 逐份比（秒数不比）。只跑 harness 档、core 库单测与 `cargo check`；不跑 checker 档测试、不起 GPU、不跑崩溃枚举。量都是副本上的数。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum`，写报告时现算）：

```
ecafe2cc6dba2d756ea2396234bb3ac22dda8dbfda9d33da681a58cc904e655b  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-m1a.txt
b2f20509134f646ca0a28bc1dd90b8d3ee5448a828e8cd47cb9ef1f2a70ccea9  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-m1b.txt
3203b32f561b066b783c8e53ff966943f32d07a7d7ae97aa174ca2b9418faf38  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-n1.txt
9569cbb2bf81bfa89bcfe02c0dc2d9eeb1ec4d9131d5c422c791edf17a515c14  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-s1.txt
a25bbfa0da47876e7bbeb2eaed0babcdfe90cf816f3a614b2b044b189a65f958  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-s2.txt
be78f747da10be9a71792b6ce1d1247eef521f425926a9ee661596204ceac389  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-s3.txt
f227a357872e1b80cf8f76ae4029da0bd7110fcda02d5f9c8e7de8c5bfb9fe25  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-s4.txt
903e64af0fc4211a9fc8c2f1480d6f0f9674d42dd902d62ec0d72f84e6eb9a66  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-u1.txt
f5c919ddebe17a1fedacd5d0ea836b03bf4333a9b168b675184fae77ff9fa6b0  research/prompts/m3-batch10z-no-code-r1-opus-model/jobs/jobs-u2.txt
74334379b3a618069922eb2526adc45ffdba8caca6cc6e6f50f8224e7e35c895  research/prompts/m3-batch10z-no-code-r1-opus-model/rerun.sh
54e237b652b7c4a02e9429fb22c7db3b30f4dc90b37380dceced58b42b76e87b  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/arms-m1-vs-today.txt
8562581b737c3a75eebf23e4090db03924bce565e7a0f684fa28ce7156f986b2  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/gpu-two-card-model.txt
98704fa34cea7da035c3bb7f32f27c7e6e792948891ce2474096ece9b630011a  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/gpu-two-card-stress.txt
10e210206802f138a92282affd91da9b89db8ca6c01f8526e0b1114dce490be6  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/k2-tests-under-arms.txt
43f57fabd002df4a2dbfafb244b8d1407459a13d78eddc7f7b637991d670b270  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/k4-ledger-mutants.txt
bd95a93791dd6372f6ffcbb446efa8d55be73b0746660b1fc451ec6e2acc55eb  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/k4-shadow.txt
4108196f372269351ba931eff07ddb3a3d0c7a43687166e69ba6abe7ffaeac1f  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/pair-n1.txt
ff34b736e6743469ec404c7d034041917d03f8d6dd20981628a55f3ada2fc7f8  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/pair-s1.txt
d28ce32e2d3d1158ba99a37c72365c95b68ee6c0d03cabe98bc7d197abe33703  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/pair-s2.txt
0e5dd33c80cfa3d16d5370c5494eb7227cd96d4ba82e6281a704d761443ed237  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/pair-s3.txt
f90223b0a6517e1f8d982821c9f5e9cb480962c25e22df735dcd92a1b90e49f7  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/pair-s4.txt
491866648cc7c9b9f047cba0a4bd06cf888940c0a836d3138c5c621e25b51888  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/red-classes-n1-pre14.txt
491866648cc7c9b9f047cba0a4bd06cf888940c0a836d3138c5c621e25b51888  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/red-classes-n1-today.txt
2dcae091d9719dd037c57e39838829e491a29ae73ce6fc762bc930c321b7f20c  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/red-classes-s1-today.txt
d3ba3c4b21146b561ad32af35f534ba4fd935b17ab4c888e246a1c2a6406fa03  research/prompts/m3-batch10z-no-code-r1-opus-model/runs-summary/unmount-branch.txt
1021df15fda7016dd83a234e1151b2714a7e8a931ef46827d8c6e40a1313bb3f  research/prompts/m3-batch10z-no-code-r1-opus-model/src/batch.sh
67213eabecd57b36f609362ddb8bb22a17b68631c198258791239c9f82b50a2d  research/prompts/m3-batch10z-no-code-r1-opus-model/src/classify_red.py
17b098c23fc5c77be7290fab3e23a9ab2806964185cf445923f88603542e38b7  research/prompts/m3-batch10z-no-code-r1-opus-model/src/compare.py
555f7ce24c4eba7cb150089595b30753d7ca4de33e420d8b9ae66d251913ad4b  research/prompts/m3-batch10z-no-code-r1-opus-model/src/gpu_model.rs
55c6f7a104f10fb15fa8b8d8be1bbb5cf2dc2d64c7576f1ddb0f62b3bce62fb8  research/prompts/m3-batch10z-no-code-r1-opus-model/src/gpu_patch.py
866dec8c9099ad72831bae134dc7e79fb785f50f5e20d728846b4bf2992e6b79  research/prompts/m3-batch10z-no-code-r1-opus-model/src/gpu_stress.rs
eaba314995de7aceb32b19df0e778e4a9af0aeb0f5d421c9f3f27f66e0055b28  research/prompts/m3-batch10z-no-code-r1-opus-model/src/k4_check.sh
6305446b410a8ebbed035fa03c7851a2205785bad239c9e4d8a7407169ad89bb  research/prompts/m3-batch10z-no-code-r1-opus-model/src/k4_shadow.py
eb02c66de0bf5466ee908d2ad06e41645884558be45e82e220e5ce9801f742d9  research/prompts/m3-batch10z-no-code-r1-opus-model/src/opus_no_probe.rs
7ee6a26947c49173f0489e9acbfdd8f6e61a24bb02c130e7fc6b7a9bd55ba3ee  research/prompts/m3-batch10z-no-code-r1-opus-model/src/pair.py
da7f8ecf767d5e08f7397069f2300f73d2cfd997390dbfc0c624952646f8ebc2  research/prompts/m3-batch10z-no-code-r1-opus-model/src/patch_probe.py
a39c2d82f3909d1962eff4078f8f7876c5f323243cbf121fd2d5c4f46cc5bf83  research/prompts/m3-batch10z-no-code-r1-opus-model/src/scan_port.py
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 回收路补不补得上隔离 | 没打中（造出来的扫法） | 回收出口只有 `reclaim_released_records_up_to` 一处；三条用它的路（记根、抬 F 的准入支、抬 F 的卸载支）都补；随机历史 2168 段（s1–s4、m1、u1、u2、n1 各批，计数见下面「结果」）与改前副本逐段配对，红的段两边一个不差；「护着而空闲」的槽除 K1-2 那一行的同一起点两个跨度那一形（两臂相同）外 0 |
| K1-1 旁证：改前改后同红的一形 | 线索，不算这一轮打中（不分辨臂） | R 的根槽与 R 的账挂上之后一直读回全零、R 的记录齐、不再挂：R 引用的单元不在 C 里，回收之后发出去，I-7.4 红；第十四块改前副本同一批 68 段里同样 16 段红 |
| K1-2 按槽跳过 / 按起点槽豁免叠加 | 没打中 | 起点槽豁免而跨度里有槽不在现行账里的，计数 0；同一起点两个跨度只隔离了窄的那一个出现过（1536、6144 两档），两臂相同、多出来那一槽不被环里的根引用、没红 |
| K1-3 改完之后少扣 / 重复扣 | 没打中 | 「护着而空闲」除同一起点那一形（两臂相同）外 0；C 那一圈的双扣在改前副本 110 / 192 段、入库 4 段，那 4 段是挂载里那一串抬 F 走被抛弃根那一圈的 `isolate_abandoned`（已知没做，第十六块） |
| K2 第十四块判别力 | **打中（窄）**，两处 | ① 抬 F 那一串卸载那一支（`ImmediatelyBelowTheNewestValidRootOnEveryDevice`）两处改回 `reclaim_released_up_to`：9 个相关 harness 目标（含慢档）与 core 库单测全绿，探针在卸载之后看到 C 护着的槽空闲 72 / 108 段与 100 / 100 段（入库 0）；没造出红。② 访问器没有阳性对照：访问器恒交 0 时把 C 那一圈改回隔离分配位，默认档 23 条全绿，只有 `#[ignore]` 的 1536 槽慢档用例靠「写被拒」红 |
| K3-1 多给 | 没打中 | 两张假卡照卡池走法的随机模型 3000 种子 × 300 步、189 708 次借：借那一刻多给 0、重起被拒 0；同一模型换成第十五块之前的走法（只看列出来的卡）多给 63 次、最多 587 MiB、借到之后重起被拒 2554 次（阳性对照） |
| K3-2 放得下却拒 / 并发被压 | 没打中 | 拒而卡上真放得下的，全落在「丢了的设备分不出」两态、读不到实际占着的、有没等成借据三类（用户已认 / 文档写明只会少借）；`DeviceStarted`、读得到、没有没等成借据的一类今天 0（改前走法 6810） |
| K4 第十五块判别力 | **打中（窄）** | 卡池模块里加一个与外部 crate 同名的模块（`mod wgpu` 或 `mod std`）遮住方法体里的路径，方法体一个字不动：整句比照绿、`cargo check -p singlefs-checker-tier --features gpu --lib` 0 警告编过；用例旁「拦不住的写法」没列这一种。账层七种改法 harness 库单测全红 |
| K5-1 / K5-2 | 同 K3-1 / K3-2 | 加 K4 的 `mod std` 那一种：poll 恒交「队列空了」，列出来的没等成借据提前销账（多给，推的）；`mod wgpu` 那一种：借之前那一下 poll 改成一直等到设备空（并发被压，推的） |

## K1 第十四块 双扣改法

### 装置

- 探针 `src/opus_no_probe.rs`（起点：上一轮攻方 `opus_lm_probe.rs`）：在内存稀疏盘上录一段带前缀与 R 的历史（每段起点状态只建一次，之后每段从内存那一份拷），R 落盘、系统配置那一下丢掉之后按故障组让若干范围读回全零（`S3_PERSIST` 设了就挂上之后照旧读回全零、写过那一段才照读），可写挂载，之后走一串用户动作：`o` 覆盖写、`n` 建 inode、`s` / `S` 顺序写、`F` 抬 F 到准入上限、`b` 回退到上一版、`u` 正常卸载、`c` 进程退出不卸载（这两个下一个动作之前再挂）。释放与回收的时机、抬 F 次数、再挂都由动作串放开，只固定前缀与故障。
- 每一步之后：池级 checker 判 I-7.4 / I-4.8（写根之前那一刻与之后各一次）；分配器探针 `opus_no_probe` 数「隔离还护着（环里被抛弃根引用的、还没解开的 C）而三种挡位都没置」的槽（K1-1 / K1-3 少扣）与「分配位、隔离位同时置着」的槽并按归属拆（K1-3 重复扣）；`u` 之后另在卸载用过的分配器上数一次「护着而空闲」。挂载与抬 F 算影子账那一圈另加两个计数（`patch_probe.py`）：C 的落点按起点槽被现行账豁免、而跨度里有槽不在现行账的仍分配记录里（K1-2）；同一起点后来那个落点比先隔离的宽（K1-2）。
- 两臂：入库原样（today）与第十四块改前（pre14），同一份件表、同一个种子，逐段配对。另在 today 上加四条只看环境变量 `OPUS_NO_ARM` 的臂：`raise_plain`（抬 F 那一串三处都改回不补隔离）、`held_plain`（只改准入那一支）、`unmount_plain`（只改卸载那一支两处）、`first_slot_only`（C 那一圈按起点槽一槽的分配位判整段）。
- 取样（`jobs/`）：几何 384 / 1536 / 6144 槽；前缀 `o`×10、×20、×40、`S`×6、`oooon`、`S`×22 再 `b8`、`s`×8、×15、`S`×4、×8；R 取 `b1 b2 b3 b4 b5 b6 b7`（回退）、`n`、`s`；故障组 `rroot,rlin`、`rroot,racct,rlin`、`rroot,racct`、`racct`、`racct,rlin`、`rall`、`racct,oldroot`、`racct,oldacct`；动作串随机（字母表 `onsSFbuc`，长 12–30）与固定串（`F` 打头、全 `o`、全 `s`、全 `n`、`oF` 交替、`u` / `ou` / `Fu` …）。

### 结果（`runs-summary/` 原样）

`pair-s1.txt`（384 槽、故障挂上之后照旧）：

```
PAIR batch=s1 tree=today histories=192 skipped=8 mount_err=6 red=45 exposed=0 dbl_c_or_other=4 refused_any=114 c_exempt_partial=0 c_exempt_partial_unprotected_slots=0 dup_key_wider=0 ab_exempt_partial=0 probe_max=0/0/28/0/28/0
PAIR batch=s1 tree=pre14 histories=192 skipped=8 mount_err=6 red=45 exposed=0 dbl_c_or_other=110 refused_any=114 c_exempt_partial=0 c_exempt_partial_unprotected_slots=0 dup_key_wider=0 ab_exempt_partial=0 probe_max=0/0/242/0/242/0
PAIR batch=s1 paired=192 red_differs=0
```

`pair-s2.txt`（384 槽、挂上之后照读）、`pair-s3.txt`（1536 槽）、`pair-s4.txt`（6144 槽）各自的 `red_differs` 行：

```
PAIR batch=s2 paired=192 red_differs=0
PAIR batch=s3 paired=32 red_differs=0
PAIR batch=s4 paired=8 red_differs=0
```

`arms-m1-vs-today.txt`（四条臂对入库，同 s1 那 192 段）：`raise_plain` 与 `held_plain` 各有 2 段出现「护着而空闲」（入库 0），红数相同；`unmount_plain`、`first_slot_only` 与入库逐段相同。`unmount-branch.txt`（卸载之后那一刻）：

```
U1 arm=today histories=108 exposed_right_after_unmount=0 red=0
U1 arm=unmount_plain histories=108 exposed_right_after_unmount=72 red=0
U2 job=2 OPUS_NO_ARM=unmount_plain S3_SLOTS=384 histories=84 exposed_right_after_unmount=84 red=0
```

### 打中之后的四句（K1-1 旁证那一形，不算这一轮打中）

- 形：前缀 `o`×10，R 取 `b6`（或 `b1 b3 n`），故障 `rroot,racct`（或再加 `rlin`），挂上之后照旧读回全零，之后不卸载不再挂，只走 `F` 再覆盖写、或一直覆盖写、或一直顺序写（`pair-n1.txt`、`red-classes-n1-today.txt`：68 段里 16 段红，两臂逐段相同 `CMP paired=68 differs=0`）。一段的现场（`S3_WATCH_SLOT=1420`）：挂载那一刻盘 0 槽 1420 的那条记录「已释放、释放代 8」，在 defer 里，`in_c=false`；第 0 步抬 F 把它回收成空闲，第 6 步覆盖写把它发出去，R（实例 1、txg 14）还在环里，I-7.4 红。
- 机理（读码，推的）：R 的记录读得出且齐、R 的账读不出时，C 的账那一份照条款不交——「那时它的单元多半也验不过，交 F1 乙」（`crates/singlefs-core/src/mount.rs:1847`），代码那一支是「None => continue,」（`crates/singlefs-core/src/mount.rs:1910`）；R 的根槽又读不出，它不进「比所选那一版新、读得出的根」那一份，丙与 tri 都走不到；R 引用而 R 的记录没点名的单元（回退复活的、R 之前就有的数据单元）谁都不护。
- 四句：不分辨臂（第十四块改前改后同红，病根在 C 的取法，不在跳过与补隔离）；系统当时看得到（R 的记录齐，只是账读不出）；归到正文 K1-1「I-7.4 红」那一分句，但不是「回收路漏补」——回收路照「还护着的」补了，只是这几槽从来不在「还护着的」里；跑前没有给这一格的改法。是不是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 那一族，交主 agent 判。
- 同一批里先卸载或先崩溃再挂之后才红的（`red-classes-s1-today.txt` 45 段全是 `remount-before-red`）：再挂之后 R 的记录不再算水位之上没施加的，R 的根槽仍读回全零，与 C554 的题面同形，两臂相同。

### K1-2 与 K1-3 的细节

- 起点槽豁免「if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {」（`crates/singlefs-core/src/mount.rs:1652`）：计数 `c_exempt_partial` 在全部批里都是 0——取样里没有一个 C 的落点按起点被现行账豁免、而跨度里还有槽不在现行账的仍分配记录里。`first_slot_only` 臂与入库逐段相同：取样里也没出现「一部分槽分配位置着、一部分没置」的落点，按槽判与按起点一槽判分不出来（见「限度」）。
- 同一起点两个跨度（`dup_key_wider`，s3 8 段、s4 4 段，两臂相同）：1536 槽、前缀 `o`×40、R=`b3`、故障 `rall` 那一形里，C 在 R 那一槽同时记着 `1544+1` 与 `1544+2`（`S3_WATCH_SLOT=1544/1545` 的现场），只有先进来的窄的那一个隔离了，槽 1545 空闲而在 C 里；那条 `1544+2` 是现行账里一条早已回收、记录还留着的已释放记录（释放代 11），是 tri 并进来的陈旧的那一份，没有环里的根引用它，这几段都没红。
- 入库那 4 段双扣（`probe_max=0/0/28/0/28/0`，384 槽、前缀 `o`×20、R=`b1`/`b3`、故障 `racct,oldroot`）：槽 1450–1451、1671–1682，释放代 23 的 defer 槽。加打印看到先走 C 那一圈（分配位置着，跳过），再在挂载里那一串抬 F 时走被抛弃根那一圈，「allocator.isolate_abandoned(device, slot, span_slots);」（`crates/singlefs-core/src/mount.rs:1622`）把它们整段隔离（R 在那一遍按实例表判被抛弃，落点借的是 C 那一槽记着的）。这是正文「已知没做」里「被抛弃根那一圈与根槽读坏那一圈遇到 defer 槽也双扣」，不算打中。

## K2 第十四块判别力

### ① 卸载那一支改回不补隔离，没有用例红

- 改法：「ShadowLedger::On => allocator」（`crates/singlefs-core/src/mount.rs:4729`）那一处在 `reuse == ReclaimedReuse::Immediately` 时改走 `reclaim_released_up_to`。`Immediately` 只出现在「ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice => {」（`crates/singlefs-core/src/mount.rs:4506`）那一支里那两处，即卸载那一格。
- 用例：9 个相关 harness 目标带 `--include-ignored` 与 core 库单测（`k2-tests-under-arms.txt`）：

```
K2U arm=unmount_plain raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table rc=0 test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.41s
K2U arm=unmount_plain reuse_after_raising_the_floor rc=0 test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.34s
K2U arm=unmount_plain singlefs_core-lib test result: ok. 215 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

  另 7 个目标同样全绿（`model_answers_unmount_with_the_floor_at_the_current_version` 5、`unreadable_abandoned_root_slot` 3、`shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring` 2 …，原样在文件里）。变异表追加的 10 行与整行替换 2 行里没有这一支。
- 探针：同一批历史，卸载用过的分配器上 C 护着而空闲的槽，入库 0 段、这条臂 72 / 108 段（384 槽）与 100 / 100 段（384、1536 槽，`u2`）。卸载那一串之后进程就退了，这几槽要在那一串自己的空发布里被取走才会红；取样里没取走，红 0。
- 四句：分辨臂（同一段历史上入库 0、这条臂有，量过）；系统看得到（根环表里 C 那一份在）；归正文 K2「能不能拦住改回去」——拦不住；跑前没给改法。我提的改法：加一条 harness 档用例，形照 `jobs-u1.txt`（384 槽、前缀 `o`×10、R=`b6`、`rroot,rlin` 挂上之后照旧、挂上就卸载），断言卸载之后分配器里 C 护着的槽一个都不空闲；并补一行变异。只在我的模型上量过、被攻过零轮；这条用例在入库代码上绿、在这条臂上红是推的（探针的读数是量的，用例没写）。

### ② 访问器没有阳性对照

- 入库对访问器「pub fn slots_counted_both_as_allocated_and_as_isolated(&self) -> u64 {」（`crates/singlefs-core/src/allocator.rs:625`）的用法全是 `assert_eq!(…, 0, …)`（测试文件里转一层的「fn slots_counted_both_as_allocated_and_as_isolated(allocator: &PoolAllocator) -> u64 {」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2379`），六处调用都比 0），没有一处断言它在已知双扣的状态上交正数。
- 量的（`k2-tests-under-arms.txt`，同一个测试二进制带 `--include-ignored`，臂只看环境变量）：

```
K2A arm=isoall rc=101 test result: FAILED. 21 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.83s failed:revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices 
K2A arm=acczero rc=0 test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.97s failed:
K2A arm=isoall+acczero rc=101 test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.56s failed:wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices
```

  `isoall` 即变异表第三行（C 那一圈隔离时不看分配位）；`acczero` 是访问器恒交 0。两样叠在一起，384 槽那条与复活那条照绿，只剩 1536 槽慢档用例「#[ignore = "harness 慢档用例：1536 槽两块盘、40 步每步跑池级 checker，debug 下单条过 60 秒"]」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2717`）靠「写被拒」红。
- 四句：分辨（`isoall` 单独 3 红、叠上 `acczero` 1 红，量过）；看得到；归正文 K2「访问器本身会不会算错让用例照绿」——会，默认档照绿；改法：core 库加一条访问器的单测，分配一个槽再 `isolate_abandoned` 它，断言访问器交跨度（推的，没写没跑；被攻过零轮）。

## K3 / K5 第十五块 借之前那一步看每张卡

- 装置（`src/gpu_model.rs`，插进副本 `gpu_memory_ledger.rs` 的测试模块，不要卡）：两张假卡、每张一格 `DeviceStartedOnDemand`，照卡池的走法——借之前那一步（「for card_position in 0..card_count {」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:1457`），四样照卡池接：留着的设备、丢没丢、poll 随机交回空了、账上的那一台）→ `book_on_the_first_card_with_room` → 留着的没有或丢了就 `started_or_start` 里先 `prepare_the_driver_share_before_starting_the_device` 再记起好 → 借据拿着设备的克隆；另有建满、等成或没等成再还、在用的设备随机丢、别的进程只拿此刻没许出去的。真值按每台设备没放掉时的驱动、建的、没等成留下的算；借到那一刻核「借的 ≤ 卡上真能再给的」（空闲减驱动还要长的、减活着没建的），两张都借不到时核有没有一张真放得下，按那张卡的账走到哪一步分。另一条臂把借之前那一步换回第十五块之前的走法（只看列出来的卡）。
- 结果（`gpu-two-card-model.txt`，3000 种子 × 300 步）：入库 `worst_over_mebibytes=0`、没有一条 `over@`、没有一条 `restart_refused_after_booking@`、没有 `physically_overfull`；改前走法：

```
OPUS_NO_GPU arm=listed_only seeds=3000 steps=300 worst_over_mebibytes=587 {"bookings": 192115, "over@DeviceStarted/lostkept=0/failedwait=0/listed=true": 63, "refusals": 168121, "restart_refused_after_booking@DeviceStarted/lostkept=0/failedwait=0/listed=true": 2554, "under@card0:DeviceLostNotStartedAgain/lostkept=1/failedwait=0/listed=false": 1479, "under@card0:DeviceLostNotStartedAgain/lostkept=1/failedwait=0/listed=true": 12422, "under@card0:DeviceLostNotStartedAgain/lostkept=2/failedwait=0/listed=false": 534, "under@card0:DeviceLostNotStartedAgain/lostkept=2/failedwait=0/listed=true": 1040, "under@card0:DeviceStarted/lostkept=0/failedwait=0/listed=false": 10424, "under@card0:DeviceStarted/lostkept=0/failedwait=0/listed=true": 3510, "under@card0:DeviceStarted/lostkept=0/failedwait=1/listed=false": 863, "under@card0:DeviceStarted/lostkept=0/failedwait=1/listed=true": 922, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=0/listed=false": 6652, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=0/listed=true": 17207, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=1/listed=false": 193, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=1/listed=true": 608, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=0/listed=false": 1533, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=0/listed=true": 3578, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=1/listed=false": 24, "under@card0:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=1/listed=true": 98, "under@card1:DeviceLostNotStartedAgain/lostkept=1/failedwait=0/listed=false": 647, "under@card1:DeviceLostNotStartedAgain/lostkept=1/failedwait=0/listed=true": 6330, "under@card1:DeviceLostNotStartedAgain/lostkept=2/failedwait=0/listed=false": 199, "under@card1:DeviceLostNotStartedAgain/lostkept=2/failedwait=0/listed=true": 530, "under@card1:DeviceStarted/lostkept=0/failedwait=0/listed=false": 6042, "under@card1:DeviceStarted/lostkept=0/failedwait=0/listed=true": 3300, "under@card1:DeviceStarted/lostkept=0/failedwait=1/listed=false": 381, "under@card1:DeviceStarted/lostkept=0/failedwait=1/listed=true": 717, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=0/listed=false": 3083, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=0/listed=true": 9262, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=1/listed=false": 69, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=1/failedwait=1/listed=true": 230, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=0/listed=false": 421, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=0/listed=true": 1201, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=1/listed=false": 6, "under@card1:DeviceStartedAgainWhileTheLostDeviceRemains/lostkept=2/failedwait=1/listed=true": 31}
```

  入库那一行同在 `gpu-two-card-model.txt` 里。入库的少给全落在 `DeviceLostNotStartedAgain`、`DeviceStartedAgainWhileTheLostDeviceRemains`（「丢了的设备分不出」，「| DriverShareOnTheCard::DeviceStartedAgainWhileTheLostDeviceRemains => return 0,」（`crates/singlefs-harness/src/gpu_memory_ledger.rs:217`），用户已认的少借）、`listed=false`（读不到实际占着的）与 `failedwait=1`（有没等成借据）三类；`DeviceStarted/lostkept=0/failedwait=0/listed=true` 这一类入库 0、改前走法两张卡合计 6810——上一轮判决要先查成因的「`DeviceStarted` 那 1569 次」在这个模型里就是它，第十五块之后没了。
- 并发（`src/gpu_stress.rs`，两张假卡、8 线程 × 5000 轮，借之前那一步看两张卡、借第一张放得下的、丢了就重起、随机让设备丢）：`gpu-two-card-stress.txt` 原样

```
OPUS_NO_STRESS threads=8 rounds=5000 bookings=40000 restarts=2039 restart_refused=0 wrongly_lost=0 panics=0 lost_and_still_in_use_after_one_more_pass=0
```

  `panics=0` 即「取它的那张卡」那条断言一次没响；没丢的设备从没被记成丢了。
- 读码看到、没造成反例的一处窗口（推的）：借之前那一步看完、借在锁里记上之前设备才丢，那一笔照「没丢」的账借；之后重起照此刻再判，放不下报本次未跑。与第十五块之前同一类，窗口从「列卡到借」那一段缩成「看到借」那一段；没量窗口多长。

## K4 第十五块判别力

- 账层（harness 库单测，`k4-ledger-mutants.txt`）：借之前那一步七种改法——只看列出来的卡、少看最后一张、只看第一张、丢了的只在列出来的卡上告诉账、遇到丢了就 `break`、丢了照样 poll、没列出来的也 poll——每种都有用例红；`break_after_lost` 只被断言那一条（`a_kept_device_on_the_books_of_another_card_is_refused_by_an_assertion`）顺带拦下，没有一条行为用例在「前一张丢了、后一张要销」的盘面上。
- 卡池接线保留扫源码的那一条：取方法体到第一处缩进十二格的右括号「.find("\n            \u{7d}\n")」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2434`），去掉 `//` 开头的行与空白，整句比。方法体里的路径 `std::panic::catch_unwind`、`wgpu::PollType::Poll`「started.device().poll(wgpu::PollType::Poll)」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1346`）按方法所在模块解析；在「mod card_pool {」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:1170`）下面加一个与外部 crate 同名的模块就把它们换掉，方法体一个字不动（`src/k4_shadow.py`）：
  - `mod wgpu { pub use ::wgpu::*; pub mod PollType { pub const Poll: ::wgpu::PollType = ::wgpu::PollType::wait_indefinitely(); } }`：借之前那一下 poll 改成一直等到设备空（K5-2 并发被压，推的）；
  - `mod std { pub use ::std::*; pub mod panic { pub use ::std::panic::*; pub fn catch_unwind(…) -> … { Ok(Ok(::wgpu::PollStatus::QueueEmpty)) } } }`：不 poll、恒交队列空了，列出来的没等成借据一律销账（K5-1 多给，推的）。
- 量的（`k4-shadow.txt`）：两种各 `cargo check --offline -p singlefs-checker-tier --features gpu --lib` rc=0、0 警告；照搬那条用例取法与比法的 `src/scan_port.py` 判两种都绿；阴性对照：遮蔽模块里写一个不存在的变体 `cargo check` 拒（`error[E0599]`），方法体最前面加 `if true { return; }` 扫源码判红。用例旁「拦不住的写法」（「改 `GpuCard::device_condition`、`device_on_the_books` 放进去的是哪一台」（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:2413`） 起那一段）没列模块级同名遮蔽。checker 档测试本身没跑（重型），用例判绿是照搬取法推的。
- 四句：分辨（改了行为、用例取法判绿，量过编得过与取法；行为改变是推的）；系统看得到（遮蔽在同一份源文件里，扫源码可以扫）；归正文 K4「拦不住的写法写全了没有」——没写全；改法二选一：用例旁补这一种并写明拦不住，或整句比之外再断言 `card_pool` 模块里没有 `mod wgpu` / `mod std` / `mod GpuCard` 这类与路径首段同名的条目（推的，没写没跑，被攻过零轮）。同一招对另四层叫停与 `the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first` 那几条保留扫源码的用例同样成立（推的，没逐条试）。

## 几个改法各修哪一格

| 改法 | K2 ① 卸载那一支 | K2 ② 访问器 | K4 遮蔽 | K1-1 旁证那一形 |
|---|---|---|---|---|
| 卸载那一支补用例（`jobs-u1.txt` 那一形） | 修（推的：探针读数量过，用例没写） | 不碰 | 不碰 | 不碰（推的：两臂同红，病根不在回收路） |
| 访问器阳性单测 | 不碰 | 修（推的） | 不碰 | 不碰 |
| 扫源码补「无同名模块」断言或写明拦不住 | 不碰 | 不碰 | 修（推的） | 不碰 |
| C 的账那一份在「记录齐、账读不出」时不再 `continue`（改从映射现算或拒） | 不碰 | 不碰 | 不碰 | 推的：R 的映射根同在 `racct` 故障里，现算多半也不成，结局是拒可写挂载而不是护住；没实现没跑 |

## 没打中的形状

- K1-1：回收路。读码：清分配位只有 `mark_reclaimed`，调它的只有 `reclaim_released_records_up_to`；它的调用者是挂载那一刻（在算影子账之前，回收出来的照隔离）、记根（补）、抬 F 的准入支与卸载支（补）、复用窗口置 0（已知没做）「ReuseWindow::ForcedToZero => {」（`crates/singlefs-core/src/allocator.rs:1842`）、影子账关着的对照。造：上面 2168 段，动作串放开释放（`o s S b`）、回收（`F`、写根转环）、抬 F 次数、卸载再挂与崩溃再挂；几何 384 / 1536 / 6144；故障组 8 种，挂上之后读回全零与照读两种。入库对改前逐段红数相同，入库「护着而空闲」只在同一起点那一形出现。`raise_plain` / `held_plain` 两条臂在同一批里各 2 段出现「护着而空闲」，说明探针看得见回收路漏补。
- K1-2：起点槽豁免跨度更宽、一部分槽分配位置着的落点，在取样里都没出现（计数 0、`first_slot_only` 臂逐段相同）。
- K1-3：入库 C 那一圈双扣 0（4 段的双扣来自被抛弃根那一圈，已知）；改前 110 / 192 段。对照的条款是第九项那一句「只除去现行账里仍分配着的槽（那些算在「已分配」里，不在这里再算一次；这次挂载里释放、回收时分配器照补隔离）」（`.claude/kb/decisions/28-挂载期承诺量.md:27`）；红法对照 I-7.4 那一行「近 K 代块未被复用」（`.claude/kb/invariants.md:58`）。管理员回退在会话里复活一个已回收而隔离着的槽（`resurrect_released_record` 走 `mark_allocated`）会不会双扣：动作串里的 `b` 覆盖到了，没出现；没专门造。
- K3-1：两线程同借一张卡、重起正在进行时取留着的设备、丢了的旧设备还被借据拿着——都在并发压测与随机模型里，多给 0。
- K3-2：每次借都看每张卡，只多一次 `started_now` 克隆与 `is_lost`，没列出来的不 poll（`poll_all` 臂有用例红）；没量到被压住。

## 这条腿自己的限度

- 崩溃状态：派发提示「重型测试：不跑」写明崩溃枚举不跑，这条腿没调 `enumerate_layer0` 一族，每段历史只取一个崩溃点（R 的根落盘、系统配置那一下丢掉），没算 `closed_form_state_count`，枚举的状态数 0。打中的几格都不靠崩溃点。
- 故障只有「读回全零」一种，按写过那一段就照读解开；没有读错、没有半写。
- 取样之外：每批先跑一小段估时，一批至多 8 件、估在 4 分钟以内（前台命令上限）；1536 槽一段约 16 秒、384 槽约 1–5 秒、6144 槽 6–8 段一批 13 秒。6144 只跑了 16 段；1536 只跑了 64 段加 `u2` 16 段；`S`×22 再 `b8` 那一形只跑了 4 段。
- K2 ① 没造出红：卸载那一串的空发布没取到那几个空闲槽；换几何、换开放段的满度可能造得出，没试。
- GPU：假卡，驱动实占取 600、留给驱动 2560（照库里测试的取法）；没起真卡，没跑 checker 档的卡池用例，K4 的「行为改变」是推的。
- 副本上的数都不进 kb：`copy` 加了探针与臂（只看环境变量，不设就是入库原样），`pre14` 是倒推改动全文得到的，不是取自版本库。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-no-code-r1-opus/`：`copy/`、`pre14/`、`k4copy/`、`verify/`（四份仓副本）与 `target/`、`target-pre14/`、`target-k4/`（三个编译目录），`runs/`（每件日志）、`summary/`。四份仓副本与三个编译目录交回前已删（`ls` 现查过）；`runs/`（每件日志）、`summary/`、`model/`（模型目录的原件）留着给主 agent 核。

## 没做什么

- 没跑 checker 档测试、门禁阶段、崩溃枚举、GPU 用例；`cargo check -p singlefs-checker-tier --features gpu --lib` 只编不跑。
- 没判正推那几格；没替主 agent 采纳；K1-1 旁证那一形是不是 C554 一族没下结论。
- 没写改法的代码，表里「修」都是推的。
- harness 档 179 个测试目标只在 `unmount_plain` 臂下跑了 9 个相关的（加 core 库单测），没跑全。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-batch10z-no-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-no-code-r1/tree --root <仓根>` 末行：

```
  ✓ 核了 17 处引文，对上 17 处，没判 0 处（逐处列在上面）
```
