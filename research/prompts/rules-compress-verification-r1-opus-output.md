# rules-compress-verification-r1 攻方腿报告

日期：2026-11-16。轮名 rules-compress-verification-r1，第一轮，前几轮判决：无。

没写模型，没有复跑命令；这一轮只对文字与脚本文件头。合稿树照新路径引（`research/prompts/rules-compress-r1-merged/dot-claude/…`）；开工时核过合稿树 54 份与 `research/prompts/rules-compress-verification-r1-snapshot/merged-tree-sha256.txt` 逐份相同（`sha256sum -c` 54 个 OK，量的），三份原件与 `judged-sha256.txt` 相同（3 个 OK，量的）。干到核快照那一步收到主 agent 的消息（合稿树 `.claude/` 改名 `dot-claude/`、`CLAUDE.md` 改名 `CLAUDE-merged.md`），重读了正文，之后全部照新路径读与引；内容没变（sha256 对得上）。

下文「合稿」指 `research/prompts/rules-compress-r1-merged/dot-claude/` 下的同名文件，「原件」指仓里 `.claude/rules/` 下的三份。「矛盾总表」指 `research/prompts/rules-compress-r1-merged/contradictions.md`。

## 各格判定一览

| 格 | 判定 | 打中几条 | 最重的一条 |
|---|---|---|---|
| 一 丢失 | 不成立（有丢的） | 5 条（1 条中、4 条轻） | 一-1：旧库由谁删丢了 |
| 二 二义 | 不成立（有二义） | 5 条（2 条重、3 条中） | 二-1：参考件里两个同名节「门禁管哪一半」，指针不唯一 |
| 三 常驻够不够 | 不成立（弱） | 3 条（都轻到中） | 三-1：「测试线程」的入口只写给用 GPU 的活 |
| 四 子 agent 读得到 | 推不出 | 1 类 | 四-1：细则挪走后只剩两跳指针，共用约束没写链上的指针跟不跟 |
| 五 悄悄改了做法 | 不成立（有没登的） | 4 条（都是照脚本改、没登进矛盾总表） | 五-4：登记表第三列从「只剩这一行」改成「只许是这两类」 |

「打中」都是文字对照出来的，一次样本；主 agent 采之前逐条现查。「没打中」的范围写在末尾，按规则一次不算数。

## 格一：丢失

### 一-1 旧库由谁删（中）

- 原件 `.claude/rules/verification.md:68`：「那一族接完之后旧库由主 agent 删掉」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:54`：「旧库不原地打开、不并存」
- 找过的地方：合稿树整棵 `grep -rn '由主 agent 删'`，命中 2 处（`dot-claude/skills/dispatch/SKILL.md:35`、`dot-claude/agent-common.md:100`），说的都是副本与编译目录，不是旧判定库（量的）。参考件「判定库只有一份」三小节逐句读过，没有。
- 为什么算丢：「接完」第 ③ 件只剩「只删那一族旧库所在的目录」，主语没了。
- 照合稿做会怎样：接一族的工具员或实现员读到第 ③ 件，会把删旧库当成自己那件活的一步；原件把这一步收在主 agent 手里。

### 一-2 一条测试自己的配置文件「与多机无关」这个例外（轻，连带格二）

- 原件 `.claude/rules/verification.md:78`：「是那条测试的事，与多机无关」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:63`：「进仓或放共享根，两台都读得到」；上一条 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:62`：「环境相关的一切只由多机配置定，没写就是没有，不补缺省」
- 找过的地方：合稿树整棵 `grep -rn '与多机无关'` 0 命中，`rocksdb-prebuilt` 0 命中（量的）。
- 为什么算丢、算二义：原件先说「环境相关的一切只由多机配置定」，紧跟一句把测试自己的配置文件划出去。合稿把划出去的那半句删了，第 62、63 两条并排，读得出两种：测试自己的配置文件是「只由多机配置定」的例外；或者它里面不许放环境相关的值。两种读法下 `qemu.env` 里能不能写机器相关的参数答案相反。合稿「配置文件读出什么就是什么」一节把三种配置文件并列，能帮着读成前一种，但那是另一节。

### 一-3 「任何命令都派得到两台，哪台由余量定」（轻）

- 原件 `.claude/rules/verification.md:77`：「任何命令都派得到两台，哪台由余量定」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:61`：「也不钉台」
- 找过的地方：合稿树整棵 `grep -rn '余量'` 2 处，都在 `dot-claude/skills/dispatch/SKILL.md`，说的是根分区（量的）。
- 为什么算丢：这是一句事实（服务此刻没有名单、按余量挑台），合稿只留了禁令。照合稿做不会做错，但「此刻有没有名单」要自己去读 `task-server/task_server.py` 文件头。

