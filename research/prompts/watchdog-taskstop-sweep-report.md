# 阶段同步回扫报告：watchdog-taskstop

阶段名：watchdog-taskstop（看门狗认 TaskStop 停掉的子 agent；check-staged 不把 77 当红）。
基准 HEAD：`00c9d4f`。改动文件：`research/scripts/agent-watch.py`、`research/scripts/check-staged.sh`、
`records/2026-09-16-subagent拆分提案.md`（新增第三十三节，本身就是这一阶段的记录，不算要改的载体）。

## 关键词

从改动范围与「这一阶段做成的事」（含两条「没改」）取的关键词：
`agent-watch`（脚本名/agent-watch.py）、`check-staged`（脚本名/check-staged.sh）、`TaskStop`、
`看门狗`、`被停`、`gate-triage`（起因例子里被停的 agent 名）、`无动静`（旧误报的告警名）、
`77`（check-staged 的退出码口径）、`nohup`/`disown`（没改那两处之一）、
`has not reported yet` / `waiting on its own background`（没改那两处之二，"completed" 中间通知原文）。
没有带简称的编号、没有步号可取——这一阶段不产出这类东西。

## 搜索（载体：CLAUDE.md、README.md、.claude/agent-common.md、.claude/agents/、.claude/rules/、
.claude/kb/ 排除 decisions-history.md 与 experiments-history.md 两份变更史、records/ 各文件「## 历史版本」以外的正文；
不搜 research/prompts/ 与 briefs/）

```
$ grep -rln "agent-watch" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
.claude/agent-common.md
.claude/main-agent.md
（2 个文件命中）

$ grep -rln "check-staged" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
CLAUDE.md
.claude/kb/tooling.md
（2 个文件命中）

$ grep -rln "看门狗" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
.claude/agent-common.md
.claude/agents/sweep.md
.claude/main-agent.md
（3 个文件命中）

$ grep -rln "TaskStop" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
（0 个文件命中）

$ grep -rln "gate-triage" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
.claude/agents/crash-verifier.md
.claude/agent-common.md
.claude/agents/experiment-runner.md
.claude/agents/gate-triage.md
.claude/main-agent.md
（5 个文件命中）

$ grep -rln "无动静" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
.claude/main-agent.md
（1 个文件命中）

$ grep -rn "77" .claude/agents/*.md .claude/agent-common.md .claude/rules/*.md CLAUDE.md README.md .claude/main-agent.md 2>/dev/null | grep -vi "177\|277\|377\|477\|577\|677\|777\|877\|977\|2077\|1977"
.claude/agents/gate-triage.md:35
.claude/agents/crash-verifier.md:25
.claude/agents/crash-verifier.md:36
.claude/agent-common.md:49
.claude/rules/three-way-inference.md:72
（5 处命中）

$ grep -rln "nohup\|disown" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
.claude/agents/three-way-local-attack.md
.claude/rules/three-way-inference.md
（2 个文件命中）

$ grep -rln "has not reported yet\|waiting on its own background" CLAUDE.md README.md .claude/agent-common.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/main-agent.md 2>/dev/null | grep -v "decisions-history\|experiments-history"
（0 个文件命中）

$ grep -rln "agent-watch\|check-staged\|TaskStop\|看门狗\|被停" records/ 2>/dev/null
records/2026-09-17-已分配口径三方与两个实验.md
records/2026-09-16-subagent拆分提案.md
（2 个文件命中；后者是本阶段的记录本身，第三十三节不算要改的载体）
```

补搜（.claude/kb/ 下除两份变更史文件外的全部 198 个 `.md`，含 `decisions-history/`、`decisions/`、`experiments/`、
`milestone/`、`layout/` 等子目录，比派发要求更宽）：

