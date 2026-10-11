# m3-run-compare-split-r3 攻方腿（Opus）报告（2026-10-10）

立场：假设第二轮判决第三节七条与两轮判决第二节「坐实」的格是错的，造可达的改动序列或输入打穿它们。只攻分到的三问：第 3 问「已定」撑不撑得住、第 4 问未定项清单完整不完整、第 5 问两轮报出的数能不能核；不重复第一轮（漏、错配、落不下、分界越位、次序、第 9 问）与第二轮（修补本身、产生结论的方法、没被看过的地方）的角度。
代码引用：「副本」= `/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo`（主工作区 + V…E5，只读），带原文的副本引文写绝对路径；harness 与 RefFS 没被补丁动过的写主工作区路径。我在草稿目录 `~/.local/share/singlefs/work/m3-run-compare-split-r3-attack/copies/e5/` 拷了一份副本（`rsync -a --exclude target --exclude 'target-*' --exclude .git`），改与编都在拷贝上。每条结论标「读代码」「量过（我的模型）」「算的」「推的」之一；我的模型上的数只是模型上的数，不进 kb。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-run-compare-split-r3-opus-model/`。`<副本>` 是上面那份拷贝（或任一份主工作区 + V…E5 的树），`<草稿>` 是任一空目录。五样都不编 checker 档的测试、不起任何 checker 档测试二进制；history-enumerated 链接 checker 档的库（默认特性，不带 rocksdb、wgpu），只调 `crash::layer0_enumeration_tables` 与 `crash_injection::HistoryAfterTheMakeFilesystem::of` 两个公开函数，不摆崩溃状态。

```
# ① 闭包探针（#[path] 引副本的 judging_closure.rs，只用 std）：两份随机历史设点文件与几份候选入口出发的闭包
bash scripts/hosts/run-with-memory-cap.sh 2G nice -n 19 bash scripts/hosts/capped.sh 5 \
  bash research/prompts/m3-run-compare-split-r3-opus-model/run_closure.sh <副本> <草稿>
# ② 簇 0 真搬家一次（把 crash.rs 的枚举挪进叶子 crash_enumeration.rs，cargo check 编得过），量叶子闭包与按项多重集（先跑 ①）
bash scripts/hosts/run-with-memory-cap.sh 6G nice -n 19 bash scripts/hosts/capped.sh 5 \
  bash research/prompts/m3-run-compare-split-r3-opus-model/run_move_probe.sh <副本> <草稿>
# ③ 随机历史写表过 2048：录制口径与枚举写表口径各在第几步（36 段 × 144 步，我分三批、每批 3 件并行，每件 4 个种子）
bash scripts/hosts/run-with-memory-cap.sh 2G nice -n 19 bash scripts/hosts/capped.sh 5 \
  bash research/prompts/m3-run-compare-split-r3-opus-model/run_history_enumerated.sh <副本> <草稿> 0 36 144
# ④ 双机块换主：从副本原样抽 WeightedShares 的 Rust 源码，拿原函数 owner_of_block 核整周期表，再数换主比例
bash scripts/hosts/run-with-memory-cap.sh 1G nice -n 19 bash scripts/hosts/capped.sh 5 \
  bash research/prompts/m3-run-compare-split-r3-opus-model/run_owner_drift.sh <副本> <草稿>
# ⑤ 第一轮攻方的内核量法原样拷出来，在 E5、E7、E9 三份底子上各跑一遍
bash scripts/hosts/run-with-memory-cap.sh 6G nice -n 19 bash scripts/hosts/capped.sh 5 \
  bash research/prompts/m3-run-compare-split-r3-opus-model/run_kernel_measure_e7_e9.sh /home/fy5090/code/singlefs <草稿>
