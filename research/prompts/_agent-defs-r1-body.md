# 背景材料：16 个项目 subagent 定义的低成本正反对抗（2026-09-17）

主 agent 写正文、判据与跑前条款；小节清单由 `research/scripts/kb-sections.py` 全量生成；附录由 `research/scripts/checklist-specs.py` 按清单抽取。
读法：正文只写指路与主 agent 的观测；定义、共用约束、规则的原文整段在附录，不在正文里另做短版本。

这一轮是 `records/2026-09-16-subagent拆分提案.md` 第七节「每批一次低成本对抗」，用户 2026-09-17 要求对全部定义做一次，与逐个试跑一起。**单轮**，不做三轮多数判。

## 一、被判的对象

| 对象 | 路径 |
|---|---|
| 16 个定义 | `.claude/agents/*.md` |
| 共用约束 | `.claude/agent-common.md` |
| 门禁阶段归属表 | `.claude/gate.d/stage-owners.tsv`（核表的门禁 `.claude/gate.d/62-stage-owners.sh`） |
| 写范围闸 | `.claude/hooks/agent-write-scope.sh`、`.claude/hooks/agent-write-scope.tsv`（注册在 `.claude/settings.json`，核它的门禁 `.claude/gate.d/63-agent-write-scope.sh`） |
| 主 agent 的调度表 | `CLAUDE.md`「什么时候派哪个 agent」 |

**用户已定、不在这一轮攻的范围内的**（计划第九节）：按立场拆腿、本地模型一攻一辩、实现员在主工作区改、全部放项目本地、写范围闸走项目 settings hook、写代码按活的形态分、这件事与文件系统 kb 编号分开记。腿若拿出推翻其中一项的**新观测**，照写，主 agent 核实后报用户，不在这一轮判。

## 二、现查事实（主 agent 的观测，2026-09-17）

**实现今天的样子**：

| 看了什么 | 看到什么 | 命令 |
|---|---|---|
| `crates/` 里与 agent 调度有关的实现 | 没有 | `grep -rniE 'subagent\|\.claude/agents\|agent_type' crates/ \| wc -l` → 0 |
| 定义上挂 hook | 在本仓整段不注册：定义所在文件夹没被标成受信任，debug 日志记 `Skipping frontmatter hooks … not trusted` | 计划第十二节 |
| 项目 settings 的 hook | 生效，输入里子 agent 的调用带 `agent_id` 与 `agent_type`，主 agent 的调用不带 | 计划第十二节 |
| 自定义 agent 继承什么 | 不写 `omitClaudeMd` 时继承项目 CLAUDE.md、它 `@` 的规则、用户级 CLAUDE.md、memory 索引（sonnet 探针 5 问全对，起步 99,968 token）；写了就四样都没有（起步 4,741 token） | 计划第十二节 |
| 续做与改定义 | SendMessage 续做沿用第一次派发时的系统提示；改定义后新派取新版；新建定义要等目录监视器刷新（建完约 3 秒派发报 not found，约 27 秒后派得出去） | 计划第十二节 |
| 工具 | 定义里写了会话里不存在的工具名会被静默丢掉；没有 Agent 工具；子 agent 的 Bash 环境里有 `CLAUDE_CODE_EXECPATH`，能起完整 claude 会话，拦它的只有权限 | 计划第十二节 |
| 写范围闸 | 只管 tools 里有 Write 或 Edit 的三个定义；自证 19 种情形，含 4 种走真实 stdin 入口；Bash 里的写拦不住 | `bash .claude/hooks/agent-write-scope.sh --selftest` |
| 交回工具 | 交回（SubagentHandback）只收第一份，第二次被拒 | 计划第十二节、第十六节 |
| 工作区 | 另有会话在改 `crates/`、`.claude/kb/`、`research/`，也在跑 cargo（层 0 全量可跑几个钟头） | `git status --short \| wc -l`、`ps` |

**试跑读数**（主 agent 逐项核过报告里的关键读数；定义里被试跑暴露的问题，多数已在派这一轮之前改掉，J6 判的是还没改的）：

| 定义 | 试跑报告 | 做了什么 |
|---|---|---|
| `implementation-writer` | 计划第十六节（报告已随一次性 crate 删除） | 一次性块分配位图 crate |
| `sweep` | `research/prompts/agent-defs-r1-trial-sweep.md`（它没写报告文件，这是它交回的原文） | 树表条目 148 → 200 回扫 |
| `prior-art` | `research/prompts/agent-defs-r1-trial-prior-art.md` | `btrfs_is_zoned` 在两棵源码树里的计数 |
| `kb-scribe` | `research/prompts/agent-defs-r1-trial-kb-scribe.md` | 仓副本里写回一条虚构定案 |
| `experiment-designer` | `research/prompts/agent-defs-r1-trial-experiment-designer.md` | 仓副本里 E153 跑前登记 |
| `experiment-runner` | `research/prompts/agent-defs-r1-trial-experiment-runner.md`（它没写报告文件，这是它交回的原文） | 仓副本里照 E153 登记实现、产物、复跑、实验页 |
| `mutation-triage` | `research/prompts/agent-defs-r1-trial-mutation-triage.md` | 仓副本里 E67 变异表 |
| `crash-verifier` | `research/prompts/agent-defs-r1-trial-crash-verifier.md` | 真仓上 54、55、57、59 号 |
| `gate-triage` | 试跑与这一轮同时在跑，腿不读 | 真仓整轮门禁 |
| 三方论证七个 | 这一轮本身就是它们的试跑 | — |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 格 | 打中的定义 | 什么观测会让它触发 |
|---|---|---|
| J1 事实 | 定义或共用约束里一句可核的事实（路径、脚本参数与行为、门禁阶段号与判据、规则小节标题）与现查不符 | 腿给出命令与原样输出，或文件路径加行号加整行抄，主 agent 复跑后与定义原文不同 |
| J2 规则冲突 | 定义要求的做法与仓里一条现行规则冲突，或定义点名的「依据」原文并不支持它那一步 | 腿把规则原文（文件加行号，整行抄）与定义原文并排，两句不能同时成立 |
| J3 做不下去 | 照定义做，在一个具体情形里卡住：输入不够开工、步骤之间互相矛盾、要写的文件不在写范围里（或写范围闸会拒）、要用的工具不在 tools 里 | 腿写出情形，每一步指到定义原句；写范围闸那一类给出 hook 的输入 JSON 与退出码 |
| J4 会出错 | 照定义做，一段可达操作序列导致：越界写、改坏别的会话的文件、证据被污染（产物与输入对不上、腿读到别的腿这一轮的产出）、判决实际由 subagent 做出、该红的门禁不红 | 序列的每一步都能指到定义原句许可它 |
| J5 分工 | 调度表、归属表与定义三处对同一件事说法不同（该谁跑、谁先谁后、谁写哪个文件） | 腿把三处原文并排 |
| J6 试跑读数 | 试跑报告里暴露的问题，定义到今天还没改 | 腿指到试跑报告的那一句与定义原句 |

