# 验证纪律：只有一个门禁，不指定范围与指定范围两种跑法

## 定义与名字

四个包，全仓只用这四个名字称呼它们：

| 名字 | 包 | 装什么 | 什么时候跑 |
|---|---|---|---|
| **验证包** | `crates/singlefs-verification` | 单元测试、集成测试、它们用的脚手架库与实验、真设备装置二进制（`src/bin/`），连同各道门禁判的东西 | 照「一个门禁：两种跑法」那张表 |
| **池级 checker** | `crates/singlefs-checker` | 判一个镜像的判决器库（O2，`check_pool_image`，D13（验证路线） 已定项 7）；只依赖 `singlefs-format` | 被验证包调用；它自己的库单测随时跑 |
| **RefFS** | `crates/singlefs-reference-filesystem` | 语义级的参照文件系统：只在内存里按 POSIX 操作走，判一次崩溃之后恢复读出的内容落不落在允许集合里；依赖只有 `std`，不引实现、池级 checker、验证包，实现与池级 checker 也不依赖它 | 被验证包调用；它自己的单测随时跑 |
| **函数单元图** | `crates/singlefs-unit-graph` | 把 Rust 与 WGSL 源码按条目切成单元、取去注释指纹、从入口收闭包、算复用键的库，只用 `std`（D13（验证路线） 已定项 17 的键算法住在这里）；归测试基础设施，不是产品代码 | 被验证包的构建脚本与门禁脚本调用；它自己的单测随时跑 |

查验证包哪个模块装什么、哪几道门禁连带判它之前，先读 `.claude/rules/verification-reference.md`「定义与名字」。

- 测试文件按它测什么起名，领域在前、场景在后；崩溃枚举、崩溃点、崩溃注入、坏盘输入、记录核对器的测试文件以 `crash_enumeration_`、`crash_points_`、`crash_injection_`、`bad_disk_input_`、`record_checker_` 起头。名字不带来历（`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」）；本仓判红的形态：以里程碑序数起头的、`*_step_four_*`、`*_supplement_two_*`、`*_parallel_line_one_*`、`*_c519_*`。
- 不许用的叫法：「checker 包」（分不清是池级 checker 还是验证包）、「放量用例」「验证档」（说验证包）。

## 测试不分档，红了只重跑红的

- 验证包的测试不分档：`cargo test -p singlefs-verification` 跑到包里全部测试，不按快慢、大小标 `#[ignore]`；谁交、怎么交照 `.claude/agent-common.md`「子 agent 不交任务」。
- 红了只重跑红的那几条：交 `python3 scripts/gate/rerun-failed-tests.py <上一趟的日志> -p <包>`，它按测试目标打的命令（`task-server/task log <号> --all` 里取）逐条原样交；它判红、一条命令都不打时才整份重跑。完整的一遍归提交时的整轮门禁。
- 崩溃枚举用例：调崩溃枚举函数（`enumerate_crash_states` 一族），或自己逐个造崩溃状态（名字带 `every_crash`；循环里对录制操作取到循环变量为止的前缀去 `apply`，或造 `CrashImage`）的测试，直接做或经同一文件里的函数做都算。它写进 `crates/singlefs-verification/tests/`、登记 `crash-case:`（登记进崩溃点重放合表 `.claude/gate.d/crash-point-replay-items.tsv`，点名 `crash-case:` 行或放量分项）、不标 `#[ignore]`。验证包里导入崩溃引擎模块的测试文件第一行声明它测哪几个模块（照 `scripts/crash/crash-case-check.py` 的 modules；门禁 `crash-point-replay` 开跑前判，红了不起 cargo）。
- 一条用例一个场景（`.claude/rules/test-script-discipline.md`「用例原子化：每一条都能单独跑」）：本仓把每一轮新建一个池（`build_pool`、`build_through_*`、`format_pool`、`MemoryPool::with_devices`）的参数循环认作多个场景；确是一个场景的，在用例上面写一行 `// harness-test-granularity:one-scenario <理由>`。
- 判器调用点（池级 checker、记录核对器、GPU 判器）落在判定段里，或包住它的函数带 `// gpu-exempt: <截名>` 或 `// cpu-reference-for-gpu-comparison` 标记；两样都不是的判红（门禁 checker-independence-and-sync 的 cpu-judging-sites 格，判法 `scripts/crash/cpu-judging-sites-check.py`）。标记写在包住调用点的那个函数前面：属性、文档注释与签名之前紧挨着的 `//` 注释行，或属性与签名之间的注释行；外层函数带了标记，里面嵌套的函数一并算；写在函数体里的不算。
- 变异行点名的测试要跑得到：`--` 之后的筛选词筛得到点名的测试（门禁 code-source-discipline 的 mutation-tables 格判）。
- 控制测试线程的只有变异那一路：别的 `cargo test`（门禁 `crash-point-replay` 各趟、`scripts/crash/crash-point-replay.sh`、`scripts/gate/admission.py` 拼的 crash-case 命令、崩溃放量、手跑的 GPU 测试）参数里不带 `--test-threads`，测试线程取登记的每台默认重活线程数（`.claude/heavy-thread-default`，服务经 `RUST_TEST_THREADS` 交下来）；变异那一路怎么带见 `.claude/rules/verification-reference.md`「测试线程」。

