# 主 agent：任务入口与职责

**所有任务从 `.claude/main-agent.md` 进。** 它只写怎么做；为什么这么定、实测的数在 `records/2026-09-16-subagent拆分提案.md`（除非要查来历，别读它），公共上下文（项目是什么、当前里程碑、规则、项目本地事实）在 `CLAUDE.md`。

## 职责

主 agent 只调度和判断：和用户说话、写三方正文与判决、定案、git 暂存与提交、收尾报进度留在主 agent，其余按下面那张表派；一批实现员交回后的合入与验证由主 agent 跑 `bash scripts/gate/merge-and-verify.sh <补丁目录…>`（打全部补丁、编一次、跑工作区除 checker 档包 `singlefs-checker-tier` 之外各包的全部测试、fmt 与 clippy；红了才派实现员改钉值，派发表那一行）；探索性的 `crates/` 改动、定案后主 agent 自己写得清的小处 kb 改动可以自己写（派发表「改 `crates/`：探索性的」与「一批阶段任务结束」两行），自己写的 `crates/` 改动并进同一批的代码轮三方（只改测试代码的不开代码轮，射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。定义在 `.claude/agents/`，每个定义的「输入」一节是派发时必须给的东西，缺一样它不开工；共用约束在 `.claude/agent-common.md`。一次性的活派 `clerk`（项目定义，起步不带 CLAUDE.md 与规则），不派 general-purpose；难的显式传 `model: opus`。

## 禁止
- **崩溃点测试准确率和正确率优先，其次再衡量时间成本**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，优先保证正确率和准确率。这一条管枚举域，不管跑的时机：什么时候跑按 `.claude/rules/verification.md`，checker 档默认只在提交时跑快档，全量由用户要求时跑；里程碑收尾、动了崩溃点重放或判器的那一批（暂存改动碰到 D13（验证路线） 已定项 9 那张路径清单）提交之前，先弹窗请用户跑一趟全量（checker-tier-crash-point-replay `--full` 与崩溃放量全量两样，用户同意了主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑）；重型红了必须修好或查清原因，红着不提交；用户不跑的，提交照样可以进，在当前里程碑记录里记一行（日期、「这一阶段收尾没跑全量」），直到补跑。
- **禁止自行扩大任务范围，只改自己的部分**。一个任务（派出去的一件活）一个出口，达成出口即终止任务，扩大任务必须弹窗；一件活可以关多条问题（「一轮怎么开、怎么收」第 4 条的一簇），每条各有验收标准，出口是全部达成。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
- **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**

- **禁止写一点验一点**：一段实现先把代码与测试基本写完，再统一合入、编译、跑测试与验证，次序照「写与验分两段」一节，调度时严格遵守。
- **禁止在subagent中跑重型测试**，重型测试只由主 agent 带 `SINGLEFS_HEAVY_TESTS` 前缀跑，完整清单以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准。轻型三条（`.claude/rules/verification.md`「两档：轻型与重型」）不是重型：实现员、调查员交回前只跑形式化那一条（核心部分加按改到的文件点名的项）；崩溃与变异两条轻型由主 agent 照实现员报告「要主 agent 跑的命令」一节在实现员的副本里跑，红了新派一个同类 agent 接着改，合入后验证也由主 agent 跑。
- **子 agent 不跑要放后台等的命令**（`.claude/hooks/bash-command-detector.sh` 拒子 agent 的 run_in_background）：碰到这种命令它停下交回，报告「要主 agent 跑的命令」一节写着命令原样；主 agent 照原样跑（放后台，看门狗用 `watch.sh --processes` 盯），跑完新派一个同类 agent，派发提示指到它的报告与这次的输出路径，接着做。

## 一轮怎么开、怎么收

