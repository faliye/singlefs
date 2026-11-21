# 产品包测试的对拍键怎么算（第一轮）：攻方腿报告

日期 2026-11-19。攻击面：正文「判据格」第 2、3、6、7 格与「实现今天的样子」逐句核；本轮角度：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

没写模型：打中的几处都是读码坐实的（下面每处给 `文件:行` 与原文，主 agent 照「核对用的只读命令」一节现查）。区域分类拒因次序那一处要上量的话，探针怎么写在第三节末尾，没写成代码：子 agent 编不了，写了验不了。没建仓副本、没建编译目录，草稿目录里没有要交接的东西。

引文口径：主工作区的文件写仓内相对路径；拆包第一簇的快照（`~/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo`，下称「快照」）里的文件写绝对路径，cite-check 认不出、会列进「没判」。快照里 `crates/singlefs-background-gpu/src/` 下的 `checksum_kernels.rs`、`live_slot_kernels.rs`、`region_record_kernels.rs` 与主工作区逐字节相同（`cmp` 量的），这三份照主工作区路径引。

## 各格判定一览

| 格 | 攻方判定 | 打中的（名字） | 分不分辨候选 |
|---|---|---|---|
| 实现今天的样子 | 四句里两句对不上、一句越位 | 「385 条都过」不是合入后验证里过的、真卡目标没编；`crash_identity.rs:804` 被当成约束用；主工作区今天已是独立测试包加产品 `build.rs` 交特性，没写 | 不涉候选 |
| 第 2 格 共用代码 | 打中 | 甲撞门禁 ⑨；丙离不开产品 `build.rs` 交特性；门禁 ⑧ 的允许清单今天放行被用户否掉的那一形；门禁已按测试函数算键、正文的三层键是第二份 | 分辨（甲、丙各中各的；⑧ 那条不分辨） |
| 第 3 格 键做不做得全 | 打中 | 显卡驱动与卡型不进键（全候选同中）；甲收不到 `singlefs-core` 的闭包与 cfg 真值，造得出「代码改了而键没变」 | 驱动那条不分辨；甲那条分辨 |
| 第 6 格 参照不得是产品代码 | 打中 | 区域分类的拒因次序逼独立参照照抄核心层；活槽数「同一个落点两条」逼独立参照照抄「各算一次」；三段的「算法不同」在字面上做不到；未列的以产品为参照的断言；「换成」与 D26 已定的对拍相冲；两条引据越位 | 不分辨（各候选同中，病在判据） |
| 第 7 格 检测强度 | 打中 | 照抄来的语义错没有一条测试会红；对拍罩不到 CPU 裁决那一支；卡上对拍只取到 4096 字节；端到端两边都是产品 | 不分辨 |

## 一、「实现今天的样子」逐句核

**第一句（验证包的键在构建时算、编进二进制）**：对得上。`crates/singlefs-verification/build.rs` 第 19–20 行 `#[path = "src/judging_units_entries.rs"]` 编登记表；「跑时不读盘上的树，也不收仓根」（`crates/singlefs-verification/src/crash_identity.rs:804`）；「include_str!(concat!(env!("OUT_DIR"), "/judging_units_keys.tsv"));」（`crates/singlefs-verification/src/crash_identity.rs:629`）。

越位一处：调度记录把这一行当成约束用——「键只能在构建时算：运行时算要读活树」，括号里引的就是 `crash_identity.rs:804`（`records/2026-11-19-产品GPU包测试与E202装置归位.md` 第 160 行，行号用 grep 现取）。`:804` 是验证包那段代码自述它怎么做的，不是规则；D13（验证路线） 已定项 17 只要求键的组成做全、收不全拒算（`.claude/kb/decisions/13-验证路线.md` 第 437 行「收不全……的那一份拒算键、照跑，不静默少收」），不禁运行时算。候选甲不能拿 `:804` 判出局，要拿第三节的那几条判。

**第二句（拆包第一簇补丁）**：前半对得上，后半对不上。
- 对得上的：快照的产品包 `build.rs` 把测试目录里的登记表编进构建脚本：「#[path = "tests/common_pairing_side_units_entries/mod.rs"]」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/build.rs:26`），三张表在同一文件第 219–232 行逐张算键；`[build-dependencies]` 有单元图（同目录 `Cargo.toml:27`）。族名 `background-gpu-pairing`、经 `prune_server::reliable_client` 存取都对（快照 `tests/common_pairing_input_through_the_library/mod.rs` 第 1–2 行与第 69 行）。
- 对不上的：「这一版在合入后验证里 fmt、clippy、整包 385 条测试都过」。调度记录写的是合入后验证那一步「退 3（判不出 `Cargo.toml`、`build.rs` 落点）」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:157`），385 条是另交的一趟整包测试：「合计 passed=385 failed=0」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:157`）。那一趟不带特性：确认文件原话「所以产品包整包跑一遍（不带特性，16 份里挂 `tests-on-a-real-gpu` 的 5 份不编、留给提交时门禁 product-gpu-real-card-tests 格）」（`~/.local/share/singlefs/work/gpu-tests-placement-whole-package/confirm-singlefs-background-gpu.md` 第 12 行，仓外文件）。另有「5 个要 `tests-on-a-real-gpu` 特性的真卡目标要卡」（`records/2026-11-19-产品GPU包测试与E202装置归位.md:153`）。所以 385 条里没有一条真卡对拍，三张键表算出来的键在真卡对拍上一次没被读过；这一版「都过」不能当键那一段跑通过的证据。

