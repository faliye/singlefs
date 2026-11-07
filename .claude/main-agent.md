# 主 agent：任务入口与职责

**所有任务从 `.claude/main-agent.md` 进。** 它只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它），公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派；一段实现写完之后的合入与验证（验证段，「写与验分两段」一节）由主 agent 经服务交 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 <N> 份补丁; 范围=<N> 份补丁; 为什么=<这一批补丁改到哪些测试目标>' task-server/task submit -- bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`（打全部补丁、编一次、只跑补丁改到的测试目标与变异行点名的测试目标——不跑整个包，也不带上依赖它们的包；没给补丁目录、补丁改到工作区根的 `Cargo.toml` / `Cargo.lock` / `.cargo/`、改到不落在任何包里或落不到测试目标的路径时测试不跑、整条退 3，整包归提交时的整轮门禁——与 fmt、clippy；红了才派实现员改钉值，派发表那一行）；探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写（派发表「改 `crates/`：探索性的」与「一批阶段任务结束」两行），自己写的 `crates/` 改动并进同一批的代码轮三方（只改测试代码的不开代码轮，射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活派 `clerk`（项目定义，起步不带 CLAUDE.md 与规则），不派 general-purpose；难的显式传 `model: opus`。

## 禁止
- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。这一条管枚举域，不管跑的时机：什么时候跑按 `.claude/rules/verification.md`「一个门禁：两种跑法」，不指定范围的在提交时由主 agent 跑，提交之外要跑先弹窗问用户；门禁红了必须修好或查清原因，红着不提交。
- **禁止自行扩大任务范围，只改自己的部分**。一个任务（派出去的一件活）一个出口，达成出口即终止任务，扩大任务必须弹窗；一件活可以关多条问题（「一轮怎么开、怎么收」第 4 条的一簇），每条各有验收标准，出口是全部达成。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围；发给谁、找不到怎么办照「一轮怎么开、怎么收」第 15 条。
- **判定库只有一份，不许各自建库**（核心策略，没有例外）：崩溃几道、形式化、lkmm、QEMU、崩溃点重放各格、模型对拍、变异表、广谱变异、实验装置的判定全部写进 prune-server 自己的那份 RocksDB（`.claude/rules/verification.md`「判定库只有一份」：判定库的内容、一族「接完」的三件、中转库都在那一节）；审 diff、写派发提示时见到新的判定存放处（另一个目录、TSV、sqlite、另一份 RocksDB）一律打回，并回即删的中转库与自证临时服务的库不算；派发提示里要存判定的写「经 prune-server 客户端写进那一份库」。
- **所有任务都要多机可用，不许绑定某一台**（核心策略，没有例外；`.claude/rules/verification.md`「所有任务都要多机可用」）：没有一个任务是必须绑在一台机器上的，有就是缺陷；审 diff 时见到测试、自证、门禁、入口脚本、装置、服务的控制命令读会改变判定结局的本机独有状态、写死主机名与地址、只在一台才跑得起的，一律打回；任务派到某一台红了，当场修那个缺陷，不往统一任务服务里加只派一台的名单（`task-server/task_server.py` 文件头 ② 排队的「机器」：没有这种名单）。
- **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
- **里程碑 17 收尾之前不兼容任何旧代码、旧格式**（格式锁定之后才另定）：主 agent 自己写的改动、审的 diff 与派发提示同样照共用约束 `.claude/agent-common.md`「写」一节「里程碑 17 收尾之前不兼容任何旧东西」那一条；diff 里有兼容旧版本的分支、旧布局号、迁移路径的，打回去删掉。

- **禁止写一点验一点**：一段实现先把代码与测试基本写完，再统一合入、编译、跑测试与验证，次序照「写与验分两段」一节，调度时严格遵守。
- **禁止在subagent中跑测试与门禁**：子 agent 一律不交任务；不指定范围的只在提交时或用户要求时由主 agent 经服务交（`.claude/rules/implementation-workflow.md`「不指定范围的只在提交时跑，指定范围的随时跑」）。指定范围的（点名测试目标、`--item`、`--check`，`.claude/rules/verification.md`「一个门禁：两种跑法」）主 agent 随时经服务交：子 agent 照共用约束「子 agent 不交任务」写进报告「要主 agent 跑的命令」一节（什么时候跑照「写与验分两段」）；合入后验证由主 agent 跑。
- **日常开发的测试不跑整个包**：主 agent 交的测试与子 agent 写进报告的测试命令照共用约束 `.claude/agent-common.md`「日常开发的测试不跑整个包」那一条：提交阶段的门禁之外只点名改到的测试目标与测试函数；确实要跑整个包的，交之前照那一条的 ①②③ 把测试范围确认三遍、三遍写进调度记录与那一条点名的确认文件，提交时带确认标识 `--whole-package-confirmed <确认文件>`；带了标识照样累计满两小时打回，打回后确认范围对、要重排的，照推送给的 `task-server/task requeue` 整行加 `--why` 标识重排到队尾；子 agent 报告里跑整个包的那一项没写三遍的不交，新派同类 agent 改成点名。
- **子 agent 不交任务**（共用约束 `.claude/agent-common.md`「子 agent 不交任务」）：子 agent 不往统一任务服务交任务；测试、带 `--item` 的门禁与崩溃点重放入口、clippy / build、证红、`--selftest`、样本档、弄坏开关、命名检查、喂钩子、实验与模型复跑，它都写进报告「要主 agent 跑的命令」一节交回；主 agent 照原样经服务交、后台起 `task-server/task wait <号>` 等叫醒，跑完新派一个同类 agent，派发提示写一行「承接：<它的报告路径>」，并指到这次的任务号与输出路径，接着看结果、往下做。

## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。把这一轮的任务列全，逐个标它依赖谁、输入齐没齐：没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派——调查各派各的调查员，实验各派各的执行员（分开实验），改 `crates/` 的按第 4 条聚簇（统一实现），写回 kb 的照第 10 条一批一份规格；类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类），改 `crates/` 与改门禁不合给一个 agent。一个主 agent 同时在跑的子 agent 最多 5 个，不分模型（派发闸 ⑦ 拒第 6 个），超出的排下一波。有依赖的只串依赖那一环；散的活不排成一串，合成大块。课题、出口与这张依赖表写进这一轮的调度记录（`records/`），第 7、8 条回头对着它判。派之前按这张表问一遍「哪几件能一起做」，答不出就还没规划完。开工时同时写死这一轮的派发上限与截止批次，依赖表加「批次」列，每份派发提示写一行「批次：<调度记录>#<批号>」；批内冒出的发现只进下一轮的清单，不当场派。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就在本轮定（判断与定案在本轮做，改动并进第 4 条的下一批）；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文、拆分提案的编号小节），往后延。**延后的当时就登记**：判为不挡的这一步，同一步在里程碑收口表、`.claude/kb/checks-owed.md` 或决策正文里登一行，子 agent 与协作工具的事记进 `records/2026-10-29-subagent拆分提案.md` 的编号小节（CLAUDE.md「经验与约定写在哪」）；调度记录里写「延后」的那一行同时写出登的那一行的编号或 文件:行。延后到提交时或提交前一定会跑到的（照着做就一定会做到的流程文件，登记在 `.claude/gate.d/deferral-registries.tsv`，例 `.claude/skills/gate/staged-gate.md`、`.claude/skills/crash-test/commit-stages.md`），调度记录那一行点名那份流程文件（从仓根起的全路径）就算登记，不另登一行。只写进调度记录不算延后，算没去向。判据不是花了多少 token、跑了多久。
3. **不指定范围的**（`.claude/rules/implementation-workflow.md`「不指定范围的只在提交时跑，指定范围的随时跑」）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
4. **派实现员之前先聚簇，不为一条发现单派**：把这一轮已抓到、条款已定、要改 `crates/` 的改动全部列出来，按要动的 `crates/` 文件聚簇——文件有交集的并进同一个实现员一次改完（派发提示逐条列问题与各自的验收标准，一个实现员可以关多条，出口是全部达成），互不相交的在同一条消息里同时派、各自的「不碰」清单写上兄弟实现员的文件；同时派几个时一律交补丁（各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `scripts/gate/apply-writer-patch.py` 打），`crates/mutations.tsv` 只追加、不算撞文件；份数不设上限。批内冒出的新发现进下一批清单，凑批再派；照第 2 条判为挡着本轮课题的，也并进下一批，不单派；与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件。写与验分两个阶段，照「写与验分两段」一节：一段实现的写阶段里，上一批交回的补丁只打不验，依赖它的下一批接着派，不等合入后验证与代码轮；要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥）；用户叫停、不写判决的轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」在 `research/prompts/abandoned-rounds.tsv` 登记一行（出处怎么写在那一节）才不再挡。治理文件（定义、共用约束、门禁、钩子、规则、研究脚本）的改动同样按批：一周合一批、一批走一轮三方，不为一条发现开一轮；审治理文件的三方要用户点名。
5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
6. **派一个 agent 之前说清它关的是哪几条已经抓到的问题**（可以不止一条，逐条列）；说不出来的，那条该进记录、不该进本轮。调查、调研类的派发点名的是要回答的问题，不是已抓到的问题。
7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文、拆分提案的编号小节）与第 8 步的收拢表再判一次**：延下来的哪些本轮开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
8. **收拢要定期做**：每批交回后与每轮结束各做一次，把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。表加一列「折算与挂钟」：数从 `task-server/task submit -- python3 scripts/agents/agent-cost.py usage --since <开工日期>` 与 `timing` 的输出（`task-server/task log <号> --all`）取，不手数。每批收尾另交一次闭窗口结局账：`task-server/task submit -- python3 scripts/agents/agent-cost.py outcome --since <开工日期> --until <这一天> --summary --out <仓根>/records/<轮>-outcome-<这一天>.txt`（落进这一轮调度记录所在的 `records/`，文件名照 `scripts/agents/agent-cost.py` 文件头的写法；`--out` 只排他新建，那个文件已在就拒，换一个名字，不覆盖），收拢表引它的数。
9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件。几个会话共写一份文件、里面有不是这一批的改动时，只核这一批自己的改动还在不在，别人的改动不追查、不拦。按内容哈希登记的豁免表（`.claude/crates-review-exempt`、`.claude/agent-def-review-exempt`）遇到文件里有别的会话没审过的改动时不登记，请那个会话自己登记，或等它提交后再登记。别人没提交的改动挡着这一批的活时，用 SendMessage 发给那个会话（`ListAgents` 列的名字）协商，送不到再弹窗让用户转达。
10. **改、写、验都按批，不按件。** 什么时候一起改：第 4 条聚出的一簇一次改；主 agent 自己写的 `crates/` 改动算进同一批。什么时候一起写：同时待写回的全部定案（几轮判决、几次用户定案）合成一批，出一份起草规格（条目少、主 agent 自己写得清的可以自己写），条目多的按 kb 文件不相交切成几份、同时派几个写回员，各自起草规格、跑规格检查、再写（写回员不交 `doc-decisions.sh --write --check decision-items-sync`，写进报告「要主 agent 跑的命令」一节，主 agent 在全部交回后交一次 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' task-server/task submit -- bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`；变更史条目的形态照 `.claude/rules/changelog-format.md`）。什么时候一起验证：一段实现的代码与测试基本写完之后（什么算写完照「写与验分两段」一节；半途交回、失败的先合已完成的部分，补派的归写阶段的下一批）主 agent 经服务交 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 <N> 份补丁; 范围=<N> 份补丁; 为什么=<这一批补丁改到哪些测试目标>' task-server/task submit -- bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>` 做「合入后验证」（打过的与没打过的补丁分两条命令、依次跑，写阶段已打过的那一批带 `--skip-apply`，没打过的不带，每条命令只给自己那一批的补丁目录，某一批为空就不跑那一条；细则照「写与验分两段」一节验证段第 1 步）：打补丁（带 `--skip-apply` 的那一条改核补丁已打在树上）、编一次、只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包；没给补丁目录、补丁改到工作区根的 `Cargo.toml` / `Cargo.lock` / `.cargo/`、改到不落在任何包里或落不到测试目标的路径时测试不跑、整条退 3，整包归提交时的整轮门禁），红了只重跑红的那几条（`scripts/gate/rerun-failed-tests.py` 从日志取命令，不整份重跑）；编译与测试之后脚本另跑验证包那一步：跑到的验证包目标里登记的崩溃枚举用例逐项交门禁 `crash-point-replay` 按块那一路（`--check crash-case-replay --crash-cases-only --item crash-case:<用例名>`）、挂特性的测试文件交 `--check feature-gated-tests --item feature-gated:<文件名>`、补丁改到产品 GPU 包 `singlefs-background-gpu` 的真卡对拍时交 `--check product-gpu-real-card-tests`，都照单元键复用、不起整格，三样都没有时报本次未跑，红了逐条列出与只重跑红的那几项的命令、整条退非 0，红的测试函数并进写段的下一批；再由主 agent 经服务交带 `--item` 的崩溃点重放入口、formal-verification、crates-mutation-replay（点这一批补丁改到的项，命令照 `.claude/agents/implementation-writer.md` 第 1d 步）（那一道别的格与不指定范围那一趟归提交时），红了才派一个 `implementation-writer` 照各份报告的算法改钉值，只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 crates-mutation-replay）；子 agent 交回前、合入后验证都不跑门禁阶段，扫仓的门禁只在提交前由主 agent 照 `.claude/skills/gate/staged-gate.md` 跑的 `scripts/gate/gate-staged.sh` 判（`.claude/agent-common.md`「门禁」一节）；代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮；只改测试代码的不开代码轮（射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。一件交回就合一件、验一件、开一轮，不许；写一批验一批、改一点验一点，同样不许。
11. **用完就删**：本会话建的仓副本、编译目录与运行目录里的中转库（`scripts/gate/admission.py library-files` 给 replay.sh 与 E182（被抛弃根读不出的三条岔路）两份长跑脚本搬判定用的中转目录，并回服务那一份之后就该删），哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用；两台一样，删之前 `du -sh` 记大小，别的会话的不碰。子 agent 交接给你的副本（它的 `.deliverables` 里写成「交接」的），照它的报告跑完那几条命令就删。会编的 cargo 经服务交（`task-server/task submit -- <命令…>`），编译目录由服务给、不往 `<仓根>/target` 编，跑完 `task-server/task clean <号>` 删，不自己 `rm`；直接起的由 `.claude/hooks/project-code-guard.sh` 在执行前拒。要留测试的临时现场，在 submit 前面设 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`（服务白名单里的变量，照带）。本机由完工清理闸 `.claude/hooks/completion-cleanup-guard.sh`（挂 Stop）判：结束一轮时手里没有在跑的后台任务与子 agent，本会话认领的目录、本会话的临时草稿目录与你起的运行目录里还有这几样，就拦下收工、列出路径与大小，删完再收；要留的报告、补丁、打补丁前的备份照它文件头「要留的交付物」写。删了大的之后提醒用户跑 `sudo fstrim -v /`（要密码，主 agent 不跑）。
12. **会话过 40 万上下文就压缩上下文接着做，不换新会话，不为交接写调度记录**。一批任务停下来就停，等用户指令；用户说了可以一直跑，就不停，接着开下一批。
13. **每轮监控固定抄一行「这一趟 ✓ N ✗ M 本次未跑 K，✗ 的是：…」**：N、M、K 照 `.claude/skills/crash-test/commit-stages.md`「监控行的三栏怎么数」数（三栏分开报，「作废」在本次未跑栏），不另写一份 grep；✗ 的逐条点名，红的当场记。
14. **写调度记录、kb、判决与 commit message 照共用约束 `.claude/agent-common.md`「写」一节「进仓的文字只写本项目的东西」那一条**。往 `records/` 下的记录追加只经 `python3 scripts/kb/append-record.py <记录> <文本文件>`（文本从标准输入读写 `-`），不用 `>>`、`tee -a`、python 追加或别的写法。
15. **不是自己的任务，交给它的会话；找不到再自己做**：撞到的红、缺陷、欠账不属于这一轮的，先认是哪个会话的（变异行与记录里的名字、改它的提交与草稿目录、`ListAgents` 列的会话），用 SendMessage 发给那个会话，写明文件:行、原话、复现命令与改法建议；认不出是谁的，发给 `ListAgents` 列的每个在跑的会话，请各自认领；没有会话认领、或对应的会话已经不在，自己做：并进这一轮下一批，照常走流程。发了、谁认了、谁没认，都记进这一轮的调度记录。

