# agent-defs-r1 主 agent 判决（2026-09-17）

正文 `research/prompts/_agent-defs-r1-body.md`，背景材料 `research/prompts/_agent-defs-r1-background.md`。**单轮**低成本对抗（计划第七节），用户 2026-09-17 要求对全部定义做一次。下面每一条改法都是**被攻过零轮**的（跑前条款 K2、K6）。

## 一、腿与用量

| 腿 | 产出 | 用量（token / 工具调用 / 墙钟） |
|---|---|---|
| 材料员 `three-way-materials` | `_agent-defs-r1-checklist.md`、`_agent-defs-r1-appendix.md`、`_agent-defs-r1-background.md` | 234,328 / 60 / 14.0 分钟 |
| 正推 `three-way-forward`（Sonnet） | `agent-defs-r1-sonnet-output.md` | 402,687 / 129 / 26.7 分钟 |
| 云端攻方 `three-way-attack`（Opus） | `agent-defs-r1-opus-output.md`、`agent-defs-r1-opus-model/` | 422,931 / 121 / 30.7 分钟 |
| 本地攻方 `three-way-local-attack` | 提示、核对表、运行记录，干净样本 s1、s5（s4 带损坏，void1、void2 作废） | 238,854 / 35 / 23.7 分钟 |
| 本地辩方 `three-way-local-defense` | 提示、核对表、运行记录，干净样本 s1、s2（void1 作废） | 260,489 / 56 / 20.4 分钟 |
| 核查员 `three-way-verifier` | `agent-defs-r1-verifier-output.md`（主 agent 从交回原文存档） | 315,667 / 89 / 17.0 分钟 |

## 二、核查员读数

核了 73 处：✓ 65、✗ 6、分不清 1、核不动 1。承重的三条：

- 正推腿贴的 `grep -n "^## " .claude/singlefs-ai-sop/skills/gate/SKILL.md` 输出，首行「6:# 准入门禁」不匹配那个模式，是编出来的（它自陈初稿行号凭印象写、后来逐条改过，这一处漏改）。
- 本地攻方第 1 问的前提「正推、材料员、核查员三个定义的输入里没有禁读清单」，对 `three-way-forward` 不成立（它第 20 行写着禁读清单）；对另外两个成立。
- 三条腿四处行号差 1，方向全一样（引用比实际小 1）。
- 正推腿对 `experiment-designer` 的 J6 建在旧版文件上：它交回之后主 agent 已按试跑观察改过那份定义。
- Opus 的 4 个模型脚本在核查员草稿目录里复跑：3 个逐字节一致，`hook-probes.sh` 15 个 `got=` 逐一一致（整份哈希不同是嵌的临时路径不同）。

## 三、逐格判决

