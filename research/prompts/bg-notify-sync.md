<!-- knowledge-sync -->
# bg-notify 阶段同步

阶段：后台任务的通知与交回——共用约束、主 agent 入口、Bash 检出 hook、看门狗、三方腿的 effort（2026-09-18）。基准 HEAD `00c9d4f`，UTC。
触发文件：.claude/agent-common.md、.claude/hooks/bash-command-detector.sh、research/scripts/agent-watch.py、research/scripts/check-staged.sh、.claude/agents/three-way-attack.md、.claude/agents/three-way-forward.md、.claude/agents/three-way-defense.md
三方一轮与写回在 `research/prompts/bg-notify-r1-main-verification.md`；三份定义加 `effort: high` 与去掉子 agent 的缓存续期是用户直接定的（判决第五节）。

## 搜索

回扫员（`sweep`）的报告与原样命令在 `research/prompts/bg-notify-sweep-report.md`「搜索命令与计数」：`run_in_background` → 9，`结束本轮` → 11，`TaskStop` → 5，`交回怎么读` → 3，`bg-notify` → 1，
`cache-keepalive` → 6，`bash-command-detector` → 4，`agent-watch` → 15，`check-staged` → 3，`看门狗` → 24，`completed` → 2，`60 秒` → 2，检出 / 告警 / 自检几种的合并正则 → 15；
去重后 57 处，逐处判定在报告的命中清单里。回扫员交完报告后主 agent 停掉了它（报告已写到「没做什么」一节）。三份定义的 `effort: high` 是回扫之后才改的，主 agent 另查
`grep -rn "effort" CLAUDE.md .claude/agent-common.md .claude/main-agent.md .claude/rules .claude/kb records --include=*.md | grep -ic "effort: \|reasoning effort"` → 0，没有现状句说这三份的 effort。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/main-agent.md:19 | 发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent | 改了：告警清单补上「结束本轮而手里没有在跑的后台任务」 |
| .claude/agent-common.md:40 | 长活可以等……交给它的命令要一直跑到真正的活结束才退出…… | 不改：这一阶段写的那一句本身 |
| .claude/agent-common.md:41 | 结束本轮去等之前，再用 `run_in_background` 起一次 `bash research/scripts/cache-keepalive.sh`…… | 改了：用户定去掉缓存续期，改成「等长活时不起缓存计时器，结束本轮直接等完成通知」 |
| .claude/main-agent.md:21 | 子 agent 的提示缓存是 5 分钟档，由它自己续（`research/scripts/cache-keepalive.sh`……） | 改了：子 agent 等长活时不续提示缓存，主 agent 也不 ping |
| research/scripts/agent-watch.py:18 | 只结束本轮、没交回的子 agent 还在等后台任务（缓存续命见 research/scripts/cache-keepalive.sh），不算结束。 | 改了：删掉缓存续命那句指路 |
| .claude/main-agent.md:29 | 交回按子 agent 的 SubagentHandback 消息判……不会自己醒…… | 不改：这一阶段写的那一句本身 |
| records/2026-09-16-subagent拆分提案.md:751 | 第三十三节的表与小结 | 不改：这一阶段的记录本身，已按写回更新 |
| records/2026-09-17-已分配口径三方与两个实验.md:167 | 已做：`research/scripts/agent-watch.py`（watch 每 4 分钟复检……） | 不改：「已做」的交付记录，没列告警种数，换成新行为仍是真话 |
