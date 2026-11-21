# gpu-tests-placement-design-r1 攻方腿（Opus）

日期：2026-11-19。方案按 `crates/` 今天的实现来谈；读的是主工作区此刻的树（开工快照另由主 agent 倒推核）。

## 复跑与读法

- 这条腿没写模型，模型目录没建。读码用了两份草稿脚本（只读源文件、按 `use` 树与 `crate::<模块>` 引用、剥掉 `#[cfg(test)]` 条目、标出 `#[cfg(feature = "gpu")]` 条目，收模块级闭包），放在草稿目录，不入库：
  - `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack/tools/module_closure.py` sha256 `159f8964ab563b38c4fa41971d5142f6962c6bd43e76655681bcef0350c1e488`
  - `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack/tools/external_crates.py` sha256 `8c2eae743aecf9afe892b307ba3a8175afcefabd43e133b11b5a163f63bd410a`
  - 输出 `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack/closure.txt` sha256 `a1ffab6d3cc2ec6c07948c636222a5974c93d77fc10ab95e03689c7b76082af5`
- 脚本自己的一处核对：关掉 `gpu` 时闭包里恰好没有 `crates/singlefs-verification/src/lib.rs` 挂特性的那四个模块（`crash_judge_dispatch`、`crash_judge_gpu`、`crash_verify_gpu`、`gpu_unit_checks`，lib.rs 第 39–55 行），开着时四个都在：与编译器的 cfg 对得上。它是读文本的近似，不是编译器；下文每条结论另带直接 `grep` 到的 文件:行，结论不只靠脚本。脚本报的模块数与行数是「推的，待主 agent 复跑」（命令在「要主 agent 跑的命令」）。

## 各格判定一览

| 攻击面 | 格 | 判定 | 一句话 |
|---|---|---|---|
| 一 | F1 甲「不含判器的存取与算键包」 | 打中 | 照今天的代码拆不出来：`crash_library` 的开库结构体字段类型就是崩溃放量的类型，`crash_library` 与 `crash_amplification`、`verdict_store` 互相引用；闭包带出 CPU 参照判器、池级 checker，开 `gpu` 时还带出 GPU 判器的派活。要拆先改验证包的代码，候选甲没写这一步 |
| 一 | F1 甲下 ⑧ 还有没有意义 | 打中 | 候选甲的形状就是 ⑧ 自己的红样本 `checker-implementation-disjoint-gpu-shared-closure-red`；⑧ 只认包名，看不见新包里按特性编进来的 GPU 判器 |
| 一 | F1 乙 | 打中 | 「产品 GPU 包测试目录里自己写的录与读回共用模块」是用户撤掉过的形状；乙还要依赖 prune-server 与 unit-graph，⑧ 交集同样要放宽；产品测试的录入键今天取的是验证包 `recording` 组的键，乙算不出 |
| 二 | F2 甲（新装置包） | 打中 | 依赖产品 GPU 包加存取包、带 131 条内联测试与构建脚本：除了没有 `tests/`，就是要撤的测试包；也撞 `.claude/rules/verification.md`「定义与名字」把真设备装置二进制放进验证包 `src/bin/` 的那一格（转述） |
| 二 | F2 乙（`scripts/experiment-apparatus/`） | 打中 | 撞 implementation-first 第 4 条字面；E202 `#[path]` 进来的取卡模块按 `CARGO_MANIFEST_DIR/../../scripts/…` 找脚本，挪到 `scripts/experiment-apparatus/<装置目录>/` 后路径落空（推的） |
| 二 | F2 丙（验证包 `src/bin/`） | 打中 | 撞 ⑧ 的两样：依赖那一样（验证包依赖产品 GPU 包）与源码那一样（E202 源码引 `singlefs_background_gpu`） |
| 二 | 三个候选合起来 | 打中不分辨臂 | 三个各撞一条，病根是共用前提：E202 同时要产品 GPU 包、验证包的库与卡、和测试共用的两个模块 |
| 三 | F3「运行时与 checker 不同算法」 | 打中不分辨臂 | 替身三段都直接调核心层的运行时函数，产品一侧的 CPU 参照也是这三个函数；四个候选都一样，替身放哪不改变算法 |
| 三 | F3 甲、乙、丙、丁 | 甲没打中；乙依赖 F1 甲；丙要改 ⑧；丁是跨包 `#[path]` | 见第三节 |
| 四 | 第 5 格共用模块 | 打中 | E202 `#[path]` 引的两个模块在任何 F2 候选下都要跨包；做成库时，装对拍过库那一份的库依赖产品 GPU 包又依赖验证包，正是 ⑧ 的红样本 `gpu-bridge-red`；另有一份今天就抄成两份的函数 |
| 五 | 「实现今天的样子」 | 打中 6 处 | 「9 个测试目标」实为 8；「用到的只有」漏 13 个名字；「被 10 个测试目标共用」实为 11；第 542 行读的不是测试文件头；「不依赖验证包」一组里的一个自己就写了依赖；前提引 code-discipline 那一节没有「跨包 `#[path]`」这一条 |

## 攻击面一：F1 判定库存取与算键放哪

### 1.1 候选甲拆出来的闭包有多大（照 `use` 链实读）

产品 GPU 测试用到的 `crash_library` 名字（全表在第五节）全部经 `CrashLibrary` 开库：`pairing_input_through_the_pairing_library` 在 `crates/singlefs-verification/src/crash_library.rs:2331` 开 `CrashLibrary::open`，`pairing_conclusion_reused_or_judged` 收 `&CrashLibrary`（同文件 2595 行）。从 `CrashLibrary` 往下一条字段类型链（条目级，不靠脚本）：

