# gpu-tests-placement-design-r2：正推腿报告（2026-11-19）

批次：records/2026-11-19-产品GPU包测试与E202装置归位.md#2。正文 `research/prompts/_gpu-tests-placement-design-r2-body.md`，第一轮判决 `research/prompts/gpu-tests-placement-design-r1-main-verification.md`。方案按 `crates/` 今天的实现来谈（读码时仓 HEAD 是 4319aa2b0，工作区另有别的会话未提交的改动，行号以我读到的那一刻为准）。

## 0. 读法

- 标「读码」的是我读源码得到的；标「量的」的是我跑只读命令（`grep`、`git grep`、`wc` 与草稿目录里的小脚本）得到的，脚本与输出在草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r2-forward/`；标「推的」的没量过。行数一律是「非空、非 `//` 注释」的代码行（`codelines.py`），带注释的物理行数大约多四分之一。
- 没写模型目录、没起样本进程；草稿目录里是五份量数脚本（`path_attr_scan.py`、`include_scan.py`、`discrete_closure.py`、`codelines.py`、`gate_starts_bin.py`，共约 140K 连量数中间文件），只读、只用标准库，没有 `rerun.sh`。要跑项目代码才能量的数（现判多少、键变几组）写进「要主 agent 跑的命令」。
- 没读禁读清单里的任何文件。

### 0.1 与背景材料「实现今天的样子」对不上的几处（量的）

| 背景材料写的 | 我现量的 |
|---|---|
| ⑧ 盲区句在 `checker-independence-and-sync.sh:45` | 现在在 `:46`：「`include_str!` 跨包目录引源码或着色器文件不认」（`.claude/gate.d/checker-independence-and-sync.sh:46`） |
| 全仓 16 处 `#[path]` | `git grep -nE '#\[path = "\.\./\.\./' -- crates` 命中 16 行，其中 1 行是 `crash_amplification.rs:1292` 的文档注释、1 行是三层 `../../../`；真属性 15 处。全仓 `#[path` 属性一共 190 处（`path_attr_scan.py`），186 处同包、4 处跨包 |
| 6 处跨包 `include_str!` | 字面路径落在另一个包目录里的 11 处（e202 摘要读 5 份 `Cargo.toml`、单元图单测读 4 处、读测试包与验证包构建脚本 2 处）；落在 `crates/` 之外的数据文件 11 处（`.claude/heavy-thread-default` 5、`litmus/` 6）。「6」只对得上后一类里的 `litmus/` 那 6 处（`include_scan.py`） |
| 26 个装置每一份都引验证包自己的模块（2 到 56 处） | `e184_regular_layer_cost_counts_write_dump.rs` 本体 0 处，它的 `…write_dump/mod.rs` 里 8 处（`grep -cE '(crate|singlefs_verification)::'`） |
| `task_server_lib.py:2029` 把选卡变量列为任务带了就拒 | 现在在 `task-server/task_server_lib.py:2046`（`CARD_CHOICE_VARIABLES = ("GPU_CARDS_BY_PRIORITY", …`） |
| 测试包 `#[path]` 引 `tests/common/mod.rs` 的有 6 份 | 对：e161、e170、e171、e171 修后版、e184、e185 共 6 份、9 处属性（e184 引 4 份共用模块）。验证包 340 个测试目标 `mod common;` 也对（`grep -lE '^\s*mod common;' tests/*.rs` 340） |

## 1. 两侧录入键逐字节相同怎么证

### 1.1 验证包那一套今天写什么（读码）

- 录入键 = SHA-256（文本 `singlefs recording key\nrecording_units=<录入函数单元指纹十六进制>\ninputs=<录入输入摘要十六进制>\n`）；录入函数单元拒算时写 `recording_units=refused`（「singlefs recording key」，`crates/singlefs-verification/src/crash_library.rs:1789`、`:1801`）。
- 有参照的对拍，录入输入摘要 = SHA-256（`singlefs pairing input with the reference\nreference_units=<hex>\ninput=<sha256(输入) hex>\n`，`crash_library.rs:2394` 起的函数）；参照一侧拒算时退回只拼输入摘要、`skips_the_recording_on_a_hit=false`（`crash_library.rs:2445` 起）。
- 清单字节：`SFITEM01` ‖ u64 小端长度加 kind ‖ u64 小端个数 ‖ 逐条（u64 小端长度加名字 ‖ 32 字节摘要）（`crash_library.rs:1819` 魔数、`ItemManifest::to_bytes`）。
- 行：`0x10 ‖ 录入键` → 清单（`verdict_store.rs:1790`）；`0x1C ‖ sha256(字节)` → 字节本身（`:1802`）；`0x19 ‖ 输入摘要 ‖ 一边指纹 ‖ 另一边指纹` → u32 大端长度 ‖ 结论字节 ‖ 1 字节一致码（1 一致、2 不一致）‖ u32 大端长度加东京日期文本 ‖ 1 字节跑法码（2 剪枝、3 逐条）（`:1799`，值布局 `:572` 起的 `to_bytes`）；`0x09 ‖ 指纹` → 组成清单文本（一行 `<单元名>\t<十六进制>`）；`0x00` → 布局号 6（4 字节大端，`:1655`）。全部经 `prune_server::reliable_client`，键原样（全仓那一份没有名字空间前缀，`:2782` 起的 `LibraryKeySpace::TheOneLibrary`）。

### 1.2 按定义两侧不同的几段，与相同的几段

| 段 | 两侧 | 为什么 |
|---|---|---|
| 录入键里的 `recording_units` | 不同 | 验证包取它自己的 `Recording` 组（入口登在 `judging_units_entries.rs`，流录入与 GPU 判表建表）；产品一侧取产品 `build.rs` 登记的自己那一组。同一份输入两侧的 `0x10` 键本来就不同 |
| `0x19` 键里一边、另一边的判法单元指纹 | 不同 | 参照侧与 GPU 侧的闭包全在产品包（登记表 31 加 8 组，`gpu_side_units_entries.rs`、`reference_units_entries.rs`） |
| `0x09` 行的值 | 不同 | 组成清单每行带 `<种类> <限定名> @ <文件路径>`（`crates/singlefs-verification/src/crash_identity.rs:787`），路径与限定名是各自包里的 |
| `0x19` 值里的结论字节、日期、跑法码 | 不是算法不同，是随用例与那一趟变 | 结论字节是调用方给的；日期取东京日期；跑法码取 `SINGLEFS_CRASH_LIBRARY_RUN_KIND` |
| 录入键的文本模板、录入输入摘要的文本模板、清单字节、`0x1C` 行、`0x19` 键与值布局、`0x00` 布局号 | 必须相同 | 这是「输出的键值要一样」（`records/2026-11-11-GPU两段与按需领卡核查.md:141`）能管到的全部：库里别的读者（`crash_library_reader` 数 `0x10`、`units_changed_between` 读 `0x09`）按验证包的布局解，产品写成别的样子它们解不出来 |

结论：「逐字节相同」只能写成谓词 P——**给定同一组参数**（输入字节、参照字节、录入单元指纹 Fr、参照单元指纹 Fref、GPU 一侧指纹 Fg 或其中一个「拒算」、结论字节、一致码、日期、跑法），两侧各自会写进库的（键，值）行集合逐字节相同；三把指纹当参数注入，不比它们本身。这一句要写进验证包与产品包两边测试的文件头，不然「键一样」没有可核的意思。

### 1.3 证法甲：两侧钉同一张黄金表，一道门禁格比两份钉表

做法：两个包各在 `tests/` 下放一份同样内容的 `pairing_row_golden.tsv`（输入字节十六进制、Fr、Fref、Fg 取固定字节、结论、一致码、日期、跑法 → 期望的录入输入摘要、录入键、清单字节、`0x1C`、`0x10`、`0x19` 键与值各一行十六进制）；两边各一个测试按自己的代码算、逐行比；门禁新格（放 `.claude/gate.d/checker-independence-and-sync.sh`，与 `layout-checker-sync` 同类）逐字节比两份表。
- 拦得住：任一侧改了算法没改表（自己那边红）；一侧改表另一侧没跟（门禁红）；表里每个变量的取法（模板、字节序、长度前缀）。
- 拦不住：表没覆盖到的分支（录入单元拒算那一支、参照侧拒算那一支、日期跨年边界、值长过一帧要分段存）；两侧同时改成同一个错的样子；写库那一步的行为（表标签写错、没同步落盘、分段存的格式）——表里只有「该写什么」，没有「写进去了什么」。

### 1.4 证法乙：一道门禁格两侧各算一次再比

做法：两边各出一个只打印的测试（或小二进制）`PAIRING_ROW name=… key=… value=…`，对同一份确定性生成的语料（长度 0 到 N 的输入若干、三把指纹各取「有」与「拒算」、一致码两种、日期与跑法各几种）逐条算行；门禁格起两边、diff 两份输出。
- 拦得住：语料里任一分支上两侧算出不同；不需要黄金表那样人手钉值，两侧一起漂移也不会漏（比的是两边此刻的算出值，不是两份存着的常数）。
- 拦不住：语料外的分支；写库行为；两侧共有的、与验证包别的读者不一致的约定（比如读者期望 `0x10` 值里带的 kind 串）；起两边要编译两个包，门禁挂钟长。

### 1.5 证法丙：对一份临时库交叉写读

做法：门禁格在临时 prune-server 底下（`scripts/prune-server/prune_server_client_selftest.py --with-temporary-service`）先起产品一侧的测试目标把一组行写进库，再起验证包一侧的测试用 `VerdictStore` 的 `read_recorded_run_manifest`、`read_pairing_input`、`read_pairing_conclusion`、`read_judging_units_composition` 读回并逐行核；方向反过来再做一遍（验证包写、产品读）。
- 拦得住：表标签、键拼法、值布局、分段存、同步落盘、布局行、读者解不出的任何一处；产品写出来的行验证包的读者（`crash_library_reader` 一类）真能解。
- 拦不住：没写过的值域（长值、特殊日期）；两边读写函数同时错成「自洽」的形态——但读者是验证包那一份，所以自洽不了：它是三种里唯一把「验证包读得懂」当判据的。

### 1.6 推荐与判定

甲加丙两种是最小一组：甲让每个包自己的 `cargo test` 就能红、改动当场知道；丙拦写库那一层。乙作加强项，门禁挂钟允许时加。单用一种不够：只有甲拦不住写库层，只有丙拦不住算法在语料外漂移。

**判定：成立**（证法写得出、各有拦与不拦）。**算不算另开判定存放处：不算。** 产品一侧写的是同一份 prune-server 的 RocksDB、键原样、同一个客户端 `prune_server::reliable_client`，与 `.claude/rules/verification.md` 同一条的写法一致：「不许任何测试、门禁、入口脚本、实验装置、agent 新建自己的判定库」（`.claude/rules/verification.md:56`）禁的是另开目录、TSV、sqlite、另一份 RocksDB，不是另写存取代码。要守的两条边：产品一侧不得开 RocksDB、不得写 TSV 当判定；黄金表与交叉样本是测试输入、不是判定（交叉样本只对临时服务写，不写真服务）。

