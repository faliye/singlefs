# 攻方腿：rules-compress-contradictions-r3（第三轮，最后一轮）

日期：2026-11-16。被判对象：矛盾总表第三版 `research/prompts/rules-compress-r3-merged/contradictions.md` 与它点名的第三版合稿树。开工时 `sha256sum -c` 核过：原件清单 `judged-sha256.txt` 55 份全部 OK，合稿树清单 `merged-tree-sha256.txt` 56 份在合稿树根下全部 OK（命令输出里没有一行不是 `: OK`）。没写模型，没有要复跑的数。

## 各格判定一览

| 格 | 判定 | 打中的条目 |
|---|---|---|
| 一 第二轮修补对不对 | 打中 4 处 | 用户定-23（落点句多出一条收窄，依据里三句不在用户原话里）；定义-11（弹窗闸的出处少了两样）；定义-15（拷 `.claude/gate.d` 的指令变成了前提）；用户定-20（标签里「改回原判据」那一半不实） |
| 二 修补引出的新问题 | 打中 3 处 | 代码轮停法指到一段自称只管设计推论轮的话；验证-17 与合稿-7 是同一处改动登了两行；验证-19 说明栏「停法只住一处」与 `implementation-workflow.md` 那一节重写停法相抵 |
| 三 能不能换上（漏登） | 打中 2 处 | `implementation-workflow.md:30` 的「那一轮只攻上一轮的修补」；`implementation-writer.md:46` 删掉「一并拷 `.claude/gate.d`」（定义-15 那一行没登这一半） |
| 抽查回查 | 456 处引文 455 处逐字在写的那一行 | 用户定-14 的〔〕不是原话（原件那一行另有「已有的」「（只缩不涨）」），表头 problems=0 不成立 |
| 用户定 21 到 26 | 5 条落得对，1 条落偏 | 用户定-23 |

一句结论：第三版**还不能**原样当换上时的对照表用。上面 4 处格一与 2 处漏登要先补（都是改总表与两三句合稿的活）；其余新补与改过的条目逐条核过，改得对。

## 格一：新补与改过的条目逐条

判定四种：改得对 / 改错了 / 其实改了判据而标签不对 / 核不动。依据栏引的脚本文件头都是这一刻工作区里的原文。

