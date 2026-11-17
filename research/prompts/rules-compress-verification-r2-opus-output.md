# rules-compress-verification-r2 攻方腿报告

日期：2026-11-16。轮名 rules-compress-verification-r2，第二轮；前几轮判决 `research/prompts/rules-compress-verification-r1-main-verification.md`。

没写模型，没有复跑命令；这一轮只对文字、记录与脚本文件头。开工核过：三份原件与 `research/prompts/rules-compress-verification-r2-snapshot/judged-sha256.txt` 逐份相同（`sha256sum -c` 3 个 OK，量的）；合稿树对 `merged-tree-sha256.txt` 跑 `sha256sum -c`，筛掉「: OK」之后没有剩行（量的）。

下文「第二版」指 `research/prompts/rules-compress-r2-merged/dot-claude/` 下的同名文件，「原件」指仓里 `.claude/rules/` 下的三份。没读禁读清单里的任何文件。第一轮两份腿报告已打中、第一轮判决已采的不再报，只报修补本身与第一轮写明没核的地方。

## 各格判定一览

| 格 | 判定 | 打中几条 | 最重的一条 |
|---|---|---|---|
| 一 修补对不对 | 不成立 | 4 条 | 一-1：第 18 条的落法只定了提交时那一趟，提交之外要不要两次弹窗没定，而第一轮判决写「没有留着没定的」 |
| 二 修补引出的新问题 | 不成立 | 5 条 | 二-1：「设计推论轮」没有定义，规则、定义、治理文件这几类轮（含这一题自己）照哪一套开轮，两份规则读出两种 |
| 三 第一轮没看到的 | 不成立（多半是原件就带着、第二版照抄） | 6 条 | 三-1：参考件说 formal-verification「钉全」是一项，门禁文件头与参考件自己的另一节都说钉全不是这道门禁的项 |
| 四 不采纳的那几条 | 不归攻方 | — | — |

「打中」都是文字对照与 grep 出来的，一次样本；主 agent 采之前逐条现查。「没打中」的范围写在末尾，照规则一次不算数。

## 格一：修补对不对

### 一-1 第 18 条（提交之外一律先弹窗）只落了提交时那一趟，提交之外的落法是两次弹窗（重）

- 定案原话 `records/2026-11-16-三方豁免取消与存量补轮.md:170`：「用户要求的那一趟也弹窗确认一次再跑」
- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`：「提交时那一趟的确认就是带 `--full-gate` 的那一次弹窗」；同一行前半「一律先弹窗问用户，同意了才跑」
- 钩子 `.claude/hooks/run-scope-guard.sh:33`：「对话不许自己用，要用户在界面上确认」（带 `--full-gate`、`--whole-package-confirmed` 的提交，钩子自己弹一次）
- 服务 `task-server/task_server_lib.py:2092`：「full-gate-not-commit-entry｜提交时拒」（同一行只按命令认，不看是不是提交时：六道门禁、崩溃放量全部分项与 `gate-staged.sh` 在提交之外带 `--full-gate` 照收）
- 第二版技能 `research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/staged-gate.md:17`：「（用户要求时 `=user-request`）」（用户要求的那一趟交的是同一条带 `--full-gate` 的命令）
- 为什么算打中：第二版把「确认一次」只落到提交时（`--full-gate` 那一次就是确认）。提交之外，字面是先 AskUserQuestion 问一次、同意了再交，交的命令带 `--full-gate`（六道、`gate-staged.sh`）或 `--whole-package-confirmed`（整包测试、不带选条参数的入口），钩子再弹一次：一次用户要求两次确认，与定案「确认一次」不合。第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/session-wrapup.md:37`：「同意了才跑 `gate.sh --staged`」那一条路同样是两次。
- 修稿一方自己知道这一处没定：`research/prompts/rules-compress-r2-merged/contradictions.md:147`：「两次弹窗还是一次，交主 agent 或用户定」。第一轮判决却写 `research/prompts/rules-compress-verification-r1-main-verification.md:32`：「没有留着没定的」。修稿后补的半句只答了提交时，提交之外仍开着，判决与总表对不上。
- 照第二版做会怎样：主 agent 照常驻先弹窗，用户同意，交命令，钩子再弹；或主 agent 把钩子那一次当成「先弹窗」直接交。两种做法都读得出，定案要的是一次。

