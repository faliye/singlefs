# 背景材料：agent 定义「只写怎么做」清理、主 agent 入口拆分与搬迁的三方第一轮（2026-09-18）

<!-- doc-lint:not-numbers A1 A2 A3 A4 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 -->

主 agent 写正文、判据与跑前条款；小节清单由 `research/scripts/kb-sections.py` 全量生成，附录由 `research/scripts/checklist-specs.py` 按清单抽取。正文只写指路与主 agent 的观测，定义与规则原文在附录。
规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」——改 `.claude/agents/` 与改 `crates/` 同规矩，门禁 72 号要求这一轮的判决按路径点名每一份改过的定义。

## 一、被判的对象

**开工快照**：`research/prompts/m2-agentdef-r1-start-snapshot.sha256`（24 个文件，腿跑着的时候主 agent 不改它们）。

| 对象 | 路径 | 今天改了什么 |
|---|---|---|
| 16 份 subagent 定义 | `.claude/agents/<名字>.md`（16 份） | 清掉写在定义里的经过、日期与解释（清理前原件在 `research/prompts/m2-agent-def-cleanup/before/`，逐处处置表 `research/prompts/m2-agent-def-cleanup/disposition.md` 66 行）；「依据：」一行改成「开工先读：」 |
| 共用约束 | `.claude/agents/agent-common.md`（今天从 `.claude/` 搬进来，加了只有 `name`、`description` 的 frontmatter） | 同上清理；开头换成「只写怎么做」的规则句 |
| 主 agent 入口 | `.claude/agents/main-agent.md`（今天新建，同样搬进来、加了 frontmatter） | 从 `CLAUDE.md` 拆出来的调度、判阻塞、收拢与调度表 |
| 公共上下文 | `CLAUDE.md` | 「什么时候派哪个 agent」一节换成「任务从哪进」，指到 `main-agent.md`；改前原文 `git show HEAD:CLAUDE.md` |
| 规则 | `.claude/rules/implementation-workflow.md` | 新加「改 agent 定义与共用约束，走同一条三步」与「代码轮派腿之前记一份开工快照」两节 |
| 门禁 | `.claude/gate.d/71-agent-def-flow-only.sh`（定义只写怎么做）、`.claude/gate.d/72-agent-def-adversarial-review.sh`（改定义要有判决点名）、`.claude/gate.d/63-agent-write-scope.sh`（跳过两份非定义文件） | 71、72 新立；63 改 |

对象的完整改动：`git diff HEAD -M -- .claude/agents .claude/rules/implementation-workflow.md CLAUDE.md`（`main-agent.md` 是新文件）；清理那一步单独的改动：`diff -r research/prompts/m2-agent-def-cleanup/before/agents .claude/agents`。

## 二、现查事实（主 agent 的观测，2026-09-18）

**实现今天的样子**：这一轮判的是 agent 定义、规则与门禁，不是文件系统实现；`crates/` 里与 agent 调度有关的实现没有（`grep -rniE 'subagent|\.claude/agents|agent_type' crates/ | wc -l` → 0）。