什么现象会推翻：产品一侧的存取代码里出现 `rocksdb`、`sqlite` 或写本地文件当判定；丙对着临时库写读都绿而对真库读不回（分段存或键空间前缀两边对不上）；谓词 P 里的某个「相同」段被验证包改了而产品没改，三种证法里没有一种红（说明门禁格没接上）。

## 2. 产品 GPU 包测试要的存取代码有多大、放哪

### 2.1 照 `use` 链列出的最小一组（读码）

产品测试今天从验证包用到的，逐文件 `grep -n singlefs_verification` 的结果：8 个测试目标与 2 个共用模块只用 `crash_library::PairingJudged` 一个类型；共用模块 `common_pairing_input_through_the_library` 用 `crash_identity::{judging_units_key, judging_units_key_of_the_group_in_the_table_text, JudgingUnits, JudgingUnitsKey}`、`crash_library` 的 `pairing_conclusion_reused_or_judged`、`pairing_input_and_reference_through_the_library_under_the_recording_key`、`pairing_input_and_reference_through_the_pairing_library`、`pairing_input_through_the_pairing_library`、`recording_key_of_the_pairing_input_and_the_reference_units`、`toolchain_of_this_process`、`CrashLibrary`、`CrashLibraryClass`、`ItemName`、`PairingConclusionSource`、`PairingJudged`、`RecordingKeyOfThisRun`、`RunStage`、`StageOutcome`，`gpu_card_configuration::{cards_handed_over_or_leased_on_demand, CardLeaseChannelOfTheCaller}`，`verdict_store::PairingAgreement`（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:16-32`）；对拍结论那个目标另用 `DisposableLibrary`、`sha256::DIGEST_BYTES`（`tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs:26-30`）。这些函数往下各自又拉的东西（量的，读 `crash_library.rs`、`verdict_store.rs`、`gpu_card_configuration.rs`）：

| 链上的东西 | 产品要自己写的最小形态 | 对应验证包代码行数（代码行，`codelines.py`） |
|---|---|---|
| 录入键与对拍录入输入摘要、拒算那一支 | 两个纯函数，文本模板逐字照 1.1 | `crash_library.rs:1754-1812` 47、`:2381-2520` 里键与摘要那三段约 35 |
| 清单字节编解码 | `ItemManifest` 的 `to_bytes`、`from_bytes`、`digest_named` | `crash_library.rs:1814-1900` 75 |
| 对拍结论键、值、一致码、日期、跑法码 | `PairingConclusionKey`、值的编解码、`ByteCursor`、东京日期换算、`SINGLEFS_CRASH_LIBRARY_RUN_KIND` | `verdict_store.rs:521-600` 64 加跑法码与日期约 60、字节游标约 25 |
| 组成清单行 | `0x09 ‖ 指纹` 的值；单元名取 `<种类> <限定名> @ <文件>` 加重名序号 | `verdict_store.rs:5384-5396` 12、`crash_identity.rs:777-801` 约 25 |
| 连库与读写 | 取路由（socket 或多机配置 `PRUNE_SERVER_TCP_ADDRESS`）、`ReliableClient` 连接、点读写（值长过一帧走 `write_segments_of_large_values`、`assemble_segmented_value`）、布局行核对 | `crash_library.rs:272-430` 121 的一半、`verdict_store.rs:2888-2994` 84、`:6510-6545` 29 |
| 两段控制 | `RunStage` 读 `SINGLEFS_CRASH_AMPLIFICATION_STAGE`、`two_stages_under_the_recording_key`、清单按录入键登与读 | `crash_library.rs:1621-1650` 20、`:1901-1976` 52、`:1978-2077` 82 的主干 |
| 对拍输入与参照读回、核摘要 | `PairingInputReadBack`、读回 | `crash_library.rs:2264-2330` 44、`:2473-2520` 约 45 |
| `pairing_conclusion_reused_or_judged` | 读结论、复用一致的、判完存、写两把组成清单 | `crash_library.rs:2565-2655` 73 |
| 临时服务（只给「结论复用」那一个测试目标） | `prune_server::server::start_in_this_process`、独占临时目录、socket 路径超长退到短目录 | `crash_library.rs:433-530` 80 加 `:742-870` 里与线程额度无关的部分约 40 |
| 按需领卡 | 见 2.3 | `gpu_card_configuration.rs:340-722` 的有用部分约 270，产品只要其中 110 上下 |

不要从验证包带过来的：`LibraryOpenInThisProcess` 的进程内共享与持锁计时（`crash_library.rs:1306-1420`，约 110）、`ValuesReadBackOnceInThisProcess` 的流缓存（只对崩溃放量有用）、`ThreadsInUseInTheTestProcess` 的线程额度（`:596-742`）、工具链身份（`toolchain_of_this_process`，验证包自己都不用它的入参：`crash_identity.rs:809` 的 `_toolchain`）、`VerdictStore` 的判定向量编号、块表与照判不入库那一套。`sha256` 不用抄：`prune_server::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES}` 现成（`prune-server/src/lib.rs:62`，`prune-server/src/sha256.rs:109`），工作区没有 `sha2` 一类第三方包（`grep '^name = "sha' Cargo.lock` 零命中）。

### 2.2 落成产品包 `tests/` 下的哪几份、各多少行（推的，按上表代码行）

| 文件 | 装什么 | 代码行（推的） |
|---|---|---|
| `crates/singlefs-background-gpu/tests/common_pairing_row_layouts/mod.rs`（新） | 纯函数、不碰 IO：行标签常量、录入键、录入输入摘要、清单编解码、对拍结论键与值编解码、日期与跑法码、组成清单文本 | 约 330（25 加 55 加 65 加 155 加 25） |
| `…/tests/common_prune_server_library/mod.rs`（新） | IO：取路由、连接、点读写与分段存、布局行核对、两段控制、对拍输入存与读回核摘要、`pairing_conclusion_reused_or_judged` | 约 330（35 加 40 加 45 加 20 加 85 加 30 加 60） |
| `…/tests/common_discrete_gpus_of_this_host/mod.rs`（`git mv` 来，加一段） | 取卡模块里加按需领卡（2.3） | 现 859 行加约 110 |
| `…/tests/common_pairing_input_through_the_library/mod.rs`（`git mv` 来，改引用） | 对拍用例共用的胶水：`judgement_of_the_assertions_on_the_cards`、`pairing_concluded_on_the_cards_in_the_library`、输入编码 `append_*`；只把 `use singlefs_verification::…` 换成上面两份与自己的 `PairingJudged`、`PairingAgreement`、`CrashLibrary` | 现 493 行，改约 40 行 |
| `…/tests/pairing_conclusion_on_the_cards_is_reused_by_its_keys_and_judged_again_when_a_side_changes.rs`（`git mv` 来） | 一次性库那一段改用自己起的临时服务（`start_in_this_process`），写在这个目标自己里 | 加约 60 |
| 两个证法的测试目标（1.3、1.5，新） | 黄金表比对、对临时库写读 | 各约 100（推的） |

新写的存取代码合计约 **830 代码行**（330 加 330 加 110 加 60），连注释约 1000 行；不含 1.3、1.5 的测试。CPU 替身 `tests/common_background_gpu_compute/mod.rs`（F3，验证包那份 310 行只依赖核心层）另算，照验证包做法在产品 `tests/` 下建一份，不进这一格。

### 2.3 `[dev-dependencies]` 与 `[build-dependencies]` 加什么

产品包 `Cargo.toml` 今天 `[dependencies]` 已有 `singlefs-core`、`singlefs-replay-prefix`、`singlefs-mount-runtime`、`wgpu`、`ash`、`libc`（`crates/singlefs-background-gpu/Cargo.toml:12-23`），`[dev-dependencies]` 有 noop 版 `wgpu`。要加的只有三行：
- `[dev-dependencies] prune-server = { path = "../../prune-server" }`（连库、分段存、临时服务、sha256）；
- `[dev-dependencies] singlefs-unit-graph = { path = "../singlefs-unit-graph" }`（运行时用它的 `parse_entry_group_table` 读 `build.rs` 写的键表，省约 40 行自写解析）；
- `[build-dependencies] singlefs-unit-graph = { path = "../singlefs-unit-graph" }`（F4，`build.rs` 算键）。
`ash`、`singlefs-replay-prefix`、`singlefs-mount-runtime`、`wgpu` 测试包里的那几行产品包已有，不加。没有别的。`[[test]]` 照测试包今天那五条 `required-features = ["tests-on-a-real-gpu"]` 搬过来（`crates/singlefs-background-gpu-tests/Cargo.toml:35-53`），产品包里那个原本空着的 `tests-on-a-real-gpu = []`（`crates/singlefs-background-gpu/Cargo.toml:34`）从此是真被 `required-features` 用到的特性，它上面那段「本包的源码里没有按这个特性编的代码…」的注释要改成现状。

### 2.4 按需领卡那一段怎么写，撞不撞「卡只归 GPU 服务，任务不拿卡」

写法（推的，对着 `gpu_card_configuration.rs:340-722` 与 `gpu-verification-server/control` 的 `lease`）：放在 `common_discrete_gpus_of_this_host/mod.rs`，因为「领到的卡记给取卡那一份」那个静态量本来就在那里（`:666` 的 `CARDS_LEASED_ON_DEMAND_FOR_THIS_PROCESS`）。
1. 取卡串三层：环境里有 `GPU_CARDS_BY_PRIORITY`（外层已借好）就用它；没有而三个尝试变量 `TASK_SERVER_ATTEMPT_LABEL`、`TASK_SERVER_TASK`、`TASK_SERVER_ATTEMPT` 都在且非空，才向 GPU 服务借；两样都没有交回「没经服务给卡」，用例照现在的写法打 `NOT_RUN_NO_GPU_ADAPTER`，不自己枚举。
2. 借：找 `gpu-verification-server/control`（取 `TASK_SERVER_SOURCE_REPOSITORY` 下那一份，没有从当前目录往上逐层找），环境里设 `GPU_CARD_LEASE_BORROWER=<本进程号>:<启动刻>`（`/proc/<进程号>/stat` 第 22 项），起 `python3 <control> lease take`，标准输出去空白就是卡串，退非 0 就 panic、不重试、不退回 CPU。
3. 还：借到之后起一个 `sh -c 'while read -r _; do :; done; exec "$0" "$1" lease return "$2"'`，标准输入接管道、写端只在本进程手里，进程退出（含 panic、被杀）管道关，它起 `lease return <卡串>`；还的那一条在借的进程退出之后才跑，客户端只认环境里的借方变量。进程里只借一回（`OnceLock`）。

撞不撞：**不撞**，逐句对：
- `.claude/rules/verification.md:50` 的「卡只归 GPU 服务，任务不拿卡」说的是统一任务服务「不领卡、不分卡、不清场」，调用方不在命令外面套清场、不挑卡。上面三步里任务服务什么都不做：卡是产品测试进程自己向 GPU 服务的 `control lease` 借的，借与还只经 GPU 服务（`gpu-verification-server/README.md:48`：「判段之外要卡的进程（经统一任务服务照常交、自己开卡的测试与实验装置）在要开卡的那一刻向这一台的服务借这一台的全部卡，用完还」）。
- 任务不带选卡变量：`task-server/task_server_lib.py:2046` 的 `CARD_CHOICE_VARIABLES` 提交时拒带，产品测试读到的 `GPU_CARDS_BY_PRIORITY` 只可能是外层进程（比如 GPU 服务代跑）交下来的；这一点与今天的取卡模块一样。
- 任务服务读源文件头的领卡声明（`# task-server-card-lease:` 等）只对「命令位置上起的脚本」判（`task-server/README.md:186`）；产品测试源头那行 `// task-server-gpu-cards: N` 是 `.rs` 注释，不是脚本，不会被拒（服务也不再认它，它只被 `gpu_card_count_declared_in_the_source_header` 的单测读）。
- 一处要主 agent 记着：`.claude/rules/verification.md:50` 还写「崩溃判段之外用卡的测试交 GPU 服务代跑（接口待定）」，而 `gpu-verification-server/README.md:48` 的借卡接口已定；规则文字没跟上，不是产品测试的问题，由工具员改规则那一句（不在我的写范围）。
- 验证包与产品包各有一份「借卡协议」的副本（环境变量名、`control lease` 子命令、`/proc` 第 22 项）：GPU 服务改协议要两处改。产品那份的单测用假的 `control` 脚本（验证包有同样的做法，`gpu_card_configuration.rs:1492` 的 `fake_card_lease_client`）钉住命令行与环境。

