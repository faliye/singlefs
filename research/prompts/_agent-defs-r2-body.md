# 背景材料：16 个项目 subagent 定义的第二轮正反对抗（2026-09-17）

主 agent 写正文、判据与跑前条款；小节清单由 `research/scripts/kb-sections.py` 全量生成；附录由 `research/scripts/checklist-specs.py` 按清单抽取。
读法：正文只写指路与主 agent 的观测；定义、共用约束、规则的原文整段在附录，不在正文里另做短版本。

第一轮（agent-defs-r1）是 `records/2026-09-16-subagent拆分提案.md` 第七节「每批一次低成本对抗」的单轮，判决在 `research/prompts/agent-defs-r1-main-verification.md`，采纳的改法全部「被攻过零轮」。这一轮攻那些改法本身、攻第一轮得出结论的方法，并派辩方复核第一轮判决。**仍不做三轮多数判**：打中的照跑前条款改，改法照样标「被攻过零轮」。

## 一、被判的对象

| 对象 | 路径 |
|---|---|
| 16 个定义（今天的版本） | `.claude/agents/*.md` |
| 共用约束 | `.claude/agent-common.md` |
| 门禁阶段归属表 | `.claude/gate.d/stage-owners.tsv`（核表的门禁 `.claude/gate.d/62-stage-owners.sh`） |
| 写范围闸 | `.claude/hooks/agent-write-scope.sh`、`.claude/hooks/agent-write-scope.tsv`（注册在 `.claude/settings.json`，核它的门禁 `.claude/gate.d/63-agent-write-scope.sh`） |
| 主 agent 的调度表 | `CLAUDE.md`「什么时候派哪个 agent」 |
| 第一轮判决 | `research/prompts/agent-defs-r1-main-verification.md` |
| 第一轮时的定义与共用约束原文（改之前） | `research/prompts/_agent-defs-r1-appendix.md` |

**用户已定、不在这一轮攻的范围内的**（计划第九节）：按立场拆腿、本地模型一攻一辩、实现员在主工作区改、全部放项目本地、写范围闸走项目 settings hook、写代码按活的形态分、这件事与文件系统 kb 编号分开记。腿若拿出推翻其中一项的**新观测**，照写，主 agent 核实后报用户，不在这一轮判。

## 二、现查事实（主 agent 的观测，2026-09-17）

**实现今天的样子**：

| 看了什么 | 看到什么 | 命令 |
|---|---|---|
| `crates/` 里与 agent 调度有关的实现 | 没有 | `grep -rniE 'subagent\|\.claude/agents\|agent_type' crates/ \| wc -l` → 0 |
| 第一轮的事实表 | 仍然成立（定义上的 hook 不注册、项目 settings hook 生效并带 `agent_type`、继承四样、续做沿用旧定义、工具名静默丢、交回只收第一份） | 计划第十二节；第一轮正文第二节 |
| 写范围闸对仓副本的判法 | 副本放在 `/tmp/claude-1000/` 下，表里三个定义都有 `/tmp/claude-1000/**`，所以副本里任何路径都放行，连副本的 `CLAUDE.md` 也放行；真仓的 `CLAUDE.md` 拒（退出码 2）。**试跑在副本里做时，闸没被测到，测到的只是定义里写的做法** | 对 `kb-scribe` 各喂一条 Edit 输入，目标分别是副本的 `.claude/kb/decisions/22-x.md`、副本的 `CLAUDE.md`、真仓的 `CLAUDE.md`：退出码 0、0、2 |
| 会话给 subagent 的系统说明 | 写着不要写报告 .md 文件、结果直接交回；第一轮之后共用约束改成报告放进交回内容、由主 agent 存档 | 计划第十七节 |
| 工作区 | 另有会话在改 `crates/`、`.claude/kb/`、`research/`，也在跑 cargo | `git status --short \| wc -l`、`ps` |

**第一轮之后改过的地方**（判决第三节「改了哪里」一列之外另有这些；改之前的原文在 `_agent-defs-r1-appendix.md`，重新试跑之前的原文没有另存，读数与改法在计划第十八节）：