### 一-2 「停」与「再抽一次」的先后，修稿把依据写成第 17 条，第 17 条没说这件事（中）

- 去向表 `research/prompts/rules-compress-r2-merged/dispositions.md:35`：「用户定案第 17 条已定」（这一行处置的是第一轮攻方二-3「第一轮没打中就停」与「两次都没打中才记」谁先）
- 矛盾总表 `research/prompts/rules-compress-r2-merged/contradictions.md:128`：「「没打中」的结论拿去支撑别的结论时才再抽一次」（这一行标「用户定」「改了判据」）
- 定案第 17 条原话 `records/2026-11-16-三方豁免取消与存量补轮.md:169`：「17. 代码轮打中之后：「现查坐实就改，再攻一轮」」，整条说的是代码轮打中之后怎么收，没有一个字讲「没打中」要不要再抽样；讲开轮的是第 2 条 `records/2026-11-16-三方豁免取消与存量补轮.md:119`：「2. 三方开轮：「两句管两件事，都留」」，也没讲两句谁先。
- 为什么算打中：第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43` 与 `:70` 新写的划界（停只管开不开下一轮，再抽一次只管「拿去支撑别的结论」）是修稿一方定的，总表把它挂在用户定案名下、标「用户定」。口径 `records/2026-11-16-三方豁免取消与存量补轮.md:77`：「「矛盾与不明晰」里改了做法的条目，主 agent 换上之前逐条核，改变了判据的交用户定」。这一处改了判据（原件 `.claude/rules/three-way-inference.md:158`：「拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次」，第二版收窄成「别的结论」，见二-3），没交用户，却登成用户定的。
- 照第二版做会怎样：主 agent 判决时把这条当用户定案、不攻该不该；第三轮也会照「定案不攻该不该」放过它。

### 一-3 「开了第二、三轮的」没把第一轮攻方二-3 的第二处修掉（中）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43`：「开了第二、三轮的，其中至少一轮派一条辩方腿复核已有的判决」；同一行「前两轮不一致、无法收敛的开第三轮」
- 第一轮攻方说的是：没打中而不收敛时开第二轮，第二轮收敛了就停在两轮，这时「至少一轮派辩方」管不管第二轮。
- 为什么算打中：「第二、三轮」用顿号并列，读得出两种：开了第二轮与第三轮两轮的（只开了第二轮的不受管）；开了第二轮或第三轮的（只开了第二轮，第二轮就得派辩方）。停在两轮的那条路正是第一轮问的那一种，修法加了四个字，两种读法都还在。
- 照第二版做会怎样：只开第二轮的一题，一个主 agent 派正推，另一个派辩方，都合字面。

### 一-4 放回常驻的判器调用点那一句，判据是「两样都不是就红」，文件头是五样之一成立就合规（中）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:25`：「两样都不是的判红」（两样：落在判定段里，或包住它的函数带标记）
- 原件 `.claude/rules/verification.md:141`：「两样都不是的调用点判红」（原件在表里，第二版照它放回常驻）
- 文件头 `scripts/crash/cpu-judging-sites-check.py:15`：「一处调用点算合规，有一样成立就行」，下面列五样：判定段、`scripts/crash/cpu-judging-sites-check.py:17`：「② 落在判器本体里」、标记、`scripts/crash/cpu-judging-sites-check.py:21`：「④ 文件在登记表 excluded 那几行里」、`scripts/crash/cpu-judging-sites-check.py:22`：「⑤ 落在判器自测里」。
- 为什么算打中：判据格一要「放回的句子与它指到的脚本文件头是一回事」。放回的那句是原件原话，但它点名的判法文件头里合规的有五样，常驻写成两样、而且是闭合的「都不是就红」。参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:154` 同一条已改成「合规的几种位置…在它文件头」，常驻这一句没跟。原件里这句在表里、旁边就是门禁格名；放回常驻之后，它成了实现员定义点名要读的那一节里的判据句。
- 照第二版做会怎样：实现员写判器自测（手摆盘、给判器塞已知坏法）或判器本体里的调用，照常驻会给它补 `// gpu-exempt:` 或 `// cpu-reference-for-gpu-comparison` 标记、或挪进判定段；照文件头该登 `judge-self-test`、`judging-body` 一行。打错标记的那一处 gpu-exemptions 那一格还要另判截名。