| 格 | 出处 | 判决 | 主 agent 怎么坐实 | 改了哪里 |
|---|---|---|---|---|
| A1 执行类照共用约束用 Bash 写，写范围闸一次都看不见（K3 共用前提） | Opus | **采纳** | hook matcher 是 `Write\|Edit`（`.claude/settings.json`）；共用约束旧第 15、17 行让新建与改文件都走 Bash；实现员试跑 Bash 21 次、Write 1 次；核查员复跑 `copy-repo-sequences.sh` 一致 | `.claude/agent-common.md`「写」：有 Write / Edit 的手写改动用 Edit、新建用 Write，Bash 里只许定义点名的脚本写；只有 Read / Bash 的照旧 |
| A2 规格点名写范围外的文件时书记员没有停法 | Opus | **采纳** | 读 `kb-scribe` 定义 | `kb-scribe`：规格点名的文件不在写范围里就停下报告 |
| B1 `crash-verifier` 指纹看不见未跟踪文件改内容、改完暂存、改完提交 | Opus | **采纳** | 现查第 6 步命令只哈希未跟踪文件名单与 `git diff`；核查员复跑 `fingerprint-blindspots.sh` 逐字节一致 | `crash-verifier` 第 6 步：记 `git rev-parse HEAD`、`git diff HEAD`、未跟踪文件按内容哈希 |
| C1 relabel 改写 `research/**/*.rs` 而不改变异表锚点，33 号红 | Opus | **采纳** | 现查 9 张变异表共 31 处「已定项 / 未定项 N」；核查员复跑 `relabel-anchor-scan.sh` 逐字节一致 | `kb-scribe` 第 3 步：relabel 改到 `.rs` 就加跑 33 号并把改到的实验源码列给主 agent；归属表 33 号加 `kb-scribe` |
| C2 变更史条目标题不在输入里 | Opus | **采纳** | 读 `lib-history-brief.py` 第 59–65 行、`lib-item-ref-status.py` 第 62–63 行（核查员核过引用） | `kb-scribe` 输入：标题整行由主 agent 给 |
| C3 49 号 `--write` 会给别的会话的条目补「（待补）」 | Opus | **采纳（条件打中）** | 核查员复跑一致 | `kb-scribe` 第 2 步：跑前跑后 diff，改到这一轮之外的条目就停 |
| C4 子 agent 取不到派发前的哈希 | Opus | **采纳（轻）** | 读定义 | `kb-scribe`：哈希记开工时与收尾时两次 |
| D1 定义没写 `mutate.sh` 的参数与 bin 名，写错时脚本报成「基线就是红的」 | Opus；`mutation-triage` 试跑同一问题 | **采纳** | 现查 `research/e7-index-bench/Cargo.toml`：显式 `[[bin]]` 名是连字符（例 `e67-device-subset`）；`mutation-triage` 试跑撞到退出码 6 | `experiment-runner` 第 3 步：写明参数、在仓根下跑、bin 名以 Cargo.toml 为准 |
| D2 重跑已有实验要写的重跑登记与实验变更史不在写范围，闸会拒 | Opus | **采纳** | 现查 `research/prompts/` 下 6 份 `e*-r*-prereg.md`；写范围表里没有 `experiments-history.md` | `experiment-runner` 写范围与 `.claude/hooks/agent-write-scope.tsv` 各加两条；实测 `e150-r2-prereg.md` 放行、`_e150-r2-body.md` 仍拒 |
| D3 执行员可以经「修订」少报一个量 | Opus（归格有保留） | **采纳** | E153 试跑里执行员确实把依赖读法甲的 A10 挪出了报告流 | `experiment-runner`：修订只许收严或补臂，少报交主 agent |
| E1 要改的文件在「别的会话在改」清单里时实现员没有停法；`crates/mutations.tsv` 又要写又被排除 | Opus | **采纳** | 读定义第 3 步与写范围 | `implementation-writer` 新第 5 步：清单里的文件停下交回；`mutations.tsv` 只追加整行 |
| G1 没有定义文件的 `agent_type`（改名或删掉之后续做的旧实例）写哪都放行 | Opus | **不改，挂起** | 核查员复跑 H15 `got=0` | 条件弱；按内置 agent 名单拒掉其余，会误拦插件带来的 agent。记在这里，等真出现改名或删定义时再判 |
| J6 `experiment-designer` 13 条试跑观察大半没进定义 | 正推 | **已处理** | 核查员指出它交回之后定义已改；主 agent 现读当前文件，13 条里 1–8、12 已进定义，9（登记模板）没做，10、11 另行处理 | 无新改动 |
| 本地攻方第 3 问：复跑依赖绝对路径会被误判成对不上 | 本地攻方 s5 | **采纳** | 核查员复跑 `hook-probes.sh` 时正是嵌着的临时路径让整份哈希不同 | `three-way-verifier` 第 4 步：带路径的环境变量换到草稿目录，嵌临时路径的按字段比 |
| 本地攻方第 6 问：转述由同一条腿自译自核 | 本地攻方 s5（它自己答「结构上抓得住」） | **部分采纳** | 核查员这一轮核出本地攻方核对表一处行号差 1 | `three-way-verifier` 新第 5 步：本地腿核对表的「原文文件:行」逐条核 |
| 本地攻方第 1、2、4、5 问 | 本地攻方 s5 | 不采纳 | 第 1 问前提一半为假（核查员）；第 2 问说交回之后消息被忽略，与实测不符（SendMessage 能续做已交回的 agent，计划第十二节）；第 4 问是主 agent 自己的纪律；第 5 问与 A1 同族，Bash 写闸看不见已在共用约束写明 | 无 |
| 本地辩方第 2 条：bin 名写错应按「缺输入」停下，不该照脚本提示改 | 本地辩方 s2 | **不采纳，并列写出** | bin 名对不对是 `mutate.sh` 能机械核出、并打印唯一改法的事实，不是设计判断；照改并在报告写明，主 agent 可核 | 维持 `mutation-triage` 与 `experiment-runner` 的「以 Cargo.toml 为准」 |
| 本地辩方第 1、3、4 条：替 `ps` 停下、只报不修、`todo!` 停下辩护 | 本地辩方 s1、s2 | 与现定义相容 | 读样本 | 无 |
| 编造的命令输出、行号差 1 | 核查员 | **采纳（共用约束）** | 见第二节 | `.claude/agent-common.md`「报告」：行号用 `grep -n` 或 `awk` 现取，不从 `sed -n 'A,Bp'` 数偏移；贴的输出必须是这一次真跑的 |
| 报告不写文件（`sweep`、`experiment-runner`、核查员） | 试跑与核查员 | **采纳，改共用约束** | 核查员与实验设计员都指出会话给 subagent 的系统说明写着「不要写报告文件」，与共用约束冲突；加了「以派发提示为准」之后核查员仍按系统说明做 | `.claude/agent-common.md`「报告」：定义点名的产出文件照写，「报告」放进交回内容，报告路径由主 agent 存档 |

