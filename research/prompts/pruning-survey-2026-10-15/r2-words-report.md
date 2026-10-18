# 第二轮复查（路子 words：文字、钩子、派发闸与自证）

日期：2026-10-15。只读：主工作区一个文件没改；没起 cargo、门禁、测试二进制、判定库。探针只跑在草稿目录的仓副本（`.claude` 与 `scripts` 两个目录，`rsync -a --exclude target --exclude .git --exclude __pycache__`）上，经 `scripts/hosts/run-with-memory-cap.sh 8G`。
记号：「量」= 这一轮现读、现跑过（行号用 `grep -n` / `awk 'NR==n'` 现取，原文整行抄在同目录 `appendix-cited-lines.md`，由脚本逐行取出，不手抄）；「推」= 由底稿第一节的「重做方向」或句意推出，这一路没核算法。
底稿 = `~/.local/share/singlefs/work/survey-pruning-ledger/ledger-r1.md`（机制号 L1–L29）；第一轮 docs 报告 = `~/.local/share/singlefs/work/survey-pruning-r1-docs/report.md`（D1–D97）。

## 一、结论

1. D1–D97（D49 空号）每个号都有去向：照改、并入、不改三类，逐号见第二节，三类各几个由第二节末尾那条命令现数（原样输出贴在那里）。「不改」的每一条写了理由，多是「史料：说的是那一次发生的事」「说的是跑的范围，不是复用单位」「只是指过去，那一处改了这句就对」。
2. 新找到的另编 N 号（第三节，条数由第三节末尾命令现数）：第一轮没列的有 `verification.md:13` 前半「几类测试各一份的长期判定库」、`verification.md:46` 的库名、`fs-design.md:114`「这次 diff 到底碰了哪几条」、`commit-stages.md:13`、`:14`、`agent-common.md:86`、`stage-owners.tsv:5`、`:11`、`scripts/gate/README.md:7,8,10`、`stage-inputs.tsv:1`、`:35`、`implementation-workflow.md:74` 里崩溃用例「两台各写各的判定库」、`verification-build.md:269` 里「空库起跑先清空自己那一份」、checks-owed 的 C630、C793、C836，以及七道重型门禁与二十个脚本文件头里命中旧词表的 513 行（机械扫出，含误命中，同目录 `appendix-header-lines.md`）。
3. 钩子与派发闸的判法没有一处读 `--changed-since`、`--sync-tree`、`refs/sop/staged-green`、全绿标记或 `judge-version`（量：副本里起 `lib_heavy_tests.heavy_tests_in_command_text`，原样输出在第四节）。要跟着改的是注释、自证用例、一个弄坏开关的定义与格数注释、派发闸 ⑨ 的拒绝句与自证名（第四节逐处列）；派发闸 ⑨ 只认 `admission.py experiment` 退 77，实验准入改成单元键后判法不动。
4. 上游两句的新写法与交上游会话的一段在第五节；D13 新立已定项 17 的整段、已定项 9、15、16 的改句、索引与变更史条目，十三个 C 号各怎么改或关，E164 F4 / W5 的回看行，在第六节。
5. 收尾核：第七节一份只读脚本，今天在 17 份文件里命中 86 行（量），改完应只剩第七节列明留着的几行。

## 二点〇、新口径用语（下面每一句新写法都按这几个词写）

- **函数单元**：Rust 的 fn 项与方法、const、static、类型定义（struct、enum、union、type、trait 与 impl 头）、宏定义；Python 的 def 与模块级赋值；shell 的函数与顶层语句段；wgsl 的 fn 与模块级 const、struct。每个单元去注释后取指纹（文档注释里会编成 doctest 的代码块与 `// SAFETY:` 正文照代码算）。
- **单元闭包**：从一条判定的入口单元（判器入口 fn、测试 fn、装置 `main`、被验的 Verus fn、litmus 锚点指到的 fn）沿调用与引用收到的全部函数单元，连同它们引用的常量、类型、静态项与宏展开。收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一条拒算键、照跑，不静默少收（底稿 L13）。
- **单元键**：单元闭包里各单元指纹的多重集 + 工具链（`rustc -Vv`、外部工具版本、cargo 解析后的生效构建环境，底稿第二节第 4 条）+ 从 `Cargo.lock` 解析出的、闭包真引到的外部包（名、版本、来源、checksum）及其传递依赖 + 这条判定自己的内容输入（段写表与起步镜像摘要、litmus 去注释内容、变异原文与替换文这类）。路径、名字、行号、序号、登记行、资源上限、限时、提交、树都不进。
- **组成清单**：每把键连同组成它的单元清单一起落盘，作废时报得出是哪几个单元变了（底稿一之二节「归因缺口」）。
- **判定库**：每类判定一份，固定在一处持久目录，不随趟、不随判法换目录、没有版本目录；落在开机会清空的目录被拒；第二台判出的并进本机那一份，第二台不留长期库。
- **全量开关**（`--full`、`--start-over`、`SINGLEFS_GATE_FULL=1`、`GATE_MUTATION_START_OVER=1`）：只写不读，现判的照键写回同一份长期库，不另建库（底稿第二节第 1 条）。

## 二点一、判法与推翻条件

- 一句话算「旧口径」：说复用或重跑看树、ref、提交、HEAD；说键里有整文件原文；说键或作废单位是路径、文件、目录、包、整表、整道、整格；说判法版本是手写数或一份管整库的摘要；说键里有路径、名字、行号、上限、限时；说库放在会清的位置、两库各判、按版本目录删。只提「剪枝」「复用」而没说键与粒度的不算。
- 判「改」还是「留」照 `evidence-discipline.md`「扫出来的清单不能一把全替换」：旧值换成新值仍是真话的是现状句，改；换了就成假话的是那一次的事（实验页整行抄的产物、记录、已还清的欠账），留，必要时在旁边加一句回看。
- 每条新写法照底稿第一节对应机制的「重做方向」写成现状句；底稿没定、要设计判断的，句末标「推」，并在第八节列给主 agent。
- 推翻条件：① 某句新写法拿去对照改完的算法，算法实际不是那样做的（例：广谱变异驱动改完仍认 `--sync-tree`），那一句作废、照算法重写；② 某处「不改」的句子改完之后把旧值换成新值仍是真话，那一处改判「照改」；③ 收尾核（第七节）改完还命中第七节没列的行。
- 改 `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/*.md` 的那几处要走一轮三方或按内容哈希登记用户豁免（`implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）；改 D13 正文走写回（kb-scribe）与变更史；改上游两句不在本仓改（第五节）。

## 二、D1–D97 逐号去向

写法：每条一行「号 ｜ 去向 ｜ 文件:行 ｜ 旧〔…〕 ｜ 新〔…〕 ｜ 量/推」。旧〔〕里是那一行里的一段原文（第二节末尾的脚本逐条核过：在那份文件里恰好出现一次，可直接当 Edit 的旧串）；新〔〕是换上去的现状句。整行原文见 `appendix-cited-lines.md`。去向只有三种：照改、并入（并到哪个号写明）、不改（写理由）。

### `.claude/rules/verification.md`（底稿 L1–L6、L12、L17、L18、L22、L29）

- D1 ｜ 照改 ｜ `.claude/rules/verification.md:13` ｜ 旧〔提交门禁 checker-tier-broad-mutation 判这一次暂存树那一版每一条变异都有结论、没有没红〕 ｜ 新〔提交门禁 checker-tier-broad-mutation 判此刻源码上每一条变异都有单元键相同的结论、没有没红〕 ｜ 量（句）/ 推（L22 改完的形态）
- D2 ｜ 照改 ｜ `.claude/rules/verification.md:34` ｜ 旧〔再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，标记不作数的报「本次未跑」、不判红；快档的复用数从日志 `CRASH_POINT_REPLAY_PARALLEL_FINISHED` 行的 `resumed_slices=` 现算，取不出记 unknown、成功句点名〕 ｜ 新〔再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例：每条用例的崩溃状态块在崩溃判定库里都有单元键相同、不红的判定，缺块的报「本次未跑」、不判红；快档两趟里每条测试函数各是一条判定，按单元键（测试函数的单元闭包、特性组合、工具链、解析出的依赖版本）入库，只跑键对不上的那几条（`--exact` 名单），成功句报「现判 N、复用 M」〕 ｜ 量（句）/ 推（L17、L18）
- D3 ｜ 照改 ｜ `.claude/rules/verification.md:35` ｜ 旧〔在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），那一格全绿标记在就复用〕 ｜ 新〔在 HEAD + 暂存区的 worktree 里逐条跑登记的崩溃枚举用例（`--include-ignored --exact`），每条用例的崩溃状态块按单元键在崩溃判定库里复用、键对不上的块现判（再带 `--start-over` 不读库、全部现判，现判的照键写回同一份库）〕 ｜ 量（句）/ 推（L17；`--start-over` 的新义照底稿第二节第 1 条）
- D4 ｜ 照改 ｜ `.claude/rules/verification.md:36` ｜ 旧〔一份库装全部分项，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind`，落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」，键与判法版本在 `crates/singlefs-checker-tier/src/crash_identity.rs`，存放布局在 `crates/singlefs-checker-tier/src/verdict_store.rs`〕 ｜ 新〔崩溃放量、崩溃枚举用例、给定状态表与两种对拍共用一份崩溃判定库，落在 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}` 下固定的一个库目录（不随趟、不随判器换，不带测试类与存法名），落在开机会清空的目录里被拒；复用与续跑照同一篇「判定可复用：按输入存，按前缀剪」：每块的键是它的单元键（判这一块真调到的判器单元闭包，wgsl 按 fn 切、去注释；段写表与起步镜像摘要、段展开、工具链、解析出的依赖版本），算法在 `crates/singlefs-checker-tier/src/crash_identity.rs` 与 `judging_closure.rs`，存放布局在 `crates/singlefs-checker-tier/src/verdict_store.rs`〕 ｜ 量（句）/ 推（L1–L4、L12；库目录名由改动批定）
- D5 ｜ 照改 ｜ `.claude/rules/verification.md:38` ｜ 旧〔checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑（`scripts/gate/stage-must-run.sh` 文件头）。〕 ｜ 新〔checker-tier-qemu-device-streams（每档）、checker-tier-lkmm（每份 litmus）、checker-tier-crates-mutation-replay（每行）、checker-tier-formal-verification（每条）、checker-tier-broad-mutation（每条变异）与 checker-tier-research-build-and-replay 的 experiment-replay 格（每行复跑）、mutation-tables-compile 格（每条变异）各按判定入库：每条判定的键是它的单元键，键对得上的复用、对不上的现判，阶段成功句报「现判 N、复用 M」；整道、整格没有复用判定（各道的键写在它文件头「复用」一节）。〕 ｜ 量（句）/ 推（L14、L18–L25）
- D6 ｜ 照改 ｜ `.claude/rules/verification.md:47` ｜ 旧〔不带参数是剪枝（只剪键相同、已判过的与等价的）〕 ｜ 新〔不带参数是剪枝（只剪单元键相同、已判过的与等价的；单元键只由判定依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本与判定自己的内容输入组成，路径、名字、行号、资源上限、限时、提交与树不进键，D13（验证路线） 已定项 17）〕 ｜ 量
- D7 ｜ 照改 ｜ `.claude/rules/verification.md:52` ｜ 旧〔选逐条的，主 agent 列出输入变了的项给用户点〕 ｜ 新〔选逐条的，主 agent 列出有判定的单元键对不上的项（各道 `--list-items` 连同每项键对不上的判定条数）给用户点〕 ｜ 量（句）/ 推（`--list-items` 报条数是新要求，第八节）
- D8 ｜ 照改 ｜ `.claude/rules/verification.md:111` ｜ 旧〔按条复用的键与判定库写在它的文件头「按条复用」〕 ｜ 新〔每条的单元键（被验 fn 与它 requires、ensures 用到的 spec fn、proof fn；钉表每行是那一行档与参数重放真走到的装置函数单元，连同状态输入与期望逐态行）与判定库写在它的文件头「按条复用」〕 ｜ 量（句）/ 推（L24）
- D9 ｜ 照改 ｜ `.claude/rules/verification.md:119` ｜ 旧〔崩溃点重放跑了、快档绿、每条登记用例的全绿标记作不作数〕 ｜ 新〔崩溃点重放跑了、快档绿、每条登记用例的崩溃状态块在崩溃判定库里有没有单元键相同、不红的判定〕 ｜ 量
- D10 ｜ 照改 ｜ `.claude/rules/verification.md:121` ｜ 旧〔广谱变异：这一次暂存树那一版里改到的变异每一条都有这一版的结论、沿用的结论都在、没有一条没红 | 门禁 checker-tier-broad-mutation：library-version-is-staged-tree 格判驱动库目录此刻那一版与暂存树在导出范围上逐文件相同、工具链与全局键相同，gate-verdict 格判〕 ｜ 新〔广谱变异：此刻源码上每一条变异都有单元键相同的结论、没有一条没红 | 门禁 checker-tier-broad-mutation：gate-verdict 格判〕 ｜ 量（句）/ 推（L22：library-version-is-staged-tree 格随「一版」删）
- D10（同一行第二处）｜ 照改 ｜ `.claude/rules/verification.md:121` ｜ 旧〔基线按登记行记的行号找那一版 `baseline.tsv` 的那一行复核〕 ｜ 新〔基线按抓它的那条测试的单元键找那一条基线判定复核〕 ｜ 量（句）/ 推（L22「覆盖与基线按执行过的单元存」）

### `.claude/rules/implementation-workflow.md`（底稿 L14、L15、L17、L21、L22）

- D11 ｜ 照改（整节重写，D12–D15 并入这一条）｜ `.claude/rules/implementation-workflow.md:29` 到 `:41`（节标题到「复用要在收尾报告里写明」那一段；第 43 行「不许拿「我只改了文档」当理由」留着）｜ 旧〔## 复用上一次全量门禁的判定：按「那一道读的输入变没变」判，不按「改了哪个目录」判〕 ｜ 新：第 29–41 行整段换成下面三段（第 43 行原样接在后面）：
  - 〔## 复用重型门禁的判定：按每条判定的单元键判，不按「改了哪个目录」或「哪棵树绿过」判〕
  - 〔改动之后一条判定要不要重判，判据是**它的单元键在判定库里有没有键相同、作数的判定**：键只由这条判定依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本与它自己的内容输入组成（`.claude/kb/decisions/13-验证路线.md` 已定项 17）。`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation、checker-tier-research-build-and-replay 都按判定入库，整道、整格没有复用判定；没有判定库的阶段每次照跑。〕
  - 〔复用由阶段自己做、自己报：每道的成功句报「现判 N、复用 M」，作废的判定报得出是哪几个函数单元变了（键的组成清单随判定落盘）。收尾报告抄每道的这一句；没有这一句的那一道按没跑算（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）。〕 ｜ 量（句）/ 推（L14、L15）