### 一-4 判器标记「写在函数体里的不算」只剩指针，指到的地方没有这句（轻）

- 原件 `.claude/rules/verification.md:141`：「写在函数体里的不算」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:154`：「标记写在哪才算在它文件头」
- 指到的地方：`scripts/crash/cpu-judging-sites-check.py` 第 14 到 21 行；`grep -n '函数体里' scripts/crash/cpu-judging-sites-check.py` 0 命中（量的）。文件头第 18 行只正面列了算数的位置（`scripts/crash/cpu-judging-sites-check.py:18`：「前面写了标记」），「函数体里的不算」要读的人自己反推。
- 另一处：合稿把行号写死成 14 到 21，而文件头的合规条目列到第 22 行（⑤ 判器自测），第 22 行落在指针外。参考件里这类写死的行号指针共 6 处（第 138 行两处、第 146 行、第 154 行、第 195 行两处），文件头一改就漂，没有门禁核。

### 一-5 主 agent 派发表里「含…」的括注指到的节已没有那两句（轻）

- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:96`：「带 gpu 特性那一趟按需领卡」（这一行说这两样「含」在 `.claude/rules/verification.md`「一个门禁：两种跑法」里）
- 合稿 `verification.md`「一个门禁：两种跑法」（第 26 到 46 行逐句读过）没有 feature-gated-tests 另跑一趟与按需领卡这两句；它们在参考件 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:20`：「feature-gated-tests 格再起一趟」与第 21 行。
- 为什么算：指针说「含」，指到的节不含，要再跟一跳（常驻第 44 行的入口）才到。不会做错，是两跳。

## 格二：二义

### 二-1 参考件里有两个节都叫「门禁管哪一半」，指针不唯一（重）

- 合稿参考件 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md` 第 140 行是二级标题「门禁管哪一半」，第 213 行是三级标题「门禁管哪一半」（挂在「测试与崩溃检测优先多线程」底下）；`grep -n '^##* 门禁管哪一半'` 在这份文件里命中 2 行（量的）。
- 指到它的两处：常驻 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:99`：「查一条判据有没有门禁管、红了归哪一格之前先读它」；参考件自己 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:217`：「判工作线程的那一条在」。
- 两种读法：读到第 140 行那张表；读到第 213 行那两行。后一种读不到任何一道门禁的判法。
- 连带：`scripts/agents/cite-check.py` 的 `路径@标题` 写法「标题命中不是恰好一次判红」，换上之后谁用 `verification-reference.md@门禁管哪一半` 引都红；`scripts/kb/quote-kb.py` 的 `@标题` 取法同理（推的，没跑）。
- 原件里没有这个问题：原件 `implementation-workflow.md` 的「门禁管哪一半」是行内粗体，不是标题。

### 二-2 提交之外跑不指定范围的：入口说「用户要求的照跑」，这一组三处仍写「先弹窗」（重）

- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/main-agent.md:26`：「提交之外，用户要求的照跑；主 agent 自己判要跑的，先弹窗问用户，同意了才跑」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:32`：「提交之外要跑先弹窗问用户」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:59`：「提交之外要跑先弹窗问用户」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:51`：「提交之外要跑不指定范围的」（这一行整行指回 `verification.md` 那张表）
- 为什么算：矛盾总表「入口-1」登了 `main-agent.md` 这一处改了判据（用户要求的那一趟不再另弹窗），但两种跑法的权威落点 `verification.md` 那张表与共用约束没跟着改。用户说「把整轮跑一遍」时，照 `main-agent.md` 直接交，照 `verification.md` 还要弹一次窗。原件里这两处本来就不一致（原件 `.claude/main-agent.md:18`：「不指定范围的只在提交时或用户要求时由主 agent 经服务交」），合稿修了一头、留了另一头。

### 二-3 「第一轮没打中就停」与「没打中一次不算」放进同一份文件之后没写谁先（中）

- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「第一轮两条腿结论一致、没打中，就停，不开第二轮」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:70`：「两次都没打中才记」
- 两种读法：第一轮攻方腿一份报告说没打中，就停；或者先照第 70 行再派一条同立场的腿（`-s2-`），两份都没打中才算「没打中」、才停。两种读法差一条腿。
- 这是用户定案第 2 条的落法问题，不是该不该：定案原话只说「第一轮两条腿一致、没打中就停」（`records/2026-11-16-三方豁免取消与存量补轮.md` 用户弹窗定的四条第 2 条），没说这里的「没打中」要不要先过抽样那一条。原件里「就停」不在 `three-way-inference.md`，两句不同文件；合稿并进同一份之后两句相隔 27 行、互不提。
- 同一段另一处：第 43 行写「其中至少一轮派一条辩方腿复核已有的判决」，而同一行许「前两轮不一致」收敛后停在两轮；停在两轮时这句管不管第二轮必须派辩方，读不出。

### 二-4 代码轮「打中的写回代码」什么时候写回（中）

- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:10`：「打中的写回代码，开不开下一轮照」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:43`：「打中的东西先挂起、不直接写进正文」
- 两种读法：代码轮第一轮打中就改代码，再看开不开下一轮；或者照「多轮」先挂起，三轮多数打穿才改。矛盾总表「合稿-1」把「再攻一轮」换成了指针，但「写回代码」四个字留在指针前面，次序上仍是先写回。按前一种读，第二轮攻的是改过的代码，「同一个结论再攻两轮」就不是同一个对象了。

### 二-5 见一-2（多机配置与测试自己的配置文件）

## 格三：常驻够不够

### 三-1 「测试线程」的入口只写给用 GPU 的活（中）

- 入口：合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:46`：「跑或改用 GPU 的测试、入口与门禁之前，先读」
- 细则：合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:81`：「控制测试线程的只有变异那一路」，同一行点名的不带 `--test-threads` 的几处里有 `scripts/crash/crash-point-replay.sh` 与 `scripts/gate/admission.py` 拼的 crash-case 命令，这两样不限于 GPU。
- 原件这一条常驻：`.claude/rules/verification.md:59`：「控制测试线程的只有变异那一路」
- 不读参考件会做错的一步：主 agent 手写点名测试目标的 `cargo test … -- --test-threads <n>`（不带 `gpu` 特性），或工具员改 `admission.py` 拼的命令往里加 `--test-threads`。合稿常驻三份里没有一句不许；第 44 行的入口写的是「起一道门禁或入口、照它的写法单跑一趟」，点名测试目标的 `cargo test` 不在它的字面里；共用约束拒的线程参数名单（`--threads`、`SINGLEFS_THREAD_CAP`）里没有 `--test-threads`（推的：按合稿 `agent-common.md` 字面读，没去试服务拒不拒）。

### 三-2 技能里「判决由主 agent 做，不由投票做」一节没有常驻入口（轻）

- 技能 `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:24`：「什么时候读它：腿要交命令给主 agent 跑时」
- 常驻对应那一条 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:35`：「腿要跑的都交主 agent」，这一条里没有「先读技能」；常驻全篇指到技能的 6 处是整行抄、叫停轮、核查、骨架、云端腿、（经 `implementation-workflow.md`）快照与检查单，没有这一节。
- 只在技能里的两句：原件 `.claude/rules/three-way-inference.md:107`：「并指到输出路径」与同一行的「`--cwd` 给腿自己的草稿目录，或输出写绝对路径」；合稿只在 `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:26`：「并指到输出路径」。
- 后果轻：共用约束「要跑的命令」那一条另写了在哪个目录起与指到任务号、输出路径，腿与主 agent 不读技能这一节也大体做得对。算入口缺一处。