## 四、顺带发现（不归 agent 定义，交用户）

| 发现 | 出处 | 主 agent 核的结果 |
|---|---|---|
| `.claude/kb/checks-owed.md` 第 575 行汇总清单写「C157（树表容量在两处按不同条目宽算）（112 棵/层，…）」，没有日期，是两代之前的数 | `sweep` 试跑 | 现查属实（第 575 行是一整段长汇总，截断输出看不到） |
| `.claude/kb/checks-owed.md` 第 150、159 行 C148、C157 的判别力自证写「条目宽改回 67 回到 243」，按今天的分母 16253 是 242 | `experiment-designer` 试跑 | `16253/67 = 242`、`16284/67 = 243`，属实 |
| `.claude/scripts/fetch-deps.sh` 第 10 行 source 的 `.claude/scripts/lib.sh` 不存在 | `crash-verifier` 试跑 | 现查属实 |
| `relabel-item.py` 按 22 号的要求改写了变更史里过去时条目的标签，`decisions-history/2026-09.md` 第 217 行改完会说 2026-09-13 那天 D22 第 7 条是未定 | Opus C1 附带 | 核查员复跑一致；没在真仓上跑 |
| `quote-kb.py` 的 `@标题` 取法遇到标题里带冒号会切错，报错给的下一步误导 | 材料员试跑 | 主 agent 复现命令写错了参数形式，**没复现** |
| `replace-batch.py --help` 抛 traceback；`quote-kb.py` 取单行要写 `文件:960-960` | `experiment-designer` 试跑 | 没核 |
| 实验设计员占号写完登记、实验页建起来之前，门禁 86 号全仓红 | `experiment-designer` 试跑 | 副本里实跑 86 号判红属实；定义里已写明，由主 agent 尽快接派执行员 |

## 五、没做什么

- 单轮，不是三轮多数判；上表所有改法被攻过零轮。
- 没有给每个定义做第二轮对抗，也没有再派一次试跑去验证这些改法。
- `gate-triage` 的试跑与这一轮同时在跑，它的读数不在这份判决里。
- G1 挂起；登记模板（`experiment-designer` 观察 9）没做。