```

③ 我实际是先在草稿目录 `cargo build --release --offline`（不经包装，cargo build 不跑代码），再把编出来的二进制经 `run-with-memory-cap.sh 2G` 分 9 件跑（`{ …; echo "$?" > b<批>-<件>.rc; } &` 加一个 `wait`，三批各数到 3 个 `.rc`、全是 0）；九件输出按种子次序拼成 `outputs/history-enumerated-144-steps.out`。脚本 ③ 是同一件事的一条命令写法。

```
f7b1cd5f2617c760a606715603d3d9e1a04df72106b519d5944967cb18c55abd  ./closure-probe3/Cargo.toml
23d6f0451acd9ffad38fb1540d4591ea2851ab1f8cc85042dbe70d61e2d2bcce  ./closure-probe3/src/main.rs.template
45ac3bdebb7b6566417acf6364180c6990537d59e62dab0c7220d7fd26d479da  ./history-enumerated/Cargo.toml.template
1b980d6230db6032dc2c43e654f18dfb58e7d6ba95dfd1e281fa2ed56ae93a7a  ./history-enumerated/src/main.rs
f31cd85ce893a318d0fa2a5703663d8f7f1c4211e3d9913dd609286cf564a15c  ./move_enumeration.py
03e45389d9bd12e7b40f23ed786d56fec4c854b195ca2b724537924e26ee06c7  ./outputs/cluster0-item-acceptance.out
5d9ac1bd26297ec9ec8c9cb00b86bf6a66684074a69b19603d2ad8f7cc2177ef  ./outputs/history-enumerated-144-steps.out
c0205807c0d916970da0748fa7ef32cdfc26ae130db14ed45cdde783ac832bbc  ./outputs/history-family-closures.out
d539b36fe1f6d88c69d34a0566e80aa9dd8b12d761727872191476af1f888029  ./outputs/items-base-move2-crash.out
751165a60cbf8c78cb6b5f9276b2daf2fc248b92fc3b377f38d86ba20c836ee2  ./outputs/items-base-swap-crash.out
c3a065b3d85a03f3b85556c1c567660d600b7a62be91c751922c7263429fce54  ./outputs/items-loose-base-move2-crash.out
fdb47589c8dd96f153fe441bd0f2ceebff22e16588d306e0d4caecc3319a4d9b  ./outputs/kernel-measure-e5.out
fdb47589c8dd96f153fe441bd0f2ceebff22e16588d306e0d4caecc3319a4d9b  ./outputs/kernel-measure-e7.out
fdb47589c8dd96f153fe441bd0f2ceebff22e16588d306e0d4caecc3319a4d9b  ./outputs/kernel-measure-e9.out
342e18dbca489c644b907770612d2052f4636348642a116bf9da4b537a958541  ./outputs/leaf-closures.out
bc31496bd85dcea895a2a689c1c098dd47eeca7474cce580fa1522fa61852e7a  ./outputs/move-compile.out
bac31c79efbeeb6a68d900c047b111b070a9e64709a2a043709b7a7866f5b35f  ./outputs/owner-drift-rust.out
352add41e930892d13e50ced7904011d5a9e1360d4a1d106e72db94b11fa9962  ./owner-drift/Cargo.toml
abe7acbe07522f4e4614aa698e2f61838c6a729453cbc3103d55a07e8fc88eef  ./owner-drift/src/main.rs.template
fa3d664f7323a36351881a7f650ac53e31bec2b61000a3d89264e0ff09ae8392  ./run_closure.sh
daf4f221e96ce3f3c1bd325eb1dc4d4ea516fac067740829d29d96a9ce1e5cfc  ./run_history_enumerated.sh
be67787f8e1e9cc71cab395f7d3d573a641c44fb2ad86b315f10860b969d905e  ./run_kernel_measure_e7_e9.sh
e718a894964b0a8881ec68549dc81ca41df225d485b5065468344fd3f84d5592  ./run_move_probe.sh
879501c0a50256e19a7c1dac647c798866c4fd4ca696afca3f777e8c81a30246  ./run_owner_drift.sh
```

自证：① 副本 `crash_injection.rs` 出发的闭包摘要 `7d1ed58a…0d822da`、60 份文件，与第二轮攻方 `outputs/record-entry-closures.out` 里 tree=base 那一行逐字相同。② 第二轮攻方 R5 那一处对调（swap 树）在两种读法下都判 differ，搬家树在宽松读法下判 same——验收那一侧分得开。③ 录制口径那一列 36 个种子逐个与第二轮攻方 `outputs/history-writes-144-steps.out` 的 `first_step_over_2048` 相同（`paste` 逐行比，0 行不同）。④ 整周期表在 400 个位置上与原函数 `owner_of_block` 逐个相同才往下数（断言写在程序里），流水线五组换主数与第二轮 Python 的五个数逐个相同。⑤ 三份底子的输出与第一轮攻方 `outputs/kernel-measure.out` 逐字节相同（sha256 都是 `fdb47589…`）。崩溃状态 0 个：没有一样调 `enumerate_layer0*`，`closed_form_state_count` 累计 0 态。

取样与挂钟：① 两份设点文件、四份候选入口，几秒；② 三棵树（base、move、move2）加 swap，cargo check 两次，整条 2 分钟；③ 先跑 1 段 144 步估时（18 秒），36 段 ≈ 11 分钟单件，分 3 批 × 3 件，每批 1–2 分钟，没缩取样；④ 五对权重，秒级；⑤ 三份底子各 15–35 秒。

## 各格判定一览

| 格 | 问 | 被打的条 | 判定 | 数怎么来的 | 节 |
|---|---|---|---|---|---|
| T1 | 3 | 第 2 条「录入版本照 H8 并上设点测试文件出发的闭包」× 第 3 条「族闭包里不许有 CPU 判器入口」 | 打中，自相矛盾：两份随机历史设点文件出发的闭包今天就含 CPU 判器入口 `crash.rs`（各 62 份）；拆两段后设点文件仍要调对比那一半，按字面这条检查在「历史跑数据」族上每趟红；只判叶子那一半，H8 那一半又把 CPU 判器拉回这一族的录入版本 | 量过 | 一.1 |
| T2 | 3 | 第 3 条叶子与检查、第 6 条簇 0 清单 | 打中，两处：只拆 `crash.rs` 做不出无 `crash.rs` 的叶子（`crash_facts.rs:28` 回引）；补这一行之后叶子闭包过了检查，却罩着第 ② 段的 CPU 判法（`verify_state_from_facts` 住在 `crash_facts.rs`），改第 ② 段判据就换 `10`/`11` 的录入版本，检查照字面绿 | 量过（编得过的真搬家） | 一.2 |
| T3 | 3 | 第 6 条「按项多重集（去 use、mod、可见性）」 | 打中，假红：同一次编得过的纯搬家，按项多重集判 differ——4 项是项内字段与方法补的 `pub(crate)`，10 项是 `use a::{…};` 切出来的孤立 `;`；项内可见性也去、孤立 `;` 不算，搬家 same、对调 differ | 量过 | 一.3 |
| T4 | 3 | 第 3 条「派活那一刻拼进实际编进管线的源码摘要（现成的 kernel_sources_digest）」 | 打中，时点与覆盖：复用判定在开库之前、按计划时就算好的版本做；`kernel_sources_digest` 只认三个判器内核、管线恰好三条；两条自证撤了以后没有一条会红的检查盯「每种结论都拼了」 | 读代码；后果推的 | 一.4 |
| T5 | 3 | 第 7 条「按依不依赖已判红的参照分」 | 部分打中（推的）：分法只认两个参照的红；实现自己先红（历史执行器的 harness 判、每步 checker）之后，后面的票不依赖参照、照常进总数、照登记新发现，今天这些票根本不存在 | 读代码 | 一.5 |
| U | 4 | 未定项清单 | 打中：两轮判决与四份腿报告里提过、没进第三节、交用户表、零轮与欠账任何一处的，数出 14 条（U1–U14） | 读两轮报告与判决（`grep -c` 核过判决里没有） | 二 |
| N1 | 5 | 「写表过 2048 在 71–136 步」 | 打中，口径反了：第二轮攻方说 GPU 掩码数的「不会比我数的多」，实际枚举写表多出 180–246 个撕裂镜像与重放；按建判表那一判的口径，36 段里 35 段在第 64–140 步过 2048，比 71–136 早 4–24 步 | 量过 | 三.1 |
| N2 | 5 | CPU「116–525 线程时」 | 打中，基数旧：乘的是 26.16 亿（28 深），同两轮自己的 30 深下界 37.4 亿算是 166–751 线程时（仍是下界、旧树单线程参考数） | 算的 | 三.2 |
| N3 | 5 | 展开量 62660 / 387189 | 核过：第一轮攻方的量法在 E5、E7、E9 三份底子上输出逐字节相同；E9（`MAX_RECORDS` 512 → 2048）调查报告写「展开量不变，推的，没量」，这里量了 | 量过（第一轮攻方的模型） | 三.3 |
| N4 | 5 | 块换主 40–43% | 核过：从副本原样抽 `WeightedShares` 的 Rust 源码跑，流水线五组换主数与第二轮 Python 逐个相同；给定状态表换一组替身是 40.7–43.1% | 量过 | 三.4 |
| N5 | 5 | 层 0 片与块对不齐 | 核过公式（读代码）；三个态数里两个是放量分项的态数、不是层 0 用例的枚举域，结论不受影响 | 读代码 + 算的 | 三.5 |
| N6 | 5 | 389.8 GiB / 598.9 GB | 算式复算相同；160 字节是事实表按字段推的、没量；两个数基数不同（28 深与 30 深下界） | 算的 | 三.6 |
| N7 | 5 | 各数一张来路表 | 给出：哪个只在腿的模型上、入库装置上重做要什么 | — | 三.7 |

## 一、第 3 问：「已定」撑不撑得住

### 一.1 T1 H8 那一把闭包里有 CPU 判器入口：第 2 条与第 3 条对不上（量过）

被判对象许可它的两句（第二轮判决第三节原行）：`research/prompts/m3-run-compare-split-r2-main-verification.md:54` 那一行写「录入版本照 H8 并上设点测试文件出发的闭包」，`:57` 那一行写「族闭包里不许有 `*.wgsl`、CPU 判器入口与别族叶子」。CPU 判器入口就是 `crash.rs`：「"crates/singlefs-checker-tier/src/crash.rs",」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/judging_closure.rs:638`）。

量过（`outputs/history-family-closures.out` 原样节选，摘要截掉）：

```
CLOSURE entries=crates/singlefs-checker-tier/tests/crash_injection_campaign.rs files=62 … wgsl=0 cpu_judge_entry_crash_rs=true gpu_judge_entry_crash_judge_gpu_rs=false crash_injection_rs=true crash_given_states_rs=true harness_model_rs=true harness_history_rs=true reffs_lib_rs=true core_recovery_rs=true
CLOSURE entries=crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs files=62 … wgsl=0 cpu_judge_entry_crash_rs=true … crash_given_states_rs=true …
```