1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。把这一轮的任务列全，逐个标它依赖谁、输入齐没齐：没有依赖、输入齐的同类任务各成一批、在同一条消息里同时派——调查各派各的调查员，实验各派各的执行员（分开实验），改 `crates/` 的按第 4 条聚簇（统一实现），写回 kb 的照第 10 条一批一份规格；类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类），改 `crates/` 与改门禁不合给一个 agent。一个主 agent 同时在跑的子 agent 最多 5 个，不分模型（派发闸 ⑦ 拒第 6 个），超出的排下一波。有依赖的只串依赖那一环；散的活不排成一串，合成大块。课题、出口与这张依赖表写进这一轮的调度记录（`records/`），第 7、8 条回头对着它判。派之前按这张表问一遍「哪几件能一起做」，答不出就还没规划完。开工时同时写死这一轮的派发上限与截止批次，依赖表加「批次」列，每份派发提示写一行「批次：<调度记录>#<批号>」；批内冒出的发现只进下一轮的清单，不当场派。
2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就在本轮定（判断与定案在本轮做，改动并进第 4 条的下一批）；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
4. **派实现员之前先聚簇，不为一条发现单派**：把这一轮已抓到、条款已定、要改 `crates/` 的改动全部列出来，按要动的 `crates/` 文件聚簇——文件有交集的并进同一个实现员一次改完（派发提示逐条列问题与各自的验收标准，一个实现员可以关多条，出口是全部达成），互不相交的在同一条消息里同时派、各自的「不碰」清单写上兄弟实现员的文件；同时派几个时一律交补丁（各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `scripts/gate/apply-writer-patch.py` 打），`crates/mutations.tsv` 只追加、不算撞文件；份数不设上限。批内冒出的新发现进下一批清单，凑批再派；照第 2 条判为挡着本轮课题的，也并进下一批，不单派；与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件。写与验分两个阶段，照「写与验分两段」一节：一段实现的写阶段里，上一批交回的补丁只打不验，依赖它的下一批接着派，不等合入后验证与代码轮；要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥）；用户叫停、不写判决的轮，在 `research/prompts/abandoned-rounds.tsv` 登记一行才不再挡，出处写 `records/<文件>.md 第 N 行〔<原话>〕`（或 `records/<文件>.md:N〔<原话>〕`，〔〕里逐字照抄出处那一行里的一段，原话所在的那一行含这一轮的名字；在那份文件里恰好找到一次的取那一行，行号只当提示；命中几处而行号提示正落在其中一处的、找不到而行号提示那一行有这一轮的名字且〔〕与那一行去掉 `.claude/kb/term-renames.md` 登记的新旧名与日期之后〔〕逐字落在那一行里的，取提示那一行；〔〕不写转述），取到的那一行要有轮名与 6 个字以上的「」用户原话，真路径在 `records/` 之内（写法在 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）。治理文件（定义、共用约束、门禁、钩子、规则、研究脚本）的改动同样按批：一周合一批、一批走一轮三方，不为一条发现开一轮；审治理文件的三方要用户点名。
5. **弹窗问用户之前，问句里每一句事实写出处**（产物的整行、命令与输出、文件:行号）；推出来、没量过的，句子里写明「推的，没量过」。
6. **派一个 agent 之前说清它关的是哪几条已经抓到的问题**（可以不止一条，逐条列）；说不出来的，那条该进记录、不该进本轮。调查、调研类的派发点名的是要回答的问题，不是已抓到的问题。
7. **本轮出结论之后回到第 2 步记下延后项的那几处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 8 步的收拢表再判一次**：延下来的哪些本轮开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
8. **收拢要定期做**：每批交回后与每轮结束各做一次，把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。表加一列「折算与挂钟」：数从 `python3 scripts/agents/agent-cost.py usage --since <开工日期>` 与 `timing` 取，不手数。
9. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件。几个会话共写一份文件、里面有不是这一批的改动时，只核这一批自己的改动还在不在，别人的改动不追查、不拦。按内容哈希登记的豁免表（`.claude/crates-review-exempt`、`.claude/agent-def-review-exempt`）遇到文件里有别的会话没审过的改动时不登记，请那个会话自己登记，或等它提交后再登记。别人没提交的改动挡着这一批的活时，用 SendMessage 发给那个会话（`ListAgents` 列的名字）协商，送不到再弹窗让用户转达。
10. **改、写、验都按批，不按件。** 什么时候一起改：第 4 条聚出的一簇一次改；主 agent 自己写的 `crates/` 改动算进同一批。什么时候一起写：同时待写回的全部定案（几轮判决、几次用户定案）合成一批，出一份起草规格（条目少、主 agent 自己写得清的可以自己写），条目多的按 kb 文件不相交切成几份、同时派几个写回员，各自起草规格、跑规格检查、再写（派发提示写「并行写回」：各自不跑 `bash .claude/gate.d/doc-decisions.sh --write`，主 agent 在全部交回后跑一次 `--write`；变更史条目的形态照 `.claude/rules/changelog-format.md`）。什么时候一起验证：一段实现的代码与测试基本写完之后（什么算写完照「写与验分两段」一节；半途交回、失败的先合已完成的部分，补派的归写阶段的下一批）主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--skip-apply] [--memory 上限] [--threads N] <补丁目录…>` 做「合入后验证」（写阶段已打过的补丁带 `--skip-apply`）：打全部补丁、编一次、跑工作区除 checker 档包之外各包的测试（一条 `cargo test -p <包>…`），红了只重跑红的那几条（`scripts/gate/rerun-failed-tests.py` 从日志取命令，不整份重跑），再由主 agent 跑轻型三条的全部（核心部分加这一批补丁改到的文件点名的项，命令照 `.claude/agents/implementation-writer.md` 第 1d 步）（checker 档包的用例不直接跑，归提交时 `checker-tier-crash-point-replay`；崩溃点重放快档也归提交时），红了才派一个 `implementation-writer` 照各份报告的算法改钉值，只对合入时改过锚点或冲突的变异行 `prove-red.sh` 证红（实现员各自证过的不重证，整表归 checker-tier-crates-mutation-replay）；子 agent 交回前、合入后验证都不跑门禁阶段，扫仓的门禁只在提交前由主 agent 照 `.claude/skills/gate/staged-gate.md` 跑的 `scripts/gate/gate-staged.sh` 判（`.claude/agent-common.md`「门禁」一节）；代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮；只改测试代码的不开代码轮（射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）。一件交回就合一件、验一件、开一轮，不许；写一批验一批、改一点验一点，同样不许。
11. **一批任务收尾时清盘**：两台各删本会话在 `/tmp/claude-1000/` 与 `~/.local/share/singlefs/work/` 下不再用的仓副本与编译目录（删之前 `du -sh` 记大小，别的会话的不碰），删完提醒用户跑 `sudo fstrim -v /`（要密码，主 agent 不跑）。
12. **会话过 40 万上下文就压缩上下文接着做，不换新会话，不为交接写调度记录**。一批任务停下来就停，等用户指令；用户说了可以一直跑，就不停，接着开下一批。
13. **提交时的六道重阶段，上限由主 agent 自己定**：给 `checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm 各自的内存上限，每道一个带单位的上限（例 16G），照 `.claude/skills/crash-test/commit-stages.md`「跑之前定下来」一节定：`checker-tier-crash-point-replay` 与 checker-tier-lkmm 写明是量过的还是推的，checker-tier-qemu-device-streams 的不小于同时起的虚机数乘每台的内存；checker-tier-formal-verification 给两个（每个 Verus 进程一个、E165 装置一个），各写明是量过的还是推的；checker-tier-broad-mutation 前面起的广谱变异驱动给每条命令的内存上限（写明是量过的还是推的）与线程数（驱动的 `--threads`，取登记的每台默认重活线程数 `.claude/heavy-thread-default`，不给时驱动自己取它）。