- D12 ｜ 并入 D11 ｜ `.claude/rules/implementation-workflow.md:31` ｜ 整句随第 29–41 行换掉 ｜ — ｜ 量
- D13 ｜ 并入 D11 ｜ `.claude/rules/implementation-workflow.md:38` ｜ 整行随第 29–41 行换掉（「两棵树比」那一行删） ｜ — ｜ 量
- D14 ｜ 并入 D11 ｜ `.claude/rules/implementation-workflow.md:39` ｜ 同上 ｜ — ｜ 量
- D15 ｜ 并入 D11 ｜ `.claude/rules/implementation-workflow.md:41` ｜ 同上（「复用了哪几道、引的是哪一次跑、比对了哪些路径」改成 D11 第三段的「现判 N、复用 M」）｜ — ｜ 量
- D16 ｜ 照改 ｜ `.claude/rules/implementation-workflow.md:52` ｜ 旧〔再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」）〕 ｜ 新〔再逐条核崩溃枚举用例的崩溃状态块在判定库里都有单元键相同的判定，缺块的报「本次未跑」）〕 ｜ 量
- D16（同一行第二处）｜ 照改 ｜ `.claude/rules/implementation-workflow.md:52` ｜ 旧〔先起广谱变异驱动把库目录换到这一次的暂存树、`--changed-since HEAD` 只现判改到的，`.claude/skills/crash-test/commit-stages.md` 第 2b 步〕 ｜ 新〔先起广谱变异驱动不带参数剪枝跑：每条变异按单元键沿用库里的结论、对不上的现判，`.claude/skills/crash-test/commit-stages.md` 第 2b 步〕 ｜ 量（句）/ 推（L22）
- D17 ｜ 照改 ｜ `.claude/rules/implementation-workflow.md:53` ｜ 旧〔（HEAD + 暂存区的 worktree 里，逐条按输入复用）〕 ｜ 新〔（HEAD + 暂存区的 worktree 里，逐条用例按崩溃状态块的单元键复用）〕 ｜ 量
- D18 ｜ 照改 ｜ `.claude/rules/implementation-workflow.md:74` ｜ 旧〔本机没有作数判定库记录的行两堆均分、两台各判一堆，第二台的判定库记录连同覆盖记录经 `scripts/mutation/crates-mutation-rows.py import` 核过判法版本、行键，再照本机此刻的源码过一遍按覆盖复用的判据（判据在它文件头「复用判据」）才导入；驱动退出之后本机整张判一遍（按覆盖复用得上的复用、别的现跑；只读看会怎么判用它的 `plan <根>`），全绿标记由本机写。〕 ｜ 新〔本机库里没有键相同判定的行两堆均分、两台各判一堆，第二台判出的行连同覆盖记录经 `scripts/mutation/crates-mutation-rows.py import` 按本机此刻源码现算的行键逐行核过才并进本机那一份库（第二台不留长期库，导入完即弃）；驱动退出之后本机整张判一遍（行键相同的复用、别的现跑；只读看会怎么判用它的 `plan <根>`）。〕 ｜ 量（句）/ 推（L21）
- D19 ｜ 照改 ｜ `.claude/rules/implementation-workflow.md:76` ｜ 旧〔按它这批输入的指纹核那一格全绿标记（两条流的崩溃点重放全量都要 `exhaustive=true`），不作数的报「本次未跑」、不判红〕 ｜ 新〔核它的崩溃状态块在崩溃判定库里都有单元键相同、不红的判定（两条流的崩溃点重放全量都要 `exhaustive=true`），缺块的报「本次未跑」、不判红〕 ｜ 量
- D19（同一行第二处）｜ 照改 ｜ `.claude/rules/implementation-workflow.md:76` ｜ 旧〔逐条按输入复用、只重跑输入变了的〕 ｜ 新〔逐条用例按块的单元键复用、只现判键对不上的块〕 ｜ 量

### `.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md`、`.claude/skills/crash-test/SKILL.md`（底稿 L14、L15、L17、L22）

- D20 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:8` ｜ 旧〔- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。〕 ｜ 新〔- 这次改动的范围：`git diff --stat -- crates litmus` 原样（只作记录；跑哪几条由各道按单元键现算，不由这份范围定）。〕 ｜ 量
- D21 ｜ 照改（第 1 步前半整段换）｜ `.claude/skills/crash-test/commit-stages.md:19` ｜ 旧〔每道开跑前先 `python3 scripts/gate/admission.py stage-marker-check <根> <阶段文件名>`，退 0 就抄它打的 ok 行、这一道记「复用」不跑（判绿的阶段自己写标记，整轮门禁与下一趟按它复用）；要跑的再取它自己的输入（checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay、checker-tier-formal-verification 是 `.claude/gate.d/stage-inputs.tsv` 里它那一行的路径，checker-tier-lkmm 是 `litmus crates .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh`，四道都另加它自己的阶段脚本 `.claude/gate.d/<文件>`），跑 `git diff --quiet -- <这些路径>`（要退 0）〕 ｜ 新〔每道开跑前先核工作区等于暂存区：跑 `git diff --quiet -- crates litmus Cargo.toml Cargo.lock .cargo scripts .claude/gate.d .claude/scripts`（要退 0；这是这几道单元闭包与入口可能落到的全部源码根，复用由道内按单元键做，这一步不查标记、不记「复用」）〕 ｜ 量（句）/ 推（路径集合是设计判断，第八节）
- D21（同一行第二处）｜ 照改（第 1 步后半，广谱那一支整段换）｜ `.claude/skills/crash-test/commit-stages.md:19` ｜ 旧〔checker-tier-broad-mutation 判的是暂存区（git 索引）上的源码，不是工作区：开跑前不起 `admission.py stage-marker-check`，起 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh --marker-check`〕 ｜ 新〔checker-tier-broad-mutation 同样判工作区那一份、开跑前做同一样核对（广谱变异驱动读此刻源码，第 2b 步）；核对过了照第 2b 步起驱动，不起 `--marker-check`〕 ｜ 量（句）/ 推（L22）。这一句之后到「那一行原样抄进产出那张表。」为止（`--marker-check` 两支的「✗」「→」、退 0 / 3 的读法、「判绿之后写全绿标记之前…」）整段删。
- D22 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:22` ｜ 旧〔再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例各自那一格全绿标记，不作数的它报「本次未跑」、不判红，原样抄进产出那张表）〕 ｜ 新〔再逐条核 `.claude/gate.d/stage-inputs.tsv` 里键是 `crash-case:` 的用例的崩溃状态块在崩溃判定库里都有单元键相同的判定，缺块的它报「本次未跑」、不判红，原样抄进产出那张表）〕 ｜ 量
- D22（同一行第二处）｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:22` ｜ 旧〔它逐条跑登记的崩溃枚举用例，这批输入那一格全绿标记在的复用、不在的才跑，不用你挑哪几条；要从头跑时加 `--start-over`〕 ｜ 新〔它逐条跑登记的崩溃枚举用例，每块按单元键复用、键对不上的才现判，不用你挑哪几条；要不读库、全部现判时加 `--start-over`（现判的照键写回同一份库）〕 ｜ 量
- D23 ｜ 照改（第 2b 步整块换，D24–D26 并入这一条）｜ `.claude/skills/crash-test/commit-stages.md:23` 到 `:28` ｜ 旧〔在主工作区按次序做四步：〕 ｜ 新：第 23–28 行换成下面五行（缩进照原文三格）：
  - 〔   2b. checker-tier-broad-mutation 等 checker-tier-crates-mutation-replay 跑完再起，在主工作区按次序做三步：〕
  - 〔   ① 起驱动之前在会话 shell 里跑 `SINGLEFS_HEAVY_TESTS=commit python3 scripts/mutation/broad-mutation.py --unit-environment`，原样抄它打的那一行（会话 shell 里工具链与构建环境认的变量写成的 `env 名=值 …`，例 RUSTUP_TOOLCHAIN 换了工具链的；空行是一个都没有）；〕
  - 〔   ② 再起 `SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label broad-mutation-pruned --log <草稿目录>/broad-mutation-run.log -- <它打的那一行，空行就不写> nice -n 19 python3 scripts/mutation/broad-mutation.py --threads <「跑之前定下来」定的线程数> --memory-cap <「跑之前定下来」定的上限>`（不带选条参数就是剪枝跑全部条：每条变异现算单元键，库里有键相同结论的沿用，没有的现判，连同库里超时与内存撞顶的；超时、撞顶的那一步驱动当场升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到、登记进库里，判法在驱动文件头「升档」）：一趟以小时计、跑得过 Bash 后台时限，照崩溃放量全量那样经 `detached-run.sh start` 起、会话里用 run_in_background 起 `bash scripts/hosts/detached-run.sh watch` 盯，跑完由 `detached-run.sh list` 读退出码；驱动每条命令自己经内存包装，外面不再包一层；被停（`detached-run.sh stop broad-mutation-pruned`、或单元被停）驱动退 143、在途那一条不落行，原样记下；重起同一条 ② 接着跑（已有定论的条按键沿用、不重跑：超时、内存撞顶不算定论，照样再选中重跑；被打断的那一条重跑）；〕
  - 〔   ③ `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh`，整份输出写进 `<草稿目录>/checker-tier-broad-mutation.log`（只读库目录、起驱动的 `--summary`；只问版本地起 `rustc -Vv`、`cargo nextest --version`、`cargo mutants --version` 与 `llvm-profdata --version`，不编不跑，外面不包内存包装）。〕
  - 〔   ①② 退非 0 时原样抄驱动的「✗」「→」与退出码（含义在驱动文件头「退出码」一段），不换参数重跑：退 4（库目录的锁被别的一趟占着）抄 `ps -o pid,args -u "$(id -u)"` 里那一趟驱动的进程行，不等、不抢；别的退出码照驱动文件头的出路办（会话 shell 里设的编译变量或换工具链的变量没照 `--unit-environment` 带进单元的，带上重起 ②）；② 退 143 照 ② 那一句重起。不论 ② 退几，③ 照跑、原样抄它的判定。③ 列的 NO_CONCLUSION 里还有超时、内存撞顶的（升档证据不齐，那几条结果行的备注写缺哪一样）：加大 `--timeout-floor` 或 `--memory-cap`（都不进键），照 ② 的起法（同一行 `--unit-environment`、另起一个 `--label`）加 `--item <序号> --rerun` 点名重跑（序号取同一刻 `--list-items` 打的），跑完再 ③；③ 红在「升档判的抓到复核不过」的，照它点的那一样查，再同样 `--item <序号> --rerun` 重判那几条。〕 ｜ 量（句）/ 推（L22：驱动删 `--sync-tree`、`--changed-since`，退出码 7「全局键与这一版不同」随全局键删，新退出码表由改动批定）