## 格二：修补引出的新问题

### 二-1 「设计推论轮」没有定义，规则、定义、治理文件的轮照哪一套开轮读出两种（重）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43`：「**多轮这一套只管设计推论轮**」；代码轮指到 `research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:10`：「改完再攻一轮，那一轮没新打中就收，最多三轮」
- 找过的地方：合稿树 `dot-claude/` 与 `CLAUDE-merged.md` 整棵 `grep -rn '设计推论轮\|推论轮'`，只命中上面两行（量的）；「代码轮」有射程（`crates/` 下的代码），「设计推论轮」没有。
- 第三类轮：`research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:19`：「**改它们与改 `crates/` 同规矩**」（定义与共用约束）、`:21`：「**治理文件同规矩**」（规则、技能、钩子、门禁、脚本、两个服务）。
- 两种读法：（一）「同规矩」连轮数一起照代码轮：打中就改、再攻一轮、没新打中就收；（二）这几类轮不是代码轮，按适用范围「设计判断、取舍结论」算推论，照「先挂起、同一个结论再攻两轮、三轮多数」。
- 为什么算打中：定案第 17 条只把轮分成「代码轮」与「设计推论轮」两类，第二版照写，没写第三类归哪边。这一题自己就是第三类：第一轮判决 `research/prompts/rules-compress-verification-r1-main-verification.md:28`：「改法都落在第二版合稿树」，第二轮「只攻第一轮的修补」，是代码轮的走法；可这一轮的攻击面又取自设计推论轮那一套的「第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方」。
- 照第二版做会怎样：这一题第二轮没新打中时，照（一）收、不开第三轮；照（二）第一轮的打中还挂着，要开第三轮、三轮多数才算。

### 二-2 「多轮这一套」管到哪儿：整节还是开轮那一段（中）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43`：「**多轮这一套只管设计推论轮**」，这一句是「多轮：一次打穿不算数，三轮里多数打穿才算」一节的起句；同一节下面还有第 45 行（第三轮之后停、零轮交用户表）、第 51 行（叫停的轮）、第 53 到 58 行（核查与判决由主 agent 做）、第 60 行（判决从骨架起笔）。技能里「核查与判决」也挂在这一节底下：`research/prompts/rules-compress-r2-merged/dot-claude/skills/three-way/SKILL.md:29`：「## 多轮：一次打穿不算数，三轮里多数打穿才算」、`:43`：「### 核查与判决由主 agent 做，不派子 agent 去核子 agent」。
- 同一行后半：「那里不用「先挂起、三轮多数」」，又像只排掉这两样。
- 两种读法：（一）只排「先挂起、三轮多数」，代码轮照旧守「三轮各换一组攻击面、不许重复已攻过的角度」「开了第二、三轮的至少一轮派辩方」「第三轮之后停」；（二）整节都不管代码轮，代码轮第二轮派什么腿、攻什么面、第三轮之后新打中的怎么办都没有规矩。
- 定案第 17 条只排掉「先挂起、三轮多数」（`records/2026-11-16-三方豁免取消与存量补轮.md:169` 那一条末句）。照（二），代码轮连「核查与判决由主 agent 做」也落在射程外，这一点主 agent 派发表另有一句（代码轮对着开工快照核腿引的行），不至于做错；攻击面与辩方腿两样没有别处兜着。