| 条目 | 判定 | 依据（现读） |
|---|---|---|
| 验证-10 | 改得对 | `.claude/gate.d/formal-verification.sh:51`：「toolchain、proof（整格）、table 各一项」；原件把钉全写成一项，门禁里那一项是 table |
| 验证-11 | 改得对 | `.claude/gate.d/broad-mutation.sh:30`：「不是服务分片起的这一道（gate.sh 整轮、手跑）与指定范围（--check gate-verdict）：不起驱动，只读 --summary」 |
| 验证-12 | 改得对；原件引文指错了句 | 格名表七格在 `.claude/gate.d/formal-verification.sh:23` 起的 same-origin-counter 一行；总表原件栏引的是 `.claude/rules/verification.md:126`（项的那一句），原件里只列六格的是 `.claude/rules/verification.md:135`：「红例钉表 `.claude/gate.d/e165-pinned-red-examples.tsv` 两格」那一行，引那一行才看得出少了哪一格 |
| 验证-13 | 改得对 | `.claude/gate.d/crash-point-replay.sh:340`、`:351`、`:359` 三处各写各的领卡没成怎么办，合稿 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:21` 三支照抄 |
| 验证-14 | 改得对 | 文件头 `.claude/gate.d/crash-point-replay.sh:249`：「挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没卡时打这一行后返回成功）都报「本次未跑」、不判红」；代码现读也是这样：`.claude/gate.d/crash-point-replay.sh:4404` 走 `crash_point_replay_report_not_run`，不判红。文件头格名表 `:17` 写「不为 0」，与 `:249` 自相矛盾，合稿照代码那一边写，对 |
| 验证-15 | 改得对 | `.claude/gate.d/lkmm.sh:14`：「复用：每份 litmus 两条判定」；`.claude/gate.d/qemu-device-streams.sh:5` 写两个对照必须判红 |
| 验证-16 | 改得对 | `scripts/crash/cpu-judging-sites-check.py:15`：「一处调用点算合规，有一样成立就行：」下列 ① 到 ⑤ 五样 |
| 验证-17 | 改得对；与合稿-7 重登 | `scripts/crash/crash-case-check.py:7` 的 modules 一行；同一处改动（合稿 `verification.md:23`）在合稿-7 已登，见格二 |
| 验证-18 | 改得对 | `.claude/gate.d/crash-point-replay.sh:256`：特性里有 gpu 时「不带 --test-threads」 |
| 验证-19 | 标签对；依据不实（核不动一节） | 依据栏写「用户定案第 23 条：开了第二轮的在第二轮派辩方腿」，用户原话那一行没有这一句，见下文「用户定-23」 |
| 定义-11 | 改错了（落得不全） | 见下文 |
| 定义-12 | 改得对 | `.claude/hooks/write-guard.sh:41` 写闸另判批次代号，合稿共用约束两处照它写 |
| 定义-13 | 改得对 | `.claude/hooks/timer-only-remind-guard.sh:5`：「主 agent 与子 agent 一样拒」；`remind.sh` 在项目代码闸的主 agent 名单里 |
| 定义-14 | 改得对 | `.claude/hooks/handback-guard.sh:31` ③ 点名 kb-scribe / experiment-runner |
| 定义-15 | 改了判据，标签对；同一处删掉了一条指令没登 | 见下文与「漏登」 |
| 定义-16 | 改得对 | `scripts/mutation/prove-red.sh:954` 对主工作区退 2；合稿调查员第 6 步改完另建副本证红 |
| 定义-17 | 改得对 | 四处原话逐字在合稿四个落点（自写脚本逐字核） |
| 入口-12、入口-13 | 改得对 | 合稿两行与原件 `CLAUDE.md:41`、`CLAUDE.md:43` 只差一个标点（句号改分号），标「只改措辞」照实 |
| 入口-14 | 改得对 | `scripts/gate/merge-and-verify.sh:13`：`--fingerprint-of` 那一种退 2 才是「主工作区变了」 |
| 入口-15 | 改得对 | 用户定案第 19 条，标「改了判据」照实 |
| 入口-16 | 改得对 | 拒算三类与 needs-attention、control check、`--check-facts` 各只住一处，指针都指得到 |
| 常驻-20 | 改得对 | 合稿 `show-me-test-by-review.md:7` 指到的「每一处会判红的分支配一个弄坏开关或一条变异」在合稿 `test-script-discipline.md:40` 在 |
| 常驻-21 | 改得对 | `task-server/task_server_lib.py:2412`：`SINGLEFS_HEAVY_TESTS` 退役，`=user-request` 是它的残句 |
| 常驻-22 | 改得对 | 原件 `.claude/rules/test-script-discipline.md:7` 点名整份上游文件，括号里不是节名 |
| 入口-4、合稿-7、合稿-10、常驻-10、常驻-11、常驻-12、常驻-14（表头点名更正标签的） | 改得对 | 逐条对过原件与合稿；常驻-12 的「服务停卡住的任务」有 `task-server/task_server_lib.py` 自证格 stalled-attempt-stopped 佐证 |
| 用户定-20（表头点名更正标签的） | 其实改了判据而标签不对（一半） | 见下文 |
| 用户定-21、22、24、26 | 改得对 | 落点句与 `records/2026-11-16-三方豁免取消与存量补轮.md` 第 199、200、212、214 行逐句对过 |
| 用户定-25 | 改得对（一处是转述） | 三处里 `CLAUDE-merged.md:21` 与 `main-agent.md:9` 逐字是原话；`agent-common.md:11` 把「自己直接改」写成「自己直接做」，意思没变，不是原话 |
| 用户定-23 | 改错了（落偏） | 见下文 |

## 打中的几处，逐处写清

### 一、用户定-23：落点句多出「那一轮只攻上一轮的修补」，依据栏三句不在用户原话里（格一、格三）

用户原话：`records/2026-11-16-三方豁免取消与存量补轮.md:211`：「打中的主 agent 现查坐实就改，改完再攻一轮，没新打中就收，最多三轮」。

合稿落点：`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30`：「改完再攻一轮，那一轮只攻上一轮的修补；没新打中就收，最多三轮。开了第二轮的，在第二轮派一条辩方腿复核已有的判决。」

- 「那一轮只攻上一轮的修补」用户原话里没有，原件里也没有：原件代码轮只写 `.claude/rules/implementation-workflow.md:10`：「打中的写回代码，再攻一轮」；原件三方规则给第二轮的攻击面比这宽，`.claude/rules/three-way-inference.md:135`：「第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方」。主 agent 这一题自己开第二轮时也不是只攻修补，`records/2026-11-16-三方豁免取消与存量补轮.md:206`：「判据四格：修补对不对、修补引出的新问题、第一轮没看到的、不采纳的那几条」。
- 做法因此不同：照原件，代码轮与治理文件轮的第二轮可以去攻第一轮没看到的地方；照第三版，第二、三轮的腿只许攻上一轮的修补，第一轮漏看的地方再没有一轮会去看。在合稿树里这一句是代码轮与治理文件轮唯一的攻击面规定（三方规则里分三轮攻击面那几句写在「开不开下一轮（设计推论轮）」里，只管设计推论轮）。
- 总表里没有登：`grep -c '只攻上一轮'` 对 `contradictions.md`、`dispositions-r2.md`、`references.md`、`revise-result.md` 四份都是 0（命令与输出见「抽查回查」一节末尾）。
- 依据栏把三句算在用户定案第 23 条名下，原话那一行没有：合稿-1 的依据 `research/prompts/rules-compress-r3-merged/contradictions.md:96`：「开了第二轮的在第二轮派辩方腿）」，验证-19 的依据 `research/prompts/rules-compress-r3-merged/contradictions.md:117`：「用户定案第 23 条：开了第二轮的在第二轮派辩方腿」；同一格里另两句「现查推翻的不改、不为它开轮」「第三轮又打中的改了收、写进交用户表、不开第四轮」也不在第 211 行。`grep -n '辩方\|第四轮\|现查推翻\|交用户表'` 对整份调度记录只命中第 131、206、217 行，三行都不是用户定案。「不开第四轮」「写进交用户表」出自主 agent 第二轮判决第三节，不是用户的话。
- 打中之后的四句：分不分辨——只打这一条落点，不牵连别的定案；被判的系统看不看得到——看得到，合稿那一句字面就写了；满足的是格一「定案落偏」那一分句（落点句多了一条用户没说的收窄）与格三「找到一处没登的不同」；改法：删掉「那一轮只攻上一轮的修补」或改成登记的原件口径，依据栏三句改标「主 agent 定（第二轮判决）」或交用户追认（只在我的读法上推的、被攻过零轮）。

### 二、用户定-20：标签里「改回原判据」那一半不实（格一，表头点名更正标签的一条）

总表 `research/prompts/rules-compress-r3-merged/contradictions.md:159`：「规则与共用约束样本进程那一条改回原判据」。表头 `research/prompts/rules-compress-r3-merged/contradictions.md:7` 把这一种定义成「改回原判据（前稿改错、这一版放回）」，即放回原件原话。

- 原件规则：`.claude/rules/command-safety-by-review.md:9`：「超时与结束时对整组发信号，先 TERM，过宽限再 KILL」；第三版：`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:9`：「不在循环里逐个发 `kill`（shell 里的循环一律拒」。
- 原件共用约束：`.claude/agent-common.md:72`：「超时与结束时在脚本里对整组发信号，跑完核没有留下进程；命令行里停进程照旧一次点名一个」；第三版：`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67`：「不对别的进程组发、不在 shell 循环里逐个发」，同一行另写「命令行上与脚本里同一个判法」。
- 两处都多了原件没有的限制（只许自己用新会话起的那一组、循环里不许逐个发、脚本与命令行同一个判法），这是用户定案第 20 条的新判据，`records/2026-11-16-三方豁免取消与存量补轮.md:194`：「命令行上与脚本里都只许对自己用新会话起的那一个子进程的进程组发信号，别的组、循环逐个发、先挑再发一律拒」。不是放回原话。照原件写的脚本作者，在 shell 循环里对几条样本各自的组逐个发 TERM 是许的；照第三版不许。
- 整条的总标签「改了判据」对，括号里拆出的「改回原判据」那一半要改成「改了判据（用户定 20）」。原件原话恢复的只有「对整组发信号」那半句；底下补充第 181 行「样本进程那一节与共用约束对应条目照原件恢复」同样不实。

### 三、定义-11：弹窗闸的出处少了两样，与文件头不是一回事（格一）

第三版：`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:104`：「出处（反引号里的路径或命令、文件:行号、产物行、量过的数）」。

文件头 `.claude/hooks/ask-user-claim-guard.sh:25` 与 `:26` 认的出处另有两样：research/results/ 下的文件名，与「实测」「量过」「产物」「输出」前后若干字以内带着一个数或路径（而且中间不隔断言词）；原件共用约束也列了，`.claude/agent-common.md:104`：「`research/results/` 下的文件、`name=` 开头的产物行」。第三版把后一样写成「量过的数」：既丢了「路径」，也丢了「旁边要有那四个词之一」的条件。

- 做法不同的两例（推的，按文件头判法推，没喂钩子）：问句里写「见 research/results/ 下某份产物的文件名，defer 一定为 0」，钩子放行，照第三版的列法这句没有出处，主 agent 会多加「推的」；写「第 3 次重跑读到 0，所以 defer 一定为 0」，照第三版「量过的数」读得出有出处，钩子不放（这句没有「实测」「量过」「产物」「输出」四个词），主 agent 被拒一次再改。
- 总表说明栏写「三方对照后改回」（`research/prompts/rules-compress-r3-merged/contradictions.md:118`：「三方对照后改回」），断言词那一半改回了，出处那一半没有。改法：出处照原件那五样抄全（只在我的读法上推的、被攻过零轮）。

### 四、定义-15：「一并拷 `.claude/gate.d`」的指令变成了前提（格一、格三）

原件第 3 步：`.claude/agents/implementation-writer.md:36`：「第 1d 步那几条要起的 `scripts/` 与 `.claude/gate.d` 一并拷」（第二版照留）。第三版：`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:46`：「第 1d 步那几条要起的 `scripts/` 与 `.claude/gate.d` 建副本时已照共用约束」……「的 `--with` 带上」。

- 「已……带上」读成前提，不是指令；而建副本那一步的两句都只在部分情形下带 `.claude/gate.d`：`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:15`：「新写崩溃枚举用例、放量分项或要拒算测试的另带 `--with-gate-tables` 与 `--with scripts=<为什么>`」；共用约束 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:35`：「要门禁表的加 `--with-gate-tables`」，同一句给带 `--item` 的门禁只点名 `--with scripts`、`--with .claude/scripts`、`--with .claude/singlefs-ai-sop` 三样，没有 `.claude/gate.d`。
- 不带那个参数时副本里没有 `.claude/gate.d`：`scripts/hosts/minimal-copy.py:54`：「不带 --with-gate-tables 也拷 .claude/gate.d/」是弄坏开关的说明，正常时不拷。
- 第 1d 步的命令在副本里起：`research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:34`：「命令原样可抄，`<根>` 填你改动所在的树」，起的是 `bash .claude/gate.d/formal-verification.sh`、`bash .claude/gate.d/crates-mutation-replay.sh` 这类。
- 做法不同的历史（推的，按文件头推，没建副本）：实现员只改一个已有的崩溃枚举用例或只加一条 formal-verification 的项（不是新用例、不是放量分项、不拒算），照第三版第 15 行与共用约束建副本不带 `--with-gate-tables`；第 3 步告诉它这些「已带上」；它把第 1d 步的命令写进报告，主 agent 在这份副本里交，`.claude/gate.d/` 下的门禁脚本不在、当场起不来，多一来回。照原件第 3 步，建副本时就被明令一并拷。
- 定义-15 那一行只登了「证红改在自己的副本里」，这一半没登。改法：第 46 行放回「一并拷」那半句，或在第 15 行把条件写成「第 1d 步要起门禁的另带 `--with-gate-tables`」（只在我的读法上推的、被攻过零轮）。