## 一个门禁：两种跑法

门禁只有一个，跑法两种，没有分档，也没有不读判定库、全部现判的跑法；两种跑法是同一套脚本、同一份判定库，只差范围。

| 跑法 | 怎么起 | 谁跑、什么时候 |
|---|---|---|
| 不指定范围 | 门禁阶段不带 `--check` 与 `--item`；`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh` 不带 `--item`；`scripts/experiment-apparatus/replay.sh` 不带实验号；`cargo test` 不点名测试目标；`gate.sh` 整轮。范围声明种类 `whole` | 提交时由主 agent 经任务服务交（`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md`）；提交之外照「提交之外」那一条 |
| 指定范围 | `--item <项>`、`--check <格>`、`cargo test` 点名测试目标（`--test`、`--lib`、`--bin`）。范围声明种类 `items` 或 `target` | 主 agent 随时经任务服务交，点这一次改到的项、格与目标；子 agent 一律不交，写进报告「要主 agent 跑的命令」一节 |

- **提交之外**：不指定范围的只在提交时跑；提交之外要跑（包括用户要求的那一趟），一律先弹窗问用户，同意了才跑。提交时那一趟的确认就是带 `--full-gate` 的那一次弹窗。
- 判哪些、复用哪些由门禁照每条判定的单元键自己选，只剪键相同、已判过的与等价的。单元键只由判定依赖的函数单元各自去注释后的指纹、闭包真引到的 cfg 原子与它们的真值、工具链、解析出的闭包真引到的依赖版本与判定自己的内容输入组成；路径、名字、行号、资源上限、限时、提交与树不进键（D13（验证路线） 已定项 17）。两种排除不在这句全称里：算键的库自己两份（`scripts/crash/lib_verdict_library.py`、`scripts/gate/lib_function_units.py`）的单元不进任何一份判定的键（crates 变异判法除外，那里照进），一条只打提示、不改判定结局的子进程边按实测排；估「哪些判定要重判」之前先读 `.claude/rules/verification-reference.md`「单元键的组成与排除」。崩溃判定库的键不读仓根；断言就是关于真仓、要读活树的测试拒算（照跑、不复用、不入库）。
- 按判定入库：凡起编译、跑自证、现算键的判定都按判定入库、按单元键复用、记红撤绿（判红的照同一把键记红、撤掉这把键原有的绿，下一趟现判），没有每趟整格照跑的格；例外只有 prune-server 与 task-server 自己的自证与只读文本的静态检查格。broad-mutation 的驱动只算翻面（同一把键先抓到、后判没红），不沿用、现判一次。带判定库的门禁与入口登记在 `.claude/gate.d/verdict-library-scripts.tsv`，各道的键与键前缀写在它的文件头。
- 每一趟真跑开跑前从键相同、库里有判定的项里随机抽几条当哨兵、再抽一条强制现判（种子打进日志），哨兵有一条被现判、强制现判的判定与库里那一条不同、或强制现判那一条仍复用，整道红。
- **缩域**：崩溃域只往小缩；不给缩域参数、不设缩域变量就是全量域。缩过域的那一趟成功句写「缩过域、不算门禁通过」并写出实际值，不当门禁通过。
- 指定范围的那几趟（点名测试目标、`--item`）崩溃状态数压在 20 万量级；不指定范围那一趟不管。几十万态以上的大域对拍，跑之前先报预计要多久。
- 门禁红了必须修好或查清原因，红着不提交。
- 提交时只走一条路：先照 `.claude/skills/crash-test/commit-stages.md` 跑提交时的六道门禁与崩溃放量全部分项（都带 `task-server/task submit --full-gate`），再照 `.claude/skills/gate/staged-gate.md` 跑 `scripts/gate/gate-staged.sh` 整轮；整轮里这六道照键复用先跑那一趟写下的判定、不重判。
- 一切跑项目代码的命令只认经 `task-server/task submit` 起；起任何验证的命令开头带范围声明 `SINGLEFS_RUN_SCOPE='种类=<whole|items|target>; 跑什么=…; 范围=…; 为什么=…'`（写在 submit 前面）。改完只跑上次红的与这一次改到的包、目标、项或格，整轮留提交前；范围合不合适由主 agent 事后对着检出记录看。

