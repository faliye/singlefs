# 工具与治理批的零轮形态（第二题）：第二轮判决（2026-10-14）

<!-- doc-lint:not-numbers S1 S2 S3 S4 S5 S6 S7 P1 P2 P3 P4 P5 P6 P7 U1 U2 U3 U4 Z1 Z2 Z4 Z5 Z6 Z7 F-S1a F-S1b F-S3a F-S5a F-S7a -->

正文 `research/prompts/_m3-defs-b9-r2-body.md`；腿：辩方 `research/prompts/m3-defs-b9-r2-sonnet-output.md`（sha256 fa4d45ee…，模型 `research/prompts/m3-defs-b9-r2-sonnet-model/`）、攻方 `research/prompts/m3-defs-b9-r2-opus-output.md`（sha256 5dd4a08d…，模型 `research/prompts/m3-defs-b9-r2-opus-model/`）。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/implementation-workflow.md`、`.claude/agents/clerk.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`、`.claude/agents/three-way-forward.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 辩方腿的会话在上一个会话结束时没走交回，报告与模型已完整落盘（报告各节齐、末行是 cite-check 结果，模型目录 `SHA256SUMS` 逐份核过），按交回办，没有另派接续的腿。
- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b9-r2/repo/` 对 `research/prompts/m3-defs-b9-r2-snapshot/frozen-copy-sha256.txt` 35 份非 OK 0 行；kb 快照 `kb-sha256.txt` 非 OK 0 行。
- `cite-check.py <两份报告> --root <主工作区> --background research/prompts/_m3-defs-b9-r2-background.md`：核 134 处引文、对上 134 处；17 处裸行号没有引文可比，判决用到的逐处人工核过：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/implementation-workflow.md` 那几处的句子在冻结副本里现查在；`out/` 下的行与复跑输出一同核。
- 模型复跑（拷进草稿目录，`run-with-memory-cap.sh 2G`）：攻方 9 份全 same；辩方 47 份与 `out/` 同名文件逐字节相同（`out/s1-real-prompts.txt` 按它写的随语料变、不在复跑里）。
- 门禁格与样本档主 agent 跑（两份冻结副本的拷贝，`SINGLEFS_HEAVY_TESTS=user-request`，脚本与日志在 `~/.local/share/singlefs/work/m3-defs-b9-r2-main-out/`）：
  - 三道样本档 code-tooling 46 例、doc-process-records 22 例、doc-text 27 例判对，含批 10 新加的四份与改过的两份样本。
  - heavy-thread-defaults：攻方 `htd-s3` 在 r2 点名 3 处（`conditional_literal.sh:4`、`stderr_silenced.sh:4`、`via_own_function.sh:5`），在 r1 点名 8 处；辩方 `s3-shell` 点名 15 行（13 份）；第一轮攻方 `htd-shell` 点名 4 处、不再点名 `var_from_registry.sh`；`htd-insert`、`htd-edit` 退 0（另立账 Z2-甲）。
  - abandoned-rounds：攻方 `abandoned-misread` 在 r2 退 0、成功句列「照行号提示取那一行照收」第 1594 行，在 r1 退 1「找不到〔m3-batch63-code-r1 用户叫停，不再打〕这段原话」；辩方四棵 r2 退 0、0、0、1，r1 四棵都退 1；第一轮攻方 `abandoned-after` 退 0。
  - doc-text：辩方 `s6/plain`、`s6/anchor` 与第一轮攻方 `z5` 退 1，日志里样本的（编的）私有值整串与半截 0 处。
  - long-run-header-exempt：基准有表那棵退 1、没表那棵退 0（判法没动）。
  - 两条腿预计的退出码与点名逐条相同。
- 主 agent 另在冻结副本现查：`.claude/main-agent.md` 里「一概不照」只有线程上限那一句，没有多机那一句（攻方 S7-乙）；登记表第 8 行 m3-batch63-code-r1 的出处是第 1595 行（攻方 S5-甲的误登记写第 1594 行）。
- 两条腿都报开工读了主工作区的共用约束与规则（定义点名要读），被判字面取自冻结副本；派发提示已写「做法照主工作区的共用约束，被判字面只取冻结副本」，这一条按定义读不算违禁读。

## 逐格判决