两份随机历史设点文件（第 2 条里「历史跑数据」这一族只有它们两项）出发的闭包都是 62 份、都含 `crash.rs`，也都含 `crash_given_states.rs`。拆两段之后这两份测试照样要调对比那一半——它们断言的是判定（读故障态那份数「判红的态」，战役那份数红绿），所以它们的闭包恒含 CPU 对比入口，拆不掉。

两种读法都过不去：
- 「族闭包」= 叶子闭包 ∪ H8 那一把（录入版本实际罩的那一份）：检查在这一族上每趟红，做不成。
- 「族闭包」= 只看叶子：检查绿，但录入版本里 H8 那一把仍含 CPU 判器，改 `crash.rs` 一处就换 `12`/`14` 的录入版本。第二轮 R3 要这条检查挡的正是「判器改了就重录」；在这一族上它挡不住，只是代价小（重录整族的上界是 24 段崩溃注入跑一遍，第二轮辩方引的 16.3 秒，`research/prompts/m3-run-compare-split-r2-sonnet-output.md:145`）。
另：两份闭包都含 `crash_given_states.rs`（给定状态表的类型与 CPU 就地判），若它被定成哪一族的叶子，「不许有别族叶子」同样红。

四句：分辨臂——分（「检查罩录入版本的全部闭包」与「检查只罩叶子、H8 那一把豁免并写明」两条臂，一个做不成、一个做得成）；系统看得到——检查那一刻手里有整份闭包清单，看得到；分句——第 3 问「叶子文件与族闭包里不许有 WGSL 与判器入口那条检查做不做得出来」；跑前改法——第 3 条只有这一条检查，没有给 H8 那一把开口子的写法。
改法（推的、被攻过零轮）：检查只判叶子出发的那一把，H8 那一把在版本里另记一段、不进检查，判决写明「`12`/`14` 的录入版本含 CPU 判器，判器改了这一族重录，代价按秒计」。

### 一.2 T2 叶子做得出，但要多拆一份；拆完之后检查罩不住第 ② 段判法（量过）

改动序列（真做了一次，编得过）：把副本 `crash.rs` 的枚举那几段原样挪进新文件 `crash_enumeration.rs`——发布归属（第 339–412 行）、段归属（532–548）、枚举表（1561–1630）、展开与撕裂与状态计划（1886–2508），一共 784 行；`crash.rs` 补一行 `pub use` 与一行 `use`，`lib.rs` 加 `pub mod`，新文件抄 `crash.rs` 原来的 use 块；再按 cargo check 的报错加 `pub(crate)`：8 处项头、12 处项内（`WritesWithTornImages.writes`、`Layer0StatePlan` 的 4 个字段与 6 个方法、`WritesWithTornImages::of`）。这一段就是第 3 条要的「`10`/`11` 族叶子」里必有的那一块（从状态序号解出持久集合，`crash_facts.rs` 文件头说第 ① 段抽事实也走它）。

`outputs/move-compile.out` 与 `outputs/leaf-closures.out` 原样节选（摘要截掉）：

```
CHECK tree=move compiles=yes
CHECK tree=move2 compiles=yes
tree=move CLOSURE entries=crates/singlefs-checker-tier/src/crash_enumeration.rs files=48 wgsl=0 cpu_judge_entry_crash_rs=true …
tree=move2 CLOSURE entries=crates/singlefs-checker-tier/src/crash_enumeration.rs files=46 wgsl=0 cpu_judge_entry_crash_rs=false …
```

- move（只拆 `crash.rs`）：叶子闭包仍含 `crash.rs`。回路是枚举里的分段函数调 `crate::crash_facts`，而 `crash_facts.rs` 回引 `crash.rs`：「use crate::crash::{layer0_enumeration_tables, Layer0EnumerationTables, Layer0SegmentExpansion};」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_facts.rs:28`）。第 6 条簇 0 的清单（拆 `crash.rs`、`crash_injection.rs`、挪 `GivenStatesEngine`、流水线录入段搬出 `crash_amplification.rs`）没有 `crash_facts.rs`。
- move2（再把那一行改指叶子）：叶子闭包 46 份，没有 `crash.rs`、没有 WGSL，检查绿。但这 46 份里有 `crash_facts.rs` 本身，而它是第 ② 段判状态的 CPU 参照：「第 ② 段读事实判状态：GPU 内核在 `crash_verify_gpu`，CPU 参照实现是这里的 [`verify_state_from_facts`]」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_facts.rs:9`）。改第 ② 段的一条判据就换 `10`/`11` 的录入版本，26 亿态的 Rust 恢复两遍重录（代价量级见三.2，推的）——R3 要挡的同一种事，检查的三样禁物（WGSL、CPU 判器入口、别族叶子）都不碰它。

四句：分辨臂——分（「检查列三样禁物」与「叶子闭包里只许有录入与枚举本身」）；系统看得到——看得到（闭包清单里就有 `crash_facts.rs`）；分句——第 3 问「叶子文件与那条检查做不做得出来」；跑前改法——第 3 条的禁物表碰不到这一格。
改法（只在我的模型上量过前半、被攻过零轮）：簇 0 清单加 `crash_facts.rs`，按「枚举与抽事实」/「第 ② 段判法」拆两份；检查的禁物表加「任何一种结论的入口文件」，并在 move2 这类树上配一条会红的样本（move2 照今天的 `crash_facts.rs` 应判红）。

### 一.3 T3 按项多重集在一次编得过的纯搬家上判红（量过）

被判对象许可它的那一句：`research/prompts/m3-run-compare-split-r2-main-verification.md:67` 那一行「簇 0 的验收改成按项多重集（去 `use`、`mod`、可见性），另配名字解析不变的检查」。它采的是第二轮攻方 R5 的改法；R5 的两棵树（move、swap）没编译（`research/prompts/m3-run-compare-split-r2-opus-output.md:361` 自己写了），move 里只给项头加了 `pub(crate)`、新文件的 `use` 都是不带花括号的单行。

我用的就是一.2 那一次编得过的搬家（move2），切项照 R5 的写法（花括号深度回到 0 的 `}` 或深度 0 的 `;` 收一项，丢 `use`、`mod x;`，去项头 `pub`/`pub(…)`）。`outputs/cluster0-item-acceptance.out` 原样：

```
pair=base-move2 ITEMS mode=head-visibility-only entries=crates/singlefs-checker-tier/src/crash.rs items_a=2744 items_b=2754 only_in_a=4 only_in_b=14 verdict=differ
pair=base-move2 ITEMS mode=head-visibility-only entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs items_a=2897 items_b=2907 only_in_a=4 only_in_b=14 verdict=differ
pair=base-swap ITEMS mode=head-visibility-only entries=crates/singlefs-checker-tier/src/crash.rs items_a=2744 items_b=2744 only_in_a=1 only_in_b=1 verdict=differ
pair=base-swap ITEMS mode=head-visibility-only entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs items_a=2897 items_b=2897 only_in_a=1 only_in_b=1 verdict=differ
pair=base-move2 ITEMS mode=loose entries=crates/singlefs-checker-tier/src/crash.rs items_a=2522 items_b=2522 only_in_a=0 only_in_b=0 verdict=same
pair=base-move2 ITEMS mode=loose entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs items_a=2662 items_b=2662 only_in_a=0 only_in_b=0 verdict=same
pair=base-swap ITEMS mode=loose entries=crates/singlefs-checker-tier/src/crash.rs items_a=2522 items_b=2522 only_in_a=1 only_in_b=1 verdict=differ
pair=base-swap ITEMS mode=loose entries=crates/singlefs-checker-tier/src/crash_judge_gpu.rs items_a=2662 items_b=2662 only_in_a=1 only_in_b=1 verdict=differ
```