**判定：成立**（最小一组能列出，约 830 代码行，加两个 dev-dependency 与一个 build-dependency；按需领卡不撞）。什么现象会推翻：提交时服务因产品测试的源头被拒（`card-declaration`）；`control lease take` 要的不止 `GPU_CARD_LEASE_BORROWER`（`gpu-verification-server/control:826` 起的 `lease` 只转给二进制、不读尝试变量，所以今天不成立）；用户读「任务不拿卡」为「任务里的任何进程都不许借」。

## 3. 实验包的接法

### 3.1 名字

`crates/singlefs-implementation-experiments`（实现驱动的实验装置）。不叫 `singlefs-experiment-apparatus`：仓里 `scripts/experiment-apparatus/` 是另一个工作区，装的是「不与实现共用代码、只用来独立验证」的装置（`.claude/rules/implementation-first.md` 第 4 条），两个名字只差一个词、`--bin` 一类命令与引用里极易看混。名字是主 agent 与用户的决定，我只给推荐与理由。`.claude/rules/verification.md:5` 的「四个包，全仓只用这四个名字称呼它们」要改成五个，加一行「**实验包**」。

### 3.2 `Cargo.toml`（推的；依赖按 22 份装置加 E202 的 `use` 实数，量的 `use` 计数见草稿目录）

```toml
[package]
name = "singlefs-implementation-experiments"
version = "0.1.0"
edition = "2021"
publish = false
description = "实验包：驱动实现（核心层、产品 GPU 包、验证包的脚手架）跑的实验装置二进制，不装测试"

[dependencies]
singlefs-core = { path = "../singlefs-core" }                 # 22 份用（量的）
singlefs-format = { path = "../singlefs-format" }             # 18 份
singlefs-checker = { path = "../singlefs-checker" }           # 8 份
singlefs-verification = { path = "../singlefs-verification", default-features = false }  # 21 份加 E202
singlefs-background-gpu = { path = "../singlefs-background-gpu" }   # E202；不能写可选：features_enabled_on_the_dependencies 不许依赖链上有可选的工作区依赖
singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }     # E197；验证包里只有它用，验证包那一行随之删
singlefs-unit-graph = { path = "../singlefs-unit-graph" }           # E158、E182 运行时算键
singlefs-device-library = { path = "../singlefs-device-library" }   # E158；验证包里只有它用，验证包那一行随之删
libc = "0.2"                                                        # E185 的线程钟、绑核系统调用
wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"] }  # E202 与 E185 的带卡单测，版本钉与产品包、验证包同一份

[build-dependencies]
singlefs-unit-graph = { path = "../singlefs-unit-graph" }

[features]
gpu = ["singlefs-verification/gpu"]                            # E171、E185 的 cfg(feature = "gpu") 读的是本包的特性
tests-on-a-real-gpu = ["singlefs-background-gpu/tests-on-a-real-gpu"]   # E202 内联真卡测试；任务服务、变异脚本按这个特性名认
```

要点（读码）：`singlefs-unit-graph` 的 `features_enabled_on_the_dependencies` 只认直接的普通依赖、带 `links`、依赖链上不许有可选的工作区包（`crates/singlefs-unit-graph/src/build_features.rs:165-170` 的文档），所以产品包不能写成可选；代价是实验包里任何一份装置编译都要编产品 GPU 包与 wgpu（测试包今天就是这样）。`libc` 两边都要：验证包 `tests/common_given_states_of_harness_scenario_images/mod.rs:666` 还用它（量的：`grep -rn 'libc::' src tests` 在验证包 `src/bin` 之外只命中这一处，所以验证包那一行留着、可改挂 `[dev-dependencies]`），实验包给 E185 另写一行。同理验证包里 `singlefs-device-library` 与 `singlefs-replay-prefix` 两行随装置搬走而删（量的：`grep -rn 'singlefs_device_library' src tests build.rs`、`singlefs_replay_prefix` 在 `src/bin` 之外都零命中，注释不算）。`e158_root_choice_repair` 里 `cfg(feature = "e158-r3-arm")` 引了一个验证包 `[features]` 没声明的特性（`grep` 命中 `e158…rs:8200-8232`），搬的时候照原样带、不在这一轮改。

### 3.3 `build.rs` 与单元键入口登记

- 今天参照一侧单元键在两处编进去：验证包 `build.rs` 按 `src/judging_units_entries.rs` 的 13 组（`grep -c 'group_name:'` 13）与观察者表算进 `OUT_DIR`；测试包 `build.rs` 按 `src/reference_units_entries.rs` 8 组与 `src/gpu_side_units_entries.rs` 31 组算进两张表（量的）。
- 搬完：产品包 `build.rs`（现 23 行，只经 `links` 交特性）并入测试包 `build.rs` 的算键那段（218 行，`crates/singlefs-background-gpu-tests/build.rs`），产品组 7 加 30 共 37 组；两张登记表搬到产品包 `tests/common_pairing_units_entries/`（不放产品 `src/`：`src/` 只放产品代码，数据表是测试的定义，`.claude/rules/directory-layout.md` 「测试的定义与输入跟着测试放」）。组名、入口 fn 路径的头一段由 `singlefs_background_gpu_tests::` 改 `singlefs_background_gpu::`，只用来找包、不进键（`.claude/kb/decisions/13-验证路线.md:436`）。
- 实验包自己一份 `build.rs` 与 `build_inputs/units_entries.rs`（同一种行），装 3 组：E185 的 `e185_cpu_reference_crcs`（今天登在验证包 `judging_units_entries.rs:146`，目标 `bin:e185_gc_batch_wall_clock_split`）、E202 参照侧 1 组、E202 GPU 侧 1 组。读键用验证包现成的 `judging_units_key_of_the_group_in_the_table_text`（`crates/singlefs-verification/src/crash_identity.rs:852`，它的文档本来就写「别的包的构建脚本照同一套…写下的键表」）；E185 今天调的是 `judging_units_key(&JudgingUnits::PairingComparison{…})` 查验证包自己编进去的表（`e185_gc_batch_wall_clock_split.rs:803-811`），搬后改成读实验包自己的表。
- **验证包库里要连带改的三处**（读码，不改会编不过或红）：`judging_units_entries.rs:144-153` 删 E185 那组（目标 `bin:` 找不到，验证包 `build.rs` 停下）、`:536-538` 的 E161 观察者行同理；`tests/crash_points_observer_registration_is_complete.rs:193` 读 `src/bin/<名>.rs` 的那一支随之无行可判。E161 的观察者还有个更大的问题，见 4.3。

### 3.4 它在 ⑧ 下是什么

⑧ 判两件事：「两边的传递闭包（dev-dependencies 也算）里都没有对方」（`.claude/gate.d/checker-independence-and-sync.sh:38`）与「两边闭包的交集减去允许共用的必须为空」（`.claude/gate.d/checker-independence-and-sync.sh:39`）。实验包依赖两边，但**没有任何包依赖它**，所以两边的闭包里都没有它：⑧ 不变、不失效。间接路径只有一条：产品包或验证包（含 dev 依赖）依赖了实验包——这时那一边的闭包里就带进了另一边，⑧ 当场红（dev 依赖也算，`:38` 原话）。守住这条要新加一道检查「角色是实验装置的包没有任何 `crates/*/Cargo.toml`（含 dev）依赖它」，写进第 9 格的检查一。今天 cargo 允许 dev 依赖成环，所以这条不能只靠 cargo 拦。

**判定：成立**。什么现象会推翻：`cargo metadata` 里出现验证包或产品包到实验包的任何一条边；`features_enabled_on_the_dependencies` 对实验包报「声明了特性的包取不到」（那就说明哪个直接依赖缺 `links`）；E202 与 E185 在同一个包里使「不开 GPU 的装置也编 wgpu」成了用户不能接受的代价（那就得拆成两个实验包，本轮不推荐）。

## 4. 26 个装置怎么分

### 4.1 判据（先写死）

**留验证包**：二进制是验证流程自己起的工具——出现在下面任一处的起命令位置：门禁格（`.claude/gate.d/*.sh`，`fixtures/` 除外）、崩溃入口脚本（`scripts/crash/`）、合入后验证（`scripts/gate/merge-and-verify.sh`）、真设备脚本（`scripts/devices/`），或出现在 `.claude/gate.d/stage-inputs.tsv` 行首是 `<门禁>.sh:<格>` 的行里（经脚本间接被门禁格起）。**进实验包**：其余的，即只被实验页、`replay.sh`、`stage-inputs.tsv` 里 `E<号>` 起头的准入行、`scripts/experiment-apparatus/` 里的装置驱动点名的。
- 不算「起」的：`crates/mutations.tsv` 里 `cargo test … --bin <名>` 的行（那是装置二进制自己的内联单测被变异门禁跑，不是起装置）；只在文档、`path-moves.tsv`、`preflight-exclude`、`data-name-lint-exclude` 一类登记表里出现的名字。
- 辅判：二进制搬走会让验证包库里的登记断掉的（观察者登记、判法单元入口登记），连登记一起改（3.3、4.3）。
- 判据的核法（命令写进「要主 agent 跑的命令」）：`git grep -nE -- '--bin +<名>' -- .claude/gate.d/*.sh scripts/crash scripts/gate/merge-and-verify.sh scripts/devices` 对 26 个名字逐个跑；今天（量的）命中的只有 `crash_library_reader`、`new_pool_file_creation_on_device`、`new_pool_file_creation_device_log_check` 三份，加上经 `stage-inputs.tsv:39` 被门禁格 `formal-verification.sh:pinned-red-examples-replay` 起的 `e165_replay_bounded_enumeration`（`scripts/experiment-apparatus/e165-pin-red-examples.py` 起它）。验证包 `tests/` 没有一份用 `env!("CARGO_BIN_EXE_…")` 起装置（`grep -rn CARGO_BIN_EXE crates` 只命中单元图自己的 `unit-graph` 命令行）。

