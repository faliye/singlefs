# 实现改动的流程：写代码 → 三方对抗 → checker，不指定范围的只在提交时跑

三方论证照 `.claude/rules/three-way-inference.md`。

## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/rules/show-me-test-by-review.md`「新增的测试必须先证明它会红」） | show-me-test 阶段判「有没有测试」；会不会红写进变异清单与 commit message，两样都要，写法与哪类测试进哪张表照 `.claude/rules/show-me-test-by-review.md`「新增的测试必须先证明它会红」 |
| 2 多 agent 正反对抗推理 | 改完的 `crates/` 代码（产品代码与测试代码）走一轮三方：材料带上 diff 与它压着的条款，`three-way-forward`（或 `three-way-defense`）核「代码做的是不是条款说的」、`three-way-attack` 攻「哪一格会错」（不派本地模型的腿）；轮数与收法照「代码轮与治理文件轮怎么收」一节；代码轮正文每一轮按「代码轮正文的检查单」一节逐格点名。射程是 `crates/` 下的全部代码：各包的 `src/`（含其中的 `#[cfg(test)]` 模块）与 `tests/`，验证包 `crates/singlefs-verification/`、池级 checker `crates/singlefs-checker/`、函数单元图 `crates/singlefs-unit-graph/` 同样在内，加 `crates/mutations.tsv`；只改了注释的文件不走（去掉注释与空白之后代码与改前逐个相同；文档注释里会编成 doctest 的代码块、文档注释与 `// SAFETY:` 注释的增删照代码算；判法是规范副本的 `.claude/singlefs-ai-sop/scripts/comment-only-change.py`，show-me-test 用同一份） | 门禁 doc-process-records 的 crates-adversarial-review 格判形式：这次改动里每个 `crates/*/src/*.rs`（`crates/singlefs-verification/`、`crates/singlefs-checker/`、`crates/singlefs-unit-graph/` 下的与只改了注释的除外）都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名；那三个包的 `src/`、各包的 `tests/` 与 `crates/mutations.tsv` 这一格还没判，由主 agent 写判决时逐份点名。豁免表 `.claude/crates-review-exempt` 与 `.claude/agent-def-review-exempt` 不再加行，现有的行补完一轮三方删一行。点名了判得对不对，要人看；同一道的 code-round-checklist 格判代码轮正文（`_<题>-code-r<数字>-body.md`）里检查单八格的标题在不在、底下写没写字，答案对不对由腿与主 agent 核 |
| 3 checker 测试 | 池级 checker、崩溃点重放、变异表：三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 门禁 `crash-point-replay`、crates-mutation-replay、code-source-discipline 的 mutation-tables 格（`scripts/mutation/tables/` 与 `crates/mutations.tsv` 的静态几样）、`check.sh` |

**次序不能倒**：第 2 步攻的是写好的产品代码与它的测试，攻在计划上的那一轮替代不了它。只改测试代码的同样走第 2 步，并进按批的代码轮；测试怎么写、覆盖够不够、用例对不对在同一轮里攻。

## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写步骤、约束、判据、交付；为什么与是什么归 `.claude/kb/`、`records/`，定义里不写、也不留解释的尾巴，要用时写成「开工先读 `.claude/kb/<某份>` 的〈某节〉」。上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方（轮数与收法照「代码轮与治理文件轮怎么收」一节），判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 doc-process-records 的 agent-def-adversarial-review 格判形式（形态照同一道的 crates-adversarial-review 格）。

