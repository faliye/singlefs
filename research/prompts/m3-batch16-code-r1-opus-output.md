# m3-batch16-code-r1 攻方腿（Opus）报告（2026-10-04）

攻击面：正文第三节 K2（重点 N1 甲与第 14 批代码轮「没打中」各格的第二次抽样），另带 N4、N5、N7。冻结副本 `/tmp/claude-1000/m3-batch16-code-r1/tree/` 只读，拷到草稿目录改、编、跑；只跑了 harness 档测试目标与自己的原型 crate `attack-b16-probe`。副本上的数都是副本上的数，不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-batch16-code-r1-opus-model/`。在一个空目录 W 里按阶段跑（每个阶段前台 4 分钟以内；本机有别的会话的 cargo 在跑时，个别阶段到过 3 分 40 秒）：

```
M=research/prompts/m3-batch16-code-r1-opus-model
bash $M/rerun.sh list W            # 列出 45 个阶段，按次序逐个 bash $M/rerun.sh <阶段> W
bash $M/rerun.sh compare W         # 重算判定行，与 $M/out/summary.txt 逐行比
```

冻结树认 `FROZEN_TREE`（默认 `/tmp/claude-1000/m3-batch16-code-r1/tree`）。交回之前在空目录 `/tmp/claude-1000/m3-batch16-code-r1-opus/R` 里从 build-jia 到 compare 逐阶段跑完一次，compare 原样输出：

```
compare: 判定行与 out/summary.txt 逐行相同（244 行）
```

模型目录各文件 sha256（`find . -type f | sort | xargs sha256sum` 原样）：

```
f4c384d399ae2dc38369610b618350221ef792c2afb4abd7dda3726aa46d6451  out/bad-samples-ga.txt
a88498defd519abc7d3e6f8584df1e366136745e0ec2e3d5ad790f503385ebac  out/bad-samples-jia-hide1.txt
bb33b93655056580639b9e2688f63159a51742030afd59ea839ed901ed7461ee  out/e1-renum-h2-jia-hide1.log
5e2f419c1c2dbfa94ce0fd8955d9d280307b127df050580fbf977bfa54370442  out/e2-renum-h2-jia.log
9e198d1c351d5b297d7200704d7da88f7ae27bb8b72e124d9816ad26b610a092  out/l1-ga.log
d255bb24e75434d73abaffdafe1f952e01a3cd38720dab7a357ddc6fc16a56f3  out/l1-jia-hide1.log
62316256eb36aba9c9215f40227743167a37284bbdec428ab566a8ee8a02254a  out/l1-jia.log
46d3ba58adc3c46e740956dcaa2cbec69c7564e031873c9917057f3db8cda7e9  out/l1-nh-hide1.log
52f1910e0794f0a354e2c8490af631f9cf14556462d42b446eb115b50547b436  out/l2-ga.log
ab3962ff01948d0e3c16d53730406740f802c65cf9724bbb6b10afa4ca2c9db5  out/l2-jia-hide1.log
7c79b7c3047f938558ec91cfd5e28e4e575daad2c818ec8cae4dddce2e3a6f9e  out/l2-jia.log
4f7e12b9506e39d064cfb312f16a5115bf86c10e273bdd6ee6cda033a4085edd  out/l2-nh-hide1.log
d3d363fecca4a4cd69ae7b1b593da860824a40252be4b28b45b1fe45465befd6  out/qf-n7clamp.log
e394d6afed6ede39ce240ffc6a19efadad05476f9abf9ba082149364eccf9999  out/remount-err-samples-nh-hide1.txt
28fbb5a262e6fcf3f35cf541a508abcdb35bdf5b53c357ac44232ea3b80d14a9  out/remount-samples-jia-hide0.txt
b55b0a73b644aa186945c2f1eb19f94e46575a40120dddefa71cccd70a4a131e  out/summary.txt
f6eed511644eb85340e00656c64501c7f865cb194f0024b984d6a0957fcebaa2  out/tests-n7clamp.log
170de835c843dbcba42ca87442f6cea60264b4f209ed8beb6513c0e32c9586f2  patches/ga.patch
3b33f7d8256013205599406534bc15aca20579b1c5e1f5ccef029de823261872  patches/imm.patch
de8f89e9d799eebc11daf7fb657e54c7bd87d21422416d5b7a4528aa4116b934  patches/n7-clamp.patch
74ec3a38cde0d0ff69d9ad863d5476c79af50df25f8cc9352fa05174ace07946  patches/nh.patch
9558e45bfd284d2eed84d1387b611c5e0f4a258180f5d1c0ac295a3c01e9ef11  patches/workspace-member.patch
2577d44e100a661e85301949ec67c8de6e6a4abf455774b21f84c8f1dca9424a  probe/Cargo.toml
6badb4d999e8dbf8cd589f17dd5fd1f18068b2845e86a3f8d652a935ae5650f8  probe/main.rs
35d264e97696df98e47b72643decc575cb1e942c25bbd4db0a114f963ac5a2d5  rerun.sh
293351a3d28bee45dc10e3e37b3a2c883751b14582c7c5a32bb99288298b2c10  scripts/build.sh
90beaeb129adce7e30795c12b31a52889071ec965133f615bb1834780764038d  scripts/grid-batch.sh
db86967a5e1c25e74ea0e60d6a57c4fc4012354683d99b24bfcf2610032dc5ae  scripts/n1sum.sh
43705aa906f7d3c5ecb50f7d9191aa2cd7d7f2f95b485a68b648a6772802fcb9  scripts/r2sum.sh
c6ca02956ce05db187201828f926f76d7e76d59fd9d762078e52a86cb19c9d39  scripts/run.sh
edfe168863b4e0e2c71cbeb021582358fa3a35d5e92f30c2492eb4c345ca9321  scripts/summary-all.sh
211a542b06602de5711438b0af353923055190fb314b9ad2482c1a9744ecf138  scripts/sweep-batch.sh
```

臂（都在冻结副本的拷贝上，补丁在 `patches/`）：
- **jia**：冻结副本原样（甲已在里面）。
- **ga**：甲之前那一版，重建分配器整截当场放开（`patches/ga.patch`，只改冻结副本 `mount.rs` 第 1883 行那一句）。只作对照，看取样点分不分辨。
- **nh**：我提的改法，零轮：重建分配器时某块盘一条有效根都读不出（m 取不到），当场只放到一圈窗口里最旧有效根，不照原门槛整截放开（`patches/nh.patch`，只改第 1881 行）。
- **imm**：卸载那一格整截当场放开（上一轮的丙1 对照），只用在第二次抽样，看新取样点分不分辨卸载那一格。
- **n7clamp**：N7 的一种错实现，意图那次写入时间取 max(调用方给的, 这条记录上一版的)（`patches/n7-clamp.patch`）。

## 各格判定一览

判定都是一次抽样。「判定」是一个崩溃状态丢一块盘之后剩下那块盘的结局：只读挂载、整读每个活文件，读不出（或读出的不是任何一版合法内容）记坏（`probe/main.rs` 的 `R2Judge`）。卸载之前剩下那块盘自己就读不出的（C695 那一形）单列，不记坏。

| 格 | 判定 | 要点（数出自 `out/summary.txt`） |
|---|---|---|
| N1 甲：形 R 两族放开扫，不注读错 | **没打中** | jia：窄盘 222 + 更窄 95 个起点、普通盘 96 个起点，再挂历史 8724 + 3560 + 8664 段，坏 0、再挂被拒 0。同一批取样点上 ga 窄盘 16 + 18 个起点、普通盘 9 个起点有坏，取样点分辨甲 |
| N1 甲：再挂这次挂载期间某块盘的根环整段读坏（重建时 m 取不到） | **打中** | jia 与 ga 逐格同数：窄盘 16 个起点 5479 个坏判定、普通盘三档窄宽 6 个起点每个 j 1362 个；层 0 枚举域两段历史各 20511 个状态里 14339、15363 个坏，另一段 h2 12302 个。nh 臂（零轮）同一批 0 个坏，代价是窄盘 902 / 8724 段再挂在任何写之前被拒 |
| N1 甲：扣住的那一截放开得对不对（暖机之后再覆盖写、同一次挂载里准入抬 F） | **没打中** | 普通盘再挂做成之后覆盖写 2 次（j = 2），落在暖机之后的前缀刀 jia 0 坏；nh 读坏族里 3158 段再挂在暖机之后推了准入抬 F，每个前缀刀 0 坏 |
| N1 甲：窄得多的盘上扣住让再挂被拒 | **0 格** | jia 在单元区 136–328 槽上 12284 段再挂，被拒 0（不注读错）。注读错时 jia 也 0，nh 902 |
| N1 甲：卸载半途崩溃之后连续两次再挂都在暖机落满之前崩 | **没打中** | 4 个起点（3 窄盘、1 普通盘），jia 坏 0；ga 4 个起点都坏 |
| 第二次抽样：GA 卸载那一串（四族） | **没打中，但取样点分辨力弱** | 新取样点 jia 322 个起点读法三交集 0、整读坏 0。imm 对照在窄盘族 12 个起点有交集、整读坏 0；cycle、ncycle、abandon 三族 imm 也是 0，这三族的新点分不出卸载那一格 |
| 第二次抽样：单盘恢复择中被抛弃的根 | **没打中** | abandon 族新点 2799 个判定，择中被抛弃的根 0 次 |
| 第二次抽样：Q-F ①b | **没打中** | 180 份镜像各自续做两次逐字节相同；696 步带这条记录的续做批，写入时间都等于意图那次给的 |
| N4 | 卸载那一支**从入口走不到**；重建那一支**走得到且改结局** | 卸载期间一块盘根环整段读坏，1272 次全在重读那一步被拒，走不到「整截扣」；重建那一支的 m 取不到见第二行 |
| N5 | **没找到** | 没造出让滤与不滤改结局的历史；论证是推的，见第六节 |
| N7 | **打中（测试漏判）** | 意图时间取 max(给的, 上一版) 的错实现，5 个 harness 目标 71 条全过；原型 Q-F 那一刀（给的时间早于上一版）抓到 |

## 一、N1 甲：形 R 放开扫

### 1.1 扫了什么

一段历史 = 起点 → 在起点的会话上录一次卸载 → 卸载写流的前缀刀 n（只取系统配置已写、根还没落满每块盘的那一截）→ 两块盘都在、再可写挂载（窄盘族照起点同一种开关，普通盘族照 `crash_and_mount_writable` 那一种）→ 做成之后照用户动作覆盖写 j 次（窄盘族第一个文件已删，改成建一个 inode）。挂载与 j 次写录在同一条写流里，每个前缀刀 m 都判丢盘 0、丢盘 1。用户决定的几步放开：

| 参数 | 窄盘族 | 普通盘族 |
|---|---|---|
| 单元区槽数 / 盘宽 | 136, 144, 152, 160, 168, 184, 200, 232, 264, 296, 328 | 4 GiB、384、256、240 槽 |
| 起点覆盖写次数 | 0, 1, 3, 6（更窄那一组 0, 1, 3） | 0, 1, 2, 4 |
| 文件单元数 / 意图后批数 | 2, 4, 8 / 0, 2, 3 | — |
| 崩了再挂次数 / 每次之后覆盖写 | — | 0, 1, 2 / 0, 1 |
| 卸载里崩在哪个写之后 | 系统配置已写、根没落满的每一刀 | 同左 |
| 再挂里崩在哪个写之后 | 每一刀（含暖机之后、j 次写里） | 同左 |
| 丢哪块盘 | 0、1 | 0、1 |
| 再挂之后覆盖写 j | 0（窄盘再挂之后建 inode 报 `NoSpaceAfterRaisingTheFloor`，ga 上同样，窄盘本身的性质） | 0, 2 |

取样点与第 14 批攻方那一份不同：那一份窄盘取 176–336 每 32 槽、覆盖写 0/2/5/8、单元 1/4/14、批 0/1/3，普通盘覆盖写 0/1/3/7；这一份见上表。

### 1.2 不注读错：没打中

`out/summary.txt` 原样（各组头两行）：

```
[r2-narrow-jia] starts=222 start_failed=30 remounts_ok=8724 remount_err=0
[r2-narrow-jia] starts_with_bad=0
[r2-narrow-ga] starts=222 start_failed=30 remounts_ok=8724 remount_err=0
[r2-narrow-ga] starts_with_bad=16
[r2-narrower-jia] starts=95 start_failed=13 remounts_ok=3560 remount_err=0
[r2-narrower-jia] starts_with_bad=0
[r2-narrower-ga] starts=95 start_failed=13 remounts_ok=3560 remount_err=0
[r2-narrower-ga] starts_with_bad=18
[r2-plain-jia] starts=96 start_failed=0 remounts_ok=8664 remount_err=0
[r2-plain-jia] starts_with_bad=0
[r2-plain-ga] starts=96 start_failed=0 remounts_ok=8664 remount_err=0
[r2-plain-ga] starts_with_bad=9
```

起不来的起点（30 + 13）：意图之后的批数超过文件单元数（`UnlinkOfAnInodeThatIsNotALiveRecord`）与起点那次可写挂载取号之前被拒，与甲无关。

### 1.3 打中：再挂这次挂载期间一块盘的根环整段读坏

**一段具体的历史**（窄盘 `U=264 k=6 u=4 b=0`，层 0 那一段 `out/l1-jia-hide1.log`）：

1. 起点：不补环窗口、准入关着的窄盘（单元区 264 槽），第一个文件之后覆盖写 6 次、顺序写长到 4 个单元，发 unlink 意图就崩；重开可写续做到空。卸载之前现行那一版 txg 20，盘 1 上最新的根 txg 19。
2. 卸载那一串落下 15 个写就崩：两块盘的系统配置都带了新 F = 20，带新 F 的根一条都没落（`R2START … current_txg=20 unmount=Raised publishes=2 writes=36`，`n=15 sc=2 roots=0`）。条款许可这一刀：D16 已定项 1「抬 F 那一串」先写系统配置、过屏障再发根。
3. 两块盘都在、再可写挂载。**这次挂载期间盘 1 的根环每一槽都读坏**（原型里读回全 0，`HIDE_ERR=1` 时改成报读错，两种结局同数；挂载交回之后读得出）。条款把挂载那一刻读坏的槽当作这一次的正常读数处理：「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽」（`.claude/kb/decisions/16-发布语义.md:38`）。挂载做成（`ok chosen=20 effective=21 … row_txg=22 warm_up=1`）。
4. 重建分配器时盘 1 一条有效根都没读出，`newest_valid_root_txg_on_every_device` 交回 None：「- 有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效（不按 txg 0 去比，说清是哪一格）。」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/mount.rs:2474`）；重建那一处的 None 支是「None => floor_of_the_reclaim,」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/mount.rs:1881`），接着「allocator.reclaim_released_up_to(floor_released_immediately, ReclaimedReuse::Immediately);」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/mount.rs:1883`）按原门槛 max(F_生效 = 20, 窗口里最旧有效根) 整截当场放开。这一支照条款写：「其余扣到这次挂载的暖机在每块盘上都落了根，某块盘一条有效根都读不出时照」（`.claude/kb/decisions/16-发布语义.md:42`，后半句是「可再分配」那一行的门槛）；代码注释同一句「// 某块盘一条有效根都读不出（m 取不到）时照原门槛当场放开。」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/mount.rs:1863`）。
5. 写行那次的单元写进释放代 20 的槽，其中一个是盘 1 上 txg 19 那条根还引用的映射树节点（槽 1410 或 1412）。写行的第 6、7 个写之后、暖机的根落到盘 1 之前再崩。
6. 丢盘 0。盘 1 此时读得出（读坏只在那次挂载期间），恢复择根不看 F，择到它自己上面最新的 txg 19，只读挂载走树读不出：