- D24 ｜ 并入 D23 ｜ `.claude/skills/crash-test/commit-stages.md:24`（① 包暂存树）｜ 整行删 ｜ — ｜ 量
- D25 ｜ 并入 D23 ｜ `.claude/skills/crash-test/commit-stages.md:25`（② `--sync-tree --commit`）｜ 整行删 ｜ — ｜ 量
- D26 ｜ 并入 D23 ｜ `.claude/skills/crash-test/commit-stages.md:26`、`:28`（③ `--changed-since HEAD`、退 7、「① 之后再取一次 `git write-tree`」）｜ 随 D23 换掉 ｜ — ｜ 量
- D27 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:30` ｜ 旧〔`checker-tier-crash-point-replay` `--full` 逐条用例各抄一行（「复用」「判绿」连同它记下的行、「✗」连同紧跟的「→」），末句的判绿、复用、判红各几条原样抄。〕 ｜ 新〔`checker-tier-crash-point-replay` `--full` 逐条用例各抄一行（现判块数、复用块数、判绿连同它记下的行，或「✗」连同紧跟的「→」），末句的现判、复用、判红各几块原样抄。〕 ｜ 量（句）/ 推（输出行的新形态）
- D27（同一行第二处）｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:30` ｜ 旧〔③（两台输入指纹相同）〕 ｜ 新〔③（两台工具链相同、两片的块键各自现算）〕 ｜ 量（句）/ 推（双机驱动的 ③ 行跟着 L12 改）
- D28 ｜ 照改（只删一句；指纹核对本身不改）｜ `.claude/skills/crash-test/commit-stages.md:32` ｜ 旧〔；跑 checker-tier-broad-mutation 的那一趟两条命令的路径再加 `.cargo`，另记第 2b 步 ① 的树号与提交号〕 ｜ 新〔；跑 checker-tier-broad-mutation 的那一趟两条命令的路径再加 `.cargo`〕 ｜ 量。第 6 步其余不改，理由：它核的是「几道绿对应的是不是同一份源码」，不是复用键，不读它不影响任何判定沿不沿用；若主 agent 定连 `git rev-parse HEAD` 也去掉，换成 `git ls-files -s -- crates litmus Cargo.toml Cargo.lock scripts | sha256sum` 加未跟踪那一条即可（推）。
- D29 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:36` ｜ 旧〔与 `checker-tier-crash-point-replay` 写进 git common-dir 的全绿标记、续跑的进度文件是阶段本身的行为；广谱变异驱动写的库目录（`${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}`）是驱动本身的行为，第 2b 步 ① 写进对象库的那一个提交对象是那一步本身；〕 ｜ 新〔与各道写进自己判定库的判定（崩溃判定库、crates 变异库、形式化库、QEMU 库、LKMM 库、研究复跑库，库目录写在各道文件头）是阶段本身的行为；广谱变异驱动写的库目录（`${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}`）是驱动本身的行为；〕 ｜ 量（句）/ 推（各库落点由改动批定）
- D30 ｜ 照改 ｜ `.claude/skills/gate/staged-gate.md:16` ｜ 旧〔（用户要求时 `=user-request`；它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`；复用判据在 `scripts/gate/stage-must-run.sh` 文件头）〕 ｜ 新〔（用户要求时 `=user-request`；它跑 `gate.sh --staged`）〕 ｜ 量（句）/ 推（`refs/sop/staged-green` 删不删见 D32）
- D30（同一行第二处）｜ 照改 ｜ `.claude/skills/gate/staged-gate.md:16` ｜ 旧〔这几道重阶段在 `gate.sh` 里照各自的复用判定走（`checker-tier-crash-point-replay` 跑快档并核全绿标记，标记不作数报「本次未跑」不判红，`.claude/rules/verification.md`）〕 ｜ 新〔这几道重阶段在 `gate.sh` 里照跑、道内按每条判定的单元键复用（提交前单跑过的那几道判定已在库里，整轮里只现判键对不上的；`checker-tier-crash-point-replay` 跑快档并核崩溃枚举用例的崩溃状态块，缺块报「本次未跑」不判红，`.claude/rules/verification.md`）〕 ｜ 量
- D31 ｜ 照改 ｜ `.claude/skills/gate/staged-gate.md:17` ｜ 旧〔红句说的是「这一批输入」本身的（`checker-tier-crash-point-replay` 的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是另列的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径、或碰了 `checker-tier-crash-point-replay` 脚本本身（`.claude/gate.d/checker-tier-crash-point-replay.sh`；全绿标记的键还含它与 `cargo -V`、`rustc -V`） ⇒「这一轮」，下一步原样抄 `checker-tier-crash-point-replay` 出路里的三行命令，一条都没碰 ⇒「不是这一轮」〕 ｜ 新〔红句说的是某几块判定缺了的（`checker-tier-crash-point-replay` 的「这条用例有 N 块没有单元键相同的判定」这类）不按它点名的库路径判：阶段打的作废单元清单（缺块的键与库里旧键相比变了的那几个函数单元，连同它们所在的文件与行）落在暂存区 diff 的块里 ⇒「这一轮」，下一步原样抄 `checker-tier-crash-point-replay` 出路里的命令；一个都不在（只换了工具链或依赖版本的也算）⇒「不是这一轮」〕 ｜ 量（句）/ 推（作废单元清单是 L1 改完才有的输出）
- D32 ｜ 照改 ｜ `.claude/skills/gate/staged-gate.md:22` ｜ 旧〔、`gate-staged.sh` 整轮全绿时 `update-ref refs/sop/staged-green`〕 ｜ 新〔〕（删掉这一段）｜ 量（句）/ 推：今天读 `refs/sop/staged-green` 的只有 `scripts/gate/admission.py` 的 gate-reuse（`git grep -n staged-green -- scripts .claude/scripts .claude/singlefs-ai-sop/scripts` 命中 `admission.py:97,265,3990,4442`、`change-touches-crates.sh:15` 注释与 `gate-staged.sh` 写它的几行），gate-reuse 删了就没人读，`gate-staged.sh` 那一步 `update-ref` 随之删；要留给别的用途由主 agent 定
- D33 ｜ 照改 ｜ `.claude/skills/crash-test/SKILL.md:21` ｜ 旧〔加逐条核崩溃枚举用例的全绿标记（不作数的报「本次未跑」，不判红）；全量 `--full` 在 HEAD + 暂存区的 worktree 里逐条按输入复用〕 ｜ 新〔加逐条核崩溃枚举用例的崩溃状态块在判定库里都有单元键相同的判定（缺块的报「本次未跑」，不判红）；全量 `--full` 在 HEAD + 暂存区的 worktree 里逐条用例按块的单元键复用〕 ｜ 量
- D34 ｜ 照改 ｜ `.claude/skills/crash-test/SKILL.md:38` ｜ 旧〔全量那几条有没有作数的全绿标记看门禁 checker-tier-crash-point-replay 的输出〕 ｜ 新〔全量那几条的崩溃状态块是不是都有单元键相同、不红的判定，看门禁 checker-tier-crash-point-replay 的输出〕 ｜ 量

### `.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/*.md`（改这几份走三方或登记用户豁免）

- D35 ｜ 照改 ｜ `.claude/main-agent.md:10` ｜ 旧〔里程碑收尾、动了崩溃点重放或判器的那一批（暂存改动碰到 D13（验证路线） 已定项 9 那张路径清单）提交之前〕 ｜ 新〔里程碑收尾、动了判器的那一批（崩溃判定库按单元键现算出有判器单元变了、作废块数不为 0，D13（验证路线） 已定项 9）提交之前〕 ｜ 量（句）/ 推（L1、L29）
- D36 ｜ 照改 ｜ `.claude/main-agent.md:94` ｜ 旧〔再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）〕 ｜ 新〔再逐条核崩溃枚举用例的崩溃状态块在判定库里都有单元键相同的判定，缺块的报「本次未跑」不判红）〕 ｜ 量
- D36（第二处）｜ 照改 ｜ `.claude/main-agent.md:94` ｜ 旧〔判绿按输入哈希写全绿标记），其余五道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（〕 ｜ 新〔每块按单元键复用、现判的写回崩溃判定库），其余五道同样带前缀、同样在道内按每条判定的单元键只现判键对不上的（〕 ｜ 量
- D36（第三处）｜ 照改 ｜ `.claude/main-agent.md:94` ｜ 旧〔先起广谱变异驱动把库目录换到这一次的暂存树、只现判改到的（照〕 ｜ 新〔先起广谱变异驱动不带参数剪枝跑（照〕 ｜ 量
- D36（第四处）｜ 照改 ｜ `.claude/main-agent.md:94` ｜ 旧〔（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这六道在 `gate.sh` 里照各自的复用判定走：全绿标记或复用判定作数的不跑，不作数的——提交前没单跑那一道、跑过之后那一道的输入或环境变了——整轮里真跑那一道，`checker-tier-crash-point-replay` 照跑快档、逐条核全绿标记；整轮里不另外直接调这六道）〕 ｜ 新〔（它跑 `gate.sh --staged`）并分诊（这六道在 `gate.sh` 里照跑、道内按单元键复用：提交前单跑过的那几道判定已在库里，整轮里只现判键对不上的，`checker-tier-crash-point-replay` 照跑快档、逐条核崩溃枚举用例的崩溃状态块；整轮里不另外直接调这六道）〕 ｜ 量
- D37 ｜ 不改 ｜ `.claude/agent-common.md:56`「crates 变异整表」 ｜ — ｜ — ｜ 理由：说的是重型那一批跑的范围（整张表），不是复用单位；按行剪枝写在 `verification.md` 与 D5 的新句里。量
- D38 ｜ 不改 ｜ `.claude/agents/implementation-writer.md:17` ｜ — ｜ — ｜ 理由：只指向 `verification-build.md`「八、…」；那一节按 D77 补了 2026-10-15 定案之后这句就对。量
- D39 ｜ 不改 ｜ `.claude/agents/implementation-writer.md:40`、`:63` ｜ — ｜ — ｜ 理由同 D37（整表复跑、crates 变异整表说的是范围）。量
- D40 ｜ 照改 ｜ `.claude/agents/experiment-designer.md:35` ｜ 旧〔退 77（输入自上次产物以来没变）就不写重跑登记〕 ｜ 新〔退 77（这一实验的单元键与上次产物头上记的相同）就不写重跑登记〕 ｜ 量（句）/ 推（L26）
- D41 ｜ 不改 ｜ `.claude/agents/experiment-designer.md:34`、`.claude/agents/experiment-runner.md:16`、`:33` ｜ — ｜ — ｜ 理由：引的是上游节标题「判定可复用：按输入存，按前缀剪」，标题不改；上游改句（第五节）之后这三处自动对。量
- D42 ｜ 并入 N24（实验变异表按条入库）｜ `.claude/agents/mutation-triage.md:33` ｜ 旧〔确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；〕 ｜ 新〔确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」，并核它的「现判 N、复用 M」两数之和等于表的条数；〕 ｜ 推（L23 改完 `mutate.sh` 才打这两个数；改动批合入前不换）
- D43 ｜ 不改 ｜ 其余定义 ｜ — ｜ — ｜ 理由：这一轮现查 `grep -n -E '复用|剪枝|全绿|判定库|changed-since|按输入存|整表|输入没变|输入变了|admission.py experiment' .claude/agents/*.md` 只命中上面几处（mutation-triage、experiment-designer、implementation-writer、experiment-runner），别的定义 0 处。量

### `.claude/kb/decisions/13-验证路线.md` 与 `.claude/kb/decisions.md` 索引（写回走 kb-scribe，整段正文在第六节）

- D44 ｜ 照改 ｜ `.claude/kb/decisions/13-验证路线.md:254` ｜ 旧〔「动了崩溃点重放或判器」按暂存改动碰到的路径判：`crates/singlefs-checker/`、`crates/singlefs-checker-tier/src/` 下 `crash*`、`crash_point_replay_progress`、`verdict_store`、`crash_identity`、`crash_judge_*`、`crates/singlefs-replay-prefix/`。〕 ｜ 新〔「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。〕 ｜ 量（句）/ 推（L1、L29）
- D45 ｜ 照改 ｜ `.claude/kb/decisions/13-验证路线.md:17`（索引第 9 行）｜ 旧〔里程碑收尾与动了崩溃点重放或判器的那批提交前弹窗请用户跑全量档〕 ｜ 新〔里程碑收尾与判器单元有变的那批提交前弹窗请用户跑全量档〕 ｜ 量。第 253 行同义正文不改（定义在 254 行，改了 254 就够）
- D46 ｜ 照改 ｜ `.claude/kb/decisions/13-验证路线.md:396` ｜ 旧〔几类测试各一份的长期判定库与两段协议〕 ｜ 新〔一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议〕 ｜ 量（句）/ 推（L12）；另新立已定项 17，正文在第六节
- D47 ｜ 照改 ｜ `.claude/kb/decisions/13-验证路线.md:410` ｜ 旧〔片起点是判定库里的续点。〕 ｜ 新〔片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。〕 ｜ 量：今天的续点键带流名与段号——`crates/singlefs-checker-tier/src/verdict_store.rs:242` `let name_bytes = self.flow_name.as_str().as_bytes();` 与 `:248` `key_bytes.extend_from_slice(&self.segment_index.to_be_bytes());`（第一轮记「推」，这一轮现读改判「量」）；改法归 crates 那一批（L3、L12），这一句随它改
- D48 ｜ 照改（机械）｜ `.claude/kb/decisions.md:52`、`:244` ｜ 新立已定项 17 之后跑 `bash .claude/gate.d/doc-decisions.sh --write` 重生成（「已定 17 项」「分项 17」与已实现计数）｜ — ｜ 量
- D49 ｜ 空号 ｜ — ｜ — ｜ — ｜ 第一轮本来就空

### `.claude/kb/checks-owed.md`（写回走 kb-scribe；欠账行的「怎么改」「前置」两格换句，正题一格不动的写明）

