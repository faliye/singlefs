<!-- 回扫员（sweep）2026-09-18 交回的原文：它只有读权限、没写报告文件，主 agent 从交回消息里原样落盘（去掉交回框架加的两格缩进） -->
m2-wave2 阶段同步——只判不改，未写任何文件（无写范围可用）。

## 关键词与搜索命令、计数

载体清单（278 个文件）：`CLAUDE.md`、`README.md`、`.claude/agent-common.md`、`.claude/agents/*.md`（16 个）、`.claude/rules/*.md`（7 个）、`.claude/kb/**/*.md`（排除 `.claude/kb/decisions-history.md` 与 `.claude/kb/decisions-history/`）、`records/*.md`（51 个，顶层）；不搜 `research/prompts/`、`briefs/`。

改动范围：`{ git diff --name-only 502ba80 --; git diff --name-only --cached --; git ls-files --others --exclude-standard --; } | sort -u` → 1035 个文件；按 68 号同款五条正则筛出「触发文件」→ **75 个**（`.claude/agents/*.md` 16、`.claude/gate.d/*.{sh,py,tsv}` 22、`.claude/hooks/after-compact.sh`、`CLAUDE.md`、`.claude/agent-common.md`、`.claude/rules/*.md` 3、`crates/*/src/*.rs` 22、`research/scripts/*` 9）。注：主 agent 给的 `g68-files.txt`（54 个，取自不同基准）与这里的 75 个不是同一批——本轮以「502ba80 比工作区」为准，按此重算。

关键词 51 个，`grep -F -f <(printf '%s\n' "$kw") -n <278 个载体文件> | wc -l`：

```
I-5.4=5  I-3.9=7  I-9.14=7  E142=172  21/21=6
71-agent-def-flow-only=1  72-agent-def-adversarial-review=1  69-evidence-in-repo=1
56-crates-adversarial-review=1  54-layer0-replay=3  55-qemu-first-transaction=5
57-lkmm=1  59-crates-mutation-replay=1  rewrite-moved-paths.py=6  path-moves.tsv=5
main-agent.md=7  cache-keepalive.sh=5  agent-watch.py=14  omitClaudeMd=53
crash-verifier=32  implementation-writer=33  kb-scribe=26  three-way-verifier=14
three-way-attack=7  three-way-defense=7  three-way-forward=7  three-way-local-attack=13
three-way-local-defense=9  three-way-materials=15  mutation-triage=13  prior-art=51
experiment-designer=21  experiment-runner=31  gate-triage=29  C381=2  m2-wave2=9
增补 3=4  取号前准入=0  改动计数=20  定义只写怎么做=3  上游副本目录=1
26 条=13  29 条=12  after-compact.sh=0  path-moves.md=3  three-way-inference.md=88
implementation-workflow.md=12  stage-owners.tsv=8  3 小时 50 分=2  6 行锚点=1  123 条=3
```

第二步：与进度词（没做|还没|没试跑|没用过|没拿真活|没核过|没测|没有读数|只试跑|只做了一轮|待|欠|开着|计划|下一步）同句：
`grep -F -n -f <(printf '%s\n' "$kw") <278 个载体> | grep -E '<进度词>'`，51 个关键词各跑一遍，去重合并 → **98 行**。

## 命中处置

逐行核过 98 行，结果如下（原句很长的табле格，按仓规「引产物整行抄」我在文件里已核对，这里给文件:行 + 判定 + 理由，原句可用上面命令复现）：

**要改（4 处，逐条给改法）**：

| 载体 | 判定 | 理由与改法 |
|---|---|---|
| `.claude/kb/verification-build.md:4` | 要改 | 原句「层 0 崩溃点重放在门禁 54 号（第一个事务的全部 262165 个崩溃状态），QEMU 真设备在门禁 55 号」。`.claude/gate.d/54-layer0-replay.sh:2` 的 `gate-stage` 头已写明现在跑两条流（第一个事务 262165 个状态 + 里程碑「第二个事务」固定脚本到 E 的 2104413 个状态）。改成写两条流与状态数 |
| `.claude/kb/verification-build.md:5` | 要改 | 原句「崩溃点重放与 QEMU 真实负载由 54、55 号覆盖，射程只到第一个事务」。同上，`54` 号射程已不止第一个事务；`55` 号仍只到第一个事务（`milestone/02-second-txn.md:222` 现查确认）。两个门禁号的射程要分开写 |
| `.claude/kb/verification-build.md:139` | 要改 | 原句「池级 checker...26 条：第一版 23 条加 I-3.8、I-7.4、I-4.8」。checker 现在是 29 条（`.claude/kb/invariants.md:12`、`CLAUDE.md:97` 都已经是 29 条），少了本阶段新加的 I-3.9、I-9.14、I-5.4 三条。改成「29 条：...加 I-3.9、I-9.14、I-5.4」 |
| `records/2026-09-16-subagent拆分提案.md:581` | 要改 | 「第六批没做的」段末句「共用约束这一条还没在真派发里用过」（指 cache-keepalive 自续提示缓存那条共用约束）。同一文件第三十节（2026-09-18）已经用真实会话数据核过它的效果（起过计时器 7 个 agent，含崩溃验证员），这句已不成立；且 2026-09-17 那次回扫（第 559 行记的「改了 15 处」）没有覆盖到这一批（它列的清单里没有「第六批」）。改成指向第三十节，或直接删掉这半句 |