```
R2BAD narrow U=264 k=6 u=4 b=0 cont=WalkedToEmpty n=15 sc=2 roots=0 j=0 m=6 next2=unit_write R2Verdict { lost: 0, chosen: Some(19), read: "BAD_err[ro:Open(Walk(MappingStillUnreadable { slot: SlotNumber(1410)]", inter_unmount: 0, inter_remount: 1 }
```

同一段历史不注读错时（jia），重建按 m = 19 只当场放到 19，释放代 20 的槽扣到暖机之后，这一刀读得出。

**几臂对照**（`out/summary.txt` 与层 0 日志原样）：

```
[r2-narrow-jia-hide1] starts=222 start_failed=30 remounts_ok=8724 remount_err=0
[r2-narrow-jia-hide1] starts_with_bad=16
[r2-narrow-jia-hide1] sum prefix j=0 in_mount lost=dev0 read=BAD_err count=5479
[r2-narrow-ga] sum prefix j=0 in_mount lost=dev0 read=BAD_err count=5479
[r2-narrow-jia-hide0-half] sum prefix j=0 in_mount lost=dev0 read=BAD_err count=3404
[r2-narrow-jia-hide1err-half] sum prefix j=0 in_mount lost=dev0 read=BAD_err count=3404
[r2-narrow-nh-hide1] starts=222 start_failed=30 remounts_ok=7822 remount_err=902
[r2-narrow-nh-hide1] starts_with_bad=0
[r2-plain-small-jia-hide1] sum prefix j=0 in_mount lost=dev0 read=BAD_err count=1362
[r2-plain-small-nh-hide1] starts_with_bad=0
```