**第三句（第一轮正推腿第 4.4 节）**：转述对得上（`research/prompts/gpu-tests-placement-design-r1-sonnet-output.md:210` 有「测试运行时没有构建时的 `CARGO_ENCODED_RUSTFLAGS`、`TARGET`、`$RUSTC -vV` 这些输入」）。但它论的是「随新判定库包、运行时读 `crates/**/*.rs`」那一形，不是本轮的甲（`include_str!` 编进测试二进制）；甲不读盘上的树，这一句挡不住甲。真挡甲的是门禁 ⑨ 与 cfg 真值（第三节）。另一处不全：同一个包的构建脚本用 `cargo:rustc-env` 交出的变量与 `OUT_DIR` 对本包的测试目标一样可见（快照的测试就靠 `include_str!(concat!(env!("OUT_DIR"), …))` 读键表，快照 `common_pairing_input_through_the_library/mod.rs` 第 57–66 行），所以「测试运行时拿不到」只在「产品 `build.rs` 不替测试交」这一前提下成立——而那正是用户否掉的那一种共用。

**第四句（测试目标没有自己的构建脚本、`build.rs` 是产品包的）**：字面对，漏了今天的样子。主工作区今天：
- 13 个对拍目标住独立的测试包 `crates/singlefs-background-gpu-tests`，它有自己的 `build.rs` 与 `[build-dependencies]` 单元图（`crates/singlefs-background-gpu-tests/Cargo.toml:22`）——候选丙今天就是这个形状，只是它还依赖验证包。
- 产品包今天的 `build.rs` 只做一件事：替那个测试包交特性。「本包的测试包照它算对拍里参照一侧的判法单元键。」（`crates/singlefs-background-gpu/Cargo.toml:4`），交出那一行是「"cargo::metadata=enabled_features={}",」（`crates/singlefs-background-gpu/build.rs:14`）。`cargo build -p singlefs-background-gpu` 今天就编、跑这段只为测试算键服务的代码。
- 单元图的单测把产品包的构建脚本编进来：「#[path = "../../singlefs-background-gpu/build.rs"]」（`crates/singlefs-unit-graph/src/build_features.rs:278`），放行表登了这一处（`.claude/gate.d/crate-cross-references-allowed.tsv` 第 4 行）。
- 门禁已经按测试函数给这些对拍算键：「每条测试函数按单元键复用」（`.claude/gate.d/crash-point-replay.sh:12`）。
- 今天的 CPU 替身对拍经 `#[path]` 用验证包测试里的假实现：「#[path = "../../singlefs-verification/tests/common_background_gpu_compute/mod.rs"]」（`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11`），滞后表登了它（`.claude/gate.d/crate-cross-references-lag.tsv` 第 3 行）。

## 二、第 2 格：每个候选里藏着的共用代码

### 打中：甲要把 `singlefs-core` 的源码编进产品测试，撞门禁 ⑨

甲的两侧闭包都走出产品包：GPU 一侧的产品函数直接调核心层，例如选段打分那一段比对之前核整段清单「cluster_segment_entry_keys_of_the_unit_area(unit_area).as_slice(),」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:508`），被调的「pub fn cluster_segment_entry_keys_of_the_unit_area(」在 `crates/singlefs-core/src/compaction_plan.rs:156`。甲要收全闭包，就得在产品包的测试里 `include_str!` 核心层的 `.rs`；门禁 ⑨ 对这一种判红：「落在另一个 crates/<包>/ 目录下、后缀是 .rs 或 .wgsl 的判红」（`.claude/gate.d/checker-independence-and-sync.sh:52`）。量的（`find … -name '*.rs' -o -name '*.wgsl' | wc -l`，主工作区）：`singlefs-core` 41 份、`singlefs-replay-prefix` 10 份、`singlefs-mount-runtime` 1 份、`singlefs-format` 1 份，合 53 份要逐份登放行表；不登就只能收到产品包边界为止，键缺核心层那一截（后果见第三节「甲收不到核心层」）。另一处推的：单元图今天只从盘上装工作区（快照 `build.rs:162`「let workspace = load_workspace(&repository_root, threads, &switches)」），甲要它新收「编进二进制的一组源码文本」，这是单元图要新写的入口。

四句：分辨候选（只中甲）；被判对象看得到（⑨ 是静态门禁）；满足第 2 格「每个候选撞不撞前提」里「产品构建会不会……」之外的另一分句——不是撞前提 1–4，是撞已有门禁 ⑨，正文第 2 格的字面没列门禁，判据少一列；正文没给跑前改法。

### 打中：乙按构造就撞前提 4

乙的「别的输入」只要不是按函数单元取的，就落在 D13（验证路线） 已定项 17 明令不进键的那一类：「整文件原文；文件级、包级、整表级的指纹或闭包」（`.claude/kb/decisions/13-验证路线.md:436`）。测试二进制摘要是包级与整表级的一种，2026-11-11 用户否过的就是它；按产品源文件取摘要是文件级的。乙要合前提 4 只能回到按函数单元取，那就不是乙了；乙若读成「不复用、每趟现判」，撞前提 4 的「都要按键复用进库」。这一格不藏共用，是直接出局，写在这里只为把出局的分句点清。

### 打中：丙离不开产品 `build.rs` 替测试交特性

快照里三份对拍目标整份挂特性：`checksum_batches_over_two_real_gpus_…rs` 第 13 行、`checksum_batches_over_three_real_gpus_…rs` 第 13 行、`mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs` 第 6 行都是 `#![cfg(feature = "tests-on-a-real-gpu")]`（快照 `crates/singlefs-background-gpu/tests/` 下，grep 现取）。这几个目标的 GPU 一侧入口在这层 cfg 里，按已定项 17 这个原子「真引到」、要带真值进键。丙的测试支撑包构建脚本要知道产品包这一趟开没开这个特性，cargo 只经 `links` 的元数据把它交给直接依赖产品包的构建脚本——也就是今天主工作区那一形：产品包留 `links` 与 `build.rs`，`build.rs` 只为测试交特性（第一节第四句，`crates/singlefs-background-gpu/build.rs:14`）。两条路：

