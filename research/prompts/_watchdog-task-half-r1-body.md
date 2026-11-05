# watchdog-task-half-r1：看门狗删掉盯任务服务那一半，第一轮正文（2026-11-05）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

用户原话：「按照目前的这个策略 我觉得可以 移除看门狗了。 因为我们的任务系统自带了提醒，所以看门狗没有存在的必要了」。主 agent 弹窗问删到什么程度，用户选「只删盯任务那一半」（选项原文：「① 删掉（和任务服务重复），② 盯子 agent 的留着；定义里「派出去之后」那一节只删任务那几句」）。用户随后定这一处定义改动走三方：「走三方吧 闲着也是闲着」。

工具员照这个改完（批 49，报告 `/home/fy5090/.local/share/singlefs/work/watchdog-drop-task-half-49/result.md`），主 agent 已把补丁打进主工作区。被判的是这一批的改动：

- `scripts/agents/agent-watch.py`、`scripts/agents/watch.sh`、`scripts/agents/watch.conf`：删掉「任务服务不通」与本会话任务的五种告警（排队过久、跑得远超预计、反复重试、异常结局、服务告警）、只进报告的两样（`control check` 末行、没 clean 的编译目录）、三个阈值（`task-queue-minutes`、`task-overrun-factor`、`task-retry-count`）、`--ack` 的两种键形（`任务:<号>:<告警名>`、`任务服务:任务服务不通`）与它们的自证格、24 + 1 条弄坏开关。
- `.claude/main-agent.md`「派出去之后」一节 7 处改写（定义文件，这一轮的主要对象）。
- `scripts/agents/README.md` 第 9 行跟着改（另一份补丁）。

`.claude/main-agent.md` 此刻还夹着别的会话「删轻型并入重型」那一批没审过的改动（第 10、14 条，写段第 3 步，验证段第 1、2 步，派发表两行），**不在这一轮射程里**；只判批 49 的补丁那几处。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

方案按 `crates/` 与 `scripts/`、`task-server/` 今天的实现来谈。这一轮不碰 `crates/`（`git apply --numstat research/prompts/watchdog-task-half-r1-snapshot/batch-49.patch` 只列四份文件，没有 `crates/` 下的）。被判的定义句子靠的那几样现状：

- 任务卡住由服务判：`task-server/task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行；经 `task-server/task wait <号>` 叫醒（`.claude/main-agent.md` 改后保留的两句，第 67、73 行一带）。
- 调用方怎么用任务服务、出了问题看哪里：`.claude/kb/tooling.md`「跑项目代码的命令一律经 task-server 交」那一节（附录里整节抄了）。
- 服务健康：`python3 task-server/control check`（退 0 才算通）；服务的 `/health` 判「排头放得下却没起」（`task_server.py` 的 `head_fits` 与 `starved_problem`）。
- 坏卡：任务结局 `card-fault`（`task wait` 末行 `type=card-fault`、退 237），主 agent 当场弹窗——这一段原本就不归看门狗，改后保留。
- 看门狗留下的那一半：子 agent 的会话记录复检（无动静、工具调用过长、同一命令反复、结束本轮不会醒、交回后后台还在跑、副本没删、草稿根里超阈值没动的编译目录与仓副本、旧进程留下的记录标「已失效」），判法在 `scripts/agents/agent-watch.py`。

开工快照：`research/prompts/watchdog-task-half-r1-snapshot/judged-sha256.txt`（被判五份文件此刻的 sha256）、`batch-49.patch` 与 `batch-49-readme.patch`（被判补丁原样）。

## 三、格

### K1　改后的定义说的是不是脚本此刻做的（正推）

- `.claude/main-agent.md`「派出去之后」改后的每一句，对着打过补丁的 `scripts/agents/agent-watch.py`、`watch.sh`、`watch.conf`：说看门狗报什么、不报什么、`--ack` 有哪几种写法、阈值在哪配，逐句核与脚本一致；有没有一句还在说已删的告警或阈值，有没有一句把留下的那一半也说没了。
- 删掉之后主 agent 该靠什么知道「本会话任务异常」与「任务服务不通」：改后的定义有没有写清（`task wait` 叫醒、`control check`），这几样在 `task-server/` 里是不是真有（文件:行）。

### K2　删掉那一半之后，有没有原来只靠它才拦得住的事漏了（攻方）

- 原来看门狗盯任务那一半报的每一种（排队过久、跑得远超预计、反复重试、异常结局、服务告警、任务服务不通），逐种问：任务服务此刻自己会不会报、报给谁、主 agent 不在 `task wait` 上时还会不会被叫醒。找出删完之后没人报的那一种，写出能复现的情形。
- 「服务告警」原来经看门狗读 `alerts` 转给主 agent：删完之后服务写的告警（例 `card-fault`、`peer-unavailable`、`record-tampered`）有没有人读。
- 定义里别处（`.claude/main-agent.md` 其余各节、`.claude/agent-common.md`、`.claude/agents/*.md`、`.claude/hooks/` 的说明与拒绝说明、`.claude/rules/`）还在引已删的告警、阈值、`--ack` 键形的，逐处列（全仓 grep，不截断）。

### K3　自证与开关删得对不对（攻方）

- 删掉的 24 + 1 条弄坏开关与自证格里，有没有一条其实还管着留下的那一半（删了它，留下的判法就少了一格能红的）；留下的开关点名的格原文改了的（`ack`、`ackscope`），新格是不是还核原来那件事。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1 | 逐句核改后的定义与打过补丁的脚本一致 |
| 云端攻方（Opus） | K2、K3 | 找删完之后漏报的情形、还在引已删东西的地方、删错的开关与格 |

## 五、被判的改动

`research/prompts/watchdog-task-half-r1-snapshot/batch-49.patch`（四份文件）与 `batch-49-readme.patch`（`scripts/agents/README.md` 一行）。`.claude/main-agent.md` 里不是这两份补丁带来的改动不判。