层 0 枚举域（再挂写流按屏障与 FUA 切段、段内整写子集逐个枚举，每个状态池级 checker 照跑；数的是丢盘 0 那一侧读不出的状态）：

| 历史 | 状态数 | jia | jia + 盘 1 根环读坏 | nh + 盘 1 根环读坏 | ga |
|---|---|---|---|---|---|
| 窄盘 `176:0:4:3` n = 1（h2，第 14 批那一形） | 20511 | 0 | 12302 | 没跑 | 没跑（第 14 批判决记 12302） |
| 窄盘 `264:6:4:0` n = 15 | 20511 | 0 | 14339 | 0 | 14339 |
| 普通盘 `240:0:1:1` n = 1 | 20511 | 0 | 15363 | 0 | 15363 |

两盘都在时池级 checker 0 违例（`checker_violated={}`），与 C683 同一笔。层 0 自带的版本 oracle 在窄盘族两臂上都报 `violations=20511`（第一个文件已删，版本表只记文件内容，不分辨臂），普通盘族两臂都报 0；我不拿它判。

盘 0 的根环读坏（`HIDE_DEV=0`，窄盘一半取样点）同样中、同数（3404）：那次挂载从盘 1 择根、journal 重放到更新的一版（`out/remount-samples-jia-hide0.txt` 第 3 行 `chosen=10 effective=11`），重建时盘 0 一条有效根都没读出，m 同样取不到。

