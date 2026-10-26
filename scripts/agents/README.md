# scripts/agents

派子 agent 与盯着它们、三方论证的材料与引文核对、定时提醒（`remind.sh`）。从仓根跑：`bash scripts/agents/watch.sh`。

新增回归测试放在 [`tests/`](tests/README.md)。从仓根运行 `python3 -B -m unittest discover -s scripts/agents/tests -v`；各脚本原有的 `--selftest` 仍可独立运行。

| 文件 | 做什么 |
|---|---|
| `watch.sh`、`watch.conf`、`agent-watch.py` | 看门狗：复检子 agent 的会话记录与本实例底下的长进程，阈值在 `watch.conf` |
| `dispatch-prompt.py` | 派发提示的骨架与检查 |
| `copy-originals.py` | 改仓内文件之前把原文件留底：`copy <草稿目录> <文件>…` 拷到 `<草稿目录>/originals/<相对仓根的路径>` 并记清单（路径、sha256），`check <草稿目录>` 列出自那以后改过的；目标由它照相对路径算，不在 Bash 里拼 `$(echo $f | tr / _)` 这类变量目标 |
| `agent-handover.py` | 从会话记录抽交接摘要 |
| `agent-cost.py` | 折算用量与挂钟；`outcome [--root …] [--since …] [--until YYYY-MM-DD] [--repo …] [--summary]` 是每个子 agent 一行的结局账（交回后落没落地、被谁用上、返工字样），再按桶、按类型汇总；`--until` 含那一天，成功句报 `--since`、`--until` 的取值 |
| `subagent_transcript.py` | 子 agent 会话记录的共用认法（被 import 的库）：「交回」按送达认（调过 SubagentHandback、后面有它对应、不是错误的 tool_result），「起点」按首条记录的时刻；续做闸 `.claude/hooks/continuation-guard.sh`、`agent-watch.py`、`agent-cost.py` 都从它取 |
| `three-way-materials.py`、`checklist-specs.py`、`cite-check.py` | 三方论证：拼材料、按小节清单抽附录、核腿报告里的引文（带引文的逐字比；不带引文的 `路径:行号` 核行号在不在文件里，超出判红） |
| `snapshot-tree.py` | 三方开工快照倒推成一棵树：`<轮> --out <目录>` 照 `research/prompts/<轮>-snapshot/` 的 `judged-sha256.txt` 与对 HEAD（旧轮对暂存区）的 diff 只铺快照点名的文件，逐份核 sha256，倒推不出、对不上的点名；判决里 `cite-check.py` 的 `--root` 给它铺出的树 |
| `remind.sh` | 本工程唯一的定时提醒：`start --every <分钟> --message <内容> [--name <名字>] [--once]`、`stop`、`list`；循环的到点之后由停止钩子 `.claude/hooks/remind-rearm-stop-guard.sh` 让主 agent 重起，写法在 `.claude/skills/remind/SKILL.md` |
| `three-way-rerun-template.sh` | 三方腿模型目录 `tools/rerun.sh` 的模板：仓根与冻结副本从参数接，拷到哪都能跑；腿拷过去只改 `rerun_steps` |