| 定义 | 改了什么 | 依据 |
|---|---|---|
| `experiment-designer` | 加「登记的固定节名」一节：十三个节名与次序，执行员照节名找东西 | 第一轮 `experiment-designer` 试跑观察 9 |
| `.claude/agent-common.md` | 「写」：这一轮自己新建的文件可以 `replace_all`；草稿目录里用什么写都行。开头一句改成试跑过、对抗过一轮 | 实现员重新试跑 |
| `implementation-writer` | 第 3 步：跑整个测试二进制、记同时红了哪些，每份副本用自己的 target、先跑一份不改的；第 5 步：只停落在那个文件上的条款；第 6 步：条款没写的不是分支时不加、列进报告 | 实现员重新试跑；共用 target 那条主 agent 复现过 |
| `experiment-runner`、`mutation-triage` | `mutate.sh` 改成在 `research/` 下跑、路径相对 `research/`（**第一轮 D1 写的「在仓根下跑」是错的**） | 执行员重新试跑；`research/scripts/mutate.sh` 第 45–47 行 |
| `crash-verifier` | 每个阶段前各 `ps` 一次；后台阶段退出码写进文件；环境判定以阶段报错为准，三条探测命令只作旁证；开跑指纹与起阶段写同一条命令 | 崩溃验证员重新试跑 |
| `kb-scribe` | 输入：翻状态另给状态句、31 号判定、标题行计数，变更史标题写翻完之后的标签；第 1、3 步：哈希开工时记规格点名的文件与当月变更史，relabel 预演之后再记它列出的文件；第 4 步：阶段数用 `wc -l` 数 | 书记员重新试跑 |

**重新试跑读数**（主 agent 核过的；全表在计划第十八节，报告在 `research/prompts/agent-defs-r2-trial-<定义名>.md`）：

| 定义 | 做了什么 | 结果 |
|---|---|---|
| `implementation-writer` | 仓外一次性块区间 crate，`lib.rs` 列成别人在改 | 10 条测试、10 条变异全红；按第 5 步没碰 `lib.rs` |
| `experiment-runner` | 副本里复跑 E67 | 复跑字节一致，变异抓到 3、无效 1、没红 0；在仓根下跑 `mutate.sh` 报「基线就是红的」 |
| `crash-verifier` | 真仓上 57、59、55 号 | 三个全绿；指纹抓到 57 号与 59 号之间别的会话改了 `crates/` |
| `kb-scribe` | 副本里把 D22 第 7 项翻回未定，规格夹一条写 `CLAUDE.md` 的 | 出界那条停住；20、31、33 号红（规格少给、relabel 让变异表锚点腐化）；阶段数手数错 |

**主 agent 已知、不用腿再报的**：门禁 59 号跨轮共用 target 会继承上一轮最后一条变异（文件系统那边的事，计划第十八节「顺带发现」）；副本里的试跑测不到写范围闸（第二节「写范围闸对仓副本的判法」那一行）；第一轮 D1「在仓根下跑」已被打中并改掉。

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

J1–J4、J6 与第一轮正文第三节相同；J5 补了后半句（上一个 agent 的产出与下一个的输入对不上）；另加 J7、J8 两格：

| 格 | 打中的定义 | 什么观测会让它触发 |
|---|---|---|
| J1 事实 | 定义或共用约束里一句可核的事实（路径、脚本参数与行为、门禁阶段号与判据、规则小节标题）与现查不符 | 腿给出命令与原样输出，或文件路径加行号加整行抄，主 agent 复跑后与定义原文不同 |
| J2 规则冲突 | 定义要求的做法与仓里一条现行规则冲突，或定义点名的「依据」原文并不支持它那一步 | 腿把规则原文（文件加行号，整行抄）与定义原文并排，两句不能同时成立 |
| J3 做不下去 | 照定义做，在一个具体情形里卡住：输入不够开工、步骤之间互相矛盾、要写的文件不在写范围里（或写范围闸会拒）、要用的工具不在 tools 里 | 腿写出情形，每一步指到定义原句；写范围闸那一类给出 hook 的输入 JSON 与退出码 |
| J4 会出错 | 照定义做，一段可达操作序列导致：越界写、改坏别的会话的文件、证据被污染（产物与输入对不上、腿读到别的腿这一轮的产出）、判决实际由 subagent 做出、该红的门禁不红 | 序列的每一步都能指到定义原句许可它 |
| J5 分工 | 调度表、归属表与定义三处对同一件事说法不同（该谁跑、谁先谁后、谁写哪个文件），或上一个 agent 的产出与下一个 agent 的输入对不上 | 腿把几处原文并排 |
| J6 试跑读数 | 试跑报告里暴露的问题，定义到今天还没改 | 腿指到试跑报告的那一句与定义原句 |
| J7 修补本身 | 第一轮判决采纳的一条改法：没修到它说要修的那一格（原情形照样可达），或引出了原来没有的 J3 / J4 | 腿把第一轮判决那一行、改前原文（`_agent-defs-r1-appendix.md`）、改后原文三处并排，再给出原情形或新情形的每一步 |
| J8 方法 | 第一轮与试跑得出「定义可用」「改法有效」的方法分不出好坏：换一个真坏了的定义，同样的试跑或对抗也会判它可用 | 腿写出一个具体的坏定义或坏改法，指明哪一次试跑或哪一格判据照样会放过它 |

每一格各报各的判定，不合成一个结论。「没打中」要写试过哪些形状、取样范围多大。

## 四、跑前条款