差在哪（`outputs/items-base-move2-crash.out` 原样，每项只打前 14 个词）：

```
ITEM_ONLY_IN_B x10 ;
ITEM_ONLY_IN_A x1 impl < 'plan > Layer0StatePlan < 'plan > { fn new ( writes :
ITEM_ONLY_IN_B x1 impl < 'plan > Layer0StatePlan < 'plan > { pub ( crate ) fn
ITEM_ONLY_IN_A x1 impl WritesWithTornImages { fn of ( base : & MemoryPool , writes : &
ITEM_ONLY_IN_B x1 impl WritesWithTornImages { pub ( crate ) fn of ( base : & MemoryPool
ITEM_ONLY_IN_B x1 struct Layer0StatePlan < 'plan > { pub ( crate ) segments : & 'plan
ITEM_ONLY_IN_A x1 struct Layer0StatePlan < 'plan > { segments : & 'plan [ Vec < usize
ITEM_ONLY_IN_B x1 struct WritesWithTornImages { pub ( crate ) writes : Vec < RetainedWrite > ,
ITEM_ONLY_IN_A x1 struct WritesWithTornImages { writes : Vec < RetainedWrite > , torn_image_by_write : BTreeMap <
```

- 4 对是项内的可见性：私有字段与私有方法被搬出原模块，编译器要它们 `pub(crate)`，这几个字落在项的花括号里，「去项头可见性」碰不到。私有结构体两边都用（`Layer0StatePlan` 这个名字在副本 `crash.rs` 里 `grep -c` 命中 14 行，去掉定义、impl 与一行文档注释是 11 处用到）的搬家都会这样。
- 10 个孤立的 `;`：新文件要自己的 use 块，`use a::{b, c};` 的 `}` 先把深度带回 0 收了一项，剩下的 `;` 单成一项，丢 `use` 那一判认不出它。读法放宽成「可见性哪儿都去」，这 10 项照样让它判 differ。
- 宽松读法（项内任何位置的 `pub`/`pub(…)` 都去、孤立 `;` 不算项）：move2 判 same、swap 判 differ，两头都对。

四句：分辨臂——分（「项头可见性」读法与宽松读法一假红一不假红）；系统看得到——验收那一刻手里有两棵树的全部词，看得到；分句——第 3 问「第 6 条按项多重集验收的漏处」，这一格是假红那一头（R5 自报的漏处是假绿那一头：同名项互换、`use … as`）；跑前改法——第 6 条只有这一个验收，配的名字解析检查不管它。
改法（只在我的模型上量过、被攻过零轮）：切项时项内的可见性记号一律去掉，`use … {…};` 整条当一项丢；另加一条「每个被搬的项在新文件里出现恰好一次」的计数，免得宽松之后两份一样的项各丢一半也判 same。我的宽松读法只在这一次搬家与 swap 一处上量过；它看不见可见性本身改错（推的：本来就不改判定）。

### 一.4 T4 「派活那一刻拼进 `kernel_sources_digest`」：时点在复用判定之后，助手只认三个内核（读代码）

被判对象许可它的那一句：`research/prompts/m3-run-compare-split-r2-main-verification.md:58` 那一行「结论版本的 `extra_inputs` 绑它读的录入版本，并在派活那一刻拼进实际编进管线的源码摘要（现成的 `kernel_sources_digest`）」。

1. 时点：一块要不要判，在开库之前就按计划时算好的版本定了。「// 结论的版本（J8）：开库之前算、与编译时写进二进制的那一份核」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:7627`），这一把版本交给「pub fn record_crash_points_needing(」（同文件 `:1113`），它逐块问库「这一种结论、这一把版本在不在」，在就整块复用、不派活。派活在这之后。照字面在派活那一刻才把内核摘要拼进 `extra_inputs`，有两种做法，都不对：写库用拼过的那一把、查库用没拼的那一把——每块都查不到，永远重判（不出错，复用全丢）；或者查与写都用计划时那一把、派活时拼的那一份没进任何一把——第二轮 R2 那个洞原样留着。要的是在排计划那一刻就算出「将要编进管线的源码」的摘要并进版本，派活时再核编的就是它。
2. 覆盖：「fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:163`）只读 `JudgeKernelSources` 的三个字段（恢复、走读、扫描），结构体就这三样（同一副本 `crash_judge_gpu.rs:535` 起），派活取管线时按摘要缓存、断言「这个摘要那一格只由 compile_the_three_judge_kernels 填，里面恰好三条管线」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:578`）。第 5 条要新写的 P14 内核与六合一内核都不在这三样里，「现成的」用不上，要另写一份；`crash_verify_gpu.rs` 与 `gpu_unit_checks.rs` 自建管线、也不经它。
3. 没有会红的检查：第 3 条同时撤了第一轮的两条自证，「每一种用了内核的结论都拼了它的摘要」再没有任何一条检查盯着；忘拼一处就是 R2 的假绿，今天的门禁看不出。

四句：分辨臂——分（「派活时拼」与「排计划时现算、派活时核」）；系统看得到——复用判定那一刻看得到的是计划时的版本，派活时的摘要它看不到，这就是「判别子观测不到」那一形；分句——第 3 问「已定」撑不撑得住；跑前改法——第 3 条没给别的写法。
改法（推的、被攻过零轮）：版本在排计划时并进「这一种结论将要编进管线的全部源码」的摘要（源码由常量拼出，开跑前就算得出），派活时断言编的源码摘要与它相同；配一条会红的样本：给某种结论的内核改一个字、不改它的入口文件，跑一趟剪枝，那一种结论必须重判（第二轮 k-gpu-edit 那棵树就是现成的样本）。

### 一.5 T5 第 7 条只认参照的红，不认实现自己先红（读代码，后果推的）