| 格 | 判 | 依据 | 去向 |
|---|---|---|---|
| S1 认代码块（P1） | 站不住 | 第一轮四形都修到（辩方逐例 b9 放→b10 拒、示例 b9 拒→b10 放）；但列表项里的围栏关在顶格、`<details>` 里带空行两形让 CommonMark 另开一块，遮掉块外真「线程上限：16」「多机：…现算」，r2 放、r1 拒（攻方 S1-甲，P1 带进来，模型复跑 same）；定义写「代码块（照 CommonMark）」，缩进代码块、没闭合的围栏、在列表项结束处结束的围栏钩子照判（攻方 S1-乙、辩方「带进来（文字过宽）」）；库载不进时一行都不遮，b9 认得的最普通的围栏示例也被拒（辩方，试改量过）；`<div>` 后紧跟的围栏示例 b9 放 b10 拒（辩方，误拒方向） | 改法 U1；S1-丙另立账 |
| S2 缺行出路（P2） | 站得住 | 两种出路文字修到、`dispatch-prompt.py` 改 clerk，15 个形态放拒两版逐行相同（辩方）；单独已判行而理由里写了数的那一形钩子放、子 agent 停下交回，b9 同 | 照留；那一形另立账 |
| S3 heavy-thread-defaults（P3） | 站不住 | `:=` 补数、被 source 的文件给数、`${SINGLEFS_THREAD_CAP:-$变量}`、登记值除以变量四形 r2 不点名、r1 点名（攻方 S3-甲，主 agent 跑格坐实）；同一变量一处登记位读法、另一处 `(( x = 8 ))`、`eval`、`read`、`printf -v`、`for x in 8`、`x+=0` 或被 source 的文件改写，七形 b9 点名、b10 不点名（辩方，跑格坐实 15 行里没有它们）；照出路写 `"$(heavy_thread_default 2>/dev/null)"` 仍被点名（攻方 S3-乙，第一轮 Z2-乙改后又中） | 改法 U2 |
| S4 仓副本自证（P4） | 站得住 | 规则与两处说明同义，钩子七例退出码两版相同，子 agent 自加前缀仍拒（辩方） | 照留 |
| S5 叫停登记出处（P5） | 站不住 | 第一轮三种日常改动都修到（辩方、主 agent 跑格坐实）；但出处〔〕写转述、行号指着同一轮另一句带名字与「」原话的话（真记录第 1594 行说的是写判决），r2 收、⑥ 放实现员，r1 不收（攻方 S5-甲，主 agent 跑格坐实）；改字加漂移、补记插在前面、原话瞎编三形 b9 不收 b10 收（辩方 S5 专问）；把原话抄成那一句的同类历史 b9 也收（辩方 `out/s5-line-number-read.txt`），P5 多出来的是不靠抄就无声改指 | 改法 U3；漂移加追加那一形（r1、r2 都不收）另立账 |
| S6 doc-text 打码（P6） | 站得住 | 两形修到，三格片段两头都挪，term-renames 按行豁免对不上时打出行内锚那第三条通道顺带堵上（辩方，主 agent 跑样本私有值 0 处）；攻方在 P6 范围内没打中 | 照留；两个私有值首尾重叠、主机名换大小写、doc-text 之外 16 道门禁不打码（攻方，改前也中）另立账 |
| S7 多机一句、豁免表两句（P7） | 部分站不住 | 豁免表两句与判法一致（两腿）；`.claude/main-agent.md` 没有多机那一句（攻方 S7-乙，主 agent grep 坐实）；用户给逐台不同的线程数时，照普通正文写子 agent「一概不照」，写成「线程上限：16」加已判行两台都降到 16，没有一种写法跑得成（攻方 S7-甲，钩子两版同放、r1 定义没有「一概不照」，是 P7 的文字带进来的） | 改法 U4 |

## 改法（全部被攻过零轮；进批 11，改完开这一题第三轮，最终轮）

| 改法 | 改什么 | 修的格 |
|---|---|---|
| U1 | ⑮ ⑭ 的标签行，在 CommonMark 认法与宽的逐行认法（反引号、波浪线都认，去掉行首 `>` 与任意缩进，往后第一行同一种字符三个以上当关块）之间遮不遮不一致就拒，说明叫把围栏写规整（攻方 F-S1a，原型量过 `research/prompts/m3-defs-b9-r2-opus-model/out/s1-ambiguity-prototype.txt`；代价是四个反引号套三个反引号的写法由放变拒）；库载不进时退回逐行认法，不再一行都不遮（辩方试改 `research/prompts/m3-defs-b9-r2-sonnet-model/out/s1-fallback-trial.txt`）；定义、共用约束与主 agent 定义写「闭合了的围栏代码块（照 CommonMark 认）」（攻方 F-S1b、辩方） | S1 |
| U2 | 变量算「登记位取的」要同时满足：右边整条是登记位读法（`"$(heavy_thread_default)"`，可带 ` 2>/dev/null`；`"$(default_heavy_threads)"`；`"${SINGLEFS_THREAD_CAP}"`）；同一份文件里它没出现在 `${变量:-…}`、`${变量:=…}` 里，也没有别的赋值写法（`(( 变量 = … ))`、`let`、`declare -i`、`read`、`printf -v`、`for 变量 in`、`变量+=`、`eval`）；文件 source 了别的文件时不算（攻方 F-S3a 量过的一半加辩方七形） | S3 |
| U3 | 原话找不到时，只在〔〕与行号提示那一行去掉改名表里的新旧名与日期之后逐字相同，才取提示那一行；原话命中几处时只在提示那一行在命中里才取（攻方 F-S5a 前一种，推的） | S5 |
| U4 | ⑭ 加一行放行「多机已判：<理由>」，有它时子 agent 照提示里逐台写的参数，没有就只照规范句；定义、共用约束与 `.claude/main-agent.md` 写这一条，`.claude/main-agent.md` 补「子 agent 跑多机只照规范句，提示里别处写的台数、线程取法一概不照」（攻方 F-S7a 前一种，推的） | S7 |

另立一笔账（改前也中，不拿来否这批改动）：S1-丙（引用块关块漏 `>`、关块缩进 4 格或用制表符、开四关三，两版都放）；S2 单独已判行而理由写数那一形；S5 漂移加追加；S6 私有值首尾重叠、大小写、doc-text 之外的门禁不打码；辩方「判轻」那一行列的第一轮正推 Z1-04、Z4-01、Z6-07、Z7-01 其余两处、Z7-06（改前就有的文字缺口，危害是误拒或认错）。第一轮判决与上一题三轮判决的另立账照旧。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。