### 4.2 逐份表（谁点名它读 `git grep -l` 与上面的 `--bin` 命令；每份还被 `crates/mutations.tsv`、`gate.d/path-moves.tsv`、`.claude/preflight-exclude` 一类登记表点名，搬时照 `.claude/rules/path-moves.md` 改，不逐份列）

| # | 装置 | 谁点名它（起它的 / 页） | 去处 |
|---|---|---|---|
| 1 | `crash_library_reader` | `scripts/crash/crash-amplification.sh`、`crash-point-replay.sh`、`crash-library-totals.sh`、`lib_crash_point_replay_items.py`、`scripts/gate/merge-and-verify.sh`、`verdict-library-scripts.tsv` | 留验证包 |
| 2 | `new_pool_file_creation_on_device` | 门禁 `qemu-device-streams.sh`、`scripts/devices/e152-run.sh:104`、`e152-stage-root.sh:21`、E152 的 `e152_file_system_benchmark.rs`；验证包库里 `on_device_modes.rs`、`fault_injection.rs:9` 的注释 | 留验证包 |
| 3 | `new_pool_file_creation_device_log_check` | 门禁 `qemu-device-streams.sh:1036` | 留验证包 |
| 4 | `e165_replay_bounded_enumeration` | 门禁格 `formal-verification.sh:pinned-red-examples-replay`（`stage-inputs.tsv:39`）；E165 页 2、`scripts/experiment-apparatus` 5 份 | 留验证包（门禁起的工具；同时是 E165 的装置，页里的命令不用改包名） |
| 5 | `deletion_write_dump` | E172 装置 `e172_deletion_write_dry_run.rs`、`replay.sh`、`stage-inputs.tsv:48`（准入行）；kb 页 2 | 实验包 |
| 6 | `e142_new_pool_file_creation_write_dump` | E142 页、`replay.sh`、`stage-inputs.tsv:47`（准入行）；验证包库 `new_pool_file_creation_regions.rs` 注释 | 实验包 |
| 7 | `e142_new_pool_file_creation_write_dump_one_device` | E142 页 | 实验包 |
| 8 | `new_pool_file_creation_region_bytes` | E142 页 2、`replay.sh`；验证包测试 `new_pool_file_creation_write_regions.rs:1`、`:34` 的注释 | 实验包 |
| 9 | `e156_allocation_basis_counts` | E156 页 3、装置目录 | 实验包 |
| 10 | `e158_root_choice_repair` | E158 页 5、装置目录、`quoted-product-unreplayable.tsv`（表，不是起它） | 实验包 |
| 11 | `e161_crash_state_dedup_and_time_split` | E161 页 2、装置目录；库里 `judging_units_entries.rs:536-538`、`crash_point_replay_provenance.rs:480`、`crash-entries.tsv:23` 点名它的观察者与流名 | 实验包，**带库内登记改动，见 4.3** |
| 12 | `e168_delete_ring_window_borrow` | E168 页 3、装置目录 | 实验包 |
| 13 | `e170_specification_executor_export` | E170 页 3、装置目录 5 份 | 实验包 |
| 14 | `e171_three_oracle_collusion` | E171 页 4、装置目录 49 份（行驱动 python、`rows_orchestrator.py` 经 `task_server_lib.py` 分片登记） | 实验包 |
| 15 | `e171_three_oracle_collusion_after_the_oracle_fixes` | E171 修后页 3、装置目录 37 份 | 实验包 |
| 16 | `e173_checkpoint_cost_against_empty_publish_cost` | E173 页、装置目录 2 | 实验包 |
| 17 | `e175_product_path_inode_watermark_regression` | E175 页 2、`dead-path-kept.tsv`（旧路径登记，搬后再登一行） | 实验包 |
| 18 | `e180_admission_reading_versus_writable_bytes` | E180 页 2 | 实验包 |
| 19 | `e181_landing_check_fixed_point_and_mount_lower_bound` | E181 页、装置目录 2 | 实验包 |
| 20 | `e182_unreadable_abandoned_root_forks` | E182 页 2、装置目录 3 | 实验包 |
| 21 | `e183_capacity_read_cache_reserve_and_sweep_costs` | E183 页 2、`stage-inputs.tsv:49`（准入行）、`e183-capacity-cells.sh` | 实验包 |
| 22 | `e184_regular_layer_cost_counts` | E184 页 2、装置目录；它经 `#[path]` 引同目录 `…write_dump/mod.rs` | 实验包 |
| 23 | `e184_regular_layer_cost_counts_write_dump` | E184 页 2；与 22 共用 `…write_dump/mod.rs`（`e184_regular_layer_cost_counts.rs:52`、`…write_dump.rs:27`） | 实验包（必须与 22 同进，同目录共用模块不拆） |
| 24 | `e185_gc_batch_wall_clock_split` | E185 页 3、装置目录 3；库里 `judging_units_entries.rs:146` 的参照组 | 实验包，**带库内登记改动（3.3）** |
| 25 | `e197_scan_journal_glue_memory_by_device_count` | E197 页、装置目录 2、`break-switch-replay-files.tsv` | 实验包 |
| 26 | `e198_instance_table_pointer_birth_skip_premise` | E198 页 2、装置目录、验证包测试 `common_number_acquisition_skip_histories/mod.rs` 注释 | 实验包 |

留 4 份、进 22 份。（背景材料写「不带实验号的 5 份」的另两份 `deletion_write_dump`、`new_pool_file_creation_region_bytes` 属于实验包；留下的 4 份里只有 `e165_replay_bounded_enumeration` 带实验号。）

### 4.3 判据套完之后落下来的三件事

1. **E161 的观察者键**：E161 的 S3 观察者 `e161_crates_comparison` 的判法单元键由验证包 `build.rs` 按 `OBSERVER_ENTRIES` 编进 `compiled_judging_units_keys()`（一个 `OnceLock` 静态表，`crates/singlefs-verification/src/crash_identity.rs:636`），崩溃枚举引擎只查这一张表（`judging_units_key(&JudgingUnits::ObserverCode{…})`）。装置搬走、登记行删掉之后，查不到这一组，引擎照「拒算：照判、不入库」处理（`crash_identity.rs:574` 的 `NotComputedAtTheBuild`），E161 的 S3 观察者不再按键复用。三条路：(a) 接受不复用——要读 E161 登记里 S3 是否靠复用才推得出对结果有没有影响，我**推不出**；(b) 给引擎加注入口：`ObserverOnTheNodes` 再带一个可选的单元键（`crash_state_enumeration.rs:1989` 一族、`crash_amplification.rs:29436` 一处构造点，`git grep -l 'CrashPointReplayObserver {' -- crates` 27 份文件共 39 处构造，所以不能给这个结构体加必填字段，要走 `ObserverOnTheNodes` 内部结构或新构造函数），实验包自己的 `build.rs` 算这一组、装置传进去；(c) E161 留验证包（违反「26 个一起搬」的弹窗定案，要问用户）。我推荐 (b)，但它是对验证包库代码的改动、要走三方。
2. **E184 的逐字抄与自检**：E184 有 11 处 `COPIED-BEGIN` 逐字抄验证包测试文件的代码，并 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/crash_enumeration_*_stream.rs"))` 读原文做逐字比对（`e184_regular_layer_cost_counts.rs:2919`、`:4422-4434`，量的 5 处）。搬完这 5 处变成跨包 `.rs` 的 `include_str!`，⑨ 判红（第 9 格）；改成运行时从仓根 `std::fs::read_to_string`（路径照 `env!("CARGO_MANIFEST_DIR")` 往上两层），不再是编译期引用，逐字比对照旧。这是实验装置读验证包测试文件当数据，不是共用代码，不撞前提第 6 条。
3. **`CARGO_MANIFEST_DIR` 一族**：装置里 `src/bin/<自己>.rs`、`../../.claude/singlefs-ai-sop/scripts/preflight.py`、`../../.claude/heavy-thread-default` 的写法在 `crates/<包>/` 同一层深度下不变（量的：e161、e173、e181、e182、e184、e185、e197、e198 的命中行）；e197 读 `../singlefs-core/src/mount.rs`（`e197…rs:2947-2950`）、e182 读 `crates/singlefs-verification/src/crash.rs` 与 `fault_injection.rs`（`e182…rs:2720-2724`）是运行时读文件，不是 `include`，不被 ⑨ 管，搬后仍对（同在 `crates/` 下），但它们读的验证包源码路径要核一遍没有跟着别的改动挪。

**判定：成立**（判据可写死、可命令核、逐份有去处）；其中 E161 的去处**推不出**，要主 agent 在 4.3 的 (a)(b)(c) 里选。什么现象会推翻：核法命令命中了表里没列的第五份被门禁起的二进制；`task-server`、变异门禁或 `merge-and-verify.sh` 经别的写法（脚本变量拼的 `--bin`）起了「进实验包」的某一份（这类写法 `git grep` 看不见，要读 `.claude/hooks/project-code-guard.sh` 认的「认不出起什么的写法」那一类）。

## 5. 装置用到的验证包 `tests/` 共用模块从哪来

### 5.1 6 份装置真要的（量的）

- e161、e170、e171、e171 修后版、e185 五份只用 `tests/common/mod.rs`（1416 行）里的：`build_pool`、`format_pool`、`BuiltPool`、`FormattedPool`、`Recorded`、`geometry`、`file_content`、`IMAGE_BYTES`、`FILE_BYTES`、`FIXED_WRITE_TIME_SECONDS`，加两个再导出 `parameters`、`E142_FILESYSTEM_IDENTIFIER`（后两个本来就是验证包 `src/known_bad_images.rs` 的东西，`tests/common/mod.rs:31-35` 只是 `pub use`）。从这些名字按标识符往下收的条目闭包：17 条、244 行（代码 231 行，`discrete_closure.py` 近似、不含 impl 块，实际再多几十行），只用验证包 `src/` 的公开接口（`RecordingBlockDevice`、`memory_pool`、`temporary_files`、`known_bad_images`）。
- e184 另引三份：`common_crash_point_observers`（只用 `multi_record_publish`、`MultiRecordPublishBoundaryInputs`，闭包代码 66 行，全模块 942 行）、`common_crash_points`（只用 `content_needing`、`sequential_write`，25 行，全模块 890 行）、`common_snapshot_prefix`（`prepare_the_snapshot_crash_flow` 一族，497 行，全模块 854 行）；加上上一条的 231 行，E184 共约 820 代码行。
- 被引的面：`mod common;` 在验证包 340 个测试目标里（量的）；其中点名了上面 12 个名字之一的 233 个（`import` 与 `common::<名>` 的文本匹配，上界）；35 个 `common_*` 模块里 23 个引 `crate::common` 或 `super::common`。三份 e184 专用模块各被 27、45、22 个测试目标 `mod` 进来。

### 5.2 候选甲：挪进验证包 `src/` 的公开模块

