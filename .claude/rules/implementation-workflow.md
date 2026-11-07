# 实现改动的流程：写代码 → 三方对抗 → checker，不指定范围的只在提交时跑

**这是 singlefs 的项目本地规则**，不在共享 SOP 里；三方论证照 `.claude/rules/three-way-inference.md`。共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的产品代码走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」（不派本地模型的腿）；打中的写回代码，再攻一轮。射程是产品代码：`crates/singlefs-core/`、`crates/singlefs-format/`、`crates/singlefs-replay-prefix/`、`crates/singlefs-reference-filesystem/` 的 `src/`（含其中的 `#[cfg(test)]` 模块）。测试代码不走这一步：`crates/*/tests/` 下的一切、验证包 `crates/singlefs-verification/`、池级 checker `crates/singlefs-checker/`、函数单元图 `crates/singlefs-unit-graph/`（测试基础设施库，归验证包依赖的一侧）、`crates/mutations.tsv`；只改了注释的产品代码文件也不走（去掉注释与空白之后代码与改前逐个相同；文档注释里会编成 doctest 的代码块、文档注释与 `// SAFETY:` 注释的增删照代码算；判法是规范副本的 `.claude/singlefs-ai-sop/scripts/comment-only-change.py`，show-me-test 用同一份） | 门禁 doc-process-records 的 crates-adversarial-review 格判形式：这次改动里每个 `crates/*/src/*.rs`（`crates/singlefs-verification/`、`crates/singlefs-checker/`、`crates/singlefs-unit-graph/` 下的与只改了注释的除外）都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名，或者用户明说这次不走三方、按内容哈希登记进 `.claude/crates-review-exempt`（一行：路径、sha256、他说了什么，形态同 `.claude/agent-def-review-exempt`）。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 门禁 `crash-point-replay`（跑法照 `.claude/rules/verification.md`「一个门禁：两种跑法」）、crates-mutation-replay（crates 变异表复跑）、code-source-discipline 的 mutation-tables 格（实验装置的变异表 `scripts/mutation/tables/`，加 `crates/mutations.tsv` 的静态几样）、`check.sh` |

**次序不能倒**：第 2 步的三方攻的是写好的产品代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。只改测试代码的不走第 2 步：第 1 步照做（带测试、每条新测试先证明会红），改完直接跑。测试代码的设计判断——测试怎么写、覆盖范围够不够、用例对不对——照旧是推论，照 `.claude/rules/three-way-inference.md` 开推论轮走三方，不并进按批的代码轮。

## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 doc-process-records 的 agent-def-adversarial-review 格判形式（形态照同一道的 crates-adversarial-review 格）。

**三步在定义上各取什么**：第 1 步不适用（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外）；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `show-me-test.md`「新增的测试必须先证明它会红」里「设计论证同样照办」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），主 agent 写判决时拿它核腿引的行，门禁另拿它比判决之后又改过没有（门禁 doc-process-records 的 crates-adversarial-review、agent-def-adversarial-review 两格）；腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：腿交齐之后、写判决之前拿快照 `sha256sum -c` 一遍；对不上的，倒推出快照时的原样再核——HEAD 之后没被这一轮碰过的取 HEAD，被别的会话定点替换过的把那几处替换反着做一遍，副本的 sha256 与快照相同才算数；倒推用的改动清单存进这一轮的证据，判决开头写明哪个文件、被改了几处、腿引的行落没落在那几处。

## 复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判

改动之后一条判定要不要重判，判据是**它的单元键在判定库里有没有键相同、作数的判定**：键只由这条判定依赖的函数单元各自去注释后的指纹、闭包真引到的 cfg 原子与它们的真值、工具链、解析出的闭包真引到的依赖版本与它自己的内容输入组成（D13（验证路线） 已定项 17）。算键时有两种排除。文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键，crates 变异判法除外：crates 变异判法不在文件级排除的射程里，那里两份照进判法单元指纹、覆盖号带判法单元指纹。边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`。边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。除判定库服务 prune-server 与统一任务服务 task-server 自己的自证、只读文本的静态检查格之外，起编译、跑自证、现算键的判定都按判定入库、按单元键复用剪枝、记红撤绿（`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation、research-build-and-replay，与 code-tooling、model-differential-and-scenarios 按条复用的各格），整道、整格没有复用判定；带判定库的门禁与入口登记在 `.claude/gate.d/verdict-library-scripts.tsv`。