| 看了什么 | 看到什么 | 命令 |
|---|---|---|
| `main-agent.md` 会不会被主会话自动读到 | `CLAUDE.md` 用 `@` 引了 21 份规则，没有引 `main-agent.md`，只在第 10 行写一句指路 | `grep -c '^@' CLAUDE.md` → 21；`grep -n main-agent CLAUDE.md` |
| 71 号在清理前的原件上判多少 | 35 行日期没指路、3 处解释性段落、30 处解释性半句 | 现在的 71 号跑在 `before/` 的拷贝上，原样输出 `research/prompts/m2-agent-def-cleanup/gate71-final-on-before-files.txt` |
| 71 号在今天的文件上 | 绿：18 份文件、4 行带日期（2 行只在「」里、2 行同行指路） | `bash .claude/gate.d/71-agent-def-flow-only.sh` |
| 71 号自己写明认不出的 | 不带标签词、不带日期的解释句（例「发散是探索该有的样子」） | 71 号文件头「判不到的」一行 |
| 清理执行员自己点名要审的 6 处 | 删掉的「（本地模型读不到附录）」「（多半是别的会话在制的改动）」、`kb-scribe` 里 doc-lint 那段说明、`crash-verifier` 的「（`env.sh` 不查这三样）」与「别的会话常在同时改 `crates/`：」；`gate-triage` 第 4 步「多半红」改成条件句；`experiment-runner` 第 3 步两句诊断说明合成一句指令 | `records/2026-09-16-subagent拆分提案.md` 第二十八节「没做的」 |
| 搬迁 | 两份文件的全仓引用已改写，门禁 64 号绿；doc-lint 要求 `.claude/agents/` 下每个 `.md` 都有 `name`、`description`（只豁免 `INDEX.md`），两份各加了最小 frontmatter，这一条是交用户的预想 | `records/2026-09-16-subagent拆分提案.md` 第二十九节 |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| A1 清理改没改指令 | 今天对 16 份定义、共用约束的每一处删改，有没有把一条指令的执行行为改了：删掉的「解释」里是否夹着条件、例外、判据或前提（执行者据以分支的东西）；「依据：」→「开工先读：」16 行点名的文件与小节是否一个不少 | 一处删改之后，同一个输入下照新定义做出的动作与照旧定义不同，而照旧定义做才对；或「开工先读」少点名了一处 |
| A2 门禁 71 的判据 | 71 号四条判据有没有漏判（今天 18 份文件里还留着 71 号判绿的解释、记录或经过）与误判（一行纯指令被判红） | 在今天的 18 份文件里指出一行按 71 号字面判绿、而按规则「只写怎么做」是说明的；或造一行纯指令、71 号判红 |
| A3 入口完整 | `git show HEAD:CLAUDE.md` 的「什么时候派哪个 agent」一节每一句，今天在 `main-agent.md` 或 `CLAUDE.md` 里有没有等价的指令；主会话开工时会不会读到 `main-agent.md` | 一句调度指令两处都没有；或一个新开的主会话按 `CLAUDE.md` 与自动加载的规则开工，读不到 `main-agent.md` 的某一条而照旧做法做错 |
| A4 规则与门禁对得上 | `implementation-workflow.md` 两节新规则与 71、72、63 号的判法是否一致；搬迁之后 63、71、72 号对 `.claude/agents/` 下每个文件判一次、不漏不重 | 规则要求的一条门禁判不到，或门禁判的一条规则没写；或一个文件被判两次、被漏掉 |

