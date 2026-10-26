# m4-gpugc-design-r2 攻方腿（Opus）报告（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 Q9 Q13 Q17 F7 -->

立场：假设正文第四节的读法与第一轮推的五条改法都是错的，造输入与序列去打穿。同一轮辩方腿的报告在禁读清单里，我没读。行号都在主工作区现查；开工快照 `research/prompts/m4-gpugc-design-r2-snapshot/sha256sums.txt` 21 行，`sha256sum -c` 21 行 OK。下文「卡 0」是 RTX 5090，「卡 1」是 RTX 5060 Ti（产物 `name=gpu_card` 两行）；候选 A、B、C、D 在产物里写成 `scan_checksum_only`、`scan_checksum_and_choice`、`every_stage`、`release_check_only`（登记修订 R3-3 的对照）。凡写「推的」都没量；凡写「量的」都贴了命令与输出。

## 复跑

模型全是读 E185 第 3 次重跑第一段产物的 python，外加一个本机 CRC 速度探针（C，gcc 编）。不编 Rust、不起 GPU。这一轮没有冻结副本，`--frozen` 给仓根即可（脚本只读 `$REPO_ROOT/research/results/` 里第一段产物，先核它的 sha256）。

```bash
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-gpugc-design-r2-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根> --work <新的空目录>
```

我这一次的原样输出（`--work` 是草稿目录里的 `rerun3`）：

```text
name=rerun_compare model=pivot identical=true
name=rerun_compare model=pivot_wait identical=true
name=rerun_compare model=readings identical=true
name=rerun_compare model=decompose identical=true
name=rerun_compare model=judged_points identical=true
name=rerun_compare model=identity_index identical=true
name=rerun_compare model=c_inputs identical=true
name=rerun_compare model=a_per_intent identical=true
name=rerun_compare model=ratio_end identical=true
name=rerun_compare model=round_robin identical=true
name=rerun_compare model=drift identical=true
name=rerun_compare model=foreground_wall identical=true
name=rerun_compare model=hardware_crc identical=true
name=rerun_compare model=combined identical=true
name=rerun_compare model=g8 identical=true
name=rerun_probe name=crc_speed implementation=software_eight_tables bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=6070085 bytes_per_second=2763917803
name=rerun_probe name=crc_speed implementation=sse42_instruction bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=1416664 bytes_per_second=11842762998
exit=0
```

探针那两行随机器状态变，不与 `out/` 比；`hardware_crc`、`combined` 两个模型读的是 `out/crc_speed.out` 那一份（软件八表 5903485 ns、SSE4.2 指令 1484834 ns，倍数 3.9759）。三次复跑的探针倍数是 4.40、4.48、4.28，都比我用的 3.9759 大，往对候选 A、C 有利的一侧偏。

模型目录每个文件的 sha256：

```text
e784b94749b99d1ff00e117970d63a838f2b173c45b157e000c1b7b76d34215c  research/prompts/m4-gpugc-design-r2-opus-model/a_per_intent.py
10a5a4754828c806a740c5701df286f90eb191404ea22e8341ca3a70673e36ca  research/prompts/m4-gpugc-design-r2-opus-model/c_inputs.py
8fe92ae3aa104b31eacb829fc6b1588bb7d2967f2841f708678e4753e1be2407  research/prompts/m4-gpugc-design-r2-opus-model/combined.py
c45e24bad58b7f401a11b3a059640c0ecea2a551c85fb3ffed1e634661df8d6f  research/prompts/m4-gpugc-design-r2-opus-model/crc_speed.c
8291303c2b00c67d398d8a31957cbb4def31c096e54de547df389f31c104630d  research/prompts/m4-gpugc-design-r2-opus-model/decompose.py
a0115530551e41ee3567e6f6dd98627933d7ec92d9417c33431dbdf032081955  research/prompts/m4-gpugc-design-r2-opus-model/drift.py
9ddede861950e58c6c257f9806770e78995a9eee761664c03512753076def80a  research/prompts/m4-gpugc-design-r2-opus-model/e185lib.py
7c037f29a001d6860f3326a2544cd549e018bfef9ca9e086d5680d64169e3c09  research/prompts/m4-gpugc-design-r2-opus-model/foreground_wall.py
01c9e64ff8657fbeabc6d257f31f9ea0a0607fcc6dd3625372ceb8bf369642b4  research/prompts/m4-gpugc-design-r2-opus-model/g8_background_loop.py
5cf41a2021b13c11a9aded06a9104abb5ef4348aaafb05e07ac84235b66bce51  research/prompts/m4-gpugc-design-r2-opus-model/hardware_crc.py
50ee53f73089b05bdca2d65a9b8be259a77dac456d3fdedf81dd6b5ed0c70cd0  research/prompts/m4-gpugc-design-r2-opus-model/identity_index.py
e853671b09fc6debc4624290bb5a8063de9542aa01334f572cae6d71bcad32c6  research/prompts/m4-gpugc-design-r2-opus-model/judged_points.py
36820e9f7e0796c6cdda85d90681238d9521283f74743e5f4bae56be29c53702  research/prompts/m4-gpugc-design-r2-opus-model/out/a_per_intent.out
45a527dfdb2c20c76bdd753112eb620716a0ece92567825e7a4887dcae595ebf  research/prompts/m4-gpugc-design-r2-opus-model/out/c_inputs.out
c5984b987dbb726032693b64b8d97fa88503f2ee43f3a5772b6e29fc43727448  research/prompts/m4-gpugc-design-r2-opus-model/out/combined.out
e7ec3e7b68399de813cbddc3d9fb2ffcfd0e63dd7fe7ba1f9f19eaf7ddf99719  research/prompts/m4-gpugc-design-r2-opus-model/out/crc_speed.out
1423df0b680599ccb69acd1967883232f69ffffacd444f02230faf422eb9a479  research/prompts/m4-gpugc-design-r2-opus-model/out/decompose.out
33a6e75c66e09fc83056a47681530175fcb7bb332162d16cde781a073498d3f4  research/prompts/m4-gpugc-design-r2-opus-model/out/drift.out
69b773d89dee3a72d44ab474937eeb2659c591b2b16f9f2691d46ac01e506bee  research/prompts/m4-gpugc-design-r2-opus-model/out/foreground_wall.out
cff1117a6090d6089df226d8e33df6c9c747057a06724c767ed62cefe7f7030b  research/prompts/m4-gpugc-design-r2-opus-model/out/g8.out
43263717592cc306c8cddeba35534548b7bbe67044ed6ce8293bcfce2193138a  research/prompts/m4-gpugc-design-r2-opus-model/out/hardware_crc.out
4951123c1a54ee957709e2f289aa169beacc0cb5a1dae5b42a44bdf96e684b5a  research/prompts/m4-gpugc-design-r2-opus-model/out/identity_index.out
429ecb6d01a4075b9cc0aaf3f2ba534f0e8f76fe47ee3afb9f5d47cae814beb8  research/prompts/m4-gpugc-design-r2-opus-model/out/judged_points.out
0f4768dcf7849fcd40e95eea3ac272175cc5b070ffa9b834b2730ca33cdfd527  research/prompts/m4-gpugc-design-r2-opus-model/out/pivot.out
d6b741aae5e27112318050f894979f3d84d8428cde809b4ff23cfa3b4b0c916e  research/prompts/m4-gpugc-design-r2-opus-model/out/pivot_wait.out
2d2a7d66629eae238c634f4e5c4c629a88317b8e5cc80fe58381245acc4034b2  research/prompts/m4-gpugc-design-r2-opus-model/out/ratio_end.out
bac668ac7f166799ee0b9fc4c70ec529e2e8fe20a56c252f0646e58caa46e47b  research/prompts/m4-gpugc-design-r2-opus-model/out/readings.out
407d6014054f66500322f8937d81afbeaf4f9e7d0e210ef145a7b092e062e77c  research/prompts/m4-gpugc-design-r2-opus-model/out/round_robin.out
93e6b63e451d9edd33b6b0528b0ca0ebddd71754eba761ca88eaec73a96dac90  research/prompts/m4-gpugc-design-r2-opus-model/pivot.py
f71f74b200ff0e0ae0f6b736f6e46bfa92d574288070e4237a0835e7f07675d6  research/prompts/m4-gpugc-design-r2-opus-model/pivot_wait.py
a4ae9b48379d6a68a8eb18f1f441a5ca2169b29f3bfaf0ca9404b5e4449094ef  research/prompts/m4-gpugc-design-r2-opus-model/ratio_end.py
9ed3c27597c06cfe31994cc20a71d677efcb6346d7c7d656803dbc13526dff07  research/prompts/m4-gpugc-design-r2-opus-model/readings.py
4c1ae6728203fdf67ba2a7a1a404ef14ccb95106d1fad4a2fca61a16f8cb2e6b  research/prompts/m4-gpugc-design-r2-opus-model/round_robin.py
823485dbd632e7d9f687ffab67021691bc3cc3134016fe96d3eb07c11edbc477  research/prompts/m4-gpugc-design-r2-opus-model/tools/rerun.sh
```

读的产物（sha256 与开工快照里登记的相同）：第一段 `research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`（`840b6835…`）、第二段 `research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out`（`a56a0572…`）、第三段 `research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out`（`31fd33a3…`）。

## 各格判定一览

| 格 | 判定 | 最要紧的一处（一句话） | 证据形态 |
|---|---|---|---|
| 改法一 按设备类型筛卡 | **打中** | 本机还有第三张显卡：AMD 集显（`79:00.0`，`amdgpu` 在用，`radeon_icd.json` 装着）；「只收独显与集显」把它收进来，E185 只量了两张 NVIDIA 卡 | 文件与 `lspci` 观测＋推的 |
| 改法二 对拍从这一版起算 | 不够（推的） | 小池测试流碰不到输入准备的分块边界（`wgpu` 默认上限一次派活至多 4194240 条线程、存储缓冲绑定 128 MiB） | 推的＋上游源码行 |
| 改法三 重挂时把不确定的条目交 CPU | **打中** | 照字面要知道「重挂之前哪几条不确定」，那份记录只在内存里，持久化又撞已定项 13 不改盘上字节那一句；做得出的只有「重挂后第一轮 N 取 1」 | 条款字面 |
| 改法四 挂载时已知答案自检 | 不够（推的，构造不出 crates 实例） | 固定输入只覆盖固定形状；挂载之后才出的错看不见 | 推的 |
| 改法五 GC 挪出调用线程 | **打中** | 见 G8：发布预算、空间准入、「不增全空段就不签下一条」三条保证会丢 | 代码行＋模型 g8 |
| 第四节透视的口径 | **打中多处** | 照今天的门槛，「A、B、D 没有一格不赔」只在「unstable 不算、看全部旋钮、间隔 0 也算」这一种读法下成立（CPU 模式换成指令 CRC 之后 A 处处赔，D 只剩卡 0 的 2 格不稳定）；「C 唯一成批不赔」把 C 的下界当了判定，C 与 A 的 c_rt 432 组逐字相同，C 赢只赢在门槛；第三段 8 行汇总有 7 行各项之和大于整次调用 | 产物现数（模型 pivot、readings、judged_points、decompose） |
| G1 | **打中** | C 稳赢的 3 个点全是 f = 0.75、P = 8192，赢在乙段，即 `region_scan.rs` 的线性查找；CPU 模式那一臂用的是软件八表 CRC 与线性查找，两处都换成现成做法，间隔 50 ms 下 C 一格不赔都没有 | 产物＋本机探针（量的）＋推的替换 |
| G2 | **打中** | 卡 0 的 9 条拟合 7 条 `fit_not_linear`，卡 1 只 2 条，跟着卡 0 五轮的漂移走；墙钟那 5.3–6.4 ms 的「固定开销」是每单元一条线程逐位算的串行深度 | 产物现数 |
| G3 | **反例** | 正推的「按窗口轮转」在两张卡上让 C 有 4 格由不赔翻成赔、23 格翻成不稳定；第三张卡（AMD）进来时，一张卡与几张卡在单一厂商编译器缺陷下判定不同 | 模型 round_robin＋推的序列 |
| G4 | 脆弱，没打中 | 第二段零次不一致的上界只罩两份 CRC 核；按上界算，1 TiB 一遍约 47.93 条不确定 | 算术 |
| G5 | **反例** | 「切到 cpu」请求在一批 GPU 计算途中到、之后这一批 GPU 出错：GC 停下，而第 4 条要的「人切回 cpu」已经发生在出错之前，进门读开关那一步再也不走 | 条款字面＋推的序列 |
| G6 | 改法够不着（推的） | 挪出之后「有 fsync 在等」看得到了，但只在批与批之间看；飞着的一次 GPU 提交与一整批都不能中途让 | 推的 |
| G7 | **反例（推的）** | checker 档把 `wgpu` 钉成 `=30.0.1`，产品的 GPU 包进同一个工作区只能用同一份 naga 与驱动；「与 checker 档不共用」只管源码，管不到共用的着色器编译链 | 代码行 |
| G8 | **打中三处** | 预算的 `.max(1)` 加定时兜底，空闲时连着的整理发布没有上限（模型：200 拍 213 次，线是 20）；GC 批的落点不过准入，挪出之后可以插进用户改动的「抬 F 再判」中间；「不增全空段就不签下一条」是一趟里的局部变量 | 代码行＋模型 g8 |

「打中」只是线索，按规则由主 agent 现查；每处都写了四句（分不分辨臂、判别子看不看得到、满足的是哪一个分句、改法还中不中）。我自己提的改法都只在我的模型上量过、被攻过零轮。

## 一、第一轮推的五条改法

### 改法一　按设备类型筛卡、只收独显与集显（第一轮攻方 G3-①，被攻过零轮）