- D50 ｜ 照改 ｜ `.claude/kb/checks-owed.md:22`（C8）｜ 旧〔**已有（按阶段算输入）**：`scripts/gate/stage-must-run.sh` 拿 `refs/sop/staged-green`（上次整轮全绿那棵暂存树）与这一次的暂存树做 git diff 当指纹，接在 checker-tier 七道与 harness-model-differential-and-scenarios 上（`git grep -l stage-must-run -- '.claude/gate.d/*.sh'` 2026-10-11 现数 9 份，另一份 code-tooling 只跑它的 `--selftest`），同一轮里第二、三次只要那几条路径没变就退 77；粗粒度前身 `scripts/gate/change-touches-crates.sh` 降为第二道闸，基准恒取 `HEAD`（判别力自证是第 6 格：基准改回 `GATE_BASE`，那一格必须判红）。〕 ｜ 新〔**已有（按判定算单元键）**：七道重型与 harness-model-differential-and-scenarios 的每条判定按单元键入库（D13（验证路线） 已定项 17），一轮里第二、三次只现判键对不上的；按树比的 `scripts/gate/stage-must-run.sh` 与按 `HEAD` 比的 `scripts/gate/change-touches-crates.sh` 两条路已删。〕 ｜ 量（句）/ 推（L15、L18 改完才成立：改动批合入那一次提交里改）。正题、「仍欠的是本行正题那一半」与 2026-09-21 实测不动（实测是那一次的事；仍欠的「按布局算受影响集合」不由单元键还清）
- D51 ｜ 照改 ｜ `.claude/kb/checks-owed.md:354`（C528）｜ 旧〔；checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-crates-mutation-replay、harness-model-differential-and-scenarios、checker-tier-research-build-and-replay、checker-tier-lkmm、checker-tier-formal-verification、checker-tier-broad-mutation 八道接了 `scripts/gate/stage-must-run.sh`，取相对 `refs/sop/staged-green` 的变没变，是第三套算法〕 ｜ 新〔〕（删掉这一段，豁免表那一类随之没有）｜ 量（句）/ 推（L15）
- D52 ｜ 照改 ｜ `.claude/kb/checks-owed.md:370`（C564）｜ 旧〔③ 全绿标记：git common-dir 下 `singlefs-crash-case-green.*` 现只有 `new-pool-file-creation-stream` 一份，其余 12 条登记的用例没有一份在输入指纹上作数的全绿标记（现数）〕 ｜ 新〔③ 判定：登记的用例里哪几条的崩溃状态块在崩溃判定库里都有单元键相同、不红的判定，以 `checker-tier-crash-point-replay` 快档逐条核的输出为准；键改成函数单元之后旧库作废，这一样从零开始，要用户要求的全量跑一趟〕 ｜ 量（句）/ 推（L17）
- D52（同一格「怎么改」）｜ 照改 ｜ `.claude/kb/checks-owed.md:370` ｜ 旧〔全量跑到每条都有全绿标记。判别力在各条用例自己（计数断言与同文件的靶向阳性对照）；门禁侧标记在且作数就复用，没有就跑〕 ｜ 新〔全量跑到每条用例的崩溃状态块都有判定。判别力在各条用例自己（计数断言与同文件的靶向阳性对照）；门禁侧按块的单元键复用，没有就跑〕 ｜ 量
- D53 ｜ 照改 ｜ `.claude/kb/checks-owed.md:403`（C605）｜ 旧〔键层另点变异：`crates/singlefs-checker-tier/src/crash_identity.rs` 的 `each_attribute_changes_the_reuse_key`（两项属性各改一次键要变）`crates/mutations.tsv` 里没有一行点它（`grep -c each_attribute_changes_the_reuse_key crates/mutations.tsv` 输出 0），补一行「复用键少罩一项」的变异点名它红〕 ｜ 新〔键层四格自证，各有一行变异点名它红：只改一个被调函数单元，只作废依赖它的块；改一个没被调的函数，零作废；只改注释，零作废；只改被引用的常量（E164（崩溃放量新设计的小范围功能测试） 的 V11 形），必须作废〕 ｜ 量（句）/ 推（L1；这四格就是第一轮 D81 的「四格自证」）
- D54 ｜ 不改 ｜ `.claude/kb/checks-owed.md:417`（C623）｜ — ｜ — ｜ 理由：「取不出复用数的项记 `reused=unknown`」只管报数的写法，键改了它照样成立。量
- D55 ｜ 照改（改动批合入时挪进已还清）｜ `.claude/kb/checks-owed.md:421`（C629）｜ 还法一句：〔QEMU 那一道按档入库：每档的键是宿主与来宾两侧真走到的函数单元、QEMU 版本、内核镜像与 initramfs 内容摘要、工具链；换了 QEMU 或来宾内核只作废那几档，`environment=` 那一层随整道标记删〕 ｜ — ｜ 推（L19；「内核镜像与 initramfs 怎么取摘要」是底稿第三节的问，答案归 gaps / design 那一路）
- D56 ｜ 照改 ｜ `.claude/kb/checks-owed.md:425`（C635）｜ 旧〔checker-tier-crash-point-replay 逐条的全绿标记（git common-dir 的 `singlefs-crash-case-green.<用例名>.<输入指纹>`）按输入指纹认、不记日期〕 ｜ 新〔崩溃判定库每块记着判定日期与来自哪一种跑法（剪枝、`--full`），没有门禁读〕 ｜ 量（句）/ 推（「每块记日期与跑法」是新要求，第八节）
- D56（同一格「怎么改」）｜ 照改 ｜ `.claude/kb/checks-owed.md:425` ｜ 旧〔两样全量判绿时各写一份带日期与输入指纹的标记；提交前那一趟门禁核「这一批暂存改动碰了 D13（验证路线） 已定项 9 的路径清单，而标记早于这一批」时报出来〕 ｜ 新〔提交前那一趟门禁按崩溃判定库现算：这一批有没有判器单元变了、作废的块在这一批之后有没有被 `--full` 那一趟现判过，没有就报出来〕 ｜ 量（句）/ 推
- D56（同一行「前置」）｜ 照改 ｜ `.claude/kb/checks-owed.md:425` ｜ 旧〔路径清单只写在 D13（验证路线） 已定项 9 正文里，门禁要读得先登记成一份文件；〕 ｜ 新〔崩溃判定库每块要记判定日期与跑法；〕 ｜ 量（句）/ 推
- D57 ｜ 照改 ｜ `.claude/kb/checks-owed.md:432`（C645）｜ 旧〔在 HEAD + 暂存区的 worktree 里跑、逐条写全绿标记〕 ｜ 新〔在 HEAD + 暂存区的 worktree 里跑、逐块写进崩溃判定库〕 ｜ 量
- D58 ｜ 照改 ｜ `.claude/kb/checks-owed.md:448`（C668）｜ 旧〔没登记的库模块这样引不被拦，`history.rs` 也不进那条用例的判法摘要（复用键对随机历史的改动不敏感）。〕 ｜ 新〔没登记的库模块这样引不被拦。〕 ｜ 量（句）/ 推：用例级判法摘要随 L17 删、闭包按调用图收之后，「复用键对随机历史不敏感」那一半没有了；白名单那一半照旧欠
- D59 ｜ 照改 ｜ `.claude/kb/checks-owed.md:538`（C777）｜ 旧〔准入判不到它们「输入没变」〕 ｜ 新〔准入判不到它们的单元键与上次产物相同〕 ｜ 量
- D59（同一行第二处）｜ 照改 ｜ `.claude/kb/checks-owed.md:538` ｜ 旧〔（登记了准入的键要求产物头有 `E7INPUT` 指纹行）〕 ｜ 新〔（登记了准入的键要求产物头记着那一实验的单元键）〕 ｜ 量（句）/ 推（L26；产物头那一行叫什么由改动批定，同一行另两处「`E7INPUT`」随它改）
- D60 ｜ 照改（改动批合入时挪进已还清；「崩溃放量库不受影响」那一句现在就错，先删）｜ `.claude/kb/checks-owed.md:551`（C800）｜ 旧〔崩溃放量库不受影响：它的键是写表、父镜像、判法代码（`crates/singlefs-checker-tier/src/crash_identity.rs` 文件头）〕 ｜ 新〔崩溃放量库也一样：它的判法版本是整个判器闭包逐文件的词法摘要（底稿 L1），闭包里任何一份文件改一行代码整库作废〕 ｜ 量（句）/ 推（键成分转引底稿 L1，crates 路量过）。还法一句：〔复用键改成函数单元：每条判定的键只由它依赖的函数单元去注释后的指纹、工具链、解析出的依赖版本组成（D13（验证路线） 已定项 17），只改注释、改没被调的函数都不作废；`scripts/gate/lib_source_digest.py` 从按文件算摘要改成按函数单元切〕
- D61 ｜ 照改 ｜ `.claude/kb/checks-owed.md:338`（C489）｜ 旧〔已有，对今天的代码还没有一版结论〕 ｜ 新〔已有，结论按每条变异的单元键存，今天的代码有多少条有键相同的结论以驱动 `--summary` 现报为准〕 ｜ 量（句）/ 推（L22）。「2026-09-30 在 HEAD fcc4d340 的干净 worktree 数出」与「要在**独立 worktree** 上跑」不改：前者是那一次的数，后者防并发改 `crates/` 污染「无效」一栏，不是复用键
- D62 ｜ 不改 ｜ `.claude/kb/checks-owed.md:715`（C632）｜ — ｜ — ｜ 理由：工具链指纹是许可的键成分。量
- D63 ｜ 不改 ｜ `.claude/kb/checks-owed.md:784`（C795，已还清表）｜ — ｜ — ｜ 理由：已还清行记的是那一次怎么还的（史料）；mutation-tables-compile 格按条入库归 L25 那一批，改完不另立欠账。量

### 实验页、里程碑页、名词表、`verification-build.md`、本批记录

- D64 ｜ 照改（加回看，不改逐行判定）｜ `.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:149`（F4）｜ F4 那一行是整行抄的判定，不改；在同页「影响的决策」表（第 192 行表头）末尾加一行，整行见第六节 ｜ — ｜ 量
- D65 ｜ 并入 D64 ｜ `.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:10`、`:158`（W5）｜ 同一行回看里写 W5 的口径（只在按 crate 闭包下量过）｜ — ｜ 量
- D66 ｜ 不改 ｜ `.claude/kb/experiments/171-三oracle合谋测试.md:33`、`:399`、`:409` ｜ — ｜ — ｜ 理由：说的是那几趟被测树的身份（那一次的事）；复跑表不拿它当键（量：`grep -c snapshot_digest scripts/experiments/replay.sh` 输出 0）。E171 冻结修订不改（底稿第二节第 7 条）
- D67 ｜ 不改 ｜ `.claude/kb/experiments/142-新池新建文件的干跑.md:240`、`:557` ｜ — ｜ — ｜ 理由：第 240 行整行抄的是那一次 `admission.py experiment` 的原样输出，第 557 行记那一次复跑怎么判的，都是那一次的事；实验准入改成单元键（L26）之后下一次准入照新键判，页上不回填。量
- D68 ｜ 照改（加一句取代说明）｜ `.claude/kb/milestone/02-second-txn.md:483` ｜ 旧〔测试代码一改从改动点起下游全重验〕 ｜ 新〔测试代码一改从改动点起下游全重验。（2026-10-15 用户定案盖过其中两半：身份不带 `<仓内路径>::<崩溃点名称>`，只由内容定；判法不是整版属性，判器按函数单元进块键，改一个判器函数只作废用到它的块；原话在 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 批3」那一段。）〕 ｜ 量
- D69 ｜ 照改（改动批合入后）｜ `.claude/kb/milestone/03-third-txn.md:186` ｜ 旧〔checker-tier-lkmm 仍整道复用（2026-10-11 现查，报告 `~/.local/share/singlefs/work/m3-pending-optimizations-facts/report.md`）〕 ｜ 新〔checker-tier-lkmm 仍整道复用（2026-10-11 现查，报告 `~/.local/share/singlefs/work/m3-pending-optimizations-facts/report.md`）；2026-10-15 批3 起七道重型都按判定的单元键入库，LKMM 每份 litmus 一条判定〕 ｜ 推（L20 合入后才是真话；没合入前不改）
- D70 ｜ 照改 ｜ `.claude/kb/milestone/03-third-txn.md:150` ｜ 旧〔C8（门禁范围判不出来） 在放量按块复用之后是不是已由复用键还清（推的，没核）〕 ｜ 新〔C8（门禁范围判不出来） 的「按阶段算输入」那一半由判定的单元键取代（D13（验证路线） 已定项 17），仍欠的只有按布局算受影响集合〕 ｜ 量（句）/ 推（L14、L15 合入后成立）
- D71 ｜ 不改 ｜ `.claude/kb/milestone/03-third-txn.md:139` ｜ — ｜ — ｜ 理由：只指向 `crash_identity.rs`；文件改名或拆分时由改名那一批的 sweep 跟（`.claude/rules/path-moves.md`）。量
- D72 ｜ 照改（加一行登记）｜ `.claude/kb/term-renames.md:81`、`:82` 之后 ｜ 新加一行〔| 判法版本 | 判法单元 | 整串 | 2026-10-15 起没有管整库的判法版本：判一块状态、判「抓没抓到」、判一条证明的那几个函数单元各自的指纹进每条判定的单元键 |〕 ｜ — ｜ 量（句）/ 推（新名由主 agent 定，第八节）
- D73 ｜ 照改 ｜ `.claude/kb/translation-glossary.md:84` ｜ 旧〔| 判法版本 | judging-method version | 判定法バージョン | |〕 ｜ 新〔| 判法单元 | judging units | 判定法ユニット | |〕 ｜ 量（句）/ 推（随 D72）
- D74 ｜ 照改 ｜ `.claude/kb/verification-build.md:269` ｜ 旧〔下按测试类各一份长期库，跑完不删、下一趟复用：`library-by-conclusion-kind`（崩溃放量）、`crash-state-enumeration-library`（崩溃枚举用例）、`given-states-library`（崩溃注入与坏盘输入的给定状态表）、`judge-tables-pairing-library` 与 `facts-pairing-library`（两种对拍）、`amplification-unit-tests-library`（流水线单元测试，按用例标签分子目录）。没有临时库；钉「空库起跑」那一套账的用例起跑先清空自己那一份、跑完留着。〕 ｜ 新〔下一份长期库，跑完不删、下一趟复用：块键只由内容定（判器单元闭包、段写表与起步镜像摘要、段展开、工具链、解析出的依赖版本），崩溃放量、崩溃枚举用例、给定状态表、两种对拍、流水线单元测试各自要哪些块只是索引，同一块全仓只判一次；库目录名固定，不带测试类与存法名。没有临时库；钉「空库起跑」那一套账的用例在一次性副本里建库，不清长期库。〕 ｜ 量（句）/ 推（L10、L12）
- D75 ｜ 照改（原行不改，另加一节）｜ `.claude/kb/verification-build.md:278` ｜ 第 278 行是用户 2026-10-11 定案的整理，不改；在第 279 行（这张表之后、「### 用户定的（2026-10-10…）」之前）插一小节，整段见第六节 ｜ — ｜ 量
- D76 ｜ 照改（记录里补一行）｜ `records/2026-10-13-复用键与全仓改名.md:412`、`:418` ｜ 两行是当日史料，不改；在「2026-10-15 批3」那一段末尾补一行〔- 2026-10-15：第 412 行「判定库的复用键与门禁全绿标记都只读代码内容…」那一句的核法与第 418 行「对拍重贴」一题，被同日「剪枝就是单元的」定案取代：判器代码变了只作废用到变了的函数单元的块，没有整版判法版本可重贴，「对拍重贴」不立。〕 ｜ — ｜ 量
- D77 ｜ 并入 D75 ｜ `.claude/kb/verification-build.md:257`（第八节）｜ 同一小节就是「复用键」那一条用户定案；`implementation-writer.md:17` 指过来就够（D38）｜ — ｜ 量

