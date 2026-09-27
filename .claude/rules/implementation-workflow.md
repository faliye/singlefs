# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」、本地腿（`three-way-local-attack` 或 `-defense`，主 agent 按 `.claude/rules/three-way-inference.md`「各条腿必须互不重复」定）按分到的那一面找反例或辩护；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），连同派腿的时刻（`date -u`）交核查员当输入；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、派核查员之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再交核查员——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、几点、被改了几处、腿引的行落没落在那几处。

## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；54、55、57、59 号都登记着。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `research/scripts/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树；只在 `research/scripts/gate-staged.sh` 起的那一趟里比，直接跑 `gate.sh` 一律照跑），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

## 重型测试只在提交时跑

**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：

- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
- 全部实验复跑：87 号。
- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
- 崩溃枚举用例：`.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的那几条用例的测试目标（`cargo test` 点名它、`--test` 的通配命中它、或不挑目标而包里有它；直接执行它的测试二进制），而下面任一条成立的：libtest 参数带 `--ignored` 或 `--include-ignored`（`cargo nextest run` 是 `--run-ignored only` / `all`）；`--config` 或环境变量里定了测试二进制的 runner（`--config` 的值按 TOML 读，带引号的键也算；`systemd-run -E` / `--setenv`、`strace -E` / `--env` 设给里面那条命令的环境变量也算），或子命令是 `--config` / `CARGO_ALIAS_` 定的别名；登记的用例函数有一处定义没标 `#[ignore]`（同名的每一处 `fn <名>(` 都算），或判不出标没标（找不到那个函数、宏生成的用例、导入不了 `research/scripts/admission.py`，按没标算）。另外任何命令带 `--ignored` / `--include-ignored` 又点名登记的用例函数的也算（`grep`、`git` 这类按文本处理参数的除外）。不带这两个参数、用例函数标了 `#[ignore]`、只跑那个测试目标里快用例的不算。双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。

只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算，`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf stat|record|trace` 包在外面的剥掉照算（短选项合写的 `strace -fo <文件>`、`flock -xw 10` 同样剥）。重型测试清单各类里的 `cargo test` 同样指 `cargo nextest run`、`cargo miri test`、`cargo llvm-cov`、`cargo hack test`、`cargo mutants` 与别名；libtest 参数里 `--list` 当选项出现（只列用例、一条都不跑）的哪一类都不算，跟在 `--skip`、`--logfile` 这类带值的选项后面的 `--list` 是那个选项的值，照算。命令位置上执行的脚本（`bash <脚本>`、`./<脚本>.sh`、`source <脚本>`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。

| 场合 | 跑不跑 |
|---|---|
| 每次提交代码 | **必须跑**，命令都带 `SINGLEFS_HEAVY_TESTS=commit`：层 0 全量（HEAD + 暂存区的 worktree 里 `--full`，连同登记的崩溃枚举用例）、QEMU、herd7、crates 变异整表由 `crash-verifier` 跑，主 agent 后台派、看门狗盯；整轮门禁由 `gate-triage` 带前缀跑 `research/scripts/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
| 其余任何时候 | 不跑 |
| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员在 HEAD + 暂存区的 worktree 里跑 54 号 `--full`（连同登记的崩溃枚举用例），另跑 55、57、59 号；门禁分诊员跑 `research/scripts/gate-staged.sh`（它跑 `gate.sh --staged`）并分诊：54、55、59、74、87 号只在这一趟里能复用上一次整轮全绿的判定（判据在 `research/scripts/stage-must-run.sh` 文件头），要跑时 54 号跑快档并核全绿标记，55、57、59 号同样只核各自的全绿标记 |
| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |

由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

## 测试与崩溃检测优先多线程

- **写法**：彼此独立的单位按区间切片，用 `std::thread::scope` 并行，不为这个加依赖。每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）。线程数从环境变量取，没设就取 `std::thread::available_parallelism`。
- **合并要确定**：计数按片的次序相加，「第一处」取序号最小的。输出与线程数 = 1 时逐字相同，并且在同一份代码上核过一次。
- **跑的过程中报进度**：每片报区间与已跑的数，长用例的日志一直在涨。
- **不能并行的写明为什么**：共享状态、次序本身就是被测对象，写在那条用例的注释里。
- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。54 号 `--full` 在本地配置（仓根 `layer0-shard.env`，模板 `layer0-shard.env.example`，判法 `research/scripts/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `research/scripts/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `research/scripts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 54 号的那一层内存包装管。驱动只由 54 号 `--full` 调。

**门禁管哪一半**：崩溃点重放由门禁 54 号判：整轮门禁跑快档，再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`）；全量在 HEAD + 暂存区的 worktree 里用那棵树里的 54 号跑 `bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `research/scripts/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `research/scripts/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
