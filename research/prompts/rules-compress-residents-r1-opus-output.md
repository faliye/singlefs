# rules-compress-residents-r1 攻方腿报告

日期：2026-11-16。立场：假设「压缩后一条没丢、没有二义、没有悄悄改做法」是错的，找反例。没写模型，没有复跑命令；两份只读核对脚本在草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r1-opus/`（`pointer-check.py`、`span-check.py`，输出 `pointer-check.txt`、`span-check.txt`），只读文件、不起项目代码。

合稿树路径前缀下面简写成「合稿」= `research/prompts/rules-compress-r1-merged/dot-claude/`，引文照写全路径。行号都是现查的。

## 各格判定一览

| 格 | 判定 | 打中几处 | 最重的一处 |
|---|---|---|---|
| 一 丢失 | 打中（都是「指针指到的那一节没有那句话」，原话在合稿树别处找得到） | 2 | 提交技能第 2 步把各片与合并判什么指到 verification.md，那一节没有 |
| 二 二义 | 打中 | 2 | 「新增的测试先证明会红」的变异行被写死成 crates/mutations.tsv |
| 三 常驻够不够 | 没打中（只核了八份被并掉的小文件的接收文件是不是常驻） | 0 | — |
| 四 子 agent 读得到 | 没打中（机械核了合稿树里全部「文件加节名」指针，另抽了共用约束与攻方定义） | 0 | — |
| 五 悄悄改了做法 | 没找到没登记的做法变化；找到 2 处登了但落得不对或种类标错 | 2 | fs-design 写「规则里还没有对应的节」，合稿树自己的 verification-reference.md 里有 |

打中的都只抽了一次样，照 `.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算` 先挂起，由主 agent 现查。

## 格一：指针指到的地方没有那句话

### 一-1 提交技能第 2 步：各片与合并判什么、三处怎么领卡，指到的节里没有

- 原件把这一段写在技能里：`.claude/skills/crash-test/commit-stages.md:17`：「合并那一步照常判九格，crash-case-replay 格核账、不现判」。
- 合稿删成指针：`research/prompts/rules-compress-r1-merged/dot-claude/skills/crash-test/commit-stages.md:24`：「各片与合并各判什么、要卡的三处怎么按需领卡，照」，后面点名的是 verification.md 的「一个门禁：两种跑法」与门禁文件头。
- 找过的地方：合稿 verification.md「一个门禁：两种跑法」一节是第 26 到 46 行，命令 `awk 'NR>=26 && NR<=46' <合稿>/rules/verification.md | grep -cE 'lkmm|litmus|九格|核账|领卡'` 输出 `0`（量的）。那两句实际在按需读的参考件里：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:18`：「合并那一步照常判九格、crash-case-replay 格核账不现判」，`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:21`：「要上卡的三处」。
- 为什么算：判据格一「落点只剩指针而指到的地方没有那句话」。verification.md 那一节末尾只有一句转指：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:44`：「起一道门禁或入口、照它的写法单跑一趟、写范围声明、核它的判定单位、缩域与复用自证之前，先读」，要再跳一次才到。
- 照合稿做会怎样：主 agent 提交时照技能读 verification.md 那一节，读不到合并那一步核账、不现判与领卡被拒判红的细则；门禁文件头另有一份（技能同一句也点了文件头「分片」「显卡与清场」），所以不至于做错，只是规则这一侧的指针落空。改法（只在这条腿读过、被攻过零轮）：指针改成 verification-reference.md「门禁 crash-point-replay 怎么跑」。

### 一-2 fs-design「每份 litmus 一条判定」指到的节里没有

- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「工具是门禁 lkmm（每份 litmus 一条判定」，括号里点名 verification.md「一个门禁：两种跑法」。
- 找过的地方：同上那条 `awk … | grep -c` 输出 `0`，那一节没有 lkmm 也没有 litmus。原话在：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:42`：「lkmm 每份 litmus」。
- 照合稿做会怎样：要核「lkmm 的判定单位是什么」的人照指针读不到；与一-1 同一个病根（verification.md 拆出参考件之后，指到它的指针没有跟着改到参考件）。这一类我只核了这一组文件里的两处，别的组指到 verification.md「一个门禁：两种跑法」的指针没逐个核。

## 格二：二义

### 二-1 「新增的测试必须先证明它会红」的变异行被写死成一份文件