### 上游 SOP（本仓不改，交上游会话；句子在第五节）

- D78 ｜ 照改（上游）｜ `.claude/singlefs-ai-sop/rules/test-script-discipline.md:62` ｜ 旧〔- 每一份判定按它依赖的全部输入算键存下：被测代码、判法代码、参数、工具链。键相同的直接复用，不重算。〕 ｜ 新：见第五节 ｜ 量
- D79 ｜ 不改 ｜ `.claude/singlefs-ai-sop/rules/test-script-discipline.md:21`、`:28`、`:66` ｜ — ｜ — ｜ 理由：「只剪键相同、已判过的与等价的」本身对；违反它的是驱动（广谱变异不带参数不读结论表，底稿 L22），改驱动。量
- D80 ｜ 照改（上游）｜ `.claude/singlefs-ai-sop/rules/command-safety.md:153` ｜ 旧〔有意跨轮复用的缓存（编译产物、判定库这类）不是镜像：放 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}` 下或项目定的位置，不放 `$TMPDIR`。〕 ｜ 新：见第五节 ｜ 量
- D81 ｜ 并入 D53 与第五节 ｜ `.claude/singlefs-ai-sop/rules/test-script-discipline.md:103`（「复用的键罩没罩全输入」靠 review）｜ 本仓每个判定库的四格自证（D53 的新句）兜机械那一半；上游那一句交上游一并改（第五节第三句）｜ — ｜ 量

### 钩子（逐处的行号、判法 / 注释 / 自证在第四节）

- D82 ｜ 照改 ｜ `.claude/hooks/heavy-test-guard.sh:53` ｜ 旧〔#                 --sync-tree、--changed-since HEAD，再跑那一道），不算的那几样谁都能跑。〕 ｜ 新〔#                 不带选条参数剪枝跑，再跑那一道），不算的那几样谁都能跑。〕 ｜ 量（注释）
- D83 ｜ 照改 ｜ `.claude/hooks/lib_heavy_tests.py:448`、`:437`、`:235`、`:578`、`:579` ｜ 见第四节 H03–H07 ｜ — ｜ 量
- D84 ｜ 照改 ｜ `.claude/hooks/heavy-test-guard.sh:1167`–`:1173` ｜ 见第四节 H10–H12 ｜ — ｜ 量
- D85 ｜ 照改 ｜ `.claude/hooks/lib_heavy_tests.py:3403`、`:3404` ｜ 见第四节 H13 ｜ — ｜ 量
- D86 ｜ 照改 ｜ `.claude/hooks/runner-dispatch-guard.sh:57`、`:873` ｜ 见第四节 H17–H18 ｜ — ｜ 量

### `.claude/gate.d/stage-inputs.tsv` 文件头与 `scripts/gate/stage-must-run.sh`（底稿 L14、L15、L17、L26）

- D87 ｜ 照改（文件头第 1–35 行整段换，D88–D93 并入这一条，新文件头整段在第六节末尾）｜ `.claude/gate.d/stage-inputs.tsv:5`、`:6` ｜ 旧〔#     这几条路径，在 refs/sop/staged-green 那棵树与这一次的暂存树之间，git 说变没变。〕 ｜ 新：见第六节「stage-inputs.tsv 新文件头」 ｜ 量
- D88 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:7`–`:9`（`environment=` 另进复用判定、`gate-record-environment`）｜ 新文件头里 `environment=` 改成「打出来的工具版本进这一道每条判定的键」 ｜ — ｜ 量（句）/ 推（L16、L19、L20）
- D89 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:11`、`:12`（实验：每个文件的 sha256、登记行本身）｜ — ｜ — ｜ 量
- D90 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:13`–`:17`（崩溃枚举用例全绿标记、整个 `crates/`）｜ — ｜ — ｜ 量
- D91 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:25`–`:28`（按格复用、`closure=` 按 crate 目录）｜ — ｜ — ｜ 量
- D92 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:30`–`:32`（整道阶段的键：清单自己进比对）｜ 整段删 ｜ — ｜ 量
- D93 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:34`（宁宽勿窄）｜ 换成新文件头末段「键不手列」 ｜ — ｜ 量
- D94 ｜ 照改（整份删）｜ `scripts/gate/stage-must-run.sh:16`–`:46`（连同 `scripts/gate/stage-must-run-selftest.sh`、`code-tooling.sh:269` 那一条自证、`lib_heavy_tests.py:3641` 那一格自证）｜ 七道重型与 harness-model-differential-and-scenarios 改成道内按判定复用之后，`git grep -l stage-must-run -- '.claude/gate.d/*.sh'` 列出的调用方（今天 8 道加 code-tooling 的自证）一个不剩，脚本删 ｜ — ｜ 量（调用方清单）/ 推（L15、L18 改完才成立）
- D95 ｜ 并入 D94 ｜ `scripts/gate/stage-must-run.sh:21`–`:25`（四道整道全绿标记 `singlefs-stage-green.*`）｜ 随脚本删；`admission.py stage-marker-check`、`stage-marker-write`、`gate-record-environment` 三个子命令随之删（调用方见第三节 N27）｜ — ｜ 量
- D96 ｜ 并入 D94 ｜ `scripts/gate/stage-must-run.sh:42`–`:46`（「管不到的：环境变了而内容没变」）｜ 新键里工具链进每条判定，这一段随脚本删 ｜ — ｜ 量
- D97 ｜ 照改 ｜ `.claude/gate.d/stage-inputs.tsv:36`–`:66` 数据行第二列 ｜ 门禁行与 `crash-case:` 行的第二列从路径改成入口（写法照新文件头）；`crash-case:` 各行第二列今天都是 `crates/ Cargo.toml Cargo.lock`（量：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 现数 14，逐行第二列相同），改成 `-`（入口就是第三列的 `test=`）｜ — ｜ 量（数据行）/ 推（新写法由改动批定）

### 第二节的计数与旧串核对（原样输出）

命令（草稿目录下，脚本在 `probe/`）：`python3 -I probe/count_d.py report.md` 与 `python3 -I probe/check_old.py /home/fy5090/code/singlefs report.md`（后者逐条取「旧〔〕」，在点名的文件里数出现次数，只认恰好一次）。

```
号数 97 ；各去向 {'照改': 65, '并入': 19, '不改': 12, '空号': 1} ；缺号 []
旧串核对：恰好一次 68 条，不是一次 0 条
```

（「号数 97」含 D49 空号；有去向的 96 个号 = 照改 65 + 并入 19 + 不改 12。旧串核对数的是第二、三节里全部带「旧〔〕」的条，同一号几处各算一条；第三节写完之后重跑一次，结果贴在第三节末尾。）

## 三、新找到的（第一轮没列的，N 号）

写法同第二节。

### 规则、技能、定义、登记表、说明文件里的句子

- N01 ｜ 照改 ｜ `.claude/rules/verification.md:13` ｜ 旧〔几类测试各一份的长期判定库与两段协议（`crash_library`）〕 ｜ 新〔一份长期判定库与两段协议（`crash_library`；块键只由内容定，几类测试各自要哪些块只当索引）〕 ｜ 量（句）/ 推（L12）
- N02 ｜ 照改 ｜ `.claude/rules/verification.md:46` ｜ 旧〔放量那一段经长期库 `library-by-conclusion-kind` 复用键相同、已判过的块（与重型同一份库）〕 ｜ 新〔放量那一段经崩溃判定库复用单元键相同、已判过的块（与重型同一份库）〕 ｜ 量（第一轮列在射程外，这一轮改判在射程里：库名带存法名，L12）
- N03 ｜ 照改 ｜ `.claude/rules/fs-design.md:114` ｜ 旧〔「这次 diff 到底碰了哪几条」判得出 ⇒ 只跑受影响的那些；判不出 ⇒ 全跑。〕 ｜ 新〔「这次改动作废了哪几条判定」由每条判定的单元键算得出 ⇒ 只跑键对不上的那些；单元切不开、调用图收不全的那几条拒算键 ⇒ 照跑，不静默当成没变。〕 ｜ 量（句）/ 推（L13「认不出就拒算」）
- N04 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:13` ｜ 旧〔加逐条核全绿标记，`.claude/rules/verification.md`〕 ｜ 新〔加逐条核崩溃枚举用例的崩溃状态块，`.claude/rules/verification.md`〕 ｜ 量
- N05 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:13` ｜ 旧〔要丢掉续跑的进度文件从头跑的，加「--start-over」。〕 ｜ 新〔要不读判定库、全部现判的，加「--start-over」（现判的照键写回同一份库，不另建库）。〕 ｜ 量（句）/ 推（底稿第二节第 1 条；今天 `--start-over` 每条用例另建一份空库 `crash-point-replay-start-over/<输入指纹>`，`.claude/gate.d/checker-tier-crash-point-replay.sh:19` 量）
- N06 ｜ 照改 ｜ `.claude/skills/crash-test/commit-stages.md:14` ｜ 旧〔（里程碑收尾、动了崩溃点重放或判器的那一批，D13（验证路线） 已定项 9）〕 ｜ 新〔（里程碑收尾、判器单元有变的那一批，D13（验证路线） 已定项 9）〕 ｜ 量（随 D44）
- N07 ｜ 照改 ｜ `.claude/agent-common.md:86` ｜ 旧〔派 `experiment-designer` 写准入判输入没变的重跑登记〕 ｜ 新〔派 `experiment-designer` 写准入判单元键与上次产物相同的重跑登记〕 ｜ 量（随 D86；改共用约束走三方或登记豁免）
- N08 ｜ 照改 ｜ `.claude/rules/implementation-workflow.md:74` ｜ 旧〔本机 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、第二台 1/2 各写各的判定库，第二台的库取回，本机带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 再跑一趟〕 ｜ 新〔本机 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、第二台 1/2 各判一半，第二台判出的块取回、按块键并进本机那一份崩溃判定库（第二台不留长期库），本机带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT` 再跑一趟〕 ｜ 量（句）/ 推（L12）
- N09 ｜ 照改 ｜ `.claude/gate.d/stage-owners.tsv:5` ｜ 旧〔（crash-case: 行，逐条按输入复用）〕 ｜ 新〔（crash-case: 行，逐条用例按崩溃状态块的单元键复用）〕 ｜ 量
- N10 ｜ 照改 ｜ `.claude/gate.d/stage-owners.tsv:11` ｜ 旧〔没有结论的由主 agent补跑驱动（--changed-since HEAD）〕 ｜ 新〔没有结论的由主 agent补跑驱动（不带选条参数剪枝跑）〕 ｜ 量（句）/ 推（L22）
- N11 ｜ 照改 ｜ `scripts/gate/README.md:7` ｜ 旧〔| 跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green` |〕 ｜ 新〔| 跑 `gate.sh --staged` |〕 ｜ 量（句）/ 推（随 D32）
- N12 ｜ 照改 ｜ `scripts/gate/README.md:8` ｜ 旧〔| `stage-must-run.sh`、`stage-must-run-selftest.sh`、`stage-run-or-skip.sh`、`changed-paths.sh`、`change-touches-crates.sh` | 一道阶段要不要跑、这次改动碰了哪些路径 |〕 ｜ 新〔| `stage-run-or-skip.sh`、`changed-paths.sh` | 一道阶段要不要跑、这次改动碰了哪些路径（只给静态门禁「只判新加行」这类判据语义用，不给任何判定当复用依据） |〕 ｜ 量（句）/ 推（D94；`stage-run-or-skip.sh` 还有 `.claude/gate.d/lib/stage-cells.sh` 与 code-tooling 在用，留）
- N13 ｜ 照改 ｜ `scripts/gate/README.md:10` ｜ 旧〔| `lib_source_digest.py` | 源码摘要：给一份文件算不随注释与空白变的摘要；`admission.py` 的门禁指纹（全绿标记、崩溃枚举用例）与 `stage-must-run.sh` 两棵树的比对共用 |〕 ｜ 新〔| `lib_source_digest.py` | 函数单元的切分与指纹：把 Rust、Python、shell 源码切成函数单元、各算一份不随注释与空白变的指纹；`admission.py` 各类判定的单元键共用 |〕 ｜ 量（句）/ 推（底稿第二节第 8 条：全仓共用一份切分，放哪第二轮定；落点换了这一行跟着换路径）
- N14 ｜ 并入 D87 ｜ `.claude/gate.d/stage-inputs.tsv:1`（「每道门禁阶段与每个接了准入的实验读哪几条路径」）、`:35`（「目录写成带斜杠的前缀（git diff 与 git ls-files 的路径限定都按前缀匹配）」）｜ 新文件头第一行与末段 ｜ — ｜ 量
- N15 ｜ 不改，交主 agent 定 ｜ `.claude/kb/verification-build.md:288` ｜ — ｜ — ｜ 「所以每次的轻型是可定制的；没有剪枝。」是用户 2026-10-10 定案的整理，与 `verification.md:46`「放量那一段经长期库……复用」（N02 改完同义）字面相反（第一轮列在射程外）。2026-10-15「剪枝就是单元的」盖不盖过「轻型没有剪枝」，是设计判断（第八节）。量
- N16 ｜ 照改 ｜ `.claude/kb/checks-owed.md:422`（C630）｜ 旧〔或指纹只按被判输入算〕 ｜ 新〔或只核这一道单元闭包真读到的文件〕 ｜ 量（句）/ 推
- N17 ｜ 关（改动批合入时挪进已还清）｜ `.claude/kb/checks-owed.md:557`（C836）｜ 还法一句：〔重阶段改成按判定的单元键复用之后，读 `refs/sop/staged-green` 判「输入没变、可复用」的 `scripts/gate/stage-must-run.sh` 与 `admission.py gate-reuse` 已删，这条 ref 不再撑任何复用，`gate-staged.sh` 也不再前移它；前移的树是不是被判的树不再影响任何判定〕 ｜ — ｜ 量（读者清单见 D32）/ 推（若主 agent 定留着 ref 给别的用途，这一笔改成只记那个用途）
- N18 ｜ 不改 ｜ `.claude/kb/checks-owed.md:549`（C793）、`:400`（C602）｜ — ｜ — ｜ 理由：C793 里「录入段写进了默认库 `library-by-conclusion-kind`……判段在 `layer0-enumeration-library` 里找不到清单退 101」与 C602 里「`refs/sop/staged-green` 停在 2026-09-22」都是那一次的事。量
- N19 ｜ 不改 ｜ `.claude/agents/sweep.md:38`「宁宽勿窄」 ｜ — ｜ — ｜ 理由：误命中，说的是书记员写事实表的取舍，不是键。量
- N20 ｜ 不改 ｜ `scripts/devices/snapshot-check.sh`（派发提示写的 `snapshot-check.sh` 不在 `.claude/hooks/`，现查在 `scripts/devices/`）｜ — ｜ — ｜ 理由：它核三方开工快照（第 2 行 `# admission: always 每一次调都拿此刻的工作区与 HEAD 比快照`），比 HEAD 是为了说清「工作区改了」还是「提交改了」，是证据新鲜度，不是判定复用。量
- N21 ｜ 不改 ｜ `.claude/hooks/handback-guard.sh:20`、`:124`（`--unchanged-since`）｜ — ｜ — ｜ 理由：引文核对的时刻窗口，不是复用键。量
- N22 ｜ 不改 ｜ `.claude/hooks/lib_long_run.py`（第 2 行引「判定可复用：按输入存，按前缀剪」，第 8 行 `# long-run-resume:` 的写法）｜ — ｜ — ｜ 理由：只判长活声明三样写没写齐，不判续跑靠什么键；`scripts/hosts/detached-run.sh` 自证里的样本句「结果表里已有的行跳过」是写法样本。量

