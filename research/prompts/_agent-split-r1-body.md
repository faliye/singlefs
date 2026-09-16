# 背景材料：subagent 拆分执行计划（`records/2026-09-16-subagent拆分提案.md`）正反对抗第一轮（2026-09-16）

主 agent 写正文、判据与跑前条款；小节清单由 `research/scripts/kb-sections.py` 全量生成；附录由 `research/scripts/checklist-specs.py` 按清单抽取。
读法：正文第二节只写指路与主 agent 的观测，规则与提案原文整段在附录，不在正文里另做短版本。

## 一、被判的对象

`records/2026-09-16-subagent拆分提案.md` 全文（附录第一段）。它是一份执行计划：把本工程的活拆给十五个项目本地 subagent、
主 agent 平铺调度、先建写范围闸与新门禁再分三批铺、每个落地的 agent 都走一次正反对抗。

**用户 2026-09-16 已定、不在这一轮攻的范围内的五项**（提案第九节表格）：第一批范围；云端腿按立场拆、本地模型加一个攻方一个辩方；
实现员在主工作区改；全部放项目本地；执行计划与每个落地的 agent 都走正反对抗。
腿若拿出推翻其中一项的**新观测**（提案写成之后才有的），照写，主 agent 核实后报给用户，不在这一轮判。

## 二、现查事实（主 agent 的观测，2026-09-16）

**实现今天的样子**：

| 看了什么 | 看到什么 | 命令 |
|---|---|---|
| `crates/` 里与 agent 调度有关的实现 | 没有 | `grep -rniE 'subagent\|\.claude/agents' crates/ \| wc -l` → 0 |
| `.claude/agents/` | 不存在 | `ls .claude/agents` |
| 门禁、脚本、hook 里提到 agent 定义的地方 | 没有 | `grep -rnE 'agents/\|subagent' .claude/gate.d/*.sh .claude/scripts/*.sh .claude/hooks/*.sh research/scripts/*.sh research/scripts/*.py \| wc -l` → 0 |
| 按本地腿旧文件名（`-local`）匹配的门禁或脚本 | 只有 `.claude/gate.d/86-experiment-orphans.sh` 第 8 行一处注释，不是匹配规则 | `grep -rlE '\-local(\*\|\.md\|-output\|-v)' .claude/gate.d/ .claude/scripts/ research/scripts/` |
| 现有的 PreToolUse hook | `.claude/settings.json` 注册 `.claude/hooks/refuse-overwrite-untracked.sh`，只挂在 Write 上；门禁 46 号判它注册着、而且会拒绝 | `cat .claude/settings.json` |
| 实现改动的对抗门禁 | `.claude/gate.d/56-crates-adversarial-review.sh`：改过的 `crates/*/src/*.rs` 要在同一次改动带来的 `*-main-verification.md` 里被按路径点名 | 读脚本第 1–30 行 |
| `research/scripts/ask-local.sh` | 判红时把原样输出存成 `<提示文件名去 .md>-output-void<n>.md`，放在提示文件同目录（第 63–76 行） | 读脚本 |
| 工作区 | 另有会话的未提交改动 107 项，`research/prompts/` 下有它们没跟踪的三方材料（c245、c355-c363、c364 几轮） | `git status --short \| wc -l` |

**subagent 继承什么（主 agent 实测，射程只到 general-purpose 类型）**：2026-09-16 派一个 general-purpose subagent（model sonnet），不许调工具，
问五句分别只出现在项目 CLAUDE.md、共享规则、项目规则、用户级 CLAUDE.md、memory 索引里的话，五句全部逐字抄对；那次派发 0 次工具调用，用量 137,814 token。
自定义 agent（`.claude/agents/*.md`）没测。

**Claude Code 的能力（claude-code-guide 查官方文档，未在本仓实测）**：frontmatter 支持 `name`、`description`、`tools`、`disallowedTools`、`model`（sonnet / opus / haiku / inherit）、
`permissionMode`、`skills`、`memory`、`isolation`（worktree）、`maxTurns`、`background`；subagent 可以再派 subagent（文档写默认最多 3 层）；
frontmatter 里可以写 `hooks`，示例按工具名匹配、调外部脚本，按路径拦要脚本自己判；文档没写 subagent 是否继承 CLAUDE.md，没写每个 agent 单独设推理强度。

**提案第一节那组数**：主 agent 2026-09-16 用提案表里写的命令数出（110 / 62 / 35 / 145 / 42），腿可以复数。