被判对象许可它的那一句：`research/prompts/m3-run-compare-split-r2-main-verification.md:71` 那一行（理想模型第一处红之后、RefFS 自己第一处红之后各自那一类单列，「不依赖的照常记」）。
今天一段历史里三样任一判红就停：「if !violations.is_empty() || harness_judgement.is_some() || model_disagreement.is_some()」（`crates/singlefs-harness/src/history.rs:5897`）。第一轮第 1 条把历史执行器的 harness 判（分配记录的代、冷启动读回报错）划进对比那一列，第 7 条又让历史跑到实现自己停。于是：第 k 步分配记录的代写错（harness 判红，实现自己的毛病），历史照跑，第 k 步之后摆的崩溃点上池级 checker、记录核对器、oracle 判出的红，多半是第 k 步那个毛病的连锁；它们不依赖模型、不依赖 RefFS，第 7 条照常进总数、照常登记新发现。这一类票今天一张都不存在（第 k 步就停了）。
与第二轮 R7 不同：R7 是参照（RefFS）自己红之后票的含义失效；这里票本身有效（镜像真坏），错在「新发现」的计数——一个毛病在发现日志里按签名记成好几个。冷启动读回报错（`ColdStartRecoveryFailedOnCheckerGreenImage`）那一支更进一步：恢复失败之后执行器拿什么池接着跑下一步，今天的代码没有定义（它停了），推的。
四句：分辨臂——分（「按参照分」与「按第一处红分，不管谁红」）；系统看得到——对比段判第 m 步时手里有第 k 步那一判的结果，看得到；分句——第 3 问；跑前改法——第 7 条只有这一种分法。
改法（推的、零轮）：第 7 条加一类：历史里实现自己第一处红（harness 判、每步 checker 判红）之后的票照判、单列、不登记新发现；冷启动读回报错时录入段照今天停。

## 二、第 4 问：未定项清单完整不完整

判法：两轮判决（`research/prompts/m3-run-compare-split-r1-main-verification.md`、`…-r2-main-verification.md`）的第三节七条、第四节零轮与欠账、第五节交用户表里，按关键词 `grep -c` 数过都是 0 次（或命中的那一行说的是别的事）的，才列进来。来源写腿报告的行号（现取）。

| # | 没落处的问题 | 谁提的（文件:行） | 为什么要定 | 该去哪（推的） |
|---|---|---|---|---|
| U1 | P8 每步模型比在开 GPU 时落哪个内核、要不要在 WGSL 里重写一份模型判法 | 第一轮攻方 D4（`research/prompts/m3-run-compare-split-r1-opus-output.md:242`，「造第二份规范」）；第一轮正推 9.4 节（做得出、无吞吐收益） | 用户原话「开启了gpu的时候，所有的对比都需要用gpu对比」；两轮第 5 条列了 P1–P3、P5、P7、P10–P12、P14–P17，没有 P8；它又是项目唯一功能对照物的第二份实现 | 交用户表加一行（上 GPU 重写 / 留 CPU 并写明是用户原话的例外），零轮 |
| U2 | 拒绝合法性（B5）吃理想模型那一票（B1）的结论，GPU 上 P14 要排在 P10 之后、或把 P10 的结论位当 P14 的输入列 | 第一轮正推 G5（`research/prompts/m3-run-compare-split-r1-sonnet-output.md:98`） | 第 5 条把 P10 放进六合一新内核、P14 单独成内核，两个内核谁先派、怎么传结论没写 | 第 5 条补一句 |
| U3 | 观察者另读的流程输入（`versions_with_a_file`、`deletion_publishes`）不在 `0F`；观察者的挂载跑在没有录制层的盘上，挂载写录不下来 | 第一轮正推 Q2 表（`…-r1-sonnet-output.md:140`、`:138`） | P5 上 GPU 要这两样；M2 要数挂载写分组也先要录制层 | 第 2 条（表）补 `0F` 一行与录制层，归丁一 |
| U4 | 坏盘输入战役（`bad_disk_input.rs` 里 CPU 上的 `check_pool_image`）在不在两段形态的射程里 | 第一轮正推 G6（`…-r1-sonnet-output.md:99`，「要主 agent 定射程」） | 它是 checker 档战役、没有 GPU 路；不在射程里要写一句，在就缺一整格 | 第 1 条分界加一句 |
| U5 | P3 的「Rust 录入 vs WGSL 恢复」比一致，不该随 J6 被用户授权跳过一起跳 | 第一轮正推 P3 行（`…-r1-sonnet-output.md:59`） | 第 5 条把 P1–P3 并进恢复内核，没说 J6 跳过时这一比还在不在；它是拆两段之后 CPU 录入与 GPU 恢复唯一的对齐 | 第 5 条补一句 |
| U6 | 开跑前的环境检测（卡上判一个已知红的小输入，红不出来就退 78） | 第一轮攻方 E3（`…-r1-opus-output.md:264`）；第一轮判决第二节写「另记一条」；第二轮辩方说它零轮、没进第三节（`…-r2-sonnet-output.md:45`） | 第一轮判决说「另记」，两轮的第四节都没有这一条，记在哪查不到 | 欠账或第 1 条左列，零轮 |
| U7 | 改名不许动设点文件的路径与崩溃点名（节点身份进块键） | 第一轮正推 10.1 节（`…-r1-sonnet-output.md:327`） | 判决只记「改名先做：一致」，前提没跟着进；动了就是全量重录 | 第 6 条簇的次序那一句补前提 |
| U8 | `lib.rs` 是每个加模块的簇都要改的撞点 | 第一轮正推 10.2 节第 5 条（`…-r1-sonnet-output.md:335`） | 第 6 条的簇 0、丙、丁二都加模块；实现员派发闸按文件查撞车 | 第 6 条派发写法里错开 |
| U9 | `--full` 在两层上的含义（「全量 = 重录 + 重比」写进弹窗）；要不要另立「只重比不重录」一档 | 第一轮正推 Q8（`…-r1-sonnet-output.md:382`） | 第 4 条把录入库搬出双机拷库的目录之后，`--full` 换新库时换不换录入库没写；换就是几天的 CPU，不换就不是「清空复用」 | 交用户表加一行，零轮 |
| U10 | 历史不再停在红步之后，同一个种子抽到的崩溃点会变；已知红清单里依赖「停在第几步」的签名要重核；联合判的分类名在红步之后不成立 | 第一轮正推 12.3 节（`…-r1-sonnet-output.md:410`、`:402`） | 第 7 条只定了计数怎么分，没定已知红清单与联合判分类怎么改 | 第 7 条补两句 |
| U11 | 不依赖参照的那一类在红步之后照常记，但仍要带用户要的「红步之后」那一标 | 第二轮辩方（`…-r2-sonnet-output.md:41`） | 第 7 条写「不依赖的照常记」，没写保留那一标；照第 7 条字面实现，用户那半句（标在红步之后）就丢了 | 第 7 条补半句 |
| U12 | 「已知红」归类只给第一处红做；依赖模型的红不钉回快档 | 第一轮攻方第七节（`…-r1-opus-output.md:295`） | 第 6 条的钉回只写了「钉内容、GPU 才红的进 GPU 组」，没写红步之后的红钉不钉 | 第 7 条或第 6 条钉回那一句补 |
| U13 | 变异复跑的三份脚本不设也不清 `SINGLEFS_CRASH_AMPLIFICATION_*`，调用方环境带着它时录入跨趟复用 | 第二轮攻方 N4（`…-r2-opus-output.md:345`，「这一条我没核」） | 拆两段之后录入层复用多了一层，变异行改了停止文件里的录入代码时可能吃旧录入；没核、也没进欠账 | 欠账一行（先核） |
| U14 | P14、P17 的 GPU 一份与恢复内核共用同一段 WGSL 择根时，「已知红」归类与恢复判共享缺陷 | 第一轮攻方 E2（`…-r1-opus-output.md:259`） | 第一轮判决第二节「坐实：要写明」，两轮第三节都没写 | 第 5 条补一句 |
| U15 | 轻型判红时留下那一块的录入行看现场，不随库删 | 第一轮正推 Q8（`…-r1-sonnet-output.md:377`） | 拆开之后红态正文靠「现场重跑补细节」（第一轮第 1 条第三列），轻型库跑完就删 | 小事，第 1 条第三列补一句或不采并写明 |

