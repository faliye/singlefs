# no-task-numbers-governance-r3：攻方腿报告（2026-11-19）

轮名 no-task-numbers-governance-r3，治理文件轮第三轮（最后一轮）。背景材料 `research/prompts/_no-task-numbers-governance-r3-body.md`，攻击面是正文「分工」表攻方那一行，不重复「已攻过的角度」。被判文件在主工作区现读；`sha256sum -c research/prompts/no-task-numbers-governance-r3-snapshot/judged-sha256.txt` 只有 `.claude/hooks/write-guard.sh` 对不上（开工之后被改过，工作区相对暂存区多了 session-descriptions 的自证格与会话号出路的新文字，判号那一段 `library_hits_before_and_after` 没动），其余 15 份与快照相同。

报告里不写任务号：探针里的号一律写成 `<号>`，量的时候用的是一个假号（五位数字），原样在模型 `tools/cases.py` 里（`.py` 不判这一形态）与草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus/live-contents.txt`。

## 复跑

模型目录 `research/prompts/no-task-numbers-governance-r3-opus-model/`。复跑命令（交主 agent，见末尾「要主 agent 跑的命令」）：

```
task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r3-opus-model
```

每个文件的 sha256：

```
490aff7c175d274283ed9ea8df931584c7167d56ceefac1bf5873c00e50b9dad  tools/cases.py
740d0107ad5fbe9dd55fea2bfb47a0cf0e25a7c61829a13b6635a036fa64cbfb  tools/judged-sha256.txt
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
e04f27b0aae49e14f17d9e201015b3daffff7f588b6faadf595b6a19cba5cb4e  tools/rerun.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：先把 `tools/judged-sha256.txt` 里七份被判文件（判据库、写闸、检出钩子、派发闸、doc-text、`scripts/kb/lib_notation_check.py`、`scripts/kb/replace-once.py`）与 `--repo` 下的比，对不上的逐份打 `name=judged_changed`（不停），再起 `tools/cases.py <仓根> <运行目录>`。`cases.py` 34 格：25 格直接调判据库的 `new_violation_hits`（records/ 与判决路径的格，这条腿写不进那两处），9 格把 stdin JSON 喂给检出钩子与写闸（主 agent 身份、只判不执行，什么都不往仓里写）；每格打 `name=case label=… want=… got=… agree=…`，`name=case_summary cases=34 disagreements=0` 才算复跑与腿一致；之后打两行现存真行扫描 `name=scan_session_descriptions`、`name=scan_description_variants_without_named_word`（没有 want，明细落运行目录的 `scan-*.txt`）。外面那一层 `rerun-from-a-clean-work-directory.sh` 照范例写：只删工作根下名字以 `-run` 收尾的那一个运行目录，别的退 2。探针的内存与时间没量（推的：几十行正则与 9 次钩子子进程，几秒、百兆以内）。

「量的」分两种，都标在格里：一是我自己的 Bash 命令过检出钩子 ⑰ 的结局（⑰ 用判据库判字面内容，放行就真写进了模型目录 `live/`，拒了就什么都没写）；二是我在主工作区直接起 `scripts/kb/replace-once.py`（共用约束点名可直接跑）的结局。往 `records/` 与判决路径写的几条是故意让 ⑫ 拒的探针：⑫ 与 ⑰ 的拒绝在同一条拒绝说明里列全，⑰ 不列就是判据库放过；两处现查过都没有落下文件。`cases.py` 与写闸那几格没跑过，标「推的，待主 agent 跑」。