**四句**：

1. **分不分辨臂**：分辨。甲（入库原样）与 ga（甲之前）在注读错的每一格上逐格同数；第 14 批攻方的 FX 原型 None 支同样取原门槛（`research/prompts/m3-batch14-code-r1-opus-model/patches/fx-mount-holds-above-the-newest-root-on-every-device.patch` 里 `map_or(floor_of_the_reclaim, …)`，推的，FX 这一轮没跑）；nh 不中。所以这一格不分辨「甲」与「乙 不改代码」，分辨的是 m 取不到时放多少。
2. **被判的系统当时看不看得到判别它的东西**：看得到。重建分配器那一刻它知道某块盘一条有效根都没读出（m 是 None），也知道 F_生效 来自系统配置；它看不到那块盘上真正最新的根是哪一条，但可以按保守的读法处置，nh 就是一种。
3. **满足的是判据字面的哪一个分句**：正文第五节「N1 判的是用户看得到的结局：丢一块盘之后，剩下那块盘只读挂载做不做得成、整读活文件与卸载之前读出的相同不相同。」（`research/prompts/_m3-batch16-code-r1-background.md:93`）的「只读挂载做不做得成」。实现与条款字面对得上；被打中的是条款「某块盘一条有效根都读不出时照「可再分配」那一行的门槛」那一分句本身，不是实现偏离条款。
4. **跑前条款给的改法在打中的格上还中不中**：正文第五节只写「打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。」（`research/prompts/_m3-batch16-code-r1-background.md:92`），没有列改法，这一句无格可核。我提的改法见 1.4。