做法：新建 `crates/singlefs-verification/src/<脚手架模块>.rs`，把 5.1 的 17 条（及 e184 那三份的闭包）挪进去；`tests/common/mod.rs` 里原处改成 `pub use singlefs_verification::<模块>::{…}`；装置与实验包 `use` 它。
- 前提第 6 条：不撞（实验包是 `use` 验证包的公开模块，没有跨包 `#[path]`）。
- `.claude/rules/verification.md` 「定义与名字」：不撞，验证包那一行本来写「它们用的脚手架库」（`.claude/rules/verification.md:9`），而且有先例（`known_bad_images::parameters_of_the_common_test_pool`，`known_bad_images.rs:35`，`tests/common/mod.rs` 已经这样再导出）。
- 单元键：**会换**。键是闭包里每个单元（种类，去注释指纹）的多重集加入口链加 `use` 与 `mod` 头（`crates/singlefs-unit-graph/src/closure.rs:3423` 的 `reuse_key`、`:3491` 的 `entry_key_lines`）：函数体逐字不变则函数单元的指纹不变，但原处的 `use singlefs_core::…` 与 `mod common;` 头单元换成新模块里写法不同的 `use`，加一条新的 `pub use`，用到这些名字的测试闭包里的单元集合就变了。上界 233 个测试目标加 23 个共用模块（上面的名字匹配），实际换几个、涉及多少崩溃状态与变异行，**推不出**，要量（命令见末节：`crash-point-replay.sh --list-items --with-keys`、`crates-mutation-replay.sh --list-items --with-keys` 搬前搬后各一趟）。D13（验证路线） 已定项 17 写明键由函数单元、工具链、依赖版本组成、路径名字不进键（`.claude/kb/decisions/13-验证路线.md:436`），所以挪动本身不是理由，文本变了才是。
- 结论：**成立，但代价是上界 233 个测试目标的键重判**。

### 5.3 候选乙：实验包自己写一份（各写各的）

做法：实验包 `src/` 下一个装置共用的私有模块（不挂 `tests/`），装 5.1 的闭包：5 份装置约 231 行，e184 再加约 590 行。
- 前提第 6 条、「定义与名字」：不撞。
- 单元键：验证包 340 个测试目标与 `tests/common*` 一个字不动，键不变（量的依据：不改这些文件）。
- 代价与风险：这是**抄**不是各写各的——231 加 590 行与验证包的 `tests/` 逐字相同，验证包改了、副本不跟。守法：照 E184 已有的做法钉住——E184 的 `selfcheck` 里有 `copied_text_verbatim`（`e184_regular_layer_cost_counts.rs:27` 起的头注释）把抄来的每段去缩进后在原测试文件里逐字查一次，副本不同于原文就红；实验包里加一条内联单测，运行时从仓根读 `crates/singlefs-verification/tests/common/mod.rs` 等四份，逐条目比对（运行时读文件，不是编译期引用，不被第 9 格的 ⑨ 管）。
- 结论：**成立**（键不动、不撞条款），但副本要靠上面那条比对钉。

### 5.4 候选丙：别的

- 丙一：装置改用验证包 `src/` 里已有的公开接口造池，不再要 `common`——`grep -rnE 'pub fn (build_pool|format_pool)' src` 零命中，验证包 `src/` 没有造格式化池的函数，等于要新写 17 条，和甲同一件事。**不成立**。
- 丙二：实验包写成库、验证包的测试反过来依赖它——实验包依赖验证包，会成环（dev 依赖成环 cargo 放行，但 ⑧ 的传递闭包会红，3.4）。**不成立**。
- 丙三：e184 及其 `write_dump` 留验证包（与 4.1 判据冲突，且违反「26 个一起搬」的弹窗定案）。**推不出**，要问用户。

### 5.5 推荐与判定

推荐乙（加比对钉）：不换 340 个测试目标的键。甲留作以后——等哪一天本来就要整体重判验证包键的窗口再并（那时副本删掉、一处两用）。两者都不撞前提第 6 条。

**判定：成立**（乙）；甲成立但换键的量推不出。什么现象会推翻：比对单测里副本与原文出现合法的分歧（说明副本已经不是抄、各写各的，要回头改 4.1 的口径）；乙的 820 行副本里有一处依赖验证包 `tests/` 里没挪出的私有函数（闭包近似漏了 impl 块——要编一遍才知道）。

## 6. E202 的取卡与对拍那一层从哪来

量的：E202 从测试包两个共用模块用到的名字 15 个——取卡模块 6 个（`GpusThisProcessIsOn`、`gpus_this_process_is_on_judged_against_the_handed_over`、`MEMORY_LIMIT_OF_EACH_PARTICIPATING_GPU`、`handed_over_gpus_or_report_not_run`、`note_the_cards_leased_on_demand_for_this_process`、`VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER`），对拍模块 9 个（`append_count`、`append_number`、`PairingInputBeingRead`、`judgement_of_the_assertions_on_the_cards`、`ComparisonOnTheCards`、`KeysOfAPairingOnTheCards`、`PairingOnTheCardsOutcome`、`pairing_concluded_on_the_cards_in_the_library`、`pairing_concluded_on_the_cards_through_the_pairing_library`）。按标识符往下收：取卡 34 条、449 代码行（模块 860 行）；对拍 15 条、235 代码行（不含 `PairingInputBeingRead` 的 impl，约再加 50）。借卡本身 E202 今天就直接用验证包：`use singlefs_verification::gpu_card_configuration::{cards_handed_over_or_leased_on_demand, CardLeaseChannelOfTheCaller}`（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:152-154`）。

- **取卡匹配层**（`nvidia-smi` 的序号与总线地址对上 wgpu 的适配器、卡数对账）：验证包没有现成的可改用——它的卡池 `gpu_unit_checks::gpu_test_start` 直接用 wgpu，不经产品 GPU 包的 `adapter_inventory`、`BackgroundGpu` 类型；而 E202 开的是产品包的后台 GPU，必须按产品类型对卡。所以各写各的：E202 自己一份，`git mv` 的原模块里 E202 用到的闭包 449 行放进 `src/bin/e202_gpu_cpu_gc_comparison/` 下的私有子模块（`mod` 默认路径，不用 `#[path]`）。其中 `pci_bus_identifier_without_the_domain` 那 4 行不带，E202 直接 `use singlefs_verification::gpu_memory_ledger::pci_bus_identifier_without_the_domain`（第 7 格）。
- **对拍胶水层**：E202 可以继续建在验证包 `crash_library` 的对拍函数上（它今天就是这么用的；现在的胶水 `common_pairing_input_through_the_library` 本来就写在验证包接口之上）。所以 E202 的胶水 = 今天那份模块里 E202 用到的闭包（约 285 行）原样 `git mv` 进同一个私有子模块目录，**逻辑一行不改**，只是不再被产品测试共用；产品测试那份是第 2 格新写的（建在产品自己的存取代码上）。
- 「是不是两份」：**是两份**，取卡两份（产品 `tests/common_discrete_gpus_of_this_host` 与 E202 私有子模块）、胶水两份（产品、E202）。各自的测试：产品那份由产品 `tests/real_gpu_tests_open_only_the_gpus_the_task_server_handed_over.rs`（378 行）与 `…create_one_vulkan_instance_per_process…`（287 行）测；E202 那份由 E202 内联 `mod tests` 里已有的几条（`e202_gpu_cpu_gc_comparison.rs:13488-13507` 用到 `gpus_this_process_is_on_judged_against_the_handed_over`）加第 5 格同款的运行时逐条比对（与产品那份比）钉。
- 有没有绕回测试包的形状：没有。测试包的形状是「一个包装一批 `tests/` 目标与共用模块、库是空的、靠 `#[path]` 把共用模块编进装置」；E202 在实验包里只装二进制，它的子模块是自己的、没有 `tests/`、没有跨包 `#[path]`。
- 想去掉两份只有两条路：模块放进产品 `src/`（成为 `#[doc(hidden)]` 的测试支撑，产品包放测试协议代码，违反 M1「产品包不放测试」）或另开测试支撑包（违反 M1「不为测试另开包」），我都不推荐。

**判定：成立**（各写各的，E202 不依赖产品测试的任何东西）；两份重复约 735 行是已知代价。什么现象会推翻：E202 那份取卡闭包里有依赖产品 `tests/` 里没挪的东西（编一遍才知道）；用户把两份重复判为不可接受（那就要走上面两条路之一，要弹窗）。

## 7. `pci_bus_identifier_without_the_domain` 两份