**腿的上下文里已经有什么**：若自定义 agent 与 general-purpose 一样继承（没测），云端腿的上下文里已有项目 CLAUDE.md、全部共享与项目规则、用户级 CLAUDE.md、memory 索引。
附录仍整段抄提案引到的规则小节，给的是带行号的原文；引规则时写规则文件自己的行号。

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 格 | 打中的定义 | 什么观测会让它触发 |
|---|---|---|
| J1 事实 | 提案里一句可核的事实（数、命令输出、文件内容、脚本行为、文档能力、实测的射程）与现查不符 | 腿给出命令与原样输出、或文件路径加行号加整行抄，主 agent 复跑后与提案原文不同 |
| J2 规则冲突 | 提案要求的某个做法与仓里一条现行规则冲突，而提案没写它要改那条规则 | 腿把规则原文（文件加行号，整行抄）与提案原文并排，两句不能同时成立 |
| J3 操作序列 | 照提案派发，一段具体操作序列导致：越界写、三方证据被污染（腿读到别的腿这一轮的产出、产物与提示对不上）、判决实际由 subagent 做出、或一道门禁该红不红 | 序列的每一步都能指到提案原文许可它（写出提案里那一句） |
| J4 闸的判别力 | 提案第五节的某条检查（agent 上挂的 hook、派发前后比对、新门禁的七条判据）存在该红不红、或不该红却红的输入 | 腿给出那个输入：文件内容、工具调用的参数、或 git 状态 |
| J5 基线 | 提案记在「拆成 agent」名下的某项收益，在基线上同样拿得到，而提案没写基线拿不到的理由。基线定义：不建 `.claude/agents/`，把共同交付规矩收进一份提示模板文件（例如 `research/prompts/_leg-delivery.md`），每轮提示引用它，照旧派 general-purpose | 腿写出基线拿到同一收益的具体做法，且提案原文里找不到「基线拿不到」的一句 |
| J6 代价数 | 提案的某个取舍依赖一个它没给数的代价（派发次数、每次继承的 token、每个 agent 一轮对抗三轮多数判的总轮数、墙钟） | 腿指出是哪个取舍、缺哪个数；这一格记「缺代价数」，不算打中 |

每一格各报各的判定，不合成一个结论。「没打中」要写试过哪些形状、取样范围多大。

## 四、跑前条款

| 条款 | 内容 | 什么观测会让它触发 |
|---|---|---|
| K1 | J1 打中 ⇒ 主 agent 现查坐实后当场改提案正文（事实更正是观测，不等三轮），旧句挪到提案文末历史 | 主 agent 复跑腿给的命令，输出与提案原文不同 |
| K2 | J2–J5 打中 ⇒ 挂起，不改提案正文；第二、三轮换攻击面再攻，三轮里多数打穿才改 | 主 agent 逐步走读腿给的序列或输入，每一步都成立 |
| K3 | 一格打中让提案与 J5 基线一起出局（同一序列在基线上同样出错）⇒ 不判提案，另记成共用前提的一笔账 | 主 agent 在基线上重放同一序列，同样出错 |
| K4 | 腿拿出推翻用户已定五项之一的新观测 ⇒ 主 agent 核实后报用户，不在这一轮判 | 腿的证据产生于提案写成之后，且直接对着五项之一 |
| K5 | 本地两腿任一样本被 `ask-local.sh` 判红（退出码 5）⇒ 作废重跑，不记成不一致 | 退出码 5，或同目录出现 `-output-void<n>.md` |
| K6 | 攻方腿自己提的改法 ⇒ 只算线索，写回时标「被攻过零轮」 | 改法只在腿自己的推导或模型上量过 |

## 五、腿的分工（四条腿，攻击面互不重叠）

| 腿 | 立场 | 分到的格与攻击面 |
|---|---|---|
| 云端正推（Sonnet） | 从规则原文推出「应该是什么」，与提案逐格比 | J1 全部事实；J2 对第二节六条判据、第三节保留清单、第四节第一批八个 agent 的定义逐条核它们压着的规则；第七节「每个 agent 三步」与第五节「对抗过没有」那一格能不能照 56 号的形态做成会红的检查 |
| 云端攻方（Opus） | 假设计划错了，找能打穿的输入与序列 | J3 与 J4：写范围三层闸的绕过与误判、新门禁七条判据的误判、并发会话下派发前后比对的失效、限额续做与资源锁；J6：十五个 agent 各走对抗、三轮多数判的总代价 |
| 本地攻方（本机模型，英文） | 攻「自举」与「独立性」 | 用来对抗 agent 定义的腿本身就是待落地的 agent：第一批腿的定义由谁、用什么腿来攻；核查员与被核的腿继承同一份规则，它还算不算独立的第二条路 |
| 本地辩方（本机模型，英文） | 替被提案排除的方案辩护 | J5 基线（提示模板文件、不建 agent 定义）；嵌套派发（一个编排 agent 派三条腿）；二者各自拿得到提案记在拆分名下的哪些收益 |

## 六、交付规矩（云端两腿）

- 读 `research/prompts/_agent-split-r1-background.md`（正文 + 小节清单 + 附录）。引规则或提案写**那份文件自己的行号**，行号去原文件里现查，不从背景材料里数。
- 报告写进 `research/prompts/agent-split-r1-<sonnet|opus>-output.md`：分段写，每一次工具调用写进文件的内容不超过 150 行；第一段排他新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告），之后 `>>` 追加。草稿放 `/tmp/claude-1000/agent-split-r1-<sonnet|opus>/`。最后的回复只写一两句话和文件路径。
- 不许改仓里任何已有文件、不许 git 写操作、不许读别的腿这一轮的产出（`research/prompts/agent-split-r1-*-output*.md`）。
- **不许在任何位置建 agent 定义文件**：项目 `.claude/agents/` 与用户级 `~/.claude/agents/` 都会被本机别的会话加载。要实测的，写成「第 0 步该怎么测、两种结果各读到什么」。
- 不许跑 `gate.sh` 全量、不许编译 Rust；可以读任何文件、跑只读命令、跑单个脚本的 `--selftest`。跑脚本一律 `nice -n 19`。
- 转述规则整行抄，不许摘句；每条结论写「什么现象会推翻它」；打中之后先答四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那一小节）。