另有三条是这一轮自己冒出来的零轮形态，照规则不再开轮，写进交用户表、标「零轮」：一.1 的「检查豁免 H8 那一把」、一.2 的「簇 0 加 `crash_facts.rs`、禁物表加结论入口」、一.4 的「排计划时并进内核摘要、派活时核」。一.3 的宽松按项多重集只在我的模型上量过一次搬家。

推翻条件：判决或欠账表（`.claude/kb/checks-owed.md`）里另有一行把上表某一条的意思写了、只是用词不同——那一条撤。我只按关键词数，没逐句读欠账表全文。

## 三、第 5 问：两轮报出的数能不能核

### 三.1 N1 「写表第一次过 2048 在第 71–136 步」：口径数错了方向（量过）

两轮的说法并排：第二轮攻方「GPU 掩码按 `layer0_enumeration_tables` 的枚举写表数，不会比我数的多——所以过 2048 的那一步只会比我量的晚」（`research/prompts/m3-run-compare-split-r2-opus-output.md:288`）；第二轮辩方「探针日志只数录制流的写，没数撕裂镜像与重放，所以 2048 位的真实封顶只会比下面的数更低」（`research/prompts/m3-run-compare-split-r2-sonnet-output.md:101`）。两句方向相反，第二轮判决采了攻方的量（`research/prompts/m3-run-compare-split-r2-main-verification.md:28`），没判方向。
代码（读代码）：给定状态表建判表那一判比的是枚举写表的长度——「let enumerated = &tables.enumerated_writes;」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_judge_tables.rs:387`）、「if enumerated.len() > MAXIMUM_ENUMERATED_WRITES {」（同文件 `:388`），而枚举写表是录制流的写再接上撕裂镜像与重放：「enumerated_writes: writes_with_torn_images.writes.clone(),」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash.rs:1596`）。给定状态表的写表就是整段历史 mkfs 之后的写（同一副本 `crash_injection.rs:2077`）。所以枚举写表不短于第二轮攻方数的那一份，辩方的方向对。
量过（history-enumerated，照崩溃注入建给定状态表的同一条路逐步取前缀；`outputs/history-enumerated-144-steps.out` 36 行，按种子次序；下面两行由 `grep -o 'first_step_<口径>_over_2048=[a-z0-9]*' … | cut -d= -f2 | sort -n` 出）：

```
录制口径：  none none 71 75 77 78 78 80 80 81 82 82 84 86 87 89 90 92 92 93 93 94 94 95 97 98 103 107 107 117 119 120 125 134 135 136
枚举写表口径：none 64 70 71 72 72 73 74 74 74 75 75 79 80 82 83 84 85 85 86 86 87 87 91 92 93 96 98 100 109 112 113 113 118 119 140
```

原行形如 `HISTORY_ENUMERATED steps=144 seed_offset=0 steps_run=144 first_step_recorded_over_2048=84 first_step_enumerated_over_2048=74 recorded_and_enumerated_at_recorded_crossing=2063/2283 last_step_measured=84 recorded_there=2063 enumerated_there=2283 tearable_there=220 seconds=17.97`。录制口径那一列 36 个种子逐个与第二轮攻方的产物相同（自证）。枚举写表多出来的个数在 36 行里都恰好等于取三态的写数（180–246，`awk` 核过 36 行相等）：多的是每次原地覆写一份撕裂镜像，这几段历史在不展开时没有重放。结论：按建判表那一判的口径，36 段里 35 段在第 64–140 步就建不出 GPU 判表，比第二轮报的早 4–24 步（逐个种子相减）；第二轮写「第三截更早、没量」那一句照旧（第三截再加挂载写，我也没量）。
翻不翻面：快档 24 步离得远，第三节没有哪一条因此翻面；改的是零轮数的口径，判决里那一行「采攻方的量」要改成枚举写表口径。
限度：一组比重（BROAD）、一种盘宽、36 个种子，与第二轮同一批；只数到第一次过 2048 为止。

### 三.2 N2 CPU「116–525 线程时」：基数是 28 深的 26.16 亿（算的）

来路（读第一轮攻方模型）：`research/prompts/m3-run-compare-split-r1-opus-model/volume_and_cpu.py` 取 E161 第二轮单线程产物 `research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-threads1-2026-10-05.out` 的 g3 五格，每格「恢复看 journal + 不看 journal」两项纳秒之和除以态数，得每态 159583–722596 纳秒；乘 26.16 亿、除 3600 秒、换成线程时。产物在仓里、今天还在（`ls research/results/` 有这一份），五格都是 45–4096 态的小格、2026-10-05 的树、E142 那两条流。
复算（`python3 -c`，本机这一次的原样输出）：`[116, 260, 525] [166, 372, 751]`——前一组是 26.16 亿态，后一组是两轮自己报的 30 深去重下界 3 743 376 012 态。两轮报体量时一个用 26.16 亿（389.8 GiB）、一个用 37.4 亿（598.9 GB），报 CPU 只用了 26.16 亿；今天登记的全量深度是 30，所以 CPU 那一行该是「≥ 166–751 线程时（下界、旧树单线程参考数）」。不翻第三节任何一条（M1 照样要量），改的是第一轮判决第二节 CPU 那一行与交用户表第 3 行的代价口径。

### 三.3 N3 展开量 62660 / 387189：E7、E9 上不变（量过，第一轮攻方的模型）

第一轮攻方的量法原样拷到草稿目录，在 E5、E7、E9 三份底子上各跑一遍（复跑命令 ⑤）。E7 与 E9 都改了 `crash_judge_common.wgsl` 与 `crash_judge_checker_common.wgsl`（sha256 前 12 位三份各不相同），E9 把 `MAX_RECORDS` 从 512 抬到 2048。三份输出逐字节相同（`outputs/kernel-measure-e5.out`、`-e7.out`、`-e9.out` 的 sha256 都是 `fdb47589…`），前五行：

```
kernel=recovery KERNEL_INLINED entry=main expressions=334042 margin_to_400000=65958
kernel=checker-scan KERNEL_INLINED entry=main expressions=395566 margin_to_400000=4434
kernel=checker-walk KERNEL_INLINED entry=main expressions=352481 margin_to_400000=47519
probe=p14-alone KERNEL_INLINED entry=p14_probe expressions=62660 margin_to_400000=337340
probe=p14-merged-into-recovery KERNEL_INLINED entry=merged_probe expressions=387189 margin_to_400000=12811
```

