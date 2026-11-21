# 账本清欠这一轮的治理文件改动：治理文件轮第二轮，攻第一轮判决的修补（ledger-topdown-governance-r2，2026-11-18）

这一轮攻的是第一轮判决（`research/prompts/ledger-topdown-governance-r1-main-verification.md`）坐实的 12 条的修补，都已换上主工作区；另加第一轮写明只按线索记、没现查的地方。辩方腿复核第一轮判决。第一轮攻过而没打中的角度不重复：写闸第十一道对 `src/` 的误拒、登记表条数与文件头开关数、注释历史「缀」「者」两个例外、复跑入口别的派行格里的取法。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 每一处修补做的是不是第一轮判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 第一轮判决够不够得着：坐实的 12 条理由成不成立，认下记账的 8 条该不该在这一轮改，不采的 2 条撤得对不对（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件，读数是任务日志里的原句）：

- 这一轮没有改 `crates/` 下的代码；与 `crates/` 有交点的只有任务服务认的记号行（由 `crates/singlefs-verification/src/gpu_memory_ledger.rs` 与产品 GPU 测试包的取卡模块打出）。
- **任务服务认行内的本次未跑记号**（判决第 9 条）：`task-server/task_server.py` 新函数 `gpu_not_run_marker_start`，认行首、行首带空白、接在 `test <名> ... ` 之后三种，别的夹在行中间的照旧不认；新开关 `gpu-not-run-line-start-only`、新自证格 `gpu-not-run-line-forms`；三处现状句与三语 README 写明不带 `--nocapture` 时读不到。真换 20037，整份自证 20041「自证 408 格都对」，新开关 20039 红 1 格、旧开关 20040 红 2 格与点名相同；服务已重启载入。工具员停下没做的：命令没带 `--nocapture` 时在结局说明里提一句（要定对哪些命令提、提在哪）。
- **换上工具与规格工具**（第 5、6、7、8、11 条）：`scripts/gate/hook-swap.py` 按库名认 `sys.modules[…]` 取出的库、键折不出的列本次未判，提早退出的条件把 `from 库 import` 算作有绑定；`scripts/gate/make-swap-spec.py` 的工作根只从多机配置取、不认命令前的环境变量，临时文件排他新建，六种拒绝分支各一格自证一条开关。真换 20052（带两份自证当验证）；探针 20053、20054 都退 1 并点名缺的名字（换上前同形退 0）；沙箱 20057 退 3「不在工作根下」；规格工具十三条开关 20059 逐格相同；换上工具二十四条开关逐条跑，二十三条相同，`skip-post-swap-check` 多红一格、文件头已补点名（20142 红 4 格与点名相同）；换上后只核不换九份库「520 处引用，缺 0 处，本次未判 10 处」（换上前 359 处、6 处）。工具员写明没做：`sys.modules.get(…)` 与 `from sys import modules` 之后的写法仍走旧路径。
- **写闸与检出钩子**（第 2、3、4、19 条）：`.claude/hooks/bash-command-detector.sh` 新加第十四项，判工具员在 Bash 里往两个服务 `src/` 与 `Cargo.toml`、`Cargo.lock` 下已存在的文件写（追加、`cp`、定点替换工具、`tee`、就地改、python 写文件），主 agent、别的类型、新建文件、草稿与副本放行；`.claude/hooks/write-guard.sh` 第十一道加这几份构建清单，拒绝说明读 `.claude/hooks/selftest-temporary-service.tsv` 分两种出路；`.claude/agents/tooling-writer.md` 第 3、7 步与 `.claude/agent-common.md` 写闸那一句同步。换上前旧版 20118「喂了 9 条，4 条不对」，草稿里新版 20119「0 条不对」；真换 20123、20125；自证 20126「查了 351 种情形」、20127「查了 1221 种」；换上后主工作区喂 20128、20129 各 0 条不对；开关复跑：写闸 20131 四条逐格相同，检出钩子 20132 一百一十一条里一条多红一格、已补点名（20150 红 12 格与点名相同）。工具员写明没做：第十四项不判 `mv` 挪走与 `rm` 删掉已存在的源码，今天没有任何一道拦工具员挪走或删掉服务源码；换上工具的脚本规格没排除两个服务的构建清单；认不出类型的子 agent 第十四项不判。
- **注释历史例外、指针与叫法**（第 10、12、20 条）：`scripts/gate/comment-history-check.py` 里「以前」后接「面」的只在前面紧挨「所」时放过，新开关 `face-exception-widened`、新红样本 `.claude/gate.d/fixtures/doc-text.sh/comment-history-before-face-red/`；`.claude/gate.d/crash-point-replay.sh:294` 与 `scripts/hosts/run-with-memory-cap.sh:73` 的节号指针改指 `records/2026-10-29-subagent拆分提案.md` 的第一百七十节与第一百三十七节 ⑧；`.claude/gate.d/doc-process-records.sh` 三行样本说明改了叫法；`list_check.py:416` 帮助串改了。真换 20098、20099；十九条开关 20100；三道样本档 20109、20110、20111 都判对；真仓命中换上前 872 行、换上后 874 行（多的两行是工具员自己在 `.claude/gate.d/doc-text.sh` 文件头新写的两句），主 agent 给那两行加了特殊缘由标记之后 20145 回到 872 行。工具员写明没查：同两份脚本里另 5 处同类节号指针（`crash-point-replay.sh:265`、`:366`，`run-with-memory-cap.sh:103`、`:112`、`:306`）。
- **两份定义**（第 1 条）：`.claude/agents/implementation-writer.md` 第 3a 步缩域证红那一句与 `.claude/agents/investigator.md:33` 证红那一句的 `--cwd` 改成仓根，主 agent 直接改的，没有自证。
- 第一轮判决的一处更正：它写「写闸今天就拒 `tests/` 下的 `.rs`」不成立；探针 20120、20130 量到第十一道对 `prune-server/tests/` 下已存在的 `.rs` 今天放行，当时那一条退 2 是喂法的旧串以空格收尾撞了另一道。`tests/` 要不要罩进去，第一轮没定。
- 第一轮顺带量到、没在这一轮改的：只有 Read 与 Bash 的那几类子 agent 在 Bash 里往仓里写，写范围那一道不判（现行账本第二百六十七节）。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `task-server/task_server.py`、`task-server/task_server_lib.py`、`task-server/task_server_selftest.py`、`task-server/README.md`
- `scripts/gate/hook-swap.py`、`scripts/gate/make-swap-spec.py`、`scripts/gate/README.md`
- `.claude/hooks/write-guard.sh`、`.claude/hooks/bash-command-detector.sh`
- `.claude/agents/tooling-writer.md`、`.claude/agent-common.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`
- `scripts/gate/comment-history-check.py`、`.claude/gate.d/doc-text.sh`、`.claude/gate.d/crash-point-replay.sh`、`.claude/gate.d/doc-process-records.sh`、`scripts/hosts/run-with-memory-cap.sh`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py`
- `.claude/gate.d/break-switch-replay-files.tsv`（写闸、检出钩子、注释历史判法、规格工具、任务服务五行）

各批工具员的报告（改了什么、没做什么）在 `~/.local/share/singlefs/work/` 下：`governance-r1-fix-not-run-marker/result.md`、`governance-r1-fix-swap-tools/result.md`、`governance-r1-fix-service-source-guards/result.md` 与 `governance-r1-fix-service-source-guards-second/result.md`、`governance-r1-fix-comment-history-and-pointers/result.md`。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中，打中给能照喂的输入或 文件:行）：

1. 检出钩子第十四项：它认的写法之外还有哪些能改到两个服务 `src/` 与构建清单下已存在的文件（`dd`、`install`、`ln -f`、`truncate`、`git apply`、`patch`、重定向到经变量拼出的路径、`mv` 一份草稿盖上去）；`mv` 盖上去算不算它判的「写」；它对主 agent 放行、定义却写两个服务的 `src/`「打补丁」，主 agent 自己直接改有没有别的闸。拒绝说明给的出路过不过别的闸。
2. 写闸拒绝说明分两种出路：读表判哪一种的那一段，表读不到、脚本在表里登的是别的判定值、一份规格里两种脚本都有，各走哪一支、说明写得对不对。
3. 换上工具：这一轮新认的 `from 库 import` 与按库名取库，会不会把不是要核的库的同名模块也算进来（误报缺名字、把该换的挡住）；本次未判从 6 处涨到 10 处，多的 4 处是什么写法、该不该认；工具员写明没做的 `sys.modules.get(…)` 一类今天真仓里有几处。
4. 规格工具：工作根改成只从多机配置取之后，自证里造的小仓、派到第二台跑、多机配置读不到时各是什么结局；排他新建的临时文件在写到一半被杀时留不留垃圾、下一趟拒不拒。
5. 任务服务认记号行的三种形态：给得出被误认的行（测试的普通输出里以 `test ` 起头、带 ` ... ` 再接记号的字面）与仍漏认的形态（多线程时别的测试的输出插在中间、记号前有 ANSI 颜色码、`test <名> ... ` 与记号之间有别的字）；它与门禁那一份库（`scripts/gate/lib_libtest_result_lines.py`）的认法今天差在哪几种行上，差的每一种谁对。
6. 注释历史判法：「面」的例外收窄之后，还有没有「所以前面」之外该放的写法被新判红（真仓 872 行命中里没有新红，判法之外的写法你自己造）；主 agent 给两行加的特殊缘由标记，理由成不成立、该不该改写那两行而不是豁免。
7. 两份定义里证红命令的起点与 `scripts/mutation/prove-red.sh` 今天的行为：仓根起、`--copy` 指副本这一形，证红脚本读的判法、闭包与变异表各取自哪棵树；主工作区与副本的变异表不同时，哪一份说了算。

辩方腿（逐条回「判决站得住」「判决够不着」「推不出」）：

8. 第一轮坐实的 12 条：每条的理由对别的改法是不是同样成立；判决第 7 条说检出钩子那一层的读数不采，判它不采得对不对。
9. 第一轮认下记账的 8 条（换上工具没有锁、退 8 误诊、二十二条开关没有复跑兜底、项目代码闸两条开关的点名、两台上限不同改计时行结局、正文表四处错、定义与表头次序、三处少改）：每条替「该在这一轮改」那一方写出最强的理由再判；其中「换上工具没有锁」今天这一轮连着换了十几趟、两个会话轮流用，判记账够不够。
10. 第一轮不采的 2 条与「核不动的」3 条：第 22 条已由探针推翻（见上），判第一轮当时凭什么写下那一句、同一种错在判决别处还有没有（拿一条退出码当「哪一道拒的」的证据）。
11. 这一轮各批修补是不是照判决「三、改法」做的：多做、少做、做偏的逐条写，工具员各自写明没做的那几件判该不该算这一轮的缺口。

## 分工

攻方腿做第 1 到 7 问，辩方腿做第 8 到 11 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