起一道门禁或入口、照它的写法单跑一趟、写范围声明、核它的判定单位、缩域与复用自证之前，先读 `.claude/rules/verification-reference.md`「一个门禁：两种跑法」。

**凡用 GPU 先清场，跑完不复原**：清场与分卡都由服务做，调用方经 `task-server/task submit -- <命令>` 交，不在外面套清场、不挑卡；跑完、跑红或被打断都不复原。清场判出卡坏（`task wait` 末行 `type=card-fault`）时，主 agent 当场弹窗请用户重置那张卡，不等、不重试。跑或改用 GPU 的测试、入口与门禁之前，先读 `.claude/rules/verification-reference.md`「凡用 GPU 先清场，跑完不复原」「测试线程」。

## 判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB

- 全仓只有一份判定库：prune-server 自己的那份 RocksDB。库里只放剪枝按单元键复用时要读的键：Rust 一侧是 `crates/singlefs-verification/src/verdict_store.rs` 文件头登记的各表，Python 一侧是 `scripts/prune-server/prune_server_client.py` 文件头「键空间」点名的各族；各族的键前缀以那两处为准。
- 不进库的：运行记录（峰值表、占盘时序、上次红的清单）放多机配置 `NFS_SHARED_ROOTS` 点名的共享根，两台同一绝对路径、读同一份，不经 `~` 与 `$HOME` 取（第二台的家目录是另一个）；进仓的登记表随 git 走；产物、日志、临时现场归各自的目录。
- 不许任何测试、门禁、入口脚本、实验装置、agent 新建自己的判定库：另开一个目录存判定文件、另写一张 TSV 或 sqlite、另开一份 RocksDB 都算。要存判定就经 prune-server 的客户端写进那一份（Python `scripts/prune-server/prune_server_client.py`，Rust `prune_server::reliable_client`）。中转库（只在并回服务那一份之前存在、并回即删）不算另开，并不进去的由开它的那一趟报红、不留着。自证与压测一律对自己起的临时服务，格完即删，不写真服务，也不拿真任务当测试。
- 不设开关：读写判定库只有经服务这一条路，交任务不带任何选库的变量。服务连不上退 75，不退回本地文件。
- 一族「接完」= 三件按次序做完：① 它的入口与门禁缺省走服务，本地文件库那条代码路径删掉、不留开关；② 旧库不导入：那一族还要的判定由它的入口经服务现判、照键写回；③ 只删那一族旧库所在的目录，别的族的目录不碰；那一族接完之后旧库由主 agent 删，删法照这一件。旧库不原地打开、不并存。哪族接完以 `.claude/gate.d/verdict-library-scripts.tsv` 第三列现读为准，task-server 按这一列收任务。再写旧库、或悄悄另建一份本地库，都算违例。
- 这一条门禁还没判，欠账 C948 在 `.claude/kb/checks-owed.md`；立前靠 review：审改动时见到新的判定存放处就打回。