### 三-3 常驻给单元键下了全称定义，两种排除只在参考件（轻）

- 常驻 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md:35`：「单元键只由判定依赖的函数单元各自去注释后的指纹」
- 排除在参考件 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md`「单元键的组成与排除」；入口在 `research/prompts/rules-compress-r1-merged/dot-claude/rules/implementation-workflow.md:42`：「改算键的代码、登记文件级或边级排除」。
- 不读参考件会说错的一步：有人改了 `scripts/gate/lib_function_units.py` 之后估「哪些判定要重判」，照常驻那句全称会答「引到它的都要」，照文件级排除实际是「除 crates 变异判法外都不变」。入口只写给改算键代码的人，没写给估重判范围的人。门禁自己照键选，不会判错；错的是人说出口的那句。

## 格四：子 agent 读得到（推不出）

### 四-1 定义点名的节，细则挪走后只剩节里一句「先读」把人带过去

- 材料员：合稿 `research/prompts/rules-compress-r1-merged/dot-claude/agents/three-way-materials.md:16`：「不许摘句——三处都管」（点名 `three-way-inference.md` 这一节整节）。它干活要用的细则（索引表用 `@标题` 取会退 6 时怎么标清单、非用行区间时看被抄内容最后一行）原件在这一节里，合稿在技能 `research/prompts/rules-compress-r1-merged/dot-claude/skills/three-way/SKILL.md:15`：「例外是分项索引表」；定义没点名技能，靠常驻 `research/prompts/rules-compress-r1-merged/dot-claude/rules/three-way-inference.md:22`：「材料与附录用脚本做」那一句转过去。
- 实现员、执行员、调查员：三份定义点名 `verification.md`「一个门禁：两种跑法」，细则挪进参考件；合稿 `dot-claude/agents/` 里点名 `verification-reference.md` 的只有 `tooling-writer.md` 一份（`grep -rn verification-reference` 在 `agents/` 下命中 1 份，量的），其余靠常驻第 44 行那一句转。
- 为什么判「推不出」而不是打中：定义指到的文件（规则）子 agent 确实读；规则里那一句「先读」再指到的文件，合稿共用约束讲规则怎么读的那一节（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:17`：「点到就整份读」）只说了定义点到的规则文件怎么读，没说规则正文里的「先读某技能、某参考件」算不算点到。算，就读得到；不算，材料员照合稿做会少掉索引表那一支的处置（`quote-kb.py --checklist` 退 6 之后不知道怎么改清单）。
- 出路（我提的，零轮，没量过）：共用约束讲规则怎么读的那一节加半句「规则正文里写『先读』的参考件与技能小节照点名办」，或在这四份定义的「开工先读」里直接点名。

## 格五：悄悄改了做法（没登进矛盾总表的）

四条都是合稿照脚本文件头改了原件的说法，方向与脚本一致；在矛盾总表里按关键词各搜一遍都是 0 命中（`grep -c` 「挂特性」「跑格」「每一种跑法」「无判定库」各 0，量的）。

### 五-1 crash-point-replay 可点的项多了一类

- 原件 `.claude/rules/verification.md:126`：「按崩溃枚举用例（」（这一处只列了崩溃枚举用例一类）
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:130`：「与挂特性的测试文件（它文件头「逐项点名」）」
- 脚本 `.claude/gate.d/crash-point-replay.sh:40`：「项有两类」
- 做法变没变：变了，照合稿可以 `--item feature-gated:<测试文件名>`，照原件没有这一类。

### 五-2 crash-point-replay source 不 source 共用库