## 格二：修补引出的新问题

1. 代码轮停法指到一段自称只管设计推论轮的话。`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:29`：「两条腿都没打中就记没打中、这一题停，不开第二轮」，同一句前半写「第一轮的停法照同一份」开不开下一轮那一段；那一段标题是 `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`：「开不开下一轮（设计推论轮）」，而 `research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:43` 写治理文件轮与代码轮不归它管：「的治理文件轮不是设计推论轮」。读者照指针过去，会读到同一段里的「先挂起、三轮多数」与「第一轮结论不一致、无法收敛的开第二轮」，后者在代码轮那一节没有对应句。两处停法的字面也不一样：那一段要「两条腿结论一致、都没打中」，代码轮那一句只要「两条腿都没打中」。指针指得到、唯一，但读得出两种意思（照指过去那段的全文，还是只取它的停法）。
2. 验证-17 与合稿-7 是同一处改动登了两行：两行的合稿落点都是 `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:23` 同一个片段〔验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块〕，原件都是 `.claude/rules/verification.md:138` 同一句，标签都是「照脚本改」。总表 118 行里同一处改动住了两行，换上时按行对照会数重。
3. 验证-19 说明栏写「停法只在三方规则「开不开下一轮」住一处」，而代码轮与治理文件轮的停法在 `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:29` 又写了一遍（见第 1 条），「只住一处」不成立。

