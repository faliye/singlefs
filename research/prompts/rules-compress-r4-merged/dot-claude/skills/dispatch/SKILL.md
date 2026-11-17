---
name: dispatch
description: 主 agent 每次派发子 agent 之前、起看门狗之前、被叫醒后处置之前、看或停自己交的任务之前、读交回之前读它（正文不在此刻的上下文里就整份重读）——一轮怎么开怎么收几条的细则、派出去之后怎么盯（看门狗起法与告警处置）、交回怎么读、派发提示怎么写，与派发表各行的细则。
---

# 派发与调度的细则

`.claude/main-agent.md` 同名各节写着判据与骨架；`.claude/skills/dispatch/SKILL.md` 写各条的做法。节名、条号与派发表的行名与 `.claude/main-agent.md` 相同。

## 一轮怎么开、怎么收

**第 1 条**：类别按 agent 定义分（调查、调研、改 `crates/`、改门禁与定义与脚本、写回 kb、跑实验、回扫、分诊、三方腿各是一类）：调查各派各的调查员，实验各派各的执行员（分开实验）；改 `crates/` 的照第 4 条聚簇；写回 kb 的照第 10 条一批一份规格；改 `crates/` 与改门禁不合给一个 agent。有依赖的只串依赖那一环；散的活不排成一串，合成大块。派之前按依赖表问一遍「哪几件能一起做」，答不出就还没规划完。

**第 4 条**：派发提示逐条列问题与各自的验收标准，一个实现员可以关多条，出口是全部达成。互不相交的簇在同一条消息里同时派，各自的「不碰」清单写上兄弟实现员的文件；实现员一律交补丁，只派一个也这样（实现员各在副本里改，交 `crates.patch` 与 `mutations-append.tsv` 等，主 agent 用 `scripts/gate/apply-writer-patch.py` 打；调查员「查到就修」那一支是例外，例外写在 `.claude/main-agent.md`「写与验分两段」写段第 2 条，什么算「查到就修」写在 `.claude/agents/investigator.md`），份数不设上限；`crates/mutations.tsv` 只追加、不算撞文件。照第 2 条判为挡着本轮课题的新发现同样并进下一批，不单派；与在跑的实现员撞文件的等它交回、进下一批，不给在跑的追加文件。写与验的次序照 `.claude/main-agent.md`「写与验分两段」。要改一轮还没判决的代码轮快照里的文件，等那一轮判决（派发闸 ⑥）；用户叫停、不写判决的轮，照 `.claude/skills/three-way/SKILL.md`「用户叫停、不写判决的轮」在 `research/prompts/abandoned-rounds.tsv` 登一行，才不再挡。

**第 8 条**：「折算与挂钟」那一列的数从 `task-server/task submit -- python3 scripts/agents/agent-cost.py usage --since <开工日期>` 与 `timing` 的输出（`task-server/task log <号> --all`）取。闭窗口结局账交 `task-server/task submit -- python3 scripts/agents/agent-cost.py outcome --since <开工日期> --until <这一天> --summary --out <仓根>/records/<轮>-outcome-<这一天>.txt`，落进这一轮调度记录所在的 `records/`，文件名照 `scripts/agents/agent-cost.py` 文件头；`--out` 只排他新建，那个文件已在就换一个名字。