- 原件 `.claude/rules/verification.md:112`：「不 source 它，参数照同一张表自己解析」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:116`：「不 source 它跑格，参数照同一张表自己解析」
- 脚本 `.claude/gate.d/crash-point-replay.sh:237`：「只 source 它取」
- 做法变没变：原件那句与脚本对不上，合稿改对了；没登。

### 五-3 crash-point-replay 哪几种跑法分片

- 原件 `.claude/rules/verification.md:39`：「不指定范围那一趟经服务交，服务照它文件头」；原件另一处 `.claude/rules/implementation-workflow.md:88`：「经服务交的每一种跑法都拆成计划、各片、合并」
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:18`：「经服务交的每一种跑法（不带参数、」
- 脚本 `.claude/gate.d/crash-point-replay.sh:135`：「让服务把经它交的每一种跑法」
- 原件两处说法宽窄不同，合稿取了宽的（与脚本一致）；这一处原件内部不一致没登。

### 五-4 登记表第三列：从「只剩这一行」变成「只许是这两类」

- 原件 `.claude/rules/verification.md:67`：「第三列不是 prune-server 的只剩」（接着点名「不算：」行与 `.claude/gate.d/fixtures` 那一行）
- 合稿 `research/prompts/rules-compress-r1-merged/dot-claude/rules/verification-reference.md:106`：「第三列不是 prune-server 的只许是」（接着写「不算：」行与「无判定库：<欠账号或待立>」行）
- 表头 `.claude/gate.d/verdict-library-scripts.tsv:3`：「没有判定库的测试入口写」
- 做法变没变：原件是现状句（此刻只有 fixtures 一行写「无判定库」），合稿改成许可句（任何没有判定库的测试入口都许登「无判定库：<欠账号>」）。与表头一致，但射程宽了：照合稿，新加一份入口登「无判定库：C 号」是合规的；照原件，多出这样一行就与规则那句现状对不上、要回来改规则。

## 没打中的形状（一次样本，不算「没打中」的结论）

逐句对过、没找到丢失或改做法的范围：

- 原件 `verification.md`「一个门禁：两种跑法」第 29 到 61 行全部句子：单元键的组成、crates 变异判法除外、边级排除三格自证与「键不变而退出码变」、python 模块级语句、`TAIL_READ_SCOPES`、缩域与合入后验证的回落深度、各道判定单位与记红撤绿名单、库名与命名空间、哨兵与强制现判、GPU 清场各支、测试线程变异那一路。落点都在合稿常驻或参考件，句子在。
- 挪成「以文件头为准」的那几处，逐个到文件头里找过原话或同义句：`separate-run=`、`red-blocks=judged-by-test` 与「红块不判」、`CRASH_CASE_BLOCKS` 行、`NOT_RUN_NO_GPU_ADAPTER` 逐条报本次未跑、挂特性的文件跑到加复用 0 条报本次未跑、`GPU_CARDS_BY_PRIORITY` 已有照用、按条复用成功句（都在 `.claude/gate.d/crash-point-replay.sh` 文件头，`grep -c` 各不为 0，量的）；crates-mutation-replay 不经服务分片那一趟只核账、不现判（`.claude/gate.d/crates-mutation-replay.sh:19`）；整道退出码四支（`.claude/gate.d/lib/stage-cells.sh` 第 52 到 56 行）；formal-verification 的项怎么切（`scripts/gate/verus-proof-check.py` 第 55 到 60 行）与闭包（第 151 行）；`exhaustive=` 缩域不判（`scripts/gate/admission.py` 第 163 到 164 行）。行号指针此刻都指得中。
- 参考件点名的文件头小节在不在：`lib_reuse_checks.py`「一」「二」「三」、`lib_function_units.py`「Python 闭包：import 时执行的模块级语句」「按尾巴读的登记」、`lib_verdict_library.py`「存放」、`merge-and-verify.sh`「崩溃域上限」、`lib_crash_point_replay_items.py`「指定范围的崩溃域上限」、`formal-verification.sh`「复用」「分片」、`broad-mutation.sh` 与 `broad-mutation.py`「分片」「判定库」「内存与限时」、`task_server.py`「提交时拒」、`task_server_lib.py`「分片」「挑机器」、`prune_server_client.py`「键空间」、`replay.sh`「判定库」、`doc-process-records.sh` 的 code-round-snapshot 格：都 `grep` 得到（量的）。只核了小节在，没逐句核小节内容与参考件的转述。
- 原件 `verification.md`「判定库只有一份」第 65 到 72 行、「所有任务都要多机可用」第 76 到 79 行：除一-1、一-2、一-3 外句子都在。「不经 `~` 与 `$HOME` 取」常驻删了，共用约束留着（`research/prompts/rules-compress-r1-merged/dot-claude/agent-common.md:29`），不算丢。
- 原件 `three-way-inference.md` 第 124 行（叫停轮登记判法）对技能第 35 到 41 行：四列、出处两种写法、三支收与四支不收、轮名全集、不许指的四类、派发闸与门禁共用判法、主 agent 核的三样，一句没少。
- 原件核查四步（第 126 到 131 行）对技能第 47 到 50 行：两条命令、`--root` 给倒推的树、三种「没判」的处置、复跑在副本里、删副本与留输出的时点、核不动的三种，一句没少。
- 原件整行抄一节（第 33 到 100 行）对合稿常驻第 15 到 30 行加技能第 14 到 20 行：做法、判据、两种漏引、七个 ⚠️ 都在；脚本的退出码 3 到 8 与弄坏开关名挪成「在文件头」，`scripts/kb/quote-kb.py` 文件头有退出码（`grep -c` 29 处，量的），弄坏开关名没逐个核。
- 原件 `implementation-workflow.md` 检查单八格对技能第 80 到 89 行：表逐格相同。

