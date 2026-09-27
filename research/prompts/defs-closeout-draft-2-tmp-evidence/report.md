# 定义改动第二批：交回报告（2026-09-26 JST 09:0x）

## 一、结论一览

| 项 | 做了什么 | 在哪 |
|---|---|---|
| 1 主 agent 第 3 条重型测试清单 | 括号里的六类删掉，改成指到 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单（那张清单里有 87 号全部实验复跑与 E152 装置），不抄第二份 | `.claude/main-agent.md:20` |
| 2 拒绝清单统一 | 列全的一处放在 `.claude/agent-common.md`「不做」一节新加的「执行前拒绝的写法」一条（按 hook 分组，Bash 检出 hook 按它自己的 ①–⑦ 编号，含 ⑥ 终止进程、⑦ 原地改写已有脚本）；`main-agent.md`「派出去之后」第一段与 `agent-common.md` 原来「长活可以等」那一条里的清单都改成指过去 | `agent-common.md` 新条；`main-agent.md:30`；`agent-common.md`「长活可以等」第三行 |
| 3 `mutations-append.tsv` | 是写法错了：同一份定义第 3、5 步、「写范围」「产出」说的都是 `crates/mutations.tsv`，改成「追加进 `crates/mutations.tsv` 末尾」 | `implementation-writer.md:29` 第 4 步 |
| 4 跑二进制经内存包装 | `agent-common.md` 原来没有这一句（改前 `grep -n 'run-with-memory-cap' .claude/agent-common.md` 零命中），加「跑编译出来的代码经内存包装」一条；执行员第 1b 步改成指过去，崩溃验证员、门禁分诊员、实现员各加第 1b 步指过去 | 四份定义「做什么」第 1b 步 |
| 5 攻方「内存与进程」 | 收进 `three-way-attack.md` 第 3c 步，紧跟第 3b 步 | `three-way-attack.md` |
| 6 84 号归属 | 84 号文件头判的是「`replay.sh` 登记的入库产物里判决行 `字段=false` 要在实验页里被点名」，与执行员第 4c 步同一件事，登记给 `experiment-runner` | `stage-owners.tsv` 第 48 行 |
| 记录 | 第四十节第 30 行现状格「欠：崩溃验证员、门禁分诊员、实现员…」那一段改成「定义已改、待定义三方（2026-09-26 JST）」＋改了哪份文件哪一节，剩下的欠账照留 | `records/2026-09-16-subagent拆分提案.md` 第 941 行 |
| 门禁 | 47、62、63、doc-lint、规则纪律（项目本地）全绿；62 号由红转绿 | 第四节 |