### 1.4 改法（都只在我的模型上量过或只是推的，被攻过零轮）

| 改法 | 不注读错的形 R | 再挂期间一块盘根环整段读坏 | 代价 |
|---|---|---|---|
| 甲，入库原样（m 取不到时照原门槛） | 0 坏，量过（第 1.2 节） | 中，量过（第 1.3 节） | 再挂被拒 0，量过 |
| nh：m 取不到时当场只放到一圈窗口里最旧有效根（挂载时读不出的槽按它在窗口里能住的最小 txg 并进这个最小值，与回收门槛同一个读数），其余照甲扣到暖机之后 | 推的：nh 只改 m 取不到那一支，不注读错的历史上 m 都取得到时与甲走同一条路；nh 没在不注读错的族上跑 | 0 坏，量过：窄盘 222 个起点、普通盘三档窄宽 72 个起点、层 0 两段各 20511 个状态 | 窄盘注读错时 902 / 8724 段再挂在任何写之前被拒，都是 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`、`writes_before_err=0`（`out/remount-err-samples-nh-hide1.txt`），只在单元区 168 / 184 / 200 槽上（485 / 298 / 119 段）；普通盘 0。量过 |
| 照卸载那一格：m 取不到时这一截一个都不当场放、整截扣到暖机之后 | 推的：同 nh | 推的：不中（比 nh 扣得更多） | 推的：被拒不少于 nh |
| m 取不到时拒这次可写挂载 | 推的：同 nh | 推的：不中 | 推的：注读错的每一段再挂都被拒（窄盘 8724、普通盘 6162 段） |
| 重建前对挂载那一刻读不出的根环槽重读一次 | 推的：同甲 | 推的：读错持续整次挂载时照样中（我的注错就是这一形） | 推的 |

nh 被拒那 902 段，用户看到的是可写挂载报错、盘上一个字节不变，读错过去之后再挂。甲在同一批历史上挂载做成，但暖机落满之前再崩、丢掉另一块盘就读不出。两者之间怎么取，是条款那一分句要重写的事，不归我判。

## 二、扣住的那一截放开得对不对

**暖机之后、抬 F 之前再崩、再丢盘**：普通盘族 96 个起点，再挂做成之后覆盖写 2 次（j = 2），这两次的单元会落进暖机之后才放开的槽。前缀刀落在暖机之后（`after_mount`）的判定原样：

```
[r2-plain-jia] sum prefix j=2 after_mount lost=dev0 read=known count=99240
[r2-plain-jia] sum prefix j=2 after_mount lost=dev0 read=same count=141800
[r2-plain-jia] sum prefix j=2 after_mount lost=dev1 read=known count=154220
[r2-plain-jia] sum prefix j=2 after_mount lost=dev1 read=same count=86820
```

坏 0（`known` 是读出的内容等于再挂之后某一版两盘整读的内容）。这一段 j = 2 的层 0 枚举域每段 2.1×10⁶ 到 5.4×10⁸ 个状态，超过约 10⁶ 的单次上限，没跑（第八节）。

读代码：暖机之后那一句放开「allocator.release_reclaim_holds();」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/mount.rs:4748`）排在写行之后的准入（`push_floor_raises_after_the_row_publish`）之前；抬 F 那一串自己置的扣住位在它做完之后自己放（同一句在第 3204 行）。

**同一次挂载里准入抬 F 扣着的会不会被一起放开**：注读错的族里有大量再挂在暖机之后推了准入抬 F（jia 盘 1 读坏 3424 段、盘 0 读坏（半组取样点）2766 段；nh 盘 1 读坏 3158 段，`R2REMOUNT … floor_raises=1`）。nh 那 3158 段在每个前缀刀（包括抬 F 那一串中间）上丢盘 0、丢盘 1 都是 0 坏；jia 那几段的坏按第 1.3 节的机理只会落在暖机的根落到盘 1 之前（推的；打印出来的坏判定每个起点前 3 条，刀号 m 在 4–9），nh 与 jia 只差 m 取不到那一支，nh 上 0 坏。没看到抬 F 那一串的扣住位被提早放开。

## 三、窄得多的盘上，扣住让再挂在预演里被拒的格子

不注读错时 jia 在单元区 136–328 槽上 12284 段再挂，被拒 0：

```
[r2-narrow-jia] starts=222 start_failed=30 remounts_ok=8724 remount_err=0
[r2-narrower-jia] starts=95 start_failed=13 remounts_ok=3560 remount_err=0
```

更窄（136–160 槽）那一组里起点自己（起点那次可写挂载、意图之后的批）先撞空间，甲那一扣没有在再挂这一步多拒一格。注读错时 jia 也是 0（m 取不到就整截放开，见第一节）；被拒全在 nh 上。

## 四、卸载半途崩溃之后连续两次再挂都在暖机落满之前崩

`TWICE=1`：第一次再挂写流里每个落在暖机写完之前的前缀刀，在那份镜像上再可写挂载一次，第二次的写流每个前缀刀再丢盘 0、丢盘 1 判。4 个起点（窄盘 `264:6:4:0`、`200:1:4:2`、`232:0:4:3`，普通盘 `240:0:1:1`），jia 两次再挂之后的判定原样（各起点 lost=dev0 那一行）：

