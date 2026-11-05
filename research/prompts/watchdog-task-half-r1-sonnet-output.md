# watchdog-task-half-r1 正推腿（K1）报告

轮名：watchdog-task-half-r1（第一轮）。立场：正推。只判 K1 的两问，K2、K3 不是这条腿的格。
被判对象：`research/prompts/watchdog-task-half-r1-snapshot/batch-49.patch` 与 `batch-49-readme.patch` 打进主工作区之后的样子。

## 0 开工核对

- 五份被判文件此刻的 sha256 与开工快照 `research/prompts/watchdog-task-half-r1-snapshot/judged-sha256.txt` 逐行相同（`sha256sum` 五份与快照比，五行全对；`agent-watch.py` 为 `ee462e58…feae95`，`watch.sh` 为 `ffdcc32d…425136`，`watch.conf` 为 `1fb36f8e…ab0`，`main-agent.md` 为 `3ee752df…5905866`，`README.md` 为 `060f9403…f31`）。没有开工后又被改过的件。
- 语法：`python3 -I -c "import ast;ast.parse(open('scripts/agents/agent-watch.py').read())"` 打 `ast ok`；`bash -n scripts/agents/watch.sh` 退 0。
- 补丁对 `.claude/main-agent.md` 的实际改动，我用补丁里「-」「+」行按句号、分号切开逐句比（脚本在会话里跑，没留文件）。结果：只有三个自然段动了，合起来是 7 处——
  ① 第 63 行去掉「与『任务服务不通』」；
  ② 第 67 行「它只管 agent、任务服务的健康与本会话任务正不正常：…起 control check、读 task status」改成「它只管 agent：复检子 agent 的会话记录、读 hook 的检出记录」；
  ③ 同段告警清单去掉「任务服务不通」「本会话任务异常（…阈值是 watch.conf 的三个键）」；
  ④ 「看门狗只读服务给的状态、不停任务」改成「看门狗不读服务、不停任务」；
  ⑤ 删「看门狗报任务异常时，主 agent 在下一条回复里告诉用户…」；
  ⑥ 「卡住…、『任务服务不通』与本会话任务异常才处置：…」改成「卡住（…）才处置」；
  ⑦ 第 73 行「--ack 的写法」四种改成三种、去掉 `任务:<号>:<告警名>`、示例由 `类:任务服务不通` 换成 `类:交回之后后台还在跑`。
  其余句子补丁没碰；按派发提示，没碰的句子只在它对着脚本说了「看门狗做什么」时才判（标「未改动」）。
- 实现今天的样子（主 agent 观测的复核）：`crates/` 不涉及；`git apply --numstat` 未跑（它只列文件，不影响本腿）。

## 1 K1 第一问：改后的定义逐句对脚本

判定档：一致 / 不一致 / 说不清。引文先在被引文件里 `grep -nF` 过，行号即命中行。

### 1.1 第 63 行（开头一段）

**S1**「看门狗报出来的卡住，主 agent 在它下一次回报之前处置或 `--ack`」（`.claude/main-agent.md:63`）
- 判：一致。
- 依据：看门狗的回报就是退出：`agent-watch.py:27`「退出码：0 被看的子 agent 全部交回或被停、没有告警；3 有告警；4 定时回报」。卡住类告警在 `agent_alerts()` 里生成：`agent-watch.py:1565`（`alerts.append(("工具调用过长"`）、`:1582`（`alerts.append(("无动静"`）、`:1575`（`alerts.append(("结束本轮却不会醒"`）。`--ack` 由 `split_acknowledged()` 认（`agent-watch.py:2108`）。
- 没有残留已删的东西：本句不再带「任务服务不通」。
- 什么现象会推翻：看门狗对「工具调用过长」等卡住类告警不认 `--ack`（自证 `AGENT_WATCH_BREAK=ack` 红的那几格就是守这一点，`agent-watch.py:132-138`）。

**S2**「看门狗只报、不停子 agent 也不停进程」（`.claude/main-agent.md:63`，未改动）
- 判：一致。`agent-watch.py:25`「它看、它报，不停子 agent、不停进程」。我 `grep -n 'os\.kill\|\.terminate(\|\.kill(\|killpg'` 全文件：命中的 `:2652、:2742、:2840、:3262、:3585、:3724、:3737、:3743` 都在 `--selftest` 区（造假进程用），业务路径没有。`PROCESS_STOP_COMMAND`（`:466`）只是拼进告警说明里的出路命令（`:1069`），看门狗自己不执行。
- 推翻：业务路径里出现对被盯进程的 kill / terminate。

**S3**「hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的在执行前拒绝…」（`.claude/main-agent.md:63`，未改动）
- 判：说的是钩子，不是看门狗；与脚本只在「检出记录」这一点相交：`agent-watch.py:49-50`「hook 的检出也在这里汇给主 agent」。本腿不判钩子本身。

### 1.2 第 67 行（看门狗一段，被改动最多）

