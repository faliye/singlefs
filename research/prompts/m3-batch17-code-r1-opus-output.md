# m3-batch17-code-r1 攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 K2（重点 N1 甲 m 取不到那一支；第 16 批「没打中（一次抽样）」各格的第二次抽样；N6）。被判的是冻结副本 `/tmp/claude-1000/m3-batch17-code-r1/tree/`（四份被判文件的 sha256 与快照 `research/prompts/m3-batch17-code-r1-snapshot/crates-sha256.txt` 逐行相同，开工时现核）。副本上量出的数都是副本上的数，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-batch17-code-r1-opus-model/`。在一个空的工作目录 W 里逐阶段跑（每阶段前台 4 分钟以内；本机满载时个别阶段到 3 分半）：

```
cd research/prompts/m3-batch17-code-r1-opus-model
bash rerun.sh list W            # 列出阶段，按次序逐个 bash rerun.sh <阶段> W
bash rerun.sh compare W         # 重算判定行，与 out/summary.txt 逐行比，一致才退 0
```

冻结树路径认 `FROZEN_TREE`（默认 `/tmp/claude-1000/m3-batch17-code-r1/tree`）。交回之前在空目录 `/tmp/claude-1000/m3-batch17-code-r1-opus/R/` 里从 build-jia 起逐阶段跑完，`compare` 的原样输出：

```
compare: 判定行与 out/summary.txt 逐行相同（230 行）
```

这一遍跑的过程中改过 rerun.sh 四处（diag、rr-cell 两行补 `env`；加 redo-fxw / redo-cell 两个补跑阶段；qf 阶段先建目录；加 narrower 阶段），改的都是改之前还没跑到的阶段，已跑的阶段命令一字未动；build-ga 第一次退 252（内存包装排不上，slice 已占 39.9 GiB），重跑做成。层 0 那八段与草稿目录 `W/` 里先跑的一遍逐数相同（第八节）。

每个文件的 sha256（`find . -type f | sort | xargs sha256sum` 原样，交回前现取）：

```
04739ade2299d2c3b2615abf30bf2bb0bc7a46e47dc3c481727b5403ea3cfacb  out/diag.txt
66af3e032af8b11db610a6807b3cf6d23000844def9d8999917b612711634fb3  out/l0-jia-192k9-n8-allbut1-z.txt
6dea3a1ca3c53029a91e2a31ecd49c1f76d544385db8a400cafaa835ac99b4c1  out/l0-jia-192k9-n8-but1all-z.txt
e631454e342d5897b147fca5103b8d9c0480a952ba543fba3d2912fa4833ab8e  out/l0-jia-192-n17-rebuildpass-e.txt
02444869b818dbe10ea5e884c42855a968d69f350efe50cc2f9d8aed5fe1df2b  out/l0-jia-248-n17-clean.txt
d64f964cce62cd79672886db6c570fddabd969306a229af219fe81c204d30e09  out/l0-jia-264-n15-but1all-z.txt
c79874233724f9ea36423b4eedcaf07a5b38245c89ac70e510d05323ed7268c3  out/l0-jia-264-n15-rebuildpass-e.txt
66f374e335e8ff4bda3348489e66c20dc3af3bb5bc205c702cfbe29752de1847  out/l0-jia-p256-n10-clean.txt
852e6f30c298b08b64cc2eefe8e25f585fde44fbe17fdaceff9bce07840394c5  out/l0-zs-264-n15-but1all-z.txt
e61ea34153bd49a36c0ce65fe42f445ab470668428ff356e3353775d736bc135  out/qf-qfj.txt
93f0087e497cbcaf6148730cb856e2ef883a806593a798242ae23e9fc39fd596  out/qf-qfm.txt
46924ce7f606653517e93c496df5eaaf2ae3ce72770e11e7b2e95c75ecea5931  out/qf-qfr.txt
3f435e19cf62fa0a49b91ba76b6b3932f438cc755a2b140e44cfc6a5e29eefef  out/qf-qfu.txt
0959ce548394c219aa6fde3ad6309460eb6ba380a41b857e1bd1f1faf392a0a2  out/rr-rebuildpass.txt
bce46a1881fa5158608efbb1e4cbbe75d96d5f4c7993be9cfd6ba938bfe5db2b  out/summary.txt
3548924c690c62eaea1463cae8fd9ededb74ec11e4383a411a93fd3a05053d24  patches/diag.patch
5228a09bbd2d4c5529674b0d58fe7799441b85b96616fc6a83d145d7ca42fc1a  patches/ga.patch
519b9ae27ab42ce10fbcf1d78e3eba7c1c59c9a543112f256cdfcebacbb53fae  patches/nf.patch
98da819ec0b4d5e72b9103a744d4b027a3ac945731ab4c8326569878411c7bda  patches/qfm.patch
de7bbc5cd0564b734b3c65b3bb1a7def09142db3650341d7b24283d8ac798ba2  patches/qfr.patch
cbce496bd1b63123b0eaf12d60d0a0f26ebfe51779420b96e39d4047511317ed  patches/qfu.patch
0baad09c03246e5ed678646a095b013f0efc231103cf3cbf48853236ade64a53  patches/rr.patch
84f72356fa543fe10ecbfed336f7b3bae78511405535d4b156f69642b44aac5d  patches/workspace-member.patch
c96c14aae11d97e3ea353b88514f95cfdb6566fd1b0b9e3b90bea89a76628619  patches/zs.patch
02278be0ee370f1f095bef82b74e1f04c50eef20ab4c091335e099bf65109084  probe/Cargo.toml
2235cb3af4f2bf50f56880932766a461b91767d1f02883ffb8b5e57c46c4be9e  probe/main.rs
4d19b7444f4d292a2c8836a21fbdb96b25f84c90b0379d10bb9c1e75f5c9dc7f  probe/qf_attack_probe.rs
4afe1e7b9e535399c612fb71826381e30fe88894a6cd2570d60f36ead2cec168  rerun.sh
bc61dda22ce30ebb119f4c5fd45497b6f18ade92df53478db32f1ccf37021ba9  scripts/arm.sh
3d4edd467a4e0b8d4b0ecd75039a23dc48c957a1fb1bddddb3602fce0f50f3c7  scripts/batch.sh
8e852c568111d03c2f5020afd88e272f9093c0101a023a56714e5283ba1148f9  scripts/build.sh
1bb883cd19cdb3a3e53f7d4cbd64859918ba57a790013f5ddaf49aa85b4fbf23  scripts/complete.sh
d1b6f05abf720e9cc80bcd8a4ffcabc30b0c070516909b224180542a1d644f01  scripts/fxsum2.sh
fbca7ad8e41c826abd92b473fb9feb69aa3bdc00f143b17dbb847582958dfb56  scripts/qf.sh
43705aa906f7d3c5ecb50f7d9191aa2cd7d7f2f95b485a68b648a6772802fcb9  scripts/r2sum.sh
5eb5c270486673d201db4a069ba3533df90c339af219abd7a706f5e0dd103d66  scripts/run.sh
4d6acb15873392c7ff4933bad07889937ad0c364c2f56fb0b44f61a04c47d83f  scripts/summary.sh
```