只改了派发点名的八份文件与记录那一格：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/agents/{implementation-writer,crash-verifier,gate-triage,experiment-runner,three-way-attack}.md`、`.claude/gate.d/stage-owners.tsv`。hooks、SOP 副本、`crates/`、kb、`CLAUDE.md` 没碰。

## 二、两份 hook 今天实际拒哪些（现查）

做法：往 hook 的标准输入喂 hook JSON，只看退出码（2 = 拒），被判的命令一条都没执行。脚本 `/tmp/claude-1000/defs-closeout-draft-2/probe_hooks.py`、`probe_heavy.py`、`probe_heredoc.py`，原样输出 `/tmp/claude-1000/defs-closeout-draft-2/probe-hooks.log`。另读了两份 hook 的入口：`bash-command-detector.sh` 的 `main()`（第 2339 行起，`return 2` 在第 2386–2453 行的七处）依次调 `watchdog_rejection`、`process_signal_refusal`、`detaching_refusal`、`unwaited_ampersand_refusal`、`wait_loop_refusal`、`results_overwrite_refusal`、`script_in_place_refusal`，命中就 `return 2`，一共七种；`write-guard.sh` 的 `decide()`（第 159–170 行）依次三道。`.claude/settings.json` 里 `write-guard.sh` 挂在 `Write|Edit`（没挂 MultiEdit），`bash-command-detector.sh`、`heavy-test-guard.sh`、`pattern-process-guard.sh` 挂在 `Bash`。

`bash-command-detector.sh`（原样输出，每行末尾是 stderr 第一行的前 120 字）：

```
bash-command-detector	① 看门狗前台起	exit=2	✗ 看门狗起法不对（run_in_background 不是 true；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台加 &	exit=2	✗ 看门狗起法不对（命令里有单独的 &；认出的调用：watch.sh abc123）：这样起的看门狗叫不醒主 agent，等于没盯
bash-command-detector	① 看门狗后台正确起（对照放行）	exit=0	
bash-command-detector	② 前台无超时等待循环	exit=2	✗ 前台的等待循环没有超时（认出的循环：until grep -q x log）：等的条件不成立就一直不返回，这一次调用卡在这里期间，发给你的消息也送不到
bash-command-detector	② 外套 timeout（对照放行）	exit=0	
bash-command-detector	③ disown	exit=2	✗ 把活放出了追踪（认出的写法：disown）：…
bash-command-detector	③ nohup … &	exit=2	✗ 把活放出了追踪（认出的写法：nohup … &）：…
bash-command-detector	③ setsid -f	exit=2	✗ 把活放出了追踪（认出的写法：setsid -f）：…
bash-command-detector	④ 后台里单独 & 无 wait	exit=2	✗ run_in_background 里又把活放到了后台（以单独的 & 收尾、之后同一条命令里没有 wait 的作业：sleep 100）：…
bash-command-detector	④ a & b & wait（对照放行）	exit=0	
bash-command-detector	⑤ > 覆盖未跟踪产物	exit=2	✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：> research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out）：旧
bash-command-detector	⑤ >| 覆盖未跟踪产物	exit=2	（同上形，认出 >|）
bash-command-detector	⑤ &> 覆盖未跟踪产物	exit=2	（认出 &>）
bash-command-detector	⑤ tee 不带 -a	exit=2	（认出 tee）
bash-command-detector	⑤ cp 覆盖	exit=2	（认出 cp）
bash-command-detector	⑤ mv 覆盖	exit=2	（认出 mv）
bash-command-detector	⑤ install 覆盖	exit=2	（认出 install）
bash-command-detector	⑤ dd of=	exit=2	（认出 dd）
bash-command-detector	⑤ truncate	exit=2	（认出 truncate）
bash-command-detector	⑤ >> 追加（对照放行）	exit=0	
bash-command-detector	⑥ kill 负号	exit=2	✗ 终止进程的写法打得到不是自己点名的那一个进程（kill：目标 -1 带负号，是整个进程组（-1 是能发到的全部进程）（kill -9 -1））：…
bash-command-detector	⑥ kill 0	exit=2	✗ …（kill：目标 0 是发这条命令的整个进程组（kill 0））…
bash-command-detector	⑥ kill 两个目标	exit=2	✗ …（kill 一次给了 2 个目标（999991 999992）：一次只停点名的一个…）
bash-command-detector	⑥ kill 命令替换	exit=2	✗ …（kill：目标 $(pgrep sleep) 是命令替换…）
bash-command-detector	⑥ 循环里 kill	exit=2	✗ …（kill 在 for / while / until 循环里逐个发：这是批量停，不是点名停一个…）
bash-command-detector	⑥ proc.py stop 循环	exit=2	✗ …（proc.py stop 在 for / while / until 循环里逐个发…）
bash-command-detector	⑥ systemctl stop ssh	exit=2	✗ …（systemctl stop ssh：这是 SSH、登录会话、用户级 systemd、本地模型服务或它们依赖的系统服务…）
bash-command-detector	⑥ kill "$!"（对照放行）	exit=0	
bash-command-detector	⑥ proc.py stop 单个（对照放行）	exit=0	
bash-command-detector	⑦ > 覆盖已有脚本	exit=2	✗ 要在同一个 inode 上改一个已经存在的脚本（认出的写法与目标：> research/scripts/replay.sh）：…
bash-command-detector	⑦ cp 覆盖已有脚本	exit=2	（认出 cp）
bash-command-detector	⑦ tee 覆盖已有脚本	exit=2	（认出 tee）
bash-command-detector	⑦ python open w	exit=2	（认出 python open(…, 'w')）
bash-command-detector	⑦ mv 换上（对照放行）	exit=0	
bash-command-detector	⑦ >> 追加（对照放行）	exit=0	
bash-command-detector	旁：sed -i 改脚本	exit=0	
```

（这一段为省篇幅把重复的 stderr 尾巴写成了「…」「（认出 X）」，完整原样在 `probe-hooks.log`。）

`write-guard.sh`（原样）：

```
write-guard	一 Write 覆盖未跟踪已有文件	exit=2	✗ 拒绝用 Write 整份覆盖 /home/fy5090/code/singlefs/research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out：它已存在而且没进 git
write-guard	一 Write 新文件（对照放行）	exit=0	
write-guard	二 子 agent 越出写范围	exit=2	✗ experiment-runner 的写范围不含 /home/fy5090/code/singlefs/.claude/main-agent.md
write-guard	二 未登记的项目 agent	exit=2	✗ gate-triage 在写范围表里没有登记，按拒绝处理：/home/fy5090/code/singlefs/README.md
write-guard	三 Edit 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 （U+2032）在「…A…」（/tmp/claude-1000/x.md，这次写的内容里共 1 处）
write-guard	三 Write 写进撇号角标	exit=2	✗ 写进去的内容里有撇号类角标 （U+02B9）在「…B…」（/tmp/claude-1000/probe-new-xyz.md，这次写的内容里共 1 处）
```

（最后两行里的角标字符本身在这份报告里删掉了，原样在 `probe-hooks.log`。）

清单里另收了两道同挂 Bash、同样在执行前拒绝的 hook，也现查了：

```
pattern-process-guard	pgrep -f foo	exit=2
pattern-process-guard	pkill -f foo	exit=2
pattern-process-guard	killall foo	exit=2
pattern-process-guard	grep -n pgrep x.sh	exit=0
heavy-test-guard	crash-verifier	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（crash-verifier 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（gate-triage 不经 run-with-memory-cap.sh）
heavy-test-guard	gate-triage	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo test -p singlefs-core --lib	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
heavy-test-guard	implementation-writer	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-core --lib	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo build --offline --all-targets	exit=0	
heavy-test-guard	implementation-writer	nice -n 19 cargo clippy --all-targets	exit=0	
heavy-test-guard	experiment-runner	nice -n 19 cargo run --release --bin e142-first-txn-dry-run	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo run（experiment-runner 不经 run-with-memory-cap.sh）
heavy-test-guard	experiment-runner	nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G cargo run --release --bin e142-first-txn-dry-run	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/54-layer0-replay.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/55-qemu-first-transaction.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/57-lkmm.sh	exit=0	
heavy-test-guard	crash-verifier	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh	exit=0	
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged	exit=0	
heavy-test-guard	gate-triage	SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/87-replay.sh	exit=0	
heavy-test-guard	implementation-writer	'bash research/scripts/run-with-memory-cap.sh 8G bash <<\'X\'\ncargo test -p singlefs-core --lib\nX'	exit=0	
heavy-test-guard	implementation-writer	"bash research/scripts/run-with-memory-cap.sh 8G bash -c 'cargo test -p singlefs-core --lib'"	exit=0	
```

据此写进清单的与没写进的：

- 写进：`bash-command-detector.sh` ①–⑦（每种至少一例拒、一例对照放行现查过；⑤ 的 `>&` 文件、带 fd 号的 `2>`，⑥ 的 pkill / fuser -k / cgroup.kill / loginctl / 关机，⑦ 的 `>|`、`&>`、`dd`、`truncate`、`Path.write_text` 这些没逐个喂，照 hook 文件头第 71–132 行写的归类，没写进逐字清单的用「这一类」收）；`write-guard.sh` 三道（都现查过）；`pattern-process-guard.sh`（现查过）与 `heavy-test-guard.sh` 的两类拒绝（重型测试、子 agent 不经包装跑编译出来的代码，后一类现查过）。
- 收这两道的理由：`agent-common.md` 改前那句写「run_in_background 里的等待循环与按模式找进程这类命令只记检出」，按模式找进程今天由 `pattern-process-guard.sh` 在执行前拒（上面第 1–3 行），那半句是错的；`main-agent.md` 改前的清单里本来就有「子 agent 跑重型测试」一项（`heavy-test-guard.sh`）。
- 没写进：只记检出、不拒的（run_in_background 里的等待循环、起了几个 `&` 而 `wait "$pid"` 只等了其中一个、`start-stop-daemon -b`、⑤ 目标这一刻算不出的）；拒的不是 Bash 命令写法的几道（`continuation-guard.sh`、`runner-dispatch-guard.sh`、`ask-user-claim-guard.sh`、`handback-scratch-check.sh`，各自在 `main-agent.md` 与共用约束别处已经点名）。
- `sed -i` 改已有脚本 ⑦ 不拒（exit=0），清单里没列它。
- `heavy-test-guard.sh` 文件头第 81 行说 `run-with-memory-cap.sh 4G bash <<EOF` 会被误拒，现查退 0（上面倒数第 2 行），所以没把「不写成 heredoc」写进共用约束。hook 文件头与实际对不上这一处，没改 hook，交主 agent。

## 三、每处改了什么（diff）

行号（今天工作区）：`.claude/main-agent.md:20`、`:30`；`.claude/agent-common.md:48`（新条「跑编译出来的代码经内存包装」）、`:58` 第三行（「长活可以等」里原来的清单）、`:59`–`:70`（新条「执行前拒绝的写法」）；`.claude/agents/implementation-writer.md:26`（1b）、`:29`（第 4 步）；`crash-verifier.md:25`、`gate-triage.md:26`、`experiment-runner.md:27`（1b）；`three-way-attack.md:34`–`:39`（3c）；`.claude/gate.d/stage-owners.tsv:48`。

下面是对我开工时拷的改前副本（`/tmp/claude-1000/defs-closeout-draft-2/before/`）的 diff，只含这一批（`experiment-runner.md`、`three-way-attack.md` 里第一批的改动不在里面）；上下文 1 行，`records` 另列。完整带 3 行上下文的在 `batch2-final.diff`（sha256 8cee2b0e744dd2bce42babc0826d18df42ff6ac32ba9e6192fff98283419a5be）。

```diff
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -19,3 +19,3 @@
 2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