**S4**「看门狗一个会话一只：会话里第一次派发之后，用 Bash 的 `run_in_background` 起 `bash scripts/agents/watch.sh`（不带 agent 号：会话目录它自己找，之后新派的它自己接上；阈值在 `scripts/agents/watch.conf` 里配，不在命令行上手敲；调用里不加 `&`、`nohup`、`disown`）」（`.claude/main-agent.md:67`，未改动）
- 判：一致。
  - 「一个会话一只」：`watch.sh:8`「一个会话一个看门狗：从 CLAUDE_CODE_SESSION_ID 找会话目录，每次检查自己找还没结束的子 agent」与「已有一个在盯就退 5」；实现是 `acquire_single_instance()`（`agent-watch.py:2133`），`:2473` 退 5。
  - 「不带 agent 号…新派的自己接上」：`watch.sh:8` 同句；转成 `watch --discover --session-dir …`（`watch.sh:256` 的 `agents)` 分支，`:263` `run_or_print python3 "$watchdog" watch --discover`）。
  - 「阈值在 watch.conf 里配，不在命令行上手敲」：`watch.sh:239` 对任何别的 `-*` 选项报「不认识的选项」，出路写「阈值写进 $conf，不在命令行上给」。`KNOWN_CONFIG_KEYS`（`watch.sh:66`）里没有任何 `task-` 键，所以 `watch.conf` 若再写 `task-queue-minutes` 会被 `config_options()` 拒（`watch.sh:78` 那条 `agent-watch.py 不认识` 的 die）——删得干净。
  - 「调用里不加 &、nohup、disown」：`watch.sh:7`「一律放进 Bash 工具的 run_in_background；这里不再自己放后台」。这一条的拒绝在 hook 一侧（`agent-common.md:81` ①），脚本只给了旁证。
- 推翻：`watch.sh` 对不带 agent 号的调用不走 `--discover`，或同会话第二只不退 5。

**S5**「之后的派发不重起它；它退出了（按告警退出、定时回报、全部交回）才再起一只，起之前先读它的输出」（`.claude/main-agent.md:67`，未改动）
- 判：一致。三种退出对得上退出码 3（告警）、4（定时回报）、0（全部交回），`agent-watch.py:27`。

**S6**「它只管 agent：复检子 agent 的会话记录、读 hook 的检出记录，告警有：…」（`.claude/main-agent.md:67`，本句整体被改）
- 总判：说「复检会话记录、读 hook 检出」一致；说「只管 agent」并列出九项告警，**部分一致、有一处说不清的缺项**（见 S6-缺项）。脚本里没有任何一行再起 `control check`、读 `task status` 或碰 `task-server/`：`grep -nE 'task-server|control check|task status|task_server' agent-watch.py watch.sh` 只命中说明文字（`agent-watch.py:4、:10、:12、:26、:64、:70、:74`，全是「归 task-server」或「删了」的叙述）与 `TASK_SERVER_SOURCE_REPOSITORY` 这个环境变量名（`watch.sh:35、:54`、`agent-watch.py:64、:387、:390`，只是取来源仓路径，不读服务）。`agent-watch.py` 里的 `subprocess` 调用全在自证区（`:2534` 起）。
- 逐项（定义里列的每一项 ↔ 脚本里的产出点与默认阈值）：

| 定义里的项 | 脚本产出点 | 阈值 | 判 |
|---|---|---|---|
| 没超时的等待循环 | `agent-watch.py:1562` `alerts.append(("等待循环"` | `--wait-loop-minutes` 默认 3（`:3838`） | 一致；但脚本把它归「只记不叫醒」（`INFO_ALERT_NAMES`，`:513`），见 S6-只记 |
| 一次工具调用超过 8 分钟 | `:1565` `alerts.append(("工具调用过长"` | `--tool-minutes` 默认 8（`:3837`） | 一致 |
| 同一命令反复且输出不变 | `:1619` `alerts.append(("同一命令反复且输出不变"` | `--repeat-count` 默认 3（`:3840`） | 一致；同样在 `INFO_ALERT_NAMES`（`:513`） |
| 按模式找进程 | `:1629` `alerts.append(("禁用命令"`，模式 `BANNED_COMMAND_PATTERN`（`:398`，`pgrep -f` / `pkill -f` / `killall`） | 窗口 `BANNED_COMMAND_WINDOW_SECONDS` = 600 秒（`:401`） | 一致 |
| 10 分钟无动静 | `:1582` `alerts.append(("无动静"` | `--idle-minutes` 默认 10（`:3839`） | 一致 |
| 结束本轮而手里没有在跑的后台任务 | `:1569`（`STOPPED_MIDWAY_ALERT`，一个后台任务都没起过）与 `:1575`（`"结束本轮却不会醒"`，起过的都结束了） | 60 秒（`:42`） | 一致（一句话盖住了脚本里的两个告警名） |
| hook 检出本会话的问题命令（执行前已拒掉的、钩子判不了而照常放行的只记进报告、不叫醒） | `:2019` `alerts.append((subject, "hook 检出",`；被拒的 `:2014` 的 `notes.append` 不叫醒；判不了的 `UNJUDGED_DETECTION_PREFIXES` 同样只进报告 | `--detection-poll-seconds` 15（`:3867`） | 一致 |
| 交回之后后台还在跑（被停的同样算） | `:1067` `alerts.append((agent_id, LEFTOVER_BACKGROUND_ALERT,`；「被停」`:36-37、:77-79` | `--leftover-background-grace-minutes` 默认 2 | 一致 |
| 交回之后副本还在（交回报告写了「没删 <全路径>」而路径还在，主 agent 删） | `:1159` `alerts.append((agent_id, KEPT_COPY_ALERT,`；只报不删 `:104` | `--kept-copy-hours` 默认 6，`watch.conf:30` 也写 6 | 一致 |

