# 产品包测试的对拍键怎么算（产品与测试不共用代码）（第一轮）：正推腿报告

腿：three-way-forward（Sonnet）。日期 2026-11-19。方案按 `crates/` 今天的实现来谈；拆包第一簇补丁的快照在 `~/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo`（下称「快照」，只读），补丁在 `~/.local/share/singlefs/work/gpu-tests-placement-product-tests/patch` 与 `~/.local/share/singlefs/work/gpu-tests-placement-product-tests-repair/patch-with-the-tests`。不读攻方腿这一轮的任何产出（禁读清单照守）。

## 0. 复跑命令、文件摘要、读法

模型（只用标准库、不读仓、不起进程，三段：CRC-32C 两种写法对已知答案、活槽数三种读法、长度取样格）与探针（一个没有构建脚本的测试目标，看编译时与跑起来时各读得到哪几样环境）**都没跑**，数全部「待主 agent 跑」；我只做了 `python3 -m py_compile` 与 `bash -n`。

| 文件 | sha256 |
|---|---|
| `research/prompts/product-test-pairing-keys-design-r1-sonnet-model/model/independent_reference_model.py` | `26dada674319948aaf47a696f749df45a00daa14cefd6773de3d3ea07d3fa6e7` |
| `research/prompts/product-test-pairing-keys-design-r1-sonnet-model/tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`） | `48c12779f0f2098142802cbe63948b5b2a556cfa76dfc85391538036b62dd707` |
| `research/prompts/product-test-pairing-keys-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `~/.local/share/singlefs/work/product-test-pairing-keys-design-r1-forward/probe/Cargo.toml` | `5b39eeb0cb262f388d8bdd49b49fedda4d8344f1f9cc06f97a31b456ff633142` |
| `~/.local/share/singlefs/work/product-test-pairing-keys-design-r1-forward/probe/tests/probe.rs` | `5083ebfd054ba07ddb637b090d26c2c12e06969f28c9720e35466c62906fa22b` |

模型复跑（整条交服务，运行目录名以 `-run` 收尾）与探针命令都在末尾「要主 agent 跑的命令」一节；探针与模型的内存与时间是推的：模型纯标准库、几千次整数运算，预计不到 1 秒、不到 50 MiB；探针是一个无依赖的小包，首次编译预计十几秒（推的）。

读法：「量的」是我这一趟 `grep` / `wc` / 读文件现查的；「推的」是从机制推出来、没有跑；快照里的行号写成「快照 N」，不写成 `文件:行`（快照里的同名文件与主工作区里测试包的同名文件行号不同，引文核对会认错文件）。主工作区里的 13 个目标今天都住 `crates/singlefs-background-gpu-tests/tests/`（下称「测试包」），快照里住 `crates/singlefs-background-gpu/tests/`。

## 0.1 开工先核的事实（量的）

1. 产品 GPU 包今天的构建脚本只交特性：`crates/singlefs-background-gpu/build.rs:12` 的 `fn main() {`，23 行（`wc -l`），清单里 `crates/singlefs-background-gpu/Cargo.toml:5` 有 `links = "singlefs-background-gpu"`。产品包源码里没有按特性编的代码：`grep -rn 'feature\s*=' crates/singlefs-background-gpu/src crates/singlefs-background-gpu/build.rs` 只命中 `crates/singlefs-background-gpu/build.rs:10` 那一行特性表；`grep -rn 'cfg(' src` 去掉 `cfg(test)` 后 0 命中。清单里注释自己写了「本包的源码里没有按这个特性编的代码」（`crates/singlefs-background-gpu/Cargo.toml:31`）。
2. 拆包第一簇补丁让产品包 `build.rs` 变成 274 行（快照 `build.rs`，`wc -l`）：快照 `build.rs` 第 25–26 行 `#[cfg(not(test))]` 加 `#[path = "tests/common_pairing_side_units_entries/mod.rs"]` 把测试登记表编进构建脚本；快照 `Cargo.toml` 第 24–27 行 `[build-dependencies]` 加 `singlefs-unit-graph`，第 29–35 行 `[dev-dependencies]` 加 `prune-server` 与 `singlefs-unit-graph`。所以那一版里「不开任何特性的 `cargo build -p singlefs-background-gpu`」会编、也会跑一份测试登记表加单元图的键计算（构建脚本不挂特性，`load_workspace` 整个工作区）。这是被用户否掉的形态。
3. 今天的键住测试包：`crates/singlefs-background-gpu-tests/build.rs:124` 调 `load_workspace(&repository_root, threads, &switches)`，`:131` 调 `features_enabled_on_the_dependencies`，写 `OUT_DIR` 三张表；测试经 `include_str!(concat!(env!("OUT_DIR"), "/reference_units_keys.tsv"));`（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:36`）编进去读。测试包是 `crates/singlefs-background-gpu-tests/Cargo.toml:15` 的 `singlefs-verification = { path = "../singlefs-verification" }` 依赖验证包的，产品包与验证包互不依赖（门禁 ⑧，`.claude/gate.d/checker-independence-and-sync.sh:38`「两边的传递闭包（dev-dependencies 也算）里都没有对方」）。
4. 对拍结论的键是（输入摘要，两边判法单元指纹）：`crates/singlefs-verification/src/crash_library.rs:3316` 的 `pub fn pairing_conclusion_reused_or_judged(`；任一边拒算「判了、不入库」（`crash_library.rs:3325` 的 `CRASH_LIBRARY_PAIRING_KEY_REFUSED`）。登记表今天 31 个 GPU 一侧组、7 个参照一侧组（`grep -c 'group_name:'`，`crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs` 31、`reference_units_entries.rs` 8 含结构体字段一行）。
5. 键在验证包里的语义是「编进二进制」：`crates/singlefs-verification/src/crash_identity.rs:11` 「二进制比盘上的树旧时它的键对不上新树算出来的键，查库不中就现判」；`:804` 「跑时不读盘上的树，也不收仓根」。
6. 单元图今天的依赖者只有两个包：`grep -rn '^singlefs-unit-graph\s*=' --include=Cargo.toml .`（去掉 `fixtures` 与 `target`）命中 `crates/singlefs-verification/Cargo.toml:20`、`:40` 与 `crates/singlefs-background-gpu-tests/Cargo.toml:22`，即验证包与测试包。门禁 code-tooling 的 crate-roles 格有一条 `support-dependents`：`scripts/gate/crate-roles-check.py:104` 「SUPPORT_DEPENDENTS_MINIMUM = 2」，测试支撑包的依赖者不到 2 个判红；登记表 `.claude/gate.d/crate-roles.tsv:14` 把单元图登成测试支撑、理由写「函数单元图，验证包与产品 GPU 包的测试一侧都依赖」。**测试包撤掉、产品包又不依赖单元图时，单元图只剩验证包一个依赖者，这一格会红**（推的：撤包之后的 `Cargo.toml` 才有这个状态，要拆完再跑 `crate-roles-check.py` 才坐实）。
7. 规则对「测试支撑包」也有同一条：`.claude/rules/directory-layout.md:20` 「至少两个包都无条件依赖的才单开测试支撑包」。

## 1. 候选清单

「键」指对拍结论的三把：录入键（输入加参照一侧加录入段代码）、参照一侧的判法单元键、GPU 一侧的判法单元键（`crash_library.rs:3316` 的键只拼后两把加输入摘要）。每个候选写到文件与函数；「产品构建」指不开任何特性的 `cargo build -p singlefs-background-gpu`。

### 甲：测试运行时从编进测试二进制的源码现算键

- 算键代码住：产品包 `tests/` 下新模块（例 `tests/common_pairing_keys_from_embedded_sources/mod.rs`），经 dev 依赖用 `singlefs-unit-graph`；单元图要新加「从内存里的（路径，文本）装工作区」的入口，今天只有 `crates/singlefs-unit-graph/src/workspace.rs:1367` 的 `pub fn load_workspace(` 读盘（`:672` 的 `read_dir`、`:703` 的 `std::fs::read`、`:1385`、`:1415` 都是读盘，量的）。
- 构建时：只有 rustc 展开 `include_str!("../src/checksum_kernels.rs")` 这类（产品包自己的源与 wgsl、测试自己的文件，同包目录，门禁 ⑨ 不拦）。没有构建脚本。
- 运行时：测试进程里装工作区、收闭包、算三把键，写进库。
- 产品构建：不编测试代码（`tests/` 与 dev 依赖产品构建不碰）。
- 到不了的部分（量的）：产品 GPU 侧的闭包会走进 `singlefs-core`（例：`crates/singlefs-background-gpu/src/checksum_kernels.rs:214` 调 `singlefs_core::checksum::crc32_castagnoli`，`live_slot_kernels.rs:483` 调核心层的 `live_slots_per_cluster_segment_by_the_records`，`region_record_kernels.rs:1354` 调 `scan_region_over_the_identity_index`；整理与挂载运行库两组更是整个核心层的整理路径）。把核心层的 `.rs` 用 `include_str!` 编进产品包测试，门禁 ⑨ 判红（`.claude/gate.d/checker-independence-and-sync.sh:52`「落在另一个 crates/<包>/ 目录下、后缀是 .rs 或 .wgsl 的判红」）；不编进去就收不全闭包。

### 甲变体：测试运行时读活树

- 与甲同，只是 `load_workspace` 读盘上的 `crates/`（路径从 `CARGO_MANIFEST_DIR` 往上推）。
- 键对应的是「跑的时候盘上的树」而不是「编进二进制的代码」：二进制比树旧时，键算的是新树、结果来自旧二进制，库里记成「新树的绿」。`crates/singlefs-verification/src/crash_identity.rs:11` 为这个把键编进二进制，让旧二进制「键对不上新树算出来的键，查库不中就现判」。规则也写了：`.claude/rules/verification.md@一个门禁：两种跑法` 「断言就是关于真仓、要读活树的测试拒算（照跑、不复用、不入库）」——读活树本身就被判成拒算。

### 乙：不按单元键、改按别的输入入库

- 乙一，键取测试可执行文件的摘要（加用例名）：代码住产品包 `tests/` 里读 `std::env::current_exe()` 的一小段；构建时无、运行时读自己的二进制。2026-11-11 用户已否过这个特例：`records/2026-11-11-GPU两段与按需领卡核查.md:128` 「测试可执行文件的摘要加用例名」「两样都是特例」。
- 乙二，键只取（输入摘要，参照输出摘要），不带任何代码身份：改 GPU 一侧的代码键不变。
- 改哪：快照 `tests/common_pairing_input_through_the_library/mod.rs` 的 `recording_key_under`（快照 224）与 `conclusion_key`（快照 308），两处各一个函数，另删构建脚本与三张登记表。
- 产品构建：不编测试代码。

### 丙：键由一个测试支撑包的构建脚本算，产品包测试经 dev 依赖读它

- 算键代码住：新包 P（`crates/<名>/`，角色登「测试支撑」），`Cargo.toml` 写 `[build-dependencies] singlefs-unit-graph`、`[dependencies] singlefs-core`（为了读核心层 `links` 交出的特性，见第 3 格）；`build.rs` 做测试包今天 `build.rs` 的第二件事（`load_workspace`、`features_enabled_on_the_dependencies`、工具链行、`Cargo.lock`、逐组 `entry_group_key_with_the_features_of_the_build`，写 `OUT_DIR` 三张表）；`src/lib.rs` 只放三个 `include_str!(concat!(env!("OUT_DIR"), …))` 常量。
- 入口登记表：由 Rust 模块（今天 `src/reference_units_entries.rs` 与 `gpu_side_units_entries.rs` 经 `#[path]` 编进构建脚本）改成数据文件（例 `crates/singlefs-background-gpu/tests/pairing_units_entries.tsv`）：构建脚本按路径读数据文件，门禁 ⑨ 不拦（`.claude/gate.d/checker-independence-and-sync.sh:52` 只拦 `.rs`、`.wgsl`）。
- 构建时：P 的构建脚本读产品包 `src/`、`tests/` 与核心层源码（`load_workspace` 读工作区，不是依赖关系）。运行时：产品包测试读 P 的三个常量。
- 产品构建：产品包 `[dev-dependencies]` 加 P；`cargo build` 不编 dev 依赖、不跑它的构建脚本（cargo 语义，推的；探针命令里有一条 `cargo build -p singlefs-background-gpu -v` 看日志里有没有 P，待主 agent 跑）。产品包 `Cargo.toml` 里会出现 P 的名字（dev 表）。
- 产品包自己的 `build.rs` 与 `links`：只给测试包的构建脚本读特性用的，P 读不到产品包自己的特性（P 不是产品包的依赖者，cargo 只把 `DEP_<links>_*` 交给依赖者），所以这一版里它们没有读者；E202 装置包（第二簇）若要读，留着，否则删（`.claude/gate.d/crate-cross-references-allowed.tsv` 里 `crates/singlefs-unit-graph/src/build_features.rs` 那一行与单元图的单测随之改）。
- 前置：P 要有至少两个依赖者（`scripts/gate/crate-roles-check.py:104`，`.claude/rules/directory-layout.md:20`）。产品包是一个；第二个要 E202 装置包或别的包无条件依赖它，不然 `support-dependents` 红。

### 丁：键由门禁（外面）算好，经环境变量或键表文件交给测试

- 算键代码住：`scripts/`（Python 的 `scripts/gate/admission.py` 的 `test-plan`：`scripts/gate/admission.py:227` 「test-plan <根> <种类> <包> <计划文件>」逐条测试函数算键、写计划文件第三列；或 `crates/singlefs-unit-graph/src/bin/unit_graph.rs:7` 的 `unit-graph key <根> <入口 fn 路径>`，`--toolchain-of-the-build <profile>` 现拼工具链行，`unit_graph.rs:13`）。
- 构建时无；门禁起 cargo 之前算键、写成文件，路径放进环境变量，测试运行时读；或把键作为 `env!` 在编译时进测试二进制（rustc 把 `env!` 用到的变量记进依赖信息，值变了 cargo 重编，推的）。
- 改哪：`.claude/gate.d/crash-point-replay.sh` 的 product-gpu-real-card-tests 格（`:781` 的 `PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"` 改包名，起 cargo 前导出键表文件）；新脚本（例 `scripts/gate/pairing-side-keys.py`）读登记表、逐行调 `unit-graph key`；测试里只剩读环境变量。
- 产品构建：无关。单项直接 `cargo test … --exact`（不经门禁）读不到键：键拒算，`crash_library.rs:3325` 的语义是「判了、不入库」。

### 戊：产品包自己写算键的新实现，放产品包 `build.rs`

- 这是用户第二句话字面允许的另一条（「需要的话自己去产品中写新实现」）：不用单元图，产品包自己写一份从入口收闭包、取去注释指纹的实现，放产品包 `build.rs`（或产品包 `src/` 的一个模块）。
- 规模参照：单元图 `crates/singlefs-unit-graph/src` 共 19415 行（`wc -l`，量的）；收窄到只服务这一个包的闭包，仍要一份 Rust 与 WGSL 切分、`use` 与 `mod` 连线、cfg 原子、跨包解析。推的：低则几千行、与单元图同一份算法的重写。
- 产品构建：**会编、会跑**：构建脚本属于产品包，每次 `cargo build -p singlefs-background-gpu` 都跑；不挂特性就是产品构建带着测试用的键计算，挂特性就是被用户否掉的那一种。

### 己：只在门禁一层按测试函数复用，库里只做两段交接

- 今天已有：`crash-point-replay.sh` 的 product-gpu-real-card-tests 格对每条真卡对拍测试函数按单元键复用：「按条复用：每条对拍测试函数各一条判定」（`.claude/gate.d/crash-point-replay.sh:275`），键由 `admission.py test-plan` 算（计划函数 `plan_tests`，`.claude/gate.d/crash-point-replay.sh:1793`）。
- 改哪：测试里删掉三张键表与对拍结论入库，只留「录入段 → 判段」交接（输入与参照字节入库、判段读回），交接的键不用代码身份，用门禁给的一次跑的标记（例环境变量里一个随机串）加输入摘要；`pairing_conclusion_reused_or_judged` 那一层不再用。
- 产品构建：无关。

## 2. 每个候选撞不撞前提 1–4

先答两个问题。

**「格式类可以共用」指什么**：`.claude/kb/decisions/13-验证路线.md@已定项 5：只共享一份从 kb 生成的常量` 写的是「其余一律不共享」，共享的只有一份由 kb 字段表生成的常量模块（`crates/singlefs-format`，只发射标量值，门禁 ③ 判它的正文没有 `if`、`match`、`for`），同一节里还写了「格式解析各写一份、校验和各用一份独立实现」。所以「格式类」是 `NODE_BYTES`、`DATA_UNIT_BYTES`、`CLUSTER_SEGMENT_SLOTS` 这类标量（`crates/singlefs-format/src/lib.rs:17`、`:20`、`:324`），不是解析函数，更不是算法。

**单元图算不算**：不算格式类。它是一套 19415 行、带分支与递归的算法（量的，`wc -l crates/singlefs-unit-graph/src/*.rs`）。它的角色是「测试支撑」（`.claude/gate.d/crate-roles.tsv:14`），不是产品。所以「共用」要看两头是谁：**产品包（含它的 `build.rs`）与测试共用它才撞前提 1、2；两个测试一侧（验证包的构建脚本与产品包 `tests/`）共用它，不撞**——被否的拆包第一簇正是把它放进了产品包的 `[build-dependencies]`（快照 `Cargo.toml` 第 24–27 行）。这一读法是我推的：用户原话是「如果是产品包就放入产品中 如果是测试包 就放入测试」，没有直接说「两个测试一侧共用它」，要用户确认。

| 候选 | 前提 1（产品放产品、测试放测试，不用同一份代码） | 前提 2（不许共用，除非格式类；要算键就在产品写新实现或测试写新实现） | 前提 3（产品包测试住它自己的 `tests/`） | 前提 4（按键复用、单项跑、键只由函数单元等组成） | 判定 |
|---|---|---|---|---|---|
| 甲 | 不撞：键代码只在 `tests/` 与 dev 依赖 | 不撞（测试写新入口） | 不撞 | **撞**：闭包到不了核心层（门禁 ⑨）、工具链与 cfg 原子运行时读不到，见第 3 格 | 不成立 |
| 甲变体 | 不撞 | 不撞 | 不撞 | **撞**：读活树被 `.claude/rules/verification.md@一个门禁：两种跑法` 判成拒算；二进制旧于树时键对不上代码 | 不成立 |
| 乙一 | 不撞 | 不撞 | 不撞 | **撞**：键不是函数单元；且 2026-11-11 用户已否 | 不成立 |
| 乙二 | 不撞 | 不撞 | 不撞 | **撞**：键里没有任何代码身份，改 GPU 一侧代码键不变，复用错 | 不成立 |
| 丙 | 不撞，条件：产品包 `[dev-dependencies]` 出现测试支撑包算不算「产品」要用户认；产品构建不编它 | 不撞，条件同上；P 对核心层只借 `links` 元数据 | 不撞 | 不撞（做得全，见第 3 格），条件：P 要有第二个依赖者，否则门禁 `support-dependents` 红 | 成立，附两个要用户定的条件 |
| 丁 | 不撞 | 不撞 | 不撞 | 条件成立：键齐，但门禁之外单项直跑拿不到键，拒算「判了、不入库」 | 成立（条件）；单项直跑的复用要用户认 |
| 戊 | 字面不撞（产品包里放产品自己的新实现，用户第一句话）；但产品构建会编、会跑测试用的键计算 | 字面不撞（用户第二句话列的出路）；代价是第二份算法实现 | 不撞 | 不撞（能做全） | 成立（字面）；撞「产品构建不编测试代码」的要求，推不出，要用户定 |
| 己 | 不撞 | 不撞 | 不撞 | 条件成立：门禁一层按条复用今天就在跑；库内对拍结论复用整层删；单项直跑无复用 | 成立（条件） |

每个「不成立」的什么现象会推翻它：
- 甲：若 `include_str!` 跨包引核心层 `.rs` 被门禁 ⑨ 放行（放行表登一行）且单元图加了内存装载，**并且**测试运行时能拿到编译时的工具链与 cfg（探针里 `at_run_time` 一列出现 `TARGET`、`RUSTC`、`CARGO_ENCODED_RUSTFLAGS` 为 set），甲才站得住。探针命令在末尾。
- 乙一：若用户撤回 2026-11-11 那句「两样都是特例」，且二进制摘要在两台主机上同一份源码编出来逐字节相同（要在两台各编一次比 `sha256sum`，我没做），则剩的问题只是「少复用」，不再是「复用错」。
- 乙二：键里加任何代码身份就不再是乙二。

## 3. 键的组成在各候选下能不能做全

组成（`.claude/kb/decisions/13-验证路线.md@已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成`）：函数单元去注释指纹；闭包真引到的 cfg 原子的真值；工具链（`rustc -vV`、`cargo -V`、目标、RUSTFLAGS、profile 的生效值）；`Cargo.lock` 里闭包真引到的依赖版本；判定自己的内容输入。外加这一仓特有的一件：依赖包在这一次编译里开了哪些特性（核心层有 `[features]` 与 `links = "singlefs-core"`，`crates/singlefs-core/Cargo.toml:10`；`crates/singlefs-replay-prefix`、`singlefs-mount-runtime`、`singlefs-format` 没有 `[features]`，`grep` 量的）。「键过窄（引用的常量、类型、宏漏收）」同样算违反（同节）。

| 组成 | 甲 | 乙一 | 乙二 | 丙 | 丁 | 戊 | 己 |
|---|---|---|---|---|---|---|---|
| 函数单元指纹（含跨包闭包进核心层） | **缺**核心层部分（⑨）：复用错 | 整份二进制：不是单元，过粗：少复用 | **缺**：复用错 | 全（`load_workspace` 读整个工作区） | 全（单元图命令行或 Python 单元算法） | 全（要自己写那份算法） | 全 |
| cfg 原子的真值 | **缺**：测试里读不到 `TARGET`，只能靠 `cfg!` 零散拼：复用错 | 隐含在二进制里 | 缺：复用错 | 全（构建脚本有 `TARGET`、特性） | 全（命令行从工具链行的目标三元组查表，`unit_graph.rs` 文件头） | 全 | 全 |
| 工具链：`rustc -vV` | **缺**：运行时 `rustc` 在 PATH 上不一定是编它的那份：复用错 | 隐含 | 缺 | 全（`$RUSTC -vV`） | 全（`--toolchain-of-the-build`，`unit_graph.rs:13`） | 全 | 全 |
| 工具链：RUSTFLAGS、profile | **缺**：运行时没有 `CARGO_ENCODED_RUSTFLAGS`：复用错 | 隐含 | 缺 | 全 | 门禁环境变量里的 RUSTFLAGS 全（`crates/singlefs-unit-graph/src/bin/unit_graph.rs:15` 写明从 `CARGO_ENCODED_RUSTFLAGS`、没设取 `RUSTFLAGS`）；`.cargo/config.toml` 里写的 rustflags 它读不到（推的：命令行只读环境变量），`[profile.<名>.package.<包>]` 认不了（`crates/singlefs-unit-graph/src/build_toolchain.rs:7` 写明不认）；今天 `.cargo/config.toml` 只有 `[env]`、没有 rustflags（量的），所以今天两处缺口都是 0 处 | 全 | 同丁 |
| `Cargo.lock` 里的依赖版本 | 能：`include_str!` 数据文件不触门禁 ⑨ | 隐含 | 缺 | 全 | 全 | 全 | 全 |
| 依赖包开了哪些特性（核心层） | **缺**：运行时看不到：复用错 | 隐含 | 缺 | 全（P 以普通依赖依赖核心层，构建脚本读 `DEP_SINGLEFS_CORE_ENABLED_FEATURES`；产品包自己的特性 P 读不到，按今天的惯例照开着 `tests-on-a-real-gpu` 收，`crates/singlefs-background-gpu-tests/src/reference_units_entries.rs:73`；产品源里没有按特性编的代码，所以这一缺口不改键，量的） | **缺**：命令行没有 `features_enabled_on_the_dependencies`，要门禁把核心层特性传进来；`test-plan` 有 `--feature <包>/<特性>` 参数可传（`scripts/gate/admission.py:227`），门禁今天给产品 GPU 包传什么没核 | 全（产品包自己的 `links` 元数据直接可读） | 同丁 |
| 内容输入（输入摘要、参照输出摘要） | 全 | 全 | 全 | 全 | 全 | 全 | 全 |

拿不全的算哪一项缺、后果：甲的核心层闭包、cfg 原子、RUSTFLAGS、依赖特性四项缺，后果是**复用错**（核心层改了、`-C target-cpu` 变了、特性变了，库里的绿照样被读成这一趟的绿），这是前提 4 里「一律不进键、也不许当复用依据」的反面，不是只少复用；乙二整个缺，同样复用错；乙一过粗，只少复用（推的）；丁的依赖特性一项要门禁传、传错是复用错（推的）；丙、戊全。

P 的构建脚本读核心层特性的前提：`crates/singlefs-background-gpu-tests/Cargo.toml:10` 「那个取法只认直接的普通依赖，所以写在这里、不写在 dev 依赖里」，所以 P 要把 `singlefs-core` 写在 `[dependencies]`。

## 4. 代价

只写「键」这一层，参照换独立实现的代价在第 6、7 格。数标「量的」是今天文件的 `wc -l` 或 `grep -c`，标「推的」是我估的。

| 候选 | 要写多少 | 要改哪些文件 | 对单项跑与复用的影响 |
|---|---|---|---|
| 甲（不成立，仍列代价） | 单元图加内存装载入口，推的 150–300 行；测试里一份模块；仍补不全核心层 | `crates/singlefs-unit-graph/src/workspace.rs`（`:1367` 旁新加）、`closure.rs` 入口、产品包 `tests/` 新模块、`.claude/gate.d/checker-independence-and-sync.sh:52` 的放行 | 单项跑与复用都在，但复用会错 |
| 乙一 | 测试里两个函数，推的几十行 | 快照 `tests/common_pairing_input_through_the_library/mod.rs` 的 `recording_key_under`（快照 224）、`conclusion_key`（快照 308） | 任何改动都重判；用户已否 |
| 乙二 | 同乙一 | 同乙一 | 改 GPU 核照样复用旧绿：错 |
| **丙** | P：`Cargo.toml` 约 20 行、`build.rs` 约 200–230 行（取测试包 `build.rs` 的第二件事，今天整份 218 行，量的）、`src/lib.rs` 约 10 行；登记表由两张 Rust 表（73 与 168 行，量的）改成一份数据文件约 40 行（38 组，量的，推的行数） | 新建 `crates/<名>/` 三份；仓根 `Cargo.toml` 第 15 行旁的 `members`；`Cargo.lock`；产品包 `Cargo.toml`（去掉补丁里的 `[build-dependencies]`，`[dev-dependencies]` 加 P）；产品包 `build.rs` 回到只交特性的 23 行（或随 `links` 一起删）；快照 `tests/common_pairing_input_through_the_library/mod.rs` 三处 `include_str!(concat!(env!("OUT_DIR"), …))`（快照 58、62、66）改指 P 的三个常量；`tests/common_pairing_side_units_entries/mod.rs`（快照 241 行）改成数据文件；`.claude/gate.d/crate-roles.tsv` 加一行；`.claude/gate.d/crash-point-replay.sh:781` 的包名；`crates/mutations.tsv` 里命名测试包的行（`awk -F'\t' '$5 ~ /background-gpu-tests/'` 数得 263 行，量的，补丁里的 `mutations-replacements.tsv` 已做这一步，与候选无关） | 单项 `cargo test -p singlefs-background-gpu --test <目标> -- <名> --exact` 键都在、复用照常；代价是产品包每一次 `cargo test`（含 `--lib` 与不用键的目标）都要先编 P 并跑它的构建脚本（`load_workspace` 读整个工作区，每次编都要读源码，推的）。键表随核心层任一被收进闭包的文件变而重算（`rerun-if-changed`，同今天） |
| **丁** | 一个脚本约 150–250 行（推的：读登记表、逐行调 `unit-graph key`、写键表文件），门禁格改约 20 行；测试里读键约 30 行 | 新脚本；`.claude/gate.d/crash-point-replay.sh` 的 product 格（`:781` 起 `:4100` 起那一段）起 cargo 前导出键表；产品包 `build.rs` 回到 23 行；登记表数据文件；产品包 `Cargo.toml` 不加任何键相关的依赖 | 门禁里复用照常；门禁之外 `cargo test --exact` 无键，「判了、不入库」（`crash_library.rs:3325`），要靠门禁跑才复用 |
| 戊 | 推的：几千行，与单元图 19415 行（量的）同量级的重写，加上 `build.rs` 的驱动 | 产品包 `build.rs`/`src/` 新模块；测试读 `OUT_DIR` | 复用照常；产品每次构建都多跑它 |
| **己** | 减：测试里去掉三张表、结论入库与 `build.rs` 第二件事；加：一次跑标记约 20 行 | 产品包 `tests/common_pairing_input_through_the_library/mod.rs`（快照 1381 行）里键的部分；`crates/singlefs-background-gpu/build.rs` 回 23 行；门禁格导出标记 | 门禁一层照常按条复用；库内结论复用整层没有；单项直跑每趟都判（要卡的都要领卡） |

## 5. 防再发

要拦的不是某一个候选，而是三件事：产品包重新依赖测试支撑；产品包的构建脚本又编进测试登记表；对拍参照又落在产品代码上。每条都要能在被否的补丁上证红（补丁在快照里，是现成的红样本）。

1. **产品包的 `[dependencies]` 与 `[build-dependencies]`（含 `[target.<条件>.*]`）里出现角色不是「产品」的包判红**：读 `.claude/gate.d/crate-roles.tsv` 的角色列，加在 `scripts/gate/crate-roles-check.py` 的新格（该脚本已经解析三张依赖表，`scripts/gate/crate-roles-check.py:106` 的 `DEPENDENCY_TABLES`；约 30 行，推的）。`[dev-dependencies]` 里允许测试支撑、不允许「验证」（门禁 ⑧ 另判验证包与产品 GPU 包互不依赖）。证红样本：快照 `Cargo.toml` 第 27 行 `singlefs-unit-graph` 在 `[build-dependencies]`，这一格必红。
2. **产品包的 `build.rs` 与 `src/` 不出现测试支撑包的标识符，也不 `#[path]`、`include!` 进本包 `tests/`**：门禁 ⑨ 只判跨包（`.claude/gate.d/checker-independence-and-sync.sh:52`），快照 `build.rs` 第 26 行的 `#[path = "tests/common_pairing_side_units_entries/mod.rs"]` 是同包内的，⑨ 不拦（推的，从 ⑨ 的判法推：「落在另一个 crates/<包>/ 目录下」）。要补一格同包的。证红样本：快照 `build.rs`。
3. **测试支撑包至少两个依赖者**：已有（`support-dependents`，`scripts/gate/crate-roles-check.py:104`）。丙与丁都要它不红：丙要给 P 找第二个依赖者；丁、己下单元图只剩验证包一个依赖者，这一格会红（第 0.1 节第 6 点）。
4. **参照一侧闭包与产品代码不相交**（第 6 格要的）：对登记表里每个参照一侧的入口跑 `unit-graph closure`（`crates/singlefs-unit-graph/src/bin/unit_graph.rs:7` 同一组命令），组成清单里没有任何单元落在 `crates/singlefs-core/src`、`crates/singlefs-background-gpu/src`、`crates/singlefs-replay-prefix/src`、`crates/singlefs-mount-runtime/src`（`singlefs-format` 除外）。证红样本：今天的三个参照入口都调产品 CPU 参照，这一格今天必红（第 6 格列了行）。
5. **丙的专有一条**：P 的 `[dependencies]` 里只许 `singlefs-core`（借 `links` 元数据）与格式包，不许任何产品 GPU 包与验证包；用第 1 条的同一个读表函数判。
6. **已知答案向量必须在**：独立 CRC-32C 参照那一份测试里 `known_answer` 向量表不空、且每条对一个外部来源（RFC 3720 附录 B.4 与公开检验值，模型 `crc32c_section` 先核值）；去掉向量或改错一个值，对拍自检必须红（变异行，见第 7 格）。

## 6. 对拍的参照不得是产品代码

**判定：今天不成立。** 13 个目标里 8 个的断言以产品代码为参照（量的，下表），其余 5 个不拿参照比值。依据：产品 CPU 参照三个函数都只是核心层函数的薄包装，不是另一份实现：

- `cpu_reference_outcome` 调 `singlefs_core::checksum::crc32_castagnoli`：「let computed_crc32c = singlefs_core::checksum::crc32_castagnoli(unit_bytes);」（`crates/singlefs-background-gpu/src/checksum_kernels.rs:214`）；核心层那一份在 x86_64 上走硬件指令、否则 slicing-by-8：「查到 SSE4.2 走硬件指令，否则走 slicing-by-8」（`crates/singlefs-core/src/checksum.rs:84`）。
- `cpu_reference_live_slots` 就是一行转调：「live_slots_per_cluster_segment_by_the_records(allocation_records, unit_area)」（`crates/singlefs-background-gpu/src/live_slot_kernels.rs:483`），本体在 `crates/singlefs-core/src/compaction_plan.rs:127`。
- `cpu_reference_classification` 调核心层区域扫描：「let scan = scan_region_over_the_identity_index(」（`crates/singlefs-background-gpu/src/region_record_kernels.rs:1354`），本体在 `crates/singlefs-core/src/region_scan.rs:408`，分类在 `:702`。
- 它们是产品运行时代码，不只是测试用：两份核连续不确定到 N 轮时交它们裁决（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:110` 的 `reconcile_unless_the_batch_failed`）。
- 规则：`.claude/rules/fs-design.md:22` 「运行时与 checker 必须用不同的算法」，`.claude/kb/decisions/13-验证路线.md@已定项 5：只共享一份从 kb 生成的常量` 「其余一律不共享」。**要求的是算法不同，不只是代码不同。**

### 6.1 逐条列：以产品代码为参照的断言

`T` = `crates/singlefs-background-gpu-tests/tests`（主工作区行号，量的）。「快照」列是快照里同一断言的行号，在 `crates/singlefs-background-gpu/tests/` 下（`grep -nF` 量的，同一份函数体、整体前移 1–2 行）。参照来源写到核心层函数。

| 目标（13 个里的 8 个） | 参照怎么来 | 断言（主工作区） | 快照 |
|---|---|---|---|
| `checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference` | `crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs:64-70`（`cpu_reference_outcomes`）→ 产品 `cpu_reference_outcome` → 核心层 `crc32_castagnoli`；入口 `:143` | `crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:107-110`（查表核）、`:148-151`（逐位核）、`:205`（对账后确认的）、`:248-255`、`:311-318`、`:624-631`、`:718-725` | 109–112、150–153、207、250–257、313–320、626–633、720–727 |
| 同上的输入一侧 | 期望值用核心层 CRC 现算：`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:62`、`:508`；`crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs:189` | 输入里的「对的」期望值就是核心层的值（核心层错则「对的」不对） | 64、510、189 |
| `checksum_batches_over_two_real_gpus_…` | 同上 | `crates/singlefs-background-gpu-tests/tests/checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference.rs:97-104` | 99–106 |
| `checksum_batches_over_three_real_gpus_…` | 同上，断言在共用模块 | `crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs:300-307` 「按卡的顺序填满几张卡，两份核照样逐条与 CPU 参照相同」（`:306`） | 同行（共用模块无位移） |
| `live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference` | `crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:275` 调产品 `cpu_reference_live_slots` → 核心层 `live_slots_per_cluster_segment_by_the_records` | `:338-342`、`:380-384`、`:444`、`:515-523`、`:578-586`；另 `:276-283` 断言「参照里活槽不为 0 的段」够多，也靠参照 | 340–344、382–386、446、517–525、580–588、278–285 |
| `region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference` | `crates/singlefs-background-gpu-tests/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:250` 调产品 `cpu_reference_classification` → 核心层 `scan_region_over_the_identity_index`；造输入的载荷校验和也是核心层 CRC：`:204` | `:698-702`、`:743-747`、`:805-808`、`:855-866`、`:936-947`、`:1003-1035` | 700–704、745–749、807–810、857–868、938–949、1005–1037；校验和 206、参照 252 |
| `compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu` | 参照是**产品的 CPU 模式整理一趟**（选 `cpu` 重挂、推到停）：`crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:182` 的 `reference_bytes_of_the_compaction_run_on_the_cpu`；一张卡那条用产品 `cpu_reference_outcome`（`:206`）；点名不在本机的卡那条用核心层 CPU 上写下的拒法 | `:256` 与 `:826`（`assert_the_pieces_equal_the_reference`，定义在 `crates/singlefs-background-gpu-tests/tests/common_history_pool_through_the_library/mod.rs:124`）、`:377`（拒法文字）、`:487` | 258、828、379、489、184 |
| `mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu` | 参照是**产品运行库选 `cpu` 跑一趟**：`crates/singlefs-background-gpu-tests/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs:189`（`reference_bytes_of_the_runtime_on_the_cpu`） | `:255`（`assert_the_pieces_equal_the_reference`） | 256、190 |
| `background_gpu_compute_agrees_with_the_cpu_backed_test_double` | 见 6.2 | `crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:678-681`（产品实现 vs 核心层测试假实现，逐轮逐条）、`:1027-1058`（GPU 模式整理 vs CPU 模式整理，逐样） | 679–682、1028–1059 |

不拿参照比值的 5 个（量的：`grep -nE 'cpu_reference_(outcome|live_slots|classification)|crc32_castagnoli|live_slots_per_cluster_segment_by_the_records|scan_region'` 在这 5 个里 0 命中；「cpu_reference」字样只出现在测试文件名字符串里）：`mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs`（比的是前台 fsync 与批交错，`:175` 起）、`real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus`、`real_gpu_tests_open_only_the_gpus_the_task_server_handed_over`（数实例与开卡）、`pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes`（持卡判换成只比字节的假判法，测的是键与复用）、`reference_units_entries_name_functions_that_exist`（核登记的入口名在不在）。

分两类看：
- **核对核**（校验和、活槽、区域记录三组，共 5 个目标、23 条用例：`grep -c '^#\[test\]'` 数得校验和 8 + 2 + 1、活槽 6、区域 6，量的）：参照应该是**另一份对同一函数的实现**，可以换成测试自己写的独立参照（6.3）。
- **模式对模式**（compaction、mount_runtime 共 8 个用例，一个替身目标）：两边都是产品（GPU 模式 vs CPU 模式，D26 已定项 13 就是这样要求的「两份实现在测试里对拍」）。小参照换不了：参照是整个整理一趟。要独立，只能在盘上接池级 checker（O2）判结果，见第 7 格。

### 6.2 CPU 替身对拍的另一边是不是也落在产品代码上

**是。** 这个目标比的两边是：产品那一份实现背后接 CPU 替身（`BothKernelsComputedOnTheCpu`），与核心层测试用的假实现。
- 替身的三个方法就是产品 CPU 参照：`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:221`（`cpu_reference_outcome`）、`:289`（`cpu_reference_classification`）、`:363`（`cpu_reference_live_slots`）；期望值输入也用核心层 CRC（`:537`）。
- 另一边的假实现在验证包 `crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:189`（`crc32_castagnoli`）、`:221`（`scan_region_over_the_identity_index`）、`:257`（`live_slots_per_cluster_segment_by_the_records`），都是核心层函数；快照里的拷贝在 `tests/common_cpu_backed_background_gpu_compute/mod.rs` 的 100、124、156 行。
- 两边最后落在**同一批核心层函数**上：「三轮三段逐条相同」那条断言（`:680`）对值是同义反复，它的力量只在接线（打包、切片、`N = 2` 的判定次序），不在值。这个目标还靠 `#[path]` 跨包引验证包的源码（`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11`，`.claude/gate.d/crate-cross-references-lag.tsv` 登了滞后行）；快照已把那份假实现拷成产品包自己的 `tests/common_cpu_backed_background_gpu_compute/mod.rs`，但拷的还是同一批核心层函数。

### 6.3 测试那一侧独立参照写在哪、多大、怎么判「算法不同」

**写在哪**：产品包 `tests/` 下三个只给产品测试用的模块（`.claude/rules/directory-layout.md:20` 「只给一个包的测试用的支撑代码写成那个包的 `tests/common_<名>/mod.rs`」），按组各一个，各目标只 `mod` 自己要的那一个：`tests/common_independent_crc32c/mod.rs`、`tests/common_independent_live_slots/mod.rs`、`tests/common_independent_region_classification/mod.rs`。只许依赖标准库与格式包 `singlefs-format` 的标量常量（`NODE_BYTES`、`DATA_UNIT_BYTES`、`CLUSTER_SEGMENT_SLOTS` 等，`crates/singlefs-format/src/lib.rs:17`、`:20`、`:324`）；输入类型（`AllocationRecord`、`UnitAreaOfOneDevice`、`CompactionRegion`）是核心层的数据类型，只读它们的字段、不调核心层的任何函数（读字段不算共用算法，但要在门禁第 5 格第 4 条里点明哪几个类型豁免，推的）。

| 组 | 独立参照 | 多大（推的） | 与产品那一份的差别 |
|---|---|---|---|
| 校验和 | 一份**非反射、最高位先、逐位**的 CRC-32C（多项式 `0x1EDC6F41`，输入字节与输出各做一次位反转），加 5 条外部已知答案向量（公开检验值 `"123456789"` 与 RFC 3720 附录 B.4 的四条，值与两种写法的互验在模型 `crc32c_section`，待主 agent 跑） | 40–60 行 | 核心层：硬件指令或 slicing-by-8（`crates/singlefs-core/src/checksum.rs:84`），反射形、逐字节或逐字查表；产品 GPU：查表核与逐字移位核（`crates/singlefs-background-gpu/src/checksum_kernels.rs:31`、`:34` 的入口名），片间合成用 GF(2) 矩阵（`:292`）与多项式乘幂（`:371`），都是反射形。独立参照换了**多项式的表示方向**与**位序**，两头靠位反转接回来，不是同一个循环换个变量名 |
| 活槽数 | 每段对每条未释放记录求**区间重叠长度**再求和（`max(0, min(记录末, 段末) - max(记录起, 段起))`） | 30–50 行 | 核心层：逐记录逐槽累加（`crates/singlefs-core/src/compaction_plan.rs:127` 起的双重循环，带 `checked_sub` 与除法定段号）；产品 GPU 两份核：逐条走遍记录、二分查排好的序号。重叠求和没有逐槽循环、没有段号除法 |
| 区域记录分类 | 对每条记录：自己读单元头（三种单元类各一个解析，按格式包的头长与 kb 的字段表偏移）、自己拼映射 key（码 1/2/3 三种取法）、在映射项里**线性找**（不建身份索引）、按「钉住 → 隔离 → 读头 → 类标签 → key 与位置」五步判；结果用测试自己的小枚举，对产品的 `AllocationInRegion` 只比到大类（可搬 + key/类/诞生代、钉住、隔离、死、不自洽三种），不比解析器拒绝的内部原因 | 250–400 行（核心层同一段：`parse_index_node` 85 行、`parse_packed_unit` 约 120 行、`parse_data_unit` 约 64 行、两个 key 构造约 50 行，分别在 `crates/singlefs-core/src/unit.rs:1501`、`:1701`、`:1822` 与 `crates/singlefs-core/src/records.rs:932`、`:958`，这是带 Verus 规格的行数，量的区间；`classify_allocation` 与 `read_unit_header` 在 `crates/singlefs-core/src/region_scan.rs:702` 起到测试模块之前约 140 行；测试版去掉规格但要加一份大类枚举） | 核心层与产品 GPU 的区域核都按「建身份索引再查」走；独立参照线性扫。解析器不共用：`.claude/kb/decisions/13-验证路线.md@已定项 5：只共享一份从 kb 生成的常量` 「格式解析各写一份、校验和各用一份独立实现」 |

**怎么判「算法不同」（同一份算法换个写法不算）**——三道，前两道能机判，第三道靠人与攻方腿：
1. **闭包不相交（机判）**：参照一侧入口的 `unit-graph closure` 组成清单里没有任何单元落在产品与核心层的 `src/`（第 5 格第 4 条）。这判「没共用代码」，判不了「算法不同」。
2. **外部已知答案与手写字面期望（机判）**：校验和有 RFC 向量；活槽与分类各配一张**手算的小例子表**，期望值写成字面（每条一句话写明怎么从 kb 读出，例：「记录 `[63,65)` 跨段界，段 0 得 1、段 1 得 1」），不是代码算出来的。参照实现与产品都对这张表过，说明两边读 kb 的读法同一个；参照实现与产品读法一致、而与字面不一致时红。**这是抓「共同误读」的唯一办法，因为共同误读下两份实现互相一致**。
3. **差异说明（人判，攻方腿证伪）**：每个独立参照模块的文件头写一张「与产品那一份不同在哪」表（上表第四列就是那张表的底稿），审阅时拿它逐行核：表里写的差别，代码里是不是真有（不是只在注释里）。判不过的判据：控制结构与数据结构和产品那一份同构（同样的循环套同样的映射、同样的拒绝分支次序），即使变量名全换也算同一份。

什么现象会推翻「这三道够用」：攻方腿给出一个**同时**让产品与独立参照都过、但与 kb 不一致的输入（说明第 2 道的字面表没盖到那一类）；或第 3 道里有一行「不同」其实是同构（说明人判漏了）。

### 6.4 一个第 6 格自己就带出的 kb 问题（推的，模型第二段会量）

活槽测试输入里有两条**一模一样**的记录（`crates/singlefs-background-gpu-tests/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:129` 与 `:130` 同一条 `record(UNIT_AREA_DEVICE, start + width + 6, 2, false)`）。核心层数法是逐记录逐槽累加（重复的记录数两次）；定义只写在核心层的文档注释里：「按判据打分（活槽数：R 那块盘上未释放记录罩住的槽）」（`crates/singlefs-core/src/compaction_plan.rs:87`）与「未释放记录罩住的槽数」（`crates/singlefs-core/src/compaction_plan.rs:123`）；kb 的决策页里没有：`grep -rl '活槽' .claude/kb/decisions` 只命中 `27-小数据打包容器.md` 与 `09-加密.md`，D26（后台整理与放置回收） 那一页 0 命中（`grep -c`，量的）。写独立参照时要先定：重复记录数一次（槽集合去重）还是两次（逐记录累加）。两种读法在这条输入的第 1 段上差 2（模型 `live_slots_section` 的 `segments_where_distinct_differs`，待主 agent 跑）。这不是候选的问题，是「参照独立写」一写就会撞出来的规格缺口：独立参照的出处不能是产品代码的注释，所以先要 kb 写下活槽数的定义（要主 agent 判哪一种、要不要补进 D26），再定输入里的重复记录留不留。

## 7. 判定与检测强度

先说判定：**换成独立参照之后，核对核那 23 条能抓到今天抓不到的一类（共同误读），抓不到的几类要另补**；模式对模式的 8 条（整理、运行库）换不了小参照，靠接池级 checker 才有独立判官。下面按「产品错」分类，每类写今天红不红、换完红不红、补法与代价。数标量的或推的。

现状里量到的（都是量的）：
- 输入长度格：`const UNIT_LENGTHS_UNDER_TEST: [usize; 7] = [0, 1, 7, 8, 9, 4095, 4096];`（`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:48`）乘 4 种起点余数；另有一条用 64 KiB 的单元：`const UNIT_BYTES_OF_THE_BATCH_SHORT_OF_FREE_MEMORY: usize = 64 << 10;`（同文件 `:327`）。真单元是 16384 与 32768 字节（`crates/singlefs-format/src/lib.rs:17`、`:20`），校验和核那几个目标（`checksum_kernels_…`、两张与三张卡）里只有那一条 64 KiB 的碰过比 4096 大的长度；整理与运行库端到端里走的是 16 KiB 与 32 KiB 的真单元，但它们比的是整个整理一趟，不逐长度比 CRC 值（推的，没逐条核）。输入字节由固定种子的线性同余发生器造（`crates/singlefs-background-gpu-tests/tests/common_checksum_batches_over_the_handed_over_gpus/mod.rs:33`）。
- 变异表 `crates/mutations.tsv` 里点名这三个核对核真卡目标的行 28 条（`awk -F'\t' '$5 ~ /--test (checksum_kernels|live_slot_kernels|region_record_kernels)_on_a_real_gpu_agree_with_the_cpu_reference/'` 数得，产品 GPU 包的 `background_gpu.rs` 8、各 `.wgsl` 19、测试包的共用模块 1，共 28）；**点名产品测试、而改的是核心层 `checksum.rs`、`compaction_plan.rs`、`region_scan.rs` 的行：0 条**（同一个 `awk` 加 `$2 ~ /singlefs-core\/src\/(checksum|compaction_plan|region_scan)\.rs$/`，量的）。那三个文件里有 110 条变异行（3 + 84 + 23，同法数得），命令的 `-p` 点名 `singlefs-core` 34 条、`singlefs-mount-runtime` 11 条、`singlefs-verification` 65 条（`awk` 取 `-p` 后一词，量的）。也就是说：今天核心层那三个纯函数错了，表里登记要抓它们的不是产品对拍测试；这 110 条各自抓不抓得到，我没逐行核。
- 点名整理与运行库端到端目标的行 24 条（同法，量的）。

| 产品错的类别 | 今天红不红 | 换成独立参照后 | 补法 | 代价 |
|---|---|---|---|---|
| A 两份核里只有一份错 | 红（两份核逐条互比，又比参照） | 红 | 无需补 | 无 |
| B 两份 GPU 核与核心层 CPU 参照**共同误读**（同一个位序、同一个跨段口径） | **不红**：参照就是核心层那一份（第 6 格 6.1） | 在独立参照覆盖到的输入上红；覆盖不到的仍不红 | 外部已知答案向量与手算字面期望表（6.3 第 2 道）；CRC 向量取自公开来源 | 向量表约 10 行、活槽与分类手算例各 10–20 条（推的）；一次性写，不随键变 |
| C 核心层 CPU 参照错、GPU 对 | 红，但归因到 GPU（对不上就红） | **不红**（对拍里不再跑核心层） | 核心层自己的测试与变异行罩它；产品里「两份核连续不确定、交 CPU 裁决」那条路要单独测（下一行） | 见下一行 |
| D 两份核连续不确定、交核心层 CPU 裁决后裁决错 | 不红：替身两边都是核心层（第 6 格 6.2），「三轮三段逐条相同」对值同义反复 | 补一条才红 | 在替身对拍里把一边换成独立参照：注入「两份核连续 `N = 2` 轮不一样」，断言产品最终采用的值等于独立参照 | 一个目标里加三条断言，推的约 60 行；要先有独立参照 |
| E 输入取样只覆盖一部分长度与偏移（核在 4097–65535 字节、或某个长度余数上错） | 不红 | 不红 | 分层取样：长度取 0–65、每个 2 的幂两侧各 ±1（128 到 65536）、真单元 16384 与 32768 两侧、固定种子的随机一批；起点余数全取 | 输入总字节约 300 KiB 量级（推的：模型 `length_grid_section` 打出各格的长度数与抓到的假设族数，待主 agent 跑）；真卡一趟几十毫秒量级（推的）；输入过判定库的字节随之变大 |
| F 编排错（切批、切片、显存上限、重复键、连续不确定计数） | 红：现有用例与 `gpu_gc_formal_contracts_through_the_host.rs` 罩 | 同左 | 无需补 | 无 |
| G 整理之后的**盘**错（GPU 路径与 CPU 路径盘上字节相同、但都错） | 不红：比的是 GPU 模式整理与 CPU 模式整理两个产品路径（`crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:1027-1058`、`crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:256`） | 不红 | 两个模式整理完的盘各交池级 checker 判（独立实现，D13 已定项 6 的 O2），并断言 POSIX 内容与整理前相同 | 产品包 dev 依赖 `singlefs-checker`（角色「验证」，门禁 ⑧ 只禁验证包与产品 GPU 包互依，没禁 checker；本仓有没有别的门禁拦，推不出，要主 agent 查）；每个用例多一次池级判，内存盘上的小池，推的 |
| H 打包与回读解码（Rust 外壳）错 | 红（端到端与参照比） | 红 | 无需补 | 无 |
| I 某一型号的卡上才错 | 不红：键不含卡型（`.claude/kb/decisions/13-验证路线.md@已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成` 射程「显卡驱动、卡型、几张卡怎么分活留作块的属性或跑的参数，不进键」），一张卡判绿后换卡复用 | 同左 | 不属于这一轮 | — |

要变异行罩的（把「哪条变异要红」落成表）：
1. 校验和：把独立 CRC 的位反转去掉一处、把末尾取反去掉 → 已知答案那条红；把产品 GPU 两份核之一的初值改掉 → 现有 4 条 `checksum_kernel_*.wgsl` 行仍红（今天已有，量的）。
2. 共同误读不能用单点变异行表达（要同改核心层与两份核）。能写成行的是：在 `crates/singlefs-core/src/checksum.rs` 与两份 WGSL 核**同时**改初值，对拍必须红——变异表一行只改一个文件的一处（`crates/mutations.tsv` 第 2 列文件、第 3 列旧串、第 4 列新串、第 5 列命令，`awk -F'\t' '{print NF}'` 量得 6 列的 8389 行），所以这一类改用字面期望表的**自检**代替：把字面表里一个期望值改错，字面期望那条测试必须红（一条变异行，改的是测试文件）。
3. 活槽、分类：改独立参照里一处（跨段界取等号、钉住与隔离的先后），字面期望表那条测试红。

什么现象会推翻「B 类换完能抓」：独立参照写出来后在全部取样输入上与产品一致、而攻方腿给出一个与 kb 不一致的输入同时被两边放过（说明字面表覆盖不够，回到 6.3 第 2 道补）。
什么现象会推翻「C 类换完不红是可接受的」：若主 agent 判核心层那三个纯函数在别处没有罩（变异表 0 行、验证包也没有对拍），则 C 类要靠这里的字面期望表直接测核心层函数——那就等于在产品包测试里再测一遍核心层，要用户认。

## 8. 推荐与代价

**推荐：丙（测试支撑包的构建脚本算键，产品包测试经 dev 依赖读）＋第 6 格的独立参照＋第 5 格前四条检查。丙不成时的退路是丁。** 理由（推的，不是裁决）：

1. 用户 2026-11-11 要的是所有地方同一套键逻辑、不留特例：「我希望所有的地方都是一致的逻辑 不要做特例」（`records/2026-11-11-GPU两段与按需领卡核查.md:126`）。丙用的是验证包 `build.rs` 同一套机制（`singlefs-unit-graph` 的 `entry_groups`、`build_toolchain`、`build_features`），键的组成逐项做全（第 3 格），单项直跑也有键；产品包的构建里没有任何测试代码、测试登记表与单元图依赖。丁换了一份算键的地方（门禁与 Python），门禁之外拿不到键；己的交接键是「一次跑的标记」，是用户 2026-11-11 说过的那种特例。
2. 甲、乙在前提 4 上过不去（第 2、3 格），不是代价问题，是复用会错。
3. 戊字面上是用户第二句话列的出路，但要在产品包 `build.rs` 里写第二份与单元图同量级的算法，产品构建每次都编它；用户若坚持「产品只放产品」，戊就不如丙。

**丙落地要先定的三件事（我定不了）**：
- 产品包 `Cargo.toml` 的 `[dev-dependencies]` 出现测试支撑包 P（与单元图，因为测试要它的 `sha256`、`entry_groups` 读表），算不算「产品」一侧。我读成不算（dev 依赖产品构建不编）；用户的原话没直接说，要弹窗确认。
- P 的第二个依赖者：`scripts/gate/crate-roles-check.py:104` 要求至少 2 个；候选是第二簇搬进实验包的 E202 装置（它的 `bin:` 对拍组也要算键）。E202 不用 P 时，P 过不了 `support-dependents`，要用户定是改登记还是换成丁。
- 活槽数的定义写进 kb（6.4），不然独立参照没有出处。

**总代价（量的与推的分开写）**：

| 件 | 量的（今天文件） | 推的（新写） |
|---|---|---|
| 键：P 包 | 测试包 `build.rs` 218 行、登记表 73 + 168 行 | P 约 250 行 + 数据文件约 40 行 + 产品包 `Cargo.toml` 与 `build.rs` 改动、`crate-roles.tsv` 一行、`.claude/gate.d/crash-point-replay.sh:781` 的包名 |
| 独立参照 | 核心层对应的实现 85 + 约 120 + 约 64 + 约 50 + 约 146 行（带 Verus 规格） | CRC 40–60、活槽 30–50、分类 250–400、字面期望表 30–60 |
| 取样 | 长度格 7 种，64 KiB 一条 | 分层取样改输入发生器约 50 行；输入约 300 KiB 量级 |
| 裁决路径测试（D 类） | 替身对拍目标 11 条用例 | 约 60 行 |
| 池级 checker 判整理后的盘（G 类） | 8 个模式对模式用例 | 每个用例加几行 + dev 依赖，先查门禁是否拦（推不出） |
| 门禁 | `scripts/gate/crate-roles-check.py` 已解析三张依赖表 | 新格：产品包依赖表角色（约 30 行）、同包 `#[path]` 进 `tests/`（约 30 行）、参照闭包与产品不相交（约 80 行，要起 `unit-graph closure`）；每格要弄坏开关与样本档 |
| 变异表 | 命名测试包的行 263 条、核对核真卡目标的行 28 条 | 改命令里的包名（补丁里的 `mutations-replacements.tsv` 已做）；加字面期望表自检行约 3 条 |

对单项跑与复用的影响（丙）：`cargo test -p singlefs-background-gpu --test <目标> -- <名> --exact` 键齐、复用照常；改核心层被收进闭包的任一文件，P 的构建脚本重跑、31 + 7 组键重算，产品包测试重编（今天的测试包同样）；产品包的**每一次** `cargo test`（含 `--lib`、不用键的目标）多编一个 P 与它的构建脚本（`load_workspace` 读整个工作区，推的几秒到十几秒，没量）。

### 要用户或主 agent 定、我推不出的
1. 产品包 dev 依赖测试支撑包算不算产品一侧（上面第一件）。
2. 门禁之外单项直跑要不要有复用（决定丙还是丁）。
3. 活槽数的 kb 定义；输入里的重复记录留不留（6.4）。
4. 产品测试能不能 dev 依赖 `singlefs-checker`（G 类的补法）。
5. 门禁点名测试函数（`--item`）能不能点到产品 GPU 包的真卡对拍测试：我没查 `crash-point-replay.sh` 的逐项点名那一段有没有这一类（只读了文件头第 39–58 行的写法），推不出。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 候选清单 | 成立 | 甲、甲变体、乙一、乙二、丙、丁、戊、己共 8 个，各写到文件与函数，产品构建会不会编测试代码逐个写明（戊会） |
| 2 撞不撞前提 | 甲、乙不成立；丙、丁、己条件成立；戊推不出 | 甲、乙撞前提 4；丙的条件是 dev 依赖与 P 的第二个依赖者；戊字面不撞、但产品构建要编测试用的键计算；「格式类」只指 `singlefs-format` 的标量常量，单元图不算 |
| 3 键的组成做全 | 丙、戊成立；甲、乙不成立；丁条件成立 | 甲缺核心层闭包、cfg、RUSTFLAGS、依赖特性四项，后果是复用错；乙二整个缺；丁的依赖特性要门禁传 |
| 4 代价 | 成立 | 丙 P 约 250 行加登记表约 40 行；单元图依赖者数要补；每个候选的文件与单项跑影响见表 |
| 5 防再发 | 成立 | 六条检查，其中产品包依赖表角色、同包 `#[path]` 进 `tests/`、参照闭包与产品不相交三条要新写，证红样本就是被否的补丁 |
| 6 参照不得是产品代码 | 今天不成立；核对核可换独立参照成立；模式对模式推不出 | 13 个目标里 8 个以核心层函数为参照；替身对拍另一边也是核心层函数；独立参照 CRC 40–60、活槽 30–50、分类 250–400 行；活槽数 kb 里没有定义 |
| 7 判定与检测强度 | 成立（B 类可抓，D、E、G 类要补） | 换完能抓共同误读；抓不到：取样缺口（长度只到 4096 加一条 64 KiB）、裁决路径、整理后的盘；各写了补法与代价 |

## 要主 agent 跑的命令

每条一行、一项；复跑前我没有跑过任何一条（只做了 `bash -n`、`python3 -m py_compile` 与只读命令）。

1. 模型复跑（CRC-32C 两种写法互验与已知答案、活槽数三种读法在测试输入形状上的数、长度取样格抓到的假设族数）。
   - 命令：`task-server/task submit -- bash research/prompts/product-test-pairing-keys-design-r1-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo /home/fy5090/.local/share/singlefs/work/product-test-pairing-keys-design-r1-sonnet-model-rerun-run /home/fy5090/code/singlefs/research/prompts/product-test-pairing-keys-design-r1-sonnet-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded` 与 `exit=0`；日志里 `name=summary checks=… failures=0`、没有 `name=FAIL`；`name=crc_known_answer` 十行的 `expected` 与 `got` 逐行相同；`name=live_slots_area` 两行里 `segments_where_distinct_differs` 非空；`name=length_grid` 两行的 `caught` 数（现有格应少于分层格）。`failures` 非 0 时先看是不是我记错了 RFC 向量。
2. 探针：没有构建脚本的测试目标在编译时与跑起来时读不读得到 `TARGET`、`RUSTC`、`CARGO_ENCODED_RUSTFLAGS` 等（第 2 格推翻甲的条件）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=探针包 what-a-test-target-sees 的测试目标 probe 里的 1 条测试; 范围=1 个测试; 为什么=核测试目标在编译时与跑起来时各读得到哪几样 cargo 给构建脚本的环境，决定甲能不能拿到工具链' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/product-test-pairing-keys-design-r1-forward/probe -- cargo test --test probe -- what_the_test_sees --exact --nocapture`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/product-test-pairing-keys-design-r1-forward/probe`（空工作区的小包）。
   - 看哪几行：日志里 9 行 `name=probe variable=<名> at_compile_time=<set|unset> at_run_time=<set|unset>`；`TARGET`、`RUSTC`、`CARGO_ENCODED_RUSTFLAGS` 三行的两列都是 `unset` 时，甲的运行时拿不到工具链成立；任一列为 `set` 就按那一格改第 3 格。
3. 产品包今天与快照里的非 dev 依赖（核第 0.1 节第 2 点与第 4 格「产品构建」一列）。
   - 命令一：`task-server/task submit -- cargo tree --offline --locked -p singlefs-background-gpu -e normal,build --depth 1`，在仓根起。
   - 命令二：同一条，`--cwd /home/fy5090/.local/share/singlefs/work/merge-and-verify/2026-11-19-3970763/repo`。
   - 看哪几行：命令一的输出里没有 `singlefs-unit-graph`；命令二的输出里有 `singlefs-unit-graph`（那是被否的形态）。
4. 参照闭包里有没有核心层单元（第 5 格第 4 条与 6.3 第 1 道的演示，今天必红）。
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- cargo run --release -p singlefs-unit-graph --bin unit-graph -- closure . singlefs_background_gpu::checksum_kernels::cpu_reference_outcome --target lib --toolchain-of-the-build dev --lock-file Cargo.lock`
   - 看哪几行：输出的组成清单里有路径在 `crates/singlefs-core/src/checksum.rs` 的单元行；没有就是我对闭包的推断错了（命令的选项是照 `crates/singlefs-unit-graph/src/bin/unit_graph.rs` 文件头写的，没试过）。

## 没做什么

- 不判别的腿的格、不替主 agent 采纳或出判决；没往统一任务服务交任务、判决要引的数没复跑（命令在上一节）；模型、探针没跑，模型里的数都是「待主 agent 跑」，报告里没有写成量的。
- 没起 cargo：`cargo tree` 被权限闸拒了一次，我没有换法绕过去，已写进上一节第 3 条。
- 没读攻方腿这一轮的任何产出。没读 `scripts/gate/lib_function_units.py` 的算法正文，丁里「Python 单元算法与单元图同一套键」只引了 `scripts/gate/admission.py` 的用法说明，没核两边键逐字相同。
- 没核两台主机上同一份源码编出的测试二进制摘要是否逐字节相同（乙一的「少复用」那一半）。
- 没查 `crash-point-replay.sh` 逐项点名能不能点到产品 GPU 包的真卡对拍测试（上面推不出第 5 条）。
- 没量区域分类独立参照的真实行数（250–400 是从核心层同一段的行数推的）；没量 P 的构建脚本每次编的耗时。
- 没改仓里任何别的文件；报告与模型目录是这一腿的产出，草稿与探针在草稿目录。