-3. **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。
+3. **重型测试**（哪几样以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准）提交之外任务确实要跑，先弹窗问用户，同意了才跑。
 4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
@@ -29,3 +29,3 @@
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（起看门狗的错误写法、前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、用 shell 的 `>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp`、`mv`、`install` 整份覆盖 `research/results/` 下已存在且未跟踪的产物、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝，不碰已经在跑的东西，拒哪几种列在共用约束 `.claude/agent-common.md`「不做」一节「执行前拒绝的写法」那一条（主 agent 同样被拒）。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -47,2 +47,3 @@
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
+- 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
@@ -56,3 +57,15 @@
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对五种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里的等待循环只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+- 执行前拒绝的写法：项目 settings 里的几道 hook 在执行前拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
+  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
+    ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
+    ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
+    ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。
+    ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
+    ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
+    ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
+  - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
+  - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
+  - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
 
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -25,5 +25,6 @@
 1. 开跑前照共用约束「不做」一节看负载。
+1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -24,2 +24,3 @@
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，上限与退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -25,2 +25,3 @@
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
 2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -26,3 +26,3 @@
 1. 开跑前照共用约束「不做」一节看负载。登记第六节里有耗时类的量时，跑产物之前连别的 `cargo`、`gate.sh` 也要等。
-1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（cargo 编出来的可执行文件，例 `research/target/release/e<号>-<简称>`）一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>`；写进脚本的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取跑前登记或派发提示给的，都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`mutate.sh` 与 `replay.sh` 在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算产物，命令与退出码写进报告，不绕开包装重跑。
+1b. 跑单测（`cargo test`）、`cargo run` 与装置二进制（例 `research/target/release/e<号>-<简称>`）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；上限先取跑前登记给的，没给再照那一条取；包装退出码 250–254 的那一次输出不算产物。
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -33,2 +33,8 @@
    - 随机跑批小批量、限时，每批的段数与限时写进报告。
+3c. 内存与进程：
+   - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
+   - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
+   - 并行起的几件各记下 `$!`，逐个 `wait "$pid"` 收退出码。
+   - 等一行字之前先确认那一行真会写进那个文件。
+   - 不改正在跑的脚本，要改的写同目录临时文件再 `mv` 换上（共用约束「执行前拒绝的写法」那一条的 ⑦）。
 4. 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
--- a/.claude/gate.d/stage-owners.tsv
+++ b/.claude/gate.d/stage-owners.tsv
@@ -47,2 +47,3 @@
 80-absolute-assertions.sh	experiment-runner	实验钉绝对值的断言
+84-verdict-false-named.sh	experiment-runner	实验产物判决行里的 false 字段要被实验页点名：判决行由执行员交回之前逐行读、实验页由它写，写完就跑
 85-repro-command.sh	experiment-runner	实验复跑命令
```

