<!-- knowledge-sync -->
# watchdog-taskstop 阶段同步

阶段：看门狗认 TaskStop 停掉的子 agent；check-staged 不把退出码 77 当红（2026-09-18）。基准 HEAD `00c9d4f`，UTC。
触发文件：research/scripts/agent-watch.py、research/scripts/check-staged.sh
经过与没改的两处（main-agent.md 没写「completed」中间通知怎么读；共用约束与 Bash 检出 hook 不管 `run_in_background` 里再加 `nohup … & disown`）记在 `records/2026-09-16-subagent拆分提案.md` 第三十三节。

## 搜索

回扫员（`sweep`）的报告与原样输出在 `research/prompts/watchdog-taskstop-sweep-report.md`「搜索」一节：
`agent-watch` → 2 个文件，`check-staged` → 2，`看门狗` → 3，`TaskStop` → 0，`gate-triage` → 5，`无动静` → 1，`77`（退出码口径）→ 5 处，
`nohup|disown` → 2 个文件，`has not reported yet|waiting on its own background` → 0，records/ 里 `agent-watch|check-staged|TaskStop|看门狗|被停` → 2 个文件；
.claude/kb/ 下 198 份 `.md` 补搜，只有 tooling.md（check-staged）与 prior-art.md（「被停」误命中）。共 14 处命中，主 agent 抽核 `.claude/main-agent.md:19`、`CLAUDE.md:80` 两处原句与报告一致。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/main-agent.md:19 | 子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。 | 不改：说的是被停之后看门狗退出这个效果，没写怎么认被停；修了之后这句才真正成立 |
| .claude/main-agent.md:23 | 叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯…… | 不改：说的是主 agent 收到告警之后怎么办，与看门狗的判据无关 |
| .claude/agent-common.md:42 | 等的时候看到进程不动、报错、或等的条件不会成立……主 agent 的看门狗定时复检你 | 不改：只说有看门狗复检，没写判据 |
| .claude/agent-common.md:49 | 退出码 77 是「本次未跑」，不是通过。 | 不改：正是 check-staged 这次对齐的那条规则，原句本来就对 |
| .claude/agents/gate-triage.md:35 | 一张表：阶段 / 绿·红·77 / 归属…… | 不改：管的是 gate.sh 阶段的 77，口径一直是「未跑」 |
| .claude/agents/crash-verifier.md:25 | 退出码 77 记「本次未跑」，不记通过。 | 不改：同上 |
| .claude/agents/crash-verifier.md:36 | 一张表：阶段 / 退出码（0、非 0、77）…… | 不改：同上 |
| .claude/agents/sweep.md:22 | 例：第一次在真派发里用看门狗 | 不改：定义里的举例，与判据无关 |
| CLAUDE.md:80 | check-staged.sh 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段 | 不改：只说跑什么，没写红绿口径 |
| .claude/kb/tooling.md:580 | check-staged.sh 在临时 worktree 上只拿 HEAD + 暂存区跑 kb 阶段 | 不改：同上 |
| .claude/rules/three-way-inference.md:72 | 三档头 68 / 77 / 86 的来历…… | 不改：字节宽度，不是退出码 |
| .claude/agents/three-way-local-attack.md:27 | 不许 `setsid`、`&`、`disown` | 不改：管本地腿那一条命令；共用约束里 `run_in_background` 的一般用法没管，记在拆分提案第三十三节，改定义要走三方 |
| .claude/rules/three-way-inference.md:302 | 本地腿要前台跑，不要 `setsid ... & disown` | 不改：同上 |
| records/2026-09-17-已分配口径三方与两个实验.md:167 | 已做：`research/scripts/agent-watch.py`（watch 每 4 分钟复检……） | 不改：「已做」的交付记录，没断言被停怎么判，换成新行为仍是真话 |
