# dyn-share-r1 正推腿报告：K1 闲置消没消、K2 键与复用、K3 断点

日期：2026-10-18（`TZ=Asia/Tokyo date +%F` 的输出；派发提示与正文写的是 2026-10-17）。腿：three-way-forward。
范围：只读 `crates/`、`scripts/`、`.claude/kb/`、`records/` 与背景材料；没起 cargo、没跑测试、没读崩溃放量库目录。
引文写法：`文件:行号` 都是写进报告前在被引文件里 `grep -nF` 过的此刻行号。算术脚本与原样输出在草稿目录 `~/.local/share/singlefs/work/dyn-share-r1-forward/k1.py`、`k1.out`。

## 0 先说三件影响读数的事

### 0.1 正文第二节的行号在工作树里已漂，下面一律用此刻的

| 正文写的 | 此刻现查 |
|---|---|
| `HostShare::parse` 在 `:11900` | 「pub fn parse(text: &str) -> Result<Self, String> {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:11910`） |
| `host_share_and_weights_from_the_environment` 在 `:11929` | 「pub(crate) fn host_share_and_weights_from_the_environment() -> Result<(HostShare, Vec<u64>), String>」（`crates/singlefs-checker-tier/src/crash_amplification.rs:11934`） |
| `record_one_observer_piece` 在 `:16819` | 「fn record_one_observer_piece(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17195`） |
| `owner_of_block` 在 `:1297–1335` | 「pub fn owner_of_block(&self, block_sequence: u64) -> usize {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1315`） |
| 停下文件 `:2033–2042` | 「pub fn stop_is_requested_by_the_environment() -> bool {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2036`） |
| 驱动文件头第 17–35 行、第 30 行 | 步骤 ①–④ 在 17–36 行，「取回的库里第二台录的观察者块不并进本机库，并进去的是它判的结论（步骤 ④）」（`scripts/crash/crash-amplification-two-hosts.sh:30`）；「② 失败…也先导入再退」那句落在 35 行、「第二台判到一半的块不丢」在 36 行 |

行号漂移不改变正文第二节的事实判断（我逐条对过代码，见各节）；它说明背景材料里的行号不能直接抄。

### 0.2 题面给的块数有三套，互相对不上

| 来源 | 数 |
|---|---|
| K1 格 | 本机 8946 块、第二台 13419 块，合 22365 块（本机占 2/5，即权重 2:3，见 `k1.out`） |
| `records/2026-10-13-复用键与全仓改名.md:920` | 「本机 2 小时 25 分录完自己那一份后第二台还要约 4 小时」，每态录入速度「约每秒 8.5 万与 9.1 万态（量的）」：8700 s × 85000 态/s ÷ 65536 态/块 = 11284 块是本机那一份；本机占 2/5 时总块数 = 28210 |
| `records/2026-10-13-复用键与全仓改名.md:933` | 停下时「本机 every_state 段录完、mount_continuation 段到 full.index_node.23 共 17664 块；第二台 mount_continuation 段到 full.index_node.15 共 23551 块」（口径是块数还是含别的段，我判不了） |

我判不了哪一套是第 6 条计划的真块数。引擎录入段结束时打一行「CRASH_AMPLIFICATION_RECORDED flow={flow_name} recorded_flow={} facts={} packed_judge_tables={} blocks={} exhaustive={} recording_reused={recording_reused}」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12494`），其中 `blocks=` 是计划的块数。要主 agent 在本机那趟日志里 `grep -n 'CRASH_AMPLIFICATION_RECORDED' <日志>` 取真块数。K1 的式子对块数是符号式，下面三套数都算。

### 0.3 录入段的有效速度比「量到的每态速度」低约两成（推的）

把 0.2 里两个整份用时当整份录入时间（它们是 `records` 里的话，口径没法在这里核）：

- 本机：8946 × 65536 ÷ 8700 s = 67389 态/s，对 85000 是 0.793。
- 第二台：13419 × 65536 ÷ 12240 s = 71849 态/s，对 91000 是 0.790。
- 两个系数几乎相等，两台速度比 67389 ÷ 71849 = 0.938，与量到的 85 ÷ 91 = 0.934 对上。
- 一块实际用时：本机 8700 ÷ 8946 = 0.973 s，第二台 12240 ÷ 13419 = 0.912 s；格内数 9 万态/s 给出 65536 ÷ 90000 = 0.728 s。下面「一块的用时」取 0.73 到 0.97 s 一段，界写成 `t_b` 的符号式，数字两头都给。

什么现象会推翻它：`records:920` 的 8700 s 不是整份录入（例如含判段或含等待），或块数取 28210 而不是 22365——那样系数就不再两台相等，这一节作废。

## 1 K1　闲置消没消

### 1.1 算术：两套数各算一遍（式子 + 原样输出）

记 `B = 65536`（「pub const STATES_PER_BLOCK: u64 = 1 << 16;」，`crates/singlefs-checker-tier/src/crash_amplification.rs:175`），`n_L`、`n_P` 本机与第二台的块数，`r` 态/秒，`t_b = B ÷ r` 一块的用时。

- 今天的分工：「第 k 块归哪一方只由权重表与 k 定，几方各算各的、不用商量。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:1295`）。两台各自的完成时刻 `T_L = n_L · t_b^L`、`T_P = n_P · t_b^P`，先完成的一台空闲 `|T_P − T_L|`，只受权重对速度估计的误差约束，没有「几块」量级的界。
- 理想的动态分工（可分、零开销）：两台同时收尾，`T_dyn = N ÷ (1/t_b^L + 1/t_b^P)`。

原样输出（`python3 ~/.local/share/singlefs/work/dyn-share-r1-forward/k1.py`，`nice` 不需要：纯算术，不起重活）：

```
== 格内数 (第 6 条): n_L=8946, n_P=13419, r=90000 态/秒, B=65536 ==
N = 8946 + 13419 = 22365  nL/N = 2/5
t_b = B/r = 65536/90000 = 0.72818 s
T_L = nL*t_b = 6514.3 s = 1.810 h = 108.6 min
T_P = nP*t_b = 9771.4 s = 2.714 h = 162.9 min
静态空闲(本机) = T_P - T_L = 3257.1 s = 0.905 h = 54.3 min  = (nP-nL)*t_b, nP-nL = 4473
动态理想 = N*t_b/2 = 8142.8 s = 2.262 h = 135.7 min  比静态少 1628.6 s = 0.452 h = 27.1 min
states L = 586285056  states P = 879427584

== 格内数, 分阶段速度换成 记录 line 920/945 的量的数 ==
录入 T_L = 6897.5 s = 1.916 h = 115.0 min | T_P = 9664.0 s = 2.684 h = 161.1 min | 空闲方: 本机 2766.6 s = 0.768 h = 46.1 min
   动态理想 N*B/(r_L+r_P) = 8327.9 s = 2.313 h = 138.8 min  理想本机份额 = 0.48295454545454547
判段 T_L = 6662.3 s = 1.851 h = 111.0 min | T_P = 5025.3 s = 1.396 h = 83.8 min | 空闲方: 第二台 1637.0 s = 0.455 h = 27.3 min
   动态理想 N*B/(r_L+r_P) = 5573.1 s = 1.548 h = 92.9 min  理想本机份额 = 0.33460076045627374

== 题面+第二节数 ==
本机 录入508 + 判段663 = 1171 s
第二台还要 录 3.4h = 12240.0 s; 判 1.4h = 5040.0 s; 合计 17280.0 s = 4.8 h
静态: 本机空闲到第二台收尾 = 17280.0 s (推的)
```

同一次输出的后半（接上面第 19 行起）：

```
反推: 第二台判 13419 块 @17.5万/s = 5025.30048 s = 1.3959168 h  (对 1.4h)
反推: 第二台录 13419 块 用 3.4h 的速度 = 71848.65882352942 态/秒; 若按 9.1万/s 则 2.6844553846153842 h
动态(理想,可分,零开销): 录 = 6328.6 s = 1.758 h = 105.5 min  判 = 3353.6 s = 0.932 h = 55.9 min  合计 9682.2 s = 2.690 h = 161.4 min  比静态少 7597.8 s = 2.110 h = 126.6 min

== 第 2 条 (8452/17114/12596 s) ==
本机速度 = 0.6710066687837408 块/秒 = 43975.09304541124 态/秒 (块按 65536)
权重比 n2P/n2L = 2.024846190250828
第二台速度上界(在 12596 s 时还没判完): < 1.358685296919657 块/秒; 速度比 <  2.024846190250828

== 甲 重复块上界 (L = 前沿取一次的间隔, 举 L=10 s, 30 s, 60 s; 两台各 9万/s) ==
L = 10 s: 两台各自越过交叉点的块数上界 L*r/B = 13.73291015625   合计重复 <= 2x = 27.4658203125
L = 30 s: 两台各自越过交叉点的块数上界 L*r/B = 41.19873046875   合计重复 <= 2x = 82.3974609375
L = 60 s: 两台各自越过交叉点的块数上界 L*r/B = 82.3974609375   合计重复 <= 2x = 164.794921875

== 乙 段长 S 块: 收尾空闲 <= S*t_b ==
S = 1 -> 0.7281777777777778 s
S = 16 -> 11.650844444444445 s
S = 64 -> 46.60337777777778 s
S = 256 -> 186.41351111111112 s

== 丁 传输量下界: 每块 observation_of_each_state 是 u32*65536 ==
每块 >= 262144 B = 256.0 KiB; 第二台 13419 块 >= 3517710336 B = 3.276123046875 GiB; 全部 22365 块 >= 5.460205078125 GiB
```

读数（式子都在上面输出的标签里，这里只落结论）：

- 同速一条：格内数、两台同速（`n_L = 8946`、`n_P = 13419`、`r = 90000`）：`T_L = 8946 × 0.72818 = 6514.3 s`，`T_P = 13419 × 0.72818 = 9771.4 s`，今天的做法下本机空 `(13419 − 8946) × 0.72818 = 3257.1 s`（54.3 min）；动态理想 `T_dyn = 22365 × 0.72818 ÷ 2 = 8142.8 s`，两台都比今天的 `T_P` 早 1628.6 s（27.1 min）。
- 分阶段速度一条：格内数、分阶段用 `records` 里量的速度（录入 85000 与 91000，判段 88000 与 175000，来源 `records/2026-10-13-复用键与全仓改名.md:920` 与 `:945`）：录入时本机先收尾空 2766.6 s（46.1 min），判段时**第二台**先收尾空 1637.0 s（27.3 min）。空闲的一方在两个阶段里换了边：同一个 2:3 的静态权重对录入偏向第二台、对判段偏向本机，没有一个静态权重对两个阶段都对（理想本机份额录入 0.483、判段 0.335）。
- 题面一条：本机 508 + 663 = 1171 s 收尾后，第二台还要 3.4 h + 1.4 h = 17280 s。今天的做法下本机空闲就是这 17280 s（4.8 h，推的：这两个时长是正文里标「推的」的）。动态理想（本机按量的速度比分担第二台剩下的，录入 85000 ÷ 91000，判段 88000 ÷ 175000）：录 `12240 ÷ (1 + 0.934) = 6328.6 s`，判 `5040 ÷ (1 + 0.503) = 3353.6 s`，合 9682.2 s（2.69 h），比今天少 7597.8 s（2.11 h）。
- 互核一条：两套数互相的核对：题面的「判约 1.4 小时」= 13419 × 65536 ÷ 175000 = 5025.3 s = 1.396 h，与格内块数、`records:945` 的第二台判速 17.5 万逐位对上；题面的「录约 3.4 小时」对不上 9.1 万的速度（13419 × 65536 ÷ 91000 = 2.68 h），对得上 0.2/0.3 说的有效速度 71849 态/s。
- 全量第 2 条一条：8452 块、17114 块、本机 12596 s 判完）：本机速度 8452 ÷ 12596 = 0.671 块/s（43975 态/s，块按 65536 算）；权重比 17114 ÷ 8452 = 2.025，本机收尾时第二台还没完，所以第二台的速度比不到 2.025（上界，不是测到的）。这条说明连「按量到的速度配权重」也会错，且错的方向在本机已经出现。

什么现象会推翻上面五条读数：块数不是 22365（0.2），或 `r` 随阶段、随点（观察者点与非观察者点）变到这几个数之外。题面一条的 7597.8 s 还要求「本机能直接接手第二台没录的块」，即候选能分、不丢（后面各节）；它是上界，不是承诺。

### 1.2 每个候选的闲置界（符号式 + 数）

先记前提，每条都是代码事实：

- 共用块序号：录入与判段用同一个块序号：录入段 「let this_block = block_sequence;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17561`）、观察者判段 「let this_block = block_sequence;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18657`）、`pending_blocks` 「let this_block = block_sequence;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2694`），都是在 `plan.points` 里按点、按块顺序数全部块（不分观察与否）。所以一份前沿或一份领用账可以同时管两个阶段。
- 叫停谓词：叫停钩子每领一块问一次，且一趟里见过一次就记住：「if stop_is_requested() {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17566`，录入段）与（`crates/singlefs-checker-tier/src/crash_amplification.rs:2314`，CPU 判段），「/// 一趟里见过一次就记住（第 10z 批第六块，第 10z 批第五块代码轮判决 K2-2），与一轮收尾问的是同一个。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2852`，GPU 判段）。「前沿越过」是单调的，和「记住」相容，能直接当这个谓词用。
- 两种结论：判段的两种结论只有一种要读录入块：镜像判定走 「|point_index, local| judge_one_block(flow, plan, point_index, local)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2099`）与 「// 判表先入库、再读库，GPU 只拿读回的那一份算」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3412`），只用流与判表，哪台都能判任何一块；观察者结论读 「.read_observer_block(&key, &kind, &observer_versions.observer)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18673`），读不到报 「的录入段那一块不在库里：先跑录入段」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18677`）。
- 没有跑中通道：`multi-host-run.sh` 没有跑中取、送文件的口子：取回只在命令退出或收到 TERM 之后（`grep -n 'multi_host_fetch' scripts/hosts/multi-host-run.sh` 命中 184、189、254、373 四行，189 是函数定义，三处调用是 184 收到 TERM 之后、254 本机模式、373 第二台命令退出之后）；跑中它每 5 s（`MULTI_HOST_RUN_POLL_SECONDS` 默认 5，文件头第 24 行）经 ssh 问一次「退出码在不在、进程在不在」。

`crates/` 里没有这几样（候选要加在哪一处在各行里写）：

```
$ grep -rnE '前沿|领块方向|发块|领用账|frontier' crates/singlefs-checker-tier/src | wc -l
0
$ grep -nE '外部命令领块|claim_next_block' crates/singlefs-checker-tier/src/crash_amplification.rs | wc -l
0
pending_blocks(2684-2706)内 .rev(): 0
record_the_observers_of_the_flow(17497-17690)内 .rev(): 0
judge_the_observers_of_the_flow(18623-18800)内 .rev(): 0
```

| 候选 | 录入段先收尾一台的空闲界 | 判段先收尾一台的空闲界 | 数（`t_b` 取 0.73 到 0.97 s） | 要在 `crates/` 与 `scripts/` 里加的 |
|---|---|---|---|---|
| 今天（对照） | `\|T_P − T_L\|`，无「几块」界 | 同 | 格内数 54.3 min；分阶段 46.1 min（录入，本机空）与 27.3 min（判段，第二台空）；题面 4.8 h（推的） | 无 |
| 甲 | `≤ t_b^慢 + L_x`；两台各越过交叉点最多 `⌈L_x·r ÷ B⌉` 块，重复做的块 `≤ 2·⌈L_x·r ÷ B⌉`（`L_x` = 对方前沿的陈旧度 = 写出间隔 + 通道延迟 + 读间隔） | 判段所有权 = 录入所有权时，等于判段不均（同「今天」那一行）；判段另扫一遍时 `≤ t_b + L_x`，但观察者结论读不到对方录的块（见两种结论一条） | `r = 90000` 时 `L_x = 10 / 30 / 60 s` 重复 `≤ 28 / 83 / 165` 块（两边合计，见 `k1.out` 末段）；`L_x` 候选没给，下限是现有 5 s 的轮询间隔（见没有跑中通道一条），上面三个值是示例不是拍的 | 录入、判段两个循环都得能反向走（上面零命中）；一条跑中交换前沿的通道（`multi-host-run.sh` 没有，见没有跑中通道一条）；叫停谓词读对端前沿（见叫停谓词一条，钩子现成） |
| 乙 | `≤ S·t_b + ρ`（`S` = 一段的块数，`ρ` = 一次向发块方领块的往返） | 同；但发块方若按「谁手里有这块的录入块」发，判段的不均回来；不按位置发就要搬行 | `S = 1 / 16 / 64 / 256` 给 `S·t_b = 0.73 / 11.7 / 46.6 / 186 s`（`t_b = 0.728`）；`ρ` 没量到 | 发块方常驻服务、向外部命令领块的接口（上面零命中）、重领与超时；吞吐损失 `ρ ÷ (S·t_b)` 随 `S` 反比 |
| 丙 | `≤ P + C_r`（`P` = 检查进度的间隔，`C_r` = 一次重分让两台停下再起的代价）；重分次数 `R ≤ ⌈T_dyn ÷ P⌉` | 同，且判段权重可以和录入权重不同（驱动每趟自己带权重） | `P = 30 min` 时 `R ≤ ⌈135.7 ÷ 30⌉ = 5`（格内数 `T_dyn`）；`C_r` 本机 = 清单、流程、事实表整条录「几十秒」（`scripts/crash/crash-amplification-two-hosts.sh:18`），第二台 = 拷树加重编本地包（`scripts/crash/crash-amplification-two-hosts.sh:62`），**没有量到的秒数** | 只改驱动：周期性读两台进度、建停下文件、换权重重起；第二台那一趟现在录入与判段在一个进程里（`scripts/crash/crash-amplification-two-hosts.sh:28`），要分别设权重就得拆成两趟，本机已经是两趟（`scripts/crash/crash-amplification-two-hosts.sh:20`） |
| 丁（两种做法） | 不减闲置，只让停下时不丢（K3） | 同 | 并进做法一趟要搬的行：第二台 13419 块的观察者块行至少 `13419 × 4 × 65536 = 3.28 GiB`（`observation_of_each_state: Vec<u32>,` 在 `crates/singlefs-checker-tier/src/crash_amplification.rs:16880`，每状态 4 字节，不含看过的恢复镜像行，不计压缩），实际大小没量（库在禁读范围） | 见 K3 |

### 1.3 K1 的结论（每条的推翻条件跟在后面）

- 录入段：甲与乙都能把先收尾的一台的空闲压到「一块的用时加一个通信量」（秒到十几秒），丙压不到块的量级，要付「检查间隔 + 重起代价」（分钟级）。甲与乙谁更小取决于甲的 `L_x` 与乙的 `ρ`、`S`，这两个数材料里没有，我没量（量 ssh 往返要碰多机配置里的私有值，且两台此刻都在跑全量）。推翻：量到 `L_x` 或 `ρ` 的实测值后，两个界的大小关系换边。
- 判段：**任何候选想让判段也不闲，都得让观察者结论的录入块在判它的那台上**。理由是代码：判段读本进程库里的录入块，没有就报错（`crates/singlefs-checker-tier/src/crash_amplification.rs:18677`）；而录入段与判段的理想份额不同（1.1 的分阶段速度一条：录入本机 0.483、判段本机 0.335），按这些量到的速度分，判段份额和录入份额不一致，所以必有行要从录它的那台搬到判它的那台。这一条把 K1 和 K3 接上：不带丁，判段只能保留「谁录谁判」，闲置回到今天的量级。推翻：量到观察者结论占判段用时的比例很小（我没有这个比例，材料里没有），那时镜像判定一种可以不搬行地动态（见 1.2 的两种结论一条）。
- 引擎的另一种入口：给定状态引擎的归属不是块序号而是表摘要头 8 字节，`owns_the_table`（定义在 `crates/singlefs-checker-tier/src/crash_amplification.rs:13528`）里「let block_number = u64::from_le_bytes(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:13529`），没有「从头往后、从尾往前」的次序。甲的「两端对扫」对这个引擎无从定义（要先给表排序），乙与丙没有这个问题。正文第二节说「流水线与给定状态表两种引擎同一种读法」只对 `SHARE`、`HOST_WEIGHTS` 成立，不对块序成立。推翻：给定状态表另有稳定的序号（我没找到：`grep -n 'owns_the_table' crates/singlefs-checker-tier/src/crash_amplification.rs` 命中 13528 的定义与 14339、14343、14353 三处使用，判法只有 13529 起取摘要头 8 字节这一种）。

## 2 K2　键与复用

### 2.1 今天的键里有什么、没有什么（现查）

- 块键：「块键只由内容定：段写表与起步镜像（复用键）、计划哈希（段展开、判器另读的输入、态数）、段内序号；设点文件、点名、身份都不进。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:405`）；签名「pub fn block_key(&self, local: &Range<u64>) -> VerdictBlockKey {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:359`）只收块的点内区间，份、权重、方向、前沿没有进参数的口子。
- 录入键：驱动文件头说清单、流程、事实表整条录，「（几十秒，录入键不含份）」（`scripts/crash/crash-amplification-two-hosts.sh:18`）。
- 观察者行的键（`VerdictStoreKey`，都在 `crates/singlefs-checker-tier/src/verdict_store.rs`）：「/// 观察者基底的键：它的摘要。」（`crates/singlefs-checker-tier/src/verdict_store.rs:1299`）、「/// 观察者看过的一张恢复镜像的键：去重键。」（`crates/singlefs-checker-tier/src/verdict_store.rs:1301`）、「/// 观察者去重一个桶的读集见证的键：桶键。」（`crates/singlefs-checker-tier/src/verdict_store.rs:1303`）、「/// 观察者录入段一块的键：块键 + 观察者种类 + 观察者版本。」（`crates/singlefs-checker-tier/src/verdict_store.rs:1305`）。去重键与桶键由内容拼：「/// 交回镜像的观察者去重的桶键：观察者名、观察者代码那一把版本（另读的输入那一格全零）、另读的输入的摘要、恢复报告的摘要。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:16550`）、「/// 一个见证看出的那张镜像在库里的键：桶键接见证摘要。」（`crates/singlefs-checker-tier/src/crash_amplification.rs:16566`）。
- 份与权重只产生「这台领哪些块号」这一个集合：「let (share, host_weights) = host_share_and_weights_from_the_environment()?;」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12372`）→「let host_shares = WeightedShares::new(host_weights);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12590`）→「let selected: std::collections::HashSet<u64> = my_blocks.iter().copied().collect();」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12599`）。`selected` 只经 `&|block| selected.contains(&block)` 交给下游，全文件 9 处（`grep -c 'selected.contains(&block)' crates/singlefs-checker-tier/src/crash_amplification.rs` 输出 9），下游函数的签名里这个闭包只有「是不是我的块号」一个用处；结论的版本从「ConclusionVersionsOfThePlan::of_the_plan_keeping_the_refused_out(&plan, toolchain);」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12536`）来，参数里没有份。

对着 D13（验证路线） 已定项 17：「一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。」（`.claude/kb/decisions/13-验证路线.md:428`）。这一条里「份、权重、方向、前沿」没有被点名，但「只有整道或整格的标记而没有按判定的库」与「库放在会丢的位置」「两份库重复判同一批」三个分句直接管下面的几个候选。

### 2.2 重起后「已录、已判的块按键认」今天是怎么认的

- 录入段：「observer_block_is_recorded_or_concluded(store, &key, observer_versions)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17571`）→ 它读本进程的库：「.read_observer_block(key, &kind, &observer_versions.observer)?」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17727`）或结论已在（`crates/singlefs-checker-tier/src/crash_amplification.rs:17730`）。认的键是（块键，种类，版本），与这块归谁无关；但**只在本进程开着的这一个库里找**。
- 判段：`pending_blocks`「if conclusions.is_stored_for_reuse(store, point_index, &point.block_key(&local))? {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2699`），观察者判段同理（`crates/singlefs-checker-tier/src/crash_amplification.rs:18663`）。
- 所以「按键认」的前提是行在这一台的库里。录入块今天不会离开录它的那台的库（K3 查），这是 K2 里候选之间真正的差别。

### 2.3 各候选逐格

| 候选 | 新增的、不是内容的输入 | 进块键 / 判器组键 / 库键吗 | 重起后已录、已判的块照旧按键认吗 | 判定 |
|---|---|---|---|---|
| 甲 | 领块方向、对方前沿 | 不进：只进遍历次序与叫停谓词（见 1.2 的叫停谓词一条），键函数的签名没有这两个口子（`crates/singlefs-checker-tier/src/crash_amplification.rs:359`、`:406`、`:16550`） | 判段的已判结论照认；录入块只认本机库里的，对方录的块不在本机库就当没录（2.2），要靠丁 | 一致（键）；重起复用靠丁 |
| 乙 | 领用账 | 账若只管「下一个发谁」、跳过仍由库里的键判（`crates/singlefs-checker-tier/src/crash_amplification.rs:2699`），不进键；账若在重起时被当成「这段做过了」的依据，就是 428 行的「只有整道或整格的标记而没有按判定的库」 | 同甲 | 条件一致；候选正文只写「发块方记一张领用账」（`research/prompts/_dyn-share-r1-body.md:30`），没写账的用途，规则没说账管不管跳过 |
| 丙 | 分工权重每趟换一份 | 不进：权重只在 `owner_of_block` 里，产出块号集合（`crates/singlefs-checker-tier/src/crash_amplification.rs:1315`） | 已判结论每次重分后要导入再重起才认（导入只导「不归本机」的块，见下 2.4 的导入归属一条）；录入块同甲 | 一致（键）；重起复用靠丁与每次重分后的导入 |
| 丁（并进） | 无：搬的行带着原来的键 | 不进：四种行的键见 2.1，没有主机 | 并进之后本机库里有，认 | 一致 |
| 丁（留库） | 第二台长期库 | 键同上；但多一份长期库 | 在第二台那份库里认，本机认不到 | 冲突，见下并排两句 |

冲突的并排两句（丁的留库做法）：

- 候选：「第二台的库不随树删、下一趟同一条命令送回去接着录」（`research/prompts/_dyn-share-r1-body.md:32`）。
- 现行规则：「第二台判出的块取回、按块键并进本机那一份崩溃判定库（第二台那一趟的树连同库跑完就删，不留长期库）」（`.claude/rules/implementation-workflow.md:66`）；kb：「每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。」（`.claude/kb/decisions/13-验证路线.md:430`）。

这是留库做法与现行规则、与已定项 17 字面的冲突；同一句规则也说明今天的现状：第二台的观察者行放在「会丢的位置」（428 行）。用户 2026-10-17 的话是要改这一点，改规则前留库做法与第 66 行冲突，并进做法不冲突。判哪一个由主 agent。

### 2.4 两处会让「按键认」落空的代码事实（候选没写，K2 要点出来）

- 导入归属一条：导入只导「不归本机」的块，归属用的是开跑时的公式：「（`is_mine` 为假）的块，每块连同两种判器与挂在这个点上的观察者的结论版本」（`crates/singlefs-checker-tier/src/crash_amplification.rs:19123` 起，导入函数的文档），循环里「if is_mine(this_block) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:19175`），调用处「Some(list) => import_the_planned_blocks_of_the_other_hosts(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:12739`）交的是 `selected`。动态分工下，对方实际判过、而按公式「归本机」的块不会被导入，随后 `pending_blocks` 的「if !is_mine(this_block) {」（`crates/singlefs-checker-tier/src/crash_amplification.rs:2696`）把它们当作本机的活、库里没有就重判。键没错，是进不来。所以甲、乙两个候选在收尾那一趟导入时必须拿到「这一趟实际的分工」（甲的最终前沿、乙的领用账），不能再用开跑公式；丙每个阶段结束都在该阶段权重下导入一次就够。这份「实际分工」是一个新的、不是内容的输入，它只能活在库外（驱动的运行目录），不能进库键（已定项 17 的 428 行）。
- 给定状态引擎一条：给定状态引擎的归属不是块序号（1.3 的给定状态引擎一条），甲在这个引擎上没有定义。

### 2.5 K2 的结论

- 三个分工候选在块键、观察者键、判器组键上**一致**：份、权重、方向、前沿只出现在遍历次序、叫停谓词和 `selected` 集合里，键函数的签名（`crates/singlefs-checker-tier/src/crash_amplification.rs:359`、`:406`、`:16550`）没有口子可以进去。
- 区别在两处：重起复用靠行在不在这台的库里（2.2），收尾导入靠实际分工（2.4）。丁（并进）让前者成立且不动键；丁（留库）与现行规则及已定项 17 字面冲突。
- 乙的领用账是唯一一个有可能违反 428 行的东西，条件是账被拿来决定「这段不用再做」。
- 推翻这一节的现象：任何候选的实现把份、权重、方向、前沿传进 `block_key_under`、`observer_bucket_key`、`observed_image_key_of_the_witness` 或 `ConclusionVersionsOfThePlan` 的参数表（`grep -n 'fn block_key_under\|fn observer_bucket_key\|fn observed_image_key' crates/singlefs-checker-tier/src/crash_amplification.rs` 看参数表）；或量到同一内容的块在两台各存一份、键不同（那说明键里混进了别的东西）。

## 3 K3　断点

### 3.1 第二台录的行今天到了哪、被谁丢的（现查）

- 录在第二台树里的库：「录的块和判的块都写进它树里的 crash-amplification-library/，随 --fetch 取回」（`scripts/crash/crash-amplification-two-hosts.sh:29`）。
- 取回落在本机的运行目录，不是临时目录：「fetch_directory="$run_directory/fetched"」（`scripts/crash/crash-amplification-two-hosts.sh:1566`），`run_directory` 在「run_directory="$crash_amplification_home/runs/$label"」（`scripts/crash/crash-amplification-two-hosts.sh:1311`）。驱动里没有删 `fetched` 的 `rm`（我 `grep -nE 'rm -r|rm -f|fetched'` 过全文件，非自证部分只有删停下文件与失败标记）。
- 导入时丢的是录入行：导入的点名表只有三样，「pub conclusions: Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>,」（`crates/singlefs-checker-tier/src/verdict_store.rs:1729`）、「pub paths: Vec<String>,」（`crates/singlefs-checker-tier/src/verdict_store.rs:1731`）、「pub verifier_blocks: Vec<(VerdictBlockKey, [u8; SHA256_DIGEST_BYTES])>,」（`crates/singlefs-checker-tier/src/verdict_store.rs:1733`）；整库导入 `import_from` 的文档只列「录入表、路径索引、事实表、核对表与判法单元指纹表逐行照抄」（`crates/singlefs-checker-tier/src/verdict_store.rs:3258`）。观察者四张表（基底、看过的恢复镜像、读集见证、录入块）在这两个导入函数的行区间里零命中：

```
$ awk 'NR>=3264 && NR<=3700 && /ObserverBlock|ObservedImage|ObserverFoundation|ObserverReadSetWitnesses/' crates/singlefs-checker-tier/src/verdict_store.rs | wc -l
0
```

  这与驱动文件头 30 行的话一致（上面 0.1 表里引过）。
- **一条要主 agent 现查的线索**：第二台那一份在被停时若走的是 TERM（`multi-host-run.sh` 的「第二台的停下之后先取回 --fetch 点名的路径（写到一半的产物不随树一起丢），」，`scripts/hosts/multi-host-run.sh:37`），库会先取回再删树。那么上一趟被停下的第二台约 7 小时录的行很可能还在本机盘 `<家目录>/runs/<标签>/fetched/crash-amplification-library/` 下（`$crash_amplification_home` 缺省是 `~/.local/share/singlefs/crash-amplification`）。这个目录在我的禁读范围里，**我没看**；要主 agent 跑 `ls -d ~/.local/share/singlefs/crash-amplification/runs/*/fetched/crash-amplification-library/CURRENT`。在，则并进做法的合并代码写好后可以一次性把那批行捞回来；不在（取回没成或被清），这一条作废。

### 3.2 三种情形 × 两种做法：丢什么、重起接到哪

先说三种「丢」的原因，都是代码：

- 被停是硬停，不是叫停：引擎里没有 TERM 处理（`grep -rnE 'SIGTERM|signal_hook|ctrlc|sigaction' crates/singlefs-checker-tier/src crates/singlefs-harness/src | wc -l` 输出 0），`multi-host-run.sh` 先 TERM、「至多等 30 秒，还在的 KILL」（`scripts/hosts/multi-host-run.sh:38`）。叫停文件才是软停：录入段每块前问一次（`crates/singlefs-checker-tier/src/crash_amplification.rs:17566`），GPU 判段「已经领的块判完、写进库再收尾（库里不留空洞）」（`crates/singlefs-checker-tier/src/crash_amplification.rs:3392`）。
- 一块的录入行落库在这一块整个录完之后：「.store_observer_block(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17662`）；块里真跑出来的恢复镜像和它所在桶的见证清单则是随真跑当场一批原子落盘：「.store_observed_image_with_the_witnesses_of_its_bucket(」（`crates/singlefs-checker-tier/src/crash_amplification.rs:17159`）。所以硬停丢的是在飞那一块的块行，它已经落盘的镜像行与见证行还在，重做那一块只剩逐状态对见证、不再真跑已看过的镜像（推的：没有量过对见证一遍的用时）。
- 取回只在第二台命令退出后或收到 TERM 后发生（`multi_host_fetch` 三处调用，见 1.2 的没有跑中通道一条）；连不上走「第二台那条命令没等到退出码」那一支（`scripts/hosts/multi-host-run.sh:365`）→ `multi_host_fail` 退 255 → EXIT 上只做删树（`scripts/hosts/multi-host-run.sh:158`），**不取回**。连不上的判定是「连续 60 次 ssh 不通」，轮询间隔缺省 5 s，即约 300 s（推的：60 × 5）。

| 情形 | 今天（对照） | 并进做法（取回时把第二台的录入四张表并进本机库） | 留库做法（第二台的库不随树删） |
|---|---|---|---|
| 停在录入段（叫停文件） | 第二台已录的块行全部不进本机库，下一趟第二台重录（上一趟丢约 7 小时的就是这个形态） | 丢 0 块：叫停在块边上停，块行全落盘；取回后合并，本机库快照送过去，两台下一趟都按键跳过（`crates/singlefs-checker-tier/src/crash_amplification.rs:17571`）。重起接到「计划里第一个库里没有块行也没有结论的块」，每趟从块 0 重扫，扫过的已录块只花两次点读（`blocks_already_recorded` 就是这个计数，`crates/singlefs-checker-tier/src/crash_amplification.rs:17574`） | 丢 0 块；第二台那份库留在原地，下一趟接着录，不用经本机 |
| 停在录入段（TERM 硬停） | 同上 | 丢在飞的块行，至多每台每观察者 1 块；镜像行与见证行已落盘；前提是取回成功 | 同左，且不依赖取回 |
| 停在判段 | 第二台判过的结论取回并导入（`scripts/crash/crash-amplification-two-hosts.sh:1641` 起的被停分支）；它**录了没判**的块行不进本机库，丢 | 判过的结论照旧导入；录了没判的块行也进本机库，下一趟谁判都行；丢在飞那几块的结论（硬停）或 0（叫停）；看过的镜像的池级 checker 结论（`ObservedImageJudgement` 行）不并进的话要重判这些镜像 | 第二台自己接着判；丢在飞的结论；没有跨机器的搬运 |
| 第二台掉线（连不上，或第二台断电） | 取回不发生（见上面讲取回时机的一条）：这一趟第二台**录与判**的行都不回来，掉线期间第二台还在跑也白跑；树能删就删了（`scripts/hosts/multi-host-run.sh:158`） | 同今天：并进的前提是取回，取回在掉线时不发生。想覆盖掉线，要跑中定期取，而跑着的库不能拷（「库开着的时候不能拷（RocksDB 的文件在变）」，`scripts/crash/crash-amplification-two-hosts.sh:1559`），引擎里没有检查点或只读副本的口子（`grep -rniE 'create_checkpoint\|Checkpoint::new\|open_as_secondary\|BackupEngine' crates/singlefs-checker-tier/src crates/singlefs-harness/src \| wc -l` 输出 0）；要引擎加一条「每隔一段把新录的行导成可取的一小批」的功能 | 库在第二台盘上、不在树里，掉线后恢复：丢在飞的块，其余都在；掉线期间本机用不到这些行 |

表后补三句：

- 叫停文件今天只在本机建（预热那一趟「warm_stop_file="$run_directory/stop-the-run-on-this-host"」，`scripts/crash/crash-amplification-two-hosts.sh:1480`，与导入那一趟），第二台没有送停下文件的通道；所以表里「叫停文件」一行对第二台是候选要新加的东西（丙必须有，甲的前沿越过也要）。
- 并进做法每趟本机库快照要整份拷再送给第二台（「库开着的时候不能拷…本机那一份开跑之前先照一份快照」，`scripts/crash/crash-amplification-two-hosts.sh:1559`；`--send` 在 `scripts/crash/crash-amplification-two-hosts.sh:1562`），并进后快照里多出第二台的录入行：13419 块至少 3.28 GiB（1.2 表里的下界，不含镜像行、不计压缩）；实际库大小没量（库目录在我的禁读范围）。
- 留库做法有两种读法：库留在第二台盘上（上表）；或库取回后留在本机、下一趟再送回（`fetched` 本来就不删）。后一种读法在掉线一行与并进做法相同，在其余行上等于并进做法少了合并、多了一份长期库，它的下一趟要把整份 `fetched` 当快照送去，而本机自己判第二台录的块时仍读不到这些行——所以不写成独立一栏。

### 3.3 并进时同一块两边都录了：怎么对账

今天没有这种情况：分工是划分，导入只导「不归本机」的块（2.4 的导入归属一条）。三个分工候选各自会造出它：甲在交叉点附近两边都越界（1.2 的重复块界）；乙在一次领块请求丢了、重领时；丙在换权重后同一块号落到另一台、且上一阶段的录入行没并进来。对账口径照驱动步骤 ④：「用例在导入之后断言对账为空、违例为空，它退 0 这一趟才算全绿。」（`scripts/crash/crash-amplification-two-hosts.sh:35`），而 ④ 的逐块规则是下面结论行那一条。分行写：

- **录入块行**（键＝块键＋观察者种类＋观察者版本，`crates/singlefs-checker-tier/src/verdict_store.rs:1305`）。今天的写是覆盖：「同一键再写就盖掉上一趟的。」（`crates/singlefs-checker-tier/src/verdict_store.rs:2879`），「是否已录」只看在不在（`crates/singlefs-checker-tier/src/crash_amplification.rs:17727`）。并进时本库有就跳过并数一个重复，与「在即已录」一致，**不比逐字节**。不比的理由是代码：块行里的去重键来自「这台库里先到的见证」，两台的见证清单不同，同一块的观察结果清单里去重键可能不同；镜像行带「persisted: Vec<bool>,」（`crates/singlefs-checker-tier/src/crash_amplification.rs:16666`），是首个产生该见证的那个状态的持久化标记，两台的首个状态不同，同一键下的镜像行字节可以不同。以上是推的，我没有拿两台各录同一块的行逐字节比过（库在禁读范围，且从没有过两台同块都录的情形）。推翻它：主 agent 让两台各录同一个块再逐字节比四张表的行，若全同，可以把对账收严成逐字节。
- **读集见证清单**（键＝桶键，一行存整张清单）：「观察者真跑了一张镜像：那张镜像的记录与它所在桶的整份见证清单一批原子地落盘——见证在、它引的镜像必在，被打断后重跑找到见证就读得到镜像。」（`crates/singlefs-checker-tier/src/verdict_store.rs:2851`）。合并必须按摘要取并集，不能整行覆盖：摘要决定镜像键（`crates/singlefs-checker-tier/src/crash_amplification.rs:16566`），`add` 对同摘要不重复添（「真跑之后添一条见证：桶里已有同一摘要的见证就不添」，`crates/singlefs-checker-tier/src/crash_amplification.rs:16445`），所以并集与先后无关；唯一写明的例外是「一个状态对得上两组里的见证」（`crates/singlefs-checker-tier/src/crash_amplification.rs:16465`），文档接着说它要观察者在两张镜像上读的段不同，而观察者读哪几段只由它读到的字节定时不会有。整行覆盖丢的是另一边的见证，只损失去重（下次对不上就真跑，得同一个内容键），不损失正确（推的）。
- **悬空引用不变量**：见证引镜像（见读集见证清单一条）；块行引镜像（缺则判段报「的录入块引的恢复镜像 {} 不在库里：先跑录入段」，`crates/singlefs-checker-tier/src/crash_amplification.rs:18700`）；镜像引基底（缺则报「观察者基底不在库里：先跑录入段」，`crates/singlefs-checker-tier/src/crash_amplification.rs:18721`）。并进的写序因此必须是：基底、镜像、见证清单、最后块行；这与今天单库的写序同向（镜像与见证一批，块行最后，`crates/singlefs-checker-tier/src/crash_amplification.rs:17662`），中途被停不留悬空。
- **结论行**（对账真有判别力的一层）：同一块两边都判了，走 ④ 的规则：块、违例、计数向量、恢复字、被抛弃根逐项相同才算已存，「stored.block == requested.block」（`crates/singlefs-checker-tier/src/verdict_store.rs:3841`）；不同报「DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {」（`crates/singlefs-checker-tier/src/verdict_store.rs:3849`）；只有在哪张卡判的不同算相同并留先存的一份（「StoredIdenticallyOnAnotherJudgingDevice,」，`crates/singlefs-checker-tier/src/verdict_store.rs:1722`）。同块只有一边判过则没有比较。所以口径是：**录入行按键在即录；结论行按 ④ 逐项相同**。同键异字节的录入行若真让两边判出不同结论，④ 会红，这是要的结果，不是要消的结果。

### 3.4 K3 的结论

- 并进做法：停在录入段不丢（叫停）或丢在飞的块行（硬停）；停在判段不再丢录了没判的块行；掉线与今天一样丢这一趟第二台的全部行，要覆盖只能让引擎加跑中导出的口子（零命中）。
- 留库做法：三种情形都最轻（掉线也只丢在飞的块），但它与第 66 行及已定项 17 字面冲突（2.3），且只要分工让块在两台之间换主（甲、乙、丙都会），仍要写同一套四表合并代码，两个方向都要；它省不掉合并，只多一份长期库。
- 两种做法共同的缺口：第二台没有叫停文件通道；四张表的合并函数 `crates/` 里没有（导入区间零命中，3.1）。
- 推翻：主 agent 现查到的 `fetched` 目录证明取回在被停时并不发生（那么并进做法在停下情形的前提也不成立）；或量到掉线在实践中不出现（那么留库做法省的那一行用不上）；或同键异字节在两台对拍中不出现（3.3 的录入块行一条收严）。

## 4 三条岔路：哪个候选在哪一格上占优（各一句，不下最终结论）

1. 分工机制（甲、乙、丙）：K1 的录入段甲与乙占优（界是「一块的用时加一个通信量」，秒级；依据叫停钩子每块问一次、可当前沿谓词用，`crates/singlefs-checker-tier/src/crash_amplification.rs:17566`，丙是「检查间隔加一次重起」，分钟级且重起秒数没量）；K2 三者持平（键函数签名没有口子，`crates/singlefs-checker-tier/src/crash_amplification.rs:359`）；K3 丙占优（它的每个重起点本来就是取回、合并、快照、送回的同步点，驱动已有快照与 `--send`，`scripts/crash/crash-amplification-two-hosts.sh:1562`，甲乙要在判段改向时让两个方向搬行，而 `multi-host-run.sh` 没有跑中取送文件的口子）；甲与乙之间 K1 分不出来，缺 `L_x` 与 `ρ` 两个没量的数。
2. 丁（并进或留库）：并进做法在 K2 占优（与「第二台判出的并进本机那一份」同向，`.claude/kb/decisions/13-验证路线.md:430`；留库做法与现行规则第 66 行字面冲突，2.3）并在 K3 的两种停下情形够用；留库做法只在 K3 的第二台掉线一行占优（掉线只丢在飞的块，并进做法丢这一趟的全部，取回在掉线时不发生，`scripts/hosts/multi-host-run.sh:365`）；而分工换主时留库做法也省不掉四表合并代码（3.4），所以它的占优只剩掉线这一格。
3. 判段动态与否：K1 的算术说不动态则判段闲置留着（分阶段量到的速度下第二台空 27.3 min，格内同速数下本机空 54.3 min，见 1.1 的同速一条与分阶段速度一条），理想份额录入 0.483、判段 0.335 说明同一份权重管不了两个阶段；动态判段里镜像判定一种不依赖录入行就能先动态（只用流与判表，`crates/singlefs-checker-tier/src/crash_amplification.rs:2099`），观察者结论的动态要先有丁（读不到录入块就报错，`crates/singlefs-checker-tier/src/crash_amplification.rs:18677`）；两种结论各占判段用时多少材料里没有，这一岔的够判条件是先量这个比例。

## 5 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 今天（对照） | 一致于题面 | 空闲 `\|T_P − T_L\|` 无块量级的界；格内数 54.3 min，分阶段 46.1 / 27.3 min（空闲方换边），题面 4.8 h（推的） |
| K1 甲，录入段 | 一致 | 先收尾一台空 `≤ t_b + L_x`，重复块 `≤ 2·⌈L_x·r ÷ B⌉`；`L_x` 候选没给 |
| K1 甲，判段 | 冲突 | 候选「录入段与判段各扫一遍」（`research/prompts/_dyn-share-r1-body.md:29`）对代码「的录入段那一块不在库里：先跑录入段」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18677`）：观察者结论只读本库录入块，判段扫到对方录的块就报错，除非丁把行搬到判它的那台 |
| K1 乙，录入段 | 一致 | `≤ S·t_b + ρ`；`S = 1` 时 0.73 s 起，`ρ` 没量 |
| K1 乙，判段 | 规则没说 | 候选正文没写发块方按不按「谁手里有这块的录入块」发；按位置发不均回来，不按位置发要搬行 |
| K1 丙 | 一致 | `≤ P + C_r`，`R ≤ ⌈T_dyn ÷ P⌉` 次重起；`C_r` 没量；判段权重可与录入权重分开设（驱动级） |
| K1 丁 | 规则没说 | 不减闲置；并进做法每趟快照多带第二台的录入行（13419 块至少 3.28 GiB） |
| K2 甲 | 一致 | 方向、前沿只进次序与叫停谓词；收尾导入要拿实际分工，不能用开跑公式（2.4） |
| K2 乙 | 规则没说 | 领用账只管「下一个发谁」则一致；拿来当「这段做过了」则撞 428 行「只有整道或整格的标记而没有按判定的库」 |
| K2 丙 | 一致 | 权重只产生块号集合；每个阶段结束要在该阶段权重下导入一次 |
| K2 丁，并进 | 一致 | 四张表的键都是内容键，合并不改键 |
| K2 丁，留库 | 冲突 | 候选「第二台的库不随树删、下一趟同一条命令送回去接着录」（`research/prompts/_dyn-share-r1-body.md:32`）对「第二台那一趟的树连同库跑完就删，不留长期库」（`.claude/rules/implementation-workflow.md:66`）与「每类判定一份库，固定在持久目录，不随一趟、不随判法换目录」（`.claude/kb/decisions/13-验证路线.md:430`） |
| K3 丁，并进，停在录入段 | 一致 | 叫停不丢，硬停丢在飞的块行；前提是取回成功 |
| K3 丁，并进，停在判段 | 一致 | 录了没判的块行也回来；丢在飞的结论 |
| K3 丁，并进，掉线 | 规则没说 | 候选没写掉线；取回不发生，这一趟第二台的行全丢，与今天相同 |
| K3 丁，留库 | 一致（K3 内） | 三种情形都只丢在飞的块；代价是 K2 的冲突与分工换主时仍要合并 |
| K3 同块双录对账 | 规则没说 | 录入行按键在即录、不比逐字节；结论行按 ④ 逐项相同，不同报错 |

## 6 要主 agent 现查的事（我复核不了的）

| 事 | 为什么我做不了 | 命令 |
|---|---|---|
| 第 6 条计划的真块数（0.2 三套数哪套对） | 本机那趟日志不在我的读范围里 | `grep -n 'CRASH_AMPLIFICATION_RECORDED' <那趟日志>` 看 `blocks=` |
| 上一趟被停时第二台的库有没有取回、还在不在 | 目录在崩溃放量库目录之下，被禁读 | `ls -d ~/.local/share/singlefs/crash-amplification/runs/*/fetched/crash-amplification-library/CURRENT` |
| 同键异字节（3.3 的录入块行一条） | 要读两份库的行 | 让两台各录同一个块，逐字节比四张观察者表的同键行 |
| `L_x`、`ρ`、`C_r`（K1 的三个没量的数） | 量 ssh 往返与拷树重编要碰多机配置里的私有值，且两台此刻在跑全量 | 各自单独量一次再回填 1.2 表 |
| 观察者结论占判段用时的比例 | 材料里没有 | 判段日志里分别累计两种结论的用时 |
| 本机库与第二台行的真实大小 | 库目录被禁读 | 本机 `du -sh` 库目录，不在我这里做 |

## 7 没做什么

- 没判 K4、K5、K6（攻方腿的格）；没替主 agent 采纳哪个候选；第 4 节只是各格上谁占优的一句话，不是结论。
- 没起 cargo、没跑测试、没起重活；没建仓副本、没建编译目录；没读 `~/.local/share/singlefs/crash-amplification/` 下的任何东西。
- 没写模型：K1 是纯算术，脚本 `~/.local/share/singlefs/work/dyn-share-r1-forward/k1.py` 与输出 `k1.out` 在草稿目录，不入库；不是需要钉绝对值断言的计数模型，也没有变异表。
- 没量：ssh 往返、拷树加重编的秒数、库的字节数、观察者结论的用时占比、两台同块行是否逐字节相同（见第 6 节）。
- 凡标「推的」的数与判断：0.3 的有效速度与系数、1.1 题面一条的动态理想（上界）、1.2 表里 `L_x`、`S` 的示例值、3.2 的约 300 s 掉线判定、3.3 的录入块行与见证清单两条里的同键异字节与覆盖后果。
- 行号随工作树漂：报告里的行号是写报告时现查的；背景材料与正文里的行号对不上的见 0.1。

## 8 各条结论的推翻条件汇总

- 1.1：块数不是 22365，或 `r` 随阶段、随点变到这几个数之外。
- 1.3：量到 `L_x`、`ρ` 后甲乙界的大小换边；或观察者结论占判段用时很小；或给定状态表另有稳定序号。
- 2.5：任何候选把份、权重、方向、前沿传进键函数的参数表；或同内容的块在两台键不同。
- 3.4：被停时 `fetched` 并不产生；或掉线在实践中不出现；或同键异字节不出现。