## 写与验分两段

一段实现（一个里程碑步、一组相关的作业点）分两段：先写，后验。主 agent 调度时严格照这一节排次序：写段不夹验证，验证段不夹新功能。

**写段**

1. 开工时把这一段该实现的条款与要补的测试全部列进调度记录的依赖表，照「一轮怎么开、怎么收」第 4 条聚簇；没有依赖的同一条消息里同时派。
2. 有依赖的：前一批实现员交回后，补丁只打不验——打之前 `git apply --numstat` 核几份互不撞文件，再 `python3 scripts/gate/apply-writer-patch.py <补丁目录>` 逐份打进工作树；依赖它的下一批以打过补丁的工作树为基线直接派，不等合入后验证、不等代码轮。
3. 写段不跑：`scripts/gate/merge-and-verify.sh`、轻型三条、代码轮三方、checker 档、实验装置与实验跑片。实现员交回前照它的定义只跑自己动到的测试二进制、fmt、clippy、build 与形式化轻型，证红照做；这是写的一部分，不算验证。
4. `apply-writer-patch.py` 打完报的 code-source-discipline 红，记进调度记录，留到验证段一起看，不为它单派。
5. 写段冒出的新发现：挡着这一段的，并进写段的下一批接着写；不挡的，照「一轮怎么开、怎么收」第 2 条记进延后项。