### 二-3 「任何结论」与「别的结论」在同一条里，停下的那一轮的结论算不算被支撑，读出两种（中）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:70`：「拿一条腿的「没打中」支撑任何结论之前」，同一行末句「后者管「没打中」能不能拿去支撑别的结论」；第 43 行「不是已经能拿这个「没打中」去支撑别的结论」
- 第 34 行：「**三方一致 → 继续。**」
- 原件 `.claude/rules/three-way-inference.md:158`：「拿一条腿的「没打中」去支撑任何结论之前，至少再抽一次」
- 两种读法：（一）这一轮被判的那个结论不算「别的结论」：第一轮两条腿一致没打中，就停、判决写成立，不再抽；（二）被判的那个结论也在「任何结论」里：停只是不开第二轮，判决要写成立之前，第一轮攻方腿照样用同一份提示再派一条 `-s2-`。
- 为什么算打中：第一轮攻方二-3 问的正是这两种读法；修稿加了一句划界，划界词与同一条首句的全称词不一样，两种读法都还在，而两种读法差一条腿。照（一），原件「支撑任何结论」被收窄了（见一-2，没交用户）。

### 二-4 代码轮的新走法留了两个空：现查推翻了怎么收，第三轮又打中怎么收（中）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:10`：「第一轮两条腿结论一致、没打中就停；打中的，主 agent 现查坐实、做成会红的测试后写回代码，改完再攻一轮，那一轮没新打中就收，最多三轮」
- 空一：打中而主 agent 现查推翻（不坐实、不改代码）。「改完再攻一轮」没有可攻的改动；「第一轮…没打中就停」又不合（有打中）。停还是再攻一轮，字面两边都不落。
- 空二：第三轮又有新打中。「最多三轮」只说不开第四轮；新打中改不改、改了算被攻过几轮、交不交用户，没写。设计推论轮有一句兜底（`research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:45`：「这一轮里新冒出来的零轮形态不再为它开一轮」，写进交用户表、标「零轮」），照二-2 读法（二）它不管代码轮；照读法（一）管，第三轮的改动是零轮改法，又撞上 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:38`：「腿提的、判决里标「零轮」的改法，写进产品代码之前先在探针里对它做 A/B」。
- 照第二版做会怎样：代码轮第三轮打中一处，一个主 agent 直接改了合入，另一个先做探针 A/B，另一个交用户；都读得出。

### 二-5 「第一轮一致没打中就停」又住了两处（轻）

- 第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/three-way-inference.md:43`：「第一轮两条腿结论一致、没打中，就停」；`research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:10`：「第一轮两条腿结论一致、没打中就停」
- 定案第 2 条 `records/2026-11-16-三方豁免取消与存量补轮.md:119`：「写成一处、另一处指过去」
- 为什么算打中：判据格二「同一条判据是不是又住了两处」。两处措辞差一个逗号，第 43 行那一处接着写了「停是不开下一轮…」的划界，第 10 行没有；代码轮停下之后要不要再抽一次，读第 10 行的人读不到划界，要靠第 70 行那句「第一轮「没打中就停」不受…拦」反找回来。

## 格三：第一轮没看到的（四道门禁文件头与 crash-point-replay 文件头逐句对参考件）

这一格六条里，除三-4 外都是原件就这样写、第二版照抄，不是压缩带来的；按正文「规则里说门禁与脚本怎么做的句子以它们文件头今天的原文为准」，照第二版读的人与照文件头读的人做得不一样，所以列出。主 agent 判它们算不算这一题的打中。

### 三-1 formal-verification 的「钉全」：参考件一处说是门禁的项，另一处说归 E165 驱动判（中，二义）

- 第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:138`：「证明整格、工具链、钉全各一项」
- 同一份参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:170`：「钉全由 E165 第二段驱动」（「写完一对红例账与产物时当场判」）
- 门禁文件头 `.claude/gate.d/formal-verification.sh:51`：「toolchain、proof（整格）、table 各一项」；判法 `scripts/experiment-apparatus/e165-pin-red-examples.py:35`：「门禁不调 --check」（钉全就是 `--check` 那一支）
- 原件 `.claude/rules/verification.md:126` 同样写「证明整格、工具链、钉全各一项」。
- 为什么算：参考件第 138 行把钉全当门禁的一项，第 170 行与文件头都说门禁不判钉全，门禁那一项是钉表成形（table）。要点名 formal-verification 的项的人照第 138 行找 `--item` 里的「钉全」，`--list-items` 里没有；照第 170 行才知道它不在门禁里。

### 三-2 broad-mutation：「不指定范围那一趟自己先起驱动」与文件头的「只在服务分片起的那一趟」（中）

