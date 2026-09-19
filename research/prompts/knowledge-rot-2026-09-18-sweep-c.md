# 知识腐烂回扫 · 领域 C：subagent 协作体系、hook、三方规则（2026-09-18）

回扫员：sweep。阶段：knowledge-rot-c。改动范围：`git diff b1c8cef~1 HEAD -- CLAUDE.md .claude/main-agent.md .claude/agent-common.md .claude/agents/ .claude/hooks/ .claude/settings.json .claude/rules/ research/scripts/agent-watch.py research/scripts/cache-keepalive.sh records/`，加工作区。重点提交：39da7ca、81df09b、f0e2185、502ba80、dddbabf。

只找、只分类，不改任何文件。

## 关键词（从改动范围与做成的事取）

改过的文件/路径：`refuse-overwrite-untracked.sh`、`agent-write-scope.sh`（脚本本体，非 tsv）、`write-guard.sh`、`agent-common.md`、`main-agent.md`、`CLAUDE.md`、`agent-watch.py`、`cache-keepalive.sh`、`after-compact.sh`、`bash-command-detector.sh`、`runner-dispatch-guard.sh`、`bash-wait-guard.sh`（旧名）、`stage-owners.tsv`、`implementation-workflow.md`、`three-way-inference.md`。
门禁号：62、63、64、66、67、68、69、71、72、34、15、87。
带简称编号/轮名：E153、E154、E155、m2-agentdef-r1、alloc-basis-r3。
旧说法：「四条腿」「三轮不停」「定义继承 CLAUDE.md」「CLAUDE.md 调度表」「15/87 号归 experiment-runner」、看门狗旧寿命说法（240 分钟）。

## 搜索命令与计数

```
grep -rn "refuse-overwrite-untracked" CLAUDE.md README.md .claude/agent-common.md .claude/main-agent.md .claude/agents/ .claude/rules/ .claude/kb/ .claude/skills/ records/  → 2 处（均在 records/2026-09-16-subagent拆分提案.md 第 257、538 行）
grep -rn "agent-write-scope\.sh" 同上范围  → 4 处（均在同一份 records，第十六、十七节等，均为经过陈述）
grep -rn "四条腿" 同上范围  → 4 处（均在同一份 records，第十六、十九节与历史版本，均为经过/事件陈述）
grep -rn "三轮不停|不停机|三轮之后停|第三轮之后停" 同上范围  → 命中于 .claude/main-agent.md:39、.claude/rules/three-way-inference.md:212、两处 kb/records 文件，逐条见下
grep -rln "继承.*CLAUDE|CLAUDE.md.*继承" 同上范围  → 15 份 .claude/agents/*.md（「开工先读」句式）+ 2 份 records
grep -rln "什么时候派哪个 agent|调度表" 同上范围  → .claude/main-agent.md、CLAUDE.md、.claude/agents/sweep.md、2 份 records
grep -rn "看门狗|agent-watch|write-guard|omitClaudeMd|cache-keepalive|runner-dispatch-guard|bash-command-detector|after-compact" .claude/kb/（除两份变更史）  → 0 处
grep -n "agent|subagent|门禁|watchdog" README.md  → 与本阶段无关的通用门禁说明，0 处涉及本阶段工具
```

## 命中清单

### 1. `.claude/agents/*.md`（15 份，「继承」关键词命中）

文件：`.claude/agents/{crash-verifier,gate-triage,three-way-verifier,three-way-materials,three-way-local-attack,three-way-forward,kb-scribe,implementation-writer,sweep,mutation-triage,experiment-designer,experiment-runner,three-way-attack,three-way-defense,prior-art}.md` 各第 11 行

原句（各文件相同）：「开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。」

判定：不改（说的是现状，与「16 份定义都开了 omitClaudeMd」一致；判据「把旧值换成新值还是不是真话」——这里没有旧值可换，句子本身就是当前事实）。

理由：这正是 dddbabf 之前 f0e2185 推广 omitClaudeMd 之后的现状描述，与本阶段事实一致，不构成腐烂。

### 2. `CLAUDE.md`、`.claude/main-agent.md` 全文

判定：不改。已核实现状：`CLAUDE.md` 只剩「任务从哪进」一句 `@.claude/main-agent.md`，「什么时候派哪个 agent」的调度表已在 `.claude/main-agent.md:35-41` 里，未见旧版调度表残留在 `CLAUDE.md` 正文（`CLAUDE.md` 全文已读，无「什么时候派哪个 agent」的表格本体，只有一句指过去）。三方一行 `.claude/main-agent.md:39` 写「一轮一条」「三轮之后停」，与 dddbabf 后现状一致。

### 3. `.claude/rules/three-way-inference.md`

```
grep -n "条腿|四条|三条" .claude/rules/three-way-inference.md
```
第 29 行：「一轮派三条推论腿：云端攻方（Opus）、云端正推或云端辩方（Sonnet，一轮一条）、本地攻方或本地辩方（一轮一条……）」

判定：不改（现状句，与「三条腿」现状一致）。第 95 行「四条臂的定义在 31–34 行」指的是另一个实验的判据臂数，与三方腿数无关，不误判为四条腿的残留。

### 4. `.claude/gate.d/stage-owners.tsv`