1. `crates/singlefs-verification/src/crash_library.rs:1539`：「library: LibraryOpenInThisProcess,」
2. `crates/singlefs-verification/src/crash_library.rs:1308`：「open: Option<Arc<OpenLibrary>>,」
3. `crates/singlefs-verification/src/crash_library.rs:948`：「ValuesReadBackOnceInThisProcess<crate::crash_amplification::RecordedFlowAndItsPlan>,」（`OpenLibrary` 的 `flows_read_back` 字段）
4. `crates/singlefs-verification/src/crash_amplification.rs:6150`：「pub(crate) struct RecordedFlowAndItsPlan {」，字段 `plan: CrashPlan`（同文件 6152 行；`CrashPlan` 在 510 行，`RecordedFlowReadBack` 在 6051 行）。

也就是说开一份判定库的连接，类型上就要崩溃放量的崩溃点计划。`crash_library` 非测试代码另有几处直接用崩溃放量：

- `crates/singlefs-verification/src/crash_library.rs:2656`：「pub const STATES_PER_PAIRING_BLOCK: u64 = crate::crash_amplification::STATES_PER_BLOCK;」
- `crates/singlefs-verification/src/crash_library.rs:2699`：「share_of_this_run: crate::crash_amplification::HostShare,」（同文件 2700、2788、2798 行同类）
- `crates/singlefs-verification/src/crash_library.rs:2811`：「crate::crash_amplification::host_share_and_weights_from_the_environment()?;」
- 1589–1592 行 `recorded_flow_and_its_plan_read_back_once` 的签名。

反方向也引：`crates/singlefs-verification/src/crash_amplification.rs:166`：「use crate::crash_library::CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE;」（167–170 行另 `pub use` 了 `LibraryOpenInThisProcess`、`RunStage`）；`crates/singlefs-verification/src/verdict_store.rs:2900`：「_journal_directory: Option<crate::crash_library::ExclusiveTemporaryDirectory>,」，而 `crash_library` 第 53–61 行 `use` 了 `verdict_store`。三个模块两两成环。cargo 不许包之间成环，所以照今天的代码，`crash_library`、`verdict_store`、`crash_amplification` 要么一起进新包，要么先改验证包的代码把环切开；候选甲的写法（「把这几样连同它们的闭包拆出去」）没有改代码这一步。

挂 `gpu` 的那一半：`crash_library.rs` 第 50 行是 `#[cfg(feature = "gpu")]`，接着 `crates/singlefs-verification/src/crash_library.rs:51`：「use crate::crash_judge_dispatch::{PackedJudgeTables, PackedJudgeTablesReadBack};」；第 2123–2201 行与 2521–2525 行几段挂 `gpu` 的函数读写 GPU 判器的打包判表（2124 行的注释写明「GPU 判器只收读回的那一份」）。`crash_library` 进了新包而 `crash_judge_dispatch` 留在验证包，开 `gpu` 时就是新包依赖验证包：要么把 GPU 判器的派活一起搬进新包，要么把这几段挪回验证包（又是改代码）。

非测试代码里引的外部 crate（`external_crates.py` 列的，逐处 grep 核过）：

- `crates/singlefs-verification/src/verdict_store.rs:1372`：「singlefs_checker::image::IMPLEMENTED_INVARIANTS」（1383 行又一处）：存取那一层直接引池级 checker 包。
- `crates/singlefs-verification/src/crash.rs:13`：「use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};」：`verdict_store.rs:146` 引了 `crate::crash`。
- `crates/singlefs-verification/src/crash_identity.rs:18`：「use singlefs_unit_graph::entry_groups::{」
- `crates/singlefs-verification/src/crash_library.rs:68`：「use prune_server::server::{start_in_this_process, ServiceInThisProcess};」：一次性库在进程里起整份服务，带进服务那一半（验证包 `Cargo.toml` 写它连带 librocksdb-sys）。

模块级闭包（`module_closure.py`，根是背景点名的五份；行数是各文件整份，含文件里的测试模块；推的，待主 agent 复跑）：

| 闭包 | 模块数 | 行数 | 带出来的判器与崩溃放量 |
|---|---|---|---|
| 关 `gpu` | 23 / 43 | 92356 | `crash_amplification`（36247 行，崩溃放量本身）、`crash`（`judge_crash_image`，`crates/singlefs-verification/src/crash.rs:1228`）、`crash_judge_cpu_blocks`（`judge_a_piece_of_a_block_on_the_cpu`，第 44 行）、`crash_given_states`（`judge_the_given_states_on_the_cpu`，第 282 行）、`crash_facts`（`verify_states_from_facts`，第 1282 行）、`crash_judge_tables`、`crash_state_enumeration`、`gpu_memory_ledger`、`crash_point_replay_provenance` |
| 开 `gpu` | 27 / 43 | 106365 | 上面那些，加 GPU 判器 `crash_judge_gpu`、`crash_judge_dispatch`、`crash_verify_gpu`、`gpu_unit_checks` |

关 `gpu` 那一行点名的四个 CPU 函数，正是 `crash_identity` 里判法单元的定义所说的「CPU 参照判器的闭包」：`crates/singlefs-verification/src/crash_identity.rs:341`：「/// 崩溃镜像该判出什么的定义：CPU 参照判器的闭包（`crash::judge_crash_image`，连同结论的编码与 CPU 路线判一块的入口」。验证纪律的参考也把这几样归在崩溃放量下：`.claude/rules/verification-reference.md@定义与名字`「崩溃放量：崩溃点身份与复用键 crash_identity、先录后核的流水线 crash_amplification、判定存储 verdict_store」。

编译期那一半也带着判器：`crash_identity` 读回的是本包构建脚本写进 `OUT_DIR` 的判器键表（`compiled_judging_units_keys`，`crash_identity.rs` 636–643 行）。产品 GPU 测试取的录入键是这张表里 `recording` 那一组：`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:314`：「&judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())」；那一组的入口在 `crates/singlefs-verification/src/judging_units_entries.rs:122`：「group_name: "recording",」起，其中 `crates/singlefs-verification/src/judging_units_entries.rs:130`：「"singlefs_verification::crash_judge_dispatch::PackedJudgeTables::of",」、`crates/singlefs-verification/src/judging_units_entries.rs:132`：「"singlefs_verification::crash_amplification::RecordedFlow::to_bytes",」。候选甲下新包要自己算出这一组的键，它的构建脚本就要收 GPU 判器派活与崩溃放量那几个函数的闭包（只收指纹，不编它们的代码）。