- 原件：`.claude/rules/show-me-test-by-review.md:12`：「有变异 harness 的仓要再进一步」，同一行说的是「存进仓里的变异清单」，没点名哪一份。
- 用户定案只说两样都要：`records/2026-11-16-三方豁免取消与存量补轮.md:121`：「变异行之外 commit message 也写改坏了哪一行、哪条断言红」。
- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/rules/show-me-test-by-review.md:7`：「做成存进仓里的变异行…交给门禁反复复跑」，省略处是括号里的 crates/mutations.tsv。
- 两种读法：甲，每条新增的测试（节名管的是全部新增测试，实验装置的测试、脚本与钩子的自证也在内）都要在 crates/mutations.tsv 留一行；乙，这一句只管 `crates/` 的测试，别处的测试只剩 commit message 一样。两种做法不同：甲对装置测试做不到（它们的变异表在另一处，合稿自己写着两处：`research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:11`：「scripts/mutation/tables/ 与 crates/mutations.tsv 的静态几样」）；乙把原件对装置测试「做成存进仓里的变异清单」的要求收没了。
- 归哪一格：登在矛盾总表「用户定-4」（`research/prompts/rules-compress-r1-merged/contradictions.md:106`），定案本身不攻；攻的是落法多写了一个文件名，把射程读窄或读成做不到。改法（零轮）：写成「存进仓里的变异表（`crates/` 的测试是 crates/mutations.tsv，实验装置的是 scripts/mutation/tables/ 下那一份）」。

### 二-2 沟通纪律的射程例外指到上游一个不存在的节名

- 原件写在本地：`.claude/rules/communication-discipline-by-review.md:16`：「写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」
- 合稿删成指针：`research/prompts/rules-compress-r1-merged/dot-claude/rules/writing-discipline-by-review.md:56`：「射程的例外照它的」，后面的节名是「射程与例外」。
- 上游 `.claude/singlefs-ai-sop/rules/communication-discipline.md` 的标题只有「不写客套」「转述别处的做法，带出处或原文」「收工前核一遍别的会话发来的消息」「找别的会话，只找正在跑的」「会话标题开头写编号」五节（`grep -n '^#'` 量的），没有「射程与例外」；那句话在开头段：`.claude/singlefs-ai-sop/rules/communication-discipline.md:5`：「写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」
- 为什么算：内容没丢（上游逐字有），节名是原件本地那一节的名字、套到了上游头上；判据格二「指针指到的节是不是唯一」这里是零个。照合稿做：按节名找不到，要整份读上游才碰得到。改法（零轮）：写成「射程的例外照它开头一段」。

## 格五：登了，但落得不对或种类标错（没找到没登记的做法变化）

### 五-1 「并发交错由工具穷举判定」写「规则里还没有对应的节」，合稿树自己有

- 原件：`.claude/rules/exhaustive-interleaving.md:16`：「用哪个工具、怎么证明它的判定分得出差别、怎么把写下的模型绑到代码上，写在」，后面点名 verification.md。
- 矛盾总表登的处置（`research/prompts/rules-compress-r1-merged/contradictions.md:120`，主 agent 处置-6）是后两样「找不到对应的节，记欠账」，合稿照它写成：`research/prompts/rules-compress-r1-merged/dot-claude/rules/fs-design.md:112`：「规则里还没有对应的节，记在欠账表」。
- 反例在合稿树里：`research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:157`：「下每条 Never 有对照组、绑到代码，herd7 判定与声明相符」。对照组答的就是「怎么证明判定分得出差别」，绑到代码答的就是「怎么把写下的模型绑到代码上」。这一行在原件里也有，只是不在 verification.md：`.claude/rules/implementation-workflow.md:77`：「下每条 Never 有对照组、绑到代码，herd7 判定与声明相符」。做法的细则在 `.claude/scripts/lkmm.sh` 文件头「会失败的检查」第 3、4 条（每条 Never 有自己的去屏障对照组、每条 Never 绑到代码的锚点）。
- 第二半也对不上：合稿说「记在欠账表」，`grep -cE 'lkmm|对照组' .claude/kb/checks-owed.md` 命中 6 行（量的：C8、C16、C161、C836、C577、C525），我只看了这 6 行的编号与标题两列，标题没有一行是「交错模型怎么绑到代码、判定怎么证明分得出」这笔账；总表自己也写着没登：`research/prompts/rules-compress-r1-merged/contradictions.md:129`：「欠账号没写，要主 agent 登了再补」。这一句只核了 checks-owed.md 一份文件里带 lkmm 或「对照组」的行，别的写法没搜。
- 为什么算：原件的错只是指错了文件（内容在 implementation-workflow.md 的门禁表与脚本文件头），合稿把「指错文件」处置成「没有这一节、记欠账」，等于把一条有落点的规则说成没有；照合稿做，读的人去欠账表找不到，也不会去读 verification-reference.md 那一行与 lkmm.sh 文件头。改法（零轮）：那一句改成指 verification-reference.md「门禁管哪一半」lkmm 那一行与 `.claude/scripts/lkmm.sh` 文件头，「记在欠账表」删掉；总表处置-6 的种类从「只改措辞」改成「改指针」。

### 五-2 常驻-11 标「只改措辞」，实际收掉了一种许可

- 原件：`.claude/rules/command-safety-by-review.md:21`：「不杀进程、不停子 agent，只有调度的一方不响应时例外」，下一行点名谁可以终止：`.claude/rules/command-safety-by-review.md:22`：「报告的一方（人、agent 或检出的工具）可以终止它」。
- 合稿：`research/prompts/rules-compress-r1-merged/dot-claude/rules/command-safety-by-review.md:17`：「报告的人或 agent 可以终止它」，「检出的工具」没了，「只有调度的一方不响应时例外」也没了。
- 总表登了这一处（`research/prompts/rules-compress-r1-merged/contradictions.md:46`，常驻-11），种类写的是「只改措辞」，说明写「检出工具只报，终止归报告的人或 agent」。原件许检出工具在调度的一方不响应时自己终止，合稿不许：这是做法变了，不是措辞。依据（main-agent.md 的看门狗只报不停）我没有异议，攻的是种类标错：矛盾那一题的三方若只挑「改了判据」的行细看，这一行会漏。
- 照合稿做会怎样：以后写会自己停进程的检出工具，原件许、合稿不许。

## 用户定案落没落对（只核这一组文件里的四条）

| 定案 | 合稿落点 | 判定 |
|---|---|---|
| 用户定-3 并行度缺省取登记的线程数 | 合稿 command-safety-by-review.md 第 39 行；合稿树里 `grep -rn nproc` 只剩这一行里「不按 nproc 取」那一处（量的） | 落对 |
| 用户定-4 两样都要 | 合稿 show-me-test-by-review.md 第 7 行 | 两样都在；多写的文件名生出二义，见二-1 |
| 用户定-6 进程组信号一律不许 | 合稿 command-safety-by-review.md 第 4 到 12 行、共用约束第 67 行、工具员定义第 33 行；合稿树里 `grep -rnE '整组|进程组|killpg'` 没有剩下「对整组发信号」的写法（量的），攻方定义第 3c 步原来那一句也已不在 | 落对 |
| 用户定-8 留格式示例、删代码示例 | fs-design 两段 Rust 示意删了，判据句留着（合稿 fs-design.md 第 91 行） | 落对 |

## 没打中的范围

### 逐句对过的（原件每一行都在合稿树里找了落点）

- 八份被并掉的小文件：verify-before-claiming（落合稿 evidence-discipline.md 第 22 到 55 行）、artifact-line-pointer（同文件第 141 到 145 行）、exhaustive-interleaving（合稿 fs-design.md 第 103 到 114 行，打中见五-1、一-2）、service-stop-readback（合稿 command-safety-by-review.md 第 45 到 49 行，四条都在）、communication-discipline-by-review（合稿 writing-discipline-by-review.md 第 54 到 65 行，打中见二-2）、engineering-philosophy（合稿 machine-first.md 第 1 到 45 行）、design-doc-discipline-by-review（同文件第 97 到 102 行）、pushback-discipline-by-review（同文件第 104 到 132 行）。
- evidence-discipline、session-wrapup、command-safety-by-review、format-evolution、fs-design、show-me-test-by-review 六份，与 crash-test/commit-stages.md、gate/staged-gate.md 两份提交技能，原件与合稿并排逐行对过。
- 两份提交技能的步骤号（1、1b、2、2b ①②、2d、3 到 6）、每条命令的整条写法、退出码（77、78、1、0、2、4、5、6、75、143）与弹窗次数（7 加 1）都在；`span-check.py` 对这两份原件的反引号片段逐个在合稿树里找，commit-stages 112 个里只有 1 个找不到（占位写法 `<未跟踪文件要查的路径>` 改成了 `<路径>`），staged-gate 51 个全在（量的，输出在草稿目录 `span-check.txt`）。

### 删成上游指针、去上游逐条核过的

| 原件那一句 | 合稿的指针 | 上游有没有 |
|---|---|---|
| engineering-philosophy 第 33 到 34 行（名字不限长度、不用缩写、不用单字母、语义放类型先于名字先于注释） | 合稿 machine-first.md 第 28 行指上游 code-discipline.md | 有：上游第 22、24 行与「语义放在哪：注释 < 名字 < 类型」一节 |
| session-wrapup 第 65 行（历史条目撞号怎么判） | 合稿 session-wrapup.md 第 38 行指上游 kb-discipline「8.」 | 有：上游 kb-discipline.md 第 87 行 |
| show-me-test-by-review 第 21 到 22 行（踩坑先想怎么做成会红的一项） | 合稿第 12 行指上游 sop-first「推论」「边界」 | 有：上游 sop-first.md 第 7 到 13、40 到 49 行 |
| code-discipline-by-review 类型与注释两张表的「我们的写法」列（try_from、Default、builder、SAFETY、文档注释、魔法数字） | 合稿第 53 行与「注释与文档」一条指上游两节 | 有：上游 code-discipline.md 第 113 到 157 行逐条有；「别用 typedef」上游没有，合稿第 55 行自己留着 |
| pushback 第 8 行（警告记录的格式） | 合稿 machine-first.md 第 106 行指上游 pushback-discipline | 有：上游那一节标题逐字相同 |
| path-moves 的登记行生效、不许串、搬迁实据、门禁关不住的四样 | 合稿 path-moves.md 指 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」 | 有：文件头前 400 行里各关键词都命中（量的；只数了命中，没逐句对） |

### 只抽了的

- code-discipline-by-review、path-moves、timer-only-remind 与 remind、gate、halt、decide 四份技能：只看了 `span-check.py` 报找不到片段的那几行与对应的合稿行。
- changelog-format、directory-layout、implementation-first、long-run-split、mutation-sampling、test-discipline、test-script-discipline、machine-first 原有各节、writing-discipline-by-review 的长度一节、crash-test/SKILL.md、queue-health、trilingual：只有机械两项（反引号片段在不在合稿树、文件加节名的指针解不解得到），正文没逐句读。这十二份上「没打中」不成立，只能说「机械两项没报」。
- 机械核的口径：`pointer-check.py` 认反引号路径后面紧跟「节名」的写法，合稿树里这样的指针 367 处；报节名找不到的 25 处（`grep -c ^NOHEAD pointer-check.txt`，量的）我逐个看了，除二-2 那一类它认不出的写法（节名前隔着「它的」）之外，都是指到加粗条目名或表格行（共用约束的「子 agent 不交任务」、主 agent 派发表的行名），条目与行都在。

### 看过、判不算打中的

- 被取消的那一次运行「引的时候引那一行」（`.claude/rules/verify-before-claiming.md:48`：「引的时候引那一行」）合稿那一条没带，但同一节上一条已写出结局后引判定那一行，不另算丢。
- evidence-discipline 六问末格「先问它是哪个参数的函数，再写进正文」合稿只留前半，次序那半句没了；我判是说明不是判据。
- 提交技能合稿第 23 行还写「一节定的前缀」，而「跑之前定下来」一节只说不带某个前缀；原件就这样，不是压缩带来的。staged-gate 把「前缀照带」「同一个前缀」删了，两份技能处理不一致，做法没差。
- gate 技能的 light / heavy 两档、收尾第 4 条「分不清就跑 gate.sh --staged」、装置样本进程放进进程组，三处做法都变了，都登在总表（按需-M14、常驻-8、用户定-6）。

## 格三、格四：试过的形状

- 格三：八份被并掉的小文件里五份原来 `@` 常驻（verify-before-claiming、artifact-line-pointer、exhaustive-interleaving、service-stop-readback、communication-discipline-by-review），接收它们的 evidence-discipline、fs-design、command-safety-by-review、writing-discipline-by-review 在合稿入口文件里仍 `@` 常驻（`grep -nE '^@' <合稿树>/CLAUDE-merged.md` 第 47、55、58、59 行，量的）；另三份原来就按需，并进的 machine-first 在入口文件写着开会话先整份读一次。没找到「原来常驻、并完变成不读就做错」的。
- 格四：合稿共用约束里指到被并文件的三处（说外部状态之前现查、话缩到核过的范围、样本进程怎么停）都改指接收文件且节名解得到；`grep -rnE` 八个旧文件名在合稿树里 0 命中（量的）。攻方定义没有再点名已并掉的文件。只核了这几处，各定义「开工先读：」一行里的节名靠 `pointer-check.py` 机械核。

## 我认为最脆弱的三处

1. **verification.md 拆出参考件之后，指到它「一个门禁：两种跑法」的指针**。这一组里两处落空（一-1、一-2）是同一个病根；别的组还有多少处指到这一节而要的句子其实在 verification-reference.md，我没核，值得全树扫一遍：对每一处指针，核它要的那句话在不在第 26 到 46 行。
2. **照「找不到」处置的条目**（五-1 一类）。总表里凡写「上游没有」「找不到对应的节」「0 命中」而把指针删掉或改成「记欠账」的，依据都是写手的一次搜索；五-1 那一次搜的是 verification.md，内容在 implementation-workflow.md 的门禁表里。常驻-6（「门禁是下限」说 sop-first 里 0 命中）是同一种处置，我没复核它在别的上游文件里有没有。
3. **总表「只改措辞」的行**（五-2 一类）。这一组登的 19 行（常驻-1 到常驻-19）里 9 行标「只改措辞」（`grep -c` 量的），我只对着原件核了常驻-11、常驻-12 两行；常驻-11 实际收掉了检出工具自己终止的许可。矛盾那一题若按种类分轻重，这一类是漏网的口子。

## 要主 agent 跑的命令

没有。这一轮没写模型、没起项目代码；两份核对脚本只读文件，判决要引上面标「量的」那几个数时，主 agent 可在仓根直接重跑下面几条只读命令核对：

- `awk 'NR>=26 && NR<=46' research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md | grep -cE 'lkmm|litmus|九格|核账|领卡'`，要输出 0。
- `grep -n '^#' .claude/singlefs-ai-sop/rules/communication-discipline.md`，要没有「射程与例外」。
- `grep -nE '每条 Never 有对照组' research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md .claude/rules/implementation-workflow.md`，要各命中一行。

## 这条腿自己的限度

- 每个打中只抽了一次样，也没有辩方复核；五-1、二-1 的改法都是这条腿自己提的，只读过文字、被攻过零轮。
- 「逐句对过」是人眼并排读，不是机械比对；合稿把一段话拆成几条、换了次序的地方可能漏看一个限定词。机械两项只认反引号片段与「文件加节名」的指针，认不出不带反引号的阈值、次序与例外。
- `span-check.py` 只在合稿树里找片段，不去上游找；它报「找不到」的片段我逐个看了去向，它报「都在」不等于那句话的意思还在。
- 没读各写手的对照表与 merge-result.md 的正文（只用 merge-result.md 的清单一栏认哪些文件并掉了），也没读禁读清单里的任何文件。
- 判据「没登而做法变了」我只在逐句对过的那十六份上核过；只抽了的那十九份（看了片段的七份与只有机械两项的十二份）上不成立。

## 什么现象会推翻这份报告的结论

- 一-1、一-2：合稿 verification.md 第 26 到 46 行里其实有合并核账、领卡或 litmus 判定单位的句子（我的 grep 关键词取漏了）。
- 二-1：仓里新增测试的变异行只有 crates/mutations.tsv 一个落点（`scripts/mutation/tables/` 不收新测试的证红），那就只有一种读法。
- 二-2：上游 communication-discipline.md 在这一轮之后会加「射程与例外」一节（合稿是照将来的上游写的）。
- 五-1：欠账表里已有这笔账而编号行不带 lkmm 或「对照组」字样；或用户、主 agent 说的「后两样」指的不是对照组与锚点。
- 五-2：原件「检出的工具可以终止」在今天的钩子与看门狗里本来就没有任何一处在用（那就只是删了一句空话）。

## 没做什么

- 不判正推腿那几格，不替主 agent 采纳；没往统一任务服务交任何任务。
- 没核 `crates/` 与脚本的行为，只在三处读了脚本文件头（`.claude/scripts/lkmm.sh`、`.claude/gate.d/lib-lag-baseline.py`、`scripts/mutation/broad-mutation.py`）当指针的落点。
- 没复核矛盾总表里不属于这一组的行，也没核合稿树的 sha256（开工只核了原件那一份 `judged-sha256.txt` 全部对上）。
- 没建仓副本、编译目录；草稿目录里只有两份脚本与两份输出，不用清。