| 条款 | 内容 | 什么观测会让它触发 |
|---|---|---|
| K1 | J1、J5、J6 打中 ⇒ 主 agent 现查坐实后当场改定义 | 主 agent 复跑腿给的命令，或并排读几处原文，确实不一致 |
| K2 | J2–J4 打中 ⇒ 主 agent 逐步走读坐实后改定义，判决里写明这条改法「被攻过零轮」 | 主 agent 走读腿给的情形或序列，每一步都成立 |
| K3 | 一格打中的是全部定义共用的前提（共用约束或写范围闸本身）⇒ 改共用约束或闸，不在单个定义里各改一遍 | 同一情形在三个以上定义上都成立 |
| K4 | 腿拿出推翻用户已定项的新观测 ⇒ 主 agent 核实后报用户，不在这一轮判 | 证据直接对着第一节列的已定项之一 |
| K5 | 本地两腿任一样本被 `ask-local.sh` 判红（退出码 5）⇒ 作废重跑，不记成不一致 | 退出码 5，或同目录出现 `-output-void<n>.md` |
| K6 | 攻方腿自己提的改法 ⇒ 只算线索，写回时标「被攻过零轮」 | 改法只在腿自己的推导上成立 |
| K7 | J7 打中第一轮的一条改法 ⇒ 这条改法算「两轮里被打中一次」：改它，判决里写清第一轮打中的原情形在新改法下还可不可达 | 主 agent 走读原情形与新情形，都有结论 |
| K8 | 辩方拿出第一轮一条命中够不着的证据（去查同类先例或实测，不是重新论证利弊）⇒ 主 agent 核实后撤回那条改法或改窄，判决写明撤回依据 | 辩方给的先例或实测主 agent 复跑成立 |
| K9 | J8 打中 ⇒ 不改定义，改计划里试跑与对抗的做法，记进计划 | 主 agent 能照腿写的坏定义复现「照样判可用」 |

## 五、腿的分工（四条腿，攻击面互不重叠）

| 腿 | 立场 | 分到的对象与格 |
|---|---|---|
| 云端辩方（`three-way-defense`，Sonnet） | 复核第一轮判决 | 判决第三节每一行：命中够不够得着、改法是不是同样打中所有替代做法；替第一轮**没采纳**的一方辩护（G1 挂起那条的反面「该拒」、本地辩方第 2 条、本地攻方第 1、2、4、5 问）。格：K8 与 J2 |
| 云端攻方（`three-way-attack`，Opus） | 假设第一轮的改法与之后的改动是错的 | J7、J3、J4：第一轮采纳的每条改法在今天定义里的落点，第一轮之后与重新试跑之后改的地方；另加 J5 的后半句：`CLAUDE.md` 调度表里前后接着派的几对 agent，上一个的产出与下一个的输入对不上 |
| 本地攻方（`three-way-local-attack`，本机模型，英文） | 攻得出结论的方法 | J8：试跑在副本里做、闸没被测到；每个定义只试跑一次；试跑的活与规格由写定义的主 agent 出、由主 agent 自己核读数；对抗的判决由改定义的同一个主 agent 做 |
| 本地辩方（`three-way-local-defense`，本机模型，英文） | 替第一轮被换掉的旧做法辩护 | 三条：有 Write / Edit 的定义旧时用 Bash 跑 `replace-once.py` / `replace-batch.py` 改文件（A1 换成 Edit）；报告先写进文件再交回（换成放进交回内容、主 agent 存档）；执行员经「修订」调整报告的量（D3 换成只许收严）。各自防住了什么，换掉之后会出什么错 |

## 六、这一轮的路径与禁读

| 腿 | 报告 | 模型目录 | 草稿目录 |
|---|---|---|---|
| 云端辩方 | `research/prompts/agent-defs-r2-sonnet-output.md`（主 agent 从交回原文存档） | `research/prompts/agent-defs-r2-sonnet-model/`（要写才建） | `/tmp/claude-1000/agent-defs-r2-sonnet/` |
| 云端攻方 | `research/prompts/agent-defs-r2-opus-output.md`（主 agent 从交回原文存档） | `research/prompts/agent-defs-r2-opus-model/`（要写才建） | `/tmp/claude-1000/agent-defs-r2-opus/` |
| 本地攻方 | 提示 `research/prompts/agent-defs-r2-local-attack.md`，样本前缀 `research/prompts/agent-defs-r2-local-attack` | — | `/tmp/claude-1000/agent-defs-r2-local-attack/` |
| 本地辩方 | 提示 `research/prompts/agent-defs-r2-local-defense.md`，样本前缀 `research/prompts/agent-defs-r2-local-defense` | — | `/tmp/claude-1000/agent-defs-r2-local-defense/` |

**禁读**：别的腿这一轮的产出，即 `research/prompts/agent-defs-r2-*-output*.md`、`agent-defs-r2-*-model/` 与上表别的腿的草稿目录。第一轮的全部产出、试跑报告（`agent-defs-r1-trial-*.md`、`agent-defs-r2-trial-*.md`）、仓里的定义与脚本随便读。