- **S6-只记**：`agent-watch.py:513`「INFO_ALERT_NAMES = {"等待循环", "同一命令反复且输出不变", PROGRESS_STALE_ALERT}」；`split_informational()`（`:2086`）把它们摘成「只记不叫醒」的报告行。定义的「告警有」把这两项和会叫醒的并排，没说它们不叫醒。这是补丁前就有的写法（补丁只改了句首与句尾），**不是批 49 带来的偏差**，列出供主 agent 定要不要顺手改；本腿判「说不清」（词「告警」宽，不算错）。
- **S6-缺项（说没说全）**：脚本还会产出这几样，定义这一句没列：
  1. 「根分区余量过低」：`agent-watch.py:1924`（`alerts.extend((ROOT_FREE_SUBJECT, …`），叫醒主 agent（`:113-114`），`watch.conf:23` 与 `:32`（`root-free-gib-alert=128`）配着。这是主机级告警，不属于「只管 agent」。
  2. 「没启动」（`:1873`，`--not-started-minutes` 默认 5，派发后还没会话记录）。
  3. 「进度文件不涨」（`:1603`，`INFO_ALERT_NAMES` 里，只记）。
  4. 「该交接」（`:1594`）与「跑满 N 小时」（`:1608`）：`watch.conf:29`、`:28` 把它们写成 0 关掉，不会出。
  这四处**补丁前就缺**（补丁前的句子同样没列），补丁把句首从「只管 agent、任务服务的健康与本会话任务正不正常」收成「只管 agent」后，「根分区余量过低」与「只管 agent」更显出不合：它是 `watch.conf:23`、`agent-watch.py:113` 白纸黑字写着「叫醒主 agent」的现役告警，而定义没有一处提到它。判：**说不清（不是批 49 引入，但批 49 把这一句改成了更窄的口径）**。
- 没有一项还在说已删的告警：列表里没有「任务服务不通」「本会话任务异常」、没有 `task-queue-minutes` 等三键。
- 什么现象会推翻「一致」：`agent_alerts()` 再出现某个不在这九项里、会叫醒主 agent 的新告警名而定义没跟着改。

**S7**「任务卡住由服务判…经 `task wait` 叫醒，看门狗不读服务、不停任务」（`.claude/main-agent.md:67`，本句被改；引文中间的省略号接的是同一行里的原文，整句见该行）
- 判：一致。
  - `stalled` 一行存在：`task-server/task_server_lib.py:119`「stalled                 233    卡住：起来后先宽限 STALL_GRACE_SECONDS（10 分钟）不判」。
  - 经 `task wait` 叫醒：`task-server/task:16`「agent 用 run_in_background 起它，任务完成时它退出、harness 叫醒 agent」；`stalled` 的退出码 233（`task_server_lib.py:1155`）。
  - 「看门狗不读服务」：前述 grep 无命中；`agent-watch.py:4`「这里不读服务、不判任务、不停任务」与实现一致。
  - 小提醒（不判错）：服务另有「疑似卡住」一路——CPU 满窗口几乎不涨，服务不停，`wait` 打 `TASK_SERVER_SUSPECT` 退 231（`task-server/task:18`、`:297` `return lib.SUSPECT_EXIT`）。这一路也经 `wait` 叫醒，但**不是「服务判、服务停」**，而是「通知、任务照跑」。定义只点了 `stalled` 一行，没提 231；说得不全但不错。
- 推翻：`agent-watch.py` 里出现对 `task-server/` 的读（读 `alerts.jsonl`、起 `control`、读 `task status`）。

**S8**「读到告警先分两类…只记进这一轮的调度记录，不处置；卡住（工具调用过长、无动静、结束本轮不会醒）才处置」（`.claude/main-agent.md:67`，本句只去掉了后半；引文中间的省略号接的是同一行里的原文，整句见该行）
- 判：一致。前半对 `agent-watch.py:52-58`（被拒的与判不了的「只写进报告、不叫醒」），后半的三项就是 S6 表里叫醒的前三类。去掉的是「『任务服务不通』…照告警说明里的末行办」那一整截，脚本里没有对应的告警了，所以没有悬空的「末行」。
- 说不清的一处（补丁前就有）：「卡住」括号只列三项，叫醒的「禁用命令」「hook 检出（只记）」「交回之后后台还在跑」「交回之后副本还在」「根分区余量过低」不在「先分两类」之内，也不在「才处置」之内；主 agent 读到它们该怎么办，这一句没说。非批 49 引入。

