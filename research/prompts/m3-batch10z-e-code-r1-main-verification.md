# 根环实现第 10z 批第五块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 G2 H1 H19 M01 M05 M10 M13 M14 M17 M22 M23 M33 C1 C2 C3 C4 F1 F2 F3 F4 F5 -->

正文 `research/prompts/_m3-batch10z-e-code-r1-body.md`。两腿报告：
- 正推：`research/prompts/m3-batch10z-e-code-r1-sonnet-output.md`，sha256 3d0a162b…f6，252 行；
- 攻方：`research/prompts/m3-batch10z-e-code-r1-opus-output.md`，sha256 f1229a8e…ea，245 行。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1/tree/` 对 `research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 全过 |
| 引文 | `cite-check.py` 两份报告合跑：`--root` 冻结副本 `✓ 核了 51 处引文，对上 47 处，没判 4 处`，没判的 4 处引的是仓根 `research/prompts/` 下的判决文件；`--root .` `✓ 核了 51 处引文，对上 51 处，没判 0 处` |
| 主 agent 现读 | 冻结副本 `crates/singlefs-harness/src/gpu_memory_ledger.rs` 第 1906 行 `first < largest`、第 1959 行取 max、第 2178 行 `StopOfTheRun::Requested if blocks_waiting == 0 => AfterARoundOfTheCards::StoppedOnRequest,`、第 2294–2302 行轮末两条断言；`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 3105 行一轮收尾现读叫停、第 5591 行整趟收尾再现读一次：与两腿引的相同 |
| 复跑 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1-main/rerun.sh`，两份冻结副本拷贝、两个编译目录并行，rc 全 0。攻方两臂 `ATTACK arm=` 行与 `output-models.txt` 逐字相同；`MUTANT H1-largest-is-the-last-judged lib_rc=0 model_rc=101`、`MUTANT H19-stop-with-one-waiting-block-stops lib_rc=0 model_rc=101`。正推 `endings_new` 的 `ENDINGS_NEW` 与 `table` 行与 `logs/endings_new.log` 排序后逐行相同；`killers_new` 真代码上 `7 passed`，在 M05、M10、M13、M14、M17、M22、M23 上各红且只红对应那一条；`rounds_new`（`RUNS=20000`）`stop-in-the-closing-round` 三格都 `ended=EveryBlockStored`，随机格 `flags` 只有那三项（`cards_used_gt…` 1008、`cards_used_lt…` 3476、`notrun_judged_unstored` 322，腿报 1076、3487、326：它的模型用真线程，计数每次不同），没出现的判错项照旧 0 |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`。

## 二、逐格判决

| 格 | 两腿 | 判 |
|---|---|---|
| K1-1 K1-5 叫停 | 两腿：叫停只出现时任一轮任一时点都不 panic、不判两遍、不漏判、照叫停收尾时判完的块全入库（攻方 200000 段、正推 20000 格）；「在等的」按比判完的最大块号小来判，两腿各推了两个方向，与「在等的」逐块相同（M01 / H4 `<=` 等价）。`cards_used` 按卡去重做到；正推：口径仍是「收过工的卡」，判了块又报 OOM 的不算、一块没领的算，只进汇总行 | 站住；`cards_used` 口径记下 |
| K1-2 (a) 比对 wgpu 原文 | 两腿：比对两头取自 `wgpu =30.0.1` 同一个 `Display`，给回调的只有 `self.lose(&error.to_string())` 一处；措辞变只回落判红。攻方：主机内存不够、碎片、gpu-allocator 都给「Out of memory」，主机内存不够说成显存不够，改前错误范围那一路就这样合并 | 站住；主机内存那一形不是这一批带进来的，记下 |
| K1-2 (b) 交回那一刻 | 两腿：甲形 2000 / 2000 认成显存不够，乙形设备没丢时 `None`。攻方量了第二段宽：默认钩子首次 49 µs、之后中位 9 µs，`RUST_BACKTRACE=1` 首次 7782 µs（harness 档进程，不是卡上）；第一段由丢设备那条线程在 wgpu-core 第 5420 与 5424 行之间被调走多久定。正推另报一形：驱动重置丢的设备加上别段记过 OOM，说成显存不够，是文档写明的设计 | 站住，两段窗口照实现员所报还在 |
| K1-3 照叫停收尾 | 两腿：库里只有块序前缀、没判的下一趟重算（攻方照叫停收尾 55771 段判完没入库 0）；代价是叫停加有卡报 OOM 时本次未跑那一行不打，OOM 缘由外面读不到 | 站住；缘由丢了并进下一批 |
| K2-1 账多给 | 两腿：账的算术这一批一行没动 | 没打中 |
| K2-2 叫停文件被删 | **攻方量到**：叫停每次现读文件在不在，四处读到的可以不同。文件在一轮收尾附近被删：轮末断言 panic「没人叫停，却有 4 块没判」1369 / 200000（改前就有）；照叫停收尾而第 5591 行再读时文件没了，整趟没记成提早收尾、进对账判红 2799 / 200000，其中最后一轮有卡报 OOM 的 1352 段是实现员第 2 处带进来的（改前那一形本次未跑、`gpu_memory_was_short` 恒真）。正推读码推出同一窗口，事件模型造不出。真机要在微秒级的缝里删文件（推的，没量） | **打中（窄）**，违反 K2-2「判红」；病根在共用的「叫停每次现读」，并进下一批 |
| K2-2 其余 | 两腿：叫停之后只判在等的那一轮按全部剩下的块借草稿区，多借到每张卡此刻带得了的量、持到这一轮收尾（正推 20000 格合计多算 315 个状态，模型单位） | 记下：方向是占着不用、不是多给 |
| K3 判别力 | **两腿各自量到同两处**：判完的最大块号改成最后判完的那一块（攻方 H1、正推 M05），在等 1 块也照叫停收尾（攻方 H19、正推 M10），lib 照绿、模型红。正推另 5 条照绿：轮末两条断言去掉（M13、M14）、`give_back` 只交回一次的断言去掉（M22）、头一句 OOM 取最后一句（M17）、认 OOM 取最早一条（M23），已写 `probes/killers_new.rs` 七条会红用例。两腿：checker 档接线四处没有不要卡的用例也没有变异行（交回那一刻的读法与位置、交 `after_the_round` 的参数、`cards_used` 改回 `outcome.cards.len()`、起下一张卡的看叫停）。攻方：丢失缘由那条用例期望值与产品代码取自同一个 `to_string()`，钉不住 wgpu-core 丢设备时给的那一句 | **打中**，补用例与变异 |

## 三、下一批（第 10z 批第六块）

一簇，都在 `gpu_memory_ledger.rs`、`crash_amplification.rs`、`gpu_unit_checks.rs` 与 `crates/mutations.tsv`；G2 若定乙，乙的实现落在同几份文件，并进同一批：
1. K2-2 叫停：一趟里见过一次叫停就记住（攻方 F2：外层循环与卡线程共用一个「这一趟见过叫停」，一轮收尾、整趟收尾读它，不再各自现读文件）；用攻方 `attack_10ze_rounds.rs` 的 `stop-file-removed` 一臂当用例，两样结局都要为 0。攻方提的、被攻过零轮。
2. K1-3：叫停加有卡报 OOM 而照叫停收尾时，把头一句 OOM 照本次未跑那一行的形态打出来。
3. K3：正推 `probes/killers_new.rs` 七条并进 `gpu_memory_ledger.rs` 测试段，M05、M10、M13、M14、M17、M22、M23 各一行变异（锚点与替换文在正推 `mut/mutants.py`）；checker 档接线能抽成 harness 档纯函数的抽出来测（攻方 F4：`catch_unwind` 之后立刻记交回那一刻设备丢没丢），抽不出的写明为什么；丢失缘由用例写死「Out of memory」，另加一条不要卡的用例扫 wgpu-core 源码里 `self.lose(&error.to_string());`（攻方 F5）。

记下、不改：`cards_used` 口径；叫停轮多借；主机内存不够说成显存不够；两段交回窗口。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：判的是显存账与第 ① 段派活的实现与测试，三条硬约束不变 |