记录第四十节第 30 行现状格，`research/scripts/replace-once.py` 定点替换（「命中 1 次，已替换（改名换上新 inode）并回读确认」）。换掉的旧串：

> 欠：崩溃验证员、门禁分诊员、实现员的定义里还没写经包装跑（改定义走 72 号那一轮三方）；执行员的定义已改、待定义三方（2026-09-26 JST：`experiment-runner.md`「做什么」加第 1b 步，单测、`cargo run` 与装置二进制一律经 `run-with-memory-cap.sh`，`mutate.sh`、`replay.sh` 自己在里面套了、外面不再包；`agent-common.md` 里没有同类的一句，没改）；`research/scripts/mutate.sh` 每条变异外面套

换成：

> 定义已改、待定义三方（2026-09-26 JST）：`agent-common.md`「不做」一节加「跑编译出来的代码经内存包装」一条（`cargo test` / `run` / `bench` 与 cargo 编出来的二进制一律经 `run-with-memory-cap.sh`，写进自己脚本的整条经它；上限取派发提示给的、没给取 `replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值；`mutate.sh`、`replay.sh`、59 号在里面套了、外面不再包；退出码 250–254 的那一次不算结果；`cargo build` / `clippy` / `fmt` 不要求），执行员、崩溃验证员、门禁分诊员、实现员四份定义「做什么」的第 1b 步都指过去：`experiment-runner.md` 第 1b 步改成指到这一条、上限先取跑前登记给的；`crash-verifier.md` 加第 1b 步，54、55、57 号整条经包装跑、59 号直接跑；`gate-triage.md` 加第 1b 步，自己单跑的 `cargo test`、测试二进制经包装，`gate.sh` 与单跑的门禁阶段直接跑；`implementation-writer.md` 加第 1b 步，第 3 步证红与第 4 步动到的测试二进制经包装，fmt、clippy、build 不经。欠：`research/scripts/mutate.sh` 每条变异外面套

格子后半（`mutate.sh` 的 timeout、按名字判的门禁阶段里的 cargo、slice 外面涨过余量、两条老包装）原样留着，仍是欠账。

## 四、门禁判定行（原样）

跑的时刻 2026-09-25 UTC 23:58–2026-09-26 UTC 00:02（JST 08:58–09:02），开跑前 `ps -o pid,etime,args -u "$(id -u)"` 过滤 `qemu-system|vm-bench|e152|fio|cargo|gate\.sh` 零命中。八份文件的最后改动（`three-way-attack.md` 23:58:17 UTC）早于 62、63、doc-lint、规则纪律开跑（47 号 00:00:38 UTC 结束之后才起）；47 号不读这几份定义。日志在 `/tmp/claude-1000/defs-closeout-draft-2/gate-*.log`、`doc-lint.log`、`rules-lint.log`，汇总 `gates-summary.log`：

```
47-research-script-selftests.sh exit=0
62-stage-owners.sh exit=0
63-agent-write-scope.sh exit=0
doc-lint exit=0
rules-lint exit=0
```

- 47 号 `nice -n 19 bash .claude/gate.d/47-research-script-selftests.sh`：
  `  ✓ research 脚本的自证都通过（本阶段跑了 31 条；research/scripts/ 里声称有 --selftest 的 35 份中 34 份有门禁阶段在跑）`
- 62 号 `nice -n 19 bash .claude/gate.d/62-stage-owners.sh`（改前同一条命令退 1，红在「84-verdict-false-named.sh」没登记）：
  `  ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）`
- 63 号 `nice -n 19 bash .claude/gate.d/63-agent-write-scope.sh`：
  `  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 12 个文件），共用重型测试判定模块的 15 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 12 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT）`
- doc-lint `nice -n 19 bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`：
  `  ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部）`
- 规则纪律（项目本地），照 `.claude/singlefs-ai-sop/scripts/gate.sh` 第 418–419 行的接法：`GATE_IN_STAGE=1 RULES_LINT_DIR="$PWD/.claude/rules" RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md" nice -n 19 bash .claude/singlefs-ai-sop/scripts/rules-lint.sh "$PWD"`：
  `  ✓ 规则只写怎么做（扫了 29 份文件 1772 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在「」或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1649 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=）`