- 留着它：产品构建（`cargo build -p singlefs-background-gpu`，不开特性）照样编、跑这段只为测试算键服务的构建脚本，撞前提 1「如果是测试包 就放入测试」；
- 拆掉它：支撑包取不到这个原子的真值，单元图拒算（照已定项 17 是「照跑、不复用」），这三个目标从此不复用——不是错复用，是少复用。

前提 3 让测试住产品包的 `tests/`，挂特性的 `[[test]] required-features` 与 `[features] tests-on-a-real-gpu` 就只能写在产品包的 `Cargo.toml` 里：产品清单里至少有一个只为测试存在的特性，这一处任何候选都绕不开（推的：cargo 的 `required-features` 只认本包的特性）。

另两处丙要一起改的（推的）：登记表若仍住产品包 `tests/`，支撑包构建脚本 `#[path]` 引它就是跨包引 `.rs`，⑨ 判红，登记表要搬进支撑包；单元图的单测今天把产品包构建脚本编进来（`crates/singlefs-unit-graph/src/build_features.rs:278`「#[path = "../../singlefs-background-gpu/build.rs"]」）——测试代码编进产品代码的字面形状，只要产品 `build.rs` 留着就留着。

四句：分辨候选（中丙；甲若要 cfg 真值同样要它，见第三节）；看得到；满足第 2 格「产品构建会不会编进或跑到任何测试代码」那一分句的「跑到」；正文没给跑前改法。

### 打中：门禁 ⑧ 今天放行被用户否掉的那一形（防再发要先改它）

⑧ 的允许共用写的是「函数单元图 singlefs-unit-graph 与它经 [dependencies] / [build-dependencies] 带进来的」（`.claude/gate.d/checker-independence-and-sync.sh:41`），判的是验证包与产品包两边闭包的交集，不分产品包是经 `[build-dependencies]` 还是 `[dev-dependencies]` 拉进单元图。快照那一形（产品包 `[build-dependencies]` 单元图，快照 `Cargo.toml:27`）在 ⑧ 上是绿的；用户 2026-11-19 否掉的正是它。第 5 格「防再发」若只加「产品包依赖表里出现测试支撑包判红」，单元图这一条照样漏：要另加一条「产品包的 `[dependencies]`、`[build-dependencies]` 闭包里不许有单元图与判定库客户端」。不分辨候选（是防再发的缺口）。

### 打中：门禁已按测试函数算键，正文要的是第二份库

产品 GPU 对拍那一格已经「每条测试函数按单元键复用」（`.claude/gate.d/crash-point-replay.sh:12`；同文件第 275 行起写明种类同验证包那两趟、键相同的经 `--exact --skip` 跳过）。正文要在测试里另算录入键与对拍结论键，复用的是同一批判定，已定项 17 把这种情形列为违反：「两份库重复判同一批，同样算违反」（`.claude/kb/decisions/13-验证路线.md:436`）。所以第 1 格的问题设定本身越了一步：「产品包测试的键怎么算」默认了测试里要算键。攻方提的丁（只在我的读码上推过、被攻过零轮）：测试里一个单元键都不算；复用只靠门禁按测试函数的那把键（算键在 `scripts/gate/` 的 Python 那一份，不进任何包）；录入段与判段的交接按输入内容摘要加「这一趟门禁」的标识存取，判段只读同一趟录的那一份。它要付的：门禁之外直接 `cargo test` 的那一趟不复用；录入段不再「命中就跳过」。它要防的：两台同时跑、参照代码不同时按输入摘要串读——所以交接的清单必须按趟分，不能只按输入摘要。

## 三、第 3 格：拿不全的键的组成会不会让判定被错复用

### 打中（不分辨候选）：显卡驱动与卡型不进键，真卡对拍的「一致」跨驱动、跨卡复用