**治理文件同规矩**：项目规则（`.claude/rules/`）、技能（`.claude/skills/`）、钩子（`.claude/hooks/`）、门禁（`.claude/gate.d/`）、`scripts/` 下的脚本、统一任务服务 `task-server/` 与判定库服务 `prune-server/`（客户端在 `scripts/prune-server/`）的改动按批走一轮三方（`.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条；轮数与收法照「代码轮与治理文件轮怎么收」一节），判决按路径点名改过的每一份，不等用户点名、不登豁免；这几类门禁还没判，由主 agent 写判决时逐份点名。

**三步在定义上各取什么**：第 1 步不适用（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没有测试的 patch 一律不收」只管 `crates/*/src/`，只改文档和脚本除外）；第 2 步就是「改它们与改 `crates/` 同规矩」那一轮三方；第 3 步照 `.claude/rules/show-me-test-by-review.md`「新增的测试必须先证明它会红」里「设计论证同样照办」那一句：三方打中的每一条，先在模型里做成一个必须报非 0 的世界，再改定义。

## 代码轮与治理文件轮怎么收

代码轮（`crates/` 改动的三方轮）与治理文件轮（改定义、共用约束、规则、技能、钩子、门禁、`scripts/` 下的脚本、统一任务服务与判定库服务的三方轮）照同一套收，不用「先挂起、三轮多数」；那一套只管设计推论轮，设计推论轮的定义与那一套在 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」。

- 每一轮（第一、第二、第三轮）停不停，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」第一条，判据只在那里写。
- 有打中的，主 agent 逐条现查：坐实的就改（代码轮做成会红的测试后写回代码；治理文件照被打中的那一条条款改，改定义与共用约束的先照「改 agent 定义与共用约束，走同一条三步」一节第 3 步做成必须报非 0 的世界再改），改完再攻一轮，没新打中就收，最多三轮。
- 改完再攻的那一轮，攻上一轮的修补，加前一轮在正文或判决里写明只抽查过、没看过的地方；已攻过而没打中的角度不重复，派发提示里明令写上。攻击腿撤回自己先前给的方案，比它打中新东西更有价值。
- 开了第二轮的，在第二轮派一条辩方腿复核第一轮的判决；设计推论轮的辩方腿时点另有写法，在 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」「开不开下一轮（设计推论轮）」。
- 第三轮改的修补没被再攻过，在判决里标「零轮」；要写进产品代码的，先照 `.claude/rules/three-way-inference.md`「腿提的、判决里标「零轮」的改法，写进产品代码之前先在探针里对它做 A/B」做 A/B。

## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照，放这一轮的材料目录；主 agent 写判决时拿它核腿引的行，门禁 doc-process-records 的 crates-adversarial-review、agent-def-adversarial-review 两格拿它比判决之后又改过没有。腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。

**别的会话改了快照里的文件**：腿交齐之后、写判决之前 `sha256sum -c` 一遍，对不上的倒推出快照时的原样再核，副本的 sha256 与快照相同才算数。记快照、倒推、在判决里写明之前，先读 `.claude/skills/three-way/SKILL.md`「代码轮派腿之前记一份开工快照」。

## 代码轮正文的检查单

代码轮的正文 `research/prompts/_<题>-code-r<数字>-body.md` 每一轮都逐格点名八格，每格一个标题「### 检查单：<格名>」，底下写这一轮对这一格的答案与证据（文件:行）；只改测试代码的轮同样写。腿对每一格回「成立」「不成立」或「推不出」，推不出的算打中，按三方的写回判决办。八个格名逐字是：走不到的理由二次查实、类型与名字、不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、带 --item 的命令与项、新崩溃流多罩了什么。写正文或当腿判这八格之前，先读 `.claude/skills/three-way/SKILL.md`「代码轮正文的检查单」。

## 复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判

- 改动之后一条判定要不要重判，判据是**它的单元键在判定库里有没有键相同、作数的判定**；键由什么组成在 `.claude/rules/verification.md`「一个门禁：两种跑法」。
- 复用由阶段自己做、自己报：每道的成功句报这一趟现判、复用各几条，有拒算的另报拒算几条（broad-mutation 的这几个数在它起的驱动的成功句里）；作废的判定报得出是哪几个函数单元变了。收尾报告抄每道的这一句；没有这一句的那一道按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。
- 复用核查（同一份输入跑第二趟现判 0、哨兵与强制现判那一条）固定在一棵快照树上做，不在活的主工作区量。
- ⚠️ **不许拿「我只改了文档」当理由**（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。

改算键的代码、登记文件级或边级排除、在快照树上做复用核查之前，先读 `.claude/rules/verification-reference.md`「复用门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判」。

## 不指定范围的只在提交时跑，指定范围的随时跑

两种跑法各是什么在 `.claude/rules/verification.md`「一个门禁：两种跑法」；实现改动流程里的落点：

| 场合 | 跑不跑 |
|---|---|
| 实现员交回前 | 一条都不交（`.claude/agent-common.md`「子 agent 不交任务」），只自己跑 `cargo fmt`；动到的测试目标、clippy / build、证红、命名检查、登记的崩溃枚举用例（门禁 `crash-point-replay` 按块那一路）、崩溃放量分项、formal-verification、crates-mutation-replay 带 `--item`（点这一次改到的项）各写成 `task-server/task submit --cwd <自己的副本> -- <命令>` 写进报告 |
| 每次提交代码、提交之外要跑不指定范围的、其余任何时候 | 照 `.claude/rules/verification.md`「一个门禁：两种跑法」那张表、「提交之外」那一条与提交时只走一条路那一条 |
| 仓副本里正文改过的脚本的自证 | 子 agent 不交，把命令写进交回报告；主 agent 经 `task-server/task submit --cwd <集成副本> -- …` 交，不弹窗；跑之前主 agent 逐份 `diff -u <主工作区里那一份> <副本里那一份>` 看那份自证（连同它 source / import 的库），正文起编译、测试或别的重活的不交，退回改成假驱动 |

herd7 / LKMM、QEMU 与 Verus 不归上游 singlefs-ai-sop 管，是本工程自己的三道门禁 lkmm、qemu-device-streams、formal-verification；三道都在 `gate.sh` 里，`--staged` 那条路同样跑，单跑一道不算跑过门禁。

## 测试与崩溃检测优先多线程

- **写法**：切片并行、线程数、合并要确定、报进度、不能并行的写明为什么，照 `.claude/rules/test-script-discipline.md`「编译与算法都要快」。本仓用 `std::thread::scope`，不为这个加依赖；每片各自建状态；线程数的环境变量没设就取登记的每台默认重活线程数（`.claude/heavy-thread-default`）。经服务跑的线程固定是这个数、不许覆盖：提交时不写线程，命令里写了 `SINGLEFS_THREAD_CAP`（`-- env SINGLEFS_THREAD_CAP=…` 也算）当场拒。没照这样取的登记在排除表 `.claude/heavy-thread-exempt`，只缩不涨（门禁 code-tooling 的 heavy-thread-defaults 格判）。
- **进度间隔**：5 分钟。单项跑得超过 5 分钟的，每过 5 分钟至少打一行进度：每片报区间与已判的数。
- **分片**：经服务交的入口，文件头写了 `# task-server-shards:` 的一律分片，服务把一件拆成计划、各片、合并三步；哪一片去哪台、每一片的上限由服务定；任务里面不自己连第二台。
- **门禁管哪一半**：门禁判得出的那一半是门禁 `crash-point-replay` 的工作线程判据与 heavy-thread-defaults 格；别的测试是不是能并行而没并行，门禁看不出，靠实现员与代码三方。

改各道门禁的分片三步、Rust 装置怎么取线程数、读门禁判工作线程的那一行之前，先读 `.claude/rules/verification-reference.md`「测试与崩溃检测优先多线程」。