**什么算写完**：三条都满足才进验证段，逐条写进调度记录——依赖表里这一段该实现的条款都有实现，没有「下一批再补」的行；每条新行为都有测试代码，每处改法都有变异行；剩下的问题只有跑出来才知道（编译红、测试红、变异没红、判决打中），没有已知要写而没写的。

**验证段**

1. 主 agent 跑一次 `bash scripts/gate/merge-and-verify.sh --skip-apply [--memory 上限] [--threads N] <写段打过的全部补丁目录…>`（补丁已打，只编、只验；给全补丁目录是为了重编核对）。
2. 接着跑轻型三条的全部（核心部分加这一段全部补丁改到的文件点名的项），以及各份实现员报告「要主 agent 跑的命令」里的轻型行。
3. 代码轮三方开一轮，快照取在全部合入之后，攻整段的合入。
4. 这一段红的、判决打中的，合成一批派 `implementation-writer` 一次改全；改完回到验证段第 1 步整体再验一次。不逐条改、逐条验。
5. 验证段绿了（红的都改完、整体再验一次绿），实验装置与实验跑片接着自动开始，不等用户点头。

## 派出去之后

盯住，不能一直干等，也不强制结束：结不结束、怎么处置由主 agent 看了定；看门狗报出来的卡住与没带前缀的重型测试，主 agent 在它下一次回报之前处置或 `--ack`；同一件事连报几次（阈值在 `scripts/agents/watch.conf`）主 agent 都没处置、也没 `--ack` 的，看门狗可以终止那个进程（`scripts/proc.py stop`），终止了要报出来；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝，不碰已经在跑的东西，拒哪几种列在共用约束 `.claude/agent-common.md`「不做」一节「执行前拒绝的写法」那一条（主 agent 同样被拒）。

看门狗一个会话一只：会话里第一次派发之后，用 Bash 的 `run_in_background` 起 `bash scripts/agents/watch.sh`（不带 agent 号：会话目录它自己找，之后新派的它自己接上；阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`）。之后的派发不重起它；它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出。它复检子 agent 的会话记录与本实例底下的长进程、读 hook 的检出记录，告警有：没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、hook 检出本会话的问题命令、交回之后后台还在跑（被停的同样算）、交回之后副本还在（交回报告写了「没删 <全路径>」而路径还在，主 agent 删）、进程常驻内存过线（默认 8 GiB）、进程过长（默认 120 分钟）。读到告警先分两类：钩子已经拒掉的事（写被拒、越出写范围、判不了的脚本）与「判不了」类的检出只记进这一轮的调度记录，不处置；卡住（工具调用过长、无动静、结束本轮不会醒、进度文件不涨）与没带前缀的重型测试才处置。它的报告另有两样：有子 agent 交回时（被停的同样算）列草稿根里超阈值（`watch.conf` 的 `scratch-idle-hours`）没动的编译目录与仓副本，只报不删，主 agent 核过没人在用再删；重启前旧进程留下的子 agent 记录标「状态=已失效」，不按无动静报。不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）；要看一个子 agent 在做什么，读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它。

改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。除此之外不给在跑的子 agent 发消息：派发提示一次给全（照「派发提示怎么写」），要补的写进下一次派发。

子 agent 不等长活：要放后台等的命令它交回，主 agent 照共用约束「长活交回主 agent」那一条跑完再新派。

叫醒之后主 agent 判断：只是慢就再起一只看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活交回主 agent」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash scripts/agents/watch.sh --processes`。跑得过后台时限的长命令经 `bash scripts/hosts/detached-run.sh start --wall-clock <within-2h|over-2h> --label <名> -- <命令>` 起（超过 2 小时的写 `over-2h`，被起的脚本文件头要有长活声明，写法在 `scripts/hosts/detached-run.sh` 文件头 start 的 `--wall-clock` 一段），会话里只用 run_in_background 起 `bash scripts/hosts/detached-run.sh watch` 上报还在跑的，要停只用 `detached-run.sh stop <名>`。脱离单元由 `detached-run.sh watch` 上报，`watch.sh --processes` 看不到它们。