事实（量的）：产品测试那份 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:206-209`，验证包那份 `crates/singlefs-verification/src/gpu_memory_ledger.rs:780-783`，函数体三行逐字相同；产品那份第 204 行注释写「本包测试不依赖 验证包，自己留一份」，今天这句不对（测试包 `Cargo.toml` 直接依赖验证包）。第三份在 `scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:488`，属独立验证的另一个工作区，不算。产品包 `src/` 没有等价函数（`grep -n 'to_ascii_lowercase\|fn .*pci_bus' src/*.rs`），产品 lib 的卡标识是 wgpu 报的原样（`adapter_inventory.rs:89`）。

- 产品测试那份：**合法的各写各的**。拆包之后它在产品包 `tests/` 里，产品包不依赖验证包（⑧），前提第 2 条许「代码可以不同」，也没要求必须不同；它是测试协议代码（`nvidia-smi` 写法对 wgpu 写法），不是产品逻辑，所以不进产品 `src/`。
- 实验包里 E202 那份取卡闭包：不带这 4 行，直接 `use singlefs_verification::gpu_memory_ledger::pci_bus_identifier_without_the_domain`（实验包依赖验证包，正常）。所以拆完全仓是两份函数体：产品 `tests/` 一份、验证包 `src/` 一份，没有第三份。
- 不另设「一处」：另设就是新开支撑包或进产品 `src/`，理由同第 6 格。
- 第 204 行的注释改成（只写现状，不带「原先」「后来」）：

```
/// PCI 总线地址只留 `总线:设备.功能` 那一段、小写：`nvidia-smi` 报的 `00000000:01:00.0` 与 wgpu 报的 `0000:01:00.0` 认成同一张卡。
/// 产品 GPU 包的测试不依赖验证包（门禁 checker-independence-and-sync 的 ⑧ 判两边的传递闭包里都没有对方），所以这个函数在这个测试模块里自成一份；
/// 验证包 `gpu_memory_ledger` 里有函数体相同的一份，两份各有单测钉写法。
```

**判定：成立**。什么现象会推翻：两份函数体分歧（一份改了另一份没改，且 `nvidia-smi` 与 wgpu 的对账因此在一边错）——要钉的话可以在第 1 格的黄金表机制里多加一条两份函数的同例输出比对，今天没有。

## 8. 单元键与现判的量

### 8.1 哪几组键会变（量的组数；变不变是推的，要 8.3 的命令量）

| 组 | 组数（量的） | 搬后键 | 依据 | 要现判的 |
|---|---|---|---|---|
| 产品 GPU 侧（登记表 `gpu_side_units_entries.rs`，不含 E202） | 30（`grep -c 'group_name:'` 31 减 E202 的 1） | **变** | 每条持卡判入口都调 `judgement_of_the_assertions_on_the_cards`，它与 `PairingJudged`、`PairingAgreement` 的出处从验证包换成产品自己的两份模块，用到它们的单元的文本与 `use` 头都变（`common_pairing_input_through_the_library/mod.rs:16-32`） | 30 条持卡判（每条一次开卡运行，不是崩溃态枚举），键不同于库里旧的，库里旧结论不再被读到（推的；R1 的 10⁹ 量级是拆包方案甲的数，不用） |
| E202 GPU 侧 | 1 | **变** | 胶水 `git mv` 进 E202 私有子模块、`use` 与 `mod` 头换了；表也换成实验包自己的 | 1 条 |
| 产品参照侧（`reference_units_entries.rs`，不含 E202） | 7 | **预计不变** | 入口是 `reference_bytes_of_*`，闭包是产品 CPU 参照与核心层函数加 `append_*` 编码；只要 `append_*` 与这些函数的文本不改、模块名 `common_pairing_input_through_the_library` 不改（本报告第 2 格保留这个名字），它们的单元指纹与 `use` 不变 | 0（若变，参照一侧变使录入键与结论键都变，每组再各多一条） |
| E202 参照侧、E185 参照侧 | 1、1 | E202 预计不变；E185 预计不变 | E202 的 `reference_bytes_of_the_apparatus_count…` 与 E185 的 `cpu_reference_of_*` 不碰被换的 `use`；表换成实验包的 | 0 |
| 录入键（产品每条走录入段的对拍用例一条记录） | 约 31 条用例加 `pairing_input_read_back…` 那 1 条（量的用例数：GPU 侧登记 31 组） | **变** | 产品一侧新开自己的录入组（第 2 格、第 3 格），录入单元指纹换成产品自己的（本来就按定义不同，1.2） | 31 条 CPU 重录，不开卡；`0x1C` 行内容定址，同一份输入不重存 |
| 验证包 `judging_units_entries.rs` 13 组（删 E185 一组后 12 组）与观察者表 | 13 | **预计不变**，但有一个要量的风险 | 文件头自己写：「这份表里的常量会这样被并进去、表里的文字随它进键」（`crates/singlefs-verification/src/judging_units_entries.rs:6`，指 GPU 判器那类组的闭包经 WGSL 外名着落把这份表里的常量并进来）。删两行（E185、E161）改了这些常量的文本，GPU 判器组的键可能连带变——变了则验证包全部镜像判定与给定状态判定的块都作废（D13（验证路线） 已定项 17「射程」那一段写的版本口径），**推不出**，必须先量 | 不变则 0；变则全库——这一条是整个方案里最大的未知数 |
| E161 观察者 `e161_crates_comparison` | 1 | 变（观察者闭包含 `common::` 引用，文本变） | 4.3 | 只影响 E161 自己的 S3 |
| 验证包崩溃枚举用例、按块判定的 340 个测试目标的键 | — | 乙：**不变**；甲：上界 233 目标 | 第 5 格 | 乙 0 |

### 8.2 `crates/mutations.tsv` 与判法指纹

- 表现有 8312 条数据行（量的：非 `#` 起头的行，每行 6 段）。要改文本的行：文件列在测试包里的 189 行（`src/` 148、`tests/` 40、`build.rs` 1），cargo 参数列写着 `-p singlefs-background-gpu-tests` 的 243 行，两者并集 244 行；文件列在 `crates/singlefs-verification/src/bin/` 下、装置要搬的 815 行（该目录下共 986 行，留验证包的 4 份占 171 行），另有 7 行文件在别处、参数里点名要搬的 `--bin`；合计约 1066 行改文本（路径与 `-p`）。变异原文与替换文不变，行的键不含路径（`.claude/kb/decisions/13-验证路线.md:436`）；基线键含「点名测试所在的包与测试目标」（`scripts/mutation/crates-mutation-rows.py:202`），所以这 1066 行的基线要重做（只跑未变异的测试，比变异便宜），变异本身照键复用。
- **判法单元指纹**：`scripts/mutation/crates-mutation-rows.py:391` 的文档串点名测试包的路径 `crates/singlefs-background-gpu-tests/tests/compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs:400-438`，第一轮判决已采「文档串进判法单元指纹，改它要与别的换键改动放在同一窗口」（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:38`）。改这一行换掉判法指纹，8312 行全部要现判。两条路：(i) 改它，并把所有换键的改动收进同一个窗口一次落完；(ii) 不改文档串，把旧路径登进 `.claude/gate.d/dead-path-kept.tsv`（`.claude/rules/path-moves.md`「怎么做」第 5 步留旧路径的出路，第一轮判决出口三已改成允许）。我推荐 (ii)：为一句引用路径重判 8312 行不值；文档串里那个路径是个行号区间的例子，路径本身搬到产品包后只差一个包名。这是主 agent 与用户的取舍，我只列代价。

### 8.3 要量的命令（写进「要主 agent 跑的命令」；以下是形态）

- 变异表：`bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys`，搬前搬后各一趟；读「键对不上（要现判）的条数」那一行与逐项 `ITEM_PLAN … reusable=`（`.claude/gate.d/crates-mutation-replay.sh:23` 的用法行）。
- 崩溃判定库：`bash scripts/crash/crash-point-replay.sh --list-items --with-keys`，搬前搬后各一趟；读末尾「键」行（块键对不上几块、与两种判器的判法单元指纹）与 `ITEM_PLAN to_judge=`（`scripts/crash/crash-point-replay.sh:25` 的用法行）；GPU 判器组的判法单元指纹两趟逐字对（判 8.1 倒数第三行的风险）。
- 产品与实验包的对拍键：编完后读 `OUT_DIR` 里 `reference_units_keys.tsv`、`gpu_side_units_keys.tsv` 的 `group` 行，搬前（测试包）搬后（产品包、实验包）逐组比十六进制键；`unit-graph key` 命令行可单组核（`crates/singlefs-unit-graph/src/bin/unit_graph.rs` 文件头的 `key` 子命令，要带 `--judgment-kind product-behavior`、`--no-default-features`、`--cfg 'feature="tests-on-a-real-gpu"'` 与 `--lock-file`，才与构建脚本写进去的键逐字相同）。

**判定：成立**（哪几组会变、各几组能列；30 加 1 条持卡判、约 31 条 CPU 重录是推的）；「验证包 GPU 判器组的键换不换」与「混合窗口里 8312 行要不要重判」**推不出**，各由 8.3 的命令与 8.2 的取舍定。什么现象会推翻：产品参照侧 7 组里有键变了（那就是 `append_*` 或模块名被改动了）；GPU 判器组的键变了（那就要把删登记行的改动与别的换键改动拆开评估）。

## 9. 措施判据定稿

### 9.1 角色与五条检查（第一轮正推报告第 7 节的五行，第一轮判决「二、判决」第 7 条采用）

四种角色，写进新登记表 `.claude/gate.d/crate-roles.tsv`（包名、角色、目录、理由，制表符隔开）：**产品**（核心层、产品 GPU 包这类随产品发的代码）、**验证**（验证包、池级 checker、RefFS）、**测试支撑**（单元图、判定库口子这类只给测试与装置用的库）、**实验装置**（实验包）。

| 检查 | 判据（按新角色重写） | 放哪 |
|---|---|---|
| 一 位置标准（原缺口 1） | 逐包读登记表角色，对 `crates/<包>/`：①**产品**：`src/bin/` 与 `[[bin]]` 里没有名字形如 `e<数字>_` 的二进制；`[dev-dependencies]`、`[build-dependencies]` 里没有角色是验证或实验装置的包。②**验证**：`src/bin/` 下每个二进制都被验证流程起过（判据 L，4.1：门禁格、`scripts/crash/`、`merge-and-verify.sh`、`scripts/devices/` 里的 `--bin <名>`，或 `stage-inputs.tsv` 行首是 `<门禁>.sh:<格>` 的行点名），一个没被起的判红（装置去实验包）；今天 4 份都过（`gate_starts_bin.py` 量的 stay 4）。③**实验装置**：没有 `[lib]`、没有 `tests/` 下的 `.rs`；每个二进制名在实验页、`replay.sh` 或 `stage-inputs.tsv` 的 `E<号>` 行里至少点名一次（防孤儿装置落进来）；**没有任何 `crates/*/Cargo.toml`（含 dev、build）依赖它**（3.4，守住 ⑧ 的间接路径）。④**测试支撑**：被不少于 2 个包依赖（数依赖者时算 `crates/*/Cargo.toml` 加 `scripts/experiment-apparatus/Cargo.toml`，否则 `singlefs-device-library` 搬完只有实验包一个依赖者，会误红；单元图今天验证包加产品包加实验包三个）。⑤空库测试包：角色不是测试支撑、`src/lib.rs` 去掉注释与 `#![…]` 之后没有条目而 `tests/` 下有测试目标，判红（今天的 `crates/singlefs-background-gpu-tests` 就是这个形状：`src/lib.rs` 4 行全是注释与 `#![deny(unsafe_code)]`） | 门禁 code-tooling 新格 `crate-roles`，判法 `scripts/gate/crate-roles-check.py`；规则写进 `.claude/rules/directory-layout.md` 新一节（角色、位置标准、判据 L） |
| 二 登记覆盖（原缺口 2） | ①`crates/*/Cargo.toml` 的每个包在表里恰一行；②登记的目录存在；③角色是四种之一；④成功行报「查了 N 个包」，N 为 0 或与 `crates/` 下 `Cargo.toml` 数对不上判红；⑤同名两行判红 | 同一格的第二个子检查 |
| 三 跨包引源码（原缺口 3，⑨） | 扫 `crates/**/*.rs`（含 `build.rs`、`tests/`、`src/bin/`）里 `#[path = "<字面>"]`、`include!`、`include_str!`、`include_bytes!`，字面（含 `concat!(env!("CARGO_MANIFEST_DIR"), "<字面>")` 拼得出的）按所在文件目录解析：落在**另一个** `crates/<包>/` 里且后缀是 `.rs` 或 `.wgsl`，判红；后缀是数据（`.toml`、`.md`、`.tsv`）不红、只在成功行计数；落在 `crates/` 之外（`.claude/heavy-thread-default`、`litmus/`）不判；`concat!(env!("OUT_DIR"), …)` 不判；运行时 `std::fs` 读文件不判（读不出静态落点，要写进成功行「运行时读 N 处，未判」）。豁免表 `.claude/gate.d/crate-includes-exempt.tsv`（文件、行、理由，理由至少 8 个字） | `.claude/gate.d/checker-independence-and-sync.sh` 的 checker-implementation-disjoint 格在 ⑧ 后加 ⑨；同一份 Python 里已有扫两边源码的 `package_sources`；并把 `:46` 那句「⑧ 罩不到的…不认」改成指向 ⑨ |
| 四 代码轮检查单（原缺口 4） | 八格改九格，第九格「放的位置」：对这一轮每个新增或搬动的 `crates/` 文件，写它所在包的角色与该角色的位置标准；新增或搬动的二进制写判据 L 的答案（被哪个验证流程起 → 验证包；没有 → 实验包）；弱判据：这一格非空且点名至少一个 `crates/<包>/` 路径与该包在登记表里的角色 | `.claude/rules/implementation-workflow.md`「代码轮正文的检查单」与门禁 doc-process-records 的 `code-round-checklist` 格同一轮改 |
| 五 弹窗放置类问题（原缺口 5） | 判据不变：放置类问题的 `question` 里引 `.claude/rules/directory-layout.md` 加小节名；每个选项以「标准：」或「偏离：」起头，偏离的写明违反哪条；至少一个「标准：」选项。关键词表加「实验包」「角色」；「测试包」留着（用户说过「测试包撤掉」，仍是放置类用词） | 项目钩子 `ask-user-placement-guard.sh`，挂 `PreToolUse[AskUserQuestion]`，与 `ask-user-claim-guard.sh` 同事件；不是门禁格 |