```
[twice-tw-jia-200:1:4:2] sum twice j=0 lost=dev0 read=same count=70428
[twice-tw-jia-232:0:4:3] sum twice j=0 lost=dev0 read=same count=56811
[twice-tw-jia-264:6:4:0] sum twice j=0 lost=dev0 read=same count=56811
[twice-tw-jia-p240] sum twice j=0 lost=dev0 read=known count=60538
[twice-tw-jia-p240] sum twice j=0 lost=dev0 read=same count=10162
```

jia 0 坏；ga 同 4 个起点两次再挂之后坏 9634、25223、14539、9634 个判定。第二次再挂没有注读错；两次再挂之间叠第 1.3 节的读错没扫（第九节）。

## 五、第二次抽样（第 14 批代码轮「没打中」各格，照原来的问法、换一组取样点）

原型的 MODE=n1、MODE=qf 两段与第 14 批攻方那一份逐字相同（`probe/main.rs` 文件头写明底子），只换取样点：

| 族 | 第 14 批的点 | 这一次的点 |
|---|---|---|
| 窄盘 | 单元区 176–352 每 16 槽；覆盖写 0–10；单元 1, 2, 4, 8, 14；批 0, 1, 3, 7, 13 | 单元区 168, 200, 232, 264, 296, 328, 360；覆盖写 1, 4, 9；单元 3, 6, 11；批 2, 5 |
| 卸载—再挂循环（cycle） | 覆盖写 0, 2, 5, 9, 14；崩了再挂 0, 1；再挂后覆盖写 0, 2；卸载后覆盖写 0, 1, 2, 4, 8 | 覆盖写 1, 3, 7, 11；崩了再挂 0, 2；再挂后覆盖写 1, 3；卸载后覆盖写 3, 5, 6（四档盘宽同） |
| 窄盘循环（ncycle） | 单元区 176–336 每 32 槽；覆盖写 0, 2, 5, 8；单元 1, 4, 14；批 0, 1, 3 | 单元区 184, 216, 248, 280, 312；覆盖写 1, 3, 6；单元 2, 6；批 2；卸载后覆盖写 0（窄盘族第一个文件已删，大于 0 的起点全起不来） |
| 被抛弃的根（abandon） | 覆盖写 1, 2, 3, 5, 8；之后覆盖写 0, 1, 2, 4 | 覆盖写 4, 6, 7；之后覆盖写 3, 5（四档盘宽同） |
| Q-F ①b | 单元区 176–336 每 32 槽；覆盖写 0, 3；单元 1, 4, 14；批 0, 1, 3；时间 1788100000, 0, u64 最大 | 单元区 184, 216, 248, 280, 312；覆盖写 1, 4；单元 2, 6, 11；批 1, 2；时间 1700000000, 1, 1788000001（三个都早于这条记录上一版的时间） |

结果（`out/summary.txt` 原样）：

```
[n1-s2n-jia] starts=93 start_failed=33 unmount_raised=93 unmount_err=0 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0
[n1-s2n-imm] starts=93 start_failed=33 unmount_raised=93 unmount_err=0 starts_with_intersection=12 intersection_slot_lines_sum=19 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0
[n1-s2nc-jia] starts=30 start_failed=0 unmount_raised=30 unmount_err=0 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0
[n1-s2c-jia] starts=177 start_failed=15 unmount_raised=177 unmount_err=0 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0
[n1-s2a-jia] starts=22 start_failed=2 unmount_raised=20 unmount_err=2 starts_with_intersection=0 intersection_slot_lines_sum=0 start_lost_pairs_with_old_root_read_failure=0 such_cuts=0
[abandon-chosen-jia] cut_verdicts=2799 chosen_the_abandoned_root=0
[qf-jia] 180 qf_lines; identical=true wrong_times_nonzero=false=180 
```

- **GA 卸载那一串**：jia 四族 322 个起点读法三交集 0、整读坏 0、卸载被拒 0（abandon 族那 2 次被拒是 4 GiB `k=4 k2=3`、`k=4 k2=5` 报 `AccountOfAnAbandonedRoot`，即 C694 那一形，imm 同样 2 次，与 GA 无关）。**分辨力**：imm 在窄盘新点上 12 个起点有读法三交集、整读坏 0；在 cycle、ncycle、abandon 三族新点上 imm 也是 0（`[n1-s2nc-imm]`、`[n1-s2c-imm]`、`[n1-s2a-imm]` 与 jia 同行）。这三族的新点分不出卸载那一格，它们的「没打中」不能拿来支撑 GA；窄盘族的新点只在读法三上分辨，结局读法上不分辨。
- **单盘恢复择中被抛弃的根**：abandon 族新点 2799 个判定，择中 0 次。造法与第 14 批同（一次挂载期间把一条根与它那次发布的单元读回全 0），那次挂载的暖机在每块盘上落了新实例的根，卸载时各盘最新的根都不是被抛弃的；换造法（被抛弃之后暖机落满之前再崩）没做。
- **Q-F ①b**：180 份镜像各自续做两次逐字节相同，续做里还带这条记录的 696 步写入时间都等于意图那次给的值。这 180 份给的时间都早于这条记录上一版的时间（上一版 1788000002 或 1788000005，日志 `time_before`），与第 14 批那组（有晚于上一版的）方向不同。

