# 实现改动的流程：写代码 → 三方对抗 → checker，checker 档只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」（不派本地模型的腿）；打中的写回代码，再攻一轮 | 门禁 doc-process-records 的 crates-adversarial-review 格判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 门禁 `checker-tier-layer0-replay`（checker 档快档，全量按 `.claude/rules/verification.md`）、checker-tier-crates-mutation-replay（crates 变异表复跑）、code-source-discipline 的 mutation-tables 格（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 doc-process-records 的 agent-def-adversarial-review 格判形式（形态照同一道的 crates-adversarial-review 格）。

**三步在定义上各取什么**：第 1 步不适用——定义是文档，`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「这条同样管设计论证」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），主 agent 写判决时拿它核腿引的行；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：主 agent 管不住别的会话。腿交齐之后、写判决之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再核——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、被改了几处、腿引的行落没落在那几处。

## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判

改动之后要不要重跑某一道，判据是**那一道读的全部输入自上次跑绿以来变没变**，不是「我改的是不是 `crates/`」。一道读的输入常常不止源码：herd7 读 `litmus/` 与代码里的锚点，QEMU 读装置二进制；每一道读哪些路径，以 `.claude/gate.d/stage-inputs.tsv` 里它那一行为准。没登记在那张表里的阶段没有复用判定，每次照跑；`checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation 都登记着。

⇒ 要复用一道的判定，三样都要拿出来，缺一样就重跑：

| 要拿出什么 | 怎么算数 |
|---|---|
| 那一次跑的日志，且那一道在里面判绿 | 引它的原样判定行，不转述 |
| 那一次跑的索引与现在的索引，在**那一道读的每个路径**上逐字相同 | 登记进 `.claude/gate.d/stage-inputs.tsv` 的阶段由 `scripts/gate/stage-must-run.sh` 自动比对（`refs/sop/staged-green` 那棵树与这一次的暂存树；只在 `scripts/gate/gate-staged.sh` 起的那一趟里比，直接跑 `gate.sh` 一律照跑），不必再手工逐个路径现查；没登记的阶段仍要手工核，输出不截断，说不全那一道读什么就没有复用的资格 |
| 那一次之后的改动一条都碰不到那些路径 | 把改动清单与输入清单并排列出来 |

复用要在收尾报告里写明：复用了哪几道、引的是哪一次跑、比对了哪些路径。不写的按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

## checker 档只在提交时跑，harness 随时跑

两档各是什么、谁跑、带什么前缀，在 `.claude/rules/verification.md`；这里只写实现改动流程里的落点：

| 场合 | 跑不跑 |
|---|---|
| 实现员交回前 | harness：自己动到的测试二进制（`cargo test -p singlefs-harness --test <目标>`、`--lib`）、fmt / clippy / build，经内存包装；轻型里形式化那一条：核心部分加按自己改到的文件点名的项（命令与跑法照 `.claude/rules/verification.md`「两档：轻型与重型」），经内存包装；崩溃与变异两条轻型不起，命令写进报告「要主 agent 跑的命令」一节，主 agent 在实现员的副本里跑；重型一样都不跑（三条不带 `--quick` 的跑法也是重型） |
| 每次提交代码 | checker 档快档，命令带 `SINGLEFS_HEAVY_TESTS=commit`：主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑 `checker-tier-layer0-replay`（快档：`cargo test --release -p singlefs-checker-tier --lib --tests`，再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）与 checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation（先起广谱变异驱动把库目录换到这一次的暂存树、`--changed-since HEAD` 只现判改到的，`.claude/skills/crash-test/commit-stages.md` 第 2b 步；有一条没结论或没红就判红，红了不提交）；整轮门禁由主 agent 照 `.claude/skills/gate/staged-gate.md` 带前缀跑 `scripts/gate/gate-staged.sh`（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行） |
| 用户要求 | checker 档全量：`checker-tier-layer0-replay` `--full`（HEAD + 暂存区的 worktree 里，逐条按输入复用）、其余重型测试；命令带 `SINGLEFS_HEAVY_TESTS=user-request`；任务确实要跑时主 agent 先弹窗问用户 |
| 其余任何时候 | 重型不跑；子 agent 只跑 harness 与形式化那一条轻型 |