⑧ 本身要改两处（第一轮判决已定、我只补落地写法）：允许共用的清单加 `singlefs-unit-graph` 与 `prune-server`（`.claude/gate.d/checker-independence-and-sync.sh:39-40` 现在是 format、core、wgpu、naga 一族）；两个加进去的另判「不依赖 wgpu、不含判器」——`prune-server` 的 `Cargo.toml` 里没有 wgpu、naga、任何 `singlefs-*`，`grep -rln 'singlefs_' prune-server/src` 零命中（量的）；单元图 `Cargo.toml` 只有 `verus_syn`、`proc-macro2`、`quote`，描述里写「不依赖工作区任何 crate」。**注意**：单元图是 `crates/` 下的包，⑧ 的闭包会把它的 `[dependencies]` 里的 `verus_syn`、`proc-macro2`、`quote` 一起算进两边交集，所以允许清单要写成「单元图与它经 `[dependencies]` / `[build-dependencies]` 带进来的」，照 format、core 的写法（`:39-40` 的措辞），不然 ⑧ 马上红。`prune-server` 在 `crates/` 之外（自己的空 `[workspace]`），⑧ 按名字当外部叶子，名字进清单即可。

### 9.2 `crate-roles.tsv` 初始行（搬完之后的样子，逐包）

| 包 | 角色 | 理由（一句） |
|---|---|---|
| `singlefs-format` | 产品 | 格式常量，核心层与池级 checker 共用的一边（⑧ 允许共用） |
| `singlefs-core` | 产品 | 核心层（⑧ 允许共用） |
| `singlefs-replay-prefix` | 产品 | 取 journal 前缀的纯函数核，核心层调、Verus 验 |
| `singlefs-mount-runtime` | 产品 | 挂载就起的后台运行库，只依赖核心层 |
| `singlefs-background-gpu` | 产品 | 产品 GPU 路径，测试目标 `tests/` 在包内 |
| `singlefs-checker` | 验证 | 池级 checker，判决器库 |
| `singlefs-reference-filesystem` | 验证 | RefFS，语义级参照 |
| `singlefs-verification` | 验证 | 验证包；`src/bin/` 只留 4 份验证流程起的工具 |
| `singlefs-unit-graph` | 测试支撑 | 函数单元图，验证包、产品包、实验包都依赖（⑧ 允许共用，用户弹窗定） |
| `singlefs-device-library` | 测试支撑 | 实验装置读写判定库的口子，实验包与 `scripts/experiment-apparatus` 依赖（依赖者数要算后者，见检查一④） |
| `singlefs-implementation-experiments` | 实验装置 | 22 份装置二进制，不装测试，没有依赖者 |

`crates/singlefs-background-gpu-tests` 撤掉、不登。`gpu-verification-server/`、`prune-server/`、`task-server/` 在 `crates/` 之外，不归这张表（第一轮没有要求，本轮不扩）。

### 9.3 ⑨ 对今天已有的逐处判（量的：`path_attr_scan.py` 全仓 190 处 `#[path]` 属性、`include_scan.py` 171 处 `include_str!`/`include_bytes!`）

「16 处 `#[path]`」我对上的是 `#[path = "../../` 起头的 16 行：15 处属性加 `crash_amplification.rs:1292` 一处文档注释；逐处如下。「6 处跨包 `include_str!`」对不上，我量到跨包字面 11 处、落 `crates/` 之外的数据 11 处。

| 位置 | 引什么 | 今天 | 搬完 | 判 | 理由与去处 |
|---|---|---|---|---|---|
| 测试包 `tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:11` | 验证包 `tests/common_background_gpu_compute/mod.rs` | 跨包 | 消失 | **红** | F3 定案：产品包 `tests/` 自己建 CPU 替身，这一处随之删 |
| `crates/singlefs-unit-graph/src/build_features.rs:267`、`:278` | 核心层与产品包的 `build.rs`（`#[path]` 编进单测） | 跨包 | 不变 | **放行（登记）** | 单测要把被测包的构建脚本编进来，核特性表与 `Cargo.toml` `[features]` 逐个相同（`:753`、`:802` 的两条单测）；被测的东西就是那两份文件；单元图是测试支撑 |
| `build_features.rs:722`、`:773`（`include_str!` 同两份 `build.rs`）、`:718`、`:769`（两份 `Cargo.toml`） | 同上 | 跨包（`.rs` 两处红、`.toml` 两处数据） | 不变 | `build.rs` 两处**放行（登记）**，`Cargo.toml` 两处放行（数据） | 同上 |
| `crates/singlefs-unit-graph/src/build_toolchain.rs:538`、`:542` | 验证包 `build.rs`、测试包 `build.rs` | 跨包 | `:542` 改指产品包 `crates/singlefs-background-gpu/build.rs`，另加一处指实验包 `build.rs` | **放行（登记）**，登记行同步改 | 单测核每份构建脚本把 profile 读到的环境变量逐个报给 cargo |
| 验证包 `tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:53` | 核心层 `build.rs`（`#[path = "../../../singlefs-core/build.rs"]`） | 跨包（验证到核心层） | 不变 | **放行（登记）** | 核心层是 ⑧ 的允许共用一边，编的是只含特性表常量的构建脚本，被判的正是这张表（同文件注释写明） |
| 验证包 `src/bin/e161…:28`、`e170…:45`、`e171…:61`、`e171 修后…:50`、`e184…:40/42/44/46`、`e185…:72`（9 处） | 验证包 `tests/common*/mod.rs` | 同包放行 | 装置搬进实验包后变跨包 | **红，必须改写** | 第 5 格乙：换成实验包自己的私有模块，`#[path]` 不再有 |
| 测试包 E202 `:157`、`:160`（2 处） | 测试包 `tests/common_*` | 同包放行 | E202 进实验包后变跨包 | **红，必须改写** | 第 6 格：换成 `mod`（默认路径）私有子模块 |
| 验证包 `e184…:52`、`…write_dump.rs:27` | 同目录 `…write_dump/mod.rs` | 同包 | 同包（两份一起搬） | 放行 | 同包内 |
| 测试包 `tests/pairing_conclusion…:8`、`:14`、`tests/reference_units_entries_name_functions_that_exist.rs:14`、`build.rs:8`、`:10` | `src/*units_entries.rs` | 同包 | 产品包内（登记表搬到产品 `tests/common_pairing_units_entries/`，第 3 格） | 放行 | 同包内 |
| 验证包 `e184…rs:2919`、`:4422`、`:4426`、`:4430`、`:4434`（5 处 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/….rs"))`） | 验证包 `tests/crash_enumeration_*_stream.rs` | 同包 | 变跨包 `.rs` | **红，必须改写** | 4.3：改成运行时读文件，不再是编译期引用 |
| 测试包 `tests/reference_units_entries_name_functions_that_exist.rs:173` | `../src/bin/e202_gpu_cpu_gc_comparison.rs` | 同包 | E202 进实验包后变跨包 `.rs` | **红，必须改写** | 把 E202 那一项从「调参照入口的源文件」清单里拿掉，E202 自己的参照入口核对放进实验包 E202 的内联单测 |
| E202 `:13836-13852`（5 份 `Cargo.toml`） | 产品包、核心层、格式、运行库、`replay-prefix` 的清单 | 跨包数据 | 同层深度不变 | 放行（数据，计数） | 后缀 `.toml` |
| `.claude/heavy-thread-default` 5 处、`litmus/` 6 处 | 仓根数据文件 | 不在 `crates/` 内 | 不变 | 不判 | ⑨ 只看落在另一个 `crates/<包>/` 里的 |
| 全仓其余 `#[path]` 167 处（验证包 `tests/` 内）与 `include_str!` 同包 136 处 | 同包 | 同包 | 同包 | 放行 | 同包内 |

**判定：成立**。五条检查都能按新角色写出判据；`crate-roles.tsv` 11 行逐包列了；⑨ 对今天 15 处属性、11 处跨包 `include_str!` 逐处判了。什么现象会推翻：⑨ 的扫描器对 `concat!` 拼路径漏认（量的 `include_scan.py` 对 `concat!(env!("CARGO_MANIFEST_DIR"), …)` 与 `OUT_DIR` 两种都做了，但没做宏里嵌宏的）；登记表初始行里 `singlefs-device-library` 的依赖者数算法被主 agent 判为不该跨工作区数；检查一②的判据 L 把用户想留在验证包的某份装置判成要搬（命令跑出来与 4.2 不同）。

## 10. 落地的次序

三类改动合成五簇。撞文件的地方先写出来：根 `Cargo.toml` 的 `members` 与 `Cargo.lock`（簇二、簇三都动）；`crates/mutations.tsv`（簇二、簇三、簇四都动，共约 1066 行改文本）；`crates/singlefs-unit-graph/src/build_toolchain.rs:538-546` 与 `build_features.rs`（簇二、簇三各动一处）；`crates/singlefs-verification/src/judging_units_entries.rs`（只簇三）；`.claude/gate.d/checker-independence-and-sync.sh`（只簇一）；`.claude/gate.d/crash-point-replay.sh:781` 与 `scripts/gate/merge-and-verify.sh` 的 `PRODUCT_GPU_PACKAGE_NAME`（只簇四）。所以 crates 这一侧**一次只放一个写者**，三簇串行；工具员那一侧只有簇一、簇五与簇四的门禁部分，簇一先于所有。