### 门禁与脚本的文件头、出路句、样本（随代码一起改；逐行原文在 `appendix-header-lines.md`）

文件头描述的是代码，代码重写之前给不出逐行的定稿句；下面每份给它「复用」那一段改完之后的现状句（照底稿对应机制的重做方向写），改代码的那一批照它重写文件头，写完跑第七节的收尾核。命中行数是附录里那份文件的 `hits=`（机械扫出，含「原文」「HEAD」这类误命中）。

- N23 ｜ 照改（文件头随代码重写）｜ 七道重型门禁与模型对拍的文件头 ｜ 新的「复用」一段：
  - `.claude/gate.d/checker-tier-crash-point-replay.sh`（14 行；L1–L6、L17、L18）：〔# 复用：快档两趟里每条测试函数各是一条判定，键是它的单元键（测试函数的单元闭包、特性组合、工具链、解析出的依赖版本），只跑键对不上的那几条（`--exact` 名单，结局经 `scripts/gate/lib_libtest_result_lines.py` 逐条认）；崩溃枚举用例的判定单位是崩溃状态块，块键与库写在 `crates/singlefs-checker-tier/src/crash_identity.rs` 文件头，这一道逐条核每条用例的块在库里都有键相同、不红的判定，缺块的报「本次未跑」；`--full` 跑登记的用例、按块复用，`--start-over` 不读库、全部现判、照键写回同一份库。没有用例级标记，不比树、不比 HEAD。〕第 4 行 `# gate-stage:` 与第 9 行 `# gate-cell:` 里「核每条用例的全绿标记」「逐条照复用判定跑」换成「核每条用例的崩溃状态块」「逐条按块的单元键跑」（格名表要与 `stage_cell` 登记逐字相同，两处一起改）
  - `.claude/gate.d/checker-tier-qemu-device-streams.sh`（4 行；L19）：〔# 复用：每档一条判定，键是宿主与来宾两侧真走到的函数单元、QEMU 版本、内核镜像与 initramfs 的内容摘要、工具链；键对得上的档复用，对不上的现跑；`--item <模式>` 只跑点名的档，判出的照键写库；`SINGLEFS_GATE_FULL=1` 不读库、全部现跑、照键写回。没有整道标记，不问 stage-must-run.sh 与 change-touches-crates.sh；stage-inputs.tsv 本阶段那一行只登记三个前提与入口，不进任何键。〕
  - `.claude/gate.d/checker-tier-lkmm.sh`（3 行；L20）：〔#   ① 复用：每份 litmus 一条判定，键是 litmus 去注释后的内容、锚点指到的函数单元指纹、herd7 版本、`.cat` / `.bell` 内容；键对得上的复用，对不上的现判。〕（`tools/memory-model` 怎么进 `.cat` / `.bell` 那一项，推，取法归第三节的问）
  - `.claude/gate.d/checker-tier-crates-mutation-replay.sh`（20 行；L21）：〔# 复用：每行一条判定，行键是被改坏的函数单元指纹、原文、替换文、点名测试执行过的单元集合指纹、判「抓没抓到」的判法单元指纹、工具链、解析出的依赖版本，不带变异名、路径、内存上限、限时；没有整库的判法版本、没有版本目录；一份库，第二台判出的经 `crates-mutation-rows.py import` 逐行核键并进本机那一份。没有整道全绿标记，不问 stage-must-run.sh。〕
  - `.claude/gate.d/checker-tier-formal-verification.sh`（18 行；L24）：〔# 复用：每条（证明按函数、空真每处 requires、必红世界每行、钉表每行、工具链一项）一条判定，键是这一条涉及的 Verus 函数单元（被验 fn、它 requires / ensures 用到的 spec fn 与 proof fn）去注释后的指纹、Verus 与 vstd 版本；钉表行的键是那一行档与参数重放真走到的装置函数单元闭包、状态输入、期望逐态行、变异原文与替换文；限时不进键；空真项身份是所属函数单元指纹加函数内第几处，不带路径与行号。格不复用，不问 stage-must-run.sh。〕（「证明按函数拆项」能不能做到是底稿第三节的问，做不到这一句改成「证明一项，键是核 crate 全部函数单元的多重集」）
  - `.claude/gate.d/checker-tier-broad-mutation.sh`（67 行；L22）：〔# 判什么：此刻源码上每一条变异都有单元键相同的结论、没有一条没红（gate-verdict 格）；结论键是被变异的函数单元指纹、原文、替换文、抓它的测试执行过的单元集合指纹、工具链、cargo-mutants / nextest / llvm 版本、解析出的依赖版本。库目录里没有「一版」：不比暂存树、不比提交，没有 library-version-is-staged-tree 格，没有 `--marker-check`；驱动由主 agent 照 `.claude/skills/crash-test/commit-stages.md` 第 2b 步不带选条参数剪枝跑。〕
  - `.claude/gate.d/checker-tier-research-build-and-replay.sh`（11 行；L23、L25）：〔#   experiment-replay    每行复跑一条判定，行键是装置 main 起的单元闭包、这一行的参数、GPU 开关与特性集、期望产物内容、工具链、解析出的依赖版本，不带实验号与文件名；先算全部行的键，全复用就不编、不清场。〕〔#   mutation-tables-compile 每条变异一条判定，键是原文、替换文、包住原文的 fn 单元与它依赖的单元指纹、cargo 参数与特性、工具链、解析出的依赖版本；库在持久目录，不在 /tmp。两格整格都不复用，不问 stage-must-run.sh 与 change-touches-crates.sh。〕
  - `.claude/gate.d/harness-model-differential-and-scenarios.sh`（1 行；L18）：〔# 每格的每条测试函数一条判定，键是它的单元键（测试函数单元闭包、特性组合、工具链、解析出的依赖版本），只跑键对不上的那几条；不问 stage-must-run.sh 与 change-touches-crates.sh。〕
  ｜ 量（命中行）/ 推（新句）
- N24 ｜ 照改（文件头随代码重写）｜ 脚本文件头 ｜ 新的「复用」一段：
  - `scripts/gate/admission.py`（84 行；L14、L15、L17、L26）：〔① 单元键相同 ⇒ 不跑，沿用那一条判定。门禁各道在道内按判定入库，这个模块只给它们算单元键与查库；实验：装置入口单元闭包的单元键与跑前登记里影响结论的参数，与 research/results/ 里某份产物头记的相同就拒重跑。登记表 .claude/gate.d/stage-inputs.tsv 只答从哪些入口起算，整张表与登记行都不进键。〕没有 gate-reuse、stage-marker-check、stage-marker-write、gate-record-environment 与 crash-case 用例指纹这几支（`manifest` 子命令有没有调用方是底稿第三节的问）
  - `scripts/gate/lib_source_digest.py`（30 行）：〔按函数单元切：Rust 的 fn 项与方法、const、static、类型定义、宏，Python 的 def 与模块级赋值，shell 的函数与顶层语句段；每个单元去注释取指纹。不给一份文件算一个摘要，也不取原文。〕
  - `scripts/mutation/broad-mutation.py`（113 行）与 `scripts/mutation/mutation-prune.py`（11 行；L22）：〔清单与结论都按单元键存：每条变异过沿用判据，沿用不了才现判；不带参数就是剪枝，`--full` 只写不读。没有 `--changed-since`、`--sync-tree`、`changed-since-tree`、`notice_if_head_moved`，没有「一版」与 `git archive <提交>` 导出的版本树。函数外的项各切单元，切不开当解析缺陷修，不退整文件。判法是判结局那几个函数单元的指纹，没有手写的判法版本。〕
  - `scripts/mutation/crates-mutation-rows.py`（23 行）与 `scripts/mutation/mutation-shard-run.sh`（10 行；L21）：同 N23 crates 变异那一句；`judge-version` 子命令改名（例 `judging-units`，打判法单元清单），`records-directory` 的「版本目录」一说删
  - `scripts/mutation/mutate.sh`（6 行；L23）：第 2 行 `# admission: always` 那一句随按条入库改；新一段〔按条入库：键是原文、替换文、包住原文的 fn 单元与其依赖单元指纹、cargo 参数与特性、工具链、解析出的依赖版本；只编译那一格同一把键。〕
  - `scripts/gate/verus-proof-check.py`（18 行）、`scripts/experiments/e165-pin-red-examples.py`（13 行）、`scripts/crash/lib_verdict_library.py`（9 行；L24）：同 N23 形式化那一句；`lib_verdict_library.py` 另加〔共用库给判定算键用函数单元摘要（与 `scripts/gate/lib_source_digest.py` 同一份切法）〕
  - `scripts/experiments/replay.sh`（2 行；L25）：同 N23 研究复跑那一句
  - `scripts/crash/crash-point-replay.sh`（9 行）、`crash-amplification.sh`（7 行）、`crash-amplification-item.sh`（2 行）、`crash-amplification-lib.sh`（2 行）、`crash-point-replay-shard-run.sh`（3 行；L11、L12）：〔一份崩溃判定库，库目录固定、不随趟换、不带存法名；录入输出按录入函数单元指纹与流输入摘要入库，命中就跳过；清单与进度按内容摘要寻址。〕
  - `.claude/scripts/lkmm.sh`（3 行）：第 2 行「门禁 checker-tier-lkmm 的复用另由 stage-must-run.sh 判」换成〔门禁 checker-tier-lkmm 每份 litmus 按单元键复用，写在那一道文件头〕
  - `scripts/gate/stage-must-run.sh`（20 行）、`scripts/gate/change-touches-crates.sh`（7 行）：整份删（D94；`change-touches-crates.sh` 今天的调用方是 `checker-tier-crash-point-replay.sh:354`、`checker-tier-qemu-device-streams.sh:231`、`checker-tier-research-build-and-replay.sh:339`、`harness-model-differential-and-scenarios.sh:160` 与 code-tooling 的自证 `:260`，四道改完一个不剩）
  - `scripts/gate/gate-staged.sh`（3 行）：第 4 行「整轮全绿才把 `refs/sop/staged-green` 前移到……」与第 12 行「三条规矩（`scripts/gate/stage-must-run.sh` 的文件头写了为什么）」随 D32 改
  ｜ 量（命中行与调用方，`git grep` 现取）/ 推（新句）
- N25 ｜ 照改 ｜ `.claude/gate.d/checker-tier-broad-mutation.sh` 的出路句：第 168、173、249、260、261、449、480、499、867 行（`RUN_DRIVER_HOWTO`、`howto_run` 与几处「→」里的 `--sync-tree --commit`、`--changed-since HEAD`、`tree.source.pending`）｜ 出路一律换成〔主 agent 照 .claude/skills/crash-test/commit-stages.md 第 2b 步不带选条参数起驱动剪枝跑，再跑这一道〕；`tree.source.pending` 那两行随「一版」删 ｜ — ｜ 量（行号 `grep -n` 现取）
- N26 ｜ 照改 ｜ `.claude/gate.d/checker-tier-crash-point-replay.sh:355`、`checker-tier-qemu-device-streams.sh:232`、`checker-tier-research-build-and-replay.sh:340`、`harness-model-differential-and-scenarios.sh:161` 的「要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的……」｜ 随 `change-touches-crates.sh` 删掉；强制全判的出路改写〔要不读库、全部现判：SINGLEFS_GATE_FULL=1 再跑一次（现判的照键写回同一份库）〕 ｜ — ｜ 量
- N27 ｜ 照改（样本删换）｜ `.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/` 下 `library-version-is-staged-tree-*` 19 份（其中 `…-marker-check-clean-reused`、`…-marker-check-clean-unmarked`、`…-marker-check-staged-differs-red` 三份的 `setup.sh` 起 `admission.py stage-marker-write`，`…-marker-clean-written` 与 `…-marker-check-clean-reused` 的 `expect` 认标记）｜ 随 library-version-is-staged-tree 格与 `--marker-check` 删；gate-verdict 格另补「改一个被调函数单元，只有那几条变异要现判」「只改注释，零条要现判」两份样本 ｜ — ｜ 量（`ls -d … | wc -l` 现数 19）/ 推（新样本）
- N28 ｜ 照改 ｜ `.claude/hooks/selftest-registry.tsv:38`（`scripts/gate/change-touches-crates.sh	light`）、`:114`（`scripts/gate/stage-must-run.sh	light`）｜ 两份脚本删了，两行随之删 ｜ — ｜ 量

