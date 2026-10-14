# 工具与治理批的零轮形态（第二题）：第三轮（最终轮）判决（2026-10-14）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 S2 S4 S6 U1 U2 U3 U4 P1 P3 P4 Z4 L1 L2 L3 L4 L5 L6 L7 L8 L9 L10 F-Q1a F-Q2a F-Q4a F-Q4b Q6-06 Q6-14 Q6-19 Q6-23 Q6-24 Q6-27 Q6-07 Q6-43 Q6-47 -->

正文 `research/prompts/_m3-defs-b9-r3-body.md`；腿：正推 `research/prompts/m3-defs-b9-r3-sonnet-output.md`（sha256 c696ce90…，模型 `research/prompts/m3-defs-b9-r3-sonnet-model/`）、攻方 `research/prompts/m3-defs-b9-r3-opus-output.md`（sha256 50331456…，模型 `research/prompts/m3-defs-b9-r3-opus-model/`）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r3/repo/` 对 `research/prompts/m3-defs-b9-r3-snapshot/frozen-copy-sha256.txt` 35 份非 OK 0 行；kb 快照非 OK 0 行；两份模型目录 `SHA256SUMS` 逐份核过。
- `cite-check.py <报告> --root <主工作区> --background research/prompts/_m3-defs-b9-r3-background.md`：攻方 30 处引文对上 30 处；正推 13 处引文对上 13 处，250 处裸行号是它 Q6、Q5 两张表里的「文件:行」出处，没有引文可比，判决用到的逐行人工核（下面各条）。
- 模型复跑（拷进草稿目录，`run-with-memory-cap.sh 2G`）：攻方 10 份 same；正推 2 份 same。
- 攻方门禁格（三份冻结副本的拷贝，`SINGLEFS_HEAVY_TESTS=user-request`，日志在 `~/.local/share/singlefs/work/m3-defs-b9-r3-main-out/log/`）：`abandoned-paren-after-migrate` r3 退 1、点名 hx-paren-r1「找不到〔hx-paren-r1 用户叫停（2026-10-14）〕这段原话」，r2 退 0、成功句列「照行号提示取那一行照收」，r1 退 1；`abandoned-paren-before-migrate` 三版退 0；与预计逐条相同。
- 主 agent 现查正推 Q6 四处「对不上」：
  - Q6-06 坐实：第一轮判决核查记录写的命令（`--root <冻结副本>`）今天跑出「核了 117 处引文、42 处裸行号，对上 91 处，没判 68 处」；判决写的「117 / 117」是以主工作区为根那一趟的数（`records/2026-10-12-里程碑三工具与治理批.md` 当天那一行「cite-check 两份合计 117/117」）。
  - Q6-14 坐实：`~/.local/share/singlefs/work/m3-defs-b9-r1-main-out/rerun.log` 里 `z6-script-header.tsv`「与模型目录 out/ 里同名文件不同或 out/ 里没有」，其余逐行相同；判决写「全 same」是错的。
  - Q6-19 坐实：「46 / 22 / 27」是第二轮冻结副本（`after-b10-fixup`）的数，第二轮判决里那一句对；第三轮正文把它列进 Q6 时没写是哪一版，`after-b11` 的 code-tooling 样本是 47 份（`verify14` 日志与 q623b 复跑都是「47 个用例」）。
  - Q6-27 坐实：调度记录「批 11 加的样本一份都不在红项里」按记录写的命令算是 2 处（改过的 `abandoned-rounds-source-green` / `-red` 里批 10 之前就有的旧日期行）；新建的样本 0 处。
- Q6-23 主 agent 复跑（`~/.local/share/singlefs/work/m3-tooling-b2-integration/q623b/`）：`LIB_LONG_RUN_BREAK=exempt-path-unchecked` / `exempt-growth-unchecked` / `exempt-repin-unchecked` 直接设，在 `after-b10-fixup` 与 `after-b11` 上四条命令全退 0，与 verify12、verify14 判的一样（直接设库开关被门禁覆盖）；verify13 那一趟三项退非 0 不是开关起作用，`redany` 把输出丢了、原因核不动。另有一趟（`q623/`）把开关名写成带 `long-run-` 前缀，库报「认不出」退 1，那一趟作废。
- Q6-24 后半主 agent 在 2026-10-14 跑过、日志存进 `~/.local/share/singlefs/work/m3-defs-b9-r3-main-out/log/q6-24-after-b9-fixup-doc-text.log`：`after-b9-fixup` 工作树上 clock-times「15 行……在 6 份文件里」、term-renames「豁免登记表里有 10 条指不到文件」，与 `after-b10` 之后的数相同。
- 正推 Q5 漏列的 L4、L5、L6 主 agent 读了它的复现输出（`research/prompts/m3-defs-b9-r3-sonnet-model/out/q5-repro.txt` 段 Q5-4、Q5-5、Q5-6），模型复跑 same：L4「B1 主 agent 带 =commit，副本正文被改成起 cargo test（闸不读正文） 退出码=0」r1、r3 同；L5 有改名表时「drift+reword」r1、r2、r3 都不收；L6 挪位置后 r1、r3 都退 1、点名为涨。
- 两条腿都报开工读了主工作区的共用约束与规则（定义点名要读），被判字面取自冻结副本。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| Q1 S2 缺行出路 | 站得住 | 攻方没给出违反用户定案的可达历史；Q1-甲（共用约束第 61 行「同样设成不超过 N」，子 agent 机器忙时照字面降）r1 同、推的 | Q1-甲另立账，改法 F-Q1a 进交用户表 |
| Q2 S4 仓副本自证 | 部分站不住 | 攻方九例 r3 与 r1 逐例相同、没打中；正推 L4：P4 叫主 agent 在集成副本里带 `=commit` 跑改过正文的自证、不弹窗，而钩子不读正文，副本里自证正文被改成起重活时照放（r1 钩子同放，P4 的文字让这条路成了照规则做） | 改法 F-Q2a 进交用户表 |
| Q3 S6 doc-text 打码 | 站得住 | 攻方八种新形状没有整值或半截漏出；正推 L10 三处没经打码的输出不带仓内值 | L10 另立账 |
| Q4 第二轮辩方判「修到」的条目 | 部分站不住 | Z1 围栏四形、Z6-甲、Z1-乙、Z4 三种日常改动原形、Z5 两形、Z7 文字还修着（攻方量过）；Q4-甲：日期写在全角括号里的叫停行，照 date-once 出路删日期连括号一起删，r3 不收、r2 收、r1 不收——是 U3（批 11，零轮）把 P5 修到的第一轮 Z4-丙 带回来；Q4-乙同根更弱 | 进交用户表 U3 那一行（F-Q4a 量过、F-Q4b 推的） |
| Q5 清单完整性 | 站不住（漏列） | 正推列 52 条：38 在、2 部分在、12 不在，真漏列 L1–L10 | 逐条补进另立账或交用户表 |
| Q6 数字能不能核 | 站不住（四处对不上） | Q6-06、Q6-14、Q6-19、Q6-27 坐实；Q6-07、Q6-23、Q6-43 口径缺；Q6-47 复核不了 | 第一轮判决与调度记录各追加更正（不改原文）；Q6-19 是冻结正文，在这里写明 |

## Q5 漏列的去向

| 线 | 是什么 | 哪一版带进来 | 去向 |
|---|---|---|---|
| L1 | 回车换行的派发提示：`--check-prompt` 放、钩子 stdin 入口拒 | r1、r2 同 | 另立账 |
| L2 | 经自己的小函数取登记值仍被点名 | r1 同 | 另立账 |
| L3 | S3 判法三个语义洞（函数没调用、赋值在使用之后、子 shell 里赋）与一个误报 | 语义洞 P3 带进来 | 交用户表 U2 那一行一并标 |
| L4 | 仓副本自证正文被改成起重活，主 agent 带 `=commit` 照放 | 钩子 r1 同；P4 的文字 | 交用户表 F-Q2a |
| L5 | 前面补一句再按改名表换词，三版都不收 | r1、r2 同 | 另立账（与第二轮判决的「漂移加追加」并列） |
| L6 | 豁免表被钉脚本挪位置、sha256 不变判涨，门禁说明却写「路径搬了就改」 | r1 同 | 另立账 |
| L7 | `python3-markdown-it` 是没声明的依赖 | P1 带进来 | 交用户表 U1 那一行一并标 |
| L8 | 缺行出路钩子放、按定义读出来相反的三形 | r1 同 | 另立账（并第二轮 S2 那一形） |
| L9 | P1 的收益没在真提示上量过（工作根下 54 份提示含围栏行 0 份） | 观察 | 记一行，不是错 |
| L10 | doc-text 三处输出没经打码（认不出的选项、进不去项目根、读不到禁读表），都不带仓内值 | r1 同 | 另立账 |

攻方顺手记的（不判）：`scripts/crash/crash-amplification-two-hosts.sh` 文件头与自证格还按现算期望第二台线程，库已改成取登记值（推的，没跑自证）；记进下一批工具活的清单。

## 交用户表（全部被攻过零轮）

| 形态 | 是什么 | 被攻过 | 已知问题 |
|---|---|---|---|
| U1 | ⑮ ⑭ 标签行两种认法遮不遮不一致就拒；库载不进退回宽认法；定义写「闭合了的围栏代码块（照 CommonMark 认）」 | 零轮（批 11 已落进集成副本 `after-b11`） | 四个反引号或波浪线套三个的写法由放变拒；依赖 `python3-markdown-it` 没声明（L7）；收益没在真提示上量过（L9） |
| U2 | heavy-thread-defaults 变量来源四条件 | 零轮（同上） | 三个语义洞与一个误报（L3）；经自己的小函数取登记值仍误红（L2） |
| U3 | 叫停登记出处：找不到时〔〕与提示行去掉改名表新旧名、日期、空白与逗号逐字对上才取 | 零轮（同上） | Q4-甲（括号里的日期连括号删）把第一轮 Z4-丙带回来；改法 F-Q4a：去掉日期后再去掉剩下的空「（）」（攻方量过：Q4-甲修到、第一轮三种照收、第二轮 S5-甲照不收、碰不到 Q4-乙）；F-Q4b：找不到时到登记那次提交时的记录里找（推的） |
| U4 | ⑭ 认「多机已判：<理由>」，有它时子 agent 照提示里逐台写的参数 | 零轮（同上） | 这一轮没人攻 |
| F-Q2a | `.claude/rules/implementation-workflow.md` 与钩子说明补：主 agent 跑副本里改过正文的自证之前看一眼那份自证的 diff，正文起编译、测试或别的重活的照重型办（提交时跑或弹窗） | 零轮（主 agent 推的） | 钩子不读正文，靠主 agent 看 |
| F-Q1a | 共用约束第 61 行「同样设成不超过 N」改成「设成 N」 | 零轮（攻方推的） | 改前也中 |

另立一笔账（改前也中，不拿来否这一题的形态）：Q1-甲；L1、L2、L5、L6、L8、L10；第二轮判决与第一轮判决的另立账照旧；上一题三轮的另立账（正推第二张表列了 14 行出处，没核现在是否仍中）照旧。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 用户定案（2026-10-14）

弹窗选「采纳并补三条后合入」：U1–U4 照留；F-Q4a、F-Q2a、F-Q1a 由一个 tooling-writer 在集成副本里改完、主 agent 复核，不再开三方；之后等别的会话提交，经 hook-swap 合入主工作区。