另记一处不算打中（原件就这样、第三版没改坏）：常驻-10 之后，规则写 `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8`：「`setsid [--wait] <命令> &` 起（不带 `-f`）」，共用约束写 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67`：「shell 用 `setsid --wait` 起」，又指过去说写法照规则。两处一松一严；原件也是一处 `setsid`、一处 `setsid --wait`，总表常驻-10 说明栏登了。

## 抽查回查

总表表头的回查（`research/prompts/rules-compress-r3-merged/contradictions.md:5`：「rows=118 users=26 refs_ok=455 problems=0」）用的脚本「取离写的行号最近的一处命中」，所以 refs_ok 不保证行号对。我另写了一份脚本（草稿目录 `refcheck2.py`，sha256 `5cd533542be7d0eb78fa3594d7e5a2c2cc45e2c6e81431526a91eec00ea8b7b9`），不借它的代码：逐行切总表，取每处 `文件:行〔片段〕`，原件侧对仓里的文件、合稿侧对合稿树（`.claude/` 换成 `dot-claude/`，`CLAUDE.md` 换成 `CLAUDE-merged.md`），只认**写的那一行**（区间就是区间里的行）逐字含片段。

```
$ python3 refcheck2.py > refcheck2.txt 2>&1; echo rc=$?
rc=0
$ cut -f3 refcheck2.txt | sort | uniq -c
    245 orig
    211 tree
