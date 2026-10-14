# 定时只用 scripts/agents/remind.sh

**这是 singlefs 的项目本地规则**，不在共享 SOP 里。共享规则在 `.claude/singlefs-ai-sop/rules/`。

**本工程设定时（到点提醒、周期检查、过一阵再回来看一眼）只用 `bash scripts/agents/remind.sh`。** 管主 agent 与所有子 agent。

## 怎么做

- 起：`bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> [--name <名字>] [--once]`，用 Bash 的 `run_in_background` 起。不带 `--once` 的循环提醒，每次到点之后在结束这一轮时由 `.claude/hooks/remind-rearm-stop-guard.sh` 拦下、给出照原样重起的命令，一直到 `remind.sh stop`。
- 命令写法、到点之后做什么、关掉与列出：`.claude/skills/remind/SKILL.md`。
- 子 agent 起不了后台任务：要定时的活写进报告交回主 agent。

## 不用

- 工具 `CronCreate`、`ScheduleWakeup`、`RemoteTrigger`；技能 `loop`、`schedule`。
- Bash 与 Monitor 里只为定时而写的 `sleep`（`sleep N; echo …`、`while true; do sleep N; echo …; done`、套 `timeout` 的同类写法、按时刻比较的 `until`）。
- `crontab`、`at`、`batch`、带定时属性的 `systemd-run`、对 `.timer` 单元的 `systemctl start`。

`.claude/hooks/timer-only-remind-guard.sh` 在执行前拒绝这些，拒绝信息写出路。

## 不是定时的等待照旧

等一个进程结束（`until ! kill -0 <pid>; do sleep 20; done`、`python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`）、看门狗 `scripts/agents/watch.sh`、`scripts/hosts/detached-run.sh watch`、Monitor 盯日志（`tail -f <日志> | grep --line-buffered <模式>`）、脚本里面的 `sleep` 不在拒绝范围。

## 门禁管哪一半

门禁 code-tooling 的 agent-write-scope 格判两个钩子在 `.claude/settings.json` 里注册着：`timer-only-remind-guard.sh` 的 matcher 覆盖 CronCreate、ScheduleWakeup、RemoteTrigger、Skill、Monitor 与 Bash，`remind-rearm-stop-guard.sh` 注册在 Stop 上。两个钩子与 `remind.sh` 各有 `--selftest` 和弄坏开关，登记在 `.claude/hooks/selftest-registry.tsv`。

**它管不了的**：提醒内容有没有点名该跑的检查、变量里拼出来的命令与脚本文件里的 `sleep`、MCP 工具提供的定时器，靠 review。