| 簇 | 做什么 | 动哪些文件 | 撞谁 | 先后 |
|---|---|---|---|---|
| 零 量基线 | 8.3 的三类命令搬前各跑一趟，存产物；和在改 E202 的会话、(3b)、(10)、(eb) 约窗口（调度记录里已问），以它们落完的样子为基线 | 不改文件 | — | 最先；每簇之后复量一次 |
| 一 门禁 ⑧ 允许清单 | `singlefs-unit-graph` 与 `prune-server` 加进允许共用（带它们的 `[dependencies]`）；⑧ 另判两者不依赖 wgpu、不含判器；先不加 ⑨ | `.claude/gate.d/checker-independence-and-sync.sh` 与它的样本、弄坏开关 | 不撞 crates | 先于簇二（产品包一依赖这两个，⑧ 当场红） |
| 二 产品落测试（实现员） | 产品包 `tests/` 收测试目标、共用模块；新写两份存取模块与领卡（第 2 格）；CPU 替身；`build.rs` 合并与登记表搬到 `tests/common_pairing_units_entries/`；产品 `Cargo.toml` 加 dev-dep `prune-server`、`singlefs-unit-graph`、build-dep `singlefs-unit-graph`、`[[test]]`；两个证法的测试（1.3、1.5）；**E202 与它的两个共用模块此刻先搬进实验包**（只带 E202 一份装置，其余 21 份留原处），测试包撤掉 | 测试包整包删；产品包 `Cargo.toml`、`build.rs`、`tests/`；新建实验包（含 `Cargo.toml`、`build.rs`、`build_inputs/`、E202 与子模块）；根 `Cargo.toml`、`Cargo.lock`；`build_toolchain.rs`、`build_features.rs`；`mutations.tsv` 里测试包与 E202 的 244 行 | 撞簇三（根清单、`mutations.tsv`、单元图两份文件） | 簇一之后；先于簇三 |
| 三 21 份装置搬实验包（实现员） | `git mv` 21 份加 `…write_dump/` 目录；第 5 格乙的私有脚手架副本与比对单测；E185 登记组搬实验包 `build.rs`；E161 的 4.3 三选一的落地；验证包删 21 条 `[[bin]]`、两行依赖、`judging_units_entries.rs` 两处与 `crash_points_observer_registration_is_complete.rs` 的 `bin:` 支；E184 的 5 处 `include_str!` 改运行时读 | `crates/singlefs-verification/Cargo.toml`、`src/judging_units_entries.rs`、`src/bin/`、`tests/crash_points_observer_registration_is_complete.rs`；实验包；`mutations.tsv` 815 加 7 行 | 撞簇二（同上三处） | 簇二之后，不并行 |
| 四 引用与门禁常量（工具员，照 `.claude/rules/path-moves.md`「怎么做」） | 全仓引测试包与 22 份装置路径的地方改完（191 个文件 1274 处是第一轮量的，现要重量）、出口三「除第 5 步登记外 0 命中」；`PRODUCT_GPU_PACKAGE_NAME` 两处；门禁样本档 `product-gpu*` 11 个目录；`task-server/task_server_lib.py` 里点名测试包的地方；22 个实验页里的起法命令（`-p singlefs-verification --bin <名>` 换成 `-p singlefs-implementation-experiments --bin <名>`）；`crates-mutation-rows.py:391` 走 8.2 的 (ii)（登记旧路径，不改文档串） | `.claude/`、`scripts/`、`task-server/`、`.claude/kb/experiments/` | 撞簇二、三的 `mutations.tsv`（该表由簇二、三改，簇四只复核） | 簇三之后 |
| 五 措施 M1–M5（工具员） | 检查一二（`crate-roles` 格、`crate-roles.tsv`、`crate-roles-check.py`）、检查三（⑨ 与 `crate-includes-exempt.tsv`，豁免行按 9.3）、检查四、检查五；规则文字：`.claude/rules/directory-layout.md` 新一节、`.claude/rules/verification.md`「定义与名字」（五个包，验证包一行改、加实验包一行、`:50` 的「接口待定」同步）；各配红样本与弄坏开关 | `.claude/rules/`、`.claude/gate.d/`、`scripts/gate/`、`.claude/hooks/` | 簇一动过 `checker-independence-and-sync.sh`，簇五在它上面加 ⑨：簇一落完再改；其余不撞 crates | **最后**：新检查要在落完的树上一次判绿；样本与脚本可以更早写，换上排在簇四之后 |

三方：`crates/` 全部改动走三方（`.claude/agent-common.md` 的规定），簇二、簇三各一轮代码轮，第九格「放的位置」那一轮就写这两簇；治理文件（规则、门禁、钩子）簇一、簇五各一轮。

### 10.1 什么现象会推翻这个次序

簇二里「E202 先搬」与测试包的撤包同窗口做不到（E202 的两个共用模块被 11 个测试目标共用，产品包还没有自己的取卡模块时 E202 无处可搬）——那就把簇二拆成「二甲：产品落测试」「二乙：E202 搬」，二乙在前、二甲在后，只要 E202 先 `git mv` 取卡模块与胶水的副本、测试包原模块暂留到二甲删；代价是多一个窗口、E202 两组键多重判一次。

## 要主 agent 跑的命令

我没往统一任务服务交任何东西。下面每条的范围声明（`SINGLEFS_RUN_SCOPE='…'`）种类我没核对（`.claude/hooks/run-scope-guard.sh` 文件头），主 agent 照那份补；命令本体如下。

1. 4.1 判据的核法与 4 留 22 搬的分法：
   `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r2-forward/gate_starts_bin.py /home/fy5090/code/singlefs`；在仓根起；看末行 `stay 4 move 22` 与四行 `STAY …`（`crash_library_reader`、`e165_replay_bounded_enumeration`、`new_pool_file_creation_device_log_check`、`new_pool_file_creation_on_device`）。我已用同一脚本只读跑过一遍，输出同上。
2. 9.3 的 `#[path]` 扫描：
   `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r2-forward/path_attr_scan.py /home/fy5090/code/singlefs`；在仓根起；看 4 行 `CROSS …` 与末行 `{'same-package': 186, 'cross-package': 4} total 190`。
3. 9.3 的 `include_str!` 扫描：
   `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r2-forward/include_scan.py /home/fy5090/code/singlefs`；在仓根起；看 `CROSS-package` 11 行与末行 `{'same-package': 136, 'CROSS-package': 11, 'generated/other': 13, 'outside-crates': 11} total 171`。
4. 8.3 变异表基线（搬前、每簇后各一趟，存输出比）：
   `task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items --with-keys`；在仓根起；看末尾「键对不上（要现判）的条数」那一行与每项 `ITEM_PLAN` 的 `reusable=`，`TASK_SERVER_RESULT` 的 `exit=0` 算过。
5. 8.3 崩溃判定库基线：
   `task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items --with-keys`；在仓根起；看末尾「键」行（块键对不上几块、两种判器那一份的判法单元指纹）与各项 `ITEM_PLAN to_judge=`；GPU 判器组的判法单元指纹搬前搬后逐字比。它会起只读读库程序，连的是真库（只读）。
6. 8.3 对拍键表：编完产品包与实验包后，在各自 `target` 下的 `out` 目录里取 `reference_units_keys.tsv`、`gpu_side_units_keys.tsv`，逐组比 `group` 行的十六进制键；编的命令是 `cargo test --release -p singlefs-background-gpu --features tests-on-a-real-gpu --no-run` 与 `cargo test --release -p singlefs-implementation-experiments --features tests-on-a-real-gpu --no-run`，由主 agent 照 `scripts/hosts` 的编译目录口径交（这两个包在簇二、三落地前还不存在，所以这一条在簇二之后才能跑）。**待主 agent 跑**。
7. 1.3、1.5 的证法测试：两个包各一条按名点名的 `cargo test -p <包> --test <目标> -- <测试全名…> --exact`（形态见 `.claude/agent-common.md`「日常开发的测试不跑整个包」），目标与测试全名由簇二的实现员定；**待簇二写完**。

没有模型目录，没有 `rerun.sh`，所以没有「复跑命令」与 sha256；草稿目录里的脚本只读、只用标准库。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 两侧录入键怎么证 | 成立 | 谓词 P 写死；证法甲（双钉黄金表加门禁比表）与丙（临时库交叉读写）是最小一组，乙作加强；键的录入单元指纹、两侧单元指纹、`0x09` 值按定义不同；不算另开判定存放处 |
| 2 产品测试的存取代码 | 成立 | 两份新模块各约 330 行、领卡 +110、临时库 +60，合计约 830 代码行（推的）；加 dev-dep `prune-server`、`singlefs-unit-graph` 与 build-dep `singlefs-unit-graph`；按需领卡不撞「任务不拿卡」，但 `verification.md:50` 的「接口待定」要同步 |
| 3 实验包的接法 | 成立 | `crates/singlefs-implementation-experiments`；产品包不能写可选依赖；E185 的参照组、E202 两组登在实验包自己的 `build.rs`；验证包库里 `judging_units_entries.rs` 与一个测试要连带改；⑧ 不失效，前提是没有任何包依赖实验包 |
| 4 26 个装置怎么分 | 成立（E161 推不出） | 判据 L 写死、命令核过：留 4（`crash_library_reader`、`new_pool_file_creation_on_device`、`…device_log_check`、`e165_replay_bounded_enumeration`）、进 22；E161 的观察者键在验证包静态表里，三选一要主 agent 选 |
| 5 共用模块从哪来 | 成立（乙）；甲成立但换键量推不出 | 乙：实验包私有副本约 820 行加运行时比对钉，340 个测试键不动；甲：上界 233 个目标换键；丙两条不成立、一条推不出 |
| 6 E202 取卡与对拍层 | 成立 | 取卡层与胶水层各一份，各写各的；借卡用验证包现成的；约 735 行重复是已知代价；没有绕回测试包的形状 |
| 7 `pci_bus_identifier_without_the_domain` | 成立 | 产品测试那份合法各写各的；E202 改用验证包的；第 204 行注释改成 7 节那三行 |
| 8 单元键与现判的量 | 成立（两项推不出） | 30 加 1 条持卡判、约 31 条 CPU 重录是推的；产品参照 7 组预计不变；验证包 GPU 判器组键会不会因删登记行而变、混合窗口里 8312 行要不要重判，推不出，命令在 8.3 |
| 9 措施判据定稿 | 成立 | 五条检查按四种角色重写；`crate-roles.tsv` 11 行；⑨ 逐处判完：红 4 类（要改写）、放行（登记）6 处、数据放行与不判若干；单元图要写进 ⑧ 允许清单的连带依赖 |

## 没做什么

- 不判别的腿（攻方腿）的格：没有碰，第 1、3、5、6 格我只给了能落地的做法与「什么会推翻」，不替主 agent 采纳、不出判决。
- 没往统一任务服务交任务；判决要引的数（8.3 的三类现判量、对拍键表、两个证法的测试）没复跑，命令在上节；`CARGO_BIN_EXE`、`include_scan.py` 之类的量数是我用只读命令与草稿目录小脚本跑的，主 agent 要引的话照上节命令再跑一遍。
- 没写模型目录、没有 `rerun.sh`、没起样本进程；没改仓里任何文件，只写了这份报告与草稿目录里的五份只读脚本。
- 没读 `research/prompts/gpu-tests-placement-design-r2-opus-output.md`、`…-opus-model/` 与攻方腿草稿目录。
- 没核的：E161 在 `crash-entries.tsv:23`（门禁 crash-flow-coverage 格的入口登记）里的角色——要读 `scripts/crash/crash-flow-coverage.py` 才知道搬走会不会让那一格红；`e158_root_choice_repair` 的 `e158-r3-arm` 特性是否真是未声明（只读到 `cfg(feature = …)` 命中）；`e171` 两份的 `cfg(feature = "gpu")` 是否都在 `unexpected_cfgs` 之下；任务服务的范围声明种类。