反例：本机自己。第一轮列驱动清单时只列到 `research/prompts/m4-gpugc-design-r1-opus-output.md:240`：「nvidia_icd.json」，字母序排在它后面的两份没列出来。现查（只列 PCI 设备与文件，没起任何 GPU）：

```text
$ lspci -nn | grep -i -E 'vga|3d|display'
01:00.0 VGA compatible controller [0300]: NVIDIA Corporation Device [10de:2b85] (rev a1)
10:00.0 VGA compatible controller [0300]: NVIDIA Corporation Device [10de:2d04] (rev a1)
79:00.0 VGA compatible controller [0300]: Advanced Micro Devices, Inc. [AMD/ATI] Device [1002:13c0] (rev c1)
$ lspci -k -s 79:00.0
79:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Device 13c0 (rev c1)
	Subsystem: Micro-Star International Co., Ltd. [MSI] Device 7e59
	Kernel driver in use: amdgpu
	Kernel modules: amdgpu
$ ls /usr/share/vulkan/icd.d/
asahi_icd.json  gfxstream_vk_icd.json  intel_hasvk_icd.json  intel_icd.json  lvp_icd.json  nouveau_icd.json  nvidia_icd.json  radeon_icd.json  virtio_icd.json
```

（`ls` 原样是一行一个文件，这里并成一行。）

序列：用户照 D26 已定项 13 第 2 条（`.claude/kb/decisions/26-后台整理与放置回收.md:246`：「2. **用哪几张卡**：挂载参数 `background_gpu_devices` 是一张卡的列表（可以点几张）；不给就用本机全部显卡。」）不给卡表、开 `gpu`。改法一把软件适配器筛掉，但 AMD 集显（RADV 报的类型按推是集显，我没起 GPU 去看）照收。于是：

1. 参与的卡是三张，E185 第一段只量了两张（`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out` 第 130 行是 `name=gpu_card index=0`，第 557 行是 `index=1`，都是 NVIDIA、驱动 `595.99.02`）。第三张卡的每次提交主机 CPU、每字节那一部分都没有数；卡 1 的每字节部分已经是卡 0 的 7.9 倍（同一份产物第 3667 行 `per_mebibyte_milliseconds=0.487637`，卡 0 同格 0.061909），换一家驱动更不知道。
2. 用户按两张 NVIDIA 卡给了两个显存上限：参与的卡数是三，第 4 条照个数不等拒绝挂载（D26 第 247、248 行，第 3 条定个数、第 4 条定拒挂，指路不抄）。用户没改任何东西，只因 BIOS 里开着集显就挂不上；BIOS 里关掉集显又挂得上。
3. 集显与主机 CPU 共用封装功耗与内存带宽：它算的那一份会压低 CPU 别的线程的频率，第 7 条要算的收益（后台任务少占的主机 CPU）里这一块，发起提交那条线程的线程钟看不见（推的，没量）。

四句：分辨「按类型筛（含集显）」与「只收量过的卡 ／ 已验厂商独显」；要判的「这张卡用了赔不赔」在挂载那一刻系统看不到（产品里没有在挂载时量 c_rt 的路），这是判别子观测不到；满足的是第 2 条「全部显卡」的字面——字面要收集显，筛掉它就要用户改口；改法一在这一格上照中，正推的「已验厂商独显」碰得到，代价是把「全部」改窄，要用户答。

### 改法二　七·3 的对拍从这一版起算、流里带快照（第一轮攻方 G4-① 改法甲，零轮）

没构造出 crates 上的实例（产品的 GPU 输入准备还不存在），给的是结构上的序列，推的：

- GPU 的输入表要过 `wgpu` 的上限。默认上限在 `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/wgpu-types-30.0.1/src/limits.rs:441`：「max_storage_buffer_binding_size: 128 << 20, // (128 MiB)」，同一文件第 456 行：「max_compute_workgroups_per_dimension: 65535,」。工作组 64（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:90`：「@compute @workgroup_size(64)」）时一次派活至多 65535 × 64 = 4194240 条线程。
- 乙段每条映射一条线程时，映射超过 4194240 条（按 32 KiB 一个单元是 137436856320 字节，即 128 GiB 的池）就要二维派活或分几次派；映射表按每条 60 字节（推的布局）超过 128 MiB 约在 2236962 条（约 68 GiB 的池）就要分块绑定。checker 档起卡时按卡支持的上限要 limits（登记第三节那一行），产品若照做，分块的边界随卡变。
- 序列：准备那一步在分块边界上差一条（例如第 k·L 条漏传）。七·3 的对拍在测试流上跑，测试流的池远小于 68 GiB，表只有一块，两种模式各读这一版也逐项相同；用户的池过了边界，那一条在两份核里都看不到，两份一致、判「确认」。

四句：分辨「对拍跑在小池上」与「对拍覆盖到分块边界」；判别子（这一次准备有没有跨块）运行时看得到，测试里看不到；满足的是射程 ⚠️ 那句里共用的输入表准备那一样（第 256 行，指路）；改法二在这一格上照中，补法是测试里把上限人为调小，让小池也跨块（我提的，零轮）。

### 改法三　重挂与切换时把不确定的条目直接交 CPU（第一轮攻方 G4-③，零轮）

打中，形态是判别子观测不到。改法要系统在重挂之后知道「上一次挂载里哪几条不确定」；第 8 条的不确定只活在这一次挂载的内存里（连续 N 轮的计数住哪没写，第一轮已指出），重挂之后这份记录没了。要留下来只能写盘，撞 D26 已定项 13 定案末段改字节那一句（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 254 行，指路不抄）。

做得出的读法只有「重挂或切换之后的第一轮，N 取 1」。这一读法下：
- 同一个池、同一条确定性不一致的条目，长挂着的那一次要等 N 轮才交 CPU，刚重挂的那一次一轮就交；交 CPU 之前这一批不搬，于是同一段历史下搬迁的次序随重挂的时点变。七·4 要两种模式写出的块序列逐字节相同，凡有不确定的历史本来就做不到，这一条只是又多一个变量。
- 不重挂的序列它不管：今天 GC 在用户改动的调用里跑，一轮就是一次越线后的用户改动；文件系统闲着时一条确定性不一致的条目可以挂很久，R 一直排他。

四句：分辨「计数住内存」与「计数住盘上」；判别子（重挂之前哪几条不确定）重挂之后看不到；满足第 8 条与第 1 条（挂载参数只活在内存里）合起来；改法三照字面实现不了，改成「重挂后第一轮 N 取 1」碰得到第一轮那一格，碰不到「不重挂而轮次稀」那一格。

### 改法四　挂载与切换时用已知答案自检（第一轮攻方 G4-④，零轮）

构造不出 crates 上的反例：产品 GPU 代码不存在，没有可以喂的核。试过的形状与推的结论：
- 固定输入只覆盖固定的形状。第二段的单元只有 32768 与 16384 两种长度（登记第五节第二段那一条），E185 的池全是 32 KiB 数据单元、没有快照；乙、丙两段的核读单元头与映射，已知答案要按格式写，等于第三份实现，与两份核读同一份字段表——又回到射程 ⚠️ 那句里对规则的同一种误读那一样。
- 挂载之后才出的错（驱动重置、别的进程占卡，`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err:4`：「卡 0 上有别的计算进程 [76757]，等它（第 0 次看）」）自检看不到。
- 第二段两张卡在 16384 个单元上逐条相同（`research/results/e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out:315`：「E7RESULT name=both_cards_same_input cards=0,1 units=16384 identical=16384 cards_differ=false」）：同一家驱动下自检能抓的那一类，这一份产物没有出现过；它抓的是另一家驱动或另一版编译器，正好是本机那张没量过的 AMD 集显。

### 改法五　GC 挪出调用线程

见第十节 G8。

## 二、第四节透视的口径（关的第 2 条）

下面每个数都从第一段产物现数，命令在模型目录 `research/prompts/m4-gpugc-design-r2-opus-model/` 里起（`out/` 是复跑逐字相同的那一份）。

### 1. 正文那句「每种等法 14–38 格」「6–24 格」把今天的写法算进去了

登记把今天的写法（一份核、每次建缓冲）写成只报的真实基线：`research/prompts/e185-r3-prereg.md:234` 那一行里有「它是仓里现在真有的 GPU 提交，不是 GPU 模式（只有一份核），判定只报、不当第 1 行的读数」（那一行整行是真实基线一段，指路）。按候选 C、卡、等法、门槛种类数，含与不含 `form=today_single_kernel` 各一遍：

```text
$ grep 'candidate=every_stage' pivot_wait.out
name=pivot_wait include_today=False candidate=every_stage card=0 wait=blocking threshold_kind=attack_ledger losing=3 not_losing=23 unstable=10
name=pivot_wait include_today=False candidate=every_stage card=0 wait=blocking threshold_kind=machine_scaled losing=1 not_losing=29 unstable=6
name=pivot_wait include_today=False candidate=every_stage card=0 wait=sleep_100_microseconds threshold_kind=attack_ledger losing=1 not_losing=14 unstable=21
name=pivot_wait include_today=False candidate=every_stage card=0 wait=sleep_100_microseconds threshold_kind=machine_scaled not_losing=16 unstable=20
name=pivot_wait include_today=False candidate=every_stage card=0 wait=sleep_1_millisecond threshold_kind=attack_ledger losing=1 not_losing=22 unstable=13
name=pivot_wait include_today=False candidate=every_stage card=0 wait=sleep_1_millisecond threshold_kind=machine_scaled not_losing=28 unstable=8
name=pivot_wait include_today=False candidate=every_stage card=1 wait=blocking threshold_kind=attack_ledger losing=14 not_losing=13 unstable=9
name=pivot_wait include_today=False candidate=every_stage card=1 wait=blocking threshold_kind=machine_scaled losing=10 not_losing=18 unstable=8
name=pivot_wait include_today=False candidate=every_stage card=1 wait=sleep_100_microseconds threshold_kind=attack_ledger losing=19 not_losing=6 unstable=11
name=pivot_wait include_today=False candidate=every_stage card=1 wait=sleep_100_microseconds threshold_kind=machine_scaled losing=13 not_losing=15 unstable=8
name=pivot_wait include_today=False candidate=every_stage card=1 wait=sleep_1_millisecond threshold_kind=attack_ledger losing=17 not_losing=14 unstable=5
name=pivot_wait include_today=False candidate=every_stage card=1 wait=sleep_1_millisecond threshold_kind=machine_scaled losing=12 not_losing=19 unstable=5
name=pivot_wait include_today=True candidate=every_stage card=0 wait=blocking threshold_kind=attack_ledger losing=3 not_losing=29 unstable=16
name=pivot_wait include_today=True candidate=every_stage card=0 wait=blocking threshold_kind=machine_scaled losing=1 not_losing=38 unstable=9
name=pivot_wait include_today=True candidate=every_stage card=0 wait=sleep_100_microseconds threshold_kind=attack_ledger losing=1 not_losing=14 unstable=21
name=pivot_wait include_today=True candidate=every_stage card=0 wait=sleep_100_microseconds threshold_kind=machine_scaled not_losing=16 unstable=20
name=pivot_wait include_today=True candidate=every_stage card=0 wait=sleep_1_millisecond threshold_kind=attack_ledger losing=1 not_losing=22 unstable=13
name=pivot_wait include_today=True candidate=every_stage card=0 wait=sleep_1_millisecond threshold_kind=machine_scaled not_losing=28 unstable=8
name=pivot_wait include_today=True candidate=every_stage card=1 wait=blocking threshold_kind=attack_ledger losing=18 not_losing=15 unstable=15
name=pivot_wait include_today=True candidate=every_stage card=1 wait=blocking threshold_kind=machine_scaled losing=13 not_losing=24 unstable=11
name=pivot_wait include_today=True candidate=every_stage card=1 wait=sleep_100_microseconds threshold_kind=attack_ledger losing=19 not_losing=6 unstable=11
name=pivot_wait include_today=True candidate=every_stage card=1 wait=sleep_100_microseconds threshold_kind=machine_scaled losing=13 not_losing=15 unstable=8
name=pivot_wait include_today=True candidate=every_stage card=1 wait=sleep_1_millisecond threshold_kind=attack_ledger losing=17 not_losing=14 unstable=5
name=pivot_wait include_today=True candidate=every_stage card=1 wait=sleep_1_millisecond threshold_kind=machine_scaled losing=12 not_losing=19 unstable=5
```

正文的 38 只在含今天的写法时出现（卡 0、阻塞等、`machine_scaled`：不含 29、含 38）；卡 1 的 24 同样（不含 18、含 24）。不含时卡 0 是 14–29 格、卡 1 是 6–19 格。读数方向不变，口径错了一处。

### 2. 候选 C 的 `not_losing` 是下界，与 B 的 `not_losing_lower_bound` 同义，却没有这样标

A、B、C 交的是同一串提交（每次只交单元字节、两份 CRC 核），c_rt 是同一组数，只门槛不同：

```text
$ grep 'same_c_rt' decompose.out
name=same_c_rt_for_a_b_c groups=432 identical=432 different=0
```

登记自己写了 C 只到下界：`research/prompts/e185-r3-prereg.md:292` 判定那一格里有「有一种等法 `not_losing`：那一点 C 只到「下界不赔」（乙、丙的输入准备没量）」；第 294 行给 B 的理由是「B 的 `not_losing` 写成 `not_losing_lower_bound`（丙的输入是分配记录，展平它的 CPU 没量）」。C 含丙，又多一个乙（要映射表），没量的输入比 B 还多，产物却写成不带下界的 `not_losing`。所以「B 只有 4–14 格下界不赔、C 成批不赔」比的是两个口径不同的标签：两个都是下界，C 的门槛更高，格数自然多。