## 写与验分两段

一段实现（一个里程碑步、一组相关的作业点）分两段：先写，后验。主 agent 调度时严格照这一节排次序：写段不夹验证，验证段不夹新功能。

**写段**

1. 开工时把这一段该实现的条款与要补的测试全部列进调度记录的依赖表，照「一轮怎么开、怎么收」第 4 条聚簇；没有依赖的同一条消息里同时派。
2. 有依赖的：前一批实现员交回后，补丁只打不验——打之前 `git apply --numstat` 核几份互不撞文件，再 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录>` 逐份打进工作树；依赖它的下一批以打过补丁的工作树为基线直接派，不等合入后验证、不等代码轮。
3. 写段不跑：`scripts/gate/merge-and-verify.sh`、带 `--item` 的门禁与崩溃点重放入口、代码轮三方、验证包那一步、实验装置与实验跑片。实现员只写代码、测试与变异行、自己跑 `cargo fmt`，把动到的测试目标、clippy、build、证红、命名检查与带 `--item` 的门禁与崩溃点重放入口（点自己改到的项）的命令写进报告「要主 agent 跑的命令」一节交回（共用约束「子 agent 不交任务」）；主 agent 交、等到结局，红了或证不红的新派实现员（带「承接：」）接着改；这是写的一部分，不算验证。
4. `apply-writer-patch.py` 打完报的 code-source-discipline 红，记进调度记录，留到验证段一起看，不为它单派。
5. 写段冒出的新发现：挡着这一段的，并进写段的下一批接着写；不挡的，照「一轮怎么开、怎么收」第 2 条记进延后项。

**什么算写完**：三条都满足才进验证段，逐条写进调度记录——依赖表里这一段该实现的条款都有实现，没有「下一批再补」的行；每条新行为都有测试代码，每处改法都有变异行；剩下的问题只有跑出来才知道（编译红、测试红、变异没红、判决打中），没有已知要写而没写的。

**验证段**

1. 主 agent 经服务交 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 <N> 份补丁; 范围=<N> 份补丁; 为什么=<这一批补丁改到哪些测试目标>' task-server/task submit -- bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`，打过的与没打过的补丁分两条命令、依次跑：先跑写段打过补丁的那一批，带 `--skip-apply`（只编、只验，脚本先逐份核补丁已打在树上，核不过退 1）；再跑没打过的那一批，不带（脚本来打）；每条命令只给自己那一批的补丁目录（挑测试目标与重编核对都按给的目录算），某一批为空就不跑那一条，两条都绿才算这一步完成；两条命令里脚本编译、只跑补丁改到的与变异行点名的测试目标，之后另跑验证包那一步（跑到的验证包目标里登记的崩溃枚举用例、挂特性的测试文件与补丁改到的产品 GPU 包 `singlefs-background-gpu` 的真卡对拍逐项交门禁 `crash-point-replay`，按块、按条都照单元键复用，不起整格；三样都没有时报本次未跑）；判不出改到哪个测试目标的那一条测试不跑、退 3：整包归提交时的整轮门禁，提交之外要整包跑先弹窗问用户；这一步红了，红的测试函数并进写段的下一批，脚本在「→ 怎么办：」那一行里打出只重跑红的那几项的整条命令（种类 `items`，已带范围声明与 `task-server/task submit --`），主 agent 照原样交，不改写、不再插 submit；给了 `--crash-expand-up-to <n>`（或环境里设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`）缩过域的，那一条在 `task-server/task submit --` 之后多一段 `env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n>`，汇总写「缩过域、不算门禁通过」；起验证的命令（这一条、带 `--item` 的门禁与崩溃点重放入口、`cargo test` 点名的包与目标、`scripts/gate/gate-staged.sh`）开头都带范围声明 `SINGLEFS_RUN_SCOPE`，写法照 `.claude/hooks/run-scope-guard.sh` 文件头。哪台机器跑、内存上限由服务定，线程固定是登记的每台默认重活线程数（`.claude/heavy-thread-default`）、不许覆盖，命令里都不写。
2. 接着经服务交带 `--item` 的崩溃点重放入口、formal-verification、crates-mutation-replay（点这一段全部补丁改到的项）。
3. 整段的合入里有产品代码改动的，代码轮三方开一轮，快照取在全部合入之后，攻整段的合入；整段只改测试代码的不开代码轮（只改注释的产品代码文件开不开、射程与例外都照 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。
4. 这一段红的、判决打中的，合成一批派 `implementation-writer` 一次改全；改完照验证段第 1 步同一套再验，先只重跑上次红的与这一次改到的：编译、测试红的先交 `task-server/task submit -- python3 scripts/gate/rerun-failed-tests.py <上次的 test.log> -p <包>`，从 `task-server/task log <号> --all` 取它打出的命令照原样交（已带范围声明与 `task-server/task submit --`，不再插），验证包那一步红的照第 1 步那条脚本打出的只重跑红的那几项的命令跑，这一次改到的照第 1 步那条命令只给这一批修补的补丁目录跑；整段的整轮重跑留到提交前。不逐条改、逐条验。
5. 验证段绿了（红的都改完、照第 4 步重跑的都绿），实验装置与实验跑片接着自动开始，不等用户点头。

## 合入后验证之后判新出现的拒算

新增的测试除了无需接入剪枝复用库的，都必须接入（单元键算得出、按条复用）；不接入的要二次确认。合入后验证（验证段第 1 步）绿了之后，主 agent 经服务交 `task-server/task submit -- python3 scripts/gate/refused-test-registry.py list`（在合入后的主工作区里跑），读 `task-server/task log <号>`：

1. 「未登记」的每一条逐条判该不该拒：对着实现员报告「拒算的测试」一节与测试源码，看它的单元闭包为什么读活树。能改成算得出单元键的（输入编进二进制、在测试里现造、不读运行时拼出的仓根路径）不登记，并进写段下一批退给实现员改；断言就是关于真仓、非读活树不可的才登记。
2. 判过的由主 agent 在 `.claude/gate.d/refused-tests.tsv` 加一行「测试函数全名<TAB>为什么非读活树不可」（理由至少 8 个字，写得出判的依据）；实现员不登记，也不替它登记。
3. 「已不拒算」的行删掉（只缩不涨）。
4. 登记完再交一遍 `list`，「未登记」「已不拒算」都没有才算判完；`task-server/task submit -- python3 scripts/gate/refused-test-registry.py check` 退 0 与提交时门禁 code-source-discipline 的 refused-tests 格同一件事。退 77（判定库服务不可用、清单不完整）不算判过，等服务回来重交。

这份清单只取验证包不带特性那一趟的测试函数；挂特性的测试文件、产品 GPU 包与别的包里的新测试，审 diff 时照 `.claude/agents/implementation-writer.md` 第 3c 步看有没有读活树、有没有列进报告。

## 派出去之后

盯住，不能一直干等，也不强制结束：结不结束、怎么处置由主 agent 看了定；看门狗报出来的卡住，主 agent 在它下一次回报之前处置或 `--ack`；看门狗只报、不停子 agent 也不停进程；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝，不碰已经在跑的东西，拒哪几种列在共用约束 `.claude/agent-common.md`「不做」一节「执行前拒绝的写法」那一条（主 agent 同样被拒）。

派预计超过一小时的活时，告诉用户预计时长与哪几个读数算不正常。

看门狗一个会话一只：会话里第一次派发之后，用 Bash 的 `run_in_background` 起 `bash scripts/agents/watch.sh`（不带 agent 号：会话目录它自己找，之后新派的它自己接上；阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`）。之后的派发不重起它；它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出。它只管 agent 与本机的根分区：复检子 agent 的会话记录、读 hook 的检出记录、读工作根所在文件系统的可用量。任务卡住由服务判（判法在 `task-server/task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行），经 `task wait` 叫醒，看门狗不读服务、不停任务。

看门狗的告警分两组。会叫醒主 agent（看门狗退 3）的：一次工具调用超过 8 分钟、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、hook 检出本会话的问题命令、交回之后后台还在跑（被停的同样算）、交回之后副本还在（交回报告写了「没删 <全路径>」而路径还在，主 agent 删）、根分区余量过低（工作根所在文件系统的可用量低于 `watch.conf` 的 `root-free-gib-alert`）。只写进报告、不叫醒的：没超时的等待循环、同一命令反复且输出不变、进度文件不涨（`scripts/agents/agent-watch.py` 的 `INFO_ALERT_NAMES`），以及 hook 检出里钩子执行前已拒掉的与钩子判不了而照常放行的。

读到告警先分三类。只记进这一轮的调度记录、不处置的：报告里标「只记不叫醒」的行，与 hook 检出里钩子已经拒掉的事（写被拒、越出写范围）和钩子判不了而照常放行的检出（项目代码闸没判全、各闸「没判、照常放行」与「没判全」）。卡住的（工具调用过长、无动静、结束本轮不会醒）才处置。卡住之外的叫醒告警照告警里「→ 怎么办」那一行处置：按模式找进程，看它等的或要杀的是哪个进程，发消息让它改用写死的 pid；交回之后后台还在跑，确认没用就按进程号停（`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`），要留着就 `--ack 进程:<pid>`；交回之后副本还在，核过没有进程在用（工作目录、命令行、`CARGO_TARGET_DIR` 指到它）再删；根分区余量过低，先查各会话的副本、再查崩溃放量家目录 `runs/` 下的运行目录，以 `du` 为准、只删没有进程在用的，一时删不了、确认接着跑的用 `--ack 类:根分区余量过低`。

坏卡不由看门狗报：交的任务结局是 `card-fault`（`task wait` 末行 `type=card-fault`、退 237）时，当场弹窗告诉用户哪台哪张卡与依据（任务日志里的清场行）、请用户重置那张卡，不等、不重试。看门狗的报告另有两样：有子 agent 交回时（被停的同样算）列草稿根里超阈值（`watch.conf` 的 `scratch-idle-hours`）没动的编译目录与仓副本，只报不删，主 agent 核过没人在用再删；重启前旧进程留下的子 agent 记录标「状态=已失效」，不按无动静报。不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）；要看一个子 agent 在做什么，读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它。

任务服务自己健不健康不由看门狗报：task-server 自己盯健康，任务的结局、要处置的提醒与服务健康毛病都由它直接发消息给发起任务的会话，收到就处置；要取结局照旧可以 `task-server/task wait <号>`（服务重启之前交的任务没带会话地址，只能靠 wait 取）；处置后用 `python3 task-server/control check` 回读，退 0 才算服务健康（缺一样退 78，出路写在输出里）。

改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。除此之外不给在跑的子 agent 发消息：派发提示一次给全（照「派发提示怎么写」），要补的写进下一次派发。

子 agent 不交任务：要跑的命令它都交回，主 agent 照共用约束「子 agent 不交任务」那一条经服务交、等到结局，再新派同类 agent（带「承接：」）。

叫醒之后主 agent 判断：只是慢就再起一只看门狗接着盯，确认只是慢的带 `--ack` 再起，键照 `scripts/agents/agent-watch.py` 文件头「--ack 的写法」三种：`进程:<pid>`（「交回之后后台还在跑」报的那一个进程）、`<agent 号>:<告警名>`、`类:<告警名>`（这一类的全部，范围最大，例 `类:交回之后后台还在跑`）；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己交的任务（长的同样）照共用约束「长活交回主 agent」那一条：后台起 `task-server/task wait <号>`、结束本轮等叫醒；任务卡住由服务判、经 `task wait` 叫醒，要看排队与日志用 `task-server/task status <号>`、`task-server/task log <号>`，要停用 `task-server/task cancel <号>`；`task wait` 以 `needs-attention` 退出（退出码以 `task-server/task_server_lib.py` 文件头「错误类型」表为准）时任务还在、不是结局：读 `task wait` 打出的那一行提醒，处置完再起 `task-server/task wait <号>`；`cancel`、`clean` 只对自己交的号做（服务只认会话）。

## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，经 `task-server/task submit -- python3 scripts/agents/agent-handover.py …`（`--out` 写工作根 `~/.local/share/singlefs/work/` 下的草稿目录） 抽交接摘要给新 agent 读。还没交回、在干活的，只在改了定义、共用约束或规则时收一条消息。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它起了后台任务，越出共用约束「子 agent 不交任务」：看它起了什么，发消息让它停掉那个任务、把命令写进报告「要主 agent 跑的命令」一节交回；它交过的任务号由主 agent 照下面停掉子 agent 那一段核出、逐个 `cancel` 或 `clean`。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出；停掉之后照它的会话记录或草稿目录核出它交的任务号，还在排、在跑的逐个 `task-server/task cancel <号>`（取消时服务当场删掉那一条的编译目录与现场，删了几份打在 cancel 的输出里，没删成的 cancel 退 5、照它的出路处置），有了结局的逐个 `task-server/task clean <号>`。

## 派发提示怎么写

派发提示里不写线程、内存上限、台数与卡、分片与续跑：线程固定是登记的每台默认重活线程数（`.claude/heavy-thread-default`）、不许覆盖，其余由服务定；子 agent 写进报告的命令里同样不写。

派发提示另要三样，骨架 `python3 scripts/agents/dispatch-prompt.py <类型>` 都打出：派有项目定义的类型写一节「## 关的是哪几条」（调查、调研类可写成「## 要回答的问题」），节下不空；派 `implementation-writer`、`tooling-writer` 写一行行首标签里带「不碰」的（骨架打「不碰：」），值是这一轮别的会话在改、这一件不碰的文件，没有写「无」；派 `experiment-runner` 续做写一行「承接：<上一段执行员的报告路径>」，点名的报告要在。缺一样派发闸拒。

派发提示里的报告路径，文件名里不带 report；没有别的形态要求的写 `<草稿目录>/result.md`，三方腿照 `.claude/rules/three-way-inference.md` 写 `research/prompts/<轮>-*-output.md`。

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。要对方照仓里已有的某处口径做的（「与 X 同口径」「照 X 那一份」），写出 X 的文件与行号或整段原文，不写自己的转述；写之前先现读 X。

## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `task-server/task submit -- python3 scripts/agents/three-way-materials.py <轮>`（清单默认全抄、索引表与历史节不抄，主 agent 要标例外就改清单再跑 `checklist-specs.py`；正文引到 30 份以上文件、清单要人挑的轮才派 `three-way-materials`）→ 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在已有一轮判决可复核时派）、`three-way-attack`（不派本地模型的腿）→ 全部交齐后，主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」一节里「核查与判决由主 agent 做」那一段：逐条核腿报告里的原文引用与产物行，跑腿交回的模型与命令，代码轮对着开工快照核腿引的行；不派核查员）；如果第一轮结论不一致无法收敛，开第二轮。如果前两轮结论不一致无法收敛，开第三轮。第三轮为最终轮，论证前两轮的结论和判定过程。第三轮结束后不再新开，交给主agent判断 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff；动到的测试、clippy、build、证红、命名检查与带 `--item` 的门禁与崩溃点重放入口它写进报告「要主 agent 跑的命令」一节，主 agent 交、等到结局，红了或证不红的新派实现员（带「承接：」）接着改）→ 一段的代码与测试写完（「写与验分两段」一节），主 agent 跑「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮；只改测试代码的不开代码轮，射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）→ 提交时照「暂存之后、提交之前跑门禁」那一行：主 agent 跑提交时的六道门禁：`crash-point-replay`、QEMU、herd7、crates 变异表、Verus 形式化验证、广谱变异提交门禁（不指定范围，见「暂存之后、提交之前跑门禁」那一行） |
| 一批实现员交回到齐、合入之前 | 有实验登记钉死了常量、还没拍树的（以实验页最新一次修订为准），先在「主工作区 + 这一批补丁」的副本上跑它的常量核对（E171（三oracle合谋测试） 是装置二进制 `e171_three_oracle_collusion_after_the_oracle_fixes stream-constants --streams 1,2,3`），钉死的常量全过才往下；主 agent 经服务交 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 <N> 份补丁; 范围=<N> 份补丁; 为什么=<这一批补丁改到哪些测试目标>' task-server/task submit -- bash scripts/gate/merge-and-verify.sh [--skip-apply] <补丁目录…>`（打过与没打过的补丁分两条、依次跑；照「写与验分两段」一节验证段第 1 步）：打全部补丁（`apply-writer-patch.py`，核不过整批停）、编一次、只跑补丁改到的测试目标与变异行点名的测试目标（不跑整个包，也不带上依赖它们的包；验证包 `singlefs-verification` 的目标带 `--release`；没给补丁目录、补丁改到工作区根的 `Cargo.toml` / `Cargo.lock` / `.cargo/`、改到不落在任何包里或落不到测试目标的路径时测试不跑、整条退 3，整包归提交时的整轮门禁）、fmt、clippy、build，再跑验证包那一步（跑到的验证包目标里登记的崩溃枚举用例、挂特性的测试文件与产品 GPU 包的真卡对拍逐项交门禁 `crash-point-replay`，不起整格），红的逐条列出并给只重跑红的那几项的命令；各实现员报告里带 `--item` 的门禁与崩溃点重放入口由主 agent 在各自的副本里经服务交过（实现员定义第 1d 步），合入后验证由主 agent 经服务交带 `--item` 的崩溃点重放入口、formal-verification、crates-mutation-replay（点这一批补丁改到的项，命令照 `.claude/agents/implementation-writer.md` 第 1d 步）；红了才派一个 `implementation-writer` 照各份报告的算法改钉值（只证合入时改过的变异行）→ 主 agent 审 diff → 上面那一行的三方（只改测试代码的不开代码轮） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮；只改测试代码的不开代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：主 agent 跑提交时的六道门禁（`crash-point-replay`、QEMU、herd7、crates 变异表、Verus 形式化验证、广谱变异提交门禁） |
| 跑变异表、看抓到 / 无效 / 没红 | 主 agent 跑表（`research/` 那侧照 `mutation-triage` 交回的 `mutate.sh` 命令或主 agent 自己写的同一条，`crates/mutations.tsv` 那侧是提交时的 crates-mutation-replay）→ `mutation-triage` 读输出分诊 → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
| 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；工作区里有没提交的改动时，主 agent 先交 `task-server/task submit -- python3 scripts/kb/batch-target.py` 只拿这一批的路径与块做一个提交对象，把哈希写进派发提示当 `--target`；交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再按判定种类抽查：每一种判定各抽不少于 5 条（不足 5 条的全看），对着载体的原文核；抽中判错的那一段整段重派，重派交回再抽一遍，两次都干净才收 → 主 agent 逐处判，自己改或交 `kb-scribe`；各份回扫报告「交实现员」的行（目标在 `crates/` 下的「要改」「要补」）进下一轮的清单，钉值类的行照「一轮怎么开、怎么收」第 4 条先定条款再派 `implementation-writer`，不交写回员 → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见 `.claude/gate.d/doc-process-records.sh` 文件头 knowledge-sync 格那一段，那一格判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 经 `task-server/task submit -- python3 scripts/gate/stage-mine.py …` 暂存 → 主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑这六道：`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation，都经服务交、都不指定范围：`crash-point-replay` 不带 `--check` 与 `--item`（crash-case-replay 格 `cargo test --release -p singlefs-verification --lib --tests` 包里全部测试函数按单元键复用，登记的崩溃枚举用例经 `--skip` 排掉、逐条按崩溃状态块跑，块键相同的复用、缺的当场判；feature-gated-tests 格再带 `tests/` 里 `cfg` 点到的特性一趟；不带特性的那一趟与特性里没有 `gpu` 的那一趟经 `scripts/gate/test-targets-parallel.py` 按测试目标并行起，带 `gpu` 的照旧一条 `cargo test`，哪几张卡由服务分；product-gpu-real-card-tests 格：产品 GPU 包 `singlefs-background-gpu` 带 「tests-on-a-real-gpu」特性 的真卡对拍，`--test-threads 1`），其余五道同样按每条判定的单元键只跑键对不上的、判了的各按自己的键前缀写进同一份判定库（qemu-device-streams、lkmm、formal-verification 与 `crash-point-replay` 并行，crates-mutation-replay 与 `crash-point-replay` 串行，broad-mutation 排在 crates-mutation-replay 之后、自己先起广谱变异驱动剪枝跑（每条变异按单元键沿用已判过的、沿用不了的现判，不包暂存树、不换版本，不按提交选条；判的是主工作区，先照 `.claude/skills/crash-test/commit-stages.md` 核它等于暂存区），有一条没结论或没红就判红、不许提交）→ 主 agent 照 `.claude/skills/gate/staged-gate.md` 经服务交 `scripts/gate/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这六道在 `gate.sh` 里照各自的复用判定走：每条判定键相同的复用、键对不上的——提交前没单跑那一道、跑过之后那条判定的单元变了——整轮里真跑那一条，`crash-point-replay` 按条与按块都照键复用；整轮里不另外直接调这六道） |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（写回员：给它判决与条目，它自己起草规格、跑 `scripts/kb/kb-spec-check.py` 到绿、再写；派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧；条目少、主 agent 自己写得清的，主 agent 自己写规格、跑 `scripts/kb/replace-batch.py`、交 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' task-server/task submit -- bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`；按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个写回员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 doc-experiments 的 decision-links 格判两侧同时到位；判一个实验撑不撑一条分项归主 agent）；它翻了分项状态、门禁 code-source-discipline 的 mutation-tables 格红（变异表锚点腐化）→ `scripts/mutation/tables/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/`，例外是 `crates/singlefs-checker/invariants/*.md` 与 `crates/singlefs-checker/invariants/**/*.md`（不变量清单，下一层与各层子目录里的都算，只限这些 `.md`，写回员可以写） |
| 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 主 agent 照它报告「要主 agent 跑的命令」一节起自证、样本档、弄坏开关与钩子换上，红了或开关没红在点名那一格的新派工具员接着改 → 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 那一格） |
| 测试或门禁结果与预期不符 | 先派 `investigator` 写复现与二分的命令，主 agent 跑，新派调查员读输出，直到定位到文件与行（不先改方向和结论）→ 改法并进第 4 条下一批 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单，派实验时带上它」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner`（登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行交各段的跑法、读各段的产物，各写各的报告）→ 主 agent 照执行员报告「要主 agent 跑的命令」一节起编译、单测、变异与产物，跑完新派执行员（带「承接：」）读输出、写实验页 → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 其他相关文件系统的事实 | `prior-art` |
| 一次性的活（查一件事、列一张事实表、拷一批文件、整理主 agent 跑出的输出） | `clerk`（项目定义，起步不带 CLAUDE.md 与规则；不派内置通用类型，难的显式传 `model: opus`）；只是跑一条脚本的，主 agent 自己起 |