快照里对拍结论的键只有三样：「对拍结论的键 = 读回输入的摘要 ‖ 参照一侧的判法单元指纹 ‖ GPU 一侧的判法单元指纹」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/common_pairing_input_through_the_library/mod.rs:14`）。`wgpu`、`naga` 的版本照单元图的依赖行进键（「从整份 `Cargo.lock` 收窄出这个闭包的依赖行：闭包成员点名的外部 crate 解析到的包与它们的传递依赖」，`crates/singlefs-unit-graph/src/lock_file.rs:227`）；显卡驱动（Vulkan 驱动里那一段着色器编译器）与卡型不进。

反例（推的，读码）：本机那张卡上判出 `agreement=both_sides_agree` 入库；之后驱动升级，或同一棵树派到第二台、第二台的卡型不同——输入摘要、两侧单元指纹三样都没变，判段打 `PAIRING_CONCLUSION_REUSED`、不开卡。门禁那一层按测试函数的键同样不变，整条测试被跳过。驱动的着色器编译器错了，只有某次改代码碰巧让键变了才会被看到。

这是前提 4 自己的射程问题：「显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键」（`.claude/kb/decisions/13-验证路线.md:444`）写在崩溃判器块的射程里，同一句前面给的理由是「GPU 判器是加速路径」（`.claude/kb/decisions/13-验证路线.md:444`）——它的准入靠「与定义那一组的对拍」。产品包的真卡对拍正是那种对拍本身，被测的就是这张卡、这份驱动上的 GPU 路径；把「驱动不进键」从加速路径搬到被测路径，等于把唯一会因驱动变化而重判的地方也关了。前提 4 不在本轮重议，这一条交主 agent 判要不要问用户。

四句：不分辨候选（甲乙丙丁全中）；被判的系统当时看得到驱动与卡型（`wgpu` 的适配器信息报驱动名与版本，推的、没量）；满足第 3 格「拿不全的算哪一项缺、后果是复用错还是只是少复用」里的「复用错」，缺的不是前提 4 列的哪一项，是前提 4 没列的一项；正文没给跑前改法。照 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」，这一格是「打中不分辨臂」，不拿它判候选，另记一笔。

### 打中（中甲）：甲收不到核心层，造得出「代码改了而键没变」，而且会盖掉门禁那一层的重判

甲若守门禁 ⑨、不编核心层源码（第二节），闭包收到产品包边界就停。反例（推的，读码）：

1. 改 `crates/singlefs-core/src/compaction_plan.rs:156` 的 `cluster_segment_entry_keys_of_the_unit_area`（产品 GPU 选段打分那一段拿它核整段清单，`crates/singlefs-background-gpu/src/live_slot_kernels.rs:508`）。
2. 甲在测试里算出的 GPU 一侧键不变（那个函数不在它收得到的源码里）；输入摘要不变（输入是记录表与单元区，生成器不调这个函数，推的）。
3. 门禁那一层的键（Python 一份，收全闭包）变了，门禁现判这条测试函数——测试一进去，测试里那一层库命中两边一致的旧结论，打 `PAIRING_CONCLUSION_REUSED`、不开卡、判绿，门禁照绿记库。

第 3 步是关键：测试里那一层键只要比门禁那一层窄，它就把门禁正确发起的重判吃掉。这对任何「测试里另算一把键」的候选都成立，只是甲最容易窄。

cfg 真值也收不全：区域扫描这一截带特性原子——「#[cfg(feature = "lookup-work-counters")]」（`crates/singlefs-core/src/region_scan.rs:59`），这个特性开不开取决于这一次 cargo 怎么并特性（核心层构建脚本文件头写明验证包的 dev 依赖会替它打开，`crates/singlefs-core/build.rs` 第 3–4 行）。甲在运行时看不到核心层这一次的特性；要看只能读核心层为测试交出来的常量（`crates/singlefs-core/src/transaction.rs:3527` 的 `ENABLED_FEATURES_REPORTED_BY_THE_BUILD_SCRIPT`）——又是拿产品代码替测试算键。

### 打中（不分辨，现有机制的口子）：命令行 `--config` 与 `.cargo/config.toml` 里的 profile 不进键

单元图的 profile 生效值只由三层定：「每个设置依次由内置默认、根清单从继承链的根到叶、`CARGO_PROFILE_<名>_<键>` 盖」（`crates/singlefs-unit-graph/src/build_toolchain.rs:168`）。`cargo test --release --config 'profile.release.overflow-checks=false'`，或往 `.cargo/config.toml` 加一节 `[profile.release]`，编出的产品代码溢出时从 panic 变成回绕，三层都没变、键不变（推的，读码；根清单今天写的是 `overflow-checks = true`，`Cargo.toml:26`）。这一处验证包、快照那一形、丙都中，甲更甚（运行时连 `CARGO_PROFILE_*` 也只看得到继承下来的那部分）。不分辨候选，记成现有键机制的一笔账，不在本轮判。

### 没打中的形状（第 3 格）

- WGSL 模块级声明漏收：试了改 `const`、`struct` 字段、`var<workgroup>`、绑定号——单元图把 `const`、`override`、`struct`、`alias`、`var<…>` 都切成单元（`crates/singlefs-unit-graph/src/wgsl.rs` 文件头第 1 行），没造出漏收。
- 传递依赖漏收：`naga` 是 `wgpu` 的传递依赖，单元图收传递依赖（上引 `lock_file.rs:227`），没造出。
- 丙的工具链：支撑包构建脚本拿得到 `$RUSTC -vV`、`TARGET`、`CARGO_ENCODED_RUSTFLAGS`，与验证包同一套，除上面 `--config` 那一处外没造出缺口。
- 核心层 CRC 运行时选路（`crates/singlefs-core/src/checksum.rs:3` 起：查到 SSE4.2 走硬件指令、查不到走 slicing-by-8）不进键：参照若仍取核心层，两台若一台有一台没有 SSE4.2，录入段在哪台录就存哪条路的值；两条路逐字节同值时无害，两条路有一条错时先录的那台赢。换成测试侧独立参照后这一形没了，不单列。

## 四、第 6 格：测试那一侧的「独立参照」是不是换个写法的同一份算法

### 打中：区域分类比到拒因那一层，独立参照只能照抄核心层的检查次序

产品两份核交回的分类细到解析器拒的是哪一种（`crates/singlefs-background-gpu/src/region_record_kernels.rs` 第 1216 行起 `unit_error_of_the_codes` 逐个译回 `UnitError` 的九种，结构字段那一种另带缺陷名），对拍是整个 `AllocationInRegion` 逐条相等。核心层解析单元头的次序是：长度、magic、版本、类标签、flags、头校验和——类标签那一判「if bytes[6] != expected_class {」（`crates/singlefs-core/src/unit.rs:1360`）在头校验和「return Err(UnitError::HeaderChecksumMismatch);」（`crates/singlefs-core/src/unit.rs:1370`）之前。区域扫描还拿拒因走分支：跨度 2 的单元先按码 1 解，拒因是「found: UNIT_CLASS_PACKED,」（`crates/singlefs-core/src/region_scan.rs:822`）才改按码 3 解——检查次序在产品里是承重的。

仓里已有一份照 kb 另写的单元头判法（池级 checker 的 `check_unit`），次序不同：magic「if &unit[..4] != UNIT_MAGIC {」（`crates/singlefs-checker/src/lib.rs:693`）→ flags → 按类标签取布局「let unit_class = unit[6];」（`crates/singlefs-checker/src/lib.rs:699`）→ 长度「if unit.len() != usize::try_from(expected_length_in_bytes).expect("单元长度") {」（`crates/singlefs-checker/src/lib.rs:708`）→ 头校验和（`:711`）→ 载荷 CRC → 版本。

反例输入就在快照的区域对拍里：「"类标签不对",」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:501`），跨度 2 的码 1 单元把第 6 字节改成「&[UNIT_CLASS_INDEX_NODE],」（同文件 `:506`），不重封头。读码推的两边结局：