结论：照今天的 `crates/`，候选甲「不含任何判器、不依赖 wgpu」不成立。只搬文件，带出来的是 CPU 参照判器、池级 checker 包与崩溃放量本身，开 `gpu` 时还有 GPU 判器。要成立得先改验证包，改法是推的、没写没编：把 `OpenLibrary` 的流缓存类型从 `crash_library` 拿出去（做成泛型，或把缓存挪进 `crash_amplification`）；`PairingBlocksShare` 与 `STATES_PER_PAIRING_BLOCK` 不再取崩溃放量的类型；挂 `gpu` 的打包判表读写留在验证包；`verdict_store` 里点名池级 checker 与崩溃判定向量的部分拆出去；`crash_identity` 拆成「键类型与读表」和「崩溃镜像身份（`memory_pool`，第 16 行）加判器键表」两份。这些都改完，新包仍然依赖 prune-server（服务那一半）与 unit-graph。

推翻条件：真去拆（在副本里把五份文件挪进一个新包）时，不改验证包任何一行就编得过，而且新包的依赖表里没有 `singlefs-checker`，这一条就不成立。

### 1.2 候选甲下 ⑧ 还有没有意义

- ⑧ 的闭包只看 `crates/*/Cargo.toml` 的包名（`.claude/gate.d/checker-independence-and-sync.sh:332`：「for manifest_path in sorted(glob.glob("crates/*/Cargo.toml")):」），外部 crate 是叶子。候选甲下两边的交集多出新包、`singlefs-checker`、`singlefs-unit-graph`、`prune-server`；新包带着 `gpu` 那几个可选依赖时，`pollster` 也在交集里。这几个都不在允许名单里（`.claude/gate.d/checker-independence-and-sync.sh:39`：「两边闭包的交集减去允许共用的必须为空。允许共用的：singlefs-format、singlefs-core 与它们经 [dependencies] / [build-dependencies]」），所以甲必须按包名把它们加进允许名单。
- 甲的形状就是 ⑧ 自己的红样本：同一份文件第 103–105 行的 `checker-implementation-disjoint-gpu-shared-closure-red` 写的是「两边互不依赖，都依赖本仓的 sample-gpu-kernels 与外部的 pollster」，判红。允许名单按包名放进新包之后，这个样本只因为名字不同才照旧判红。
- ⑧ 不看特性。根 `Cargo.toml` 第 5 行是 `resolver = "2"`。一次构建里同时选了验证包与产品 GPU 包的测试（整工作区这一类起法）时，新包只编一份，`gpu` 由验证包打开，GPU 判器的派活就编进产品 GPU 测试的二进制（推的，没编）。⑧ 的依赖那一样看不到它（名字只有新包），源码那一样也看不到：`.claude/gate.d/checker-independence-and-sync.sh:42`：「#      两边的源码（包目录下全部 .rs，target/ 除外）零处引对方：真名按标识符认，经 `package =` 改名引进来的别名照 ② 的认法（`别名::`）。」，新包里的 GPU 判器是新包自己的模块，产品测试的源码一处 `singlefs_verification` 都不用写。
- 所以要另判「新包不含判器」，而且这一判要看模块与特性，不能只看包名。照今天的代码搬过去，这一判是红的（1.1 节）。

打中之后的四句：① 分辨臂：只打甲，乙不搬代码。② 判别子 ⑧ 看不看得到：看不到，它只认包名，要新加一格。③ 判据字面：第 1 格「拆出来的闭包实际有多大、要不要把判器或崩溃放量一起拖出来」，两样都拖出来了；「⑧ 的理由在甲下还成不成立」：只有加了「新包不含判器」那一格才成立。④ 改法：这一轮正文没有跑前的改法条款。上面那套推的改代码改法能去掉 CPU 判器、崩溃放量与 GPU 判器那几格，去不掉 prune-server、unit-graph 要进允许名单那一格。

### 1.3 候选乙撞不撞前提第 3 条

- 乙的形状用户撤过一次：`records/2026-11-11-GPU两段与按需领卡核查.md:128`：「撤回：产品 GPU 包测试目录里自己写的录与读回共用模块」（同一行接着列了「`Cargo.toml` 的判定库客户端依赖」）。撤的理由里点名的是那一版的键（「测试可执行文件的摘要加用例名」）。乙把键改成与验证包逐字节相同，但「产品 GPU 包测试自己写的录与读回」这一层没变。主 agent 当时对前提第 2 条的读法见同文件 `records/2026-11-11-GPU两段与按需领卡核查.md:141`：「新开的对拍测试包直接调验证包的 `two_stages` 时键值出自同一份代码，自然相同」。乙走的正是那一行留给「代码不同的地方」的口子，所以乙站不站得住，要看前提第 2 条与第 3 条哪一条管存取代码。正文把两条都列成已定前提，没写哪条优先。这是前提之间的张力，不是乙一个候选的毛病，交主 agent 判。
- 乙也过不了 ⑧，不比甲省：产品 GPU 包要自己读写判定库，就要依赖 prune-server 的客户端；要算判法单元键，就要依赖 unit-graph（今天测试包是构建依赖，验证包两样都有）。两个都会进两边闭包的交集，都不在允许名单里（1.2 节第一条）。正文问「⑧ 的允许共用清单要加什么」只对甲问了，乙同样要加 prune-server 与 unit-graph。
- 乙算不出今天的录入键。录入键的录入函数单元一半是验证包判器键表里的 `recording` 组（`crates/singlefs-verification/src/crash_library.rs:2427`：「&judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())」，组的入口在 `judging_units_entries.rs` 第 122–133 行，全是验证包的函数）。乙自己算，只有两条路：把那 8 个入口抄进产品一侧的登记表（登记表抄成两份），或者换一组录入函数单元（录入键的算法就不是「原来那一套」了，同一条用例的键也跟今天的不同）。两条都撞前提第 3 条或「重复要生成，不许手抄」。
- 「键值与验证包逐字节对拍」本身要一个同时看得到两份实现的地方：要么一个同时依赖两边的包（正是这一轮要撤的测试包），要么两边各跑一趟、经那一份库比（一边写、另一边读回核）。正文没写走哪一条。走前一条是循环：乙要靠它要撤掉的东西来证明自己对。