E9 的调查报告写「展开量不变（只改常量值），这一点是推的，没量」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/j4-after-e6/report.md:15`）：在这个模型上量了，不变。限度照旧：生成常量那一段用 `u32 = 1u` 替身（第一轮攻方的做法）；P14 两个数是只摆调用点、判据本体一行没写的下界。

### 三.4 N4 块换主 40–43%：拿原函数核过（量过）

第二轮那一份是 Python 照抄 `owner_of_block`。我从副本原样抽 `WeightedShares` 的结构与 impl（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:962` 起 37 行），先在 400 个位置上拿原函数「pub fn owner_of_block(&self, block_sequence: u64) -> usize {」（同文件 `:982`）核整周期表，再数换主（`outputs/owner-drift-rust.out` 原样节选）：

```
OWNER_DRIFT_RUST kind=pipeline before=136700,45275 after=140427,62390 blocks=39917 moved=16094 moved_fraction=0.4032 checked_against_owner_of_block=400+400
OWNER_DRIFT_RUST kind=pipeline before=140427,62390 after=642805,291558 blocks=39917 moved=17007 moved_fraction=0.4261 checked_against_owner_of_block=400+400
OWNER_DRIFT_RUST kind=pipeline before=136700,45275 after=136701,45275 blocks=39917 moved=2178 moved_fraction=0.0546 checked_against_owner_of_block=400+400
OWNER_DRIFT_RUST kind=given_states before=136700,45275 after=140427,62390 tables=20000 moved=8144 moved_fraction=0.4072
OWNER_DRIFT_RUST kind=given_states before=140427,62390 after=642805,291558 tables=20000 moved=8618 moved_fraction=0.4309
```

流水线五组与第二轮 `outputs/owner-drift.out`、`owner-drift-small.out` 的五个 `moved=` 逐个相同（16094、17007、16189、2178、0）。给定状态表我换了一组替身指纹（splitmix64），得 40.7–43.1%，第二轮是 40.3–42.4%：指纹按权重总和取模，比例不依赖替身。两处口径要写明：块数 39917 是 26.16 亿 ÷ 65536 的总数，不是逐项的块号（块号按项从 0 起，比例同量级，推的）；三组权重来自 `runs/` 下三趟日志，不是三组「典型」权重。第三节第 4 条已经把归属写进库，这个数不再撑任何一条，只撑「为什么要写进库」。

### 三.5 N5 层 0 片与块对不齐：公式对，取样的态数有两个不是层 0 用例的

公式读代码：片数「const LAYER0_RESUMABLE_SLICE_COUNT: u64 = 65_536;」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash.rs:2537`），片长 = ⌈态数 ÷ 65536⌉、至少 16（同文件下一个函数）；块长 65536 态。第二轮攻方的三个态数：1358954634 是多记录流层 0 用例的枚举域（副本 `tests/crash_enumeration_multi_record_publish_stream.rs:21` 的文件头），另两个 1448083638（删除流 30 深）与 1610875015（固定脚本 28 深，`/home/fy5090/.local/share/singlefs/crash-amplification/full-run-20260930-i.log:38` 那一项）是放量分项（流水线）的态数，不是登记了 `shard=across-machines` 的层 0 用例的域。结论不受影响：态数小于 2³² 时片长总小于块长（算的：3 743 376 012 态时片长 57120、一块 1.15 片），只在片长恰好是块长整数倍时才对得齐。入库装置上要钉它，不用重型：给 `layer0_progress` 加一条单测断言「片号按 `% n` 分台时，块里的态落在几台」即可（推的）。

### 三.6 N6 体量 389.8 GiB 与 598.9 GB（算的）

复算（同三.2 那一条 `python3 -c`）：2.616×10⁹ × 160 ÷ 2³⁰ = 389.8；3 743 376 012 × 160 ÷ 10⁹ = 598.94；2 563 727 926 − 268 435 552 + 1 448 083 638 = 3 743 376 012。三个输入的来路：2 563 727 926 是 9/30 那一趟总数（「CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926」（`/home/fy5090/.local/share/singlefs/crash-amplification/full-run-20260930-i.log:266`）），268 435 552 是同一份日志里多记录流那一项（第 60 行，那一项 `exhaustive=false`，C761），1 448 083 638 是 10/08 删除流 30 深（`records/2026-10-01-删除与并发事务.md:1292`）。160 字节每态是事实表按字段推的：事实表第 136 行自己写「按代码推，没量过」（`~/.local/share/singlefs/work/m3-merge-impl/gpu-split-survey/report.md`）；第一轮正推按比较真用到的字段另算 106–112 字节。两个体量数一个 28 深、一个 30 深下界，不能并排比；体量那一格的结论（闸在双机拷库）第三节第 4 条已经改了形态，这两个数今天只撑「一份放不放得下」，要等 M3 与一趟 30 深全量各项的态数。

### 三.7 N7 两轮报出的数，逐个来路

| 数 | 怎么来的 | 原始输入在哪 | 谁复跑或复算过 | 只在腿的模型上？入库装置上重做要什么 |
|---|---|---|---|---|
| 389.8 GiB | 算的 | 26.16 亿（CLAUDE.md 口径）× 160 B（事实表推的字段账） | 第一轮判决复跑模型相同；本轮复算相同 | 算术，无模型；要 M3（新布局落地后库字节 ÷ 态数） |
| 598.9 GB | 算的 | 9/30 日志总数、同日志多记录流一项、10/08 删除流一项 | 本轮复算相同 | 算术；下界（9/30 另有一条流不穷举，另五项 30 深没核）；要 M5 与一趟 30 深全量的各项态数 |
| 116–525 线程时 | 算的（参考数是量的旧数） | E161 第二轮单线程产物五格（2026-10-05 树、E142 两条流、45–4096 态的小格） | 第一轮判决复跑相同；本轮复算相同，并按 30 深下界得 166–751 | 只在第一轮攻方的算术里；要 M1（今天的树、录入段单独计时） |
| 3.7–5.4 天（第一轮正推） | 算的 | 旧记录「CPU 判器两台合计每秒约 8 千态」（`briefs/2026-09-28.md:88`），含池级 checker | 没人复跑（旧记录） | 是旧整判器速率的上界，不是录入段；要 M1 |
| 观察者约 17 天（第一轮正推） | 算的 | 一次轻型跑 9.2 秒 / 140 态（含建库、起线程的固定开销） | 没人复跑 | 外推一个固定开销主导的小样本；要 M2 |
| 62660 / 387189 | 量过（第一轮攻方的 naga 小程序，三钉值自证） | 副本三个内核 + 两个只摆调用点的探针入口 | 第一轮判决复跑相同；本轮在 E5、E7、E9 上逐字节相同 | 只在腿的模型上；下界；要 M4（内核写出后在入库测试里调 `kernel_inlined_size`） |
| 40–43% 块换主 | 量过（第二轮 Python；本轮用原函数） | `runs/` 下三趟日志的权重 | 第二轮判决复跑相同；本轮用原 Rust 函数，流水线五组逐个相同 | 只在腿的模型上；第 4 条改了形态之后不用再做 |
| 71–136 步 | 量过（第二轮攻方，录制口径） | BROAD 比重、两块 4 GiB、36 个种子 | 第二轮判决复跑相同；本轮录制口径逐个相同，但按枚举写表口径是 64–140 | 只在腿的模型上；口径要换（三.1）；入库要一条 checker 档用例（重型）对快档生成器的历史算 `enumerated_writes` 长度 |
| 层 0 片长 < 块长 | 读代码 + 算的 | `crash.rs:2537` 常量与片长函数 | 第二轮判决复算相同；本轮读代码相同 | 不需要装置；可钉一条 `layer0_progress` 单测 |
| 60–100 步（第二轮辩方） | 算的 | 主 agent 探针日志 24 段 24 步的写数，线性外推 | 没人复跑 | 线性外推，被 71–136 与本轮 64–140 取代 |
| 16.3 秒（第二轮辩方，历史族重录上界） | 量的（主 agent 探针日志 `rh-probe/probe.log` 第 55 行） | 24 段崩溃注入含判定 | 本轮只核了那一行在 | 主 agent 自己的探针，不是入库装置 |
| 514G / 1.8T 盘余 | 现查 | `df` | 两腿各查到两档 | 会变，不是数 |

## 没打中的形状

- 第 2 条「键按内容取」罩不罩全：找过历史执行在内容之外还读什么。环境变量只有工作线程数（副本 `crash_injection.rs:187` 一带，判定与线程数无关）；`HistoryExecution` 三个字段（每步 checker、盘宽、空间准入）整个 `{:?}` 进键就全了；比重、种子、取历史与取崩溃态的判据都在设点文件里，H8 那一把罩着。剩一处推的：崩溃点按 `history.seed` 抽，「生成好的操作序列」若不含种子，两颗种子生成逐字相同的操作序列时会撞键；读故障那一组比重里会话关着时只有两种步（`CloseAndMountWritable` 92、`ColdStartRecover` 8），但带参数的步（发布、覆写）让整段相同的机会很小，我没造出实例。写法上补一句「键含种子」就没有这一格。
- 第 6 条己的检查「当结论用的那一份来自 GPU」与 E9 那几条流：GPU 判不了的状态今天照 H6 判红（「// GPU 判不了的状态不进库（照 H6 判红）；往下走的结论取 CPU 判器的」（`/home/fy5090/.local/share/singlefs/work/m3-merge-impl/e5/repo/crates/singlefs-checker-tier/src/crash_amplification.rs:9145`）），那一趟本来就红，己的检查再红一次不造出新的错判。不算打中。
- 第 4 条历史族的归属「按历史键归台」与给定状态表按指纹分块：只有历史的主人跑这段历史时，这段历史的表都在它那台录、也就都归它，与「归属在录入那一刻写进库」不冲突；要写一句「给定状态表的块跟它的历史走」，不算打中。
- 第一轮第二节「第二次崩溃 = {R1}」：`judge_a_second_crash_after_the_restart` 的文档写明挂载不进 RefFS 的操作序列、二次崩溃落在「都返回过之后」（`crates/singlefs-harness/src/reference_filesystem_translation.rs` 第 913 行那一段文档），造不出集合不止 {R1} 的历史。
- 第一轮第二节「读故障那一标」的另一样输入 `ReadFaultsUnderTheRefusedMount`：崩溃注入里是一个常量（副本 `crash_injection.rs:3083`），不用存。
- 取样范围：闭包探针两份设点文件、四份候选入口、三棵搬家树加一棵对调树；搬家只做了枚举那一块（784 行），`crash_injection.rs` 与流水线录入段没搬；历史写表 36 段 × 144 步、一组比重、一种盘宽；块换主五对权重；内核量法三份底子。崩溃状态 0 个，`closed_form_state_count` 累计 0 态（这一条腿与它前面的腿合起来，本轮原型跑的崩溃状态都是 0）。

## 这条腿自己的限度

- 没编 checker 档的测试、没跑任何 checker 档测试二进制，轻型也没跑。history-enumerated 链接 checker 档的库（默认特性），只调两个公开函数、不摆崩溃状态；cargo check 只检查编不编得过。
- T4、T5 与 U 表都是读代码或读报告推的，没造跑得起来的实例；T5 里「冷启动读回报错之后执行器拿什么池接着跑」没读到底。
- 一.3 的宽松按项多重集只在一次搬家（枚举那一块）与一处对调上量过；别的搬家（`crash_injection.rs`、`GivenStatesEngine`）可能还有别的假红形（宏、`#[path]`、内联 `mod tests` 里补的 `use`），没试。
- U 表按关键词数，可能漏掉判决里换了说法写过的那几条（推翻条件见二节末）。
- 我提的改法全部被攻过零轮：

| 改法 | 修哪一格 | 标记 |
|---|---|---|
| 第 3 条的检查只判叶子出发的那一把，H8 那一把在版本里另记、写明「`12`/`14` 含 CPU 判器、改判器这一族重录、按秒计」 | T1 | 推的（只量了今天两份设点文件的闭包含 `crash.rs`） |
| 簇 0 清单加 `crash_facts.rs`、拆成「枚举与抽事实」与「第 ② 段判法」两份；禁物表加「任何一种结论的入口文件」 | T2 | 前半量过（move2 树：改 `crash_facts.rs:28` 一行之后叶子闭包 46 份、无 `crash.rs`、无 WGSL）；后半推的 |
| 按项多重集切项时项内可见性一律去掉、`use … {…};` 整条丢，另加「每个被搬的项恰好一次」的计数 | T3 | 前两样量过（`outputs/cluster0-item-acceptance.out`：move2 same、swap differ）；计数那一样推的 |
| 版本在排计划时并进将要编进管线的源码摘要，派活时断言编的就是它；配 k-gpu-edit 那类会红样本 | T4 | 推的 |
| 第 7 条加「实现自己第一处红之后」一类：照判、单列、不登记新发现；冷启动读回报错时录入段照今天停 | T5 | 推的 |
| 第二轮判决第二节「随机历史封顶」那一行改成枚举写表口径 64–140 步 | N1 | 量过（`outputs/history-enumerated-144-steps.out`） |
| CPU 代价那一行改成 30 深下界 ≥ 166–751 线程时 | N2 | 算的 |

## 没做什么

- 没判辩方那几格（第 1、2 问）；没替主 agent 采纳；副本与我的模型上的数不算入库装置上的数。
- 没读禁读清单里的三处（同轮辩方腿的报告、模型目录与草稿目录）。
- 没量：第三截表（崩溃镜像上挂载、发布）的枚举写表长度；M1–M5；搬 `crash_injection.rs` 与流水线录入段之后的叶子闭包。
- 跑前看了负载：别的会话在跑 `cargo test --offline --release -p singlefs-checker-tier --lib --tests`（重型，不是我起的）、一条 harness 档 `cargo test` 与 clippy、check；没有性能测量在跑。内存包装第一次排队 4 分多钟（slice 已占 40 GiB），我停了自己那一条（`proc.py stop`，停前列过子进程、停后再列为空），改成包装外先 `cargo build`、再按 2G 上限跑编出来的二进制；那一次没起跑，不算结果。

## 要主 agent 跑的命令

没有必须由主 agent 跑才能交回的。要把三.1 的数做成入库装置上的数：在 checker 档里加一条只建表不判的用例（照 `outputs/history-enumerated-144-steps.out` 的取法，按快档生成器跑 144 步历史，逐步取 `HistoryAfterTheMakeFilesystem::of` 的前缀、算 `layer0_enumeration_tables(…).enumerated_writes.len()`），它属于 checker 档测试，是重型，归主 agent 照 `.claude/skills/crash-test/commit-stages.md` 的起法跑。

## 草稿目录里留着的

`~/.local/share/singlefs/work/m3-run-compare-split-r3-attack/`：编译目录（`target-closure-probe3`、`target-move`、`target-history-enumerated`、`target-owner-drift`、`ks-*/target-kernel-size`）、副本拷贝 `copies/e5`、搬家树 `trees/` 与 `r1-opus-model` 拷贝在交回前删掉；留日志、`.rc`、分批输出与 `model-sha256.txt`。