- 第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:149`：「不指定范围那一趟这一格自己先起驱动剪枝跑」
- 文件头 `.claude/gate.d/broad-mutation.sh:30`：「不是服务分片起的这一道（gate.sh 整轮、手跑）与指定范围（--check gate-verdict）：不起驱动」
- 原件 `.claude/rules/verification.md:136`：「不指定范围那一趟 gate-verdict 格自己先起驱动剪枝跑」，第二版照抄。
- 为什么算：`gate.sh` 整轮也是不指定范围（第二版 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:35` 那张表把它列在不指定范围一行），照参考件它会起驱动，照文件头它只读库、库里没有键相同结论的判红。第二版常驻 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:45`：「整轮里这六道照键复用先跑那一趟写下的判定」与文件头一致，与参考件这一行不一致。

### 三-3 「formal-verification 各格」少了第七格 same-origin-counter（轻）

- 第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:165`：「什么时候读它：要知道 formal-verification 哪一格判哪一样时」，下面四条只讲 Verus 四格与红例钉表两格；参考件与常驻两份 `grep -c same-origin` 都是 0（量的）。
- 文件头 `.claude/gate.d/formal-verification.sh:16`：「不给就七格按下面的次序全跑」，`.claude/gate.d/formal-verification.sh:23`：「gate-cell: same-origin-counter」（不一致事件表里已立的对累计 ≥ 5 次不一致而判「spec 错」0 次即判红）。
- 原件同样没有这一格。参考件第 165 行同时写了「格名表以 formal-verification.sh 文件头为准」，按指针做不会错；列出来是因为这一节自称答「哪一格判哪一样」，列了六格、整道是七格。

### 三-4 按需领卡：「领卡被拒的那一格判红、那一趟不起」对 feature-gated-tests 那一处不成立（轻，修稿留下的半句）

- 第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:21`：「领卡被拒的那一格判红、那一趟不起」（管三处：崩溃枚举用例带 gpu 的、feature-gated-tests 带 gpu 那一趟、产品 GPU 对拍）
- 文件头 feature-gated-tests 那一处 `.claude/gate.d/crash-point-replay.sh:353`：「外层不领卡、不设 GPU_CARDS_BY_PRIORITY」（测试进程到真要上卡才自己领），`.claude/gate.d/crash-point-replay.sh:359`：「别的测试目标照结局记，排在最后的那一趟不再起」；领卡没成的是那个测试目标记本次未跑、这一格判红，带特性那一趟已经起了。
- 原件 `.claude/rules/verification.md:39` 写的是「带 `gpu` 特性那一趟先只编（`--no-run`，不持卡）、编完领一张、跑完就还」，这正是文件头今天的弄坏开关 `.claude/gate.d/crash-point-replay.sh:361`：「弄坏开关 feature-pass-outer-takes-card 下照外层先只编、再领一张」。第二版把各处怎么领改成指文件头（对），留下了原件那套做法配的半句「那一趟不起」。
- 为什么算：领法的改动已登在矛盾总表 `research/prompts/rules-compress-r2-merged/contradictions.md:24`：「规则不再写这一趟怎么领卡，照门禁文件头「显卡与清场」」（验证-1），登得对；登的是删掉的那半，留下的「那一趟不起」没跟着改，与它指到的文件头对 feature-gated-tests 不是一回事（按「外层领卡」「测试进程」在总表里 `grep -c` 都是 0，量的）。

### 三-5 crash-point-replay：挂特性的测试文件跑到 0 条，参考件列在「判什么」里，文件头不判红（轻）

- 第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:146`：「跑到或复用的用例不为 0；产品 GPU 包」（这一行左栏是「判什么」）
- 文件头 `.claude/gate.d/crash-point-replay.sh:249`：「挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER」（同一行「都报「本次未跑」、不判红」），退出码 `.claude/gate.d/crash-point-replay.sh:223`：「0   绿；带特性那一格有本次未跑时也退 0」
- 原件 `.claude/rules/verification.md:134` 同样写在「判什么」那一栏。照参考件读，0 条是这道门禁判红的一种；照文件头是本次未跑、整道退 0，提交时要人去看「本次未跑」那几行。