$ cut -f6 refcheck2.txt | sort | uniq -c
    455 EXACT
      1 MISSING
```

结果（量的）：456 处引文，原件侧 245 处（含用户原话的 38 处）、合稿侧 211 处；455 处逐字落在写的那一行，行号一个不偏。对不上的 1 处：

- 用户定-14：总表〔`nproc` 存量：取并行度缺省的那几处登进 `.claude/heavy-thread-exempt` 逐批改〕，写的位置是调度记录第 149 到 151 行，三行里都没有这串字。原话在 `records/2026-11-16-三方豁免取消与存量补轮.md:151`：「取并行度缺省的那几处登进已有的 `.claude/heavy-thread-exempt`（只缩不涨）逐批改」。〔〕里删了「已有的」与「（只缩不涨）」，是摘改，不是原话；「只缩不涨」是那张豁免表的判据，丢了它读得出可以往表里加行。表头「引号里只放逐字原文」「problems=0」在这一处不成立。

派发要求的「不少于 40 处、原件侧与合稿侧各半」：从新补与改过的条目里按表中次序各取前 20 处（`sample40.py`，sha256 `d091afbe575095f6cbcae056ab73e674172768aac42caa2bd788020946c7b41a`），逐处打出片段在那一行里的上下文看过。40 处全部对上。脚本对常驻-10 与定义-11 两处的合稿侧报过「对不上」，原因是这两行原件侧与合稿侧引的是同一个 `文件:行`，脚本取了原件那一个片段去合稿里找；手核：合稿 `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8`：「`setsid [--wait] <命令> &` 起（不带 `-f`）」，合稿 `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:15`：「定位之后接着修（第 6 步）」，都对上。

| 条目 | 哪一侧 | 位置 | 结果 |
|---|---|---|---|
| 入口-4 | 原件侧 | `.claude/main-agent.md:7` | 对上 |
| 入口-4 | 原件侧 | `.claude/main-agent.md:124` | 对上 |
| 常驻-10 | 原件侧 | `.claude/rules/command-safety-by-review.md:8` | 对上 |
| 常驻-10 | 原件侧 | `.claude/agent-common.md:72` | 对上 |
| 常驻-11 | 原件侧 | `.claude/rules/command-safety-by-review.md:21` | 对上 |
| 常驻-11 | 原件侧 | `.claude/main-agent.md:76` | 对上 |
| 常驻-12 | 原件侧 | `.claude/rules/command-safety-by-review.md:30` | 对上 |
| 常驻-12 | 原件侧 | `.claude/agent-common.md:79` | 对上 |
| 常驻-13 | 原件侧 | `.claude/rules/command-safety-by-review.md:61` | 对上 |
| 常驻-13 | 原件侧 | `.claude/rules/implementation-workflow.md:86` | 对上 |
| 常驻-14 | 原件侧 | `.claude/rules/command-safety-by-review.md:41` | 对上 |
| 常驻-14 | 原件侧 | `.claude/main-agent.md:38` | 对上 |
| 定义-11 | 原件侧 | `.claude/agents/investigator.md:15` | 对上 |
| 合稿-7 | 原件侧 | `.claude/rules/verification.md:138` | 对上 |
| 合稿-10 | 原件侧 | `.claude/agent-common.md:22` | 对上 |
| 验证-10 | 原件侧 | `.claude/rules/verification.md:126` | 对上 |
| 验证-11 | 原件侧 | `.claude/rules/verification.md:136` | 对上 |
| 验证-12 | 原件侧 | `.claude/rules/verification.md:126` | 对上 |
| 验证-13 | 原件侧 | `.claude/rules/verification.md:39` | 对上 |
| 验证-14 | 原件侧 | `.claude/rules/verification.md:134` | 对上 |
| 入口-4 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:9` | 对上 |
| 常驻-10 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8` | 对上（脚本取错了同一行另一处片段，手核见下） |
| 常驻-11 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:18` | 对上 |
| 常驻-12 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:17` | 对上 |
| 常驻-13 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:40` | 对上 |
| 常驻-14 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:31` | 对上 |
| 定义-11 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:15` | 对上（脚本取错了同一行另一处片段，手核见下） |
| 合稿-7 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:23` | 对上 |
| 合稿-7 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:151` | 对上 |
| 合稿-10 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:17` | 对上 |
| 验证-10 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:138` | 对上 |
| 验证-11 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:149` | 对上 |
| 验证-12 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:170` | 对上 |
| 验证-13 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:21` | 对上 |
| 验证-14 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:146` | 对上 |
| 验证-15 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:42` | 对上 |
| 验证-15 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:158` | 对上 |
| 验证-16 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:25` | 对上 |
| 验证-16 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:154` | 对上 |
| 验证-17 | 合稿侧 | `research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:23` | 对上 |