几条臂（都只改冻结副本的拷贝，补丁在 `patches/`）：

- **jia**：冻结副本原样（第 17 批甲）。
- **ga**：m 取不到那一支照原门槛整截当场放开（`None => Some(floor_of_the_reclaim)`，与变异表那一行同一刀），只作对照。
- **rr**（我提的，零轮）：重建分配器那一遍根环一条有效根都没读出（`window_anchor` 取不到）时，任何写之前拒这次可写挂载。
- **zs**（我提的，零轮）：重建分配器算回收门槛时，读得出而自证不过的槽与读不出的槽一样按它在窗口里能住的最小 txg 并进。
- **nf**：`newest_valid_root_txg_on_every_device` 不滤被抛弃的根，只作对照。
- **diag**：只加两行 `eprintln!`（重建时的门槛、进程内每记一条根时的门槛与回收了哪些槽），看机理用。
- **qfm / qfu / qfr**：N6 的三种错实现（第三节）。

## 各格判定一览

「坏」= 一个崩溃状态丢一块盘之后，剩下那块盘只读挂载、整读每个活文件读不出，或读出的不是任何一版合法内容（卸载之前、再挂之后各版的两盘整读哈希）；卸载之前剩下那块盘自己就读不出的不记坏。下面「历史」= 起点 × 卸载刀 × 读坏计划，每段历史的再挂写流按前缀刀判（筛查），挑出的八段另跑层 0 枚举域。

| 格 | 判定 | 要点（数出自 `out/summary.txt` 与 `out/l0-*.txt`） |
|---|---|---|
| N1 甲：再挂期间根环读坏那一族放开扫 | **打中（甲一）** | m 取不到、而另一块盘上除最新那条根之外的槽这次挂载都读回全 0（自证不过）：窗口里最旧有效根被抬到最新那条根，当场放开到它，与第 16 批那一刀同一个结局。窄盘 20 个起点 813 段历史里两种选槽法各 98、53 段坏；层 0 两段 20511 个状态各坏 14339、8195；zs 臂同一段 0 坏。读错（报读错而不是全 0）的同一批计划 0 坏 |
| N1 甲：m 与窗口里最旧有效根都取不到 | **造出来了，而且打中（甲二）** | 只让重建分配器那一遍根环读坏两块盘（择根那几遍读得出）：当场一个都不放，但进程内根环表装的是空表，写行那次记根时回收门槛跳到写行的 txg，释放代高于 F_生效 的一截当场放开，暖机写进窗口里有效根引用的槽。窄盘 30 个起点 1134 段里 305 段坏、599 段在写行与暖机落盘之后报 `FloorRaiseFailedAfterTheMountsPublishes`、33 段任何写之前被拒；层 0 两段坏 3075 / 20511、7182 / 24618，两盘都在时池级 checker I-7.4 判红 4107、8214 个状态；rr 臂全部在任何写之前被拒、0 坏 |
| N1 甲：窄盘上扣住让再挂被拒 | **有，按盘宽数得出** | 一块盘根环整段读坏（m 取不到）：单元区 192 槽 132 段里 99 段、216 槽 166 段里 66 段在任何写之前被拒，248 槽起 0；140–180 槽 792 段里 17 段。ga 同批 0 被拒、但有坏（第 16 批那一形） |
| 第二次抽样：形 R 不注读错 | **没打中，分辨力弱** | 窄盘 20 个起点 813 段、普通盘 6 个起点 253 段 0 坏 0 被拒；层 0 两段 0 坏。ga 在这批新点上不注错也 0 坏，这批点不分辨甲与 ga |
| 第二次抽样：扣住那一截放开 | **没打中** | 普通盘（含 4 GiB）6 个起点再挂之后覆盖写 1、3 次，298 段历史 0 坏；同一次挂载里准入抬 F 的写流里，坏只出在甲一那两种计划上 |
| 第二次抽样：窄得多的盘上被拒 | **不注错时 0 格** | 140–180 槽 24 个起点 792 段、192–320 槽 813 段，不注错 0 被拒；注错的数见第三行 |
| 第二次抽样：连续两次再挂 | **没打中** | 13 个起点（含第一次再挂一块盘根环整段读坏的 5 个）× 4 个卸载刀，239546 个两次再挂之后的判定 0 坏 |
| 第二次抽样：被抛弃根的过滤 | **没打中** | 新造法（被抛弃之后、暖机落满之前再崩）383 段再挂，jia 与 nf 写流逐条相同 |
| 第二次抽样：单盘恢复择中被抛弃的根（换造法） | **择中了；其中一部分读不出，看着是 C554 那一族（交主 agent 判）** | 被抛弃那次挂载在暖机落满之前崩、丢另一块盘：464 个判定择中被抛弃的根，其中 99 个读不出（`MappingStillUnreadable`）；与甲无关（jia、nf 逐数相同） |
| N6：有没有错实现让四条 Q-F 用例照样过 | **有，三种** | 会话里时间只进不退、时间按 32 位存、时间取整到分钟：Q-F 那一个测试目标 18 条全过（四条 Q-F 用例在内）；我加的三条判别形各抓到 |

## 一、N1 甲：再挂期间根环读坏那一族放开扫

### 1.1 装置与取样

原型 `probe/main.rs` 的 MODE=fx（底子是第 16 批攻方原型原样，文件末尾接 fx 与 ab2 两段）。一段历史 = 起点 → 在起点的会话上录一次卸载 → 卸载写流的前缀刀 n（只取系统配置已写、根没落满每块盘的那一截，同第 16 批）→ 两块盘都在、按一份「读坏计划」再可写挂载 → 那条再挂写流的每个前缀刀上丢盘 0、丢盘 1 各判一遍。读坏计划是用户与盘决定的那几步，放开扫：