打中之后的四句：① 分辨臂：录与读回这一层的重复只打乙；⑧ 交集要放宽那一条甲乙都打，不分辨。② 看不看得到：⑧ 看得到 prune-server 与 unit-graph 两个名字（会红），看不到「两份存取代码各写各的」。③ 判据字面：第 1 格「乙撞不撞前提第 3 条」：存取那一层撞「不要做第二套」，算键那一层撞第 2 条读法里「输出的键值要一样」。④ 改法：没有跑前条款。把录入键换成产品自己的录入函数单元，能去掉「抄登记表」那一格，换来的是「录入键算法两套」那一格。

## 攻击面二：F2 E202 的装置放哪

E202 要什么（读码）：产品 GPU 包与核心层的实现（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` 第 62–141 行一串 `use singlefs_background_gpu::…`、`use singlefs_core::…`）；验证包的库与卡（第 142–145 行，第 13354–13360 行内联测试里的 `CrashLibrary`、`DisposableLibrary`、`RunStage`、`PairingAgreement`、`DIGEST_BYTES`，第 14060 行 `TheOneLibraryKeptForThisProcess`）；两个测试共用模块，用 `#[path]` 引进来（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:157`：「#[path = "../../tests/common_discrete_gpus_of_this_host/mod.rs"]」、同文件 160 行引 `common_pairing_input_through_the_library`）；构建脚本按两张登记表算的键（两张表里点名 `bin:e202_gpu_cpu_gc_comparison` 的组共 2 个，`grep target:` 数的）；内联测试 131 条（`grep -c '#\[test\]'`），其中挂 `tests-on-a-real-gpu` 的真卡对拍过判定库。二进制用不到 dev 依赖：`crates/singlefs-background-gpu-tests/Cargo.toml:16`：「（bin 用不到 dev 依赖）」。所以不论 E202 放进哪个包，上面这些都得写进那个包的 `[dependencies]`。

### 2.1 甲：新开一个「要产品 GPU 包的实现驱动装置」包

- 是不是换了名字的测试包：拿今天测试包的说明对，`crates/singlefs-background-gpu-tests/Cargo.toml:6` 写的是「这个包同时依赖两边，两边仍互不在对方的依赖里」。甲只要还从验证包拿库与卡（F1 不选甲时只能这样），依赖就和测试包一样是两边。特性名 `tests-on-a-real-gpu`、构建脚本、两张登记表、两份 `#[path]` 引进来的模块也都一样。和测试包比只少一个 `tests/` 目录。内联测试本身不算记号：验证包的装置二进制里也有大批内联测试（`e185_gc_batch_wall_clock_split.rs` 240 条、`e158_root_choice_repair.rs` 166 条，`grep -c` 数的）。
- ⑧ 罩不到甲：⑧ 只判验证包与产品 GPU 包这两个包的闭包，第三个包同时依赖两边，它不判。今天的测试包就是这样过的。所以甲过 ⑧ 不说明甲干净。
- 撞 `.claude/rules/verification.md@定义与名字`：那一格给验证包装的东西是「单元测试、集成测试、它们用的脚手架库与实验、真设备装置二进制（src/bin/）」。表头写「四个包，全仓只用这四个名字称呼它们」，甲在这四个之外另开了一处装置的家。用户说的「装置就该在装置的地方」，按这份规则读，就是验证包 `src/bin/`，也就是丙。
- F1 选甲（存取包拆成了）时，甲的依赖变成产品 GPU 包加存取包，不再是两边。这时仍剩上一条：装置的家多了一处。

### 2.2 乙：`scripts/experiment-apparatus/` 下的一个包

- 撞 `.claude/rules/implementation-first.md@规矩` 第 4 条字面：「scripts/experiment-apparatus/ 下的实验装置与 research/prompts/ 下各腿自己写的模型只用来独立验证：不与实现共用代码」。E202 量的就是实现本身（挂载、整理、产品 GPU 核），它调的就是实现的代码。照抄的先例 `scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:3`：「只有它以路径依赖引 crates/ 的 singlefs-core、singlefs-format」，它拿 crates 的两个纯函数当参照，和 E202 的用法不同，而且它自己把自己写成唯一的一个。照它的接法开第二个，那一句先失效。
- 同一份文件第 4–5 行记过路径依赖的一个坑：变异脚本把 `research/` 拷进临时目录编译时，副本旁边没有 `crates/`，「整个包每一张变异表的基线都编不过」（`scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:5`）。E202 今天在 `crates/mutations.tsv` 里有 146 行点名它的源文件（`grep -c e202_gpu_cpu_gc_comparison`，从第 6414 行起），归门禁 crates-mutation-replay。搬到乙，这些行要么换去装置那一路的变异表、撞上这个坑，要么让 crates 那一路的变异驱动去编一个 `crates/` 之外的包（推的，两种都没核）。
- 路径落空（推的，没编没跑）：E202 用的取卡入口 `handed_over_gpus_or_report_not_run`（E202 第 148 行 `use` 它）经 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:801`：「this_host_address_by_the_configuration_check,」起多机配置判法，判法的路径是 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:542`：「let script = Path::new(env!("CARGO_MANIFEST_DIR"))」接 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:543`：「.join("../../scripts/crash/crash-point-replay-shard-configuration-check.sh");」。`env!` 在引这份模块的那个包里展开，包目录要正好在仓根下两层才对得上。乙的包目录是 `scripts/experiment-apparatus/<装置目录>/`，往上两层是 `scripts/`，拼出来是 `scripts/scripts/crash/…`，不存在：判法起不来，panic，E202 取不到卡。甲（包放在 `crates/<名>/`）与丙（`crates/singlefs-verification`）不撞这一条。
- 层数本身不撞：`.claude/rules/directory-layout.md@scripts/：反复要用的脚本`「最深是 scripts/<子目录>/<装置目录>/<文件>」，清单、构建脚本、二进制、两张登记表平铺在一个装置目录里放得下。但两份共用模块要么跨包 `#[path]` 回 `crates/` 下，要么抄一份进来（第四节）。