没核的：`.claude/skills/crash-test/commit-stages.md`、`.claude/skills/gate/staged-gate.md` 与这一组的指针对不对得上；合稿 `agent-common.md` 第 59 行之后「门禁只有一个」那一长条与原件逐句；`broad-mutation`、`formal-verification`、`qemu-device-streams`、`lkmm` 四道的文件头与参考件「门禁管哪一半」各行逐句。

## 我认为最脆弱的三处

1. 参考件写死的行号指针与同名节（一-4、二-1）：参考件靠「以文件头为准」省字，6 处写死行号、1 对同名标题。文件头一改或谁用 `@标题` 引，指针就错，现在没有门禁核这些指针。
2. 「两种跑法」的权威落点没收齐（二-2）：同一条判据在 `main-agent.md`、`verification.md`、`agent-common.md` 三处，入口改了判据而另两处没跟；这一类跨组的改动最容易在分组判的时候漏。
3. 三方开轮的落法（二-3、二-4）：定案第 2 条并进来之后，「没打中就停」「没打中一次不算」「打中先挂起」「打中写回代码」四句同在两份常驻里，先后与前提没写；这一轮自己就要照它判开不开第二轮。

## 这条腿自己的限度

- 全部是读文字对照，没跑任何项目代码，没建模型、没建副本；量的只有 `grep`、`sha256sum -c`、`awk` 取行。
- 脚本文件头只核了被指到的小节在不在与上面点名的那几句，没通读八百多行的 `crash-point-replay.sh` 文件头，参考件对它的转述可能还有对不上的。
- 用户定案十六条里只核了落在这一组文件的第 2、9、11 条与矛盾总表「验证-1」到「验证-9」；第 9 条两处都写成只有 `std`，落对了；第 11 条三份开头声明句都删了，落对了。
- 一次样本。没打中的那几格照规则一次不算，要支撑「没丢」得再抽一次。

## 要主 agent 跑的命令

无。这一轮没有要经任务服务跑的命令。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务。
- 格四只判到「推不出」，没去试材料员照合稿实际会不会跟到技能。
- 出路（四-1 那一条）是我提的，只按文字推、被攻过零轮。
