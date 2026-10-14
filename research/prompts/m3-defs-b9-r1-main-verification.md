# 工具与治理批的零轮形态（第二题）：第一轮判决（2026-10-13）

<!-- doc-lint:not-numbers Z1 Z2 Z3 Z4 Z5 Z6 Z7 M1 M2 M3 M4 M5 M6 M7 N1 N2 N3 N5 F-Y3 P1 P2 P3 P4 P5 P6 P7 F-Z1 F-Z1c F-Z2a F-Z2b F-Z2c F-Z4 F-Z5 F-Z5b F-Z6 F-Z7 V8 -->

正文 `research/prompts/_m3-defs-b9-r1-body.md`；腿：正推 `research/prompts/m3-defs-b9-r1-sonnet-output.md`（sha256 85cd6ce5…3385，模型 `research/prompts/m3-defs-b9-r1-sonnet-model/`）、攻方 `research/prompts/m3-defs-b9-r1-opus-output.md`（sha256 1c1ade79…ae47，模型 `research/prompts/m3-defs-b9-r1-opus-model/`）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r1/repo/` 对 `research/prompts/m3-defs-b9-r1-snapshot/frozen-copy-sha256.txt` 34 份非 OK 0 行（写判决前又核一遍，同样 0 行）。
- kb 快照 `research/prompts/m3-defs-b9-r1-snapshot/kb-sha256.txt` 两份都对不上：`.claude/kb/term-renames.md`、`.claude/kb/experiments/171-三oracle合谋测试.md` 在快照之后被别的会话改过（term-renames 加了「层 0 → 崩溃点重放」「第一条流 / 第二条流…」两节与四行表）。两份腿报告按行号或标题引这两份文件 0 处（`grep` 两份报告里这两个文件名后接行号或 `@` 的写法零命中；攻方第 218 行只按名字提到 term-renames、第 251–253 行引的是 doc-text 格与 sweep-term 的代码），没有一处引文落在改动上，所以没有倒推原样。
- `cite-check.py <两份报告> --root <冻结副本> --background research/prompts/_m3-defs-b9-r1-background.md`：两份合计核 117 处、对上 117 处（正推 60/60）。
- 正推模型拷进草稿目录经 `run-with-memory-cap.sh 2G` 复跑：全 same。攻方模型同样复跑：17 份 16 same，`z7.txt` 两行 DIFF；查了原因：`tools/z7_exempt_repin.sh` 第 49 行先把输出按字截到 300（`trim 300`），`tools/rerun.sh` 第 115 行之后才把绝对草稿路径换成 `<WORK>`，复跑的草稿路径比腿的长，同一行少了末尾「动；命中复用库」几个字；两行的退出码（`script 退出=0`）与截断点之前的文字逐字相同，判定不变。
- 正推点的三条主 agent 在冻结副本拷贝里跑（`SINGLEFS_HEAVY_TESTS=user-request`）：code-tooling 样本档 44 例判对（含 long-run-header-exempt 的 base-green、repin-red、sha256-red），doc-process-records 样本档 22 例判对（含 abandoned-rounds-source-green / -red）；Z7-01 第三处小树：重钉没提交时退 1 点名「与基准那一版里的」sha256 不同，把重钉提交进基准之后同一条退 0、写「与基准那一版比没涨」。
- 攻方门禁格样本（`research/prompts/m3-defs-b9-r1-opus-model/gate-samples/make_trees.sh`，冻结副本拷贝）：abandoned-rounds 追加补记那棵在 b9 退 1、点名 m3-batch63-code-r1「这段原话命中 2 次」，同一棵在 r3 退 0，b9 拷贝本身退 0；heavy-thread-defaults 插一处、改一行两棵退 0 不点名，`var_from_registry.sh` 那棵退 1 点名第 5 行；doc-text 四格那棵退 1、输出里出现样本的（编的）私有值 3 处；long-run-header-exempt 基准有表那棵退 1、基准没表那棵退 0。都与攻方报告「要主 agent 跑的命令」一节的预计相同。
- 主 agent 另在冻结副本现查：`scripts/agents/dispatch-prompt.py` 第 43 行弄坏开关说明写「预计挂钟表少书记员」，第 90–91 行那一版 `LONG_RUN_TYPES` 少的是 clerk（正推 Z1-16）；`.claude/hooks/lib_abandoned_rounds.py` 的 `source_problem` 查轮名的对象是原话所在的整行（`cited_text`），不是〔〕里的原话（正推 Z4-02）；`ROUND_MATERIAL_FORMS` 只认 `<轮>-main-verification.md` 一种判决文件名（正推 Z4-09）；`.claude/hooks/runner-dispatch-guard.sh` 第 39 行与 `.claude/gate.d/doc-process-records.sh` 第 103 行仍写「不按 -r<数字> 形」（正推 Z4-10、Z4-11，攻方 Z4-丁）。
- 两条腿开工都读了主工作区的 `.claude/agent-common.md`（定义点名要读），与禁读清单相抵，两份报告都专节披露，判定取自冻结副本。下一轮起派发提示写「做法照主工作区的共用约束，被判字面只取冻结副本」。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| Z1 ⑮ ⑭ 认围栏、只有已判行、clerk | 站不住 | 波浪线代码块里露出一行三个反引号、列表项里没闭合的反引号围栏、HTML 注释里一行三个反引号，钩子都当围栏开头、一直遮到下一个真代码块，真「线程上限：16」不判、块里的示例被当声明（攻方 Z1-甲，r2 拒、r3 放：M1 带进来、N1 没修到，量过）；嵌套列表里缩进 4 格的围栏、引用块里的围栏、波浪线围栏里的示例照判拒（攻方 Z1-乙、正推 Z1-10：文件头「认围栏照 CommonMark」说过头）；缺行时钩子的出路叫补已判行，共用约束说只有已判行时子 agent 不起重活、要起就停下交回（攻方 Z1-丙，N3 带进来）；⑭ 围栏里不判、同一标签第二行拒两条规矩在定义与共用约束里没写（正推 Z1-07）；`dispatch-prompt.py` 第 43 行「书记员」应为 clerk（正推 Z1-16） | 改法 P1、P2 |
| Z2 heavy-thread-defaults | 站不住 | 先从登记位取进变量、再 `\|\| X=$default_threads`，照格的出路写仍被点名成写死的默认（攻方 Z2-乙，r2 不点名：M2 带进来，主 agent 跑树坐实） | 改法 P3；Z2-甲、Z2-丙另立账 |
| Z3 重型测试闸仓副本那一支 | 站不住 | 拒绝说明指到 `.claude/rules/implementation-workflow.md` 的那一节没写改过正文的自证在副本里怎么跑；照那一节读，登记为 light 的 `--selftest` 不算重型、随时可跑，钩子却在主 agent 不带前缀时拒（退 2）、带 `=commit` 或 `=user-request` 放（正推 Z3-05，喂法 C）；这一支的说明不写前缀，同一条 stderr 末尾的提交检查常量仍写两种前缀（正推 Z3-04） | 改法 P4 |
| Z4 叫停登记的出处 | 站不住（最脆弱） | 已收的叫停轮在出处那份记录里追加一行抄了原话的补记、照 `.claude/rules/path-moves.md` 跑术语改名、照 date-once 格的出路删重复日期，三种日常改动各一下就让原话不再恰好一次，⑥ 与门禁 abandoned-rounds 都翻回「开着」（攻方 Z4-甲乙丙，r3 收：N2 带进来；甲在真树上主 agent 跑格坐实）；整行被重复之后「多抄几个字」无路可走（正推 Z4-05）；两份文件头仍写「不按 -r<数字> 形」，与库相反（攻方 Z4-丁、正推 Z4-10、Z4-11，M4 带进来）；规则写〔〕里的原话要含轮名，库查的是那一行（正推 Z4-02）；「判决文件」只认 `-main-verification.md`（正推 Z4-09） | 改法 P5 |
| Z5 doc-text 打码 | 部分站不住 | 文件头写各格打印路径都经打码，date-once 的红行与 term-renames 转出的 sweep-term 输出原样打路径（攻方 Z5-甲，主 agent 跑样本坐实漏 3 处）；clock-times 的片段截在私有值中间、半截原样打出（攻方 Z5-乙）。漏值本身 r2、r3 同样中，「各格」那句话是 M5 写的 | 改法 P6 |
| Z6 ⑭ 多机一行 | 部分站不住 | 围栏那一形落在「多机：」上，真声明被遮、代码块里的规范句被当声明（攻方 Z6-甲，同 Z1-甲的根）；规范句下一行写「每台线程按此刻空闲现算」三版都放（攻方 Z6-乙、正推 Z6-12，改前也中）：线程上限有「提示里别处写的数一概不照」，多机没有对应的一句 | 改法 P1（甲）、P7（乙，文字） |
| Z7 豁免表成对比 | 部分站不住 | 基准那一版里没有这张表时不比，改被钉的入口并重钉照绿，`stage-owners.tsv` 那一行却写「按只缩不涨判红」（攻方 Z7-甲，主 agent 跑样本坐实；正推 Z7-01 同）；重钉提交进基准之后再跑不红（正推 Z7-01，主 agent 小树坐实），这是按基准比的表共有的性质，文字没写；被钉的 E171 入口 `exec` 没被钉的编排脚本，改编排照起（攻方 Z7-乙，改前也中，与上一题 V8 甲同根） | 改法 P7；Z7-乙另立账 |

## 改法（全部被攻过零轮；进批 10，改完开这一题第二轮）

| 改法 | 改什么 | 修的格 |
|---|---|---|
| P1 | `without_code_blocks` 改用 CommonMark 参照实现认代码块（本机 `python3-markdown-it` 3.0.0 的 commonmark 预设；反引号与波浪线、列表与引用块里的都认，只遮闭合了的围栏块，没闭合的照判）；载不进那个库时一行都不遮。定义与共用约束把「三个反引号围起来的代码块」改成「代码块（照 CommonMark）」，⑭ 写明围栏里的行不判、同一标签第二行拒（攻方 F-Z1，`research/prompts/m3-defs-b9-r1-opus-model/patches/runner-dispatch-guard.sh.diff`，攻方模型上量过；派发闸自证与上一题第二轮辩方 195 个标签没重跑） | Z1-甲乙、Z6-甲、正推 Z1-07、Z1-10 |
| P2 | 缺行时的出路写成「写「线程上限：N」再加已判行」，单独的已判行只留给确实不起重活（攻方 F-Z1c）；`dispatch-prompt.py` 第 43 行「书记员」改 clerk | Z1-丙、正推 Z1-16 |
| P3 | `\|\| X=$var` 只在同一份文件里 var 不是由登记位读法（`heavy_thread_default`、`default_heavy_threads`、`$SINGLEFS_THREAD_CAP`）赋值时算钉数（攻方 F-Z2b） | Z2-乙 |
| P4 | `.claude/rules/implementation-workflow.md`「checker 档只在提交时跑，harness 随时跑」那一节补一句：仓副本里正文改过的登记为 light 的自证按提交时的检查办，主 agent 在集成副本里带 `SINGLEFS_HEAVY_TESTS=commit` 跑、不弹窗；钩子这一支的说明与那一句同义 | Z3 |
| P5 | 原话命中几处而行号提示正落在其中一处，取那一行；原话找不到而行号提示那一行还有这一轮的名字，取那一行，往下照判名字、别的轮名与「」原话（攻方 F-Z4，`…-opus-model/patches/lib_abandoned_rounds.py.diff`，量过；弱点：漂移后提示那一行恰好是另一句带这一轮名字与「」的话会被当出处收，推的）；两份文件头的「不按 -r<数字> 形」改成与库同义；规则与共用约束写成「原话所在的那一行含这一轮的名字」、「判决文件」写成 `<轮>-main-verification.md` | Z4 |
| P6 | date-once 的红行与 term-renames 转出的 sweep-term 输出路径都经 `shown_path`（攻方 F-Z5b，推的）；clock-times、prime-marks、session-names 的片段两头落在私有值里就挪到值的边上、整值进片段再打码（攻方 F-Z5，`…-opus-model/patches/doc-text.sh.diff`，量过；doc-text 样本档没跑） | Z5 |
| P7 | 共用约束与九份定义补一句：多机只照规范句，提示里别处写的台数、线程取法一概不照（攻方 F-Z6）；`stage-owners.tsv` long-run-header-exempt 那一行与共用约束写明「基准那一版没有这张表时不比；重钉提交进基准之后不再红，按基准比的表都这样」 | Z6-乙、Z7-甲 |

另立一笔账（改前也中，不拿来否这批改动）：Z2-甲（排除表 17 行的锚串命中 2–5 条被判行，攻方 F-Z2a 推的，且罩不住改被钉那一行而锚串还在的一半）；Z2-丙（`${X:=8}` 不点名，攻方 F-Z2c 推的）；Z7-乙（入口钉不住它 exec 的编排，攻方 F-Z7 后一半推的，随 V8 甲与 E171 编排进代码欠账）；正推 Z4-08（轮名字符集只认小写、数字、连字符，仓里 306 个轮名都合）与 Z4-12（旧行名单里三个名字已无行仍可免出处再登记，「只缩不涨」只是注释）。上一题三轮判决的另立账照旧。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 更正（2026-10-14，第三轮正推 Q6 核出，主 agent 现查坐实）

- 核查记录 cite-check 那一条：写的命令是 `--root <冻结副本>`，而「两份合计核 117 处、对上 117 处」是以主工作区为根那一趟的数；照写的命令今天跑是「核了 117 处引文、42 处裸行号，对上 91 处，没判 68 处」。判决用到的引文当时另有人工核，判决方向不变。
- 核查记录模型复跑那一条：正推模型不是「全 same」，`~/.local/share/singlefs/work/m3-defs-b9-r1-main-out/rerun.log` 里 9 份 8 份逐行相同、`z6-script-header.tsv` 不同（差在截断位置，退出码列相同，判定不变）。