**重型测试**就是 checker 档那一批加上整机资源级的活，判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（逐类的写法在它文件头）。轻型三条（`scripts/crash/crash-point-replay.sh`、门禁 checker-tier-formal-verification、checker-tier-crates-mutation-replay 带 `--quick`，可再带 `--item <项>`，`.claude/rules/verification.md`「两档：轻型与重型」）不算重型；子 agent 交回前只跑形式化那一条，崩溃与变异两条交主 agent 跑；同三条不带 `--quick` 的剪枝、逐条（`--item`）、全量（`--full`）是重型。重型是：崩溃点重放入口的重型跑法、跑到 checker 档包 `singlefs-checker-tier` 的测试（`cargo test -p singlefs-checker-tier`、不挑包而包的范围含它、直接执行它的测试二进制）、门禁 `checker-tier-layer0-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification（连同直接起它的判法脚本 `scripts/gate/verus-proof-check.py` 证明、空真、必红三种用法与 `scripts/experiments/e165-pin-red-examples.py --replay`）与 checker-tier-research-build-and-replay 的 experiment-replay 格、`qemu-system-*` 与 `scripts/devices/vm-bench.sh`、`.claude/scripts/lkmm.sh` 与 `herd7`、参数里有 `crates/mutations.tsv` 的 `scripts/mutation/mutate.sh`、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `.claude/scripts/check.sh`、`gate.sh` 整轮与 `scripts/gate/gate-staged.sh`、E152 装置、广谱变异（`scripts/mutation/broad-mutation.py` 起 cargo 或改库目录的每一种用法；`--list-items`、`--summary`、`--check-ignored`、`--help`（`-h`）与登记为 light 的单独 `--selftest` 不算）与门禁 checker-tier-broad-mutation（只点名 gate-verdict 一格的 `--check gate-verdict` 不算）、崩溃放量（`scripts/crash/crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-two-hosts.sh` 的真跑；`crash-amplification.sh --list-items` 与 `.claude/hooks/selftest-registry.tsv` 登记为 light 的 `--selftest` 不算）、真编 QEMU（`scripts/devices/build-patched-qemu.sh` 除 `--selftest` 之外的用法）。包装（`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算；命令位置上执行的脚本闸读进去逐行判。由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：子 agent 跑任何一样拒绝；主 agent 不带前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝；许可子 agent 跑轻型三条的放行。

herd7 / LKMM、QEMU 与 Verus 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。三样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/checker-tier-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/checker-tier-qemu-device-streams.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |
| Verus 形式化验证 | `.claude/gate.d/checker-tier-formal-verification.sh`（判法在 `scripts/gate/verus-proof-check.py` 与 `scripts/experiments/e165-pin-red-examples.py`） | 逐格判什么在 `.claude/rules/verification.md`「门禁管哪一半」 |

三道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

## 测试与崩溃检测优先多线程

- **写法**：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么，照 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「编译与算法都要快」。本仓用 `std::thread::scope`，不为这个加依赖；每片各自建状态（`SharedStream` 这类 `Rc` 不能跨线程）；线程数的环境变量没设就取 `std::thread::available_parallelism`。
- **进度间隔**：5 分钟。单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度：每片报区间与已判的数。
- **双机分片**：崩溃枚举用例的枚举认分片开关的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）。`checker-tier-layer0-replay` `--full` 在本地配置（仓根 `multi-host.env`，模板 `multi-host.env.example`，判法 `scripts/crash/layer0-shard-configuration-check.sh`）判得过时，把这几条交给 `scripts/crash/layer0-shard-run.sh --merged-log`：本机跑 0/2、第二台跑 1/2、本机 merge；判不过照单机跑。第二台那一片由驱动在第二台上经 `scripts/hosts/run-with-memory-cap.sh` 起，上限是配置的 `PEER_MEMORY_CAP`；本机那一片与 merge 由起 `checker-tier-layer0-replay` 的那一层内存包装管。驱动只由 `checker-tier-layer0-replay` `--full` 调。门禁 `checker-tier-crates-mutation-replay.sh` 只在本地多机配置写了 `ENABLE_ACROSS_MACHINES=1` 时按行分两台（判法同一份：`scripts/crash/layer0-shard-configuration-check.sh` 退 0 双机开、退 3 双机关、退 1 判不过），不认环境变量开，设了 `SINGLEFS_GATE_FULL=1` 或 `GATE_MUTATION_START_OVER=1` 时也不分：开着时先交 `scripts/mutation/mutation-shard-run.sh`，本机没有作数按条记录的行两堆均分、两台各判一堆，第二台的按条记录连同覆盖记录经 `scripts/mutation/crates-mutation-rows.py import` 核过判法指纹、行键，再照本机此刻的源码过一遍按覆盖复用的判据（判据在它文件头「复用判据」）才导入；驱动退出之后本机整张判一遍（按覆盖复用得上的复用、别的现跑；只读看会怎么判用它的 `plan <根>`），全绿标记由本机写。判法退 3 时驱动报一行「双机：关」、什么都不分。这个驱动只由这一道门禁调；它与这道门禁的整套重设计归别的会话。

**门禁管哪一半**：崩溃点重放由门禁 `checker-tier-layer0-replay` 判：整轮门禁与提交时跑快档（`cargo test --release -p singlefs-checker-tier --lib --tests`），再逐条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）按它这批输入的指纹核那一格全绿标记（两条流的层 0 全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红；全量只在用户要求时，在 HEAD + 暂存区的 worktree 里用那棵树里的 `checker-tier-layer0-replay` 跑 `bash <它的根>/.claude/gate.d/checker-tier-layer0-replay.sh --full <它的根>`，逐条按输入复用、只重跑输入变了的；这一趟跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `scripts/gate/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。双机分片跑的那几条判的是 merge 那一趟的日志，工作线程逐片判：某一片这一趟跑了至少两片、那台机器多于 1 核、那一片的线程数没显式设成 1，却只起了 1 个线程，判红（判法在 `judge_threads_of_each_shard`）；驱动另判两台的工具链与输入指纹相同、两片的账本各恰好一份，第二台那一片退 250–254（内存包装自己的结局）判红；工具链、账本与第二台那一片经没经内存包装由 `scripts/crash/layer0-shard-run.sh --selftest` 核，输入指纹两台不同与 250–254 那两支没有自证格。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。
