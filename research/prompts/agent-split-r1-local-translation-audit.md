# subagent 拆分执行计划 第一轮：本地两腿英文提示的逐句核对（2026-09-16，主 agent）

依据 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节：英文里的每一句转述写完都要与原文并排核，缺限定词就补。
下表逐项对到原文件自己的行号；「首稿缺的」一列记下跑之前改掉的地方（用 `research/scripts/replace-batch.py` 定点替换，9 处、2 个文件、回读一致）。
两份提示的定稿即 `agent-split-r1-local-attack.md` 与 `agent-split-r1-local-defense.md` 现在的内容，发给本地模型的就是它们。

| 英文项 | 原文（文件:行） | 首稿缺的 / 放宽的 | 定稿 |
|---|---|---|---|
| S1 推论的定义与三条腿 | `.claude/rules/three-way-inference.md:10-11`、`:28-38` | 无 | 与原文同范围 |
| S2 主 agent 评判 | `.claude/rules/three-way-inference.md:199`，节标题「判决由主 agent 做，不由投票做」 | 无 | 同 |
| S3（攻方）校验路子独立且要证明会红 | `.claude/singlefs-ai-sop/rules/evidence-discipline.md:11`、`:17-23` | 无 | 同 |
| S3（辩方）跑前写死、不看已有结论 | `.claude/singlefs-ai-sop/rules/test-discipline.md:37-38`、`:48` | 漏了前半句「不许预设结论」 | 补上 |
| S4 继承实测 | `records/2026-09-16-subagent拆分提案.md:25-28` | 把「五句逐字抄对」说成「继承全部项目规则」——抽了五句，不是逐行核过 | 改成「这五个来源在它上下文里；只抽了五句、一源一句，没有逐行核」 |
| S5（攻方）腿今天的做法 | `research/prompts/c143-r3-opus.md:16`（不许读别的腿这一轮的产出）；general-purpose 类型工具为全部（Agent 工具说明） | 无 | 同 |
| S5（辩方）仓里的数 | `records/2026-09-16-subagent拆分提案.md:11-19`、`:24` | 写了「每轮每腿一份」，那个数只是文件计数，没核每轮每腿 | 删掉「每轮每腿一份」 |
| S6（辩方）同一个人漏节 | `.claude/rules/three-way-inference.md:101` | 无 | 同 |
| P1（攻方）主 agent 保留清单 | `records/2026-09-16-subagent拆分提案.md` 第三节表 | 漏了「实现今天的样子那一行观测」「决定采纳」「决策的判断部分」「读报告后抽查」 | 补上 |
| P2（攻方）本地两腿与核查员 | `records/2026-09-16-subagent拆分提案.md:83-85` | 本地腿漏了「不用 markdown 强调」「补限定词」「前台」「退出码 5 留作废副本」「交样本、词数与过闸记录」与辩方立场；核查员写成「规则或 kb 引用」（原文只有 kb 引用），漏了 sha256 与「行号误写成背景材料的行号」 | 补上，核查员改回只核 kb 引用 |
| P3（攻方）派发次序 | `records/2026-09-16-subagent拆分提案.md:55-57` | 漏了材料员 | 补上 |
| P4（攻方）每个 agent 三步 | `records/2026-09-16-subagent拆分提案.md:162-168` | 第 1 步漏了「一起写」「每条交付规矩指到规则原文」 | 补上 |
| P5（攻方）落地次序 | `records/2026-09-16-subagent拆分提案.md:157-160` | 第 0 步漏了「各配会红的样本」与实测的三件事 | 补上 |
| P6（攻方）/ P2 (b)(c)（辩方）隔离收益 | `records/2026-09-16-subagent拆分提案.md:39-40` | 无 | 同 |
| P2 (d)（辩方）主线不被淹 | `records/2026-09-16-subagent拆分提案.md:24`（中间产物量）与 `:43`（回扫一行） | 把两件事并成「agent 读完报告与材料交分类清单」，原文没有这句 | 拆回两件事 |
| P2 (e)（辩方）清单由另一个上下文标 | `records/2026-09-16-subagent拆分提案.md:42` | 无 | 同 |
| P3 (i)(ii)(iii)（辩方）不嵌套的三条理由 | `records/2026-09-16-subagent拆分提案.md:56-57` | 无 | 同 |