「漏登」第一条的计数（量的）：

```
$ grep -c '只攻上一轮' contradictions.md dispositions-r2.md references.md revise-result.md; echo rc=$?
revise-result.md:0
dispositions-r2.md:0
contradictions.md:0
references.md:0
rc=1
```

## 漏登的清单（格三）

做法：把 `diff-from-r2.patch` 里落在合稿树（`CLAUDE-merged.md` 与 `dot-claude/`，不含总表与 `references.md`）的加行，凡带「不」「只」「一律」「除外」「先」「才」的，按合稿行号列出（草稿目录 `kwlines.py`，sha256 `1d479ae19097149845d362452083319d1216fe915a8b1ae50b70910d6472be59`），共 118 行（量的）；逐行对它的减行与原件，看做法变没变、总表有没有对应的一行。

归类（推的，逐行读过）：

| 类 | 行数与去处 |
|---|---|
| 上游指针从（节）改回「节」，节名逐字是上游今天的标题（主 agent 处置-8） | 多数；我另写 `ptrcheck.py` 把合稿树里 64 处上游「节」逐个对上游标题行，64 处都有对应的标题（3 处报错是脚本把「8. 正文只写现状，历史放文末的「历史版本」」里套着的引号切开了，手核 `.claude/singlefs-ai-sop/rules/kb-discipline.md:81` 有这一节） |
| 有对应行、判定见格一表 | 用户定-18、20、23、24、25、26，验证-10 到 19，定义-11 到 17，入口-12 到 16，常驻-12、20、21、22，合稿-1、5、9、10 |
| 做法变了、总表没登 | 下面两条 |
| 读取入口补全，做法没变 | `research/prompts/rules-compress-r3-merged/CLAUDE-merged.md:66` 参考件的读取时机加了「`.claude/main-agent.md` 派发表点名它的那一步之前」；参考件是合稿新建的文件，合稿-4 已登它的登记办法，这一处可不另登 |