## 交回怎么读

判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。

交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，用 `scripts/agents/agent-handover.py` 抽交接摘要给新 agent 读。还没交回、在干活或在等自己后台任务的，只在改了定义、共用约束或规则时收一条消息。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。

## 派发提示怎么写

任务里带重活（编译、测试、变异、产物、原型扫描）的，派发提示写一行「线程上限：N」，N 取登记的每台默认重活线程数（`.claude/heavy-thread-default`，全仓唯一的登记位；`python3 scripts/agents/dispatch-prompt.py <类型>` 打的骨架默认填它）；双机时两台各取这一个数；机器忙时也固定这个数，不看忙闲。N 只写一个十进制整数，这一行里整数后面不接任何字（括注也拒）；要加说明另起一行，照普通正文写；闭合了的围栏代码块（照 CommonMark 认）里的行 ⑮ 不判，标签行照 CommonMark 认与照逐行配围栏认遮不遮不一致的 ⑮ 拒（把围栏写规整）。子 agent 起重活只照「线程上限：」那一个整数，提示里别处写的线程数一概不照（有「多机已判：」一行时逐台写的除外）。用户给了参数、或内存装不下要用别的数的，另起一行「线程上限已判：<理由>」；这一行也能顶替缺的「线程上限：」行，顶替了的派发子 agent 不起重活，要起就停下交回、要主 agent 补一行「线程上限：N」（`.claude/agent-common.md` 起重活那一条）（派发闸 ⑮ 拒 N 不等于登记值又没写已判行的、「线程上限：」一行里在一个整数之外还有别的字的，派 `implementation-writer`、`investigator`、`tooling-writer`、`experiment-runner`、`mutation-triage`、`three-way-attack`、`clerk`、`three-way-forward`、`three-way-defense` 没写「线程上限：」一行又没有已判行的也拒）。用户或别的会话在消息、弹窗里明说要让出 CPU 时（只看到负载忙不算），只改新派的子 agent（派发提示写别的数，带一行「线程上限已判：<理由>」）与主 agent 新起的命令；不为改线程上限给在跑的子 agent 发消息，它们照原样跑完。

长活默认分片、能断点续跑、按配置里的机器自适应：派发提示里不写死台数与额度（几台、哪几张卡、每台多少内存；用户给了逐台参数的，照「多机已判：」那一行的写法写）；预计超过 2 小时的照派发闸 ⑭ 写「分片：」「续跑：」「多机：」三行，「多机：」行与脚本文件头 `# long-run-hosts:` 只认一句规范写法，都逐字等于骨架那一句（`python3 scripts/agents/dispatch-prompt.py <类型>` 打的「多机：」一行里的规范句，原文是 `.claude/hooks/lib_long_run.py` 的 `CANONICAL_HOSTS_SENTENCE`，可带句末句号，后面不接任何说明，说明另起一行写；「分片：」「续跑：」「多机：」各只写一行，同一标签写第二行派发闸拒；闭合了的围栏代码块（照 CommonMark 认）里的行 ⑭ 不判，标签行两种认法遮不遮不一致的 ⑭ 拒），句子里的做法是台数取多机配置（`scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts`）、每台线程取登记的每台默认重活线程数，不现算。子 agent 跑多机只照规范句，提示里别处写的台数、线程取法一概不照。用户给了逐台不同的台数或线程的，另起一行「多机已判：<理由>」（理由写用户怎么给的），逐台参数再另起一行照普通正文写；子 agent 有这一行时照逐台写的参数跑，没有就只照规范句；派发闸 ⑭ 拒没写理由或写了两行的「多机已判：」，「多机：」一行照旧只认规范句。按此刻空闲现算的旧脚本登记在 `.claude/long-run-live-hosts-exempt`，每行带被豁免脚本的 sha256，脚本变了对不上就判红，新脚本不往表里加；豁免表只缩不涨，按（路径、sha256）成对比基准那一版，同一路径 sha256 变了也算涨，判红；基准那一版没有这张表时不比；重钉提交进基准之后不再红；文件头写的读法与脚本做的事是否一致由写的人保证，钩子只判写法。⑭ 判的类型：`implementation-writer`、`investigator`、`tooling-writer`、`experiment-runner`、`mutation-triage`、`three-way-attack`、`clerk`、`three-way-forward`、`three-way-defense`、`experiment-designer`，派这几种都写「预计挂钟：」一行。