72 号没跑：这一批与第一批的定义都还没有判决，跑了必红，等主 agent 那一轮。

## 五、要三方判的点（我自己定的、推的）

1. **崩溃验证员 54、55、57 号的上限**：第 1b 步照记录第 30 行 ④「54、55、57 号不改，整条经包装跑」写；上限照共用约束取派发提示给的、没给取 `replay.sh` 的默认值 `8G`（`research/scripts/replay.sh` 第 24 行 `REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"`）。记录同一格写着「三道要多少内存都没量过」，例子用的是 16G：8G 够不够层 0 全量、够不够 55 号的虚机，推的，没量过；不够时撞 250 / 254，照共用约束交回、不绕开重跑，代价是那一趟白跑。要不要在崩溃验证员「输入」里把上限列成必给项，三方定。
2. **门禁分诊员「`gate.sh` 与单跑的门禁阶段直接跑，外面不包一层」**：hook 不要求（第二节 `gate.sh --staged`、87 号退 0）。不包的推断是：`gate.sh` 里 59 号、87 号（经 `replay.sh`）在里面逐条套了，外面再包就是嵌套，按包装文件头「已占」的算法可能重复记账、排到 252（第一批报告第二节第 5 条同一个推断，没量过）。代价是 `gate.sh` 里 `check.sh`、15 号、74 号起的 cargo 不在任何包装里，这正是记录第 30 行现状格里留着的欠账「按名字判的门禁阶段（15、74 号这些）里起的 cargo 不在闸的射程里」，定义这一侧没补它。
3. **攻方第 3c 步「`cargo build` 也经它」**：照 `_m2-rollback-forward-r3-body.md` 第 59 行「编译与跑一律经」，比共用约束严（共用约束写 `cargo build` 不要求）；两处不矛盾，但同一件事两种要求，三方看要不要统一。
4. **「执行前拒绝的写法」一条的射程**：派发只点了两份 hook，我另收了 `pattern-process-guard.sh` 与 `heavy-test-guard.sh`（第二节写了理由与现查）。⑤⑥⑦ 里没逐个喂的形态是照 hook 文件头归类的，写成「这一类」，没写成穷举。
5. **main-agent 第 3 条只指不列**：87 号与 E152 装置靠指过去的那张清单罩着，字面不再出现在第 3 条里；主 agent 的上下文里有 `implementation-workflow.md`（`CLAUDE.md` `@` 了它）。
6. **实现员第 1b 步「副本里跑的也算」**：包装按文件名认（`.claude/hooks/lib_shell_words.py:51` `WRAPPERS_TAKING_ONE_ARGUMENT = {"capped.sh", "run-with-memory-cap.sh"}`），副本里的那一份也认；账与锁在 `${XDG_RUNTIME_DIR}/singlefs-heavy-admission`（`run-with-memory-cap.sh:107`），各副本共用一份。峰值表从不带 `.git` 的副本里跑时落在副本自己的 `research/scripts/memory-peaks.tsv`（`run-with-memory-cap.sh:227–231`），与主仓那份不通；没写进定义，推的影响是峰值表记不到主仓。

