# scripts/agents

派子 agent 与盯着它们、三方论证的材料与引文核对、本地模型网关。从仓根跑：`bash scripts/agents/watch.sh`。

| 文件 | 做什么 |
|---|---|
| `watch.sh`、`watch.conf`、`agent-watch.py` | 看门狗：复检子 agent 的会话记录与本实例底下的长进程，阈值在 `watch.conf` |
| `dispatch-prompt.py` | 派发提示的骨架与检查 |
| `agent-handover.py` | 从会话记录抽交接摘要 |
| `agent-cost.py` | 折算用量与挂钟 |
| `three-way-materials.py`、`checklist-specs.py`、`cite-check.py` | 三方论证：拼材料、按小节清单抽附录、核腿报告里的引文（带引文的逐字比；不带引文的 `路径:行号` 核行号在不在文件里，超出判红） |
| `remind.sh` | 本工程唯一的定时提醒：`start --every <分钟> --message <内容> [--name <名字>] [--once]`、`stop`、`list`；循环的到点之后由停止钩子 `.claude/hooks/remind-rearm-stop-guard.sh` 让主 agent 重起，写法在 `.claude/skills/remind/SKILL.md` |
| `three-way-rerun-template.sh` | 三方腿模型目录 `tools/rerun.sh` 的模板：仓根与冻结副本从参数接，拷到哪都能跑；腿拷过去只改 `rerun_steps` |
