# sync-local-legs-r1 本地攻方提示：逐句核转述

核对表：英文项 / 原文文件:行 / 首稿缺的 / 定稿。提示只攻甲格（规则文字矛盾与时间算术），每一句译自中文原文（或引自英文工具说明）的句子都在这里对照一遍；英文比原文多出来的限定词、括注单列一行并写明为什么加。

## 1. Fact C（three-way-local-attack.md 第 29 行相关分句）

英文项（定稿）：Run it in the foreground: bash research/scripts/ask-local.sh <prompt file> > <prefix>-output-s<n>.md; do not use setsid, &, or disown. When the single request's maximum, ASK_LOCAL_TIMEOUT (default 900 seconds), exceeds the Bash tool's foreground cap, switch to starting the same command with Bash's run_in_background instead; once started, end this turn and wait for the completion notification, and take the exit code from that notification.

原文文件:行：`.claude/agents/three-way-local-attack.md:29`：「前台跑 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`，不许 `setsid`、`&`、`disown`；单次请求最长 `ASK_LOCAL_TIMEOUT`（默认 900 秒）超过 Bash 前台上限时，改用 Bash 的 `run_in_background` 起同一条命令，起完结束本轮等完成通知，退出码取通知里的。」

首稿缺的：无——首稿与定稿同一版，逐词对得上：「前台跑」→run it in the foreground；「不许 setsid、&、disown」→do not use setsid, &, or disown（三项都列了）；「单次请求最长 ASK_LOCAL_TIMEOUT（默认 900 秒）超过 Bash 前台上限时」→保留原句的歧义，见下「多出来的」一条；「改用...run_in_background 起同一条命令」→switch to starting the same command with run_in_background；「起完结束本轮等完成通知」→once started, end this turn and wait for the completion notification；「退出码取通知里的」→take the exit code from that notification。

多出来的（需要单列并写理由）：
- 提示里加了一条说明句（未进入定稿的引文本身，是紧跟在引文后面的单独一段）：指出「超过 Bash 前台上限时」在原句里有两种读法——比较「900 秒」这个固定数与固定上限（提前就能判定的比较），或比较某一次具体调用实际花的时间与上限（要等那次调用跑完才知道的比较）——原句字面两种读法都成立，提示没有替它选一个。加这条说明是为了不在转述阶段就把原句本身留着的歧义先悄悄消掉，让本地模型自己在任务里面对这处歧义（这正是甲格要打的矛盾点之一）；这条说明不改变引文本身，是引文之外单独加的一段，已在提示里与引文分开写。

## 2. Fact D（three-way-inference.md 第 206 行）

英文项（定稿）：The local leg is started with Bash's run_in_background; end this turn and wait for the completion notification (a single request takes at most 900 seconds, which exceeds the foreground cap); the command must not add setsid, &, or disown.

原文文件:行：`.claude/rules/three-way-inference.md:206`：「本地腿用 Bash 的 `run_in_background` 起、结束本轮等完成通知（单次请求最长 900 秒，超过前台上限），命令里不加 `setsid`、`&`、`disown`。」

首稿缺的：无——逐词对得上：「本地腿用 Bash 的 run_in_background 起」→the local leg is started with Bash's run_in_background；「结束本轮等完成通知」→end this turn and wait for the completion notification；括注「单次请求最长 900 秒，超过前台上限」→(a single request takes at most 900 seconds, which exceeds the foreground cap)；「命令里不加 setsid、&、disown」→the command must not add setsid, &, or disown。

多出来的：提示在引文后加一条说明句，指出这句的「用 run_in_background 起」没有任何「当……时」的条件句，括注是给这条无条件指令的理由、比较的是两个固定数（900 秒与「前台上限」），不是某一次具体调用的实际时长。这条说明同样不改变引文，是为了让模型注意到这句与 Fact C 在「有没有条件句」这一点上的字面差异，帮它把甲格「两句是否相反」这个问题问准，不是我替它下结论。

## 3. Fact E1（agent-common.md 第 57 行开头分句）

英文项（定稿）：Long-lived tasks can be waited on; do not give them a timeout, and do not kill them yourself partway through: long-lived tasks such as compiling, mutation tables, and replays are started with Bash's run_in_background; once started, end this turn, and you will be notified when it finishes.

原文文件:行：`.claude/agent-common.md:57`，开头分句：「长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。」

首稿缺的：无——逐词对得上：「长活可以等」→long-lived tasks can be waited on；「不给它设超时」→do not give them a timeout；「不自己中途杀掉」→do not kill them yourself partway through；「编译、变异表、复跑这类长活」→long-lived tasks such as compiling, mutation tables, and replays（「这类」译成 such as，保留「举例、不封闭」的原意，没有译成 namely 或 only）；「用 Bash 的 run_in_background 起」→are started with Bash's run_in_background；「起完结束本轮」→once started, end this turn；「完成时会通知你」→you will be notified when it finishes。

多出来的：提示在引文后加一条说明句，点出「编译、变异表、复跑」三个例子前面的字是「这类」（举例），不是「只有」；调 ask-local.sh 不在这三个点名的例子里。这条说明没有改变引文，只是把原句里已经含着的「这类＝举例不封闭」这个语义标出来，帮模型判断 Fact E1 到底管不管「本地腿调 ask-local.sh」这件事，不是新造的限定。

## 4. Fact E2（agent-common.md 第 57 行中段分句）

英文项（定稿）：A command handed to it must keep running all the way until the real work is actually finished before it exits: do not, inside that command, put the work into the background a second time -- do not use disown, coproc, setsid -f, nohup ... &, the detached mode of tmux or screen, or a systemd-run that does not wait until it ends; do not write a bare & inside a subshell or inside $( ... ) either; & is only used when the same command is immediately followed by a plain wait with no arguments to join on it, with each job's exit code collected the way a separately numbered item elsewhere in the same file specifies.

原文文件:行：`.claude/agent-common.md:57`，中段分句：「交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。」

首稿缺的：首稿把「systemd-run」直接译成「systemd-run」，没有把原文别处「不等到结束的 systemd-run」这一限定（见共用约束「执行前拒绝的写法」③ 一条：「不等到结束的 systemd-run」）带进来，容易让人以为这句是全面禁用 systemd-run。定稿已加限定「a systemd-run that does not wait until it ends」，把禁的范围收回到「不等到结束」这一种用法，与共用约束③条一致。

多出来的：末句「每件的退出码照……那一条④的写法收」译成「each job's exit code collected the way a separately numbered item elsewhere in the same file specifies」，把原文点名的「「执行前拒绝的写法」那一条④」改写成「a separately numbered item elsewhere in the same file」——这是遵照提示自己的规则（答复与提示都不写文件行号，指路用条款名或表格行号），原句本身点名的是共用约束内部一个条款序号，不是外部文件的行号，但为了和提示「不写代码行号与文件行号」的自我要求保持一致的风格，转述时把具体序号也换成了泛指。这处改写不影响 Fact E2 在甲格任务里的用途（甲格只用到 E1、E3，E2 只作背景陈列），故未逐字保留序号。

## 5. Fact E3（agent-common.md 第 57 行后段分句，前台超时上限）

英文项（定稿）：Ending this turn means this reply writes only a single sentence saying what it is waiting for, and calls no more tools; a foreground command's timeout must not exceed 240000 milliseconds.

原文文件:行：`.claude/agent-common.md:57`，后段分句：「结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。」

首稿缺的：无——逐词对得上：「结束本轮就是这一条回复只写一句在等什么、不再调工具」→ending this turn means this reply writes only a single sentence saying what it is waiting for, and calls no more tools；「前台命令的 timeout 不超过 240000 毫秒」→a foreground command's timeout must not exceed 240000 milliseconds。

多出来的：无。

## 6. Fact E4（agent-common.md 第 57 行收尾分句）

英文项（定稿）：Waiting for a background task you started yourself must always be done by ending this turn and waiting for the completion notification; do not write a loop that polls its output file.

原文文件:行：`.claude/agent-common.md:57`，收尾分句：「等自己起的后台任务一律结束本轮等完成通知，不写轮询它输出文件的循环（run_in_background 里也拒，见「执行前拒绝的写法」②）。」

首稿缺的：首稿漏译了括注「（run_in_background 里也拒，见「执行前拒绝的写法」②）」，只译了主句。定稿补上了「不写轮询它输出文件的循环」这半句主句内容（已在定稿英文里），但括注本身（指向共用约束②条的钩子会拒）判为「不抄」处理：这半句只是给同一条禁令追加一个「钩子会拦」的执行后果，不改变这条规则本身要求什么，且甲格任务不涉及钩子层（钩子归云端攻方乙格），故未译入 Fact E4，在这里记录「为什么不抄」。

## 7. Fact B（Bash 工具说明，英文原文，非中文转述）

英文项（定稿）：You may specify an optional timeout in milliseconds (up to 600000ms / 10 minutes).

来源：Bash 工具自身的 description 字段（不在仓里），本会话工具列表原文逐字如此，不是从中文转述来的，故不适用「逐句核转述」这一步；这里列出是为了核对提示里 Fact B 的引文与工具说明原文逐字一致。

核对结果：一致，一字不差。提示里 Fact B 的「meaning」说明句（「a foreground Bash call's timeout parameter cannot be set above 600000 milliseconds...不假设有默认值」）是提示自己的推断说明，不是转述，已在提示正文里与引文分开写、且明确写了「不要假设默认值」，避免把主 agent 派发消息之外、这份工具说明原文没有写的「120000 毫秒默认值」当成核实过的事实带进去（这份工具说明原文在本次核查中没有找到这一句，是否存在没有核实，因此没有写进提示）。