## 六、顺带看到、没改的（不在这一次的写范围里，交主 agent）

1. `.claude/main-agent.md:14`「禁止」一节「禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等」与第 3 条改前是同一张六类清单（缺 87 号、E152 装置，靠「等」收尾）；派发只点了第 20 行，没动。
2. `.claude/agent-common.md:49`「要停自己起的一条链」那一条写 `scripts/proc.py stop <pid>`：`ls scripts/proc.py` 报 No such file，实际在 `.claude/singlefs-ai-sop/scripts/proc.py`（新条 ⑥ 与攻方 3c 写的是全路径）。
3. `.claude/agents/implementation-writer.md:29` 第 4 步末尾「交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交」与第 13 行「在主工作区改」、第 3 步「`crates/mutations.tsv` 在的话，每条再加一行变异」是两种交法（打补丁 / 直接改）；`mutations-append.tsv` 正是打补丁那种交法的草稿表（`research/prompts/m2-*-implementer-report.md` 里十几份都写 `/tmp/claude-1000/<草稿目录>/mutations-append.tsv` 由主 agent 追加进 `crates/mutations.tsv`）。我只照任务把名字改成 `crates/mutations.tsv`；定义到底要哪一种交法，`git apply --check` 那半句要不要跟着改，没动。
4. `.claude/main-agent.md:59`「暂存之后、提交之前跑门禁」一行写的层 0 全量命令 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>` 没有包装；崩溃验证员第 1b 步现在要求 54、55、57 号经包装，两处命令形态不一样（照定义跑的是带包装那一种）。
5. `heavy-test-guard.sh:81` 文件头说 `run-with-memory-cap.sh 4G bash <<EOF` 会被误拒，现查放行（第二节）。
6. 84 号文件头写着「已知会红的一格（不开豁免）」：E142 第十六次跑 `r16-combined.out` 第 683 行 `control_violations_ok=false` 实验页没点名。登记给执行员之后，任何一个执行员照第 6 步跑归属表都会看到这一格红，不是它那一轮的；共用约束「门禁」一节「红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写」罩着。
7. 记录只改了第 30 行那一格：拒绝清单统一（第 2 项）、攻方 3c（第 5 项）、84 号登记（第 33 行 ② 那一半）、`mutations-append.tsv`（第一批报告第六节 #17）这几件记录第四十节里没有对应的「现状」格被改，要不要记、记在哪一行，主 agent 定。

## 七、没做什么

- 没碰 hooks、`.claude/singlefs-ai-sop/`、`crates/`、kb、`CLAUDE.md`；`.claude/main-agent.md` 只改第 20、30 行两处；`.claude/agent-common.md` 只加两条、改「长活可以等」第三行那一句。
- 没跑 72 号（定义还没有判决，必红）、没跑任何重型测试、没编译；hook 只喂 JSON 看退出码，被判的命令一条没执行。
- 第六节七处没改。
- 没核 ⑤⑥⑦ 里我没逐个喂的形态（第二节列了哪些），照 hook 文件头写。
- 没量 8G 默认上限够不够 54、55、57 号，没量嵌套包装会不会排到 252。
- 草稿目录 `/tmp/claude-1000/defs-closeout-draft-2/` 里只有文本：`before/`（改前九份拷贝）、三份探针脚本与 `probe-hooks.log`、`probe-detections.jsonl`（write-guard 与 heavy-test-guard 拒绝时照例写的检出记录，环境变量 `AGENT_HOOK_DETECTIONS` 指到这个文件，没写进全局检出记录）、门禁日志、`batch2.diff`（3c 最后一处改之前生成的，作废）、`batch2-final.diff`、`defs-only.diff` 与这份报告。没建编译目录、没拷仓副本、没起 worktree，没有要删的；后台进程只起过一个（47、62、63、doc-lint、规则纪律串跑那一条），已经结束。