```
$ find .claude/kb -type f -name "*.md" | grep -v "^\.claude/kb/decisions-history\.md$" | grep -v "^\.claude/kb/experiments-history\.md$" | wc -l
198
$ 对这 198 个文件逐个搜 agent-watch / check-staged / TaskStop / 看门狗 / 被停 / gate-triage / 无动静
只命中 .claude/kb/tooling.md（check-staged）与 .claude/kb/prior-art.md（被停，与 scrub 速率无关，误命中）
```

## 命中清单

| 载体 文件:行 | 原句 | 分类 | 理由 |
|---|---|---|---|
| `.claude/agent-common.md:42` | `等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（\`research/scripts/agent-watch.py\`）定时复检你；……` | 不相干 | 只说「看门狗会定时复检你」，不涉及「被停」判定用哪种记录、TaskStop 怎么读；换成新行为句子仍然为真 |
| `.claude/main-agent.md:19` | `……或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。` | 不相干 | 只说「被停也会让看门狗退出」这一结论，不描述判据用哪个文件、哪种记录；旧行为与新行为下这句都成立（旧实现漏判的是本次这一种「停在等后台任务」的情形，句子本身没有说错） |
| `.claude/main-agent.md:23` | `叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。` | 不相干 | 说的是主 agent 收到告警之后的决策动作，不描述看门狗内部判定机制 |
| `.claude/agents/sweep.md:22` | `……这一阶段做成的事与第一次在真活上用上的东西，一行一件，没留改动的也写（例：第一次在真派发里用看门狗）；……` | 不相干 | `sweep.md` 自己定义里的举例，指的是「看门狗」这个概念第一次在真派发里用上，与看门狗内部怎么判「被停」无关 |
| `CLAUDE.md:80` | `\`research/scripts/check-staged.sh\` \| 在临时 worktree 上只拿「HEAD + 暂存区」跑 doc-lint 与快的 kb 阶段` | 不相干 | 只说它跑什么阶段，不描述退出码 77 的红绿口径；换成新行为句子仍然为真 |
| `.claude/kb/tooling.md:580` | `……\`research/scripts/check-staged.sh\` 在临时 worktree 上只拿 HEAD + 暂存区跑 kb 阶段。它会把 \`.gitignore\` 掉的 \`.claude/singlefs-ai-sop/\` 拷进 worktree；……` | 不相干 | 同上，只说跑什么阶段与拷贝行为，不涉及 77 的红绿口径 |
| `.claude/agents/gate-triage.md:35` | `一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。` | 不相干 | 这里的 77 是 `gate.sh` 全量门禁本身的退出码口径（本次未跑），`gate-triage` 跑的是 `gate.sh` 不是 `check-staged.sh`；这条口径本来就与 `agent-common.md:49` 一致，未受这一阶段改动影响 |
| `.claude/agents/crash-verifier.md:25` | `每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。` | 不相干 | 同上，管的是 `gate.sh` 阶段的口径，不是 `check-staged.sh` |
| `.claude/agents/crash-verifier.md:36` | `一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行；末尾「没做什么」。` | 不相干 | 同上 |
| `.claude/agent-common.md:49` | `退出码 77 是「本次未跑」，不是通过。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。` | 不相干 | 这条本来就是「77 不算红」的权威定义；`check-staged.sh` 这次的修复是让脚本自己的行为对齐这条既有规则，不是这条规则本身描述了旧行为 |
| `.claude/rules/three-way-inference.md:72` | `D18（块里携带什么信息） 分项 12 那一轮 \| 三档头 68 / 77 / 86 的来历写成「有些树带 key 区间」，而 E73（节点 key 区间的扇出代价） 逐字是……` | 不相干 | 这里的 77 是字节宽度表里的一个数值（头宽 68/77/86 字节），与门禁退出码完全是两回事，纯属数字巧合 |
| `.claude/agents/three-way-local-attack.md:27` | `前台跑 \`bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md\`，不许 \`setsid\`、\`&\`、\`disown\`。` | 不相干 | 管的是三方论证本地腿的跑法，禁令对象是 `ask-local.sh` 这条命令本身，不是 `run_in_background` 起的后台长活；这条规则本来就已经禁 `disown`，与本阶段发现的 `run_in_background` 里加 `nohup … & disown` 是同一类坑在不同命令上的表现，但载体、管辖范围都不同，不是这一阶段之后变假的话 |
| `.claude/rules/three-way-inference.md:302` | `本地腿要前台跑，不要 \`setsid ... & disown\`。实测两次：后台进程消失而……` | 不相干 | 同上，管的是本地腿这一种命令，不是共用约束里 `run_in_background` 的一般用法 |
| `records/2026-09-17-已分配口径三方与两个实验.md:166` | `\`research/scripts/agent-cost.py\` 按 agent 报调用次数……\| 已做：统计做成 \`research/scripts/agent-watch.py cost\`；新旧定义各跑一次同一件确定性的活，结果见第 2b 行` | 事件句不改 | 记的是「用量统计功能已经做成」这一件事，与「被停」判定机制无关，属于历史交付记录 |
| `records/2026-09-17-已分配口径三方与两个实验.md:167` | `子 agent 与脚本的定时监控……\| 已做：\`research/scripts/agent-watch.py\`（watch 每 4 分钟复检……不停任何子 agent、不杀任何进程）……` | 事件句不改 | 记的是「这四项当时做成了什么」，描述的是复检间隔、hook 检出等仍然为真的行为，不涉及「被停」判定用哪种记录；把旧值换成新值（新增了 TaskStop 判据）这句依然是真话——它没有说「被停只认打断」 |