**不改：事件句（代表性举例，逐条已核，同类不再重复列）**：

| 载体 | 理由 |
|---|---|
| `.claude/kb/invariants.md:299`（## 历史版本内） | 「2026-09-18...加 I-3.9...I-9.14...checker 待接」是 2026-09-13 定案条目上追加的一句历史注，描述加进列表那一刻的状态；同一份文件正文（第 12 行）与该不变量自己的表行（第 131 行）都已写「已实现」，不冲突——历史句留着仍是真话 |
| `records/2026-09-16-subagent拆分提案.md:73,91,92,93` 等「曾经写…现在写…」体例的段落 | 这些本身就是当天回扫改动的记录（对第三批、第十七/十八/十九/二十一节等的「改前/改后」），是事件陈述，不是现状陈述 |
| `records/2026-09-16-subagent拆分提案.md:59,62,63,65` 等「没拿真活试跑」「只做了一轮」段落（第七、十六、十九节区域） | 均带明确日期段落（2026-09-16/17），描述当时那一轮试跑/对抗做没做，后续section已各自记录「现在指到第 X 节」的更新，不需要我重复标记 |
| `.claude/kb/experiments/142-第一个事务的干跑.md` 与 `experiments-history.md` 里“第九次跑”“第十次跑”一类 | 均为带序数的事件描述，与「第十一次跑」（当前最新，已在 25 行正确记录）并不矛盾 |

**不相干（同一个词、不同的量/话题，代表性举例）**：

| 载体 | 理由 |
|---|---|
| `.claude/kb/invariants.md:451`（## 待补 内） | 「现共 26 条在用，全部未实现」说的是 2026-08-26 那批意图日志相关「待补」清单的计数，与本阶段 checker 26→29 条完全是两个量 |
| `.claude/kb/checks-owed.md:136,202,210,246,285,288,289,292,297,330,377` 等（约 20 处） | 都是 `E142` 与「欠/待/计划」在同一个巨大表格单元格内共现，但讨论的是各自独立的历史欠账（C132/C211/C234/C264/C306/C309/C310/C314/C323/C358/C220 等），跟本阶段的具体新事实（21/21、11 次跑、I-5.4 等）无关，只是碰巧提到 E142 这个实验名 |
| `.claude/kb/decisions/05-快照-空间记账机制.md:224,253,365`、`decisions/06-快照实现模型.md:253`、`decisions/16-发布语义.md:209`、`decisions/17-实现分层与第三方管道.md:184,186,199,200`、`decisions/22-单元原子性怎么合成.md:216,657`、`decisions/23-journal的角色与格式.md:691` | 同上，E142/26条/29条/欠/待 的巧合共现，主题各自独立（校验和、livelist、发布语义、journal 重放下界等），与本阶段事实无关 |
| `.claude/kb/experiments/34-根环槽几何.md:176`、`.claude/kb/pitfalls.md` 未命中 | `three-way-inference.md` 关键词的 88 次命中里，除 C234、22-单元原子性、34-根环槽几何 这三处「两条云端腿仍欠着」外，其余绝大多数是正文引用该规则文件名做出处标注（如「见 `.claude/rules/three-way-inference.md`」），非「进度」语义，属误配 |

**要人看（分不清，1 处）**：

| 载体 | 为什么分不清 |
|---|---|
| `.claude/kb/milestone/02-second-txn.md:531` 「步 4 现状加「准入读数」那格没做」 | 这一行本身是 2026-09-17 那批核查（`m2-closeout-inventory-*.md`）改写收口表之后新留的欠账项，我不确定「准入读数」这格是否正是本阶段（wave2）「取号前准入」系列修复（20a/38/39 行）已经顺带做掉的那部分，还是仍是独立开着的欠账；核这个需要读 `m2-wave2-code-r1-main-verification.md` 判决原文之外的具体准入读数实现，超出本轮关键词回扫能确认的范围 |

## 反向核对：做成的事有没有落点