| 维 | 取值 |
|---|---|
| 哪块盘、哪几槽 | 每块盘各取一种：不坏 / 全部槽 / 除这块盘上最新那条根所在槽之外全部（`but1`）/ 只最新那条根所在槽（`only1`），两块盘的组合 15 种 |
| 读坏的样子 | 读回全 0（自证不过，`z`）/ 报读错（读不出，`e`） |
| 持续多久 | 整次挂载（`w`）；挂载交回之后仍坏（`:p`）；只坏在挂载里第 a 遍到第 b 遍根环读之间（`r<起>-<止>`，按这一刀不注错那一次挂载的整遍读切开，任意一对边界）；第 5 遍（`P5`，就是重建分配器那一遍） |
| 写到哪一槽 | 那一槽从此不再坏（同第 16 批 Hiding） |

一次不注错的再挂里根环整遍读 15 遍；用回溯栈认过每一遍是谁读的（草稿，命令见第八节）：第 0 遍择根、第 1 遍新实例第一次发布的 txg、第 2 遍树 ID 水位、第 3 遍最新那条根的实例表（择根）、第 4 遍 F_生效、**第 5 遍重建分配器**（`rebuilt_allocator` 那一遍 `read_root_ring_slot`）、第 6–7 遍取号、第 8–13 遍准入那几次 F_生效 重读、第 14 遍抬 F 上限。

取样点与第 16 批不同（那一份窄盘 136–328、覆盖写 0/1/3/6、单元 2/4/8、批 0/2/3）：

| 族 | 这一次 |
|---|---|
| 窄盘（整次挂载那 34 份计划） | 单元区 192、216、248、280、320 槽 × 覆盖写 4、9 × 单元 3、10 × 批 1，20 个起点 |
| 普通盘（同上） | 384、256、240 槽，`384:3:1:2`、`384:5:3:2`、`256:3:1:2`、`256:5:3:2`、`240:3:1:2`、`240:5:1:2`（盘宽:覆盖写:崩了再挂:每次之后覆盖写） |
| 只坏重建那一遍（P5） | 窄盘 192–320 槽 × 覆盖写 4、7 × 单元 3、5、10 × 批 1，30 个起点；普通盘同上 6 个 |
| 任意两遍边界之间（两盘全部槽） | 窄盘 `192:4:3:1` 等 7 个、普通盘 2 个起点 |
| 更窄的盘 | 单元区 140、148、156、164、172、180 槽 × 覆盖写 1、2 × 单元 2、3 × 批 1，24 个起点 |

### 1.2 打中甲一：m 取不到、窗口里最旧有效根被抬到最新那条根

**一段具体的历史**（窄盘 `264:6:4:0`，卸载刀 n = 15，计划 `w:but1/all:z`；层 0 那一段 `out/l0-jia-264-n15-but1all-z.txt`）：