复用由阶段自己做、自己报：每道的成功句报这一趟现判、复用各几条，有拒算的另报拒算几条（broad-mutation 读驱动的结论，这几个数在它起的驱动的成功句里），作废的判定报得出是哪几个函数单元变了（键的组成清单随判定落盘）。收尾报告抄每道的这一句；没有这一句的那一道按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。

⚠️ **不许拿「我只改了文档」当理由**（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

## 不指定范围的只在提交时跑，指定范围的随时跑

两种跑法各是什么，在 `.claude/rules/verification.md`「一个门禁：两种跑法」；这里只写实现改动流程里的落点：

| 场合 | 跑不跑 |
|---|---|
| 实现员交回前 | 一条都不交（`.claude/agent-common.md`「子 agent 不交任务」），`cargo fmt` 自己跑：自己动到的测试目标（`cargo test -p <包> --test <目标>`、`--lib`）、clippy / build、证红、命名检查与崩溃点重放入口、formal-verification、crates-mutation-replay 带 `--item`（点这一次改到的项）各写成 `task-server/task submit --cwd <自己的副本> -- <命令>`，范围声明照 `.claude/hooks/run-scope-guard.sh` 文件头写在 submit 前面，写进报告「要主 agent 跑的命令」一节；主 agent 交、等到结局，红了的新派实现员（带「承接：」）接着改 |
| 每次提交代码 | 不指定范围那一趟：主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑提交时的六道门禁——`crash-point-replay`（crash-case-replay 格包里全部测试函数按单元键复用、登记的崩溃枚举用例逐条按崩溃状态块跑；feature-gated-tests 格带 `tests/` 里 `cfg` 点到的特性另跑一趟；product-gpu-real-card-tests 格产品 GPU 包带 `tests-on-a-real-gpu` 的真卡对拍，`--test-threads 1`，服务没交卡判红、测试找不到显卡报「本次未跑」）与 qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation（自己先起广谱变异驱动剪枝跑：每条变异按单元键沿用库里的结论、沿用不了的现判；有一条没结论或没红就判红，红了不提交）；它们各按自己的键前缀把判定写进同一份判定库，再由主 agent 照 `.claude/skills/gate/staged-gate.md` 经服务交 `scripts/gate/gate-staged.sh` 整轮（`.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行），整轮里这六道照键复用先跑那一趟写下的判定、不重判 |
| 提交之外要跑不指定范围的 | 主 agent 先弹窗问用户，同意了才交 |
| 仓副本里正文改过的脚本的自证 | 子 agent 不交（`.claude/agent-common.md`「子 agent 不交任务」），把命令写进交回报告；主 agent 经 `task-server/task submit --cwd <集成副本> -- …` 交，不弹窗；副本里的拷贝不在 `scripts/gate/selftest-process-guard.py` 的射程里，跑之前主 agent 看一眼那份自证（连同它 source / import 的库）相对主工作区的 diff（`diff -u <主工作区里那一份> <副本里那一份>`，逐份比），正文起编译、测试或别的重活的不交，退回改成假驱动 |
| 其余任何时候 | 指定范围的（点名测试目标、`--item`、`--check`）主 agent 随时经任务服务交；子 agent 都写进交回报告由主 agent 交 |

一切跑项目代码的命令只认经 `task-server/task submit` 起，由 `.claude/hooks/project-code-guard.sh`（Bash 的 PreToolUse）在执行前拒绝直接起的（判定在 `.claude/hooks/lib_project_code_runs.py`，逐类的写法在它文件头）；包装（`bash -c`、`nice`、`timeout`、`env`、`/usr/bin/time`、`flock` 这类）里面的同样算，命令位置上执行的脚本闸读进去逐行判；子 agent 起 `task-server/task` 的 submit、wait、cancel、clean 同样拒。派发提示里叫子 agent 跑项目代码的句子由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。

herd7 / LKMM、QEMU 与 Verus 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。三样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/qemu-device-streams.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |
| Verus 形式化验证 | `.claude/gate.d/formal-verification.sh`（判法在 `scripts/gate/verus-proof-check.py` 与 `scripts/experiment-apparatus/e165-pin-red-examples.py`） | 逐格判什么在 `.claude/rules/verification.md`「门禁管哪一半」 |

三道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

## 测试与崩溃检测优先多线程

- **写法**：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么，照 `.claude/rules/test-script-discipline.md`「编译与算法都要快」。本仓用 `std::thread::scope`，不为这个加依赖；每片各自建状态；线程数的环境变量没设就取登记的每台默认重活线程数（`.claude/heavy-thread-default`；Rust 装置的取法在 `scripts/experiment-apparatus/heavy_threads_default/threads.rs`）；经服务跑的线程固定是这个数、不许覆盖：分到哪台、机器忙不忙都是它，提交时不写线程，命令里写了 `SINGLEFS_THREAD_CAP`（`-- env SINGLEFS_THREAD_CAP=…` 也算）客户端当场拒。没照这样取的登记在排除表 `.claude/heavy-thread-exempt`，只缩不涨（门禁 code-tooling 的 heavy-thread-defaults 格判）。
- **进度间隔**：5 分钟。单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度：每片报区间与已判的数。
- **分片**：分不分片、哪一片去哪台机器、每一片的上限都由服务定，服务把一件拆成计划、各片、合并三步，各步各是一条子任务（`task-server/task_server_lib.py` 文件头「分片」）；任务里面不自己连第二台。门禁 `crash-point-replay` 与 `crates-mutation-replay` 都不调分片驱动、不自己分两台，每条在起它的那一台上跑。崩溃枚举用例要两台分着判的，登记行第三列加 `shard=across-machines`（登记表 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行），另经 `task-server/task submit -- bash scripts/crash/crash-point-replay-shard-run.sh --merged-log …` 交服务：它把同一组参数转给崩溃放量的分片驱动 `scripts/crash/crash-amplification-two-hosts.sh` 的同一步（计划写两片 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2` 与 `1/2`，都不钉台、由服务挑机器，两片判的块都经客户端写进 prune-server 那一份判定库；合并起一趟 `SHARE=0/1` 的核账，两片判的块按此刻的键全在服务里读到、一块不现判才算过），核账那一趟的日志抄进给的日志文件、照单机的判法判，门禁下一趟照块复用。crates 变异表要两台分着判的，在仓根另经 `task-server/task submit -- bash scripts/mutation/mutation-shard-run.sh .` 交服务：计划把判定库里没有键相同判定的行按上一次现跑的用时分成两堆（拒算与锚点失效的行不进两堆，留给合并之后经服务起的那一趟整张判），各片照 `GATE_MUTATION_ROW_SELECTION` 起这一道门禁只判自己那一堆、判定写进 prune-server 里同一份判定库，合并按本机此刻源码现算的行键核两片判的行在服务里读得到（一片判的行一行都读不到就判红、点名那一片）；之后照常经服务起这一道整张判（行键相同的复用、别的现跑）。