## 六、N4、N5

**N4（卸载那一格「某块盘没有有效根整截扣」）**：在起点的会话上卸载，卸载期间一块盘的根环全部槽读坏（盘 0、盘 1 各一遍，读回全 0 与报读错各一遍），窄盘与普通盘两族全部起点，1272 次全部报 `NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot …)`：

```
[unmount-hide]  1272 Err NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot 
```

这次挂载的暖机在每块盘上写过根，那几槽是「这个进程知道住着根的槽」，读坏就重读、仍坏就拒，走不到 `newest_valid_root_txg_on_every_device` 交回 None 那一支。只读坏一部分槽时，知道住着根的那一槽要么读得出、要么拒，也到不了 None（推的）。所以卸载那一支从入口走不到，与实现员报告一致；改不改成断言由主 agent 判。**重建分配器那一处的 None 支从入口走得到，而且改结局**（第 1.3 节），那一支不该照卸载这一支的结论处理。

**N5（被抛弃根的过滤）**：没找到滤与不滤改结局的历史。找过：abandon 族（卸载一侧，见第五节）；形 R 各族（再挂一侧，这些历史里重建时没有被抛弃的根）。推的论证：重建时若某块盘上最新的根是被抛弃的根 R_a，不滤会让 m 取到 R_a、当场多放开一截；剩下那块盘单独恢复时它不知道 R_a 被抛弃，择的就是 R_a，而 R_a 引用的槽影子账整个隔离（`isolate_slots_referenced_only_by_abandoned_roots` 对被抛弃的根不按 F、不按窗口滤），多放开的那一截只被那块盘上更旧的有效根引用，恢复择不到它们。没造出「被抛弃的根是某块盘上最新的根、又碰上卸载半途崩溃」的历史去量。

## 七、N7：一种错实现两条新用例照样过

错实现：意图那次的写入时间取 max(调用方给的, 这条记录上一版的)，即时间只进不退（`patches/n7-clamp.patch` 第 8 行「write_time_seconds: write_time_seconds.max(record.write_time_seconds), // N7 错实现（攻方副本）：时间只进不退」）。条款要的是调用方给的时间：「写入时间取调用方给的 `write_time_seconds`（用户 2026-10-03 定 Q-F「乙 意图带时间、三格同改」」（`/tmp/claude-1000/m3-batch16-code-r1/tree/crates/singlefs-core/src/transaction.rs:5842`）。两条新用例给的时间都是上一版加 600，比上一版晚，max 之后照样是给的那个。

副本上编跑 5 个 harness 测试目标（经内存包装），`out/tests-n7clamp.log` 原样：

```
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.06s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 41.34s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.02s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.19s
```