**S9**「坏卡不由看门狗报：交的任务结局是 `card-fault`（`task wait` 末行 `type=card-fault`、退 237）时，当场弹窗告诉用户哪台哪张卡与依据（任务日志里的清场行）、请用户重置那张卡，不等、不重试」（`.claude/main-agent.md:67`，未改动，补丁前后一字不差）
- 判：一致。
  - 「看门狗不报」：`grep -n card scripts/agents/agent-watch.py` 零命中（没有任何 card-fault 的字样）；告警清单（S6 表）没有它。
  - 「退 237、末行 type=card-fault」：`task-server/task_server_lib.py:106`「card-fault              237    清场判出卡坏」与 `:1148`（`"card-fault": (237, "清场判出卡坏")`）；末行格式 `task-server/task:10`「TASK_SERVER_RESULT task=<号> type=<错误类型> exit=<码>」。
  - 「任务日志里的清场行」：`task_server_lib.py:986`「CARD_FAULT_MARK = "GPU_QUIESCE step=quiesce result=card-fault"」。
  - 「当场弹窗…请用户重置那张卡，不等、不重试」：`task_server_lib.py:106` 同行后半「出路是主 agent 弹窗请用户重置那张卡」。
- 注意这一句**批 49 之后成了定义里「任务异常」的唯一一条点名处置**（删掉了「本会话任务异常…告诉用户」）。它仍然成立，但见第二问。

**S10**「它的报告另有两样：有子 agent 交回时（被停的同样算）列草稿根里超阈值（`watch.conf` 的 `scratch-idle-hours`）没动的编译目录与仓副本，只报不删，主 agent 核过没人在用再删；重启前旧进程留下的子 agent 记录标…，不按无动静报」（`.claude/main-agent.md:67`，未改动；引文中间的省略号接的是同一行里的原文，整句见该行）
- 判：两样本身一致；「另有两样」这个数说不清。
  - 草稿根那一节：`agent-watch.py:107-111`；产出点 `:1935`（`if any_handed_back:` 才查）；键 `scratch-idle-hours` 在 `watch.conf:31`（值 6）与 `watch.sh:66` 的键表里；「只报不删」`:110`。
  - 「状态=已失效」：`agent-watch.py:123-126`，判法函数 `stale_session_reason()`（`:1194`）。
  - 数：补丁删掉的是脚本报告里「只进报告的两样」（`control check` 末行、没 clean 的编译目录），所以「另有两样」这个数恰好是补丁前后都没改的句子；但报告里另有**范围声明一节**（`:59-61`）、**中途停下没交回的子 agent 一节**（`:46`）、**根分区可用量一行**（`:114`），「两样」不是穷举。非批 49 引入，且「另有」后面是否穷举读法两可，判：说不清，不判错。

**S11**「不发整点例行询问（`watch.conf` 的 `ask-every-minutes=0`）；要看一个子 agent 在做什么，读它草稿目录的 `progress.md` 或它的会话记录，不发消息叫醒它」（`.claude/main-agent.md:67`，未改动）
- 判：前半一致：`watch.conf:28` 就是 `ask-every-minutes=0`，`routine_inquiry_alert()` 在 `ask_every_minutes <= 0` 时返回 None（`agent-watch.py:1517` 起，`:1519` 那条判）；脚本默认值是 60（`:3862`），所以没有 `watch.conf` 这一行就会出整点询问——定义说「靠 watch.conf 的 ask-every-minutes=0」是对的。后半（读 progress.md）是主 agent 行为，不对脚本。

## 2 第 73 行与 `--ack`、阈值

**S12**「键照 `scripts/agents/agent-watch.py` 文件头…三种：`进程:<pid>`（…报的那一个进程）、`<agent 号>:<告警名>`、`类:<告警名>`（这一类的全部，范围最大，例 `类:交回之后后台还在跑`）」（`.claude/main-agent.md:73`，本句被改；引文中间的省略号接的是同一行里的原文，整句见该行）
- 判：与脚本文件头一致；与 `watch.sh` 一致；**但有一个旁证不一致**（见 S12-本机）。
  - 文件头：`agent-watch.py:21`「键三种：进程:<pid>、<agent 号>:<告警名>、类:<告警名>」；`:94` 起的「--ack 的写法（可给多次；watch.sh 也认逗号连写」下面三行 `:95-97` 就是三种；示例 `:97`「类:交回之后后台还在跑」。
  - `watch.sh` 头：`watch.sh:18` 写「--ack 的写法一共三种」，`watch.sh:19-21` 逐项列。
  - `watch.sh` 实际认的头：`watch.sh:209` `ack_head='^(a[0-9a-f]{8,}|进程|类):.+$'`——恰好三种头，`任务` 头没有了；对应 die 在 `watch.sh:221`（「不是 --ack 认的形态」）。逗号连写拆分用同一个正则（`watch.sh:229` 的 `ack_piece =~ $ack_head`）。
  - 文件头的「例 `类:交回之后后台还在跑`」有格守：`watch.sh` 自证⑪的命令行用的就是它（补丁 `batch-49.patch` 里 `watch.sh` 的 ⑪ 行）。
  - `agent-watch.py` 一侧：`ack_scope_keys()`（`:2101`，`{f"{ACK_CLASS_PREFIX}{alert[1]}"}`）认 `类:`；`LEFTOVER_BACKGROUND_ALERT` 认 `进程:`（`split_acknowledged()` 内）；`{subject}:{name}` 认 `<agent 号>:`。**脚本没有任何一处认 `任务:`**：`grep -nE '任务:' scripts/agents/agent-watch.py scripts/agents/watch.sh scripts/agents/watch.conf scripts/agents/README.md .claude/main-agent.md` 零命中。