查库里有哪几张表、改那张登记表或判一份脚本该怎么登、核库名怎么取之前，先读 `.claude/rules/verification-reference.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」。

## 所有任务都要多机可用，不许绑定某一台

- 测试、门禁、入口脚本、实验装置、服务的控制命令，派到两台里任何一台都要跑得起、判得对，判出来的结果两台都读得到（判定进那一份库，运行记录与产物放 `NFS_SHARED_ROOTS` 点名的共享根）。派到某一台就红的是缺陷，由脚本那一侧当场修，不登例外；不往 `task-server/task_server.py` 里加只派一台的名单（它没有这种名单，文件头 ② 排队的「机器」）：任何命令都派得到两台，哪台由余量定，不钉台。
- 不读会改变判定结局的本机独有状态：许读的本机私有状态只有 `~/.config/singlefs/` 下的私有名单与密钥；多机配置 `multi-host.env` 两台用同一份（仓根 `multi-host.env.example` 文件头）；不写死主机名与地址；环境相关的一切只由多机配置定，没写就是没有，不补缺省。
- 一条测试自己的配置文件（QEMU 的 `qemu.env`、预编库的 `rocksdb-prebuilt.env` 这类）是那条测试的事，不在「环境相关的一切只由多机配置定」之内：进仓或放共享根，两台都读得到；只在一台有的，是那条测试的缺陷。判本台环境的门禁格（机器干净不干净、本机私有名单）两台结局本来不同，不算违例。
- 这一条门禁还没判，欠账 C949 在 `.claude/kb/checks-owed.md`；立前靠 review 与服务派到第二台的那几趟。

## 配置文件读出什么就是什么，脚本不另设自己的变量

- 机器、显卡、显卡数、进程数、线程数、CPU 数这类环境参数，脚本只从配置文件读（多机配置 `multi-host.env`、一条测试自己的配置文件、进仓的登记表），读出什么就用什么：不另设只有这份脚本认的环境变量，不在脚本里补缺省值，不把读出的值再换算成另一个数。
- 测试确实要用别的值时（压小并发、只用一张卡这类），用配置文件里那个参数的名字显式覆盖，写在起那条测试的命令或用例里，看得出盖的是哪一项、盖成多少；不另起名字绕开配置，不写不带名字的数。线程数例外：经服务交的命令里不许写，照 `.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」。
- 这一条门禁还没判，立前靠 review：审改动时见到脚本新读一个配置文件里没有的环境变量、或给配置项补缺省，就打回。

## 函数名与类型名不许只由空泛词拼成

每一个词都在 `.claude/naming-vague-words` 里的名字判违规（`run_cell`、`CellRun`、`check_cell`、`fn run`、`fn get`）：改成说出跑的是什么、判的是哪条、取的是哪个量（`evaluate_stream_domain`、`count_named`、`perform_commit_step`）。`impl <trait> for <类型>` 块里 trait 规定的方法名不判。还没改完名的文件登记在 `.claude/naming-vague-exclude`，只缩不涨。

## 崩溃一致性只能靠崩溃点重放验证

把块层所有写请求记下来，在**每一个**可能的崩溃点截断、重放、跑池级 checker 与记录核对器。**没跑过这个的写路径就不算验过。**

## 功能正确性靠模型对拍

在内存里维护一个只管语义、不管性能也不管崩溃的理想文件系统（`crates/singlefs-verification/src/model.rs`），把同一串随机操作分别施加到模型和实现上，比结果。它是这个项目唯一的功能对照物。

## checker 即规范

不变量清单（`crates/singlefs-checker/invariants/`）每加一条，池级 checker 就加一个检查。**「这个格式到底是什么」，答案以 checker 的源码为准，不是文档。**

## 文件系统特有的反推缺口

- **「测试全绿」不等于「实现正确」。** 要说「崩溃一致性成立」，先说清这一趟本来撞不撞得上：枚举了多少个崩溃点，是不是全部。
- **「checker 没报错」不等于「镜像是好的」。** 说这句话之前，先看 `crates/singlefs-checker/invariants/` 里对应那条的实现状态。

## 门禁的结构

每道门禁都能全跑，也能逐格点名跑：每格一个函数、判一件事，经共用库 `.claude/gate.d/lib/stage-cells.sh` 逐格登记，文件头写格名表；按项判的门禁另认 `--list-items` 与 `--item <项>`；每道阶段认第一个参数当项目根（门禁 code-tooling 的 stage-owners 格判）。写或改一道门禁、读它的参数、汇总与退出码之前，先读 `.claude/rules/verification-reference.md`「门禁的结构」。

## 门禁管哪一半

`.claude/rules/verification.md` 各条判据由哪道门禁、哪一格判，逐行在 `.claude/rules/verification-reference.md`「门禁管哪一半」；查一条判据有没有门禁管、红了归哪一格之前先读它。