真写进仓的探针（`live/` 下 11 份 `.md`）量完已挪出仓，现场在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus/live-moved-out-of-repo/`；模型目录此刻只有 `tools/`。

## 各格判定一览

| 关 | 格 | 判定 | 依据（一句） |
|---|---|---|---|
| 1 整份写前写后 | 1a ⑰ 认不出拆开写的号：printf 格式串加参数、赋过值的变量、`echo -e` 转义 | 成立（量的，三种都真写进了模型目录） | ⑰ 只拿字面词判，号由 shell 拼出来；records/ 下提交时 doc-text 兜，research/prompts/ 下没人兜（2b） |
| 1 | 1b 相抵按行文：旧行已带别的违例，这一次写让它出围栏、离开批次列，不拒 | 成立（字面不一致；出围栏那一对量的，表头那一对推的待跑） | 共用约束写「照拒」，库按整行文相抵；不带进新的号文字 |
| 1 | 1c 判据行的近似写法漏拒：`mainagent(5b)`、`agent(cc)`、`（会话 4a，…）`、`截止批次为三`、`它的批三`、`任务编号 <号>`、`<号> 号任务` | 成立（量的，模型目录与 records/ 路径两处） | 用户撤回的 `agent(<短号>)` 写法照样写得进；现存记录里已有 `截止批次为三` 三行 |
| 1 | 1d 同一行几处要一次改全，有没有正常改法被拒又没有出路 | 不成立（找到的每一种都有出路） | 最接近的两处：搬家改名（1e）、`截止批次`（main-agent.md 要写，数字写法被拒，出路是写那一批做什么） |
| 1 | 1e 门禁新格对搬家、改名的文件基准取空串 | 成立（误拒，推的待跑；doc-text.sh 文件头自认） | path-moves.md 要全仓改路径，碰到带旧号的行要连号一起改，与「旧记录里的号不回改」相抵 |
| 2 认列与豁免 | 2a 表头认列：`\\|`（转义的反斜杠加分隔符）让别的列的批号落进批次列 | 成立（量的，records/ 路径；写法少见） | 库按「前面不是反斜杠」切格，GFM 按转义成对切（GFM 这一句未核实） |
| 2 | 2b doc-text 豁免整个 `research/prompts/`：判决、岔路单、跑前登记、`abandoned-rounds.tsv` 都在里面 | 成立（⑰ 那一半量的，门禁那一半按代码推） | 与 1a 合起来，往判决里拆开写的号写闸不判、⑰ 不判、门禁不判 |
| 2 | 2c 路径归一、符号链接、大小写 | 不成立 | git 列的路径已归一；符号链接 git 只存链接文字；`Records/` 这类大小写变体按非豁免判，只会更严 |
| 2 | 2d 派发闸照判据库判 A/B 一行，单行结局与判决里是否相同 | 不成立（单行逐条同一个函数） | 派发闸把那一行原样交 `new_violation_hits`，路径同判决 |
| 2 | 2e 两臂分两行写：派发闸只判第一行，判决里第二行拒 | 成立（轻，推的待跑） | 两边判的不是同一段文字 |
| 2 | 2f 列表里的引用块 `- > A/B 读数：…` 两边都放过 | 成立（量的，判决路径） | 规则写「引用块里的不算」，库只认去掉空白后以 `>` 起头 |
| 2 | 2g 判据库读不到时派发闸放行记检出 | 不成立 | 与 ⑰ 同一处置；那一行写进判决时写闸读不到库照拒 |
| 3 判据行 | 3a `session-descriptions` 误拒现存正常行 | 不成立（量的：五类目录 103 行命中全是真指别的会话、或在反引号里、或是 `.json`） | `按块那一路`、`变异那一路` 后面不跟会话词，不中 |
| 3 | 3b 近似变体漏拒：`<做什么>那一路` 加回信、报、认领、转达；`<做什么>的会话`、`那个会话`；`那一路的工具员` | 成立（量的：⑰ 放过；现存 records 与 kb 里前一种 98 行、后一种 367 处） | 用户点名的原句去掉「会话」两个字就过闸 |
| 3 | 3c mainagent(A)、subagent(A) 放过 | 放过本身对；括号里换成两位短号同样放过，归 1c | 判据只认「那一路」那一种描述，不管括号里装什么 |

## 关 1：整份写前写后各判一遍的接法

### 1a ⑰ 认不出拆开写的号（成立，量的）

⑰ 的认法：「printf 交格式串与每个参数各占一行（格式重复几遍算不准，把参数逐个列进来，哪个参数里有都判得到）」（`.claude/hooks/bash-command-detector.sh:5174`）。号被格式串与参数拆开时，两边单看都没有整串；赋过值的变量、`echo -e` 的转义同理，⑰ 判的是 shell 展开之前的词。

我这条腿（three-way-attack）在主工作区的三条命令，⑰ 都放行，文件真写进了模型目录（之后挪出仓）：

| 写法 | 命令的样子（号写成 `<号>`） | 落盘的内容 |
|---|---|---|
| printf 格式串拆开 | `printf '（任务 %s）退 0\n' <号> >> …/live/printf-split.md` | `（任务 <号>）退 0` |
| 赋过值的变量 | `n=<号>; echo "（任务 $n）退 0" >> …/live/variable.md` | `（任务 <号>）退 0` |
| `echo -e` 转义空格 | `echo -e '批\x20105 在修' >> …/live/echo-escape.md` | `批 105 在修` |

对照：`printf '%s\n' '控制格（任务 <号>）退 0' >> …/live/control.md` 被 ⑰ 拒，拒绝说明点名第 2 行的任务号（号换成了占位）。

四句：不涉及几个臂，打中的是 ⑰ 的认法本身；⑰ 判的那一刻看得到整条命令，看不到 shell 展开后的字；满足的是共用约束「写闸、检出钩子 ⑰ 与 `scripts/kb/` 的写入工具照……判写进 `.md`、`.tsv`、`.txt` 的内容」那一句的反面；第二轮改法第 5 条（整份写前写后）在这几格上不起作用，它判的仍是展开前的字。

影响范围：`records/`、kb 下这样写进的，提交时门禁 doc-text 新格照判（推的，按 `.claude/gate.d/doc-text.sh` 的格代码，同一个 `new_violation_hits`）；`research/prompts/` 下没人兜，见 2b。`records/*.md` 的追加另有 ⑲ 不看内容一概拒，`>` 整份新建不拦。

### 1b 相抵按行文：已带别的违例的旧行出围栏、离开批次列不拒（成立，字面不一致）

规则原文：「这一次写让后面已有的行出了围栏、离开「批次」那一列的照拒」（`.claude/agent-common.md:45`）。库的 `new_violation_hits` 拿写后有违例的行按行文与写前有违例的行相抵：一行旧行写前就因别的号算违例（任务号在围栏里照判），写后行文一字不差，就被抵掉，哪怕它这一次多出了出围栏的批号。

量的（`scripts/kb/replace-once.py`，目标在模型目录，删开头那一行围栏 `~~~`）：

| 写前（围栏里那一行） | 结局 |
|---|---|
| `第 6 批 交回` | 退 8，拒绝说明点名第 2 行的批号 `第 6 批` |
| `第 6 批 交回（任务 <号>）`（写前那一份是我用 1a 的 printf 拆号写法造的「旧行」） | 退 0，「命中 1 次，已替换（改名换上新 inode）并回读确认」，`第 6 批` 出了围栏 |

表头那一对（`records/` 下把表头 `批次` 改成 `批`，那一行另一列本来就有批号）只能在库里判，进了 `cases.py` 两格「records:已有违例的行改表头离开批次列」「records:改表头对照」，推的，待主 agent 跑。

四句：打中的是相抵的键（只有行文），不分臂；库判那一刻两份整文都看得到，看得到那一行写前写后命中的东西不同；满足的是上面那句「照拒」的字面；第二轮改法第 5 条就是这一接法本身，在这一格上不中。分量：这一次写只是让旧行里已有的批号露出来，带不进新的号文字（新文字一定是新行文，抵不掉）；它是规范文字与判法不是同一件事的一处，不是写进新号的路。

自己的改法（只在我的模型上量过、被攻过零轮）：相抵的键从「行文」换成「行文加这一行的命中集合（判据行键与命中的串）」，写后那一行的命中集合比写前同文那一行多了，就算新违例。推的：出围栏那一对会翻成拒，出围栏对照不变。

### 1c 判据行的近似写法漏拒（成立，量的）

用户撤回调度记录里的短号写法，原话记在「撤回调度记录写 agent(<短号>) 那一件」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:37`）。共用约束要「一律照这个叫法，不换别的说法」（`.claude/agent-common.md:41`）。判据库对括号短号的四支都要求左括号前面不挨 ASCII 字母数字（`(?<![{ASCII_WORD}])`），`mainagent(`、`agent(` 后面跟什么都不判。

量的，⑰ 对我自己的命令：下面每一行写进模型目录都放行；同样的字写到 `records/zz-r3-opus-probe.md`（⑫ 拒，什么都没写），拒绝说明里只有写范围那一类、没有 ⑰ 那一类，即判据库在 `records/` 路径上也放过：

| 写法 | 结局 | 对照 |
|---|---|---|
| `mainagent(5b) 在修 …，subagent(c3) 交回` | 放过 | — |
| `agent(5b) 在修；agent(cc) 认领了；问 agent(4a) 回信` | 放过 | — |
| `交给里程碑三收尾那一轮（会话 4a，2026-11-19）` | 放过 | `交给收尾那一轮（4a）` 在 `records/` 路径上 ⑰ 拒，点名这一处括号短号 |
| `截止批次为三；它的批三在修；批次是十二` | 放过 | `截止批次：第 3 批。` 判据库拒（推的，第 N 批那一支） |
| `任务编号 <号> 退 0；<号> 号任务卡住；job <号> 跑完；TASK <号> 退 1` | 放过 | `（任务 <号>）` 拒（1a 的对照） |

现存真行（`git grep` 量的）：`截止批次为三` 三行，`records/2026-11-17-overwrite-composition-proofs.md:15`、`records/2026-11-18-range-intersection-composition.md:17`、`records/2026-11-19-unlink-prefix-proofs.md:17`；括号里会话词在短号前的一行：「这一轮剩下的活与验证全部交给里程碑三收尾那一轮（会话 4a，记录」（`records/2026-10-28-崩溃轻型提速.md:151`）。`截止批次` 是 `.claude/main-agent.md` 第 1 条要写进调度记录的：「开工前写死这一轮的课题、出口、派发上限与截止批次」（`.claude/main-agent.md:24`）；数字写法被拒，写的人换成中文数字就过了。

自己的改法（只在我的模型上量过、被攻过零轮）：括号短号那四支的左边界放过 `mainagent`、`subagent`、`agent` 这几个词，括号里只放过一个大写字母（`mainagent(A)`）；`会话` 在括号里、紧挨在短号前面的算挨着会话词；批号那一条加「批 / 批次」后面（可隔「为、是」与空白）跟中文数字；任务号加「任务编号」与「<号> 号任务」。推的：`截止批次` 的出路要在 `.claude/main-agent.md` 第 1 条写明写成「截止到哪一件做完」，不然改法只会把写的人推到下一个变体。

### 1d 同一行几处要一次改全，会不会把正常改记录的写法挡死（不成立）

找的办法：把规则文件里要求往 `records/`、kb、判决写号的句子与这一形态对一遍，再把「改一行就要连同一行的旧号一起改」放到正常的改记录动作上。每一处都找到了照着改得过去的出路，没有挡死：

| 正常动作 | 被拒的样子 | 出路 |
|---|---|---|
| 改依赖表一行的「输入齐没齐」，这一行「依赖」列写着别的批号 | 那一行成了新违例 | 「依赖」列改写成依赖哪一件（规则只留批次那一列） |
| 表头写成 `批`、`批号` 的依赖表（`records/` 下 35 张写 `批`、5 张写 `批号`，`git ls-files` 加 awk 量的）里改任何一行 | 批号不在表头是 `批次` 的列 | 先把表头改成 `批次`（表头那一行不带号，改完这一列整列放过） |
| 照 `.claude/main-agent.md` 第 1 条写截止批次 | `截止批次：第 4 批` 被拒 | 写「截止到哪一件做完」；`records/2026-11-18-前台超时与提交时大文件两道闸.md:10` 已这样写（表里截止批次那一格写的是写两道闸那一批） |
| 记用户原话，原话里带号 | 号照判（反引号里也判任务号） | 号换成占位并注明，本轮调度记录第 5 行就是这样写的 |
| 三方腿整行抄一行带旧号的 kb 或记录 | 腿的报告在 `research/prompts/` 下，写闸照判 | 改成转述并标「转述」，或只写指路 |
| 搬家、改名、术语改名之后全仓改路径 | 见 1e | 连旧号一起改 |

最接近挡死的是 1e：出路要改旧记录里的号，与本轮调度记录「已有记录里的号不回改（经过靠 git）」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:13`）相抵，但照着改得过去。

### 1e 门禁新格对搬家、改名的文件基准取空串（成立，误拒，推的待跑）

门禁格自己写进了管不到的：「整份搬家、改名之后基准里取不到原样的文件，里面已有的号照报」（`.claude/gate.d/doc-text.sh:194`）。格里 `base_text(path)` 按此刻的路径去基准里取，改了名的取不到、当空串，整份旧号都算这一次多出来的。共用约束那一句写的是「判相对基准多出来的行」（`.claude/agent-common.md:45`），搬家没有多出任何一行。

这不是罕见动作：`.claude/rules/path-moves.md` 要「全仓指向它的地方一次改成新路径」（`.claude/rules/path-moves.md:3`），`git log --diff-filter=R --since=2026-10-20 -- records .claude/kb` 量到 338 处改名。搬一份带旧号的记录或 kb 页，提交时整份红；搬一份脚本，全仓改路径时碰到的记录行里有旧号的，那几行也算新违例（照 `git grep` 粗量：`records/` 与 kb 里带反引号路径又带任务号、批号或括号短号形状的行 2973 行；单拿 `bash-command-detector.sh` 一个文件名，带号的记录与 kb 行 36 行；正则是近似的，偏宽）。

`cases.py` 两格「doc-text:改名后基准取空串」「doc-text:没改名对照」，推的，待主 agent 跑。

四句：不分臂；格判的那一刻看得到 `git diff -M` 的改名对（同一份 `scripts/gate/changed-paths.sh` 里的 `gate_added_lines` 就认 `.claude/gate.d/path-moves.tsv` 登记的搬迁，这一格没用它）；满足的是「相对基准多出来的行」的反面；第二轮改法第 7 条（加这一格）就是它，在这一格上照报。

自己的改法（被攻过零轮，推的）：`base_text` 先按 `git diff -M --name-status <基准>` 的改名对把新路径换回旧路径再取；术语改名、路径改写那一类改行的，照 1b 的命中集合比，行文变了而号没多的不报。

## 关 2：按表头认「批次」列、doc-text 豁免的边界、派发闸那一行

### 2a 表头认列：转义的反斜杠加分隔符（成立，量的，写法少见）

库按「前面不是反斜杠」认分隔符：「UNESCAPED_PIPE = re.compile(r'(?<!\\)\|')」（`.claude/gate.d/lib-forbidden-notations.py:498`），`\\|` 被当成转义的竖线；GFM 把 `\\` 认成转义的反斜杠，后面那个 `|` 照样分格（这一句是我对 GFM 表格的读法，未在本项目核实）。

量的（⑰ 对我写到 `records/zz-r3-opus-probe.md` 的命令，⑫ 拒、什么都没写）：表头 `| 批次 | 依赖 |`，行 `| x\\| 第 6 批 |`，拒绝说明里只有写范围一类，判据库放过；对照行 `| x | 第 6 批 |` 多出 ⑰ 那一类，点名第 3 行的批号。按 GFM，前一行的批号在「依赖」列。

改法（被攻过零轮，推的）：分隔符按前面连续反斜杠的奇偶认。分量轻：现存 `records/` 里这种写法我没找过，写的人要特意这样写。

### 2b doc-text 豁免整个 `research/prompts/`（成立）

格的代码：「FROZEN_EVIDENCE_PREFIX = "research/prompts/"」（`.claude/gate.d/doc-text.sh:2186`），前缀下的一律跳过、只报个数；格的文件头给判决另找了依据：「判决 *-main-verification.md 在写的那一刻另有写闸与写入工具判」（`.claude/gate.d/doc-text.sh:188`）。

用户选的是「现在换上，冻结证据豁免」。前缀罩住的不只是冻结证据：`git ls-files` 顶层、不是腿报告、不是判决、不以 `_` 起头的 `.md`、`.tsv`、`.txt` 有 62 份，里面跑前登记 15 份、岔路单 7 份（「实验每段交回，主 agent 逐行更新状态」（`.claude/rules/three-way-inference.md:66`））、`abandoned-rounds.tsv` 一份（叫停的轮逐行登记）。判决在写的那一刻也不是冻结证据。

与 1a 合起来是一条完整的路：往判决、岔路单或腿报告里用 printf 拆号、变量或 `echo -e` 写进的号，写闸不经手，⑰ 量的放过（1a 的三条就落在 `research/prompts/` 下），门禁按代码跳过。共用约束说的是「绕过它们写进仓的（`cp`、`mv`、命令的输出）由门禁 doc-text 的 `task-and-session-numbers` 格在提交时判相对基准多出来的行」（`.claude/agent-common.md:45`），没写 `research/prompts/` 不判；同一份共用约束又点名判决是照写任务号的地方之一（第 42 行），也就是在这一形态的射程里。

四句：不分臂；格判的那一刻路径看得到；满足的是共用约束第 45 行那一句的反面；第二轮改法第 7 条加的就是这一格，豁免是后来按用户弹窗加的，在这几格上不判。

自己的改法（被攻过零轮，推的）：豁免收窄到真冻结的那几类（`_*-body.md`、`_*-appendix.md`、`_*-background.md`、`*-snapshot/`、`*-model/`、已有判决的那一轮的腿报告），判决、岔路单、跑前登记、`abandoned-rounds.tsv` 照判新加的行；旧判决里的号在基准里，不报。另一条路是只改共用约束第 45 行，写明 `research/prompts/` 不在提交时的格里、靠写的人。

### 2c 路径归一、符号链接、大小写（不成立）

试过的形状：`research/prompts/../records/x.md` 这类写法（git 列的改动路径已归一，格拿到的是归一后的）；符号链接（格跳过 `os.path.islink`，git 只存链接文字，链接指向的仓里文件若改了另作为改动路径被判；仓里不放软链接另有门禁）；大小写变体 `Research/prompts/`、`Records/`（Linux 上是别的目录，不在豁免里，照判；`Records/` 下不认批次列、中文数字第几批与不挨会话词的括号短号，只会更宽地放过这三样，但这样的目录本身不该出现）；路径里带引号或换行的文件名（`scripts/gate/changed-paths.sh` 文件头自认 git 照样转义，格拿到转义串后 `os.path.isfile` 为假、不报也不计数，属于造出来的名字，不往下追）。

### 2d 派发闸照判据库判 A/B 一行：单行结局（不成立）

派发闸 `zero_round_reading_library_verdict` 把提示里每一行「A/B 读数：」原样（只把闭合围栏里的整行换成空行，那一行本身不改）交 `new_violation_hits(NOTATIONS_FORM, "", line, ZERO_ROUND_VERDICT_PATH)`，路径是 `research/prompts/x-main-verification.md`，与判决里判同一行的是同一个函数、同一类路径。我对着两边的认法（`LINE_START`、`ZERO_ROUND_READING_LINE`、`ZERO_ROUND_TASK_NUMBER` 三条两边逐字相同，写闸自证另核）试了：全角冒号与 ASCII 冒号、粗体、列表记号、行尾 `\r`、`#` 号、全角空格（派发闸认不出号而拒、库也拒）、`tasks`、大写 `Task`、号后面跟「个」，单行上两边结局都一样（推的，按两边代码逐条对）。

### 2e 两臂分两行写（成立，轻，推的待跑）

派发闸的读数行正则以行尾收（`.*?` 不跨行），提示里写成

```
A/B 读数：任务 <号>（有改法）
任务 <号>（没有改法）
```

派发闸只把第一行交判据库，放过；照抄进判决，第二行不是「A/B 读数：」一行，写闸拒。`cases.py` 两格「verdict:两臂分两行写」「dispatch:派发闸只判第一行」，推的，待主 agent 跑。拒绝说明的出路（「两臂各写一个「任务 <号>」」）照着写成一行就过，分量轻；它说明派发闸判的是「那一行」，不是「要抄进判决的那一段」。

### 2f 列表里的引用块 `- > A/B 读数：…`（成立，量的）

规则写引用块里的不算（`.claude/agent-common.md` 第 42 行「引用块里的不算」）；库的认法是「去掉行首空白后以「>」起头的引用块不算」（`.claude/gate.d/lib-forbidden-notations.py:93`），`- > ` 起头的列表项里的引用块不在内，`LINE_START`（`^[ \t>*_\-]*`）又认得它是读数行。

量的（⑰ 对我写到 `research/prompts/zz-r3-opus-probe-main-verification.md` 的命令，⑫ 拒、什么都没写）：`- > A/B 读数：任务 <号>` 拒绝说明里只有写范围一类，判据库放过；`> A/B 读数：任务 <号>` 多出 ⑰ 那一类、点名那个任务号。派发闸用同一个函数，推的同样放过。引用块不放的理由是引旧判决的读数会把旧号带进新文字，列表里引一段旧判决正是这样写。

改法（被攻过零轮，推的）：引用块的认法改成「去掉行首空白与列表记号（`-`、`*`、`+`、`数字.`、`数字)`）之后以 `>` 起头」，派发闸同一处一起改（两边逐字相同的约束照旧）。

### 2g 判据库读不到时派发闸放行记检出（不成立）

派发闸文件头写「判据库读不到时这一处没判，照「放行但必记检出」放行」（`.claude/hooks/runner-dispatch-guard.sh:192`）。⑰ 读不到判据库同样照常执行、记一条检出（⑰ 文件头），写闸读不到就拒、`scripts/kb/lib_notation_check.py` 读不到按拒。派发提示不进仓，那一行要进仓只能经判决，写进判决那一刻写闸照拒；放行这一步带不进号。

## 关 3：`session-descriptions` 判据行

### 3a 误拒现存正常行（不成立，量的）

用与判据行同形的 `git grep -E` 扫 `records`、`.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills`、`.claude/agent-common.md`、`.claude/main-agent.md`：命中 103 行，其中 `records/` 里 `.json` 6 行（这一形态不判 `.json`），两份规则文件 4 行都在反引号里，其余逐条看过上下文都是在指别的会话或子 agent（例：`E185 那一路的会话`、`统一任务启动器那一路的会话`、`prune-server 临时服务短套接字路径那一路的子 agent`）。说跑法的 `按块那一路`（`.claude/agent-common.md` 第 66 行）后面跟的是「与放量入口」，不中。判据库自己的扫描行在 `cases.py` 的 `name=scan_session_descriptions`，待主 agent 跑（我推的是 93 行左右：103 减 `.json` 6 行减反引号 4 行）。

### 3b 近似变体漏拒（成立，量的）

判据库写明「「那一路」后面不跟会话词的不判」（`.claude/gate.d/lib-forbidden-notations.py:90`），为的是放过 `按块那一路`。可用户点名的原句去掉「会话」两个字，`里程碑四收口的那一路回信` 就过闸；共用约束要的是「也不写 `<做什么>那一路的会话`、`那一路的 agent` 这类描述」（`.claude/agent-common.md:41`），「这类」不止带会话词的那几种。

量的，⑰（写到 `records/zz-r3-opus-probe.md`，⑫ 拒、只有写范围一类）：`里程碑四收口的那一路回信；GPU 垃圾回收那一路认领；task-server 那一路转达` 判据库放过；写进模型目录的 `管任务服务的会话认领了；改检出钩子那个会话回信；它那一路的工具员建的；兼容开关回扫会话报` 放过；对照 `改检出钩子那一路会话回信` 被 ⑰ 拒、点名这一处描述。

现存真行（`git grep` 量的，`records` 与 `.claude/kb`）：`那一路` 后面紧跟回信、来信、回话、答复、回复、认领、认了、认下、已认、通知、转达、已答应、报、群发、说、交回、在改、在修、在跑之一的 98 行（例：`GPU 垃圾回收那一路报`、`task-server 那一路转达`、`看门狗与钩子那一路通知`）；`<描述>的会话` 或 `<描述>那个会话` 后面跟这类动作词、去掉「别的、本、主、各、另一个」这类泛指之后的 367 处（例：`管任务服务的会话认领`、`跑崩溃对照的那个会话回话`、`做全量提交门禁的会话回话`）；`那一路的工具员` 一行：「46 随后更正是它那一路的工具员建的」（`records/2026-11-06-门禁判定全部入库.md:170`）。这些都是旧行，新库不挡它们，可同样的写法新写也照样过。

四句：不分臂；判据库判的那一刻只看得到这一行的字，`那一路回信` 与 `那一路会话回信` 指的是同一件事，判别它的是后面跟不跟会话动作，看得到；满足的是共用约束「这类描述」的字面；第二轮之后加的判据行就是这一条，在这几格上不中。

自己的改法（只在我的模型上量过、被攻过零轮）：判据行加两支：`那一路` 后面（可隔「的」与空白）紧跟上面那组会话动作词；`<描述>(的|那个)会话` 后面紧跟同一组动作词，描述不是泛指词表（别的、本、主、各、另一个、两个、几个、所有、新、旧、子、没有）里的。推的误拒面：`那一路` 加动作词的 98 行我抽看的前十五种都是指会话；`的会话` 那一支泛指词表要靠扫全仓再定，扫描明细在 `cases.py` 写的 `scan-description-variants.txt`。

### 3c mainagent(A)、subagent(A) 放过（放过本身对）

`mainagent(A) 在改 …`、`subagent(A) 交回了报告` 放过是规则要的。问题在判据不看括号里装的是什么：两位短号装进同一个壳照样放过，见 1c。

## 几个改法各修哪一格

全部只在我的模型上推过、被攻过零轮；没有一格是在副本上实现后跑过的。

| 改法 | 修哪一格 | 不碰哪几格 | 标 |
|---|---|---|---|
| ⑰ 判 printf 时把格式串按参数展开、`echo -e` 与 `$'…'` 先解转义、赋过字面值的变量代入参数 | 1a | 1b、1c（判据本身的漏） | 推的，待主 agent 跑 |
| doc-text 新格的豁免收窄到真冻结的几类，判决、岔路单、跑前登记、`abandoned-rounds.tsv` 照判新加的行；或只改共用约束第 45 行写明不判 | 2b | 1a 在 `records/` 下本来就有门禁兜 | 推的 |
| `new_violation_hits` 相抵的键加上这一行的命中集合 | 1b | 1e（改名是整份基准取空） | 推的 |
| 括号短号的左边界放过 `mainagent` / `subagent` / `agent` 而括号里只放一个大写字母；`（会话 <短号>` 算挨着会话词；「批 / 批次」后（可隔为、是）跟中文数字；「任务编号」「<号> 号任务」 | 1c、3c | 1a（展开前的字） | 推的 |
| `base_text` 按 `git diff -M` 的改名对取旧路径；改行而号没多的按命中集合比 | 1e | 2b | 推的 |
| 表格分隔符按前面连续反斜杠的奇偶认 | 2a | — | 推的 |
| 引用块认法去掉行首列表记号再看 `>`，派发闸同改 | 2f | 2e | 推的 |
| `那一路` 加会话动作词、`<描述>(的\|那个)会话` 加会话动作词两支 | 3b | 3c | 推的（误拒面靠 `git grep` 量的 98 行、367 处，未经判据库） |

## 要主 agent 跑的命令

1. 模型整份复跑（新 shell 里起，交任务服务）：
   - 命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r3-opus-model`
   - 在哪个目录起：仓根 `/home/fy5090/code/singlefs`（冻结副本参数也给仓根：被判文件在主工作区现读，漂移由 `judged-sha256.txt` 那一步逐份报）。
   - 跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号> --all` 里 `name=judged_check changed=0`（不为 0 时先看 `name=judged_changed` 点名的是哪份，那几格的结局要对着新版本重判）、`name=case_summary cases=34 disagreements=0`，与 `name=scan_session_descriptions …`、`name=scan_description_variants_without_named_word …` 两行的数（判决要引的话照这两行引）。有 `agree=no` 的格，看同一行的 `detail=`：写闸那四格若因别的道（不是号那一判）退 2，是我对写闸别的道没摸清，那一格算推不出。

## 没打中的形状

- 写闸 Edit / MultiEdit 的 `edited_content` 与 replace_all：照代码读，写后整份是施加之后的文件，与 `replace-once.py` 用的同一个 `new_violation_hits`；先写进围栏再拆围栏、replace_all 一处在围栏里一处在外，第二轮改法之后都拒（自证格在写闸里，没另造）。
- 同一条命令几处写同一份：⑰ 按解开符号链接之后的落点依次施加（`literal_landing_text`），`./`、绝对路径、符号链接几种拼法落到同一个键；没试出先后错位。
- 追加改变前面行的上下文：围栏与表格都只看前面的行，纯追加改不了已有行在不在围栏、属哪一列；`records/` 与 kb 里没有没收口的围栏（awk 按 0 到 3 格缩进的 ``` 与 ~~~ 粗数，零份）。
- `append-record.py --root` 指别处：落点要解开链接后落在 `<根>/records/` 下，指不回主工作区的记录。
- `session-descriptions` 误拒：五类目录 103 行，没有一行是说跑法而被当成指会话，见 3a。
- 派发闸与判据库对单行：见 2d 列的那几种写法。
- 门禁新格的路径归一、符号链接、大小写：见 2c。

取样范围：⑰ 量的 13 条命令（模型目录 9 条、records/ 路径 3 条、判决路径 2 条，其中对照 4 条），`replace-once.py` 2 条，`git grep` 扫的是 `records`、`.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills` 与两份共用文件。

顺带撞上的一处（不在这一轮射程，交主 agent 看）：我写这份报告时，一句里「会话」后面紧跟用括号括起来的本报告小节号，被 ⑰ 当成挨着会话词的括号短号拒了；改成「见」加小节号就过。三方腿报告里小节号与会话短号同形，这一类误拒在腿报告里还会有。

## 这条腿自己的限度

- 我手里没有 Edit / Write，写闸那一道一条都没亲手喂过；写闸的结局全按「写闸调的是同一个 `new_violation_hits`」推，`cases.py` 里四格写闸喂法待主 agent 跑。
- `records/` 与判决路径上的结局，是让 ⑫ 拒掉的命令里看 ⑰ 那一类在不在得来的：⑰ 不列即判据库放过。这依赖拒绝说明「一次列全」的写法；两处现查过都没有落下文件。
- `git grep` 的计数是近似正则（批号那一支会把「批 3 次」这类数量也数进去、「的会话」那一支的泛指词表是我拍的），只当量级，判决要引的数照 `cases.py` 的扫描行或另跑判据库。
- GFM 表格对 `\\|` 的切法是我对 GFM 的读法，没在本项目核实（2a）。
- `cases.py` 没跑过，34 格的 want 里标「推的」那些是按代码推的结局。
- 我造探针的「旧行」（1b 的写前那一份）用的是 1a 的绕法写进去的，不是仓里真有的一行；真有的同形旧行（围栏里带任务号又带批号、或表格里两列都带批号的行）我没去数。

## 没做什么

- 不判正推那几格（共用约束、沟通纪律与派发闸拒绝说明的新文字逐句对用户原话、第二轮改法八条落没落地）。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。没喂钩子：⑰ 的结局是它对我自己的命令自动判出来的。
- 没建仓副本与编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r3-opus/` 共 172K（`du -sh`），里面是扫描输出、`judged-sha256.txt`、`live-contents.txt` 与挪出仓的 `live-moved-out-of-repo/`（11 份探针 `.md`，里面有假号，不入库），没有要删的副本。
- 模型目录里写过的 `tools/__pycache__/`（`py_compile` 留下的）当场删了。