### 2.3 丙：验证包 `src/bin/`

- 撞 ⑧ 的依赖那一样：二进制用不到 dev 依赖，验证包的 `[dependencies]` 只能加产品 GPU 包；可选依赖也照样列在依赖表里，⑧ 照认（`.claude/gate.d/checker-independence-and-sync.sh:38`：「#   ⑧ 验证包 singlefs-verification 与产品 GPU 包 singlefs-background-gpu：两边的传递闭包（dev-dependencies 也算）里都没有对方；」）。
- 也撞源码那一样：E202 源码第 62–76 行引 `singlefs_background_gpu`，放进验证包目录就是验证包源码引对方（同文件第 42 行那一条）。⑧ 有一个现成的红样本是丙的形状：第 99–100 行的 `checker-implementation-disjoint-gpu-verification-depends-red`。
- 要让丙过，只能改 ⑧（比如不判 `src/bin/`）。可依赖是按包算的，不是按目标：产品 GPU 包一进验证包的依赖表，验证包的库（GPU 判器所在的那一层）就能 `use` 产品代码，挡它的只剩源码那一样，而且只挡没被豁免的目录。第 50 行的理由（GPU 判器与产品路径共用一段代码）就落在这个口子上。丙让 ⑧ 失去意义的程度比甲大。

### 2.4 三个候选合起来

甲撞 `verification.md`「定义与名字」（F1 不选甲时还是换名的测试包），乙撞 implementation-first 第 4 条与 `CARGO_MANIFEST_DIR` 那一条路径，丙撞 ⑧。每个候选都撞一条规则，这是「打中不分辨臂」的形态：病根在三个候选共用的前提里，就是 E202 一个二进制同时要产品 GPU 包、验证包的库与卡、和测试共用的两个模块。拿这几条判候选之前，先问这个共用前提能不能拆：E202 要的验证包那一部分只有 `crash_library` 两段、`gpu_card_configuration` 的领卡、`verdict_store::PairingAgreement`、`crash_identity::JudgingUnitsKey`、`sha256::DIGEST_BYTES`（第五节的名单）。其中 `gpu_card_configuration` 的非测试代码不引本包任何模块（脚本收的边是零条，第 1012 行以后的 `crate::` 引用都在 `#[cfg(test)]` 里），能干净拆出来；`crash_library` 要先做 1.1 节那套改代码。F1 能落成「改过代码的甲」，F2 的甲就只剩「装置多了一处家」这一条。

打中之后的四句（2.2 节路径那一条）：① 分辨臂：只打乙。② 系统看不看得到：编译看不到，`env!` 照样展开，到跑时才 panic；没有门禁核这条路径。③ 判据字面：第 2 格「撞不撞 implementation-first 第 4 条、directory-layout 的层数与不放结果」里没有这一类；它属于「装置读判定库那一段在各候选下从哪来」旁边那一类，正文没给格，记成新的一格。④ 改法：把这个模块做成 `crates/` 下的一个库（第四节），`env!` 就在那个库里展开，这一格就不中了。

## 攻击面三：F3 核心层特质的 CPU 替身放哪

### 3.1 「运行时与 checker 必须用不同的算法」这条判据本身

- 正文把它写成小节名，其实它是 `.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 那一节里的一句：「被禁止的是两件事：需要答案的那一刻还没有答案；审计与被审计用同一段代码。运行时与 checker 必须用不同的算法。」那一节的射程是记账：谁消费这个数，运行时决策路径能不能遍历，checker 必须遍历。拿它判 GPU 计算的 CPU 替身是外推。这一格判据可能写错了（`.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的「打中归错了判据」），交主 agent 判这一句管不管替身。
- 就算它管，也不分辨四个候选。替身三段都直接调核心层的运行时函数：`crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:1` 写的是「背后照核心层的 CPU 写法算」，`crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:24`：「use singlefs_core::checksum::crc32_castagnoli;」，第 25–32 行另引 `live_slots_per_cluster_segment_by_the_records`、`scan_region_over_the_identity_index`。产品 GPU 包的 CPU 参照也是这三个函数：`crates/singlefs-background-gpu/src/checksum_kernels.rs:214`：「let computed_crc32c = singlefs_core::checksum::crc32_castagnoli(unit_bytes);」、`crates/singlefs-background-gpu/src/region_record_kernels.rs:12`：「//! CPU 参照（[`cpu_reference_classification`]）就是核心层那一份」、`crates/singlefs-background-gpu/src/live_slot_kernels.rs:483`：「live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area)」。对拍目标 `background_gpu_compute_agrees_with_the_cpu_backed_test_double` 两边算的那一半是同一份核心层代码，比的是上面那一层：`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:3`：「比对、连续不确定计数、判定与出错的映射都走产品那一份代码」。这一层替身是自己另写的（`crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:5`：「连续不确定的计数照 D26（后台整理与放置回收） 已定项 13 第 8 条在这里数」）。替身放进甲乙丙丁哪一处都不改它的算法，所以是「打中不分辨臂」：共用前提是替身本来就照核心层的 CPU 写法算。

### 3.2 四个候选逐个