**第 11 条**：中转库指 `.claude/rules/verification.md`「判定库只有一份」点名的、并回服务之后就该删的中转目录。删之前 `du -sh` 记大小。子 agent 在 `.deliverables` 里写成「交接」的副本，照它的报告跑完那几条命令就删。会编的 cargo 经服务交、编译目录由服务给，不往 `<仓根>/target` 编，跑完 `task-server/task clean <号>` 删，不自己 `rm`。要留测试的临时现场，在 submit 前面设 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1`。完工清理闸 `.claude/hooks/completion-cleanup-guard.sh`（挂 Stop）在结束一轮时拦下还留着的这几样与本会话合入后验证留下的快照（判哪几处照它文件头），删完再收；要留的报告、补丁、打补丁前的备份、留着接着用的段树与快照（种类「基线」）照它文件头「要留的交付物」写。删了大的之后提醒用户跑 `sudo fstrim -v /`（要密码，主 agent 不跑）。

**第 15 条**：认会话看变异行与记录里的名字、改它的提交与草稿目录、`ListAgents` 列的会话；查谁改过某个路径或谁提到过某个符号，在本机直接跑 `python3 scripts/agents/who-touched.py <路径或符号>… [--since-minutes N]`，读法照它文件头，Edit / Write 为 0 的会话没改过它。发给那个会话的消息写明文件:行、原话、复现命令与改法建议；认不出是谁的，发给 `ListAgents` 列的每个在跑的会话，请各自认领。

## 派出去之后

派预计超过一小时的活时，告诉用户预计时长与哪几个读数算不正常。

看门狗的起法：用 Bash 的 `run_in_background` 起 `bash scripts/agents/watch.sh`（不带 agent 号，会话目录与之后新派的它自己接上；阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`）。之后的派发不重起它；它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出。它只复检子 agent 的会话记录、读 hook 的检出记录、读工作根所在文件系统的可用量，不读服务、不停任务；任务卡住由服务判（`task-server/task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行），经 `task wait` 叫醒。hook 里检出类的只记录，拒绝类的在执行前拒绝一种危险写法，不碰已经在跑的东西。

看门狗的告警分两组：
- 叫醒主 agent（看门狗退 3）的：一次工具调用超过 8 分钟、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、hook 检出本会话的问题命令、交回之后后台还在跑（被停的同样算）、交回之后副本还在（交回报告写了「没删 <全路径>」而路径还在）、根分区余量过低（工作根所在文件系统的可用量低于 `watch.conf` 的 `root-free-gib-alert`）。
- 只写进报告、不叫醒的：没超时的等待循环、同一命令反复且输出不变、进度文件不涨（`scripts/agents/agent-watch.py` 的 `INFO_ALERT_NAMES`），以及 hook 检出里钩子执行前已拒掉的与钩子判不了而照常放行的。

读到告警的处置：
- 只记进这一轮的调度记录、不处置的：报告里标「只记不叫醒」的行，hook 检出里钩子已经拒掉的事（写被拒、越出写范围）与钩子判不了而照常放行的检出（项目代码闸没判全、各闸「没判、照常放行」与「没判全」）。
- 卡住的（工具调用过长、无动静、结束本轮不会醒）才处置，在看门狗下一次回报之前处置或 `--ack`。
- 别的叫醒告警照告警里「→ 怎么办」那一行办：按模式找进程，看它等的或要杀的是哪个进程，发消息让它改用写死的 pid；交回之后后台还在跑，确认没用就按进程号停（`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`），要留着就 `--ack 进程:<pid>`；交回之后副本还在，核过没有进程在用（工作目录、命令行、`CARGO_TARGET_DIR` 指到它）由主 agent 删；根分区余量过低，先查各会话的副本、再查崩溃放量家目录 `runs/` 下的运行目录，以 `du` 为准、只删没有进程在用的，一时删不了、确认接着跑的用 `--ack 类:根分区余量过低`。

叫醒之后主 agent 判断：只是慢就再起一只看门狗接着盯，确认只是慢的带 `--ack` 再起，键照 `scripts/agents/agent-watch.py` 文件头「--ack 的写法」三种：`进程:<pid>`、`<agent 号>:<告警名>`、`类:<告警名>`（这一类的全部，范围最大）；是问题就决定发消息让它改、停掉重派、还是报给用户。看门狗在有子 agent 交回时（被停的同样算）另列草稿根里超过 `watch.conf` 的 `scratch-idle-hours` 没动的编译目录与仓副本，只报不删，主 agent 核过没人在用再删；重启前旧进程留下的子 agent 记录标「状态=已失效」，不按无动静报。不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）；要看一个子 agent 在做什么，读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它。

坏卡与服务健康不由看门狗报。交的任务结局是 `card-fault`（`task wait` 末行 `type=card-fault`、退 237）时，当场弹窗告诉用户哪台哪张卡与依据（任务日志里的清场行）、请用户重置那张卡，不等、不重试。task-server 把任务的结局、要处置的提醒与服务健康毛病直接发消息给发起任务的会话，收到就处置；服务重启之前交的任务没带会话地址，只能靠 `task-server/task wait <号>` 取结局；处置后的回读照 `.claude/main-agent.md`「派出去之后」；`python3 task-server/control check` 缺一样退 78，出路写在输出里。

主 agent 自己交的任务（长的同样）照 `.claude/agent-common.md`「长活交回主 agent」：看排队与日志用 `task-server/task status <号>`、`task-server/task log <号>`，停用 `task-server/task cancel <号>`；`cancel`、`clean` 只对自己交的号做。`task wait` 以 `needs-attention` 退出时怎么办照 `.claude/main-agent.md`「派出去之后」；退出码以 `task-server/task` 文件头里 `task wait <号>` 那一条与「退出码（wait 之外）」那一段为准（`needs-attention` 退 230）。

## 交回怎么读

子 agent 报告「要主 agent 跑的命令」一节的命令，主 agent 照原样经服务交、后台起 `task-server/task wait <号>` 等叫醒，跑完新派一个同类 agent，派发提示写一行「承接：<它的报告路径>」，并指到这次的任务号与输出路径。

要它会话里的进度，经 `task-server/task submit -- python3 scripts/agents/agent-handover.py …`（`--out` 写工作根 `~/.local/share/singlefs/work/` 下的草稿目录）抽交接摘要给新 agent 读。

后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知，分三种读：
- 有交回消息的：照交回判。
- 结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的：它起了后台任务，越出 `.claude/agent-common.md`「子 agent 不交任务」。看它起了什么，发消息让它停掉那个任务、把命令写进报告「要主 agent 跑的命令」一节交回；它交过的任务号照停掉子 agent 的那一套核出、逐个 `cancel` 或 `clean`。
- 没有交回消息、note 与 result 里也没有这两句的：它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）。看它最后在等什么，发消息让它接着做或交回，或者停掉。

主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。停掉之后照它的会话记录或草稿目录核出它交的任务号：还在排、在跑的逐个 `task-server/task cancel <号>`（服务当场删掉那一条的编译目录与现场，删了几份打在 cancel 的输出里；没删成的 cancel 退 5，照它的出路处置），有了结局的逐个 `task-server/task clean <号>`。

## 派发提示怎么写

派发提示与子 agent 写进报告的命令里，不写线程、内存上限、台数与卡、分片与续跑：线程固定是登记的每台默认重活线程数（`.claude/heavy-thread-default`）、不许覆盖，其余由服务定。

派发提示另要三样，骨架都打出：派有项目定义的类型写一节「## 关的是哪几条」（调查、调研类可写成「## 要回答的问题」），节下不空；派 `implementation-writer`、`tooling-writer` 写一行行首标签里带「不碰」的（骨架打「不碰：」），值是这一轮别的会话在改、这一件不碰的文件，没有写「无」；派 `experiment-runner` 续做写一行「承接：<上一段执行员的报告路径>」，点名的报告要在。每份派发提示写一行「批次：<调度记录>#<批号>」。

派发提示里的报告路径，文件名里不带 report；没有别的形态要求的写 `<草稿目录>/result.md`，三方腿照 `.claude/rules/three-way-inference.md` 写 `research/prompts/<轮>-*-output.md`。

写简练的做法：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉；嫌长就指到文件，不删内容。要对方照仓里已有的某处口径做的，写出那一处的文件与行号或整段原文，不写自己的转述；写之前先现读那一处。

## 什么时候派哪个 agent

**「要推论」那一行**：`three-way-materials.py` 的清单默认全抄、索引表与历史节不抄，要标例外就改清单再跑 `checklist-specs.py`。

**「一批实现员交回到齐、合入之前」那一行**：E171（三oracle合谋测试） 的常量核对是装置二进制 `e171_three_oracle_collusion_after_the_oracle_fixes stream-constants --streams 1,2,3`。

**「跑变异表、看抓到 / 无效 / 没红」那一行**：`research/` 那侧的表照 `mutation-triage` 交回的 `mutate.sh` 命令或主 agent 自己写的同一条跑，`crates/mutations.tsv` 那侧是提交时的 crates-mutation-replay。`crates/` 那侧交实现员时输入给分诊报告。续派执行员带「这一段回答的岔路：…」与「上一段岔路表里还差：…」两句（派发闸 ⑪）。

**「一批阶段任务结束」那一行**：
- 事实表那一趟，派发提示的输入给改动范围与做成的事，只用过、没留改动的也写；事实表的验收照 `.claude/main-agent.md` 派发表这一行；工作区里有没提交的改动时，主 agent 先交 `task-server/task submit -- python3 scripts/kb/batch-target.py` 只拿这一批的路径与块做一个提交对象，哈希写进派发提示当 `--target`。
- 候选表按组切段，每段约 200 行（一组超过就整组一段、不拆组）；逐行判那几趟的输入同样给做成的事。
- 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退 0 才往下；每一种判定各抽不少于 5 条（不足 5 条的全看）对着载体的原文核，抽中判错的那一段整段重派、交回再抽，两次都干净才收。
- 回扫报告「交实现员」的行是目标在 `crates/` 下的「要改」「要补」；钉值类的照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条先定条款再派 `implementation-writer`，不交写回员。
- `research/prompts/<阶段>-sync.md` 的格式见 `.claude/gate.d/doc-process-records.sh` 文件头 knowledge-sync 格那一段。

**「定案之后写回 kb」那一行**：
- 写回员拿到判决与条目，自己起草规格、跑 `scripts/kb/kb-spec-check.py` 到绿、再写，规格与写法照它的定义；派发提示写一行「规格检查已判：写回员自己起草并跑规格检查」过派发闸 ⑧。主 agent 自己写的，写规格、跑 `scripts/kb/replace-batch.py`。
- 写回员把 decision-items-sync 那一条写进报告，全部写完后主 agent 交一次 `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-decisions.sh 的 decision-items-sync 格（--write）; 范围=1 格; 为什么=…' task-server/task submit -- bash .claude/gate.d/doc-decisions.sh --write --check decision-items-sync`。
- 规格动了某条分项「**依据**」段时，实验页配对行照 `.claude/agents/kb-scribe.md`「输入（主 agent 必须给）」给；判一个实验撑不撑一条分项归主 agent。
- 变异表锚点腐化指翻了分项状态之后门禁 code-source-discipline 的 mutation-tables 格红；`crates/` 下要交实现员的是 `crates/mutations.tsv` 的锚点与 `relabel-item.py` 列出的 `crates/` 下的 `.rs`（写回员只许写 `crates/` 下的不变量清单 `.md`，照 `.claude/agents/kb-scribe.md`「写范围」）。

**「要建计数实验，或重跑已有实验」那一行**：
- 登记写完，主 agent 删掉登记里待删的问法（有的话），再派执行员。
- 登记分了几段、段与段不共用要改的装置与产物的：先派一个执行员把装置、单测与变异表写完，再一段派一个执行员并行交各段的跑法、读各段的产物，各写各的报告。
- 主 agent 照执行员报告「要主 agent 跑的命令」一节起编译、单测、变异与产物，跑完新派执行员（带「承接：」）读输出、写实验页。
- 续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句）。