做法变了、总表没登的两条：

1. `research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30`：「改完再攻一轮，那一轮只攻上一轮的修补」。原件与用户定案都没有这一收窄；代码轮与治理文件轮第二、三轮的攻击面从「修补本身 / 产生结论的方法 / 从未被看过的地方」缩成只攻修补。详见「打中的几处」第一节。
2. `research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:46`：原件第 3 步「第 1d 步那几条要起的 `scripts/` 与 `.claude/gate.d` 一并拷」的指令改成「已……带上」的前提，建副本那两句只在新用例、放量分项、拒算时带 `.claude/gate.d`。详见第四节。定义-15 那一行登了同一句里的另一半，这一半没登。

## 核不动的（原样交用户）

1. **用户定案第 23 条有没有包括「开了第二轮的，在第二轮派一条辩方腿」「现查推翻的不改、不为它开一轮」「第三轮又打中的写进交用户表」三句。** 总表合稿-1、验证-19 与用户定-23 都把这三句记在第 23 条名下；调度记录第 211 行用户原话里没有，整份调度记录 grep 不到这三句的用户原话。要判它，得问用户当时有没有在弹窗或对话里说过这三句（会话记录里找弹窗的原文）；有，就把原话补进调度记录、总表引那一行；没有，这三句改标「主 agent 定」，其中「在第二轮派辩方腿」比原件「三轮里至少一轮」更紧，要不要这样定交用户。
2. **派实现员证红的那条模板在副本里起 `prove-red.sh` 时会不会被它自己当成主工作区拒。** 这不是第三版改出来的（原件第 3a 步就这样写），顺带看到：`scripts/mutation/prove-red.sh:263` 把「这份脚本所在的仓根」当主工作区，`:953` 起 `--copy` 等于它就退 2；模板是 `task-server/task submit --cwd <副本> -- bash scripts/mutation/prove-red.sh --copy <副本> …`，在副本里起的是副本里那一份脚本，算出来的主工作区就是副本自己。要判它，得在一份带 `scripts/` 的副本里经服务真起一次这条命令看退出码（命令见下一节）。第三版把证红改到实现员自己的副本里（定义-15），这条模板走不走得通直接决定定义-15 落不落得了地。

## 要主 agent 跑的命令

判决不靠下面这条也能写（第一到第四节都是读文件就坐实的）；它只为核不动第 2 条。

