# m3-w3-r1 攻方腿（Opus）报告：K1、K2、K3（2026-10-08）

轮名 m3-w3-r1；背景材料 `research/prompts/_m3-w3-r1-background.md`（正文 `research/prompts/_m3-w3-r1-body.md`）；前几轮判决：无。
冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`（只读）；草稿 `~/.local/share/singlefs/work/m3-w3-r1-opus/`；模型 `research/prompts/m3-w3-r1-opus-model/`。
副本上量出的数全部是副本装置的数，不是入库装置上的数；我提的改法都只在我的模型上量过、被攻过零轮。

## 复跑命令与模型文件

在仓根，`W` 取一个空目录；每段一条前台命令（各在 4 分钟以内），编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`（编译 6G、跑 4G），线程 6：

```
M=research/prompts/m3-w3-r1-opus-model; W=<空目录>
for p in build quick device-k1a device-k1b model-1 model-2 model-3 compare; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
# 可选的长段（副本装置全量，合计约 35 分钟）：glong-v1-a glong-v1-b f2-v1 glong-p-1..4 f2-p-1..4 crafted-v1-1..3，产物对 $M/results-long/ 比
```

交回前在空目录 `~/.local/share/singlefs/work/m3-w3-r1-opus/rerun2/` 从头跑通一次，最后一段 `compare` 的输出末行 `compared=11 status=0`；`expected/` 里的 11 份就是那一次的输出。`compare.sh` 去掉 `E7TIMING` 行与 `elapsed_ns` 字段、排序后比，不读 `SINGLEFS_REPO`。