第 5、54 行：15、87 号已登记归 `gate-triage`（备注写明 2026-09-17 从 `experiment-runner` 挪过来）；第 58-65 行：64、65、66、67、68、69、71、72 号均已登记归属（64→sweep，65→experiment-runner，66/68/71/72→gate-triage，67→kb-scribe，69→三方各半，70→kb-scribe,prior-art）。

判定：不改，均为现状且与本阶段事实一致。

### 5. `records/2026-09-16-subagent拆分提案.md`（正文，历史版本节除外）——**别的会话在写**

该文件工作区里有未提交改动（`git status` 显示 `M`），本次判定基于工作区当前内容。

该文件此前已被多轮阶段同步回扫过（文内第 554-562 行「同日第五批」记录了 2026-09-18 一次针对同一份文件的知识腐烂回扫：回扫员判已腐 13、仍为真 16、分不清 11，主 agent 现查改了 15 处；第七批「共用约束这一条还没在真派发里用过」等句已在历史版本节 798 行记为改过）。第 562 行留言：「11 条分不清里……其余 7 条按字面仍真留着」——这些是此前回扫的遗留结论，不在本次重新判定范围内（本次改动范围是 b1c8cef~1..HEAD，晚于那次回扫）。

对本次改动范围（重点 dddbabf 引入的「定义只写怎么做」「main-agent.md 拆分」）逐节核对：

- 第二十八节「定义只写怎么做：门禁 71、72 号与清出定义的经过」——事件记录，不改。
- 第二十九节「共用约束与主 agent 入口搬进 `.claude/agents/`」——记录了搬迁又被用户退回的完整经过，末尾「同日退回（用户定案）」句清楚说明当前状态是「退回 `.claude/` 原位置」+「`CLAUDE.md` 用 `@` 引入 `main-agent.md`」，与现状一致（已核实 `.claude/agent-common.md`、`.claude/main-agent.md` 确实在 `.claude/` 原位置，非 `.claude/agents/` 下）。不改。
- 开头进度句（第 5 行）："看门狗与续派闸 2026-09-17 起在真派发里用过"——与本阶段给定事实"看门狗在真派发里用过"一致，不改。未提及 write-guard 合并与 omitClaudeMd 推广到 16 份的事实，但这两件事在同一段后文（第十六、十二节，及后续第二十一节第三批）已有专述，不算漏登记。

判定：均不改；不再重复回扫已被前一轮知识腐烂回扫处理过的部分（第 554-798 行范围内的判断不重新做）。

### 6. `records/2026-09-17-已分配口径三方与两个实验.md`（正文，历史版本节除外）

第七节「改法与落地状态」表格逐行核对（1a–7 共 10 行）：1e 行原文「2026-09-18 第一次在真派发里放行（派 E155 执行员做第一、二段）」——与本阶段给定事实一致；2b 行「16 份定义开了 omitClaudeMd」——与事实一致；2c 行「不做：用户 2026-09-17 16:26 UTC 答……」——事件句，不改；4 行看门狗描述与本阶段一致。

判定：全部不改，现状描述与本阶段给定事实吻合。

### 7. `.claude/kb/tooling.md`

```
grep -n "唤醒|watchdog|agent-watch|缓存|keepalive" .claude/kb/tooling.md → 只命中第 415、473、626 行，均与本阶段 agent-watch.py/cache-keepalive.sh 无关（讲的是「会话到点自己开工」的通用唤醒机制，早于本阶段）
```

判定：不改，无关。

### 8. `.claude/skills/*`

```
find .claude/skills -type f → crash-test/SKILL.md, gate/SKILL.md, decide/SKILL.md
grep -rln "subagent|agent-common|agent-watch|write-guard|门禁 6[0-9]|门禁 7[0-9]" .claude/skills/ → 0 处
```

判定：不相干，无命中。

### 9. `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」新节（第 16-22 行）

原句摘录：「2026-09-18 一轮里 16 份定义里有 36 行是在定义里记经过，没有任何检查拦过」。

判定：不改，事件句（这是「为什么」段落，规则文件不受 71 号「定义只写怎么做」约束，允许留经过）。

## 没做什么

- 没改任何文件。
- 没有重新判定 `records/2026-09-16-subagent拆分提案.md` 第 554-798 行（含历史版本节）范围内已经被前一轮知识腐烂回扫处理过的判断，只核对了本次改动范围新引入的第二十八、二十九节及开头进度句。
- 没有对 `.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/hooks/bash-command-detector.sh`、`research/scripts/agent-watch.py`、`.claude/gate.d/stage-owners.tsv`、`records/2026-09-16-subagent拆分提案.md` 这几份工作区里有未提交改动的文件做逐字 diff 对比（改动范围的 git diff 只能看到已提交部分）；判定均基于工作区当前内容，按共用约束「本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修」处理，不改动它们。
- 按定义「没做什么」固定条款：按式子推不出的派生（手算后硬编码、换了单位的）可能漏，照写；阶段同步只搜得到派发提示里写了的事，只用过、没留改动、提示里又没写的搜不到。
- 未搜 `research/prompts/`（冻结证据）与 `briefs/`（按日期不回改），按定义排除。