C 的乙、丙要的输入按每条映射 60 字节、每条分配记录 20 字节（推的布局）、照同一格拟合的每字节部分补上传那一份，再判（模型 c_inputs，推的）：

```text
$ grep 'name=c_with_the_inputs_of_scan_and_choice key=[01],' c_inputs.out
name=c_with_the_inputs_of_scan_and_choice key=0,1024,losing,losing cells=6
name=c_with_the_inputs_of_scan_and_choice key=0,1024,not_losing,not_losing cells=31
name=c_with_the_inputs_of_scan_and_choice key=0,1024,not_losing,unstable cells=2
name=c_with_the_inputs_of_scan_and_choice key=0,1024,unstable,unstable cells=69
name=c_with_the_inputs_of_scan_and_choice key=0,8192,not_losing,not_losing cells=94
name=c_with_the_inputs_of_scan_and_choice key=0,8192,not_losing,unstable cells=5
name=c_with_the_inputs_of_scan_and_choice key=0,8192,unstable,unstable cells=9
name=c_with_the_inputs_of_scan_and_choice key=1,1024,losing,losing cells=60
name=c_with_the_inputs_of_scan_and_choice key=1,1024,not_losing,not_losing cells=9
name=c_with_the_inputs_of_scan_and_choice key=1,1024,not_losing,unstable cells=9
name=c_with_the_inputs_of_scan_and_choice key=1,1024,unstable,losing cells=14
name=c_with_the_inputs_of_scan_and_choice key=1,1024,unstable,unstable cells=16
name=c_with_the_inputs_of_scan_and_choice key=1,8192,losing,losing cells=25
name=c_with_the_inputs_of_scan_and_choice key=1,8192,not_losing,losing cells=34
name=c_with_the_inputs_of_scan_and_choice key=1,8192,not_losing,not_losing cells=19
name=c_with_the_inputs_of_scan_and_choice key=1,8192,not_losing,unstable cells=14
name=c_with_the_inputs_of_scan_and_choice key=1,8192,unstable,losing cells=16
```

卡 1、P = 8192 上 67 格 `not_losing` 里有 34 格翻成 `losing`、14 格翻成 `unstable`；卡 0 每字节便宜，99 格里 94 格照旧。

### 3. `unstable` 的读法

`unstable` 是「5 轮不全相同」（登记第 286 行「5 轮相同写那个判定，不全相同写 `unstable`」那一句所在的一段）。把它按几种读法展开：

```text
$ cat readings.out
name=rounds_not_losing candidate=scan_checksum_only card=0 k0=108 k1=56 k2=40 k3=4 k4=8 k5=0
name=rounds_not_losing candidate=scan_checksum_only card=1 k0=216 k1=0 k2=0 k3=0 k4=0 k5=0
name=rounds_not_losing candidate=scan_checksum_and_choice card=0 k0=29 k1=33 k2=24 k3=48 k4=45 k5=37
name=rounds_not_losing candidate=scan_checksum_and_choice card=1 k0=140 k1=5 k2=12 k3=29 k4=16 k5=14
name=rounds_not_losing candidate=release_check_only card=0 k0=64 k1=82 k2=46 k3=18 k4=6 k5=0
name=rounds_not_losing candidate=release_check_only card=1 k0=216 k1=0 k2=0 k3=0 k4=0 k5=0
name=rounds_not_losing candidate=every_stage card=0 k0=6 k1=20 k2=4 k3=26 k4=28 k5=132
name=rounds_not_losing candidate=every_stage card=1 k0=85 k1=2 k2=6 k3=20 k4=18 k5=85
name=reading candidate=scan_checksum_only card=0 cells=216 majority_not_losing=12 median_not_losing=12 any_round_not_losing=108 last_round_not_losing=88
name=reading candidate=scan_checksum_only card=1 cells=216 majority_not_losing=0 median_not_losing=0 any_round_not_losing=0 last_round_not_losing=0
name=reading candidate=scan_checksum_and_choice card=0 cells=216 majority_not_losing=130 median_not_losing=130 any_round_not_losing=187 last_round_not_losing=162
name=reading candidate=scan_checksum_and_choice card=1 cells=216 majority_not_losing=59 median_not_losing=59 any_round_not_losing=76 last_round_not_losing=64
name=reading candidate=release_check_only card=0 cells=216 majority_not_losing=24 median_not_losing=24 any_round_not_losing=152 last_round_not_losing=108
name=reading candidate=release_check_only card=1 cells=216 majority_not_losing=0 median_not_losing=0 any_round_not_losing=0 last_round_not_losing=0
name=reading candidate=every_stage card=0 cells=216 majority_not_losing=186 median_not_losing=186 any_round_not_losing=210 last_round_not_losing=195
name=reading candidate=every_stage card=1 cells=216 majority_not_losing=123 median_not_losing=123 any_round_not_losing=131 last_round_not_losing=123
name=round_trend card=0 wait=blocking points=36 median_c_rt_by_round_ms=0.2880,0.2197,0.3236,0.1642,0.1404
name=round_trend card=0 wait=sleep_100_microseconds points=36 median_c_rt_by_round_ms=0.2918,0.3018,0.4671,0.3440,0.2102
name=round_trend card=0 wait=sleep_1_millisecond points=36 median_c_rt_by_round_ms=0.2432,0.2215,0.3419,0.2033,0.1501
name=round_trend card=1 wait=blocking points=36 median_c_rt_by_round_ms=0.3479,0.4117,0.3692,0.4008,0.3200
name=round_trend card=1 wait=sleep_100_microseconds points=36 median_c_rt_by_round_ms=0.4414,0.4434,0.4330,0.5134,0.4704
name=round_trend card=1 wait=sleep_1_millisecond points=36 median_c_rt_by_round_ms=0.4108,0.4044,0.3561,0.4789,0.4095
```

- 按多数轮（≥ 3 轮不赔）或 5 轮中位读：A 在卡 0 有 12 格不赔，D 在卡 0 有 24 格不赔，B 在卡 0 有 130 格、卡 1 有 59 格（下界）。「A、B、D 没有一格不赔」在这一读法下翻面；卡 1 上 A、D 的 216 格 5 轮全赔，哪种读法都不翻。
- 按最后一轮读：A 在卡 0 有 88 格、D 有 108 格不赔。最后一轮读法有它的理由，见下一条。

### 4. 卡 0 的五轮不是同一个分布的五次抽样：c_rt 随轮次往下走

同一格五轮的 c_rt，按轮次（时间先后）比：

```text
$ cat drift.out
name=round_drift card=0 cells=72 round5_over_round1_median=0.643 round5_below_round1=64 floor_round5_over_round1_median=0.655 max_over_min_of_five_rounds_median=2.966 max_over_min_maximum=6.650 spearman_with_round_median=-0.50 spearman_at_most_minus_0_7=23 calibration_ratio_min=0.8587 calibration_ratio_max=1.0972
name=round_drift card=1 cells=72 round5_over_round1_median=0.992 round5_below_round1=44 floor_round5_over_round1_median=0.996 max_over_min_of_five_rounds_median=1.293 max_over_min_maximum=3.158 spearman_with_round_median=0.10 spearman_at_most_minus_0_7=2 calibration_ratio_min=0.8355 calibration_ratio_max=1.0327
```

卡 0 上 72 格里 64 格第 5 轮比第 1 轮便宜，中位便宜到 0.643 倍，五轮最贵与最便宜差中位 2.966 倍、最大 6.650 倍；同一时段每格前后的校准比只在 0.8587–1.0972。卡 1（在卡 0 之后量）没有走势（中位 0.992 倍）。一个具体的格：

`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out` 第 149 行（第 1 轮）`thread_cpu_median_nanoseconds=192560.000000`、`floor_thread_cpu_median_nanoseconds=151871.000000`，第 553 行（第 5 轮）`thread_cpu_median_nanoseconds=60910.000000`、`floor_thread_cpu_median_nanoseconds=39285.000000`，两行的等待挂钟都约 5.4 ms、校准比 1.059529 与 0.917904。

段末校准也说机器在这一段里变了：

`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:8004`：「E7RESULT name=calibration_baseline segment=host_cpu_per_submission position=end runs=21 median_nanoseconds=5493404.0 chase_median_nanoseconds=102919583.000000 anchors_hold=true ratio_of_the_end_to_the_start=0.876478 drifted=true」

读法：作废谓词用的校准是一段 16 MiB 的 CRC 忙算，量的是 CPU 算得快不快；一次提交的主机 CPU 里大头是 c_floor（拷字节、调驱动、读回比对），对频率爬升、缓存冷热、别的进程抢内存带宽的反应与忙算不同。校准看不见 c_floor 的三倍漂移，所以「机器被别的活挤」那一条作废护不住这一段。卡 0 的 `unstable` 多半是时间走势，不是随机抖动；`readings.out` 里「最后一轮读」与「多数读」差这么多，就是这个走势。起跑之前卡 0 上有别的计算进程（`.err` 第 4 行，改法四那一段引过），段内本机还有别的会话在编译（我这一次 `ps` 也看到别的会话的 `cargo build` 在跑，见文末「这条腿自己的限度」），哪一样造成走势，这一份产物分不出（推的）。

### 5. 按登记 J4 逐点读：A、D 在卡 0 上不是「出局」，是「不当已判」；C 在卡 1 上一个点都没判

登记 J4：`research/prompts/e185-r3-prereg.md:415` 那一行里有「那一点在那个旋钮上记「随 <旋钮> 翻面」，第 1 行在那一点不当已判」。把每个 (f, P, K) × 候选 × 卡跨输入形态、等法、间隔、门槛机器的判定合起来：

```text
$ grep 'selection=all_knobs\|selection=gap_50_only' judged_points.out
name=judged_points selection=all_knobs candidate=scan_checksum_only card=0 points=12 flips_with_a_knob=12
name=judged_points selection=all_knobs candidate=scan_checksum_only card=1 points=12 losing=12
name=judged_points selection=all_knobs candidate=scan_checksum_and_choice card=0 points=12 flips_with_a_knob=12
name=judged_points selection=all_knobs candidate=scan_checksum_and_choice card=1 points=12 flips_with_a_knob=10 losing=2
name=judged_points selection=all_knobs candidate=release_check_only card=0 points=12 flips_with_a_knob=12
name=judged_points selection=all_knobs candidate=release_check_only card=1 points=12 losing=12
name=judged_points selection=all_knobs candidate=every_stage card=0 points=12 flips_with_a_knob=9 not_losing=3
name=judged_points selection=all_knobs candidate=every_stage card=1 points=12 flips_with_a_knob=12
name=judged_points selection=gap_50_only candidate=scan_checksum_only card=0 points=12 flips_with_a_knob=4 losing=8
name=judged_points selection=gap_50_only candidate=scan_checksum_only card=1 points=12 losing=12
name=judged_points selection=gap_50_only candidate=scan_checksum_and_choice card=0 points=12 flips_with_a_knob=10 unstable=2
name=judged_points selection=gap_50_only candidate=scan_checksum_and_choice card=1 points=12 losing=12
name=judged_points selection=gap_50_only candidate=release_check_only card=0 points=12 flips_with_a_knob=8 losing=4
name=judged_points selection=gap_50_only candidate=release_check_only card=1 points=12 losing=12
name=judged_points selection=gap_50_only candidate=every_stage card=0 points=12 flips_with_a_knob=5 not_losing=5 unstable=2
name=judged_points selection=gap_50_only candidate=every_stage card=1 points=12 flips_with_a_knob=5 losing=7
```

- 全部旋钮一起看：A、D 在卡 0 上 12 个点全「随旋钮翻面」，照 J4 不当已判；卡 1 上 12 个点全 `losing`。C 在卡 0 上只有 3 个点跨全部旋钮不赔（`identity_index.out` 头三行：都是 f = 0.75、P = 8192），9 个点翻面；C 在卡 1 上 12 个点全翻面。
- 正文的透视按「候选 × 卡 × 等法 × 门槛种类」数格，把翻面的点拆成几格各自算，读不出 J4 的「不当已判」。

### 6. 间隔：0（连发）不是产品会有的形态，产品的间隔不短于一批的 CPU

登记自己写间隔 50 ms 是「产品里两批之间隔着一批的 CPU」（`research/prompts/e185-r3-prereg.md:55` 那一行里有「0（连发）与 50 ms（产品里两批之间隔着一批的 CPU，推的）」）。第三段量到的一批 CPU：

`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out:149` 是 P = 1024、f = 0.25 那一点的 `scope=batches` 汇总，`call_cpu_nanoseconds_median=17993662.000000`；第 1253 行是 P = 8192、f = 0.75，`call_cpu_nanoseconds_median=338405211.000000`。

所以两次提交之间至少隔 18–338 ms；今天的形态下一次越线后的用户改动只跑至多一批，间隔就是两次越线的用户改动之间的时间。间隔 0 的那 2/3 格在产品里不会出现。间隔是翻得最多的旋钮：

`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:5908`：「E7RESULT name=knob_flip_summary quantity=host_cpu_per_submission knob=gap clause=flips_with_a_knob pairs=700 flipped=345」

只看间隔 50 ms（只量了共用输入）：

```text
$ grep 'label=thread_gap_50 ' pivot.out
name=pivot label=thread_gap_50 candidate=scan_checksum_only card=0 losing=66 unstable=6
name=pivot label=thread_gap_50 candidate=scan_checksum_only card=1 losing=72
name=pivot label=thread_gap_50 candidate=scan_checksum_and_choice card=0 losing=26 not_losing_lower_bound=2 unstable=44
name=pivot label=thread_gap_50 candidate=scan_checksum_and_choice card=1 losing=72
name=pivot label=thread_gap_50 candidate=release_check_only card=0 losing=52 unstable=20
name=pivot label=thread_gap_50 candidate=release_check_only card=1 losing=72
name=pivot label=thread_gap_50 candidate=every_stage card=0 losing=6 not_losing=35 unstable=31
name=pivot label=thread_gap_50 candidate=every_stage card=1 losing=59 not_losing=3 unstable=10
```