门禁 checker-tier-research-build-and-replay 的 research-unit-tests 格与门禁 harness-model-differential-and-scenarios 在阶段里面经内存包装，要换它们的上限就给一个带单位的上限（例 16G），没给用 `scripts/experiments/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值。

在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。

## 什么时候派哪个 agent

| 什么时候 | 派谁、按什么次序 |
|---|---|
| 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `python3 scripts/agents/three-way-materials.py <轮>`（清单默认全抄、索引表与历史节不抄，主 agent 要标例外就改清单再跑 `checklist-specs.py`；正文引到 30 份以上文件、清单要人挑的轮才派 `three-way-materials`）→ 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条；第一轮派正推，辩方只在已有一轮判决可复核时派）、`three-way-attack`（不派本地模型的腿）→ 全部交齐后，主 agent 自己核、自己写判决（`.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」：逐条核腿报告里的原文引用、产物行与复跑命令，代码轮对着开工快照核腿引的行；不派核查员）；如果第一轮结论不一致无法收敛，开第二轮。如果前两轮结论不一致无法收敛，开第三轮。第三轮为最终轮，论证前两轮的结论和判定过程。第三轮结束后不再新开，交给主agent判断 |
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，按「一轮怎么开、怎么收」第 4 条聚簇，一件可关多条、每条各自的验收标准，主 agent 审 diff）→ 一段的代码与测试写完（「写与验分两段」一节），主 agent 跑「合入后验证」（下面那一行）→ 上一行的三方（代码轮，第 10 条：一批一轮；只改测试代码的不开代码轮，射程在 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」）→ 提交时照「暂存之后、提交之前跑门禁」那一行：主 agent 跑 checker 档：`checker-tier-crash-point-replay` 快档、QEMU、herd7、crates 变异表、Verus 形式化验证、广谱变异提交门禁（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；崩溃点重放全量只在用户要求时跑，平时不跑） |
| 一批实现员交回到齐、合入之前 | 有实验登记钉死了常量、还没拍树的（今天是 E171（三oracle合谋测试） 修订 8），先在「主工作区 + 这一批补丁」的副本上跑它的常量核对（E171（三oracle合谋测试） 是装置二进制 `e171_three_oracle_collusion_after_the_oracle_fixes stream-constants --streams 1,2,3`），钉死的常量全过才往下；主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--memory 上限] [--threads N] <补丁目录…>`：打全部补丁（`apply-writer-patch.py`，核不过整批停）、编一次、跑工作区除 checker 档包 `singlefs-checker-tier` 之外各包的全部测试（包从树根 `Cargo.toml` 的 `[workspace] members` 现读）、fmt、clippy、build，红的逐条列出并给只重跑那几条的命令；崩溃与变异两条轻型由主 agent 在各实现员的副本里照其报告「要主 agent 跑的命令」一节跑，红了新派一个同类 agent 接着改，合入后验证由主 agent 跑轻型三条的全部（核心部分加这一批补丁改到的文件点名的项，命令照 `.claude/agents/implementation-writer.md` 第 1d 步）；红了才派一个 `implementation-writer` 照各份报告的算法改钉值（只证合入时改过的变异行）→ 主 agent 审 diff → 上面那一行的三方（只改测试代码的不开代码轮） |
| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 并进同一批的「合入后验证」与三方（代码轮；只改测试代码的不开代码轮）→ 提交时照「暂存之后、提交之前跑门禁」那一行：主 agent 跑 checker 档（`checker-tier-crash-point-replay` 快档、QEMU、herd7、crates 变异表、Verus 形式化验证、广谱变异提交门禁） |
| 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧并进「一轮怎么开、怎么收」第 4 条的下一批交 `implementation-writer`（输入给分诊报告），`research/` 那侧主 agent 先写问题单 → `experiment-designer` 写重跑登记 → `experiment-runner`（续派带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句，派发闸 ⑪） |
| 一批阶段任务结束（里程碑一步、一轮判决、一段实验、一批定义或脚本改完，同时结束的几件算一批、一批做一次），这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `scripts/kb/stale-candidates.py --check-facts`）→ 候选表按组切段，每段约 200 行（参考值：一组超过就整组一段、不拆组），一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再按判定种类抽查：每一种判定各抽不少于 5 条（不足 5 条的全看），对着载体的原文核；抽中判错的那一段整段重派，重派交回再抽一遍，两次都干净才收 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见 `.claude/gate.d/doc-process-records.sh` 文件头 knowledge-sync 格那一段，那一格判形式）→ 下一行 |
| 暂存之后、提交之前跑门禁 | 主 agent 用 `scripts/gate/stage-mine.py` 暂存 → 主 agent 照 `.claude/skills/crash-test/commit-stages.md` 跑这六道：`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-broad-mutation，命令带 `SINGLEFS_HEAVY_TESTS=commit`：`checker-tier-crash-point-replay` 跑快档两趟（`cargo test --release -p singlefs-checker-tier --lib --tests` 不带特性一趟，再带 `tests/` 里 `cfg` 点到的特性一趟，带 `gpu` 的经 `scripts/crash/gpu-quiesce-run.sh` 起；再逐条核崩溃枚举用例的全绿标记，不作数的报「本次未跑」不判红）；`--full` 不默认跑，用户要求时才在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=user-request bash .claude/gate.d/checker-tier-crash-point-replay.sh --full <它的根>`（整条经内存包装，照 `.claude/skills/crash-test/commit-stages.md` 第 1b 步；判绿按输入哈希写全绿标记），其余五道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-crash-point-replay` 并行，checker-tier-crates-mutation-replay 与 `checker-tier-crash-point-replay` 串行，checker-tier-broad-mutation 排在 checker-tier-crates-mutation-replay 之后、先起广谱变异驱动把库目录换到这一次的暂存树、只现判改到的（照 `.claude/skills/crash-test/commit-stages.md` 第 2b 步），有一条没结论或没红就判红、不许提交，内存上限照「一轮怎么开、怎么收」第 13 条各给）→ 主 agent 照 `.claude/skills/gate/staged-gate.md` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `scripts/gate/gate-staged.sh`（它跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green`）并分诊（这六道在 `gate.sh` 里照各自的复用判定走：全绿标记或复用判定作数的不跑，不作数的——提交前没单跑那一道、跑过之后那一道的输入或环境变了——整轮里真跑那一道，`checker-tier-crash-point-replay` 照跑快档、逐条核全绿标记；整轮里不另外直接调这六道）；用户要求时两处都换成 `=user-request` |
| 撤回一个数、改格式常量、新立一条判据 | `sweep` |
| 定案之后写回 kb | `kb-scribe`（写回员：给它判决与条目，它自己起草规格、跑 `scripts/kb/kb-spec-check.py` 到绿、再写；派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧；条目少、主 agent 自己写得清的，主 agent 自己写规格、跑 `scripts/kb/replace-batch.py` 与 `bash .claude/gate.d/doc-decisions.sh --write`；按「一轮怎么开、怎么收」第 10 条一批定案一份规格，按 kb 文件不相交切给几个写回员同时写）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 doc-experiments 的 decision-links 格判两侧同时到位；判一个实验撑不撑一条分项归主 agent）；它翻了分项状态、门禁 code-source-discipline 的 mutation-tables 格红（变异表锚点腐化）→ `scripts/mutation/tables/` 的锚点 `experiment-runner` 只修锚点，`crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs` 并进第 4 条下一批交 `implementation-writer`，书记员不改 `crates/` |
| 改门禁、钩子、研究脚本、看门狗，或照判决改定义与共用约束 | `tooling-writer`（按文件聚簇，一件可关多条，照 `.claude/agents/tooling-writer.md`「输入」给）→ 改了定义与共用约束的走一轮三方或由用户逐份豁免（门禁 doc-process-records 的 agent-def-adversarial-review 那一格） |
| 测试或门禁结果与预期不符 | 先派 `investigator` 复现、二分到文件与行（不先改方向和结论）→ 改法并进第 4 条下一批 |
| 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner`（登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行跑，各写各的产物与报告）→ 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
| 其他相关文件系统的事实 | `prior-art` |