| 判法 | 结局 | 依据 |
|---|---|---|
| 核心层（产品两份核照它） | `Inconsistent(HeaderUnreadable(RefusedByTheParser(WrongUnitClass { expected: 1, found: 2 })))` | 类标签先于头校验和判（`unit.rs:1360` 在 `:1370` 前） |
| 照池级 checker 次序另写的参照 | 拒因是「太短」：码 2 的布局要 16384 字节（「expected_length_in_bytes: NODE_BYTES,」，`crates/singlefs-checker/src/lib.rs:673`），这个单元 32768 字节 | `lib.rs:699` 取布局在 `:708` 判长度之前 |

一份真照 kb 另写的参照，在现有输入上就会红；要它绿，只能把核心层 `check_common_prefix` 的次序与码 3 回退原样搬过来——正是第 6 格自己说的「同一份算法换个写法不算独立」。三条出路各有丢的（推的）：比得粗（只比可搬、钉住、隔离、已死、不一致五类与可搬的 key），拒因错了的产品错不再红；把拒因优先级写进 kb 当规约，两边各照 kb 写，独立性改按来处判；或认下拒因那一层的参照就是产品次序，写明这一层不算独立。

四句：不分辨候选（病在判据）；被判的系统看得到（参照与产品对同一输入各自交回拒因）；满足第 6 格「与产品那一份『算法不同』怎么判（同一份算法换个写法不算独立）」那一分句；正文没给跑前改法。探针（没写成代码）：用 `singlefs_core::unit::build_data_unit` 造一个码 1 单元，第 6 字节改 2、不重封，分别交 `singlefs_core::unit::parse_data_unit` 与 `singlefs_checker::check_unit`，打两边的拒因；再对快照区域对拍里每一种坏单元各打一行，数两边拒因不同的有几种。

### 打中：活槽数「同一个落点两条」逼独立参照照抄「各算一次」