- 甲（核心层带一个只给测试用的特性导出替身）：没打中。替身只依赖核心层，搬进去不改变依赖方向；⑧ 本来就允许 `singlefs-core`，不用改。核心层已有两个只供测试的特性，由验证包的 dev 依赖打开：`crates/singlefs-verification/Cargo.toml` 第 55 行 `features = ["lookup-work-counters", "test-only-publish-thresholds"]`。代价是推的、没量：一，`resolver = "2"` 下同一次构建里核心层只编一份，选了验证包测试又选了产品二进制的整工作区构建，替身会编进产品二进制，这和那两个已有的特性是同一种；二，替身里的单元多了一个 cfg 原子，按 D13（验证路线） 已定项 17 进键，用它的 8 个验证包目标（含 2 个崩溃枚举用例）与产品那一个目标的键各变一次、要现判一次。
- 乙（并进 F1 甲那个包）：只在 F1 选甲时存在。⑧ 那一面跟着 F1 甲（1.2 节）。存取与算键的包里多了一份 `BackgroundGpuCompute` 实现，两件不相干的事放进一个包；没有更多打中。
- 丙（单开一个只依赖核心层的测试支撑包）：两边都依赖它，它就进了交集；它不是核心层经 `[dependencies]` 带进来的，不在允许名单里，⑧ 判红（还是 1.2 节那个红样本 `gpu-shared-closure-red` 的形状）。要过，就得按包名放宽 ⑧。
- 丁（今天的样子，各自 `#[path]` 引同一份）：落在 ⑧ 认不出的那一类里（`.claude/gate.d/checker-independence-and-sync.sh:46`：「`include_str!` 跨包目录引源码或着色器文件不认（只认 crate 名与别名）。」，上一行是 `#[path = …]`、`include!`）。正文第 5 格说跨包 `#[path]` 不许，依据写的是 code-discipline「注释、断言、常量、重复」，那一节只有「重复要生成，不许手抄。」（`.claude/singlefs-ai-sop/rules/code-discipline.md:152`），没写 `#[path]`；`grep -rn '跨包' .claude/rules/ .claude/singlefs-ai-sop/rules/` 与 `grep -rln '#\[path' …` 两条都是零命中。不许跨包 `#[path]` 的依据只有前提第 1 条（「干净的拆开」）与这一轮的正文，没有成文的规则。另有一处对不上没核：验证包 `Cargo.toml` 第 35 行写「从仓根算单元键时 unit-graph 看不到 `#[path]` 引进来的文件」，`crates/singlefs-unit-graph/src/workspace.rs:2` 写的是「按 `mod` 声明（连 `#[path]`）连出每个编译目标的模块树」。丁下产品那一个目标的键跟不跟得到替身，推不出。

## 攻击面四：第 5 格，共用模块拆完之后各住哪

五个模块的用户与依赖（`grep` 数的，`mod <名>;` 一行算一个用户）：

| 模块 | 行数 | 谁用 | 依赖 |
|---|---|---|---|
| `common_discrete_gpus_of_this_host` | 859 | 11 个测试目标、E202（`#[path]`）、`common_pairing…`、`common_checksum…` | 产品 GPU 包（mod.rs 第 28、33 行）、核心层 |
| `common_pairing_input_through_the_library` | 493 | 9 个测试目标、E202（`#[path]`）、`common_checksum…`、`common_history…` | 验证包（第 16–32 行）、`common_discrete…`（第 12 行）、本包构建脚本的 `OUT_DIR`（第 36、40 行） |
| `common_history_pool_through_the_library` | 158 | 3 个测试目标 | 验证包 `PairingJudged`、`common_pairing…`、`common_pool…` |
| `common_checksum_batches_over_the_handed_over_gpus` | 322 | 3 个测试目标 | 产品 GPU 包、核心层、验证包 `PairingJudged`、`common_discrete…`、`common_pairing…` |
| `common_pool_on_memory_devices` | 432 | 4 个测试目标 | 只有核心层 |

