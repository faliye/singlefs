<!-- knowledge-sync -->
# 知识同步：阶段同步任务点与门禁 68 号（2026-09-17）

阶段：知识腐烂的回扫 + 阶段同步任务点 + 门禁 68 号。基准 HEAD，UTC。
触发文件（这一阶段自己改的）：`.claude/agents/sweep.md`、`.claude/agent-common.md`、`CLAUDE.md`、`.claude/gate.d/68-knowledge-sync.sh`、`.claude/gate.d/stage-owners.tsv`。
改动范围里别的触发文件是别的会话正在改的（`crates/` 两个实现员、66 / 67 号、E155），各自写各自的同步记录。

做成与第一次用上的事：
- 看门狗（`research/scripts/agent-watch.py watch`）2026-09-17 16:51 起随真派发起过（会话 32df8a2f 六个真活，17:04 `experiment-designer`）——只用过、没留改动。
- 续派闸（`.claude/hooks/runner-dispatch-guard.sh`）17:40 放行 E155 第一、二段的执行员——只用过、没留改动。
- 项目 settings 的 Bash 检出 hook 对 VSCode 会话的子 agent 生效（检出记录里带 `agent_type` 与 `agent_id`）——只用过、没留改动。
- 中途给在跑的子 agent 发消息、交回之后续做同一个子 agent，都在真派发里用过——只用过、没留改动。
- 新写：`sweep` 第四种活、CLAUDE.md 调度表那一行、门禁 68 号与它的红绿样本、归属表一行。

## 搜索

回扫员（`sweep`，报告存档 `research/prompts/knowledge-sync-gate-sweep-output.md`）跑的：

```
grep -rn '看门狗' --include='*.md' --include='*.sh' . | grep -E '没|还没|未'                                  → 16 行 / 7 个文件
grep -rn '续派闸' --include='*.md' --include='*.sh' . | grep -E '没|还没|未'                                  → 2 行
grep -rn 'runner-dispatch-guard' --include='*.md' --include='*.sh' --include='*.tsv' . | grep -E '没|还没|未'  → 3 行
grep -rn 'agent-watch' --include='*.md' --include='*.sh' --include='*.tsv' . | grep -E '没|还没|未'            → 8 行
sed -n '1,552p' records/2026-09-16-subagent拆分提案.md | grep -n -E '没做|还没|没试跑|没拿真活用过|没核过|没测到|没入库|没算'  → 27 行
grep -n -E '没做|还没|没试跑|没拿真活用过|没核过|没测到|没入库' .claude/agent-common.md                        → 3 行
sed -n '8,28p' CLAUDE.md | grep -c -E '没做|还没|没试跑|没拿真活用过|没核过|没测到|没入库'                      → 0 行
sed -n '1,178p' records/2026-09-17-已分配口径三方与两个实验.md | grep -n -E '已做|未做|没做|还没'              → 15 行
```

拆成 40 条逐条判：已腐 13、仍为真 16、分不清 11（逐条判定在存档的回扫员报告里）。主 agent 现查坐实的证据：会话记录导出 `/tmp/claude-1000/-home-fy5090-code-singlefs/147eac16-ddb2-4441-837f-9e6caa39d39f/scratchpad/rot/usage-timeline.md`（163 次派发、看门狗与 ping）、`/tmp/claude-1000/agent-hook-detections.jsonl` 11 行（其中 8 行带子 agent 的 `agent_type`）、子 agent 元数据 `~/.claude/projects/-home-fy5090-code-singlefs/*/subagents/agent-*.meta.json`。

用户同日再指第三批那一段之后补的两条观测：`python3 research/scripts/agent-watch.py cost --agents <本会话两个子 agent>` → `sweep` 起步 1.1 万、内置 `general-purpose` 起步 12.6 万（这一条把「还没核过起步上下文」当场做掉）；用户 16:26 UTC 那句「算了 我觉得不需要做了」的原话与它回的三条，从会话记录 e166d536 16:21–16:26 逐条读出。