（依次是 `mount_transparency_judged_field_by_field_after_a_writable_mount`、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`、`unlink_in_histories_judged_by_the_model`、`unlink_mount_continuation_and_the_emptied_version`、`unlink_single_file_publishes`，71 条全过。）原型的 Q-F 那一刀给早于上一版的时间，在同一个错实现上抓到：

```
QFTALLY identical=true wrong_times_nonzero=true count=2
```

改法（零轮，推的，没在副本上写成用例跑过）：两条新用例各加一形给的时间早于这条记录上一版的时间（例如上一版减 600），会话那一条判意图与两批三格都等于给的值，续做那一条判各批照抄意图写下的（较早的）值。

## 八、取样、分批与估时

- 盘：全部用内存稀疏盘（`SparseBlockDevice`），没有在磁盘上建镜像文件。每个起点在内存里建一次，卸载与再挂都在 `MemoryPool` 的拷贝上录。
- **崩溃状态的取法，这一条偏离了定义的要求，照实写**：大扫（第一至四节的前缀刀判定）每段历史只取写流的每个前缀刀（层 0 枚举域的一个子集），没有对每段历史都展开层 0 枚举域；层 0 枚举域（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开、段内整写子集逐个枚举，每个状态池级 checker 照跑）只跑在 5 段历史上（第 1.3 节那张表）。缩前：窄盘族 8724 段再挂历史每段约 2×10⁴ 个状态，全展开约 1.8×10⁸ 个（推的，按 h2 那一段的 20511 乘），普通盘 j = 2 的历史每段 2.1×10⁶ 到 5.4×10⁸ 个；缩后：层 0 共跑 21 次、每次 20511 个状态（草稿里 11 次、空目录复跑 10 次），合计 430731 个，每次都不超过约 10⁶，合计不超过约 10⁷。超过约 10⁶ 而没跑的：4 GiB `4:1:1:1` n = 1 的 j = 0（4259856）与 j = 2（541130780）、384 槽 `384:2:1:0` j = 2（33579042）、240 槽 `240:4:0:0` j = 2（2121762）、`240:0:1:1` 与 `384:1:1:1` 的 j = 1（16797718）、`256:1:2:1` j = 1（1314838）、`240:4:0:0` j = 1（1073180）。这些交主 agent 判要不要另起。
- 估时：开跑之前只用单个起点试过（窄盘一个起点约 1.6–10 秒、4 GiB 一个起点约 42 秒、一段层 0 约 70–130 秒），没有先估全量；整套一遍在空目录复跑里逐阶段量的挂钟合计约 72 分钟（本机同时有别的会话的 cargo），超过 40 分钟这一条我没有在开跑前照办。分批：每批的件数按单起点试跑定，不设限时：窄盘每批 14 件（每件 9 个起点），普通盘 4 GiB 每批 8 件或 4 件（每件 2 个起点），其余盘宽每批 12 件；`rerun.sh` 的阶段就是这些批，复跑时各批挂钟 53–220 秒。草稿阶段有 4 条命令跑过 240 秒被转到后台，等完成通知之后照常读结果，没有中途停。
- 线程：每件原型 `ATTACK_THREADS=1`、同批不超过 14 件；层 0 那几段 14 线程；编译 15 线程。
- 内存包装的退出码（都在编译原型时）：`bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 15 cargo build --release --offline -p attack-b16-probe` 退 251 一次（上限超过峰值表两倍、一行没跑）；同一条换成 `auto … capped.sh 7` 退 251 两次（峰值表里没有这个键）；`auto … capped.sh 15`（ga 臂）退 250 一次（撞了 1752M 上限被杀）；`4G … capped.sh 15`（jia、ga、nh 依次）退 251 三次（4G 超过峰值表 1162M 的两倍）。这几次的输出都没用，改成写明理由的 4G 上限重编（`scripts/build.sh`）。

## 九、没打中的形状

| 形状 | 取样 | 结局 |
|---|---|---|
| 形 R 不注读错，卸载刀 × 再挂刀 × 丢盘 | 第 1.1 节表，317 + 96 个起点，20948 段再挂历史 | jia 0 坏、0 被拒 |
| 再挂做成之后覆盖写 2 次再崩、丢盘 | 普通盘 96 个起点 | 0 坏（只前缀刀） |
| 同一次挂载里准入抬 F 之后的每一刀 | nh 读坏族 3158 段 | 0 坏 |
| 连续两次再挂都在暖机落满之前崩 | 4 个起点，每个起点 33 段第一次再挂 × 第一次再挂的每一刀 × 第二次再挂的每一刀 | jia 0 坏 |
| GA 卸载那一串第二次抽样 | 第五节表，322 个起点 | 0（三族新点不分辨 imm） |
| 单盘恢复择中被抛弃的根 | abandon 族 22 个起点、2799 个判定 | 0 |
| Q-F ①b | 180 份镜像 | 逐字节相同、时间全对 |
| 卸载那一格 None 支从入口走 | 1272 次卸载，一块盘根环整段读坏 | 全在重读那一步被拒 |

## 十、这条腿自己的限度

- 大扫只判前缀刀，层 0 只跑 5 段历史（第八节）。打中那一格三段层 0 都中，没打中的各格只有前缀刀的证据（加 h2、`264:6:4:0`、`240:0:1:1` 三段 jia 不注读错的层 0 为 0）。
- 读错模型只有一种：一块盘的根环在那次挂载里每一槽都读坏、挂载交回之后读得出。只读坏最新那一槽（m 会取到更旧的根、只放得更少）没跑，是推的。读错持续到挂载之后、或在两次再挂之间换盘读坏，都没跑。
- 第 1.3 节「打中」的判法是我的结局 oracle：剩下那块盘只读挂载、整读每个活文件，与卸载之前、再挂之后各版两盘整读的哈希比；`BAD_content` 一次没出现，坏的都是读不出。两盘都在时池级 checker 0 违例（C683），不分辨。
- FX 原型这一轮没跑；它与甲在 None 支上同取原门槛是读补丁推的。
- 第二次抽样在 cycle、ncycle、abandon 三族的新点上连 imm 都不分辨，这三族这一次的「没打中」分辨力不够；窄盘族只在读法三上分辨 imm。
- N5 没造出被抛弃的根正好是某块盘上最新的根、又碰上卸载半途崩溃的历史，只有推的论证。
- nh 没在不注读错的各族上跑；它对那些历史无影响是推的。

## 十一、没做什么

- K1、K3 与正推那几格不判；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何重型测试与 checker 档包的测试二进制；层 0 枚举函数只在自己的原型 crate 里调，流是原型自己录的。
- 没改仓里任何文件，只写了本报告与模型目录。
- 草稿目录 `/tmp/claude-1000/m3-batch16-code-r1-opus/`：交回之前删掉了自己建的仓副本与编译目录（`W/` 与 `R/` 下的 jia、ga、nh、imm、n7clamp 五份拷贝与各自的 `target-*`，以及 `model0/`）；留着两份日志目录 `W/out/`（草稿阶段原始输出）与 `R/out/`（空目录复跑的原始输出），主 agent 要逐字核可以去读。
- 引 D16 写的是仓里现行文件的行号（第 38、42 行）。现行文件 sha256 `de5689ef1f9a96b1b40b2109e14e7553c6205cee938fd2eb05dc92a74cafefff`，与开工快照 `research/prompts/m3-batch16-code-r1-snapshot/kb-sha256.txt` 记的 `f9aabb90…` 不同；引到的两段在背景材料的 D16 抄录里逐字都有。