C 在卡 1 上 72 格里 59 格赔、3 格不赔；卡 0 上 35 格不赔。间隔只量了两点，50 ms 已在产品最短间隔（18 ms）之上、远在 P = 8192 的 338 ms 之下；从 0 到 50 ms 卡 0 的固定部分由 0.193968 涨到 0.369545 ms（同一份产物第 3658、3659 行，`c_inputs.out` 头两行抄着），卡 1 的每字节部分由 0.487637 涨到 1.463544 ms/MiB（第 3667、3668 行）。更长的间隔没量，是扫描区间的端点（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@扫描出来的极值落在扫描区间的端点，就不是量到的数`）。

### 7. 门槛种类：`machine_scaled` 用的是段开头的 ρ

`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:121`：「E7RESULT name=machine_ratio local_baseline_nanoseconds=6267586.0 peer_baseline_nanoseconds=5224477.000000 ratio=1.199658 far_from_one=false status=measured」

段末本机校准 5493404 ns（上面第 8004 行），换算成 ρ 是 1.051474。卡 1 在后半段量，用段末的 ρ 再判 `machine_scaled` 那几行：

```text
$ cat ratio_end.out
name=machine_ratio_at_both_ends rho_start=1.199658 rho_end=1.051474
name=machine_scaled_with_rho_end candidate=every_stage card=0 judgement_with_rho_start=not_losing judgement_with_rho_end=unstable cells=10
name=machine_scaled_with_rho_end candidate=every_stage card=0 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=3
name=machine_scaled_with_rho_end candidate=every_stage card=1 judgement_with_rho_start=not_losing judgement_with_rho_end=unstable cells=11
name=machine_scaled_with_rho_end candidate=every_stage card=1 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=8
name=machine_scaled_with_rho_end candidate=release_check_only card=0 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=4
name=machine_scaled_with_rho_end candidate=scan_checksum_and_choice card=0 judgement_with_rho_start=not_losing judgement_with_rho_end=unstable cells=9
name=machine_scaled_with_rho_end candidate=scan_checksum_and_choice card=0 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=6
name=machine_scaled_with_rho_end candidate=scan_checksum_and_choice card=1 judgement_with_rho_start=not_losing judgement_with_rho_end=unstable cells=4
name=machine_scaled_with_rho_end candidate=scan_checksum_and_choice card=1 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=16
name=machine_scaled_with_rho_end candidate=scan_checksum_only card=0 judgement_with_rho_start=unstable judgement_with_rho_end=losing cells=11
```

C 在两张卡上 21 格由 `not_losing` 掉成 `unstable`、11 格由 `unstable` 掉成 `losing`。门槛机器这一旋钮本来就翻 214 对（`knob=threshold_machine … pairs=1342 flipped=214`，同一份产物的 `knob_flip_summary`），ρ 取哪一刻又让它多翻一些。

### 8. 只量了两张卡

两张卡同一家、同一版驱动、都在 PCIe 上；本机第三张显卡（AMD 集显）没量（改法一）。两张卡之间主机 CPU 每字节差 7.9 倍、间隔 50 ms 时差 23.6 倍（1.463544 ÷ 0.061993），而「主机 CPU 每字节」按理与卡无关——差在驱动怎么把字节送过去（推的：卡 1 那条路像是主机自己拷进可映射内存）。这一项是卡与平台的性质，别的机器上不能由这两张卡外推。判别卡与卡之间是不是同一种拷法，这一份产物没有量（每次提交只报线程钟与挂钟，不报拷贝走哪条路）。

### 9. 提交的字节与真实 GC 一批对不对得上

- 字节本身：第一段每次提交交的是「造池时写的一个真数据单元连放 512 份」（`research/prompts/e185-r3-prereg.md:485` 那一行里的原话），E185 的池只有 32 KiB 数据单元、没有快照、没有索引节点与打包单元。单元数（每批 R 里剩下的那几个）与代码里每批之前都重扫 R 对得上；单元的种类与长度对不上真实的池。
- 候选 C 的乙、丙交的不是单元字节，是映射与分配记录，没进 c_rt（第 2 条）。
- 候选 D 每批交「这一批搬的单元两块盘各一份」（登记第一节「一个点的提交序列」那一格），只对两副本的池成立，单盘池减半。

### 10. 岔路单第 1 行的「5368 行」

产物里 `name=host_cpu_per_submission_point` 是 2684 行；5368 是 `judgement=` 与 `floor_judgement=` 两个字段合起来的个数，1529 是 1054 个 `judgement=unstable` 加 475 个 `floor_judgement=unstable`：

```text
$ F=research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out
$ grep -c 'host_cpu_per_submission_point' $F
2684
$ grep 'name=host_cpu_per_submission_point' $F | grep -c ' judgement=unstable'
1054
$ grep 'name=host_cpu_per_submission_point' $F | grep -c 'floor_judgement=unstable'
475
```

### 11. 第三段的拆分自己对不上：各项之和大于整次调用

正文第四节只引了第三段的区域扫描份额，说「读这一版、克隆两项的份额见同一份产物同几行」。那几行：

`research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out:150`：「E7RESULT name=function_split_summary fraction=0.25 pool_data_units=1024 batch_units=8 scope=issue rounds=5 call_cpu_nanoseconds_median=10022201.000000 call_share_median=1.000000 clone_cpu_nanoseconds_median=2544443.000000 clone_share_median=0.250856 read_the_version_cpu_nanoseconds_median=13352238.000000 read_the_version_share_median=1.330892 region_scan_cpu_nanoseconds_median=103210.000000 region_scan_share_median=0.010215 choose_cpu_nanoseconds_median=365262.000000 choose_share_median=0.036336 remainder_cpu_nanoseconds_median=-6317112.000000 remainder_share_median=-0.630513」

```text
$ G=research/results/e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out
$ grep 'name=function_split_summary' $G | grep -c 'remainder_cpu_nanoseconds_median=-'
7
$ grep 'name=function_split_summary' $G | grep -o 'read_the_version_share_median=[0-9.]*' | tr '\n' ' '
read_the_version_share_median=0.744810 read_the_version_share_median=1.330892 read_the_version_share_median=1.087456 read_the_version_share_median=2.082164 read_the_version_share_median=0.961849 read_the_version_share_median=1.255026 read_the_version_share_median=1.390565 read_the_version_share_median=1.803361 
$ grep -c 'name=function_split ' $G; grep 'name=function_split ' $G | grep -c 'remainder_cpu_nanoseconds=-'
60
54
```

8 行汇总里 7 行「发布与其余」是负的，6 行「读这一版」一项就超过整次调用；60 行逐次记录里 54 行余数为负。重放的读这一版（装置自己那一份钉住槽那一圈）比 crates 在调用里做的那一份贵得多，量的不是调用里的那一段。这一段的 verdict 照样是 `positive_control_failures=0`：阳性对照只核了 2 ms 自旋加得进去，没核「各项之和不超过整次」。岔路单第 10 行据区域扫描份额判了「够判」；区域扫描是单独重放的（`scan_region`），份额本身不受这一处牵连；读这一版与克隆两项的份额，这一份产物给不了。
### 透视的岔路（交用户定；不写倾向）

| 候选（读法） | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 读法甲 | `unstable` 不算判定；一格一格数；间隔 0 与 50 ms 都算 | 同一格五轮在时间上是否可交换（卡 0 第 5 轮 ÷ 第 1 轮中位 0.643） | 卡 0 重量一段，轮次打乱到不同时刻、每轮前后另做一次与提交同形的校准（拷字节、调驱动），走势消失 |
| 读法乙 | 照登记 J4 逐点：任何旋钮翻面的点不当已判 | 同上；外加哪些旋钮该算（间隔 0 算不算） | 用户或主 agent 定间隔 0 是不是产品形态 |
| 读法丙 | 只读间隔不短于产品一批 CPU 的格 | 间隔 ≥ 18 ms（P = 1024）与 ≥ 338 ms（P = 8192）两档上的 c_rt | 第一段补量间隔 100、300、1000 ms（今天只有 0 与 50 ms） |

## 三、G1　GPU 上算哪几段

### G1-①　C 赢在哪一段：乙段，就是区域扫描里那两处线性查找

A、B、C 的门槛拆成甲（CRC）、丙（选段打分）、乙（扫描增量）三段（`decompose.py` 用 `name=candidate_threshold` 现减：甲 = T_A，丙 = T_B − T_A，乙 = T_C − T_B）：

```text
$ grep 'threshold_split' decompose.out
name=threshold_split fraction=0.25 pool_data_units=1024 batch_units=4 T_A=0.068814 T_B=0.192802 T_C=0.215934 T_D=0.082975 checksum_part=0.068814 choice_part=0.123988 scan_increment_part=0.023132 scan_increment_share_of_T_C=0.1071 submissions=3
name=threshold_split fraction=0.25 pool_data_units=1024 batch_units=8 T_A=0.082310 T_B=0.268607 T_C=0.297882 T_D=0.165240 checksum_part=0.082310 choice_part=0.186297 scan_increment_part=0.029275 scan_increment_share_of_T_C=0.0983 submissions=2
name=threshold_split fraction=0.25 pool_data_units=1024 batch_units=16 T_A=0.082370 T_B=0.268551 T_C=0.295256 T_D=0.165491 checksum_part=0.082370 choice_part=0.186181 scan_increment_part=0.026705 scan_increment_share_of_T_C=0.0904 submissions=2
name=threshold_split fraction=0.25 pool_data_units=8192 batch_units=4 T_A=0.068767 T_B=0.197514 T_C=0.354802 T_D=0.082765 checksum_part=0.068767 choice_part=0.128747 scan_increment_part=0.157288 scan_increment_share_of_T_C=0.4433 submissions=3
name=threshold_split fraction=0.25 pool_data_units=8192 batch_units=8 T_A=0.082446 T_B=0.275757 T_C=0.468612 T_D=0.164871 checksum_part=0.082446 choice_part=0.193311 scan_increment_part=0.192855 scan_increment_share_of_T_C=0.4115 submissions=2
name=threshold_split fraction=0.25 pool_data_units=8192 batch_units=16 T_A=0.082470 T_B=0.275326 T_C=0.469108 T_D=0.165301 checksum_part=0.082470 choice_part=0.192856 scan_increment_part=0.193782 scan_increment_share_of_T_C=0.4131 submissions=2
name=threshold_split fraction=0.75 pool_data_units=1024 batch_units=4 T_A=0.159201 T_B=0.212104 T_C=0.263739 T_D=0.082572 checksum_part=0.159201 choice_part=0.052903 scan_increment_part=0.051635 scan_increment_share_of_T_C=0.1958 submissions=7
name=threshold_split fraction=0.75 pool_data_units=1024 batch_units=8 T_A=0.185948 T_B=0.278487 T_C=0.338034 T_D=0.164921 checksum_part=0.185948 choice_part=0.092539 scan_increment_part=0.059547 scan_increment_share_of_T_C=0.1762 submissions=4
name=threshold_split fraction=0.75 pool_data_units=1024 batch_units=16 T_A=0.192774 T_B=0.315202 T_C=0.376482 T_D=0.247601 checksum_part=0.192774 choice_part=0.122428 scan_increment_part=0.061280 scan_increment_share_of_T_C=0.1628 submissions=3
name=threshold_split fraction=0.75 pool_data_units=8192 batch_units=4 T_A=0.159751 T_B=0.214600 T_C=0.565853 T_D=0.082699 checksum_part=0.159751 choice_part=0.054849 scan_increment_part=0.351253 scan_increment_share_of_T_C=0.6207 submissions=7
name=threshold_split fraction=0.75 pool_data_units=8192 batch_units=8 T_A=0.185739 T_B=0.280979 T_C=0.688038 T_D=0.164948 checksum_part=0.185739 choice_part=0.095240 scan_increment_part=0.407059 scan_increment_share_of_T_C=0.5916 submissions=4
name=threshold_split fraction=0.75 pool_data_units=8192 batch_units=16 T_A=0.192428 T_B=0.320865 T_C=0.721447 T_D=0.247267 checksum_part=0.192428 choice_part=0.128437 scan_increment_part=0.400582 scan_increment_share_of_T_C=0.5552 submissions=3
```

- P = 1024 时乙只占 T_C 的 9.0%–19.6%，C 比 B 多出的那一点余量很小；P = 8192 时乙占 41.2%–62.1%，而且随 P 涨（f = 0.75、K = 4：0.051635 → 0.351253 ms）。例：`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:93`：「E7RESULT name=candidate_threshold fraction=0.75 pool_data_units=8192 batch_units=8 candidate=every_stage saved_per_intent_milliseconds=2.752154 submissions_per_intent=4 threshold_milliseconds=0.688038」，同一份第 92 行：「E7RESULT name=candidate_threshold fraction=0.75 pool_data_units=8192 batch_units=8 candidate=scan_checksum_and_choice saved_per_intent_milliseconds=1.123916 submissions_per_intent=4 threshold_milliseconds=0.280979」。
- C 跨全部旋钮都不赔的 3 个点正好是乙最大的三个（f = 0.75、P = 8192、K = 4、8、16，卡 0；第二节第 5 条）。
- 随 P 涨的那一部分是每条记录在全部映射条目上线性找：`crates/singlefs-core/src/region_scan.rs:288`：「.filter(|entry| entry.points_at(record.device, record.slot))」，同一文件第 311 行：「.any(|entry| entry.key == header.mapping_key);」。第一轮判决第三节第 3 条把它登记成 CPU 侧可修的延后项，岔路单第 10 行问「并进七·3 ／ 不并」。

所以第 1 行（GPU 上算哪几段）与第 10 行（按身份查）不独立：第 10 行选「并」，CPU 模式那一侧的乙不再随 P 涨，C 在 P = 8192 赢的那一块跟着没了。把 P = 8192 的乙换成同 f、K 的 P = 1024 值（推的：按身份查之后乙至多是 P = 1024 的线性查找那么多，这样换对 C 有利）再判：

```text
$ grep 'identity_index_pushed' identity_index.out
name=identity_index_pushed card=0 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=losing cells=3
name=identity_index_pushed card=0 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=not_losing cells=37
name=identity_index_pushed card=0 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=unstable cells=59
name=identity_index_pushed card=0 pool_data_units=8192 judgement_as_measured=unstable judgement_with_the_scan_increment_of_1024=losing cells=3
name=identity_index_pushed card=0 pool_data_units=8192 judgement_as_measured=unstable judgement_with_the_scan_increment_of_1024=unstable cells=6
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=losing judgement_with_the_scan_increment_of_1024=losing cells=25
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=losing cells=18
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=not_losing cells=18
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=not_losing judgement_with_the_scan_increment_of_1024=unstable cells=31
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=unstable judgement_with_the_scan_increment_of_1024=losing cells=14
name=identity_index_pushed card=1 pool_data_units=8192 judgement_as_measured=unstable judgement_with_the_scan_increment_of_1024=unstable cells=2
```

卡 0、P = 8192 上 99 格 `not_losing` 只剩 37 格，卡 1 上 67 格只剩 18 格。

### G1-②　CPU 模式那一臂的 CRC 是软件八表，换成 SSE4.2 指令快 3.98 倍

第 7 条的收益是 CPU 模式下的 CPU 时间减 GPU 模式下的 CPU 时间（第 251 行，我的转述）。CPU 模式今天的 CRC 是 `crates/singlefs-core/src/checksum.rs:43`：「pub fn crc32_castagnoli(bytes: &[u8]) -> u32 {」，八张表、每次 8 字节查表。本机同一份 16 MiB（E185 校准那一份，CRC 锚点 `0x3430be3b` 对上）用同一算法与 SSE4.2 的 `crc32` 指令（单路、不交错）各算 21 遍（量的，模型目录 `crc_speed.c`）：

```text
$ cat crc_speed.out
name=crc_speed implementation=software_eight_tables bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=5903485 bytes_per_second=2841917274
name=crc_speed implementation=sse42_instruction bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=1484834 bytes_per_second=11299051611
```

软件八表 2.84 GB/s 与 E185 段开头本机校准（16 MiB 6267586 ns，即 2.68 GB/s）对得上；指令单路就快 3.9759 倍（三次复跑 4.28–4.48 倍）。把各候选门槛里 CRC 那一部分按 3.9759 倍缩（D 全是 CRC，A 全是 CRC，B、C 只缩甲那一段），c_rt 照量到的不动：

```text
$ cat hardware_crc.out
name=hardware_crc_factor software_over_instruction=3.9759
name=judgement_with_the_instruction_crc candidate=scan_checksum_only card=0 losing=216
name=judgement_with_the_instruction_crc candidate=scan_checksum_only card=1 losing=216
name=judgement_with_the_instruction_crc candidate=scan_checksum_and_choice card=0 losing=89 not_losing=4 unstable=123
name=judgement_with_the_instruction_crc candidate=scan_checksum_and_choice card=1 losing=182 not_losing=4 unstable=30
name=judgement_with_the_instruction_crc candidate=release_check_only card=0 losing=214 unstable=2
name=judgement_with_the_instruction_crc candidate=release_check_only card=1 losing=216
name=judgement_with_the_instruction_crc candidate=every_stage card=0 losing=29 not_losing=96 unstable=91
name=judgement_with_the_instruction_crc candidate=every_stage card=1 losing=125 not_losing=55 unstable=36
```

A 在两张卡上 432 格全赔，D 只剩卡 0 的 2 格不稳定（正文第四节对 A、D 的结论在这一读法下更硬）；C 的不赔（不含今天的写法）由卡 0 的 132 格降到 96 格，卡 1 由 85 格降到 55 格。

### G1-③　两处一起换、只看间隔 50 ms

```text
$ cat combined.out
name=every_stage_after_both_cpu_improvements card=0 gap_milliseconds=0 judgement=losing cells=15
name=every_stage_after_both_cpu_improvements card=0 gap_milliseconds=0 judgement=not_losing cells=18
name=every_stage_after_both_cpu_improvements card=0 gap_milliseconds=0 judgement=unstable cells=111
name=every_stage_after_both_cpu_improvements card=0 gap_milliseconds=50 judgement=losing cells=41
name=every_stage_after_both_cpu_improvements card=0 gap_milliseconds=50 judgement=unstable cells=31
name=every_stage_after_both_cpu_improvements card=1 gap_milliseconds=0 judgement=losing cells=100
name=every_stage_after_both_cpu_improvements card=1 gap_milliseconds=0 judgement=not_losing cells=6
name=every_stage_after_both_cpu_improvements card=1 gap_milliseconds=0 judgement=unstable cells=38
name=every_stage_after_both_cpu_improvements card=1 gap_milliseconds=50 judgement=losing cells=72
```

CPU 模式换成 SSE4.2 指令的 CRC、乙段按身份查（推的替换），间隔 50 ms 下 C 在两张卡上一格不赔都没有（卡 0：41 格赔、31 格不稳定；卡 1：72 格全赔）；间隔 0 下卡 0 剩 18 格、卡 1 剩 6 格。这还没加 C 的乙、丙输入（第二节第 2 条），加上只会更少。

四句：
1. 分不分辨臂：分辨。A、D 在这几种读法下照旧出局（更硬），C 由「唯一成批不赔」变成「不赔与否取决于 CPU 模式那一臂写成什么样」。
2. 判别子看不看得到：看得到，CPU 模式的 CRC 用哪种实现、区域扫描按不按身份查，都在代码里。
3. 满足的是哪一个分句：第 7 条收益式子里减号前那一项（CPU 模式下的 CPU 时间）；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@不许先有结论再建模型`，那一节里的 `.claude/singlefs-ai-sop/rules/evidence-discipline.md:95`：「⚠️ **稻草人对照臂**：给对面那条路挑一个没人会真用的实现方式（比如「每个条目单独存一条记录」），拿它量出来的差当那条路的代价。」与第 96 行：「**判据**：对面那条臂的样子，**支持那条路的人认不认**。」——支持 CPU 模式的人不会认软件八表 CRC 与每条记录线性找全部映射这一臂。
4. 改法还中不中：岔路单第 10 行选「并」碰得到乙那一格；CPU 侧 CRC 换指令碰得到甲那一格（我提的，只在我的模型上量过、被攻过零轮）。两处都是 CPU 侧的改动，不碰格式。