- **S12-本机（旁证，非批 49 引入，主 agent 请定）**：`agent-watch.py:118`「条件类告警：--ack 本机:根分区余量过低 确认之后只进报告」与 `watch.conf:24` 都推荐写 `--ack 本机:根分区余量过低`，而 `watch.sh:209` 的头只认 `a<十六进制>`、`进程`、`类`，`本机:…` 过不了 `[[ "$1" =~ $ack_head ]]`，会落到 `watch.sh:221` 的 die。（`watch.sh --dry-run --ack 本机:…` 我没跑：`heavy-test-guard` 把「起 watch.sh」按项目脚本拒了，命令见第 4 节。）这个口子补丁前就在（补丁前的 `ack_head` 同样只有 `a…|进程|类|任务` 四头），批 49 没改变它；`类:根分区余量过低` 是能走的写法（自证⑪用的就是它，`watch.sh` 补丁 ⑪ 行）。定义写的是「三种」，没提 `本机:`，与 `watch.sh` 反而一致。
- 推翻「一致」：`watch.sh` 的 `ack_head` 或 `agent-watch.py` 文件头的 `--ack` 清单与定义的三种不同。

**S13**「主 agent 自己交的任务（长的同样）照共用约束…那一条：后台起 `task-server/task wait <号>`、结束本轮等叫醒；任务卡住由服务判、经 `task wait` 叫醒，要看排队与日志用 `task-server/task status <号>`、`task-server/task log <号>`，要停用 `task-server/task cancel <号>`；`cancel`、`clean` 只对自己交的号做（服务只认会话）」（`.claude/main-agent.md:73`，未改动；引文中间的省略号接的是同一行里的原文，整句见该行）
- 判：一致（对 `task-server/`，不是对看门狗）。`task-server/task:16` 同上；`task status` 给「在等：…」`.claude/agent-common.md:76`；`cancel`、`clean` 只许提交它的会话 `task-server/task:24`、`:26`。

## 3 逐处找残留（是否还有一句在说已删的告警或阈值）

我对五份文件与定义做了两遍全文 grep（`grep -nE '任务服务不通|task-queue|task-overrun|task-retry|任务:|排队过久|跑得远超预计|远超预计|反复重试|异常结局|服务告警|clean|control check|task status|alerts'`，输出贴在会话里）：
- `scripts/agents/watch.conf`：无命中旧告警名、旧阈值键（`task-queue-minutes`、`task-overrun-factor`、`task-retry-count` 都没了）。
- `scripts/agents/watch.sh`：无命中；`ack_head`、`die` 文案、头注释都改成三种。
- `scripts/agents/agent-watch.py`：只命中 `:4`、`:10`、`:12` 的历史叙述（说「这一半删了」，并非继续告警），以及 `agent_alerts` 等函数名里的 `alerts`。**没有**旧告警名、旧键形、旧阈值。
- `scripts/agents/README.md:9`：补丁后「看门狗：复检子 agent 的会话记录与 hook 的检出记录；`--processes` 只等本实例底下起的长命令退出。任务卡不卡、停不停、用多少内存与核数、异常结局都归 `task-server`（经 `task wait` 叫醒），看门狗不读服务；阈值在 `watch.conf`」——与脚本一致，见 S6、S7。
- `.claude/main-agent.md`：命中 `:33`（`task clean <号>`，讲编译目录用完就删，不是看门狗）、`:73`（`task status`，S13）、`:79`（`cancel` 或 `clean`，讲交回处置）三处，**都不是已删告警或阈值**。该文件里不再有「任务服务不通」「control check」「task-queue」等字样。
- 结论：**没有一句还在说已删的告警或阈值；也没有一句把留下的那一半（盯子 agent）也说没了**——留下的几样（等待循环…副本还在、`scratch-idle-hours`、已失效、`ask-every-minutes=0`、`--ack` 三种）在定义里都还在、都对得上。

## 4 K1 第二问：删掉之后主 agent 靠什么知道「本会话任务异常」与「任务服务不通」

问法：改后的定义有没有写到 `task wait` 叫醒、`control check`；这几样在 `task-server/` 里是不是真有。

### 4.1 `task wait` 叫醒