逐条对派发提示「做成的事」核了一遍记它的载体：

| 做成的事 | 落点 | 结论 |
|---|---|---|
| I-5.4 / 复用删相交记录 / 取号前准入 | `.claude/kb/checks-owed.md:354`（C380 邻近）、`.claude/kb/milestone/02-second-txn.md:354,355,356` | 已记，与派发提示一致 |
| E142 第十一次跑 21/21 | `.claude/kb/experiments/142-第一个事务的干跑.md:25,28`、`milestone/02-second-txn.md:320` | 已记 |
| checker 26→29 条 | `.claude/kb/invariants.md:12`、`CLAUDE.md:97` 已更新；**`.claude/kb/verification-build.md:139` 未更新**（见上「要改」） |
| 门禁 54/55/57/59 与 3h50m49s、6 行锚点、123 条 | `records/2026-09-16-subagent拆分提案.md` 第三十二节、`milestone/02-second-txn.md:358,359` | 已记，细节吻合 |
| 门禁 71/72、定义只写怎么做 | `records/…` 第二十八节；`stage-owners.tsv:64,65` 已登记归 `gate-triage` | 已记 |
| 门禁 69/56/72 判决口径 | 未单独搜到有载体明确复述这三句机制（69 号放过纯注释改动、56/72 只认本次新判决），但这是门禁脚本自身的实现细节，不是通常会被复述的「现状句」，未见有陈旧风险 |
| 共用约束/主 agent 入口搬迁与改法 | `records/…` 第二十九节 | 已记，含两个坑与自证 |
| `CLAUDE.md` 用 `@` 引入 `main-agent.md` | `CLAUDE.md:11` 现查确认 `@.claude/main-agent.md` 已在 | 已记 |
| cache-keepalive 首次真用+统计 | `records/…` 第三十节；**但第 581 行旧的「没用过」未回改**（见上「要改」） | 部分：新记录已补，旧记录未清 |
| agent-watch.py 每次派发用上 | `records/…` 第三十二节等多处 | 已记 |
| omitClaudeMd 仍被注入上游 CLAUDE.md | `records/…` 第三十一节 | 已记（这正是新出的判据/发现本身） |
| crash-verifier/implementation-writer/kb-scribe/three-way-verifier 首次真派发 | `records/…` 第三十二节；`m2-wave2-kb-writeback-report.md`、`m2-wave2-code-r1-verifier-output.md`（不在我搜索范围，未查） | kb 之外的记录未查，超出载体清单 |
| 里程碑收口表 37–42 行、增补 3、C381 | `milestone/02-second-txn.md:354-359,381-412,527`；`checks-owed.md:355`（C381） | 已记 |

## 没做什么

- 只搜得到派发提示写了的事；`research/prompts/` 与 `briefs/` 按规则不搜，所以像 `m2-wave2-kb-writeback-report.md`、判决/攻防报告本身有没有陈旧句，没有查。
- 98 行「关键词 + 进度词同句」命中里，`E142` 与 `three-way-inference.md`、`26 条` 三个关键词贡献了绝大多数「不相干」噪音（表格单元格巨大、共现纯属巧合）；这三个关键词本身选得偏宽，回扫价值有限，供主 agent 参考以后不必照抄。
- `.claude/kb/milestone/02-second-txn.md:531` 那一条「要人看」没有再往下核（需要读 `m2-wave2-code-r1-main-verification.md` 等判决原文，不在我的载体清单和关键词回扫范围内）。
- 没有跑门禁 68 号确认；`g68-files.txt`（54 个）与我自己按 502ba80 重算的 75 个触发文件不是同一批，两者差异（主要是 `crash-verifier.md`、`sweep.md`、`agent-common.md`、`CLAUDE.md` 等，可能是另一基准或已被别处点名过），主 agent 写 `research/prompts/m2-wave2-sync.md` 的「触发文件：」一行时建议用我这份 75 个的清单（基准 502ba80，与本轮任务给的基准一致），逐路径核对齐 g68 报的缺口清单。
- 没有改任何文件（本次任务无写范围）；`stale=` 候选不适用于本轮活（本轮不是改格式常量）。

草稿：本轮未使用 `/tmp/claude-1000/m2-wave2-sync/` 草稿目录（没有需要落盘的中间产物），报告全文在本条交回里；主 agent 要求的报告路径 `/tmp/claude-1000/m2-wave2-sync/report.md` 不在本仓写范围之内（该路径不是 `research/prompts/` 也不是我的草稿目录，我没有工具能写到 `/tmp/claude-1000/m2-wave2-sync/`，只能在 `SubagentHandback` 里交回全文，请主 agent 自行落盘）。