1. 起点同第 16 批那一形：不补环窗口、准入关着的窄盘（单元区 264 槽），覆盖写 6 次、顺序写长到 4 个单元，发 unlink 意图就崩，重开可写续做到空。卸载之前现行那一版 txg 20，盘 1 上最新的根 txg 19（`FXSTART … current_txg=20 … baseline={0: (Some(19), …), 1: (Some(20), …)}`）。
2. 卸载那一串落下 15 个写就崩：系统配置带了新 F = 20，带新 F 的根一条都没落。
3. 两块盘都在、再可写挂载；这次挂载期间盘 1 的根环每一槽、盘 0 除最新那条根（txg 20）所在槽之外的每一槽都读回全 0，挂载交回之后读得出。条款把挂载那一刻自证不过的槽当没有根：「判回退候选与算 F_生效 时当没有根」（`.claude/kb/decisions/16-发布语义.md:37`）；算回收门槛的那一句只把读不出的槽并进来：「读不出的槽在 txg > N − 3S（一圈，3S 是环长）里只对应一两个 txg：门槛按最小的算，根环表记最大的」（`.claude/kb/decisions/16-发布语义.md:34`），代码同：「/// 只管设备报读错的槽；读得出而自证不过（没写过、内容坏）的槽照旧当没有根。」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1683`）。挂载做成（`ok chosen=20 effective=21 floor_in_row=20 row_txg=22 warm_up=1`）。
4. 重建分配器时盘 1 一条有效根都没读出，m 取不到，走「None => oldest_valid_root_in_the_window,」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1884`）。这一遍读得出的有效根只剩盘 0 上 txg 20 那一条，窗口里最旧有效根就是 20，当场放开到释放代 20。这一步照条款写：「某块盘一条有效根都读不出时只到环里最旧有效根（挂载时读不出的槽按它能住的最小 txg 并进这个最小值，与回收门槛同一个读数）」（`.claude/kb/decisions/16-发布语义.md:41`）。
5. 写行那次的单元写进释放代 20 的槽，其中有盘 1 上 txg 19 那条根还引用的映射树节点；暖机的根落到盘 1 之前再崩，丢盘 0，盘 1 择到 txg 19，只读挂载读不出：

```
FXENUMBAD narrow U=264 k=6 u=4 b=0 cont=WalkedToEmpty n=15 plan=w:but1/all:z j=0 lost=dev0 chosen=Some(19) surviving_writes_persisted=2 states_with_this_survivor=128 BAD_err[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1412) }))]
```

层 0 枚举域（再挂写流按屏障与 FUA 切段、段内整写子集逐个枚举，每个状态池级 checker 照跑）：

| 历史 | 臂 | 状态数 | 丢盘 0 读不出 | 池级 checker |
|---|---|---|---|---|
| `264:6:4:0` n = 15 `w:but1/all:z` | jia | 20511 | 14339 | `{}` |
| 同上 | zs | 20511 | 0 | `{}` |
| `192:9:3:1` n = 8 `w:but1/all:z` | jia | 20511 | 8195 | `{}` |

（`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:204`：「bad_judgements=14339」；`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:221`：「bad_judgements=0」；`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:185`：「bad_judgements=8195」；层 0 自带的版本 oracle 在窄盘族照旧报 `violations` = 状态数，第一个文件已删，不分辨，我不拿它判。）

筛查（每段历史的前缀刀）原样：

```
[fxw-narrow] plan=w:but1/all:z histories=813 refused_before_any_write=0 refused_after_writes=0 histories_with_bad=98 bad_prefix_judgements=1761 starts_with_bad=5
[fxw-narrow] plan=w:all/but1:z histories=813 refused_before_any_write=0 refused_after_writes=0 histories_with_bad=53 bad_prefix_judgements=852 starts_with_bad=4
[fxw-narrow] plan=w:but1/all:e histories=813 refused_before_any_write=165 refused_after_writes=0 histories_with_bad=0 bad_prefix_judgements=0 starts_with_bad=0
```

报读错的同一种选槽法 0 坏：读不出的槽按它在窗口里能住的最小 txg 并进门槛，门槛压回到盘 1 的根之下。普通盘 6 个起点上这两种计划都 0 坏（「[fxw-plain] plan=w:but1/all:z histories=253」（`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:53`））。

**四句**：

1. 分不分辨臂：分辨。分辨的是「m 取不到时，自证不过的槽进不进窗口里最旧有效根」：jia 与 ga 都中（`gacmp` 那 7 个起点上，ga 的 `w:all/but1:z` 83 段坏、`w:but1/all:z` 32 段坏；jia 同一批 51、32 段），zs 不中（层 0 同一段 0 坏）；rr 不碰这一格（这一遍读得出盘 0 的根，`window_anchor` 取得到）。对第 16 批岔路单的三个候选：甲、乙（不改代码）都中，丙（拒这次可写挂载）不中，推的。
2. 系统当时看不看得到判别它的东西：看得到。重建那一刻它知道盘 1 一条有效根都没读出、也知道这一遍盘 0 上大半槽自证不过；它看不到盘 1 上真正最新的根是哪一条，但「自证不过的槽当作可能住着有效根」这个保守读法在同一份条款里已经用在抬 F 的上限上（「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它」，`.claude/kb/decisions/16-发布语义.md:37`）。
3. 满足判据字面哪一分句：正文第五节「丢一块盘之后，剩下那块盘只读挂载做不做得成」——做不成。
4. 跑前条款给的改法在打中的格上还中不中：正文第五节只写「改法进下一批」，没有列改法，无格可核。我的改法见 1.5。

实现与条款对得上（第 4 步那一句逐字对），被打中的是条款「环里最旧有效根」那一行只并读不出的槽、不并自证不过的槽，与 m 取不到这一支合在一起时的结局。读回全 0 这种读坏第 16 批攻方那一刀就用过（那一刀读回全 0 与报读错同数，判决坐实）；这一次两种读坏头一回分开了结局。

### 1.3 打中甲二：m 与窗口里最旧有效根都取不到（实现员推的那一格，造出来了）

**造法**：读坏只落在第 5 遍（重建分配器读根环那一遍），两块盘的根环全部槽；择根那几遍（第 0、3 遍）、算 F_生效 那一遍都读得出。读回全 0 与报读错两种写流逐字节相同（去重时并成一条）。这一遍里 `window_anchor` 取不到，于是「let unreadable_ring_slots_inside_the_window = window_anchor.map_or_else(Vec::new, |anchor| {」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1838`）给的是空表，窗口里最旧有效根取不到，走实现员写的那一支「// 这一遍一条有效根都没读出、窗口里最旧有效根也取不到时，「只到窗口里最旧有效根」的那一截是空的：当场一个都不放，整截扣到暖机落满。」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1866`）。