**门禁管哪一半**：崩溃点重放由门禁 `crash-point-replay` 判：整轮门禁与提交时跑不指定范围那一趟（crash-case-replay 格 `cargo test --release -p singlefs-verification --lib --tests` 包里全部测试函数按单元键复用；feature-gated-tests 格再带 `tests/` 里 `cfg` 点到的特性另跑一趟；product-gpu-real-card-tests 格：产品 GPU 包带 `tests-on-a-real-gpu` 的真卡对拍，`--test-threads 1`，服务没交卡判红、测试找不到显卡报「本次未跑」），登记的崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行）经 `--skip` 排出按条那几趟、逐条按崩溃状态块跑：块的判器结论与挂在这个点上的观察者结论都有单元键相同、不红的判定才复用，缺的当场判（登记行第三列写了 `exhaustive=<前缀>` 的，那一行计数行都要带 `exhaustive=true`），有红块判红；用例跑了至少两片、没有显式把线程数设成 1、机器多于 1 核却只用了 1 个线程，判红（判法在 `scripts/gate/admission.py` 的 `judge_worker_threads`，由它的 `--selftest` 拿合成日志核）。经服务分片判的用例判的是合并核账那一趟的日志，工作线程照单机判；分片三步由 `scripts/crash/crash-amplification-two-hosts.sh --selftest` 核（假的服务照分片协议逐步起它），`scripts/crash/crash-point-replay-shard-run.sh --selftest` 核交给驱动的参数、三步各转给驱动的同一步、合并与不分片时日志抄回与退出码；派到第二台的那一片在同一棵共享的树里、同一个绝对路径上跑，不送树、不取回（`task-server/task_server_lib.py` 文件头「挑机器」与「分片」）。别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方（只改测试代码的不开代码轮，只靠实现员）。