「仍为真」的 16 条不进下面的表：它们不需要处置，推翻条件写在存档报告里。下表只放要处置的。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| records/2026-09-16-subagent拆分提案.md:542 | **第三批没做的**：看门狗与续派闸还没在真实验派发里用过；`omitClaudeMd` 推广之后还没在重新加载了定义的派发里核过起步上下文；登记拆成…都没做。 | 改了：整段三项都了结——看门狗与续派闸的第一次真派发写进第五批；起步上下文当场核出 1.1 万（`sweep`，定义在会话开始时重新加载）对 12.6 万（同会话内置 `general-purpose`）；登记拆三份那组按用户 2026-09-17 16:26 UTC 的原话记成不做 |
| records/2026-09-16-subagent拆分提案.md:5 | …第一次拿真活跑之后评估并直接改了一批（第二十一节）； | 补了：进度句补看门狗、续派闸已在真派发里用过与门禁 68 号 |
| .claude/agent-common.md:4 | 这些定义 2026-09-17 写成，每个拿一次性小活试跑过一次…没拿真活用过。 | 改了：不再写进度，只指到那份计划（现状句只留一处） |
| records/2026-09-16-subagent拆分提案.md:224 | 第 0 步实测还有两格没测到…项目 settings 的 hook 在 VSCode 会话里对子 agent 生不生效（只在嵌套会话里测过） | 改了：改成一格；settings hook 那一格按检出记录里的 `agent_type` 判已生效 |
| records/2026-09-16-subagent拆分提案.md:225 | 每个定义只试跑过一次…真活上长期用的粒度与代价没有读数；对抗只做了一轮 | 改了：写两次试跑、两轮对抗与 `agent-watch.py cost` |
| records/2026-09-16-subagent拆分提案.md:295 | 第十四节表第 2 行状态格「没做」 | 改了：写明改成了共用约束「规则怎么读」，没按定义逐个写清单 |
| records/2026-09-16-subagent拆分提案.md:296 | 第十四节表第 3 行状态格「没做」 | 改了：写两次试点与推广到 16 份定义 |
| records/2026-09-16-subagent拆分提案.md:359 | **没做**：只试跑了 `implementation-writer` 一个、只一次；写到一半才收到修改的交互没测到 | 改了：指到第十七、十八节；中途交互写明之后在真派发里用过 |
| records/2026-09-16-subagent拆分提案.md:399 | **没做**：只做了一轮对抗。 | 改了：写明这一节是第一轮，第二轮见第十九节 |
| records/2026-09-16-subagent拆分提案.md:451 | 干到一半收到主 agent 的消息、交回之后再收到消息或被打回、两个 agent 同时改同一个文件这三条路径都没试跑过 | 改了：只留「两个 agent 同时改同一个文件」，另两条写明真派发里用过 |
| records/2026-09-17-已分配口径三方与两个实验.md:9 | 交用户的 8 条岔路列在…第六节，**还没交**。 | 改了：写交过一次、第 5、6 行用户已定，指到第四之二节 |
| records/2026-09-17-已分配口径三方与两个实验.md:158 | 已做：…；这一轮的岔路单还没写 | 改了：写岔路单路径 `research/prompts/alloc-basis-r3-forks.md` |
| records/2026-09-17-已分配口径三方与两个实验.md:174 | 第八节「还开着的」前三条与 E154 六项对应那一条 | 改了：三条已做的删掉；六项对应按 `forks-verify-e154.md` 第 406 行已核，只留对拍不在复跑里 |
| records/2026-09-16-subagent拆分提案.md:554 | （新增） | 补了：第二十一节「同日第五批」三行——两句腐掉的进度句、任务点、门禁 68 号 |
| records/2026-09-16-subagent拆分提案.md:590 | （新增） | 补了：历史版本 2026-09-17 一条，逐处写「曾经…现在…」 |
| records/2026-09-17-已分配口径三方与两个实验.md:182 | （新增） | 补了：那份记录的历史版本一条 |
| .claude/agents/sweep.md:22 | （新增） | 补了：第四种活「阶段同步」的输入、第 7–9 步、产出与「没做什么」各一条；词表补「没有读数」「只试跑」「只做了一轮」 |
| CLAUDE.md:22 | （新增） | 补了：调度表加「一个阶段任务结束」那一行 |
| .claude/gate.d/68-knowledge-sync.sh:1 | （新增） | 补了：门禁 68 号与红绿样本 |
| .claude/gate.d/stage-owners.tsv:62 | （新增） | 补了：68 号归 `gate-triage` |
| records/2026-09-16-subagent拆分提案.md:318 | **没做**：`crash-verifier` 与各定义新加的门禁那一步都没拿真活试跑；归属表里每一格归得对不对是判断，没过对抗 | 改了：后半收窄成「第二轮只核过第 8、20 两行引用属不属实」（`agent-defs-r2-verifier-output.md` 第 15、55 行）；前半不动，`crash-verifier` 确实只有 01:18、06:33 两次标「试跑」的派发 |
| records/2026-09-16-subagent拆分提案.md:430 | **没做**：四个定义各再试跑了一次，其余十二个没有；副本里的试跑测不到写范围闸（第二轮改成按真仓路径探闸，第十九节） | 不改：字面仍真；真活用过不等于「再试跑」，后半句已自带指向第十九节 |
| records/2026-09-16-subagent拆分提案.md:520 | **没做的**：材料员与核查员的耗时没压；代码三方的轮数规则没动；改过的定义没有再试跑 | 不改：字面仍真——`git log` 看 `.claude/rules/implementation-workflow.md` 最新一条仍是 2026-09-16 的 5f9e449 |
| records/2026-09-17-已分配口径三方与两个实验.md:165 | 2c 登记拆成设计、修订记录、读过的文件三份…\| 未做 | 改了：状态从「未做」改成「不做」，带用户 2026-09-17 16:26 UTC 的原话与射程（他回的是分段上限、长等待交回、换新会话三条，这一行其余几项没逐项点名，一并归为不做）；东西确实还没做（`grep -n '自带源文件\|source_file' research/scripts/mutate.sh` 零命中） |
| research/prompts/agent-defs-r2-main-verification.md:72 | G1 仍挂起。 | 不改：冻结证据（判决文件），而且现状仍是挂起 |

## 没做什么

- 68 号只判形式：搜没搜全、判得对不对、「不改」的理由站不站得住，靠人。
- 11 条分不清没有逐条坐实：其中三条（中途消息、交回后续做、E154 六项对应）主 agent 现查坐实为已腐并改了，其余八条按「字面仍真」留着或写在存档报告里。
- 「仍为真」的 16 条没进上表，只在存档报告里逐条写了推翻条件。
- 这一阶段的三处改法（任务点、`sweep` 第四种活、68 号）被攻过零轮，也没试跑过 `sweep` 的阶段同步（这一轮的回扫是按临时提示派的，不是按新写的第 7–9 步）。