### G1-④　卡 1 上 C 也赔得多：见第五节 G3。

## 四、G2　攒批的形态

### G2-①　拟合不线性跟着卡 0 的漂移走，不是代价本身不线性

```text
$ grep 'name=fit ' c_inputs.out
name=fit index=0 form=shared wait=blocking gap=0 fixed_ms=0.193968 per_mebibyte_ms=0.061909 maximum_relative_residual=0.217393 fit_not_linear=true
name=fit index=0 form=shared wait=blocking gap=50 fixed_ms=0.369545 per_mebibyte_ms=0.061993 maximum_relative_residual=0.489041 fit_not_linear=true
name=fit index=0 form=shared wait=sleep_100_microseconds gap=0 fixed_ms=0.226304 per_mebibyte_ms=0.056607 maximum_relative_residual=0.570083 fit_not_linear=true
name=fit index=0 form=shared wait=sleep_100_microseconds gap=50 fixed_ms=0.395357 per_mebibyte_ms=0.059690 maximum_relative_residual=0.141281 fit_not_linear=false
name=fit index=0 form=shared wait=sleep_1_millisecond gap=0 fixed_ms=0.179838 per_mebibyte_ms=0.064026 maximum_relative_residual=0.689029 fit_not_linear=true
name=fit index=0 form=shared wait=sleep_1_millisecond gap=50 fixed_ms=0.307083 per_mebibyte_ms=0.076497 maximum_relative_residual=0.118389 fit_not_linear=false
name=fit index=0 form=split wait=blocking gap=0 fixed_ms=0.157242 per_mebibyte_ms=0.118703 maximum_relative_residual=0.380355 fit_not_linear=true
name=fit index=0 form=split wait=sleep_100_microseconds gap=0 fixed_ms=0.182489 per_mebibyte_ms=0.114715 maximum_relative_residual=0.252071 fit_not_linear=true
name=fit index=0 form=split wait=sleep_1_millisecond gap=0 fixed_ms=0.099305 per_mebibyte_ms=0.120569 maximum_relative_residual=0.990022 fit_not_linear=true
name=fit index=1 form=shared wait=blocking gap=0 fixed_ms=0.056578 per_mebibyte_ms=0.487637 maximum_relative_residual=0.218780 fit_not_linear=true
name=fit index=1 form=shared wait=blocking gap=50 fixed_ms=0.106233 per_mebibyte_ms=1.463544 maximum_relative_residual=0.449607 fit_not_linear=true
name=fit index=1 form=shared wait=sleep_100_microseconds gap=0 fixed_ms=0.066975 per_mebibyte_ms=0.487991 maximum_relative_residual=0.095107 fit_not_linear=false
name=fit index=1 form=shared wait=sleep_100_microseconds gap=50 fixed_ms=0.218017 per_mebibyte_ms=1.461432 maximum_relative_residual=0.055185 fit_not_linear=false
name=fit index=1 form=shared wait=sleep_1_millisecond gap=0 fixed_ms=0.046974 per_mebibyte_ms=0.490537 maximum_relative_residual=0.066904 fit_not_linear=false
name=fit index=1 form=shared wait=sleep_1_millisecond gap=50 fixed_ms=0.164840 per_mebibyte_ms=1.459890 maximum_relative_residual=0.070459 fit_not_linear=false
name=fit index=1 form=split wait=blocking gap=0 fixed_ms=0.039440 per_mebibyte_ms=0.975133 maximum_relative_residual=0.044922 fit_not_linear=false
name=fit index=1 form=split wait=sleep_100_microseconds gap=0 fixed_ms=0.094627 per_mebibyte_ms=0.974610 maximum_relative_residual=0.180543 fit_not_linear=false
name=fit index=1 form=split wait=sleep_1_millisecond gap=0 fixed_ms=0.037045 per_mebibyte_ms=0.976690 maximum_relative_residual=0.047351 fit_not_linear=false
```

卡 0 的 9 条拟合有 7 条 `fit_not_linear=true`，卡 1 只有 2 条（都在阻塞等）。一条拟合用 8 档提交大小，每档取「各档 5 轮中位的中位」；一轮里 80 格轮流量，8 档分在不同时刻。卡 0 五轮之间差到 3–6.65 倍（第二节第 4 条），同一轮里先后量的两档也就混了时间走势进来；卡 1 没有走势，拟合就大多线性。所以拿卡 0 的拟合判「乙-1 ／ 乙-2 ／ 丙」，判的是走势；卡 1 的拟合能用：间隔 0 时固定 0.037–0.095 ms、每 MiB 0.487–0.977 ms，间隔 50 ms 每 MiB 1.46 ms（推的归因：两张卡的差别也可能来自别的，这一份产物分不开）。

### G2-②　墙钟那 5.3–6.4 ms 是核的串行深度，不是一次提交的固定开销

一次提交的等待挂钟几乎不随提交大小变：同一份第一段产物，卡 0 阻塞等、间隔 0、共用输入，第 146 行（4 个单元）`wait_wall_median_nanoseconds=5348923.500000`，第 155 行（512 个单元，16 MiB）`wait_wall_median_nanoseconds=6076675.000000`；卡 1 第 573 行（4 个）`5377443.500000`、第 580 行（512 个）`6201826.000000`。单元多 128 倍，等待只多 13.6%–15.3%。

核是每单元一条线程、逐字节逐位串着算：`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:92`：「let local_offset = global_id.x;」，一条线程把一个 32 KiB 单元的 32768 字节 × 8 位一步步做完。几百个单元并排跑，等多久由一个单元的串行长度定。所以第一轮用的 φ（固定开销占挂钟的份额）与 E185 Q13「要攒批」那一判，墙钟一侧的「固定开销」大半是这种核的写法；把一个单元拆给一个工作组、各算一段再按 CRC 的线性合起来，这一段就变短（推的，没写核）。D26 已定项 13 第 3 条那句「攒批」是用户定案，这一条不改它，只说「为什么要攒」的挂钟依据不在提交开销上。按第 7 条判的是主机 CPU，阻塞等在这两张卡上本来就睡（第 149 行 `wait_thread_cpu_median_nanoseconds=40360.000000` 对挂钟 5440233.5 ns），所以这一条对第 7 条的输赢不起作用，只对前台挂钟（G8）与攒批的理由起作用。

### G2-③　「跨意图只有 1 格」与 A 的意图内一次交

`merging_shape` 只对候选 C 的门槛（上界）判过；A、B、D 没有攒批类别。按卡、间隔数类别（命令对产物逐行取 `index`、`gap_milliseconds`、`category`）：