**挂载那一刻确实一个都不放；破口在之后**：同一遍读数装进分配器的根环表（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1942`）是一张空表。写行那次发布记根时，进程内的回收门槛按这张表算：「let reclaim_floor = crate::mount::reclaim_floor(」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/allocator.rs:1834`）、「occupancy.oldest_valid_root_in_the_window(),」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/allocator.rs:1836`）——表里只有写行这一条根，窗口里最旧有效根就是写行自己的 txg；接着「self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/allocator.rs:1855`）把释放代高于 F_生效、不高于写行 txg 的那一截当场放开。暖机的空发布「带文件的一版重写固定点单元」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:3432`），取落点就取到这一截。diag 臂原样（`264:6:4:0` n = 15，先不注错、再只坏第 5 遍）：

```
DIAG rebuild effective_floor=20 anchor=Some(20) oldest=Some(0) floor_of_the_reclaim=20 immediately=Some(19) roots_read=21
DIAG record_root txg=22 occupants=22 floor=20 reclaimed=[]
DIAG rebuild effective_floor=20 anchor=None oldest=None floor_of_the_reclaim=20 immediately=None roots_read=0
DIAG record_root txg=22 occupants=1 floor=22 reclaimed=[(0, 1435, 21), (0, 1437, 21), (0, 1439, 21), (0, 1440, 21), (0, 1441, 21), (0, 1442, 21), (0, 1472, 22), (0, 1473, 22), (0, 1474, 22), (0, 14
```

（`out/diag.txt` 第 32、35、38、39 行，第 39 行截到 200 字。）释放代 21 的槽是盘 0 上 txg 20 那条根还引用的单元。条款要的是「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:33`），环里最旧有效根在盘上是 ≤ 19 的那几条，这一截不该放。

**一段具体的历史**（`out/l0-jia-264-n15-rebuildpass-e.txt`）：同 1.2 的起点与卸载刀；再挂时只有第 5 遍两块盘根环都读坏；写行（txg 22，落盘 1）之后暖机 txg 23 的单元写进槽 1441 / 1442；暖机的根落到盘 0 之前崩，丢盘 1，盘 0 择到 txg 20：

```
FXENUMBAD narrow U=264 k=6 u=4 b=0 cont=WalkedToEmpty n=15 plan=r120-144:all/all:e j=0 lost=dev1 chosen=Some(20) surviving_writes_persisted=11 states_with_this_survivor=64 BAD_err[ro:Open(Walk(UnitUnreadable { slot: SlotNumber(1441) }))]
```

这次挂载自己的结局：写行与暖机都落了盘，然后报 `FloorRaiseFailedAfterTheMountsPublishes`，原因是 `RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`（写行之后的准入算抬 F 上限，读窗口里一条有效根的树表读不出；推的：是 txg 20 那条，它的单元刚被暖机写掉，没逐个核是哪一条）。这是两块盘都在时用户就看得到的一个错，而且盘上已经写了。

层 0 枚举域：

| 历史 | 臂 | 状态数 | 坏 | 两盘都在时池级 checker |
|---|---|---|---|---|
| `264:6:4:0` n = 15 只坏第 5 遍 | jia | 20511 | 3075（丢盘 1） | I-7.4 红 4107、I-2.1 4107、I-3.1 4116、I-4.8 4107、I-3.10 1295 |
| `192:4:3:1` n = 17 只坏第 5 遍 | jia | 24618 | 7182（丢盘 0） | I-7.4 红 8214、I-2.1 8214、I-3.1 8223、I-4.8 8214、I-3.10 3019、I-5.1 9 |
| 同上两段 | rr | — | 任何写之前拒（`Recovery(NoValidRoot)`），一个写都没有 | — |

这一格与 C683（池级 checker 不分辨丢盘）不同：两盘都在时池级 checker 就判红，I-7.4（近 K 代块未被复用）。

筛查（只坏第 5 遍，读回全 0 / 报读错同一条写流）原样：

```
[cell-jia-narrow] plan=r:all/all:e histories=1134 refused_before_any_write=33 refused_after_writes=599 histories_with_bad=305 bad_prefix_judgements=3946 starts_with_bad=20
[cell-rr-narrow] plan=r:all/all:e histories=1134 refused_before_any_write=1134 refused_after_writes=0 histories_with_bad=0 bad_prefix_judgements=0 starts_with_bad=0
[cell-ga-narrow] plan=r:all/all:e histories=1134 refused_before_any_write=0 refused_after_writes=303 histories_with_bad=184 bad_prefix_judgements=1460 starts_with_bad=15
```

（每行只抄到 `starts_with_bad`，原行在 `out/summary.txt` 第 138、140、142 行，后半是 z 那一份同数。）普通盘 6 个起点 253 段，三臂都 0 坏；jia、ga 0 被拒，rr 253 段全在任何写之前被拒。窄盘里有 20 / 30 个起点出坏，甲比甲之前（ga）还多：ga 当场放到 F_生效、写行取的落点不同，暖机碰上被提早放开那一截的次数少一些（推的，没逐段对过）。

再挂被拒（问「整截扣住会不会让再挂在预演里被拒」）：jia 在这一格 1134 段里任何写之前被拒 33 段，599 段是写行与暖机落盘之后才报错（不在预演里，是写行之后那次准入抬 F 的上限读不出被写掉的树表）。

任意两遍边界之间两盘全部槽读坏（`pass-*`，每个起点 4009–5726 段）：坏只出在包含第 5 遍、不包含第 0–3 遍的那些窗口；`192:4:3:1` 4524 段里 364 段坏，`264:6:4:0` 40 段，普通盘两个起点 0。

**四句**：

1. 分不分辨臂：分辨「重建那一遍一条有效根都没读出时怎么办」。jia、ga 都中；rr（拒）不中；zs 不碰这一格（`pass-zs` 与 `pass-jia` 同数 40 段）。它不分辨甲的 m 取不到那一支怎么写：那一支在这一格是「一个都不放」，破口在同一遍读数装进去的根环表。
2. 系统当时看得到：看得到。择根那一遍刚择到一条根，重建那一遍一条有效根都没读出（`roots_read=0`），读数前后不一致这件事在它手里。
3. 满足判据字面哪一分句：「剩下那块盘只读挂载做不做得成」做不成；另有一个两盘都在时就看得到的结局（挂载落了盘之后报错），以及池级 checker 两盘都在时判 I-7.4 红。
4. 跑前条款给的改法：无格可核，同 1.2。

实现员报告说这一格「推断走得到的、没造过」，造法就是上面这一种；它的推断（当场一个都不放）对，结论（整截扣到暖机落满）对挂载那一刻成立，对写行之后不成立。

### 1.4 窄盘上扣住让再挂被拒的格子

不注错时一格都没有：窄盘 192–320 槽 813 段、140–180 槽 792 段、普通盘 253 段，`w:-/-:z` 那一行都是「refused_before_any_write=0」（`out/summary.txt` 第 35、149、72 行）。

一块盘根环整段读坏（m 取不到）时，甲当场只放到窗口里最旧有效根，窄盘上取号之前的预演取不到落点（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，一个写都没有）。按盘宽数（`out/summary.txt` 第 37 行，读回全 0、盘 0 或盘 1 整环各一份计划）：

| 单元区槽数 | 历史 | 任何写之前被拒 |
|---|---|---|
| 192 | 132 | 99 |
| 216 | 166 | 66 |
| 248、280、320 | 166、166、183 | 0 |
| 140–180（24 个起点合计） | 792 | 17 |

同一批 7 个起点上 jia 与 ga 对照（`gacmp`）：jia `w:-/all:z` 231 段里 99 段被拒、0 坏；ga 0 被拒、32 段坏（第 16 批那一形）。140 / 156 / 172 槽上 ga 396 段 0 被拒 0 坏（`out/summary.txt` 第 153 行）。报读错时同样数（读不出的槽并进门槛，放得更少）。第 16 批 nh 在 168–200 槽 8724 段里 902 段被拒，这一次的数与它同一个量级、换了取样点。

只坏重建那一遍（1.3）时被拒的样子不同：jia 1134 段里任何写之前被拒 33 段、落了写之后报错 599 段。

### 1.5 改法（我提的，只在我的模型上量过、被攻过零轮）

| 改法 | 甲一（自证不过的槽不并进） | 甲二（重建那一遍一条有效根都没读出） | 不注错的形 R | 代价 |
|---|---|---|---|---|
| jia（入库原样） | 中，量过（1.2） | 中，量过（1.3） | 0 坏，量过（第二节） | 见 1.4 |
| zs：重建算回收门槛时自证不过的槽与读不出的槽一样按最小 txg 并进 | 不中，量过（层 0 同一段 0 / 20511） | 中，量过（`pass-zs` 40 段，与 jia 同数） | 推的：窗口里没写过的槽（新池、环没转满一圈）也会被当成可能有根，门槛只会更低、放得更少，被拒只会更多；没在不注错的族上跑 | 推的，没量 |
| rr：重建那一遍一条有效根都没读出就在任何写之前拒这次可写挂载（`Recovery(NoValidRoot)`） | 不碰（这一遍读得出盘 0 的根），推的 | 不中，量过：窄盘 1134、普通盘 253 段全在任何写之前被拒，0 坏 | 推的：不注错时 `window_anchor` 恒取得到，不走这一支 | 那几段再挂一个写都没有、报错；读坏过去之后再挂。量过 |
| rr + zs | 推的：不中 | 推的：不中 | 推的 | 推的：两者之和 |
| 重建那一遍读坏时重读一次再判（照「根槽这一次读坏」重读一次的路数） | 推的：读坏持续整次挂载时照样中 | 推的：只坏一遍时不中；持续两遍以上照样中 | 推的 | 推的 |

rr 的代价是一个可写挂载报错，盘上不动；jia 在同一格上挂载照样落写、报错，暖机落满之前再崩就读不出。两者之间怎么取、要不要并进 zs，是条款「环里最旧有效根」那一行与「正常卸载」那一行 m 取不到那一分句要重写的事，不归我判。

## 二、第二次抽样（第 16 批「没打中（一次抽样）」各格，照原来的问法、换一组取样点）

### 2.1 甲不注读错的形 R

窄盘 20 个起点 813 段、普通盘 6 个起点 253 段，前缀刀筛查 0 坏 0 被拒（`out/summary.txt` 第 35、72 行）。层 0 两段：

| 历史 | 状态数 | 坏 | 池级 checker |
|---|---|---|---|
| 窄盘 `248:4:3:1` n = 17 | 24618 | 0 | `{}` |
| 普通盘 `256:5:1:2` n = 10 | 20511 | 0 | `{}` |

**分辨力弱，照实记**：`gacmp` 那 7 个起点上 ga 不注错也是 0 坏（「[gacmp-ga] plan=w:-/-:z histories=231 refused_before_any_write=0 refused_after_writes=0 histories_with_bad=0」，`out/summary.txt` 第 88 行）。第 16 批那一批取样点上 ga 窄盘 16 + 18 个起点有坏；这一批新点在不注错的形 R 上分不出甲与甲之前那一版，它的「没打中」不能拿来支撑甲那一句。

### 2.2 扣住的那一截放开得对不对

- 暖机之后、抬 F 之前再崩、再丢盘：普通盘（含 4 GiB 两个起点）6 个起点，再挂做成之后覆盖写 1、3 次，298 段历史 0 坏（`out/summary.txt` 第 158 行；另 2 个起点在起点那一步就撞空间，`start_failed=2`）。前缀刀落在暖机之后的那一截有坏就会记进 `after_mount`，那几格全是 `same` / `known`。
- 同一次挂载里准入抬 F 扣着的会不会被一起放开：窄盘不注错的 3252 条写流（含各份计划里重复跑的那一条）一条都没推准入抬 F（`floor_raises=0`）。注错的历史里推了的写流（`floor_raises=1`），按计划数（`out/summary.txt` `[floor-raise-streams]` 那一行）：只有甲一那两种计划有坏（`w:all/but1:z` 297 条里 51、`w:but1/all:z` 324 条里 34），别的 10 种计划 0 坏；那两种的坏按 1.2 的机理落在暖机的根落满之前（推的，没逐条核刀号）。读代码：「allocator.release_reclaim_holds();」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:4756`）在暖机之后、写行之后那次准入之前，与第 16 批读的同一处。
- 没跑层 0：再挂之后覆盖写的历史状态数上百万，超过单次上限（第五节）。

### 2.3 窄得多的盘上扣住让再挂被拒

不注错：单元区 140、148、156、164、172、180 槽 24 个起点 792 段，任何写之前被拒 0（「[narrower-jia] plan=w:-/-:z histories=792 refused_before_any_write=0」，`out/summary.txt` 第 149 行）；192–320 槽 813 段同样 0。与第 16 批（136–328 槽 12284 段被拒 0）同结论，取样点不同。注错时的数在 1.4。

### 2.4 卸载半途崩溃之后连续两次再挂都在暖机落满之前崩

原型 MODE=r2 `TWICE=1`（第 16 批那一段原样，只加卸载刀 `CUT_LIST`）。13 个起点：窄盘 `192:4:3:1`、`248:4:3:1`、`216:7:5:4`、`280:4:3:1`、`320:4:3:1`，普通盘 `384:3:1:2`、`256:5:1:2`、`240:5:1:2`；另 5 个（`248`、`280`、`320` 窄盘与 `384`、`256` 普通盘）第一次再挂期间盘 1 根环整段读回全 0（第 16 批没扫的那一叠）。每个起点卸载刀 1、15、17、30，第一次再挂每个落在暖机写完之前的前缀刀再挂第二次，第二次的每个前缀刀丢盘 0、丢盘 1 判。原样：

```
[twice-all] j=0 lost=dev0 read=known=20718;j=0 lost=dev0 read=same=99055;j=0 lost=dev1 read=known=12792;j=0 lost=dev1 read=same=106981
```

239546 个判定 0 坏，13 个起点每个 `starts_with_bad=0`。分辨力：这批点上没跑 ga 对照（第七节）。

### 2.5 被抛弃根的过滤

换造法（2.6 那一族）：被抛弃的根留在一块盘上、它那次挂载的暖机没落满就崩，这时那条根就是那块盘上最新的根；再两盘都在可写挂载，重建分配器时 m 滤与不滤真的不同。jia 与 nf 在 4 GiB、384、256、240 槽 × 覆盖写 2、5、8 共 12 个起点、383 段再挂上写流逐条相同（「[ab2-streams] jia=383 nf=383 differing_lines=0」（`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:177`）），每个判定也逐数相同。

为什么同（推的）：这几段历史里 F_生效 低于两种 m，当场放开的是 min(F_生效, m) 与窗口里最旧有效根取大，F 那一半压住了。要让过滤改结局，得 F_生效 高于被抛弃那条根的 txg，即卸载那一串抬了 F 之后、被抛弃的根仍是某块盘上最新的根；卸载接在一次做成的可写挂载之后，那次暖机已在每块盘上落了新实例的根，走不到。另一个方向：滤只会让 m 更小、放得更少，丢盘之后剩下那块盘择到的正是那块盘上最新的根（它不知道那条被抛弃），不滤的 m 恰好等于它，所以滤与不滤都不会让读回变坏，最多让窄盘上多拒一次可写挂载。没造出这一形。

### 2.6 单盘恢复择中被抛弃的根（换一种造法：被抛弃之后、暖机落满之前再崩）

造法（原型 MODE=ab2）：普通盘覆盖写 k 次；再录一次覆盖写，崩在它的根槽写落下之后（系统配置没轮换）；在那一刀上可写挂载，挂载期间那条根的根槽与那次发布的单元读回全 0（瞬时），恢复落到上一版、取新实例，那条根成了被抛弃的根；这次挂载的写流每个前缀刀 m1 上崩（含写行的根没落、暖机没落满两段），丢一块盘，剩下那块盘单独恢复。

结局（jia，`out/summary.txt` 第 175 行原样里的几项）：被抛弃的根所在那块盘单独恢复时择中它，暖机落满之前的 464 个判定全择中（`before_row_root` 248、`before_warm_up_covers` 216），其中 99 个只读挂载读不出：

```
AB2CHOSE ab2 W=384 k=5 abandoned_txg=9 row_txg=10 warm_up=1 mount_writes=38 m1=3 before_row_root lost=dev1 chosen=Some(9) ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1536) }))
```

读不出的那几个都在覆盖写 5 次那三个起点（384、256、240 槽），被抛弃那次挂载从第 3 个写落下起、到暖机落满之前（每个起点 16 + 17 个判定）：那次挂载读不出被抛弃那条根的账（单元读回全 0），影子账隔离不到它引用的槽，写行那次取落点取到其中一个。看着是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）那一族的另一种进入法：C554 写的是「撤故障后再把新实例的根全改坏，恢复落在被抛弃实例的根上读不出」（`.claude/kb/checks-owed.md:427`），这里是暖机落满之前崩、丢另一块盘。nf 逐数相同，与第 17 批的改动无关。暖机落满之后（`after_warm_up` 72 个判定）择中 0 次，与第 14、16 批的结论一致。

四句：不分辨臂（jia、nf 同数，甲碰不到这一格）；系统当时看不到（被抛弃那条根的账在那次挂载里读不出，影子账没东西可隔离）；满足「只读挂载做不做得成」做不成；正文第五节没有给这一格的改法。按「打中不分辨臂」那一行处置：不拿它判甲，另记进 C554 那一笔（交主 agent 判是不是同一笔）。择中而读得出的那 365 个判定，读到的是被抛弃那一版的内容；那次挂载还没交回（暖机没落满），用户没看到过新实例，这算不算坏，我不判。

## 三、N6：有没有一种错实现四条 Q-F 用例照样过

有，三种（补丁在 `patches/qfm.patch`、`qfu.patch`、`qfr.patch`，只改冻结副本的拷贝）：

| 错实现 | 改哪儿 | Q-F 那一个测试目标（18 条，四条 Q-F 用例在内） | 我加的判别形 |
|---|---|---|---|
| qfm：会话里时间只进不退（会话记下给过的最晚时间，unlink 与建 inode 的时间取 max(给的, 它)） | `mounted_session.rs` 加一个字段，`publish_user_change` 开头改 `change` | 18 / 18 过 | 会话里先建一个 inode（上一版 + 1000 秒），再 unlink（上一版 + 300 秒）：红 |
| qfu：意图时间按 32 位存（`u64::from(write_time_seconds as u32)`） | `transaction.rs` 意图那一格 | 18 / 18 过 | 给 2^32 + 7 秒：红 |
| qfr：意图时间取整到分钟（`write_time_seconds / 60 * 60`） | 同上 | 18 / 18 过 | 给上一版 + 7 秒：红（2^32 + 7 那一条也红） |

qfm 那一行 `research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:228`：「[qf-qfm] test attack_a_session_unlink_after_a_later_new_inode_in_the_same_session_writes_the_given_time ... FAILED;test result: FAILED. 20 passed; 1 failed」，同一行后半「test result: ok. 18 passed; 0 failed」是 Q-F 那一个目标。不改的副本上两个目标 21 / 21、18 / 18 全过（第 227 行）。

为什么漏：四条用例给的时间都是上一版 ± 600 秒（1787999460、1788000660，都是 60 的倍数、都小于 2^32），会话里 unlink 都是这次挂载的第一次改动。Q-F 乙只说意图写调用方给的时间、各批照抄。三种错实现都与「意图取 max(给的, 上一版)」那一刀不同，变异表第 2476–2477 行那两行抓不到它们（推的，没在变异驱动上跑）。改法（零轮，推的）：用例加一形「同一会话里先给一个更晚的时间再 unlink」、一形给不是 60 倍数且大于 2^32 的时间；三条判别形就是这个写法（`probe/qf_attack_probe.rs`，在副本上跑过）。

## 四、顺手看到、不在我这几格里的（线索，没深挖）

- 甲一那一族的另一种选槽法 `w:all/but1:z`（盘 0 整环读回全 0、盘 1 除最新那条根之外读回全 0），`192:9:3:1` n = 8 那一段层 0 28733 个状态里我的判法 0 坏，但两盘都在时池级 checker 在 4116 个状态上判 I-7.9（回退下界 F 不高于抬 F 的上限）红（「checker_violated={"I-7.9": 4116} bad_judgements=0」（`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt:178`））。那次挂载择到的是盘 1 上的 txg 19（盘 0 的根这一刻全看不见），之后推了一次准入抬 F。是实现越了上限还是 checker 在这种读数下判错，我没核。
- 条款「环里最旧有效根」那一行写「六处都不再按一圈窗口滤根」（`.claude/kb/decisions/16-发布语义.md:34`），重建分配器算窗口里最旧有效根时仍按一圈窗口滤：「.filter(|root| !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg))」（`/tmp/claude-1000/m3-batch17-code-r1/tree/crates/singlefs-core/src/mount.rs:1849`）。这是 K1 那一格的事，正推腿判；我没造窗口之外留着旧根、又改结局的历史。

## 五、取样、分批与估时

- 盘：全部内存稀疏盘（`SparseBlockDevice`），没有在磁盘上建镜像文件；每个起点在内存里建一次，卸载、再挂、两次再挂都在 `MemoryPool` 的拷贝上录。
- **崩溃状态**：筛查（第一、二节的大部分数）每段历史只判再挂写流的每个前缀刀，是层 0 枚举域的一个子集；层 0 枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开、段内整写子集逐个枚举、每个状态池级 checker 照跑）跑在八段历史上（模型目录 `rerun.sh` 的 l0-1 到 l0-8；草稿里另把 l0-1 那一段多跑过一次）。这一条没照定义「留下的每段历史，它的崩溃状态一个不落」办：空目录复跑里筛查判过的不同写流 30129 条（`FXPLAN` 行数），每条若展开层 0 约 2×10⁴ 个状态，合计约 6×10⁸ 个（推的，按层 0 那几段的状态数乘），按本机实测每段约 2 分钟（14 线程、20511 个状态约 120 秒）要上千小时，远超约 10⁷ 的总量上限；所以只把打中的、对照的、不注错的各挑几段展开。筛查的「没打中」只有前缀刀的证据。
- 层 0 状态数：空目录复跑八段 180524 个（20511 × 5 + 24618 × 2 + 28733），草稿目录九段 201035 个，合计 381559 个；每次都不超过约 10⁶。超过约 10⁶ 而没跑的：普通盘再挂之后覆盖写的历史（2.2），`384:6:1:2` n = 10 的 j = 1、j = 3 各 17047580、19144744 个，`256:3:1:2` n = 10 的 j = 1、j = 3 各 1073180、3170344 个（`closed_form_state_count` 现算）。
- 估时：开跑前只试了单个起点（窄盘一个起点 33 个卸载刀 × 34 份计划约 31 秒；一段层 0 约 130 秒），没有先估全量；草稿里窄盘整次挂载那 34 份计划原定 108 个起点，跑了几批之后按实测（每批 14 件 2–3.5 分钟，本机同时有别的会话把负载推到 30–46）缩成空目录复跑里的 20 个起点（单元区 192–320 × 覆盖写 4、9 × 单元 3、10 × 批 1），只坏重建那一遍那一格缩成 30 + 6 个起点（批 4 去掉），4 GiB 盘的读坏扫两次撞 235 秒限时，丢掉没用。空目录复跑各阶段挂钟合计约 53 分钟（编译 4.4 分钟、窄盘读坏扫 16.3 分钟、层 0 八段 15.2 分钟、其余 14.4 分钟，另 build-ga 第一次排不上等了 3 分钟），超过 40 分钟这一条我没在开跑前照办。
- 分批：每批件数按单件试跑定（14 件、9–13 件），不给批设限时；每条前台命令外面套了 `timeout 235`（只为不越过前台 4 分钟），被它掐断的件由 `redo-fxw` / `redo-cell` 阶段原样补跑（空目录复跑里补了 1 件），汇总只算跑完了的起点（`scripts/complete.sh`）。定义要的是批不设限时，这一条外层 `timeout` 照实记。
- 线程：筛查每件 1 线程、同批不超过 14 件；层 0 枚举与判定各 14 线程；编译 15 线程。
- 内存包装：筛查每件 256M（实测峰值 46–58M）、层 0 6G；编译 4G。`build-ga` 第一次退 252（「slice 已占 39.9 GiB，这一条要 2.4 GiB」，别的会话占着），重跑做成；草稿里 `build-diag` 一次排队时被我外层 `timeout 110` 掐断，重跑做成。这几次的输出都没用。
- 判法：草稿里先跑的一遍与空目录复跑在重叠的取样点上逐数相同（层 0 八段的坏判定数 3075、14339、0、7182、0、8195、0、0；`pass-*` 每个起点的坏段数；`ab2` 两臂；Q-F 四份副本）。

## 六、没打中的形状

| 形状 | 取样 | 结局 |
|---|---|---|
| 再挂期间读坏报读错（`e`）的 17 种选槽法 | 窄盘 20 个起点 813 段、普通盘 6 个 253 段；更窄 24 个 792 段只跑单块盘整环那 2 种 | 0 坏；被拒数见 1.4 |
| 读坏只落在一块盘、或只落在最新那条根所在槽（`only1`） | 同上 | 0 坏 |
| 挂载交回之后读坏还持续（`:p`），再挂之后不写 | 同上 | 与不持续的同数 |
| 只坏择根那几遍（第 0–3 遍）、不坏第 5 遍 | `pass-*` 9 个起点 | 0 坏（坏只出在包含第 5 遍、不含第 0–3 遍的窗口） |
| 普通盘上的甲一、甲二 | 6 + 2 个起点 | 0 坏 |
| 不注错的形 R、扣住那一截放开、更窄的盘被拒、连续两次再挂、被抛弃根的过滤 | 第二节 | 0 坏 / 0 被拒 / 写流逐条相同 |

## 七、这条腿自己的限度

- 大部分数是前缀刀筛查，层 0 只展开了八段（第五节）；没打中的各格只有前缀刀的证据。
- 读坏模型只有两种：读回全 0、报读错。读回别的乱码（自证不过但不是全 0）没跑，推的与全 0 同一支。读坏的边界只切在整遍读之间，一遍读的中间切开没扫。
- 2.1 的新取样点在不注错时连 ga 都不分辨；2.4 没跑 ga 对照，分辨力没量。
- 甲二里 jia 比 ga 坏得多那一句只是推的解释，没逐段对。
- zs、rr 都只在我的模型上量过，被攻过零轮；zs 在不注错的族上的代价没量。
- 我的判法（剩下那块盘只读挂载、整读、比哈希）是我自己的结局 oracle；甲二另有两盘都在时池级 checker 的 I-7.4 可对照，甲一没有（池级 checker `{}`，C683 那一笔）。
- 2.6 的读不出归不归 C554、择中而读得出算不算坏，我没判。

## 八、没做什么

- K1、K3 与正推那几格不判；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何重型测试与 checker 档包的测试二进制；层 0 枚举函数只在自己的原型 crate 里调，流是原型自己录的；Q-F 那一个目标与我加的判别形目标是 harness 档，在冻结副本的拷贝上跑。
- 没改仓里任何别的文件，只写了本报告与模型目录。
- 回溯栈认遍号那一步（`FX_TRACE=1`）只在草稿里跑过，命令：`MODE=fx FAMILY=list SHAPES="264:6:4:0;" FX_PLANS=clean CUT_LIST=15 FX_TRACE=1 bash scripts/run.sh jia <日志>`（在臂目录建好之后）。
- 草稿目录 `/tmp/claude-1000/m3-batch17-code-r1-opus/`：交回之前删掉自己建的仓副本与编译目录；留着 `W/out/`（草稿那一遍的原始日志）与 `R/out/`、`R/summary.txt`（空目录复跑的原始日志与判定行），主 agent 要逐字核可以去读。
