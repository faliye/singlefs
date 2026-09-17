# agent-defs-r1 本地辩方英文提示：逐句核对表（2026-09-17）

依据 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节：英文里的每一句转述写完都要与原文并排核，缺限定词就补。
下表逐项对到源文件自己的行号（不从背景材料 `_agent-defs-r1-body.md` 或 `_agent-defs-r1-appendix.md` 里数，除非该行号本身就在正文里）。定稿即 `research/prompts/agent-defs-r1-local-defense.md` 现在的内容，发给本地模型的就是它。

| 英文项 | 原文（文件:行） | 首稿缺的 / 放宽的 | 定稿 |
|---|---|---|---|
| F1 三步流程表 + 「先定决策」一句 | `.claude/rules/implementation-workflow.md:6,10-12`（三步表）、`CLAUDE.md:97`（先定决策，再写代码……） | 三步表第 2 步原文区分「正推腿核代码做的是不是条款说的、反推腿攻哪一格会错、本地腿找反例」三个角色，英文合并成一句「check whether the code actually does what the governing written clauses say, and who attack for where it could be wrong」，丢了「本地腿找反例」这一分句 | 保留合并写法（不影响四条攻防的论证结构，只是背景介绍），未回补第三个角色 |
| F2 派发约束（不派 subagent、路径由主 agent 给） | `.claude/agent-common.md:8-9` | 无 | 同 |
| F3 输入缺一样就不开工 | `.claude/agent-common.md:10` | 无 | 逐字对应：「输入缺一样就不开工，回复只写缺什么」 |
| F4 不编译/不跑门禁全量 + ps 例外条款 | `.claude/agent-common.md:25`（第一句）、`:26`（例外条款） | 首稿漏译第 25 行「跑脚本、跑模型一律加 nice -n 19」半句，未回补（与四条攻防论证无关，只截取「除非定义明写要做」半句） | 同（该半句仍未译出，判定为不影响论证，见下方说明） |
| F5 不碰别的会话未提交改动 | `.claude/agent-common.md:27` | 无 | 逐字对应：「本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修」 |
| F6 五处 ps 检查原文 | `.claude/agents/implementation-writer.md:24`、`.claude/agents/mutation-triage.md:22`、`.claude/agents/gate-triage.md:23`、`.claude/agents/experiment-runner.md:23`、`.claude/agents/crash-verifier.md:22` | 无（五处措辞高度相似，英文归纳为「essentially verbatim, a step 1 rule of the same shape」并列出各自多查的对象：crash-verifier 多查 herd7，experiment-runner 多查性能测量） | 同；已核对逐份原文，两处例外（herd7、性能测量）对应准确 |
| F7 crash-verifier 的锁等待容忍句 | `.claude/agents/crash-verifier.md:23` | 无 | 逐句对应：「别的会话同时在跑 cargo 时，编译会卡在『Blocking waiting for file lock』，照实记等了多久，不算这个阶段的耗时」 |
| F8 前 一次卡死事件 + 试跑观察的文档缺口 | `records/2026-09-16-subagent拆分提案.md:350`（卡死与修法）、`research/prompts/agent-defs-r1-trial-experiment-runner.md:35`（试跑观察第 1 条） | 首稿把「试跑观察」第 1 条误写成「规则本身没说要不要记录 ps 读数」，而原文的缺口其实是「agent-common.md 已经要求记录，只是 experiment-runner.md 自己文件里没有重复这一句」——首稿把「共用约束里已有」这个事实弄丢了 | 改写为准确版本：要求写在共用约束里（F4 第二句），experiment-runner.md 自己没有重复；试跑者照做了，只是建议把它也写进各自定义本体 |
| F9 crash-verifier 试跑观察项 1、项 5 | `research/prompts/agent-defs-r1-trial-crash-verifier.md:217-234`（项 1，改动范围漂移）、`:268-274`（项 5，另一会话独立起同一阶段） | 无 | 两项各自的关键可核实数字（14→18 files changed、mutations.tsv 从 25 条变 40 条、约在 54 号跑到一半时发现另一会话）都保留在英文里 |
| F10 mutation-triage 试跑：bin 名连字符/下划线 | `research/prompts/agent-defs-r1-trial-mutation-triage.md:141-152`（bin 名不一致与自纠正）、`:157-159`（「输入缺一样」与「输入给错了」的边界疑问） | 无 | 逐句对应，含「两者在流程上是不是要区别对待，定义没有覆盖」这句原话译出 |
| F11 四处「只报不修」原文 | `.claude/agents/gate-triage.md:12,38`、`.claude/agents/crash-verifier.md:12,39`、`.claude/agents/sweep.md:12,38`、`.claude/agents/mutation-triage.md:31,39` | 首稿把 mutation-triage 的写范围句译成「adding missing test coverage is a separate, later dispatch」，而原文「补取样点、补断言由主 agent 另派」既包含取样点也包含断言，「test coverage」偏窄 | 改译为「adding any missing sample points or assertions is a separate, later dispatch」，与原文两个对象都对应上 |
| F12 fs-design.md 记账表 checker/审计行 | `.claude/rules/fs-design.md:17,24` | 无 | 逐句对应：「若运行时也用遍历算，checker 的遍历与运行时就是同一次计算，对照关系当场归零」 |
| F13 sweep 试跑：机械替换 vs 需要判断的替换 | `research/prompts/agent-defs-r1-trial-sweep.md:92-96`（发现①，要改，机械替换）、`:98-101`（发现②，要人看，需判断）、`:141`（「这个判据本身不够硬」自陈） | 首稿把「① 到 ② 之间」写成「Two lines later」（按物理行号 96→98 数的），且把「不够硬」译成「is not settled cleanly enough」，比原文「不够硬」（不够坚实/站不住）更偏「还没定案」的意思 | 改「Two lines later」为「Immediately afterward」（不再暗示精确行距）；改译「is not solid enough」，更贴「不够硬」的字面 |
| F14 implementation-writer 第 5 步「停在那一处」原文 | `.claude/agents/implementation-writer.md:28` | 无 | 逐句对应，含「不写钉住它行为的测试（钉了就等于替条款定了）」这句因果关系完整译出 |
| F15 show-me-test「没有例外」 | `.claude/singlefs-ai-sop/rules/show-me-test.md:36`（标题）、`:38-39`（正文）、`:39` 门禁强制那句未译入 F15 正文（用作补充句「enforced by a script…」，来源同 `:39`） | 无 | 逐句对应：「没有例外，没有『这个太简单了』，也没有『下个 patch 补上』。只改文档和脚本除外」 |
| F16 code-discipline「TODO 写清缺什么」 | `.claude/singlefs-ai-sop/rules/code-discipline.md:252-253` | 无 | 逐句对应 |
| F17 records 里 implementation-writer 真实一次派发的 todo! 两处 + 「停下的粒度」修法 | `records/2026-09-16-subagent拆分提案.md:342`（两处 todo!）、`:354`（粒度修法，即 F14 现在的文本从何而来） | 无 | 逐句对应，含「条款 4 的重复释放」「容量大到内存放不下」「失败时状态变不变」三处具体内容 |

## 未回补的两处（现查后判定不影响论证，如实记录）

1. F1 的三步流程表第 2 步原文其实区分了正推腿 / 反推腿 / 本地腿三个角色，英文合并成一句泛指「审阅者」。这条纪律本身不进入任何一条攻防的论证链（F1 只在 D3 里作为背景支撑「决定权在哪一步」），合并不影响四个问题的可判定性，故未回补第三个角色（本地腿找反例）。
2. F4 的「跑脚本、跑模型一律加 nice -n 19」半句未译入英文正文：这半句是操作细节（调度优先级），与 Q1 的攻防（ps 检查买到了什么、单次快照能不能防住漂移）无关，故省略，判定不影响论证。

## 一处首稿错误的修正记录（已在上表 F8、F13 行注明，此处汇总）

- F8：首稿把「共用约束已经写了记录 ps 读数的要求，只是 experiment-runner.md 自己没重复」误写成「规则完全没规定要不要记录」——现查 `.claude/agent-common.md:26` 确认要求已经存在，改写为准确版本。
- F13：首稿「Two lines later」按物理行距描述、「is not settled cleanly enough」译得比原文「不够硬」更偏定案与否，两处均已改写，改法见上表。