```text
$ grep 'name=merging_shape' research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out | awk '{for(i=1;i<=NF;i++){split($i,a,"="); v[a[1]]=a[2]} print v["index"], v["gap_milliseconds"], v["category"]}' | sort | uniq -c
     97 0 0 one_submission_per_batch
     47 0 0 unstable
     35 0 50 one_submission_per_batch
      6 0 50 one_submission_per_intent
     31 0 50 unstable
     82 1 0 one_submission_per_batch
     26 1 0 one_submission_per_intent
     36 1 0 unstable
      1 1 50 fit_not_linear
      1 1 50 merge_across_intents
      3 1 50 one_submission_per_batch
     46 1 50 one_submission_per_intent
     21 1 50 unstable
```

间隔 50 ms 下卡 1 的 72 格只有 3 格能一批一交，46 格要意图内一次交（乙-1）。乙-1 就是岔路单第 4 行的「窗口开头验一次字节、每批只重扫状态」，那一行还开着。

把 A 照乙-1 判（一条意图窗口开头交一次 m 个单元，c_rt(m) 对 A 一条意图省下的；模型 a_per_intent）：

```text
$ cat a_per_intent.out
name=scan_checksum_only_one_submission_per_intent card=0 gap_milliseconds=0 judgement=not_losing cells=74
name=scan_checksum_only_one_submission_per_intent card=0 gap_milliseconds=0 judgement=unstable cells=70
name=scan_checksum_only_one_submission_per_intent card=0 gap_milliseconds=50 judgement=losing cells=26
name=scan_checksum_only_one_submission_per_intent card=0 gap_milliseconds=50 judgement=not_losing cells=34
name=scan_checksum_only_one_submission_per_intent card=0 gap_milliseconds=50 judgement=unstable cells=12
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=0 judgement=losing cells=58
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=0 judgement=not_losing cells=54
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=0 judgement=unstable cells=32
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=50 judgement=losing cells=64
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=50 judgement=not_losing cells=1
name=scan_checksum_only_one_submission_per_intent card=1 gap_milliseconds=50 judgement=unstable cells=7
```

卡 0 间隔 0 有 74 格、间隔 50 ms 有 34 格不赔：A 只在「一批一交」这一形下出局。但这一读法有一处不公平：GPU 模式一个窗口只验一次字节，CPU 模式照旧每批验，两边验的次数不同。要公平，CPU 模式也只验一次，A 省下的就只剩 m 个单元算一次 CRC：本机 8 个单元 0.092680 ms、24 个 0.278361 ms（第一段产物第 123、127 行 `name=cpu_crc_of_the_same_bytes`），而卡 0 那一格 c_rt(8) 五轮是 0.192560、0.218315、0.218925、0.130126、0.060910 ms（`merging_shape` 第一行的 `one_submission_per_intent_host_cpu_milliseconds_per_round`），五轮四轮赔。

四句：分辨「一批一交」与「意图内一次交」两形，也分辨「两种模式验的次数相同 ／ 不同」；判别子（一个窗口里验几次字节）在代码里看得到；满足的是岔路单第 3 行翻面观测那一格的前提（拟合要线性）；改法「卡 0 重量、轮次打乱」碰得到 G2-① 那一格，G2-③ 那一格要第 4 行岔路先定。

## 五、G3　多卡分活（第一轮没打中，这一轮给反例）

攻的是正推的推荐：PCI 总线号定次序、已验厂商独显、按窗口轮转。

### G3-①　按窗口轮转让两张卡的平均替了卡 0：C 有 4 格由不赔翻成赔、23 格翻成不稳定

一条意图的提交轮流交卡 0、卡 1，每次提交的主机 CPU 取两张卡同一格同一轮的平均（模型 round_robin；c_rt 量的、轮转是推的分法）：

```text
$ grep every_stage round_robin.out
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=losing judgement_round_robin=losing cells=6
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=losing cells=4
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=not_losing cells=105
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=not_losing judgement_round_robin=unstable cells=23
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=unstable judgement_round_robin=losing cells=32
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=unstable judgement_round_robin=not_losing cells=10
name=round_robin_two_cards candidate=every_stage judgement_card0_alone=unstable judgement_round_robin=unstable cells=36
```

只用卡 0 不赔的 132 格里，轮转之后 4 格赔、23 格不稳定；只用卡 0 不稳定的 78 格里 32 格赔。同一个池、同一负载，「全部显卡 + 轮转」比「只用卡 0」少省主机 CPU，在这些格上由赚翻成赔。卡与卡之间的判定差得多：

`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out:4738`：「E7RESULT name=knob_flip_summary quantity=host_cpu_per_submission knob=card clause=two_cards_judge_differently pairs=1096 flipped=630」

第 2 条（第 246 行，改法一那一段整行抄过）下卡 1 照用；第 7 条按省下的 CPU 判值不值，两条在卡 1 上相撞。第 8 条末尾一张与几张判定相同那一句说的是条目的确认与不确定，管不到这一层。

### G3-②　第三张卡（AMD 集显）进来时，一张卡与几张卡在单一厂商编译器的缺陷下判定不同

本机三张显卡分属两家（改法一的 `lspci`）。序列（推的，没起 GPU）：

1. 某个着色器写法在 NVIDIA 的编译器上两份核都编错成同一个结果（第一轮攻方 G4-④ 说的共同前提），在 AMD 的编译器（RADV）上编对。
2. 只点卡 0：两份核都在卡 0，两份一致，判「确认」，错的结果交 CPU 改位图。
3. 不点卡（全部显卡）且两份核分在两张卡（岔路单第 5 行丙），一份在卡 0、一份在 AMD 集显：两份不同，判「不确定」，N 轮后交 CPU 判，结果对。

同一个池、同一批输入，一张卡与几张卡判定不同，撞第 8 条字面。第一轮攻方 G3-② 的序列是显存翻位（同一张卡共用输入），这一条是编译器，条件是机器上有两家的卡——本机就是。

### G3-③　BIOS 里开关集显，「全部显卡」的个数就变，显存上限的列表跟着对不上

用户照两张 NVIDIA 卡给 `background_gpu_memory_limit` 两个数、不给卡表。BIOS 里集显开着：参与的卡（按改法一筛完）是三张，个数不等，第 4 条拒绝挂载；关着：两张，挂得上。按 PCI 总线号排序（本机是 `01:00.0`、`10:00.0`、`79:00.0`，卡 0 在前）不改这一点。本机的总线号次序与 `nvidia-smi` 序号一致，「按总线号排序把赔的卡排在前」这一形在本机构造不出。

四句：
1. 分不分辨臂：G3-① 分辨「轮转」与「按卡的 c_rt 定用不用、用多少」；G3-② 分辨「两份核总在同一张卡」与「可以分在两张卡」；G3-③ 分辨「全部 = 枚举到的」与「全部 = 用户点名或挂载时登记的」。
2. 判别子看不看得到：G3-① 的「这张卡赔不赔」运行时看不到（产品里不量 c_rt）；G3-② 的「两份核是不是同一个编译器编的」看得到（适配器的厂商号）；G3-③ 看得到。
3. 满足的分句：G3-① 第 2 条字面与第 7 条；G3-② 第 8 条末尾判定相同那一句；G3-③ 第 3 条个数相等、第 4 条拒绝挂载。
4. 改法还中不中：「已验厂商独显」碰得到 G3-②、G3-③（把 AMD 集显筛掉），碰不到 G3-①（卡 1 也是已验厂商独显）；G3-① 要的改法是挂载时在每张卡上量一次 c_rt、赔的卡不用（我提的，零轮），它与第 2 条「全部」相撞，要用户答。

## 六、G4　两份核比对与 N（只补第二段的数罩到哪）

第二段每张卡两种输入各 1048560 个天然单元次、零次不一致，两种输入合起来的单侧 95% 上界 1.428497e-6。算术的命令与原样输出在文末「附：算术」，这一格用的是它的 `name=g4_per_tebibyte` 那一行。

- 罩到的：两份 CRC 核、32768 与 16384 两种长度、随机字节、两张 NVIDIA 卡、同一时段。
- 罩不到的：候选 C 里乙、丙两段的核（读单元头、反查映射、给段打分），一份都没跑过；本机 AMD 集显；长时间挂着、别的进程同时占卡。
- 按上界算，1 TiB 的池（33554432 个 32 KiB 单元）扫一遍，期望的不确定条目可以多到 47.93 条；零次观测排除不了这个量，N 取几不能由它翻面（登记 R7 的 `zero` 那一类本来就写了「N 的取值不由这个量翻面」）。没打中：这一格不是反例，是射程。

## 七、G5　控制接口（第一轮没打中，这一轮给反例）

攻的是正推的推荐：会话方法、在 `advance_compaction_by_one_publish` 进门读一次（第一轮判决 `research/prompts/m4-gpugc-design-r1-main-verification.md` 第 38 行那一格）。第一轮攻方攻过的「下一批退成下一趟」不重复。

反例（推的序列，读的是条款字面）：D26 已定项 13 第 1 条管切换什么时候生效（从下一批起，一批之内不换），第 4 条管挂着时 GPU 出错（放弃这一批、停下报错、不换回 CPU，要人切回 cpu 或修好 GPU 再切）；两条在 `.claude/kb/decisions/26-后台整理与放置回收.md` 第 245、248 行，括号里是我的转述，指路不抄。三个事件：S＝用户经控制接口请求切到 `cpu`；E＝正在算的这一批 GPU 出错；B＝下一批开头（进门读开关）。

| 次序 | 第 1 条下 | 第 4 条下 | 结局 |
|---|---|---|---|
| S、B、E | B 读到 `cpu`，这一批走 CPU | E 不会发生（没有 GPU 批） | 照常 |
| E、S、B | E 让 GC 停下 | S 就是「人切回 `cpu`」，恢复；B 读到 `cpu` | 照常 |
| **S、E、B** | S 只记下，等下一批（这一批在 GPU 上算着） | E 让 GC 停下；「人切回 `cpu`」已经在 E 之前发生过 | GC 停着、模式已是 `cpu`，B 不再发生，进门读开关那一步永远走不到；用户看到的是「已切到 cpu」而 GC 不跑。要再起，只能再发一次切换（`cpu` 切 `cpu`），这一步条款没有写 |

停着的时候空间只减不增（D26 射程那条 ⚠️）。GC 挪出调用线程之后 S 在一批的中间到是常态，这一格才可达；今天整理在调用线程里跑、会话独占借用，S 落不进一批中间（第一轮攻方 G5 那一句），也没有「停下」这个持久状态：今天跑砸了是这一趟交回 `crates/singlefs-core/src/mounted_session.rs:2785`：「CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),」，只记一笔 `crates/singlefs-core/src/mounted_session.rs:2632`：「compaction_of_the_run.observations.last_stop = Some(*stop);」，下一次越线照样起。

四句：分辨「进门读开关」与「切换请求自己就能把停着的 GC 拉起来」两种接口；判别子（切换请求与出错谁先谁后）系统看得到，两件事都在会话里有时刻；满足第 1 条的生效时点与第 4 条的恢复办法两句合起来；正推的推荐在 S、E、B 这一格上照中，补法是「停着时收到切换请求当场生效、起 GC」（我提的，零轮）。

## 八、G6　三个默认值（攻它的改法「挪出调用线程」）

推的，没打中：挪出之后「前台有 fsync 在等」看得到了，但只在批与批之间看（`.claude/kb/decisions/26-后台整理与放置回收.md:250`：「6. **起跑之后的节奏**：GC 占盘写带宽有一个上限（挂载参数）；前台有 fsync 在等时不发下一批。」）。fsync 在一批开头之后到，要等这一批（第三段 `scope=batches` 的一批 CPU 18–338 ms，第二节第 6 条），GPU 模式另有一次飞着的提交不能中途收回（每次 5.5–7.7 ms 墙钟，第十节）；第 3 条「攒批」把几批连成一次交，「一批」又变长。改法让判别子看得到，等多久仍由一批的长度定，第一轮攻方 G6-② 的那一格不受它影响。

## 九、G7　GPU 代码住哪、用什么写（第一轮没打中，这一轮给反例，推的）

攻的是正推的推荐：新包、核心层不依赖它、CPU 参照用核心层现成的、与 checker 档不共用。

- 「不共用」只管源码。checker 档把版本钉死：`crates/singlefs-checker-tier/Cargo.toml:27`：「wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"], optional = true }」，`Cargo.lock` 里 `naga` 也是 30.0.1（第 452–453 行）。产品的 GPU 包进同一个工作区，Cargo 把同一个大版本统一成一份，产品只能用 `wgpu` 30.0.1、同一份 naga、同一个驱动编译器。D13 已定项 5 要 checker 与实现的校验和各用一份独立实现（`.claude/kb/decisions/13-验证路线.md` 第 156 行，指路不抄）：两份源码独立，翻译链共用。
- 序列：naga 30.0.1 把某种循环边界写法译错，让核少读单元末尾几个字节。产品 GPU 模式两份核都用这种写法，一个末尾坏了的单元两份都判「校验和对」，照「可搬」搬走（本该隔离）；checker 档的 GPU 判器若用同一种写法核同一个单元，同样判对。产品与判器一起放过，崩溃点重放看不见。要它成立，两边要恰好用同一种写法，这一条我没法在 crates 上构造（产品 GPU 代码不存在），记作结构上的共同路径。
- 用什么写：墙钟那 5.3–6.4 ms 来自「每单元一条线程逐位算」（G2-②），换 WGSL 还是直写 SPIR-V 不改这一点，改的是核怎么拆单元。第 7 条下这一项不分输赢（阻塞等本来就睡），对 G8 的前台挂钟起作用。