快照的活槽数对拍把这一种当输入：「同一个落点两条（各算一次）」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:10`）。核心层逐条记录逐槽加一（「for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {」，`crates/singlefs-core/src/compaction_plan.rs:140`），而它自己的说明写的是「未释放记录罩住的槽数」（`crates/singlefs-core/src/compaction_plan.rs:123`），读成槽的集合大小也通。三份现有写法（两份核与核心层）都按记录走；测试侧唯一没被占的另一类算法是「先在位图上标出被罩住的槽、再逐段数」，它在这条输入上数出跨度、产品数出两倍跨度——独立参照要绿，就得照抄产品的「各算一次」。这一格的语义 kb 怎么写的我没核（没读 D26 已定项 13 的选段打分条文），要先由 kb 定，参照才谈得上独立。

### 打中（内部矛盾）：三段里「算法不同」字面上做不到的两段

判据两处：「同一份算法换个写法不算独立」（正文第 6 格）；「运行时与 checker 必须用不同的算法」（`.claude/rules/fs-design.md:22`）。各段已被占的算法类：

| 段 | 已有的写法 | 测试侧还剩什么 |
|---|---|---|
| CRC-32C | GPU 查表一份、GPU 按字逐位移一份；核心层硬件指令三路交错与 slicing-by-8 两条路（`crates/singlefs-core/src/checksum.rs:3`）；池级 checker 按位一份（`.claude/kb/decisions/13-验证路线.md:167`「CRC-32C 另写了一份按位的」） | 非反射的 GF(2) 长除、无进位乘法折叠这类冷门写法；而且参数钉死之后五份算的是同一个函数（`.claude/kb/decisions/13-验证路线.md:165`），「算法不同」只能指实现路线 |
| 区域分类里的映射反查 | GPU 逐条走映射、GPU 在排好的索引上二分；核心层按身份建哈希索引（`crates/singlefs-core/src/region_scan.rs` 第 128–133 行 `CentralMappingIdentityIndex` 的两张 `HashMap`） | 线性、二分、哈希三类全被占；有序树查找与二分同类。加上上一条的拒因次序，这一段字面上做不到 |
| 每段活槽数 | GPU 逐条记录走、GPU 在排好的记录上查；核心层逐条记录逐槽加 | 位图一类还剩，但撞上一条的语义分叉 |

结论（推的）：第 6 格要的「算法不同」得改成按来处判——参照只照 kb 条文写、依赖闭包里没有产品包与核心层（可做成门禁）——再配一组「产品与两份核一起改成同一种错、对拍必须红」的必红世界来证它独立（照 D13（验证路线） 已定项 6「独立性要被验证，不许自称」那一形）。按算法类判，三段里两段无解。

### 打中（内部矛盾）：「换成独立参照」与 D26 已定的对拍相冲

D26（后台整理与放置回收） 已定项 13 写定「两份实现在测试里对拍」（`.claude/kb/decisions/26-后台整理与放置回收.md:261`），两份指 GPU 路径与 CPU 路径，CPU 路径就是 `cpu_reference_*` 背后的核心层。第 7 格的字面是「换成独立参照之后」；照字面换掉，这条已定的对拍就没了，而产品连续不确定时真正拿来裁决的就是那份 CPU 写法。不冲突的读法是加不是换：每段三方——两份核、产品 CPU 写法、测试侧独立参照——两两都比。正文没写哪一种，正推腿若写成「换」，撞 D26。

### 打中（越位）：两条引据的射程都不罩产品包测试

`.claude/rules/fs-design.md:22` 住在「记账是事务的副产品，不是事后的遍历」那一节，管的是统计量运行时增量维护、checker 遍历重算（`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历`「审计与被审计用同一段代码」）。D13（验证路线） 已定项 5 的射程是「**射程**：管 checker 与实现的 crate 边界」（`.claude/kb/decisions/13-验证路线.md:167`）。产品包 `tests/` 里的对拍参照两条都不直接罩；真正立这条的是用户 2026-11-19 的原话。不推翻第 6 格，影响两处：一，已定项 5 里「CRC32C 各写一份的前提是参数钉死」（`.claude/kb/decisions/13-验证路线.md:165`）跟着搬过来——多项式、初值这几个钉死的参数算格式类，测试侧参照照用不算共用；二，独立性的判法要另立（上一条），不能从这两条里读出来。

### 打中：正文点名之外，以产品代码为对错标准的断言

正文点名了三个 `cpu_reference_*` 与 CPU 替身那一条。下面是另外找到的（主工作区 `src/` 里的单测与快照里搬过去的目标；逐条 grep 现取行号；「产品对产品」指两边都是产品代码的差分，D26 要的那一种，「以产品为标准」指拿产品代码的输出当对错）：

| 位置 | 断言 | 两边各是什么 | 归哪类 |
|---|---|---|---|
| `crates/singlefs-background-gpu/src/region_record_kernels.rs:2010` | `the_cpu_reference_classifies_each_record_as_the_whole_region_scan_does` | 产品的 `cpu_reference_classification` 对核心层整段 `scan_region` | 产品对产品 |
| `crates/singlefs-background-gpu/src/live_slot_kernels.rs:719`、`:911`、`:930` | 两份核不一致时交 CPU 判、切片之后逐段比 | 参照是 `cpu_reference_live_slots` | 以产品为标准 |
| `crates/singlefs-background-gpu/src/checksum_kernels.rs:674` | `units_cut_into_pieces_under_a_small_memory_limit_match_the_cpu_reference_on_every_unit` | 两份「核」都是 `cpu_reference_outcome`，参照也是它；真在测的只有两种合法 | 以产品为标准 |
| `crates/singlefs-background-gpu/src/background_gpu.rs:4931` 起 `cpu_reference_outcomes_of`，全文件 10 处调用（`grep -c` 量的） | 卡层的各条用例拿它当两份核的输出 | 被测的是卡层的分批、出错映射；核输出由产品代码造 | 产品当输入，不当标准 |
| `/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:64` | 期望值「一半对、一半错」 | 「对」的那一半由核心层 CRC 算 | 产品造输入：核心层 CRC 若错，「对」的那一半全变成「错」，两份核与任何参照照样一致、用例照绿，只是判定「对得上」那一支再没被走到 |
| 快照 `compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs`、`mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs` | GPU 模式整理的结局、写块序列与 CPU 模式逐项相同 | 两边都是整个产品，只差三段核 | 产品对产品 |
| 快照 CPU 替身对拍：产品一侧「let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);」（快照 `tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:222`）；另一边假实现「let computed_crc32c = crc32_castagnoli(unit.unit_bytes);」（快照 `tests/common_cpu_backed_background_gpu_compute/mod.rs:100`） | 三段逐轮逐条相同 | 两边落在核心层同三个函数上；另一边只多一份独立写的连续不确定计数 | 产品对产品——回答正文的问句：是，另一边也落在产品代码上 |
| 主工作区同名目标：另一边经 `#[path]` 引验证包测试里的假实现（`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11`），那一份同样调核心层三个函数 | 同上 | 同上 | 同上 |

反面一例（不以产品为标准）：`crates/singlefs-background-gpu/tests/gpu_gc_formal_contracts_through_the_host.rs` 的历史模型只记「每个条目有没有一次没解决的分歧」，自己推期望，不调产品的比对——这一形可以当测试侧参照的写法样板。