- 改后的定义写到了，两处：`.claude/main-agent.md:67`「任务卡住由服务判…经 `task wait` 叫醒，看门狗不读服务、不停任务」，与 `:73`「后台起 `task-server/task wait <号>`、结束本轮等叫醒；任务卡住由服务判、经 `task wait` 叫醒」。`.claude/agent-common.md:76-78` 补了 `wait` 的细节（起法、不设超时、退 78 的处置）。
- `task-server/` 里真有，逐条：
  - 任务完成它退出、harness 叫醒：`task-server/task:16`「agent 用 run_in_background 起它，任务完成时它退出、harness 叫醒 agent」；末行 `TASK_SERVER_RESULT task=<号> type=<错误类型> exit=<码> …` 在 `task-server/task:10`。
  - 非 succeeded 的结局都带 `type=` 和退出码：错误类型表 `task_server_lib.py:89` 起；`card-fault` 237（`:106`、`:1148`）、`stalled` 233（`:119`、`:1155`）。**这就是原来看门狗的「异常结局」一类**：现在由 `wait` 的末行与退出码交出来，`wait` 一退出主 agent 就被叫醒。
  - 疑似卡住（CPU 满窗口不涨、服务不停任务）：`wait` 打 `TASK_SERVER_SUSPECT` 行、退 231，`task-server/task:18`、`:292-297`（`if not reply.get("finished"):` … `return lib.SUSPECT_EXIT`）。
  - 服务连不上：`wait` 先按 `RECONNECT` 间隔重连，连续连不上超过 `STARTUP_RETRY_SECONDS = 60.0`（`task-server/task:55`）就 `report_unavailable_and_exit`（`task-server/task:276-278`）退 78（`task_server_lib.py:960`、`:1343-1346`），stderr 点名 `systemctl --user status`、起法与装法（`task_server_lib.py:2328-2331`）。`.claude/agent-common.md:78`「`task wait` 退 78（连不上服务）：先 `task-server/task status <号>`，任务在排或在跑就再起 `task wait <号>`，不重交」是主 agent 的处置。
- 判：**写到了，且实现真有**。

### 4.2 `control check`

- 改后的定义**没有写到**：`grep -rn 'control check' .claude/main-agent.md .claude/agent-common.md .claude/agents .claude/rules .claude/kb/tooling.md` 零命中；`.claude/main-agent.md` 全文也无「服务不通」「`/health`」「`control watch`」字样（`grep -n '服务不通\|退 78\|/health\|control watch\|连不上\|task status\|在等'` 只命中 `main-agent.md:73` 的 `task status` 与 `agent-common.md:64-78` 的 `wait`、`status` 用法）。补丁删掉的正是原句里的「修好后 `python3 task-server/control check` 回读到退 0」。
  - 主 agent 现在**只有**这条文字路径知道服务不通：`agent-common.md:78` 的 `wait` 退 78，与 `.claude/kb/tooling.md:44`「`wait` 退 78 是连不上服务，先 `task-server/task status <号>` 看任务还在不在，不重交」。两处都只管「我正等着某条 `wait` 时」的情形。
- `task-server/` 里真有：
  - `task-server/control:13`「python3 task-server/control check               fastapi、uvicorn 在不在，多机配置在不在，服务通不通（/health 回 ok），NFS 共享四项…逐样判：都过退 0；缺一样退 78」；实现 `task-server/control:176`（`def check():`）。
  - `/health` 回 ok 才算通：`task_server_lib.py:2334`（`def require_server`）与 `:2340`（`if status != 200 or not health.get("ok"):` 抛 `Unavailable`，出路写「修好之后 python3 task-server/control check 回读到退 0」）。所以 `control check` 也会判出库写不进与排头放得下却不起：`/health` 的 `problem` 见 `task_server.py:15` 「problem 只报库写不进与排头放得下却连着三轮没起」，轮数见 `task_server.py:297` 「STARVED_ROUNDS = 3」。
  - 服务端自己也把出路指回 `control check`：`task-server/control:173`、`:185`、`:347`。
- 判：**定义没写、实现真有**。这是 K1 第二问的第一条发现：改后的定义已不指向 `control check`，而服务侧的错误出路仍在指它。

### 4.3 与「任务服务不通」直接相关、定义也没写的另一样：`control watch`

