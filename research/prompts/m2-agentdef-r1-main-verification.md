# 判决：agent 定义「只写怎么做」清理、主 agent 入口拆分与搬迁，三方第一轮（2026-09-18）

<!-- doc-lint:not-numbers A1 A2 A3 A4 H1 H2 FP1 FP2 FP3 FP9 M01 M02 M03 M04 M11 M12 M15 M17 M27 R1 R4 -->

正文 `research/prompts/_m2-agentdef-r1-body.md`；背景材料 `_m2-agentdef-r1-background.md`；附录二 `_m2-agentdef-r1-diff.md`；开工快照 `research/prompts/m2-agentdef-r1-start-snapshot.sha256`（24 个文件，腿跑着时没改）。
腿：云端攻方（Opus，A1、A2，报告 `m2-agentdef-r1-opus-output.md`，模型 `m2-agentdef-r1-opus-model/`）、云端正推（Sonnet，A3、A4，`-sonnet-output.md`）、本地攻方（A2 算术那一半，提示 `-local-attack.md`，两份干净样本 `-output-s1.md`、`-output-s2.md`）。核查员 `m2-agentdef-r1-verifier-output.md`：核 174 处，169 ✓、3 ✗、1 核不动；Opus 四份探针在核查员副本上复跑，输出与 sha256 一致。

## 一、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| A1 清理改没改指令 | **打中两处**：H1 `.claude/agents/implementation-writer.md` 第 3 步删掉的「`rsync -a` 保留旧修改时间、cargo 跑上一份二进制」管着另一条路——改坏的副本用 `rsync -a` 从原件还原，定义里没有一句挡它（攻方玩具 crate 量过机制：还原后内容与原件相同，照样跑旧二进制）；H2 `.claude/agents/three-way-verifier.md` 删掉的前提（主 agent 常已改主树）管所有轮，留下的指令只管代码轮（弱）。`gate-triage` 第 4 步改了判归属的条件，逐格比没有一格旧对新错，不算打中。其余各处、「依据 → 开工先读」16 行没打中（14 行冒号后逐字相同，2 行指向相同） | 攻方报告 A1 各节；核查员复跑 `mtime_toy.sh` |
| A2 门禁 71 的判据 | **打中**：误判 FP1、FP2——「开工先读：`文件`「小节」」点名的小节名里带「实测」时 ④ 判红，病根是 ② 先剥「」再判而 ④ 不剥（`.claude/gate.d/71-agent-def-flow-only.sh` 第 109、120 行，主 agent 现查属实）；FP3–FP9 是说明与指令字面逐字相同，属「判别子观测不到」。漏判 27 处（M01–M27），25 处此前没人点名；M01–M04 记的是实测数、经过与现状（例 `crash-verifier.md` 第 24 行「实测约 47 分钟」、`mutation-triage.md` 第 25 行「今天 4 张」「计划第十八节」）；71 号文件头「判不到的」一行把自己的漏判范围写窄了。本地攻方算术那一半没打中：两份样本 40 格与真值（现在的 71 号跑在清理前原件上的判定）逐格相同 | 攻方报告 A2 两节；本地腿 s1、s2 |
| A3 入口完整 | **打中一处**：改前 `CLAUDE.md` 第 12 行「要盯住，不能一直干等，也不强制结束」，今天 `.claude/agents/main-agent.md` 只剩「盯住，不强制结束」，两处都搜不到「干等」（主 agent 现查属实）。其余每一句（含整张调度表）都在。主会话读不读得到 `main-agent.md`：`CLAUDE.md` 没有用 `@` 引它，只有一句指路；正推腿判「有条件一致」，运行时读没读到没法在这一轮里实测 | 正推腿 A3；主 agent 现查 |
| A4 规则与门禁 | **打中一处**：门禁 72 号把改动范围里任何一份 `*-main-verification.md` 都当成这一次的判决，今天路径改写碰过的旧判决 `research/prompts/agent-defs-r1-main-verification.md` 因此让 `agent-common.md` 算作已点名（主 agent 现查：那份文件状态 `M`）；门禁 56 号是同一个形状。71 号 18/18 不漏不重。63 号按名字跳过 `agent-common.md`、`main-agent.md`，而这两份加了 frontmatter 之后被 Claude Code 列成可派发的 agent（全部工具），跳过等于这两份在被派发时没有 `omitClaudeMd` 与「开工先读」两道——写入一侧有写范围闸默认拒绝未登记 agent 兜着 | 正推腿 A4；主 agent 现查 |

## 二、处置：挡不挡本轮课题