### 三-6 qemu-device-streams 与 lkmm 两行的转述比文件头窄（轻）

- qemu：第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:158`：「两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比」；文件头 `.claude/gate.d/qemu-device-streams.sh:12`：「gate-cell: device-streams」（同一行「各档负载跑完…两个对照红在该红的地方」，六个模式各一档，漏一道屏障与走页缓存两个对照必须判红）。参考件没写两个对照必须红，读参考件的人改负载时不知道对照档红不起来也会让这一格红。
- lkmm：第二版参考件 `research/prompts/rules-compress-r2-merged/dot-claude/rules/verification-reference.md:42`：「lkmm 每份 litmus」（按判定入库的单位）；文件头 `.claude/gate.d/lkmm.sh:14`：「每份 litmus 两条判定」（herd7 一条、绑定一条，绑定那一条的键另含锚点指到的函数单元指纹）。改锚点指到的函数只作废绑定那一条，照参考件读会以为整份 litmus 重判。
- 两处原件同样窄（原件 `.claude/rules/implementation-workflow.md:78` 与 `.claude/rules/verification.md:42`），不是压缩带来的。

## 打中之后的四句（照 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

这一轮判的是文字，没有臂；四句按文字题改着答：

- 分不分辨：一-1、一-2、二-1、二-3、二-4 都是「照第二版能做出两种不同动作」（弹几次窗、派不派 `-s2-` 腿、开不开第三轮、第三轮的改动怎么收），分辨得出；三-3、三-6 按指针做不会错，只分辨「读参考件的人」与「读文件头的人」说出口的那句话。
- 判别它的东西看不看得到：都在仓里，`grep` 得到，引文行号照上面现查。
- 满足判据字面的哪一句：格一「与文件头不符」（一-4、三-4）、「放回的话…做法因此不同」（一-1、一-3）；格二「新写的句子读不读得出两种意思」（二-1、二-2、二-3、二-4）、「同一条判据又住了两处」（二-5）；格三「二义」（三-1）与「文件头为准」下的不一致（三-2、三-3、三-5、三-6）。一-2 归格一（修补的依据写错）还是格四（不采纳的理由）由主 agent 定；我按「修补对不对」报。
- 改法碰不碰得到：下面的改法只按文字推，被攻过零轮，没在任何副本上量过。一-1：在 `verification.md:38` 写明提交之外带放宽标识的那一次钩子弹窗算不算定案的「那一次」，算不出就交用户；一-2、二-3：把划界交用户定，或改回原件「任何结论」；一-3：写成「只开了第二轮的，第二轮派辩方」或「开到第三轮的」二选一；一-4：常驻那句改成指文件头「一处调用点算合规」并写「五样之一」；二-1：给规则、定义、治理文件的轮写明归哪一套；二-2：把「这一套」写成「「先挂起、三轮多数」这两样」；二-4：补现查推翻与第三轮新打中两支；三-1：第 138 行「钉全」改「钉表成形（table）」；三-2：写成「经服务分片起的那一趟」。

## 没打中的形状（一次样本，不算「没打中」的结论）

逐句对过、没找到问题的：

- `diff-from-r1.patch` 里落在这一组的修补，除上面几条外：`verification.md` 第 23 行（崩溃枚举用例登记进合表、测试文件第一行声明模块，对 `scripts/crash/crash-case-check.py` 文件头 placement ③④ 与 modules 一致）、第 26 行、第 27 行（测试线程；原件「带 gpu 特性那一趟与产品 GPU 真显卡那一趟」放宽成「各趟」，对 `scripts/crash/crash-point-replay.sh`、`scripts/gate/admission.py` 与 `.claude/gate.d/crash-point-replay.sh` 里 `--test-threads` 的用法 `grep` 过，只有弄坏开关下才带，一致）、第 39 行（两种排除，对 `scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 一致）、第 55、58、65、67、99 行；`implementation-workflow.md` 第 9、23、38、51 行；`three-way-inference.md` 第 35 行；参考件第 138 行的两个段落名（`scripts/gate/verus-proof-check.py` 第 55、151 行在）、第 146 行 `exhaustive=`（对 `scripts/gate/admission.py` 第 163 到 164 行一致）、第 154 行、第 195 行（`scripts/hosts/minimal-copy.py` 第 11、35 行在，`scripts/hosts/lib_repository_copy.py` 有 `configuration-path`）、第 211 行（对 `.claude/gate.d/crash-point-replay.sh` 第 137 到 145 行一致）、第 213、217 行；技能第 16、17 行（退出码 5 到 8 与 C273、C234、C320 对 `scripts/kb/quote-kb.py` 文件头在；退出码 4 没列，技能写了「退出码在两份脚本的文件头」，不算）。
- 指针：上面列到的与格一到格三引到的节名、段落名逐个 `grep` 过，都指得到；这一组别的指针没逐个核，修稿一方另有全树指针脚本的结果（`research/prompts/rules-compress-r2-merged/dispositions.md` 第 53 行那一格），我没复核（`fs-design.md`「门禁的范围必须可判定」是编号条目的粗体名，原件也是）。
- crash-point-replay 文件头第 1 到 846 行里与参考件「门禁 crash-point-replay 怎么跑」「门禁管哪一半」第一行、「分片三步」有关的段落（格名表、指定范围、逐项点名、复用、分片、没有分片阶段、用法、退出码、feature-gated-tests 格、product-gpu-real-card-tests 格、显卡与清场、按条复用两趟并行）逐段对过：九格、`--item` 两类、合并核账、`separate-run=`、`red-blocks=judged-by-test`、片数上界三种取法、test-targets-parallel 的 `--jobs` 与原样转交的参数，都对得上；只三-4、三-5 两处不合。样本档、弄坏开关表、出口 4 那五格的细节没逐句对（参考件没转述它们）。
- lkmm 文件头第 1 到 63 行、qemu-device-streams 第 1 到 148 行、broad-mutation 第 1 到 116 行、formal-verification 第 1 到 120 行整段读过，与参考件逐行对；formal-verification 第 121 到 245 行（样本与弄坏开关表的后半）没读。
- 定案第 2、17、18 条在这一组的落点逐处读过；第 9 条（RefFS 只许 `std`）参考件第 153 行写了门禁那一格还许 `singlefs-format`、收严记在欠账，落得对。