每一格各报各的判定，不合成一个结论。「没打中」要写试过哪些形状、取样范围多大。

## 四、跑前条款

| 条款 | 内容 | 什么观测会让它触发 |
|---|---|---|
| K1 | J1、J5、J6 打中 ⇒ 主 agent 现查坐实后当场改定义（事实与一致性更正是观测） | 主 agent 复跑腿给的命令，或并排读三处原文，确实不一致 |
| K2 | J2–J4 打中 ⇒ 主 agent 逐步走读坐实后改定义，判决里写明这条改法「被攻过零轮」（这一轮单轮，不等三轮多数） | 主 agent 走读腿给的情形或序列，每一步都成立 |
| K3 | 一格打中的是全部定义共用的前提（共用约束或写范围闸本身）⇒ 改共用约束或闸，不在单个定义里各改一遍 | 同一情形在三个以上定义上都成立 |
| K4 | 腿拿出推翻用户已定项的新观测 ⇒ 主 agent 核实后报用户，不在这一轮判 | 证据直接对着第一节列的已定项之一 |
| K5 | 本地两腿任一样本被 `ask-local.sh` 判红（退出码 5）⇒ 作废重跑，不记成不一致 | 退出码 5，或同目录出现 `-output-void<n>.md` |
| K6 | 攻方腿自己提的改法 ⇒ 只算线索，写回时标「被攻过零轮」 | 改法只在腿自己的推导上成立 |

## 五、腿的分工（四条腿，攻击面互不重叠）

| 腿 | 立场 | 分到的对象与格 |
|---|---|---|
| 云端正推（`three-way-forward`，Sonnet） | 从规则原文与仓里脚本推出「应该是什么」，与定义逐格比 | J1、J2、J5、J6：观测类定义 `sweep`、`prior-art`、`mutation-triage`、`gate-triage`，设计类 `experiment-designer`，以及 `CLAUDE.md` 调度表与 `stage-owners.tsv` 对全部定义 |
| 云端攻方（`three-way-attack`，Opus） | 假设定义错了，找能打穿的输入与序列 | J3、J4：执行类定义 `implementation-writer`、`kb-scribe`、`experiment-runner`、`crash-verifier`，以及写范围闸（hook 与表）与共用约束「写」「不做」两节 |
| 本地攻方（`three-way-local-attack`，本机模型，英文） | 攻三方论证那七个定义 | J3、J4：`three-way-materials`、`three-way-forward`、`three-way-attack`、`three-way-defense`、`three-way-local-attack`、`three-way-local-defense`、`three-way-verifier`，重点是腿之间的禁读、交回与核查的独立性 |
| 本地辩方（`three-way-local-defense`，本机模型，英文） | 替定义里被试跑报告质疑过的保守做法辩护 | 「开工前 `ps` 看到重负载就停下」「输入缺一样就不开工」「只报不修」「碰到条款没写的地方停在那一处」四条：各自防住了什么，去掉之后会出什么错 |

## 六、这一轮的路径与禁读（共同交付规矩写在各腿定义与 `.claude/agent-common.md`，这里只给路径）

| 腿 | 报告 | 模型目录 | 草稿目录 |
|---|---|---|---|
| 云端正推 | `research/prompts/agent-defs-r1-sonnet-output.md` | `research/prompts/agent-defs-r1-sonnet-model/`（要写才建） | `/tmp/claude-1000/agent-defs-r1-sonnet/` |
| 云端攻方 | `research/prompts/agent-defs-r1-opus-output.md` | `research/prompts/agent-defs-r1-opus-model/`（要写才建） | `/tmp/claude-1000/agent-defs-r1-opus/` |
| 本地攻方 | 提示 `research/prompts/agent-defs-r1-local-attack.md`，样本前缀 `research/prompts/agent-defs-r1-local-attack` | — | `/tmp/claude-1000/agent-defs-r1-local-attack/` |
| 本地辩方 | 提示 `research/prompts/agent-defs-r1-local-defense.md`，样本前缀 `research/prompts/agent-defs-r1-local-defense` | — | `/tmp/claude-1000/agent-defs-r1-local-defense/` |

**禁读**：别的腿这一轮的产出，即 `research/prompts/agent-defs-r1-*-output*.md`、`agent-defs-r1-*-model/` 与上表别的腿的草稿目录。背景材料、试跑报告（`agent-defs-r1-trial-*.md`）、仓里的定义与脚本随便读。