## 没找到「要改」「要补」的命中

对每个命中都用了判据「把这句话换成这一阶段之后的说法，原句还是不是真话」：全部 14 处命中原句在
新行为下依然成立，没有一处断言「被停只按 `[Request interrupted` 判」「check-staged 的 77 一律算红」这类
会被本阶段推翻的具体机制。反向检查（每件做成的事，在该记它的载体里有没有一处记着）：

- 「看门狗改进」——`research/scripts/agent-watch.py` 自身文件头注释（第 19-21 行）已经写清新判据，
  这是权威落点；共用约束与主入口只描述外部可见效果（会被叫醒/会退出），没写内部判据，属于合理的分层，不算漏记。
- 「check-staged 77 不算红」——`research/scripts/check-staged.sh` 自身脚本头注释已写清 `CHECK_STAGED_77_IS_RED=1`
  自检项；`CLAUDE.md:80`、`.claude/kb/tooling.md:580` 只描述它跑哪些阶段，不是它的红绿口径的权威落点，不算漏记。
- 「没改的两处」（main-agent.md 交回怎么读缺"completed"中间通知读法；共用约束与 hook 缺 nohup/disown 警告）——
  全仓搜索确认没有任何地方已经把这两件事描述成「已处理/已覆盖」，所以没有需要撤回的假话；也没有找到
  应当登记这类 agent 基建欠账的调度表或欠账表（`.claude/kb/checks-owed.md` 明确限定在「对着镜像判不出来的
  代码路径要求」，管的是磁盘格式/decisions.md 分项，不是 agent 定义欠账），不算「要补」漏记。

## 没做什么（固定会有的）

- 没改任何文件。
- 阶段同步只搜得到派发提示里写了的事：这一阶段只用过、没留改动、提示里没写的东西搜不到。
- `.claude/kb/decisions-history/`、`.claude/kb/decisions/`、`.claude/kb/experiments/`、`.claude/kb/milestone/`、
  `.claude/kb/layout/` 等子目录不在派发要求的「.claude/kb/（两份变更史除外）」字面范围内（字面只除外两份
  history 文件），但已经额外搜过、0 命中，写在「补搜」一节，供主 agent 判断要不要采信。
- 没有带简称的编号、没有步号可从这一阶段的改动里取出，两项关键词类别为空。