- 实现真有：`task-server/control:25`「python3 task-server/control watch               每 WATCH_SECONDS 秒查一次：服务不通、库写不进…」，同段末句（`:28`）写明「主 agent 用 run_in_background 起它盯着，退出即处置」；实现 `task-server/control:311`（`def watch():`）；`task-server/README.md:67` 也点名它。
- 定义没写：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/kb/tooling.md` 对 `control watch` 零命中。
- 这是原「任务服务不通」告警最直接的接替者：原来看门狗每次回报起一次 `control check`（`batch-49.patch` 删的那一半），现在只有「主 agent 自己起 `control watch`」或「正好有一条 `wait` 挂着」才会被叫醒。定义没把这条路交给主 agent。**这是否算漏洞（主 agent 没挂 `wait` 时服务掉了谁叫醒）是 K2 的格，本腿不判**，只把事实递过去。

### 4.4 「本会话任务异常」的五种，改后各靠什么（只列实现事实，不替 K2 判漏）

| 原看门狗报的 | 改后主 agent 知道的路 | 实现证据 | 定义写没写 |
|---|---|---|---|
| 异常结局 | `wait` 末行 `type=`、退出码，`wait` 退出即叫醒 | `task-server/task:10`、`:16`；`task_server_lib.py:89` 起错误类型表 | 写了：`main-agent.md:73` + `agent-common.md:76` |
| 跑得远超预计 | 只有「跑超记录用时 5 倍而日志满 10 分钟没涨」服务停并打回（`wait` 叫醒，`stalled` 233）；日志在涨的超预计，`wait` 不叫醒，要主 agent 自己看 `task status` | `task_server_lib.py:119`；原看门狗的历史说明 `agent-watch.py:10`「远超记录用时、核数过少只写进服务的 `task status`」 | 写了 `stalled` 一行与 `task status`，没提「日志在涨的超预计不会叫醒」 |
| 反复重试 | 重试在服务里自动做；`wait` 逐次尝试各打一行，但只在结局时一起出（`task-server/task:16`「逐次尝试一行」） | 中途不叫醒 | 没写 |
| 排队过久 | `task status <号>` 的「在等：…」（`task-server/task:343`、`:369`）；`/health` 判「排头放得下却连着三轮没起」→ `control check`/`control watch` 报；`wait` 不因排队叫醒（`task:292` 的 `continue` 只在 `finished` 为假且无 suspect 时继续等） | 同左 | 写了 `task status`；没写 `control check`/`control watch` |
| 服务告警 | `alerts` 表与 `alerts.jsonl`（`task_server.py:1740-1746`，`alert()` 同时 `say` 到服务自己的输出）；`task-server/task`、`task-server/control` 里**没有读 `alerts` 的子命令**（`grep -n alert task-server/task task-server/control` 只命中 `task_server_lib.py:20` 一类文字）。`card-fault`、`record-tampered`（任务以该类型收尾）、`suspected-stall` 另经 `wait` 出；`peer-unavailable` 经 `/health` 的 `peer` 栏与 `control watch`；其余（`cleanup-pending`、`build-directory-left`、`settle-failed`、`start-uncertain`、`shard-advance-failed`、`item-rates-unrecorded`）只有 `alerts` | `task_server.py:1751、:1920、:2164、:2305、:2340、:2361、:2524、:2610、:2808、:4328` 的 `self.alert("…` 各点 | 只写了 `card-fault`（`main-agent.md:67`）；其余没写 |

- 上表第三、四、五行是「改后靠谁」的事实，不是我判漏。K2（攻方）要的复现情形我没给。
- 什么现象会推翻第二问的结论：在 `.claude/main-agent.md` 或 `.claude/agent-common.md` 里找到对 `control check` / `control watch` 的指引（我 grep 零命中），或在 `task-server/` 里找到读 `alerts` 并叫醒主 agent 的路径（我 grep 零命中）。

## 5 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 / S1 `main-agent.md:63` 卡住处置或 `--ack` | 一致 | 卡住类告警与 `--ack` 都在 `agent-watch.py:1565/1575/1582`、`:2108` |
| K1-1 / S2 `:63` 只报不停 | 一致 | `agent-watch.py:25`；kill/terminate 只在自证区 |
| K1-1 / S3 `:63` hook 不停任务 | 不判（钩子侧） | 只与 `agent-watch.py:49-50` 相交 |
| K1-1 / S4 `:67` 起法、阈值在 watch.conf | 一致 | `watch.sh:8、:239、:256-263`；键表 `watch.sh:66` 无 `task-*` 键 |
| K1-1 / S5 `:67` 退出再起 | 一致 | 退出码 `agent-watch.py:27` |
| K1-1 / S6 `:67` 只管 agent、九项告警 | 一致（九项各自）；整句口径说不清 | 缺「根分区余量过低」「没启动」等；等待循环与同一命令反复实为只记不叫醒；非批 49 引入，但「只管 agent」收窄后与现役的根分区告警更不合 |
| K1-1 / S7 `:67` 任务卡住由服务判、看门狗不读服务 | 一致 | `task_server_lib.py:119`；`task-server/task:16`；脚本无读服务的代码 |
| K1-1 / S8 `:67` 先分两类、卡住才处置 | 一致 | 去掉的是已无对应告警的一截；括号没覆盖其余叫醒的告警是补丁前就有的缺口 |
| K1-1 / S9 `:67` 坏卡不由看门狗报 | 一致 | 脚本零 `card` 命中；`task_server_lib.py:106、:1148` |
| K1-1 / S10 `:67` 另有两样 | 一致（两样本身）；「两样」说不清 | 报告里另有范围声明、中途停下没交回、根分区一行 |
| K1-1 / S11 `:67` 不发整点询问 | 一致 | `watch.conf:28`；脚本默认 60（`agent-watch.py:3862`） |
| K1-1 / S12 `:73` `--ack` 三种 | 一致（对文件头与 `watch.sh:209`）；旁证不一致 | `watch.conf:24` / `agent-watch.py:118` 推荐的 `本机:根分区余量过低` 过不了 `watch.sh:209` 的头（补丁前就在） |
| K1-1 / S13 `:73` 自己交的任务 | 一致 | `task-server/task:16、:24、:26` |
| K1-1 / 残留已删告警或阈值 | 无 | 五份文件两遍全文 grep 零残留（`agent-watch.py:4、:10、:12` 是「已删」叙述） |
| K1-1 / 把留下的一半也说没了 | 无 | 子 agent 侧的告警、`scratch-idle-hours`、已失效、`ask-every-minutes=0`、`--ack` 三种都还在 |
| K1-2 / `task wait` 叫醒 | 定义写了；实现真有 | `main-agent.md:67、:73`；`task-server/task:16、:276-278、:292-297` |
| K1-2 / `control check` | 定义没写；实现真有 | 全 `.claude` 零命中；`task-server/control:13、:176`；`task_server_lib.py:2334-2342` |
| K1-2 / `control watch`（交界发现） | 定义没写；实现真有 | `task-server/control:25、:311`；是原「任务服务不通」告警的最近替身，是否构成漏报归 K2 |

## 6 要主 agent 跑的命令

我没写模型，没有 `tools/rerun.sh`，也没有要复跑的 sha256 清单（五份被判文件的 sha256 见第 0 节，与快照一致）。下面这些我自己没起（`heavy-test-guard` 把「起 `scripts/agents/watch.sh`」按项目脚本拒了），都是整条交服务的写法；两份自证在 `.claude/hooks/selftest-registry.tsv:32、:169` 登记为 light，没有重型。

1. 自证（`watch.sh` 补丁改了 ⑪ ⑫ 两格，删了 ⑫ 旧格与 `acktaskform` 开关）：`task-server/task submit -- bash scripts/agents/watch.sh --selftest`。跑完看末行是否为「✓ watch.sh 自检通过」；退 0 算过。
2. 自证（补丁删了 24+1 条开关与自证格；我只核了语法 `ast ok`）：`task-server/task submit -- python3 scripts/agents/agent-watch.py --selftest`。退 0 算过。
3. 核 S12 的四种写法（期望：前两条退 2 且 stderr 有「不是 --ack 认的形态」；后一条打出带两个 `--ack` 的命令行）：
   - `task-server/task submit -- bash scripts/agents/watch.sh --dry-run --ack 本机:根分区余量过低`
   - `task-server/task submit -- bash scripts/agents/watch.sh --dry-run --ack 任务:12:排队过久`
   - `task-server/task submit -- bash scripts/agents/watch.sh --dry-run --ack 类:交回之后后台还在跑,进程:7`
4. 核旧键被拒（期望退 2，stderr 有「agent-watch.py 不认识」）：`task-server/task submit -- bash -c 'printf "task-queue-minutes=10\n" > /home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-sonnet/old.conf && WATCH_CONF=/home/fy5090/.local/share/singlefs/work/watchdog-task-half-r1-sonnet/old.conf bash scripts/agents/watch.sh --dry-run a0000000000000001'`。
5. 弄坏开关各自红在哪一格（期望退 1 且点名 ⑪）：`task-server/task submit -- env WATCH_BREAK=ackscopeform bash scripts/agents/watch.sh --selftest`。
6. 交回前我要跑的引文核对（直接起的文本工具，不经服务）：`python3 scripts/agents/cite-check.py research/prompts/watchdog-task-half-r1-sonnet-output.md`（见第 7 节）。

## 7 引文核对

交回前在本报告上跑 `python3 scripts/agents/cite-check.py research/prompts/watchdog-task-half-r1-sonnet-output.md`，末行：

`✓ 核了 31 处引文、75 处裸行号，对上 31 处，没判 75 处（其中裸行号没有引文可比 66 处；逐处列在上面）`

（没判的 9 处是只写文件名的 `main-agent.md`、`agent-common.md` 引用，checker 认不出是仓里哪个同名文件；它们指的是 `.claude/main-agent.md`、`.claude/agent-common.md`，我已用 `grep -nF` 逐句核过行号。）

## 8 没做什么

- 不判 K2、K3（漏报、别处残留引用、被删开关与自证格）；第 4.3、4.4 节只摆事实，不替主 agent 或攻方腿判「漏」。
- 没往统一任务服务交任务，第 6 节的命令都没跑，判决要引的读数（两份自证末行、四条 `--dry-run` 的退出码）还没有，主 agent 复跑之前一律当「推的」。
- 没核 `.claude/main-agent.md` 里不是这两份补丁带来的改动（别的会话「删轻型并入重型」那几处）。
- 没读攻方腿这一轮的提示、产出与草稿（禁读清单三项）。
- 没全仓 grep 其他定义、`.claude/hooks/` 的拒绝说明、`.claude/rules/` 里对已删告警的引用（K2 的格）；我的残留 grep 只限五份被判文件与 `.claude/main-agent.md`。
