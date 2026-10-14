---
name: remind
description: 本工程设定时提醒、周期检查、过一阵再回来看一眼时用它——唯一允许的定时办法是 scripts/agents/remind.sh；包含循环、只一次、关掉、列出的命令写法，以及提醒到点之后主 agent 做什么。
---

本工程设定时只有一个办法：`bash scripts/agents/remind.sh`。别的设定时的办法在执行前被 `.claude/hooks/timer-only-remind-guard.sh` 拒绝。规则在 `.claude/rules/timer-only-remind.md`。

## 什么时候用

- 要每隔一段时间跑一次检查或报一次进度（例：每半小时跑一次例行检查）。
- 要过一阵再回来看一眼某件事（例：二十分钟后看长命令跑到哪了）。
- 等一个进程结束、盯日志里的一行字，不是定时：用 `until ! kill -0 <pid>; do sleep 20; done`、`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`、Monitor 跑 `tail -f <日志> | grep --line-buffered <模式>`，或结束本轮等完成通知。

## 命令写法

全部用 Bash 的 `run_in_background` 起 `start`，起完结束这一轮，不轮询，不另起别的计时。

| 要做什么 | 命令 |
|---|---|
| 循环提醒 | `bash scripts/agents/remind.sh start --every <分钟> --message "<提醒内容>" --name <名字>` |
| 只提醒一次 | `bash scripts/agents/remind.sh start --every <分钟> --message "<提醒内容>" --name <名字> --once` |
| 关掉一条 | `bash scripts/agents/remind.sh stop --name <名字>` |
| 列出这个会话登记着的 | `bash scripts/agents/remind.sh list` |

- `--every` 是正整数分钟；`--message` 写成一行，里面点名到点要跑的命令或要看的东西；`--name` 缺省 `default`，只许字母、数字、下划线、连字符。
- 同名的已经有一个在等，再 `start` 会说「已经有一个在等」并退出，不另起；要换内容或周期，先 `stop` 再 `start`。
- 子 agent 起不了后台任务：要定时的活写进报告交回主 agent。

## 到点之后

1. 完成通知带来一行 `REMIND … 「<名字>」第 N 次：<提醒内容>`。
2. 把提醒内容里点名的检查跑完，有问题就处理，没问题报一行进度。
3. 循环提醒：结束这一轮时停止钩子 `.claude/hooks/remind-rearm-stop-guard.sh` 会拦下，给出登记里 `restart=` 那一行的命令；用 `run_in_background` 原样重起，再结束这一轮。一直重起，直到 `stop`。
4. `--once` 的提醒到点即删登记，不重起。

## 只许这一个

这几样在执行前被拒，主 agent 与子 agent 一样；改成「命令写法」一节的写法：

- 工具 `CronCreate`、`ScheduleWakeup`、`RemoteTrigger`，技能 `loop`、`schedule`。
- Bash 与 Monitor 里只为定时而写的 `sleep`：`sleep 1500; echo …`、`while true; do sleep N; echo …; done`、`timeout … bash -c 'until false; do sleep N; done'`、按时刻比较的 `until`。
- `crontab`（`-l` 除外）、`at`、`batch`、带定时属性的 `systemd-run`、对 `.timer` 单元的 `systemctl start`。