## 五、第 7 格：换成独立参照之后，哪类产品错仍然一条测试都不红

### 打中：照抄来的语义错，没有一条测试会红

上一节两处（拒因次序、同一个落点两条）独立参照为了在现有输入上绿，只能照抄产品的语义。反例（推的）：若 kb 的本意是「罩住的槽的集合大小」，核心层与两份核一起按「各算一次」数——三份互相一致、照抄来的参照也一致、GPU 模式与 CPU 模式的端到端也一致（两边都是产品）——唯一的后果是选段打分把有重叠记录的段打高、少选它，一条测试都不红。拒因次序同理：三份都把「类标签不对且头校验和也坏」报成类标签不符，若规约要的是先判头校验和，没有测试会红。能抓这一类的只有不看产品、只看规约的判法：语义先写进 kb，参照照 kb 写；或者在整理之后的盘上用池级 checker 与模型判「整理不改现行内容」（D13（验证路线） 已定项 18 的性质），后者抓得到「把活单元判成已死而丢了」这种有盘上后果的，抓不到只改选段次序的。

### 打中：CPU 裁决那一支在真卡上走不到，在替身上比的还是核心层

真卡对拍都在健康的卡上跑，两份核逐条一致，`reconcile_unless_the_batch_failed` 交 CPU 判的那一支一次都不走（快照端到端那一条还断言它是 0）。走到它的只有 CPU 替身那一条，而那一条的另一边也是核心层（上表）。产品错若落在「连续两轮不一致之后 CPU 怎么判」——例如 `region_record_kernels.rs` 里交 CPU 判时按身份的索引建错（只在头一次要它时建）——换成独立参照之后，只要独立参照只接在真卡对拍上、没接进替身那一条，照样不红。补法（推的、零轮）：替身那一条的另一边换成测试侧独立参照，注入不一致的条目覆盖三段各至少一条；代价是替身那一份假实现改写，三段各一个独立参照的调用点。

### 打中：卡上的 CRC 对拍只取到 4096 字节，产品的单元是 16 KiB 与 32 KiB