1. 命令：`task-server/task submit --cwd <主 agent 自己认领的草稿目录>/prove-red-probe-copy -- bash scripts/mutation/prove-red.sh --copy <主 agent 自己认领的草稿目录>/prove-red-probe-copy singlefs-format <变异表里一条 singlefs-format 的行名>`。前一步由主 agent 直接跑（不经服务）`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <主 agent 自己认领的草稿目录>/prove-red-probe-copy --with-test-inputs --with scripts=核证红模板在副本里起的那一份脚本`。在哪个目录起：那份副本。看哪几行：`TASK_SERVER_RESULT` 的 `exit=`；退 2 且日志里有「--copy 指的是主工作区」就是核不动第 2 条成立；退 0 或红在点名的行上就是不成立。跑完副本照共用约束删。

## 结论：能不能当换上时的对照表用

不能原样用。逐字层面它几乎干净（456 处引文 455 处落在写的那一行），标签层面多数照实；挡着的是六处：用户定-23 的落点句多一条没人定过的收窄、依据栏三句没有用户原话撑着，用户定-20 括号里的「改回原判据」，定义-11 少两样出处，定义-15 丢了一并拷 `.claude/gate.d` 的指令，用户定-14 的〔〕不是原话，外加验证-17 与合稿-7 重登一处。前五处改完、重跑表头那条回查（改成只认写的那一行），就可以当对照表用。

什么现象会推翻它：调度记录之外找得到用户说过「开了第二轮的在第二轮派辩方腿」「第二轮只攻修补」的原话（推翻第一节的依据部分）；`.claude/agent-common.md`「建仓副本」或 `minimal-copy.py` 在不带 `--with-gate-tables` 时其实会拷 `.claude/gate.d`（推翻第四节）；弹窗闸文件头之外另有一处把出处收窄成「量过的数」的现行判法（推翻第三节）。

## 没打中的形状

- 格一：新补的二十五行（验证-10 到 19、定义-11 到 17、入口-12 到 16、常驻-20 到 22）逐行把依据栏点名的文件头段落现读一遍、对合稿那一句；验证-13、14、18 另对了 `crash-point-replay.sh` 的代码（第 4400 到 4404 行附近的零条判法）。除上面几处，没找到改错的。
- 用户定 21 到 26：每条拿调度记录第 199、200、211、212、213、214 行原话对合稿落点的每一处，逐句读。
- 表头点名更正标签的八条：每条把原件两处与合稿落点并排读，判标签是不是五种里对的那一种。
- 格二：查了新写的句子里每个「照 X」「见 X」指针是否指得到、是否唯一（`ptrcheck.py` 对上游 64 处；项目内的逐个 grep），找同一条判据住两处（只找到第 1、3 条那一处与常驻-10 的一松一严）。
- 格三：118 行带关键词的加行全部过了一遍（见上表）。
- 没碰的：第二轮两条腿都判改得对、第三版没动的条目不再逐条判（派发提示的范围）；它们的引文在全量回查里一起核过。

## 这条腿自己的限度

- 「做法因此不同」那几例都是按文件头与代码推的，没喂钩子、没建副本跑（第三、四节标了「推的」）。
- 用户原话只查了调度记录；会话记录与弹窗原文没读（不在给的材料里），所以第一节的依据部分放进了核不动。
- 回查脚本与 40 处抽样用的是同一份切表逻辑（`sample40.py` 读 `refcheck2.py` 的输出），两者不独立；独立的那一条路是表头的 `contra_build.py`，我没跑它（它在别的草稿目录，起它要经服务），只读了表头写的判法。
- 格三只按六个关键词筛加行，不带这几个字的改动行（例如只改名词的）没有逐行对原件。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往统一任务服务交任务（要跑的一条在上一节）；没写模型、没有判决要引的数。
- 没读禁读清单里的文件与目录。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/rules-compress-contradictions-r3-opus` 里只有我写的几份文本脚本与它们的输出（`refcheck.py`、`refcheck2.py`、`sample40.py`、`kwlines.py`、`ptrcheck.py` 与对应的 `.txt`），没建仓副本、编译目录与库拷贝，没有要删的。