模型目录 `research/prompts/m3-w3-r1-opus-model/` 每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
540312c24c3243205edc127cbd3e1bc9d4a8ff3a03fef1e4294d84ef71363361  ./compare.sh
18a788a0b50e0d1be46ff9a947cc3df11e2a7380f7fb22bba95b4f8126ae5f7c  ./expected/device-crafted-V1.out
a136f6cd3651ba3c85095a3a040897ce5ff3d0a47fe74a502d5215e8d94bca89  ./expected/device-f2-V1-gone.out
5f47d578b313b9b3902aa65f165cd7fa43ac82c2aae265593f69a65d1c0e4aaa  ./expected/device-g-long-V1-index24.out
c481b25d2e2d4b4af114d319daf41f51f0651a8ddad98dec99ebdce3569ba89c  ./expected/model-main-0.out
c54803a3a882948886f8d8cc877ee2d889da96d1219952661155427c926c8b95  ./expected/model-main-1.out
ce6824488c191c4506e9f3a3e7a03889eaeba81cb0d3c88e5ceba727589b894d  ./expected/model-main-2.out
c47f33946daf4252ca1d6234ea916f74aa51d91ca146bdaa9807af8f8c3e2e2c  ./expected/model-main-3.out
efd33d9ee07f5e3c9278ec3213e398da0ec79b6c22a67e509aa95f1ece69ecd1  ./expected/model-main-4.out
d15706b7de95ae253dfdeb793a287f84a026536097f7827ecd380a904a555467  ./expected/model-main-5.out
fcd408362c120eb19d008652b582e7c83331f2feb155d04a4e2d340eaa09adb6  ./expected/model-mutation.out
b4a76fba52dd156c3773f939f972edba57f3533b2441f31103ac16c777ca3d29  ./expected/model-ring6.out
59e4d169797a97042d6c4c72213031fcfd85a2d42255f38a8e56035d6163b866  ./make_device.py
4d94d5479391112aaf09b10ba72edaeed2acf66f780b8542ab2b4c4ad88cd041  ./rerun.sh
f4a56198751d25bbd63512bfb04c0d3d235a8938b97636427bb0e7b10d99e44b  ./results-long/full-crafted-V1.summary
6b1405b8e134f02688076fbf92b473e9d208bee4d344cbda2a07947aebaf1838  ./results-long/full-f2-P.out
c2d58704eea749792a2bbad7da63984350a477a177f86807dc8327b52d295cd3  ./results-long/full-f2-V1.out
01abee097df25a3a8713d2c159925dd5cb6218031047627aaf2742093d8251cb  ./results-long/full-g-long-P.out
7e09dd6198aac74b32ff22d5e2d39b4717d9fa1cca74f406d44f8aff3fcbfebd  ./results-long/full-g-long-V1.out
5c497c6fd9dae392f37b851678c86b535f8388e8e4060b51bcc9a25f41ad0bef  ./sum.awk
18626b3ca714e0340b83bc1b08964ea6b6954662259a9f2de625f975ea727c0c  ./timeline-model/Cargo.lock
e713572288d0a07a23e16cc13eca2baf146a1f306773bdb181d985d7d59df088  ./timeline-model/Cargo.toml
80b06c479a638632051347d6cca5efe5cce6609230ec671497c7f118cd5709a9  ./timeline-model/src/main.rs
c9fc7f0c291f9021e59810d3e7b13c375bd5b819696abfa2337a1ebc6b95339f  ./w3_module.rs
```

## 各格判定一览

| 候选（读法，定义见第一节） | K1：V1 那两次与看不见的一形 | K2：误报与历史在回退、切换、行回收时丢 | K3：放过的变异 |
|---|---|---|---|
| 候选一 C1（行记所选那一版的水位） | **打中**：op92、op95 都不红（副本装置）；模型最短 `[create,rollback1,create]` | 误报没打中（模型 P 臂 21800832 张合法镜像 0）；**打中漏报**：清扫删行丢 M，`[create,create,mount,rollback3,create,crash-hide,create]` | **打中**：M 不写（写 0）不红；C1s 只在那条根还在环里时红 |
| 候选一 C1m（切换时 M 取内存里的） | 同 C1 | **打中误报**：`[switch-error]` | 同 C1 |
| 候选二 C2a（保留历代，带逐条比那一句） | 没打中：两次都红；模型 V1 670350 次 0 漏（扫了：环 4/6、阈值 0/从不、长 7 全部用户步） | 没打中：0 误报（同左的扫法，含 83398 次合法跨抛弃重发） | **打中**：A（回退那次从 R_old 带）、B（只照抄这一版）加严到 C2as 仍不红；环内交叉核 C2ax 在旁证挤出环后漏 6350 次；V2 一形 1523216 次全漏 |
| 候选二只写「水位 ≥ 历代最大」（= C2c） | **打中**：op95 不红 | 没打中 | **打中**：C（不写）不红；V2 全漏 |
| 候选二 C2b（每个实例最后一代） | **打中**：op92、op95 都不红 | 没打中 | **打中**：A、B、C 都不红；V2 全漏 |
| 候选三 C3r（根下只追加 (号, 出生代)） | 没打中：两次都红；模型 V1、V2 0 漏 | 没打中：0 误报 | **打中**：A、B 加严到 C3rs 不红；交叉核 C3rx 挤出环后漏 8778 次，`[create,delete,rollback2,rollback3,create]` |
| 候选三 C3f（根外定长区，按实例表判） | 没打中 | **打中误报**：清扫删行之后，`[crash-mid-create,create,create,mount]` | 没攻（定长区不随回退走） |
| 候选三 C3fb（定长区只有 (号区间, 出生代)） | 没打中 | **打中误报**：`[crash-mid-create]` | 没攻 |
| 候选四 C4（谱系带回退前水位，我加的） | **打中**：op95 不红 | 没打中 | **打中**：A、C 不红；V2 全漏 |
| 候选四 C2d（峰值连同代，我加的） | 两次都红；**打中**越过旧峰值之后那一形（副本装置定制族第 692 段 op7） | 没打中 | **打中**：A、B、C 不红；V2 全漏 |
| 候选四 C2ay（历代加谱系核对，我加的，零轮） | 没打中 | 没打中 | 没打中 A、B、C；**打中** V2（全漏） |

一句话：只记水位的历史（候选一、候选二的「最后一代」与「最大值」、C4）看不见「重发的正好是峰值减一」与 V2；看得见这两种的只有按号记出生代的 C3r，而 C3r 自己挡不住「回退那次从 R_old 带」，要再拿一样随现行那一版走的东西（谱系记录）核它（推的）。

## 一、我攻的是哪些读法（候选的定义）

正文第三节 K1 的三个候选，每个按「谁写、写什么、判什么」拆成几种读法分别攻；判只读一个镜像：最新那条根、它的实例表与它下面的记录、环里读得出的根、盘上的定长区。
这些读法都只在我的两套装置里实现成「影子记录」，不是盘上格式；判词是我按候选最自然的读法写的，被攻过零轮。

| 名字 | 候选 | 写什么、谁写、什么时候写 | 单镜像上怎么判红 |
|---|---|---|---|
| C1 | 候选一 | 实例表行预留字节记 M = 这次恢复（或切换）所选那一版的水位；只在恢复与切换写行时写（D18 已定项 11「行怎么写」） | 最新那条根的水位 < 它的实例表里某一行的 M |
| C1m | 候选一 | 同 C1，但切换时 M 取内存里的水位（含没发布成的那次建） | 同上 |
| C1s | 候选一 | 同 C1，判词加一句：行 (i, T) 的根还在环里时，M 必须等于那条根的水位 | 同 C1，或上面那句不等 |
| C2a | 候选二「保留历代」 | 记账里每一代一行 (代, 水位)，随根 COW 带下去；回退那次从 cur 带 | 水位 < 某一行；或树里一条 (号 n, 出生代 b) 的 n 小于代 < b 的行里最大的水位 |
| C2as / C2ax | 候选二 | C2a 的判词加「最新那一代有行且等于水位」；C2ax 再加「环里读得出、按实例表不被抛弃的每一条根，它的 (代, 水位) 都在行里」 | 见左 |
| C2b | 候选二「每个实例最后一代」 | 每个实例一行，值是这个实例在这条线上最后一代的水位（后写覆盖） | 水位 < 某一行 |
| C2c / C2cs | 候选二的退化形 | 一条峰值统计量，取最大值合并，回退那次从 cur 带 | 水位 < 峰值（C2cs：峰值 ≠ 水位） |
| C2d | 候选四（我加的） | (峰值, 第一次到达峰值的代) 两个数 | 水位 < 峰值；或树里一条号 < 峰值而出生代 > 那个代 |
| C3r / C3rs / C3rx | 候选三，挂在根下 | 只追加的 (号, 出生代)，随根 COW；回退那次从 cur 带 | 同号两个出生代，或水位 ≤ 记录里最大的号；C3rs 加「最新那版树里每条都在记录里」；C3rx 再加「环里不被抛弃的每条根的树都在记录里」 |
| C3f | 候选三，盘上定长区 | (实例, txg, 号, 出生代)，在那次发布的根之前单独持久；按最新那条根的实例表判这一条发布了没有（无行 ⇒ 已发布） | 已发布的条目里同号两个出生代，或号 ≥ 水位 |
| C3fe | 候选三，定长区 | 同 C3f，实例表清扫删一行时，把这个实例 txg > T_pub 的条目一起抹掉 | 同上 |
| C3fb | 候选三「(号区间, 出生代)」 | 定长区里只有 (号, 出生代)，没有实例；出生代 ≤ 最新那条根的 txg 的都算 | 同上 |
| C4 | 候选四（我加的） | 谱系记录 (T_rb, T_old) 多带 8 字节：回退前 cur 的水位（谱系树随现行那一版） | 水位 < 谱系里带的最大值 |

两套装置：

- 副本装置：E175 装置源码原样加一段影子记录（`research/prompts/m3-w3-r1-opus-model/make_device.py` 从 `device-source.rs` 定点生成，`w3_module.rs` 是加的那一段），放进冻结树副本的 `crates/singlefs-harness/tests/w3_attack_candidates.rs`，P 臂原样、V1 臂施加 E175 的 `patch-V1.diff`。影子按每条根的父版本算：父版本 = 之前见过的根里 txg 比它小、按最新那条根的实例表不被抛弃的根中 (txg, 实例) 最大的那条（会话里与回退那次都是 cur，挂载那次是所选根）。C1 的行用盘上真写出来的实例表行，M 取行第一次出现时那条根的水位。原型输出的 E175 字段与 E175 产物逐行相同（第二节第 1 条）。
- 抽象时间线模型（`research/prompts/m3-w3-r1-opus-model/timeline-model/`，不依赖 `crates/`）：只照条文走发号、删、回退（P 取 max(cur, 环)，V1 取目标的）、抬 F、空发布、挂载、崩溃恢复抛弃最新那条根（系统配置没见证它那一形）、建到一半崩溃、切换重做、切换报错、实例表清扫；起点固定在新池新建文件之后，后面每一步都由用户定、全部穷举（长 7，12 个字母）。环 4 槽与 6 槽、清扫阈值 0 与「从不」各跑。

## 二、K1：E175 V1 那两次单镜像判不判得红；各候选看不见的一形

### 1. 副本装置先对上 E175

原型输出去掉加的 `w3_` 字段之后，事件行与结果行和 E175 产物逐行相同（排序、去掉 `elapsed_ns` 后取 sha256 前 16 位；在 `research/prompts/m3-w3-r1-opus-model/results-long/` 里跑，`E` 是 E175 产物目录）：

```
for f in g-long-V1 f2-V1 g-long-P f2-P; do a=$(grep '^E7RESULT name=event' $E/$f.out | sort | sha256sum | cut -c1-16); b=$(grep '^E7RESULT name=event' full-$f.out | sed -E 's/ (w3_c1=|w3=not_judged).*$//' | sort | sha256sum | cut -c1-16); c=$(grep '^E7RESULT name=history' $E/$f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); d=$(grep '^E7RESULT name=history' full-$f.out | sed -E 's/ elapsed_ns=[0-9]*//; s/ w3_steps_judged=.* operations=/ operations=/' | sort | sha256sum | cut -c1-16); echo "$f events e175=$a prototype=$b history e175=$c prototype=$d"; done
g-long-V1 events e175=83c2c5a46b6caeb7 prototype=83c2c5a46b6caeb7 history e175=1a7f8bb4cb36796d prototype=1a7f8bb4cb36796d
f2-V1 events e175=ba3ebf581c8163ba prototype=ba3ebf581c8163ba history e175=953e7036ebef7f67 prototype=953e7036ebef7f67
g-long-P events e175=e3b0c44298fc1c14 prototype=e3b0c44298fc1c14 history e175=fbb3317a8d91bc98 prototype=fbb3317a8d91bc98
f2-P events e175=e3b0c44298fc1c14 prototype=e3b0c44298fc1c14 history e175=1df2ef67f47bf988 prototype=1df2ef67f47bf988
```

### 2. 那两次（副本装置，G-long V1 第 24 段）

`research/prompts/m3-w3-r1-opus-model/results-long/full-g-long-V1.out:389`：「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op92 inode=7 new_birth=77 older_births=[51] q9=Gone i9_29_at_step=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=1 w3_c2d=1 w3_c3=1 w3_c4=1」

`research/prompts/m3-w3-r1-opus-model/results-long/full-g-long-V1.out:390`：「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op95 inode=8 new_birth=78 older_births=[52] q9=Gone i9_29_at_step=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=1 w3_c3=1 w3_c4=0」

| 候选（读法） | op92 | op95 | 判定 |
|---|---|---|---|
| C1 实例表行 M | 不红 | 不红 | **打中**：两次都漏 |
| C2b 每个实例最后一代 | 不红 | 不红 | **打中**：两次都漏 |
| C2c 峰值 | 红 | 不红 | **打中**：op95 漏 |
| C4 谱系带回退前水位 | 红 | 不红 | **打中**：op95 漏 |
| C2a 历代行（带逐条记录那一句） | 红 | 红 | 没打中（这两次）；只留「水位 ≥ 历代最大」那一半时等于 C2c，op95 漏 |
| C2d 峰值连同代 | 红 | 红 | 没打中（这两次）；别的形打中，见第 5 条 |
| C3 发号记录 (号, 出生代) | 红 | 红 | 没打中（这两次） |

G-long V1 全部 32 段、F2 V1 全部 64 段的合计（`awk -f ../sum.awk <文件>`，原样）：

```
full-g-long-V1 histories=32 w3_gone_events=6 w3_gone_red_c1=0 w3_gone_red_c2a=6 w3_gone_red_c2b=0 w3_gone_red_c2c=1 w3_gone_red_c2d=6 w3_gone_red_c3=6 w3_gone_red_c4=1 w3_parent_missing=0 w3_q1_events=476 w3_q1_red_c1=70 w3_q1_red_c2a=476 w3_q1_red_c2b=70 w3_q1_red_c2c=233 w3_q1_red_c2d=476 w3_q1_red_c3=476 w3_q1_red_c4=233 w3_red_steps_c1=574 w3_red_steps_c2a=4070 w3_red_steps_c2b=574 w3_red_steps_c2c=1943 w3_red_steps_c2d=2659 w3_red_steps_c3=4095 w3_red_steps_c4=1943 w3_row_root_unknown=0 w3_steps_judged=4832
full-f2-V1 histories=64 w3_gone_events=5 w3_gone_red_c1=0 w3_gone_red_c2a=5 w3_gone_red_c2b=0 w3_gone_red_c2c=2 w3_gone_red_c2d=5 w3_gone_red_c3=5 w3_gone_red_c4=2 w3_parent_missing=0 w3_q1_events=336 w3_q1_red_c1=32 w3_q1_red_c2a=336 w3_q1_red_c2b=32 w3_q1_red_c2c=164 w3_q1_red_c2d=336 w3_q1_red_c3=336 w3_q1_red_c4=164 w3_red_steps_c1=334 w3_red_steps_c2a=2619 w3_red_steps_c2b=334 w3_red_steps_c2c=1476 w3_red_steps_c2d=1913 w3_red_steps_c3=2645 w3_red_steps_c4=1476 w3_row_root_unknown=0 w3_steps_judged=3904
```

读法：同一条时间线上的重发 G-long 476 次、F2 336 次，C1、C2b 只红 70、32 次，C2c、C4 只红 233、164 次；I-9.29 看不见的 Gone 那 6 + 5 次，C1、C2b 一次不红，C2c、C4 各红 1、2 次。C2a、C2d、C3 每一次都红。`w3_parent_missing=0`、`w3_row_root_unknown=0`：影子的父版本与行的 M 都找得到，没有按 0 顶的。

### 3. 为什么漏：两种机理

**机理甲（C1、C2b）：高水位在同一个实例里到达、又在同一个实例里退回去。** G-long V1 第 24 段，实例 8 在 op69、op70 发 7、8（水位到 9），op71 那次回退按 V1 退回 7，op77 挂载时新实例给实例 8 写的行是「所选那一版的水位」7。逐步打印（`research/prompts/m3-w3-r1-opus-model/expected/device-g-long-V1-index24.out` 第 86、87、93 行，整行原样）：

```
E7W3TRACE family=G-long arm=V1 index=24 position=op70 op=create newest=8:52 wm=9 peak=9 lineage=7 per_instance={1:2,2:4,3:5,4:7,5:6,6:7,7:7,8:9} rows=[1:5:M2,2:9:M4,3:17:M5,4:23:M7,5:30:M6,6:37:M7,7:43:M7] births=[2@8,3@9,4@13,5@22,6@33,7@51,8@52] ring=[5:29:F5:wm7,5:30:F5:wm6,6:31:F5:wm6,6:32:F5:wm6,6:33:F5:wm7,6:34:F5:wm6,6:35:F5:wm6,6:36:F5:wm7,6:37:F5:wm7,7:38:F5:wm7,7:39:F5:wm7,7:40:F5:wm7,7:41:F15:wm7,7:42:F15:wm7,7:43:F15:wm7,8:44:F15:wm7,8:45:F15:wm7,8:46:F15:wm7,8:47:F15:wm7,8:48:F15:wm7,8:49:F15:wm7,8:50:F15:wm7,8:51:F15:wm8,8:52:F15:wm9] i9_29=violated i9_6=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=0 w3_c3=1 w3_c4=0
E7W3TRACE family=G-long arm=V1 index=24 position=op71 op=rollback(RingRoot{index_from_newest:3}) newest=8:53 wm=7 peak=9 lineage=9 per_instance={1:2,2:4,3:5,4:7,5:6,6:7,7:7,8:7} rows=[1:5:M2,2:9:M4,3:17:M5,4:23:M7,5:30:M6,6:37:M7,7:43:M7] births=[2@8,3@9,4@13,5@22,6@33] ring=[5:30:F5:wm6,6:31:F5:wm6,6:32:F5:wm6,6:33:F5:wm7,6:34:F5:wm6,6:35:F5:wm6,6:36:F5:wm7,6:37:F5:wm7,7:38:F5:wm7,7:39:F5:wm7,7:40:F5:wm7,7:41:F15:wm7,7:42:F15:wm7,7:43:F15:wm7,8:44:F15:wm7,8:45:F15:wm7,8:46:F15:wm7,8:47:F15:wm7,8:48:F15:wm7,8:49:F15:wm7,8:50:F15:wm7,8:51:F15:wm8,8:52:F15:wm9,8:53:F15:wm7] i9_29=violated i9_6=violated w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=1 w3_c2d=1 w3_c3=1 w3_c4=1
E7W3TRACE family=G-long arm=V1 index=24 position=op77 op=mount newest=9:58 wm=7 peak=9 lineage=9 per_instance={1:2,2:4,3:5,4:7,5:6,6:7,7:7,8:7,9:7} rows=[1:5:M2,2:9:M4,3:17:M5,4:23:M7,5:30:M6,6:37:M7,7:43:M7,8:55:M7] births=[2@8,3@9,4@13,5@22,6@33] ring=[6:35:F5:wm6,6:36:F5:wm7,6:37:F5:wm7,7:38:F5:wm7,7:39:F5:wm7,7:40:F5:wm7,7:41:F15:wm7,7:42:F15:wm7,7:43:F15:wm7,8:44:F15:wm7,8:45:F15:wm7,8:46:F15:wm7,8:47:F15:wm7,8:48:F15:wm7,8:49:F15:wm7,8:50:F15:wm7,8:51:F15:wm8,8:52:F15:wm9,8:53:F15:wm7,8:54:F16:wm7,8:55:F16:wm7,9:56:F16:wm7,9:57:F16:wm7,9:58:F16:wm7] i9_29=violated i9_6=violated w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=1 w3_c2d=1 w3_c3=1 w3_c4=1
```

行只在恢复与切换时写（「**行怎么写**」那一条在 `.claude/kb/decisions/18-块里携带什么信息.md:310`），当前实例没有行；下一次挂载写的是所选那一版的水位，已经是退回去之后的值。所以凡是「同一个实例里发、同一个实例里退」的，C1 一次都看不见；C2b 的「每个实例最后一代」被同一个实例后面的那一代覆盖，同形。

**机理乙（C2c、C4，以及只留「水位 ≥ 历代最大」那一半的 C2a）：重发的正好是峰值减一那个号，发完水位又回到峰值。** op95 重发 8 之后水位 9 = 峰值 9，镜像上「水位 < 历史」不成立。这一类候选只看得见「水位低于历史」，看不见「号被重发」；要看见 op95，得有按号（或按代）比的记录。第 92、95 两步（同一文件第 108、111 行）：

```
E7W3TRACE family=G-long arm=V1 index=24 position=op92 op=create newest=14:77 wm=8 peak=9 lineage=9 per_instance={1:2,2:4,3:5,4:7,5:6,6:7,7:7,8:7,9:7,10:7,11:7,12:7,13:7,14:8} rows=[1:5:M2,2:9:M4,3:17:M5,4:23:M7,5:30:M6,6:37:M7,7:43:M7,8:55:M7,9:62:M7,10:64:M7,11:67:M7,12:70:M7,13:74:M7] births=[2@8,3@9,4@13,5@22,6@33,7@77] ring=[8:54:F16:wm7,8:55:F16:wm7,9:56:F16:wm7,9:57:F16:wm7,9:58:F16:wm7,9:59:F16:wm7,9:60:F16:wm7,9:61:F16:wm7,9:62:F16:wm7,10:63:F16:wm7,10:64:F16:wm7,11:65:F16:wm7,11:66:F16:wm7,11:67:F16:wm7,12:68:F16:wm7,12:69:F16:wm7,12:70:F16:wm7,13:71:F16:wm7,13:72:F16:wm7,13:73:F16:wm7,13:74:F16:wm7,14:75:F16:wm7,14:76:F16:wm7,14:77:F16:wm8] i9_29=holds i9_6=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=1 w3_c2d=1 w3_c3=1 w3_c4=1
E7W3TRACE family=G-long arm=V1 index=24 position=op95 op=create newest=14:78 wm=9 peak=9 lineage=9 per_instance={1:2,2:4,3:5,4:7,5:6,6:7,7:7,8:7,9:7,10:7,11:7,12:7,13:7,14:9} rows=[1:5:M2,2:9:M4,3:17:M5,4:23:M7,5:30:M6,6:37:M7,7:43:M7,8:55:M7,9:62:M7,10:64:M7,11:67:M7,12:70:M7,13:74:M7] births=[2@8,3@9,4@13,5@22,6@33,7@77,8@78] ring=[8:55:F16:wm7,9:56:F16:wm7,9:57:F16:wm7,9:58:F16:wm7,9:59:F16:wm7,9:60:F16:wm7,9:61:F16:wm7,9:62:F16:wm7,10:63:F16:wm7,10:64:F16:wm7,11:65:F16:wm7,11:66:F16:wm7,11:67:F16:wm7,12:68:F16:wm7,12:69:F16:wm7,12:70:F16:wm7,13:71:F16:wm7,13:72:F16:wm7,13:73:F16:wm7,13:74:F16:wm7,14:75:F16:wm7,14:76:F16:wm7,14:77:F16:wm8,14:78:F16:wm9] i9_29=holds i9_6=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=1 w3_c3=1 w3_c4=0
```

### 4. 抽象模型里最短的反例（长 7 穷举，环 4 槽，V1 臂）

`research/prompts/m3-w3-r1-opus-model/expected/model-main-2.out` 第 1 行是这一组的规模：

```
W3MODEL arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 full_length_histories=9863012 images=10836269 legal_images=7914147 events=340776 events_where_I-9.29_holds=144904 residual_images=443952 reissues_across_abandoned_roots=47880
```

同一文件里各候选漏掉的次数（第 2、4、7、9、10、12、13、18 行，去掉了每行开头相同的那段配置）：

| 候选 | 漏掉的重发（共 340776） | 其中 I-9.29 判成立的（共 144904） | 后来的镜像里被重发的对象还活着却不红 |
|---|---|---|---|
| I-9.29（现状） | 144904 | 144904 | 372112 |
| C1 | 335560 | 143336 | 438352 |
| C2b | 335560 | 143336 | 438352 |
| C2c | 297712 | 129368 | 396848 |
| C4 | 297712 | 129368 | 396848 |
| C2d | 0 | 0 | 142136 |
| C2a | 0 | 0 | 0 |
| C3r | 0 | 0 | 0 |

最短反例（同一文件第 58、63、129、163、172、177 行的 `ops=`）：C1、C2b、C2c、C4 都漏 `[create,rollback1,create]`（V1：建 2 号、回退一步、再建又发 2 号，发完水位 3 = 峰值 3）；I-9.29 也判成立的那一形是 `[create,rollback1,raiseF,create]`（抬 F 把持有 2@4 的根挤出候选集）；C2d 漏 `[create,rollback1,create,create]` 的最后一步：越过旧峰值之后 (峰值, 代) 被新值覆盖，2@6 这个被重发的对象还活着，C2d 判成立。第 178–182 行是第 177 行那一例的逐步打印（整行原样）：

```
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 candidate=C4 kind=event_missed_where_I-9.29_holds step=start newest=1:3 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 candidate=C4 kind=event_missed_where_I-9.29_holds step=0 op=create newest=1:4 wm=3 F=0 tree=[1@3,2@4] rows=[] peak=3 peak_pair=3@4 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 candidate=C4 kind=event_missed_where_I-9.29_holds step=1 op=rollback1 newest=1:5 wm=2 F=0 tree=[1@3] rows=[] peak=3 peak_pair=3@4 lineage=3 c3r_log=[1@3,2@4] ring=[1:4,1:5,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[I-9.6,C2a,C2as,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 candidate=C4 kind=event_missed_where_I-9.29_holds step=2 op=raiseF newest=1:6 wm=2 F=5 tree=[1@3] rows=[] peak=3 peak_pair=3@4 lineage=3 c3r_log=[1@3,2@4] ring=[1:4,1:5,1:6,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[C2a,C2as,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 candidate=C4 kind=event_missed_where_I-9.29_holds step=3 op=create newest=1:7 wm=3 F=5 tree=[1@3,2@7] rows=[] peak=3 peak_pair=3@4 lineage=3 c3r_log=[1@3,2@4,2@7] ring=[1:4,1:5,1:6,1:7] fixed=[1:3:1@3,1:4:2@4,1:7:2@7] truth_reissue=1 truth_regressed=0 red=[C2a,C2as,C2d,C3r,C3rs,C3f,C3fe,C3fb,C2ax,C3rx,C2ay]
```

### 5. 副本装置上的同形（C2d 越过峰值之后看不见）

W3 定制族（E175 F3a 同形，中段把抬 F 的步数放开 1..12、加挂载）V1 第 692 段 `[unlink,create,create,rollback(RingRoot{index_from_newest:1}),raise(2),raise(8),create,create]`：op6 重发 3 号时 C2d 红，op7 越过旧峰值之后 3@11 还在、C2d 不红（`research/prompts/m3-w3-r1-opus-model/expected/device-crafted-V1.out` 第 10、18、19 行）：

```
E7RESULT name=event family=W3c arm=V1 index=692 kind=q1_same_timeline position=op6 inode=3 new_birth=11 older_births=[7] q9=Still i9_29_at_step=violated w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=1 w3_c3=1 w3_c4=0
E7W3TRACE family=W3c arm=V1 index=692 position=op6 op=create newest=1:11 wm=4 peak=4 lineage=4 per_instance={1:4} rows=[] births=[2@6,3@11] ring=[0:0:F0:wm-,1:1:F0:wm-,1:2:F0:wm-,1:3:F0:wm2,1:4:F0:wm2,1:5:F0:wm2,1:6:F0:wm3,1:7:F0:wm4,1:8:F0:wm3,1:9:F2:wm3,1:10:F2:wm3,1:11:F2:wm4] i9_29=violated i9_6=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=1 w3_c3=1 w3_c4=0
E7W3TRACE family=W3c arm=V1 index=692 position=op7 op=create newest=1:12 wm=5 peak=5 lineage=4 per_instance={1:5} rows=[] births=[2@6,3@11,4@12] ring=[0:0:F0:wm-,1:1:F0:wm-,1:2:F0:wm-,1:3:F0:wm2,1:4:F0:wm2,1:5:F0:wm2,1:6:F0:wm3,1:7:F0:wm4,1:8:F0:wm3,1:9:F2:wm3,1:10:F2:wm3,1:11:F2:wm4,1:12:F2:wm5] i9_29=violated i9_6=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=0 w3_c3=1 w3_c4=0
```

这一段 op6 那次重发持有旧出生代的根还在候选集里（`q9=Still`），I-9.29 判红；C2d 越过峰值之后不红是「后来的镜像」那一格，不是那两次。定制族全量 5220 段（V1）的合计在 `research/prompts/m3-w3-r1-opus-model/results-long/full-crafted-V1.summary`：Gone 事件 0 次（这一族里抬 F 第二次被拒、挤不出旧根，没造出来），K1 那两次只靠 G-long、F2。

## 三、K2：和「崩溃恢复抛弃的时间线可以重发」怎么共存

条文的分界：「时间线」含管理员回退抛弃的那一段、不含崩溃恢复抛弃的那一段（`.claude/kb/decisions/08-核心索引结构.md:207`：「**射程**：**同一条时间线上 inode 号不复用**」）；抛弃按实例表行判，`publish_is_abandoned_by_the_instance_table` 只有一句（「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1561`））。

### 1. 合法的跨被抛弃时间线重发，模型里真造出来了

P 臂两组（环 4 槽、长 7）每一张镜像都合法（`legal_images` = `images`），其中「这一步发的号，在环里一条按最新那条根的实例表判被抛弃的根上有另一个出生代」分别 48356 次、35042 次（`expected/model-main-0.out`、`expected/model-main-1.out` 第 1 行，整行原样）：

```
W3MODEL arm=Product mutation=None ring=4 sweep_threshold=never longest=7 full_length_histories=9863012 images=10836269 legal_images=10836269 events=0 events_where_I-9.29_holds=0 residual_images=0 reissues_across_abandoned_roots=48356
W3MODEL arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 full_length_histories=9985685 images=10964563 legal_images=10964563 events=0 events_where_I-9.29_holds=0 residual_images=0 reissues_across_abandoned_roots=35042
```

副本装置这一形一次都没有：E175 的 Q3b 全 0，原型的 G-long P 64 段、F2 P 256 段 25280 步每个候选红 0 步（`results-long/full-g-long-P.out`、`results-long/full-f2-P.out`，第二节第 1 条那张合计的同一条命令）；产品路径上崩溃恢复抛弃根在 C554 乙之后只剩「系统配置没见证最新那条根」一形，执行器的 crash 那一步造不出来，只有模型造得出。

### 2. 误报：只在三种读法上打中

两组 P 臂各候选判红的镜像数（同两份文件第 2–22 行的 `false_positive=`）：C1、C1s、C2a、C2as、C2ax、C2b、C2c、C2cs、C2d、C3r、C3rs、C3rx、C3fe、C4 全是 0；打中的三种：

| 读法 | 阈值「从不」 | 阈值 0 | 最短反例（`expected/model-main-1.out` 的 `ops=`） | 为什么红 |
|---|---|---|---|---|
| C1m（切换时 M 取内存里的水位） | 3073090 | 2418349 | 第 23 行 `[switch-error]` | 建 inode 那次发布写失败、切换之后向调用者报错不重做：行里 M = 3，盘上水位 2，合法 |
| C3f（定长区，按行判抛弃，清扫不管它） | 0 | 4192897 | 第 26 行 `[crash-mid-create,create,create,mount]` | 建到一半崩溃留下 (实例 1, txg 4, 2 号@4)，新实例从 txg 4 起又发 2 号@5；挂载末尾清扫删掉实例 1 那一行之后「无行 ⇒ 已发布」，2 号两个出生代 |
| C3fb（定长区只有 (号, 出生代)） | 8793358 | 8916308 | 第 32 行 `[crash-mid-create]` | 没有实例就判不了抛弃：没成的那一条出生代 4 ≤ 新实例的 txg 4，算进来，2 号 ≥ 水位 2 |

C3f 那一条的逐步打印（同一文件第 27–31 行，整行原样）：

```
W3TRACE arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C3f kind=false_positive step=start newest=1:3 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3]
W3TRACE arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C3f kind=false_positive step=0 op=crash-mid-create newest=2:4 wm=2 F=0 tree=[1@3] rows=[1:3:M2:Mm2] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3] ring=[2:4,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=0 red=[C3fb]
W3TRACE arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C3f kind=false_positive step=1 op=create newest=2:5 wm=3 F=0 tree=[1@3,2@5] rows=[1:3:M2:Mm2] peak=3 peak_pair=3@5 lineage=0 c3r_log=[1@3,2@5] ring=[2:4,2:5,1:3] fixed=[1:3:1@3,1:4:2@4,2:5:2@5] truth_reissue=0 truth_regressed=0 red=[C3fb]
W3TRACE arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C3f kind=false_positive step=2 op=create newest=2:6 wm=4 F=0 tree=[1@3,2@5,3@6] rows=[1:3:M2:Mm2] peak=4 peak_pair=4@6 lineage=0 c3r_log=[1@3,2@5,3@6] ring=[2:4,2:5,2:6,1:3] fixed=[1:3:1@3,1:4:2@4,2:5:2@5,2:6:3@6] truth_reissue=0 truth_regressed=0 red=[C3fb]
W3TRACE arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C3f kind=false_positive step=3 op=mount newest=3:8 wm=4 F=0 tree=[1@3,2@5,3@6] rows=[2:6:M4:Mm4] peak=4 peak_pair=4@6 lineage=0 c3r_log=[1@3,2@5,3@6] ring=[3:8,2:5,2:6,3:7] fixed=[1:3:1@3,1:4:2@4,2:5:2@5,2:6:3@6] truth_reissue=0 truth_regressed=0 red=[C3f,C3fb]
```

C3fe（清扫删行时把这个实例 txg > T_pub 的条目一起抹掉）两组都是 0，同一批历史里 V1、V2 的重发也一次不漏（`expected/model-main-3.out`、`expected/model-main-5.out` 的 C3fe 行）。所以候选三若放在根外，它的每一条要么跟着实例表清扫一起抹（D18 已定项 11「回收」那一段只管单元，`.claude/kb/decisions/18-块里携带什么信息.md:315`：「其中没有该实例的未发布单元」），要么就挂在根下随根 COW（C3r），不能只按行判。

### 3. 历史在哪几处会丢

| 处 | 候选 | 丢了是什么 | 依据 |
|---|---|---|---|
| 管理员回退 | C2*、C3r（挂在根下的） | 回退那次若从 R_old 带（与 inode 树同处理），被回退抛弃那一段的历史没了，之后的重发漏报 | 第四节变异 A；D23 已定项 14 只给记账写了「现算、不从 R_old 那棵账载入」，没给新记录写 |
| 管理员回退 | C1 | 不丢：实例表照 cur 的 | 「实例表照 cur 的」在 `.claude/kb/decisions/23-journal的角色与格式.md:378` |
| 实例切换 | C1m | 误报（上表） | 模型 `[switch-error]` |
| 实例切换 | C2*、C3r | 按读盘择的根带就不丢；若从内存带，与 C1m 同形误报（推的，没跑） | 切换「重新读盘择根」 |
| 实例表行回收 | C1 | 漏报：删掉一行就删掉那一行的 M | 下面的反例 |
| 实例表行回收 | C3f | 误报（上表） | 模型 `[crash-mid-create,create,create,mount]` |
| 实例表行回收 | C2*、C3r、C4 | 不受影响（不在实例表里） | 模型阈值 0 那一组这几种读法误报 0 |

C1 因为清扫删行才漏掉的那一次（V1、阈值 0，`expected/model-main-3.out` 第 75 行 `[create,create,mount,rollback3,create,crash-hide,create]`；C1k 是同一个判词但删掉的行也留着）：第 2 步挂载给实例 1 写行 M4；第 3 步 V1 回退到水位 2；第 5 步崩溃恢复抛弃最新那条根、新实例 3 挂载，清扫删掉实例 1 那一行（环里已没有实例 1 的根），剩下的行只有 (2, 7, M2)；第 6 步重发 2 号，C1 不红、C1k 红。逐步打印第 76–83 行：

```
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=start newest=1:3 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=0 op=create newest=1:4 wm=3 F=0 tree=[1@3,2@4] rows=[] peak=3 peak_pair=3@4 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=1 op=create newest=1:5 wm=4 F=0 tree=[1@3,2@4,3@5] rows=[] peak=4 peak_pair=4@5 lineage=0 c3r_log=[1@3,2@4,3@5] ring=[1:4,1:5,1:3] fixed=[1:3:1@3,1:4:2@4,1:5:3@5] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=2 op=mount newest=2:6 wm=4 F=0 tree=[1@3,2@4,3@5] rows=[1:5:M4:Mm4] peak=4 peak_pair=4@5 lineage=0 c3r_log=[1@3,2@4,3@5] ring=[1:4,1:5,2:6,1:3] fixed=[1:3:1@3,1:4:2@4,1:5:3@5] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=3 op=rollback3 newest=2:7 wm=2 F=0 tree=[1@3] rows=[1:5:M4:Mm4] peak=4 peak_pair=4@5 lineage=4 c3r_log=[1@3,2@4,3@5] ring=[1:4,1:5,2:6,2:7] fixed=[1:3:1@3,1:4:2@4,1:5:3@5] truth_reissue=0 truth_regressed=1 red=[I-9.6,C1,C1m,C1s,C2a,C2as,C2b,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C1k,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=4 op=create newest=2:8 wm=3 F=0 tree=[1@3,2@8] rows=[1:5:M4:Mm4] peak=4 peak_pair=4@5 lineage=4 c3r_log=[1@3,2@4,2@8,3@5] ring=[2:8,1:5,2:6,2:7] fixed=[1:3:1@3,1:4:2@4,1:5:3@5,2:8:2@8] truth_reissue=1 truth_regressed=1 red=[I-9.29,I-9.6,C1,C1m,C1s,C2a,C2as,C2b,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C1k,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=5 op=crash-hide newest=3:10 wm=2 F=0 tree=[1@3] rows=[2:7:M2:Mm2] peak=4 peak_pair=4@5 lineage=4 c3r_log=[1@3,2@4,3@5] ring=[2:8,3:9,3:10,2:7] fixed=[1:3:1@3,1:4:2@4,1:5:3@5,2:8:2@8] truth_reissue=0 truth_regressed=1 red=[C2a,C2as,C2b,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C1k,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=0 longest=7 candidate=C1 kind=event_missed_only_because_the_sweep_deleted_the_row step=6 op=create newest=3:11 wm=3 F=0 tree=[1@3,2@11] rows=[2:7:M2:Mm2] peak=4 peak_pair=4@5 lineage=4 c3r_log=[1@3,2@4,2@11,3@5] ring=[2:8,3:9,3:10,3:11] fixed=[1:3:1@3,1:4:2@4,1:5:3@5,2:8:2@8,3:11:2@11] truth_reissue=1 truth_regressed=1 red=[C2a,C2as,C2b,C2c,C2cs,C2d,C3r,C3rs,C3f,C3fe,C3fb,C4,C1k,C2ax,C3rx,C2ay]
```

产品路径上清扫阈值的占位是 369 行（「pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:74`）），阈值是配置项（D18 已定项 11「回收」那一段），阈值 0 是合法配置；占位下要 370 次挂载才走到这一支，副本装置上没跑。

## 四、K3：新不变量的文字放过哪条变异

### 1. 每个候选我按什么文字判

自然读法就是第一节表里的判词：「最新那条根的水位 ≥ 历史里记的每一个值」（C1、C2a 前半、C2b、C2c、C2d 前半、C4），C3 是「同号只有一个出生代，水位大于记录里每一个号」。加严的三种：
- 「历史没写」也要红：C2as（最新那一代有行且等于水位）、C2cs（峰值 ≠ 水位就红）、C3rs（最新那版树里每条都在记录里）、C1s（行的根还在环里时 M 等于它的水位）；
- 再拿环里读得出、按实例表不被抛弃的根交叉核：C2ax、C3rx；
- 我提的 C2ay（只在我的模型上量过、被攻过零轮）：谱系记录 (T_rb, T_old) 再带 (cur 的 txg, 回退前水位)，判词加「谱系里每一条带的这一对都在历代行里」。谱系树随现行那一版、不取 R_old（`.claude/kb/decisions/23-journal的角色与格式.md:378` 那一段：「墓碑树、谱系树随现行那一版，不取 R_old」），所以它是回退那次唯一不跟 R_old 走的旁证。

### 2. 变异与结果

三条历史写错的变异（模型里写进影子记录，管理员回退的水位照 P 或 V1）：A 回退那次根下的历史取 R_old 那一版的（就是 V1 的同源错：V1 只把水位取错，A 把历史一起取错）；B 历史只照抄这一版自己的值、不从父版本带；C 历史一样都不写。另有 V2（发布不加新建个数，不是历史写错，是号根本没往前走）。

`research/prompts/m3-w3-r1-opus-model/expected/model-mutation.out`（环 4、阈值 0、长 6，每组 978878 张镜像；V1 组 26410 次重发，其中 I-9.29 判成立 10030 次），`mutated_history_undetected`／`mutated_images` 是「历史被改过的镜像里不红的／全部」，V1 那一列是漏掉的重发：

| 判词 | P+A 不红 | P+B 不红 | P+C 不红 | V1+A 漏 | V1+B 漏 | V1+C 漏 |
|---|---|---|---|---|---|---|
| C2a | 593287/593287 | 978878/978878 | 978878/978878 | 26410 | 26410 | 26410 |
| C2as | 593287/593287 | 978878/978878 | 0/978878 | 26410 | 26410 | 0 |
| C2ax | 155355/593287 | 0/978878 | 0/978878 | 6350 | 0 | 0 |
| C2ay（我提的） | 0/593287 | 0/978878 | 0/978878 | 0 | 0 | 0 |
| C2c / C2cs | 0/0 / 0/0 | 0/0 / 0/0 | 978878 / 0（共 978878） | 26410 / 26410 | 26410 / 26410 | 26410 / 0 |
| C3r | 217753/217753 | 338149/338149 | 978878/978878 | 26410 | 26410 | 26410 |
| C3rs | 217753/217753 | 338149/338149 | 0/978878 | 26410 | 26410 | 0 |
| C3rx | 76091/217753 | 112490/338149 | 0/978878 | 8778 | 8670 | 0 |
| C4 | 196430/196430 | 0/0 | 593287/593287 | 26402 | 23054 | 26410 |
| C1 / C1s | 0/0 / 0/0 | 0/0 / 0/0 | 961526 / 220384（共 961526） | 26058 / 26058 | 26058 / 26058 | 26410 / 6851 |

（出自同一文件第 4–22、67–85、115–133、168–186、535–553、867–885 行的这几条；C2c 在 P+A、P+B 上峰值本来就等于水位，变异改不动它，「0/0」是没有被改过的镜像。）

读法：
- **自然读法放过「不写」（C）**：C1、C2a、C2c、C3r、C4 在 P+C 上一张都不红。正文第三节 K3 要的「把那份历史写错或不写，它必须红」，自然读法一个都过不了；加「最新那一代要有」（C2as、C2cs、C3rs）才红。
- **加严之后仍放过「回退那次从 R_old 带」（A）**：C2as、C3rs 全放过；连环里交叉核的 C2ax、C3rx 也只在被回退抛弃那一段的根还在环里时红，挤出去之后就看不见（V1+A 漏 6350、8778 次，全是 I-9.29 也判成立的那一形）。C3rx 的最短反例 `[create,delete,rollback2,rollback3,create]`（同一文件第 506 行），逐步打印第 507–512 行：

```
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=start newest=1:3 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=0 op=create newest=1:4 wm=3 F=0 tree=[1@3,2@4] rows=[] peak=3 peak_pair=3@4 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=1 op=delete newest=1:5 wm=3 F=0 tree=[1@3] rows=[] peak=3 peak_pair=3@4 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:5,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=0 red=[]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=2 op=rollback2 newest=1:6 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=2 c3r_log=[1@3] ring=[1:4,1:5,1:6,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[I-9.6,C3f,C3fe,C3fb,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=3 op=rollback3 newest=1:7 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=2 c3r_log=[1@3] ring=[1:4,1:5,1:6,1:7] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[I-9.6,C3f,C3fe,C3fb,C2ax,C3rx,C2ay]
W3TRACE arm=RollbackTakesTheTargetsWatermark mutation=HistoryFromTheTarget ring=4 sweep_threshold=0 longest=6 candidate=C3rx kind=event_missed step=4 op=create newest=1:8 wm=3 F=0 tree=[1@3,2@8] rows=[] peak=3 peak_pair=3@8 lineage=2 c3r_log=[1@3,2@8] ring=[1:8,1:5,1:6,1:7] fixed=[1:3:1@3,1:4:2@4,1:8:2@8] truth_reissue=1 truth_regressed=0 red=[C3f,C3fe,C3fb,C2ax,C2ay]
```

- **V2 一形，所有「水位历史」读法全放过**：发布不加新建个数时水位从不低于历史，C1、C2a、C2ax、C2ay、C2c、C2d、C4 在 V2 组 1523216 次重发里一次不红，只有按号记出生代的 C3r、C3rx 全红（`expected/model-main-4.out` 第 4、7、10、12、13、18、20、21、22 行）。E175 V2 臂的重发都是「还在」、I-9.29 已经红（E175 那一页「判据逐条」一节 R-V2 一行）；看不见的是 V2 加删除加抬 F 那一形，最短 `[create,delete,raiseF,create]`（同一文件第 227 行），逐步打印第 228–232 行：

```
W3TRACE arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 candidate=C2ay kind=event_missed_where_I-9.29_holds step=start newest=1:3 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3]
W3TRACE arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 candidate=C2ay kind=event_missed_where_I-9.29_holds step=0 op=create newest=1:4 wm=2 F=0 tree=[1@3,2@4] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[I-9.6,C3r,C3rs,C3f,C3fe,C3fb,C3rx]
W3TRACE arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 candidate=C2ay kind=event_missed_where_I-9.29_holds step=1 op=delete newest=1:5 wm=2 F=0 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:5,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[I-9.6,C3r,C3rs,C3f,C3fe,C3fb,C3rx]
W3TRACE arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 candidate=C2ay kind=event_missed_where_I-9.29_holds step=2 op=raiseF newest=1:6 wm=2 F=5 tree=[1@3] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3,2@4] ring=[1:4,1:5,1:6,1:3] fixed=[1:3:1@3,1:4:2@4] truth_reissue=0 truth_regressed=1 red=[C3r,C3rs,C3f,C3fe,C3fb,C3rx]
W3TRACE arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 candidate=C2ay kind=event_missed_where_I-9.29_holds step=3 op=create newest=1:7 wm=2 F=5 tree=[1@3,2@7] rows=[] peak=2 peak_pair=2@3 lineage=0 c3r_log=[1@3,2@4,2@7] ring=[1:4,1:5,1:6,1:7] fixed=[1:3:1@3,1:4:2@4,1:7:2@7] truth_reissue=1 truth_regressed=1 red=[I-9.6,C3r,C3rs,C3f,C3fe,C3fb,C3rx]
```

所以 K3：只记水位的历史（C1、C2*、C4）造不出「V2 那一形必须红」；只记 (号, 出生代) 的 C3 造不出「回退那次从 R_old 带也必须红」，除非再拿一样不跟 R_old 走的东西（谱系记录）核它。C2ay 在我的模型上把 A、B、C 三条都判红，但它对 V2 同样瞎（第 22 行）；能同时过 V2 与 A 的组合是「C3r 加一条与 C2ay 同形的谱系核对」（推的，没实现、没跑）。

## 五、每个打中先答四句

| 打中 | 分不分辨臂（候选） | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一句 | 跑前条款给的改法在这几格上 |
|---|---|---|---|---|
| K1-甲：C1、C2b 漏 op92、op95 | 分辨：同两步 C2a、C2d、C3 红（第二节第 2 条两行） | 看不到：那两个号的旧出生代只在已挤出候选集的根里，C1、C2b 记的值在实例 8 之内被覆盖成 7 | 正文第五节「一个候选被攻方坐实一形漏报（同一条时间线上重发而判不红）」；E175 的 `kind=q1_same_timeline` 与模型的时间线真相都按 D8 已定项 6 射程判 | 跑前条款只给「出局」，没给改法 |
| K1-乙：C2c、C4 漏 op95 | 分辨：同一步 C2a、C2d、C3 红 | 看不到：重发 8 号之后水位 9 = 峰值 9，镜像上与合法的同一个状态逐字段相同，只记水位的读法分不开 | 同上 | 同上 |
| K1-丙：C2d 在越过旧峰值之后不红 | 分辨：C2a、C3 同一张镜像红 | 看不到：(峰值, 代) 被新值覆盖 | 按最弱读法记：重发那一步 C2d 红，后来的镜像里被重发的对象还活着而不红；归不归「判不红」那一句由主 agent 定 | 同上 |
| K2：C1m、C3f、C3fb 误报 | 分辨：C1、C3fe、C3r 同一批历史 0 误报 | 判别子在盘上：C3f 缺的是「清扫删行时连这个实例没发布的条目一起抹」；C3fb 缺实例代号；C1m 缺的是「M 只取盘上所选那一版」 | C3f、C3fb 是正文 K2「被崩溃恢复抛弃的那一段里发过的号合法重发却被它判红」（没成的那次发布里发的号，恢复之后再发）；C1m 那一例没有重发，只是合法镜像判红，字面不在「合法重发判红」里，归哪一句由主 agent 定 | 跑前条款没给改法 |
| K2：C1 因清扫删行漏报 | 分辨：C1k（删掉的行也留着）同一步红 | 看不到：那一行已删 | 第五节「漏报」那一句 | 同上 |
| K3：变异 A、B、C 不红 | 分辨：C2ay 在三条上全红；C2as、C3rs 只在 C 上红 | A：被回退抛弃那一段的根挤出环之后，镜像里没有旁证（谱系记录不带这份东西时） | 正文 K3「一条变异：把那份历史写错或不写，它必须红」，字面分句「写错」（A、B）与「不写」（C） | 同上 |
| K3：V2 一形所有水位历史读法不红 | 分辨：C3r、C3rx 全红 | 看不到：V2 下水位从不低于任何历史值 | 不是「历史写错」，按字面落在第五节「漏报」那一句（同一条时间线上重发而判不红） | 同上 |

## 六、改法各修哪一格（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 | 依据 |
|---|---|---|---|
| 判词里加「按号或按代」比：C2a 的后半句（树里 (n, b) 的 n 小于代 < b 的行里最大水位就红） | K1-乙 op95 | 量过 | 副本装置 op95 那一行 `w3_c2a=1`；模型 V1 两组 C2a 漏 0（`expected/model-main-2.out`、`-3.out` 的 C2a 行） |
| 只记 (峰值, 代) 的 C2d | K1-乙 | 量过 | 同上 `w3_c2d=1`；但留下 K1-丙 |
| 保留历代（C2a）而不是一个峰值 | K1-丙 | 量过 | 模型 C2a 的 `residual_missed=0` |
| 候选一改成「当前实例也要有记录」 | K1-甲 | 推的 | 写行的时机由 D18 已定项 11 定在恢复与切换；改了就不是候选一 |
| C3 放进根下随根 COW（C3r），或放根外但清扫删行时连该实例 txg > T_pub 的条目一起抹（C3fe） | K2 C3f | 量过 | 模型两种都 0 误报 |
| 定长区每条带实例代号 | K2 C3fb | 量过 | C3f、C3fe 在没清扫时 0 误报 |
| 切换写行时 M 只取读盘择的那一版 | K2 C1m | 量过 | C1 0 误报 |
| 清扫删行时把 M 并进留下来的行（C1k 的效果） | K2 C1 清扫漏报 | 量过（只这一格） | C1k `event_missed_only_because_the_sweep_deleted_the_row=0`；C1k 自己仍漏 K1-甲 |
| 判词加「最新那一代要有、要等于水位」 | K3 C | 量过 | C2as、C2cs、C3rs 在 P+C、V1+C 上全红 |
| 谱系记录带 (cur 的 txg, 回退前水位)，判词核它在历代行里（C2ay） | K3 A、B、C | 量过 | 第四节第 2 条表 |
| C3r 加一条同形的谱系核对（谱系记录带回退前那一版发号记录的条数或摘要） | K3 A（C3 那一侧） | 推的 | 没实现 |
| 只有按号记出生代的记录（C3） | K3 V2 | 量过 | `expected/model-main-4.out` 第 13、21 行 |

## 七、没打中的形状

- **挂在根下的读法在崩溃恢复抛弃之后误报**（C1、C2a/as/ax/ay、C2b、C2c、C2d、C3r/rs/rx、C4、C3fe）：模型 P 臂环 4、长 7 两组共 21800832 张镜像，其中跨被抛弃时间线的合法重发 48356 + 35042 次，误报 0；环 6、长 6 再 1009096 张、合法跨抛弃重发 6432 次，误报 0（`expected/model-ring6.out`）；副本装置 P 臂 G-long 64 段、F2 256 段共 25280 步，误报 0（但那里一次跨抛弃重发都没有，只证「没有退回的历史上不误报」）。扫的步：建、删、回退 1–3 步、抬 F、空发布、挂载、崩溃抛弃最新那条根、建到一半崩溃、切换重做、切换报错，阈值 0 与「从不」。
- **C2a、C3r 在正确写法下漏 V1 的重发**：模型 V1 两组（环 4、长 7）共 670350 次重发 0 漏，环 6 那组 28468 次 0 漏；副本装置 G-long V1 476 次、F2 V1 336 次 0 漏。
- **实例切换重做（switch-redo）让任何读法误报**：没有，C1m 的误报只来自 switch-error。
- **E175 原样的 crash 一步造出跨抛弃重发**：执行器那一步在 C554 乙之后拒可写，造不出；定制族 V1 5220 段里抬 F 第二次被拒、Gone 0 次（第二节第 5 条），副本装置上 K1 那一形只在 G-long、F2 里出现。

## 八、这条腿自己的限度

- 候选都没写进 `crates/`：副本装置里是按根链算的影子，模型里是按条文写的状态机；候选自己的写在崩溃点上怎么落（先写根还是先写定长区）只在模型里按我选的次序穷举，层 0 枚举没有对它们跑。原因：候选的写不在真代码的录制流里，`enumerate_layer0` 判的是真流；它在 checker 档包里，子 agent 不跑那个包的测试。这一轮我起的层 0 状态数是 0。
- 模型的抬 F 直接抬到 cur 的 txg；副本装置上第二次抬 F 被拒（第七节最后一条），实际的上限比模型紧，所以模型里「挤出候选集」来得比产品路径快。K1 那两次本身在副本装置上（G-long）出现过，不靠这一点。
- 模型只有一个头、没有克隆；没有 unlink 之外的删除批、没有多盘部分失败；回退候选集的「带文件」一律当带文件。
- 判词是我写的最自然的读法与几种加严，不是正推腿给的文字；正推腿若给了别的判词，第四节那张表要按它的文字重判。
- 每格只抽了一次样（没有 s2），没打中的那几格照 `.claude/rules/three-way-inference.md` 不算「没打中」。
- 副本装置上的数不是入库装置上的数。

## 九、没做什么

- K4（代价）不归这条腿，没量字节与片数。
- 定制族 P 臂（5220 段）没跑：副本装置合计已到约 35 分钟（草稿目录 `progress.md`）。
- 没跑任何重型测试；没跑门禁与 lint。