**反向接受条款**：A1 打中 ⇒ 把那条条件或前提写回定义（写成指令，不写经过）；A2 打中 ⇒ 漏判的那一行改掉，71 号能补判据就补、补不了写进它的「判不到的」；A3 打中 ⇒ 补进 `main-agent.md`，若主会话读不到是真问题，把「`CLAUDE.md` 用 `@` 引入 `main-agent.md`」列成交用户的一条；A4 打中 ⇒ 改规则或门禁。
**失败条款**：一条腿引的定义或规则原文与附录、快照里的文件不符 ⇒ 那一格作废；「没打中」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（三条推论腿，攻击面不重叠）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 找清理改坏了的指令、找 71 号的漏判与误判 | A1、A2 | A3、A4 |
| Sonnet 正推（`three-way-forward`） | 逐句核入口搬家有没有丢指令、规则与门禁是否一致 | A3、A4 | 不替攻方找反例 |
| 本地攻方（`three-way-local-attack`，英文，抽两次） | 按附录三的特征表与 71 号文件头第 9–27 行的四条判据，逐行逐条判红绿；再对每条判据给一个「判据文字这么判、而按规则本意该反过来判」的行的特征组合 | A2 的算术那一半 | A1、A3、A4 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-agentdef-r1-*-output*.md`）与主 agent 的判决（`research/prompts/m2-agentdef-r1-main-verification.md`）；`research/prompts/m2-agent-def-cleanup/gate71-final-on-before-files.txt` 对本地腿禁读（那是附录三的真值）。前几轮判决可读：`research/prompts/agent-defs-r1-main-verification.md`、`research/prompts/agent-defs-r2-main-verification.md`。本地腿的提示不写汉字变量名；样本行里的汉字关键词按附录三给的拼音标记转述，并在核对表里写明每个标记对应的原文。

## 附录三、本地攻方的特征表（主 agent 逐行读原文填的，真值不在这里）

每一行一个样本：出处、它是不是标题行、是不是表格行、是段落头一行还是段落中间一行（列表项各自成段）、剥掉列表记号与 `**` 之后开头的几个字、「」之外有没有 `20\d\d-\d\d-\d\d`、同一行有没有指到文件的 `records/…` 或 `.claude/kb/…` 路径、以及行内每一处紧跟在 `（(，,；;。` 或 `——` 之后（中间可隔空白与一个日期）的那几个字。

| 样本 | 出处 | 标题 | 表格 | 段落位置 | 开头几个字 | 「」外有日期 | 同行指到 records/ 或 .claude/kb/ 的文件 | 紧跟在标点之后的几个字（逐处） |
|---|---|---|---|---|---|---|---|---|
| P1 | `.claude/agent-common.md:9` | 否 | 否 | 中间一行 | 这份与各份 | 有（在 `records/2026-09-16-…md` 路径里） | 有 | 「。」之后是「「为什么」」；「，」之后是「这里不写」；「；」之后是「要用到那些」；「（」之后是「「开工先读」；「。」之后是「门禁 71」；「（」之后是「规则：」 |
| P2 | `.claude/main-agent.md:8` | 否 | 否 | 头一行 | 所有任务从这里 | 有（在 `records/…` 路径里） | 有 | 「；」之后是「为什么这么定」；「，」之后是「公共上下文」；「（」之后是「项目是什么」 |
| P3 | `.claude/main-agent.md:20` | 否 | 否 | 头一行（列表项） | 收拢要定期做 | 无 | 无 | 「，」之后是「逐条判」；「，」之后是「一条都不许」；「。」之后是「发散是探索」；「，」之后是「开口子」；「，」之后是「开出来」 |
| P4 | `.claude/agents/three-way-defense.md:14` | 否 | 否 | 中间一行 | 开工先读： | 无（唯一的日期在「」里） | 无 | 「（」之后是「2026-09-06 用户明令」（在「」里） |
| P5 | `.claude/agents/implementation-writer.md:32` | 否 | 否 | 头一行（列表项） | 在报告里写出 | 无 | 无 | 「（」之后是「哪条构造保证」；「，」之后是「才许写」 |
| P6 | `.claude/agents/three-way-forward.md:27` | 否 | 否 | 头一行（列表项） | 能用命令核的 | 无 | 无 | 「；」之后是「只能由主」；「，」之后是「写「复核」 |
| P7 | `.claude/agents/kb-scribe.md:18` | 否 | 否 | 头一行（列表项） | 逐条改动规格： | 无 | 无 | 「（」之后是「原文整行」；「（」之后是「判决或用户」（「依据」紧跟在顿号之后） |
| P8 | `research/prompts/m2-agent-def-cleanup/before/agents/three-way-local-defense.md:24` | 否 | 否 | 头一行（列表项） | 实测（ | 有（「（2026-09-16 第一轮）」） | 无 | 「（」之后是「2026-09-16 第一轮」；「；」之后是「样本更容易」；「，」之后是「不够就照实报」 |
| P9 | `research/prompts/m2-agent-def-cleanup/before/agents/kb-scribe.md:29` | 否 | 否 | 头一行（列表项） | 跑阶段归属表 | 有（两处） | 无（只有 `.claude/agent-common.md` 与 `.claude/singlefs-ai-sop/scripts/doc-lint.sh`） | 「（」之后是「共用约束」；「，」之后是「不手数」；「（」之后是「2026-09-17 两次试跑」；「。」之后是「红的判它」；「（」之后是「看点名的」；「；」之后是「不是这一轮」；「，」之后是「照写」；「。」之后是「30 号报」；「，」之后是「照抄」；「，」之后是「不据此」；「。」之后是「另跑一次」；「，」之后是「登记给你的」；「，」之后是「而照规格」；「（」之后是「裸编号」；「；」之后是「红在这一轮」；「，」之后是「停下交主」；「，」之后是「不自己改」；「（」之后是「2026-09-17 实测：」 |
| P10 | `research/prompts/m2-agent-def-cleanup/before/agents/implementation-writer.md:27` | 否 | 否 | 头一行（列表项） | 每条新测试证明 | 有（「2026-09-17 实测」） | 无 | 「（」之后是「`rsync -a`」；「，」之后是「跑那条测试」；「，」之后是「记「改坏」；「（」之后是「不把」；「，」之后是「2026-09-17 实测源码」；「，」之后是「先跑一份」；「（」之后是「多半是别的」；「，」之后是「变异证明」；「，」之后是「debug 下」；「，」之后是「照实记」；「。」之后是「`crates/mutations.tsv`」 |