- 后三个只给测试用，跟着测试进产品 GPU 包的 `tests/`，同一个包里照今天的 `mod` 写法用，不跨包。前两个 E202 也用，而 E202 不论选哪个 F2 候选，都和测试不在一个包里（产品 GPU 包的 `src/bin/` 也不行：二进制要的验证包只能进产品包的 `[dependencies]`，⑧ 依赖那一样判红）。所以这两个要么跨包 `#[path]`，要么抄一份（两样都不许），要么做成库。
- 做成库：`common_pairing…` 依赖 `common_discrete…`，后者依赖产品 GPU 包，所以这个库依赖产品 GPU 包。它还要验证包的库。F1 不选甲时，这个库就依赖验证包；产品 GPU 包的测试 dev 依赖它，就成了「产品 GPU 包 → 中间包 → 验证包」，正是 ⑧ 的红样本：`.claude/gate.d/checker-independence-and-sync.sh:101`：「产品 GPU 包的 dev-dependencies 用别名 `judge` 引中间包 sample-gpu-judge-bridge」，下一行接着「中间包依赖验证包、转出它的 GPU 判器模块：⑧ 红」。只有 F1 选甲（而且是改过代码的甲）时，这个库才能改成依赖存取包、过 ⑧。那时它是「依赖产品 GPU 包与存取包、装测试共用模块、带构建脚本」的一个包，就是今天测试包除去 `tests/` 的样子（推的）。
- 这个库还连着构建脚本：`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:36`：「include_str!(concat!(env!("OUT_DIR"), "/reference_units_keys.tsv"));」。`OUT_DIR` 是编这个库的那个包的，它的构建脚本要替别的包里的目标（登记表里写的是 `test:checksum_kernels…`、`bin:e202_gpu_cpu_gc_comparison`）算键。unit-graph 能不能从一个包的构建脚本去收另一个包的目标的闭包，这一格归第 4 格，攻方没核，推不出。不改这一点的出路是把读 `OUT_DIR` 那两个常量留在各自的包里、当参数传进库（改代码，推的）。
- 第二套录制入库：F1 选甲时没有。F1 选乙时 `common_pairing…` 要改接产品一侧自己写的存取代码，就是第二套（1.3 节）。
- 今天已经抄成两份的一处（量的）：`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:206`：「pub fn pci_bus_identifier_without_the_domain(pci_bus_address: &str) -> String {」与 `crates/singlefs-verification/src/gpu_memory_ledger.rs:780`：「pub fn pci_bus_identifier_without_the_domain(text: &str) -> String {」，两个函数体的三行逐字相同（mod.rs 207–209 行，gpu_memory_ledger.rs 781–783 行）。抄的理由写在 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:204`：「本包测试不依赖 验证包，自己留一份」，今天这句不对（测试包 `Cargo.toml` 第 15 行直接依赖验证包）；搬进产品 GPU 包之后它会变成对的，两份却还留着。哪个 F1、F2 候选都不碰这一处，所以不分辨臂。F1 选只搬文件的甲时 `gpu_memory_ledger` 跟着崩溃放量进了存取包（`crash_amplification.rs` 第 89 行引它），产品测试可以改用那一份；选改过代码的甲时它留在验证包，这份抄的照旧在。

打中之后的四句（库的那一条）：① 分辨臂：分辨 F1 的甲与乙，乙下没有干净的落点。② ⑧ 看不看得到：中间包依赖验证包这个形状它看得到（会红）。③ 判据字面：第 5 格「有没有一处要跨包 `#[path]` 或抄成两份」：两个模块有，出路只剩做成库，做成库又撞 ⑧ 或要求 F1 甲。④ 改法：没有跑前条款。

## 攻击面五：正文「实现今天的样子」逐句核

| 正文的句子 | 核的命令 | 核到的 | 对不对 |
|---|---|---|---|
| `research/prompts/_gpu-tests-placement-design-r1-body.md:31`：「直接 `use singlefs_verification` 的有 9 个测试目标与 E202 的 bin」 | `grep -c singlefs_verification` 逐文件 | `tests/*.rs` 里大于 0 的只有 8 个：三个校验和目标、区域记录核、活槽核、真卡整理、挂载运行库块序列、对拍结论复用；另有三个共用模块（`common_pairing…`、`common_history…`、`common_checksum…`）直接用，它们不是测试目标 | 不对，是 8 |
| 同一行「用到的只有：」后面列的 11 个名字 | 从全包源码（去掉 `//` 注释）里抽 `singlefs_verification::<模块>::<名字或 {…}>`，花括号里的名字逐个收 | 共 24 个名字。漏掉的 13 个：`crash_identity` 的 `judging_units_key`、`judging_units_key_of_the_group_in_the_table_text`、`JudgingUnits`（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:17`：「judging_units_key, judging_units_key_of_the_group_in_the_table_text, JudgingUnits,」）；`crash_library` 的 `pairing_input_and_reference_through_the_library_under_the_recording_key`、`recording_key_of_the_pairing_input_and_the_reference_units`、`toolchain_of_this_process`、`CrashLibrary`、`CrashLibraryClass`、`ItemName`、`PairingConclusionSource`、`RecordingKeyOfThisRun`、`StageOutcome`（同文件第 21–27 行）；`gpu_card_configuration::CardLeaseChannelOfTheCaller`（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:30`：「cards_handed_over_or_leased_on_demand, CardLeaseChannelOfTheCaller,」）。漏掉的多半是花括号里多行列的名字，按「模块::名字」一行一行 grep 抓不到 | 不对。漏的里面有 `JudgingUnits::Recording`，它连着 1.1、1.3 节那一条：录入键取验证包判器键表里的 `recording` 组 |
| 同一行「它们与验证包其余模块…怎么缠，主 agent 没读，推不出」 | — | 第一节已答：关 `gpu` 时 23 个模块，带出崩溃放量与 CPU 参照判器 | 这一轮补上了 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:32`：「不依赖验证包的目标：」 | 逐个看 `mod` 与 `use` | 列的五个里，`mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs` 自己 `mod` 了 `common_pairing…` 与 `common_history…`（第 9–10 行），两个都用验证包。正文括注只说了前一个。另外四个确实不引验证包 | 自相矛盾：它不该列在「不依赖」那一组；括注漏了第二个模块 |
| 同一行「验证包另有 8 个测试目标用它」 | `grep -rln 'mod common_background_gpu_compute' crates/singlefs-verification/tests` | 8 个 | 对 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:33`：「被 10 个测试目标与 E202 的 bin 共用，第 542 行按 `CARGO_MANIFEST_DIR` 读本包测试的文件头」 | `grep -rn 'mod common_discrete_gpus_of_this_host'`；读 542 行 | 11 个测试目标（三个校验和目标、区域记录核、活槽核、真卡整理、挂载运行库块序列、维护线程、对拍结论复用、两个 `real_gpu_tests_*`），另有两个共用模块与 E202。第 542–543 行拼的是 `../../scripts/crash/crash-point-replay-shard-configuration-check.sh` 的路径（起多机配置判法），不是读测试文件头；读测试文件头的是 `gpu_card_count_declared_in_the_source_header`（第 425 行），源文本由调用方给（`real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs` 第 306、321 行 `include_str!`） | 两处不对。第二处要紧：这条路径换个包的深度就落空（2.2 节） |
| 同一行「`common_pool_on_memory_devices/mod.rs` 被 4 个共用、只依赖核心层」 | `grep` | 4 个；`use` 只有 `singlefs_core` 与 `std` | 对 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:34` 判定库服务的 Rust 客户端在 `prune-server/src/lib.rs` | `ls prune-server/src` | 客户端在 `client.rs`、`reliable_client.rs`，`lib.rs` 是包根。要紧的是正文没写的那一半：`crash_library` 还引了服务本身（`crates/singlefs-verification/src/crash_library.rs:68`：「use prune_server::server::{start_in_this_process, ServiceInThisProcess};」），拆出来的包带着的不只是客户端 | 大意对，漏了服务那一半 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:35`：「`scripts/experiment-apparatus/` 下有一个经路径依赖引 `crates/` 的包 `e176_crates_oracle`」 | `grep -n crates scripts/experiment-apparatus/Cargo.toml` | 主装置包也有一条：`scripts/experiment-apparatus/Cargo.toml:254`：「singlefs-device-library = { path = "../../crates/singlefs-device-library" }」（E174、E178 读写判定库的口子）。e176 那份清单第 3 行的「只有它」同样过时了 | 不对：至少两个包 |
| 同一行「`Cargo.toml` 26 条 `[[bin]]`」 | `grep -c '^\[\[bin\]\]'` | 26 | 对 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:36` 产品 GPU 包 `tests/` 3 个目标、`[dev-dependencies]` 只有 noop 后端的 wgpu | `ls`、读清单 | 3 个；dev 依赖只有 `crates/singlefs-background-gpu/Cargo.toml:28`：「wgpu = { version = "=30.0.1", default-features = false, features = ["noop"] }」 | 对。正文没写的一点：产品包的 `[dependencies]` 已经有 `singlefs-mount-runtime`（第 15 行），测试包的 dev 依赖 `singlefs-mount-runtime` 搬过来不新增包 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:37` 的两处行号（`mod.rs:42`、`task_server_lib.py:2029`） | `sed -n` | 第 42 行是 `GPU_CARDS_BY_PRIORITY` 常量；第 2029 行是 `CARD_CHOICE_VARIABLES` | 对 |
| `research/prompts/_gpu-tests-placement-design-r1-body.md:47`：「两样都不许，见前提与」code-discipline「注释、断言、常量、重复」那一节 | 读那一节，全规则目录 grep | 那一节只禁手抄，没写跨包 `#[path]`（3.2 节丁那一条） | 引的出处只罩一半 |

## 没打中的形状

- F1 甲「不依赖 wgpu」那一半：关 `gpu` 时闭包的非测试代码里一处 `wgpu::` 都没有（`external_crates.py` 列 23 个模块，`wgpu`、`pollster` 只在 `crash_amplification` 挂 `gpu` 的条目里出现）。这一半只要新包不带 `gpu` 特性就成立；带不带要看 1.1 节挂 `gpu` 那几段怎么处置。
- `gpu_card_configuration` 与 `sha256` 能不能干净拆：能。前者的非测试代码不引本包任何模块，后者只引 `hexadecimal`（26 行）。
- F3 甲：试了 ⑧、依赖方向、fs-design 那一句，没打中，只有 3.2 节那两笔推的代价。
- F2 甲过不过 ⑧：过（⑧ 不判第三个包），所以不算打中，只记成「⑧ 过不说明干净」。
- 跨包的 `include!` 与 `include_str!`：测试包今天的 `include_str!` 都引本包的文件（`reference_units_entries_name_functions_that_exist.rs` 第 147–155 行、`real_gpu_tests_open_only_…` 第 306、321 行），跟着测试一起搬就不跨包；没找到藏着的跨包引用。E202 的两处 `#[path]` 是唯一会变成跨包的（第四节）。
- `common_pool_on_memory_devices`：只给测试用、只依赖核心层，哪个候选下都不跨包。
- 取样范围：模块级闭包扫了验证包 `src/` 下全部 43 个模块文件（`src/bin/` 不在内）；条目级只追了 `CrashLibrary` 往下那一条字段类型链，与 1.1 节点名的十几处引用。「用到的名字」扫了测试包全部 `.rs`（`tests/` 13 个目标、5 个共用模块、`src/` 下 4 份）。

## 这条腿自己的限度

- 一次抽样。上面「没打中」的各条一次不算数（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。
- 闭包是按文本扫的模块级近似：一个模块里只要有一处非测试引用就整份算进去，所以行数是上界，条目级的闭包会小。这一点不影响「拆不开」的结论，那条结论靠的是 1.1 节条目级那条字段类型链与成环，不靠行数。
- 没编、没跑：1.1 节那套改代码的改法、1.2 节特性合并会把 GPU 判器编进产品测试、2.2 节 `CARGO_MANIFEST_DIR` 拼出的路径会落空、3.2 节甲的两笔代价，全是推的。改法只是我自己推的，没在任何模型上量过，被攻过零轮。
- 没读 prune-server 自己的依赖树，没核 unit-graph 能不能从一个包的构建脚本去收另一个包里目标的闭包（第 4 格的事）。
- 按禁读清单，没读正推腿这一轮的产出。

## 要主 agent 跑的命令

1. 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack -- python3 /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack/tools/module_closure.py /home/fy5090/code/singlefs/crates/singlefs-verification/src crash_library,verdict_store,crash_identity,gpu_card_configuration,sha256 edges`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack`（草稿目录，即 `--cwd`）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`；`task-server/task log <号> --all` 里两行 `total_modules=23 total_lines=92356 of_all=43`（关 `gpu`）与 `total_modules=27 total_lines=106365 of_all=43`（开 `gpu`）；`## edges` 下面 `crash_library -> crash_amplification	src/crash_library.rs:948` 一行。它是只读文本的脚本，不编不跑项目代码。1.1 节的结论另有逐处 `grep` 到的 文件:行，判决可以不依赖这条。
2. 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack -- python3 /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r1-attack/tools/external_crates.py /home/fy5090/code/singlefs/crates/singlefs-verification/src crash_library,verdict_store,crash_identity,crash,crash_amplification`
   - 在哪起：同上。
   - 看哪几行：`exit=0`；日志里有 `verdict_store	singlefs_checker	always	count=2	first=src/verdict_store.rs:1372` 与 `crash_identity	singlefs_unit_graph	always`，`crash_amplification	wgpu` 那一行第三列是 `gpu`。

## 没做什么

- 不判正推那几格（第 4、6、7 格没攻），不替主 agent 采纳。
- 没写模型，模型目录没建，所以没有 `rerun.sh`；两份草稿脚本只是读码的辅助。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 没建仓副本与编译目录，草稿目录里只有两份脚本与三份文本输出，没什么要删的。