四句：分辨「与 checker 档不共用源码」与「也不共用翻译链（不同的 `wgpu` 版本或不同的后端）」；判别子（两边用的是哪一份 naga）在 `Cargo.lock` 里看得到；满足 D13 已定项 5 那一句的字面要看「独立实现」算不算翻译链，这一处要用户或主 agent 读条款；改法「产品的 GPU 包另开工作区或钉另一个大版本」碰得到这一格，代价是两份 `wgpu`（我提的，零轮）。

## 十、G8　GC 挪不挪出用户改动的调用线程（关的第 4 条）

今天那一趟在用户改动里给的保证，从代码读出四样：发成改动之后才清零已过去的发布数（`crates/singlefs-core/src/mounted_session.rs:1926`：「compaction.publishes_since_the_last_user_change = 0;」）；次序在这次改动的删除续做与抬 F 之后（同一文件第 1925 行：「次序在这次改动的删除续做与为它推的抬 F 之后；整理没发成不让这次改动算没发成」）；至多一批（第 1935 行：「CompactionRunReach::AtMostOneBatchAfterAUserChange,」）；会话 `&mut self` 独占，整理与用户改动不会交错（第 1849 行：「pub fn publish_user_change<Device: BlockDevice>(」）。挪出之后逐样看。

### G8-①　发布预算：`.max(1)` 加定时兜底，空闲时连着的整理发布没有上限

预算的式子与它要护的东西：`crates/singlefs-core/src/compaction_plan.rs:805`：「/// 一次整理运行的发布预算（判决 G8 `B_gc`，正推 O11：整理的发布不改用户可见状态，连发会把回退候选集挤成 1 个状态）：」，第 815 行：「.max(1)」。已过去的发布数只在用户改动里清零、每次整理发布加一（同一文件 `mounted_session.rs` 第 2736 行：「compaction.publishes_since_the_last_user_change += 1;」）。D26 已定项 13 第 5 条要两样起跑：`.claude/kb/decisions/26-后台整理与放置回收.md:249`：「5. **什么时候起跑**：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同。」

模型 g8（把 `run_compaction_on` 的循环抄成状态机：预算、一趟的局部变量、清零、至多一批；每条意图签发一次、搬 b 批，完成之后不增全空段；用户改动的间隔放开扫：只在开头一次、每 1、5、50 拍一次）：

```text
$ cat g8.out
name=g8_loop batches_per_intent=2 policy=today user_change_every=once ticks=200 publishes=3 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=0
name=g8_loop batches_per_intent=2 policy=today user_change_every=1 ticks=200 publishes=500 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=99
name=g8_loop batches_per_intent=2 policy=today user_change_every=5 ticks=200 publishes=100 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=19
name=g8_loop batches_per_intent=2 policy=today user_change_every=50 ticks=200 publishes=10 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=1
name=g8_loop batches_per_intent=2 policy=background_timer user_change_every=once ticks=200 publishes=214 max_consecutive_gc_publishes=213 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=70
name=g8_loop batches_per_intent=2 policy=background_timer user_change_every=1 ticks=200 publishes=800 max_consecutive_gc_publishes=3 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=199
name=g8_loop batches_per_intent=2 policy=background_timer user_change_every=5 ticks=200 publishes=640 max_consecutive_gc_publishes=15 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=199
name=g8_loop batches_per_intent=2 policy=background_timer user_change_every=50 ticks=200 publishes=256 max_consecutive_gc_publishes=63 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=83
name=g8_loop batches_per_intent=2 policy=background_after_every_publish user_change_every=once ticks=200 publishes=200014 max_consecutive_gc_publishes=200013 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=66670
name=g8_loop batches_per_intent=2 policy=background_after_every_publish user_change_every=1 ticks=200 publishes=202700 max_consecutive_gc_publishes=1013 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=67499
name=g8_loop batches_per_intent=2 policy=background_after_every_publish user_change_every=5 ticks=200 publishes=200560 max_consecutive_gc_publishes=5013 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=66839
name=g8_loop batches_per_intent=2 policy=background_after_every_publish user_change_every=50 ticks=200 publishes=200056 max_consecutive_gc_publishes=50013 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=66683
name=g8_loop batches_per_intent=6 policy=today user_change_every=once ticks=200 publishes=3 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=0
name=g8_loop batches_per_intent=6 policy=today user_change_every=1 ticks=200 publishes=434 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=33
name=g8_loop batches_per_intent=6 policy=today user_change_every=5 ticks=200 publishes=87 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=6
name=g8_loop batches_per_intent=6 policy=today user_change_every=50 ticks=200 publishes=9 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=0
name=g8_loop batches_per_intent=6 policy=background_timer user_change_every=once ticks=200 publishes=218 max_consecutive_gc_publishes=217 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=30
name=g8_loop batches_per_intent=6 policy=background_timer user_change_every=1 ticks=200 publishes=1600 max_consecutive_gc_publishes=7 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=199
name=g8_loop batches_per_intent=6 policy=background_timer user_change_every=5 ticks=200 publishes=882 max_consecutive_gc_publishes=22 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=120
name=g8_loop batches_per_intent=6 policy=background_timer user_change_every=50 ticks=200 publishes=270 max_consecutive_gc_publishes=67 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=37
name=g8_loop batches_per_intent=6 policy=background_after_every_publish user_change_every=once ticks=200 publishes=200018 max_consecutive_gc_publishes=200017 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=28573
name=g8_loop batches_per_intent=6 policy=background_after_every_publish user_change_every=1 ticks=200 publishes=203434 max_consecutive_gc_publishes=1017 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=29033
name=g8_loop batches_per_intent=6 policy=background_after_every_publish user_change_every=5 ticks=200 publishes=200688 max_consecutive_gc_publishes=5017 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=28663
name=g8_loop batches_per_intent=6 policy=background_after_every_publish user_change_every=50 ticks=200 publishes=200069 max_consecutive_gc_publishes=50017 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=28580
```

- 今天的形态（只在用户改动之后起、至多一批）：任何间隔下连着的整理发布至多 2 次（签发加一批），20 那条线碰不到。
- 后台按定时兜底每拍起一趟显式入口：用户改动只在开头一次时，200 拍连着 213 次（b = 2）、217 次（b = 6），过了 3S − 4 = 20 与根环 3S = 24；每 50 拍一次时 63 次、67 次。原因就是 `.max(1)`：预算用完之后每一趟仍给 1 次。
- 后台每次发布之后都查水位（整理自己的发布也算发布）：每一趟发 1 次就再起一趟，模型里每拍推满 1000 次才停，连着的发布只受模型自己的截断管。

四句：分辨「今天只在用户改动之后起」与「后台按定时或按每次发布起」；判别子（自上一次用户改动以来已发了几次）系统看得到；满足的是 `compaction_plan.rs` 第 805 行那一句要护的回退候选集，第 815 行 `.max(1)` 让它在没有用户改动时护不住；改法「预算用完就等下一次用户改动，`.max(1)` 只给用户改动之后的那一趟」碰得到（我提的，只在模型 g8 上量过、零轮），它让空闲的池整理停在 20 次。

### G8-②　「不增全空段就不签下一条」是一趟里的局部变量

`crates/singlefs-core/src/mounted_session.rs:2704`：「let mut last_completed_intent: Option<CompactionIntent> = None;」，判它的在第 3044 行：「if let Some(completed) = *last_completed_intent {」。每起一趟它从空开始。模型 g8 最后一列：今天的形态下用户改动每 1 拍一次时，200 拍里也签了 99 条（b = 2）跟在不增全空段的意图后面的新意图——今天也护不住跨趟，只是每次用户改动至多推一批，空转的速度被用户改动的速度管着。挪到后台、每拍或每次发布起一趟，这个速度就没人管了（定时兜底、只开头一次：70 条、30 条）。

四句：分辨「护在一趟里」与「护在会话里」；判别子（上一条完成的意图有没有增全空段）看得到，只是没跨趟留下；满足的是 `crates/singlefs-core/src/compaction_plan.rs:785`：「pub const fn the_completed_intent_yielded(」那一判（它的文档注释点名攻方 K-h）；改法「把上一条完成的意图存进会话的整理状态」碰得到（我提的，零轮）。

### G8-③　空间准入与抬 F 的次序：整理批的落点不过准入，挪出之后能插进用户改动的「抬 F 再判」中间

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:100`：「2. **整理不是释放路径，它走常规准入**，批次最坏占用计入「已承诺预留」。」

今天的代码：搬一批的那一次发布不判准入，`crates/singlefs-core/src/transaction.rs:9667` 那一行里写「搬迁副本不进重写的角色、不进需求，所以两判都不判」；落点现取，第 9661 行里写「目的落点由这次发布时的分配器现取」；全空聚簇段那一合取式也免，第 11210 行：「if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {」。我没找到整理批的最坏占用进「已承诺预留」的那一处：`grep -n '整理\|compaction\|relocat' crates/singlefs-core/src/admission.rs` 4 处命中，第 838、844、862、865 行，都在讲合取 7 免整理批；`grep -n '预留' crates/singlefs-core/src/mounted_session.rs crates/singlefs-core/src/compaction_plan.rs` 0 处。条款与代码在这一处对不上，今天被「整理跑在这次改动的准入之后」盖住了。

用户改动被拒是空间不够时，会话推一串抬 F 再判，接在推完的那一版后面：`crates/singlefs-core/src/mounted_session.rs:3483` 那一行里写「推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）」。

序列（推的；数是举例，机制照上面几行）：
1. 池接近满：可分配的比这次写要的 x 个单元少 1 个；之前删掉的文件留着 K 个已释放、还没到 F 之上的单元。一条整理意图活着，下一批要搬 K 个单元。
2. 今天：用户写 → 准入拒 → 推抬 F，K 个回收 → 再判，过 → 发成 → 这一趟整理才跑，一批取走 K 个落点。用户写成功。
3. 挪出之后：用户写 → 准入拒 → 推抬 F，K 个回收 → **后台整理这时拿到会话，发一批，取走 K 个落点（不判准入）** → 用户写再判，又拒 → 再推抬 F：刚搬走的 K 个源单元释放代是这一批的，还没被 F 罩住，回收不出来 → 准入预算推满，交回 `NoSpaceAfterRaisingTheFloor`，报给用户就是 ENOSPC。

四句：分辨「整理与用户改动串在同一次调用里、整理在后」与「整理在别的线程、可以插进来」；判别子（整理这一批的落点算不算进准入）代码里看得到，今天是不算；满足的是上面整行抄的已定项 3 第 2 条后半句的字面（今天没兑现）；改法「挪出调用线程」在这一格上照中，要补「整理批的最坏占用进已承诺预留」（条款原文要的）或「一次用户改动的抬 F 再判期间不让整理插进来」。

### G8-④　GPU 等的那几毫秒，会话放不放

挪出调用线程是为了让用户改动不等 GPU（第一轮判决 G1-③）。两种做法：
- 不放：后台整理从扫 R 到发成都占着会话，用户改动照样等一整批，外加这一批的 GPU 往返（每次提交 5.5–7.7 ms 墙钟，下面 G8-⑥）。前台挂钟没少，只换了一条线程等。
- 放：整理在 V 这一版上扫、交 GPU、等的时候放开会话；用户改动发成 V + 1（例如改写了 R 里一个文件的数据单元，新副本落在别处、旧的释放）；整理拿回会话、按 V 上的计划对 V + 1 发这一批。逐条核计划那一步拒（`crates/singlefs-core/src/transaction.rs:10225`：「RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {」），这一趟交回 `mounted_session.rs` 第 2785 行那一种失败。热文件落在 R 里时每一趟都这样，GPU 那一次的主机 CPU 白花，CPU 还要重扫。

第一轮判决把「挪出调用线程」当成 G1-③、G5、G6 共同的改法，没分这两种；两种在前台挂钟与整理能不能推进上方向相反。

### G8-⑤　`run_compaction` 够不够当后台入口

`crates/singlefs-core/src/mounted_session.rs:1956`：「pub fn run_compaction<Device: BlockDevice>(」，`&mut self`，一趟推到预算用完或停（第 1950 行里写「这一趟至多发 `B_gc` 次整理发布」），没有「至多一批」。拿它当后台入口、每次占会话跑完一趟，用户改动最坏等一整趟：P = 8192、f = 0.75、K = 8 一批 CPU 338.405 ms（第三段第 1253 行），20 次发布按这个数算约 6768 ms（文末「附：算术」，推的上界：20 次里有签发与删意图，不全是整批）。今天用户改动至多等一批。按一次发布切开（`advance_compaction_by_one_publish` 那一层）才能把前台的等压回一批，而切开又让 G8-② 那个局部变量每次从空开始。

### G8-⑥　不挪出、只在调用线程上用 GPU：前台挂钟多多少

一次提交的墙钟（模型 foreground_wall，共用输入、8 与 24 个单元两档）：

```text
$ grep 'form=shared' foreground_wall.out | grep 'units=8 \|units=24 '
name=submission_wall card=0 form=shared wait=blocking gap_milliseconds=0 units=8 rounds=5 wall_ms_median=5.593 wall_ms_max=5.627
name=submission_wall card=0 form=shared wait=blocking gap_milliseconds=0 units=24 rounds=5 wall_ms_median=5.959 wall_ms_max=6.162
name=submission_wall card=0 form=shared wait=blocking gap_milliseconds=50 units=8 rounds=5 wall_ms_median=5.792 wall_ms_max=5.932
name=submission_wall card=0 form=shared wait=blocking gap_milliseconds=50 units=24 rounds=5 wall_ms_median=6.361 wall_ms_max=6.511
name=submission_wall card=0 form=shared wait=sleep_100_microseconds gap_milliseconds=0 units=8 rounds=5 wall_ms_median=5.598 wall_ms_max=5.616
name=submission_wall card=0 form=shared wait=sleep_100_microseconds gap_milliseconds=0 units=24 rounds=5 wall_ms_median=6.080 wall_ms_max=6.242
name=submission_wall card=0 form=shared wait=sleep_100_microseconds gap_milliseconds=50 units=8 rounds=5 wall_ms_median=5.769 wall_ms_max=5.795
name=submission_wall card=0 form=shared wait=sleep_100_microseconds gap_milliseconds=50 units=24 rounds=5 wall_ms_median=6.169 wall_ms_max=6.237
name=submission_wall card=0 form=shared wait=sleep_1_millisecond gap_milliseconds=0 units=8 rounds=5 wall_ms_median=6.416 wall_ms_max=6.561
name=submission_wall card=0 form=shared wait=sleep_1_millisecond gap_milliseconds=0 units=24 rounds=5 wall_ms_median=6.586 wall_ms_max=6.622
name=submission_wall card=0 form=shared wait=sleep_1_millisecond gap_milliseconds=50 units=8 rounds=5 wall_ms_median=6.636 wall_ms_max=6.666
name=submission_wall card=0 form=shared wait=sleep_1_millisecond gap_milliseconds=50 units=24 rounds=5 wall_ms_median=6.607 wall_ms_max=6.708
name=submission_wall card=1 form=shared wait=blocking gap_milliseconds=0 units=8 rounds=5 wall_ms_median=5.618 wall_ms_max=5.771
name=submission_wall card=1 form=shared wait=blocking gap_milliseconds=0 units=24 rounds=5 wall_ms_median=6.295 wall_ms_max=6.398
name=submission_wall card=1 form=shared wait=blocking gap_milliseconds=50 units=8 rounds=5 wall_ms_median=6.132 wall_ms_max=6.161
name=submission_wall card=1 form=shared wait=blocking gap_milliseconds=50 units=24 rounds=5 wall_ms_median=7.448 wall_ms_max=7.520
name=submission_wall card=1 form=shared wait=sleep_100_microseconds gap_milliseconds=0 units=8 rounds=5 wall_ms_median=5.697 wall_ms_max=5.772
name=submission_wall card=1 form=shared wait=sleep_100_microseconds gap_milliseconds=0 units=24 rounds=5 wall_ms_median=6.391 wall_ms_max=6.407
name=submission_wall card=1 form=shared wait=sleep_100_microseconds gap_milliseconds=50 units=8 rounds=5 wall_ms_median=6.314 wall_ms_max=6.417
name=submission_wall card=1 form=shared wait=sleep_100_microseconds gap_milliseconds=50 units=24 rounds=5 wall_ms_median=7.545 wall_ms_max=7.619
name=submission_wall card=1 form=shared wait=sleep_1_millisecond gap_milliseconds=0 units=8 rounds=5 wall_ms_median=6.462 wall_ms_max=6.534
name=submission_wall card=1 form=shared wait=sleep_1_millisecond gap_milliseconds=0 units=24 rounds=5 wall_ms_median=6.700 wall_ms_max=6.867
name=submission_wall card=1 form=shared wait=sleep_1_millisecond gap_milliseconds=50 units=8 rounds=5 wall_ms_median=6.891 wall_ms_max=6.982
name=submission_wall card=1 form=shared wait=sleep_1_millisecond gap_milliseconds=50 units=24 rounds=5 wall_ms_median=7.660 wall_ms_max=7.737
```

一次越线后的用户改动至多跑签发加一批，至多两次提交，多 11.0–15.4 ms 墙钟（5.5–7.7 ms 的两倍）；第 7 条下省下的主机 CPU 每次至多 0.72 ms（`g1.out` 门槛上界），抵不掉。相对今天那一批的 CPU：P = 1024、f = 0.25 多 61.1%–85.6%，P = 8192、f = 0.75 多 3.3%–4.6%（文末「附：算术」）。墙钟里大头是核的串行深度（G2-②），换一种拆单元的核会小很多（推的）。

四句（G8 整格）：
1. 分不分辨臂：分辨「留在调用线程」与「挪出调用线程」，而且把「挪出」拆成两种（占着会话等 GPU ／ 等的时候放开会话），两种各丢一样（前台挂钟 ／ 计划过期）；G8-①②③ 三样对两种挪法都中。
2. 判别子看不看得到：都看得到（预算计数、上一条完成的意图、落点算不算进准入、会话有没有被别的调用拿走），只是今天代码靠独占借用不用看。
3. 满足的分句：G8-① 是 `compaction_plan.rs` 第 805 行要护的回退候选集；G8-③ 是 D26 已定项 3 第 2 条；G8-④⑥ 是 D24 已定项 2 第 2 条不许进前台路径（第一轮判决 G1-③ 那一条）。
4. 改法还中不中：第一轮推的「挪出调用线程」在 G8-①②③ 上照中，要另带三样补法（预算只在用户改动之后给 `.max(1)`；上一条完成的意图存进会话；整理批的最坏占用进已承诺预留或抬 F 再判期间不让整理插进来），三样都是我提的、零轮。

### G8 的岔路（交用户定；不写倾向）

| 候选 | 定义 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 甲 留在调用线程 | 今天的形态，越线后的用户改动返回之前跑至多一批 | 开 GPU 时一次越线后的用户改动多等的墙钟（11.0–15.4 ms，今天这种核）与一批 CPU 之比 | 换一种拆单元的核再量一次提交墙钟；P 多取几档 |
| 乙-1 挪到后台、占着会话等 GPU | 后台线程按一次发布切开推，每次占会话从扫 R 到发成 | 用户改动最坏等多久（一批 CPU 加一次 GPU 往返）；G8-①②③ 补了没有 | 补法三样落进代码、模型 g8 那一类状态机在 crates 上对拍 |
| 乙-2 挪到后台、等 GPU 时放开会话 | 同上，GPU 等的时候让用户改动先发 | 热文件在 R 里时整理一趟被逐条核计划拒的频率 | 带热文件的负载上量拒绝率与整理推进速度 |
| 丙 后台按整趟占会话 | 用 `run_compaction` 当入口 | 用户改动最坏等一整趟（P = 8192 约 6.8 s，推的上界） | 同一负载上量用户改动的挂钟分布 |

## 十一、交岔路的地方（汇总；不写倾向）

| 岔路 | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| 第 1 行 GPU 上算哪几段 × 第 10 行按身份查 | C 配「不并」／ C 配「并」／ A、B、D | C 在间隔不短于一批 CPU 的格上，CPU 模式那一臂用指令 CRC 与按身份查之后还赔不赔 | 第一段补量长间隔（100、300、1000 ms）；CPU 侧 CRC 换指令、按身份查之后重量门槛（入库装置，不是我的模型） |
| 「全部显卡」 | 枚举到的全部（含集显）／ 已验厂商独显 ／ 挂载时量 c_rt、赔的卡不用 | 第三张卡（AMD 集显）的每次提交主机 CPU；卡 1 在长间隔上赔不赔 | 本机三张卡都量第一段；用户答「全部」要不要改窄 |
| 卡 0 的读数能不能用 | 照现在的五轮读 ／ 重量、轮次打乱 | 同一格五轮的时间走势（第 5 轮 ÷ 第 1 轮中位 0.643）消不消失 | 卡 0 重量一段，每格前后加一次与提交同形的校准 |
| 控制接口与 GPU 出错的次序 | 进门读开关 ／ 切换请求当场拉起停着的 GC | S、E、B 那一序列下 GC 起不起得来 | 条款补一句，或实现里加一条单测走这一序 |
| G8 | 见上表 | 见上表 | 见上表 |

## 没打中的形状

| 格 | 试过什么 | 为什么没成 |
|---|---|---|
| 改法四 | 找一个已知答案自检放过、而 crates 上真会出的错 | 产品 GPU 代码不存在；第二段两张卡 16384 个单元逐条相同，同一家驱动下没有可借的实例 |
| 改法二 | 在 crates 上造一个跨分块边界才出错的准备 | 同上，准备那一步不存在；只给了上限的数 |
| G3 | 按 PCI 总线号排序把赔的卡排到前面 | 本机总线号次序（01、10、79）与 `nvidia-smi` 序号一致，卡 0 在前 |
| G4 | 第二段里找确定性不一致或两核一起错 | 四组 `disagreement_summary` 全 0，`determinism_replay` 四行 `all_identical=true` |
| G6 | 找「挪出」之后仍观测不到 fsync 在等的序列 | 挪出之后批与批之间看得到；只剩等多久的问题（第一轮 G6-② 那一格） |
| G7 | 找产品 GPU 包与 core 依赖方向上的反例 | 两种方向第一轮都推过，这一轮没找到新的 |

取样范围：第一段 2684 行点判定全读（两张卡、三种等法、两种输入、两种间隔、两种门槛、两种钟）；G8 模型两种批数（每条意图 2 批、6 批）× 三种起跑 × 四种用户改动间隔，200 拍；CRC 探针 21 遍 × 4 次（一次入库、三次复跑）。

## 这条腿自己的限度

- 没起 GPU、没编 Rust、没跑 crates。G1 的两处 CPU 侧替换（指令 CRC、按身份查）是在门槛上按比例改，不是在入库装置上重量；指令 CRC 的倍数是我本机探针量的（单路、不交错，交错会更快），乙段按身份查之后取 P = 1024 的值是推的。
- 卡 0 走势的原因没分出来：起跑之前卡 0 上有别的计算进程；我这一次开探针之前 `ps` 看到本机有别的会话的 `cargo build --offline --all-targets` 与一串 `cargo test -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`（经内存包装、在 `/tmp` 下的运行目录），第一段跑的那天有没有同样的负载，产物里没记。
- 模型 g8 是 `run_compaction_on` 控制流的抄写，没有空间、没有快照、没有 GPU；「每次发布之后起一趟」那一种起法是我对第 5 条的读法。
- C 的乙、丙输入的每条字节数（60、20）是推的布局。
- 引 kb 长行时只指路、不抄；指路处的括号是我的转述。

## 没做什么

- 没读同一轮辩方腿的报告（禁读）。
- 没跑门禁、没跑 checker 档测试、没起 GPU、没用第二台；只读 `lspci` 与文件清单看了本机的卡。
- 没在入库装置上重量任何数；文中副本与模型上的数都不是入库装置上的数。
- 草稿目录 `~/.local/share/singlefs/work/m4-gpugc-design-r2-opus/` 里有模型草稿、三次复跑的输出目录（`rerun1`–`rerun3`，内含 gcc 编出的探针二进制）与报告分段。三次复跑的输出目录交回前已删（删之前 `du -sh`：`rerun1` 100K、`rerun2` 108K、`rerun3` 108K）；没建仓副本与编译目录。

## 附：算术

G4、G8-⑤、G8-⑥ 与改法二用到的算术（命令与原样输出；一批 CPU 取第三段第 149、1253 行的 `call_cpu_nanoseconds_median`）：

```bash
nice -n 19 python3 -I -c '
units_per_tebibyte = 2**40 // 32768
print("name=g4_per_tebibyte units=%d expected_uncertain_at_the_upper_bound=%.2f" % (units_per_tebibyte, units_per_tebibyte * 1.428497e-6))
print("name=g2_threads_per_dispatch max=%d pool_bytes_at_one_thread_per_entry=%d" % (65535 * 64, 65535 * 64 * 32768))
for label, batch_ms in (("P1024_f025_K8_batches", 17.993662), ("P8192_f075_K8_batches", 338.405211)):
    for wall in (5.5, 7.7):
        print("name=g8_in_thread label=%s batch_cpu_ms=%.3f submission_wall_ms=%.1f two_submissions_ms=%.1f increase_share=%.3f" % (label, batch_ms, wall, 2 * wall, 2 * wall / batch_ms))
print("name=g8_explicit_entry worst_publishes=20 batch_cpu_ms=338.405 run_ms=%.0f" % (20 * 338.405211))
'
```

```text
name=g4_per_tebibyte units=33554432 expected_uncertain_at_the_upper_bound=47.93
name=g2_threads_per_dispatch max=4194240 pool_bytes_at_one_thread_per_entry=137436856320
name=g8_in_thread label=P1024_f025_K8_batches batch_cpu_ms=17.994 submission_wall_ms=5.5 two_submissions_ms=11.0 increase_share=0.611
name=g8_in_thread label=P1024_f025_K8_batches batch_cpu_ms=17.994 submission_wall_ms=7.7 two_submissions_ms=15.4 increase_share=0.856
name=g8_in_thread label=P8192_f075_K8_batches batch_cpu_ms=338.405 submission_wall_ms=5.5 two_submissions_ms=11.0 increase_share=0.033
name=g8_in_thread label=P8192_f075_K8_batches batch_cpu_ms=338.405 submission_wall_ms=7.7 two_submissions_ms=15.4 increase_share=0.046
name=g8_explicit_entry worst_publishes=20 batch_cpu_ms=338.405 run_ms=6768
```

改法二里映射表过 128 MiB 那一处（每条 60 字节是推的布局）：

```bash
python3 -I -c 'n=(128<<20)//60; print("name=mapping_table_binding_limit entries=%d pool_bytes=%d gibibytes=%.1f" % (n, n*32768, n*32768/2**30))'
```

```text
name=mapping_table_binding_limit entries=2236962 pool_bytes=73300770816 gibibytes=68.3
```

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-design-r2-opus-output.md` 的末行：

```text
  ✓ 核了 27 处引文、9 处裸行号，对上 26 处，没判 10 处（其中裸行号没有引文可比 9 处；逐处列在上面）
```

没判的 10 处：1 处是仓外的 `wgpu-types` 源码（改法二那一处，工具认不出 `~`）；9 处是带了「」却没被工具认成引文的行（引文里套着「」或整行很长），要主 agent 人工核。