### 第三节的旧串核对（原样输出，第二、三节合计）

见第四节末尾（第四节也带「旧〔〕」，三节写完一起核）。

## 四、钩子与派发闸：逐处列（行号现取；「判法」= 改了会改变放行或拒绝的代码，「注释」= 文件头或代码注释，「自证」= `--selftest` 的用例，「拒绝句」= 拒绝时打给人看的话）

判法核对（量，副本里跑，命令与原样输出）：

```
$ cd <草稿目录>/repo && nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 8G python3 -I <草稿目录>/probe/classify_probe2.py <草稿目录>/repo
python3 broad-mutation.py => ['broad-mutation']
python3 broad-mutation.py --changed-since HEAD => ['broad-mutation']
python3 broad-mutation.py --sync-tree --commit HEAD~1 => ['broad-mutation']
python3 broad-mutation.py --item 7 => ['broad-mutation']
python3 broad-mutation.py --summary => []
python3 broad-mutation.py --list-items => []
python3 crates-mutation-rows.py judge-version . => []
python3 crates-mutation-rows.py judging-units . => []
python3 crates-mutation-rows.py --item 5 . => ['crates-mutation-stage']
```

探针分得出（造推翻现象的那一次）：副本里把 `BROAD_MUTATION_QUERY_ARGUMENTS` 加上 `"--changed-since"`，同一条命令的前三行变成

```
python3 broad-mutation.py => ['broad-mutation']
python3 broad-mutation.py --changed-since HEAD => []
python3 broad-mutation.py --sync-tree --commit HEAD~1 => ['broad-mutation']
```

所以：驱动删掉 `--changed-since`、`--sync-tree` 之后，不带参数与 `--item` 的起法照旧判广谱变异（主 agent 带前缀放行、子 agent 拒），钩子判法不用动；crates 变异判法模块的子命令改名不碰判法（`lib_heavy_tests.py:2741` 只认 `--item`）。要动判法的只有两种情形：驱动新加一个只读的查询参数（例「只列键对不上的条」），要加进 `lib_heavy_tests.py:577` 的 `BROAD_MUTATION_QUERY_ARGUMENTS`，否则那条只读命令被当重型拒（误拒，不漏）；弄坏开关 `broad-mutation-first-round-exemptions` 的放行集合（H07）。

| 号 | 位置 | 类别 | 现在写的（摘要，整行见附录） | 改成 |
|---|---|---|---|---|
| H01 | `.claude/hooks/heavy-test-guard.sh:50` | 注释 | 广谱变异算重型的参数里列「--sync-tree（换全局的版本指针）」 | 删「、--sync-tree（换全局的版本指针）」；`--import-results` 留不留跟驱动 |
| H02 | `.claude/hooks/heavy-test-guard.sh:53` | 注释 | 「提交时先起驱动 --sync-tree、--changed-since HEAD，再跑那一道」 | D82 的新句 |
| H03 | `.claude/hooks/lib_heavy_tests.py:74` | 注释 | crates 变异判法模块的只读子命令列「judge-version、records-directory」 | 随子命令改名（例「judging-units」）；`records-directory` 改了义就跟着改 |
| H04 | `.claude/hooks/lib_heavy_tests.py:235` | 注释（弄坏开关说明） | 「broad-mutation-first-round-exemptions（「--item 点名又带 --skip-stage-two」与 --sync-tree 照第一版放过去）」 | 「broad-mutation-first-round-exemptions（「--item 点名又带 --skip-stage-two」照第一版放过去）」 |
| H05 | `.claude/hooks/lib_heavy_tests.py:437` | 注释 | 算的参数里列「--sync-tree（换全局的版本指针）」 | 删这一项 |
| H06 | `.claude/hooks/lib_heavy_tests.py:448` | 注释 | 「（提交时先 --sync-tree、再 --changed-since HEAD 起驱动，再跑那一道）」 | 「（提交时先不带选条参数起驱动剪枝跑，再跑那一道）」 |
| H07 | `.claude/hooks/lib_heavy_tests.py:578`、`:579` | 判法（只在弄坏开关打开时走） | `BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS = BROAD_MUTATION_QUERY_ARGUMENTS \| {"--sync-tree"}` | 驱动删 `--sync-tree` 后这一半没有对象：常量与第 578 行注释删，`:2144` 那一支只留「`--item` 加 `--skip-stage-two`」 |
| H08 | `.claude/hooks/lib_heavy_tests.py:2144`–`:2146` | 判法（弄坏开关那一支） | 认 H07 的集合 | 随 H07 |
| H09 | `.claude/hooks/lib_heavy_tests.py:2151`、`:2152` | 拒绝句 | 「--sync-tree 与 --import-results 改库目录里全局的版本指针与结果表」 | 「--import-results 改库目录里的结果表」（留不留跟驱动） |
| H10 | `.claude/hooks/heavy-test-guard.sh:1167`、`:1168` | 自证 | 「主 agent带 =commit 把广谱变异库目录换到暂存树那一版：放行」，命令 `--sync-tree --commit 0123…` | 删这一格（H11 的新格顶它） |
| H11 | `.claude/hooks/heavy-test-guard.sh:1169`、`:1170` | 自证 | 「主 agent带 =commit 起广谱变异驱动只现判改到的：放行」，命令带 `--changed-since HEAD` | 格名「主 agent带 =commit 起广谱变异驱动剪枝跑：放行」，命令 `nice -n 19 python3 scripts/mutation/broad-mutation.py --threads 8 --memory-cap 16G`，期望 0 不变 |
| H12 | `.claude/hooks/heavy-test-guard.sh:1173` | 自证 | 「主 agent不带前缀起广谱变异驱动：照拒」，命令带 `--changed-since HEAD` | 命令改 `python3 scripts/mutation/broad-mutation.py`，期望 2 不变 |
| H13 | `.claude/hooks/lib_heavy_tests.py:3403`、`:3404`（连同 `:3392` 注释「这八格红」） | 自证 | 两格 `--sync-tree`、`--sync-tree --commit HEAD~1` | 两格删；`:3392` 注释改「这六格红」（量：`:3393`–`:3404` 之间开关下红的八格，删两格剩六格；`:3405` 的 `--import-results` 那一格开关下照判重型，不在八格里） |
| H14 | `.claude/hooks/lib_heavy_tests.py:3505` | 自证 | 「crates 变异判法模块打判法版本（judge-version）不算」 | 随子命令改名，例「crates 变异判法模块打判法单元清单（judging-units）不算」，argv 换名；判法不动（上面探针 `judging-units` 已判不算） |
| H15 | `.claude/hooks/lib_heavy_tests.py:3641` | 自证 | 「从 scripts/experiments 里相对着写的 stage-must-run.sh」，命令 `bash ../gate/stage-must-run.sh --selftest` | 脚本删了，换成同目录另一份登记为 light 的：`bash ../gate/changed-paths.sh --selftest`（`selftest-registry.tsv:37` 登记 light），格名跟着换 |
| H16 | `.claude/hooks/selftest-registry.tsv:38`、`:114` | 登记表 | 两份要删的脚本 | N28 |
| H17 | `.claude/hooks/runner-dispatch-guard.sh:56`、`:57` | 注释 | ⑨「退 77（输入自上次产物以来没变）就拒」 | 「退 77（这一实验的单元键与上次产物头记的相同）就拒」；判法 `designer_admission_verdict`（`:861` 起）只认退出码，不动 |
| H18 | `.claude/hooks/runner-dispatch-guard.sh:873` | 拒绝句 | 「✗ E… 的输入自上次产物以来没变（scripts/gate/admission.py experiment 退 77），不写它的重跑登记。」 | 「✗ E… 的单元键与上次产物头记的相同（scripts/gate/admission.py experiment 退 77），不写它的重跑登记。」；「→」那一句不动 |
| H19 | `.claude/hooks/runner-dispatch-guard.sh:119`、`:155`、`:1993`、`:1998`、`:2581` | 自证名、弄坏开关说明、汇总句 | 自证格名「准入:输入没变不写重跑登记」与「设计员重跑登记准入判输入没变的拒」 | 五处一起改成「准入:单元键没变不写重跑登记」「设计员重跑登记准入判单元键没变的拒」；`:119` 与 `:155` 是弄坏开关打开后应红的格名清单，与 `:1998` 的格名逐字相同才对得上，三处同一次改 |

不在这张表里、看过不用改的：`.claude/hooks/continuation-guard.sh`（命中只有第 6 行「复用 id」，说的是子 agent 会话号）、`.claude/hooks/lib_long_run.py`（N22）、`.claude/hooks/handback-guard.sh`（N21）。

## 五、上游两句（本仓不改 `.claude/singlefs-ai-sop/`，整段交上游 zh 会话）

交上游会话的一段（可原样发）：

> 本仓 2026-10-15 用户定案：剪枝的单元是函数，一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本（原话「剪枝 剪枝 剪枝 ！！！ 阻碍剪枝的 ， head 注释  ， 整文件，指纹等等方案 全部删掉重做。 剪枝就是单元的 做不到就重做。」）。上游 `rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」第一条只写「被测代码、判法代码、参数、工具链」，不说粒度，又把「参数」整个放进键，本仓据它写出了按文件、按包、带内存上限与限时的键；`rules/command-safety.md`「测试镜像一律放临时目录」里跨轮缓存那一句给判定库的默认位置是 `/tmp`，本仓研究复跑的库因此落在 /tmp、开机即丢。请在三语仓改下面四处（中文定稿如下，en / ja 照译）：
>
> 1. `test-script-discipline.md` 第 62 行整条换成：「- 每一份判定按它依赖的全部输入算键存下：被测代码与判法代码按函数单元计——去注释的函数、方法连同它引用的常量、类型、静态项与宏，各取一份指纹，键只收这一份判定真走到的那些单元；再加工具链、解析出的依赖版本与改变判定的参数。路径、名字、行号、资源上限、限时、提交与树不进键；整文件原文、文件级或包级的摘要、手写的判法版本号都不当键。收不全的那一份不算键、照跑。键相同的直接复用，不重算。」
> 2. 同一节第 68 行「剪枝要自证」之后加一条：「- 键要自证：只改一个被调的函数单元，只作废依赖它的判定；改一个没被调的函数、只改注释，零作废；只改被引用的常量，必须作废。四格各配一个弄坏开关或一条变异，进快档。」
> 3. 「门禁管哪一半」第 101 行能按字面判的那一串末尾加「、判定库的四格键自证在不在、红不红」；第 103 行「复用的键罩没罩全输入」改成「复用的键罩没罩全输入（四格自证之外的那些形态）」。
> 4. `command-safety.md` 第 153 行整句换成：「有意跨轮复用的缓存不是镜像，不放 `$TMPDIR`：编译产物放 `${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}` 下或项目定的位置；判定库放项目定的持久位置（不随开机清空，不随一趟或判法换目录），落在开机会清空的目录里拒跑。」
>
> 第 72 行「整个脚本的输入没变就不重跑，写成 `admission: inputs-changed`」本仓这一批不动（主 agent 定它不在射程），上游不必跟着改。

本仓在上游发版之前的过渡：`.claude/rules/verification.md`「两档：轻型与重型」那一节的 D6 新句已把单元键写死，本仓照它办；上游发版、副本同步之后 D41 那三处引的节标题不变，自动对上。量（四处行号与原句见附录）/ 推（新句）

## 六、kb 与变更史要写进去的整段

### D13 新立已定项 17（kb-scribe 写回；索引行、正文、变更史三处）

索引表（`13-验证路线.md` 第 7–24 行那张，第 24 行（第 16 项）之后）加一行：

〔| 17 | **判定复用的单元是什么** | 剪枝的单元是函数：一份判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或树算改动、整文件原文、文件级与包级指纹、手写版本号、路径名字进键、整道标记一律不许 **状态：已定。** **进度：未实现。** |〕

正文（第 421 行「**欠**：无。」之后、第 423 行「## 历史版本」之前）插：

```
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。GPU 与 CPU 判器的单元不同，结论不互借；显卡驱动与卡型留作块的属性，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 批3」那几段。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 没经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：全仓复用键的改造（批3 改动批）；每份判定库的四格键自证（只改一个被调单元只作废依赖它的、改没被调的函数零作废、只改注释零作废、只改被引用的常量必须作废）。
```

（依据里若要引盘上的数——只差名字的块各存各判、判法版本一换多判 3 935 242 934 态——先把 disk 路报告入库到 `research/prompts/` 再引，kb 不引草稿目录路径。推）

变更史（`.claude/kb/decisions-history.md`「## D13（验证路线）」下新开「### 2026-10-15」，放在第 6096 行「### 2026-10-13」之前）：

```
### 2026-10-15

- 新立已定项 17「判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成」；已定项 9「动了崩溃点重放或判器」从按暂存改动碰到的路径清单判改成按崩溃判定库现算的作废判器单元判；已定项 15「几类测试各一份的长期判定库」改成一份库、测试类只当索引；已定项 16 的续点改成按段写表摘要与判器单元键寻址。
  - **依据**：用户原话（2026-10-15）「剪枝 剪枝 剪枝 ！！！ 阻碍剪枝的 ， head 注释  ， 整文件，指纹等等方案 全部删掉重做。 剪枝就是单元的 做不到就重做。」「做完 对账 不用问我 怎么改 不符合要求的统统改掉 从零开始跑也再所不惜」「改完 代码 强攻 三轮 就看是不是原子化的，不是就改到是为止」（`records/2026-10-13-复用键与全仓改名.md`「2026-10-15 批3」）。
  - **改前**：`git show <改动批之前那次提交>:.claude/kb/decisions/13-验证路线.md` 已定项 9、15、16 三节（那一版没有已定项 17）。