快照里长度只有「const UNIT_LENGTHS_UNDER_TEST: [usize; 7] = [0, 1, 7, 8, 9, 4095, 4096];」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:50`），模 4 只取到 0、1、3，没有余 2 的长度。产品交给校验和那一段的是整单元字节，码 2 是 16384、码 1 与码 3 是 32768（`crates/singlefs-core/src/region_scan.rs` 第 783、806 行按跨度读的两种长度）。端到端那一条只在健康的盘上断言「健康的盘上整单元 CRC 都对得上」（`/home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:284`），抓得到「好单元判成坏」，抓不到「坏单元判成好」；而校验和判坏在产品里只加一个计数（「gpu_work.units_whose_checksum_mismatched_on_the_gpu += 1;」，`crates/singlefs-core/src/region_scan.rs:652`），不改分类。所以「两份核在 16 KiB、32 KiB 上把坏单元一起判成好」——例如同一份 naga 与驱动对长循环一起编错（推的，两份核源码不同、编译链相同）——换成独立参照之后，卡上对拍取不到这个长度，端到端看不到这个方向，不红。补法（推的、零轮）：卡上对拍的长度加 16384、32768 与余 2 的一档，每档各放一个改了一个字节的坏单元；代价是录入那一段多存约 50 KiB 输入，卡上多算几十条。

### 打中（不分辨）：驱动不进键让上面能抓到的那几类也只抓一次

第三节「显卡驱动与卡型不进键」：任何一条靠真卡对拍才抓得到的错，只要代码没变，换驱动、换卡之后都复用旧的「一致」，不重判。

### 没打中的形状（第 7 格）

- 两份核共用的主机侧打包与回读（`crates/singlefs-background-gpu/src/checksum_kernels.rs:116` 的 `packed_checksum_chunk`、`:169` 的 `decoded_kernel_outcomes`）：错了两份核一起错、对拍确认，但测试侧独立参照对原始单元字节算，在现有取样上就红；偏移大的那一档有「空闲显存不够」那一条的 512 MiB 一批罩着，没造出取样之外才出现的打包错。
- 选段打分的单元区几何：两块单元区（默认环长一块、起点在槽 2^32 − 128 一块，快照活槽数对拍文件头第 9 行）罩住了高 32 位进位与末尾凑不满一段，没造出漏的。
- 校验和判坏只计数这一条在验证包有变异行罩着：`crates/mutations.tsv` 第 5572 行把 `+= 1` 改成 `+= 0`，点名 `compaction_batches_on_the_gpu_path_through_the_background_gpu_compute` 里的用例（那一条走的是核心层假实现，不是真卡），所以我没把「计数不加」当打中。

## 六、攻方自己提的改法（只在我的读码上推过、被攻过零轮）

| 改法 | 修的是哪一格 | 量过 / 推的 |
|---|---|---|
| 门禁 ⑧ 收严：产品包 `[dependencies]`、`[build-dependencies]` 的闭包里不许有单元图与判定库客户端，只许在 `[dev-dependencies]` | 第 2 格「⑧ 今天放行被用户否掉的那一形」 | 推的，待主 agent 跑（造一个产品包 `[build-dependencies]` 拉单元图的样本，⑧ 应红） |
| 丁：测试里一个单元键都不算，复用只靠门禁按测试函数的那把键；录入与判段的交接按输入摘要加这一趟的标识 | 第 2 格「门禁已按测试函数算键」；第 3 格「测试里那一层盖掉门禁的重判」 | 推的 |
| 独立性按来处判（参照只照 kb 写、依赖闭包里没有产品包与核心层，做成门禁），配「产品与两份核一起改成同一种错、对拍必须红」的必红世界 | 第 6 格「算法不同字面上做不到」 | 推的 |
| 拒因优先级、同一个落点两条怎么数，先写进 kb 当规约 | 第 6 格两条「逼独立参照照抄」；第 7 格「照抄来的语义错」 | 推的 |
| 每段三方两两比（两份核、产品 CPU 写法、测试侧参照），加不换 | 第 6 格「换成与 D26 相冲」 | 推的 |
| CPU 替身那一条的另一边换成测试侧参照，注入不一致覆盖三段 | 第 7 格「CPU 裁决那一支」 | 推的 |
| 卡上 CRC 对拍加 16384、32768 与余 2 的长度，每档一个坏单元 | 第 7 格「只取到 4096 字节」 | 推的 |
| 驱动与卡型记成块的属性，产品真卡对拍复用时要求属性相同（碰前提 4，要问用户） | 第 3 格「驱动与卡型不进键」 | 推的 |

这张表里没有一格对驱动那一条与 `--config` 那一条以外的「不分辨」打中起作用：第 6、7 格的几条病在判据，换哪个候选都一样中。

## 要主 agent 跑的命令

无交服务的命令：没写模型、没建副本，打中的都是读码坐实的。核对用的只读命令在下一节。

## 核对用的只读命令（仓根起，主 agent 直接跑）

- `grep -nF 'if bytes[6] != expected_class {' crates/singlefs-core/src/unit.rs`（看 `:1360` 在头校验和 `:1370` 之前）
- `grep -nF 'let unit_class = unit[6];' crates/singlefs-checker/src/lib.rs`（`:699`，与 `:708` 的长度判、`:711` 的头校验和判对照）
- `grep -nF '"类标签不对",' /home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs`
- `grep -nF '同一个落点两条（各算一次）' /home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo/crates/singlefs-background-gpu/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs`
- `grep -nF '函数单元图 singlefs-unit-graph 与它经 [dependencies] / [build-dependencies] 带进来的' .claude/gate.d/checker-independence-and-sync.sh`
- `grep -nF '落在另一个 crates/<包>/ 目录下、后缀是 .rs 或 .wgsl 的判红' .claude/gate.d/checker-independence-and-sync.sh`
- `grep -nF '每条测试函数按单元键复用' .claude/gate.d/crash-point-replay.sh`
- `grep -nF '显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键' .claude/kb/decisions/13-验证路线.md`
- `grep -nF '所以产品包整包跑一遍' /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-whole-package/confirm-singlefs-background-gpu.md`
- `grep -nF '每个设置依次由内置默认、根清单从继承链的根到叶' crates/singlefs-unit-graph/src/build_toolchain.rs`

## 没打中的形状（汇总）

- 第 3 格：WGSL 模块级声明漏收、`naga` 传递依赖漏收、丙的工具链（第三节末尾）。
- 第 7 格：两份核共用的打包与回读、单元区几何、校验和判坏只计数（第五节末尾）。
- 第 2 格：试了「测试侧 CRC 参照用核心层的多项式常量算不算共用」——已定项 5 把钉死的参数归成格式类（`.claude/kb/decisions/13-验证路线.md:165`），不算打中；试了「丙的支撑包依赖产品包会不会成环」——产品包 dev 依赖支撑包、支撑包依赖产品包，cargo 收这种 dev 环（推的、没量），集成测试目标里是同一份产品库，没造出类型对不上。
- 取样范围：快照产品包 `tests/` 下 16 份目标文件与 7 份共用模块、主工作区产品包 `src/` 下有单测模块的 12 份（逐份 grep `cpu_reference_`、`crc32_castagnoli`、`live_slots_per_cluster_segment_by_the_records`、`scan_region`）、主工作区测试包 `tests/` 下的 CPU 替身那一条；没逐份读 `background_gpu.rs`（11062 行）的测试模块，只 grep 了以上四个名字。

## 这条腿自己的限度

- 所有打中都是读码推的，没有一条跑过：拒因次序那一条的两边结局、`--config` 不进键、驱动不进键的复用，主 agent 要坐实得照上面的探针或样本跑。
- 没核 D26（后台整理与放置回收） 已定项 13 里选段打分「活槽数」与区域扫描拒因的 kb 条文，第四节两条「逼照抄」的分量取决于 kb 写没写这两处语义。
- `wgpu` 的适配器信息报不报驱动版本、cargo 收不收 dev 依赖环、`required-features` 只认本包特性，三处是我印象里的，没在本项目里量。
- 只读了正文、背景材料点名的 D13 条款与上面引到的源码；正推腿这一轮的产出按禁读清单没读，可能与它重复。

## 没做什么

- 不判第 1、4、5 格；第 5 格只在「⑧ 放行被否的那一形」处顺带指出防再发的缺口。
- 不替主 agent 采纳；没往统一任务服务交任务；没写模型、没建仓副本与编译目录，草稿目录 `~/.local/share/singlefs/work/product-test-pairing-keys-design-r1-attack` 里只有认领时就在的 `.owner` 与 `.writer-lock`，没有要删的。