## 我认为最脆弱的三处

1. 轮的分类（二-1、二-2、二-4，连同一-2）：第 17 条把轮切成两类，第二版照切，规则、定义、治理文件的轮与「这一套」的射程都没写，这一题自己开不开第三轮就取决于读哪一种。
2. 第 18 条提交之外的弹窗次数（一-1）：判决说没有留着没定的，矛盾总表说交主 agent 或用户定，常驻只答了提交时。
3. 参考件转述门禁文件头的句子（三-1 到三-5）：参考件节首都写「以文件头为准」，正文又照原件转述了一遍，原件转述里与文件头不合的几句跟着进来；文件头一改，这类句子没有门禁核。

## 这条腿自己的限度

- 全部是读文字对照，没跑任何项目代码，没建模型、没建副本；量的只有 `grep`、`sha256sum -c`、`sed -n` 取行。中途有一条命令误点到门禁的 `--list`，被项目代码闸拒了，没跑成，结论里没有用到它。
- 一-1 的「两次弹窗」是按钩子与服务文件头推的，没在界面上试；`--whole-package-confirmed` 那一路只读了 `.claude/hooks/run-scope-guard.sh` 文件头第 33 行与 `task-server/task_server_lib.py` 第 2092 到 2095 行。
- 三-1 到三-6 多半是原件带来的，这一题判「压缩丢没丢」，它们算不算打中由主 agent 定。
- 一次样本。

## 要主 agent 跑的命令

无。这一轮没有要经任务服务跑的命令。

## 没做什么

- 不判辩方那几格（格四），不替主 agent 采纳；没往统一任务服务交任务；没建副本与编译目录，草稿目录里只有一份标题行摘录 `cpr-top.txt`。
- 合稿树里别组的文件（`main-agent.md`、`agent-common.md`、技能 crash-test 与 gate）只在追落点时读到相关的行，没逐句判。
- 改法都是我提的，只按文字推、被攻过零轮。