```

`decisions.md` 第 52、244 行：`bash .claude/gate.d/doc-decisions.sh --write` 重生成（D48）。

### `verification-build.md` 第八节加的一小节（D75、D77；插在第 279 行之后、「### 用户定的（2026-10-10…）」之前）

```
### 用户定的（2026-10-15，原话在 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 批3」）

| 项 | 定案 |
|---|---|
| 复用键 | 剪枝的单元是函数：一条判定沿不沿用只看它依赖的函数单元各自去注释后的指纹、工具链、解析出的依赖版本；按提交或 HEAD 算改动、整文件原文、文件级 / 包级 / 整表级指纹、手写版本常量、路径与名字进键、没有库只有整道标记、键过窄、库放会丢的位置或两库重复判，一律删掉重做（D13（验证路线） 已定项 17） |
| 一份库 | 几类测试共用一份块级判定，各自要哪些块只是索引；盖过上表「几类测试分开做、按各自的特性存储与复用」里「分开存储」那一半，「分开做」（各自的跑法与判据）照旧 |
| 不做迁移 | 改键让旧库作废不算代价，不做兼容迁移、不为旧行补签，改完重跑 |
```

### E164 页「影响的决策」表加的一行（D64、D65；已定项 17 写进去之后再加，关系词只认「支撑 / 推翻 / 备料 / 不影响」，`.claude/gate.d/doc-experiments.sh:735`）

〔| D13（验证路线） 已定项 17 | 备料 | 2026-10-15：F4 量到只按函数取指纹、不连引用的常量一起收会漏 V5、V11（V11 只改常量 `DATA_POINTER_BYTES`），已定项 17 的函数单元因此连同引用的常量、类型、静态项与宏一起算；改完拿 V5、V11 当「必须作废」的样本把 F4 那一格重跑一次。W5「崩溃枚举节点省不下」只在按 crate 闭包下量过，按函数单元键没量，别处不拿它当剪枝省不下的依据 |〕

### checks-owed 十三个 C 号一览（细节在第二、三节对应的号）

| C 号 | 去向 | 改哪一格 | 号 |
|---|---|---|---|
| C8 | 改「已有」一段；正题与仍欠不动 | 问题格 | D50 |
| C489 | 改一句 | 问题格 | D61 |
| C528 | 删豁免表里八道那一类 | 怎么改格 | D51 |
| C564 | 改 ③ 与「怎么改」 | 问题格、怎么改格 | D52 |
| C605 | 键层那一句换成四格自证 | 怎么改格 | D53 |
| C629 | 改动批合入时挪进已还清 | 整行 | D55 |
| C630 | 改出路里的改法 | 怎么改格 | N16 |
| C635 | 改问题格、怎么改格、前置格各一句 | 三格 | D56 |
| C645 | 改一句 | 问题格 | D57 |
| C668 | 删「判法摘要」那半句 | 问题格 | D58 |
| C777 | 改两句 | 问题格、怎么改格 | D59 |
| C800 | 「崩溃放量库不受影响」现在就改；改动批合入时挪进已还清 | 问题格、整行 | D60 |
| C836 | 改动批合入时关（挪进已还清） | 整行 | N17 |

不改的：C602、C793（史料，N18）、C623、C632（D54、D62）、C795（已还清，D63）。另有 C604、C624 只说「剪枝自证进快档」，键改了照样成立，不改。是否另立欠账：不立——四格自证、组成清单落盘、每块记日期与跑法都在改动批里做；改动批交回时没做完的，再按 `checks-owed.md` 下一个没用过的号立（今天现查最大是 C837：`grep -oE '^\| C[0-9]+ ' .claude/kb/checks-owed.md | tr -dc '0-9\n' | sort -n | tail -1` 输出 837）。

### `.claude/gate.d/stage-inputs.tsv` 新文件头（D87–D93、N14；第 1–35 行整段换成下面这些，第 18–24 行崩溃点重放合表与第三列字段的说明原样保留，接在「崩溃枚举用例」一段后面）

```
# 准入登记表：每道门禁阶段、每一格、每条崩溃枚举用例与每个接了准入的实验从哪些入口算单元闭包、开跑前要满足哪些条件（门禁与实验共用这一份，唯一登记位）。一行一条：
#   <键><制表符><入口，空格分隔；没有就写 -><制表符><准入条件与跑法字段，空格分隔><制表符>#<为什么是这几个入口>
#
# 判法都在一个模块里：scripts/gate/admission.py（文件头写全了各子命令、入口与条件的写法、退出码）。这份表不进任何判定的键：
#   每条判定的键是它自己的单元键——从入口沿调用与引用收到的函数单元各自去注释后的指纹、工具链、解析出的依赖版本，
#   加这条判定自己的内容输入（D13（验证路线） 已定项 17）。改这份表只改「从哪儿起算」，不让任何判定作废；作废只看算出来的单元键变没变。
#   门禁阶段（键是 .claude/gate.d/ 下的文件名）与阶段里的一格（键是 <阶段文件名>:<格名>）：第二列写这一道 / 这一格判定的入口
#     （测试函数、装置 main、被验的 Verus 函数、litmus 文件，写法见 admission.py 文件头）；第三列 command=<可执行名>、readwrite=<路径>、
#     probe=<仓内脚本>[:<参数>…] 是开跑前要齐的前提，阶段经 admission.py gate-preconditions 判，没齐判红（不退 77）；
#     environment=<仓内脚本>[:<参数>…] 打出来的工具版本（QEMU、herd7、Verus 这类不在 cargo 工具链里的）进这一道每条判定的键；
#     closure=<包>:<bin> 指一个 bin 的 main 当入口，单元闭包由 admission.py 现算，不按 crate 目录收。阶段与格本身不复用，复用在每条判定上。
#   实验（键是实验号；同一个实验另有一种调用方式的写 E<号>/<方式>）：装置开跑前调 admission.py experiment：
#     装置入口单元闭包的单元键与跑前登记里影响结论的参数，research/results/ 里有一份产物头上记着同一把键 ⇒ 拒绝重跑；
#     第三列的准入条件没齐 ⇒ 拒绝开跑。入口写成 @<别的键> 就是那个键登记的全部入口。
#   崩溃枚举用例（键是 crash-case:<名>）：门禁 checker-tier-crash-point-replay 逐条跑，每条用例的崩溃状态块按块键在崩溃判定库里复用、
#     键对不上的块现判（块键与库在 crates/singlefs-checker-tier/src/crash_identity.rs 文件头）；第二列写 -，入口就是第三列的 test=。
#     （原第 18–24 行照旧接在这里）
#
# 键不手列：单元闭包由代码从入口现算；收不全（动态分派、宏展开、include_str!(concat!…) 认不出）的那一条拒算键、照跑，不静默少收。
# 少收一项由各判定库的四格键自证兜：只改一个被调单元只作废依赖它的判定、改没被调的函数零作废、只改注释零作废、只改被引用的常量必须作废。
# 入口与 readwrite= 的路径只写从仓库根起的。
```

（`environment=` 留名不改义之外，「入口」一列的具体写法、`closure=` 是否保留、`@<别的键>` 是否保留，由改动批定，推。）

## 七、收尾核与这一轮的计数（原样输出）

收尾核脚本（只读，主工作区根下跑）：`bash ~/.local/share/singlefs/work/survey-pruning-r2-words/probe/closing-check.sh /home/fy5090/code/singlefs`（词表与文件清单写在脚本里；脚本随草稿目录留着，主 agent 要进仓就挪进 `scripts/`）。今天的输出数（`| wc -l` 与按文件数）：

```
86
      1 .claude/agent-common.md
      1 .claude/agents/sweep.md
     17 .claude/gate.d/stage-inputs.tsv
      2 .claude/gate.d/stage-owners.tsv
      6 .claude/hooks/heavy-test-guard.sh
     10 .claude/hooks/lib_heavy_tests.py
      5 .claude/hooks/runner-dispatch-guard.sh
      2 .claude/hooks/selftest-registry.tsv
     13 .claude/kb/checks-owed.md
      1 .claude/kb/verification-build.md
      1 .claude/main-agent.md
      5 .claude/rules/implementation-workflow.md
      6 .claude/rules/verification.md
      8 .claude/skills/crash-test/commit-stages.md
      2 .claude/skills/crash-test/SKILL.md
      3 .claude/skills/gate/staged-gate.md
      3 scripts/gate/README.md
```

改完之后该留着的只有：`.claude/agents/sweep.md:38`（误命中，N19）；`checks-owed.md` 里 C8 新句（点名已删的 `stage-must-run.sh`，D50）、C602、C793（史料，N18）、已还清表里的 C454，与挪进已还清表的 C629、C800、C836 三行。别处再命中的，照第二节判法补改。另两份附录在改代码那一批收尾时各自重扫：`probe/headers.py` 对门禁与脚本文件头（今天 513 行），命中只许是误命中。

本报告自己的计数（`probe/` 下三个脚本，原样输出）：

```
旧串核对：恰好一次 82 条，不是一次 0 条
号数 97 ；各去向 {'照改': 65, '并入': 19, '不改': 12, '空号': 1} ；缺号 []
N 号 28 {'照改': 20, '并入 D87': 1, '不改': 6, '关': 1}
```

## 八、交主 agent 定的（要设计判断，我没定；句子里已标「推」）

1. 轻型「没有剪枝」（`verification-build.md:288`，用户 2026-10-10 定案的整理）与 2026-10-15「剪枝就是单元的」、`verification.md:46`「放量那一段经长期库复用」并不并（N15）。
2. `checker-tier-crash-point-replay` 的 `--full` 今天是「跑全量档（登记的崩溃枚举用例）」、照样复用；底稿第二节第 1 条把 `--full` 列成只写不读的全量开关。新句按「`--full` = 全量档、按块复用；`--start-over` = 只写不读、写回同一份库」写（D3、N05），反过来定的话 D3、D22、N05、N23 四处跟着换。
3. commit-stages 第 1 步「工作区等于暂存区」那条核对的路径集合（D21 写成 `crates litmus Cargo.toml Cargo.lock .cargo scripts .claude/gate.d .claude/scripts`）：今天它从 `stage-inputs.tsv` 各行取路径，新表不再写路径。
4. `refs/sop/staged-green` 删不删（D32、N11、N17、`gate-staged.sh`）：今天读它的只有 `admission.py` 的 gate-reuse。
5. 第 6 步「几道绿是不是同一份源码」的指纹留不留 `git rev-parse HEAD`（D28 只删了树号与提交号那一句）。
6. 「判法版本」的取代名（D72、D73 写的是「判法单元」）与 `crates-mutation-rows.py judge-version` 子命令的新名（H03、H14）。
7. 新要求，改动批要不要做：各道 `--list-items` 报每项键对不上的判定条数（D7）；崩溃判定库每块记判定日期与跑法（D56，C635 要用）；作废单元清单进红句（D31）。
8. 广谱变异 `--import-results` 留不留（H01、H09）；它的结果并进本机那一份库，与「第二台不留长期库」同一套。
9. `stage-inputs.tsv` 第二列「入口」的写法、`closure=` 与 `@<别的键>` 留不留（D87、D97）。
10. D13 已定项 17 的依据要引盘上的数，先把 disk 路报告入库（第六节括注）。
11. 改 `.claude/main-agent.md`（D35、D36）、`.claude/agent-common.md`（N07）、`.claude/agents/experiment-designer.md`（D40）、`mutation-triage.md`（D42）走一轮三方或按内容哈希登记用户豁免。

## 九、要主 agent 跑的命令

这一轮没有要跑的重活与自证。改动批合入之后（不是现在）：

- 收尾核：工作目录主工作区根，`bash ~/.local/share/singlefs/work/survey-pruning-r2-words/probe/closing-check.sh /home/fy5090/code/singlefs | wc -l`，不用内存包装、不用线程数；看输出只剩第七节列的那几行。
- 钩子改完的自证由改它的 tooling-writer 照钩子拒绝说明跑（`heavy-test-guard.sh`、`runner-dispatch-guard.sh` 的 `--selftest`，以 `.claude/hooks/selftest-registry.tsv` 的登记为准），看 H13 的弄坏开关仍红在点名的格、H19 三处格名对得上。
- D13 写回之后：`bash .claude/gate.d/doc-decisions.sh --write`（D48）。

## 十、没做什么

- 没改主工作区任何文件；没起 cargo、门禁、测试二进制、判定库；没跑任何 `--selftest`（钩子的判法核对是在副本里直接 import `lib_heavy_tests` 调函数，不是起它的自证）。
- 门禁与脚本文件头只给了每份「复用」一段的现状句（N23、N24），没逐行给定稿：那些行描述的是要重写的代码，代码没写之前给不出逐行真话；逐行原文在 `appendix-header-lines.md`。
- 新写法里标「推」的，没对照改完的算法核过（算法还没改）；底稿第三节的问（Verus 按函数拆项、QEMU 内核摘要、LKMM 锚点到 fn、libtest 逐条认结局、`admission.py manifest` 调用方、E164 V5/V11 阳性对照样本）不归这一路，句子里用到的地方写了「做不到就怎么改」。
- 实验页只核了 E142、E164、E171 三页与第一轮点名的行，没逐页通读别的实验页；`records/` 只看了本批那一份；`decisions-history.md` 只看了 D13 一节的格式。
- 上游 `.claude/singlefs-ai-sop/` 没改，第五节那一段要主 agent 发给上游会话。
- 仓副本已删：`~/.local/share/singlefs/work/survey-pruning-r2-words/repo`（只含 `.claude` 与 `scripts`，删前 `du -sh` 输出 `64M`）。草稿目录里留着的是本报告、两份附录（`appendix-cited-lines.md`、`appendix-header-lines.md`）与 `probe/` 下的探针脚本和它们的输出（删副本之后整个目录 `du -sh` 输出 `660K`）。