本轮课题：今天这批定义、规则、门禁的改动是否把定义清成「只写怎么做」而没改坏指令，入口拆分与搬迁有没有丢东西。A1、A2、A3、A4 的打中都在课题之内。

| # | 打中 | 挡不挡 | 这一轮做什么 |
|---|---|---|---|
| 1 | H1 | 挡 | `implementation-writer.md` 第 3 步加一条指令：还原改坏的副本时，拷回原件之后 `touch` 被改的文件，或删掉副本重拷；不用 `rsync -a` 拷回了事 |
| 2 | H2 | 挡 | `three-way-verifier.md`：没给开工快照的轮（设计轮），腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」，不记 ✗ |
| 3 | 漏判 M01–M27 | 挡 | 逐行改：纯说明、实测数、经过与现状删掉；执行者据以分支的前提（M01 同一行「后台起它的那条命令自己的 `$?` 只说明起没起来」、M11、M12、M15、M17）改写成指令，不直接删 |
| 4 | A3「不能一直干等」 | 挡 | `main-agent.md`「派出去之后」一节补回 |
| 5 | FP1、FP2 | 挡 | 71 号 ④ 与 ② 一样先剥「」再判（攻方改法 R1：两套样本照过，清理前原件上 ①0 ②35 ③3 ④30 与原版逐项相同）；绿样本加一行这种形态 |
| 6 | 漏判里有词法特征的 7 处 | 挡 | 71 号加攻方的五条词法（R4：「实测」后八字内有数字、轮名后跟分数、「今天」后跟数字、标点后「免得 / 以免 / 为了 / 所以」、「计划第 N 节」），红样本各放一行；FP3–FP9 与其余 20 处写进 71 号文件头「判不到的」，写准范围（带词但不在标点后、日期的替身、表格行、括注 / 冒号 / 逗号引出的为什么、格言） |
| 7 | 72 号、56 号的判决来源 | 挡 | 只认这一次改动里**新写**的判决（新增或未跟踪），被改过的旧判决不算；两道各加一份红样本（只有被改过的旧判决点名） |
| 8 | 63 号跳过的两份被列成可派发 agent | 不挡清理本身；是 frontmatter 那条预想的代价 | 交用户：留 frontmatter（再给 `agent-common` 限工具到 `Read`）、上游 doc-lint 加豁免后去掉 frontmatter、或退回原位置 |
| 9 | 主会话读不读得到 `main-agent.md` | 不挡清理本身 | 交用户：`CLAUDE.md` 用 `@` 引入它（主会话与不开 `omitClaudeMd` 的 general-purpose 都会带上约 6 KB），还是保留一句指路 |

正推腿顺带报的一条不在判据格里：开了 `omitClaudeMd` 的子 agent 读 `.claude/singlefs-ai-sop/rules/` 下的规则时，那个目录里的 `CLAUDE.md`（`@` 了 14 份规则）会被自动注入，`omitClaudeMd` 省下的上下文被加回去一部分。不挡本轮，记进 `records/2026-09-16-subagent拆分提案.md` 收尾再判。

**改法都被攻过零轮**：第二节第 1–7 行改完之后走第二轮，只攻这一次的改动差（改写成指令的那几行有没有改义、71 号新判据的误判与 72 / 56 号的新判法）。

## 三、被判的文件（门禁 72 号按路径认）

今天改过的 18 份：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/kb-scribe.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/prior-art.md`、`.claude/agents/sweep.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/three-way-local-attack.md`、`.claude/agents/three-way-local-defense.md`、`.claude/agents/three-way-materials.md`、`.claude/agents/three-way-verifier.md`。十八份都在攻方 A1（清理前后逐处比）与 A2（71 号在今天文件上逐行判）的对象里，`agent-common.md`、`main-agent.md` 另在正推腿 A3、A4 里。

## 四、核查员查出的引用错

- 正推腿引 `.claude/hooks/write-guard.sh` 第 71–76 行「`grep -nF` 命中 `if not patterns: return 2`」，实际 `if not patterns:` 在第 86 行、返回拆在第 87–88 行；「无登记即拒绝」的实质结论不变。
- 本地腿转述核对表把 ② 的「只提到目录（后面直接是反引号或空白）」写成「后面是任意标点」，比原文宽；十个样本不测这一分支。
- 正文附录三 P8 漏列了一处「，」之后的位置（主 agent 填表时漏的），那一处不含任何触发词，不改任何一格的真值。

## 五、这一轮之后

第二节第 1–7 行交一个执行者按逐条规格改（定义、71 / 72 / 56 号及样本），跑 26、47、56、62、63、71、72、89 号与 doc-lint；然后第二轮。第 8、9 行与正推腿顺带那条在收尾时交用户或回表再判。
